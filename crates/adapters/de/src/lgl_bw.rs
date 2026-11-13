//! LGL Baden-Württemberg open data (dl-de/by-2-0, "Datenquelle: LGL, www.lgl-bw.de"), all in
//! EPSG:25832: the orthophoto from the WMS, dated by the flight tiles of a WFS; the terrain
//! model (DGM1, XYZ text) and the normalised surface model (nDOM1, GeoTIFF) out of the 2 km
//! archives of the file server, read tile by tile with range requests; buildings and land use
//! from the ALKIS WFS, whose views are not the simplified schema
//! (`docs/regions/baden-wuerttemberg-reference.md`).

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;
use serde_json::Value;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::PathBuf;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher};
use crate::note::note;
use crate::raster::{GridSpec, Raster};
use crate::sources::alkis_wfs::{WfsService, fetch_wfs_by};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::height_model::{HeightModel, centre_tile, names, whole_tile};
use crate::sources::region::{BADEN_WUERTTEMBERG, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::sources::zip_range::{Member, cached_member, find_member, read_member, refused};
use crate::tiles::{TileId, tiles_covering};
use crate::xyz_archive::{kept_directory, not_cached, xyz_tile};

/// The orthophoto WMS (20 cm, cut at the state border). Its capabilities promise 15 000 px;
/// from 6 144 px on it answers a white image with status 200, so the pieces stay at 4 000.
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C"),
    layer: Cow::Borrowed("IMAGES_DOP_20_RGB"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 4000,
};

/// The WFS with the flight of each 1 km tile of the orthophoto.
pub const FLIGHTS: &str =
    "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet";

/// The ALKIS WFS of the state: GeoServer views with names of their own (`gml_id`, `objektname`,
/// `geom`), not the simplified schema. [`LglBwAlkis`] turns its features into that schema.
pub const ALKIS: WfsService = WfsService {
    name: Cow::Borrowed("LGL ALKIS"),
    url: Cow::Borrowed("https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS"),
    buildings_type: Cow::Borrowed("nora:v_al_gebaeude"),
    land_use_type: Cow::Borrowed("nora:v_al_tatsaechliche_nutzung"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Baden-Württemberg)")),
    sorted: true,
};

/// The file server of the open data.
pub const FILES: &str = "https://opengeodata.lgl-bw.de/data";

/// The provider id of the adapter, and the name of its folder in the cache.
const PROVIDER: &str = "lglbw";

/// The state as the messages name it.
const STATE: &str = "Baden-Württemberg";

/// Whether `text` is a date written `YYYY-MM-DD`.
fn is_date(text: &str) -> bool {
    text.len() == 10
        && text.bytes().enumerate().all(|(at, byte)| match at {
            4 | 7 => byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

/// The flight date (`YYYY-MM-DD`) of the orthophoto tile at the centre of `bbox` (EPSG:25832),
/// from the WFS of the flight tiles; `None` when the service does not answer or has no date.
/// The WMS shows the flight this service names; the tiles of the file server can be a year
/// older.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        ((bbox.min_e + bbox.max_e) / 2.0).round(),
        ((bbox.min_n + bbox.max_n) / 2.0).round(),
    );
    let url = format!(
        "{FLIGHTS}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES=verm:v_dop_20_bildflugkacheln\
         &BBOX={},{},{},{},urn:ogc:def:crs:EPSG::25832&COUNT=10",
        e - 1.0,
        n - 1.0,
        e + 1.0,
        n + 1.0
    );
    let bytes = crate::fetch::get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    flight_of(&String::from_utf8_lossy(&bytes), centre_tile(bbox))
}

/// The `befliegungsdatum` of `tile` in an answer of the flight service, when it is a date. The
/// service names a tile `32` + easting + northing in km and pads its text fields with blanks.
fn flight_of(gml: &str, tile: TileId) -> Option<String> {
    let name = format!("32{:03}{:04}", tile.east_km, tile.north_km);
    gml.split("<wfs:member>")
        .skip(1)
        .find(|member| element(member, "dop_kachel") == Some(&name))
        .and_then(|member| element(member, "befliegungsdatum"))
        .filter(|date| is_date(date))
        .map(str::to_string)
}

/// The text of the first element `name` in `text`, trimmed, whatever its namespace prefix.
fn element<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let start = text.find(&format!(":{name}>"))? + name.len() + 2;
    Some(text[start..].split('<').next()?.trim())
}

/// A height model of the file server: 1 km tiles in one archive per 2 km block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain from the laser scan, as XYZ text.
    Dgm1,
    /// nDOM1, the height above the terrain of the surface matched from the aerial images of a
    /// summer flight (leaf-on; usually the flight of the orthophoto or the one before), as
    /// GeoTIFF.
    Ndom1,
}

impl Model {
    /// The folder of the file server that holds the archives of the model.
    fn folder(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm",
            Self::Ndom1 => "ndom1",
        }
    }

    /// Start of the archive and tile names, e.g. `dgm1_32_511_5403_1_bw_2023.xyz`.
    fn prefix(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Ndom1 => "ndom1",
        }
    }

    /// The archive of `block` ([`block_of`]).
    pub fn archive_url(self, block: TileId) -> String {
        format!(
            "{FILES}/{}/{}_32_{:03}_{:04}_2_bw.zip",
            self.folder(),
            self.prefix(),
            block.east_km,
            block.north_km
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LGL DGM1)",
            Self::Ndom1 => "surface model (LGL nDOM1)",
        }
    }

    /// The ending of the member that holds the heights of a tile.
    fn extension(self) -> &'static str {
        match self {
            Self::Dgm1 => ".xyz",
            Self::Ndom1 => ".tif",
        }
    }

    /// The column of the CSV of a tile with the date of the model: the last update of the
    /// terrain, the flight the surface was matched from.
    fn date_column(self) -> &'static str {
        match self {
            Self::Dgm1 => "Fortfuehrung",
            Self::Ndom1 => "bDOM20_Aktualitaet",
        }
    }

    /// The folder of the cache that holds the tiles and the directories of the model.
    fn cache_folder(self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(PROVIDER).join(self.prefix())
    }

    /// The start of the name of every cache file of `tile`; the checksum of the member the
    /// tile was read from follows.
    fn cache_name(self, tile: TileId) -> String {
        format!(
            "{}_32_{:03}_{:04}_",
            self.prefix(),
            tile.east_km,
            tile.north_km
        )
    }

    /// Whether the cache holds `tile` of this model, read from whichever member.
    pub fn is_cached(self, cache: &CacheDir, tile: TileId) -> bool {
        let start = self.cache_name(tile);
        std::fs::read_dir(self.cache_folder(cache)).is_ok_and(|files| {
            files.flatten().any(|file| {
                let name = file.file_name();
                let name = name.to_string_lossy();
                name.starts_with(&start) && name.ends_with(".tif")
            })
        })
    }

    /// The member of `tile` with the ending `extension` among `members`. The terrain members
    /// carry the year of their last update after the tile, so the name comes from the
    /// directory and is never built.
    fn member_of<'a>(
        self,
        members: &'a [Member],
        tile: TileId,
        extension: &str,
    ) -> Option<&'a Member> {
        let start = format!(
            "{}_32_{:03}_{:04}_1_bw",
            self.prefix(),
            tile.east_km,
            tile.north_km
        );
        find_member(members, |file| {
            file.starts_with(&start) && file.ends_with(extension)
        })
    }
}

