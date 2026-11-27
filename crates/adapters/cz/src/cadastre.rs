// SPDX-License-Identifier: AGPL-3.0-or-later

//! The buildings and land use of Czechia from ČÚZK (CC BY 4.0). The buildings are the footprints
//! of RÚIAN (`StavebniObjekt`), one object per building; the land use is the polygon layers of
//! ZABAGED (forest, grassland, water, parks, cemeteries and more). Both are ArcGIS map services
//! of `ags.cuzk.gov.cz`, asked for the map area as GeoJSON in the CRS of the map, so no
//! projection is needed. The objects come in the schema the engine reads (ALKIS "vereinfacht":
//! `oid`, `gebnutzbez`, `funktion`; `nutzart`, `bez`), with the German words the engine knows.

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::fetch::{Fetcher, get_with_retries, one_line};
use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};
use freemapper_georef::crs::{BoundingBox, Crs};
use serde_json::{Value, json};

/// The map service with the register RÚIAN.
pub const RUIAN: &str = "https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer";

/// The map service with the polygons of ZABAGED.
pub const ZABAGED: &str =
    "https://ags.cuzk.gov.cz/arcgis/rest/services/ZABAGED_POLOHOPIS/MapServer";

/// The layer `StavebniObjekt` of RÚIAN: the footprints of the buildings.
const BUILDINGS_LAYER: u32 = 3;

/// Objects per request; the services answer 2000 at most.
const PAGE: usize = 1000;

/// Requests per layer before a fetch gives up (1000 objects each): a guard against a service
/// that never ends its list.
const MAX_PAGES: usize = 500;

/// One land use layer of ZABAGED: its number, `nutzart` and `bez` in the words of the engine.
/// Water has two layers' worth of kinds and is chosen by the attribute `stojvoda` (see `water`).
pub struct LandUseLayer {
    pub layer: u32,
    pub name: &'static str,
    pub nutzart: &'static str,
    pub bez: Option<&'static str>,
}

const fn layer(
    layer: u32,
    name: &'static str,
    nutzart: &'static str,
    bez: Option<&'static str>,
) -> LandUseLayer {
    LandUseLayer {
        layer,
        name,
        nutzart,
        bez,
    }
}

/// The polygon layers of ZABAGED that tell what the ground is, in the words of the engine:
/// forest and scrub are white forest (405), meadows and parks lawn (401), arable land, gardens,
/// vineyards and cemeteries only candidates for it, wet ground rough open land (403).
pub const LAND_USE: &[LandUseLayer] = &[
    layer(142, "Lesní půda se stromy", "Wald", None),
    layer(144, "Lesní půda se stromy kategorizovaná", "Wald", None),
    layer(140, "Lesní půda s křovinatým porostem", "Gehölz", None),
    layer(141, "Lesní půda s kosodřevinou", "Gehölz", None),
    layer(
        139,
        "Trvalý travní porost",
        "Landwirtschaft",
        Some("Grünland"),
    ),
    layer(138, "Orná půda", "Landwirtschaft", Some("Ackerland")),
    layer(
        135,
        "Ovocný sad, zahrada",
        "Landwirtschaft",
        Some("Gartenland"),
    ),
    layer(136, "Vinice", "Landwirtschaft", Some("Weingarten")),
    layer(137, "Chmelnice", "Landwirtschaft", Some("Hopfen")),
    layer(
        134,
        "Udržovaná zeleň",
        "Sport-, Freizeit- und Erholungsfläche",
        Some("Grünanlage"),
    ),
    layer(116, "Hřbitov", "Friedhof", None),
    layer(131, "Bažina, močál", "Sumpf", None),
    layer(18, "Rašeliniště", "Moor", None),
    layer(
        118,
        "Povrchová těžba, lom",
        "Tagebau, Grube, Steinbruch",
        None,
    ),
    layer(
        114,
        "Areál účelové zástavby",
        "Industrie- und Gewerbefläche",
        None,
    ),
    layer(132, "Vodní plocha", "Stehendes Gewässer", None),
];

/// The layer of the water areas; a river is told from a pond by the attribute `stojvoda`.
const WATER_LAYER: u32 = 132;

/// The request for one page of `layer` of `service`: the objects inside `bbox` (in `crs`) as
/// GeoJSON in the same CRS, in a stable order.
pub fn query_url(service: &str, layer: u32, bbox: &BoundingBox, crs: Crs, offset: usize) -> String {
    let epsg = crs.epsg();
    format!(
        "{service}/{layer}/query?geometry={},{},{},{}&geometryType=esriGeometryEnvelope\
         &inSR={epsg}&outSR={epsg}&spatialRel=esriSpatialRelIntersects&outFields=*\
         &returnGeometry=true&orderByFields=OBJECTID&resultOffset={offset}\
         &resultRecordCount={PAGE}&f=geojson",
        bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n
    )
}

