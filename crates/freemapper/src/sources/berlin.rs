//! ALKIS buildings and land use from the Geoportal Berlin WFS (dl-de/zero-2-0).
//!
//! Features are normalised into the LGB property schema (`oid`, `gebnutzbez`, `funktion`,
//! `rellage`, `nutzart`, `bez`), so the stored files and the build rules stay the same for both
//! states.

use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::project::MapKind;
use crate::sources::alkis::invalid;
use crate::sources::region::{Region, regions_for};

pub const BUILDINGS_SERVICE: &str = "https://gdi.berlin.de/services/wfs/alkis_gebaeude";
pub const BUILDINGS_TYPE: &str = "alkis_gebaeude:gebaeude";
pub const LAND_USE_SERVICE: &str = "https://gdi.berlin.de/services/wfs/alkis";
pub const LAND_USE_TYPE: &str = "alkis:tatsaechlichenutzungflaechen";
/// Features per page; a 1 km tile in the city centre holds about 1100 buildings.
const PAGE_COUNT: u32 = 5000;
/// `bez` of a Berlin square: Berlin never records what a square is used for, so it stays a
/// candidate instead of becoming a paved map object.
pub const SQUARE_WITHOUT_FUNCTION: &str = "ohne Unterart (Berlin)";

/// AAA class (`bezeich`) → LGB `nutzart`.
const LAND_USE_CLASSES: [(&str, &str); 19] = [
    ("AX_Wohnbauflaeche", "Wohnbaufläche"),
    (
        "AX_IndustrieUndGewerbeflaeche",
        "Industrie- und Gewerbefläche",
    ),
    ("AX_FlaecheGemischterNutzung", "Fläche gemischter Nutzung"),
    (
        "AX_FlaecheBesondererFunktionalerPraegung",
        "Fläche besonderer funktionaler Prägung",
    ),
    (
        "AX_SportFreizeitUndErholungsflaeche",
        "Sport-, Freizeit- und Erholungsfläche",
    ),
    ("AX_Friedhof", "Friedhof"),
    ("AX_Strassenverkehr", "Straßenverkehr"),
    ("AX_Weg", "Weg"),
    ("AX_Platz", "Platz"),
    ("AX_Bahnverkehr", "Bahnverkehr"),
    ("AX_Flugverkehr", "Flugverkehr"),
    ("AX_Schiffsverkehr", "Schiffsverkehr"),
    ("AX_Wald", "Wald"),
    ("AX_Gehoelz", "Gehölz"),
    ("AX_Landwirtschaft", "Landwirtschaft"),
    ("AX_Moor", "Moor"),
    ("AX_Fliessgewaesser", "Fließgewässer"),
    ("AX_StehendesGewaesser", "Stehendes Gewässer"),
    ("AX_Hafenbecken", "Hafenbecken"),
];

/// One extra Berlin layer: a WFS feature type stored as `data/berlin/<file>.geojson`.
#[derive(Debug, PartialEq)]
pub struct ExtraLayer {
    /// Stored file name without the extension; also the `berlin:layer` value of its candidates.
    pub file: &'static str,
    pub service: &'static str,
    pub type_name: &'static str,
    /// The map kinds that use the layer.
    pub kinds: &'static [MapKind],
}

const BOTH_KINDS: &[MapKind] = &[MapKind::Sprint, MapKind::Schoolyard];
const TREE_SERVICE: &str = "https://gdi.berlin.de/services/wfs/baumbestand";

/// The extra layers, in fetch order (ALKIS structures, relief and hedges, tree cadastre).
pub const EXTRA_LAYERS: [ExtraLayer; 6] = [
    ExtraLayer {
        file: "bauwerke-flaechen",
        service: LAND_USE_SERVICE,
        type_name: "alkis:bauwerkeflaechen",
        kinds: BOTH_KINDS,
    },
    ExtraLayer {
        file: "bauwerke-linien",
        service: LAND_USE_SERVICE,
        type_name: "alkis:bauwerkelinien",
        kinds: BOTH_KINDS,
    },
    ExtraLayer {
        file: "relief",
        service: LAND_USE_SERVICE,
        type_name: "alkis:relief",
        kinds: &[MapKind::Sprint],
    },
    ExtraLayer {
        file: "vegetation-linien",
        service: LAND_USE_SERVICE,
        type_name: "alkis:gewaesservegetationlinien",
        kinds: BOTH_KINDS,
    },
    ExtraLayer {
        file: "strassenbaeume",
        service: TREE_SERVICE,
        type_name: "baumbestand:strassenbaeume",
        kinds: BOTH_KINDS,
    },
    ExtraLayer {
        file: "anlagenbaeume",
        service: TREE_SERVICE,
        type_name: "baumbestand:anlagenbaeume",
        kinds: BOTH_KINDS,
    },
];

