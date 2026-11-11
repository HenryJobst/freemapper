//! The LVermGeo SH Downloadportal (`geodaten.schleswig-holstein.de`, CC BY 4.0, "© GeoBasis-DE/
//! LVermGeo SH"): the DGM1 and bDOM tiles under an area, looked up through the portal's AJAX
//! `overview.php` (EPSG:25832, 1 km tiles).
//!
//! File names carry a year that differs per tile, so they must come from the portal; the
//! `massen.php` links are built from the same attributes as the portal's own index
//! (`link_data`).

use crate::note::{Arg, Note, note};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{
    Cache, FetchError, FetchMode, Fetcher, STALE_PART_AGE, download_with_retries, get_with_retries,
    write_cache_file,
};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{
    decode_float_tiff, decode_float_tiff_from, encode_float_geotiff, write_float_raster,
};
use crate::sources::height_model::HeightModel;
use crate::sources::region::{Region, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::tiles::{TileId, tiles_covering};

/// The Downloadportal; its host is reached through the bundled D-TRUST anchors (see `fetch`).
pub const PORTAL: &str = "https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload";

/// A product of the Downloadportal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Product {
    /// DGM1, the terrain (XYZ on the portal).
    Dgm1,
    /// bDOM 0.2 m, the image-based surface (GeoTIFF).
    Bdom,
}

impl Product {
    /// The `type[]` of `overview.php`.
    pub(crate) fn overview_type(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Bdom => "bdom",
        }
    }
}

/// A tile of a product as the portal lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalTile {
    pub tile: TileId,
    /// File name without extension, e.g. `dgm1_32_565_5949_1_sh_2022`.
    pub name: String,
    pub year: u16,
    /// Acquisition (bDOM: flight) date, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// The `massen.php` download URL.
    pub link: String,
}

/// The AJAX tile lookup of `product` over `bbox` (EPSG:25832), rounded outward to whole metres.
/// The product comes first, so a test server can answer each product by URL prefix.
pub(crate) fn overview_url(bbox: &BoundingBox, product: Product) -> String {
    format!(
        "{}{}&crs=EPSG:25832",
        overview_prefix(product),
        bbox_key(bbox).replace('_', ",")
    )
}

/// The start of every [`overview_url`] of `product`, up to the bbox.
pub(crate) fn overview_prefix(product: Product) -> String {
    format!(
        "{PORTAL}/_ajax/overview.php?type[]={}&bbox=",
        product.overview_type()
    )
}

/// The bbox of an overview request: `min_e_min_n_max_e_max_n`, rounded outward to whole metres.
fn bbox_key(bbox: &BoundingBox) -> String {
    format!(
        "{}_{}_{}_{}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// The tiles of `product` intersecting `bbox` (EPSG:25832), sorted by tile.
///
/// The portal answers at most 200 tiles (a 4 x 4 km area has 25); more, and an answer that is
/// not a tile overview (the portal reports errors as HTML with status 200), are errors.
pub fn overview(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    product: Product,
) -> Result<Vec<PortalTile>, FetchError> {
    let url = overview_url(bbox, product);
    // Retries cover the transport only; a refused or malformed answer does not change.
    let bytes = get_with_retries(fetcher, &url, |bytes| Ok(bytes.to_vec()))?;
    parse_overview(&url, &bytes, product)
}

/// An `overview.php` answer from `url` (see [`overview`]).
fn parse_overview(
    url: &str,
    bytes: &[u8],
    product: Product,
) -> Result<Vec<PortalTile>, FetchError> {
    let error = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let json: Value = serde_json::from_slice(bytes).map_err(|_| {
        error(format!(
            "not a tile overview: {}",
            excerpt(&String::from_utf8_lossy(bytes))
        ))
    })?;
    if json["success"] != true {
        return Err(error(match json["reason"].as_str() {
            Some("tooManyObjects") => format!(
                "the Downloadportal lists at most {} tiles per request; reduce the map area",
                json["max"].as_u64().unwrap_or(200)
            ),
            _ => format!("not a tile overview: {}", excerpt(&json.to_string())),
        }));
    }
    let features = json["result"]["features"]
        .as_array()
        .ok_or_else(|| error("not a tile overview: no features".to_string()))?;
    let matching: Vec<&Value> = features
        .iter()
        .map(|feature| &feature["properties"])
        .filter(|properties| text(&properties["type"]).as_deref() == Some(product.overview_type()))
        .collect();
    // Features, but none of the requested type: the portal changed, not an empty area.
    if matching.is_empty() && !features.is_empty() {
        return Err(error(format!(
            "the overview lists {} tiles but none of type `{}`; the Downloadportal has changed",
            features.len(),
            product.overview_type()
        )));
    }
    let mut tiles = matching
        .into_iter()
        .map(|properties| portal_tile(properties, product).map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    tiles.sort_by_key(|t| t.tile);
    Ok(tiles)
}

/// One overview entry; every part of the link is checked to be plain letters, digits and `_`.
fn portal_tile(properties: &Value, product: Product) -> Result<PortalTile, String> {
    let field = |key: &str| {
        text(&properties[key])
            .filter(|value| is_plain(value))
            .ok_or_else(|| format!("tile entry without a valid `{key}`: {properties}"))
    };
    let title = field("title")?;
    let tile = parse_tile_key(&title).ok_or_else(|| format!("unknown tile key {title}"))?;
    let year = field("jahr")?;
    let year: u16 = year.parse().map_err(|_| format!("unknown year {year}"))?;
    let date =
        text(&properties["a_datum"]).filter(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok());
    let (name, link) = match product {
        Product::Dgm1 => {
            let name = field("kaname")?;
            let km = field("filepath")?;
            let link = format!("{PORTAL}/massen.php?file={name}.xyz&id=2&live={year}&km={km}");
            (name, link)
        }
        Product::Bdom => {
            let listed = field("kachel_n")?;
            // The overview names the tile `bdom20rgbi_…`; the file is `bdom20nc_…`. Any other
            // name would build a link the portal answers with an HTML error.
            let name = match listed.strip_prefix("bdom20rgbi_") {
                Some(rest) => format!("bdom20nc_{rest}"),
                None if listed.starts_with("bdom20nc_") => listed,
                None => return Err(format!("unknown bDOM tile name {listed}")),
            };
            let km = field("kach10km")?;
            let stack = field("filepath")?;
            let link = format!(
                "{PORTAL}/massen.php?file={name}.tif&id=6&live={year}&km={km}&stack={stack}"
            );
            (name, link)
        }
    };
    Ok(PortalTile {
        tile,
        name,
        year,
        date,
        link,
    })
}

/// A string or number property as text.
fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Safe to put into a URL unescaped.
pub(crate) fn is_plain(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// `325655949` (zone 32, E 565 km, N 5949 km).
fn parse_tile_key(key: &str) -> Option<TileId> {
    let digits = key.strip_prefix("32")?;
    if digits.len() != 7 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(TileId {
        east_km: digits[..3].parse().ok()?,
        north_km: digits[3..].parse().ok()?,
    })
}

/// The start of an error answer, on one line and without control characters (an escape
/// sequence from the server must not reach the user's terminal).
fn excerpt(text: &str) -> String {
    let printable: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let line = printable.split_whitespace().collect::<Vec<_>>().join(" ");
    match line.char_indices().nth(200) {
        Some((end, _)) => format!("{}…", &line[..end]),
        None => line,
    }
}

/// The orthophoto WMS (WMS 1.3.0, MaxWidth/MaxHeight 5000; an oversize request is an XML
/// exception). The layer shows the current mosaic; its tones are processed, not the tiles'.
pub const WMS: WmsService = WmsService {
    url: "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD",
    layer: "sh_dop20_rgb",
    crs: Crs::Utm32,
    extra: "",
    max_px: 4000,
};

/// The metadata WMS of the orthophoto tiles (queryable; one polygon per 1 km tile).
const WMS_INFO: &str = "https://service.gdi-sh.de/WMS_SH_MD_DOP";
const WMS_INFO_LAYER: &str = "DOP20";

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the metadata WMS; `None` outside the coverage or when the answer has none.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let url = format!(
        "{WMS_INFO}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={WMS_INFO_LAYER}\
         &QUERY_LAYERS={WMS_INFO_LAYER}&STYLES=&CRS=EPSG:{}&BBOX={},{},{},{}\
         &WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain",
        Crs::Utm32.epsg(),
        crate::sources::wms::mm(e - 100.0),
        crate::sources::wms::mm(n - 100.0),
        crate::sources::wms::mm(e + 100.0),
        crate::sources::wms::mm(n + 100.0),
    );
    let bytes = get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    parse_flight_date(&String::from_utf8_lossy(&bytes))
}

/// `A_DATUM2` (`DD.MM.YYYY`, else `A_DATUM` as `YYYY-MM-DD`) of a semicolon-separated
/// GetFeatureInfo text answer: the field names, then the values of the one tile, each ending
/// in `;` (the server puts both on one line).
fn parse_flight_date(text: &str) -> Option<String> {
    let tokens: Vec<&str> = text.split(';').map(str::trim).collect();
    // The names end where the first value (the numeric FID) begins; the first record is the
    // `k` tokens from there, however many records follow.
    let k = tokens
        .iter()
        .position(|t| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))?;
    let (header, values) = (&tokens[..k], tokens.get(k..2 * k)?);
    if header.is_empty() {
        return None;
    }
    let column = |name: &str| {
        header
            .iter()
            .position(|h| h.trim_start_matches("@DOP20 ") == name)
            .map(|at| values[at])
    };
    let digits = |s: &str, n| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    if let Some(value) = column("A_DATUM2") {
        let parts: Vec<&str> = value.split('.').collect();
        if let [d, m, y] = parts[..]
            && digits(d, 2)
            && digits(m, 2)
            && digits(y, 4)
        {
            return Some(value.to_string());
        }
    }
    let parts: Vec<&str> = column("A_DATUM")?.split('-').collect();
    let [y, m, d] = parts[..] else { return None };
    (digits(d, 2) && digits(m, 2) && digits(y, 4)).then(|| format!("{d}.{m}.{y}"))
}

/// The INSPIRE ATOM data directory of the DGM1 GeoTIFFs (`<name>.tif`); a static server, valid
/// TLS, 404 for a name it does not have.
pub const ATOM_DATA: &str = "https://service.gdi-sh.de/INSPIRE_Atom/EL/data";

/// The height models are the Downloadportal's products.
pub type Model = Product;

impl Product {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LVermGeo SH DGM1)",
            Self::Bdom => "surface model (LVermGeo SH bDOM)",
        }
    }
}