/// The features of one answer; an error the service reports (HTTP 200 with an `error` object,
/// or a page that is no feature collection) is an error naming it.
pub fn read_page(body: &[u8], url: &str) -> Result<Vec<Value>, FetchError> {
    let refused = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let value: Value = serde_json::from_slice(body).map_err(|e| {
        let start = one_line(&String::from_utf8_lossy(&body[..body.len().min(200)]), 200);
        refused(format!("no GeoJSON in the answer ({e}): {start}"))
    })?;
    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(refused(format!(
            "the service answered with an error: {message}"
        )));
    }
    match value.get("features").and_then(Value::as_array) {
        Some(features) => Ok(features.clone()),
        None => Err(refused("the answer holds no list of features".to_string())),
    }
}

/// Every object of `layer` inside `bbox`, page by page.
pub fn fetch_layer(
    fetcher: &dyn Fetcher,
    service: &str,
    layer: u32,
    bbox: &BoundingBox,
    crs: Crs,
) -> Result<Vec<Value>, FetchError> {
    let mut out = Vec::new();
    for page in 0..MAX_PAGES {
        let url = query_url(service, layer, bbox, crs, page * PAGE);
        let features = get_with_retries(fetcher, &url, |body| read_page(body, &url))?;
        let full = features.len() >= PAGE;
        out.extend(features);
        if !full {
            return Ok(out);
        }
    }
    Err(FetchError::Http {
        url: format!("{service}/{layer}"),
        message: format!(
            "the service lists more than {} objects for the map area",
            MAX_PAGES * PAGE
        ),
    })
}

/// The function of a building as ALKIS words it, from the code of RÚIAN (`zpusobvyuzitikod`, the
/// domain `VFR_ZpusobVyuziti Objektu`). `None` for what is no building: dams, weirs, locks and
/// other structures of water works.
pub fn building_function(code: i64) -> Option<&'static str> {
    Some(match code {
        2..=3 | 6..=8 => "Wohngebäude",
        1 | 12 => "Industriegebäude",
        4 | 13 => "Gebäude für Wirtschaft und Gewerbe",
        5 | 15 => "Gebäude für öffentliche Zwecke",
        9 => "Gebäude zur Freizeitgestaltung",
        10 | 11 => "Handels- und Dienstleistungsgebäude",
        14 => "Bürogebäude",
        16 => "Gebäude für Versorgung",
        17 => "Gebäude für Verkehr und Nachrichtenwesen",
        18 => "Garage",
        21 => "Treibhaus, Gewächshaus",
        19 | 20 | 30 => "Sonstiges Gebäude",
        22..=29 => return None,
        _ => "Sonstiges Gebäude",
    })
}

/// The buildings of RÚIAN as the engine reads them. A building that no longer stands
/// (`platido` set) or is marked wrong (`nespravny`) is left out, and so is a structure of water
/// works ([`building_function`]).
pub fn buildings(features: &[Value], crs: Crs) -> AlkisPart {
    let features = features
        .iter()
        .filter_map(|f| {
            let p = f.get("properties")?;
            if !p.get("nespravny").is_none_or(Value::is_null)
                || !p.get("platido").is_none_or(Value::is_null)
            {
                return None;
            }
            let kod = p.get("kod")?.as_i64()?;
            let function = p
                .get("zpusobvyuzitikod")
                .and_then(Value::as_i64)
                .map_or(Some("Sonstiges Gebäude"), building_function)?;
            let geometry = f.get("geometry").filter(|g| !g.is_null())?;
            let oid = format!("ruian-{kod}");
            Some(json!({
                "type": "Feature",
                "id": oid,
                "geometry": geometry,
                "properties": {
                    "oid": oid,
                    "gebnutzbez": "Gebäude",
                    "funktion": function,
                },
            }))
        })
        .collect();
    AlkisPart { features, crs }
}

/// `nutzart` and `bez` of a water area of ZABAGED: `stojvoda` is `A` for a pond or a lake.
fn water(properties: &Value) -> &'static str {
    match properties.get("stojvoda").and_then(Value::as_str) {
        Some("N") => "Fließgewässer",
        _ => "Stehendes Gewässer",
    }
}

