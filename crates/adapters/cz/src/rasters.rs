// SPDX-License-Identifier: AGPL-3.0-or-later

//! The rasters of Czechia from ČÚZK: the terrain model DMR 5G, the surface model DMP 1G and the
//! orthophoto, all from `ags.cuzk.gov.cz` (CC BY 4.0). The heights come from the two image
//! services of ArcGIS Server, asked for the map area in EPSG:25833 at 2 m, the pixel of the
//! models; the server mosaics its tiles and resamples into the grid of the map. A fetch reads
//! them in pieces, lays them on the 1 m grid of the area (bilinear) and stores them as
//! `data/dgm1.tif` and `data/dom1.tif`, where `derive` finds them like those of any per-area
//! provider. The orthophoto is the WMS of the service ORTOFOTO, stitched by the shared reader.

use std::borrow::Cow;
use std::path::Path;

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::{FetchMode, Fetcher, get_with_retries, one_line};
use freemapper_adapter_api::note::Note;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::float_tiff::{decode_float_tiff, require_stored, write_float_raster};
use freemapper_sources::height_model::HeightModel;
use freemapper_sources::raster::{GridSpec, Raster};
use freemapper_sources::raster_fetch::{fetch_area_heights, fetch_wms_orthophoto};
use freemapper_sources::wms::{OrthophotoSource, WmsService};

/// The id of the provider.
pub const PROVIDER: &str = "cuzk";

/// The host of the image services and of the orthophoto.
pub const HOST: &str = "ags.cuzk.gov.cz";

/// The image service of the terrain model DMR 5G.
pub const DMR5G: &str = "https://ags.cuzk.gov.cz/arcgis2/rest/services/dmr5g/ImageServer";

/// The image service of the surface model DMP 1G.
pub const DMP1G: &str = "https://ags.cuzk.gov.cz/arcgis2/rest/services/dmp1g/ImageServer";

/// The WMS of the orthophoto of the Czech Republic, asked for EPSG:25833 (the server lists
/// 32633 and answers 25833 too).
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://ags.cuzk.gov.cz/arcgis1/services/ORTOFOTO/MapServer/WMSServer"),
    layer: Cow::Borrowed("0"),
    crs: Crs::Utm33,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};

/// The pixel size of both models, in metres.
const NATIVE_M: f64 = 2.0;
/// The most pixels of one answer per side (the service answers 4100 rows at most).
const PIECE_PX: u32 = 4000;
/// MB of a Float32 piece per km² at 2 m (250 000 pixels of 4 bytes).
const HEIGHT_MB_PER_KM2: f64 = 1.05;
/// MB of the orthophoto per km² at 0.2 m, measured on a 2000 px JPEG of 0.16 km² (1.06 MB).
const ORTHOPHOTO_MB_PER_KM2: f64 = 6.6;

/// A height model of the services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DMR 5G, the terrain.
    Dmr5g,
    /// DMP 1G, the first-return surface.
    Dmp1g,
}

impl Model {
    /// The model as the log and the errors name it (`terrain model (ČÚZK DMR 5G)`).
    pub fn label(self) -> &'static str {
        match self {
            Self::Dmr5g => "terrain model (ČÚZK DMR 5G)",
            Self::Dmp1g => "surface model (ČÚZK DMP 1G)",
        }
    }

    fn service(self) -> &'static str {
        match self {
            Self::Dmr5g => DMR5G,
            Self::Dmp1g => DMP1G,
        }
    }

    fn source(self) -> &'static str {
        match self {
            Self::Dmr5g => "ČÚZK DMR 5G",
            Self::Dmp1g => "ČÚZK DMP 1G",
        }
    }
}

/// The request for `piece` (a grid of the models' pixel in EPSG:25833): the image of the service
/// as a Float32 GeoTIFF, bilinear where the server resamples.
pub fn export_url(model: Model, piece: &GridSpec) -> String {
    let b = piece.bounds();
    let epsg = Crs::Utm33.epsg();
    format!(
        "{}/exportImage?bbox={},{},{},{}&bboxSR={epsg}&imageSR={epsg}&size={},{}\
         &format=tiff&pixelType=F32&interpolation=RSP_BilinearInterpolation&f=image",
        model.service(),
        b.min_e,
        b.min_n,
        b.max_e,
        b.max_n,
        piece.width,
        piece.height
    )
}

