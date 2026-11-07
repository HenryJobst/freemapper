//! Tile download with a shared on-disk cache keyed by tile and publication date.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::NaiveDate;

use crate::sources::alkis::AlkisError;
use crate::sources::lgb::{CurrencyCsvError, Product, TileCurrency, parse_currency_csv};
use crate::tiles::TileId;

/// Largest download accepted; LGB DOP20 tiles are about 8 MB.
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("download of {url} failed: {message}")]
    Http { url: String, message: String },
    #[error("{path}: {source}", path = .0.display(), source = .1)]
    Io(PathBuf, std::io::Error),
    #[error("download of {url} is not a complete tile archive: {message}")]
    CorruptArchive { url: String, message: String },
    #[error("tiles {} are not covered by LGB Brandenburg data", tile_names(.0))]
    TilesNotCovered(Vec<TileId>),
    #[error("offline and not cached: {}", tile_names(.0))]
    MissingOffline(Vec<TileId>),
    #[error(transparent)]
    CurrencyCsv(#[from] CurrencyCsvError),
    #[error(transparent)]
    Alkis(#[from] AlkisError),
}

fn tile_names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(TileId::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Source of downloads; HTTP in production, in-memory in tests.
pub trait Fetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

/// Time to open a connection and to wait for the response headers.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);
/// Time to receive a whole body; an 8 MB tile at 128 kbit/s takes about 9 minutes.
const BODY_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Downloads over HTTPS; a stalled server fails with a timeout instead of hanging.
pub struct HttpFetcher {
    agent: ureq::Agent,
}

impl HttpFetcher {
    pub fn new() -> Self {
        Self::with_timeouts(CONNECT_TIMEOUT, RESPONSE_TIMEOUT, BODY_TIMEOUT)
    }

    /// The same timeout for connecting, the response headers and the body.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_timeouts(timeout, timeout, timeout)
    }

    fn with_timeouts(connect: Duration, response: Duration, body: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(connect))
            .timeout_recv_response(Some(response))
            .timeout_recv_body(Some(body))
            .build();
        Self {
            agent: config.into(),
        }
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for HttpFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let http_error = |e: ureq::Error| FetchError::Http {
            url: url.to_string(),
            message: e.to_string(),
        };
        let mut response = self
            .agent
            .get(url)
            .header(
                "User-Agent",
                concat!("freemapper/", env!("CARGO_PKG_VERSION")),
            )
            .call()
            .map_err(http_error)?;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_DOWNLOAD_BYTES)
            .read_to_end(&mut bytes)
            .map_err(|e| FetchError::Http {
                url: url.to_string(),
                message: e.to_string(),
            })?;
        Ok(bytes)
    }
}

/// Tile cache shared by all projects.
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// `<system cache dir>/freemapper`, if the system has a cache directory.
    pub fn default_location() -> Option<Self> {
        dirs::cache_dir().map(|dir| Self::new(dir.join("freemapper")))
    }

    fn product_dir(&self, product: Product) -> PathBuf {
        self.root.join("lgb").join(product.cache_dir_name())
    }

    pub fn tile_path(&self, product: Product, tile: TileId, published: NaiveDate) -> PathBuf {
        self.product_dir(product)
            .join(format!("{}_{published}.zip", tile.name()))
    }

    pub fn currency_csv_path(&self, product: Product) -> PathBuf {
        self.product_dir(product).join("currency.csv")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    Online,
    /// Use the cache only.
    Offline,
}

/// A tile archive available in the cache.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedTile {
    pub product: Product,
    pub tile: TileId,
    pub url: String,
    pub currency: TileCurrency,
    pub zip: PathBuf,
}

