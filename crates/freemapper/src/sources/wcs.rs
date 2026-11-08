//! OGC WCS 2.0.1 GetCoverage of Float32 height models at 1 m (LVermGeo Saxony-Anhalt, LAiV
//! Mecklenburg-Vorpommern): the request, the answer (a `multipart/related` answer or a plain
//! TIFF), exception reports, and the mosaic of pieces.

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries, is_ogc_report, ogc_exception, one_line};
use crate::raster::Raster;
use crate::sources::float_tiff::decode_float_tiff;

/// A WCS serving one Float32 coverage at 1 m.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WcsService {
    /// The endpoint, without a query.
    pub url: &'static str,
    /// `COVERAGEID`.
    pub coverage: &'static str,
    /// Further parameters after `FORMAT=image/tiff`, e.g. `&GEOTIFF:COMPRESSION=LZW`.
    pub extra: &'static str,
    /// The TIFF comes in a `multipart/related` answer (LVermGeo), else as the whole answer
    /// (MapServer).
    pub multipart: bool,
    /// The coverage's extent; requests are cut to it.
    pub envelope: BoundingBox,
    /// Largest side of one GetCoverage in metres (= px).
    pub max_m: f64,
}

impl WcsService {
    /// GetCoverage over `bbox`, rounded outward to whole metres.
    pub fn get_coverage_url(&self, bbox: &BoundingBox) -> String {
        format!(
            "{}?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID={}&FORMAT=image/tiff{}\
             &SUBSET=x({},{})&SUBSET=y({},{})",
            self.url,
            self.coverage,
            self.extra,
            bbox.min_e.floor(),
            bbox.max_e.ceil(),
            bbox.min_n.floor(),
            bbox.max_n.ceil()
        )
    }

