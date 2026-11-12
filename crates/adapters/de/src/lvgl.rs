//! LVGL Saarland open data (dl-de/by-2-0, "© GeoBasis DE/LVGL-SL"), all in EPSG:25832: the
//! orthophoto from the free WMS, the terrain and the surface model (2025) from the district
//! archives of the open-data share, read tile by tile with range requests.
//!
//! The two INSPIRE WCS services of the state are not asked: their own capabilities still say
//! that use beyond viewing needs a contract (`docs/regions/saarland-reference.md`).

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::{AdapterId, WcsRecord, stored_bbox};
use freemapper_sources::raster_fetch::{
    SURFACE_RELPATH, TERRAIN_RELPATH, fetch_wms_orthophoto, keep_stored_surface, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher, write_cache_file};
use crate::note::{Arg, Note, note};
use crate::raster::{GridSpec, Raster};
use crate::sources::alkis_wfs::WfsService;
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::region::{SAARLAND, tile_centre_in};
use crate::sources::wms::{DateLabel, WmsService};
use crate::sources::zip_range::{
    Member, cached_member, find_member, parse_directory, read_directory_bytes, refused,
};
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS of the newest flight (2025), free to use by its capabilities and
/// dl-de/by-2-0 by the catalogue.
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://geoportal.saarland.de/freewms/dop2025"),
    layer: Cow::Borrowed("sl_dop20_rgb"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};
/// The queryable layer with the date of each 2 km block.
const WMS_INFO_LAYER: &str = "sl_dop_info";
/// freemapper's name for the date taken as the flight date. The service calls it
/// `Aktualisierung` (update) and gives no other date for a block.
pub const FLIGHT_LABEL: DateLabel = DateLabel {
    record: "Aktualisierung",
    shown: "date of the update",
};

/// The simplified ALKIS of the state: the WFS of the geoportal in front of an ArcGIS server,
/// which writes the attribute names in upper case and refuses `SORTBY`.
pub const ALKIS: WfsService = WfsService {
    name: Cow::Borrowed("LVGL ALKIS"),
    url: Cow::Borrowed("https://geoportal.saarland.de/registry/wfs/414"),
    buildings_type: Cow::Borrowed("ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ALKIS_ALKIS_WFS_ohne_Eig:Nutzung"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Saarland)")),
    sorted: false,
};

/// The open-data share of the LVGL, as the download feeds of DGM1 and DOM1 name it.
pub const SHARE: &str =
    "https://www.shop.lvgl.saarland.de/cloud/public.php/dav/files/NK8ndP55qAqGEZD";

/// The districts the archives are cut by, in the order they are looked through.
pub const DISTRICTS: [&str; 6] = ["MZG", "NK", "SB", "SLS", "SPK", "WND"];

/// The date (`DD.MM.YYYY`) of the orthophoto block at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the metadata layer; `None` when the answer has none.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        ((bbox.min_e + bbox.max_e) / 2.0).round(),
        ((bbox.min_n + bbox.max_n) / 2.0).round(),
    );
    let url = format!(
        "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={WMS_INFO_LAYER}\
         &QUERY_LAYERS={WMS_INFO_LAYER}&STYLES=&CRS=EPSG:25832&BBOX={},{},{},{}\
         &WIDTH=101&HEIGHT=101&I=50&J=50&INFO_FORMAT=text/html",
        WMS.url,
        e - 50.0,
        n - 50.0,
        e + 50.0,
        n + 50.0
    );
    let bytes = crate::fetch::get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    update_date(&String::from_utf8_lossy(&bytes))
}

/// The cell after `Aktualisierung` in the table the feature info answers, when it is a date.
fn update_date(html: &str) -> Option<String> {
    let row = html.split("Aktualisierung").nth(1)?;
    let cell = row.split("<td>").nth(1)?.split("</td>").next()?.trim();
    let is_date = cell.len() == 10
        && cell.bytes().enumerate().all(|(at, byte)| match at {
            2 | 5 => byte == b'.',
            _ => byte.is_ascii_digit(),
        });
    is_date.then(|| cell.to_string())
}