/// Makes `tiles` of `product` available in `cache` and returns them in the given order.
///
/// Online, the currency CSV is downloaded on every call and a tile is downloaded only when the
/// cache lacks its current publication. Offline, the cached CSV and archives are used.
pub fn fetch_tiles(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<FetchedTile>, FetchError> {
    let csv_path = cache.currency_csv_path(product);
    let csv = match mode {
        FetchMode::Online => {
            let url = product.currency_csv_url();
            progress(&format!("downloading {url}"));
            fetcher.get(url)?
        }
        FetchMode::Offline => match std::fs::read(&csv_path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(FetchError::MissingOffline(tiles.to_vec())),
        },
    };
    let table = parse_currency_csv(&String::from_utf8_lossy(&csv))?;
    if mode == FetchMode::Online {
        // Only a CSV that parsed replaces the cached one, so `--offline` keeps working.
        write_atomically(&csv_path, &csv)?;
    }

    let uncovered: Vec<TileId> = tiles
        .iter()
        .copied()
        .filter(|tile| !table.contains_key(tile))
        .collect();
    if !uncovered.is_empty() {
        return Err(FetchError::TilesNotCovered(uncovered));
    }

    let mut fetched = Vec::new();
    let mut missing = Vec::new();
    for &tile in tiles {
        let mut currency = table[&tile];
        let mut zip = cache.tile_path(product, tile, currency.published);
        let url = product.tile_url(tile);
        if zip.exists() {
            progress(&format!("cached {}", tile.name()));
        } else if mode == FetchMode::Offline {
            match newest_cached_version(&zip, tile) {
                Some((published, older)) => {
                    progress(&format!(
                        "warning: using cached {} from {published} (newer {} not downloaded)",
                        tile.name(),
                        currency.published
                    ));
                    currency.published = published;
                    zip = older;
                }
                None => {
                    missing.push(tile);
                    continue;
                }
            }
        } else {
            progress(&format!("downloading {url}"));
            let bytes = fetcher.get(&url)?;
            check_archive(&bytes, product, tile).map_err(|message| FetchError::CorruptArchive {
                url: url.clone(),
                message,
            })?;
            write_atomically(&zip, &bytes)?;
            remove_older_versions(&zip, tile)?;
        }
        fetched.push(FetchedTile {
            product,
            tile,
            url,
            currency,
            zip,
        });
    }
    if !missing.is_empty() {
        return Err(FetchError::MissingOffline(missing));
    }
    Ok(fetched)
}

/// Checks that `bytes` is a zip archive with the tile's raster and world file, reading both
/// members to the end so their checksums are verified.
fn check_archive(bytes: &[u8], product: Product, tile: TileId) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for member in [product.raster_member(tile), product.world_file_member(tile)] {
        let mut entry = archive
            .by_name(&member)
            .map_err(|e| format!("{member}: {e}"))?;
        std::io::copy(&mut entry, &mut std::io::sink()).map_err(|e| format!("{member}: {e}"))?;
    }
    Ok(())
}

/// Cached publications of `tile` in the folder of `path`, newest first.
fn cached_versions(path: &Path, tile: TileId) -> Vec<(NaiveDate, PathBuf)> {
    let dir = path.parent().expect("cache paths have a parent");
    let prefix = format!("{}_", tile.name());
    let mut versions: Vec<(NaiveDate, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(&prefix)?
                .strip_suffix(".zip")?
                .parse()
                .ok()?;
            Some((date, entry.path()))
        })
        .collect();
    versions.sort_by_key(|v| std::cmp::Reverse(v.0));
    versions
}

fn newest_cached_version(path: &Path, tile: TileId) -> Option<(NaiveDate, PathBuf)> {
    cached_versions(path, tile).into_iter().next()
}

/// Writes `bytes` to `<path>.part` and renames it, so `path` never holds a partial file.
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    let dir = path.parent().expect("cache paths have a parent");
    std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    // The process id keeps concurrent runs from writing the same partial file.
    let mut part = path.as_os_str().to_owned();
    part.push(format!(".{}.part", std::process::id()));
    let part = PathBuf::from(part);
    std::fs::write(&part, bytes).map_err(|e| FetchError::Io(part.clone(), e))?;
    std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
}

