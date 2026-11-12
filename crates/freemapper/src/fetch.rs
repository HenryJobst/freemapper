//! Tile download with a shared on-disk cache keyed by tile and publication date.

use crate::note::{Note, note};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;

use crate::space::{DiskSpace, SystemDiskSpace};

// The fetcher, its errors and the tile types live in `freemapper-adapter-api` (ADR 0082); the
// old paths stay.
pub use freemapper_adapter_api::error::FetchError;
pub(crate) use freemapper_adapter_api::error::tile_names;
#[cfg(test)]
pub(crate) use freemapper_adapter_api::fetch::download_with_retries;
pub(crate) use freemapper_adapter_api::fetch::status_answer_error;
pub use freemapper_adapter_api::fetch::{ByteRange, FetchMode, FetchedTile, Fetcher};
pub(crate) use freemapper_adapter_api::fetch::{
    get_limited_with_retries, get_with_attempts, get_with_retries, one_line, write_cache_file,
    write_inside,
};
#[cfg(test)]
pub(crate) use freemapper_adapter_api::fetch::{
    get_range_with_retries, is_ogc_report, ogc_exception,
};
use freemapper_adapter_api::fetch::{megabytes, too_large};
use freemapper_adapter_api::trust::TrustAnchors;

/// Largest download accepted; LGB DOP20 tiles are about 8 MB.
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

pub(crate) use freemapper_adapter_api::cache::STALE_PART_AGE;

/// A [`Fetcher`] that refuses every request once `cancelled` says so, with
/// [`FetchError::Cancelled`]. The fetch code asks it before each request, so a cancel takes
/// effect at the next request and no download is cut in the middle of the file it writes.
pub(crate) struct CancelCheck<'a> {
    pub(crate) inner: &'a dyn Fetcher,
    pub(crate) cancelled: &'a dyn Fn() -> bool,
}

impl CancelCheck<'_> {
    fn check(&self) -> Result<(), FetchError> {
        if (self.cancelled)() {
            Err(FetchError::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl Fetcher for CancelCheck<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.check()?;
        self.inner.get(url)
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        self.check()?;
        self.inner.last_modified(url)
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        self.check()?;
        self.inner.get_range(url, range)
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        self.check()?;
        self.inner.download_to(url, path, max_bytes, progress)
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
    progress: &mut dyn FnMut(Note),
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
    progress: &mut dyn FnMut(Note),
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
            progress(match total {
                Some(total) => note!(
                    "progress-downloaded-of",
                    megabytes = megabytes(done).to_string(),
                    total = megabytes(total).to_string()
                ),
                None => note!(
                    "progress-downloaded",
                    megabytes = megabytes(done).to_string()
                ),
            });
        }
    }
    file.flush()
        .map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    progress(note!(
        "progress-downloaded",
        megabytes = megabytes(done).to_string()
    ));
    Ok(())
}

/// A redirect answer as an error. Only the agents of a pinned host return one: they follow
/// none, as a followed redirect would keep the host's pinned trust. `pinned` is that host.
fn refuse_redirect(
    url: &str,
    response: &ureq::http::Response<ureq::Body>,
    pinned: Option<&str>,
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
            "{} answered with a redirect (HTTP {}) to {location}; freemapper does not \
             follow redirects of this host, because it pins the host's certificates",
            pinned.unwrap_or("the server"),
            status.as_u16()
        ),
    })
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

/// Largest error answer read for its exception report.
const MAX_ERROR_BODY_BYTES: u64 = 64 * 1024;

