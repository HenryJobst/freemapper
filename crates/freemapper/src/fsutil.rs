//! File system helpers.

use std::path::{Component, Path, PathBuf};

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
    write_part_with(path, bytes, &|part, bytes| {
        use std::io::Write as _;
        create_new_file(part)?.write_all(bytes)
    })
}

/// Creates the partial or temporary file `path` (a name freemapper made) for writing. Whatever
/// is there is removed first, without following it: a stale link with that name (to a file
/// outside the project or cache) is removed as a link, and its target is never written. The
/// file is then created exclusively (`create_new`), so a link put there meanwhile makes the
/// creation fail instead of being followed (ADR 0059).
pub(crate) fn create_new_file(path: &Path) -> std::io::Result<std::fs::File> {
    if std::fs::symlink_metadata(path).is_ok() {
        std::fs::remove_file(path)?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
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
        // The error names the file the user knows, not the partial file just removed.
        return Err((path.to_path_buf(), e));
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

/// Whether `name`, read from a project file or a server, is one plain file name: no path
/// separator, no `..`, no drive prefix (`C:`), no NUL or other control character, not absolute
/// and not empty. Names that fail are never put into a path.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains(['/', '\\', ':'])
        && !name.chars().any(char::is_control)
        && matches!(
            Path::new(name).components().collect::<Vec<_>>()[..],
            [Component::Normal(_)]
        )
}

/// Whether `path` stays inside the folder `root` once links and `..` are resolved. `path` need
/// not exist: its nearest existing folder is resolved, and the rest must be plain names. A
/// `root` that does not exist holds nothing. Checked before a file named by a project file is
/// written or removed, so a link in the project folder cannot lead the write elsewhere.
pub(crate) fn stays_inside(root: &Path, path: &Path) -> bool {
    let Ok(root) = std::fs::canonicalize(root) else {
        return false;
    };
    let (Some(parent), Some(Component::Normal(_))) = (path.parent(), path.components().next_back())
    else {
        return false;
    };
    for folder in parent.ancestors() {
        let Ok(resolved) = std::fs::canonicalize(folder) else {
            continue;
        };
        let rest = parent.strip_prefix(folder).unwrap_or(parent);
        return resolved.starts_with(&root)
            && rest.components().all(|c| matches!(c, Component::Normal(_)));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[cfg(unix)]
    #[test]
    fn atomic_write_does_not_follow_a_linked_part_file() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, b"secret").unwrap();
        let path = dir.path().join("a.txt");
        // A stale partial file of this process id that is a link out of the folder.
        let part = dir
            .path()
            .join(format!("a.txt.{}.part", std::process::id()));
        std::os::unix::fs::symlink(&victim, &part).unwrap();
        write_atomically(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&victim).unwrap(), b"secret");
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert!(std::fs::symlink_metadata(&part).is_err());
    }

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
        // The error names the file the user knows; the partial file is gone.
        assert_eq!(path, target);
        let leftovers: Vec<_> = std::fs::read_dir(target.parent().unwrap())
            .unwrap()
            .flatten()
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
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

    #[test]
    fn plain_names_are_single_file_names() {
        for name in [
            "relief",
            "sb-gelaender",
            "bauwerke-flaechen",
            "a.b",
            "trinkwasserbrunnen",
        ] {
            assert!(is_plain_name(name), "{name}");
        }
        for name in [
            "",
            ".",
            "..",
            "../evil",
            "../../evil",
            "a/b",
            r"a\b",
            "/etc/evil",
            r"\\server\share",
            "C:evil",
            "C:",
            "a..b",
            "nul\0x",
            "tab\tx",
            "line\nx",
        ] {
            assert!(!is_plain_name(name), "{name:?}");
        }
    }

    #[test]
    fn paths_below_the_root_stay_inside() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        std::fs::create_dir_all(root.join("data")).unwrap();
        assert!(stays_inside(&root, &root.join("data/a.geojson")));
        // Folders that do not exist yet are fine.
        assert!(stays_inside(&root, &root.join("data/berlin/a.geojson")));
        assert!(!stays_inside(&root, &root.join("data/../../a.geojson")));
        assert!(!stays_inside(
            &root,
            &root.join("data/berlin/../../../a.geojson")
        ));
        assert!(!stays_inside(&root, &dir.path().join("a.geojson")));
        assert!(!stays_inside(&root, &root.join("data/..")));
        // A root that does not exist holds nothing.
        assert!(!stays_inside(
            &dir.path().join("none"),
            &dir.path().join("none/a")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn links_out_of_the_root_do_not_stay_inside() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("data/berlin")).unwrap();
        assert!(!stays_inside(&root, &root.join("data/berlin/a.geojson")));
        assert!(!stays_inside(
            &root,
            &root.join("data/berlin/new/a.geojson")
        ));
        // A root reached through a link is still the same folder.
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&root, &link).unwrap();
        assert!(stays_inside(&link, &link.join("data/a.geojson")));
        assert!(stays_inside(&link, &root.join("data/a.geojson")));
    }
}
