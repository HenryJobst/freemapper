//! ALKIS buildings and land use: the LGB OGC API Features service (dl-de/by-2-0), and inside
//! Berlin the Geoportal Berlin WFS (see [`berlin`]).

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher};
use crate::sources::berlin;
use crate::sources::region::{Region, regions_for};

pub const BUILDINGS_ENDPOINT: &str =
    "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/gebaeude_bauwerk/items";
const CRS_25833: &str = "http://www.opengis.net/def/crs/EPSG/0/25833";
/// Features per page; a 1 km tile holds about 800 buildings.
const PAGE_LIMIT: u32 = 10_000;
/// `rellage` value of building parts below ground.
const UNDERGROUND: &str = "Unter der Erdoberfläche";

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid ALKIS data: {message}")]
pub struct AlkisError {
    pub message: String,
}

pub(crate) fn invalid(message: impl Into<String>) -> AlkisError {
    AlkisError {
        message: message.into(),
    }
}

/// One `gebaeude_bauwerk` feature.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    /// ALKIS object id, e.g. `DEBBAL010007KdLbBL`.
    pub oid: String,
    /// `gebnutzbez`, e.g. `Gebäude` or `Bauteil`.
    pub kind: String,
    /// `funktion`, e.g. `Wohnhaus`.
    pub function: String,
    pub underground: bool,
    /// Date of `aktualit`.
    pub updated: Option<NaiveDate>,
    /// Footprint in EPSG:25833.
    pub footprint: geo_types::MultiPolygon<f64>,
}

/// Land use per parcel part (`nutzung_flurstueck`).
pub const LAND_USE_ENDPOINT: &str =
    "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/nutzung_flurstueck/items";

/// URL of the first page of buildings in `bbox` (EPSG:25833), rounded outward to whole metres.
pub fn buildings_url(bbox: &BoundingBox) -> String {
    collection_url(BUILDINGS_ENDPOINT, bbox)
}

