//! Recorded answers of services: one fetcher keeps what the services answer, one plays it back
//! without a network. The golden net of the adapters and the fixtures of an adapter use them.
//!
//! A recording is an index (`index.toml`: every request with the SHA-256 of its answer, or with
//! the error it got) and a folder with the answers, one file `<sha-256>.bin` per answer, so an
//! answer asked twice is kept once.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::{ByteRange, Fetcher};
use freemapper_adapter_api::note::Note;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// The name of the index in the folder of a recording.
pub const INDEX_FILE: &str = "index.toml";

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
    /// SHA-256 of the answer, the name of its file.
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

/// The kinds of error an index keeps.
const ERROR_KINDS: [&str; 5] = ["not-found", "refused", "http", "range-ignored", "too-large"];

impl Recorded {
    /// A request without its answer yet.
    pub fn request(method: &str, url: &str, range: Option<String>) -> Self {
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

    /// The request with the error it failed with, as the index keeps it. Only what a service
    /// can answer is kept: `None` for a full disk, a cancel or a request that was not sent.
    pub fn failed(mut self, error: &FetchError) -> Option<Self> {
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
            _ => return None,
        }
        Some(self)
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
            "range-ignored" => FetchError::RangeIgnored {
                url,
                range: self.range.clone().unwrap_or_default(),
                status: self.status.unwrap_or(200),
            },
            "too-large" => FetchError::TooLarge {
                url,
                limit_mb: self.limit_mb.unwrap_or(0),
            },
            // `http`; [`ReplayFetcher::open`] lets no other kind through.
            _ => FetchError::Http {
                url,
                message: self.message.clone().unwrap_or_default(),
            },
        })
    }
}

/// Method, URL and range: what makes two requests the same.
type Key = (String, String, Option<String>);

/// The index of a recording.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    #[serde(default)]
    pub response: Vec<Recorded>,
}

/// What is wrong with a recording.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("{}: {error}", .path.display())]
    Read {
        path: PathBuf,
        error: std::io::Error,
    },
    #[error("{}: not an index of recorded answers: {message}", .path.display())]
    Parse { path: PathBuf, message: String },
    /// An answer file is named by its SHA-256 and nothing else: an index cannot name a path.
    #[error(
        "{}: the answer of {url} is named {body:?}; an answer is named by its SHA-256 (64 hex digits)",
        .index.display()
    )]
    BodyName {
        index: PathBuf,
        url: String,
        body: String,
    },
    #[error("{}: {url} has the unknown error kind {kind:?}", .index.display())]
    ErrorKind {
        index: PathBuf,
        url: String,
        kind: String,
    },
}

/// The `Range` header a fetcher sends for `range`; `None` for an empty range, which sends no
/// request.
pub fn range_header(range: ByteRange) -> Option<String> {
    match range {
        ByteRange::From { len: 0, .. } | ByteRange::Last(0) => None,
        ByteRange::From { start, len } => Some(format!("bytes={start}-{}", start + len - 1)),
        ByteRange::Last(n) => Some(format!("bytes=-{n}")),
    }
}

fn is_extract(url: &str) -> bool {
    url.ends_with(".osm.pbf")
}

/// The SHA-256 of `bytes` in hex: the name of an answer.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_sha256_hex(name: &str) -> bool {
    name.len() == 64
        && name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The file of the answer `hash` in the folder `bodies`.