    /// The TIFF of an answer; an exception report (or anything else) is an error naming it.
    pub fn tiff<'a>(&self, bytes: &'a [u8]) -> Result<&'a [u8], String> {
        if self.multipart {
            multipart_tiff(bytes)
        } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
            Ok(bytes)
        } else {
            Err(format!(
                "no TIFF in the WCS answer: {}",
                exception_text(bytes)
            ))
        }
    }

    /// Fills `raster` (1 m) where it lies inside `within` and the envelope, in pieces of at most
    /// [`Self::max_m`]; each answer is pasted on its own grid (an answer clipped to the coverage
    /// fills only its part). Nothing is asked for when no part of `raster` lies inside.
    pub fn fill(
        &self,
        fetcher: &dyn Fetcher,
        raster: &mut Raster<f32>,
        within: &BoundingBox,
    ) -> Result<(), FetchError> {
        let Some(b) = raster
            .grid
            .bounds()
            .intersection(within)
            .and_then(|b| b.intersection(&self.envelope))
        else {
            return Ok(());
        };
        let step = self.max_m;
        let starts = |min: f64, max: f64| {
            let n = ((max - min) / step).ceil().max(1.0) as u32;
            (0..n).map(move |i| {
                (
                    min + f64::from(i) * step,
                    (min + f64::from(i + 1) * step).min(max),
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
                let url = self.get_coverage_url(&piece);
                let error = |message| FetchError::Http {
                    url: url.clone(),
                    message,
                };
                // The outer error is asked again; an exception report is final (the inner one).
                let part = get_with_retries(fetcher, &url, |bytes| match self.tiff(bytes) {
                    Ok(tiff) => decode_float_tiff(tiff).map(Ok).map_err(error),
                    Err(message) if is_ogc_report(bytes) => Ok(Err(error(message))),
                    Err(message) => Err(error(message)),
                })
                .and_then(|part| part)?;
                raster
                    .paste_with(&part.grid, |x, y| part.get(x, y))
                    .map_err(|e| FetchError::Http {
                        url: url.clone(),
                        message: e.to_string(),
                    })?;
            }
        }
        Ok(())
    }
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

/// What an error answer says: an OGC exception's code and text, else its start, on one clean
/// line.
pub(crate) fn exception_text(bytes: &[u8]) -> String {
    ogc_exception(bytes).unwrap_or_else(|| one_line(&String::from_utf8_lossy(bytes), 80))
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::raster::GridSpec;

    const SERVICE: WcsService = WcsService {
        url: "https://wcs.example/dgm",
        coverage: "dgm",
        extra: "",
        multipart: false,
        envelope: BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 10_000.0,
            max_n: 10_000.0,
        },
        max_m: 4000.0,
    };

    #[test]
    fn exception_text_is_one_clean_line() {
        // A multi-line MapServer report, with an escape sequence and a right-to-left override.
        let report = "<?xml version=\"1.0\"?>\n<ows:ExceptionReport xmlns:ows=\"http://www.opengis.net/ows/2.0\">\n  <ows:Exception exceptionCode=\"Extent\u{1b}[2JError\">\n    <ows:ExceptionText>msWCSGetCoverage20(): WCS\n      server error.\u{202E} Image\u{200B} extent</ows:ExceptionText>\n  </ows:Exception>\n</ows:ExceptionReport>";
        assert_eq!(
            exception_text(report.as_bytes()),
            "Extent [2JError: msWCSGetCoverage20(): WCS server error. Image extent"
        );
        // Anything else: its start, cleaned the same way.
        let page = format!(
            "<html>\u{1b}]0;owned\u{7}\n<body>{}</body></html>",
            "x".repeat(200)
        );
        let text = exception_text(page.as_bytes());
        assert!(text.starts_with("<html> ]0;owned <body>xxx"), "{text}");
        assert!(text.chars().all(|c| !c.is_control()), "{text:?}");
        assert!(text.chars().count() <= 81, "{text}");
    }

    #[test]
    fn exception_report_with_status_200_is_not_retried() {
        let grid = GridSpec {
            west: 100.0,
            north: 200.0,
            pixel_size: 1.0,
            width: 10,
            height: 10,
        };
        let report = br#"<ows:ExceptionReport><ows:Exception exceptionCode="InvalidSubsetting"><ows:ExceptionText>out</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#;
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(SERVICE.url, report.to_vec());
        let mut raster = Raster::filled(grid, f32::NAN);
        let err = SERVICE
            .fill(&fetcher, &mut raster, &grid.bounds())
            .unwrap_err()
            .to_string();
        assert!(err.contains("InvalidSubsetting: out"), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
        // An error page that is not a report may be a passing fault: asked again.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(SERVICE.url, b"<html>Proxy Error</html>".to_vec());
        assert!(SERVICE.fill(&fetcher, &mut raster, &grid.bounds()).is_err());
        assert_eq!(fetcher.calls.get(), 3);
        // Only the root element decides: a page that mentions a report is still retried.
        let page = b"<!DOCTYPE html><html><body><h1>502</h1><pre>&lt;ows:ExceptionReport&gt; \
            <ExceptionReport exceptionCode=\"X\"></pre></body></html>";
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(SERVICE.url, page.to_vec());
        assert!(SERVICE.fill(&fetcher, &mut raster, &grid.bounds()).is_err());
        assert_eq!(fetcher.calls.get(), 3);
        // A real report after a declaration and a comment is final.
        let report = br#"<?xml version="1.0" encoding="UTF-8"?>
<!-- MapServer version 8.0 -->
<ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/2.0" version="2.0.1"><ows:Exception exceptionCode="NoSuchCoverage"><ows:ExceptionText>mv_dgx</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#;
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(SERVICE.url, report.to_vec());
        let err = SERVICE
            .fill(&fetcher, &mut raster, &grid.bounds())
            .unwrap_err()
            .to_string();
        assert!(err.contains("NoSuchCoverage: mv_dgx"), "{err}");
        assert_eq!(fetcher.calls.get(), 1);
    }
}
