//! Tile download with a shared on-disk cache keyed by tile and publication date.

use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;

use crate::sources::alkis::AlkisError;
use crate::sources::lgb::ndsm_tile_url;
use crate::sources::lgb::{CurrencyCsvError, Product, TileCurrency, parse_currency_csv};
use crate::sources::region::Provider;
use crate::space::{DiskSpace, SystemDiskSpace};
use crate::tiles::TileId;

/// Largest download accepted; LGB DOP20 tiles are about 8 MB.
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("download of {url} failed: {message}")]
    Http { url: String, message: String },
    #[error("{}", crate::space::io_error_text(.0, .1))]
    Io(PathBuf, std::io::Error),
    #[error("download of {url} is not a complete tile archive: {message}")]
    CorruptArchive { url: String, message: String },
    #[error("download of {url} is not a complete OSM extract: {message}")]
    CorruptExtract { url: String, message: String },
    #[error(
        "{} not covered by the published tiles; reduce the map area or buffer_m in freemapper.toml",
        tiles_are(.0)
    )]
    TilesNotCovered(Vec<TileId>),
    #[error(
        "offline and not cached: {product} {tiles}; run `freemapper fetch` online",
        product = .0,
        tiles = tile_list(.1)
    )]
    MissingOffline(Product, Vec<TileId>),
    #[error(transparent)]
    CurrencyCsv(#[from] CurrencyCsvError),
    #[error(transparent)]
    Alkis(#[from] AlkisError),
    #[error("no OSM extract cached; run `freemapper fetch` online")]
    OsmExtractMissing,
    #[error("{0} not found on the server")]
    NotFound(String),
    /// The server refused the request (HTTP 4xx other than 404); asking again does not help.
    /// `detail` is what an OGC exception report in the answer says.
    #[error(
        "{url}: the server refused the request (HTTP {status}){}",
        detail.as_deref().map(|d| format!(": {d}")).unwrap_or_default()
    )]
    Refused {
        url: String,
        status: u16,
        detail: Option<String>,
    },
    /// A streamed download grew past its limit (a misbehaving server); asking again does not
    /// help.
    #[error("download of {url} is larger than {limit_mb} MB; stopped")]
    TooLarge { url: String, limit_mb: u64 },
    /// Offline: the model `label` stored at `path` is missing, cannot be read or does not cover
    /// the map area.
    #[error(
        "no {label} stored for this map area: {path} is missing, cannot be read or does not cover it; run `freemapper fetch` online first",
        path = .path.display()
    )]
    NotStored { label: String, path: PathBuf },
    /// The server answered a byte range with the whole file; asking again does not help.
    #[error("{url}: the server ignored the byte range {range} (HTTP {status})")]
    RangeIgnored {
        url: String,
        range: String,
        status: u16,
    },
}

impl FetchError {
    /// Whether writing failed because the volume is full.
    pub fn is_disk_full(&self) -> bool {
        matches!(self, FetchError::Io(_, e) if crate::space::is_storage_full(e))
    }
}

pub(crate) fn tile_names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(TileId::name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `tile X` or `tiles X, Y`.
fn tile_list(tiles: &[TileId]) -> String {
    let noun = if tiles.len() == 1 { "tile" } else { "tiles" };
    format!("{noun} {}", tile_names(tiles))
}

/// `tile X is` or `tiles X, Y are`.
fn tiles_are(tiles: &[TileId]) -> String {
    let verb = if tiles.len() == 1 { "is" } else { "are" };
    format!("{} {verb}", tile_list(tiles))
}

/// Partial files older than this are left over from interrupted runs.
pub(crate) const STALE_PART_AGE: Duration = Duration::from_secs(3600);

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
        progress: &mut dyn FnMut(&str),
    ) -> Result<(), FetchError> {
        let bytes = self.get(url)?;
        if bytes.len() as u64 > max_bytes {
            let _ = std::fs::remove_file(path);
            return Err(too_large(url, max_bytes));
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
        }
        let written = crate::fsutil::create_new_file(path).and_then(|mut f| f.write_all(&bytes));
        if let Err(e) = written {
            let _ = std::fs::remove_file(path);
            return Err(FetchError::Io(path.to_path_buf(), e));
        }
        progress(&format!("downloaded {} MB", megabytes(bytes.len() as u64)));
        Ok(())
    }
}

/// Streams `reader`, the body of `url` (`total` bytes, if the server says), into the file `path`
/// that `create` opens, at most `max_bytes`, reporting the size.
fn stream_to_file(
    url: &str,
    reader: &mut dyn std::io::Read,
    total: Option<u64>,
    path: &Path,
    create: &dyn Fn(&Path) -> std::io::Result<Box<dyn std::io::Write>>,
    max_bytes: u64,
    progress: &mut dyn FnMut(&str),
) -> Result<(), FetchError> {
    let result = stream(url, reader, total, path, create, max_bytes, progress);
    // A partial file is of no use; on a full disk it also takes the space others need.
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

/// The work of [`stream_to_file`], which removes `path` when this fails.
fn stream(
    url: &str,
    reader: &mut dyn std::io::Read,
    total: Option<u64>,
    path: &Path,
    create: &dyn Fn(&Path) -> std::io::Result<Box<dyn std::io::Write>>,
    max_bytes: u64,
    progress: &mut dyn FnMut(&str),
) -> Result<(), FetchError> {
    let mut file = create(path).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    let mut buffer = vec![0u8; 1 << 16];
    let (mut done, mut reported) = (0u64, 0u64);
    loop {
        let n = reader.read(&mut buffer).map_err(|e| FetchError::Http {
            url: url.to_string(),
            message: e.to_string(),
        })?;
        if n == 0 {
            break;
        }
        if done + n as u64 > max_bytes {
            return Err(too_large(url, max_bytes));
        }
        file.write_all(&buffer[..n])
            .map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
        done += n as u64;
        if done - reported >= PROGRESS_STEP_BYTES {
            reported = done;
            let of = total
                .map(|t| format!(" of {} MB", megabytes(t)))
                .unwrap_or_default();
            progress(&format!("downloaded {} MB{of}", megabytes(done)));
        }
    }
    file.flush()
        .map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    progress(&format!("downloaded {} MB", megabytes(done)));
    Ok(())
}

/// A part of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// `len` bytes from `start`.
    From { start: u64, len: u64 },
    /// The last `n` bytes.
    Last(u64),
}

/// A redirect answer as an error. Only the agents of [`PORTAL_HOST`] return one: they follow
/// none, as a followed redirect would keep the portal's pinned trust.
fn refuse_redirect(
    url: &str,
    response: &ureq::http::Response<ureq::Body>,
) -> Result<(), FetchError> {
    let status = response.status();
    if !status.is_redirection() {
        return Ok(());
    }
    let location: String = response
        .headers()
        .get("location")
        .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
        .unwrap_or_else(|| "nowhere".to_string())
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect();
    Err(FetchError::Http {
        url: url.to_string(),
        message: format!(
            "{PORTAL_HOST} answered with a redirect (HTTP {}) to {location}; freemapper does not \
             follow redirects of this host, because it pins the host's certificates",
            status.as_u16()
        ),
    })
}

/// [`FetchError::TooLarge`] for `url` over `max_bytes`.
fn too_large(url: &str, max_bytes: u64) -> FetchError {
    FetchError::TooLarge {
        url: url.to_string(),
        limit_mb: max_bytes.div_ceil(1024 * 1024),
    }
}

/// A ureq error as [`FetchError`]; 404 becomes [`FetchError::NotFound`].
fn status_error(url: &str, e: ureq::Error) -> FetchError {
    match e {
        ureq::Error::StatusCode(status) => status_answer_error(url, status, b""),
        e => FetchError::Http {
            url: url.to_string(),
            message: e.to_string(),
        },
    }
}

/// An answer of HTTP `status` (400 or more) with `body` as [`FetchError`]: 404 is
/// [`FetchError::NotFound`], another 4xx [`FetchError::Refused`] with the text of an OGC
/// exception report in `body` (MapServer answers a WCS request outside its coverage with a 400
/// `ows:ExceptionReport`), a 5xx a (retried) [`FetchError::Http`].
pub(crate) fn status_answer_error(url: &str, status: u16, body: &[u8]) -> FetchError {
    match status {
        404 => FetchError::NotFound(url.to_string()),
        400..500 => FetchError::Refused {
            url: url.to_string(),
            status,
            detail: ogc_exception(body),
        },
        _ => FetchError::Http {
            url: url.to_string(),
            message: ureq::Error::StatusCode(status).to_string(),
        },
    }
}

/// The code and text of an OGC (OWS or WMS service) exception report, as [`one_line`]s; `None`
/// for any other body, and for a report that names neither.
pub(crate) fn ogc_exception(body: &[u8]) -> Option<String> {
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
pub(crate) fn one_line(text: &str, max_chars: usize) -> String {
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
pub(crate) fn is_ogc_report(body: &[u8]) -> bool {
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

/// Largest error answer read for its exception report.
const MAX_ERROR_BODY_BYTES: u64 = 64 * 1024;

/// Identifies freemapper to the servers (Nominatim requires it).
const USER_AGENT: &str = concat!(
    "freemapper/",
    env!("CARGO_PKG_VERSION"),
    " (+",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

/// Time to open a connection and to wait for the response headers.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);
/// Time to receive a whole body; an 8 MB tile at 128 kbit/s takes about 9 minutes.
const BODY_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Progress is reported after each step of a large download.
const PROGRESS_STEP_BYTES: u64 = 25 * 1024 * 1024;

fn megabytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

/// Time to receive a large file such as the OSM extract (300 MB at 1 Mbit/s: 40 minutes).
const LARGE_BODY_TIMEOUT: Duration = Duration::from_secs(2 * 3600);

/// The LVermGeo SH Downloadportal. It sends its leaf certificate without the intermediate
/// `D-TRUST BR CA 1-20-1 2020`, and rustls does not fetch missing intermediates, so this host
/// is verified against the bundled intermediate and its root instead of the default roots.
///
/// The agents of this host follow no redirects: a followed redirect would keep the trust of the
/// URL it started from (a redirect away from the portal checked against the D-TRUST anchors, one
/// to the portal against the default roots). A redirect answer is an error naming this host.
const PORTAL_HOST: &str = "geodaten.schleswig-holstein.de";

/// The only trust anchors for [`PORTAL_HOST`]: the intermediate `D-TRUST BR CA 1-20-1 2020`
/// (valid to 2035-02-11) and the root `D-TRUST BR Root CA 1 2020` that signed it.
///
/// The intermediate is a trust anchor because the server omits it from its chain. As an anchor,
/// its own validity period, extended key usage and revocation are not checked, and the pin does
/// not follow a later distrust of the root in the default roots. This applies to that one host
/// only; the leaf is still fully verified (chain, validity and host name). When the portal
/// changes its CA or the intermediate expires, its requests fail with a certificate error until
/// `assets/tls/` is updated.
const PORTAL_ANCHORS: [&[u8]; 2] = [
    include_bytes!("../../../assets/tls/d-trust-br-ca-1-20-1-2020.pem"),
    include_bytes!("../../../assets/tls/d-trust-br-root-ca-1-2020.pem"),
];

/// Which certificates a URL is verified against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trust {
    /// ureq's default (webpki) roots, for every host but the portal.
    DefaultRoots,
    /// Only the bundled [`PORTAL_ANCHORS`].
    PortalAnchors,
}

/// The trust for `url`: the bundled anchors exactly for absolute URLs on the host
/// [`PORTAL_HOST`], the default roots for everything else, including URLs that do not parse.
/// Hosts compare without case, and a single trailing dot (the fully qualified form of the same
/// name, as a pasted URL may carry) is the same host.
fn trust_for(url: &str) -> Trust {
    let Ok(uri) = url.parse::<ureq::http::Uri>() else {
        return Trust::DefaultRoots;
    };
    let host = uri
        .host()
        .map(|host| host.strip_suffix('.').unwrap_or(host));
    match (uri.scheme(), host) {
        (Some(_), Some(host)) if host.eq_ignore_ascii_case(PORTAL_HOST) => Trust::PortalAnchors,
        _ => Trust::DefaultRoots,
    }
}

fn portal_anchors() -> Vec<ureq::tls::Certificate<'static>> {
    PORTAL_ANCHORS
        .iter()
        .map(|pem| ureq::tls::Certificate::from_pem(pem).expect("bundled certificates are PEM"))
        .collect()
}

/// An agent for ordinary requests and one for large bodies.
struct Agents {
    normal: ureq::Agent,
    large: ureq::Agent,
}

/// Downloads over HTTPS; a stalled server fails with a timeout instead of hanging.
pub struct HttpFetcher {
    default: Agents,
    portal: Agents,
}

impl HttpFetcher {
    pub fn new() -> Self {
        Self::with_timeouts(CONNECT_TIMEOUT, RESPONSE_TIMEOUT, BODY_TIMEOUT)
    }

    /// The same timeout for connecting, the response headers and the body.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_timeouts(timeout, timeout, timeout)
    }

    fn with_timeouts(connect: Duration, response: Duration, body: Duration) -> Self {
        let agent = |body: Duration, trust: Trust| -> ureq::Agent {
            let config = ureq::Agent::config_builder()
                .timeout_connect(Some(connect))
                .timeout_recv_response(Some(response))
                .timeout_recv_body(Some(body));
            match trust {
                Trust::DefaultRoots => config,
                Trust::PortalAnchors => config.max_redirects(0).tls_config(
                    ureq::tls::TlsConfig::builder()
                        .root_certs(ureq::tls::RootCerts::new_with_certs(&portal_anchors()))
                        .build(),
                ),
            }
            .build()
            .into()
        };
        let agents = |trust: Trust| Agents {
            normal: agent(body, trust),
            large: agent(LARGE_BODY_TIMEOUT.max(body), trust),
        };
        Self {
            default: agents(Trust::DefaultRoots),
            portal: agents(Trust::PortalAnchors),
        }
    }

    /// The agents verifying `url`'s host.
    fn agents(&self, url: &str) -> &Agents {
        match trust_for(url) {
            Trust::DefaultRoots => &self.default,
            Trust::PortalAnchors => &self.portal,
        }
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for HttpFetcher {
    /// Asks for `range` only; a server that answers with the whole file (not 206) is refused
    /// before its body is read, so a 1 GB archive is never downloaded by accident.
    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let (header, limit) = match range {
            ByteRange::From { len: 0, .. } | ByteRange::Last(0) => return Ok(Vec::new()),
            ByteRange::From { start, len } => (format!("bytes={start}-{}", start + len - 1), len),
            ByteRange::Last(n) => (format!("bytes=-{n}"), n),
        };
        let mut response = self
            .agents(url)
            .normal
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Range", &header)
            .call()
            .map_err(|e| status_error(url, e))?;
        refuse_redirect(url, &response)?;
        if response.status() != 206 {
            return Err(FetchError::RangeIgnored {
                url: url.to_string(),
                range: header,
                status: response.status().as_u16(),
            });
        }
        // One byte more than asked, so an oversized answer shows.
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| FetchError::Http {
                url: url.to_string(),
                message: e.to_string(),
            })?;
        Ok(bytes)
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(&str),
    ) -> Result<(), FetchError> {
        let http_error = |message: String| FetchError::Http {
            url: url.to_string(),
            message,
        };
        let mut response = self
            .agents(url)
            .large
            .get(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| http_error(e.to_string()))?;
        refuse_redirect(url, &response)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
        }
        let total = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        if total.is_some_and(|total| total > max_bytes) {
            let _ = std::fs::remove_file(path);
            return Err(too_large(url, max_bytes));
        }
        let create = |path: &Path| -> std::io::Result<Box<dyn std::io::Write>> {
            Ok(Box::new(crate::fsutil::create_new_file(path)?))
        };
        let mut reader = response.body_mut().as_reader();
        stream_to_file(url, &mut reader, total, path, &create, max_bytes, progress)
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        let response = self
            .agents(url)
            .normal
            .head(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| status_error(url, e))?;
        refuse_redirect(url, &response)?;
        Ok(response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|time| time.date_naive()))
    }

    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let http_error = |e: ureq::Error| status_error(url, e);
        let mut response = self
            .agents(url)
            .normal
            .get(url)
            .header("User-Agent", USER_AGENT)
            // An error answer is read for its exception report.
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(http_error)?;
        refuse_redirect(url, &response)?;
        let status = response.status().as_u16();
        if status >= 400 {
            let mut body = Vec::new();
            // The report is a courtesy; a body that does not arrive leaves it out.
            let _ = response
                .body_mut()
                .as_reader()
                .take(MAX_ERROR_BODY_BYTES)
                .read_to_end(&mut body);
            return Err(status_answer_error(url, status, &body));
        }
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_DOWNLOAD_BYTES)
            .read_to_end(&mut bytes)
            .map_err(|e| FetchError::Http {
                url: url.to_string(),
                message: e.to_string(),
            })?;
        Ok(bytes)
    }
}

