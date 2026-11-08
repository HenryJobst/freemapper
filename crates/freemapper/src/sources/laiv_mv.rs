//! LAiV Mecklenburg-Vorpommern open data (CC BY 4.0, "©GeoBasis-DE/MV"), all in EPSG:25833:
//! DGM1 and DOM1 from the MapServer WCS, masked to the 2 km tiles that the Downloadportal's
//! `overview.php` lists.
//!
//! The height models mark nothing as NoData: the sea, the Bodden and the Warnow are 0.0 m, and
//! the WCS fills areas without a tile (Brandenburg beyond the MV tiles, open sea) with 0.0 too.
//! So the tile set decides: cells outside it become NaN, zeros inside it stay.

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{
    Cache, FetchError, FetchMode, Fetcher, get_with_attempts, one_line, write_atomically,
};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::write_float_raster;
use crate::sources::height_model::HeightModel;
use crate::sources::region::{Region, contains_point};
use crate::sources::wcs::WcsService;
use crate::tiles::{TileId, tiles_covering_grid};

/// The Downloadportal's AJAX tile index.
pub const OVERVIEW: &str = "https://laiv.geodaten-mv.de/afgvk/_apps/dladownload/_ajax/overview.php";

/// The extent of `mv_dgm` and `mv_dom1` (DescribeCoverage: 265,000 x 189,000 cells of 1 m).
pub const ENVELOPE: BoundingBox = BoundingBox {
    min_e: 200_000.0,
    min_n: 5_886_000.0,
    max_e: 465_000.0,
    max_n: 6_075_000.0,
};

/// Edge of a portal tile in km.
const TILE_KM: u32 = 2;
/// Tries of an `overview.php` request: the portal answers a few requests with an empty body.
const OVERVIEW_ATTEMPTS: u32 = 5;
/// Largest side of one GetCoverage (the WCS answered 8000 px; 4000 px take about a minute).
const WCS_MAX_M: f64 = 4000.0;

/// A height model served by the WCS at 1 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain.
    Dgm1,
    /// DOM1, the (image-matched) surface.
    Dom1,
}

impl Model {
    /// The WCS of `self`: a plain LZW TIFF, cut to [`ENVELOPE`].
    pub fn wcs(self) -> WcsService {
        let (url, coverage) = match self {
            Self::Dgm1 => ("https://www.geodaten-mv.de/dienste/dgm_wcs", "mv_dgm"),
            Self::Dom1 => ("https://www.geodaten-mv.de/dienste/dom_wcs", "mv_dom1"),
        };
        WcsService {
            url,
            coverage,
            extra: "&GEOTIFF:COMPRESSION=LZW",
            multipart: false,
            envelope: ENVELOPE,
            max_m: WCS_MAX_M,
        }
    }

    /// The `type[]` of `overview.php` (`dom1` is an "Interner Fehler" there).
    pub fn overview_type(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LAiV M-V DGM1)",
            Self::Dom1 => "surface model (LAiV M-V DOM1)",
        }
    }
}

/// A tile as the portal lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedTile {
    /// The 2 km tile (south-west corner in km).
    pub tile: TileId,
    /// `aktualitaet`, `YYYY-MM-DD` (DGM1/DOM1: the first of the month).
    pub date: Option<String>,
}

/// The tile lookup of `model` over `bbox` (EPSG:25833), rounded outward to whole metres. The
/// type comes first, so a test server can answer each product by URL prefix.
pub(crate) fn overview_url(bbox: &BoundingBox, model: Model) -> String {
    format!(
        "{}{}&crs=EPSG:25833",
        overview_prefix(model),
        bbox_key(bbox).replace('_', ",")
    )
}

/// The start of every [`overview_url`] of `model`, up to the bbox.
pub(crate) fn overview_prefix(model: Model) -> String {
    format!("{OVERVIEW}?type[]={}&bbox=", model.overview_type())
}

