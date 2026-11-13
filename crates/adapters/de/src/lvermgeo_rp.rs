//! LVermGeo Rheinland-Pfalz (`geobasis-rlp.de`, dl-de/by-2-0, "©GeoBasis-DE / LVermGeoRP<year>"):
//! the DGM1 and DOM1 1 km GeoTIFF tiles (Float32, LZW, EPSG:25832).
//!
//! A tile's file name carries the year of its laser scan, which differs per tile. The tile list
//! is the `Laserbefliegung` layer of the Blattschnitt WFS: it gives the flight days per tile, and
//! the year in the name follows from them. Names are never guessed by trying years. Everything
//! comes from the origin hosts; the GeoPortal.rlp registry proxy answers errors with HTML and
//! status 200.

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;

use crate::note::{Arg, Note, note};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{Datelike, NaiveDate};

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{
    CacheDir, FetchError, FetchMode, Fetcher, get_limited_with_retries, get_with_retries, one_line,
    write_cache_file,
};
use crate::sources::alkis_wfs::{number_matched, parse_gml};
use crate::sources::height_model::{
    AreaTiles, HeightModel, TileStore, centre_date, centre_tile, mosaic_tiles, short_name,
};
use crate::sources::region::{RHEINLAND_PFALZ, tile_centre_in, tile_touches};
use crate::sources::wms::{DateLabel, WmsService};
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS (the tiles are lossless JPEG 2000 of 105-155 MB). It allows 10000 px a
/// side, and a larger request is an HTTP 200 XML exception. It is white outside the state, also
/// where the DGM1 and DOM1 tiles reach across the border.
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://geo4.service24.rlp.de/wms/rp_dop20.fcgi"),
    layer: Cow::Borrowed("rp_dop20"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};
/// The queryable layer with the date of each 2 km block.
const WMS_INFO_LAYER: &str = "rp_dop20_info";
/// freemapper's name for the date taken as the flight date. The service calls it `erstellung`
/// (creation) and has no field for the flight date; every block is dated in May to September,
/// so the date is taken as the flight's.
pub const FLIGHT_LABEL: DateLabel = DateLabel {
    record: "Aktualität (Erstellung)",
    shown: "date of creation",
};

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832): the
/// `erstellung` of its block, by a GetFeatureInfo on the metadata layer; `None` outside the
/// state or when the answer has none.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let url = format!(
        "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={WMS_INFO_LAYER}\
         &QUERY_LAYERS={WMS_INFO_LAYER}&STYLES=&CRS=EPSG:{}&BBOX={},{},{},{}\
         &WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain",
        WMS.url,
        WMS.crs.epsg(),
        crate::sources::wms::mm(e - 100.0),
        crate::sources::wms::mm(n - 100.0),
        crate::sources::wms::mm(e + 100.0),
        crate::sources::wms::mm(n + 100.0),
    );
    let bytes = get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    parse_flight_date(&String::from_utf8_lossy(&bytes))
}

