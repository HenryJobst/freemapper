// SPDX-License-Identifier: AGPL-3.0-or-later

//! The raster sources of a region: orthophoto, terrain model and surface model. A backend
//! fetches them and tells `derive` where to read them.

use std::collections::BTreeMap;
use std::path::PathBuf;

use freemapper_georef::crs::BoundingBox;
use freemapper_georef::tiles::TileId;

use crate::cache::CacheDir;
use crate::env::AdapterEnv;
use crate::error::AdapterError;
use crate::estimate::{EstimateArea, EstimateEnv};
use crate::fetch::FetchedTile;
use crate::project::RasterProject;
use crate::records::{AdapterId, EstimateItem, NdsmRecord, TileRecord, WcsRecord, WmsRecord};
use crate::registry::Registry;
use crate::trust::TrustAnchors;

/// How a provider delivers its rasters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterKind {
    /// Tiles of a grid in EPSG:25833 with `grid_km` km a side, kept in the shared cache.
    Tiles { grid_km: u32 },
    /// Requests for the area itself, stored in the project folder.
    PerArea,
}

/// A provider of rasters: one authority, which may serve several regions (LGB: Brandenburg and
/// Berlin).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    pub id: AdapterId,
    /// The folder below the cache the provider keeps its downloads in; `None`: it keeps none.
    pub cache_dir: Option<String>,
    /// The terrain model the contours come from, for `freemapper:source` on contour candidates.
    pub terrain_source: String,
    pub kind: RasterKind,
}

/// What `[orthophoto]` of a manifest says besides the name of its backend: where the backend
/// asks for the orthophoto (issue #273). A key the manifest leaves out is `None`, and the
/// backend then takes the one of the built-in manifest of its state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RasterConfig {
    /// The WMS address, without a query.
    pub url: Option<String>,
    /// The layer of the orthophoto.
    pub layer: Option<String>,
    /// The largest side of one request in pixels.
    pub max_px: Option<u32>,
    /// The other addresses the backend asks, by the name the backend gives each (`metadata`,
    /// `dgm1`, …), without a query. A name the manifest leaves out is the built-in one.
    pub endpoints: BTreeMap<String, String>,
}

impl RasterConfig {
    /// Whether the manifest said none of the keys.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    /// This config, with the keys it leaves out taken from `fallback`.
    pub fn or(&self, fallback: &Self) -> Self {
        Self {
            url: self.url.clone().or_else(|| fallback.url.clone()),
            layer: self.layer.clone().or_else(|| fallback.layer.clone()),
            max_px: self.max_px.or(fallback.max_px),
            endpoints: fallback
                .endpoints
                .iter()
                .chain(&self.endpoints)
                .map(|(name, url)| (name.clone(), url.clone()))
                .collect(),
        }
    }

    /// The address named `name`, if the config holds it.
    pub fn endpoint(&self, name: &str) -> Option<&str> {
        self.endpoints.get(name).map(String::as_str)
    }

    /// The address named `name` of the backend `backend`, or the error that says it is missing.
    pub fn require(&self, backend: &str, name: &str) -> Result<String, String> {
        self.endpoint(name)
            .map(str::to_string)
            .ok_or_else(|| format!("the endpoint `{name}` of the backend `{backend}` is missing"))
    }
}

/// The area `derive` builds the templates for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterArea {
    /// The map area plus buffer in the CRS of the map.
    pub bbox: BoundingBox,
    /// The same in EPSG:25833.
    pub region_bbox: BoundingBox,
}

/// What `derive` hands a backend: the regions and the cache.
#[derive(Clone, Copy)]
pub struct DeriveEnv<'a> {
    pub regions: &'a Registry,
    pub cache: &'a CacheDir,
}

/// The raster records of a project, to read.
#[derive(Debug, Clone, Copy)]
pub struct StoredRasters<'a> {
    pub tiles: &'a [TileRecord],
    pub ndsm_tiles: &'a [NdsmRecord],
    pub orthophoto: Option<&'a WmsRecord>,
    pub terrain: Option<&'a WcsRecord>,
    pub surface: Option<&'a WcsRecord>,
}

/// Where the orthophoto template comes from.
#[derive(Debug)]
pub enum OrthophotoInput {
    /// Tile archives in the cache.
    Tiles(Vec<FetchedTile>),
    /// The image a WMS or WCS delivered for the area, stored in the project folder.
    Image,
}

/// Where a height model comes from.
#[derive(Debug)]
pub enum HeightInput {
    /// Tile archives in the cache.
    Tiles(Vec<FetchedTile>),
    /// The raster fetched for the area, stored in the project folder.
    Stored,
}

/// An nDSM point cloud the project records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NdsmInput {
    /// The tile as the project file names it.
    pub tile: String,
    /// The file in the cache; `None` when `tile` is no tile id.
    pub path: Option<PathBuf>,
}

