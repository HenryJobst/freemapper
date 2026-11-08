//! File system helpers.

use std::path::{Path, PathBuf};

/// Attempts to replace a file; a viewer holding it open (OOM on Windows) blocks only briefly.
const RENAME_ATTEMPTS: u32 = 3;
const RETRY_PAUSE: std::time::Duration = std::time::Duration::from_millis(200);

/// Writes `bytes` to `<path>.<pid>.part` and renames it, so `path` never holds a partial file.
/// The process id keeps concurrent runs from writing the same partial file. On failure, returns
/// the path the error is about.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, std::io::Error)> {
    write_files_atomically(&[(path, bytes)])
}

/// Writes files that belong together (an image and its world file): first every file as a
/// partial file, then renames them one after the other, retrying a failed rename. When writing
/// fails, the targets stay as they were and the partial files are removed.
pub(crate) fn write_files_atomically(
    files: &[(&Path, &[u8])],
) -> Result<(), (PathBuf, std::io::Error)> {
    let mut parts: Vec<(PathBuf, &Path)> = Vec::new();
    let cleanup = |parts: &[(PathBuf, &Path)]| {
        for (part, _) in parts {
            let _ = std::fs::remove_file(part);
        }
    };
    for (path, bytes) in files {
        let written = write_part(path, bytes);
        match written {
            Ok(part) => parts.push((part, path)),
            Err(e) => {
                cleanup(&parts);
                return Err(e);
            }
        }
    }
    for (i, (part, path)) in parts.iter().enumerate() {
        if let Err(e) = retry(RENAME_ATTEMPTS, || std::fs::rename(part, path)) {
            cleanup(&parts[i..]);
            return Err((path.to_path_buf(), e));
        }
    }
    Ok(())
}

fn write_part(path: &Path, bytes: &[u8]) -> Result<PathBuf, (PathBuf, std::io::Error)> {
    write_part_with(path, bytes, &|part, bytes| std::fs::write(part, bytes))
}

/// [`write_part`] through `write`, which tests replace with a full disk.
fn write_part_with(
    path: &Path,
    bytes: &[u8],
    write: &dyn Fn(&Path, &[u8]) -> std::io::Result<()>,
) -> Result<PathBuf, (PathBuf, std::io::Error)> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| (dir.to_path_buf(), e))?;
    let mut part = path.as_os_str().to_owned();
    part.push(format!(".{}.part", std::process::id()));
    let part = PathBuf::from(part);
    if let Err(e) = write(&part, bytes) {
        // Half a file is of no use; on a full disk it also takes the space others need.
        let _ = std::fs::remove_file(&part);
        return Err((part, e));
    }
    Ok(part)
}

/// Runs `op` up to `attempts` times with a short pause between attempts.
fn retry<T>(attempts: u32, mut op: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut last = None;
    for attempt in 0..attempts {
        if attempt > 0 {
            std::thread::sleep(RETRY_PAUSE);
        }
        match op() {
            Ok(value) => return Ok(value),
            Err(e) => last = Some(e),
        }
    }
    Err(last.expect("at least one attempt"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn pair_is_written_completely() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("t/a.png"), dir.path().join("t/a.pgw"));
        write_files_atomically(&[(&a, b"image"), (&b, b"world")]).unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), b"image");
        assert_eq!(std::fs::read(&b).unwrap(), b"world");
        let parts = std::fs::read_dir(dir.path().join("t"))
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .count();
        assert_eq!(parts, 0);
    }

    #[test]
    fn failing_second_file_leaves_the_first_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        std::fs::write(&a, b"old image").unwrap();
        // The second file's folder cannot be created: a file of that name exists.
        std::fs::write(dir.path().join("blocked"), b"").unwrap();
        let b = dir.path().join("blocked").join("a.pgw");
        let (path, _) = write_files_atomically(&[(&a, b"new image"), (&b, b"world")]).unwrap_err();
        assert!(path.starts_with(dir.path().join("blocked")), "{path:?}");
        assert_eq!(std::fs::read(&a).unwrap(), b"old image");
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn storage_full_while_writing_removes_the_part_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("templates/hillshade.png");
        // Half the file arrives, then the disk is full.
        let full_disk = |part: &Path, bytes: &[u8]| -> std::io::Result<()> {
            std::fs::write(part, &bytes[..bytes.len() / 2])?;
            Err(std::io::Error::from(std::io::ErrorKind::StorageFull))
        };
        let (path, e) = write_part_with(&target, b"a whole image", &full_disk).unwrap_err();
        assert!(crate::space::is_storage_full(&e), "{e:?}");
        assert!(path.to_string_lossy().ends_with(".part"), "{path:?}");
        assert!(!path.exists());
        assert!(!target.exists());
    }

    #[test]
    fn transient_failures_are_retried() {
        let attempts = Cell::new(0);
        let result = retry(RENAME_ATTEMPTS, || {
            attempts.set(attempts.get() + 1);
            if attempts.get() < RENAME_ATTEMPTS {
                Err(std::io::Error::other("busy"))
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok());
        assert_eq!(attempts.get(), RENAME_ATTEMPTS);
        let always = retry(RENAME_ATTEMPTS, || {
            Err::<(), _>(std::io::Error::other("busy"))
        });
        assert!(always.is_err());
    }
}
