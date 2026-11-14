//! The laser point clouds of the Bayerische Vermessungsverwaltung (CC BY 4.0): one LAZ file per
//! km tile in EPSG:25832 under an address that follows from the tile (ADR 0128). No list names
//! the tiles, so a tile is kept like Bavaria's other files ([`crate::ldbv_files`]): named by the
//! day the server last changed it, and one `HEAD` tells whether it is still current. The day of
//! the flight is in the GPS time of the points, except in older lots with GPS week time.
//!
//! The classes are the agency's (September 2023): ground (2), buildings (6), water (9), objects
//! such as vegetation (20), cellar exits and, until 2020, bridges (22), and synthetic ground
//! (23, 24). `derive` leaves out 22 and 23 for Bavaria ([`PointCloudInfo::left_out`]) besides
//! what it leaves out everywhere.

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

use crate::boundaries::{BAYERN, tile_centre_in};
use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode};
use crate::las_files::{
    download_las, flight_day, header_day, parse_utm32_tile_name as parse_tile_name,
    utm32_tile_name as tile_name,
};
use crate::ldbv_files::{Checks, Current, Published, cached, current, drop_older};
use crate::note::{Arg, note};
use crate::tiles::{TileId, tiles_covering_grid};

/// The folder of the tiles. They lie on a third host of the agency; the server of the other
/// tiles (`download1.bayernwolke.de`) does not have them.
pub const TILES: &str = "https://geodaten.bayern.de/odd_data/laser";
/// The folder of the point clouds below the LDBV folder of the cache.
const DIR: &str = "las";
const EXT: &str = "laz";
/// Fourteen tiles across the state had 34 to 614 MB, 230 MB in the median (2026); the portal
/// says up to 1 GB. The estimate counts this much per tile.
pub const MB_PER_TILE: f64 = 250.0;
/// Classes of the agency that are no returns of what grows there: cellar exits (bridges until
/// 2020) and synthetic ground from a terrestrial survey.
const LEFT_OUT: [u8; 2] = [22, 23];

/// The name of a tile on the server and in the cache: its south-west corner in km.
fn stem(tile: TileId) -> String {
    format!("{}_{}", tile.east_km, tile.north_km)
}

/// URL of the point cloud of `tile`.
pub fn tile_url(tile: TileId) -> String {
    format!("{TILES}/{}.{EXT}", stem(tile))
}

/// The point-cloud backend of Bavaria.
pub fn backend() -> &'static dyn PointCloudBackend {
    static LDBV: LazyLock<LdbvLasBackend> = LazyLock::new(LdbvLasBackend::new);
    &*LDBV
}

/// The laser point clouds of the LDBV, kept in the cache as the server gives them.
pub struct LdbvLasBackend {
    info: PointCloudInfo,
}

impl LdbvLasBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in(crate::ldbv::PROVIDER),
                cache_dir: crate::ldbv::PROVIDER.to_string(),
                source: "ldbv-las".to_string(),
                epsg: 25832,
                grid_km: 1,
                left_out: LEFT_OUT.to_vec(),
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(DIR)
    }
}

impl Default for LdbvLasBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// The day the cached tile at `path` stands for: the day of its flight where its points tell
/// it (lots with GPS week time do not), else the day its header says it was made, else the day
/// the server last changed it (in the name of the file), else none.
fn published(path: &Path, today: NaiveDate) -> Option<NaiveDate> {
    flight_day(path, today)
        .or_else(|| header_day(path, today))
        .or_else(|| {
            let name = path.file_stem()?.to_str()?;
            let (_, day) = name.rsplit_once('-')?;
            NaiveDate::parse_from_str(day, "%Y%m%d").ok()
        })
}

