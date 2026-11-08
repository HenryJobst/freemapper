//! LVermGeo Saxony-Anhalt open data (dl-de/by-2-0, "© GeoBasis-DE / LVermGeo ST"), all in
//! EPSG:25832: the orthophoto WMS, DGM1 and DOM1 from the WCS, and flight dates from the tile
//! index WFS.

use std::path::Path;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::wms::WmsService;

/// The orthophoto WMS (TrueDOP20, newest flight).
pub const WMS: WmsService = WmsService {
    url: "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest",
    layer: "lsa_lvermgeo_dop20_2",
    crs: Crs::Utm32,
    extra: "",
    max_px: 4000,
};
/// The tile index with flight dates (ArcGIS WFS, GeoJSON output).
pub const INDEX_URL: &str = "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis";

/// A height model served by the WCS at 1 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain.
    Dgm1,
    /// DOM1, the (image-based) surface.
    Dom1,
}

impl Model {
    pub fn wcs_endpoint(self) -> &'static str {
        match self {
            Self::Dgm1 => "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1",
            Self::Dom1 => "https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LVermGeo DGM1)",
            Self::Dom1 => "surface model (LVermGeo DOM1)",
        }
    }
}

/// GetCoverage of `model` over `bbox` (EPSG:25832), rounded outward to whole metres.
pub fn wcs_url(model: Model, bbox: &BoundingBox) -> String {
    format!(
        "{}?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=Coverage1&FORMAT=image/tiff\
         &SUBSET=x({},{})&SUBSET=y({},{})",
        model.wcs_endpoint(),
        bbox.min_e.floor(),
        bbox.max_e.ceil(),
        bbox.min_n.floor(),
        bbox.max_n.ceil()
    )
}

/// The TIFF part of a WCS `multipart/related` answer.
pub fn multipart_tiff(bytes: &[u8]) -> Result<&[u8], String> {
    let refused = || format!("no TIFF in the WCS answer: {}", exception_text(bytes));
    // The first line is the delimiter, e.g. `--wcs`.
    let first_line_end = bytes.iter().position(|&b| b == b'\n').ok_or_else(refused)?;
    let delimiter = bytes[..first_line_end]
        .strip_suffix(b"\r")
        .unwrap_or(&bytes[..first_line_end]);
    if !delimiter.starts_with(b"--") {
        return Err(refused());
    }
    let mut rest = &bytes[first_line_end + 1..];
    loop {
        // Headers end at an empty line.
        let (header_end, body_start) = find(rest, b"\r\n\r\n")
            .map(|i| (i, i + 4))
            .or_else(|| find(rest, b"\n\n").map(|i| (i, i + 2)))
            .ok_or_else(refused)?;
        let headers = String::from_utf8_lossy(&rest[..header_end]).to_ascii_lowercase();
        let body = &rest[body_start..];
        let mut marker = b"\n".to_vec();
        marker.extend_from_slice(delimiter);
        let body_end = find(body, &marker).ok_or_else(refused)?;
        let part = body[..body_end]
            .strip_suffix(b"\r")
            .unwrap_or(&body[..body_end]);
        if headers.contains("content-type: image/tiff") {
            return Ok(part);
        }
        rest = &body[body_end + marker.len()..];
        if rest.starts_with(b"--") {
            return Err(refused());
        }
        rest = rest
            .strip_prefix(b"\r\n")
            .or_else(|| rest.strip_prefix(b"\n"))
            .unwrap_or(rest);
    }
}

