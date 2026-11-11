//! Geobasis NRW (`www.opengeodata.nrw.de`, dl-de/zero-2-0, "Geobasisdaten © Geobasis NRW
//! (Bezirksregierung Köln)"): the DGM1 and DOM1 1 km GeoTIFF tiles (Float32, LZW, EPSG:25832).
//!
//! A tile's file name carries the year of its laser scan, which differs per tile. The names and
//! dates come from the product's `*_meta.zip` (one CSV row per tile), never from the directory
//! listing, which still holds stale duplicates of older years.

use crate::note::{Arg, Note, note};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher, get_with_retries, write_cache_file};
use crate::sources::height_model::{
    AreaTiles, HeightModel, TileStore, centre_date, centre_tile, mosaic_tiles, short_name,
};
use crate::sources::lvermgeo_sh::is_plain;
use crate::sources::region::{Region, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS (the tiles are JPEG 2000 only). MaxWidth/MaxHeight is 5000; a larger request
/// is an HTTP 200 XML exception.
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://www.wms.nrw.de/geobasis/wms_nw_dop"),
    layer: Cow::Borrowed("nw_dop_rgb"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};
/// The queryable layer with the flight date per tile.
const WMS_INFO_LAYER: &str = "nw_dop_utm_info";

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the metadata layer; `None` outside the coverage or when the answer has none.
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

/// `Bildflugdatum = 'DD.MM.YYYY'` of a GetFeatureInfo text answer.
fn parse_flight_date(text: &str) -> Option<String> {
    let rest = text.split("Bildflugdatum").nth(1)?;
    let value = rest.split('\'').nth(1)?;
    let parts: Vec<&str> = value.split('.').collect();
    let [d, m, y] = parts[..] else { return None };
    let digits = |s: &str, n| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    (digits(d, 2) && digits(m, 2) && digits(y, 4)).then(|| format!("{d}.{m}.{y}"))
}

/// The tiles among `tiles` that need no download: the newest kept tile list names them and their
/// file exists (files are written whole, atomically), or the server answered 404 for them.
pub fn cached_tiles(cache: &Cache, model: Model, tiles: &[TileId]) -> HashSet<TileId> {
    let dir = cache.provider_dir("geobasis_nrw");
    let Some(meta) = kept_meta(&dir, model) else {
        return HashSet::new();
    };
    tiles
        .iter()
        .filter(|tile| {
            meta.tiles.get(tile).is_some_and(|entry| {
                cache_path(&dir, model, &entry.name).exists()
                    || marker_path(&dir, model, &entry.name).exists()
            })
        })
        .copied()
        .collect()
}

/// The height model files; the product folder appears twice in the path.
const HM: &str = "https://www.opengeodata.nrw.de/produkte/geobasis/hm";

/// The height models of Geobasis NRW.
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
            Self::Dgm1 => "terrain model (Geobasis NRW DGM1)",
            Self::Dom1 => "surface model (Geobasis NRW DOM1)",
        }
    }
}

/// The tile list of `model`: a zip with one CSV.
pub fn meta_url(model: Model) -> String {
    let slug = model.slug();
    format!("{HM}/{slug}_tiff/{slug}_tiff/{slug}_meta.zip")
}

/// The GeoTIFF `name` (from the tile list, without extension) of `model`.
pub fn tile_url(model: Model, name: &str) -> String {
    let slug = model.slug();
    format!("{HM}/{slug}_tiff/{slug}_tiff/{name}.tif")
}

/// One row of the tile list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MetaTile {
    /// `dgm1_32_354_5642_1_nw_2022`; letters, digits and underscores only.
    name: String,
    tile: TileId,
    /// The laser scan date, `YYYY-MM-DD`.
    date: Option<String>,
}

/// What the tile list says.
#[derive(Debug, Default, PartialEq, Eq)]
struct Meta {
    /// The date the list was last updated (`Aktualitaet_Kachelinformationen`).
    release: String,
    tiles: HashMap<TileId, MetaTile>,
    warnings: Vec<Note>,
}

