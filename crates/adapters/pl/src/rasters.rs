// SPDX-License-Identifier: AGPL-3.0-or-later

//! The rasters of Poland from GUGiK: the terrain model (NMT), the surface model (NMPT) and the
//! orthophoto, per sheet in PL-1992 (EPSG:2180; orthophoto sheets of old flights also in PL-2000). A fetch asks a [`SheetCatalog`] which sheets touch
//! the map area, keeps the best one per sheet number ([`best_per_cell`]), downloads them into the
//! shared cache and warps them into the grid of the map, where `derive` finds them like those of
//! any per-area provider: `data/dgm1.tif`, `data/dom1.tif` and `data/orthophoto-wms.jpg`.
//!
//! The catalog is the one part that needs the index service of GUGiK; everything else works on
//! the files, so it is built and tested without it.

use std::path::PathBuf;
use std::sync::Arc;

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::{FetchMode, Fetcher};
use freemapper_adapter_api::fs::write_inside;
use freemapper_adapter_api::note;
use freemapper_adapter_api::note::{Arg, Note};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::{
    AdapterId, EstimateItem, Part, Store, WcsRecord, WmsRecord, stored_bbox,
};
use freemapper_georef::crs::{BoundingBox, Crs, Pl1992, ProjectedPoint};
use freemapper_sources::asc::read_asc;
use freemapper_sources::float_tiff::write_float_raster;
use freemapper_sources::orto::{self, FINEST_PIXEL_M};
use freemapper_sources::raster::{GridSpec, Raster};
use freemapper_sources::raster_fetch::{
    SURFACE_RELPATH, TERRAIN_RELPATH, WMS_ORTHOPHOTO_RELPATH, keep_stored_surface, keep_surface,
};
use freemapper_sources::warp::{Resample, warp_into, warp_rgb_into_from};
use freemapper_sources::wms::ORTHOPHOTO_PIXEL_M;

use crate::sheets::{Product, Sheet, best_per_cell};

/// The id of the provider.
pub const PROVIDER: &str = "gugik";

/// The most a map area may download, in MB, unless a person asks for more.
pub const DEFAULT_CAP_MB: u64 = 2048;

/// MB of an ASC height sheet per km², measured on an NMT sheet of 2020 (31.7 MB for 5.5 km²).
const HEIGHT_MB_PER_KM2: f64 = 5.7;
/// MB of a 0.25 m orthophoto sheet per km² (44.3 MB for 5.5 km²).
const ORTHOPHOTO_MB_PER_KM2: f64 = 8.0;
/// The area of a sheet of the models, in km², for an estimate that knows no sheet yet.
const SHEET_KM2: f64 = 5.5;
/// The side of such a sheet, in metres.
const SHEET_SIDE_M: f64 = 2350.0;
/// Quality of the stored JPEG, as the WMS orthophotos have it.
const JPEG_QUALITY: u8 = 90;
/// The margin around the map area, in metres, that is read from the sheets so that the warp has
/// pixels on every side of the area's edge.
const MARGIN_M: f64 = 5.0;

/// Which sheets of a product touch an area: the index service of GUGiK, or a stand-in.
pub trait SheetCatalog: Send + Sync {
    /// Every sheet of `product` (all years, all resolutions) that touches `area` in PL-1992 and
    /// that this program can read. A sheet the index lists wrongly is left out and reported
    /// through `progress` as a warning; the others stay.
    fn sheets(
        &self,
        fetcher: &dyn Fetcher,
        product: Product,
        area: &BoundingBox,
        progress: &mut dyn FnMut(Note),
    ) -> Result<Vec<Sheet>, AdapterError>;
}

/// The rasters of GUGiK: requested for the area and stored in the project folder.
pub struct PolskaRasters {
    info: ProviderInfo,
    catalog: Arc<dyn SheetCatalog>,
    cap_mb: u64,
}

impl PolskaRasters {
    pub fn new(catalog: Arc<dyn SheetCatalog>) -> Self {
        Self::with_cap(catalog, DEFAULT_CAP_MB)
    }

    /// The same with another download cap, in MB.
    pub fn with_cap(catalog: Arc<dyn SheetCatalog>, cap_mb: u64) -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: "gugik-nmt".to_string(),
                kind: RasterKind::PerArea,
            },
            catalog,
            cap_mb,
        }
    }
}

/// `area` (in `crs`) as a box in PL-1992 with a small margin; `None` when a corner has no
/// position on the globe.
pub(crate) fn area_in_pl1992(crs: Crs, area: &BoundingBox) -> Option<BoundingBox> {
    let corners = [
        (area.min_e, area.min_n),
        (area.min_e, area.max_n),
        (area.max_e, area.min_n),
        (area.max_e, area.max_n),
    ];
    let mut out: Option<BoundingBox> = None;
    for (easting, northing) in corners {
        let p = Pl1992::forward(crs.inverse(ProjectedPoint { easting, northing })?);
        out = Some(match out {
            None => BoundingBox {
                min_e: p.easting,
                min_n: p.northing,
                max_e: p.easting,
                max_n: p.northing,
            },
            Some(b) => BoundingBox {
                min_e: b.min_e.min(p.easting),
                min_n: b.min_n.min(p.northing),
                max_e: b.max_e.max(p.easting),
                max_n: b.max_n.max(p.northing),
            },
        });
    }
    out.map(|b| BoundingBox {
        min_e: b.min_e - MARGIN_M,
        min_n: b.min_n - MARGIN_M,
        max_e: b.max_e + MARGIN_M,
        max_n: b.max_n + MARGIN_M,
    })
}

