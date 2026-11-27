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

/// The share of `bbox`, in whole percent rounded up, that no box of `covered` reaches: a tile the
/// catalogue does not list (the coast, a tile not yet published). Sampled every 10 m.
fn uncovered_percent(bbox: &BoundingBox, covered: &[BoundingBox]) -> u32 {
    const STEP: f64 = 10.0;
    let (mut total, mut missing) = (0_u64, 0_u64);
    let mut n = bbox.max_n - STEP / 2.0;
    while n > bbox.min_n {
        let mut e = bbox.min_e + STEP / 2.0;
        while e < bbox.max_e {
            total += 1;
            let inside =
                |b: &BoundingBox| b.min_e <= e && e < b.max_e && b.min_n <= n && n < b.max_n;
            if !covered.iter().any(inside) {
                missing += 1;
            }
            e += STEP;
        }
        n -= STEP;
    }
    if total == 0 {
        return 0;
    }
    (missing * 100).div_ceil(total) as u32
}

/// The blocks of `layout` that lie under `bbox`.
fn blocks_under(layout: &TiledTiff, bbox: &BoundingBox) -> Vec<usize> {
    let grid = layout.grid();
    let (bw, bh) = (
        f64::from(layout.block_width),
        f64::from(layout.block_height),
    );
    let first = |from: f64, origin: f64, size: f64| {
        ((from - origin) / (size * grid.pixel_size))
            .floor()
            .max(0.0) as u32
    };
    let across = layout.blocks_across();
    let down = layout.height.div_ceil(layout.block_height);
    let x0 = first(bbox.min_e, grid.west, bw);
    let x1 = first(bbox.max_e - 1e-9, grid.west, bw).min(across.saturating_sub(1));
    let y0 = first(grid.north - bbox.max_n, 0.0, bh).min(down.saturating_sub(1));
    let y1 = first(grid.north - bbox.min_n - 1e-9, 0.0, bh).min(down.saturating_sub(1));
    let mut blocks = Vec::new();
    if bbox.max_e <= grid.west || bbox.min_n >= grid.north {
        return blocks;
    }
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
    let mut used = 0;
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
        let blocks = blocks_under(&layout, bbox);
        if blocks.is_empty() {
            continue;
        }
        progress(freemapper_adapter_api::note!(
            "progress-model-read",
            model = freemapper_adapter_api::note::model(&format!("{PROVIDER} {}", item.id))
        ));
        let read = read_blocks(fetcher, url, &layout, &blocks).map_err(|e| refusal(url, e))?;
        let grid = layout.grid();
        let across = layout.blocks_across();
        for (index, values) in read {
            let (bx, by) = (index as u32 % across, index as u32 / across);
            for row in 0..layout.block_height {
                let sy = by * layout.block_height + row;
                if sy >= layout.height {
                    break;
                }
                for column in 0..layout.block_width {
                    let sx = bx * layout.block_width + column;
                    if sx >= layout.width {
                        break;
                    }
                    // The centre of the source cell, in the grid of the area.
                    let e = grid.west + (f64::from(sx) + 0.5) * grid.pixel_size;
                    let n = grid.north - (f64::from(sy) + 0.5) * grid.pixel_size;
                    let (ox, oy) = (
                        ((e - terrain.grid.west) / terrain.grid.pixel_size).floor(),
                        ((terrain.grid.north - n) / terrain.grid.pixel_size).floor(),
                    );
                    if ox < 0.0
                        || oy < 0.0
                        || ox >= f64::from(terrain.grid.width)
                        || oy >= f64::from(terrain.grid.height)
                    {
                        continue;
                    }
                    let z = values[(row * layout.block_width + column) as usize];
                    if !z.is_nan() {
                        terrain.set(ox as u32, oy as u32, z);
                    }
                }
            }
        }
        used += 1;
        covered.push(grid.bounds());
        // The date of the tile that holds the middle of the area.
        let (mid_e, mid_n) = (
            (bbox.min_e + bbox.max_e) / 2.0,
            (bbox.min_n + bbox.max_n) / 2.0,
        );
        let grid_bounds = grid.bounds();
        if date.is_none()
            || (grid_bounds.min_e <= mid_e
                && mid_e < grid_bounds.max_e
                && grid_bounds.min_n <= mid_n
                && mid_n < grid_bounds.max_n)
        {
            date = item.datetime.clone();
        }
    }
    if used == 0 {
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
        let cache = CacheDir::new(scratch.join("cache"));
        let regions = Registry::new();
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
            backend().fetch(&mut env, &mut project, &area())
        };
        (result, lines, project)
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
        assert!(project.surface.is_none() && project.orthophoto.is_none());
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
}