/// Deletes the other cached publications of `tile` next to `current`, and partial downloads of
/// it left by interrupted runs.
fn remove_older_versions(current: &Path, tile: TileId) -> Result<(), FetchError> {
    let dir = current.parent().expect("cache paths have a parent");
    let prefix = format!("{}_", tile.name());
    let entries = std::fs::read_dir(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let stale = name.ends_with(".zip") || name.ends_with(".part");
        if path != current && name.starts_with(&prefix) && stale {
            std::fs::remove_file(&path).map_err(|e| FetchError::Io(path.clone(), e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::io::Write as _;

    use super::{FetchError, Fetcher};
    use crate::sources::alkis::BUILDINGS_ENDPOINT;
    use crate::sources::alkis::test_support::{building_feature, page};
    use crate::sources::lgb::Product;
    use crate::tiles::TileId;

    /// Serves fixed responses by URL and counts the requests.
    #[derive(Default)]
    pub(crate) struct MapFetcher {
        pub responses: RefCell<HashMap<String, Vec<u8>>>,
        /// Answers for URLs starting with the key, when no exact response matches.
        pub prefixes: RefCell<HashMap<String, Vec<u8>>>,
        pub calls: Cell<usize>,
    }

    impl MapFetcher {
        pub fn insert(&self, url: &str, bytes: Vec<u8>) {
            self.responses.borrow_mut().insert(url.to_string(), bytes);
        }

        pub fn insert_prefix(&self, prefix: &str, bytes: Vec<u8>) {
            self.prefixes.borrow_mut().insert(prefix.to_string(), bytes);
        }
    }

    impl Fetcher for MapFetcher {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            let by_prefix = || {
                let prefixes = self.prefixes.borrow();
                prefixes
                    .iter()
                    .filter(|(prefix, _)| url.starts_with(prefix.as_str()))
                    .max_by_key(|(prefix, _)| prefix.len())
                    .map(|(_, bytes)| bytes.clone())
            };
            self.responses
                .borrow()
                .get(url)
                .cloned()
                .or_else(by_prefix)
                .ok_or_else(|| FetchError::Http {
                    url: url.to_string(),
                    message: "404 Not Found".to_string(),
                })
        }
    }

    /// A tile archive with the raster and world file members LGB uses.
    pub fn tile_zip(product: Product, tile: TileId, raster: &[u8], world_file: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file(product.raster_member(tile), options)
            .unwrap();
        zip.write_all(raster).unwrap();
        zip.start_file(product.world_file_member(tile), options)
            .unwrap();
        zip.write_all(world_file.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

    /// A uniformly red JPEG.
    pub fn red_jpeg(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([255, 0, 0]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95)
            .encode_image(&img)
            .unwrap();
        bytes
    }

    /// A 32-bit float LZW GeoTIFF body like DGM1 (without GeoKeys; the world file locates it).
    pub fn dtm_tiff(width: u32, height: u32, values: &[f32]) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        tiff::encoder::TiffEncoder::new(&mut bytes)
            .unwrap()
            .with_compression(tiff::encoder::Compression::Lzw)
            .write_image::<tiff::encoder::colortype::Gray32Float>(width, height, values)
            .unwrap();
        bytes.into_inner()
    }

    /// Serves the currency CSVs and coarse synthetic orthophoto and DTM archives (100 × 100
    /// pixels of 10 m) for `tile`, published on `published`. The DTM rises 1 m per 10 m east.
    pub fn serve_synthetic_tile(fetcher: &MapFetcher, tile: TileId, published: &str) {
        let bounds = tile.bounds();
        let world = format!(
            "10\n0\n0\n-10\n{}\n{}\n",
            bounds.min_e + 5.0,
            bounds.max_n - 5.0
        );
        let heights: Vec<f32> = (0..100 * 100).map(|i| (i % 100) as f32).collect();
        for (product, raster) in [
            (Product::Orthophoto, red_jpeg(100, 100)),
            (Product::Dtm, dtm_tiff(100, 100, &heights)),
        ] {
            fetcher.insert(
                product.currency_csv_url(),
                currency_csv(&[(tile, published)]),
            );
            fetcher.insert(
                &product.tile_url(tile),
                tile_zip(product, tile, &raster, &world),
            );
        }
        let school = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Allgemein bildende Schule",
            SCHOOL_BUILDING,
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[school], None));
    }

    /// A 10 m square near the centroid of the fixture school area (EPSG:25833 corner + side).
    pub const SCHOOL_BUILDING: [f64; 3] = [421_605.0, 5_821_730.0, 10.0];

    /// A currency CSV with one row per tile and the given publication date.
    pub fn currency_csv(rows: &[(TileId, &str)]) -> Vec<u8> {
        let mut csv = String::from(
            "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n",
        );
        for (tile, published) in rows {
            csv.push_str(&format!("{};;2018-11-27;;{published};\n", tile.name()));
        }
        csv.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };
    const PRODUCT: Product = Product::Dtm;

    fn serve(fetcher: &MapFetcher, published: &str) {
        fetcher.insert(
            PRODUCT.currency_csv_url(),
            currency_csv(&[(TILE, published)]),
        );
        fetcher.insert(
            &PRODUCT.tile_url(TILE),
            tile_zip(
                PRODUCT,
                TILE,
                b"raster",
                "1\n0\n0\n-1\n421000.5\n5821999.5\n",
            ),
        );
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &Cache,
        tiles: &[TileId],
        mode: FetchMode,
    ) -> Result<Vec<FetchedTile>, FetchError> {
        fetch_tiles(fetcher, cache, PRODUCT, tiles, mode, &mut |_| {})
    }

    fn cached_files(cache: &Cache) -> Vec<String> {
        let dir = cache.currency_csv_path(PRODUCT);
        let mut names: Vec<String> = std::fs::read_dir(dir.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn fetch_downloads_then_reuses_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");

        let first = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(fetcher.calls.get(), 2);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].tile, TILE);
        assert_eq!(first[0].url, PRODUCT.tile_url(TILE));
        assert_eq!(first[0].currency.published.to_string(), "2026-01-26");
        assert!(first[0].zip.exists());

        let second = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            fetcher.calls.get(),
            3,
            "only the currency CSV is downloaded again"
        );
        assert_eq!(second[0].zip, first[0].zip);
    }

    #[test]
    fn new_publication_replaces_old_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        serve(&fetcher, "2026-06-01");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            cached_files(&cache),
            ["33421-5821_2026-06-01.zip", "currency.csv"]
        );
    }

    #[test]
    fn truncated_download_is_not_cached() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let url = PRODUCT.tile_url(TILE);
        let truncated = fetcher.responses.borrow()[&url][..100].to_vec();
        fetcher.insert(&url, truncated);

        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CorruptArchive { .. }), "{err:?}");
        assert_eq!(cached_files(&cache), ["currency.csv"]);
    }

    #[test]
    fn fetch_unknown_tile_names_it() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let outside = TileId {
            east_km: 999,
            north_km: 5999,
        };
        let err = fetch(&fetcher, &cache, &[outside], FetchMode::Online).unwrap_err();
        assert!(err.to_string().contains("33999-5999"), "{err}");
    }

    #[test]
    fn offline_without_cache_lists_missing_tiles() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let north = TileId {
            east_km: 421,
            north_km: 5822,
        };
        let csv = cache.currency_csv_path(PRODUCT);
        std::fs::create_dir_all(csv.parent().unwrap()).unwrap();
        std::fs::write(
            &csv,
            currency_csv(&[(TILE, "2026-01-26"), (north, "2026-01-26")]),
        )
        .unwrap();
        let fetcher = MapFetcher::default();

        let err = fetch(&fetcher, &cache, &[TILE, north], FetchMode::Offline).unwrap_err();
        assert_eq!(fetcher.calls.get(), 0);
        assert!(
            matches!(&err, FetchError::MissingOffline(tiles) if tiles.len() == 2),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("33421-5821") && message.contains("33421-5822"),
            "{message}"
        );
    }

    const NORTH: TileId = TileId {
        east_km: 421,
        north_km: 5822,
    };
    const EAST: TileId = TileId {
        east_km: 422,
        north_km: 5821,
    };

    #[test]
    fn all_uncovered_tiles_reported_before_downloading() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let err = fetch(&fetcher, &cache, &[TILE, NORTH, EAST], FetchMode::Online).unwrap_err();
        assert!(
            matches!(&err, FetchError::TilesNotCovered(t) if t == &[NORTH, EAST]),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("33421-5822") && message.contains("33422-5821"),
            "{message}"
        );
        assert_eq!(fetcher.calls.get(), 1, "only the currency CSV");
    }

    #[test]
    fn offline_uses_older_cached_publication() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        std::fs::write(
            cache.currency_csv_path(PRODUCT),
            currency_csv(&[(TILE, "2026-06-01")]),
        )
        .unwrap();

        let mut messages = Vec::new();
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            PRODUCT,
            &[TILE],
            FetchMode::Offline,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(fetched[0].currency.published.to_string(), "2026-01-26");
        assert_eq!(
            fetched[0].zip,
            cache.tile_path(PRODUCT, TILE, "2026-01-26".parse().unwrap())
        );
        assert!(
            messages.contains(&"warning: using cached 33421-5821 from 2026-01-26 (newer 2026-06-01 not downloaded)".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn corrupt_member_data_is_not_cached() {
        use std::io::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(PRODUCT.raster_member(TILE), stored).unwrap();
        zip.write_all(b"RASTERDATA-RASTERDATA").unwrap();
        zip.start_file(PRODUCT.world_file_member(TILE), stored)
            .unwrap();
        zip.write_all(b"1\n0\n0\n-1\n421000.5\n5821999.5\n")
            .unwrap();
        let mut bytes = zip.finish().unwrap().into_inner();
        let at = bytes.windows(10).position(|w| w == b"RASTERDATA").unwrap();
        bytes[at] = b'X';
        fetcher.insert(&PRODUCT.tile_url(TILE), bytes);

        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CorruptArchive { .. }), "{err:?}");
        assert_eq!(cached_files(&cache), ["currency.csv"]);
    }

    #[test]
    fn stale_part_files_are_removed() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let product_dir = cache
            .currency_csv_path(PRODUCT)
            .parent()
            .unwrap()
            .to_path_buf();
        std::fs::create_dir_all(&product_dir).unwrap();
        for stale in [
            "33421-5821_2026-01-26.zip.part",
            "33421-5821_2025-01-01.zip.999999.part",
        ] {
            std::fs::write(product_dir.join(stale), b"partial").unwrap();
        }
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            cached_files(&cache),
            ["33421-5821_2026-01-26.zip", "currency.csv"]
        );
    }

    #[test]
    fn offline_fetch_succeeds_from_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let online = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        let calls = fetcher.calls.get();
        let offline = fetch(&fetcher, &cache, &[TILE], FetchMode::Offline).unwrap();
        assert_eq!(offline, online);
        assert_eq!(fetcher.calls.get(), calls);
    }

    #[test]
    fn test_fetcher_prefers_exact_then_longest_prefix() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://a.example/", b"short".to_vec());
        fetcher.insert_prefix("https://a.example/items", b"long".to_vec());
        fetcher.insert("https://a.example/items?x=1", b"exact".to_vec());
        assert_eq!(
            fetcher.get("https://a.example/items?x=1").unwrap(),
            b"exact"
        );
        assert_eq!(fetcher.get("https://a.example/items?x=2").unwrap(), b"long");
        assert_eq!(fetcher.get("https://a.example/other").unwrap(), b"short");
        assert!(fetcher.get("https://b.example/").is_err());
    }

    #[test]
    fn invalid_csv_download_keeps_cached_csv() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();

        fetcher.insert(
            PRODUCT.currency_csv_url(),
            b"<html>captive portal</html>".to_vec(),
        );
        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CurrencyCsv(_)), "{err:?}");
        let offline = fetch(&fetcher, &cache, &[TILE], FetchMode::Offline).unwrap();
        assert_eq!(offline[0].currency.published.to_string(), "2026-01-26");
    }

    #[test]
    fn stalled_server_times_out() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/tile.zip", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            // Accept and hold connections without ever answering.
            let _held: Vec<_> = listener.incoming().take(4).collect();
            std::thread::sleep(std::time::Duration::from_secs(30));
        });
        let fetcher = HttpFetcher::with_timeout(std::time::Duration::from_millis(300));
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || sender.send(fetcher.get(&url)));
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("fetch did not time out");
        assert!(matches!(result, Err(FetchError::Http { .. })), "{result:?}");
    }

    #[test]
    #[ignore = "network"]
    fn live_school_tile_currency() {
        let text = HttpFetcher::new()
            .get(Product::Orthophoto.currency_csv_url())
            .unwrap();
        let table = parse_currency_csv(&String::from_utf8(text).unwrap()).unwrap();
        assert!(table.contains_key(&TILE));
    }
}
