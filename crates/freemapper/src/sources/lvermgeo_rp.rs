//! LVermGeo Rheinland-Pfalz (`geobasis-rlp.de`, dl-de/by-2-0, "©GeoBasis-DE / LVermGeoRP<year>"):
//! the DGM1 and DOM1 1 km GeoTIFF tiles (Float32, LZW, EPSG:25832).
//!
//! A tile's file name carries the year of its laser scan, which differs per tile. The tile list
//! is the `Laserbefliegung` layer of the Blattschnitt WFS: it gives the flight days per tile, and
//! the year in the name follows from them. Names are never guessed by trying years. Everything
//! comes from the origin hosts; the GeoPortal.rlp registry proxy answers errors with HTML and
//! status 200.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{Datelike, NaiveDate};

use crate::crs::BoundingBox;
use crate::fetch::{
    Cache, FetchError, FetchMode, Fetcher, get_with_retries, one_line, write_atomically,
};
use crate::raster::{GridSpec, Raster};
use crate::sources::alkis_wfs::{number_matched, parse_gml};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::height_model::HeightModel;
use crate::sources::lvermgeo_sh::{paste_tile, read_cached, whole_tile};
use crate::sources::region::{Region, tile_centre_in};
use crate::tiles::{TileId, tiles_covering};

/// The Blattschnitt WFS (MapServer) with the tile list.
pub const TILE_LIST_URL: &str = "https://geo5.service24.rlp.de/wfs/blattschnitt_rp.fcgi";
/// The layer with one feature per 1 km tile of the laser scan (DGM1, DOM1, point clouds).
const TILE_LIST_LAYER: &str = "Laserbefliegung";
/// The file server of the tiles.
const DATA: &str = "https://geobasis-rlp.de/data";

/// The height models of LVermGeo RP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Dgm1,
    Dom1,
}

impl Model {
    /// `dgm1`: the product folder, the cache folder and the start of every tile name.
    fn slug(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom1",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LVermGeo RP DGM1)",
            Self::Dom1 => "surface model (LVermGeo RP DOM1)",
        }
    }
}

/// The start of every [`tile_list_url`], up to the bbox.
pub(crate) fn tile_list_prefix() -> String {
    format!(
        "{TILE_LIST_URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={TILE_LIST_LAYER}"
    )
}

