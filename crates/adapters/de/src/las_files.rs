//! The cached point-cloud tiles of a provider: one LAS/LAZ file per tile and publication,
//! `<tile>_<published>.laz`, checked to be complete before it is kept (ADR 0105).

use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::fetch::FetchError;

/// `<tile>_<published>.laz` in `dir`; `tile` is the provider's name of the tile (`33421-5821`).
pub(crate) fn las_path(dir: &Path, tile: &str, published: NaiveDate) -> PathBuf {
    dir.join(format!("{tile}_{published}.laz"))
}

/// The cached point clouds of `tile` in `dir`, newest first.
pub(crate) fn cached_las(dir: &Path, tile: &str) -> Vec<(NaiveDate, PathBuf)> {
    let prefix = format!("{tile}_");
    let mut files: Vec<(NaiveDate, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(&prefix)?
                .strip_suffix(".laz")?
                .parse()
                .ok()?;
            Some((date, entry.path()))
        })
        .collect();
    files.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
    files
}

/// Checks that the file at `path` is a complete LAS/LAZ point cloud: its header, the LAZ chunk
/// table at the end and the last point have to be readable.
pub(crate) fn check_las_file(url: &str, path: &Path) -> Result<(), FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a complete LAS point cloud: {message}"),
    };
    let mut reader = las::Reader::from_path(path).map_err(|e| corrupt(e.to_string()))?;
    let total = reader.header().number_of_points();
    if total > 0 {
        reader.seek(total - 1).map_err(|e| corrupt(e.to_string()))?;
        reader.read_points(1).map_err(|e| corrupt(e.to_string()))?;
    }
    Ok(())
}