pub fn body_path(bodies: &Path, hash: &str) -> PathBuf {
    bodies.join(format!("{hash}.bin"))
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

/// Answers every request from an index and its answer files, and logs it as
/// `METHOD url [range]`. A request the index does not know fails like a server error and is
/// logged too, so a new request shows in the trace.
pub struct ReplayFetcher {
    answers: HashMap<Key, Recorded>,
    bodies: PathBuf,
    log: Log,
    /// What every `.osm.pbf` URL is answered with, if set.
    extract: Option<Vec<u8>>,
}

impl ReplayFetcher {
    /// The recording with the index `index` and the answer files in `bodies`. An index that
    /// names an answer by anything but its SHA-256 is refused: the name becomes a path.
    pub fn open(index: &Path, bodies: &Path) -> Result<Self, ReplayError> {
        let text = std::fs::read_to_string(index).map_err(|error| ReplayError::Read {
            path: index.to_path_buf(),
            error,
        })?;
        let parsed: Index = toml::from_str(&text).map_err(|e| ReplayError::Parse {
            path: index.to_path_buf(),
            message: e.to_string(),
        })?;
        for recorded in &parsed.response {
            if let Some(body) = recorded.body.as_ref().filter(|body| !is_sha256_hex(body)) {
                return Err(ReplayError::BodyName {
                    index: index.to_path_buf(),
                    url: recorded.url.clone(),
                    body: body.clone(),
                });
            }
            if let Some(kind) = recorded
                .error
                .as_ref()
                .filter(|kind| !ERROR_KINDS.contains(&kind.as_str()))
            {
                return Err(ReplayError::ErrorKind {
                    index: index.to_path_buf(),
                    url: recorded.url.clone(),
                    kind: kind.clone(),
                });
            }
        }
        Ok(Self {
            answers: parsed.response.into_iter().map(|r| (r.key(), r)).collect(),
            bodies: bodies.to_path_buf(),
            log: Log::default(),
            extract: None,
        })
    }

    /// The recording in the folder `dir`: its `index.toml` and the answer files beside it.
    pub fn from_dir(dir: &Path) -> Result<Self, ReplayError> {
        Self::open(&dir.join(INDEX_FILE), dir)
    }

    /// Answers every `.osm.pbf` URL with `bytes` and looks no such request up: an extract has
    /// 50 to 500 MB and is not part of a recording.
    pub fn with_extract(mut self, bytes: &[u8]) -> Self {
        self.extract = Some(bytes.to_vec());
        self
    }

    /// The answer files the index names that are not in the folder.
    pub fn missing_bodies(&self) -> Vec<PathBuf> {
        let mut missing: Vec<PathBuf> = self
            .answers
            .values()
            .filter_map(|recorded| recorded.body.as_deref())
            .map(|hash| body_path(&self.bodies, hash))
            .filter(|path| !path.exists())
            .collect();
        missing.sort();
        missing.dedup();
        missing
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

    fn body(&self, recorded: &Recorded) -> Result<Vec<u8>, FetchError> {
        let broken = |message: String| FetchError::Http {
            url: recorded.url.clone(),
            message,
        };
        let hash = recorded
            .body
            .as_deref()
            .ok_or_else(|| broken("the recording names no answer for this request".to_string()))?;
        let path = body_path(&self.bodies, hash);
        let bytes = std::fs::read(&path).map_err(|e| {
            broken(format!(
                "{}: {e}; the recording is incomplete",
                path.display()
            ))
        })?;
        if sha256_hex(&bytes) != hash {
            return Err(broken(format!(
                "{} is damaged: its content is not what its name says",
                path.display()
            )));
        }
        Ok(bytes)
    }
}

impl Fetcher for ReplayFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let logged = format!("GET {url}");
        if let Some(extract) = self.extract.as_ref().filter(|_| is_extract(url)) {
            self.log.lock().unwrap().push(logged);
            return Ok(extract.clone());
        }
        self.body(self.answer(&logged, "GET", url, None)?)
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
        self.body(self.answer(&logged, "GET", url, Some(header))?)
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        _progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        let logged = format!("GET {url} [to file]");
        if let Some(extract) = self.extract.as_ref().filter(|_| is_extract(url)) {
            self.log.lock().unwrap().push(logged);
            return write_download(url, path, extract, max_bytes);
        }
        let bytes = self.body(self.answer(&logged, "GET", url, None)?)?;
        write_download(url, path, &bytes, max_bytes)
    }
}

/// Passes every request on to `inner` and keeps the answer: the body as a file named by its
/// hash, the request with that hash in the index. An extract (`.osm.pbf`) is passed on and not
/// kept. A request whose URL carries an `@` is not sent: a recording is published, and an
/// address of a person has no place in it.
pub struct RecordingFetcher<F> {
    inner: F,
    index: PathBuf,
    bodies: PathBuf,
    entries: Mutex<Vec<Recorded>>,
    log: Log,
}

impl<F: Fetcher> RecordingFetcher<F> {
    /// Records what `inner` answers into the index `index` and the folder `bodies`.
    pub fn new(inner: F, index: &Path, bodies: &Path) -> Self {
        Self {
            inner,
            index: index.to_path_buf(),
            bodies: bodies.to_path_buf(),
            entries: Mutex::new(Vec::new()),
            log: Log::default(),
        }
    }

    pub fn log(&self) -> Log {
        Arc::clone(&self.log)
    }

    fn begin(&self, logged: String, url: &str) -> Result<(), FetchError> {
        if url.contains('@') {
            return Err(FetchError::NotAllowed {
                url: url.split(['?', '#']).next().unwrap_or(url).to_string(),
                reason: "a recording keeps no request with an @ in its address".to_string(),
            });
        }
        self.log.lock().unwrap().push(logged);
        Ok(())
    }

    /// Keeps `entry`. Asked twice, a request keeps its place; a good answer replaces an error
    /// (a retry that worked), an error never replaces a good answer.
    fn keep(&self, entry: Option<Recorded>) {
        let Some(entry) = entry else {
            return;
        };
        let mut entries = self.entries.lock().unwrap();
        match entries.iter_mut().find(|e| e.key() == entry.key()) {
            Some(known) if entry.error.is_none() || known.error.is_some() => *known = entry,
            Some(_) => {}
            None => entries.push(entry),
        }
    }

