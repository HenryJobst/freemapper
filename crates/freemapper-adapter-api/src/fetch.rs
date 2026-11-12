//! The fetcher an adapter downloads through, and the tiles a fetch leaves in the cache.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::NaiveDate;
use freemapper_georef::tiles::TileId;

use crate::error::FetchError;
use crate::note::Note;
use crate::records::{Product, TileCurrency};

/// Source of downloads; HTTP in production, in-memory in tests.
pub trait Fetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;

    /// The `Last-Modified` date of `url`, if the server sends one.
    fn last_modified(&self, _url: &str) -> Result<Option<NaiveDate>, FetchError> {
        Ok(None)
    }

    /// Bytes `range` of `url`, for reading single members of very large archives.
    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let bytes = self.get(url)?;
        let len = bytes.len() as u64;
        let (start, end) = match range {
            ByteRange::From { start, len: n } => (start.min(len), (start + n).min(len)),
            ByteRange::Last(n) => (len.saturating_sub(n), len),
        };
        Ok(bytes[start as usize..end as usize].to_vec())
    }

    /// Downloads `url` into the file `path`, reporting the size; large files stream instead of
    /// being held in memory. More than `max_bytes` is [`FetchError::TooLarge`], and `path` is
    /// removed.
    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        let bytes = self.get(url)?;
        if bytes.len() as u64 > max_bytes {
            let _ = std::fs::remove_file(path);
            return Err(too_large(url, max_bytes));
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
        }
        let written = freemapper_fs::create_new_file(path).and_then(|mut f| f.write_all(&bytes));
        if let Err(e) = written {
            let _ = std::fs::remove_file(path);
            return Err(FetchError::Io(path.to_path_buf(), e));
        }
        progress(crate::note!(
            "progress-downloaded",
            megabytes = megabytes(bytes.len() as u64).to_string()
        ));
        Ok(())
    }
}

/// A part of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// `len` bytes from `start`.
    From { start: u64, len: u64 },
    /// The last `n` bytes.
    Last(u64),
}

/// [`FetchError::TooLarge`] for `url` over `max_bytes`.
pub fn too_large(url: &str, max_bytes: u64) -> FetchError {
    FetchError::TooLarge {
        url: url.to_string(),
        limit_mb: max_bytes.div_ceil(1024 * 1024),
    }
}

/// Whole megabytes in `bytes`, for the progress lines of a download.
pub fn megabytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

/// Whether a fetch may use the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    Online,
    /// Use the cache only.
    Offline,
}

/// A tile archive available in the cache.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedTile {
    pub product: Product,
    pub tile: TileId,
    pub url: String,
    pub currency: TileCurrency,
    pub zip: PathBuf,
}

/// An answer of HTTP `status` (400 or more) with `body` as [`FetchError`]: 404 is
/// [`FetchError::NotFound`], another 4xx [`FetchError::Refused`] with the text of an OGC
/// exception report in `body` (MapServer answers a WCS request outside its coverage with a 400
/// `ows:ExceptionReport`), a 5xx a (retried) [`FetchError::Http`] with the
/// words the HTTP client uses for a status (`http status: 503`).
pub fn status_answer_error(url: &str, status: u16, body: &[u8]) -> FetchError {
    match status {
        404 => FetchError::NotFound(url.to_string()),
        400..500 => FetchError::Refused {
            url: url.to_string(),
            status,
            detail: ogc_exception(body),
        },
        _ => FetchError::Http {
            url: url.to_string(),
            message: format!("http status: {status}"),
        },
    }
}

/// The code and text of an OGC (OWS or WMS service) exception report, as [`one_line`]s; `None`
/// for any other body, and for a report that names neither.
pub fn ogc_exception(body: &[u8]) -> Option<String> {
    if !is_ogc_report(body) {
        return None;
    }
    let text = String::from_utf8_lossy(body);
    let between = |open: &str, close: char| {
        let start = text.find(open)? + open.len();
        text[start..].split(close).next().map(str::trim)
    };
    let code = between("exceptionCode=\"", '"').or_else(|| between(" code=\"", '"'));
    // OWS `<ows:ExceptionText>…<`, WMS `<ServiceException code="…">…<`.
    let message = between("ExceptionText>", '<').or_else(|| {
        // Past the `<ServiceExceptionReport` element.
        let from = text.find("<ServiceExceptionReport").map_or(0, |at| at + 1);
        let rest = &text[from..];
        let rest = &rest[rest.find("<ServiceException")?..];
        let body = &rest[rest.find('>')? + 1..];
        body.split('<').next().map(str::trim)
    });
    let line = |s: &str| one_line(s, 300);
    match (code, message) {
        (Some(code), Some(message)) => Some(format!("{}: {}", line(code), line(message))),
        (Some(one), None) | (None, Some(one)) => Some(line(one)),
        (None, None) => None,
    }
}

