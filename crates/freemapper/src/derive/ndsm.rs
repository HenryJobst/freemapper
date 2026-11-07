//! nDSM template: LGB nDOM point clouds rasterized to heights above ground, as a coloured PNG.

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
}
