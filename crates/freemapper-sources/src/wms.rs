//! Orthophotos from a state's WMS (GeoSN, LVermGeo) or WCS (LAiV M-V): JPEG pieces of at most
//! 4000 px at 0.2 m, stitched on the template grid of the map area.

use std::borrow::Cow;
use std::path::Path;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries, is_ogc_report, one_line, write_inside};
use crate::raster::GridSpec;
use crate::wcs::{WcsOrthophoto, exception_text};

/// Pixel size of the orthophoto template, as DOP20.
pub const ORTHOPHOTO_PIXEL_M: f64 = 0.2;
/// JPEG quality of the stitched orthophoto.
const JPEG_QUALITY: u8 = 90;

/// How an orthophoto service's date is named when it is no flight date as such.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateLabel {
    /// freemapper's name for the date, kept in the project record (`flight_label`).
    pub record: &'static str,
    /// Shown before the date in the log, in place of "flight".
    pub shown: &'static str,
}

/// A WMS layer with the orthophoto, asked for in `crs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WmsService {
    pub url: Cow<'static, str>,
    pub layer: Cow<'static, str>,
    pub crs: Crs,
    /// More GetMap parameters, e.g. `&TIME=2026`.
    pub extra: Cow<'static, str>,
    /// Largest side of one GetMap request in pixels (the server's MaxWidth/MaxHeight).
    pub max_px: u32,
}

impl WmsService {
    /// The GetMap request for `piece` (JPEG).
    pub fn piece_url(&self, piece: &GridSpec) -> String {
        self.piece_url_as(piece, "image/jpeg")
    }

    /// The GetMap request for `piece` in `format` (a MIME type).
    fn piece_url_as(&self, piece: &GridSpec, format: &str) -> String {
        let b = piece.bounds();
        format!(
            "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS={}&STYLES=\
             &CRS=EPSG:{}&BBOX={},{},{},{}&WIDTH={}&HEIGHT={}&FORMAT={}{}",
            self.url,
            self.layer,
            self.crs.epsg(),
            mm(b.min_e),
            mm(b.min_n),
            mm(b.max_e),
            mm(b.max_n),
            piece.width,
            piece.height,
            format,
            self.extra
        )
    }
}

/// The GetMap requests of the map `service` as PNG pieces covering `grid`: a map to lay under
/// the map as a template, not an orthophoto.
pub fn png_pieces(service: &WmsService, grid: &GridSpec) -> Vec<(GridSpec, String)> {
    pieces_as(service, grid, "image/png")
}

/// Where the orthophoto comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrthophotoSource {
    /// GetMap of a WMS layer.
    Wms(WmsService),
    /// GetCoverage of a WCS coverage (a WMS that stamps a watermark is no source).
    Wcs(WcsOrthophoto),
}

impl OrthophotoSource {
    /// `WMS` or `WCS`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Wms(_) => "WMS",
            Self::Wcs(_) => "WCS",
        }
    }

    fn url(&self) -> &str {
        match self {
            Self::Wms(s) => &s.url,
            Self::Wcs(s) => &s.url,
        }
    }

    fn max_px(&self) -> u32 {
        match self {
            Self::Wms(s) => s.max_px,
            Self::Wcs(s) => s.max_px,
        }
    }

    fn piece_url(&self, piece: &GridSpec) -> String {
        match self {
            Self::Wms(s) => s.piece_url(piece),
            Self::Wcs(s) => s.piece_url(piece),
        }
    }
}

/// The GetMap requests covering `grid` (in the service's CRS): pieces of at most `service.max_px`
/// per side, each with its own grid inside `grid`.
pub fn pieces(service: &WmsService, grid: &GridSpec) -> Vec<(GridSpec, String)> {
    source_pieces(&OrthophotoSource::Wms(service.clone()), grid)
}

/// The requests covering `grid` (in the source's CRS): pieces of at most the source's `max_px`
/// per side, each with its own grid inside `grid`.
pub fn source_pieces(source: &OrthophotoSource, grid: &GridSpec) -> Vec<(GridSpec, String)> {
    split_grid(source.max_px(), grid, |piece| source.piece_url(piece))
}

/// The GetMap requests of a WMS `service` in `format` covering `grid`.
fn pieces_as(service: &WmsService, grid: &GridSpec, format: &str) -> Vec<(GridSpec, String)> {
    split_grid(service.max_px, grid, |piece| {
        service.piece_url_as(piece, format)
    })
}

/// `grid` cut into pieces of at most `max` pixels per side, each with its request URL.
fn split_grid(
    max: u32,
    grid: &GridSpec,
    url_of: impl Fn(&GridSpec) -> String,
) -> Vec<(GridSpec, String)> {
    let starts = |len: u32| (0..len.div_ceil(max)).map(|i| i * max);
    let mut pieces = Vec::new();
    for x0 in starts(grid.width) {
        for y0 in starts(grid.height) {
            let piece = GridSpec {
                west: mm(grid.west + f64::from(x0) * grid.pixel_size),
                north: mm(grid.north - f64::from(y0) * grid.pixel_size),
                pixel_size: grid.pixel_size,
                width: max.min(grid.width - x0),
                height: max.min(grid.height - y0),
            };
            let url = url_of(&piece);
            pieces.push((piece, url));
        }
    }
    pieces
}

/// `v` rounded to millimetres, so that sums of 0.2 m pixels print without float noise.
pub fn mm(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// How much of an answer is read to find an XML exception message.
const XML_PROBE_BYTES: usize = 2000;

/// The text of a WMS ServiceException / `ows:ExceptionText` document, tags removed and white
/// space collapsed; `None` when `text` is not XML.
pub fn xml_message(text: &str) -> Option<String> {
    if !text.trim_start().starts_with('<') {
        return None;
    }
    let mut plain = String::new();
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                plain.push(' ');
            }
            _ if !in_tag => plain.push(c),
            _ => {}
        }
    }
    let message = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    let message = if message.is_empty() {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        message
    };
    Some(one_line(&message, 300))
}

