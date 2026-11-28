// SPDX-License-Identifier: AGPL-3.0-or-later

//! The terrain model of Lantmäteriet (Markhöjdmodell, 1 m, EPSG:3006 and so EPSG:25833 in the
//! plane): 10 km cloud-optimised GeoTIFFs of Float32 with Deflate blocks, found through the
//! STAC API and read by range, so that a map area costs the blocks it needs and not the 300 MB of
//! a tile.

use std::path::Path;

use crate::account;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::{FetchMode, Fetcher};
use freemapper_adapter_api::note::Note;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, HeightInput, OrthophotoInput, ProviderInfo, RasterArea, RasterBackend,
    RasterConfig, RasterKind, StoredRasters,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};
use freemapper_georef::crs::{BoundingBox, ProjectedPoint, utm33_inverse};
use freemapper_sources::float_tiff::write_float_raster;
use freemapper_sources::height_model::HeightModel;
use freemapper_sources::raster::{GridSpec, Raster};
use freemapper_sources::raster_fetch::{TERRAIN_RELPATH, height_record};
use freemapper_sources::stac;
use freemapper_sources::tiff_range::{TiledTiff, read_blocks, read_layout};

/// The name of the provider and of the raster backend a manifest names.
pub const PROVIDER: &str = "lantmateriet";

/// The cache folder of the provider and the name of its terrain model in the `source` tag.
const TERRAIN_SOURCE: &str = "lantmateriet-mhm";

/// The collection of the terrain model in the height catalogue.
const COLLECTION: &str = "dtm-cog";

/// The product a person orders for the download, named when the server refuses it.
const ORDER: &str = "Markhöjdmodell Nedladdning";

/// A terrain model in megabytes per km² of the area (about 0.56 MB per 512 px block, measured
/// on tile `617_38`, `docs/regions/sweden-reference.md` §8.4), with room for the blocks that
/// reach beyond the area.
const MB_PER_KM2: f64 = 3.0;

/// The terrain model of Lantmäteriet: requested for the area from the 10 km tiles by range and
/// stored in the project folder. Sweden has no orthophoto and no surface model here.
pub struct LantmaterietBackend {
    info: ProviderInfo,
    /// The STAC API of the height products, from the manifest.
    stac_hojd: String,
}

impl LantmaterietBackend {
    /// The backend of the manifest's `[terrain]`: it needs the endpoint `stac-hojd`.
    pub fn from_config(config: &RasterConfig) -> Result<Self, String> {
        Ok(Self {
            stac_hojd: config.require(PROVIDER, "stac-hojd")?,
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: TERRAIN_SOURCE.to_string(),
                kind: RasterKind::PerArea,
            },
        })
    }
}

/// The corners of `bbox` (EPSG:25833) as a box in degrees: west, south, east, north.
pub(crate) fn degrees(bbox: &BoundingBox) -> Option<[f64; 4]> {
    let mut out: Option<[f64; 4]> = None;
    for (easting, northing) in [
        (bbox.min_e, bbox.min_n),
        (bbox.min_e, bbox.max_n),
        (bbox.max_e, bbox.min_n),
        (bbox.max_e, bbox.max_n),
    ] {
        let at = utm33_inverse(ProjectedPoint { easting, northing })?;
        let [w, s, e, n] = out.get_or_insert([at.lon, at.lat, at.lon, at.lat]);
        *w = w.min(at.lon);
        *s = s.min(at.lat);
        *e = e.max(at.lon);
        *n = n.max(at.lat);
    }
    out
}

/// What a refused request means here: the account has not ordered the product, or it has
/// reached its limit.
fn refusal(url: &str, error: FetchError) -> FetchError {
    account::refusal(url, ORDER, error)
}

/// Whether the point (`e`, `n`) lies in `bounds`: the west and south edges belong to it, the east
/// and north edges to the neighbour.
fn holds(bounds: &BoundingBox, e: f64, n: f64) -> bool {
    bounds.min_e <= e && e < bounds.max_e && bounds.min_n <= n && n < bounds.max_n
}