/// Identifies freemapper to the servers (Nominatim requires it).
pub const USER_AGENT: &str = concat!(
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

/// Time to receive a large file such as the OSM extract (300 MB at 1 Mbit/s: 40 minutes).
const LARGE_BODY_TIMEOUT: Duration = Duration::from_secs(2 * 3600);

/// The pinned hosts: the trust anchors the raster backends of the registry name, each host
/// once (the first backend that names a host decides its anchors).
fn registered_anchors() -> Vec<TrustAnchors> {
    let mut anchors: Vec<TrustAnchors> = Vec::new();
    for adapter in crate::adapters::registry().adapters() {
        if let Some(trust) = adapter.rasters().and_then(|backend| backend.trust())
            && !anchors
                .iter()
                .any(|known| known.host.eq_ignore_ascii_case(trust.host))
        {
            anchors.push(*trust);
        }
    }
    anchors
}

/// Which of `anchors` verifies `url`: the anchors of a host exactly for absolute URLs on that
/// host. `None` is the default roots, for everything else, including URLs that do not parse.
/// Hosts compare without case, and a single trailing dot (the fully qualified form of the same
/// name, as a pasted URL may carry) is the same host.
fn trust_for(url: &str, anchors: &[TrustAnchors]) -> Option<usize> {
    let uri = url.parse::<ureq::http::Uri>().ok()?;
    let host = uri
        .host()
        .map(|host| host.strip_suffix('.').unwrap_or(host));
    match (uri.scheme(), host) {
        (Some(_), Some(host)) => anchors
            .iter()
            .position(|trust| host.eq_ignore_ascii_case(trust.host)),
        _ => None,
    }
}

fn certificates(trust: &TrustAnchors) -> Vec<ureq::tls::Certificate<'static>> {
    trust
        .pems
        .iter()
        .map(|pem| ureq::tls::Certificate::from_pem(pem).expect("bundled certificates are PEM"))
        .collect()
}

/// An agent for ordinary requests and one for large bodies.
struct Agents {
    normal: ureq::Agent,
    large: ureq::Agent,
    /// The host these agents are pinned to: they verify it against its trust anchors only and
    /// follow no redirect. `None`: the default roots, for every other host.
    pinned: Option<&'static str>,
}

/// Downloads over HTTPS; a stalled server fails with a timeout instead of hanging.
pub struct HttpFetcher {
    default: Agents,
    /// The agents of the pinned hosts, each with its trust anchors.
    pinned: Vec<(TrustAnchors, Agents)>,
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
        let agent = |body: Duration, trust: Option<&TrustAnchors>| -> ureq::Agent {
            let config = ureq::Agent::config_builder()
                .timeout_connect(Some(connect))
                .timeout_recv_response(Some(response))
                .timeout_recv_body(Some(body));
            match trust {
                None => config,
                Some(trust) => config.max_redirects(0).tls_config(
                    ureq::tls::TlsConfig::builder()
                        .root_certs(ureq::tls::RootCerts::new_with_certs(&certificates(trust)))
                        .build(),
                ),
            }
            .build()
            .into()
        };
        let agents = |trust: Option<&TrustAnchors>| Agents {
            normal: agent(body, trust),
            large: agent(LARGE_BODY_TIMEOUT.max(body), trust),
            pinned: trust.map(|trust| trust.host),
        };
        Self {
            default: agents(None),
            pinned: registered_anchors()
                .into_iter()
                .map(|trust| (trust, agents(Some(&trust))))
                .collect(),
        }
    }

    /// The agents verifying `url`'s host.
    fn agents(&self, url: &str) -> &Agents {
        let anchors: Vec<TrustAnchors> = self.pinned.iter().map(|(trust, _)| *trust).collect();
        match trust_for(url, &anchors) {
            Some(index) => &self.pinned[index].1,
            None => &self.default,
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
        let agents = self.agents(url);
        let mut response = agents
            .normal
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Range", &header)
            .call()
            .map_err(|e| status_error(url, e))?;
        refuse_redirect(url, &response, agents.pinned)?;
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
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        let http_error = |message: String| FetchError::Http {
            url: url.to_string(),
            message,
        };
        let agents = self.agents(url);
        let mut response = agents
            .large
            .get(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| http_error(e.to_string()))?;
        refuse_redirect(url, &response, agents.pinned)?;
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
            Ok(Box::new(freemapper_fs::create_new_file(path)?))
        };
        let mut reader = response.body_mut().as_reader();
        stream_to_file(url, &mut reader, total, path, &create, max_bytes, progress)
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        let agents = self.agents(url);
        let response = agents
            .normal
            .head(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| status_error(url, e))?;
        refuse_redirect(url, &response, agents.pinned)?;
        Ok(response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|time| time.date_naive()))
    }

    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let http_error = |e: ureq::Error| status_error(url, e);
        let agents = self.agents(url);
        let mut response = agents
            .normal
            .get(url)
            .header("User-Agent", USER_AGENT)
            // An error answer is read for its exception report.
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(http_error)?;
        refuse_redirect(url, &response, agents.pinned)?;
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

pub use freemapper_adapter_api::cache::CacheDir;

