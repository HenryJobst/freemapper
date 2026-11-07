//! Project folder and project file (`freemapper.toml`).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use chrono::{Datelike as _, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::area::GeoBox;
use crate::area::{AreaError, MapArea};
use crate::build::buildings::{Rule, building_objects, classify};
use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::out_of_bounds::out_of_bounds_objects;
use crate::build::paved::{PavedObjects, paved_area_objects, unpaved_streets};
use crate::build::peripheral::{PaperRect, PeripheralInfo, info_block, north_lines};
use crate::crs::ProjectedPoint;
use crate::crs::{BoundingBox, LatLon, utm33_forward};
use crate::derive::contours::{
    contour_lines, contours_from_geojson, contours_to_geojson, gaussian_smooth, generalize,
};
use crate::derive::ndsm::{fill_from_dsm, rasterize, write_ndsm_png};
use crate::derive::orthophoto::build_orthophoto;
use crate::derive::terrain::{hillshade, mosaic_dtm, slope_deg, slope_shade, write_gray_png};
use crate::derive::{DeriveError, MAX_RASTER_SIDE_PX};
use crate::fetch::{
    Cache, FetchError, FetchMode, FetchedTile, Fetcher, fetch_tiles, write_atomically,
};
use crate::fetch::{fetch_ndsm_tiles, fetch_tiles_each};
use crate::georef::{GeorefError, Georeferencing};
use crate::ocd::convert::{Background, WorldFile, map_to_ocd};
use crate::omap::MapCoord;
use crate::omap::{
    MapDocument, MapObject, MapObjectKind, MapPart, OmapError, SymbolSet, TemplateKind,
    TemplateRef, XmlFormat, closed_ring, write_map,
};
use crate::raster::GridSpec;
use crate::sources::alkis::{LandUse, fetch_land_use, parse_land_use};
use crate::sources::alkis::{fetch_buildings, parse_buildings};
use crate::sources::lgb::{Product, TileCurrency, attribution, attribution_with_berlin};
use crate::sources::osm::{
    OsmFeature, features_from_geojson, features_to_geojson, fetch_extract, read_extract,
};
use crate::sources::region::{Region, raster_region, touches_berlin};
use crate::tiles::{TileId, tiles_covering};

pub const PROJECT_FILE_NAME: &str = "freemapper.toml";

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("project already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error("map already exists (it may contain your work); use --force to replace it: {0}")]
    MapExists(PathBuf),
    #[error("invalid scale 1:{0}")]
    InvalidScale(u32),
    #[error("invalid project file: {0}")]
    Toml(String),
    #[error(transparent)]
    Area(#[from] AreaError),
    #[error(transparent)]
    Georef(#[from] GeorefError),
    #[error(transparent)]
    Omap(#[from] OmapError),
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Derive(#[from] DeriveError),
    #[error("no tiles fetched yet; run `freemapper fetch` first")]
    NotFetched,
    #[error("{0}")]
    Layer(String),
    #[error("OCAD export: {0}")]
    Ocd(#[from] crate::ocd::file::OcdBuildError),
    #[error(
        "{}: image and world file do not match the map area; run `freemapper derive` again",
        .0.display()
    )]
    TemplateOutdated(PathBuf),
    #[error("the map area needs tiles {} that were not fetched; run `freemapper fetch`", tile_names(.0))]
    TilesOutdated(Vec<TileId>),
    #[error("no ALKIS buildings stored for this map area; run `freemapper fetch` online first")]
    BuildingsNotFetched,
    #[error("the stored OSM candidates do not cover the map area; run `freemapper fetch`")]
    CandidatesOutdated,
    #[error("{}: {message}; run `freemapper fetch` again", path.display())]
    StoredCandidates { path: PathBuf, message: String },
    #[error(transparent)]
    Osm(#[from] crate::sources::osm::OsmError),
    #[error(
        "map area plus buffer is {width_m:.0} × {height_m:.0} m, more than {MAX_AREA_SIDE_M:.0} m per side; reduce the map area or buffer_m in freemapper.toml"
    )]
    AreaTooLarge { width_m: f64, height_m: f64 },
    #[error(
        "the map area lies outside the supported states (Brandenburg, Berlin, Saxony); freemapper has data sources for those only so far"
    )]
    OutsideCoverage,
    #[error("{}: {message}; run `freemapper fetch` again", path.display())]
    StoredBuildings { path: PathBuf, message: String },
}

/// Kind of orienteering map; selects norm, symbol set and default scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapKind {
    Schoolyard,
    Sprint,
}

impl MapKind {
    pub fn default_scale(self) -> u32 {
        match self {
            Self::Schoolyard => 1000,
            Self::Sprint => 4000,
        }
    }

    pub fn default_buffer_m(self) -> f64 {
        match self {
            Self::Schoolyard => 20.0,
            Self::Sprint => 50.0,
        }
    }

    /// The map kind's symbol set plus the sheet symbols for the peripheral information.
    pub fn symbol_set(self) -> SymbolSet {
        match self {
            Self::Schoolyard => SymbolSet::school_2019(),
            Self::Sprint => SymbolSet::issprom_2019(),
        }
        .with_sheet_symbols()
    }

    /// Enlargement of the norm's symbols over ISSprOM 2019 (school set: 125 %).
    pub fn symbol_enlargement(self) -> f64 {
        match self {
            Self::Schoolyard => crate::omap::symbol_set::SCHOOL_SYMBOL_ENLARGEMENT,
            Self::Sprint => 1.0,
        }
    }

    /// Slope at which the slope template turns black: school grounds are flat, so their ramp is
    /// steeper to show small banks and ramps.
    pub fn slope_black_deg(self) -> f32 {
        match self {
            Self::Schoolyard => 15.0,
            Self::Sprint => 30.0,
        }
    }

    /// Gaussian smoothing of the DTM before tracing contours, in metres.
    pub fn contour_sigma_m(self) -> f64 {
        match self {
            Self::Schoolyard => 1.5,
            Self::Sprint => 2.0,
        }
    }

    /// Contour interval in metres (spec §4.4).
    pub fn contour_interval_m(self) -> u32 {
        match self {
            Self::Schoolyard => 1,
            Self::Sprint => 2,
        }
    }
}

/// Contents of `freemapper.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub map_kind: MapKind,
    pub scale_denominator: u32,
    pub declination_date: NaiveDate,
    pub buffer_m: f64,
    /// Identifier of the object the map area was taken from, e.g. `way/992423119`.
    pub map_area_source_id: Option<String>,
    /// Map area as GeoJSON geometry in EPSG:4326.
    pub map_area: serde_json::Value,
    /// Map title for the peripheral information; the folder name when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tiles used by the last `fetch`.
    #[serde(default)]
    pub tiles: Vec<TileRecord>,
    /// The ALKIS buildings of the last online `fetch`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buildings: Option<AlkisRecord>,
    /// The ALKIS land use of the last online `fetch`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub land_use: Option<AlkisRecord>,
    /// The OSM extract the candidates were read from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osm: Option<OsmRecord>,
    /// nDSM point clouds used by the last `fetch`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ndsm_tiles: Vec<NdsmRecord>,
}

/// One fetched nDSM tile; LGB publishes no currency for it, only the server date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NdsmRecord {
    pub tile: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<NaiveDate>,
}

/// Which OSM extract the stored candidates come from and the area they cover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmRecord {
    /// Date of the Geofabrik extract.
    pub extract: NaiveDate,
    /// Geographic box read: min latitude, min longitude, max latitude, max longitude.
    pub area: [f64; 4],
    pub count: usize,
}

impl OsmRecord {
    fn covers(&self, area: &GeoBox) -> bool {
        let [min_lat, min_lon, max_lat, max_lon] = self.area;
        min_lat <= area.min_lat
            && min_lon <= area.min_lon
            && max_lat >= area.max_lat
            && max_lon >= area.max_lon
    }
}

/// Which ALKIS buildings were fetched and how current they are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlkisRecord {
    /// Queried area in EPSG:25833: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
    pub fetched: NaiveDate,
    /// Newest `aktualit` among the buildings.
    pub latest_update: Option<NaiveDate>,
    pub count: usize,
    /// The states whose services delivered objects (missing in projects fetched before Berlin).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<Region>,
}

impl AlkisRecord {
    fn covers(&self, bbox: &BoundingBox) -> bool {
        let [min_e, min_n, max_e, max_n] = self.bbox;
        min_e <= bbox.min_e && min_n <= bbox.min_n && max_e >= bbox.max_e && max_n >= bbox.max_n
    }
}

/// One fetched tile: where it came from and its data currency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileRecord {
    pub product: Product,
    pub tile: String,
    pub url: String,
    pub currency: NaiveDate,
    pub published: NaiveDate,
}

impl ProjectFile {
    pub fn load(dir: &Path) -> Result<Self, ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path, e))?;
        toml::from_str(&text).map_err(|e| ProjectError::Toml(e.to_string()))
    }

    /// The title without control characters, or the folder name of `dir` with each
    /// `-`/`_`-separated word capitalized and joined by `-` (`clara-zetkin-schule` →
    /// `Clara-Zetkin-Schule`), or `Karte` when neither gives a text.
    pub fn display_title(&self, dir: &Path) -> String {
        if let Some(title) = self.title.as_deref().and_then(clean_title) {
            return title;
        }
        let name = folder_name(dir).unwrap_or_default();
        let title = name
            .split(['-', '_'])
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join("-");
        clean_title(&title).unwrap_or_else(|| DEFAULT_TITLE.to_string())
    }

    /// Map area plus buffer in EPSG:25833.
    pub fn bbox(&self) -> Result<BoundingBox, ProjectError> {
        let area = MapArea::from_geojson(&self.map_area.to_string())?;
        Ok(area.projected_bbox().buffered(self.buffer_m))
    }

    pub fn save(&self, dir: &Path) -> Result<(), ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = toml::to_string(self).map_err(|e| ProjectError::Toml(e.to_string()))?;
        Ok(write_atomically(&path, text.as_bytes())?)
    }
}

/// Creates the project folder `dir` with a project file for `area`. Refuses to overwrite an
/// existing project file.
pub fn init_project(
    dir: &Path,
    area: &MapArea,
    map_kind: MapKind,
    scale_denominator: Option<u32>,
    declination_date: NaiveDate,
    title: Option<String>,
) -> Result<ProjectFile, ProjectError> {
    let scale_denominator = scale_denominator.unwrap_or(map_kind.default_scale());
    if scale_denominator == 0 {
        return Err(ProjectError::InvalidScale(scale_denominator));
    }
    let path = dir.join(PROJECT_FILE_NAME);
    if path.exists() {
        return Err(ProjectError::AlreadyExists(path));
    }
    check_area(&area.projected_bbox().buffered(map_kind.default_buffer_m()))?;
    std::fs::create_dir_all(dir).map_err(|e| ProjectError::Io(dir.to_path_buf(), e))?;
    let project = ProjectFile {
        map_kind,
        scale_denominator,
        declination_date,
        buffer_m: map_kind.default_buffer_m(),
        map_area_source_id: area.source_id.clone(),
        map_area: area.to_geojson_value(),
        title: title.as_deref().and_then(clean_title),
        tiles: Vec::new(),
        buildings: None,
        land_use: None,
        osm: None,
        ndsm_tiles: Vec::new(),
    };
    project.save(dir)?;
    Ok(project)
}

use crate::fetch::tile_names;

/// Partial template files older than this are left over from interrupted runs.
const STALE_TEMPLATE_PART_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// Largest side of map area plus buffer: [`MAX_RASTER_SIDE_PX`] orthophoto pixels of 0.2 m.
pub const MAX_AREA_SIDE_M: f64 = MAX_RASTER_SIDE_PX as f64 * 0.2;

/// ISSprOM spacing of magnetic north lines on paper, before the norm's symbol enlargement.
const NORTH_LINE_SPACING_MM: f64 = 30.0;

/// Products fetched for every project, in this order.
const PRODUCTS: [Product; 2] = [Product::Orthophoto, Product::Dtm];
pub(crate) const ORTHOPHOTO_RELPATH: &str = "templates/orthophoto.jpg";
pub(crate) const HILLSHADE_RELPATH: &str = "templates/hillshade.png";
pub(crate) const SLOPE_RELPATH: &str = "templates/slope.png";
pub(crate) const NDSM_RELPATH: &str = "templates/ndsm.png";
/// Contour lines from `derive` (GeoJSON in EPSG:25833), input of the export.
const CONTOURS_RELPATH: &str = "data/contours.geojson";
/// OSM features matching the candidate rules (GeoJSON, EPSG:4326), input of the export.
const OSM_CANDIDATES_RELPATH: &str = "data/osm-candidates.geojson";
/// ALKIS land use per parcel part around the map area (GeoJSON, EPSG:25833), input of the export.
pub(crate) const LAND_USE_RELPATH: &str = "data/alkis-land-use.geojson";
/// Skipped ALKIS features listed in a progress message.
const SKIPPED_SHOWN: usize = 5;
/// ALKIS buildings around the map area, as fetched (GeoJSON, EPSG:25833).
pub(crate) const BUILDINGS_RELPATH: &str = "data/alkis-buildings.geojson";

/// Sets the map area of the project in `dir` (from the area selection page), keeping every other
/// setting. Data fetched for the old area is fetched again for the new one by the next `fetch`.
pub fn set_map_area(dir: &Path, area: &MapArea) -> Result<ProjectFile, ProjectError> {
    let mut project = ProjectFile::load(dir)?;
    project.map_area = area.to_geojson_value();
    project.map_area_source_id = area.source_id.clone();
    check_area(&project.bbox()?)?;
    project.save(dir)?;
    Ok(project)
}

/// Refuses a map area plus buffer outside every supported region or larger than the templates
/// can cover.
pub(crate) fn check_area(bbox: &BoundingBox) -> Result<(), ProjectError> {
    if raster_region(bbox).is_none() {
        return Err(ProjectError::OutsideCoverage);
    }
    if bbox.width() > MAX_AREA_SIDE_M || bbox.height() > MAX_AREA_SIDE_M {
        return Err(ProjectError::AreaTooLarge {
            width_m: bbox.width(),
            height_m: bbox.height(),
        });
    }
    Ok(())
}