/// The tile a name like `dgm1_32_354_5642_1_nw_2022` stands for.
fn tile_of_name(model: Model, name: &str) -> Option<TileId> {
    if !is_plain(name) {
        return None;
    }
    let parts: Vec<&str> = name.split('_').collect();
    let [slug, "32", east, north, ..] = parts[..] else {
        return None;
    };
    (slug == model.slug()).then_some(())?;
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

fn iso_date(value: &str) -> Option<String> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .ok()
        .map(|d| d.to_string())
}

/// The CSV of the tile list: a few preamble lines (`Aktualitaet_Kachelinformationen;2026-10-26`),
/// the header `Kachelname;Aktualitaet;…`, then a row per tile (`;`-separated, CRLF).
fn parse_meta_csv(model: Model, text: &str) -> Result<Meta, String> {
    let mut lines = text.lines().map(str::trim);
    let mut release = None;
    let header = loop {
        let line = lines
            .next()
            .ok_or("no `Kachelname` header line in the list")?;
        if let Some(value) = line.strip_prefix("Aktualitaet_Kachelinformationen;") {
            release = iso_date(value);
        }
        if line.starts_with("Kachelname;") {
            break line;
        }
    };
    let release = release.ok_or("the list does not say when it was updated")?;
    let columns: Vec<&str> = header.split(';').collect();
    let column = |name: &str| columns.iter().position(|c| *c == name);
    let (name_at, date_at) = (
        column("Kachelname").ok_or("no `Kachelname` column")?,
        column("Aktualitaet").ok_or("no `Aktualitaet` column")?,
    );
    let (mut unreadable, mut undated) = (0, 0);
    let mut tiles: HashMap<TileId, MetaTile> = HashMap::new();
    for row in lines.filter(|l| !l.is_empty()) {
        let cells: Vec<&str> = row.split(';').collect();
        let name = cells.get(name_at).copied().unwrap_or_default();
        let Some(tile) = tile_of_name(model, name) else {
            unreadable += 1;
            continue;
        };
        let date = cells.get(date_at).and_then(|d| iso_date(d));
        undated += u32::from(date.is_none());
        let entry = MetaTile {
            name: name.to_string(),
            tile,
            date,
        };
        // A tile twice: the newer scan wins.
        match tiles.get(&tile) {
            Some(old) if (&old.date, &old.name) >= (&entry.date, &entry.name) => {}
            _ => {
                tiles.insert(tile, entry);
            }
        }
    }
    let mut warnings = Vec::new();
    if unreadable > 0 {
        warnings.push(note!(
            "warning-nrw-list-rows-unusable",
            count = unreadable,
            model = crate::note::model(model.label())
        ));
    }
    if undated > 0 {
        warnings.push(note!(
            "warning-nrw-list-rows-undated",
            count = undated,
            model = crate::note::model(model.label())
        ));
    }
    Ok(Meta {
        release,
        tiles,
        warnings,
    })
}

/// The largest CSV read from the meta zip (the DGM1 one is 4 MB).
const MAX_CSV_BYTES: u64 = 64 * 1024 * 1024;
/// The largest meta zip accepted (110 KB expected).
const MAX_ZIP_BYTES: usize = 8 * 1024 * 1024;

/// The tile list in the zip `bytes`.
fn parse_meta_zip(model: Model, bytes: &[u8]) -> Result<Meta, String> {
    parse_meta_zip_limited(model, bytes, MAX_CSV_BYTES)
}