/// How many squares of a grid of [`SHEET_SIDE_M`] the span `lo..hi` (metres) touches.
fn squares_touched(lo: f64, hi: f64) -> f64 {
    (hi / SHEET_SIDE_M).floor() - (lo / SHEET_SIDE_M).floor() + 1.0
}

/// How many squares of the grid the box `b` touches.
fn squares_under(b: &BoundingBox) -> f64 {
    squares_touched(b.min_e, b.max_e) * squares_touched(b.min_n, b.max_n)
}

/// What `squares` squares weigh at `per_km2` MB a km².
fn square_megabytes(squares: f64, per_km2: f64) -> f64 {
    squares * SHEET_KM2 * per_km2
}

/// What a download of `sheet` is expected to weigh, in MB.
fn megabytes(product: Product, sheet: &Sheet) -> f64 {
    let b = &sheet.bbox_2180;
    let km2 = (b.max_e - b.min_e) * (b.max_n - b.min_n) / 1.0e6;
    match product {
        Product::Terrain | Product::Surface => km2 * HEIGHT_MB_PER_KM2,
        Product::Orthophoto => km2 * ORTHOPHOTO_MB_PER_KM2,
    }
}

fn product_dir(product: Product) -> &'static str {
    match product {
        Product::Terrain => "nmt",
        Product::Surface => "nmpt",
        Product::Orthophoto => "orto",
    }
}

fn extension(product: Product) -> &'static str {
    match product {
        Product::Terrain | Product::Surface => "asc",
        Product::Orthophoto => "tif",
    }
}

fn not_fetched(product: Product) -> AdapterError {
    match product {
        Product::Orthophoto => AdapterError::OrthophotoNotFetched,
        _ => AdapterError::TerrainNotFetched,
    }
}

fn refused(sheet: &Sheet, message: impl std::fmt::Display) -> AdapterError {
    AdapterError::Fetch(FetchError::Http {
        url: sheet.url.clone(),
        message: message.to_string(),
    })
}

impl PolskaRasters {
    /// The most one download may weigh, in bytes.
    fn cap_bytes(&self) -> u64 {
        self.cap_mb * 1_000_000
    }

    /// The best sheet per sheet number for `product` over `area`.
    fn choose(
        &self,
        fetcher: &dyn Fetcher,
        product: Product,
        area: &BoundingBox,
        progress: &mut dyn FnMut(Note),
    ) -> Result<Vec<Sheet>, AdapterError> {
        let finest = match product {
            Product::Orthophoto => FINEST_PIXEL_M,
            _ => 0.0,
        };
        let all = self.catalog.sheets(fetcher, product, area, progress)?;
        Ok(best_per_cell(&all, finest))
    }

    /// The file of `sheet` in the shared cache. The sheet number comes from the server, so
    /// nothing but letters, digits, dashes and single dots is let into the name.
    fn cache_file(
        env: &AdapterEnv<'_>,
        product: Product,
        sheet: &Sheet,
    ) -> Result<PathBuf, AdapterError> {
        // Letters, digits and dashes (`N-33-126-C-c-4-3`) and the dots of the old numbering
        // (`7.125.11.08`); never a leading dot or two in a row, so no name leads out of the folder.
        let clean = !sheet.godlo.is_empty()
            && !sheet.godlo.starts_with('.')
            && !sheet.godlo.contains("..")
            && sheet
                .godlo
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
        if !clean {
            return Err(refused(
                sheet,
                format!("the sheet number {:?} is no sheet number", sheet.godlo),
            ));
        }
        Ok(env
            .cache
            .provider_dir(PROVIDER)
            .join(product_dir(product))
            .join(format!(
                "{}_{}.{}",
                sheet.godlo,
                sheet.year,
                extension(product)
            )))
    }

    /// The bytes of `sheet`, downloaded into the cache first unless they lie there.
    fn sheet_bytes(
        &self,
        env: &mut AdapterEnv<'_>,
        product: Product,
        sheet: &Sheet,
    ) -> Result<(PathBuf, Vec<u8>), AdapterError> {
        let path = Self::cache_file(env, product, sheet)?;
        if !path.exists() {
            if env.mode == FetchMode::Offline {
                return Err(not_fetched(product));
            }
            let fetcher = env.fetcher;
            fetcher.download_to(&sheet.url, &path, self.cap_bytes(), env.progress)?;
        }
        let bytes = std::fs::read(&path).map_err(|e| AdapterError::Io(path.clone(), e))?;
        Ok((path, bytes))
    }

    /// The heights of `sheets` warped onto the 1 m grid of `bbox`.
    fn heights(
        &self,
        env: &mut AdapterEnv<'_>,
        product: Product,
        sheets: &[Sheet],
        crs: Crs,
        bbox: &BoundingBox,
    ) -> Result<Raster<f32>, AdapterError> {
        let mut out = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
        for sheet in sheets {
            let (path, bytes) = self.sheet_bytes(env, product, sheet)?;
            let src = read_asc(&bytes).map_err(|e| {
                // A broken download must not be read again from the cache.
                let _ = std::fs::remove_file(&path);
                refused(sheet, e)
            })?;
            warp_into(&mut out, crs, &src, Resample::Bilinear);
        }
        if out.data.iter().all(|z| z.is_nan()) {
            return Err(AdapterError::Fetch(FetchError::Http {
                url: "GUGiK".into(),
                message: "the sheets hold no height inside the map area".into(),
            }));
        }
        Ok(out)
    }

