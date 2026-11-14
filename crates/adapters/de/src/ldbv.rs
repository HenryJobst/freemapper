//! Bayerische Vermessungsverwaltung (LDBV, `geodaten.bayern.de`, CC BY 4.0): the orthophoto
//! from the WMS, the DGM1 from 1 km GeoTIFF tiles, and the surface model from the DOM20 tiles
//! (20 cm, computed from the aerial pictures), which are cached reduced to 1 m.
//!
//! The tiles are named by their south-west corner in km of EPSG:25832, in the whole state, so
//! their addresses follow from the map area. A cached tile carries the day the server last
//! changed it ([`crate::ldbv_files`]).

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher, one_line, write_cache_file};
use crate::ldbv_files::{Checks, Current, Published, cached, current, drop_older, missing_note};
use crate::note::Note;
use crate::sources::float_tiff::{decode_float_tiff, encode_float_geotiff, gdal_metadata_item};
use crate::sources::height_model::{
    AreaTiles, HeightModel, Prepared, TileStore, as_delivered, centre_tile, coarsen_max,
    mosaic_tiles, whole_tile,
};
use crate::sources::region::{BAYERN, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS (DOP20). MaxWidth and MaxHeight of the service are 6500.
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://geoservices.bayern.de/od/wms/dop/v1/dop20"),
    layer: Cow::Borrowed("by_dop20c"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 6000,
};
/// The queryable layer with the flights; its field `ua` is the day of the flight.
const WMS_INFO_LAYER: &str = "by_dop20_info";
/// The field with the flight date, with its equals sign: the two letters alone could stand
/// anywhere in an answer.
const WMS_DATE_FIELD: &str = "ua =";
/// What every GetFeatureInfo request of the orthophoto WMS starts with.
#[cfg(any(test, feature = "testing"))]
const FLIGHT_INFO_PREFIX: &str = "https://geoservices.bayern.de/od/wms/dop/v1/dop20?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo";

/// The file server of the tiles.
pub const FILES: &str = "https://download1.bayernwolke.de/a";

/// The provider id of the adapter.
pub const PROVIDER: &str = "ldbv";
/// The cache folder of the provider.
const CACHE_DIR: &str = "ldbv";
/// The state as the missing-tiles warning names it.
const STATE: &str = "Bavaria";

/// How many cells of a DOM20 tile make one cell of 1 m, along a side.
const DOM20_FACTOR: u32 = 5;

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the layer of the flights; `None` outside the coverage or when the answer
/// has none.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    freemapper_sources::dates::feature_info_date(
        fetcher,
        (&WMS.url, WMS_INFO_LAYER, WMS_DATE_FIELD),
        WMS.crs,
        bbox,
    )
}

/// The two height models.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Model {
    /// The terrain model, 1 m.
    Dgm1,
    /// The surface model of 20 cm, computed from the aerial pictures of the orthophoto flight.
    Dom20,
}

impl Model {
    pub fn label(self) -> &'static str {
        match self {
            Model::Dgm1 => "terrain model (LDBV DGM1)",
            Model::Dom20 => "surface model (LDBV DOM20)",
        }
    }

    /// `691_5334`: the south-west corner of the tile in km of EPSG:25832.
    pub fn stem(self, tile: TileId) -> String {
        format!("{}_{}", tile.east_km, tile.north_km)
    }

    /// Where the agency publishes the tile.
    pub fn tile_url(self, tile: TileId) -> String {
        match self {
            Model::Dgm1 => format!("{FILES}/dgm/dgm1/{}.tif", self.stem(tile)),
            Model::Dom20 => format!("{FILES}/dom20/DOM/32{}_20_DOM.tif", self.stem(tile)),
        }
    }

    /// The cache folder of the model.
    pub fn dir(self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(CACHE_DIR).join(match self {
            Model::Dgm1 => "dgm1",
            Model::Dom20 => "dom20",
        })
    }

    /// Whether the tile needs no download: a copy is cached, or the server did not have it.
    pub fn is_cached(self, cache: &CacheDir, tile: TileId) -> bool {
        let (dir, stem) = (self.dir(cache), self.stem(tile));
        cached(&dir, &stem, TILE_EXT).is_some() || missing_note(&dir, &stem).exists()
    }
}

