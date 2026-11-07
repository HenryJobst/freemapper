//! Tile download with a shared on-disk cache keyed by tile and publication date.

use std::collections::HashMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::NaiveDate;

use crate::sources::alkis::AlkisError;
use crate::sources::lgb::ndsm_tile_url;
use crate::sources::lgb::{CurrencyCsvError, Product, TileCurrency, parse_currency_csv};
use crate::sources::region::Provider;
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
    #[error(
        "{} not covered by the published tiles; reduce the map area or buffer_m in freemapper.toml",
        tiles_are(.0)
    )]
    TilesNotCovered(Vec<TileId>),
    #[error(
        "offline and not cached: {product} {tiles}; run `freemapper fetch` online",
        product = .0,
        tiles = tile_list(.1)
    )]
    MissingOffline(Product, Vec<TileId>),
    #[error(transparent)]
    CurrencyCsv(#[from] CurrencyCsvError),
    #[error(transparent)]
    Alkis(#[from] AlkisError),
    #[error("no OSM extract cached; run `freemapper fetch` online")]
    OsmExtractMissing,
    #[error("{0} not found on the server")]
    NotFound(String),
    /// The server refused the request (HTTP 4xx other than 404); asking again does not help.
    #[error("{url}: the server refused the request (HTTP {status})")]
    Refused { url: String, status: u16 },
}

pub(crate) fn tile_names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(TileId::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `tile X` or `tiles X, Y`.
fn tile_list(tiles: &[TileId]) -> String {
    let noun = if tiles.len() == 1 { "tile" } else { "tiles" };
    format!("{noun} {}", tile_names(tiles))
}

/// `tile X is` or `tiles X, Y are`.
fn tiles_are(tiles: &[TileId]) -> String {
    let verb = if tiles.len() == 1 { "is" } else { "are" };
    format!("{} {verb}", tile_list(tiles))
}

/// Partial files older than this are left over from interrupted runs.
const STALE_PART_AGE: Duration = Duration::from_secs(3600);

/// Source of downloads; HTTP in production, in-memory in tests.
pub trait Fetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;

    /// The `Last-Modified` date of `url`, if the server sends one.
    fn last_modified(&self, _url: &str) -> Result<Option<NaiveDate>, FetchError> {
        Ok(None)
    }

    /// Bytes `range` of `url`, for reading single members of very large archives.
    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let bytes = self.get(url)?;
        let len = bytes.len() as u64;
        let (start, end) = match range {
            ByteRange::From { start, len: n } => (start.min(len), (start + n).min(len)),
            ByteRange::Last(n) => (len.saturating_sub(n), len),
        };
        Ok(bytes[start as usize..end as usize].to_vec())
    }

    /// Downloads `url` into the file `path`, reporting the size; large files stream instead of
    /// being held in memory.
    fn download_to(
        &self,
        url: &str,
        path: &Path,
        progress: &mut dyn FnMut(&str),
    ) -> Result<(), FetchError> {
        let bytes = self.get(url)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
        }
        std::fs::write(path, &bytes).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
        progress(&format!("downloaded {} MB", megabytes(bytes.len() as u64)));
        Ok(())
    }
}

/// A part of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// `len` bytes from `start`.
    From { start: u64, len: u64 },
    /// The last `n` bytes.
    Last(u64),
}

/// A ureq error as [`FetchError`]; 404 becomes [`FetchError::NotFound`].
fn status_error(url: &str, e: ureq::Error) -> FetchError {
    match e {
        ureq::Error::StatusCode(404) => FetchError::NotFound(url.to_string()),
        ureq::Error::StatusCode(status) if (400..500).contains(&status) => FetchError::Refused {
            url: url.to_string(),
            status,
        },
        e => FetchError::Http {
            url: url.to_string(),
            message: e.to_string(),
        },
    }
}