/// `min_e_min_n_max_e_max_n`, rounded outward to whole metres.
fn bbox_key(bbox: &BoundingBox) -> String {
    format!(
        "{}_{}_{}_{}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// Why an `overview.php` answer was not read.
struct OverviewError {
    /// An empty or non-JSON body (the portal sends a few): asking again helps.
    retry: bool,
    error: FetchError,
}

/// An `overview.php` answer from `url`: the listed tiles of `model`, sorted.
fn parse_overview(url: &str, bytes: &[u8], model: Model) -> Result<Vec<ListedTile>, OverviewError> {
    let fail = |retry: bool, message: String| OverviewError {
        retry,
        error: FetchError::Http {
            url: url.to_string(),
            message,
        },
    };
    let json: Value = serde_json::from_slice(bytes).map_err(|_| {
        let text = String::from_utf8_lossy(bytes);
        let what = if text.trim().is_empty() {
            "an empty answer".to_string()
        } else {
            format!("not a tile overview: {}", excerpt(&text))
        };
        fail(true, what)
    })?;
    if json["success"] != true {
        return Err(fail(
            false,
            match json["reason"].as_str() {
                Some("tooManyObjects") => format!(
                    "the Downloadportal lists at most {} tiles per request; reduce the map area",
                    json["max"].as_u64().unwrap_or(500)
                ),
                _ => format!("not a tile overview: {}", excerpt(&json.to_string())),
            },
        ));
    }
    let features = json["result"]["features"]
        .as_array()
        .ok_or_else(|| fail(false, "not a tile overview: no features".to_string()))?;
    let matching: Vec<&Value> = features
        .iter()
        .map(|feature| &feature["properties"])
        .filter(|properties| properties["type"].as_str() == Some(model.overview_type()))
        .collect();
    // Features, but none of the requested type: the portal changed, not an empty area.
    if matching.is_empty() && !features.is_empty() {
        return Err(fail(
            false,
            format!(
                "the overview lists {} tiles but none of type `{}`; the Downloadportal has changed",
                features.len(),
                model.overview_type()
            ),
        ));
    }
    let mut tiles = matching
        .into_iter()
        .map(|properties| {
            let key = properties["kachel_nr"].as_str().unwrap_or_default();
            let tile = parse_tile_key(key)
                .ok_or_else(|| fail(false, format!("unknown tile key `{}`", excerpt(key))))?;
            let date = properties["aktualitaet"]
                .as_str()
                .filter(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok())
                .map(str::to_string);
            Ok(ListedTile { tile, date })
        })
        .collect::<Result<Vec<_>, _>>()?;
    tiles.sort_by_key(|t| t.tile);
    tiles.dedup_by_key(|t| t.tile);
    Ok(tiles)
}

/// `33_312_5996` (zone 33, the south-west corner of a 2 km tile in km).
fn parse_tile_key(key: &str) -> Option<TileId> {
    let (east, north) = key.strip_prefix("33_")?.split_once('_')?;
    let digits = |s: &str, n| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(east, 3) || !digits(north, 4) {
        return None;
    }
    let tile = TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    };
    (tile.east_km.is_multiple_of(TILE_KM) && tile.north_km.is_multiple_of(TILE_KM)).then_some(tile)
}

/// The start of an answer as one clean line (see [`one_line`]).
fn excerpt(text: &str) -> String {
    one_line(text, 200)
}

/// The overview of `model` over `bbox`, kept in `dir` for offline runs (and for an online run
/// whose overview stays unreadable, with a warning). Empty and non-JSON answers are asked again
/// up to [`OVERVIEW_ATTEMPTS`] times; a refusal by the portal (`success: false`) is not.
fn tile_list(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    bbox: &BoundingBox,
    model: Model,
    warnings: &mut Vec<String>,
) -> Result<Vec<ListedTile>, FetchError> {
    let url = overview_url(bbox, model);
    let path =
        dir.join("overview")
            .join(format!("{}_{}.json", model.overview_type(), bbox_key(bbox)));
    let cached = || {
        std::fs::read(&path)
            .ok()
            .and_then(|bytes| parse_overview(&url, &bytes, model).ok())
    };
    if mode == FetchMode::Offline {
        return cached().ok_or_else(|| FetchError::Http {
            url: url.clone(),
            message: format!(
                "offline and not cached: the tile list of the {} for this area; run \
                 `freemapper fetch` online",
                model.label()
            ),
        });
    }
    // The outer error is retried, the inner one is final.
    let answer = get_with_attempts(
        fetcher,
        &url,
        OVERVIEW_ATTEMPTS,
        |bytes| match parse_overview(&url, bytes, model) {
            Ok(tiles) => Ok(Ok((tiles, bytes.to_vec()))),
            Err(OverviewError { retry: true, error }) => Err(error),
            Err(OverviewError { error, .. }) => Ok(Err(error)),
        },
    )
    .and_then(|answer| answer);
    match answer {
        Ok((tiles, bytes)) => {
            write_atomically(&path, &bytes)?;
            Ok(tiles)
        }
        Err(e) => {
            let Some(tiles) = cached() else {
                return Err(e);
            };
            warnings.push(format!(
                "the Downloadportal's tile list of the {} could not be read ({e}); used the one \
                 kept from an earlier run",
                model.label()
            ));
            Ok(tiles)
        }
    }
}

/// The 2 km tile holding the point.
fn tile_at(e: f64, n: f64) -> TileId {
    let km = |v: f64| ((v / 1000.0).floor() as u32) / TILE_KM * TILE_KM;
    TileId {
        east_km: km(e),
        north_km: km(n),
    }
}

/// The grid of the model stored at `out` (its world file and TIFF size).
fn stored_grid(out: &Path) -> Option<GridSpec> {
    let world = std::fs::read_to_string(out.with_extension("tfw")).ok()?;
    let file = std::fs::File::open(out).ok()?;
    let (width, height) = tiff::decoder::Decoder::new(std::io::BufReader::new(file))
        .ok()?
        .dimensions()
        .ok()?;
    GridSpec::from_world_file(&world, width, height).ok()
}

/// DGM1 or DOM1 over `bbox` (EPSG:25833) from the WCS, masked to the MV tile set and written as
/// a Float32 GeoTIFF `out` with its world file.
///
/// The tile set and the dates come from `overview.php` (kept per area for offline runs, and as
/// the fallback when the portal does not answer), the heights from one GetCoverage per piece of
/// at most 4000 m. Cells whose 2 km tile is not in the set are NaN; zeros inside it (sea,
/// Bodden) stay. Tiles whose centre lies in Mecklenburg-Vorpommern but that the portal does not
/// list are returned as missing; the date is the centre tile's. No listed tile at all is an
/// error.
///
/// Offline, the model stored at `out` is kept when it covers the area (the caller checks its
/// record too, as for Saxony-Anhalt); the kept tile list gives the missing tiles and the date.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let dir = cache.laiv_mv_dir();
    let mut warnings = Vec::new();
    let grid = GridSpec::covering(bbox, 1.0);
    let area = grid.bounds();
    if mode == FetchMode::Offline {
        let covers = stored_grid(out).is_some_and(|stored| {
            let s = stored.bounds();
            s.min_e <= area.min_e
                && s.min_n <= area.min_n
                && s.max_e >= area.max_e
                && s.max_n >= area.max_n
        });
        if !covers {
            return Err(FetchError::Http {
                url: out.display().to_string(),
                message: format!(
                    "no {} stored for this map area; run `freemapper fetch` online first",
                    model.label()
                ),
            });
        }
        let set = match tile_list(fetcher, &dir, mode, bbox, model, &mut warnings) {
            Ok(listed) => tile_set(listed, &area),
            Err(e) => {
                warnings.push(format!("{e}; the stored model has no date"));
                return Ok(HeightModel {
                    warnings,
                    ..HeightModel::default()
                });
            }
        };
        return Ok(summary(&set, &area, bbox, model, warnings));
    }
    let listed = tile_list(fetcher, &dir, mode, bbox, model, &mut warnings)?;
    let set = tile_set(listed, &area);
    // The WCS is asked for the listed tiles' part of the area only.
    let Some(within) = set
        .keys()
        .map(|tile| tile.bounds_km(TILE_KM))
        .reduce(|a, b| BoundingBox {
            min_e: a.min_e.min(b.min_e),
            min_n: a.min_n.min(b.min_n),
            max_e: a.max_e.max(b.max_e),
            max_n: a.max_n.max(b.max_n),
        })
    else {
        return Err(FetchError::Http {
            url: overview_url(bbox, model),
            message: format!(
                "no tile of the {} covers the map area; the Downloadportal may have changed",
                model.label()
            ),
        });
    };
    let mut raster = Raster::filled(grid, f32::NAN);
    model.wcs().fill(fetcher, &mut raster, &within)?;
    let column_tiles: Vec<u32> = (0..grid.width)
        .map(|x| tile_at(grid.west + f64::from(x) + 0.5, area.min_n).east_km)
        .collect();
    let mut empty = 0usize;
    for y in 0..grid.height {
        let north_km = tile_at(area.min_e, grid.north - f64::from(y) - 0.5).north_km;
        let row = &mut raster.data[y as usize * grid.width as usize..][..grid.width as usize];
        for (z, &east_km) in row.iter_mut().zip(&column_tiles) {
            if !set.contains_key(&TileId { east_km, north_km }) {
                *z = f32::NAN;
            } else if z.is_nan() {
                empty += 1;
            }
        }
    }
    if empty > 0 {
        warnings.push(format!(
            "{}: {empty} cells inside the Mecklenburg-Vorpommern tiles have no height",
            model.label()
        ));
    }
    write_float_raster(&raster, out, model.wcs().url)?;
    Ok(summary(&set, &area, bbox, model, warnings))
}

