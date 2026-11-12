//! Moved to `freemapper-adapters-de` as `berlin` (ADR 0082); this module keeps the old paths
//! working and takes the map kind of the engine where the adapter crate takes the contract's.

pub use freemapper_adapters_de::berlin::*;

use crate::crs::BoundingBox;
use crate::fetch::Fetcher;
use crate::project::MapKind;

/// The extra layers `kind` uses.
pub fn extra_layers(kind: MapKind) -> impl Iterator<Item = &'static ExtraLayer> {
    freemapper_adapters_de::berlin::extra_layers(kind.contract())
}

/// The extra layers of `kind` in `region_bbox` (EPSG:25833), one request series per layer; none
/// at all, and no request, when the area is not near Berlin.
pub fn fetch_extra_layers(
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
) -> Vec<FetchedLayer> {
    freemapper_adapters_de::berlin::fetch_extra_layers(fetcher, region_bbox, kind.contract())
}
