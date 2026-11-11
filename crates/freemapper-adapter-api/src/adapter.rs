//! What a region supplies: the trait an adapter implements.

use geo_types::MultiPolygon;

use crate::info::RegionInfo;

/// Everything one state or country supplies. This is the first cut (plan task C1): identity
/// and boundary. The tasks that move a concern off the region enums add its method.
pub trait RegionAdapter: Send + Sync {
    /// Identity and static facts.
    fn info(&self) -> &RegionInfo;

    /// The boundary in EPSG:25833, parsed once and valid.
    fn boundary(&self) -> &MultiPolygon<f64>;
}
