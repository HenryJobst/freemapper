//! The laser points of the LVGL Saarland ("3D-Messdaten (2025)", dl-de/by-2-0, "© GeoBasis
//! DE/LVGL-SL"): one LAZ file per km tile in EPSG:25832, thinned to at most one point per
//! 0.5 m cell, packed into six zip64 archives of 12 to 26 GB by district on the open-data share
//! that holds the height models too (ADR 0105). A tile is read out of an archive by byte ranges
//! (the central directory, then the member), unpacked, checked against its CRC and kept in the
//! cache. The survey is in the names of the archives (`Entstehung-2025`) and of the cache
//! folder, so a cached tile serves without asking; a new survey comes with new names. The
//! classes: ground (2), buildings (6), water (9), other non-ground, mostly vegetation (20), and
//! synthetic fill points (8, 30).

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::NaiveDate;
use freemapper_adapter_api::cache::{CacheDir, STALE_PART_AGE};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::pointcloud::{
    PointCloudBackend, PointCloudInfo, PointCloudRecord, PointCloudTile,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode};
use crate::las_files::{
    cached_las, check_las_file, fallback, las_path, not_published,
    parse_utm32_tile_name as parse_tile_name, utm32_tile_name as tile_name,
};
use crate::lvgl::{DISTRICTS, SHARE};
use crate::note::note;
use crate::sources::zip_range::{Member, find_member, read_members, stream_member};
use crate::tiles::{TileId, tiles_covering_grid};

/// The folder of the share that holds the archives.
const FOLDER: &str = "OD_LIDAR_Punktwolke_2025_laz_LK";
/// The folder of the point clouds below the folder of the LVGL in the cache, by survey.
const DIR: &str = "las-2025";
/// A tile has 26 to 46 MB, 39 MB in the median (2026); the estimate counts this much per tile.
pub const MB_PER_TILE: f64 = 40.0;

/// URL of the folder of the archives.
pub fn folder_url() -> String {
    format!("{SHARE}/{FOLDER}")
}

/// URL of the archive of `district` (one of [`DISTRICTS`]).
pub fn archive_url(district: &str) -> String {
    format!(
        "{}/LIDAR_laz_{district}_EPSG-25832_Entstehung-2025.zip",
        folder_url()
    )
}

/// The name of the LAZ file of `tile` in an archive, lower-cased as [`find_member`] compares.
fn member_name(tile: TileId) -> String {
    format!(
        "3dm_32_{}_{}_1_sl_2025_050.laz",
        tile.east_km, tile.north_km
    )
}

/// The point-cloud backend of Saarland.
pub fn backend() -> &'static dyn PointCloudBackend {
    static SAARLAND: LazyLock<LvglLasBackend> = LazyLock::new(LvglLasBackend::new);
    &*SAARLAND
}

/// The laser points of the LVGL, kept in the cache as LAZ.
pub struct LvglLasBackend {
    info: PointCloudInfo,
}

impl LvglLasBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in("lvgl"),
                cache_dir: "lvgl".to_string(),
                source: "lvgl-las".to_string(),
                epsg: 25832,
                grid_km: 1,
                left_out: Vec::new(),
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(DIR)
    }
}

impl Default for LvglLasBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// What a fetch learnt about one archive, so each is asked once per fetch: its members, or why
/// they could not be read, and the day of its last change once a tile came out of it.
#[derive(Default)]
struct Archive {
    members: Option<Result<Vec<Member>, String>>,
    changed: Option<Result<NaiveDate, String>>,
}

/// The error an [`Archive`] remembered, again as an error of `url`.
fn again(url: &str, message: &str) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message: message.to_string(),
    }
}

