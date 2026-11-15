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

/// A Float32 GeoTIFF on `grid` like [`geotiff`], LZW-compressed and with the GDAL metadata
/// `metadata` (the XML of tag 42112), like the DOM20 tiles of the LDBV.
pub fn geotiff_with_metadata(
    grid: &crate::raster::GridSpec,
    values: &[f32],
    metadata: &str,
) -> Vec<u8> {
    use tiff::tags::Tag;
    let mut tiff = std::io::Cursor::new(Vec::new());
    {
        let mut encoder = tiff::encoder::TiffEncoder::new(&mut tiff)
            .unwrap()
            .with_compression(tiff::encoder::Compression::Lzw);
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
        image
            .encoder()
            .write_tag(Tag::Unknown(42112), metadata)
            .unwrap();
        image.write_data(values).unwrap();
    }
    tiff.into_inner()
}

/// A tiled Float32 TIFF as the LDBV writes its DOM20: little endian, LZW, `predictor` 1 or 3,
/// blocks of `block` × `block` px stored in row order, and the IFD with its arrays at the end of
/// the file. `metadata` goes into the GDAL metadata tag (42112) if given.
pub fn tiled_float_tiff(
    grid: &crate::raster::GridSpec,
    values: &[f32],
    block: u32,
    predictor: u16,
    metadata: Option<&str>,
) -> Vec<u8> {
    assert_eq!(values.len(), (grid.width * grid.height) as usize);
    let (across, down) = (grid.width.div_ceil(block), grid.height.div_ceil(block));
    let mut file = b"II*\0\0\0\0\0".to_vec();
    let (mut offsets, mut counts) = (Vec::new(), Vec::new());
    for by in 0..down {
        for bx in 0..across {
            let mut raw = Vec::with_capacity((block * block * 4) as usize);
            for y in 0..block {
                let mut row = Vec::with_capacity((block * 4) as usize);
                for x in 0..block {
                    let (px, py) = (bx * block + x, by * block + y);
                    let z = if px < grid.width && py < grid.height {
                        values[(py * grid.width + px) as usize]
                    } else {
                        -9999.0
                    };
                    row.push(z);
                }
                match predictor {
                    1 => row.iter().for_each(|z| raw.extend(z.to_le_bytes())),
                    3 => {
                        // The bytes of each sample in planes, most significant first, then
                        // differenced over the row.
                        let n = row.len();
                        let mut planes = vec![0u8; n * 4];
                        for (i, z) in row.iter().enumerate() {
                            for (plane, byte) in z.to_be_bytes().into_iter().enumerate() {
                                planes[plane * n + i] = byte;
                            }
                        }
                        for i in (1..planes.len()).rev() {
                            planes[i] = planes[i].wrapping_sub(planes[i - 1]);
                        }
                        raw.extend(planes);
                    }
                    other => panic!("predictor {other}"),
                }
            }
            let compressed = weezl::encode::Encoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8)
                .encode(&raw)
                .unwrap();
            offsets.push(file.len() as u32);
            counts.push(compressed.len() as u32);
            file.extend(compressed);
        }
    }
    if file.len() % 2 == 1 {
        file.push(0);
    }
    let ifd = file.len() as u32;
    file[4..8].copy_from_slice(&ifd.to_le_bytes());
    // (tag, type, count, inline value or out-of-line bytes)
    enum Value {
        Inline(u32),
        Out(Vec<u8>),
    }
    let longs = |v: &[u32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let doubles = |v: &[f64]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let mut entries: Vec<(u16, u16, u32, Value)> = vec![
        (256, 4, 1, Value::Inline(grid.width)),
        (257, 4, 1, Value::Inline(grid.height)),
        (258, 3, 1, Value::Inline(32)),
        (259, 3, 1, Value::Inline(5)),
        (262, 3, 1, Value::Inline(1)),
        (277, 3, 1, Value::Inline(1)),
        (284, 3, 1, Value::Inline(1)),
        (317, 3, 1, Value::Inline(u32::from(predictor))),
        (322, 3, 1, Value::Inline(block)),
        (323, 3, 1, Value::Inline(block)),
        (324, 4, offsets.len() as u32, Value::Out(longs(&offsets))),
        (325, 4, counts.len() as u32, Value::Out(longs(&counts))),
        (339, 3, 1, Value::Inline(3)),
        (
            33550,
            12,
            3,
            Value::Out(doubles(&[grid.pixel_size, grid.pixel_size, 0.0])),
        ),
        (
            33922,
            12,
            6,
            Value::Out(doubles(&[0.0, 0.0, 0.0, grid.west, grid.north, 0.0])),
        ),
    ];
    if let Some(metadata) = metadata {
        let mut text = metadata.as_bytes().to_vec();
        text.push(0);
        entries.push((42112, 2, text.len() as u32, Value::Out(text)));
    }
    // A single offset or count fits in the entry itself.
    for entry in &mut entries {
        if let Value::Out(bytes) = &entry.3
            && bytes.len() <= 4
        {
            let mut word = [0u8; 4];
            word[..bytes.len()].copy_from_slice(bytes);
            entry.3 = Value::Inline(u32::from_le_bytes(word));
        }
    }
    let mut next = ifd + 2 + 12 * entries.len() as u32 + 4;
    let mut ifd_bytes = (entries.len() as u16).to_le_bytes().to_vec();
    let mut out_of_line: Vec<u8> = Vec::new();
    for (tag, kind, count, value) in &entries {
        ifd_bytes.extend(tag.to_le_bytes());
        ifd_bytes.extend(kind.to_le_bytes());
        ifd_bytes.extend(count.to_le_bytes());
        match value {
            Value::Inline(v) if *kind == 3 => {
                ifd_bytes.extend((*v as u16).to_le_bytes());
                ifd_bytes.extend([0, 0]);
            }
            Value::Inline(v) => ifd_bytes.extend(v.to_le_bytes()),
            Value::Out(bytes) => {
                ifd_bytes.extend(next.to_le_bytes());
                next += bytes.len() as u32;
                out_of_line.extend(bytes);
            }
        }
    }
    ifd_bytes.extend(0u32.to_le_bytes());
    file.extend(ifd_bytes);
    file.extend(out_of_line);
    file
}