    /// The orthophoto of `sheets` warped onto the 0.2 m grid of `bbox` and stored as a JPEG with
    /// its world file.
    fn orthophoto(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        sheets: &[Sheet],
        area: &BoundingBox,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let crs = project.crs();
        let grid = GridSpec::covering(bbox, ORTHOPHOTO_PIXEL_M);
        let mut rgba = Raster::filled(grid, [0u8; 4]);
        let mut unreadable = None;
        for sheet in sheets {
            let (path, bytes) = self.sheet_bytes(env, Product::Orthophoto, sheet)?;
            let read = orto::Sheet::open(&bytes).and_then(|s| {
                let projection = s.crs.projection();
                s.read_window(&s.window_for(area))
                    .map(|w| w.map(|w| (w, projection)))
            });
            match read {
                Ok(Some((window, projection))) => {
                    warp_rgb_into_from(&mut rgba, crs, &window, &projection, Resample::Bilinear);
                }
                Ok(None) => {}
                // A sheet of an old flight may carry a coordinate system the reader does not know
                // (the old sheet numbers of 2014 do): it is left out with a warning, and the area
                // is still covered by the sheets of other flights. It is not kept in the cache.
                Err(e) => {
                    let _ = std::fs::remove_file(&path);
                    (env.progress)(note!(
                        "warning-error",
                        error = Arg::error(refused(sheet, &e))
                    ));
                    unreadable.get_or_insert_with(|| refused(sheet, e));
                }
            }
        }
        if rgba.data.iter().all(|p| p[3] == 0) {
            if let Some(error) = unreadable {
                return Err(error);
            }
            return Err(AdapterError::Fetch(FetchError::Http {
                url: "GUGiK".into(),
                message: "the orthophoto sheets hold no pixel inside the map area".into(),
            }));
        }
        let rgb: Vec<u8> = rgba.data.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
            .encode(
                &rgb,
                grid.width,
                grid.height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| {
                AdapterError::Fetch(FetchError::Http {
                    url: "GUGiK".into(),
                    message: e.to_string(),
                })
            })?;
        let dir = project.dir().to_path_buf();
        let out = dir.join(WMS_ORTHOPHOTO_RELPATH);
        write_inside(&dir, &out, &jpeg).map_err(|(p, e)| AdapterError::Io(p, e))?;
        write_inside(
            &dir,
            &out.with_extension("jgw"),
            grid.world_file().as_bytes(),
        )
        .map_err(|(p, e)| AdapterError::Io(p, e))?;
        Ok(())
    }
}

/// The sheets an online run used, as the project keeps them for an offline run.
struct StoredSheets {
    terrain: Vec<Sheet>,
    orthophoto: Vec<Sheet>,
    /// `None` when no run has chosen surface sheets yet.
    surface: Option<Vec<Sheet>>,
}

const SHEETS_RELPATH: &str = "data/gugik-sheets.json";

fn sheets_to_json(sheets: &[Sheet]) -> serde_json::Value {
    sheets
        .iter()
        .map(|s| serde_json::json!({"godlo": s.godlo, "year": s.year, "url": s.url}))
        .collect()
}

/// A sheet as stored: only what the cache file and its name need. The extent and the pixel size
/// are not read offline (the cap was checked online).
fn sheets_from_json(value: &serde_json::Value) -> Option<Vec<Sheet>> {
    value
        .as_array()?
        .iter()
        .map(|v| {
            Some(Sheet {
                godlo: v.get("godlo")?.as_str()?.to_string(),
                year: u16::try_from(v.get("year")?.as_u64()?).ok()?,
                resolution_m: 0.0,
                source: crate::sheets::SheetSource::Unknown,
                url: v.get("url")?.as_str()?.to_string(),
                bbox_2180: BoundingBox {
                    min_e: 0.0,
                    min_n: 0.0,
                    max_e: 0.0,
                    max_n: 0.0,
                },
            })
        })
        .collect()
}