/// Identifies freemapper to the servers (Nominatim requires it).
const USER_AGENT: &str = concat!(
    "freemapper/",
    env!("CARGO_PKG_VERSION"),
    " (+",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

/// Time to open a connection and to wait for the response headers.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);
/// Time to receive a whole body; an 8 MB tile at 128 kbit/s takes about 9 minutes.
const BODY_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Progress is reported after each step of a large download.
const PROGRESS_STEP_BYTES: u64 = 25 * 1024 * 1024;

fn megabytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

/// Time to receive a large file such as the OSM extract (300 MB at 1 Mbit/s: 40 minutes).
const LARGE_BODY_TIMEOUT: Duration = Duration::from_secs(2 * 3600);

/// Downloads over HTTPS; a stalled server fails with a timeout instead of hanging.
pub struct HttpFetcher {
    agent: ureq::Agent,
    large: ureq::Agent,
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
        let agent = |body: Duration| -> ureq::Agent {
            ureq::Agent::config_builder()
                .timeout_connect(Some(connect))
                .timeout_recv_response(Some(response))
                .timeout_recv_body(Some(body))
                .build()
                .into()
        };
        Self {
            agent: agent(body),
            large: agent(LARGE_BODY_TIMEOUT.max(body)),
        }
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for HttpFetcher {
    /// Asks for `range` only; a server that answers with the whole file (not 206) is refused
    /// before its body is read, so a 1 GB archive is never downloaded by accident.
    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let header = match range {
            ByteRange::From { start, len } => format!("bytes={start}-{}", start + len.max(1) - 1),
            ByteRange::Last(n) => format!("bytes=-{n}"),
        };
        let mut response = self
            .agent
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Range", &header)
            .call()
            .map_err(|e| status_error(url, e))?;
        if response.status() != 206 {
            return Err(FetchError::Http {
                url: url.to_string(),
                message: format!(
                    "the server ignored the byte range {header} (HTTP {})",
                    response.status().as_u16()
                ),
            });
        }
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

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        progress: &mut dyn FnMut(&str),
    ) -> Result<(), FetchError> {
        let http_error = |message: String| FetchError::Http {
            url: url.to_string(),
            message,
        };
        let mut response = self
            .large
            .get(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| http_error(e.to_string()))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
        }
        let total = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        let mut file =
            std::fs::File::create(path).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
        let mut reader = response.body_mut().as_reader();
        let mut buffer = vec![0u8; 1 << 16];
        let (mut done, mut reported) = (0u64, 0u64);
        loop {
            let n = std::io::Read::read(&mut reader, &mut buffer)
                .map_err(|e| http_error(e.to_string()))?;
            if n == 0 {
                break;
            }
            std::io::Write::write_all(&mut file, &buffer[..n])
                .map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
            done += n as u64;
            if done - reported >= PROGRESS_STEP_BYTES {
                reported = done;
                let of = total
                    .map(|t| format!(" of {} MB", megabytes(t)))
                    .unwrap_or_default();
                progress(&format!("downloaded {} MB{of}", megabytes(done)));
            }
        }
        progress(&format!("downloaded {} MB", megabytes(done)));
        Ok(())
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        let response = self
            .agent
            .head(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| status_error(url, e))?;
        Ok(response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|time| time.date_naive()))
    }

    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let http_error = |e: ureq::Error| status_error(url, e);
        let mut response = self
            .agent
            .get(url)
            .header("User-Agent", USER_AGENT)
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
        self.root
            .join(product.provider_dir_name())
            .join(product.cache_dir_name())
    }

    pub fn tile_path(&self, product: Product, tile: TileId, published: NaiveDate) -> PathBuf {
        self.product_dir(product)
            .join(format!("{}_{published}.zip", tile.name()))
    }

    /// Folder of the cached nDSM point clouds.
    pub fn ndsm_dir(&self) -> PathBuf {
        self.root.join("lgb").join("ndom")
    }

    /// Cached nDSM file of `tile`, named after the server date (`<tile>.laz` when undated).
    pub fn ndsm_path(&self, tile: TileId, published: Option<NaiveDate>) -> PathBuf {
        self.ndsm_dir().join(match published {
            Some(date) => format!("{}_{date}.laz", tile.name()),
            None => format!("{}.laz", tile.name()),
        })
    }

    /// Whether some publication of `tile` of `product` is cached.
    pub fn has_tile(&self, product: Product, tile: TileId) -> bool {
        let probe = self.tile_path(product, tile, NaiveDate::MIN);
        !cached_versions(&probe, tile).is_empty()
    }

    /// Whether an nDSM point cloud of `tile` is cached.
    pub fn has_ndsm(&self, tile: TileId) -> bool {
        !cached_ndsm(&self.ndsm_dir(), tile).is_empty()
    }

    /// Folder of the cached OSM extracts.
    pub fn osm_dir(&self) -> PathBuf {
        self.root.join("osm")
    }

    pub fn currency_csv_path(&self, product: Product) -> PathBuf {
        self.product_dir(product).join("currency.csv")
    }

    /// Folder of the LGV Hamburg archive members.
    pub fn lgv_dir(&self) -> PathBuf {
        self.root.join("lgv")
    }

    /// The collected GeoSN tile index rows of `product`.
    pub fn currency_index_path(&self, product: Product) -> PathBuf {
        self.product_dir(product).join("currency.json")
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
    let table = currency_table(fetcher, cache, product, tiles, mode, progress)?;
    let uncovered: Vec<TileId> = tiles
        .iter()
        .copied()
        .filter(|tile| !table.contains_key(tile))
        .collect();
    if !uncovered.is_empty() {
        return Err(not_in_table(product, mode, uncovered));
    }
    let mut fetched = Vec::new();
    let mut missing = Vec::new();
    for &tile in tiles {
        match fetch_tile(
            fetcher,
            cache,
            product,
            tile,
            table[&tile].clone(),
            mode,
            progress,
        )? {
            Some(tile) => fetched.push(tile),
            None => missing.push(tile),
        }
    }
    if !missing.is_empty() {
        return Err(FetchError::MissingOffline(product, missing));
    }
    Ok(fetched)
}

/// One tile of [`fetch_tiles_each`] and how fetching it went.
pub type TileOutcome = (TileId, Result<FetchedTile, FetchError>);

/// Like [`fetch_tiles`], but every tile succeeds or fails on its own; only a currency CSV that
/// cannot be read fails the whole call.
pub fn fetch_tiles_each(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<TileOutcome>, FetchError> {
    let table = currency_table(fetcher, cache, product, tiles, mode, progress)?;
    Ok(tiles
        .iter()
        .map(|&tile| {
            let result = match table.get(&tile) {
                None => Err(not_in_table(product, mode, vec![tile])),
                Some(entry) => {
                    fetch_tile(fetcher, cache, product, tile, entry.clone(), mode, progress)
                        .and_then(|fetched| {
                            fetched.ok_or(FetchError::MissingOffline(product, vec![tile]))
                        })
                }
            };
            (tile, result)
        })
        .collect())
}

/// Tiles missing from the currency table: not published, or (GeoSN offline, whose cached index
/// only holds the areas asked for before) not fetched yet.
fn not_in_table(product: Product, mode: FetchMode, tiles: Vec<TileId>) -> FetchError {
    if mode == FetchMode::Offline && product.provider() == Provider::GeoSn {
        FetchError::MissingOffline(product, tiles)
    } else {
        FetchError::TilesNotCovered(tiles)
    }
}

/// Currency and download URL of each tile a provider publishes.
type CurrencyTable = HashMap<TileId, (TileCurrency, String)>;

/// The currency table of `product`, from its provider: the LGB currency CSV or the GeoSN tile
/// index. Online the answer replaces (CSV) or extends (index) the cached copy once it parses;
/// offline the cached copy is used.
fn currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    match product.provider() {
        Provider::Lgb => lgb_currency_table(fetcher, cache, product, tiles, mode, progress),
        Provider::GeoSn => geosn_currency_table(fetcher, cache, product, tiles, mode, progress),
        // LVermGeo rasters are fetched per area (WMS, WCS), not as tiles.
        Provider::LvermGeo | Provider::Lgv => Err(FetchError::TilesNotCovered(tiles.to_vec())),
    }
}