impl PointCloudBackend for LvglLasBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let mut archives: Vec<Archive> = DISTRICTS.iter().map(|_| Archive::default()).collect();
        let mut fetched = Vec::new();
        for tile in tiles_covering_grid(bbox, self.info.grid_km) {
            let name = tile_name(tile);
            let cached = cached_las(&dir, &name);
            // A tile on a district border is in two archives, the same in each: the record
            // names the folder of the archives.
            let found = |(published, path): (NaiveDate, PathBuf)| PointCloudTile {
                record: PointCloudRecord {
                    tile: name.clone(),
                    url: folder_url(),
                    published: Some(published),
                },
                path,
            };
            // The survey is in the folder's name: a cached tile is the tile.
            if let Some(newest) = cached.first().cloned() {
                if env.mode == FetchMode::Online {
                    (env.progress)(note!("progress-point-cloud-cached", tile = &name));
                }
                fetched.push(found(newest));
                continue;
            }
            if env.mode == FetchMode::Offline {
                (env.progress)(note!("warning-point-cloud-no-cached", tile = &name));
                continue;
            }
            // The first archive that holds the tile; an archive that cannot be read is passed
            // over, and is the reason only when no other one holds the tile.
            let wanted = member_name(tile);
            let mut holder = None;
            let mut unread = None;
            for (index, district) in DISTRICTS.iter().enumerate() {
                let url = archive_url(district);
                let known = &mut archives[index];
                if known.members.is_none() {
                    known.members = Some(match read_members(env.fetcher, &url) {
                        Ok(members) => Ok(members),
                        Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                        Err(e) => Err(e.to_string()),
                    });
                }
                match &known.members {
                    Some(Ok(members)) => {
                        if let Some(member) = find_member(members, |file| file == wanted) {
                            holder = Some((index, member.clone()));
                            break;
                        }
                    }
                    Some(Err(message)) => {
                        unread.get_or_insert_with(|| again(&url, message));
                    }
                    None => unreachable!("asked above"),
                }
            }
            let Some((index, member)) = holder else {
                match unread {
                    Some(error) => fallback(env, &name, cached, error, &mut fetched, found),
                    None => not_published(env, &name, cached, &mut fetched, found),
                }
                continue;
            };
            let url = archive_url(DISTRICTS[index]);
            let known = &mut archives[index];
            let changed = match &known.changed {
                Some(Ok(changed)) => Ok(*changed),
                Some(Err(message)) => Err(again(&url, message)),
                None => env.fetcher.last_modified(&url).and_then(|changed| {
                    changed.ok_or_else(|| again(&url, "the server names no date of the archive"))
                }),
            };
            let changed = match changed {
                Ok(changed) => changed,
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Err(e) => {
                    known.changed = Some(Err(e.to_string()));
                    fallback(env, &name, cached, e, &mut fetched, found);
                    continue;
                }
            };
            known.changed = Some(Ok(changed));
            let path = las_path(&dir, &name, changed);
            (env.progress)(note!("progress-downloading", url = &url));
            match download_tile(env, &url, &member, &path) {
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Ok(()) => fetched.push(found((changed, path))),
                Err(e) => fallback(env, &name, cached, e, &mut fetched, found),
            }
        }
        freemapper_fs::remove_stale_cache_parts(&dir, STALE_PART_AGE);
        Ok(fetched)
    }

    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        let tile = parse_tile_name(&record.tile)?;
        let path = las_path(&self.dir(cache), &tile_name(tile), record.published?);
        path.exists().then_some(path)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let dir = self.dir(env.cache);
        tiles_covering_grid(&area.map_bbox, self.info.grid_km)
            .into_iter()
            .map(|tile| EstimateItem {
                what: format!("LVGL point cloud {}", tile_name(tile)),
                megabytes: MB_PER_TILE,
                cached: !cached_las(&dir, &tile_name(tile)).is_empty(),
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

/// Reads `member` out of the archive at `url` next to `path`, checks that it is a complete
/// point cloud and moves it to `path`. No partial file is left behind.
fn download_tile(
    env: &mut AdapterEnv<'_>,
    url: &str,
    member: &Member,
    path: &Path,
) -> Result<(), FetchError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    }
    let part = crate::las_files::part_path(path);
    let result = stream_member(env.fetcher, url, member, &part, env.progress)
        .and_then(|()| check_las_file(url, &part))
        .and_then(|()| std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.into(), e)));
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::registry::Registry;

    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::zip_range::test_support::archive;

    /// The tile of the old town of Saarlouis.
    const SAARLOUIS: TileId = TileId {
        east_km: 337,
        north_km: 5464,
    };
    /// A box inside the tile.
    const BBOX: BoundingBox = BoundingBox {
        min_e: 337_300.0,
        min_n: 5_464_300.0,
        max_e: 337_400.0,
        max_n: 5_464_400.0,
    };

    fn registry() -> Registry {
        let mut registry = Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        registry
    }

    /// A small LAZ point cloud of `points` points like the members of the archives (LAS 1.2,
    /// point format 1, no CRS).
    fn laz(points: u32) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 2));
        builder.point_format = las::point::Format::new(1).unwrap();
        builder.point_format.is_compressed = true;
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(0.0),
            y: transform(5_000_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..points {
            writer
                .write_point(las::Point {
                    x: 337_350.0,
                    y: 5_464_350.0 + f64::from(i) * 0.5,
                    z: 180.0,
                    return_number: 1,
                    number_of_returns: 1,
                    classification: las::point::Classification::Ground,
                    gps_time: Some(0.0),
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }

    /// The member path of `tile` in the archive of `district`, spelt as the state spells it.
    fn member_path(district: &str, tile: TileId) -> String {
        format!(
            "LIDAR_laz_{district}/3dm_32_{}_{}_1_SL_2025_050.laz",
            tile.east_km, tile.north_km
        )
    }

    /// Serves the archive of `district` with `tiles` (tile, points), changed on `changed`.
    fn serve(fetcher: &MapFetcher, district: &str, tiles: &[(TileId, u32)], changed: &str) {
        let members: Vec<(String, Vec<u8>, bool)> = tiles
            .iter()
            .map(|(tile, points)| (member_path(district, *tile), laz(*points), true))
            .collect();
        let members: Vec<(&str, Vec<u8>, bool)> = members
            .iter()
            .map(|(name, bytes, deflate)| (name.as_str(), bytes.clone(), *deflate))
            .collect();
        let url = archive_url(district);
        fetcher.insert(&url, archive(&members));
        fetcher.set_modified(&url, changed.parse().unwrap());
    }

    /// Serves all six archives empty, the way a map outside them sees them.
    fn serve_empty(fetcher: &MapFetcher) {
        let other = TileId {
            east_km: 360,
            north_km: 5480,
        };
        for district in DISTRICTS {
            serve(fetcher, district, &[(other, 1)], "2026-01-16");
        }
    }

    fn fetch(
        fetcher: &dyn crate::fetch::Fetcher,
        cache: &CacheDir,
        mode: FetchMode,
    ) -> Result<(Vec<PointCloudTile>, Vec<String>), AdapterError> {
        let registry = registry();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache,
            mode,
            today: "2026-11-12".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        let tiles = LvglLasBackend::new().fetch(&mut env, &BBOX)?;
        Ok((tiles, notes))
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("lvgl/las-2025"))
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn the_archives_and_members_are_named_as_on_the_share() {
        assert_eq!(
            archive_url("SLS"),
            "https://www.shop.lvgl.saarland.de/cloud/public.php/dav/files/NK8ndP55qAqGEZD/\
             OD_LIDAR_Punktwolke_2025_laz_LK/LIDAR_laz_SLS_EPSG-25832_Entstehung-2025.zip"
        );
        assert_eq!(member_name(SAARLOUIS), "3dm_32_337_5464_1_sl_2025_050.laz");
    }

    #[test]
    fn a_tile_is_read_out_of_the_first_archive_that_holds_it_and_kept_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_empty(&fetcher);
        // On a district border: in the archives of Saarlouis and Saarbrücken.
        serve(&fetcher, "SB", &[(SAARLOUIS, 30)], "2026-01-16");
        serve(&fetcher, "SLS", &[(SAARLOUIS, 30)], "2026-01-17");
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert_eq!(tiles.len(), 1, "{notes:?}");
        assert_eq!(
            tiles[0].record,
            PointCloudRecord {
                tile: "32337-5464".to_string(),
                url: folder_url(),
                published: Some("2026-01-16".parse().unwrap()),
            }
        );
        assert_eq!(folder(&cache), ["32337-5464_2026-01-16.laz"]);
        let mut reader = las::Reader::from_path(&tiles[0].path).unwrap();
        assert_eq!(reader.read_points(100).unwrap().len(), 30);
        // The archives after the one that held it were not asked.
        for district in ["SLS", "SPK", "WND"] {
            assert!(
                !fetcher
                    .log
                    .borrow()
                    .iter()
                    .any(|u| u == &archive_url(district)),
                "{district}"
            );
        }
        assert_eq!(
            LvglLasBackend::new().cached(&cache, &tiles[0].record),
            Some(tiles[0].path.clone())
        );
        // The second fetch asks nothing: the survey is in the cache folder's name.
        let silent = MapFetcher::default();
        let (again, notes) = fetch(&silent, &cache, FetchMode::Online).unwrap();
        assert_eq!(again, tiles);
        assert_eq!(silent.calls.get(), 0);
        assert!(
            notes.iter().any(|n| n == "cached point cloud 32337-5464"),
            "{notes:?}"
        );
    }

    #[test]
    fn an_archive_that_cannot_be_read_is_passed_over() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_empty(&fetcher);
        serve(&fetcher, "SLS", &[(SAARLOUIS, 10)], "2026-01-16");
        fetcher.failing.borrow_mut().push(archive_url("MZG"));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert_eq!(tiles.len(), 1, "{notes:?}");
        // When the tile is in no archive that can be read, the reason is the failing one.
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_empty(&fetcher);
        fetcher.failing.borrow_mut().push(archive_url("NK"));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(
            notes.iter().any(
                |n| n.starts_with("warning: point cloud 32337-5464 unavailable")
                    && n.contains("LIDAR_laz_NK_")
            ),
            "{notes:?}"
        );
    }

    #[test]
    fn a_tile_in_no_archive_is_not_published_and_offline_takes_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_empty(&fetcher);
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n == "no laser point cloud published for tile 32337-5464"),
            "{notes:?}"
        );
        let silent = MapFetcher::default();
        let (offline, notes) = fetch(&silent, &cache, FetchMode::Offline).unwrap();
        assert!(offline.is_empty());
        assert!(notes.iter().any(|n| n.contains("no cached point cloud")));
        serve(&fetcher, "SLS", &[(SAARLOUIS, 10)], "2026-01-16");
        fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        let (offline, _) = fetch(&silent, &cache, FetchMode::Offline).unwrap();
        assert_eq!(offline.len(), 1);
        assert_eq!(silent.calls.get(), 0, "offline asks nothing");
    }

    #[test]
    fn a_damaged_member_or_an_archive_without_a_date_keeps_nothing_and_a_cancel_stops() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_empty(&fetcher);
        let mut zip = archive(&[(&member_path("MZG", SAARLOUIS), laz(20), false)]);
        let at = zip.windows(4).position(|w| w == b"LASF").unwrap();
        zip[at + 200] ^= 0xff;
        fetcher.insert(&archive_url("MZG"), zip);
        fetcher.set_modified(&archive_url("MZG"), "2026-01-16".parse().unwrap());
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(notes.iter().any(|n| n.contains("CRC")), "{notes:?}");
        assert!(folder(&cache).is_empty());
        // No date of the archive: nothing to name the file by.
        let undated = MapFetcher::default();
        for district in DISTRICTS {
            undated.insert(
                &archive_url(district),
                archive(&[(&member_path(district, SAARLOUIS), laz(5), true)]),
            );
        }
        let (tiles, notes) = fetch(&undated, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(notes.iter().any(|n| n.contains("no date")), "{notes:?}");
        struct Stopped;
        impl crate::fetch::Fetcher for Stopped {
            fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
                Err(FetchError::Cancelled)
            }
            fn last_modified(&self, _url: &str) -> Result<Option<NaiveDate>, FetchError> {
                Err(FetchError::Cancelled)
            }
        }
        assert!(fetch(&Stopped, &cache, FetchMode::Online).is_err());
    }

    #[test]
    fn the_estimate_counts_every_tile_of_the_map_box() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let area = EstimateArea {
            bbox_25833: BBOX,
            map_bbox: BoundingBox {
                max_e: 338_100.0,
                ..BBOX
            },
            today: "2026-11-12".parse().unwrap(),
        };
        let items = LvglLasBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let what: Vec<(&str, f64)> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes))
            .collect();
        assert_eq!(
            what,
            [
                ("LVGL point cloud 32337-5464", MB_PER_TILE),
                ("LVGL point cloud 32338-5464", MB_PER_TILE),
            ]
        );
    }

    #[test]
    #[ignore = "network"]
    fn live_the_archive_of_saarlouis_holds_its_tile() {
        use crate::fetch::HttpFetcher;
        let members = read_members(&HttpFetcher::new(), &archive_url("SLS")).unwrap();
        let wanted = member_name(SAARLOUIS);
        assert!(find_member(&members, |file| file == wanted).is_some());
    }
}