/// The land use of one layer of ZABAGED, tagged with the layer so that two layers never give one
/// `oid` twice.
pub fn land_use_of(features: &[Value], kind: &LandUseLayer) -> Vec<Value> {
    features
        .iter()
        .filter_map(|f| {
            let p = f.get("properties")?;
            let geometry = f.get("geometry").filter(|g| !g.is_null())?;
            let id = p
                .get("fid_zbg")
                .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string))
                .or_else(|| p.get("OBJECTID").map(Value::to_string))?;
            let oid = format!("zabaged-{}-{id}", kind.layer);
            let nutzart = if kind.layer == WATER_LAYER {
                water(p)
            } else {
                kind.nutzart
            };
            let mut properties = json!({"oid": oid, "nutzart": nutzart});
            if let Some(bez) = kind.bez {
                properties["bez"] = json!(bez);
            }
            Some(json!({
                "type": "Feature",
                "id": oid,
                "geometry": geometry,
                "properties": properties,
            }))
        })
        .collect()
}

/// The cadastre of ČÚZK with every request held to the hosts of the adapter's policy.
pub struct CzCadastre {
    pub policy: UrlPolicy,
}

impl AlkisSource for CzCadastre {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let fetcher = PolicyFetcher::new(env.fetcher, &self.policy);
        let features = fetch_layer(&fetcher, RUIAN, BUILDINGS_LAYER, &area.bbox, area.crs)?;
        Ok(buildings(&features, area.crs))
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let fetcher = PolicyFetcher::new(env.fetcher, &self.policy);
        let mut out = Vec::new();
        for kind in LAND_USE {
            let features = fetch_layer(&fetcher, ZABAGED, kind.layer, &area.bbox, area.crs)?;
            out.extend(land_use_of(&features, kind));
        }
        Ok(AlkisPart {
            features: out,
            crs: area.crs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::testing::MapFetcher;

    fn bbox() -> BoundingBox {
        BoundingBox {
            min_e: 458_385.0,
            min_n: 5_548_301.0,
            max_e: 458_881.0,
            max_n: 5_548_737.0,
        }
    }

    fn square() -> Value {
        json!({"type": "Polygon", "coordinates": [[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 0.0]]]})
    }

    fn collection(features: Vec<Value>) -> Vec<u8> {
        json!({"type": "FeatureCollection", "features": features})
            .to_string()
            .into_bytes()
    }

    fn building(kod: i64, function: i64, ended: Option<i64>, wrong: Option<i64>) -> Value {
        json!({"type": "Feature", "geometry": square(), "properties": {
            "kod": kod, "zpusobvyuzitikod": function, "platido": ended, "nespravny": wrong}})
    }

    #[test]
    fn the_request_names_the_layer_the_envelope_and_the_crs_of_the_map() {
        assert_eq!(
            query_url(RUIAN, 3, &bbox(), Crs::Utm33, 2000),
            "https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer/3/query?\
             geometry=458385,5548301,458881,5548737&geometryType=esriGeometryEnvelope\
             &inSR=25833&outSR=25833&spatialRel=esriSpatialRelIntersects&outFields=*\
             &returnGeometry=true&orderByFields=OBJECTID&resultOffset=2000&resultRecordCount=1000\
             &f=geojson"
        );
    }

    #[test]
    fn a_building_gets_its_function_and_one_that_is_gone_or_wrong_is_left_out() {
        let part = buildings(
            &[
                building(1, 7, None, None),
                building(2, 14, None, None),
                building(3, 7, Some(1_700_000_000_000), None),
                building(4, 7, None, Some(1)),
                building(5, 23, None, None),
                building(6, 99, None, None),
            ],
            Crs::Utm33,
        );
        let kinds: Vec<(&str, &str)> = part
            .features
            .iter()
            .map(|f| {
                (
                    f["properties"]["oid"].as_str().unwrap(),
                    f["properties"]["funktion"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            kinds,
            [
                ("ruian-1", "Wohngebäude"),
                ("ruian-2", "Bürogebäude"),
                ("ruian-6", "Sonstiges Gebäude")
            ]
        );
        assert!(
            part.features
                .iter()
                .all(|f| f["properties"]["gebnutzbez"] == "Gebäude")
        );
    }

    #[test]
    fn every_code_of_the_register_has_a_function_or_is_no_building() {
        for code in 1..=30 {
            let function = building_function(code);
            assert_eq!(function.is_none(), (22..=29).contains(&code), "{code}");
        }
    }

    #[test]
    fn a_pond_and_a_river_are_told_apart_by_the_attribute_of_the_water_layer() {
        let kind = LAND_USE.iter().find(|l| l.layer == WATER_LAYER).unwrap();
        let features = [
            json!({"geometry": square(), "properties": {"fid_zbg": "1", "stojvoda": "A"}}),
            json!({"geometry": square(), "properties": {"fid_zbg": "2", "stojvoda": "N"}}),
            json!({"geometry": square(), "properties": {"fid_zbg": "3"}}),
        ];
        let land: Vec<String> = land_use_of(&features, kind)
            .iter()
            .map(|f| f["properties"]["nutzart"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            land,
            ["Stehendes Gewässer", "Fließgewässer", "Stehendes Gewässer"]
        );
    }

    #[test]
    fn land_use_keeps_its_words_and_an_oid_of_its_layer() {
        let meadow = LAND_USE.iter().find(|l| l.layer == 139).unwrap();
        let land = land_use_of(
            &[json!({"geometry": square(), "properties": {"fid_zbg": "4711"}})],
            meadow,
        );
        assert_eq!(land[0]["properties"]["oid"], "zabaged-139-4711");
        assert_eq!(land[0]["properties"]["nutzart"], "Landwirtschaft");
        assert_eq!(land[0]["properties"]["bez"], "Grünland");
        // An object with no geometry is no land use.
        assert!(
            land_use_of(
                &[json!({"geometry": null, "properties": {"fid_zbg": "1"}})],
                meadow
            )
            .is_empty()
        );
    }

    #[test]
    fn a_layer_is_read_page_by_page_until_a_page_is_not_full() {
        let fetcher = MapFetcher::default();
        let full: Vec<Value> = (0..PAGE)
            .map(|i| json!({"geometry": square(), "properties": {"kod": i}}))
            .collect();
        fetcher.insert(
            &query_url(RUIAN, 3, &bbox(), Crs::Utm33, 0),
            collection(full),
        );
        fetcher.insert(
            &query_url(RUIAN, 3, &bbox(), Crs::Utm33, PAGE),
            collection(vec![
                json!({"geometry": square(), "properties": {"kod": 5000}}),
            ]),
        );
        let all = fetch_layer(&fetcher, RUIAN, 3, &bbox(), Crs::Utm33).unwrap();
        assert_eq!(all.len(), PAGE + 1);
        assert_eq!(fetcher.log.borrow().len(), 2);
    }

    #[test]
    fn an_error_of_the_service_is_an_error_that_names_it() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(ZABAGED, 139, &bbox(), Crs::Utm33, 0),
            br#"{"error":{"code":500,"message":"Service not started"}}"#.to_vec(),
        );
        let error = fetch_layer(&fetcher, ZABAGED, 139, &bbox(), Crs::Utm33)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Service not started"), "{error}");
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(ZABAGED, 139, &bbox(), Crs::Utm33, 0),
            b"<html>proxy</html>".to_vec(),
        );
        let error = fetch_layer(&fetcher, ZABAGED, 139, &bbox(), Crs::Utm33)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no GeoJSON"), "{error}");
    }

    #[test]
    fn a_layer_that_never_ends_is_stopped() {
        let fetcher = MapFetcher::default();
        let full: Vec<Value> = (0..PAGE)
            .map(|i| json!({"geometry": square(), "properties": {"kod": i}}))
            .collect();
        fetcher.insert_prefix(
            "https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer/3/query?",
            collection(full),
        );
        let error = fetch_layer(&fetcher, RUIAN, 3, &bbox(), Crs::Utm33)
            .unwrap_err()
            .to_string();
        assert!(error.contains("more than"), "{error}");
        assert_eq!(fetcher.log.borrow().len(), MAX_PAGES);
    }

    #[test]
    fn every_code_of_the_use_gets_its_german_function_word() {
        let named = [
            (1, "Industriegebäude"),
            (2, "Wohngebäude"),
            (3, "Wohngebäude"),
            (4, "Gebäude für Wirtschaft und Gewerbe"),
            (5, "Gebäude für öffentliche Zwecke"),
            (6, "Wohngebäude"),
            (7, "Wohngebäude"),
            (8, "Wohngebäude"),
            (9, "Gebäude zur Freizeitgestaltung"),
            (10, "Handels- und Dienstleistungsgebäude"),
            (11, "Handels- und Dienstleistungsgebäude"),
            (12, "Industriegebäude"),
            (13, "Gebäude für Wirtschaft und Gewerbe"),
            (14, "Bürogebäude"),
            (15, "Gebäude für öffentliche Zwecke"),
            (16, "Gebäude für Versorgung"),
            (17, "Gebäude für Verkehr und Nachrichtenwesen"),
            (18, "Garage"),
            (19, "Sonstiges Gebäude"),
            (20, "Sonstiges Gebäude"),
            (21, "Treibhaus, Gewächshaus"),
            (30, "Sonstiges Gebäude"),
            (99, "Sonstiges Gebäude"),
        ];
        for (code, word) in named {
            assert_eq!(building_function(code), Some(word), "code {code}");
        }
        for code in 22..=29 {
            assert_eq!(building_function(code), None, "code {code}");
        }
    }
}
