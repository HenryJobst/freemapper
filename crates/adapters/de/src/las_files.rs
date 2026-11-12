//! The cached point-cloud tiles of a provider: one LAS/LAZ file per tile and publication,
//! `<tile>_<published>.laz`, checked to be complete before it is kept (ADR 0105).

use std::io::Read as _;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::env::AdapterEnv;
use sha2::{Digest as _, Sha256};

use crate::fetch::FetchError;
use crate::tiles::TileId;

/// A tile a list says is larger is no tile of a state (the largest of Geobasis NRW has 553 MB,
/// of Rhineland-Palatinate about 1.2 GB).
const MAX_TILE_BYTES: u64 = 2 << 30;

/// The name of a km tile in zone 32 in the project file and the cache: `32356-5644`.
pub(crate) fn utm32_tile_name(tile: TileId) -> String {
    format!("32{:03}-{:04}", tile.east_km, tile.north_km)
}

/// The tile of a [`utm32_tile_name`].
pub(crate) fn parse_utm32_tile_name(name: &str) -> Option<TileId> {
    let (east, north) = name.strip_prefix("32")?.split_once('-')?;
    (east.len() == 3 && north.len() == 4).then_some(())?;
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// `<tile>_<published>.laz` in `dir`; `tile` is the provider's name of the tile (`33421-5821`).
pub(crate) fn las_path(dir: &Path, tile: &str, published: NaiveDate) -> PathBuf {
    dir.join(format!("{tile}_{published}.laz"))
}

/// The cached point clouds of `tile` in `dir`, newest first.
pub(crate) fn cached_las(dir: &Path, tile: &str) -> Vec<(NaiveDate, PathBuf)> {
    let prefix = format!("{tile}_");
    let mut files: Vec<(NaiveDate, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(&prefix)?
                .strip_suffix(".laz")?
                .parse()
                .ok()?;
            Some((date, entry.path()))
        })
        .collect();
    files.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
    files
}

/// Checks that the file at `path` is a complete LAS/LAZ point cloud: its header, the LAZ chunk
/// table at the end and the last point have to be readable.
pub(crate) fn check_las_file(url: &str, path: &Path) -> Result<(), FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a complete LAS point cloud: {message}"),
    };
    let mut reader = las::Reader::from_path(path).map_err(|e| corrupt(e.to_string()))?;
    let total = reader.header().number_of_points();
    if total > 0 {
        reader.seek(total - 1).map_err(|e| corrupt(e.to_string()))?;
        reader.read_points(1).map_err(|e| corrupt(e.to_string()))?;
    }
    Ok(())
}

/// The SHA-256 of the file at `path` in hex, read in pieces.
fn file_sha256(path: &Path) -> Result<String, FetchError> {
    let io = |e| FetchError::Io(path.to_path_buf(), e);
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = file.read(&mut buffer).map_err(io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Streams the tile at `url` to a partial file next to `path`, checks that it has the size
/// `bytes` and the SHA-256 `sha256` (when the list gives one) the list of tiles gives and is a
/// complete point cloud, then moves it to `path`. A broken file is not left behind.
pub(crate) fn download_las(
    env: &mut AdapterEnv<'_>,
    url: &str,
    bytes: u64,
    sha256: Option<&str>,
    path: &Path,
) -> Result<(), FetchError> {
    if bytes > MAX_TILE_BYTES {
        return Err(FetchError::TooLarge {
            url: url.to_string(),
            limit_mb: MAX_TILE_BYTES >> 20,
        });
    }
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message,
    };
    let part = path.with_extension("laz.part");
    let _ = std::fs::remove_file(&part);
    let result = env
        .fetcher
        .download_to(url, &part, bytes, env.progress)
        .and_then(|()| {
            let got = std::fs::metadata(&part)
                .map_err(|e| FetchError::Io(part.clone(), e))?
                .len();
            if got == bytes {
                Ok(())
            } else {
                Err(corrupt(format!("{got} bytes, the index says {bytes}")))
            }
        })
        .and_then(|()| match sha256 {
            Some(expected) => {
                let got = file_sha256(&part)?;
                if got.eq_ignore_ascii_case(expected) {
                    Ok(())
                } else {
                    Err(corrupt(format!("SHA-256 {got}, the index says {expected}")))
                }
            }
            None => Ok(()),
        })
        .and_then(|()| check_las_file(url, &part))
        .and_then(|()| {
            std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}