/// Downloads the orthophoto over `bbox` (in the service's CRS) at 0.2 m from the WMS, stitched, as
/// `out` (JPEG) with its world file (`.jgw`), on the template grid of `bbox`.
pub fn fetch_orthophoto(
    fetcher: &dyn Fetcher,
    service: &WmsService,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<(), FetchError> {
    fetch_orthophoto_from(
        fetcher,
        &OrthophotoSource::Wms(service.clone()),
        bbox,
        root,
        out,
    )
}

/// [`fetch_orthophoto`] from a WMS or a WCS.
pub fn fetch_orthophoto_from(
    fetcher: &dyn Fetcher,
    source: &OrthophotoSource,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<(), FetchError> {
    let grid = GridSpec::covering(bbox, ORTHOPHOTO_PIXEL_M);
    let mut mosaic = image::RgbImage::new(grid.width, grid.height);
    for (piece, url) in source_pieces(source, &grid) {
        // A WMS reports errors as HTTP 200 with an XML document, a WCS with an exception report.
        // The inner `Err` is such an answer, which would come again, so it is not retried.
        let answer = get_with_retries(fetcher, &url, |bytes| {
            let refused = |message: String| FetchError::Http {
                url: url.clone(),
                message,
            };
            let exception = match source {
                OrthophotoSource::Wms(_) => xml_message(&String::from_utf8_lossy(
                    &bytes[..bytes.len().min(XML_PROBE_BYTES)],
                )),
                OrthophotoSource::Wcs(_) => is_ogc_report(bytes).then(|| exception_text(bytes)),
            };
            if let Some(message) = exception {
                return Ok(Err(refused(format!(
                    "the {} answered with an error: {message}",
                    source.kind()
                ))));
            }
            let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg)
                .map_err(|e| {
                    let start =
                        one_line(&String::from_utf8_lossy(&bytes[..bytes.len().min(80)]), 80);
                    refused(format!("not a JPEG ({e}): {start}"))
                })?
                .into_rgb8();
            if (image.width(), image.height()) != (piece.width, piece.height) {
                // The same size would come again: final, not retried.
                return Ok(Err(refused(format!(
                    "{} × {} pixels instead of {} × {}",
                    image.width(),
                    image.height(),
                    piece.width,
                    piece.height
                ))));
            }
            Ok(Ok(image))
        })?;
        let image = answer?;
        let (x0, y0) = (
            ((piece.west - grid.west) / grid.pixel_size).round() as i64,
            ((grid.north - piece.north) / grid.pixel_size).round() as i64,
        );
        image::imageops::replace(&mut mosaic, &image, x0, y0);
    }
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
        .encode(
            mosaic.as_raw(),
            mosaic.width(),
            mosaic.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| FetchError::Http {
            url: source.url().to_string(),
            message: e.to_string(),
        })?;
    write_inside(root, out, &jpeg)?;
    write_inside(
        root,
        &out.with_extension("jgw"),
        grid.world_file().as_bytes(),
    )?;
    Ok(())
}

/// Downloads the map `service` over `bbox` (in the CRS of the service), stitched from PNG pieces
/// at `pixel_size` (made coarser when the image would have more than `max_side_px` pixels on
/// its longest side), as `out` (PNG) with its world file (`.pgw`), on the template grid of
/// `bbox`.
pub fn fetch_png_template(
    fetcher: &dyn Fetcher,
    service: &WmsService,
    bbox: &BoundingBox,
    pixel_size: f64,
    max_side_px: f64,
    root: &Path,
    out: &Path,
) -> Result<(), FetchError> {
    let longest = bbox.width().max(bbox.height());
    let grid = GridSpec::covering(bbox, pixel_size.max(longest / max_side_px));
    let mut mosaic = image::RgbImage::from_pixel(grid.width, grid.height, image::Rgb([255; 3]));
    for (piece, url) in png_pieces(service, &grid) {
        // A WMS reports errors as HTTP 200 with an XML document; that would come again.
        let answer = get_with_retries(fetcher, &url, |bytes| {
            let refused = |message: String| FetchError::Http {
                url: url.clone(),
                message,
            };
            if let Some(message) = xml_message(&String::from_utf8_lossy(
                &bytes[..bytes.len().min(XML_PROBE_BYTES)],
            )) {
                return Ok(Err(refused(format!(
                    "the WMS answered with an error: {message}"
                ))));
            }
            let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                .map_err(|e| {
                    let start =
                        one_line(&String::from_utf8_lossy(&bytes[..bytes.len().min(80)]), 80);
                    refused(format!("not a PNG ({e}): {start}"))
                })?
                .into_rgb8();
            if (image.width(), image.height()) != (piece.width, piece.height) {
                return Ok(Err(refused(format!(
                    "{} × {} pixels instead of {} × {}",
                    image.width(),
                    image.height(),
                    piece.width,
                    piece.height
                ))));
            }
            Ok(Ok(image))
        })?;
        let image = answer?;
        let (x0, y0) = (
            ((piece.west - grid.west) / grid.pixel_size).round() as i64,
            ((grid.north - piece.north) / grid.pixel_size).round() as i64,
        );
        image::imageops::replace(&mut mosaic, &image, x0, y0);
    }
    let mut png = Vec::new();
    mosaic
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| FetchError::Http {
            url: service.url.to_string(),
            message: e.to_string(),
        })?;
    write_inside(root, out, &png)?;
    write_inside(
        root,
        &out.with_extension("pgw"),
        grid.world_file().as_bytes(),
    )?;
    Ok(())
}
