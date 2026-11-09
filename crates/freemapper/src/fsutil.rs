//! File system helpers.

use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// Attempts to replace a file; a viewer holding it open (OOM on Windows) blocks only briefly.
const RENAME_ATTEMPTS: u32 = 3;
const RETRY_PAUSE: std::time::Duration = std::time::Duration::from_millis(200);

/// Names tried for a partial file before giving up; taken names are skipped, never reused.
const PART_NAME_ATTEMPTS: u32 = 16;

/// Numbers the partial files of this process, so no two writes pick the same name.
static NEXT_PART: AtomicU64 = AtomicU64::new(0);

/// What a copy says about a source that is a link or lies outside the project folder.
pub(crate) const NOT_COPIED: &str =
    "a link or outside the project folder, so freemapper does not copy it";

/// What a write into the project folder says when a link would lead it elsewhere.
pub(crate) const OUTSIDE_PROJECT: &str =
    "a link leads out of the project folder, so freemapper writes nothing there";

/// Writes the cache file `path`: `bytes` go into a new partial file in the same folder, which is
/// then renamed, so `path` never holds a partial file. Never follows a link: the partial file
/// is created new (a link or a file of that name is skipped), and the rename replaces a link at
/// `path` instead of writing to its target. Folders on the way may be links (the cache can live
/// on another drive), so this is for the cache only; project files go through
/// [`write_inside`]. On failure, returns the path the error is about.
pub(crate) fn write_cache_file(path: &Path, bytes: &[u8]) -> Result<(), (PathBuf, io::Error)> {
    write_files(None, &[(path, bytes)])
}

/// [`write_cache_file`] for a file of the project in `root`: refused when the folder of `path`
/// leads out of `root` through a link (see [`check_inside`]).
pub(crate) fn write_inside(
    root: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<(), (PathBuf, io::Error)> {
    write_files(Some(root), &[(path, bytes)])
}

/// Writes files of the project in `root` that belong together (an image and its world file):
/// first every file as a partial file, then renames them one after the other, retrying a
/// failed rename. When writing fails, the targets stay as they were and the partial files are
/// removed. Refused like [`write_inside`], and like it never follows a link.
pub(crate) fn write_files_inside(
    root: &Path,
    files: &[(&Path, &[u8])],
) -> Result<(), (PathBuf, io::Error)> {
    write_files(Some(root), files)
}

/// Copies `from` to `to` inside the project in `root` like [`write_inside`]: the copy goes
/// into a new partial file and is renamed, so a link at `to` is replaced, never followed. The
/// source must be a file of the project itself (see [`check_source`]).
pub(crate) fn copy_inside(root: &Path, from: &Path, to: &Path) -> Result<(), (PathBuf, io::Error)> {
    copy_inside_with(root, from, to, &|| {})
}

/// [`copy_inside`] that runs `between` after the checks, before the source is opened. Tests
/// swap the source there, as a local attacker might.
fn copy_inside_with(
    root: &Path,
    from: &Path,
    to: &Path,
    between: &dyn Fn(),
) -> Result<(), (PathBuf, io::Error)> {
    check_source(root, from)?;
    check_inside(root, to)?;
    between();
    let mut source = open_source(root, from)?;
    let part = write_part_with(Some(root), to, &mut |file| {
        io::copy(&mut source, file).map(|_| ())
    })?;
    rename_parts(&[(part, to)])
}

/// Opens the copy source `from` and makes sure the file opened is the one checked: a regular
/// file, no link, and the very file that `from` resolves to inside `root` after the open.
/// Someone who swaps the file or its folder for a link around the open gets [`NOT_COPIED`].
/// Errors name `from`.
fn open_source(root: &Path, from: &Path) -> Result<std::fs::File, (PathBuf, io::Error)> {
    open_source_with(root, from, &|| {})
}

/// [`open_source`] that runs `after_open` right after the open (tests swap the folder back
/// there).
fn open_source_with(
    root: &Path,
    from: &Path,
    after_open: &dyn Fn(),
) -> Result<std::fs::File, (PathBuf, io::Error)> {
    let named = |e: io::Error| (from.to_path_buf(), e);
    let refused = || named(io::Error::other(NOT_COPIED));
    let before = std::fs::symlink_metadata(from).map_err(named)?;
    let file = open_not_following(from).map_err(named)?;
    after_open();
    let opened = file.metadata().map_err(named)?;
    let inside = stays_inside(root, from);
    // The path as resolved now must lie in the project and name the opened file (volume and
    // file index on Windows, device and inode on Unix), so a folder swapped and swapped back
    // around the open does not pass.
    let resolved = std::fs::canonicalize(from).map_err(named)?;
    let root = std::fs::canonicalize(root).map_err(named)?;
    let opened_handle = same_file::Handle::from_file(file.try_clone().map_err(named)?);
    let resolved_handle = same_file::Handle::from_path(&resolved);
    let is_resolved = matches!((opened_handle, resolved_handle), (Ok(a), Ok(b)) if a == b);
    let after = std::fs::symlink_metadata(from).map_err(named)?;
    let unchanged = same_inode(&before, &opened) && same_inode(&after, &opened);
    let is_link = before.file_type().is_symlink() || after.file_type().is_symlink();
    if !opened.is_file()
        || is_reparse_point(&opened)
        || is_link
        || !unchanged
        || !inside
        || !resolved.starts_with(&root)
        || !is_resolved
    {
        return Err(refused());
    }
    Ok(file)
}

/// Opens `path` for reading. On Windows a link or junction at `path` is opened itself, not
/// its target, so [`is_reparse_point`] sees it.
fn open_not_following(path: &Path) -> io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        // FILE_FLAG_OPEN_REPARSE_POINT
        const OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(OPEN_REPARSE_POINT);
    }
    options.open(path)
}