/// A height model of the 2025 laser scan: 1 km Float32 GeoTIFF tiles in one archive per
/// district.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain.
    Dgm1,
    /// DOM1, the surface with buildings and vegetation.
    Dom1,
}

impl Model {
    /// The folder of the share that holds the archives of the model.
    pub fn folder(self) -> &'static str {
        match self {
            Self::Dgm1 => "OD_DGM1_2025_tif_LK",
            Self::Dom1 => "OD_DOM1_2025_tif_LK",
        }
    }

    /// The archive of `district` (one of [`DISTRICTS`]).
    pub fn archive_url(self, district: &str) -> String {
        let product = match self {
            Self::Dgm1 => "DGM1",
            Self::Dom1 => "DOM1",
        };
        format!(
            "{SHARE}/{}/{product}_tif_{district}_EPSG-25832_Entstehung-2025.zip",
            self.folder()
        )
    }

    /// The release the cache keeps apart, so tiles of a newer survey never mix with these.
    pub fn release(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1-2025",
            Self::Dom1 => "dom1-2025",
        }
    }

    /// When the release was surveyed, as recorded in the project.
    pub fn date(self) -> &'static str {
        "2025"
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LVGL DGM1 2025)",
            Self::Dom1 => "surface model (LVGL DOM1 2025)",
        }
    }

    /// Start of the tile file names, e.g. `dgm1_32_348_5475_1_SL_2025.tif`.
    fn file_prefix(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom1",
        }
    }

    /// The folder of the cache that holds the tiles and the directories of this release.
    fn cache_folder(self, cache: &CacheDir) -> PathBuf {
        cache
            .provider_dir(PROVIDER)
            .join(self.file_prefix())
            .join(self.release())
    }

    /// Where a tile of this model is cached: named after the tile, never the archive member.
    pub fn tile_cache_file(self, cache: &CacheDir, tile: TileId) -> PathBuf {
        self.cache_folder(cache).join(format!(
            "{}_32_{:03}_{:04}_.tif",
            self.file_prefix(),
            tile.east_km,
            tile.north_km
        ))
    }
}

/// Whether `error` is what an archive or its server answered: not there, refused, broken. Only
/// such an error lets the other archives go on. A cancel, an address freemapper does not ask
/// and an error of this computer (a full disk) stop the fetch.
fn of_the_archive(error: &FetchError) -> bool {
    matches!(
        error,
        FetchError::Http { .. }
            | FetchError::NotFound(_)
            | FetchError::Refused { .. }
            | FetchError::TooLarge { .. }
            | FetchError::RangeIgnored { .. }
            | FetchError::CorruptArchive { .. }
    )
}

/// The directory of one archive.
struct Listing {
    members: Vec<Member>,
    /// Why the archive could not be read online, when `members` are those of the directory an
    /// earlier run kept.
    kept_because: Option<FetchError>,
}

