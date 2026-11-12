//! The reader of an "ALKIS vereinfacht" OGC API Features service: paging through a collection,
//! and parsing buildings and land use in the LGB property schema, which the WFS reader produces
//! too. Which service serves which state is decided by the caller.

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher};

pub use freemapper_adapter_api::alkis::FetchArea;
pub use freemapper_adapter_api::error::AlkisError;

/// Features per page; a 1 km tile holds about 800 buildings.
const PAGE_LIMIT: u32 = 10_000;
/// `rellage` value of building parts below ground.
const UNDERGROUND: &str = "Unter der Erdoberfläche";

pub fn invalid(message: impl Into<String>) -> AlkisError {
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

/// `feature` with its geometry's coordinates moved from `from` into `to`.
pub fn reproject_feature(mut feature: Value, from: Crs, to: Crs) -> Value {
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

pub fn feature_collection(features: Vec<Value>) -> String {
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
pub fn collection_features(
    fetcher: &dyn Fetcher,
    endpoint: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    collection_features_in(fetcher, endpoint, bbox, Crs::Utm33)
}

/// The features of all pages of a collection endpoint in `bbox`, in `crs`.
pub fn collection_features_in(
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
    Ok(parse_building_features(features))
}

/// Like [`parse_buildings`] for the features of an already parsed collection.
pub fn parse_building_features(features: &[Value]) -> ParsedBuildings {
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
    parsed
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
