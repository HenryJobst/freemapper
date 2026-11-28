// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bayerische Vermessungsverwaltung (LDBV, `geodaten.bayern.de`, CC BY 4.0): the orthophoto
//! from the WMS, the DGM1 from 1 km GeoTIFF tiles, and the surface model from the DOM20 tiles
//! (20 cm, computed from the aerial pictures), which are cached reduced to 1 m.
//!
//! The tiles are named by their south-west corner in km of EPSG:25832, in the whole state, so
//! their addresses follow from the map area. A cached tile carries the day the server last
//! changed it ([`crate::ldbv_files`]).

#[cfg(any(test, feature = "testing"))]
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;

use crate::crs::BoundingBox;
#[cfg(any(test, feature = "testing"))]
use crate::crs::Crs;
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher, one_line, write_cache_file};
use crate::ldbv_files::{Checks, Current, Published, cached, current, drop_older, missing_note};
use crate::note::{Note, note};
use crate::sources::float_tiff::{
    decode_float_tiff, encode_described_geotiff, encode_float_geotiff, gdal_metadata_item,
    tiff_description,
};
use crate::sources::height_model::{
    AreaTiles, HeightModel, Prepared, TileStore, as_delivered, centre_tile, coarsen_max,
    mosaic_tiles, whole_tile,
};
use crate::sources::region::{BAYERN, tile_centre_in};
use crate::sources::tiff_range::{TiledTiff, read_blocks, read_layout};
use crate::sources::wms::WmsService;
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS (DOP20). MaxWidth and MaxHeight of the service are 6500.
#[cfg(any(test, feature = "testing"))]
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://geoservices.bayern.de/od/wms/dop/v1/dop20"),
    layer: Cow::Borrowed("by_dop20c"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 6000,
};
/// The queryable layer with the flights; its field `ua` is the day of the flight.
const WMS_INFO_LAYER: &str = "by_dop20_info";
/// The field with the flight date, with its equals sign: the two letters alone could stand
/// anywhere in an answer.
const WMS_DATE_FIELD: &str = "ua =";

/// The file server of the tiles.
pub const FILES: &str = "https://download1.bayernwolke.de/a";

/// The provider id of the adapter.
pub const PROVIDER: &str = "ldbv";
/// The cache folder of the provider.
const CACHE_DIR: &str = "ldbv";
/// The state as the missing-tiles warning names it.
const STATE: &str = "Bavaria";

/// How many cells of a DOM20 tile make one cell of 1 m, along a side.
const DOM20_FACTOR: u32 = 5;
/// The 20 cm pixels along a side of a DOM20 tile, and of one of its blocks (ADR 0141).
const DOM20_SIDE_PX: u32 = 5000;
pub(crate) const DOM20_BLOCK_PX: u32 = 256;

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the layer of the flights; `None` outside the coverage or when the answer
/// has none.
pub fn flight_date(fetcher: &dyn Fetcher, wms: &WmsService, bbox: &BoundingBox) -> Option<String> {
    freemapper_sources::dates::feature_info_date(
        fetcher,
        (&wms.url, WMS_INFO_LAYER, WMS_DATE_FIELD),
        wms.crs,
        bbox,
    )
}

/// The two height models.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Model {
    /// The terrain model, 1 m.
    Dgm1,
    /// The surface model of 20 cm, computed from the aerial pictures of the orthophoto flight.
    Dom20,
}

impl Model {
    pub fn label(self) -> &'static str {
        match self {
            Model::Dgm1 => "terrain model (LDBV DGM1)",
            Model::Dom20 => "surface model (LDBV DOM20)",
        }
    }

    /// `691_5334`: the south-west corner of the tile in km of EPSG:25832.
    pub fn stem(self, tile: TileId) -> String {
        format!("{}_{}", tile.east_km, tile.north_km)
    }

    /// Where the agency publishes the tile, below the file server `files`.
    pub fn tile_url(self, files: &str, tile: TileId) -> String {
        match self {
            Model::Dgm1 => format!("{files}/dgm/dgm1/{}.tif", self.stem(tile)),
            Model::Dom20 => format!("{files}/dom20/DOM/32{}_20_DOM.tif", self.stem(tile)),
        }
    }

    /// The cache folder of the model.
    pub fn dir(self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(CACHE_DIR).join(match self {
            Model::Dgm1 => "dgm1",
            Model::Dom20 => "dom20",
        })
    }

    /// Whether the tile needs no download: a copy is cached, or the server did not have it.
    pub fn is_cached(self, cache: &CacheDir, tile: TileId) -> bool {
        let (dir, stem) = (self.dir(cache), self.stem(tile));
        cached(&dir, &stem, TILE_EXT).is_some() || missing_note(&dir, &stem).exists()
    }
}

/// The extension of a cached tile.
const TILE_EXT: &str = "tif";

/// Where the flight date of a cached DOM20 tile is kept: the reduced tile has no metadata.
fn flight_path(dir: &Path, stem: &str) -> PathBuf {
    dir.join(format!("{stem}.flight"))
}

/// How the ImageDescription of a cached DOM20 copy that holds only some blocks starts, before
/// the numbers of the blocks (ADR 0141). The list lives in the copy, so that one rename writes
/// both: a crash, or two projects writing the same tile, cannot leave a list that names blocks
/// the copy lacks. A copy without it holds the whole tile, as every copy cached before.
const BLOCKS_READ: &str = "freemapper DOM20 blocks read:";

/// The blocks read into the copy at `copy`: `None` for a whole tile, empty when there is no copy
/// or it cannot be read (it is then written anew).
fn blocks_read(copy: &Path) -> Option<Vec<usize>> {
    if !copy.exists() {
        return Some(Vec::new());
    }
    match tiff_description(copy) {
        Ok(Some(text)) if text.starts_with(BLOCKS_READ) => Some(
            text[BLOCKS_READ.len()..]
                .split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect(),
        ),
        Ok(_) => None,
        Err(_) => Some(Vec::new()),
    }
}

