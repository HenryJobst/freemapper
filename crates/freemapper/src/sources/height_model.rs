//! What a fetch of a tiled height model (HVBG Hesse, LVermGeo SH) found besides the raster.

use crate::tiles::TileId;

/// What a tiled height model fetch found besides the raster.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeightModel {
    /// Tiles inside the provider's state that it does not have.
    pub missing: Vec<TileId>,
    /// The acquisition (flight) date (`YYYY-MM-DD`) of the tile at the centre of the area.
    pub date: Option<String>,
    /// Hesse: places of the metadata list without a district or file in the Downloadcenter
    /// (such as a gemeindefrei area), as `<municipality> (<district>)`; their tiles came from
    /// other zips or are missing.
    pub unresolved: Vec<String>,
    /// Hesse: tiles inside the map area that stayed empty and that an unresolved place is listed
    /// for.
    pub unresolved_empty: Vec<TileId>,
    /// Problems that did not stop the fetch (a date that cannot be read, a list that fell back
    /// to a kept older one, cells without height).
    pub warnings: Vec<String>,
}