/// The listed tiles touching `area`, with their dates.
fn tile_set(listed: Vec<ListedTile>, area: &BoundingBox) -> HashMap<TileId, Option<String>> {
    listed
        .into_iter()
        .filter(|t| t.tile.bounds_km(TILE_KM).intersection(area).is_some())
        .map(|t| (t.tile, t.date))
        .collect()
}

/// The missing tiles and the centre tile's date of a fetch with the tile `set`.
fn summary(
    set: &HashMap<TileId, Option<String>>,
    area: &BoundingBox,
    bbox: &BoundingBox,
    model: Model,
    mut warnings: Vec<String>,
) -> HeightModel {
    let missing = tiles_covering_grid(area, TILE_KM)
        .into_iter()
        .filter(|tile| !set.contains_key(tile))
        .filter(|tile| {
            let b = tile.bounds_km(TILE_KM);
            // The exact boundary: a tile centred just beyond it need not exist.
            let (e, n) = ((b.min_e + b.max_e) / 2.0, (b.min_n + b.max_n) / 2.0);
            contains_point(Region::MecklenburgVorpommern, e, n)
        })
        .collect();
    let centre = tile_at(
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    // Listed without a date, or not listed (a border map centred beyond the MV tiles).
    let date = set.get(&centre).cloned().flatten();
    if date.is_none() {
        warnings.push(format!(
            "{}: no date for the centre tile 33_{:03}_{:04}",
            model.label(),
            centre.east_km,
            centre.north_km
        ));
    }
    HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub const WCS_DGM1: &[u8] =
        include_bytes!("../../../../tests/fixtures/mecklenburg-vorpommern-wcs-dgm1.tif");
    pub const WCS_DOM1: &[u8] =
        include_bytes!("../../../../tests/fixtures/mecklenburg-vorpommern-wcs-dom1.tif");
    pub const OVERVIEW_DGM1: &str =
        include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-overview-dgm1.json");
    pub const OVERVIEW_DOM: &str =
        include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-overview-dom.json");

    /// An `overview.php` answer listing `tiles` of `model`, each dated `date`.
    pub fn overview_answer(model: Model, tiles: &[TileId], date: &str) -> Vec<u8> {
        let features: Vec<Value> = tiles
            .iter()
            .enumerate()
            .map(|(i, tile)| {
                let b = tile.bounds_km(TILE_KM);
                serde_json::json!({
                    "type": "Feature",
                    "geometry": {"type": "Polygon", "coordinates": [[
                        [b.min_e, b.min_n], [b.min_e, b.max_n], [b.max_e, b.max_n],
                        [b.max_e, b.min_n], [b.min_e, b.min_n]
                    ]]},
                    "properties": {
                        "gid": (1000 + i).to_string(),
                        "aktualitaet": date,
                        "kachel_nr": format!("33_{:03}_{:04}", tile.east_km, tile.north_km),
                        "type": model.overview_type(),
                    }
                })
            })
            .collect();
        serde_json::json!({
            "success": true,
            "result": {"type": "FeatureCollection", "features": features}
        })
        .to_string()
        .into_bytes()
    }

    /// A plain WCS answer (an uncompressed Float32 GeoTIFF) on `grid`.
    pub fn plain_answer(grid: &GridSpec, values: &[f32]) -> Vec<u8> {
        crate::fetch::test_support::geotiff(grid, values)
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::float_tiff::decode_float_tiff;

    /// The school tile and its eastern neighbour.
    const SCHOOL_TILE: TileId = TileId {
        east_km: 312,
        north_km: 5996,
    };
    const EAST_TILE: TileId = TileId {
        east_km: 314,
        north_km: 5996,
    };

    /// 40 x 20 m straddling E 314 000, the edge between [`SCHOOL_TILE`] and [`EAST_TILE`].
    const ACROSS: BoundingBox = BoundingBox {
        min_e: 313_980.0,
        min_n: 5_997_000.0,
        max_e: 314_020.0,
        max_n: 5_997_020.0,
    };

    fn cache() -> (tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        (dir, cache)
    }

    /// Serves `model` over `bbox`: the overview lists `tiles` (dated 2024-03-01), and the WCS
    /// answers `height(e, n)` at each cell centre.
    fn serve(
        fetcher: &MapFetcher,
        model: Model,
        bbox: &BoundingBox,
        tiles: &[TileId],
        height: impl Fn(f64, f64) -> f32,
    ) {
        fetcher.insert_prefix(
            &overview_prefix(model),
            overview_answer(model, tiles, "2024-03-01"),
        );
        let grid = GridSpec::covering(bbox, 1.0);
        let values: Vec<f32> = (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
            .map(|(x, y)| {
                height(
                    grid.west + f64::from(x) + 0.5,
                    grid.north - f64::from(y) - 0.5,
                )
            })
            .collect();
        fetcher.insert_prefix(model.wcs().url, plain_answer(&grid, &values));
    }

    #[test]
    fn mv_wcs_answer_decodes() {
        use tiff::tags::Tag;
        let expected = GridSpec {
            west: 312_100.0,
            north: 5_996_350.0,
            pixel_size: 1.0,
            width: 100,
            height: 100,
        };
        let mut rasters = Vec::new();
        for (model, bytes) in [(Model::Dgm1, WCS_DGM1), (Model::Dom1, WCS_DOM1)] {
            // The answer is LZW-compressed, as asked for.
            let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
            assert_eq!(
                decoder.find_tag_unsigned::<u16>(Tag::Compression).unwrap(),
                Some(5)
            );
            let tiff = model.wcs().tiff(bytes).unwrap();
            let raster = decode_float_tiff(tiff).unwrap();
            assert_eq!(raster.grid, expected, "{model:?}");
            assert!(raster.data.iter().all(|z| z.is_finite()), "{model:?}");
            rasters.push(raster);
        }
        let (dgm, dom) = (&rasters[0], &rasters[1]);
        // GetFeatureInfo on dgm_wms gives 12.542 m at E 312157.5 / N 5996316.5 (reference §5.3).
        assert_eq!(dgm.get(57, 33), 12.542);
        // The surface lies on or above the terrain, with buildings and trees on the school site.
        let above: Vec<f32> = dom.data.iter().zip(&dgm.data).map(|(s, t)| s - t).collect();
        assert!(above.iter().all(|d| *d > -1.0), "surface below terrain");
        assert!(above.iter().any(|d| *d > 5.0), "no object above 5 m");
    }

    #[test]
    fn cells_outside_the_tile_set_are_nan() {
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        // The portal lists only the school tile; the WCS fills the eastern one with 0.0.
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |e, _| {
            if e < 314_000.0 { 7.5 } else { 0.0 }
        });
        let out = dir.path().join("dgm1.tif");
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (40, 20));
        for y in 0..20 {
            for x in 0..40 {
                let z = raster.get(x, y);
                if x < 20 {
                    assert_eq!(z, 7.5, "({x}, {y})");
                } else {
                    assert!(z.is_nan(), "({x}, {y}): {z}");
                }
            }
        }
        // Rostock lies east of E 314 000 too, so the portal should have the tile.
        assert_eq!(found.missing, vec![EAST_TILE]);
        // The centre (E 314 000) lies in the unlisted tile: no date.
        assert_eq!(found.date, None);
        // The request asks only for the listed tile's part.
        let wcs: Vec<String> = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.starts_with(Model::Dgm1.wcs().url))
            .cloned()
            .collect();
        assert_eq!(
            wcs,
            [format!(
                "{}?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=mv_dgm\
                 &FORMAT=image/tiff&GEOTIFF:COMPRESSION=LZW\
                 &SUBSET=x(313980,314000)&SUBSET=y(5997000,5997020)",
                Model::Dgm1.wcs().url
            )]
        );
    }

    #[test]
    fn sea_inside_stays_zero() {
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        // Both tiles listed; the Warnow at 0.0 m west of E 314 000.
        serve(
            &fetcher,
            Model::Dom1,
            &ACROSS,
            &[SCHOOL_TILE, EAST_TILE],
            |e, _| if e < 314_000.0 { 0.0 } else { 3.25 },
        );
        let out = dir.path().join("dom1.tif");
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dom1,
            &ACROSS,
            &out,
        )
        .unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert!(raster.data[..20].iter().all(|z| *z == 0.0), "sea kept");
        assert!(raster.data[20..40].iter().all(|z| *z == 3.25));
        assert!(raster.data.iter().all(|z| z.is_finite()));
        assert!(found.missing.is_empty());
        assert!(found.warnings.is_empty(), "{:?}", found.warnings);
        assert_eq!(found.date.as_deref(), Some("2024-03-01"));
        // The DOM1 is listed as `dom`.
        assert!(
            fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.starts_with(&format!("{OVERVIEW}?type[]=dom&bbox=")))
        );
    }

    /// A MapServer WCS 2.0 exception report.
    const EXTENT_ERROR: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/2.0" version="2.0.1" xml:lang="en-US">
  <ows:Exception exceptionCode="ExtentError" locator="mapserv">
    <ows:ExceptionText>msWCSGetCoverage20(): WCS server error. Image extent does not intersect with desired region.</ows:ExceptionText>
  </ows:Exception>
