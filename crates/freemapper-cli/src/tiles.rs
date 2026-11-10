//! The OpenStreetMap base layer of the area selection page, fetched by the local server.
//!
//! OpenStreetMap's tile servers refuse requests without a valid `Referer` or an identifying
//! `User-Agent` (<https://operations.osmfoundation.org/policies/tiles/>). The page does not send
//! the first (its `Referrer-Policy` keeps the secret path out of other servers' logs) and a
//! browser's own `User-Agent` is not ours, so the server fetches the tiles itself with a fixed
//! `User-Agent` that names freemapper and its repository, and nothing else. It keeps what it
//! fetched for as long as OpenStreetMap allows, so panning back costs no request.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio::sync::Semaphore;

/// Identifies freemapper to the tile servers. Fixed: no address or other personal data.
pub(crate) const USER_AGENT: &str = concat!(
    "freemapper/",
    env!("CARGO_PKG_VERSION"),
    " (+",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

const UPSTREAM: &str = "https://tile.openstreetmap.org";
/// The highest zoom the standard tile layer serves.
const MAX_ZOOM: u8 = 19;
/// OpenStreetMap asks for at most two connections at once.
const CONCURRENT_REQUESTS: usize = 2;
const TIMEOUT: Duration = Duration::from_secs(20);
/// Tiles are 5 to 30 KB; anything larger is not a tile.
const MAX_TILE_BYTES: u64 = 1 << 20;
/// Used when the answer carries no `max-age`; the tile policy asks for at least 7 days then.
const DEFAULT_LIFETIME: Duration = Duration::from_secs(7 * 24 * 3600);
/// The cache holds at most this many tiles and bytes (about 32 MB), oldest out first.
const CACHE_TILES: usize = 1024;
const CACHE_BYTES: usize = 32 << 20;

type Key = (u8, u32, u32);

/// One tile image.
#[derive(Debug, Clone)]
pub(crate) struct Tile {
    pub content_type: String,
    pub bytes: Vec<u8>,
    /// How long a browser may keep it: what is left of the upstream lifetime.
    pub remaining: Duration,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TileError {
    /// The request is not a tile address.
    BadRequest(String),
    /// OpenStreetMap could not be reached or refused.
    Upstream(String),
}

struct Entry {
    content_type: String,
    bytes: Vec<u8>,
    expires: Instant,
}

#[derive(Default)]
struct Store {
    tiles: HashMap<Key, Entry>,
    order: VecDeque<Key>,
    bytes: usize,
}

impl Store {
    fn get(&mut self, key: Key) -> Option<Tile> {
        let now = Instant::now();
        match self.tiles.get(&key) {
            Some(entry) if entry.expires > now => Some(Tile {
                content_type: entry.content_type.clone(),
                bytes: entry.bytes.clone(),
                remaining: entry.expires - now,
            }),
            Some(_) => {
                self.remove(key);
                None
            }
            None => None,
        }
    }

    fn remove(&mut self, key: Key) {
        if let Some(entry) = self.tiles.remove(&key) {
            self.bytes -= entry.bytes.len();
            self.order.retain(|k| *k != key);
        }
    }

    fn put(&mut self, key: Key, entry: Entry) {
        self.remove(key);
        self.bytes += entry.bytes.len();
        self.tiles.insert(key, entry);
        self.order.push_back(key);
        while self.tiles.len() > CACHE_TILES || self.bytes > CACHE_BYTES {
            match self.order.pop_front() {
                Some(oldest) => {
                    if let Some(gone) = self.tiles.remove(&oldest) {
                        self.bytes -= gone.bytes.len();
                    }
                }
                None => break,
            }
        }
    }
}

/// Fetches and keeps OpenStreetMap tiles.
pub(crate) struct TileProxy {
    base: String,
    agent: ureq::Agent,
    permits: Semaphore,
    store: Mutex<Store>,
}

impl TileProxy {
    pub(crate) fn new() -> Self {
        Self::with_base(UPSTREAM)
    }

    /// Another server instead of OpenStreetMap (tests).
    pub(crate) fn with_base(base: &str) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            // A redirect could lead to another host.
            .max_redirects(0)
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            base: base.trim_end_matches('/').to_string(),
            agent,
            permits: Semaphore::new(CONCURRENT_REQUESTS),
            store: Mutex::new(Store::default()),
        }
    }

    /// The tile `z/x/y`; `y` still has its `.png`.
    pub(crate) async fn get(&self, z: &str, x: &str, y: &str) -> Result<Tile, TileError> {
        let key = parse(z, x, y).map_err(TileError::BadRequest)?;
        if let Some(tile) = self.lock().get(key) {
            return Ok(tile);
        }
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|e| TileError::Upstream(e.to_string()))?;
        // Another request may have fetched it while this one waited.
        if let Some(tile) = self.lock().get(key) {
            return Ok(tile);
        }
        let url = format!("{}/{}/{}/{}.png", self.base, key.0, key.1, key.2);
        let agent = self.agent.clone();
        let fetched = tokio::task::spawn_blocking(move || fetch(&agent, &url))
            .await
            .map_err(|e| TileError::Upstream(format!("internal error: {e}")))??;
        let tile = Tile {
            content_type: fetched.content_type.clone(),
            bytes: fetched.bytes.clone(),
            remaining: fetched.lifetime,
        };
        self.lock().put(
            key,
            Entry {
                content_type: fetched.content_type,
                bytes: fetched.bytes,
                expires: Instant::now() + fetched.lifetime,
            },
        );
        Ok(tile)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(|e| e.into_inner())
    }
}

