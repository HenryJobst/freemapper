//! Extra sources a region may have beside rasters and ALKIS vereinfacht: a full ALKIS (NAS)
//! service and the tree cadastres of its cities, which an adapter names as data (the readers are
//! in `freemapper-sources`), and sets of layers an adapter fetches with code of its own
//! ([`ExtraSet`]).

use std::borrow::Cow;

use freemapper_contract::MapKind;
use freemapper_georef::crs::{BoundingBox, Crs, ProjectedPoint};
use serde_json::Value;

use crate::error::FetchError;
use crate::fetch::Fetcher;
use crate::records::AdapterId;

/// A set of extra layers an adapter fetches with code of its own, such as the registers and
/// surveys of a city. The set says which layers there are and fetches them; the engine stores
/// them one file per layer under `data/<key>/`, keeps the file of a layer that could not be
/// fetched while it covers the area, removes what is no longer used and reads the layers back
/// for the rules that draw from them.
pub trait ExtraSet: Send + Sync {
    /// What the engine needs to store the layers and to report on them.
    fn info(&self) -> &ExtraSetInfo;

    /// The layers a map of `kind` uses, in the order they are fetched and read.
    fn layers(&self, kind: MapKind) -> Vec<ExtraLayerInfo>;

    /// Whether `key` is a layer of the set, for any map kind. The engine touches a file only
    /// under such a key.
    fn has(&self, key: &str) -> bool;

    /// The layers of `kind` for `region_bbox` (EPSG:25833), in the order of [`Self::layers`]. A
    /// layer that fails does not stop the others.
    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        region_bbox: &BoundingBox,
        kind: MapKind,
    ) -> Vec<FetchedExtra>;
}

/// The facts of an [`ExtraSet`].
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraSetInfo {
    /// The key of the set; its layers are stored in the folder `data/<key>` of a project.
    pub key: &'static str,
    /// The CRS the features of the stored layers are in.
    pub crs: Crs,
    pub notes: ExtraNotes,
    pub estimate: ExtraEstimate,
}

/// The message ids of a set, texts of the catalog of this crate. The messages about one layer
/// name it as `file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraNotes {
    /// Offline, the area is in reach of the set, and nothing is stored.
    pub none_stored: &'static str,
    /// The area left the reach of the set and the stored layers go.
    pub removing: &'static str,
    pub download: &'static str,
    /// A layer was stored: `count` and `file`.
    pub features: &'static str,
    /// A layer failed and its stored file stays: `file` and `detail`.
    pub kept: &'static str,
    /// A layer failed and nothing of it is stored: `file` and `detail`.
    pub dropped: &'static str,
}

/// The item of a set in the download estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraEstimate {
    pub what: &'static str,
    pub megabytes_per_km2: f64,
}

/// One layer of an [`ExtraSet`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraLayerInfo {
    /// The name of the stored file without its extension.
    pub key: &'static str,
}

/// One layer as fetched; an empty list is a layer without objects in the area.
#[derive(Debug)]
pub struct FetchedExtra {
    pub key: &'static str,
    /// The features in the CRS of the set, or why the layer could not be fetched.
    pub features: Result<Vec<Value>, FetchError>,
}

/// A NAS WFS of one state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NasService {
    /// For messages, e.g. "LGB ALKIS (NAS)".
    pub name: Cow<'static, str>,
    pub url: Cow<'static, str>,
    /// The adapter of the state the service covers; it is asked only for areas that touch it.
    pub region: AdapterId,
    /// The CRS asked for and answered in.
    pub crs: Crs,
    /// Features per GetFeature (`COUNT`); the service has no paging.
    pub max_count: usize,
    /// The largest answer read; a larger one is stopped and asked again in quadrants.
    pub max_bytes: u64,
    /// The files of the NAS layers the service serves; `None`: all of them. A type the state
    /// does not serve answers an exception report, which would warn on every fetch.
    pub layers: Option<Cow<'static, [Cow<'static, str>]>>,
    /// The `NAMESPACES` parameter, for a server that wants the `adv` prefix declared.
    pub namespaces: Option<Cow<'static, str>>,
}

impl NasService {
    /// Whether the service serves the layer stored as `file`.
    pub fn serves(&self, file: &str) -> bool {
        self.layers
            .as_ref()
            .is_none_or(|files| files.iter().any(|served| served == file))
    }

    /// The GetFeature of `type_name` features in `bbox` (in [`NasService::crs`]), rounded
    /// outward, with `COUNT` = [`NasService::max_count`].
    pub fn get_feature_url(&self, type_name: &str, bbox: &BoundingBox) -> String {
        let epsg = self.crs.epsg();
        format!(
            "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}\
             &SRSNAME=urn:ogc:def:crs:EPSG::{epsg}&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::{epsg}\
             &COUNT={}{}",
            self.url,
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil(),
            self.max_count,
            self.namespaces
                .as_deref()
                .map(|n| format!("&NAMESPACES={n}"))
                .unwrap_or_default()
        )
    }
}

/// How a dataset is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeFormat {
    /// An OpenDataSoft `exports/geojson` endpoint, asked with `where=in_bbox(<geo_field>, …)` in
    /// WGS84 (no paging). The geometry is WGS84 too.
    OdsGeoJsonExport { geo_field: Cow<'static, str> },
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
    pub endpoints: Cow<'static, [Cow<'static, str>]>,
    pub fields: TreeFields,
    /// Kinds that are no tree and are not stored.
    pub skipped_kinds: Cow<'static, [Cow<'static, str>]>,
    /// The line for the attribution once a tree of the dataset is drawn.
    pub attribution: Cow<'static, str>,
    /// The largest answer of one endpoint (bytes); a larger one fails the dataset.
    pub max_bytes: u64,
}

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
        Some(match &self.format {
            TreeFormat::OdsGeoJsonExport { geo_field } => format!(
                "{endpoint}?where=in_bbox({geo_field},{lat_min:.6},{lon_min:.6},{lat_max:.6},{lon_max:.6})"
            ),
        })
    }
}