/// The share of `bbox`, in whole percent rounded up, that no box of `covered` reaches: a tile the
/// catalogue does not list (the coast, a tile not yet published). Sampled in the middle of every
/// 10 m square.
fn uncovered_percent(bbox: &BoundingBox, covered: &[BoundingBox]) -> u32 {
    const STEP: f64 = 10.0;
    let across = (bbox.width() / STEP).ceil() as u32;
    let down = (bbox.height() / STEP).ceil() as u32;
    let total = u64::from(across) * u64::from(down);
    if total == 0 {
        return 0;
    }
    let missing = (0..down)
        .flat_map(|row| (0..across).map(move |column| (column, row)))
        .filter(|&(column, row)| {
            let e = bbox.min_e + (f64::from(column) + 0.5) * STEP;
            let n = bbox.max_n - (f64::from(row) + 0.5) * STEP;
            !covered.iter().any(|tile| holds(tile, e, n))
        })
        .count() as u64;
    (missing * 100).div_ceil(total) as u32
}

/// The blocks of `layout` (cells of 1 m) that lie under `bbox`, row by row.
fn blocks_under(layout: &TiledTiff, bbox: &BoundingBox) -> Vec<usize> {
    let bounds = layout.grid().bounds();
    let overlaps = bbox.min_e < bounds.max_e
        && bbox.max_e > bounds.min_e
        && bbox.min_n < bounds.max_n
        && bbox.max_n > bounds.min_n;
    if !overlaps {
        return Vec::new();
    }
    let (across, down) = (
        layout.blocks_across(),
        layout.height.div_ceil(layout.block_height),
    );
    let column = |e: f64| {
        ((e - bounds.min_e) / f64::from(layout.block_width))
            .floor()
            .clamp(0.0, f64::from(across - 1)) as u32
    };
    let row = |n: f64| {
        ((bounds.max_n - n) / f64::from(layout.block_height))
            .floor()
            .clamp(0.0, f64::from(down - 1)) as u32
    };
    // The far edge of the area belongs to the next block only when it passes the boundary.
    let (x0, x1) = (column(bbox.min_e), column(bbox.max_e - 1e-9));
    let (y0, y1) = (row(bbox.max_n), row(bbox.min_n + 1e-9));
    let mut blocks = Vec::new();
    for by in y0..=y1 {
        for bx in x0..=x1 {
            blocks.push((by * across + bx) as usize);
        }
    }
    blocks
}

