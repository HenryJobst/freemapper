//! What a fetch of a tiled height model (HVBG Hesse, LVermGeo SH) found besides the raster, and
//! the mosaic of 1 km tiles named by a tile list (Geobasis NRW, LVermGeo RP).

use std::path::{Path, PathBuf};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode, Fetcher, get_with_retries, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::lvermgeo_sh::{paste_tile, read_cached, whole_tile};
use crate::tiles::TileId;

/// What a tiled height model fetch found besides the raster.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeightModel {
    /// Tiles inside the provider's state that it does not have.
    pub missing: Vec<TileId>,
    /// The acquisition (flight) date (`YYYY-MM-DD`) of the tile at the centre of the area.
    pub date: Option<String>,
    /// Hesse: places of the metadata list without a district or file in the Downloadcenter
    /// (such as a gemeindefrei area), as `<municipality> (<district>)`; their tiles came from
    /// other zips or are missing.
    pub unresolved: Vec<String>,
    /// Hesse: tiles inside the map area that stayed empty and that an unresolved place is listed
    /// for.
    pub unresolved_empty: Vec<TileId>,
    /// Problems that did not stop the fetch (a date that cannot be read, a list that fell back
    /// to a kept older one, cells without height).
    pub warnings: Vec<String>,
}

/// How a provider's 1 km zone-32 tiles (named by its tile list) are downloaded, cached and
/// checked; see [`mosaic_tiles`].
pub(crate) struct TileStore<'a> {
    /// `terrain model (…)`, in messages.
    pub label: &'a str,
    /// The URL an error names when no tile covers the map area (the tile list).
    pub list_url: &'a str,
    /// The service an error names when the mosaic cannot be written.
    pub source: &'a str,
    /// The download URL of a tile name.
    pub url: &'a dyn Fn(&str) -> String,
    /// Where the tile of a name is cached (a new scan year is a new name, so a new file).
    pub cache_path: &'a dyn Fn(&str) -> PathBuf,
    /// Where the note is kept that the server answered 404 for a tile name.
    pub marker_path: &'a dyn Fn(&str) -> PathBuf,
    /// Whether the 404 note at a path still holds.
    pub known_missing: &'a dyn Fn(&Path) -> bool,
    /// Refuses a download before it is decoded (an HTML page), with the reason.
    pub precheck: &'a dyn Fn(&[u8]) -> Result<(), String>,
    /// Whether the error for an area without any tile says why each tile is missing.
    pub explain_empty: bool,
}

/// The tiles of the map area as the provider's tile list gives them.
#[derive(Debug, Default)]
pub(crate) struct AreaTiles {
    /// Tiles with a (checked) file name.
    pub named: Vec<(TileId, String)>,
    /// Tiles missing before any download, each with why (`not in the tile list`).
    pub missing: Vec<(TileId, String)>,
}

/// Why a named tile is missing: the server does not have it.
const NOT_ON_SERVER: &str = "not on the server (404)";

/// `32_400_5579`.
pub(crate) fn short_name(tile: TileId) -> String {
    format!("32_{:03}_{:04}", tile.east_km, tile.north_km)
}