fn read_stored(dir: &std::path::Path) -> Option<StoredSheets> {
    let text = std::fs::read_to_string(dir.join(SHEETS_RELPATH)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(StoredSheets {
        terrain: sheets_from_json(v.get("terrain")?)?,
        orthophoto: sheets_from_json(v.get("orthophoto")?)?,
        surface: v.get("surface").and_then(sheets_from_json),
    })
}

fn write_stored(
    dir: &std::path::Path,
    terrain: &[Sheet],
    orthophoto: &[Sheet],
    surface: Option<&[Sheet]>,
) -> Result<(), AdapterError> {
    let mut v = serde_json::json!({
        "terrain": sheets_to_json(terrain),
        "orthophoto": sheets_to_json(orthophoto),
    });
    if let Some(surface) = surface {
        v["surface"] = sheets_to_json(surface);
    }
    let text = serde_json::to_string_pretty(&v).unwrap_or_default();
    write_inside(dir, &dir.join(SHEETS_RELPATH), text.as_bytes())
        .map_err(|(p, e)| AdapterError::Io(p, e))
}

/// The years of `sheets` as one text, `2020` or `2018, 2020`.
fn years(sheets: &[Sheet]) -> String {
    let mut years: Vec<u16> = sheets.iter().map(|s| s.year).collect();
    years.sort_unstable();
    years.dedup();
    years
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

impl RasterBackend for PolskaRasters {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let crs = project.crs();
        let area = area_in_pl1992(crs, bbox).ok_or_else(|| {
            AdapterError::Fetch(FetchError::Http {
                url: "GUGiK".into(),
                message: "the map area has no position in PL-1992".into(),
            })
        })?;
        let fetcher = env.fetcher;
        let mut stored = read_stored(project.dir());
        let (terrain, photo, surface) = if env.mode == FetchMode::Offline {
            let stored = stored.take();
            // An offline run never asks the index service: it reads the sheets an online run
            // recorded in the project, and without them there is nothing to look up.
            let stored = stored.ok_or(AdapterError::TerrainNotFetched)?;
            let surface = if env.skip_surface {
                Vec::new()
            } else {
                stored.surface.unwrap_or_default()
            };
            (stored.terrain, stored.orthophoto, surface)
        } else {
            let terrain = self.choose(fetcher, Product::Terrain, &area, &mut *env.progress)?;
            let photo = self.choose(fetcher, Product::Orthophoto, &area, &mut *env.progress)?;
            let surface = if env.skip_surface {
                Vec::new()
            } else {
                self.choose(fetcher, Product::Surface, &area, &mut *env.progress)?
            };
            (terrain, photo, surface)
        };
        // The cap counts what the sheets weigh, cached or not: a map area that needs more than
        // the cap is too large for this provider whichever run asks.
        let total: f64 = [
            (Product::Terrain, &terrain),
            (Product::Orthophoto, &photo),
            (Product::Surface, &surface),
        ]
        .iter()
        .flat_map(|(product, sheets)| sheets.iter().map(|s| megabytes(*product, s)))
        .sum();
        if total > self.cap_mb as f64 {
            return Err(AdapterError::Fetch(FetchError::TooLarge {
                url: "the GUGiK sheets of this map area".into(),
                limit_mb: self.cap_mb,
            }));
        }
        let dir = project.dir().to_path_buf();
        if env.mode == FetchMode::Online {
            // A run that leaves the surface model out keeps the sheets recorded for it before.
            let kept = if env.skip_surface {
                stored.and_then(|s| s.surface)
            } else {
                Some(surface.clone())
            };
            write_stored(&dir, &terrain, &photo, kept.as_deref())?;
        }
        project.records().tiles.clear();
        project.records().ndsm_tiles.clear();

        (env.progress)(note!(
            "progress-model-read",
            model = note::model("terrain model (GUGiK NMT)")
        ));
        let raster = self.heights(env, Product::Terrain, &terrain, crs, bbox)?;
        write_float_raster(&raster, &dir, &dir.join(TERRAIN_RELPATH), "GUGiK NMT")
            .map_err(AdapterError::Fetch)?;
        *project.records().terrain = Some(WcsRecord {
            fetched: env.today,
            bbox: stored_bbox(bbox),
            date: Some(years(&terrain)),
        });

        (env.progress)(note!(
            "progress-orthophoto-download",
            provider = "GUGiK",
            service = "sheet download"
        ));
        self.orthophoto(env, project, &photo, &area, bbox)?;
        *project.records().orthophoto = Some(WmsRecord {
            flight: Some(years(&photo)),
            flight_label: None,
            fetched: env.today,
            bbox: stored_bbox(bbox),
        });
        project.save()?;

        if env.skip_surface {
            keep_stored_surface(env, project, bbox);
        } else {
            let label = "surface model (GUGiK NMPT)";
            (env.progress)(note!("progress-model-read", model = note::model(label)));
            match self
                .heights(env, Product::Surface, &surface, crs, bbox)
                .and_then(|r| {
                    write_float_raster(&r, &dir, &dir.join(SURFACE_RELPATH), "GUGiK NMPT")
                        .map_err(AdapterError::Fetch)
                }) {
                Ok(()) => {
                    *project.records().surface = Some(WcsRecord {
                        fetched: env.today,
                        bbox: stored_bbox(bbox),
                        date: Some(years(&surface)),
                    });
                }
                Err(e) => keep_surface(env, project, bbox, label, Arg::error(e)),
            }
        }
        project.save()?;
        Ok(())
    }

    /// The sheets are not known without the index service, so this counts the squares of a grid
    /// of the size of a sheet that the area touches: an upper bound, never less than one square
    /// of each product.
    fn estimate(&self, area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let Some(pl) = area_in_pl1992(Crs::Utm33, &area.bbox_25833) else {
            return Vec::new();
        };
        let n = squares_under(&pl);
        let item = |what: &str, per_km2: f64, part: Part| EstimateItem {
            what: what.to_string(),
            megabytes: square_megabytes(n, per_km2),
            cached: false,
            part,
            store: Store::Cache,
        };
        vec![
            item(
                "GUGiK terrain model (NMT)",
                HEIGHT_MB_PER_KM2,
                Part::Terrain,
            ),
            item("GUGiK orthophoto", ORTHOPHOTO_MB_PER_KM2, Part::Orthophoto),
            item(
                "GUGiK surface model (NMPT)",
                HEIGHT_MB_PER_KM2,
                Part::Surface,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sheets::SheetSource;
    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::project::RasterRecords;
    use freemapper_adapter_api::records::{NdsmRecord, TileRecord};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_georef::crs::LatLon;
    use freemapper_sources::orto::SheetCrs;
    use freemapper_sources::orto::test_support::{fixture_at, fixture_with_keys};
    use std::path::Path;

    /// Słubice: the place of the tests in PL-1992 and UTM 33.
    fn place() -> (ProjectedPoint, ProjectedPoint) {
        let ll = LatLon {
            lat: 52.35,
            lon: 14.57,
        };
        (Pl1992::forward(ll), Crs::Utm33.forward(ll))
    }

    /// A map area of `side` metres around the place, in UTM 33.
    fn area(side: f64) -> BoundingBox {
        let (_, utm) = place();
        BoundingBox {
            min_e: utm.easting - side / 2.0,
            min_n: utm.northing - side / 2.0,
            max_e: utm.easting + side / 2.0,
            max_n: utm.northing + side / 2.0,
        }
    }

    /// An ASC sheet of 80 by 80 cells of 1 m around the place, all of height `z`.
    fn asc(z: f32) -> Vec<u8> {
        let (pl, _) = place();
        let mut text = format!(
            "ncols 80\nnrows 80\nxllcorner {}\nyllcorner {}\ncellsize 1\nNODATA_value -9999\n",
            pl.easting.floor() - 40.0,
            pl.northing.floor() - 40.0
        );
        for _ in 0..80 {
            text.push_str(&format!("{z} ").repeat(80));
            text.push('\n');
        }
        text.into_bytes()
    }

    /// An orthophoto sheet of 8 by 8 m (0.25 m pixels) around the place.
    fn orto() -> Vec<u8> {
        let (pl, _) = place();
        fixture_at(
            0.25,
            2180,
            true,
            (0.0, 0.0),
            (pl.easting - 4.0, pl.northing + 4.0),
        )
    }

    fn sheet(godlo: &str, year: u16, resolution_m: f64, url: &str, km: f64) -> Sheet {
        let (pl, _) = place();
        let half = km * 500.0;
        Sheet {
            godlo: godlo.into(),
            year,
            resolution_m,
            source: SheetSource::Lidar,
            url: url.into(),
            bbox_2180: BoundingBox {
                min_e: pl.easting - half,
                min_n: pl.northing - half,
                max_e: pl.easting + half,
                max_n: pl.northing + half,
            },
        }
    }

    /// A catalog that lists fixed sheets by product.
    struct Stub(Vec<(Product, Sheet)>);

    impl SheetCatalog for Stub {
        fn sheets(
            &self,
            _: &dyn Fetcher,
            product: Product,
            _: &BoundingBox,
            _: &mut dyn FnMut(Note),
        ) -> Result<Vec<Sheet>, AdapterError> {
            Ok(self
                .0
                .iter()
                .filter(|(p, _)| *p == product)
                .map(|(_, s)| s.clone())
                .collect())
        }
    }

    const NMT: &str = "https://opendata.example/nmt.asc";
    const NMPT: &str = "https://opendata.example/nmpt.asc";
    const ORTO: &str = "https://opendata.example/orto.tif";

    /// The usual three sheets, served by `fetcher`.
    fn served() -> (Stub, MapFetcher) {
        let fetcher = MapFetcher::default();
        fetcher.insert(NMT, asc(7.0));
        fetcher.insert(NMPT, asc(12.0));
        fetcher.insert(ORTO, orto());
        let stub = Stub(vec![
            (
                Product::Terrain,
                sheet("N-33-126-C-c-4-3", 2020, 1.0, NMT, 0.08),
            ),
            (
                Product::Surface,
                sheet("N-33-126-C-c-4-3", 2020, 1.0, NMPT, 0.08),
            ),
            (
                Product::Orthophoto,
                sheet("N-33-126-C-c-4-3", 2025, 0.25, ORTO, 0.008),
            ),
        ]);
        (stub, fetcher)
    }

    struct TestProject {
        dir: PathBuf,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
    }

    impl TestProject {
        fn new(dir: &Path) -> Self {
            std::fs::create_dir_all(dir).unwrap();
            Self {
                dir: dir.to_path_buf(),
                tiles: Vec::new(),
                ndsm_tiles: Vec::new(),
                orthophoto: None,
                terrain: None,
                surface: None,
            }
        }
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
            area(10.0)
        }

        fn crs(&self) -> Crs {
            Crs::Utm33
        }
    }

    /// One fetch of the 6 m area; returns the result, the progress lines and the project.
    fn run(
        backend: &PolskaRasters,
        fetcher: &MapFetcher,
        scratch: &Path,
        mode: FetchMode,
        skip_surface: bool,
    ) -> (Result<(), AdapterError>, TestProject) {
        let cache = CacheDir::new(scratch.join("cache"));
        let regions = Registry::new();
        let mut project = TestProject::new(&scratch.join("project"));
        let mut progress = |_| {};
        let mut env = AdapterEnv {
            fetcher,
            regions: &regions,
            cache: &cache,
            mode,
            today: NaiveDate::from_ymd_opt(2026, 11, 20).unwrap(),
            skip_surface,
            progress: &mut progress,
        };
        let result = backend.fetch(&mut env, &mut project, &area(6.0));
        (result, project)
    }

    #[test]
    fn a_fetch_writes_the_terrain_the_surface_and_the_orthophoto_with_their_records() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        let (result, project) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, false);
        result.unwrap();
        let data = project.dir.join("data");
        for file in [
            "dgm1.tif",
            "dgm1.tfw",
            "dom1.tif",
            "orthophoto-wms.jpg",
            "orthophoto-wms.jgw",
        ] {
            assert!(data.join(file).exists(), "{file}");
        }
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2020"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2020"));
        assert_eq!(project.orthophoto.unwrap().flight.as_deref(), Some("2025"));
        assert_eq!(fetcher.log.borrow().len(), 3);
    }

    #[test]
    fn the_orthophoto_holds_the_colours_of_the_sheet_where_it_lies() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        run(&backend, &fetcher, scratch.path(), FetchMode::Online, true)
            .0
            .unwrap();
        let jpeg = scratch.path().join("project/data/orthophoto-wms.jpg");
        let image = image::open(jpeg).unwrap().into_rgb8();
        // The place lies at the corner of the four tiles (red, green, blue, white); the middle of
        // the area is within a pixel of it, so the four colours all show.
        let has = |near: [u8; 3]| {
            image
                .pixels()
                .any(|p| p.0.iter().zip(near).all(|(a, b)| a.abs_diff(b) < 40))
        };
        assert!(
            has([250, 20, 20]) && has([20, 250, 20]) && has([20, 20, 250]) && has([250, 250, 250])
        );
    }

    /// The orthophoto sheet around the place in PL-2000 zone 5, as old flights have it: once with
    /// its EPSG code, once user-defined (32767) with the parameters. Zone 5 has the central
    /// meridian of UTM 33, so the sheet lies straight in the project grid and the four colours
    /// must lie where the place is, in their quarters.
    #[test]
    fn a_pl2000_orthophoto_sheet_is_warped_to_where_it_lies() {
        let (_, utm) = place();
        let zone5 = SheetCrs::Pl2000(5).projection();
        let (e, n) = zone5.forward(52.35, 14.57);
        let origin = (e - 4.0, n + 4.0);
        let user_keys: Vec<u16> = vec![
            1, 1, 0, 7, // header: seven keys
            3072, 0, 1, 32767, // user-defined projected system
            3075, 0, 1, 1, // transverse Mercator
            3080, 34736, 1, 0, // central meridian
            3081, 34736, 1, 1, // latitude of origin
            3082, 34736, 1, 2, // false easting
            3083, 34736, 1, 3, // false northing
            3092, 34736, 1, 4, // scale
        ];
        let doubles = [15.0, 0.0, 5_500_000.0, 0.0, 0.999_923];
        for sheet_bytes in [
            fixture_at(0.25, 2176, true, (0.0, 0.0), origin),
            fixture_with_keys(0.25, &user_keys, &doubles, origin),
        ] {
            let (stub, fetcher) = served();
            fetcher.insert(ORTO, sheet_bytes);
            let backend = PolskaRasters::new(Arc::new(stub));
            let scratch = tempfile::tempdir().unwrap();
            run(&backend, &fetcher, scratch.path(), FetchMode::Online, true)
                .0
                .unwrap();
            let dir = scratch.path().join("project/data");
            let image = image::open(dir.join("orthophoto-wms.jpg"))
                .unwrap()
                .into_rgb8();
            let world = std::fs::read_to_string(dir.join("orthophoto-wms.jgw")).unwrap();
            let v: Vec<f64> = world.lines().map(|l| l.trim().parse().unwrap()).collect();
            let (pixel, west, north) = (v[0], v[4], v[5]);
            let colour_at = |de: f64, dn: f64| {
                let x = ((utm.easting + de - west) / pixel).round() as u32;
                let y = ((north - (utm.northing + dn)) / pixel).round() as u32;
                image.get_pixel(x, y).0
            };
            let near = |p: [u8; 3], q: [u8; 3]| p.iter().zip(q).all(|(a, b)| a.abs_diff(b) < 40);
            // A sheet in zone 5 straight in the UTM grid: red north-west of the place, green
            // north-east, blue south-west, white south-east.
            assert!(near(colour_at(-2.0, 2.0), [250, 20, 20]));
            assert!(near(colour_at(2.0, 2.0), [20, 250, 20]));
            assert!(near(colour_at(-2.0, -2.0), [20, 20, 250]));
            assert!(near(colour_at(2.0, -2.0), [250, 250, 250]));
        }
    }

    #[test]
    fn without_a_surface_model_none_is_asked_for_and_none_is_written() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        let (result, project) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        result.unwrap();
        assert!(project.surface.is_none());
        assert!(!project.dir.join("data/dom1.tif").exists());
        assert!(!fetcher.log.borrow().iter().any(|u| u == NMPT));
    }

    #[test]
    fn the_best_sheet_of_a_number_is_downloaded_and_a_too_fine_orthophoto_never() {
        let (_, fetcher) = served();
        fetcher.insert("https://opendata.example/old.asc", asc(1.0));
        fetcher.insert("https://opendata.example/orto005.tif", vec![0; 4]);
        let stub = Stub(vec![
            (
                Product::Terrain,
                sheet("A", 2014, 1.0, "https://opendata.example/old.asc", 0.08),
            ),
            (Product::Terrain, sheet("A", 2020, 1.0, NMT, 0.08)),
            (
                Product::Orthophoto,
                sheet(
                    "A",
                    2025,
                    0.05,
                    "https://opendata.example/orto005.tif",
                    0.008,
                ),
            ),
            (Product::Orthophoto, sheet("A", 2022, 0.25, ORTO, 0.008)),
        ]);
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        run(&backend, &fetcher, scratch.path(), FetchMode::Online, true)
            .0
            .unwrap();
        assert_eq!(*fetcher.log.borrow(), [NMT, ORTO]);
    }

    #[test]
    fn an_area_that_needs_more_than_the_cap_downloads_nothing() {
        let (_, fetcher) = served();
        // A sheet of 100 km² weighs about 570 MB for the heights alone.
        let stub = Stub(vec![
            (Product::Terrain, sheet("A", 2020, 1.0, NMT, 10.0)),
            (Product::Orthophoto, sheet("A", 2025, 0.25, ORTO, 0.008)),
        ]);
        let backend = PolskaRasters::with_cap(Arc::new(stub), 100);
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("100 MB"), "{message}");
        assert!(fetcher.log.borrow().is_empty());
    }

    #[test]
    fn offline_reads_the_cache_and_without_it_says_to_fetch_online() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Offline, true);
        assert!(matches!(result, Err(AdapterError::TerrainNotFetched)));
        run(&backend, &fetcher, scratch.path(), FetchMode::Online, true)
            .0
            .unwrap();
        let calls = fetcher.log.borrow().len();
        let (again, project) = run(&backend, &fetcher, scratch.path(), FetchMode::Offline, true);
        again.unwrap();
        assert!(project.terrain.is_some());
        assert_eq!(
            fetcher.log.borrow().len(),
            calls,
            "nothing is downloaded offline"
        );
    }

    /// A catalog that must not be asked.
    struct Forbidden;

    impl SheetCatalog for Forbidden {
        fn sheets(
            &self,
            _: &dyn Fetcher,
            _: Product,
            _: &BoundingBox,
            _: &mut dyn FnMut(Note),
        ) -> Result<Vec<Sheet>, AdapterError> {
            panic!("an offline run asked the index service");
        }
    }

    #[test]
    fn offline_never_asks_the_catalog_and_says_to_fetch_online_when_nothing_is_stored() {
        let (_, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(Forbidden));
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Offline, true);
        assert!(matches!(result, Err(AdapterError::TerrainNotFetched)));
        assert!(fetcher.log.borrow().is_empty());
    }

    #[test]
    fn offline_after_an_online_run_reads_the_recorded_sheets_without_the_catalog() {
        let (stub, fetcher) = served();
        let scratch = tempfile::tempdir().unwrap();
        run(
            &PolskaRasters::new(Arc::new(stub)),
            &fetcher,
            scratch.path(),
            FetchMode::Online,
            false,
        )
        .0
        .unwrap();
        let calls = fetcher.log.borrow().len();
        let backend = PolskaRasters::new(Arc::new(Forbidden));
        let (again, project) = run(
            &backend,
            &fetcher,
            scratch.path(),
            FetchMode::Offline,
            false,
        );
        again.unwrap();
        assert!(project.terrain.is_some() && project.surface.is_some());
        assert_eq!(fetcher.log.borrow().len(), calls);
    }

    #[test]
    fn a_broken_download_is_an_error_and_is_not_kept_in_the_cache() {
        let (stub, fetcher) = served();
        fetcher.insert(NMT, b"not a grid".to_vec());
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        assert!(result.is_err());
        let cached = scratch
            .path()
            .join("cache/gugik/nmt/N-33-126-C-c-4-3_2020.asc");
        assert!(!cached.exists());
    }

    const OLD_ORTO: &str = "https://opendata.geoportal.gov.pl/ortofotomapa/70076/B-OLD.tif";

    /// A sheet of an old flight that cannot be read is left out, and the map is still made from
    /// the other sheets; with nothing else to read, the fetch fails.
    #[test]
    fn an_unreadable_orthophoto_sheet_is_left_out_while_another_covers_the_area() {
        let (_, fetcher) = served();
        fetcher.insert(OLD_ORTO, b"not a tiff".to_vec());
        let both = Stub(vec![
            (
                Product::Terrain,
                sheet("N-33-126-C-c-4-3", 2020, 1.0, NMT, 0.08),
            ),
            (
                Product::Orthophoto,
                sheet("N-33-126-C-c-4-3", 2025, 0.25, ORTO, 0.008),
            ),
            (
                Product::Orthophoto,
                sheet("B-OLD", 2014, 0.25, OLD_ORTO, 0.008),
            ),
        ]);
        let backend = PolskaRasters::new(Arc::new(both));
        let scratch = tempfile::tempdir().unwrap();
        let (result, project) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        assert!(result.is_ok(), "{result:?}");
        assert!(project.dir().join(WMS_ORTHOPHOTO_RELPATH).exists());
        assert!(
            !scratch
                .path()
                .join("cache/gugik/orto/B-OLD_2014.tif")
                .exists()
        );

        let only_old = Stub(vec![
            (
                Product::Terrain,
                sheet("N-33-126-C-c-4-3", 2020, 1.0, NMT, 0.08),
            ),
            (
                Product::Orthophoto,
                sheet("B-OLD", 2014, 0.25, OLD_ORTO, 0.008),
            ),
        ]);
        let backend = PolskaRasters::new(Arc::new(only_old));
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        assert!(result.is_err());
    }

    #[test]
    fn a_sheet_number_that_is_a_path_is_refused_before_any_download() {
        let (_, fetcher) = served();
        let stub = Stub(vec![
            (Product::Terrain, sheet("../../evil", 2020, 1.0, NMT, 0.08)),
            (Product::Orthophoto, sheet("A", 2025, 0.25, ORTO, 0.008)),
        ]);
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        assert!(result.is_err());
        assert!(fetcher.log.borrow().is_empty());
    }

    /// The sheets of the old numbering (`7.125.11.08`, Kraków and other cities) have dots in their
    /// number: they are cached under it, and a number that climbs out of the folder is not.
    #[test]
    fn a_sheet_number_of_the_old_numbering_is_cached_under_its_number() {
        let scratch = tempfile::tempdir().unwrap();
        let registry = Registry::new();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let fetcher = MapFetcher::default();
        let path = |godlo: &str| {
            let mut progress = |_| {};
            let env = AdapterEnv {
                fetcher: &fetcher,
                regions: &registry,
                cache: &cache,
                mode: FetchMode::Online,
                today: NaiveDate::from_ymd_opt(2026, 11, 21).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            PolskaRasters::cache_file(&env, Product::Terrain, &sheet(godlo, 2021, 1.0, NMT, 0.08))
        };
        let ok = path("7.125.11.08").unwrap();
        assert!(ok.ends_with("nmt/7.125.11.08_2021.asc"), "{ok:?}");
        for bad in [".", "..", "a..b", ".hidden", "a/b", ""] {
            assert!(path(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_area_in_pl1992_surrounds_the_place_with_the_margin() {
        let (pl, _) = place();
        let b = area_in_pl1992(Crs::Utm33, &area(6.0)).unwrap();
        // The grids of UTM 33 and PL-1992 are turned against each other by a few degrees.
        for (got, want, sign) in [
            (b.min_e, pl.easting, -1.0),
            (b.max_e, pl.easting, 1.0),
            (b.min_n, pl.northing, -1.0),
            (b.max_n, pl.northing, 1.0),
        ] {
            let d = (got - want) * sign;
            assert!((3.0 + MARGIN_M..=3.5 + MARGIN_M + 0.5).contains(&d), "{d}");
        }
    }

    #[test]
    fn the_estimate_for_a_schoolyard_lists_the_three_downloads_by_whole_squares() {
        let backend = PolskaRasters::new(Arc::new(Stub(Vec::new())));
        let cache = CacheDir::new(std::env::temp_dir().join("freemapper-pl-estimate"));
        let (_, utm) = place();
        let map = area(200.0);
        let items = backend.estimate(
            &EstimateArea {
                bbox_25833: map,
                map_bbox: map,
                today: NaiveDate::from_ymd_opt(2026, 11, 20).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        let _ = utm;
        let parts: Vec<Part> = items.iter().map(|i| i.part).collect();
        assert_eq!(parts, [Part::Terrain, Part::Orthophoto, Part::Surface]);
        let squares = items[0].megabytes / (SHEET_KM2 * HEIGHT_MB_PER_KM2);
        assert!(
            (1.0..=4.0).contains(&squares) && squares.fract() == 0.0,
            "{squares}"
        );
        // One square is about 76 MB for the terrain model and the orthophoto together.
        let total: f64 = items.iter().map(|i| i.megabytes).sum();
        assert!(
            (total - squares * SHEET_KM2 * (2.0 * HEIGHT_MB_PER_KM2 + ORTHOPHOTO_MB_PER_KM2)).abs()
                < 1e-6
        );
        assert!(items.iter().all(|i| !i.cached && i.store == Store::Cache));
    }

    #[test]
    fn a_sheet_weighs_by_its_area_and_its_product() {
        let mut sheet = sheet("A", 2020, 1.0, NMT, 0.0);
        // 2 km by 3 km.
        sheet.bbox_2180 = BoundingBox {
            min_e: 1000.0,
            min_n: 5000.0,
            max_e: 3000.0,
            max_n: 8000.0,
        };
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(close(
            megabytes(Product::Terrain, &sheet),
            6.0 * HEIGHT_MB_PER_KM2
        ));
        assert!(close(
            megabytes(Product::Surface, &sheet),
            6.0 * HEIGHT_MB_PER_KM2
        ));
        assert!(close(
            megabytes(Product::Orthophoto, &sheet),
            6.0 * ORTHOPHOTO_MB_PER_KM2
        ));
    }

    #[test]
    fn the_squares_of_the_estimate_are_counted_by_the_span() {
        assert_eq!(squares_touched(10.0, 100.0), 1.0);
        assert_eq!(squares_touched(2300.0, 2400.0), 2.0);
        assert_eq!(squares_touched(100.0, 4800.0), 3.0);
        assert_eq!(square_megabytes(3.0, 2.0), 3.0 * SHEET_KM2 * 2.0);
        // Two squares east and three north are six, not five.
        let b = BoundingBox {
            min_e: 2300.0,
            max_e: 2400.0,
            min_n: 100.0,
            max_n: 4800.0,
        };
        assert_eq!(squares_under(&b), 6.0);
    }

    #[test]
    fn the_cap_is_taken_in_megabytes_of_a_million_bytes() {
        let backend = PolskaRasters::new(Arc::new(Stub(Vec::new())));
        assert_eq!(backend.cap_bytes(), 2_048_000_000);
    }

    #[test]
    fn the_cache_keeps_each_product_in_its_own_folder_under_its_own_extension() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        run(&backend, &fetcher, scratch.path(), FetchMode::Online, false)
            .0
            .unwrap();
        let cache = scratch.path().join("cache/gugik");
        assert!(cache.join("nmt/N-33-126-C-c-4-3_2020.asc").exists());
        assert!(cache.join("nmpt/N-33-126-C-c-4-3_2020.asc").exists());
        assert!(cache.join("orto/N-33-126-C-c-4-3_2025.tif").exists());
    }

    #[test]
    fn offline_without_the_orthophoto_names_the_orthophoto() {
        let (stub, fetcher) = served();
        let backend = PolskaRasters::new(Arc::new(stub));
        let scratch = tempfile::tempdir().unwrap();
        run(&backend, &fetcher, scratch.path(), FetchMode::Online, true)
            .0
            .unwrap();
        std::fs::remove_file(
            scratch
                .path()
                .join("cache/gugik/orto/N-33-126-C-c-4-3_2025.tif"),
        )
        .unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Offline, true);
        assert!(matches!(result, Err(AdapterError::OrthophotoNotFetched)));
    }

    #[test]
    fn an_area_that_weighs_exactly_the_cap_is_still_fetched() {
        let (_, fetcher) = served();
        // One orthophoto sheet of exactly 1 km² weighs 8 MB, the cap is 8 MB: the fetch goes on
        // and fails later on the missing terrain sheets, not on the cap.
        let stub = Stub(vec![(
            Product::Orthophoto,
            sheet("A", 2025, 0.25, ORTO, 1.0),
        )]);
        let backend = PolskaRasters::with_cap(Arc::new(stub), 8);
        let scratch = tempfile::tempdir().unwrap();
        let (result, _) = run(&backend, &fetcher, scratch.path(), FetchMode::Online, true);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("no height"), "{message}");
    }
}
