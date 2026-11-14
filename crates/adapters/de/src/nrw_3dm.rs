//! The 3D-Messdaten of Geobasis NRW (dl-de/zero-2-0): one LAZ file per km tile in EPSG:25832,
//! listed with its size and date in an `index.json` (ADR 0105). The classes follow the AdV scheme:
//! ground (2), last echoes above ground (20), intermediate echoes (1), and synthetic fill points
//! (26, 21, 9), cellar points (24) and bridges (17), which `derive` leaves out.

use std::collections::HashMap;
use std::path::PathBuf;
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
    cached_las, download_las, las_path, not_published, parse_utm32_tile_name as parse_tile_name,
    utm32_tile_name as tile_name,
};
use crate::note::{Arg, note};
use crate::tiles::{TileId, tiles_covering_grid};

/// The folder of the tiles; the product folder appears twice in the path.
const BASE: &str = "https://www.opengeodata.nrw.de/produkte/geobasis/hm/3dm_l_las/3dm_l_las";
/// Every tile with its size and the time it was published (about 3 MB for 36 000 tiles, 2026).
pub const INDEX_URL: &str =
    "https://www.opengeodata.nrw.de/produkte/geobasis/hm/3dm_l_las/3dm_l_las/index.json";
/// The folder of the point clouds below the NRW folder of the cache.
const DIR: &str = "3dm";
/// A tile has 80 MB in the median and less than 165 MB in nine of ten cases (2026); the estimate
/// counts this much per tile.
pub const MB_PER_TILE: f64 = 120.0;

/// URL of the tile file `name` (from the index).
pub fn tile_url(name: &str) -> String {
    format!("{BASE}/{name}")
}

/// The point-cloud backend of North Rhine-Westphalia.
pub fn backend() -> &'static dyn PointCloudBackend {
    static NRW: LazyLock<Nrw3dmBackend> = LazyLock::new(Nrw3dmBackend::new);
    &*NRW
}

/// The 3D-Messdaten of Geobasis NRW, kept in the cache as LAZ.
pub struct Nrw3dmBackend {
    info: PointCloudInfo,
}

impl Nrw3dmBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in("geobasisnrw"),
                cache_dir: "geobasis_nrw".to_string(),
                source: "geobasisnrw-3dm".to_string(),
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

impl Default for Nrw3dmBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// One tile of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexTile {
    /// `3dm_32_356_5644_1_nw.laz`.
    name: String,
    bytes: u64,
    published: NaiveDate,
}

/// The tile a name like `3dm_32_356_5644_1_nw.laz` stands for.
fn tile_of_name(name: &str) -> Option<TileId> {
    let stem = name.strip_suffix(".laz")?;
    if !crate::lvermgeo_sh::is_plain(stem) {
        return None;
    }
    let ["3dm", "32", east, north, "1", "nw"] = stem.split('_').collect::<Vec<_>>()[..] else {
        return None;
    };
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// The tiles of the index, the newest file of each; names, sizes or dates it cannot read are
/// left out.
fn parse_index(bytes: &[u8]) -> Result<HashMap<TileId, IndexTile>, FetchError> {
    let index: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| FetchError::CorruptArchive {
            url: INDEX_URL.to_string(),
            message: e.to_string(),
        })?;
    let datasets = index["datasets"]
        .as_array()
        .ok_or_else(|| FetchError::CorruptArchive {
            url: INDEX_URL.to_string(),
            message: "no `datasets`".to_string(),
        })?;
    let mut tiles: HashMap<TileId, IndexTile> = HashMap::new();
    let files = datasets
        .iter()
        .filter_map(|d| d["files"].as_array())
        .flatten();
    for file in files {
        let text = |key: &str| file[key].as_str().unwrap_or_default();
        let (name, size, timestamp) = (text("name"), text("size"), text("timestamp"));
        let (Some(tile), Ok(bytes), Some(Ok(published))) = (
            tile_of_name(name),
            size.parse::<u64>(),
            timestamp.get(..10).map(str::parse::<NaiveDate>),
        ) else {
            continue;
        };
        let entry = IndexTile {
            name: name.to_string(),
            bytes,
            published,
        };
        match tiles.get(&tile) {
            Some(old) if old.published >= entry.published => {}
            _ => {
                tiles.insert(tile, entry);
            }
        }
    }
    Ok(tiles)
}

