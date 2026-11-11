//! What a fetch of a tiled height model (HVBG Hesse, LVermGeo SH) found besides the raster, and
//! the mosaic of 1 km tiles named by a tile list (Geobasis NRW, LVermGeo RP).

use crate::note::{Note, note};
use std::path::{Path, PathBuf};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode, Fetcher, get_with_retries, write_cache_file};
use crate::float_tiff::{decode_float_tiff, write_float_raster};
use crate::raster::{GridSpec, Raster};
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
    pub warnings: Vec<Note>,
}

/// How a provider's 1 km zone-32 tiles (named by its tile list) are downloaded, cached and
/// checked; see [`mosaic_tiles`].
pub struct TileStore<'a> {
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
pub struct AreaTiles {
    /// Tiles with a (checked) file name.
    pub named: Vec<(TileId, String)>,
    /// Tiles missing before any download, each with why (`not in the tile list`).
    pub missing: Vec<(TileId, String)>,
}

/// Why a named tile is missing: the server does not have it.
const NOT_ON_SERVER: &str = "not on the server (404)";

/// `32_400_5579`.
pub fn short_name(tile: TileId) -> String {
    format!("32_{:03}_{:04}", tile.east_km, tile.north_km)
}

/// The cached tile at `path`, if it is there and reads as `tile`.
pub fn read_cached(path: &Path, tile: TileId) -> Option<Raster<f32>> {
    let bytes = std::fs::read(path).ok()?;
    decode_float_tiff(&bytes)
        .and_then(|raster| whole_tile(raster, tile, &[1.0]))
        .ok()
}

/// `raster` if it covers exactly the 1 km `tile` with pixels of one of the `pixels` sizes.
pub fn whole_tile(
    raster: Raster<f32>,
    tile: TileId,
    pixels: &[f64],
) -> Result<Raster<f32>, String> {
    let g = raster.grid;
    let near = |a: f64, b: f64| (a - b).abs() < 1e-3;
    if !pixels.iter().any(|&p| (g.pixel_size - p).abs() < 1e-9) {
        let wanted: Vec<String> = pixels.iter().map(|p| format!("{p} m")).collect();
        return Err(format!(
            "pixels of {} m, not {}",
            g.pixel_size,
            wanted.join(" or ")
        ));
    }
    let fits = near(g.west, f64::from(tile.east_km) * 1000.0)
        && near(g.north, f64::from(tile.north_km + 1) * 1000.0)
        && near(f64::from(g.width) * g.pixel_size, 1000.0)
        && near(f64::from(g.height) * g.pixel_size, 1000.0);
    if fits {
        Ok(raster)
    } else {
        Err(format!(
            "the raster (W {} N {}, {} x {} px of {} m) is not the whole tile {}",
            g.west,
            g.north,
            g.width,
            g.height,
            g.pixel_size,
            short_name(tile)
        ))
    }
}

/// Pastes the 1 m `part` into the 1 m `raster`; returns how many pasted cells have no height.
pub fn paste_tile(raster: &mut Raster<f32>, part: &Raster<f32>) -> Result<u64, String> {
    let empty = std::cell::Cell::new(0);
    raster
        .paste_with(&part.grid, |x, y| {
            let z = part.get(x, y);
            if z.is_nan() {
                empty.set(empty.get() + 1);
            }
            z
        })
        .map_err(|e| e.to_string())?;
    Ok(empty.get())
}

