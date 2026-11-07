//! Terrain templates from the DTM: mosaic, hillshade and slope.

use std::path::Path;

use super::{DeriveError, read_member};
use crate::crs::BoundingBox;
use crate::fetch::FetchedTile;
use crate::raster::{GridSpec, Raster};

/// DGM1 marks cells without data with -9999.
const NODATA_BELOW: f32 = -9998.0;
/// Sun position for the hillshade, in degrees.
const SUN_AZIMUTH_DEG: f64 = 315.0;
const SUN_ALTITUDE_DEG: f64 = 45.0;
/// Slope at which the slope template is black.
const SLOPE_BLACK_DEG: f32 = 30.0;

/// Mosaics the DTM tiles over `bbox`; cells without data are `NaN`.
pub fn mosaic_dtm(tiles: &[FetchedTile], bbox: &BoundingBox) -> Result<Raster<f32>, DeriveError> {
    let mut mosaic: Option<Raster<f32>> = None;
    for tile in tiles {
        let tiff_bytes = read_member(&tile.zip, &tile.product.raster_member(tile.tile))?;
        let world = read_member(&tile.zip, &tile.product.world_file_member(tile.tile))?;
        let tiff_error = |e: tiff::TiffError| DeriveError::Image(e.to_string());
        let mut decoder =
            tiff::decoder::Decoder::new(std::io::Cursor::new(tiff_bytes)).map_err(tiff_error)?;
        let (width, height) = decoder.dimensions().map_err(tiff_error)?;
        let tiff::decoder::DecodingResult::F32(values) =
            decoder.read_image().map_err(tiff_error)?
        else {
            return Err(DeriveError::Image(format!(
                "{}: DTM is not 32-bit float",
                tile.tile.name()
            )));
        };
        let grid = GridSpec::from_world_file(&String::from_utf8_lossy(&world), width, height)?;
        let mosaic = mosaic.get_or_insert_with(|| {
            Raster::filled(GridSpec::covering(bbox, grid.pixel_size), f32::NAN)
        });
        mosaic.paste_with(&grid, |x, y| {
            let z = values[y as usize * width as usize + x as usize];
            if z < NODATA_BELOW { f32::NAN } else { z }
        });
    }
    mosaic.ok_or(DeriveError::NoTiles)
}

/// Height gradient (dz/de, dz/dn) at a cell after Horn, with edge cells replicated and the
/// spacing reduced accordingly; `None` if a height in the 3 × 3 window is missing.
fn gradient(dtm: &Raster<f32>, x: u32, y: u32) -> Option<(f64, f64)> {
    let (w, h) = (dtm.grid.width, dtm.grid.height);
    let (left, right) = (x.saturating_sub(1), (x + 1).min(w - 1));
    let (top, bottom) = (y.saturating_sub(1), (y + 1).min(h - 1));
    let z = |xx: u32, yy: u32| -> Option<f64> {
        let v = dtm.get(xx, yy);
        (!v.is_nan()).then_some(f64::from(v))
    };
    let column = |xx: u32| Some(z(xx, top)? + 2.0 * z(xx, y)? + z(xx, bottom)?);
    let row = |yy: u32| Some(z(left, yy)? + 2.0 * z(x, yy)? + z(right, yy)?);
    let (west, east, north, south) = (column(left)?, column(right)?, row(top)?, row(bottom)?);
    // The sums above cover all cells but the centre one; it must exist too.
    column(x)?;
    let px = dtm.grid.pixel_size;
    let spacing = |a: u32, b: u32| f64::from(b - a) * px;
    let dz_de = if right > left {
        (east - west) / (4.0 * spacing(left, right))
    } else {
        0.0
    };
    let dz_dn = if bottom > top {
        (north - south) / (4.0 * spacing(top, bottom))
    } else {
        0.0
    };
    Some((dz_de, dz_dn))
}

