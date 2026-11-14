//! Project folder and project file (`freemapper.toml`).

use crate::note::{Arg, Note, note};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::credit::{AlkisUse, CreditLine, UsedSources, lines_with_licences};
use freemapper_adapter_api::records::AdapterId;
use serde::{Deserialize, Serialize};

use crate::adapters::{RegionAdapter, registry};
use crate::area::GeoBox;
use crate::area::{AreaError, MapArea};
use crate::build::buildings::building_objects;
use crate::build::candidates::Rules;
use crate::build::canopy::{CANOPY_LAYER, CanopyInput, canopy_objects};
use crate::build::content::ContentSettings;
use crate::build::open_land::{BASE_PART, walkable_base};
use crate::build::peripheral::{PaperRect, PeripheralInfo, info_block, is_scale_bar};
use crate::build::points::{GENERALIZATION_SOURCE, HIDDEN_KEY};
use crate::build::source_block;
use crate::build::stairs::{
    NAS_STAIR_LAYER, STEP_EDGE_LAYER, SURVEY_STAIR_LAYER, has_nas_stairs_without_edge, has_stairs,
};
use crate::crs::{BoundingBox, Crs};
use crate::derive::contours::{
    contour_interval_for, contour_lines, contours_from_geojson, contours_to_geojson,
    gaussian_smooth, generalize, terrain_inside,
};
use crate::derive::landforms::{
    height_jumps, knolls_and_depressions, landforms_from_geojson, landforms_to_geojson,
};
use crate::derive::ndsm::{fill_from_dsm, rasterize, write_ndsm_png};
use crate::derive::orthophoto::{build_orthophoto, build_orthophoto_from_image};
use crate::derive::relief::{RELIEF_SPAN_M, local_relief, relief_shade};
use crate::derive::terrain::{
    hillshade, mosaic_dtm, read_float_raster, slope_deg, slope_shade, write_gray_png,
};
use crate::derive::vegetation::{
    BuildingMask, CELL_M as VEGETATION_CELL_M, CellCounts, vegetation_areas, vegetation_to_geojson,
};
use crate::derive::{DeriveError, MAX_RASTER_SIDE_PX};
use crate::fetch::Product;
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher, write_inside};
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::SymbolScale;
use crate::ocd::convert::{Background, WorldFile, map_to_ocd};
use crate::omap::{
    MapDocument, MapObject, MapPart, OmapError, SymbolSet, TemplateKind, TemplateRef, XmlFormat,
    write_map,
};
use crate::raster::{GridSpec, Raster};
use crate::sources::alkis::{FetchArea, LandUse, fetch_land_use_in, parse_land_use};
use crate::sources::alkis::{fetch_buildings_in, parse_building_features, parse_buildings};
use crate::sources::alkis_nas;
use crate::sources::osm::{
    OsmFeature, cached_extract_date, extracts_for, features_from_geojson, features_to_geojson,
    fetch_extract, read_extract,
};
use crate::sources::region::{
    no_buildings_warnings, overlaps_in, raster_provider, raster_region_in, region_of_object_id,
    terrain_source,
};
use crate::sources::trees;
use crate::space::{Part, SpacePlan};
use crate::tiles::TileId;
use freemapper_adapters_de::berlin;

mod export;
mod extras;
pub use export::*;

pub const PROJECT_FILE_NAME: &str = freemapper_contract::tags::SIDECAR_FILE;

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
    #[error("invalid contour interval {0} m; it must be more than 0")]
    InvalidContourInterval(f64),
    #[error("scale 1:{scale} is not allowed for {kind} maps; use {allowed}{hint}")]
    ScaleNotAllowed {
        kind: &'static str,
        scale: u32,
        /// The allowed scales as `1:4000, 1:5000`.
        allowed: String,
        /// Starts with `. ` when present, or is empty.
        hint: &'static str,
    },
    #[error("invalid project file: {0}")]
    Toml(String),
    /// The project file names a provider or a region this program has no adapter for: it was
    /// made with adapters that are not installed here.
    #[error(
        "the project file names the adapter \"{0}\", which this freemapper does not have; a newer freemapper may bring it (`freemapper update`), or it comes from your adapter folder (`freemapper adapters list` shows it) or, with --adapters-from-project on the command line, from the folder `adapters` of the project (only for a project you trust)"
    )]
    UnknownAdapter(String),
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
    /// The caller asked to stop (see [`run_project_cancellable`]); what was written before is
    /// whole, and the next run continues.
    #[error("cancelled")]
    Cancelled,
    /// What is wrong with a layer or a template, as a message of the catalog.
    #[error("{0}")]
    Layer(Note),
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
        "the map area lies outside the supported states ({}); freemapper has data sources for those only so far",
        crate::adapters::region_names(false).join(", ")
    )]
    OutsideCoverage,
    #[error("{}: {message}; run `freemapper fetch` again", path.display())]
    StoredBuildings { path: PathBuf, message: String },
    /// A part the run needs does not fit on its volume.
    #[error("{0}")]
    NotEnoughSpace(crate::space::Shortfall),
}

/// Kind of orienteering map; selects norm, symbol set and default scale.
///
/// These are the kinds freemapper writes. The suite's definition is
/// [`freemapper_contract::MapKind`] (ADR 0095): names and scales come from there, and the code
/// here branches on the kinds it can build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapKind {
    Schoolyard,
    Sprint,
    /// Forest map after ISOM 2017-2 (spec 2026-11-13).
    Forest,
}

impl MapKind {
    /// Every kind freemapper writes.
    pub const ALL: [MapKind; 3] = [MapKind::Schoolyard, MapKind::Sprint, MapKind::Forest];

    /// The same kind in the words of the suite contract.
    pub fn contract(self) -> freemapper_contract::MapKind {
        match self {
            Self::Schoolyard => freemapper_contract::MapKind::Schoolyard,
            Self::Sprint => freemapper_contract::MapKind::Sprint,
            Self::Forest => freemapper_contract::MapKind::Forest,
        }
    }

    /// The kind with this name, as `freemapper.toml` and the command line write it.
    pub fn from_name(name: &str) -> Option<MapKind> {
        MapKind::ALL.into_iter().find(|kind| kind.name() == name)
    }

    pub fn default_scale(self) -> u32 {
        self.contract().base_scale()
    }

    /// The map scales freemapper writes for this kind, which freemapper-print accepts (its
    /// `iof-print.toml` tables). Sprint maps are drawn at 1:4000 (100 % symbols) or 1:5000;
    /// 1:3000 needs 133 % symbols, which only freemapper-print applies (ADR 0085). Forest maps
    /// are drawn at 1:15000, 1:10000 or 1:7500 with their own enlargement
    /// ([`MapKind::symbol_enlargement`]).
    pub fn allowed_scales(self) -> &'static [u32] {
        self.contract().drawing_scales()
    }

    /// The lower-case name, as `freemapper.toml` and the command line write it.
    pub fn name(self) -> &'static str {
        self.contract().name()
    }

    fn allowed_text(self) -> String {
        self.allowed_scales()
            .iter()
            .map(|s| format!("1:{s}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Refuses a scale freemapper-print would refuse later.
    pub fn check_scale(self, scale: u32) -> Result<(), ProjectError> {
        if scale == 0 {
            return Err(ProjectError::InvalidScale(scale));
        }
        if self.allowed_scales().contains(&scale) {
            return Ok(());
        }
        let hint = if self == Self::Sprint && scale == 3000 {
            ". Draw at 1:4000, freemapper-print enlarges to 1:3000 when printing"
        } else {
            ""
        };
        Err(ProjectError::ScaleNotAllowed {
            kind: self.name(),
            scale,
            allowed: self.allowed_text(),
            hint,
        })
    }

    /// A note for an allowed scale that has a catch, shown after `init`.
    pub fn scale_note(self, scale: u32) -> Option<Note> {
        (self == Self::Sprint && scale == 5000).then(|| note!("note-scale-outside-norm"))
    }

    pub fn default_buffer_m(self) -> f64 {
        match self {
            Self::Schoolyard => 20.0,
            Self::Sprint => 50.0,
            Self::Forest => 100.0,
        }
    }

    /// The symbol set of the map kind at this scale with the sheet symbols; the sprint map also
    /// gets the walkable area (belaufbare Fläche, ADR 0069).
    pub fn symbol_set(self, scale: u32) -> SymbolSet {
        match self {
            Self::Schoolyard => SymbolSet::school_2019().with_sheet_symbols(),
            Self::Sprint => SymbolSet::issprom_2019()
                .with_sheet_symbols()
                .with_walkable_area(),
            Self::Forest => {
                SymbolSet::isom_2017_2(self.symbol_enlargement(scale)).with_sheet_symbols()
            }
        }
    }

    /// Enlargement of the norm's symbols at this scale: the school set draws ISSprOM 2019 at
    /// 125 %, a forest map the 1:15000 set of ISOM 2017-2 at 150 % for 1:10000 and 200 % for
    /// 1:7500.
    pub fn symbol_enlargement(self, scale: u32) -> f64 {
        match self {
            Self::Schoolyard => crate::omap::symbol_set::SCHOOL_SYMBOL_ENLARGEMENT,
            Self::Sprint => 1.0,
            Self::Forest => FOREST_BASE_SCALE / f64::from(scale.max(1)),
        }
    }

    /// Slope at which the slope template turns black: school grounds are flat, so their ramp is
    /// steeper to show small banks and ramps.
    pub fn slope_black_deg(self) -> f32 {
        match self {
            Self::Schoolyard => 15.0,
            Self::Sprint | Self::Forest => 30.0,
        }
    }

    /// Gaussian smoothing of the DTM before tracing contours, in metres.
    pub fn contour_sigma_m(self) -> f64 {
        match self {
            Self::Schoolyard => 1.5,
            Self::Sprint => 2.0,
            Self::Forest => 3.0,
        }
    }

    /// Width of the mean the local relief template is measured against (`derive/relief.rs`).
    pub fn relief_sigma_m(self) -> f64 {
        match self {
            Self::Schoolyard => 3.0,
            Self::Sprint => 5.0,
            Self::Forest => 8.0,
        }
    }

    /// Smallest height of a knoll, depth of a depression or drop of a bank candidate
    /// (`derive/landforms.rs`).
    pub fn landform_min_height_m(self) -> f64 {
        match self {
            Self::Schoolyard => 0.5,
            Self::Sprint | Self::Forest => 1.0,
        }
    }

    /// Contour interval in metres (spec §4.4); a forest map in flat land halves it
    /// (`derive::contours::contour_interval_for`).
    pub fn contour_interval_m(self) -> f64 {
        match self {
            Self::Schoolyard => 1.0,
            Self::Sprint => 2.0,
            Self::Forest => 5.0,
        }
    }
}

/// The scale the ISOM 2017-2 symbols are drawn for at 100 %.
const FOREST_BASE_SCALE: f64 = 15000.0;

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
    pub provider: Option<AdapterId>,
    /// Map title for the peripheral information; the folder name when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Contour interval in metres; without it `derive` picks the one of the map kind and the
    /// relief (`derive::contours::contour_interval_for`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contour_interval_m: Option<f64>,
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
    /// The point-cloud tiles of the last `fetch`, for the vegetation candidates (ADR 0105);
    /// empty where the state publishes none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub point_clouds: Vec<PointCloudRecord>,
    /// What goes straight onto the map and what stays a candidate (ADR 0100); the `balanced`
    /// profile when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<ContentSettings>,
    /// The contour interval of the last `derive` when it is not the map kind's (a contract key,
    /// `[contours] interval_m`: freemapper-print names it); absent otherwise (ADR 0116).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contours: Option<ContourRecord>,
}

/// The `[contours]` record of `freemapper.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ContourRecord {
    /// The interval the contours were traced at, in metres.
    pub interval_m: f64,
}

// The tile and raster records live in `freemapper-adapter-api` (ADR 0082); the old paths stay.
use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::pointcloud::{PointCloudBackend, PointCloudRecord};
use freemapper_adapter_api::project::{RasterProject, RasterRecords};
use freemapper_adapter_api::rasters::{
    DeriveEnv, HeightInput, OrthophotoInput, RasterArea, RasterBackend, StoredRasters,
};
use freemapper_adapters_de::berlin::BERLIN_REGISTERS;
// The steps of a raster fetch that name no state, and the files they write.
use freemapper_adapter_api::records::on_grid;
pub use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
#[cfg(test)]
use freemapper_sources::raster_fetch::UNDATED_ORTHOPHOTO_MAX_AGE_DAYS;
pub(crate) use freemapper_sources::raster_fetch::{
    SURFACE_RELPATH, TERRAIN_RELPATH, WMS_ORTHOPHOTO_RELPATH,
};

/// The grid the area of the OSM record is stored on, in degrees (about a millimetre).
const STORED_DEGREE: f64 = 1e-8;

/// `area` as the OSM record stores it.
fn stored_geo_box(area: &GeoBox) -> [f64; 4] {
    [area.min_lat, area.min_lon, area.max_lat, area.max_lon]
        .map(|value| on_grid(value, STORED_DEGREE))
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
        min_lat <= area.min_lat + STORED_DEGREE
            && min_lon <= area.min_lon + STORED_DEGREE
            && max_lat >= area.max_lat - STORED_DEGREE
            && max_lon >= area.max_lon - STORED_DEGREE
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
    pub regions: Vec<AdapterId>,
}

impl AlkisRecord {
    fn covers(&self, bbox: &BoundingBox) -> bool {
        let [min_e, min_n, max_e, max_n] = self.bbox;
        min_e <= bbox.min_e && min_n <= bbox.min_n && max_e >= bbox.max_e && max_n >= bbox.max_n
    }
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
        progress: &mut dyn FnMut(Note),
    ) -> Result<Self, ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path, e))?;
        let mut project: Self =
            toml::from_str(&text).map_err(|e| ProjectError::Toml(e.to_string()))?;
        if let Some(interval) = project.contour_interval_m {
            check_contour_interval(interval)?;
        }
        project.check_adapters()?;
        project.drop_unknown_names(progress);
        if project
            .map_kind
            .check_scale(project.scale_denominator)
            .is_err()
        {
            // Loadable all the same: the project may predate the table, and print decides.
            progress(note!(
                "warning-scale-not-allowed",
                scale = project.scale_denominator.to_string(),
                kind = project.map_kind.name(),
                allowed = project.map_kind.allowed_text()
            ));
        }
        Ok(project)
    }

    /// Fails when the file names a provider or a region the registry has no adapter for.
    fn check_adapters(&self) -> Result<(), ProjectError> {
        let known_provider = |id: &AdapterId| {
            registry()
                .adapters()
                .any(|adapter| adapter.info().provider == *id)
        };
        if let Some(provider) = self.provider.as_ref().filter(|id| !known_provider(id)) {
            return Err(ProjectError::UnknownAdapter(provider.to_string()));
        }
        let records = [&self.buildings, &self.land_use]
            .into_iter()
            .flatten()
            .chain(self.berlin_layers.values())
            .chain(self.nas_layers.values())
            .chain(self.tree_layers.values());
        for region in records.flat_map(|record| &record.regions) {
            if registry().get(region.as_str()).is_none() {
                return Err(ProjectError::UnknownAdapter(region.to_string()));
            }
        }
        Ok(())
    }

    /// Leaves out the Berlin and NAS layer records whose key is no plain file name or no layer
    /// of the table, each with a warning. The key is shown escaped, as it may hold control
    /// characters.
    fn drop_unknown_names(&mut self, progress: &mut dyn FnMut(Note)) {
        let mut keep = |what: &str, key: &str, known: bool| {
            // The catalogs word the reason by this key.
            let reason = if !freemapper_fs::is_plain_name(key) {
                "name"
            } else if !known {
                "unknown"
            } else {
                return true;
            };
            progress(note!(
                "warning-layer-entry-ignored",
                what = what,
                key = format!("{key:?}"),
                reason = reason
            ));
            false
        };
        let registers = registers();
        self.berlin_layers.retain(|key, _| {
            keep(
                "Berlin",
                key,
                registers.as_ref().is_some_and(|family| family.has(key)),
            )
        });
        self.nas_layers
            .retain(|key, _| keep("NAS", key, extras::NAS.has(key)));
        self.tree_layers
            .retain(|key, _| keep("tree", key, extras::TREES.has(key)));
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

    /// The stored map area, checked like a drawn one ([`MapArea::validate`]): a hand-edited or
    /// crafted project file with an empty, crossing or oversized outline is an error, not a
    /// panic further down.
    pub fn area(&self) -> Result<MapArea, ProjectError> {
        let area = MapArea::from_geojson(&self.map_area.to_string())?;
        area.validate()?;
        Ok(area)
    }

    /// Map area plus buffer in the map's CRS.
    pub fn bbox(&self) -> Result<BoundingBox, ProjectError> {
        let area = self.area()?;
        Ok(area.projected_bbox_in(self.crs()).buffered(self.buffer_m))
    }

    /// Map area plus buffer in EPSG:25833, where the region boundaries are.
    pub fn region_bbox(&self) -> Result<BoundingBox, ProjectError> {
        let area = self.area()?;
        Ok(area.projected_bbox().buffered(self.buffer_m))
    }

    /// Whether the content settings ask for the vegetation areas, and so for the point clouds.
    pub fn wants_vegetation(&self) -> bool {
        self.content
            .as_ref()
            .is_some_and(ContentSettings::wants_vegetation)
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
                freemapper_fs::remove_inside(dir, &file);
            }
        }
        self.tiles.clear();
        self.ndsm_tiles.clear();
        self.point_clouds.clear();
        self.buildings = None;
        self.land_use = None;
        if let Some(registers) = registers() {
            for key in self.berlin_layers.keys() {
                extras::remove(dir, &registers, key);
            }
        }
        self.berlin_layers.clear();
        for key in self.nas_layers.keys() {
            extras::remove(dir, &extras::NAS, key);
        }
        self.nas_layers.clear();
        for key in self.tree_layers.keys() {
            extras::remove(dir, &extras::TREES, key);
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
    init_project_with_content(
        dir,
        area,
        map_kind,
        scale_denominator,
        declination_date,
        title,
        None,
    )
}

/// [`init_project`] with the content settings (ADR 0100) in the same write.
pub fn init_project_with_content(
    dir: &Path,
    area: &MapArea,
    map_kind: MapKind,
    scale_denominator: Option<u32>,
    declination_date: NaiveDate,
    title: Option<String>,
    content: Option<ContentSettings>,
) -> Result<ProjectFile, ProjectError> {
    let scale_denominator = scale_denominator.unwrap_or(map_kind.default_scale());
    map_kind.check_scale(scale_denominator)?;
    let path = dir.join(PROJECT_FILE_NAME);
    if path.exists() {
        return Err(ProjectError::AlreadyExists(path));
    }
    area.validate()?;
    let region_bbox = area.projected_bbox().buffered(map_kind.default_buffer_m());
    check_area(&region_bbox)?;
    let crs = map_crs(&region_bbox);
    let provider = raster_provider(&region_bbox);
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
        contour_interval_m: None,
        contours: None,
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
        point_clouds: Vec::new(),
        content,
    };
    // Always write a title: the PDF, freemapper-print and freemapper-courseplan then read the
    // same one instead of each deriving its own from the folder name.
    let mut project = project;
    project.title = Some(project.display_title(dir));
    project.save(dir)?;
    Ok(project)
}

use crate::fetch::tile_names;

/// Partial template files older than this are left over from interrupted runs.
const STALE_TEMPLATE_PART_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// Largest side of map area plus buffer: [`MAX_RASTER_SIDE_PX`] orthophoto pixels of 0.2 m.
pub const MAX_AREA_SIDE_M: f64 = MAX_RASTER_SIDE_PX as f64 * 0.2;

/// Products fetched for every project, in this order.
pub(crate) const ORTHOPHOTO_RELPATH: &str = "templates/orthophoto.jpg";
pub(crate) const HILLSHADE_RELPATH: &str = "templates/hillshade.png";
pub(crate) const SLOPE_RELPATH: &str = "templates/slope.png";
pub(crate) const RELIEF_RELPATH: &str = "templates/relief.png";
pub(crate) const NDSM_RELPATH: &str = "templates/ndsm.png";
/// The Berlin K5 map (1:5000) as fetched; a template the mapper draws on, optional.
pub(crate) const K5_RELPATH: &str = "templates/k5.png";
/// The key of the raster template whose record the project file keeps in `k5` and whose file
/// is [`K5_RELPATH`]: the one template there is. A second one needs a field of its own.
pub(crate) const K5_TEMPLATE: &str = "k5";
/// The terrain mosaic of a tile provider (LGB, GeoSN) cut to the map area plus buffer, with its
/// world file. `derive` writes it only where the extra Berlin layers have stair areas: the export
/// reads their walking direction from it, offline and without the tile cache.
pub(crate) const TERRAIN_MOSAIC_RELPATH: &str = "data/terrain.tif";
/// Contour lines from `derive` (GeoJSON in the map's CRS), input of the export.
const CONTOURS_RELPATH: &str = "data/contours.geojson";
/// Knolls, depressions, banks and cliffs from `derive` (GeoJSON in the map's CRS), input of the
/// export.
const LANDFORMS_RELPATH: &str = "data/landforms.geojson";
/// The vegetation areas from the point clouds (ADR 0105), in the map's CRS.
const VEGETATION_RELPATH: &str = "data/vegetation.geojson";
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

/// A project after a new map area was saved, with why the data fetched for the old area was
/// dropped, if it was.
#[derive(Debug)]
pub struct AreaSaved {
    pub project: ProjectFile,
    /// The new area lies in another UTM zone.
    pub zone_changed: bool,
    /// Another provider serves the new area, in the same zone or not.
    pub provider_changed: bool,
}

impl AreaSaved {
    /// Whether the fetched data was dropped: the next `fetch` loads it again for the new area.
    pub fn dropped_fetched(&self) -> bool {
        self.zone_changed || self.provider_changed
    }
}

/// Sets the map area of the project in `dir` (from the area selection page), keeping every other
/// setting. Data fetched for the old area is fetched again for the new one by the next `fetch`;
/// the result says whether it was dropped and why.
pub fn set_map_area(dir: &Path, area: &MapArea) -> Result<AreaSaved, ProjectError> {
    set_map_area_with_content(dir, area, None)
}

/// [`set_map_area`] that also stores `content` (ADR 0100) in the same write; `None` leaves the
/// saved settings as they are.
pub fn set_map_area_with_content(
    dir: &Path,
    area: &MapArea,
    content: Option<ContentSettings>,
) -> Result<AreaSaved, ProjectError> {
    area.validate()?;
    let mut project = ProjectFile::load(dir)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    // Projects saved before the provider was recorded: that of the old area.
    let old_provider = project
        .provider
        .clone()
        .or_else(|| project.region_bbox().ok().and_then(|b| raster_provider(&b)));
    project.map_area = area.to_geojson_value();
    project.map_area_source_id = area.source_id.clone();
    let region_bbox = project.region_bbox()?;
    check_area(&region_bbox)?;
    // Data fetched in the other UTM zone cannot be compared with the new area: fetch it again.
    let crs = map_crs(&region_bbox);
    check_size(&area.projected_bbox_in(crs).buffered(project.buffer_m))?;
    // Another provider (in the same zone) serves other rasters: fetch them again too.
    let provider = raster_provider(&region_bbox);
    let provider_changed = old_provider.is_some() && old_provider != provider;
    let zone_changed = crs != project.crs();
    if zone_changed || provider_changed {
        project.crs = crs.epsg();
        project.drop_fetched(dir);
    }
    project.provider = provider;
    if content.is_some() {
        project.content = content;
    }
    project.save(dir)?;
    Ok(AreaSaved {
        project,
        zone_changed,
        provider_changed,
    })
}

/// Sets what goes straight onto the map and what stays a candidate (ADR 0100) for the project in
/// `dir`, keeping every other setting; `None` takes the project back to the default. The map is
/// written by the next `export`.
pub fn set_content(
    dir: &Path,
    content: Option<ContentSettings>,
) -> Result<ProjectFile, ProjectError> {
    let mut project = ProjectFile::load(dir)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    project.content = content;
    project.save(dir)?;
    Ok(project)
}