/// `text` on one line of at most `max_chars` characters (then `…`): control characters become
/// spaces, bidi controls and zero-width characters (U+061C, U+200B–U+200F, U+202A–U+202E,
/// U+2060, U+2066–U+2069, U+FEFF) are dropped, runs of white space shrink to
/// one. Server text must not reach the terminal as escape sequences or reordered lines.
pub fn one_line(text: &str, max_chars: usize) -> String {
    let invisible = |c: char| {
        matches!(
            c,
            '\u{061C}'
                | '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2060}'
                | '\u{2066}'..='\u{2069}'
                | '\u{FEFF}'
        )
    };
    let printable: String = text
        .chars()
        .filter(|&c| !invisible(c))
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let line = printable.split_whitespace().collect::<Vec<_>>().join(" ");
    match line.char_indices().nth(max_chars) {
        Some((end, _)) => format!("{}…", &line[..end]),
        None => line,
    }
}

/// Whether the root element of `body` (after a BOM, an XML declaration, comments and a DOCTYPE) is an OGC
/// exception report: `ExceptionReport` (any namespace prefix) or `ServiceExceptionReport`.
pub fn is_ogc_report(body: &[u8]) -> bool {
    let mut rest = body.strip_prefix(b"\xef\xbb\xbf").unwrap_or(body);
    loop {
        rest = rest.trim_ascii_start();
        let (open, close): (&[u8], &[u8]) = if rest.starts_with(b"<?") {
            (b"<?", b"?>")
        } else if rest.starts_with(b"<!--") {
            (b"<!--", b"-->")
        } else if rest.starts_with(b"<!") {
            // A DOCTYPE; an internal subset in `[ … ]` may itself contain `>`.
            let end = match rest.iter().position(|&b| b == b'[') {
                Some(open) if rest[..open].iter().all(|&b| b != b'>') => {
                    rest.windows(2).position(|w| w == b"]>").map(|i| i + 2)
                }
                _ => rest.iter().position(|&b| b == b'>').map(|i| i + 1),
            };
            let Some(end) = end else { return false };
            rest = &rest[end..];
            continue;
        } else {
            break;
        };
        let inner = &rest[open.len()..];
        let Some(end) = inner.windows(close.len()).position(|w| w == close) else {
            return false;
        };
        rest = &inner[end + close.len()..];
    }
    let Some(tag) = rest.strip_prefix(b"<") else {
        return false;
    };
    let Some(end) = tag
        .iter()
        .position(|&b| b.is_ascii_whitespace() || b == b'>' || b == b'/')
    else {
        return false;
    };
    let name = &tag[..end];
    let local = name
        .iter()
        .rposition(|&b| b == b':')
        .map_or(name, |colon| &name[colon + 1..]);
    matches!(local, b"ExceptionReport" | b"ServiceExceptionReport")
}

/// Attempts per request to a server that drops connections now and then (gdi.berlin.de, GeoSN).
pub const ATTEMPTS: u32 = 3;
/// The pause before the second attempt; the third waits twice as long. None in tests.
#[cfg(not(any(test, feature = "testing")))]
const RETRY_DELAY: Duration = Duration::from_secs(2);
#[cfg(any(test, feature = "testing"))]
const RETRY_DELAY: Duration = Duration::ZERO;

/// Downloads `url` and reads it with `check`, up to [`ATTEMPTS`] times: server errors, resets
/// and answers `check` refuses (an error page with status 200) are retried; a missing URL or a
/// refused request is not, and neither is a write to a full disk.
pub fn get_with_retries<T>(
    fetcher: &dyn Fetcher,
    url: &str,
    check: impl Fn(&[u8]) -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    get_with_attempts(fetcher, url, ATTEMPTS, check)
}

