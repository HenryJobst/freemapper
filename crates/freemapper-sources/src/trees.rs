//! City tree cadastres (single trees with a species, a survey date and sometimes a crown
//! diameter) for the 417/418 candidates.
//!
//! One row per city in [`TREE_DATASETS`]: where the points come from ([`TreeFormat`], the
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

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::nas::PartFile;
use crate::ogc_api::{AlkisError, FetchArea, invalid};
use freemapper_adapter_api::records::AdapterId;

/// How a dataset is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeFormat {
    /// An OpenDataSoft `exports/geojson` endpoint, asked with `where=in_bbox(<geo_field>, …)` in
    /// WGS84 (no paging). The geometry is WGS84 too.
    OdsGeoJsonExport { geo_field: &'static str },
}

/// The field names of a dataset (properties of the served features).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeFields {
    /// The kind of the point (`Laubbaum`, `Baumstumpf`).
    pub kind: Cow<'static, str>,
    /// The tree's number, the `freemapper:source_id`.
    pub id: Cow<'static, str>,
    /// The survey date (`2009-06-10T00:00:00+00:00`).
    pub date: Cow<'static, str>,
    pub species: Option<Cow<'static, str>>,
    /// The crown diameter in metres.
    pub crown: Option<Cow<'static, str>>,
}

/// One city's tree cadastre.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeDataset {
    /// The file name under `data/trees/` and the `freemapper:source` suffix (`trees/<id>`).
    pub id: Cow<'static, str>,
    /// For messages.
    pub name: Cow<'static, str>,
    /// The adapter of the state the city lies in; the dataset is asked only for areas that
    /// touch it.
    pub region: AdapterId,
    /// The CRS of `envelope`.
    pub crs: Crs,
    /// The data extent rounded outward: an area outside asks for nothing.
    pub envelope: BoundingBox,
    pub format: TreeFormat,
    /// The endpoints, whose points are merged (Potsdam serves 283 772 points in three datasets).
    pub endpoints: &'static [&'static str],
    pub fields: TreeFields,
    /// Kinds that are no tree and are not stored.
    pub skipped_kinds: &'static [&'static str],
    /// The line for the attribution once a tree of the dataset is drawn.
    pub attribution: Cow<'static, str>,
    /// The largest answer of one endpoint (bytes); a larger one fails the dataset.
    pub max_bytes: u64,
}

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

/// The tree cadastres, one per city.
pub static TREE_DATASETS: [TreeDataset; 1] = [POTSDAM];

/// The request grows the area by this much (ground metres): a tree on its edge is in.
const REQUEST_MARGIN_M: f64 = 1.0;

/// The WGS84 envelope (lat_min, lon_min, lat_max, lon_max) of `bbox` (EPSG:25833) grown by
/// [`REQUEST_MARGIN_M`], from points along its outline (the edges bend in WGS84).
fn wgs84_bbox(bbox: &BoundingBox) -> Option<(f64, f64, f64, f64)> {
    const STEPS: u32 = 8;
    let bbox = bbox.buffered(REQUEST_MARGIN_M);
    let mut out: Option<(f64, f64, f64, f64)> = None;
    for i in 0..=STEPS {
        let t = f64::from(i) / f64::from(STEPS);
        let e = bbox.min_e + t * (bbox.max_e - bbox.min_e);
        let n = bbox.min_n + t * (bbox.max_n - bbox.min_n);
        for (easting, northing) in [
            (e, bbox.min_n),
            (e, bbox.max_n),
            (bbox.min_e, n),
            (bbox.max_e, n),
        ] {
            let p = Crs::Utm33.inverse(ProjectedPoint { easting, northing })?;
            out = Some(match out {
                None => (p.lat, p.lon, p.lat, p.lon),
                Some((a, b, c, d)) => (a.min(p.lat), b.min(p.lon), c.max(p.lat), d.max(p.lon)),
            });
        }
    }
    out
}

impl TreeDataset {
    /// The request for `endpoint` for the points in `region_bbox` (EPSG:25833, grown by 1 m).
    /// The box is asked as `lat_min,lon_min,lat_max,lon_max`. `None` for an area no position
    /// projects to.
    pub fn url(&self, endpoint: &str, region_bbox: &BoundingBox) -> Option<String> {
        let (lat_min, lon_min, lat_max, lon_max) = wgs84_bbox(region_bbox)?;
        Some(match self.format {
            TreeFormat::OdsGeoJsonExport { geo_field } => format!(
                "{endpoint}?where=in_bbox({geo_field},{lat_min:.6},{lon_min:.6},{lat_max:.6},{lon_max:.6})"
            ),
        })
    }
}

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