/// What `derive` reads. Each input carries its own error, because `derive` meets them one after
/// the other: a missing surface model costs a warning after the terrain templates are written.
#[derive(Debug)]
pub struct DeriveInputs {
    /// Tiles the area needs that the project does not record: the area changed since the fetch.
    pub outdated: Vec<TileId>,
    pub orthophoto: Result<OrthophotoInput, AdapterError>,
    pub dtm: Result<HeightInput, AdapterError>,
    /// The surface model, if the project can have one. [`HeightInput::Stored`] says where to
    /// look, not that the file is there.
    pub dsm: Option<Result<HeightInput, AdapterError>>,
    /// The nDSM point clouds the project records.
    pub ndsm: Vec<NdsmInput>,
}

impl DeriveInputs {
    /// The inputs of a per-area provider: everything lies in the project folder.
    pub fn stored() -> Self {
        Self {
            outdated: Vec::new(),
            orthophoto: Ok(OrthophotoInput::Image),
            dtm: Ok(HeightInput::Stored),
            dsm: Some(Ok(HeightInput::Stored)),
            ndsm: Vec::new(),
        }
    }
}

/// The raster sources of a region.
pub trait RasterBackend: Send + Sync {
    /// Who delivers the rasters, and how.
    fn provider(&self) -> &ProviderInfo;

    /// Fetches the rasters of `bbox` (the map area plus buffer in the CRS of the map) into the
    /// cache or the project folder, updates the raster records and saves the project file after
    /// each part that finished.
    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError>;

    /// The trust anchors of a host of this provider that the default roots cannot verify.
    /// `None`: every host is verified against the default roots.
    fn trust(&self) -> Option<&TrustAnchors> {
        None
    }

    /// The downloads of a map of `area` (what, MB, cached, part, store), in the order they are
    /// listed. The default lists none.
    fn estimate(&self, _area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        Vec::new()
    }

    /// What `derive` reads for `area`. The default is that of a per-area provider.
    fn derive_inputs(
        &self,
        _env: &DeriveEnv<'_>,
        _stored: &StoredRasters<'_>,
        _area: &RasterArea,
    ) -> DeriveInputs {
        DeriveInputs::stored()
    }
}

#[cfg(test)]
mod config_tests {
    use super::RasterConfig;
    use std::collections::BTreeMap;

    fn config(url: Option<&str>, layer: Option<&str>, max_px: Option<u32>) -> RasterConfig {
        RasterConfig {
            url: url.map(str::to_string),
            layer: layer.map(str::to_string),
            max_px,
            endpoints: BTreeMap::new(),
        }
    }

    #[test]
    fn a_config_with_no_key_is_empty_and_one_key_is_not() {
        assert!(RasterConfig::default().is_empty());
        assert!(!config(Some("https://a"), None, None).is_empty());
        assert!(!config(None, Some("l"), None).is_empty());
        assert!(!config(None, None, Some(1)).is_empty());
    }

    #[test]
    fn a_key_of_the_config_wins_and_a_missing_one_comes_from_the_fallback() {
        let fallback = config(Some("https://fallback"), Some("fl"), Some(100));
        let own = config(Some("https://own"), Some("ol"), Some(200));
        assert_eq!(own.or(&fallback), own);
        assert_eq!(RasterConfig::default().or(&fallback), fallback);
        assert_eq!(
            config(Some("https://own"), None, None).or(&fallback),
            config(Some("https://own"), Some("fl"), Some(100))
        );
        assert_eq!(
            config(None, Some("ol"), None).or(&fallback),
            config(Some("https://fallback"), Some("ol"), Some(100))
        );
        assert_eq!(
            config(None, None, Some(200)).or(&fallback),
            config(Some("https://fallback"), Some("fl"), Some(200))
        );
    }

    #[test]
    fn an_endpoint_makes_a_config_non_empty_and_wins_per_name() {
        let named = |pairs: &[(&str, &str)]| RasterConfig {
            endpoints: pairs
                .iter()
                .map(|(n, u)| (n.to_string(), u.to_string()))
                .collect(),
            ..RasterConfig::default()
        };
        assert!(!named(&[("a", "https://a")]).is_empty());
        let fallback = named(&[("a", "https://fa"), ("b", "https://fb")]);
        let merged = named(&[("b", "https://ob")]).or(&fallback);
        assert_eq!(merged.endpoint("a"), Some("https://fa"));
        assert_eq!(merged.endpoint("b"), Some("https://ob"));
        assert_eq!(merged.endpoint("c"), None);
        assert_eq!(merged.require("b1", "a").unwrap(), "https://fa");
        let missing = merged.require("b1", "c").unwrap_err();
        assert!(
            missing.contains("`c`") && missing.contains("`b1`"),
            "{missing}"
        );
    }
}