fn map_cells<T: Copy>(dtm: &Raster<f32>, f: impl Fn(f64, f64) -> T) -> Raster<Option<T>> {
    let mut out = Raster::filled(dtm.grid, None);
    for y in 0..dtm.grid.height {
        for x in 0..dtm.grid.width {
            out.set(x, y, gradient(dtm, x, y).map(|(de, dn)| f(de, dn)));
        }
    }
    out
}

/// Hillshade, sun at azimuth 315° and altitude 45°; flat ground is 180.
pub fn hillshade(dtm: &Raster<f32>) -> Raster<Option<u8>> {
    let (az, alt) = (SUN_AZIMUTH_DEG.to_radians(), SUN_ALTITUDE_DEG.to_radians());
    let sun = (az.sin() * alt.cos(), az.cos() * alt.cos(), alt.sin());
    map_cells(dtm, |dz_de, dz_dn| {
        let norm = (dz_de * dz_de + dz_dn * dz_dn + 1.0).sqrt();
        let light = (-dz_de * sun.0 - dz_dn * sun.1 + sun.2) / norm;
        (255.0 * light.max(0.0)).round() as u8
    })
}

/// Slope in degrees.
pub fn slope_deg(dtm: &Raster<f32>) -> Raster<Option<f32>> {
    map_cells(dtm, |dz_de, dz_dn| {
        (dz_de * dz_de + dz_dn * dz_dn).sqrt().atan().to_degrees() as f32
    })
}

/// Slope as grey: flat white, 30° and steeper black.
pub fn slope_shade(slope: &Raster<Option<f32>>) -> Raster<Option<u8>> {
    Raster {
        grid: slope.grid,
        data: slope
            .data
            .iter()
            .map(|s| {
                s.map(|s| 255 - (s.min(SLOPE_BLACK_DEG) / SLOPE_BLACK_DEG * 255.0).round() as u8)
            })
            .collect(),
    }
}