/// Whether the metadata of an opened handle is that of a link or junction (Windows only; on
/// Unix opening follows links, and [`same_file`] catches them).
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        // FILE_ATTRIBUTE_REPARSE_POINT
        const REPARSE_POINT: u32 = 0x0400;
        metadata.file_attributes() & REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

/// Whether `a` and `b` describe the same file by device and inode (Unix). std has no stable
/// file id on Windows; there the `same_file` handle comparison in [`open_source_with`] does it.
fn same_inode(a: &std::fs::Metadata, b: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        a.dev() == b.dev() && a.ino() == b.ino()
    }
    #[cfg(not(unix))]
    {
        let _ = (a, b);
        true
    }
}

/// Renames the finished cache file `part` (a download created by [`create_new_file`]) to
/// `path`, retrying like the other writes. The rename replaces a link at `path` instead of
/// following it.
pub(crate) fn rename_cache_file(part: &Path, path: &Path) -> Result<(), (PathBuf, io::Error)> {
    retry(RENAME_ATTEMPTS, || std::fs::rename(part, path)).map_err(|e| (path.to_path_buf(), e))
}

/// Refuses `path` when its folder leads out of `root` once links are resolved (a linked
/// `data`, `templates` or `ocad` folder), with [`OUTSIDE_PROJECT`].
pub(crate) fn check_inside(root: &Path, path: &Path) -> Result<(), (PathBuf, io::Error)> {
    if stays_inside(root, path) {
        Ok(())
    } else {
        Err((path.to_path_buf(), io::Error::other(OUTSIDE_PROJECT)))
    }
}

/// Refuses a file to be copied that is a link or whose folder leads out of `root`, with
/// [`NOT_COPIED`]: a copy would bring a file from elsewhere (a key, a password store) into the
/// project. A source that does not exist passes; reading it then fails as usual.
pub(crate) fn check_source(root: &Path, from: &Path) -> Result<(), (PathBuf, io::Error)> {
    let is_link = std::fs::symlink_metadata(from).is_ok_and(|m| m.file_type().is_symlink());
    if is_link || !stays_inside(root, from) {
        return Err((from.to_path_buf(), io::Error::other(NOT_COPIED)));
    }
    Ok(())
}