/// [`parse_meta_zip`] with a CSV of at most `limit` bytes.
fn parse_meta_zip_limited(model: Model, bytes: &[u8], limit: u64) -> Result<Meta, String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index_raw(i)
                .is_ok_and(|f| f.name().is_ok_and(|n| n.ends_with(".csv")))
        })
        .ok_or("no CSV in the zip")?;
    let mut bytes = Vec::new();
    archive
        .by_index(index)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("the list is larger than {limit} bytes"));
    }
    // Only the ASCII columns matter. A stray byte (a Latin-1 umlaut in the preamble) becomes
    // U+FFFD, which no tile name or date accepts, so it costs that row and nothing else.
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    parse_meta_csv(model, text)
}

/// Where the zip of the list updated on `release` is kept.
fn meta_path(dir: &Path, model: Model, release: &str) -> PathBuf {
    dir.join("meta")
        .join(format!("{}_{release}.zip", model.slug()))
}

/// The newest list kept from an earlier run.
fn kept_meta(dir: &Path, model: Model) -> Option<Meta> {
    let prefix = format!("{}_", model.slug());
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("meta"))
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .map(|e| e.path())
        .collect();
    // The names end in the ISO release date.
    files.sort();
    files
        .into_iter()
        .rev()
        .find_map(|path| parse_meta_zip(model, &std::fs::read(path).ok()?).ok())
}