/// The pieces of `grid`: sub-grids of at most [`PIECE_PX`] pixels a side that tile it.
fn pieces(grid: &GridSpec) -> Vec<GridSpec> {
    let mut out = Vec::new();
    let mut top = 0;
    while top < grid.height {
        let height = PIECE_PX.min(grid.height - top);
        let mut left = 0;
        while left < grid.width {
            let width = PIECE_PX.min(grid.width - left);
            out.push(GridSpec {
                west: grid.west + f64::from(left) * grid.pixel_size,
                north: grid.north - f64::from(top) * grid.pixel_size,
                pixel_size: grid.pixel_size,
                width,
                height,
            });
            left += width;
        }
        top += height;
    }
    out
}

/// One piece of a model: the TIFF of the service, or why it is none (an error text of the
/// server, a page of a proxy). The size is checked so that a piece lies where it was asked for.
fn fetch_piece(
    fetcher: &dyn Fetcher,
    model: Model,
    piece: &GridSpec,
) -> Result<Raster<f32>, FetchError> {
    let url = export_url(model, piece);
    get_with_retries(fetcher, &url, |bytes| {
        let refused = |message: String| FetchError::Http {
            url: url.clone(),
            message,
        };
        let raster = decode_float_tiff(bytes).map_err(|e| {
            let start = one_line(
                &String::from_utf8_lossy(&bytes[..bytes.len().min(200)]),
                200,
            );
            refused(format!("no height model in the answer ({e}): {start}"))
        })?;
        if (raster.grid.width, raster.grid.height) != (piece.width, piece.height) {
            return Err(refused(format!(
                "{} × {} pixels instead of {} × {}",
                raster.grid.width, raster.grid.height, piece.width, piece.height
            )));
        }
        Ok(raster)
    })
}

/// `coarse` laid on `grid` (finer, inside the same CRS): bilinear where the four cells around a
/// pixel hold a height, else the nearest cell that does, else `NaN` (outside the country).
pub fn upsample(coarse: &Raster<f32>, grid: GridSpec) -> Raster<f32> {
    let mut out = Raster::filled(grid, f32::NAN);
    let g = coarse.grid;
    let cell = |x: i64, y: i64| -> f32 {
        if x < 0 || y < 0 || x >= i64::from(g.width) || y >= i64::from(g.height) {
            f32::NAN
        } else {
            coarse.get(x as u32, y as u32)
        }
    };
    for y in 0..grid.height {
        for x in 0..grid.width {
            let e = grid.west + (f64::from(x) + 0.5) * grid.pixel_size;
            let n = grid.north - (f64::from(y) + 0.5) * grid.pixel_size;
            let fx = (e - g.west) / g.pixel_size - 0.5;
            let fy = (g.north - n) / g.pixel_size - 0.5;
            let (x0, y0) = (fx.floor(), fy.floor());
            let (tx, ty) = ((fx - x0) as f32, (fy - y0) as f32);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let corners = [
                (cell(x0, y0), (1.0 - tx) * (1.0 - ty)),
                (cell(x0 + 1, y0), tx * (1.0 - ty)),
                (cell(x0, y0 + 1), (1.0 - tx) * ty),
                (cell(x0 + 1, y0 + 1), tx * ty),
            ];
            let value = if corners.iter().all(|(z, _)| !z.is_nan()) {
                corners.iter().map(|(z, w)| z * w).sum()
            } else {
                corners
                    .iter()
                    .filter(|(z, _)| !z.is_nan())
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map_or(f32::NAN, |(z, _)| *z)
            };
            out.set(x, y, value);
        }
    }
    out
}

