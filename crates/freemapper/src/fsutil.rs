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

/// Deletes `*.part` files in `dir` older than `max_age`: partial writes of interrupted runs.
/// Younger ones may belong to a run still in progress and stay. Best effort: errors are ignored.
pub(crate) fn remove_stale_parts(dir: &Path, max_age: std::time::Duration) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let is_part = entry.file_name().to_string_lossy().ends_with(".part");
        let age = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok());
        if is_part && age.is_some_and(|age| age > max_age) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