/// The extra layers `kind` uses.
pub fn extra_layers(kind: MapKind) -> impl Iterator<Item = &'static ExtraLayer> {
    EXTRA_LAYERS.iter().filter(move |l| l.kinds.contains(&kind))
}

/// One extra layer as fetched; an empty `features` is a layer without objects in the area.
#[derive(Debug)]
pub struct FetchedLayer {
    pub layer: &'static ExtraLayer,
    /// The features as served (EPSG:25833), or why the layer could not be fetched.
    pub features: Result<Vec<Value>, FetchError>,
}

/// Whether `region_bbox` (EPSG:25833) needs the Berlin services: the test of the Berlin ALKIS.
pub fn near_berlin(region_bbox: &BoundingBox) -> bool {
    regions_for(region_bbox).contains(&Region::Berlin)
}

/// The extra layers of `kind` in `region_bbox` (EPSG:25833), one request series per layer; none
/// at all, and no request, when the area is not near Berlin. A layer that fails does not stop
/// the others.
pub fn fetch_extra_layers(
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
) -> Vec<FetchedLayer> {
    if !near_berlin(region_bbox) {
        return Vec::new();
    }
    extra_layers(kind)
        .map(|layer| FetchedLayer {
            layer,
            features: fetch_wfs(fetcher, layer.service, layer.type_name, region_bbox),
        })
        .collect()
}