/// The cache shared by all projects: its folder ([`CacheDir`], which the cache derefs to) and how
/// much space the volumes freemapper writes to have left (the system's free space, unless
/// replaced).
#[derive(Debug, Clone)]
pub struct Cache {
    dir: CacheDir,
    space: Arc<dyn DiskSpace>,
}

impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            dir: CacheDir::new(root),
            space: Arc::new(SystemDiskSpace),
        }
    }

    /// The cache with `space` as the source of free space (tests simulate a full disk).
    pub fn with_disk_space(self, space: Arc<dyn DiskSpace>) -> Self {
        Self { space, ..self }
    }

    pub fn disk_space(&self) -> &dyn DiskSpace {
        self.space.as_ref()
    }

    /// `<system cache dir>/freemapper`, if the system has a cache directory.
    pub fn default_location() -> Option<Self> {
        dirs::cache_dir().map(|dir| Self::new(dir.join("freemapper")))
    }
}

impl std::ops::Deref for Cache {
    type Target = CacheDir;

    fn deref(&self) -> &CacheDir {
        &self.dir
    }
}

// The tile download of LGB and GeoSN (cache paths, currency lists, nDSM point clouds) lives in
// `freemapper-adapters-de` (ADR 0082).
pub use freemapper_adapters_de::tile_fetch::{
    NdsmTile, TileOutcome, TilePaths, fetch_ndsm_tiles, fetch_tiles, fetch_tiles_each,
};

#[cfg(test)]
pub(crate) mod test_support {
    use crate::sources::lgb::ProductTiles as _;
    use std::io::Write as _;

