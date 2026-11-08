//! ALKIS buildings and land use: the LGB OGC API Features service (dl-de/by-2-0), and inside
//! Berlin the Geoportal Berlin WFS (see [`berlin`]).

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher};
use crate::sources::region::{Region, regions_for};
use crate::sources::{alkis_wfs, berlin, lgv};

pub const BUILDINGS_ENDPOINT: &str =
    "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/gebaeude_bauwerk/items";
/// LAiV M-V's OGC API (the same LGB schema and spellings, EPSG:25833).
pub const MV_BUILDINGS_ENDPOINT: &str =
    "https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/gebaeude_bauwerk/items";
pub const MV_LAND_USE_ENDPOINT: &str =
    "https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/nutzung_flurstueck/items";
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

/// URL of the first page of the features of a collection endpoint in `bbox` (EPSG:25833),
/// rounded outward.
pub fn collection_url(endpoint: &str, bbox: &BoundingBox) -> String {
    collection_url_in(endpoint, bbox, Crs::Utm33)
}

/// URL of the first page of a collection endpoint in `bbox`, asked and answered in `crs`.
pub fn collection_url_in(endpoint: &str, bbox: &BoundingBox, crs: Crs) -> String {
    let crs = format!("http://www.opengis.net/def/crs/EPSG/0/{}", crs.epsg());
    format!(
        "{endpoint}?f=json&bbox={},{},{},{}&bbox-crs={crs}&crs={crs}&limit={PAGE_LIMIT}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// The area ALKIS is fetched for: its EPSG:25833 box, which decides the regions, and the map's
/// CRS with the box in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FetchArea {
    pub region_bbox: BoundingBox,
    pub crs: Crs,
    /// The area in `crs`.
    pub bbox: BoundingBox,
}

impl FetchArea {
    pub fn new(region_bbox: BoundingBox, crs: Crs) -> Self {
        Self {
            region_bbox,
            crs,
            bbox: bbox_in(&region_bbox, Crs::Utm33, crs),
        }
    }

    /// The area in `crs`: exact for the map's CRS and EPSG:25833, else the envelope of the
    /// region box's outline in `crs`.
    fn bbox_for(&self, crs: Crs) -> BoundingBox {
        if crs == self.crs {
            self.bbox
        } else {
            bbox_in(&self.region_bbox, Crs::Utm33, crs)
        }
    }
}

/// The envelope of `bbox` (in `from`) in `to`, from points along its outline, grown by 1 m.
fn bbox_in(bbox: &BoundingBox, from: Crs, to: Crs) -> BoundingBox {
    if from == to {
        return *bbox;
    }
    const STEPS: u32 = 8;
    let mut out: Option<BoundingBox> = None;
    for i in 0..=STEPS {
        let t = f64::from(i) / f64::from(STEPS);
        let e = bbox.min_e + t * (bbox.max_e - bbox.min_e);
        let n = bbox.min_n + t * (bbox.max_n - bbox.min_n);
        for (pe, pn) in [
            (e, bbox.min_n),
            (e, bbox.max_n),
            (bbox.min_e, n),
            (bbox.max_e, n),
        ] {
            let Some(p) = to.reproject(
                from,
                ProjectedPoint {
                    easting: pe,
                    northing: pn,
                },
            ) else {
                continue;
            };
            let b = out.get_or_insert(BoundingBox {
                min_e: p.easting,
                min_n: p.northing,
                max_e: p.easting,
                max_n: p.northing,
            });
            b.min_e = b.min_e.min(p.easting);
            b.min_n = b.min_n.min(p.northing);
            b.max_e = b.max_e.max(p.easting);
            b.max_n = b.max_n.max(p.northing);
        }
    }
    out.map_or(*bbox, |b| b.buffered(1.0))
}

/// Downloads the buildings in `bbox` (EPSG:25833) from every state service covering it, like
/// [`fetch_buildings_in`] for a map in EPSG:25833.
pub fn fetch_buildings(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<String, FetchError> {
    fetch_buildings_in(fetcher, &FetchArea::new(*bbox, Crs::Utm33))
}

/// Downloads the land use in `bbox` (EPSG:25833), like [`fetch_land_use_in`].
pub fn fetch_land_use(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<String, FetchError> {
    fetch_land_use_in(fetcher, &FetchArea::new(*bbox, Crs::Utm33))
}

/// Downloads the buildings of `area` from every state service covering it and returns them as
/// one GeoJSON FeatureCollection in the LGB schema and the map's CRS.
pub fn fetch_buildings_in(fetcher: &dyn Fetcher, area: &FetchArea) -> Result<String, FetchError> {
    merged(area, |region| {
        let in_crs = |crs: Crs| area.bbox_for(crs);
        Ok(match region {
            Region::Brandenburg => (
                collection_features(fetcher, BUILDINGS_ENDPOINT, &in_crs(Crs::Utm33))?,
                Crs::Utm33,
            ),
            Region::Berlin => (
                berlin::fetch_buildings(fetcher, &in_crs(Crs::Utm33))?,
                Crs::Utm33,
            ),
            Region::Saxony => wfs_buildings(fetcher, &alkis_wfs::GEOSN, &in_crs)?,
            Region::SaxonyAnhalt => wfs_buildings(fetcher, &alkis_wfs::LVERMGEO, &in_crs)?,
            Region::Hamburg => (
                ogc_api_or_wfs(
                    fetcher,
                    (
                        lgv::BUILDINGS_ENDPOINT,
                        &alkis_wfs::LGV,
                        AlkisLayer::Buildings,
                    ),
                    &in_crs(Crs::Utm32),
                    lgv::normalise_building,
                )?,
                Crs::Utm32,
            ),
            Region::Hessen => wfs_buildings(fetcher, &alkis_wfs::HVBG, &in_crs)?,
            Region::SchleswigHolstein => wfs_buildings(fetcher, &alkis_wfs::LVERMGEO_SH, &in_crs)?,
            Region::MecklenburgVorpommern => (
                ogc_api_or_wfs(
                    fetcher,
                    (
                        MV_BUILDINGS_ENDPOINT,
                        &alkis_wfs::LAIV_MV,
                        AlkisLayer::Buildings,
                    ),
                    &in_crs(Crs::Utm33),
                    std::convert::identity,
                )?,
                Crs::Utm33,
            ),
            Region::NordrheinWestfalen => {
                wfs_buildings(fetcher, &alkis_wfs::GEOBASIS_NRW, &in_crs)?
            }
        })
    })
}

/// Downloads the land use of `area`, like [`fetch_buildings_in`].
pub fn fetch_land_use_in(fetcher: &dyn Fetcher, area: &FetchArea) -> Result<String, FetchError> {
    merged(area, |region| {
        let in_crs = |crs: Crs| area.bbox_for(crs);
        Ok(match region {
            Region::Brandenburg => (
                collection_features(fetcher, LAND_USE_ENDPOINT, &in_crs(Crs::Utm33))?,
                Crs::Utm33,
            ),
            Region::Berlin => (
                berlin::fetch_land_use(fetcher, &in_crs(Crs::Utm33))?,
                Crs::Utm33,
            ),
            Region::Saxony => wfs_land_use(fetcher, &alkis_wfs::GEOSN, &in_crs)?,
            Region::SaxonyAnhalt => wfs_land_use(fetcher, &alkis_wfs::LVERMGEO, &in_crs)?,
            Region::Hamburg => (
                ogc_api_or_wfs(
                    fetcher,
                    (lgv::LAND_USE_ENDPOINT, &alkis_wfs::LGV, AlkisLayer::LandUse),
                    &in_crs(Crs::Utm32),
                    lgv::normalise_land_use,
                )?,
                Crs::Utm32,
            ),
            Region::Hessen => wfs_land_use(fetcher, &alkis_wfs::HVBG, &in_crs)?,
            Region::SchleswigHolstein => wfs_land_use(fetcher, &alkis_wfs::LVERMGEO_SH, &in_crs)?,
            Region::MecklenburgVorpommern => (
                ogc_api_or_wfs(
                    fetcher,
                    (
                        MV_LAND_USE_ENDPOINT,
                        &alkis_wfs::LAIV_MV,
                        AlkisLayer::LandUse,
                    ),
                    &in_crs(Crs::Utm33),
                    std::convert::identity,
                )?,
                Crs::Utm33,
            ),
            Region::NordrheinWestfalen => wfs_land_use(fetcher, &alkis_wfs::GEOBASIS_NRW, &in_crs)?,
        })
    })
}

type Features = (Vec<Value>, Crs);

/// Which ALKIS layer the WFS fallback of [`ogc_api_or_wfs`] reads.
#[derive(Clone, Copy)]
enum AlkisLayer {
    Buildings,
    LandUse,
}

/// The features of one collection in `bbox` (in the WFS service's CRS), normalised to the LGB
/// spellings: from the OGC API `endpoint`, or from the classic WFS (the same data) when the OGC
/// API fails. When both fail, the OGC API's error is reported. `layer` says which
/// of the service's feature types the WFS fallback reads; land use gets the service's `square_bez` rule, if it has one.
fn ogc_api_or_wfs(
    fetcher: &dyn Fetcher,
    (endpoint, wfs, layer): (&str, &alkis_wfs::WfsService, AlkisLayer),
    bbox: &BoundingBox,
    normalise: fn(Value) -> Value,
) -> Result<Vec<Value>, FetchError> {
    let features = collection_features_in(fetcher, endpoint, bbox, wfs.crs).or_else(|e| {
        match layer {
            AlkisLayer::LandUse => alkis_wfs::fetch_land_use(fetcher, wfs, bbox),
            AlkisLayer::Buildings => alkis_wfs::fetch_buildings(fetcher, wfs, bbox),
        }
        .map_err(|_| e)
    })?;
    Ok(features.into_iter().map(normalise).collect())
}

fn wfs_buildings(
    fetcher: &dyn Fetcher,
    service: &alkis_wfs::WfsService,
    in_crs: &dyn Fn(Crs) -> BoundingBox,
) -> Result<Features, FetchError> {
    let features = alkis_wfs::fetch_buildings(fetcher, service, &in_crs(service.crs))?;
    Ok((features, service.crs))
}

fn wfs_land_use(
    fetcher: &dyn Fetcher,
    service: &alkis_wfs::WfsService,
    in_crs: &dyn Fn(Crs) -> BoundingBox,
) -> Result<Features, FetchError> {
    let features = alkis_wfs::fetch_land_use(fetcher, service, &in_crs(service.crs))?;
    Ok((features, service.crs))
}

/// The features of every region covering `area` as one FeatureCollection in the map's CRS;
/// object ids of the states never collide (`DEBBAL…`, `DEBE…`, `DESN…`, `DEST…`, `DEHE…`, `DESH…`, `DEMV…`, `DENW…`).
fn merged(
    area: &FetchArea,
    mut fetch: impl FnMut(Region) -> Result<Features, FetchError>,
) -> Result<String, FetchError> {
    let mut features = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for region in regions_for(&area.region_bbox) {
        let (part, crs) = fetch(region)?;
        // An object on a state border may come from both services: keep it once.
        features.extend(
            part.into_iter()
                .filter(|f| {
                    f["properties"]["oid"]
                        .as_str()
                        .is_none_or(|oid| seen.insert(oid.to_string()))
                })
                .map(|f| reproject_feature(f, crs, area.crs)),
        );
    }
    Ok(feature_collection(features))
}

/// `feature` with its geometry's coordinates moved from `from` into `to`.
fn reproject_feature(mut feature: Value, from: Crs, to: Crs) -> Value {
    fn walk(v: &mut Value, from: Crs, to: Crs) {
        let Some(items) = v.as_array_mut() else {
            return;
        };
        if let [Value::Number(e), Value::Number(n), ..] = items.as_slice()
            && let (Some(e), Some(n)) = (e.as_f64(), n.as_f64())
        {
            if let Some(p) = to.reproject(
                from,
                ProjectedPoint {
                    easting: e,
                    northing: n,
                },
            ) {
                items[0] = Value::from(p.easting);
                items[1] = Value::from(p.northing);
            }
            return;
        }
        for item in items {
            walk(item, from, to);
        }
    }
    if from != to {
        walk(&mut feature["geometry"]["coordinates"], from, to);
    }
    feature
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
    collection_features_in(fetcher, endpoint, bbox, Crs::Utm33)
}

/// The features of all pages of a collection endpoint in `bbox`, in `crs`.
fn collection_features_in(
    fetcher: &dyn Fetcher,
    endpoint: &str,
    bbox: &BoundingBox,
    crs: Crs,
) -> Result<Vec<Value>, FetchError> {
    let mut features: Vec<Value> = Vec::new();
    let mut seen_oids = std::collections::HashSet::new();
    let mut visited = Vec::new();
    let mut next = Some(collection_url_in(endpoint, bbox, crs));
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
    fn first_state_wins_for_an_object_in_both_services() {
        // Brüssow: Brandenburg and Mecklenburg-Vorpommern.
        let area = FetchArea::new(
            BoundingBox {
                min_e: 436_900.0,
                min_n: 5_917_900.0,
                max_e: 437_100.0,
                max_n: 5_918_900.0,
            },
            Crs::Utm33,
        );
        let regions = regions_for(&area.region_bbox);
        assert!(regions.len() >= 2, "{regions:?}");
        let first = regions[0];
        let copy = |from: &str| -> Features {
            let feature = serde_json::json!({
                "type": "Feature", "id": from,
                "geometry": {"type": "Point", "coordinates": [437_000.0, 5_918_000.0]},
                "properties": {"oid": "SAME", "from": from}
            });
            (vec![feature], Crs::Utm33)
        };
        let text = merged(&area, |region| {
            Ok(copy(if region == first { "first" } else { "later" }))
        })
        .unwrap();
        let json: Value = serde_json::from_str(&text).unwrap();
        let features = json["features"].as_array().unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(features[0]["properties"]["from"], "first");
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
    fn border_merges_brandenburg_and_saxony() {
        // A vertex shared by the Brandenburg and Saxony boundaries (Lausitz).
        let lausitz = BoundingBox {
            min_e: 459_209.0,
            min_n: 5_709_717.0,
            max_e: 459_809.0,
            max_n: 5_710_317.0,
        };
        let fetcher = MapFetcher::default();
        let lgb = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [459_300.0, 5_710_100.0, 10.0],
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[lgb], None));
        fetcher.insert_prefix(
            alkis_wfs::GEOSN.url,
            alkis_wfs::test_support::BUILDINGS.as_bytes().to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings(&fetcher, &lausitz).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert_eq!(oids.len(), 4, "{oids:?}");
        assert_eq!(oids[0], "DEBBAL0100000001");
        assert!(
            oids[1..].iter().all(|oid| oid.starts_with("DESN")),
            "{oids:?}"
        );
    }

    #[test]
    fn border_features_are_reprojected_into_the_project_crs() {
        // A vertex shared by the Brandenburg and Saxony-Anhalt boundaries (Altmark/Prignitz),
        // mapped in zone 32 like Saxony-Anhalt.
        let region_bbox = BoundingBox {
            min_e: 271_597.0,
            min_n: 5_881_373.0,
            max_e: 272_197.0,
            max_n: 5_881_973.0,
        };
        let area = FetchArea::new(region_bbox, crate::crs::Crs::Utm32);
        assert!(
            (area.bbox.min_e - 700_000.0).abs() < 50_000.0,
            "{:?}",
            area.bbox
        );
        let fetcher = MapFetcher::default();
        let lgb = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [271_800.0, 5_881_600.0, 10.0],
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[lgb], None));
        fetcher.insert_prefix(
            alkis_wfs::LVERMGEO.url,
            alkis_wfs::test_support::ST_BUILDINGS.as_bytes().to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert_eq!(oids.len(), 4, "{oids:?}");
        // The LGB corner (25833) now lies in zone 32; the LVermGeo ones are unchanged.
        let corner = parsed.buildings[0].footprint.0[0].exterior().0[0];
        let want = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: 271_800.0,
                    northing: 5_881_600.0,
                },
            )
            .unwrap();
        assert!(
            (corner.x - want.easting).abs() < 0.001 && (corner.y - want.northing).abs() < 0.001,
            "{corner:?} vs {want:?}"
        );
        let latina = parsed.buildings[1].footprint.0[0].exterior().0[0];
        assert!((700_000.0..710_000.0).contains(&latina.x), "{latina:?}");
        // The zone-32 service was asked in zone 32.
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.starts_with(alkis_wfs::LVERMGEO.url) && u.contains("EPSG::25832"))
        );
    }

    #[test]
    fn hessen_alkis_is_fetched_from_the_hvbg_wfs() {
        // Lessing-Gymnasium Frankfurt (25833).
        let region_bbox = BoundingBox {
            min_e: 47_638.0,
            min_n: 5_571_540.0,
            max_e: 48_038.0,
            max_n: 5_571_940.0,
        };
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            alkis_wfs::HVBG.url,
            alkis_wfs::test_support::HE_BUILDINGS.as_bytes().to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DEHE")));
        assert!(parsed.buildings.iter().any(|b| b.underground));
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .all(|u| u.starts_with(alkis_wfs::HVBG.url) && u.contains("EPSG::25832"))
        );
    }

    #[test]
    fn cross_border_alkis_reads_both_states() {
        // A box over the Norderstedt / Hamburg-Langenhorn border (25832 E 565500 / N 5947700).
        let centre = Crs::Utm33
            .reproject(
                Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 565_500.0,
                    northing: 5_947_700.0,
                },
            )
            .unwrap();
        let region_bbox = BoundingBox {
            min_e: centre.easting - 400.0,
            min_n: centre.northing - 400.0,
            max_e: centre.easting + 400.0,
            max_n: centre.northing + 400.0,
        };
        assert_eq!(
            regions_for(&region_bbox),
            vec![Region::Hamburg, Region::SchleswigHolstein]
        );
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            crate::sources::lgv::BUILDINGS_ENDPOINT,
            crate::sources::lgv::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        fetcher.insert_prefix(
            alkis_wfs::LVERMGEO_SH.url,
            alkis_wfs::test_support::SH_BUILDINGS.as_bytes().to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert!(oids.iter().any(|o| o.starts_with("DEHH")), "{oids:?}");
        assert!(oids.iter().any(|o| o.starts_with("DESH")), "{oids:?}");
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "no duplicates: {oids:?}");
        // Both states are delivered in the map CRS (25832).
        for b in &parsed.buildings {
            let c = b.footprint.0[0].exterior().0[0];
            assert!((560_000.0..570_000.0).contains(&c.x), "{} {c:?}", b.oid);
        }
        fetcher.insert_prefix(
            crate::sources::lgv::LAND_USE_ENDPOINT,
            crate::sources::lgv::test_support::LAND_USE
                .as_bytes()
                .to_vec(),
        );
        fetcher.insert_prefix(
            alkis_wfs::LVERMGEO_SH.url,
            alkis_wfs::test_support::SH_LAND_USE.as_bytes().to_vec(),
        );
        let raw: Value =
            serde_json::from_str(&fetch_land_use_in(&fetcher, &area).unwrap()).unwrap();
        let features = raw["features"].as_array().unwrap();
        let ids: Vec<&str> = features
            .iter()
            .map(|f| f["properties"]["oid"].as_str().unwrap())
            .collect();
        assert!(ids.iter().any(|o| o.starts_with("DEHH")), "{ids:?}");
        assert!(ids.iter().any(|o| o.starts_with("DESH")), "{ids:?}");
        let unique: std::collections::HashSet<&&str> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "no duplicate land use: {ids:?}");
        let land = parse_land_use(&raw.to_string()).unwrap();
        for l in &land {
            let c = l.footprint.0[0].exterior().0[0];
            assert!((560_000.0..570_000.0).contains(&c.x), "{} {c:?}", l.oid);
        }
        let log = fetcher.log.borrow();
        assert!(
            log.iter()
                .any(|u| u.starts_with(alkis_wfs::LVERMGEO_SH.url) && u.contains("EPSG::25832"))
        );
    }

    /// The text of the first `<oid>` element of a WFS answer.
    fn first_oid(gml: &str) -> String {
        let start = gml.find("<oid>").unwrap() + "<oid>".len();
        gml[start..start + gml[start..].find("</oid>").unwrap()].to_string()
    }

    /// `gml` with the text of its first `<oid>` element replaced by `oid`.
    fn with_first_oid(gml: &str, oid: &str) -> String {
        let old = first_oid(gml);
        gml.replacen(
            &format!("<oid>{old}</oid>"),
            &format!("<oid>{oid}</oid>"),
            1,
        )
    }

    #[test]
    fn nrw_alkis_asks_the_geobasis_nrw_wfs_for_both_types() {
        // Hildegard-von-Bingen-Gymnasium, Cologne (25832 E 354700 / N 5642650), box in 25833.
        let c = Crs::Utm33
            .reproject(
                Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 354_700.0,
                    northing: 5_642_650.0,
                },
            )
            .unwrap();
        let region_bbox = BoundingBox {
            min_e: c.easting - 200.0,
            min_n: c.northing - 200.0,
            max_e: c.easting + 200.0,
            max_n: c.northing + 200.0,
        };
        assert_eq!(regions_for(&region_bbox), vec![Region::NordrheinWestfalen]);
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            alkis_wfs::GEOBASIS_NRW.url,
            alkis_wfs::test_support::NRW_BUILDINGS.as_bytes().to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        assert!(!parsed.buildings.is_empty());
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DENW")));
        {
            let log = fetcher.log.borrow();
            assert!(!log.is_empty());
            for url in log.iter() {
                assert!(
                    url.starts_with("https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht?"),
                    "{url}"
                );
                assert!(url.contains("TYPENAMES=ave:GebaeudeBauwerk"), "{url}");
                assert!(url.contains("SRSNAME=urn:ogc:def:crs:EPSG::25832"), "{url}");
            }
        }
        fetcher.log.borrow_mut().clear();
        fetcher.insert_prefix(
            alkis_wfs::GEOBASIS_NRW.url,
            alkis_wfs::test_support::NRW_LAND_USE.as_bytes().to_vec(),
        );
        let raw = fetch_land_use_in(&fetcher, &area).unwrap();
        {
            let log = fetcher.log.borrow();
            assert!(!log.is_empty());
            for url in log.iter() {
                assert!(
                    url.starts_with("https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht?"),
                    "{url}"
                );
                assert!(url.contains("TYPENAMES=ave:NutzungFlurstueck"), "{url}");
                assert!(url.contains("SRSNAME=urn:ogc:def:crs:EPSG::25832"), "{url}");
            }
        }
        let land = parse_land_use(&raw).unwrap();
        assert!(land.iter().all(|l| l.oid.starts_with("DENW")));
        // Squares without a type come back as candidates.
        assert!(land.iter().any(|l| l.nutzart == "Platz"
            && l.bez.as_deref() == Some("ohne Unterart (Nordrhein-Westfalen)")));
        assert!(land.iter().any(|l| l.nutzart == "Straßenverkehr"
            && l.bez.as_deref() == Some("Begleitfläche Straßenverkehr")));
    }

    #[test]
    fn hesse_border_reads_both_states() {
        // A box over the Nordrhein-Westfalen / Hesse border (25833 E 115329 / N 5736651).
        let region_bbox = BoundingBox {
            min_e: 115_329.0 - 400.0,
            min_n: 5_736_651.0 - 400.0,
            max_e: 115_329.0 + 400.0,
            max_n: 5_736_651.0 + 400.0,
        };
        assert_eq!(
            regions_for(&region_bbox),
            vec![Region::Hessen, Region::NordrheinWestfalen]
        );
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            alkis_wfs::HVBG.url,
            alkis_wfs::test_support::HE_BUILDINGS.as_bytes().to_vec(),
        );
        // A border object that both services deliver under the same oid.
        let shared = first_oid(alkis_wfs::test_support::HE_BUILDINGS);
        fetcher.insert_prefix(
            alkis_wfs::GEOBASIS_NRW.url,
            with_first_oid(alkis_wfs::test_support::NRW_BUILDINGS, &shared).into_bytes(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert!(oids.iter().any(|o| o.starts_with("DEHE")), "{oids:?}");
        assert!(oids.iter().any(|o| o.starts_with("DENW")), "{oids:?}");
        assert_eq!(
            oids.iter().filter(|o| **o == shared).count(),
            1,
            "shared object once: {oids:?}"
        );
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "no duplicates: {oids:?}");
        // Both services are asked and answered in the map CRS (25832).
        for b in &parsed.buildings {
            let c = b.footprint.0[0].exterior().0[0];
            assert!((350_000.0..500_000.0).contains(&c.x), "{} {c:?}", b.oid);
        }
        fetcher.insert_prefix(
            alkis_wfs::HVBG.url,
            alkis_wfs::test_support::HE_LAND_USE.as_bytes().to_vec(),
        );
        let shared_land = first_oid(alkis_wfs::test_support::HE_LAND_USE);
        fetcher.insert_prefix(
            alkis_wfs::GEOBASIS_NRW.url,
            with_first_oid(alkis_wfs::test_support::NRW_LAND_USE, &shared_land).into_bytes(),
        );
        let raw: Value =
            serde_json::from_str(&fetch_land_use_in(&fetcher, &area).unwrap()).unwrap();
        let ids: Vec<&str> = raw["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["properties"]["oid"].as_str().unwrap())
            .collect();
        assert!(ids.iter().any(|o| o.starts_with("DEHE")), "{ids:?}");
        assert!(ids.iter().any(|o| o.starts_with("DENW")), "{ids:?}");
        assert_eq!(
            ids.iter().filter(|o| **o == shared_land).count(),
            1,
            "shared land use once: {ids:?}"
        );
        let unique: std::collections::HashSet<&&str> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "no duplicate land use: {ids:?}");
        let log = fetcher.log.borrow();
        for service in [alkis_wfs::HVBG.url, alkis_wfs::GEOBASIS_NRW.url] {
            assert!(
                log.iter()
                    .any(|u| u.starts_with(service) && u.contains("EPSG::25832")),
                "{service}"
            );
        }
    }

    #[test]
    fn hamburg_collection_url_is_25832() {
        let bbox = BoundingBox {
            min_e: 564_950.4,
            min_n: 5_937_650.0,
            max_e: 565_350.0,
            max_n: 5_938_000.2,
        };
        let url = collection_url_in(crate::sources::lgv::BUILDINGS_ENDPOINT, &bbox, Crs::Utm32);
        assert!(
            url.starts_with(crate::sources::lgv::BUILDINGS_ENDPOINT),
            "{url}"
        );
        assert!(url.contains("bbox=564950,5937650,565350,5938001"), "{url}");
        assert!(
            url.contains("bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25832"),
            "{url}"
        );
        assert!(
            url.contains("&crs=http://www.opengis.net/def/crs/EPSG/0/25832"),
            "{url}"
        );
        assert_eq!(
            collection_url(BUILDINGS_ENDPOINT, &bbox),
            collection_url_in(BUILDINGS_ENDPOINT, &bbox, Crs::Utm33)
        );
    }

    /// The Gymnasium Eppendorf (25833 region box), mapped in zone 32.
    fn eppendorf_area() -> FetchArea {
        FetchArea::new(
            BoundingBox {
                min_e: 167_878.0,
                min_n: 5_948_879.0,
                max_e: 168_278.0,
                max_n: 5_949_279.0,
            },
            Crs::Utm32,
        )
    }

    /// A fetcher whose LGV OGC API resets every connection.
    fn ogc_api_down() -> MapFetcher {
        let fetcher = MapFetcher::default();
        for endpoint in [
            crate::sources::lgv::BUILDINGS_ENDPOINT,
            crate::sources::lgv::LAND_USE_ENDPOINT,
        ] {
            fetcher.flaky.borrow_mut().insert(endpoint.to_string(), 100);
        }
        fetcher
    }

    #[test]
    fn hamburg_alkis_falls_back_to_the_wfs() {
        let fetcher = ogc_api_down();
        for (type_name, body) in [
            (
                alkis_wfs::LGV.buildings_type,
                include_str!("../../../../tests/fixtures/hamburg-wfs-buildings.gml"),
            ),
            (
                alkis_wfs::LGV.land_use_type,
                include_str!("../../../../tests/fixtures/hamburg-wfs-land-use.gml"),
            ),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    alkis_wfs::LGV.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let area = eppendorf_area();
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        assert_eq!(parsed.buildings.len(), 15);
        assert!(
            parsed
                .buildings
                .iter()
                .all(|b| b.kind == "Gebäude" || b.kind == "Bauteil")
        );
        let land = parse_land_use(&fetch_land_use_in(&fetcher, &area).unwrap()).unwrap();
        assert_eq!(land.len(), 20);
        assert!(
            land.iter().any(|l| l.nutzart == "Straßenverkehr"),
            "normalised"
        );
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.starts_with(alkis_wfs::LGV.url) && u.contains("EPSG::25832"))
        );
    }

    #[test]
    fn hamburg_alkis_reports_the_ogc_api_error_when_both_fail() {
        let err = fetch_buildings_in(&ogc_api_down(), &eppendorf_area()).unwrap_err();
        assert!(
            err.to_string().contains("api.hamburg.de"),
            "the first error is reported: {err}"
        );
    }

    #[test]
    fn hamburg_alkis_is_fetched_normalised() {
        // Gymnasium Eppendorf (25833 region box), mapped in zone 32.
        let region_bbox = BoundingBox {
            min_e: 167_878.0,
            min_n: 5_948_879.0,
            max_e: 168_278.0,
            max_n: 5_949_279.0,
        };
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            crate::sources::lgv::BUILDINGS_ENDPOINT,
            crate::sources::lgv::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        fetcher.insert_prefix(
            crate::sources::lgv::LAND_USE_ENDPOINT,
            crate::sources::lgv::test_support::LAND_USE
                .as_bytes()
                .to_vec(),
        );
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        assert_eq!(parsed.buildings.len(), 4);
        assert_eq!(parsed.buildings[0].kind, "Gebäude");
        let corner = parsed.buildings[0].footprint.0[0].exterior().0[0];
        assert!(
            (560_000.0..570_000.0).contains(&corner.x),
            "kept in 25832: {corner:?}"
        );
        let land = parse_land_use(&fetch_land_use_in(&fetcher, &area).unwrap()).unwrap();
        assert_eq!(land[2].nutzart, "Wohnbaufläche");
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .all(|u| !u.contains("geobasis-bb")),
            "no LGB request"
        );
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

    const MV_BUILDINGS: &str =
        include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-buildings.json");
    const MV_LAND_USE: &str =
        include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-land-use.json");

    /// The Innerstaedtisches Gymnasium Rostock (25833).
    fn rostock_area() -> FetchArea {
        FetchArea::new(
            BoundingBox {
                min_e: 311_957.0,
                min_n: 5_996_167.0,
                max_e: 312_357.0,
                max_n: 5_996_467.0,
            },
            Crs::Utm33,
        )
    }

    #[test]
    fn mv_buildings_parse() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(MV_BUILDINGS_ENDPOINT, MV_BUILDINGS.as_bytes().to_vec());
        let parsed =
            parse_buildings(&fetch_buildings_in(&fetcher, &rostock_area()).unwrap()).unwrap();
        assert_eq!(parsed.buildings.len(), 30);
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DEMV")));
        assert!(parsed.buildings.iter().any(|b| b.kind == "Bauteil"));
        assert!(parsed.buildings.iter().any(|b| b.underground), "rellage");
        let log = fetcher.log.borrow();
        assert!(log[0].starts_with(MV_BUILDINGS_ENDPOINT), "{log:?}");
        assert!(log[0].contains("/25833&"), "{log:?}");
        assert!(log.iter().all(|u| !u.contains("geobasis-bb")), "{log:?}");
    }

    #[test]
    fn mv_land_use_parses() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(MV_LAND_USE_ENDPOINT, MV_LAND_USE.as_bytes().to_vec());
        let land = parse_land_use(&fetch_land_use_in(&fetcher, &rostock_area()).unwrap()).unwrap();
        assert_eq!(land.len(), 30);
        assert!(land.iter().any(|l| {
            l.nutzart == "Fläche besonderer funktionaler Prägung"
                && l.bez.as_deref() == Some("Bildung und Wissenschaft")
        }));
        assert!(land.iter().any(|l| l.nutzart == "Straßenverkehr"));
    }

    #[test]
    fn mv_alkis_falls_back_to_the_wfs() {
        let fetcher = MapFetcher::default();
        fetcher
            .flaky
            .borrow_mut()
            .insert(MV_BUILDINGS_ENDPOINT.to_string(), 100);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={}&",
                alkis_wfs::LAIV_MV.url,
                alkis_wfs::LAIV_MV.buildings_type
            ),
            include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-wfs-buildings.gml")
                .as_bytes()
                .to_vec(),
        );
        let parsed =
            parse_buildings(&fetch_buildings_in(&fetcher, &rostock_area()).unwrap()).unwrap();
        assert_eq!(parsed.buildings.len(), 25);
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DEMV")));
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.starts_with(alkis_wfs::LAIV_MV.url) && u.contains("EPSG::25833"))
        );
    }

    #[test]
    fn brussow_border_reads_brandenburg_and_mv() {
        let area = FetchArea::new(
            BoundingBox {
                min_e: 436_007.0,
                min_n: 5_917_677.0,
                max_e: 438_007.0,
                max_n: 5_919_677.0,
            },
            Crs::Utm33,
        );
        let fetcher = MapFetcher::default();
        let lgb = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [437_000.0, 5_917_800.0, 10.0],
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[lgb], None));
        fetcher.insert_prefix(MV_BUILDINGS_ENDPOINT, MV_BUILDINGS.as_bytes().to_vec());
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert!(oids.iter().any(|o| o.starts_with("DEBB")), "{oids:?}");
        assert!(oids.iter().any(|o| o.starts_with("DEMV")), "{oids:?}");
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "no duplicates: {oids:?}");
    }

    #[test]
    fn mv_land_use_falls_back_to_the_wfs() {
        let fetcher = MapFetcher::default();
        fetcher
            .flaky
            .borrow_mut()
            .insert(MV_LAND_USE_ENDPOINT.to_string(), 100);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={}&",
                alkis_wfs::LAIV_MV.url,
                alkis_wfs::LAIV_MV.land_use_type
            ),
            include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-wfs-land-use.gml")
                .as_bytes()
                .to_vec(),
        );
        let land = parse_land_use(&fetch_land_use_in(&fetcher, &rostock_area()).unwrap()).unwrap();
        assert_eq!(land.len(), 30);
        let has = |nutzart: &str, bez: Option<&str>| {
            land.iter()
                .any(|l| l.nutzart == nutzart && l.bez.as_deref() == bez)
        };
        assert!(has("Platz", Some("Parkplatz")));
        assert!(has("Straßenverkehr", Some("Begleitfläche Straßenverkehr")));
        assert!(has(
            "Fläche besonderer funktionaler Prägung",
            Some("Bildung und Wissenschaft")
        ));
        // Squares without `bez` stay as they are (the OGC API does the same).
        assert!(land.iter().all(|l| l.nutzart != "Platz"
            || l.bez.as_deref() != Some("ohne Unterart (Mecklenburg-Vorpommern)")));
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.starts_with(alkis_wfs::LAIV_MV.url) && u.contains("EPSG::25833"))
        );
    }

    #[test]
    fn border_object_returned_by_both_states_is_kept_once() {
        let area = FetchArea::new(
            BoundingBox {
                min_e: 436_007.0,
                min_n: 5_917_677.0,
                max_e: 438_007.0,
                max_n: 5_919_677.0,
            },
            Crs::Utm33,
        );
        let fetcher = MapFetcher::default();
        let mv: Value = serde_json::from_str(MV_BUILDINGS).unwrap();
        let shared = mv["features"][0].to_string();
        let own = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [437_000.0, 5_917_800.0, 10.0],
        );
        // Brandenburg's service also returns the object on the border.
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[own, shared], None));
        fetcher.insert_prefix(MV_BUILDINGS_ENDPOINT, MV_BUILDINGS.as_bytes().to_vec());
        let parsed = parse_buildings(&fetch_buildings_in(&fetcher, &area).unwrap()).unwrap();
        let oids: Vec<&str> = parsed.buildings.iter().map(|b| b.oid.as_str()).collect();
        assert_eq!(oids.len(), 31, "{oids:?}");
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "{oids:?}");
    }
}