/// The extension of a cached tile.
const TILE_EXT: &str = "tif";

/// Where the flight date of a cached DOM20 tile is kept: the reduced tile has no metadata.
fn flight_path(dir: &Path, stem: &str) -> PathBuf {
    dir.join(format!("{stem}.flight"))
}

/// `2024-06-29` of `29.06.2024`.
fn iso_date(day_month_year: &str) -> Option<String> {
    chrono::NaiveDate::parse_from_str(day_month_year.trim(), "%d.%m.%Y")
        .ok()
        .map(|date| date.format("%Y-%m-%d").to_string())
}

/// A DOM20 tile as it is pasted and cached: reduced to 1 m, each cell the highest of its 25
/// values. The flight date of its metadata is kept beside it.
fn reduced(dir: &Path, bytes: &[u8], tile: TileId) -> Result<Prepared, String> {
    let fine = decode_float_tiff(bytes).and_then(|raster| whole_tile(raster, tile, &[0.2]))?;
    let coarse = coarsen_max(&fine, DOM20_FACTOR)?;
    let kept = encode_float_geotiff(&coarse)?;
    let flight = flight_path(dir, &Model::Dom20.stem(tile));
    match gdal_metadata_item(bytes, "BILDFLUG_DATUM").and_then(|date| iso_date(&date)) {
        Some(date) => write_cache_file(&flight, date.as_bytes()).map_err(|e| e.to_string())?,
        None => {
            let _ = std::fs::remove_file(&flight);
        }
    }
    Ok((coarse, kept))
}

/// The tile stem of the name of a cached copy (`691_5334-20260115`).
fn stem_of(name: &str) -> &str {
    name.rsplit_once('-').map_or(name, |(stem, _)| stem)
}

/// The tile of a stem (`691_5334`).
fn tile_of(stem: &str) -> Option<TileId> {
    let (east, north) = stem.split_once('_')?;
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file. The addresses follow from the map area; a tile the server does not
/// have is missing when its centre lies in Bavaria and silent beyond the border.
#[allow(clippy::too_many_arguments)]
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<HeightModel, FetchError> {
    const NOT_ON_SERVER: &str = "not on the server (404)";
    let dir = model.dir(cache);
    let mut area = AreaTiles::default();
    let mut older = Vec::new();
    let mut checks = Checks::default();
    for tile in tiles_covering(bbox) {
        let (stem, url) = (model.stem(tile), model.tile_url(tile));
        let file = Published {
            dir: &dir,
            stem: &stem,
            ext: TILE_EXT,
            url: &url,
            // Beyond the border a tile that is not there stays not there; inside Bavaria the
            // server is asked again at every run.
            missing_holds: !tile_centre_in(tile, BAYERN),
        };
        let name = |path: &Path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        match current(fetcher, mode, &file, &mut checks, progress)? {
            Current::Cached(path) => area.named.push((tile, name(&path))),
            Current::Fetch { path, older: stale } => {
                area.named.push((tile, name(&path)));
                older.extend(stale);
            }
            Current::Missing if tile_centre_in(tile, BAYERN) => {
                area.missing.push((tile, NOT_ON_SERVER.to_string()));
            }
            Current::Missing => {}
        }
    }
    let list_url = format!("{FILES}/");
    let tile_url = |name: &str| {
        tile_of(stem_of(name)).map_or_else(|| list_url.clone(), |tile| model.tile_url(tile))
    };
    let reduce = |bytes: &[u8], tile: TileId| reduced(&dir, bytes, tile);
    let store = TileStore {
        label: model.label(),
        list_url: &list_url,
        source: FILES,
        url: &tile_url,
        cache_path: &|name| dir.join(format!("{name}.{TILE_EXT}")),
        marker_path: &|name| missing_note(&dir, stem_of(name)),
        // Whether the server has a tile was asked above.
        known_missing: &|_| false,
        // The file server answers some errors with an HTML page.
        precheck: &|bytes| {
            if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
                return Ok(());
            }
            let start = String::from_utf8_lossy(&bytes[..bytes.len().min(1000)]);
            Err(format!("not a GeoTIFF: {}", one_line(&start, 200)))
        },
        prepare: match model {
            Model::Dgm1 => &as_delivered,
            Model::Dom20 => &reduce,
        },
        explain_empty: true,
    };
    let mut warnings = Vec::new();
    let mut missing = mosaic_tiles(fetcher, mode, bbox, &store, area, root, out, &mut warnings)?;
    drop_older(&older);
    missing.retain(|tile| tile_centre_in(*tile, BAYERN));
    let centre = centre_tile(bbox);
    let date = match model {
        // The terrain tiles carry no date.
        Model::Dgm1 => None,
        Model::Dom20 if missing.contains(&centre) => None,
        Model::Dom20 => std::fs::read_to_string(flight_path(&dir, &model.stem(centre)))
            .ok()
            .map(|date| date.trim().to_string())
            .filter(|date| !date.is_empty()),
    };
    Ok(HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    })
}

