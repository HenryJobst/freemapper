//! The project as a raster fetch sees it: the records it fills, the folder it writes into and
//! the checkpoint that saves the project file.

use std::path::Path;

use freemapper_georef::crs::{BoundingBox, Crs};

use crate::error::AdapterError;
use crate::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};

/// The records of the project file a raster fetch reads and writes. A view, not a part of the
/// project file with a name of its own: the file keeps its keys and their order.
pub struct RasterRecords<'a> {
    pub tiles: &'a mut Vec<TileRecord>,
    pub ndsm_tiles: &'a mut Vec<NdsmRecord>,
    pub orthophoto: &'a mut Option<WmsRecord>,
    pub terrain: &'a mut Option<WcsRecord>,
    pub surface: &'a mut Option<WcsRecord>,
}

/// A project with its folder, for the raster fetch of an adapter built into the program.
pub trait RasterProject {
    /// The raster records, to read and to change.
    fn records(&mut self) -> RasterRecords<'_>;

    /// Writes the project file as it is now. A fetch calls it after each part that finished, so
    /// that a fetch which fails later leaves the project with exactly those parts.
    fn save(&mut self) -> Result<(), AdapterError>;

    /// The project folder. The rasters are written below it, through `freemapper_adapter_api::fs`.
    fn dir(&self) -> &Path;

    /// The map area plus buffer in EPSG:25833, where the boundaries of the regions are.
    fn region_bbox(&self) -> BoundingBox;

    /// The CRS of the map.
    fn crs(&self) -> Crs;
}