/// The tiles of `area` mosaicked into a Float32 GeoTIFF `out` (with its world file) on the
/// 1 m grid over `bbox` (EPSG:25832); cells no tile covers are NaN.
///
/// Cached tiles are read, the others downloaded and cached (offline, one not cached is an
/// error). A tile the server answers 404 for gets a note and is missing, as is one with a note
/// that still holds. Returns the missing tiles, sorted; no tile at all is an error.
#[allow(clippy::too_many_arguments)]
pub fn mosaic_tiles(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    bbox: &BoundingBox,
    store: &TileStore,
    area: AreaTiles,
    root: &Path,
    out: &Path,
    warnings: &mut Vec<Note>,
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
            warnings.push(note!(
                "warning-model-tile-cells-empty",
                model = crate::note::model(store.label),
                tile = short_name(tile),
                count = empty
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
                write_cache_file(&(store.cache_path)(&name), &bytes)?;
                if marker.exists() {
                    let _ = std::fs::remove_file(&marker);
                }
                paste(&mut raster, tile, &name, &part)?;
            }
            // Named by the list, not on the server. The note keeps later runs from asking
            // again (for as long as `known_missing` says) and offline runs from failing.
            Err(FetchError::NotFound(_)) => {
                write_cache_file(&marker, b"")?;
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
    write_float_raster(&raster, root, out, store.source)?;
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
pub fn centre_tile(bbox: &BoundingBox) -> TileId {
    TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    }
}

/// The date of the centre tile of `bbox`. `listed` is what the tile list says of it: `None`
/// when it does not name it, else its date or the warning to give when it has none. A centre
/// tile among `missing`, unlisted or undated gives no date and a warning.
pub fn centre_date(
    bbox: &BoundingBox,
    label: &str,
    listed: Option<Result<String, Note>>,
    missing: &[TileId],
    warnings: &mut Vec<Note>,
) -> Option<String> {
    let centre = centre_tile(bbox);
    match listed {
        Some(_) if missing.contains(&centre) => {
            warnings.push(note!(
                "warning-centre-tile-missing",
                model = crate::note::model(label),
                tile = short_name(centre)
            ));
            None
        }
        Some(Err(warning)) => {
            warnings.push(warning);
            None
        }
        Some(Ok(date)) => Some(date),
        None => {
            warnings.push(note!(
                "warning-centre-tile-unlisted",
                model = crate::note::model(label),
                tile = short_name(centre)
            ));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, geotiff};

    /// Two tiles side by side (centre at the tile border 400 km).
    const BBOX: BoundingBox = BoundingBox {
        min_e: 399_900.0,
        min_n: 5_579_400.0,
        max_e: 400_100.0,
        max_n: 5_579_600.0,
    };
    const WEST: TileId = TileId {
        east_km: 399,
        north_km: 5579,
    };
    const EAST: TileId = TileId {
        east_km: 400,
        north_km: 5579,
    };

    fn url(name: &str) -> String {
        format!("https://tiles.example/{name}.tif")
    }

    /// A whole 1 km tile of height `z`.
    fn tile_file(tile: TileId, z: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        geotiff(&grid, &vec![z; 1_000_000])
    }

    /// Mosaics `named` (and the `missing` given) with a store in `dir`; the error text, or
    /// the missing tiles and the mosaic's values.
    fn mosaic(
        fetcher: &MapFetcher,
        dir: &Path,
        named: &[(TileId, &str)],
        missing: &[(TileId, &str)],
    ) -> Result<(Vec<TileId>, Vec<f32>), String> {
        let store = TileStore {
            label: "terrain model (test)",
            list_url: "https://tiles.example/list",
            source: "https://tiles.example",
            url: &url,
            cache_path: &|name| dir.join(format!("{name}.tif")),
            marker_path: &|name| dir.join(format!("{name}.missing")),
            known_missing: &|path| path.exists(),
            precheck: &|_| Ok(()),
            explain_empty: true,
        };
        let area = AreaTiles {
            named: named.iter().map(|(t, n)| (*t, n.to_string())).collect(),
            missing: missing.iter().map(|(t, w)| (*t, w.to_string())).collect(),
        };
        let out = dir.join("out.tif");
        let mut warnings = Vec::new();
        let missing = mosaic_tiles(
            fetcher,
            FetchMode::Online,
            &BBOX,
            &store,
            area,
            out.parent().unwrap(),
            &out,
            &mut warnings,
        )
        .map_err(|e| e.to_string())?;
        let bytes = std::fs::read(&out).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
            panic!("not float")
        };
        Ok((missing, values))
    }

    #[test]
    fn two_tiles_paste_into_the_grid() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&url("w"), tile_file(WEST, 10.0));
        fetcher.insert(&url("e"), tile_file(EAST, 20.0));
        let dir = tempfile::tempdir().unwrap();
        let (missing, values) =
            mosaic(&fetcher, dir.path(), &[(WEST, "w"), (EAST, "e")], &[]).unwrap();
        assert!(missing.is_empty(), "{missing:?}");
        assert_eq!(values.len(), 200 * 200);
        assert_eq!(
            (values[100 * 200 + 99], values[100 * 200 + 100]),
            (10.0, 20.0)
        );
        assert!(values.iter().all(|z| z.is_finite()));
        // Cached by name: a second run asks for nothing.
        let silent = MapFetcher::default();
        mosaic(&silent, dir.path(), &[(WEST, "w"), (EAST, "e")], &[]).unwrap();
        assert_eq!(silent.calls.get(), 0);
    }

    #[test]
    fn a_404_tile_lands_in_missing() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&url("w"), tile_file(WEST, 10.0));
        let dir = tempfile::tempdir().unwrap();
        let (missing, values) =
            mosaic(&fetcher, dir.path(), &[(WEST, "w"), (EAST, "e")], &[]).unwrap();
        assert_eq!(missing, vec![EAST]);
        assert!(values[100 * 200 + 100].is_nan());
        assert!(dir.path().join("e.missing").exists());
        // The note holds: not asked again.
        fetcher.log.borrow_mut().clear();
        let (missing, _) = mosaic(&fetcher, dir.path(), &[(WEST, "w"), (EAST, "e")], &[]).unwrap();
        assert_eq!(missing, vec![EAST]);
        assert!(!fetcher.log.borrow().contains(&url("e")));
    }

    #[test]
    fn no_tile_is_an_error_with_the_reasons() {
        let dir = tempfile::tempdir().unwrap();
        let err = mosaic(
            &MapFetcher::default(),
            dir.path(),
            &[(WEST, "w")],
            &[(EAST, "not in the tile list")],
        )
        .unwrap_err();
        assert!(
            err.contains(
                "no tile of the terrain model (test) covers the map area: not in the tile list: \
                 32_400_5579; not on the server (404): 32_399_5579"
            ),
            "{err}"
        );
        let reasons = [
            (EAST, "b".to_string()),
            (WEST, "a".to_string()),
            (
                TileId {
                    east_km: 398,
                    north_km: 5579,
                },
                "b".to_string(),
            ),
        ];
        assert_eq!(
            grouped(&reasons),
            "b: 32_398_5579, 32_400_5579; a: 32_399_5579"
        );
    }

    #[test]
    fn the_centre_tile_gives_the_date() {
        let mut warnings = Vec::new();
        let dated = Some(Ok("2024-04-06".to_string()));
        assert_eq!(centre_tile(&BBOX), EAST);
        assert_eq!(
            centre_date(&BBOX, "x", dated.clone(), &[], &mut warnings).as_deref(),
            Some("2024-04-06")
        );
        assert!(warnings.is_empty());
        assert_eq!(centre_date(&BBOX, "x", dated, &[EAST], &mut warnings), None);
        assert_eq!(
            centre_date(
                &BBOX,
                "x",
                Some(Err(note!("warning-lgln-no-date-unlisted", model = "x"))),
                &[],
                &mut warnings
            ),
            None
        );
        assert_eq!(centre_date(&BBOX, "x", None, &[], &mut warnings), None);
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        assert_eq!(
            warnings[1],
            "x: no date, the STAC lists no tile at the centre of the map area"
        );
    }
}
