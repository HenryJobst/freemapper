//! City tree cadastres (single trees with a species, a survey date and sometimes a crown
//! diameter) for the 417/418 candidates.
//!
//! One row per city (the adapter of its state names it): where the points come from ([`TreeFormat`], the
//! endpoints), the field names, the kinds that are no tree (stumps), the licence line and the
//! envelope that decides whether a map area needs the dataset. Potsdam is the first row
//! ([`POTSDAM`], an OpenDataSoft GeoJSON export); the WFS, OGC API and CSV formats of the other
//! cities (Leipzig, Dresden, Hamburg, Frankfurt, Kiel, Lübeck, Rostock, Köln, Dortmund, Münster)
//! come with the row that first needs them. Every format normalises to the same stored feature:
//! a point in the map's CRS with the `tree_*` properties of [`parse_trees`].

use std::borrow::Cow;
use std::collections::HashSet;

use chrono::NaiveDate;
use serde_json::{Map, Value, json};

use crate::crs::{BoundingBox, Crs, LatLon};
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::nas::PartFile;
use crate::ogc_api::{AlkisError, FetchArea, invalid};
pub use freemapper_adapter_api::extras::{TreeDataset, TreeFields, TreeFormat};
use freemapper_adapter_api::records::AdapterId;

/// 256 MB, as for the NAS answers: the whole Potsdam envelope (283 772 points) is about 110 MB.
const MAX_ANSWER_BYTES: u64 = 256 << 20;

/// Potsdam, "Bäume der Stadtkarte" (`opendata.potsdam.de`, dl-de/zero-2-0): 283 772 points in
/// three datasets, no crown diameter. The envelope is the data extent (E 359 000–375 000,
/// N 5 801 000–5 820 000, EPSG:25833) rounded outward.
pub const POTSDAM: TreeDataset = TreeDataset {
    id: Cow::Borrowed("potsdam"),
    name: Cow::Borrowed("Potsdam tree cadastre"),
    region: AdapterId::built_in("brandenburg"),
    crs: Crs::Utm33,
    envelope: BoundingBox {
        min_e: 359_000.0,
        min_n: 5_801_000.0,
        max_e: 375_000.0,
        max_n: 5_820_000.0,
    },
    format: TreeFormat::OdsGeoJsonExport {
        geo_field: "geo_point_2d",
    },
    endpoints: &[
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-1-stadtkarte-potsdam/exports/geojson",
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-2-stadtkarte-potsdam/exports/geojson",
        "https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-3-stadtkarte-potsdam/exports/geojson",
    ],
    fields: TreeFields {
        kind: Cow::Borrowed("ggartbez"),
        id: Cow::Borrowed("pktnr"),
        date: Cow::Borrowed("datum"),
        species: None,
        crown: None,
    },
    skipped_kinds: &["Baumstumpf"],
    attribution: Cow::Borrowed("Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0"),
    max_bytes: MAX_ANSWER_BYTES,
};

/// One answer: the stored features (points in the map's CRS) and the ones that were left out.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeAnswer {
    pub features: Vec<Value>,
    /// Features that were no usable tree point, as `<id>: <why>`.
    pub skipped: Vec<String>,
}

/// Reads an answer of `dataset` into stored features in `crs`.
///
/// A stored feature is `{"type":"Feature","id":<tree id>,"geometry":Point,"properties":{
/// "tree_dataset", "tree_id", "tree_kind", "tree_date" (YYYY-MM-DD), "tree_species" and
/// "tree_crown_m" when the dataset has them}}`. Kinds in [`TreeDataset::skipped_kinds`] are left
/// out silently; a feature without a point or a tree number is listed in `skipped`.
pub fn parse_trees(text: &str, dataset: &TreeDataset, crs: Crs) -> Result<TreeAnswer, AlkisError> {
    let page: Value = serde_json::from_str(text)
        .map_err(|e| invalid(format!("{}: not GeoJSON: {e}", dataset.name)))?;
    let features = page["features"]
        .as_array()
        .filter(|_| page["type"] == "FeatureCollection")
        .ok_or_else(|| invalid(format!("{}: not a GeoJSON FeatureCollection", dataset.name)))?;
    let mut answer = TreeAnswer::default();
    for feature in features {
        match dataset.format {
            TreeFormat::OdsGeoJsonExport { .. } => normalise(feature, dataset, crs, &mut answer),
        }
    }
    Ok(answer)
}

/// The text of a property, whether the service serves it as a string or a number.
fn text<'a>(properties: &'a Map<String, Value>, key: &str) -> Option<std::borrow::Cow<'a, str>> {
    match properties.get(key)? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().into()),
        Value::Number(n) => Some(n.to_string().into()),
        _ => None,
    }
}