/// The members of the archive of `district`: online from the archive, kept in the cache, and
/// from the cache when the archive cannot be read just now; offline from the cache, and none
/// when the directory was never read (a tile that was fetched has its directory there).
fn directory(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    district: &str,
) -> Result<Listing, FetchError> {
    let url = model.archive_url(district);
    // The member count (u16, little-endian), then the central directory.
    let file = model
        .cache_folder(cache)
        .join(format!("directory-{district}.bin"));
    // The directory kept by an earlier run; `None` when there is none.
    let kept = || -> Option<Result<Vec<Member>, FetchError>> {
        let cached = std::fs::read(&file).ok()?;
        let (count, directory) = cached.split_at_checked(2)?;
        Some(
            parse_directory(directory, u16::from_le_bytes([count[0], count[1]]))
                .map_err(|message| refused(&url, message)),
        )
    };
    let fresh = |members| Listing {
        members,
        kept_because: None,
    };
    match mode {
        FetchMode::Online => {
            let read = read_directory_bytes(fetcher, &url).and_then(|(directory, count)| {
                let members =
                    parse_directory(&directory, count).map_err(|message| refused(&url, message))?;
                Ok((members, directory, count))
            });
            match read {
                Ok((members, directory, count)) => {
                    // Kept only once it parsed, so a broken answer never replaces a good copy.
                    // A directory that cannot be stored is an error of this computer and is
                    // never taken for an archive that is down.
                    write_cache_file(&file, &[&count.to_le_bytes()[..], &directory].concat())?;
                    Ok(fresh(members))
                }
                // The archive is down: the tiles of the kept directory that are in the cache
                // need nothing from it.
                Err(e) if of_the_archive(&e) => match kept() {
                    Some(Ok(members)) => Ok(Listing {
                        members,
                        kept_because: Some(e),
                    }),
                    _ => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        FetchMode::Offline => kept().unwrap_or_else(|| Ok(Vec::new())).map(fresh),
    }
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), read out of the district archives and
/// cached, mosaicked into a Float32 GeoTIFF `out` with its world file. A tile is looked for in
/// the archives in the order of [`DISTRICTS`], and the directory of an archive is read only
/// when the archives before it do not hold a tile. A tile on a district border is in the
/// archive of each district it touches, whole and the same in each (the 294 such tiles of
/// either model had one CRC-32 per tile on 2026-11-12), so the first archive that holds it is
/// as good as any. Tiles outside Saarland are in no archive and stay empty; returned are the
/// tiles inside Saarland that no archive holds. No tile at all is an error (the archives were
/// renamed or replaced by a newer survey). An archive that cannot be read is passed over; it
/// is the error only when a tile inside Saarland is in none of the others. When the fetch
/// goes through, `notes` gets a warning for every archive that was passed over and for every
/// archive whose directory of an earlier run stood in for it.
#[allow(clippy::too_many_arguments)]
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    notes: &mut dyn FnMut(Note),
) -> Result<Vec<TileId>, FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let folder = format!("{SHARE}/{}", model.folder());
    let offline_hint = |what: &str| {
        refused(
            &folder,
            format!("offline and not cached: {what}; run `freemapper fetch` online"),
        )
    };
    let mut directories: Vec<Option<Vec<Member>>> = DISTRICTS.iter().map(|_| None).collect();
    // The archives that could not be read, with the reason, and whether the directory of an
    // earlier run stood in. The tiles of an archive that was passed over may be in another
    // one too; only a tile that none of the others holds makes its reason the error.
    let mut unread: Vec<(&str, FetchError, bool)> = Vec::new();
    let mut passed_over: Option<usize> = None;
    let mut found = 0;
    let mut missing = Vec::new();
    for tile in tiles_covering(bbox) {
        let prefix = format!(
            "{}_32_{:03}_{:04}_",
            model.file_prefix(),
            tile.east_km,
            tile.north_km
        );
        let mut holder = None;
        for (index, district) in DISTRICTS.iter().enumerate() {
            if directories[index].is_none() {
                directories[index] = Some(match directory(fetcher, cache, mode, model, district) {
                    Ok(listing) => {
                        if let Some(reason) = listing.kept_because {
                            unread.push((district, reason, true));
                        }
                        listing.members
                    }
                    Err(e) if of_the_archive(&e) => {
                        passed_over.get_or_insert(unread.len());
                        unread.push((district, e, false));
                        Vec::new()
                    }
                    Err(e) => return Err(e),
                });
            }
            let members = directories[index].as_deref().unwrap_or_default();
            if let Some(member) = find_member(members, |file| {
                file.starts_with(&prefix) && file.ends_with(".tif")
            }) {
                holder = Some((district, member.clone()));
                break;
            }
        }
        let Some((district, member)) = holder else {
            if tile_centre_in(tile, SAARLAND) {
                // It may be in the archive that could not be read: no map with a hole.
                if let Some(at) = passed_over {
                    return Err(unread.swap_remove(at).1);
                }
                missing.push(tile);
            }
            continue;
        };
        found += 1;
        // Named after the tile, never after the archive's member name (which comes from the
        // server and could hold `..` or `\`).
        let file = model.tile_cache_file(cache, tile);
        let url = model.archive_url(district);
        let bytes = cached_member(fetcher, mode, &url, &member, &file, || {
            offline_hint(&member.name)
        })?;
        let part = decode_float_tiff(&bytes).map_err(|message| refused(&url, message))?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(&url, e.to_string()))?;
    }
    if found == 0 {
        if let Some(at) = passed_over {
            return Err(unread.swap_remove(at).1);
        }
        return Err(match mode {
            FetchMode::Offline => offline_hint(model.label()),
            FetchMode::Online => refused(
                &folder,
                format!(
                    "no tile of the {} covers the map area; the archives may have been renamed",
                    model.label()
                ),
            ),
        });
    }
    write_float_raster(&raster, root, out, &folder)?;
    for (_, reason, kept) in unread {
        let model = crate::note::model(model.label());
        // The reason names the address of the archive.
        let error = Arg::error(reason);
        notes(if kept {
            note!(
                "warning-archive-directory-kept",
                model = model,
                error = error
            )
        } else {
            note!("warning-archive-passed-over", model = model, error = error)
        });
    }
    Ok(missing)
}