/// URL of the first page of the features of a collection endpoint in `bbox`, rounded outward.
pub fn collection_url(endpoint: &str, bbox: &BoundingBox) -> String {
    format!(
        "{endpoint}?f=json&bbox={},{},{},{}&bbox-crs={CRS_25833}&crs={CRS_25833}&limit={PAGE_LIMIT}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// Downloads the buildings in `bbox` from every state service covering it (Brandenburg, Berlin)
/// and returns them as one GeoJSON FeatureCollection in the LGB schema.
pub fn fetch_buildings(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<String, FetchError> {
    merged(bbox, |region| match region {
        Region::Brandenburg => collection_features(fetcher, BUILDINGS_ENDPOINT, bbox),
        Region::Berlin => berlin::fetch_buildings(fetcher, bbox),
        // Saxony ALKIS follows with the GeoSN WFS.
        Region::Saxony => Ok(Vec::new()),
    })
}

/// Downloads the land use in `bbox` from every state service covering it, like
/// [`fetch_buildings`].
pub fn fetch_land_use(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<String, FetchError> {
    merged(bbox, |region| match region {
        Region::Brandenburg => collection_features(fetcher, LAND_USE_ENDPOINT, bbox),
        Region::Berlin => berlin::fetch_land_use(fetcher, bbox),
        Region::Saxony => Ok(Vec::new()),
    })
}

/// The features of every region covering `bbox` as one FeatureCollection; object ids of the
/// states never collide (`DEBBAL…`, `DEBE…`).
fn merged(
    bbox: &BoundingBox,
    mut fetch: impl FnMut(Region) -> Result<Vec<Value>, FetchError>,
) -> Result<String, FetchError> {
    let mut features = Vec::new();
    for region in regions_for(bbox) {
        features.extend(fetch(region)?);
    }
    Ok(feature_collection(features))
}

fn feature_collection(features: Vec<Value>) -> String {
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// Downloads all pages of a collection endpoint in `bbox` as one GeoJSON FeatureCollection.
/// Paging stops at an empty page or a `next` link seen before; a feature repeated on a later page
/// (same `oid`) is kept once.
pub fn fetch_collection(
    fetcher: &dyn Fetcher,
    endpoint: &str,
    bbox: &BoundingBox,
) -> Result<String, FetchError> {
    collection_features(fetcher, endpoint, bbox).map(feature_collection)
}

/// The features of all pages of a collection endpoint in `bbox`.
fn collection_features(
    fetcher: &dyn Fetcher,
    endpoint: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    let mut features: Vec<Value> = Vec::new();
    let mut seen_oids = std::collections::HashSet::new();
    let mut visited = Vec::new();
    let mut next = Some(collection_url(endpoint, bbox));
    while let Some(url) = next.take() {
        let bytes = fetcher.get(&url)?;
        let page: Value =
            serde_json::from_slice(&bytes).map_err(|e| invalid(format!("{url}: {e}")))?;
        let page_features = page["features"]
            .as_array()
            .ok_or_else(|| invalid(format!("{url}: no features")))?;
        let empty = page_features.is_empty();
        for feature in page_features {
            let new = match feature["properties"]["oid"].as_str() {
                Some(oid) => seen_oids.insert(oid.to_string()),
                None => true,
            };
            if new {
                features.push(feature.clone());
            }
        }
        visited.push(url);
        if empty {
            break;
        }
        next = page["links"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| link["rel"] == "next")
            .and_then(|link| link["href"].as_str())
            .map(str::to_string)
            .filter(|href| !visited.contains(href));
    }
    Ok(features)
}

/// Buildings read from a FeatureCollection, and the features that could not be read.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedBuildings {
    pub buildings: Vec<Building>,
    /// One line per skipped feature: its oid (if any) and why.
    pub skipped: Vec<String>,
}

/// Reads the buildings of a GeoJSON FeatureCollection as returned by [`fetch_buildings`].
/// A feature without oid or with an unusable geometry is skipped and listed; a document that is
/// not a FeatureCollection is an error.
pub fn parse_buildings(geojson: &str) -> Result<ParsedBuildings, AlkisError> {
    let collection: Value = serde_json::from_str(geojson).map_err(|e| invalid(e.to_string()))?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| invalid("no features"))?;
    let mut parsed = ParsedBuildings {
        buildings: Vec::new(),
        skipped: Vec::new(),
    };
    for feature in features {
        match parse_feature(feature) {
            Ok(building) => parsed.buildings.push(building),
            Err(e) => parsed.skipped.push(e.message),
        }
    }
    Ok(parsed)
}

/// The oid, polygon footprint and `aktualit` date every ALKIS feature has.
fn feature_basics(
    feature: &Value,
) -> Result<(String, geo_types::MultiPolygon<f64>, Option<NaiveDate>), AlkisError> {
    let properties = &feature["properties"];
    let oid = properties["oid"].as_str().unwrap_or_default().to_string();
    if oid.is_empty() {
        return Err(invalid("feature without oid"));
    }
    let geometry = serde_json::from_value::<geojson::Geometry>(feature["geometry"].clone())
        .map_err(|e| invalid(format!("{oid}: {e}")))?;
    let footprint = match geo_types::Geometry::<f64>::try_from(geometry)
        .map_err(|e| invalid(format!("{oid}: {e}")))?
    {
        geo_types::Geometry::Polygon(p) => geo_types::MultiPolygon(vec![p]),
        geo_types::Geometry::MultiPolygon(mp) => mp,
        other => return Err(invalid(format!("{oid}: unexpected geometry {other:?}"))),
    };
    let updated = properties["aktualit"]
        .as_str()
        .and_then(|t| t.get(..10))
        .and_then(|d| d.parse().ok());
    Ok((oid, footprint, updated))
}

fn parse_feature(feature: &Value) -> Result<Building, AlkisError> {
    let properties = &feature["properties"];
    let text = |key: &str| properties[key].as_str().unwrap_or_default().to_string();
    let (oid, footprint, updated) = feature_basics(feature)?;
    Ok(Building {
        kind: text("gebnutzbez"),
        function: text("funktion"),
        underground: properties["rellage"] == UNDERGROUND,
        updated,
        footprint,
        oid,
    })
}

/// One part of a parcel with its actual land use (`nutzung_flurstueck`).
#[derive(Debug, Clone, PartialEq)]
pub struct LandUse {
    pub oid: String,
    /// `nutzart`, e.g. `Wohnbaufläche`.
    pub nutzart: String,
    /// `bez`, a refinement such as `Bildung und Wissenschaft`.
    pub bez: Option<String>,
    pub updated: Option<NaiveDate>,
    /// Footprint in EPSG:25833.
    pub footprint: geo_types::MultiPolygon<f64>,
}

/// Reads the land use features of a FeatureCollection as returned by [`fetch_collection`];
/// features without oid or with an unusable geometry are skipped.
pub fn parse_land_use(geojson: &str) -> Result<Vec<LandUse>, AlkisError> {
    let collection: Value = serde_json::from_str(geojson).map_err(|e| invalid(e.to_string()))?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| invalid("no features"))?;
    Ok(features
        .iter()
        .filter_map(|feature| {
            let (oid, footprint, updated) = feature_basics(feature).ok()?;
            let properties = &feature["properties"];
            Some(LandUse {
                oid,
                nutzart: properties["nutzart"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                bez: properties["bez"].as_str().map(str::to_string),
                updated,
                footprint,
            })
        })
        .collect())
}

#[cfg(test)]
pub(crate) mod test_support {
    /// A GeoJSON feature in the shape the LGB service returns, with a square footprint.
    pub fn building_feature(oid: &str, kind: &str, function: &str, square: [f64; 3]) -> String {
        let [e, n, side] = square;
        format!(
            r#"{{"type":"Feature","id":"{oid}","geometry":{{"type":"MultiPolygon","coordinates":[[[[{e},{n}],[{e2},{n}],[{e2},{n2}],[{e},{n2}],[{e},{n}]]]]}},"properties":{{"oid":"{oid}","aktualit":"2023-05-04T10:27:03Z","gebnutzbez":"{kind}","funktion":"{function}","gmdschl":"12064472"}}}}"#,
            e2 = e + side,
            n2 = n + side
        )
    }

    /// A land use feature (`nutzung_flurstueck`) with a square footprint.
    pub fn land_use_feature(
        oid: &str,
        nutzart: &str,
        bez: Option<&str>,
        square: [f64; 3],
    ) -> String {
        let [e, n, side] = square;
        let bez = bez.map(|b| format!(r#","bez":"{b}""#)).unwrap_or_default();
        format!(
            r#"{{"type":"Feature","id":"{oid}","geometry":{{"type":"Polygon","coordinates":[[[{e},{n}],[{e2},{n}],[{e2},{n2}],[{e},{n2}],[{e},{n}]]]}},"properties":{{"oid":"{oid}","aktualit":"2025-09-30T06:47:45Z","nutzart":"{nutzart}"{bez},"flstkennz":"12454800900602______"}}}}"#,
            e2 = e + side,
            n2 = n + side
        )
    }

    /// A FeatureCollection page with an optional `next` link.
    pub fn page(features: &[String], next: Option<&str>) -> Vec<u8> {
        let links = match next {
            Some(url) => {
                format!(r#"[{{"rel":"self","href":"x"}},{{"rel":"next","href":"{url}"}}]"#)
            }
            None => r#"[{"rel":"self","href":"x"}]"#.to_string(),
        };
        format!(
            r#"{{"type":"FeatureCollection","features":[{}],"links":{links}}}"#,
            features.join(",")
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const BBOX: BoundingBox = BoundingBox {
        min_e: 421_490.0,
        min_n: 5_821_580.0,
        max_e: 421_736.0,
        max_n: 5_821_896.0,
    };

    #[test]
    fn buildings_url_has_bbox_and_crs() {
        let url = buildings_url(&BBOX);
        assert!(url.starts_with(BUILDINGS_ENDPOINT), "{url}");
        assert!(url.contains("bbox=421490,5821580,421736,5821896"), "{url}");
        assert!(
            url.contains("crs=http://www.opengis.net/def/crs/EPSG/0/25833"),
            "{url}"
        );
        assert!(
            url.contains("bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25833"),
            "{url}"
        );
        assert!(url.contains("limit=10000"), "{url}");
    }

    #[test]
    fn parse_live_like_features() {
        let school = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Allgemein bildende Schule",
            [421_600.0, 5_821_700.0, 20.0],
        );
        let garage = building_feature(
            "DEBBAL0100000002",
            "Bauteil",
            "Tiefgarage",
            [421_650.0, 5_821_750.0, 10.0],
        )
        .replace(
            r#""gmdschl""#,
            r#""rellage":"Unter der Erdoberfläche","gmdschl""#,
        );
        let text = String::from_utf8(page(&[school, garage], None)).unwrap();
        let buildings = parse_buildings(&text).unwrap().buildings;
        assert_eq!(buildings.len(), 2);
        assert_eq!(buildings[0].oid, "DEBBAL0100000001");
        assert_eq!(buildings[0].kind, "Gebäude");
        assert_eq!(buildings[0].function, "Allgemein bildende Schule");
        assert!(!buildings[0].underground);
        assert_eq!(buildings[0].updated, "2023-05-04".parse().ok());
        assert_eq!(buildings[0].footprint.0.len(), 1);
        assert!(buildings[1].underground);
    }

    #[test]
    fn fetch_follows_next_links() {
        let fetcher = MapFetcher::default();
        let second = "https://ogc-api.example/page2";
        let feature = |oid: &str| {
            building_feature(oid, "Gebäude", "Wohnhaus", [421_600.0, 5_821_700.0, 10.0])
        };
        fetcher.insert(&buildings_url(&BBOX), page(&[feature("A")], Some(second)));
        fetcher.insert(second, page(&[feature("B")], None));

        let text = fetch_buildings(&fetcher, &BBOX).unwrap();
        assert_eq!(fetcher.calls.get(), 2);
        let oids = oids(&text);
        assert_eq!(oids, ["A", "B"]);
    }

    fn wohnhaus(oid: &str) -> String {
        building_feature(oid, "Gebäude", "Wohnhaus", [421_600.0, 5_821_700.0, 10.0])
    }

    fn oids(text: &str) -> Vec<String> {
        parse_buildings(text)
            .unwrap()
            .buildings
            .into_iter()
            .map(|b| b.oid)
            .collect()
    }

    #[test]
    fn fetch_stops_on_repeated_next_link() {
        let fetcher = MapFetcher::default();
        let first = buildings_url(&BBOX);
        fetcher.insert(&first, page(&[wohnhaus("A")], Some(&first)));
        let text = fetch_buildings(&fetcher, &BBOX).unwrap();
        assert_eq!(fetcher.calls.get(), 1);
        assert_eq!(oids(&text), ["A"]);
    }

    #[test]
    fn fetch_stops_on_empty_page() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &buildings_url(&BBOX),
            page(&[wohnhaus("A")], Some("https://x/p2")),
        );
        fetcher.insert("https://x/p2", page(&[], Some("https://x/p3")));
        fetcher.insert("https://x/p3", page(&[wohnhaus("C")], None));
        let text = fetch_buildings(&fetcher, &BBOX).unwrap();
        assert_eq!(fetcher.calls.get(), 2);
        assert_eq!(oids(&text), ["A"]);
    }

    #[test]
    fn duplicate_oids_across_pages_are_merged() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &buildings_url(&BBOX),
            page(&[wohnhaus("A")], Some("https://x/p2")),
        );
        fetcher.insert("https://x/p2", page(&[wohnhaus("A"), wohnhaus("B")], None));
        let text = fetch_buildings(&fetcher, &BBOX).unwrap();
        assert_eq!(oids(&text), ["A", "B"]);
    }

    #[test]
    fn unusual_feature_is_skipped() {
        let no_geometry = r#"{"type":"Feature","geometry":null,"properties":{"oid":"X1","gebnutzbez":"Gebäude"}}"#;
        let no_oid = wohnhaus("X2").replace(r#""oid":"X2","#, "");
        let text = String::from_utf8(page(
            &[no_geometry.to_string(), no_oid, wohnhaus("A")],
            None,
        ))
        .unwrap();
        let parsed = parse_buildings(&text).unwrap();
        assert_eq!(parsed.buildings.len(), 1);
        assert_eq!(parsed.skipped.len(), 2, "{:?}", parsed.skipped);
        assert!(parsed.skipped[0].contains("X1"), "{:?}", parsed.skipped);
    }

    #[test]
    fn buildings_url_rounds_bbox_outward() {
        let fractional = BoundingBox {
            min_e: 421_490.6,
            min_n: 5_821_580.2,
            max_e: 421_736.1,
            max_n: 5_821_896.9,
        };
        assert!(buildings_url(&fractional).contains("bbox=421490,5821580,421737,5821897"));
    }

    #[test]
    fn land_use_url_uses_its_collection() {
        let url = collection_url(LAND_USE_ENDPOINT, &BBOX);
        assert!(url.starts_with(
            "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/nutzung_flurstueck/items?"
        ));
        assert!(url.contains("bbox=421490,5821580,421736,5821896"));
    }

    #[test]
    fn parse_land_use_features() {
        let school = land_use_feature(
            "DEBBAL0100000010",
            "Fläche besonderer funktionaler Prägung",
            Some("Bildung und Wissenschaft"),
            [421_600.0, 5_821_700.0, 50.0],
        );
        let homes = land_use_feature(
            "DEBBAL0100000011",
            "Wohnbaufläche",
            None,
            [421_700.0, 5_821_700.0, 30.0],
        );
        let broken =
            r#"{"type":"Feature","geometry":null,"properties":{"oid":"X","nutzart":"Wald"}}"#;
        let text = String::from_utf8(page(&[school, homes, broken.to_string()], None)).unwrap();
        let land_use = parse_land_use(&text).unwrap();
        assert_eq!(land_use.len(), 2, "the feature without geometry is skipped");
        assert_eq!(
            land_use[0].nutzart,
            "Fläche besonderer funktionaler Prägung"
        );
        assert_eq!(land_use[0].bez.as_deref(), Some("Bildung und Wissenschaft"));
        assert_eq!(land_use[1].bez, None);
        assert_eq!(land_use[1].updated, "2025-09-30".parse().ok());
        assert_eq!(land_use[1].footprint.0.len(), 1);
    }

    #[test]
    fn invalid_alkis_answer_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&buildings_url(&BBOX), b"<html>maintenance</html>".to_vec());
        let err = fetch_buildings(&fetcher, &BBOX).unwrap_err();
        assert!(matches!(err, FetchError::Alkis(_)), "{err:?}");
    }

    #[test]
    #[ignore = "network"]
    fn live_school_buildings() {
        let text = fetch_buildings(&crate::fetch::HttpFetcher::new(), &BBOX).unwrap();
        let buildings = parse_buildings(&text).unwrap().buildings;
        assert!(buildings.len() >= 10, "{} buildings", buildings.len());
        assert!(
            buildings
                .iter()
                .any(|b| b.function == "Allgemein bildende Schule")
        );
        assert!(buildings.iter().any(|b| b.underground));
    }

    /// Across the Berlin border at Schönefeld.
    const BORDER: BoundingBox = BoundingBox {
        min_e: 407_550.0,
        min_n: 5_799_224.0,
        max_e: 408_150.0,
        max_n: 5_799_824.0,
    };

    #[test]
    fn border_merges_both_sources() {
        let fetcher = MapFetcher::default();
        let lgb = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [407_600.0, 5_799_300.0, 10.0],
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[lgb], None));
        fetcher.insert_prefix(
            crate::sources::berlin::BUILDINGS_SERVICE,
            crate::sources::berlin::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings(&fetcher, &BORDER).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert_eq!(oids.len(), 5, "{oids:?}");
        assert_eq!(oids[0], "DEBBAL0100000001");
        assert!(
            oids[1..].iter().all(|oid| oid.starts_with("DEBE")),
            "{oids:?}"
        );
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn border_land_use_merges_both_sources() {
        let fetcher = MapFetcher::default();
        let lgb = land_use_feature(
            "DEBBAL0200000001",
            "Wald",
            None,
            [407_600.0, 5_799_300.0, 10.0],
        );
        fetcher.insert_prefix(LAND_USE_ENDPOINT, page(&[lgb], None));
        fetcher.insert_prefix(
            crate::sources::berlin::LAND_USE_SERVICE,
            crate::sources::berlin::test_support::LAND_USE
                .as_bytes()
                .to_vec(),
        );
        let land = parse_land_use(&fetch_land_use(&fetcher, &BORDER).unwrap()).unwrap();
        assert_eq!(land.len(), 7);
        assert_eq!(land[0].nutzart, "Wald");
        assert_eq!(land[1].nutzart, "Fläche besonderer funktionaler Prägung");
    }

    #[test]
    fn brandenburg_area_makes_no_berlin_request() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[], None));
        fetcher.insert_prefix(LAND_USE_ENDPOINT, page(&[], None));
        fetch_buildings(&fetcher, &BBOX).unwrap();
        fetch_land_use(&fetcher, &BBOX).unwrap();
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn berlin_area_makes_no_brandenburg_request() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            crate::sources::berlin::BUILDINGS_SERVICE,
            crate::sources::berlin::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        let school = BoundingBox {
            min_e: 392_700.0,
            min_n: 5_820_750.0,
            max_e: 392_900.0,
            max_n: 5_820_950.0,
        };
        let parsed = parse_buildings(&fetch_buildings(&fetcher, &school).unwrap()).unwrap();
        assert_eq!(parsed.buildings.len(), 4);
        assert_eq!(fetcher.calls.get(), 1);
    }
}
