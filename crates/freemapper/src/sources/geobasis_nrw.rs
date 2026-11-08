//! Geobasis NRW (`www.opengeodata.nrw.de`, dl-de/zero-2-0, "Geobasisdaten © Geobasis NRW
//! (Bezirksregierung Köln)"): the DGM1 and DOM1 1 km GeoTIFF tiles (Float32, LZW, EPSG:25832).
//!
//! A tile's file name carries the year of its laser scan, which differs per tile. The names and
//! dates come from the product's `*_meta.zip` (one CSV row per tile), never from the directory
//! listing, which still holds stale duplicates of older years.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::crs::BoundingBox;
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher, get_with_retries, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::height_model::HeightModel;
use crate::sources::lvermgeo_sh::{is_plain, paste_tile, read_cached, whole_tile};
use crate::sources::region::{Region, tile_centre_in};
use crate::tiles::{TileId, tiles_covering};

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

/// The meta list of `model`: a zip with one CSV.
pub fn meta_url(model: Model) -> String {
    let slug = model.slug();
    format!("{HM}/{slug}_tiff/{slug}_tiff/{slug}_meta.zip")
}

/// The GeoTIFF `name` (from the meta list, without extension) of `model`.
pub fn tile_url(model: Model, name: &str) -> String {
    let slug = model.slug();
    format!("{HM}/{slug}_tiff/{slug}_tiff/{name}.tif")
}

/// One row of the meta list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MetaTile {
    /// `dgm1_32_354_5642_1_nw_2022`; letters, digits and underscores only.
    name: String,
    tile: TileId,
    /// The laser scan date, `YYYY-MM-DD`.
    date: Option<String>,
}

