//! The fetchers of the golden net (layers G3 and G4): one records the answers of the services,
//! one plays them back without a network.
//!
//! The repository keeps an index per state (`tests/golden/adapters/<id>/index.toml`): every
//! request with the SHA-256 of its answer, or with the error it got. The answers themselves are
//! too large for the repository and live in the fixture store, one file per hash under
//! `responses/`, so an answer two states share is kept once. See
//! `tests/golden/adapters/README.md`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use freemapper::fetch::{ByteRange, FetchError, Fetcher, HttpFetcher, USER_AGENT};
use freemapper::note::Note;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// A valid OSM extract with a header block and no data. Every `.osm.pbf` URL is answered with
/// it: an extract has 50 to 500 MB, and the adapter work does not touch the PBF reader.
const EMPTY_PBF: &[u8] = include_bytes!("../../../../tests/golden/adapters/empty.osm.pbf");

/// The requests and warnings of a run, in the order they happened.
pub type Log = Arc<Mutex<Vec<String>>>;

/// One request and what the service answered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recorded {
    /// `GET` or `HEAD`.
    pub method: String,
    pub url: String,
    /// The `Range` header of a range request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// SHA-256 of the answer, the name of its file in the store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// The `Last-Modified` date a `HEAD` answered with, if it had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<NaiveDate>,
    /// `not-found`, `refused`, `http`, `range-ignored` or `too-large`: the request failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_mb: Option<u64>,
}

impl Recorded {
    fn request(method: &str, url: &str, range: Option<String>) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            range,
            body: None,
            bytes: None,
            last_modified: None,
            error: None,
            status: None,
            detail: None,
            message: None,
            limit_mb: None,
        }
    }

    fn key(&self) -> Key {
        (self.method.clone(), self.url.clone(), self.range.clone())
    }

    /// The error of a failed request as the index keeps it. Only what a service can answer is
    /// kept; a full disk or a cancel is no answer.
    fn failed(mut self, error: &FetchError) -> Self {
        match error {
            FetchError::NotFound(_) => self.error = Some("not-found".to_string()),
            FetchError::Refused { status, detail, .. } => {
                self.error = Some("refused".to_string());
                self.status = Some(*status);
                self.detail = detail.clone();
            }
            FetchError::Http { message, .. } => {
                self.error = Some("http".to_string());
                self.message = Some(message.clone());
            }
            FetchError::RangeIgnored { status, .. } => {
                self.error = Some("range-ignored".to_string());
                self.status = Some(*status);
            }
            FetchError::TooLarge { limit_mb, .. } => {
                self.error = Some("too-large".to_string());
                self.limit_mb = Some(*limit_mb);
            }
            other => panic!("{}: not an answer of a service: {other}", self.url),
        }
        self
    }

    /// The error the request failed with, if it did.
    fn error(&self) -> Option<FetchError> {
        let url = self.url.clone();
        Some(match self.error.as_deref()? {
            "not-found" => FetchError::NotFound(url),
            "refused" => FetchError::Refused {
                url,
                status: self.status.unwrap_or(400),
                detail: self.detail.clone(),
            },
            "http" => FetchError::Http {
                url,
                message: self.message.clone().unwrap_or_default(),
            },
            "range-ignored" => FetchError::RangeIgnored {
                url,
                range: self.range.clone().unwrap_or_default(),
                status: self.status.unwrap_or(200),
            },
            "too-large" => FetchError::TooLarge {
                url,
                limit_mb: self.limit_mb.unwrap_or(0),
            },
            other => panic!("{url}: unknown error kind {other:?} in the index"),
        })
    }
}

/// Method, URL and range: what makes two requests the same.
type Key = (String, String, Option<String>);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    #[serde(default)]
    response: Vec<Recorded>,
}

