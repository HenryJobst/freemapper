//! OGC WCS 2.0.1 GetCoverage of Float32 height models at 1 m (LVermGeo Saxony-Anhalt, LAiV
//! Mecklenburg-Vorpommern): the request, the answer (a `multipart/related` answer or a plain
//! TIFF), exception reports, and the mosaic of pieces.

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries};
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
                let part = get_with_retries(fetcher, &url, |bytes| {
                    self.tiff(bytes)
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

/// What an error answer says: an OWS exception's code and text, else its start.
pub(crate) fn exception_text(bytes: &[u8]) -> String {
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