/// The 2 km block whose archive holds the 1 km `tile`: named by its south-west tile, which has
/// an odd easting and an even northing.
pub fn block_of(tile: TileId) -> TileId {
    TileId {
        east_km: tile.east_km - (tile.east_km + 1) % 2,
        north_km: tile.north_km - tile.north_km % 2,
    }
}

/// The heights of a terrain tile from its XYZ member: one cell centre per line, `E N height`.
/// A tile at the state border holds only the rows inside the state; the cells without a line
/// stay empty. A point that is no cell centre of `tile` is an error.
fn parse_xyz(bytes: &[u8], tile: TileId) -> Result<Raster<f32>, String> {
    let grid = GridSpec {
        west: f64::from(tile.east_km) * 1000.0,
        north: f64::from(tile.north_km + 1) * 1000.0,
        pixel_size: 1.0,
        width: 1000,
        height: 1000,
    };
    let mut raster = Raster::filled(grid, f32::NAN);
    let mut points = 0_u32;
    for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        let text = String::from_utf8_lossy(line);
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let mut fields = text.split_ascii_whitespace().map(str::parse::<f64>);
        let (Some(Ok(e)), Some(Ok(n)), Some(Ok(z)), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            let excerpt: String = text.chars().take(60).collect();
            return Err(format!("line {} is not `E N height`: {excerpt}", index + 1));
        };
        let (col, row) = (e - grid.west - 0.5, grid.north - n - 0.5);
        if (col - col.round()).abs() > 0.01 || (row - row.round()).abs() > 0.01 {
            return Err(format!(
                "line {}: E {e:.2} N {n:.2} is no cell centre of the 1 m grid",
                index + 1
            ));
        }
        let (col, row) = (col.round(), row.round());
        if !(0.0..1000.0).contains(&col) || !(0.0..1000.0).contains(&row) {
            return Err(format!(
                "line {}: E {e:.2} N {n:.2} lies outside the tile",
                index + 1
            ));
        }
        let height = if z < -9998.0 { f32::NAN } else { z as f32 };
        raster.set(col as u32, row as u32, height);
        points += 1;
    }
    if points == 0 {
        return Err("no point in the file".to_string());
    }
    Ok(raster)
}

/// The date in the column `column` of the CSV of a tile (a header line, then one line), as
/// `YYYY-MM-DD`. Older files write the date with slashes.
fn csv_date(csv: &str, column: &str) -> Option<String> {
    let mut lines = csv.lines().filter(|line| !line.trim().is_empty());
    let index = lines
        .next()?
        .split(';')
        .position(|name| name.trim() == column)?;
    let date = lines
        .next()?
        .split(';')
        .nth(index)?
        .trim()
        .replace('/', "-");
    is_date(&date).then_some(date)
}

/// The members of the archive of `block`: online from the archive, and kept in the cache;
/// offline from the cache, and none when the directory was never read. The LGL publishes an
/// archive for every block with a tile in the state, so a 404 means no tile only for a block
/// none of whose tiles has its centre inside the state; otherwise the archive was renamed or
/// is being replaced, and the 404 is the error that names its address.
fn directory(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    block: TileId,
) -> Result<Vec<Member>, FetchError> {
    let file = model.cache_folder(cache).join(format!(
        "directory-{:03}_{:04}.bin",
        block.east_km, block.north_km
    ));
    match kept_directory(fetcher, mode, &model.archive_url(block), &file) {
        Err(FetchError::NotFound(_)) if !block_in_state(block) => Ok(Vec::new()),
        members => members,
    }
}

/// Whether one of the four 1 km tiles of the 2 km `block` has its centre inside the state.
fn block_in_state(block: TileId) -> bool {
    [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().any(|(e, n)| {
        tile_centre_in(
            TileId {
                east_km: block.east_km + e,
                north_km: block.north_km + n,
            },
            BADEN_WUERTTEMBERG,
        )
    })
}

/// The heights of `tile` from `member` of the archive of its block, kept in the cache as a
/// GeoTIFF named after the tile and the checksum of the member, so a tile the LGL replaces is
/// read anew. The name of the member comes from the server and never becomes a path.
fn tile_heights(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    tile: TileId,
    member: &Member,
) -> Result<Raster<f32>, FetchError> {
    let url = model.archive_url(block_of(tile));
    let file =
        model
            .cache_folder(cache)
            .join(format!("{}{:08x}.tif", model.cache_name(tile), member.crc));
    match model {
        Model::Ndom1 => {
            let offline = || not_cached(&url, member);
            let bytes = cached_member(fetcher, mode, &url, member, &file, offline)?;
            decode_float_tiff(&bytes)
                .and_then(|raster| whole_tile(raster, tile, &[1.0]))
                .map_err(|message| refused(&url, format!("{}: {message}", member.name)))
        }
        // The XYZ text of a tile has 29 MB; the cache keeps its heights as a GeoTIFF.
        Model::Dgm1 => xyz_tile(fetcher, mode, (&url, member), &file, tile, parse_xyz),
    }
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832) on its 1 m grid, read out of the
/// archives of their blocks and cached, and what the fetch found besides: the tiles inside the
/// state that no archive holds, and online the date of the model at the centre of the area. A
/// tile outside the state is in no archive and stays empty. No tile at all is an error, and so
/// is an archive that cannot be read.
fn mosaic(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
) -> Result<(Raster<f32>, HeightModel), FetchError> {
    let folder = format!("{FILES}/{}", model.folder());
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let mut found = HeightModel::default();
    let mut blocks: BTreeMap<TileId, Vec<Member>> = BTreeMap::new();
    let mut read = 0;
    for tile in tiles_covering(bbox) {
        let block = block_of(tile);
        let members = match blocks.entry(block) {
            Entry::Occupied(read) => read.into_mut(),
            Entry::Vacant(unread) => unread.insert(directory(fetcher, cache, mode, model, block)?),
        };
        let Some(member) = model.member_of(members, tile, model.extension()) else {
            if tile_centre_in(tile, BADEN_WUERTTEMBERG) {
                found.missing.push(tile);
            }
            continue;
        };
        let part = tile_heights(fetcher, cache, mode, model, tile, member)?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(&folder, e.to_string()))?;
        read += 1;
    }
    if read == 0 {
        return Err(refused(
            &folder,
            match mode {
                FetchMode::Offline => format!(
                    "offline and not cached: {}; run `freemapper fetch` online",
                    model.label()
                ),
                FetchMode::Online => {
                    format!("no tile of the {} covers the map area", model.label())
                }
            },
        ));
    }
    found.missing.sort();
    if mode == FetchMode::Online {
        found.date = date(fetcher, model, bbox, &blocks);
    }
    Ok((raster, found))
}

