//! The tile download of LGB and GeoSN: where the tiles and their currency lists lie in the
//! cache, which publication of a tile is current, and the nDSM point clouds of LGB.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::cache::STALE_PART_AGE;

use crate::fetch::{
    CacheDir, FetchError, FetchMode, FetchedTile, Fetcher, get_with_retries, write_cache_file,
};
use crate::lgb::{
    Product, ProductTiles as _, TileCurrency, TileProvider, ndsm_tile_url, parse_currency_csv,
};
use crate::note::{Arg, Note, note};
use crate::tiles::TileId;

/// Where the tiles of LGB and GeoSN and their currency lists lie in the cache. It stays here,
/// beside the tile download, until the tile backends move (ADR 0082).
pub trait TilePaths {
    fn tile_path(&self, product: Product, tile: TileId, published: NaiveDate) -> PathBuf;

    /// Folder of the cached nDSM point clouds.
    fn ndsm_dir(&self) -> PathBuf;

    /// Cached nDSM file of `tile`, named after the server date (`<tile>.laz` when undated).
    fn ndsm_path(&self, tile: TileId, published: Option<NaiveDate>) -> PathBuf;

    /// Whether some publication of `tile` of `product` is cached.
    fn has_tile(&self, product: Product, tile: TileId) -> bool;

    /// Whether an nDSM point cloud of `tile` is cached.
    fn has_ndsm(&self, tile: TileId) -> bool;

    fn currency_csv_path(&self, product: Product) -> PathBuf;

    /// The collected GeoSN tile index rows of `product`.
    fn currency_index_path(&self, product: Product) -> PathBuf;
}

/// The folder of `product` in the cache.
fn product_dir(cache: &CacheDir, product: Product) -> PathBuf {
    cache
        .provider_dir(product.provider_dir_name())
        .join(product.cache_dir_name())
}

impl TilePaths for CacheDir {
    fn tile_path(&self, product: Product, tile: TileId, published: NaiveDate) -> PathBuf {
        product_dir(self, product).join(format!("{}_{published}.zip", tile.name()))
    }

    fn ndsm_dir(&self) -> PathBuf {
        self.provider_dir("lgb").join("ndom")
    }

    fn ndsm_path(&self, tile: TileId, published: Option<NaiveDate>) -> PathBuf {
        self.ndsm_dir().join(match published {
            Some(date) => format!("{}_{date}.laz", tile.name()),
            None => format!("{}.laz", tile.name()),
        })
    }

    fn has_tile(&self, product: Product, tile: TileId) -> bool {
        let probe = self.tile_path(product, tile, NaiveDate::MIN);
        !cached_versions(&probe, tile).is_empty()
    }

    fn has_ndsm(&self, tile: TileId) -> bool {
        !cached_ndsm(&self.ndsm_dir(), tile).is_empty()
    }

    fn currency_csv_path(&self, product: Product) -> PathBuf {
        product_dir(self, product).join("currency.csv")
    }

    fn currency_index_path(&self, product: Product) -> PathBuf {
        product_dir(self, product).join("currency.json")
    }
}

/// Makes `tiles` of `product` available in `cache` and returns them in the given order.
///
/// Online, the currency CSV is downloaded on every call and a tile is downloaded only when the
/// cache lacks its current publication. Offline, the cached CSV and archives are used.
pub fn fetch_tiles(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
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
    cache: &CacheDir,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
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
    if mode == FetchMode::Offline && product.provider() == TileProvider::GeoSn {
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
    cache: &CacheDir,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
) -> Result<CurrencyTable, FetchError> {
    match product.provider() {
        TileProvider::Lgb => lgb_currency_table(fetcher, cache, product, tiles, mode, progress),
        TileProvider::GeoSn => geosn_currency_table(fetcher, cache, product, tiles, mode, progress),
    }
}

fn lgb_currency_table(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
) -> Result<CurrencyTable, FetchError> {
    let csv_path = cache.currency_csv_path(product);
    let csv = match mode {
        FetchMode::Online => {
            let url = product.currency_csv_url();
            progress(note!("progress-downloading", url = url));
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
        write_cache_file(&csv_path, &csv)?;
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
    cache: &CacheDir,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
) -> Result<CurrencyTable, FetchError> {
    let path = cache.currency_index_path(product);
    let mut cached: std::collections::BTreeMap<String, (NaiveDate, String)> = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    if mode == FetchMode::Online {
        let url = crate::sources::geosn::index_url(product, tiles);
        progress(note!(
            "progress-tile-index-download",
            product = format!("{product}")
        ));
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
        write_cache_file(&path, &json)?;
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
    cache: &CacheDir,
    product: Product,
    tile: TileId,
    (mut currency, url): (TileCurrency, String),
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
) -> Result<Option<FetchedTile>, FetchError> {
    let mut zip = cache.tile_path(product, tile, currency.published);
    if zip.exists() {
        progress(note!("progress-tile-cached", tile = tile.name()));
    } else if mode == FetchMode::Offline {
        match newest_cached_version(&zip, tile) {
            Some((published, older)) => {
                progress(note!(
                    "warning-tile-cached-older",
                    product = format!("{product}"),
                    tile = tile.name(),
                    published = published.to_string(),
                    newer = currency.published.to_string()
                ));
                currency.published = published;
                zip = older;
            }
            None => return Ok(None),
        }
    } else {
        progress(note!("progress-downloading", url = &url));
        let bytes = fetcher.get(&url)?;
        check_archive(&bytes, product, tile).map_err(|message| FetchError::CorruptArchive {
            url: url.clone(),
            message,
        })?;
        write_cache_file(&zip, &bytes)?;
        remove_older_versions(&zip, tile)?;
        freemapper_fs::remove_stale_cache_parts(zip.parent().expect("cache paths"), STALE_PART_AGE);
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
    cache: &CacheDir,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
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
                None => progress(note!("warning-ndsm-no-cached", tile = tile.name())),
            }
            continue;
        }
        // The nDSM is only a template: a failing server falls back to the cache or skips the tile.
        let unavailable = |reason: Arg, progress: &mut dyn FnMut(Note)| {
            let fallback = cached.first().cloned();
            progress(if fallback.is_some() {
                note!(
                    "warning-ndsm-tile-unavailable-cached",
                    tile = tile.name(),
                    reason = reason
                )
            } else {
                note!(
                    "warning-ndsm-tile-unavailable-skipped",
                    tile = tile.name(),
                    reason = reason
                )
            });
            fallback
        };
        let published = match fetcher.last_modified(&url) {
            Ok(date) => date,
            Err(FetchError::NotFound(_)) => {
                progress(note!("progress-ndsm-not-published", tile = tile.name()));
                continue;
            }
            Err(e) => {
                if let Some((published, path)) = unavailable(Arg::error(e), progress) {
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
            progress(note!("progress-ndsm-cached", tile = tile.name()));
        } else {
            progress(note!("progress-downloading", url = &url));
            let bytes = match fetcher.get(&url).and_then(|b| check_las(&url, b)) {
                Ok(bytes) => bytes,
                Err(e) => {
                    if let Some((published, path)) = unavailable(Arg::error(e), progress) {
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
            write_cache_file(&path, &bytes)?;
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
pub fn cached_ndsm(dir: &Path, tile: TileId) -> Vec<(Option<NaiveDate>, PathBuf)> {
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
