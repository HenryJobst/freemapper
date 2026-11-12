//! What the rasters a manifest declares as services would download for an area.

use freemapper_adapter_api::estimate::{
    EstimateArea, ORTHOPHOTO_MB_PER_KM2, WCS_MB_PER_KM2, per_km2_item,
};
use freemapper_adapter_api::records::{EstimateItem, Part};

use crate::model::{HeightModel, Manifest, Orthophoto};

/// The download items of the raster tables of `manifest` that are services (WMS, WCS): each
/// grows with the area, by the megabytes per km² of `[estimate]` or the usual ones. A table
/// that names a backend gives no item here; its backend knows what it downloads.
pub fn declared_items(manifest: &Manifest, area: &EstimateArea) -> Vec<EstimateItem> {
    let bbox = &area.bbox_25833;
    let sizes = &manifest.estimate;
    let mut items = Vec::new();
    match &manifest.orthophoto {
        Some(Orthophoto::Wms { label, .. } | Orthophoto::Wcs { label, .. }) => {
            items.push(per_km2_item(
                bbox,
                label,
                sizes.orthophoto_mb_per_km2.unwrap_or(ORTHOPHOTO_MB_PER_KM2),
                Part::Orthophoto,
            ));
        }
        Some(Orthophoto::Backend { .. }) | None => {}
    }
    for (model, megabytes, part) in [
        (&manifest.terrain, sizes.terrain_mb_per_km2, Part::Terrain),
        (&manifest.surface, sizes.surface_mb_per_km2, Part::Surface),
    ] {
        if let Some(HeightModel::Wcs { label, .. }) = model {
            items.push(per_km2_item(
                bbox,
                label,
                megabytes.unwrap_or(WCS_MB_PER_KM2),
                part,
            ));
        }
    }
    items
}
