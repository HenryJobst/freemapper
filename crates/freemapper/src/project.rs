//! Project folder and project file (`freemapper.toml`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{Datelike as _, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::area::GeoBox;
use crate::area::{AreaError, MapArea};
use crate::build::berlin::{
    BUILDING_PARTS_LAYER, LayerInput, berlin_objects, building_part_features,
    drop_shelters_under_canopies,
};
use crate::build::buildings::{Rule, building_objects, classify};
use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::canopy::{CANOPY_LAYER, CanopyInput, canopy_objects};
use crate::build::dedup::{
    drop_cadastre_lines_by_rank, drop_osm_lines_near_cadastre, drop_points_near_registers,
    drop_trees_by_rank,
};
use crate::build::out_of_bounds::out_of_bounds_objects;
use crate::build::paved::{
    GREEN_LAYER, PavedObjects, SURFACE_LAYER, StreetSurvey, paved_area_objects, unpaved_streets,
};
use crate::build::peripheral::{PaperRect, PeripheralInfo, info_block, north_lines};
use crate::build::points::{GENERALIZATION_SOURCE, HIDDEN_KEY, generalize_points};
use crate::build::stairs::{
    NAS_STAIR_LAYER, STEP_EDGE_LAYER, SURVEY_STAIR_LAYER, StairInput, has_nas_stairs_without_edge,
    has_stairs, stair_objects,
};
use crate::build::state::{NasInput, TreeInput, nas_objects, state_objects, tree_objects};
use crate::build::water::{drop_covered_water, drop_points_in_water, outside_water, water_objects};
use crate::crs::ProjectedPoint;
use crate::crs::{BoundingBox, Crs, LatLon};
use crate::derive::contours::{
    contour_lines, contours_from_geojson, contours_to_geojson, gaussian_smooth, generalize,
};
use crate::derive::ndsm::{fill_from_dsm, rasterize, write_ndsm_png};
use crate::derive::orthophoto::{build_orthophoto, build_orthophoto_from_image};
use crate::derive::terrain::{
    hillshade, mosaic_dtm, read_float_raster, slope_deg, slope_shade, write_gray_png,
};
use crate::derive::{DeriveError, MAX_RASTER_SIDE_PX};
use crate::fetch::{Cache, FetchError, FetchMode, FetchedTile, Fetcher, fetch_tiles, write_inside};
use crate::fetch::{fetch_ndsm_tiles, fetch_tiles_each};
use crate::georef::{GeorefError, Georeferencing};
use crate::ocd::convert::{Background, WorldFile, map_to_ocd};
use crate::omap::MapCoord;
use crate::omap::{
    MapDocument, MapObject, MapObjectKind, MapPart, OmapError, SymbolSet, TemplateKind,
    TemplateRef, XmlFormat, closed_ring, write_map,
};
use crate::raster::{GridSpec, Raster};
use crate::sources::alkis::{FetchArea, LandUse, fetch_land_use_in, parse_land_use};
use crate::sources::alkis::{fetch_buildings_in, parse_building_features, parse_buildings};
use crate::sources::lgb::{Product, TileCurrency, attribution, attribution_with_berlin};
use crate::sources::osm::{
    OsmFeature, cached_extract_date, extracts_for, features_from_geojson, features_to_geojson,
    fetch_extract, read_extract,
};
use crate::sources::region::{
    Provider, Region, provider_tiles, raster_region, regions_for, touches_berlin,
};
use crate::sources::trees;
use crate::sources::wms::{DateLabel, OrthophotoSource};
use crate::sources::{alkis_nas, berlin};
use crate::space::{Part, SpacePlan};
use crate::tiles::{TileId, tiles_covering_grid};

pub const PROJECT_FILE_NAME: &str = "freemapper.toml";

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("{}", crate::space::io_error_text(.0, .1))]
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
    #[error("no orthophoto stored for this map area; run `freemapper fetch` online first")]
    OrthophotoNotFetched,
    #[error("no terrain model stored for this map area; run `freemapper fetch` online first")]
    TerrainNotFetched,
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
        "the map area lies outside the supported states (Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate); freemapper has data sources for those only so far"
    )]
    OutsideCoverage,
    #[error("{}: {message}; run `freemapper fetch` again", path.display())]
    StoredBuildings { path: PathBuf, message: String },
    /// A part the run needs does not fit on its volume.
    #[error("{0}")]
    NotEnoughSpace(crate::space::Shortfall),
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
    /// EPSG code of the map's CRS: 25832 or 25833, from the raster region. Projects from before
    /// it was recorded are 25833.
    #[serde(default = "default_crs")]
    pub crs: u32,
    /// Who serves the rasters, from the raster region (missing in older projects).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
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
    /// The extra Berlin layers (ALKIS structures, tree cadastre) of the last online `fetch`, by
    /// file name under [`BERLIN_LAYERS_DIR`]; empty outside Berlin. A layer without objects in
    /// the area is recorded with count 0.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub berlin_layers: BTreeMap<String, AlkisRecord>,
    /// The NAS layers (full ALKIS) of the last online `fetch`, by file name under
    /// [`NAS_LAYERS_DIR`]; empty outside the states with a NAS service. A layer without objects
    /// in the area is recorded with count 0, and `latest_update` is the newest `beginnt`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nas_layers: BTreeMap<String, AlkisRecord>,
    /// The city tree cadastres of the last online `fetch`, by dataset id under
    /// [`TREE_LAYERS_DIR`]; empty outside the cities of the table. A dataset without trees in the
    /// area is recorded with count 0, and `latest_update` is the newest tree survey date.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tree_layers: BTreeMap<String, AlkisRecord>,
    /// The OSM extract the candidates were read from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osm: Option<OsmRecord>,
    /// The orthophoto from a WMS or WCS (GeoSN, LVermGeo, LGV, HVBG, LVermGeo SH: WMS; LAiV M-V: WCS); LGB orthophotos are tiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orthophoto: Option<WmsRecord>,
    /// The Berlin K5 map (1:5000) written as a template at [`K5_RELPATH`]; `flight` is the K5
    /// dataset date. Berlin only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub k5: Option<WmsRecord>,
    /// The terrain model of a per-area provider (LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V), at [`TERRAIN_RELPATH`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<WcsRecord>,
    /// The surface model of a per-area provider, at [`SURFACE_RELPATH`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<WcsRecord>,
    /// nDSM point clouds used by the last `fetch`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ndsm_tiles: Vec<NdsmRecord>,
}

/// The orthophoto fetched from a WMS for one area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WmsRecord {
    /// The flight date text the server gives, e.g. `19.03. + 07.04.2024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight: Option<String>,
    /// What the service calls the date in `flight` when it is no flight date as such
    /// (LVermGeo RP: `Aktualität (Erstellung)`, the date taken as the flight date).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight_label: Option<String>,
    pub fetched: NaiveDate,
    /// Area in the map's CRS: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
}

impl WmsRecord {
    fn covers(&self, bbox: &BoundingBox) -> bool {
        let [min_e, min_n, max_e, max_n] = self.bbox;
        min_e <= bbox.min_e && min_n <= bbox.min_n && max_e >= bbox.max_e && max_n >= bbox.max_n
    }
}

/// A height model fetched from a WCS for one area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WcsRecord {
    pub fetched: NaiveDate,
    /// Area in the map's CRS: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
    /// The date the provider gives for the data, e.g. `13.05.2024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

impl WcsRecord {
    fn covers(&self, bbox: &BoundingBox) -> bool {
        let [min_e, min_n, max_e, max_n] = self.bbox;
        min_e <= bbox.min_e && min_n <= bbox.min_n && max_e >= bbox.max_e && max_n >= bbox.max_n
    }
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
    /// Queried area in the map's CRS: min easting, min northing, max easting, max northing.
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
    /// The project file in `dir`, without the entries [`ProjectFile::load_reporting`] warns
    /// about.
    pub fn load(dir: &Path) -> Result<Self, ProjectError> {
        Self::load_reporting(dir, &mut |_| {})
    }

    /// The project file in `dir`. Entries whose name would become part of a file path but is
    /// not one freemapper knows (a hand-edited or crafted file: `../../x`, `/etc/x`) are left out
    /// with a warning through `progress`; the rest of the project loads (ADR 0059).
    pub fn load_reporting(
        dir: &Path,
        progress: &mut dyn FnMut(&str),
    ) -> Result<Self, ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path, e))?;
        let mut project: Self =
            toml::from_str(&text).map_err(|e| ProjectError::Toml(e.to_string()))?;
        project.drop_unknown_names(progress);
        Ok(project)
    }

    /// Leaves out the Berlin and NAS layer records whose key is no plain file name or no layer
    /// of the table, each with a warning. The key is shown escaped, as it may hold control
    /// characters.
    fn drop_unknown_names(&mut self, progress: &mut dyn FnMut(&str)) {
        let mut keep = |what: &str, key: &str, known: bool| {
            let reason = if !crate::fsutil::is_plain_name(key) {
                "not a plain file name"
            } else if !known {
                "no layer freemapper knows"
            } else {
                return true;
            };
            progress(&format!(
                "warning: freemapper.toml: ignored the {what} layer entry {key:?}: {reason}"
            ));
            false
        };
        self.berlin_layers
            .retain(|key, _| keep("Berlin", key, known_berlin_layer(key).is_some()));
        self.nas_layers
            .retain(|key, _| keep("NAS", key, known_nas_layer(key).is_some()));
        self.tree_layers
            .retain(|key, _| keep("tree", key, known_tree_dataset(key).is_some()));
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

    /// The map's CRS.
    pub fn crs(&self) -> Crs {
        Crs::from_epsg(self.crs).unwrap_or(Crs::Utm33)
    }

    /// Map area plus buffer in the map's CRS.
    pub fn bbox(&self) -> Result<BoundingBox, ProjectError> {
        let area = MapArea::from_geojson(&self.map_area.to_string())?;
        Ok(area.projected_bbox_in(self.crs()).buffered(self.buffer_m))
    }

    /// Map area plus buffer in EPSG:25833, where the region boundaries are.
    pub fn region_bbox(&self) -> Result<BoundingBox, ProjectError> {
        let area = MapArea::from_geojson(&self.map_area.to_string())?;
        Ok(area.projected_bbox().buffered(self.buffer_m))
    }

    /// Forgets everything fetched for another CRS, and deletes the stored data of it.
    fn drop_fetched(&mut self, dir: &Path) {
        for relpath in [
            BUILDINGS_RELPATH,
            LAND_USE_RELPATH,
            OSM_CANDIDATES_RELPATH,
            WMS_ORTHOPHOTO_RELPATH,
            K5_RELPATH,
            TERRAIN_RELPATH,
            TERRAIN_MOSAIC_RELPATH,
            SURFACE_RELPATH,
        ] {
            let path = dir.join(relpath);
            for file in [
                path.clone(),
                path.with_extension("jgw"),
                path.with_extension("pgw"),
                path.with_extension("tfw"),
            ] {
                crate::fsutil::remove_inside(dir, &file);
            }
        }
        self.tiles.clear();
        self.ndsm_tiles.clear();
        self.buildings = None;
        self.land_use = None;
        for key in self.berlin_layers.keys() {
            remove_berlin_layer(dir, key);
        }
        self.berlin_layers.clear();
        for key in self.nas_layers.keys() {
            remove_nas_layer(dir, key);
        }
        self.nas_layers.clear();
        for key in self.tree_layers.keys() {
            remove_tree_layer(dir, key);
        }
        self.tree_layers.clear();
        self.osm = None;
        self.orthophoto = None;
        self.k5 = None;
        self.terrain = None;
        self.surface = None;
    }

    pub fn save(&self, dir: &Path) -> Result<(), ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = toml::to_string(self).map_err(|e| ProjectError::Toml(e.to_string()))?;
        Ok(write_inside(dir, &path, text.as_bytes())?)
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
    let region_bbox = area.projected_bbox().buffered(map_kind.default_buffer_m());
    check_area(&region_bbox)?;
    let crs = map_crs(&region_bbox);
    let provider = raster_region(&region_bbox).map(Region::provider);
    check_size(
        &area
            .projected_bbox_in(crs)
            .buffered(map_kind.default_buffer_m()),
    )?;
    std::fs::create_dir_all(dir).map_err(|e| ProjectError::Io(dir.to_path_buf(), e))?;
    let project = ProjectFile {
        map_kind,
        scale_denominator,
        declination_date,
        buffer_m: map_kind.default_buffer_m(),
        crs: crs.epsg(),
        provider,
        map_area_source_id: area.source_id.clone(),
        map_area: area.to_geojson_value(),
        title: title.as_deref().and_then(clean_title),
        tiles: Vec::new(),
        buildings: None,
        land_use: None,
        berlin_layers: BTreeMap::new(),
        nas_layers: BTreeMap::new(),
        tree_layers: BTreeMap::new(),
        osm: None,
        orthophoto: None,
        k5: None,
        terrain: None,
        surface: None,
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
/// The Berlin K5 map (1:5000) as fetched; a template the mapper draws on, optional.
pub(crate) const K5_RELPATH: &str = "templates/k5.png";
/// The date of the K5 dataset (catalogue `karte-von-berlin-1-5000-k5-farbausgabe-9f3ecf8d`); the
/// WMS gives no date of its own.
const K5_DATASET_DATE: &str = "2026-01-02";
/// The WMS orthophoto (GeoSN, LVermGeo) as fetched, before it becomes the template.
pub(crate) const WMS_ORTHOPHOTO_RELPATH: &str = "data/orthophoto-wms.jpg";
/// The WCS terrain model (LVermGeo DGM1, 1 m) with its world file.
pub(crate) const TERRAIN_RELPATH: &str = "data/dgm1.tif";
/// The terrain mosaic of a tile provider (LGB, GeoSN) cut to the map area plus buffer, with its
/// world file. `derive` writes it only where the extra Berlin layers have stair areas: the export
/// reads their walking direction from it, offline and without the tile cache.
pub(crate) const TERRAIN_MOSAIC_RELPATH: &str = "data/terrain.tif";
/// The WCS surface model (LVermGeo DOM1, 1 m) with its world file.
pub(crate) const SURFACE_RELPATH: &str = "data/dom1.tif";
/// Contour lines from `derive` (GeoJSON in the map's CRS), input of the export.
const CONTOURS_RELPATH: &str = "data/contours.geojson";
/// OSM features matching the candidate rules (GeoJSON, EPSG:4326), input of the export.
const OSM_CANDIDATES_RELPATH: &str = "data/osm-candidates.geojson";
/// ALKIS land use around the map area (GeoJSON in the map's CRS), input of the export.
pub(crate) const LAND_USE_RELPATH: &str = "data/alkis-land-use.geojson";
/// Skipped ALKIS features listed in a progress message.
const SKIPPED_SHOWN: usize = 5;
/// Folder of the extra Berlin layers, one `<file>.geojson` per layer (GeoJSON as served, EPSG:25833).
pub(crate) const BERLIN_LAYERS_DIR: &str = "data/berlin";
/// Folder of the NAS layers (full ALKIS), one `<file>.geojson` per layer (GeoJSON in the map's CRS).
pub(crate) const NAS_LAYERS_DIR: &str = "data/alkis-nas";
/// Folder of the city tree cadastres, one `<dataset id>.geojson` per dataset (point features in
/// the map's CRS, `sources::trees`).
pub(crate) const TREE_LAYERS_DIR: &str = "data/trees";
/// ALKIS buildings around the map area, as fetched (GeoJSON in the map's CRS).
pub(crate) const BUILDINGS_RELPATH: &str = "data/alkis-buildings.geojson";

/// Sets the map area of the project in `dir` (from the area selection page), keeping every other
/// setting. Data fetched for the old area is fetched again for the new one by the next `fetch`.
pub fn set_map_area(dir: &Path, area: &MapArea) -> Result<ProjectFile, ProjectError> {
    let mut project = ProjectFile::load(dir)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    // Projects saved before the provider was recorded: that of the old area.
    let old_provider = project.provider.or_else(|| {
        project
            .region_bbox()
            .ok()
            .and_then(|b| raster_region(&b))
            .map(Region::provider)
    });
    project.map_area = area.to_geojson_value();
    project.map_area_source_id = area.source_id.clone();
    let region_bbox = project.region_bbox()?;
    check_area(&region_bbox)?;
    // Data fetched in the other UTM zone cannot be compared with the new area: fetch it again.
    let crs = map_crs(&region_bbox);
    check_size(&area.projected_bbox_in(crs).buffered(project.buffer_m))?;
    // Another provider (in the same zone) serves other rasters: fetch them again too.
    let provider = raster_region(&region_bbox).map(Region::provider);
    let provider_changed = old_provider.is_some() && old_provider != provider;
    if crs != project.crs() || provider_changed {
        project.crs = crs.epsg();
        project.drop_fetched(dir);
    }
    project.provider = provider;
    project.save(dir)?;
    Ok(project)
}

/// The CRS of a map of `region_bbox` (EPSG:25833): that of its raster region.
pub(crate) fn map_crs(region_bbox: &BoundingBox) -> Crs {
    raster_region(region_bbox).map_or(Crs::Utm33, Region::crs)
}

fn default_crs() -> u32 {
    Crs::Utm33.epsg()
}

/// Refuses a map area plus buffer outside every supported region or larger than the templates
/// can cover.
pub(crate) fn check_area(bbox: &BoundingBox) -> Result<(), ProjectError> {
    if raster_region(bbox).is_none() {
        return Err(ProjectError::OutsideCoverage);
    }
    check_size(bbox)
}

/// Refuses a map area plus buffer larger than the templates can cover. The templates are built
/// in the map's CRS, so this is checked there too: the two UTM grids are rotated against each
/// other, and a box that fits in one may not fit in the other.
pub(crate) fn check_size(bbox: &BoundingBox) -> Result<(), ProjectError> {
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
    let mut project = ProjectFile::load_reporting(dir, progress)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    let region_bbox = project.region_bbox()?;
    check_area(&region_bbox)?;
    // The stored CRS may be stale (written by hand, or by a version that only knew zone 33).
    let crs = map_crs(&region_bbox);
    if crs != project.crs() {
        progress(&format!(
            "warning: the map area lies in UTM zone {} (EPSG:{}); freemapper fetches everything again in that zone",
            if crs == Crs::Utm32 { 32 } else { 33 },
            crs.epsg()
        ));
        project.crs = crs.epsg();
        project.drop_fetched(dir);
        project.save(dir)?;
    }
    let provider = raster_region(&region_bbox).map(Region::provider);
    if project.provider.is_some() && project.provider != provider {
        progress(
            "warning: the map area now lies in another state's data; freemapper fetches everything again",
        );
        project.drop_fetched(dir);
        project.provider = provider;
        project.save(dir)?;
    }
    project.provider = provider;
    let bbox = project.bbox()?;
    check_size(&bbox)?;
    if crate::sources::region::reaches_beyond_coverage(&region_bbox) {
        progress(BEYOND_COVERAGE_WARNING);
    }
    // Offline nothing is downloaded, so nothing needs room.
    let plan = match mode {
        FetchMode::Online => check_space(dir, cache, &project, &region_bbox, &bbox, progress)?,
        FetchMode::Offline => None,
    };
    let skips = |part: Part| plan.as_ref().is_some_and(|plan| !plan.fits(part));
    let skip_surface = skips(Part::Surface);
    let p = &mut project;
    match raster_region(&region_bbox).map(Region::provider) {
        Some(Provider::LvermGeo) => {
            fetch_lvermgeo_rasters(dir, fetcher, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::Lgv) => {
            fetch_lgv_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::GeoSn) => {
            fetch_geosn_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::Hvbg) => {
            fetch_hvbg_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::LvermGeoSh) => {
            fetch_sh_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::LaivMv) => {
            fetch_mv_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::GeobasisNrw) => {
            fetch_nrw_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::Lgln) => {
            fetch_lgln_rasters(dir, fetcher, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::LvermGeoRp) => {
            fetch_rp_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
        Some(Provider::Lgb) | None => {
            fetch_lgb_rasters(dir, fetcher, cache, mode, skip_surface, p, &bbox, progress)?
        }
    }
    project.save(dir)?;
    let area = FetchArea {
        region_bbox: project.region_bbox()?,
        crs: project.crs(),
        bbox: project.bbox()?,
    };
    let previous = project.buildings.as_ref();
    if let Some(record) = fetch_buildings_into(dir, fetcher, &area, mode, previous, progress)? {
        project.buildings = Some(record);
        project.save(dir)?;
    }
    if let Some(record) = fetch_land_use_into(
        dir,
        fetcher,
        &area,
        mode,
        project.land_use.as_ref(),
        progress,
    ) {
        project.land_use = record;
        project.save(dir)?;
    }
    project.berlin_layers = fetch_berlin_layers_into(
        dir,
        fetcher,
        &area.region_bbox,
        project.map_kind,
        mode,
        skips(Part::BerlinLayers),
        &project.berlin_layers,
        progress,
    );
    project.save(dir)?;
    project.nas_layers = fetch_nas_layers_into(
        dir,
        fetcher,
        &area,
        project.map_kind,
        mode,
        skips(Part::StateLayers),
        &project.nas_layers,
        progress,
    );
    project.save(dir)?;
    project.tree_layers = fetch_tree_layers_into(
        dir,
        fetcher,
        &area,
        mode,
        skips(Part::StateLayers),
        &project.tree_layers,
        progress,
    );
    project.save(dir)?;
    fetch_k5_into(
        dir,
        fetcher,
        mode,
        skips(Part::K5),
        &mut project,
        &area.region_bbox,
        &area.bbox,
        progress,
    );
    project.save(dir)?;
    let skip_update = skips(Part::OsmUpdate);
    project.osm = Some(fetch_osm_into(
        dir,
        fetcher,
        cache,
        mode,
        skip_update,
        &project,
        progress,
    )?);
    project.save(dir)?;
    Ok(project)
}

/// Marks in `estimate` what the project folder `dir` already holds for `bbox` (the map area plus
/// buffer in the map's CRS): the stored orthophoto, terrain and surface models of a per-area
/// provider are replaced in place ([`DownloadEstimate::in_place`](crate::layers::DownloadEstimate)),
/// and the orthophoto counts as cached, as the fetch reuses it while the flight date stays.
pub(crate) fn mark_stored(
    estimate: &mut crate::layers::DownloadEstimate,
    dir: &Path,
    project: &ProjectFile,
    bbox: &BoundingBox,
) {
    let wcs = |record: &Option<WcsRecord>, relpath: &str| {
        record.as_ref().is_some_and(|r| r.covers(bbox)) && dir.join(relpath).exists()
    };
    let orthophoto = project.orthophoto.as_ref().is_some_and(|r| r.covers(bbox))
        && dir.join(WMS_ORTHOPHOTO_RELPATH).exists();
    for (part, stored) in [
        (Part::Terrain, wcs(&project.terrain, TERRAIN_RELPATH)),
        (Part::Orthophoto, orthophoto),
        (Part::Surface, wcs(&project.surface, SURFACE_RELPATH)),
    ] {
        if stored && !estimate.in_place.contains(&part) {
            estimate.in_place.push(part);
        }
    }
    let k5 = project.k5.as_ref().is_some_and(|r| r.covers(bbox)) && dir.join(K5_RELPATH).exists();
    // Stored layers count only when every record covers the area and its file is there: an
    // area moved within the state is downloaded again.
    let berlin_layers = !project.berlin_layers.is_empty()
        && project.berlin_layers.iter().all(|(key, record)| {
            record.covers(bbox)
                && known_berlin_layer(key).is_some_and(|l| berlin_layer_path(dir, l).exists())
        });
    let nas_layers = !project.nas_layers.is_empty()
        && project.nas_layers.iter().all(|(key, record)| {
            record.covers(bbox)
                && known_nas_layer(key).is_some_and(|l| nas_layer_path(dir, l).exists())
        });
    let tree_layers = !project.tree_layers.is_empty()
        && project.tree_layers.iter().all(|(key, record)| {
            record.covers(bbox)
                && known_tree_dataset(key).is_some_and(|d| tree_layer_path(dir, d).exists())
        });
    if orthophoto || k5 || berlin_layers || nas_layers || tree_layers {
        for item in &mut estimate.items {
            let stored = match (item.part, item.what.as_str()) {
                (Part::Orthophoto, _) => orthophoto,
                (Part::K5, _) => k5,
                (Part::BerlinLayers, _) => berlin_layers,
                (Part::StateLayers, crate::layers::NAS_LAYERS_ITEM) => nas_layers,
                (Part::StateLayers, crate::layers::TREE_LAYERS_ITEM) => tree_layers,
                _ => false,
            };
            if stored && item.store == crate::space::Store::Project {
                item.cached = true;
            }
        }
        estimate.total_megabytes = estimate
            .items
            .iter()
            .filter(|i| !i.cached)
            .map(|i| i.megabytes)
            .sum();
    }
}

/// Checks the downloads of an online fetch against the free disk space ([`crate::space::plan`]):
/// fails when a part the run needs does not fit, and warns for each part left out. `None` when
/// the free space cannot be read; the fetch then goes on unchecked.
fn check_space(
    dir: &Path,
    cache: &Cache,
    project: &ProjectFile,
    region_bbox: &BoundingBox,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<SpacePlan>, ProjectError> {
    let today = chrono::Local::now().date_naive();
    let alkis_cached = dir.join(BUILDINGS_RELPATH).exists() && dir.join(LAND_USE_RELPATH).exists();
    let mut estimate = crate::layers::estimate_area(region_bbox, bbox, cache, today, alkis_cached);
    mark_stored(&mut estimate, dir, project, bbox);
    let plan = match crate::space::plan(&estimate, cache.root(), dir, cache.disk_space()) {
        Ok(plan) => plan,
        Err((path, e)) => {
            progress(&format!(
                "warning: cannot read the free disk space of {} ({e}), so freemapper does not check it",
                path.display()
            ));
            return Ok(None);
        }
    };
    if let Some(missing) = plan.missing() {
        return Err(ProjectError::NotEnoughSpace(missing.clone()));
    }
    for skipped in plan.skipped() {
        progress(&format!("warning: {skipped}"));
    }
    Ok(Some(plan))
}

/// The LGB rasters of `bbox`: orthophoto and DTM tiles, the nDOM point clouds, and the bDOM where
/// no nDOM is published. With `skip_surface` (no disk space for it) neither surface model is
/// downloaded; the cached tiles recorded before stay.
#[allow(clippy::too_many_arguments)]
fn fetch_lgb_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    let tiles = raster_tiles(bbox, Provider::Lgb, 1, progress);
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
    if skip_surface {
        let cached = |tile: &str, path: &dyn Fn(TileId) -> PathBuf| {
            TileId::parse(tile).is_some_and(|t| tiles.contains(&t) && path(t).exists())
        };
        project
            .ndsm_tiles
            .retain(|r| cached(&r.tile, &|t| cache.ndsm_path(t, r.published)));
        project.tiles.extend(
            previous_dsm
                .into_iter()
                .filter(|r| cached(&r.tile, &|t| cache.tile_path(r.product, t, r.published))),
        );
        let kept: Vec<&str> = project
            .ndsm_tiles
            .iter()
            .map(|r| r.tile.as_str())
            .chain(
                project
                    .tiles
                    .iter()
                    .filter(|r| r.product == Product::ImageDsm)
                    .map(|r| r.tile.as_str()),
            )
            .collect();
        say_kept_surface_tiles(&kept, progress);
        drop_wms_orthophoto(dir, project);
        project.save(dir)?;
        return Ok(());
    }
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
    drop_wms_orthophoto(dir, project);
    project.save(dir)?;
    Ok(())
}

/// An orthophoto of an earlier map area in another state (GeoSN and the per-area providers) is
/// no longer used by an LGB project.
fn drop_wms_orthophoto(dir: &Path, project: &mut ProjectFile) {
    project.orthophoto = None;
    let wms = dir.join(WMS_ORTHOPHOTO_RELPATH);
    for stale in [wms.clone(), wms.with_extension("jgw")] {
        crate::fsutil::remove_inside(dir, &stale);
    }
}

/// The GeoSN rasters of `bbox`: DGM1 tiles, DOM1 tiles for the nDSM (each one optional: it is
/// only a template; with `skip_surface` only the cached ones recorded before), and the
/// orthophoto from the WMS.
#[allow(clippy::too_many_arguments)]
fn fetch_geosn_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    let tiles = raster_tiles(
        bbox,
        Provider::GeoSn,
        Product::SaxonyDtm.grid_km(),
        progress,
    );
    let previous = std::mem::take(&mut project.tiles);
    let mut records = Vec::new();
    for fetched in fetch_tiles(fetcher, cache, Product::SaxonyDtm, &tiles, mode, progress)? {
        records.push(tile_record(fetched, &previous, progress));
    }
    project.tiles = records;
    project.ndsm_tiles.clear();
    project.save(dir)?;
    let dsm = if skip_surface {
        let cached = previous.iter().filter(|r| {
            r.product == Product::SaxonyDsm
                && TileId::parse(&r.tile).is_some_and(|t| {
                    tiles.contains(&t) && cache.tile_path(r.product, t, r.published).exists()
                })
        });
        project.tiles.extend(cached.cloned());
        let kept: Vec<&str> = project
            .tiles
            .iter()
            .filter(|r| r.product == Product::SaxonyDsm)
            .map(|r| r.tile.as_str())
            .collect();
        say_kept_surface_tiles(&kept, progress);
        Ok(Vec::new())
    } else {
        fetch_tiles_each(fetcher, cache, Product::SaxonyDsm, &tiles, mode, progress)
    };
    match dsm {
        Ok(results) => {
            for (tile, result) in results {
                match result {
                    Ok(fetched) => project
                        .tiles
                        .push(tile_record(fetched, &previous, progress)),
                    Err(e) => progress(&format!(
                        "warning: {} of tile {} unavailable ({e}); no nDSM template there",
                        Product::SaxonyDsm,
                        tile.name()
                    )),
                }
            }
        }
        Err(e) => progress(&format!(
            "warning: {} unavailable ({e}); no nDSM template",
            Product::SaxonyDsm
        )),
    }
    let flight = || crate::sources::geosn::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(crate::sources::geosn::WMS),
            "GeoSN",
            &flight,
            None,
        ),
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The orthophoto of `bbox` from a WMS or WCS into [`WMS_ORTHOPHOTO_RELPATH`]: downloaded again only
/// when the flight date changed (the date tells whether the state flew the area again);
/// offline, the stored image must cover `bbox`. `label` names a date that is no flight date as
/// such, in the record and in the log.
fn fetch_wms_orthophoto(
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    (source, provider, flight_date, label): (
        &OrthophotoSource,
        &str,
        &dyn Fn() -> Option<String>,
        Option<&DateLabel>,
    ),
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    let path = dir.join(WMS_ORTHOPHOTO_RELPATH);
    match mode {
        FetchMode::Online => {
            let flight = flight_date();
            // Reused while the stored image covers the area and the flight date is the same.
            // Without a date from the service (none known, or the query failed), the image is
            // reused only while it is recent, and keeps the date it has.
            let today = chrono::Local::now().date_naive();
            let recent =
                |r: &WmsRecord| (today - r.fetched).num_days() < UNDATED_ORTHOPHOTO_MAX_AGE_DAYS;
            let unchanged = project.orthophoto.as_ref().is_some_and(|r| {
                r.covers(bbox)
                    && if flight.is_some() {
                        r.flight == flight
                    } else {
                        recent(r)
                    }
            }) && path.exists();
            if unchanged {
                let kept = flight
                    .as_deref()
                    .or_else(|| project.orthophoto.as_ref()?.flight.as_deref());
                let shown = label.map_or("flight", |l| l.shown);
                progress(&match kept {
                    Some(date) => format!("orthophoto unchanged ({shown} {date})"),
                    None => "orthophoto unchanged".to_string(),
                });
            } else {
                progress(&format!(
                    "downloading the orthophoto from the {provider} {}",
                    source.kind()
                ));
                crate::sources::wms::fetch_orthophoto_from(fetcher, source, bbox, dir, &path)?;
                let flight_label = label
                    .filter(|_| flight.is_some())
                    .map(|l| l.record.to_string());
                project.orthophoto = Some(WmsRecord {
                    flight,
                    flight_label,
                    fetched: chrono::Local::now().date_naive(),
                    bbox: [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n],
                });
            }
        }
        FetchMode::Offline => {
            let covered = project.orthophoto.as_ref().is_some_and(|r| r.covers(bbox));
            if !path.exists() || !covered {
                return Err(ProjectError::OrthophotoNotFetched);
            }
        }
    }
    Ok(())
}

/// The Berlin K5 map as the template [`K5_RELPATH`] (PNG + world file) over `bbox`, the map area
/// plus buffer in the map's CRS. Optional like the Berlin layers: a failed request or a full disk
/// (`skip`) leaves the template out with a warning, never fails the fetch. A stored template
/// that covers the area (same grid, see `check_template`) is kept. Outside Berlin (`region_bbox`), and in
/// any CRS but UTM zone 33 that the service is asked for, nothing is requested and an earlier
/// template is removed. Offline the stored template stays.
#[allow(clippy::too_many_arguments)]
fn fetch_k5_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    skip: bool,
    project: &mut ProjectFile,
    region_bbox: &BoundingBox,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) {
    if mode == FetchMode::Offline {
        return;
    }
    let path = dir.join(K5_RELPATH);
    let remove = |project: &mut ProjectFile| {
        for file in [path.clone(), path.with_extension("pgw")] {
            crate::fsutil::remove_inside(dir, &file);
        }
        project.k5 = None;
    };
    if !berlin::near_berlin(region_bbox) || project.crs() != Crs::Utm33 {
        remove(project);
        return;
    }
    if skip {
        // The disk plan already said that the template is left out.
        remove(project);
        return;
    }
    let pixel = crate::sources::wms::K5_PIXEL_M;
    let unchanged =
        project.k5.as_ref().is_some_and(|r| r.covers(bbox)) && check_template(&path, bbox).is_ok();
    if unchanged {
        progress("K5 template unchanged");
        return;
    }
    progress("downloading the K5 map (1:5000) from the Geoportal Berlin WMS");
    match crate::sources::wms::fetch_k5(fetcher, bbox, pixel, dir, &path) {
        Ok(()) => {
            project.k5 = Some(WmsRecord {
                flight: Some(K5_DATASET_DATE.to_string()),
                flight_label: Some("K5 dataset".to_string()),
                fetched: chrono::Local::now().date_naive(),
                bbox: [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n],
            });
        }
        Err(e) => {
            progress(&format!(
                "warning: the K5 map template is unavailable ({}); skipped",
                crate::fetch::one_line(&e.to_string(), 300)
            ));
            remove(project);
        }
    }
}

/// The warning for a map area that reaches beyond the union of the supported states: abroad, into
/// sea outside a state boundary, or into a German state that is no region yet (Bremen, Thuringia,
/// Bavaria, Baden-Württemberg, Saarland). The states' services stop at
/// their borders.
const BEYOND_COVERAGE_WARNING: &str = "warning: part of the map area lies outside the supported states. It lies abroad, at sea or in a state freemapper does not support yet, such as Bremen or Thuringia. The states have no data there, so the orthophoto can be white and ALKIS gives no buildings or land use";

/// How many days a stored orthophoto is reused while the service gives no flight date (none known,
/// or the query failed) before the next online fetch downloads it again.
const UNDATED_ORTHOPHOTO_MAX_AGE_DAYS: i64 = 30;

/// The LVermGeo rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM1 from
/// the WCS. The DOM1 is only a template: an unavailable one costs a warning.
fn fetch_lvermgeo_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::lvermgeo::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    // LVermGeo serves Saxony-Anhalt only; LGB and GeoSN report this through their missing tiles.
    warn_beyond_state(project, Provider::LvermGeo, "LVermGeo", progress)?;
    let flight = || lvermgeo::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(lvermgeo::WMS),
            "LVermGeo",
            &flight,
            None,
        ),
        progress,
    )?;
    let record = |date: Option<String>| WcsRecord {
        fetched: chrono::Local::now().date_naive(),
        bbox: [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n],
        date,
    };
    match mode {
        FetchMode::Online => {
            progress(&format!("downloading the {}", Model::Dgm1.label()));
            lvermgeo::fetch_model(fetcher, Model::Dgm1, bbox, dir, &dir.join(TERRAIN_RELPATH))?;
            project.terrain = Some(record(None));
            if skip_surface {
                keep_stored_surface(dir, mode, project, bbox, progress);
                project.save(dir)?;
                return Ok(());
            }
            progress(&format!("downloading the {}", Model::Dom1.label()));
            match lvermgeo::fetch_model(fetcher, Model::Dom1, bbox, dir, &dir.join(SURFACE_RELPATH))
            {
                Ok(()) => project.surface = Some(record(lvermgeo::surface_date(fetcher, bbox))),
                Err(e) => progress(&format!(
                    "warning: {} unavailable ({e}); no nDSM template",
                    Model::Dom1.label()
                )),
            }
        }
        FetchMode::Offline => {
            let stored = |r: &Option<WcsRecord>, relpath: &str| {
                r.as_ref().is_some_and(|r| r.covers(bbox)) && dir.join(relpath).exists()
            };
            if !stored(&project.terrain, TERRAIN_RELPATH) {
                return Err(ProjectError::TerrainNotFetched);
            }
            if !stored(&project.surface, SURFACE_RELPATH) {
                progress(
                    "warning: no surface model stored; run `freemapper fetch` online for the nDSM template",
                );
            }
        }
    }
    project.save(dir)?;
    Ok(())
}

/// The LGV rasters of `bbox` (EPSG:25832): the orthophoto from the WMS (2026), DGM1 and bDOM
/// (2022) read out of the statewide archives. The bDOM is only a template: an unavailable one
/// costs a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_lgv_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::lgv::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::Lgv, "LGV", progress)?;
    let flight = || lgv::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (&OrthophotoSource::Wms(lgv::WMS), "LGV", &flight, None),
        progress,
    )?;
    // Offline the tiles come from the cache: they were fetched when the record says.
    let record = |model: Model, old: &Option<WcsRecord>| WcsRecord {
        fetched: match (mode, old) {
            (FetchMode::Offline, Some(old)) => old.fetched,
            _ => chrono::Local::now().date_naive(),
        },
        bbox: [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n],
        date: Some(model.date().to_string()),
    };
    let warn_missing = |progress: &mut dyn FnMut(&str), model: Model, missing: Vec<_>| {
        if !missing.is_empty() {
            progress(&format!(
                "warning: the {} has no {} inside Hamburg; the templates stay empty there",
                model.label(),
                lgv::tile_names(&missing)
            ));
        }
    };
    progress(&format!("reading the {}", Model::Dgm1.label()));
    let missing = lgv::fetch_model(
        fetcher,
        cache,
        mode,
        Model::Dgm1,
        bbox,
        dir,
        &dir.join(TERRAIN_RELPATH),
    )?;
    warn_missing(progress, Model::Dgm1, missing);
    project.terrain = Some(record(Model::Dgm1, &project.terrain));
    if skip_surface {
        keep_stored_surface(dir, mode, project, bbox, progress);
        project.save(dir)?;
        return Ok(());
    }
    progress(&format!("reading the {}", Model::Bdom.label()));
    match lgv::fetch_model(
        fetcher,
        cache,
        mode,
        Model::Bdom,
        bbox,
        dir,
        &dir.join(SURFACE_RELPATH),
    ) {
        Ok(missing) => {
            warn_missing(progress, Model::Bdom, missing);
            project.surface = Some(record(Model::Bdom, &project.surface));
        }
        Err(e) => progress(&format!(
            "warning: {} unavailable ({e}); no nDSM template",
            Model::Bdom.label()
        )),
    }
    project.save(dir)?;
    Ok(())
}