fn lgb_currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    let csv_path = cache.currency_csv_path(product);
    let csv = match mode {
        FetchMode::Online => {
            let url = product.currency_csv_url();
            progress(&format!("downloading {url}"));
            fetcher.get(url)?
        }
        FetchMode::Offline => match std::fs::read(&csv_path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(FetchError::MissingOffline(product, tiles.to_vec())),
        },
    };
    let table = parse_currency_csv(&String::from_utf8_lossy(&csv))?;
    if mode == FetchMode::Online {
        // Only a CSV that parsed replaces the cached one, so `--offline` keeps working.
        write_atomically(&csv_path, &csv)?;
    }
    Ok(table
        .into_iter()
        .map(|(tile, currency)| (tile, (currency, product.tile_url(tile))))
        .collect())
}

/// The GeoSN index rows of `tiles`. The index is queried per area, so the cached copy collects
/// the rows of every query (`{"33414-5654": ["2024-11-30", "https://…"], …}`).
fn geosn_currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    let path = cache.currency_index_path(product);
    let mut cached: std::collections::BTreeMap<String, (NaiveDate, String)> = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    if mode == FetchMode::Online {
        let url = crate::sources::geosn::index_url(product, tiles);
        progress(&format!("downloading the {product} tile index"));
        let rows = get_with_retries(fetcher, &url, |answer| {
            crate::sources::geosn::parse_index(&String::from_utf8_lossy(answer)).map_err(
                |message| FetchError::Http {
                    url: url.clone(),
                    message,
                },
            )
        })?;
        for row in rows {
            let download = row.url.unwrap_or_else(|| product.tile_url(row.tile));
            cached.insert(row.tile.name(), (row.currency, download));
        }
        let json = serde_json::to_vec(&cached).expect("index table serialises");
        write_atomically(&path, &json)?;
    } else if cached.is_empty() {
        return Err(FetchError::MissingOffline(product, tiles.to_vec()));
    }
    Ok(cached
        .into_iter()
        .filter_map(|(name, (date, url))| {
            let currency = TileCurrency {
                created: date,
                revised: None,
                published: date,
            };
            Some((TileId::parse(&name)?, (currency, url)))
        })
        .collect())
}

/// One tile of `product` in the cache; `None` offline when no version of it is cached.
fn fetch_tile(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tile: TileId,
    (mut currency, url): (TileCurrency, String),
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<FetchedTile>, FetchError> {
    let mut zip = cache.tile_path(product, tile, currency.published);
    if zip.exists() {
        progress(&format!("cached {}", tile.name()));
    } else if mode == FetchMode::Offline {
        match newest_cached_version(&zip, tile) {
            Some((published, older)) => {
                progress(&format!(
                    "warning: using cached {product} tile {} from {published} (newer {} not downloaded)",
                    tile.name(),
                    currency.published
                ));
                currency.published = published;
                zip = older;
            }
            None => return Ok(None),
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
        crate::fsutil::remove_stale_parts(zip.parent().expect("cache paths"), STALE_PART_AGE);
    }
    Ok(Some(FetchedTile {
        product,
        tile,
        url,
        currency,
        zip,
    }))
}

/// A cached nDSM point cloud of one tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NdsmTile {
    pub tile: TileId,
    pub url: String,
    /// The server's `Last-Modified` date, if it sends one.
    pub published: Option<NaiveDate>,
    pub path: PathBuf,
}

/// Makes the nDSM point clouds of `tiles` available in `cache`. A tile is downloaded when the
/// server's `Last-Modified` date has no cached file yet; tiles the server does not have are
/// skipped with a warning. Offline, the newest cached file per tile is used.
pub fn fetch_ndsm_tiles(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<NdsmTile>, FetchError> {
    let dir = cache.ndsm_dir();
    let mut fetched = Vec::new();
    for &tile in tiles {
        let url = ndsm_tile_url(tile);
        let cached = cached_ndsm(&dir, tile);
        if mode == FetchMode::Offline {
            match cached.into_iter().next() {
                Some((published, path)) => fetched.push(NdsmTile {
                    tile,
                    url,
                    published,
                    path,
                }),
                None => progress(&format!("warning: no cached nDSM for tile {}", tile.name())),
            }
            continue;
        }
        // The nDSM is only a template: a failing server falls back to the cache or skips the tile.
        let unavailable = |reason: String, progress: &mut dyn FnMut(&str)| {
            let fallback = cached.first().cloned();
            progress(&format!(
                "warning: nDSM tile {} unavailable ({reason}); {}",
                tile.name(),
                if fallback.is_some() {
                    "using the cached file"
                } else {
                    "skipped"
                }
            ));
            fallback
        };
        let published = match fetcher.last_modified(&url) {
            Ok(date) => date,
            Err(FetchError::NotFound(_)) => {
                progress(&format!(
                    "no nDSM point cloud published for tile {}",
                    tile.name()
                ));
                continue;
            }
            Err(e) => {
                if let Some((published, path)) = unavailable(e.to_string(), progress) {
                    fetched.push(NdsmTile {
                        tile,
                        url,
                        published,
                        path,
                    });
                }
                continue;
            }
        };
        let path = cache.ndsm_path(tile, published);
        if path.exists() {
            progress(&format!("cached nDSM {}", tile.name()));
        } else {
            progress(&format!("downloading {url}"));
            let bytes = match fetcher.get(&url).and_then(|b| check_las(&url, b)) {
                Ok(bytes) => bytes,
                Err(e) => {
                    if let Some((published, path)) = unavailable(e.to_string(), progress) {
                        fetched.push(NdsmTile {
                            tile,
                            url,
                            published,
                            path,
                        });
                    }
                    continue;
                }
            };
            write_atomically(&path, &bytes)?;
            for (_, older) in cached_ndsm(&dir, tile)
                .into_iter()
                .filter(|(_, p)| *p != path)
            {
                let _ = std::fs::remove_file(older);
            }
        }
        fetched.push(NdsmTile {
            tile,
            url,
            published,
            path,
        });
    }
    Ok(fetched)
}

/// Checks that `bytes` is a complete LAS/LAZ point cloud: the header, the LAZ chunk table at the
/// end of the file and the point count all have to be readable.
fn check_las(url: &str, bytes: Vec<u8>) -> Result<Vec<u8>, FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a complete LAS point cloud: {message}"),
    };
    {
        let mut reader = las::Reader::new(std::io::Cursor::new(bytes.clone()))
            .map_err(|e| corrupt(e.to_string()))?;
        let total = reader.header().number_of_points();
        // The last chunk ends the file: reading the final point fails on a truncated download.
        if total > 0 {
            reader.seek(total - 1).map_err(|e| corrupt(e.to_string()))?;
            reader.read_points(1).map_err(|e| corrupt(e.to_string()))?;
        }
    }
    Ok(bytes)
}

