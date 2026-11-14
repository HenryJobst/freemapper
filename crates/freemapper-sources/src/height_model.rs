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

/// A downloaded tile made ready: the 1 m tile that is pasted, and the bytes that are cached.
pub type Prepared = (Raster<f32>, Vec<u8>);
/// What makes a downloaded tile ready, or says why it cannot be used.
pub type Prepare<'a> = dyn Fn(&[u8], TileId) -> Result<Prepared, String> + 'a;

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
    /// What a downloaded tile becomes: the 1 m tile that is pasted, and the bytes that are
    /// cached (a cached file must read back as that 1 m tile). [`as_delivered`] for a
    /// provider whose tiles are 1 m GeoTIFFs.
    pub prepare: &'a Prepare<'a>,
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

/// The names of `tiles` as the providers write them (`32_565_5937`), joined by commas.
pub fn names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(|t| format!("32_{:03}_{:04}", t.east_km, t.north_km))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The cached tile at `path`, if it is there and reads as `tile`.
pub fn read_cached(path: &Path, tile: TileId) -> Option<Raster<f32>> {
    let bytes = std::fs::read(path).ok()?;
    decode_float_tiff(&bytes)
        .and_then(|raster| whole_tile(raster, tile, &[1.0]))
        .ok()
}

/// The tile as it was delivered: a 1 m GeoTIFF of exactly the tile, cached unchanged.
pub fn as_delivered(bytes: &[u8], tile: TileId) -> Result<Prepared, String> {
    let part = decode_float_tiff(bytes).and_then(|raster| whole_tile(raster, tile, &[1.0]))?;
    Ok((part, bytes.to_vec()))
}

/// `raster` with cells `factor` times as wide; each takes the highest of its values, cells
/// without a value (NaN) are passed over, a cell with none stays NaN.
pub fn coarsen_max(raster: &Raster<f32>, factor: u32) -> Result<Raster<f32>, String> {
    let fine = raster.grid;
    if factor == 0 || !fine.width.is_multiple_of(factor) || !fine.height.is_multiple_of(factor) {
        return Err(format!(
            "{} x {} px cannot be reduced by {factor}",
            fine.width, fine.height
        ));
    }
    let grid = GridSpec {
        pixel_size: fine.pixel_size * f64::from(factor),
        width: fine.width / factor,
        height: fine.height / factor,
        ..fine
    };
    let mut coarse = Raster::filled(grid, f32::NAN);
    let (width, step) = (fine.width as usize, factor as usize);
    for (row, cells) in raster.data.chunks_exact(width).enumerate() {
        let start = (row / step) * grid.width as usize;
        let target = &mut coarse.data[start..start + grid.width as usize];
        for (block, highest) in cells.chunks_exact(step).zip(target) {
            for z in block {
                // `f32::max` passes over a NaN on either side.
                *highest = highest.max(*z);
            }
        }
    }
    Ok(coarse)
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
            (store.precheck)(bytes)
                .and_then(|()| (store.prepare)(bytes, tile))
                .map_err(|message| FetchError::Http {
                    url: url.clone(),
                    message: format!("{} tile {name}: {message}", store.label),
                })
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
            prepare: &as_delivered,
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

    /// A raster of `width` × `height` cells of `pixel` m at the origin, row by row.
    fn raster(width: u32, height: u32, pixel: f64, data: Vec<f32>) -> Raster<f32> {
        Raster {
            grid: GridSpec {
                west: 1000.0,
                north: 2000.0,
                pixel_size: pixel,
                width,
                height,
            },
            data,
        }
    }

    #[test]
    fn coarsen_max_takes_the_highest_of_each_block() {
        let fine = raster(4, 4, 0.5, (1..=16).map(|z| z as f32).collect());
        let coarse = coarsen_max(&fine, 2).unwrap();
        assert_eq!(coarse.data, [6.0, 8.0, 14.0, 16.0]);
        assert_eq!(
            coarse.grid,
            GridSpec {
                west: 1000.0,
                north: 2000.0,
                pixel_size: 1.0,
                width: 2,
                height: 2,
            }
        );
    }

    #[test]
    fn coarsen_max_passes_over_cells_without_a_value() {
        let nan = f32::NAN;
        let fine = raster(4, 2, 0.5, vec![nan, 3.0, nan, nan, nan, nan, nan, nan]);
        let coarse = coarsen_max(&fine, 2).unwrap();
        assert_eq!(coarse.data[0], 3.0);
        assert!(coarse.data[1].is_nan());
    }

    #[test]
    fn coarsen_max_refuses_a_size_that_does_not_divide() {
        let fine = raster(5, 4, 0.5, vec![1.0; 20]);
        assert!(coarsen_max(&fine, 2).is_err());
        assert!(coarsen_max(&fine, 0).is_err());
    }

    #[test]
    fn a_store_can_cache_another_form_than_it_downloads() {
        // The tile comes with cells of 0.5 m and is cached with cells of 1 m.
        let grid = GridSpec {
            west: f64::from(EAST.east_km) * 1000.0,
            north: f64::from(EAST.north_km + 1) * 1000.0,
            pixel_size: 0.5,
            width: 2000,
            height: 2000,
        };
        let fetcher = MapFetcher::default();
        fetcher.insert(&url("e"), geotiff(&grid, &vec![7.0; 4_000_000]));
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("e.tif");
        let reduce = |bytes: &[u8], tile: TileId| {
            let fine = decode_float_tiff(bytes).and_then(|r| whole_tile(r, tile, &[0.5]))?;
            let coarse = coarsen_max(&fine, 2)?;
            let cached = crate::float_tiff::encode_float_geotiff(&coarse)?;
            Ok((coarse, cached))
        };
        let store = TileStore {
            label: "surface model (test)",
            list_url: "https://tiles.example/list",
            source: "https://tiles.example",
            url: &url,
            cache_path: &|name| dir.path().join(format!("{name}.tif")),
            marker_path: &|name| dir.path().join(format!("{name}.missing")),
            known_missing: &|path| path.exists(),
            precheck: &|_| Ok(()),
            prepare: &reduce,
            explain_empty: true,
        };
        let run = |fetcher: &MapFetcher| {
            let area = AreaTiles {
                named: vec![(EAST, "e".to_string())],
                missing: Vec::new(),
            };
            let out = dir.path().join("out.tif");
            let mut warnings = Vec::new();
            mosaic_tiles(
                fetcher,
                FetchMode::Online,
                &BBOX,
                &store,
                area,
                dir.path(),
                &out,
                &mut warnings,
            )
            .unwrap();
            // The mosaic has a world file, no GeoTIFF tags: read its values as they are.
            let bytes = std::fs::read(&out).unwrap();
            let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
            let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
                panic!("not float")
            };
            values
        };
        let mosaic = run(&fetcher);
        // The east half of the area lies in the tile.
        assert_eq!(mosaic[100 * 200 + 150], 7.0);
        assert!(mosaic[100 * 200 + 50].is_nan());
        let kept = read_cached(&cache, EAST).expect("the cached file is the 1 m tile");
        assert_eq!((kept.grid.width, kept.grid.height), (1000, 1000));
        assert!(std::fs::metadata(&cache).unwrap().len() < 1_000_000);
        // The second run reads the reduced file and asks for nothing.
        let silent = MapFetcher::default();
        assert_eq!(run(&silent)[100 * 200 + 150], 7.0);
        assert_eq!(silent.calls.get(), 0);
    }
}