</ows:ExceptionReport>"#;

    #[test]
    fn wcs_exception_is_an_error() {
        let wcs = Model::Dgm1.wcs();
        let calls = |fetcher: &MapFetcher| {
            fetcher
                .log
                .borrow()
                .iter()
                .filter(|u| u.starts_with(wcs.url))
                .count()
        };
        // With status 200: an error with the exception's text (retried like any bad answer).
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |_, _| 1.0);
        fetcher.insert_prefix(wcs.url, EXTENT_ERROR.to_vec());
        let out = dir.path().join("dgm1.tif");
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("no TIFF in the WCS answer"), "{err}");
        assert!(err.contains("ExtentError: msWCSGetCoverage20()"), "{err}");
        assert_eq!(calls(&fetcher), 1, "a report is final");
        assert!(!out.exists());

        // With status 400: refused at once, with the exception's text.
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |_, _| 1.0);
        fetcher.refusing.borrow_mut().push(wcs.url.to_string());
        fetcher
            .refusal_bodies
            .borrow_mut()
            .insert(wcs.url.to_string(), EXTENT_ERROR.to_vec());
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("refused the request (HTTP 400)"), "{err}");
        assert!(
            err.contains("ExtentError: msWCSGetCoverage20(): WCS server error. Image extent"),
            "{err}"
        );
        assert_eq!(calls(&fetcher), 1, "a refusal is not retried");

        // A dropped connection is retried.
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |_, _| 1.0);
        fetcher.flaky.borrow_mut().insert(wcs.url.to_string(), 2);
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        assert_eq!(calls(&fetcher), 3);
    }

    #[test]
    fn overview_answers_parse() {
        let url = overview_url(&ACROSS, Model::Dgm1);
        let school = |date: &str| {
            vec![ListedTile {
                tile: SCHOOL_TILE,
                date: Some(date.to_string()),
            }]
        };
        let parse = |bytes: &[u8], model| parse_overview(&url, bytes, model);
        assert_eq!(
            parse(OVERVIEW_DGM1.as_bytes(), Model::Dgm1).ok(),
            Some(school("2024-03-01"))
        );
        assert_eq!(
            parse(OVERVIEW_DOM.as_bytes(), Model::Dom1).ok(),
            Some(school("2024-08-01"))
        );
        // Another product's answer is not taken for this one.
        assert!(parse(OVERVIEW_DOM.as_bytes(), Model::Dgm1).is_err());
        // Refusals are final, empty and HTML answers are asked again.
        for (body, retry, text) in [
            (
                &br#"{"success":false,"message":"Too many objects returned - max 500","reason":"tooManyObjects","max":500}"#[..],
                false,
                "at most 500 tiles",
            ),
            (
                br#"{"success":false,"message":"Interner Fehler"}"#,
                false,
                "Interner Fehler",
            ),
            (b"", true, "an empty answer"),
            (b"<html>\x1b[31mBad\xe2\x80\xae Gateway</html>", true, "<html> [31mBad Gateway"),
        ] {
            let Err(e) = parse(body, Model::Dgm1) else {
                panic!("{body:?} parsed")
            };
            assert_eq!(e.retry, retry, "{body:?}");
            assert!(e.error.to_string().contains(text), "{}", e.error);
        }
        assert_eq!(parse_tile_key("33_313_5996"), None, "odd corner");
        assert_eq!(parse_tile_key("32_312_5996"), None);
    }

    /// Answers the first `empty` overview requests with an empty body, like the portal now and
    /// then.
    struct EmptyFirst {
        inner: MapFetcher,
        empty: std::cell::Cell<u32>,
    }

    impl Fetcher for EmptyFirst {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            if url.starts_with(OVERVIEW) && self.empty.get() > 0 {
                self.empty.set(self.empty.get() - 1);
                return Ok(Vec::new());
            }
            self.inner.get(url)
        }
    }

    #[test]
    fn empty_overview_answers_are_retried() {
        let (dir, cache) = cache();
        let fetcher = EmptyFirst {
            inner: MapFetcher::default(),
            empty: std::cell::Cell::new(OVERVIEW_ATTEMPTS - 1),
        };
        serve(
            &fetcher.inner,
            Model::Dgm1,
            &ACROSS,
            &[SCHOOL_TILE],
            |_, _| 4.0,
        );
        let out = dir.path().join("dgm1.tif");
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        assert_eq!(fetcher.empty.get(), 0);

        // Empty every time: the list kept from the run above, with a warning.
        fetcher.empty.set(u32::MAX);
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        // The second warning: the centre (E 314 000) lies in the unlisted tile.
        assert_eq!(found.warnings.len(), 2, "{:?}", found.warnings);
        assert!(
            found.warnings[0].contains("an empty answer"),
            "{:?}",
            found.warnings
        );
        assert_eq!(fetcher.empty.get(), u32::MAX - OVERVIEW_ATTEMPTS);

        // And without a kept list, an error.
        let (_other, empty_cache) = self::cache();
        let err = fetch_model(
            &fetcher,
            &empty_cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("an empty answer"), "{err}");
    }

    #[test]
    fn offline_reuses_the_stored_model() {
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |e, _| {
            (e - 313_000.0) as f32
        });
        let out = dir.path().join("dgm1.tif");
        let online = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        let first = std::fs::read(&out).unwrap();
        // No mosaic is kept in the shared cache.
        assert!(!cache.laiv_mv_dir().join("wcs").exists());
        // Offline: the stored model stays, the kept tile list gives the same result.
        let none = MapFetcher::default();
        let offline = fetch_model(
            &none,
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        assert_eq!(none.calls.get(), 0);
        assert_eq!(offline, online);
        assert_eq!(std::fs::read(&out).unwrap(), first);
        // A smaller area inside it is covered too.
        let inner = BoundingBox {
            min_e: 313_990.0,
            max_e: 314_010.0,
            ..ACROSS
        };
        let found =
            fetch_model(&none, &cache, FetchMode::Offline, Model::Dgm1, &inner, &out).unwrap();
        assert_eq!(found.date, None);
        assert_eq!(std::fs::read(&out).unwrap(), first);
        // A larger area is not, and nothing stored is the usual error.
        let wider = BoundingBox {
            max_e: 314_100.0,
            ..ACROSS
        };
        for (bbox, path) in [(wider, out.clone()), (ACROSS, dir.path().join("dom1.tif"))] {
            let err = fetch_model(&none, &cache, FetchMode::Offline, Model::Dgm1, &bbox, &path)
                .unwrap_err();
            assert!(
                err.to_string().contains(
                    "no terrain model (LAiV M-V DGM1) stored for this map area; run \
                     `freemapper fetch` online first"
                ),
                "{err}"
            );
        }
    }

    #[test]
    fn brandenburg_beyond_the_mv_tiles_is_nan() {
        // Brüssow: the MV tile row N 5918 exists, the row N 5916 (all Brandenburg) does not.
        let bbox = BoundingBox {
            min_e: 436_800.0,
            min_n: 5_917_800.0,
            max_e: 437_200.0,
            max_n: 5_918_200.0,
        };
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_mv(&fetcher, &bbox);
        for (model, date, lift) in [
            (Model::Dgm1, "2024-03-01", 5.0),
            (Model::Dom1, "2024-08-01", 17.0),
        ] {
            let out = dir.path().join("model.tif");
            let found =
                fetch_model(&fetcher, &cache, FetchMode::Online, model, &bbox, &out).unwrap();
            let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
            assert_eq!((raster.grid.width, raster.grid.height), (400, 400));
            for y in [0, 199, 200, 399] {
                let z = raster.get(10, y);
                if y < 200 {
                    assert_eq!(z, lift + 0.1, "{model:?} ({y})");
                } else {
                    assert!(z.is_nan(), "{model:?} ({y}): {z}");
                }
            }
            assert!(found.missing.is_empty(), "{:?}", found.missing);
            assert_eq!(found.date.as_deref(), Some(date));
        }
    }

    #[test]
    #[ignore = "live: asks the LAiV M-V portal and WCS"]
    fn live_rostock_school_models() {
        // 200 m around the Innerstädtisches Gymnasium Rostock.
        let bbox = BoundingBox {
            min_e: 312_050.0,
            min_n: 5_996_200.0,
            max_e: 312_250.0,
            max_n: 5_996_400.0,
        };
        let (dir, cache) = cache();
        let fetcher = crate::fetch::HttpFetcher::new();
        let mut rasters = Vec::new();
        for model in [Model::Dgm1, Model::Dom1] {
            let out = dir.path().join(format!("{}.tif", model.wcs().coverage));
            let found =
                fetch_model(&fetcher, &cache, FetchMode::Online, model, &bbox, &out).unwrap();
            let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
            let (min, max) = raster
                .data
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), &z| (lo.min(z), hi.max(z)));
            eprintln!(
                "{}: {:?}, {min} .. {max} m, {} NaN, {found:?}",
                model.label(),
                raster.grid,
                raster.data.iter().filter(|z| z.is_nan()).count()
            );
            assert_eq!((raster.grid.width, raster.grid.height), (200, 200));
            assert!(raster.data.iter().all(|z| z.is_finite()));
            assert!(
                found.missing.is_empty() && found.warnings.is_empty(),
                "{found:?}"
            );
            assert!(found.date.is_some());
            rasters.push(raster);
        }
        // E 312157.5 / N 5996316.5 (reference §5.3).
        assert_eq!(rasters[0].get(107, 83), 12.542);
    }

    #[test]
    fn missing_needs_the_tile_centre_inside_mv() {
        // The centre of 33_256_5890 lies in Brandenburg, less than 100 m from Mecklenburg-
        // Vorpommern: an unlisted tile there is not missing.
        let south = TileId {
            east_km: 256,
            north_km: 5890,
        };
        let north = TileId {
            east_km: 256,
            north_km: 5892,
        };
        let bbox = BoundingBox {
            min_e: 256_980.0,
            min_n: 5_891_980.0,
            max_e: 257_020.0,
            max_n: 5_892_020.0,
        };
        let (dir, cache) = cache();
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &bbox, &[north], |_, _| 30.0);
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            &dir.path().join("dgm1.tif"),
        )
        .unwrap();
        assert!(!found.missing.contains(&south), "{:?}", found.missing);
        assert!(found.missing.is_empty(), "{:?}", found.missing);
    }

    #[test]
    fn centre_tile_without_date_warns() {
        let bbox = BoundingBox {
            min_e: 312_100.0,
            min_n: 5_996_250.0,
            max_e: 312_200.0,
            max_n: 5_996_350.0,
        };
        let (dir, cache) = cache();
        let out = dir.path().join("dgm1.tif");
        // Listed, but `aktualitaet` is not a date.
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &bbox, &[SCHOOL_TILE], |_, _| 9.0);
        fetcher.insert_prefix(
            &overview_prefix(Model::Dgm1),
            OVERVIEW_DGM1.replace("2024-03-01", "03.2024").into_bytes(),
        );
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            &out,
        )
        .unwrap();
        assert_eq!(found.date, None);
        assert_eq!(
            found.warnings,
            ["terrain model (LAiV M-V DGM1): no date for the centre tile 33_312_5996"]
        );
        // Not listed at all (a border map centred beyond the MV tiles).
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, &ACROSS, &[SCHOOL_TILE], |_, _| 9.0);
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &ACROSS,
            &out,
        )
        .unwrap();
        assert_eq!(found.date, None);
        assert_eq!(
            found.warnings,
            ["terrain model (LAiV M-V DGM1): no date for the centre tile 33_314_5996"]
        );
    }
}