/// The blocks of a DOM20 tile (`side_px` 20 cm pixels a side, in blocks of `block_px`) under the
/// 1 m cells of `tile` that `bbox` covers, as the mosaic lays them (`GridSpec::covering`).
pub(crate) fn blocks_of(
    tile: TileId,
    bbox: &BoundingBox,
    side_px: u32,
    block_px: u32,
) -> Vec<usize> {
    let cells = crate::raster::GridSpec::covering(bbox, 1.0);
    let (west, north) = (
        f64::from(tile.east_km) * 1000.0,
        f64::from(tile.north_km + 1) * 1000.0,
    );
    let side_m = f64::from(side_px / DOM20_FACTOR);
    let clamp = |v: f64| v.clamp(0.0, side_m) as u32;
    let (x0, x1) = (
        clamp(cells.west - west),
        clamp(cells.west + f64::from(cells.width) - west),
    );
    let (y0, y1) = (
        clamp(north - cells.north),
        clamp(north - cells.north + f64::from(cells.height)),
    );
    if x1 <= x0 || y1 <= y0 {
        return Vec::new();
    }
    let block = |cell: u32| cell * DOM20_FACTOR / block_px;
    let last = |cell: u32| (cell * DOM20_FACTOR - 1) / block_px;
    let across = side_px.div_ceil(block_px);
    (block(y0)..=last(y1))
        .flat_map(|by| (block(x0)..=last(x1)).map(move |bx| (by * across + bx) as usize))
        .collect()
}

