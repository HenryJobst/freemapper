//! Disk space: how much the volumes freemapper writes to have free, and which parts of a fetch
//! fit there while a reserve stays free.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::layers::DownloadEstimate;

/// Space every check leaves free on a volume, for the system and other programs.
pub const RESERVE_BYTES: u64 = 1 << 30;

/// One MB as the download estimate counts it.
const MB: f64 = (1u64 << 20) as f64;

/// A part of a fetch: downloads freemapper loads or leaves out together. The order is the order
/// in which they are kept when space runs short.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// ALKIS buildings and land use, and an OSM extract not cached yet: the map objects.
    MapData,
    /// The terrain model: hillshade, slope and contours.
    Terrain,
    /// The orthophoto, the template the mapper draws on.
    Orthophoto,
    /// The extra Berlin layers (ALKIS structures, tree cadastre): candidates and a few map
    /// objects, optional like the K5 map.
    BerlinLayers,
    /// The surface model (nDOM, bDOM, DOM1): only the nDSM template.
    Surface,
    /// A newer OSM extract where an older one is cached; the older one serves meanwhile.
    OsmUpdate,
    /// The Berlin K5 map (1:5000), a template the mapper may draw on.
    K5,
}

impl Part {
    /// Whether a run cannot go on without the part: derive needs the terrain model and the
    /// orthophoto, and the map objects come from the map data.
    pub fn required(self) -> bool {
        matches!(self, Part::MapData | Part::Terrain | Part::Orthophoto)
    }

    pub fn label(self) -> &'static str {
        match self {
            Part::MapData => "the ALKIS and OSM map data",
            Part::Terrain => "the terrain model",
            Part::Orthophoto => "the orthophoto",
            Part::Surface => "the surface model for the nDSM template",
            Part::OsmUpdate => "the update of the cached OSM extract",
            Part::K5 => "the Berlin K5 map template",
            Part::BerlinLayers => "the Berlin ALKIS structures and tree cadastre",
        }
    }

    /// What the files derived from a download (mosaics in `data/`, templates) add in the project
    /// folder, as a share of the download. Generous on purpose: the templates of a raster are
    /// about as large as its download, the nDSM template is smaller than the surface model it
    /// comes from, and the OSM candidates are a small cut of the extract. Where the Berlin
    /// layers have stairs, `derive` keeps the DGM mosaic as `data/terrain.tif` (Float32, LZW,
    /// about 4 MB per km², as much as the layers): their share covers it.
    fn derived_share(self) -> f64 {
        match self {
            // The K5 image is written straight into the templates, so it needs no extra room.
            Part::MapData | Part::OsmUpdate | Part::K5 => 0.0,
            Part::Terrain | Part::Orthophoto | Part::BerlinLayers => 1.0,
            Part::Surface => 0.5,
        }
    }
}

/// Where a download is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    /// The cache shared by all projects.
    Cache,
    /// The project folder (`data/`).
    Project,
}

/// The volume a path lies on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// Equal for two paths on the same volume.
    pub id: String,
    /// Bytes this user may still write there.
    pub available: u64,
}

/// Free space of the volume holding a path: the system's in production, a fixed one in tests.
pub trait DiskSpace: std::fmt::Debug + Send + Sync {
    /// The volume holding `path`, or the folder it will be created in.
    fn volume(&self, path: &Path) -> io::Result<Volume>;
}

/// The free space the operating system reports.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemDiskSpace;

impl DiskSpace for SystemDiskSpace {
    fn volume(&self, path: &Path) -> io::Result<Volume> {
        let existing = existing_ancestor(path)?;
        Ok(Volume {
            id: volume_id(&existing)?,
            available: fs4::available_space(&existing)?,
        })
    }
}

/// `path` or the nearest folder above it that exists: a project folder or cache not created yet
/// is created on that folder's volume.
fn existing_ancestor(path: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    absolute
        .ancestors()
        .find(|p| p.exists())
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no existing folder above it"))
}