/// What the meta list says.
#[derive(Debug, Default, PartialEq, Eq)]
struct Meta {
    /// The date the list was last updated (`Aktualitaet_Kachelinformationen`).
    release: String,
    tiles: HashMap<TileId, MetaTile>,
    warnings: Vec<String>,
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

/// The CSV of the meta list: a few preamble lines (`Aktualitaet_Kachelinformationen;2026-10-26`),
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
        warnings.push(format!(
            "{unreadable} rows of the {} tile list have no usable tile name; skipped",
            model.label()
        ));
    }
    if undated > 0 {
        warnings.push(format!(
            "{undated} rows of the {} tile list have no readable date",
            model.label()
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

/// The meta list in the zip `bytes`.
fn parse_meta_zip(model: Model, bytes: &[u8]) -> Result<Meta, String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index_raw(i)
                .is_ok_and(|f| f.name().is_ok_and(|n| n.ends_with(".csv")))
        })
        .ok_or("no CSV in the zip")?;
    let mut text = String::new();
    archive
        .by_index(index)
        .map_err(|e| e.to_string())?
        .take(MAX_CSV_BYTES)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    parse_meta_csv(model, &text)
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

/// The meta list of `model`: downloaded and kept per release; offline, or when the download
/// fails, the newest kept one.
fn meta_list(
    fetcher: &dyn Fetcher,
    dir: &Path,
    mode: FetchMode,
    model: Model,
    warnings: &mut Vec<String>,
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
    let answer = get_with_retries(fetcher, &url, |bytes| {
        let meta = parse_meta_zip(model, bytes).map_err(|message| FetchError::Http {
            url: url.clone(),
            message: format!("not the tile list of the {}: {message}", model.label()),
        })?;
        Ok((meta, bytes.to_vec()))
    });
    match answer {
        Ok((meta, bytes)) => {
            let path = meta_path(dir, model, &meta.release);
            write_atomically(&path, &bytes)?;
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
            let Some(meta) = kept_meta(dir, model) else {
                return Err(e);
            };
            warnings.push(format!(
                "the tile list of the {} could not be read ({e}); used the one kept from an \
                 earlier run",
                model.label()
            ));
            Ok(meta)
        }
    }
}

/// Where the tile `name` is cached; the name carries the year, so a new scan is a new file.
fn cache_path(dir: &Path, model: Model, name: &str) -> PathBuf {
    dir.join(model.slug()).join(format!("{name}.tif"))
}

/// `32_354_5642`.
fn short_name(tile: TileId) -> String {
    format!("32_{:03}_{:04}", tile.east_km, tile.north_km)
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file.
///
/// The tile names and dates come from the product's meta list (kept per release for offline
/// runs). Tiles are cached under their name. A tile the list names but the server lacks (404),
/// and a tile inside North Rhine-Westphalia that the list does not name, are returned as
/// missing; the date is the centre tile's. No tile at all is an error.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let dir = cache.geobasis_nrw_dir();
    let mut warnings = Vec::new();
    let meta = meta_list(fetcher, &dir, mode, model, &mut warnings)?;
    warnings.extend(meta.warnings.iter().cloned());
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let (mut found, mut missing, mut pending) = (0, Vec::new(), Vec::new());
    let mut paste = |raster: &mut Raster<f32>, entry: &MetaTile, part: &Raster<f32>| {
        found += 1;
        let empty = paste_tile(raster, part).map_err(|message| FetchError::Http {
            url: tile_url(model, &entry.name),
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
    for tile in tiles_covering(bbox) {
        match meta.tiles.get(&tile) {
            None if tile_centre_in(tile, Region::NordrheinWestfalen) => missing.push(tile),
            None => {}
            Some(entry) => match read_cached(&cache_path(&dir, model, &entry.name), tile) {
                Some(part) => paste(&mut raster, entry, &part)?,
                None => pending.push(entry),
            },
        }
    }
    if let (FetchMode::Offline, Some(entry)) = (mode, pending.first()) {
        return Err(FetchError::Http {
            url: tile_url(model, &entry.name),
            message: format!(
                "offline and not cached: {} tile {}; run `freemapper fetch` online",
                model.label(),
                entry.name
            ),
        });
    }
    for entry in pending {
        let url = tile_url(model, &entry.name);
        let downloaded = get_with_retries(fetcher, &url, |bytes| {
            let part = decode_float_tiff(bytes)
                .and_then(|r| whole_tile(r, entry.tile, &[1.0]))
                .map_err(|message| FetchError::Http {
                    url: url.clone(),
                    message: format!("{} tile {}: {message}", model.label(), entry.name),
                })?;
            Ok((part, bytes.to_vec()))
        });
        match downloaded {
            Ok((part, bytes)) => {
                write_atomically(&cache_path(&dir, model, &entry.name), &bytes)?;
                paste(&mut raster, entry, &part)?;
            }
            // Named by the list, not on the server.
            Err(FetchError::NotFound(_)) => missing.push(entry.tile),
            Err(e) => return Err(e),
        }
    }
    if found == 0 {
        return Err(FetchError::Http {
            url: meta_url(model),
            message: format!("no tile of the {} covers the map area", model.label()),
        });
    }
    write_float_raster(&raster, out, HM)?;
    let centre = TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    };
    let date = match meta.tiles.get(&centre) {
        Some(entry) if entry.date.is_none() => {
            warnings.push(format!(
                "the tile list gives no date for the {} tile {}",
                model.label(),
                short_name(centre)
            ));
            None
        }
        entry => entry.and_then(|e| e.date.clone()),
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
    use crate::fetch::test_support::{MapFetcher, nrw_meta_zip, serve_synthetic_nrw};

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

    fn run(
        fetcher: &MapFetcher,
        cache: &Cache,
        mode: FetchMode,
        model: Model,
        out: &Path,
    ) -> Result<HeightModel, FetchError> {
        fetch_model(fetcher, cache, mode, model, &BBOX, out)
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
        let zip = nrw_meta_zip("dgm1_nw.csv", &rows);
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
        fetcher.insert(&meta_url(Model::Dgm1), nrw_meta_zip("dgm1_nw.csv", &rows));
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
            let found =
                fetch_model(&fetcher, &cache, FetchMode::Online, model, &bbox, &tif).unwrap();
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