/// What an error answer says: an OWS exception's code and text, else its start.
fn exception_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let between = |open: &str, close: char| {
        let start = text.find(open)? + open.len();
        text[start..].split(close).next().map(str::to_string)
    };
    let code = between("exceptionCode=\"", '"');
    let message = between("ExceptionText>", '<');
    match (code, message) {
        (Some(code), Some(message)) => format!("{code}: {message}"),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => text.chars().take(80).collect(),
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// A Float32 GeoTIFF from the WCS as a raster; its grid comes from ModelTiepoint and
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
    // chunk larger than the image may be.
    let padded = u64::from(across)
        .checked_mul(u64::from(chunk_width))
        .zip(u64::from(down).checked_mul(u64::from(chunk_height)))
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

/// LVermGeo marks cells without data with −9999.
const NODATA_BELOW: f32 = -9998.0;
/// Largest side of one GetCoverage; the WCS allows 4000 px (= m) per axis.
const WCS_MAX_M: f64 = 4000.0;
/// The WCS answers 404 for any subset reaching past its coverage (verified live 2026-11-07), so
/// requests are cut to it; the rest stays empty.
const COVERAGE: BoundingBox = BoundingBox {
    min_e: 606_000.0,
    min_n: 5_646_000.0,
    max_e: 790_000.0,
    max_n: 5_882_000.0,
};

/// Downloads `model` over `bbox` (EPSG:25832) in pieces of at most 4000 m and writes it as a
/// Float32 GeoTIFF `out` with its world file (`.tfw`).
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<(), FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let Some(b) = grid.bounds().intersection(&COVERAGE) else {
        // Nothing of the area lies in the coverage: an empty model.
        return write_float_raster(&raster, out, model.wcs_endpoint());
    };
    let starts = |min: f64, max: f64| {
        let n = ((max - min) / WCS_MAX_M).ceil().max(1.0) as u32;
        (0..n).map(move |i| {
            (
                min + f64::from(i) * WCS_MAX_M,
                (min + f64::from(i + 1) * WCS_MAX_M).min(max),
            )
        })
    };
    for (min_e, max_e) in starts(b.min_e, b.max_e) {
        for (min_n, max_n) in starts(b.min_n, b.max_n) {
            let piece = BoundingBox {
                min_e,
                min_n,
                max_e,
                max_n,
            };
            let url = wcs_url(model, &piece);
            let part = get_with_retries(fetcher, &url, |bytes| {
                multipart_tiff(bytes)
                    .and_then(decode_float_tiff)
                    .map_err(|message| FetchError::Http {
                        url: url.clone(),
                        message,
                    })
            })?;
            raster
                .paste_with(&part.grid, |x, y| part.get(x, y))
                .map_err(|e| FetchError::Http {
                    url: url.clone(),
                    message: e.to_string(),
                })?;
        }
    }
    write_float_raster(&raster, out, model.wcs_endpoint())
}

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

/// The flight date of the orthophoto at the centre of `bbox` (EPSG:25832); `None` when the index
/// does not tell it.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    index_date(fetcher, "truedop_kacheln", bbox)
}

/// The date of the surface model (DOM1) at the centre of `bbox`.
pub fn surface_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    index_date(fetcher, "bdom_aktualitaet_zyklus", bbox)
}

fn index_date(fetcher: &dyn Fetcher, layer: &str, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let url = format!(
        "{INDEX_URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES=atkis_geobasis_bestand:{layer}&OUTPUTFORMAT=GEOJSON\
         &BBOX={},{},{},{},urn:ogc:def:crs:EPSG::25832",
        (e - 10.0).floor(),
        (n - 10.0).floor(),
        (e + 10.0).ceil(),
        (n + 10.0).ceil()
    );
    // A failed index costs a re-download of the orthophoto, so it gets the usual retries.
    get_with_retries(fetcher, &url, |bytes| {
        let text = String::from_utf8_lossy(bytes);
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .filter(|answer| answer["features"].is_array())
            .map(|_| parse_index_date(&text))
            .ok_or_else(|| FetchError::Http {
                url: url.clone(),
                message: format!("not a tile index answer: {}", exception_text(bytes)),
            })
    })
    .ok()
    .flatten()
}

