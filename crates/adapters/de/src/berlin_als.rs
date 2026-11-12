//! The laser points of Berlin (Geoportal Berlin, "Airborne Laserscanning (ALS) Primäre 3D
//! Laserscan-Daten", dl-de/zero-2-0, scan of 2021): one LAS file per km tile in EPSG:25833,
//! packed into nine zip64 archives of 1 to 51 GB by part of the city (ADR 0105). A tile is read
//! out of its archive by byte ranges (the central directory, then the member), unpacked,
//! checked against its CRC and kept in the cache as LAZ. The archive's last change names the
//! cached file, so a new scan replaces it. The classes: ground (2), low, medium and high
//! vegetation (3, 4, 5), in which buildings lie too, and low points (7).

use std::collections::HashMap;
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
use crate::las_files::{cached_las, check_las_file, compress_las, fallback, las_path};
use crate::note::note;
use crate::sources::zip_range::{Member, find_member, read_members, stream_member};
use crate::tiles::{TileId, tiles_covering_grid};

const BASE: &str = "https://gdi.berlin.de/data/a_als/atom";
/// The folder of the point clouds below the Berlin folder of the cache.
const DIR: &str = "als";
/// A tile is about 240 MB to download (packed LAS) and 100 MB as LAZ in the cache (2021 scan);
/// the estimate counts the download.
pub const MB_PER_TILE: f64 = 240.0;

/// The archive that holds `tile`: three columns (west of km 384, to km 399, east of it) times
/// three rows (south of km 5816, to km 5825, north of it).
fn archive_of(tile: TileId) -> &'static str {
    let column = match tile.east_km {
        ..384 => 0,
        384..400 => 1,
        _ => 2,
    };
    let row = match tile.north_km {
        ..5816 => 0,
        5816..5826 => 1,
        _ => 2,
    };
    [
        ["Suedwest", "West", "Nordwest"],
        ["Sued", "Mitte", "Nord"],
        ["Suedost", "Ost", "Nordost"],
    ][column][row]
}

/// URL of the archive `name`.
pub fn archive_url(name: &str) -> String {
    format!("{BASE}/{name}.zip")
}

/// The name of the LAS file of `tile` in its archive.
fn member_name(tile: TileId) -> String {
    format!("3dm_33_{}_{}_1_be.las", tile.east_km, tile.north_km)
}

/// The point-cloud backend of Berlin.
pub fn backend() -> &'static dyn PointCloudBackend {
    static BERLIN: LazyLock<BerlinAlsBackend> = LazyLock::new(BerlinAlsBackend::new);
    &*BERLIN
}

/// The laser points of Berlin, kept in the cache as LAZ.
pub struct BerlinAlsBackend {
    info: PointCloudInfo,
}

impl BerlinAlsBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in("berlin"),
                cache_dir: "berlin".to_string(),
                source: "berlin-als".to_string(),
                epsg: 25833,
                grid_km: 1,
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(DIR)
    }
}

impl Default for BerlinAlsBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// What a fetch learnt about one archive: the day of its last change and, once a tile had to be
/// read, its members.
#[derive(Default)]
struct Archive {
    changed: Option<NaiveDate>,
    members: Option<Vec<Member>>,
}