/// The tile list of `model`: downloaded and kept per release; offline, or when the download
/// fails, the newest kept one.
fn meta_list(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    model: Model,
    warnings: &mut Vec<Note>,
) -> Result<Meta, FetchError> {
    let url = meta_url(model);
    if mode == FetchMode::Offline {
        return kept_meta(dir, model).ok_or_else(|| FetchError::Http {
            url,
            message: format!(
                "offline and not cached: the tile list of the {}; run `freemapper fetch` online",
                model.label()
            ),
        });
    }
    let kept = kept_meta(dir, model);
    let answer = get_with_retries(fetcher, &url, |bytes| {
        if bytes.len() > MAX_ZIP_BYTES {
            return Err(FetchError::TooLarge {
                url: url.clone(),
                limit_mb: (MAX_ZIP_BYTES / (1024 * 1024)) as u64,
            });
        }
        let meta = parse_meta_zip(model, bytes).map_err(|message| FetchError::Http {
            url: url.clone(),
            message: format!("not the tile list of the {}: {message}", model.label()),
        })?;
        Ok((meta, bytes.to_vec()))
    });
    match answer {
        Ok((meta, _)) if kept.as_ref().is_some_and(|k| k.release > meta.release) => {
            // A mirror or cache behind the origin: keep the newer list we have.
            let kept = kept.expect("checked above");
            warnings.push(note!(
                "warning-nrw-list-older",
                model = crate::note::model(model.label()),
                server = meta.release.to_string(),
                kept = kept.release.to_string()
            ));
            Ok(kept)
        }
        Ok((meta, bytes)) => {
            let path = meta_path(dir, model, &meta.release);
            if !path.exists() {
                // A new release: the tiles the old one named but the server lacked may be there.
                clear_markers(dir, model);
            }
            write_cache_file(&path, &bytes)?;
            // The older lists are not needed any more.
            for old in std::fs::read_dir(dir.join("meta"))
                .into_iter()
                .flatten()
                .flatten()
            {
                let name = old.file_name();
                let name = name.to_string_lossy();
                if old.path() != path && name.starts_with(&format!("{}_", model.slug())) {
                    let _ = std::fs::remove_file(old.path());
                }
            }
            Ok(meta)
        }
        Err(e) => {
            let Some(meta) = kept else {
                return Err(e);
            };
            warnings.push(note!(
                "warning-list-unreadable-kept",
                model = crate::note::model(model.label()),
                error = Arg::error(e)
            ));
            Ok(meta)
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

/// Removes the 404 notes of `model`.
fn clear_markers(dir: &Path, model: Model) {
    for entry in std::fs::read_dir(dir.join(model.slug()))
        .into_iter()
        .flatten()
        .flatten()
    {
        if entry.file_name().to_string_lossy().ends_with(".missing") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file.
///
/// The tile names and dates come from the product's tile list (kept per release for offline
/// runs). Tiles are cached under their name. A tile the list names but the server lacks (404),
/// and a tile inside North Rhine-Westphalia that the list does not name, are returned as
/// missing; the date is the centre tile's. No tile at all is an error.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let dir = cache.provider_dir("geobasis_nrw");
    let mut warnings = Vec::new();
    let meta = meta_list(fetcher, &dir, mode, model, &mut warnings)?;
    warnings.extend(meta.warnings.iter().cloned());
    let mut area = AreaTiles::default();
    for tile in tiles_covering(bbox) {
        match meta.tiles.get(&tile) {
            Some(entry) => area.named.push((tile, entry.name.clone())),
            None if tile_centre_in(tile, Region::NordrheinWestfalen) => area
                .missing
                .push((tile, "not in the tile list".to_string())),
            None => {}
        }
    }
    let list_url = meta_url(model);
    let store = TileStore {
        label: model.label(),
        list_url: &list_url,
        source: HM,
        url: &|name| tile_url(model, name),
        cache_path: &|name| cache_path(&dir, model, name),
        marker_path: &|name| marker_path(&dir, model, name),
        // A new release of the list clears the notes.
        known_missing: &|path| path.exists(),
        precheck: &|_| Ok(()),
        explain_empty: false,
    };
    let missing = mosaic_tiles(fetcher, mode, bbox, &store, area, root, out, &mut warnings)?;
    let centre = centre_tile(bbox);
    let listed = meta.tiles.get(&centre).map(|entry| {
        entry.date.clone().ok_or_else(|| {
            note!(
                "warning-nrw-list-no-date",
                model = crate::note::model(model.label()),
                tile = short_name(centre)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, nrw_meta_zip, serve_synthetic_nrw};
    use crate::raster::GridSpec;
    use crate::sources::float_tiff::decode_float_tiff;

    /// Two tiles side by side in Köln (centre at the tile border 355 km).
    const BBOX: BoundingBox = BoundingBox {
        min_e: 354_900.0,
        min_n: 5_642_400.0,
        max_e: 355_100.0,
        max_n: 5_642_600.0,
    };

    const WEST: TileId = TileId {
        east_km: 354,
        north_km: 5642,
    };
    const EAST: TileId = TileId {
        east_km: 355,
        north_km: 5642,
    };

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

    #[test]
    fn flight_date_from_get_feature_info() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            crate::fetch::test_support::NRW_FLIGHT_INFO
                .as_bytes()
                .to_vec(),
        );
        assert_eq!(flight_date(&fetcher, &BBOX).as_deref(), Some("11.05.2025"));
        let url = fetcher.log.borrow()[0].clone();
        for part in [
            "LAYERS=nw_dop_utm_info",
            "QUERY_LAYERS=nw_dop_utm_info",
            "CRS=EPSG:25832",
            "BBOX=354900,5642400,355100,5642600",
            "WIDTH=100&HEIGHT=100&I=50&J=50",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(url.contains(part), "{part} in {url}");
        }
    }

    #[test]
    fn flight_date_is_none_without_a_date() {
        for answer in [
            "GetFeatureInfo results:\n\n  Search returned no results.\n",
            "Bildflugdatum = '2025-05-11'",
            "Bildflugdatum = '1.5.2025'",
            "Bildflugdatum = ''",
            "<ServiceExceptionReport/>",
        ] {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(&WMS.url, answer.as_bytes().to_vec());
            assert_eq!(flight_date(&fetcher, &BBOX), None, "{answer}");
        }
        // No answer at all.
        assert_eq!(flight_date(&MapFetcher::default(), &BBOX), None);
    }

    #[test]
    fn cached_tiles_are_those_of_the_kept_list_that_exist() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let area = [WEST, EAST];
        assert!(cached_tiles(&cache, Model::Dgm1, &area).is_empty());
        serve_synthetic_nrw(&fetcher, &BBOX);
        run(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &area),
            [WEST, EAST].into_iter().collect()
        );
        // Only the asked tiles count.
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &[EAST]),
            [EAST].into_iter().collect()
        );
        // The DOM1 was not read.
        assert!(cached_tiles(&cache, Model::Dom1, &area).is_empty());
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

    #[test]
    fn meta_csv_gives_names_and_dates() {
        let csv = "Kachelinformationen des DGM1 fuer die Datenabgabe\r\n\
            Land;Nordrhein-Westfalen\r\n\
            Aktualitaet_Kachelinformationen;2026-10-26\r\n\
            Version_Standard;3.3\r\n\
            Kachelname;Aktualitaet;Erfassungsmethode\r\n\
            dgm1_32_354_5642_1_nw_2022;2022-02-12;5020\r\n\
            dgm1_32_355_5642_1_nw_2023;;5020\r\n\
            ../evil;2022-02-12;5020\r\n\
            dom1_32_356_5642_1_nw_2022;2022-02-12;5020\r\n";
        let meta = parse_meta_csv(Model::Dgm1, csv).unwrap();
        assert_eq!(meta.release, "2026-10-26");
        assert_eq!(meta.tiles.len(), 2);
        let west = &meta.tiles[&WEST];
        assert_eq!(west.name, "dgm1_32_354_5642_1_nw_2022");
        assert_eq!(west.date.as_deref(), Some("2022-02-12"));
        assert_eq!(meta.tiles[&EAST].date, None);
        assert_eq!(meta.warnings.len(), 2, "{:?}", meta.warnings);
        // Through the zip, too.
        let rows = [("dgm1_32_354_5642_1_nw_2022".to_string(), "2022-02-12")];
        let zip = nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &rows);
        assert_eq!(parse_meta_zip(Model::Dgm1, &zip).unwrap().tiles.len(), 1);
        assert!(parse_meta_zip(Model::Dgm1, b"<html>").is_err());
        assert!(parse_meta_csv(Model::Dgm1, "Kachelname;Aktualitaet\r\n").is_err());
    }

    #[test]
    fn tiles_mosaic_into_the_area() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let out = tempfile::tempdir().unwrap();
        for (model, lift) in [(Model::Dgm1, 0.0), (Model::Dom1, 12.0)] {
            let tif = out.path().join(format!("{}.tif", model.slug()));
            let found = run(&fetcher, &cache, FetchMode::Online, model, &tif).unwrap();
            assert!(found.missing.is_empty(), "{:?}", found.missing);
            assert_eq!(found.date.as_deref(), Some("2022-02-12"));
            assert!(found.warnings.is_empty(), "{:?}", found.warnings);
            let (width, height, data) = read_out(&tif);
            assert_eq!((width, height), (200, 200), "{model:?}");
            let at = |x: usize, y: usize| data[y * 200 + x];
            // The terrain rises 0.01 m per metre east inside each tile (from 20 m): column 910 of the west tile, 90 of the east one.
            let west = at(10, 100);
            let east = at(190, 100);
            assert!((west - (29.1 + lift)).abs() < 1e-3, "{west}");
            assert!((east - (20.9 + lift)).abs() < 1e-3, "{east}");
        }
        assert!(
            cache_dir
                .path()
                .join("geobasis_nrw/dgm1/dgm1_32_354_5642_1_nw_2022.tif")
                .is_file()
        );
    }

    #[test]
    fn listed_but_missing_tile_is_reported() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_355_5642_1_nw_2022"));
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let out = tempfile::tempdir().unwrap();
        let tif = out.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(found.missing, vec![EAST]);
        let (_, _, data) = read_out(&tif);
        assert!(data[100 * 200 + 10].is_finite());
        assert!(data[100 * 200 + 190].is_nan());
        // Nothing of the area at all is an error.
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_354_5642_1_nw_2022"));
        let cache_dir = tempfile::tempdir().unwrap();
        let err = run(
            &fetcher,
            &Cache::new(cache_dir.path().to_path_buf()),
            FetchMode::Online,
            Model::Dgm1,
            &out.path().join("none.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no tile"), "{err}");
    }

    #[test]
    fn a_tile_that_answered_404_stays_missing_offline_and_online_until_a_new_release() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        let east_url = tile_url(Model::Dgm1, "dgm1_32_355_5642_1_nw_2022");
        let east_file = fetcher.responses.borrow_mut().remove(&east_url).unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let tif = cache_dir.path().join("dgm1.tif");
        let online = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(online.missing, vec![EAST]);
        // Offline: the same answer, no error, no request.
        let silent = MapFetcher::default();
        let offline = run(&silent, &cache, FetchMode::Offline, Model::Dgm1, &tif).unwrap();
        assert_eq!(offline.missing, vec![EAST]);
        assert_eq!(silent.calls.get(), 0);
        // Online again with the same list: the tile is not asked for again.
        fetcher.log.borrow_mut().clear();
        let again = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(again.missing, vec![EAST]);
        assert!(
            !fetcher.log.borrow().contains(&east_url),
            "{:?}",
            fetcher.log.borrow()
        );
        // A new release of the list: asked for again, and found.
        let rows = [
            ("dgm1_32_354_5642_1_nw_2022".to_string(), "2022-02-12"),
            ("dgm1_32_355_5642_1_nw_2022".to_string(), "2022-02-12"),
        ];
        fetcher.insert(
            &meta_url(Model::Dgm1),
            nrw_meta_zip("dgm1_nw.csv", "2026-11-20", &rows),
        );
        fetcher.insert(&east_url, east_file);
        let fixed = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert!(fixed.missing.is_empty(), "{:?}", fixed.missing);
    }

    #[test]
    fn a_centre_tile_that_is_missing_gives_no_date() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        // The centre of the area lies in the east tile.
        fetcher
            .responses
            .borrow_mut()
            .remove(&tile_url(Model::Dgm1, "dgm1_32_355_5642_1_nw_2022"));
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let tif = cache_dir.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(found.missing, vec![EAST]);
        assert_eq!(found.date, None);
        assert!(
            found.warnings.iter().any(|w| w.contains("32_355_5642")),
            "{:?}",
            found.warnings
        );
    }

    #[test]
    fn nrw_dgm1_decodes() {
        let bytes = include_bytes!("../../../../tests/fixtures/nordrhein-westfalen-dgm1.tif");
        let raster = decode_float_tiff(bytes).unwrap();
        let g = raster.grid;
        assert_eq!((g.width, g.height, g.pixel_size), (1000, 100, 1.0));
        assert_eq!((g.west, g.north), (354_000.0, 5_643_000.0));
        let heights: Vec<f32> = raster
            .data
            .iter()
            .copied()
            .filter(|z| z.is_finite())
            .collect();
        assert!(heights.len() > 99_000, "{}", heights.len());
        // Köln, the Rhine plain.
        assert!(heights.iter().all(|z| (20.0..120.0).contains(z)));
    }

    #[test]
    fn centre_tile_gives_the_date_and_offline_reads_the_kept_files() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        // Other dates per tile: the west tile is older; the centre of the area is the east one.
        let rows = [
            ("dgm1_32_354_5642_1_nw_2022".to_string(), "2021-01-02"),
            ("dgm1_32_355_5642_1_nw_2022".to_string(), "2022-03-04"),
        ];
        fetcher.insert(
            &meta_url(Model::Dgm1),
            nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &rows),
        );
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let out = tempfile::tempdir().unwrap();
        let tif = out.path().join("dgm1.tif");
        let online = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(online.date.as_deref(), Some("2022-03-04"));
        // Offline: nothing is requested.
        let silent = MapFetcher::default();
        let offline = run(&silent, &cache, FetchMode::Offline, Model::Dgm1, &tif).unwrap();
        assert_eq!(offline.date, online.date);
        assert_eq!(silent.calls.get(), 0);
        // Online with a dead server: the kept list and tiles, with a warning.
        let dead = MapFetcher::default();
        let kept = run(&dead, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(kept.date, online.date);
        assert_eq!(kept.warnings.len(), 1, "{:?}", kept.warnings);
        // Never fetched: offline says so.
        let empty = tempfile::tempdir().unwrap();
        let err = run(
            &silent,
            &Cache::new(empty.path().to_path_buf()),
            FetchMode::Offline,
            Model::Dgm1,
            &tif,
        )
        .unwrap_err();
        assert!(err.to_string().contains("offline and not cached"), "{err}");
    }

    fn zip_of(csv: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("dgm1_nw.csv", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(csv).unwrap();
        zip.finish().unwrap().into_inner()
    }

    const HEAD: &str = "Aktualitaet_Kachelinformationen;2026-10-26\r\nKachelname;Aktualitaet\r\n";

    #[test]
    fn oversized_list_is_an_error_not_a_short_list() {
        let rows: Vec<(String, &str)> = (0..50)
            .map(|n| (format!("dgm1_32_{}_5642_1_nw_2022", 300 + n), "2022-02-12"))
            .collect();
        let zip = nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &rows);
        assert_eq!(parse_meta_zip(Model::Dgm1, &zip).unwrap().tiles.len(), 50);
        let err = parse_meta_zip_limited(Model::Dgm1, &zip, 500).unwrap_err();
        assert!(err.contains("larger than"), "{err}");
        // The download is capped too.
        let fetcher = MapFetcher::default();
        fetcher.insert(&meta_url(Model::Dgm1), vec![0; MAX_ZIP_BYTES + 1]);
        let cache_dir = tempfile::tempdir().unwrap();
        let err = run(
            &fetcher,
            &Cache::new(cache_dir.path().to_path_buf()),
            FetchMode::Online,
            Model::Dgm1,
            &cache_dir.path().join("x.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("larger than"), "{err}");
        // An oversized answer is not downloaded again.
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn bom_and_stray_bytes_do_not_reject_the_list() {
        let mut csv = b"\xEF\xBB\xBFLand;K\xF6ln\r\n".to_vec();
        csv.extend_from_slice(HEAD.as_bytes());
        csv.extend_from_slice(b"dgm1_32_354_5642_1_nw_2022;2022-02-12\r\n");
        csv.extend_from_slice(b"dgm1_32_355_56\xE442_1_nw_2022;2022-02-12\r\n");
        let meta = parse_meta_zip(Model::Dgm1, &zip_of(&csv)).unwrap();
        assert_eq!(meta.tiles.len(), 1);
        assert_eq!(meta.warnings.len(), 1, "{:?}", meta.warnings);
        // The BOM in front of the header line itself.
        let mut csv = b"\xEF\xBB\xBF".to_vec();
        csv.extend_from_slice(HEAD.as_bytes());
        csv.extend_from_slice(b"dgm1_32_354_5642_1_nw_2022;2022-02-12\r\n");
        assert_eq!(
            parse_meta_zip(Model::Dgm1, &zip_of(&csv))
                .unwrap()
                .tiles
                .len(),
            1
        );
    }

    #[test]
    fn duplicate_row_resolves_to_the_newer_date() {
        for rows in [
            "dgm1_32_354_5642_1_nw_2022;2022-02-12\r\ndgm1_32_354_5642_1_nw_2023;2023-05-06\r\n",
            "dgm1_32_354_5642_1_nw_2023;2023-05-06\r\ndgm1_32_354_5642_1_nw_2022;2022-02-12\r\n",
        ] {
            let meta = parse_meta_csv(Model::Dgm1, &format!("{HEAD}{rows}")).unwrap();
            assert_eq!(meta.tiles.len(), 1);
            assert_eq!(meta.tiles[&WEST].name, "dgm1_32_354_5642_1_nw_2023");
            assert_eq!(meta.tiles[&WEST].date.as_deref(), Some("2023-05-06"));
        }
    }

    #[test]
    fn unlisted_tile_in_nrw_is_missing_and_an_unlisted_centre_warns() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        let rows = [("dgm1_32_354_5642_1_nw_2022".to_string(), "2022-02-12")];
        fetcher.insert(
            &meta_url(Model::Dgm1),
            nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &rows),
        );
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let tif = cache_dir.path().join("dgm1.tif");
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert_eq!(found.missing, vec![EAST]);
        // The centre of the area lies in the unlisted east tile.
        assert_eq!(found.date, None);
        assert!(
            found.warnings.iter().any(|w| w.contains("32_355_5642")),
            "{:?}",
            found.warnings
        );
    }

    #[test]
    fn a_new_vintage_is_a_new_cache_file() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let tif = cache_dir.path().join("dgm1.tif");
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        let tiles = cache_dir.path().join("geobasis_nrw/dgm1");
        let old = tiles.join("dgm1_32_354_5642_1_nw_2022.tif");
        let old_bytes = std::fs::read(&old).unwrap();
        // The west tile is flown again in 2024.
        let rows = [
            ("dgm1_32_354_5642_1_nw_2024".to_string(), "2024-03-01"),
            ("dgm1_32_355_5642_1_nw_2022".to_string(), "2022-02-12"),
        ];
        fetcher.insert(
            &meta_url(Model::Dgm1),
            nrw_meta_zip("dgm1_nw.csv", "2026-11-20", &rows),
        );
        let grid = GridSpec {
            west: 354_000.0,
            north: 5_643_000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        fetcher.insert(
            &tile_url(Model::Dgm1, "dgm1_32_354_5642_1_nw_2024"),
            crate::fetch::test_support::geotiff(&grid, &vec![77.0; 1_000_000]),
        );
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        assert!(tiles.join("dgm1_32_354_5642_1_nw_2024.tif").is_file());
        assert_eq!(std::fs::read(&old).unwrap(), old_bytes);
        assert_eq!(read_out(&tif).2[100 * 200 + 10], 77.0);
    }

    #[test]
    fn an_older_list_from_the_server_does_not_replace_a_newer_kept_one() {
        let fetcher = MapFetcher::default();
        serve_synthetic_nrw(&fetcher, &BBOX);
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let tif = cache_dir.path().join("dgm1.tif");
        run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        let rows = [("dgm1_32_354_5642_1_nw_2022".to_string(), "2022-02-12")];
        fetcher.insert(
            &meta_url(Model::Dgm1),
            nrw_meta_zip("dgm1_nw.csv", "2026-09-01", &rows),
        );
        let found = run(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &tif).unwrap();
        let meta = cache_dir.path().join("geobasis_nrw/meta");
        assert!(meta.join("dgm1_2026-10-26.zip").is_file());
        assert!(!meta.join("dgm1_2026-09-01.zip").exists());
        // The kept, newer list still names the east tile.
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert!(
            found.warnings.iter().any(|w| w.contains("2026-09-01")),
            "{:?}",
            found.warnings
        );
    }

    /// Needs the network: the Köln school, tile 354/5642, 200 m box.
    #[test]
    #[ignore]
    fn live_koeln_school() {
        let bbox = BoundingBox {
            min_e: 354_400.0,
            min_n: 5_642_400.0,
            max_e: 354_600.0,
            max_n: 5_642_600.0,
        };
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
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
            let (lo, hi) = finite
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &z| (a.min(z), b.max(z)));
            println!(
                "{model:?}: {found:?}; {} cells, heights {lo}..{hi}",
                finite.len()
            );
            assert!(finite.len() > 39_000);
        }
    }
}