/// Refuses a contour interval that is no positive number.
pub fn check_contour_interval(interval_m: f64) -> Result<(), ProjectError> {
    if interval_m.is_finite() && interval_m > 0.0 {
        Ok(())
    } else {
        Err(ProjectError::InvalidContourInterval(interval_m))
    }
}

/// A contour interval as a person writes it: `auto` (the choice of `derive`, `None`) or metres
/// with a dot or a comma, such as `2,5`.
pub fn parse_contour_interval(text: &str) -> Result<Option<f64>, ProjectError> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    let interval: f64 = text.replace(',', ".").parse().unwrap_or(f64::NAN);
    check_contour_interval(interval)?;
    Ok(Some(interval))
}

/// Sets the contour interval of the project in `dir`; `None` gives the choice back to `derive`
/// (the map kind's interval, 2.5 m in a flat forest). The next `derive` traces at it.
pub fn set_contour_interval(
    dir: &Path,
    interval_m: Option<f64>,
) -> Result<ProjectFile, ProjectError> {
    if let Some(interval) = interval_m {
        check_contour_interval(interval)?;
    }
    let mut project = ProjectFile::load(dir)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    if project.contour_interval_m != interval_m {
        project.contour_interval_m = interval_m;
        project.save(dir)?;
    }
    Ok(project)
}

/// The CRS of a map of `region_bbox` (EPSG:25833): that of its raster region.
pub(crate) fn map_crs(region_bbox: &BoundingBox) -> Crs {
    raster_region_in(registry(), region_bbox).map_or(Crs::Utm33, |adapter| adapter.info().crs)
}

fn default_crs() -> u32 {
    Crs::Utm33.epsg()
}

/// Refuses a map area plus buffer outside every supported region or larger than the templates
/// can cover.
pub(crate) fn check_area(bbox: &BoundingBox) -> Result<(), ProjectError> {
    if raster_region_in(registry(), bbox).is_none() {
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
    progress: &mut dyn FnMut(Note),
) -> Result<ProjectFile, ProjectError> {
    fetch_project_cancellable(dir, fetcher, cache, mode, progress, &|| false)
}

/// [`fetch_project`] without the OSM extract: what the services of the adapters deliver for
/// the map area. The check of an adapter asks this; an extract has 50 to 500 MB and is the
/// same for every adapter of its region.
pub fn fetch_project_sources(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
) -> Result<ProjectFile, ProjectError> {
    fetch_project_parts(dir, fetcher, cache, mode, progress, &|| false, false)
}

/// A project file with its folder, as the raster fetch of an adapter sees it.
struct ProjectDir<'a> {
    dir: &'a Path,
    project: &'a mut ProjectFile,
    /// The map area plus buffer in EPSG:25833.
    region_bbox: BoundingBox,
}

impl RasterProject for ProjectDir<'_> {
    fn records(&mut self) -> RasterRecords<'_> {
        RasterRecords {
            tiles: &mut self.project.tiles,
            ndsm_tiles: &mut self.project.ndsm_tiles,
            orthophoto: &mut self.project.orthophoto,
            terrain: &mut self.project.terrain,
            surface: &mut self.project.surface,
        }
    }

    fn save(&mut self) -> Result<(), AdapterError> {
        // A failed write comes as a fetch error (`fetch::write_inside`). Whatever the save
        // fails with reaches the caller as the same error: nothing is lost, nothing panics.
        self.project.save(self.dir).map_err(|error| match error {
            ProjectError::Fetch(e) => AdapterError::Fetch(e),
            ProjectError::Io(path, e) => AdapterError::Io(path, e),
            ProjectError::Toml(message) => AdapterError::ProjectFile(message),
            other => AdapterError::Engine(Box::new(other)),
        })
    }

    fn dir(&self) -> &Path {
        self.dir
    }

    fn region_bbox(&self) -> BoundingBox {
        self.region_bbox
    }

    fn crs(&self) -> Crs {
        self.project.crs()
    }
}

/// Each error of a raster fetch is the engine's error it stands for, so the caller words it as
/// before.
impl From<AdapterError> for ProjectError {
    fn from(error: AdapterError) -> Self {
        match error {
            AdapterError::Fetch(e) => ProjectError::Fetch(e),
            AdapterError::Io(path, e) => ProjectError::Io(path, e),
            AdapterError::ProjectFile(message) => ProjectError::Toml(message),
            AdapterError::OrthophotoNotFetched => ProjectError::OrthophotoNotFetched,
            AdapterError::TerrainNotFetched => ProjectError::TerrainNotFetched,
            // The engine's own error, as it was; an error of another kind keeps its words.
            AdapterError::Engine(e) => match e.downcast::<ProjectError>() {
                Ok(own) => *own,
                Err(other) => ProjectError::Toml(other.to_string()),
            },
        }
    }
}

/// Stops with [`ProjectError::Cancelled`] when `cancelled` says so.
fn stop_if(cancelled: &dyn Fn() -> bool) -> Result<(), ProjectError> {
    if cancelled() {
        Err(ProjectError::Cancelled)
    } else {
        Ok(())
    }
}

/// [`fetch_project`] that asks `cancelled` before every request and after every part, and then
/// stops with [`ProjectError::Cancelled`]. Files are written whole (partial file, then rename)
/// and the project file only records parts that finished, so the next fetch continues.
pub fn fetch_project_cancellable(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
    cancelled: &dyn Fn() -> bool,
) -> Result<ProjectFile, ProjectError> {
    let guarded = crate::fetch::CancelCheck {
        inner: fetcher,
        cancelled,
    };
    // A failure after the stop was asked for is the stop (a refused request surfaces as many
    // kinds of error).
    fetch_project_parts(dir, &guarded, cache, mode, progress, cancelled, true).map_err(|e| {
        if cancelled() {
            ProjectError::Cancelled
        } else {
            e
        }
    })
}

/// `osm`: whether the OSM extract is fetched and read as well.
fn fetch_project_parts(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    progress: &mut dyn FnMut(Note),
    cancelled: &dyn Fn() -> bool,
    osm: bool,
) -> Result<ProjectFile, ProjectError> {
    stop_if(cancelled)?;
    let mut project = ProjectFile::load_reporting(dir, progress)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    let region_bbox = project.region_bbox()?;
    check_area(&region_bbox)?;
    // The stored CRS may be stale (written by hand, or by a version that only knew zone 33).
    let crs = map_crs(&region_bbox);
    if crs != project.crs() {
        progress(note!(
            "warning-other-zone",
            zone = if crs == Crs::Utm32 { "32" } else { "33" },
            epsg = crs.epsg().to_string()
        ));
        project.crs = crs.epsg();
        project.drop_fetched(dir);
        project.save(dir)?;
    }
    let provider = raster_provider(&region_bbox);
    if project.provider.is_some() && project.provider != provider {
        progress(note!("warning-other-state"));
        project.drop_fetched(dir);
        project.provider = provider.clone();
        project.save(dir)?;
    }
    project.provider = provider;
    let bbox = project.bbox()?;
    check_size(&bbox)?;
    if crate::sources::region::reaches_beyond_coverage(&region_bbox) {
        progress(note!("warning-beyond-coverage"));
    }
    // Offline nothing is downloaded, so nothing needs room.
    let plan = match mode {
        FetchMode::Online => check_space(dir, cache, &project, &region_bbox, &bbox, progress)?,
        FetchMode::Offline => None,
    };
    let skips = |part: Part| plan.as_ref().is_some_and(|plan| !plan.fits(part));
    let skip_surface = skips(Part::Surface);
    {
        let mut env = AdapterEnv {
            fetcher,
            regions: registry(),
            cache,
            mode,
            today: crate::clock::today(),
            skip_surface,
            progress: &mut *progress,
        };
        let (env, p) = (
            &mut env,
            &mut ProjectDir {
                dir,
                project: &mut project,
                region_bbox,
            },
        );
        raster_backend(&region_bbox)?.fetch(env, p, &bbox)?;
    }
    stop_if(cancelled)?;
    project.save(dir)?;
    let area = FetchArea {
        region_bbox: project.region_bbox()?,
        crs: project.crs(),
        bbox: project.bbox()?,
    };
    let previous = project.buildings.as_ref();
    let buildings = fetch_buildings_into(dir, fetcher, cache, &area, mode, previous, progress)?;
    stop_if(cancelled)?;
    if let Some(record) = buildings {
        project.buildings = Some(record);
        project.save(dir)?;
    }
    let land_use = fetch_land_use_into(
        dir,
        fetcher,
        cache,
        &area,
        mode,
        project.land_use.as_ref(),
        progress,
    );
    stop_if(cancelled)?;
    if let Some(record) = land_use {
        project.land_use = record;
        project.save(dir)?;
    }
    let berlin_layers = fetch_berlin_layers_into(
        dir,
        fetcher,
        &area.region_bbox,
        project.map_kind,
        mode,
        skips(Part::BerlinLayers),
        &project.berlin_layers,
        progress,
    );
    stop_if(cancelled)?;
    project.berlin_layers = berlin_layers;
    project.save(dir)?;
    let nas_layers = fetch_nas_layers_into(
        dir,
        fetcher,
        &area,
        project.map_kind,
        mode,
        skips(Part::StateLayers),
        &project.nas_layers,
        progress,
    );
    stop_if(cancelled)?;
    project.nas_layers = nas_layers;
    project.save(dir)?;
    let tree_layers = fetch_tree_layers_into(
        dir,
        fetcher,
        &area,
        mode,
        skips(Part::StateLayers) || !fetches_tree_registers(project.map_kind),
        &project.tree_layers,
        progress,
    );
    stop_if(cancelled)?;
    project.tree_layers = tree_layers;
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
    stop_if(cancelled)?;
    project.save(dir)?;
    fetch_point_clouds_into(
        fetcher,
        cache,
        mode,
        skips(Part::PointCloud),
        &mut project,
        &area,
        progress,
    );
    stop_if(cancelled)?;
    project.save(dir)?;
    if !osm {
        return Ok(project);
    }
    let skip_update = skips(Part::OsmUpdate);
    let osm = fetch_osm_into(dir, fetcher, cache, mode, skip_update, &project, progress)?;
    stop_if(cancelled)?;
    project.osm = Some(osm);
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
    let registers = registers();
    let berlin_layers = !project.berlin_layers.is_empty()
        && project.berlin_layers.iter().all(|(key, record)| {
            record.covers(bbox)
                && registers
                    .as_ref()
                    .is_some_and(|family| extras::is_stored(dir, family, key))
        });
    let nas_layers = !project.nas_layers.is_empty()
        && project
            .nas_layers
            .iter()
            .all(|(key, record)| record.covers(bbox) && extras::is_stored(dir, &extras::NAS, key));
    let tree_layers = !project.tree_layers.is_empty()
        && project.tree_layers.iter().all(|(key, record)| {
            record.covers(bbox) && extras::is_stored(dir, &extras::TREES, key)
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
    }
    // Counted from zero: the sum of nothing is -0.0, and an estimate with nothing left to
    // download says 0 MB.
    estimate.total_megabytes = estimate
        .items
        .iter()
        .filter(|i| !i.cached)
        .fold(0.0, |total, i| total + i.megabytes);
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
    progress: &mut dyn FnMut(Note),
) -> Result<Option<SpacePlan>, ProjectError> {
    let today = crate::clock::today();
    let alkis_cached = dir.join(BUILDINGS_RELPATH).exists() && dir.join(LAND_USE_RELPATH).exists();
    let mut estimate = crate::layers::estimate_area(
        region_bbox,
        bbox,
        cache,
        today,
        alkis_cached,
        project.wants_vegetation(),
    );
    mark_stored(&mut estimate, dir, project, bbox);
    let plan = match crate::space::plan(&estimate, cache.root(), dir, cache.disk_space()) {
        Ok(plan) => plan,
        Err((path, e)) => {
            progress(note!(
                "warning-disk-space-unreadable",
                path = path.display().to_string(),
                error = Arg::error(e)
            ));
            return Ok(None);
        }
    };
    if let Some(missing) = plan.missing() {
        return Err(ProjectError::NotEnoughSpace(missing.clone()));
    }
    for skipped in plan.skipped() {
        progress(note!("warning-error", error = Arg::error(skipped.clone())));
    }
    Ok(Some(plan))
}

/// The point-cloud backend of `project`, a map of `region_bbox`, as [`raster_backend`] finds
/// the region: `None` where the state publishes no point clouds, where the content settings do
/// not ask for the vegetation, and where the tiles are in another CRS than the map.
fn point_cloud_backend(
    project: &ProjectFile,
    region_bbox: &BoundingBox,
) -> Option<&'static dyn PointCloudBackend> {
    point_cloud_source(project, region_bbox).map(|(_, backend)| backend)
}

/// The classes the provider of the project's point clouds says are no vegetation
/// ([`PointCloudInfo::left_out`]); none without point clouds.
///
/// [`PointCloudInfo::left_out`]: freemapper_adapter_api::pointcloud::PointCloudInfo::left_out
fn left_out_classes(project: &ProjectFile, region_bbox: &BoundingBox) -> &'static [u8] {
    point_cloud_backend(project, region_bbox)
        .map_or(&[], |backend| backend.info().left_out.as_slice())
}

/// [`point_cloud_backend`] with the adapter it belongs to, whose hosts its requests are held to.
fn point_cloud_source(
    project: &ProjectFile,
    region_bbox: &BoundingBox,
) -> Option<(&'static dyn RegionAdapter, &'static dyn PointCloudBackend)> {
    let region = raster_region_in(registry(), region_bbox)?;
    let backend = region.point_cloud().filter(|backend| {
        project.wants_vegetation() && backend.info().epsg == project.crs().epsg()
    })?;
    Some((region, backend))
}

/// The point-cloud tiles under `area.bbox` (the map area plus buffer in the map's CRS) for the
/// vegetation candidates (ADR 0105), recorded in `project` when [`point_cloud_backend`] gives a
/// backend; otherwise nothing is requested and the records go. Optional like the K5 map: a full
/// disk (`skip`) or a failed fetch keeps the recorded tiles that are still cached, the latter
/// with a warning.
fn fetch_point_clouds_into(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    skip: bool,
    project: &mut ProjectFile,
    area: &FetchArea,
    progress: &mut dyn FnMut(Note),
) {
    let Some((region, backend)) = point_cloud_source(project, &area.region_bbox) else {
        // The choice has no effect here: say so once rather than leave the map maker waiting.
        if project.wants_vegetation() {
            progress(note!("progress-no-point-clouds"));
        }
        project.point_clouds.clear();
        return;
    };
    let still_cached = |records: &[PointCloudRecord]| -> Vec<PointCloudRecord> {
        records
            .iter()
            .filter(|record| backend.cached(cache, record).is_some())
            .cloned()
            .collect()
    };
    if skip {
        project.point_clouds = still_cached(&project.point_clouds);
        return;
    }
    // Like every request for a state, only to the hosts its manifest declares.
    let fetched = crate::adapters::asking(region, fetcher, |fetcher| {
        let mut env = AdapterEnv {
            fetcher,
            regions: registry(),
            cache,
            mode,
            today: crate::clock::today(),
            skip_surface: false,
            progress: &mut *progress,
        };
        backend.fetch(&mut env, &area.bbox)
    });
    match fetched {
        Ok(tiles) => project.point_clouds = tiles.into_iter().map(|t| t.record).collect(),
        Err(e) => {
            progress(note!(
                "warning-point-clouds-unavailable",
                reason = Arg::error(e)
            ));
            project.point_clouds = still_cached(&project.point_clouds);
        }
    }
}

/// The cached files of the recorded point-cloud tiles with their tile names, while
/// [`point_cloud_backend`] gives a backend. A tile no longer cached is left out with a warning.
fn point_cloud_files(
    project: &ProjectFile,
    region_bbox: &BoundingBox,
    cache: &Cache,
    progress: &mut dyn FnMut(Note),
) -> Vec<(String, PathBuf)> {
    let Some(backend) = point_cloud_backend(project, region_bbox) else {
        return Vec::new();
    };
    if project.point_clouds.is_empty() {
        // Chosen after the last fetch (on the page, with `export --content-set`): the point
        // clouds still have to be loaded.
        progress(note!("warning-point-clouds-not-fetched"));
        return Vec::new();
    }
    project
        .point_clouds
        .iter()
        .filter_map(|record| {
            let Some(path) = backend.cached(cache, record) else {
                progress(note!("warning-point-cloud-no-cached", tile = &record.tile));
                return None;
            };
            Some((record.tile.clone(), path))
        })
        .collect()
}

/// The raster backend of a map of `region_bbox`: that of the region its centre lies in.
fn raster_backend(region_bbox: &BoundingBox) -> Result<&'static dyn RasterBackend, ProjectError> {
    raster_region_in(registry(), region_bbox)
        .and_then(|adapter| adapter.rasters())
        .ok_or(ProjectError::OutsideCoverage)
}

