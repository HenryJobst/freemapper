//! The classified laser point clouds of a region, for the vegetation classes (ADR 0105). A
//! backend fetches the tiles under an area into the shared cache; `derive` reads them there.

use std::path::PathBuf;

use chrono::NaiveDate;
use freemapper_georef::crs::BoundingBox;
use serde::{Deserialize, Serialize};

use crate::cache::CacheDir;
use crate::env::AdapterEnv;
use crate::error::AdapterError;
use crate::estimate::{EstimateArea, EstimateEnv};
use crate::records::{AdapterId, EstimateItem};

/// Who publishes the point clouds of a region, and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointCloudInfo {
    /// The provider, as for its rasters.
    pub id: AdapterId,
    /// The folder below the cache the provider keeps its downloads in.
    pub cache_dir: String,
    /// The product, for `freemapper:source` on the vegetation candidates (`lgb-als`).
    pub source: String,
    /// The side of a tile in km, in EPSG:25833.
    pub grid_km: u32,
}

/// One point-cloud tile the project records: where it came from and when it was published.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointCloudRecord {
    pub tile: String,
    pub url: String,
    /// The publication date the provider gives; `None` where it gives none.
    pub published: Option<NaiveDate>,
}

/// One point-cloud tile in the cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointCloudTile {
    pub record: PointCloudRecord,
    /// The LAS or LAZ file, checked to be complete.
    pub path: PathBuf,
}

/// The point clouds of a region.
pub trait PointCloudBackend: Send + Sync {
    /// Who publishes them, and how.
    fn info(&self) -> &PointCloudInfo;

    /// Makes the tiles under `bbox` (EPSG:25833) available in the cache and returns them in grid
    /// order. A tile the provider does not publish is left out with a progress line; offline,
    /// the newest cached file of a tile is used.
    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError>;

    /// The cached file of a recorded tile, if it is still there.
    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf>;

    /// The point-cloud downloads of a map of `area` (part [`Part::PointCloud`], stored in the
    /// cache).
    ///
    /// [`Part::PointCloud`]: crate::records::Part::PointCloud
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem>;
}
