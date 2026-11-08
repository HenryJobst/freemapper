//! Float32 GeoTIFF height models: reading (ModelTiepoint and ModelPixelScale, strips or tiles,
//! with a lenient LZW fallback) and writing (with a world file, or with GeoTIFF tags for caching).

use std::path::Path;

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, write_atomically};
use crate::raster::{GridSpec, Raster};

/// A Float32 GeoTIFF (a WCS answer, a tile) as a raster; its grid comes from ModelTiepoint and
/// ModelPixelScale (the 25832 answers carry no GeoKeys). NoData (−9999) becomes `NaN`.
/// Striped and tiled (LVermGeo SH: 256 px tiles) TIFFs are read.
pub fn decode_float_tiff(bytes: &[u8]) -> Result<Raster<f32>, String> {
    decode_float_tiff_from(std::io::Cursor::new(bytes))
}

/// [`decode_float_tiff`] from a reader, so a large file is never held as bytes besides its
/// pixels.
pub(crate) fn decode_float_tiff_from(
    reader: impl std::io::Read + std::io::Seek,
) -> Result<Raster<f32>, String> {
    use tiff::tags::Tag;
    let error = |e: tiff::TiffError| e.to_string();
    let mut decoder = tiff::decoder::Decoder::new(reader).map_err(error)?;
    let (width, height) = decoder.dimensions().map_err(error)?;
    let scale = decoder
        .get_tag_f64_vec(Tag::ModelPixelScaleTag)
        .map_err(error)?;
    let tiepoint = decoder
        .get_tag_f64_vec(Tag::ModelTiepointTag)
        .map_err(error)?;
    let (Some(&pixel), Some(&west), Some(&north)) =
        (scale.first(), tiepoint.get(3), tiepoint.get(4))
    else {
        return Err("GeoTIFF without tiepoint or pixel scale".to_string());
    };
    let values = match decoder.read_image() {
        Ok(tiff::decoder::DecodingResult::F32(values)) => values,
        Ok(_) => return Err("height model is not 32-bit float".to_string()),
        // `tiff` 0.11.3 refuses some complete LZW chunks ("no lzw end code found", seen on
        // LVermGeo SH edge tiles); read those like libtiff does.
        Err(tiff::TiffError::IoError(e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof
                && decoder
                    .find_tag_unsigned::<u16>(Tag::Compression)
                    .ok()
                    .flatten()
                    == Some(LZW) =>
        {
            lenient_lzw_values(&mut decoder, width, height)?
        }
        Err(e) => return Err(error(e)),
    };
    // Tiepoints may carry float noise (LGV: 564999.9999999999); round to millimetres.
    let mm = |v: f64| (v * 1000.0).round() / 1000.0;
    let grid = GridSpec {
        west: mm(west),
        north: mm(north),
        pixel_size: pixel,
        width,
        height,
    };
    Ok(Raster {
        grid,
        data: values
            .into_iter()
            .map(|z| if z < NODATA_BELOW { f32::NAN } else { z })
            .collect(),
    })
}

/// TIFF compression 5.
const LZW: u16 = 5;
/// Largest compressed chunk read by [`lenient_lzw_values`] (the `tiff` crate's default limit).
const MAX_CHUNK_BYTES: u64 = 128 * 1024 * 1024;
/// Largest image read by [`lenient_lzw_values`] (the `tiff` crate's default buffer limit).
const MAX_PIXELS: u64 = 256 * 1024 * 1024 / 4;

/// The Float32 pixels of an LZW TIFF (strips or tiles, predictor 1, 2 or 3) read chunk by chunk
/// with `weezl` like libtiff does: a chunk's data ends at its end code or where it has given
/// all the chunk's bytes, whichever comes first. The chunks must cover the image, and each must
/// give exactly its size.
fn lenient_lzw_values<R: std::io::Read + std::io::Seek>(
    decoder: &mut tiff::decoder::Decoder<R>,
    width: u32,
    height: u32,
) -> Result<Vec<f32>, String> {
    use tiff::decoder::ChunkType;
    use tiff::tags::Tag;
    let error = |e: tiff::TiffError| e.to_string();
    let mut tag = |t: Tag, default: u16| {
        decoder
            .find_tag_unsigned::<u16>(t)
            .map(|v| v.unwrap_or(default))
            .map_err(error)
    };
    let (samples, bits, format, predictor) = (
        tag(Tag::SamplesPerPixel, 1)?,
        tag(Tag::BitsPerSample, 1)?,
        tag(Tag::SampleFormat, 1)?,
        tag(Tag::Predictor, 1)?,
    );
    if (samples, bits, format) != (1, 32, 3) {
        return Err(format!(
            "LZW: {samples} samples of {bits} bit (format {format}), not one Float32"
        ));
    }
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(format!("{width} x {height} px are too large"));
    }
    let tiled = decoder.get_chunk_type() == ChunkType::Tile;
    let (offsets, counts) = if tiled {
        (Tag::TileOffsets, Tag::TileByteCounts)
    } else {
        (Tag::StripOffsets, Tag::StripByteCounts)
    };
    let offsets = decoder.get_tag_u64_vec(offsets).map_err(error)?;
    let counts = decoder.get_tag_u64_vec(counts).map_err(error)?;
    let (chunk_width, chunk_height) = decoder.chunk_dimensions();
    if chunk_width == 0 || chunk_height == 0 || (!tiled && chunk_width != width) {
        return Err(format!("LZW: chunks of {chunk_width} x {chunk_height} px"));
    }
    let (across, down) = (width.div_ceil(chunk_width), height.div_ceil(chunk_height));
    // The image rounded up to whole chunks: a hostile TileWidth/TileLength must not make one
    // chunk larger than the image may be. A strip holds at most the image's rows (and
    // RowsPerStrip = 2^32 - 1 is libtiff's "the whole image in one strip").
    let rows_per_chunk = if tiled {
        chunk_height
    } else {
        chunk_height.min(height)
    };
    let padded = u64::from(across)
        .checked_mul(u64::from(chunk_width))
        .zip(u64::from(down).checked_mul(u64::from(rows_per_chunk)))
        .and_then(|(w, h)| w.checked_mul(h));
    if !padded.is_some_and(|px| px <= MAX_PIXELS) {
        return Err(format!(
            "LZW: tiles of {chunk_width} x {chunk_height} px are too large"
        ));
    }
    let chunks = across as usize * down as usize;
    if offsets.len() != chunks || counts.len() != chunks {
        return Err(format!(
            "LZW: {} offsets and {} byte counts for {chunks} chunks of {width} x {height} px",
            offsets.len(),
            counts.len()
        ));
    }
    let file_len = decoder
        .inner()
        .seek(std::io::SeekFrom::End(0))
        .map_err(|e| format!("LZW: {e}"))?;
    let little = decoder.byte_order() == tiff::tags::ByteOrder::LittleEndian;
    let mut values = vec![f32::NAN; width as usize * height as usize];
    let mut compressed = Vec::new();
    for (i, (&offset, &count)) in offsets.iter().zip(&counts).enumerate() {
        let (cx, cy) = (i as u32 % across, i as u32 / across);
        // A tile is always whole; the last strip holds only the rows left.
        let rows = if tiled {
            chunk_height
        } else {
            chunk_height.min(height - cy * chunk_height)
        };
        if count > MAX_CHUNK_BYTES {
            return Err(format!("LZW chunk {i}: {count} bytes are too large"));
        }
        if !offset.checked_add(count).is_some_and(|end| end <= file_len) {
            return Err(format!(
                "LZW chunk {i}: {count} bytes at {offset} reach past the end of the file \
                 ({file_len} bytes)"
            ));
        }
        compressed.resize(count as usize, 0);
        let reader = decoder.inner();
        reader
            .seek(std::io::SeekFrom::Start(offset))
            .and_then(|_| reader.read_exact(&mut compressed))
            .map_err(|e| format!("LZW chunk {i}: {e}"))?;
        // Bounded by MAX_PIXELS above; checked all the same.
        let row_bytes = (chunk_width as usize)
            .checked_mul(4)
            .ok_or_else(|| format!("LZW chunk {i}: rows too large"))?;
        let expected = row_bytes
            .checked_mul(rows as usize)
            .ok_or_else(|| format!("LZW chunk {i}: too large"))?;
        let mut bytes =
            lzw_decode(&compressed, expected).map_err(|m| format!("LZW chunk {i}: {m}"))?;
        for (y, row) in bytes.chunks_exact_mut(row_bytes).enumerate() {
            let row =
                predicted_row(row, predictor, little).map_err(|m| format!("LZW chunk {i}: {m}"))?;
            let image_y = cy * chunk_height + y as u32;
            if image_y >= height {
                break;
            }
            let x0 = cx * chunk_width;
            let used = chunk_width.min(width - x0) as usize;
            let start = image_y as usize * width as usize + x0 as usize;
            values[start..start + used].copy_from_slice(&row[..used]);
        }
    }
    Ok(values)
}

/// The TIFF LZW data `input` (MSB first, 8 bit, early size switch) as exactly `expected` bytes,
/// stopping at the end code, at `expected` bytes or at the end of `input`.
fn lzw_decode(input: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    let mut decoder =
        weezl::decode::Configuration::with_tiff_size_switch(weezl::BitOrder::Msb, 8).build();
    let mut out = vec![0; expected];
    let (mut read, mut written) = (0, 0);
    while written < expected {
        let result = decoder.decode_bytes(&input[read..], &mut out[written..]);
        read += result.consumed_in;
        written += result.consumed_out;
        match result.status {
            Ok(weezl::LzwStatus::Done | weezl::LzwStatus::NoProgress) => break,
            Ok(weezl::LzwStatus::Ok) if result.consumed_in + result.consumed_out == 0 => break,
            Ok(weezl::LzwStatus::Ok) => {}
            Err(e) => return Err(format!("LZW: {e}")),
        }
    }
    if written != expected {
        return Err(format!("LZW data for {written} of {expected} bytes"));
    }
    Ok(out)
}

/// One row of Float32 samples as stored (`little` endian) with `predictor` undone: 1 none,
/// 2 horizontal differencing of the 32-bit words, 3 the floating point predictor (byte
/// differencing over the row, the bytes of each sample in planes, most significant first).
fn predicted_row(row: &mut [u8], predictor: u16, little: bool) -> Result<Vec<f32>, String> {
    let word = |b: &[u8; 4]| {
        if little {
            u32::from_le_bytes(*b)
        } else {
            u32::from_be_bytes(*b)
        }
    };
    match predictor {
        1 => Ok(row
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_bits(word(b)))
            .collect()),
        2 => {
            let mut previous = 0u32;
            Ok(row
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| {
                    previous = previous.wrapping_add(word(b));
                    f32::from_bits(previous)
                })
                .collect())
        }
        3 => {
            for i in 1..row.len() {
                row[i] = row[i].wrapping_add(row[i - 1]);
            }
            let n = row.len() / 4;
            Ok((0..n)
                .map(|i| f32::from_be_bytes([row[i], row[n + i], row[2 * n + i], row[3 * n + i]]))
                .collect())
        }
        other => Err(format!("unknown predictor {other}")),
    }
}

