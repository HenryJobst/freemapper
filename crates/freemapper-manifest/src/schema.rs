//! The tables and keys of format 1 as the TOML parser reads them: every table refuses a key it
//! does not know, and every value keeps its place in the file for the findings of the
//! validator. Which keys belong to which `kind` is the validator's part.

use std::collections::BTreeMap;

use serde::Deserialize;
use toml::Spanned;

pub(crate) type Text = Spanned<String>;
pub(crate) type Params = BTreeMap<String, Text>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Raw {
    /// Checked before this struct is read.
    #[serde(rename = "format")]
    pub _format: i64,
    pub adapter: RawAdapter,
    pub net: RawNet,
    pub boundary: RawBoundary,
    #[serde(default)]
    pub osm: RawOsm,
    pub attribution: RawAttribution,
    pub alkis: Option<RawAlkis>,
    pub orthophoto: Option<RawRaster>,
    pub terrain: Option<RawRaster>,
    pub surface: Option<RawRaster>,
    #[serde(default)]
    pub extra: RawExtra,
    pub preview: Option<RawPreview>,
    #[serde(default)]
    pub estimate: RawEstimate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAdapter {
    pub id: Text,
    pub provider: Option<Text>,
    pub revision: Spanned<i64>,
    pub min_freemapper: Text,
    pub crs: Spanned<i64>,
    pub object_id_prefix: Option<Text>,
    pub names: RawNames,
    pub agency: Option<Text>,
    #[serde(default)]
    pub maintainers: Vec<Text>,
    pub last_verified: Option<Text>,
    pub licence_of_manifest: Option<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawNames {
    pub en: Text,
    pub de: Option<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawNet {
    pub hosts: Vec<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawBoundary {
    pub file: Text,
    pub crs: Spanned<i64>,
    pub simplify_m: Option<Spanned<f64>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawOsm {
    #[serde(default)]
    pub extract: Vec<RawExtract>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawExtract {
    pub geofabrik: Text,
    pub megabytes: Spanned<f64>,
    pub label: Text,
    pub boundary: Option<Text>,
    pub cache_prefix: Option<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAttribution {
    pub licence: Text,
    pub licence_text: Option<Text>,
    pub text: Text,
    pub year: Option<Text>,
    pub short: Option<Text>,
    pub note: Option<Text>,
}

/// `[alkis]`: the keys of both kinds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAlkis {
    pub kind: Text,
    pub crs: Option<Spanned<i64>>,
    pub name: Option<Text>,
    pub url: Option<Text>,
    pub buildings_type: Option<Text>,
    pub land_use_type: Option<Text>,
    pub square_bez: Option<Text>,
    pub sorted: Option<Spanned<bool>>,
    pub buildings: Option<Text>,
    pub land_use: Option<Text>,
    pub normalise: Option<Text>,
    pub fallback: Option<RawFallback>,
}

/// `[alkis.fallback]`: a WFS.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawFallback {
    pub kind: Option<Text>,
    pub name: Option<Text>,
    pub url: Text,
    pub buildings_type: Text,
    pub land_use_type: Text,
    pub square_bez: Option<Text>,
}

/// `[orthophoto]`, `[terrain]` and `[surface]`: the keys of all their kinds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawRaster {
    pub kind: Option<Text>,
    pub backend: Option<Text>,
    pub label: Option<Text>,
    pub url: Option<Text>,
    pub layer: Option<Text>,
    pub coverage: Option<Text>,
    pub crs: Option<Spanned<i64>>,
    pub params: Option<Spanned<Params>>,
    pub max_px: Option<Spanned<i64>>,
    pub multipart: Option<Spanned<bool>>,
    pub envelope: Option<Spanned<Vec<f64>>>,
    pub max_m: Option<Spanned<f64>>,
    pub date: Option<Spanned<RawDate>>,
    pub source_label: Option<Text>,
}

/// `date` of a raster table: the keys of all date kinds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDate {
    pub kind: Option<Text>,
    pub url: Option<Text>,
    pub layer: Option<Text>,
    pub field: Option<Text>,
    pub format: Option<Text>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawExtra {
    #[serde(default)]
    pub nas: Vec<RawNas>,
    #[serde(default)]
    pub trees: Vec<RawTrees>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawNas {
    pub name: Text,
    pub url: Text,
    pub crs: Option<Spanned<i64>>,
    pub max_count: Spanned<i64>,
    pub max_bytes: Spanned<i64>,
    pub layers: Option<Vec<Text>>,
    pub namespaces: Option<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawTrees {
    pub id: Text,
    pub name: Text,
    pub crs: Option<Spanned<i64>>,
    pub envelope: Spanned<Vec<f64>>,
    pub format: Text,
    pub geo_field: Text,
    pub endpoints: Vec<Text>,
    pub fields: RawTreeFields,
    #[serde(default)]
    pub skipped_kinds: Vec<Text>,
    pub attribution: Text,
    pub max_bytes: Spanned<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawTreeFields {
    pub kind: Text,
    pub id: Text,
    pub date: Text,
    pub species: Option<Text>,
    pub crown: Option<Text>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawPreview {
    pub wms: Text,
    pub layer: Text,
    pub format: Text,
    pub params: Option<Spanned<Params>>,
    pub attribution: Text,
    pub agency: Text,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawEstimate {
    pub orthophoto_mb_per_km2: Option<Spanned<f64>>,
    pub terrain_mb_per_km2: Option<Spanned<f64>>,
    pub surface_mb_per_km2: Option<Spanned<f64>>,
}