/// The rasters of the Bayerische Vermessungsverwaltung (Bavaria): requested for the area and
/// stored in the project folder.
pub struct LdbvBackend {
    info: ProviderInfo,
}

impl LdbvBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(CACHE_DIR.to_string()),
                terrain_source: "ldbv-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for LdbvBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LdbvBackend {
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

/// The LDBV rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM20 from
/// the 1 km tiles. The DOM20 is only a template: an unavailable one costs a warning.
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
    warn_beyond_state(env, project, PROVIDER, "LDBV");
    let flight = || flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (&OrthophotoSource::Wms(WMS), "LDBV", &flight, None),
    )?;
    fetch_area_heights(
        env,
        project,
        bbox,
        STATE,
        (Model::Dgm1.label(), Model::Dom20.label()),
        &mut |surface, out, progress| {
            let model = if surface { Model::Dom20 } else { Model::Dgm1 };
            fetch_model(fetcher, cache, mode, model, bbox, dir, out, progress)
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
    use chrono::NaiveDate;

    use super::{FLIGHT_INFO_PREFIX, Model, WMS};
    use crate::fetch::test_support::{MapFetcher, geotiff, geotiff_with_metadata, solid_jpeg};
    use crate::raster::GridSpec;
    use crate::tiles::TileId;

    /// The GetFeatureInfo text answer of the orthophoto WMS at Regensburg (flight 11.05.2025),
    /// as the server gave it on 2026-11-14.
    pub const FLIGHT_INFO: &str = include_str!("../../../../tests/fixtures/bayern-dop-info.txt");

    /// The day the synthetic tiles were last changed on the server.
    pub fn changed() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    }

    /// A whole DGM1 tile: `z` at its west edge, rising 0.01 m per metre east.
    pub fn dgm1_tile(tile: TileId, z: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let heights: Vec<f32> = (0..1000 * 1000)
            .map(|i| z + (i % 1000) as f32 * 0.01)
            .collect();
        geotiff(&grid, &heights)
    }

    /// A whole DOM20 tile (5000 × 5000 cells of 20 cm) of the height `z`, with `peak` in the
    /// cell `(x, y)` if given, and the flight date (`DD.MM.YYYY`) in its GDAL metadata.
    pub fn dom20_tile(
        tile: TileId,
        z: f32,
        peak: Option<(u32, u32, f32)>,
        flight: &str,
    ) -> Vec<u8> {
        const SIDE: u32 = 5000;
        let mut heights = vec![z; (SIDE * SIDE) as usize];
        if let Some((x, y, top)) = peak {
            heights[(y * SIDE + x) as usize] = top;
        }
        let grid = GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 0.2,
            width: SIDE,
            height: SIDE,
        };
        let metadata = format!(
            "<GDALMetadata>\n  <Item name=\"BILDFLUG_DATUM\">{flight}</Item>\n</GDALMetadata>\n"
        );
        geotiff_with_metadata(&grid, &heights, &metadata)
    }

    /// Puts `bytes` on the server as the tile of `model`, last changed on `changed`.
    pub fn serve_tile(
        fetcher: &MapFetcher,
        model: Model,
        tile: TileId,
        bytes: Vec<u8>,
        changed: NaiveDate,
    ) {
        let url = model.tile_url(tile);
        fetcher.insert(&url, bytes);
        fetcher.set_modified(&url, changed);
    }

    /// Serves the LDBV rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the
    /// right size with the flight date of [`FLIGHT_INFO`], the DGM1 tiles (20 m rising 0.01 m
    /// per metre east) and the DOM20 tiles (32 m, flown on 29.06.2024), all last changed on
    /// [`changed`].
    pub fn serve_synthetic_ldbv(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        let photo = GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [0, 100, 200]));
        }
        fetcher.insert_prefix(FLIGHT_INFO_PREFIX, FLIGHT_INFO.as_bytes().to_vec());
        for tile in crate::tiles::tiles_covering(bbox) {
            serve_tile(fetcher, Model::Dgm1, tile, dgm1_tile(tile, 20.0), changed());
            let surface = dom20_tile(tile, 32.0, None, "29.06.2024");
            serve_tile(fetcher, Model::Dom20, tile, surface, changed());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{FLIGHT_INFO, changed, dgm1_tile, dom20_tile, serve_tile};
    use super::*;
    use crate::fetch::Cache;
    use crate::fetch::test_support::MapFetcher;

    /// Two tiles side by side in Munich (centre at the tile border 692 km).
    const BBOX: BoundingBox = BoundingBox {
        min_e: 691_900.0,
        min_n: 5_334_400.0,
        max_e: 692_100.0,
        max_n: 5_334_600.0,
    };
    const WEST: TileId = TileId {
        east_km: 691,
        north_km: 5334,
    };
    const EAST: TileId = TileId {
        east_km: 692,
        north_km: 5334,
    };

    /// The mosaic `out` is written without GeoTIFF tags (its world file has the grid).
    fn read_out(path: &Path) -> Vec<f32> {
        let bytes = std::fs::read(path).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
            panic!("not float")
        };
        values
    }

    /// Fetches `model` for `bbox` into `<dir>/out.tif` with the cache in `dir`.
    fn run(
        fetcher: &MapFetcher,
        dir: &Path,
        mode: FetchMode,
        model: Model,
        bbox: &BoundingBox,
    ) -> Result<(HeightModel, Vec<f32>), String> {
        let cache = Cache::new(dir.join("cache"));
        let out = dir.join("out.tif");
        let found = fetch_model(fetcher, &cache, mode, model, bbox, dir, &out, &mut |_| {})
            .map_err(|e| e.to_string())?;
        Ok((found, read_out(&out)))
    }

    fn serve_terrain(fetcher: &MapFetcher) {
        serve_tile(
            fetcher,
            Model::Dgm1,
            WEST,
            dgm1_tile(WEST, 500.0),
            changed(),
        );
        serve_tile(
            fetcher,
            Model::Dgm1,
            EAST,
            dgm1_tile(EAST, 600.0),
            changed(),
        );
    }

    #[test]
    fn tile_addresses_follow_the_agency() {
        assert_eq!(
            Model::Dgm1.tile_url(WEST),
            "https://download1.bayernwolke.de/a/dgm/dgm1/691_5334.tif"
        );
        assert_eq!(
            Model::Dom20.tile_url(WEST),
            "https://download1.bayernwolke.de/a/dom20/DOM/32691_5334_20_DOM.tif"
        );
        assert_eq!(Model::Dgm1.stem(WEST), "691_5334");
    }

    #[test]
    fn the_flight_date_is_the_ua_field() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(FLIGHT_INFO_PREFIX, FLIGHT_INFO.as_bytes().to_vec());
        assert_eq!(flight_date(&fetcher, &BBOX).as_deref(), Some("11.05.2025"));
        let url = fetcher.log.borrow()[0].clone();
        for part in [
            "LAYERS=by_dop20_info",
            "QUERY_LAYERS=by_dop20_info",
            "CRS=EPSG:25832",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(url.contains(part), "{part} in {url}");
        }
        for answer in [
            "GetFeatureInfo results:\n\n  Search returned no results.\n",
            "    nummer = '125039'\n    unr = '0'\n",
            "    ua = '2025-05-11'\n",
            "<ServiceExceptionReport/>",
        ] {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(&WMS.url, answer.as_bytes().to_vec());
            assert_eq!(flight_date(&fetcher, &BBOX), None, "{answer}");
        }
    }

    #[test]
    fn the_terrain_is_the_mosaic_of_the_tiles() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert_eq!(found.date, None);
        assert_eq!(values.len(), 200 * 200);
        // The last column of the west tile and the first of the east tile.
        assert_eq!(values[100 * 200 + 99], 500.0 + 999.0 * 0.01);
        assert_eq!(values[100 * 200 + 100], 600.0);
        let tiles = dir.path().join("cache/ldbv/dgm1");
        assert!(tiles.join("691_5334-20260115.tif").exists());
        assert!(tiles.join("692_5334-20260115.tif").exists());
        let cache = Cache::new(dir.path().join("cache"));
        assert!(Model::Dgm1.is_cached(&cache, WEST));
        assert!(!Model::Dom20.is_cached(&cache, WEST));
    }

    #[test]
    fn a_second_run_downloads_nothing_and_offline_asks_nothing() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        let (_, first) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        let downloads = fetcher.log.borrow().len();
        assert_eq!(downloads, 2);
        let (_, second) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        // Only the two checks went out, no tile a second time.
        assert_eq!(fetcher.log.borrow().len(), downloads);
        assert_eq!(first, second);
        let silent = MapFetcher::default();
        let (_, offline) =
            run(&silent, dir.path(), FetchMode::Offline, Model::Dgm1, &BBOX).unwrap();
        assert_eq!(silent.calls.get(), 0);
        assert_eq!(first, offline);
    }

    #[test]
    fn a_newer_tile_replaces_the_cached_one() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        let later = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        serve_tile(&fetcher, Model::Dgm1, WEST, dgm1_tile(WEST, 700.0), later);
        let (_, values) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        assert_eq!(values[100 * 200 + 99], 700.0 + 999.0 * 0.01);
        let tiles = dir.path().join("cache/ldbv/dgm1");
        assert!(tiles.join("691_5334-20260601.tif").exists());
        assert!(!tiles.join("691_5334-20260115.tif").exists());
        assert!(tiles.join("692_5334-20260115.tif").exists());
    }

    #[test]
    fn a_missing_tile_warns_inside_bavaria_only() {
        // A strip across the Iller south of Ulm: its west end is Baden-Württemberg.
        let strip = BoundingBox {
            min_e: 570_900.0,
            min_n: 5_358_400.0,
            max_e: 575_100.0,
            max_n: 5_358_600.0,
        };
        let tile = |east_km| TileId {
            east_km,
            north_km: 5358,
        };
        assert!(
            !tile_centre_in(tile(570), BAYERN),
            "570 lies beyond the border"
        );
        assert!(tile_centre_in(tile(575), BAYERN), "575 lies in Bavaria");
        // The server has the tiles from 573 on, as it has none beyond the border.
        let fetcher = MapFetcher::default();
        for east in 573..=575 {
            let bytes = dgm1_tile(tile(east), 480.0);
            serve_tile(&fetcher, Model::Dgm1, tile(east), bytes, changed());
        }
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &strip).unwrap();
        let expected: Vec<TileId> = (570..573)
            .map(tile)
            .filter(|t| tile_centre_in(*t, BAYERN))
            .collect();
        assert_eq!(found.missing, expected);
        assert!(!found.missing.contains(&tile(570)));
        assert!(values[0].is_nan());
        assert!(values[100 * 4200 + 4100].is_finite());
        // Offline the same: the tiles beyond the border are remembered as not there.
        let silent = MapFetcher::default();
        let (again, _) = run(&silent, dir.path(), FetchMode::Offline, Model::Dgm1, &strip).unwrap();
        assert_eq!(again.missing, expected);
        assert_eq!(silent.calls.get(), 0);
        // A tile inside Bavaria that the server lacks is missing, with a warning to come.
        let other = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        for east in 573..=574 {
            let bytes = dgm1_tile(tile(east), 480.0);
            serve_tile(&fetcher, Model::Dgm1, tile(east), bytes, changed());
        }
        let (found, _) = run(
            &fetcher,
            other.path(),
            FetchMode::Online,
            Model::Dgm1,
            &strip,
        )
        .unwrap();
        assert!(found.missing.contains(&tile(575)), "{:?}", found.missing);
        // The next run asks for that tile again and finds it; the tiles beyond the border
        // are not asked for a second time.
        let bytes = dgm1_tile(tile(575), 480.0);
        serve_tile(&fetcher, Model::Dgm1, tile(575), bytes, changed());
        let before = fetcher.calls.get();
        let (found, _) = run(
            &fetcher,
            other.path(),
            FetchMode::Online,
            Model::Dgm1,
            &strip,
        )
        .unwrap();
        assert!(!found.missing.contains(&tile(575)), "{:?}", found.missing);
        // Three checks (573 to 575) and one download; 570 to 572 are remembered.
        let beyond = (570..573)
            .filter(|east| !tile_centre_in(tile(*east), BAYERN))
            .count();
        assert_eq!(fetcher.calls.get() - before, 4 + (3 - beyond));
    }

    #[test]
    fn the_surface_model_is_cached_reduced_with_its_flight_date() {
        // Inside one tile; the cell (2502, 2502) of the 20 cm tile is 25 m higher.
        let bbox = BoundingBox {
            min_e: 691_400.0,
            min_n: 5_334_400.0,
            max_e: 691_600.0,
            max_n: 5_334_600.0,
        };
        let fetcher = MapFetcher::default();
        let bytes = dom20_tile(WEST, 400.0, Some((2502, 2502, 425.0)), "29.06.2024");
        serve_tile(&fetcher, Model::Dom20, WEST, bytes, changed());
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dom20, &bbox).unwrap();
        assert_eq!(found.date.as_deref(), Some("2024-06-29"));
        assert_eq!(values[100 * 200 + 100], 425.0);
        assert_eq!(values[100 * 200 + 101], 400.0);
        let kept = dir.path().join("cache/ldbv/dom20/691_5334-20260115.tif");
        let reduced = crate::sources::height_model::read_cached(&kept, WEST)
            .expect("the cached file is the 1 m tile");
        assert_eq!((reduced.grid.width, reduced.grid.height), (1000, 1000));
        assert!(std::fs::metadata(&kept).unwrap().len() < 5_000_000);
        // Offline the date comes from what was kept beside the tile.
        let silent = MapFetcher::default();
        let (again, _) = run(&silent, dir.path(), FetchMode::Offline, Model::Dom20, &bbox).unwrap();
        assert_eq!(again.date.as_deref(), Some("2024-06-29"));
        assert_eq!(silent.calls.get(), 0);
    }

    #[test]
    fn an_html_page_instead_of_a_tile_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let tiles = dir.path().join("cache/ldbv/dgm1");
        std::fs::create_dir_all(&tiles).unwrap();
        let older = tiles.join("691_5334-20250101.tif");
        std::fs::write(&older, dgm1_tile(WEST, 500.0)).unwrap();
        let fetcher = MapFetcher::default();
        serve_tile(
            &fetcher,
            Model::Dgm1,
            WEST,
            b"<html><body>502 Bad Gateway</body></html>".to_vec(),
            changed(),
        );
        serve_tile(
            &fetcher,
            Model::Dgm1,
            EAST,
            dgm1_tile(EAST, 600.0),
            changed(),
        );
        let error = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap_err();
        assert!(error.contains("691_5334"), "{error}");
        assert!(error.contains("not a GeoTIFF"), "{error}");
        assert!(!tiles.join("691_5334-20260115.tif").exists());
        assert!(older.exists(), "the older tile stays");
    }

    #[test]
    fn the_backend_names_its_provider() {
        let backend = LdbvBackend::new();
        let info = backend.provider();
        assert_eq!(info.id.as_str(), "ldbv");
        assert_eq!(info.cache_dir.as_deref(), Some("ldbv"));
        assert_eq!(info.terrain_source, "ldbv-dgm1");
        assert_eq!(info.kind, RasterKind::PerArea);
    }
}