/// Removes the project file `path` if it is there, unless its folder leads out of `root`
/// through a link (a link at `path` itself is removed, not its target). Best effort.
pub(crate) fn remove_inside(root: &Path, path: &Path) {
    if stays_inside(root, path) {
        let _ = std::fs::remove_file(path);
    }
}

/// Refuses the folder `folder` when it leads out of `root` through a link, with
/// [`OUTSIDE_PROJECT`]. A folder that does not exist yet passes: it is created as a folder.
pub(crate) fn check_folder_inside(root: &Path, folder: &Path) -> Result<(), (PathBuf, io::Error)> {
    check_inside(root, &folder.join("file")).map_err(|(_, e)| (folder.to_path_buf(), e))
}

/// Opens `path` as a new, empty file for a download: an entry of that name (a file a killed
/// run left, or a link) is removed first, so the download never writes through a link.
pub(crate) fn create_new_file(path: &Path) -> io::Result<std::fs::File> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    new_file(path)
}

/// Creates `path`; fails if anything of that name exists, also a link (it is not followed).
fn new_file(path: &Path) -> io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

fn write_files(root: Option<&Path>, files: &[(&Path, &[u8])]) -> Result<(), (PathBuf, io::Error)> {
    if let Some(root) = root {
        for (path, _) in files {
            check_inside(root, path)?;
        }
    }
    let mut parts: Vec<(PathBuf, &Path)> = Vec::new();
    for (path, bytes) in files {
        match write_part_with(root, path, &mut |file| file.write_all(bytes)) {
            Ok(part) => parts.push((part, path)),
            Err(e) => {
                remove_parts(&parts);
                return Err(e);
            }
        }
    }
    rename_parts(&parts)
}

fn remove_parts(parts: &[(PathBuf, &Path)]) {
    for (part, _) in parts {
        let _ = std::fs::remove_file(part);
    }
}

/// Renames each partial file over its target, retrying a failed rename; on failure the
/// partial files not yet renamed are removed.
fn rename_parts(parts: &[(PathBuf, &Path)]) -> Result<(), (PathBuf, io::Error)> {
    for (i, (part, path)) in parts.iter().enumerate() {
        if let Err(e) = retry(RENAME_ATTEMPTS, || std::fs::rename(part, path)) {
            remove_parts(&parts[i..]);
            return Err((path.to_path_buf(), e));
        }
    }
    Ok(())
}

/// Writes a new partial file next to `path` with `fill` and flushes it to the disk. The folder
/// is created if needed; with a `root`, it is checked again once it exists.
fn write_part_with(
    root: Option<&Path>,
    path: &Path,
    fill: &mut dyn FnMut(&mut std::fs::File) -> io::Result<()>,
) -> Result<PathBuf, (PathBuf, io::Error)> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| (dir.to_path_buf(), e))?;
    if let Some(root) = root {
        check_inside(root, path)?;
    }
    let names = (0..PART_NAME_ATTEMPTS).map(|_| part_name(path, NEXT_PART.fetch_add(1, Relaxed)));
    let (part, mut file) = create_part(path, names)?;
    let written = fill(&mut file).and_then(|()| file.sync_all());
    drop(file);
    if let Err(e) = written {
        // Half a file is of no use; on a full disk it also takes the space others need.
        let _ = std::fs::remove_file(&part);
        // The error names the file the user knows, not the partial file just removed.
        return Err((path.to_path_buf(), e));
    }
    Ok(part)
}

/// `<path>.<process id>.<number>.part`: the process id keeps concurrent runs apart.
fn part_name(path: &Path, number: u64) -> PathBuf {
    let mut part = path.as_os_str().to_owned();
    part.push(format!(".{}.{number}.part", std::process::id()));
    PathBuf::from(part)
}