    use crate::sources::alkis::test_support::{building_feature, land_use_feature, page};
    use crate::sources::alkis::{BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT};
    use crate::sources::lgb::Product;
    use crate::tiles::TileId;
    pub(crate) use freemapper_adapter_api::testing::MapFetcher;
    pub(crate) use freemapper_sources::testing::{
        dtm_tiff, geotiff, red_jpeg, solid_jpeg, wcs_answer,
    };

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
        // The full ALKIS (NAS) has nothing there.
        fetcher.insert_prefix(
            &crate::sources::alkis_nas::LGB_NAS.url,
            crate::sources::alkis_nas::test_support::EMPTY
                .as_bytes()
                .to_vec(),
        );
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
        let saxony = crate::sources::osm::extract_of("saxony", 0);
        fetcher.insert(
            &saxony.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &saxony.url(),
            std::fs::read(OSM_FIXTURE).expect("OSM fixture"),
        );
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
        let extract = crate::sources::osm::extract_of("saxonyanhalt", 0);
        fetcher.insert(
            &extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &extract.url(),
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
            fetcher.insert_prefix(&model.wcs().url, geotiff(&grid, &values));
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
        let extract = crate::sources::osm::extract_of("mecklenburgvorpommern", 0);
        fetcher.insert(
            &extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &extract.url(),
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
        let extract = crate::sources::osm::extract_of("hamburg", 0);
        fetcher.insert(
            &extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &extract.url(),
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
        let extract = crate::sources::osm::extract_of("hessen", 0);
        fetcher.insert(
            &extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &extract.url(),
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
        let extract = crate::sources::osm::extract_of("schleswigholstein", 0);
        fetcher.insert(
            &extract.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &extract.url(),
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
            fetcher.insert_prefix(&model.wcs().url, geotiff(&grid, &values));
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

    /// An OSM way for [`osm_pbf_with`]: id, node ids, tags.
    pub type PbfWay<'a> = (i64, &'a [i64], &'a [(&'a str, &'a str)]);

    /// An OSM relation for [`osm_pbf_with`]: id, way members (way id, role), tags.
    pub type PbfRelation<'a> = (i64, &'a [(i64, &'a str)], &'a [(&'a str, &'a str)]);

    /// A minimal OSM PBF (uncompressed blobs, plain nodes): the header block, then one block
    /// with each node.
    pub fn osm_pbf(nodes: &[PbfNode<'_>]) -> Vec<u8> {
        osm_pbf_with(nodes, &[], &[])
    }

    /// Like [`osm_pbf`], with ways and relations (way members only): one group each of nodes,
    /// ways and relations.
    pub fn osm_pbf_with(
        nodes: &[PbfNode<'_>],
        ways: &[PbfWay<'_>],
        relations: &[PbfRelation<'_>],
    ) -> Vec<u8> {
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
        let mut way_group = Vec::new();
        for &(id, refs, tags) in ways {
            let (mut keys, mut vals) = (Vec::new(), Vec::new());
            for (k, v) in tags {
                varint(&mut keys, index(k));
                varint(&mut vals, index(v));
            }
            let mut deltas = Vec::new();
            let mut last = 0;
            for &r in refs {
                varint(&mut deltas, zigzag(r - last));
                last = r;
            }
            let mut way = Vec::new();
            number(&mut way, 1, id as u64);
            bytes(&mut way, 2, &keys);
            bytes(&mut way, 3, &vals);
            bytes(&mut way, 8, &deltas);
            bytes(&mut way_group, 3, &way);
        }
        let mut relation_group = Vec::new();
        for &(id, members, tags) in relations {
            let (mut keys, mut vals) = (Vec::new(), Vec::new());
            for (k, v) in tags {
                varint(&mut keys, index(k));
                varint(&mut vals, index(v));
            }
            let (mut roles, mut ids, mut types) = (Vec::new(), Vec::new(), Vec::new());
            let mut last = 0;
            for &(way, role) in members {
                varint(&mut roles, index(role));
                varint(&mut ids, zigzag(way - last));
                last = way;
                varint(&mut types, 1); // MemberType::WAY
            }
            let mut relation = Vec::new();
            number(&mut relation, 1, id as u64);
            bytes(&mut relation, 2, &keys);
            bytes(&mut relation, 3, &vals);
            bytes(&mut relation, 8, &roles);
            bytes(&mut relation, 9, &ids);
            bytes(&mut relation, 10, &types);
            bytes(&mut relation_group, 4, &relation);
        }
        let mut table = Vec::new();
        for s in &strings {
            bytes(&mut table, 1, s.as_bytes());
        }
        let mut block = Vec::new();
        bytes(&mut block, 1, &table);
        bytes(&mut block, 2, &group);
        if !way_group.is_empty() {
            bytes(&mut block, 2, &way_group);
        }
        if !relation_group.is_empty() {
            bytes(&mut block, 2, &relation_group);
        }
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
    use crate::sources::lgb::{Product, ProductTiles as _, ndsm_tile_url, parse_currency_csv};
    use crate::tiles::TileId;
    use freemapper_adapters_de::tile_fetch::cached_ndsm;
    use std::cell::Cell;

    /// `status_answer_error` lives in the api crate, which has no HTTP client, and words a
    /// server error as the client does.
    #[test]
    fn a_server_error_is_worded_as_the_http_client_words_it() {
        let ours = status_answer_error("https://example.invalid/x", 503, b"");
        let FetchError::Http { message, .. } = ours else {
            panic!("{ours:?}");
        };
        assert_eq!(message, ureq::Error::StatusCode(503).to_string());
    }

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
            _progress: &mut dyn FnMut(Note),
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

    use crate::sources::lvermgeo_sh::{PORTAL_HOST, PORTAL_TRUST};

    /// The agents of the portal in `fetcher`.
    fn portal(fetcher: &HttpFetcher) -> &Agents {
        let (trust, agents) = &fetcher.pinned[0];
        assert_eq!(*trust, PORTAL_TRUST);
        agents
    }

    /// The portal is the only pinned host of the built-in adapters.
    #[test]
    fn the_registry_pins_the_portal_only() {
        assert_eq!(registered_anchors(), [PORTAL_TRUST]);
        let fetcher = HttpFetcher::new();
        assert_eq!(fetcher.pinned.len(), 1);
        assert_eq!(portal(&fetcher).pinned, Some(PORTAL_HOST));
        assert_eq!(fetcher.default.pinned, None);
    }

    #[test]
    fn portal_host_uses_the_bundled_anchors() {
        let anchors: Vec<_> = certificates(&PORTAL_TRUST)
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
            assert_eq!(trust_for(url, &[PORTAL_TRUST]), Some(0), "{url}");
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
            assert_eq!(trust_for(url, &[PORTAL_TRUST]), None, "{url}");
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
        for agent in [&portal(&fetcher).normal, &portal(&fetcher).large] {
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
                normal: portal(&fetcher).normal.clone(),
                large: portal(&fetcher).large.clone(),
                pinned: Some(PORTAL_HOST),
            },
            pinned: Vec::new(),
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
        let result = portal(&fetcher)
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
