//! The fetchers of the golden net (layers G3 and G4): one records the answers of the services,
//! one plays them back without a network. They are `freemapper_sources::replay`; this module
//! ties them to the fixture store.
//!
//! The repository keeps an index per state (`tests/golden/adapters/<id>/index.toml`): every
//! request with the SHA-256 of its answer, or with the error it got. The answers themselves are
//! too large for the repository and live in the fixture store, one file per hash under
//! `responses/`, so an answer two states share is kept once. See
//! `tests/golden/adapters/README.md`.

use std::path::Path;

use chrono::NaiveDate;
use freemapper::fetch::{ByteRange, FetchError, Fetcher, HttpFetcher, USER_AGENT};
use freemapper::note::Note;
pub use freemapper_sources::replay::ReplayFetcher;
use freemapper_sources::replay::range_header;

/// A valid OSM extract with a header block and no data. Every `.osm.pbf` URL is answered with
/// it: an extract has 50 to 500 MB, and the adapter work does not touch the PBF reader.
const EMPTY_PBF: &[u8] = include_bytes!("../../../../tests/golden/adapters/empty.osm.pbf");

/// The recording `index` played back from the fixture store `store`. An index that is not
/// there and a store that lacks an answer stop the test and say what to run.
pub fn replay(index: &Path, store: &Path) -> ReplayFetcher {
    let fetcher = ReplayFetcher::open(index, &store.join("responses"))
        .unwrap_or_else(|e| panic!("{e}; record the state first"))
        .with_extract(EMPTY_PBF);
    if let Some(missing) = fetcher.missing_bodies().first() {
        panic!(
            "{} is missing; the fixture store is incomplete: scripts/golden.sh fetch",
            missing.display()
        );
    }
    fetcher
}

/// A fetcher that loses the network: from the first request whose URL contains `marker` on,
/// every request fails like a refused connection. Before that it answers through `inner`, and
/// it logs the failed requests in the same log.
pub struct CutOff<'a> {
    inner: &'a ReplayFetcher,
    marker: &'a str,
    gone: std::cell::Cell<bool>,
}

impl<'a> CutOff<'a> {
    pub fn new(inner: &'a ReplayFetcher, marker: &'a str) -> Self {
        Self {
            inner,
            marker,
            gone: std::cell::Cell::new(false),
        }
    }

    /// Whether the network is gone for the request `logged` asks; if so, the error it gets.
    fn cut(&self, logged: String, url: &str) -> Option<FetchError> {
        if !self.gone.get() && !url.contains(self.marker) {
            return None;
        }
        self.gone.set(true);
        self.inner
            .log()
            .lock()
            .unwrap()
            .push(format!("{logged} (no network)"));
        Some(FetchError::Http {
            url: url.to_string(),
            message: "connection refused".to_string(),
        })
    }
}

impl Fetcher for CutOff<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match self.cut(format!("GET {url}"), url) {
            Some(error) => Err(error),
            None => self.inner.get(url),
        }
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        match self.cut(format!("HEAD {url}"), url) {
            Some(error) => Err(error),
            None => self.inner.last_modified(url),
        }
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let Some(header) = range_header(range) else {
            return Ok(Vec::new());
        };
        match self.cut(format!("GET {url} [{header}]"), url) {
            Some(error) => Err(error),
            None => self.inner.get_range(url, range),
        }
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        match self.cut(format!("GET {url} [to file]"), url) {
            Some(error) => Err(error),
            None => self.inner.download_to(url, path, max_bytes, progress),
        }
    }
}

/// The recorder of the golden net: the network through the HTTP fetcher of the program.
pub type RecordingFetcher = freemapper_sources::replay::RecordingFetcher<HttpFetcher>;

/// Records into `index` and `store`. Only when `FREEMAPPER_GOLDEN_RECORD` is set, and never
/// with a user agent that could carry an e-mail address.
pub fn record(inner: HttpFetcher, index: &Path, store: &Path) -> RecordingFetcher {
    assert!(
        std::env::var_os("FREEMAPPER_GOLDEN_RECORD").is_some(),
        "recording needs FREEMAPPER_GOLDEN_RECORD: scripts/golden.sh record <id>"
    );
    assert!(
        !USER_AGENT.contains('@'),
        "the user agent must not carry an address: {USER_AGENT}"
    );
    RecordingFetcher::new(inner, index, &store.join("responses"))
}