/// Tile cache shared by all projects. It also answers how much space the volumes freemapper
/// writes to have left (the system's free space, unless replaced).
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
    space: Arc<dyn DiskSpace>,
}

impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            space: Arc::new(SystemDiskSpace),
        }
    }

    /// The cache with `space` as the source of free space (tests simulate a full disk).
    pub fn with_disk_space(self, space: Arc<dyn DiskSpace>) -> Self {
        Self { space, ..self }
    }

    /// The folder of the cache.
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn disk_space(&self) -> &dyn DiskSpace {
        self.space.as_ref()
    }

    /// `<system cache dir>/freemapper`, if the system has a cache directory.
    pub fn default_location() -> Option<Self> {
        dirs::cache_dir().map(|dir| Self::new(dir.join("freemapper")))
    }

    fn product_dir(&self, product: Product) -> PathBuf {
        self.root
            .join(product.provider_dir_name())
            .join(product.cache_dir_name())
    }

    pub fn tile_path(&self, product: Product, tile: TileId, published: NaiveDate) -> PathBuf {
        self.product_dir(product)
            .join(format!("{}_{published}.zip", tile.name()))
    }

    /// Folder of the cached nDSM point clouds.
    pub fn ndsm_dir(&self) -> PathBuf {
        self.root.join("lgb").join("ndom")
    }

    /// Cached nDSM file of `tile`, named after the server date (`<tile>.laz` when undated).
    pub fn ndsm_path(&self, tile: TileId, published: Option<NaiveDate>) -> PathBuf {
        self.ndsm_dir().join(match published {
            Some(date) => format!("{}_{date}.laz", tile.name()),
            None => format!("{}.laz", tile.name()),
        })
    }

    /// Whether some publication of `tile` of `product` is cached.
    pub fn has_tile(&self, product: Product, tile: TileId) -> bool {
        let probe = self.tile_path(product, tile, NaiveDate::MIN);
        !cached_versions(&probe, tile).is_empty()
    }

    /// Whether an nDSM point cloud of `tile` is cached.
    pub fn has_ndsm(&self, tile: TileId) -> bool {
        !cached_ndsm(&self.ndsm_dir(), tile).is_empty()
    }

    /// Folder of the cached OSM extracts.
    pub fn osm_dir(&self) -> PathBuf {
        self.root.join("osm")
    }

    pub fn currency_csv_path(&self, product: Product) -> PathBuf {
        self.product_dir(product).join("currency.csv")
    }

    /// Folder of the LGV Hamburg archive members.
    pub fn lgv_dir(&self) -> PathBuf {
        self.root.join("lgv")
    }

    /// Folder of the HVBG Hesse tiles, zip directories and metadata list.
    pub fn hvbg_dir(&self) -> PathBuf {
        self.root.join("hvbg")
    }

    /// Folder of the LVermGeo SH tiles (DGM1, the bDOM reduced to 1 m) and tile lists.
    pub fn lvermgeo_sh_dir(&self) -> PathBuf {
        self.root.join("lvermgeo_sh")
    }

    /// Folder of the Geobasis NRW DGM1/DOM1 tiles and the kept tile lists.
    pub fn geobasis_nrw_dir(&self) -> PathBuf {
        self.root.join("geobasis_nrw")
    }

    /// Folder of the LVermGeo RP DGM1/DOM1 tiles and the kept tile lists (per area).
    pub fn lvermgeo_rp_dir(&self) -> PathBuf {
        self.root.join("lvermgeo_rp")
    }

    /// Folder of the LAiV M-V tile lists (`overview.php` answers), per area.
    pub fn laiv_mv_dir(&self) -> PathBuf {
        self.root.join("laiv_mv")
    }

    /// The collected GeoSN tile index rows of `product`.
    pub fn currency_index_path(&self, product: Product) -> PathBuf {
        self.product_dir(product).join("currency.json")
    }
}

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

/// Makes `tiles` of `product` available in `cache` and returns them in the given order.
///
/// Online, the currency CSV is downloaded on every call and a tile is downloaded only when the
/// cache lacks its current publication. Offline, the cached CSV and archives are used.
pub fn fetch_tiles(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<FetchedTile>, FetchError> {
    let table = currency_table(fetcher, cache, product, tiles, mode, progress)?;
    let uncovered: Vec<TileId> = tiles
        .iter()
        .copied()
        .filter(|tile| !table.contains_key(tile))
        .collect();
    if !uncovered.is_empty() {
        return Err(not_in_table(product, mode, uncovered));
    }
    let mut fetched = Vec::new();
    let mut missing = Vec::new();
    for &tile in tiles {
        match fetch_tile(
            fetcher,
            cache,
            product,
            tile,
            table[&tile].clone(),
            mode,
            progress,
        )? {
            Some(tile) => fetched.push(tile),
            None => missing.push(tile),
        }
    }
    if !missing.is_empty() {
        return Err(FetchError::MissingOffline(product, missing));
    }
    Ok(fetched)
}

/// One tile of [`fetch_tiles_each`] and how fetching it went.
pub type TileOutcome = (TileId, Result<FetchedTile, FetchError>);

/// Like [`fetch_tiles`], but every tile succeeds or fails on its own; only a currency CSV that
/// cannot be read fails the whole call.
pub fn fetch_tiles_each(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<TileOutcome>, FetchError> {
    let table = currency_table(fetcher, cache, product, tiles, mode, progress)?;
    Ok(tiles
        .iter()
        .map(|&tile| {
            let result = match table.get(&tile) {
                None => Err(not_in_table(product, mode, vec![tile])),
                Some(entry) => {
                    fetch_tile(fetcher, cache, product, tile, entry.clone(), mode, progress)
                        .and_then(|fetched| {
                            fetched.ok_or(FetchError::MissingOffline(product, vec![tile]))
                        })
                }
            };
            (tile, result)
        })
        .collect())
}

/// Tiles missing from the currency table: not published, or (GeoSN offline, whose cached index
/// only holds the areas asked for before) not fetched yet.
fn not_in_table(product: Product, mode: FetchMode, tiles: Vec<TileId>) -> FetchError {
    if mode == FetchMode::Offline && product.provider() == Provider::GeoSn {
        FetchError::MissingOffline(product, tiles)
    } else {
        FetchError::TilesNotCovered(tiles)
    }
}

/// Currency and download URL of each tile a provider publishes.
type CurrencyTable = HashMap<TileId, (TileCurrency, String)>;

/// The currency table of `product`, from its provider: the LGB currency CSV or the GeoSN tile
/// index. Online the answer replaces (CSV) or extends (index) the cached copy once it parses;
/// offline the cached copy is used.
fn currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    match product.provider() {
        Provider::Lgb => lgb_currency_table(fetcher, cache, product, tiles, mode, progress),
        Provider::GeoSn => geosn_currency_table(fetcher, cache, product, tiles, mode, progress),
        // LVermGeo rasters are fetched per area (WMS, WCS), not as tiles.
        Provider::LvermGeo
        | Provider::Lgv
        | Provider::Hvbg
        | Provider::LvermGeoSh
        | Provider::LaivMv
        | Provider::GeobasisNrw
        | Provider::Lgln
        | Provider::LvermGeoRp => Err(FetchError::TilesNotCovered(tiles.to_vec())),
    }
}

fn lgb_currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    let csv_path = cache.currency_csv_path(product);
    let csv = match mode {
        FetchMode::Online => {
            let url = product.currency_csv_url();
            progress(&format!("downloading {url}"));
            fetcher.get(url)?
        }
        FetchMode::Offline => match std::fs::read(&csv_path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(FetchError::MissingOffline(product, tiles.to_vec())),
        },
    };
    let table = parse_currency_csv(&String::from_utf8_lossy(&csv))?;
    if mode == FetchMode::Online {
        // Only a CSV that parsed replaces the cached one, so `--offline` keeps working.
        write_cache_file(&csv_path, &csv)?;
    }
    Ok(table
        .into_iter()
        .map(|(tile, currency)| (tile, (currency, product.tile_url(tile))))
        .collect())
}

/// The GeoSN index rows of `tiles`. The index is queried per area, so the cached copy collects
/// the rows of every query (`{"33414-5654": ["2024-11-30", "https://…"], …}`).
fn geosn_currency_table(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<CurrencyTable, FetchError> {
    let path = cache.currency_index_path(product);
    let mut cached: std::collections::BTreeMap<String, (NaiveDate, String)> = std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    if mode == FetchMode::Online {
        let url = crate::sources::geosn::index_url(product, tiles);
        progress(&format!("downloading the {product} tile index"));
        let rows = get_with_retries(fetcher, &url, |answer| {
            crate::sources::geosn::parse_index(&String::from_utf8_lossy(answer)).map_err(
                |message| FetchError::Http {
                    url: url.clone(),
                    message,
                },
            )
        })?;
        for row in rows {
            let download = row.url.unwrap_or_else(|| product.tile_url(row.tile));
            cached.insert(row.tile.name(), (row.currency, download));
        }
        let json = serde_json::to_vec(&cached).expect("index table serialises");
        write_cache_file(&path, &json)?;
    } else if cached.is_empty() {
        return Err(FetchError::MissingOffline(product, tiles.to_vec()));
    }
    Ok(cached
        .into_iter()
        .filter_map(|(name, (date, url))| {
            let currency = TileCurrency {
                created: date,
                revised: None,
                published: date,
            };
            Some((TileId::parse(&name)?, (currency, url)))
        })
        .collect())
}

/// One tile of `product` in the cache; `None` offline when no version of it is cached.
fn fetch_tile(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    product: Product,
    tile: TileId,
    (mut currency, url): (TileCurrency, String),
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<FetchedTile>, FetchError> {
    let mut zip = cache.tile_path(product, tile, currency.published);
    if zip.exists() {
        progress(&format!("cached {}", tile.name()));
    } else if mode == FetchMode::Offline {
        match newest_cached_version(&zip, tile) {
            Some((published, older)) => {
                progress(&format!(
                    "warning: using cached {product} tile {} from {published} (newer {} not downloaded)",
                    tile.name(),
                    currency.published
                ));
                currency.published = published;
                zip = older;
            }
            None => return Ok(None),
        }
    } else {
        progress(&format!("downloading {url}"));
        let bytes = fetcher.get(&url)?;
        check_archive(&bytes, product, tile).map_err(|message| FetchError::CorruptArchive {
            url: url.clone(),
            message,
        })?;
        write_cache_file(&zip, &bytes)?;
        remove_older_versions(&zip, tile)?;
        crate::fsutil::remove_stale_parts(zip.parent().expect("cache paths"), STALE_PART_AGE);
    }
    Ok(Some(FetchedTile {
        product,
        tile,
        url,
        currency,
        zip,
    }))
}

/// A cached nDSM point cloud of one tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NdsmTile {
    pub tile: TileId,
    pub url: String,
    /// The server's `Last-Modified` date, if it sends one.
    pub published: Option<NaiveDate>,
    pub path: PathBuf,
}

/// Makes the nDSM point clouds of `tiles` available in `cache`. A tile is downloaded when the
/// server's `Last-Modified` date has no cached file yet; tiles the server does not have are
/// skipped with a warning. Offline, the newest cached file per tile is used.
pub fn fetch_ndsm_tiles(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    tiles: &[TileId],
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<NdsmTile>, FetchError> {
    let dir = cache.ndsm_dir();
    let mut fetched = Vec::new();
    for &tile in tiles {
        let url = ndsm_tile_url(tile);
        let cached = cached_ndsm(&dir, tile);
        if mode == FetchMode::Offline {
            match cached.into_iter().next() {
                Some((published, path)) => fetched.push(NdsmTile {
                    tile,
                    url,
                    published,
                    path,
                }),
                None => progress(&format!("warning: no cached nDSM for tile {}", tile.name())),
            }
            continue;
        }
        // The nDSM is only a template: a failing server falls back to the cache or skips the tile.
        let unavailable = |reason: String, progress: &mut dyn FnMut(&str)| {
            let fallback = cached.first().cloned();
            progress(&format!(
                "warning: nDSM tile {} unavailable ({reason}); {}",
                tile.name(),
                if fallback.is_some() {
                    "using the cached file"
                } else {
                    "skipped"
                }
            ));
            fallback
        };
        let published = match fetcher.last_modified(&url) {
            Ok(date) => date,
            Err(FetchError::NotFound(_)) => {
                progress(&format!(
                    "no nDSM point cloud published for tile {}",
                    tile.name()
                ));
                continue;
            }
            Err(e) => {
                if let Some((published, path)) = unavailable(e.to_string(), progress) {
                    fetched.push(NdsmTile {
                        tile,
                        url,
                        published,
                        path,
                    });
                }
                continue;
            }
        };
        let path = cache.ndsm_path(tile, published);
        if path.exists() {
            progress(&format!("cached nDSM {}", tile.name()));
        } else {
            progress(&format!("downloading {url}"));
            let bytes = match fetcher.get(&url).and_then(|b| check_las(&url, b)) {
                Ok(bytes) => bytes,
                Err(e) => {
                    if let Some((published, path)) = unavailable(e.to_string(), progress) {
                        fetched.push(NdsmTile {
                            tile,
                            url,
                            published,
                            path,
                        });
                    }
                    continue;
                }
            };
            write_cache_file(&path, &bytes)?;
            for (_, older) in cached_ndsm(&dir, tile)
                .into_iter()
                .filter(|(_, p)| *p != path)
            {
                let _ = std::fs::remove_file(older);
            }
        }
        fetched.push(NdsmTile {
            tile,
            url,
            published,
            path,
        });
    }
    Ok(fetched)
}

/// Checks that `bytes` is a complete LAS/LAZ point cloud: the header, the LAZ chunk table at the
/// end of the file and the point count all have to be readable.
fn check_las(url: &str, bytes: Vec<u8>) -> Result<Vec<u8>, FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a complete LAS point cloud: {message}"),
    };
    {
        let mut reader = las::Reader::new(std::io::Cursor::new(bytes.clone()))
            .map_err(|e| corrupt(e.to_string()))?;
        let total = reader.header().number_of_points();
        // The last chunk ends the file: reading the final point fails on a truncated download.
        if total > 0 {
            reader.seek(total - 1).map_err(|e| corrupt(e.to_string()))?;
            reader.read_points(1).map_err(|e| corrupt(e.to_string()))?;
        }
    }
    Ok(bytes)
}