impl PointCloudBackend for Nrw3dmBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        // Without the index the cached tiles serve, as offline.
        let mut offline = env.mode == FetchMode::Offline;
        let index = if offline {
            HashMap::new()
        } else {
            (env.progress)(note!("progress-downloading", url = INDEX_URL));
            match env
                .fetcher
                .get(INDEX_URL)
                .and_then(|bytes| parse_index(&bytes))
            {
                Ok(index) => index,
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Err(e) => {
                    (env.progress)(note!(
                        "warning-point-clouds-unavailable",
                        reason = Arg::error(e)
                    ));
                    offline = true;
                    HashMap::new()
                }
            }
        };
        let mut fetched = Vec::new();
        for tile in tiles_covering_grid(bbox, self.info.grid_km) {
            let cached = cached_las(&dir, &tile_name(tile));
            let found = |url: String, (published, path): (NaiveDate, PathBuf)| PointCloudTile {
                record: PointCloudRecord {
                    tile: tile_name(tile),
                    url,
                    published: Some(published),
                },
                path,
            };
            let name = format!("3dm_32_{}_{}_1_nw.laz", tile.east_km, tile.north_km);
            if offline {
                match cached.into_iter().next() {
                    Some(newest) => fetched.push(found(tile_url(&name), newest)),
                    None => (env.progress)(note!(
                        "warning-point-cloud-no-cached",
                        tile = tile_name(tile)
                    )),
                }
                continue;
            }
            let Some(entry) = index.get(&tile) else {
                let found = |newest| found(tile_url(&name), newest);
                not_published(env, &tile_name(tile), cached, &mut fetched, found);
                continue;
            };
            let url = tile_url(&entry.name);
            let path = las_path(&dir, &tile_name(tile), entry.published);
            if path.exists() {
                (env.progress)(note!("progress-point-cloud-cached", tile = tile_name(tile)));
                fetched.push(found(url, (entry.published, path)));
                continue;
            }
            (env.progress)(note!("progress-downloading", url = &url));
            match download_las(env, &url, entry.bytes, None, &path) {
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Ok(()) => {
                    for (_, older) in cached.into_iter().filter(|(_, p)| *p != path) {
                        let _ = std::fs::remove_file(older);
                    }
                    fetched.push(found(url, (entry.published, path)));
                }
                // The vegetation is only a candidate: a failing tile falls back to the cache or
                // is left out.
                Err(e) => match cached.into_iter().next() {
                    Some(older) => {
                        (env.progress)(note!(
                            "warning-point-cloud-unavailable-cached",
                            tile = tile_name(tile),
                            reason = Arg::error(e)
                        ));
                        fetched.push(found(url, older));
                    }
                    None => (env.progress)(note!(
                        "warning-point-cloud-unavailable-skipped",
                        tile = tile_name(tile),
                        reason = Arg::error(e)
                    )),
                },
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
                what: format!("Geobasis NRW point cloud {}", tile_name(tile)),
                megabytes: MB_PER_TILE,
                cached: !cached_las(&dir, &tile_name(tile)).is_empty(),
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

    const TILE: &str = "32356-5644";
    const NAME: &str = "3dm_32_356_5644_1_nw.laz";
    /// A box inside the tile, in Cologne.
    const BBOX: BoundingBox = BoundingBox {
        min_e: 356_300.0,
        min_n: 5_644_300.0,
        max_e: 356_400.0,
        max_n: 5_644_400.0,
    };

    fn registry() -> Registry {
        let mut registry = Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        registry
    }

    /// A small LAZ point cloud of `points` points, in EPSG:25832 like the tiles.
    fn laz(points: u32) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 2));
        builder.point_format.is_compressed = true;
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(300_000.0),
            y: transform(5_600_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..points {
            writer
                .write_point(las::Point {
                    x: 356_350.0,
                    y: 5_644_350.0 + f64::from(i) * 0.2,
                    z: 50.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }

    fn index(files: &[(&str, usize, &str)]) -> Vec<u8> {
        let files: Vec<serde_json::Value> = files
            .iter()
            .map(|(name, size, timestamp)| {
                serde_json::json!({"name": name, "size": size.to_string(), "timestamp": timestamp})
            })
            .collect();
        serde_json::json!({
            "metafiles": [{"name": "3dm_meta.zip", "size": "110499", "timestamp": "2026-11-10T22:25:03"}],
            "datasets": [{"name": "3dm_kacheln", "title": "3D-Messdaten", "files": files}]
        })
        .to_string()
        .into_bytes()
    }

    fn serve(fetcher: &MapFetcher, laz: Vec<u8>, timestamp: &str) {
        fetcher.insert(INDEX_URL, index(&[(NAME, laz.len(), timestamp)]));
        fetcher.insert(&tile_url(NAME), laz);
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &CacheDir,
        mode: FetchMode,
    ) -> (Vec<PointCloudTile>, Vec<String>) {
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
        let tiles = Nrw3dmBackend::new().fetch(&mut env, &BBOX).unwrap();
        (tiles, notes)
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("geobasis_nrw/3dm"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn tile_names_are_read_strictly() {
        assert_eq!(
            tile_of_name(NAME),
            Some(TileId {
                east_km: 356,
                north_km: 5644
            })
        );
        for bad in [
            "3dm_32_356_5644_1_nw.las",
            "3dm_33_356_5644_1_nw.laz",
            "3dm_32_356_5644_2_nw.laz",
            "../3dm_32_356_5644_1_nw.laz",
            "3dm_32_x_5644_1_nw.laz",
        ] {
            assert_eq!(tile_of_name(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_index_keeps_the_newest_file_of_a_tile_and_skips_what_it_cannot_read() {
        let text = index(&[
            (NAME, 10, "2022-08-08T10:35:40"),
            (NAME, 20, "2024-01-02T00:00:00"),
            ("3dm_32_357_5644_1_nw.laz", 30, "no date"),
            ("readme.txt", 1, "2024-01-02T00:00:00"),
        ]);
        let tiles = parse_index(&text).unwrap();
        assert_eq!(tiles.len(), 1);
        let entry = &tiles[&parse_tile_name(TILE).unwrap()];
        assert_eq!(
            (entry.bytes, entry.published),
            (20, "2024-01-02".parse().unwrap())
        );
        assert!(parse_index(b"<html>").is_err());
    }

    #[test]
    fn a_tile_is_downloaded_checked_and_recorded_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, laz(50), "2023-07-19T07:30:49");
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        assert_eq!(
            tiles[0].record,
            PointCloudRecord {
                tile: TILE.to_string(),
                url: format!("{BASE}/{NAME}"),
                published: Some("2023-07-19".parse().unwrap()),
            }
        );
        assert_eq!(folder(&cache), ["32356-5644_2023-07-19.laz"]);
        assert_eq!(
            Nrw3dmBackend::new().cached(&cache, &tiles[0].record),
            Some(tiles[0].path.clone())
        );
        let (again, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(again, tiles);
        let downloads = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.ends_with(".laz"))
            .count();
        assert_eq!(downloads, 1);
        assert!(
            notes.iter().any(|n| n == "cached point cloud 32356-5644"),
            "{notes:?}"
        );
        // A newer publication replaces the cached file.
        serve(&fetcher, laz(60), "2025-03-01T00:00:00");
        let (newer, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(newer[0].record.published, "2025-03-01".parse().ok());
        assert_eq!(folder(&cache), ["32356-5644_2025-03-01.laz"]);
    }

    #[test]
    fn a_tile_of_another_size_than_the_index_says_is_not_kept() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let file = laz(50);
        fetcher.insert(
            INDEX_URL,
            index(&[(NAME, file.len() + 1, "2023-07-19T07:30:49")]),
        );
        fetcher.insert(&tile_url(NAME), file);
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(tiles.is_empty());
        assert!(
            notes.iter().any(
                |n| n.starts_with("warning: point cloud 32356-5644 unavailable")
                    && n.contains("the index says")
            ),
            "{notes:?}"
        );
        assert!(folder(&cache).is_empty());
    }

    #[test]
    fn a_tile_the_index_lacks_is_not_published_and_offline_takes_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(INDEX_URL, index(&[]));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n == "no laser point cloud published for tile 32356-5644"),
            "{notes:?}"
        );
        let (offline, notes) = fetch(&MapFetcher::default(), &cache, FetchMode::Offline);
        assert!(offline.is_empty());
        assert!(
            notes.iter().any(|n| n.contains("no cached point cloud")),
            "{notes:?}"
        );
        serve(&fetcher, laz(10), "2023-07-19T07:30:49");
        fetch(&fetcher, &cache, FetchMode::Online);
        let silent = MapFetcher::default();
        let (offline, _) = fetch(&silent, &cache, FetchMode::Offline);
        assert_eq!(offline.len(), 1);
        assert_eq!(silent.calls.get(), 0, "offline asks nothing");
    }

    #[test]
    fn without_the_index_the_cached_tiles_serve_and_a_cancel_stops() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, laz(10), "2023-07-19T07:30:49");
        fetch(&fetcher, &cache, FetchMode::Online);
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(INDEX_URL.to_string());
        let (tiles, notes) = fetch(&down, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1, "the cached tile");
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: the laser point clouds are unavailable")),
            "{notes:?}"
        );
        // A stop asked for is no failing service: it ends the fetch.
        struct Stopped;
        impl crate::fetch::Fetcher for Stopped {
            fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
                Err(FetchError::Cancelled)
            }
        }
        let registry = registry();
        let stopped = Stopped;
        let mut env = AdapterEnv {
            fetcher: &stopped,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-12".parse().unwrap(),
            skip_surface: false,
            progress: &mut |_| {},
        };
        assert!(Nrw3dmBackend::new().fetch(&mut env, &BBOX).is_err());
    }

    #[test]
    fn the_estimate_counts_every_tile_of_the_map_box() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let area = EstimateArea {
            bbox_25833: BBOX,
            map_bbox: BoundingBox {
                max_e: 357_100.0,
                ..BBOX
            },
            today: "2026-11-12".parse().unwrap(),
        };
        let items = Nrw3dmBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let what: Vec<(&str, f64, Part)> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes, i.part))
            .collect();
        assert_eq!(
            what,
            [
                (
                    "Geobasis NRW point cloud 32356-5644",
                    MB_PER_TILE,
                    Part::PointCloud
                ),
                (
                    "Geobasis NRW point cloud 32357-5644",
                    MB_PER_TILE,
                    Part::PointCloud
                ),
            ]
        );
    }

    #[test]
    #[ignore = "network"]
    fn live_index_names_the_cologne_tile() {
        use crate::fetch::{Fetcher as _, HttpFetcher};
        let tiles = parse_index(&HttpFetcher::new().get(INDEX_URL).unwrap()).unwrap();
        assert!(tiles.len() > 30_000, "{}", tiles.len());
        assert!(tiles.contains_key(&parse_tile_name(TILE).unwrap()));
    }
}