/// The line `erstellung = 'DD.MM.YYYY'` of a GetFeatureInfo text answer.
fn parse_flight_date(text: &str) -> Option<String> {
    let value = text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("erstellung = '")?
            .strip_suffix('\'')
    })?;
    let parts: Vec<&str> = value.split('.').collect();
    let [d, m, y] = parts[..] else { return None };
    (digits(d, 2) && digits(m, 2) && digits(y, 4)).then(|| format!("{d}.{m}.{y}"))
}

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
pub fn tile_list_prefix() -> String {
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
    /// Why the year or the date is missing, as what the tile list does (`gives no flight days`).
    note: Option<Note>,
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
    warnings: Vec<Note>,
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
        (None, None) => (None, None, Some(note!("word-rp-list-neither"))),
        (Some(year), None) => (Some(year), None, Some(note!("word-rp-list-no-days"))),
        (year, Some(None)) => (
            year,
            None,
            Some(note!("word-rp-list-days-unreadable", days = quoted(days))),
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
                    Some(note!(
                        "word-rp-list-disagree",
                        days = quoted(days),
                        year = quoted(flight_year)
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
        warnings.push(note!(
            "warning-rp-list-features-unusable",
            count = unreadable
        ));
    }
    Ok(TileList { tiles, warnings })
}

/// The largest tile list answer accepted (1.3 KB per tile; a 5 x 5 km area is 50 KB); a larger
/// one is stopped while it is read.
const MAX_LIST_BYTES: u64 = 32 * 1024 * 1024;

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
    warnings: &mut Vec<Note>,
) -> Result<TileList, FetchError> {
    let url = tile_list_url(bbox);
    let path = list_path(dir, bbox);
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
    let part = path.with_extension("gml.part");
    let answer = get_limited_with_retries(fetcher, &url, &part, MAX_LIST_BYTES, |bytes| {
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
            write_cache_file(&path, &bytes)?;
            Ok(list)
        }
        Err(e) => {
            let Some(list) = kept() else {
                return Err(e);
            };
            warnings.push(note!(
                "warning-rp-list-unreadable-kept",
                error = Arg::error(e)
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
/// the server got new files, so a missing tile is asked for again after this. A kept Metalink
/// is used online for as long.
const RECHECK_AFTER: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Whether the 404 note at `path` holds: offline always, online while it is recent.
fn known_missing(path: &Path, mode: FetchMode) -> bool {
    if std::fs::metadata(path).is_err() {
        return false;
    }
    mode == FetchMode::Offline || crate::clock::age_of(path).is_some_and(|age| age < RECHECK_AFTER)
}

/// The Metalink of `model` for the whole Land (AGS `07`, 11 MB): file name, size and SHA-256 of
/// every file. The Kreis and Gemeinde ones are smaller, but which of them covers the map area
/// would need the area's municipality key.
pub fn metalink_url(model: Model) -> String {
    let slug = model.slug();
    format!("{DATA}/{slug}/current/meta4/{slug}_tif_07.meta4")
}

/// The largest Metalink accepted (11 MB expected); a larger one is stopped while it is read.
const MAX_METALINK_BYTES: u64 = 32 * 1024 * 1024;

/// The GeoTIFF tile names of `model` in a Metalink (`<file name="dgm1_32_400_5579_1_rp_2024.tif">`),
/// by tile; a name the strict pattern refuses is skipped, and of two years the later wins. An
/// answer that is not a Metalink (an HTML page) is an error.
fn parse_metalink(model: Model, text: &str) -> Result<HashMap<TileId, String>, String> {
    let mut names: HashMap<TileId, String> = HashMap::new();
    for name in freemapper_sources::metalink::file_names(text)? {
        let Some(stem) = name.strip_suffix(".tif") else {
            continue;
        };
        if !valid_name(model, stem) {
            continue;
        }
        // Checked: `dgm1_32_EEE_NNNN_…`.
        let parts: Vec<&str> = stem.split('_').collect();
        let (Ok(east_km), Ok(north_km)) = (parts[2].parse(), parts[3].parse()) else {
            continue;
        };
        let tile = TileId { east_km, north_km };
        if names.get(&tile).is_none_or(|old| old.as_str() < stem) {
            names.insert(tile, stem.to_string());
        }
    }
    Ok(names)
}

/// Where the Metalink of `model` is kept.
fn metalink_path(dir: &Path, model: Model) -> PathBuf {
    dir.join("meta4")
        .join(format!("{}_tif_07.meta4", model.slug()))
}

/// The kept Metalink of `model`, read.
fn kept_metalink(dir: &Path, model: Model) -> Option<HashMap<TileId, String>> {
    let bytes = std::fs::read(metalink_path(dir, model)).ok()?;
    parse_metalink(model, &String::from_utf8_lossy(&bytes)).ok()
}

/// The tile names of the Metalink of `model`, for tiles the tile list gives no year. A kept one
/// younger than [`RECHECK_AFTER`] is used as it is; else it is downloaded (at most once per run
/// and product, as only `fetch_model` asks) and kept. Offline, or when the download fails, the
/// kept one of any age. `None`, with a warning, when there is none. DGM1 and DOM1 have a
/// Metalink each.
fn metalink_names(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    model: Model,
    warnings: &mut Vec<Note>,
) -> Option<HashMap<TileId, String>> {
    let path = metalink_path(dir, model);
    let error = match mode {
        FetchMode::Offline => "offline and not kept from an earlier run".to_string(),
        FetchMode::Online => {
            if let Some(names) = fresh_metalink(dir, model) {
                return Some(names);
            }
            let url = metalink_url(model);
            let part = path.with_extension("meta4.part");
            let answer =
                get_limited_with_retries(fetcher, &url, &part, MAX_METALINK_BYTES, |bytes| {
                    let names = parse_metalink(model, &String::from_utf8_lossy(bytes)).map_err(
                        |message| FetchError::Http {
                            url: url.clone(),
                            message,
                        },
                    )?;
                    Ok((names, bytes.to_vec()))
                });
            match answer {
                Ok((names, bytes)) => match write_cache_file(&path, &bytes) {
                    Ok(()) => return Some(names),
                    Err(e) => {
                        warnings.push(note!("warning-rp-metalink-not-kept", error = Arg::error(e)));
                        return Some(names);
                    }
                },
                Err(e) => e.to_string(),
            }
        }
    };
    match kept_metalink(dir, model) {
        Some(names) => {
            if mode == FetchMode::Online {
                warnings.push(note!(
                    "warning-rp-metalink-unreadable-kept",
                    model = crate::note::model(model.label()),
                    detail = &error
                ));
            }
            Some(names)
        }
        None => {
            warnings.push(note!(
                "warning-rp-metalink-unreadable",
                model = crate::note::model(model.label()),
                detail = &error
            ));
            None
        }
    }
}

/// The size of a Metalink download (the Land file, 11 MB), for the estimate.
pub const METALINK_MB: f64 = 11.0;

/// Whether a run over `tiles` of the area `bbox` (EPSG:25832) would download the Metalink of
/// `model` (an estimate before a run): the tile list kept for this area gives one of them no
/// year, and no Metalink kept within [`RECHECK_AFTER`] can stand in.
pub fn metalink_to_fetch(
    cache: &CacheDir,
    model: Model,
    bbox: &BoundingBox,
    tiles: &[TileId],
) -> bool {
    let dir = cache.provider_dir("lvermgeo_rp");
    let Some(list) = kept_list(&dir, bbox) else {
        return false;
    };
    let unnamed = tiles.iter().any(|tile| {
        list.tiles
            .get(tile)
            .is_some_and(|entry| entry.name(model).is_none())
    });
    unnamed && fresh_metalink(&dir, model).is_none()
}

/// The tile list kept from an earlier run over `bbox`.
fn kept_list(dir: &Path, bbox: &BoundingBox) -> Option<TileList> {
    let bytes = std::fs::read(list_path(dir, bbox)).ok()?;
    parse_tile_list(&String::from_utf8_lossy(&bytes)).ok()
}

/// The kept Metalink of `model` if it was downloaded within [`RECHECK_AFTER`].
fn fresh_metalink(dir: &Path, model: Model) -> Option<HashMap<TileId, String>> {
    let age = crate::clock::age_of(&metalink_path(dir, model))?;
    (age < RECHECK_AFTER)
        .then(|| kept_metalink(dir, model))
        .flatten()
}

/// The tiles among `tiles` of the area `bbox` that need no download (an estimate before a run):
/// the file the tile list kept for this area names (with its scan year) is cached, the server
/// answered 404 for it lately, or the list gives the tile no year and a Metalink kept within
/// [`RECHECK_AFTER`] names no file for it either (or names a cached one). Without such a
/// Metalink the run downloads one, and it is taken to name the file. Without a kept list, a file
/// of any year counts.
pub fn cached_tiles(
    cache: &CacheDir,
    model: Model,
    bbox: &BoundingBox,
    tiles: &[TileId],
) -> HashSet<TileId> {
    let dir = cache.provider_dir("lvermgeo_rp");
    let files: HashSet<String> = std::fs::read_dir(dir.join(model.slug()))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let ok = name.ends_with(".tif")
                || (name.ends_with(".missing") && known_missing(&entry.path(), FetchMode::Online));
            ok.then_some(name)
        })
        .collect();
    let has = |stem: &str| {
        files.contains(&format!("{stem}.tif")) || files.contains(&format!("{stem}.missing"))
    };
    let list = kept_list(&dir, bbox);
    let metalink = list
        .as_ref()
        .filter(|list| list.tiles.values().any(|entry| entry.name(model).is_none()))
        .and_then(|_| fresh_metalink(&dir, model));
    tiles
        .iter()
        .filter(
            |tile| match list.as_ref().and_then(|list| list.tiles.get(tile)) {
                Some(entry) => match entry.name(model) {
                    Some(name) => has(&name),
                    None => metalink.as_ref().is_some_and(|names| {
                        names
                            .get(tile)
                            .is_none_or(|file| has(file.trim_end_matches(".tif")))
                    }),
                },
                None => {
                    let prefix = format!(
                        "{}_32_{:03}_{:04}_1_rp_",
                        model.slug(),
                        tile.east_km,
                        tile.north_km
                    );
                    files.iter().any(|name| name.starts_with(&prefix))
                }
            },
        )
        .copied()
        .collect()
}

/// The path of the tile list kept for `bbox` (EPSG:25832) under `dir`.
pub fn list_path(dir: &Path, bbox: &BoundingBox) -> PathBuf {
    dir.join("tile_list")
        .join(format!("{}.gml", bbox_key(bbox)))
}

/// The tiles a run of [`fetch_model`] over `bbox` (EPSG:25832) would download, as an estimate
/// before the run (cached ones not left out yet). With a tile list kept for this area they are
/// the ones it names, as in the run, also those wholly beyond the state border. Without one they
/// are the tiles that touch the state.
pub fn tiles_to_fetch(cache: &CacheDir, bbox: &BoundingBox) -> Vec<TileId> {
    let kept = std::fs::read(list_path(&cache.provider_dir("lvermgeo_rp"), bbox))
        .ok()
        .and_then(|bytes| parse_tile_list(&String::from_utf8_lossy(&bytes)).ok());
    tiles_covering(bbox)
        .into_iter()
        .filter(|tile| match &kept {
            Some(list) => list.tiles.contains_key(tile),
            None => tile_touches(*tile, RHEINLAND_PFALZ),
        })
        .collect()
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
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let dir = cache.provider_dir("lvermgeo_rp");
    let mut warnings = Vec::new();
    let list = tile_list(fetcher, &dir, mode, bbox, &mut warnings)?;
    warnings.extend(list.warnings.iter().cloned());
    let mut area = AreaTiles::default();
    let mut unnamed = Vec::new();
    for tile in tiles_covering(bbox) {
        let Some(entry) = list.tiles.get(&tile) else {
            if tile_centre_in(tile, RHEINLAND_PFALZ) {
                area.missing
                    .push((tile, "not in LVermGeo RP's tile list".to_string()));
            }
            continue;
        };
        match entry.name(model) {
            Some(name) => area.named.push((tile, name)),
            None => unnamed.push(entry),
        }
    }
    // The tile list leaves these without a year; the Metalink has the exact names.
    let metalink = if unnamed.is_empty() {
        None
    } else {
        metalink_names(fetcher, &dir, mode, model, &mut warnings)
    };
    for entry in unnamed {
        if let Some(name) = metalink.as_ref().and_then(|names| names.get(&entry.tile)) {
            area.named.push((entry.tile, name.clone()));
            continue;
        }
        let note = entry
            .note
            .clone()
            .unwrap_or_else(|| note!("word-rp-list-no-year"));
        let (either, why) = if metalink.is_some() {
            (
                "metalink",
                "no flight year in LVermGeo RP's tile list (Laserbefliegung) and no file in its \
                 Metalink",
            )
        } else {
            (
                "none",
                "no flight year in LVermGeo RP's tile list (Laserbefliegung)",
            )
        };
        warnings.push(note!(
            "warning-rp-tile-without-name",
            model = crate::note::model(model.label()),
            tile = short_name(entry.tile),
            note = note,
            either = either
        ));
        area.missing.push((entry.tile, why.to_string()));
    }
    let list_url = tile_list_url(bbox);
    let store = TileStore {
        label: model.label(),
        list_url: &list_url,
        source: DATA,
        url: &|name| tile_url(model, name),
        cache_path: &|name| cache_path(&dir, model, name),
        marker_path: &|name| marker_path(&dir, model, name),
        known_missing: &|path| known_missing(path, mode),
        // The file server answers some errors with an HTML page.
        precheck: &|bytes| {
            if is_tiff(bytes) {
                return Ok(());
            }
            let start = String::from_utf8_lossy(&bytes[..bytes.len().min(1000)]);
            Err(format!("not a GeoTIFF: {}", one_line(&start, 200)))
        },
        prepare: &crate::sources::height_model::as_delivered,
        explain_empty: true,
    };
    let missing = mosaic_tiles(fetcher, mode, bbox, &store, area, root, out, &mut warnings)?;
    let centre = centre_tile(bbox);
    let listed = list.tiles.get(&centre).map(|entry| {
        entry.date.clone().ok_or_else(|| {
            note!(
                "warning-rp-centre-no-date",
                model = crate::note::model(model.label()),
                tile = short_name(centre),
                note = entry
                    .note
                    .clone()
                    .unwrap_or_else(|| note!("word-rp-list-no-days"))
            )
        })
    });
    let date = centre_date(bbox, model.label(), listed, &missing, &mut warnings);
    Ok(HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    })
}

/// The provider id of the adapters.
const PROVIDER: &str = "lvermgeorp";

/// The rasters of LVermGeo RP (Rhineland-Palatinate): requested for the area and stored in the project folder.
pub struct LvermgeoRpBackend {
    info: ProviderInfo,
}

impl LvermgeoRpBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some("lvermgeo_rp".to_string()),
                terrain_source: "lvermgeo-rp-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for LvermgeoRpBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LvermgeoRpBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox)
    }
}

/// The LVermGeo RP rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, dated by the
/// `erstellung` of its block, and DGM1 and DOM1 from the 1 km tiles the `Laserbefliegung` list
/// names. The DOM1 is only a template: an unavailable one costs a warning.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Result<(), AdapterError> {
    let (fetcher, mode) = (env.fetcher, env.mode);
    let cache = env.cache;
    let dir = &project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    // The WMS is white beyond the state border, while the tiles reach across it.
    warn_beyond_state(env, project, PROVIDER, "LVermGeo RP");
    let flight = || flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(WMS),
            "LVermGeo RP",
            &flight,
            // The block's creation date, taken as the flight date: the record and the log say so.
            Some(&FLIGHT_LABEL),
        ),
    )?;
    fetch_area_heights(
        env,
        project,
        bbox,
        "the map area",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            fetch_model(fetcher, cache, mode, model, bbox, dir, out)
        },
        &mut |_, found, _, progress| {
            for warning in &found.warnings {
                progress(warning.clone());
            }
        },
    )?;
    project.save()?;
    Ok(())
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    use crate::fetch::test_support::{MapFetcher, geotiff, solid_jpeg};
    use crate::tiles::TileId;

    /// A `Laserbefliegung` answer of the LVermGeo RP Blattschnitt WFS (GML 3.2, as MapServer
    /// writes it) with a feature per `(tile, flugjahr, flugtage)`; an empty `flugjahr` or
    /// `flugtage` is left out, as the server does.
    pub fn rp_tile_list(rows: &[(TileId, &str, &str)]) -> String {
        let mut gml = format!(
            "<?xml version='1.0' encoding=\"UTF-8\" ?>\n<wfs:FeatureCollection \
             xmlns:ms=\"http://mapserver.gis.umn.edu/mapserver\" \
             xmlns:gml=\"http://www.opengis.net/gml/3.2\" \
             xmlns:wfs=\"http://www.opengis.net/wfs/2.0\" timeStamp=\"2026-11-08T07:03:21\" \
             numberMatched=\"{0}\" numberReturned=\"{0}\">\n",
            rows.len()
        );
        for (gid, (tile, year, days)) in rows.iter().enumerate() {
            let (e, n) = (tile.east_km * 1000, tile.north_km * 1000);
            gml.push_str(&format!(
                "<wfs:member><ms:Laserbefliegung gml:id=\"Laserbefliegung.{gid}\"><ms:msGeometry>\
                 <gml:Polygon gml:id=\"Laserbefliegung.{gid}.1\" \
                 srsName=\"urn:ogc:def:crs:EPSG::25832\"><gml:exterior><gml:LinearRing>\
                 <gml:posList srsDimension=\"2\">{e} {n1} {e1} {n1} {e1} {n} {e} {n} {e} {n1} \
                 </gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></ms:msGeometry>\
                 <ms:gid>{gid}</ms:gid><ms:art>Laserbefliegung</ms:art>\
                 <ms:betrifft>DGM, DOM, Laserpunkte</ms:betrifft>\
                 <ms:kachel>{:03}{:04}</ms:kachel>",
                tile.east_km,
                tile.north_km,
                e1 = e + 1000,
                n1 = n + 1000,
            ));
            if !year.is_empty() {
                gml.push_str(&format!("<ms:flugjahr>{year}</ms:flugjahr>"));
            }
            if !days.is_empty() {
                gml.push_str(&format!("<ms:flugtage>{days}</ms:flugtage>"));
            }
            gml.push_str("</ms:Laserbefliegung></wfs:member>\n");
        }
        gml.push_str("</wfs:FeatureCollection>\n");
        gml
    }

    /// The GetFeatureInfo text answer of the LVermGeo RP orthophoto WMS at the Görres-Gymnasium,
    /// Koblenz (block 400/5578, `erstellung` 25.08.2025), as the server gives it.
    pub const RP_FLIGHT_INFO: &str = "GetFeatureInfo results:\n\nLayer 'rp_dop20_info'\n  \
        Feature 5098: \n\nLayer 'metadaten'\n  Feature 5098: \n    gid = '5098'\n    \
        publikation = '2026-01-11 02:15:10.166'\n    name = 'dop20rgbi_32400_5578'\n    \
        titel = 'Digitale Orthophotos 0,2m Rheinland-Pfalz (DOP20RGB) - 4005578'\n    \
        erstellung = '25.08.2025'\n    aktualisierung = 'n/a'\n    \
        csw-anfrage = 'https://metadatenkatalog-vermkv.service24.rlp.de/extern/vertriebsprodukte?\
        request=GetRecords&service=CSW&version=2.0.2'\n";

    /// Serves the LVermGeo RP height models of a project in `bbox` (EPSG:25832): the tile list
    /// naming every tile of `bbox` (flown 30.03.24 and 06.04.24, file names with `_2024`) and
    /// the DGM1 and DOM1 tiles (terrain 20 m rising 0.01 m per metre east, the surface 12 m
    /// above).
    pub fn serve_synthetic_rp_heights(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use super::{Model, tile_list_prefix, tile_url};
        let tiles = crate::tiles::tiles_covering(bbox);
        let rows: Vec<(TileId, &str, &str)> = tiles
            .iter()
            .map(|t| (*t, "2024", "30.03.24, 06.04.24"))
            .collect();
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        for (model, slug, lift) in [(Model::Dgm1, "dgm1", 0.0), (Model::Dom1, "dom1", 12.0)] {
            for tile in &tiles {
                let grid = crate::raster::GridSpec {
                    west: f64::from(tile.east_km) * 1000.0,
                    north: f64::from(tile.north_km + 1) * 1000.0,
                    pixel_size: 1.0,
                    width: 1000,
                    height: 1000,
                };
                let heights: Vec<f32> = (0..1000 * 1000)
                    .map(|i| 20.0 + (i % 1000) as f32 * 0.01 + lift)
                    .collect();
                let name = format!("{slug}_32_{}_{}_1_rp_2024", tile.east_km, tile.north_km);
                fetcher.insert(&tile_url(model, &name), geotiff(&grid, &heights));
            }
        }
    }

    /// Serves the LVermGeo RP rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the
    /// right size with the flight date [`RP_FLIGHT_INFO`] (25.08.2025) and the height models of
    /// [`serve_synthetic_rp_heights`].
    pub fn serve_synthetic_rp(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use super::WMS;
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [80, 120, 60]));
        }
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            RP_FLIGHT_INFO.as_bytes().to_vec(),
        );
        serve_synthetic_rp_heights(fetcher, bbox);
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{rp_tile_list, serve_synthetic_rp_heights};
    use super::*;
    use crate::fetch::Cache;
    use crate::fetch::test_support::MapFetcher;
    use crate::note::Note;
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
        fetch_model(
            fetcher,
            cache,
            mode,
            model,
            &BBOX,
            out.parent().unwrap(),
            out,
        )
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
            // The 14 tiles north of Trier have neither.
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
        // Every tile has a name: no Metalink.
        assert!(!log.iter().any(|u| u.ends_with(".meta4")), "{log:?}");
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
    fn rp_rechecks_missing_tiles_after_30_days_by_the_clock() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let east_url = tile_url(Model::Dgm1, "dgm1_32_400_5579_1_rp_2024");
        let east_file = fetcher.responses.borrow_mut().remove(&east_url).unwrap();
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        let dir = cache_dir.path().join("lvermgeo_rp");
        let marker = dir.join("dgm1/dgm1_32_400_5579_1_rp_2024.missing");
        let kept = dir.join("meta4/dgm1_tif_07.meta4");
        std::fs::create_dir_all(kept.parent().unwrap()).unwrap();
        std::fs::write(&kept, metalink(&["dgm1_32_401_5579_1_rp_2025.tif"])).unwrap();
        fetcher.insert(&east_url, east_file);
        // Both files are written now; the clock moves, not their modification time.
        let in_days = |days: i64| chrono::Utc::now().naive_utc() + chrono::Duration::days(days);
        crate::clock::with_fixed(in_days(29), || {
            assert!(known_missing(&marker, FetchMode::Online));
            assert!(fresh_metalink(&dir, Model::Dgm1).is_some());
        });
        crate::clock::with_fixed(in_days(31), || {
            assert!(!known_missing(&marker, FetchMode::Online));
            assert!(known_missing(&marker, FetchMode::Offline));
            assert!(fresh_metalink(&dir, Model::Dgm1).is_none());
            let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
            assert!(found.missing.is_empty(), "{:?}", found.missing);
        });
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
            found.warnings.iter().any(|w| w.contains("32_400_5579")
                && w.contains("no heights in the map")
                && w.contains("LVermGeo RP's tile list (Laserbefliegung)")
                && !w.contains("\u{2014}")),
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
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &[WEST, EAST]).is_empty());
        let online = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &BBOX, &[WEST, EAST]),
            [WEST, EAST].into_iter().collect()
        );
        assert!(cached_tiles(&cache, Model::Dom1, &BBOX, &[WEST, EAST]).is_empty());
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
    fn dom1_with_the_ifd_at_the_end_decodes() {
        // The DOM1 of 2022-2024: no predictor, strips of 20 rows, the IFD behind the pixels, no
        // world file; the grid comes from the GeoTIFF tags.
        let bytes = include_bytes!("../../../../tests/fixtures/rheinland-pfalz-dom1.tif");
        let ifd = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        assert!(ifd > bytes.len() / 2, "the IFD at {ifd} of {}", bytes.len());
        let raster = decode_float_tiff(bytes).unwrap();
        let g = raster.grid;
        assert_eq!((g.width, g.height, g.pixel_size), (1000, 20, 1.0));
        assert_eq!((g.west, g.north), (400_000.0, 5_580_000.0));
        for (x, y, want) in [(0, 0, 61.496), (999, 0, 61.326), (500, 19, 61.516)] {
            let z = raster.get(x, y);
            assert!((z - want).abs() < 1e-4, "{x} {y}: {z}");
        }
    }

    /// A Metalink as the file server writes it, with an entry per file name.
    fn metalink(names: &[&str]) -> Vec<u8> {
        let mut xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metalink \
            xmlns=\"urn:ietf:params:xml:ns:metalink\">\n    \
            <generator>RLP-Met4Generator</generator>\n    \
            <published>2026-08-12 09:59:48.526579+00:00</published>"
            .to_string();
        for name in names {
            xml.push_str(&format!(
                "<file name=\"{name}\">\n        <size>1358359</size>\n        \
                 <url>https://geobasis-rlp.de/data/dgm1/current/tif/{name}</url>\n\n    </file>"
            ));
        }
        xml.push_str("</metalink>\n");
        xml.into_bytes()
    }

    #[test]
    fn an_area_of_unnamed_tiles_says_why_it_has_no_heights() {
        // Inside the block north of Trier: the list names the tiles, but without a year, and the
        // Metalink has no file for them.
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let rows = [(WEST, "", ""), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        fetcher.insert(
            &metalink_url(Model::Dgm1),
            include_bytes!("../../../../tests/fixtures/rheinland-pfalz-dgm1.meta4").to_vec(),
        );
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let err = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(
                "no tile of the terrain model (LVermGeo RP DGM1) covers the map area: no flight \
                 year in LVermGeo RP's tile list (Laserbefliegung) and no file in its Metalink: \
                 32_399_5579, 32_400_5579"
            ),
            "{err}"
        );
        // One tile not on the server, one without a year, no Metalink.
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_399_5579_1_rp_2024"));
        let (_empty_dir, empty) = temp_cache();
        let err = run(&fetcher, &empty, FetchMode::Online, Model::Dgm1, &tif)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(
                "no flight year in LVermGeo RP's tile list (Laserbefliegung): 32_400_5579"
            ),
            "{err}"
        );
        assert!(
            err.contains("not on the server (404): 32_399_5579"),
            "{err}"
        );
    }

    #[test]
    fn the_metalink_names_tiles_the_tile_list_leaves_unnamed() {
        let live = include_str!("../../../../tests/fixtures/rheinland-pfalz-dgm1.meta4");
        let names = parse_metalink(Model::Dgm1, live).unwrap();
        assert_eq!(
            names,
            [(tile(400, 5580), "dgm1_32_400_5580_1_rp_2024".to_string())]
                .into_iter()
                .collect()
        );
        assert!(parse_metalink(Model::Dom1, live).unwrap().is_empty());
        assert!(parse_metalink(Model::Dgm1, "<html>Proxy Error</html>").is_err());
        // Strict names; the later year wins.
        let synthetic = metalink(&[
            "dgm1_32_400_5579_1_rp_2025.tif",
            "dgm1_32_400_5579_1_rp_2025.tfw",
            "dgm1_32_400_5579_1_rp_2023.tif",
            "dgm1_32_399_5579_1_rp_2026/../x.tif",
            "dgm1_32_399_5579_1_rp_20260.tif",
        ]);
        let names = parse_metalink(Model::Dgm1, &String::from_utf8_lossy(&synthetic)).unwrap();
        assert_eq!(
            names,
            [(EAST, "dgm1_32_400_5579_1_rp_2025".to_string())]
                .into_iter()
                .collect()
        );

        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        fetcher.insert(&metalink_url(Model::Dgm1), synthetic);
        let grid = GridSpec {
            west: 400_000.0,
            north: 5_580_000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        fetcher.insert(
            &tile_url(Model::Dgm1, "dgm1_32_400_5579_1_rp_2025"),
            crate::fetch::test_support::geotiff(&grid, &vec![77.0; 1_000_000]),
        );
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert_eq!(read_out(&tif).2[100 * 200 + 190], 77.0);
        assert_eq!(
            metalink_url(Model::Dgm1),
            "https://geobasis-rlp.de/data/dgm1/current/meta4/dgm1_tif_07.meta4"
        );
        let asked = |url: &str| fetcher.log.borrow().iter().filter(|u| *u == url).count();
        assert_eq!(asked(&metalink_url(Model::Dgm1)), 1);
        // Offline: the kept Metalink, nothing requested.
        let silent = MapFetcher::default();
        let offline = run(&silent, &cache, FetchMode::Offline, Model::Dgm1, &tif).unwrap();
        assert!(offline.missing.is_empty(), "{:?}", offline.missing);
        assert_eq!(silent.calls.get(), 0);
    }

    /// Answers the tile list and the Metalink only through the size-limited `download_to`.
    struct StreamOnly(MapFetcher);

    impl Fetcher for StreamOnly {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            if url.starts_with(TILE_LIST_URL) || url.ends_with(".meta4") {
                panic!("{url} read without a size limit");
            }
            self.0.get(url)
        }

        fn download_to(
            &self,
            url: &str,
            path: &Path,
            max_bytes: u64,
            progress: &mut dyn FnMut(Note),
        ) -> Result<(), FetchError> {
            self.0.download_to(url, path, max_bytes, progress)
        }
    }

    #[test]
    fn the_tile_list_and_the_metalink_are_read_with_a_size_limit() {
        let inner = MapFetcher::default();
        serve_synthetic_rp_heights(&inner, &BBOX);
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        inner.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        inner.insert(
            &metalink_url(Model::Dgm1),
            metalink(&["dgm1_32_400_5579_1_rp_2024.tif"]),
        );
        let fetcher = StreamOnly(inner);
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let found = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &BBOX,
            tif.parent().unwrap(),
            &tif,
        )
        .unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        // Over the limit: stopped, not asked again.
        fetcher
            .0
            .insert_prefix(&tile_list_prefix(), vec![b' '; MAX_LIST_BYTES as usize + 1]);
        fetcher.0.calls.set(0);
        let (_empty_dir, empty) = temp_cache();
        let err = fetch_model(
            &fetcher,
            &empty,
            FetchMode::Online,
            Model::Dgm1,
            &BBOX,
            tif.parent().unwrap(),
            &tif,
        )
        .unwrap_err();
        assert!(err.to_string().contains("larger than"), "{err}");
        assert_eq!(fetcher.0.calls.get(), 1);
    }

    /// Sets the modification time of `path` to `days` ago.
    fn age(path: &Path, days: u64) {
        let then = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86_400);
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(then)
            .unwrap();
    }

    #[test]
    fn a_kept_metalink_is_reused_for_a_month() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        for (model, slug) in [(Model::Dgm1, "dgm1"), (Model::Dom1, "dom1")] {
            let name = format!("{slug}_32_400_5579_1_rp_2025");
            fetcher.insert(&metalink_url(model), metalink(&[&format!("{name}.tif")]));
            let grid = GridSpec {
                west: 400_000.0,
                north: 5_580_000.0,
                pixel_size: 1.0,
                width: 1000,
                height: 1000,
            };
            fetcher.insert(
                &tile_url(model, &name),
                crate::fetch::test_support::geotiff(&grid, &vec![77.0; 1_000_000]),
            );
        }
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let area = [WEST, EAST];
        // Before any run nothing tells that a Metalink is needed.
        assert!(!metalink_to_fetch(&cache, Model::Dgm1, &BBOX, &area));
        let asked = |url: &str| fetcher.log.borrow().iter().filter(|u| *u == url).count();
        let dgm1 = metalink_url(Model::Dgm1);
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(asked(&dgm1), 1);
        // The DOM1 has its own Metalink, still to be fetched; the DGM1 one is fresh.
        assert!(!metalink_to_fetch(&cache, Model::Dgm1, &BBOX, &area));
        assert!(metalink_to_fetch(&cache, Model::Dom1, &BBOX, &area));
        assert!(!metalink_to_fetch(&cache, Model::Dom1, &BBOX, &[WEST]));
        // Online again: the kept one is used.
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert!(
            found.warnings.iter().all(|w| !w.contains("Metalink")),
            "{:?}",
            found.warnings
        );
        assert_eq!(asked(&dgm1), 1);
        let dom1 = run(&fetcher, &cache, FetchMode::Online, Model::Dom1, &tif).unwrap();
        assert!(dom1.missing.is_empty(), "{:?}", dom1.missing);
        assert_eq!(asked(&metalink_url(Model::Dom1)), 1);
        assert!(!metalink_to_fetch(&cache, Model::Dom1, &BBOX, &area));
        // A month later it is fetched again.
        age(
            &cache_dir.path().join("lvermgeo_rp/meta4/dgm1_tif_07.meta4"),
            31,
        );
        assert!(metalink_to_fetch(&cache, Model::Dgm1, &BBOX, &area));
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(asked(&dgm1), 2);
        const { assert!(METALINK_MB > 10.0) };
    }

    #[test]
    fn the_estimate_reads_the_list_of_its_own_area() {
        let (cache_dir, cache) = temp_cache();
        let dir = cache.provider_dir("lvermgeo_rp");
        let lists = cache_dir.path().join("lvermgeo_rp/tile_list");
        std::fs::create_dir_all(&lists).unwrap();
        // The list of this area gives the east tile no year; a newer one of another area does.
        std::fs::write(list_path(&dir, &BBOX), rp_tile_list(&[(EAST, "", "")])).unwrap();
        let other = BoundingBox {
            min_e: 399_000.0,
            ..BBOX
        };
        std::fs::write(
            list_path(&dir, &other),
            rp_tile_list(&[(EAST, "2024", "30.03.24, 06.04.24")]),
        )
        .unwrap();
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &[EAST]).is_empty());
        assert!(metalink_to_fetch(&cache, Model::Dgm1, &BBOX, &[EAST]));
        // The other area has a year: no Metalink.
        assert!(!metalink_to_fetch(&cache, Model::Dgm1, &other, &[EAST]));
    }

    #[test]
    fn a_cached_file_of_another_scan_year_is_not_cached() {
        let (cache_dir, cache) = temp_cache();
        let dir = cache.provider_dir("lvermgeo_rp");
        std::fs::create_dir_all(dir.join("tile_list")).unwrap();
        std::fs::write(
            list_path(&dir, &BBOX),
            rp_tile_list(&[(EAST, "2024", "30.03.24, 06.04.24")]),
        )
        .unwrap();
        let tiles = cache_dir.path().join("lvermgeo_rp/dgm1");
        std::fs::create_dir_all(&tiles).unwrap();
        std::fs::write(tiles.join("dgm1_32_400_5579_1_rp_2022.tif"), b"x").unwrap();
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &[EAST]).is_empty());
        std::fs::write(tiles.join("dgm1_32_400_5579_1_rp_2024.tif"), b"x").unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &BBOX, &[EAST]),
            [EAST].into_iter().collect()
        );
    }

    #[test]
    fn the_estimate_fetches_the_tiles_the_area_list_names() {
        let (cache_dir, cache) = temp_cache();
        // Before a run: every tile of the area that touches the state.
        assert_eq!(tiles_to_fetch(&cache, &BBOX), [WEST, EAST]);
        // At the Hesse border in Mainz, 448/5540 lies wholly in Hesse.
        let border = BoundingBox {
            min_e: 447_900.0,
            min_n: 5_540_400.0,
            max_e: 448_100.0,
            max_n: 5_540_600.0,
        };
        assert_eq!(tiles_to_fetch(&cache, &border), [tile(447, 5540)]);
        // The list kept for an area decides: what it names is downloaded, the Hesse tile too,
        // and an unlisted tile is not (it is reported missing).
        let lists = cache_dir.path().join("lvermgeo_rp/tile_list");
        std::fs::create_dir_all(&lists).unwrap();
        std::fs::write(
            list_path(&cache.provider_dir("lvermgeo_rp"), &border),
            rp_tile_list(&[(tile(448, 5540), "2024", "30.03.24, 06.04.24")]),
        )
        .unwrap();
        assert_eq!(tiles_to_fetch(&cache, &border), [tile(448, 5540)]);
        // A list of another area does not.
        assert_eq!(tiles_to_fetch(&cache, &BBOX), [WEST, EAST]);
    }

    #[test]
    fn cached_tiles_counts_what_needs_no_download() {
        let fetcher = MapFetcher::default();
        serve_synthetic_rp_heights(&fetcher, &BBOX);
        // The west tile is not on the server; the list gives the east one no year.
        let rows = [(WEST, "2024", "30.03.24, 06.04.24"), (EAST, "", "")];
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        let (cache_dir, cache) = temp_cache();
        let tif = cache_dir.path().join("dgm1.tif");
        let area = [WEST, EAST, tile(401, 5579)];
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &area).is_empty());
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_399_5579_1_rp_2024"));
        let _ = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif);
        // A recent 404 needs no download. The tile the list cannot name does: no Metalink is
        // kept, so the next run downloads one, and it names the file.
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &BBOX, &area),
            [WEST].into_iter().collect()
        );
        // A 404 older than the re-check is asked for again.
        let marker = cache_dir
            .path()
            .join("lvermgeo_rp/dgm1/dgm1_32_399_5579_1_rp_2024.missing");
        let month_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 86_400);
        std::fs::File::options()
            .write(true)
            .open(&marker)
            .unwrap()
            .set_modified(month_ago)
            .unwrap();
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &area).is_empty());
        // A fresh kept Metalink that names no file for the east tile: nothing to download.
        let kept = cache_dir.path().join("lvermgeo_rp/meta4/dgm1_tif_07.meta4");
        std::fs::create_dir_all(kept.parent().unwrap()).unwrap();
        std::fs::write(&kept, metalink(&["dgm1_32_401_5579_1_rp_2025.tif"])).unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &BBOX, &area),
            [EAST].into_iter().collect()
        );
        // An old one is downloaded again, and may name it.
        age(&kept, 31);
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &area).is_empty());
        // One that names it makes it a download.
        std::fs::write(&kept, metalink(&["dgm1_32_400_5579_1_rp_2025.tif"])).unwrap();
        assert!(cached_tiles(&cache, Model::Dgm1, &BBOX, &area).is_empty());
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

    #[test]
    fn flight_date_from_get_feature_info() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            test_support::RP_FLIGHT_INFO.as_bytes().to_vec(),
        );
        assert_eq!(flight_date(&fetcher, &BBOX).as_deref(), Some("25.08.2025"));
        let log = fetcher.log.borrow();
        assert_eq!(log.len(), 1, "{log:?}");
        for part in [
            "https://geo4.service24.rlp.de/wms/rp_dop20.fcgi?",
            "LAYERS=rp_dop20_info",
            "QUERY_LAYERS=rp_dop20_info",
            "CRS=EPSG:25832",
            "BBOX=399900,5579400,400100,5579600",
            "WIDTH=100&HEIGHT=100&I=50&J=50",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(log[0].contains(part), "{part} in {}", log[0]);
        }
    }

    #[test]
    fn flight_date_is_none_without_a_date() {
        for answer in [
            // Outside the state.
            "GetFeatureInfo results:\n\nLayer 'rp_dop20_info'\n\nLayer 'metadaten'\n",
            "    erstellung = '2025-08-25'\n",
            "    erstellung = '5.8.2025'\n",
            "    erstellung = ''\n",
            "    erstellung = 'n/a'\n",
            // Only the other dates of a block.
            "    publikation = '2026-01-11 02:15:10.166'\n    aktualisierung = 'n/a'\n",
            "<ServiceExceptionReport/>",
        ] {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(&WMS.url, answer.as_bytes().to_vec());
            assert_eq!(flight_date(&fetcher, &BBOX), None, "{answer}");
        }
        // No answer at all.
        assert_eq!(flight_date(&MapFetcher::default(), &BBOX), None);
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
            let found = fetch_model(
                &fetcher,
                &cache,
                FetchMode::Online,
                model,
                &bbox,
                tif.parent().unwrap(),
                &tif,
            )
            .unwrap();
            let (_, _, data) = read_out(&tif);
            let finite: Vec<f32> = data.iter().copied().filter(|z| z.is_finite()).collect();
            println!("{model:?}: {found:?}; {} cells", finite.len());
            assert!(finite.len() > 39_000);
        }
    }
}