#[cfg(unix)]
fn volume_id(path: &Path) -> io::Result<String> {
    use std::os::unix::fs::MetadataExt as _;
    Ok(std::fs::metadata(path)?.dev().to_string())
}

/// Windows: the drive or network share the path starts with.
#[cfg(not(unix))]
fn volume_id(path: &Path) -> io::Result<String> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(canonical
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .unwrap_or_default())
}

/// Whether `e` says the volume is full or the user's quota is used up. The standard library maps
/// the systems' codes to these kinds: ENOSPC and EDQUOT on Linux and macOS, ERROR_DISK_FULL,
/// ERROR_HANDLE_DISK_FULL and ERROR_DISK_QUOTA_EXCEEDED on Windows.
pub fn is_storage_full(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded
    )
}

/// The text of an error writing or reading `path`: plain words when the disk is full, else the
/// path and the system's message.
pub fn io_error_text(path: &Path, e: &io::Error) -> String {
    if is_storage_full(e) {
        format!(
            "the disk is full, so freemapper could not write {}. Free some space and try again",
            path.display()
        )
    } else {
        format!("{}: {e}", path.display())
    }
}

/// What a fetch needs on one volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeSpace {
    /// The folder checked there: the cache folder, or the project folder on a volume of its own.
    pub path: PathBuf,
    pub holds_cache: bool,
    pub holds_project: bool,
    pub available: u64,
    /// Bytes all downloads of the fetch and the files derived from them need there.
    pub needed: u64,
}

/// A part of a fetch that does not fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortfall {
    pub part: Part,
    /// The folder of the volume where it does not fit.
    pub path: PathBuf,
    /// Bytes the part needs there.
    pub needed: u64,
    /// Bytes free there beyond the reserve, after the parts kept before it.
    pub left: u64,
}

impl std::fmt::Display for Shortfall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.part.required() {
            write!(f, "not enough disk space for {}", self.part.label())?;
        } else {
            write!(
                f,
                "freemapper skips {} for lack of disk space",
                self.part.label()
            )?;
        }
        write!(
            f,
            ". It needs {}, but {} has only {} free beyond the {} reserve. Free some space and run `freemapper fetch` again",
            size(self.needed),
            self.path.display(),
            size_down(self.left),
            size(RESERVE_BYTES)
        )
    }
}

/// Which parts of a fetch fit on the volumes of the cache and the project folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacePlan {
    /// The cache's volume first, then the project folder's when it is another one.
    pub volumes: Vec<VolumeSpace>,
    /// The parts that do not fit, in their order.
    pub short: Vec<Shortfall>,
}

impl SpacePlan {
    pub fn fits(&self, part: Part) -> bool {
        !self.short.iter().any(|s| s.part == part)
    }

    /// The first part the run needs that does not fit.
    pub fn missing(&self) -> Option<&Shortfall> {
        self.short.iter().find(|s| s.part.required())
    }

    /// The parts freemapper leaves out.
    pub fn skipped(&self) -> impl Iterator<Item = &Shortfall> {
        self.short.iter().filter(|s| !s.part.required())
    }
}