/// Cells per side of a DGM1 XYZ (1 m).
const XYZ_CELLS: u32 = 1000;
/// The size of a bDOM tile download (5000 x 5000 Float32, uncompressed).
const BDOM_MB: u32 = 105;
/// The size of a DGM1 XYZ download.
const XYZ_MB: u32 = 28;
/// Largest bDOM download accepted (a tile is 105 MB); a portal streaming without end stops here.
const BDOM_MAX_BYTES: u64 = 150 * 1024 * 1024;
/// Largest DGM1 XYZ download accepted (a tile is 28 MB).
const XYZ_MAX_BYTES: u64 = 64 * 1024 * 1024;
// The limits leave room above the sizes the portal sends.
const _: () = assert!(BDOM_MAX_BYTES > BDOM_MB as u64 * 1024 * 1024);
const _: () = assert!(XYZ_MAX_BYTES > XYZ_MB as u64 * 1024 * 1024);

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file.
///
/// The tiles and their names come from the Downloadportal's overview (cached per area for
/// offline runs). DGM1 is the INSPIRE ATOM GeoTIFF, or the portal's XYZ where the ATOM directory
/// lacks the tile (404). The bDOM (0.2 m, 105 MB, about 90 s per tile) is streamed to a
/// temporary file and reduced to 1 m (the median of each 5 x 5 block). Only the 1 m tiles are
/// cached, under the portal's name (which carries the vintage). Tiles inside Schleswig-Holstein
/// that the portal does not list are returned as missing; the date is the centre tile's. No
/// tile at all is an error.
#[allow(clippy::too_many_arguments)]
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<HeightModel, FetchError> {
    fetch_model_with(
        fetcher, cache, mode, model, bbox, root, out, progress, XYZ_CELLS,
    )
}

/// [`fetch_model`] with XYZ tiles of `xyz_cells` x `xyz_cells` points (tests use fewer).
#[allow(clippy::too_many_arguments)]
fn fetch_model_with(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    progress: &mut dyn FnMut(Note),
    xyz_cells: u32,
) -> Result<HeightModel, FetchError> {
    let dir = cache.lvermgeo_sh_dir();
    let mut warnings = Vec::new();
    let listed = tile_list(fetcher, &dir, mode, bbox, model, &mut warnings)?;
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let tiles: Vec<(TileId, Option<&PortalTile>)> = tiles_covering(bbox)
        .into_iter()
        .map(|tile| (tile, listed.iter().find(|t| t.tile == tile)))
        .collect();
    let (mut found, mut missing, mut pending) = (0, Vec::new(), Vec::new());
    let mut paste = |raster: &mut Raster<f32>, entry: &PortalTile, part: &Raster<f32>| {
        found += 1;
        let empty = paste_tile(raster, part).map_err(|message| FetchError::Http {
            url: entry.link.clone(),
            message: format!("{} tile {}: {message}", model.label(), entry.name),
        })?;
        if empty > 0 {
            warnings.push(format!(
                "{} tile {}: {empty} cells of the map area have no height",
                model.label(),
                short_name(entry.tile)
            ));
        }
        Ok::<(), FetchError>(())
    };
    // The cached tiles first, so the downloads are counted right (a cached tile that does not
    // read is downloaded again).
    for (tile, entry) in tiles {
        match entry {
            None if tile_centre_in(tile, Region::SchleswigHolstein) => missing.push(tile),
            None => {}
            Some(entry) => match read_cached(&cache_path(&dir, model, entry), tile) {
                Some(part) => paste(&mut raster, entry, &part)?,
                None => pending.push(entry),
            },
        }
    }
    if let (FetchMode::Offline, Some(entry)) = (mode, pending.first()) {
        return Err(FetchError::Http {
            url: entry.link.clone(),
            message: format!(
                "offline and not cached: {} tile {}; run `freemapper fetch` online",
                model.label(),
                entry.name
            ),
        });
    }
    // Reported after the tiles, as the paste closure holds the warnings.
    let mut fallbacks = Vec::new();
    for (n, entry) in pending.iter().enumerate() {
        let part = match model {
            Model::Dgm1 => {
                let (part, fallback) = dgm1_tile(fetcher, &dir, entry, xyz_cells, progress)?;
                fallbacks.extend(fallback);
                part
            }
            Model::Bdom => {
                progress(note!(
                    "progress-sh-bdom-download",
                    tile = short_name(entry.tile),
                    megabytes = BDOM_MB.to_string(),
                    index = (n + 1).to_string(),
                    total = pending.len().to_string()
                ));
                bdom_tile(fetcher, &dir, entry, progress)?
            }
        };
        paste(&mut raster, entry, &part)?;
    }
    warnings.extend(fallbacks);
    if found == 0 {
        return Err(FetchError::Http {
            url: overview_url(bbox, model),
            message: format!(
                "no tile of the {} covers the map area; the Downloadportal may have changed",
                model.label()
            ),
        });
    }
    write_float_raster(&raster, root, out, PORTAL)?;
    let centre = TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    };
    let date = match listed.iter().find(|t| t.tile == centre) {
        Some(entry) if entry.date.is_none() => {
            warnings.push(format!(
                "the Downloadportal gives no date for the {} tile {}",
                model.label(),
                short_name(centre)
            ));
            None
        }
        entry => entry.and_then(|e| e.date.clone()),
    };
    Ok(HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    })
}

/// `32_565_5949`.
fn short_name(tile: TileId) -> String {
    format!("32_{:03}_{:04}", tile.east_km, tile.north_km)
}

/// The overview of `product` over `bbox`, kept in `dir` for offline runs (and for an online
/// run whose overview request fails, with a warning).
fn tile_list(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    bbox: &BoundingBox,
    product: Product,
    warnings: &mut Vec<String>,
) -> Result<Vec<PortalTile>, FetchError> {
    let url = overview_url(bbox, product);
    let path = dir.join("overview").join(format!(
        "{}_{}.json",
        product.overview_type(),
        bbox_key(bbox)
    ));
    let cached = || {
        std::fs::read(&path)
            .ok()
            .and_then(|bytes| parse_overview(&url, &bytes, product).ok())
    };
    if mode == FetchMode::Offline {
        return cached().ok_or_else(|| FetchError::Http {
            url: url.clone(),
            message: format!(
                "offline and not cached: the tile list of the {} for this area; run \
                 `freemapper fetch` online",
                product.label()
            ),
        });
    }
    let answer = get_with_retries(fetcher, &url, |bytes| Ok(bytes.to_vec())).and_then(|bytes| {
        let tiles = parse_overview(&url, &bytes, product)?;
        Ok((tiles, bytes))
    });
    match answer {
        Ok((tiles, bytes)) => {
            write_cache_file(&path, &bytes)?;
            Ok(tiles)
        }
        // Unreachable or unreadable (an error page, broken JSON): the list kept from an
        // earlier run of this area, if there is one.
        Err(e) => {
            let Some(tiles) = cached() else {
                return Err(e);
            };
            warnings.push(format!(
                "the Downloadportal's tile list of the {} could not be read ({e}); used the one \
                 kept from an earlier run",
                product.label()
            ));
            Ok(tiles)
        }
    }
}