/// The date of `model` at the centre of `bbox`, from the CSV of the centre tile; from that of
/// the first other tile of the area that has one when the centre tile is in no archive.
fn date(
    fetcher: &dyn Fetcher,
    model: Model,
    bbox: &BoundingBox,
    blocks: &BTreeMap<TileId, Vec<Member>>,
) -> Option<String> {
    let centre = centre_tile(bbox);
    let others = tiles_covering(bbox)
        .into_iter()
        .filter(|tile| *tile != centre);
    let (block, member) = std::iter::once(centre).chain(others).find_map(|tile| {
        let block = block_of(tile);
        let member = model.member_of(blocks.get(&block)?, tile, ".csv")?;
        Some((block, member))
    })?;
    let bytes = read_member(fetcher, &model.archive_url(block), member).ok()?;
    csv_date(&String::from_utf8_lossy(&bytes), model.date_column())
}

/// The text of the attribute `name` of a feature of the views, when it has one.
fn text<'a>(feature: &'a Value, name: &str) -> Option<&'a str> {
    feature["properties"][name]
        .as_str()
        .filter(|text| !text.is_empty())
}

/// `feature` with the attributes `properties` of the simplified schema, named by its `oid`.
fn in_schema(feature: &Value, properties: Vec<(&str, Option<&str>)>) -> Value {
    let properties: serde_json::Map<String, Value> = properties
        .into_iter()
        .filter_map(|(name, value)| Some((name.to_string(), Value::from(value?))))
        .collect();
    serde_json::json!({
        "type": "Feature",
        "id": properties.get("oid").cloned().unwrap_or(Value::Null),
        "geometry": feature["geometry"],
        "properties": properties,
    })
}

/// The day of the timestamp `beginn` of a feature.
fn day(feature: &Value) -> Option<&str> {
    text(feature, "beginn")?.get(..10)
}

/// A building of the view `v_al_gebaeude` in the simplified schema. The view holds buildings
/// only (no building parts, no roofed areas), so `gebnutzbez` is "Gebäude"; its position to
/// the surface is the `rellage` that marks an underground building.
fn building(feature: Value) -> Value {
    in_schema(
        &feature,
        vec![
            ("oid", text(&feature, "gml_id")),
            ("gebnutzbez", Some("Gebäude")),
            ("funktion", text(&feature, "gebaeudefunktion_name")),
            ("rellage", text(&feature, "lage_oberflaeche_name")),
            ("name", text(&feature, "eigenname")),
            ("aktualit", day(&feature)),
        ],
    )
}

/// A land use area of the view `v_al_tatsaechliche_nutzung` in the simplified schema. The
/// view repeats the class as the name of the subtype when an area has none, so the subtype is
/// `bez` only with its id. A square without one gets the `square_bez` of the service: the state
/// does not record what most squares are used for, so they stay candidates.
fn land_use(feature: Value) -> Value {
    let class = text(&feature, "objektname");
    let subtype = text(&feature, "unterart_id")
        .and_then(|_| text(&feature, "unterart_name"))
        .or_else(|| {
            ALKIS
                .square_bez
                .as_deref()
                .filter(|_| class == Some("Platz"))
        });
    in_schema(
        &feature,
        vec![
            ("oid", text(&feature, "gml_id")),
            ("nutzart", class),
            ("bez", subtype),
            ("name", text(&feature, "eigenname")),
            ("aktualit", day(&feature)),
        ],
    )
}

/// The ALKIS of Baden-Württemberg: the views of the WFS of the LGL, paged by `gml_id` and
/// turned into the simplified schema.
pub struct LglBwAlkis;

impl AlkisSource for LglBwAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let bbox = area.bbox_for(ALKIS.crs);
        let features = fetch_wfs_by(env.fetcher, &ALKIS, &ALKIS.buildings_type, &bbox, "gml_id")?;
        Ok(AlkisPart {
            features: features.into_iter().map(building).collect(),
            crs: ALKIS.crs,
        })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let bbox = area.bbox_for(ALKIS.crs);
        let features = fetch_wfs_by(env.fetcher, &ALKIS, &ALKIS.land_use_type, &bbox, "gml_id")?;
        Ok(AlkisPart {
            features: features.into_iter().map(land_use).collect(),
            crs: ALKIS.crs,
        })
    }
}

/// The rasters of the LGL (Baden-Württemberg): requested for the area and stored in the project
/// folder.
pub struct LglBwBackend {
    info: ProviderInfo,
}

impl LglBwBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: "lgl-bw-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for LglBwBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for LglBwBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox)
    }
}

