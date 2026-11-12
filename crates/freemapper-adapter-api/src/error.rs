//! The errors of a fetch, and the errors of the readers they carry.

use std::io;
use std::path::{Path, PathBuf};

use freemapper_georef::tiles::TileId;

use crate::records::Product;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("download of {url} failed: {message}")]
    Http { url: String, message: String },
    #[error("{}", io_error_text(.0, .1))]
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
    /// The caller asked to stop; no request was made.
    #[error("cancelled")]
    Cancelled,
    /// freemapper does not send this request: the URL, the address its host resolves to or a
    /// redirect is against the rules for where requests go. Asking again does not help.
    /// `url` has no query and no fragment.
    #[error("{url}: freemapper does not send requests to this address: {reason}")]
    NotAllowed { url: String, reason: String },
    /// freemapper does not send this request: the adapter it was made for does not name the
    /// host. The message says what the person can do. Asking again does not help.
    #[error("{0}")]
    HostNotNamed(Box<HostNotNamed>),
}

/// A request that was not sent because its host is not one the adapter names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostNotNamed {
    /// The address that was not asked, without query and fragment.
    pub target: String,
    /// Its host.
    pub host: String,
    /// The address that redirected to `target`, without query and fragment.
    pub from: Option<String>,
    /// The id of the adapter the request was made for.
    pub adapter: String,
    /// The English name of its region.
    pub name: String,
    pub remedy: Remedy,
}

/// What a person can do when an adapter does not name a host its service leads to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Remedy {
    /// The adapter is part of the program: the person allows the host for it
    /// (`freemapper adapters allow-host`) and reports the change.
    AllowHost,
    /// The adapter is a folder of the person: the host goes into its manifest.
    Manifest,
}

/// Where a change of a built-in adapter's services is reported.
pub const ISSUES_URL: &str = "https://github.com/HenryJobst/freemapper/issues";

impl std::fmt::Display for HostNotNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            target,
            host,
            from,
            adapter,
            name,
            remedy,
        } = self;
        write!(
            f,
            "{target}: freemapper does not send requests to this address: "
        )?;
        match remedy {
            Remedy::AllowHost => write!(f, "the adapter of {name} does not name the host {host}")?,
            Remedy::Manifest => write!(f, "the adapter {adapter} does not name the host {host}")?,
        }
        if let Some(from) = from {
            write!(f, " ({from} redirects there)")?;
        }
        match remedy {
            Remedy::AllowHost => write!(
                f,
                ". If the service has moved there, allow the host with `freemapper adapters \
                 allow-host {adapter} {host}` and report it at {ISSUES_URL}"
            ),
            Remedy::Manifest => write!(
                f,
                ". If the service has moved there, add the host to `[net] hosts` of the \
                 adapter's manifest"
            ),
        }
    }
}

/// What a raster fetch can end with. Each variant stands for one error of the engine, which
/// turns it back into its own, so the caller words it as before.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    /// A file of the project could not be written or read.
    #[error("{}", io_error_text(.0, .1))]
    Io(PathBuf, io::Error),
    /// The project file could not be written or read as what it must be.
    #[error("invalid project file: {0}")]
    ProjectFile(String),
    #[error("no orthophoto stored for this map area; run `freemapper fetch` online first")]
    OrthophotoNotFetched,
    #[error("no terrain model stored for this map area; run `freemapper fetch` online first")]
    TerrainNotFetched,
    /// An error of the engine that passes through the fetch as it is (the checkpoint failed in
    /// a way the variants above do not name). The engine takes its own error out again.
    #[error("{0}")]
    Engine(Box<dyn std::error::Error + Send + Sync>),
}

impl FetchError {
    /// Whether writing failed because the volume is full.
    pub fn is_disk_full(&self) -> bool {
        matches!(self, FetchError::Io(_, e) if is_storage_full(e))
    }
}

/// The names of `tiles`, separated by commas.
pub fn tile_names(tiles: &[TileId]) -> String {
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

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("currency CSV line {line}: {message}")]
pub struct CurrencyCsvError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid ALKIS data: {message}")]
pub struct AlkisError {
    pub message: String,
}

/// Whether `e` says the volume is full or the user's quota is used up. The standard library maps
/// the systems' codes to these kinds: ENOSPC and EDQUOT on Linux and macOS, ERROR_DISK_FULL,
/// ERROR_HANDLE_DISK_FULL and ERROR_DISK_QUOTA_EXCEEDED on Windows.
pub fn is_storage_full(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded
    )
}

/// The text of an error writing or reading `path`: plain words when the disk is full, else the
/// path and the system's message.
pub fn io_error_text(path: &Path, e: &io::Error) -> String {
    if is_storage_full(e) {
        format!(
            "the disk is full, so freemapper could not write {}. Free some space and try again",
            path.display()
        )
    } else {
        format!("{}: {e}", path.display())
    }
}
