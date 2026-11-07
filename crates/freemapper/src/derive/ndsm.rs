//! nDSM template: LGB nDOM point clouds rasterized to heights above ground, where none is
//! published the bDOM surface model minus the DGM1, as a coloured PNG.

use std::path::{Path, PathBuf};

use super::{DeriveError, template_grid, write_image_with_world_file};
use crate::crs::BoundingBox;
use crate::raster::Raster;

/// Grid of the LGB nDOM: one point per 0.2 m cell.
const NDSM_PIXEL_M: f64 = 0.2;
/// Points read per chunk; a tile holds 25 million.
const CHUNK_POINTS: u64 = 1_000_000;

/// Heights above ground over `bbox`: the highest point per 0.2 m cell, `NaN` where no point lies.
pub fn rasterize(tiles: &[PathBuf], bbox: &BoundingBox) -> Result<Raster<f32>, DeriveError> {
    let grid = template_grid(bbox, NDSM_PIXEL_M)?;
    let mut raster = Raster::filled(grid, f32::NAN);
    let las_error =
        |path: &Path, e: las::Error| DeriveError::Image(format!("{}: {e}", path.display()));
    for path in tiles {
        let mut reader = las::Reader::from_path(path).map_err(|e| las_error(path, e))?;
        let mut points = las::PointDataBuilder::new()
            .for_header(reader.header())
            .build();
        while reader
            .fill_points(CHUNK_POINTS, &mut points)
            .map_err(|e| las_error(path, e))?
            > 0
        {
            for point in points.points() {
                let point = point.map_err(|e| las_error(path, e))?;
                let col = ((point.x - grid.west) / grid.pixel_size).floor();
                let row = ((grid.north - point.y) / grid.pixel_size).floor();
                if col < 0.0
                    || row < 0.0
                    || col >= f64::from(grid.width)
                    || row >= f64::from(grid.height)
                {
                    continue;
                }
                let (x, y) = (col as u32, row as u32);
                let z = point.z as f32;
                let current = raster.get(x, y);
                if current.is_nan() || z > current {
                    raster.set(x, y, z);
                }
            }
        }
    }
    Ok(raster)
}

/// Colour of a height above ground: transparent below 0.3 m, then low features, hedges and
/// walls, bushes and sheds, trees and buildings.
pub fn colour(height: f32) -> [u8; 4] {
    match height {
        h if h.is_nan() || h < 0.3 => [0, 0, 0, 0],
        h if h < 1.0 => [0xf2, 0xd1, 0x4b, 255],
        h if h < 2.5 => [0xf0, 0x8a, 0x24, 255],
        h if h < 8.0 => [0x4a, 0xa3, 0x5a, 255],
        _ => [0x1f, 0x6b, 0x3a, 255],
    }
}

/// Fills the empty cells of `ndsm` with the surface model minus the terrain (bilinear), clamped
/// at 0; cells where either is missing stay empty.
pub fn fill_from_dsm(ndsm: &mut Raster<f32>, dsm: &Raster<f32>, dtm: &Raster<f32>) {
    let grid = ndsm.grid;
    for y in 0..grid.height {
        let n = grid.north - (f64::from(y) + 0.5) * grid.pixel_size;
        for x in 0..grid.width {
            if !ndsm.get(x, y).is_nan() {
                continue;
            }
            let e = grid.west + (f64::from(x) + 0.5) * grid.pixel_size;
            let (Some(surface), Some(ground)) = (nearest(dsm, e, n), bilinear(dtm, e, n)) else {
                continue;
            };
            ndsm.set(x, y, (surface - ground).max(0.0));
        }
    }
}

/// The value of the cell containing (e, n), if it has one.
fn nearest(r: &Raster<f32>, e: f64, n: f64) -> Option<f32> {
    let col = ((e - r.grid.west) / r.grid.pixel_size).floor();
    let row = ((r.grid.north - n) / r.grid.pixel_size).floor();
    let inside =
        col >= 0.0 && row >= 0.0 && col < f64::from(r.grid.width) && row < f64::from(r.grid.height);
    inside
        .then(|| r.get(col as u32, row as u32))
        .filter(|v| !v.is_nan())
}