/// Fetches the tiles covering the map area plus buffer into `cache` and records them in the
/// project file. Warns through `progress` when a tile was republished since the last fetch.
pub fn fetch_project(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<ProjectFile, ProjectError> {
    let mut project = ProjectFile::load(dir)?;
    let bbox = project.bbox()?;
    check_area(&bbox)?;
    let tiles = tiles_covering(&bbox);
    let mut records = Vec::new();
    for product in PRODUCTS {
        for fetched in fetch_tiles(fetcher, cache, product, &tiles, mode, progress)? {
            let record = TileRecord {
                product,
                tile: fetched.tile.name(),
                url: fetched.url,
                currency: fetched.currency.currency(),
                published: fetched.currency.published,
            };
            let previous = project
                .tiles
                .iter()
                .find(|r| r.product == product && r.tile == record.tile);
            if let Some(previous) = previous.filter(|p| p.published != record.published) {
                progress(&format!(
                    "warning: {product:?} tile {} changed upstream ({} → {})",
                    record.tile, previous.published, record.published
                ));
            }
            records.push(record);
        }
    }
    // Record the tiles first: they may have replaced older cached versions, and a failing
    // buildings download must not leave the project pointing at those.
    let previous_dsm: Vec<TileRecord> = project
        .tiles
        .iter()
        .filter(|r| r.product == Product::ImageDsm)
        .cloned()
        .collect();
    project.tiles = records;
    project.save(dir)?;
    let ndsm_tiles: Vec<NdsmRecord> = fetch_ndsm_tiles(fetcher, cache, &tiles, mode, progress)?
        .into_iter()
        .map(|t| NdsmRecord {
            tile: t.tile.name(),
            url: t.url,
            published: t.published,
        })
        .collect();
    for record in &ndsm_tiles {
        let previous = project.ndsm_tiles.iter().find(|p| p.tile == record.tile);
        if let Some(previous) = previous.filter(|p| p.published != record.published)
            && let (Some(old), Some(new)) = (previous.published, record.published)
        {
            progress(&format!(
                "warning: nDSM tile {} changed upstream ({old} → {new})",
                record.tile
            ));
        }
    }
    // Where no nDOM point cloud is published (Berlin), the nDSM comes from the bDOM surface
    // model minus the DTM. It is only a template: an unavailable bDOM costs a warning.
    let without_ndsm: Vec<TileId> = tiles
        .iter()
        .copied()
        .filter(|tile| !ndsm_tiles.iter().any(|r| r.tile == tile.name()))
        .collect();
    project.ndsm_tiles = ndsm_tiles;
    if !without_ndsm.is_empty() {
        match fetch_tiles_each(
            fetcher,
            cache,
            Product::ImageDsm,
            &without_ndsm,
            mode,
            progress,
        ) {
            Ok(results) => {
                for (tile, result) in results {
                    let fetched = match result {
                        Ok(fetched) => fetched,
                        Err(e) => {
                            progress(&format!(
                                "warning: surface model (bDOM) of tile {} unavailable ({e}); no nDSM template there",
                                tile.name()
                            ));
                            continue;
                        }
                    };
                    progress(&format!(
                        "nDSM of tile {} from the surface model (bDOM)",
                        tile.name()
                    ));
                    let record = TileRecord {
                        product: Product::ImageDsm,
                        tile: tile.name(),
                        url: fetched.url,
                        currency: fetched.currency.currency(),
                        published: fetched.currency.published,
                    };
                    if let Some(previous) = previous_dsm
                        .iter()
                        .find(|p| p.tile == record.tile && p.published != record.published)
                    {
                        progress(&format!(
                            "warning: {:?} tile {} changed upstream ({} → {})",
                            Product::ImageDsm,
                            record.tile,
                            previous.published,
                            record.published
                        ));
                    }
                    project.tiles.push(record);
                }
            }
            Err(e) => progress(&format!(
                "warning: surface model (bDOM) unavailable ({e}); no nDSM template for {}",
                crate::fetch::tile_names(&without_ndsm)
            )),
        }
    }
    project.save(dir)?;
    let bbox = project.bbox()?;
    let previous = project.buildings.as_ref();
    if let Some(record) = fetch_buildings_into(dir, fetcher, &bbox, mode, previous, progress)? {
        project.buildings = Some(record);
        project.save(dir)?;
    }
    if let Some(record) = fetch_land_use_into(
        dir,
        fetcher,
        &bbox,
        mode,
        project.land_use.as_ref(),
        progress,
    ) {
        project.land_use = record;
        project.save(dir)?;
    }
    project.osm = Some(fetch_osm_into(
        dir, fetcher, cache, mode, &project, progress,
    )?);
    project.save(dir)?;
    Ok(project)
}

/// Reads the candidate features of the map area plus buffer from the OSM extract into
/// [`OSM_CANDIDATES_RELPATH`]; offline from the cached extract.
fn fetch_osm_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    project: &ProjectFile,
    progress: &mut dyn FnMut(&str),
) -> Result<OsmRecord, ProjectError> {
    let area = MapArea::from_geojson(&project.map_area.to_string())?.geo_box(project.buffer_m);
    let extract = match fetch_extract(fetcher, cache, mode, progress) {
        // Offline without an extract, stored candidates that cover the area are still good.
        Err(FetchError::OsmExtractMissing)
            if dir.join(OSM_CANDIDATES_RELPATH).exists()
                && project.osm.as_ref().is_some_and(|r| r.covers(&area)) =>
        {
            let record = project.osm.clone().expect("checked above");
            progress(&format!(
                "warning: no OSM extract cached; keeping the stored OSM candidates from {}",
                record.extract
            ));
            return Ok(record);
        }
        other => other?,
    };
    progress(&format!("reading OSM extract from {}", extract.data_date));
    let rules = Rules::embedded();
    let features = read_extract(&extract.path, &area, &|tags| {
        rules.wants(tags, project.map_kind)
    })?;
    let path = dir.join(OSM_CANDIDATES_RELPATH);
    write_atomically(&path, features_to_geojson(&features).as_bytes())?;
    progress(&format!("{} OSM candidate features", features.len()));
    Ok(OsmRecord {
        extract: extract.data_date,
        area: [area.min_lat, area.min_lon, area.max_lat, area.max_lon],
        count: features.len(),
    })
}

/// Downloads the ALKIS buildings in `bbox` to [`BUILDINGS_RELPATH`] and returns the new record;
/// the stored file is only replaced by an answer that parses. Offline, the stored file must
/// exist and, if recorded, cover `bbox`; nothing new is recorded then.
fn fetch_buildings_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<AlkisRecord>, ProjectError> {
    let path = dir.join(BUILDINGS_RELPATH);
    if mode == FetchMode::Offline {
        if previous.is_none() && path.exists() {
            progress(
                "warning: ALKIS buildings coverage unknown; run `freemapper fetch` online to record it",
            );
        }
        let covered = previous.is_none_or(|record| record.covers(bbox));
        return if path.exists() && covered {
            Ok(None)
        } else {
            Err(ProjectError::BuildingsNotFetched)
        };
    }
    progress("downloading ALKIS buildings");
    let text = fetch_buildings(fetcher, bbox)?;
    let parsed = parse_buildings(&text).map_err(FetchError::from)?;
    write_atomically(&path, text.as_bytes())?;
    let count = parsed.buildings.len();
    progress(&format!("{count} ALKIS buildings"));
    if count == 0 {
        progress("warning: no ALKIS buildings found in the map area");
    }
    if !parsed.skipped.is_empty() {
        let shown = parsed.skipped.iter().take(SKIPPED_SHOWN).cloned();
        let more = parsed.skipped.len() > SKIPPED_SHOWN;
        progress(&format!(
            "skipped {} ALKIS features: {}{}",
            parsed.skipped.len(),
            shown.collect::<Vec<_>>().join("; "),
            if more { "; …" } else { "" }
        ));
    }
    let record = AlkisRecord {
        bbox: [
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil(),
        ],
        fetched: chrono::Local::now().date_naive(),
        latest_update: parsed.buildings.iter().filter_map(|b| b.updated).max(),
        count,
        regions: regions_of(parsed.buildings.iter().map(|b| b.oid.as_str())),
    };
    // Compare only the same area: a grown area brings in other buildings, not changed ones.
    if let Some(old) = previous.filter(|old| old.bbox == record.bbox) {
        let mut changes = Vec::new();
        if old.latest_update != record.latest_update
            && let (Some(a), Some(b)) = (old.latest_update, record.latest_update)
        {
            changes.push(format!("{a} → {b}"));
        }
        if old.count != record.count {
            changes.push(format!("{} → {} buildings", old.count, record.count));
        }
        if !changes.is_empty() {
            progress(&format!(
                "warning: ALKIS buildings changed upstream ({})",
                changes.join(", ")
            ));
        }
    }
    Ok(Some(record))
}

/// Downloads the ALKIS land use in `bbox` to [`LAND_USE_RELPATH`] and returns the new record
/// (`None`: leave the record as it is; `Some(None)`: drop it). Land use only gives candidates:
/// when the download fails, a stored file that still covers `bbox` stays, one that does not is
/// removed; each case is reported. Offline, nothing is downloaded.
fn fetch_land_use_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> Option<Option<AlkisRecord>> {
    let path = dir.join(LAND_USE_RELPATH);
    if mode == FetchMode::Offline {
        if previous.is_none() || !path.exists() {
            progress(
                "no ALKIS land use stored; run `freemapper fetch` online for out-of-bounds candidates",
            );
        }
        return None;
    }
    progress("downloading ALKIS land use");
    let downloaded = fetch_land_use(fetcher, bbox).and_then(|text| {
        let land_use = parse_land_use(&text).map_err(FetchError::from)?;
        write_atomically(&dir.join(LAND_USE_RELPATH), text.as_bytes())?;
        Ok(land_use)
    });
    match downloaded {
        Ok(land_use) => {
            progress(&format!("{} ALKIS land use parts", land_use.len()));
            Some(Some(AlkisRecord {
                bbox: [
                    bbox.min_e.floor(),
                    bbox.min_n.floor(),
                    bbox.max_e.ceil(),
                    bbox.max_n.ceil(),
                ],
                fetched: chrono::Local::now().date_naive(),
                latest_update: land_use.iter().filter_map(|l| l.updated).max(),
                count: land_use.len(),
                regions: regions_of(land_use.iter().map(|l| l.oid.as_str())),
            }))
        }
        Err(e) => match previous.filter(|_| path.exists()) {
            Some(record) if record.covers(bbox) => {
                progress(&format!(
                    "warning: ALKIS land use unavailable ({e}); keeping the stored land use"
                ));
                None
            }
            Some(_) => {
                progress(&format!(
                    "warning: ALKIS land use unavailable ({e}); the stored land use no longer covers the map area and was removed"
                ));
                let _ = std::fs::remove_file(&path);
                Some(None)
            }
            None => {
                progress(&format!(
                    "warning: ALKIS land use unavailable ({e}); no out-of-bounds candidates"
                ));
                None
            }
        },
    }
}

