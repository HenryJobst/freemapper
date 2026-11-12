//! What a region supplies: the trait an adapter implements.

use freemapper_georef::crs::BoundingBox;
use geo_types::MultiPolygon;

use crate::alkis::AlkisSource;
use crate::credit::{CreditLine, UsedSources};
use crate::estimate::{EstimateArea, EstimateEnv};
use crate::info::RegionInfo;
use crate::rasters::RasterBackend;
use crate::records::EstimateItem;

/// Everything one state or country supplies. This is the first cut (plan task C1): identity
/// and boundary, by now with the OSM extracts (C3), the credit lines (C4), the estimate (C5), ALKIS (C6) and the rasters (C7). The tasks that move a concern off the region enums add its method.
pub trait RegionAdapter: Send + Sync {
    /// Identity and static facts.
    fn info(&self) -> &RegionInfo;

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

    /// The credit lines for what a project holds of this region, in the order they are printed;
    /// empty before anything of it was fetched.
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine>;

    /// The raster sources of the region. `None`: the region has ALKIS but no rasters, and no
    /// map can have its centre in it.
    fn rasters(&self) -> Option<&dyn RasterBackend>;

    /// The raster downloads of a map of `area` whose rasters come from this region (what, MB,
    /// cached, part, store), in the order they are listed.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem>;

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