/// The provider id of the adapter.
const PROVIDER: &str = "lvgl";

/// The rasters of the LVGL (Saarland): requested for the area and stored in the project folder.
pub struct LvglBackend {
    info: ProviderInfo,
}

impl LvglBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: "lvgl-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for LvglBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LvglBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox)
    }
}

/// The LVGL rasters of `bbox` (EPSG:25832): the orthophoto from the WMS (2025), DGM1 and DOM1
/// (2025) read out of the district archives. The DOM1 is only a template: an unavailable one
/// costs a warning.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Result<(), AdapterError> {
    let (fetcher, mode, skip_surface) = (env.fetcher, env.mode, env.skip_surface);
    let cache = env.cache;
    let dir = &project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    warn_beyond_state(env, project, PROVIDER, "LVGL");
    let flight = || flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(WMS),
            "LVGL",
            &flight,
            // The date of the block's update, taken as the flight date: the record says so.
            Some(&FLIGHT_LABEL),
        ),
    )?;
    // Offline the tiles come from the cache: they were fetched when the record says.
    let record = |model: Model, old: &Option<WcsRecord>| WcsRecord {
        fetched: match (mode, old) {
            (FetchMode::Offline, Some(old)) => old.fetched,
            _ => env.today,
        },
        bbox: stored_bbox(bbox),
        date: Some(model.date().to_string()),
    };
    let warn_missing = |progress: &mut dyn FnMut(Note), model: Model, missing: Vec<_>| {
        if !missing.is_empty() {
            progress(note!(
                "warning-model-tiles-missing",
                model = crate::note::model(model.label()),
                count = missing.len(),
                names = crate::sources::lgv::names(&missing),
                state = "Saarland"
            ));
        }
    };
    (env.progress)(note!(
        "progress-model-read",
        model = crate::note::model(Model::Dgm1.label())
    ));
    let missing = fetch_model(
        fetcher,
        cache,
        mode,
        Model::Dgm1,
        bbox,
        dir,
        &dir.join(TERRAIN_RELPATH),
        env.progress,
    )?;
    warn_missing(env.progress, Model::Dgm1, missing);
    let old = project.records().terrain.clone();
    *project.records().terrain = Some(record(Model::Dgm1, &old));
    if skip_surface {
        keep_stored_surface(env, project, bbox);
        project.save()?;
        return Ok(());
    }
    (env.progress)(note!(
        "progress-model-read",
        model = crate::note::model(Model::Dom1.label())
    ));
    match fetch_model(
        fetcher,
        cache,
        mode,
        Model::Dom1,
        bbox,
        dir,
        &dir.join(SURFACE_RELPATH),
        env.progress,
    ) {
        Ok(missing) => {
            warn_missing(env.progress, Model::Dom1, missing);
            let old = project.records().surface.clone();
            *project.records().surface = Some(record(Model::Dom1, &old));
        }
        Err(e) => (env.progress)(note!(
            "warning-model-unavailable",
            model = crate::note::model(Model::Dom1.label()),
            error = Arg::error(e)
        )),
    }
    project.save()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, geotiff};
    use crate::fetch::{ByteRange, Cache};
    use crate::sources::zip_range::test_support::archive;

    /// A 1 km tile of the value `value`, as the LVGL cuts them.
    fn tile(e_km: u32, n_km: u32, value: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(e_km) * 1000.0,
            north: f64::from(n_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        geotiff(&grid, &vec![value; 1000 * 1000])
    }

    fn member(district: &str, e_km: u32, n_km: u32) -> String {
        format!("DGM1_tif_{district}/dgm1_32_{e_km}_{n_km}_1_SL_2025.tif")
    }

    /// The six archives of the terrain model, each with the tiles `tiles` gives for it.
    fn serve(fetcher: &MapFetcher, tiles: &[(&str, u32, u32, f32)]) {
        for district in DISTRICTS {
            let members: Vec<(String, Vec<u8>, bool)> = tiles
                .iter()
                .filter(|(holder, ..)| *holder == district)
                .map(|(_, e, n, value)| (member(district, *e, *n), tile(*e, *n, *value), true))
                .collect();
            let members: Vec<(&str, Vec<u8>, bool)> = members
                .iter()
                .map(|(name, bytes, deflated)| (name.as_str(), bytes.clone(), *deflated))
                .collect();
            fetcher.insert(&Model::Dgm1.archive_url(district), archive(&members));
        }
    }

    /// Saarbrücken: across the tile edge E 354 000.
    fn saarbruecken() -> BoundingBox {
        BoundingBox {
            min_e: 353_900.0,
            min_n: 5_455_500.0,
            max_e: 354_100.0,
            max_n: 5_455_600.0,
        }
    }

    fn requests_of(fetcher: &MapFetcher, district: &str) -> usize {
        let url = Model::Dgm1.archive_url(district);
        fetcher
            .log
            .borrow()
            .iter()
            .filter(|asked| **asked == url)
            .count()
    }

    #[test]
    fn the_archives_are_named_after_their_district() {
        assert_eq!(
            Model::Dgm1.archive_url("NK"),
            "https://www.shop.lvgl.saarland.de/cloud/public.php/dav/files/NK8ndP55qAqGEZD/\
             OD_DGM1_2025_tif_LK/DGM1_tif_NK_EPSG-25832_Entstehung-2025.zip"
        );
        assert_eq!(
            Model::Dom1.archive_url("SB"),
            "https://www.shop.lvgl.saarland.de/cloud/public.php/dav/files/NK8ndP55qAqGEZD/\
             OD_DOM1_2025_tif_LK/DOM1_tif_SB_EPSG-25832_Entstehung-2025.zip"
        );
        let cache = Cache::new(PathBuf::from("/cache"));
        let tile = TileId {
            east_km: 353,
            north_km: 5455,
        };
        assert_eq!(
            Model::Dgm1.tile_cache_file(&cache, tile),
            Path::new("/cache/lvgl/dgm1/dgm1-2025/dgm1_32_353_5455_.tif")
        );
        assert_eq!(
            Model::Dom1.tile_cache_file(&cache, tile),
            Path::new("/cache/lvgl/dom1/dom1-2025/dom1_32_353_5455_.tif")
        );
    }

    /// A tile is read from the first archive that holds it, once, and the archives after it
    /// are not asked: the tile at the district border is in NK and in SB. (The archives of the
    /// LVGL hold the same tile there; the two values here only show which archive was read.)
    #[test]
    fn a_tile_is_read_from_the_district_archive_that_holds_it() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(
            &fetcher,
            &[
                ("NK", 353, 5455, 6.0),
                ("NK", 354, 5455, 8.0),
                ("SB", 353, 5455, 60.0),
                ("SB", 354, 5455, 80.0),
            ],
        );
        let out = dir.path().join("dgm1.tif");
        let missing = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &out,
            &mut |_| {},
        )
        .unwrap();
        assert!(missing.is_empty());
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (200, 100));
        assert_eq!(raster.get(10, 50), 6.0);
        assert_eq!(raster.get(190, 50), 8.0);
        // MZG was looked through and holds nothing; NK served both tiles; SB and the rest
        // were never asked.
        assert!(requests_of(&fetcher, "MZG") > 0);
        assert!(requests_of(&fetcher, "NK") > 0);
        for district in ["SB", "SLS", "SPK", "WND"] {
            assert_eq!(requests_of(&fetcher, district), 0, "{district}");
        }
    }

    #[test]
    fn directories_and_tiles_come_from_the_cache_offline() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, &[("NK", 353, 5455, 6.0), ("NK", 354, 5455, 8.0)]);
        let out = dir.path().join("dgm1.tif");
        let fetch = |fetcher: &MapFetcher, mode| {
            fetch_model(
                fetcher,
                &cache,
                mode,
                Model::Dgm1,
                &saarbruecken(),
                dir.path(),
                &out,
                &mut |_| {},
            )
        };
        fetch(&fetcher, FetchMode::Online).unwrap();
        let online = std::fs::read(&out).unwrap();
        let offline = MapFetcher::default();
        fetch(&offline, FetchMode::Offline).unwrap();
        assert_eq!(offline.calls.get(), 0);
        assert_eq!(std::fs::read(&out).unwrap(), online);
        // Offline with nothing in the cache says to fetch online.
        let empty = Cache::new(dir.path().join("empty"));
        let refused = fetch_model(
            &offline,
            &empty,
            FetchMode::Offline,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &dir.path().join("none.tif"),
            &mut |_| {},
        )
        .unwrap_err()
        .to_string();
        assert!(refused.contains("offline and not cached"), "{refused}");
    }

    /// The six archives without those of `gone`, which answer "not found".
    fn serve_without(fetcher: &MapFetcher, tiles: &[(&str, u32, u32, f32)], gone: &[&str]) {
        serve(fetcher, tiles);
        for district in gone {
            fetcher
                .responses
                .borrow_mut()
                .remove(&Model::Dgm1.archive_url(district));
        }
    }

    /// An archive that cannot be read (renamed, or its server is down) does not stop the tiles
    /// another archive holds: MZG is looked through first for every tile and is gone here.
    #[test]
    fn an_archive_that_is_gone_does_not_stop_the_tiles_another_holds() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_without(
            &fetcher,
            &[("NK", 353, 5455, 6.0), ("NK", 354, 5455, 8.0)],
            &["MZG"],
        );
        let out = dir.path().join("dgm1.tif");
        let missing = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &out,
            &mut |_| {},
        )
        .unwrap();
        assert!(missing.is_empty());
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.get(10, 50), 6.0);
        assert_eq!(raster.get(190, 50), 8.0);
    }

    /// A tile inside Saarland that no readable archive holds, while one archive could not be
    /// read: the tile may be in that one, so the fetch fails and names it. It does not go on
    /// with a hole, and it does not say the archives were renamed.
    #[test]
    fn a_tile_that_may_be_in_the_archive_that_is_gone_fails_the_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_without(&fetcher, &[("NK", 353, 5455, 6.0)], &["SB"]);
        let out = dir.path().join("dgm1.tif");
        let refused = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &out,
            &mut |_| {},
        )
        .unwrap_err()
        .to_string();
        assert!(refused.contains("DGM1_tif_SB_"), "{refused}");
        assert!(!refused.contains("renamed"), "{refused}");
        assert!(!out.exists());
    }

    /// Tiles in the cache survive an archive that is down: its directory is read from the
    /// cache, and the tiles match it, so nothing more is asked of that archive.
    #[test]
    fn cached_tiles_survive_an_archive_that_is_down() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let tiles = [("NK", 353, 5455, 6.0), ("NK", 354, 5455, 8.0)];
        let out = dir.path().join("dgm1.tif");
        let fetch = |fetcher: &MapFetcher| {
            fetch_model(
                fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &saarbruecken(),
                dir.path(),
                &out,
                &mut |_| {},
            )
        };
        let up = MapFetcher::default();
        serve(&up, &tiles);
        fetch(&up).unwrap();
        let first = std::fs::read(&out).unwrap();
        let down = MapFetcher::default();
        serve_without(&down, &tiles, &["NK"]);
        assert!(fetch(&down).unwrap().is_empty());
        assert_eq!(std::fs::read(&out).unwrap(), first);
    }

    /// The person is told about an archive that was passed over, and about a directory of an
    /// earlier run that stood in for an archive that is down: the tiles of the first may be
    /// newer there, and the second is not checked against the archive.
    #[test]
    fn a_passed_over_archive_and_a_kept_directory_are_told() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let tiles = [("NK", 353, 5455, 6.0), ("NK", 354, 5455, 8.0)];
        let out = dir.path().join("dgm1.tif");
        let fetch = |fetcher: &MapFetcher| {
            let mut notes = Vec::new();
            fetch_model(
                fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &saarbruecken(),
                dir.path(),
                &out,
                &mut |note| notes.push(note.to_string()),
            )
            .unwrap();
            notes
        };
        // MZG is gone and was never read: passed over.
        let without_mzg = MapFetcher::default();
        serve_without(&without_mzg, &tiles, &["MZG"]);
        let url = Model::Dgm1.archive_url("MZG");
        assert_eq!(
            fetch(&without_mzg),
            [format!(
                "warning: terrain model (LVGL DGM1 2025): an archive could not be read ({url} \
                 not found on the server); it was passed over, and the tiles of the map area \
                 come from the other archives"
            )]
        );
        // Every archive answers: nothing to tell.
        let up = MapFetcher::default();
        serve(&up, &tiles);
        assert_eq!(fetch(&up), Vec::<String>::new());
        // NK is down now, and its directory of the run before stands in.
        let without_nk = MapFetcher::default();
        serve_without(&without_nk, &tiles, &["NK"]);
        let url = Model::Dgm1.archive_url("NK");
        assert_eq!(
            fetch(&without_nk),
            [format!(
                "warning: terrain model (LVGL DGM1 2025): an archive could not be read ({url} \
                 not found on the server); its directory kept from an earlier run is used, so a \
                 tile the LVGL replaced since is not noticed"
            )]
        );
    }

    /// A directory that cannot be stored is an error of this computer (a full disk, a folder
    /// without write access). It fails the fetch, also when an older directory is there: it
    /// is never taken for an archive that is down.
    #[cfg(unix)]
    #[test]
    fn a_directory_that_cannot_be_stored_fails_the_fetch() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, &[("NK", 353, 5455, 6.0), ("NK", 354, 5455, 8.0)]);
        let out = dir.path().join("dgm1.tif");
        let fetch = |notes: &mut Vec<String>| {
            fetch_model(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &saarbruecken(),
                dir.path(),
                &out,
                &mut |note| notes.push(note.to_string()),
            )
        };
        fetch(&mut Vec::new()).unwrap();
        let folder = Model::Dgm1.cache_folder(&cache);
        let writable = std::fs::metadata(&folder).unwrap().permissions();
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
        let mut notes = Vec::new();
        let refused = fetch(&mut notes);
        std::fs::set_permissions(&folder, writable).unwrap();
        assert!(matches!(refused, Err(FetchError::Io(..))), "{refused:?}");
        assert_eq!(notes, Vec::<String>::new());
    }

    /// Only what an archive or its server answers lets the others go on. A cancel, an address
    /// freemapper does not ask and an error of this computer stop the fetch at once.
    #[test]
    fn only_an_answer_of_the_archive_lets_the_others_go_on() {
        let url = || "https://www.shop.lvgl.saarland.de/x.zip".to_string();
        assert!(of_the_archive(&FetchError::NotFound(url())));
        assert!(of_the_archive(&refused(
            &url(),
            "no end of central directory"
        )));
        assert!(!of_the_archive(&FetchError::Cancelled));
        assert!(!of_the_archive(&FetchError::NotAllowed {
            url: url(),
            reason: "not https".to_string()
        }));
        assert!(!of_the_archive(&FetchError::Io(
            PathBuf::from("directory-MZG.bin"),
            std::io::Error::other("disk full")
        )));
    }

    /// The share was renamed or holds another survey: the error names the folder and nothing
    /// is written.
    #[test]
    fn no_tile_at_all_names_the_archive_folder() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, &[("WND", 370, 5480, 1.0)]);
        let out = dir.path().join("dgm1.tif");
        let refused = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &out,
            &mut |_| {},
        )
        .unwrap_err()
        .to_string();
        assert!(refused.contains("OD_DGM1_2025_tif_LK"), "{refused}");
        assert!(refused.contains("may have been renamed"), "{refused}");
        assert!(!out.exists());
    }

    /// Across the border to France at Saarbrücken: the tile whose centre lies in France is in
    /// no archive and is no finding; a tile inside Saarland that no archive holds is one.
    #[test]
    fn tiles_outside_saarland_stay_empty_without_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, &[("SB", 354, 5451, 5.0)]);
        // Along N 5 451 000 to 5 452 000: the tile E 353 lies in France (Spicheren), E 354
        // and E 355 in Saarbrücken.
        let bbox = BoundingBox {
            min_e: 353_100.0,
            min_n: 5_451_400.0,
            max_e: 355_900.0,
            max_n: 5_451_600.0,
        };
        let france = TileId {
            east_km: 353,
            north_km: 5451,
        };
        let inside = TileId {
            east_km: 355,
            north_km: 5451,
        };
        assert!(!tile_centre_in(france, SAARLAND));
        assert!(tile_centre_in(inside, SAARLAND));
        let missing = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("dgm1.tif"),
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(missing, [inside]);
    }

    /// A server that answers a range with the whole archive: the fetch stops there.
    #[test]
    fn a_range_answered_with_the_whole_file_is_refused() {
        struct WholeFile;
        impl Fetcher for WholeFile {
            fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
                Err(FetchError::NotFound(url.to_string()))
            }

            fn get_range(&self, url: &str, _: ByteRange) -> Result<Vec<u8>, FetchError> {
                Err(FetchError::RangeIgnored {
                    url: url.to_string(),
                    range: "bytes=-65558".to_string(),
                    status: 200,
                })
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let refused = fetch_model(
            &WholeFile,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &saarbruecken(),
            dir.path(),
            &dir.path().join("dgm1.tif"),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(refused, FetchError::RangeIgnored { .. }),
            "{refused:?}"
        );
    }

    /// The date of the orthophoto is the `Aktualisierung` row of the feature info, as recorded
    /// on 2026-11-12; an answer without it gives no date.
    #[test]
    fn the_flight_date_is_the_update_row_of_the_feature_info() {
        let recorded = "<html><body><table border=\"1\"> <!-- <tr><td>Nr</td><td>[designatio]</td></tr> --> \
            <tr><td>Name</td><td>dop20rgbi_32_348_5454_2_sl_2025</td></tr> <tr><td>Titel</td><td>Gersweiler</td></tr> \
            <tr><td>Erstellung</td><td>n/a</td></tr> <tr><td>Aktualisierung</td><td>10.05.2025</td></tr> \
            <tr><td>Publikation</td><td>n/a</td></tr></table></body></html>";
        assert_eq!(update_date(recorded).as_deref(), Some("10.05.2025"));
        assert_eq!(update_date(&recorded.replace("10.05.2025", "n/a")), None);
        assert_eq!(
            update_date("GetFeatureInfo results:\n\nLayer 'sl_dop_info'\n"),
            None
        );
        assert_eq!(update_date(""), None);
        // The request asks the metadata layer at the centre of the area, as HTML.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(&WMS.url, recorded.as_bytes().to_vec());
        assert_eq!(
            flight_date(&fetcher, &saarbruecken()).as_deref(),
            Some("10.05.2025")
        );
        assert_eq!(
            *fetcher.log.borrow(),
            [
                "https://geoportal.saarland.de/freewms/dop2025?SERVICE=WMS&VERSION=1.3.0\
              &REQUEST=GetFeatureInfo&LAYERS=sl_dop_info&QUERY_LAYERS=sl_dop_info&STYLES=\
              &CRS=EPSG:25832&BBOX=353950,5455500,354050,5455600&WIDTH=101&HEIGHT=101&I=50&J=50\
              &INFO_FORMAT=text/html"
            ]
        );
    }

    #[test]
    fn the_backend_names_its_provider() {
        let backend = LvglBackend::new();
        let info = backend.provider();
        assert_eq!(info.id.as_str(), "lvgl");
        assert_eq!(info.cache_dir.as_deref(), Some("lvgl"));
        assert_eq!(info.terrain_source, "lvgl-dgm1");
        assert_eq!(info.kind, RasterKind::PerArea);
    }
}