/// The stored ALKIS land use; none when it was not fetched or does not cover the map area plus
/// buffer (an optional source: `fetch` reports this, the export goes on without it).
fn stored_land_use(dir: &Path, project: &ProjectFile) -> Result<Vec<LandUse>, ProjectError> {
    let path = dir.join(LAND_USE_RELPATH);
    let Some(record) = project.land_use.as_ref().filter(|_| path.exists()) else {
        return Ok(Vec::new());
    };
    if !record.covers(&project.bbox()?) {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    parse_land_use(&text).map_err(|e| ProjectError::StoredCandidates {
        path: path.clone(),
        message: e.to_string(),
    })
}

/// 520 candidates for the private land around the map area; none without stored land use.
fn stored_out_of_bounds_objects(
    land_use: &[LandUse],
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    Ok(out_of_bounds_objects(
        land_use,
        &area.projected_polygon(),
        &project.bbox()?,
        project.map_kind,
        georef,
        symbol_set,
    )?)
}

/// 501.5 paved areas from the stored land use; sprint maps only (spec §4.5).
fn stored_paved_objects(
    land_use: &[LandUse],
    osm: Option<&(Vec<OsmFeature>, NaiveDate)>,
    project: &ProjectFile,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<PavedObjects, ProjectError> {
    if project.map_kind != MapKind::Sprint {
        return Ok(PavedObjects::default());
    }
    let streets = osm.map(|(features, _)| unpaved_streets(features));
    Ok(paved_area_objects(
        land_use,
        streets.as_deref().unwrap_or_default(),
        &project.bbox()?,
        georef,
        symbol_set,
    )?)
}

/// Writes orthophoto, hillshade and slope templates from the fetched tiles and returns the
/// image paths.
pub fn derive_project(
    dir: &Path,
    cache: &Cache,
    progress: &mut dyn FnMut(&str),
) -> Result<Vec<PathBuf>, ProjectError> {
    let project = ProjectFile::load(dir)?;
    if project.tiles.is_empty() {
        return Err(ProjectError::NotFetched);
    }
    let bbox = project.bbox()?;
    let needed = tiles_covering(&bbox);
    let outdated: Vec<TileId> = needed
        .iter()
        .copied()
        .filter(|tile| {
            PRODUCTS.iter().any(|product| {
                !project
                    .tiles
                    .iter()
                    .any(|r| r.product == *product && r.tile == tile.name())
            })
        })
        .collect();
    if !outdated.is_empty() {
        return Err(ProjectError::TilesOutdated(outdated));
    }
    let tiles = |product: Product| -> Result<Vec<FetchedTile>, ProjectError> {
        let mut fetched = Vec::new();
        let mut missing = Vec::new();
        for record in project.tiles.iter().filter(|r| r.product == product) {
            let tile = TileId::parse(&record.tile)
                .ok_or_else(|| ProjectError::Toml(format!("invalid tile id {:?}", record.tile)))?;
            let zip = cache.tile_path(product, tile, record.published);
            if !zip.exists() {
                missing.push(tile);
                continue;
            }
            fetched.push(FetchedTile {
                product,
                tile,
                url: record.url.clone(),
                currency: TileCurrency {
                    created: record.currency,
                    revised: None,
                    published: record.published,
                },
                zip,
            });
        }
        if !missing.is_empty() {
            return Err(FetchError::MissingOffline(product, missing).into());
        }
        Ok(fetched)
    };

    crate::fsutil::remove_stale_parts(&dir.join("templates"), STALE_TEMPLATE_PART_AGE);
    let orthophoto = dir.join(ORTHOPHOTO_RELPATH);
    progress(&format!("writing {ORTHOPHOTO_RELPATH}"));
    build_orthophoto(&tiles(Product::Orthophoto)?, &bbox, &orthophoto)?;

    let dtm = mosaic_dtm(&tiles(Product::Dtm)?, &bbox)?;
    let hillshade_path = dir.join(HILLSHADE_RELPATH);
    progress(&format!("writing {HILLSHADE_RELPATH}"));
    write_gray_png(&hillshade(&dtm), &hillshade_path)?;
    let slope_path = dir.join(SLOPE_RELPATH);
    progress(&format!("writing {SLOPE_RELPATH}"));
    let black_deg = project.map_kind.slope_black_deg();
    write_gray_png(&slope_shade(&slope_deg(&dtm), black_deg), &slope_path)?;

    progress(&format!("writing {CONTOURS_RELPATH}"));
    let smoothed = gaussian_smooth(&dtm, project.map_kind.contour_sigma_m());
    let interval = f64::from(project.map_kind.contour_interval_m());
    let lines = generalize(
        contour_lines(&smoothed, interval),
        project.scale_denominator,
    );
    let contours_path = dir.join(CONTOURS_RELPATH);
    write_atomically(&contours_path, contours_to_geojson(&lines).as_bytes())?;

    let mut written = vec![orthophoto, hillshade_path, slope_path];
    if project.buildings.as_ref().is_some_and(|b| b.count == 0) && touches_berlin(&bbox) {
        progress(
            "warning: no ALKIS buildings stored for this area in Berlin; run `freemapper fetch` to get them from the Geoportal Berlin",
        );
    }
    let dsm_recorded = project.tiles.iter().any(|r| r.product == Product::ImageDsm);
    if project.ndsm_tiles.is_empty() && !dsm_recorded {
        progress("no nDSM recorded; run `freemapper fetch` for the nDSM template");
    }
    let ndsm_files: Vec<PathBuf> = project
        .ndsm_tiles
        .iter()
        .filter_map(|record| {
            // The tile comes from the project file: only a valid tile id names a cache file.
            let Some(tile) = TileId::parse(&record.tile) else {
                progress(&format!(
                    "warning: invalid nDSM tile {:?} in freemapper.toml; skipped",
                    record.tile
                ));
                return None;
            };
            let path = cache.ndsm_path(tile, record.published);
            if !path.exists() {
                progress(&format!(
                    "warning: nDSM {} not cached; run `freemapper fetch`",
                    record.tile
                ));
                return None;
            }
            // A damaged cached file costs only its part of the template, not the whole derive.
            if let Err(e) = las::Reader::from_path(&path) {
                progress(&format!(
                    "warning: nDSM {} unreadable ({e}); delete {} and run `freemapper fetch`",
                    record.tile,
                    path.display()
                ));
                return None;
            }
            Some(path)
        })
        .collect();
    let ndsm_path = dir.join(NDSM_RELPATH);
    let dsm = if dsm_recorded {
        match tiles(Product::ImageDsm).and_then(|t| Ok(mosaic_dtm(&t, &bbox)?)) {
            Ok(dsm) => Some(dsm),
            Err(e) => {
                progress(&format!(
                    "warning: surface model (bDOM) unusable ({e}); run `freemapper fetch`"
                ));
                None
            }
        }
    } else {
        None
    };
    if ndsm_files.is_empty() && dsm.is_none() {
        // An older nDSM template would no longer match the map area: remove it.
        for stale in [ndsm_path.clone(), ndsm_path.with_extension("pgw")] {
            let _ = std::fs::remove_file(stale);
        }
    } else {
        progress(&format!("writing {NDSM_RELPATH}"));
        let mut ndsm = rasterize(&ndsm_files, &bbox)?;
        // Consumed here: the 0.2 m surface model is as large as the nDSM itself.
        if let Some(dsm) = dsm {
            fill_from_dsm(&mut ndsm, &dsm, &dtm);
        }
        write_ndsm_png(&ndsm, &ndsm_path)?;
        written.push(ndsm_path);
    }
    written.push(contours_path);
    Ok(written)
}

/// Path of the project's map file: `<dir>/<dir name>.omap`.
pub fn map_path(dir: &Path) -> PathBuf {
    let name = folder_name(dir).unwrap_or_else(|| "map".to_string());
    dir.join(format!("{name}.omap"))
}

/// Name of the folder `dir`, also for paths like `.` that have no file name of their own.
fn folder_name(dir: &Path) -> Option<String> {
    let name = match dir.file_name() {
        Some(name) => name.to_owned(),
        // `.` or `x/..`: resolve the path, but only then, so a symlinked folder keeps its name.
        None => std::fs::canonicalize(dir).ok()?.file_name()?.to_owned(),
    };
    name.to_str().map(str::to_string)
}

/// Title for the peripheral information when neither title nor folder name gives one.
const DEFAULT_TITLE: &str = "Karte";

/// `title` trimmed and without control characters; `None` when nothing is left.
fn clean_title(title: &str) -> Option<String> {
    let cleaned: String = title.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

/// What [`run_project`] did with the map file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapOutcome {
    Written(PathBuf),
    /// The map existed and `force` was not set, so it was left alone; the templates it
    /// references by path (images and candidates) were refreshed.
    Kept {
        map: PathBuf,
        references_images: bool,
        /// Buildings are stored for the project, but the kept map has none.
        missing_buildings: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub templates: Vec<PathBuf>,
    pub map: MapOutcome,
}

/// Fetch, derive and export. An existing map is kept unless `force` is set, because it may
/// contain the mapper's work.
pub fn run_project(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    force: bool,
    progress: &mut dyn FnMut(&str),
) -> Result<RunOutcome, ProjectError> {
    fetch_project(dir, fetcher, cache, mode, progress)?;
    let templates = derive_project(dir, cache, progress)?;
    let existing = map_path(dir);
    let map = if existing.exists() && !force {
        let text = std::fs::read_to_string(&existing)
            .map_err(|e| ProjectError::Io(existing.clone(), e))?;
        write_starter_project(dir, None)?;
        MapOutcome::Kept {
            references_images: text.contains(ORTHOPHOTO_RELPATH),
            missing_buildings: map_lacks_buildings(dir, &text)?,
            map: existing,
        }
    } else {
        MapOutcome::Written(export_starter_project(dir, force)?)
    };
    Ok(RunOutcome { templates, map })
}

/// Checks that a template image and its world file form the grid `derive` would write for
/// `bbox` now; a pair from different runs (crash between the two files, changed map area) fails.
fn check_template(image: &Path, bbox: &BoundingBox) -> Result<(), ProjectError> {
    let outdated = || ProjectError::TemplateOutdated(image.to_path_buf());
    let world_extension = match image.extension().and_then(|e| e.to_str()) {
        Some("jpg") => "jgw",
        _ => "pgw",
    };
    let world =
        std::fs::read_to_string(image.with_extension(world_extension)).map_err(|_| outdated())?;
    let (width, height) = image::image_dimensions(image).map_err(|_| outdated())?;
    let grid = GridSpec::from_world_file(&world, width, height).map_err(|_| outdated())?;
    let expected = GridSpec::covering(bbox, grid.pixel_size);
    let tolerance = grid.pixel_size / 100.0;
    let matches = (grid.width, grid.height) == (expected.width, expected.height)
        && (grid.west - expected.west).abs() < tolerance
        && (grid.north - expected.north).abs() < tolerance;
    if matches { Ok(()) } else { Err(outdated()) }
}

/// 521 objects for the stored ALKIS buildings; none when no buildings were fetched. Fails when the
/// recorded fetch does not cover the current map area plus buffer.
fn stored_building_objects(
    dir: &Path,
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let path = dir.join(BUILDINGS_RELPATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    if let Some(record) = &project.buildings
        && !record.covers(&project.bbox()?)
    {
        return Err(ProjectError::BuildingsNotFetched);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let buildings = parse_buildings(&text)
        .map_err(|e| ProjectError::StoredBuildings {
            path: path.clone(),
            message: e.to_string(),
        })?
        .buildings;
    Ok(building_objects(
        &buildings,
        &area.projected_polygon(),
        project.buffer_m,
        georef,
        symbol_set,
    )?)
}

/// The stored OSM features and the date of their extract; none when no features were stored.
/// Fails when the recorded read does not cover the current map area plus buffer.
fn stored_osm_features(
    dir: &Path,
    project: &ProjectFile,
    area: &MapArea,
) -> Result<Option<(Vec<OsmFeature>, NaiveDate)>, ProjectError> {
    let path = dir.join(OSM_CANDIDATES_RELPATH);
    let Some(record) = project.osm.as_ref().filter(|_| path.exists()) else {
        return Ok(None);
    };
    if !record.covers(&area.geo_box(project.buffer_m)) {
        return Err(ProjectError::CandidatesOutdated);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let features = features_from_geojson(&text).map_err(|e| ProjectError::StoredCandidates {
        path: path.clone(),
        message: e.to_string(),
    })?;
    Ok(Some((features, record.extract)))
}

/// Objects for the stored OSM features whose rule has `target`.
fn stored_osm_objects(
    target: Target,
    osm: Option<&(Vec<OsmFeature>, NaiveDate)>,
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let Some((features, extract)) = osm else {
        return Ok(Vec::new());
    };
    Ok(osm_objects(
        features,
        &Rules::embedded(),
        target,
        project.map_kind,
        &area.projected_polygon(),
        project.buffer_m,
        georef,
        symbol_set,
        *extract,
    )?)
}

/// 101/102 candidate objects for the contour lines from `derive`; none when there are none.
fn stored_contour_objects(
    dir: &Path,
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    use geo::{Contains as _, Distance as _, InteriorPoint as _};
    let path = dir.join(CONTOURS_RELPATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let lines = contours_from_geojson(&text).map_err(|e| ProjectError::StoredCandidates {
        path: path.clone(),
        message: e.to_string(),
    })?;
    let index_every = f64::from(5 * project.map_kind.contour_interval_m());
    let currency = project
        .tiles
        .iter()
        .filter(|t| t.product == Product::Dtm)
        .map(|t| t.currency)
        .max();
    let id = |code: &str| {
        symbol_set
            .id_for_code(code)
            .expect("symbol set defines contours")
    };
    let (contour, index) = (id(CONTOUR_CODE), id(INDEX_CONTOUR_CODE));
    let map_area = area.projected_polygon();
    // DGM1 is interpolated under buildings: closed rings there are artefacts.
    let footprints: Vec<geo_types::Polygon<f64>> = stored_buildings(dir)?
        .into_iter()
        .filter(|b| classify(b) == Rule::Object)
        .flat_map(|b| b.footprint.0)
        .collect();
    let on_a_building = |ring: &geo_types::LineString<f64>| {
        geo_types::Polygon::new(ring.clone(), vec![])
            .interior_point()
            .is_some_and(|p| footprints.iter().any(|f| f.contains(&p)))
    };
    lines
        .iter()
        .filter(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            let near = geo::Euclidean.distance(&ls, &map_area) <= project.buffer_m;
            near && !(ls.is_closed() && ls.0.len() > 3 && on_a_building(&ls))
        })
        .map(|line| {
            let coords: Vec<(i32, i32)> = line
                .points
                .iter()
                .map(|(e, n)| {
                    georef
                        .projected_to_map(ProjectedPoint {
                            easting: *e,
                            northing: *n,
                        })
                        .map(|c| (c.x, c.y))
                })
                .collect::<Result<_, _>>()?;
            let closed = coords.len() > 3 && coords.first() == coords.last();
            let mut tags = vec![
                ("freemapper:source".to_string(), "lgb-dgm1".to_string()),
                ("freemapper:height".to_string(), format!("{}", line.height)),
            ];
            if let Some(currency) = currency {
                tags.push(("freemapper:currency".to_string(), currency.to_string()));
            }
            Ok(MapObject {
                symbol_id: if (line.height / index_every).fract() == 0.0 {
                    index
                } else {
                    contour
                },
                kind: MapObjectKind::Path,
                coords: if closed {
                    closed_ring(&coords)
                } else {
                    coords
                        .into_iter()
                        .map(|(x, y)| MapCoord::new(x, y))
                        .collect()
                },
                tags,
            })
        })
        .collect()
}

/// ISSprOM contour and index contour.
const CONTOUR_CODE: &str = "101";
const INDEX_CONTOUR_CODE: &str = "102";

/// The stored ALKIS buildings; none when no buildings were fetched.
fn stored_buildings(dir: &Path) -> Result<Vec<crate::sources::alkis::Building>, ProjectError> {
    let path = dir.join(BUILDINGS_RELPATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    Ok(parse_buildings(&text)
        .map_err(|e| ProjectError::StoredBuildings {
            path: path.clone(),
            message: e.to_string(),
        })?
        .buildings)
}

/// Whether the stored buildings give map objects that the map at `map_text` lacks.
fn map_lacks_buildings(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if map_text.contains(ALKIS_SOURCE_TAG) {
        return Ok(false);
    }
    let project = ProjectFile::load(dir)?;
    let area = MapArea::from_geojson(&project.map_area.to_string())?;
    let georef = Georeferencing::new(
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set();
    Ok(!stored_building_objects(dir, &project, &area, &georef, &symbol_set)?.is_empty())
}

/// How a building object's provenance appears in the map file.
const ALKIS_SOURCE_TAG: &str = r#"<t k="freemapper:source">alkis</t>"#;

/// Writes the starter project (map, candidates template map, attribution) into `dir` and
/// returns the path of the map file.
///
/// An existing map file may already contain the mapper's work: it is only replaced when
/// `force` is set, and then kept as `<name>.omap.bak`.
pub fn export_starter_project(dir: &Path, force: bool) -> Result<PathBuf, ProjectError> {
    write_starter_project(dir, Some(force))
}

/// Writes the candidates template map and the attribution, and the map unless `map_force` is
/// `None` (the map is kept: it may contain the mapper's work). Candidates are a template that
/// always follows the current map area.
fn write_starter_project(dir: &Path, map_force: Option<bool>) -> Result<PathBuf, ProjectError> {
    let StarterDocuments {
        map,
        candidates,
        attribution_text,
    } = starter_documents(dir)?;
    let map_path = map_path(dir);
    if let Some(force) = map_force
        && map_path.exists()
    {
        if !force {
            return Err(ProjectError::MapExists(map_path));
        }
        let backup = map_path.with_extension("omap.bak");
        std::fs::copy(&map_path, &backup).map_err(|e| ProjectError::Io(backup, e))?;
    }
    write_document(&candidates, &dir.join(CANDIDATES_RELPATH))?;
    if map_force.is_some() {
        write_document(&map, &map_path)?;
    }
    let attribution = dir.join("ATTRIBUTION.txt");
    std::fs::write(
        &attribution,
        format!("{attribution_text}\n\n{FREEMAPPER_CREDIT}\n"),
    )
    .map_err(|e| ProjectError::Io(attribution, e))?;
    Ok(map_path)
}

/// Folder of the OCAD starting point inside a project.
pub const OCAD_DIR: &str = "ocad";

/// What [`export_ocad`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcadOutcome {
    pub map: PathBuf,
    /// The OCAD map existed and `force` was not set, so it was left alone (it may contain the
    /// mapper's work); candidates and images were refreshed.
    pub kept: bool,
    pub candidates: PathBuf,
    pub objects: usize,
    /// Objects OCAD cannot take (no symbol for OCAD, no area, outside its drawing area).
    pub skipped: usize,
    /// Degenerate line parts and hole rings left out of written objects.
    pub dropped_parts: usize,
}

/// The world file next to an image: `.jpg` → `.jgw`, `.png` → `.pgw`.
fn world_file_path(image: &Path) -> Option<PathBuf> {
    let ext = image.extension()?.to_str()?;
    let mut chars = ext.chars();
    let (first, last) = (chars.next()?, ext.chars().last()?);
    Some(image.with_extension(format!("{first}{last}w")))
}

/// Writes the OCAD starting point into `ocad/`: `<name>.ocd` (OCAD 12) with the map objects,
/// `<name>-candidates.ocd`, and the raster templates with their world files as background maps.
/// An existing `<name>.ocd` is only replaced when `force` is set (kept as `.ocd.bak`).
pub fn export_ocad(dir: &Path, force: bool) -> Result<OcadOutcome, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let documents = starter_documents(dir)?;
    let ocad_dir = dir.join(OCAD_DIR);
    std::fs::create_dir_all(&ocad_dir).map_err(|e| ProjectError::Io(ocad_dir.clone(), e))?;
    // OCAD finds background maps by the ASCII names in the parameter strings, so the files get
    // the same ASCII names ("Straußberg" → "Straussberg").
    let name = map_path(dir)
        .file_stem()
        .map(|s| {
            crate::ocd::symbols::ascii(&s.to_string_lossy())
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "map".to_string());
    let copy = |from: &Path, to: &Path| {
        std::fs::copy(from, to)
            .map(|_| ())
            .map_err(|e| ProjectError::Io(to.to_path_buf(), e))
    };
    let mut backgrounds = Vec::new();
    for template in &documents.map.templates {
        if template.kind != TemplateKind::Image {
            continue;
        }
        let image = dir.join(&template.relpath);
        let Some(world) = world_file_path(&image).filter(|w| w.exists() && image.exists()) else {
            continue;
        };
        let (Some(image_name), Some(world_name)) = (image.file_name(), world.file_name()) else {
            continue;
        };
        copy(&image, &ocad_dir.join(image_name))?;
        copy(&world, &ocad_dir.join(world_name))?;
        let text =
            std::fs::read_to_string(&world).map_err(|e| ProjectError::Io(world.clone(), e))?;
        let world_file = WorldFile::parse(&text)
            .ok_or_else(|| ProjectError::Layer(format!("{}: not a world file", world.display())))?;
        let (width_px, height_px) = image::image_dimensions(&image)
            .map_err(|e| ProjectError::Layer(format!("{}: {e}", image.display())))?;
        backgrounds.push(Background::Image {
            file_name: image_name.to_string_lossy().into_owned(),
            width_px,
            height_px,
            world: world_file,
            visible: template.visible,
            opacity: template.opacity,
        });
    }
    let candidates_name = format!("{name}-candidates.ocd");
    backgrounds.push(Background::Map {
        file_name: candidates_name.clone(),
        visible: true,
    });
    let candidates = ocad_dir.join(&candidates_name);
    let (bytes, _) = map_to_ocd(&documents.candidates, project.map_kind, &[])?;
    write_atomically(&candidates, &bytes)?;
    let map = ocad_dir.join(format!("{name}.ocd"));
    let kept = map.exists() && !force;
    let (bytes, report) = map_to_ocd(&documents.map, project.map_kind, &backgrounds)?;
    if !kept {
        if map.exists() {
            copy(&map, &map.with_extension("ocd.bak"))?;
        }
        write_atomically(&map, &bytes)?;
    }
    Ok(OcadOutcome {
        map,
        kept,
        candidates,
        objects: report.objects,
        skipped: report.skipped,
        dropped_parts: report.dropped_parts,
    })
}

/// The map part with the peripheral information (north lines, title, scale bar, credits).
const SHEET_PART: &str = "peripheral information";

/// The objects of the training map: everything of the map and every candidate except the map
/// area outline (the first candidate), and separately the sheet objects, which are not clipped.
fn training_objects(documents: &StarterDocuments) -> (Vec<&MapObject>, Vec<&MapObject>) {
    let (sheet, own): (Vec<&MapPart>, Vec<&MapPart>) = documents
        .map
        .parts
        .iter()
        .partition(|p| p.name == SHEET_PART);
    let map = own
        .iter()
        .flat_map(|p| &p.objects)
        .chain(
            documents
                .candidates
                .parts
                .iter()
                .flat_map(|p| p.objects.iter().skip(1)),
        )
        .collect();
    (map, sheet.iter().flat_map(|p| &p.objects).collect())
}

/// Writes `<name>.pdf`, a fully generated training map: the map objects and all candidates
/// drawn with the project's symbol set (this deliberately goes beyond "only reliable data as map
/// objects"), the peripheral information on the sheet.
pub fn export_training_pdf(dir: &Path) -> Result<PathBuf, ProjectError> {
    use crate::render::{draw, pdf, symbols::RenderSymbols};
    let project = ProjectFile::load(dir)?;
    let documents = starter_documents(dir)?;
    let symbols = RenderSymbols::from_set(&documents.map.symbol_set);
    let (map, sheet) = training_objects(&documents);
    let render = |objects: &[&MapObject]| -> Vec<draw::Primitive> {
        objects
            .iter()
            .flat_map(|o| draw::primitives(o, &symbols))
            .collect()
    };
    let content = PaperRect::of_bbox(&project.bbox()?, &documents.map.georef)?;
    let clip = pdf::PaperBox {
        min: (
            f64::from(content.min_x) / 1000.0,
            f64::from(content.min_y) / 1000.0,
        ),
        max: (
            f64::from(content.max_x) / 1000.0,
            f64::from(content.max_y) / 1000.0,
        ),
    };
    let bytes = pdf::write_pdf(
        &render(&map),
        &render(&sheet),
        clip,
        &symbols.colours,
        &project.display_title(dir),
    );
    let path = map_path(dir).with_extension("pdf");
    write_atomically(&path, &bytes)?;
    Ok(path)
}

/// The map and the candidates template map of a project, as written to OOM and OCAD files.
struct StarterDocuments {
    map: MapDocument,
    candidates: MapDocument,
    attribution_text: String,
}

fn starter_documents(dir: &Path) -> Result<StarterDocuments, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let area = MapArea::from_geojson(&project.map_area.to_string())?;
    let georef = Georeferencing::new(
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set();
    let attribution_text = attribution_lines(&project).join("\n");
    let notes = format!("{attribution_text}\n{FREEMAPPER_CREDIT}");

    // Buildings first: they are the map's own objects, candidates only a template.
    let building_objects = stored_building_objects(dir, &project, &area, &georef, &symbol_set)?;
    let land_use = stored_land_use(dir, &project)?;
    let osm = stored_osm_features(dir, &project, &area)?;
    let paved = stored_paved_objects(&land_use, osm.as_ref(), &project, &georef, &symbol_set)?;
    let candidates = MapDocument {
        notes: notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts: vec![MapPart {
            name: "candidates".to_string(),
            objects: std::iter::once(map_area_outline(&area, &georef, &symbol_set, &project)?)
                .chain(stored_osm_objects(
                    Target::Candidates,
                    osm.as_ref(),
                    &project,
                    &area,
                    &georef,
                    &symbol_set,
                )?)
                .chain(stored_contour_objects(
                    dir,
                    &project,
                    &area,
                    &georef,
                    &symbol_set,
                )?)
                .chain(paved.candidates)
                .chain(stored_out_of_bounds_objects(
                    &land_use,
                    &project,
                    &area,
                    &georef,
                    &symbol_set,
                )?)
                .collect(),
        }],
        templates: Vec::new(),
    };
    let crs_spec = format!("+init=epsg:{}", georef.epsg);
    let mut templates: Vec<TemplateRef> = [
        (ORTHOPHOTO_RELPATH, true, 1.0),
        (HILLSHADE_RELPATH, false, 0.5),
        (SLOPE_RELPATH, false, 0.5),
        (NDSM_RELPATH, false, 0.6),
    ]
    .into_iter()
    .filter(|(relpath, _, _)| dir.join(relpath).exists())
    .map(|(relpath, visible, opacity)| {
        check_template(&dir.join(relpath), &project.bbox()?)?;
        Ok(TemplateRef {
            kind: TemplateKind::Image,
            relpath: relpath.to_string(),
            georeferenced: true,
            crs_spec: Some(crs_spec.clone()),
            visible,
            opacity,
            behind_map: true,
        })
    })
    .collect::<Result<_, ProjectError>>()?;
    templates.push(TemplateRef {
        kind: TemplateKind::Map,
        relpath: CANDIDATES_RELPATH.to_string(),
        georeferenced: true,
        crs_spec: None,
        visible: true,
        opacity: 0.7,
        behind_map: false,
    });
    let mut map_objects = building_objects;
    map_objects.extend(paved.map);
    map_objects.extend(stored_osm_objects(
        Target::Map,
        osm.as_ref(),
        &project,
        &area,
        &georef,
        &symbol_set,
    )?);
    let buildings_part = MapPart {
        name: "default part".to_string(),
        objects: map_objects,
    };
    let content = PaperRect::of_bbox(&project.bbox()?, &georef)?;
    let mut attribution = vec![OSM_ATTRIBUTION_DE.to_string()];
    attribution.extend(lgb_attribution(&project));
    attribution.extend(berlin_attribution(&project));
    attribution.push(FREEMAPPER_CREDIT_DE.to_string());
    let info = PeripheralInfo {
        title: project.display_title(dir),
        scale_denominator: project.scale_denominator,
        contour_interval_m: project.map_kind.contour_interval_m(),
        attribution,
    };
    let spacing_mm = NORTH_LINE_SPACING_MM * project.map_kind.symbol_enlargement();
    let mut sheet_objects = north_lines(&content, spacing_mm, &symbol_set);
    sheet_objects.extend(info_block(&content, &info, &symbol_set));
    let parts = vec![
        buildings_part,
        MapPart {
            name: SHEET_PART.to_string(),
            objects: sheet_objects,
        },
    ];
    let map = MapDocument {
        notes,
        georef,
        symbol_set,
        parts,
        templates,
    };

    Ok(StarterDocuments {
        map,
        candidates,
        attribution_text,
    })
}

pub(crate) const CANDIDATES_RELPATH: &str = "templates/candidates.omap";
const OSM_ATTRIBUTION: &str = "© OpenStreetMap contributors, ODbL";
/// Geoportal Berlin data needs no attribution (dl-de/zero-2-0); it is named anyway.
const BERLIN_ATTRIBUTION: &str =
    "© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert";
/// [`OSM_ATTRIBUTION`] for the German peripheral information.
const OSM_ATTRIBUTION_DE: &str = "© OpenStreetMap-Mitwirkende, ODbL";
/// [`FREEMAPPER_CREDIT`] for the German peripheral information.
const FREEMAPPER_CREDIT_DE: &str =
    concat!("Erstellt mit freemapper – ", env!("CARGO_PKG_REPOSITORY"));
/// Names the tool and where to get it, in every generated project.
const FREEMAPPER_CREDIT: &str = concat!("Created with freemapper – ", env!("CARGO_PKG_REPOSITORY"));

/// Attribution of every source the project uses: OSM, LGB once tiles were fetched, and the
/// Geoportal Berlin.
fn attribution_lines(project: &ProjectFile) -> Vec<String> {
    let mut lines = vec![OSM_ATTRIBUTION.to_string()];
    lines.extend(lgb_attribution(project));
    lines.extend(berlin_attribution(project));
    lines
}

/// The Geoportal Berlin ALKIS line, once Berlin ALKIS objects were fetched.
fn berlin_attribution(project: &ProjectFile) -> Option<String> {
    [&project.buildings, &project.land_use]
        .into_iter()
        .flatten()
        .any(|record| record.regions.contains(&Region::Berlin))
        .then(|| BERLIN_ATTRIBUTION.to_string())
}

/// The states of the given ALKIS object ids, Brandenburg first.
fn regions_of<'a>(oids: impl Iterator<Item = &'a str>) -> Vec<Region> {
    let found: Vec<Region> = oids.filter_map(Region::of_object_id).collect();
    [Region::Brandenburg, Region::Berlin]
        .into_iter()
        .filter(|r| found.contains(r))
        .collect()
}

/// The LGB attribution line, once tiles were fetched. LGB tiles in Berlin hold Berlin data, and
/// LGB's licence note then asks for the Geoportal Berlin as well.
fn lgb_attribution(project: &ProjectFile) -> Option<String> {
    let year = project.tiles.iter().map(|t| t.published.year()).max()?;
    let in_berlin = project.bbox().is_ok_and(|bbox| touches_berlin(&bbox));
    Some(if in_berlin {
        attribution_with_berlin(year)
    } else {
        attribution(year)
    })
}
/// Line symbol used for the map area outline in the candidates template map.
const OUTLINE_SYMBOL_CODE: &str = "501.1";

/// The exterior ring of the map area as a closed path object.
fn map_area_outline(
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    project: &ProjectFile,
) -> Result<MapObject, GeorefError> {
    let points = area
        .polygon
        .exterior()
        .points()
        .map(|p| {
            let c = georef.projected_to_map(utm33_forward(LatLon {
                lat: p.y(),
                lon: p.x(),
            }))?;
            Ok((c.x, c.y))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut tags = vec![("freemapper:source".to_string(), "osm".to_string())];
    if let Some(id) = &project.map_area_source_id {
        tags.push(("freemapper:source_id".to_string(), id.clone()));
    }
    Ok(MapObject {
        symbol_id: symbol_set
            .id_for_code(OUTLINE_SYMBOL_CODE)
            .expect("symbol set defines 501.1"),
        kind: MapObjectKind::Path,
        coords: closed_ring(&points),
        tags,
    })
}

fn write_document(doc: &MapDocument, path: &Path) -> Result<(), ProjectError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ProjectError::Io(parent.to_path_buf(), e))?;
    }
    let file = std::fs::File::create(path).map_err(|e| ProjectError::Io(path.to_path_buf(), e))?;
    let mut writer = std::io::BufWriter::new(file);
    write_map(doc, &mut writer, XmlFormat::Compact)?;
    writer
        .flush()
        .map_err(|e| ProjectError::Io(path.to_path_buf(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_map_area_keeps_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Sprint,
            None,
            date,
            Some("Vorstadt".into()),
        )
        .unwrap();
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/strausberg-vorstadt-sprint.geojson"
        ))
        .unwrap();
        let area = MapArea::from_geojson(&text).unwrap();
        let project = set_map_area(dir.path(), &area).unwrap();
        let loaded = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project, loaded);
        assert_eq!(loaded.map_area, area.to_geojson_value());
        assert_eq!(loaded.map_area_source_id, None);
        assert_eq!(loaded.title.as_deref(), Some("Vorstadt"));
        assert_eq!(loaded.map_kind, MapKind::Sprint);
        assert_eq!(loaded.declination_date, date);
    }

    #[test]
    fn init_refuses_too_large_area() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("huge");
        let huge = MapArea::from_geojson(
            r#"{"type":"Polygon","coordinates":[[[13.7,52.5],[13.9,52.5],[13.9,52.6],[13.7,52.6],[13.7,52.5]]]}"#,
        )
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let err = init_project(&project, &huge, MapKind::Sprint, None, date, None).unwrap_err();
        assert!(matches!(err, ProjectError::AreaTooLarge { .. }), "{err:?}");
        assert!(!project.exists());
    }

    #[test]
    fn init_refuses_area_outside_lgb_data() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("munich");
        let munich = MapArea::from_geojson(r#"{"type":"Polygon","coordinates":[[[11.57,48.13],[11.58,48.13],[11.58,48.14],[11.57,48.14],[11.57,48.13]]]}"#).unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let err = init_project(&project, &munich, MapKind::Sprint, None, date, None).unwrap_err();
        assert!(matches!(err, ProjectError::OutsideCoverage), "{err:?}");
        assert!(err.to_string().contains("Brandenburg"), "{err}");
        assert!(err.to_string().contains("Saxony"), "{err}");
        assert!(!project.exists());
    }

    #[test]
    fn init_accepts_saxony_and_its_east_edge() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        // Dresden, and Görlitz north (east of E 500 km, outside the old LGB box).
        for (name, lon, lat) in [("dresden", 13.818, 51.043), ("goerlitz", 15.0, 51.22)] {
            let area = MapArea::from_geojson(&format!(
                r#"{{"type":"Polygon","coordinates":[[[{lon},{lat}],[{},{lat}],[{},{}],[{lon},{}],[{lon},{lat}]]]}}"#,
                lon + 0.002,
                lon + 0.002,
                lat + 0.002,
                lat + 0.002
            ))
            .unwrap();
            init_project(
                &dir.path().join(name),
                &area,
                MapKind::Sprint,
                None,
                date,
                None,
            )
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    #[test]
    fn set_map_area_refuses_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Sprint,
            None,
            date,
            None,
        )
        .unwrap();
        let before = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        let huge = MapArea::from_geojson(
            r#"{"type":"Polygon","coordinates":[[[13.7,52.5],[13.9,52.5],[13.9,52.6],[13.7,52.6],[13.7,52.5]]]}"#,
        )
        .unwrap();
        let err = set_map_area(dir.path(), &huge).unwrap_err();
        assert!(matches!(err, ProjectError::AreaTooLarge { .. }), "{err:?}");
        let after = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert_eq!(before, after);
        assert!(set_map_area(tempfile::tempdir().unwrap().path(), &fixture_area()).is_err());
    }
    use crate::fetch::test_support::{MapFetcher, serve_synthetic_tile};

    #[test]
    fn map_kind_defaults() {
        assert_eq!(MapKind::Schoolyard.default_scale(), 1000);
        assert_eq!(MapKind::Sprint.default_scale(), 4000);
        assert_eq!(MapKind::Schoolyard.default_buffer_m(), 20.0);
        assert_eq!(MapKind::Sprint.default_buffer_m(), 50.0);
        assert_eq!(MapKind::Schoolyard.symbol_set().symbol_count(), 195);
        assert_eq!(MapKind::Sprint.symbol_set().symbol_count(), 189);
    }

    #[test]
    fn slope_black_threshold_per_map_kind() {
        assert_eq!(MapKind::Schoolyard.slope_black_deg(), 15.0);
        assert_eq!(MapKind::Sprint.slope_black_deg(), 30.0);
    }

    #[test]
    fn map_kind_sheet_values() {
        assert_eq!(MapKind::Schoolyard.symbol_enlargement(), 1.25);
        assert_eq!(MapKind::Sprint.symbol_enlargement(), 1.0);
        assert_eq!(MapKind::Schoolyard.contour_interval_m(), 1);
        assert_eq!(MapKind::Sprint.contour_interval_m(), 2);
        assert_eq!(
            MapKind::Schoolyard.symbol_set().id_for_code("991"),
            Some(191)
        );
        assert_eq!(MapKind::Sprint.symbol_set().id_for_code("991"), Some(185));
    }

    #[test]
    fn project_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let file = ProjectFile {
            map_kind: MapKind::Schoolyard,
            scale_denominator: 1000,
            declination_date: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            buffer_m: 20.0,
            map_area_source_id: Some("way/992423119".to_string()),
            map_area: serde_json::json!({
                "type": "Polygon",
                "coordinates": [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]
            }),
            title: Some("Clara-Zetkin-Schule".to_string()),
            tiles: vec![TileRecord {
                product: Product::Dtm,
                tile: "33421-5821".to_string(),
                url: "https://example.org/dgm.zip".to_string(),
                currency: NaiveDate::from_ymd_opt(2023, 5, 4).unwrap(),
                published: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            }],
            buildings: Some(AlkisRecord {
                bbox: [421_490.0, 5_821_580.0, 421_737.0, 5_821_897.0],
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                latest_update: NaiveDate::from_ymd_opt(2023, 5, 4),
                count: 18,
                regions: vec![Region::Brandenburg],
            }),
            land_use: Some(AlkisRecord {
                bbox: [421_490.0, 5_821_580.0, 421_737.0, 5_821_897.0],
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                latest_update: NaiveDate::from_ymd_opt(2025, 9, 30),
                count: 24,
                regions: Vec::new(),
            }),
            osm: Some(OsmRecord {
                extract: NaiveDate::from_ymd_opt(2026, 11, 6).unwrap(),
                area: [52.5385, 13.8418, 52.5418, 13.8457],
                count: 7,
            }),
            ndsm_tiles: vec![NdsmRecord {
                tile: "33421-5821".to_string(),
                url: "https://example.org/ndom.laz".to_string(),
                published: NaiveDate::from_ymd_opt(2025, 3, 3),
            }],
        };
        file.save(dir.path()).unwrap();
        assert!(dir.path().join(PROJECT_FILE_NAME).exists());
        assert_eq!(ProjectFile::load(dir.path()).unwrap(), file);
    }

    fn fixture_area() -> MapArea {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/clara-zetkin-schule.geojson"
        ))
        .unwrap();
        MapArea::from_geojson(&text).unwrap()
    }

    const SCHOOL_TILE: &str = "33421-5821";

    fn school_tile() -> crate::tiles::TileId {
        crate::tiles::TileId::parse(SCHOOL_TILE).unwrap()
    }

    fn school_project(dir: &Path) {
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(dir, &fixture_area(), MapKind::Schoolyard, None, date, None).unwrap();
    }

    fn fetch(dir: &Path, fetcher: &MapFetcher, cache: &Cache) -> (ProjectFile, Vec<String>) {
        let mut messages = Vec::new();
        let project = fetch_project(dir, fetcher, cache, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        (project, messages)
    }

    #[test]
    fn project_file_without_tiles_loads() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(PROJECT_FILE_NAME),
            r#"map_kind = "schoolyard"
scale_denominator = 1000
declination_date = "2026-11-07"
buffer_m = 20.0
map_area_source_id = "way/992423119"

[map_area]
type = "Polygon"
coordinates = [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]
"#,
        )
        .unwrap();
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.tiles, []);
        assert_eq!(project.title, None);
        assert_eq!(project.buildings, None);
    }

    #[test]
    fn fetch_records_tiles() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");

        fetch(dir.path(), &fetcher, &cache);
        let tiles = ProjectFile::load(dir.path()).unwrap().tiles;
        assert_eq!(tiles.len(), 2);
        for (record, product) in tiles.iter().zip([Product::Orthophoto, Product::Dtm]) {
            assert_eq!(record.product, product);
            assert_eq!(record.tile, SCHOOL_TILE);
            assert_eq!(record.url, product.tile_url(school_tile()));
            assert_eq!(record.currency.to_string(), "2018-11-27");
            assert_eq!(record.published.to_string(), "2026-01-26");
        }
    }

    #[test]
    fn fetch_warns_on_upstream_change() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let (_, first) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !first.iter().any(|m| m.contains("changed upstream")),
            "{first:?}"
        );

        serve_synthetic_tile(&fetcher, school_tile(), "2026-06-01");
        let (_, second) = fetch(dir.path(), &fetcher, &cache);
        let warnings: Vec<_> = second
            .iter()
            .filter(|m| m.contains("changed upstream"))
            .collect();
        assert_eq!(warnings.len(), 2, "{second:?}");
        assert!(
            warnings[0].contains("2026-01-26 → 2026-06-01"),
            "{warnings:?}"
        );
    }

    #[test]
    fn derive_writes_three_templates() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetch(dir.path(), &fetcher, &cache);

        let written = derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let templates = dir.path().join("templates");
        assert_eq!(
            written,
            [
                templates.join("orthophoto.jpg"),
                templates.join("hillshade.png"),
                templates.join("slope.png"),
                dir.path().join("data/contours.geojson")
            ]
        );
        for file in ["orthophoto.jgw", "hillshade.pgw", "slope.pgw"] {
            assert!(templates.join(file).exists(), "{file} missing");
        }
    }

    fn templates_of(map: &str) -> Vec<String> {
        map.match_indices("<template type=")
            .map(|(i, _)| {
                let rel = i + map[i..].find("relpath=\"").unwrap() + "relpath=\"".len();
                map[rel..rel + map[rel..].find('"').unwrap()].to_string()
            })
            .collect()
    }

    #[test]
    fn export_without_rasters_has_only_candidates() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let map_path = export_starter_project(dir.path(), false).unwrap();
        let map = std::fs::read_to_string(map_path).unwrap();
        assert_eq!(templates_of(&map), ["templates/candidates.omap"]);
        assert!(map.contains(r#"first_front_template="0""#));
        assert!(!map.contains("GeoBasis-DE/LGB"));
    }

    #[test]
    fn export_after_derive_references_images() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("school");
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(&project_dir);
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetch(&project_dir, &fetcher, &cache);
        derive_project(&project_dir, &cache, &mut |_| {}).unwrap();

        let map_path = export_starter_project(&project_dir, false).unwrap();
        let map = std::fs::read_to_string(map_path).unwrap();
        assert_eq!(
            templates_of(&map),
            [
                "templates/orthophoto.jpg",
                "templates/hillshade.png",
                "templates/slope.png",
                "templates/candidates.omap"
            ]
        );
        assert!(map.contains(r#"first_front_template="3""#), "{map}");
        assert!(map.contains(r#"type="TemplateImage""#));
        let lgb = "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert";
        assert!(map.contains(lgb));
        let attribution = std::fs::read_to_string(project_dir.join("ATTRIBUTION.txt")).unwrap();
        assert!(
            attribution.contains(lgb) && attribution.contains("OpenStreetMap"),
            "{attribution}"
        );
    }

    fn run(dir: &Path, fetcher: &MapFetcher, cache: &Cache, force: bool) -> RunOutcome {
        run_project(dir, fetcher, cache, FetchMode::Online, force, &mut |_| {}).unwrap()
    }

    #[test]
    fn run_writes_map_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");

        let outcome = run(dir.path(), &fetcher, &cache, false);
        assert_eq!(outcome.templates.len(), 4, "three images and the contours");
        assert_eq!(outcome.map, MapOutcome::Written(map_path(dir.path())));
    }

    /// The fetched and derived school project.
    fn derived_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir) {
        let (dir, cache_dir, cache) = fetched_school(fetcher);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        (dir, cache_dir)
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn training_pdf_draws_map_and_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let pdf_path = export_training_pdf(dir.path()).unwrap();
        assert_eq!(pdf_path.extension().unwrap(), "pdf");
        let file = std::fs::read(&pdf_path).unwrap();
        assert!(file.starts_with(b"%PDF-"));
        let pdf = crate::render::pdf::content_of(&file);
        // Building fill (black 60 %), tartan pitch candidate (lower brown 50 %), contours (brown).
        assert!(contains(&pdf, b"0 0 0 0.6 k"), "building");
        assert!(contains(&pdf, b"0 0.28 0.5 0.09 k"), "pitch");
        assert!(contains(&pdf, b"0 0.56 1 0.18 K"), "contours");
        // Texts are WinAnsi hex strings in the content stream.
        let credit: String = "github.com/HenryJobst/freemapper"
            .bytes()
            .map(|b| format!("{b:02X}"))
            .collect();
        assert!(
            contains(&pdf, credit.as_bytes()),
            "credit line on the sheet"
        );
    }

    #[test]
    fn training_map_takes_all_but_the_outline() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let documents = starter_documents(dir.path()).unwrap();
        let (map, sheet) = training_objects(&documents);
        let outline = &documents.candidates.parts[0].objects[0];
        assert!(!map.iter().any(|o| std::ptr::eq(*o, outline)));
        let candidates = documents.candidates.parts[0].objects.len() - 1;
        let own = documents
            .map
            .parts
            .iter()
            .filter(|p| p.name != SHEET_PART)
            .map(|p| p.objects.len())
            .sum::<usize>();
        assert_eq!(map.len(), own + candidates);
        assert!(!sheet.is_empty());
    }

    #[test]
    fn training_pdf_without_data_still_has_the_sheet() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let file = std::fs::read(export_training_pdf(dir.path()).unwrap()).unwrap();
        assert!(contains(&file, b"/MediaBox"));
        assert!(
            contains(&crate::render::pdf::content_of(&file), b"Tj"),
            "sheet texts"
        );
    }

    #[test]
    fn export_ocad_writes_map_candidates_and_images() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let outcome = export_ocad(dir.path(), false).unwrap();
        let name = map_path(dir.path())
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let ocad = dir.path().join("ocad");
        assert_eq!(outcome.map, ocad.join(format!("{name}.ocd")));
        assert!(!outcome.kept);
        for file in [
            "orthophoto.jpg",
            "orthophoto.jgw",
            "hillshade.png",
            "hillshade.pgw",
            "slope.png",
        ] {
            assert!(ocad.join(file).exists(), "{file}");
        }
        let map = crate::ocd::read::read(&std::fs::read(&outcome.map).unwrap());
        assert!(map.objects.iter().any(|o| o.symbol == 521_000), "building");
        let backgrounds: Vec<&str> = map
            .strings
            .iter()
            .filter(|s| s.kind == 8)
            .map(|s| s.text.split('\t').next().unwrap())
            .collect();
        assert!(backgrounds.contains(&"orthophoto.jpg"), "{backgrounds:?}");
        assert!(backgrounds.contains(&format!("{name}-candidates.ocd").as_str()));
        let notes = &map.strings.iter().find(|s| s.kind == 1061).unwrap().text;
        assert!(
            notes.contains("github.com/HenryJobst/freemapper"),
            "{notes}"
        );
        let candidates = crate::ocd::read::read(&std::fs::read(&outcome.candidates).unwrap());
        assert!(!candidates.objects.is_empty());
        assert!(outcome.objects > 0);
    }

    #[test]
    fn export_ocad_names_files_in_ascii() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("Straußberg Schule");
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(&dir, &fixture_area(), MapKind::Schoolyard, None, date, None).unwrap();
        let outcome = export_ocad(&dir, false).unwrap();
        assert_eq!(outcome.map.file_name().unwrap(), "Straussberg Schule.ocd");
        assert_eq!(
            outcome.candidates.file_name().unwrap(),
            "Straussberg Schule-candidates.ocd"
        );
        let map = crate::ocd::read::read(&std::fs::read(&outcome.map).unwrap());
        let background = map.strings.iter().find(|s| s.kind == 8).unwrap();
        assert_eq!(background.text, "Straussberg Schule-candidates.ocd\ts1\tr1");
    }

    #[test]
    fn export_ocad_keeps_an_edited_ocd() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let first = export_ocad(dir.path(), false).unwrap();
        std::fs::write(&first.map, "edited in OCAD").unwrap();
        let kept = export_ocad(dir.path(), false).unwrap();
        assert!(kept.kept);
        assert_eq!(
            std::fs::read_to_string(&kept.map).unwrap(),
            "edited in OCAD"
        );
        assert!(kept.candidates.exists());
        let replaced = export_ocad(dir.path(), true).unwrap();
        assert!(!replaced.kept);
        let backup = replaced.map.with_extension("ocd.bak");
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "edited in OCAD");
        assert_eq!(&std::fs::read(&replaced.map).unwrap()[0..2], &[0xAD, 0x0C]);
    }

    #[test]
    fn run_keeps_the_map_but_rewrites_the_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let map = export_starter_project(dir.path(), false).unwrap();
        std::fs::write(&map, "edited in OOM").unwrap();
        let candidates = dir.path().join(CANDIDATES_RELPATH);
        std::fs::write(&candidates, "candidates of the old area").unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        run(dir.path(), &fetcher, &cache, false);
        assert_eq!(std::fs::read_to_string(&map).unwrap(), "edited in OOM");
        let text = std::fs::read_to_string(&candidates).unwrap();
        assert!(
            text.contains("way/992423119"),
            "outline of the current area"
        );
        assert!(text.contains("way/1028507308"), "OSM candidates");
    }

    #[test]
    fn run_keeps_existing_map_and_refreshes_templates() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let map = export_starter_project(dir.path(), false).unwrap();
        std::fs::write(&map, "edited in OOM").unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");

        let outcome = run(dir.path(), &fetcher, &cache, false);
        assert_eq!(
            outcome.map,
            MapOutcome::Kept {
                map: map.clone(),
                references_images: false,
                missing_buildings: true
            }
        );
        assert_eq!(std::fs::read_to_string(&map).unwrap(), "edited in OOM");
        assert!(dir.path().join(ORTHOPHOTO_RELPATH).exists());

        let replaced = run(dir.path(), &fetcher, &cache, true);
        assert_eq!(replaced.map, MapOutcome::Written(map.clone()));
        let again = run(dir.path(), &fetcher, &cache, false);
        assert_eq!(
            again.map,
            MapOutcome::Kept {
                map,
                references_images: true,
                missing_buildings: false
            }
        );
    }

    #[test]
    fn run_does_not_hint_when_there_are_no_buildings_to_add() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(&[], None),
        );
        run(dir.path(), &fetcher, &cache, false);
        let again = run(dir.path(), &fetcher, &cache, false);
        assert!(
            matches!(
                again.map,
                MapOutcome::Kept {
                    missing_buildings: false,
                    ..
                }
            ),
            "{:?}",
            again.map
        );
    }

    #[test]
    fn export_refuses_buildings_that_do_not_cover_the_area() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m += 40.0;
        project.save(dir.path()).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let err = export_starter_project(dir.path(), false).unwrap_err();
        assert!(matches!(err, ProjectError::BuildingsNotFetched), "{err:?}");
    }

    #[test]
    fn oversized_area_is_refused_before_downloading() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m = 2_500.0;
        project.save(dir.path()).unwrap();
        let fetcher = MapFetcher::default();
        let err = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err();
        assert!(matches!(err, ProjectError::AreaTooLarge { .. }), "{err:?}");
        assert!(err.to_string().contains("buffer_m"), "{err}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    fn fetched_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        serve_synthetic_tile(fetcher, school_tile(), "2026-01-26");
        fetch(dir.path(), fetcher, &cache);
        (dir, cache_dir, cache)
    }

    /// A schoolyard project around the Helene-Haeusler-Schule, Berlin-Prenzlauer Berg (tile
    /// 33392-5820), served with LGB rasters and the Berlin ALKIS fixtures.
    fn berlin_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(392_750.0, 5_820_800.0),
            corner(392_850.0, 5_820_800.0),
            corner(392_850.0, 5_820_900.0),
            corner(392_750.0, 5_820_900.0),
            corner(392_750.0, 5_820_800.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        let tile = crate::tiles::TileId::parse("33392-5820").unwrap();
        serve_synthetic_tile(fetcher, tile, "2026-01-26");
        fetcher.insert_prefix(
            crate::sources::berlin::BUILDINGS_SERVICE,
            crate::sources::berlin::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        fetcher.insert_prefix(
            crate::sources::berlin::LAND_USE_SERVICE,
            crate::sources::berlin::test_support::LAND_USE
                .as_bytes()
                .to_vec(),
        );
        (dir, cache_dir, cache)
    }

    #[test]
    fn berlin_project_gets_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m == "4 ALKIS buildings"),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|m| m == "6 ALKIS land use parts"),
            "{messages:?}"
        );
        assert_eq!(project.buildings.unwrap().count, 4);
        let text = std::fs::read_to_string(dir.path().join(BUILDINGS_RELPATH)).unwrap();
        let parsed = crate::sources::alkis::parse_buildings(&text).unwrap();
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DEBE")));
    }

    #[test]
    fn attribution_names_geoportal_berlin() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        let lgb = "© GeoBasis-DE/LGB 2026, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten geändert";
        assert!(attribution.contains(lgb), "{attribution}");
        assert!(attribution.contains(BERLIN_ATTRIBUTION), "{attribution}");
        let documents = starter_documents(dir.path()).unwrap();
        let sheet_texts: Vec<&str> = documents
            .map
            .parts
            .iter()
            .filter(|p| p.name == SHEET_PART)
            .flat_map(|p| &p.objects)
            .filter_map(|o| match &o.kind {
                crate::omap::model::MapObjectKind::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(sheet_texts.contains(&BERLIN_ATTRIBUTION), "{sheet_texts:?}");
        assert!(sheet_texts.contains(&lgb), "{sheet_texts:?}");

        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(!attribution.contains("Geoportal Berlin"), "{attribution}");
    }

    #[test]
    fn missing_ndom_falls_back_to_bdom() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let tile = crate::tiles::TileId::parse("33392-5820").unwrap();
        crate::fetch::test_support::serve_synthetic_bdom(&fetcher, tile, "2026-09-10");
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m == "nDSM of tile 33392-5820 from the surface model (bDOM)"),
            "{messages:?}"
        );
        assert!(
            project
                .tiles
                .iter()
                .any(|r| r.product == Product::ImageDsm && r.tile == "33392-5820")
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn unavailable_bdom_only_warns() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: surface model (bDOM) unavailable")),
            "{messages:?}"
        );
        assert!(!project.tiles.iter().any(|r| r.product == Product::ImageDsm));
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(!dir.path().join(NDSM_RELPATH).exists());
    }

    #[test]
    fn alkis_records_name_their_regions() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.buildings.unwrap().regions, [Region::Berlin]);
        assert_eq!(project.land_use.unwrap().regions, [Region::Berlin]);

        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.buildings.unwrap().regions, [Region::Brandenburg]);
    }

    #[test]
    fn berlin_alkis_line_needs_berlin_objects() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        assert!(
            !attribution_lines(&project)
                .iter()
                .any(|l| l.contains("Berlin"))
        );
        project.buildings.as_mut().unwrap().regions = vec![Region::Brandenburg, Region::Berlin];
        assert!(attribution_lines(&project).contains(&BERLIN_ATTRIBUTION.to_string()));
    }

    #[test]
    fn republished_bdom_warns() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let tile = crate::tiles::TileId::parse("33392-5820").unwrap();
        crate::fetch::test_support::serve_synthetic_bdom(&fetcher, tile, "2026-09-10");
        fetch(dir.path(), &fetcher, &cache);
        crate::fetch::test_support::serve_synthetic_bdom(&fetcher, tile, "2026-10-01");
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.contains(
                &"warning: ImageDsm tile 33392-5820 changed upstream (2026-09-10 → 2026-10-01)"
                    .to_string()
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn empty_berlin_buildings_hint_to_refetch() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        fetcher.insert_prefix(
            crate::sources::berlin::BUILDINGS_SERVICE,
            crate::sources::alkis::test_support::page(&[], None),
        );
        fetch(dir.path(), &fetcher, &cache);
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: no ALKIS buildings stored for this area")),
            "{messages:?}"
        );
    }

    #[test]
    fn empty_alkis_warns() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(&[], None),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: no ALKIS buildings found")),
            "{messages:?}"
        );
    }

    #[test]
    fn fetch_stores_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let text = std::fs::read_to_string(dir.path().join(BUILDINGS_RELPATH)).unwrap();
        let parsed = crate::sources::alkis::parse_buildings(&text).unwrap();
        assert_eq!(parsed.buildings.len(), 1);
    }

    #[test]
    fn failed_alkis_fetch_keeps_stored_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let stored = std::fs::read(dir.path().join(BUILDINGS_RELPATH)).unwrap();
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            b"<html>maintenance</html>".to_vec(),
        );
        let err = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err();
        assert!(
            matches!(err, ProjectError::Fetch(FetchError::Alkis(_))),
            "{err:?}"
        );
        assert_eq!(
            std::fs::read(dir.path().join(BUILDINGS_RELPATH)).unwrap(),
            stored
        );
    }

    #[test]
    fn alkis_failure_after_republished_tile_keeps_project_derivable() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        serve_synthetic_tile(&fetcher, school_tile(), "2026-06-01");
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            b"<html>maintenance</html>".to_vec(),
        );
        let err = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err();
        assert!(
            matches!(err, ProjectError::Fetch(FetchError::Alkis(_))),
            "{err:?}"
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    #[test]
    fn offline_fetch_uses_stored_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let before = fetcher.calls.get();
        fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetcher.calls.get(), before, "offline makes no requests");

        std::fs::remove_file(dir.path().join(BUILDINGS_RELPATH)).unwrap();
        let err = fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::BuildingsNotFetched), "{err:?}");
    }

    #[test]
    fn corrupt_stored_buildings_name_the_file() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        std::fs::write(dir.path().join(BUILDINGS_RELPATH), "{ broken").unwrap();
        let err = export_starter_project(dir.path(), false).unwrap_err();
        assert!(
            matches!(err, ProjectError::StoredBuildings { .. }),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("alkis-buildings.geojson") && message.contains("freemapper fetch"),
            "{message}"
        );
    }

    #[test]
    fn fetch_records_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        let record = project.buildings.clone().expect("buildings recorded");
        assert_eq!(record.count, 1);
        assert_eq!(record.latest_update, "2023-05-04".parse().ok());
        let bbox = project.bbox().unwrap();
        assert!(
            record.bbox[0] <= bbox.min_e && record.bbox[3] >= bbox.max_n,
            "{record:?}"
        );

        let newer = crate::sources::alkis::test_support::building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Allgemein bildende Schule",
            crate::fetch::test_support::SCHOOL_BUILDING,
        )
        .replace("2023-05-04", "2024-01-01");
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(&[newer], None),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.contains(
                &"warning: ALKIS buildings changed upstream (2023-05-04 → 2024-01-01)".to_string()
            ),
            "{messages:?}"
        );

        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(
                &[
                    crate::sources::alkis::test_support::building_feature(
                        "DEBBAL0100000001",
                        "Gebäude",
                        "Allgemein bildende Schule",
                        crate::fetch::test_support::SCHOOL_BUILDING,
                    )
                    .replace("2023-05-04", "2024-01-01"),
                    crate::sources::alkis::test_support::building_feature(
                        "DEBBAL0100000002",
                        "Gebäude",
                        "Schuppen",
                        [421_620.0, 5_821_740.0, 3.0],
                    )
                    .replace("2023-05-04", "2024-01-01"),
                ],
                None,
            ),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.contains(
                &"warning: ALKIS buildings changed upstream (1 → 2 buildings)".to_string()
            ),
            "{messages:?}"
        );

        let mut grown = ProjectFile::load(dir.path()).unwrap();
        grown.buffer_m += 100.0;
        grown.save(dir.path()).unwrap();
        let err = fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::BuildingsNotFetched), "{err:?}");

        // A grown area changes the newest date only because more buildings come into range.
        let (_, after_growth) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !after_growth.iter().any(|m| m.contains("changed upstream")),
            "{after_growth:?}"
        );
    }

    #[test]
    fn offline_without_buildings_record_warns() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buildings = None;
        project.save(dir.path()).unwrap();
        let mut messages = Vec::new();
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Offline, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: ALKIS buildings coverage unknown")),
            "{messages:?}"
        );
    }

    #[test]
    fn fetch_stores_osm_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let text = std::fs::read_to_string(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        let features = crate::sources::osm::features_from_geojson(&text).unwrap();
        assert!(
            features.iter().any(|f| f.id == "way/1028507308"),
            "tartan pitch"
        );
        assert!(
            !features.iter().any(|f| f.id == "node/9669755236"),
            "the underground hydrant is not wanted"
        );
        let record = ProjectFile::load(dir.path())
            .unwrap()
            .osm
            .expect("OSM recorded");
        assert_eq!(record.extract.to_string(), "2026-11-06");
        assert_eq!(record.count, features.len());
    }

    #[test]
    fn export_adds_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        // The tartan pitch lies on the school grounds; the hydrant stands 63 m away in the street,
        // beyond the 20 m buffer.
        assert!(candidates.contains(r#"<t k="freemapper:source_id">way/1028507308</t>"#));
        assert!(!candidates.contains("node/9669755236"));
        assert!(candidates.contains(r#"<t k="freemapper:currency">2026-11-06</t>"#));
        let pitch_symbol = MapKind::Schoolyard
            .symbol_set()
            .id_for_code("501.10")
            .unwrap();
        assert!(candidates.contains(&format!(r#"<object type="1" symbol="{pitch_symbol}""#)));
    }

    /// The fetched school project as a sprint map, with two stored OSM ways across the area:
    /// way/1 an unpaved track (a map object), way/2 a path without surface (a candidate).
    fn sprint_with_ways(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir) {
        let (dir, cache_dir, _) = fetched_school(fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.map_kind = MapKind::Sprint;
        project.scale_denominator = 4000;
        project.save(dir.path()).unwrap();
        let c = fixture_area().centroid();
        let points = vec![
            c,
            LatLon {
                lat: c.lat + 0.0003,
                lon: c.lon + 0.0003,
            },
        ];
        let way = |id: &str, tags: &[(&str, &str)]| crate::sources::osm::OsmFeature {
            id: id.to_string(),
            tags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            geometry: crate::sources::osm::OsmGeometry::Line(points.clone()),
        };
        let features = [
            way("way/1", &[("highway", "track"), ("surface", "gravel")]),
            way("way/2", &[("highway", "path")]),
        ];
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&features),
        )
        .unwrap();
        (dir, cache_dir)
    }

    /// Replaces the stored land use with one carriageway (`Straßenverkehr`) and one way (`Weg`)
    /// next to the school building.
    fn store_traffic_areas(dir: &Path) {
        use crate::sources::alkis::test_support::{land_use_feature, page};
        let land_use = [
            land_use_feature(
                "DEBBAL0100000030",
                "Straßenverkehr",
                None,
                [421_560.0, 5_821_690.0, 30.0],
            ),
            land_use_feature(
                "DEBBAL0100000031",
                "Weg",
                None,
                [421_620.0, 5_821_690.0, 30.0],
            ),
        ];
        std::fs::write(dir.join(LAND_USE_RELPATH), page(&land_use, None)).unwrap();
    }

    #[test]
    fn export_adds_paved_areas_to_sprint_map() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        store_traffic_areas(dir.path());
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let paved = MapKind::Sprint.symbol_set().id_for_code("501.5").unwrap();
        assert!(map.contains(r#"<t k="freemapper:source_id">DEBBAL0100000030</t>"#));
        assert!(map.contains(&format!(r#"<object type="1" symbol="{paved}""#)));
        assert!(!map.contains("DEBBAL0100000031"), "Weg is a candidate");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(r#"<t k="freemapper:source_id">DEBBAL0100000031</t>"#));
    }

    #[test]
    fn schoolyard_export_has_no_paved_areas() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        store_traffic_areas(dir.path());
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(!map.contains("DEBBAL0100000030"));
        // On the schoolyard map the way parcel is out of bounds (520), not a paved candidate.
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let out_of_bounds = MapKind::Schoolyard.symbol_set().id_for_code("520").unwrap();
        assert_eq!(symbols_of(&candidates, "DEBBAL0100000031"), [out_of_bounds]);
    }

    /// The symbol ids of the objects tagged with `source_id` in an OOM file.
    fn symbols_of(omap: &str, source_id: &str) -> Vec<u32> {
        omap.split("<object ")
            .skip(1)
            .filter(|o| o.contains(&format!(r#"<t k="freemapper:source_id">{source_id}</t>"#)))
            .map(|o| {
                let start = o.find(r#"symbol=""#).unwrap() + 8;
                o[start..].split('"').next().unwrap().parse().unwrap()
            })
            .collect()
    }

    /// The geographic position of an EPSG:25833 point (iterating the forward projection).
    fn lat_lon_at(easting: f64, northing: f64) -> LatLon {
        let mut p = fixture_area().centroid();
        for _ in 0..6 {
            let q = crate::crs::utm33_forward(p);
            p.lat += (northing - q.northing) / 111_200.0;
            p.lon += (easting - q.easting) / (111_200.0 * p.lat.to_radians().cos());
        }
        p
    }

    #[test]
    fn unpaved_street_keeps_its_parcel_a_candidate() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        store_traffic_areas(dir.path());
        // A sand road across the Straßenverkehr square (421 560–421 590 E, 5 821 690–720 N).
        let road = crate::sources::osm::OsmFeature {
            id: "way/3".to_string(),
            tags: vec![
                ("highway".to_string(), "residential".to_string()),
                ("surface".to_string(), "sand".to_string()),
            ],
            geometry: crate::sources::osm::OsmGeometry::Line(vec![
                lat_lon_at(421_550.0, 5_821_705.0),
                lat_lon_at(421_600.0, 5_821_705.0),
            ]),
        };
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&[road]),
        )
        .unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(!map.contains("DEBBAL0100000030"), "sand road: not paved");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let paved = MapKind::Sprint.symbol_set().id_for_code("501.5").unwrap();
        assert_eq!(symbols_of(&candidates, "DEBBAL0100000030"), [paved]);
    }

    #[test]
    fn export_puts_unpaved_paths_into_the_sprint_map() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let track = MapKind::Sprint.symbol_set().id_for_code("505.2").unwrap();
        assert!(map.contains(r#"<t k="freemapper:source_id">way/1</t>"#));
        assert!(map.contains(&format!(r#"<object type="1" symbol="{track}""#)));
        assert!(!map.contains("way/2"));
    }

    #[test]
    fn export_puts_paths_without_surface_into_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(r#"<t k="freemapper:source_id">way/2</t>"#));
        assert!(!candidates.contains("way/1<"));
    }

    #[test]
    fn export_without_osm_candidates_still_works() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        std::fs::remove_file(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("way/1028507308"), "no OSM candidates");
        assert!(candidates.contains("way/992423119"), "outline stays");
    }

    #[test]
    fn corrupt_stored_candidates_name_the_file() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        std::fs::write(dir.path().join(OSM_CANDIDATES_RELPATH), "{ broken").unwrap();
        let message = export_starter_project(dir.path(), false)
            .unwrap_err()
            .to_string();
        assert!(
            message.contains("osm-candidates.geojson") && message.contains("freemapper fetch"),
            "{message}"
        );
    }

    #[test]
    fn offline_without_extract_keeps_stored_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        for entry in std::fs::read_dir(cache.osm_dir()).unwrap().flatten() {
            std::fs::remove_file(entry.path()).unwrap();
        }
        let mut messages = Vec::new();
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Offline, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: no OSM extract cached")),
            "{messages:?}"
        );
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains("way/1028507308"));
    }

    #[test]
    fn offline_fetch_reads_cached_extract() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        std::fs::remove_file(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        let calls = fetcher.calls.get();
        fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetcher.calls.get(), calls);
        assert!(dir.path().join(OSM_CANDIDATES_RELPATH).exists());
    }

    #[test]
    fn export_refuses_candidates_not_covering_the_area() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m += 10.0;
        project.buildings = None;
        project.save(dir.path()).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let err = export_starter_project(dir.path(), false).unwrap_err();
        assert!(matches!(err, ProjectError::CandidatesOutdated), "{err:?}");
        assert!(err.to_string().contains("freemapper fetch"), "{err}");
    }

    #[test]
    fn fetch_stores_land_use() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let text = std::fs::read_to_string(dir.path().join(LAND_USE_RELPATH)).unwrap();
        assert_eq!(
            crate::sources::alkis::parse_land_use(&text).unwrap().len(),
            2
        );
        let record = ProjectFile::load(dir.path())
            .unwrap()
            .land_use
            .expect("recorded");
        assert_eq!(record.count, 2);
    }

    #[test]
    fn export_adds_out_of_bounds_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(r#"<t k="freemapper:source_id">DEBBAL0100000011</t>"#));
        assert!(
            !candidates.contains("DEBBAL0100000010"),
            "the school's own land"
        );
        let id = MapKind::Schoolyard.symbol_set().id_for_code("520").unwrap();
        assert!(candidates.contains(&format!(r#"<object type="1" symbol="{id}">"#)));
        let map = std::fs::read_to_string(map_path(dir.path())).unwrap();
        assert!(
            !map.contains("alkis-nutzung"),
            "private land stays a template"
        );
    }

    #[test]
    fn export_without_land_use_still_works() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        std::fs::remove_file(dir.path().join(LAND_USE_RELPATH)).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("alkis-nutzung"));
    }

    #[test]
    fn stale_land_use_record_is_skipped_in_export() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.land_use.as_mut().unwrap().bbox[2] -= 50.0;
        project.save(dir.path()).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("alkis-nutzung"));
    }

    #[test]
    fn failed_land_use_fetch_after_area_change_drops_the_record() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m += 10.0;
        project.save(dir.path()).unwrap();
        fetcher.insert_prefix(
            crate::sources::alkis::LAND_USE_ENDPOINT,
            b"<html>maintenance</html>".to_vec(),
        );
        let mut messages = Vec::new();
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(ProjectFile::load(dir.path()).unwrap().land_use.is_none());
        assert!(!dir.path().join(LAND_USE_RELPATH).exists());
        assert!(
            messages
                .iter()
                .any(|m| m.contains("no longer covers the map area")),
            "{messages:?}"
        );
    }

    #[test]
    fn offline_fetch_leaves_land_use_alone() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let stored = std::fs::read(dir.path().join(LAND_USE_RELPATH)).unwrap();
        let calls = fetcher.calls.get();
        fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(fetcher.calls.get(), calls);
        assert_eq!(
            std::fs::read(dir.path().join(LAND_USE_RELPATH)).unwrap(),
            stored
        );
        assert!(ProjectFile::load(dir.path()).unwrap().land_use.is_some());
    }

    #[test]
    fn first_land_use_failure_does_not_claim_a_stored_file() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetcher.insert_prefix(
            crate::sources::alkis::LAND_USE_ENDPOINT,
            b"<html>maintenance</html>".to_vec(),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        let warning = messages
            .iter()
            .find(|m| m.starts_with("warning: ALKIS land use unavailable"))
            .expect("warning");
        assert!(!warning.contains("keeping"), "{warning}");
    }

    #[test]
    fn failed_land_use_fetch_keeps_stored_file() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let stored = std::fs::read(dir.path().join(LAND_USE_RELPATH)).unwrap();
        fetcher.insert_prefix(
            crate::sources::alkis::LAND_USE_ENDPOINT,
            b"<html>maintenance</html>".to_vec(),
        );
        let mut messages = Vec::new();
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert_eq!(
            std::fs::read(dir.path().join(LAND_USE_RELPATH)).unwrap(),
            stored
        );
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: ALKIS land use unavailable")),
            "{messages:?}"
        );
    }

    #[test]
    fn export_includes_buildings() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let map_path = export_starter_project(dir.path(), false).unwrap();
        let map = std::fs::read_to_string(map_path).unwrap();
        let parts = map.find("<parts ").unwrap();
        assert_eq!(
            map[parts..]
                .matches(r#"<t k="freemapper:source">alkis</t>"#)
                .count(),
            1
        );
    }

    #[test]
    fn export_without_buildings_has_empty_map() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let map_path = export_starter_project(dir.path(), false).unwrap();
        let map = std::fs::read_to_string(map_path).unwrap();
        assert!(!map.contains("alkis"));
    }

    #[test]
    fn project_without_title_loads() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.title, None);
        let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert!(!text.contains("title"), "{text}");
    }

    #[test]
    fn display_title_from_folder_name() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let folder = Path::new("/maps/clara-zetkin-schule");
        assert_eq!(project.display_title(folder), "Clara-Zetkin-Schule");
        assert_eq!(
            project.display_title(Path::new("/maps/sport_park")),
            "Sport-Park"
        );
        project.title = Some("Clara-Zetkin-Schule Strausberg".to_string());
        assert_eq!(
            project.display_title(folder),
            "Clara-Zetkin-Schule Strausberg"
        );
    }

    #[test]
    fn empty_title_falls_back_to_folder_name() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Schoolyard,
            None,
            date,
            Some("  ".to_string()),
        )
        .unwrap();
        assert_eq!(project.title, None);
        assert_eq!(project.display_title(Path::new("/maps/--")), "Karte");
        let mut titled = project.clone();
        titled.title = Some("Schule\u{7} Nord".to_string());
        assert_eq!(titled.display_title(Path::new("/maps/x")), "Schule Nord");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_project_keeps_its_own_name() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("abc123");
        school_project(&real);
        let link = dir.path().join("clara-zetkin-schule");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let project = ProjectFile::load(&link).unwrap();
        assert_eq!(project.display_title(&link), "Clara-Zetkin-Schule");
        assert_eq!(
            map_path(&link).file_name().unwrap(),
            "clara-zetkin-schule.omap"
        );
    }

    #[test]
    fn display_title_of_dot_uses_the_real_folder() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("clara-zetkin-schule");
        school_project(&project_dir);
        let project = ProjectFile::load(&project_dir).unwrap();
        // Like `.` from inside the folder: the path has no file name of its own.
        std::fs::create_dir(project_dir.join("x")).unwrap();
        let unnamed = project_dir.join("x").join("..");
        assert_eq!(unnamed.file_name(), None);
        let title = project.display_title(&unnamed);
        let map = map_path(&unnamed);
        assert_eq!(title, "Clara-Zetkin-Schule");
        assert_eq!(map.file_name().unwrap(), "clara-zetkin-schule.omap");
    }

    #[test]
    fn export_sprint_project() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Sprint,
            None,
            date,
            None,
        )
        .unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(map.contains("<text>Maßstab 1:4000 · Äquidistanz 2 m</text>"));
        assert!(
            map.contains(r#"<coords count="2">30000 "#),
            "north line at 30 mm"
        );
        assert!(!map.contains(r#"<coords count="2">37500 "#));
    }

    #[test]
    fn export_has_one_peripheral_part() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("clara-zetkin-schule");
        school_project(&project_dir);
        export_starter_project(&project_dir, false).unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(&project_dir, true).unwrap()).unwrap();
        assert_eq!(
            map.matches(r#"<part name="peripheral information">"#)
                .count(),
            1
        );
        assert!(map.contains("<text>Clara-Zetkin-Schule</text>"));
        assert!(map.contains(
            "<text>Erstellt mit freemapper – https://github.com/HenryJobst/freemapper</text>"
        ));
        assert!(map.contains("<text>© OpenStreetMap-Mitwirkende, ODbL</text>"));
        assert!(
            map.contains(FREEMAPPER_CREDIT),
            "notes keep the English credit"
        );
        assert!(map.contains("<text>Maßstab 1:1000 · Äquidistanz 1 m</text>"));
        let default = map.find(r#"<part name="default part">"#).unwrap();
        let sheet = map.find(r#"<part name="peripheral information">"#).unwrap();
        assert!(default < sheet, "buildings part first");
    }

    #[test]
    fn export_names_freemapper_and_its_repository() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let map_path = export_starter_project(dir.path(), false).unwrap();
        let map = std::fs::read_to_string(map_path).unwrap();
        let credit = "Created with freemapper – https://github.com/HenryJobst/freemapper";
        assert!(map.contains(credit), "map notes");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(credit), "candidates notes");
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(attribution.contains(credit), "{attribution}");
    }

    #[test]
    fn derive_detects_stale_tile_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m = 600.0;
        project.save(dir.path()).unwrap();
        let err = derive_project(dir.path(), &cache, &mut |_| {}).unwrap_err();
        assert!(matches!(err, ProjectError::TilesOutdated(_)), "{err:?}");
        assert!(err.to_string().contains("freemapper fetch"), "{err}");
    }

    #[test]
    fn derived_images_leave_no_partial_files() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let leftovers: Vec<_> = std::fs::read_dir(dir.path().join("templates"))
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn fetch_not_covered_leaves_project_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let before = std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        let fetcher = MapFetcher::default();
        let elsewhere = crate::tiles::TileId::parse("33999-5999").unwrap();
        serve_synthetic_tile(&fetcher, elsewhere, "2026-01-26");
        let err = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err();
        assert!(
            matches!(err, ProjectError::Fetch(FetchError::TilesNotCovered(_))),
            "{err:?}"
        );
        assert_eq!(
            std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap(),
            before
        );
    }

    #[test]
    fn export_detects_template_not_matching_the_area() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();

        // A world file of another run, e.g. after a crash between image and world file.
        let world = dir.path().join("templates/hillshade.pgw");
        let shifted = std::fs::read_to_string(&world)
            .unwrap()
            .replacen("421", "420", 1);
        std::fs::write(&world, shifted).unwrap();
        let err = export_starter_project(dir.path(), true).unwrap_err();
        assert!(matches!(err, ProjectError::TemplateOutdated(_)), "{err:?}");
        let message = err.to_string();
        assert!(
            message.contains("hillshade.png") && message.contains("freemapper derive"),
            "{message}"
        );

        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), true).unwrap();
    }

    fn derived_school_with_ndsm() -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        let (dir, cache_dir, cache) = fetched_school(&fetcher);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        (dir, cache_dir, cache)
    }

    #[test]
    fn derive_writes_contours_and_ndsm() {
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        let text = std::fs::read_to_string(dir.path().join(CONTOURS_RELPATH)).unwrap();
        let lines = crate::derive::contours::contours_from_geojson(&text).unwrap();
        assert!(lines.len() >= 10, "{} contour lines", lines.len());
        assert!(
            lines.iter().all(|l| l.height.fract() == 0.0),
            "1 m interval"
        );
        assert!(dir.path().join(NDSM_RELPATH).exists());
        assert!(dir.path().join("templates/ndsm.pgw").exists());
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.ndsm_tiles.len(), 1);
        assert_eq!(project.ndsm_tiles[0].published, "2025-03-03".parse().ok());
    }

    #[test]
    fn ndsm_record_with_a_path_as_tile_is_ignored() {
        let (dir, _cache_dir, cache) = derived_school_with_ndsm();
        let outside = dir.path().join("outside.laz");
        std::fs::copy(
            cache.ndsm_path(school_tile(), "2025-03-03".parse().ok()),
            &outside,
        )
        .unwrap();
        std::fs::remove_file(dir.path().join(NDSM_RELPATH)).unwrap();
        let mut project = ProjectFile::load(dir.path()).unwrap();
        // A tampered project file pointing outside the cache.
        project.ndsm_tiles[0].tile =
            format!("../../../../{}", outside.with_extension("").display());
        project.ndsm_tiles[0].published = None;
        project.save(dir.path()).unwrap();
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(
            !dir.path().join(NDSM_RELPATH).exists(),
            "nothing read outside the cache"
        );
        assert!(
            messages.iter().any(|m| m.contains("invalid nDSM tile")),
            "{messages:?}"
        );
    }

    #[test]
    fn export_adds_contour_candidates() {
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(r#"<t k="freemapper:source">lgb-dgm1</t>"#));
        let set = MapKind::Schoolyard.symbol_set();
        for code in ["101", "102"] {
            let id = set.id_for_code(code).unwrap();
            assert!(
                candidates.contains(&format!(r#"<object type="1" symbol="{id}">"#)),
                "{code}"
            );
        }
    }

    #[test]
    fn export_references_ndsm_template() {
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(map.contains(r#"relpath="templates/ndsm.png""#));
        let ndsm_ref = map
            .split("<ref ")
            .nth(4)
            .expect("fourth template ref is the nDSM");
        assert!(
            ndsm_ref.starts_with(r#"template="3" visible="false" opacity="0.6""#),
            "{ndsm_ref}"
        );
        assert!(
            map.contains(r#"first_front_template="4""#),
            "four rasters behind the map"
        );
    }

    #[test]
    fn stale_ndsm_template_is_removed_when_no_ndsm_is_available() {
        let (dir, _cache_dir, cache) = derived_school_with_ndsm();
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.ndsm_tiles.clear();
        project.save(dir.path()).unwrap();
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(!dir.path().join(NDSM_RELPATH).exists());
        assert!(!dir.path().join("templates/ndsm.pgw").exists());
        assert!(
            messages.iter().any(|m| m.contains("no nDSM recorded")),
            "{messages:?}"
        );
        export_starter_project(dir.path(), true).unwrap();
    }

    #[test]
    fn contours_inside_buildings_and_far_away_are_left_out() {
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        let mut lines = vec![
            // A ring on the synthetic school building (10 m square at 421 605 / 5 821 730).
            crate::derive::contours::ContourLine {
                height: 61.0,
                points: vec![
                    (421_606.0, 5_821_731.0),
                    (421_614.0, 5_821_731.0),
                    (421_614.0, 5_821_739.0),
                    (421_606.0, 5_821_739.0),
                    (421_606.0, 5_821_731.0),
                ],
            },
            // A line 500 m away, beyond map area and buffer.
            crate::derive::contours::ContourLine {
                height: 62.0,
                points: vec![(422_200.0, 5_821_700.0), (422_210.0, 5_821_700.0)],
            },
        ];
        let kept_line = crate::derive::contours::ContourLine {
            height: 63.0,
            points: vec![(421_600.0, 5_821_700.0), (421_640.0, 5_821_700.0)],
        };
        lines.push(kept_line);
        std::fs::write(
            dir.path().join(CONTOURS_RELPATH),
            crate::derive::contours::contours_to_geojson(&lines),
        )
        .unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let heights: Vec<&str> = candidates
            .match_indices(r#"<t k="freemapper:height">"#)
            .map(|(i, m)| {
                let rest = &candidates[i + m.len()..];
                &rest[..rest.find('<').unwrap()]
            })
            .collect();
        assert_eq!(heights, ["63"]);
    }

    #[test]
    fn republished_ndsm_tile_warns() {
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        fetcher.set_modified(
            &crate::sources::lgb::ndsm_tile_url(school_tile()),
            "2026-09-01".parse().unwrap(),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.contains(
                &"warning: nDSM tile 33421-5821 changed upstream (2025-03-03 → 2026-09-01)"
                    .to_string()
            ),
            "{messages:?}"
        );
    }

    #[test]
    fn unreadable_cached_ndsm_is_skipped() {
        let (dir, _cache_dir, cache) = derived_school_with_ndsm();
        let cached = cache.ndsm_path(school_tile(), "2025-03-03".parse().ok());
        std::fs::write(&cached, b"LASF broken").unwrap();
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(
            messages.iter().any(|m| m.contains("unreadable")),
            "{messages:?}"
        );
        assert!(!dir.path().join(NDSM_RELPATH).exists());
    }

    #[test]
    fn export_without_contours_still_works() {
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        std::fs::remove_file(dir.path().join(CONTOURS_RELPATH)).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("lgb-dgm1"));
    }

    #[test]
    fn derive_before_fetch_fails() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let err = derive_project(
            dir.path(),
            &Cache::new(cache_dir.path().to_path_buf()),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::NotFetched), "{err:?}");
        assert!(err.to_string().contains("freemapper fetch"), "{err}");
    }

    #[test]
    fn init_rejects_scale_zero() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let result = init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Schoolyard,
            Some(0),
            date,
            None,
        );
        assert!(
            matches!(result, Err(ProjectError::InvalidScale(0))),
            "{result:?}"
        );
        assert!(!dir.path().join(PROJECT_FILE_NAME).exists());
    }

    #[test]
    fn outline_maps_back_onto_area_vertices() {
        let dir = tempfile::tempdir().unwrap();
        let area = fixture_area();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project =
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        let georef = Georeferencing::new(area.centroid(), 1000, date).unwrap();
        let outline =
            map_area_outline(&area, &georef, &MapKind::Schoolyard.symbol_set(), &project).unwrap();

        let vertices: Vec<_> = area.polygon.exterior().points().collect();
        assert_eq!(outline.coords.len(), vertices.len());
        for (coord, vertex) in outline.coords.iter().zip(&vertices) {
            let expected = utm33_forward(LatLon {
                lat: vertex.y(),
                lon: vertex.x(),
            });
            let back = georef.map_to_projected(*coord);
            // 1 µm on paper is 1 mm on the ground at 1:1000.
            assert!(
                (back.easting - expected.easting).abs() < 0.002,
                "{back:?} vs {expected:?}"
            );
            assert!(
                (back.northing - expected.northing).abs() < 0.002,
                "{back:?} vs {expected:?}"
            );
        }
        let keys: Vec<&str> = outline.tags.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["freemapper:source", "freemapper:source_id"]);
        // Regression pin: the first vertex, south-west of the centroid on paper.
        assert_eq!(
            (outline.coords[0].x, outline.coords[0].y),
            (-46_668, 142_622)
        );
    }
}