/// Checks the downloads of `estimate` (those not cached yet, plus the files derived from them)
/// against the free space of the volumes of `cache_dir` and `project_dir`, keeping
/// [`RESERVE_BYTES`] free on each. The parts the run needs come first; the others are kept in
/// their order as long as they fit, so a later small part may stay where a larger one is left
/// out. Fails with the folder whose free space cannot be read.
pub fn plan(
    estimate: &DownloadEstimate,
    cache_dir: &Path,
    project_dir: &Path,
    space: &dyn DiskSpace,
) -> Result<SpacePlan, (PathBuf, io::Error)> {
    let volume = |dir: &Path| space.volume(dir).map_err(|e| (dir.to_path_buf(), e));
    let cache = volume(cache_dir)?;
    let project = volume(project_dir)?;
    let shared = cache.id == project.id;
    let mut volumes = vec![VolumeSpace {
        path: cache_dir.to_path_buf(),
        holds_cache: true,
        holds_project: shared,
        available: cache.available,
        needed: 0,
    }];
    if !shared {
        volumes.push(VolumeSpace {
            path: project_dir.to_path_buf(),
            holds_cache: false,
            holds_project: true,
            available: project.available,
            needed: 0,
        });
    }
    let project_index = volumes.len() - 1;
    // Bytes per part and volume; ordered by part, so the required parts come first.
    let mut needs: BTreeMap<Part, Vec<u64>> = BTreeMap::new();
    for item in estimate.items.iter().filter(|i| !i.cached) {
        let bytes = |mb: f64| (mb * MB).ceil() as u64;
        let need = needs
            .entry(item.part)
            .or_insert_with(|| vec![0; volumes.len()]);
        let stored = match item.store {
            Store::Cache => 0,
            Store::Project => project_index,
        };
        need[stored] += bytes(item.megabytes);
        // Files that replace stored ones of the area need no new room once written.
        if !estimate.in_place.contains(&item.part) {
            need[project_index] += bytes(item.megabytes * item.part.derived_share());
        }
    }
    for (i, volume) in volumes.iter_mut().enumerate() {
        volume.needed = needs.values().map(|need| need[i]).sum();
    }
    let mut left: Vec<u64> = volumes
        .iter()
        .map(|v| v.available.saturating_sub(RESERVE_BYTES))
        .collect();
    let mut short = Vec::new();
    for (&part, need) in &needs {
        match (0..volumes.len()).find(|&i| need[i] > left[i]) {
            Some(i) => short.push(Shortfall {
                part,
                path: volumes[i].path.clone(),
                needed: need[i],
                left: left[i],
            }),
            None => {
                for (left, need) in left.iter_mut().zip(need) {
                    *left -= need;
                }
            }
        }
    }
    Ok(SpacePlan { volumes, short })
}

/// `350 MB` below 1 GB, else `1.2 GB` (MB and GB of 1024² and 1024³ bytes, as the estimate
/// counts).
pub fn size(bytes: u64) -> String {
    let mb = bytes.div_ceil(1 << 20);
    if mb < 1024 {
        format!("{mb} MB")
    } else {
        format!("{:.1} GB", bytes as f64 / (1u64 << 30) as f64)
    }
}