/// [`get_with_retries`] for an answer of at most `max_bytes`: it streams into the file `part`
/// (removed afterwards) through [`Fetcher::download_to`], so a larger answer is stopped while it
/// is read, as [`FetchError::TooLarge`] (not asked again).
pub fn get_limited_with_retries<T>(
    fetcher: &dyn Fetcher,
    url: &str,
    part: &Path,
    max_bytes: u64,
    check: impl Fn(&[u8]) -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    retrying(ATTEMPTS, || {
        let read = fetcher
            .download_to(url, part, max_bytes, &mut |_| {})
            .and_then(|()| std::fs::read(part).map_err(|e| FetchError::Io(part.to_path_buf(), e)));
        let _ = std::fs::remove_file(part);
        read.and_then(|bytes| check(&bytes))
    })
}

/// [`get_with_retries`] with up to `attempts` tries.
pub fn get_with_attempts<T>(
    fetcher: &dyn Fetcher,
    url: &str,
    attempts: u32,
    check: impl Fn(&[u8]) -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    retrying(attempts, || {
        fetcher.get(url).and_then(|bytes| check(&bytes))
    })
}

/// `once` up to `attempts` times, like [`get_with_retries`].
fn retrying<T>(
    attempts: u32,
    mut once: impl FnMut() -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    let mut attempt = 1;
    loop {
        let result = once();
        match result {
            Err(
                FetchError::NotFound(_)
                | FetchError::Refused { .. }
                | FetchError::TooLarge { .. }
                | FetchError::Cancelled,
            ) => return result,
            // Asking again does not free disk space.
            Err(ref e) if e.is_disk_full() => return result,
            Err(_) if attempt < attempts => {
                std::thread::sleep(RETRY_DELAY * attempt);
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// [`Fetcher::download_to`] of `url` into `path` (at most `max_bytes`), tried up to
/// [`ATTEMPTS`] times like [`get_with_retries`]; only the transfer is retried, what was
/// downloaded is the caller's to check. A download over the limit is not retried.
pub fn download_with_retries(
    fetcher: &dyn Fetcher,
    url: &str,
    path: &Path,
    max_bytes: u64,
    progress: &mut dyn FnMut(Note),
) -> Result<(), FetchError> {
    let mut attempt = 1;
    loop {
        let result = fetcher.download_to(url, path, max_bytes, progress);
        match result {
            Err(
                FetchError::NotFound(_)
                | FetchError::Refused { .. }
                | FetchError::TooLarge { .. }
                | FetchError::Cancelled,
            ) => return result,
            Err(ref e) if e.is_disk_full() => return result,
            Err(_) if attempt < ATTEMPTS => {
                std::thread::sleep(RETRY_DELAY * attempt);
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// Bytes `range` of `url`, asked for up to three times like [`get_with_retries`]; an answer
/// of another length than asked for (a clamped or cut range) is an error.
pub fn get_range_with_retries(
    fetcher: &dyn Fetcher,
    url: &str,
    range: ByteRange,
) -> Result<Vec<u8>, FetchError> {
    let mut attempt = 1;
    loop {
        let result = fetcher.get_range(url, range).and_then(|bytes| {
            let fits = match range {
                ByteRange::From { len, .. } => bytes.len() as u64 == len,
                ByteRange::Last(n) => !bytes.is_empty() && bytes.len() as u64 <= n,
            };
            if fits {
                return Ok(bytes);
            }
            let asked = match range {
                ByteRange::From { start, len } => format!("{len} bytes from byte {start}"),
                ByteRange::Last(n) => format!("the last {n} bytes"),
            };
            Err(FetchError::Http {
                url: url.to_string(),
                message: format!("asked for {asked}, got {} bytes", bytes.len()),
            })
        });
        match result {
            Err(
                FetchError::NotFound(_)
                | FetchError::Refused { .. }
                | FetchError::RangeIgnored { .. }
                | FetchError::Cancelled,
            ) => return result,
            Err(_) if attempt < ATTEMPTS => {
                std::thread::sleep(RETRY_DELAY * attempt);
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// [`freemapper_fs::write_cache_file`] with the error as [`FetchError::Io`]; for the cache.
pub fn write_cache_file(path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    freemapper_fs::write_cache_file(path, bytes).map_err(|(path, e)| FetchError::Io(path, e))
}

/// [`crate::fs::write_inside`] with the error as [`FetchError::Io`]; for project files.
pub fn write_inside(root: &Path, path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    crate::fs::write_inside(root, path, bytes).map_err(|(path, e)| FetchError::Io(path, e))
}
