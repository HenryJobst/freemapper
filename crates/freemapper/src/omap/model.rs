//! In-memory model of an OOM map document.

use super::coord::{FLAG_CLOSE, FLAG_HOLE, MapCoord};
use super::symbol_set::SymbolSet;
use crate::georef::Georeferencing;

#[derive(Debug, Clone, PartialEq)]
pub enum MapObjectKind {
    Point { rotation_rad: f64 },
    Path,
}

/// A single map object with symbol, geometry and tags.
#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub symbol_id: u32,
    pub kind: MapObjectKind,
    pub coords: Vec<MapCoord>,
    pub tags: Vec<(String, String)>,
}

/// A group of map objects (OOM part).
#[derive(Debug, Clone, PartialEq)]
pub struct MapPart {
    pub name: String,
    pub objects: Vec<MapObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    /// Raster image located by a world file (`TemplateImage`).
    Image,
    Gdal,
    Ogr,
    Map,
}

/// Reference to a template file, relative to the map file.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateRef {
    pub kind: TemplateKind,
    pub relpath: String,
    pub georeferenced: bool,
    pub crs_spec: Option<String>,
    pub visible: bool,
    pub opacity: f32,
    /// Drawn below the map objects (rasters) rather than above them.
    pub behind_map: bool,
}

/// A complete OOM map document.
#[derive(Debug, Clone, PartialEq)]
pub struct MapDocument {
    pub notes: String,
    pub georef: Georeferencing,
    pub symbol_set: SymbolSet,
    pub parts: Vec<MapPart>,
    pub templates: Vec<TemplateRef>,
}

/// Builds one closed ring: drops a duplicate closing input point, then repeats the first point
/// with the hole and close flags (18), as OOM expects for every ring of an area.
pub fn closed_ring(points: &[(i32, i32)]) -> Vec<MapCoord> {
    let mut ring: Vec<MapCoord> = points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect();
    if ring.len() > 1 && ring.first() == ring.last() {
        ring.pop();
    }
    if let Some(&first) = ring.first() {
        ring.push(MapCoord {
            flags: FLAG_HOLE | FLAG_CLOSE,
            ..first
        });
    }
    ring
}