/// Reports once that the map area reaches into a state other than `provider`'s. Whether the `name`
/// rasters end at the border depends on the service (the LGLN WMS and WCS cover the Hamburg corner
/// but the WMS is white in Bremen; the LVermGeo RP WMS is white across the Rhine at Mainz while
/// its tiles cover Wiesbaden; tiles may reach slightly past), so the line promises no empty
/// templates. ALKIS and OSM are read from both states.
fn warn_beyond_state(
    project: &ProjectFile,
    provider: Provider,
    name: &str,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    if regions_for(&project.region_bbox()?)
        .iter()
        .any(|r| r.provider() != provider)
    {
        progress(&format!(
            "the map area reaches beyond the state border; beyond it the templates are empty wherever the {name} orthophoto and height models end. ALKIS and OSM come from both states"
        ));
    }
    Ok(())
}

/// The record of a height model just read; offline the tiles come from the cache, so the record
/// keeps the date they were fetched.
fn height_record(
    mode: FetchMode,
    bbox: &BoundingBox,
    found: &crate::sources::height_model::HeightModel,
    old: &Option<WcsRecord>,
) -> WcsRecord {
    WcsRecord {
        fetched: match (mode, old) {
            (FetchMode::Offline, Some(old)) => old.fetched,
            _ => chrono::Local::now().date_naive(),
        },
        bbox: [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n],
        // Offline without the portal's tile list there is no date: the stored file is unchanged.
        date: found.date.clone().or_else(|| {
            old.as_ref()
                .filter(|_| mode == FetchMode::Offline)?
                .date
                .clone()
        }),
    }
}

/// The warning for the tiles of `label` that the provider lacks inside `state`.
fn warn_missing_tiles(
    progress: &mut dyn FnMut(&str),
    state: &str,
    label: &str,
    found: &crate::sources::height_model::HeightModel,
) {
    if !found.missing.is_empty() {
        progress(&format!(
            "warning: the {label} has no {} inside {state}; the templates stay empty there",
            crate::sources::lgv::tile_names(&found.missing)
        ));
    }
}

/// The HVBG rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM1 read out
/// of the Downloadcenter's municipal zips. The DOM1 is only a template: an unavailable one costs
/// a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_hvbg_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::hvbg::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::Hvbg, "HVBG", progress)?;
    let flight = || hvbg::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (&OrthophotoSource::Wms(hvbg::WMS), "HVBG", &flight, None),
        progress,
    )?;
    // What both models report alike is said once, after both: the places without a
    // Downloadcenter file (with the tiles that stayed empty for want of another file) and
    // the metadata list's problems.
    let mut unresolved: Vec<String> = Vec::new();
    let mut unresolved_empty: Vec<TileId> = Vec::new();
    let mut list_warnings: Vec<String> = Vec::new();
    let mut memo = hvbg::MetadataMemo::default();
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "Hesse",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            hvbg::fetch_model_with(fetcher, cache, mode, model, bbox, dir, out, &mut memo)
        },
        &mut |_, found, _, _| {
            for place in &found.unresolved {
                if !unresolved.contains(place) {
                    unresolved.push(place.clone());
                }
            }
            for tile in &found.unresolved_empty {
                if !unresolved_empty.contains(tile) {
                    unresolved_empty.push(*tile);
                }
            }
            for warning in &found.warnings {
                if !list_warnings.contains(warning) {
                    list_warnings.push(warning.clone());
                }
            }
        },
        progress,
    )?;
    for warning in &list_warnings {
        progress(&format!("warning: {warning}"));
    }
    if !unresolved.is_empty() {
        let places: Vec<String> = unresolved.iter().map(|p| format!("\"{p}\"")).collect();
        let empty = if unresolved_empty.is_empty() {
            String::new()
        } else {
            format!(
                "; {} stayed empty",
                crate::sources::lgv::tile_names(&unresolved_empty)
            )
        };
        progress(&format!(
            "warning: the Downloadcenter has no file for {}{empty}",
            places.join(", ")
        ));
    }
    project.save(dir)?;
    Ok(())
}

/// The Geobasis NRW rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM1
/// from the 1 km tiles. The DOM1 is only a template: an unavailable one costs a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_nrw_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::geobasis_nrw::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::GeobasisNrw, "Geobasis NRW", progress)?;
    let flight = || geobasis_nrw::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(geobasis_nrw::WMS),
            "Geobasis NRW",
            &flight,
            None,
        ),
        progress,
    )?;
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "North Rhine-Westphalia",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            geobasis_nrw::fetch_model(fetcher, cache, mode, model, bbox, dir, out)
        },
        &mut |_, found, _, progress| {
            for warning in &found.warnings {
                progress(&format!("warning: {warning}"));
            }
        },
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The LVermGeo RP rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, dated by the
/// `erstellung` of its block, and DGM1 and DOM1 from the 1 km tiles the `Laserbefliegung` list
/// names. The DOM1 is only a template: an unavailable one costs a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_rp_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::lvermgeo_rp::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    // The WMS is white beyond the state border, while the tiles reach across it.
    warn_beyond_state(project, Provider::LvermGeoRp, "LVermGeo RP", progress)?;
    let flight = || lvermgeo_rp::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(lvermgeo_rp::WMS),
            "LVermGeo RP",
            &flight,
            // The block's creation date, taken as the flight date: the record and the log say so.
            Some(&lvermgeo_rp::FLIGHT_LABEL),
        ),
        progress,
    )?;
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "the map area",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            lvermgeo_rp::fetch_model(fetcher, cache, mode, model, bbox, dir, out)
        },
        &mut |_, found, _, progress| {
            for warning in &found.warnings {
                progress(&format!("warning: {warning}"));
            }
        },
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The LVermGeo SH rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and bDOM
/// from the Downloadportal. The bDOM is only a template (and a 105 MB download per km²): an
/// unavailable one costs a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_sh_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::lvermgeo_sh::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::LvermGeoSh, "LVermGeo SH", progress)?;
    let flight = || lvermgeo_sh::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(lvermgeo_sh::WMS),
            "LVermGeo SH",
            &flight,
            None,
        ),
        progress,
    )?;
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "Schleswig-Holstein",
        (Model::Dgm1.label(), Model::Bdom.label()),
        &mut |surface, out, progress| {
            let model = if surface { Model::Bdom } else { Model::Dgm1 };
            lvermgeo_sh::fetch_model(fetcher, cache, mode, model, bbox, dir, out, progress)
        },
        &mut |surface, found, project, progress| {
            for warning in &found.warnings {
                progress(&format!("warning: {warning}"));
            }
            if !surface {
                return;
            }
            let year = |date: &str, flight_style: bool| -> Option<i32> {
                let part = if flight_style {
                    date.rsplit('.').next()
                } else {
                    date.split('-').next()
                };
                part?.parse().ok()
            };
            let flight_year = project
                .orthophoto
                .as_ref()
                .and_then(|o| o.flight.as_deref())
                .and_then(|d| year(d, true));
            let surface_year = found.date.as_deref().and_then(|d| year(d, false));
            if let (Some(photo), Some(surface)) = (flight_year, surface_year)
                && photo - surface >= 1
            {
                progress(&format!(
                    "warning: the surface model (bDOM {surface}) is older than the orthophoto ({photo}); new buildings may be missing in the nDSM"
                ));
            }
        },
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The LAiV M-V rasters of `bbox` (EPSG:25833): the orthophoto from the DOP WCS, DGM1 and DOM1
/// from the height WCS, masked to the MV tile set. The DOM1 is only a template: an unavailable
/// one costs a warning.
#[allow(clippy::too_many_arguments)]
fn fetch_mv_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::laiv_mv::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::LaivMv, "LAiV M-V", progress)?;
    let flight = || laiv_mv::flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (
            &OrthophotoSource::Wcs(laiv_mv::DOP),
            "LAiV M-V",
            &flight,
            None,
        ),
        progress,
    )?;
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "Mecklenburg-Vorpommern",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            laiv_mv::fetch_model(fetcher, cache, mode, model, bbox, dir, out)
        },
        &mut |_, found, _, progress| {
            for warning in &found.warnings {
                progress(&format!("warning: {warning}"));
            }
        },
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The LGLN rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM1 from the
/// WCS, all dated by the STAC searches. The DOM1 is only a template: an unavailable one costs a
/// warning, and so does one much older than the orthophoto.
fn fetch_lgln_rasters(
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    skip_surface: bool,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    use crate::sources::lgln::{self, Model};
    project.tiles.clear();
    project.ndsm_tiles.clear();
    warn_beyond_state(project, Provider::Lgln, "LGLN", progress)?;
    // One DOP search gives the flight date and the leaf state; offline nothing is asked.
    let (flown, missing) = match mode {
        FetchMode::Online => match lgln::orthophoto_date(fetcher, bbox) {
            Ok(Some(date)) => (Some(date), None),
            Ok(None) => (
                None,
                Some("the STAC lists no image at the centre of the map area".to_string()),
            ),
            Err(e) => (
                None,
                Some(format!("the STAC search gave no usable answer ({e})")),
            ),
        },
        FetchMode::Offline => (None, None),
    };
    let flight_text = flown.map(|date| date.format("%d.%m.%Y").to_string());
    let flight = || flight_text.clone();
    fetch_wms_orthophoto(
        dir,
        fetcher,
        mode,
        project,
        bbox,
        (&OrthophotoSource::Wms(lgln::WMS), "LGLN", &flight, None),
        progress,
    )?;
    // Without a fresh date a recent stored orthophoto is kept with its own (see
    // `fetch_wms_orthophoto`); the warning says which of the two happened.
    let stored = project.orthophoto.as_ref().and_then(|o| o.flight.clone());
    if let Some(reason) = missing {
        progress(&match &stored {
            Some(date) => format!(
                "warning: orthophoto (LGLN WMS): {reason}; kept the stored orthophoto and its flight date {date}"
            ),
            None => format!("warning: orthophoto (LGLN WMS): no flight date, {reason}"),
        });
    }
    let shown = stored
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%d.%m.%Y").ok());
    if let (FetchMode::Online, Some(date)) = (mode, shown) {
        progress(&lgln::leaf_state(date));
    }
    fetch_area_heights(
        dir,
        (mode, skip_surface),
        project,
        bbox,
        "Lower Saxony",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            lgln::fetch_model(fetcher, mode, model, bbox, dir, out)
        },
        &mut |surface, found, project, progress| {
            for warning in &found.warnings {
                progress(&format!("warning: {warning}"));
            }
            if !surface {
                return;
            }
            // The stored flight date also covers an orthophoto kept after a failed search.
            let flight = project
                .orthophoto
                .as_ref()
                .and_then(|o| o.flight.as_deref())
                .and_then(|d| NaiveDate::parse_from_str(d, "%d.%m.%Y").ok());
            let scanned = found
                .date
                .as_deref()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            if let (Some(scanned), Some(flight)) = (scanned, flight)
                && let Some(warning) = lgln::surface_age_warning(scanned, flight)
            {
                progress(&format!("warning: {warning}"));
            }
        },
        progress,
    )?;
    project.save(dir)?;
    Ok(())
}

/// The terrain model (required) and the surface model (a template: a failure costs a warning;
/// with `skip_surface` it is not read, and the stored one stays while it covers `bbox`) of a
/// per-area provider whose fetch reports a [`HeightModel`](crate::sources::height_model::HeightModel).
/// `fetch(surface, out, progress)` reads one model into `out`; `after(surface, found, project, progress)`
/// says what is specific to the provider once a model was read.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fetch_area_heights(
    dir: &Path,
    (mode, skip_surface): (FetchMode, bool),
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    state: &str,
    (terrain_label, surface_label): (&str, &str),
    fetch: &mut dyn FnMut(
        bool,
        &Path,
        &mut dyn FnMut(&str),
    ) -> Result<crate::sources::height_model::HeightModel, FetchError>,
    after: &mut dyn FnMut(
        bool,
        &crate::sources::height_model::HeightModel,
        &ProjectFile,
        &mut dyn FnMut(&str),
    ),
    progress: &mut dyn FnMut(&str),
) -> Result<(), ProjectError> {
    progress(&format!("reading the {terrain_label}"));
    let found = fetch(false, &dir.join(TERRAIN_RELPATH), progress)?;
    warn_missing_tiles(progress, state, terrain_label, &found);
    after(false, &found, project, progress);
    project.terrain = Some(height_record(mode, bbox, &found, &project.terrain));
    if skip_surface {
        keep_stored_surface(dir, mode, project, bbox, progress);
        project.save(dir)?;
        return Ok(());
    }
    progress(&format!("reading the {surface_label}"));
    match fetch(true, &dir.join(SURFACE_RELPATH), progress) {
        Ok(found) => {
            warn_missing_tiles(progress, state, surface_label, &found);
            after(true, &found, project, progress);
            project.surface = Some(height_record(mode, bbox, &found, &project.surface));
        }
        Err(e) => keep_surface(dir, mode, project, bbox, surface_label, &e, progress),
    }
    project.save(dir)?;
    Ok(())
}

/// After the surface model `label` of a per-area provider failed with `e`: the stored one is
/// kept when its record still covers `bbox` (derive uses it), and dropped with its file when it
/// does not; offline nothing is deleted (derive ignores a record that does not cover the area).
fn keep_surface(
    dir: &Path,
    mode: FetchMode,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    label: &str,
    e: &dyn std::fmt::Display,
    progress: &mut dyn FnMut(&str),
) {
    match stored_surface_or_drop(dir, mode, project, bbox) {
        Some(fetched) => progress(&format!(
            "warning: {label} unavailable ({e}); kept the surface model fetched on {fetched}"
        )),
        None => progress(&format!(
            "warning: {label} unavailable ({e}); no nDSM template"
        )),
    }
}

/// The surface model of a per-area provider left out for lack of disk space: the stored one stays
/// while it covers `bbox`, and the user hears so.
fn keep_stored_surface(
    dir: &Path,
    mode: FetchMode,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) {
    if let Some(fetched) = stored_surface_or_drop(dir, mode, project, bbox) {
        progress(&format!(
            "freemapper keeps the surface model fetched on {fetched} for the nDSM template"
        ));
    }
}

/// After the surface model of a tile provider was left out: the cached tiles that still serve.
fn say_kept_surface_tiles(tiles: &[&str], progress: &mut dyn FnMut(&str)) {
    if !tiles.is_empty() {
        let noun = if tiles.len() == 1 { "tile" } else { "tiles" };
        progress(&format!(
            "freemapper keeps the cached surface model of {noun} {} for the nDSM template",
            tiles.join(", ")
        ));
    }
}

/// The surface model of a per-area provider that is not read this time: the stored one stays
/// while its record covers `bbox` (the day it was fetched is returned); else it is dropped with
/// its file, online only (offline derive ignores a record that does not cover the area).
fn stored_surface_or_drop(
    dir: &Path,
    mode: FetchMode,
    project: &mut ProjectFile,
    bbox: &BoundingBox,
) -> Option<NaiveDate> {
    let stored = project
        .surface
        .as_ref()
        .filter(|r| r.covers(bbox) && dir.join(SURFACE_RELPATH).exists());
    if let Some(record) = stored {
        return Some(record.fetched);
    }
    if mode == FetchMode::Online {
        project.surface = None;
        let surface = dir.join(SURFACE_RELPATH);
        for file in [surface.clone(), surface.with_extension("tfw")] {
            crate::fsutil::remove_inside(dir, &file);
        }
    }
    None
}

/// The tiles of the provider's `km` grid over `bbox` that touch its regions; the others are not
/// published, and the templates stay empty there (reported once).
fn raster_tiles(
    bbox: &BoundingBox,
    provider: Provider,
    km: u32,
    progress: &mut dyn FnMut(&str),
) -> Vec<TileId> {
    if provider.per_area() {
        // No 25833 tile grid (Hesse even has negative eastings, which a `TileId` cannot hold).
        return Vec::new();
    }
    let tiles = provider_tiles(bbox, provider, km);
    let outside: Vec<TileId> = tiles_covering_grid(bbox, km)
        .into_iter()
        .filter(|t| !tiles.contains(t))
        .collect();
    if !outside.is_empty() {
        progress(&format!(
            "the map area reaches beyond the state border; no {} data for {}, so the templates stay empty there",
            match provider {
                Provider::Lgb => "LGB",
                Provider::GeoSn => "GeoSN",
                Provider::LvermGeo => "LVermGeo",
                Provider::Lgv => "LGV",
                Provider::Hvbg => "HVBG",
                Provider::LvermGeoSh => "LVermGeo SH",
                Provider::LaivMv => "LAiV M-V",
                Provider::GeobasisNrw => "Geobasis NRW",
                Provider::Lgln => "LGLN",
                Provider::LvermGeoRp => "LVermGeo RP",
            },
            crate::fetch::tile_names(&outside)
        ));
    }
    tiles
}

/// The record of a fetched tile; warns when the tile was republished since `previous`.
fn tile_record(
    fetched: FetchedTile,
    previous: &[TileRecord],
    progress: &mut dyn FnMut(&str),
) -> TileRecord {
    let record = TileRecord {
        product: fetched.product,
        tile: fetched.tile.name(),
        url: fetched.url,
        currency: fetched.currency.currency(),
        published: fetched.currency.published,
    };
    if let Some(old) = previous.iter().find(|p| {
        p.product == record.product && p.tile == record.tile && p.published != record.published
    }) {
        progress(&format!(
            "warning: {:?} tile {} changed upstream ({} → {})",
            record.product, record.tile, old.published, record.published
        ));
    }
    record
}

/// Reads the candidate features of the map area plus buffer from every OSM extract the area needs
/// ([`extracts_for`]: one per state, one per North Rhine-Westphalia government district) into
/// [`OSM_CANDIDATES_RELPATH`]; a feature in two extracts is kept once. Offline from the cached
/// extracts, and so is every extract with a cached copy when `skip_update` (no disk space for a
/// newer one).
fn fetch_osm_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip_update: bool,
    project: &ProjectFile,
    progress: &mut dyn FnMut(&str),
) -> Result<OsmRecord, ProjectError> {
    let area = MapArea::from_geojson(&project.map_area.to_string())?.geo_box(project.buffer_m);
    let mut extracts = Vec::new();
    for source in extracts_for(&project.region_bbox()?, progress) {
        let mode = if skip_update && cached_extract_date(cache, source).is_some() {
            FetchMode::Offline
        } else {
            mode
        };
        match fetch_extract(fetcher, cache, source, mode, progress) {
            Ok(extract) => extracts.push((source, extract)),
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
            Err(e) => return Err(e.into()),
        }
    }
    let rules = Rules::embedded();
    let mut parts = Vec::new();
    for (source, extract) in &extracts {
        progress(&format!(
            "reading OSM extract {} from {}",
            source.label(),
            extract.data_date
        ));
        parts.push(read_extract(&extract.path, &area, &|tags| {
            rules.wants(tags, project.map_kind)
        })?);
    }
    let features = merge_features(parts);
    let path = dir.join(OSM_CANDIDATES_RELPATH);
    write_inside(dir, &path, features_to_geojson(&features).as_bytes())?;
    progress(&format!("{} OSM candidate features", features.len()));
    let extract_date = extracts
        .iter()
        .map(|(_, e)| e.data_date)
        .min()
        .expect("every map area touches a region");
    Ok(OsmRecord {
        extract: extract_date,
        area: [area.min_lat, area.min_lon, area.max_lat, area.max_lon],
        count: features.len(),
    })
}

/// The features of several extracts, each OSM object once, nodes first, each kind by id.
fn merge_features(parts: Vec<Vec<OsmFeature>>) -> Vec<OsmFeature> {
    let mut seen = std::collections::HashSet::new();
    let mut features: Vec<OsmFeature> = parts
        .into_iter()
        .flatten()
        .filter(|f| seen.insert(f.id.clone()))
        .collect();
    let key = |f: &OsmFeature| {
        let (kind, id) = f.id.split_once('/').unwrap_or(("", "0"));
        (kind != "node", id.parse::<i64>().unwrap_or(0))
    };
    features.sort_by_key(key);
    features
}

/// Downloads the ALKIS buildings in `bbox` to [`BUILDINGS_RELPATH`] and returns the new record;
/// the stored file is only replaced by an answer that parses. Offline, the stored file must
/// exist and, if recorded, cover `bbox`; nothing new is recorded then.
fn fetch_buildings_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<AlkisRecord>, ProjectError> {
    let bbox = &area.bbox;
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
    let text = fetch_buildings_in(fetcher, area)?;
    let parsed = parse_buildings(&text).map_err(FetchError::from)?;
    write_inside(dir, &path, text.as_bytes())?;
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
    area: &FetchArea,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> Option<Option<AlkisRecord>> {
    let bbox = &area.bbox;
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
    let downloaded = fetch_land_use_in(fetcher, area).and_then(|text| {
        let land_use = parse_land_use(&text).map_err(FetchError::from)?;
        write_inside(dir, &dir.join(LAND_USE_RELPATH), text.as_bytes())?;
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
                    "warning: ALKIS land use unavailable ({e}); the stored land use no longer covers the map area, so freemapper removed it"
                ));
                crate::fsutil::remove_inside(dir, &path);
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

/// The NAS layer the project file records under `key`, from the static table; `None` for a key
/// no layer has.
fn known_nas_layer(key: &str) -> Option<&'static alkis_nas::NasLayer> {
    alkis_nas::NAS_LAYERS.iter().find(|layer| layer.file == key)
}

/// Path of the stored NAS `layer`: from a layer of the table, never from a name read from the
/// project file (ADR 0059).
fn nas_layer_path(dir: &Path, layer: &alkis_nas::NasLayer) -> PathBuf {
    dir.join(NAS_LAYERS_DIR)
        .join(format!("{}.geojson", layer.file))
}

/// Removes the stored NAS layer the project file records under `key`: only a layer of the
/// table, and only when its file lies inside the project folder.
fn remove_nas_layer(dir: &Path, key: &str) {
    let Some(layer) = known_nas_layer(key) else {
        return;
    };
    crate::fsutil::remove_inside(dir, &nas_layer_path(dir, layer));
}

/// The extra Berlin layer the project file records under `key`, from the static table; `None`
/// for a key no layer has.
fn known_berlin_layer(key: &str) -> Option<&'static berlin::ExtraLayer> {
    berlin::EXTRA_LAYERS.iter().find(|layer| layer.file == key)
}

/// Path of the stored extra Berlin `layer`. It takes a layer of the table, never a name read from
/// the project file, so the path is always one freemapper built (ADR 0059).
fn berlin_layer_path(dir: &Path, layer: &berlin::ExtraLayer) -> PathBuf {
    dir.join(BERLIN_LAYERS_DIR)
        .join(format!("{}.geojson", layer.file))
}

/// Removes the stored layer the project file records under `key`: only a layer of the table, and
/// only when its file lies inside the project folder (no link leads out of it).
fn remove_berlin_layer(dir: &Path, key: &str) {
    let Some(layer) = known_berlin_layer(key) else {
        return;
    };
    let path = berlin_layer_path(dir, layer);
    crate::fsutil::remove_inside(dir, &path);
}

/// Downloads the extra Berlin layers of `kind` in `region_bbox` to [`BERLIN_LAYERS_DIR`] and
/// returns the records that replace `previous`. They only give candidates and a few map
/// objects, so a layer that fails never fails the fetch: a stored file that still covers the
/// area stays, one that does not is removed; each case is reported. A layer without objects is
/// stored as an empty collection. Outside Berlin nothing is requested and stored layers of an
/// earlier area are removed. Offline, or when the disk plan left the layers out (`skip`), nothing
/// is downloaded and `previous` stays.
#[allow(clippy::too_many_arguments)]
fn fetch_berlin_layers_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
    mode: FetchMode,
    skip: bool,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> BTreeMap<String, AlkisRecord> {
    if mode == FetchMode::Offline {
        if berlin::near_berlin(region_bbox) && previous.is_empty() {
            progress(
                "no Berlin ALKIS structures stored; run `freemapper fetch` online for stairs, walls, trees and canopies",
            );
        }
        return previous.clone();
    }
    if !berlin::near_berlin(region_bbox) {
        if !previous.is_empty() {
            progress("removing the stored Berlin layers: the map area is no longer near Berlin");
            for key in previous.keys() {
                remove_berlin_layer(dir, key);
            }
        }
        return BTreeMap::new();
    }
    if skip {
        // The disk plan already said that the layers are left out; stored ones stay.
        return previous.clone();
    }
    progress("downloading Berlin ALKIS structures, tree cadastre, street survey and point layers");
    let bbox = [
        region_bbox.min_e.floor(),
        region_bbox.min_n.floor(),
        region_bbox.max_e.ceil(),
        region_bbox.max_n.ceil(),
    ];
    let mut records = BTreeMap::new();
    for fetched in berlin::fetch_extra_layers(fetcher, region_bbox, kind) {
        let file = fetched.layer.file;
        let path = berlin_layer_path(dir, fetched.layer);
        let stored = fetched
            .features
            .map_err(|e| crate::fetch::one_line(&e.to_string(), 300))
            .and_then(|features| {
                if !crate::fsutil::stays_inside(dir, &path) {
                    return Err(format!(
                        "{BERLIN_LAYERS_DIR} leads out of the project folder"
                    ));
                }
                write_inside(dir, &path, berlin::layer_to_geojson(&features).as_bytes())
                    .map_err(|e| e.to_string())?;
                Ok(features.len())
            });
        match stored {
            Ok(count) => {
                progress(&format!("{count} Berlin {file} features"));
                records.insert(
                    file.to_string(),
                    AlkisRecord {
                        bbox,
                        fetched: chrono::Local::now().date_naive(),
                        latest_update: None,
                        count,
                        regions: vec![Region::Berlin],
                    },
                );
            }
            Err(e) => match previous
                .get(file)
                .filter(|r| path.exists() && r.covers(region_bbox))
            {
                Some(record) => {
                    progress(&format!(
                        "warning: Berlin {file} unavailable ({e}); keeping the stored layer"
                    ));
                    records.insert(file.to_string(), record.clone());
                }
                None => {
                    // Nothing was written for it; a stale file is removed with the unused layers.
                    progress(&format!(
                        "warning: Berlin {file} unavailable ({e}); layer skipped"
                    ));
                }
            },
        }
    }
    // Layers the map kind no longer uses.
    for key in previous.keys().filter(|f| !records.contains_key(*f)) {
        remove_berlin_layer(dir, key);
    }
    records
}

/// Downloads the NAS layers (full ALKIS) of `kind` for `area` to [`NAS_LAYERS_DIR`] and returns
/// the records that replace `previous`, as [`fetch_berlin_layers_into`] does for the Berlin
/// layers: a layer that fails is a warning and never fails the fetch (a stored file that still
/// covers the area stays), a layer without objects is stored as an empty collection, outside the
/// states with a NAS service nothing is requested and stored layers of an earlier area are
/// removed. Offline, or when the disk plan left the state layers out (`skip`), nothing is
/// downloaded and `previous` stays.
#[allow(clippy::too_many_arguments)]
fn fetch_nas_layers_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    kind: MapKind,
    mode: FetchMode,
    skip: bool,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> BTreeMap<String, AlkisRecord> {
    let in_nas_state = !alkis_nas::nas_services_for(&area.region_bbox).is_empty();
    if mode == FetchMode::Offline {
        if in_nas_state && previous.is_empty() {
            progress(
                "no full ALKIS structures stored; run `freemapper fetch` online for walls, fences, hedges and bridges",
            );
        }
        return previous.clone();
    }
    if !in_nas_state {
        if !previous.is_empty() {
            progress(
                "removing the stored full ALKIS layers: the map area is no longer in a state with a NAS service",
            );
            for key in previous.keys() {
                remove_nas_layer(dir, key);
            }
        }
        return BTreeMap::new();
    }
    if skip {
        // The disk plan already said that the layers are left out; stored ones stay.
        return previous.clone();
    }
    // Answers stream into a part file beside the layers with a fresh name, its folder checked
    // right before each download (ADR 0060), removed after each one.
    let layers_dir = dir.join(NAS_LAYERS_DIR);
    if crate::fsutil::check_folder_inside(dir, &layers_dir).is_err() {
        progress(&format!(
            "warning: full ALKIS layers skipped ({NAS_LAYERS_DIR} leads out of the project folder)"
        ));
        return BTreeMap::new();
    }
    // Only after the check: a linked folder never has its part files removed.
    crate::fsutil::remove_stale_parts(&layers_dir, crate::fetch::STALE_PART_AGE);
    let part = || -> Result<PathBuf, FetchError> {
        let part = crate::fsutil::fresh_part_name(&layers_dir.join("answer.gml"));
        crate::fsutil::check_inside(dir, &part).map_err(|(path, e)| FetchError::Io(path, e))?;
        Ok(part)
    };
    progress("downloading the full ALKIS structures (NAS)");
    let bbox = &area.bbox;
    let rounded = [
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil(),
    ];
    let mut records = BTreeMap::new();
    for fetched in alkis_nas::fetch_nas_layers(fetcher, area, kind, &part) {
        let file = fetched.layer.file;
        let path = nas_layer_path(dir, fetched.layer);
        let stored = fetched
            .features
            .map_err(|e| crate::fetch::one_line(&e.to_string(), 300))
            .and_then(|fetched| {
                if !crate::fsutil::stays_inside(dir, &path) {
                    return Err(format!("{NAS_LAYERS_DIR} leads out of the project folder"));
                }
                let geojson = berlin::layer_to_geojson(&fetched.features);
                write_inside(dir, &path, geojson.as_bytes()).map_err(|e| e.to_string())?;
                Ok(fetched)
            });
        match stored {
            Ok(fetched) => {
                let count = fetched.features.len();
                progress(&format!("{count} ALKIS NAS {file} features"));
                if !fetched.skipped.is_empty() {
                    let shown = fetched.skipped.iter().take(SKIPPED_SHOWN);
                    let more = fetched.skipped.len() > SKIPPED_SHOWN;
                    progress(&format!(
                        "skipped {} ALKIS NAS {file} features: {}{}",
                        fetched.skipped.len(),
                        shown
                            .map(|s| crate::fetch::one_line(s, 120))
                            .collect::<Vec<_>>()
                            .join("; "),
                        if more { "; …" } else { "" }
                    ));
                }
                let latest_update = fetched
                    .features
                    .iter()
                    .filter_map(|f| f["properties"]["beginnt"].as_str()?.parse().ok())
                    .max();
                records.insert(
                    file.to_string(),
                    AlkisRecord {
                        bbox: rounded,
                        fetched: chrono::Local::now().date_naive(),
                        latest_update,
                        count,
                        regions: fetched.regions,
                    },
                );
            }
            Err(e) => match previous
                .get(file)
                .filter(|r| path.exists() && r.covers(bbox))
            {
                Some(record) => {
                    progress(&format!(
                        "warning: ALKIS NAS {file} unavailable ({e}); keeping the stored layer"
                    ));
                    records.insert(file.to_string(), record.clone());
                }
                None => {
                    // Nothing was written for it; a stale file is removed with the unused layers.
                    progress(&format!(
                        "warning: ALKIS NAS {file} unavailable ({e}); layer skipped"
                    ));
                }
            },
        }
    }
    // Layers the map kind no longer uses, and failed ones that no longer cover the area.
    for key in previous.keys().filter(|f| !records.contains_key(*f)) {
        remove_nas_layer(dir, key);
    }
    records
}

/// One stored NAS layer (full ALKIS).
#[derive(Debug, Clone)]
pub struct StoredNasLayer {
    pub layer: &'static alkis_nas::NasLayer,
    /// The features in the map's CRS, as [`alkis_nas::parse_nas_gml`] reads them (`oid`,
    /// `nas_type`, the codes, `beginnt`, `gehoert_zu`).
    pub features: Vec<serde_json::Value>,
    /// The fetch date, the currency of features without `beginnt`.
    pub fetched: NaiveDate,
}

/// The stored NAS layers of the map kind that cover the map area plus buffer, in table order. A
/// layer that was not fetched, is missing or does not cover the area is left out (optional
/// sources: `fetch` reports this, the export goes on without them). Empty outside the states
/// with a NAS service.
pub fn stored_nas_layers(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Vec<StoredNasLayer>, ProjectError> {
    let bbox = project.bbox()?;
    let mut layers = Vec::new();
    for layer in alkis_nas::nas_layers(project.map_kind) {
        let path = nas_layer_path(dir, layer);
        let Some(record) = project
            .nas_layers
            .get(layer.file)
            .filter(|r| path.exists() && r.covers(&bbox))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
        let features = berlin::parse_layer(&text).map_err(|e| ProjectError::StoredCandidates {
            path: path.clone(),
            message: e.to_string(),
        })?;
        layers.push(StoredNasLayer {
            layer,
            features,
            fetched: record.fetched,
        });
    }
    Ok(layers)
}

/// The tree dataset the project file records under `key`, from the static table; `None` for a
/// key no dataset has.
fn known_tree_dataset(key: &str) -> Option<&'static trees::TreeDataset> {
    trees::TREE_DATASETS
        .iter()
        .find(|dataset| dataset.id == key)
}

/// Path of the stored tree `dataset`: from a dataset of the table, never from a name read from
/// the project file (ADR 0059).
fn tree_layer_path(dir: &Path, dataset: &trees::TreeDataset) -> PathBuf {
    dir.join(TREE_LAYERS_DIR)
        .join(format!("{}.geojson", dataset.id))
}

/// Removes the stored tree dataset the project file records under `key`: only a dataset of the
/// table, and only when its file lies inside the project folder.
fn remove_tree_layer(dir: &Path, key: &str) {
    let Some(dataset) = known_tree_dataset(key) else {
        return;
    };
    crate::fsutil::remove_inside(dir, &tree_layer_path(dir, dataset));
}

/// Downloads the city tree cadastres for `area` to [`TREE_LAYERS_DIR`] and returns the records
/// that replace `previous`, as [`fetch_nas_layers_into`] does for the NAS layers: a dataset that
/// fails is a warning and never fails the fetch (a stored file that still covers the area
/// stays), a dataset without trees is stored as an empty collection, outside the envelopes of
/// the table nothing is requested and stored datasets of an earlier area are removed. Offline,
/// or when the disk plan left the state layers out (`skip`), nothing is downloaded and
/// `previous` stays.
fn fetch_tree_layers_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    mode: FetchMode,
    skip: bool,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(&str),
) -> BTreeMap<String, AlkisRecord> {
    let in_city = !trees::tree_datasets_for(&area.region_bbox).is_empty();
    if mode == FetchMode::Offline {
        if in_city && previous.is_empty() {
            progress(
                "no city tree cadastre stored; run `freemapper fetch` online for the single trees",
            );
        }
        return previous.clone();
    }
    if !in_city {
        if !previous.is_empty() {
            progress(
                "removing the stored tree cadastres: the map area is no longer in a city with one",
            );
            for key in previous.keys() {
                remove_tree_layer(dir, key);
            }
        }
        return BTreeMap::new();
    }
    if skip {
        // The disk plan already said that the datasets are left out; stored ones stay.
        return previous.clone();
    }
    // Answers stream into a part file beside the datasets, as for the NAS layers (ADR 0060).
    let trees_dir = dir.join(TREE_LAYERS_DIR);
    if crate::fsutil::check_folder_inside(dir, &trees_dir).is_err() {
        progress(&format!(
            "warning: city tree cadastres skipped ({TREE_LAYERS_DIR} leads out of the project folder)"
        ));
        return BTreeMap::new();
    }
    crate::fsutil::remove_stale_parts(&trees_dir, crate::fetch::STALE_PART_AGE);
    let part = || -> Result<PathBuf, FetchError> {
        let part = crate::fsutil::fresh_part_name(&trees_dir.join("answer.geojson"));
        crate::fsutil::check_inside(dir, &part).map_err(|(path, e)| FetchError::Io(path, e))?;
        Ok(part)
    };
    progress("downloading the city tree cadastres");
    let bbox = &area.bbox;
    let rounded = [
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil(),
    ];
    let mut records = BTreeMap::new();
    for fetched in trees::fetch_tree_layers(fetcher, area, &part) {
        let id = fetched.dataset.id;
        let path = tree_layer_path(dir, fetched.dataset);
        let stored = fetched
            .features
            .map_err(|e| crate::fetch::one_line(&e.to_string(), 300))
            .and_then(|fetched| {
                if !crate::fsutil::stays_inside(dir, &path) {
                    return Err(format!("{TREE_LAYERS_DIR} leads out of the project folder"));
                }
                let geojson = berlin::layer_to_geojson(&fetched.features);
                write_inside(dir, &path, geojson.as_bytes()).map_err(|e| e.to_string())?;
                Ok(fetched)
            });
        match stored {
            Ok(fetched) => {
                let count = fetched.features.len();
                progress(&format!("{count} {id} trees"));
                if !fetched.skipped.is_empty() {
                    let shown = fetched.skipped.iter().take(SKIPPED_SHOWN);
                    let more = fetched.skipped.len() > SKIPPED_SHOWN;
                    progress(&format!(
                        "skipped {} {id} trees: {}{}",
                        fetched.skipped.len(),
                        shown
                            .map(|s| crate::fetch::one_line(s, 120))
                            .collect::<Vec<_>>()
                            .join("; "),
                        if more { "; …" } else { "" }
                    ));
                }
                let latest_update = fetched
                    .features
                    .iter()
                    .filter_map(|f| f["properties"]["tree_date"].as_str()?.parse().ok())
                    .max();
                records.insert(
                    id.to_string(),
                    AlkisRecord {
                        bbox: rounded,
                        fetched: chrono::Local::now().date_naive(),
                        latest_update,
                        count,
                        regions: fetched.regions,
                    },
                );
            }
            Err(e) => match previous.get(id).filter(|r| path.exists() && r.covers(bbox)) {
                Some(record) => {
                    progress(&format!(
                        "warning: {id} tree cadastre unavailable ({e}); keeping the stored one"
                    ));
                    records.insert(id.to_string(), record.clone());
                }
                None => {
                    progress(&format!(
                        "warning: {id} tree cadastre unavailable ({e}); trees skipped"
                    ));
                }
            },
        }
    }
    // Datasets of an earlier area, and failed ones that no longer cover the area.
    for key in previous.keys().filter(|f| !records.contains_key(*f)) {
        remove_tree_layer(dir, key);
    }
    records
}

/// One stored tree dataset.
#[derive(Debug, Clone)]
pub struct StoredTreeLayer {
    pub dataset: &'static trees::TreeDataset,
    /// The trees as [`trees::parse_trees`] stores them: points in the map's CRS with the
    /// `tree_*` properties.
    pub features: Vec<serde_json::Value>,
    /// The fetch date, the currency of a tree without `tree_date`.
    pub fetched: NaiveDate,
}