/// The flight date (`BFDATUM`, e.g. `29.04.2026`) of the first feature in a tile index answer.
pub fn parse_index_date(json: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(json).ok()?;
    let date = answer["features"].get(0)?["properties"]["BFDATUM"].as_str()?;
    (!date.is_empty()).then(|| date.to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const WCS_DGM1: &[u8] =
        include_bytes!("../../../../tests/fixtures/lvermgeo-wcs-dgm1.multipart");
    pub const TRUEDOP_INDEX: &str =
        include_str!("../../../../tests/fixtures/lvermgeo-truedop-index.json");
    pub const BDOM_INDEX: &str =
        include_str!("../../../../tests/fixtures/lvermgeo-bdom-index.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// The live sample: 20 × 16 m at the Latina, Halle.
    const SAMPLE: BoundingBox = BoundingBox {
        min_e: 706_440.0,
        min_n: 5_706_900.0,
        max_e: 706_460.0,
        max_n: 5_706_916.0,
    };

    #[test]
    fn wcs_url_subsets_whole_metres() {
        let bbox = BoundingBox {
            min_e: 706_440.4,
            min_n: 5_706_899.6,
            max_e: 706_459.2,
            max_n: 5_706_915.1,
        };
        assert_eq!(
            wcs_url(Model::Dgm1, &bbox),
            "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1?SERVICE=WCS&VERSION=2.0.1\
             &REQUEST=GetCoverage&COVERAGEID=Coverage1&FORMAT=image/tiff\
             &SUBSET=x(706440,706460)&SUBSET=y(5706899,5706916)"
        );
        assert!(wcs_url(Model::Dom1, &bbox).starts_with(Model::Dom1.wcs_endpoint()));
    }

    #[test]
    fn multipart_answer_yields_the_tiff() {
        let tiff = multipart_tiff(WCS_DGM1).unwrap();
        assert!(tiff.starts_with(b"II*\0"), "{:?}", &tiff[..8]);
        assert!(multipart_tiff(b"<?xml version=\"1.0\"?><ExceptionReport/>").is_err());
        assert!(multipart_tiff(b"--wcs\nContent-Type: text/xml\n\n<x/>\n--wcs--\n").is_err());
    }

    #[test]
    fn geotiff_tags_give_the_grid() {
        let raster = decode_float_tiff(multipart_tiff(WCS_DGM1).unwrap()).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (20, 16));
        assert_eq!(
            (raster.grid.west, raster.grid.north),
            (706_440.0, 5_706_916.0)
        );
        assert_eq!(raster.grid.pixel_size, 1.0);
        // The school yard lies at about 90.8 m (reference §5).
        let z = raster.get(10, 8);
        assert!((85.0..96.0).contains(&z), "{z}");
    }

    #[test]
    fn fetched_model_is_written_with_its_world_file() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(Model::Dgm1.wcs_endpoint(), WCS_DGM1.to_vec());
        let out = dir.path().join("dgm1.tif");
        fetch_model(&fetcher, Model::Dgm1, &SAMPLE, &out).unwrap();
        let world = std::fs::read_to_string(out.with_extension("tfw")).unwrap();
        assert_eq!(
            world,
            crate::raster::GridSpec::covering(&SAMPLE, 1.0).world_file()
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (20, 16));
        assert!((85.0..96.0).contains(&raster.get(10, 8)));
    }

    #[test]
    fn wcs_error_page_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            Model::Dgm1.wcs_endpoint(),
            b"<html><body>ArcGIS Server Error</body></html>".to_vec(),
        );
        let err =
            fetch_model(&fetcher, Model::Dgm1, &SAMPLE, &dir.path().join("x.tif")).unwrap_err();
        assert!(err.to_string().contains("no TIFF"), "{err}");
        assert_eq!(fetcher.calls.get(), 3, "retried");
        assert!(!dir.path().join("x.tif").exists());
    }

    #[test]
    fn tile_index_gives_the_flight_date() {
        assert_eq!(
            parse_index_date(TRUEDOP_INDEX).as_deref(),
            Some("29.04.2026")
        );
        assert_eq!(parse_index_date(BDOM_INDEX).as_deref(), Some("13.05.2024"));
        assert_eq!(
            parse_index_date(r#"{"type":"FeatureCollection","features":[]}"#),
            None
        );
    }

    #[test]
    fn wms_is_asked_in_zone_32() {
        let grid = crate::raster::GridSpec::covering(&SAMPLE, 0.2);
        let (_, url) = &crate::sources::wms::pieces(&WMS, &grid)[0];
        assert!(
            url.starts_with(WMS.url) && url.contains("&CRS=EPSG:25832&"),
            "{url}"
        );
        assert!(url.contains("LAYERS=lsa_lvermgeo_dop20_2"), "{url}");
    }

    /// Answers every GetCoverage with a synthetic TIFF of exactly the requested subset (heights
    /// 50 m), and records the subsets asked for.
    struct Wcs {
        subsets: std::cell::RefCell<Vec<(f64, f64, f64, f64)>>,
    }

    impl Fetcher for Wcs {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            let range = |axis: &str| -> (f64, f64) {
                let start = url.find(&format!("SUBSET={axis}(")).unwrap() + axis.len() + 8;
                let inner = &url[start..url[start..].find(')').unwrap() + start];
                let (a, b) = inner.split_once(',').unwrap();
                (a.parse().unwrap(), b.parse().unwrap())
            };
            let ((x0, x1), (y0, y1)) = (range("x"), range("y"));
            self.subsets.borrow_mut().push((x0, x1, y0, y1));
            let grid = crate::raster::GridSpec {
                west: x0,
                north: y1,
                pixel_size: 1.0,
                width: (x1 - x0) as u32,
                height: (y1 - y0) as u32,
            };
            let values = vec![50.0; (grid.width * grid.height) as usize];
            Ok(crate::fetch::test_support::wcs_answer(&grid, &values))
        }
    }

    #[test]
    fn wcs_requests_stay_inside_the_coverage_and_split_at_4000_m() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = Wcs {
            subsets: std::cell::RefCell::new(Vec::new()),
        };
        // 4100 m wide, reaching 50 m past the east edge of the coverage (E 790 000).
        let bbox = BoundingBox {
            min_e: 785_950.0,
            min_n: 5_700_000.0,
            max_e: 790_050.0,
            max_n: 5_700_010.0,
        };
        let out = dir.path().join("dgm1.tif");
        fetch_model(&fetcher, Model::Dgm1, &bbox, &out).unwrap();
        let subsets = fetcher.subsets.borrow();
        assert_eq!(
            *subsets,
            [
                (785_950.0, 789_950.0, 5_700_000.0, 5_700_010.0),
                (789_950.0, 790_000.0, 5_700_000.0, 5_700_010.0)
            ]
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.grid.width, 4100);
        assert_eq!(raster.get(0, 0), 50.0);
        assert!(raster.get(4099, 0).is_nan(), "beyond the coverage");
    }

    #[test]
    fn area_outside_the_coverage_asks_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = Wcs {
            subsets: std::cell::RefCell::new(Vec::new()),
        };
        let bbox = BoundingBox {
            min_e: 795_000.0,
            min_n: 5_700_000.0,
            max_e: 795_100.0,
            max_n: 5_700_100.0,
        };
        fetch_model(&fetcher, Model::Dgm1, &bbox, &dir.path().join("x.tif")).unwrap();
        assert!(fetcher.subsets.borrow().is_empty());
    }

    #[test]
    fn wcs_exception_text_is_shown() {
        let report = br#"<?xml version="1.0" encoding="UTF-8"?><ExceptionReport xmlns="http://www.opengis.net/ows/2.0" version="2.0.0"><Exception exceptionCode="InvalidSubsetting" locator="x"><ExceptionText>Subset is outside the coverage extent</ExceptionText></Exception></ExceptionReport>"#;
        let err = multipart_tiff(report).unwrap_err();
        assert!(err.contains("InvalidSubsetting"), "{err}");
        assert!(
            err.contains("Subset is outside the coverage extent"),
            "{err}"
        );
    }

    #[test]
    fn tile_index_is_retried() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(INDEX_URL, TRUEDOP_INDEX.as_bytes().to_vec());
        fetcher.flaky.borrow_mut().insert(INDEX_URL.to_string(), 1);
        assert_eq!(
            flight_date(&fetcher, &SAMPLE).as_deref(),
            Some("29.04.2026")
        );
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
    #[ignore = "needs the full Lübeck tile in the session scratchpad"]
    fn full_lubeck_tile_decodes() {
        let path = std::env::var("FREEMAPPER_LZW_TILE").unwrap_or_else(|_| {
            "/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/\
             626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/lzwprobe/t.tif"
                .to_string()
        });
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