/// Cached nDSM files of `tile`, newest first (undated ones last).
fn cached_ndsm(dir: &Path, tile: TileId) -> Vec<(Option<NaiveDate>, PathBuf)> {
    let name = tile.name();
    let mut files: Vec<(Option<NaiveDate>, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().into_owned();
            let stem = file.strip_prefix(&name)?.strip_suffix(".laz")?;
            let date = match stem {
                "" => None,
                dated => Some(dated.strip_prefix('_')?.parse().ok()?),
            };
            Some((date, entry.path()))
        })
        .collect();
    files.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
    files
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

/// Attempts per request to a server that drops connections now and then (gdi.berlin.de, GeoSN).
pub(crate) const ATTEMPTS: u32 = 3;
#[cfg(not(test))]
const RETRY_DELAY: Duration = Duration::from_secs(2);
#[cfg(test)]
const RETRY_DELAY: Duration = Duration::ZERO;

/// Downloads `url` and reads it with `check`, up to [`ATTEMPTS`] times: server errors, resets
/// and answers `check` refuses (an error page with status 200) are retried; a missing URL or a
/// refused request is not.
pub(crate) fn get_with_retries<T>(
    fetcher: &dyn Fetcher,
    url: &str,
    check: impl Fn(&[u8]) -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    let mut attempt = 1;
    loop {
        let result = fetcher.get(url).and_then(|bytes| check(&bytes));
        match result {
            Err(FetchError::NotFound(_) | FetchError::Refused { .. }) => return result,
            Err(_) if attempt < ATTEMPTS => {
                std::thread::sleep(RETRY_DELAY * attempt);
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// [`crate::fsutil::write_atomically`] with the error as [`FetchError::Io`].
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    crate::fsutil::write_atomically(path, bytes).map_err(|(path, e)| FetchError::Io(path, e))
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
    use crate::sources::alkis::test_support::{building_feature, land_use_feature, page};
    use crate::sources::alkis::{BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT};
    use crate::sources::lgb::Product;
    use crate::tiles::TileId;

    /// Serves fixed responses by URL and counts the requests.
    #[derive(Default)]
    pub(crate) struct MapFetcher {
        pub responses: RefCell<HashMap<String, Vec<u8>>>,
        /// Answers for URLs starting with the key, when no exact response matches.
        pub prefixes: RefCell<HashMap<String, Vec<u8>>>,
        /// `Last-Modified` dates by URL.
        pub modified: RefCell<HashMap<String, chrono::NaiveDate>>,
        pub calls: Cell<usize>,
        /// URLs that fail with a server error.
        pub failing: RefCell<Vec<String>>,
        /// URL prefixes the server refuses with HTTP 400.
        pub refusing: RefCell<Vec<String>>,
        /// URL prefixes that fail with a reset this many more times before they answer.
        pub flaky: RefCell<HashMap<String, u32>>,
        /// Every URL requested with `get`, in order.
        pub log: RefCell<Vec<String>>,
    }

    impl MapFetcher {
        pub fn insert(&self, url: &str, bytes: Vec<u8>) {
            self.responses.borrow_mut().insert(url.to_string(), bytes);
        }

        pub fn insert_prefix(&self, prefix: &str, bytes: Vec<u8>) {
            self.prefixes.borrow_mut().insert(prefix.to_string(), bytes);
        }

        fn fail_if_failing(&self, url: &str) -> Result<(), FetchError> {
            if self
                .refusing
                .borrow()
                .iter()
                .any(|p| url.starts_with(p.as_str()))
            {
                return Err(FetchError::Refused {
                    url: url.to_string(),
                    status: 400,
                });
            }
            if let Some(left) = self
                .flaky
                .borrow_mut()
                .iter_mut()
                .find(|(p, left)| url.starts_with(p.as_str()) && **left > 0)
                .map(|(_, left)| left)
            {
                *left -= 1;
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "Connection reset by peer".to_string(),
                });
            }
            if self.failing.borrow().iter().any(|u| u == url) {
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "503 Service Unavailable".to_string(),
                });
            }
            Ok(())
        }

        pub fn set_modified(&self, url: &str, date: chrono::NaiveDate) {
            self.modified.borrow_mut().insert(url.to_string(), date);
        }
    }

    impl Fetcher for MapFetcher {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            self.log.borrow_mut().push(url.to_string());
            self.fail_if_failing(url)?;
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
                .ok_or_else(|| FetchError::NotFound(url.to_string()))
        }

        fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            self.fail_if_failing(url)?;
            if !self.responses.borrow().contains_key(url) {
                return Err(FetchError::NotFound(url.to_string()));
            }
            Ok(self.modified.borrow().get(url).copied())
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
        solid_jpeg(width, height, [255, 0, 0])
    }

    /// A JPEG of one colour.
    pub fn solid_jpeg(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
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
        // The school's own parcel and residential land south-east of it.
        let land_use = [
            land_use_feature(
                "DEBBAL0100000010",
                "Fläche besonderer funktionaler Prägung",
                Some("Bildung und Wissenschaft"),
                [421_560.0, 5_821_660.0, 100.0],
            ),
            land_use_feature("DEBBAL0100000011", "Wohnbaufläche", None, SCHOOL_NEIGHBOUR),
        ];
        fetcher.insert_prefix(LAND_USE_ENDPOINT, page(&land_use, None));
        fetcher.insert(
            crate::sources::osm::STATE_URL,
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            crate::sources::osm::EXTRACT_URL,
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// A coarse synthetic bDOM archive for `tile` (100 × 100 pixels of 10 m) and its currency
    /// CSV: the surface lies 12 m above the synthetic DTM of [`serve_synthetic_tile`].
    pub fn serve_synthetic_bdom(fetcher: &MapFetcher, tile: TileId, published: &str) {
        let bounds = tile.bounds();
        let world = format!(
            "10\n0\n0\n-10\n{}\n{}\n",
            bounds.min_e + 5.0,
            bounds.max_n - 5.0
        );
        let heights: Vec<f32> = (0..100 * 100).map(|i| (i % 100) as f32 + 12.0).collect();
        let product = Product::ImageDsm;
        fetcher.insert(
            product.currency_csv_url(),
            currency_csv(&[(tile, published)]),
        );
        fetcher.insert(
            &product.tile_url(tile),
            tile_zip(product, tile, &dtm_tiff(100, 100, &heights), &world),
        );
    }

    /// Serves everything a Saxony project in `bbox` fetches: the GeoSN index rows and coarse
    /// synthetic DGM1/DOM1 archives (200 × 200 pixels of 10 m; the DTM rises 1 m per 10 m east,
    /// the surface lies 12 m above it) for the 2 km tiles of `bbox`, a WMS orthophoto of the
    /// right size, the Saxon ALKIS fixtures and the Saxony OSM extract.
    pub fn serve_synthetic_saxony(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::{alkis_wfs, geosn};
        let tiles = crate::tiles::tiles_covering_grid(&bbox.buffered(0.0), 2);
        for (product, layer, lift) in [(Product::SaxonyDtm, 6, 0.0), (Product::SaxonyDsm, 4, 12.0)]
        {
            let rows: Vec<String> = tiles
                .iter()
                .map(|t| {
                    format!(
                        r#"{{"attributes":{{"Kachel":"{:03}{:04}","Download":"{}","Stand":"2024-11-30"}}}}"#,
                        t.east_km,
                        t.north_km,
                        product.tile_url(*t)
                    )
                })
                .collect();
            fetcher.insert_prefix(
                &format!("{}/{layer}/query?", geosn::INDEX_URL),
                format!(r#"{{"features":[{}]}}"#, rows.join(",")).into_bytes(),
            );
            for &tile in &tiles {
                let b = tile.bounds_km(2);
                let world = format!("10\n0\n0\n-10\n{}\n{}\n", b.min_e + 5.0, b.max_n - 5.0);
                let heights: Vec<f32> = (0..200 * 200).map(|i| (i % 200) as f32 + lift).collect();
                fetcher.insert(
                    &product.tile_url(tile),
                    tile_zip(product, tile, &dtm_tiff(200, 200, &heights), &world),
                );
            }
        }
        let grid = crate::raster::GridSpec::covering(bbox, geosn::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                geosn::WMS_URL
            ),
            solid_jpeg(grid.width, grid.height, [0, 200, 0]),
        );
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                geosn::WMS_URL
            ),
            b"Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP".to_vec(),
        );
        for (type_name, body) in [
            (
                alkis_wfs::GEOSN.buildings_type,
                alkis_wfs::test_support::BUILDINGS,
            ),
            (
                alkis_wfs::GEOSN.land_use_type,
                alkis_wfs::test_support::LAND_USE,
            ),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    alkis_wfs::GEOSN.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let saxony = crate::sources::osm::ExtractSource::Saxony;
        fetcher.insert(
            saxony.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            saxony.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// A WCS `multipart/related` answer like LVermGeo's: a GML part, then a Float32 TIFF on
    /// `grid` with ModelPixelScale and ModelTiepoint tags.
    pub fn wcs_answer(grid: &crate::raster::GridSpec, values: &[f32]) -> Vec<u8> {
        let mut answer =
            b"--wcs\nContent-Type: text/xml\nContent-ID: GML-Part\n\n<coverage/>\n--wcs\nContent-Type: image/tiff\nContent-ID: 1.tif\n\n"
                .to_vec();
        answer.extend_from_slice(&geotiff(grid, values));
        answer.extend_from_slice(b"\n--wcs--\n");
        answer
    }

    /// A Float32 GeoTIFF on `grid` with ModelPixelScale and ModelTiepoint tags (no world file),
    /// like the LVermGeo WCS answers and the LGV height model tiles.
    pub fn geotiff(grid: &crate::raster::GridSpec, values: &[f32]) -> Vec<u8> {
        use tiff::tags::Tag;
        let mut tiff = std::io::Cursor::new(Vec::new());
        {
            let mut encoder = tiff::encoder::TiffEncoder::new(&mut tiff).unwrap();
            let mut image = encoder
                .new_image::<tiff::encoder::colortype::Gray32Float>(grid.width, grid.height)
                .unwrap();
            image
                .encoder()
                .write_tag(
                    Tag::ModelPixelScaleTag,
                    &[grid.pixel_size, grid.pixel_size, 0.0][..],
                )
                .unwrap();
            image
                .encoder()
                .write_tag(
                    Tag::ModelTiepointTag,
                    &[0.0, 0.0, 0.0, grid.west, grid.north, 0.0][..],
                )
                .unwrap();
            image.write_data(values).unwrap();
        }
        tiff.into_inner()
    }

    /// Serves everything a Saxony-Anhalt project in `bbox` (EPSG:25832) fetches: DGM1 and DOM1
    /// from the WCS (the terrain rises 1 m per metre east from 80 m, the surface lies 12 m above
    /// it), a WMS orthophoto of the right size, the tile index dates, the Saxony-Anhalt ALKIS
    /// fixtures and the Saxony-Anhalt OSM extract.
    pub fn serve_synthetic_saxony_anhalt(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::{alkis_wfs, lvermgeo};
        let grid = crate::raster::GridSpec::covering(bbox, 1.0);
        for (model, lift) in [(lvermgeo::Model::Dgm1, 80.0), (lvermgeo::Model::Dom1, 92.0)] {
            let values: Vec<f32> = (0..grid.width * grid.height)
                .map(|i| (i % grid.width) as f32 * 0.01 + lift)
                .collect();
            fetcher.insert_prefix(model.wcs_endpoint(), wcs_answer(&grid, &values));
        }
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                lvermgeo::WMS.url
            ),
            solid_jpeg(photo.width, photo.height, [200, 0, 200]),
        );
        for (layer, body) in [
            ("truedop_kacheln", lvermgeo::test_support::TRUEDOP_INDEX),
            (
                "bdom_aktualitaet_zyklus",
                lvermgeo::test_support::BDOM_INDEX,
            ),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=atkis_geobasis_bestand:{layer}&",
                    lvermgeo::INDEX_URL
                ),
                body.as_bytes().to_vec(),
            );
        }
        let service = alkis_wfs::LVERMGEO;
        for (type_name, body) in [
            (
                service.buildings_type,
                alkis_wfs::test_support::ST_BUILDINGS,
            ),
            (service.land_use_type, alkis_wfs::test_support::ST_LAND_USE),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    service.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let extract = crate::sources::osm::ExtractSource::SaxonyAnhalt;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// An nDSM tile like LGB's: LAZ, 0.01 m scale; a 2 m hedge line north–south through the school.
    pub fn serve_synthetic_ndsm(fetcher: &MapFetcher, tile: TileId) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ndsm.laz");
        let mut builder = las::Builder::from((1, 2));
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
        for i in 0..500 {
            writer
                .write_point(las::Point {
                    x: 421_610.1,
                    y: 5_821_650.1 + f64::from(i) * 0.2,
                    z: 2.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        let url = crate::sources::lgb::ndsm_tile_url(tile);
        fetcher.insert(&url, std::fs::read(&path).unwrap());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
    }

    /// The OSM extract fixture cut around the school.
    pub const OSM_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/strausberg-school.osm.pbf"
    );

    /// Residential land south-east of the fixture school area (EPSG:25833 corner + side).
    pub const SCHOOL_NEIGHBOUR: [f64; 3] = [421_690.0, 5_821_600.0, 20.0];

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
        assert!(err.to_string().contains("tile 33999-5999 is not"), "{err}");
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
            matches!(&err, FetchError::MissingOffline(_, tiles) if tiles.len() == 2),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("33421-5821") && message.contains("33421-5822"),
            "{message}"
        );
        assert!(message.contains("terrain model (DGM1)"), "{message}");
    }

    const NORTH: TileId = TileId {
        east_km: 421,
        north_km: 5822,
    };
    const EAST: TileId = TileId {
        east_km: 422,
        north_km: 5821,
    };

    /// Serves the GeoSN DGM1 index fixture and archives for its first two tiles.
    fn serve_geosn(fetcher: &MapFetcher) -> [TileId; 2] {
        let tiles = [
            TileId::parse("33414-5654").unwrap(),
            TileId::parse("33416-5654").unwrap(),
        ];
        let product = Product::SaxonyDtm;
        fetcher.insert_prefix(
            &format!("{}/6/query?", crate::sources::geosn::INDEX_URL),
            crate::sources::geosn::test_support::DGM1_INDEX
                .as_bytes()
                .to_vec(),
        );
        for tile in tiles {
            let world = format!(
                "1\n0\n0\n-1\n{}.5\n{}.5\n",
                tile.east_km * 1000,
                tile.north_km * 1000 + 1999
            );
            fetcher.insert(
                &product.tile_url(tile),
                tile_zip(product, tile, &dtm_tiff(2, 2, &[100.0; 4]), &world),
            );
        }
        tiles
    }

    #[test]
    fn refused_requests_are_not_retried() {
        let fetcher = MapFetcher::default();
        fetcher
            .refusing
            .borrow_mut()
            .push("https://example.org/".to_string());
        let err =
            get_with_retries(&fetcher, "https://example.org/x", |b| Ok(b.to_vec())).unwrap_err();
        assert!(
            matches!(err, FetchError::Refused { status: 400, .. }),
            "{err:?}"
        );
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn index_query_is_retried() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        fetcher
            .flaky
            .borrow_mut()
            .insert(format!("{}/6/query?", crate::sources::geosn::INDEX_URL), 1);
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
    }

    #[test]
    fn index_row_without_download_uses_the_share() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        fetcher.insert_prefix(
            &format!("{}/6/query?", crate::sources::geosn::INDEX_URL),
            br#"{"features":[{"attributes":{"Kachel":"4145654","Download":null,"Stand":"2024-11-30"}}]}"#.to_vec(),
        );
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles[..1],
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched[0].url, Product::SaxonyDtm.tile_url(tiles[0]));
    }

    #[test]
    fn offline_tile_missing_from_the_cached_index_is_missing_offline() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        let product = Product::SaxonyDtm;
        fetch_tiles(
            &fetcher,
            &cache,
            product,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        let elsewhere = TileId::parse("33440-5660").unwrap();
        let err = fetch_tiles(
            &MapFetcher::default(),
            &cache,
            product,
            &[elsewhere],
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::MissingOffline(_, t) if t == &[elsewhere]),
            "{err:?}"
        );
    }

    #[test]
    fn geosn_tiles_fetch_and_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].url, Product::SaxonyDtm.tile_url(tiles[0]));
        assert_eq!(fetched[0].currency.published, "2024-11-30".parse().unwrap());
        assert_eq!(fetched[1].currency.published, "2020-11-30".parse().unwrap());
        assert!(
            fetched[0]
                .zip
                .starts_with(dir.path().join("geosn").join("dgm1")),
            "{:?}",
            fetched[0].zip
        );
        assert!(fetched[0].zip.exists());
    }

    #[test]
    fn offline_uses_cached_index() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let online = MapFetcher::default();
        let tiles = serve_geosn(&online);
        let product = Product::SaxonyDtm;
        fetch_tiles(
            &online,
            &cache,
            product,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        let offline = MapFetcher::default();
        let fetched = fetch_tiles(
            &offline,
            &cache,
            product,
            &tiles,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(offline.calls.get(), 0);
        assert_eq!(fetched[1].currency.published, "2020-11-30".parse().unwrap());
    }

    #[test]
    fn geosn_tile_missing_from_the_index_is_not_covered() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_geosn(&fetcher);
        let far = TileId::parse("33500-5600").unwrap();
        let err = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &[far],
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::TilesNotCovered(t) if t == &[far]),
            "{err:?}"
        );
        assert!(!err.to_string().contains("LGB"), "{err}");
    }

    #[test]
    fn each_tile_succeeds_or_fails_alone() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetcher.insert(
            PRODUCT.currency_csv_url(),
            currency_csv(&[(TILE, "2026-01-26"), (EAST, "2026-01-26")]),
        );
        fetcher.failing.borrow_mut().push(PRODUCT.tile_url(EAST));
        let tiles = [TILE, EAST, NORTH];
        let results = fetch_tiles_each(
            &fetcher,
            &cache,
            PRODUCT,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, TILE);
        assert!(results[0].1.is_ok(), "{:?}", results[0].1);
        assert!(
            matches!(results[1].1, Err(FetchError::Http { .. })),
            "{:?}",
            results[1].1
        );
        assert!(
            matches!(&results[2].1, Err(FetchError::TilesNotCovered(t)) if t == &[NORTH]),
            "{:?}",
            results[2].1
        );

        let results = fetch_tiles_each(
            &fetcher,
            &cache,
            PRODUCT,
            &[TILE, EAST],
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert!(results[0].1.is_ok());
        assert!(
            matches!(&results[1].1, Err(FetchError::MissingOffline(_, t)) if t == &[EAST]),
            "{:?}",
            results[1].1
        );
    }

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
        assert!(
            message.contains("tiles 33421-5822, 33422-5821 are"),
            "{message}"
        );
        assert!(message.contains("buffer_m"), "{message}");
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
            messages.contains(&"warning: using cached terrain model (DGM1) tile 33421-5821 from 2026-01-26 (newer 2026-06-01 not downloaded)".to_string()),
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
        let two_hours_ago = std::time::SystemTime::now() - Duration::from_secs(2 * 3600);
        for stale in [
            "33421-5821_2026-01-26.zip.part",
            "33421-5821_2025-01-01.zip.999999.part",
            "currency.csv.999999.part",
        ] {
            let path = product_dir.join(stale);
            std::fs::write(&path, b"partial").unwrap();
            let file = std::fs::File::options().write(true).open(&path).unwrap();
            file.set_modified(two_hours_ago).unwrap();
        }
        // Another process is still writing this one.
        let running = "33421-5821_2026-01-26.zip.888888.part";
        std::fs::write(product_dir.join(running), b"partial").unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            cached_files(&cache),
            [
                "33421-5821_2026-01-26.zip",
                "33421-5821_2026-01-26.zip.888888.part",
                "currency.csv"
            ]
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

    /// A small valid LAZ point cloud.
    fn las_bytes() -> Vec<u8> {
        let source = MapFetcher::default();
        test_support::serve_synthetic_ndsm(&source, TILE);
        source.get(&ndsm_tile_url(TILE)).unwrap()
    }

    fn ndsm(fetcher: &MapFetcher, cache: &Cache, mode: FetchMode) -> (Vec<NdsmTile>, Vec<String>) {
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(fetcher, cache, &[TILE, NORTH], mode, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        (tiles, messages)
    }

    #[test]
    #[ignore = "network"]
    fn live_ndsm_last_modified() {
        let date = HttpFetcher::new()
            .last_modified(&ndsm_tile_url(TILE))
            .unwrap();
        assert!(
            date.is_some_and(|d| d.to_string().as_str() >= "2025-01-01"),
            "{date:?}"
        );
        let missing = TileId {
            east_km: 999,
            north_km: 5999,
        };
        let err = HttpFetcher::new()
            .last_modified(&ndsm_tile_url(missing))
            .unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err:?}");
    }

    #[test]
    fn ndsm_server_error_falls_back_to_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let url = ndsm_tile_url(TILE);
        fetcher.insert(&url, las_bytes());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        fetcher.failing.borrow_mut().push(url);
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(
            &fetcher,
            &cache,
            &[TILE, NORTH],
            FetchMode::Online,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(tiles.len(), 1, "the cached tile is used");
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: nDSM tile 33421-5821 unavailable")),
            "{messages:?}"
        );
    }

    #[test]
    fn truncated_laz_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let mut bytes = std::fs::read(cache_free_laz(dir.path())).unwrap();
        bytes.truncate(bytes.len() / 2);
        fetcher.insert(&ndsm_tile_url(TILE), bytes);
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(tiles.is_empty());
        assert!(
            messages
                .iter()
                .any(|m| m.contains("not a complete LAS point cloud")),
            "{messages:?}"
        );
    }

    /// A small valid LAZ file written outside the cache.
    fn cache_free_laz(dir: &Path) -> PathBuf {
        let source = MapFetcher::default();
        test_support::serve_synthetic_ndsm(&source, TILE);
        let path = dir.join("source.laz");
        std::fs::write(&path, source.get(&ndsm_tile_url(TILE)).unwrap()).unwrap();
        path
    }

    #[test]
    fn ndsm_downloaded_then_reused() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        for tile in [TILE, NORTH] {
            let url = ndsm_tile_url(tile);
            fetcher.insert(&url, las_bytes());
            fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        }
        let (first, _) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].published, "2025-03-03".parse().ok());
        assert!(
            first[0].path.ends_with("33421-5821_2025-03-03.laz"),
            "{:?}",
            first[0].path
        );
        let calls = fetcher.calls.get();
        let (again, _) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(again, first);
        assert_eq!(fetcher.calls.get(), calls + 2, "only the two date checks");
    }

    #[test]
    fn ndsm_republished_replaces_old() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let url = ndsm_tile_url(TILE);
        fetcher.insert(&url, las_bytes());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        fetcher.set_modified(&url, "2026-09-01".parse().unwrap());
        let tiles =
            fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        let names: Vec<String> = std::fs::read_dir(tiles[0].path.parent().unwrap())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["33421-5821_2026-09-01.laz"]);
    }

    #[test]
    fn missing_ndsm_tile_is_skipped_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), las_bytes());
        let (tiles, messages) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].published, None, "no Last-Modified");
        assert!(
            messages.contains(&"no nDSM point cloud published for tile 33421-5822".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn offline_uses_cached_ndsm() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), las_bytes());
        fetcher.set_modified(&ndsm_tile_url(TILE), "2025-03-03".parse().unwrap());
        ndsm(&fetcher, &cache, FetchMode::Online);
        let calls = fetcher.calls.get();
        let (tiles, messages) = ndsm(&fetcher, &cache, FetchMode::Offline);
        assert_eq!(fetcher.calls.get(), calls);
        assert_eq!(tiles.len(), 1);
        assert!(
            messages.contains(&"warning: no cached nDSM for tile 33421-5822".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn not_a_las_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), b"<html>maintenance</html>".to_vec());
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(tiles.is_empty(), "skipped, not cached");
        assert!(
            messages
                .iter()
                .any(|m| m.contains("not a complete LAS point cloud")),
            "{messages:?}"
        );
        assert!(cached_ndsm(&cache.ndsm_dir(), TILE).is_empty());
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
    fn ignored_range_is_an_error() {
        // A server that answers every request with the whole file (200, not 206).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/big.zip", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            use std::io::{Read as _, Write as _};
            for stream in listener.incoming().take(2).flatten() {
                let mut stream = stream;
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let body = vec![b'x'; 10_000];
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        let fetcher = HttpFetcher::new();
        let err = fetcher
            .get_range(
                &url,
                ByteRange::From {
                    start: 100,
                    len: 10,
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("byte range"), "{err}");
    }

    #[test]
    fn default_range_slices_the_answer() {
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/f", b"0123456789".to_vec());
        let url = "https://example.org/f";
        assert_eq!(
            fetcher
                .get_range(url, ByteRange::From { start: 2, len: 3 })
                .unwrap(),
            b"234"
        );
        assert_eq!(fetcher.get_range(url, ByteRange::Last(4)).unwrap(), b"6789");
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
