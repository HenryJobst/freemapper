//! Orthophoto template: DOP20 tiles mosaicked and clipped to the map area.

use std::path::Path;

use super::{DeriveError, read_member};
use crate::crs::BoundingBox;
use crate::fetch::FetchedTile;
use crate::raster::{GridSpec, Raster};

/// JPEG quality of the orthophoto template.
const JPEG_QUALITY: u8 = 90;
/// Background where no tile covers the map area.
const WHITE: [u8; 3] = [255, 255, 255];

/// Mosaics the orthophoto tiles over `bbox` and writes `out_jpg` plus its world file (`.jgw`).
pub fn build_orthophoto(
    tiles: &[FetchedTile],
    bbox: &BoundingBox,
    out_jpg: &Path,
) -> Result<(), DeriveError> {
    let mut mosaic: Option<Raster<[u8; 3]>> = None;
    for tile in tiles {
        let jpeg = read_member(&tile.zip, &tile.product.raster_member(tile.tile))?;
        let world = read_member(&tile.zip, &tile.product.world_file_member(tile.tile))?;
        let image = image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
            .map_err(|e| DeriveError::Image(e.to_string()))?
            .into_rgb8();
        let grid = GridSpec::from_world_file(
            &String::from_utf8_lossy(&world),
            image.width(),
            image.height(),
        )?;
        let mosaic = mosaic.get_or_insert_with(|| {
            Raster::filled(GridSpec::covering(bbox, grid.pixel_size), WHITE)
        });
        mosaic.paste_with(&grid, |x, y| image.get_pixel(x, y).0);
    }
    let mosaic = mosaic.ok_or(DeriveError::NoTiles)?;

    if let Some(dir) = out_jpg.parent() {
        std::fs::create_dir_all(dir).map_err(|e| DeriveError::Io(dir.to_path_buf(), e))?;
    }
    let file =
        std::fs::File::create(out_jpg).map_err(|e| DeriveError::Io(out_jpg.to_path_buf(), e))?;
    let mut writer = std::io::BufWriter::new(file);
    let pixels: Vec<u8> = mosaic.data.iter().flatten().copied().collect();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY)
        .encode(
            &pixels,
            mosaic.grid.width,
            mosaic.grid.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| DeriveError::Image(e.to_string()))?;
    let world_path = out_jpg.with_extension("jgw");
    std::fs::write(&world_path, mosaic.grid.world_file())
        .map_err(|e| DeriveError::Io(world_path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::tile_zip;
    use crate::sources::lgb::{Product, TileCurrency};
    use crate::tiles::TileId;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    fn red_jpeg(size: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(size, size, image::Rgb([255, 0, 0]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95)
            .encode_image(&img)
            .unwrap();
        bytes
    }

    #[test]
    fn orthophoto_from_synthetic_tile() {
        let dir = tempfile::tempdir().unwrap();
        let zip = dir.path().join("tile.zip");
        let world = "1\n0\n0\n-1\n421000.5\n5821999.5\n";
        std::fs::write(
            &zip,
            tile_zip(Product::Orthophoto, TILE, &red_jpeg(50), world),
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
        build_orthophoto(&[tile], &bbox, &out).unwrap();

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
    fn no_tiles_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let bbox = BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 1.0,
            max_n: 1.0,
        };
        let err = build_orthophoto(&[], &bbox, &dir.path().join("o.jpg")).unwrap_err();
        assert!(matches!(err, DeriveError::NoTiles), "{err:?}");
    }
}