/// Cached nDSM files of `tile`, newest first (undated ones last).
fn cached_ndsm(dir: &Path, tile: TileId) -> Vec<(Option<NaiveDate>, PathBuf)> {
    let name = tile.name();
    let mut files: Vec<(Option<NaiveDate>, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().into_owned();
            let stem = file.strip_prefix(&name)?.strip_suffix(".laz")?;
            let date = match stem {
                "" => None,
                dated => Some(dated.strip_prefix('_')?.parse().ok()?),
            };
            Some((date, entry.path()))
        })
        .collect();
    files.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
    files
}

/// Checks that `bytes` is a zip archive with the tile's raster and world file, reading both
/// members to the end so their checksums are verified.
fn check_archive(bytes: &[u8], product: Product, tile: TileId) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for member in [product.raster_member(tile), product.world_file_member(tile)] {
        let mut entry = archive
            .by_name(&member)
            .map_err(|e| format!("{member}: {e}"))?;
        std::io::copy(&mut entry, &mut std::io::sink()).map_err(|e| format!("{member}: {e}"))?;
    }
    Ok(())
}

/// Cached publications of `tile` in the folder of `path`, newest first.
fn cached_versions(path: &Path, tile: TileId) -> Vec<(NaiveDate, PathBuf)> {
    let dir = path.parent().expect("cache paths have a parent");
    let prefix = format!("{}_", tile.name());
    let mut versions: Vec<(NaiveDate, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(&prefix)?
                .strip_suffix(".zip")?
                .parse()
                .ok()?;
            Some((date, entry.path()))
        })
        .collect();
    versions.sort_by_key(|v| std::cmp::Reverse(v.0));
    versions
}

fn newest_cached_version(path: &Path, tile: TileId) -> Option<(NaiveDate, PathBuf)> {
    cached_versions(path, tile).into_iter().next()
}

/// Attempts per request to a server that drops connections now and then (gdi.berlin.de, GeoSN).
pub(crate) const ATTEMPTS: u32 = 3;
#[cfg(not(test))]
const RETRY_DELAY: Duration = Duration::from_secs(2);
#[cfg(test)]
const RETRY_DELAY: Duration = Duration::ZERO;

/// Downloads `url` and reads it with `check`, up to [`ATTEMPTS`] times: server errors, resets
/// and answers `check` refuses (an error page with status 200) are retried; a missing URL or a
/// refused request is not, and neither is a write to a full disk.
pub(crate) fn get_with_retries<T>(
    fetcher: &dyn Fetcher,
    url: &str,
    check: impl Fn(&[u8]) -> Result<T, FetchError>,
) -> Result<T, FetchError> {
    get_with_attempts(fetcher, url, ATTEMPTS, check)
}

