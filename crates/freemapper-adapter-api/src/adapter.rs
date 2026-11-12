//! What a region supplies: the trait an adapter implements.

use freemapper_georef::crs::BoundingBox;
use geo_types::MultiPolygon;

use crate::alkis::AlkisSource;
use crate::credit::{CreditLine, UsedSources};
use crate::estimate::{EstimateArea, EstimateEnv};
use crate::extras::{NasService, TreeDataset};
use crate::info::{Declared, RegionInfo};
use crate::policy::UrlPolicy;
use crate::pointcloud::PointCloudBackend;
use crate::rasters::RasterBackend;
use crate::records::EstimateItem;

/// Everything one state or country supplies: who it is and where it lies, its OSM extracts, its
/// ALKIS service, its rasters, its extra sources, the credit it asks for and what a map of it
/// downloads. The engine asks the adapters of the registry and names no region itself.
pub trait RegionAdapter: Send + Sync {
    /// Identity and static facts.
    fn info(&self) -> &RegionInfo;

    /// What the manifest of the adapter declares about itself. `None`: the adapter has no
    /// manifest.
    fn declared(&self) -> Option<&Declared> {
        None
    }

    /// The hosts every request for this adapter is held to, redirects included: an adapter
    /// asks the hosts its manifest declares and no other, whether it is built into the program
    /// or came from a folder. `None`: the adapter has no manifest and asks as its code asks.
    fn policy(&self) -> Option<&UrlPolicy> {
        None
    }

    /// The boundary in EPSG:25833, parsed once and valid.
    fn boundary(&self) -> &MultiPolygon<f64>;

    /// The boundary (EPSG:25833) of the OSM extract `index` of [`RegionInfo::osm`], for a region
    /// with several extracts: a map reads those whose boundary lies near it. `None`: the extract
    /// has no boundary of its own and is always read.
    fn extract_boundary(&self, _index: usize) -> Option<&MultiPolygon<f64>> {
        None
    }

    /// The ALKIS service of the region: buildings and land use as features in the LGB schema.
    /// `None`: the region has no ALKIS data to offer.
    fn alkis(&self) -> Option<&dyn AlkisSource>;

    /// The full ALKIS (NAS) service of the region, for the structures ALKIS vereinfacht leaves
    /// out. `None`: the region has none.
    fn nas(&self) -> Option<&NasService> {
        None
    }

    /// The tree cadastres of cities in the region.
    fn tree_datasets(&self) -> &[TreeDataset] {
        &[]
    }

    /// The credit lines for what a project holds of this region, in the order they are printed;
    /// empty before anything of it was fetched.
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine>;

    /// The raster sources of the region. `None`: the region has ALKIS but no rasters, and no
    /// map can have its centre in it.
    fn rasters(&self) -> Option<&dyn RasterBackend>;

    /// The raster downloads of a map of `area` whose rasters come from this region (what, MB,
    /// cached, part, store), in the order they are listed.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem>;

    /// The classified laser point clouds of the region, for the vegetation classes. `None`: the
    /// region publishes none.
    fn point_cloud(&self) -> Option<&dyn PointCloudBackend> {
        None
    }

    /// The box around the boundary (EPSG:25833). The region functions ask it before the
    /// boundary itself: what lies outside the box cannot touch the region. An adapter that
    /// keeps the box answers faster than this default, which goes through the boundary.
    fn bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox {
            min_e: f64::INFINITY,
            min_n: f64::INFINITY,
            max_e: f64::NEG_INFINITY,
            max_n: f64::NEG_INFINITY,
        };
        for polygon in &self.boundary().0 {
            for point in polygon.exterior() {
                bounds.min_e = bounds.min_e.min(point.x);
                bounds.min_n = bounds.min_n.min(point.y);
                bounds.max_e = bounds.max_e.max(point.x);
                bounds.max_n = bounds.max_n.max(point.y);
            }
        }
        bounds
    }
}
