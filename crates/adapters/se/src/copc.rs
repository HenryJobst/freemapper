// SPDX-License-Identifier: AGPL-3.0-or-later

//! The laser data of Lantmäteriet (Laserdata Skog): classified point clouds in 2.5 km tiles of
//! about 80 MB, COPC files that the `las` crate reads as LAZ. The tiles are found through the STAC
//! API and kept whole in the cache; `derive` reads the vegetation classes from them.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::FetchMode;
use freemapper_adapter_api::note::Arg;
use freemapper_adapter_api::pointcloud::{
    PointCloudBackend, PointCloudInfo, PointCloudRecord, PointCloudTile,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};
use freemapper_georef::crs::BoundingBox;
use freemapper_sources::stac;

use crate::account;
use crate::stac_terrain::{PROVIDER, degrees};

/// The collection of the laser data in the height catalogue.
const COLLECTION: &str = "dsm-skoglig-copc";

/// The product to order in Geotorget for these files.
const ORDER: &str = "Laserdata Skog";

/// The folder of the tiles below the cache folder of the provider.
const DIR: &str = "copc";

/// The side of a tile in metres.
const TILE_M: f64 = 2_500.0;

/// What a tile weighs, in MB (81.3 and 81.7 MB measured, `docs/regions/sweden-reference.md` §3.1).
const MB_PER_TILE: f64 = 82.0;

/// The most a tile may have: five times what one weighs.
const MAX_TILE_BYTES: u64 = 400 << 20;

/// The laser data of Lantmäteriet.
pub struct CopcBackend {
    info: PointCloudInfo,
    /// The STAC API of the height products, from the manifest (`[point_cloud] url`).
    stac_hojd: String,
}