/// Reads the terrain model of `bbox` from the tiles the catalogue lists and writes it as a
/// Float32 GeoTIFF `out`. Cells no tile covers, and NoData, stay `NaN`.
fn fetch_model(
    fetcher: &dyn Fetcher,
    stac_hojd: &str,
    bbox: &BoundingBox,
    dir: &Path,
    out: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<HeightModel, FetchError> {
    let wgs84 = degrees(bbox).ok_or_else(|| FetchError::Http {
        url: stac_hojd.to_string(),
        message: "the map area lies outside the coordinate system of Sweden".to_string(),
    })?;
    let items = stac::search(fetcher, stac_hojd, COLLECTION, wgs84, 100)?;
    let mut terrain = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let mut date = None;
    let mut covered: Vec<BoundingBox> = Vec::new();
    for item in &items {
        let Some(asset) = item.assets.get("data") else {
            continue;
        };
        let url = &asset.href;
        let layout = match read_layout(fetcher, url).map_err(|e| refusal(url, e))? {
            Ok(layout) => layout,
            Err(why) => {
                return Err(FetchError::Http {
                    url: url.clone(),
                    message: format!("not a terrain tile freemapper can read: {why}"),
                });
            }
        };
        let grid = layout.grid();
        if grid.pixel_size != 1.0 {
            return Err(FetchError::Http {
                url: url.clone(),
                message: format!(
                    "not a terrain tile freemapper can read: cells of {} m, not 1 m",
                    grid.pixel_size
                ),
            });
        }
        let blocks = blocks_under(&layout, bbox);
        if blocks.is_empty() {
            continue;
        }
        progress(freemapper_adapter_api::note!(
            "progress-model-read",
            model = freemapper_adapter_api::note::model(&format!("{PROVIDER} {}", item.id))
        ));
        let read = read_blocks(fetcher, url, &layout, &blocks).map_err(|e| refusal(url, e))?;
        let across = layout.blocks_across();
        // The whole metres between the grid of the tile and the grid of the area.
        let dx = (grid.west - terrain.grid.west).round() as i64;
        let dy = (terrain.grid.north - grid.north).round() as i64;
        let (width, height) = (
            i64::from(terrain.grid.width),
            i64::from(terrain.grid.height),
        );
        for (index, values) in read {
            let (bx, by) = (index as u32 % across, index as u32 / across);
            for row in 0..layout.block_height {
                let sy = by * layout.block_height + row;
                let oy = dy + i64::from(sy);
                if sy >= layout.height || oy < 0 || oy >= height {
                    continue;
                }
                for column in 0..layout.block_width {
                    let sx = bx * layout.block_width + column;
                    let ox = dx + i64::from(sx);
                    if sx >= layout.width || ox < 0 || ox >= width {
                        continue;
                    }
                    let z = values[(row * layout.block_width + column) as usize];
                    if !z.is_nan() {
                        terrain.set(ox as u32, oy as u32, z);
                    }
                }
            }
        }
        covered.push(grid.bounds());
        // The date of the tile that holds the middle of the area.
        let (mid_e, mid_n) = (
            (bbox.min_e + bbox.max_e) / 2.0,
            (bbox.min_n + bbox.max_n) / 2.0,
        );
        if date.is_none() || holds(&grid.bounds(), mid_e, mid_n) {
            date = item.datetime.clone();
        }
    }
    if covered.is_empty() {
        return Err(FetchError::Http {
            url: stac_hojd.to_string(),
            message: "no terrain model tile covers the map area; the terrain model of Lantmäteriet covers Sweden only".to_string(),
        });
    }
    let missing = uncovered_percent(bbox, &covered);
    if missing > 0 {
        progress(freemapper_adapter_api::note!(
            "warning-terrain-area-uncovered",
            model = freemapper_adapter_api::note::model(PROVIDER),
            percent = missing.to_string()
        ));
    }
    write_float_raster(&terrain, dir, out, TERRAIN_RELPATH)?;
    Ok(HeightModel {
        date,
        ..HeightModel::default()
    })
}

impl RasterBackend for LantmaterietBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let dir = project.dir().to_path_buf();
        let records = project.records();
        records.tiles.clear();
        records.ndsm_tiles.clear();
        *records.orthophoto = None;
        *records.surface = None;
        let out = dir.join(TERRAIN_RELPATH);
        match env.mode {
            FetchMode::Online => {
                let found =
                    fetch_model(env.fetcher, &self.stac_hojd, bbox, &dir, &out, env.progress)?;
                let records = project.records();
                *records.terrain = Some(height_record(
                    env.mode,
                    env.today,
                    bbox,
                    &found,
                    records.terrain,
                ));
            }
            FetchMode::Offline => {
                if !records.terrain.as_ref().is_some_and(|r| r.covers(bbox)) || !out.exists() {
                    return Err(AdapterError::TerrainNotFetched);
                }
            }
        }
        project.save()?;
        Ok(())
    }

    fn estimate(&self, area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let b = &area.bbox_25833;
        vec![EstimateItem {
            what: "Lantmäteriet terrain model".to_string(),
            megabytes: (b.width() * b.height() / 1e6 * MB_PER_KM2).max(0.1),
            cached: false,
            part: Part::Terrain,
            store: Store::Project,
        }]
    }

    fn derive_inputs(
        &self,
        _env: &DeriveEnv<'_>,
        _stored: &StoredRasters<'_>,
        _area: &RasterArea,
    ) -> DeriveInputs {
        DeriveInputs {
            outdated: Vec::new(),
            orthophoto: Ok(OrthophotoInput::Absent),
            dtm: Ok(HeightInput::Stored),
            dsm: None,
            ndsm: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::Path;

    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::env::AdapterEnv;
    use freemapper_adapter_api::error::{AdapterError, FetchError};
    use freemapper_adapter_api::fetch::{ByteRange, FetchMode, Fetcher};
    use freemapper_adapter_api::project::{RasterProject, RasterRecords};
    use freemapper_adapter_api::rasters::{RasterBackend, RasterConfig, RasterKind};
    use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_georef::crs::{BoundingBox, Crs};
    use freemapper_sources::raster::GridSpec;
    use freemapper_sources::testing::tiled_float_tiff_with;

    use super::*;

    const STAC: &str = "https://stac.example.org/stac-hojd/v1";

    /// The model a fetch stored: the TIFF and its world file.
    fn stored(path: &Path) -> Raster<f32> {
        let bytes = std::fs::read(path).unwrap();
        let world = std::fs::read_to_string(path.with_extension("tfw")).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let (width, height) = decoder.dimensions().unwrap();
        let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
            panic!("not Float32");
        };
        Raster {
            grid: GridSpec::from_world_file(&world, width, height).unwrap(),
            data: values,
        }
    }

    /// Answers a 401 for the URLs that start with `prefix`, as the download host does for an
    /// account that has not ordered the product.
    struct Unauthorised<'a> {
        inner: &'a MapFetcher,
        prefix: &'static str,
    }

    impl Fetcher for Unauthorised<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            if url.starts_with(self.prefix) {
                return Err(freemapper_adapter_api::fetch::status_answer_error(
                    url, 401, b"",
                ));
            }
            self.inner.get(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            if url.starts_with(self.prefix) {
                return Err(freemapper_adapter_api::fetch::status_answer_error(
                    url, 401, b"",
                ));
            }
            self.inner.get_range(url, range)
        }
    }

    /// The height at an easting and a northing of the synthetic model.
    fn height(e: f64, n: f64) -> f32 {
        ((e - 389_000.0) * 0.01 + (6_176_000.0 - n) * 0.001) as f32
    }

    /// A deflate tile of 1000 × 1000 m with its north-west corner at `west`, 6 176 000, in blocks
    /// of 256. A cell of the east side is NoData.
    fn tile(west: f64, nodata_at: Option<(u32, u32)>) -> Vec<u8> {
        let grid = GridSpec {
            west,
            north: 6_176_000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let mut values = Vec::new();
        for y in 0..1000 {
            for x in 0..1000 {
                values.push(if nodata_at == Some((x, y)) {
                    -9999.0
                } else {
                    height(west + f64::from(x) + 0.5, 6_176_000.0 - f64::from(y) - 0.5)
                });
            }
        }
        tiled_float_tiff_with(&grid, &values, 256, 3, None, 8)
    }

    fn item(id: &str, datetime: &str) -> String {
        format!(
            r#"{{"id":"{id}","collection":"dtm-cog","properties":{{"datetime":"{datetime}"}},
            "assets":{{"data":{{"href":"https://dl.example.org/{id}.tif","type":"image/tiff"}}}}}}"#
        )
    }

    fn search_answer(items: &[String]) -> Vec<u8> {
        format!(
            r#"{{"type":"FeatureCollection","features":[{}],"links":[]}}"#,
            items.join(",")
        )
        .into_bytes()
    }

    fn served(items: &[String]) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{STAC}/search?collections=dtm-cog"),
            search_answer(items),
        );
        fetcher.insert("https://dl.example.org/617_38.tif", tile(389_000.0, None));
        fetcher.insert(
            "https://dl.example.org/617_39.tif",
            tile(390_000.0, Some((50, 50))),
        );
        fetcher
    }

    fn both() -> Vec<String> {
        vec![item("617_38", "2025-09-29"), item("617_39", "2024-05-01")]
    }

    /// The area 389 900 to 390 100 by 6 175 900 to 6 176 000: the two tiles meet in it.
    fn area() -> BoundingBox {
        BoundingBox {
            min_e: 389_900.0,
            min_n: 6_175_900.0,
            max_e: 390_100.0,
            max_n: 6_176_000.0,
        }
    }

    /// Counts the bytes of the ranges that are read.
    struct Counting<'a> {
        inner: &'a MapFetcher,
        ranged: Cell<u64>,
    }

    impl Fetcher for Counting<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.inner.get(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            let bytes = self.inner.get_range(url, range)?;
            self.ranged.set(self.ranged.get() + bytes.len() as u64);
            Ok(bytes)
        }
    }

    struct TestProject {
        dir: std::path::PathBuf,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
    }

    impl RasterProject for TestProject {
        fn records(&mut self) -> RasterRecords<'_> {
            RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            area()
        }

        fn crs(&self) -> Crs {
            Crs::Utm33
        }
    }

    fn backend() -> LantmaterietBackend {
        let mut config = RasterConfig::default();
        config
            .endpoints
            .insert("stac-hojd".to_string(), STAC.to_string());
        LantmaterietBackend::from_config(&config).unwrap()
    }

    /// One fetch; the result, the progress lines and the project.
    fn run(
        fetcher: &dyn Fetcher,
        scratch: &Path,
        mode: FetchMode,
    ) -> (Result<(), AdapterError>, Vec<String>, TestProject) {
        run_for(fetcher, scratch, mode, &area())
    }

    fn run_for(
        fetcher: &dyn Fetcher,
        scratch: &Path,
        mode: FetchMode,
        area: &BoundingBox,
    ) -> (Result<(), AdapterError>, Vec<String>, TestProject) {
        let dir = scratch.join("project");
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = TestProject {
            dir,
            tiles: Vec::new(),
            ndsm_tiles: Vec::new(),
            orthophoto: None,
            terrain: None,
            surface: None,
        };
        let (result, lines) = fetch_with(fetcher, scratch, mode, area, &mut project);
        (result, lines, project)
    }

    /// One fetch into `project`, with the cache under `scratch`.
    fn fetch_with(
        fetcher: &dyn Fetcher,
        scratch: &Path,
        mode: FetchMode,
        area: &BoundingBox,
        project: &mut TestProject,
    ) -> (Result<(), AdapterError>, Vec<String>) {
        let cache = CacheDir::new(scratch.join("cache"));
        let regions = Registry::new();
        let mut lines = Vec::new();
        let result = {
            let mut progress = |note: freemapper_adapter_api::note::Note| {
                lines.push(note.id().to_string());
            };
            let mut env = AdapterEnv {
                fetcher,
                regions: &regions,
                cache: &cache,
                mode,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            backend().fetch(&mut env, project, area)
        };
        (result, lines)
    }

    #[test]
    fn the_backend_is_a_per_area_provider_without_a_surface_model() {
        let info = backend().provider().clone();
        assert_eq!(info.id.as_str(), "lantmateriet");
        assert_eq!(info.terrain_source, "lantmateriet-mhm");
        assert_eq!(info.kind, RasterKind::PerArea);
    }

    #[test]
    fn a_config_without_the_stac_endpoint_is_refused() {
        let error = LantmaterietBackend::from_config(&RasterConfig::default());
        assert!(error.is_err());
    }

    #[test]
    fn reads_only_the_blocks_under_the_area_and_joins_two_tiles() {
        let inner = served(&both());
        let fetcher = Counting {
            inner: &inner,
            ranged: Cell::new(0),
        };
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, project) = run(&fetcher, scratch.path(), FetchMode::Online);
        result.unwrap();
        let terrain = stored(&project.dir.join("data/dgm1.tif"));
        assert_eq!(terrain.grid.pixel_size, 1.0);
        assert_eq!(
            (terrain.grid.west, terrain.grid.north),
            (389_900.0, 6_176_000.0)
        );
        assert_eq!((terrain.grid.width, terrain.grid.height), (200, 100));
        // West of the border from the first tile, east of it from the second.
        for (x, y) in [(0_u32, 0_u32), (99, 50), (100, 50), (199, 99)] {
            let (e, n) = (389_900.5 + f64::from(x), 6_176_000.0 - f64::from(y) - 0.5);
            let got = terrain.get(x, y);
            assert!(
                (got - height(e, n)).abs() < 1e-3,
                "({x},{y}) {got} {}",
                height(e, n)
            );
        }
        // One tile is 1 000 000 cells; the area is 20 000 of them.
        let file = inner.responses.borrow()["https://dl.example.org/617_38.tif"].len() as u64;
        assert!(
            fetcher.ranged.get() < file,
            "{} of {file}",
            fetcher.ranged.get()
        );
        // The record holds the date of the tile at the middle of the area.
        let record = project.terrain.expect("terrain record");
        assert_eq!(
            record.bbox,
            [389_900.0, 6_175_900.0, 390_100.0, 6_176_000.0]
        );
        assert_eq!(
            record.date.as_deref(),
            Some("2024-05-01"),
            "the east tile holds the middle"
        );
        assert!(project.surface.is_none() && project.orthophoto.is_none());
    }

    fn bounds(min_e: f64, min_n: f64, max_e: f64, max_n: f64) -> BoundingBox {
        BoundingBox {
            min_e,
            min_n,
            max_e,
            max_n,
        }
    }

    #[test]
    fn degrees_gives_the_box_of_the_corners() {
        // Stockholm is at 674 032 east and 6 580 822 north in SWEREF 99 TM: 18.07 E, 59.33 N.
        let [west, south, east, north] =
            degrees(&bounds(674_032.0, 6_580_822.0, 674_032.0, 6_580_822.0)).unwrap();
        assert!(
            (west - 18.07).abs() < 0.02 && (east - 18.07).abs() < 0.02,
            "{west} {east}"
        );
        assert!(
            (south - 59.33).abs() < 0.02 && (north - 59.33).abs() < 0.02,
            "{south} {north}"
        );
        // The box has the west and south first, the east and north last.
        let [west, south, east, north] =
            degrees(&bounds(600_000.0, 6_500_000.0, 700_000.0, 6_600_000.0)).unwrap();
        assert!(
            west < east && south < north,
            "{west} {south} {east} {north}"
        );
    }

    #[test]
    fn a_box_holds_its_west_and_south_edges_and_not_its_east_and_north_edges() {
        let b = bounds(10.0, 20.0, 30.0, 40.0);
        assert!(holds(&b, 20.0, 30.0));
        assert!(holds(&b, 10.0, 30.0), "west edge");
        assert!(holds(&b, 20.0, 20.0), "south edge");
        assert!(!holds(&b, 30.0, 30.0), "east edge");
        assert!(!holds(&b, 20.0, 40.0), "north edge");
        assert!(!holds(&b, 9.9, 30.0) && !holds(&b, 30.1, 30.0));
        assert!(!holds(&b, 20.0, 19.9) && !holds(&b, 20.0, 40.1));
    }

    /// The blocks of the 1000 m tile `617_38` (4 by 4 blocks of 256 cells) under an area.
    fn blocks(min_e: f64, min_n: f64, max_e: f64, max_n: f64) -> Vec<usize> {
        let fetcher = served(&both());
        let layout = read_layout(&fetcher, "https://dl.example.org/617_38.tif")
            .unwrap()
            .unwrap();
        blocks_under(&layout, &bounds(min_e, min_n, max_e, max_n))
    }

    #[test]
    fn the_blocks_under_an_area_are_numbered_row_by_row() {
        // The tile spans 389 000 to 390 000 by 6 175 000 to 6 176 000. Column 1 is x 256 to 511,
        // row 2 is 512 to 767 below the north edge.
        assert_eq!(blocks(389_300.0, 6_175_300.0, 389_400.0, 6_175_400.0), [9]);
        assert_eq!(
            blocks(389_000.0, 6_175_000.0, 390_000.0, 6_176_000.0),
            (0..16).collect::<Vec<_>>()
        );
        // An area across a block boundary takes both blocks, each row after the other.
        assert_eq!(
            blocks(389_200.0, 6_175_900.0, 389_300.0, 6_175_950.0),
            [0, 1]
        );
        assert_eq!(
            blocks(389_100.0, 6_175_700.0, 389_150.0, 6_175_800.0),
            [0, 4]
        );
    }

    #[test]
    fn an_area_that_ends_on_a_block_boundary_does_not_take_the_next_block() {
        // x = 256 and 512 are boundaries; y = 256 below the north edge is 6 175 744.
        assert_eq!(blocks(389_100.0, 6_175_800.0, 389_256.0, 6_175_900.0), [0]);
        assert_eq!(blocks(389_256.0, 6_175_800.0, 389_300.0, 6_175_900.0), [1]);
        assert_eq!(blocks(389_100.0, 6_175_744.0, 389_200.0, 6_175_900.0), [0]);
        assert_eq!(blocks(389_100.0, 6_175_800.0, 389_200.0, 6_175_900.0), [0]);
    }

    #[test]
    fn an_area_over_the_edge_of_the_tile_takes_only_the_blocks_of_the_tile() {
        // The last column and row (3), reached from beyond the south-east corner.
        assert_eq!(blocks(389_800.0, 6_174_000.0, 391_000.0, 6_175_100.0), [15]);
        // From beyond the north-west corner: block 0 only.
        assert_eq!(blocks(388_000.0, 6_175_900.0, 389_100.0, 6_177_000.0), [0]);
        // Wholly outside, on each side, and touching an edge only: no block.
        assert!(blocks(388_000.0, 6_175_100.0, 388_900.0, 6_175_200.0).is_empty());
        assert!(blocks(390_100.0, 6_175_100.0, 390_200.0, 6_175_200.0).is_empty());
        assert!(blocks(389_100.0, 6_176_100.0, 389_200.0, 6_176_200.0).is_empty());
        assert!(blocks(389_100.0, 6_174_000.0, 389_200.0, 6_174_900.0).is_empty());
        assert!(blocks(388_000.0, 6_175_100.0, 389_000.0, 6_175_200.0).is_empty());
        assert!(blocks(390_000.0, 6_175_100.0, 391_000.0, 6_175_200.0).is_empty());
        assert!(blocks(389_100.0, 6_176_000.0, 389_200.0, 6_177_000.0).is_empty());
        assert!(blocks(389_100.0, 6_174_000.0, 389_200.0, 6_175_000.0).is_empty());
    }

    #[test]
    fn the_uncovered_share_is_counted_in_percent_rounded_up() {
        let area = bounds(0.0, 0.0, 100.0, 100.0);
        let tile = |max_e: f64| bounds(0.0, 0.0, max_e, 100.0);
        assert_eq!(uncovered_percent(&area, &[tile(50.0)]), 50);
        assert_eq!(uncovered_percent(&area, &[area]), 0);
        assert_eq!(uncovered_percent(&area, &[]), 100);
        assert_eq!(uncovered_percent(&area, &[tile(90.0)]), 10);
        // Samples lie at 5, 15, ...: a tile ending at 55 holds the sample at 55 not at all.
        assert_eq!(uncovered_percent(&area, &[tile(55.0)]), 50);
        assert_eq!(uncovered_percent(&area, &[tile(55.1)]), 40);
        // One column of ten, from the east: 90 % are missing; rows count from the north.
        assert_eq!(
            uncovered_percent(&area, &[bounds(0.0, 0.0, 10.0, 100.0)]),
            90
        );
        assert_eq!(
            uncovered_percent(&area, &[bounds(0.0, 90.0, 100.0, 100.0)]),
            90
        );
        assert_eq!(
            uncovered_percent(&area, &[bounds(0.0, 0.0, 100.0, 10.0)]),
            90
        );
        // A share that is not whole is rounded up; an area that is no multiple of 10 m is sampled
        // in as many squares as it needs.
        let odd = bounds(0.0, 0.0, 25.0, 100.0);
        assert_eq!(
            uncovered_percent(&odd, &[bounds(0.0, 0.0, 10.0, 100.0)]),
            67
        );
        assert_eq!(uncovered_percent(&bounds(0.0, 0.0, 0.0, 100.0), &[]), 0);
    }

    #[test]
    fn the_date_is_the_one_of_the_tile_at_the_middle_whatever_the_order() {
        let both_reversed: Vec<String> = both().into_iter().rev().collect();
        for items in [both(), both_reversed] {
            let fetcher = served(&items);
            let scratch = tempfile::tempdir().unwrap();
            let (result, _, project) = run(&fetcher, scratch.path(), FetchMode::Online);
            result.unwrap();
            assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-05-01"));
        }
    }

    #[test]
    fn without_a_tile_at_the_middle_the_date_is_the_first_tile_s() {
        let fetcher = served(&both()[..1]);
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, project) = run(&fetcher, scratch.path(), FetchMode::Online);
        result.unwrap();
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2025-09-29"));
    }

    #[test]
    fn an_area_in_the_middle_of_a_tile_is_copied_cell_by_cell() {
        // Block row 2 and column 1 of the tile: the rows above and the columns left of the area
        // are read with the blocks and left out.
        let area = bounds(389_300.0, 6_175_300.0, 389_400.0, 6_175_400.0);
        let fetcher = served(&both());
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, project) = run_for(&fetcher, scratch.path(), FetchMode::Online, &area);
        result.unwrap();
        let terrain = stored(&project.dir.join("data/dgm1.tif"));
        assert_eq!((terrain.grid.width, terrain.grid.height), (100, 100));
        for (x, y) in [(0_u32, 0_u32), (99, 0), (0, 99), (99, 99), (40, 60)] {
            let (e, n) = (389_300.5 + f64::from(x), 6_175_400.0 - f64::from(y) - 0.5);
            let got = terrain.get(x, y);
            assert!(
                (got - height(e, n)).abs() < 1e-3,
                "({x},{y}) {got} {}",
                height(e, n)
            );
        }
    }

    #[test]
    fn a_tile_with_cells_of_another_size_is_refused() {
        let inner = served(&both());
        let grid = GridSpec {
            west: 389_000.0,
            north: 6_176_000.0,
            pixel_size: 2.0,
            width: 500,
            height: 500,
        };
        inner.insert(
            "https://dl.example.org/617_38.tif",
            tiled_float_tiff_with(&grid, &vec![1.0; 250_000], 256, 3, None, 8),
        );
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, _) = run(&inner, scratch.path(), FetchMode::Online);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("cells of 2 m"), "{message}");
    }

    #[test]
    fn the_estimate_counts_the_area_in_square_kilometres() {
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let two_km2 = BoundingBox {
            min_e: 389_000.0,
            min_n: 6_175_000.0,
            max_e: 391_000.0,
            max_n: 6_176_000.0,
        };
        let items = backend().estimate(
            &EstimateArea {
                bbox_25833: two_km2,
                map_bbox: two_km2,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].part, Part::Terrain);
        assert!(
            (items[0].megabytes - 6.0).abs() < 1e-9,
            "{}",
            items[0].megabytes
        );
    }

    #[test]
    fn a_part_of_the_area_no_tile_covers_is_reported_with_its_share() {
        let fetcher = served(&both()[..1]);
        let scratch = tempfile::tempdir().unwrap();
        let (result, lines, _) = run(&fetcher, scratch.path(), FetchMode::Online);
        result.unwrap();
        assert!(
            lines.contains(&"warning-terrain-area-uncovered".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn an_area_every_tile_covers_has_no_such_warning() {
        let fetcher = served(&both());
        let scratch = tempfile::tempdir().unwrap();
        let (_, lines, _) = run(&fetcher, scratch.path(), FetchMode::Online);
        assert!(
            !lines.contains(&"warning-terrain-area-uncovered".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn nodata_stays_nan() {
        let inner = served(&both());
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, project) = run(&inner, scratch.path(), FetchMode::Online);
        result.unwrap();
        let terrain = stored(&project.dir.join("data/dgm1.tif"));
        // Cell (50, 50) of the second tile is the NoData cell: easting 390 050, northing 6 175 949,
        // so column 150, row 50 of the area.
        assert!(terrain.get(150, 50).is_nan());
        assert!(!terrain.get(151, 50).is_nan() && !terrain.get(150, 51).is_nan());
        assert_eq!(terrain.data.iter().filter(|z| z.is_nan()).count(), 1);
    }

    #[test]
    fn an_area_outside_every_item_is_an_error_that_says_so() {
        let inner = served(&[]);
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, project) = run(&inner, scratch.path(), FetchMode::Online);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("Sweden"), "{message}");
        assert!(!project.dir.join("data/dgm1.tif").exists());
    }

    #[test]
    fn a_refused_download_says_which_order_is_missing() {
        let inner = served(&both());
        let fetcher = Unauthorised {
            inner: &inner,
            prefix: "https://dl.example.org/",
        };
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, _) = run(&fetcher, scratch.path(), FetchMode::Online);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("Markhöjdmodell Nedladdning"), "{message}");
    }

    #[test]
    fn offline_keeps_what_the_project_holds_and_fails_without_it() {
        let inner = served(&both());
        let scratch = tempfile::tempdir().unwrap();
        let (result, _, _) = run(&inner, scratch.path(), FetchMode::Offline);
        assert!(matches!(
            result.unwrap_err(),
            AdapterError::TerrainNotFetched
        ));
    }

    #[test]
    fn offline_reads_nothing_when_the_project_holds_the_area_and_its_file() {
        let inner = served(&both());
        let scratch = tempfile::tempdir().unwrap();
        let (online, _, mut project) = run(&inner, scratch.path(), FetchMode::Online);
        online.unwrap();
        // Nothing may be asked: an empty fetcher answers every request with an error.
        let nothing = MapFetcher::default();
        let (offline, _) = fetch_with(
            &nothing,
            scratch.path(),
            FetchMode::Offline,
            &area(),
            &mut project,
        );
        offline.unwrap();
        assert!(nothing.log.borrow().is_empty());
        // A part of the area the record does not hold: not fetched.
        let wider = bounds(389_900.0, 6_175_900.0, 390_200.0, 6_176_000.0);
        let (outside, _) = fetch_with(
            &nothing,
            scratch.path(),
            FetchMode::Offline,
            &wider,
            &mut project,
        );
        assert!(matches!(
            outside.unwrap_err(),
            AdapterError::TerrainNotFetched
        ));
        // The record without its file: not fetched.
        std::fs::remove_file(project.dir.join("data/dgm1.tif")).unwrap();
        let (gone, _) = fetch_with(
            &nothing,
            scratch.path(),
            FetchMode::Offline,
            &area(),
            &mut project,
        );
        assert!(matches!(gone.unwrap_err(), AdapterError::TerrainNotFetched));
    }
}