/// The tile list over `bbox` (EPSG:25832), rounded outward to whole metres.
pub fn tile_list_url(bbox: &BoundingBox) -> String {
    format!(
        "{}&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::25832",
        tile_list_prefix(),
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// The GeoTIFF `name` (without extension) of `model`.
pub fn tile_url(model: Model, name: &str) -> String {
    format!("{DATA}/{}/current/tif/{name}.tif", model.slug())
}

/// One tile of the tile list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ListedTile {
    tile: TileId,
    /// The year in the file name; `None` when the list gives none or contradicts itself.
    year: Option<i32>,
    /// The last flight day, `YYYY-MM-DD`.
    date: Option<String>,
    /// Why the year or the date is missing.
    note: Option<String>,
}

impl ListedTile {
    /// `dgm1_32_400_5579_1_rp_2024`, checked against the strict pattern; `None` without a year.
    fn name(&self, model: Model) -> Option<String> {
        let name = format!(
            "{}_32_{:03}_{:04}_1_rp_{}",
            model.slug(),
            self.tile.east_km,
            self.tile.north_km,
            self.year?
        );
        valid_name(model, &name).then_some(name)
    }
}

/// What the tile list says.
#[derive(Debug, Default)]
struct TileList {
    tiles: HashMap<TileId, ListedTile>,
    warnings: Vec<String>,
}

fn digits(value: &str, n: usize) -> bool {
    value.len() == n && value.bytes().all(|b| b.is_ascii_digit())
}

/// Whether `name` is a tile name of `model`: `dgm1_32_EEE_NNNN_1_rp_YYYY`, digits only.
fn valid_name(model: Model, name: &str) -> bool {
    let Some(rest) = name.strip_prefix(model.slug()) else {
        return false;
    };
    let parts: Vec<&str> = rest.split('_').collect();
    matches!(
        parts[..],
        ["", "32", east, north, "1", "rp", year]
            if digits(east, 3) && digits(north, 4) && digits(year, 4)
    )
}

/// `4005579` (E 400 km, N 5579 km).
fn parse_tile_key(key: &str) -> Option<TileId> {
    if !digits(key, 7) {
        return None;
    }
    Some(TileId {
        east_km: key[..3].parse().ok()?,
        north_km: key[3..].parse().ok()?,
    })
}

/// `flugjahr`: four digits of this century.
fn parse_year(value: &str) -> Option<i32> {
    digits(value, 4)
        .then(|| value.parse().ok())
        .flatten()
        .filter(|y| (2000..2100).contains(y))
}

/// The last day of each item of `flugtage`: `30.03.24`, `07.-09.02.23` or `27.02.-01.03.23`,
/// separated by commas; `None` when an item has another form.
fn flight_days(text: &str) -> Option<Vec<NaiveDate>> {
    let two = |s: &str| digits(s, 2).then(|| s.parse::<u32>().ok()).flatten();
    text.split(',')
        .map(|item| {
            let item = item.trim();
            let (start, end) = match item.split_once('-') {
                Some((start, end)) => (Some(start), end),
                None => (None, item),
            };
            // The first day of a range: `07.` or `27.02.`.
            if let Some(start) = start {
                let parts: Vec<&str> = start.split('.').collect();
                let readable = match parts[..] {
                    [d, ""] => two(d).is_some(),
                    [d, m, ""] => two(d).is_some() && two(m).is_some(),
                    _ => false,
                };
                if !readable {
                    return None;
                }
            }
            let parts: Vec<&str> = end.split('.').collect();
            let [d, m, y] = parts[..] else { return None };
            NaiveDate::from_ymd_opt(2000 + two(y)? as i32, two(m)?, two(d)?)
        })
        .collect()
}

/// The tile `tile` with its `flugjahr` and `flugtage` (both may be empty). The name year is the
/// year of the flight days; `flugjahr` stands in when there are none. When the two disagree, or
/// the days span years, the tile has no name.
fn listed_tile(tile: TileId, flight_year: &str, days: &str) -> ListedTile {
    let year = parse_year(flight_year);
    let flown = (!days.is_empty()).then(|| flight_days(days));
    let quoted = |s: &str| format!("`{}`", one_line(s, 80));
    let (year, date, note) = match (year, flown) {
        (None, None) => (
            None,
            None,
            Some("the tile list gives neither a flight year nor flight days".to_string()),
        ),
        (Some(year), None) => (
            Some(year),
            None,
            Some("the tile list gives no flight days".to_string()),
        ),
        (year, Some(None)) => (
            year,
            None,
            Some(format!("the flight days {} cannot be read", quoted(days))),
        ),
        (year, Some(Some(dates))) => {
            let years: HashSet<i32> = dates.iter().map(|d| d.year()).collect();
            let last = dates.iter().max().map(|d| d.to_string());
            match (years.iter().next(), years.len(), year) {
                (Some(&flown), 1, None) => (Some(flown), last, None),
                (Some(&flown), 1, Some(year)) if flown == year => (Some(flown), last, None),
                _ => (
                    None,
                    last,
                    Some(format!(
                        "the flight days {} and the flight year {} do not give one year",
                        quoted(days),
                        quoted(flight_year)
                    )),
                ),
            }
        }
    };
    ListedTile {
        tile,
        year,
        date,
        note,
    }
}

/// The tiles of a `Laserbefliegung` answer (GML 3.2); anything else (an HTML page, an exception
/// report, a cut answer) is an error.
fn parse_tile_list(text: &str) -> Result<TileList, String> {
    let features = parse_gml(text).map_err(|_| {
        format!(
            "not a tile list ({TILE_LIST_LAYER}): {}",
            one_line(text, 200)
        )
    })?;
    if let Some(matched) = number_matched(text)
        && matched > features.len()
    {
        return Err(format!(
            "the tile list holds {} of the {matched} tiles it matched",
            features.len()
        ));
    }
    let mut tiles: HashMap<TileId, ListedTile> = HashMap::new();
    let mut unreadable = 0;
    for feature in &features {
        let properties = &feature["properties"];
        let field = |name: &str| properties[name].as_str().map(str::trim).unwrap_or_default();
        let Some(tile) = parse_tile_key(field("kachel")) else {
            unreadable += 1;
            continue;
        };
        let entry = listed_tile(tile, field("flugjahr"), field("flugtage"));
        // A tile twice: the later flight wins.
        match tiles.get(&tile) {
            Some(old) if (&old.date, old.year) >= (&entry.date, entry.year) => {}
            _ => {
                tiles.insert(tile, entry);
            }
        }
    }
    let mut warnings = Vec::new();
    if unreadable > 0 {
        warnings.push(format!(
            "{unreadable} features of the tile list have no usable tile key; skipped"
        ));
    }
    Ok(TileList { tiles, warnings })
}

/// The largest tile list answer accepted (1.3 KB per tile; a 5 x 5 km area is 50 KB).
const MAX_LIST_BYTES: usize = 32 * 1024 * 1024;

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

/// The tile list over `bbox`, kept in `dir` for offline runs (and for an online run whose list
/// stays unreadable, with a warning). DGM1 and DOM1 share it: one flight gives both.
fn tile_list(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    bbox: &BoundingBox,
    warnings: &mut Vec<String>,
) -> Result<TileList, FetchError> {
    let url = tile_list_url(bbox);
    let path = dir
        .join("tile_list")
        .join(format!("{}.gml", bbox_key(bbox)));
    let kept = || {
        let bytes = std::fs::read(&path).ok()?;
        parse_tile_list(&String::from_utf8_lossy(&bytes)).ok()
    };
    if mode == FetchMode::Offline {
        return kept().ok_or_else(|| FetchError::Http {
            url: url.clone(),
            message: "offline and not cached: the LVermGeo RP tile list for this area; run \
                      `freemapper fetch` online"
                .to_string(),
        });
    }
    let answer = get_with_retries(fetcher, &url, |bytes| {
        if bytes.len() > MAX_LIST_BYTES {
            return Err(FetchError::TooLarge {
                url: url.clone(),
                limit_mb: (MAX_LIST_BYTES / (1024 * 1024)) as u64,
            });
        }
        let list = parse_tile_list(&String::from_utf8_lossy(bytes)).map_err(|message| {
            FetchError::Http {
                url: url.clone(),
                message,
            }
        })?;
        Ok((list, bytes.to_vec()))
    });
    match answer {
        Ok((list, bytes)) => {
            write_atomically(&path, &bytes)?;
            Ok(list)
        }
        Err(e) => {
            let Some(list) = kept() else {
                return Err(e);
            };
            warnings.push(format!(
                "the LVermGeo RP tile list could not be read ({e}); used the one kept from an \
                 earlier run"
            ));
            Ok(list)
        }
    }
}

/// Where the tile `name` is cached; the name carries the year, so a new scan is a new file.
fn cache_path(dir: &Path, model: Model, name: &str) -> PathBuf {
    dir.join(model.slug()).join(format!("{name}.tif"))
}

/// Where the note is kept that the server answered 404 for the listed tile `name`.
fn marker_path(dir: &Path, model: Model, name: &str) -> PathBuf {
    dir.join(model.slug()).join(format!("{name}.missing"))
}

/// How long a 404 is believed online. The tile list has no release date that would tell when
/// the server got new files, so a missing tile is asked for again after this.
const RECHECK_AFTER: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Whether the 404 note at `path` holds: offline always, online while it is recent.
fn known_missing(path: &Path, mode: FetchMode) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    mode == FetchMode::Offline
        || meta
            .modified()
            .ok()
            .and_then(|written| written.elapsed().ok())
            .is_some_and(|age| age < RECHECK_AFTER)
}

