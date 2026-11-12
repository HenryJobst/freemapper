//! freemapper's side of the shared crate `freemapper-fs` (ADR 0067, freemapper-core ADR 0009),
//! for the engine, the readers and the adapters.
//!
//! The crate writes, copies and removes inside the project folder without following a link. Its
//! refusals name no app; the wrappers here put them back into freemapper's own words, which the
//! manual quotes. They also print the warning when the stale-part cleanup refuses a folder, and
//! name the partial files that downloads stream into. Calls that cannot be refused go to
//! `freemapper_fs` directly.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// What a copy says about a source that is a link or lies outside the project folder.
pub const NOT_COPIED: &str = "a link or outside the project folder, so freemapper does not copy it";

/// What a write into the project folder says when a link would lead it elsewhere.
pub const OUTSIDE_PROJECT: &str =
    "a link leads out of the project folder, so freemapper writes nothing there";

/// Numbers the partial files of [`fresh_part_name`], so no two downloads pick the same name.
static NEXT_PART: AtomicU64 = AtomicU64::new(0);

/// A refusal of the crate in freemapper's words; any other error stays as it is.
fn own_words((path, e): (PathBuf, io::Error)) -> (PathBuf, io::Error) {
    let own = match e.get_ref().map(ToString::to_string).as_deref() {
        Some(freemapper_fs::OUTSIDE_PROJECT) => OUTSIDE_PROJECT,
        Some(freemapper_fs::NOT_COPIED) => NOT_COPIED,
        _ => return (path, e),
    };
    (path, io::Error::other(own))
}

/// [`freemapper_fs::write_inside`] with freemapper's refusal text.
pub fn write_inside(root: &Path, path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::write_inside(root, path, bytes).map_err(own_words)
}

/// [`freemapper_fs::write_files_inside`] with freemapper's refusal text.
pub fn write_files_inside(
    root: &Path,
    files: &[(&Path, &[u8])],
) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::write_files_inside(root, files).map_err(own_words)
}

/// [`freemapper_fs::copy_inside`] with freemapper's refusal texts.
pub fn copy_inside(root: &Path, from: &Path, to: &Path) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::copy_inside(root, from, to).map_err(own_words)
}

/// [`freemapper_fs::check_inside`] with freemapper's refusal text.
pub fn check_inside(root: &Path, path: &Path) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::check_inside(root, path).map_err(own_words)
}

/// [`freemapper_fs::check_source`] with freemapper's refusal text.
pub fn check_source(root: &Path, from: &Path) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::check_source(root, from).map_err(own_words)
}

/// [`freemapper_fs::check_folder_inside`] with freemapper's refusal text.
pub fn check_folder_inside(root: &Path, folder: &Path) -> Result<(), (PathBuf, io::Error)> {
    freemapper_fs::check_folder_inside(root, folder).map_err(own_words)
}

/// [`freemapper_fs::remove_stale_parts_inside`]; a refused folder is not cleaned and gets
/// `warning: not cleaning stale parts in <folder>: <reason>`.
pub fn remove_stale_parts_inside(root: &Path, folder: &Path, max_age: std::time::Duration) {
    let cleaned = freemapper_fs::remove_stale_parts_inside(root, folder, max_age);
    if let Err((path, e)) = cleaned.map_err(own_words) {
        eprintln!(
            "warning: not cleaning stale parts in {}: {e}",
            path.display()
        );
    }
}

/// A partial file name for `path` no other download of any run uses:
/// `<path>.<process id>.<number>.part`. For a download that streams into a file of its own
/// ([`freemapper_fs::create_new_file`] opens it). The crate names its own partial files after
/// their targets, and freemapper streams only into `answer.*` names the crate never writes, so
/// the two counters cannot meet.
pub fn fresh_part_name(path: &Path) -> PathBuf {
    let mut part = path.as_os_str().to_owned();
    part.push(format!(
        ".{}.{}.part",
        std::process::id(),
        NEXT_PART.fetch_add(1, Relaxed)
    ));
    PathBuf::from(part)
}

#[cfg(test)]
mod tests {
    //! The crate tests its own guarantees (freemapper-core). These check what freemapper adds:
    //! its wording and its warning.

    use super::*;