/// Adds the stored form of one WGS84 point feature to `answer`.
fn normalise(feature: &Value, dataset: &TreeDataset, crs: Crs, answer: &mut TreeAnswer) {
    let empty = Map::new();
    let properties = feature["properties"].as_object().unwrap_or(&empty);
    let fields = &dataset.fields;
    let kind = text(properties, &fields.kind);
    if kind
        .as_deref()
        .is_some_and(|k| dataset.skipped_kinds.contains(&k))
    {
        return;
    }
    let Some(id) = text(properties, &fields.id) else {
        answer.skipped.push(format!(
            "{}: no tree number",
            text(properties, "objectid").unwrap_or_default()
        ));
        return;
    };
    let point = (feature["geometry"]["type"] == "Point")
        .then(|| &feature["geometry"]["coordinates"])
        .and_then(|c| Some((c[0].as_f64()?, c[1].as_f64()?)))
        .filter(|(lon, lat)| lon.is_finite() && lat.is_finite());
    let Some((lon, lat)) = point else {
        answer.skipped.push(format!("{id}: no point"));
        return;
    };
    let p = crs.forward(LatLon { lat, lon });
    // Centimetres are plenty for a tree.
    let round = |v: f64| (v * 100.0).round() / 100.0;
    let mut stored = Map::new();
    stored.insert("tree_dataset".into(), dataset.id.clone().into());
    stored.insert("tree_id".into(), id.as_ref().into());
    if let Some(kind) = kind {
        stored.insert("tree_kind".into(), kind.as_ref().into());
    }
    // A date that does not parse leaves the fetch date as the currency.
    let date = text(properties, &fields.date)
        .and_then(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok());
    if let Some(date) = date {
        stored.insert("tree_date".into(), date.to_string().into());
    }
    if let Some(species) = fields.species.as_deref().and_then(|f| text(properties, f)) {
        stored.insert("tree_species".into(), species.as_ref().into());
    }
    let crown = fields
        .crown
        .as_deref()
        .and_then(|f| text(properties, f))
        .and_then(|c| c.replace(',', ".").parse::<f64>().ok())
        .filter(|c| c.is_finite() && *c > 0.0);
    if let Some(crown) = crown {
        stored.insert("tree_crown_m".into(), crown.into());
    }
    answer.features.push(json!({
        "type": "Feature",
        "id": id.as_ref(),
        "geometry": {"type": "Point", "coordinates": [round(p.easting), round(p.northing)]},
        "properties": stored,
    }));
}

/// The trees of one dataset in a map area, in the map's CRS.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeFeatures {
    pub features: Vec<Value>,
    pub skipped: Vec<String>,
    pub regions: Vec<AdapterId>,
}

/// One dataset as fetched; an empty `features` is a dataset without trees in the area.
#[derive(Debug)]
pub struct FetchedTreeLayer {
    pub dataset: &'static TreeDataset,
    pub features: Result<TreeFeatures, FetchError>,
}

/// All endpoints of `dataset` for `area`, each point once (by tree number and position). A
/// dataset whose endpoint fails fails as a whole (a half-filled file would pass for complete).
/// Each answer streams into a file `part` names right before the download (removed afterwards),
/// so one over the dataset's `max_bytes` is stopped while it is read.
pub fn fetch_dataset(
    fetcher: &dyn Fetcher,
    dataset: &TreeDataset,
    area: &FetchArea,
    part: PartFile,
) -> Result<TreeFeatures, FetchError> {
    let mut seen = HashSet::new();
    let mut fetched = TreeFeatures {
        features: Vec::new(),
        skipped: Vec::new(),
        regions: vec![dataset.region.clone()],
    };
    for endpoint in dataset.endpoints {
        let url = dataset.url(endpoint, &area.region_bbox).ok_or_else(|| {
            FetchError::Alkis(invalid(format!("{}: area out of range", dataset.name)))
        })?;
        // An error page with status 200 is not a FeatureCollection and is asked again; an
        // answer over the cap is not.
        let answer =
            get_limited_with_retries(fetcher, &url, &part()?, dataset.max_bytes, |bytes| {
                Ok(parse_trees(
                    &String::from_utf8_lossy(bytes),
                    dataset,
                    area.crs,
                )?)
            })?;
        for feature in answer.features {
            let c = &feature["geometry"]["coordinates"];
            let key = format!("{}@{},{}", feature["id"], c[0], c[1]);
            if seen.insert(key) {
                fetched.features.push(feature);
            }
        }
        fetched.skipped.extend(answer.skipped);
    }
    Ok(fetched)
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    /// A Potsdam export answer: three trees of the Stadtkarte (Sanssouci area, 2009) and a stump.
    pub const POTSDAM: &str = include_str!("../../../tests/fixtures/potsdam-trees.geojson");
}