/// The LGL rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and nDOM1 read
/// out of the archives of the file server. The project stores a surface model, and the nDOM1
/// is a height above the ground, so the surface stored is the nDOM1 on the DGM1. It is only a
/// template: an unavailable one costs a warning, and a stored one stays while it covers the
/// area.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Result<(), AdapterError> {
    let (fetcher, mode, cache) = (env.fetcher, env.mode, env.cache);
    let root = project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    warn_beyond_state(env, project, PROVIDER, "LGL");
    let flight = || flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (&OrthophotoSource::Wms(WMS), "LGL", &flight, None),
    )?;
    let mut ground: Option<Raster<f32>> = None;
    fetch_area_heights(
        env,
        project,
        bbox,
        STATE,
        (Model::Dgm1.label(), Model::Ndom1.label()),
        &mut |surface, out, progress| {
            let model = if surface { Model::Ndom1 } else { Model::Dgm1 };
            let (mut raster, mut found) = mosaic(fetcher, cache, mode, model, bbox)?;
            match (&ground, surface) {
                (Some(ground), true) => {
                    for (height, ground) in raster.data.iter_mut().zip(&ground.data) {
                        *height += ground;
                    }
                }
                _ => ground = Some(raster.clone()),
            }
            write_float_raster(&raster, &root, out, &format!("{FILES}/{}", model.folder()))?;
            // Told here, with the state as it is written: the shared warning makes a key of
            // the name, and the catalogs have none with an umlaut.
            let missing = std::mem::take(&mut found.missing);
            if !missing.is_empty() {
                progress(note!(
                    "warning-model-tiles-missing",
                    model = crate::note::model(model.label()),
                    count = missing.len(),
                    names = names(&missing),
                    state = STATE
                ));
            }
            Ok(found)
        },
        &mut |_, _, _, _| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::Cache;
    use crate::fetch::test_support::{MapFetcher, geotiff, solid_jpeg};
    use crate::sources::alkis_wfs::parse_gml;
    use crate::sources::alkis_wfs::test_support::{BW_BUILDINGS, BW_LAND_USE};
    use crate::sources::zip_range::test_support::archive;
    use freemapper_adapter_api::records::{WcsRecord, stored_bbox};
    use freemapper_sources::raster_fetch::{SURFACE_RELPATH, TERRAIN_RELPATH};
    use std::path::Path;

    fn tile(east_km: u32, north_km: u32) -> TileId {
        TileId { east_km, north_km }
    }

    /// Stuttgart: a strip across the edge between the tiles 511 and 512 (both in the block
    /// `511_5402`), in their ten northern rows.
    fn stuttgart() -> BoundingBox {
        BoundingBox {
            min_e: 511_900.0,
            min_n: 5_403_990.0,
            max_e: 512_100.0,
            max_n: 5_404_000.0,
        }
    }

    /// The northern `rows` rows of a terrain tile as the LGL writes them: cell centres from the
    /// north-west, `E N height` with two decimals, CRLF. The height is `base` plus the column
    /// in centimetres.
    fn xyz(east_km: u32, north_km: u32, rows: u32, base: f32) -> Vec<u8> {
        let mut text = String::new();
        for row in 0..rows {
            for col in 0..1000 {
                text.push_str(&format!(
                    "{}.50 {}.50 {:.2}\r\n",
                    east_km * 1000 + col,
                    (north_km + 1) * 1000 - 1 - row,
                    base + col as f32 / 100.0
                ));
            }
        }
        text.into_bytes()
    }

    /// A whole nDOM tile of one height.
    fn ndom(east_km: u32, north_km: u32, value: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(east_km) * 1000.0,
            north: f64::from(north_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        geotiff(&grid, &vec![value; 1000 * 1000])
    }

    const DGM_CSV: &str = "Kachelname;Aktualitaet;Erfassungsmethode;Fortfuehrung;Fortfuehrungsmethode;Genauigkeit;Koordinatenreferenzsystem_Lage;Koordinatenreferenzsystem_Hoehe;Hoehenanomalie\r\n\
        dgm1_32_511_5403_1_bw_2023;2016-04-06;5020;2023-02-11;5020;0.15;ETRS89_UTM32;DE_DHHN2016_NH;DE_AdV_GCG2016_QGH\r\n";
    const NDOM_CSV: &str = "Kachel; DGM_Aktualitaet; DGM_Erfassungsmethode; DGM_Fortfuehrung; DGM_Fortfuehrungsmethode; DGM_Genauigkeit; DGM_Koordinatenreferenzsystem_Lage; DGM_Koordinatenreferenzsystem_Hoehe; DGM_Hoehenanomalie; bDOM20_Aktualitaet; bDOM20_Erfassungsmethode\r\n\
        ndom1_32_459_5482_1_bw;2016-01-27;5020;2022/02/14;5020;+/-15cm;ETRS89_UTM32;DE_DHHN2016_NH;DE_AdV_GC-20-6_QGH;2024/06/26;5040\r\n";

    /// The archive of the block of the school with the terrain of the tiles `tiles` (their ten
    /// northern rows), each with its CSV, as the file server holds it.
    fn serve_terrain(fetcher: &MapFetcher, tiles: &[(u32, u32, f32)]) {
        let mut members = Vec::new();
        for (e, n, base) in tiles {
            let name = format!("dgm1_32_511_5402_2_bw/dgm1_32_{e}_{n}_1_bw_2023");
            members.push((format!("{name}.csv"), DGM_CSV.as_bytes().to_vec()));
            members.push((format!("{name}.xyz"), xyz(*e, *n, 10, *base)));
        }
        members.push((
            "dgm1_32_511_5402_2_bw/Meta-ATKIS_DGM1_ALS2.txt".to_string(),
            b"https://metadaten.geoportal-bw.de".to_vec(),
        ));
        insert_archive(fetcher, &Model::Dgm1.archive_url(tile(511, 5402)), &members);
    }

    /// The archive of the block of the school with the nDOM of the tiles `tiles`.
    fn serve_surface(fetcher: &MapFetcher, tiles: &[(u32, u32, f32)]) {
        let mut members = Vec::new();
        for (e, n, value) in tiles {
            let name = format!("ndom1_32_511_5402_2_bw/ndom1_32_{e}_{n}_1_bw");
            let csv = NDOM_CSV.replace("2024/06/26", "2025-08-11");
            members.push((format!("{name}.csv"), csv.into_bytes()));
            members.push((format!("{name}.tif"), ndom(*e, *n, *value)));
        }
        insert_archive(
            fetcher,
            &Model::Ndom1.archive_url(tile(511, 5402)),
            &members,
        );
    }

    fn insert_archive(fetcher: &MapFetcher, url: &str, members: &[(String, Vec<u8>)]) {
        let members: Vec<(&str, Vec<u8>, bool)> = members
            .iter()
            .map(|(name, bytes)| (name.as_str(), bytes.clone(), true))
            .collect();
        fetcher.insert(url, archive(&members));
    }

    fn read(out: &Path) -> Raster<f32> {
        crate::derive::terrain::read_float_raster(out).unwrap()
    }

    #[test]
    fn a_tile_lies_in_the_archive_of_its_block() {
        // The block is named by its south-west tile: odd easting, even northing.
        assert_eq!(block_of(tile(511, 5403)), tile(511, 5402));
        assert_eq!(block_of(tile(512, 5402)), tile(511, 5402));
        assert_eq!(block_of(tile(413, 5315)), tile(413, 5314));
        assert_eq!(block_of(tile(572, 5359)), tile(571, 5358));
        assert_eq!(
            Model::Dgm1.archive_url(tile(511, 5402)),
            "https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_511_5402_2_bw.zip"
        );
        assert_eq!(
            Model::Ndom1.archive_url(tile(459, 5482)),
            "https://opengeodata.lgl-bw.de/data/ndom1/ndom1_32_459_5482_2_bw.zip"
        );
    }

    /// The flight date is that of the tile at the centre of the area. The service pads its
    /// text fields with blanks; an answer without the tile gives no date.
    #[test]
    fn the_flight_date_is_that_of_the_tile_at_the_centre() {
        let recorded = "<wfs:FeatureCollection numberMatched=\"2\"><wfs:member>\
            <verm:v_dop_20_bildflugkacheln gml:id=\"v_dop_20_bildflugkacheln.1\">\
            <verm:dop_kachel>325125403</verm:dop_kachel>\
            <verm:befliegungsdatum>2025-08-10</verm:befliegungsdatum>\
            <verm:kamera>UltraCam Eagle </verm:kamera></verm:v_dop_20_bildflugkacheln></wfs:member>\
            <wfs:member><verm:v_dop_20_bildflugkacheln gml:id=\"v_dop_20_bildflugkacheln.2\">\
            <verm:dop_kachel>325115403 </verm:dop_kachel>\
            <verm:befliegungsdatum>2026-05-24</verm:befliegungsdatum>\
            <verm:bearbeitungsjahr>2026</verm:bearbeitungsjahr>\
            </verm:v_dop_20_bildflugkacheln></wfs:member></wfs:FeatureCollection>";
        assert_eq!(
            flight_of(recorded, tile(511, 5403)).as_deref(),
            Some("2026-05-24")
        );
        assert_eq!(
            flight_of(recorded, tile(512, 5403)).as_deref(),
            Some("2025-08-10")
        );
        assert_eq!(flight_of(recorded, tile(513, 5403)), None);
        assert_eq!(
            flight_of(
                &recorded.replace("2026-05-24", "unbekannt"),
                tile(511, 5403)
            ),
            None
        );
        assert_eq!(
            flight_of("<html>Request Rejected</html>", tile(511, 5403)),
            None
        );
        // The request asks for the tiles around the centre of the area.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(FLIGHTS, recorded.as_bytes().to_vec());
        let school = BoundingBox {
            min_e: 511_780.0,
            min_n: 5_403_760.0,
            max_e: 511_920.0,
            max_n: 5_403_930.0,
        };
        assert_eq!(
            flight_date(&fetcher, &school).as_deref(),
            Some("2026-05-24")
        );
        assert_eq!(
            *fetcher.log.borrow(),
            [
                "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet\
                 ?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=verm:v_dop_20_bildflugkacheln\
                 &BBOX=511849,5403844,511851,5403846,urn:ogc:def:crs:EPSG::25832&COUNT=10"
            ]
        );
    }

    /// A terrain tile at the state border holds only the rows inside the state. The rows it
    /// does not hold stay empty; a point that is no cell centre of the tile is refused, and so
    /// is a page that is no list of points.
    #[test]
    fn a_terrain_tile_with_rows_missing_keeps_them_empty() {
        let raster = parse_xyz(&xyz(573, 5359, 3, 470.0), tile(573, 5359)).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (1000, 1000));
        assert_eq!(raster.grid.west, 573_000.0);
        assert_eq!(raster.grid.north, 5_360_000.0);
        assert_eq!(raster.get(0, 0), 470.0);
        assert_eq!(raster.get(999, 2), 479.99);
        assert!(raster.get(0, 3).is_nan());
        assert!(raster.get(999, 999).is_nan());
        // The rows may come in any order, and a tile may hold its southern rows only.
        let south = "573000.50 5359000.50 466.54\r\n573001.50 5359000.50 466.60\r\n";
        let raster = parse_xyz(south.as_bytes(), tile(573, 5359)).unwrap();
        assert_eq!(raster.get(0, 999), 466.54);
        assert_eq!(raster.get(1, 999), 466.6);
        assert!(raster.get(0, 0).is_nan());

        let off = parse_xyz(b"573000.25 5359000.50 466.54\r\n", tile(573, 5359)).unwrap_err();
        assert!(off.contains("no cell centre"), "{off}");
        let beyond = parse_xyz(b"574000.50 5359000.50 466.54\r\n", tile(573, 5359)).unwrap_err();
        assert!(beyond.contains("outside the tile"), "{beyond}");
        let page = parse_xyz(b"<html>Request Rejected</html>", tile(573, 5359)).unwrap_err();
        assert!(page.contains("not `E N height`"), "{page}");
        let empty = parse_xyz(b"\r\n", tile(573, 5359)).unwrap_err();
        assert!(empty.contains("no point"), "{empty}");
    }

    /// The dates of a tile are in its CSV, by the name of the column; older files write them
    /// with slashes.
    #[test]
    fn the_date_of_a_tile_is_read_in_both_spellings() {
        assert_eq!(
            csv_date(DGM_CSV, "Fortfuehrung").as_deref(),
            Some("2023-02-11")
        );
        assert_eq!(
            csv_date(DGM_CSV, "Aktualitaet").as_deref(),
            Some("2016-04-06")
        );
        // The header of the nDOM puts a blank after each semicolon.
        assert_eq!(
            csv_date(NDOM_CSV, "bDOM20_Aktualitaet").as_deref(),
            Some("2024-06-26")
        );
        assert_eq!(
            csv_date(NDOM_CSV, "DGM_Fortfuehrung").as_deref(),
            Some("2022-02-14")
        );
        assert_eq!(csv_date(NDOM_CSV, "DGM_Genauigkeit"), None);
        assert_eq!(csv_date(NDOM_CSV, "Fortfuehrung"), None);
        assert_eq!(
            csv_date("Kachelname;Fortfuehrung\r\n", "Fortfuehrung"),
            None
        );
    }

    /// A building of the views becomes one of the simplified schema: the id, the function, the
    /// date, and `rellage` from the position to the surface, which marks every underground
    /// garage. The views hold buildings only, so `gebnutzbez` is "Gebäude".
    #[test]
    fn a_building_of_the_views_becomes_one_of_the_schema() {
        let features: Vec<Value> = parse_gml(BW_BUILDINGS)
            .unwrap()
            .into_iter()
            .map(building)
            .collect();
        let garage = &features[0];
        assert_eq!(garage["id"], "DEBWL52210003w2f");
        assert_eq!(
            garage["properties"],
            serde_json::json!({
                "oid": "DEBWL52210003w2f",
                "gebnutzbez": "Gebäude",
                "funktion": "Tiefgarage",
                "rellage": "Unter der Erdoberfläche",
                "aktualit": "2002-07-30",
            })
        );
        assert_eq!(garage["geometry"]["type"], "MultiPolygon");
        let school = &features[1];
        assert_eq!(
            school["properties"]["funktion"],
            "Allgemein bildende Schule"
        );
        assert!(school["properties"].get("rellage").is_none());
        // A building with a name keeps it.
        let named = building(serde_json::json!({
            "type": "Feature",
            "id": null,
            "geometry": null,
            "properties": {
                "gml_id": "DEBWL5221000named",
                "gebaeudefunktion_name": "Museum",
                "eigenname": "Staatl. Kunsthalle",
                "beginn": "2018-02-16T07:36:27Z",
            },
        }));
        assert_eq!(named["properties"]["name"], "Staatl. Kunsthalle");
    }

    /// A land use area of the views becomes one of the simplified schema. The views repeat the
    /// class as the name of the subtype when an area has none, so `bez` is taken only from an
    /// area with a subtype id; a square without one stays a candidate.
    #[test]
    fn a_land_use_area_has_a_subtype_only_with_its_id() {
        let features: Vec<Value> = parse_gml(BW_LAND_USE)
            .unwrap()
            .into_iter()
            .map(land_use)
            .collect();
        assert_eq!(
            features[0]["properties"],
            serde_json::json!({
                "oid": "DEBWL52210003sx0",
                "nutzart": "Wohnbaufläche",
                "aktualit": "2018-02-16",
            })
        );
        assert_eq!(features[0]["id"], "DEBWL52210003sx0");
        assert_eq!(
            features[1]["properties"],
            serde_json::json!({
                "oid": "DEBWL52210003t5E",
                "nutzart": "Fläche besonderer funktionaler Prägung",
                "bez": "Öffentliche Zwecke",
                "aktualit": "2018-02-16",
            })
        );
        assert_eq!(features[1]["geometry"]["type"], "MultiPolygon");
        let square = land_use(serde_json::json!({
            "type": "Feature",
            "id": null,
            "geometry": null,
            "properties": {
                "gml_id": "DEBWL522100square",
                "objektname": "Platz",
                "unterart_name": "Platz",
                "eigenname": "Marktplatz",
            },
        }));
        assert_eq!(
            square["properties"],
            serde_json::json!({
                "oid": "DEBWL522100square",
                "nutzart": "Platz",
                "bez": "ohne Unterart (Baden-Württemberg)",
                "name": "Marktplatz",
            })
        );
        let car_park = land_use(serde_json::json!({
            "type": "Feature",
            "id": null,
            "geometry": null,
            "properties": {
                "gml_id": "DEBWL52210carpark",
                "objektname": "Platz",
                "unterart_id": "5310",
                "unterart_name": "Parkplatz",
            },
        }));
        assert_eq!(car_park["properties"]["bez"], "Parkplatz");
    }

    /// The source asks the WFS of the state sorted by `gml_id` and answers in the schema.
    #[test]
    fn the_alkis_of_the_state_is_asked_by_gml_id() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=nora:v_al_gebaeude",
            BW_BUILDINGS.as_bytes().to_vec(),
        );
        fetcher.insert_prefix(
            "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=nora:v_al_tatsaechliche_nutzung",
            BW_LAND_USE.as_bytes().to_vec(),
        );
        let cache = Cache::new(PathBuf::from("no-cache"));
        let registry = freemapper_adapter_api::registry::Registry::new();
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut |_| {},
        };
        let area = FetchArea::new(
            BoundingBox {
                min_e: 511_780.0,
                min_n: 5_403_760.0,
                max_e: 511_920.0,
                max_n: 5_403_930.0,
            },
            Crs::Utm32,
        );
        let buildings = LglBwAlkis.buildings(&mut env, &area).unwrap();
        assert_eq!(buildings.crs, Crs::Utm32);
        assert_eq!(buildings.features.len(), 3);
        assert_eq!(buildings.features[2]["properties"]["funktion"], "Wohnhaus");
        let land_use = LglBwAlkis.land_use(&mut env, &area).unwrap();
        assert_eq!(land_use.features.len(), 3);
        assert_eq!(
            land_use.features[2]["properties"]["bez"],
            "Handel und Dienstleistung"
        );
        let log = fetcher.log.borrow();
        assert!(
            log.iter().any(|url| url.contains("&SORTBY=gml_id&")),
            "{log:?}"
        );
        assert!(log.iter().all(|url| !url.contains("SORTBY=oid")), "{log:?}");
    }

    /// The tiles of an area are read out of the archives of their blocks: the terrain from XYZ
    /// members named with a year, with the date of the centre tile from its CSV.
    #[test]
    fn the_terrain_is_read_from_the_xyz_members_of_the_block() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher, &[(511, 5403, 300.0), (512, 5403, 400.0)]);
        let (raster, found) = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (200, 10));
        // Column 900 of the western tile, column 0 of the eastern one.
        assert_eq!(raster.get(0, 5), 309.0);
        assert_eq!(raster.get(100, 5), 400.0);
        assert!(found.missing.is_empty());
        assert_eq!(found.date.as_deref(), Some("2023-02-11"));
        // A second run reads the tiles from the cache: only the directory is asked again.
        let asked = fetcher.calls.get();
        let (again, _) = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap();
        assert_eq!(again, raster);
        // The end of the archive, its directory, and the CSV of the centre tile (two ranges).
        assert_eq!(fetcher.calls.get() - asked, 4);
        // Offline nothing is asked: directory and tiles come from the cache, without a date.
        let asked = fetcher.calls.get();
        let (offline, found) = mosaic(
            &fetcher,
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap();
        assert_eq!(offline, raster);
        assert_eq!(found.date, None);
        assert_eq!(fetcher.calls.get(), asked);
    }

    /// Offline, an area whose tiles were never fetched says so.
    #[test]
    fn offline_without_the_tiles_says_to_fetch_online() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let err = mosaic(
            &fetcher,
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("offline and not cached"), "{err}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    /// A tile the archive of its block does not hold is missing when it lies inside the state
    /// (some nDOM archives hold no tile where the terrain has one). The other tiles are read.
    #[test]
    fn a_tile_inside_the_state_that_no_archive_holds_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_surface(&fetcher, &[(511, 5403, 7.5)]);
        let (raster, found) = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Ndom1,
            &stuttgart(),
        )
        .unwrap();
        assert_eq!(raster.get(0, 5), 7.5);
        assert!(raster.get(150, 5).is_nan());
        assert_eq!(found.missing, [tile(512, 5403)]);
        assert_eq!(found.date.as_deref(), Some("2025-08-11"));
    }

    /// Across the Rhine at Mannheim the blocks hold only the tiles that touch the state, and a
    /// block wholly outside has no archive (404). Such tiles stay empty and are not missing.
    #[test]
    fn tiles_outside_the_state_stay_empty_without_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // Ludwigshafen (block 455_5480, no archive) and Mannheim (block 459_5480).
        let bbox = BoundingBox {
            min_e: 456_900.0,
            min_n: 5_480_990.0,
            max_e: 459_100.0,
            max_n: 5_481_000.0,
        };
        insert_archive(
            &fetcher,
            &Model::Ndom1.archive_url(tile(459, 5480)),
            &[(
                "ndom1_32_459_5480_2_bw/ndom1_32_459_5480_1_bw.tif".to_string(),
                ndom(459, 5480, 4.0),
            )],
        );
        // The block between them holds a tile of the Mannheim bank only.
        insert_archive(
            &fetcher,
            &Model::Ndom1.archive_url(tile(457, 5480)),
            &[(
                "ndom1_32_457_5480_2_bw/ndom1_32_458_5480_1_bw.tif".to_string(),
                ndom(458, 5480, 2.0),
            )],
        );
        assert!(!tile_centre_in(tile(456, 5480), BADEN_WUERTTEMBERG));
        assert!(!tile_centre_in(tile(457, 5480), BADEN_WUERTTEMBERG));
        let (raster, found) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Ndom1, &bbox).unwrap();
        assert!(raster.get(50, 5).is_nan());
        assert_eq!(raster.get(1500, 5), 2.0);
        assert_eq!(raster.get(2150, 5), 4.0);
        assert!(found.missing.is_empty(), "{:?}", found.missing);
    }

    /// A block that holds a tile with its centre inside the state has an archive. When the
    /// server answers 404 for it, the archive was renamed or is being replaced: an error that
    /// names its address, not a map with a hole where the state has data.
    #[test]
    fn a_missing_archive_of_a_block_inside_the_state_is_an_error_with_its_address() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // Stuttgart, across the edge between the blocks 511_5402 (served) and 513_5402 (404).
        serve_terrain(&fetcher, &[(512, 5403, 400.0)]);
        let bbox = BoundingBox {
            min_e: 512_900.0,
            min_n: 5_403_990.0,
            max_e: 513_100.0,
            max_n: 5_404_000.0,
        };
        assert!(tile_centre_in(tile(513, 5403), BADEN_WUERTTEMBERG));
        let err = mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bbox).unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err:?}");
        assert!(
            err.to_string()
                .contains("https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_513_5402_2_bw.zip"),
            "{err}"
        );
        // The same when every block of the area is missing.
        let fetcher = MapFetcher::default();
        let err = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_511_5402_2_bw.zip"),
            "{err}"
        );
    }

    /// A block none of whose tiles has its centre inside the state has no archive: its 404
    /// gives no tile and no error, and the terrain of the other blocks is read.
    #[test]
    fn a_missing_archive_of_a_block_beyond_the_state_holds_no_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // Ludwigshafen (block 455_5480, no archive) and the Mannheim bank (block 457_5480).
        let block = tile(455, 5480);
        for (e, n) in [(455, 5480), (456, 5480), (455, 5481), (456, 5481)] {
            assert_eq!(block_of(tile(e, n)), block);
            assert!(!tile_centre_in(tile(e, n), BADEN_WUERTTEMBERG), "{e}_{n}");
        }
        insert_archive(
            &fetcher,
            &Model::Dgm1.archive_url(tile(457, 5480)),
            &[(
                "dgm1_32_457_5480_2_bw/dgm1_32_458_5480_1_bw_2023.xyz".to_string(),
                xyz(458, 5480, 10, 95.0),
            )],
        );
        let bbox = BoundingBox {
            min_e: 456_900.0,
            min_n: 5_480_990.0,
            max_e: 458_100.0,
            max_n: 5_481_000.0,
        };
        let (raster, found) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bbox).unwrap();
        assert!(raster.get(50, 5).is_nan());
        assert_eq!(raster.get(1100, 5), 95.0);
        assert!(found.missing.is_empty(), "{:?}", found.missing);
    }

    /// No tile at all is an error that names the file server, not an empty raster.
    #[test]
    fn no_tile_at_all_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // The archive of the block is there, but holds no tile of the area.
        serve_terrain(&fetcher, &[]);
        let err = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("no tile of the terrain model (LGL DGM1)"),
            "{text}"
        );
        assert!(
            text.contains("https://opengeodata.lgl-bw.de/data/dgm"),
            "{text}"
        );
    }

    /// An archive the server does not deliver (not a 404) stops the fetch: a map with a hole
    /// where the state has data is no result.
    #[test]
    fn an_archive_that_fails_stops_the_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher, &[(511, 5403, 300.0), (512, 5403, 400.0)]);
        fetcher
            .failing
            .borrow_mut()
            .push(Model::Dgm1.archive_url(tile(511, 5402)));
        let err = mosaic(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &stuttgart(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("503"), "{err}");
    }

    /// A project folder with its raster records.
    struct TestProject {
        dir: PathBuf,
        tiles: Vec<freemapper_adapter_api::records::TileRecord>,
        ndsm_tiles: Vec<freemapper_adapter_api::records::NdsmRecord>,
        orthophoto: Option<freemapper_adapter_api::records::WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
    }

    impl TestProject {
        fn new(dir: PathBuf) -> Self {
            std::fs::create_dir_all(&dir).unwrap();
            Self {
                dir,
                tiles: Vec::new(),
                ndsm_tiles: Vec::new(),
                orthophoto: None,
                terrain: None,
                surface: None,
            }
        }
    }

    impl RasterProject for TestProject {
        fn records(&mut self) -> freemapper_adapter_api::project::RasterRecords<'_> {
            freemapper_adapter_api::project::RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            stuttgart()
        }

        fn crs(&self) -> Crs {
            Crs::Utm32
        }
    }

    fn serve_orthophoto(fetcher: &MapFetcher, bbox: &BoundingBox) {
        let photo = GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&WMS, &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [90, 110, 70]));
        }
    }

    /// The nDOM1 is a height above the ground. The project stores a surface model, so the
    /// backend puts the nDOM on the terrain; where the nDOM has no tile, the surface has no
    /// height. The records carry the dates of the centre tile, and the missing tile is told
    /// with the name of the state.
    #[test]
    fn the_surface_model_is_the_ndom_on_the_terrain() {
        let bbox = stuttgart();
        let scratch = tempfile::tempdir().unwrap();
        let cache = Cache::new(scratch.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher, &[(511, 5403, 300.0), (512, 5403, 400.0)]);
        serve_surface(&fetcher, &[(511, 5403, 7.5)]);
        serve_orthophoto(&fetcher, &bbox);
        fetcher.insert_prefix(
            FLIGHTS,
            b"<wfs:FeatureCollection><wfs:member><verm:v_dop_20_bildflugkacheln>\
              <verm:dop_kachel>325125403</verm:dop_kachel>\
              <verm:befliegungsdatum>2026-05-24</verm:befliegungsdatum>\
              </verm:v_dop_20_bildflugkacheln></wfs:member></wfs:FeatureCollection>"
                .to_vec(),
        );
        let mut project = TestProject::new(scratch.path().join("project"));
        let registry = freemapper_adapter_api::registry::Registry::new();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        fetch_rasters(&mut env, &mut project, &bbox).unwrap();
        let terrain = read(&project.dir.join(TERRAIN_RELPATH));
        let surface = read(&project.dir.join(SURFACE_RELPATH));
        assert_eq!(terrain.get(0, 5), 309.0);
        assert_eq!(surface.get(0, 5), 316.5);
        assert_eq!(surface.get(99, 5), 317.49);
        assert_eq!(terrain.get(150, 5), 400.5);
        assert!(surface.get(150, 5).is_nan());
        let record = |date: &str| {
            Some(WcsRecord {
                fetched: "2026-11-13".parse().unwrap(),
                bbox: stored_bbox(&bbox),
                date: Some(date.to_string()),
            })
        };
        assert_eq!(project.terrain, record("2023-02-11"));
        assert_eq!(project.surface, record("2025-08-11"));
        let orthophoto = project.orthophoto.as_ref().unwrap();
        // The centre of the area lies on the edge of the tiles; the eastern one holds it.
        assert_eq!(orthophoto.flight.as_deref(), Some("2026-05-24"));
        assert_eq!(orthophoto.flight_label, None);
        let missing: Vec<&String> = notes.iter().filter(|n| n.contains("has no tile")).collect();
        assert_eq!(
            missing,
            [
                "warning: the surface model (LGL nDOM1) has no tile 32_512_5403 inside \
              Baden-Württemberg; the templates stay empty there"
            ]
        );
    }

    /// A surface model that cannot be read costs a warning, as with every provider that reads
    /// the models for the area: a stored one that covers the area stays.
    #[test]
    fn a_surface_model_that_fails_keeps_a_covering_one() {
        let bbox = stuttgart();
        let scratch = tempfile::tempdir().unwrap();
        let cache = Cache::new(scratch.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_terrain(&fetcher, &[(511, 5403, 300.0), (512, 5403, 400.0)]);
        serve_orthophoto(&fetcher, &bbox);
        let mut project = TestProject::new(scratch.path().join("project"));
        let surface = project.dir.join(SURFACE_RELPATH);
        std::fs::create_dir_all(surface.parent().unwrap()).unwrap();
        std::fs::write(&surface, b"an earlier surface model").unwrap();
        project.surface = Some(WcsRecord {
            fetched: "2026-11-01".parse().unwrap(),
            bbox: stored_bbox(&bbox),
            date: None,
        });
        let registry = freemapper_adapter_api::registry::Registry::new();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        fetch_rasters(&mut env, &mut project, &bbox).unwrap();
        let warning = notes
            .iter()
            .find(|n| n.starts_with("warning: surface model (LGL nDOM1) unavailable"))
            .unwrap_or_else(|| panic!("{notes:?}"));
        assert!(
            warning.ends_with("kept the surface model fetched on 2026-11-01"),
            "{warning}"
        );
        assert!(project.surface.is_some());
        assert!(surface.exists());
        // No date from the flight service: the orthophoto is stored without one.
        assert_eq!(project.orthophoto.as_ref().unwrap().flight, None);
    }

    #[test]
    fn the_backend_names_its_provider() {
        let backend = LglBwBackend::new();
        let info = backend.provider();
        assert_eq!(info.id.as_str(), "lglbw");
        assert_eq!(info.cache_dir.as_deref(), Some("lglbw"));
        assert_eq!(info.terrain_source, "lgl-bw-dgm1");
        assert_eq!(info.kind, RasterKind::PerArea);
    }
}