impl PointCloudBackend for BerlinAlsBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let offline = env.mode == FetchMode::Offline;
        let mut archives: HashMap<&str, Archive> = HashMap::new();
        let mut fetched = Vec::new();
        for tile in tiles_covering_grid(bbox, self.info.grid_km) {
            let name = tile.name();
            let archive = archive_of(tile);
            let url = archive_url(archive);
            let cached = cached_las(&dir, &name);
            let found = |(published, path): (NaiveDate, PathBuf)| PointCloudTile {
                record: PointCloudRecord {
                    tile: name.clone(),
                    url: url.clone(),
                    published: Some(published),
                },
                path,
            };
            if offline {
                match cached.into_iter().next() {
                    Some(newest) => fetched.push(found(newest)),
                    None => (env.progress)(note!("warning-point-cloud-no-cached", tile = &name)),
                }
                continue;
            }
            let known = archives.entry(archive).or_default();
            let changed = match known.changed {
                Some(changed) => Ok(changed),
                None => env.fetcher.last_modified(&url).and_then(|changed| {
                    changed.ok_or_else(|| FetchError::Http {
                        url: url.clone(),
                        message: "the server names no date of the archive".to_string(),
                    })
                }),
            };
            let changed = match changed {
                Ok(changed) => changed,
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Err(e) => {
                    fallback(env, &name, cached, e, &mut fetched, found);
                    continue;
                }
            };
            known.changed = Some(changed);
            let path = las_path(&dir, &name, changed);
            if path.exists() {
                (env.progress)(note!("progress-point-cloud-cached", tile = &name));
                fetched.push(found((changed, path)));
                continue;
            }
            if known.members.is_none() {
                match read_members(env.fetcher, &url) {
                    Ok(members) => known.members = Some(members),
                    Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                    Err(e) => {
                        fallback(env, &name, cached, e, &mut fetched, found);
                        continue;
                    }
                }
            }
            let wanted = member_name(tile);
            let Some(member) = known
                .members
                .as_deref()
                .and_then(|members| find_member(members, |file| file == wanted))
                .cloned()
            else {
                (env.progress)(note!("progress-point-cloud-not-published", tile = &name));
                continue;
            };
            (env.progress)(note!("progress-downloading", url = &url));
            match download_tile(env, &url, &member, &path) {
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Ok(()) => {
                    for (_, older) in cached.iter().filter(|(_, p)| *p != path) {
                        let _ = std::fs::remove_file(older);
                    }
                    fetched.push(found((changed, path)));
                }
                Err(e) => fallback(env, &name, cached, e, &mut fetched, found),
            }
        }
        freemapper_fs::remove_stale_cache_parts(&dir, STALE_PART_AGE);
        Ok(fetched)
    }

    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        let tile = TileId::parse(&record.tile)?;
        let path = las_path(&self.dir(cache), &tile.name(), record.published?);
        path.exists().then_some(path)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let dir = self.dir(env.cache);
        tiles_covering_grid(&area.map_bbox, self.info.grid_km)
            .into_iter()
            .map(|tile| EstimateItem {
                what: format!("Geoportal Berlin point cloud {}", tile.name()),
                megabytes: MB_PER_TILE,
                cached: !cached_las(&dir, &tile.name()).is_empty(),
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

/// Reads `member` out of the archive at `url` as LAS next to `path`, compresses it to LAZ,
/// checks that and moves it to `path`. No partial file is left behind.
fn download_tile(
    env: &mut AdapterEnv<'_>,
    url: &str,
    member: &Member,
    path: &Path,
) -> Result<(), FetchError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    }
    let las = path.with_extension("las.part");
    let laz = path.with_extension("laz.part");
    let result = stream_member(env.fetcher, url, member, &las, env.progress)
        .and_then(|()| compress_las(url, &las, &laz))
        .and_then(|()| check_las_file(url, &laz))
        .and_then(|()| std::fs::rename(&laz, path).map_err(|e| FetchError::Io(path.into(), e)));
    let _ = std::fs::remove_file(&las);
    if result.is_err() {
        let _ = std::fs::remove_file(&laz);
    }
    result
}

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::registry::Registry;

    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::zip_range::test_support::archive;

    const VIKTORIAPARK: TileId = TileId {
        east_km: 391,
        north_km: 5818,
    };
    /// A box inside the tile of the Viktoriapark.
    const BBOX: BoundingBox = BoundingBox {
        min_e: 391_300.0,
        min_n: 5_818_300.0,
        max_e: 391_400.0,
        max_n: 5_818_400.0,
    };

    fn registry() -> Registry {
        let mut registry = Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        registry
    }

    /// A small plain LAS point cloud of `points` points, like the members of the archives.
    fn las(points: u32) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.las");
        let mut builder = las::Builder::from((1, 4));
        builder.point_format = las::point::Format::new(6).unwrap();
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(0.0),
            y: transform(0.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..points {
            writer
                .write_point(las::Point {
                    x: 391_350.0,
                    y: 5_818_350.0 + f64::from(i) * 0.2,
                    z: 60.0,
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

    /// The archive of the Viktoriapark with the tile `points` points long and a neighbour,
    /// changed on `changed`.
    fn serve(fetcher: &MapFetcher, points: u32, changed: &str) {
        let url = archive_url("Mitte");
        fetcher.insert(
            &url,
            archive(&[
                ("3dm_33_391_5817_1_be.las", las(3), true),
                (&member_name(VIKTORIAPARK), las(points), true),
            ]),
        );
        fetcher.set_modified(&url, changed.parse().unwrap());
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
        let tiles = BerlinAlsBackend::new().fetch(&mut env, &BBOX)?;
        Ok((tiles, notes))
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("berlin/als"))
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn every_tile_has_its_archive() {
        for (east_km, north_km, archive) in [
            (369, 5805, "Suedwest"),
            (383, 5825, "West"),
            (372, 5836, "Nordwest"),
            (384, 5803, "Sued"),
            (391, 5818, "Mitte"),
            (399, 5837, "Nord"),
            (416, 5799, "Suedost"),
            (409, 5816, "Ost"),
            (403, 5834, "Nordost"),
        ] {
            assert_eq!(archive_of(TileId { east_km, north_km }), archive);
        }
    }

    #[test]
    fn a_tile_is_read_out_of_its_archive_and_kept_as_laz_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, 40, "2025-08-14");
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert_eq!(tiles.len(), 1, "{notes:?}");
        assert_eq!(
            tiles.iter().map(|t| &t.record).collect::<Vec<_>>(),
            [&PointCloudRecord {
                tile: "33391-5818".to_string(),
                url: archive_url("Mitte"),
                published: Some("2025-08-14".parse().unwrap()),
            }]
        );
        assert_eq!(folder(&cache), ["33391-5818_2025-08-14.laz"]);
        // LAZ marks its point format with the highest bit (byte 104 of the header).
        assert_ne!(std::fs::read(&tiles[0].path).unwrap()[104] & 0x80, 0, "LAZ");
        let mut reader = las::Reader::from_path(&tiles[0].path).unwrap();
        assert_eq!(reader.read_points(100).unwrap().len(), 40);
        assert_eq!(
            BerlinAlsBackend::new().cached(&cache, &tiles[0].record),
            Some(tiles[0].path.clone())
        );
        // The second fetch asks only the date of the archive.
        let reads = fetcher.log.borrow().len();
        let (again, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert_eq!(again, tiles);
        assert_eq!(fetcher.log.borrow().len(), reads);
        assert!(
            notes.iter().any(|n| n == "cached point cloud 33391-5818"),
            "{notes:?}"
        );
        // A changed archive replaces the cached file.
        serve(&fetcher, 50, "2026-02-01");
        let (newer, _) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert_eq!(newer[0].record.published, "2026-02-01".parse().ok());
        assert_eq!(folder(&cache), ["33391-5818_2026-02-01.laz"]);
    }

    #[test]
    fn a_tile_the_archive_lacks_is_not_published_and_offline_takes_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let url = archive_url("Mitte");
        fetcher.insert(&url, archive(&[("3dm_33_391_5817_1_be.las", las(3), true)]));
        fetcher.set_modified(&url, "2025-08-14".parse().unwrap());
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n == "no laser point cloud published for tile 33391-5818"),
            "{notes:?}"
        );
        let silent = MapFetcher::default();
        let (offline, notes) = fetch(&silent, &cache, FetchMode::Offline).unwrap();
        assert!(offline.is_empty());
        assert!(notes.iter().any(|n| n.contains("no cached point cloud")));
        serve(&fetcher, 10, "2025-08-14");
        fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        let (offline, _) = fetch(&silent, &cache, FetchMode::Offline).unwrap();
        assert_eq!(offline.len(), 1);
        assert_eq!(silent.calls.get(), 0, "offline asks nothing");
    }

    #[test]
    fn without_a_date_or_with_a_damaged_tile_the_cache_serves_and_a_cancel_stops() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        // No date: nothing to name the file by.
        fetcher.insert(&archive_url("Mitte"), Vec::new());
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        assert!(tiles.is_empty());
        assert!(
            notes.iter().any(
                |n| n.starts_with("warning: point cloud 33391-5818 unavailable")
                    && n.contains("no date")
            ),
            "{notes:?}"
        );
        serve(&fetcher, 10, "2025-08-14");
        fetch(&fetcher, &cache, FetchMode::Online).unwrap();
        // A newer archive whose member is damaged keeps the cached tile.
        let damaged = MapFetcher::default();
        let mut zip = archive(&[(&member_name(VIKTORIAPARK), las(20), false)]);
        let at = zip.windows(4).position(|w| w == b"LASF").unwrap();
        zip[at + 200] ^= 0xff;
        damaged.insert(&archive_url("Mitte"), zip);
        damaged.set_modified(&archive_url("Mitte"), "2026-02-01".parse().unwrap());
        let (tiles, notes) = fetch(&damaged, &cache, FetchMode::Online).unwrap();
        assert_eq!(tiles.len(), 1, "the cached tile");
        assert_eq!(tiles[0].record.published, "2025-08-14".parse().ok());
        assert!(notes.iter().any(|n| n.contains("CRC")), "{notes:?}");
        assert_eq!(folder(&cache), ["33391-5818_2025-08-14.laz"]);
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
                max_e: 392_100.0,
                ..BBOX
            },
            today: "2026-11-12".parse().unwrap(),
        };
        let items = BerlinAlsBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let what: Vec<(&str, f64)> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes))
            .collect();
        assert_eq!(
            what,
            [
                ("Geoportal Berlin point cloud 33391-5818", MB_PER_TILE),
                ("Geoportal Berlin point cloud 33392-5818", MB_PER_TILE),
            ]
        );
    }

    #[test]
    #[ignore = "network"]
    fn live_archive_names_the_viktoriapark_tile() {
        use crate::fetch::{Fetcher as _, HttpFetcher};
        let fetcher = HttpFetcher::new();
        let url = archive_url("Mitte");
        assert!(fetcher.last_modified(&url).unwrap().is_some());
        let members = read_members(&fetcher, &url).unwrap();
        let member = find_member(&members, |f| f == member_name(VIKTORIAPARK)).unwrap();
        assert!(member.compressed() > 100 << 20, "{}", member.compressed());
    }
}