impl PointCloudBackend for LdbvLasBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let (today, online) = (env.today, env.mode == FetchMode::Online);
        let mut checks = Checks::default();
        let mut fetched = Vec::new();
        for tile in tiles_covering_grid(bbox, self.info.grid_km) {
            let (name, stem, url) = (tile_name(tile), stem(tile), tile_url(tile));
            let file = Published {
                dir: &dir,
                stem: &stem,
                ext: EXT,
                url: &url,
                // Beyond the border a tile that is not there stays not there; inside Bavaria the
                // server is asked again at every fetch.
                missing_holds: !tile_centre_in(tile, BAYERN),
            };
            let found = |path: PathBuf| PointCloudTile {
                record: PointCloudRecord {
                    tile: name.clone(),
                    url: url.clone(),
                    published: published(&path, today),
                },
                path,
            };
            match current(env.fetcher, env.mode, &file, &mut checks, env.progress) {
                Ok(Current::Cached(path)) => {
                    if online {
                        (env.progress)(note!("progress-point-cloud-cached", tile = &name));
                    }
                    fetched.push(found(path));
                }
                Ok(Current::Missing) => {
                    (env.progress)(note!("progress-point-cloud-not-published", tile = &name));
                }
                Ok(Current::Fetch { path, older }) => {
                    (env.progress)(note!("progress-downloading", url = &url));
                    // No list names a size: the check for a complete point cloud is the check.
                    match download_las(env, &url, None, None, &path) {
                        Ok(()) => {
                            drop_older(&older);
                            fetched.push(found(path));
                        }
                        Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                        // The vegetation is only a candidate: a failing tile falls back to the
                        // newest copy in the cache or is left out.
                        Err(e) => match older.last() {
                            Some(copy) => {
                                (env.progress)(note!(
                                    "warning-point-cloud-unavailable-cached",
                                    tile = &name,
                                    reason = Arg::error(e)
                                ));
                                fetched.push(found(copy.clone()));
                            }
                            None => (env.progress)(note!(
                                "warning-point-cloud-unavailable-skipped",
                                tile = &name,
                                reason = Arg::error(e)
                            )),
                        },
                    }
                }
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                // Nothing cached, and offline or a server that does not answer.
                Err(_) if !online => {
                    (env.progress)(note!("warning-point-cloud-no-cached", tile = &name));
                }
                Err(e) => (env.progress)(note!(
                    "warning-point-cloud-unavailable-skipped",
                    tile = &name,
                    reason = Arg::error(e)
                )),
            }
        }
        freemapper_fs::remove_stale_cache_parts(&dir, STALE_PART_AGE);
        Ok(fetched)
    }

    /// The newest cached copy of the recorded tile: a project keeps finding its tile after
    /// another one fetched a newer copy.
    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        let tile = parse_tile_name(&record.tile)?;
        cached(&self.dir(cache), &stem(tile), EXT)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let dir = self.dir(env.cache);
        tiles_covering_grid(&area.map_bbox, self.info.grid_km)
            .into_iter()
            .map(|tile| EstimateItem {
                what: format!("LDBV point cloud {}", tile_name(tile)),
                megabytes: MB_PER_TILE,
                cached: cached(&dir, &stem(tile), EXT).is_some(),
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::registry::Registry;

    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::las_files::testing::{gps_noon, laz_created, laz_with_times};

    /// The tile of the Schmausenbuck, and its neighbour to the east.
    const TILE: TileId = TileId {
        east_km: 655,
        north_km: 5479,
    };
    const EAST: TileId = TileId {
        east_km: 656,
        north_km: 5479,
    };
    /// A tile in Hesse, beyond the border.
    const ABROAD: TileId = TileId {
        east_km: 500,
        north_km: 5510,
    };
    const NAME: &str = "32655-5479";

    fn day(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    /// A box of 100 m inside `tile`.
    fn inside(tile: TileId) -> BoundingBox {
        let (e, n) = (
            f64::from(tile.east_km) * 1000.0,
            f64::from(tile.north_km) * 1000.0,
        );
        BoundingBox {
            min_e: e + 300.0,
            min_n: n + 300.0,
            max_e: e + 400.0,
            max_n: n + 400.0,
        }
    }

    /// A box across the border of [`TILE`] and [`EAST`].
    const BOTH: BoundingBox = BoundingBox {
        min_e: 655_900.0,
        min_n: 5_479_300.0,
        max_e: 656_100.0,
        max_n: 5_479_400.0,
    };

    /// A tile whose points were measured on `flown`; with week time they do not say so.
    fn laz(flown: &str, standard: bool) -> Vec<u8> {
        laz_with_times(&[gps_noon(day(flown)); 30], standard)
    }

    /// Puts `bytes` on the server as `tile`, last changed on `changed` (or without a day).
    fn serve(fetcher: &MapFetcher, tile: TileId, bytes: Vec<u8>, changed: Option<&str>) {
        let url = tile_url(tile);
        fetcher.insert(&url, bytes);
        match changed {
            Some(changed) => fetcher.set_modified(&url, day(changed)),
            None => {
                fetcher.modified.borrow_mut().remove(&url);
            }
        }
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &CacheDir,
        mode: FetchMode,
        bbox: &BoundingBox,
    ) -> (Vec<PointCloudTile>, Vec<String>) {
        let registry = Registry::new();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache,
            mode,
            today: day("2026-11-14"),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        let tiles = LdbvLasBackend::new().fetch(&mut env, bbox).unwrap();
        (tiles, notes)
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("ldbv/las"))
            .map(|entries| {
                entries
                    .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn cache() -> (tempfile::TempDir, CacheDir) {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        (dir, cache)
    }

    #[test]
    fn the_address_follows_the_agency() {
        assert_eq!(
            tile_url(TILE),
            "https://geodaten.bayern.de/odd_data/laser/655_5479.laz"
        );
        let info = LdbvLasBackend::new().info;
        assert_eq!(
            (info.source.as_str(), info.epsg, info.grid_km),
            ("ldbv-las", 25832, 1)
        );
        assert_eq!(info.left_out, [22, 23]);
    }

    #[test]
    fn a_fetch_downloads_the_tile_and_records_the_flight_day() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles.len(), 1, "{notes:?}");
        assert_eq!(
            tiles[0].record,
            PointCloudRecord {
                tile: NAME.to_string(),
                url: tile_url(TILE),
                published: Some(day("2021-12-20")),
            }
        );
        assert_eq!(folder(&cache), ["655_5479-20230310.laz"]);
        assert!(tiles[0].path.ends_with("ldbv/las/655_5479-20230310.laz"));
        assert!(notes.iter().all(|n| !n.starts_with("warning")), "{notes:?}");
    }

    #[test]
    fn a_second_fetch_asks_only_for_the_day() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        fetcher.log.borrow_mut().clear();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].record.published, Some(day("2021-12-20")));
        assert!(fetcher.log.borrow().is_empty(), "no download");
        assert!(
            notes.iter().any(|n| n == "cached point cloud 32655-5479"),
            "{notes:?}"
        );
    }

    #[test]
    fn a_newer_file_replaces_the_cached_one() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        serve(&fetcher, TILE, laz("2026-03-02", true), Some("2026-09-03"));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles[0].record.published, Some(day("2026-03-02")));
        assert_eq!(folder(&cache), ["655_5479-20260903.laz"]);
    }

    #[test]
    fn a_tile_with_week_time_is_dated_by_the_server() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", false), Some("2023-03-10"));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles[0].record.published, Some(day("2023-03-10")));
    }

    #[test]
    fn a_tile_with_week_time_is_dated_by_its_header() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        // A lot of 2011, processed in 2018 and put on the server in 2023: its points do not
        // tell the day, and the day it was processed is closer to the flight.
        let old_lot = |created: &str| {
            laz_created(
                &[gps_noon(day("2021-12-20")); 30],
                false,
                Some(day(created)),
            )
        };
        serve(&fetcher, TILE, old_lot("2018-08-08"), Some("2023-03-10"));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles[0].record.published, Some(day("2018-08-08")));
        // A header day after today or before the first flights is no day.
        for odd in ["2031-01-01", "1980-01-06"] {
            let (_dir, cache) = self::cache();
            serve(&fetcher, TILE, old_lot(odd), Some("2023-03-10"));
            let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
            assert_eq!(tiles[0].record.published, Some(day("2023-03-10")), "{odd}");
        }
        // The points win over the header where they tell the day.
        let (_dir, cache) = self::cache();
        let flown = laz_created(
            &[gps_noon(day("2021-12-20")); 30],
            true,
            Some(day("2022-05-18")),
        );
        serve(&fetcher, TILE, flown, Some("2023-03-10"));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles[0].record.published, Some(day("2021-12-20")));
    }

    #[test]
    fn a_server_without_a_day_still_gives_the_tile() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", false), None);
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles.len(), 1, "{notes:?}");
        assert_eq!(tiles[0].record.published, None);
        assert_eq!(folder(&cache), ["655_5479-00000000.laz"]);
        // The points still tell the day where they can.
        let (_dir, cache) = self::cache();
        serve(&fetcher, TILE, laz("2021-12-20", true), None);
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles[0].record.published, Some(day("2021-12-20")));
    }

    #[test]
    fn a_broken_download_falls_back_to_the_cached_copy() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        // The newer file is an error page with status 200.
        serve(
            &fetcher,
            TILE,
            b"<html>maintenance</html>".to_vec(),
            Some("2026-09-03"),
        );
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].record.published, Some(day("2021-12-20")));
        assert!(tiles[0].path.ends_with("655_5479-20230310.laz"));
        assert_eq!(folder(&cache), ["655_5479-20230310.laz"]);
        assert!(
            notes.iter().any(
                |n| n.starts_with("warning: point cloud 32655-5479 unavailable")
                    && n.ends_with("using the cached file")
            ),
            "{notes:?}"
        );
    }

    #[test]
    fn a_broken_download_without_a_copy_leaves_the_tile_out() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(
            &fetcher,
            TILE,
            b"<html>maintenance</html>".to_vec(),
            Some("2026-09-03"),
        );
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert!(tiles.is_empty());
        assert!(folder(&cache).is_empty(), "{:?}", folder(&cache));
        assert!(
            notes.iter().any(
                |n| n.starts_with("warning: point cloud 32655-5479 unavailable")
                    && n.ends_with("no vegetation candidates there")
            ),
            "{notes:?}"
        );
    }

    #[test]
    fn a_tile_the_server_lacks_is_a_note() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert!(tiles.is_empty());
        assert_eq!(
            notes,
            ["no laser point cloud published for tile 32655-5479"]
        );
        // Inside Bavaria the server is asked again at the next fetch.
        let asked = fetcher.calls.get();
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(fetcher.calls.get(), asked + 1);
        // Beyond the border a tile that is not there stays not there.
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(ABROAD));
        assert!(tiles.is_empty());
        let asked = fetcher.calls.get();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(ABROAD));
        assert!(tiles.is_empty());
        assert_eq!(fetcher.calls.get(), asked);
        assert_eq!(
            notes,
            ["no laser point cloud published for tile 32500-5510"]
        );
    }

    #[test]
    fn offline_serves_the_cached_tile_and_warns_without_one() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Offline, &inside(TILE));
        assert!(tiles.is_empty());
        assert_eq!(
            notes,
            ["warning: no cached point cloud for tile 32655-5479; no vegetation candidates there"]
        );
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        let asked = fetcher.calls.get();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Offline, &inside(TILE));
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].record.published, Some(day("2021-12-20")));
        assert_eq!(fetcher.calls.get(), asked, "nothing is asked offline");
        assert!(notes.is_empty(), "{notes:?}");
    }

    #[test]
    fn a_silent_server_costs_one_check() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        for tile in [TILE, EAST] {
            serve(&fetcher, tile, laz("2021-12-20", true), Some("2023-03-10"));
        }
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &BOTH);
        assert_eq!(tiles.len(), 2);
        for tile in [TILE, EAST] {
            fetcher.failing.borrow_mut().push(tile_url(tile));
        }
        let asked = fetcher.calls.get();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &BOTH);
        assert_eq!(tiles.len(), 2, "both cached tiles serve");
        assert_eq!(
            fetcher.calls.get(),
            asked + 1,
            "one check, not one per tile"
        );
        let warnings: Vec<&String> = notes.iter().filter(|n| n.starts_with("warning")).collect();
        assert_eq!(warnings.len(), 1, "{notes:?}");
        assert!(warnings[0].contains("could not check whether"));
    }

    #[test]
    fn a_silent_server_without_a_copy_leaves_the_tile_out() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetcher.failing.borrow_mut().push(tile_url(TILE));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: point cloud 32655-5479 unavailable")),
            "{notes:?}"
        );
    }

    #[test]
    fn the_recorded_tile_is_found_after_a_newer_copy_came() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        let backend = LdbvLasBackend::new();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        let record = tiles[0].record.clone();
        assert_eq!(backend.cached(&cache, &record), Some(tiles[0].path.clone()));
        serve(&fetcher, TILE, laz("2026-03-02", true), Some("2026-09-03"));
        let (newer, _) = fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        assert_eq!(backend.cached(&cache, &record), Some(newer[0].path.clone()));
        // A record that names no tile of this grid finds nothing.
        let odd = PointCloudRecord {
            tile: "../655_5479".to_string(),
            ..record
        };
        assert_eq!(backend.cached(&cache, &odd), None);
    }

    #[test]
    fn the_estimate_counts_250_mb_a_tile() {
        let (_dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, TILE, laz("2021-12-20", true), Some("2023-03-10"));
        fetch(&fetcher, &cache, FetchMode::Online, &inside(TILE));
        let area = EstimateArea {
            bbox_25833: BOTH,
            map_bbox: BOTH,
            today: day("2026-11-14"),
        };
        let items = LdbvLasBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let seen: Vec<_> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes, i.cached, i.part, i.store))
            .collect();
        assert_eq!(
            seen,
            [
                (
                    "LDBV point cloud 32655-5479",
                    250.0,
                    true,
                    Part::PointCloud,
                    Store::Cache
                ),
                (
                    "LDBV point cloud 32656-5479",
                    250.0,
                    false,
                    Part::PointCloud,
                    Store::Cache
                ),
            ]
        );
    }
}