/// The cache file of every tile named by the tile lists kept from earlier runs (of any area;
/// the file name carries the vintage). The lists are merged oldest first, so a tile takes the
/// name of the newest list that names it: a tile only another project's list names still
/// counts, and an older vintage does not hide a newer one. Used to tell what a fetch would
/// download.
pub fn cached_tile_files(
    cache: &Cache,
    model: Model,
) -> std::collections::HashMap<TileId, PathBuf> {
    let dir = cache.lvermgeo_sh_dir();
    let prefix = format!("{}_", model.overview_type());
    let mut lists: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir.join("overview"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    lists.sort();
    lists
        .into_iter()
        .filter_map(|(_, path)| std::fs::read(path).ok())
        .filter_map(|bytes| parse_overview("", &bytes, model).ok())
        .flatten()
        .map(|entry| (entry.tile, cache_path(&dir, model, &entry)))
        .collect()
}

/// Where the 1 m tile of `entry` is cached; named after the portal's (checked) name.
fn cache_path(dir: &Path, model: Model, entry: &PortalTile) -> PathBuf {
    match model {
        Model::Dgm1 => dir.join("dgm1").join(format!("{}.tif", entry.name)),
        Model::Bdom => dir.join("bdom").join(format!("{}_1m.tif", entry.name)),
    }
}

/// The cached tile at `path`, if it is there and reads as `tile`.
pub(crate) fn read_cached(path: &Path, tile: TileId) -> Option<Raster<f32>> {
    let bytes = std::fs::read(path).ok()?;
    decode_float_tiff(&bytes)
        .and_then(|raster| whole_tile(raster, tile, &[1.0]))
        .ok()
}

/// The pixel size of the bDOM tiles; test builds also take coarser synthetic ones, so a test
/// tile need not be 5000 x 5000.
#[cfg(not(test))]
const BDOM_PIXELS: &[f64] = &[0.2];
#[cfg(test)]
const BDOM_PIXELS: &[f64] = &[0.2, 0.5, 1.0];

/// `raster` if it covers exactly the 1 km `tile` with pixels of one of the `pixels` sizes.
pub(crate) fn whole_tile(
    raster: Raster<f32>,
    tile: TileId,
    pixels: &[f64],
) -> Result<Raster<f32>, String> {
    let g = raster.grid;
    let near = |a: f64, b: f64| (a - b).abs() < 1e-3;
    if !pixels.iter().any(|&p| (g.pixel_size - p).abs() < 1e-9) {
        let wanted: Vec<String> = pixels.iter().map(|p| format!("{p} m")).collect();
        return Err(format!(
            "pixels of {} m, not {}",
            g.pixel_size,
            wanted.join(" or ")
        ));
    }
    let fits = near(g.west, f64::from(tile.east_km) * 1000.0)
        && near(g.north, f64::from(tile.north_km + 1) * 1000.0)
        && near(f64::from(g.width) * g.pixel_size, 1000.0)
        && near(f64::from(g.height) * g.pixel_size, 1000.0);
    if fits {
        Ok(raster)
    } else {
        Err(format!(
            "the raster (W {} N {}, {} x {} px of {} m) is not the whole tile {}",
            g.west,
            g.north,
            g.width,
            g.height,
            g.pixel_size,
            short_name(tile)
        ))
    }
}

/// The DGM1 of `entry` from the ATOM directory, or from the portal's XYZ when the ATOM answer
/// is not a valid whole 1 m tile (404, an error page, a file that does not decode, another
/// grid, a server error that stays after the retries); cached. The fallback comes with a
/// warning naming the tile and the reason; when the XYZ fails too, the error names both.
fn dgm1_tile(
    fetcher: &dyn Fetcher,
    dir: &Path,
    entry: &PortalTile,
    xyz_cells: u32,
    progress: &mut dyn FnMut(Note),
) -> Result<(Raster<f32>, Option<String>), FetchError> {
    let path = cache_path(dir, Model::Dgm1, entry);
    let what = format!("{} tile {}", Model::Dgm1.label(), entry.name);
    let url = format!("{ATOM_DATA}/{}.tif", entry.name);
    let atom = get_with_retries(fetcher, &url, |bytes| {
        decode_float_tiff(bytes)
            .and_then(|raster| whole_tile(raster, entry.tile, &[1.0]))
            .map(|raster| (bytes.to_vec(), raster))
            .map_err(|message| FetchError::Http {
                url: url.clone(),
                message: format!("{what}: {message}"),
            })
    });
    let (atom_failed, atom_error) = match atom {
        Ok((bytes, raster)) => {
            write_cache_file(&path, &bytes)?;
            return Ok((raster, None));
        }
        Err(FetchError::NotFound(_)) => ("is not in the INSPIRE ATOM directory".to_string(), None),
        Err(e) => (
            format!("from the INSPIRE ATOM directory does not read ({e})"),
            Some(Arg::error(e)),
        ),
    };
    // The line names the same two cases as `atom_failed`, which the error below quotes.
    progress(match atom_error.clone() {
        None => note!(
            "progress-sh-dgm-not-in-atom",
            tile = short_name(entry.tile),
            megabytes = XYZ_MB.to_string()
        ),
        Some(error) => note!(
            "progress-sh-dgm-atom-unreadable",
            tile = short_name(entry.tile),
            error = error,
            megabytes = XYZ_MB.to_string()
        ),
    });
    let tile = format!("DGM1 tile {}", short_name(entry.tile));
    let raster = download_checked(
        fetcher,
        dir,
        entry,
        &what,
        XYZ_MAX_BYTES,
        progress,
        |reader| parse_xyz(reader, entry.tile, xyz_cells),
    )
    .map_err(|e| FetchError::Http {
        url: entry.link.clone(),
        message: format!("{what} {atom_failed}, and its XYZ does not read either: {e}"),
    })?;
    cache_raster(&path, &raster, &what)?;
    Ok((
        raster,
        Some(format!(
            "{tile} {atom_failed}; used its XYZ from the Downloadportal"
        )),
    ))
}

/// The bDOM of `entry`, streamed to a temporary file, decoded once, reduced to 1 m and cached.
fn bdom_tile(
    fetcher: &dyn Fetcher,
    dir: &Path,
    entry: &PortalTile,
    progress: &mut dyn FnMut(Note),
) -> Result<Raster<f32>, FetchError> {
    let what = format!("{} tile {}", Model::Bdom.label(), entry.name);
    let raster = download_checked(
        fetcher,
        dir,
        entry,
        &what,
        BDOM_MAX_BYTES,
        progress,
        |reader| {
            let start = reader.fill_buf().map_err(|e| e.to_string())?;
            if !(start.starts_with(b"II*\0") || start.starts_with(b"MM\0*")) {
                return Err(error_page(start));
            }
            // The 0.2 m pixels (100 MB) live only until they are reduced.
            let full = decode_float_tiff_from(reader)
                .and_then(|r| whole_tile(r, entry.tile, BDOM_PIXELS))?;
            reduce_to_metre(&full)
        },
    )?;
    cache_raster(&cache_path(dir, Model::Bdom, entry), &raster, &what)?;
    Ok(raster)
}

/// What an answer that is not data says: the portal reports errors as an HTML page with
/// status 200.
fn error_page(start: &[u8]) -> String {
    if start.is_empty() {
        return "the Downloadportal sent an empty answer".to_string();
    }
    let head = &start[..start.len().min(500)];
    let text = String::from_utf8_lossy(head);
    // Tags out, so the message reads as the portal's sentence.
    let mut plain = String::new();
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                plain.push(' ');
            }
            c if !in_tag => plain.push(c),
            _ => {}
        }
    }
    match excerpt(&plain) {
        text if text.is_empty() => {
            "the Downloadportal answered with an error page without text".to_string()
        }
        text => format!("the Downloadportal answered with an error page: {text}"),
    }
}

/// Downloads made of a tile whose download does not read: the portal sends no Content-Length,
/// so a body that ends early but cleanly only shows when it is read.
const DOWNLOADS: u32 = 2;

/// Downloads `entry.link` (at most `max_bytes`) into a temporary file in `dir` (removed
/// afterwards, also on error) and reads it with `read`, downloading it once more if it does not
/// read; an error names `what`. Downloads a killed run left behind are removed first.
fn download_checked<T>(
    fetcher: &dyn Fetcher,
    dir: &Path,
    entry: &PortalTile,
    what: &str,
    max_bytes: u64,
    progress: &mut dyn FnMut(Note),
    read: impl Fn(&mut BufReader<std::fs::File>) -> Result<T, String>,
) -> Result<T, FetchError> {
    let downloads = dir.join("downloads");
    freemapper_fs::remove_stale_cache_parts(&downloads, STALE_PART_AGE);
    let temp = TempFile(downloads.join(format!("{}.{}.part", entry.name, std::process::id())));
    let tile = short_name(entry.tile);
    let mut attempt = 1;
    loop {
        download_with_retries(fetcher, &entry.link, &temp.0, max_bytes, &mut |m| {
            progress(note!("progress-sh-tile", tile = &tile, text = m))
        })?;
        let file = std::fs::File::open(&temp.0).map_err(|e| FetchError::Io(temp.0.clone(), e))?;
        let message = match read(&mut BufReader::with_capacity(1 << 16, file)) {
            Ok(value) => return Ok(value),
            Err(message) => message,
        };
        if attempt == DOWNLOADS {
            return Err(FetchError::Http {
                url: entry.link.clone(),
                message: format!("{what}: {message} (downloaded twice)"),
            });
        }
        progress(note!(
            "progress-sh-download-again",
            tile = &tile,
            message = message
        ));
        attempt += 1;
    }
}

/// A file removed when dropped.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn cache_raster(path: &Path, raster: &Raster<f32>, what: &str) -> Result<(), FetchError> {
    let bytes = encode_float_geotiff(raster).map_err(|message| FetchError::Http {
        url: PORTAL.to_string(),
        message: format!("{what}: {message}"),
    })?;
    write_cache_file(path, &bytes)
}

