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
