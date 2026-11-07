//! Orthophotos from a state's WMS (GeoSN, LVermGeo): GetMap pieces of at most 4000 px at 0.2 m,
//! stitched on the template grid of the map area.

use std::path::Path;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries, write_atomically};
use crate::raster::GridSpec;

/// Pixel size of the orthophoto template, as DOP20.
pub const ORTHOPHOTO_PIXEL_M: f64 = 0.2;
/// Largest side of one WMS request; the servers allow 4096.
const WMS_MAX_PX: u32 = 4000;
/// JPEG quality of the stitched orthophoto.
const JPEG_QUALITY: u8 = 90;

/// A WMS layer with the orthophoto, asked for in `crs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WmsService {
    pub url: &'static str,
    pub layer: &'static str,
    pub crs: Crs,
    /// More GetMap parameters, e.g. `&TIME=2026`.
    pub extra: &'static str,
}

/// The GetMap requests covering `grid` (in the service's CRS): pieces of at most [`WMS_MAX_PX`]
/// per side, each with its own grid inside `grid`.
pub fn pieces(service: &WmsService, grid: &GridSpec) -> Vec<(GridSpec, String)> {
    let starts = |len: u32| (0..len.div_ceil(WMS_MAX_PX)).map(|i| i * WMS_MAX_PX);
    let mut pieces = Vec::new();
    for x0 in starts(grid.width) {
        for y0 in starts(grid.height) {
            let piece = GridSpec {
                west: mm(grid.west + f64::from(x0) * grid.pixel_size),
                north: mm(grid.north - f64::from(y0) * grid.pixel_size),
                pixel_size: grid.pixel_size,
                width: WMS_MAX_PX.min(grid.width - x0),
                height: WMS_MAX_PX.min(grid.height - y0),
            };
            let b = piece.bounds();
            let url = format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS={}&STYLES=\
                 &CRS=EPSG:{}&BBOX={},{},{},{}&WIDTH={}&HEIGHT={}&FORMAT=image/jpeg{}",
                service.url,
                service.layer,
                service.crs.epsg(),
                mm(b.min_e),
                mm(b.min_n),
                mm(b.max_e),
                mm(b.max_n),
                piece.width,
                piece.height,
                service.extra
            );
            pieces.push((piece, url));
        }
    }
    pieces
}

/// `v` rounded to millimetres, so that sums of 0.2 m pixels print without float noise.
pub(crate) fn mm(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// Downloads the orthophoto over `bbox` (in the service's CRS) at 0.2 m from the WMS, stitched, as
/// `out` (JPEG) with its world file (`.jgw`), on the template grid of `bbox`.
pub fn fetch_orthophoto(
    fetcher: &dyn Fetcher,
    service: &WmsService,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<(), FetchError> {
    let grid = GridSpec::covering(bbox, ORTHOPHOTO_PIXEL_M);
    let mut mosaic = image::RgbImage::new(grid.width, grid.height);
    for (piece, url) in pieces(service, &grid) {
        let image = get_with_retries(fetcher, &url, |bytes| {
            let refused = |message: String| FetchError::Http {
                url: url.clone(),
                message,
            };
            let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg)
                .map_err(|e| {
                    let start = String::from_utf8_lossy(&bytes[..bytes.len().min(80)]).into_owned();
                    refused(format!("not a JPEG ({e}): {start}"))
                })?
                .into_rgb8();
            if (image.width(), image.height()) != (piece.width, piece.height) {
                return Err(refused(format!(
                    "{} × {} pixels instead of {} × {}",
                    image.width(),
                    image.height(),
                    piece.width,
                    piece.height
                )));
            }
            Ok(image)
        })?;
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
            url: service.url.to_string(),
            message: e.to_string(),
        })?;
    write_atomically(out, &jpeg)?;
    write_atomically(&out.with_extension("jgw"), grid.world_file().as_bytes())?;
    Ok(())
}
