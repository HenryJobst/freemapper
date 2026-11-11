//! The fetcher an adapter downloads through, and the tiles a fetch leaves in the cache.

use std::io::Write as _;
use std::path::{Path, PathBuf};

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