/// A DGM1 XYZ of the portal: `cells` x `cells` lines `565000.50 5949999.50 29.85`, the cell
/// centres of `tile` row by row from the north-west; CRLF, a stray `\n` at the end. Every point
/// must be where the grid puts it. The tile comes out at 1 m: production XYZ have 1000 cells,
/// a test XYZ with fewer fills a block of pixels per cell.
fn parse_xyz(reader: &mut impl BufRead, tile: TileId, cells: u32) -> Result<Raster<f32>, String> {
    if cells == 0 || !1000_u32.is_multiple_of(cells) {
        return Err(format!("{cells} cells do not divide the tile"));
    }
    let start = reader.fill_buf().map_err(|e| e.to_string())?;
    if !start.first().is_some_and(u8::is_ascii_digit) {
        return Err(error_page(start));
    }
    let pixel = 1000.0 / f64::from(cells);
    let grid = GridSpec {
        west: f64::from(tile.east_km) * 1000.0,
        north: f64::from(tile.north_km + 1) * 1000.0,
        pixel_size: pixel,
        width: cells,
        height: cells,
    };
    let total = cells as usize * cells as usize;
    let mut data = Vec::with_capacity(total);
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            break;
        }
        let text = line.trim_end();
        if text.is_empty() {
            continue;
        }
        let n = data.len();
        if n == total {
            return Err(format!("more than {total} points"));
        }
        let mut fields = text.split_ascii_whitespace().map(str::parse::<f64>);
        let (Some(Ok(e)), Some(Ok(north)), Some(Ok(z)), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(format!(
                "point {} is not `E N height`: {}",
                n + 1,
                excerpt(text)
            ));
        };
        let (col, row) = ((n % cells as usize) as f64, (n / cells as usize) as f64);
        let (want_e, want_n) = (
            grid.west + (col + 0.5) * pixel,
            grid.north - (row + 0.5) * pixel,
        );
        if (e - want_e).abs() > 0.01 || (north - want_n).abs() > 0.01 {
            return Err(format!(
                "point {} is at E {e:.2} N {north:.2}, expected E {want_e:.2} N {want_n:.2} \
                 (the cells of the tile row by row from the north-west)",
                n + 1
            ));
        }
        data.push(if z < -9998.0 { f32::NAN } else { z as f32 });
    }
    if data.len() != total {
        return Err(format!("{} of {total} points", data.len()));
    }
    let k = (1000 / cells) as usize;
    if k > 1 {
        data = (0..1000 * 1000)
            .map(|i| data[i / 1000 / k * cells as usize + i % 1000 / k])
            .collect();
    }
    Ok(Raster {
        grid: GridSpec {
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
            ..grid
        },
        data,
    })
}

/// `raster` (pixels that divide 1 m, e.g. the bDOM's 0.2 m) at 1 m: the median of each block,
/// NaN ignored; a block without any height is NaN.
fn reduce_to_metre(raster: &Raster<f32>) -> Result<Raster<f32>, String> {
    let g = raster.grid;
    let k = (1.0 / g.pixel_size).round();
    if k < 1.0 || (k * g.pixel_size - 1.0).abs() > 1e-6 {
        return Err(format!("pixels of {} m do not divide 1 m", g.pixel_size));
    }
    let k = k as u32;
    if !g.width.is_multiple_of(k) || !g.height.is_multiple_of(k) {
        return Err(format!(
            "{} x {} px of {} m are not whole metres",
            g.width, g.height, g.pixel_size
        ));
    }
    let (width, height) = (g.width / k, g.height / k);
    let mut data = Vec::with_capacity(width as usize * height as usize);
    let mut block = Vec::with_capacity((k * k) as usize);
    for by in 0..height {
        for bx in 0..width {
            block.clear();
            for y in by * k..(by + 1) * k {
                let row = &raster.data[(y * g.width) as usize..][..g.width as usize];
                block.extend(
                    row[(bx * k) as usize..((bx + 1) * k) as usize]
                        .iter()
                        .copied()
                        .filter(|z| !z.is_nan()),
                );
            }
            data.push(median(&mut block));
        }
    }
    Ok(Raster {
        grid: GridSpec {
            pixel_size: 1.0,
            width,
            height,
            ..g
        },
        data,
    })
}

/// The median of `values` (the mean of the middle two for an even count); NaN for none.
fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let (len, mid) = (values.len(), values.len() / 2);
    let (lower, &mut upper, _) = values.select_nth_unstable_by(mid, f32::total_cmp);
    if !len.is_multiple_of(2) {
        return upper;
    }
    let below = lower.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    (below + upper) / 2.0
}

