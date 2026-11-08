//! Orthophotos from a state's WMS (GeoSN, LVermGeo): GetMap pieces of at most 4000 px at 0.2 m,
//! stitched on the template grid of the map area.

use std::path::Path;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries, write_atomically};
use crate::raster::GridSpec;

/// Pixel size of the orthophoto template, as DOP20.
pub const ORTHOPHOTO_PIXEL_M: f64 = 0.2;
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
    /// Largest side of one GetMap request in pixels (the server's MaxWidth/MaxHeight).
    pub max_px: u32,
}

/// The GetMap requests covering `grid` (in the service's CRS): pieces of at most `service.max_px`
/// per side, each with its own grid inside `grid`.
pub fn pieces(service: &WmsService, grid: &GridSpec) -> Vec<(GridSpec, String)> {
    let max = service.max_px;
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

/// How much of an answer is read to find an XML exception message.
const XML_PROBE_BYTES: usize = 2000;

/// The text of a WMS ServiceException / `ows:ExceptionText` document, tags removed and white
/// space collapsed; `None` when `text` is not XML.
fn xml_message(text: &str) -> Option<String> {
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
    Some(message.chars().take(300).collect())
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
        // A WMS reports errors as HTTP 200 with an XML document. The inner `Err` is such an
        // answer, which would come again, so it is not retried.
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
            url: service.url.to_string(),
            message: e.to_string(),
        })?;
    write_atomically(out, &jpeg)?;
    write_atomically(&out.with_extension("jgw"), grid.world_file().as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, solid_jpeg};
    use crate::sources::{geosn, hvbg, lgv, lvermgeo};

    fn grid(width: u32, height: u32) -> GridSpec {
        GridSpec {
            west: 476_000.0,
            north: 5_553_000.0,
            pixel_size: ORTHOPHOTO_PIXEL_M,
            width,
            height,
        }
    }

    #[test]
    fn hvbg_wms_pieces_are_at_most_3000_px() {
        let pieces = pieces(&hvbg::WMS, &grid(7000, 3500));
        let sizes: Vec<(u32, u32)> = pieces.iter().map(|(g, _)| (g.width, g.height)).collect();
        assert_eq!(
            sizes,
            [
                (3000, 3000),
                (3000, 500),
                (3000, 3000),
                (3000, 500),
                (1000, 3000),
                (1000, 500)
            ]
        );
        assert!(pieces[0].1.contains("LAYERS=he_dop20_rgb"));
        assert!(pieces[0].1.contains("CRS=EPSG:25832"));
        assert!(pieces[0].1.contains("WIDTH=3000&HEIGHT=3000"));
    }

    #[test]
    fn other_wms_pieces_stay_4000_px() {
        for service in [geosn::WMS, lvermgeo::WMS, lgv::WMS] {
            let pieces = pieces(&service, &grid(9000, 5000));
            assert_eq!(pieces.len(), 6, "{}", service.url);
            assert!(
                pieces
                    .iter()
                    .all(|(g, _)| g.width <= 4000 && g.height <= 4000),
                "{}",
                service.url
            );
            assert_eq!((pieces[0].0.width, pieces[0].0.height), (4000, 4000));
        }
    }

    fn bbox() -> BoundingBox {
        BoundingBox {
            min_e: 476_000.0,
            min_n: 5_552_800.0,
            max_e: 476_200.0,
            max_n: 5_553_000.0,
        }
    }

    #[test]
    fn xml_wms_answer_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            hvbg::WMS.url,
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ServiceExceptionReport version="1.3.0"><ServiceException code="InvalidDimensionValue">
msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be between 1 and 3000 pixels.
</ServiceException></ServiceExceptionReport>"#
                .to_vec(),
        );
        let error = fetch_orthophoto(&fetcher, &hvbg::WMS, &bbox(), &dir.path().join("o.jpg"))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("Image size out of range") && !error.contains("JPEG decode"),
            "{error}"
        );
        assert!(error.contains("the WMS answered with an error"), "{error}");
        // The same exception would come every time: asked once.
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn xml_wms_answer_names_the_inner_service_message() {
        let message = "msWMSLoadGetMapParams(): WMS server error. Image size out of range, \
                       WIDTH and HEIGHT must be between 1 and 3000 pixels.";
        let report = format!(
            "<?xml version='1.0' encoding=\"UTF-8\" standalone=\"no\" ?>\n\
             <!DOCTYPE ServiceExceptionReport SYSTEM \"http://schemas.opengis.net/wms/1.3.0/exception_1_3_0.dtd\">\n\
             <ServiceExceptionReport version=\"1.3.0\" xmlns=\"http://www.opengis.net/ogc\">\n\
             <ServiceException code=\"InvalidDimensionValue\">\n{message}\n</ServiceException>\n\
             </ServiceExceptionReport>"
        );
        assert_eq!(xml_message(&report).as_deref(), Some(message));
        let ows = "<ows:ExceptionReport><ows:Exception exceptionCode=\"X\">\
                   <ows:ExceptionText>Layer  not\n found</ows:ExceptionText></ows:Exception></ows:ExceptionReport>";
        assert_eq!(xml_message(ows).as_deref(), Some("Layer not found"));
        assert_eq!(xml_message("\u{89}PNG"), None);
    }

    #[test]
    fn non_jpeg_non_xml_answer_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        // Invalid UTF-8 becomes 3-byte replacement characters; byte 80 of that is mid-character.
        let mut png = vec![0x89];
        png.extend(b"PNG\r\n\x1a\n");
        png.extend([0x89u8; 60]);
        fetcher.insert_prefix(hvbg::WMS.url, png);
        let error = fetch_orthophoto(&fetcher, &hvbg::WMS, &bbox(), &dir.path().join("o.jpg"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("not a JPEG"), "{error}");
        assert!(!error.contains("the WMS answered"), "{error}");
        assert!(error.contains("not a JPEG ("), "{error}");
    }

    #[test]
    fn stalled_wms_request_times_out_and_is_retried() {
        use std::io::{Read as _, Write as _};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url: &'static str =
            Box::leak(format!("http://{}/wms", listener.local_addr().unwrap()).into_boxed_str());
        let jpeg = solid_jpeg(1000, 1000, [1, 2, 3]);
        std::thread::spawn(move || {
            // The first connection is held without an answer; the second gets the image.
            let (_stalled, _) = listener.accept().unwrap();
            let (mut conn, _) = listener.accept().unwrap();
            let mut request = [0u8; 4096];
            let _ = conn.read(&mut request);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                jpeg.len()
            );
            conn.write_all(head.as_bytes()).unwrap();
            conn.write_all(&jpeg).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));
        });
        let service = WmsService { url, ..hvbg::WMS };
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("o.jpg");
        let fetcher =
            crate::fetch::HttpFetcher::with_timeout(std::time::Duration::from_millis(300));
        let (sender, receiver) = std::sync::mpsc::channel();
        let path = out.clone();
        std::thread::spawn(move || {
            sender.send(fetch_orthophoto(&fetcher, &service, &bbox(), &path))
        });
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the stalled request did not time out")
            .unwrap();
        assert!(out.exists());
    }
}