/// Creates the first of `names` that does not exist yet. A name already taken (a planted
/// link, a file of another run) is skipped and left alone.
fn create_part(
    path: &Path,
    names: impl Iterator<Item = PathBuf>,
) -> Result<(PathBuf, std::fs::File), (PathBuf, io::Error)> {
    let mut taken = None;
    for name in names {
        match new_file(&name) {
            Ok(file) => return Ok((name, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => taken = Some(e),
            Err(e) => return Err((path.to_path_buf(), e)),
        }
    }
    let e = taken.unwrap_or_else(|| io::Error::from(io::ErrorKind::AlreadyExists));
    Err((path.to_path_buf(), e))
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

    #[test]
    fn pair_is_written_completely() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("t/a.png"), dir.path().join("t/a.pgw"));
        write_files(None, &[(&a, b"image"), (&b, b"world")]).unwrap();
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
        let (path, _) = write_files(None, &[(&a, b"new image"), (&b, b"world")]).unwrap_err();
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
        let mut full_disk = |file: &mut std::fs::File| -> io::Result<()> {
            file.write_all(b"a whole")?;
            Err(io::Error::from(io::ErrorKind::StorageFull))
        };
        let (path, e) = write_part_with(None, &target, &mut full_disk).unwrap_err();
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

    #[cfg(unix)]
    #[test]
    fn a_link_at_a_partial_file_name_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        let target = dir.path().join("a.png");
        // The name of the partial file of 0.6.1, planted before the run.
        let mut part = target.as_os_str().to_owned();
        part.push(format!(".{}.part", std::process::id()));
        std::os::unix::fs::symlink(&victim, &part).unwrap();
        write_cache_file(&target, b"image").unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert_eq!(std::fs::read(&target).unwrap(), b"image");
    }

    #[test]
    fn a_taken_partial_file_name_is_skipped_and_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a.png");
        let taken = part_name(&target, 0);
        std::fs::write(&taken, b"another run").unwrap();
        let fresh = part_name(&target, 1);
        let (part, _) = create_part(&target, [taken.clone(), fresh.clone()].into_iter()).unwrap();
        assert_eq!(part, fresh);
        assert_eq!(std::fs::read(&taken).unwrap(), b"another run");
        // Every name taken: an error that names the target, nothing overwritten.
        let (path, e) = create_part(&target, [taken.clone(), fresh].into_iter()).unwrap_err();
        assert_eq!(path, target);
        assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&taken).unwrap(), b"another run");
    }

    #[test]
    fn partial_file_names_differ_within_a_process() {
        let target = Path::new("a.png");
        let a = part_name(target, NEXT_PART.fetch_add(1, Relaxed));
        let b = part_name(target, NEXT_PART.fetch_add(1, Relaxed));
        assert_ne!(a, b);
        assert!(a.to_string_lossy().ends_with(".part"), "{a:?}");
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
    fn a_link_at_a_taken_partial_file_name_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        let target = dir.path().join("a.png");
        let planted = part_name(&target, 0);
        std::os::unix::fs::symlink(&victim, &planted).unwrap();
        let fresh = part_name(&target, 1);
        let (part, _) = create_part(&target, [planted, fresh.clone()].into_iter()).unwrap();
        assert_eq!(part, fresh);
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
    }

    #[cfg(unix)]
    #[test]
    fn links_at_final_names_are_replaced_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        let root = dir.path();
        let source = root.join("map.omap");
        std::fs::write(&source, "map").unwrap();
        let (written, copied) = (root.join("ATTRIBUTION.txt"), root.join("map.omap.bak"));
        for link in [&written, &copied] {
            std::os::unix::fs::symlink(&victim, link).unwrap();
        }
        write_inside(root, &written, b"credits").unwrap();
        copy_inside(root, &source, &copied).unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        for (file, text) in [(&written, "credits"), (&copied, "map")] {
            assert!(
                std::fs::symlink_metadata(file).unwrap().is_file(),
                "{file:?}"
            );
            assert_eq!(std::fs::read_to_string(file).unwrap(), text);
        }
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
        assert!(copy_inside(root, &root.join("map.omap"), &target).is_err());
        // A folder below the link is not created either.
        assert!(write_inside(root, &root.join("ocad/new/a.png"), b"x").is_err());
        let written: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
        assert!(written.is_empty(), "{written:?}");
    }

    #[cfg(unix)]
    #[test]
    fn a_download_replaces_a_link_instead_of_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        let part = dir.path().join("extract.osm.pbf.part");
        std::os::unix::fs::symlink(&victim, &part).unwrap();
        create_new_file(&part).unwrap().write_all(b"pbf").unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert_eq!(std::fs::read(&part).unwrap(), b"pbf");
    }

    #[cfg(unix)]
    #[test]
    fn copies_read_nothing_through_a_link_or_from_outside() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("id_rsa");
        std::fs::write(&secret, "secret").unwrap();
        let root = dir.path();
        let linked = root.join("templates/hillshade.png");
        std::fs::create_dir_all(linked.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&secret, &linked).unwrap();
        let escaping = root.join("templates/../../").join(
            secret
                .strip_prefix(outside.path().parent().unwrap())
                .unwrap(),
        );
        let to = root.join("ocad/hillshade.png");
        for from in [&linked, &secret, &escaping] {
            let (path, e) = copy_inside(root, from, &to).unwrap_err();
            assert_eq!((&path, e.to_string()), (from, NOT_COPIED.to_string()));
            assert!(!to.exists(), "{from:?}");
        }
    }

    #[test]
    fn a_missing_copy_source_is_named_in_the_error() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("templates/hillshade.png");
        let to = dir.path().join("ocad/hillshade.png");
        let (path, e) = copy_inside(dir.path(), &from, &to).unwrap_err();
        assert_eq!((path, e.kind()), (from, io::ErrorKind::NotFound));
        assert!(!to.exists());
    }

    /// A project with `templates/x.png` and, outside it, a secret of the user.
    #[cfg(unix)]
    fn project_and_secret() -> (tempfile::TempDir, tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("templates")).unwrap();
        std::fs::write(dir.path().join("templates/x.png"), "image").unwrap();
        let secret = outside.path().join("x.png");
        std::fs::write(&secret, "secret").unwrap();
        (dir, outside, secret)
    }

    #[cfg(unix)]
    #[test]
    fn a_source_swapped_for_a_link_before_the_copy_is_refused() {
        let (dir, _outside, secret) = project_and_secret();
        let (from, to) = (
            dir.path().join("templates/x.png"),
            dir.path().join("ocad/x.png"),
        );
        let swap = || {
            std::fs::remove_file(&from).unwrap();
            std::os::unix::fs::symlink(&secret, &from).unwrap();
        };
        let (path, e) = copy_inside_with(dir.path(), &from, &to, &swap).unwrap_err();
        assert_eq!((path, e.to_string()), (from, NOT_COPIED.to_string()));
        assert!(!to.exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_source_folder_swapped_for_a_link_before_the_copy_is_refused() {
        let (dir, outside, _secret) = project_and_secret();
        let templates = dir.path().join("templates");
        let (from, to) = (templates.join("x.png"), dir.path().join("ocad/x.png"));
        let swap = || {
            std::fs::remove_dir_all(&templates).unwrap();
            std::os::unix::fs::symlink(outside.path(), &templates).unwrap();
        };
        let (path, e) = copy_inside_with(dir.path(), &from, &to, &swap).unwrap_err();
        assert_eq!((path, e.to_string()), (from, NOT_COPIED.to_string()));
        assert!(!to.exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_source_folder_swapped_around_the_open_is_refused() {
        let (dir, outside, _secret) = project_and_secret();
        let templates = dir.path().join("templates");
        let from = templates.join("x.png");
        let kept = dir.path().join("kept");
        // The folder is a link to the secret's folder when the file is opened, and back when
        // the path is checked again.
        std::fs::rename(&templates, &kept).unwrap();
        std::os::unix::fs::symlink(outside.path(), &templates).unwrap();
        let swap_back = || {
            std::fs::remove_file(&templates).unwrap();
            std::fs::rename(&kept, &templates).unwrap();
        };
        let (path, e) = open_source_with(dir.path(), &from, &swap_back).unwrap_err();
        assert_eq!((path, e.to_string()), (from, NOT_COPIED.to_string()));
    }
}
