//! Height tiles in a ZIP archive of a file server, read member by member with range requests:
//! the directory of an archive kept in the cache, and a tile of XYZ text kept as a GeoTIFF.

use std::path::Path;

use crate::fetch::{FetchError, FetchMode, Fetcher, write_cache_file};
use crate::raster::Raster;
use crate::sources::float_tiff::encode_float_geotiff;
use crate::sources::height_model::read_cached;
use crate::sources::zip_range::{
    Member, parse_directory, read_directory_bytes, read_member, refused,
};
use crate::tiles::TileId;

/// The members of the archive at `url`: online from the archive, and kept in the cache as
/// `file`; offline from `file`, and none when the directory was never read. An archive that is
/// not there is [`FetchError::NotFound`].
pub(crate) fn kept_directory(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    url: &str,
    file: &Path,
) -> Result<Vec<Member>, FetchError> {
    match mode {
        FetchMode::Online => {
            let (directory, count) = read_directory_bytes(fetcher, url)?;
            let members =
                parse_directory(&directory, count).map_err(|message| refused(url, message))?;
            // The member count (u16, little-endian), then the central directory. Kept only
            // once it parsed, so a broken answer never replaces a good copy.
            write_cache_file(file, &[&count.to_le_bytes()[..], &directory].concat())?;
            Ok(members)
        }
        FetchMode::Offline => {
            let kept = || -> Option<Vec<Member>> {
                let cached = std::fs::read(file).ok()?;
                let (count, directory) = cached.split_at_checked(2)?;
                parse_directory(directory, u16::from_le_bytes([count[0], count[1]])).ok()
            };
            Ok(kept().unwrap_or_default())
        }
    }
}

/// The heights of `tile` from the XYZ text `member` of the archive at `url`, read by `parse`
/// and kept in the cache as the GeoTIFF `file`: the text of a tile has 20 to 30 MB. Offline, a
/// tile that is not cached is an error that says to fetch online.
pub(crate) fn xyz_tile(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    (url, member): (&str, &Member),
    file: &Path,
    tile: TileId,
    parse: impl Fn(&[u8], TileId) -> Result<Raster<f32>, String>,
) -> Result<Raster<f32>, FetchError> {
    if let Some(part) = read_cached(file, tile) {
        return Ok(part);
    }
    if mode == FetchMode::Offline {
        return Err(not_cached(url, member));
    }
    let wrong = |message: String| refused(url, format!("{}: {message}", member.name));
    let part = parse(&read_member(fetcher, url, member)?, tile).map_err(wrong)?;
    write_cache_file(file, &encode_float_geotiff(&part).map_err(wrong)?)?;
    Ok(part)
}

/// The error of a tile that is asked for offline and is not in the cache.
pub(crate) fn not_cached(url: &str, member: &Member) -> FetchError {
    refused(
        url,
        format!(
            "offline and not cached: {}; run `freemapper fetch` online",
            member.name
        ),
    )
}