struct Fetched {
    content_type: String,
    bytes: Vec<u8>,
    lifetime: Duration,
}

fn fetch(agent: &ureq::Agent, url: &str) -> Result<Fetched, TileError> {
    let upstream = TileError::Upstream;
    let mut response = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| upstream(format!("cannot reach the OpenStreetMap tile server: {e}")))?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(upstream(format!(
            "the OpenStreetMap tile server answered HTTP {status} (see https://wiki.openstreetmap.org/wiki/Blocked_tiles)"
        )));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if content_type != "image/png" {
        return Err(upstream(format!(
            "the OpenStreetMap tile server sent {content_type:?} instead of an image"
        )));
    }
    let lifetime = response
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .and_then(max_age)
        .unwrap_or(DEFAULT_LIFETIME);
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_TILE_BYTES)
        .read_to_vec()
        .map_err(|e| upstream(format!("cannot read the tile: {e}")))?;
    Ok(Fetched {
        content_type,
        bytes,
        lifetime,
    })
}

/// The `max-age` of a `Cache-Control` value.
fn max_age(value: &str) -> Option<Duration> {
    value.split(',').find_map(|directive| {
        let (name, seconds) = directive.trim().split_once('=')?;
        if !name.trim().eq_ignore_ascii_case("max-age") {
            return None;
        }
        seconds
            .trim()
            .trim_matches('"')
            .parse::<u64>()
            .ok()
            .map(Duration::from_secs)
    })
}

/// A decimal number of digits only (no sign, no space, no path characters).
fn number(text: &str, what: &str) -> Result<u32, String> {
    if text.is_empty() || text.len() > 9 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{what} must be a number"));
    }
    text.parse().map_err(|_| format!("{what} must be a number"))
}

/// Zoom, column and row of a tile address, checked: zoom 0 to 19, column and row below 2^zoom.
fn parse(z: &str, x: &str, y: &str) -> Result<Key, String> {
    let z = number(z, "zoom")?;
    if z > u32::from(MAX_ZOOM) {
        return Err(format!("zoom must be 0 to {MAX_ZOOM}"));
    }
    let y = y.strip_suffix(".png").ok_or("tile name must end in .png")?;
    let (x, y) = (number(x, "column")?, number(y, "row")?);
    if x >= 1 << z || y >= 1 << z {
        return Err(format!(
            "column and row must be below {} at zoom {z}",
            1u32 << z
        ));
    }
    Ok((u8::try_from(z).map_err(|e| e.to_string())?, x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_tile_addresses() {
        assert_eq!(parse("0", "0", "0.png"), Ok((0, 0, 0)));
        assert_eq!(parse("5", "17", "10.png"), Ok((5, 17, 10)));
        assert_eq!(
            parse("19", "524287", "524287.png"),
            Ok((19, 524_287, 524_287))
        );
    }

    #[test]
    fn refuses_everything_else() {
        for (z, x, y) in [
            ("20", "0", "0.png"),
            ("5", "32", "0.png"),
            ("5", "0", "32.png"),
            ("0", "1", "0.png"),
            ("-1", "0", "0.png"),
            ("+5", "0", "0.png"),
            ("5", "1.5", "0.png"),
            ("5", "..", "0.png"),
            ("5", "0", "0"),
            ("5", "0", "0.jpg"),
            ("5", "0", ".png"),
            ("5", "0%2f", "0.png"),
            ("", "0", "0.png"),
            ("5", "99999999999999999999", "0.png"),
            ("5", "0", "0/../x.png"),
        ] {
            assert!(parse(z, x, y).is_err(), "{z}/{x}/{y}");
        }
    }

    #[test]
    fn reads_max_age() {
        assert_eq!(
            max_age("public, max-age=28800"),
            Some(Duration::from_secs(28800))
        );
        assert_eq!(max_age("no-store"), None);
    }

    #[test]
    fn user_agent_names_the_program_and_nothing_personal() {
        assert!(USER_AGENT.starts_with("freemapper/0."), "{USER_AGENT}");
        assert!(USER_AGENT.contains("https://github.com/HenryJobst/freemapper"));
        assert!(!USER_AGENT.contains('@'));
    }

    #[test]
    fn cache_stays_bounded() {
        let mut store = Store::default();
        let entry = |n: usize| Entry {
            content_type: "image/png".into(),
            bytes: vec![0; n],
            expires: Instant::now() + Duration::from_secs(60),
        };
        for i in 0..(CACHE_TILES as u32 + 10) {
            store.put((10, i, 0), entry(10));
        }
        assert_eq!(store.tiles.len(), CACHE_TILES);
        assert!(store.get((10, 0, 0)).is_none());
        let big = CACHE_BYTES / 2 + 1;
        store.put((11, 0, 0), entry(big));
        store.put((11, 1, 0), entry(big));
        assert!(store.bytes <= CACHE_BYTES);
        assert!(store.get((11, 1, 0)).is_some());
    }
}