/// The `Range` header [`HttpFetcher`] sends for `range`; `None` for an empty range, which sends
/// no request.
fn range_header(range: ByteRange) -> Option<String> {
    match range {
        ByteRange::From { len: 0, .. } | ByteRange::Last(0) => None,
        ByteRange::From { start, len } => Some(format!("bytes={start}-{}", start + len - 1)),
        ByteRange::Last(n) => Some(format!("bytes=-{n}")),
    }
}

fn is_extract(url: &str) -> bool {
    url.ends_with(".osm.pbf")
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn body_path(store: &Path, hash: &str) -> PathBuf {
    store.join("responses").join(format!("{hash}.bin"))
}

/// Writes `bytes` to the new file `path`, as the default [`Fetcher::download_to`] does.
fn write_download(url: &str, path: &Path, bytes: &[u8], max_bytes: u64) -> Result<(), FetchError> {
    if bytes.len() as u64 > max_bytes {
        let _ = std::fs::remove_file(path);
        return Err(FetchError::TooLarge {
            url: url.to_string(),
            limit_mb: max_bytes / (1024 * 1024),
        });
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
    }
    std::fs::write(path, bytes).map_err(|e| FetchError::Io(path.to_path_buf(), e))
}

/// Answers every request from an index and the store, and logs it as `METHOD url [range]`.
/// A request the index does not know fails like a server error (HTTP 404) and is logged too, so
/// a new request shows in the trace.
pub struct ReplayFetcher {
    answers: HashMap<Key, Recorded>,
    store: PathBuf,
    log: Log,
}

impl ReplayFetcher {
    pub fn new(index: &Path, store: &Path) -> Self {
        let text = std::fs::read_to_string(index)
            .unwrap_or_else(|e| panic!("{}: {e}; record the state first", index.display()));
        let index: Index =
            toml::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", index.display()));
        Self {
            answers: index.response.into_iter().map(|r| (r.key(), r)).collect(),
            store: store.to_path_buf(),
            log: Log::default(),
        }
    }

    /// The log this fetcher writes to; a caller adds its own lines (warnings) in between.
    pub fn log(&self) -> Log {
        Arc::clone(&self.log)
    }

    /// The lines logged so far.
    pub fn trace(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }

    fn answer(
        &self,
        logged: &str,
        method: &str,
        url: &str,
        range: Option<String>,
    ) -> Result<&Recorded, FetchError> {
        self.log.lock().unwrap().push(logged.to_string());
        let recorded = self
            .answers
            .get(&(method.to_string(), url.to_string(), range))
            .ok_or_else(|| FetchError::Http {
                url: url.to_string(),
                message: "http status: 404 (not in the recording)".to_string(),
            })?;
        match recorded.error() {
            Some(error) => Err(error),
            None => Ok(recorded),
        }
    }

    fn body(&self, recorded: &Recorded) -> Vec<u8> {
        let hash = recorded
            .body
            .as_deref()
            .unwrap_or_else(|| panic!("{}: the index names no body", recorded.url));
        let path = body_path(&self.store, hash);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "{}: {e}; the fixture store is incomplete: scripts/golden.sh fetch",
                path.display()
            )
        });
        assert_eq!(sha256(&bytes), hash, "{} is damaged", path.display());
        bytes
    }
}

impl Fetcher for ReplayFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let logged = format!("GET {url}");
        if is_extract(url) {
            self.log.lock().unwrap().push(logged);
            return Ok(EMPTY_PBF.to_vec());
        }
        Ok(self.body(self.answer(&logged, "GET", url, None)?))
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        let recorded = self.answer(&format!("HEAD {url}"), "HEAD", url, None)?;
        Ok(recorded.last_modified)
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let Some(header) = range_header(range) else {
            return Ok(Vec::new());
        };
        let logged = format!("GET {url} [{header}]");
        Ok(self.body(self.answer(&logged, "GET", url, Some(header))?))
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        _progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        let logged = format!("GET {url} [to file]");
        if is_extract(url) {
            self.log.lock().unwrap().push(logged);
            return write_download(url, path, EMPTY_PBF, max_bytes);
        }
        let bytes = self.body(self.answer(&logged, "GET", url, None)?);
        write_download(url, path, &bytes, max_bytes)
    }
}