/// Reads into the cached copy `copy` of the DOM20 `tile` the blocks the area `bbox` needs and the
/// copy lacks, by range (ADR 0141): each 1 m cell is the highest of its 25 pixels, and a cell on
/// a block border takes the higher of what each read gives, which ends at the same value as
/// reading both blocks at once. Offline, blocks missing are "not cached". Returns false when the
/// server ignores ranges; then the copy is gone and the whole tile is downloaded.
#[allow(clippy::too_many_arguments)]
fn grow_dom20(
    fetcher: &dyn Fetcher,
    files: &str,
    mode: FetchMode,
    dir: &Path,
    tile: TileId,
    bbox: &BoundingBox,
    copy: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<bool, FetchError> {
    let url = Model::Dom20.tile_url(files, tile);
    let Some(read) = blocks_read(copy) else {
        return Ok(true);
    };
    let needed = blocks_of(tile, bbox, DOM20_SIDE_PX, DOM20_BLOCK_PX);
    let lacking: Vec<usize> = needed.into_iter().filter(|b| !read.contains(b)).collect();
    if lacking.is_empty() {
        return Ok(true);
    }
    if mode == FetchMode::Offline {
        return Err(FetchError::Http {
            url,
            message: format!(
                "offline and not cached: {} tile {}; run `freemapper fetch` online",
                Model::Dom20.label(),
                Model::Dom20.stem(tile)
            ),
        });
    }
    // A server that ignores ranges, or a file laid out otherwise than the blocks are counted
    // in, is downloaded whole, as before; a tile gone from the server keeps what is cached.
    let whole = |why: String, progress: &mut dyn FnMut(Note)| {
        progress(note!(
            "progress-surface-whole",
            tile = Model::Dom20.stem(tile),
            reason = why
        ));
        let _ = std::fs::remove_file(copy);
        Ok(false)
    };
    let layout = match read_layout(fetcher, &url) {
        Ok(Ok(layout)) => layout,
        Ok(Err(what)) => return whole(what, progress),
        Err(FetchError::RangeIgnored { .. }) => {
            return whole("the server ignores ranges".to_string(), progress);
        }
        Err(FetchError::NotFound(_)) => return Ok(true),
        Err(e) => return Err(e),
    };
    if let Err(what) = whole_dom20(&layout, tile) {
        return whole(what, progress);
    }
    progress(note!(
        "progress-surface-blocks",
        tile = Model::Dom20.stem(tile),
        count = lacking.len(),
        total = layout.block_count(),
        megabytes = format!("{:.1}", layout.bytes_of(&lacking) as f64 / 1e6)
    ));
    let blocks = read_blocks(fetcher, &url, &layout, &lacking)?;
    let grid = crate::raster::GridSpec {
        west: f64::from(tile.east_km) * 1000.0,
        north: f64::from(tile.north_km + 1) * 1000.0,
        pixel_size: 1.0,
        width: DOM20_SIDE_PX / DOM20_FACTOR,
        height: DOM20_SIDE_PX / DOM20_FACTOR,
    };
    // A copy that cannot be read has no blocks (`blocks_read`) and starts anew.
    let mut coarse = crate::sources::height_model::read_cached(copy, tile)
        .filter(|_| !read.is_empty())
        .unwrap_or_else(|| crate::raster::Raster::filled(grid, f32::NAN));
    let across = layout.blocks_across();
    let (bw, bh) = (layout.block_width, layout.block_height);
    for (i, values) in &blocks {
        let (bx, by) = (*i as u32 % across, *i as u32 / across);
        for (y, row) in values
            .chunks_exact(bw as usize)
            .enumerate()
            .take(bh as usize)
        {
            let py = by * bh + y as u32;
            if py >= DOM20_SIDE_PX {
                break;
            }
            let start = (py / DOM20_FACTOR * grid.width) as usize;
            for (x, z) in row.iter().enumerate() {
                let px = bx * bw + x as u32;
                if px >= DOM20_SIDE_PX {
                    break;
                }
                let cell = &mut coarse.data[start + (px / DOM20_FACTOR) as usize];
                // `f32::max` passes over a NaN on either side.
                *cell = cell.max(*z);
            }
        }
    }
    let mut all: Vec<usize> = read.into_iter().chain(lacking).collect();
    all.sort_unstable();
    all.dedup();
    let list: Vec<String> = all.iter().map(ToString::to_string).collect();
    let described = format!("{BLOCKS_READ}{}", list.join(","));
    let kept = encode_described_geotiff(&coarse, Some(&described)).map_err(|message| {
        FetchError::Http {
            url: url.clone(),
            message,
        }
    })?;
    write_cache_file(copy, &kept)?;
    let flight = flight_path(dir, &Model::Dom20.stem(tile));
    match layout
        .metadata_item("BILDFLUG_DATUM")
        .and_then(|date| iso_date(&date))
    {
        Some(date) => write_cache_file(&flight, date.as_bytes())?,
        None => {
            let _ = std::fs::remove_file(&flight);
        }
    }
    Ok(true)
}

/// The blocks of the DOM20 `tile` that an area `bbox` (EPSG:25832) needs and its cached copy
/// lacks, for the estimate; none where the server does not have the tile.
pub fn dom20_blocks_to_read(cache: &CacheDir, tile: TileId, bbox: &BoundingBox) -> usize {
    let (dir, stem) = (Model::Dom20.dir(cache), Model::Dom20.stem(tile));
    if missing_note(&dir, &stem).exists() {
        return 0;
    }
    let copy = cached(&dir, &stem, TILE_EXT).unwrap_or_else(|| dir.join(format!("{stem}.none")));
    match blocks_read(&copy) {
        None => 0,
        Some(read) => blocks_of(tile, bbox, DOM20_SIDE_PX, DOM20_BLOCK_PX)
            .into_iter()
            .filter(|b| !read.contains(b))
            .count(),
    }
}

/// Whether `layout` is a whole DOM20 tile as the blocks are counted: 5000 × 5000 px of 20 cm on
/// the tile, in blocks of 256 px.
fn whole_dom20(layout: &TiledTiff, tile: TileId) -> Result<(), String> {
    if (layout.block_width, layout.block_height) != (DOM20_BLOCK_PX, DOM20_BLOCK_PX) {
        return Err(format!(
            "blocks of {} x {} px, not {DOM20_BLOCK_PX}",
            layout.block_width, layout.block_height
        ));
    }
    let g = layout.grid();
    let near = |a: f64, b: f64| (a - b).abs() < 1e-3;
    let fits = (g.width, g.height) == (DOM20_SIDE_PX, DOM20_SIDE_PX)
        && near(g.pixel_size, 0.2)
        && near(g.west, f64::from(tile.east_km) * 1000.0)
        && near(g.north, f64::from(tile.north_km + 1) * 1000.0);
    if fits {
        Ok(())
    } else {
        Err(format!(
            "the raster (W {} N {}, {} x {} px of {} m) is not the whole tile {}",
            g.west,
            g.north,
            g.width,
            g.height,
            g.pixel_size,
            Model::Dom20.stem(tile)
        ))
    }
}

/// `2024-06-29` of `29.06.2024`.
fn iso_date(day_month_year: &str) -> Option<String> {
    chrono::NaiveDate::parse_from_str(day_month_year.trim(), "%d.%m.%Y")
        .ok()
        .map(|date| date.format("%Y-%m-%d").to_string())
}

/// A DOM20 tile as it is pasted and cached: reduced to 1 m, each cell the highest of its 25
/// values. The flight date of its metadata is kept beside it.
fn reduced(dir: &Path, bytes: &[u8], tile: TileId) -> Result<Prepared, String> {
    let fine = decode_float_tiff(bytes).and_then(|raster| whole_tile(raster, tile, &[0.2]))?;
    let coarse = coarsen_max(&fine, DOM20_FACTOR)?;
    let kept = encode_float_geotiff(&coarse)?;
    let flight = flight_path(dir, &Model::Dom20.stem(tile));
    match gdal_metadata_item(bytes, "BILDFLUG_DATUM").and_then(|date| iso_date(&date)) {
        Some(date) => write_cache_file(&flight, date.as_bytes()).map_err(|e| e.to_string())?,
        None => {
            let _ = std::fs::remove_file(&flight);
        }
    }
    Ok((coarse, kept))
}

/// The tile stem of the name of a cached copy (`691_5334-20260115`).
fn stem_of(name: &str) -> &str {
    name.rsplit_once('-').map_or(name, |(stem, _)| stem)
}

/// The tile of a stem (`691_5334`).
fn tile_of(stem: &str) -> Option<TileId> {
    let (east, north) = stem.split_once('_')?;
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), mosaicked into a Float32 GeoTIFF `out`
/// with its world file. The addresses follow from the map area; a tile the server does not
/// have is missing when its centre lies in Bavaria and silent beyond the border.
#[allow(clippy::too_many_arguments)]
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    files: &str,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<HeightModel, FetchError> {
    const NOT_ON_SERVER: &str = "not on the server (404)";
    let dir = model.dir(cache);
    let mut area = AreaTiles::default();
    let mut older = Vec::new();
    let mut checks = Checks::default();
    for tile in tiles_covering(bbox) {
        let (stem, url) = (model.stem(tile), model.tile_url(files, tile));
        let file = Published {
            dir: &dir,
            stem: &stem,
            ext: TILE_EXT,
            url: &url,
            // Beyond the border a tile that is not there stays not there; inside Bavaria the
            // server is asked again at every run.
            missing_holds: !tile_centre_in(tile, BAYERN),
        };
        let name = |path: &Path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        match current(fetcher, mode, &file, &mut checks, progress)? {
            Current::Cached(path) => area.named.push((tile, name(&path))),
            Current::Fetch { path, older: stale } => {
                area.named.push((tile, name(&path)));
                older.extend(stale);
            }
            Current::Missing if tile_centre_in(tile, BAYERN) => {
                area.missing.push((tile, NOT_ON_SERVER.to_string()));
            }
            Current::Missing => {}
        }
    }
    // The DOM20 is read by range, block by block, into its cached copy (ADR 0141); the mosaic
    // then finds it cached. A server that ignores ranges leaves the whole tile to the mosaic.
    if model == Model::Dom20 {
        for (tile, name) in &area.named {
            let copy = dir.join(format!("{name}.{TILE_EXT}"));
            if missing_note(&dir, &model.stem(*tile)).exists() {
                continue;
            }
            grow_dom20(fetcher, files, mode, &dir, *tile, bbox, &copy, progress)?;
        }
    }
    let list_url = format!("{files}/");
    let tile_url = |name: &str| {
        tile_of(stem_of(name)).map_or_else(|| list_url.clone(), |tile| model.tile_url(files, tile))
    };
    let reduce = |bytes: &[u8], tile: TileId| reduced(&dir, bytes, tile);
    let store = TileStore {
        label: model.label(),
        list_url: &list_url,
        source: files,
        url: &tile_url,
        cache_path: &|name| dir.join(format!("{name}.{TILE_EXT}")),
        marker_path: &|name| missing_note(&dir, stem_of(name)),
        // Whether the server has a tile was asked above.
        known_missing: &|_| false,
        // The file server answers some errors with an HTML page.
        precheck: &|bytes| {
            if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
                return Ok(());
            }
            let start = String::from_utf8_lossy(&bytes[..bytes.len().min(1000)]);
            Err(format!("not a GeoTIFF: {}", one_line(&start, 200)))
        },
        prepare: match model {
            Model::Dgm1 => &as_delivered,
            Model::Dom20 => &reduce,
        },
        explain_empty: true,
    };
    let mut warnings = Vec::new();
    let mut missing = mosaic_tiles(fetcher, mode, bbox, &store, area, root, out, &mut warnings)?;
    drop_older(&older);
    missing.retain(|tile| tile_centre_in(*tile, BAYERN));
    let centre = centre_tile(bbox);
    let date = match model {
        // The terrain tiles carry no date.
        Model::Dgm1 => None,
        Model::Dom20 if missing.contains(&centre) => None,
        Model::Dom20 => std::fs::read_to_string(flight_path(&dir, &model.stem(centre)))
            .ok()
            .map(|date| date.trim().to_string())
            .filter(|date| !date.is_empty()),
    };
    Ok(HeightModel {
        missing,
        date,
        warnings,
        ..HeightModel::default()
    })
}

