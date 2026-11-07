//! Derived templates: orthophoto mosaic, hillshade, slope.

pub mod orthophoto;
pub mod terrain;

use std::path::PathBuf;

use crate::raster::RasterError;

#[derive(Debug, thiserror::Error)]
pub enum DeriveError {
    #[error("{path}: {source}", path = .0.display(), source = .1)]
    Io(PathBuf, std::io::Error),
    #[error("tile archive: {0}")]
    Zip(String),
    #[error("image: {0}")]
    Image(String),
    #[error(transparent)]
    Raster(#[from] RasterError),
    #[error("no tiles to derive from")]
    NoTiles,
    #[error(
        "map area too large for one template: {width} × {height} pixels (at most {MAX_RASTER_SIDE_PX} per side)"
    )]
    AreaTooLarge { width: u32, height: u32 },
}

/// Largest template side in pixels: 4 km of orthophoto at 0.2 m.
pub const MAX_RASTER_SIDE_PX: u32 = 20_000;

/// The pixel grid of a template covering `bbox`, refused when it exceeds [`MAX_RASTER_SIDE_PX`].
pub(crate) fn template_grid(
    bbox: &crate::crs::BoundingBox,
    pixel_size: f64,
) -> Result<crate::raster::GridSpec, DeriveError> {
    let grid = crate::raster::GridSpec::covering(bbox, pixel_size);
    if grid.width > MAX_RASTER_SIDE_PX || grid.height > MAX_RASTER_SIDE_PX {
        return Err(DeriveError::AreaTooLarge {
            width: grid.width,
            height: grid.height,
        });
    }
    Ok(grid)
}

/// Writes a template file atomically.
pub(crate) fn write_file(path: &std::path::Path, bytes: &[u8]) -> Result<(), DeriveError> {
    crate::fsutil::write_atomically(path, bytes).map_err(|(path, e)| DeriveError::Io(path, e))
}

/// Reads one member of a tile archive.
pub(crate) fn read_member(zip: &std::path::Path, member: &str) -> Result<Vec<u8>, DeriveError> {
    use std::io::Read as _;
    let file = std::fs::File::open(zip).map_err(|e| DeriveError::Io(zip.to_path_buf(), e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| DeriveError::Zip(e.to_string()))?;
    let mut entry = archive
        .by_name(member)
        .map_err(|e| DeriveError::Zip(format!("{member}: {e}")))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| DeriveError::Io(zip.to_path_buf(), e))?;
    Ok(bytes)
}