/// The raster template recorded in `k5` as the file [`K5_RELPATH`] (PNG and world file) over
/// `bbox`, the map area plus buffer in the map's CRS: the template of the adapter that has it
/// ([`extras::fetch_template_key_into`]). Optional like the extra layers: a failed request or a
/// full disk (`skip`) leaves the template out with a warning, never fails the fetch. A stored
/// template that covers the area (same grid, see `check_template`) is kept. Outside the region
/// of the adapter (`region_bbox`), and for a map in another CRS than the template is served in,
/// nothing is requested and an earlier template is removed. Offline the stored template stays,
/// and so it does in a run whose registry has no adapter with the template.
#[allow(clippy::too_many_arguments)]
fn fetch_k5_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    skip: bool,
    project: &mut ProjectFile,
    region_bbox: &BoundingBox,
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(Note),
) {
    let map_crs = project.crs();
    extras::fetch_template_key_into(
        registry(),
        K5_TEMPLATE,
        dir,
        fetcher,
        mode,
        skip,
        map_crs,
        region_bbox,
        bbox,
        &mut project.k5,
        progress,
    );
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
    progress: &mut dyn FnMut(Note),
) -> Result<OsmRecord, ProjectError> {
    let area = project.area()?.geo_box(project.buffer_m);
    let mut extracts = Vec::new();
    for source in extracts_for(&project.region_bbox()?, progress) {
        let mode = if skip_update && cached_extract_date(cache, &source).is_some() {
            FetchMode::Offline
        } else {
            mode
        };
        match fetch_extract(fetcher, cache, &source, mode, progress) {
            Ok(extract) => extracts.push((source, extract)),
            // Offline without an extract, stored candidates that cover the area are still good.
            Err(FetchError::OsmExtractMissing)
                if dir.join(OSM_CANDIDATES_RELPATH).exists()
                    && project.osm.as_ref().is_some_and(|r| r.covers(&area)) =>
            {
                let record = project.osm.clone().expect("checked above");
                progress(note!(
                    "warning-osm-extract-missing-kept",
                    date = record.extract.to_string()
                ));
                return Ok(record);
            }
            Err(e) => return Err(e.into()),
        }
    }
    let rules = Rules::embedded();
    let mut parts = Vec::new();
    for (source, extract) in &extracts {
        progress(note!(
            "progress-osm-extract-read",
            region = source.label(),
            date = extract.data_date.to_string()
        ));
        parts.push(read_extract(&extract.path, &area, &|tags| {
            rules.wants(tags, project.map_kind)
        })?);
    }
    let features = merge_features(parts);
    let path = dir.join(OSM_CANDIDATES_RELPATH);
    write_inside(dir, &path, features_to_geojson(&features).as_bytes())?;
    progress(note!("progress-osm-candidates", count = features.len()));
    let extract_date = extracts
        .iter()
        .map(|(_, e)| e.data_date)
        .min()
        .expect("every map area touches a region");
    Ok(OsmRecord {
        extract: extract_date,
        area: stored_geo_box(&area),
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

/// The surroundings of an ALKIS fetch: the services are asked through the adapters.
fn alkis_env<'a>(
    fetcher: &'a dyn Fetcher,
    cache: &'a CacheDir,
    mode: FetchMode,
    progress: &'a mut dyn FnMut(Note),
) -> AdapterEnv<'a> {
    AdapterEnv {
        fetcher,
        regions: registry(),
        cache,
        mode,
        today: crate::clock::today(),
        skip_surface: false,
        progress,
    }
}

/// Downloads the ALKIS buildings in `bbox` to [`BUILDINGS_RELPATH`] and returns the new record;
/// the stored file is only replaced by an answer that parses. Offline, the stored file must
/// exist and, if recorded, cover `bbox`; nothing new is recorded then.
fn fetch_buildings_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    area: &FetchArea,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(Note),
) -> Result<Option<AlkisRecord>, ProjectError> {
    let bbox = &area.bbox;
    let path = dir.join(BUILDINGS_RELPATH);
    if mode == FetchMode::Offline {
        if previous.is_none() && path.exists() {
            progress(note!("warning-alkis-coverage-unknown"));
        }
        let covered = previous.is_none_or(|record| record.covers(bbox));
        return if path.exists() && covered {
            Ok(None)
        } else {
            Err(ProjectError::BuildingsNotFetched)
        };
    }
    progress(note!("progress-alkis-buildings-download"));
    let text = fetch_buildings_in(&mut alkis_env(fetcher, cache, mode, &mut *progress), area)?;
    let parsed = parse_buildings(&text).map_err(FetchError::from)?;
    write_inside(dir, &path, text.as_bytes())?;
    let count = parsed.buildings.len();
    progress(note!("progress-alkis-buildings", count = count));
    if count == 0 {
        progress(note!("warning-alkis-no-buildings"));
    }
    if !parsed.skipped.is_empty() {
        let shown = parsed.skipped.iter().take(SKIPPED_SHOWN).cloned();
        let more = parsed.skipped.len() > SKIPPED_SHOWN;
        progress(note!(
            "progress-alkis-skipped",
            count = parsed.skipped.len(),
            list = format!(
                "{}{}",
                shown.collect::<Vec<_>>().join("; "),
                if more { "; …" } else { "" }
            )
        ));
    }
    let record = AlkisRecord {
        bbox: [
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil(),
        ],
        fetched: crate::clock::today(),
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
            progress(note!(
                "warning-alkis-buildings-changed",
                changes = changes.join(", ")
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
    cache: &CacheDir,
    area: &FetchArea,
    mode: FetchMode,
    previous: Option<&AlkisRecord>,
    progress: &mut dyn FnMut(Note),
) -> Option<Option<AlkisRecord>> {
    let bbox = &area.bbox;
    let path = dir.join(LAND_USE_RELPATH);
    if mode == FetchMode::Offline {
        if previous.is_none() || !path.exists() {
            progress(note!("progress-alkis-no-land-use-stored"));
        }
        return None;
    }
    progress(note!("progress-alkis-land-use-download"));
    let fetched = fetch_land_use_in(&mut alkis_env(fetcher, cache, mode, &mut *progress), area);
    let downloaded = fetched.and_then(|text| {
        let land_use = parse_land_use(&text).map_err(FetchError::from)?;
        write_inside(dir, &dir.join(LAND_USE_RELPATH), text.as_bytes())?;
        Ok(land_use)
    });
    match downloaded {
        Ok(land_use) => {
            progress(note!(
                "progress-alkis-land-use-parts",
                count = land_use.len()
            ));
            Some(Some(AlkisRecord {
                bbox: [
                    bbox.min_e.floor(),
                    bbox.min_n.floor(),
                    bbox.max_e.ceil(),
                    bbox.max_n.ceil(),
                ],
                fetched: crate::clock::today(),
                latest_update: land_use.iter().filter_map(|l| l.updated).max(),
                count: land_use.len(),
                regions: regions_of(land_use.iter().map(|l| l.oid.as_str())),
            }))
        }
        Err(e) => match previous.filter(|_| path.exists()) {
            Some(record) if record.covers(bbox) => {
                progress(note!("warning-alkis-land-use-kept", error = Arg::error(e)));
                None
            }
            Some(_) => {
                progress(note!(
                    "warning-alkis-land-use-removed",
                    error = Arg::error(e)
                ));
                freemapper_fs::remove_inside(dir, &path);
                Some(None)
            }
            None => {
                progress(note!("warning-alkis-land-use-none", error = Arg::error(e)));
                None
            }
        },
    }
}

/// The key of the set of extra layers whose records the project file keeps in `berlin_layers`:
/// the one set there is. A second one needs a field of its own.
const REGISTERS_SET: &str = "berlin";

/// The family of the set of extra layers recorded in `berlin_layers`, from the adapter of the
/// registry that has it; `None` when no adapter of this run has it.
fn registers() -> Option<extras::Family<'static>> {
    extras::set_in(registry(), REGISTERS_SET).map(|(_, set)| extras::of_set(set))
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
    progress: &mut dyn FnMut(Note),
) -> BTreeMap<String, AlkisRecord> {
    extras::fetch_key_into(
        registry(),
        REGISTERS_SET,
        dir,
        fetcher,
        region_bbox,
        kind,
        mode,
        skip,
        previous,
        progress,
    )
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
    progress: &mut dyn FnMut(Note),
) -> BTreeMap<String, AlkisRecord> {
    let in_nas_state = !alkis_nas::nas_services_for(&area.region_bbox).is_empty();
    extras::fetch_into(
        &extras::NAS,
        dir,
        in_nas_state,
        mode,
        skip,
        &area.bbox,
        previous,
        progress,
        |part| {
            alkis_nas::fetch_nas_layers(fetcher, area, kind, part)
                .into_iter()
                .map(|fetched| extras::Fetched {
                    key: fetched.layer.file.to_string(),
                    features: fetched.features.map(|fetched| extras::Features {
                        features: fetched.features,
                        skipped: fetched.skipped,
                        regions: fetched.regions,
                    }),
                })
                .collect()
        },
    )
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
        if let Some(stored) =
            extras::stored(&extras::NAS, dir, &project.nas_layers, &bbox, layer.file)?
        {
            layers.push(StoredNasLayer {
                layer,
                features: stored.features,
                fetched: stored.fetched,
            });
        }
    }
    Ok(layers)
}

/// Whether a map of `kind` uses the city tree cadastres; a forest map draws none of their trees.
fn fetches_tree_registers(kind: MapKind) -> bool {
    kind != MapKind::Forest
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
    progress: &mut dyn FnMut(Note),
) -> BTreeMap<String, AlkisRecord> {
    let in_city = !trees::tree_datasets_for(&area.region_bbox).is_empty();
    extras::fetch_into(
        &extras::TREES,
        dir,
        in_city,
        mode,
        skip,
        &area.bbox,
        previous,
        progress,
        |part| {
            trees::fetch_tree_layers(fetcher, area, part)
                .into_iter()
                .map(|fetched| extras::Fetched {
                    key: fetched.dataset.id.to_string(),
                    features: fetched.features.map(|fetched| extras::Features {
                        features: fetched.features,
                        skipped: fetched.skipped,
                        regions: fetched.regions,
                    }),
                })
                .collect()
        },
    )
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
    for dataset in trees::all_tree_datasets() {
        if let Some(stored) = extras::stored(
            &extras::TREES,
            dir,
            &project.tree_layers,
            &bbox,
            &dataset.id,
        )? {
            layers.push(StoredTreeLayer {
                dataset,
                features: stored.features,
                fetched: stored.fetched,
            });
        }
    }
    Ok(layers)
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
    let Some(registers) = registers() else {
        return Ok(Vec::new());
    };
    let region_bbox = project.region_bbox()?;
    let mut layers = Vec::new();
    for layer in berlin::extra_layers(project.map_kind.contract()) {
        if let Some(stored) = extras::stored(
            &registers,
            dir,
            &project.berlin_layers,
            &region_bbox,
            layer.file,
        )? {
            layers.push(StoredBerlinLayer {
                layer,
                features: stored.features,
                fetched: stored.fetched,
            });
        }
    }
    Ok(layers)
}

/// Whether a layer that gives stairs is stored: the stair terrain is read only then.
fn stair_layers_stored(layers: &[StoredBerlinLayer], nas: &[StoredNasLayer]) -> bool {
    layers
        .iter()
        .any(|l| [CANOPY_LAYER, STEP_EDGE_LAYER, SURVEY_STAIR_LAYER].contains(&l.layer.file))
        || nas.iter().any(|l| l.layer.file == NAS_STAIR_LAYER)
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

/// Writes orthophoto, hillshade and slope templates from the fetched tiles and returns the
/// image paths.
pub fn derive_project(
    dir: &Path,
    cache: &Cache,
    progress: &mut dyn FnMut(Note),
) -> Result<Vec<PathBuf>, ProjectError> {
    derive_project_cancellable(dir, cache, progress, &|| false)
}

/// [`derive_project`] that asks `cancelled` before each template it writes. Every template is
/// written whole (partial file, then rename), so a stop leaves the finished ones and no
/// partial file; the next derive writes them all again.
pub fn derive_project_cancellable(
    dir: &Path,
    cache: &Cache,
    progress: &mut dyn FnMut(Note),
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<PathBuf>, ProjectError> {
    stop_if(cancelled)?;
    let mut project = ProjectFile::load_reporting(dir, progress)?;
    check_folders(dir, &WRITTEN_FOLDERS)?;
    if project.tiles.is_empty() && project.terrain.is_none() {
        return Err(ProjectError::NotFetched);
    }
    let bbox = project.bbox()?;
    let region_bbox = project.region_bbox()?;
    let inputs = raster_backend(&region_bbox)?.derive_inputs(
        &DeriveEnv {
            regions: registry(),
            cache,
        },
        &StoredRasters {
            tiles: &project.tiles,
            ndsm_tiles: &project.ndsm_tiles,
            orthophoto: project.orthophoto.as_ref(),
            terrain: project.terrain.as_ref(),
            surface: project.surface.as_ref(),
        },
        &RasterArea { bbox, region_bbox },
    );
    if !inputs.outdated.is_empty() {
        return Err(ProjectError::TilesOutdated(inputs.outdated));
    }

    crate::fsutil::remove_stale_parts_inside(dir, &dir.join("templates"), STALE_TEMPLATE_PART_AGE);
    let orthophoto = dir.join(ORTHOPHOTO_RELPATH);
    progress(note!("progress-writing", path = ORTHOPHOTO_RELPATH));
    stop_if(cancelled)?;
    match inputs.orthophoto? {
        OrthophotoInput::Tiles(tiles) => build_orthophoto(&tiles, &bbox, dir, &orthophoto)?,
        OrthophotoInput::Image => {
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
    stop_if(cancelled)?;
    // A tile provider's terrain mosaic is only in the cache.
    let tile_provider = matches!(inputs.dtm, Ok(HeightInput::Tiles(_)));
    let dtm = if let HeightInput::Tiles(tiles) = inputs.dtm? {
        mosaic_dtm(&tiles, &bbox)?
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
    // The export reads the stair direction and the height of the cliffs on a forest map
    // (`build/cliffs.rs`) from the terrain; a tile provider's mosaic is only in the cache, so it is
    // kept with the project of a forest map and where the Berlin layers have stairs.
    let mosaic = dir.join(TERRAIN_MOSAIC_RELPATH);
    // Only a tile provider's mosaic needs keeping, and the stored layers are read once, only
    // when the Berlin layers have no stairs.
    let keep_mosaic = tile_provider
        && (project.map_kind == MapKind::Forest || {
            let berlin_stairs = !project.berlin_layers.is_empty()
                && stored_berlin_layers(dir, &project)?
                    .iter()
                    .any(|l| has_stairs(&l.features));
            // A NAS stair without a bottom edge goes by the terrain too.
            berlin_stairs
                || stored_nas_layers(dir, &project)?.iter().any(|l| {
                    l.layer.file == NAS_STAIR_LAYER && has_nas_stairs_without_edge(&l.features)
                })
        });
    if keep_mosaic {
        progress(note!("progress-writing", path = TERRAIN_MOSAIC_RELPATH));
        freemapper_sources::float_tiff::write_float_raster(
            &dtm,
            dir,
            &mosaic,
            TERRAIN_MOSAIC_RELPATH,
        )?;
    } else {
        for file in [mosaic.clone(), mosaic.with_extension("tfw")] {
            freemapper_fs::remove_inside(dir, &file);
        }
    }
    let hillshade_path = dir.join(HILLSHADE_RELPATH);
    progress(note!("progress-writing", path = HILLSHADE_RELPATH));
    stop_if(cancelled)?;
    write_gray_png(&hillshade(&dtm), dir, &hillshade_path)?;
    let slope_path = dir.join(SLOPE_RELPATH);
    progress(note!("progress-writing", path = SLOPE_RELPATH));
    stop_if(cancelled)?;
    let black_deg = project.map_kind.slope_black_deg();
    write_gray_png(&slope_shade(&slope_deg(&dtm), black_deg), dir, &slope_path)?;
    let relief_path = dir.join(RELIEF_RELPATH);
    progress(note!("progress-writing", path = RELIEF_RELPATH));
    stop_if(cancelled)?;
    let relief = local_relief(&dtm, project.map_kind.relief_sigma_m());
    write_gray_png(&relief_shade(&relief, RELIEF_SPAN_M), dir, &relief_path)?;

    progress(note!("progress-writing", path = CONTOURS_RELPATH));
    stop_if(cancelled)?;
    let smoothed = gaussian_smooth(&dtm, project.map_kind.contour_sigma_m());
    let kind_interval = project.map_kind.contour_interval_m();
    let scale = SymbolScale::new(project.map_kind, project.scale_denominator);
    let interval = match project.contour_interval_m {
        Some(interval) => interval,
        None if project.map_kind == MapKind::Forest => {
            let map_area = project.area()?.projected_polygon_in(project.crs());
            contour_interval_for(
                project.map_kind,
                &terrain_inside(&smoothed, &map_area),
                scale,
            )
        }
        None => kind_interval,
    };
    if project.contour_interval_m.is_none() && interval > kind_interval {
        progress(note!(
            "progress-contours-steep",
            interval = &interval.to_string(),
            default = &kind_interval.to_string()
        ));
    }
    let lines = generalize(contour_lines(&smoothed, interval), scale);
    let contours_path = dir.join(CONTOURS_RELPATH);
    // The interval is stored where it is not the kind's, so the export draws index contours and
    // names it right; the files of the other projects stay as they were.
    let stored_interval = (interval != kind_interval).then_some(interval);
    write_inside(
        dir,
        &contours_path,
        contours_to_geojson(&lines, stored_interval).as_bytes(),
    )?;
    // The sidecar names it for the suite; a project at the kind's interval keeps its file as it was.
    let record = stored_interval.map(|interval_m| ContourRecord { interval_m });
    if project.contours != record {
        project.contours = record;
        project.save(dir)?;
    }
    progress(note!("progress-writing", path = LANDFORMS_RELPATH));
    stop_if(cancelled)?;
    let min_height_m = project.map_kind.landform_min_height_m();
    let mut landforms = knolls_and_depressions(&dtm, scale, min_height_m);
    landforms.extend(height_jumps(&dtm, scale, min_height_m));
    let landforms_path = dir.join(LANDFORMS_RELPATH);
    write_inside(
        dir,
        &landforms_path,
        landforms_to_geojson(&landforms).as_bytes(),
    )?;
    // The vegetation candidates (ADR 0105), where the fetch recorded point clouds.
    let vegetation_path = dir.join(VEGETATION_RELPATH);
    let point_clouds = point_cloud_files(&project, &region_bbox, cache, progress);
    let vegetation = if point_clouds.is_empty() {
        freemapper_fs::remove_inside(dir, &vegetation_path);
        None
    } else {
        progress(note!("progress-writing", path = VEGETATION_RELPATH));
        // The point clouds do not tell walls from bushes: the returns at buildings are left out.
        // Without stored buildings (or with a damaged file, which the export reports) there is
        // no mask.
        let footprints = export::read_stored_buildings(dir)
            .ok()
            .flatten()
            .map_or_else(Vec::new, |b| b.all_footprints());
        let mask = BuildingMask::new(&footprints, &bbox);
        let left_out = left_out_classes(&project, &region_bbox);
        let mut counts = CellCounts::new(GridSpec::covering(&bbox, VEGETATION_CELL_M));
        for (tile, path) in &point_clouds {
            stop_if(cancelled)?;
            // A damaged file costs only its tile, not the whole derive.
            if let Err(e) = counts.add_tile(path, &dtm, Some(&mask), left_out) {
                progress(note!(
                    "warning-point-cloud-unreadable",
                    tile = tile,
                    error = Arg::error(e),
                    path = path.display().to_string()
                ));
            }
        }
        let areas = vegetation_areas(&counts.classes(), scale);
        write_inside(
            dir,
            &vegetation_path,
            vegetation_to_geojson(&areas).as_bytes(),
        )?;
        Some(vegetation_path)
    };

    let mut written = vec![orthophoto, hillshade_path, slope_path, relief_path];
    if project.buildings.as_ref().is_some_and(|b| b.count == 0) {
        // A region whose buildings come from a service of their own says so when the area lies
        // on its ground and none is stored.
        for warning in no_buildings_warnings(&region_bbox) {
            progress(Note::new(warning));
        }
    }
    let dsm_recorded = match &inputs.dsm {
        Some(Ok(HeightInput::Stored)) => stored_model(&project.surface, SURFACE_RELPATH),
        Some(_) => true,
        None => false,
    };
    if project.ndsm_tiles.is_empty() && !dsm_recorded {
        progress(note!("progress-no-ndsm-recorded"));
    }
    let ndsm_files: Vec<PathBuf> = inputs
        .ndsm
        .into_iter()
        .filter_map(|record| {
            let Some(path) = record.path else {
                progress(note!(
                    "warning-ndsm-tile-invalid",
                    tile = format!("{:?}", record.tile)
                ));
                return None;
            };
            if !path.exists() {
                progress(note!("warning-ndsm-not-cached", tile = &record.tile));
                return None;
            }
            // A damaged cached file costs only its part of the template, not the whole derive.
            if let Err(e) = las::Reader::from_path(&path) {
                progress(note!(
                    "warning-ndsm-unreadable",
                    tile = &record.tile,
                    error = Arg::error(e),
                    path = path.display().to_string()
                ));
                return None;
            }
            Some(path)
        })
        .collect();
    let ndsm_path = dir.join(NDSM_RELPATH);
    stop_if(cancelled)?;
    let dsm = if dsm_recorded {
        let model = match inputs.dsm {
            Some(Ok(HeightInput::Tiles(tiles))) => {
                mosaic_dtm(&tiles, &bbox).map_err(ProjectError::from)
            }
            Some(Err(e)) => Err(ProjectError::from(e)),
            Some(Ok(HeightInput::Stored)) | None => {
                read_float_raster(&dir.join(SURFACE_RELPATH)).map_err(ProjectError::from)
            }
        };
        match model {
            Ok(dsm) => Some(dsm),
            Err(e) => {
                progress(note!("warning-surface-unusable", error = Arg::error(e)));
                None
            }
        }
    } else {
        None
    };
    if ndsm_files.is_empty() && dsm.is_none() {
        // An older nDSM template would no longer match the map area: remove it.
        for stale in [ndsm_path.clone(), ndsm_path.with_extension("pgw")] {
            freemapper_fs::remove_inside(dir, &stale);
        }
    } else {
        progress(note!("progress-writing", path = NDSM_RELPATH));
        stop_if(cancelled)?;
        let mut ndsm = rasterize(&ndsm_files, &bbox)?;
        // Consumed here: the 0.2 m surface model is as large as the nDSM itself.
        if let Some(dsm) = dsm {
            fill_from_dsm(&mut ndsm, &dsm, &dtm);
        }
        write_ndsm_png(&ndsm, dir, &ndsm_path)?;
        written.push(ndsm_path);
    }
    written.push(contours_path);
    written.push(landforms_path);
    written.extend(vegetation);
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
    /// The preview written with the map (`<name>-preview.webp`); none when the map was kept, or
    /// when the preview failed (the run then warns, `warning-map-preview-failed`).
    pub preview: Option<PathBuf>,
}

/// The step [`run_project_steps`] is about to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStage {
    Fetch,
    Derive,
    Map,
}

/// Fetch, derive and export. An existing map is kept unless `force` is set, because it may
/// contain the mapper's work.
pub fn run_project(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    force: bool,
    progress: &mut dyn FnMut(Note),
) -> Result<RunOutcome, ProjectError> {
    run_project_steps(dir, fetcher, cache, mode, force, progress, &mut |_| {})
}

/// [`run_project`] that also says which step starts next, for a progress display.
pub fn run_project_steps(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    force: bool,
    progress: &mut dyn FnMut(Note),
    stage: &mut dyn FnMut(RunStage),
) -> Result<RunOutcome, ProjectError> {
    run_project_steps_cancellable(dir, fetcher, cache, mode, force, progress, stage, &|| false)
}

/// [`run_project`] that stops with [`ProjectError::Cancelled`] when `cancelled` says so: before
/// each step, at each part of the fetch and derive, and before each request. Whatever was
/// written before is whole, so the next run continues from there.
pub fn run_project_cancellable(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    force: bool,
    progress: &mut dyn FnMut(Note),
    cancelled: &dyn Fn() -> bool,
) -> Result<RunOutcome, ProjectError> {
    run_project_steps_cancellable(
        dir,
        fetcher,
        cache,
        mode,
        force,
        progress,
        &mut |_| {},
        cancelled,
    )
}

/// [`run_project_steps`] with the stop of [`run_project_cancellable`].
#[allow(clippy::too_many_arguments)]
pub fn run_project_steps_cancellable(
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    force: bool,
    progress: &mut dyn FnMut(Note),
    stage: &mut dyn FnMut(RunStage),
    cancelled: &dyn Fn() -> bool,
) -> Result<RunOutcome, ProjectError> {
    stop_if(cancelled)?;
    stage(RunStage::Fetch);
    fetch_project_cancellable(dir, fetcher, cache, mode, progress, cancelled)?;
    stop_if(cancelled)?;
    stage(RunStage::Derive);
    let templates = derive_project_cancellable(dir, cache, progress, cancelled)?;
    stop_if(cancelled)?;
    stage(RunStage::Map);
    let existing = map_path(dir);
    let mut preview = None;
    let map = if existing.exists() && !force {
        let text = std::fs::read_to_string(&existing)
            .map_err(|e| ProjectError::Io(existing.clone(), e))?;
        write_starter_project(dir, &starter_documents(dir)?, None)?;
        MapOutcome::Kept {
            references_images: text.contains(ORTHOPHOTO_RELPATH),
            missing_buildings: map_lacks_buildings(dir, &text)?,
            missing_canopies: map_lacks_canopies(dir, &text)?,
            map: existing,
        }
    } else {
        // The preview goes with the map, from the same documents; never next to a kept map.
        let documents = starter_documents(dir)?;
        let map = export_starter_project_from(dir, &documents, force)?;
        preview = export_map_preview_or_warn(dir, &documents, progress);
        MapOutcome::Written(map)
    };
    Ok(RunOutcome {
        templates,
        map,
        preview,
    })
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

pub(crate) const OSM_ATTRIBUTION: &str = "© OpenStreetMap contributors, ODbL";
/// [`OSM_ATTRIBUTION`] for the German peripheral information.
const OSM_ATTRIBUTION_DE: &str = "© OpenStreetMap-Mitwirkende, ODbL";
/// [`FREEMAPPER_CREDIT`] for the German peripheral information.
const FREEMAPPER_CREDIT_DE: &str =
    concat!("Erstellt mit freemapper – ", env!("CARGO_PKG_REPOSITORY"));
/// Names the tool and where to get it, in every generated project.
const FREEMAPPER_CREDIT: &str = concat!("Created with freemapper – ", env!("CARGO_PKG_REPOSITORY"));

/// The registers whose points are among `objects`: the files of the Berlin registers
/// ([`BERLIN_REGISTERS`]) and the ids of the city tree cadastres (`trees::all_tree_datasets`). A
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
    let cities = trees::all_tree_datasets()
        .map(|d| d.id.as_ref())
        .filter(|id| drawn(format!("trees/{id}")));
    berlin.chain(cities).collect()
}

/// Attribution of every source the project uses, for the project notes: OpenStreetMap, then the
/// credit lines of the adapters with the licence texts they refer to. `registers` are the Berlin
/// registers and city tree cadastres that drew objects ([`drawn_registers`]).
fn attribution_lines(project: &ProjectFile, registers: &[&str]) -> Vec<String> {
    let mut lines = vec![OSM_ATTRIBUTION.to_string()];
    lines.extend(lines_with_licences(&credit_lines(project, registers)));
    lines
}

/// The credit lines of the data sources for the map sheet: those of [`attribution_lines`]
/// without OpenStreetMap and without the licence texts.
fn sheet_credit_lines(project: &ProjectFile, registers: &[&str]) -> Vec<String> {
    credit_lines(project, registers)
        .into_iter()
        .map(|credit| credit.text)
        .collect()
}

/// The credit lines of every adapter for what the project holds of it, in the order of the
/// registry.
fn credit_lines(project: &ProjectFile, registers: &[&str]) -> Vec<CreditLine> {
    let used = used_sources(project, registers);
    registry()
        .adapters()
        .flat_map(|adapter| adapter.attribution(&used))
        .collect()
}

/// What the project holds, as far as the credit lines depend on it.
fn used_sources<'a>(project: &'a ProjectFile, registers: &'a [&'a str]) -> UsedSources<'a> {
    let region_bbox = project.region_bbox().ok();
    UsedSources {
        raster_provider: region_bbox
            .as_ref()
            .and_then(|bbox| raster_region_in(registry(), bbox))
            .map(|adapter| adapter.info().provider.clone()),
        overlapped: region_bbox.as_ref().map_or_else(Vec::new, |bbox| {
            registry()
                .adapters()
                .filter(|adapter| overlaps_in(*adapter, bbox))
                .map(|adapter| adapter.info().id.clone())
                .collect()
        }),
        tiles: &project.tiles,
        orthophoto: project.orthophoto.as_ref(),
        terrain: project.terrain.as_ref(),
        alkis: alkis_records(project)
            .map(|record| AlkisUse {
                regions: record.regions.clone(),
                fetched: record.fetched,
            })
            .collect(),
        extra_layers: project
            .berlin_layers
            .iter()
            .map(|(file, record)| (file.as_str(), record.count))
            .collect(),
        extra_templates: project.k5.iter().map(|_| K5_TEMPLATE).collect(),
        drawn_registers: registers,
    }
}

/// The stored ALKIS records (buildings, land use).
fn alkis_records(project: &ProjectFile) -> impl Iterator<Item = &AlkisRecord> {
    [&project.buildings, &project.land_use]
        .into_iter()
        .flatten()
}

/// The regions of the given ALKIS object ids, in the order of the registry.
fn regions_of<'a>(oids: impl Iterator<Item = &'a str>) -> Vec<AdapterId> {
    let found: Vec<AdapterId> = oids.filter_map(region_of_object_id).collect();
    registry()
        .adapters()
        .map(|adapter| adapter.info().id.clone())
        .filter(|region| found.contains(region))
        .collect()
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

/// The credit lines of a project as the attribution functions give them, for the golden net of
/// the adapter migration (`tests/adapters_golden.rs`). Test and `golden` feature only.
#[cfg(any(test, feature = "golden"))]
pub mod golden {
    use super::*;

    /// What the project of [`credits`] holds.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Held {
        /// The orthophoto, terrain and surface records, and one recorded tile for the tile
        /// providers.
        Rasters,
        /// The ALKIS buildings.
        Alkis,
        Both,
    }

    /// The credit lines of a project: those of `ATTRIBUTION.txt` and those of the map sheet.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Credits {
        pub notes: Vec<String>,
        pub sheet: Vec<String>,
    }

    /// The lines of `ATTRIBUTION.txt` that a project of `region` gets when everything it holds
    /// was fetched in `year`. `rasters` adds the orthophoto, terrain and surface records (and
    /// one recorded tile for the tile providers) to the ALKIS records. `lon` and `lat` place the
    /// map area (a 150 m square) inside the region.
    pub fn credit_lines(
        region: &AdapterId,
        lon: f64,
        lat: f64,
        year: i32,
        rasters: bool,
    ) -> Vec<String> {
        let held = if rasters { Held::Both } else { Held::Alkis };
        credits(region, lon, lat, year, held, &[]).notes
    }

    /// The credit lines that a project of `region` gets when everything it holds (`held`) was
    /// fetched in `year`. `lon` and `lat` place the map area (a 150 m square) inside the region.
    /// The ALKIS services of the states in `neighbours` delivered objects too (land use record).
    pub fn credits(
        region: &AdapterId,
        lon: f64,
        lat: f64,
        year: i32,
        held: Held,
        neighbours: &[AdapterId],
    ) -> Credits {
        let (dlon, dlat) = (0.001_1, 0.000_7);
        let ring = [
            [lon - dlon, lat - dlat],
            [lon + dlon, lat - dlat],
            [lon + dlon, lat + dlat],
            [lon - dlon, lat + dlat],
            [lon - dlon, lat - dlat],
        ];
        let date = NaiveDate::from_ymd_opt(year, 6, 15).expect("valid date");
        let mut project = ProjectFile {
            map_kind: crate::project::MapKind::Sprint,
            scale_denominator: 4000,
            declination_date: date,
            buffer_m: 50.0,
            map_area_source_id: None,
            map_area: serde_json::json!({"type": "Polygon", "coordinates": [ring]}),
            crs: Crs::Utm33.epsg(),
            provider: None,
            title: None,
            contour_interval_m: None,
            contours: None,
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
            point_clouds: Vec::new(),
            content: None,
        };
        let bbox = [0.0, 0.0, 1.0, 1.0];
        let alkis = |regions: Vec<AdapterId>| AlkisRecord {
            bbox,
            fetched: date,
            latest_update: None,
            count: 1,
            regions,
        };
        if held != Held::Rasters {
            project.buildings = Some(alkis(vec![region.clone()]));
        }
        if !neighbours.is_empty() {
            project.land_use = Some(alkis(neighbours.to_vec()));
        }
        if held != Held::Alkis {
            project.orthophoto = Some(WmsRecord {
                flight: None,
                flight_label: None,
                fetched: date,
                bbox,
            });
            let wcs = WcsRecord {
                fetched: date,
                bbox,
                date: None,
            };
            project.terrain = Some(wcs.clone());
            project.surface = Some(wcs);
            let provider = registry()
                .get(region.as_str())
                .map(|adapter| adapter.info().provider.clone());
            let product = match provider.as_ref().map(AdapterId::as_str) {
                Some("geosn") => Some(Product::SaxonyDtm),
                Some("lgb") => Some(Product::Dtm),
                _ => None,
            };
            if let Some(product) = product {
                project.tiles.push(TileRecord {
                    product,
                    tile: "t".to_string(),
                    url: "https://example.invalid/t".to_string(),
                    currency: date,
                    published: date,
                });
            }
        }
        Credits {
            notes: attribution_lines(&project, &[]),
            sheet: sheet_credit_lines(&project, &[]),
        }
    }
}