/// [`size`] rounded down: what is free must not read as more than it is.
fn size_down(bytes: u64) -> String {
    let mb = bytes >> 20;
    if mb < 1024 {
        format!("{mb} MB")
    } else {
        let tenths = (bytes as f64 / (1u64 << 30) as f64 * 10.0).floor();
        format!("{:.1} GB", tenths / 10.0)
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// Volumes by folder: a path lies on the volume of the longest folder it starts with.
    #[derive(Debug, Default)]
    pub struct FixedSpace {
        volumes: Vec<(PathBuf, Volume)>,
    }

    impl FixedSpace {
        pub fn with(mut self, folder: &Path, id: &str, available: u64) -> Self {
            self.volumes.push((
                folder.to_path_buf(),
                Volume {
                    id: id.to_string(),
                    available,
                },
            ));
            self
        }
    }

    impl DiskSpace for FixedSpace {
        fn volume(&self, path: &Path) -> io::Result<Volume> {
            self.volumes
                .iter()
                .filter(|(folder, _)| path.starts_with(folder))
                .max_by_key(|(folder, _)| folder.components().count())
                .map(|(_, volume)| volume.clone())
                .ok_or_else(|| io::Error::other("no such volume"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::FixedSpace;
    use super::*;
    use crate::layers::EstimateItem;

    const GB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;

    fn item(part: Part, store: Store, megabytes: f64, cached: bool) -> EstimateItem {
        EstimateItem {
            what: format!("{part:?}"),
            megabytes,
            cached,
            part,
            store,
        }
    }

    /// An LGB-like fetch: ALKIS, an OSM extract, DTM, orthophoto and nDSM tiles in the cache.
    fn estimate(items: Vec<EstimateItem>) -> DownloadEstimate {
        let total_megabytes = items
            .iter()
            .filter(|i| !i.cached)
            .map(|i| i.megabytes)
            .sum();
        DownloadEstimate {
            tiles: Vec::new(),
            items,
            total_megabytes,
            in_place: Vec::new(),
        }
    }

    fn typical() -> DownloadEstimate {
        estimate(vec![
            item(Part::MapData, Store::Project, 1.0, false),
            item(Part::MapData, Store::Cache, 100.0, false),
            item(Part::Terrain, Store::Cache, 10.0, false),
            item(Part::Orthophoto, Store::Cache, 50.0, false),
            item(Part::Surface, Store::Cache, 200.0, false),
        ])
    }

    fn one_volume(available: u64) -> FixedSpace {
        FixedSpace::default().with(Path::new("/"), "disk", available)
    }

    fn plan_on(space: &FixedSpace, estimate: &DownloadEstimate) -> SpacePlan {
        plan(
            estimate,
            Path::new("/cache/freemapper"),
            Path::new("/maps/school"),
            space,
        )
        .unwrap()
    }

    #[test]
    fn everything_fits_changes_nothing() {
        let plan = plan_on(&one_volume(100 * GB), &typical());
        assert!(plan.short.is_empty(), "{:?}", plan.short);
        assert!(plan.missing().is_none());
        for part in [
            Part::MapData,
            Part::Terrain,
            Part::Orthophoto,
            Part::Surface,
        ] {
            assert!(plan.fits(part), "{part:?}");
        }
        // Downloads plus derived files: 101 + 10·2 + 50·2 + 200·1.5 MB.
        assert_eq!(plan.volumes.len(), 1);
        assert_eq!(plan.volumes[0].needed, 521 * MIB);
        assert!(plan.volumes[0].holds_cache && plan.volumes[0].holds_project);
    }

    #[test]
    fn parts_that_do_not_fit_are_skipped() {
        // The required parts need 221 MB; 300 MB beyond the reserve leave no room for the
        // 300 MB of the surface model.
        let plan = plan_on(&one_volume(RESERVE_BYTES + 300 * MIB), &typical());
        assert!(plan.missing().is_none());
        let skipped: Vec<&Shortfall> = plan.skipped().collect();
        assert_eq!(skipped.len(), 1, "{skipped:?}");
        assert_eq!(skipped[0].part, Part::Surface);
        assert_eq!(skipped[0].needed, 300 * MIB);
        assert_eq!(skipped[0].left, 79 * MIB);
        assert!(!plan.fits(Part::Surface) && plan.fits(Part::Orthophoto));
        assert_eq!(
            skipped[0].to_string(),
            "freemapper skips the surface model for the nDSM template for lack of disk space. It needs 300 MB, but /cache/freemapper has only 79 MB free beyond the 1.0 GB reserve. Free some space and run `freemapper fetch` again"
        );
    }

    #[test]
    fn a_smaller_later_part_fits_where_a_larger_one_does_not() {
        let mut estimate = typical();
        estimate
            .items
            .push(item(Part::OsmUpdate, Store::Cache, 50.0, false));
        let plan = plan_on(&one_volume(RESERVE_BYTES + 300 * MIB), &estimate);
        assert!(!plan.fits(Part::Surface));
        assert!(plan.fits(Part::OsmUpdate));
    }

    #[test]
    fn berlin_layers_are_optional_and_leave_room_for_the_terrain_mosaic() {
        assert!(!Part::BerlinLayers.required());
        let mut estimate = typical();
        estimate
            .items
            .push(item(Part::BerlinLayers, Store::Project, 10.0, false));
        // The layers (10 MB) and `data/terrain.tif`, which `derive` keeps where they have
        // stairs and which is about as large: 20 MB on top of the 521 MB of `typical`.
        let plan = plan_on(&one_volume(100 * GB), &estimate);
        assert_eq!(plan.volumes[0].needed, 541 * MIB);
        // Too little room for them: skipped with a warning, the fetch goes on.
        let plan = plan_on(&one_volume(RESERVE_BYTES + 230 * MIB), &estimate);
        assert!(plan.missing().is_none());
        assert!(!plan.fits(Part::BerlinLayers));
        let skipped: Vec<Part> = plan.skipped().map(|s| s.part).collect();
        assert_eq!(skipped, [Part::BerlinLayers, Part::Surface]);
    }

    #[test]
    fn required_part_that_does_not_fit_is_missing() {
        let plan = plan_on(&one_volume(RESERVE_BYTES + 100 * MIB), &typical());
        let missing = plan.missing().expect("the OSM extract does not fit");
        assert_eq!(missing.part, Part::MapData);
        assert_eq!(
            missing.to_string(),
            "not enough disk space for the ALKIS and OSM map data. It needs 101 MB, but /cache/freemapper has only 100 MB free beyond the 1.0 GB reserve. Free some space and run `freemapper fetch` again"
        );
    }

    #[test]
    fn reserve_is_kept() {
        let estimate = estimate(vec![item(Part::Surface, Store::Cache, 100.0, false)]);
        let need = 150 * MIB;
        let just = plan_on(&one_volume(RESERVE_BYTES + need), &estimate);
        assert!(just.fits(Part::Surface));
        let short = plan_on(&one_volume(RESERVE_BYTES + need - 1), &estimate);
        assert!(!short.fits(Part::Surface));
        // Below the reserve even a download of nothing is fine: nothing is written.
        let nothing = estimate_with_cached_only();
        assert!(plan_on(&one_volume(GB / 2), &nothing).short.is_empty());
    }

    fn estimate_with_cached_only() -> DownloadEstimate {
        estimate(vec![item(Part::Surface, Store::Cache, 100.0, true)])
    }

    #[test]
    fn parts_replaced_in_place_need_room_for_the_download_only() {
        let mut estimate = estimate(vec![
            item(Part::Terrain, Store::Project, 100.0, false),
            item(Part::Surface, Store::Cache, 100.0, false),
        ]);
        estimate.in_place = vec![Part::Terrain];
        let plan = plan_on(&one_volume(100 * GB), &estimate);
        // The new terrain model lands beside the old one before it replaces it; its templates
        // replace the old ones. The surface model is new to the project.
        assert_eq!(plan.volumes[0].needed, (100 + 150) * MIB);
    }

    #[test]
    fn cached_downloads_need_no_space() {
        let mut estimate = typical();
        for item in &mut estimate.items {
            item.cached = true;
        }
        let plan = plan_on(&one_volume(0), &estimate);
        assert!(plan.short.is_empty());
        assert_eq!(plan.volumes[0].needed, 0);
    }

    #[test]
    fn cache_and_project_on_different_volumes_are_checked_separately() {
        // A WMS orthophoto goes into the project folder, the tiles into the cache.
        let estimate = estimate(vec![
            item(Part::Terrain, Store::Cache, 100.0, false),
            item(Part::Orthophoto, Store::Project, 100.0, false),
        ]);
        let space = FixedSpace::default()
            .with(Path::new("/cache"), "ssd", RESERVE_BYTES + 150 * MIB)
            .with(Path::new("/maps"), "usb", RESERVE_BYTES + 250 * MIB);
        let plan = plan_on(&space, &estimate);
        assert_eq!(plan.volumes.len(), 2);
        // The cache holds the tiles; the project folder the orthophoto, its template and the
        // terrain mosaic and templates.
        assert_eq!(plan.volumes[0].needed, 100 * MIB);
        assert_eq!(plan.volumes[1].needed, 300 * MIB);
        assert!(plan.volumes[1].holds_project && !plan.volumes[1].holds_cache);
        assert!(plan.fits(Part::Terrain));
        let missing = plan.missing().expect("the project folder is too small");
        assert_eq!(missing.part, Part::Orthophoto);
        assert_eq!(missing.path, Path::new("/maps/school"));
        assert_eq!((missing.needed, missing.left), (200 * MIB, 150 * MIB));
        // On one volume the same space adds up.
        let shared = FixedSpace::default().with(Path::new("/"), "ssd", RESERVE_BYTES + 400 * MIB);
        let plan = plan_on(&shared, &estimate);
        assert_eq!(plan.volumes.len(), 1);
        assert_eq!(plan.volumes[0].needed, 400 * MIB);
        assert!(plan.short.is_empty());
    }

    #[test]
    fn unreadable_space_names_the_folder() {
        let space = FixedSpace::default().with(Path::new("/cache"), "ssd", GB);
        let (path, _) = plan(
            &typical(),
            Path::new("/cache/freemapper"),
            Path::new("/maps/school"),
            &space,
        )
        .unwrap_err();
        assert_eq!(path, Path::new("/maps/school"));
    }

    #[test]
    fn sizes_read_as_mb_or_gb() {
        assert_eq!(size(0), "0 MB");
        assert_eq!(size(1), "1 MB");
        assert_eq!(size(350 * MIB), "350 MB");
        assert_eq!(size(GB), "1.0 GB");
        assert_eq!(size(GB + 3 * GB / 10), "1.3 GB");
        // Free space rounds down, so a shortfall of one byte still reads as one.
        assert_eq!(size(23 * MIB + 1), "24 MB");
        assert_eq!(size_down(24 * MIB - 1), "23 MB");
        assert_eq!(size_down(2 * GB - 1), "1.9 GB");
    }

    #[test]
    fn storage_full_errors_are_recognised() {
        assert!(is_storage_full(&io::Error::from(
            io::ErrorKind::StorageFull
        )));
        assert!(is_storage_full(&io::Error::from(
            io::ErrorKind::QuotaExceeded
        )));
        // The system's own codes: ENOSPC, EDQUOT; ERROR_DISK_FULL, ERROR_HANDLE_DISK_FULL.
        #[cfg(unix)]
        assert!(is_storage_full(&io::Error::from_raw_os_error(28)));
        #[cfg(target_os = "macos")]
        assert!(is_storage_full(&io::Error::from_raw_os_error(69)));
        #[cfg(target_os = "linux")]
        {
            assert!(is_storage_full(&io::Error::from_raw_os_error(122)));
            // ESRMNT, not a full disk.
            assert!(!is_storage_full(&io::Error::from_raw_os_error(69)));
        }
        #[cfg(windows)]
        for code in [39, 112] {
            assert!(is_storage_full(&io::Error::from_raw_os_error(code)));
        }
        assert!(!is_storage_full(&io::Error::from(
            io::ErrorKind::PermissionDenied
        )));
    }

    #[test]
    fn disk_full_errors_say_so() {
        let path = PathBuf::from("/maps/school/templates/orthophoto.jpg");
        let full = || io::Error::from(io::ErrorKind::StorageFull);
        let expected = "the disk is full, so freemapper could not write /maps/school/templates/orthophoto.jpg. Free some space and try again";
        assert_eq!(
            crate::fetch::FetchError::Io(path.clone(), full()).to_string(),
            expected
        );
        assert_eq!(
            crate::derive::DeriveError::Io(path.clone(), full()).to_string(),
            expected
        );
        assert_eq!(
            crate::project::ProjectError::Io(path.clone(), full()).to_string(),
            expected
        );
        // Other errors keep their own text.
        let denied = io::Error::from(io::ErrorKind::PermissionDenied);
        let text = crate::project::ProjectError::Io(path, denied).to_string();
        assert!(
            text.starts_with("/maps/school/templates/orthophoto.jpg: "),
            "{text}"
        );
    }

    #[test]
    fn system_space_of_a_folder_not_created_yet() {
        let dir = tempfile::tempdir().unwrap();
        let here = SystemDiskSpace.volume(dir.path()).unwrap();
        let below = SystemDiskSpace
            .volume(&dir.path().join("new/project"))
            .unwrap();
        assert_eq!(here.id, below.id);
        assert!(here.available > 0);
    }
}