    fn keep_body(&self, mut entry: Recorded, bytes: &[u8]) -> Result<(), FetchError> {
        let hash = sha256_hex(bytes);
        let path = body_path(&self.bodies, &hash);
        if !path.exists() {
            std::fs::create_dir_all(&self.bodies)
                .map_err(|e| FetchError::Io(self.bodies.clone(), e))?;
            std::fs::write(&path, bytes).map_err(|e| FetchError::Io(path.clone(), e))?;
        }
        entry.body = Some(hash);
        entry.bytes = Some(bytes.len() as u64);
        self.keep(Some(entry));
        Ok(())
    }

    /// Writes the index: the requests in the order they were first asked.
    pub fn save(&self) -> std::io::Result<()> {
        let index = Index {
            response: self.entries.lock().unwrap().clone(),
        };
        if let Some(dir) = self.index.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string(&index).map_err(std::io::Error::other)?;
        std::fs::write(&self.index, text)
    }
}

impl<F: Fetcher> Fetcher for RecordingFetcher<F> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.begin(format!("GET {url}"), url)?;
        let answer = self.inner.get(url);
        if is_extract(url) {
            return answer;
        }
        let entry = Recorded::request("GET", url, None);
        match &answer {
            Ok(bytes) => self.keep_body(entry, bytes)?,
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        self.begin(format!("HEAD {url}"), url)?;
        let answer = self.inner.last_modified(url);
        let mut entry = Recorded::request("HEAD", url, None);
        match &answer {
            Ok(date) => {
                entry.last_modified = *date;
                self.keep(Some(entry));
            }
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let Some(header) = range_header(range) else {
            return Ok(Vec::new());
        };
        self.begin(format!("GET {url} [{header}]"), url)?;
        let answer = self.inner.get_range(url, range);
        let entry = Recorded::request("GET", url, Some(header));
        match &answer {
            Ok(bytes) => self.keep_body(entry, bytes)?,
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
        self.begin(format!("GET {url} [to file]"), url)?;
        let answer = self.inner.download_to(url, path, max_bytes, progress);
        if is_extract(url) {
            return answer;
        }
        let entry = Recorded::request("GET", url, None);
        match &answer {
            Ok(()) => {
                let bytes =
                    std::fs::read(path).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
                self.keep_body(entry, &bytes)?;
            }
            Err(e) => self.keep(entry.failed(e)),
        }
        answer
    }
}

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::testing::MapFetcher;

    use super::*;

    /// Records two answers and an error through a recorder and returns the folder.
    fn recorded() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let services = MapFetcher::default();
        services.insert("https://geo.example.org/a", b"first".to_vec());
        services.insert("https://geo.example.org/b", b"0123456789".to_vec());
        services
            .refusing
            .borrow_mut()
            .push("https://geo.example.org/refused".to_string());
        let recorder = RecordingFetcher::new(services, &dir.path().join(INDEX_FILE), dir.path());
        assert_eq!(recorder.get("https://geo.example.org/a").unwrap(), b"first");
        assert_eq!(recorder.get("https://geo.example.org/a").unwrap(), b"first");
        assert_eq!(
            recorder
                .get_range("https://geo.example.org/b", ByteRange::Last(4))
                .unwrap(),
            b"6789"
        );
        assert!(recorder.get("https://geo.example.org/refused").is_err());
        recorder.save().unwrap();
        dir
    }

    #[test]
    fn a_recording_plays_back_what_the_services_answered() {
        let dir = recorded();
        let replay = ReplayFetcher::from_dir(dir.path()).unwrap();
        assert!(replay.missing_bodies().is_empty());
        assert_eq!(replay.get("https://geo.example.org/a").unwrap(), b"first");
        assert_eq!(
            replay
                .get_range("https://geo.example.org/b", ByteRange::Last(4))
                .unwrap(),
            b"6789"
        );
        // An empty range asks nothing.
        assert!(
            replay
                .get_range("https://geo.example.org/b", ByteRange::Last(0))
                .unwrap()
                .is_empty()
        );
        let refused = replay.get("https://geo.example.org/refused").unwrap_err();
        assert!(
            matches!(refused, FetchError::Refused { status: 400, .. }),
            "{refused:?}"
        );
        assert_eq!(
            replay.trace(),
            [
                "GET https://geo.example.org/a",
                "GET https://geo.example.org/b [bytes=-4]",
                "GET https://geo.example.org/refused",
            ]
        );
        // The answer asked twice is kept once: two answer files and the index.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
        // An extract is answered with what the caller sets, and is no part of the recording.
        let replay = ReplayFetcher::from_dir(dir.path())
            .unwrap()
            .with_extract(b"pbf");
        let url = "https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf";
        assert_eq!(replay.get(url).unwrap(), b"pbf");
        let file = dir.path().join("out").join("extract.osm.pbf");
        replay.download_to(url, &file, 10, &mut |_| {}).unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"pbf");
    }

    #[test]
    fn replay_reports_unrecorded_requests_with_the_url() {
        let dir = recorded();
        let replay = ReplayFetcher::from_dir(dir.path()).unwrap();
        let error = replay.get("https://geo.example.org/new?x=1").unwrap_err();
        assert_eq!(
            error.to_string(),
            "download of https://geo.example.org/new?x=1 failed: http status: 404 (not in the recording)"
        );
        // The same URL with another range, or asked as HEAD, is another request.
        assert!(
            replay
                .get_range("https://geo.example.org/b", ByteRange::Last(5))
                .is_err()
        );
        assert!(replay.last_modified("https://geo.example.org/a").is_err());
        assert_eq!(replay.trace().len(), 3);
        // An answer file that is gone or changed is an error that names the file.
        let index: Index =
            toml::from_str(&std::fs::read_to_string(dir.path().join(INDEX_FILE)).unwrap()).unwrap();
        let hash = index.response[0].body.clone().unwrap();
        std::fs::write(body_path(dir.path(), &hash), b"changed").unwrap();
        let damaged = replay
            .get("https://geo.example.org/a")
            .unwrap_err()
            .to_string();
        assert!(damaged.contains("is damaged"), "{damaged}");
        std::fs::remove_file(body_path(dir.path(), &hash)).unwrap();
        assert_eq!(replay.missing_bodies(), [body_path(dir.path(), &hash)]);
        let missing = replay
            .get("https://geo.example.org/a")
            .unwrap_err()
            .to_string();
        assert!(missing.contains("the recording is incomplete"), "{missing}");
    }

    /// Review focus 5: the name of an answer in an index becomes a path. Only a SHA-256 is one.
    #[test]
    fn replay_rejects_index_entries_that_name_other_paths() {
        let dir = recorded();
        let index = dir.path().join(INDEX_FILE);
        let text = std::fs::read_to_string(&index).unwrap();
        let parsed: Index = toml::from_str(&text).unwrap();
        let hash = parsed.response[0].body.clone().unwrap();
        for name in [
            "../../etc/passwd",
            "/etc/passwd",
            "a/b",
            "",
            &hash[..63],
            &hash.to_uppercase(),
            &format!("{hash}.bin"),
        ] {
            std::fs::write(&index, text.replace(&hash, name)).unwrap();
            let refused = ReplayFetcher::from_dir(dir.path()).err().unwrap();
            assert!(
                matches!(refused, ReplayError::BodyName { .. }),
                "{name}: {refused}"
            );
        }
        std::fs::write(&index, text.replace("\"refused\"", "\"exploded\"")).unwrap();
        let refused = ReplayFetcher::from_dir(dir.path()).err().unwrap();
        assert!(
            matches!(refused, ReplayError::ErrorKind { .. }),
            "{refused}"
        );
        std::fs::write(&index, "[[response]]\nurl = 1\n").unwrap();
        assert!(matches!(
            ReplayFetcher::from_dir(dir.path()).err().unwrap(),
            ReplayError::Parse { .. }
        ));
        std::fs::remove_file(&index).unwrap();
        assert!(matches!(
            ReplayFetcher::from_dir(dir.path()).err().unwrap(),
            ReplayError::Read { .. }
        ));
    }

    /// A recording is published. A request with an address of a person in it is not sent and
    /// not kept; neither is an error that is no answer of a service.
    #[test]
    fn a_recording_keeps_no_personal_data_and_no_local_errors() {
        let dir = tempfile::tempdir().unwrap();
        let services = MapFetcher::default();
        services.insert_prefix("https://", b"x".to_vec());
        let recorder = RecordingFetcher::new(services, &dir.path().join(INDEX_FILE), dir.path());
        let refused = recorder
            .get("https://geo.example.org/x?mail=someone@example.org")
            .unwrap_err();
        assert!(
            matches!(refused, FetchError::NotAllowed { .. }),
            "{refused:?}"
        );
        assert!(!refused.to_string().contains("someone"), "{refused}");
        assert!(
            Recorded::request("GET", "u", None)
                .failed(&FetchError::Cancelled)
                .is_none()
        );
        recorder.save().unwrap();
        let index: Index =
            toml::from_str(&std::fs::read_to_string(dir.path().join(INDEX_FILE)).unwrap()).unwrap();
        assert!(index.response.is_empty());
    }
}