#[cfg(test)]
mod tests {
    const BEYOND_COVERAGE_WARNING: &str = "warning: part of the map area lies outside the supported states. It lies abroad, at sea or in a state freemapper does not support yet, such as Thuringia. The states have no data there, so the orthophoto can be white and ALKIS gives no buildings or land use";

    use super::*;
    use crate::build::paved::{GREEN_LAYER, SURFACE_LAYER};
    use crate::crs::LatLon;
    use crate::fetch::TilePaths as _;
    use crate::omap::{MapCoord, MapObjectKind};
    #[allow(unused_imports)]
    use crate::sources::region::ids::{IdFacts as _, providers, regions};
    use crate::sources::region::raster_region;
    use chrono::Datelike as _;
    use freemapper_adapters_de::lgb::ProductTiles as _;
    use freemapper_adapters_de::states::credit::{
        BERLIN_ATTRIBUTION, GEOSN_ATTRIBUTION, MV_ATTRIBUTION, SH_ATTRIBUTION, tree_attribution,
    };

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
        let project = set_map_area(dir.path(), &area).unwrap().project;
        let loaded = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project, loaded);
        assert_eq!(loaded.map_area, area.to_geojson_value());
        assert_eq!(loaded.map_area_source_id, None);
        assert_eq!(loaded.title.as_deref(), Some("Vorstadt"));
        assert_eq!(loaded.map_kind, MapKind::Sprint);
        assert_eq!(loaded.declination_date, date);
    }

    fn bad_areas() -> Vec<(&'static str, String)> {
        let ring: Vec<String> = (0..=crate::area::MAX_CORNERS + 1)
            .map(|i| {
                let a = i as f64 / (crate::area::MAX_CORNERS + 1) as f64 * std::f64::consts::TAU;
                format!("[{},{}]", 13.84 + 0.002 * a.cos(), 52.54 + 0.001 * a.sin())
            })
            .collect();
        vec![
            (
                "bow-tie",
                r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.54],[13.85,52.53],[13.84,52.54],[13.84,52.53]]]}"#.to_string(),
            ),
            (
                "too many corners",
                format!(
                    r#"{{"type":"Polygon","coordinates":[[{},{}]]}}"#,
                    ring.join(","),
                    ring[0]
                ),
            ),
        ]
    }

    #[test]
    fn init_checks_the_area_like_the_selection_page() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        for (what, text) in bad_areas() {
            let project = dir.path().join("p");
            let area = MapArea::from_geojson(&text).unwrap();
            let err = init_project(&project, &area, MapKind::Sprint, None, date, None).unwrap_err();
            assert!(
                matches!(err, ProjectError::Area(AreaError::NotAnArea(_))),
                "{what}: {err:?}"
            );
            assert!(!project.exists(), "{what}");
        }
        // A polygon without any corner, built by hand, is an error too, not a panic.
        let empty = MapArea {
            polygon: geo_types::Polygon::new(geo_types::LineString::new(Vec::new()), Vec::new()),
            source_id: None,
        };
        let project = dir.path().join("empty");
        let err = init_project(&project, &empty, MapKind::Sprint, None, date, None).unwrap_err();
        assert!(matches!(err, ProjectError::Area(_)), "{err:?}");
    }

    #[test]
    fn a_stored_bad_area_is_an_error_when_used() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        for (what, text) in bad_areas() {
            let mut project = ProjectFile::load(dir.path()).unwrap();
            project.map_area = serde_json::from_str(&text).unwrap();
            project.save(dir.path()).unwrap();
            // The file still loads, so the area selection can replace the area.
            let project = ProjectFile::load(dir.path()).unwrap();
            assert!(project.area().is_err(), "{what}");
            assert!(project.bbox().is_err(), "{what}");
            assert!(project.region_bbox().is_err(), "{what}");
        }
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.map_area = serde_json::json!({"type":"Polygon","coordinates":[]});
        assert!(project.bbox().is_err());
    }

    #[test]
    fn init_accepts_only_the_scales_print_knows() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let area = MapArea::from_geojson(
            r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.53],[13.85,52.54],[13.84,52.54],[13.84,52.53]]]}"#,
        )
        .unwrap();
        assert_eq!(MapKind::Schoolyard.allowed_scales(), [1000]);
        assert_eq!(MapKind::Sprint.allowed_scales(), [4000, 5000]);
        for (kind, scale, hint) in [
            (MapKind::Schoolyard, 2000, "1:1000"),
            (MapKind::Schoolyard, 4000, "1:1000"),
            (MapKind::Sprint, 2000, "1:4000, 1:5000"),
            (MapKind::Sprint, 3000, "freemapper-print enlarges to 1:3000"),
        ] {
            let project = dir.path().join(format!("p{scale}"));
            let err = init_project(&project, &area, kind, Some(scale), date, None).unwrap_err();
            assert!(
                matches!(err, ProjectError::ScaleNotAllowed { .. }),
                "{kind:?} {scale}: {err:?}"
            );
            assert!(err.to_string().contains(hint), "{err}");
            assert!(!project.exists());
        }
        for (kind, scale) in [
            (MapKind::Schoolyard, 1000),
            (MapKind::Sprint, 4000),
            (MapKind::Sprint, 5000),
        ] {
            let project = dir.path().join(format!("ok{scale}"));
            let made = init_project(&project, &area, kind, Some(scale), date, None).unwrap();
            assert_eq!(made.scale_denominator, scale);
        }
        let err = init_project(
            &dir.path().join("zero"),
            &area,
            MapKind::Sprint,
            Some(0),
            date,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, ProjectError::InvalidScale(0)), "{err:?}");
    }

    #[test]
    fn a_stored_scale_outside_the_table_loads_with_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.scale_denominator = 2000;
        project.save(dir.path()).unwrap();
        let mut warnings = Vec::new();
        let loaded =
            ProjectFile::load_reporting(dir.path(), &mut |m| warnings.push(m.to_string())).unwrap();
        assert_eq!(loaded.scale_denominator, 2000);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("warning: freemapper.toml: scale 1:2000")
                && warnings[0].contains("1:1000"),
            "{warnings:?}"
        );
        project.scale_denominator = 1000;
        project.save(dir.path()).unwrap();
        let mut warnings = Vec::new();
        ProjectFile::load_reporting(dir.path(), &mut |m| warnings.push(m.to_string())).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn init_always_writes_a_title() {
        let dir = tempfile::tempdir().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let area = MapArea::from_geojson(
            r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.53],[13.85,52.54],[13.84,52.54],[13.84,52.53]]]}"#,
        )
        .unwrap();
        for (folder, given, expected) in [
            ("clara-zetkin-schule", None, "Clara-Zetkin-Schule"),
            (
                "kreuz",
                Some("Evangelisches Kreuzgymnasium"),
                "Evangelisches Kreuzgymnasium",
            ),
            ("alte_schule", Some("  \n "), "Alte-Schule"),
        ] {
            let project = dir.path().join(folder);
            let made = init_project(
                &project,
                &area,
                MapKind::Schoolyard,
                None,
                date,
                given.map(str::to_string),
            )
            .unwrap();
            assert_eq!(made.title.as_deref(), Some(expected), "{folder}");
            // The file holds it, so print and courseplan read the same title.
            let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
            assert!(text.contains(&format!("title = \"{expected}\"")), "{text}");
            assert_eq!(made.display_title(&project), expected);
        }
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
        // Erfurt: Thuringia is the one state without data sources.
        let project = dir.path().join("erfurt");
        let erfurt = MapArea::from_geojson(r#"{"type":"Polygon","coordinates":[[[11.03,50.975],[11.04,50.975],[11.04,50.985],[11.03,50.985],[11.03,50.975]]]}"#).unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let err = init_project(&project, &erfurt, MapKind::Sprint, None, date, None).unwrap_err();
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
        assert_eq!(project.buildings.unwrap().regions, [regions::SAXONY_ANHALT]);
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
        assert_eq!(project.provider, Some(providers::LGV));
        assert!(project.tiles.is_empty());
        assert_eq!(
            project.orthophoto.unwrap().flight.as_deref(),
            Some("19.03.2026")
        );
        assert!(project.terrain.is_some() && project.surface.is_some());
        for relpath in [WMS_ORTHOPHOTO_RELPATH, TERRAIN_RELPATH, SURFACE_RELPATH] {
            assert!(dir.path().join(relpath).exists(), "{relpath}");
        }
        assert_eq!(project.buildings.unwrap().regions, [regions::HAMBURG]);
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
        assert_eq!(project.provider, Some(providers::HVBG));
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
        assert_eq!(project.buildings.unwrap().regions, [regions::HESSEN]);
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
        use freemapper_adapters_de::hvbg::{self, Model};
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
            freemapper_adapters_de::hvbg::test_support::METADATA_URL,
            freemapper_adapters_de::hvbg::test_support::metadata_xlsx(&rows),
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
            freemapper_adapters_de::hvbg::test_support::METADATA_URL,
            freemapper_adapters_de::hvbg::test_support::metadata_xlsx(&rows),
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
            freemapper_adapters_de::hvbg::test_support::METADATA_URL,
            freemapper_adapters_de::hvbg::test_support::metadata_xlsx(&rows),
        );
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        let name = freemapper_adapters_de::lgv::tile_names(&tiles[..1]);
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
        assert_eq!(project.provider, Some(providers::LVERM_GEO_SH));
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
            [regions::SCHLESWIG_HOLSTEIN]
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
            &freemapper_adapters_de::lvermgeo_sh::overview_prefix(
                freemapper_adapters_de::lvermgeo_sh::Product::Bdom,
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
        use freemapper_adapters_de::lvermgeo_sh::{Product, overview_prefix};
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
                std::fs::remove_dir_all(cache.provider_dir("lvermgeo_sh").join("bdom")).unwrap();
                &again
            }
            FetchMode::Online => {
                for entry in
                    std::fs::read_dir(cache.provider_dir("lvermgeo_sh").join("overview")).unwrap()
                {
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
            crate::sources::osm::extract_of("hamburg", 0),
            crate::sources::osm::extract_of("schleswigholstein", 0),
        ] {
            fetcher.insert(&extract.url(), pbf.clone());
        }
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        // The rasters come from the centre's provider (Schleswig-Holstein).
        assert_eq!(project.provider, Some(providers::LVERM_GEO_SH));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [regions::HAMBURG, regions::SCHLESWIG_HOLSTEIN]
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
        assert_eq!(before.provider, Some(providers::LGV));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &norderstedt_area())
            .unwrap()
            .project;
        assert_eq!(project.crs(), crate::crs::Crs::Utm32, "the same zone");
        assert_eq!(project.provider, Some(providers::LVERM_GEO_SH));
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
                freemapper_adapters_de::hvbg::WMS.url
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
        let project = set_map_area(dir.path(), &frankfurt_area()).unwrap().project;
        assert_eq!(project.provider, Some(providers::HVBG));
        assert!(project.terrain.is_none() && project.orthophoto.is_none());
        assert!(project.buildings.is_none());
        assert!(!dir.path().join(TERRAIN_RELPATH).exists());
        assert!(!dir.path().join(WMS_ORTHOPHOTO_RELPATH).exists());

        crate::fetch::test_support::serve_synthetic_hessen(&fetcher, &project.bbox().unwrap());
        fetch(dir.path(), &fetcher, &cache);
        let before = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(before.provider, Some(providers::HVBG));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &eppendorf_area()).unwrap().project;
        assert_eq!(project.provider, Some(providers::LGV));
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
        assert_eq!(before.provider, Some(providers::LVERM_GEO));
        assert!(before.terrain.is_some());
        let saved = set_map_area(dir.path(), &eppendorf_area()).unwrap();
        assert!(saved.provider_changed && !saved.zone_changed, "{saved:?}");
        let project = saved.project;
        assert_eq!(project.crs(), crate::crs::Crs::Utm32, "the same zone");
        assert_eq!(project.provider, Some(providers::LGV));
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
        let project = set_map_area(dir.path(), &eppendorf_area()).unwrap().project;
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

    /// The fetch for the check of an adapter brings what the services of the adapters deliver
    /// and leaves the OSM extract alone.
    #[test]
    fn the_sources_of_a_project_come_without_the_osm_extract() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
        let project =
            fetch_project_sources(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {})
                .unwrap();
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert!(project.buildings.is_some() && project.land_use.is_some());
        assert!(project.osm.is_none());
        assert!(!dir.path().join(OSM_CANDIDATES_RELPATH).exists());
        let asked = fetcher.log.borrow();
        assert!(!asked.is_empty());
        assert!(
            asked.iter().all(|url| !url.contains("geofabrik")),
            "{asked:?}"
        );
        // The same project fetched whole reads the extract.
        drop(asked);
        assert!(fetch(dir.path(), &fetcher, &cache).0.osm.is_some());
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
        let project = set_map_area(dir.path(), &latina_area()).unwrap().project;
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
        let back = set_map_area(dir.path(), &fixture_area()).unwrap().project;
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
    fn fixed_clock_dates_the_records() {
        // A day that is never today, so the test cannot pass by reading the system clock.
        let at = NaiveDate::from_ymd_opt(2020, 1, 2)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        crate::clock::with_fixed(at, || fetch(dir.path(), &fetcher, &cache));
        let stored = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        let fetched: Vec<&str> = stored
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("fetched = "))
            .collect();
        assert!(fetched.len() >= 3, "{stored}");
        assert!(
            fetched
                .iter()
                .all(|line| *line == "fetched = \"2020-01-02\""),
            "{fetched:?}"
        );
    }

    /// The project file must be the same bytes on every system. A projected coordinate differs
    /// in its last digits with the maths library, so the areas of the records are stored on a
    /// grid, and a record still covers the area it was fetched for.
    #[test]
    fn stored_areas_are_on_a_grid() {
        use freemapper_adapter_api::records::{STORED_METRE, stored_bbox_covers};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        let bbox = project.bbox().unwrap();
        let gridded = |value: f64, step: f64| (on_grid(value, step) - value).abs() < step / 1e3;
        let wms = project.orthophoto.clone().unwrap();
        let terrain = project.terrain.clone().unwrap();
        let surface = project.surface.clone().unwrap();
        for stored in [wms.bbox, terrain.bbox, surface.bbox] {
            assert!(
                stored.iter().all(|v| gridded(*v, STORED_METRE)),
                "{stored:?}"
            );
        }
        assert!(wms.covers(&bbox) && terrain.covers(&bbox) && surface.covers(&bbox));
        let osm = project.osm.clone().unwrap();
        assert!(
            osm.area.iter().all(|v| gridded(*v, STORED_DEGREE)),
            "{:?}",
            osm.area
        );
        assert!(osm.covers(&project.area().unwrap().geo_box(project.buffer_m)));
        // The file shows no more digits than the grid has.
        assert_eq!(
            on_grid(5_816_416.924_000_4, STORED_METRE).to_string(),
            "5816416.924"
        );
        assert_eq!(
            on_grid(13.384_737_621, STORED_DEGREE).to_string(),
            "13.38473762"
        );
        // A record written before the grid, with every digit of the projection, still covers.
        let exact = [bbox.min_e, bbox.min_n, bbox.max_e, bbox.max_n];
        assert!(stored_bbox_covers(exact, &bbox));
        // One that ends a centimetre short does not.
        let short = [bbox.min_e + 0.01, bbox.min_n, bbox.max_e, bbox.max_n];
        assert!(!stored_bbox_covers(short, &bbox));
    }

    #[test]
    fn mv_project_fetches_products() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm33);
        assert_eq!(project.provider, Some(providers::LAIV_MV));
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
            [regions::MECKLENBURG_VORPOMMERN]
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
            crate::derive::contours::contours_to_geojson(&[line], None),
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
            vec![regions::SCHLESWIG_HOLSTEIN, regions::MECKLENBURG_VORPOMMERN];
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
        freemapper_sources::float_tiff::write_float_raster(
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
        assert_eq!(before.provider, Some(providers::LAIV_MV));
        assert!(before.terrain.is_some() && dir.path().join(SURFACE_RELPATH).exists());
        let project = set_map_area(dir.path(), &fixture_area()).unwrap().project;
        assert_eq!(project.crs(), crate::crs::Crs::Utm33, "the same zone");
        assert_eq!(project.provider, Some(providers::LGB));
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
            &format!(
                "{}?type[]=dop20&bbox=",
                freemapper_adapters_de::laiv_mv::OVERVIEW
            ),
            freemapper_adapters_de::laiv_mv::test_support::dop20_overview_answer(&[], "2024-08-14"),
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
            crate::sources::osm::extract_of("brandenburg", 0),
            crate::sources::osm::extract_of("mecklenburgvorpommern", 0),
        ] {
            fetcher.insert(&extract.url(), pbf.clone());
        }
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        // Both states' ALKIS, each OSM extract once, nothing twice.
        assert_eq!(
            project.buildings.as_ref().unwrap().regions,
            [regions::BRANDENBURG, regions::MECKLENBURG_VORPOMMERN]
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
        assert_eq!(project.provider, Some(providers::LAIV_MV));
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
        assert_eq!(before.provider, Some(providers::LGB));
        assert!(!before.tiles.is_empty() && before.buildings.is_some() && before.osm.is_some());
        let project = set_map_area(dir.path(), &rostock_area()).unwrap().project;
        assert_eq!(project.crs(), crate::crs::Crs::Utm33, "the same zone");
        assert_eq!(project.provider, Some(providers::LAIV_MV));
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
        assert_eq!(MapKind::Schoolyard.symbol_set(1000).symbol_count(), 195);
        // Sprint: the walkable area (990) after the sheet symbols.
        assert_eq!(MapKind::Sprint.symbol_set(4000).symbol_count(), 190);
    }

    #[test]
    fn slope_black_threshold_per_map_kind() {
        assert_eq!(MapKind::Schoolyard.slope_black_deg(), 15.0);
        assert_eq!(MapKind::Sprint.slope_black_deg(), 30.0);
    }

    #[test]
    fn forest_is_a_kind_with_the_isom_scales() {
        assert_eq!(MapKind::from_name("forest"), Some(MapKind::Forest));
        assert_eq!(MapKind::Forest.default_scale(), 15000);
        assert_eq!(MapKind::Forest.allowed_scales(), [7500, 10000, 15000]);
        assert_eq!(MapKind::Forest.symbol_enlargement(15000), 1.0);
        assert_eq!(MapKind::Forest.symbol_enlargement(10000), 1.5);
        assert_eq!(MapKind::Forest.symbol_enlargement(7500), 2.0);
        assert_eq!(MapKind::Schoolyard.symbol_enlargement(1000), 1.25);
        assert_eq!(MapKind::Sprint.symbol_enlargement(5000), 1.0);
        assert_eq!(MapKind::Forest.contour_interval_m(), 5.0);
        assert_eq!(MapKind::Forest.default_buffer_m(), 100.0);
        assert_eq!(MapKind::Forest.landform_min_height_m(), 1.0);
        let err = MapKind::Forest.check_scale(4000).unwrap_err().to_string();
        assert!(err.contains("1:7500, 1:10000, 1:15000"), "{err}");
    }

    #[test]
    fn map_kind_sheet_values() {
        assert_eq!(MapKind::Schoolyard.symbol_enlargement(1000), 1.25);
        assert_eq!(MapKind::Sprint.symbol_enlargement(4000), 1.0);
        assert_eq!(MapKind::Schoolyard.contour_interval_m(), 1.0);
        assert_eq!(MapKind::Sprint.contour_interval_m(), 2.0);
        assert_eq!(
            MapKind::Schoolyard.symbol_set(1000).id_for_code("991"),
            Some(191)
        );
        assert_eq!(
            MapKind::Sprint.symbol_set(4000).id_for_code("991"),
            Some(185)
        );
    }

    #[test]
    fn content_is_written_only_when_set() {
        let dir = tempfile::tempdir().unwrap();
        init_project(
            dir.path(),
            &fixture_area(),
            MapKind::Sprint,
            None,
            NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            None,
        )
        .unwrap();
        let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert!(!text.contains("[content]"), "{text}");
        let mut content = ContentSettings::default();
        content.set(
            crate::build::content::Group::Fences,
            Some(crate::build::content::Place::Off),
        );
        let saved = set_content(dir.path(), Some(content.clone())).unwrap();
        assert_eq!(saved.content, Some(content));
        let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert!(
            text.contains("[content]") && text.contains("fences = \"off\""),
            "{text}"
        );
        assert_eq!(set_content(dir.path(), None).unwrap().content, None);
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
            provider: Some(providers::LGB),
            title: Some("Clara-Zetkin-Schule".to_string()),
            contour_interval_m: None,
            contours: None,
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
                regions: vec![regions::BRANDENBURG],
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
                    regions: vec![regions::BRANDENBURG],
                },
            )]),
            tree_layers: BTreeMap::from([(
                "potsdam".to_string(),
                AlkisRecord {
                    bbox: [366_200.0, 5_807_100.0, 367_100.0, 5_807_700.0],
                    fetched: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
                    latest_update: NaiveDate::from_ymd_opt(2026, 3, 2),
                    count: 3,
                    regions: vec![regions::BRANDENBURG],
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
            point_clouds: vec![PointCloudRecord {
                tile: "33421-5821".to_string(),
                url: "https://example.org/als.zip".to_string(),
                published: NaiveDate::from_ymd_opt(2024, 6, 12),
            }],
            content: Some({
                let mut c = ContentSettings::with_profile(crate::build::content::Profile::Full);
                c.set(
                    crate::build::content::Group::Walls,
                    Some(crate::build::content::Place::Candidates),
                );
                c
            }),
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
                templates.join("relief.png"),
                dir.path().join("data/contours.geojson"),
                dir.path().join("data/landforms.geojson")
            ]
        );
        for file in ["orthophoto.jgw", "hillshade.pgw", "slope.pgw", "relief.pgw"] {
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
                "templates/relief.png",
                "templates/candidates.omap"
            ]
        );
        assert!(map.contains(r#"first_front_template="4""#), "{map}");
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
        assert_eq!(
            outcome.templates.len(),
            6,
            "four images, the contours and the landforms"
        );
        assert_eq!(outcome.map, MapOutcome::Written(map_path(dir.path())));
    }

    /// Every file below `dir` whose name says it is a partial file.
    fn part_files(dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut open = vec![dir.to_path_buf()];
        while let Some(folder) = open.pop() {
            for entry in std::fs::read_dir(folder).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    open.push(path);
                } else if path.to_string_lossy().contains(".part") {
                    found.push(path);
                }
            }
        }
        found
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn cancel_before_fetch_returns_cancelled_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let before = names_in(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let error = run_project_cancellable(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Online,
            false,
            &mut |_| {},
            &|| true,
        )
        .unwrap_err();
        assert!(matches!(error, ProjectError::Cancelled), "{error}");
        assert_eq!(error.to_string(), "cancelled");
        assert_eq!(fetcher.calls.get(), 0, "no request after the flag is set");
        assert_eq!(names_in(dir.path()), before);
        assert!(names_in(cache_dir.path()).is_empty());
    }

    #[test]
    fn cancel_between_fetch_and_derive_leaves_no_part_files() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let flag = std::cell::Cell::new(false);
        let error = run_project_steps_cancellable(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Online,
            false,
            &mut |_| {},
            &mut |stage| flag.set(flag.get() || stage == RunStage::Derive),
            &|| flag.get(),
        )
        .unwrap_err();
        assert!(matches!(error, ProjectError::Cancelled), "{error}");
        assert!(
            part_files(dir.path()).is_empty(),
            "{:?}",
            part_files(dir.path())
        );
        assert!(part_files(cache_dir.path()).is_empty());
        let project = ProjectFile::load(dir.path()).unwrap();
        assert!(!project.tiles.is_empty(), "the fetch is kept");
        assert!(
            !dir.path().join(ORTHOPHOTO_RELPATH).exists(),
            "derive did not start"
        );
        assert!(!map_path(dir.path()).exists());
    }

    #[test]
    fn cancelled_project_runs_to_the_end_the_next_time() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        // Cancelled after the first request of the fetch, and again while deriving.
        let asked = &fetcher;
        run_project_cancellable(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Online,
            false,
            &mut |_| {},
            &|| asked.calls.get() >= 1,
        )
        .unwrap_err();
        let outcome = run(dir.path(), &fetcher, &cache, false);
        assert_eq!(outcome.map, MapOutcome::Written(map_path(dir.path())));
        assert_eq!(outcome.templates.len(), 6);
        assert!(part_files(dir.path()).is_empty());
    }

    #[test]
    fn cancel_in_the_middle_of_the_fetch_stops_before_the_next_request() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let error = fetch_project_cancellable(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Online,
            &mut |_| {},
            &|| fetcher.calls.get() >= 2,
        )
        .unwrap_err();
        assert!(matches!(error, ProjectError::Cancelled), "{error}");
        assert_eq!(
            fetcher.calls.get(),
            2,
            "the request after the flag is refused"
        );
        assert!(part_files(dir.path()).is_empty());
        assert!(part_files(cache_dir.path()).is_empty());
    }

    #[test]
    fn cancel_between_derive_steps_keeps_finished_templates_whole() {
        let fetcher = MapFetcher::default();
        let (dir, cache_dir, cache) = fetched_school(&fetcher);
        let flag = std::cell::Cell::new(false);
        let error = derive_project_cancellable(
            dir.path(),
            &cache,
            &mut |line| flag.set(flag.get() || line.to_string().contains("hillshade")),
            &|| flag.get(),
        )
        .unwrap_err();
        assert!(matches!(error, ProjectError::Cancelled), "{error}");
        assert!(part_files(dir.path()).is_empty());
        assert!(part_files(cache_dir.path()).is_empty());
        let orthophoto = dir.path().join(ORTHOPHOTO_RELPATH);
        assert!(std::fs::metadata(&orthophoto).unwrap().len() > 0);
        assert!(!dir.path().join(SLOPE_RELPATH).exists());
        // The next derive completes it.
        assert_eq!(
            derive_project(dir.path(), &cache, &mut |_| {})
                .unwrap()
                .len(),
            6
        );
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
    fn candidates_pdf_draws_map_and_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let pdf_path = export_candidates_pdf(dir.path(), false, PdfSources::Auto).unwrap();
        assert!(
            pdf_path.to_string_lossy().ends_with("-candidates.pdf"),
            "{pdf_path:?}"
        );
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
        let (map, sheet) = pdf_objects(&documents, true);
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
    fn map_pdf_leaves_out_the_candidates() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let documents = starter_documents(dir.path()).unwrap();
        let (with, sheet_with) = pdf_objects(&documents, true);
        let (without, sheet_without) = pdf_objects(&documents, false);
        let own = documents
            .map
            .parts
            .iter()
            .filter(|p| p.name != SHEET_PART)
            .map(|p| p.objects.len())
            .sum::<usize>();
        assert_eq!(without.len(), own);
        assert!(with.len() > without.len());
        assert_eq!(sheet_with, sheet_without, "same sheet in both");
        let map_pdf = export_pdf(dir.path(), false, PdfSources::Auto).unwrap();
        let cand_pdf = export_candidates_pdf(dir.path(), false, PdfSources::Auto).unwrap();
        assert_ne!(map_pdf, cand_pdf);
        assert_eq!(map_pdf.extension().unwrap(), "pdf");
        let a = crate::render::pdf::content_of(&std::fs::read(&map_pdf).unwrap());
        let b = crate::render::pdf::content_of(&std::fs::read(&cand_pdf).unwrap());
        assert_ne!(a, b);
        assert!(a.len() < b.len(), "the candidates add drawing operators");
        let credit: String = "github.com/HenryJobst/freemapper"
            .bytes()
            .map(|b| format!("{b:02X}"))
            .collect();
        assert!(contains(&a, credit.as_bytes()), "sheet is complete");
        // The tartan pitch is a candidate: only the candidates PDF paints it.
        assert!(!contains(&a, b"0.1 0.35 0.5 0 k"));
        assert!(contains(&b, b"0.1 0.35 0.5 0 k"));
    }

    #[test]
    fn map_preview_is_a_webp_of_the_map_area_with_its_content_record() {
        use crate::build::content::{Group, Place, Profile};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let mut content = ContentSettings::with_profile(Profile::Full);
        content.set(Group::Fences, Some(Place::Off));
        let project = set_content(dir.path(), Some(content.clone())).unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        assert_eq!(documents.content_settings, content);

        let path = export_map_preview_from(dir.path(), &documents).unwrap();
        assert_eq!(path, map_preview_path(dir.path()));
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let stem = map_path(dir.path())
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(name, format!("{stem}-preview.webp"));
        let bytes = std::fs::read(&path).unwrap();
        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::WebP)
            .unwrap()
            .to_rgba8();
        let rect = PaperRect::of_bbox(&project.bbox().unwrap(), &documents.map.georef).unwrap();
        let clip = crate::render::pdf::PaperBox {
            min: (
                f64::from(rect.min_x) / 1000.0,
                f64::from(rect.min_y) / 1000.0,
            ),
            max: (
                f64::from(rect.max_x) / 1000.0,
                f64::from(rect.max_y) / 1000.0,
            ),
        };
        let px_per_mm = crate::render::raster::preview_px_per_mm(&clip);
        let expected = (
            (clip.max.0 - clip.min.0) * px_per_mm,
            (clip.max.1 - clip.min.1) * px_per_mm,
        );
        assert!(
            (f64::from(image.width()) - expected.0).abs() <= 1.0
                && (f64::from(image.height()) - expected.1).abs() <= 1.0,
            "{:?} for {expected:?}",
            image.dimensions()
        );
        assert!(expected.0.max(expected.1) <= 2049.0);
        let white = image
            .pixels()
            .filter(|p| p.0 == [255, 255, 255, 255])
            .count();
        assert!(
            white > 0 && white < image.pixels().count(),
            "something is drawn"
        );

        let record = read_map_preview_record(dir.path()).expect("record");
        assert_eq!(record.content, content);
        assert_eq!(
            record.fingerprint,
            project_fingerprint(&ProjectFile::load(dir.path()).unwrap()).unwrap()
        );
        assert!(dir.path().join("data/map-preview.json").exists());
    }

    #[test]
    fn project_fingerprint_counts_only_what_decides_the_build() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let project = ProjectFile::load(dir.path()).unwrap();
        let fingerprint = project_fingerprint(&project).unwrap();
        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
        let with_content = ProjectFile {
            content: Some(ContentSettings::with_profile(
                crate::build::content::Profile::Safe,
            )),
            ..project.clone()
        };
        assert_eq!(project_fingerprint(&with_content).unwrap(), fingerprint);
        // The records of a fetch: every online fetch dates them anew.
        let day = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let record = AlkisRecord {
            bbox: [0.0, 0.0, 1.0, 1.0],
            fetched: day,
            latest_update: None,
            count: 3,
            regions: Vec::new(),
        };
        let fetched = ProjectFile {
            tiles: vec![TileRecord {
                product: Product::Dtm,
                tile: "33421-5821".into(),
                url: "https://example.org/t.zip".into(),
                currency: day,
                published: day,
            }],
            buildings: Some(record.clone()),
            land_use: Some(record.clone()),
            berlin_layers: BTreeMap::from([("a.geojson".to_string(), record.clone())]),
            nas_layers: BTreeMap::from([("b.geojson".to_string(), record.clone())]),
            tree_layers: BTreeMap::from([("c".to_string(), record)]),
            osm: Some(OsmRecord {
                extract: day,
                area: [52.0, 13.0, 52.1, 13.1],
                count: 7,
            }),
            ..project.clone()
        };
        assert_eq!(project_fingerprint(&fetched).unwrap(), fingerprint);
        // The title is drawn on the sheet only, never in the preview.
        for title in ["Clara-Zetkin-Schule", "Another title"] {
            let title = Some(title.to_string());
            assert_ne!(project.title, title);
            let other_title = ProjectFile {
                title,
                ..project.clone()
            };
            assert_eq!(project_fingerprint(&other_title).unwrap(), fingerprint);
        }
        let other_area = ProjectFile {
            map_area: serde_json::json!({
                "type": "Polygon",
                "coordinates": [[[13.0, 52.0], [13.001, 52.0], [13.001, 52.001], [13.0, 52.0]]]
            }),
            ..project.clone()
        };
        assert_ne!(project_fingerprint(&other_area).unwrap(), fingerprint);
        let other_scale = ProjectFile {
            scale_denominator: 2000,
            ..project.clone()
        };
        assert_ne!(project_fingerprint(&other_scale).unwrap(), fingerprint);
        // Every other field that decides the build counts too.
        let changed = [
            ProjectFile {
                map_kind: MapKind::Sprint,
                ..project.clone()
            },
            ProjectFile {
                declination_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                ..project.clone()
            },
            ProjectFile {
                buffer_m: project.buffer_m + 5.0,
                ..project.clone()
            },
            ProjectFile {
                map_area_source_id: Some("way/1".into()),
                ..project.clone()
            },
            ProjectFile {
                crs: 25832,
                ..project.clone()
            },
            ProjectFile {
                provider: Some(AdapterId::built_in("hvbg")),
                ..project.clone()
            },
        ];
        for other in changed {
            assert_ne!(
                project_fingerprint(&other).unwrap(),
                fingerprint,
                "{other:?}"
            );
        }
    }

    #[test]
    fn run_writes_the_preview_only_with_the_map() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let preview = map_preview_path(dir.path());
        // The map export alone writes no preview (the golden net hashes its files).
        let map = export_starter_project(dir.path(), false).unwrap();
        assert!(!preview.exists());
        std::fs::write(&map, "edited in OOM").unwrap();
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");

        let kept = run(dir.path(), &fetcher, &cache, false);
        assert!(matches!(kept.map, MapOutcome::Kept { .. }));
        assert_eq!(kept.preview, None);
        assert!(!preview.exists(), "a kept map gets no preview");
        assert!(read_map_preview_record(dir.path()).is_none());

        let written = run(dir.path(), &fetcher, &cache, true);
        assert_eq!(written.preview.as_ref(), Some(&preview));
        assert!(preview.exists());
        let written = read_map_preview_record(dir.path()).expect("record");
        assert_eq!(
            written.content,
            ContentSettings::with_profile(crate::build::content::Profile::Safe),
            "the settings the test build places with"
        );

        // Another content and the map kept: the record still names the settings of its export.
        let off = ContentSettings::with_profile(crate::build::content::Profile::Full);
        set_content(dir.path(), Some(off)).unwrap();
        let before = std::fs::read(&preview).unwrap();
        run(dir.path(), &fetcher, &cache, false);
        assert_eq!(std::fs::read(&preview).unwrap(), before);
        assert_eq!(read_map_preview_record(dir.path()), Some(written));
    }

    #[test]
    fn a_failed_preview_leaves_no_preview_of_the_old_map() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        run(dir.path(), &fetcher, &cache, true);
        assert!(read_map_preview_record(dir.path()).is_some());
        let map = map_path(dir.path());
        std::fs::write(&map, "edited in OOM").unwrap();
        // The next preview cannot be written: a folder stands where the image goes.
        let preview = map_preview_path(dir.path());
        std::fs::remove_file(&preview).unwrap();
        std::fs::create_dir(&preview).unwrap();
        std::fs::write(preview.join("keep"), "").unwrap();

        // The map is written, so the run ends well and says that the preview is missing.
        let mut notes = Vec::new();
        let outcome = run_project(
            dir.path(),
            &fetcher,
            &cache,
            FetchMode::Online,
            true,
            &mut |note| notes.push(note),
        )
        .unwrap();
        assert_eq!(outcome.map, MapOutcome::Written(map.clone()));
        assert_eq!(outcome.preview, None);
        let warnings: Vec<String> = notes
            .iter()
            .filter(|n| n.is_warning())
            .map(ToString::to_string)
            .collect();
        assert!(
            warnings
                .iter()
                .any(|w| w.starts_with("warning: the map is written, but its preview is missing: ")),
            "{warnings:?}"
        );
        assert_ne!(
            std::fs::read_to_string(&map).unwrap(),
            "edited in OOM",
            "the map was replaced"
        );
        assert!(read_map_preview_record(dir.path()).is_none());
        assert!(!dir.path().join("data/map-preview.json").exists());
    }

    /// The day after the export, a run that keeps the map fetches again and dates the records
    /// anew: the preview still belongs to the saved project, so live previews go on. A new area
    /// does not fit it.
    #[test]
    fn a_kept_map_on_a_later_day_keeps_its_preview_fresh() {
        let at = |day| {
            NaiveDate::from_ymd_opt(2026, 11, day)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        };
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rostock_school(&fetcher);
        let file = dir.path().join(PROJECT_FILE_NAME);
        let fresh = || {
            let record = stored_map_preview_record(dir.path()).expect("a stored preview");
            record.fingerprint
                == project_fingerprint(&ProjectFile::load(dir.path()).unwrap()).unwrap()
        };
        let exported = crate::clock::with_fixed(at(12), || run(dir.path(), &fetcher, &cache, true));
        assert!(exported.preview.is_some());
        assert!(fresh());
        let before = std::fs::read_to_string(&file).unwrap();

        let kept = crate::clock::with_fixed(at(13), || run(dir.path(), &fetcher, &cache, false));
        assert!(matches!(kept.map, MapOutcome::Kept { .. }));
        assert_ne!(
            std::fs::read_to_string(&file).unwrap(),
            before,
            "the fetch dated its records anew"
        );
        assert!(fresh(), "a later fetch keeps the preview fresh");
        // A fetch alone on yet another day, too.
        crate::clock::with_fixed(at(14), || fetch(dir.path(), &fetcher, &cache));
        assert!(fresh());

        set_map_area(dir.path(), &fixture_area()).unwrap();
        assert!(!fresh(), "another area needs a run that replaces the map");
    }

    /// The stored preview and its record are read only as files of the project folder: a link
    /// in their place, or a linked `data/` folder, is no preview.
    #[cfg(unix)]
    #[test]
    fn a_linked_preview_or_record_is_no_preview() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = derived_school(&fetcher);
        let documents = starter_documents(dir.path()).unwrap();
        export_map_preview_from(dir.path(), &documents).unwrap();
        let image = map_preview_path(dir.path());
        let record = dir.path().join("data/map-preview.json");
        assert!(stored_map_preview_record(dir.path()).is_some());
        assert_eq!(
            read_map_preview(dir.path()),
            Some(std::fs::read(&image).unwrap())
        );
        let outside = tempfile::tempdir().unwrap();
        let elsewhere = |name: &str, from: &Path| {
            let target = outside.path().join(name);
            std::fs::rename(from, &target).unwrap();
            std::os::unix::fs::symlink(&target, from).unwrap();
            target
        };

        // The image is a link to a file elsewhere.
        let target = elsewhere("image.webp", &image);
        assert!(read_map_preview_record(dir.path()).is_some());
        assert_eq!(stored_map_preview_record(dir.path()), None);
        assert_eq!(read_map_preview(dir.path()), None);
        std::fs::remove_file(&image).unwrap();
        std::fs::rename(&target, &image).unwrap();
        assert!(read_map_preview(dir.path()).is_some());

        // The record is a link.
        let target = elsewhere("record.json", &record);
        assert_eq!(read_map_preview_record(dir.path()), None);
        assert_eq!(read_map_preview(dir.path()), None);
        std::fs::remove_file(&record).unwrap();
        std::fs::rename(&target, &record).unwrap();
        assert!(read_map_preview(dir.path()).is_some());

        // The whole `data/` folder lies elsewhere.
        let data = dir.path().join("data");
        elsewhere("data", &data);
        assert_eq!(read_map_preview_record(dir.path()), None);
        assert_eq!(stored_map_preview_record(dir.path()), None);
        assert_eq!(read_map_preview(dir.path()), None);
    }

    /// The bytes of the map and of the candidates map as written to the `.omap` files.
    fn omap_bytes(documents: &StarterDocuments) -> (Vec<u8>, Vec<u8>) {
        let bytes = |doc: &MapDocument| {
            let mut out = Vec::new();
            write_map(doc, &mut out, XmlFormat::Compact).unwrap();
            out
        };
        (bytes(&documents.map), bytes(&documents.candidates))
    }

    /// A sprint project with the school building, a carriageway (on the map) and a way (a
    /// candidate) of the land use, and an OSM fence, a track and a path.
    fn sprint_with_paving_and_fence(
        fetcher: &MapFetcher,
    ) -> (tempfile::TempDir, tempfile::TempDir) {
        let (dir, cache_dir) = sprint_with_ways(fetcher);
        store_traffic_areas(dir.path());
        let c = fixture_area().centroid();
        let at = |dlat: f64, dlon: f64| LatLon {
            lat: c.lat + dlat,
            lon: c.lon + dlon,
        };
        let way = |id: &str, tags: &[(&str, &str)], points: Vec<LatLon>| {
            crate::sources::osm::OsmFeature {
                id: id.to_string(),
                tags: tags
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                geometry: crate::sources::osm::OsmGeometry::Line(points),
            }
        };
        let features = [
            way(
                "way/1",
                &[("highway", "track"), ("surface", "gravel")],
                vec![at(0.0, 0.0), at(0.0003, 0.0003)],
            ),
            way(
                "way/2",
                &[("highway", "path")],
                vec![at(0.0, 0.0), at(0.0003, 0.0003)],
            ),
            way(
                "way/3",
                &[("barrier", "fence")],
                vec![at(-0.0002, -0.0003), at(-0.0002, 0.0003)],
            ),
        ];
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&features),
        )
        .unwrap();
        (dir, cache_dir)
    }

    #[test]
    fn placing_the_built_project_gives_the_starter_documents() {
        use crate::build::content::Profile;
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_paving_and_fence(&fetcher);
        let content = ContentSettings::with_profile(Profile::Balanced);
        set_content(dir.path(), Some(content.clone())).unwrap();
        let documents = starter_documents(dir.path()).unwrap();
        // What the fixture holds: a building, paving with its border, a fence.
        let set = MapKind::Sprint.symbol_set(4000);
        let all: Vec<&MapObject> = documents
            .map
            .parts
            .iter()
            .chain(&documents.candidates.parts)
            .flat_map(|p| &p.objects)
            .collect();
        let has = |code: &str| {
            let id = set.id_for_code(code).unwrap();
            all.iter().any(|o| o.symbol_id == id)
        };
        assert!(has("521") && has("501.5") && has("518"));
        let edge = set.id_for_code("501.1").unwrap();
        assert!(all.iter().any(|o| {
            o.symbol_id == edge
                && o.tags
                    .iter()
                    .any(|(k, v)| k == "freemapper:source" && v == GENERALIZATION_SOURCE)
        }));

        let built = build_project(dir.path()).unwrap();
        assert_eq!(
            built.fingerprint(),
            project_fingerprint(&ProjectFile::load(dir.path()).unwrap()).unwrap()
        );
        let placed = place(&built, &content).unwrap();
        assert!(omap_bytes(&placed) == omap_bytes(&documents), "same files");
        assert_eq!(placed.attribution_text, documents.attribution_text);
        assert_eq!(placed.content, documents.content);
        assert_eq!(placed.info, documents.info);
        assert_eq!(placed.content_settings, content);
    }

    #[test]
    fn the_built_project_is_placed_by_any_content_settings() {
        use crate::build::content::{Group, Place, Profile};
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_paving_and_fence(&fetcher);
        let built = build_project(dir.path()).unwrap();
        let map_part = |d: &StarterDocuments| {
            d.map
                .parts
                .iter()
                .find(|p| p.name == "default part")
                .unwrap()
                .objects
                .len()
        };
        let with = |profile, group: Option<(Group, Place)>| {
            let mut settings = ContentSettings::with_profile(profile);
            if let Some((group, place)) = group {
                settings.set(group, Some(place));
            }
            settings
        };
        // The build computed the borders for the project's own settings (`safe` in tests).
        assert_eq!(built.borders_computed(), 1);
        let walls_off = with(Profile::Safe, Some((Group::Walls, Place::Off)));
        for settings in [ContentSettings::with_profile(Profile::Safe), walls_off] {
            place(&built, &settings).unwrap();
            assert_eq!(
                built.borders_computed(),
                1,
                "{settings:?} has the same streets"
            );
        }
        let streets_off = with(Profile::Safe, Some((Group::Streets, Place::Off)));
        place(&built, &streets_off).unwrap();
        assert_eq!(built.borders_computed(), 2);
        place(&built, &streets_off).unwrap();
        assert_eq!(built.borders_computed(), 2, "computed once per input");
        let safe = place(&built, &ContentSettings::with_profile(Profile::Safe)).unwrap();
        let full = place(&built, &ContentSettings::with_profile(Profile::Full)).unwrap();
        assert_ne!(map_part(&safe), map_part(&full));

        // Each placement equals a build from scratch with those settings saved, also where the
        // streets move (their borders are computed again) and when asked twice.
        for settings in [
            with(Profile::Balanced, None),
            with(Profile::Full, None),
            with(Profile::Balanced, Some((Group::Streets, Place::Off))),
            with(Profile::Safe, Some((Group::Streets, Place::Candidates))),
            with(Profile::Full, Some((Group::Fences, Place::Off))),
            with(Profile::Safe, None),
            with(Profile::Full, None),
        ] {
            set_content(dir.path(), Some(settings.clone())).unwrap();
            let fresh = starter_documents(dir.path()).unwrap();
            let placed = place(&built, &settings).unwrap();
            assert!(omap_bytes(&placed) == omap_bytes(&fresh), "{settings:?}");
            assert_eq!(placed.content_settings, settings);
        }
    }

    fn hex(text: &str) -> String {
        text.bytes().map(|b| format!("{b:02X}")).collect()
    }

    /// Whether the content stream of the PDF shows `text` (ASCII only).
    fn pdf_shows(pdf: &std::path::Path, text: &str) -> bool {
        let content = crate::render::pdf::content_of(&std::fs::read(pdf).unwrap());
        contains(&content, hex(text).as_bytes())
    }

    #[test]
    fn pdf_source_block_follows_the_mode_and_the_credit_stays() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let full_list = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).ok();
        let credit = "freemapper";
        for scale_bar in [false, true] {
            for sources in [PdfSources::Auto, PdfSources::Full, PdfSources::CreditOnly] {
                for candidates in [false, true] {
                    let path = if candidates {
                        export_candidates_pdf(dir.path(), scale_bar, sources).unwrap()
                    } else {
                        export_pdf(dir.path(), scale_bar, sources).unwrap()
                    };
                    // The credit line with the repository URL is in every mode.
                    assert!(pdf_shows(&path, "Erstellt mit freemapper"), "{sources:?}");
                    assert!(
                        pdf_shows(&path, env!("CARGO_PKG_REPOSITORY")),
                        "{sources:?}"
                    );
                    assert!(pdf_shows(&path, credit));
                    // The title block stays.
                    assert!(pdf_shows(&path, "stab 1:"), "{sources:?}");
                    // Only the data sources go in credit-only.
                    assert_eq!(
                        pdf_shows(&path, "ODbL"),
                        sources != PdfSources::CreditOnly,
                        "{sources:?}"
                    );
                }
            }
        }
        // ATTRIBUTION.txt is not touched by any PDF mode.
        let after = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).ok();
        assert_eq!(full_list, after);
    }

    #[test]
    fn attribution_file_is_complete_whatever_the_pdf_shows() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        export_starter_project(dir.path(), true).unwrap();
        let before = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert!(before.contains("ODbL"));
        export_pdf(dir.path(), false, PdfSources::CreditOnly).unwrap();
        let after = std::fs::read_to_string(dir.path().join("ATTRIBUTION.txt")).unwrap();
        assert_eq!(before, after);
        assert!(after.contains(FREEMAPPER_CREDIT));
    }

    #[test]
    fn pdf_attribution_modes_keep_the_credit_line_last() {
        let content = PaperRect {
            min_x: 0,
            min_y: 0,
            max_x: 150_000,
            max_y: 100_000,
        };
        let mut lines: Vec<String> = (0..7)
            .map(|i| format!("Geoportal Berlin / Quelle {i}, dl-de/zero-2-0"))
            .collect();
        lines.insert(0, OSM_ATTRIBUTION_DE.to_string());
        lines.push(FREEMAPPER_CREDIT_DE.to_string());
        let credit = || Some(FREEMAPPER_CREDIT_DE.to_string());
        let full = pdf_attribution(&lines, &content, PdfSources::Full);
        assert_eq!(full, lines);
        let only = pdf_attribution(&lines, &content, PdfSources::CreditOnly);
        assert_eq!(only, [FREEMAPPER_CREDIT_DE]);
        let auto = pdf_attribution(&lines, &content, PdfSources::Auto);
        assert!(auto.len() < lines.len());
        assert_eq!(auto.first().map(String::as_str), Some(OSM_ATTRIBUTION_DE));
        assert_eq!(auto.last().cloned(), credit());
        // A list that already fits is unchanged.
        let short = vec![
            OSM_ATTRIBUTION_DE.to_string(),
            FREEMAPPER_CREDIT_DE.to_string(),
        ];
        assert_eq!(pdf_attribution(&short, &content, PdfSources::Auto), short);
    }

    #[test]
    fn scale_bar_is_off_in_the_pdf_unless_asked() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let content = |bar: bool| {
            let path = export_pdf(dir.path(), bar, PdfSources::Auto).unwrap();
            crate::render::pdf::content_of(&std::fs::read(path).unwrap())
        };
        let without = content(false);
        let with = content(true);
        assert!(with.len() > without.len(), "the bar adds drawing operators");
        // The scale number in the title block stays in both.
        let label: String = "stab 1:".bytes().map(|b| format!("{b:02X}")).collect();
        assert!(contains(&without, label.as_bytes()));
        assert!(contains(&with, label.as_bytes()));
        // The OOM map keeps the bar.
        let documents = starter_documents(dir.path()).unwrap();
        assert!(
            documents
                .map
                .parts
                .iter()
                .flat_map(|p| &p.objects)
                .any(is_scale_bar)
        );
    }

    #[test]
    fn pdf_names_sit_next_to_each_other() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let plain = pdf_path(dir.path(), false);
        let cand = pdf_path(dir.path(), true);
        assert_eq!(plain.parent(), cand.parent());
        let stem = plain.file_stem().unwrap().to_string_lossy().into_owned();
        assert_eq!(
            cand.file_name().unwrap().to_string_lossy(),
            format!("{stem}-candidates.pdf")
        );
    }

    #[test]
    fn pdf_without_data_still_has_the_sheet() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let file = std::fs::read(export_pdf(dir.path(), false, PdfSources::Auto).unwrap()).unwrap();
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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

    /// Writes the fixture of the CLI tests that run offline (`crates/freemapper-cli/tests/
    /// fixtures/offline-school/`): the fetched synthetic school project and its cache. Run it
    /// again when the layout of the project or the cache changes: `cargo test -p freemapper
    /// --lib write_the_offline_cli_fixture -- --ignored`.
    #[test]
    #[ignore = "a tool for the fixture of the CLI tests, not a test"]
    fn write_the_offline_cli_fixture() {
        fn copy_tree(from: &Path, to: &Path) {
            std::fs::create_dir_all(to).unwrap();
            for entry in std::fs::read_dir(from).unwrap().flatten() {
                let target = to.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    std::fs::copy(entry.path(), target).unwrap();
                }
            }
        }
        let out = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../freemapper-cli/tests/fixtures/offline-school");
        let _ = std::fs::remove_dir_all(&out);
        let fetcher = MapFetcher::default();
        let (dir, cache_dir, _cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.title = Some("Offline school".to_string());
        project.save(dir.path()).unwrap();
        copy_tree(dir.path(), &out.join("project"));
        copy_tree(cache_dir.path(), &out.join("cache"));
    }

    /// `cache` on one volume (with the project folder) that is `short` bytes too small for the
    /// next fetch of the project in `dir`: its downloads, the files derived from them and the
    /// reserve.
    fn cache_short_of(dir: &Path, cache: &Cache, short: u64) -> Cache {
        cache_short_of_kept(dir, cache, short, |_| true)
    }

    /// Runs `f` on a day on which the OSM extract of the fixtures (state of 2026-11-06) is still
    /// current. The tests whose outcome depends on whether that extract is due for an update
    /// run on it; on the real day they began to fail a week after the fixture was made.
    fn while_the_extract_is_current<R>(f: impl FnOnce() -> R) -> R {
        let at = NaiveDate::from_ymd_opt(2026, 11, 8)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        crate::clock::with_fixed(at, f)
    }

    /// [`cache_short_of`] the parts `kept` says: the others do not fit at all.
    fn cache_short_of_kept(
        dir: &Path,
        cache: &Cache,
        short: u64,
        kept: impl Fn(Part) -> bool,
    ) -> Cache {
        use crate::space::test_support::FixedSpace;
        let today = crate::clock::today();
        let mut estimate = crate::layers::estimate_download(dir, cache, today).unwrap();
        estimate.items.retain(|item| kept(item.part));
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

    /// A Brandenburg school project served with LGB tiles and the nDSM point cloud.
    fn strausberg_school(fetcher: &MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        serve_synthetic_tile(fetcher, school_tile(), "2026-01-26");
        crate::fetch::test_support::serve_synthetic_ndsm(fetcher, school_tile());
        (dir, cache_dir, cache)
    }

    /// The raster fetch of every provider gets `skip_surface` the same way (through the
    /// environment of the fetch): on a disk without room for the surface model it asks for
    /// nothing of it, says so, and records none.
    #[test]
    fn skip_surface_reaches_every_backend() {
        type School = fn(&MapFetcher) -> (tempfile::TempDir, tempfile::TempDir, Cache);
        let providers: [(AdapterId, School); 10] = [
            (providers::LGB, strausberg_school),
            (providers::GEO_SN, dresden_school),
            (providers::LVERM_GEO, halle_school),
            (providers::LGV, eppendorf_school),
            (providers::HVBG, frankfurt_school),
            (providers::LVERM_GEO_SH, norderstedt_school),
            (providers::LAIV_MV, rostock_school),
            (providers::GEOBASIS_NRW, koeln_school),
            (providers::LGLN, hannover_school),
            (providers::LVERM_GEO_RP, koblenz_school),
        ];
        for (provider, school) in providers {
            // With room for everything: the requests of a whole fetch.
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = school(&fetcher);
            let (whole, _) = fetch(dir.path(), &fetcher, &cache);
            assert_eq!(whole.provider, Some(provider.clone()));
            let all: Vec<String> = fetcher.log.borrow().clone();
            // One byte short: the surface model is the first part to go after the point clouds.
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = school(&fetcher);
            let cache = cache_short_of_kept(dir.path(), &cache, 1, |p| p != Part::PointCloud);
            let (project, messages) = fetch(dir.path(), &fetcher, &cache);
            assert!(
                messages
                    .iter()
                    .any(|m| m.starts_with("warning: freemapper skips the surface model")),
                "{provider:?}: {messages:?}"
            );
            let asked: Vec<String> = fetcher.log.borrow().clone();
            let extra: Vec<&String> = asked.iter().filter(|url| !all.contains(url)).collect();
            assert!(extra.is_empty(), "{provider:?} asked for more: {extra:?}");
            let left_out = all.iter().filter(|url| !asked.contains(url)).count();
            assert!(
                left_out > 0,
                "{provider:?} still asked for the surface model"
            );
            assert!(project.surface.is_none(), "{provider:?}");
            assert!(project.ndsm_tiles.is_empty(), "{provider:?}");
            assert!(
                !project
                    .tiles
                    .iter()
                    .any(|r| matches!(r.product, Product::ImageDsm | Product::SaxonyDsm)),
                "{provider:?}"
            );
            // What the run needs is there.
            assert!(
                project.orthophoto.is_some()
                    || project
                        .tiles
                        .iter()
                        .any(|r| r.product == Product::Orthophoto),
                "{provider:?}"
            );
        }
    }

    /// The checkpoint of a raster fetch writes the project file. When that fails (a folder that
    /// is gone, a full disk, a link that leads out of the project), the fetch ends with the same
    /// error as before the raster fetch took a `RasterProject`.
    #[test]
    fn a_failed_save_in_a_raster_fetch_is_the_write_error() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let region_bbox = project.region_bbox().unwrap();
        let gone = dir.path().join("gone");
        let direct = project.save(&gone).unwrap_err();
        let mut in_dir = ProjectDir {
            dir: &gone,
            project: &mut project,
            region_bbox,
        };
        let through = ProjectError::from(RasterProject::save(&mut in_dir).unwrap_err());
        assert!(
            matches!(through, ProjectError::Fetch(FetchError::Io(..))),
            "{through:?}"
        );
        assert_eq!(through.to_string(), direct.to_string());
    }

    /// The raster fetch ends with an `AdapterError`; the engine turns it back into the error it
    /// stands for, with the same words, so the caller words it as before.
    #[test]
    fn adapter_errors_map_back_to_the_same_project_error() {
        let io = |kind| std::io::Error::from(kind);
        let cases: Vec<(AdapterError, ProjectError)> = vec![
            (
                AdapterError::Fetch(FetchError::NotFound("https://example.invalid/x".into())),
                ProjectError::Fetch(FetchError::NotFound("https://example.invalid/x".into())),
            ),
            (
                AdapterError::Fetch(FetchError::Cancelled),
                ProjectError::Fetch(FetchError::Cancelled),
            ),
            (
                AdapterError::Io(
                    PathBuf::from("data/dgm1.tif"),
                    io(std::io::ErrorKind::NotFound),
                ),
                ProjectError::Io(
                    PathBuf::from("data/dgm1.tif"),
                    io(std::io::ErrorKind::NotFound),
                ),
            ),
            (
                AdapterError::Io(
                    PathBuf::from("data/dgm1.tif"),
                    io(std::io::ErrorKind::StorageFull),
                ),
                ProjectError::Io(
                    PathBuf::from("data/dgm1.tif"),
                    io(std::io::ErrorKind::StorageFull),
                ),
            ),
            (
                AdapterError::ProjectFile("missing field".into()),
                ProjectError::Toml("missing field".into()),
            ),
            (
                AdapterError::OrthophotoNotFetched,
                ProjectError::OrthophotoNotFetched,
            ),
            (
                AdapterError::TerrainNotFetched,
                ProjectError::TerrainNotFetched,
            ),
            (
                AdapterError::Engine(Box::new(ProjectError::NotFetched)),
                ProjectError::NotFetched,
            ),
        ];
        for (adapter, expected) in cases {
            let words = adapter.to_string();
            let back = ProjectError::from(adapter);
            assert_eq!(
                std::mem::discriminant(&back),
                std::mem::discriminant(&expected),
                "{back:?}"
            );
            assert_eq!(back.to_string(), expected.to_string());
            assert_eq!(words, expected.to_string());
            if let (ProjectError::Fetch(back), ProjectError::Fetch(expected)) = (&back, &expected) {
                assert_eq!(
                    std::mem::discriminant(back),
                    std::mem::discriminant(expected)
                );
            }
        }
    }

    #[test]
    fn the_vegetation_is_as_old_as_its_oldest_tile() {
        let record = |tile: &str, published: Option<&str>| PointCloudRecord {
            tile: tile.to_string(),
            url: String::new(),
            published: published.map(|day| day.parse().unwrap()),
        };
        // A lot flown in 2021 beside one made in 2018, and a tile that names no day.
        let tiles = [
            record("32655-5479", Some("2021-12-20")),
            record("32656-5479", Some("2018-08-08")),
            record("32657-5479", None),
        ];
        assert_eq!(
            export::vegetation_currency(&tiles),
            "2018-08-08".parse().ok()
        );
        assert_eq!(export::vegetation_currency(&tiles[2..]), None);
        assert_eq!(export::vegetation_currency(&[]), None);
    }

    #[test]
    fn derive_leaves_out_the_classes_the_provider_of_the_point_clouds_names() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 14).unwrap();
        let classes = |e: f64, n: f64, vegetation: bool| {
            let dir = tempfile::tempdir().unwrap();
            let area = utm32_area_at(e, n, 100.0);
            init_project(dir.path(), &area, MapKind::Schoolyard, None, date, None).unwrap();
            if vegetation {
                want_vegetation(dir.path());
            }
            let project = ProjectFile::load(dir.path()).unwrap();
            left_out_classes(&project, &project.region_bbox().unwrap()).to_vec()
        };
        // The Schmausenbuck in Bavaria: cellar exits and synthetic ground.
        assert_eq!(classes(655_700.0, 5_479_700.0, true), [22, 23]);
        // Cologne: Geobasis NRW names no class of its own.
        assert!(classes(354_700.0, 5_642_650.0, true).is_empty());
        // Without the vegetation chosen no point clouds are read.
        assert!(classes(655_700.0, 5_479_700.0, false).is_empty());
    }

    /// A state without point clouds: nothing is asked for, no vegetation file, and a line only
    /// when the vegetation is chosen.
    #[test]
    fn a_state_without_point_clouds_has_no_vegetation_step() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = dresden_school(&fetcher);
        let (project, mut messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.point_clouds.is_empty());
        assert!(
            !fetcher.log.borrow().iter().any(|url| url.contains("/als/")),
            "{:?}",
            fetcher.log.borrow()
        );
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(!dir.path().join(VEGETATION_RELPATH).exists());
        assert!(
            !messages
                .iter()
                .any(|m| m.contains("laser point cloud") || m.contains("vegetation")),
            "{messages:?}"
        );
        // Chosen there, the vegetation cannot come: the fetch says so once.
        want_vegetation(dir.path());
        let (_, messages) = fetch(dir.path(), &fetcher, &cache);
        let said = messages
            .iter()
            .filter(|m| m.contains("no laser point clouds for the vegetation here yet"))
            .count();
        assert_eq!(said, 1, "{messages:?}");
    }

    #[test]
    fn parts_that_do_not_fit_are_skipped_with_a_warning() {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        want_vegetation(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        // The point clouds go first; one byte short of the rest, the surface model goes too.
        let cache = cache_short_of_kept(
            dir.path(),
            &Cache::new(cache_dir.path().to_path_buf()),
            1,
            |part| part != Part::PointCloud,
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert!(
            messages.iter().any(|m| m.starts_with(
                "warning: freemapper skips the laser point clouds for the vegetation candidates"
            )),
            "{messages:?}"
        );
        assert!(project.point_clouds.is_empty());
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
        let ndsm_url = freemapper_adapters_de::lgb::ndsm_tile_url(school_tile());
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
        while_the_extract_is_current(|| {
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
        });
    }

    #[test]
    fn skipped_surface_model_keeps_the_cached_geosn_tiles() {
        while_the_extract_is_current(|| {
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
        });
    }

    #[test]
    fn skipped_surface_model_of_hamburg_keeps_the_stored_one() {
        while_the_extract_is_current(|| {
            use freemapper_adapters_de::lgv::Model;
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
        });
    }

    /// Hamburg reads the surface model for the area. When that fails, a stored one that covers
    /// the area stays and the warning says so; one of another area goes with its file.
    #[test]
    fn hamburg_bdom_failure_keeps_a_covering_surface_and_drops_another() {
        use freemapper_adapters_de::lgv::Model;
        for shrink in [false, true] {
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = eppendorf_school(&fetcher);
            fetch(dir.path(), &fetcher, &cache);
            if shrink {
                let mut project = ProjectFile::load(dir.path()).unwrap();
                project.surface.as_mut().unwrap().bbox[2] -= 50.0;
                project.save(dir.path()).unwrap();
            }
            // A fresh cache: the bDOM must be read again, and its archive fails.
            let cache_dir = tempfile::tempdir().unwrap();
            let cache = Cache::new(cache_dir.path().to_path_buf());
            fetcher
                .failing
                .borrow_mut()
                .push(Model::Bdom.archive_url().to_string());
            let (project, messages) = fetch(dir.path(), &fetcher, &cache);
            let unavailable = format!("warning: {} unavailable", Model::Bdom.label());
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

    /// With everything of the area fetched nothing is left to download: 0 MB, not the -0.0
    /// that the sum of nothing is.
    #[test]
    fn an_estimate_with_everything_fetched_totals_zero() {
        while_the_extract_is_current(|| {
            let fetcher = MapFetcher::default();
            let (dir, _cache_dir, cache) = strausberg_school(&fetcher);
            fetch(dir.path(), &fetcher, &cache);
            let today = crate::clock::today();
            let estimate = crate::layers::estimate_download(dir.path(), &cache, today).unwrap();
            assert!(
                estimate.items.iter().all(|i| i.cached),
                "{:?}",
                estimate.items
            );
            assert_eq!(estimate.total_megabytes.to_string(), "0");
        });
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
        while_the_extract_is_current(|| {
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
        });
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
            freemapper_adapters_de::berlin::BUILDINGS_SERVICE,
            freemapper_adapters_de::berlin::test_support::BUILDINGS
                .as_bytes()
                .to_vec(),
        );
        fetcher.insert_prefix(
            freemapper_adapters_de::berlin::LAND_USE_SERVICE,
            freemapper_adapters_de::berlin::test_support::LAND_USE
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
        assert_eq!(project.buildings.unwrap().regions, [regions::SAXONY]);
        assert_eq!(project.land_use.unwrap().regions, [regions::SAXONY]);
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
                .all(|r| r.product.provider() == freemapper_adapters_de::lgb::TileProvider::Lgb)
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
            &freemapper_sources::wfs::GEOSN.url,
            include_bytes!("../../../tests/fixtures/saxony-empty.gml").to_vec(),
        );
        let saxony = crate::sources::osm::extract_of("saxony", 0);
        fetcher.insert(
            &saxony.state_url(),
            b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
        );
        fetcher.insert(
            &saxony.url(),
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
        use freemapper_sources::wfs::{self as alkis_wfs, test_support as fixtures};
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
                &extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                &extract.url(),
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
        assert_eq!(project.provider, Some(providers::GEOBASIS_NRW));
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
            [regions::NORDRHEIN_WESTFALEN]
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
        use freemapper_adapters_de::geobasis_nrw::{Model, meta_url};
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

    /// Serves the NRW point-cloud index and one tile under the Cologne school: dense bushes 1.6 m
    /// above the synthetic terrain on 30 m × 30 m west of the school.
    fn serve_nrw_point_cloud(fetcher: &MapFetcher) {
        use freemapper_adapters_de::nrw_3dm::{INDEX_URL, tile_url};
        use las::point::Classification;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 4));
        builder.point_format.is_compressed = true;
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(300_000.0),
            y: transform(5_600_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        // The terrain of `serve_synthetic_nrw` rises 0.01 m per metre east from 20 m at the tile's
        // west edge.
        let terrain = |e: f64| 20.0 + (e - 354_000.0) * 0.01;
        for i in 0..60 {
            for j in 0..60 {
                let (e, n) = (
                    354_660.25 + f64::from(i) * 0.5,
                    5_642_610.25 + f64::from(j) * 0.5,
                );
                let ground = (i + j) % 10 == 0;
                writer
                    .write_point(las::Point {
                        x: e,
                        y: n,
                        z: terrain(e) + if ground { 0.0 } else { 1.6 },
                        classification: if ground {
                            Classification::Ground
                        } else {
                            Classification::new(20).unwrap()
                        },
                        number_of_returns: 2,
                        return_number: 2,
                        ..Default::default()
                    })
                    .unwrap();
            }
        }
        writer.close().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let name = "3dm_32_354_5642_1_nw.laz";
        fetcher.insert(
            INDEX_URL,
            serde_json::json!({"datasets": [{"files": [
                {"name": name, "size": bytes.len().to_string(), "timestamp": "2023-07-19T07:30:49"}
            ]}]})
            .to_string()
            .into_bytes(),
        );
        fetcher.insert(&tile_url(name), bytes);
    }

    #[test]
    fn nrw_vegetation_comes_from_its_own_point_clouds_in_zone_32() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = koeln_school(&fetcher);
        serve_nrw_point_cloud(&fetcher);
        want_vegetation(dir.path());
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.point_clouds.len(), 1);
        assert_eq!(project.point_clouds[0].tile, "32354-5642");
        assert_eq!(project.point_clouds[0].published, "2023-07-19".parse().ok());
        assert!(
            !fetcher
                .log
                .borrow()
                .iter()
                .any(|u| u.contains("geobasis-bb")),
            "no Brandenburg point cloud for a map in zone 32"
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let codes = vegetation_codes(dir.path());
        assert!(
            codes.iter().any(|(c, e, n)| c == "410"
                && (354_660.0..354_690.0).contains(e)
                && (5_642_610.0..5_642_640.0).contains(n)),
            "{codes:?}"
        );
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
        assert_eq!(project.provider, Some(providers::GEOBASIS_NRW));
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
        assert_eq!(project.provider, Some(providers::GEOBASIS_NRW));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [regions::HESSEN, regions::NORDRHEIN_WESTFALEN]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                freemapper_sources::wfs::HVBG.url,
                freemapper_sources::wfs::GEOBASIS_NRW.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(&*service)), "{service}");
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
                freemapper_adapters_de::geobasis_nrw::WMS.url
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
                freemapper_adapters_de::geobasis_nrw::WMS.url
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
        for (_, url) in
            crate::sources::wms::pieces(&freemapper_adapters_de::geobasis_nrw::WMS, &photo)
        {
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
        use freemapper_sources::wfs::{LGLN, test_support as fixtures};
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
        use freemapper_adapters_de::lgv;
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
                &extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                &extract.url(),
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
        use freemapper_adapters_de::lgln;
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = hannover_school(&fetcher);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.crs(), crate::crs::Crs::Utm32);
        assert_eq!(project.provider, Some(providers::LGLN));
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
        assert_eq!(project.buildings.unwrap().regions, [regions::NIEDERSACHSEN]);
        {
            let log = fetcher.log.borrow();
            let getmap: Vec<&String> = log
                .iter()
                .filter(|u| u.starts_with(&*lgln::WMS.url) && u.contains("REQUEST=GetMap"))
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
                        .any(|u| u.starts_with(&*model.wcs().url) && u.contains(coverage)),
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
        use freemapper_adapters_de::lgln::{self, test_support::stac_page};
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
        use freemapper_adapters_de::lgln::{self, test_support::stac_page};
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
            .filter(|u| u.starts_with(&*lgln::WMS.url) && u.contains("REQUEST=GetMap"))
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

    /// Stuhr-Brinkum on the border to Bremen-Huchting (tile 32 485 5876, 44 % Bremen): the
    /// rasters come from Lower Saxony, where the centre lies; buildings, land use and the
    /// layers of the full ALKIS are asked from Bremen as well. Until Bremen became a region
    /// this area got the warning of a part outside the supported states.
    #[test]
    fn ni_area_reaching_into_bremen_asks_both_states() {
        use freemapper_adapters_de::geobremen;
        let area = utm32_area_at(485_500.0, 5_876_500.0, 300.0);
        assert_eq!(
            raster_region(&area.projected_bbox()),
            Some(regions::NIEDERSACHSEN)
        );
        let fetcher = MapFetcher::default();
        // The full ALKIS of Bremen answers every type with no object.
        fetcher.insert_prefix(
            geobremen::ALKIS_URL,
            freemapper_sources::nas::test_support::EMPTY
                .as_bytes()
                .to_vec(),
        );
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &area);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::LGLN));
        assert!(
            !messages.iter().any(|m| m == BEYOND_COVERAGE_WARNING),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("reaches beyond the state border")),
            "{messages:?}"
        );
        let asked = fetcher.log.borrow();
        for type_name in [
            "adv:AX_Gebaeude",
            "adv:AX_Platz",
            "adv:AX_Vegetationsmerkmal",
        ] {
            let wanted = format!(
                "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&",
                geobremen::ALKIS_URL
            );
            assert!(
                asked.iter().any(|url| url.starts_with(&wanted)),
                "{type_name}"
            );
        }
    }

    #[test]
    fn ni_area_reaching_into_thuringia_is_warned() {
        // Gerblingerode in Lower Saxony, on the border to Teistungen in Thuringia, which is no
        // region yet: the first 600 m area north of Teistungen whose centre lies in
        // Niedersachsen.
        let (e, n) = (0..60)
            .map(|step| (588_349.0, 5_702_860.0 + f64::from(step) * 50.0))
            .find(|&(e, n)| {
                raster_region(&utm32_area_at(e, n, 300.0).projected_bbox())
                    == Some(regions::NIEDERSACHSEN)
            })
            .expect("a centre in Niedersachsen");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::LGLN));
        assert_eq!(
            messages
                .iter()
                .filter(|m| m.as_str() == BEYOND_COVERAGE_WARNING)
                .count(),
            1,
            "{messages:?}"
        );
        // Thuringia is not a region: the border message between two supported states stays
        // out.
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
            Some(regions::NIEDERSACHSEN)
        );
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &area);
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::LGLN));
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
        use freemapper_adapters_de::lgln::{self, test_support::stac_page};
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
            vec![regions::SCHLESWIG_HOLSTEIN, regions::NIEDERSACHSEN];
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
                raster_region(&bbox) == Some(regions::NIEDERSACHSEN)
                    && regions_for(&bbox).contains(&regions::HAMBURG)
            })
            .expect("a border area south of Harburg");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = ni_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::LGLN));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [regions::HAMBURG, regions::NIEDERSACHSEN]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                freemapper_adapters_de::lgv::BUILDINGS_ENDPOINT,
                &freemapper_sources::wfs::LGLN.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(&*service)), "{service}");
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
        use freemapper_sources::wfs::{self as alkis_wfs, test_support as fixtures};
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
                &extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(
                &extract.url(),
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
        assert_eq!(project.provider, Some(providers::LVERM_GEO_RP));
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
        assert_eq!(
            project.buildings.unwrap().regions,
            [regions::RHEINLAND_PFALZ]
        );
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
        use freemapper_adapters_de::lvermgeo_rp::WMS;
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
                raster_region(&bbox) == Some(regions::RHEINLAND_PFALZ)
                    && regions_for(&bbox).contains(&regions::HESSEN)
            })
            .expect("a border area at Mainz");
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = rp_project_of(&fetcher, &utm32_area_at(e, n, 300.0));
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::LVERM_GEO_RP));
        assert!(project.orthophoto.is_some() && project.terrain.is_some());
        assert!(project.surface.is_some());
        assert_eq!(
            project.buildings.unwrap().regions,
            [regions::HESSEN, regions::RHEINLAND_PFALZ]
        );
        {
            let log = fetcher.log.borrow();
            for service in [
                freemapper_sources::wfs::HVBG.url,
                freemapper_sources::wfs::LVERMGEO_RP.url,
            ] {
                assert!(log.iter().any(|u| u.starts_with(&*service)), "{service}");
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
        let koeln = crate::sources::osm::extract_of("nordrheinwestfalen", 3);
        let duesseldorf = crate::sources::osm::extract_of("nordrheinwestfalen", 2);
        let centre = BoundingBox {
            min_e: ce,
            min_n: cn,
            max_e: ce,
            max_n: cn,
        };
        assert_eq!(
            crate::sources::osm::extracts_for(&centre, &mut |_| {}),
            std::slice::from_ref(&koeln)
        );
        assert_eq!(
            crate::sources::osm::extracts_for(&project.region_bbox().unwrap(), &mut |_| {}),
            [duesseldorf.clone(), koeln.clone()]
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
                &extract.state_url(),
                b"timestamp=2026-11-06T21\\:21\\:21Z\n".to_vec(),
            );
            fetcher.insert(&extract.url(), pbf.clone());
        }
        crate::fetch::test_support::serve_synthetic_nrw(&fetcher, &project.bbox().unwrap());
        // No ALKIS objects in this box.
        fetcher.insert_prefix(
            &freemapper_sources::wfs::GEOBASIS_NRW.url,
            br#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="0" numberMatched="0" xmlns:wfs="http://www.opengis.net/wfs/2.0"/>"#.to_vec(),
        );
        let (project, messages) = fetch(dir.path(), &fetcher, &cache);
        assert_eq!(project.provider, Some(providers::GEOBASIS_NRW));
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
        use freemapper_adapters_de::berlin::{EXTRA_LAYERS, test_support::answer, wfs_url};
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
        let grid = GridSpec::covering(
            &project.bbox().unwrap(),
            freemapper_adapters_de::berlin::K5_PIXEL_M,
        );
        for (piece, url) in freemapper_adapters_de::berlin::k5_pieces(&grid) {
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
                        "properties": {"id": "1", freemapper_adapters_de::berlin::TYPE_KEY: layer}
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
        let grid = GridSpec::covering(
            &project.bbox().unwrap(),
            freemapper_adapters_de::berlin::K5_PIXEL_M,
        );
        for (_, url) in freemapper_adapters_de::berlin::k5_pieces(&grid) {
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
        let canopies: Vec<&MapObject> = documents
            .map
            .parts
            .iter()
            .find(|p| p.name == "default part")
            .unwrap()
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
            project
                .map_kind
                .symbol_set(project.scale_denominator)
                .id_for_code("518")
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
            &mut |m: Note| messages.push(m.to_string()),
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
        let lines = &freemapper_adapters_de::berlin::EXTRA_LAYERS[1];
        fetcher
            .failing
            .borrow_mut()
            .push(freemapper_adapters_de::berlin::wfs_url(
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
            .filter(|u| u.starts_with(&*LGB_NAS.url))
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
        assert_eq!(walls.regions, [regions::BRANDENBURG]);
        let stored =
            std::fs::read_to_string(dir.path().join("data/alkis-nas/sonstiges-bauwerk.geojson"))
                .unwrap();
        assert_eq!(
            freemapper_sources::layer_file::parse_layer(&stored)
                .unwrap()
                .len(),
            5
        );
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
                .any(|u| u.starts_with(&*LGB_NAS.url))
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
            regions: vec![regions::BERLIN],
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
        let mut progress = |m: Note| messages.push(m.to_string());
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
            regions: vec![regions::BERLIN],
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
        let berlin = "Berlin ALKIS structures, tree cadastre, street survey and point layers";
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
            &POTSDAM.endpoints[0],
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
        assert_eq!(record.regions, [regions::BRANDENBURG]);
        assert_eq!(
            record.bbox,
            [366_200.0, 5_807_100.0, 367_100.0, 5_807_700.0]
        );
        let stored =
            std::fs::read_to_string(dir.path().join("data/trees/potsdam.geojson")).unwrap();
        assert_eq!(
            freemapper_sources::layer_file::parse_layer(&stored)
                .unwrap()
                .len(),
            3
        );

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
    fn hidden_candidates_stay_out_of_the_candidates_pdf() {
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
            content: PaperRect {
                min_x: 0,
                min_y: 0,
                max_x: 1,
                max_y: 1,
            },
            info: PeripheralInfo {
                title: String::new(),
                scale_denominator: 1000,
                contour_interval_m: 5.0,
                attribution: Vec::new(),
            },
            content_settings: ContentSettings::default(),
        };
        let (map, _) = pdf_objects(&documents, true);
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
        let found = f.run_open(trees, vec![], &[], &[], &[]);
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
        assert_eq!(project.buildings.unwrap().regions, [regions::BERLIN]);
        assert_eq!(project.land_use.unwrap().regions, [regions::BERLIN]);

        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, _) = fetched_school(&fetcher);
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.buildings.unwrap().regions, [regions::BRANDENBURG]);
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
        project.buildings.as_mut().unwrap().regions = vec![regions::BRANDENBURG, regions::BERLIN];
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
            freemapper_adapters_de::berlin::BUILDINGS_SERVICE,
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
            .symbol_set(1000)
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

    /// A forest project at 1:10000 over the school fixture with these OSM ways (two points each
    /// from the centre of the map area) and no land use.
    fn forest_with_ways(
        fetcher: &MapFetcher,
        ways: &[(&str, &[(&str, &str)])],
    ) -> (tempfile::TempDir, tempfile::TempDir) {
        use crate::sources::alkis::test_support::page;
        let (dir, cache_dir, _) = fetched_school(fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.map_kind = MapKind::Forest;
        project.scale_denominator = 10000;
        project.save(dir.path()).unwrap();
        let c = fixture_area().centroid();
        let points = vec![
            c,
            LatLon {
                lat: c.lat + 0.0003,
                lon: c.lon + 0.0003,
            },
        ];
        let features: Vec<_> = ways
            .iter()
            .map(|(id, tags)| crate::sources::osm::OsmFeature {
                id: id.to_string(),
                tags: tags
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                geometry: crate::sources::osm::OsmGeometry::Line(points.clone()),
            })
            .collect();
        std::fs::write(
            dir.path().join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&features),
        )
        .unwrap();
        std::fs::write(dir.path().join(LAND_USE_RELPATH), page(&[], None)).unwrap();
        (dir, cache_dir)
    }

    #[test]
    fn an_explicit_interval_wins_and_zero_is_refused() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir, cache) = fetched_school(&fetcher);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.map_kind = MapKind::Forest;
        project.scale_denominator = 10000;
        project.save(dir.path()).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let stored = || {
            let text = std::fs::read_to_string(dir.path().join(CONTOURS_RELPATH)).unwrap();
            crate::derive::contours::contours_from_geojson(&text).unwrap()
        };
        // The synthetic terrain rises 1 m in 10 m: hilly, 5 m, the forest interval (not stored).
        let (lines, interval) = stored();
        assert_eq!(interval, None);
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| (l.height / 5.0).fract() == 0.0));
        assert_eq!(ProjectFile::load(dir.path()).unwrap().contours, None);
        project.contour_interval_m = Some(2.5);
        project.save(dir.path()).unwrap();
        assert!(
            std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME))
                .unwrap()
                .contains("contour_interval_m = 2.5")
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let (lines, interval) = stored();
        assert_eq!(interval, Some(2.5), "the project's interval wins");
        assert!(lines.iter().any(|l| (l.height / 5.0).fract() != 0.0));
        // The suite reads the traced interval from the sidecar (`[contours] interval_m`).
        let sidecar = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        assert!(
            sidecar.contains("[contours]\ninterval_m = 2.5"),
            "{sidecar}"
        );
        assert_eq!(
            ProjectFile::load(dir.path()).unwrap().contours,
            Some(ContourRecord { interval_m: 2.5 })
        );
        let read = freemapper_contract::Sidecar::parse(&sidecar).unwrap();
        assert_eq!(read.contour_interval_m(), Some(2.5));
        for bad in ["0.0", "-2.5", "nan"] {
            let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME))
                .unwrap()
                .replace(
                    "contour_interval_m = 2.5",
                    &format!("contour_interval_m = {bad}"),
                );
            std::fs::write(dir.path().join(PROJECT_FILE_NAME), text).unwrap();
            let err = ProjectFile::load(dir.path()).unwrap_err();
            assert!(
                matches!(err, ProjectError::InvalidContourInterval(_)),
                "{bad}: {err:?}"
            );
            let text = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME))
                .unwrap()
                .replace(
                    &format!("contour_interval_m = {bad}"),
                    "contour_interval_m = 2.5",
                );
            std::fs::write(dir.path().join(PROJECT_FILE_NAME), text).unwrap();
        }
    }

    /// Lines without a stored interval were traced at the kind's: the margin names that, also
    /// when `freemapper.toml` was changed after the derive.
    #[test]
    fn the_margin_names_the_interval_the_lines_were_traced_at() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = forest_with_ways(&fetcher, &[]);
        std::fs::write(
            dir.path().join(CONTOURS_RELPATH),
            crate::derive::contours::contours_to_geojson(&[], None),
        )
        .unwrap();
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.contour_interval_m = Some(2.5);
        project.save(dir.path()).unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(
            map.contains("Äquidistanz 5 m"),
            "the lines are at the forest's 5 m"
        );
    }

    #[test]
    fn a_forest_project_fetches_no_tree_registers() {
        assert!(fetches_tree_registers(MapKind::Sprint));
        assert!(fetches_tree_registers(MapKind::Schoolyard));
        assert!(!fetches_tree_registers(MapKind::Forest));
    }

    #[test]
    fn the_contour_interval_is_read_and_set() {
        assert_eq!(parse_contour_interval("auto").unwrap(), None);
        assert_eq!(parse_contour_interval(" 2,5 ").unwrap(), Some(2.5));
        assert_eq!(parse_contour_interval("5").unwrap(), Some(5.0));
        for bad in ["0", "-1", "nan", "inf", "fünf", ""] {
            assert!(parse_contour_interval(bad).is_err(), "{bad}");
        }
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let before = std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        let project = set_contour_interval(dir.path(), Some(2.5)).unwrap();
        assert_eq!(project.contour_interval_m, Some(2.5));
        assert_eq!(
            ProjectFile::load(dir.path()).unwrap().contour_interval_m,
            Some(2.5)
        );
        assert!(matches!(
            set_contour_interval(dir.path(), Some(0.0)),
            Err(ProjectError::InvalidContourInterval(_))
        ));
        // Back to the automatic choice: the file is as before.
        set_contour_interval(dir.path(), None).unwrap();
        assert_eq!(
            std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap(),
            before
        );
    }

    #[test]
    fn a_project_file_without_the_key_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let before = std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.contour_interval_m, None);
        project.save(dir.path()).unwrap();
        assert_eq!(
            std::fs::read(dir.path().join(PROJECT_FILE_NAME)).unwrap(),
            before
        );
    }

    #[test]
    fn index_contours_every_fifth_at_2_5_m() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = forest_with_ways(&fetcher, &[]);
        let c = fixture_area().centroid();
        let crs = ProjectFile::load(dir.path()).unwrap().crs();
        let p = crs.forward(c);
        let lines: Vec<_> = [12.5, 15.0, 25.0]
            .iter()
            .enumerate()
            .map(|(i, height)| crate::derive::contours::ContourLine {
                height: *height,
                points: vec![
                    (p.easting - 40.0, p.northing + 10.0 * i as f64),
                    (p.easting + 40.0, p.northing + 10.0 * i as f64),
                ],
            })
            .collect();
        std::fs::write(
            dir.path().join(CONTOURS_RELPATH),
            crate::derive::contours::contours_to_geojson(&lines, Some(2.5)),
        )
        .unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(map.contains("Äquidistanz 2,5 m"), "the margin names 2.5 m");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let set = MapKind::Forest.symbol_set(10000);
        let (contour, index) = (
            set.id_for_code("101").unwrap(),
            set.id_for_code("102").unwrap(),
        );
        let symbol_of = |height: &str| {
            let tag = format!(r#"<t k="freemapper:height">{height}</t>"#);
            let at = candidates.find(&tag).unwrap_or_else(|| panic!("{height}"));
            let open = candidates[..at].rfind("<object ").unwrap();
            let symbol = &candidates[open..at];
            if symbol.contains(&format!(r#"symbol="{index}""#)) {
                "index"
            } else if symbol.contains(&format!(r#"symbol="{contour}""#)) {
                "contour"
            } else {
                "other"
            }
        };
        assert_eq!(symbol_of("12.5"), "index");
        assert_eq!(symbol_of("15"), "contour");
        assert_eq!(symbol_of("25"), "index");
    }

    #[test]
    fn a_forest_map_without_paths_or_land_use_is_written() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = forest_with_ways(&fetcher, &[]);
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        assert!(map.contains(r#"id="ISOM 2017-2""#), "the ISOM set");
        assert!(map.contains(r#"<georeferencing scale="10000""#));
        assert!(!map.contains(r#" code="990""#), "no walkable area");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(candidates.contains(r#"id="ISOM 2017-2""#));
    }

    #[test]
    fn a_forest_map_draws_the_osm_paths_in_isom_classes() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = forest_with_ways(
            &fetcher,
            &[
                ("way/1", &[("highway", "path")]),
                ("way/2", &[("highway", "track"), ("tracktype", "grade1")]),
                ("way/3", &[("highway", "footway"), ("footway", "sidewalk")]),
            ],
        );
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let set = MapKind::Forest.symbol_set(10000);
        for code in ["506", "504"] {
            let id = set.id_for_code(code).unwrap();
            assert!(
                map.contains(&format!(r#"<object type="1" symbol="{id}""#)),
                "{code}"
            );
        }
        assert!(map.contains(r#"<t k="freemapper:source_id">way/1</t>"#));
        assert!(!map.contains("way/3"), "a sidewalk is left out");
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("way/3"));
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
        let paved = MapKind::Sprint
            .symbol_set(4000)
            .id_for_code("501.5")
            .unwrap();
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
        let street = documents
            .map
            .parts
            .iter()
            .find(|p| p.name == "default part")
            .unwrap()
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
            let set = kind.symbol_set(kind.default_scale());
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
        let pond = MapKind::Sprint.symbol_set(4000).id_for_code("301").unwrap();
        assert_eq!(symbols_of(&candidates, "way/10"), [pond]);
        assert_eq!(symbols_of(&candidates, "way/11"), [pond]);
    }

    #[test]
    fn the_default_content_puts_an_osm_pond_on_the_map() {
        // ADR 0100: `balanced` takes OpenStreetMap water; `safe` (what the other export tests
        // run with) leaves the pond among the candidates.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        store_lake_and_ponds(dir.path());
        store_traffic_areas(dir.path());
        set_content(
            dir.path(),
            Some(ContentSettings::with_profile(
                crate::build::content::Profile::Balanced,
            )),
        )
        .unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let map = std::fs::read_to_string(map_path(dir.path())).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        let pond = MapKind::Sprint.symbol_set(4000).id_for_code("301").unwrap();
        assert_eq!(symbols_of(&map, "way/10"), [pond]);
        assert!(symbols_of(&candidates, "way/10").is_empty());
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
        let out_of_bounds = MapKind::Schoolyard
            .symbol_set(1000)
            .id_for_code("520")
            .unwrap();
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
    fn export_puts_unpaved_paths_into_the_sprint_map() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let track = MapKind::Sprint
            .symbol_set(4000)
            .id_for_code("505.2")
            .unwrap();
        assert!(map.contains(r#"<t k="freemapper:source_id">way/1</t>"#));
        assert!(map.contains(&format!(r#"<object type="1" symbol="{track}""#)));
        assert!(!map.contains("way/2"));
    }

    /// A firm-gravel footway from the open into the Straßenverkehr square (421 560–421 590 E,
    /// 5 821 690–720 N): east to west along 5 821 705 N from 421 520 E to 421 575 E.
    fn firm_footway_into_the_street(dir: &Path) {
        let way = crate::sources::osm::OsmFeature {
            id: "way/5".to_string(),
            tags: vec![
                ("highway".to_string(), "footway".to_string()),
                ("surface".to_string(), "compacted".to_string()),
            ],
            geometry: crate::sources::osm::OsmGeometry::Line(vec![
                lat_lon_at(421_520.0, 5_821_705.0),
                lat_lon_at(421_575.0, 5_821_705.0),
            ]),
        };
        std::fs::write(
            dir.join(OSM_CANDIDATES_RELPATH),
            crate::sources::osm::features_to_geojson(&[way]),
        )
        .unwrap();
    }

    #[test]
    fn export_draws_a_firm_gravel_footway_as_a_paved_line_map_object() {
        // ADR 0079: surface=compacted is hard gravel, 501.6, a map object (ADR 0073 had made it
        // a candidate), and no longer in the candidates.
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        firm_footway_into_the_street(dir.path());
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let firm = MapKind::Sprint
            .symbol_set(4000)
            .id_for_code("501.6")
            .unwrap();
        assert_eq!(symbols_of(&map, "way/5"), [firm]);
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        assert!(!candidates.contains("way/5<"));
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
        let id = MapKind::Schoolyard
            .symbol_set(1000)
            .id_for_code("520")
            .unwrap();
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
        let mut project = ProjectFile::load(dir.path()).unwrap();
        // A new project has a title (ADR 0086); older ones have none and still load.
        assert!(project.title.is_some());
        project.title = None;
        project.save(dir.path()).unwrap();
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
        project.title = None;
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
        // The blank title is not kept; the folder name stands in and is written.
        assert_eq!(
            project.title.as_deref(),
            Some(project.display_title(dir.path()).as_str())
        );
        let mut untitled = project.clone();
        untitled.title = None;
        assert_eq!(untitled.display_title(Path::new("/maps/--")), "Karte");
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
        let mut project = ProjectFile::load(&link).unwrap();
        project.title = None;
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
        let mut project = ProjectFile::load(&project_dir).unwrap();
        project.title = None;
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
        // The walkable area: its own part first, one 990 area tagged as the base; the mapper
        // draws into the default part (ADR 0069).
        let base = map
            .find(r#"<part name="base"><objects count="1">"#)
            .unwrap();
        let default = map.find(r#"<part name="default part">"#).unwrap();
        assert!(base < default);
        assert!(map.contains(r#"<parts count="3" current="1">"#));
        assert!(map[base..default].contains(r#"<t k="freemapper:source">base</t>"#));
        let documents = starter_documents(dir.path()).unwrap();
        let base = &documents.map.parts[0];
        assert_eq!(base.name, "base");
        assert_eq!(
            Some(base.objects[0].symbol_id),
            documents.map.symbol_set.id_for_code("990")
        );
        // It is drawn in the training map.
        let (drawn, _) = pdf_objects(&documents, true);
        assert!(drawn.iter().any(|o| std::ptr::eq(*o, &base.objects[0])));
    }

    #[test]
    fn schoolyard_has_no_walkable_area() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let documents = starter_documents(dir.path()).unwrap();
        assert!(documents.map.parts.iter().all(|p| p.name != "base"));
        assert_eq!(documents.map.symbol_set.id_for_code("990"), None);
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
        let (map, sheet) = pdf_objects(&documents, true);
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
            let mut expected = vec![("freemapper:source".to_string(), "sheet".to_string())];
            if is_scale_bar(o) {
                expected.push(("freemapper:sheet".to_string(), "scale-bar".to_string()));
            }
            assert_eq!(o.tags, expected);
            assert!(
                !o.tags.iter().any(|(_, v)| v == "scale-bar") || is_scale_bar(o),
                "{o:?}"
            );
        }
        // The stable contract with freemapper-print and freemapper-courseplan: no object
        // anywhere carries the source `scale-bar`.
        for o in documents.map.parts.iter().flat_map(|p| &p.objects) {
            assert!(
                !o.tags
                    .iter()
                    .any(|(k, v)| k == "freemapper:source" && v == "scale-bar"),
                "{o:?}"
            );
        }
        let bars = sheet.objects.iter().filter(|o| is_scale_bar(o)).count();
        assert!(bars > 1, "bar line, ticks and labels are all tagged");
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
        let (lines, _) = crate::derive::contours::contours_from_geojson(&text).unwrap();
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

    /// Sends the vegetation of the project in `dir` to the candidates, as `--content-set
    /// vegetation=candidates` does.
    fn want_vegetation(dir: &Path) {
        let mut project = ProjectFile::load(dir).unwrap();
        let mut content = project.content.clone().unwrap_or_default();
        content.set(
            crate::build::content::Group::Vegetation,
            Some(crate::build::content::Place::Candidates),
        );
        project.content = Some(content);
        project.save(dir).unwrap();
    }

    /// The school, fetched and derived with the vegetation chosen.
    fn derived_school_with_vegetation() -> (tempfile::TempDir, tempfile::TempDir, Cache) {
        let fetcher = MapFetcher::default();
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        want_vegetation(dir.path());
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetch(dir.path(), &fetcher, &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        (dir, cache_dir, cache)
    }

    /// The codes of `data/vegetation.geojson` with the centre of their first outline.
    fn vegetation_codes(dir: &Path) -> Vec<(String, f64, f64)> {
        let text = std::fs::read_to_string(dir.join(VEGETATION_RELPATH)).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        json["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                let ring = f["geometry"]["coordinates"][0][0].as_array().unwrap();
                let mean = |k: usize| {
                    ring.iter().map(|c| c[k].as_f64().unwrap()).sum::<f64>() / ring.len() as f64
                };
                (
                    f["properties"]["code"].as_str().unwrap().to_string(),
                    mean(0),
                    mean(1),
                )
            })
            .collect()
    }

    #[test]
    fn derive_writes_vegetation_from_the_point_clouds() {
        use crate::fetch::test_support::{SCHOOL_LAWN, SCHOOL_THICKET};
        let (dir, _cache_dir, _) = derived_school_with_vegetation();
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.point_clouds.len(), 1);
        assert_eq!(project.point_clouds[0].tile, "33421-5821");
        assert_eq!(project.point_clouds[0].published, "2024-06-12".parse().ok());
        let codes = vegetation_codes(dir.path());
        let inside = |[w, s, side]: [f64; 3], e: f64, n: f64| {
            (w..w + side).contains(&e) && (s..s + side).contains(&n)
        };
        assert!(
            codes
                .iter()
                .any(|(c, e, n)| c == "410" && inside(SCHOOL_THICKET, *e, *n)),
            "{codes:?}"
        );
        assert!(
            codes
                .iter()
                .any(|(c, e, n)| c == "401" && inside(SCHOOL_LAWN, *e, *n)),
            "{codes:?}"
        );
        // The export puts them among the candidates, and on the map when the group says so.
        export_starter_project(dir.path(), false).unwrap();
        let tagged = |path: &Path| {
            std::fs::read_to_string(path)
                .unwrap()
                .matches(r#"<t k="freemapper:point-cloud">lgb-als</t>"#)
                .count()
        };
        let candidates = dir.path().join("templates/candidates.omap");
        assert!(tagged(&candidates) >= 2);
        assert_eq!(tagged(&map_path(dir.path())), 0);
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.content.as_mut().unwrap().set(
            crate::build::content::Group::Vegetation,
            Some(crate::build::content::Place::Map),
        );
        project.save(dir.path()).unwrap();
        export_starter_project(dir.path(), true).unwrap();
        assert_eq!(tagged(&candidates), 0);
        assert!(tagged(&map_path(dir.path())) >= 2);
    }

    #[test]
    fn vegetation_goes_with_the_point_cloud_records() {
        let (dir, _cache_dir, cache) = derived_school_with_vegetation();
        assert!(dir.path().join(VEGETATION_RELPATH).exists());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.point_clouds.clear();
        project.save(dir.path()).unwrap();
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(!dir.path().join(VEGETATION_RELPATH).exists());
        assert_eq!(
            derive_project(dir.path(), &cache, &mut |_| {})
                .unwrap()
                .len(),
            6,
            "no vegetation among the written files"
        );
        // The choice is still made: derive says the point clouds have to be fetched.
        assert!(
            messages
                .iter()
                .any(|m| m.contains("no point clouds are loaded; run `freemapper fetch`")),
            "{messages:?}"
        );
    }

    #[test]
    fn a_point_cloud_record_with_a_path_as_tile_reads_nothing() {
        let (dir, _cache_dir, cache) = derived_school_with_vegetation();
        let mut project = ProjectFile::load(dir.path()).unwrap();
        // A tampered project file pointing outside the cache.
        project.point_clouds[0].tile = "../../../../etc/passwd".to_string();
        project.save(dir.path()).unwrap();
        let mut messages = Vec::new();
        derive_project(dir.path(), &cache, &mut |m| messages.push(m.to_string())).unwrap();
        assert!(!dir.path().join(VEGETATION_RELPATH).exists());
        assert!(
            messages
                .iter()
                .any(|m| m.contains("no cached point cloud for tile ../../../../etc/passwd")),
            "{messages:?}"
        );
    }

    #[test]
    fn without_the_vegetation_choice_no_point_cloud_is_fetched_or_read() {
        let (dir, _cache_dir, cache) = derived_school_with_vegetation();
        assert!(dir.path().join(VEGETATION_RELPATH).exists());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.content = None;
        project.save(dir.path()).unwrap();
        let estimate =
            crate::layers::estimate_download(dir.path(), &cache, crate::clock::today()).unwrap();
        assert!(
            !estimate.items.iter().any(|i| i.part == Part::PointCloud),
            "{:?}",
            estimate.items
        );
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        let (project, _) = fetch(dir.path(), &fetcher, &cache);
        assert!(project.point_clouds.is_empty());
        assert!(
            !fetcher
                .log
                .borrow()
                .iter()
                .any(|url| url.contains("laserscan") || url.contains("/als/")),
            "{:?}",
            fetcher.log.borrow()
        );
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        assert!(!dir.path().join(VEGETATION_RELPATH).exists());
    }

    #[test]
    fn vegetation_candidates_stay_near_the_area_and_leave_buildings_water_paving_and_the_park() {
        use crate::build::geometry::object_polygon;
        use crate::build::out_of_bounds::area_object;
        use crate::derive::vegetation::{VegetationClass, vegetation_to_geojson};
        use geo::Area as _;
        use geo_types::{LineString, MultiPolygon, Polygon};
        let (dir, _cache_dir, _) = derived_school_with_vegetation();
        let mut project = ProjectFile::load(dir.path()).unwrap();
        let georef = Georeferencing::new_in(
            project.crs(),
            project.area().unwrap().centroid(),
            project.scale_denominator,
            project.declination_date,
        )
        .unwrap();
        let symbol_set = project.map_kind.symbol_set(project.scale_denominator);
        let rect = |w: f64, s: f64, width: f64, height: f64| {
            Polygon::new(
                LineString::from(vec![
                    (w, s),
                    (w + width, s),
                    (w + width, s + height),
                    (w, s + height),
                    (w, s),
                ]),
                vec![],
            )
        };
        std::fs::write(
            dir.path().join(VEGETATION_RELPATH),
            vegetation_to_geojson(&[
                (
                    VegetationClass::Impassable,
                    MultiPolygon(vec![rect(421_595.0, 5_821_720.0, 30.0, 30.0)]),
                ),
                (
                    VegetationClass::Open,
                    MultiPolygon(vec![rect(421_570.0, 5_821_680.0, 20.0, 20.0)]),
                ),
                // A thicket 800 m east, far beyond the map area and its buffer.
                (
                    VegetationClass::Walk,
                    MultiPolygon(vec![rect(422_400.0, 5_821_700.0, 20.0, 20.0)]),
                ),
            ]),
        )
        .unwrap();
        let building = rect(421_605.0, 5_821_730.0, 10.0, 10.0);
        let water = MultiPolygon(vec![rect(421_615.0, 5_821_720.0, 10.0, 5.0)]);
        // A park the map already has, over the west half of the lawn.
        let park = area_object(
            &rect(421_570.0, 5_821_680.0, 10.0, 20.0),
            symbol_set.id_for_code("401").unwrap(),
            Vec::new(),
            &georef,
        )
        .unwrap();
        // A car park along the north edge of the thicket.
        let paved = area_object(
            &rect(421_595.0, 5_821_745.0, 30.0, 5.0),
            symbol_set.id_for_code("501.5").unwrap(),
            Vec::new(),
            &georef,
        )
        .unwrap();
        let map_objects = [park, paved];
        let build = |project: &ProjectFile| {
            let Some(vegetation) = export::read_stored_vegetation(dir.path(), project).unwrap()
            else {
                return Vec::new();
            };
            crate::build::vegetation::vegetation_objects(
                &vegetation,
                std::slice::from_ref(&building),
                &water,
                &map_objects,
                &project.area().unwrap().projected_polygon_in(project.crs()),
                project.buffer_m,
                &georef,
                &symbol_set,
            )
            .unwrap()
        };
        let objects = build(&project);
        // Map units are micrometres on paper.
        let m2_per_unit = (f64::from(project.scale_denominator) / 1e6).powi(2);
        let area_of = |code: &str| -> f64 {
            objects
                .iter()
                .filter(|o| o.symbol_id == symbol_set.id_for_code(code).unwrap())
                .filter_map(object_polygon)
                .map(|p| p.unsigned_area() * m2_per_unit)
                .sum()
        };
        assert!((area_of("410") - 600.0).abs() < 2.0, "{}", area_of("410"));
        assert!((area_of("401") - 200.0).abs() < 2.0, "{}", area_of("401"));
        assert_eq!(area_of("408"), 0.0, "nothing beyond the buffer");
        let tags = &objects[0].tags;
        for (key, value) in [
            ("freemapper:source", "lgb-als"),
            ("freemapper:point-cloud", "lgb-als"),
            ("freemapper:currency", "2024-06-12"),
        ] {
            assert!(
                tags.iter().any(|(k, v)| k == key && v == value),
                "{key}: {tags:?}"
            );
        }
        // Without the choice of the group there are none.
        project.content = None;
        assert!(build(&project).is_empty());
    }

    #[test]
    fn a_failing_point_cloud_service_keeps_the_cached_tiles() {
        let (dir, _cache_dir, cache) = derived_school_with_vegetation();
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_ndsm(&fetcher, school_tile());
        crate::fetch::test_support::serve_synthetic_tile(&fetcher, school_tile(), "2026-01-26");
        fetcher
            .failing
            .borrow_mut()
            .push(freemapper_adapters_de::lgb_als::ALS_CURRENCY_CSV.to_string());
        let mut messages = Vec::new();
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m.contains("the laser point clouds are unavailable")),
            "{messages:?}"
        );
        let project = ProjectFile::load(dir.path()).unwrap();
        assert_eq!(project.point_clouds.len(), 1, "the cached tile stays");
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
        let set = MapKind::Schoolyard.symbol_set(1000);
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
            map.contains(r#"first_front_template="5""#),
            "five rasters behind the map"
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
            crate::derive::contours::contours_to_geojson(&lines, None),
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
    fn landforms_become_candidates_but_not_on_buildings_or_far_away() {
        use crate::derive::landforms::{Landform, LandformKind, landforms_to_geojson};
        let (dir, _cache_dir, _) = derived_school_with_ndsm();
        assert!(dir.path().join(LANDFORMS_RELPATH).exists());
        let point = |kind, p: (f64, f64), height_m| Landform {
            kind,
            points: vec![p],
            rotation_deg: if kind == LandformKind::ElongatedKnoll {
                90.0
            } else {
                0.0
            },
            height_m,
        };
        let forms = [
            point(LandformKind::Knoll, (421_620.0, 5_821_700.0), 1.1),
            point(LandformKind::ElongatedKnoll, (421_630.0, 5_821_712.0), 1.2),
            point(LandformKind::Depression, (421_625.0, 5_821_690.0), 1.3),
            Landform {
                kind: LandformKind::Bank,
                points: vec![(421_600.0, 5_821_680.0), (421_640.0, 5_821_680.0)],
                rotation_deg: 0.0,
                height_m: 1.4,
            },
            // On the synthetic school building (10 m square at 421 605 / 5 821 730).
            point(LandformKind::Knoll, (421_610.0, 5_821_735.0), 1.5),
            // 500 m away, beyond map area and buffer.
            point(LandformKind::Knoll, (422_200.0, 5_821_700.0), 1.6),
        ];
        std::fs::write(
            dir.path().join(LANDFORMS_RELPATH),
            landforms_to_geojson(&forms),
        )
        .unwrap();
        export_starter_project(dir.path(), false).unwrap();
        let candidates =
            std::fs::read_to_string(dir.path().join("templates/candidates.omap")).unwrap();
        // Contour heights are whole metres; the landforms carry one decimal.
        let heights: Vec<&str> = candidates
            .match_indices(r#"<t k="freemapper:height">"#)
            .map(|(i, m)| {
                let rest = &candidates[i + m.len()..];
                &rest[..rest.find('<').unwrap()]
            })
            .filter(|h| h.contains('.'))
            .collect();
        assert_eq!(heights, ["1.1", "1.2", "1.3", "1.4"]);
        let symbol_set = ProjectFile::load(dir.path())
            .unwrap()
            .map_kind
            .symbol_set(4000);
        for (code, object_type) in [("109", 0), ("110", 0), ("111", 0), ("104", 1)] {
            let id = symbol_set.id_for_code(code).unwrap();
            let start = format!(r#"<object type="{object_type}" symbol="{id}""#);
            assert!(candidates.contains(&start), "{code}");
        }
        // The elongated knoll runs north–south, as symbol 110 does unturned: it turns only by the
        // grivation (a few degrees), not by a quarter.
        let id_110 = symbol_set.id_for_code("110").unwrap();
        let start = format!(r#"<object type="0" symbol="{id_110}" rotation=""#);
        let at = candidates.find(&start).unwrap() + start.len();
        let rotation: f64 = candidates[at..at + candidates[at..].find('"').unwrap()]
            .parse()
            .unwrap();
        assert!(rotation.abs() < 0.2, "{rotation}");
        // The map itself stays without them.
        let map = std::fs::read_to_string(map_path(dir.path())).unwrap();
        assert!(!map.contains(">1.1<"));
        // The terrain model has no source rank: `balanced` leaves the bank and the knoll
        // candidates. They are the group `landforms` whatever their symbol: `slopes` set to the
        // map (the ALKIS and OSM banks) leaves them, `landforms` set to the map moves them.
        use crate::build::content::{Group, Place, Profile};
        let mut content = ContentSettings::with_profile(Profile::Balanced);
        let on_map = |dir: &Path| {
            export_starter_project(dir, true).unwrap();
            let map = std::fs::read_to_string(map_path(dir)).unwrap();
            (map.contains(">1.4<"), map.contains(">1.1<"))
        };
        set_content(dir.path(), Some(content.clone())).unwrap();
        assert_eq!(on_map(dir.path()), (false, false));
        content.set(Group::Slopes, Some(Place::Map));
        set_content(dir.path(), Some(content.clone())).unwrap();
        assert_eq!(on_map(dir.path()), (false, false));
        content.set(Group::Landforms, Some(Place::Map));
        set_content(dir.path(), Some(content)).unwrap();
        assert_eq!(on_map(dir.path()), (true, true));
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
            crate::derive::contours::contours_to_geojson(&lines, None),
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
        let (map, _) = pdf_objects(&documents, true);
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
            &freemapper_adapters_de::lgb::ndsm_tile_url(school_tile()),
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
        let outline = crate::build::assembly::map_area_outline(
            &area,
            project.map_area_source_id.as_deref(),
            &georef,
            &MapKind::Schoolyard.symbol_set(1000),
        )
        .unwrap();

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
            outcome.warnings[0].is_warning()
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
            freemapper_sources::float_tiff::write_float_raster(
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

    #[test]
    fn outputs_from_one_build_equal_those_of_separate_builds() {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = sprint_with_ways(&fetcher);
        let dir = dir.path();
        let outputs = |written: [PathBuf; 4]| -> Vec<Vec<u8>> {
            written
                .iter()
                .chain([&dir.join(CANDIDATES_RELPATH), &dir.join("ATTRIBUTION.txt")])
                .map(|path| std::fs::read(path).unwrap())
                .collect()
        };
        let sources = PdfSources::Auto;
        let separate = outputs([
            export_starter_project(dir, true).unwrap(),
            export_ocad(dir, true).unwrap().map,
            export_pdf(dir, true, sources).unwrap(),
            export_candidates_pdf(dir, true, sources).unwrap(),
        ]);
        let documents = starter_documents(dir).unwrap();
        let shared = outputs([
            export_starter_project_from(dir, &documents, true).unwrap(),
            export_ocad_from(dir, &documents, true).unwrap().map,
            export_pdf_from(dir, &documents, true, sources).unwrap(),
            export_candidates_pdf_from(dir, &documents, true, sources).unwrap(),
        ]);
        assert_eq!(separate.len(), 6);
        for (index, (one, other)) in separate.iter().zip(&shared).enumerate() {
            assert!(!one.is_empty(), "output {index}");
            assert!(one == other, "output {index} differs");
        }
    }

    /// freemapper writes exactly the kinds the contract says it writes, under the contract's names
    /// and at the contract's scales (ADR 0095). A kind added on one side only fails here.
    #[test]
    fn contract_kinds_are_the_kinds_of_the_suite() {
        let written: Vec<_> = freemapper_contract::MapKind::ALL
            .into_iter()
            .filter(|kind| kind.is_written_by_freemapper())
            .collect();
        let mut ours: Vec<_> = MapKind::ALL.into_iter().map(MapKind::contract).collect();
        ours.sort();
        let mut theirs = written.clone();
        theirs.sort();
        assert_eq!(ours, theirs);
        for kind in MapKind::ALL {
            assert_eq!(MapKind::from_name(kind.name()), Some(kind));
            // What serde writes into freemapper.toml is the contract's name.
            let written = serde_json::to_value(kind).unwrap();
            assert_eq!(written, kind.contract().name());
            assert_eq!(kind.allowed_scales(), kind.contract().drawing_scales());
            assert!(kind.allowed_scales().contains(&kind.default_scale()));
            for scale in kind.allowed_scales() {
                assert!(
                    kind.check_scale(*scale).is_ok(),
                    "{} 1:{scale}",
                    kind.name()
                );
            }
        }
        assert_eq!(MapKind::from_name("park"), None);
        assert_eq!(PROJECT_FILE_NAME, "freemapper.toml");
    }

    /// The fixtures of the suite contract in freemapper-core (crate `freemapper-contract`): the
    /// map kind a fixture is built from, and its name there.
    const CONTRACT_FIXTURES: [(MapKind, &str); 2] = [
        (MapKind::Schoolyard, "schoolyard-1000"),
        (MapKind::Sprint, "sprint-4000"),
    ];

    /// Map XML and sidecar text of an exported project of `kind`, as freemapper-print and
    /// freemapper-courseplan read them.
    fn contract_project(kind: MapKind) -> (String, String) {
        let fetcher = MapFetcher::default();
        let (dir, _cache_dir) = match kind {
            MapKind::Schoolyard => derived_school(&fetcher),
            MapKind::Sprint => sprint_with_ways(&fetcher),
            MapKind::Forest => unreachable!("core has no contract fixture of a forest map yet"),
        };
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.title = Some("Clara-Zetkin-Schule".to_string());
        project.save(dir.path()).unwrap();
        let map =
            std::fs::read_to_string(export_starter_project(dir.path(), false).unwrap()).unwrap();
        let sidecar = std::fs::read_to_string(dir.path().join(PROJECT_FILE_NAME)).unwrap();
        (map, sidecar)
    }

    /// The contract's reader takes kind, title, scale and CRS from a project file as written.
    #[test]
    fn contract_sidecar_reader_reads_a_written_project_file() {
        for (kind, name) in CONTRACT_FIXTURES {
            let (_map, sidecar) = contract_project(kind);
            let read = freemapper_contract::Sidecar::parse(&sidecar).unwrap();
            assert_eq!(read.map_kind.known(), Some(kind.contract()), "{name}");
            assert_eq!(read.scale_denominator, Some(kind.default_scale()), "{name}");
            assert_eq!(read.title(), Some("Clara-Zetkin-Schule"), "{name}");
            assert!(matches!(read.crs, Some(25832 | 25833)), "{name}");
        }
    }

    #[test]
    fn contract_output_breaks_no_rule() {
        for (kind, name) in CONTRACT_FIXTURES {
            let (map, sidecar) = contract_project(kind);
            let violations = freemapper_contract::check(&map, &sidecar).unwrap();
            assert!(violations.is_empty(), "{name}: {violations:#?}");
        }
    }

    #[test]
    fn contract_vocabulary_matches_the_fixture() {
        for (kind, name) in CONTRACT_FIXTURES {
            let (map, sidecar) = contract_project(kind);
            let written = freemapper_contract::Vocabulary::of(&map, &sidecar).unwrap();
            let agreed = freemapper_contract::fixture(name).unwrap().vocabulary();
            let differences = written.differences(&agreed);
            assert!(
                differences.is_empty(),
                "{name}: {differences:#?}\nA change to these words is a change to the suite \
                 contract: refresh the fixtures in freemapper-core first (core ADR 0018)."
            );
        }
    }

    /// Writes map, sidecar, vocabulary and census of every fixture into
    /// `$FREEMAPPER_CONTRACT_OUT/<name>/`. Run by hand when the contract changes:
    /// `FREEMAPPER_CONTRACT_OUT=<freemapper-core>/crates/freemapper-contract/fixtures/v1
    /// cargo test -p freemapper contract_write_fixtures -- --ignored`
    #[test]
    #[ignore = "writes the fixtures of freemapper-core; run by hand"]
    fn contract_write_fixtures() {
        let out = PathBuf::from(
            std::env::var_os("FREEMAPPER_CONTRACT_OUT")
                .expect("FREEMAPPER_CONTRACT_OUT names the folder fixtures/v1 in freemapper-core"),
        );
        for (kind, name) in CONTRACT_FIXTURES {
            let (map, sidecar) = contract_project(kind);
            let vocabulary = freemapper_contract::Vocabulary::of(&map, &sidecar).unwrap();
            let census = freemapper_contract::census(&map).unwrap();
            let dir = out.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("map.omap"), map).unwrap();
            std::fs::write(dir.join(PROJECT_FILE_NAME), sidecar).unwrap();
            std::fs::write(dir.join("vocabulary.toml"), vocabulary.to_toml()).unwrap();
            std::fs::write(dir.join("census.toml"), toml::to_string(&census).unwrap()).unwrap();
        }
    }
}