/// [`get_with_retries`] for an answer of at most `max_bytes`: it streams into the file `part`
/// (removed afterwards) through [`Fetcher::download_to`], so a larger answer is stopped while it
/// is read, as [`FetchError::TooLarge`] (not asked again).
pub(crate) fn get_limited_with_retries<T>(
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
pub(crate) fn get_with_attempts<T>(
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
                FetchError::NotFound(_) | FetchError::Refused { .. } | FetchError::TooLarge { .. },
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
pub(crate) fn download_with_retries(
    fetcher: &dyn Fetcher,
    url: &str,
    path: &Path,
    max_bytes: u64,
    progress: &mut dyn FnMut(&str),
) -> Result<(), FetchError> {
    let mut attempt = 1;
    loop {
        let result = fetcher.download_to(url, path, max_bytes, progress);
        match result {
            Err(
                FetchError::NotFound(_) | FetchError::Refused { .. } | FetchError::TooLarge { .. },
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
pub(crate) fn get_range_with_retries(
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
                | FetchError::RangeIgnored { .. },
            ) => return result,
            Err(_) if attempt < ATTEMPTS => {
                std::thread::sleep(RETRY_DELAY * attempt);
                attempt += 1;
            }
            _ => return result,
        }
    }
}

/// [`crate::fsutil::write_cache_file`] with the error as [`FetchError::Io`]; for the cache.
pub(crate) fn write_cache_file(path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    crate::fsutil::write_cache_file(path, bytes).map_err(|(path, e)| FetchError::Io(path, e))
}

/// [`crate::fsutil::write_inside`] with the error as [`FetchError::Io`]; for project files.
pub(crate) fn write_inside(root: &Path, path: &Path, bytes: &[u8]) -> Result<(), FetchError> {
    crate::fsutil::write_inside(root, path, bytes).map_err(|(path, e)| FetchError::Io(path, e))
}

/// Deletes the other cached publications of `tile` next to `current`.
fn remove_older_versions(current: &Path, tile: TileId) -> Result<(), FetchError> {
    let dir = current.parent().expect("cache paths have a parent");
    let prefix = format!("{}_", tile.name());
    let entries = std::fs::read_dir(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path != current && name.starts_with(&prefix) && name.ends_with(".zip") {
            std::fs::remove_file(&path).map_err(|e| FetchError::Io(path.clone(), e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::io::Write as _;

    use super::{FetchError, Fetcher};
    use crate::sources::alkis::test_support::{building_feature, land_use_feature, page};
    use crate::sources::alkis::{BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT};
    use crate::sources::lgb::Product;
    use crate::tiles::TileId;

    /// Serves fixed responses by URL and counts the requests.
    #[derive(Default)]
    pub(crate) struct MapFetcher {
        pub responses: RefCell<HashMap<String, Vec<u8>>>,
        /// Answers for URLs starting with the key, when no exact response matches.
        pub prefixes: RefCell<HashMap<String, Vec<u8>>>,
        /// `Last-Modified` dates by URL.
        pub modified: RefCell<HashMap<String, chrono::NaiveDate>>,
        pub calls: Cell<usize>,
        /// URLs that fail with a server error.
        pub failing: RefCell<Vec<String>>,
        /// URL prefixes the server refuses with HTTP 400.
        pub refusing: RefCell<Vec<String>>,
        /// The bodies of the refusals, by URL prefix (none: an empty body).
        pub refusal_bodies: RefCell<HashMap<String, Vec<u8>>>,
        /// URL prefixes that fail with a reset this many more times before they answer.
        pub flaky: RefCell<HashMap<String, u32>>,
        /// Every URL requested with `get`, in order.
        pub log: RefCell<Vec<String>>,
    }

    impl MapFetcher {
        pub fn insert(&self, url: &str, bytes: Vec<u8>) {
            self.responses.borrow_mut().insert(url.to_string(), bytes);
        }

        pub fn insert_prefix(&self, prefix: &str, bytes: Vec<u8>) {
            self.prefixes.borrow_mut().insert(prefix.to_string(), bytes);
        }

        fn fail_if_failing(&self, url: &str) -> Result<(), FetchError> {
            if self
                .refusing
                .borrow()
                .iter()
                .any(|p| url.starts_with(p.as_str()))
            {
                let body = self
                    .refusal_bodies
                    .borrow()
                    .iter()
                    .find(|(p, _)| url.starts_with(p.as_str()))
                    .map(|(_, body)| body.clone())
                    .unwrap_or_default();
                return Err(super::status_answer_error(url, 400, &body));
            }
            if let Some(left) = self
                .flaky
                .borrow_mut()
                .iter_mut()
                .find(|(p, left)| url.starts_with(p.as_str()) && **left > 0)
                .map(|(_, left)| left)
            {
                *left -= 1;
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "Connection reset by peer".to_string(),
                });
            }
            if self.failing.borrow().iter().any(|u| u == url) {
                return Err(FetchError::Http {
                    url: url.to_string(),
                    message: "503 Service Unavailable".to_string(),
                });
            }
            Ok(())
        }

        pub fn set_modified(&self, url: &str, date: chrono::NaiveDate) {
            self.modified.borrow_mut().insert(url.to_string(), date);
        }
    }

    impl Fetcher for MapFetcher {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            self.log.borrow_mut().push(url.to_string());
            self.fail_if_failing(url)?;
            let by_prefix = || {
                let prefixes = self.prefixes.borrow();
                prefixes
                    .iter()
                    .filter(|(prefix, _)| url.starts_with(prefix.as_str()))
                    .max_by_key(|(prefix, _)| prefix.len())
                    .map(|(_, bytes)| bytes.clone())
            };
            self.responses
                .borrow()
                .get(url)
                .cloned()
                .or_else(by_prefix)
                .ok_or_else(|| FetchError::NotFound(url.to_string()))
        }

        fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            self.fail_if_failing(url)?;
            if !self.responses.borrow().contains_key(url) {
                return Err(FetchError::NotFound(url.to_string()));
            }
            Ok(self.modified.borrow().get(url).copied())
        }
    }

    /// A tile archive with the raster and world file members LGB uses.
    pub fn tile_zip(product: Product, tile: TileId, raster: &[u8], world_file: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file(product.raster_member(tile), options)
            .unwrap();
        zip.write_all(raster).unwrap();
        zip.start_file(product.world_file_member(tile), options)
            .unwrap();
        zip.write_all(world_file.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

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

    /// Serves the currency CSVs and coarse synthetic orthophoto and DTM archives (100 × 100
    /// pixels of 10 m) for `tile`, published on `published`. The DTM rises 1 m per 10 m east.
    pub fn serve_synthetic_tile(fetcher: &MapFetcher, tile: TileId, published: &str) {
        let bounds = tile.bounds();
        let world = format!(
            "10\n0\n0\n-10\n{}\n{}\n",
            bounds.min_e + 5.0,
            bounds.max_n - 5.0
        );
        let heights: Vec<f32> = (0..100 * 100).map(|i| (i % 100) as f32).collect();
        for (product, raster) in [
            (Product::Orthophoto, red_jpeg(100, 100)),
            (Product::Dtm, dtm_tiff(100, 100, &heights)),
        ] {
            fetcher.insert(
                product.currency_csv_url(),
                currency_csv(&[(tile, published)]),
            );
            fetcher.insert(
                &product.tile_url(tile),
                tile_zip(product, tile, &raster, &world),
            );
        }
        let school = building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Allgemein bildende Schule",
            SCHOOL_BUILDING,
        );
        fetcher.insert_prefix(BUILDINGS_ENDPOINT, page(&[school], None));
        // The school's own parcel and residential land south-east of it.
        let land_use = [
            land_use_feature(
                "DEBBAL0100000010",
                "Fläche besonderer funktionaler Prägung",
                Some("Bildung und Wissenschaft"),
                [421_560.0, 5_821_660.0, 100.0],
            ),
            land_use_feature("DEBBAL0100000011", "Wohnbaufläche", None, SCHOOL_NEIGHBOUR),
        ];
        fetcher.insert_prefix(LAND_USE_ENDPOINT, page(&land_use, None));
        fetcher.insert(
            crate::sources::osm::STATE_URL,
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            crate::sources::osm::EXTRACT_URL,
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// A coarse synthetic bDOM archive for `tile` (100 × 100 pixels of 10 m) and its currency
    /// CSV: the surface lies 12 m above the synthetic DTM of [`serve_synthetic_tile`].
    pub fn serve_synthetic_bdom(fetcher: &MapFetcher, tile: TileId, published: &str) {
        let bounds = tile.bounds();
        let world = format!(
            "10\n0\n0\n-10\n{}\n{}\n",
            bounds.min_e + 5.0,
            bounds.max_n - 5.0
        );
        let heights: Vec<f32> = (0..100 * 100).map(|i| (i % 100) as f32 + 12.0).collect();
        let product = Product::ImageDsm;
        fetcher.insert(
            product.currency_csv_url(),
            currency_csv(&[(tile, published)]),
        );
        fetcher.insert(
            &product.tile_url(tile),
            tile_zip(product, tile, &dtm_tiff(100, 100, &heights), &world),
        );
    }

    /// Serves everything a Saxony project in `bbox` fetches: the GeoSN index rows and coarse
    /// synthetic DGM1/DOM1 archives (200 × 200 pixels of 10 m; the DTM rises 1 m per 10 m east,
    /// the surface lies 12 m above it) for the 2 km tiles of `bbox`, a WMS orthophoto of the
    /// right size, the Saxon ALKIS fixtures and the Saxony OSM extract.
    pub fn serve_synthetic_saxony(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::{alkis_wfs, geosn};
        let tiles = crate::tiles::tiles_covering_grid(&bbox.buffered(0.0), 2);
        for (product, layer, lift) in [(Product::SaxonyDtm, 6, 0.0), (Product::SaxonyDsm, 4, 12.0)]
        {
            let rows: Vec<String> = tiles
                .iter()
                .map(|t| {
                    format!(
                        r#"{{"attributes":{{"Kachel":"{:03}{:04}","Download":"{}","Stand":"2024-11-30"}}}}"#,
                        t.east_km,
                        t.north_km,
                        product.tile_url(*t)
                    )
                })
                .collect();
            fetcher.insert_prefix(
                &format!("{}/{layer}/query?", geosn::INDEX_URL),
                format!(r#"{{"features":[{}]}}"#, rows.join(",")).into_bytes(),
            );
            for &tile in &tiles {
                let b = tile.bounds_km(2);
                let world = format!("10\n0\n0\n-10\n{}\n{}\n", b.min_e + 5.0, b.max_n - 5.0);
                let heights: Vec<f32> = (0..200 * 200).map(|i| (i % 200) as f32 + lift).collect();
                fetcher.insert(
                    &product.tile_url(tile),
                    tile_zip(product, tile, &dtm_tiff(200, 200, &heights), &world),
                );
            }
        }
        let grid = crate::raster::GridSpec::covering(bbox, geosn::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                geosn::WMS_URL
            ),
            solid_jpeg(grid.width, grid.height, [0, 200, 0]),
        );
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                geosn::WMS_URL
            ),
            b"Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP".to_vec(),
        );
        for (type_name, body) in [
            (
                alkis_wfs::GEOSN.buildings_type,
                alkis_wfs::test_support::BUILDINGS,
            ),
            (
                alkis_wfs::GEOSN.land_use_type,
                alkis_wfs::test_support::LAND_USE,
            ),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    alkis_wfs::GEOSN.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let saxony = crate::sources::osm::ExtractSource::Saxony;
        fetcher.insert(
            saxony.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            saxony.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
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

    /// Serves everything a Saxony-Anhalt project in `bbox` (EPSG:25832) fetches: DGM1 and DOM1
    /// from the WCS (the terrain rises 1 m per metre east from 80 m, the surface lies 12 m above
    /// it), a WMS orthophoto of the right size, the tile index dates, the Saxony-Anhalt ALKIS
    /// fixtures and the Saxony-Anhalt OSM extract.
    pub fn serve_synthetic_saxony_anhalt(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::{alkis_wfs, lvermgeo};
        let grid = crate::raster::GridSpec::covering(bbox, 1.0);
        for (model, lift) in [(lvermgeo::Model::Dgm1, 80.0), (lvermgeo::Model::Dom1, 92.0)] {
            let values: Vec<f32> = (0..grid.width * grid.height)
                .map(|i| (i % grid.width) as f32 * 0.01 + lift)
                .collect();
            fetcher.insert_prefix(model.wcs_endpoint(), wcs_answer(&grid, &values));
        }
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                lvermgeo::WMS.url
            ),
            solid_jpeg(photo.width, photo.height, [200, 0, 200]),
        );
        for (layer, body) in [
            ("truedop_kacheln", lvermgeo::test_support::TRUEDOP_INDEX),
            (
                "bdom_aktualitaet_zyklus",
                lvermgeo::test_support::BDOM_INDEX,
            ),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=atkis_geobasis_bestand:{layer}&",
                    lvermgeo::INDEX_URL
                ),
                body.as_bytes().to_vec(),
            );
        }
        let service = alkis_wfs::LVERMGEO;
        for (type_name, body) in [
            (
                service.buildings_type,
                alkis_wfs::test_support::ST_BUILDINGS,
            ),
            (service.land_use_type, alkis_wfs::test_support::ST_LAND_USE),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    service.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let extract = crate::sources::osm::ExtractSource::SaxonyAnhalt;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// Serves the LAiV M-V height models of `bbox` (EPSG:25833, at most 4000 m a side): the
    /// overview lists the 2 km tiles that touch Mecklenburg-Vorpommern (DGM1 dated 2024-03-01,
    /// DOM1 2024-08-01), and the WCS answers with a plain TIFF of the area inside the coverage
    /// (one answer for any subset). In the listed tiles the terrain rises 0.01 m per metre east
    /// from 5 m and the surface lies 12 m above it; elsewhere both are 0.0, as the real WCS fills
    /// areas without a tile.
    pub fn serve_synthetic_mv(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::laiv_mv::{self, Model};
        use crate::sources::region::{Region, regions_for};
        let tiles: Vec<TileId> = crate::tiles::tiles_covering_grid(bbox, 2)
            .into_iter()
            // `regions_for` adds 100 m around the box: the tile itself.
            .filter(|tile| {
                regions_for(&tile.bounds_km(2).buffered(-100.0))
                    .contains(&Region::MecklenburgVorpommern)
            })
            .collect();
        for (model, date, lift) in [
            (Model::Dgm1, "2024-03-01", 5.0),
            (Model::Dom1, "2024-08-01", 17.0),
        ] {
            fetcher.insert_prefix(
                &laiv_mv::overview_prefix(model),
                laiv_mv::test_support::overview_answer(model, &tiles, date),
            );
            let Some(inside) = bbox.intersection(&laiv_mv::ENVELOPE) else {
                continue;
            };
            let grid = crate::raster::GridSpec::covering(&inside, 1.0);
            let values: Vec<f32> = (0..grid.height)
                .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let (e, n) = (
                        grid.west + f64::from(x) + 0.5,
                        grid.north - f64::from(y) - 0.5,
                    );
                    let tile = TileId {
                        east_km: (e / 2000.0).floor() as u32 * 2,
                        north_km: (n / 2000.0).floor() as u32 * 2,
                    };
                    if tiles.contains(&tile) {
                        lift + x as f32 * 0.01
                    } else {
                        0.0
                    }
                })
                .collect();
            fetcher.insert_prefix(model.wcs().url, geotiff(&grid, &values));
        }
        // The dop20 overview (tiles dated 2024-08-14) and the DOP WCS: a flat JPEG per piece.
        fetcher.insert_prefix(
            &format!("{}?type[]=dop20&bbox=", laiv_mv::OVERVIEW),
            laiv_mv::test_support::dop20_overview_answer(&tiles, "2024-08-14"),
        );
        let ortho =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        let source = crate::sources::wms::OrthophotoSource::Wcs(laiv_mv::DOP);
        for (piece, url) in crate::sources::wms::source_pieces(&source, &ortho) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [120, 130, 110]));
        }
    }

    /// Serves everything a Mecklenburg-Vorpommern project in `bbox` (EPSG:25833) fetches: the
    /// rasters of [`serve_synthetic_mv`], the MV ALKIS fixtures (OGC API) and the MV OSM extract.
    pub fn serve_synthetic_mecklenburg_vorpommern(
        fetcher: &MapFetcher,
        bbox: &crate::crs::BoundingBox,
    ) {
        use crate::sources::alkis;
        serve_synthetic_mv(fetcher, bbox);
        fetcher.insert_prefix(
            alkis::MV_BUILDINGS_ENDPOINT,
            include_bytes!("../../../tests/fixtures/mecklenburg-vorpommern-buildings.json")
                .to_vec(),
        );
        fetcher.insert_prefix(
            alkis::MV_LAND_USE_ENDPOINT,
            include_bytes!("../../../tests/fixtures/mecklenburg-vorpommern-land-use.json").to_vec(),
        );
        let extract = crate::sources::osm::ExtractSource::MecklenburgVorpommern;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// Serves everything a Hamburg project in `bbox` (EPSG:25832) fetches: the statewide DGM1 and
    /// bDOM zips with the 1 km tiles of `bbox` (terrain 5 m rising 0.01 m per metre east, surface
    /// 12 m above), a WMS orthophoto of the right size, the DOP metadata, the Hamburg ALKIS
    /// fixtures and the Hamburg OSM extract.
    pub fn serve_synthetic_hamburg(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::lgv;
        let zip_of = |members: Vec<(String, Vec<u8>)>| {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            for (name, bytes) in members {
                zip.start_file(name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(&bytes).unwrap();
            }
            zip.finish().unwrap().into_inner()
        };
        let tiles = crate::tiles::tiles_covering(bbox);
        for (model, prefix, lift) in [
            (lgv::Model::Dgm1, "dgm1", 5.0),
            (lgv::Model::Bdom, "dom1", 17.0),
        ] {
            let members = tiles
                .iter()
                .map(|t| {
                    let grid = crate::raster::GridSpec {
                        west: f64::from(t.east_km) * 1000.0,
                        north: f64::from(t.north_km + 1) * 1000.0,
                        pixel_size: 1.0,
                        width: 1000,
                        height: 1000,
                    };
                    let values: Vec<f32> = (0..1000 * 1000)
                        .map(|i| (i % 1000) as f32 * 0.01 + lift)
                        .collect();
                    (
                        format!(
                            "s32_{}/{prefix}_32_{}_{}_1_hh_2022.tif",
                            t.east_km, t.east_km, t.north_km
                        ),
                        geotiff(&grid, &values),
                    )
                })
                .collect();
            fetcher.insert(model.archive_url(), zip_of(members));
        }
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!("{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap", lgv::WMS.url),
            solid_jpeg(photo.width, photo.height, [0, 100, 200]),
        );
        let centre = (
            ((bbox.min_e + bbox.max_e) / 2000.0).floor(),
            ((bbox.min_n + bbox.max_n) / 2000.0).floor(),
        );
        let csv = format!(
            "Kachelname;Aktualitaet;Erfassungsmethode\ndop20rgbi_32_{:03}_{:04}_1_hh_2026.tif;19.03.2026;0\n",
            centre.0, centre.1
        );
        fetcher.insert(
            lgv::METADATA_URL,
            zip_of(vec![(
                "DOP_unbelaubt_AdV-Metadaten_Hamburg/AdV-Metadaten_DOP_2026_unbelaubt_hh.csv"
                    .to_string(),
                csv.into_bytes(),
            )]),
        );
        fetcher.insert_prefix(
            lgv::BUILDINGS_ENDPOINT,
            lgv::test_support::BUILDINGS.as_bytes().to_vec(),
        );
        fetcher.insert_prefix(
            lgv::LAND_USE_ENDPOINT,
            lgv::test_support::LAND_USE.as_bytes().to_vec(),
        );
        let extract = crate::sources::osm::ExtractSource::Hamburg;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// Serves everything a Hesse project in `bbox` (EPSG:25832) fetches: the height models (see
    /// below), a WMS orthophoto of the right size with the GetFeatureInfo flight date (19.08.2025),
    /// the Hesse ALKIS fixtures and the Hesse OSM extract.
    pub fn serve_synthetic_hessen(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::{alkis_wfs, hvbg};
        serve_hessen_models(fetcher, bbox);
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!("{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap", hvbg::WMS.url),
            solid_jpeg(photo.width, photo.height, [0, 100, 200]),
        );
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                hvbg::WMS.url
            ),
            b"name = 'dop_476500_5552000'  land = 'he'  aufloesung = '20'  farbe = 'rgb'  \
              bezug = 'BDOM'  flugjahr = '2025'  bildflug = '19-08-2025'  aenderung = '12-10-2026'\n"
                .to_vec(),
        );
        let service = alkis_wfs::HVBG;
        for (type_name, body) in [
            (
                service.buildings_type,
                alkis_wfs::test_support::HE_BUILDINGS,
            ),
            (service.land_use_type, alkis_wfs::test_support::HE_LAND_USE),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    service.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let extract = crate::sources::osm::ExtractSource::Hessen;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// Serves the HVBG height models of a Hesse project in `bbox` (EPSG:25832): the DGM1 and
    /// DOM1 product trees, the Frankfurt listings, the metadata list (every tile of `bbox` in
    /// Frankfurt am Main, flown 2023-02-28) and the municipal zips with the 1 km tiles (terrain
    /// 100 m rising 0.01 m per metre east, surface 12 m above).
    pub fn serve_hessen_models(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::hvbg::{self, test_support as he};
        const DISTRICT: &str = "Kreisfreie Stadt Frankfurt am Main";
        let tiles = crate::tiles::tiles_covering(bbox);
        let rows: Vec<(TileId, u32, &str, &str)> = tiles
            .iter()
            .map(|&t| (t, 44985, "Frankfurt am Main", DISTRICT))
            .collect();
        fetcher.insert(he::METADATA_URL, he::metadata_xlsx(&rows));
        for (model, code, lift) in [
            (hvbg::Model::Dgm1, "DGM1", 100.0),
            (hvbg::Model::Dom1, "DOM1", 112.0),
        ] {
            // Only the DGM1 description links the metadata list (the DOM1 one does too, but
            // both models take it from DGM1).
            let link = model == hvbg::Model::Dgm1;
            fetcher.insert(&hvbg::tree_url(model), he::tree_answer(&[DISTRICT], link));
            let uri = format!(
                "/downloadcenter/20261108/{}/{DISTRICT}/Frankfurt_am_Main-{code}.zip",
                model.product()
            );
            fetcher.insert(
                &hvbg::listing_url(model.product(), DISTRICT, 1),
                he::listing_answer(&[(&format!("Frankfurt_am_Main-{code}"), &uri)]),
            );
            let members: Vec<(String, Vec<u8>, bool)> = tiles
                .iter()
                .map(|&t| {
                    (
                        format!("{code}_32_{}_{}_1_he.tif", t.east_km, t.north_km),
                        he::tile_tiff(t, lift, 0.01),
                        true,
                    )
                })
                .collect();
            let members: Vec<(&str, Vec<u8>, bool)> = members
                .iter()
                .map(|(name, bytes, deflate)| (name.as_str(), bytes.clone(), *deflate))
                .collect();
            fetcher.insert(
                &hvbg::file_url(&uri).expect("a server path"),
                crate::sources::zip_range::test_support::archive(&members),
            );
        }
    }

    /// Serves everything a Schleswig-Holstein project in `bbox` (EPSG:25832) fetches: a WMS
    /// orthophoto of the right size with the metadata GetFeatureInfo flight date (17.03.2025),
    /// the SH ALKIS fixtures, the SH OSM extract and the height models (see below).
    pub fn serve_synthetic_schleswig_holstein(
        fetcher: &MapFetcher,
        bbox: &crate::crs::BoundingBox,
    ) {
        use crate::sources::{alkis_wfs, lvermgeo_sh};
        serve_schleswig_holstein_models(fetcher, bbox);
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                lvermgeo_sh::WMS.url
            ),
            solid_jpeg(photo.width, photo.height, [0, 100, 200]),
        );
        fetcher.insert_prefix(
            "https://service.gdi-sh.de/WMS_SH_MD_DOP?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
            include_bytes!("../../../tests/fixtures/schleswig-holstein-dop-info.txt").to_vec(),
        );
        let service = alkis_wfs::LVERMGEO_SH;
        for (type_name, body) in [
            (
                service.buildings_type,
                alkis_wfs::test_support::SH_BUILDINGS,
            ),
            (service.land_use_type, alkis_wfs::test_support::SH_LAND_USE),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    service.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        let extract = crate::sources::osm::ExtractSource::SchleswigHolstein;
        fetcher.insert(
            extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            extract.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
    }

    /// Serves the LVermGeo SH height models of a Schleswig-Holstein project in `bbox`
    /// (EPSG:25832): the Downloadportal overviews of every tile of `bbox` (DGM1 2022, flown
    /// 2022-03-10; bDOM 2023, flown 2023-04-07), the DGM1 GeoTIFFs of the INSPIRE ATOM directory
    /// (terrain 20 m rising 0.01 m per metre east) and the bDOM tiles at 1 m instead of 0.2 m
    /// (surface 12 m above; test builds accept that size, and the reduction passes it through).
    pub fn serve_schleswig_holstein_models(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        serve_schleswig_holstein_models_lifted(fetcher, bbox, 12.0);
    }

    /// [`serve_schleswig_holstein_models`] with the bDOM `lift` metres above the DGM1 (negative:
    /// below it).
    pub fn serve_schleswig_holstein_models_lifted(
        fetcher: &MapFetcher,
        bbox: &crate::crs::BoundingBox,
        lift: f32,
    ) {
        use crate::sources::lvermgeo_sh::{ATOM_DATA, PORTAL, Product, overview_prefix};
        let tiles = crate::tiles::tiles_covering(bbox);
        let feature = |tile: &TileId, product: Product| {
            let (e, n) = (tile.east_km, tile.north_km);
            let block = format!("32{}_{}", e / 10 * 10, n / 10 * 10);
            let properties = match product {
                Product::Dgm1 => serde_json::json!({
                    "title": format!("32{e:03}{n:04}"), "type": "dgm1", "jahr": "2022",
                    "kaname": format!("dgm1_32_{e:03}_{n:04}_1_sh_2022"), "filepath": block,
                    "a_datum": "2022-03-10",
                }),
                Product::Bdom => serde_json::json!({
                    "title": format!("32{e:03}{n:04}"), "type": "bdom", "jahr": "2023",
                    "kachel_n": format!("bdom20rgbi_32_{e:03}_{n:04}_1_sh_2023"),
                    "kach10km": block, "filepath": format!("32{e:03}_{n:04}"),
                    "a_datum": "2023-04-07",
                }),
            };
            serde_json::json!({"type": "Feature", "properties": properties})
        };
        for product in [Product::Dgm1, Product::Bdom] {
            let features: Vec<_> = tiles.iter().map(|t| feature(t, product)).collect();
            let answer = serde_json::json!({
                "success": true,
                "result": {"type": "FeatureCollection", "features": features},
            });
            fetcher.insert_prefix(&overview_prefix(product), answer.to_string().into_bytes());
        }
        for tile in &tiles {
            let (e, n) = (tile.east_km, tile.north_km);
            let grid = crate::raster::GridSpec {
                west: f64::from(e) * 1000.0,
                north: f64::from(n + 1) * 1000.0,
                pixel_size: 1.0,
                width: 1000,
                height: 1000,
            };
            let terrain: Vec<f32> = (0..1000 * 1000)
                .map(|i| 20.0 + (i % 1000) as f32 * 0.01)
                .collect();
            let surface: Vec<f32> = terrain.iter().map(|z| z + lift).collect();
            fetcher.insert(
                &format!("{ATOM_DATA}/dgm1_32_{e:03}_{n:04}_1_sh_2022.tif"),
                geotiff(&grid, &terrain),
            );
            fetcher.insert(
                &format!(
                    "{PORTAL}/massen.php?file=bdom20nc_32_{e:03}_{n:04}_1_sh_2023.tif&id=6\
                     &live=2023&km=32{}_{}&stack=32{e:03}_{n:04}",
                    e / 10 * 10,
                    n / 10 * 10
                ),
                geotiff(&grid, &surface),
            );
        }
    }

    /// The GetFeatureInfo text answer of the NRW orthophoto WMS (flight 11.05.2025), as the
    /// server gives it.
    pub const NRW_FLIGHT_INFO: &str = "GetFeatureInfo results:\n\nLayer 'nw_dop_utm_info'\n  \
        Feature 20301: \n    Kachelname = '32354_5642'\n    Bildflugdatum = '11.05.2025'\n    \
        Bodenauflösung = '0.10'\n    Photometrie = 'RGBI'\n    Bit_pro_Kanal = '8'\n";

    /// The meta zip of Geobasis NRW: a CSV `csv_name` with the usual preamble and a row per
    /// `(tile name, date)`.
    pub fn nrw_meta_zip(csv_name: &str, release: &str, rows: &[(String, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut csv = format!(
            "Kachelinformationen fuer die Datenabgabe\r\nLand;Nordrhein-Westfalen\r\n\
             Aktualitaet_Kachelinformationen;{release}\r\nVersion_Standard;3.3\r\n\
             Kachelname;Aktualitaet;Erfassungsmethode\r\n",
        );
        for (name, date) in rows {
            csv.push_str(&format!("{name};{date};5020\r\n"));
        }
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file(csv_name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(csv.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

    /// Serves the Geobasis NRW rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the
    /// right size with the flight date 11.05.2025, the meta zips
    /// of DGM1 and DOM1 naming every tile of `bbox` (scanned 2022-02-12, file names with
    /// `_2022`) and the tiles (terrain 20 m rising 0.01 m per metre east, the surface 12 m
    /// above).
    pub fn serve_synthetic_nrw(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::geobasis_nrw::{Model, WMS, meta_url, tile_url};
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        // The WMS allows 4000 px a side: a larger area comes in pieces, each of its own size.
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [0, 100, 200]));
        }
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            NRW_FLIGHT_INFO.as_bytes().to_vec(),
        );
        let tiles = crate::tiles::tiles_covering(bbox);
        for (model, slug, lift) in [(Model::Dgm1, "dgm1", 0.0), (Model::Dom1, "dom1", 12.0)] {
            let name = |t: &TileId| format!("{slug}_32_{}_{}_1_nw_2022", t.east_km, t.north_km);
            let rows: Vec<(String, &str)> = tiles.iter().map(|t| (name(t), "2022-02-12")).collect();
            fetcher.insert(
                &meta_url(model),
                nrw_meta_zip(&format!("{slug}_nw.csv"), "2026-10-26", &rows),
            );
            for tile in &tiles {
                let grid = crate::raster::GridSpec {
                    west: f64::from(tile.east_km) * 1000.0,
                    north: f64::from(tile.north_km + 1) * 1000.0,
                    pixel_size: 1.0,
                    width: 1000,
                    height: 1000,
                };
                let heights: Vec<f32> = (0..1000 * 1000)
                    .map(|i| 20.0 + (i % 1000) as f32 * 0.01 + lift)
                    .collect();
                fetcher.insert(&tile_url(model, &name(tile)), geotiff(&grid, &heights));
            }
        }
    }

    /// A `Laserbefliegung` answer of the LVermGeo RP Blattschnitt WFS (GML 3.2, as MapServer
    /// writes it) with a feature per `(tile, flugjahr, flugtage)`; an empty `flugjahr` or
    /// `flugtage` is left out, as the server does.
    pub fn rp_tile_list(rows: &[(TileId, &str, &str)]) -> String {
        let mut gml = format!(
            "<?xml version='1.0' encoding=\"UTF-8\" ?>\n<wfs:FeatureCollection \
             xmlns:ms=\"http://mapserver.gis.umn.edu/mapserver\" \
             xmlns:gml=\"http://www.opengis.net/gml/3.2\" \
             xmlns:wfs=\"http://www.opengis.net/wfs/2.0\" timeStamp=\"2026-11-08T07:03:21\" \
             numberMatched=\"{0}\" numberReturned=\"{0}\">\n",
            rows.len()
        );
        for (gid, (tile, year, days)) in rows.iter().enumerate() {
            let (e, n) = (tile.east_km * 1000, tile.north_km * 1000);
            gml.push_str(&format!(
                "<wfs:member><ms:Laserbefliegung gml:id=\"Laserbefliegung.{gid}\"><ms:msGeometry>\
                 <gml:Polygon gml:id=\"Laserbefliegung.{gid}.1\" \
                 srsName=\"urn:ogc:def:crs:EPSG::25832\"><gml:exterior><gml:LinearRing>\
                 <gml:posList srsDimension=\"2\">{e} {n1} {e1} {n1} {e1} {n} {e} {n} {e} {n1} \
                 </gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></ms:msGeometry>\
                 <ms:gid>{gid}</ms:gid><ms:art>Laserbefliegung</ms:art>\
                 <ms:betrifft>DGM, DOM, Laserpunkte</ms:betrifft>\
                 <ms:kachel>{:03}{:04}</ms:kachel>",
                tile.east_km,
                tile.north_km,
                e1 = e + 1000,
                n1 = n + 1000,
            ));
            if !year.is_empty() {
                gml.push_str(&format!("<ms:flugjahr>{year}</ms:flugjahr>"));
            }
            if !days.is_empty() {
                gml.push_str(&format!("<ms:flugtage>{days}</ms:flugtage>"));
            }
            gml.push_str("</ms:Laserbefliegung></wfs:member>\n");
        }
        gml.push_str("</wfs:FeatureCollection>\n");
        gml
    }

    /// The GetFeatureInfo text answer of the LVermGeo RP orthophoto WMS at the Görres-Gymnasium,
    /// Koblenz (block 400/5578, `erstellung` 25.08.2025), as the server gives it.
    pub const RP_FLIGHT_INFO: &str = "GetFeatureInfo results:\n\nLayer 'rp_dop20_info'\n  \
        Feature 5098: \n\nLayer 'metadaten'\n  Feature 5098: \n    gid = '5098'\n    \
        publikation = '2026-01-11 02:15:10.166'\n    name = 'dop20rgbi_32400_5578'\n    \
        titel = 'Digitale Orthophotos 0,2m Rheinland-Pfalz (DOP20RGB) - 4005578'\n    \
        erstellung = '25.08.2025'\n    aktualisierung = 'n/a'\n    \
        csw-anfrage = 'https://metadatenkatalog-vermkv.service24.rlp.de/extern/vertriebsprodukte?\
        request=GetRecords&service=CSW&version=2.0.2'\n";

    /// Serves the LVermGeo RP height models of a project in `bbox` (EPSG:25832): the tile list
    /// naming every tile of `bbox` (flown 30.03.24 and 06.04.24, file names with `_2024`) and
    /// the DGM1 and DOM1 tiles (terrain 20 m rising 0.01 m per metre east, the surface 12 m
    /// above).
    pub fn serve_synthetic_rp_heights(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::lvermgeo_rp::{Model, tile_list_prefix, tile_url};
        let tiles = crate::tiles::tiles_covering(bbox);
        let rows: Vec<(TileId, &str, &str)> = tiles
            .iter()
            .map(|t| (*t, "2024", "30.03.24, 06.04.24"))
            .collect();
        fetcher.insert_prefix(&tile_list_prefix(), rp_tile_list(&rows).into_bytes());
        for (model, slug, lift) in [(Model::Dgm1, "dgm1", 0.0), (Model::Dom1, "dom1", 12.0)] {
            for tile in &tiles {
                let grid = crate::raster::GridSpec {
                    west: f64::from(tile.east_km) * 1000.0,
                    north: f64::from(tile.north_km + 1) * 1000.0,
                    pixel_size: 1.0,
                    width: 1000,
                    height: 1000,
                };
                let heights: Vec<f32> = (0..1000 * 1000)
                    .map(|i| 20.0 + (i % 1000) as f32 * 0.01 + lift)
                    .collect();
                let name = format!("{slug}_32_{}_{}_1_rp_2024", tile.east_km, tile.north_km);
                fetcher.insert(&tile_url(model, &name), geotiff(&grid, &heights));
            }
        }
    }

    /// Serves the LVermGeo RP rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the
    /// right size with the flight date [`RP_FLIGHT_INFO`] (25.08.2025) and the height models of
    /// [`serve_synthetic_rp_heights`].
    pub fn serve_synthetic_rp(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::lvermgeo_rp::WMS;
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [80, 120, 60]));
        }
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            RP_FLIGHT_INFO.as_bytes().to_vec(),
        );
        serve_synthetic_rp_heights(fetcher, bbox);
    }

    /// The orthophoto flight date and the height model date the synthetic LGLN STAC answers give
    /// (the live ones at the Kaiser-Wilhelm- und Ratsgymnasium, Hannover).
    pub const LGLN_FLIGHT: &str = "2025-03-06T00:00:00Z";
    pub const LGLN_HEIGHTS: &str = "2016-04-18T00:00:00Z";

    /// Serves the LGLN rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the right
    /// size, DGM1 and DOM1 from the WCS (one GeoTIFF of `bbox` for any subset; the terrain rises
    /// 0.01 m per metre east from 20 m, the surface lies 12 m above it) and the three STAC
    /// searches (flight [`LGLN_FLIGHT`], both height models [`LGLN_HEIGHTS`]).
    pub fn serve_synthetic_lgln(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        use crate::sources::lgln::{self, Model, test_support::stac_page};
        let photo =
            crate::raster::GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&lgln::WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [90, 110, 70]));
        }
        let grid = crate::raster::GridSpec::covering(bbox, 1.0);
        for (model, lift) in [(Model::Dgm1, 20.0), (Model::Dom1, 32.0)] {
            let values: Vec<f32> = (0..grid.height)
                .flat_map(|_| (0..grid.width).map(move |x| lift + x as f32 * 0.01))
                .collect();
            fetcher.insert_prefix(model.wcs().url, geotiff(&grid, &values));
        }
        fetcher.insert_prefix(
            lgln::DOP_STAC,
            stac_page(&[("dop20rgbi_32_550_5802_2_ni", LGLN_FLIGHT)], None),
        );
        for model in [Model::Dgm1, Model::Dom1] {
            fetcher.insert_prefix(
                model.stac(),
                stac_page(&[("dgm1_32_551_5803_1_ni_2016", LGLN_HEIGHTS)], None),
            );
        }
    }

    /// An nDSM tile like LGB's: LAZ, 0.01 m scale; a 2 m hedge line north–south through the school.
    pub fn serve_synthetic_ndsm(fetcher: &MapFetcher, tile: TileId) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ndsm.laz");
        let mut builder = las::Builder::from((1, 2));
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(400_000.0),
            y: transform(5_800_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..500 {
            writer
                .write_point(las::Point {
                    x: 421_610.1,
                    y: 5_821_650.1 + f64::from(i) * 0.2,
                    z: 2.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        let url = crate::sources::lgb::ndsm_tile_url(tile);
        fetcher.insert(&url, std::fs::read(&path).unwrap());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
    }

    /// An OSM node for [`osm_pbf`]: id, latitude, longitude, tags.
    pub type PbfNode<'a> = (i64, f64, f64, &'a [(&'a str, &'a str)]);

    /// A minimal OSM PBF (uncompressed blobs, plain nodes): the header block, then one block
    /// with each node.
    pub fn osm_pbf(nodes: &[PbfNode<'_>]) -> Vec<u8> {
        fn varint(out: &mut Vec<u8>, mut v: u64) {
            while v >= 0x80 {
                out.push((v as u8) | 0x80);
                v >>= 7;
            }
            out.push(v as u8);
        }
        fn zigzag(v: i64) -> u64 {
            ((v << 1) ^ (v >> 63)) as u64
        }
        fn number(out: &mut Vec<u8>, field: u64, v: u64) {
            varint(out, field << 3);
            varint(out, v);
        }
        fn bytes(out: &mut Vec<u8>, field: u64, b: &[u8]) {
            varint(out, (field << 3) | 2);
            varint(out, b.len() as u64);
            out.extend_from_slice(b);
        }
        fn blob(out: &mut Vec<u8>, kind: &str, data: &[u8]) {
            let mut blob = Vec::new();
            bytes(&mut blob, 1, data);
            number(&mut blob, 2, data.len() as u64);
            let mut header = Vec::new();
            bytes(&mut header, 1, kind.as_bytes());
            number(&mut header, 3, blob.len() as u64);
            out.extend((header.len() as u32).to_be_bytes());
            out.extend(header);
            out.extend(blob);
        }
        let mut strings: Vec<String> = vec![String::new()];
        let mut index = |s: &str| -> u64 {
            let at = strings.iter().position(|t| t == s).unwrap_or_else(|| {
                strings.push(s.to_string());
                strings.len() - 1
            });
            at as u64
        };
        let mut group = Vec::new();
        for &(id, lat, lon, tags) in nodes {
            let (mut keys, mut vals) = (Vec::new(), Vec::new());
            for (k, v) in tags {
                varint(&mut keys, index(k));
                varint(&mut vals, index(v));
            }
            let mut node = Vec::new();
            number(&mut node, 1, zigzag(id));
            bytes(&mut node, 2, &keys);
            bytes(&mut node, 3, &vals);
            number(&mut node, 8, zigzag((lat * 1e7).round() as i64));
            number(&mut node, 9, zigzag((lon * 1e7).round() as i64));
            bytes(&mut group, 1, &node);
        }
        let mut table = Vec::new();
        for s in &strings {
            bytes(&mut table, 1, s.as_bytes());
        }
        let mut block = Vec::new();
        bytes(&mut block, 1, &table);
        bytes(&mut block, 2, &group);
        let mut header = Vec::new();
        bytes(&mut header, 4, b"OsmSchema-V0.6");
        let mut file = Vec::new();
        blob(&mut file, "OSMHeader", &header);
        blob(&mut file, "OSMData", &block);
        file
    }

    /// The OSM extract fixture cut around the school.
    pub const OSM_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/strausberg-school.osm.pbf"
    );

    /// Residential land south-east of the fixture school area (EPSG:25833 corner + side).
    pub const SCHOOL_NEIGHBOUR: [f64; 3] = [421_690.0, 5_821_600.0, 20.0];

    /// A 10 m square near the centroid of the fixture school area (EPSG:25833 corner + side).
    pub const SCHOOL_BUILDING: [f64; 3] = [421_605.0, 5_821_730.0, 10.0];

    /// A currency CSV with one row per tile and the given publication date.
    pub fn currency_csv(rows: &[(TileId, &str)]) -> Vec<u8> {
        let mut csv = String::from(
            "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n",
        );
        for (tile, published) in rows {
            csv.push_str(&format!("{};;2018-11-27;;{published};\n", tile.name()));
        }
        csv.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use std::cell::Cell;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };
    const PRODUCT: Product = Product::Dtm;

    fn serve(fetcher: &MapFetcher, published: &str) {
        fetcher.insert(
            PRODUCT.currency_csv_url(),
            currency_csv(&[(TILE, published)]),
        );
        fetcher.insert(
            &PRODUCT.tile_url(TILE),
            tile_zip(
                PRODUCT,
                TILE,
                b"raster",
                "1\n0\n0\n-1\n421000.5\n5821999.5\n",
            ),
        );
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &Cache,
        tiles: &[TileId],
        mode: FetchMode,
    ) -> Result<Vec<FetchedTile>, FetchError> {
        fetch_tiles(fetcher, cache, PRODUCT, tiles, mode, &mut |_| {})
    }

    fn cached_files(cache: &Cache) -> Vec<String> {
        let dir = cache.currency_csv_path(PRODUCT);
        let mut names: Vec<String> = std::fs::read_dir(dir.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn fetch_downloads_then_reuses_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");

        let first = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(fetcher.calls.get(), 2);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].tile, TILE);
        assert_eq!(first[0].url, PRODUCT.tile_url(TILE));
        assert_eq!(first[0].currency.published.to_string(), "2026-01-26");
        assert!(first[0].zip.exists());

        let second = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            fetcher.calls.get(),
            3,
            "only the currency CSV is downloaded again"
        );
        assert_eq!(second[0].zip, first[0].zip);
    }

    #[test]
    fn new_publication_replaces_old_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        serve(&fetcher, "2026-06-01");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            cached_files(&cache),
            ["33421-5821_2026-06-01.zip", "currency.csv"]
        );
    }

    #[test]
    fn truncated_download_is_not_cached() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let url = PRODUCT.tile_url(TILE);
        let truncated = fetcher.responses.borrow()[&url][..100].to_vec();
        fetcher.insert(&url, truncated);

        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CorruptArchive { .. }), "{err:?}");
        assert_eq!(cached_files(&cache), ["currency.csv"]);
    }

    #[test]
    fn fetch_unknown_tile_names_it() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let outside = TileId {
            east_km: 999,
            north_km: 5999,
        };
        let err = fetch(&fetcher, &cache, &[outside], FetchMode::Online).unwrap_err();
        assert!(err.to_string().contains("tile 33999-5999 is not"), "{err}");
    }

    #[test]
    fn offline_without_cache_lists_missing_tiles() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let north = TileId {
            east_km: 421,
            north_km: 5822,
        };
        let csv = cache.currency_csv_path(PRODUCT);
        std::fs::create_dir_all(csv.parent().unwrap()).unwrap();
        std::fs::write(
            &csv,
            currency_csv(&[(TILE, "2026-01-26"), (north, "2026-01-26")]),
        )
        .unwrap();
        let fetcher = MapFetcher::default();

        let err = fetch(&fetcher, &cache, &[TILE, north], FetchMode::Offline).unwrap_err();
        assert_eq!(fetcher.calls.get(), 0);
        assert!(
            matches!(&err, FetchError::MissingOffline(_, tiles) if tiles.len() == 2),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("33421-5821") && message.contains("33421-5822"),
            "{message}"
        );
        assert!(message.contains("terrain model (DGM1)"), "{message}");
    }

    const NORTH: TileId = TileId {
        east_km: 421,
        north_km: 5822,
    };
    const EAST: TileId = TileId {
        east_km: 422,
        north_km: 5821,
    };

    /// Serves the GeoSN DGM1 index fixture and archives for its first two tiles.
    fn serve_geosn(fetcher: &MapFetcher) -> [TileId; 2] {
        let tiles = [
            TileId::parse("33414-5654").unwrap(),
            TileId::parse("33416-5654").unwrap(),
        ];
        let product = Product::SaxonyDtm;
        fetcher.insert_prefix(
            &format!("{}/6/query?", crate::sources::geosn::INDEX_URL),
            crate::sources::geosn::test_support::DGM1_INDEX
                .as_bytes()
                .to_vec(),
        );
        for tile in tiles {
            let world = format!(
                "1\n0\n0\n-1\n{}.5\n{}.5\n",
                tile.east_km * 1000,
                tile.north_km * 1000 + 1999
            );
            fetcher.insert(
                &product.tile_url(tile),
                tile_zip(product, tile, &dtm_tiff(2, 2, &[100.0; 4]), &world),
            );
        }
        tiles
    }

    #[test]
    fn refused_requests_are_not_retried() {
        let fetcher = MapFetcher::default();
        fetcher
            .refusing
            .borrow_mut()
            .push("https://example.org/".to_string());
        let err =
            get_with_retries(&fetcher, "https://example.org/x", |b| Ok(b.to_vec())).unwrap_err();
        assert!(
            matches!(err, FetchError::Refused { status: 400, .. }),
            "{err:?}"
        );
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn index_query_is_retried() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        fetcher
            .flaky
            .borrow_mut()
            .insert(format!("{}/6/query?", crate::sources::geosn::INDEX_URL), 1);
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
    }

    #[test]
    fn index_row_without_download_uses_the_share() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        fetcher.insert_prefix(
            &format!("{}/6/query?", crate::sources::geosn::INDEX_URL),
            br#"{"features":[{"attributes":{"Kachel":"4145654","Download":null,"Stand":"2024-11-30"}}]}"#.to_vec(),
        );
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles[..1],
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched[0].url, Product::SaxonyDtm.tile_url(tiles[0]));
    }

    #[test]
    fn offline_tile_missing_from_the_cached_index_is_missing_offline() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        let product = Product::SaxonyDtm;
        fetch_tiles(
            &fetcher,
            &cache,
            product,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        let elsewhere = TileId::parse("33440-5660").unwrap();
        let err = fetch_tiles(
            &MapFetcher::default(),
            &cache,
            product,
            &[elsewhere],
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::MissingOffline(_, t) if t == &[elsewhere]),
            "{err:?}"
        );
    }

    #[test]
    fn geosn_tiles_fetch_and_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let tiles = serve_geosn(&fetcher);
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].url, Product::SaxonyDtm.tile_url(tiles[0]));
        assert_eq!(fetched[0].currency.published, "2024-11-30".parse().unwrap());
        assert_eq!(fetched[1].currency.published, "2020-11-30".parse().unwrap());
        assert!(
            fetched[0]
                .zip
                .starts_with(dir.path().join("geosn").join("dgm1")),
            "{:?}",
            fetched[0].zip
        );
        assert!(fetched[0].zip.exists());
    }

    #[test]
    fn offline_uses_cached_index() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let online = MapFetcher::default();
        let tiles = serve_geosn(&online);
        let product = Product::SaxonyDtm;
        fetch_tiles(
            &online,
            &cache,
            product,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        let offline = MapFetcher::default();
        let fetched = fetch_tiles(
            &offline,
            &cache,
            product,
            &tiles,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(offline.calls.get(), 0);
        assert_eq!(fetched[1].currency.published, "2020-11-30".parse().unwrap());
    }

    #[test]
    fn geosn_tile_missing_from_the_index_is_not_covered() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve_geosn(&fetcher);
        let far = TileId::parse("33500-5600").unwrap();
        let err = fetch_tiles(
            &fetcher,
            &cache,
            Product::SaxonyDtm,
            &[far],
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::TilesNotCovered(t) if t == &[far]),
            "{err:?}"
        );
        assert!(!err.to_string().contains("LGB"), "{err}");
    }

    #[test]
    fn each_tile_succeeds_or_fails_alone() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetcher.insert(
            PRODUCT.currency_csv_url(),
            currency_csv(&[(TILE, "2026-01-26"), (EAST, "2026-01-26")]),
        );
        fetcher.failing.borrow_mut().push(PRODUCT.tile_url(EAST));
        let tiles = [TILE, EAST, NORTH];
        let results = fetch_tiles_each(
            &fetcher,
            &cache,
            PRODUCT,
            &tiles,
            FetchMode::Online,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, TILE);
        assert!(results[0].1.is_ok(), "{:?}", results[0].1);
        assert!(
            matches!(results[1].1, Err(FetchError::Http { .. })),
            "{:?}",
            results[1].1
        );
        assert!(
            matches!(&results[2].1, Err(FetchError::TilesNotCovered(t)) if t == &[NORTH]),
            "{:?}",
            results[2].1
        );

        let results = fetch_tiles_each(
            &fetcher,
            &cache,
            PRODUCT,
            &[TILE, EAST],
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert!(results[0].1.is_ok());
        assert!(
            matches!(&results[1].1, Err(FetchError::MissingOffline(_, t)) if t == &[EAST]),
            "{:?}",
            results[1].1
        );
    }

    #[test]
    fn all_uncovered_tiles_reported_before_downloading() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let err = fetch(&fetcher, &cache, &[TILE, NORTH, EAST], FetchMode::Online).unwrap_err();
        assert!(
            matches!(&err, FetchError::TilesNotCovered(t) if t == &[NORTH, EAST]),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("33421-5822") && message.contains("33422-5821"),
            "{message}"
        );
        assert!(
            message.contains("tiles 33421-5822, 33422-5821 are"),
            "{message}"
        );
        assert!(message.contains("buffer_m"), "{message}");
        assert_eq!(fetcher.calls.get(), 1, "only the currency CSV");
    }

    #[test]
    fn offline_uses_older_cached_publication() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        std::fs::write(
            cache.currency_csv_path(PRODUCT),
            currency_csv(&[(TILE, "2026-06-01")]),
        )
        .unwrap();

        let mut messages = Vec::new();
        let fetched = fetch_tiles(
            &fetcher,
            &cache,
            PRODUCT,
            &[TILE],
            FetchMode::Offline,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(fetched[0].currency.published.to_string(), "2026-01-26");
        assert_eq!(
            fetched[0].zip,
            cache.tile_path(PRODUCT, TILE, "2026-01-26".parse().unwrap())
        );
        assert!(
            messages.contains(&"warning: using cached terrain model (DGM1) tile 33421-5821 from 2026-01-26 (newer 2026-06-01 not downloaded)".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn corrupt_member_data_is_not_cached() {
        use std::io::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(PRODUCT.raster_member(TILE), stored).unwrap();
        zip.write_all(b"RASTERDATA-RASTERDATA").unwrap();
        zip.start_file(PRODUCT.world_file_member(TILE), stored)
            .unwrap();
        zip.write_all(b"1\n0\n0\n-1\n421000.5\n5821999.5\n")
            .unwrap();
        let mut bytes = zip.finish().unwrap().into_inner();
        let at = bytes.windows(10).position(|w| w == b"RASTERDATA").unwrap();
        bytes[at] = b'X';
        fetcher.insert(&PRODUCT.tile_url(TILE), bytes);

        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CorruptArchive { .. }), "{err:?}");
        assert_eq!(cached_files(&cache), ["currency.csv"]);
    }

    #[test]
    fn stale_part_files_are_removed() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let product_dir = cache
            .currency_csv_path(PRODUCT)
            .parent()
            .unwrap()
            .to_path_buf();
        std::fs::create_dir_all(&product_dir).unwrap();
        let two_hours_ago = std::time::SystemTime::now() - Duration::from_secs(2 * 3600);
        for stale in [
            "33421-5821_2026-01-26.zip.part",
            "33421-5821_2025-01-01.zip.999999.part",
            "currency.csv.999999.part",
        ] {
            let path = product_dir.join(stale);
            std::fs::write(&path, b"partial").unwrap();
            let file = std::fs::File::options().write(true).open(&path).unwrap();
            file.set_modified(two_hours_ago).unwrap();
        }
        // Another process is still writing this one.
        let running = "33421-5821_2026-01-26.zip.888888.part";
        std::fs::write(product_dir.join(running), b"partial").unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        assert_eq!(
            cached_files(&cache),
            [
                "33421-5821_2026-01-26.zip",
                "33421-5821_2026-01-26.zip.888888.part",
                "currency.csv"
            ]
        );
    }

    #[test]
    fn offline_fetch_succeeds_from_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        let online = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();
        let calls = fetcher.calls.get();
        let offline = fetch(&fetcher, &cache, &[TILE], FetchMode::Offline).unwrap();
        assert_eq!(offline, online);
        assert_eq!(fetcher.calls.get(), calls);
    }

    /// A small valid LAZ point cloud.
    fn las_bytes() -> Vec<u8> {
        let source = MapFetcher::default();
        test_support::serve_synthetic_ndsm(&source, TILE);
        source.get(&ndsm_tile_url(TILE)).unwrap()
    }

    fn ndsm(fetcher: &MapFetcher, cache: &Cache, mode: FetchMode) -> (Vec<NdsmTile>, Vec<String>) {
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(fetcher, cache, &[TILE, NORTH], mode, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        (tiles, messages)
    }

    #[test]
    #[ignore = "network"]
    fn live_ndsm_last_modified() {
        let date = HttpFetcher::new()
            .last_modified(&ndsm_tile_url(TILE))
            .unwrap();
        assert!(
            date.is_some_and(|d| d.to_string().as_str() >= "2025-01-01"),
            "{date:?}"
        );
        let missing = TileId {
            east_km: 999,
            north_km: 5999,
        };
        let err = HttpFetcher::new()
            .last_modified(&ndsm_tile_url(missing))
            .unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err:?}");
    }

    #[test]
    fn ndsm_server_error_falls_back_to_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let url = ndsm_tile_url(TILE);
        fetcher.insert(&url, las_bytes());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        fetcher.failing.borrow_mut().push(url);
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(
            &fetcher,
            &cache,
            &[TILE, NORTH],
            FetchMode::Online,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(tiles.len(), 1, "the cached tile is used");
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: nDSM tile 33421-5821 unavailable")),
            "{messages:?}"
        );
    }

    #[test]
    fn truncated_laz_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let mut bytes = std::fs::read(cache_free_laz(dir.path())).unwrap();
        bytes.truncate(bytes.len() / 2);
        fetcher.insert(&ndsm_tile_url(TILE), bytes);
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(tiles.is_empty());
        assert!(
            messages
                .iter()
                .any(|m| m.contains("not a complete LAS point cloud")),
            "{messages:?}"
        );
    }

    /// A small valid LAZ file written outside the cache.
    fn cache_free_laz(dir: &Path) -> PathBuf {
        let source = MapFetcher::default();
        test_support::serve_synthetic_ndsm(&source, TILE);
        let path = dir.join("source.laz");
        std::fs::write(&path, source.get(&ndsm_tile_url(TILE)).unwrap()).unwrap();
        path
    }

    #[test]
    fn ndsm_downloaded_then_reused() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        for tile in [TILE, NORTH] {
            let url = ndsm_tile_url(tile);
            fetcher.insert(&url, las_bytes());
            fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        }
        let (first, _) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].published, "2025-03-03".parse().ok());
        assert!(
            first[0].path.ends_with("33421-5821_2025-03-03.laz"),
            "{:?}",
            first[0].path
        );
        let calls = fetcher.calls.get();
        let (again, _) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(again, first);
        assert_eq!(fetcher.calls.get(), calls + 2, "only the two date checks");
    }

    #[test]
    fn ndsm_republished_replaces_old() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let url = ndsm_tile_url(TILE);
        fetcher.insert(&url, las_bytes());
        fetcher.set_modified(&url, "2025-03-03".parse().unwrap());
        fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        fetcher.set_modified(&url, "2026-09-01".parse().unwrap());
        let tiles =
            fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |_| {}).unwrap();
        let names: Vec<String> = std::fs::read_dir(tiles[0].path.parent().unwrap())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["33421-5821_2026-09-01.laz"]);
    }

    #[test]
    fn missing_ndsm_tile_is_skipped_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), las_bytes());
        let (tiles, messages) = ndsm(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].published, None, "no Last-Modified");
        assert!(
            messages.contains(&"no nDSM point cloud published for tile 33421-5822".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn offline_uses_cached_ndsm() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), las_bytes());
        fetcher.set_modified(&ndsm_tile_url(TILE), "2025-03-03".parse().unwrap());
        ndsm(&fetcher, &cache, FetchMode::Online);
        let calls = fetcher.calls.get();
        let (tiles, messages) = ndsm(&fetcher, &cache, FetchMode::Offline);
        assert_eq!(fetcher.calls.get(), calls);
        assert_eq!(tiles.len(), 1);
        assert!(
            messages.contains(&"warning: no cached nDSM for tile 33421-5822".to_string()),
            "{messages:?}"
        );
    }

    #[test]
    fn not_a_las_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(&ndsm_tile_url(TILE), b"<html>maintenance</html>".to_vec());
        let mut messages = Vec::new();
        let tiles = fetch_ndsm_tiles(&fetcher, &cache, &[TILE], FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(tiles.is_empty(), "skipped, not cached");
        assert!(
            messages
                .iter()
                .any(|m| m.contains("not a complete LAS point cloud")),
            "{messages:?}"
        );
        assert!(cached_ndsm(&cache.ndsm_dir(), TILE).is_empty());
    }

    #[test]
    fn test_fetcher_prefers_exact_then_longest_prefix() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://a.example/", b"short".to_vec());
        fetcher.insert_prefix("https://a.example/items", b"long".to_vec());
        fetcher.insert("https://a.example/items?x=1", b"exact".to_vec());
        assert_eq!(
            fetcher.get("https://a.example/items?x=1").unwrap(),
            b"exact"
        );
        assert_eq!(fetcher.get("https://a.example/items?x=2").unwrap(), b"long");
        assert_eq!(fetcher.get("https://a.example/other").unwrap(), b"short");
        assert!(fetcher.get("https://b.example/").is_err());
    }

    #[test]
    fn invalid_csv_download_keeps_cached_csv() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-01-26");
        fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap();

        fetcher.insert(
            PRODUCT.currency_csv_url(),
            b"<html>captive portal</html>".to_vec(),
        );
        let err = fetch(&fetcher, &cache, &[TILE], FetchMode::Online).unwrap_err();
        assert!(matches!(err, FetchError::CurrencyCsv(_)), "{err:?}");
        let offline = fetch(&fetcher, &cache, &[TILE], FetchMode::Offline).unwrap();
        assert_eq!(offline[0].currency.published.to_string(), "2026-01-26");
    }

    #[test]
    fn ignored_range_is_an_error() {
        // A server that answers every request with the whole file (200, not 206).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/big.zip", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            use std::io::{Read as _, Write as _};
            for stream in listener.incoming().take(2).flatten() {
                let mut stream = stream;
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let body = vec![b'x'; 10_000];
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        let fetcher = HttpFetcher::new();
        let err = fetcher
            .get_range(
                &url,
                ByteRange::From {
                    start: 100,
                    len: 10,
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("byte range"), "{err}");
    }

    /// A server answering every request with `status` and `body`; counts the connections.
    fn answering(
        status: &'static str,
        body: Vec<u8>,
    ) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/big.zip", listener.local_addr().unwrap());
        let connections = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = connections.clone();
        std::thread::spawn(move || {
            use std::io::{Read as _, Write as _};
            for stream in listener.incoming().flatten() {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut stream = stream;
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        (url, connections)
    }

    #[test]
    fn only_a_report_root_is_an_exception_report() {
        for report in [
            &b"<ExceptionReport><Exception exceptionCode=\"A\"/></ExceptionReport>"[..],
            b"\xef\xbb\xbf  <?xml version=\"1.0\"?>\n<!-- c -->\n<!-- d --><ows:ExceptionReport xmlns:ows=\"x\">",
            b"<?xml version=\"1.0\"?>\n<!DOCTYPE ServiceExceptionReport SYSTEM \"http://x/y.dtd\" [ <!ELEMENT a ANY> ]>\n<ServiceExceptionReport version=\"1.1.1\">",
            b"<?xml version=\"1.0\"?><ServiceExceptionReport version=\"1.3.0\"><ServiceException>no</ServiceException></ServiceExceptionReport>",
        ] {
            assert!(is_ogc_report(report), "{}", String::from_utf8_lossy(report));
        }
        for other in [
            &b"<html><body>ExceptionReport</body></html>"[..],
            b"<!DOCTYPE html><html><pre><ows:ExceptionReport exceptionCode=\"X\"></pre></html>",
            b"<?xml version=\"1.0\"?><wfs:FeatureCollection><ExceptionReport/></wfs:FeatureCollection>",
            b"<ExceptionReports/>",
            b"",
            b"<!-- unclosed <ExceptionReport>",
        ] {
            assert!(!is_ogc_report(other), "{}", String::from_utf8_lossy(other));
            assert_eq!(ogc_exception(other), None);
        }
    }

    #[test]
    fn one_line_drops_controls_and_format_characters() {
        assert_eq!(
            one_line(
                "a\u{1b}[31m\u{202E}b\u{200B}c\u{2066}\n   d\u{FEFF}\u{200F}\u{2069}\u{202A}",
                200
            ),
            "a [31mbc d"
        );
        assert_eq!(one_line("x\u{061C}y\u{2060}z", 10), "xyz");
        assert_eq!(one_line("abc def", 5), "abc d…");
        assert_eq!(one_line("abcde", 5), "abcde");
    }

    #[test]
    fn http_refusal_carries_the_exception_report() {
        let report = br#"<?xml version="1.0" encoding="UTF-8"?>
<ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/2.0" version="2.0.1">
  <ows:Exception exceptionCode="ExtentError" locator="mapserv">
    <ows:ExceptionText>msWCSGetCoverage20(): WCS server error. Image extent
      does not intersect with desired region.</ows:ExceptionText>
  </ows:Exception>
</ows:ExceptionReport>"#;
        let fetcher = HttpFetcher::new();
        let (url, _) = answering("400 Bad Request", report.to_vec());
        let err = fetcher.get(&url).unwrap_err();
        let FetchError::Refused { status, detail, .. } = &err else {
            panic!("{err:?}")
        };
        assert_eq!(*status, 400);
        assert_eq!(
            detail.as_deref(),
            Some(
                "ExtentError: msWCSGetCoverage20(): WCS server error. Image extent does not \
                 intersect with desired region."
            )
        );
        assert!(err.to_string().ends_with("(HTTP 400): ExtentError: msWCSGetCoverage20(): WCS server error. Image extent does not intersect with desired region."), "{err}");
        // Any other body adds nothing; 404 and 5xx stay what they were.
        let (url, _) = answering("403 Forbidden", b"<html>\x1b[31mno</html>".to_vec());
        let err = fetcher.get(&url).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("{url}: the server refused the request (HTTP 403)")
        );
        let (url, _) = answering("404 Not Found", b"missing".to_vec());
        assert!(matches!(fetcher.get(&url), Err(FetchError::NotFound(_))));
        let (url, _) = answering("503 Service Unavailable", report.to_vec());
        let err = fetcher.get(&url).unwrap_err();
        assert!(
            matches!(&err, FetchError::Http { message, .. } if message == "http status: 503"),
            "{err:?}"
        );
        // A WMS report.
        assert_eq!(
            ogc_exception(
                br#"<ServiceExceptionReport version="1.3.0"><ServiceException code="InvalidDimensionValue">
                  Image size out of range</ServiceException></ServiceExceptionReport>"#
            )
            .as_deref(),
            Some("InvalidDimensionValue: Image size out of range")
        );
    }

    #[test]
    fn download_over_the_limit_is_an_error_and_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("downloads/tile.part");
        // In-memory (the trait's default), then streamed over HTTP.
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.test/tile", vec![b'x'; 10_000]);
        fetcher
            .download_to("https://example.test/tile", &path, 10_000, &mut |_| {})
            .unwrap();
        assert_eq!(std::fs::read(&path).unwrap().len(), 10_000);
        let err = fetcher
            .download_to("https://example.test/tile", &path, 9_999, &mut |_| {})
            .unwrap_err();
        assert!(matches!(err, FetchError::TooLarge { .. }), "{err}");
        assert!(!path.exists());
        for length in [true, false] {
            let (url, connections) = if length {
                answering("200 OK", vec![b'x'; 300_000])
            } else {
                streaming_without_length(vec![b'x'; 300_000])
            };
            let mut messages = Vec::new();
            let err = download_with_retries(&HttpFetcher::new(), &url, &path, 200_000, &mut |m| {
                messages.push(m.to_string())
            })
            .unwrap_err();
            assert!(matches!(err, FetchError::TooLarge { .. }), "{err}");
            assert!(err.to_string().contains("larger than"), "{err}");
            assert!(!path.exists(), "length {length}");
            // Asking again does not make it smaller.
            assert_eq!(connections.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
    }

    /// A file with room for `room` bytes; then it fails like a full disk.
    struct FullDisk {
        file: std::fs::File,
        room: usize,
    }

    impl std::io::Write for FullDisk {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.room == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::StorageFull));
            }
            let n = buf.len().min(self.room);
            self.file.write_all(&buf[..n])?;
            self.room -= n;
            Ok(n)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.file.flush()
        }
    }

    #[test]
    fn storage_full_during_download_removes_the_part_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("osm/extract.osm.pbf.part");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let body = vec![b'x'; 200_000];
        let full_disk = |p: &Path| -> std::io::Result<Box<dyn std::io::Write>> {
            Ok(Box::new(FullDisk {
                file: std::fs::File::create(p)?,
                room: 100_000,
            }))
        };
        let err = stream_to_file(
            "https://example.test/extract",
            &mut body.as_slice(),
            Some(200_000),
            &path,
            &full_disk,
            MAX_DOWNLOAD_BYTES,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(err.is_disk_full(), "{err:?}");
        assert_eq!(
            err.to_string(),
            format!(
                "the disk is full, so freemapper could not write {}. Free some space and try again",
                path.display()
            )
        );
        assert!(!path.exists());
    }

    /// Fails every download like a full disk and counts the attempts.
    #[derive(Default)]
    struct FullDiskFetcher {
        attempts: std::cell::Cell<u32>,
    }

    impl Fetcher for FullDiskFetcher {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            Err(FetchError::NotFound(url.to_string()))
        }

        fn download_to(
            &self,
            _url: &str,
            path: &Path,
            _max_bytes: u64,
            _progress: &mut dyn FnMut(&str),
        ) -> Result<(), FetchError> {
            self.attempts.set(self.attempts.get() + 1);
            Err(FetchError::Io(
                path.to_path_buf(),
                std::io::Error::from(std::io::ErrorKind::StorageFull),
            ))
        }
    }

    #[test]
    fn storage_full_is_not_retried() {
        let dir = tempfile::tempdir().unwrap();
        let part = dir.path().join("tile.part");
        let fetcher = FullDiskFetcher::default();
        let err = download_with_retries(&fetcher, "https://example.test/t", &part, 10, &mut |_| {})
            .unwrap_err();
        assert!(err.is_disk_full(), "{err:?}");
        assert_eq!(fetcher.attempts.get(), 1);
        let fetcher = FullDiskFetcher::default();
        let err = get_limited_with_retries(&fetcher, "https://example.test/t", &part, 10, |b| {
            Ok(b.to_vec())
        })
        .unwrap_err();
        assert!(err.is_disk_full(), "{err:?}");
        assert_eq!(fetcher.attempts.get(), 1);
    }

    /// A server answering every request with `body` without a Content-Length.
    fn streaming_without_length(
        body: Vec<u8>,
    ) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/big.tif", listener.local_addr().unwrap());
        let connections = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = connections.clone();
        std::thread::spawn(move || {
            use std::io::{Read as _, Write as _};
            for stream in listener.incoming().flatten() {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut stream = stream;
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let _ = write!(stream, "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n");
                let _ = stream.write_all(&body);
            }
        });
        (url, connections)
    }

    #[test]
    fn ignored_range_is_not_retried() {
        let (url, connections) = answering("200 OK", vec![b'x'; 10_000]);
        let err = get_range_with_retries(
            &HttpFetcher::new(),
            &url,
            ByteRange::From {
                start: 100,
                len: 10,
            },
        )
        .unwrap_err();
        assert!(matches!(err, FetchError::RangeIgnored { .. }), "{err}");
        assert_eq!(connections.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn partial_answer_of_the_wrong_length_is_an_error() {
        let (url, _) = answering("206 Partial Content", vec![b'x'; 20]);
        let err = get_range_with_retries(
            &HttpFetcher::new(),
            &url,
            ByteRange::From {
                start: 100,
                len: 10,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("got 11 bytes"), "{err}");
    }

    #[test]
    fn range_reads_are_retried() {
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/f", b"0123456789".to_vec());
        fetcher
            .flaky
            .borrow_mut()
            .insert("https://example.org/f".to_string(), 2);
        let bytes = get_range_with_retries(
            &fetcher,
            "https://example.org/f",
            ByteRange::From { start: 2, len: 3 },
        )
        .unwrap();
        assert_eq!(bytes, b"234");
    }

    /// Answers the tail range empty this many times, then with the real bytes.
    struct EmptyTail {
        empty_left: Cell<u32>,
        calls: Cell<u32>,
    }

    impl Fetcher for EmptyTail {
        fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
            unreachable!()
        }

        fn get_range(&self, _url: &str, _range: ByteRange) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            if self.empty_left.get() > 0 {
                self.empty_left.set(self.empty_left.get() - 1);
                return Ok(Vec::new());
            }
            Ok(b"tail".to_vec())
        }
    }

    #[test]
    fn empty_tail_answer_is_retried() {
        let fetcher = EmptyTail {
            empty_left: Cell::new(1),
            calls: Cell::new(0),
        };
        let bytes = get_range_with_retries(&fetcher, "https://example.org/f", ByteRange::Last(100))
            .unwrap();
        assert_eq!(bytes, b"tail");
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn tail_answer_that_stays_empty_is_a_clear_error() {
        let fetcher = EmptyTail {
            empty_left: Cell::new(99),
            calls: Cell::new(0),
        };
        let err = get_range_with_retries(&fetcher, "https://example.org/f", ByteRange::Last(100))
            .unwrap_err();
        assert!(err.to_string().contains("got 0 bytes"), "{err}");
        assert_eq!(fetcher.calls.get(), 3);
    }

    #[test]
    fn short_range_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/f", b"0123456789".to_vec());
        let err = get_range_with_retries(
            &fetcher,
            "https://example.org/f",
            ByteRange::From { start: 8, len: 5 },
        )
        .unwrap_err();
        assert!(err.to_string().contains("got 2 bytes"), "{err}");
    }

    #[test]
    fn empty_range_needs_no_request() {
        // Nothing listens on port 9 (discard): a request would fail.
        let bytes = HttpFetcher::new()
            .get_range("http://127.0.0.1:9/f", ByteRange::From { start: 5, len: 0 })
            .unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn default_range_slices_the_answer() {
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/f", b"0123456789".to_vec());
        let url = "https://example.org/f";
        assert_eq!(
            fetcher
                .get_range(url, ByteRange::From { start: 2, len: 3 })
                .unwrap(),
            b"234"
        );
        assert_eq!(fetcher.get_range(url, ByteRange::Last(4)).unwrap(), b"6789");
    }

    #[test]
    fn stalled_server_times_out() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/tile.zip", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            // Accept and hold connections without ever answering.
            let _held: Vec<_> = listener.incoming().take(4).collect();
            std::thread::sleep(std::time::Duration::from_secs(30));
        });
        let fetcher = HttpFetcher::with_timeout(std::time::Duration::from_millis(300));
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || sender.send(fetcher.get(&url)));
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("fetch did not time out");
        assert!(matches!(result, Err(FetchError::Http { .. })), "{result:?}");
    }

    #[test]
    #[ignore = "network"]
    fn live_school_tile_currency() {
        let text = HttpFetcher::new()
            .get(Product::Orthophoto.currency_csv_url())
            .unwrap();
        let table = parse_currency_csv(&String::from_utf8(text).unwrap()).unwrap();
        assert!(table.contains_key(&TILE));
    }

    /// SHA-256 fingerprints (as `openssl x509 -fingerprint -sha256` prints them) of the bundled
    /// intermediate and root.
    const INTERMEDIATE_SHA256: &str = "19:9A:B2:AA:AF:FF:40:40:1E:0A:3B:7B:87:EE:99:64:65:9E:FF:A9:\
                                       4A:1F:EC:BE:91:8A:E1:36:E4:B4:E0:A8";
    const ROOT_SHA256: &str = "E5:9A:AA:81:60:09:C2:2B:FF:5B:25:BA:D3:7D:F3:06:F0:49:79:7C:1F:81:\
                               D8:5A:B0:89:E6:57:BD:8F:00:44";

    const PORTAL_OVERVIEW: &str = "https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/\
        dladownload/_ajax/overview.php?bbox=565100,5949100,565300,5949300&crs=EPSG:25832\
        &type[]=dgm1";

    fn fingerprint(der: &[u8]) -> String {
        use sha2::Digest as _;
        sha2::Sha256::digest(der)
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":")
    }

    fn root_certs(agent: &ureq::Agent) -> &ureq::tls::RootCerts {
        agent.config().tls_config().root_certs()
    }

    fn same_timeouts(a: &ureq::Agent, b: &ureq::Agent) -> bool {
        let (a, b) = (a.config().timeouts(), b.config().timeouts());
        (a.connect, a.recv_response, a.recv_body) == (b.connect, b.recv_response, b.recv_body)
    }

    #[test]
    fn portal_host_uses_the_bundled_anchors() {
        let anchors: Vec<_> = portal_anchors()
            .iter()
            .map(|cert| fingerprint(cert.der()))
            .collect();
        assert_eq!(
            anchors,
            [
                INTERMEDIATE_SHA256.replace(' ', ""),
                ROOT_SHA256.replace(' ', "")
            ]
        );
        let fetcher = HttpFetcher::new();
        for url in [
            PORTAL_OVERVIEW,
            "https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/massen.php?\
             file=bdom20nc_32_565_5949_1_sh_2023.tif&id=6&live=2023&km=32560_5940&stack=32565_5949",
            "https://GEODATEN.Schleswig-Holstein.de/",
            "https://geodaten.schleswig-holstein.de:443/x",
            "http://geodaten.schleswig-holstein.de/x",
            // A fully qualified name is the same host.
            "https://geodaten.schleswig-holstein.de./x",
        ] {
            assert_eq!(trust_for(url), Trust::PortalAnchors, "{url}");
            let agents = fetcher.agents(url);
            for (agent, default) in [
                (&agents.normal, &fetcher.default.normal),
                (&agents.large, &fetcher.default.large),
            ] {
                let ureq::tls::RootCerts::Specific(certs) = root_certs(agent) else {
                    panic!("{url}: {:?}", root_certs(agent));
                };
                let pinned: Vec<_> = certs.iter().map(|c| fingerprint(c.der())).collect();
                assert_eq!(pinned, anchors, "{url}");
                assert!(!agent.config().tls_config().disable_verification());
                assert!(same_timeouts(agent, default), "{url}");
            }
        }
    }

    #[test]
    fn other_hosts_keep_the_default_roots() {
        let fetcher = HttpFetcher::new();
        for url in [
            "https://geodaten.schleswig-holstein.de.evil.example/x",
            "https://xgeodaten.schleswig-holstein.de/x",
            "https://evil.geodaten.schleswig-holstein.de/x",
            "https://geodaten.schleswig-holstein.de@evil.example/x",
            "https://evil.example/geodaten.schleswig-holstein.de/x",
            "https://evil.example/?next=https://geodaten.schleswig-holstein.de/",
            "https://service.gdi-sh.de/INSPIRE_Atom/EL/data/dgm1_32_565_5949_1_sh_2022.tif",
            "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD",
            "https://opendata.schleswig-holstein.de/api/3/action/package_show",
            "geodaten.schleswig-holstein.de",
            "https://geodaten.schleswig-holstein.de../x",
            "https://geodaten.schleswig-holstein.de.evil./x",
            "not a url",
        ] {
            assert_eq!(trust_for(url), Trust::DefaultRoots, "{url}");
            let agents = fetcher.agents(url);
            for agent in [&agents.normal, &agents.large] {
                assert!(
                    matches!(root_certs(agent), ureq::tls::RootCerts::WebPki),
                    "{url}: {:?}",
                    root_certs(agent)
                );
                assert!(!agent.config().tls_config().disable_verification());
            }
        }
        // The default agents are configured as before the portal agents existed.
        let plain: ureq::Agent = ureq::Agent::config_builder().build().into();
        for agent in [&fetcher.default.normal, &fetcher.default.large] {
            assert_eq!(
                format!("{:?}", agent.config().tls_config()),
                format!("{:?}", plain.config().tls_config())
            );
        }
    }

    #[test]
    fn portal_agents_follow_no_redirects() {
        let fetcher = HttpFetcher::new();
        for agent in [&fetcher.portal.normal, &fetcher.portal.large] {
            assert_eq!(agent.config().max_redirects(), 0);
        }
        // The other hosts follow redirects as before.
        let plain: ureq::Agent = ureq::Agent::config_builder().build().into();
        for agent in [&fetcher.default.normal, &fetcher.default.large] {
            assert_eq!(
                agent.config().max_redirects(),
                plain.config().max_redirects()
            );
        }
        // A redirect the portal agents return is an error naming the host, for every kind of
        // request (here a local server stands in for the portal).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/massen.php", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            use std::io::{Read as _, Write as _};
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let _ = write!(
                    stream,
                    "HTTP/1.1 302 Found\r\nLocation: https://evil.example/x\r\n\
                     Content-Length: 0\r\nConnection: close\r\n\r\n"
                );
            }
        });
        let pinned = HttpFetcher {
            default: Agents {
                normal: fetcher.portal.normal.clone(),
                large: fetcher.portal.large.clone(),
            },
            portal: fetcher.portal,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.part");
        let errors = [
            pinned.get(&url).unwrap_err(),
            pinned
                .get_range(&url, ByteRange::From { start: 0, len: 4 })
                .unwrap_err(),
            pinned.last_modified(&url).unwrap_err(),
            pinned
                .download_to(&url, &path, 1000, &mut |_| {})
                .unwrap_err(),
        ];
        for err in errors {
            let text = err.to_string();
            assert!(matches!(err, FetchError::Http { .. }), "{text}");
            assert!(text.contains(PORTAL_HOST), "{text}");
            assert!(
                text.contains("redirect (HTTP 302) to https://evil.example/x"),
                "{text}"
            );
        }
        assert!(!path.exists());
    }

    /// The portal's leaf-only chain verifies with the bundled anchors.
    #[test]
    #[ignore = "network"]
    fn live_portal_overview_verifies() {
        let bytes = HttpFetcher::new().get(PORTAL_OVERVIEW).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["success"], true, "{json}");
        assert!(
            json["result"]["features"]
                .as_array()
                .is_some_and(|f| !f.is_empty()),
            "{json}"
        );
    }

    /// The bundled anchors do not verify a chain to another CA (Let's Encrypt here).
    #[test]
    #[ignore = "network"]
    fn live_portal_anchors_refuse_other_chains() {
        let fetcher = HttpFetcher::new();
        let result = fetcher
            .portal
            .normal
            .get("https://opendata.schleswig-holstein.de/api/3/action/status_show")
            .call();
        let err = result.expect_err("a Let's Encrypt chain must not verify");
        assert!(
            err.to_string().to_lowercase().contains("certificate"),
            "{err}"
        );
    }
}
