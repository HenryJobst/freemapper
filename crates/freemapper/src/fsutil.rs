//! File system helpers.

use std::path::{Path, PathBuf};

/// Writes `bytes` to `<path>.<pid>.part` and renames it, so `path` never holds a partial file.
/// The process id keeps concurrent runs from writing the same partial file. On failure, returns
/// the path the error is about.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, std::io::Error)> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| (dir.to_path_buf(), e))?;
    let mut part = path.as_os_str().to_owned();
    part.push(format!(".{}.part", std::process::id()));
    let part = PathBuf::from(part);
    std::fs::write(&part, bytes).map_err(|e| (part.clone(), e))?;
    std::fs::rename(&part, path).map_err(|e| (path.to_path_buf(), e))
}