/// The stored tree datasets that cover the map area plus buffer, in table order. A dataset that
/// was not fetched, is missing or does not cover the area is left out (optional sources: `fetch`
/// reports this, the export goes on without them).
pub fn stored_tree_layers(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Vec<StoredTreeLayer>, ProjectError> {
    let bbox = project.bbox()?;
    let mut layers = Vec::new();
    for dataset in &trees::TREE_DATASETS {
        let path = tree_layer_path(dir, dataset);
        let Some(record) = project
            .tree_layers
            .get(dataset.id)
            .filter(|r| path.exists() && r.covers(&bbox))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
        let features = berlin::parse_layer(&text).map_err(|e| ProjectError::StoredCandidates {
            path: path.clone(),
            message: e.to_string(),
        })?;
        layers.push(StoredTreeLayer {
            dataset,
            features,
            fetched: record.fetched,
        });
    }
    Ok(layers)
}

/// The tree candidates of the stored city tree cadastres; none outside the cities of the table.
fn stored_tree_objects(
    layers: &[StoredTreeLayer],
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let inputs: Vec<TreeInput<'_>> = layers
        .iter()
        .map(|l| TreeInput {
            dataset: l.dataset.id,
            features: &l.features,
            fetched: l.fetched,
        })
        .collect();
    Ok(tree_objects(
        &inputs,
        project.map_kind,
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        georef,
        symbol_set,
    )?)
}

/// One stored extra Berlin layer.
// Read by the Berlin rules and the canopy, stair and tree builders (Berlin plus 1).
#[derive(Debug, Clone)]
pub(crate) struct StoredBerlinLayer {
    pub layer: &'static berlin::ExtraLayer,
    /// The features as served by the WFS (EPSG:25833, `id` like `bauwerkeflaechen.DEBE…`).
    pub features: Vec<serde_json::Value>,
    /// The fetch date, which stands in for the currency (the layers carry no feature dates).
    pub fetched: NaiveDate,
}

/// The stored extra Berlin layers that cover the map area plus buffer, in table order. A layer
/// that was not fetched, is missing or does not cover the area is left out (optional sources:
/// `fetch` reports this, the export goes on without them). Empty outside Berlin.
pub(crate) fn stored_berlin_layers(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Vec<StoredBerlinLayer>, ProjectError> {
    let region_bbox = project.region_bbox()?;
    let mut layers = Vec::new();
    for layer in berlin::extra_layers(project.map_kind) {
        let path = berlin_layer_path(dir, layer);
        let Some(record) = project
            .berlin_layers
            .get(layer.file)
            .filter(|r| path.exists() && r.covers(&region_bbox))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
        let features = berlin::parse_layer(&text).map_err(|e| ProjectError::StoredCandidates {
            path: path.clone(),
            message: e.to_string(),
        })?;
        layers.push(StoredBerlinLayer {
            layer,
            features,
            fetched: record.fetched,
        });
    }
    Ok(layers)
}

/// Candidates from the stored extra Berlin layers and the building parts of the stored buildings
/// (passages), through the Berlin rules. None outside Berlin.
fn stored_berlin_objects(
    project: &ProjectFile,
    layers: &[StoredBerlinLayer],
    buildings: Option<&StoredBuildings>,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let mut inputs: Vec<(&str, &[serde_json::Value], NaiveDate)> = layers
        .iter()
        .map(|l| (l.layer.file, l.features.as_slice(), l.fetched))
        .collect();
    // The building parts are in the buildings file (`bauart` is set by the Berlin source only).
    let parts = match (&project.buildings, buildings) {
        (Some(record), Some(stored)) => {
            Some((building_part_features(&stored.features), record.fetched))
        }
        _ => None,
    };
    if let Some((features, fetched)) = &parts {
        inputs.push((BUILDING_PARTS_LAYER, features.as_slice(), *fetched));
    }
    let inputs: Vec<LayerInput<'_>> = inputs
        .into_iter()
        .map(|(file, features, currency)| LayerInput {
            file,
            features,
            currency,
        })
        .collect();
    Ok(berlin_objects(
        &inputs,
        project.map_kind,
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        georef,
        symbol_set,
    )?)
}

/// The 522 canopy map objects and the 532.x stair candidates of the stored Berlin
/// `bauwerke-flaechen` layer (canopies, stair areas) and `bauwerke-linien` layer (step edges);
/// none outside Berlin.
struct BerlinStructures {
    canopies: Vec<MapObject>,
    stairs: Vec<MapObject>,
}

#[allow(clippy::too_many_arguments)]
fn stored_berlin_structures(
    dir: &Path,
    project: &ProjectFile,
    layers: &[StoredBerlinLayer],
    nas: &[StoredNasLayer],
    buildings: Option<&StoredBuildings>,
    footprints: &[geo_types::Polygon<f64>],
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<BerlinStructures, ProjectError> {
    // The stair areas are in the same layer as the canopies; the step edges are lines.
    let areas = layers.iter().find(|l| l.layer.file == CANOPY_LAYER);
    let edges = layers.iter().find(|l| l.layer.file == STEP_EDGE_LAYER);
    let survey = layers.iter().find(|l| l.layer.file == SURVEY_STAIR_LAYER);
    // The NAS stair areas and bottom edges are in one layer.
    let nas = nas.iter().find(|l| l.layer.file == NAS_STAIR_LAYER);
    let map_area = area.projected_polygon_in(project.crs());
    // Canopies are cut at the 521 footprints; flat entrance steps face them.
    let mut canopies = match areas {
        Some(layer) => canopy_objects(
            &[CanopyInput::Berlin(&layer.features)],
            footprints,
            &map_area,
            project.buffer_m,
            georef,
            symbol_set,
            layer.fetched,
        )?,
        None => Vec::new(),
    };
    canopies.extend(stored_state_canopies(
        project, buildings, footprints, &map_area, georef, symbol_set,
    )?);
    if areas.is_none() && edges.is_none() && survey.is_none() && nas.is_none() {
        return Ok(BerlinStructures {
            canopies,
            stairs: Vec::new(),
        });
    }
    let terrain = stored_stair_terrain(dir, project)?;
    fn features(layer: Option<&StoredBerlinLayer>) -> &[serde_json::Value] {
        layer.map_or(&[], |l| &l.features)
    }
    let nas_fetched = nas.map(|l| l.fetched);
    // Both layers come from one fetch; the later date stands for the currency.
    let fetched = areas
        .iter()
        .chain(edges.iter())
        .chain(survey.iter())
        .map(|l| l.fetched)
        .chain(nas_fetched)
        .max()
        .expect("one of the layers is stored");
    let stairs = stair_objects(
        &StairInput {
            features: features(areas),
            step_edges: features(edges),
            survey: features(survey),
            nas: nas.map_or(&[], |l| &l.features),
            terrain: terrain.as_ref(),
            buildings: footprints,
        },
        project.map_kind,
        &map_area,
        project.buffer_m,
        georef,
        symbol_set,
        fetched,
    )?;
    Ok(BerlinStructures { canopies, stairs })
}

/// The 522 canopy map objects among the stored ALKIS vereinfacht features of any state
/// (`Überdachung`, `Carport`); none without stored buildings.
fn stored_state_canopies(
    project: &ProjectFile,
    stored: Option<&StoredBuildings>,
    footprints: &[geo_types::Polygon<f64>],
    map_area: &geo_types::Polygon<f64>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let (Some(stored), Some(record)) = (stored, &project.buildings) else {
        return Ok(Vec::new());
    };
    Ok(canopy_objects(
        &[CanopyInput::Buildings(&stored.features)],
        footprints,
        map_area,
        project.buffer_m,
        georef,
        symbol_set,
        record.fetched,
    )?)
}

/// 522 candidates for the building passages of the stored ALKIS vereinfacht features of any
/// state; none without stored buildings. Berlin building parts (`bauart`) go through the Berlin
/// rules instead.
fn stored_state_objects(
    project: &ProjectFile,
    stored: Option<&StoredBuildings>,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let (Some(stored), Some(record)) = (stored, &project.buildings) else {
        return Ok(Vec::new());
    };
    Ok(state_objects(
        &stored.features,
        project.map_kind,
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        georef,
        symbol_set,
        record.fetched,
    )?)
}

/// The candidates of the stored full ALKIS (NAS) layers: walls, fences, hedges, monuments,
/// bridges, embankment tops and ditches; none outside the states with a NAS service.
fn stored_nas_objects(
    layers: &[StoredNasLayer],
    project: &ProjectFile,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let inputs: Vec<NasInput<'_>> = layers
        .iter()
        .map(|l| NasInput {
            file: l.layer.file,
            features: &l.features,
            fetched: l.fetched,
        })
        .collect();
    Ok(nas_objects(
        &inputs,
        project.map_kind,
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        georef,
        symbol_set,
    )?)
}

/// The terrain model for the stair direction, in the map's CRS: the stored DGM1 of a per-area
/// provider, else the mosaic `derive` wrote ([`TERRAIN_MOSAIC_RELPATH`]); none before `derive`.
/// Stairs outside it get the long axis (`freemapper:direction=guessed`).
fn stored_stair_terrain(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Option<Raster<f32>>, ProjectError> {
    let bbox = project.bbox()?;
    let per_area = project.terrain.as_ref().is_some_and(|r| r.covers(&bbox))
        && dir.join(TERRAIN_RELPATH).exists();
    let path = if per_area {
        dir.join(TERRAIN_RELPATH)
    } else {
        dir.join(TERRAIN_MOSAIC_RELPATH)
    };
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(read_float_raster(&path)?))
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
        &area.projected_polygon_in(project.crs()),
        &project.bbox()?,
        project.map_kind,
        georef,
        symbol_set,
    )?)
}

/// 501.5 paved areas from the stored land use; sprint maps only (spec §4.5). In Berlin the
/// stored Straßenbefahrung cuts its green strips out and adds its paving as candidates.
fn stored_paved_objects(
    land_use: &[LandUse],
    osm: Option<&(Vec<OsmFeature>, NaiveDate)>,
    berlin_layers: &[StoredBerlinLayer],
    project: &ProjectFile,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<PavedObjects, ProjectError> {
    if project.map_kind != MapKind::Sprint {
        return Ok(PavedObjects::default());
    }
    let streets = osm.map(|(features, _)| unpaved_streets(features, georef.crs()));
    let layer = |file: &str| {
        berlin_layers
            .iter()
            .find(|l| l.layer.file == file)
            .map_or(&[][..], |l| l.features.as_slice())
    };
    let survey =
        StreetSurvey::from_features(layer(GREEN_LAYER), layer(SURFACE_LAYER), georef.crs());
    Ok(paved_area_objects(
        land_use,
        streets.as_deref().unwrap_or_default(),
        &survey,
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
    let project = ProjectFile::load_reporting(dir, progress)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    if project.tiles.is_empty() && project.terrain.is_none() {
        return Err(ProjectError::NotFetched);
    }
    let bbox = project.bbox()?;
    let region_bbox = project.region_bbox()?;
    let provider = raster_region(&region_bbox)
        .ok_or(ProjectError::OutsideCoverage)?
        .provider();
    // The tile products of a tile provider: required, terrain and surface model. The per-area
    // providers (LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V, Geobasis NRW, LGLN, LVermGeo RP)
    // store their height models per area and have none.
    let tile_products: Option<(&[Product], Product, Product)> = match provider {
        Provider::Lgb => Some((&PRODUCTS, Product::Dtm, Product::ImageDsm)),
        Provider::GeoSn => Some((
            &[Product::SaxonyDtm],
            Product::SaxonyDtm,
            Product::SaxonyDsm,
        )),
        Provider::LvermGeo
        | Provider::Lgv
        | Provider::Hvbg
        | Provider::LvermGeoSh
        | Provider::LaivMv
        | Provider::GeobasisNrw
        | Provider::Lgln
        | Provider::LvermGeoRp => None,
    };
    let needed = tile_products.map_or_else(Vec::new, |(_, dtm_product, _)| {
        provider_tiles(&bbox, provider, dtm_product.grid_km())
    });
    let required = tile_products.map_or(&[][..], |(required, _, _)| required);
    let outdated: Vec<TileId> = needed
        .iter()
        .copied()
        .filter(|tile| {
            required.iter().any(|product| {
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
    match provider {
        Provider::Lgb => build_orthophoto(&tiles(Product::Orthophoto)?, &bbox, dir, &orthophoto)?,
        Provider::GeoSn
        | Provider::LvermGeo
        | Provider::Lgv
        | Provider::Hvbg
        | Provider::LvermGeoSh
        | Provider::LaivMv
        | Provider::GeobasisNrw
        | Provider::Lgln
        | Provider::LvermGeoRp => {
            let wms = dir.join(WMS_ORTHOPHOTO_RELPATH);
            if !wms.exists() || !project.orthophoto.as_ref().is_some_and(|r| r.covers(&bbox)) {
                return Err(ProjectError::OrthophotoNotFetched);
            }
            build_orthophoto_from_image(&wms, &bbox, dir, &orthophoto)?;
        }
    }

    let stored_model = |record: &Option<WcsRecord>, relpath: &str| {
        record.as_ref().is_some_and(|r| r.covers(&bbox)) && dir.join(relpath).exists()
    };
    let dtm = if let Some((_, dtm_product, _)) = tile_products {
        mosaic_dtm(&tiles(dtm_product)?, &bbox)?
    } else {
        if !stored_model(&project.terrain, TERRAIN_RELPATH) {
            return Err(ProjectError::TerrainNotFetched);
        }
        // Stored for the area of the last fetch: cut to the current one, as the tile mosaic is.
        let stored = read_float_raster(&dir.join(TERRAIN_RELPATH))?;
        let mut dtm = Raster::filled(GridSpec::covering(&bbox, stored.grid.pixel_size), f32::NAN);
        dtm.paste_with(&stored.grid, |x, y| stored.get(x, y))
            .map_err(DeriveError::from)?;
        dtm
    };
    // The export reads the stair direction from the terrain; a tile provider's mosaic is only in
    // the cache, so it is kept with the project where the Berlin layers have stairs.
    let mosaic = dir.join(TERRAIN_MOSAIC_RELPATH);
    // Only a tile provider's mosaic needs keeping, and the stored layers are read once, only
    // when the Berlin layers have no stairs.
    let stairs = tile_products.is_some() && {
        let berlin_stairs = !project.berlin_layers.is_empty()
            && stored_berlin_layers(dir, &project)?
                .iter()
                .any(|l| has_stairs(&l.features));
        // A NAS stair without a bottom edge goes by the terrain too.
        berlin_stairs
            || stored_nas_layers(dir, &project)?.iter().any(|l| {
                l.layer.file == NAS_STAIR_LAYER && has_nas_stairs_without_edge(&l.features)
            })
    };
    if stairs {
        progress(&format!("writing {TERRAIN_MOSAIC_RELPATH}"));
        crate::sources::float_tiff::write_float_raster(&dtm, dir, &mosaic, TERRAIN_MOSAIC_RELPATH)?;
    } else {
        for file in [mosaic.clone(), mosaic.with_extension("tfw")] {
            crate::fsutil::remove_inside(dir, &file);
        }
    }
    let hillshade_path = dir.join(HILLSHADE_RELPATH);
    progress(&format!("writing {HILLSHADE_RELPATH}"));
    write_gray_png(&hillshade(&dtm), dir, &hillshade_path)?;
    let slope_path = dir.join(SLOPE_RELPATH);
    progress(&format!("writing {SLOPE_RELPATH}"));
    let black_deg = project.map_kind.slope_black_deg();
    write_gray_png(&slope_shade(&slope_deg(&dtm), black_deg), dir, &slope_path)?;

    progress(&format!("writing {CONTOURS_RELPATH}"));
    let smoothed = gaussian_smooth(&dtm, project.map_kind.contour_sigma_m());
    let interval = f64::from(project.map_kind.contour_interval_m());
    let lines = generalize(
        contour_lines(&smoothed, interval),
        project.scale_denominator,
    );
    let contours_path = dir.join(CONTOURS_RELPATH);
    write_inside(dir, &contours_path, contours_to_geojson(&lines).as_bytes())?;

    let mut written = vec![orthophoto, hillshade_path, slope_path];
    if project.buildings.as_ref().is_some_and(|b| b.count == 0) && touches_berlin(&region_bbox) {
        progress(
            "warning: no ALKIS buildings stored for this area in Berlin; run `freemapper fetch` to get them from the Geoportal Berlin",
        );
    }
    let dsm_recorded = match tile_products {
        Some((_, _, dsm_product)) => project.tiles.iter().any(|r| r.product == dsm_product),
        None => stored_model(&project.surface, SURFACE_RELPATH),
    };
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
        let model = match tile_products {
            Some((_, _, dsm_product)) => {
                tiles(dsm_product).and_then(|t| Ok(mosaic_dtm(&t, &bbox)?))
            }
            None => read_float_raster(&dir.join(SURFACE_RELPATH)).map_err(ProjectError::from),
        };
        match model {
            Ok(dsm) => Some(dsm),
            Err(e) => {
                progress(&format!(
                    "warning: surface model unusable ({e}); run `freemapper fetch`"
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
            crate::fsutil::remove_inside(dir, &stale);
        }
    } else {
        progress(&format!("writing {NDSM_RELPATH}"));
        let mut ndsm = rasterize(&ndsm_files, &bbox)?;
        // Consumed here: the 0.2 m surface model is as large as the nDSM itself.
        if let Some(dsm) = dsm {
            fill_from_dsm(&mut ndsm, &dsm, &dtm);
        }
        write_ndsm_png(&ndsm, dir, &ndsm_path)?;
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
        /// ALKIS canopies (522 map objects) are stored for the project, but the kept map has
        /// none (it was written before them).
        missing_canopies: bool,
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
            missing_canopies: map_lacks_canopies(dir, &text)?,
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

/// The stored ALKIS buildings, read once per export: parsed, and the raw features (the Berlin
/// building parts are picked from these).
struct StoredBuildings {
    buildings: Vec<crate::sources::alkis::Building>,
    features: Vec<serde_json::Value>,
}

impl StoredBuildings {
    /// The 521 footprints (buildings that become map objects), in the map's CRS.
    fn footprints(&self) -> Vec<geo_types::Polygon<f64>> {
        self.buildings
            .iter()
            .filter(|b| classify(b) == Rule::Object)
            .flat_map(|b| b.footprint.0.iter().cloned())
            .collect()
    }
}

/// The stored ALKIS buildings; none when no buildings were fetched.
fn read_stored_buildings(dir: &Path) -> Result<Option<StoredBuildings>, ProjectError> {
    let path = dir.join(BUILDINGS_RELPATH);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let mut features = berlin::parse_layer(&text).map_err(|e| ProjectError::StoredBuildings {
        path: path.clone(),
        message: e.to_string(),
    })?;
    // Berlin buildings carry no date; the 2022 height layer, if stored, gives their footprint
    // date. It is an optional source: an unreadable file leaves the buildings undated.
    if let Some(layer) = known_berlin_layer("gebaeudehoehen")
        && let Ok(text) = std::fs::read_to_string(berlin_layer_path(dir, layer))
        && let Ok(heights) = berlin::parse_layer(&text)
    {
        berlin::join_footprint_dates(&mut features, &heights);
    }
    let buildings = parse_building_features(&features).buildings;
    Ok(Some(StoredBuildings {
        buildings,
        features,
    }))
}

/// 521 objects for the stored ALKIS buildings; none when no buildings were fetched. Fails when the
/// recorded fetch does not cover the current map area plus buffer.
fn stored_building_objects(
    project: &ProjectFile,
    stored: Option<&StoredBuildings>,
    area: &MapArea,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, ProjectError> {
    let Some(stored) = stored else {
        return Ok(Vec::new());
    };
    if let Some(record) = &project.buildings
        && !record.covers(&project.bbox()?)
    {
        return Err(ProjectError::BuildingsNotFetched);
    }
    Ok(building_objects(
        &stored.buildings,
        &area.projected_polygon_in(project.crs()),
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
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        georef,
        symbol_set,
        "osm",
        *extract,
    )?)
}

/// 101/102 candidate objects for the contour lines from `derive`; none when there are none.
/// They stop at the drawn ALKIS `water`: ISSprOM shows no contours in uncrossable water.
fn stored_contour_objects(
    dir: &Path,
    project: &ProjectFile,
    footprints: &[geo_types::Polygon<f64>],
    area: &MapArea,
    water: &geo_types::MultiPolygon<f64>,
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
    // The contours come from the terrain model of the area's raster provider.
    let provider = match project.provider {
        Some(provider) => provider,
        None => raster_region(&project.region_bbox()?).map_or(Provider::Lgb, Region::provider),
    };
    let source = provider.terrain_source();
    let map_area = area.projected_polygon_in(project.crs());
    // Stubs of a cut contour along the shore shorter than 1 mm on paper are noise.
    let min_piece_m = f64::from(project.scale_denominator) / 1000.0;
    // DGM1 is interpolated under buildings: closed rings there are artefacts.
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
        .flat_map(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            outside_water(&ls, water, min_piece_m)
                .into_iter()
                .map(move |piece| (line, piece))
        })
        .map(|(line, piece)| {
            let coords: Vec<(i32, i32)> = piece
                .coords()
                .map(|c| {
                    georef
                        .projected_to_map(ProjectedPoint {
                            easting: c.x,
                            northing: c.y,
                        })
                        .map(|c| (c.x, c.y))
                })
                .collect::<Result<_, _>>()?;
            let closed = coords.len() > 3 && coords.first() == coords.last();
            let mut tags = vec![
                ("freemapper:source".to_string(), source.to_string()),
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
#[cfg(test)]
fn stored_buildings(dir: &Path) -> Result<Vec<crate::sources::alkis::Building>, ProjectError> {
    Ok(read_stored_buildings(dir)?.map_or_else(Vec::new, |s| s.buildings))
}

/// Whether the stored buildings give map objects that the map at `map_text` lacks.
fn map_lacks_buildings(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if map_text.contains(ALKIS_SOURCE_TAG) {
        return Ok(false);
    }
    let project = ProjectFile::load(dir)?;
    let area = MapArea::from_geojson(&project.map_area.to_string())?;
    let georef = Georeferencing::new_in(
        project.crs(),
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set();
    let stored = read_stored_buildings(dir)?;
    Ok(
        !stored_building_objects(&project, stored.as_ref(), &area, &georef, &symbol_set)?
            .is_empty(),
    )
}

/// How a building object's provenance appears in the map file.
const ALKIS_SOURCE_TAG: &str = r#"<t k="freemapper:source">alkis</t>"#;

/// Whether the stored layers and buildings give 522 canopy map objects that the map at
/// `map_text` lacks (it was written before them, or without the layers).
fn map_lacks_canopies(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if CANOPY_SOURCE_TAGS.iter().any(|tag| map_text.contains(tag)) {
        return Ok(false);
    }
    let project = ProjectFile::load(dir)?;
    let layers = stored_berlin_layers(dir, &project)?;
    let area = MapArea::from_geojson(&project.map_area.to_string())?;
    let georef = Georeferencing::new_in(
        project.crs(),
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set();
    let stored = read_stored_buildings(dir)?;
    let footprints = stored.as_ref().map_or_else(Vec::new, |b| b.footprints());
    let map_area = area.projected_polygon_in(project.crs());
    if let Some(layer) = layers.iter().find(|l| l.layer.file == CANOPY_LAYER)
        && !canopy_objects(
            &[CanopyInput::Berlin(&layer.features)],
            &footprints,
            &map_area,
            project.buffer_m,
            &georef,
            &symbol_set,
            layer.fetched,
        )?
        .is_empty()
    {
        return Ok(true);
    }
    Ok(!stored_state_canopies(
        &project,
        stored.as_ref(),
        &footprints,
        &map_area,
        &georef,
        &symbol_set,
    )?
    .is_empty())
}

/// How a canopy object's provenance appears in the map file: a Berlin canopy, or an ALKIS
/// vereinfacht one (the buildings share `freemapper:source` = `alkis`, so the kind tells them
/// apart).
const CANOPY_SOURCE_TAGS: [&str; 3] = [
    r#"<t k="freemapper:source">geoportal-berlin/bauwerke-flaechen</t>"#,
    r#"<t k="freemapper:alkis">Überdachung</t>"#,
    r#"<t k="freemapper:alkis">Carport</t>"#,
];

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
        back_up(dir, &map_path, &map_path.with_extension("omap.bak"))?;
    }
    write_document(dir, &candidates, &dir.join(CANDIDATES_RELPATH))?;
    if map_force.is_some() {
        write_document(dir, &map, &map_path)?;
    }
    let attribution = format!("{attribution_text}\n\n{FREEMAPPER_CREDIT}\n");
    write_inside(dir, &dir.join("ATTRIBUTION.txt"), attribution.as_bytes())?;
    Ok(map_path)
}

/// What to do when the map to back up is a link; ends the error of [`back_up`].
const LINKED_MAP_HINT: &str = "replace the link with the real file and run the command again";

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
    /// Templates left out of the OCAD map, as `warning: …` lines.
    pub warnings: Vec<String>,
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
    check_folders(dir, &[OCAD_DIR])?;
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
    let (mut backgrounds, warnings) = copy_backgrounds(dir, &ocad_dir, &documents.map.templates)?;
    let candidates_name = format!("{name}-candidates.ocd");
    backgrounds.push(Background::Map {
        file_name: candidates_name.clone(),
        visible: true,
    });
    let candidates = ocad_dir.join(&candidates_name);
    let (bytes, _) = map_to_ocd(&documents.candidates, project.map_kind, &[])?;
    write_inside(dir, &candidates, &bytes)?;
    let map = ocad_dir.join(format!("{name}.ocd"));
    let kept = map.exists() && !force;
    let (bytes, report) = map_to_ocd(&documents.map, project.map_kind, &backgrounds)?;
    if !kept {
        if map.exists() {
            back_up(dir, &map, &map.with_extension("ocd.bak"))?;
        }
        write_inside(dir, &map, &bytes)?;
    }
    Ok(OcadOutcome {
        map,
        kept,
        candidates,
        objects: report.objects,
        skipped: report.skipped,
        dropped_parts: report.dropped_parts,
        warnings,
    })
}

/// Copies the image `templates` with their world files into `ocad_dir` and returns them as
/// OCAD background maps. A template that is a link or lies outside the project folder is left
/// out with a warning, so no file from elsewhere is copied into the project.
fn copy_backgrounds(
    dir: &Path,
    ocad_dir: &Path,
    templates: &[TemplateRef],
) -> Result<(Vec<Background>, Vec<String>), ProjectError> {
    let (mut backgrounds, mut warnings) = (Vec::new(), Vec::new());
    for template in templates {
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
        let checked = crate::fsutil::check_source(dir, &image)
            .and_then(|()| crate::fsutil::check_source(dir, &world));
        if let Err((path, e)) = checked {
            warnings.push(format!(
                "warning: {}; the OCAD map gets no background from it",
                crate::space::io_error_text(&path, &e)
            ));
            continue;
        }
        copy_inside(dir, &image, &ocad_dir.join(image_name))?;
        copy_inside(dir, &world, &ocad_dir.join(world_name))?;
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
    Ok((backgrounds, warnings))
}

/// The map part with the peripheral information (title, scale bar, credits); the north lines
/// are map content in the default part.
const SHEET_PART: &str = "peripheral information";

/// The objects of the training map: everything of the map and every candidate except the map
/// area outline (the first candidate) and the points hidden under a 520 candidate
/// (`freemapper:hidden`), and separately the sheet objects, which are not clipped.
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
                .flat_map(|p| p.objects.iter().skip(1))
                .filter(|o| !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)),
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
    write_inside(dir, &path, &bytes)?;
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
    let georef = Georeferencing::new_in(
        project.crs(),
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set();

    // Buildings first: they are the map's own objects, candidates only a template.
    let stored_buildings = read_stored_buildings(dir)?;
    let footprints = stored_buildings
        .as_ref()
        .map_or_else(Vec::new, StoredBuildings::footprints);
    let building_objects = stored_building_objects(
        &project,
        stored_buildings.as_ref(),
        &area,
        &georef,
        &symbol_set,
    )?;
    let berlin_layers = stored_berlin_layers(dir, &project)?;
    let nas_layers = stored_nas_layers(dir, &project)?;
    let mut structures = stored_berlin_structures(
        dir,
        &project,
        &berlin_layers,
        &nas_layers,
        stored_buildings.as_ref(),
        &footprints,
        &area,
        &georef,
        &symbol_set,
    )?;
    let land_use = stored_land_use(dir, &project)?;
    let osm = stored_osm_features(dir, &project, &area)?;
    let paved = stored_paved_objects(
        &land_use,
        osm.as_ref(),
        &berlin_layers,
        &project,
        &georef,
        &symbol_set,
    )?;
    let clip = project.bbox()?;
    let water = water_objects(&land_use, &clip, &georef, &symbol_set)?;
    // OSM ponds inside the drawn ALKIS water would only duplicate it, and fountain points in
    // water are mostly the jets of a basin.
    let osm_objects = |target| -> Result<Vec<MapObject>, ProjectError> {
        let kept = drop_covered_water(
            stored_osm_objects(target, osm.as_ref(), &project, &area, &georef, &symbol_set)?,
            &water.union,
            &clip,
            &georef,
            &symbol_set,
        );
        Ok(drop_points_in_water(
            kept,
            &water.union,
            &georef,
            &symbol_set,
        ))
    };
    // The candidates of the cadastres, registers and the survey: Berlin first, then the state
    // passages and the full ALKIS (NAS). Duplicates go by rank (`build/dedup.rs`): register >
    // ALKIS > survey > OSM. A register knows the crown: an OSM tree at a register tree is the
    // same tree. ALKIS surveyed the fences, walls and hedges: an OSM line along one with its
    // symbol is the same.
    let mut cadastre_candidates = stored_berlin_objects(
        &project,
        &berlin_layers,
        stored_buildings.as_ref(),
        &area,
        &georef,
        &symbol_set,
    )?;
    cadastre_candidates.extend(stored_state_objects(
        &project,
        stored_buildings.as_ref(),
        &area,
        &georef,
        &symbol_set,
    )?);
    cadastre_candidates.extend(stored_nas_objects(
        &nas_layers,
        &project,
        &area,
        &georef,
        &symbol_set,
    )?);
    cadastre_candidates.extend(stored_tree_objects(
        &stored_tree_layers(dir, &project)?,
        &project,
        &area,
        &georef,
        &symbol_set,
    )?);
    // A survey line along an ALKIS line goes first, then an OSM line along either.
    let mut cadastre_candidates =
        drop_cadastre_lines_by_rank(cadastre_candidates, &symbol_set, georef.scale_denominator);
    let (osm_candidates, cadastre_candidates_kept) = drop_trees_by_rank(
        osm_objects(Target::Candidates)?,
        cadastre_candidates,
        &symbol_set,
        georef.scale_denominator,
    );
    cadastre_candidates = cadastre_candidates_kept;
    let osm_candidates = drop_osm_lines_near_cadastre(
        osm_candidates,
        &mut cadastre_candidates,
        &symbol_set,
        georef.scale_denominator,
    );
    // The lighting cadastre and the fountain registers beat the survey and OSM points at the same
    // place, and the survey furniture beats OSM; a cadastre, register or survey fountain in drawn
    // water is a jet like an OSM one.
    let (osm_candidates, cadastre_candidates) = drop_points_near_registers(
        osm_candidates,
        cadastre_candidates,
        &symbol_set,
        georef.scale_denominator,
    );
    let cadastre_candidates =
        drop_points_in_water(cadastre_candidates, &water.union, &georef, &symbol_set);
    // A surveyed bus shelter under an ALKIS canopy is that canopy.
    let cadastre_candidates =
        drop_shelters_under_canopies(cadastre_candidates, &mut structures.canopies);
    // The point symbols keep the norm's gap, dense tree groups become 402, 405 or 501.3, and no
    // point stands in a 520 area (`build/points.rs`). The OSM candidates stay before the others.
    let out_of_bounds =
        stored_out_of_bounds_objects(&land_use, &project, &area, &georef, &symbol_set)?;
    let mut points = osm_candidates;
    points.extend(cadastre_candidates);
    let generalized = generalize_points(
        points,
        osm_objects(Target::Map)?,
        &out_of_bounds,
        &paved.map,
        &footprints,
        &water.union,
        &clip,
        project.map_kind,
        &georef,
        &symbol_set,
    )?;
    let mut point_candidates = generalized.candidates;
    point_candidates.extend(generalized.areas);
    // A register is named once its points (or trees merged into a group area) are among the
    // candidates; OSM is named anyway.
    let registers = drawn_registers(&point_candidates);
    let attribution_text = attribution_lines(&project, &registers).join("\n");
    let notes = format!("{attribution_text}\n{FREEMAPPER_CREDIT}");
    let candidates = MapDocument {
        notes: notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts: vec![MapPart {
            name: "candidates".to_string(),
            objects: std::iter::once(map_area_outline(&area, &georef, &symbol_set, &project)?)
                .chain(point_candidates)
                .chain(structures.stairs)
                .chain(stored_contour_objects(
                    dir,
                    &project,
                    &footprints,
                    &area,
                    &water.union,
                    &georef,
                    &symbol_set,
                )?)
                .chain(paved.candidates)
                .chain(water.candidates)
                .chain(out_of_bounds)
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
        (K5_RELPATH, false, 0.7),
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
    let content = PaperRect::of_bbox(&project.bbox()?, &georef)?;
    // North lines first: within their black they lie under walls and fences.
    let spacing_mm = NORTH_LINE_SPACING_MM * project.map_kind.symbol_enlargement();
    let mut map_objects = north_lines(&content, spacing_mm, &symbol_set);
    map_objects.extend(building_objects);
    // Canopies after the buildings; their Black 20 % lies under the building fill anyway.
    map_objects.extend(structures.canopies);
    map_objects.extend(paved.map);
    map_objects.extend(water.map);
    map_objects.extend(generalized.map);
    let buildings_part = MapPart {
        name: "default part".to_string(),
        objects: map_objects,
    };
    let mut attribution = vec![OSM_ATTRIBUTION_DE.to_string()];
    attribution.extend(lgb_attribution(&project));
    attribution.extend(berlin_attribution(&project, &registers));
    attribution.extend(tree_attribution(&registers));
    attribution.extend(geosn_attribution(&project));
    attribution.extend(lvermgeo_attribution(&project));
    attribution.extend(lgv_attribution(&project));
    attribution.extend(hvbg_attribution(&project));
    attribution.extend(sh_attribution(&project));
    attribution.extend(mv_attribution(&project));
    attribution.extend(nrw_attribution(&project));
    attribution.extend(lgln_attribution(&project));
    attribution.extend(rp_attribution(&project));
    attribution.push(FREEMAPPER_CREDIT_DE.to_string());
    let info = PeripheralInfo {
        title: project.display_title(dir),
        scale_denominator: project.scale_denominator,
        contour_interval_m: project.map_kind.contour_interval_m(),
        attribution,
    };
    let parts = vec![
        buildings_part,
        MapPart {
            name: SHEET_PART.to_string(),
            objects: info_block(&content, &info, &symbol_set),
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
/// GeoSN's source note for dl-de/by-2-0 data, which asks for changes to be marked.
const GEOSN_ATTRIBUTION: &str = "Quelle: GeoSN, dl-de/by-2-0, Daten geändert";
/// Geoportal Berlin data needs no attribution (dl-de/zero-2-0); it is named anyway.
const BERLIN_ATTRIBUTION: &str =
    "© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert";
/// The Berlin K5 map, once the template was written.
const BERLIN_K5_ATTRIBUTION: &str =
    "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0";
/// The Berlin tree cadastre, once tree candidates were written.
const BERLIN_TREES_ATTRIBUTION: &str = "Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0";
/// The Straßenbefahrung layers, once one of them has features.
const BERLIN_SURVEY_ATTRIBUTION: &str =
    "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert";
/// The lighting cadastre and the fountain registers: the stored file and the name in the
/// attribution, in the order of the line.
const BERLIN_REGISTERS: [(&str, &str); 3] = [
    ("beleuchtung", "Öffentliche Beleuchtung"),
    ("zierbrunnen", "Zierbrunnen"),
    ("trinkwasserbrunnen", "Trinkwasserbrunnen"),
];
/// The building heights (footprint dates), once their layer has features.
const BERLIN_HEIGHTS_ATTRIBUTION: &str =
    "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0";
/// [`OSM_ATTRIBUTION`] for the German peripheral information.
const OSM_ATTRIBUTION_DE: &str = "© OpenStreetMap-Mitwirkende, ODbL";
/// [`FREEMAPPER_CREDIT`] for the German peripheral information.
const FREEMAPPER_CREDIT_DE: &str =
    concat!("Erstellt mit freemapper – ", env!("CARGO_PKG_REPOSITORY"));
/// Names the tool and where to get it, in every generated project.
const FREEMAPPER_CREDIT: &str = concat!("Created with freemapper – ", env!("CARGO_PKG_REPOSITORY"));

/// The registers whose points are among `objects`: the files of the Berlin registers
/// ([`BERLIN_REGISTERS`]) and the ids of the city tree cadastres (`trees::TREE_DATASETS`). A
/// group area of the point generalization draws the sources of the trees it merged
/// (`freemapper:source_id` = `<source>/<source_id>; …`).
fn drawn_registers(objects: &[MapObject]) -> Vec<&'static str> {
    let tag = |o: &'_ MapObject, key: &str| {
        o.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    let drawn = |source: String| {
        let prefix = format!("{source}/");
        objects
            .iter()
            .any(|o| match tag(o, "freemapper:source").as_deref() {
                Some(GENERALIZATION_SOURCE) => tag(o, "freemapper:source_id")
                    .is_some_and(|ids| ids.split("; ").any(|id| id.starts_with(&prefix))),
                Some(own) => own == source,
                None => false,
            })
    };
    let berlin = BERLIN_REGISTERS
        .iter()
        .map(|&(file, _)| file)
        .filter(|file| drawn(format!("geoportal-berlin/{file}")));
    let cities = trees::TREE_DATASETS
        .iter()
        .map(|d| d.id)
        .filter(|id| drawn(format!("trees/{id}")));
    berlin.chain(cities).collect()
}

/// The attribution lines of the city tree cadastres in `registers` ([`drawn_registers`]): one
/// for each dataset that drew trees.
fn tree_attribution(registers: &[&str]) -> Vec<String> {
    trees::TREE_DATASETS
        .iter()
        .filter(|d| registers.contains(&d.id))
        .map(|d| d.attribution.to_string())
        .collect()
}

/// Attribution of every source the project uses: OSM, LGB once tiles were fetched, and the
/// Geoportal Berlin. `registers` are the Berlin registers and city tree cadastres that drew
/// objects ([`drawn_registers`]).
fn attribution_lines(project: &ProjectFile, registers: &[&str]) -> Vec<String> {
    let mut lines = vec![OSM_ATTRIBUTION.to_string()];
    lines.extend(lgb_attribution(project));
    lines.extend(berlin_attribution(project, registers));
    lines.extend(tree_attribution(registers));
    lines.extend(geosn_attribution(project));
    lines.extend(lvermgeo_attribution(project));
    lines.extend(lgv_attribution(project));
    lines.extend(hvbg_attribution(project));
    lines.extend(nrw_attribution(project));
    if let Some(line) = rp_attribution(project) {
        lines.push(line);
        lines.push(RP_LICENCE_NOTE.to_string());
    }
    // The CC BY credits refer to the same licence text, which is given once.
    let cc_by = [
        sh_attribution(project),
        mv_attribution(project),
        lgln_attribution(project),
    ];
    let any_cc_by = cc_by.iter().any(Option::is_some);
    lines.extend(cc_by.into_iter().flatten());
    if any_cc_by {
        lines.push(CC_BY_LICENCE_NOTE.to_string());
    }
    lines
}

/// The Geoportal Berlin lines: ALKIS once Berlin ALKIS objects were fetched, the K5 map and the
/// tree cadastre once their data was written, the street survey and the building heights once
/// their layers have features, and one line for the `registers` that drew objects, each named.
fn berlin_attribution(project: &ProjectFile, registers: &[&str]) -> Vec<String> {
    let mut lines = Vec::new();
    if alkis_records(project).any(|record| record.regions.contains(&Region::Berlin)) {
        lines.push(BERLIN_ATTRIBUTION.to_string());
    }
    if project.k5.is_some() {
        lines.push(BERLIN_K5_ATTRIBUTION.to_string());
    }
    // A stored tree layer without trees draws no candidate.
    let trees = ["strassenbaeume", "anlagenbaeume"].iter().any(|file| {
        project
            .berlin_layers
            .get(*file)
            .is_some_and(|r| r.count > 0)
    });
    if trees {
        lines.push(BERLIN_TREES_ATTRIBUTION.to_string());
    }
    // A stored layer without features contributed nothing.
    let has = |keep: &dyn Fn(&str) -> bool| {
        project
            .berlin_layers
            .iter()
            .any(|(file, record)| record.count > 0 && keep(file))
    };
    if has(&|file| file.starts_with("sb-")) {
        lines.push(BERLIN_SURVEY_ATTRIBUTION.to_string());
    }
    // A register that is fetched but drew nothing (another map kind, outside the area) is not
    // named.
    let names: Vec<&str> = BERLIN_REGISTERS
        .iter()
        .filter(|(file, _)| registers.contains(file))
        .map(|&(_, name)| name)
        .collect();
    if !names.is_empty() {
        lines.push(format!(
            "Geoportal Berlin / {}, dl-de/zero-2-0",
            names.join(", ")
        ));
    }
    if has(&|file| file == "gebaeudehoehen") {
        lines.push(BERLIN_HEIGHTS_ATTRIBUTION.to_string());
    }
    lines
}

/// The GeoSN line, once GeoSN tiles, its orthophoto or Saxon ALKIS objects were fetched.
fn geosn_attribution(project: &ProjectFile) -> Option<String> {
    let used = project
        .tiles
        .iter()
        .any(|t| t.product.provider() == Provider::GeoSn)
        || (project.orthophoto.is_some() && raster_provider(project) == Some(Provider::GeoSn))
        || alkis_records(project).any(|record| record.regions.contains(&Region::Saxony));
    used.then(|| GEOSN_ATTRIBUTION.to_string())
}

/// The LVermGeo line, once its rasters or Saxony-Anhalt ALKIS objects were fetched; the year is
/// that of the newest fetch.
fn lvermgeo_attribution(project: &ProjectFile) -> Option<String> {
    let year = per_area_attribution_year(project, Provider::LvermGeo, Region::SaxonyAnhalt)?;
    Some(format!(
        "© GeoBasis-DE / LVermGeo ST {year}, dl-de/by-2-0, Daten geändert"
    ))
}

/// The LGV line, once its rasters or Hamburg ALKIS objects were fetched; the year is that of the
/// newest fetch.
fn lgv_attribution(project: &ProjectFile) -> Option<String> {
    let year = per_area_attribution_year(project, Provider::Lgv, Region::Hamburg)?;
    Some(format!(
        "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) {year}, dl-de/by-2-0, Daten geändert"
    ))
}

/// The HVBG line, once its rasters or Hesse ALKIS objects were fetched; the year is that of the
/// newest fetch.
fn hvbg_attribution(project: &ProjectFile) -> Option<String> {
    let year = per_area_attribution_year(project, Provider::Hvbg, Region::Hessen)?;
    Some(format!(
        "Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) {year}, dl-de/zero-2-0, verändert"
    ))
}

/// The Geobasis NRW line, once its rasters or North Rhine-Westphalia ALKIS objects were fetched;
/// the year is that of the newest fetch. The licence (dl-de/zero-2-0) asks for no note of changes.
fn nrw_attribution(project: &ProjectFile) -> Option<String> {
    let year =
        per_area_attribution_year(project, Provider::GeobasisNrw, Region::NordrheinWestfalen)?;
    Some(format!(
        "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) {year}, dl-de/zero-2-0"
    ))
}

/// The LVermGeo RP line, once its rasters or Rhineland-Palatinate ALKIS objects were fetched: the
/// form the open-data page prescribes (no blank before the year of the newest fetch), with the
/// note of changes, since freemapper derives map objects.
fn rp_attribution(project: &ProjectFile) -> Option<String> {
    let year = per_area_attribution_year(project, Provider::LvermGeoRp, Region::RheinlandPfalz)?;
    Some(format!(
        "©GeoBasis-DE / LVermGeoRP{year}, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"
    ))
}

/// The licence the LVermGeo RP line refers to (added to the project notes only).
const RP_LICENCE_NOTE: &str = "Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0";

/// The licence text the CC BY credits (Schleswig-Holstein, Mecklenburg-Vorpommern,
/// Niedersachsen) refer to (added to the project notes only).
const CC_BY_LICENCE_NOTE: &str = "Lizenz CC BY 4.0 (Namensnennung 4.0 International): https://creativecommons.org/licenses/by/4.0/";
/// The LVermGeo SH credit the licence prescribes for changed data.
const SH_ATTRIBUTION: &str = "© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)";

/// The LVermGeo SH line, once its rasters or Schleswig-Holstein ALKIS objects were fetched.
fn sh_attribution(project: &ProjectFile) -> Option<String> {
    per_area_attribution_year(project, Provider::LvermGeoSh, Region::SchleswigHolstein)?;
    Some(SH_ATTRIBUTION.to_string())
}

/// The LAiV M-V credit the licence prescribes for changed data.
const MV_ATTRIBUTION: &str = "©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)";

/// The LAiV M-V line, once its rasters or Mecklenburg-Vorpommern ALKIS objects were fetched.
fn mv_attribution(project: &ProjectFile) -> Option<String> {
    per_area_attribution_year(project, Provider::LaivMv, Region::MecklenburgVorpommern)?;
    Some(MV_ATTRIBUTION.to_string())
}

/// The LGLN line, once its rasters or Niedersachsen ALKIS objects were fetched: the credit the
/// AGNB prescribes (with the year of the newest fetch and the note of changes) and the licence.
fn lgln_attribution(project: &ProjectFile) -> Option<String> {
    let year = per_area_attribution_year(project, Provider::Lgln, Region::Niedersachsen)?;
    Some(format!(
        "© GeoBasis-DE/LGLN {year}, Daten geändert, CC BY 4.0"
    ))
}

/// The year of the newest fetch of `provider`'s data in the project: its terrain model and
/// orthophoto (when the project's rasters come from it) and its ALKIS objects of `region`.
fn per_area_attribution_year(
    project: &ProjectFile,
    provider: Provider,
    region: Region,
) -> Option<i32> {
    let raster = raster_provider(project) == Some(provider);
    [
        project
            .terrain
            .as_ref()
            .filter(|_| raster)
            .map(|r| r.fetched),
        project
            .orthophoto
            .as_ref()
            .filter(|_| raster)
            .map(|r| r.fetched),
    ]
    .into_iter()
    .flatten()
    .chain(
        alkis_records(project)
            .filter(|r| r.regions.contains(&region))
            .map(|r| r.fetched),
    )
    .map(|d| d.year())
    .max()
}

/// The provider of the project's rasters.
fn raster_provider(project: &ProjectFile) -> Option<Provider> {
    raster_region(&project.region_bbox().ok()?).map(Region::provider)
}

/// The stored ALKIS records (buildings, land use).
fn alkis_records(project: &ProjectFile) -> impl Iterator<Item = &AlkisRecord> {
    [&project.buildings, &project.land_use]
        .into_iter()
        .flatten()
}

/// The states of the given ALKIS object ids, in [`Region::ALL`] order.
fn regions_of<'a>(oids: impl Iterator<Item = &'a str>) -> Vec<Region> {
    let found: Vec<Region> = oids.filter_map(Region::of_object_id).collect();
    Region::ALL
        .into_iter()
        .filter(|r| found.contains(r))
        .collect()
}

/// The LGB attribution line, once tiles were fetched. LGB tiles in Berlin hold Berlin data, and
/// LGB's licence note then asks for the Geoportal Berlin as well.
fn lgb_attribution(project: &ProjectFile) -> Option<String> {
    let tile_year = project
        .tiles
        .iter()
        .filter(|t| t.product.provider() == Provider::Lgb)
        .map(|t| t.published.year())
        .max();
    let alkis_year = alkis_records(project)
        .filter(|r| r.regions.contains(&Region::Brandenburg))
        .map(|r| r.fetched.year())
        .max();
    let year = tile_year.or(alkis_year)?;
    let in_berlin = project
        .region_bbox()
        .is_ok_and(|bbox| touches_berlin(&bbox));
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
            let c = georef.geographic_to_map(LatLon {
                lat: p.y(),
                lon: p.x(),
            })?;
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

fn write_document(dir: &Path, doc: &MapDocument, path: &Path) -> Result<(), ProjectError> {
    // Written whole and then renamed: a full disk leaves the previous map as it was.
    let mut bytes = Vec::new();
    write_map(doc, &mut bytes, XmlFormat::Compact)?;
    Ok(write_inside(dir, path, &bytes)?)
}

/// [`crate::fsutil::copy_inside`] with the error as [`ProjectError::Io`].
fn copy_inside(dir: &Path, from: &Path, to: &Path) -> Result<(), ProjectError> {
    crate::fsutil::copy_inside(dir, from, to).map_err(|(path, e)| ProjectError::Io(path, e))
}

/// Copies the map `map` to its backup `backup` before it is replaced. A map that is a link is
/// not copied; the error then says what to do.
fn back_up(dir: &Path, map: &Path, backup: &Path) -> Result<(), ProjectError> {
    crate::fsutil::copy_inside(dir, map, backup).map_err(|(path, e)| {
        if e.to_string() == crate::fsutil::NOT_COPIED {
            let message = format!("{}; {LINKED_MAP_HINT}", crate::fsutil::NOT_COPIED);
            ProjectError::Io(path, std::io::Error::other(message))
        } else {
            ProjectError::Io(path, e)
        }
    })
}

/// Folders that receive files under fixed names from code that is not given the project
/// folder (the downloads and the derived templates). `data/berlin` is checked per layer.
const WRITTEN_FOLDERS: [&str; 2] = ["data", "templates"];

/// Refuses a project whose `folders` lead out of the project folder `dir` through a link: the
/// files written there would land elsewhere.
fn check_folders(dir: &Path, folders: &[&str]) -> Result<(), ProjectError> {
    for folder in folders {
        crate::fsutil::check_folder_inside(dir, &dir.join(folder))
            .map_err(|(path, e)| ProjectError::Io(path, e))?;
    }
    Ok(())
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
        assert!(err.to_string().contains("Saxony-Anhalt"), "{err}");
        assert!(!project.exists());
    }

    /// The Latina August Hermann Francke in Halle as a map area (a 200 m square).
    fn latina_area() -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(289_695.0, 5_706_967.0),
            corner(289_895.0, 5_706_967.0),
            corner(289_895.0, 5_707_167.0),
            corner(289_695.0, 5_707_167.0),
            corner(289_695.0, 5_706_967.0),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    #[test]
    fn halle_project_is_in_zone_32() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(
            dir.path(),
            &latina_area(),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        let bbox = project.bbox().unwrap();
        assert!(
            (bbox.min_e + bbox.max_e) / 2.0 - 706_451.0 < 50.0,
            "{bbox:?}"
        );
        assert!(
            ((bbox.min_n + bbox.max_n) / 2.0 - 5_706_913.0).abs() < 50.0,
            "{bbox:?}"
        );
        let region = project.region_bbox().unwrap();
        assert!(
            ((region.min_e + region.max_e) / 2.0 - 289_795.0).abs() < 1.0,
            "{region:?}"
        );
        let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert!(text.contains("crs = 25832"), "{text}");
    }

    /// A schoolyard project at the Latina in Halle, served with synthetic LVermGeo data.
    fn halle_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(
            dir.path(),
            &latina_area(),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        crate::fetch::test_support::serve_synthetic_saxony_anhalt(
            fetcher,
            &project.bbox().unwrap(),
        );
        (dir, cache_dir, cache)
    }

    #[test]
    fn saxony_anhalt_project_fetches_lvermgeo_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("29.04.2026")
        );
        assert!(project.terrain.is_some());
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("13.05.2024"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [Region::SaxonyAnhalt]);
        let foreign: Vec<String> = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("geobasis-bb") || u.contains("sachsen.de"))
            .cloned()
            .collect();
        assert!(foreign.is_empty(), "{foreign:?}");
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Saxony-Anhalt from 2026-11-06"),
            "{messages:?}"
        );
    }

    #[test]
    fn saxony_anhalt_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
        // The templates lie in zone 32.
        let world = std::fs::read_to_string(dir.path().join("templates/hillshade.pgw")).unwrap();
        let west: f64 = world.lines().nth(4).unwrap().parse().unwrap();
        assert!((700_000.0..710_000.0).contains(&west), "{world}");
    }

    #[test]
    fn saxony_anhalt_attribution_and_georeferencing() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let map = export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(
            attribution.contains("© GeoBasis-DE / LVermGeo ST 2026, dl-de/by-2-0, Daten geändert"),
            "{attribution}"
        );
        assert!(
            !attribution.contains("GeoSN") && !attribution.contains("GeoBasis-DE/LGB"),
            "{attribution}"
        );
        let xml = std::fs::read_to_string(map).unwrap();
        assert!(
            xml.contains("<parameter>25832</parameter>"),
            "georeferenced in zone 32"
        );
        let ocad = export_ocad(dir.path(), false).unwrap();
        let file = crate::ocd::read::read(&std::fs::read(&ocad.map).unwrap());
        let georef = &file.strings.iter().find(|s| s.kind == 1039).unwrap().text;
        assert!(georef.contains("\ti8007\t"), "{georef}");
    }

    /// A square of `side` metres along the EPSG:25833 axes around the Latina.
    fn latina_square(side: f64) -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 289_795.0 + e,
                northing: 5_707_067.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let h = side / 2.0;
        let ring = [
            corner(-h, -h),
            corner(h, -h),
            corner(h, h),
            corner(-h, h),
            corner(-h, -h),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    #[test]
    fn size_is_checked_in_the_map_crs() {
        // 3950 m along the zone-33 axes plus 2 × 20 m buffer fits in 25833, but the grids are
        // rotated by about 4.7°, so the zone-32 box is about 4300 m wide.
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let area = latina_square(3950.0);
        assert!(area.projected_bbox().buffered(20.0).width() <= MAX_AREA_SIDE_M);
        let err = init_project(
            &dir.path().join("big"),
            &area,
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::AreaTooLarge { .. }), "{err:?}");
    }

    #[test]
    fn stored_terrain_is_cut_to_the_current_area() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        // A smaller buffer: the stored DGM is larger than the new area.
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m = 5.0;
        project.save(dir.path()).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
    }

    #[test]
    fn fetch_adopts_the_crs_of_the_area() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        // As if written by hand or by an older version: the Halle area recorded as 25833.
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.crs = 25833;
        project.save(dir.path()).unwrap();
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: the map area lies in UTM zone 32")),
            "{messages:?}"
        );
    }

    #[test]
    fn border_project_in_zone_32_names_the_empty_side() {
        // Centre in Saxony-Anhalt 150 m from the Brandenburg border.
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let (e, n) = (311_932.0, 5_820_793.0);
        let ring = [
            corner(e - 100.0, n - 100.0),
            corner(e + 100.0, n - 100.0),
            corner(e + 100.0, n + 100.0),
            corner(e - 100.0, n + 100.0),
            corner(e - 100.0, n - 100.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(dir.path(), &area, MapKind::Sprint, None, date, None).unwrap();
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_saxony_anhalt(
            &fetcher,
            &project.bbox().unwrap(),
        );
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(&[], None),
        );
        fetcher.insert_prefix(
            crate::sources::alkis::LAND_USE_ENDPOINT,
            crate::sources::alkis::test_support::page(&[], None),
        );
        fetcher.insert(
            crate::sources::osm::STATE_URL,
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            crate::sources::osm::EXTRACT_URL,
            std::fs::read(crate::fetch::test_support::OSM_FIXTURE).unwrap(),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("the map area reaches beyond the state border")),
            "{messages:?}"
        );
    }

    /// A 200 m square at the Gymnasium Eppendorf, Hamburg.
    fn eppendorf_area() -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 168_078.0 + e,
                northing: 5_949_079.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-100.0, -100.0),
            corner(100.0, -100.0),
            corner(100.0, 100.0),
            corner(-100.0, 100.0),
            corner(-100.0, -100.0),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    /// A schoolyard project at the Gymnasium Eppendorf, served with synthetic LGV data.
    fn eppendorf_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(
            dir.path(),
            &eppendorf_area(),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        crate::fetch::test_support::serve_synthetic_hamburg(fetcher, &project.bbox().unwrap());
        (dir, cache_dir, cache)
    }

    #[test]
    fn hamburg_project_fetches_lgv_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::Lgv));
        assert!(project.tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("19.03.2026")
        );
        assert!(project.terrain.is_some() && project.surface.is_some());
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [Region::Hamburg]);
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Hamburg from 2026-11-06"),
            "{messages:?}"
        );
    }

    #[test]
    fn hamburg_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn hamburg_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(
            attribution.contains(
                "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) 2026, dl-de/by-2-0, Daten geändert"
            ),
            "{attribution}"
        );
        assert!(
            !attribution.contains("LVermGeo") && !attribution.contains("GeoBasis-DE/LGB"),
            "{attribution}"
        );
    }

    /// A 200 m square at the Lessing-Gymnasium, Frankfurt am Main (25833 E 47838, N 5571740).
    fn frankfurt_area() -> MapArea {
        frankfurt_area_between(-100.0, 100.0)
    }

    /// A rectangle 200 m high around the Frankfurt school, from `west` to `east` metres off it
    /// (the school's tile ends 484 m east of it).
    fn frankfurt_area_between(west: f64, east: f64) -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 47_838.0 + e,
                northing: 5_571_740.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(west, -100.0),
            corner(east, -100.0),
            corner(east, 100.0),
            corner(west, 100.0),
            corner(west, -100.0),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    /// A schoolyard project at the Lessing-Gymnasium, served with synthetic HVBG data.
    fn frankfurt_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        frankfurt_school_of(fetcher, -100.0, 100.0)
    }

    fn frankfurt_school_of(
        fetcher: &MapFetcher,
        west: f64,
        east: f64,
    ) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project = init_project(
            dir.path(),
            &frankfurt_area_between(west, east),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        crate::fetch::test_support::serve_synthetic_hessen(fetcher, &project.bbox().unwrap());
        (dir, cache_dir, cache)
    }

    #[test]
    fn hessen_project_fetches_hvbg_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::Hvbg));
        assert!(project.tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("19.08.2025")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2023-02-28"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2023-02-28"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [Region::Hessen]);
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Hesse from 2026-11-06"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.starts_with("warning")),
            "{messages:?}"
        );
    }

    #[test]
    fn hessen_dom1_failure_keeps_a_covering_surface_and_drops_another() {
        use crate::sources::hvbg::{self, Model};
        for shrink in [false, true] {
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
            fetch(dir.path(), &fetcher, &cache);
            if shrink {
                let mut project = ProjectFile::load(dir.path()).unwrap();
                project.surface.as_mut().unwrap().bbox[2] -= 50.0;
                project.save(dir.path()).unwrap();
            }
            // A fresh cache: the DOM1 must be read again, and its product tree fails.
            let cache_dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(cache_dir.path().to_path_buf());
            fetcher
                .failing
                .borrow_mut()
                .push(hvbg::tree_url(Model::Dom1));
            let (project, messages) = fetch(dir.path(), &fetcher, &cache);
            let unavailable = format!("warning: {} unavailable", Model::Dom1.label());
            let warning = messages
                .iter()
                .find(|m| m.starts_with(&unavailable))
                .unwrap_or_else(|| panic!("{messages:?}"));
            if shrink {
                assert!(warning.ends_with("no nDSM template"), "{warning}");
                assert!(project.surface.is_none());
                assert!(!dir.path().join(SURFACE_RELPATH).exists());
            } else {
                assert!(
                    warning.contains("kept the surface model fetched on"),
                    "{warning}"
                );
                assert!(project.surface.is_some());
                assert!(dir.path().join(SURFACE_RELPATH).exists());
            }
        }
    }

    #[test]
    fn hessen_fetch_warns_about_unresolved_places() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        // A second place of the metadata list that the Downloadcenter does not know.
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        let tiles = crate::tiles::tiles_covering(&bbox);
        let mut rows: Vec<_> = tiles
            .iter()
            .map(|&t| {
                (
                    t,
                    44985,
                    "Frankfurt am Main",
                    "Kreisfreie Stadt Frankfurt am Main",
                )
            })
            .collect();
        rows.insert(0, (tiles[0], 44985, "Gemeindefreies Gebiet", "Nirgendwo"));
        fetcher.insert(
            crate::sources::hvbg::test_support::METADATA_URL,
            crate::sources::hvbg::test_support::metadata_xlsx(&rows),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        // Said once (not per model), and without claiming empty templates: the tiles came from
        // Frankfurt.
        let said: Vec<&String> = messages
            .iter()
            .filter(|m| m.contains("Gemeindefreies Gebiet"))
            .collect();
        assert_eq!(
            said,
            ["warning: the Downloadcenter has no file for \"Gemeindefreies Gebiet (Nirgendwo)\""],
            "{messages:?}"
        );
    }

    #[test]
    fn hessen_fetch_reports_an_unreadable_flight_date_once() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school_of(&fetcher, -40.0, 460.0);
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        let tiles = crate::tiles::tiles_covering(&bbox);
        let rows: Vec<_> = tiles
            .iter()
            .enumerate()
            .map(|(i, &t)| {
                (
                    t,
                    if i == 0 { 0 } else { 44985 },
                    "Frankfurt am Main",
                    "Kreisfreie Stadt Frankfurt am Main",
                )
            })
            .collect();
        fetcher.insert(
            crate::sources::hvbg::test_support::METADATA_URL,
            crate::sources::hvbg::test_support::metadata_xlsx(&rows),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        let said = messages
            .iter()
            .filter(|m| m.starts_with("warning: the metadata list has no readable Bildflugdatum"))
            .count();
        assert_eq!(said, 1, "{messages:?}");
    }

    #[test]
    fn hessen_fetch_names_the_tiles_an_unresolved_place_left_empty() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school_of(&fetcher, -40.0, 460.0);
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        let tiles = crate::tiles::tiles_covering(&bbox);
        // Nobody else holds the first tile.
        let mut rows = vec![(tiles[0], 44985, "Gemeindefreies Gebiet", "Nirgendwo")];
        rows.extend(tiles[1..].iter().map(|&t| {
            (
                t,
                44985,
                "Frankfurt am Main",
                "Kreisfreie Stadt Frankfurt am Main",
            )
        }));
        fetcher.insert(
            crate::sources::hvbg::test_support::METADATA_URL,
            crate::sources::hvbg::test_support::metadata_xlsx(&rows),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        let name = crate::sources::lgv::tile_names(&tiles[..1]);
        assert!(
            messages.iter().any(|m| m.starts_with(
                "warning: the Downloadcenter has no file for \"Gemeindefreies Gebiet (Nirgendwo)\""
            ) && m.contains(&name)),
            "{messages:?}"
        );
    }

    #[test]
    fn hessen_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    /// A 200 m square at the Innerstädtisches Gymnasium, Rostock (25833 E 312157, N 5996317).
    fn rostock_area() -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 312_157.0 + e,
                northing: 5_996_317.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-100.0, -100.0),
            corner(100.0, -100.0),
            corner(100.0, 100.0),
            corner(-100.0, 100.0),
            corner(-100.0, -100.0),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    /// A 200 m square at the Coppernicus-Gymnasium, Norderstedt (25833 E 169108, N 5960473).
    fn norderstedt_area() -> MapArea {
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 169_108.0 + e,
                northing: 5_960_473.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-100.0, -100.0),
            corner(100.0, -100.0),
            corner(100.0, 100.0),
            corner(-100.0, 100.0),
            corner(-100.0, -100.0),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    /// A schoolyard project at the Coppernicus-Gymnasium, served with synthetic SH data.
    fn norderstedt_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project = init_project(
            dir.path(),
            &norderstedt_area(),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        crate::fetch::test_support::serve_synthetic_schleswig_holstein(
            fetcher,
            &project.bbox().unwrap(),
        );
        (dir, cache_dir, cache)
    }

    #[test]
    fn schleswig_holstein_project_fetches_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::LvermGeoSh));
        assert!(project.tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("17.03.2025")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2022-03-10"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2023-04-07"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::SchleswigHolstein]
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Schleswig-Holstein from 2026-11-06"),
            "{messages:?}"
        );
        // The surface model (2023) is older than the orthophoto (2025).
        assert!(
            messages.iter().any(|m| m
                == "warning: the surface model (bDOM 2023) is older than the orthophoto (2025); new buildings may be missing in the nDSM"),
            "{messages:?}"
        );
        assert_eq!(
            messages.iter().filter(|m| m.starts_with("warning")).count(),
            1,
            "{messages:?}"
        );
    }

    #[test]
    fn schleswig_holstein_bdom_failure_is_a_warning() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        fetcher.insert_prefix(
            &crate::sources::lvermgeo_sh::overview_prefix(
                crate::sources::lvermgeo_sh::Product::Bdom,
            ),
            b"<html>maintenance</html>".to_vec(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.terrain.is_some() && project.surface.is_none());
        assert!(
            messages.iter().any(|m| m
                .starts_with("warning: surface model (LVermGeo SH bDOM) unavailable")
                && m.ends_with("no nDSM template")),
            "{messages:?}"
        );
    }

    /// The SH school fetched online once, then again in `mode` with the bDOM unavailable
    /// (offline: its tiles gone from the cache; online: the tile list an error page and none
    /// kept); `shrink` makes the stored surface record one of a smaller (another) area first.
    fn refetch_with_bdom_failing(
        mode: FetchMode,
        shrink: bool,
    ) -> (tempfile::TempDir, ProjectFile, Vec<String>) {
        use crate::sources::lvermgeo_sh::{Product, overview_prefix};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert!(dir.path().join(SURFACE_RELPATH).exists());
        if shrink {
            let mut project = ProjectFile::load(dir.path()).unwrap();
            project.surface.as_mut().unwrap().bbox[2] -= 50.0;
            project.save(dir.path()).unwrap();
        }
        let again = MapFetcher::default();
        let fetcher = match mode {
            FetchMode::Offline => {
                std::fs::remove_dir_all(cache.lvermgeo_sh_dir().join("bdom")).unwrap();
                &again
            }
            FetchMode::Online => {
                for entry in std::fs::read_dir(cache.lvermgeo_sh_dir().join("overview")).unwrap() {
                    let path = entry.unwrap().path();
                    if path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("bdom_")
                    {
                        std::fs::remove_file(path).unwrap();
                    }
                }
                fetcher.insert_prefix(
                    &overview_prefix(Product::Bdom),
                    b"<html>maintenance</html>".to_vec(),
                );
                &fetcher
            }
        };
        let mut messages = Vec::new();
        let project = fetch_project(dir.path(), fetcher, &cache, mode, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        // Derive needs a cache only for tile providers.
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        (dir, project, messages)
    }

    #[test]
    fn schleswig_holstein_bdom_failure_on_a_refetch_keeps_the_old_surface() {
        for mode in [FetchMode::Offline, FetchMode::Online] {
            let (dir, project, messages) = refetch_with_bdom_failing(mode, false);
            let surface = project.surface.as_ref().expect("the old surface record");
            let kept = format!("kept the surface model fetched on {}", surface.fetched);
            assert!(
                messages.iter().any(|m| m
                    .starts_with("warning: surface model (LVermGeo SH bDOM) unavailable")
                    && m.ends_with(&kept)),
                "{mode:?}: {messages:?}"
            );
            assert!(project.terrain.is_some());
            assert!(dir.path().join(SURFACE_RELPATH).exists(), "{mode:?}");
            assert!(dir.path().join(NDSM_RELPATH).exists(), "{mode:?}");
        }
    }

    #[test]
    fn schleswig_holstein_bdom_failure_drops_a_surface_of_another_area() {
        // Online the surface of another area goes ...
        let (dir, project, messages) = refetch_with_bdom_failing(FetchMode::Online, true);
        assert!(
            messages.iter().any(|m| m.ends_with("no nDSM template")),
            "{messages:?}"
        );
        assert!(project.terrain.is_some() && project.surface.is_none());
        assert!(!dir.path().join(SURFACE_RELPATH).exists());
        assert!(!dir.path().join(NDSM_RELPATH).exists());
        // ... offline nothing is deleted, but derive does not use it.
        let (dir, project, messages) = refetch_with_bdom_failing(FetchMode::Offline, true);
        assert!(
            messages.iter().any(|m| m.ends_with("no nDSM template")),
            "{messages:?}"
        );
        assert!(project.surface.is_some());
        assert!(dir.path().join(SURFACE_RELPATH).exists());
        assert!(!dir.path().join(NDSM_RELPATH).exists());
    }

    #[test]
    fn schleswig_holstein_ndsm_pipeline_handles_bdom_below_dgm1() {
        // A smoke test: the nDSM is transparent below 0.3 m, so the PNG cannot show the clamp at
        // 0 itself; that is pinned by `derive::ndsm::ndsm_from_bdom_minus_dtm`.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        // The bDOM 5 m below the DGM1 (a removed building, a lowered pit).
        crate::fetch::test_support::serve_schleswig_holstein_models_lifted(&fetcher, &bbox, -5.0);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(0.0));
    }

    #[test]
    fn other_states_attribution_has_no_licence_link() {
        for provider_fetch in 0..2 {
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = if provider_fetch == 0 {
                eppendorf_school(&fetcher)
            } else {
                frankfurt_school(&fetcher)
            };
            fetch(dir.path(), &fetcher, &cache);
            derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
            export_starter_project(dir.path(), false).unwrap();
            let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
            assert!(
                !attribution.contains("creativecommons.org")
                    && !attribution.contains("LVermGeo SH")
                    && !attribution.contains("GeoBasis-DE/MV"),
                "{attribution}"
            );
        }
    }

    #[test]
    fn schleswig_holstein_border_map_reads_both_states_once() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // 25832 E 565500 / N 5948100 (Norderstedt), a box reaching into Hamburg-Langenhorn.
        let c = crate::crs::Crs::Utm33
            .reproject(
                crate::crs::Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 565_500.0,
                    northing: 5_948_100.0,
                },
            )
            .unwrap();
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: c.easting + e,
                northing: c.northing + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-250.0, -450.0),
            corner(250.0, -450.0),
            corner(250.0, 50.0),
            corner(-250.0, 50.0),
            corner(-250.0, -450.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        let bbox = project.bbox().unwrap();
        crate::fetch::test_support::serve_synthetic_hamburg(&fetcher, &bbox);
        crate::fetch::test_support::serve_synthetic_schleswig_holstein(&fetcher, &bbox);
        // Geofabrik's extracts overlap at the border: both hold the same hydrants, one on the
        // Norderstedt side and one in Langenhorn.
        let at = |e: f64, n: f64| {
            crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: c.easting + e,
                northing: c.northing + n,
            })
            .unwrap()
        };
        let (north, south) = (at(0.0, 0.0), at(0.0, -400.0));
        let hydrant: &[(&str, &str)] = &[("emergency", "fire_hydrant")];
        let pbf = crate::fetch::test_support::osm_pbf(&[
            (101, north.lat, north.lon, hydrant),
            (102, south.lat, south.lon, hydrant),
        ]);
        for extract in [
            crate::sources::osm::ExtractSource::Hamburg,
            crate::sources::osm::ExtractSource::SchleswigHolstein,
        ] {
            fetcher.insert(extract.url(), pbf.clone());
        }
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        // The rasters come from the centre's provider (Schleswig-Holstein).
        assert_eq!(project.provider, Some(Provider::LvermGeoSh));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::Hamburg, Region::SchleswigHolstein]
        );
        for extract in ["Hamburg", "Schleswig-Holstein"] {
            let line = format!("reading OSM extract {extract} from 2026-11-06");
            assert_eq!(
                messages.iter().filter(|m| **m == line).count(),
                1,
                "{messages:?}"
            );
        }
        let buildings = stored_buildings(dir.path()).unwrap();
        let oids: Vec<&str> = buildings.iter().map(|b| b.oid.as_str()).collect();
        assert!(oids.iter().any(|o| o.starts_with("DEHH")), "{oids:?}");
        assert!(oids.iter().any(|o| o.starts_with("DESH")), "{oids:?}");
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "no duplicates: {oids:?}");
        // Each OSM candidate once, although both extracts hold it.
        let text = std::fs::read_to_string(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        let ids: Vec<String> = crate::sources::osm::features_from_geojson(&text)
            .unwrap()
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["node/101", "node/102"]);
        assert_eq!(project.osm.unwrap().count, 2);
    }

    #[test]
    fn schleswig_holstein_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        // bDOM (terrain + 12 m) minus DGM1.
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn schleswig_holstein_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(
            attribution.contains("© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)"),
            "{attribution}"
        );
        assert!(
            attribution.contains("https://creativecommons.org/licenses/by/4.0/"),
            "{attribution}"
        );
        assert!(
            !attribution.contains("HVBG")
                && !attribution.contains("GeoBasis-DE/LGB")
                && !attribution.contains("GeoBasis-DE/MV"),
            "{attribution}"
        );
    }

    #[test]
    fn schleswig_holstein_orthophoto_is_reused_while_the_flight_is_unchanged() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        let getmaps = |fetcher: &MapFetcher| {
            fetcher
                .log
                .borrow()
                .iter()
                .filter(|u| u.contains("WMS_SH_DOP20col_OpenGBD") && u.contains("REQUEST=GetMap"))
                .count()
        };
        fetch(dir.path(), &fetcher, &cache);
        assert_eq!(getmaps(&fetcher), 1);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(getmaps(&fetcher), 1);
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (flight 17.03.2025)"),
            "{messages:?}"
        );
    }

    #[test]
    fn offline_fetch_keeps_the_schleswig_holstein_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = norderstedt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let earlier = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        for record in [project.terrain.as_mut(), project.surface.as_mut()] {
            record.unwrap().fetched = earlier;
        }
        project.save(dir.path()).unwrap();
        let project = fetch_project(
            dir.path(),
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.surface.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2022-03-10"));
    }

    #[test]
    fn hamburg_to_schleswig_holstein_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(Provider::Lgv));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &norderstedt_area()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm32, "the same zone");
        assert_eq!(project.provider, Some(Provider::LvermGeoSh));
        assert!(project.terrain.is_none() && project.surface.is_none());
        assert!(project.orthophoto.is_none() && project.buildings.is_none());
        assert!(!dir.path().join(TERRAIN_RELPATH).exists());
        assert!(!dir.path().join(SURFACE_RELPATH).exists());
        assert!(!dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
    }

    #[test]
    fn hessen_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        let year = chrono::Local::now().year();
        assert!(
            attribution.contains(&format!(
                "Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) {year}, dl-de/zero-2-0, verändert"
            )),
            "{attribution}"
        );
        assert!(
            !attribution.contains("LVermGeo") && !attribution.contains("GeoBasis-DE/LGB"),
            "{attribution}"
        );
    }

    fn hessen_getmap_requests(fetcher: &MapFetcher) -> usize {
        fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("ogc-free-images.ows") && u.contains("REQUEST=GetMap"))
            .count()
    }

    #[test]
    fn hessen_orthophoto_is_reused_while_the_flight_is_unchanged() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert_eq!(hessen_getmap_requests(&fetcher), 1);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(hessen_getmap_requests(&fetcher), 1);
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (flight 19.08.2025)"),
            "{messages:?}"
        );
        // A new flight is downloaded again.
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                crate::sources::hvbg::WMS.url
            ),
            b"bildflug = '02-06-2026'\n".to_vec(),
        );
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(hessen_getmap_requests(&fetcher), 2);
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("02.06.2026")
        );
    }

    #[test]
    fn offline_fetch_keeps_the_hvbg_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = frankfurt_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let earlier = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        for record in [project.terrain.as_mut(), project.surface.as_mut()] {
            record.unwrap().fetched = earlier;
        }
        project.save(dir.path()).unwrap();
        let project = fetch_project(
            dir.path(),
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.surface.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2023-02-28"));
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("19.08.2025")
        );
        assert!(dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
    }

    #[test]
    fn provider_change_to_and_from_hesse_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert!(dir.path().join(TERRAIN_RELPATH).exists());
        let project = set_map_area(dir.path(), &frankfurt_area()).unwrap();
        assert_eq!(project.provider, Some(Provider::Hvbg));
        assert!(project.terrain.is_none() && project.orthophoto.is_none());
        assert!(project.buildings.is_none());
        assert!(!dir.path().join(TERRAIN_RELPATH).exists());
        assert!(!dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());

        crate::fetch::test_support::serve_synthetic_hessen(&fetcher, &project.bbox().unwrap());
        fetch(dir.path(), &fetcher, &cache);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(Provider::Hvbg));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &eppendorf_area()).unwrap();
        assert_eq!(project.provider, Some(Provider::Lgv));
        assert!(project.terrain.is_none() && project.surface.is_none());
        assert!(project.orthophoto.is_none() && project.buildings.is_none());
        assert!(!dir.path().join(TERRAIN_RELPATH).exists());
        assert!(!dir.path().join(SURFACE_RELPATH).exists());
        assert!(!dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
    }

    #[test]
    fn provider_change_in_a_zone_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(Provider::LvermGeo));
        assert!(before.terrain.is_some());
        let project = set_map_area(dir.path(), &eppendorf_area()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm32, "the same zone");
        assert_eq!(project.provider, Some(Provider::Lgv));
        assert!(project.terrain.is_none() && project.orthophoto.is_none());
        assert!(project.buildings.is_none());
        assert!(!dir.path().join(TERRAIN_RELPATH).exists());
    }

    #[test]
    fn older_saxony_anhalt_project_moved_to_hamburg_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        // Saved before projects recorded their provider.
        let mut older = ProjectFile::load(dir.path()).unwrap();
        older.provider = None;
        older.save(dir.path()).unwrap();
        let project = set_map_area(dir.path(), &eppendorf_area()).unwrap();
        assert!(project.terrain.is_none() && project.surface.is_none());
        assert!(!dir.path().join(SURFACE_RELPATH).exists());
    }

    #[test]
    fn provider_drop_is_saved_before_the_fetch() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        // The map area edited by hand into Hamburg.
        let mut moved = ProjectFile::load(dir.path()).unwrap();
        moved.map_area = eppendorf_area().to_geojson_value();
        moved.save(dir.path()).unwrap();
        // Offline nothing of Hamburg is cached: the fetch fails after the drop.
        assert!(
            fetch_project(
                dir.path(),
                &MapFetcher::default(),
                &cache,
                FetchMode::Offline,
                &mut |_| {}
            )
            .is_err()
        );
        let after = ProjectFile::load(dir.path()).unwrap();
        assert!(after.terrain.is_none() && after.orthophoto.is_none());
    }

    #[test]
    fn offline_fetch_keeps_the_lgv_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let earlier = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        for record in [project.terrain.as_mut(), project.surface.as_mut()] {
            record.unwrap().fetched = earlier;
        }
        project.save(dir.path()).unwrap();
        let project = fetch_project(
            dir.path(),
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.unwrap().fetched, earlier);
        assert_eq!(project.surface.unwrap().fetched, earlier);
    }

    #[test]
    fn brandenburg_project_stays_25833() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm33);
        assert_eq!(project.bbox().unwrap(), project.region_bbox().unwrap());
    }

    #[test]
    fn moving_across_the_zone_line_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert!(!before.tiles.is_empty() && before.buildings.is_some() && before.osm.is_some());
        let project = set_map_area(dir.path(), &latina_area()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        assert!(project.buildings.is_none() && project.land_use.is_none());
        assert!(project.osm.is_none() && project.orthophoto.is_none());
        for relpath in [BUILDINGS_RELPATH, LAND_USE_RELPATH, OSM_CANDIDATES_RELPATH] {
            assert!(
                !dir.path().join(relpath).exists(),
                "{relpath} of the old zone"
            );
        }
        // Moving within a zone keeps them (they are checked against the new area on fetch).
        let back = set_map_area(dir.path(), &fixture_area()).unwrap();
        assert_eq!(back.crs(), crate::crs::Crs::Utm33);
    }

    /// A schoolyard project at the Innerstaedtisches Gymnasium in Rostock, served with
    /// synthetic MV data.
    fn rostock_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project = init_project(
            dir.path(),
            &rostock_area(),
            MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
        crate::fetch::test_support::serve_synthetic_mecklenburg_vorpommern(
            fetcher,
            &project.bbox().unwrap(),
        );
        (dir, cache_dir, cache)
    }

    #[test]
    fn mv_project_fetches_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm33);
        assert_eq!(project.provider, Some(Provider::LaivMv));
        assert!(project.tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("14.08.2024")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-03-01"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2024-08-01"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::MecklenburgVorpommern]
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Mecklenburg-Vorpommern from 2026-11-06"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.starts_with("warning")),
            "{messages:?}"
        );
    }

    #[test]
    fn mv_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        // DOM1 (terrain + 12 m) minus DGM1.
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn mv_contours_name_the_mv_terrain_model() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        // One contour line through the map area, whatever the synthetic terrain gives.
        let p = |de: f64| {
            let c = (312_157.0 + de, 5_996_317.0 + 10.0);
            (c.0, c.1)
        };
        let line = crate::derive::contours::ContourLine {
            height: 10.0,
            points: vec![p(-40.0), p(40.0)],
        };
        std::fs::write(
            dir.path().join(CONTOURS_RELPATH),
            crate::derive::contours::contours_to_geojson(&[line]),
        )
        .unwrap();
        export_starter_project(dir.path(), true).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(
            candidates.contains(r#"<t k="freemapper:source">laiv-mv-dgm1</t>"#),
            "the contours come from the LAiV M-V DGM1"
        );
        assert!(!candidates.contains("lgb-dgm1"));
    }

    #[test]
    fn mv_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(
            attribution.contains("©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)"),
            "{attribution}"
        );
        assert!(
            attribution.contains("https://creativecommons.org/licenses/by/4.0/"),
            "{attribution}"
        );
        assert!(
            !attribution.contains("LVermGeo SH") && !attribution.contains("GeoBasis-DE/LGB"),
            "{attribution}"
        );
    }

    #[test]
    fn mv_credit_and_licence_link_are_not_repeated_across_the_sh_border() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let (mut project, _) = fetch(dir.path(), &fetcher, &cache);
        // Objects of both states were fetched (a map at the Luebeck bay).
        project.buildings.as_mut().unwrap().regions =
            vec![Region::SchleswigHolstein, Region::MecklenburgVorpommern];
        let lines = attribution_lines(&project, &[]);
        assert!(lines.iter().any(|l| l == MV_ATTRIBUTION), "{lines:?}");
        assert!(lines.iter().any(|l| l == SH_ATTRIBUTION), "{lines:?}");
        let links = lines
            .iter()
            .filter(|l| l.contains("creativecommons.org"))
            .count();
        assert_eq!(links, 1, "{lines:?}");
    }

    #[test]
    fn offline_without_a_tile_list_keeps_the_stored_mv_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        // A cache that lost the portal's tile lists.
        let empty_dir = tempfile::tempdir().unwrap();
        let empty = Cache::new(empty_dir.path().to_path_buf());
        let project = fetch_project(
            dir.path(),
            &MapFetcher::default(),
            &empty,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-03-01"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2024-08-01"));
    }

    #[test]
    fn mv_ndsm_clamps_a_surface_below_the_terrain() {
        // The nDSM is transparent below 0.3 m: the PNG shows the clamp as the colour of 0.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut surface = read_float_raster(&dir.path().join(TERRAIN_RELPATH)).unwrap();
        surface.data.iter_mut().for_each(|z| *z -= 5.0);
        crate::sources::float_tiff::write_float_raster(
            &surface,
            dir.path(),
            &dir.path().join(SURFACE_RELPATH),
            "test",
        )
        .unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(0.0));
    }

    #[test]
    fn mv_to_brandenburg_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(Provider::LaivMv));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &fixture_area()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm33, "the same zone");
        assert_eq!(project.provider, Some(Provider::Lgb));
        assert!(project.terrain.is_none() && project.surface.is_none());
        assert!(project.orthophoto.is_none() && project.buildings.is_none());
        for relpath in [TERRAIN_RELPATH, SURFACE_RELPATH, WMS_ORTHOPHOTO_RELPATH] {
            assert!(!dir.path().join(relpath).exists(), "{relpath} of MV");
        }
    }

    #[test]
    fn mv_orthophoto_is_reused_while_the_flight_is_unchanged() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let getcoverages = |fetcher: &MapFetcher| {
            fetcher
                .log
                .borrow()
                .iter()
                .filter(|u| u.contains("inspire_oi_dop_wcs") && u.contains("GetCoverage"))
                .count()
        };
        fetch(dir.path(), &fetcher, &cache);
        let first = getcoverages(&fetcher);
        assert!(first >= 1);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(getcoverages(&fetcher), first);
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (flight 14.08.2024)"),
            "{messages:?}"
        );
    }

    #[test]
    fn mv_orthophoto_without_any_flight_date_is_reused() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        // The portal lists no dop20 tile for the centre: no flight date, now or later.
        fetcher.insert_prefix(
            &format!("{}?type[]=dop20&bbox=", crate::sources::laiv_mv::OVERVIEW),
            crate::sources::laiv_mv::test_support::dop20_overview_answer(&[], "2024-08-14"),
        );
        let getcoverages = |fetcher: &MapFetcher| {
            fetcher
                .log
                .borrow()
                .iter()
                .filter(|u| u.contains("inspire_oi_dop_wcs") && u.contains("GetCoverage"))
                .count()
        };
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.orthophoto.as_ref().unwrap().flight, None);
        let first = getcoverages(&fetcher);
        assert!(first >= 1);
        let age = |days: i64| {
            let mut project = ProjectFile::load(dir.path()).unwrap();
            project.orthophoto.as_mut().unwrap().fetched =
                chrono::Local::now().date_naive() - chrono::Duration::days(days);
            project.save(dir.path()).unwrap();
        };
        // 29 days old: reused.
        age(29);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(getcoverages(&fetcher), first, "{messages:?}");
        assert!(
            messages.iter().any(|m| m == "orthophoto unchanged"),
            "{messages:?}"
        );
        // 31 days old: fetched again.
        age(31);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(getcoverages(&fetcher) > first, "{messages:?}");
    }

    #[test]
    fn offline_fetch_keeps_the_mv_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let earlier = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        for record in [project.terrain.as_mut(), project.surface.as_mut()] {
            record.unwrap().fetched = earlier;
        }
        project.save(dir.path()).unwrap();
        let project = fetch_project(
            dir.path(),
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.surface.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-03-01"));
    }

    #[test]
    fn mv_border_map_masks_the_brandenburg_side() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Brüssow: the MV tile row N 5918 exists, the row N 5916 (Brandenburg) does not. The
        // centre lies 100 m inside MV; the box reaches south into Brandenburg.
        let corner = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 437_000.0 + e,
                northing: 5_918_700.0 + n,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-100.0, -800.0),
            corner(100.0, -800.0),
            corner(100.0, 800.0),
            corner(-100.0, 800.0),
            corner(-100.0, -800.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        let bbox = project.bbox().unwrap();
        crate::fetch::test_support::serve_synthetic_mecklenburg_vorpommern(&fetcher, &bbox);
        // The Brandenburg side: one building, no land use, the Brandenburg OSM extract. Geofabrik's
        // extracts overlap at the border: both hold the same hydrants, one on each side.
        let bb_building = crate::sources::alkis::test_support::building_feature(
            "DEBBAL0100000001",
            "Gebäude",
            "Wohnhaus",
            [436_950.0, 5_917_950.0, 10.0],
        );
        fetcher.insert_prefix(
            crate::sources::alkis::BUILDINGS_ENDPOINT,
            crate::sources::alkis::test_support::page(&[bb_building], None),
        );
        let at = |n: f64| {
            crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 437_000.0,
                northing: n,
            })
            .unwrap()
        };
        let (north, south) = (at(5_919_000.0), at(5_918_100.0));
        let hydrant: &[(&str, &str)] = &[("emergency", "fire_hydrant")];
        let pbf = crate::fetch::test_support::osm_pbf(&[
            (101, north.lat, north.lon, hydrant),
            (102, south.lat, south.lon, hydrant),
        ]);
        fetcher.insert_prefix(
            crate::sources::alkis::LAND_USE_ENDPOINT,
            crate::sources::alkis::test_support::page(&[], None),
        );
        fetcher.insert(
            crate::sources::osm::STATE_URL,
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        for extract in [
            crate::sources::osm::ExtractSource::Brandenburg,
            crate::sources::osm::ExtractSource::MecklenburgVorpommern,
        ] {
            fetcher.insert(extract.url(), pbf.clone());
        }
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        // Both states' ALKIS, each OSM extract once, nothing twice.
        assert_eq!(
            project.buildings.as_ref().unwrap().regions,
            [Region::Brandenburg, Region::MecklenburgVorpommern]
        );
        for extract in ["Brandenburg", "Mecklenburg-Vorpommern"] {
            let line = format!("reading OSM extract {extract} from 2026-11-06");
            assert_eq!(
                messages.iter().filter(|m| **m == line).count(),
                1,
                "{messages:?}"
            );
        }
        let buildings = stored_buildings(dir.path()).unwrap();
        let oids: Vec<&str> = buildings.iter().map(|b| b.oid.as_str()).collect();
        assert!(oids.iter().any(|o| o.starts_with("DEBB")), "{oids:?}");
        assert!(oids.iter().any(|o| o.starts_with("DEMV")), "{oids:?}");
        let unique: std::collections::HashSet<&&str> = oids.iter().collect();
        assert_eq!(unique.len(), oids.len(), "no duplicates: {oids:?}");
        let text = std::fs::read_to_string(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        let ids: Vec<String> = crate::sources::osm::features_from_geojson(&text)
            .unwrap()
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["node/101", "node/102"]);
        assert_eq!(project.osm.as_ref().unwrap().count, 2);
        // The rasters come from the centre's provider (MV).
        assert_eq!(project.provider, Some(Provider::LaivMv));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert!(
            messages.iter().any(|m| m.starts_with(
                "the map area reaches beyond the state border; beyond it the templates are empty wherever the LAiV M-V orthophoto and height models end"
            )),
            "{messages:?}"
        );
        for relpath in [TERRAIN_RELPATH, SURFACE_RELPATH] {
            let raster = read_float_raster(&dir.path().join(relpath)).unwrap();
            let (w, h) = (raster.grid.width, raster.grid.height);
            // North (MV) is a height, the Brandenburg side NaN: no cliff to zero.
            assert!(raster.get(w / 2, 5).is_finite(), "{relpath} north");
            assert!(raster.get(w / 2, h - 5).is_nan(), "{relpath} south");
            assert!(
                !(0..h).any(|y| raster.get(w / 2, y) == 0.0),
                "{relpath}: no zero cells"
            );
        }
    }

    #[test]
    fn brandenburg_to_mv_drops_records() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(Provider::Lgb));
        assert!(!before.tiles.is_empty() && before.buildings.is_some() && before.osm.is_some());
        let project = set_map_area(dir.path(), &rostock_area()).unwrap();
        assert_eq!(project.crs(), crate::crs::Crs::Utm33, "the same zone");
        assert_eq!(project.provider, Some(Provider::LaivMv));
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        assert!(project.buildings.is_none() && project.land_use.is_none());
        assert!(project.osm.is_none() && project.orthophoto.is_none());
        for relpath in [BUILDINGS_RELPATH, LAND_USE_RELPATH, OSM_CANDIDATES_RELPATH] {
            assert!(
                !dir.path().join(relpath).exists(),
                "{relpath} of Brandenburg"
            );
        }
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
            crs: 25833,
            provider: Some(Provider::Lgb),
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
            berlin_layers: BTreeMap::new(),
            nas_layers: BTreeMap::from([(
                "sonstiges-bauwerk".to_string(),
                AlkisRecord {
                    bbox: [421_490.0, 5_821_580.0, 421_737.0, 5_821_897.0],
                    fetched: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
                    latest_update: NaiveDate::from_ymd_opt(2020, 11, 24),
                    count: 4,
                    regions: vec![Region::Brandenburg],
                },
            )]),
            tree_layers: BTreeMap::from([(
                "potsdam".to_string(),
                AlkisRecord {
                    bbox: [366_200.0, 5_807_100.0, 367_100.0, 5_807_700.0],
                    fetched: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
                    latest_update: NaiveDate::from_ymd_opt(2026, 3, 2),
                    count: 3,
                    regions: vec![Region::Brandenburg],
                },
            )]),
            k5: None,
            osm: Some(OsmRecord {
                extract: NaiveDate::from_ymd_opt(2026, 11, 6).unwrap(),
                area: [52.5385, 13.8418, 52.5418, 13.8457],
                count: 7,
            }),
            orthophoto: Some(WmsRecord {
                flight: Some("19.03. + 07.04.2024".to_string()),
                flight_label: None,
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                bbox: [415_858.0, 5_655_635.0, 416_089.0, 5_655_854.0],
            }),
            terrain: Some(WcsRecord {
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                bbox: [706_350.0, 5_706_800.0, 706_550.0, 5_707_000.0],
                date: None,
            }),
            surface: Some(WcsRecord {
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                bbox: [706_350.0, 5_706_800.0, 706_550.0, 5_707_000.0],
                date: Some("13.05.2024".to_string()),
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
        assert_eq!(
            project.crs(),
            crate::crs::Crs::Utm33,
            "older projects are 25833"
        );
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
        // Building fill (black 50 %, ISSprOM 2019-2), tartan pitch candidate (lower brown 50 %),
        // contours (brown), in the CMYK of IOF Printing and Colour Definitions Rev. 4.
        assert!(contains(&pdf, b"0 0 0 0.5 k"), "building");
        assert!(contains(&pdf, b"0.1 0.35 0.5 0 k"), "pitch");
        assert!(contains(&pdf, b"0.25 0.75 1 0 K"), "contours");
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

    /// Adds a vereinfacht carport and a passage, 10 m east of the first stored building, to the
    /// stored buildings of a fetched project: the way every state but Berlin stores them.
    fn add_state_structures(dir: &Path) {
        let path = dir.join(BUILDINGS_RELPATH);
        let mut collection: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let points: Vec<(f64, f64)> = collection["features"][0]["geometry"]["coordinates"]
            .to_string()
            .trim_matches(['[', ']'])
            .split("],[")
            .map(|p| {
                let (e, n) = p.split_once(',').unwrap();
                (e.parse().unwrap(), n.parse().unwrap())
            })
            .collect();
        let east = points.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let north = points.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let square = |id: &str, kind: &str, function: &str, offset: f64| {
            let (e, n) = (east + offset, north);
            serde_json::json!({
                "type": "Feature", "id": id,
                "geometry": {"type": "MultiPolygon", "coordinates": [[[
                    [e, n], [e + 6.0, n], [e + 6.0, n + 6.0], [e, n + 6.0], [e, n]]]]},
                "properties": {"oid": id, "aktualit": "2024-04-22T07:00:00Z",
                               "gebnutzbez": kind, "funktion": function}})
        };
        let features = collection["features"].as_array_mut().unwrap();
        features.push(square(
            "DEBBAL0000CARPORT",
            "Sonstiges Bauwerk oder sonstige Einrichtung",
            "Carport",
            10.0,
        ));
        features.push(square(
            "DEBBAL00PASSAGE",
            "Bauteil",
            "Durchfahrt im Gebäude",
            20.0,
        ));
        std::fs::write(path, collection.to_string()).unwrap();
    }

    #[test]
    fn ocad_export_has_state_canopies() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        add_state_structures(dir.path());
        let outcome = export_ocad(dir.path(), true).unwrap();
        let map = crate::ocd::read::read(&std::fs::read(&outcome.map).unwrap());
        assert!(map.objects.iter().any(|o| o.symbol == 521_000), "building");
        assert_eq!(
            map.objects.iter().filter(|o| o.symbol == 522_000).count(),
            1,
            "the carport is a map object"
        );
        // The passage is a candidate, in both OCAD and OOM.
        let candidates = crate::ocd::read::read(&std::fs::read(&outcome.candidates).unwrap());
        assert_eq!(
            candidates
                .objects
                .iter()
                .filter(|o| o.symbol == 522_000)
                .count(),
            1,
            "the passage is a candidate"
        );
        let documents = starter_documents(dir.path()).unwrap();
        let set = MapKind::Schoolyard.symbol_set();
        let sources: Vec<(String, String)> = documents
            .map
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .filter(|o| Some(o.symbol_id) == set.id_for_code("522"))
            .flat_map(|o| o.tags.clone())
            .filter(|(k, _)| k == "freemapper:source" || k == "freemapper:currency")
            .collect();
        assert_eq!(
            sources,
            [
                ("freemapper:source".to_string(), "alkis".to_string()),
                ("freemapper:currency".to_string(), "2024-04-22".to_string()),
            ]
        );
    }

    #[test]
    fn kept_map_without_state_canopies_is_reported() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        add_state_structures(dir.path());
        // A map from before the canopies: it has the ALKIS buildings, but no 522.
        let map = map_path(dir.path());
        std::fs::write(
            &map,
            format!("edited in OOM {ALKIS_SOURCE_TAG} {ORTHOPHOTO_RELPATH}"),
        )
        .unwrap();
        let run = |force: bool| {
            run_project(
                dir.path(),
                &fetcher,
                &cache,
                FetchMode::Offline,
                force,
                &mut |_| {},
            )
            .unwrap()
            .map
        };
        assert!(matches!(
            run(false),
            MapOutcome::Kept {
                missing_buildings: false,
                missing_canopies: true,
                ..
            }
        ));
        // Written again with force, the map has them.
        assert_eq!(run(true), MapOutcome::Written(map.clone()));
        assert!(matches!(
            run(false),
            MapOutcome::Kept {
                missing_canopies: false,
                ..
            }
        ));
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
                missing_buildings: true,
                missing_canopies: false,
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
                missing_buildings: false,
                missing_canopies: false,
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

    /// `cache` on one volume (with the project folder) that is `short` bytes too small for the
    /// next fetch of the project in `dir`: its downloads, the files derived from them and the
    /// reserve.
    fn cache_short_of(dir: &Path, cache: &Cache, short: u64) -> Cache {
        use crate::space::test_support::FixedSpace;
        let today = chrono::Local::now().date_naive();
        let estimate = crate::layers::estimate_download(dir, cache, today).unwrap();
        let roomy = FixedSpace::everywhere("disk", u64::MAX / 2);
        let needed = crate::space::plan(&estimate, cache.root(), dir, &roomy)
            .unwrap()
            .volumes[0]
            .needed;
        let available = crate::space::RESERVE_BYTES + needed - short;
        cache
            .clone()
            .with_disk_space(std::sync::Arc::new(FixedSpace::everywhere(
                "disk", available,
            )))
    }

    #[test]
    fn parts_that_do_not_fit_are_skipped_with_a_warning() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        let cache = cache_short_of(dir.path(), &Cache::new(cache_dir.path().to_path_buf()), 1);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        let warning = messages
            .iter()
            .find(|m| {
                m.starts_with("warning: freemapper skips the surface model for the nDSM template for lack of disk space. It needs ")
            })
            .unwrap_or_else(|| panic!("{messages:?}"));
        assert!(
            warning.contains(&format!("but {} has only ", cache_dir.path().display()))
                && warning.ends_with(
                    " free beyond the 1.0 GB reserve. Free some space and run `freemapper fetch` again"
                ),
            "{warning}"
        );
        // Nothing of the surface model was downloaded, and no bDOM stands in for it.
        let ndsm_url = crate::sources::lgb::ndsm_tile_url(school_tile());
        assert!(!fetcher.log.borrow().contains(&ndsm_url));
        assert!(!cache.has_ndsm(school_tile()));
        assert!(project.ndsm_tiles.is_empty());
        assert!(!project.tiles.iter().any(|r| r.product == Product::ImageDsm));
        // The parts the run needs are all there.
        assert!(
            project
                .tiles
                .iter()
                .any(|r| r.product == Product::Orthophoto)
        );
        assert!(project.tiles.iter().any(|r| r.product == Product::Dtm));
        assert!(project.osm.is_some() && project.buildings.is_some());
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(!dir.path().join(NDSM_RELPATH).exists());
    }

    #[test]
    fn fetch_with_just_enough_space_skips_nothing() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        let cache = cache_short_of(dir.path(), &Cache::new(cache_dir.path().to_path_buf()), 0);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !messages.iter().any(|m| m.contains("disk space")),
            "{messages:?}"
        );
        assert_eq!(project.ndsm_tiles.len(), 1);
    }

    #[test]
    fn required_part_that_does_not_fit_stops_the_fetch_before_downloading() {
        use crate::space::test_support::FixedSpace;
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let space = FixedSpace::everywhere("disk", crate::space::RESERVE_BYTES + (1 << 20));
        let cache =
            Cache::new(cache_dir.path().to_path_buf()).with_disk_space(std::sync::Arc::new(space));
        let err = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err();
        assert!(matches!(err, ProjectError::NotEnoughSpace(_)), "{err:?}");
        assert!(
            err.to_string()
                .starts_with("not enough disk space for the ALKIS and OSM map data. It needs "),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 0);
        // Offline nothing is downloaded, so nothing is checked.
        let offline = fetch_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            !matches!(offline, ProjectError::NotEnoughSpace(_)),
            "{offline:?}"
        );
    }

    #[test]
    fn skipped_osm_update_keeps_the_cached_extract() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        // The cached extract is a month old, so the next fetch would download a newer one.
        let osm_dir = cache.osm_dir();
        let cached = std::fs::read_dir(&osm_dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|p| p.to_string_lossy().ends_with(".osm.pbf"))
            .unwrap();
        let name = cached.file_name().unwrap().to_string_lossy().to_string();
        let old = osm_dir.join(name.replace("2026-11-06", "2026-10-01"));
        assert_ne!(old, cached);
        std::fs::rename(&cached, &old).unwrap();
        let cache = cache_short_of(dir.path(), &cache, 1);
        fetcher.log.borrow_mut().clear();
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m.starts_with(
                "warning: freemapper skips the update of the cached OSM extract for lack of disk space. It needs "
            )),
            "{messages:?}"
        );
        let log = fetcher.log.borrow();
        assert!(
            !log.iter()
                .any(|u| u == crate::sources::osm::EXTRACT_URL
                    || u == crate::sources::osm::STATE_URL),
            "{log:?}"
        );
        assert!(old.exists());
        assert!(project.osm.is_some());
    }

    #[test]
    fn skipped_surface_model_keeps_the_stored_one() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = halle_school(&fetcher);
        let (before, _) = fetch(dir.path(), &fetcher, &cache);
        let stored = before.surface.clone().expect("a surface model");
        let cache = cache_short_of(dir.path(), &cache, 1);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: freemapper skips the surface model")),
            "{messages:?}"
        );
        let kept = format!(
            "freemapper keeps the surface model fetched on {} for the nDSM template",
            stored.fetched
        );
        assert!(messages.contains(&kept), "{messages:?}");
        assert_eq!(project.surface, Some(stored));
        assert!(dir.path().join(SURFACE_RELPATH).exists());
    }

    #[test]
    fn skipped_surface_model_keeps_the_cached_geosn_tiles() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        let (before, _) = fetch(dir.path(), &fetcher, &cache);
        let dsm: Vec<TileRecord> = before
            .tiles
            .iter()
            .filter(|r| r.product == Product::SaxonyDsm)
            .cloned()
            .collect();
        assert_eq!(dsm.len(), 2, "{:?}", before.tiles);
        // One surface tile is gone from the cache; the other stays.
        let path = |r: &TileRecord| {
            cache.tile_path(
                Product::SaxonyDsm,
                TileId::parse(&r.tile).unwrap(),
                r.published,
            )
        };
        std::fs::remove_file(path(&dsm[0])).unwrap();
        let cache = cache_short_of(dir.path(), &cache, 1);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: freemapper skips the surface model")),
            "{messages:?}"
        );
        let kept = format!(
            "freemapper keeps the cached surface model of tile {} for the nDSM template",
            dsm[1].tile
        );
        assert!(messages.contains(&kept), "{messages:?}");
        assert!(!path(&dsm[0]).exists());
        let now: Vec<&TileRecord> = project
            .tiles
            .iter()
            .filter(|r| r.product == Product::SaxonyDsm)
            .collect();
        assert_eq!(now, [&dsm[1]]);
    }

    #[test]
    fn skipped_surface_model_of_hamburg_keeps_the_stored_one() {
        use crate::sources::lgv::Model;
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        let (before, _) = fetch(dir.path(), &fetcher, &cache);
        let stored = before.surface.clone().expect("a surface model");
        // The bDOM tiles are no longer cached, so the next fetch would read them again.
        let bdom = Model::Bdom.tile_cache_file(&cache, TileId::parse("33000-5000").unwrap());
        let bdom_dir = bdom.parent().unwrap();
        std::fs::remove_dir_all(bdom_dir).unwrap();
        let cache = cache_short_of(dir.path(), &cache, 1);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: freemapper skips the surface model")),
            "{messages:?}"
        );
        let kept = format!(
            "freemapper keeps the surface model fetched on {} for the nDSM template",
            stored.fetched
        );
        assert!(messages.contains(&kept), "{messages:?}");
        assert!(!bdom_dir.exists());
        assert_eq!(project.surface, Some(stored));
        assert!(dir.path().join(SURFACE_RELPATH).exists());
    }

    #[test]
    fn stored_files_of_the_area_count_in_place() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let today = chrono::Local::now().date_naive();
        let before = crate::layers::estimate_download(dir.path(), &cache, today).unwrap();
        assert!(before.in_place.is_empty(), "{:?}", before.in_place);
        fetch(dir.path(), &fetcher, &cache);
        let after = crate::layers::estimate_download(dir.path(), &cache, today).unwrap();
        assert_eq!(
            after.in_place,
            [Part::Terrain, Part::Orthophoto, Part::Surface]
        );
        // The stored orthophoto is reused while its flight date stays; the height models are
        // requested again.
        let orthophoto = after
            .items
            .iter()
            .find(|i| i.part == Part::Orthophoto)
            .unwrap();
        assert!(orthophoto.cached, "{orthophoto:?}");
        let terrain = after
            .items
            .iter()
            .find(|i| i.part == Part::Terrain)
            .unwrap();
        assert!(!terrain.cached, "{terrain:?}");
    }

    #[test]
    fn skipped_surface_model_of_an_area_provider_keeps_the_stored_one() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let (before, _) = fetch(dir.path(), &fetcher, &cache);
        let stored = before.surface.clone().expect("a surface model");
        let cache = cache_short_of(dir.path(), &cache, 1);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: freemapper skips the surface model")),
            "{messages:?}"
        );
        assert!(
            !messages
                .iter()
                .any(|m| m.starts_with("reading the") && m.contains("DOM")),
            "{messages:?}"
        );
        let kept = format!(
            "freemapper keeps the surface model fetched on {} for the nDSM template",
            stored.fetched
        );
        assert!(messages.contains(&kept), "{messages:?}");
        assert_eq!(project.surface, Some(stored));
        assert!(dir.path().join(SURFACE_RELPATH).exists());
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
        serve_k5(fetcher, &ProjectFile::load(dir.path()).unwrap());
        (dir, cache_dir, cache)
    }

    /// A schoolyard project around the Evangelisches Kreuzgymnasium Dresden (straddling the 2 km
    /// tile edge E 416 000), served with synthetic GeoSN data.
    fn dresden_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
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
            corner(415_878.0, 5_655_655.0),
            corner(416_069.0, 5_655_655.0),
            corner(416_069.0, 5_655_834.0),
            corner(415_878.0, 5_655_834.0),
            corner(415_878.0, 5_655_655.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project =
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        crate::fetch::test_support::serve_synthetic_saxony(fetcher, &project.bbox().unwrap());
        (dir, cache_dir, cache)
    }

    #[test]
    fn saxony_project_fetches_geosn_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        let products: Vec<(Product, &str)> = project
            .tiles
            .iter()
            .map(|r| (r.product, r.tile.as_str()))
            .collect();
        assert_eq!(
            products,
            [
                (Product::SaxonyDtm, "33414-5654"),
                (Product::SaxonyDtm, "33416-5654"),
                (Product::SaxonyDsm, "33414-5654"),
                (Product::SaxonyDsm, "33416-5654"),
            ]
        );
        assert!(project.ndsm_tiles.is_empty());
        let orthophoto = project.orthophoto.clone().expect("WMS orthophoto recorded");
        assert_eq!(orthophoto.flight.as_deref(), Some("19.03. + 07.04.2024"));
        assert!(dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
        assert_eq!(project.buildings.as_ref().unwrap().count, 3);
        assert_eq!(project.buildings.unwrap().regions, [Region::Saxony]);
        assert_eq!(project.land_use.unwrap().regions, [Region::Saxony]);
        assert!(
            messages.iter().any(|m| m == "3 ALKIS buildings"),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Saxony from 2026-11-06"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.contains("geobasis-bb")),
            "{messages:?}"
        );
    }

    #[test]
    fn saxony_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let ortho = image::open(dir.path().join(ORTHOPHOTO_RELPATH))
            .unwrap()
            .to_rgb8();
        let centre = ortho.get_pixel(ortho.width() / 2, ortho.height() / 2).0;
        assert!(centre[1] > 150 && centre[0] < 60, "{centre:?}");
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn saxony_attribution_names_geosn() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(attribution.contains(GEOSN_ATTRIBUTION), "{attribution}");
        assert!(!attribution.contains("GeoBasis-DE/LGB"), "{attribution}");
        let project = ProjectFile::load(dir.path()).unwrap();
        assert!(attribution_lines(&project, &[]).contains(&GEOSN_ATTRIBUTION.to_string()));
    }

    #[test]
    fn brandenburg_project_is_unchanged() {
        let fetcher = MapFetcher::default();
        let (dir, cache_dir, cache) = fetched_school(&fetcher);
        let foreign = fetcher
            .log
            .borrow()
            .iter()
            .filter(|url| url.contains("sachsen") || url.contains("berlin.de"))
            .cloned()
            .collect::<Vec<_>>();
        assert!(foreign.is_empty(), "{foreign:?}");
        assert!(
            cache
                .tile_path(Product::Dtm, school_tile(), "2026-01-26".parse().unwrap())
                .starts_with(cache_dir.path().join("lgb"))
        );
        assert!(
            cache
                .osm_dir()
                .join("brandenburg_2026-11-06.osm.pbf")
                .exists()
        );
        let project = ProjectFile::load(dir.path()).unwrap();
        assert!(project.orthophoto.is_none());
        assert!(
            project
                .tiles
                .iter()
                .all(|r| r.product.provider() == crate::sources::region::Provider::Lgb)
        );
        assert!(
            !attribution_lines(&project, &[])
                .iter()
                .any(|l| l.contains("GeoSN"))
        );
    }

    #[test]
    fn unchanged_wms_orthophoto_is_reused() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        fetcher.log.borrow_mut().clear();
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !fetcher
                .log
                .borrow()
                .iter()
                .any(|url| url.contains("REQUEST=GetMap")),
            "{:?}",
            fetcher.log.borrow()
        );
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("orthophoto unchanged")),
            "{messages:?}"
        );
    }

    #[test]
    fn moving_to_brandenburg_removes_the_wms_orthophoto() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert!(dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
        set_map_area(dir.path(), &fixture_area()).unwrap();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetch(dir.path(), &fetcher, &cache);
        assert!(!dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
        assert!(
            !dir.path()
                .join(WMS_ORTHOPHOTO_RELPATH)
                .with_extension("jgw")
                .exists()
        );
        assert!(ProjectFile::load(dir.path()).unwrap().orthophoto.is_none());
    }

    #[test]
    fn border_area_skips_tiles_outside_the_raster_region() {
        // Centre in Brandenburg; the 1 km tile south of it lies in Saxony and LGB does not
        // publish it.
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
            corner(410_400.0, 5_691_020.0),
            corner(410_600.0, 5_691_020.0),
            corner(410_600.0, 5_691_250.0),
            corner(410_400.0, 5_691_250.0),
            corner(410_400.0, 5_691_020.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let project = init_project(dir.path(), &area, MapKind::Sprint, None, date, None).unwrap();
        assert!(crate::tiles::tiles_covering(&project.bbox().unwrap()).len() > 1);
        let fetcher = MapFetcher::default();
        let tile = crate::tiles::TileId::parse("33410-5691").unwrap();
        serve_synthetic_tile(&fetcher, tile, "2026-01-26");
        // Saxon ALKIS and the Saxony extract (empty answers are fine here).
        fetcher.insert_prefix(
            crate::sources::alkis_wfs::GEOSN.url,
            include_bytes!("../../../tests/fixtures/saxony-empty.gml").to_vec(),
        );
        let saxony = crate::sources::osm::ExtractSource::Saxony;
        fetcher.insert(
            saxony.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            saxony.url(),
            std::fs::read(crate::fetch::test_support::OSM_FIXTURE).unwrap(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        let tiles: Vec<&str> = project.tiles.iter().map(|r| r.tile.as_str()).collect();
        assert_eq!(tiles, ["33410-5691", "33410-5691"], "{messages:?}");
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    #[test]
    fn offline_saxony_uses_the_stored_orthophoto() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let offline = MapFetcher::default();
        let project = fetch_project(
            dir.path(),
            &offline,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert!(project.orthophoto.is_some());
        assert_eq!(offline.calls.get(), 0);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    /// A rectangle `half` metres each way around the EPSG:25832 point `(e, n)`, as a map area.
    fn utm32_area_at(e: f64, n: f64, half: f64) -> MapArea {
        let corner = |de: f64, dn: f64| {
            let p = crate::crs::Crs::Utm32
                .inverse(crate::crs::ProjectedPoint {
                    easting: e + de,
                    northing: n + dn,
                })
                .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-half, -half),
            corner(half, -half),
            corner(half, half),
            corner(-half, half),
            corner(-half, -half),
        ];
        MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap()
    }

    /// A schoolyard project of `area`, served with synthetic Geobasis NRW data: the rasters,
    /// the NRW ALKIS fixtures (and the Hesse ones) and every OSM extract the area needs.
    fn nrw_project_of(
        fetcher: &MapFetcher,
        area: &MapArea,
    ) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), area, MapKind::Schoolyard, None, date, None).unwrap();
        crate::fetch::test_support::serve_synthetic_nrw(fetcher, &project.bbox().unwrap());
        use crate::sources::alkis_wfs::{self, test_support as fixtures};
        for (service, buildings, land_use) in [
            (
                alkis_wfs::GEOBASIS_NRW,
                fixtures::NRW_BUILDINGS,
                fixtures::NRW_LAND_USE,
            ),
            (
                alkis_wfs::HVBG,
                fixtures::HE_BUILDINGS,
                fixtures::HE_LAND_USE,
            ),
        ] {
            for (type_name, body) in [
                (service.buildings_type, buildings),
                (service.land_use_type, land_use),
            ] {
                fetcher.insert_prefix(
                    &format!(
                        "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                        service.url
                    ),
                    body.as_bytes().to_vec(),
                );
            }
        }
        for extract in
            crate::sources::osm::extracts_for(&project.region_bbox().unwrap(), &mut |_| {})
        {
            fetcher.insert(
                extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                extract.url(),
                std::fs::read(crate::fetch::test_support::OSM_FIXTURE).expect("OSM fixture"),
            );
        }
        (dir, cache_dir, cache)
    }

    /// The Hildegard-von-Bingen-Gymnasium, Köln (25832 E 354700 / N 5642650), 200 m square.
    fn koeln_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        nrw_project_of(fetcher, &utm32_area_at(354_700.0, 5_642_650.0, 100.0))
    }

    #[test]
    fn nrw_project_fetches_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::GeobasisNrw));
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("11.05.2025")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2022-02-12"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2022-02-12"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::NordrheinWestfalen]
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract North Rhine-Westphalia, Köln from 2026-11-06"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.starts_with("warning")),
            "{messages:?}"
        );
    }

    #[test]
    fn nrw_dom1_failure_keeps_the_project_without_an_ndsm() {
        use crate::sources::geobasis_nrw::{Model, meta_url};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetcher.failing.borrow_mut().push(meta_url(Model::Dom1));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.terrain.is_some() && project.surface.is_none());
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: surface model (Geobasis NRW DOM1) unavailable")),
            "{messages:?}"
        );
        assert!(!dir.path().join(SURFACE_RELPATH).exists());
    }

    #[test]
    fn nrw_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn nrw_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        let year = chrono::Local::now().year();
        let line =
            format!("Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) {year}, dl-de/zero-2-0");
        // dl-de/zero asks for no note of changes.
        assert!(
            attribution.lines().any(|l| l.trim() == line),
            "{attribution}"
        );
        for other in ["HVBG", "LVermGeo", "GeoBasis-DE", "CC BY", "Hamburg"] {
            assert!(!attribution.contains(other), "{other} in {attribution}");
        }
    }

    #[test]
    fn aachen_project_with_negative_eastings_fetches_and_derives() {
        let fetcher = MapFetcher::default();
        // Aachen cathedral, 25833 E -128141 / N 5662838.
        let c = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: -128_141.0,
                    northing: 5_662_838.0,
                },
            )
            .unwrap();
        let (dir, _cache_dir, cache) =
            nrw_project_of(&fetcher, &utm32_area_at(c.easting, c.northing, 100.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.region_bbox().unwrap().max_e < 0.0);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::GeobasisNrw));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert!(project.surface.is_some() && project.buildings.is_some());
        assert!(
            !messages.iter().any(|m| m.starts_with("warning")),
            "{messages:?}"
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(dir.path().join(NDSM_RELPATH).exists());
    }

    #[test]
    fn nrw_project_at_the_hesse_border_takes_rasters_from_the_centre() {
        let fetcher = MapFetcher::default();
        // 25833 E 115329 / N 5735651 is on the border, 1 km south of where Niedersachsen joins; the
        // centre is 150 m inside NRW.
        let c = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: 115_329.0 - 150.0,
                    northing: 5_735_651.0,
                },
            )
            .unwrap();
        let (dir, _cache_dir, cache) =
            nrw_project_of(&fetcher, &utm32_area_at(c.easting, c.northing, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::GeobasisNrw));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::Hessen, Region::NordrheinWestfalen]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                crate::sources::alkis_wfs::HVBG.url,
                crate::sources::alkis_wfs::GEOBASIS_NRW.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(service)), "{service}");
            }
            // The rasters come from NRW alone.
            // Neither the orthophoto WMS nor the Downloadcenter of Hesse is asked (its ALKIS
            // WFS, on www.gds.hessen.de, is).
            for host in ["https://www.gds-srv.hessen.de", "https://gds.hessen.de"] {
                assert!(!log.iter().any(|u| u.starts_with(host)), "{host}");
            }
        }
        assert!(
            messages
                .iter()
                .any(|m| m.contains("reaches beyond the state border")),
            "{messages:?}"
        );
        // Both neighbours are supported states: nothing lies outside them.
        assert!(
            !messages.iter().any(|m| m == BEYOND_COVERAGE_WARNING),
            "{messages:?}"
        );
        for label in ["Hesse", "North Rhine-Westphalia"] {
            let reading: Vec<&String> = messages
                .iter()
                .filter(|m| m.starts_with(&format!("reading OSM extract {label}")))
                .collect();
            assert_eq!(reading.len(), 1, "{label}: {messages:?}");
        }
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    fn nrw_getmap_requests(fetcher: &MapFetcher) -> usize {
        fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("wms_nw_dop") && u.contains("REQUEST=GetMap"))
            .count()
    }

    #[test]
    fn nrw_orthophoto_is_reused_while_the_flight_is_unchanged() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 1);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 1);
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (flight 11.05.2025)"),
            "{messages:?}"
        );
        // A new flight is downloaded again.
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                crate::sources::geobasis_nrw::WMS.url
            ),
            b"Layer 'nw_dop_utm_info'\n    Bildflugdatum = '02.06.2027'\n".to_vec(),
        );
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 2);
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("02.06.2027")
        );
    }

    #[test]
    fn a_failed_flight_date_query_keeps_the_recent_orthophoto_and_its_date() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 1);
        // The date query now fails.
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                crate::sources::geobasis_nrw::WMS.url
            ),
            b"<ServiceExceptionReport/>".to_vec(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 1, "{messages:?}");
        assert_eq!(
            project.orthophoto.as_ref().unwrap().flight.as_deref(),
            Some("11.05.2025")
        );
        // Older than the limit: downloaded again, now without a date.
        let mut stored = ProjectFile::load(dir.path()).unwrap();
        stored.orthophoto.as_mut().unwrap().fetched = chrono::Local::now().date_naive()
            - chrono::Duration::days(UNDATED_ORTHOPHOTO_MAX_AGE_DAYS);
        stored.save(dir.path()).unwrap();
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(nrw_getmap_requests(&fetcher), 2);
        assert_eq!(project.orthophoto.unwrap().flight, None);
    }

    #[test]
    fn nrw_xml_wms_answer_is_an_error() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        let photo = crate::raster::GridSpec::covering(
            &ProjectFile::load(dir.path()).unwrap().bbox().unwrap(),
            crate::sources::wms::ORTHOPHOTO_PIXEL_M,
        );
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ServiceExceptionReport version="1.3.0"><ServiceException code="InvalidDimensionValue">
msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be between 1 and 5000 pixels.
</ServiceException></ServiceExceptionReport>"#;
        for (_, url) in crate::sources::wms::pieces(&crate::sources::geobasis_nrw::WMS, &photo) {
            fetcher.insert(&url, xml.to_vec());
        }
        let error = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
            .unwrap_err()
            .to_string();
        assert!(error.contains("Image size out of range"), "{error}");
        assert!(ProjectFile::load(dir.path()).unwrap().orthophoto.is_none());
    }

    #[test]
    fn offline_fetch_keeps_the_nrw_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let earlier = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        for record in [project.terrain.as_mut(), project.surface.as_mut()] {
            record.unwrap().fetched = earlier;
        }
        project.save(dir.path()).unwrap();
        let offline = MapFetcher::default();
        let project = fetch_project(
            dir.path(),
            &offline,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.surface.as_ref().unwrap().fetched, earlier);
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2022-02-12"));
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("11.05.2025")
        );
        assert_eq!(offline.calls.get(), 0);
        assert!(dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());
    }

    /// A schoolyard project of `area`, served with synthetic LGLN data: the rasters, the
    /// Niedersachsen ALKIS fixtures (and the Hamburg ones) and every OSM extract the area needs.
    fn ni_project_of(
        fetcher: &MapFetcher,
        area: &MapArea,
    ) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), area, MapKind::Schoolyard, None, date, None).unwrap();
        crate::fetch::test_support::serve_synthetic_lgln(fetcher, &project.bbox().unwrap());
        use crate::sources::alkis_wfs::{LGLN, test_support as fixtures};
        for (type_name, body) in [
            (LGLN.buildings_type, fixtures::NI_BUILDINGS),
            (LGLN.land_use_type, fixtures::NI_LAND_USE),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                    LGLN.url
                ),
                body.as_bytes().to_vec(),
            );
        }
        use crate::sources::lgv;
        fetcher.insert_prefix(
            lgv::BUILDINGS_ENDPOINT,
            lgv::test_support::BUILDINGS.as_bytes().to_vec(),
        );
        fetcher.insert_prefix(
            lgv::LAND_USE_ENDPOINT,
            lgv::test_support::LAND_USE.as_bytes().to_vec(),
        );
        for extract in
            crate::sources::osm::extracts_for(&project.region_bbox().unwrap(), &mut |_| {})
        {
            fetcher.insert(
                extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                extract.url(),
                std::fs::read(crate::fetch::test_support::OSM_FIXTURE).expect("OSM fixture"),
            );
        }
        (dir, cache_dir, cache)
    }

    /// The Kaiser-Wilhelm- und Ratsgymnasium, Hannover (25832 E 551900 / N 5803300), 200 m square.
    fn hannover_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        ni_project_of(fetcher, &utm32_area_at(551_900.0, 5_803_300.0, 100.0))
    }

    /// The warning of the synthetic LGLN data: DOM1 of 2016, orthophoto of 2025.
    const NI_DOM_AGE_WARNING: &str = "warning: the surface model (LGLN DOM1) is from 18.04.2016, the orthophoto from 06.03.2025. The nDSM may lack newer buildings and trees.";

    #[test]
    fn ni_project_fetches_products() {
        use crate::sources::lgln;
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::Lgln));
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("06.03.2025")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2016-04-18"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2016-04-18"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [Region::Niedersachsen]);
        {
            let log = fetcher.log.borrow();
            let getmap: Vec<&String> = log
                .iter()
                .filter(|u| u.starts_with(lgln::WMS.url) && u.contains("REQUEST=GetMap"))
                .collect();
            assert_eq!(getmap.len(), 1, "{log:?}");
            assert!(
                getmap[0].contains("LAYERS=ni_dop20&") && getmap[0].contains("CRS=EPSG:25832&"),
                "{}",
                getmap[0]
            );
            for (model, coverage) in [
                (lgln::Model::Dgm1, "ni_dgm1"),
                (lgln::Model::Dom1, "ni_dom1"),
            ] {
                assert!(
                    log.iter()
                        .any(|u| u.starts_with(model.wcs().url) && u.contains(coverage)),
                    "{coverage}: {log:?}"
                );
            }
            // One DOP search gives both the flight date and the leaf state.
            let dop = log.iter().filter(|u| u.starts_with(lgln::DOP_STAC)).count();
            assert_eq!(dop, 1, "{log:?}");
        }
        assert!(
            messages
                .iter()
                .any(|m| m == "the orthophoto was flown on 06.03.2025, with the trees bare"),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Lower Saxony from 2026-11-06"),
            "{messages:?}"
        );
        // The DOM1 of 2016 is the one warning.
        let warnings: Vec<&String> = messages
            .iter()
            .filter(|m| m.starts_with("warning"))
            .collect();
        assert_eq!(warnings, [NI_DOM_AGE_WARNING], "{messages:?}");
    }

    #[test]
    fn dom_older_than_dop_is_warned() {
        use crate::sources::lgln::{self, test_support::stac_page};
        // The synthetic DOM1 (18.04.2016) is nine years older than the orthophoto (06.03.2025).
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m == NI_DOM_AGE_WARNING),
            "{messages:?}"
        );
        // A DOM1 two years older or less, or newer, is close enough: no warning.
        for dom in [
            "2023-03-06T00:00:00Z",
            "2024-05-01T00:00:00Z",
            "2026-01-10T00:00:00Z",
        ] {
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = hannover_school(&fetcher);
            fetcher.insert_prefix(lgln::DOM_STAC, stac_page(&[("dom", dom)], None));
            let (project, messages) = fetch(dir.path(), &fetcher, &cache);
            assert_eq!(
                project.surface.unwrap().date.as_deref(),
                Some(&dom[..10]),
                "{dom}"
            );
            assert!(
                !messages.iter().any(|m| m.starts_with("warning")),
                "{dom}: {messages:?}"
            );
        }
        // Just over two years: warned.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetcher.insert_prefix(
            lgln::DOM_STAC,
            stac_page(&[("dom", "2023-03-05T00:00:00Z")], None),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m
                == "warning: the surface model (LGLN DOM1) is from 05.03.2023, the orthophoto from 06.03.2025. The nDSM may lack newer buildings and trees."),
            "{messages:?}"
        );
        // Without a DOM1 date there is nothing to compare.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetcher.insert_prefix(lgln::DOM_STAC, stac_page(&[], None));
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !messages.iter().any(|m| m.contains("is from")),
            "{messages:?}"
        );
    }

    #[test]
    fn ni_flight_date_lookup_failures_are_warned() {
        use crate::sources::lgln::{self, test_support::stac_page};
        let dop_searches = |fetcher: &MapFetcher| {
            fetcher
                .log
                .borrow()
                .iter()
                .filter(|u| u.starts_with(lgln::DOP_STAC))
                .count()
        };
        let warnings = |messages: &[String]| -> Vec<String> {
            messages
                .iter()
                .filter(|m| m.starts_with("warning"))
                .cloned()
                .collect()
        };
        // No orthophoto at the centre: no flight date, and nothing to compare the DOM1 with.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetcher.insert_prefix(lgln::DOP_STAC, stac_page(&[], None));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.orthophoto.unwrap().flight, None);
        assert_eq!(
            warnings(&messages),
            [
                "warning: orthophoto (LGLN WMS): no flight date, the STAC lists no image at the centre of the map area"
            ],
            "{messages:?}"
        );
        // A refused search (HTTP 400 is not retried) on a new project: no flight date either.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetcher
            .refusing
            .borrow_mut()
            .push(lgln::DOP_STAC.to_string());
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.orthophoto.unwrap().flight, None);
        assert_eq!(dop_searches(&fetcher), 1);
        let found = warnings(&messages);
        assert_eq!(found.len(), 1, "{messages:?}");
        assert!(
            found[0].starts_with(
                "warning: orthophoto (LGLN WMS): no flight date, the STAC search gave no usable answer ("
            ),
            "{messages:?}"
        );
        // A failed search later: the stored orthophoto stays with its flight date, and every
        // line says so.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        fetcher
            .refusing
            .borrow_mut()
            .push(lgln::DOP_STAC.to_string());
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("06.03.2025")
        );
        assert_eq!(dop_searches(&fetcher), 2, "one search per fetch");
        assert!(
            !messages.iter().any(|m| m.contains("no flight date")),
            "{messages:?}"
        );
        let kept: Vec<&String> = messages
            .iter()
            .filter(|m| {
                m.starts_with(
                    "warning: orthophoto (LGLN WMS): the STAC search gave no usable answer (",
                )
            })
            .collect();
        assert_eq!(kept.len(), 1, "{messages:?}");
        assert!(
            kept[0].ends_with("; kept the stored orthophoto and its flight date 06.03.2025"),
            "{}",
            kept[0]
        );
        for line in [
            "orthophoto unchanged (flight 06.03.2025)",
            "the orthophoto was flown on 06.03.2025, with the trees bare",
        ] {
            assert!(messages.iter().any(|m| m == line), "{line}: {messages:?}");
        }
        let getmap = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.starts_with(lgln::WMS.url) && u.contains("REQUEST=GetMap"))
            .count();
        assert_eq!(getmap, 1);
        // An empty answer later keeps the stored orthophoto the same way.
        fetcher.refusing.borrow_mut().clear();
        fetcher.insert_prefix(lgln::DOP_STAC, stac_page(&[], None));
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m
                == "warning: orthophoto (LGLN WMS): the STAC lists no image at the centre of the map area; kept the stored orthophoto and its flight date 06.03.2025"),
            "{messages:?}"
        );
    }

    #[test]
    fn ni_area_reaching_into_bremen_is_warned() {
        // The Bremen probe, tile 32 485 5876 (Stuhr-Brinkum / Bremen-Huchting, 44 % Bremen): the
        // first 600 m area south of the tile centre whose centre lies in Niedersachsen.
        let (e, n) = (0..40)
            .map(|step| (485_500.0, 5_876_500.0 - f64::from(step) * 50.0))
            .find(|&(e, n)| {
                raster_region(&utm32_area_at(e, n, 300.0).projected_bbox())
                    == Some(Region::Niedersachsen)
            })
            .expect("a centre in Niedersachsen");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::Lgln));
        assert_eq!(
            messages
                .iter()
                .filter(|m| m.as_str() == BEYOND_COVERAGE_WARNING)
                .count(),
            1,
            "{messages:?}"
        );
        // Bremen is not a region: the border message between two supported states stays out.
        assert!(
            !messages
                .iter()
                .any(|m| m.contains("reaches beyond the state border")),
            "{messages:?}"
        );
        // Fully inside Niedersachsen: no such warning.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !messages.iter().any(|m| m == BEYOND_COVERAGE_WARNING),
            "{messages:?}"
        );
    }

    #[test]
    fn ni_area_reaching_into_thuringia_names_unsupported_states() {
        // East of Duderstadt on 51.50 N the NI/TH border crosses near 25832 E 590304: a 600 m
        // area centred 150 m west of it lies in Niedersachsen and reaches into Thuringia.
        let area = utm32_area_at(590_150.0, 5_706_232.0, 300.0);
        assert_eq!(
            raster_region(&area.projected_bbox()),
            Some(Region::Niedersachsen)
        );
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &area);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::Lgln));
        let warnings: Vec<&String> = messages
            .iter()
            .filter(|m| {
                m.starts_with("warning: part of the map area lies outside the supported states")
            })
            .collect();
        assert_eq!(warnings, [BEYOND_COVERAGE_WARNING], "{messages:?}");
        // Thuringia is a German state freemapper does not support yet: the text must say so.
        assert!(
            BEYOND_COVERAGE_WARNING.contains("a state freemapper does not support yet"),
            "{BEYOND_COVERAGE_WARNING}"
        );
    }

    #[test]
    fn ni_leaf_on_flight_is_reported() {
        use crate::sources::lgln::{self, test_support::stac_page};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetcher.insert_prefix(
            lgln::DOP_STAC,
            stac_page(&[("dop", "2024-08-12T00:00:00Z")], None),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m
                == "the orthophoto was flown on 12.08.2024, with the trees in leaf; their crowns hide what lies below them"),
            "{messages:?}"
        );
    }

    #[test]
    fn ni_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        // DOM1 (terrain + 12 m) minus DGM1.
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn ni_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        let year = chrono::Local::now().year();
        let line = format!("© GeoBasis-DE/LGLN {year}, Daten geändert, CC BY 4.0");
        assert!(
            attribution.lines().any(|l| l.trim() == line),
            "{attribution}"
        );
        assert_eq!(
            attribution
                .matches("https://creativecommons.org/licenses/by/4.0/")
                .count(),
            1,
            "{attribution}"
        );
        for other in [
            "HVBG",
            "LVermGeo",
            "Geobasis NRW",
            "GeoBasis-DE/MV",
            "LGB",
            "Hamburg",
        ] {
            assert!(!attribution.contains(other), "{other} in {attribution}");
        }
        // The map's credits name LGLN too.
        let lines = attribution_lines(&project, &[]);
        assert!(lines.contains(&line), "{lines:?}");
        let map = std::fs::read_to_string(map_path(dir.path())).unwrap();
        assert!(map.contains(&line), "the map credits lack {line}");
    }

    #[test]
    fn ni_credit_and_licence_link_are_not_repeated_across_the_sh_border() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (mut project, _) = fetch(dir.path(), &fetcher, &cache);
        // Objects of both states were fetched (a map on the Elbe).
        project.buildings.as_mut().unwrap().regions =
            vec![Region::SchleswigHolstein, Region::Niedersachsen];
        let lines = attribution_lines(&project, &[]);
        assert!(lines.iter().any(|l| l == SH_ATTRIBUTION), "{lines:?}");
        assert!(
            lines.iter().any(|l| l.starts_with("© GeoBasis-DE/LGLN ")),
            "{lines:?}"
        );
        let links = lines
            .iter()
            .filter(|l| l.contains("creativecommons.org"))
            .count();
        assert_eq!(links, 1, "{lines:?}");
    }

    #[test]
    fn ni_project_at_the_hamburg_border_takes_rasters_from_the_centre() {
        use crate::sources::region::regions_for;
        // Walk south from Hamburg-Harburg (25832 E 553000 / N 5926000) to the first 600 m area
        // whose centre lies in Niedersachsen while it still reaches into Hamburg.
        let (e, n) = (0..60)
            .map(|step| (553_000.0, 5_926_000.0 - f64::from(step) * 100.0))
            .find(|&(e, n)| {
                let area = utm32_area_at(e, n, 300.0);
                let bbox = area.projected_bbox();
                raster_region(&bbox) == Some(Region::Niedersachsen)
                    && regions_for(&bbox).contains(&Region::Hamburg)
            })
            .expect("a border area south of Harburg");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::Lgln));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::Hamburg, Region::Niedersachsen]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                crate::sources::lgv::BUILDINGS_ENDPOINT,
                crate::sources::alkis_wfs::LGLN.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(service)), "{service}");
            }
            // The rasters come from Niedersachsen alone.
            for lgv_raster in ["geodienste.hamburg.de", "daten-hamburg.de"] {
                assert!(
                    !log.iter().any(|u| u.contains(lgv_raster)),
                    "{lgv_raster}: {log:?}"
                );
            }
        }
        // LGLN's orthophoto and height models also cover this Hamburg corner (live run), so the
        // line must not promise empty templates there.
        assert!(
            messages.iter().any(|m| m
                == "the map area reaches beyond the state border; beyond it the templates are empty wherever the LGLN orthophoto and height models end. ALKIS and OSM come from both states"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.contains("stay empty beyond it")),
            "{messages:?}"
        );
        // Both neighbours are supported states: nothing lies outside them.
        assert!(
            !messages.iter().any(|m| m == BEYOND_COVERAGE_WARNING),
            "{messages:?}"
        );
        for label in ["Hamburg", "Lower Saxony"] {
            let reading = messages
                .iter()
                .filter(|m| m.starts_with(&format!("reading OSM extract {label} ")))
                .count();
            assert_eq!(reading, 1, "{label}: {messages:?}");
        }
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    #[test]
    fn offline_fetch_keeps_the_ni_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let offline = MapFetcher::default();
        let project = fetch_project(
            dir.path(),
            &offline,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2016-04-18"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2016-04-18"));
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("06.03.2025")
        );
        assert_eq!(offline.calls.get(), 0);
    }

    /// A schoolyard project of `area`, served with synthetic LVermGeo RP data: the rasters, the
    /// RP ALKIS fixtures (and the Hesse ones) and every OSM extract the area needs.
    fn rp_project_of(
        fetcher: &MapFetcher,
        area: &MapArea,
    ) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), area, MapKind::Schoolyard, None, date, None).unwrap();
        crate::fetch::test_support::serve_synthetic_rp(fetcher, &project.bbox().unwrap());
        use crate::sources::alkis_wfs::{self, test_support as fixtures};
        for (service, buildings, land_use) in [
            (
                alkis_wfs::LVERMGEO_RP,
                fixtures::RP_BUILDINGS,
                fixtures::RP_LAND_USE,
            ),
            (
                alkis_wfs::HVBG,
                fixtures::HE_BUILDINGS,
                fixtures::HE_LAND_USE,
            ),
        ] {
            for (type_name, body) in [
                (service.buildings_type, buildings),
                (service.land_use_type, land_use),
            ] {
                fetcher.insert_prefix(
                    &format!(
                        "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                        service.url
                    ),
                    body.as_bytes().to_vec(),
                );
            }
        }
        for extract in
            crate::sources::osm::extracts_for(&project.region_bbox().unwrap(), &mut |_| {})
        {
            fetcher.insert(
                extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                extract.url(),
                std::fs::read(crate::fetch::test_support::OSM_FIXTURE).expect("OSM fixture"),
            );
        }
        (dir, cache_dir, cache)
    }

    /// The Görres-Gymnasium, Koblenz (25832 E 400360 / N 5579556, 25833 E -26154), 200 m square.
    fn koblenz_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        rp_project_of(fetcher, &utm32_area_at(400_360.0, 5_579_556.0, 100.0))
    }

    #[test]
    fn rp_project_fetches_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koblenz_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        // Koblenz lies west of the 25833 false easting: the region box is negative.
        assert!(project.region_bbox().unwrap().max_e < 0.0);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(Provider::LvermGeoRp));
        assert!(project.tiles.is_empty() && project.ndsm_tiles.is_empty());
        let orthophoto = project.orthophoto.unwrap();
        assert_eq!(orthophoto.flight.as_deref(), Some("25.08.2025"));
        // The date is the block's `erstellung`, and the record says so.
        assert_eq!(
            orthophoto.flight_label.as_deref(),
            Some("Aktualität (Erstellung)")
        );
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-04-06"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2024-04-06"));
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [Region::RheinlandPfalz]);
        {
            let log = fetcher.log.borrow();
            // The orthophoto and the tiles come from the origin hosts, never the registry proxy.
            assert!(
                log.iter().any(|u| u.starts_with(
                    "https://geo4.service24.rlp.de/wms/rp_dop20.fcgi?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap"
                )),
                "{log:?}"
            );
            assert!(
                log.iter()
                    .any(|u| u.starts_with("https://geobasis-rlp.de/data/dom1/current/tif/")),
                "{log:?}"
            );
            assert!(
                !log.iter().any(|u| u.contains("geoportal.rlp.de")),
                "{log:?}"
            );
        }
        // The record keeps the label in freemapper.toml.
        let toml = std::fs::read_to_string(dir.path().join("freemapper.toml")).unwrap();
        assert!(
            toml.contains("flight_label = \"Aktualität (Erstellung)\""),
            "{toml}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "reading OSM extract Rhineland-Palatinate from 2026-11-06"),
            "{messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.starts_with("warning")),
            "{messages:?}"
        );
    }

    fn rp_getmap_requests(fetcher: &MapFetcher) -> usize {
        fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("rp_dop20.fcgi") && u.contains("REQUEST=GetMap"))
            .count()
    }

    #[test]
    fn rp_orthophoto_is_reused_while_the_flight_is_unchanged() {
        use crate::sources::lvermgeo_rp::WMS;
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koblenz_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        assert_eq!(rp_getmap_requests(&fetcher), 1);
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(rp_getmap_requests(&fetcher), 1);
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (date of creation 25.08.2025)"),
            "{messages:?}"
        );
        // A new flight is downloaded again.
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            b"Layer 'metadaten'\n    erstellung = '12.06.2027'\n".to_vec(),
        );
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(rp_getmap_requests(&fetcher), 2);
        let orthophoto = project.orthophoto.unwrap();
        assert_eq!(orthophoto.flight.as_deref(), Some("12.06.2027"));
        assert_eq!(
            orthophoto.flight_label.as_deref(),
            Some("Aktualität (Erstellung)")
        );
        // The date query fails: the recent image is kept with its date and label.
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo",
                WMS.url
            ),
            b"<ServiceExceptionReport/>".to_vec(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(rp_getmap_requests(&fetcher), 2, "{messages:?}");
        let orthophoto = project.orthophoto.unwrap();
        assert_eq!(orthophoto.flight.as_deref(), Some("12.06.2027"));
        assert_eq!(
            orthophoto.flight_label.as_deref(),
            Some("Aktualität (Erstellung)")
        );
        assert!(
            messages
                .iter()
                .any(|m| m == "orthophoto unchanged (date of creation 12.06.2027)"),
            "{messages:?}"
        );
        // An old image is downloaded again without a date, and the label goes as well.
        let mut stored = ProjectFile::load(dir.path()).unwrap();
        stored.orthophoto.as_mut().unwrap().fetched = chrono::Local::now().date_naive()
            - chrono::Duration::days(UNDATED_ORTHOPHOTO_MAX_AGE_DAYS);
        stored.save(dir.path()).unwrap();
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(rp_getmap_requests(&fetcher), 3);
        let orthophoto = project.orthophoto.unwrap();
        assert_eq!((orthophoto.flight, orthophoto.flight_label), (None, None));
    }

    #[test]
    fn rp_derive_writes_all_templates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koblenz_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        for relpath in [
            ORTHOPHOTO_RELPATH,
            HILLSHADE_RELPATH,
            SLOPE_RELPATH,
            NDSM_RELPATH,
            CONTOURS_RELPATH,
        ] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        let png = image::open(dir.path().join(NDSM_RELPATH))
            .unwrap()
            .to_rgba8();
        let centre = png.get_pixel(png.width() / 2, png.height() / 2).0;
        assert_eq!(centre, crate::derive::ndsm::colour(12.0));
    }

    #[test]
    fn rp_attribution() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koblenz_school(&fetcher);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let attribution = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        let year = chrono::Local::now().year();
        // The open-data page's form: no blank before the year.
        let line = format!(
            "©GeoBasis-DE / LVermGeoRP{year}, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"
        );
        assert!(
            attribution.lines().any(|l| l.trim() == line),
            "{attribution}"
        );
        // The licence note, with the licence's proper name.
        let licence = "Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0";
        assert!(
            attribution.lines().any(|l| l.trim() == licence),
            "{attribution}"
        );
        for other in [
            "HVBG",
            "LVermGeo ST",
            "LVermGeo SH",
            "Geobasis NRW",
            "LGLN",
            "CC BY",
            "Hamburg",
        ] {
            assert!(!attribution.contains(other), "{other} in {attribution}");
        }
        // The map's credits name LVermGeo RP too.
        let lines = attribution_lines(&project, &[]);
        assert!(lines.contains(&line), "{lines:?}");
        let map = std::fs::read_to_string(map_path(dir.path())).unwrap();
        assert!(map.contains(&line), "the map credits lack {line}");
    }

    #[test]
    fn rp_project_at_the_hesse_border_takes_rasters_from_the_centre() {
        use crate::sources::region::regions_for;
        // The Hesse border probe (tile 32 447 5540, 43 % RP, Mainz-Neustadt / Rhine /
        // Wiesbaden-Amöneburg): walk west from the tile centre to the first 600 m area whose
        // centre lies in Rhineland-Palatinate while it still reaches into Hesse.
        let (e, n) = (0..40)
            .map(|step| (447_500.0 - f64::from(step) * 50.0, 5_540_500.0))
            .find(|&(e, n)| {
                let bbox = utm32_area_at(e, n, 300.0).projected_bbox();
                raster_region(&bbox) == Some(Region::RheinlandPfalz)
                    && regions_for(&bbox).contains(&Region::Hessen)
            })
            .expect("a border area at Mainz");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rp_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::LvermGeoRp));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert!(project.surface.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [Region::Hessen, Region::RheinlandPfalz]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                crate::sources::alkis_wfs::HVBG.url,
                crate::sources::alkis_wfs::LVERMGEO_RP.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(service)), "{service}");
            }
            // The rasters come from Rhineland-Palatinate alone. Neither the orthophoto WMS nor
            // the Downloadcenter of Hesse is asked (its ALKIS WFS, on www.gds.hessen.de, is).
            for host in ["https://www.gds-srv.hessen.de", "https://gds.hessen.de"] {
                assert!(!log.iter().any(|u| u.starts_with(host)), "{host}");
            }
        }
        // The RP WMS is white across the Rhine while the DGM1/DOM1 tiles reach over it: the line
        // promises empty templates only where each product ends.
        assert!(
            messages.iter().any(|m| m
                == "the map area reaches beyond the state border; beyond it the templates are empty wherever the LVermGeo RP orthophoto and height models end. ALKIS and OSM come from both states"),
            "{messages:?}"
        );
        // Both sides are supported states: nothing lies outside them.
        assert!(
            !messages.iter().any(|m| m == BEYOND_COVERAGE_WARNING),
            "{messages:?}"
        );
        for label in ["Hesse", "Rhineland-Palatinate"] {
            let reading = messages
                .iter()
                .filter(|m| m.starts_with(&format!("reading OSM extract {label} ")))
                .count();
            assert_eq!(reading, 1, "{label}: {messages:?}");
        }
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
    }

    #[test]
    fn offline_fetch_keeps_the_rp_record_dates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koblenz_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        let offline = MapFetcher::default();
        let project = fetch_project(
            dir.path(),
            &offline,
            &cache,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(project.terrain.unwrap().date.as_deref(), Some("2024-04-06"));
        assert_eq!(project.surface.unwrap().date.as_deref(), Some("2024-04-06"));
        let orthophoto = project.orthophoto.unwrap();
        assert_eq!(orthophoto.flight.as_deref(), Some("25.08.2025"));
        assert_eq!(
            orthophoto.flight_label.as_deref(),
            Some("Aktualität (Erstellung)")
        );
        assert_eq!(offline.calls.get(), 0);
    }

    #[test]
    fn regbez_border_reads_two_extracts() {
        use crate::sources::osm::{ExtractSource, GovernmentDistrict};
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Koeln-Worringen on the Koeln / Duesseldorf (Dormagen) district border, 25833
        // E -70550 / N 5689830: the centre lies in the Koeln extract only, the 1 km box reaches
        // into Duesseldorf's.
        let (ce, cn) = (-70_550.0, 5_689_830.0);
        let at = |e: f64, n: f64| {
            crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: ce + e,
                northing: cn + n,
            })
            .unwrap()
        };
        let corner = |e: f64, n: f64| {
            let p = at(e, n);
            format!("[{},{}]", p.lon, p.lat)
        };
        let ring = [
            corner(-480.0, -480.0),
            corner(480.0, -480.0),
            corner(480.0, 480.0),
            corner(-480.0, 480.0),
            corner(-480.0, -480.0),
        ];
        let area = MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 8).unwrap();
        let project =
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
        assert_eq!(project.crs(), Crs::Utm32);
        let koeln = ExtractSource::NordrheinWestfalen(GovernmentDistrict::Koeln);
        let duesseldorf = ExtractSource::NordrheinWestfalen(GovernmentDistrict::Duesseldorf);
        let centre = BoundingBox {
            min_e: ce,
            min_n: cn,
            max_e: ce,
            max_n: cn,
        };
        assert_eq!(
            crate::sources::osm::extracts_for(&centre, &mut |_| {}),
            [koeln]
        );
        assert_eq!(
            crate::sources::osm::extracts_for(&project.region_bbox().unwrap(), &mut |_| {}),
            [duesseldorf, koeln]
        );
        // Geofabrik's district extracts overlap at the border: both hold the same two hydrants.
        let (south, north) = (at(0.0, -300.0), at(0.0, 300.0));
        let hydrant: &[(&str, &str)] = &[("emergency", "fire_hydrant")];
        let pbf = crate::fetch::test_support::osm_pbf(&[
            (201, south.lat, south.lon, hydrant),
            (202, north.lat, north.lon, hydrant),
        ]);
        for extract in [koeln, duesseldorf] {
            fetcher.insert(
                extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(extract.url(), pbf.clone());
        }
        crate::fetch::test_support::serve_synthetic_nrw(&fetcher, &project.bbox().unwrap());
        // No ALKIS objects in this box.
        fetcher.insert_prefix(
            crate::sources::alkis_wfs::GEOBASIS_NRW.url,
            br#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="0" numberMatched="0" xmlns:wfs="http://www.opengis.net/wfs/2.0"/>"#.to_vec(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(Provider::GeobasisNrw));
        for extract in ["Düsseldorf", "Köln"] {
            let line =
                format!("reading OSM extract North Rhine-Westphalia, {extract} from 2026-11-06");
            assert_eq!(
                messages.iter().filter(|m| **m == line).count(),
                1,
                "{messages:?}"
            );
        }
        let mut cached: Vec<String> = std::fs::read_dir(cache.osm_dir())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        cached.sort();
        assert_eq!(
            cached,
            [
                "nrw-duesseldorf_2026-11-06.osm.pbf",
                "nrw-koeln_2026-11-06.osm.pbf"
            ]
        );
        // Each OSM candidate once, although both extracts hold it.
        let text = std::fs::read_to_string(dir.path().join(OSM_CANDIDATES_RELPATH)).unwrap();
        let ids: Vec<String> = crate::sources::osm::features_from_geojson(&text)
            .unwrap()
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["node/201", "node/202"]);
        assert_eq!(project.osm.unwrap().count, 2);
    }

    #[test]
    fn merged_extracts_keep_each_feature_once() {
        let feature = |id: &str| OsmFeature {
            id: id.to_string(),
            tags: vec![("barrier".to_string(), "fence".to_string())],
            geometry: crate::sources::osm::OsmGeometry::Point(crate::crs::LatLon {
                lat: 51.5,
                lon: 14.0,
            }),
        };
        let brandenburg = vec![feature("node/5"), feature("way/7"), feature("way/30")];
        let saxony = vec![feature("node/2"), feature("node/5"), feature("way/7")];
        let ids: Vec<String> = merge_features(vec![brandenburg, saxony])
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["node/2", "node/5", "way/7", "way/30"]);
    }

    /// Serves the extra Berlin layer fixtures for exactly `bbox` (an empty answer for the types
    /// without a fixture).
    fn serve_berlin_layers(fetcher: &MapFetcher, bbox: &BoundingBox) {
        use crate::sources::berlin::{EXTRA_LAYERS, test_support::answer, wfs_url};
        for layer in &EXTRA_LAYERS {
            for t in layer.types {
                fetcher.insert(
                    &wfs_url(layer.service, t.name, bbox),
                    answer(t.name).as_bytes().to_vec(),
                );
            }
        }
    }

    /// The stored buildings of `berlin_school` plus one passage part inside the map area.
    fn add_passage_part(dir: &Path) {
        let path = dir.join(BUILDINGS_RELPATH);
        let mut collection: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let ring = [
            [392_790.0, 5_820_840.0],
            [392_800.0, 5_820_840.0],
            [392_800.0, 5_820_850.0],
            [392_790.0, 5_820_850.0],
            [392_790.0, 5_820_840.0],
        ];
        collection["features"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "type": "Feature", "id": "DEBE00PASSAGE",
                "geometry": {"type": "MultiPolygon", "coordinates": [[ring]]},
                "properties": {"oid": "DEBE00PASSAGE", "gebnutzbez": "Bauteil",
                               "bauart": "Durchfahrt im Gebäude"}
            }));
        std::fs::write(path, collection.to_string()).unwrap();
    }

    /// The Berlin candidates (symbol code, source tag) of the starter documents.
    fn berlin_candidates(dir: &Path) -> Vec<(String, String)> {
        let documents = starter_documents(dir).unwrap();
        let set = MapKind::Schoolyard.symbol_set();
        let code = |id| {
            ["515", "522"]
                .into_iter()
                .find(|c| set.id_for_code(c) == Some(id))
        };
        documents
            .candidates
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .filter_map(|o| {
                let source = o.tags.iter().find(|(k, _)| k == "freemapper:source")?;
                source.1.starts_with("geoportal-berlin/").then(|| {
                    (
                        code(o.symbol_id).unwrap_or("other").to_string(),
                        source.1.clone(),
                    )
                })
            })
            .collect()
    }

    #[test]
    fn stored_berlin_layers_and_passages_reach_the_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        // One wall line inside the map area replaces the layer's contents.
        let wall = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "bauwerkelinien.1",
            "geometry": {"type": "MultiLineString",
                "coordinates": [[[392_770.0, 5_820_830.0], [392_830.0, 5_820_830.0]]]},
            "properties": {"bezbwf": "Mauer", "uuid": "DEBE00WALL"}}]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-linien.geojson"),
            wall.to_string(),
        )
        .unwrap();
        add_passage_part(dir.path());
        let found = berlin_candidates(dir.path());
        assert!(
            found.contains(&("515".into(), "geoportal-berlin/bauwerke-linien".into())),
            "{found:?}"
        );
        assert!(
            found.contains(&("522".into(), "geoportal-berlin/building-parts".into())),
            "{found:?}"
        );
    }

    #[test]
    fn survey_railing_along_an_alkis_fence_is_named_by_it() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        let line = |id: &str, n: f64, props: serde_json::Value| {
            serde_json::json!({
                "type": "Feature", "id": id,
                "geometry": {"type": "MultiLineString",
                    "coordinates": [[[392_770.0, n], [392_830.0, n]]]},
                "properties": props})
        };
        let write = |file: &str, features: Vec<serde_json::Value>| {
            std::fs::write(
                dir.path().join(format!("data/berlin/{file}.geojson")),
                serde_json::json!({"type": "FeatureCollection", "features": features}).to_string(),
            )
            .unwrap();
        };
        write(
            "bauwerke-linien",
            vec![line(
                "bauwerkelinien.1",
                5_820_830.0,
                serde_json::json!({"bezbwf": "Zaun", "uuid": "DEBE00FENCE"}),
            )],
        );
        // One railing 0.7 m beside the fence (the same object), one 8 m away.
        let railing = |id: &str, n: f64| {
            line(
                &format!("bf_gelaender.{id}"),
                n,
                serde_json::json!({"gis_id": format!("Li_2015-05-02_{id}"),
                    "freemapper:layer": "bf_gelaender", "survey_date": "2015-05-02"}),
            )
        };
        write(
            "sb-gelaender",
            vec![railing("1", 5_820_830.7), railing("2", 5_820_838.0)],
        );
        let documents = starter_documents(dir.path()).unwrap();
        let set = MapKind::Schoolyard.symbol_set();
        let fences: Vec<&MapObject> = documents.candidates.parts[0]
            .objects
            .iter()
            .filter(|o| Some(o.symbol_id) == set.id_for_code("518"))
            .collect();
        let tag = |o: &MapObject, key: &str| {
            o.tags
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        let ids: Vec<_> = fences
            .iter()
            .filter_map(|o| tag(o, "freemapper:source_id"))
            .collect();
        assert_eq!(ids, ["DEBE00FENCE", "Li_2015-05-02_2"], "{fences:?}");
        assert_eq!(
            tag(fences[0], "freemapper:also").as_deref(),
            Some("geoportal-berlin/sb-gelaender/Li_2015-05-02_1")
        );
        assert_eq!(
            tag(fences[1], "freemapper:currency").as_deref(),
            Some("2015-05-02")
        );
    }

    /// Serves a K5 map for the template area of a Berlin project (the pieces `fetch_k5` asks for).
    fn serve_k5(fetcher: &MapFetcher, project: &ProjectFile) {
        let grid = GridSpec::covering(&project.bbox().unwrap(), crate::sources::wms::K5_PIXEL_M);
        for (piece, url) in crate::sources::wms::k5_pieces(&grid) {
            let mut png = Vec::new();
            image::RgbImage::from_pixel(piece.width, piece.height, image::Rgb([230, 220, 200]))
                .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
                .unwrap();
            fetcher.insert(&url, png);
        }
    }

    #[test]
    fn berlin_plus_attribution_lines() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        serve_k5(&fetcher, &project);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            !messages
                .iter()
                .any(|m| m.contains("K5") && m.contains("warning")),
            "{messages:?}"
        );
        // The K5 template and its world file; the project record keeps the dataset date.
        assert!(dir.path().join(K5_RELPATH).exists());
        assert!(dir.path().join("templates/k5.pgw").exists());
        assert_eq!(
            project.k5.as_ref().unwrap().flight.as_deref(),
            Some("2026-01-02")
        );
        let lines = attribution_lines(&project, &[]);
        for line in [
            BERLIN_ATTRIBUTION,
            "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0",
            "Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0",
        ] {
            assert!(lines.contains(&line.to_string()), "{line}: {lines:?}");
        }
        // The starter map lists the K5 template, hidden behind the map.
        let documents = starter_documents(dir.path()).unwrap();
        let k5 = documents
            .map
            .templates
            .iter()
            .find(|t| t.relpath == K5_RELPATH)
            .expect("K5 template in the map");
        assert!(k5.georeferenced && k5.behind_map && !k5.visible);
        assert!(documents.map.notes.contains("Karte 1:5000 von Berlin"));

        // Without the K5 map (the service fails) and without trees there are neither lines.
        let mut bare = project.clone();
        bare.k5 = None;
        for file in ["strassenbaeume", "anlagenbaeume"] {
            bare.berlin_layers.get_mut(file).unwrap().count = 0;
        }
        let lines = attribution_lines(&bare, &[]);
        assert!(lines.contains(&BERLIN_ATTRIBUTION.to_string()));
        assert!(
            !lines
                .iter()
                .any(|l| l.contains("K5") || l.contains("Baumbestand")),
            "{lines:?}"
        );
    }

    #[test]
    fn berlin_plus_2_attribution_lines() {
        const SURVEY: &str =
            "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert";
        const REGISTERS: &str = "Geoportal Berlin / Öffentliche Beleuchtung, Zierbrunnen, Trinkwasserbrunnen, dl-de/zero-2-0";
        const HEIGHTS: &str = "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0";
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        let all = ["beleuchtung", "zierbrunnen", "trinkwasserbrunnen"];
        let lines = attribution_lines(&project, &all);
        for line in [SURVEY, REGISTERS, HEIGHTS] {
            assert!(lines.contains(&line.to_string()), "{line}: {lines:?}");
        }
        // A layer without features contributes nothing: no line for it.
        let without = |files: &[&str]| {
            let mut bare = project.clone();
            for file in files {
                bare.berlin_layers.get_mut(*file).unwrap().count = 0;
            }
            attribution_lines(&bare, &all)
        };
        // A register is named only when it drew objects, each on its own.
        let lines = attribution_lines(&project, &[]);
        assert!(
            !lines.iter().any(|l| l.contains("Zierbrunnen")),
            "{lines:?}"
        );
        assert!(lines.contains(&SURVEY.to_string()) && lines.contains(&HEIGHTS.to_string()));
        let lines = attribution_lines(&project, &["trinkwasserbrunnen", "beleuchtung"]);
        assert!(
            lines.contains(
                &"Geoportal Berlin / Öffentliche Beleuchtung, Trinkwasserbrunnen, dl-de/zero-2-0"
                    .to_string()
            ),
            "{lines:?}"
        );
        let lines = without(&["gebaeudehoehen"]);
        assert!(!lines.contains(&HEIGHTS.to_string()), "{lines:?}");
        let survey_files: Vec<&str> = project
            .berlin_layers
            .keys()
            .map(String::as_str)
            .filter(|f| f.starts_with("sb-"))
            .collect();
        let lines = without(&survey_files);
        assert!(!lines.contains(&SURVEY.to_string()), "{lines:?}");
    }

    #[test]
    fn register_attribution_names_only_the_registers_that_drew_objects() {
        const FOUNTAINS: &str = "Geoportal Berlin / Zierbrunnen, dl-de/zero-2-0";
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        fetch(dir.path(), &fetcher, &cache);
        // Stored register files: lamps and drinking fountains without a point, one ornamental
        // fountain in the map area.
        let store = |file: &str, points: &[(&str, (f64, f64))]| {
            let features: Vec<serde_json::Value> = points
                .iter()
                .map(|(layer, (e, n))| {
                    serde_json::json!({
                        "type": "Feature", "id": format!("{layer}.1"),
                        "geometry": {"type": "Point", "coordinates": [e, n]},
                        "properties": {"id": "1", crate::sources::berlin::TYPE_KEY: layer}
                    })
                })
                .collect();
            let layer = serde_json::json!({"type": "FeatureCollection", "features": features});
            std::fs::write(layer_path(dir.path(), file), layer.to_string()).unwrap();
        };
        store("beleuchtung", &[]);
        store("trinkwasserbrunnen", &[]);
        store(
            "zierbrunnen",
            &[("bez_zierbrunnen", (392_800.0, 5_820_850.0))],
        );
        let documents = starter_documents(dir.path()).unwrap();
        for text in [&documents.attribution_text, &documents.map.notes] {
            assert!(text.contains(FOUNTAINS), "{text}");
            assert!(
                !text.contains("Beleuchtung") && !text.contains("Trinkwasserbrunnen"),
                "{text}"
            );
        }
        let sheet: Vec<&str> = documents
            .map
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .filter_map(|o| match &o.kind {
                MapObjectKind::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(sheet.iter().any(|t| t.contains(FOUNTAINS)), "{sheet:?}");
        // A register point far outside the map area draws nothing: no register line.
        store(
            "zierbrunnen",
            &[("bez_zierbrunnen", (393_400.0, 5_821_500.0))],
        );
        let documents = starter_documents(dir.path()).unwrap();
        assert!(
            !documents.attribution_text.contains("Zierbrunnen"),
            "{}",
            documents.attribution_text
        );
    }

    #[test]
    fn stored_berlin_buildings_take_the_footprint_date_of_the_height_layer() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        fetch(dir.path(), &fetcher, &cache);
        let dates = |dir: &Path| -> Vec<Option<NaiveDate>> {
            let stored = read_stored_buildings(dir).unwrap().unwrap();
            let gym = stored
                .buildings
                .iter()
                .find(|b| b.oid == "DEBE03YY600004AR");
            assert!(gym.is_some());
            stored.buildings.iter().map(|b| b.updated).collect()
        };
        // The height fixture is for another place: no building matches.
        assert!(dates(dir.path()).iter().all(Option::is_none));
        // A height feature of the school gymnasium gives its footprint date; the others stay.
        let path = layer_path(dir.path(), "gebaeudehoehen");
        let heights = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "gebaeudehoehen.1", "geometry": null,
            "properties": {"gml_id": "DEBE03YY600004AR", "grundrissaktualitaet": "2015-08-27"}
        }]});
        std::fs::write(&path, heights.to_string()).unwrap();
        let stored = read_stored_buildings(dir.path()).unwrap().unwrap();
        let dated: Vec<_> = stored
            .buildings
            .iter()
            .filter(|b| b.updated.is_some())
            .map(|b| (b.oid.as_str(), b.updated))
            .collect();
        assert_eq!(dated, [("DEBE03YY600004AR", "2015-08-27".parse().ok())]);
        // Without the layer file the buildings are undated again.
        std::fs::remove_file(&path).unwrap();
        assert!(dates(dir.path()).iter().all(Option::is_none));
    }

    #[test]
    fn failing_k5_is_a_warning_and_tree_candidates_are_deduplicated() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        // The K5 service fails: the fetch goes on, the template is left out.
        let grid = GridSpec::covering(&project.bbox().unwrap(), crate::sources::wms::K5_PIXEL_M);
        for (_, url) in crate::sources::wms::k5_pieces(&grid) {
            fetcher.failing.borrow_mut().push(url);
        }
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.k5.is_none());
        assert!(!dir.path().join(K5_RELPATH).exists());
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: the K5 map template is unavailable")),
            "{messages:?}"
        );
        assert!(
            !attribution_lines(&project, &[])
                .iter()
                .any(|l| l.contains("K5"))
        );

        // One cadastre tree with a 12 m crown inside the map area; OSM has a tree 1 m from it
        // (the same tree) and one 6 m away.
        let tree = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "strassenbaeume.1",
            "geometry": {"type": "Point", "coordinates": [392_800.0, 5_820_850.0]},
            "properties": {"gisid": "TREE1", "art_bot": "Tilia cordata", "kronedurch": 12}}]});
        std::fs::write(
            dir.path().join("data/berlin/strassenbaeume.geojson"),
            tree.to_string(),
        )
        .unwrap();
        let osm_tree = |id: &str, de: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 392_800.0 + de,
                northing: 5_820_850.0,
            })
            .unwrap();
            OsmFeature {
                id: id.to_string(),
                tags: vec![("natural".to_string(), "tree".to_string())],
                geometry: crate::sources::osm::OsmGeometry::Point(p),
            }
        };
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&[
                osm_tree("node/1", 1.0),
                osm_tree("node/2", 6.0),
            ]),
        )
        .unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        let set = MapKind::Schoolyard.symbol_set();
        let objects: Vec<&MapObject> = documents
            .candidates
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .collect();
        let tag = |o: &MapObject, key: &str| {
            o.tags
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        let osm_ids: Vec<_> = objects
            .iter()
            .filter(|o| tag(o, "freemapper:source").as_deref() == Some("osm"))
            .filter_map(|o| tag(o, "freemapper:source_id"))
            .collect();
        assert_eq!(osm_ids, ["node/2"]);
        let large = objects
            .iter()
            .find(|o| {
                tag(o, "freemapper:source").as_deref() == Some("geoportal-berlin/strassenbaeume")
            })
            .expect("the cadastre tree");
        assert_eq!(Some(large.symbol_id), set.id_for_code("417"));
        assert_eq!(
            tag(large, "freemapper:baum").as_deref(),
            Some("Tilia cordata, Krone 12 m")
        );
    }

    #[test]
    fn berlin_canopies_are_map_objects_and_stairs_get_the_derived_terrain() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        let area = |id: &str, kind: &str, (e, n): (f64, f64), (w, h): (f64, f64)| {
            serde_json::json!({
                "type": "Feature", "id": format!("bauwerkeflaechen.{id}"),
                "geometry": {"type": "MultiPolygon", "coordinates": [[[
                    [e, n], [e + w, n], [e + w, n + h], [e, n + h], [e, n]]]]},
                "properties": {"uuid": id, "bezbwf": kind}})
        };
        let layer = serde_json::json!({"type": "FeatureCollection", "features": [
            area("DEBE00ROOF", "Überdachung", (392_760.0, 5_820_860.0), (6.0, 4.0)),
            area("DEBE00STEP", "Treppe", (392_760.0, 5_820_835.0), (6.0, 3.0)),
        ]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-flaechen.geojson"),
            layer.to_string(),
        )
        .unwrap();
        let source = |o: &MapObject| {
            o.tags
                .iter()
                .find(|(k, _)| k == "freemapper:source")
                .map(|(_, v)| v.clone())
        };
        let set = MapKind::Schoolyard.symbol_set();
        let stairs = |documents: &StarterDocuments| -> Vec<MapObject> {
            documents.candidates.parts[0]
                .objects
                .iter()
                // 3 m wide at 1:1000 × 1.25: the widest footprint.
                .filter(|o| Some(o.symbol_id) == set.id_for_code("532.9"))
                .cloned()
                .collect()
        };
        let documents = starter_documents(dir.path()).unwrap();
        let canopies: Vec<&MapObject> = documents.map.parts[0]
            .objects
            .iter()
            .filter(|o| Some(o.symbol_id) == set.id_for_code("522"))
            .collect();
        assert_eq!(canopies.len(), 1);
        assert_eq!(
            source(canopies[0]).as_deref(),
            Some("geoportal-berlin/bauwerke-flaechen")
        );
        // Before `derive` there is no terrain: the short axis, guessed. Then the stair is 6 m
        // wide, more than 3 × 1.25 m: four parallel lines.
        let guessed = ("freemapper:direction".to_string(), "guessed".to_string());
        let wide = ("freemapper:size".to_string(), "wide".to_string());
        let before = stairs(&documents);
        assert_eq!(before.len(), 4);
        assert!(
            before
                .iter()
                .all(|o| o.tags.contains(&guessed) && o.tags.contains(&wide))
        );

        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(dir.path().join(TERRAIN_MOSAIC_RELPATH).exists());
        // The synthetic DGM rises 0.1 m per metre to the east: 0.6 m along the stair.
        let after = stairs(&starter_documents(dir.path()).unwrap());
        assert_eq!(after.len(), 1);
        assert!(!after[0].tags.contains(&guessed), "{:?}", after[0].tags);
        assert!(!after[0].tags.contains(&wide));
        assert!(
            after[0].coords[1].x > after[0].coords[0].x,
            "uphill to the east"
        );

        // Without stair areas (the surveyed ones included) `derive` keeps no terrain.
        std::fs::write(
            dir.path().join("data/berlin/sb-treppe.geojson"),
            r#"{"type":"FeatureCollection","features":[]}"#,
        )
        .unwrap();
        let roof_only = serde_json::json!({"type": "FeatureCollection", "features": [
            area("DEBE00ROOF", "Überdachung", (392_760.0, 5_820_860.0), (6.0, 4.0))]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-flaechen.geojson"),
            roof_only.to_string(),
        )
        .unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(!dir.path().join(TERRAIN_MOSAIC_RELPATH).exists());
    }

    #[test]
    fn terrain_written_for_nas_stairs_without_edge() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        fetch(dir.path(), &fetcher, &cache);
        // The Berlin layers have no stairs; the stored NAS layer has one stair area.
        let nas = |with_edge: bool| {
            let (e, n) = (392_760.0, 5_820_835.0);
            let mut features = vec![serde_json::json!({
                "type": "Feature", "id": "DEMVHR030000STEP",
                "geometry": {"type": "Polygon", "coordinates": [[
                    [e, n], [e + 6.0, n], [e + 6.0, n + 3.0], [e, n + 3.0], [e, n]]]},
                "properties": {"oid": "DEMVHR030000STEP", "nas_type":
                    "AX_SonstigesBauwerkOderSonstigeEinrichtung", "bauwerksfunktion": "1620"}
            })];
            if with_edge {
                features.push(serde_json::json!({
                    "type": "Feature", "id": "DEMVHR030000EDGE",
                    "geometry": {"type": "LineString", "coordinates": [[e, n], [e + 6.0, n]]},
                    "properties": {"oid": "DEMVHR030000EDGE", "nas_type":
                        "AX_SonstigesBauwerkOderSonstigeEinrichtung", "bauwerksfunktion": "1630",
                        "gehoert_zu": "DEMVHR030000STEP"}
                }));
            }
            serde_json::json!({"type": "FeatureCollection", "features": features}).to_string()
        };
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.berlin_layers.clear();
        project.nas_layers.insert(
            "sonstiges-bauwerk".to_string(),
            AlkisRecord {
                bbox: [0.0, 0.0, 1e9, 1e9],
                fetched: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
                latest_update: None,
                count: 1,
                regions: Vec::new(),
            },
        );
        project.save(dir.path()).unwrap();
        let layer = dir.path().join("data/alkis-nas/sonstiges-bauwerk.geojson");
        std::fs::create_dir_all(layer.parent().unwrap()).unwrap();
        std::fs::write(&layer, nas(false)).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(dir.path().join(TERRAIN_MOSAIC_RELPATH).exists());
        // With its bottom edge the direction is known: `derive` keeps no terrain.
        std::fs::write(&layer, nas(true)).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(!dir.path().join(TERRAIN_MOSAIC_RELPATH).exists());
    }

    #[test]
    fn step_edges_of_the_stored_lines_become_one_stair() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        // Three step edges of one flight (one of them twice) and nothing else in the layer.
        let edge = |id: &str, n: f64| {
            serde_json::json!({
                "type": "Feature", "id": format!("bauwerkelinien.{id}"),
                "geometry": {"type": "MultiLineString", "coordinates": [[
                    [392_760.0, n], [392_763.0, n]]]},
                "properties": {"uuid": id, "bezbwf": "Freitreppe"}})
        };
        let n = 5_820_835.0;
        let layer = serde_json::json!({"type": "FeatureCollection", "features": [
            edge("E0", n), edge("E1", n + 0.3), edge("E1bis", n + 0.3), edge("E2", n + 0.6)]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-linien.geojson"),
            layer.to_string(),
        )
        .unwrap();
        let set = MapKind::Schoolyard.symbol_set();
        let stairs: Vec<MapObject> = starter_documents(dir.path()).unwrap().candidates.parts[0]
            .objects
            .iter()
            .filter(|o| {
                o.tags.contains(&(
                    "freemapper:source".to_string(),
                    "geoportal-berlin/bauwerke-linien".to_string(),
                ))
            })
            .cloned()
            .collect();
        assert_eq!(stairs.len(), 1, "{stairs:?}");
        // 3 m wide at 1:1000 × 1.25: the widest footprint, not yet wide.
        assert_eq!(Some(stairs[0].symbol_id), set.id_for_code("532.9"));
        // Across the edges: about north-south (the map is turned to magnetic north), 0.6 m on
        // the ground = 0.6 mm on paper.
        let [a, b] = [stairs[0].coords[0], stairs[0].coords[1]];
        let (dx, dy) = (f64::from(b.x - a.x), f64::from(b.y - a.y));
        assert!(dx.abs() < 0.2 * dy.abs(), "{a:?} → {b:?}");
        assert!((dx.hypot(dy) - 600.0).abs() < 10.0, "{a:?} → {b:?}");
    }

    #[test]
    fn osm_fence_along_an_alkis_fence_is_dropped_in_the_export() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        let (e, n) = (392_760.0, 5_820_835.0);
        let layer = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "bauwerkelinien.Z",
            "geometry": {"type": "LineString", "coordinates": [[e, n], [e + 30.0, n]]},
            "properties": {"uuid": "DEBE00ZAUN", "bezbwf": "Zaun"}}]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-linien.geojson"),
            layer.to_string(),
        )
        .unwrap();
        // OSM has the same fence 1 m off, and another one 10 m away.
        let fence = |id: &str, dn: f64| {
            let p = |de: f64| {
                crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                    easting: e + de,
                    northing: n + dn,
                })
                .unwrap()
            };
            OsmFeature {
                id: id.to_string(),
                tags: vec![("barrier".to_string(), "fence".to_string())],
                geometry: crate::sources::osm::OsmGeometry::Line(vec![p(0.0), p(30.0)]),
            }
        };
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&[fence("way/1", 1.0), fence("way/2", 10.0)]),
        )
        .unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        let tag = |o: &MapObject, key: &str| {
            o.tags
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        let objects: Vec<&MapObject> = documents
            .candidates
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .collect();
        let osm_ids: Vec<_> = objects
            .iter()
            .filter(|o| tag(o, "freemapper:source").as_deref() == Some("osm"))
            .filter_map(|o| tag(o, "freemapper:source_id"))
            .collect();
        assert_eq!(osm_ids, ["way/2"]);
        let alkis = objects
            .iter()
            .find(|o| tag(o, "freemapper:source_id").as_deref() == Some("DEBE00ZAUN"))
            .expect("the ALKIS fence");
        assert_eq!(tag(alkis, "freemapper:also").as_deref(), Some("osm/way/1"));
    }

    #[test]
    fn nas_fence_beats_the_osm_fence_in_the_export() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _cache) = fetched_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        assert!(project.nas_layers.contains_key("sonstiges-bauwerk"));
        let bbox = project.bbox().unwrap();
        let (e, n) = (
            (bbox.min_e + bbox.max_e) / 2.0,
            (bbox.min_n + bbox.max_n) / 2.0,
        );
        // A stored NAS fence (map CRS) in the middle of the map, cut into two segments.
        let segment = |id: &str, from: f64, to: f64| {
            serde_json::json!({"type": "Feature", "id": id,
                "geometry": {"type": "LineString", "coordinates": [[e + from, n], [e + to, n]]},
                "properties": {"oid": id, "nas_type": "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                    "beginnt": "2019-05-06", "bauwerksfunktion": "1740"}})
        };
        let layer = serde_json::json!({"type": "FeatureCollection", "features": [
            segment("DEBBAL00ZAUN1", 0.0, 2.0), segment("DEBBAL00ZAUN2", 2.0, 30.0)]});
        std::fs::write(
            dir.path().join("data/alkis-nas/sonstiges-bauwerk.geojson"),
            layer.to_string(),
        )
        .unwrap();
        // OSM has the same fence 1 m off, and another one 10 m away.
        let fence = |id: &str, dn: f64| {
            let p = |de: f64| {
                crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                    easting: e + de,
                    northing: n + dn,
                })
                .unwrap()
            };
            OsmFeature {
                id: id.to_string(),
                tags: vec![("barrier".to_string(), "fence".to_string())],
                geometry: crate::sources::osm::OsmGeometry::Line(vec![p(0.0), p(30.0)]),
            }
        };
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&[fence("way/1", 1.0), fence("way/2", 10.0)]),
        )
        .unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        let tag = |o: &MapObject, key: &str| {
            o.tags
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        let objects: Vec<&MapObject> = documents
            .candidates
            .parts
            .iter()
            .flat_map(|p| &p.objects)
            .collect();
        let osm_ids: Vec<_> = objects
            .iter()
            .filter(|o| tag(o, "freemapper:source").as_deref() == Some("osm"))
            .filter_map(|o| tag(o, "freemapper:source_id"))
            .filter(|id| id == "way/1" || id == "way/2")
            .collect();
        assert_eq!(osm_ids, ["way/2"]);
        let nas: Vec<&&MapObject> = objects
            .iter()
            .filter(|o| {
                tag(o, "freemapper:source").as_deref() == Some("alkis-nas/sonstiges-bauwerk")
            })
            .collect();
        assert_eq!(nas.len(), 1, "the two segments are one fence");
        assert_eq!(
            Some(nas[0].symbol_id),
            project.map_kind.symbol_set().id_for_code("518")
        );
        assert_eq!(tag(nas[0], "freemapper:alkis").as_deref(), Some("Zaun"));
        assert_eq!(
            tag(nas[0], "freemapper:currency").as_deref(),
            Some("2019-05-06")
        );
        assert_eq!(
            tag(nas[0], "freemapper:also").as_deref(),
            Some("alkis-nas/sonstiges-bauwerk/DEBBAL00ZAUN2; osm/way/1")
        );
    }

    #[test]
    fn run_says_when_the_kept_map_lacks_the_canopies() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        let roof = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature", "id": "bauwerkeflaechen.DEBE00ROOF",
            "geometry": {"type": "MultiPolygon", "coordinates": [[[
                [392_760.0, 5_820_860.0], [392_766.0, 5_820_860.0], [392_766.0, 5_820_864.0],
                [392_760.0, 5_820_864.0], [392_760.0, 5_820_860.0]]]]},
            "properties": {"uuid": "DEBE00ROOF", "bezbwf": "Überdachung"}}]});
        std::fs::write(
            dir.path().join("data/berlin/bauwerke-flaechen.geojson"),
            roof.to_string(),
        )
        .unwrap();
        // A map from before the canopies: it has the ALKIS buildings, but no 522.
        let map = map_path(dir.path());
        std::fs::write(
            &map,
            format!("edited in OOM {ALKIS_SOURCE_TAG} {ORTHOPHOTO_RELPATH}"),
        )
        .unwrap();
        let run = |force: bool| {
            run_project(
                dir.path(),
                &fetcher,
                &cache,
                FetchMode::Offline,
                force,
                &mut |_| {},
            )
            .unwrap()
            .map
        };
        assert_eq!(
            run(false),
            MapOutcome::Kept {
                map: map.clone(),
                references_images: true,
                missing_buildings: false,
                missing_canopies: true,
            }
        );
        // Written again with force, the map has them.
        assert_eq!(run(true), MapOutcome::Written(map.clone()));
        assert!(matches!(
            run(false),
            MapOutcome::Kept {
                missing_canopies: false,
                ..
            }
        ));
    }

    #[test]
    fn passages_survive_without_any_extra_layer() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        // Every extra layer is gone, the buildings still carry the parts.
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        std::fs::remove_dir_all(dir.path().join("data/berlin")).unwrap();
        assert!(
            stored_berlin_layers(dir.path(), &project)
                .unwrap()
                .is_empty()
        );
        add_passage_part(dir.path());
        let found = berlin_candidates(dir.path());
        assert_eq!(
            found,
            [(
                "522".to_string(),
                "geoportal-berlin/building-parts".to_string()
            )]
        );
    }

    #[test]
    fn berlin_layers_left_out_for_lack_of_space_keep_the_stored_ones() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert!(!project.berlin_layers.is_empty());
        let calls = fetcher.calls.get();
        let mut messages = Vec::new();
        // The disk plan left the layers out: no request, the stored layers stay.
        let records = fetch_berlin_layers_into(
            dir.path(),
            &fetcher,
            &region_bbox,
            project.map_kind,
            FetchMode::Online,
            true,
            &project.berlin_layers,
            &mut |m: &str| messages.push(m.to_string()),
        );
        assert_eq!(fetcher.calls.get(), calls);
        assert_eq!(records, project.berlin_layers);
        assert!(
            dir.path()
                .join("data/berlin/bauwerke-flaechen.geojson")
                .exists()
        );
    }

    #[test]
    fn berlin_layers_are_stored_and_recorded() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);

        // Schoolyard map: the plus-1 layers (relief is for sprint maps), the survey layers of
        // both kinds and the schoolyard ones; an empty layer is stored too.
        let files: Vec<&str> = project.berlin_layers.keys().map(String::as_str).collect();
        assert_eq!(
            files,
            [
                "anlagenbaeume",
                "bauwerke-flaechen",
                "bauwerke-linien",
                "beleuchtung",
                "gebaeudehoehen",
                "sb-gelaender",
                "sb-mauer",
                "sb-moebel",
                "sb-objekte",
                "sb-treppe",
                "strassenbaeume",
                "trinkwasserbrunnen",
                "vegetation-linien",
                "zierbrunnen"
            ]
        );
        // Three lamps in operation of four, four railings of two types, three stairs.
        let counts: Vec<usize> = project.berlin_layers.values().map(|r| r.count).collect();
        assert_eq!(counts, [4, 10, 8, 3, 4, 4, 0, 0, 0, 3, 4, 0, 0, 3]);
        assert!(
            dir.path()
                .join("data/berlin/vegetation-linien.geojson")
                .exists()
        );
        assert!(!dir.path().join("data/berlin/relief.geojson").exists());
        assert!(
            !messages.iter().any(|m| m.contains("warning: Berlin")),
            "{messages:?}"
        );

        let stored = stored_berlin_layers(dir.path(), &project).unwrap();
        let sizes: Vec<(&str, usize)> = stored
            .iter()
            .map(|l| (l.layer.file, l.features.len()))
            .collect();
        assert_eq!(
            sizes,
            [
                ("bauwerke-flaechen", 10),
                ("bauwerke-linien", 8),
                ("vegetation-linien", 0),
                ("strassenbaeume", 4),
                ("anlagenbaeume", 4),
                ("sb-treppe", 3),
                ("sb-mauer", 0),
                ("sb-gelaender", 4),
                ("sb-moebel", 0),
                ("sb-objekte", 0),
                ("beleuchtung", 3),
                ("zierbrunnen", 3),
                ("trinkwasserbrunnen", 0),
                ("gebaeudehoehen", 4)
            ]
        );
        // The stored survey features carry their type and survey date.
        let railings = stored
            .iter()
            .find(|l| l.layer.file == "sb-gelaender")
            .unwrap();
        assert_eq!(
            railings.features[0]["properties"]["freemapper:layer"],
            "bf_gelaender"
        );
        assert_eq!(
            railings.features[0]["properties"]["survey_date"],
            "2015-05-02"
        );
        // The records survive the project file.
        assert_eq!(ProjectFile::load(dir.path()).unwrap(), project);
    }

    #[test]
    fn failing_berlin_layer_is_a_warning() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        let lines = &crate::sources::berlin::EXTRA_LAYERS[1];
        fetcher
            .failing
            .borrow_mut()
            .push(crate::sources::berlin::wfs_url(
                lines.service,
                lines.types[0].name,
                &region_bbox,
            ));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(!project.berlin_layers.contains_key("bauwerke-linien"));
        assert!(project.berlin_layers.contains_key("bauwerke-flaechen"));
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: Berlin bauwerke-linien unavailable")),
            "{messages:?}"
        );
        assert!(
            !dir.path()
                .join("data/berlin/bauwerke-linien.geojson")
                .exists()
        );
    }

    #[test]
    fn brandenburg_project_makes_no_berlin_layer_request() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.berlin_layers.is_empty());
        assert!(!dir.path().join(BERLIN_LAYERS_DIR).exists());
        assert!(
            !fetcher
                .log
                .borrow()
                .iter()
                .any(|url| url.contains("gdi.berlin.de/services/wfs/baumbestand")
                    || url.contains("bauwerke")),
            "no Berlin request outside Berlin"
        );
        assert!(
            stored_berlin_layers(dir.path(), &project)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn nas_exception_report_skips_the_layer() {
        use crate::sources::alkis_nas::{LGB_NAS, test_support};
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        // Every other NAS layer is empty there.
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        let url = |type_name: &str| LGB_NAS.get_feature_url(type_name, &bbox);
        let serve = |type_name: &str, answer: &str| {
            fetcher.insert(&url(type_name), answer.as_bytes().to_vec());
        };
        const WALLS: &str = "adv:AX_SonstigesBauwerkOderSonstigeEinrichtung";
        const HEDGES: &str = "adv:AX_Vegetationsmerkmal";
        serve(WALLS, test_support::SONSTIGES_BAUWERK);
        serve(HEDGES, test_support::EXCEPTION);
        let calls = fetcher.log.borrow().len();
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);

        // Schoolyard map: four layers asked once each; the hedges answer an exception report,
        // which is not asked again, and the layer is left out with a warning.
        let asked: Vec<String> = fetcher.log.borrow()[calls..]
            .iter()
            .filter(|u| u.starts_with(LGB_NAS.url))
            .cloned()
            .collect();
        assert_eq!(asked.len(), 4, "{asked:?}");
        let files: Vec<&str> = project.nas_layers.keys().map(String::as_str).collect();
        assert_eq!(
            files,
            [
                "bauwerk-gewaesser",
                "sonstiges-bauwerk",
                "untergeordnetes-gewaesser"
            ]
        );
        let warning = messages
            .iter()
            .find(|m| m.starts_with("warning: ALKIS NAS vegetationsmerkmal unavailable ("))
            .unwrap_or_else(|| panic!("{messages:?}"));
        assert!(
            warning.contains("InvalidParameterValue: no geometry property")
                && warning.ends_with("; layer skipped"),
            "{warning}"
        );
        let walls = &project.nas_layers["sonstiges-bauwerk"];
        assert_eq!(walls.count, 5);
        assert_eq!(walls.latest_update, NaiveDate::from_ymd_opt(2013, 1, 29));
        assert_eq!(walls.regions, [Region::Brandenburg]);
        let stored =
            std::fs::read_to_string(dir.path().join("data/alkis-nas/sonstiges-bauwerk.geojson"))
                .unwrap();
        assert_eq!(berlin::parse_layer(&stored).unwrap().len(), 5);
        assert!(
            !dir.path()
                .join("data/alkis-nas/vegetationsmerkmal.geojson")
                .exists()
        );
        assert_eq!(ProjectFile::load(dir.path()).unwrap(), project);

        // Next time the walls fail and the hedges answer: the stored walls stay.
        serve(WALLS, test_support::EXCEPTION);
        serve(HEDGES, test_support::VEGETATION);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m
                .starts_with("warning: ALKIS NAS sonstiges-bauwerk unavailable (")
                && m.ends_with("; keeping the stored layer")),
            "{messages:?}"
        );
        assert_eq!(project.nas_layers["sonstiges-bauwerk"], *walls);
        assert_eq!(project.nas_layers["vegetationsmerkmal"].count, 3);
        assert_eq!(project.nas_layers.len(), 4);

        // Offline nothing is asked and every stored layer stays.
        let calls = fetcher.log.borrow().len();
        let mut messages = Vec::new();
        let offline = fetch_project(dir.path(), &fetcher, &cache, FetchMode::Offline, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert_eq!(offline.nas_layers, project.nas_layers);
        assert!(
            !fetcher.log.borrow()[calls..]
                .iter()
                .any(|u| u.starts_with(LGB_NAS.url))
        );
        assert!(
            dir.path()
                .join("data/alkis-nas/vegetationsmerkmal.geojson")
                .exists()
        );
    }

    #[test]
    fn leaving_berlin_deletes_stored_layers_only_online() {
        let dir = tempfile::tempdir().unwrap();
        let path = layer_path(dir.path(), "relief");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{}").unwrap();
        let record = AlkisRecord {
            bbox: [0.0; 4],
            fetched: NaiveDate::from_ymd_opt(2026, 11, 8).unwrap(),
            latest_update: None,
            count: 0,
            regions: vec![Region::Berlin],
        };
        let previous = BTreeMap::from([("relief".to_string(), record)]);
        // Strausberg, Brandenburg.
        let away = BoundingBox {
            min_e: 421_490.0,
            min_n: 5_821_580.0,
            max_e: 421_736.0,
            max_n: 5_821_896.0,
        };
        let fetcher = MapFetcher::default();
        let mut messages = Vec::new();
        let mut progress = |m: &str| messages.push(m.to_string());
        let kept = fetch_berlin_layers_into(
            dir.path(),
            &fetcher,
            &away,
            MapKind::Sprint,
            FetchMode::Offline,
            false,
            &previous,
            &mut progress,
        );
        assert_eq!(kept, previous, "offline never forgets stored layers");
        assert!(path.exists());
        let kept = fetch_berlin_layers_into(
            dir.path(),
            &fetcher,
            &away,
            MapKind::Sprint,
            FetchMode::Online,
            false,
            &previous,
            &mut progress,
        );
        assert!(kept.is_empty());
        assert!(!path.exists());
        assert_eq!(fetcher.calls.get(), 0);
        assert_eq!(
            messages,
            ["removing the stored Berlin layers: the map area is no longer near Berlin"]
        );
    }

    /// Path of the stored extra Berlin layer `file` of the table.
    fn layer_path(dir: &Path, file: &str) -> PathBuf {
        berlin_layer_path(dir, known_berlin_layer(file).expect("a layer of the table"))
    }

    /// Removes a file a test put outside its temporary folder, also when the test fails.
    struct RemovedAtEnd(PathBuf);

    impl Drop for RemovedAtEnd {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// A Berlin layer record under `key`, as a crafted project file would hold it.
    fn layer_record_toml(key: &str) -> String {
        // TOML spells NUL `\u0000`, Rust `\0`.
        let key = format!("{key:?}").replace(r"\0", r"\u0000");
        format!(
            "\n[berlin_layers.{key}]\nbbox = [0.0, 0.0, 1.0, 1.0]\nfetched = \"2026-11-08\"\ncount = 1\n"
        )
    }

    fn some_layer_record() -> AlkisRecord {
        AlkisRecord {
            bbox: [0.0; 4],
            fetched: NaiveDate::from_ymd_opt(2026, 11, 8).unwrap(),
            latest_update: None,
            count: 0,
            regions: vec![Region::Berlin],
        }
    }

    const IGNORED_LAYER_WARNING: &str = "warning: freemapper.toml: ignored the Berlin layer entry";

    #[test]
    fn berlin_layer_keys_never_delete_files_outside_the_layers_folder() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        // `../../evil` leads to the project folder itself, three steps up out of it.
        let inside = dir.path().join("evil.geojson");
        std::fs::write(&inside, b"{}").unwrap();
        let name = format!("{}-evil", dir.path().file_name().unwrap().to_str().unwrap());
        let outside = dir.path().parent().unwrap().join(format!("{name}.geojson"));
        std::fs::write(&outside, b"{}").unwrap();
        let _outside = RemovedAtEnd(outside.clone());
        let file = dir.path().join(PROJECT_FILE_NAME);
        let mut text = std::fs::read_to_string(&file).unwrap();
        text.push_str(&layer_record_toml("../../evil"));
        text.push_str(&layer_record_toml(&format!("../../../{name}")));
        std::fs::write(&file, text).unwrap();
        // The layers folder exists, so the system would resolve the `..`.
        std::fs::create_dir_all(dir.path().join(BERLIN_LAYERS_DIR)).unwrap();

        let (project, messages) = fetch(dir.path(), &fetcher, &cache);

        assert!(outside.exists(), "the file outside the project is kept");
        assert!(inside.exists(), "the file in the project folder is kept");
        let warnings: Vec<&String> = messages
            .iter()
            .filter(|m| m.starts_with(IGNORED_LAYER_WARNING))
            .collect();
        assert_eq!(warnings.len(), 2, "{messages:?}");
        assert!(
            warnings.iter().any(|w| w.contains(r#""../../evil""#)),
            "{warnings:?}"
        );
        assert!(project.berlin_layers.keys().all(|k| !k.contains("..")));
        let saved = ProjectFile::load(dir.path()).unwrap();
        assert!(saved.berlin_layers.keys().all(|k| !k.contains("..")));
        assert!(saved.berlin_layers.contains_key("bauwerke-flaechen"));
    }

    #[test]
    fn stored_layers_count_as_downloaded_only_where_they_cover() {
        let today = chrono::Local::now().date_naive();
        let item = |dir: &Path, cache: &Cache, what: &str| {
            crate::layers::estimate_download(dir, cache, today)
                .unwrap()
                .items
                .into_iter()
                .find(|i| i.what == what)
                .unwrap_or_else(|| panic!("{what}"))
        };
        let moved = |records: &mut BTreeMap<String, AlkisRecord>| {
            for record in records.values_mut() {
                record.bbox = [0.0, 0.0, 1.0, 1.0];
            }
        };
        // Brandenburg: the NAS layers of the fetch cover the area; moved elsewhere, they do not.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let nas = crate::layers::NAS_LAYERS_ITEM;
        assert!(item(dir.path(), &cache, nas).cached);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        assert!(!project.nas_layers.is_empty());
        moved(&mut project.nas_layers);
        project.save(dir.path()).unwrap();
        assert!(!item(dir.path(), &cache, nas).cached);
        // The same for the Berlin layers.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        serve_berlin_layers(&fetcher, &region_bbox);
        fetch(dir.path(), &fetcher, &cache);
        let berlin = crate::layers::BERLIN_LAYERS_ITEM;
        assert!(item(dir.path(), &cache, berlin).cached);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        moved(&mut project.berlin_layers);
        project.save(dir.path()).unwrap();
        assert!(!item(dir.path(), &cache, berlin).cached);
    }

    #[cfg(unix)]
    #[test]
    fn nas_part_file_link_is_not_followed() {
        use crate::sources::alkis_nas::{LGB_NAS, test_support};
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let bbox = ProjectFile::load(dir.path()).unwrap().bbox().unwrap();
        fetcher.insert(
            &LGB_NAS.get_feature_url("adv:AX_SonstigesBauwerkOderSonstigeEinrichtung", &bbox),
            test_support::SONSTIGES_BAUWERK.as_bytes().to_vec(),
        );
        // A part file of the old fixed name that is a link to a file outside the project.
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.geojson");
        std::fs::write(&victim, b"secret").unwrap();
        let layers = dir.path().join(NAS_LAYERS_DIR);
        std::fs::create_dir_all(&layers).unwrap();
        std::os::unix::fs::symlink(&victim, layers.join("answer.gml.part")).unwrap();
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(std::fs::read(&victim).unwrap(), b"secret");
        assert_eq!(project.nas_layers["sonstiges-bauwerk"].count, 5);
        // The downloads used fresh names of their own, removed after each answer; the link is
        // not freemapper's and stays.
        let parts: Vec<String> = std::fs::read_dir(&layers)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".part"))
            .collect();
        assert_eq!(parts, ["answer.gml.part"]);
    }

    #[cfg(unix)]
    #[test]
    fn nas_stale_parts_behind_a_linked_folder_survive() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        // The NAS folder is a link to a folder outside the project that holds an old part file.
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("x.part");
        let file = std::fs::File::create(&victim).unwrap();
        file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(86_400))
            .unwrap();
        drop(file);
        std::fs::create_dir_all(dir.path().join("data")).unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join(NAS_LAYERS_DIR)).unwrap();
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            victim.exists(),
            "the part file outside the project was removed"
        );
        assert!(project.nas_layers.is_empty());
        assert!(
            messages
                .iter()
                .any(|m| m.contains("leads out of the project folder")),
            "{messages:?}"
        );
    }

    /// The Sanssouci area (Potsdam) as a fetch area, where the tree fixture stands.
    fn sanssouci_area() -> FetchArea {
        FetchArea::new(
            BoundingBox {
                min_e: 366_200.0,
                min_n: 5_807_100.0,
                max_e: 367_100.0,
                max_n: 5_807_700.0,
            },
            Crs::Utm33,
        )
    }

    fn serve_potsdam_trees(fetcher: &MapFetcher) {
        use crate::sources::trees::{POTSDAM, test_support};
        fetcher.insert_prefix(
            POTSDAM.endpoints[0],
            test_support::POTSDAM.as_bytes().to_vec(),
        );
        for endpoint in &POTSDAM.endpoints[1..] {
            fetcher.insert_prefix(
                endpoint,
                br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
            );
        }
    }

    #[test]
    fn tree_layers_are_fetched_stored_and_kept() {
        let fetcher = MapFetcher::default();
        serve_potsdam_trees(&fetcher);
        let dir = tempfile::tempdir().unwrap();
        let mut messages = Vec::new();
        let records = fetch_tree_layers_into(
            dir.path(),
            &fetcher,
            &sanssouci_area(),
            FetchMode::Online,
            false,
            &BTreeMap::new(),
            &mut |m| messages.push(m.to_string()),
        );
        assert_eq!(fetcher.calls.get(), 3, "{messages:?}");
        let record = &records["potsdam"];
        assert_eq!(record.count, 3);
        assert_eq!(record.latest_update, NaiveDate::from_ymd_opt(2009, 6, 10));
        assert_eq!(record.regions, [Region::Brandenburg]);
        assert_eq!(
            record.bbox,
            [366_200.0, 5_807_100.0, 367_100.0, 5_807_700.0]
        );
        let stored =
            std::fs::read_to_string(dir.path().join("data/trees/potsdam.geojson")).unwrap();
        assert_eq!(berlin::parse_layer(&stored).unwrap().len(), 3);

        // The endpoints fail next time: the stored file stays, with a warning.
        let failing = MapFetcher::default();
        let kept = fetch_tree_layers_into(
            dir.path(),
            &failing,
            &sanssouci_area(),
            FetchMode::Online,
            false,
            &records,
            &mut |m| messages.push(m.to_string()),
        );
        assert_eq!(kept, records);
        assert!(
            messages.iter().any(
                |m| m.starts_with("warning: potsdam tree cadastre unavailable (")
                    && m.ends_with("; keeping the stored one")
            ),
            "{messages:?}"
        );
        // Offline asks nothing and keeps it too; with the disk plan against it, so does online.
        let calls = fetcher.calls.get();
        for (mode, skip) in [(FetchMode::Offline, false), (FetchMode::Online, true)] {
            let same = fetch_tree_layers_into(
                dir.path(),
                &fetcher,
                &sanssouci_area(),
                mode,
                skip,
                &records,
                &mut |_| {},
            );
            assert_eq!(same, records);
        }
        assert_eq!(fetcher.calls.get(), calls);

        // The area leaves Potsdam: nothing is asked and the file goes.
        let strausberg = FetchArea::new(
            BoundingBox {
                min_e: 421_000.0,
                min_n: 5_821_000.0,
                max_e: 422_500.0,
                max_n: 5_822_000.0,
            },
            Crs::Utm33,
        );
        let none = fetch_tree_layers_into(
            dir.path(),
            &fetcher,
            &strausberg,
            FetchMode::Online,
            false,
            &records,
            &mut |_| {},
        );
        assert!(none.is_empty());
        assert_eq!(fetcher.calls.get(), calls);
        assert!(!dir.path().join("data/trees/potsdam.geojson").exists());
    }

    #[test]
    fn tree_layer_keys_never_name_paths() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let victim = dir.path().join("victim.geojson");
        std::fs::write(&victim, b"{}").unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let far = elsewhere.path().join("victim.geojson");
        std::fs::write(&far, b"{}").unwrap();
        let far_key = far.with_extension("").display().to_string();
        let unsafe_keys = ["../../victim", far_key.as_str(), "no-such-city"];
        let file = dir.path().join(PROJECT_FILE_NAME);
        let mut text = std::fs::read_to_string(&file).unwrap();
        for key in unsafe_keys {
            text.push_str(&format!(
                "\n[tree_layers.{key:?}]\nbbox = [0.0, 0.0, 1.0, 1.0]\nfetched = \"2026-11-08\"\ncount = 1\n"
            ));
        }
        std::fs::write(&file, text).unwrap();
        let mut warnings = Vec::new();
        let mut project =
            ProjectFile::load_reporting(dir.path(), &mut |m| warnings.push(m.to_string())).unwrap();
        assert!(project.tree_layers.is_empty(), "{:?}", project.tree_layers);
        assert_eq!(
            warnings
                .iter()
                .filter(|w| w.starts_with("warning: freemapper.toml: ignored the tree layer entry"))
                .count(),
            3,
            "{warnings:?}"
        );
        // Forgetting the fetched data deletes only datasets of the table.
        project.tree_layers = unsafe_keys
            .iter()
            .map(|k| (k.to_string(), some_layer_record()))
            .collect();
        project.drop_fetched(dir.path());
        assert!(project.tree_layers.is_empty());
        assert!(victim.exists(), "the file in the project folder is kept");
        assert!(far.exists(), "the file outside the project is kept");
    }

    #[test]
    fn hidden_candidates_stay_out_of_the_training_pdf() {
        use crate::build::points::test_support::fixture;
        let f = fixture(MapKind::Sprint);
        let outline = f.rect("520", (0.0, 0.0), (1.0, 1.0));
        let mut hidden = f.tree("418", "osm", "node/1", (0.0, 0.0), None);
        hidden.tags.push((HIDDEN_KEY.into(), "520".into()));
        let shown = f.tree("418", "osm", "node/2", (10.0, 0.0), None);
        let document = |objects: Vec<MapObject>| MapDocument {
            notes: String::new(),
            georef: f.georef.clone(),
            symbol_set: f.set.clone(),
            parts: vec![MapPart {
                name: "candidates".into(),
                objects,
            }],
            templates: Vec::new(),
        };
        let documents = StarterDocuments {
            map: document(Vec::new()),
            candidates: document(vec![outline, hidden.clone(), shown.clone()]),
            attribution_text: String::new(),
        };
        let (map, _) = training_objects(&documents);
        assert_eq!(map, [&shown]);
        // The candidates map keeps it.
        assert!(documents.candidates.parts[0].objects.contains(&hidden));
    }

    #[test]
    fn generalized_area_provenance_and_attribution() {
        use crate::build::points::test_support::{fixture, ids};
        const POTSDAM: &str = "Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0";
        let f = fixture(MapKind::Sprint);
        let mut trees = f.grid("trees/potsdam", (3, 3), 3.5, 6.0);
        for (i, tree) in trees.iter_mut().enumerate() {
            let date = format!("2024-05-{:02}", 9 - i);
            tree.tags
                .iter_mut()
                .find(|(k, _)| k == "freemapper:currency")
                .unwrap()
                .1 = date;
        }
        let found = f.run(trees, vec![], &[], &[], &[]);
        assert!(ids(&found.candidates).is_empty());
        assert_eq!(found.areas.len(), 1);
        let tags: Vec<(&str, &str)> = found.areas[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let merged: Vec<String> = (0..9).map(|i| format!("trees/potsdam/t{i:02}")).collect();
        assert_eq!(tags.len(), 4, "{tags:?}");
        assert_eq!(tags[0], ("freemapper:source", "generalization"));
        assert_eq!(
            tags[1],
            ("freemapper:source_id", merged.join("; ").as_str())
        );
        assert_eq!(tags[2], ("freemapper:currency", "2024-05-01"));
        assert_eq!(tags[3].0, "freemapper:generalized");
        assert!(tags[3].1.starts_with("9 trees, cover "), "{tags:?}");
        // The Potsdam trees are drawn only inside the area: the Potsdam line is still given.
        let drawn = drawn_registers(&found.areas);
        assert_eq!(drawn, ["potsdam"]);
        assert!(tree_attribution(&drawn).contains(&POTSDAM.to_string()));
        // A source is named by the source of an entry, not by any prefix of it.
        let mut other = found.areas[0].clone();
        other.tags[1].1 = "trees/potsdamer/1; osm/node/1".into();
        assert!(drawn_registers(&[other]).is_empty());
    }

    #[test]
    fn potsdam_attribution_only_when_drawn() {
        const POTSDAM: &str = "Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0";
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let project = ProjectFile::load(dir.path()).unwrap();
        // Nothing drawn: no line, even for a project whose tree file has trees.
        let mut stored = project.clone();
        stored
            .tree_layers
            .insert("potsdam".into(), some_layer_record());
        assert!(!attribution_lines(&stored, &[]).contains(&POTSDAM.to_string()));
        assert!(!attribution_lines(&project, &[]).contains(&POTSDAM.to_string()));
        // A Berlin register that drew objects is not the tree cadastre.
        assert!(!attribution_lines(&project, &["zierbrunnen"]).contains(&POTSDAM.to_string()));
        // Drawn: the object's source names the dataset.
        let object = |source: &str| MapObject {
            symbol_id: 0,
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(0, 0)],
            tags: vec![("freemapper:source".into(), source.into())],
        };
        assert!(drawn_registers(&[object("trees/other"), object("osm")]).is_empty());
        let drawn = drawn_registers(&[object("osm"), object("trees/potsdam")]);
        assert_eq!(drawn, ["potsdam"]);
        let lines = attribution_lines(&project, &drawn);
        assert_eq!(
            lines.iter().filter(|l| *l == POTSDAM).count(),
            1,
            "{lines:?}"
        );
    }

    #[test]
    fn nas_layer_keys_never_name_paths() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        // `../../victim` leads to the project folder itself; an absolute key replaces the path.
        let victim = dir.path().join("victim.geojson");
        std::fs::write(&victim, b"{}").unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let far = elsewhere.path().join("victim.geojson");
        std::fs::write(&far, b"{}").unwrap();
        let far_key = far.with_extension("").display().to_string();
        let unsafe_keys = ["../../victim", far_key.as_str(), "no-such-layer"];
        let file = dir.path().join(PROJECT_FILE_NAME);
        let mut text = std::fs::read_to_string(&file).unwrap();
        for key in unsafe_keys {
            text.push_str(&format!(
                "\n[nas_layers.{key:?}]\nbbox = [0.0, 0.0, 1.0, 1.0]\nfetched = \"2026-11-08\"\ncount = 1\n"
            ));
        }
        std::fs::write(&file, text).unwrap();

        // Loading drops them with a warning each.
        let mut warnings = Vec::new();
        let project =
            ProjectFile::load_reporting(dir.path(), &mut |m| warnings.push(m.to_string())).unwrap();
        assert!(
            project
                .nas_layers
                .keys()
                .all(|k| alkis_nas::NAS_LAYERS.iter().any(|l| l.file == k)),
            "{:?}",
            project.nas_layers
        );
        let ignored: Vec<&String> = warnings
            .iter()
            .filter(|w| w.starts_with("warning: freemapper.toml: ignored the NAS layer entry"))
            .collect();
        assert_eq!(ignored.len(), 3, "{warnings:?}");

        // An online fetch, the cleanup of a state left, and forgetting the fetched data delete
        // only layers of the table.
        fetch(dir.path(), &fetcher, &cache);
        let previous: BTreeMap<String, AlkisRecord> = unsafe_keys
            .iter()
            .chain(&["sonstiges-bauwerk"])
            .map(|k| (k.to_string(), some_layer_record()))
            .collect();
        let strausberg = FetchArea::new(project.region_bbox().unwrap(), Crs::Utm33);
        let mitte = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_500.0,
            max_n: 5_821_000.0,
        };
        for area in [strausberg, FetchArea::new(mitte, Crs::Utm33)] {
            fetch_nas_layers_into(
                dir.path(),
                &fetcher,
                &area,
                MapKind::Sprint,
                FetchMode::Online,
                false,
                &previous,
                &mut |_| {},
            );
        }
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.nas_layers = previous;
        project.drop_fetched(dir.path());
        assert!(project.nas_layers.is_empty());
        assert!(victim.exists(), "the file in the project folder is kept");
        assert!(far.exists(), "the file outside the project is kept");
    }

    #[test]
    fn leaving_berlin_deletes_only_known_layers() {
        let dir = tempfile::tempdir().unwrap();
        let relief = layer_path(dir.path(), "relief");
        std::fs::create_dir_all(relief.parent().unwrap()).unwrap();
        std::fs::write(&relief, b"{}").unwrap();
        let victim = dir.path().join("evil.geojson");
        std::fs::write(&victim, b"{}").unwrap();
        let previous = BTreeMap::from([
            ("relief".to_string(), some_layer_record()),
            ("../../evil".to_string(), some_layer_record()),
        ]);
        // Strausberg, Brandenburg.
        let away = BoundingBox {
            min_e: 421_490.0,
            min_n: 5_821_580.0,
            max_e: 421_736.0,
            max_n: 5_821_896.0,
        };
        let kept = fetch_berlin_layers_into(
            dir.path(),
            &MapFetcher::default(),
            &away,
            MapKind::Sprint,
            FetchMode::Online,
            false,
            &previous,
            &mut |_| {},
        );
        assert!(kept.is_empty());
        assert!(!relief.exists());
        assert!(victim.exists());
    }

    #[test]
    fn forgetting_fetched_data_deletes_only_known_layers() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _cache) = berlin_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let relief = layer_path(dir.path(), "relief");
        std::fs::create_dir_all(relief.parent().unwrap()).unwrap();
        std::fs::write(&relief, b"{}").unwrap();
        let victim = dir.path().join("evil.geojson");
        std::fs::write(&victim, b"{}").unwrap();
        for key in ["relief", "../../evil"] {
            project
                .berlin_layers
                .insert(key.to_string(), some_layer_record());
        }
        project.drop_fetched(dir.path());
        assert!(project.berlin_layers.is_empty());
        assert!(!relief.exists());
        assert!(victim.exists());
    }

    #[test]
    fn project_file_with_unsafe_layer_keys_loads_without_them() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _cache) = berlin_school(&fetcher);
        let file = dir.path().join(PROJECT_FILE_NAME);
        let mut text = std::fs::read_to_string(&file).unwrap();
        let keys = [
            "relief",
            "../../evil",
            "/etc/evil",
            "a/b",
            r"a\b",
            "C:evil",
            "..",
            "nul\u{0}x",
            "no-such-layer",
        ];
        for key in keys {
            text.push_str(&layer_record_toml(key));
        }
        std::fs::write(&file, text).unwrap();
        let mut warnings = Vec::new();
        let project =
            ProjectFile::load_reporting(dir.path(), &mut |m| warnings.push(m.to_string())).unwrap();
        assert_eq!(project.berlin_layers.keys().collect::<Vec<_>>(), ["relief"]);
        assert_eq!(warnings.len(), keys.len() - 1, "{warnings:?}");
        assert!(
            warnings
                .iter()
                .all(|w| w.starts_with(IGNORED_LAYER_WARNING)),
            "{warnings:?}"
        );
        assert!(
            warnings.iter().any(|w| w.contains(r#""nul\0x""#)),
            "control characters are shown escaped: {warnings:?}"
        );
        // `load` drops them too, silently.
        assert_eq!(ProjectFile::load(dir.path()).unwrap(), project);
    }

    #[cfg(unix)]
    #[test]
    fn layers_folder_leading_out_of_the_project_is_left_alone() {
        let outside = tempfile::tempdir().unwrap();
        let relief = outside.path().join("relief.geojson");
        std::fs::write(&relief, b"{}").unwrap();
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let region_bbox = ProjectFile::load(dir.path())
            .unwrap()
            .region_bbox()
            .unwrap();
        std::fs::create_dir_all(dir.path().join("data")).unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join(BERLIN_LAYERS_DIR)).unwrap();

        // No layer is written through the link.
        serve_berlin_layers(&fetcher, &region_bbox);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        let written: Vec<_> = std::fs::read_dir(outside.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(written, ["relief.geojson"], "{messages:?}");
        assert!(project.berlin_layers.is_empty());
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: Berlin bauwerke-flaechen unavailable")),
            "{messages:?}"
        );

        // No layer is deleted through it.
        let previous = BTreeMap::from([("relief".to_string(), some_layer_record())]);
        let away = BoundingBox {
            min_e: 421_490.0,
            min_n: 5_821_580.0,
            max_e: 421_736.0,
            max_n: 5_821_896.0,
        };
        fetch_berlin_layers_into(
            dir.path(),
            &fetcher,
            &away,
            MapKind::Sprint,
            FetchMode::Online,
            false,
            &previous,
            &mut |_| {},
        );
        assert!(relief.exists());
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
            !attribution_lines(&project, &[])
                .iter()
                .any(|l| l.contains("Berlin"))
        );
        project.buildings.as_mut().unwrap().regions = vec![Region::Brandenburg, Region::Berlin];
        assert!(attribution_lines(&project, &[]).contains(&BERLIN_ATTRIBUTION.to_string()));
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
    fn berlin_sprint_export_cuts_survey_green_and_adds_survey_paving() {
        use crate::sources::alkis::test_support::{land_use_feature, page};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = berlin_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.map_kind = MapKind::Sprint;
        project.scale_denominator = 4000;
        project.save(dir.path()).unwrap();
        serve_berlin_layers(&fetcher, &project.region_bbox().unwrap());
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.berlin_layers.contains_key(GREEN_LAYER));
        assert!(project.berlin_layers.contains_key(SURFACE_LAYER));
        // A 40 m street parcel; a 10 m green inside it, a sidewalk north of it.
        let street = land_use_feature(
            "DEBE00STRASSE",
            "Straßenverkehr",
            None,
            [392_760.0, 5_820_810.0, 40.0],
        );
        std::fs::write(dir.path().join(LAND_USE_RELPATH), page(&[street], None)).unwrap();
        let area = |layer: &str, gis_id: &str, (e, n): (f64, f64), (w, h): (f64, f64)| {
            serde_json::json!({
                "type": "Feature", "id": format!("{layer}.1"),
                "geometry": {"type": "Polygon", "coordinates": [[
                    [e, n], [e + w, n], [e + w, n + h], [e, n + h], [e, n]]]},
                "properties": {"freemapper:layer": layer, "gis_id": gis_id,
                    "survey_date": "2015-04-16"}})
        };
        for (file, feature) in [
            (
                GREEN_LAYER,
                area(
                    "ce_gruenflaeche",
                    "Fl_20150416_1",
                    (392_775.0, 5_820_825.0),
                    (10.0, 10.0),
                ),
            ),
            (
                SURFACE_LAYER,
                area(
                    "cl_gehweg",
                    "Fl_20150416_2",
                    (392_770.0, 5_820_850.0),
                    (20.0, 6.0),
                ),
            ),
        ] {
            let layer = serde_json::json!({"type": "FeatureCollection", "features": [feature]});
            std::fs::write(layer_path(dir.path(), file), layer.to_string()).unwrap();
        }
        let documents = starter_documents(dir.path()).unwrap();
        let tag = |o: &MapObject, key: &str| {
            o.tags
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        let street = documents.map.parts[0]
            .objects
            .iter()
            .find(|o| tag(o, "freemapper:source_id").as_deref() == Some("DEBE00STRASSE"))
            .expect("the street");
        assert_eq!(
            tag(street, "freemapper:cut_ids").as_deref(),
            Some("Fl_20150416_1")
        );
        let sidewalk = documents.candidates.parts[0]
            .objects
            .iter()
            .find(|o| tag(o, "freemapper:source_id").as_deref() == Some("Fl_20150416_2"))
            .expect("the sidewalk candidate");
        assert_eq!(
            tag(sidewalk, "freemapper:currency").as_deref(),
            Some("2015-04-16")
        );
    }

    /// One 30 m lake parcel in the stored land use, and two OSM ponds as closed ways: one inside
    /// the lake, one clear of it.
    fn store_lake_and_ponds(dir: &Path) {
        use crate::sources::alkis::test_support::{land_use_feature, page};
        let lake = land_use_feature(
            "DEBBAL0100000040",
            "Stehendes Gewässer",
            None,
            [421_560.0, 5_821_690.0, 30.0],
        );
        std::fs::write(dir.join(LAND_USE_RELPATH), page(&[lake], None)).unwrap();
        let pond = |id: &str, e: f64, n: f64| crate::sources::osm::OsmFeature {
            id: id.to_string(),
            tags: [("natural".to_string(), "water".to_string())]
                .into_iter()
                .collect(),
            geometry: crate::sources::osm::OsmGeometry::Line(
                [
                    (0.0, 0.0),
                    (10.0, 0.0),
                    (10.0, 10.0),
                    (0.0, 10.0),
                    (0.0, 0.0),
                ]
                .into_iter()
                .map(|(de, dn)| lat_lon_at(e + de, n + dn))
                .collect(),
            ),
        };
        let features = [
            pond("way/10", 421_570.0, 5_821_700.0),
            pond("way/11", 421_620.0, 5_821_700.0),
        ];
        std::fs::write(
            dir.join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&features),
        )
        .unwrap();
    }

    #[test]
    fn export_adds_water_to_both_map_kinds() {
        let fetcher = MapFetcher::default();
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let (dir, _cache_dir) = sprint_with_ways(&fetcher);
            let mut project = ProjectFile::load(dir.path()).unwrap();
            project.map_kind = kind;
            project.scale_denominator = if kind == MapKind::Sprint { 4000 } else { 1000 };
            project.save(dir.path()).unwrap();
            store_lake_and_ponds(dir.path());
            let map = std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap())
                .unwrap();
            let set = kind.symbol_set();
            let (fill, bank) = (
                set.id_for_code("301.1").unwrap(),
                set.id_for_code("301.6").unwrap(),
            );
            assert_eq!(symbols_of(&map, "DEBBAL0100000040").len(), 2, "{kind:?}");
            assert!(map.contains(&format!(r#"symbol="{fill}""#)), "{kind:?}");
            assert!(map.contains(&format!(r#"symbol="{bank}""#)), "{kind:?}");
            let candidates =
                std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
            let pond = set.id_for_code("301").unwrap();
            assert_eq!(
                symbols_of(&candidates, "way/10"),
                Vec::<u32>::new(),
                "{kind:?}: the pond inside the lake is dropped"
            );
            assert_eq!(symbols_of(&candidates, "way/11"), [pond], "{kind:?}");
        }
    }

    #[test]
    fn fountain_jets_in_a_basin_are_left_out_of_the_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        // The ALKIS lake (421 560–590 E, 5 821 690–720 N) and the OSM pond way/11 outside it
        // (421 620–630 E, 5 821 700–710 N), which stays a candidate.
        store_lake_and_ponds(dir.path());
        let path = dir.path().join(OSM_CANDIDATES_RELPATH);
        let mut features =
            crate::sources::osm::features_from_geojson(&std::fs::read_to_string(&path).unwrap())
                .unwrap();
        let node = |id: &str, tag: (&str, &str), e: f64, n: f64| crate::sources::osm::OsmFeature {
            id: id.to_string(),
            tags: [(tag.0.to_string(), tag.1.to_string())]
                .into_iter()
                .collect(),
            geometry: crate::sources::osm::OsmGeometry::Point(lat_lon_at(e, n)),
        };
        features.extend([
            node("node/20", ("amenity", "fountain"), 421_575.0, 5_821_705.0),
            node("node/21", ("amenity", "fountain"), 421_625.0, 5_821_705.0),
            node(
                "node/22",
                ("man_made", "water_well"),
                421_626.0,
                5_821_706.0,
            ),
            node("node/23", ("amenity", "fountain"), 421_605.0, 5_821_700.0),
        ]);
        std::fs::write(&path, crate::sources::osm::features_to_geojson(&features)).unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let fountain = MapKind::Sprint.symbol_set().id_for_code("311").unwrap();
        for jet in ["node/20", "node/21", "node/22"] {
            assert_eq!(symbols_of(&candidates, jet), Vec::<u32>::new(), "{jet}");
        }
        assert_eq!(symbols_of(&candidates, "node/23"), [fountain]);
    }

    #[test]
    fn osm_pond_without_alkis_water_stays_a_candidate() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        store_lake_and_ponds(dir.path());
        // No water parcel: the same ponds, nothing to drop them.
        store_traffic_areas(dir.path());
        let candidates = {
            export_starter_project(dir.path(), false).unwrap();
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap()
        };
        let pond = MapKind::Sprint.symbol_set().id_for_code("301").unwrap();
        assert_eq!(symbols_of(&candidates, "way/10"), [pond]);
        assert_eq!(symbols_of(&candidates, "way/11"), [pond]);
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
    fn north_lines_are_in_the_map_part() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let documents = starter_documents(dir.path()).unwrap();
        let north_line = documents.map.symbol_set.id_for_code("601");
        let is_north_line = |o: &MapObject| Some(o.symbol_id) == north_line;
        let part = |name: &str| documents.map.parts.iter().find(|p| p.name == name).unwrap();
        let lines: Vec<&MapObject> = part("default part")
            .objects
            .iter()
            .filter(|o| is_north_line(o))
            .collect();
        assert!(!lines.is_empty());
        for line in lines {
            assert_eq!(
                line.tags,
                [("freemapper:source".to_string(), "north-lines".to_string())]
            );
        }
        assert!(!part(SHEET_PART).objects.iter().any(is_north_line));
        // The training map clips them to the content and paints them first within their colour,
        // so walls and fences of the same black lie on top.
        let (map, sheet) = training_objects(&documents);
        let first_other = map.iter().position(|o| !is_north_line(o));
        let last_line = map.iter().rposition(|o| is_north_line(o)).unwrap();
        assert!(first_other.is_none_or(|i| last_line < i), "{first_other:?}");
        assert!(!sheet.iter().any(|o| is_north_line(o)));
    }

    #[test]
    fn info_block_stays_on_the_sheet() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let documents = starter_documents(dir.path()).unwrap();
        let set = &documents.map.symbol_set;
        let sheet_ids: Vec<Option<u32>> = ["991", "992", "993", "994"]
            .iter()
            .map(|code| set.id_for_code(code))
            .collect();
        let sheet = documents
            .map
            .parts
            .iter()
            .find(|p| p.name == SHEET_PART)
            .unwrap();
        assert!(!sheet.objects.is_empty());
        for o in &sheet.objects {
            assert!(sheet_ids.contains(&Some(o.symbol_id)), "{o:?}");
            assert_eq!(
                o.tags,
                [("freemapper:source".to_string(), "sheet".to_string())]
            );
        }
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
    fn contours_stop_at_water() {
        use crate::derive::contours::ContourLine;
        use crate::sources::alkis::test_support::{land_use_feature, page};
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        // A 30 m lake: 421 560–590 E, 5 821 690–720 N.
        let lake = land_use_feature(
            "DEBBAL0100000040",
            "Stehendes Gewässer",
            None,
            [421_560.0, 5_821_690.0, 30.0],
        );
        std::fs::write(dir.path().join(LAND_USE_RELPATH), page(&[lake], None)).unwrap();
        let lines = [
            // Across the lake, with a vertex in the water.
            ContourLine {
                height: 63.0,
                points: vec![
                    (421_540.0, 5_821_705.0),
                    (421_575.0, 5_821_705.0),
                    (421_640.0, 5_821_705.0),
                ],
            },
            // A ring inside the lake.
            ContourLine {
                height: 64.0,
                points: vec![
                    (421_570.0, 5_821_700.0),
                    (421_580.0, 5_821_700.0),
                    (421_580.0, 5_821_710.0),
                    (421_570.0, 5_821_710.0),
                    (421_570.0, 5_821_700.0),
                ],
            },
        ];
        std::fs::write(
            dir.path().join(CONTOURS_RELPATH),
            crate::derive::contours::contours_to_geojson(&lines),
        )
        .unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        let georef = &documents.candidates.georef;
        let contours: Vec<&MapObject> = documents.candidates.parts[0]
            .objects
            .iter()
            .filter(|o| o.tags.iter().any(|(k, _)| k == "freemapper:height"))
            .collect();
        let heights: Vec<&str> = contours
            .iter()
            .flat_map(|o| o.tags.iter().filter(|(k, _)| k == "freemapper:height"))
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(heights, ["63", "63"], "two pieces, no ring in the lake");
        for contour in contours {
            for c in &contour.coords {
                let e = georef.map_to_projected(*c).easting;
                assert!(!(421_560.1..421_589.9).contains(&e), "{e} in the lake");
            }
        }
        // The training PDF paints the same candidates.
        let (map, _) = training_objects(&documents);
        let painted = map
            .iter()
            .filter(|o| o.tags.iter().any(|(k, _)| k == "freemapper:height"))
            .count();
        assert_eq!(painted, 2);
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
            let expected = crate::crs::utm33_forward(LatLon {
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

    /// A file outside the project, the target of a link in a shared project folder.
    #[cfg(unix)]
    fn victim(outside: &Path) -> PathBuf {
        let victim = outside.join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        victim
    }

    #[cfg(unix)]
    fn is_regular_file(path: &Path) -> bool {
        std::fs::symlink_metadata(path)
            .unwrap()
            .file_type()
            .is_file()
    }

    #[cfg(unix)]
    #[test]
    fn run_replaces_a_linked_attribution_instead_of_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        export_starter_project(dir.path(), false).unwrap();
        let victim = victim(outside.path());
        let attribution = dir.path().join("ATTRIBUTION.txt");
        std::fs::remove_file(&attribution).unwrap();
        std::os::unix::fs::symlink(&victim, &attribution).unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        run(dir.path(), &fetcher, &cache, false);
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert!(is_regular_file(&attribution));
        assert!(
            std::fs::read_to_string(&attribution)
                .unwrap()
                .contains("github.com/HenryJobst/freemapper")
        );
    }

    #[cfg(unix)]
    #[test]
    fn forced_run_replaces_a_linked_map_backup_instead_of_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let map = export_starter_project(dir.path(), false).unwrap();
        std::fs::write(&map, "edited in OOM").unwrap();
        let victim = victim(outside.path());
        let backup = map.with_extension("omap.bak");
        std::os::unix::fs::symlink(&victim, &backup).unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        run(dir.path(), &fetcher, &cache, true);
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert!(is_regular_file(&backup));
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "edited in OOM");
    }

    #[cfg(unix)]
    #[test]
    fn forced_ocad_export_replaces_a_linked_backup_instead_of_following_it() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let outside = tempfile::tempdir().unwrap();
        let first = export_ocad(dir.path(), false).unwrap();
        std::fs::write(&first.map, "edited in OCAD").unwrap();
        let victim = victim(outside.path());
        let backup = first.map.with_extension("ocd.bak");
        std::os::unix::fs::symlink(&victim, &backup).unwrap();
        export_ocad(dir.path(), true).unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert!(is_regular_file(&backup));
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "edited in OCAD");
    }

    #[cfg(unix)]
    #[test]
    fn ocad_export_writes_nothing_into_a_linked_ocad_folder() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join(OCAD_DIR)).unwrap();
        let error = export_ocad(dir.path(), false).unwrap_err().to_string();
        assert!(
            error.contains("a link leads out of the project folder"),
            "{error}"
        );
        let written: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
        assert!(written.is_empty(), "{written:?}");
    }

    #[cfg(unix)]
    #[test]
    fn ocad_export_replaces_a_linked_image_copy_instead_of_following_it() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let outside = tempfile::tempdir().unwrap();
        let victim = victim(outside.path());
        let copy = dir.path().join(OCAD_DIR).join("hillshade.png");
        std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&victim, &copy).unwrap();
        export_ocad(dir.path(), false).unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert!(is_regular_file(&copy));
        assert_eq!(
            std::fs::read(&copy).unwrap(),
            std::fs::read(dir.path().join(HILLSHADE_RELPATH)).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_replaces_a_linked_template_instead_of_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let victim = victim(outside.path());
        let template = dir.path().join(HILLSHADE_RELPATH);
        std::fs::create_dir_all(template.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&victim, &template).unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        run(dir.path(), &fetcher, &cache, false);
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        assert!(is_regular_file(&template));
    }

    #[cfg(unix)]
    #[test]
    fn linked_templates_and_data_folders_get_nothing() {
        for folder in ["templates", "data"] {
            let dir = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let cache_dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(cache_dir.path().to_path_buf());
            school_project(dir.path());
            std::os::unix::fs::symlink(outside.path(), dir.path().join(folder)).unwrap();
            let fetcher = MapFetcher::default();
            serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
            let error = run_project(
                dir.path(),
                &fetcher,
                &cache,
                FetchMode::Online,
                false,
                &mut |_| {},
            )
            .unwrap_err()
            .to_string();
            assert!(
                error.contains("a link leads out of the project folder"),
                "{folder}: {error}"
            );
            let written: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
            assert!(written.is_empty(), "{folder}: {written:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn saving_replaces_a_linked_project_file_instead_of_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let project = ProjectFile::load(dir.path()).unwrap();
        let file = dir.path().join(PROJECT_FILE_NAME);
        let victim = outside.path().join("victim.toml");
        std::fs::rename(&file, &victim).unwrap();
        std::os::unix::fs::symlink(&victim, &file).unwrap();
        let before = std::fs::read_to_string(&victim).unwrap();
        project.save(dir.path()).unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), before);
        assert!(is_regular_file(&file));
    }

    #[cfg(unix)]
    #[test]
    fn ocad_export_copies_no_linked_template() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let outside = tempfile::tempdir().unwrap();
        // A valid image, so the map still lists it; it lives outside the project.
        let template = dir.path().join(HILLSHADE_RELPATH);
        let secret = outside.path().join("secret.png");
        std::fs::rename(&template, &secret).unwrap();
        std::os::unix::fs::symlink(&secret, &template).unwrap();
        let outcome = export_ocad(dir.path(), false).unwrap();
        assert_eq!(outcome.warnings.len(), 1, "{:?}", outcome.warnings);
        assert!(
            outcome.warnings[0].starts_with("warning: ")
                && outcome.warnings[0].contains(crate::fsutil::NOT_COPIED),
            "{:?}",
            outcome.warnings
        );
        let ocad = dir.path().join(OCAD_DIR);
        assert!(!ocad.join("hillshade.png").exists());
        assert!(ocad.join("slope.png").exists(), "the other templates stay");
        let map = crate::ocd::read::read(&std::fs::read(&outcome.map).unwrap());
        assert!(
            !map.strings
                .iter()
                .any(|s| s.kind == 8 && s.text.starts_with("hillshade.png"))
        );
    }

    #[test]
    fn ocad_backgrounds_leave_out_a_template_outside_the_project() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("project");
        let ocad = dir.join(OCAD_DIR);
        std::fs::create_dir_all(&ocad).unwrap();
        std::fs::write(root.path().join("secret.png"), "secret").unwrap();
        std::fs::write(root.path().join("secret.pgw"), "1\n0\n0\n-1\n0\n0\n").unwrap();
        let template = TemplateRef {
            kind: TemplateKind::Image,
            relpath: "../secret.png".to_string(),
            georeferenced: true,
            crs_spec: None,
            visible: true,
            opacity: 1.0,
            behind_map: true,
        };
        let (backgrounds, warnings) = copy_backgrounds(&dir, &ocad, &[template]).unwrap();
        assert!(backgrounds.is_empty());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("secret.png"), "{warnings:?}");
        let copied: Vec<_> = std::fs::read_dir(&ocad).unwrap().collect();
        assert!(copied.is_empty(), "{copied:?}");
    }

    #[cfg(unix)]
    #[test]
    fn writers_refuse_a_folder_swapped_for_a_link_during_a_run() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let dir = dir.path();
        school_project(dir);
        // The folders were normal when the run checked them, then became links.
        for folder in ["data", "templates"] {
            std::os::unix::fs::symlink(outside.path(), dir.join(folder)).unwrap();
        }
        let grid = GridSpec::covering(
            &BoundingBox {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 4.0,
                max_n: 4.0,
            },
            1.0,
        );
        let refused = |e: String| {
            assert!(e.contains(crate::fsutil::OUTSIDE_PROJECT), "{e}");
        };
        refused(
            crate::sources::float_tiff::write_float_raster(
                &Raster::filled(grid, 1.0),
                dir,
                &dir.join(TERRAIN_RELPATH),
                "x",
            )
            .unwrap_err()
            .to_string(),
        );
        refused(
            write_gray_png(
                &Raster::filled(grid, Some(1)),
                dir,
                &dir.join(HILLSHADE_RELPATH),
            )
            .unwrap_err()
            .to_string(),
        );
        refused(
            write_ndsm_png(&Raster::filled(grid, 1.0), dir, &dir.join(NDSM_RELPATH))
                .unwrap_err()
                .to_string(),
        );
        refused(
            write_inside(dir, &dir.join(BUILDINGS_RELPATH), b"{}")
                .unwrap_err()
                .to_string(),
        );
        let written: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
        assert!(written.is_empty(), "{written:?}");
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_map_is_not_copied_into_its_backup() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let map = map_path(dir.path());
        let secret = victim(outside.path());
        std::os::unix::fs::symlink(&secret, &map).unwrap();
        let error = export_starter_project(dir.path(), true)
            .unwrap_err()
            .to_string();
        assert!(
            error.ends_with(&format!("{}; {LINKED_MAP_HINT}", crate::fsutil::NOT_COPIED)),
            "{error}"
        );
        assert!(!map.with_extension("omap.bak").exists());
        assert_eq!(std::fs::read_to_string(&secret).unwrap(), "precious");
    }
}
