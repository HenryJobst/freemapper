//! Builders of test data for the readers: images and height rasters as the services send
//! them. Behind the feature `testing`.

/// A uniformly red JPEG.
pub fn red_jpeg(width: u32, height: u32) -> Vec<u8> {
    solid_jpeg(width, height, [255, 0, 0])
}

/// A JPEG of one colour.
pub fn solid_jpeg(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 95)
        .encode_image(&img)
        .unwrap();
    bytes
}

/// A 32-bit float LZW GeoTIFF body like DGM1 (without GeoKeys; the world file locates it).
pub fn dtm_tiff(width: u32, height: u32, values: &[f32]) -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    tiff::encoder::TiffEncoder::new(&mut bytes)
        .unwrap()
        .with_compression(tiff::encoder::Compression::Lzw)
        .write_image::<tiff::encoder::colortype::Gray32Float>(width, height, values)
        .unwrap();
    bytes.into_inner()
}

/// A WCS `multipart/related` answer like LVermGeo's: a GML part, then a Float32 TIFF on
/// `grid` with ModelPixelScale and ModelTiepoint tags.
pub fn wcs_answer(grid: &crate::raster::GridSpec, values: &[f32]) -> Vec<u8> {
    let mut answer =
        b"--wcs\nContent-Type: text/xml\nContent-ID: GML-Part\n\n<coverage/>\n--wcs\nContent-Type: image/tiff\nContent-ID: 1.tif\n\n"
            .to_vec();
    answer.extend_from_slice(&geotiff(grid, values));
    answer.extend_from_slice(b"\n--wcs--\n");
    answer
}

/// A Float32 GeoTIFF on `grid` with ModelPixelScale and ModelTiepoint tags (no world file),
/// like the LVermGeo WCS answers and the LGV height model tiles.
pub fn geotiff(grid: &crate::raster::GridSpec, values: &[f32]) -> Vec<u8> {
    use tiff::tags::Tag;
    let mut tiff = std::io::Cursor::new(Vec::new());
    {
        let mut encoder = tiff::encoder::TiffEncoder::new(&mut tiff).unwrap();
        let mut image = encoder
            .new_image::<tiff::encoder::colortype::Gray32Float>(grid.width, grid.height)
            .unwrap();
        image
            .encoder()
            .write_tag(
                Tag::ModelPixelScaleTag,
                &[grid.pixel_size, grid.pixel_size, 0.0][..],
            )
            .unwrap();
        image
            .encoder()
            .write_tag(
                Tag::ModelTiepointTag,
                &[0.0, 0.0, 0.0, grid.west, grid.north, 0.0][..],
            )
            .unwrap();
        image.write_data(values).unwrap();
    }
    tiff.into_inner()
}