/// Writes a grey + alpha PNG (`None` transparent) and its world file (`.pgw`).
pub fn write_gray_png(r: &Raster<Option<u8>>, out_png: &Path) -> Result<(), DeriveError> {
    let pixels: Vec<u8> = r
        .data
        .iter()
        .flat_map(|v| match v {
            Some(g) => [*g, 255],
            None => [0, 0],
        })
        .collect();
    let image = image::GrayAlphaImage::from_raw(r.grid.width, r.grid.height, pixels)
        .expect("buffer matches the grid");
    if let Some(dir) = out_png.parent() {
        std::fs::create_dir_all(dir).map_err(|e| DeriveError::Io(dir.to_path_buf(), e))?;
    }
    image
        .save_with_format(out_png, image::ImageFormat::Png)
        .map_err(|e| DeriveError::Image(e.to_string()))?;
    let world_path = out_png.with_extension("pgw");
    std::fs::write(&world_path, r.grid.world_file()).map_err(|e| DeriveError::Io(world_path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{dtm_tiff, tile_zip};
    use crate::sources::lgb::{Product, TileCurrency};
    use crate::tiles::TileId;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    /// 5 × 5 DTM with 1 m pixels and heights from `z(e, n)` (metres relative to the north-west corner).
    fn dtm(z: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = GridSpec {
            west: 0.0,
            north: 5.0,
            pixel_size: 1.0,
            width: 5,
            height: 5,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..5 {
            for x in 0..5 {
                r.set(x, y, z(f64::from(x), -f64::from(y)));
            }
        }
        r
    }

    fn all(r: &Raster<Option<u8>>) -> Vec<Option<u8>> {
        r.data.clone()
    }

    #[test]
    fn flat_hillshade_is_180() {
        let flat = dtm(|_, _| 40.0);
        assert!(all(&hillshade(&flat)).iter().all(|v| *v == Some(180)));
        assert!(slope_deg(&flat).data.iter().all(|v| *v == Some(0.0)));
    }

    #[test]
    fn east_rising_plane() {
        let plane = dtm(|e, _| e as f32);
        assert!(
            all(&hillshade(&plane)).iter().all(|v| *v == Some(218)),
            "{:?}",
            all(&hillshade(&plane))
        );
        for s in slope_deg(&plane).data {
            assert!((s.unwrap() - 45.0).abs() < 0.01, "{s:?}");
        }
    }

    #[test]
    fn west_rising_plane() {
        let plane = dtm(|e, _| -e as f32);
        assert!(all(&hillshade(&plane)).iter().all(|v| *v == Some(37)));
    }

    #[test]
    fn north_rising_plane() {
        let plane = dtm(|_, n| n as f32);
        assert!(all(&hillshade(&plane)).iter().all(|v| *v == Some(37)));
    }

    #[test]
    fn nodata_neighbourhood_is_transparent() {
        let mut holed = dtm(|_, _| 40.0);
        holed.set(2, 2, f32::NAN);
        let shade = hillshade(&holed);
        for y in 0..5 {
            for x in 0..5 {
                let near_hole = (1..=3).contains(&x) && (1..=3).contains(&y);
                assert_eq!(shade.get(x, y).is_none(), near_hole, "({x}, {y})");
            }
        }
        assert_eq!(slope_deg(&holed).get(0, 0), Some(0.0));
        assert_eq!(slope_deg(&holed).get(2, 1), None);
    }

    #[test]
    fn slope_shade_ramp() {
        let mut slope = Raster::filled(
            GridSpec {
                west: 0.0,
                north: 1.0,
                pixel_size: 1.0,
                width: 4,
                height: 1,
            },
            Some(0.0f32),
        );
        slope.set(1, 0, Some(15.0));
        slope.set(2, 0, Some(45.0));
        slope.set(3, 0, None);
        assert_eq!(
            slope_shade(&slope).data,
            vec![Some(255), Some(127), Some(0), None]
        );
    }

    #[test]
    fn dtm_from_synthetic_tile() {
        let dir = tempfile::tempdir().unwrap();
        let mut values: Vec<f32> = (0..16).map(|v| v as f32).collect();
        values[5] = -9999.0;
        let zip = dir.path().join("dgm.zip");
        let world = "1\n0\n0\n-1\n421000.5\n5821999.5\n";
        std::fs::write(
            &zip,
            tile_zip(Product::Dtm, TILE, &dtm_tiff(4, 4, &values), world),
        )
        .unwrap();
        let date = "2026-01-26".parse().unwrap();
        let tile = FetchedTile {
            product: Product::Dtm,
            tile: TILE,
            url: String::new(),
            currency: TileCurrency {
                created: date,
                revised: None,
                published: date,
            },
            zip,
        };
        let bbox = BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_996.0,
            max_e: 421_004.0,
            max_n: 5_822_000.0,
        };
        let mosaic = mosaic_dtm(&[tile], &bbox).unwrap();
        assert_eq!((mosaic.grid.width, mosaic.grid.height), (4, 4));
        assert_eq!(mosaic.get(0, 0), 0.0);
        assert_eq!(mosaic.get(3, 3), 15.0);
        assert!(mosaic.get(1, 1).is_nan());
    }

    #[test]
    fn gray_png_has_alpha_and_world_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut r = Raster::filled(
            GridSpec {
                west: 421_000.0,
                north: 5_822_000.0,
                pixel_size: 1.0,
                width: 2,
                height: 1,
            },
            Some(10u8),
        );
        r.set(1, 0, None);
        let out = dir.path().join("hillshade.png");
        write_gray_png(&r, &out).unwrap();
        let img = image::open(&out).unwrap().to_luma_alpha8();
        assert_eq!(img.get_pixel(0, 0).0, [10, 255]);
        assert_eq!(img.get_pixel(1, 0).0[1], 0);
        let world = std::fs::read_to_string(dir.path().join("hillshade.pgw")).unwrap();
        assert_eq!(
            GridSpec::from_world_file(&world, 2, 1).unwrap().west,
            421_000.0
        );
    }
}