/// Pastes the 1 m `part` into the 1 m `raster`; returns how many pasted cells have no height.
pub(crate) fn paste_tile(raster: &mut Raster<f32>, part: &Raster<f32>) -> Result<u64, String> {
    let empty = std::cell::Cell::new(0);
    raster
        .paste_with(&part.grid, |x, y| {
            let z = part.get(x, y);
            if z.is_nan() {
                empty.set(empty.get() + 1);
            }
            z
        })
        .map_err(|e| e.to_string())?;
    Ok(empty.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const DGM1: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-dgm1.json");
    const BDOM: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-bdom.json");
    const TOO_MANY: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-too-many.json");

    /// The Coppernicus-Gymnasium in Norderstedt.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 565_100.0,
        min_n: 5_949_100.0,
        max_e: 565_300.0,
        max_n: 5_949_300.0,
    };

    const TILE: TileId = TileId {
        east_km: 565,
        north_km: 5949,
    };

    fn url(product: &str) -> String {
        format!(
            "{PORTAL}/_ajax/overview.php?type[]={product}&bbox=565100,5949100,565300,5949300\
             &crs=EPSG:25832"
        )
    }

    fn serving(product: &str, body: &str) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(&url(product), body.as_bytes().to_vec());
        fetcher
    }

    const DOP_INFO: &[u8] =
        include_bytes!("../../../../tests/fixtures/schleswig-holstein-dop-info.txt");

    #[test]
    fn flight_date_from_get_feature_info() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{WMS_INFO}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo"),
            DOP_INFO.to_vec(),
        );
        assert_eq!(
            flight_date(&fetcher, &SCHOOL).as_deref(),
            Some("17.03.2025")
        );
        let url = fetcher.log.borrow()[0].clone();
        for part in [
            "LAYERS=DOP20",
            "QUERY_LAYERS=DOP20",
            "CRS=EPSG:25832",
            "BBOX=565100,5949100,565300,5949300",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(url.contains(part), "{part} in {url}");
        }
        // Without A_DATUM2 the ISO date does.
        let iso = MapFetcher::default();
        iso.insert_prefix(
            WMS_INFO,
            b"@DOP20 FID;A_DATUM;E_DATUM;\n 1;2024-08-01;2024-09-01;\n".to_vec(),
        );
        assert_eq!(flight_date(&iso, &SCHOOL).as_deref(), Some("01.08.2024"));
        for body in [
            &b"Search returned no results.\n"[..],
            b"<ServiceExceptionReport/>",
            b"@DOP20 FID;A_DATUM2;\n 1;nonsense;\n",
        ] {
            let none = MapFetcher::default();
            none.insert_prefix(WMS_INFO, body.to_vec());
            assert_eq!(flight_date(&none, &SCHOOL), None, "{body:?}");
        }
        assert_eq!(flight_date(&MapFetcher::default(), &SCHOOL), None);
    }

    #[test]
    fn flight_date_takes_the_first_of_two_features() {
        // A point on a tile edge returns two tiles: the header once with two value rows, or
        // the header repeated per feature.
        let once = "@DOP20 FID;Shape;A_DATUM;A_DATUM2; 1;Polygon;2025-03-17;17.03.2025; \
                    2;Polygon;2024-08-01;01.08.2024;\n";
        let twice = "@DOP20 FID;Shape;A_DATUM;A_DATUM2; 1;Polygon;2025-03-17;17.03.2025;\n\
                     @DOP20 FID;Shape;A_DATUM;A_DATUM2; 2;Polygon;2024-08-01;01.08.2024;\n";
        for body in [once, twice] {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(WMS_INFO, body.as_bytes().to_vec());
            assert_eq!(
                flight_date(&fetcher, &SCHOOL).as_deref(),
                Some("17.03.2025"),
                "{body}"
            );
        }
    }

    #[test]
    fn overview_lists_dgm1_and_bdom_tiles() {
        let dgm1 = overview(&serving("dgm1", DGM1), &SCHOOL, Product::Dgm1).unwrap();
        assert_eq!(
            dgm1.iter().map(|t| t.tile).collect::<Vec<_>>(),
            [
                TILE,
                TileId {
                    east_km: 565,
                    north_km: 5950
                },
                TileId {
                    east_km: 566,
                    north_km: 5949
                },
            ]
        );
        assert_eq!(
            dgm1[0],
            PortalTile {
                tile: TILE,
                name: "dgm1_32_565_5949_1_sh_2022".to_string(),
                year: 2022,
                date: Some("2022-03-10".to_string()),
                link: format!(
                    "{PORTAL}/massen.php?file=dgm1_32_565_5949_1_sh_2022.xyz&id=2&live=2022\
                     &km=32560_5940"
                ),
            }
        );
        // The 10 km block follows the tile.
        assert!(dgm1[1].link.ends_with("&km=32560_5950"), "{}", dgm1[1].link);

        let bdom = overview(&serving("bdom", BDOM), &SCHOOL, Product::Bdom).unwrap();
        assert_eq!(
            bdom,
            [PortalTile {
                tile: TILE,
                // The overview says `bdom20rgbi_…`; the file is `bdom20nc_…`.
                name: "bdom20nc_32_565_5949_1_sh_2023".to_string(),
                year: 2023,
                date: Some("2023-04-07".to_string()),
                link: format!(
                    "{PORTAL}/massen.php?file=bdom20nc_32_565_5949_1_sh_2023.tif&id=6&live=2023\
                     &km=32560_5940&stack=32565_5949"
                ),
            }]
        );
    }

    #[test]
    fn too_many_objects_is_an_error() {
        let err = overview(&serving("dgm1", TOO_MANY), &SCHOOL, Product::Dgm1).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("at most 200 tiles"), "{message}");
        assert!(message.contains("reduce the map area"), "{message}");
    }

    #[test]
    fn html_answer_is_an_error() {
        let html = "Folgender Datensatz: foo konnte nicht heruntergeladen werden. <br> Die \
                    verwendete Massendownload-Datei ist veraltet.";
        let err = overview(&serving("bdom", html), &SCHOOL, Product::Bdom).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("not a tile overview"), "{message}");
        assert!(message.contains("Folgender Datensatz"), "{message}");
    }

    /// `fixture` with the first feature's `key` set to `value`.
    fn with_property(fixture: &str, key: &str, value: Value) -> String {
        let mut json: Value = serde_json::from_str(fixture).unwrap();
        json["result"]["features"][0]["properties"][key] = value;
        json.to_string()
    }

    #[test]
    fn unsafe_names_are_refused() {
        let evil = "x&file=../../etc";
        for (product, fixture, keys) in [
            (
                Product::Dgm1,
                DGM1,
                &["title", "jahr", "kaname", "filepath"][..],
            ),
            (
                Product::Bdom,
                BDOM,
                &["title", "jahr", "kachel_n", "kach10km", "filepath"][..],
            ),
        ] {
            let name = product.overview_type();
            for key in keys {
                let answer = with_property(fixture, key, Value::from(evil));
                let err = overview(&serving(name, &answer), &SCHOOL, product).unwrap_err();
                assert!(
                    err.to_string().contains(&format!("`{key}`")),
                    "{key}: {err}"
                );
            }
            // A number that is not a whole year.
            let answer = with_property(fixture, "jahr", Value::from(2022.5));
            let err = overview(&serving(name, &answer), &SCHOOL, product).unwrap_err();
            assert!(err.to_string().contains("`jahr`"), "{err}");
        }
    }

    #[test]
    fn control_characters_are_not_echoed() {
        let answer = "\x1b[31mFehler\x1b[0m\x07 beim\r\nAbruf";
        let err = overview(&serving("dgm1", answer), &SCHOOL, Product::Dgm1).unwrap_err();
        let message = err.to_string();
        assert!(!message.chars().any(char::is_control), "{message:?}");
        assert!(message.contains("[31mFehler"), "{message}");
        assert!(message.contains("beim Abruf"), "{message}");
    }

    #[test]
    fn features_of_another_type_are_an_error() {
        // A DGM1 answer to a bDOM request: the portal's `type` no longer matches.
        let err = overview(&serving("bdom", DGM1), &SCHOOL, Product::Bdom).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("none of type `bdom`"), "{message}");
        // No features at all is an empty area, not an error.
        let empty = DGM1.replace(
            &DGM1[DGM1.find("[{").unwrap()..DGM1.rfind("}]").unwrap() + 2],
            "[]",
        );
        let tiles = overview(&serving("dgm1", &empty), &SCHOOL, Product::Dgm1).unwrap();
        assert!(tiles.is_empty());
    }

    #[test]
    fn unknown_bdom_name_is_an_error() {
        let answer = BDOM.replace("bdom20rgbi_", "bdom40rgb_");
        let err = overview(&serving("bdom", &answer), &SCHOOL, Product::Bdom).unwrap_err();
        assert!(err.to_string().contains("bdom40rgb_32_565_5949"), "{err}");
        // The file name itself is accepted as it is.
        let answer = BDOM.replace("bdom20rgbi_", "bdom20nc_");
        let tiles = overview(&serving("bdom", &answer), &SCHOOL, Product::Bdom).unwrap();
        assert_eq!(tiles[0].name, "bdom20nc_32_565_5949_1_sh_2023");
    }

    #[test]
    #[ignore = "network"]
    fn live_overview_lists_the_school_tile() {
        let fetcher = crate::fetch::HttpFetcher::new();
        for product in [Product::Dgm1, Product::Bdom] {
            let tiles = overview(&fetcher, &SCHOOL, product).unwrap();
            assert_eq!(
                tiles.iter().map(|t| t.tile).collect::<Vec<_>>(),
                [TILE],
                "{product:?}"
            );
            assert!(tiles[0].year >= 2022, "{tiles:?}");
        }
    }

    /// DGM1 and bDOM tiles (Task 4).
    mod model {
        use super::*;
        use crate::fetch::{Cache, FetchMode};
        use crate::raster::{GridSpec, Raster};
        use crate::sources::float_tiff::decode_float_tiff;

        const DGM1_TIF: &[u8] =
            include_bytes!("../../../../tests/fixtures/schleswig-holstein-dgm1-nw.tif");
        const DGM1_NAME: &str = "dgm1_32_565_5949_1_sh_2022";
        const BDOM_NAME: &str = "bdom20nc_32_565_5949_1_sh_2023";

        fn atom_url(name: &str) -> String {
            format!("{ATOM_DATA}/{name}.tif")
        }

        fn link(fixture: &str, product: Product) -> String {
            parse_overview("fixture", fixture.as_bytes(), product).unwrap()[0]
                .link
                .clone()
        }

        /// An XYZ like the portal's: `cells` x `cells` points, row-major from the north-west,
        /// CRLF lines and one stray `\n` at the end.
        fn xyz(tile: TileId, cells: u32, height: impl Fn(u32, u32) -> f32) -> Vec<u8> {
            let pixel = 1000.0 / f64::from(cells);
            let mut text = String::new();
            for row in 0..cells {
                for col in 0..cells {
                    let e = f64::from(tile.east_km) * 1000.0 + (f64::from(col) + 0.5) * pixel;
                    let n = f64::from(tile.north_km + 1) * 1000.0 - (f64::from(row) + 0.5) * pixel;
                    text.push_str(&format!("{e:.2} {n:.2} {:.2}\r\n", height(col, row)));
                }
            }
            text.push('\n');
            text.into_bytes()
        }

        fn run(
            fetcher: &dyn Fetcher,
            cache: &Cache,
            mode: FetchMode,
            model: Model,
            bbox: &BoundingBox,
            out: &Path,
            messages: &mut Vec<String>,
        ) -> Result<HeightModel, FetchError> {
            fetch_model_with(
                fetcher,
                cache,
                mode,
                model,
                bbox,
                out.parent().unwrap(),
                out,
                &mut |m| messages.push(m.to_string()),
                10,
            )
        }

        fn read(out: &Path) -> Raster<f32> {
            let bytes = std::fs::read(out).unwrap();
            let decoded = decode_float_tiff_plain(&bytes);
            let world = std::fs::read_to_string(out.with_extension("tfw")).unwrap();
            Raster {
                grid: GridSpec::from_world_file(&world, decoded.0, decoded.1).unwrap(),
                data: decoded.2,
            }
        }

        /// The mosaic `out` is written without GeoTIFF tags (its world file has the grid).
        fn decode_float_tiff_plain(bytes: &[u8]) -> (u32, u32, Vec<f32>) {
            let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
            let (width, height) = decoder.dimensions().unwrap();
            let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
                panic!("not float")
            };
            (width, height, values)
        }

        /// Files left in the cache folder, relative to it.
        fn cached(cache: &Cache) -> Vec<String> {
            let root = cache.lvermgeo_sh_dir();
            let mut files = Vec::new();
            let mut dirs = vec![root.clone()];
            while let Some(dir) = dirs.pop() {
                for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push(path);
                    } else {
                        files.push(
                            path.strip_prefix(&root)
                                .unwrap()
                                .to_string_lossy()
                                .replace('\\', "/"),
                        );
                    }
                }
            }
            files.sort();
            files
        }

        #[test]
        fn tiled_lzw_dgm1_decodes() {
            // The north-west 300 x 300 m of the school tile, its first 2 x 2 LZW tiles of
            // 256 px copied from the ATOM file (the right and bottom ones partly outside).
            let raster = decode_float_tiff(DGM1_TIF).unwrap();
            assert_eq!(
                raster.grid,
                GridSpec {
                    west: 565_000.0,
                    north: 5_950_000.0,
                    pixel_size: 1.0,
                    width: 300,
                    height: 300,
                }
            );
            // The same cells in the portal's XYZ (2 decimals).
            for (x, y, z) in [
                (0, 0, 29.85),
                (299, 0, 30.81),
                (0, 299, 31.02),
                (299, 299, 29.93),
                (255, 255, 29.77),
                (256, 256, 29.83),
                (100, 270, 28.87),
                (270, 100, 30.92),
            ] {
                let got = raster.get(x, y);
                assert!((got - z).abs() < 0.006, "({x}, {y}): {got} vs {z}");
            }
            assert!(raster.data.iter().all(|z| z.is_finite()));
        }

        #[test]
        fn missing_atom_tile_falls_back_to_xyz() {
            let fetcher = serving("dgm1", DGM1);
            // No ATOM file (404); the XYZ: 10 x 10 cells of 100 m, 20 m + 1 m per cell east.
            fetcher.insert(
                &link(DGM1, Product::Dgm1),
                xyz(TILE, 10, |col, row| 20.0 + col as f32 + row as f32 * 0.1),
            );
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let out = dir.path().join("dgm1.tif");
            let mut messages = Vec::new();
            let found = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut messages,
            )
            .unwrap();
            assert_eq!(
                found,
                HeightModel {
                    date: Some("2022-03-10".to_string()),
                    warnings: vec![
                        "DGM1 tile 32_565_5949 is not in the INSPIRE ATOM directory; used its \
                         XYZ from the Downloadportal"
                            .to_string()
                    ],
                    ..HeightModel::default()
                }
            );
            // The ATOM file was asked for first, then the XYZ.
            let log = fetcher.log.borrow().clone();
            let atom = log.iter().position(|u| *u == atom_url(DGM1_NAME)).unwrap();
            let xyz_at = log
                .iter()
                .position(|u| *u == link(DGM1, Product::Dgm1))
                .unwrap();
            assert!(atom < xyz_at, "{log:?}");
            assert!(
                messages
                    .iter()
                    .any(|m| m.contains("not in the INSPIRE ATOM")),
                "{messages:?}"
            );
            let raster = read(&out);
            assert_eq!(raster.grid, GridSpec::covering(&SCHOOL, 1.0));
            // E 565100 is cell column 1, N 5949300 cell row 7 (from the north).
            assert!(
                (raster.get(0, 0) - 21.7).abs() < 1e-4,
                "{}",
                raster.get(0, 0)
            );
            assert!((raster.get(199, 199) - 22.8).abs() < 1e-4);
            // Cached under the portal's name, as a GeoTIFF; the download is gone.
            assert_eq!(
                cached(&cache),
                [
                    format!("dgm1/{DGM1_NAME}.tif"),
                    "overview/dgm1_565100_5949100_565300_5949300.json".to_string(),
                ]
            );
            // Offline, the cached tile and tile list give the same model without a request.
            let calls = fetcher.calls.get();
            let again = dir.path().join("again.tif");
            let offline = run(
                &fetcher,
                &cache,
                FetchMode::Offline,
                Model::Dgm1,
                &SCHOOL,
                &again,
                &mut messages,
            )
            .unwrap();
            assert_eq!(
                offline,
                HeightModel {
                    warnings: Vec::new(),
                    ..found
                }
            );
            assert_eq!(fetcher.calls.get(), calls);
            assert_eq!(read(&again), raster);
        }

        #[test]
        fn xyz_of_the_wrong_grid_is_an_error() {
            let fetcher = serving("dgm1", DGM1);
            let link = link(DGM1, Product::Dgm1);
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let out = dir.path().join("dgm1.tif");
            let full = xyz(TILE, 10, |_, _| 20.0);
            let half = full[..50 * 28].to_vec();
            for (body, expected) in [
                (half, "50 of 100 points"),
                (
                    xyz(
                        TileId {
                            east_km: 566,
                            north_km: 5949,
                        },
                        10,
                        |_, _| 20.0,
                    ),
                    "expected E 565050.00 N 5949950.00",
                ),
            ] {
                fetcher.insert(&link, body);
                let err = run(
                    &fetcher,
                    &cache,
                    FetchMode::Online,
                    Model::Dgm1,
                    &SCHOOL,
                    &out,
                    &mut Vec::new(),
                )
                .unwrap_err()
                .to_string();
                assert!(err.contains(expected), "{err}");
                assert!(err.contains(DGM1_NAME), "{err}");
            }
            assert_eq!(
                cached(&cache),
                ["overview/dgm1_565100_5949100_565300_5949300.json"]
            );
        }

        #[test]
        fn html_answer_is_an_error() {
            let html = b"<html><body>Folgender Datensatz: x konnte nicht heruntergeladen \
                         werden.</body></html>"
                .to_vec();
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let out = dir.path().join("model.tif");
            for (product, fixture, name) in [("dgm1", DGM1, DGM1_NAME), ("bdom", BDOM, BDOM_NAME)] {
                let fetcher = serving(product, fixture);
                let model = if product == "dgm1" {
                    Model::Dgm1
                } else {
                    Model::Bdom
                };
                // DGM1: no ATOM file, and the XYZ link answers an error page (status 200).
                fetcher.insert(&link(fixture, model), html.clone());
                let err = run(
                    &fetcher,
                    &cache,
                    FetchMode::Online,
                    model,
                    &SCHOOL,
                    &out,
                    &mut Vec::new(),
                )
                .unwrap_err()
                .to_string();
                assert!(err.contains(name), "{err}");
                assert!(err.contains("error page"), "{err}");
                assert!(err.contains("Folgender Datensatz"), "{err}");
                assert!(!out.exists());
            }
            // Nothing but the tile lists is cached, and no download is left behind.
            assert_eq!(
                cached(&cache),
                [
                    "overview/bdom_565100_5949100_565300_5949300.json",
                    "overview/dgm1_565100_5949100_565300_5949300.json",
                ]
            );
        }

        /// A 10 x 10 px tile of 0.2 m (2 x 2 m).
        fn small_tile(values: Vec<f32>) -> Raster<f32> {
            Raster {
                grid: GridSpec {
                    west: 565_000.0,
                    north: 5_950_000.0,
                    pixel_size: 0.2,
                    width: 10,
                    height: 10,
                },
                data: values,
            }
        }

        #[test]
        fn bdom_is_reduced_to_one_metre() {
            let mut values = vec![0.0; 100];
            for y in 0..10 {
                for x in 0..10 {
                    let block = (y / 5) * 2 + x / 5;
                    values[y * 10 + x] = match block {
                        // 25 heights 10..34 and one outlier: the median ignores it.
                        0 => 10.0 + ((y % 5) * 5 + x % 5) as f32,
                        // All NaN: no height.
                        1 => f32::NAN,
                        // 4 heights left (2, 4, 6, 100): the mean of the middle two.
                        2 => f32::NAN,
                        _ => 50.0,
                    };
                }
            }
            values[0] = 9_999.0;
            for (i, z) in [(50, 2.0), (51, 100.0), (60, 4.0), (61, 6.0)] {
                values[i] = z;
            }
            let reduced = reduce_to_metre(&small_tile(values)).unwrap();
            assert_eq!(
                reduced.grid,
                GridSpec {
                    west: 565_000.0,
                    north: 5_950_000.0,
                    pixel_size: 1.0,
                    width: 2,
                    height: 2,
                }
            );
            // 11..34 and 9999: the median is 23 (the mean would be 421.6).
            assert_eq!(reduced.get(0, 0), 23.0);
            assert!(reduced.get(1, 0).is_nan());
            assert_eq!(reduced.get(0, 1), 5.0);
            assert_eq!(reduced.get(1, 1), 50.0);
            // A pixel size that does not divide 1 m, or a grid not of whole metres, is refused.
            let mut odd = small_tile(vec![0.0; 100]);
            odd.grid.pixel_size = 0.3;
            assert!(reduce_to_metre(&odd).unwrap_err().contains("0.3"));
            let mut cut = small_tile(vec![0.0; 99]);
            cut.grid.height = 9;
            assert!(reduce_to_metre(&cut).is_err());
        }

        /// A bDOM tile of 0.5 m pixels: 40 m + 0.01 m per metre east, and in each 2 x 2 block
        /// one outlier of 1000 m (the median of 40, 40, 40, 1000 is 40).
        fn bdom_tile(tile: TileId) -> Vec<u8> {
            let grid = GridSpec {
                west: f64::from(tile.east_km) * 1000.0,
                north: f64::from(tile.north_km + 1) * 1000.0,
                pixel_size: 0.5,
                width: 2000,
                height: 2000,
            };
            let values: Vec<f32> = (0..2000 * 2000)
                .map(|i| {
                    let (x, y) = (i % 2000, i / 2000);
                    if x % 2 == 0 && y % 2 == 0 {
                        1000.0
                    } else {
                        40.0 + (x / 2) as f32 * 0.01
                    }
                })
                .collect();
            crate::fetch::test_support::geotiff(&grid, &values)
        }

        #[test]
        fn bdom_tile_is_streamed_reduced_and_cached() {
            let fetcher = serving("bdom", BDOM);
            fetcher.insert(&link(BDOM, Model::Bdom), bdom_tile(TILE));
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let out = dir.path().join("dom1.tif");
            let mut messages = Vec::new();
            let found = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &out,
                &mut messages,
            )
            .unwrap();
            assert_eq!(found.date.as_deref(), Some("2023-04-07"));
            assert!(
                found.missing.is_empty() && found.warnings.is_empty(),
                "{found:?}"
            );
            assert!(
                messages
                    .iter()
                    .any(|m| m == "downloading bDOM tile 32_565_5949 (105 MB, 1 of 1)"),
                "{messages:?}"
            );
            let raster = read(&out);
            assert_eq!(raster.grid, GridSpec::covering(&SCHOOL, 1.0));
            // E 565100: 40 m + 100 x 0.01 m.
            assert!(
                (raster.get(0, 0) - 41.0).abs() < 1e-4,
                "{}",
                raster.get(0, 0)
            );
            assert!((raster.get(199, 5) - 42.99).abs() < 1e-4);
            assert_eq!(
                cached(&cache),
                [
                    format!("bdom/{BDOM_NAME}_1m.tif"),
                    "overview/bdom_565100_5949100_565300_5949300.json".to_string(),
                ]
            );
            let tile = decode_float_tiff(
                &std::fs::read(
                    cache
                        .lvermgeo_sh_dir()
                        .join(format!("bdom/{BDOM_NAME}_1m.tif")),
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(tile.grid.pixel_size, 1.0);
            assert_eq!((tile.grid.width, tile.grid.height), (1000, 1000));
            // Online again: the cached 1 m tile is used, only the tile list is asked for.
            let before = fetcher.log.borrow().len();
            run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &out,
                &mut messages,
            )
            .unwrap();
            assert_eq!(fetcher.log.borrow()[before..], [url("bdom")]);
            assert_eq!(read(&out), raster);
        }

        #[test]
        fn missing_tiles_and_empty_cells_are_reported() {
            // Across 564/5949 (in Norderstedt, not listed) and 565/5949.
            let bbox = BoundingBox {
                min_e: 564_900.0,
                ..SCHOOL
            };
            let fetcher = MapFetcher::default();
            fetcher.insert(
                &overview_url(&bbox, Product::Bdom),
                BDOM.as_bytes().to_vec(),
            );
            // Every cell of 565/5949 is empty.
            let grid = GridSpec {
                west: 565_000.0,
                north: 5_950_000.0,
                pixel_size: 1.0,
                width: 1000,
                height: 1000,
            };
            let tile = crate::fetch::test_support::geotiff(&grid, &vec![f32::NAN; 1000 * 1000]);
            fetcher.insert(&link(BDOM, Model::Bdom), tile);
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let found = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &bbox,
                &dir.path().join("dom1.tif"),
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(
                found.missing,
                [TileId {
                    east_km: 564,
                    north_km: 5949
                }]
            );
            assert_eq!(found.warnings.len(), 1, "{:?}", found.warnings);
            assert!(
                found.warnings[0].contains("32_565_5949")
                    && found.warnings[0].contains("60000 cells"),
                "{:?}",
                found.warnings
            );
        }

        #[test]
        fn offline_without_cache_is_an_error() {
            let fetcher = MapFetcher::default();
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let out = dir.path().join("dgm1.tif");
            let err = run(
                &fetcher,
                &cache,
                FetchMode::Offline,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("offline and not cached"), "{err}");
            assert_eq!(fetcher.calls.get(), 0);
            // The tile list is cached, the tile is not.
            let online = serving("dgm1", DGM1);
            online.insert(
                &atom_url(DGM1_NAME),
                crate::sources::hvbg::test_support::tile_tiff(TILE, 20.0, 0.0),
            );
            run(
                &online,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut Vec::new(),
            )
            .unwrap();
            std::fs::remove_dir_all(cache.lvermgeo_sh_dir().join("dgm1")).unwrap();
            let err = run(
                &fetcher,
                &cache,
                FetchMode::Offline,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("offline and not cached"), "{err}");
            assert!(err.contains(DGM1_NAME), "{err}");
        }

        /// The DGM1 of the school tile with the ATOM directory answering `atom` (or failing
        /// with a server error) and the XYZ link answering `xyz`.
        fn dgm1_with(
            atom: Option<Vec<u8>>,
            xyz_body: Vec<u8>,
        ) -> (
            Result<HeightModel, FetchError>,
            Cache,
            tempfile::TempDir,
            MapFetcher,
        ) {
            let fetcher = serving("dgm1", DGM1);
            match atom {
                Some(body) => fetcher.insert(&atom_url(DGM1_NAME), body),
                None => fetcher.failing.borrow_mut().push(atom_url(DGM1_NAME)),
            }
            fetcher.insert(&link(DGM1, Product::Dgm1), xyz_body);
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let found = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &dir.path().join("dgm1.tif"),
                &mut Vec::new(),
            );
            (found, cache, dir, fetcher)
        }

        #[test]
        fn atom_tile_of_another_area_falls_back_to_xyz() {
            // The fixture covers only the north-west 300 m of the tile.
            let (found, cache, dir, _) = dgm1_with(
                Some(DGM1_TIF.to_vec()),
                xyz(TILE, 10, |col, _| 20.0 + col as f32),
            );
            let found = found.unwrap();
            assert_eq!(found.warnings.len(), 1, "{found:?}");
            let warning = &found.warnings[0];
            assert!(
                warning.starts_with(
                    "DGM1 tile 32_565_5949 from the INSPIRE ATOM directory does not read ("
                ) && warning.contains("not the whole tile")
                    && warning.ends_with("); used its XYZ from the Downloadportal"),
                "{warning}"
            );
            // The XYZ heights (E 565100 is cell column 1), cached under the portal's name.
            let raster = read(&dir.path().join("dgm1.tif"));
            assert!(
                (raster.get(0, 0) - 21.0).abs() < 1e-4,
                "{}",
                raster.get(0, 0)
            );
            assert!(cached(&cache).contains(&format!("dgm1/{DGM1_NAME}.tif")));
        }

        #[test]
        fn unreadable_atom_tile_falls_back_to_xyz() {
            let tile_xyz = || xyz(TILE, 10, |_, _| 20.0);
            for (atom, reason) in [
                // A file the decoder cannot read.
                (Some(b"II*\0broken".to_vec()), "does not read ("),
                // An error page with status 200.
                (Some(b"<html>Wartung</html>".to_vec()), "does not read ("),
                // A server error that stays after the retries.
                (None, "503 Service Unavailable"),
            ] {
                let (found, _cache, dir, fetcher) = dgm1_with(atom.clone(), tile_xyz());
                let found = found.unwrap_or_else(|e| panic!("{atom:?}: {e}"));
                assert_eq!(found.warnings.len(), 1, "{found:?}");
                assert!(
                    found.warnings[0].contains("32_565_5949")
                        && found.warnings[0].contains(reason)
                        && found.warnings[0].contains("used its XYZ"),
                    "{found:?}"
                );
                let raster = read(&dir.path().join("dgm1.tif"));
                assert_eq!(raster.get(10, 10), 20.0);
                // The ATOM file was asked for up to three times, then the XYZ once.
                let log = fetcher.log.borrow();
                let atom_asks = log.iter().filter(|u| **u == atom_url(DGM1_NAME)).count();
                assert_eq!(atom_asks, 3, "{log:?}");
            }
        }

        #[test]
        fn failing_atom_tile_and_xyz_name_both_reasons() {
            let (found, cache, _dir, _) = dgm1_with(
                Some(DGM1_TIF.to_vec()),
                b"<html><body>Folgender Datensatz: x</body></html>".to_vec(),
            );
            let err = found.unwrap_err().to_string();
            assert!(err.contains(DGM1_NAME), "{err}");
            assert!(err.contains("not the whole tile"), "{err}");
            assert!(err.contains("Folgender Datensatz"), "{err}");
            assert_eq!(
                cached(&cache),
                ["overview/dgm1_565100_5949100_565300_5949300.json"]
            );
        }

        #[test]
        fn synthetic_schleswig_holstein_serves_both_models() {
            let fetcher = MapFetcher::default();
            crate::fetch::test_support::serve_schleswig_holstein_models(&fetcher, &SCHOOL);
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let mut heights = Vec::new();
            for model in [Model::Dgm1, Model::Bdom] {
                let out = dir.path().join(format!("{model:?}.tif"));
                let found = fetch_model(
                    &fetcher,
                    &cache,
                    FetchMode::Online,
                    model,
                    &SCHOOL,
                    out.parent().unwrap(),
                    &out,
                    &mut |_| {},
                )
                .unwrap();
                assert!(found.missing.is_empty(), "{found:?}");
                assert!(found.date.is_some(), "{found:?}");
                heights.push(read(&out).get(10, 10));
            }
            assert!((heights[1] - heights[0] - 12.0).abs() < 1e-3, "{heights:?}");
        }

        /// A [`MapFetcher`] that answers some URLs from a queue first (one body per request).
        #[derive(Default)]
        struct Sequence {
            inner: MapFetcher,
            queue: std::cell::RefCell<
                std::collections::HashMap<String, std::collections::VecDeque<Vec<u8>>>,
            >,
        }

        impl Sequence {
            fn answer(&self, url: &str, bodies: Vec<Vec<u8>>) {
                self.queue
                    .borrow_mut()
                    .insert(url.to_string(), bodies.into());
            }
        }

        impl Fetcher for Sequence {
            fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
                let next = self
                    .queue
                    .borrow_mut()
                    .get_mut(url)
                    .and_then(|bodies| bodies.pop_front());
                match next {
                    Some(body) => {
                        self.inner.log.borrow_mut().push(url.to_string());
                        Ok(body)
                    }
                    None => self.inner.get(url),
                }
            }
        }

        fn cache_file(cache: &Cache, file: &str) -> PathBuf {
            cache.lvermgeo_sh_dir().join(file)
        }

        #[test]
        fn corrupt_cached_tile_is_downloaded_again() {
            let fetcher = serving("bdom", BDOM);
            fetcher.insert(&link(BDOM, Model::Bdom), bdom_tile(TILE));
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let file = cache_file(&cache, &format!("bdom/{BDOM_NAME}_1m.tif"));
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(&file, b"not a tiff").unwrap();
            let mut messages = Vec::new();
            run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &dir.path().join("dom1.tif"),
                &mut messages,
            )
            .unwrap();
            assert!(
                messages
                    .iter()
                    .any(|m| m == "downloading bDOM tile 32_565_5949 (105 MB, 1 of 1)"),
                "{messages:?}"
            );
            assert!(read_cached(&file, TILE).is_some());
        }

        #[test]
        fn dgm1_of_another_pixel_size_is_refused() {
            let fetcher = serving("dgm1", DGM1);
            let grid = GridSpec {
                west: 565_000.0,
                north: 5_950_000.0,
                pixel_size: 2.0,
                width: 500,
                height: 500,
            };
            fetcher.insert(
                &atom_url(DGM1_NAME),
                crate::fetch::test_support::geotiff(&grid, &vec![20.0; 500 * 500]),
            );
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let err = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &dir.path().join("dgm1.tif"),
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains(DGM1_NAME), "{err}");
            assert!(err.contains("pixels of 2 m, not 1 m"), "{err}");
            // A cached tile of the wrong size is not used either.
            std::fs::create_dir_all(cache_file(&cache, "dgm1")).unwrap();
            std::fs::write(
                cache_file(&cache, &format!("dgm1/{DGM1_NAME}.tif")),
                crate::fetch::test_support::geotiff(&grid, &vec![20.0; 500 * 500]),
            )
            .unwrap();
            assert!(
                read_cached(&cache_file(&cache, &format!("dgm1/{DGM1_NAME}.tif")), TILE).is_none()
            );
        }

        #[test]
        fn download_over_its_limit_is_refused_and_removed() {
            let fetcher = serving("bdom", BDOM);
            let entry = &parse_overview("fixture", BDOM.as_bytes(), Product::Bdom).unwrap()[0];
            fetcher.insert(&entry.link, bdom_tile(TILE));
            let dir = tempfile::tempdir().unwrap();
            let err = download_checked(
                &fetcher,
                dir.path(),
                entry,
                "bDOM tile",
                1000,
                &mut |_| {},
                |_| Ok(()),
            )
            .unwrap_err();
            assert!(matches!(err, FetchError::TooLarge { .. }), "{err}");
            let left: Vec<_> = std::fs::read_dir(dir.path().join("downloads"))
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.file_name())
                .collect();
            assert!(left.is_empty(), "{left:?}");
        }

        #[test]
        fn stale_downloads_of_killed_runs_are_removed() {
            let fetcher = serving("bdom", BDOM);
            fetcher.insert(&link(BDOM, Model::Bdom), bdom_tile(TILE));
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let downloads = cache.lvermgeo_sh_dir().join("downloads");
            std::fs::create_dir_all(&downloads).unwrap();
            let two_hours_ago =
                std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 3600);
            // A run killed during its download, and one still downloading.
            let killed = downloads.join(format!("{BDOM_NAME}.999999.part"));
            let running = downloads.join("bdom20nc_32_566_5949_1_sh_2023.888888.part");
            for path in [&killed, &running] {
                std::fs::write(path, b"partial").unwrap();
            }
            std::fs::File::options()
                .write(true)
                .open(&killed)
                .unwrap()
                .set_modified(two_hours_ago)
                .unwrap();
            run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &dir.path().join("dom1.tif"),
                &mut Vec::new(),
            )
            .unwrap();
            assert!(!killed.exists());
            assert!(running.exists());
        }

        #[test]
        fn short_download_is_downloaded_once_more() {
            let dir = tempfile::tempdir().unwrap();
            // bDOM: a body that ends early but cleanly, then the whole tile.
            let tile = bdom_tile(TILE);
            let fetcher = Sequence::default();
            fetcher.inner.insert(&url("bdom"), BDOM.as_bytes().to_vec());
            let bdom = link(BDOM, Model::Bdom);
            fetcher.answer(&bdom, vec![tile[..tile.len() / 2].to_vec(), tile.clone()]);
            let cache = Cache::new(dir.path().join("cache"));
            let mut messages = Vec::new();
            run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &dir.path().join("dom1.tif"),
                &mut messages,
            )
            .unwrap();
            assert!(
                messages
                    .iter()
                    .any(|m| m.contains("downloading it once more")),
                "{messages:?}"
            );
            // XYZ: short, then whole.
            fetcher.inner.insert(&url("dgm1"), DGM1.as_bytes().to_vec());
            let full = xyz(TILE, 10, |_, _| 20.0);
            let xyz_link = link(DGM1, Model::Dgm1);
            fetcher.answer(&xyz_link, vec![full[..50 * 28].to_vec(), full.clone()]);
            run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &dir.path().join("dgm1.tif"),
                &mut Vec::new(),
            )
            .unwrap();
            // Short twice: a clear error naming the tile, after two downloads.
            let other = Cache::new(dir.path().join("other"));
            fetcher.answer(
                &bdom,
                vec![tile[..100].to_vec(), tile[..tile.len() / 3].to_vec()],
            );
            let before = fetcher.inner.log.borrow().len();
            let err = run(
                &fetcher,
                &other,
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &dir.path().join("dom1.tif"),
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains(BDOM_NAME), "{err}");
            assert!(err.contains("downloaded twice"), "{err}");
            let log = fetcher.inner.log.borrow()[before..].to_vec();
            assert_eq!(log.iter().filter(|u| **u == bdom).count(), 2, "{log:?}");
        }

        #[test]
        fn unreadable_overview_falls_back_to_the_kept_list() {
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let html = b"<html><body>Wartungsarbeiten</body></html>".to_vec();
            let out = dir.path().join("dgm1.tif");
            // Without a kept list the error page is an error.
            let fetcher = serving("dgm1", std::str::from_utf8(&html).unwrap());
            let err = run(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("not a tile overview"), "{err}");
            // A successful run keeps the list ...
            let good = serving("dgm1", DGM1);
            good.insert(
                &atom_url(DGM1_NAME),
                crate::sources::hvbg::test_support::tile_tiff(TILE, 20.0, 0.0),
            );
            run(
                &good,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                &out,
                &mut Vec::new(),
            )
            .unwrap();
            // ... which an error page or broken JSON then falls back to, with a warning.
            for body in [&html[..], b"{\"success\":tr"] {
                let fetcher = MapFetcher::default();
                fetcher.insert(&url("dgm1"), body.to_vec());
                let found = run(
                    &fetcher,
                    &cache,
                    FetchMode::Online,
                    Model::Dgm1,
                    &SCHOOL,
                    &out,
                    &mut Vec::new(),
                )
                .unwrap();
                assert_eq!(found.warnings.len(), 1, "{found:?}");
                assert!(
                    found.warnings[0].contains("kept from an earlier run")
                        && found.warnings[0].contains("not a tile overview"),
                    "{found:?}"
                );
            }
        }

        #[test]
        fn empty_bdom_answer_is_named() {
            let fetcher = serving("bdom", BDOM);
            fetcher.insert(&link(BDOM, Model::Bdom), Vec::new());
            let dir = tempfile::tempdir().unwrap();
            let err = run(
                &fetcher,
                &Cache::new(dir.path().join("cache")),
                FetchMode::Online,
                Model::Bdom,
                &SCHOOL,
                &dir.path().join("dom1.tif"),
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("empty answer"), "{err}");
            assert!(!err.contains("error page"), "{err}");
            assert!(err.contains(BDOM_NAME), "{err}");
        }

        #[test]
        fn production_xyz_size_is_enforced() {
            let full = xyz(TILE, XYZ_CELLS, |col, row| 20.0 + (col + row) as f32 * 0.01);
            let raster = parse_xyz(&mut &full[..], TILE, XYZ_CELLS).unwrap();
            assert_eq!(
                raster.grid,
                GridSpec {
                    west: 565_000.0,
                    north: 5_950_000.0,
                    pixel_size: 1.0,
                    width: 1000,
                    height: 1000,
                }
            );
            assert!((raster.get(999, 999) - 39.98).abs() < 1e-4);
            // 999 lines short.
            let short = &full[..(1_000_000 - 999) * 28];
            let err = parse_xyz(&mut &short[..], TILE, XYZ_CELLS).unwrap_err();
            assert!(err.contains("999001 of 1000000 points"), "{err}");
        }

        #[test]
        #[ignore = "network: one 105 MB bDOM tile, about 90 s"]
        fn live_school_heights() {
            let fetcher = crate::fetch::HttpFetcher::new();
            let bbox = BoundingBox {
                min_e: 565_100.0,
                min_n: 5_949_150.0,
                max_e: 565_300.0,
                max_n: 5_949_350.0,
            };
            let dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(dir.path().join("cache"));
            let mut rasters = Vec::new();
            for model in [Model::Dgm1, Model::Bdom] {
                let out = dir.path().join(format!("{model:?}.tif"));
                let started = std::time::Instant::now();
                let found = fetch_model(
                    &fetcher,
                    &cache,
                    FetchMode::Online,
                    model,
                    &bbox,
                    out.parent().unwrap(),
                    &out,
                    &mut |m| eprintln!("{m}"),
                )
                .unwrap();
                eprintln!("{model:?}: {found:?} in {:.0?}", started.elapsed());
                rasters.push(read(&out));
            }
            let (dgm, dom) = (&rasters[0], &rasters[1]);
            // The tile's centre cell E 565214.5 / N 5949250.5 is 29.60 m.
            let (x, y) = (114, 99);
            eprintln!("DGM1 at the school {:.2} m", dgm.get(x, y));
            assert!((dgm.get(x, y) - 29.60).abs() < 0.01);
            // The main building wing (E 565150–565165, N 5949200–5949240) stands > 2 m.
            let mut wing: Vec<f32> = (110..150)
                .flat_map(|y| (50..65).map(move |x| (x, y)))
                .map(|(x, y)| dom.get(x, y) - dgm.get(x, y))
                .collect();
            wing.sort_by(f32::total_cmp);
            let median = wing[wing.len() / 2];
            eprintln!("bDOM - DGM1 on the main wing: median {median:.2} m");
            assert!(median > 2.0, "{median}");
            for (name, size) in cached(&cache).iter().map(|f| {
                let len = std::fs::metadata(cache.lvermgeo_sh_dir().join(f))
                    .unwrap()
                    .len();
                (f.clone(), len)
            }) {
                eprintln!("cached {name}: {size} B");
            }
        }
    }
}