/// The tiles among `tiles` that need no download: a file of some year is cached, or the server
/// answered 404 (an estimate; the tile list decides which year is current).
pub fn cached_tiles(cache: &Cache, model: Model, tiles: &[TileId]) -> HashSet<TileId> {
    let names: Vec<String> = std::fs::read_dir(cache.lvermgeo_rp_dir().join(model.slug()))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tif") || name.ends_with(".missing"))
        .collect();
    tiles
        .iter()
        .filter(|tile| {
            let prefix = format!(
                "{}_32_{:03}_{:04}_1_rp_",
                model.slug(),
                tile.east_km,
                tile.north_km
            );
            names.iter().any(|name| name.starts_with(&prefix))
        })
        .copied()
        .collect()
}

/// `32_400_5579`.
fn short_name(tile: TileId) -> String {
    format!("32_{:03}_{:04}", tile.east_km, tile.north_km)
}

/// Whether `bytes` start like a TIFF (either byte order).
fn is_tiff(bytes: &[u8]) -> bool {
    bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*")
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file.
///
/// The tile names and dates come from the tile list over the area (kept for offline runs);
/// the list also names the neighbours that touch the area, which are not downloaded. Tiles are
/// cached under their name. A tile the list names but the server lacks (404, remembered for a
/// month), a listed tile the list gives no year for, and a tile inside Rhineland-Palatinate the
/// list does not name are returned as missing; the date is the centre tile's last flight day.
/// No tile at all is an error.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let dir = cache.lvermgeo_rp_dir();
    let mut warnings = Vec::new();
    let list = tile_list(fetcher, &dir, mode, bbox, &mut warnings)?;
    warnings.extend(list.warnings.iter().cloned());
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let (mut found, mut missing, mut pending) = (0, Vec::new(), Vec::new());
    let mut paste = |raster: &mut Raster<f32>, name: &str, part: &Raster<f32>| {
        found += 1;
        let empty = paste_tile(raster, part).map_err(|message| FetchError::Http {
            url: tile_url(model, name),
            message: format!("{} tile {name}: {message}", model.label()),
        })?;
        if empty > 0 {
            warnings.push(format!(
                "{} tile {name}: {empty} cells of the map area have no height",
                model.label()
            ));
        }
        Ok::<(), FetchError>(())
    };
    let mut unnamed = Vec::new();
    for tile in tiles_covering(bbox) {
        let Some(entry) = list.tiles.get(&tile) else {
            if tile_centre_in(tile, Region::RheinlandPfalz) {
                missing.push(tile);
            }
            continue;
        };
        let Some(name) = entry.name(model) else {
            unnamed.push(entry);
            missing.push(tile);
            continue;
        };
        match read_cached(&cache_path(&dir, model, &name), tile) {
            Some(part) => paste(&mut raster, &name, &part)?,
            None if known_missing(&marker_path(&dir, model, &name), mode) => missing.push(tile),
            None => pending.push((tile, name)),
        }
    }
    if let (FetchMode::Offline, Some((_, name))) = (mode, pending.first()) {
        return Err(FetchError::Http {
            url: tile_url(model, name),
            message: format!(
                "offline and not cached: {} tile {name}; run `freemapper fetch` online",
                model.label()
            ),
        });
    }
    for (tile, name) in pending {
        let url = tile_url(model, &name);
        let downloaded = get_with_retries(fetcher, &url, |bytes| {
            let refused = |message: String| FetchError::Http {
                url: url.clone(),
                message: format!("{} tile {name}: {message}", model.label()),
            };
            // The file server answers some errors with an HTML page.
            if !is_tiff(bytes) {
                let start = String::from_utf8_lossy(&bytes[..bytes.len().min(1000)]);
                return Err(refused(format!("not a GeoTIFF: {}", one_line(&start, 200))));
            }
            let part = decode_float_tiff(bytes)
                .and_then(|r| whole_tile(r, tile, &[1.0]))
                .map_err(refused)?;
            Ok((part, bytes.to_vec()))
        });
        let marker = marker_path(&dir, model, &name);
        match downloaded {
            Ok((part, bytes)) => {
                write_atomically(&cache_path(&dir, model, &name), &bytes)?;
                let _ = std::fs::remove_file(&marker);
                paste(&mut raster, &name, &part)?;
            }
            // Named by the list, not on the server. The note keeps later runs from asking
            // again for a month, and offline runs from failing.
            Err(FetchError::NotFound(_)) => {
                write_atomically(&marker, b"")?;
                missing.push(tile);
            }
            Err(e) => return Err(e),
        }
    }
    for entry in unnamed {
        warnings.push(format!(
            "{} tile {}: {}, so it has no file name",
            model.label(),
            short_name(entry.tile),
            entry.note.as_deref().unwrap_or("no year")
        ));
    }
    if found == 0 {
        return Err(FetchError::Http {
            url: tile_list_url(bbox),
            message: format!("no tile of the {} covers the map area", model.label()),
        });
    }
    write_float_raster(&raster, out, DATA)?;
    let centre = TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    };
    let date = match list.tiles.get(&centre) {
        Some(_) if missing.contains(&centre) => {
            warnings.push(format!(
                "the {} tile {} at the centre of the map area is missing, so there is no date",
                model.label(),
                short_name(centre)
            ));
            None
        }
        Some(entry) if entry.date.is_none() => {
            warnings.push(format!(
                "the {} tile {}: {}, so there is no date",
                model.label(),
                short_name(centre),
                entry.note.as_deref().unwrap_or("no flight days")
            ));
            None
        }
        Some(entry) => entry.date.clone(),
        None => {
            warnings.push(format!(
                "the tile list names no {} tile {} at the centre of the map area, so there is no \
                 date",
                model.label(),
                short_name(centre)
            ));
            None
        }
    };
    missing.sort();
    Ok(HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, rp_tile_list, serve_synthetic_rp_heights};
    use crate::raster::GridSpec;
    use crate::sources::float_tiff::decode_float_tiff;

    /// The live answer around the Görres-Gymnasium, Koblenz, cut to four tiles.
    const TILE_LIST: &str =
        include_str!("../../../../tests/fixtures/rheinland-pfalz-laserbefliegung.gml");

    /// Two tiles side by side in Koblenz (centre at the tile border 400 km).
    const BBOX: BoundingBox = BoundingBox {
        min_e: 399_900.0,
        min_n: 5_579_400.0,
        max_e: 400_100.0,
        max_n: 5_579_600.0,
    };

    const WEST: TileId = TileId {
        east_km: 399,
        north_km: 5579,
    };
    const EAST: TileId = TileId {
        east_km: 400,
        north_km: 5579,
    };

    const fn tile(east_km: u32, north_km: u32) -> TileId {
        TileId { east_km, north_km }
    }

    /// The mosaic `out` is written without GeoTIFF tags (its world file has the grid).
    fn read_out(path: &Path) -> (u32, u32, Vec<f32>) {
        let bytes = std::fs::read(path).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let (width, height) = decoder.dimensions().unwrap();
        let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
            panic!("not float")
        };
        (width, height, values)
    }

    fn run(
        fetcher: &MapFetcher,
        cache: &Cache,
        mode: FetchMode,
        model: Model,
        out: &Path,
    ) -> Result<HeightModel, FetchError> {
        fetch_model(fetcher, cache, mode, model, &BBOX, out)
    }

    fn temp_cache() -> (tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        (dir, cache)
    }

    #[test]
    fn laserbefliegung_gives_names_and_dates() {
        let list = parse_tile_list(TILE_LIST).unwrap();
        assert!(list.warnings.is_empty(), "{:?}", list.warnings);
        let mut keys: Vec<TileId> = list.tiles.keys().copied().collect();
        keys.sort();
        assert_eq!(keys, vec![WEST, tile(400, 5578), EAST, tile(401, 5579)]);
        let west = &list.tiles[&WEST];
        assert_eq!(
            west.name(Model::Dgm1).as_deref(),
            Some("dgm1_32_399_5579_1_rp_2024")
        );
        assert_eq!(
            west.name(Model::Dom1).as_deref(),
            Some("dom1_32_399_5579_1_rp_2024")
        );
        // Flown 30.03.24 and 06.04.24: the record date is the last day.
        assert_eq!(west.date.as_deref(), Some("2024-04-06"));
        assert_eq!(
            list.tiles[&tile(400, 5578)].name(Model::Dgm1).as_deref(),
            Some("dgm1_32_400_5578_1_rp_2024")
        );

        // The other forms of the flight days, and tiles the list cannot name.
        let gml = rp_tile_list(&[
            (
                tile(300, 5500),
                "2023",
                "07.-09.02.23, 15.-17.03.23, 10.04.23",
            ),
            (tile(301, 5500), "2023", "27.02.-01.03.23"),
            (tile(302, 5500), "", "29.02.24, 03.03.24"),
            (tile(303, 5500), "2025", ""),
            (tile(304, 5500), "2024", "30.3.24"),
            // The 13 tiles near Saarburg have neither.
            (tile(333, 5539), "", ""),
            // Days and year that disagree.
            (tile(305, 5500), "2024", "30.03.23"),
            (tile(306, 5500), "", "30.12.23, 02.01.24"),
            (tile(307, 5500), "2024", "30.03.24"),
        ])
        .replace("<ms:kachel>3075500<", "<ms:kachel>../5500<");
        let list = parse_tile_list(&gml).unwrap();
        let at = |e| &list.tiles[&tile(e, 5500)];
        let name = |e| at(e).name(Model::Dgm1);
        assert_eq!(name(300).as_deref(), Some("dgm1_32_300_5500_1_rp_2023"));
        assert_eq!(at(300).date.as_deref(), Some("2023-04-10"));
        assert_eq!(at(301).date.as_deref(), Some("2023-03-01"));
        assert_eq!(name(302).as_deref(), Some("dgm1_32_302_5500_1_rp_2024"));
        assert_eq!(at(302).date.as_deref(), Some("2024-03-03"));
        assert_eq!(name(303).as_deref(), Some("dgm1_32_303_5500_1_rp_2025"));
        assert_eq!(at(303).date, None);
        assert_eq!(name(304).as_deref(), Some("dgm1_32_304_5500_1_rp_2024"));
        assert_eq!(at(304).date, None);
        assert!(at(304).note.is_some());
        let saarburg = &list.tiles[&tile(333, 5539)];
        assert_eq!(saarburg.name(Model::Dgm1), None);
        assert!(saarburg.note.is_some());
        for e in [305, 306] {
            assert_eq!(name(e), None, "{e}");
            assert!(at(e).note.is_some(), "{e}");
        }
        assert!(!list.tiles.contains_key(&tile(307, 5500)));
        assert_eq!(list.warnings.len(), 1, "{:?}", list.warnings);

        // Names are checked before they become URLs.
        assert!(valid_name(Model::Dgm1, "dgm1_32_400_5579_1_rp_2024"));
        for bad in [
            "dom1_32_400_5579_1_rp_2024",
            "dgm1_32_400_5579_1_rp_24",
            "dgm1_32_40_5579_1_rp_2024",
            "dgm1_32_400_5579_1_rp_2024/../x",
            "dgm1_32_400_5579_1_nw_2024",
            "dgm1_32_400_5579_2_rp_2024",
            "dgm1_32_400_5579_1_rp_2024_",
        ] {
            assert!(!valid_name(Model::Dgm1, bad), "{bad}");
        }

        // A cut answer is not a short list.
        let cut = TILE_LIST.replace("numberMatched=\"4\"", "numberMatched=\"25\"");
        assert!(parse_tile_list(&cut).is_err());
    }

    #[test]
    fn rp_tiles_mosaic_into_the_area() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        // The live list: it also names the neighbours that touch the box.
        fetcher.insert_prefix(&tile_list_prefix(), TILE_LIST.as_bytes().to_vec());
        let (cache_dir, cache) = temp_cache();
        let out = tempfile::tempdir().unwrap();
        for (model, lift) in [(Model::Dgm1, 0.0), (Model::Dom1, 12.0)] {
            let tif = out.path().join(format!("{}.tif", model.slug()));
            let found = run(&fetcher, &cache, FetchMode::Online, model, &tif).unwrap();
            assert!(found.missing.is_empty(), "{:?}", found.missing);
            assert_eq!(found.date.as_deref(), Some("2024-04-06"));
            assert!(found.warnings.is_empty(), "{:?}", found.warnings);
            let (width, height, data) = read_out(&tif);
            assert_eq!((width, height), (200, 200), "{model:?}");
            assert!(data.iter().all(|z| z.is_finite()), "a gap in {model:?}");
            let at = |x: usize, y: usize| data[y * 200 + x];
            // The terrain rises 0.01 m per metre east inside each tile (from 20 m): the last
            // column of the west tile, then the first of the east one.
            for (x, want) in [(0, 29.0), (99, 29.99), (100, 20.0), (199, 20.99)] {
                let z = at(x, 100);
                assert!((z - (want + lift)).abs() < 1e-3, "{model:?} {x}: {z}");
            }
        }
        assert!(
            cache_dir
                .path()
                .join("lvermgeo_rp/dgm1/dgm1_32_399_5579_1_rp_2024.tif")
                .is_file()
        );
        // Only the area's tiles are downloaded, from the origin host.
        let log = fetcher.log.borrow();
        let tiles: Vec<&String> = log.iter().filter(|u| u.ends_with(".tif")).collect();
        assert_eq!(tiles.len(), 4, "{tiles:?}");
        assert!(
            tiles
                .iter()
                .all(|u| u.starts_with("https://geobasis-rlp.de/data/"))
        );
        assert!(log[0].starts_with(TILE_LIST_URL), "{}", log[0]);
        assert!(
            log[0].contains("BBOX=399900,5579400,400100,5579600,urn:ogc:def:crs:EPSG::25832"),
            "{}",
            log[0]
        );
    }

    #[test]
    fn listed_but_missing_tile_is_reported() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let east_url = tile_url(Model::Dgm1, "dgm1_32_400_5579_1_rp_2024");
        fetcher.responses.borrow_mut().remove(&east_url);
        let (_cache_dir, cache) = temp_cache();
        let out = tempfile::tempdir().unwrap();
        let tif = out.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(found.missing, vec![EAST]);
        let (_, _, data) = read_out(&tif);
        assert!(data[100 * 200 + 10].is_finite());
        assert!(data[100 * 200 + 190].is_nan());
        // The centre of the area lies in the missing east tile.
        assert_eq!(found.date, None);
        // Remembered: neither asked again online nor an error offline.
        fetcher.log.borrow_mut().clear();
        let again = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(again.missing, vec![EAST]);
        assert!(!fetcher.log.borrow().contains(&east_url));
        let offline = run(
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &tif,
        )
        .unwrap();
        assert_eq!(offline.missing, vec![EAST]);
        // Nothing of the area at all is an error.
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_399_5579_1_rp_2024"));
        let (_empty_dir, empty) = temp_cache();
        let err = run(
            &fetcher,
            &empty,
            FetchMode::Online,
            Model::Dgm1,
            &out.path().join("none.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no tile"), "{err}");
    }

    #[test]
    fn a_missing_tile_is_asked_again_after_a_month() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let east_url = tile_url(Model::Dgm1, "dgm1_32_400_5579_1_rp_2024");
        let east_file = fetcher.responses.borrow_mut().remove(&east_url).unwrap();
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        let marker = cache_dir
            .path()
            .join("lvermgeo_rp/dgm1/dgm1_32_400_5579_1_rp_2024.missing");
        let month_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 86_400);
        std::fs::File::options()
            .write(true)
            .open(&marker)
            .unwrap()
            .set_modified(month_ago)
            .unwrap();
        fetcher.insert(&east_url, east_file);
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert!(!marker.exists());
    }

    #[test]
    fn a_listed_tile_without_a_year_is_missing_with_a_warning() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(found.missing, vec![EAST]);
        assert_eq!(found.date, None);
        assert!(
            found.warnings.iter().any(|w| w.contains("32_400_5579")),
            "{:?}",
            found.warnings
        );
        // No year was tried.
        assert!(
            !fetcher.log.borrow().iter().any(|u| u.contains("400_5579")),
            "{:?}",
            fetcher.log.borrow()
        );
    }

    #[test]
    fn html_answers_are_errors() {
        let page = b"<!DOCTYPE html><html>\x1b[2J<body>No wfs_conf data\xe2\x80\xae</body></html>";
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        fetcher.insert_prefix(&tile_list_prefix(), page.to_vec());
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let err = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a tile list"), "{err}");
        assert!(err.contains("No wfs_conf data"), "{err}");
        assert!(!err.contains('\x1b') && !err.contains('\u{202e}'), "{err}");
        // The tile host.
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        for name in ["dgm1_32_399_5579_1_rp_2024", "dgm1_32_400_5579_1_rp_2024"] {
            fetcher.insert(&tile_url(Model::Dgm1, name), page.to_vec());
        }
        let err = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a GeoTIFF"), "{err}");
        assert!(err.contains("No wfs_conf data"), "{err}");
        assert!(!err.contains('\x1b') && !err.contains('\u{202e}'), "{err}");
    }

    #[test]
    fn offline_reads_the_kept_list_and_tiles() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let (_cache_dir, cache) = temp_cache();
        let out = tempfile::tempdir().unwrap();
        let tif = out.path().join("dgm1.tif");
        assert!(cached_tiles(&cache, Model::Dgm1, &[WEST, EAST]).is_empty());
        let online = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &[WEST, EAST]),
            [WEST, EAST].into_iter().collect()
        );
        assert!(cached_tiles(&cache, Model::Dom1, &[WEST, EAST]).is_empty());
        // Offline: nothing is requested.
        let silent = MapFetcher::default();
        let offline = run(&silent, &cache, FetchMode::Offline, Model::Dgm1, &tif).unwrap();
        assert_eq!(offline, online);
        assert_eq!(silent.calls.get(), 0);
        // Online with a dead server: the kept list and tiles, with a warning.
        let kept = run(&silent, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(kept.date, online.date);
        assert_eq!(kept.warnings.len(), 1, "{:?}", kept.warnings);
        // Never fetched: offline says so.
        let (_empty_dir, empty) = temp_cache();
        let err = run(&silent, &empty, FetchMode::Offline, Model::Dgm1, &tif).unwrap_err();
        assert!(err.to_string().contains("offline and not cached"), "{err}");
        // The DOM1 tiles were not fetched: offline that is an error too.
        let err = run(&silent, &cache, FetchMode::Offline, Model::Dom1, &tif).unwrap_err();
        assert!(err.to_string().contains("offline and not cached"), "{err}");
    }

    #[test]
    fn predictor2_tile_decodes() {
        // The live DGM1 (LZW, predictor 2, strips of 2 rows), its northern 100 rows.
        let bytes = include_bytes!("../../../../tests/fixtures/rheinland-pfalz-dgm1.tif");
        let raster = decode_float_tiff(bytes).unwrap();
        let g = raster.grid;
        assert_eq!((g.width, g.height, g.pixel_size), (1000, 100, 1.0));
        assert_eq!((g.west, g.north), (400_000.0, 5_580_000.0));
        // Values as libtiff reads them.
        for (x, y, want) in [(0, 0, 61.51), (999, 0, 61.32), (500, 99, 64.66)] {
            let z = raster.get(x, y);
            assert!((z - want).abs() < 1e-3, "{x} {y}: {z}");
        }
        assert!(
            raster
                .data
                .iter()
                .all(|z| (61.20..66.79).contains(z) && z.is_finite())
        );
        // The DOM1 of 2022-2024: no predictor, strips of 20 rows, no world file; the grid comes
        // from the GeoTIFF tags.
        let bytes = include_bytes!("../../../../tests/fixtures/rheinland-pfalz-dom1.tif");
        let raster = decode_float_tiff(bytes).unwrap();
        let g = raster.grid;
        assert_eq!((g.width, g.height, g.pixel_size), (1000, 20, 1.0));
        assert_eq!((g.west, g.north), (400_000.0, 5_580_000.0));
        for (x, y, want) in [(0, 0, 61.496), (999, 0, 61.326), (500, 19, 61.516)] {
            let z = raster.get(x, y);
            assert!((z - want).abs() < 1e-4, "{x} {y}: {z}");
        }
        // NoData (−9999) of a predictor 2 tile becomes NaN.
        let grid = GridSpec {
            west: 400_000.0,
            north: 5_580_000.0,
            pixel_size: 1.0,
            width: 4,
            height: 2,
        };
        let values = [61.5, -9999.0, 61.7, 61.8, -9999.0, -9999.0, 62.0, 62.1];
        let raster = decode_float_tiff(&predictor2_tiff(&grid, &values)).unwrap();
        assert_eq!(raster.grid, grid);
        let read: Vec<String> = raster.data.iter().map(|z| format!("{z}")).collect();
        assert_eq!(
            read,
            ["61.5", "NaN", "61.7", "61.8", "NaN", "NaN", "62", "62.1"]
        );
    }

    /// A Float32 GeoTIFF on `grid`, LZW with predictor 2, like the LVermGeo RP tiles: the bit
    /// patterns written as 32-bit words (the encoder differences those, as libtiff does), the
    /// sample format set to float afterwards.
    fn predictor2_tiff(grid: &GridSpec, values: &[f32]) -> Vec<u8> {
        use tiff::encoder::{Compression, Predictor, TiffEncoder, colortype::Gray32};
        use tiff::tags::Tag;
        let words: Vec<u32> = values.iter().map(|z| z.to_bits()).collect();
        let mut tiff = std::io::Cursor::new(Vec::new());
        {
            let mut encoder = TiffEncoder::new(&mut tiff)
                .unwrap()
                .with_compression(Compression::Lzw)
                .with_predictor(Predictor::Horizontal);
            let mut image = encoder
                .new_image::<Gray32>(grid.width, grid.height)
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
            image.write_data(&words).unwrap();
        }
        let mut bytes = tiff.into_inner();
        let ifd = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let count = usize::from(u16::from_le_bytes([bytes[ifd], bytes[ifd + 1]]));
        let entry = (0..count)
            .map(|i| ifd + 2 + 12 * i)
            .find(|&e| u16::from_le_bytes([bytes[e], bytes[e + 1]]) == 339)
            .expect("a SampleFormat tag");
        bytes[entry + 8..entry + 10].copy_from_slice(&3u16.to_le_bytes());
        bytes
    }

    /// Needs the network: the Görres-Gymnasium, Koblenz, tile 400/5579, 200 m box.
    #[test]
    #[ignore]
    fn live_koblenz_school() {
        let bbox = BoundingBox {
            min_e: 400_400.0,
            min_n: 5_579_400.0,
            max_e: 400_600.0,
            max_n: 5_579_600.0,
        };
        let (cache_dir, cache) = temp_cache();
        let fetcher = crate::fetch::HttpFetcher::new();
        for model in [Model::Dgm1, Model::Dom1] {
            let tif = cache_dir.path().join(format!("{}.tif", model.slug()));
            let found =
                fetch_model(&fetcher, &cache, FetchMode::Online, model, &bbox, &tif).unwrap();
            let (_, _, data) = read_out(&tif);
            let finite: Vec<f32> = data.iter().copied().filter(|z| z.is_finite()).collect();
            println!("{model:?}: {found:?}; {} cells", finite.len());
            assert!(finite.len() > 39_000);
        }
    }
}