/// The tiles of `area` mosaicked into a Float32 GeoTIFF `out` (with its world file) on the
/// 1 m grid over `bbox` (EPSG:25832); cells no tile covers are NaN.
///
/// Cached tiles are read, the others downloaded and cached (offline, one not cached is an
/// error). A tile the server answers 404 for gets a note and is missing, as is one with a note
/// that still holds. Returns the missing tiles, sorted; no tile at all is an error.
pub(crate) fn mosaic_tiles(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    bbox: &BoundingBox,
    store: &TileStore,
    area: AreaTiles,
    out: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<TileId>, FetchError> {
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let mut reasons = area.missing;
    let mut found = 0;
    let mut pending = Vec::new();
    let mut paste = |raster: &mut Raster<f32>, tile: TileId, name: &str, part: &Raster<f32>| {
        found += 1;
        let empty = paste_tile(raster, part).map_err(|message| FetchError::Http {
            url: (store.url)(name),
            message: format!("{} tile {name}: {message}", store.label),
        })?;
        if empty > 0 {
            warnings.push(format!(
                "{} tile {}: {empty} cells of the map area have no height",
                store.label,
                short_name(tile)
            ));
        }
        Ok::<(), FetchError>(())
    };
    for (tile, name) in area.named {
        match read_cached(&(store.cache_path)(&name), tile) {
            Some(part) => paste(&mut raster, tile, &name, &part)?,
            None if (store.known_missing)(&(store.marker_path)(&name)) => {
                reasons.push((tile, NOT_ON_SERVER.to_string()));
            }
            None => pending.push((tile, name)),
        }
    }
    if let (FetchMode::Offline, Some((_, name))) = (mode, pending.first()) {
        return Err(FetchError::Http {
            url: (store.url)(name),
            message: format!(
                "offline and not cached: {} tile {name}; run `freemapper fetch` online",
                store.label
            ),
        });
    }
    for (tile, name) in pending {
        let url = (store.url)(&name);
        let downloaded = get_with_retries(fetcher, &url, |bytes| {
            let part = (store.precheck)(bytes)
                .and_then(|()| decode_float_tiff(bytes))
                .and_then(|r| whole_tile(r, tile, &[1.0]))
                .map_err(|message| FetchError::Http {
                    url: url.clone(),
                    message: format!("{} tile {name}: {message}", store.label),
                })?;
            Ok((part, bytes.to_vec()))
        });
        let marker = (store.marker_path)(&name);
        match downloaded {
            Ok((part, bytes)) => {
                write_atomically(&(store.cache_path)(&name), &bytes)?;
                if marker.exists() {
                    let _ = std::fs::remove_file(&marker);
                }
                paste(&mut raster, tile, &name, &part)?;
            }
            // Named by the list, not on the server. The note keeps later runs from asking
            // again (for as long as `known_missing` says) and offline runs from failing.
            Err(FetchError::NotFound(_)) => {
                write_atomically(&marker, b"")?;
                reasons.push((tile, NOT_ON_SERVER.to_string()));
            }
            Err(e) => return Err(e),
        }
    }
    if found == 0 {
        let why = if store.explain_empty && !reasons.is_empty() {
            format!(": {}", grouped(&reasons))
        } else {
            String::new()
        };
        return Err(FetchError::Http {
            url: store.list_url.to_string(),
            message: format!("no tile of the {} covers the map area{why}", store.label),
        });
    }
    write_float_raster(&raster, out, store.source)?;
    let mut missing: Vec<TileId> = reasons.into_iter().map(|(tile, _)| tile).collect();
    missing.sort();
    Ok(missing)
}

/// `why one: 32_333_5537, 32_334_5537; why two: 32_400_5579`, the reasons in first-seen order.
fn grouped(reasons: &[(TileId, String)]) -> String {
    let mut groups: Vec<(&str, Vec<TileId>)> = Vec::new();
    for (tile, why) in reasons {
        match groups.iter_mut().find(|(w, _)| w == why) {
            Some((_, tiles)) => tiles.push(*tile),
            None => groups.push((why, vec![*tile])),
        }
    }
    groups
        .into_iter()
        .map(|(why, mut tiles)| {
            tiles.sort();
            let names: Vec<String> = tiles.into_iter().map(short_name).collect();
            format!("{why}: {}", names.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The 1 km tile at the centre of `bbox` (EPSG:25832).
pub(crate) fn centre_tile(bbox: &BoundingBox) -> TileId {
    TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    }
}

/// The date of the centre tile of `bbox`. `listed` is what the tile list says of it: `None`
/// when it does not name it, else its date or the warning to give when it has none. A centre
/// tile among `missing`, unlisted or undated gives no date and a warning.
pub(crate) fn centre_date(
    bbox: &BoundingBox,
    label: &str,
    listed: Option<Result<String, String>>,
    missing: &[TileId],
    warnings: &mut Vec<String>,
) -> Option<String> {
    let centre = centre_tile(bbox);
    match listed {
        Some(_) if missing.contains(&centre) => {
            warnings.push(format!(
                "the {label} tile {} at the centre of the map area is missing, so there is no date",
                short_name(centre)
            ));
            None
        }
        Some(Err(warning)) => {
            warnings.push(warning);
            None
        }
        Some(Ok(date)) => Some(date),
        None => {
            warnings.push(format!(
                "the tile list names no {label} tile {} at the centre of the map area, so there is \
                 no date",
                short_name(centre)
            ));
            None
        }
    }
}
