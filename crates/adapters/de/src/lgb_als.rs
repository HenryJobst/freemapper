//! The classified laser point clouds of LGB Brandenburg (ALS, dl-de/by-2-0): one zip per km tile
//! with the LAZ file, its currency in a CSV like that of the DGM (ADR 0105).

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::NaiveDate;
use freemapper_adapter_api::cache::{CacheDir, STALE_PART_AGE};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::geometry::provider_tiles_in;
use freemapper_adapter_api::pointcloud::{
    PointCloudBackend, PointCloudInfo, PointCloudRecord, PointCloudTile,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode, write_cache_file};
use crate::las_files::{cached_las, check_las_file, las_path};
use crate::lgb::{TileCurrency, TileProvider, parse_currency_csv};
use crate::note::{Arg, note};
use crate::states::State;
use crate::tiles::{TileId, tiles_covering_grid};

const ALS_URL: &str = "https://data.geobasis-bb.de/geobasis/daten/als/laz";
pub const ALS_CURRENCY_CSV: &str = "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_laserscandaten_aktualitaet.csv";
/// The folder of the point clouds below the LGB folder of the cache.
const ALS_DIR: &str = "als";
/// The LAZ file of one tile, and its zip, has 135 to 165 MB (Potsdam, 2026); the estimate counts
/// this much per tile.
pub const ALS_MB_PER_TILE: f64 = 160.0;
/// A tile zip larger than this is no tile of LGB.
const MAX_ZIP_BYTES: u64 = 2 << 30;
/// The LAZ file of a tile hardly shrinks in the zip: more than this unpacked is no tile of LGB
/// (a broken or hostile archive).
const MAX_LAS_BYTES: u64 = 4 << 30;

/// URL of the zip of tile `t`.
pub fn als_tile_url(t: TileId) -> String {
    format!("{ALS_URL}/als_{}.zip", t.name())
}

/// The point-cloud backend of `state`: LGB for Brandenburg; the other states have none yet.
pub fn point_cloud(state: State) -> Option<&'static dyn PointCloudBackend> {
    static LGB: LazyLock<LgbAlsBackend> = LazyLock::new(LgbAlsBackend::new);
    match state {
        State::Brandenburg => Some(&*LGB),
        _ => None,
    }
}

/// The point clouds of LGB: 1 km tiles in EPSG:25833, kept in the cache as LAZ.
pub struct LgbAlsBackend {
    info: PointCloudInfo,
}

impl LgbAlsBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in(TileProvider::Lgb.id()),
                cache_dir: TileProvider::Lgb.id().to_string(),
                source: "lgb-als".to_string(),
                epsg: 25833,
                grid_km: 1,
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(ALS_DIR)
    }
}