/// LVermGeo (and others) mark cells without data with −9999.
const NODATA_BELOW: f32 = -9998.0;
/// Writes `raster` as a Float32 GeoTIFF `out` with its world file (`.tfw`); `source` names the
/// service in an error.
pub(crate) fn write_float_raster(
    raster: &Raster<f32>,
    out: &Path,
    source: &str,
) -> Result<(), FetchError> {
    let grid = raster.grid;
    let mut tiff = std::io::Cursor::new(Vec::new());
    tiff::encoder::TiffEncoder::new(&mut tiff)
        .and_then(|encoder| {
            encoder
                .with_compression(tiff::encoder::Compression::Lzw)
                .write_image::<tiff::encoder::colortype::Gray32Float>(
                grid.width,
                grid.height,
                &raster.data,
            )
        })
        .map_err(|e| FetchError::Http {
            url: source.to_string(),
            message: e.to_string(),
        })?;
    write_atomically(out, tiff.get_ref())?;
    write_atomically(&out.with_extension("tfw"), grid.world_file().as_bytes())?;
    Ok(())
}

/// The grid of the model [`write_float_raster`] stored at `out` (its world file and TIFF size).
pub(crate) fn stored_grid(out: &Path) -> Option<GridSpec> {
    let world = std::fs::read_to_string(out.with_extension("tfw")).ok()?;
    let file = std::fs::File::open(out).ok()?;
    let (width, height) = tiff::decoder::Decoder::new(std::io::BufReader::new(file))
        .ok()?
        .dimensions()
        .ok()?;
    GridSpec::from_world_file(&world, width, height).ok()
}