    #[test]
    fn crate_errors_that_are_no_refusal_stay_as_they_are() {
        let e = own_words((PathBuf::from("a"), io::Error::from(io::ErrorKind::NotFound)));
        assert_eq!(e.1.kind(), io::ErrorKind::NotFound);
        let e = own_words((PathBuf::from("a"), io::Error::other("busy")));
        assert_eq!(e.1.to_string(), "busy");
    }

    #[cfg(unix)]
    #[test]
    fn stale_part_cleanup_does_not_follow_a_templates_link() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("x.part"), b"keep").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("templates")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        remove_stale_parts_inside(
            root.path(),
            &root.path().join("templates"),
            std::time::Duration::ZERO,
        );
        assert!(outside.path().join("x.part").exists());
    }

    #[test]
    fn fresh_part_names_never_repeat() {
        let path = Path::new("data/alkis-nas/answer.gml");
        let (a, b) = (fresh_part_name(path), fresh_part_name(path));
        assert_ne!(a, b);
        let pid = format!("answer.gml.{}.", std::process::id());
        for name in [&a, &b] {
            let name = name.file_name().unwrap().to_string_lossy();
            assert!(name.starts_with(&pid) && name.ends_with(".part"), "{name}");
        }
    }

    #[test]
    fn copies_and_writes_inside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        std::fs::create_dir_all(&root).unwrap();
        let map = root.join("map.omap");
        write_inside(&root, &map, b"map").unwrap();
        copy_inside(&root, &map, &root.join("ocad/map.omap.bak")).unwrap();
        assert_eq!(
            std::fs::read(root.join("ocad/map.omap.bak")).unwrap(),
            b"map"
        );
        let (path, e) = write_inside(&root, &dir.path().join("x.txt"), b"x").unwrap_err();
        assert_eq!(path, dir.path().join("x.txt"));
        assert_eq!(e.to_string(), OUTSIDE_PROJECT);
        assert!(!dir.path().join("x.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn writes_into_a_linked_folder_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::os::unix::fs::symlink(outside.path(), root.join("ocad")).unwrap();
        std::fs::write(root.join("map.omap"), "map").unwrap();
        let target = root.join("ocad/map.ocd");
        let (path, e) = write_inside(root, &target, b"ocd").unwrap_err();
        assert_eq!(
            (path, e.to_string()),
            (target.clone(), OUTSIDE_PROJECT.to_string())
        );
        let (_, e) = copy_inside(root, &root.join("map.omap"), &target).unwrap_err();
        assert_eq!(e.to_string(), OUTSIDE_PROJECT);
        let (_, e) = write_files_inside(root, &[(&target, b"ocd")]).unwrap_err();
        assert_eq!(e.to_string(), OUTSIDE_PROJECT);
        let (path, e) = check_folder_inside(root, &root.join("ocad")).unwrap_err();
        assert_eq!(
            (path, e.to_string()),
            (root.join("ocad"), OUTSIDE_PROJECT.to_string())
        );
        assert_eq!(
            check_inside(root, &target).unwrap_err().1.to_string(),
            OUTSIDE_PROJECT
        );
        // A folder below the link is not created either.
        assert!(write_inside(root, &root.join("ocad/new/a.png"), b"x").is_err());
        let written: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
        assert!(written.is_empty(), "{written:?}");
    }

    #[cfg(unix)]
    #[test]
    fn copies_read_nothing_through_a_link_or_from_outside() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let key = outside.path().join("id_rsa");
        std::fs::write(&key, "private").unwrap();
        let root = dir.path();
        let linked = root.join("templates/hillshade.png");
        std::fs::create_dir_all(linked.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&key, &linked).unwrap();
        let escaping = root
            .join("templates/../../")
            .join(key.strip_prefix(outside.path().parent().unwrap()).unwrap());
        let to = root.join("ocad/hillshade.png");
        for from in [&linked, &key, &escaping] {
            let (path, e) = copy_inside(root, from, &to).unwrap_err();
            assert_eq!((&path, e.to_string()), (from, NOT_COPIED.to_string()));
            let (path, e) = check_source(root, from).unwrap_err();
            assert_eq!((&path, e.to_string()), (from, NOT_COPIED.to_string()));
            assert!(!to.exists(), "{from:?}");
        }
    }
}