/// `features` as the GeoJSON FeatureCollection stored for a layer (an empty one for no feature).
pub fn layer_to_geojson(features: &[Value]) -> String {
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// The features of a stored layer file.
pub fn parse_layer(text: &str) -> Result<Vec<Value>, serde_json::Error> {
    let collection: Value = serde_json::from_str(text)?;
    Ok(collection["features"]
        .as_array()
        .cloned()
        .unwrap_or_default())
}

/// URL of the first page of `type_name` features of `service` in `bbox`, rounded outward.
pub fn wfs_url(service: &str, type_name: &str, bbox: &BoundingBox) -> String {
    format!(
        "{service}?service=WFS&version=2.0.0&request=GetFeature&typeNames={type_name}\
         &outputFormat=application/json&count={PAGE_COUNT}&bbox={},{},{},{},urn:ogc:def:crs:EPSG::25833",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// All features of `type_name` in `bbox`, following `links[rel=next]`; a feature repeated on a
/// later page (same GeoServer `id`; not every layer has a `uuid`) is kept once.
pub fn fetch_wfs(
    fetcher: &dyn Fetcher,
    service: &str,
    type_name: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    let mut seen = HashSet::new();
    let mut visited = Vec::new();
    let mut next = Some(wfs_url(service, type_name, bbox));
    while let Some(url) = next.take() {
        let page = fetch_page(fetcher, &url)?;
        let page_features = page["features"].as_array().cloned().unwrap_or_default();
        let empty = page_features.is_empty();
        for feature in page_features {
            let new = match feature["id"].as_str() {
                Some(id) => seen.insert(id.to_string()),
                None => true,
            };
            if new {
                features.push(feature);
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

/// One page as a GeoJSON FeatureCollection. Server errors, resets and answers that are not a
/// FeatureCollection (GeoServer sends exception reports as XML with status 200) are retried;
/// a missing service is not.
fn fetch_page(fetcher: &dyn Fetcher, url: &str) -> Result<Value, FetchError> {
    get_with_retries(fetcher, url, |bytes| {
        serde_json::from_slice::<Value>(bytes)
            .ok()
            .filter(|page| page["type"] == "FeatureCollection" && page["features"].is_array())
            .ok_or_else(|| {
                let start = String::from_utf8_lossy(&bytes[..bytes.len().min(80)]).into_owned();
                FetchError::Alkis(invalid(format!(
                    "{url}: not a GeoJSON FeatureCollection: {start}"
                )))
            })
    })
}

/// A Berlin building feature with LGB properties.
pub fn normalise_building(feature: &Value) -> Value {
    let source = &feature["properties"];
    let kind = match source["bezeich"].as_str() {
        Some("AX_Gebaeude") => "Gebäude",
        Some("AX_Bauteil") => "Bauteil",
        Some(other) => other,
        None => "",
    };
    let mut properties = Map::new();
    properties.insert("oid".into(), source["uuid"].clone());
    properties.insert("gebnutzbez".into(), kind.into());
    properties.insert("funktion".into(), source["bezgfk"].clone());
    properties.insert("rellage".into(), source["bezofl"].clone());
    // The kind of a building part (`Durchfahrt im Gebäude`, `Arkade`, …); the build ignores it,
    // the Berlin rules read it.
    if let Some(bauart) = source["bezbat"].as_str() {
        properties.insert("bauart".into(), bauart.into());
    }
    with_properties(feature, properties)
}

/// A Berlin land use feature with LGB properties.
pub fn normalise_land_use(feature: &Value) -> Value {
    let source = &feature["properties"];
    let class = source["bezeich"].as_str().unwrap_or_default();
    let nutzart = LAND_USE_CLASSES
        .iter()
        .find(|(aaa, _)| *aaa == class)
        .map_or(class, |(_, name)| name);
    let bez = source["bezfkt"]
        .as_str()
        .filter(|b| !b.is_empty())
        .or((nutzart == "Platz").then_some(SQUARE_WITHOUT_FUNCTION));
    let mut properties = Map::new();
    properties.insert("oid".into(), source["uuid"].clone());
    properties.insert("nutzart".into(), nutzart.into());
    if let Some(bez) = bez {
        properties.insert("bez".into(), bez.into());
    }
    with_properties(feature, properties)
}

/// `feature` with its id and geometry and the given properties.
fn with_properties(feature: &Value, properties: Map<String, Value>) -> Value {
    serde_json::json!({
        "type": "Feature",
        "id": properties["oid"],
        "geometry": feature["geometry"],
        "properties": properties,
    })
}

/// The Berlin buildings in `bbox`, normalised.
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, bbox)?
        .iter()
        .map(normalise_building)
        .collect())
}

/// The Berlin land use in `bbox`, normalised.
pub fn fetch_land_use(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, LAND_USE_SERVICE, LAND_USE_TYPE, bbox)?
        .iter()
        .map(normalise_land_use)
        .collect())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/berlin-buildings.json");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/berlin-land-use.json");
    pub const BAUWERKE_FLAECHEN: &str =
        include_str!("../../../../tests/fixtures/berlin-bauwerke-flaechen.json");
    pub const BAUWERKE_LINIEN: &str =
        include_str!("../../../../tests/fixtures/berlin-bauwerke-linien.json");
    pub const RELIEF: &str = include_str!("../../../../tests/fixtures/berlin-relief.json");
    pub const VEGETATION_LINIEN: &str =
        include_str!("../../../../tests/fixtures/berlin-vegetation-linien.json");
    pub const STRASSENBAEUME: &str =
        include_str!("../../../../tests/fixtures/berlin-strassenbaeume.json");
    pub const ANLAGENBAEUME: &str =
        include_str!("../../../../tests/fixtures/berlin-anlagenbaeume.json");
    pub const BUILDING_PARTS: &str =
        include_str!("../../../../tests/fixtures/berlin-building-parts.json");
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::project::MapKind;
    use crate::sources::alkis::{parse_buildings, parse_land_use};

    /// Helene-Haeusler-Schule, Berlin-Prenzlauer Berg.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 392_700.4,
        min_n: 5_820_750.0,
        max_e: 392_900.0,
        max_n: 5_820_950.6,
    };

    fn features(collection: &str) -> Vec<Value> {
        serde_json::from_str::<Value>(collection).unwrap()["features"]
            .as_array()
            .unwrap()
            .clone()
    }

    fn collection(features: &[Value]) -> String {
        serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
    }

    #[test]
    fn url_matches_the_wfs() {
        assert_eq!(
            wfs_url(BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/alkis_gebaeude?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=alkis_gebaeude:gebaeude&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
    }

    #[test]
    fn buildings_normalise_to_lgb_schema() {
        let normalised: Vec<Value> = features(BUILDINGS).iter().map(normalise_building).collect();
        let parsed = parse_buildings(&collection(&normalised)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        let by_oid = |oid: &str| parsed.buildings.iter().find(|b| b.oid == oid).unwrap();

        let gym = by_oid("DEBE03YY600004AR");
        assert_eq!(gym.kind, "Gebäude");
        assert_eq!(gym.function, "Allgemein bildende Schule");
        assert!(!gym.underground);
        assert_eq!(gym.updated, None);

        let part = by_oid("DEBE00YY13U0001t");
        assert_eq!(part.kind, "Bauteil");
        assert_eq!(part.function, "");

        assert!(by_oid("DEBE00YY2A80000K").underground, "Tiefgarage");
    }

    #[test]
    fn land_use_classes_map_to_nutzart() {
        let normalised: Vec<Value> = features(LAND_USE).iter().map(normalise_land_use).collect();
        let land = parse_land_use(&collection(&normalised)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                (
                    "Fläche besonderer funktionaler Prägung",
                    Some("Öffentliche Zwecke")
                ),
                ("Platz", Some(SQUARE_WITHOUT_FUNCTION)),
                ("Straßenverkehr", Some("Fußgängerzone")),
                ("Wohnbaufläche", None),
                ("Weg", None),
                ("Sport-, Freizeit- und Erholungsfläche", Some("Grünanlage")),
            ]
        );
        assert_eq!(land[0].oid, "DEBE01YYK0002J1d");
    }

    #[test]
    fn every_class_has_a_name() {
        for (class, nutzart) in LAND_USE_CLASSES {
            let feature = serde_json::json!({
                "type": "Feature",
                "geometry": null,
                "properties": { "uuid": "DEBE1", "bezeich": class, "bezfkt": "" }
            });
            assert_eq!(
                normalise_land_use(&feature)["properties"]["nutzart"],
                nutzart
            );
        }
    }

    #[test]
    fn unknown_class_keeps_its_aaa_name() {
        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": null,
            "properties": { "uuid": "DEBE1", "bezeich": "AX_Heide", "bezfkt": null }
        });
        let normalised = normalise_land_use(&feature);
        assert_eq!(normalised["properties"]["nutzart"], "AX_Heide");
        assert_eq!(normalised["properties"]["bez"], Value::Null);
    }

    #[test]
    fn berlin_squares_stay_candidates() {
        let square = features(LAND_USE)
            .into_iter()
            .find(|f| f["properties"]["bezeich"] == "AX_Platz")
            .unwrap();
        let normalised = normalise_land_use(&square);
        assert_eq!(normalised["properties"]["bez"], SQUARE_WITHOUT_FUNCTION);
    }

    /// Strausberg, Brandenburg, far from the Berlin border.
    const STRAUSBERG: BoundingBox = BoundingBox {
        min_e: 421_490.0,
        min_n: 5_821_580.0,
        max_e: 421_736.0,
        max_n: 5_821_896.0,
    };

    /// Across the Berlin border at Schönefeld.
    const BORDER: BoundingBox = BoundingBox {
        min_e: 407_550.0,
        min_n: 5_799_224.0,
        max_e: 408_150.0,
        max_n: 5_799_824.0,
    };

    /// A fetcher that answers every extra layer of the school area with its fixture.
    fn extra_fetcher() -> MapFetcher {
        let fetcher = MapFetcher::default();
        for (layer, body) in [
            ("bauwerke-flaechen", BAUWERKE_FLAECHEN),
            ("bauwerke-linien", BAUWERKE_LINIEN),
            ("relief", RELIEF),
            ("vegetation-linien", VEGETATION_LINIEN),
            ("strassenbaeume", STRASSENBAEUME),
            ("anlagenbaeume", ANLAGENBAEUME),
        ] {
            let layer = EXTRA_LAYERS.iter().find(|l| l.file == layer).unwrap();
            fetcher.insert(
                &wfs_url(layer.service, layer.type_name, &SCHOOL),
                body.as_bytes().to_vec(),
            );
        }
        fetcher
    }

    #[test]
    fn extra_layers_only_inside_berlin() {
        // Brandenburg only: no request at all, no layer.
        let fetcher = MapFetcher::default();
        assert!(fetch_extra_layers(&fetcher, &STRAUSBERG, MapKind::Sprint).is_empty());
        assert_eq!(fetcher.calls.get(), 0);

        // Inside Berlin: every layer of the map kind, in table order; relief is for sprint only.
        let fetcher = extra_fetcher();
        let sprint = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Sprint);
        let files: Vec<&str> = sprint.iter().map(|l| l.layer.file).collect();
        assert_eq!(
            files,
            [
                "bauwerke-flaechen",
                "bauwerke-linien",
                "relief",
                "vegetation-linien",
                "strassenbaeume",
                "anlagenbaeume"
            ]
        );
        assert_eq!(fetcher.calls.get(), 6);
        let fetcher = extra_fetcher();
        let schoolyard = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Schoolyard);
        assert_eq!(schoolyard.len(), 5);
        assert!(schoolyard.iter().all(|l| l.layer.file != "relief"));
        assert_eq!(fetcher.calls.get(), 5);

        // A border area keeps the Berlin layers (nothing is served here, so they fail with a
        // missing service, but they are asked for).
        let fetcher = MapFetcher::default();
        let border = fetch_extra_layers(&fetcher, &BORDER, MapKind::Sprint);
        assert_eq!(border.len(), 6);
        assert_eq!(fetcher.calls.get(), 6);
    }

    #[test]
    fn extra_layer_url_matches_the_wfs() {
        let by_file = |file: &str| EXTRA_LAYERS.iter().find(|l| l.file == file).unwrap();
        let trees = by_file("strassenbaeume");
        assert_eq!(
            wfs_url(trees.service, trees.type_name, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/baumbestand?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=baumbestand:strassenbaeume&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
        let lines = by_file("bauwerke-linien");
        assert_eq!(
            wfs_url(lines.service, lines.type_name, &SCHOOL),
            "https://gdi.berlin.de/services/wfs/alkis?service=WFS&version=2.0.0\
             &request=GetFeature&typeNames=alkis:bauwerkelinien&outputFormat=application/json\
             &count=5000&bbox=392700,5820750,392900,5820951,urn:ogc:def:crs:EPSG::25833"
        );
        // The whole table: service and type per file.
        let table: Vec<(&str, &str, &str)> = EXTRA_LAYERS
            .iter()
            .map(|l| (l.file, l.service, l.type_name))
            .collect();
        let alkis = "https://gdi.berlin.de/services/wfs/alkis";
        let trees = "https://gdi.berlin.de/services/wfs/baumbestand";
        assert_eq!(
            table,
            [
                ("bauwerke-flaechen", alkis, "alkis:bauwerkeflaechen"),
                ("bauwerke-linien", alkis, "alkis:bauwerkelinien"),
                ("relief", alkis, "alkis:relief"),
                (
                    "vegetation-linien",
                    alkis,
                    "alkis:gewaesservegetationlinien"
                ),
                ("strassenbaeume", trees, "baumbestand:strassenbaeume"),
                ("anlagenbaeume", trees, "baumbestand:anlagenbaeume"),
            ]
        );
    }

    #[test]
    fn dedup_by_feature_id_without_uuid() {
        // Tree features have no `uuid`; the top-level GeoServer id decides.
        let trees = features(STRASSENBAEUME);
        assert!(trees.iter().all(|t| t["properties"]["uuid"].is_null()));
        let layer = EXTRA_LAYERS
            .iter()
            .find(|l| l.file == "strassenbaeume")
            .unwrap();
        let fetcher = MapFetcher::default();
        let first = wfs_url(layer.service, layer.type_name, &SCHOOL);
        let second = format!("{}?STARTINDEX=3", layer.service);
        let page = |fs: &[Value], next: Option<&str>| {
            let links: Vec<Value> = next
                .map(|href| serde_json::json!({ "rel": "next", "href": href }))
                .into_iter()
                .collect();
            serde_json::json!({ "type": "FeatureCollection", "features": fs, "links": links })
                .to_string()
                .into_bytes()
        };
        fetcher.insert(&first, page(&trees[..3], Some(&second)));
        fetcher.insert(&second, page(&trees[2..], None));
        let fetched = fetch_wfs(&fetcher, layer.service, layer.type_name, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
    }

    #[test]
    fn empty_layer_is_stored_not_an_error() {
        // The hedge layer has no feature in the school area: an empty collection, not an error.
        let fetcher = extra_fetcher();
        let layers = fetch_extra_layers(&fetcher, &SCHOOL, MapKind::Sprint);
        let hedges = layers
            .iter()
            .find(|l| l.layer.file == "vegetation-linien")
            .unwrap();
        let none = hedges.features.as_ref().unwrap();
        assert_eq!(none.len(), 0);
        // Written as a collection with no feature, it reads back as an empty layer.
        assert_eq!(parse_layer(&layer_to_geojson(none)).unwrap().len(), 0);
        // And a layer with content round-trips.
        let areas = layers[0].features.as_ref().unwrap();
        assert_eq!(areas.len(), 10);
        assert_eq!(parse_layer(&layer_to_geojson(areas)).unwrap(), *areas);
    }

    #[test]
    fn building_parts_keep_bauart() {
        let parts: Vec<Value> = features(BUILDING_PARTS)
            .iter()
            .map(normalise_building)
            .collect();
        let bauart: Vec<&str> = parts
            .iter()
            .map(|p| p["properties"]["bauart"].as_str().unwrap())
            .collect();
        assert!(bauart.contains(&"Durchfahrt im Gebäude"));
        assert!(bauart.contains(&"Arkade"));
        // The no-break space of the source value is kept as it is.
        assert!(bauart.contains(&"Durchfahrt an\u{a0}überbautem Verkehrsweg"));
        // Still a Bauteil with LGB properties.
        assert!(
            parts
                .iter()
                .all(|p| p["properties"]["gebnutzbez"] == "Bauteil")
        );
        // A building without a part kind has no `bauart`.
        let whole = normalise_building(&features(BUILDINGS)[0]);
        assert!(whole["properties"]["bauart"].is_null());
        // The extra property does not disturb the LGB parser.
        let parsed = parse_buildings(&collection(&parts)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), 5);
    }

    #[test]
    fn pages_are_followed_and_duplicates_dropped() {
        let all = features(BUILDINGS);
        let fetcher = MapFetcher::default();
        let first = wfs_url(BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL);
        let second = format!("{BUILDINGS_SERVICE}?STARTINDEX=3");
        let page = |fs: &[Value], next: Option<&str>| {
            let links: Vec<Value> = next
                .map(|href| serde_json::json!({ "rel": "next", "href": href }))
                .into_iter()
                .collect();
            serde_json::json!({ "type": "FeatureCollection", "features": fs, "links": links })
                .to_string()
                .into_bytes()
        };
        fetcher.insert(&first, page(&all[..3], Some(&second)));
        fetcher.insert(&second, page(&all[2..], None));
        let fetched = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
    }

    /// Fails the first `failures` requests, then serves `body`.
    struct Flaky {
        failures: Cell<u32>,
        body: Vec<u8>,
        calls: RefCell<Vec<String>>,
    }

    impl Fetcher for Flaky {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.borrow_mut().push(url.to_string());
            if self.failures.get() > 0 {
                self.failures.set(self.failures.get() - 1);
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "Connection reset by peer".to_string(),
                });
            }
            Ok(self.body.clone())
        }
    }

    fn flaky(failures: u32, body: &[u8]) -> Flaky {
        Flaky {
            failures: Cell::new(failures),
            body: body.to_vec(),
            calls: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn flaky_server_is_retried() {
        let fetcher = flaky(2, BUILDINGS.as_bytes());
        let fetched = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(fetched.len(), 4);
        assert_eq!(fetcher.calls.borrow().len(), 3);

        let fetcher = flaky(3, BUILDINGS.as_bytes());
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(err.to_string().contains("Connection reset"), "{err}");
        assert_eq!(fetcher.calls.borrow().len(), 3);
    }

    #[test]
    fn xml_answer_is_an_error() {
        let report =
            br#"<?xml version="1.0" ?><ows:ExceptionReport><ows:Exception/></ows:ExceptionReport>"#;
        let fetcher = flaky(0, report);
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(
            err.to_string().contains("not a GeoJSON FeatureCollection"),
            "{err}"
        );
        assert!(err.to_string().starts_with("invalid ALKIS data: "), "{err}");
        assert_eq!(
            fetcher.calls.borrow().len(),
            3,
            "an XML answer is retried too"
        );
    }

    #[test]
    fn missing_service_is_not_retried() {
        let fetcher = MapFetcher::default();
        let err = fetch_wfs(&fetcher, BUILDINGS_SERVICE, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
    }
}