impl Default for LgbAlsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PointCloudBackend for LgbAlsBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let tiles = provider_tiles_in(env.regions, bbox, &self.info.cache_dir, 1);
        let csv_path = dir.join("currency.csv");
        let csv = match env.mode {
            FetchMode::Online => {
                (env.progress)(note!("progress-downloading", url = ALS_CURRENCY_CSV));
                env.fetcher.get(ALS_CURRENCY_CSV)?
            }
            FetchMode::Offline => std::fs::read(&csv_path).unwrap_or_default(),
        };
        let table = if csv.is_empty() {
            Default::default()
        } else {
            parse_currency_csv(&String::from_utf8_lossy(&csv)).map_err(FetchError::from)?
        };
        if env.mode == FetchMode::Online {
            write_cache_file(&csv_path, &csv)?;
        }
        let mut fetched = Vec::new();
        for tile in tiles {
            let cached = cached_las(&dir, &tile.name());
            let url = als_tile_url(tile);
            let found = |(published, path): (NaiveDate, PathBuf)| PointCloudTile {
                record: PointCloudRecord {
                    tile: tile.name(),
                    url: url.clone(),
                    published: Some(published),
                },
                path,
            };
            let currency: Option<&TileCurrency> = table.get(&tile);
            if env.mode == FetchMode::Offline {
                match cached.into_iter().next() {
                    Some(newest) => fetched.push(found(newest)),
                    None => {
                        (env.progress)(note!("warning-point-cloud-no-cached", tile = tile.name()))
                    }
                }
                continue;
            }
            let Some(currency) = currency else {
                (env.progress)(note!(
                    "progress-point-cloud-not-published",
                    tile = tile.name()
                ));
                continue;
            };
            let path = las_path(&dir, &tile.name(), currency.published);
            if path.exists() {
                (env.progress)(note!("progress-point-cloud-cached", tile = tile.name()));
                fetched.push(found((currency.published, path)));
                continue;
            }
            (env.progress)(note!("progress-downloading", url = &url));
            match download_tile(env, &url, tile, &path) {
                Ok(()) => {
                    for (_, older) in cached.into_iter().filter(|(_, p)| *p != path) {
                        let _ = std::fs::remove_file(older);
                    }
                    fetched.push(found((currency.published, path)));
                }
                // The vegetation is only a candidate: a failing tile falls back to the cache or
                // is left out.
                Err(e) => match cached.into_iter().next() {
                    Some(older) => {
                        (env.progress)(note!(
                            "warning-point-cloud-unavailable-cached",
                            tile = tile.name(),
                            reason = Arg::error(e)
                        ));
                        fetched.push(found(older));
                    }
                    None => (env.progress)(note!(
                        "warning-point-cloud-unavailable-skipped",
                        tile = tile.name(),
                        reason = Arg::error(e)
                    )),
                },
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
        tiles_covering_grid(&area.bbox_25833, 1)
            .into_iter()
            .map(|tile| EstimateItem {
                what: format!("LGB point cloud {}", tile.name()),
                megabytes: ALS_MB_PER_TILE,
                cached: !cached_las(&dir, &tile.name()).is_empty(),
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

/// Streams the zip of `tile` next to `path`, unpacks its LAZ file to `path` and checks that it
/// is complete. The zip is removed in any case; a broken LAZ file is not left behind.
fn download_tile(
    env: &mut AdapterEnv<'_>,
    url: &str,
    tile: TileId,
    path: &Path,
) -> Result<(), FetchError> {
    let zip_path = path.with_extension("zip.part");
    let las_part = path.with_extension("laz.part");
    let _ = std::fs::remove_file(&zip_path);
    let result = env
        .fetcher
        .download_to(url, &zip_path, MAX_ZIP_BYTES, env.progress)
        .and_then(|()| {
            unpack_las(
                url,
                &zip_path,
                &format!("als_{}.laz", tile.name()),
                &las_part,
                MAX_LAS_BYTES,
            )
        })
        .and_then(|()| check_las_file(url, &las_part))
        .and_then(|()| {
            std::fs::rename(&las_part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
        });
    let _ = std::fs::remove_file(&zip_path);
    if result.is_err() {
        let _ = std::fs::remove_file(&las_part);
    }
    result
}

/// Copies `member` of the zip at `zip_path` to `to`, at most `max_bytes`; reading it to the end
/// checks its CRC. The limit holds for what comes out, whatever size the zip claims.
fn unpack_las(
    url: &str,
    zip_path: &Path,
    member: &str,
    to: &Path,
    max_bytes: u64,
) -> Result<(), FetchError> {
    use std::io::Read as _;
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message,
    };
    let file = File::open(zip_path).map_err(|e| FetchError::Io(zip_path.to_path_buf(), e))?;
    let mut archive =
        zip::ZipArchive::new(BufReader::new(file)).map_err(|e| corrupt(e.to_string()))?;
    let mut entry = archive
        .by_name(member)
        .map_err(|e| corrupt(format!("{member}: {e}")))?;
    let too_large = || FetchError::TooLarge {
        url: url.to_string(),
        limit_mb: max_bytes >> 20,
    };
    if entry.size() > max_bytes {
        return Err(too_large());
    }
    let _ = std::fs::remove_file(to);
    let mut out =
        freemapper_fs::create_new_file(to).map_err(|e| FetchError::Io(to.to_path_buf(), e))?;
    let copied = std::io::copy(&mut (&mut entry).take(max_bytes + 1), &mut out)
        .map_err(|e| corrupt(format!("{member}: {e}")))?;
    if copied > max_bytes {
        return Err(too_large());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use freemapper_adapter_api::registry::Registry;

    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const TILE: &str = "33421-5821";
    /// A box inside the tile, near Strausberg.
    const BBOX: BoundingBox = BoundingBox {
        min_e: 421_600.0,
        min_n: 5_821_600.0,
        max_e: 421_700.0,
        max_n: 5_821_700.0,
    };

    fn registry() -> Registry {
        let mut registry = Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        registry
    }

    /// A small LAZ point cloud of `points` points.
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
            x: transform(400_000.0),
            y: transform(5_800_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..points {
            writer
                .write_point(las::Point {
                    x: 421_610.0,
                    y: 5_821_650.0 + f64::from(i) * 0.2,
                    z: 40.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }

    /// The zip LGB serves: the LAZ file and its metadata.
    fn tile_zip(laz: &[u8]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file(format!("als_{TILE}.laz"), options).unwrap();
        zip.write_all(laz).unwrap();
        zip.start_file(format!("als_{TILE}.xml"), options).unwrap();
        zip.write_all(b"<metadata/>").unwrap();
        zip.finish().unwrap().into_inner()
    }

    fn csv(rows: &[&str]) -> Vec<u8> {
        let mut text =
            "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n"
                .to_string();
        for row in rows {
            text.push_str(row);
            text.push('\n');
        }
        text.into_bytes()
    }

    fn serve(fetcher: &MapFetcher, zip: Vec<u8>) {
        fetcher.insert(
            ALS_CURRENCY_CSV,
            csv(&[&format!("{TILE};;2020-12-30;;2021-11-29;")]),
        );
        fetcher.insert(&als_tile_url(TileId::parse(TILE).unwrap()), zip);
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
        let tiles = LgbAlsBackend::new().fetch(&mut env, &BBOX).unwrap();
        (tiles, notes)
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("lgb/als"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_tile_is_downloaded_unpacked_and_recorded_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, tile_zip(&laz(50)));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        let tile = &tiles[0];
        assert_eq!(
            tile.record,
            PointCloudRecord {
                tile: TILE.to_string(),
                url: format!("{ALS_URL}/als_{TILE}.zip"),
                published: Some("2021-11-29".parse().unwrap()),
            }
        );
        assert_eq!(
            las::Reader::from_path(&tile.path)
                .unwrap()
                .header()
                .number_of_points(),
            50
        );
        // Only the LAZ file and the currency list stay; the zip is gone.
        assert_eq!(
            folder(&cache),
            ["33421-5821_2021-11-29.laz", "currency.csv"]
        );
        assert_eq!(
            LgbAlsBackend::new().cached(&cache, &tile.record),
            Some(tile.path.clone())
        );
        // A second fetch finds the tile in the cache.
        let downloads = |f: &MapFetcher| {
            f.log
                .borrow()
                .iter()
                .filter(|u| u.ends_with(".zip"))
                .count()
        };
        let (again, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(again, tiles);
        assert_eq!(downloads(&fetcher), 1);
        assert!(
            notes.iter().any(|n| n == "cached point cloud 33421-5821"),
            "{notes:?}"
        );
    }

    #[test]
    fn a_newer_publication_replaces_the_cached_one() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let older = cache.root().join("lgb/als/33421-5821_2019-05-01.laz");
        std::fs::create_dir_all(older.parent().unwrap()).unwrap();
        std::fs::write(&older, laz(5)).unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher, tile_zip(&laz(50)));
        fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(
            folder(&cache),
            ["33421-5821_2021-11-29.laz", "currency.csv"]
        );
    }

    #[test]
    fn a_tile_without_a_row_is_not_published() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(
            ALS_CURRENCY_CSV,
            csv(&["33500-5900;;2020-12-30;;2021-11-29;"]),
        );
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(tiles.is_empty());
        assert!(
            notes.contains(&"no laser point cloud published for tile 33421-5821".to_string()),
            "{notes:?}"
        );
    }

    #[test]
    fn a_truncated_point_cloud_is_skipped_and_leaves_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let mut cut = laz(5_000);
        cut.truncate(cut.len() * 2 / 3);
        serve(&fetcher, tile_zip(&cut));
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: point cloud 33421-5821 unavailable")),
            "{notes:?}"
        );
        assert_eq!(folder(&cache), ["currency.csv"]);
    }

    #[test]
    fn unpacking_stops_at_the_limit() {
        let dir = tempfile::tempdir().unwrap();
        // 3 MB of zeros shrink to a few KB in the zip.
        let zip_path = dir.path().join("t.zip");
        std::fs::write(&zip_path, tile_zip(&vec![0; 3 << 20])).unwrap();
        let to = dir.path().join("t.laz");
        let member = format!("als_{TILE}.laz");
        let err = unpack_las("u", &zip_path, &member, &to, 1 << 20).unwrap_err();
        assert!(
            matches!(err, FetchError::TooLarge { limit_mb: 1, .. }),
            "{err}"
        );
        unpack_las("u", &zip_path, &member, &to, 4 << 20).unwrap();
        assert_eq!(std::fs::metadata(&to).unwrap().len(), 3 << 20);
    }

    #[test]
    fn a_failing_download_keeps_the_cached_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let older = cache.root().join("lgb/als/33421-5821_2019-05-01.laz");
        std::fs::create_dir_all(older.parent().unwrap()).unwrap();
        std::fs::write(&older, laz(5)).unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher, tile_zip(&laz(50)));
        fetcher
            .failing
            .borrow_mut()
            .push(als_tile_url(TileId::parse(TILE).unwrap()));
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].path, older);
        assert_eq!(
            tiles[0].record.published,
            Some("2019-05-01".parse().unwrap())
        );
    }

    #[test]
    fn offline_takes_the_newest_cached_tile_or_warns() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Offline);
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: no cached point cloud for tile 33421-5821")),
            "{notes:?}"
        );
        serve(&fetcher, tile_zip(&laz(50)));
        let (online, _) = fetch(&fetcher, &cache, FetchMode::Online);
        let calls = fetcher.calls.get();
        let (offline, _) = fetch(&fetcher, &cache, FetchMode::Offline);
        assert_eq!(offline, online);
        assert_eq!(fetcher.calls.get(), calls);
    }

    #[test]
    fn the_estimate_counts_every_tile_and_knows_the_cached_ones() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let cached = cache.root().join("lgb/als/33421-5821_2021-11-29.laz");
        std::fs::create_dir_all(cached.parent().unwrap()).unwrap();
        std::fs::write(&cached, b"").unwrap();
        // Across the border of two tiles.
        let bbox = BoundingBox {
            max_e: 422_100.0,
            ..BBOX
        };
        let area = EstimateArea {
            bbox_25833: bbox,
            map_bbox: bbox,
            today: "2026-11-12".parse().unwrap(),
        };
        let items = LgbAlsBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let summary: Vec<(&str, f64, bool, Part, Store)> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes, i.cached, i.part, i.store))
            .collect();
        assert_eq!(
            summary,
            [
                (
                    "LGB point cloud 33421-5821",
                    ALS_MB_PER_TILE,
                    true,
                    Part::PointCloud,
                    Store::Cache
                ),
                (
                    "LGB point cloud 33422-5821",
                    ALS_MB_PER_TILE,
                    false,
                    Part::PointCloud,
                    Store::Cache
                ),
            ]
        );
    }

    #[test]
    fn brandenburg_and_north_rhine_westphalia_have_point_clouds() {
        for state in State::ALL {
            let adapter = crate::states::StateAdapter::new(state);
            let backend = freemapper_adapter_api::adapter::RegionAdapter::point_cloud(&adapter);
            assert_eq!(
                backend.map(|b| b.info().epsg),
                match state {
                    State::Brandenburg => Some(25833),
                    State::NordrheinWestfalen => Some(25832),
                    _ => None,
                },
                "{state:?}"
            );
        }
    }

    #[test]
    #[ignore = "network"]
    fn live_currency_list_names_the_tile_and_the_zip_is_there() {
        use crate::fetch::{Fetcher as _, HttpFetcher};
        let fetcher = HttpFetcher::new();
        let csv = fetcher.get(ALS_CURRENCY_CSV).unwrap();
        let table = parse_currency_csv(&String::from_utf8_lossy(&csv)).unwrap();
        let tile = TileId::parse(TILE).unwrap();
        assert!(table.contains_key(&tile));
        fetcher.last_modified(&als_tile_url(tile)).unwrap();
    }
}