/// Offline: the model `label` stored at `out` must cover `area`, else the error asks for an
/// online fetch.
pub(crate) fn require_stored(
    out: &Path,
    area: &BoundingBox,
    label: &str,
) -> Result<(), FetchError> {
    let covers = stored_grid(out).is_some_and(|stored| {
        let s = stored.bounds();
        s.min_e <= area.min_e
            && s.min_n <= area.min_n
            && s.max_e >= area.max_e
            && s.max_n >= area.max_n
    });
    if covers {
        return Ok(());
    }
    Err(FetchError::Http {
        url: out.display().to_string(),
        message: format!(
            "no {label} stored for this map area; run `freemapper fetch` online first"
        ),
    })
}

/// `raster` as an LZW Float32 GeoTIFF with ModelPixelScale and ModelTiepoint, which
/// [`decode_float_tiff`] reads back (for caching; no world file).
pub(crate) fn encode_float_geotiff(raster: &Raster<f32>) -> Result<Vec<u8>, String> {
    use tiff::tags::Tag;
    let grid = raster.grid;
    let mut tiff = std::io::Cursor::new(Vec::new());
    let mut encoder = tiff::encoder::TiffEncoder::new(&mut tiff)
        .map_err(|e| e.to_string())?
        .with_compression(tiff::encoder::Compression::Lzw);
    let mut image = encoder
        .new_image::<tiff::encoder::colortype::Gray32Float>(grid.width, grid.height)
        .map_err(|e| e.to_string())?;
    image
        .encoder()
        .write_tag(
            Tag::ModelPixelScaleTag,
            &[grid.pixel_size, grid.pixel_size, 0.0][..],
        )
        .map_err(|e| e.to_string())?;
    image
        .encoder()
        .write_tag(
            Tag::ModelTiepointTag,
            &[0.0, 0.0, 0.0, grid.west, grid.north, 0.0][..],
        )
        .map_err(|e| e.to_string())?;
    image.write_data(&raster.data).map_err(|e| e.to_string())?;
    Ok(tiff.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_stored_needs_a_covering_model() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("dgm1.tif");
        let grid = GridSpec {
            west: 1000.0,
            north: 2000.0,
            pixel_size: 1.0,
            width: 10,
            height: 10,
        };
        let inside = BoundingBox {
            min_e: 1002.0,
            min_n: 1992.0,
            max_e: 1010.0,
            max_n: 2000.0,
        };
        let message = |e: FetchError| e.to_string();
        // Nothing stored yet.
        let err = message(require_stored(&out, &inside, "terrain model (X)").unwrap_err());
        assert!(
            err.contains(
                "no terrain model (X) stored for this map area; run `freemapper fetch` online first"
            ),
            "{err}"
        );
        write_float_raster(&Raster::filled(grid, 1.0), &out, "x").unwrap();
        require_stored(&out, &inside, "terrain model (X)").unwrap();
        require_stored(&out, &grid.bounds(), "terrain model (X)").unwrap();
        // One metre beyond an edge is not covered.
        let beyond = BoundingBox {
            max_e: 1011.0,
            ..inside
        };
        let err = message(require_stored(&out, &beyond, "surface model (X)").unwrap_err());
        assert!(err.contains("no surface model (X) stored"), "{err}");
    }

    /// The north-east 488 x 300 m of the LVermGeo SH DGM1 tile 611/5970 (Lübeck), its 2 x 2
    /// LZW tiles of 256 px copied byte for byte. The `tiff` crate (0.11.3) refuses the upper
    /// right one, an edge tile of 232 data columns: reading row by row, the padding of its last
    /// row ends in "no lzw end code found" although the stream is complete (libtiff and a
    /// plain `weezl` decode reach its end code).
    const LZW_NE: &[u8] =
        include_bytes!("../../../../tests/fixtures/schleswig-holstein-dgm1-lzw-ne.tif");

    #[test]
    fn lzw_chunks_without_end_code_decode() {
        // The crate alone fails on this crop, so the test needs the lenient path.
        let mut plain = tiff::decoder::Decoder::new(std::io::Cursor::new(LZW_NE)).unwrap();
        let err = plain.read_image().unwrap_err().to_string();
        assert!(err.contains("no lzw end code"), "{err}");

        let raster = decode_float_tiff(LZW_NE).unwrap();
        assert_eq!(
            raster.grid,
            GridSpec {
                west: 611_512.0,
                north: 5_971_000.0,
                pixel_size: 1.0,
                width: 488,
                height: 300,
            }
        );
        // Pillow (libtiff) on the full tile, at the same cells.
        for (x, y, z) in [
            (0, 0, 8.779_999_732_971_191),
            (487, 0, 5.909_999_847_412_109),
            (0, 299, 2.740_000_009_536_743),
            (100, 50, 5.5),
            (256, 255, 3.400_000_095_367_431_6),
            (487, 299, 3.400_000_095_367_431_6),
        ] {
            assert_eq!(raster.get(x, y), z as f32, "({x}, {y})");
        }
        let (mut sum, mut by_x, mut by_y) = (0.0, 0.0, 0.0);
        for y in 0..300 {
            for x in 0..488 {
                let z = f64::from(raster.get(x, y));
                sum += z;
                by_x += z * f64::from(x);
                by_y += z * f64::from(y);
            }
        }
        for (got, want) in [
            (sum, 613_968.827_340_841_3),
            (by_x, 143_868_192.343_950_03),
            (by_y, 81_677_328.628_171_92),
        ] {
            assert!((got - want).abs() < 1e-6 * want, "{got} vs {want}");
        }
    }

    #[test]
    fn lenient_lzw_reads_strips_with_horizontal_differencing() {
        use tiff::encoder::colortype::Gray32;
        use tiff::encoder::{Compression, Predictor, TiffEncoder};
        use tiff::tags::Tag;
        // LZW strips with predictor 2 (the bit patterns written as 32-bit words, the sample
        // format set to float afterwards, as in the HVBG test), several strips.
        let (width, height) = (700, 450);
        let heights: Vec<f32> = (0..width * height)
            .map(|i| 20.0 + (i % 997) as f32 * 0.37 - (i / 700) as f32 * 0.01)
            .collect();
        let words: Vec<u32> = heights.iter().map(|h| h.to_bits()).collect();
        let mut tiff = std::io::Cursor::new(Vec::new());
        {
            let mut encoder = TiffEncoder::new(&mut tiff)
                .unwrap()
                .with_compression(Compression::Lzw)
                .with_predictor(Predictor::Horizontal);
            let mut image = encoder.new_image::<Gray32>(width, height).unwrap();
            image.rows_per_strip(100).unwrap();
            image.write_data(&words).unwrap();
        }
        let mut bytes = tiff.into_inner();
        let ifd = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let count = usize::from(u16::from_le_bytes([bytes[ifd], bytes[ifd + 1]]));
        let entry = (0..count)
            .map(|i| ifd + 2 + 12 * i)
            .find(|&e| u16::from_le_bytes([bytes[e], bytes[e + 1]]) == 339)
            .unwrap();
        bytes[entry + 8..entry + 10].copy_from_slice(&3u16.to_le_bytes());
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).unwrap();
        assert_eq!(decoder.get_tag_u64_vec(Tag::StripOffsets).unwrap().len(), 5);
        let lenient = lenient_lzw_values(&mut decoder, width, height).unwrap();
        assert_eq!(lenient, heights);
        // A chunk table that does not cover the image is refused.
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).unwrap();
        let err = lenient_lzw_values(&mut decoder, width, height + 100).unwrap_err();
        assert!(err.contains("for 6 chunks"), "{err}");
        // A chunk shorter than its size is refused.
        assert!(
            lzw_decode(&[0x80, 0x00, 0x00], 4)
                .unwrap_err()
                .contains("of 4 bytes")
        );
    }

    /// A little-endian tiled Float32 LZW TIFF of `width` x `height` px with one tile of
    /// `tile_width` x `tile_length` px whose data is `data`.
    fn one_tile_tiff(
        width: u32,
        height: u32,
        tile_width: u32,
        tile_length: u32,
        data: &[u8],
    ) -> Vec<u8> {
        // (tag, type, value): type 3 SHORT, 4 LONG; every value fits the entry.
        let data_at = 8u32;
        let entries: [(u16, u16, u32); 11] = [
            (256, 4, width),
            (257, 4, height),
            (258, 3, 32),
            (259, 3, u32::from(LZW)),
            (262, 3, 1),
            (277, 3, 1),
            (322, 4, tile_width),
            (323, 4, tile_length),
            (324, 4, data_at),
            (325, 4, data.len() as u32),
            (339, 3, 3),
        ];
        let ifd = data_at + data.len() as u32;
        let mut bytes = b"II*\0".to_vec();
        bytes.extend(ifd.to_le_bytes());
        bytes.extend(data);
        bytes.extend((entries.len() as u16).to_le_bytes());
        for (tag, kind, value) in entries {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(kind.to_le_bytes());
            bytes.extend(1u32.to_le_bytes());
            if kind == 3 {
                bytes.extend((value as u16).to_le_bytes());
                bytes.extend([0, 0]);
            } else {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.extend(0u32.to_le_bytes());
        bytes
    }

    /// A little-endian striped Float32 LZW TIFF of `width` x `height` px in one strip `data`,
    /// declaring `rows_per_strip`.
    fn one_strip_tiff(
        width: u32,
        height: u32,
        rows_per_strip: u32,
        predictor: u16,
        data: &[u8],
    ) -> Vec<u8> {
        let data_at = 8u32;
        let entries: [(u16, u16, u32); 11] = [
            (256, 4, width),
            (257, 4, height),
            (258, 3, 32),
            (259, 3, u32::from(LZW)),
            (262, 3, 1),
            (273, 4, data_at),
            (277, 3, 1),
            (278, 4, rows_per_strip),
            (279, 4, data.len() as u32),
            (317, 3, u32::from(predictor)),
            (339, 3, 3),
        ];
        let ifd = data_at + data.len() as u32;
        let mut bytes = b"II*\0".to_vec();
        bytes.extend(ifd.to_le_bytes());
        bytes.extend(data);
        bytes.extend((entries.len() as u16).to_le_bytes());
        for (tag, kind, value) in entries {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(kind.to_le_bytes());
            bytes.extend(1u32.to_le_bytes());
            if kind == 3 {
                bytes.extend((value as u16).to_le_bytes());
                bytes.extend([0, 0]);
            } else {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.extend(0u32.to_le_bytes());
        bytes
    }

    #[test]
    fn lenient_lzw_reads_one_strip_of_rows_per_strip_max() {
        // libtiff's "the whole image in one strip": RowsPerStrip = 2^32 - 1.
        let (width, height) = (4, 2);
        let heights: Vec<f32> = (0..8).map(|i| 20.0 + i as f32 * 0.25).collect();
        let raw: Vec<u8> = heights.iter().flat_map(|h| h.to_le_bytes()).collect();
        let encoded = weezl::encode::Encoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8)
            .encode(&raw)
            .unwrap();
        // The stream without its end code (9-bit codes here), as on the LVermGeo SH edge
        // tiles: the code bits up to the end code, the rest of the byte zero.
        let codes = encoded.len() * 8 / 9;
        let bits = (codes - 1) * 9;
        let mut data = encoded[..bits.div_ceil(8)].to_vec();
        if !bits.is_multiple_of(8) {
            *data.last_mut().unwrap() &= 0xFF << (8 - bits % 8);
        }
        let tiff = one_strip_tiff(width, height, u32::MAX, 1, &data);
        // The fallback itself (the `tiff` crate reads this small strip; it refuses some
        // complete chunks, as `lzw_chunks_without_end_code_decode` shows): the declared strip
        // height is not the size of the strip.
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&tiff)).unwrap();
        assert_eq!(decoder.chunk_dimensions(), (width, u32::MAX));
        assert_eq!(
            lenient_lzw_values(&mut decoder, width, height).unwrap(),
            heights
        );
    }

    #[test]
    fn lenient_lzw_refuses_oversized_tiles() {
        // A clear code and an end code: a few bytes that claim nothing.
        let data = [0x80, 0x00, 0x40, 0x40];
        let tiff = one_tile_tiff(16, 16, 16, 16, &data);
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&tiff)).unwrap();
        let err = lenient_lzw_values(&mut decoder, 16, 16).unwrap_err();
        assert!(err.contains("LZW data for"), "{err}");
        // One tile far larger than the image (16 GiB of pixels), and one whose size overflows.
        for side in [65_536, u32::MAX - 15] {
            let tiff = one_tile_tiff(256, 256, side, side, &data);
            let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&tiff)).unwrap();
            assert_eq!(decoder.chunk_dimensions(), (side, side));
            let err = lenient_lzw_values(&mut decoder, 256, 256).unwrap_err();
            assert!(
                err.contains("tiles of") && err.contains("too large"),
                "{err}"
            );
        }
        // A byte count beyond the end of the file is refused before it is allocated.
        let mut tiff = one_tile_tiff(16, 16, 16, 16, &data);
        let ifd = u32::from_le_bytes(tiff[4..8].try_into().unwrap()) as usize;
        let counts = ifd + 2 + 12 * 9 + 8;
        tiff[counts..counts + 4].copy_from_slice(&(100_000_000u32).to_le_bytes());
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&tiff)).unwrap();
        let err = lenient_lzw_values(&mut decoder, 16, 16).unwrap_err();
        assert!(err.contains("past the end of the file"), "{err}");
    }

    #[test]
    fn floating_point_predictor_rows_are_undone() {
        let values = [29.85_f32, -9999.0, 0.32, 16.98, f32::MAX];
        // Encode like libtiff: the bytes of each sample in planes (most significant first),
        // then byte differencing over the row.
        let n = values.len();
        let mut row = vec![0u8; 4 * n];
        for (i, v) in values.iter().enumerate() {
            for (plane, byte) in v.to_be_bytes().iter().enumerate() {
                row[plane * n + i] = *byte;
            }
        }
        for i in (1..row.len()).rev() {
            row[i] = row[i].wrapping_sub(row[i - 1]);
        }
        for little in [true, false] {
            assert_eq!(predicted_row(&mut row.clone(), 3, little).unwrap(), values);
        }
        assert!(predicted_row(&mut row, 4, true).is_err());
    }

    #[test]
    #[ignore = "needs the full Lübeck DGM1 tile in FREEMAPPER_LIVE_TILE"]
    fn full_lubeck_tile_decodes() {
        // The whole Lübeck DGM1 GeoTIFF of the INSPIRE ATOM directory that the `LZW_NE` crop
        // comes from, downloaded by hand.
        let Ok(path) = std::env::var("FREEMAPPER_LIVE_TILE") else {
            eprintln!("FREEMAPPER_LIVE_TILE is not set; skipped");
            return;
        };
        let raster = decode_float_tiff(&std::fs::read(path).unwrap()).unwrap();
        let (min, max) = raster
            .data
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &z| (lo.min(z), hi.max(z)));
        eprintln!("{:?}: {min} .. {max} m", raster.grid);
        assert_eq!((raster.grid.width, raster.grid.height), (1000, 1000));
        assert_eq!((min, max), (0.32, 16.98));
        assert!(raster.data.iter().all(|z| z.is_finite()));
    }
}