/// DMR 5G or DMP 1G over `bbox` (EPSG:25833) on the 1 m grid of the area, read from the service
/// in pieces at 2 m. Cells the model does not hold (outside Czechia) are `NaN`.
pub fn read_model(
    fetcher: &dyn Fetcher,
    model: Model,
    bbox: &BoundingBox,
) -> Result<Raster<f32>, FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let coarse_grid = GridSpec::covering(&grid.bounds(), NATIVE_M);
    let mut coarse = Raster::filled(coarse_grid, f32::NAN);
    for piece in pieces(&coarse_grid) {
        let part = fetch_piece(fetcher, model, &piece)?;
        coarse.paste(&part).map_err(|e| FetchError::Http {
            url: export_url(model, &piece),
            message: e.to_string(),
        })?;
    }
    if coarse.data.iter().all(|z| z.is_nan()) {
        return Err(FetchError::Http {
            url: model.service().to_string(),
            message: format!(
                "the {} holds no height inside the map area; it covers Czechia only",
                model.label()
            ),
        });
    }
    Ok(upsample(&coarse, grid))
}

/// [`read_model`] written as a Float32 GeoTIFF `out` with its world file. Offline, the model
/// stored at `out` is kept when it covers the area.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    if mode == FetchMode::Offline {
        let area = GridSpec::covering(bbox, 1.0).bounds();
        require_stored(out, &area, model.label())?;
        return Ok(HeightModel::default());
    }
    write_float_raster(
        &read_model(fetcher, model, bbox)?,
        root,
        out,
        model.source(),
    )?;
    Ok(HeightModel::default())
}

/// The rasters of ČÚZK: requested for the area and stored in the project folder.
pub struct CuzkRasters {
    info: ProviderInfo,
}

impl CuzkRasters {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: "cuzk-dmr5g".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for CuzkRasters {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for CuzkRasters {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    /// The orthophoto first, then the terrain model (required) and the surface model (a template:
    /// a failure costs a warning and a stored one stays while it covers the area).
    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let (fetcher, mode) = (env.fetcher, env.mode);
        project.records().tiles.clear();
        project.records().ndsm_tiles.clear();
        // The service names no flight date; a stored image is renewed after 30 days.
        let flight = || None;
        fetch_wms_orthophoto(
            env,
            project,
            bbox,
            (&OrthophotoSource::Wms(WMS), "ČÚZK", &flight, None),
        )?;
        let root = project.dir().to_path_buf();
        fetch_area_heights(
            env,
            project,
            bbox,
            "czechia",
            (Model::Dmr5g.label(), Model::Dmp1g.label()),
            &mut |surface, out: &Path, _progress: &mut dyn FnMut(Note)| {
                let model = if surface { Model::Dmp1g } else { Model::Dmr5g };
                fetch_model(fetcher, mode, model, bbox, &root, out)
            },
            &mut |_, _, _, _| {},
        )?;
        Ok(())
    }

