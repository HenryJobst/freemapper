//! Orthophoto template: DOP20 tiles (or one WMS image) mosaicked and clipped to the map area.

use std::path::Path;

use super::{DeriveError, read_member, template_grid, write_image_with_world_file};
use crate::crs::BoundingBox;
use crate::fetch::FetchedTile;
use crate::raster::{GridSpec, Raster};

/// JPEG quality of the orthophoto template.
const JPEG_QUALITY: u8 = 90;
/// Background where no tile covers the map area.
const WHITE: [u8; 3] = [255, 255, 255];

/// The orthophoto template over `bbox` from one georeferenced JPEG (`.jpg` + `.jgw`), such as
/// the stitched GeoSN WMS image.
pub fn build_orthophoto_from_image(
    jpg: &Path,
    bbox: &BoundingBox,
    root: &Path,
    out_jpg: &Path,
) -> Result<(), DeriveError> {
    let read =
        |path: &Path| std::fs::read(path).map_err(|e| DeriveError::Io(path.to_path_buf(), e));
    let jpeg = read(jpg)?;
    let world = read(&jpg.with_extension("jgw"))?;
    // Fetched on the template grid already (the GeoSN WMS image): copied, not decoded and
    // encoded again.
    let (width, height) =
        image::ImageReader::with_format(std::io::Cursor::new(&jpeg), image::ImageFormat::Jpeg)
            .into_dimensions()
            .map_err(|e| DeriveError::Image(e.to_string()))?;
    let grid = GridSpec::from_world_file(&String::from_utf8_lossy(&world), width, height)?;
    if template_grid(bbox, grid.pixel_size)? == grid {
        return write_image_with_world_file(root, out_jpg, &jpeg, "jgw", &grid.world_file());
    }
    write_mosaic(std::iter::once(decode(&jpeg, &world)), bbox, root, out_jpg)
}

/// Mosaics the orthophoto tiles over `bbox` and writes `out_jpg` plus its world file (`.jgw`).
pub fn build_orthophoto(
    tiles: &[FetchedTile],
    bbox: &BoundingBox,
    root: &Path,
    out_jpg: &Path,
) -> Result<(), DeriveError> {
    let images = tiles.iter().map(|tile| {
        let jpeg = read_member(&tile.zip, &tile.product.raster_member(tile.tile))?;
        let world = read_member(&tile.zip, &tile.product.world_file_member(tile.tile))?;
        decode(&jpeg, &world)
    });
    write_mosaic(images, bbox, root, out_jpg)
}

/// A JPEG and the grid its world file gives it.
fn decode(jpeg: &[u8], world: &[u8]) -> Result<(image::RgbImage, GridSpec), DeriveError> {
    let image = image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg)
        .map_err(|e| DeriveError::Image(e.to_string()))?
        .into_rgb8();
    let grid = GridSpec::from_world_file(
        &String::from_utf8_lossy(world),
        image.width(),
        image.height(),
    )?;
    Ok((image, grid))
}

