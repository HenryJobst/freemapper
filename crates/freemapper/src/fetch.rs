//! Tile download with a shared on-disk cache keyed by tile and publication date.

use std::io::Read as _;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

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
    #[error("tile {} is not covered by LGB Brandenburg data", .0.name())]
    TileNotCovered(TileId),
    #[error("offline and not cached: {}", tile_names(.0))]
    MissingOffline(Vec<TileId>),
    #[error(transparent)]
    CurrencyCsv(#[from] CurrencyCsvError),
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

/// Downloads over HTTPS.
pub struct HttpFetcher;

impl Fetcher for HttpFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let http_error = |e: ureq::Error| FetchError::Http {
            url: url.to_string(),
            message: e.to_string(),
        };
        let mut response = ureq::get(url)
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

    let mut fetched = Vec::new();
    let mut missing = Vec::new();
    for &tile in tiles {
        let currency = *table.get(&tile).ok_or(FetchError::TileNotCovered(tile))?;
        let zip = cache.tile_path(product, tile, currency.published);
        let url = product.tile_url(tile);
        if zip.exists() {
            progress(&format!("cached {}", tile.name()));
        } else if mode == FetchMode::Offline {
            missing.push(tile);
            continue;
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

/// Checks that `bytes` is a zip archive with the tile's raster and world file.
fn check_archive(bytes: &[u8], product: Product, tile: TileId) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for member in [product.raster_member(tile), product.world_file_member(tile)] {
        archive
            .by_name(&member)
            .map_err(|e| format!("{member}: {e}"))?;
    }
    Ok(())
}

/// Writes `bytes` to `<path>.part` and renames it, so `path` never holds a partial file.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    let dir = path.parent().expect("cache paths have a parent");
    std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);
    std::fs::write(&part, bytes).map_err(|e| FetchError::Io(part.clone(), e))?;
    std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
}

/// Deletes the other cached publications of `tile` next to `current`.
fn remove_older_versions(current: &Path, tile: TileId) -> Result<(), FetchError> {
    let dir = current.parent().expect("cache paths have a parent");
    let prefix = format!("{}_", tile.name());
    let entries = std::fs::read_dir(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path != current && name.starts_with(&prefix) && name.ends_with(".zip") {
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
    use crate::sources::lgb::Product;
    use crate::tiles::TileId;

    /// Serves fixed responses by URL and counts the requests.
    #[derive(Default)]
    pub(crate) struct MapFetcher {
        pub responses: RefCell<HashMap<String, Vec<u8>>>,
        pub calls: Cell<usize>,
    }

    impl MapFetcher {
        pub fn insert(&self, url: &str, bytes: Vec<u8>) {
            self.responses.borrow_mut().insert(url.to_string(), bytes);
        }
    }

    impl Fetcher for MapFetcher {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            self.responses
                .borrow()
                .get(url)
                .cloned()
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
    }

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
    #[ignore = "network"]
    fn live_school_tile_currency() {
        let text = HttpFetcher
            .get(Product::Orthophoto.currency_csv_url())
            .unwrap();
        let table = parse_currency_csv(&String::from_utf8(text).unwrap()).unwrap();
        assert!(table.contains_key(&TILE));
    }
}