/// Passes every request on to the network and keeps the answer: the body in the store, the
/// request with the body's hash in the index. An extract is passed on and not kept.
pub struct RecordingFetcher {
    inner: HttpFetcher,
    index: PathBuf,
    store: PathBuf,
    entries: Mutex<Vec<Recorded>>,
    log: Log,
}

impl RecordingFetcher {
    /// Records into `index` and `store`. Only when `FREEMAPPER_GOLDEN_RECORD` is set, and never
    /// with a user agent that could carry an e-mail address.
    pub fn new(inner: HttpFetcher, index: &Path, store: &Path) -> Self {
        assert!(
            std::env::var_os("FREEMAPPER_GOLDEN_RECORD").is_some(),
            "recording needs FREEMAPPER_GOLDEN_RECORD: scripts/golden.sh record <id>"
        );
        assert!(
            !USER_AGENT.contains('@'),
            "the user agent must not carry an address: {USER_AGENT}"
        );
        Self {
            inner,
            index: index.to_path_buf(),
            store: store.to_path_buf(),
            entries: Mutex::new(Vec::new()),
            log: Log::default(),
        }
    }

    pub fn log(&self) -> Log {
        Arc::clone(&self.log)
    }

    fn begin(&self, logged: String, url: &str) {
        assert!(!url.contains('@'), "no personal data in a request: {url}");
        self.log.lock().unwrap().push(logged);
    }

    /// Keeps `entry`. Asked twice, a request keeps its place; a good answer replaces an error
    /// (a retry that worked), an error never replaces a good answer.
    fn keep(&self, entry: Recorded) {
        let mut entries = self.entries.lock().unwrap();
        match entries.iter_mut().find(|e| e.key() == entry.key()) {
            Some(known) if entry.error.is_none() || known.error.is_some() => *known = entry,
            Some(_) => {}
            None => entries.push(entry),
        }
    }

    fn keep_body(&self, mut entry: Recorded, bytes: &[u8]) {
        let hash = sha256(bytes);
        let path = body_path(&self.store, &hash);
        if !path.exists() {
            std::fs::create_dir_all(path.parent().expect("a folder")).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
        entry.body = Some(hash);
        entry.bytes = Some(bytes.len() as u64);
        self.keep(entry);
    }

    /// Writes the index: the requests in the order they were first asked.
    pub fn save(&self) {
        let index = Index {
            response: self.entries.lock().unwrap().clone(),
        };
        std::fs::create_dir_all(self.index.parent().expect("a folder")).unwrap();
        std::fs::write(&self.index, toml::to_string(&index).unwrap()).unwrap();
    }
}

impl Fetcher for RecordingFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.begin(format!("GET {url}"), url);
        let answer = self.inner.get(url);
        if is_extract(url) {
            return answer;
        }
        let entry = Recorded::request("GET", url, None);
        match &answer {
            Ok(bytes) => self.keep_body(entry, bytes),
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        self.begin(format!("HEAD {url}"), url);
        let answer = self.inner.last_modified(url);
        let mut entry = Recorded::request("HEAD", url, None);
        match &answer {
            Ok(date) => {
                entry.last_modified = *date;
                self.keep(entry);
            }
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let Some(header) = range_header(range) else {
            return Ok(Vec::new());
        };
        self.begin(format!("GET {url} [{header}]"), url);
        let answer = self.inner.get_range(url, range);
        let entry = Recorded::request("GET", url, Some(header));
        match &answer {
            Ok(bytes) => self.keep_body(entry, bytes),
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        self.begin(format!("GET {url} [to file]"), url);
        let answer = self.inner.download_to(url, path, max_bytes, progress);
        if is_extract(url) {
            return answer;
        }
        let entry = Recorded::request("GET", url, None);
        match &answer {
            Ok(()) => self.keep_body(entry, &std::fs::read(path).unwrap()),
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }
}