/// Pastes the images into the template grid over `bbox` (white where none lies) and writes it.
fn write_mosaic(
    images: impl Iterator<Item = Result<(image::RgbImage, GridSpec), DeriveError>>,
    bbox: &BoundingBox,
    root: &Path,
    out_jpg: &Path,
) -> Result<(), DeriveError> {
    let mut mosaic: Option<Raster<[u8; 3]>> = None;
    for image in images {
        let (image, grid) = image?;
        let mosaic = match &mut mosaic {
            Some(m) => m,
            None => mosaic.insert(Raster::filled(template_grid(bbox, grid.pixel_size)?, WHITE)),
        };
        mosaic.paste_with(&grid, |x, y| image.get_pixel(x, y).0)?;
    }
    let mosaic = mosaic.ok_or(DeriveError::NoTiles)?;

    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
        .encode(
            mosaic.data.as_flattened(),
            mosaic.grid.width,
            mosaic.grid.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| DeriveError::Image(e.to_string()))?;
    write_image_with_world_file(root, out_jpg, &jpeg, "jgw", &mosaic.grid.world_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{red_jpeg, solid_jpeg, tile_zip};
    use crate::sources::lgb::{Product, TileCurrency};
    use crate::tiles::TileId;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    #[test]
    fn orthophoto_from_synthetic_tile() {
        let dir = tempfile::tempdir().unwrap();
        let zip = dir.path().join("tile.zip");
        let world = "1\n0\n0\n-1\n421000.5\n5821999.5\n";
        std::fs::write(
            &zip,
            tile_zip(Product::Orthophoto, TILE, &red_jpeg(50, 50), world),
        )
        .unwrap();
        let date = "2026-10-09".parse().unwrap();
        let tile = FetchedTile {
            product: Product::Orthophoto,
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
            min_e: 421_010.0,
            min_n: 5_821_960.0,
            max_e: 421_030.0,
            max_n: 5_821_990.0,
        };
        let out = dir.path().join("orthophoto.jpg");
        build_orthophoto(&[tile], &bbox, out.parent().unwrap(), &out).unwrap();

        let img = image::open(&out).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (20, 30));
        let centre = img.get_pixel(10, 15).0;
        assert!(
            centre[0] > 245 && centre[1] < 10 && centre[2] < 10,
            "{centre:?}"
        );
        let world = std::fs::read_to_string(dir.path().join("orthophoto.jgw")).unwrap();
        let grid = GridSpec::from_world_file(&world, 20, 30).unwrap();
        assert!(grid.bounds().intersection(&bbox) == Some(bbox), "{grid:?}");
    }

    #[test]
    fn wms_image_becomes_the_template() {
        let dir = tempfile::tempdir().unwrap();
        let wms = dir.path().join("orthophoto-wms.jpg");
        std::fs::write(&wms, red_jpeg(50, 50)).unwrap();
        std::fs::write(
            wms.with_extension("jgw"),
            "1\n0\n0\n-1\n421000.5\n5821999.5\n",
        )
        .unwrap();
        let bbox = BoundingBox {
            min_e: 421_010.0,
            min_n: 5_821_960.0,
            max_e: 421_030.0,
            max_n: 5_821_990.0,
        };
        let out = dir.path().join("orthophoto.jpg");
        build_orthophoto_from_image(&wms, &bbox, out.parent().unwrap(), &out).unwrap();
        let img = image::open(&out).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (20, 30));
        assert!(img.get_pixel(10, 15).0[0] > 245);
        assert!(dir.path().join("orthophoto.jgw").exists());
    }

    #[test]
    fn wms_image_on_the_template_grid_is_copied() {
        let dir = tempfile::tempdir().unwrap();
        let bbox = BoundingBox {
            min_e: 421_010.0,
            min_n: 5_821_960.0,
            max_e: 421_030.0,
            max_n: 5_821_990.0,
        };
        let grid = GridSpec::covering(&bbox, 1.0);
        let wms = dir.path().join("orthophoto-wms.jpg");
        let jpeg = red_jpeg(grid.width, grid.height);
        std::fs::write(&wms, &jpeg).unwrap();
        std::fs::write(wms.with_extension("jgw"), grid.world_file()).unwrap();
        let out = dir.path().join("orthophoto.jpg");
        build_orthophoto_from_image(&wms, &bbox, out.parent().unwrap(), &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), jpeg, "copied, not re-encoded");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("orthophoto.jgw")).unwrap(),
            grid.world_file()
        );
    }

    fn fetched(
        dir: &std::path::Path,
        tile: TileId,
        jpeg: Vec<u8>,
        pixel: f64,
        size: u32,
    ) -> FetchedTile {
        let b = tile.bounds();
        let world = format!(
            "{pixel}\n0\n0\n-{pixel}\n{}\n{}\n",
            b.min_e + pixel / 2.0,
            b.max_n - pixel / 2.0
        );
        let zip = dir.join(format!("{}.zip", tile.name()));
        std::fs::write(&zip, tile_zip(Product::Orthophoto, tile, &jpeg, &world)).unwrap();
        let date = "2026-10-09".parse().unwrap();
        assert_eq!(f64::from(size) * pixel, 1000.0);
        FetchedTile {
            product: Product::Orthophoto,
            tile,
            url: String::new(),
            currency: TileCurrency {
                created: date,
                revised: None,
                published: date,
            },
            zip,
        }
    }

    #[test]
    fn cross_tile_orthophoto_is_seamless() {
        let dir = tempfile::tempdir().unwrap();
        let east = TileId {
            east_km: 422,
            north_km: 5821,
        };
        let tiles = [
            fetched(
                dir.path(),
                TILE,
                solid_jpeg(100, 100, [255, 0, 0]),
                10.0,
                100,
            ),
            fetched(
                dir.path(),
                east,
                solid_jpeg(100, 100, [0, 0, 255]),
                10.0,
                100,
            ),
        ];
        let bbox = BoundingBox {
            min_e: 421_900.0,
            min_n: 5_821_400.0,
            max_e: 422_100.0,
            max_n: 5_821_600.0,
        };
        let out = dir.path().join("orthophoto.jpg");
        build_orthophoto(&tiles, &bbox, out.parent().unwrap(), &out).unwrap();
        let img = image::open(&out).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (20, 20));
        for x in 0..20 {
            let p = img.get_pixel(x, 10).0;
            if x < 10 {
                assert!(p[0] > 200 && p[2] < 60, "column {x}: {p:?}");
            } else {
                assert!(p[2] > 200 && p[0] < 60, "column {x}: {p:?}");
            }
        }
    }

    #[test]
    fn oversized_area_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let tile = fetched(
            dir.path(),
            TILE,
            solid_jpeg(100, 100, [255, 0, 0]),
            10.0,
            100,
        );
        let bbox = BoundingBox {
            min_e: 200_000.0,
            min_n: 5_700_000.0,
            max_e: 500_000.0,
            max_n: 5_800_000.0,
        };
        let err =
            build_orthophoto(&[tile], &bbox, dir.path(), &dir.path().join("o.jpg")).unwrap_err();
        assert!(matches!(err, DeriveError::AreaTooLarge { .. }), "{err:?}");
        assert!(err.to_string().contains("too large"), "{err}");
    }

    #[test]
    fn no_tiles_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let bbox = BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 1.0,
            max_n: 1.0,
        };
        let err = build_orthophoto(&[], &bbox, dir.path(), &dir.path().join("o.jpg")).unwrap_err();
        assert!(matches!(err, DeriveError::NoTiles), "{err:?}");
    }
}