    /// The area at the sizes the services answer: heights at 2 m as Float32, the orthophoto as
    /// JPEG at 0.2 m. All three are stored in the project folder.
    fn estimate(&self, area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let b = &area.bbox_25833;
        let km2 = (b.max_e - b.min_e) * (b.max_n - b.min_n) / 1.0e6;
        let item = |what: &str, per_km2: f64, part: Part| EstimateItem {
            what: what.to_string(),
            megabytes: km2 * per_km2,
            cached: false,
            part,
            store: Store::Project,
        };
        vec![
            item(
                "ČÚZK terrain model (DMR 5G)",
                HEIGHT_MB_PER_KM2,
                Part::Terrain,
            ),
            item("ČÚZK orthophoto", ORTHOPHOTO_MB_PER_KM2, Part::Orthophoto),
            item(
                "ČÚZK surface model (DMP 1G)",
                HEIGHT_MB_PER_KM2,
                Part::Surface,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use freemapper_adapter_api::adapter::RegionAdapter;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::project::RasterRecords;
    use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_sources::float_tiff::encode_float_geotiff;
    use std::path::PathBuf;

    /// Prague, Staroměstské náměstí, in UTM 33.
    const E: f64 = 458_000.0;
    const N: f64 = 5_548_000.0;

    fn area(side: f64) -> BoundingBox {
        BoundingBox {
            min_e: E,
            min_n: N,
            max_e: E + side,
            max_n: N + side,
        }
    }

    /// A model of `z` metres over the pieces of `grid` the fetch will ask for, as the service
    /// would answer: the whole piece, or `NaN` where `hole` says so.
    fn answer(piece: &GridSpec, z: f32, hole: impl Fn(u32, u32) -> bool) -> Vec<u8> {
        let mut raster = Raster::filled(*piece, z);
        for y in 0..piece.height {
            for x in 0..piece.width {
                if hole(x, y) {
                    raster.set(x, y, f32::NAN);
                }
            }
        }
        encode_float_geotiff(&raster).unwrap()
    }

    fn served(side: f64, terrain: f32, surface: f32) -> MapFetcher {
        let fetcher = MapFetcher::default();
        let grid = GridSpec::covering(&area(side), NATIVE_M);
        for piece in pieces(&grid) {
            fetcher.insert(
                &export_url(Model::Dmr5g, &piece),
                answer(&piece, terrain, |_, _| false),
            );
            fetcher.insert(
                &export_url(Model::Dmp1g, &piece),
                answer(&piece, surface, |_, _| false),
            );
        }
        fetcher
    }

    struct TestProject {
        dir: PathBuf,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
        side: f64,
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
            area(self.side)
        }

        fn crs(&self) -> Crs {
            Crs::Utm33
        }
    }

    #[test]
    fn the_request_names_the_piece_in_the_crs_of_the_map() {
        let grid = GridSpec::covering(&area(1000.0), NATIVE_M);
        assert_eq!(
            export_url(Model::Dmr5g, &grid),
            "https://ags.cuzk.gov.cz/arcgis2/rest/services/dmr5g/ImageServer/exportImage?\
             bbox=458000,5548000,459000,5549000&bboxSR=25833&imageSR=25833&size=500,500&format=tiff\
             &pixelType=F32&interpolation=RSP_BilinearInterpolation&f=image"
        );
        assert!(export_url(Model::Dmp1g, &grid).contains("/dmp1g/ImageServer/exportImage?"));
    }

    #[test]
    fn a_large_area_is_asked_for_in_pieces_that_tile_it() {
        let grid = GridSpec::covering(&area(20_000.0), NATIVE_M);
        let all = pieces(&grid);
        // 10 000 px a side: three by three pieces of at most 4000 px.
        assert_eq!(all.len(), 9);
        assert!(
            all.iter()
                .all(|p| p.width <= PIECE_PX && p.height <= PIECE_PX)
        );
        let pixels: u64 = all
            .iter()
            .map(|p| u64::from(p.width) * u64::from(p.height))
            .sum();
        assert_eq!(pixels, u64::from(grid.width) * u64::from(grid.height));
        let first = &all[0];
        assert_eq!((first.west, first.north), (grid.west, grid.north));
        let last = all.last().unwrap();
        assert_eq!(last.bounds().max_e, grid.bounds().max_e);
        assert_eq!(last.bounds().min_n, grid.bounds().min_n);
    }

    #[test]
    fn the_upsample_is_bilinear_between_the_cells_and_falls_back_to_the_nearest_one() {
        let coarse_grid = GridSpec {
            west: 0.0,
            north: 4.0,
            pixel_size: 2.0,
            width: 2,
            height: 2,
        };
        let mut coarse = Raster::filled(coarse_grid, 0.0f32);
        coarse.set(1, 0, 10.0);
        coarse.set(1, 1, 10.0);
        let fine = upsample(
            &coarse,
            GridSpec {
                west: 0.0,
                north: 4.0,
                pixel_size: 1.0,
                width: 4,
                height: 4,
            },
        );
        // Between the cell centres at x = 1 m and x = 3 m the value runs from 0 to 10.
        assert_eq!(fine.get(0, 0), 0.0);
        assert!((fine.get(1, 0) - 0.0).abs() < 1e-4);
        assert!((fine.get(2, 0) - 10.0).abs() < 1e-4);
        assert_eq!(fine.get(3, 0), 10.0);
        // A hole in one cell: the nearest cell with a height stands in.
        coarse.set(0, 0, f32::NAN);
        let holed = upsample(&coarse, fine.grid);
        assert!(!holed.get(1, 1).is_nan());
        // Outside the coarse grid nothing is known.
        let outside = upsample(
            &coarse,
            GridSpec {
                west: 100.0,
                north: 4.0,
                pixel_size: 1.0,
                width: 1,
                height: 1,
            },
        );
        assert!(outside.get(0, 0).is_nan());
    }

    #[test]
    fn a_model_is_stored_on_the_one_metre_grid_of_the_area() {
        let fetcher = served(40.0, 187.5, 200.0);
        let scratch = tempfile::tempdir().unwrap();
        let out = scratch.path().join("data/dgm1.tif");
        fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dmr5g,
            &area(40.0),
            scratch.path(),
            &out,
        )
        .unwrap();
        let stored = freemapper_sources::float_tiff::stored_grid(&out).unwrap();
        assert_eq!(
            (stored.width, stored.height, stored.pixel_size),
            (40, 40, 1.0)
        );
        // The values come from the same pieces: the read raster holds the height everywhere.
        let read = read_model(&fetcher, Model::Dmr5g, &area(40.0)).unwrap();
        assert_eq!(read.grid, stored);
        assert!(read.data.iter().all(|z| (z - 187.5).abs() < 1e-4));
    }

    #[test]
    fn an_answer_that_is_no_model_is_an_error_that_shows_what_came() {
        let fetcher = MapFetcher::default();
        let grid = GridSpec::covering(&area(40.0), NATIVE_M);
        fetcher.insert(
            &export_url(Model::Dmr5g, &grid),
            br#"{"error":{"code":400,"message":"Unable to complete operation."}}"#.to_vec(),
        );
        let scratch = tempfile::tempdir().unwrap();
        let error = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dmr5g,
            &area(40.0),
            scratch.path(),
            &scratch.path().join("dgm1.tif"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("no height model"), "{error}");
        assert!(error.contains("Unable to complete operation"), "{error}");
    }

    #[test]
    fn an_area_without_a_height_is_refused() {
        let fetcher = MapFetcher::default();
        let grid = GridSpec::covering(&area(40.0), NATIVE_M);
        fetcher.insert(
            &export_url(Model::Dmr5g, &grid),
            answer(&grid, 0.0, |_, _| true),
        );
        let scratch = tempfile::tempdir().unwrap();
        let error = fetch_model(
            &fetcher,
            FetchMode::Online,
            Model::Dmr5g,
            &area(40.0),
            scratch.path(),
            &scratch.path().join("dgm1.tif"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("covers Czechia only"), "{error}");
    }

    #[test]
    fn offline_needs_the_model_stored_for_the_area() {
        let fetcher = MapFetcher::default();
        let scratch = tempfile::tempdir().unwrap();
        let result = fetch_model(
            &fetcher,
            FetchMode::Offline,
            Model::Dmr5g,
            &area(40.0),
            scratch.path(),
            &scratch.path().join("dgm1.tif"),
        );
        assert!(
            matches!(result, Err(FetchError::NotStored { .. })),
            "{result:?}"
        );
        assert!(fetcher.log.borrow().is_empty());
    }

    #[test]
    fn the_estimate_lists_the_three_downloads_with_their_sizes() {
        let rasters = CuzkRasters::new();
        let cache = CacheDir::new(std::env::temp_dir().join("freemapper-cz-estimate"));
        let items = rasters.estimate(
            &EstimateArea {
                bbox_25833: area(1000.0),
                map_bbox: area(1000.0),
                today: NaiveDate::from_ymd_opt(2026, 11, 26).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(items.len(), 3, "{items:?}");
        assert!(items.iter().all(|i| i.what.contains("ČÚZK")));
        assert!(items.iter().all(|i| i.store == Store::Project));
        let ortho = items.iter().find(|i| i.part == Part::Orthophoto).unwrap();
        assert!((ortho.megabytes - ORTHOPHOTO_MB_PER_KM2).abs() < 1e-9);
    }

    /// A fetch of `side` metres through `backend`; the files and records it stored.
    fn fetched_through(backend: &dyn RasterBackend) {
        // The WMS answer: a 2000 px JPEG piece for a 400 m area.
        let side = 40.0;
        let fetcher = served(side, 187.5, 200.0);
        let image = image::RgbImage::from_pixel(200, 200, image::Rgb([10, 120, 30]));
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 90)
            .encode(image.as_raw(), 200, 200, image::ExtendedColorType::Rgb8)
            .unwrap();
        fetcher.insert_prefix(
            "https://ags.cuzk.gov.cz/arcgis1/services/ORTOFOTO/MapServer/WMSServer?",
            jpeg,
        );
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let regions = Registry::new();
        let mut progress = |_| {};
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &regions,
            cache: &cache,
            mode: FetchMode::Online,
            today: NaiveDate::from_ymd_opt(2026, 11, 26).unwrap(),
            skip_surface: false,
            progress: &mut progress,
        };
        let dir = scratch.path().join("project");
        std::fs::create_dir_all(&dir).unwrap();
        let mut project = TestProject {
            dir: dir.clone(),
            tiles: Vec::new(),
            ndsm_tiles: Vec::new(),
            orthophoto: None,
            terrain: None,
            surface: None,
            side,
        };
        backend.fetch(&mut env, &mut project, &area(side)).unwrap();
        for file in ["dgm1.tif", "dgm1.tfw", "dom1.tif", "orthophoto-wms.jpg"] {
            assert!(dir.join("data").join(file).exists(), "{file}");
        }
        assert!(project.terrain.is_some() && project.surface.is_some());
        assert!(project.orthophoto.is_some());
    }
    #[test]
    fn a_fetch_without_a_flight_date_from_the_service_stores_all_three_rasters() {
        fetched_through(&CuzkRasters::new());
    }

    /// The adapter's guarded rasters pass the fetch on to the backend.
    #[test]
    fn the_guarded_rasters_of_the_adapter_store_all_three_rasters_too() {
        let cesko = crate::cesko::Cesko::new(&[]);
        fetched_through(cesko.rasters().unwrap());
    }

    #[test]
    fn the_upsample_of_a_slope_is_exact_between_the_cell_centres() {
        // z = 10 * column + 100 * row on 2 m cells; a linear surface is reproduced by bilinear
        // interpolation, whatever the origin of the grids.
        let coarse_grid = GridSpec {
            west: 100.0,
            north: 206.0,
            pixel_size: 2.0,
            width: 3,
            height: 3,
        };
        let mut coarse = Raster::filled(coarse_grid, 0.0f32);
        for y in 0..3 {
            for x in 0..3 {
                coarse.set(x, y, 10.0 * x as f32 + 100.0 * y as f32);
            }
        }
        // Pixel centres at 101.25 ..= 104.75 m east and 204.75 ..= 201.25 m north.
        let fine_grid = GridSpec {
            west: 101.0,
            north: 205.0,
            pixel_size: 0.5,
            width: 8,
            height: 8,
        };
        let fine = upsample(&coarse, fine_grid);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let east = 101.0 + (f64::from(x) + 0.5) * 0.5;
                let north = 205.0 - (f64::from(y) + 0.5) * 0.5;
                let column = (east - 100.0) / 2.0 - 0.5;
                let row = (206.0 - north) / 2.0 - 0.5;
                let expected = (10.0 * column + 100.0 * row) as f32;
                assert!(
                    (fine.get(x, y) - expected).abs() < 1e-3,
                    "({x}, {y}): {} against {expected}",
                    fine.get(x, y)
                );
            }
        }
    }

    #[test]
    fn the_models_are_named_for_the_log_and_the_record() {
        assert_eq!(Model::Dmr5g.label(), "terrain model (ČÚZK DMR 5G)");
        assert_eq!(Model::Dmp1g.label(), "surface model (ČÚZK DMP 1G)");
        assert_eq!(Model::Dmr5g.source(), "ČÚZK DMR 5G");
        assert_eq!(Model::Dmp1g.source(), "ČÚZK DMP 1G");
    }
}