/// The rasters of the Bayerische Vermessungsverwaltung (Bavaria): requested for the area and
/// stored in the project folder.
pub struct LdbvBackend {
    info: ProviderInfo,
    /// The orthophoto WMS, as the manifest says it.
    wms: WmsService,
    /// The file server of the tiles of DGM1 and DOM20, as the manifest says it.
    files: String,
}

impl LdbvBackend {
    /// The backend that asks `wms` for the orthophoto and the file server at `files` for the
    /// tiles of the models.
    pub fn with_files(wms: WmsService, files: String) -> Self {
        Self {
            wms,
            files,
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(CACHE_DIR.to_string()),
                terrain_source: "ldbv-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl LdbvBackend {
    /// The backend with the WMS the manifest of the state declares.
    pub fn new() -> Self {
        Self::with_files(WMS, FILES.to_string())
    }
}

#[cfg(any(test, feature = "testing"))]
impl Default for LdbvBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LdbvBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox, (&self.wms, &self.files))
    }
}

/// The LDBV rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM20 from
/// the 1 km tiles. The DOM20 is only a template: an unavailable one costs a warning.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
    (wms, files): (&WmsService, &str),
) -> Result<(), AdapterError> {
    let (fetcher, mode) = (env.fetcher, env.mode);
    let cache = env.cache;
    let dir = &project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    warn_beyond_state(env, project, PROVIDER, "LDBV");
    let flight = || flight_date(fetcher, wms, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (&OrthophotoSource::Wms(wms.clone()), "LDBV", &flight, None),
    )?;
    fetch_area_heights(
        env,
        project,
        bbox,
        STATE,
        (Model::Dgm1.label(), Model::Dom20.label()),
        &mut |surface, out, progress| {
            let model = if surface { Model::Dom20 } else { Model::Dgm1 };
            fetch_model(fetcher, files, cache, mode, model, bbox, dir, out, progress)
        },
        &mut |_, found, _, progress| {
            for warning in &found.warnings {
                progress(warning.clone());
            }
        },
    )?;
    project.save()?;
    Ok(())
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    use chrono::NaiveDate;

    use super::{FILES, Model, WMS};

    /// What every GetFeatureInfo request of the orthophoto WMS starts with. (Here, not at the
    /// top: the check of the note texts reads a source file up to its first test marker.)
    pub const FLIGHT_INFO_PREFIX: &str = "https://geoservices.bayern.de/od/wms/dop/v1/dop20?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo";
    use crate::fetch::test_support::{MapFetcher, geotiff, solid_jpeg, tiled_float_tiff};
    use crate::raster::GridSpec;
    use crate::tiles::TileId;

    /// The GetFeatureInfo text answer of the orthophoto WMS at Regensburg (flight 11.05.2025),
    /// as the server gave it on 2026-11-14.
    pub const FLIGHT_INFO: &str = include_str!("../../../../tests/fixtures/bayern-dop-info.txt");

    /// The day the synthetic tiles were last changed on the server.
    pub fn changed() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    }

    /// A whole DGM1 tile: `z` at its west edge, rising 0.01 m per metre east.
    pub fn dgm1_tile(tile: TileId, z: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let heights: Vec<f32> = (0..1000 * 1000)
            .map(|i| z + (i % 1000) as f32 * 0.01)
            .collect();
        geotiff(&grid, &heights)
    }

    /// A whole DOM20 tile (5000 × 5000 cells of 20 cm) of the height `z`, with `peak` in the
    /// cell `(x, y)` if given, and the flight date (`DD.MM.YYYY`) in its GDAL metadata.
    pub fn dom20_tile(
        tile: TileId,
        z: f32,
        peak: Option<(u32, u32, f32)>,
        flight: &str,
    ) -> Vec<u8> {
        const SIDE: u32 = 5000;
        let mut heights = vec![z; (SIDE * SIDE) as usize];
        if let Some((x, y, top)) = peak {
            heights[(y * SIDE + x) as usize] = top;
        }
        dom20_of(tile, &heights, flight)
    }

    /// A whole DOM20 tile whose heights differ from pixel to pixel (400 to 404.9 m).
    pub fn dom20_varied(tile: TileId, flight: &str) -> Vec<u8> {
        const SIDE: u32 = 5000;
        let heights: Vec<f32> = (0..SIDE * SIDE)
            .map(|i| {
                let (x, y) = (i % SIDE, i / SIDE);
                400.0 + ((x * 7 + y * 13) % 50) as f32 * 0.1
            })
            .collect();
        dom20_of(tile, &heights, flight)
    }

    /// The DOM20 tile of `heights` as the agency writes it: 256 px blocks, LZW with the floating
    /// point predictor, the IFD at the end, the flight date in the GDAL metadata.
    fn dom20_of(tile: TileId, heights: &[f32], flight: &str) -> Vec<u8> {
        const SIDE: u32 = 5000;
        let grid = GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 0.2,
            width: SIDE,
            height: SIDE,
        };
        let metadata = format!(
            "<GDALMetadata>\n  <Item name=\"BILDFLUG_DATUM\">{flight}</Item>\n</GDALMetadata>\n"
        );
        tiled_float_tiff(&grid, heights, 256, 3, Some(&metadata))
    }

    /// Puts `bytes` on the server as the tile of `model`, last changed on `changed`.
    pub fn serve_tile(
        fetcher: &MapFetcher,
        model: Model,
        tile: TileId,
        bytes: Vec<u8>,
        changed: NaiveDate,
    ) {
        let url = model.tile_url(FILES, tile);
        fetcher.insert(&url, bytes);
        fetcher.set_modified(&url, changed);
    }

    /// Serves the LDBV rasters of a project in `bbox` (EPSG:25832): a WMS orthophoto of the
    /// right size with the flight date of [`FLIGHT_INFO`], the DGM1 tiles (20 m rising 0.01 m
    /// per metre east) and the DOM20 tiles (32 m, flown on 29.06.2024), all last changed on
    /// [`changed`].
    pub fn serve_synthetic_ldbv(fetcher: &MapFetcher, bbox: &crate::crs::BoundingBox) {
        let photo = GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [0, 100, 200]));
        }
        fetcher.insert_prefix(FLIGHT_INFO_PREFIX, FLIGHT_INFO.as_bytes().to_vec());
        for tile in crate::tiles::tiles_covering(bbox) {
            serve_tile(fetcher, Model::Dgm1, tile, dgm1_tile(tile, 20.0), changed());
            let surface = dom20_tile(tile, 32.0, None, "29.06.2024");
            serve_tile(fetcher, Model::Dom20, tile, surface, changed());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        FLIGHT_INFO, FLIGHT_INFO_PREFIX, changed, dgm1_tile, dom20_tile, dom20_varied, serve_tile,
    };
    use super::*;
    use crate::fetch::ByteRange;
    use crate::fetch::Cache;
    use crate::fetch::test_support::MapFetcher;
    use std::cell::RefCell;

    /// Two tiles side by side in Munich (centre at the tile border 692 km).
    const BBOX: BoundingBox = BoundingBox {
        min_e: 691_900.0,
        min_n: 5_334_400.0,
        max_e: 692_100.0,
        max_n: 5_334_600.0,
    };
    const WEST: TileId = TileId {
        east_km: 691,
        north_km: 5334,
    };
    const EAST: TileId = TileId {
        east_km: 692,
        north_km: 5334,
    };

    /// The mosaic `out` is written without GeoTIFF tags (its world file has the grid).
    fn read_out(path: &Path) -> Vec<f32> {
        let bytes = std::fs::read(path).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
        let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().unwrap() else {
            panic!("not float")
        };
        values
    }

    /// Fetches `model` for `bbox` into `<dir>/out.tif` with the cache in `dir`.
    fn run(
        fetcher: &MapFetcher,
        dir: &Path,
        mode: FetchMode,
        model: Model,
        bbox: &BoundingBox,
    ) -> Result<(HeightModel, Vec<f32>), String> {
        let cache = Cache::new(dir.join("cache"));
        let out = dir.join("out.tif");
        let found = fetch_model(
            fetcher,
            FILES,
            &cache,
            mode,
            model,
            bbox,
            dir,
            &out,
            &mut |_| {},
        )
        .map_err(|e| e.to_string())?;
        Ok((found, read_out(&out)))
    }

    fn serve_terrain(fetcher: &MapFetcher) {
        serve_tile(
            fetcher,
            Model::Dgm1,
            WEST,
            dgm1_tile(WEST, 500.0),
            changed(),
        );
        serve_tile(
            fetcher,
            Model::Dgm1,
            EAST,
            dgm1_tile(EAST, 600.0),
            changed(),
        );
    }

    #[test]
    fn tile_addresses_follow_the_agency() {
        assert_eq!(
            Model::Dgm1.tile_url(FILES, WEST),
            "https://download1.bayernwolke.de/a/dgm/dgm1/691_5334.tif"
        );
        assert_eq!(
            Model::Dom20.tile_url(FILES, WEST),
            "https://download1.bayernwolke.de/a/dom20/DOM/32691_5334_20_DOM.tif"
        );
        assert_eq!(Model::Dgm1.stem(WEST), "691_5334");
    }

    #[test]
    fn the_flight_date_is_the_ua_field() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(FLIGHT_INFO_PREFIX, FLIGHT_INFO.as_bytes().to_vec());
        assert_eq!(
            flight_date(&fetcher, &WMS, &BBOX).as_deref(),
            Some("11.05.2025")
        );
        let url = fetcher.log.borrow()[0].clone();
        for part in [
            "LAYERS=by_dop20_info",
            "QUERY_LAYERS=by_dop20_info",
            "CRS=EPSG:25832",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(url.contains(part), "{part} in {url}");
        }
        for answer in [
            "GetFeatureInfo results:\n\n  Search returned no results.\n",
            "    nummer = '125039'\n    unr = '0'\n",
            "    ua = '2025-05-11'\n",
            "<ServiceExceptionReport/>",
        ] {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(&WMS.url, answer.as_bytes().to_vec());
            assert_eq!(flight_date(&fetcher, &WMS, &BBOX), None, "{answer}");
        }
    }

    #[test]
    fn the_terrain_is_the_mosaic_of_the_tiles() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        assert!(found.missing.is_empty(), "{:?}", found.missing);
        assert_eq!(found.date, None);
        assert_eq!(values.len(), 200 * 200);
        // The last column of the west tile and the first of the east tile.
        assert_eq!(values[100 * 200 + 99], 500.0 + 999.0 * 0.01);
        assert_eq!(values[100 * 200 + 100], 600.0);
        let tiles = dir.path().join("cache/ldbv/dgm1");
        assert!(tiles.join("691_5334-20260115.tif").exists());
        assert!(tiles.join("692_5334-20260115.tif").exists());
        let cache = Cache::new(dir.path().join("cache"));
        assert!(Model::Dgm1.is_cached(&cache, WEST));
        assert!(!Model::Dom20.is_cached(&cache, WEST));
    }

    #[test]
    fn a_second_run_downloads_nothing_and_offline_asks_nothing() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        let (_, first) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        let downloads = fetcher.log.borrow().len();
        assert_eq!(downloads, 2);
        let (_, second) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        // Only the two checks went out, no tile a second time.
        assert_eq!(fetcher.log.borrow().len(), downloads);
        assert_eq!(first, second);
        let silent = MapFetcher::default();
        let (_, offline) =
            run(&silent, dir.path(), FetchMode::Offline, Model::Dgm1, &BBOX).unwrap();
        assert_eq!(silent.calls.get(), 0);
        assert_eq!(first, offline);
    }

    #[test]
    fn a_newer_tile_replaces_the_cached_one() {
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        let later = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        serve_tile(&fetcher, Model::Dgm1, WEST, dgm1_tile(WEST, 700.0), later);
        let (_, values) = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap();
        assert_eq!(values[100 * 200 + 99], 700.0 + 999.0 * 0.01);
        let tiles = dir.path().join("cache/ldbv/dgm1");
        assert!(tiles.join("691_5334-20260601.tif").exists());
        assert!(!tiles.join("691_5334-20260115.tif").exists());
        assert!(tiles.join("692_5334-20260115.tif").exists());
    }

    #[test]
    fn a_missing_tile_warns_inside_bavaria_only() {
        // A strip across the Iller south of Ulm: its west end is Baden-Württemberg.
        let strip = BoundingBox {
            min_e: 570_900.0,
            min_n: 5_358_400.0,
            max_e: 575_100.0,
            max_n: 5_358_600.0,
        };
        let tile = |east_km| TileId {
            east_km,
            north_km: 5358,
        };
        assert!(
            !tile_centre_in(tile(570), BAYERN),
            "570 lies beyond the border"
        );
        assert!(tile_centre_in(tile(575), BAYERN), "575 lies in Bavaria");
        // The server has the tiles from 573 on, as it has none beyond the border.
        let fetcher = MapFetcher::default();
        for east in 573..=575 {
            let bytes = dgm1_tile(tile(east), 480.0);
            serve_tile(&fetcher, Model::Dgm1, tile(east), bytes, changed());
        }
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &strip).unwrap();
        let expected: Vec<TileId> = (570..573)
            .map(tile)
            .filter(|t| tile_centre_in(*t, BAYERN))
            .collect();
        assert_eq!(found.missing, expected);
        assert!(!found.missing.contains(&tile(570)));
        assert!(values[0].is_nan());
        assert!(values[100 * 4200 + 4100].is_finite());
        // Offline the same: the tiles beyond the border are remembered as not there.
        let silent = MapFetcher::default();
        let (again, _) = run(&silent, dir.path(), FetchMode::Offline, Model::Dgm1, &strip).unwrap();
        assert_eq!(again.missing, expected);
        assert_eq!(silent.calls.get(), 0);
        // A tile inside Bavaria that the server lacks is missing, with a warning to come.
        let other = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        for east in 573..=574 {
            let bytes = dgm1_tile(tile(east), 480.0);
            serve_tile(&fetcher, Model::Dgm1, tile(east), bytes, changed());
        }
        let (found, _) = run(
            &fetcher,
            other.path(),
            FetchMode::Online,
            Model::Dgm1,
            &strip,
        )
        .unwrap();
        assert!(found.missing.contains(&tile(575)), "{:?}", found.missing);
        // The next run asks for that tile again and finds it; the tiles beyond the border
        // are not asked for a second time.
        let bytes = dgm1_tile(tile(575), 480.0);
        serve_tile(&fetcher, Model::Dgm1, tile(575), bytes, changed());
        let before = fetcher.calls.get();
        let (found, _) = run(
            &fetcher,
            other.path(),
            FetchMode::Online,
            Model::Dgm1,
            &strip,
        )
        .unwrap();
        assert!(!found.missing.contains(&tile(575)), "{:?}", found.missing);
        // Three checks (573 to 575) and one download; 570 to 572 are remembered.
        let beyond = (570..573)
            .filter(|east| !tile_centre_in(tile(*east), BAYERN))
            .count();
        assert_eq!(fetcher.calls.get() - before, 4 + (3 - beyond));
    }

    #[test]
    fn the_surface_model_is_cached_reduced_with_its_flight_date() {
        // Inside one tile; the cell (2502, 2502) of the 20 cm tile is 25 m higher.
        let bbox = BoundingBox {
            min_e: 691_400.0,
            min_n: 5_334_400.0,
            max_e: 691_600.0,
            max_n: 5_334_600.0,
        };
        let fetcher = MapFetcher::default();
        let bytes = dom20_tile(WEST, 400.0, Some((2502, 2502, 425.0)), "29.06.2024");
        serve_tile(&fetcher, Model::Dom20, WEST, bytes, changed());
        let dir = tempfile::tempdir().unwrap();
        let (found, values) =
            run(&fetcher, dir.path(), FetchMode::Online, Model::Dom20, &bbox).unwrap();
        assert_eq!(found.date.as_deref(), Some("2024-06-29"));
        assert_eq!(values[100 * 200 + 100], 425.0);
        assert_eq!(values[100 * 200 + 101], 400.0);
        let kept = dir.path().join("cache/ldbv/dom20/691_5334-20260115.tif");
        let reduced = crate::sources::height_model::read_cached(&kept, WEST)
            .expect("the cached file is the 1 m tile");
        assert_eq!((reduced.grid.width, reduced.grid.height), (1000, 1000));
        assert!(std::fs::metadata(&kept).unwrap().len() < 5_000_000);
        // Offline the date comes from what was kept beside the tile.
        let silent = MapFetcher::default();
        let (again, _) = run(&silent, dir.path(), FetchMode::Offline, Model::Dom20, &bbox).unwrap();
        assert_eq!(again.date.as_deref(), Some("2024-06-29"));
        assert_eq!(silent.calls.get(), 0);
    }

    /// Records the ranges asked for, and refuses every range when told to.
    struct Ranges<'a> {
        inner: &'a MapFetcher,
        asked: RefCell<Vec<ByteRange>>,
    }

    impl<'a> Ranges<'a> {
        fn new(inner: &'a MapFetcher) -> Self {
            Self {
                inner,
                asked: RefCell::new(Vec::new()),
            }
        }

        /// The bytes of the blocks asked for: the ranges before the IFD of the tile WEST, which
        /// lies at the end of the file.
        fn block_bytes(&self) -> u64 {
            let file = self.inner.get(&Model::Dom20.tile_url(FILES, WEST)).unwrap();
            let ifd = u64::from(u32::from_le_bytes([file[4], file[5], file[6], file[7]]));
            self.asked
                .borrow()
                .iter()
                .map(|r| match r {
                    ByteRange::From { start, len } if *start < ifd => *len,
                    _ => 0,
                })
                .sum()
        }
    }

    impl Fetcher for Ranges<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.inner.get(url)
        }

        fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
            self.inner.last_modified(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            self.asked.borrow_mut().push(range);
            self.inner.get_range(url, range)
        }
    }

    /// [`run`] with any fetcher.
    fn run_with(
        fetcher: &dyn Fetcher,
        dir: &Path,
        mode: FetchMode,
        bbox: &BoundingBox,
    ) -> Result<(HeightModel, Vec<f32>), String> {
        let cache = Cache::new(dir.join("cache"));
        let out = dir.join("out.tif");
        let found = fetch_model(
            fetcher,
            FILES,
            &cache,
            mode,
            Model::Dom20,
            bbox,
            dir,
            &out,
            &mut |_| {},
        )
        .map_err(|e| e.to_string())?;
        Ok((found, read_out(&out)))
    }

    /// An area of whole metres inside WEST: `cells` east and south of its north-west corner.
    fn area_in_west(east: (u32, u32), south: (u32, u32)) -> BoundingBox {
        let (west, north) = (691_000.0, 5_335_000.0);
        BoundingBox {
            min_e: west + f64::from(east.0),
            max_e: west + f64::from(east.1),
            min_n: north - f64::from(south.1),
            max_n: north - f64::from(south.0),
        }
    }

    /// The 1 m cells of `bbox` as the whole tile gives them.
    fn whole_tile_cells(bytes: &[u8], bbox: &BoundingBox) -> Vec<f32> {
        let scratch = tempfile::tempdir().unwrap();
        let (coarse, _) = reduced(scratch.path(), bytes, WEST).unwrap();
        let grid = crate::raster::GridSpec::covering(bbox, 1.0);
        let (x0, y0) = (
            (grid.west - 691_000.0) as u32,
            (5_335_000.0 - grid.north) as u32,
        );
        (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
            .map(|(x, y)| coarse.get(x0 + x, y0 + y))
            .collect()
    }

    #[test]
    fn the_dom20_is_read_by_range() {
        let server = MapFetcher::default();
        let bytes = dom20_varied(WEST, "13.06.2025");
        serve_tile(&server, Model::Dom20, WEST, bytes.clone(), changed());
        // 200 m by 100 m, across the border of the first and the second block row (51.2 m).
        let bbox = area_in_west((300, 500), (0, 100));
        let fetcher = Ranges::new(&server);
        let dir = tempfile::tempdir().unwrap();
        let (found, values) = run_with(&fetcher, dir.path(), FetchMode::Online, &bbox).unwrap();
        assert_eq!(found.date.as_deref(), Some("2025-06-13"));
        assert_eq!(values, whole_tile_cells(&bytes, &bbox));
        // 5 blocks across (px 1500 to 2499) and 2 down of the 400: a fortieth of the file.
        let read = fetcher.block_bytes();
        assert!(
            read > 0 && read * 20 < bytes.len() as u64,
            "{read} of {}",
            bytes.len()
        );
        assert!(server.calls.get() > 0);
        // The blocks read are listed inside the copy, so one rename writes both.
        let copy = dir.path().join("cache/ldbv/dom20/691_5334-20260115.tif");
        let listed = blocks_read(&copy).expect("a copy of some blocks");
        assert_eq!(listed, [5, 6, 7, 8, 9, 25, 26, 27, 28, 29]);
        assert!(!copy.with_extension("blocks").exists());
    }

    #[test]
    fn a_second_area_reads_only_the_blocks_it_lacks() {
        let server = MapFetcher::default();
        let bytes = dom20_varied(WEST, "13.06.2025");
        serve_tile(&server, Model::Dom20, WEST, bytes.clone(), changed());
        let dir = tempfile::tempdir().unwrap();
        // Cells 40 to 50 lie in the first block column (px 200 to 254), cells 52 to 60 in the
        // second (px 260 to 304); cell 51 (px 255 to 259) straddles both.
        let first = area_in_west((40, 51), (0, 20));
        let second = area_in_west((52, 61), (0, 20));
        run_with(&server, dir.path(), FetchMode::Online, &first).unwrap();
        let fetcher = Ranges::new(&server);
        let (_, values) = run_with(&fetcher, dir.path(), FetchMode::Online, &second).unwrap();
        assert_eq!(values, whole_tile_cells(&bytes, &second));
        let one_block = fetcher.block_bytes();
        assert!(one_block > 0);
        // The straddling cell needs both blocks, which are read: nothing more is asked for, and
        // its height is the highest of all its 25 pixels.
        let straddling = area_in_west((51, 52), (0, 20));
        let silent = Ranges::new(&server);
        let (_, values) = run_with(&silent, dir.path(), FetchMode::Online, &straddling).unwrap();
        assert_eq!(silent.block_bytes(), 0);
        assert_eq!(values, whole_tile_cells(&bytes, &straddling));
    }

    #[test]
    fn an_old_copy_is_a_whole_tile() {
        let server = MapFetcher::default();
        let bytes = dom20_varied(WEST, "13.06.2025");
        serve_tile(&server, Model::Dom20, WEST, bytes.clone(), changed());
        // A copy cached before the blocks were read by range: the whole tile, no block list.
        let dir = tempfile::tempdir().unwrap();
        let tiles = dir.path().join("cache/ldbv/dom20");
        std::fs::create_dir_all(&tiles).unwrap();
        let (_, kept) = reduced(&tiles, &bytes, WEST).unwrap();
        std::fs::write(tiles.join("691_5334-20260115.tif"), kept).unwrap();
        let bbox = area_in_west((300, 500), (0, 100));
        let fetcher = Ranges::new(&server);
        let (_, values) = run_with(&fetcher, dir.path(), FetchMode::Online, &bbox).unwrap();
        assert!(
            fetcher.asked.borrow().is_empty(),
            "{:?}",
            fetcher.asked.borrow()
        );
        assert_eq!(values, whole_tile_cells(&bytes, &bbox));
    }

    #[test]
    fn offline_with_blocks_missing_is_not_cached() {
        let server = MapFetcher::default();
        serve_tile(
            &server,
            Model::Dom20,
            WEST,
            dom20_tile(WEST, 400.0, None, "13.06.2025"),
            changed(),
        );
        let dir = tempfile::tempdir().unwrap();
        run_with(
            &server,
            dir.path(),
            FetchMode::Online,
            &area_in_west((40, 50), (0, 20)),
        )
        .unwrap();
        let error = run_with(
            &MapFetcher::default(),
            dir.path(),
            FetchMode::Offline,
            &area_in_west((600, 700), (600, 700)),
        )
        .unwrap_err();
        assert!(error.contains("offline and not cached"), "{error}");
        assert!(error.contains("691_5334"), "{error}");
    }

    #[test]
    fn a_newer_tile_starts_a_new_block_list() {
        let server = MapFetcher::default();
        serve_tile(
            &server,
            Model::Dom20,
            WEST,
            dom20_tile(WEST, 400.0, None, "13.06.2025"),
            changed(),
        );
        let dir = tempfile::tempdir().unwrap();
        let bbox = area_in_west((40, 50), (0, 20));
        run_with(&server, dir.path(), FetchMode::Online, &bbox).unwrap();
        let later = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        serve_tile(
            &server,
            Model::Dom20,
            WEST,
            dom20_tile(WEST, 410.0, None, "01.03.2026"),
            later,
        );
        let (found, values) = run_with(&server, dir.path(), FetchMode::Online, &bbox).unwrap();
        assert!(values.iter().all(|z| *z == 410.0));
        assert_eq!(found.date.as_deref(), Some("2026-03-01"));
        let tiles = dir.path().join("cache/ldbv/dom20");
        assert!(blocks_read(&tiles.join("691_5334-20260301.tif")).is_some_and(|b| !b.is_empty()));
        assert!(!tiles.join("691_5334-20260115.tif").exists());
        assert!(
            std::fs::read_dir(&tiles)
                .unwrap()
                .flatten()
                .all(|e| !e.file_name().to_string_lossy().ends_with(".blocks")),
            "the old list goes too"
        );
    }

    /// A server that answers every range with the whole file, as the HTTP fetcher reports it.
    struct IgnoresRanges<'a>(&'a MapFetcher);

    impl Fetcher for IgnoresRanges<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.0.get(url)
        }

        fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
            self.0.last_modified(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            Err(FetchError::RangeIgnored {
                url: url.to_string(),
                range: format!("{range:?}"),
                status: 200,
            })
        }
    }

    #[test]
    fn a_server_that_ignores_ranges_gets_the_whole_tile_downloaded() {
        let server = MapFetcher::default();
        let bytes = dom20_varied(WEST, "13.06.2025");
        serve_tile(&server, Model::Dom20, WEST, bytes.clone(), changed());
        let dir = tempfile::tempdir().unwrap();
        let bbox = area_in_west((300, 500), (0, 100));
        let (_, values) = run_with(
            &IgnoresRanges(&server),
            dir.path(),
            FetchMode::Online,
            &bbox,
        )
        .unwrap();
        assert_eq!(values, whole_tile_cells(&bytes, &bbox));
        let copy = dir.path().join("cache/ldbv/dom20/691_5334-20260115.tif");
        assert_eq!(blocks_read(&copy), None, "the whole tile");
    }

    #[test]
    fn an_unexpected_layout_is_downloaded_whole() {
        // Blocks of 512 px instead of 256: the numbers of the blocks would name other places.
        let server = MapFetcher::default();
        let heights: Vec<f32> = (0..5000 * 5000)
            .map(|i| 400.0 + (i % 37) as f32 * 0.1)
            .collect();
        let grid = crate::raster::GridSpec {
            west: 691_000.0,
            north: 5_335_000.0,
            pixel_size: 0.2,
            width: 5000,
            height: 5000,
        };
        let bytes = crate::fetch::test_support::tiled_float_tiff(&grid, &heights, 512, 3, None);
        serve_tile(&server, Model::Dom20, WEST, bytes.clone(), changed());
        let dir = tempfile::tempdir().unwrap();
        let bbox = area_in_west((300, 500), (0, 100));
        let (_, values) = run_with(&server, dir.path(), FetchMode::Online, &bbox).unwrap();
        assert_eq!(values, whole_tile_cells(&bytes, &bbox));
        let copy = dir.path().join("cache/ldbv/dom20/691_5334-20260115.tif");
        assert_eq!(blocks_read(&copy), None, "the whole tile");
    }

    #[test]
    fn a_tile_gone_from_the_server_keeps_its_copy() {
        let server = MapFetcher::default();
        serve_tile(
            &server,
            Model::Dom20,
            WEST,
            dom20_tile(WEST, 400.0, None, "13.06.2025"),
            changed(),
        );
        let dir = tempfile::tempdir().unwrap();
        run_with(
            &server,
            dir.path(),
            FetchMode::Online,
            &area_in_west((40, 50), (0, 20)),
        )
        .unwrap();
        // The server has the tile no more; another area of it keeps what was read, and the
        // cells it lacks are empty, with a warning.
        let gone = MapFetcher::default();
        let (found, values) = run_with(
            &gone,
            dir.path(),
            FetchMode::Online,
            &area_in_west((40, 60), (0, 20)),
        )
        .unwrap();
        assert!(values.contains(&400.0));
        assert!(values.iter().any(|z| z.is_nan()));
        assert!(!found.warnings.is_empty());
    }

    #[test]
    fn an_html_page_instead_of_a_tile_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let tiles = dir.path().join("cache/ldbv/dgm1");
        std::fs::create_dir_all(&tiles).unwrap();
        let older = tiles.join("691_5334-20250101.tif");
        std::fs::write(&older, dgm1_tile(WEST, 500.0)).unwrap();
        let fetcher = MapFetcher::default();
        serve_tile(
            &fetcher,
            Model::Dgm1,
            WEST,
            b"<html><body>502 Bad Gateway</body></html>".to_vec(),
            changed(),
        );
        serve_tile(
            &fetcher,
            Model::Dgm1,
            EAST,
            dgm1_tile(EAST, 600.0),
            changed(),
        );
        let error = run(&fetcher, dir.path(), FetchMode::Online, Model::Dgm1, &BBOX).unwrap_err();
        assert!(error.contains("691_5334"), "{error}");
        assert!(error.contains("not a GeoTIFF"), "{error}");
        assert!(!tiles.join("691_5334-20260115.tif").exists());
        assert!(older.exists(), "the older tile stays");
    }

    #[test]
    fn the_backend_names_its_provider() {
        let backend = LdbvBackend::new();
        let info = backend.provider();
        assert_eq!(info.id.as_str(), "ldbv");
        assert_eq!(info.cache_dir.as_deref(), Some("ldbv"));
        assert_eq!(info.terrain_source, "ldbv-dgm1");
        assert_eq!(info.kind, RasterKind::PerArea);
    }

    #[test]
    fn the_tiles_lie_on_the_file_server_the_config_says() {
        let files = "https://x.example/f";
        assert_eq!(
            Model::Dgm1.tile_url(files, WEST),
            "https://x.example/f/dgm/dgm1/691_5334.tif"
        );
        assert_eq!(
            Model::Dom20.tile_url(files, WEST),
            "https://x.example/f/dom20/DOM/32691_5334_20_DOM.tif"
        );
    }
}