impl CopcBackend {
    /// The backend that asks the STAC API `stac_hojd`.
    pub fn new(stac_hojd: &str) -> Self {
        Self {
            stac_hojd: stac_hojd.to_string(),
            info: PointCloudInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: PROVIDER.to_string(),
                source: "lantmateriet-skog".to_string(),
                epsg: 25833,
                // Not read by the engine, only by the German backends that built it. The tiles
                // are 2.5 km; a whole number of km cannot say it, so it is the next one up.
                grid_km: 3,
                left_out: Vec::new(),
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(DIR)
    }
}

/// Whether `id` is fit to be a file name: the catalogue is not trusted with a path.
fn is_name(id: &str) -> bool {
    (1..=80).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}

fn file_of(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.copc.laz"))
}

impl PointCloudBackend for CopcBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let online = env.mode == FetchMode::Online;
        // The tiles the cache holds are the ones offline; the catalogue is asked online only.
        let items = match (online, degrees(bbox)) {
            (true, Some(wgs84)) => {
                stac::search(env.fetcher, &self.stac_hojd, COLLECTION, wgs84, 100)?
            }
            _ => Vec::new(),
        };
        let mut fetched = Vec::new();
        for item in items {
            let id = item.id.as_str();
            let Some(asset) = item.assets.values().find(|a| a.href.ends_with(".copc.laz")) else {
                continue;
            };
            if !is_name(id) {
                continue;
            }
            let path = file_of(&dir, id);
            let record = PointCloudRecord {
                tile: id.to_string(),
                url: asset.href.clone(),
                published: item
                    .datetime
                    .as_deref()
                    .and_then(|day| NaiveDate::parse_from_str(day.get(..10)?, "%Y-%m-%d").ok()),
            };
            if path.exists() {
                (env.progress)(freemapper_adapter_api::note!(
                    "progress-point-cloud-cached",
                    tile = id
                ));
                fetched.push(PointCloudTile { record, path });
                continue;
            }
            (env.progress)(freemapper_adapter_api::note!(
                "progress-downloading",
                url = &asset.href
            ));
            match download(env, &asset.href, &path) {
                Ok(()) => fetched.push(PointCloudTile { record, path }),
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                // The vegetation is only a candidate: a tile that fails is left out.
                Err(e) => {
                    let e = account::refusal(&asset.href, ORDER, e);
                    // The next tile would be refused the same way: no order, or the cap.
                    let stop = account::is_cap(&e) || account::is_not_ordered(&e);
                    (env.progress)(freemapper_adapter_api::note!(
                        "warning-point-cloud-unavailable-skipped",
                        tile = id,
                        reason = Arg::error(e)
                    ));
                    if stop {
                        break;
                    }
                }
            }
        }
        if !online {
            fetched = cached_tiles(&dir, bbox);
        }
        Ok(fetched)
    }

    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        is_name(&record.tile)
            .then(|| file_of(&self.dir(cache), &record.tile))
            .filter(|path| path.exists())
    }

    fn estimate(&self, area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let b = &area.map_bbox;
        let cells = |from: f64, to: f64| {
            ((to / TILE_M - 1e-9).ceil() - (from / TILE_M).floor()).max(1.0) as usize
        };
        let count = cells(b.min_e, b.max_e) * cells(b.min_n, b.max_n);
        (0..count)
            .map(|_| EstimateItem {
                what: "Lantmäteriet point cloud (2.5 km tile)".to_string(),
                megabytes: MB_PER_TILE,
                cached: false,
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

/// Downloads `url` to `path` through a part file and keeps it only when it is a laser file whose
/// header `las` reads.
fn download(env: &mut AdapterEnv<'_>, url: &str, path: &Path) -> Result<(), FetchError> {
    let part = path.with_extension("part");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    }
    let result = env
        .fetcher
        .download_to(url, &part, MAX_TILE_BYTES, env.progress)
        .and_then(|()| {
            las::Reader::from_path(&part)
                .map(drop)
                .map_err(|e| FetchError::CorruptArchive {
                    url: url.to_string(),
                    message: format!("not a laser point cloud: {e}"),
                })
        })
        .and_then(|()| {
            std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// Offline: the cached tiles whose name says they lie under `bbox`. A name is
/// `<flight>-<north>_<east>_25` with the corner in units of 100 m.
fn cached_tiles(dir: &Path, bbox: &BoundingBox) -> Vec<PointCloudTile> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut tiles: Vec<PointCloudTile> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let id = path
                .file_name()?
                .to_str()?
                .strip_suffix(".copc.laz")?
                .to_string();
            let (_, corner) = id.rsplit_once('-')?;
            let mut parts = corner.split('_');
            let north = parts.next()?.parse::<f64>().ok()? * 100.0;
            let east = parts.next()?.parse::<f64>().ok()? * 100.0;
            let under = east < bbox.max_e
                && east + TILE_M > bbox.min_e
                && north < bbox.max_n
                && north + TILE_M > bbox.min_n;
            under.then(|| PointCloudTile {
                record: PointCloudRecord {
                    tile: id,
                    url: String::new(),
                    published: None,
                },
                path,
            })
        })
        .collect();
    tiles.sort_by(|a, b| a.record.tile.cmp(&b.record.tile));
    tiles
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::env::AdapterEnv;
    use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
    use freemapper_adapter_api::fetch::FetchMode;
    use freemapper_adapter_api::pointcloud::PointCloudBackend;
    use freemapper_adapter_api::records::{Part, Store};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_georef::crs::BoundingBox;

    use super::*;

    const STAC: &str = "https://stac.example.org/stac-hojd/v1";
    const TILE: &str = "https://dl.example.org/hojd/data/sls/25a005/y25a005-61775_3850_25.copc.laz";

    /// A LAZ file of format 6 with three points, which is what a COPC tile is to `las`.
    fn laz() -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 4));
        builder.point_format = las::point::Format::new(6).unwrap();
        builder.point_format.is_compressed = true;
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(385_000.0),
            y: transform(6_177_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for (i, class) in [2_u8, 1, 17].into_iter().enumerate() {
            writer
                .write_point(las::Point {
                    x: 385_100.0 + i as f64,
                    y: 6_177_600.0,
                    z: 40.0,
                    classification: las::point::Classification::new(class).unwrap(),
                    gps_time: Some(1.0),
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }

    fn answer() -> Vec<u8> {
        format!(
            r#"{{"type":"FeatureCollection","features":[{{"id":"25a005-61775_3850_25",
            "collection":"dsm-skoglig-copc","properties":{{"datetime":"2025-08-14"}},
            "assets":{{"data":{{"href":"{TILE}","file:size":81714617}}}}}}],"links":[]}}"#
        )
        .into_bytes()
    }

    fn served() -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{STAC}/search?collections=dsm-skoglig-copc"),
            answer(),
        );
        fetcher.insert(TILE, laz());
        fetcher
    }

    /// The area 385 000 to 385 200 by 6 177 500 to 6 177 700, inside one 2.5 km tile.
    fn area() -> BoundingBox {
        BoundingBox {
            min_e: 385_000.0,
            min_n: 6_177_500.0,
            max_e: 385_200.0,
            max_n: 6_177_700.0,
        }
    }

    fn backend() -> CopcBackend {
        CopcBackend::new(STAC)
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &CacheDir,
        mode: FetchMode,
    ) -> (
        Result<
            Vec<freemapper_adapter_api::pointcloud::PointCloudTile>,
            freemapper_adapter_api::error::AdapterError,
        >,
        Vec<String>,
    ) {
        let regions = Registry::new();
        let mut lines = Vec::new();
        let result = {
            let mut progress = |note: freemapper_adapter_api::note::Note| {
                lines.push(note.id().to_string());
            };
            let mut env = AdapterEnv {
                fetcher,
                regions: &regions,
                cache,
                mode,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            backend().fetch(&mut env, &area())
        };
        (result, lines)
    }

    #[test]
    fn the_info_names_the_provider_and_the_crs() {
        let info = backend().info().clone();
        assert_eq!(info.id.as_str(), "lantmateriet");
        assert_eq!(info.source, "lantmateriet-skog");
        assert_eq!(info.epsg, 25833);
    }

    #[test]
    fn fetch_downloads_the_tile_under_the_area_into_the_cache() {
        let fetcher = served();
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let (result, _) = fetch(&fetcher, &cache, FetchMode::Online);
        let tiles = result.unwrap();
        assert_eq!(tiles.len(), 1);
        let tile = &tiles[0];
        assert_eq!(tile.record.tile, "25a005-61775_3850_25");
        assert_eq!(tile.record.url, TILE);
        assert_eq!(tile.record.published, NaiveDate::from_ymd_opt(2025, 8, 14));
        assert!(tile.path.starts_with(scratch.path().join("cache")));
        let reader = las::Reader::from_path(&tile.path).unwrap();
        assert_eq!(reader.header().number_of_points(), 3);
        assert_eq!(
            backend().cached(&cache, &tile.record),
            Some(tile.path.clone())
        );
    }

    #[test]
    fn a_cached_tile_is_not_downloaded_again_and_offline_uses_it() {
        let fetcher = served();
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        fetch(&fetcher, &cache, FetchMode::Online).0.unwrap();
        let downloads = |f: &MapFetcher| {
            f.log
                .borrow()
                .iter()
                .filter(|u| u.ends_with(".laz"))
                .count()
        };
        assert_eq!(downloads(&fetcher), 1);
        let (second, lines) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(second.unwrap().len(), 1);
        assert_eq!(downloads(&fetcher), 1);
        assert!(
            lines.contains(&"progress-point-cloud-cached".to_string()),
            "{lines:?}"
        );
        let (offline, _) = fetch(&MapFetcher::default(), &cache, FetchMode::Offline);
        assert_eq!(offline.unwrap().len(), 1);
    }

    #[test]
    fn a_tile_that_is_no_laz_is_left_out_with_a_warning_and_not_kept() {
        let fetcher = served();
        fetcher.insert(TILE, b"<html>login</html>".to_vec());
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let (result, lines) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(result.unwrap().is_empty());
        assert!(
            lines.contains(&"warning-point-cloud-unavailable-skipped".to_string()),
            "{lines:?}"
        );
        assert!(backend().cached(&cache, &record_of_the_tile()).is_none());
    }

    /// A download host that answers every tile with `status`, as it does for an account without
    /// the order (401) or one over its limit (429), the way a plain download reports it.
    struct Answering<'a> {
        inner: &'a MapFetcher,
        status: u16,
    }

    impl freemapper_adapter_api::fetch::Fetcher for Answering<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.inner.get(url)
        }

        fn get_range(
            &self,
            url: &str,
            range: freemapper_adapter_api::fetch::ByteRange,
        ) -> Result<Vec<u8>, FetchError> {
            self.inner.get_range(url, range)
        }

        fn download_to(
            &self,
            url: &str,
            path: &std::path::Path,
            max_bytes: u64,
            progress: &mut dyn FnMut(freemapper_adapter_api::note::Note),
        ) -> Result<(), FetchError> {
            if url.ends_with(".laz") {
                self.inner.log.borrow_mut().push(url.to_string());
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: format!("http status: {}", self.status),
                });
            }
            self.inner.download_to(url, path, max_bytes, progress)
        }
    }

    /// The catalogue answer with two tiles side by side.
    fn two_tiles() -> MapFetcher {
        let fetcher = served();
        let second = TILE.replace("61775_3850", "61775_3852");
        let body = String::from_utf8(answer()).unwrap();
        let item = body
            .split_once("\"features\":[")
            .and_then(|(_, rest)| rest.split_once("],\"links\""))
            .unwrap()
            .0
            .replace(TILE, &second)
            .replace("25a005-61775_3850_25", "25a005-61775_3852_25");
        let both = body.replacen("],\"links\"", &format!(",{item}],\"links\""), 1);
        fetcher.insert_prefix(
            &format!("{STAC}/search?collections=dsm-skoglig-copc"),
            both.into_bytes(),
        );
        fetcher
    }

    fn fetch_answering(status: u16) -> (Vec<String>, usize) {
        let inner = two_tiles();
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let fetcher = Answering {
            inner: &inner,
            status,
        };
        let regions = Registry::new();
        let mut lines = Vec::new();
        let result = {
            let mut progress = |note: freemapper_adapter_api::note::Note| {
                if note.id() == "warning-point-cloud-unavailable-skipped" {
                    for part in ["Laserdata Skog", "download limit"] {
                        if note.contains(part) {
                            lines.push(part.to_string());
                        }
                    }
                }
            };
            let mut env = AdapterEnv {
                fetcher: &fetcher,
                regions: &regions,
                cache: &cache,
                mode: FetchMode::Online,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            backend().fetch(&mut env, &area())
        };
        assert!(result.unwrap().is_empty());
        let asked = inner
            .log
            .borrow()
            .iter()
            .filter(|u| u.ends_with(".laz"))
            .count();
        (lines, asked)
    }

    #[test]
    fn a_refused_tile_names_the_order_and_stops_the_asking() {
        let (lines, asked) = fetch_answering(401);
        assert_eq!(asked, 1, "the second tile would be refused the same way");
        assert!(lines.iter().any(|l| l == "Laserdata Skog"), "{lines:?}");
    }

    #[test]
    fn the_download_limit_is_named_and_stops_the_asking() {
        let (lines, asked) = fetch_answering(429);
        assert_eq!(asked, 1);
        assert!(lines.iter().any(|l| l == "download limit"), "{lines:?}");
    }

    fn record_of_the_tile() -> freemapper_adapter_api::pointcloud::PointCloudRecord {
        freemapper_adapter_api::pointcloud::PointCloudRecord {
            tile: "25a005-61775_3850_25".to_string(),
            url: TILE.to_string(),
            published: None,
        }
    }

    #[test]
    fn the_estimate_lists_each_2_5_km_square_of_the_area_at_80_megabytes() {
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        // Two squares wide: 382 500 to 387 500 is 385 000 ± 2 500.
        let wide = BoundingBox {
            min_e: 384_900.0,
            min_n: 6_177_500.0,
            max_e: 385_100.0,
            max_n: 6_177_700.0,
        };
        let items = backend().estimate(
            &EstimateArea {
                bbox_25833: wide,
                map_bbox: wide,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(items.len(), 2);
        for item in &items {
            assert_eq!((item.part, item.store), (Part::PointCloud, Store::Cache));
            assert!((item.megabytes - 80.0).abs() < 5.0, "{}", item.megabytes);
        }
    }
}