/// Bilinear interpolation between the four cell centres around (e, n); `None` if one is missing.
fn bilinear(r: &Raster<f32>, e: f64, n: f64) -> Option<f32> {
    let fx = (e - r.grid.west) / r.grid.pixel_size - 0.5;
    let fy = (r.grid.north - n) / r.grid.pixel_size - 0.5;
    let (x0, y0) = (fx.floor(), fy.floor());
    if x0 < 0.0
        || y0 < 0.0
        || x0 + 1.0 >= f64::from(r.grid.width)
        || y0 + 1.0 >= f64::from(r.grid.height)
    {
        return None;
    }
    let (tx, ty) = (fx - x0, fy - y0);
    let (x0, y0) = (x0 as u32, y0 as u32);
    let z = |x: u32, y: u32| Some(f64::from(r.get(x, y))).filter(|v| !v.is_nan());
    let top = z(x0, y0)? * (1.0 - tx) + z(x0 + 1, y0)? * tx;
    let bottom = z(x0, y0 + 1)? * (1.0 - tx) + z(x0 + 1, y0 + 1)? * tx;
    Some((top * (1.0 - ty) + bottom * ty) as f32)
}

/// Writes the coloured nDSM as RGBA PNG with its world file (`.pgw`).
pub fn write_ndsm_png(r: &Raster<f32>, out: &Path) -> Result<(), DeriveError> {
    let pixels: Vec<u8> = r.data.iter().flat_map(|h| colour(*h)).collect();
    let image = image::RgbaImage::from_raw(r.grid.width, r.grid.height, pixels)
        .expect("buffer matches the grid");
    let mut png = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| DeriveError::Image(e.to_string()))?;
    write_image_with_world_file(out, png.get_ref(), "pgw", &r.grid.world_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A LAZ file with the given points (x, y, z in EPSG:25833 / metres above ground).
    fn laz(dir: &Path, name: &str, points: &[(f64, f64, f64)]) -> PathBuf {
        let path = dir.join(name);
        // Scale and offsets as in the LGB files (0.01 m, offset 400 000 / 5 800 000).
        let mut builder = las::Builder::from((1, 2));
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(400_000.0),
            y: transform(5_800_000.0),
            z: transform(0.0),
        };
        let header = builder.into_header().unwrap();
        let mut writer = las::Writer::from_path(&path, header).unwrap();
        for &(x, y, z) in points {
            writer
                .write_point(las::Point {
                    x,
                    y,
                    z,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        path
    }

    fn bbox() -> BoundingBox {
        BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_000.0,
            max_e: 421_001.0,
            max_n: 5_821_001.0,
        }
    }

    #[test]
    fn rasterize_keeps_max_height() {
        let dir = tempfile::tempdir().unwrap();
        let tile = laz(
            dir.path(),
            "a.laz",
            &[
                (421_000.1, 5_821_000.9, 1.0),
                (421_000.15, 5_821_000.85, 3.0),
            ],
        );
        let r = rasterize(&[tile], &bbox()).unwrap();
        assert_eq!((r.grid.width, r.grid.height), (5, 5));
        assert!((r.get(0, 0) - 3.0).abs() < 0.01, "{}", r.get(0, 0));
        assert!(r.get(4, 4).is_nan());
    }

    #[test]
    fn points_outside_bbox_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let tile = laz(
            dir.path(),
            "a.laz",
            &[(421_005.1, 5_821_000.5, 9.0), (420_999.9, 5_821_000.5, 9.0)],
        );
        let r = rasterize(&[tile], &bbox()).unwrap();
        assert!(r.data.iter().all(|z| z.is_nan()));
    }

    #[test]
    fn two_tiles_are_seamless() {
        let dir = tempfile::tempdir().unwrap();
        let west = laz(dir.path(), "w.laz", &[(421_000.9, 5_821_000.5, 2.0)]);
        let east = laz(dir.path(), "e.laz", &[(421_001.1, 5_821_000.5, 4.0)]);
        let wide = BoundingBox {
            max_e: 421_002.0,
            ..bbox()
        };
        let r = rasterize(&[west, east], &wide).unwrap();
        assert!((r.get(4, 2) - 2.0).abs() < 0.01);
        assert!((r.get(5, 2) - 4.0).abs() < 0.01);
    }

    #[test]
    fn colour_ramp() {
        assert_eq!(colour(f32::NAN)[3], 0);
        assert_eq!(colour(0.1)[3], 0);
        assert_eq!(colour(0.5), [0xf2, 0xd1, 0x4b, 255]);
        assert_eq!(colour(1.5), [0xf0, 0x8a, 0x24, 255]);
        assert_eq!(colour(5.0), [0x4a, 0xa3, 0x5a, 255]);
        assert_eq!(colour(20.0), [0x1f, 0x6b, 0x3a, 255]);
    }

    #[test]
    fn png_has_world_file() {
        let dir = tempfile::tempdir().unwrap();
        let tile = laz(dir.path(), "a.laz", &[(421_000.1, 5_821_000.9, 1.5)]);
        let r = rasterize(&[tile], &bbox()).unwrap();
        let out = dir.path().join("ndsm.png");
        write_ndsm_png(&r, &out).unwrap();
        let img = image::open(&out).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [0xf0, 0x8a, 0x24, 255]);
        assert_eq!(img.get_pixel(4, 4).0[3], 0);
        let world = std::fs::read_to_string(dir.path().join("ndsm.pgw")).unwrap();
        let grid = crate::raster::GridSpec::from_world_file(&world, 5, 5).unwrap();
        assert!((grid.west - 421_000.0).abs() < 1e-6);
    }

    /// A raster over `bbox` with `pixel` metres per cell and the value `f(e, n)` at each centre.
    fn surface(bbox: &BoundingBox, pixel: f64, f: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = template_grid(bbox, pixel).unwrap();
        let mut r = Raster::filled(grid, f32::NAN);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let e = grid.west + (f64::from(x) + 0.5) * pixel;
                let n = grid.north - (f64::from(y) + 0.5) * pixel;
                r.set(x, y, f(e, n));
            }
        }
        r
    }

    const AREA: BoundingBox = BoundingBox {
        min_e: 392_800.0,
        min_n: 5_820_800.0,
        max_e: 392_810.0,
        max_n: 5_820_810.0,
    };

    #[test]
    fn ndsm_from_bdom_minus_dtm() {
        // Ground rises 1 m per metre east from 30 m; the surface is flat at 38 m.
        let dtm = surface(&AREA.buffered(2.0), 1.0, |e, _| {
            (30.0 + (e - 392_800.0)) as f32
        });
        let dsm = surface(&AREA, 0.2, |_, _| 38.0);
        let mut ndsm = rasterize(&[], &AREA).unwrap();
        fill_from_dsm(&mut ndsm, &dsm, &dtm);
        // Cell centre at e = 392 800.1: ground 30.1 m (bilinear between 1 m centres).
        assert!((ndsm.get(0, 0) - 7.9).abs() < 1e-3, "{}", ndsm.get(0, 0));
        // e = 392 805.1: ground 35.1 m.
        assert!(
            (ndsm.get(25, 10) - 2.9).abs() < 1e-3,
            "{}",
            ndsm.get(25, 10)
        );
        // e = 392 809.9: ground 39.9 m above the surface; clamped at 0.
        assert_eq!(ndsm.get(49, 0), 0.0);
    }

    #[test]
    fn laz_wins_over_bdom() {
        let dir = tempfile::tempdir().unwrap();
        let tile = laz(dir.path(), "a.laz", &[(392_800.1, 5_820_809.9, 2.0)]);
        let mut ndsm = rasterize(&[tile], &AREA).unwrap();
        let dtm = surface(&AREA.buffered(2.0), 1.0, |_, _| 30.0);
        let dsm = surface(&AREA, 0.2, |_, _| 45.0);
        fill_from_dsm(&mut ndsm, &dsm, &dtm);
        assert_eq!(ndsm.get(0, 0), 2.0);
        assert_eq!(ndsm.get(1, 0), 15.0);
    }

    #[test]
    fn missing_surface_or_ground_stays_empty() {
        let mut ndsm = rasterize(&[], &AREA).unwrap();
        let dtm = surface(&AREA.buffered(2.0), 1.0, |e, _| {
            if e < 392_805.0 { 30.0 } else { f32::NAN }
        });
        let dsm = surface(
            &AREA,
            0.2,
            |_, n| if n > 5_820_805.0 { 45.0 } else { f32::NAN },
        );
        fill_from_dsm(&mut ndsm, &dsm, &dtm);
        assert_eq!(ndsm.get(0, 0), 15.0);
        assert!(ndsm.get(0, 49).is_nan(), "no surface");
        assert!(ndsm.get(49, 0).is_nan(), "no ground");
    }
}
