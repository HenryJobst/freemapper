//! Project folder and project file (`freemapper.toml`).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use chrono::{Datelike as _, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::area::{AreaError, MapArea};
use crate::build::buildings::building_objects;
use crate::build::peripheral::{PaperRect, PeripheralInfo, info_block, north_lines};
use crate::crs::{BoundingBox, LatLon, utm33_forward};
use crate::derive::orthophoto::build_orthophoto;
use crate::derive::terrain::{hillshade, mosaic_dtm, slope_deg, slope_shade, write_gray_png};
use crate::derive::{DeriveError, MAX_RASTER_SIDE_PX};
use crate::fetch::{
    Cache, FetchError, FetchMode, FetchedTile, Fetcher, fetch_tiles, write_atomically,
};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{
    MapDocument, MapObject, MapObjectKind, MapPart, OmapError, SymbolSet, TemplateKind,
    TemplateRef, XmlFormat, closed_ring, write_map,
};
use crate::raster::GridSpec;
use crate::sources::alkis::{fetch_buildings, parse_buildings};
use crate::sources::lgb::{Product, TileCurrency, attribution};
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
    #[error(
        "{}: image and world file do not match the map area; run `freemapper derive` again",
        .0.display()
    )]
    TemplateOutdated(PathBuf),
    #[error("the map area needs tiles {} that were not fetched; run `freemapper fetch`", tile_names(.0))]
    TilesOutdated(Vec<TileId>),
    #[error("no ALKIS buildings stored for this map area; run `freemapper fetch` online first")]
    BuildingsNotFetched,
    #[error(
        "map area plus buffer is {width_m:.0} × {height_m:.0} m, more than {MAX_AREA_SIDE_M:.0} m per side; reduce the map area or buffer_m in freemapper.toml"
    )]
    AreaTooLarge { width_m: f64, height_m: f64 },
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
    pub buildings: Option<BuildingsRecord>,
}

/// Which ALKIS buildings were fetched and how current they are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingsRecord {
    /// Queried area in EPSG:25833: min easting, min northing, max easting, max northing.
    pub bbox: [f64; 4],
    pub fetched: NaiveDate,
    /// Newest `aktualit` among the buildings.
    pub latest_update: Option<NaiveDate>,
    pub count: usize,
}

impl BuildingsRecord {
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
        std::fs::write(&path, text).map_err(|e| ProjectError::Io(path, e))
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
    };
    project.save(dir)?;
    Ok(project)
}

use crate::fetch::tile_names;

/// Partial template files older than this are left over from interrupted runs.
const STALE_TEMPLATE_PART_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// Largest side of map area plus buffer: [`MAX_RASTER_SIDE_PX`] orthophoto pixels of 0.2 m.
const MAX_AREA_SIDE_M: f64 = MAX_RASTER_SIDE_PX as f64 * 0.2;

/// ISSprOM spacing of magnetic north lines on paper, before the norm's symbol enlargement.
const NORTH_LINE_SPACING_MM: f64 = 30.0;

/// Products fetched for every project, in this order.
const PRODUCTS: [Product; 2] = [Product::Orthophoto, Product::Dtm];
const ORTHOPHOTO_RELPATH: &str = "templates/orthophoto.jpg";
const HILLSHADE_RELPATH: &str = "templates/hillshade.png";
const SLOPE_RELPATH: &str = "templates/slope.png";
/// Skipped ALKIS features listed in a progress message.
const SKIPPED_SHOWN: usize = 5;
/// ALKIS buildings around the map area, as fetched (GeoJSON, EPSG:25833).
const BUILDINGS_RELPATH: &str = "data/alkis-buildings.geojson";

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
    if bbox.width() > MAX_AREA_SIDE_M || bbox.height() > MAX_AREA_SIDE_M {
        return Err(ProjectError::AreaTooLarge {
            width_m: bbox.width(),
            height_m: bbox.height(),
        });
    }
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
    project.tiles = records;
    project.save(dir)?;
    let bbox = project.bbox()?;
    let previous = project.buildings.as_ref();
    if let Some(record) = fetch_buildings_into(dir, fetcher, &bbox, mode, previous, progress)? {
        project.buildings = Some(record);
        project.save(dir)?;
    }
    Ok(project)
}

/// Downloads the ALKIS buildings in `bbox` to [`BUILDINGS_RELPATH`] and returns the new record;
/// the stored file is only replaced by an answer that parses. Offline, the stored file must
/// exist and, if recorded, cover `bbox`; nothing new is recorded then.
fn fetch_buildings_into(
    dir: &Path,
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    mode: FetchMode,
    previous: Option<&BuildingsRecord>,
    progress: &mut dyn FnMut(&str),
) -> Result<Option<BuildingsRecord>, ProjectError> {
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
    let record = BuildingsRecord {
        bbox: [
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil(),
        ],
        fetched: chrono::Local::now().date_naive(),
        latest_update: parsed.buildings.iter().filter_map(|b| b.updated).max(),
        count,
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

    Ok(vec![orthophoto, hillshade_path, slope_path])
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
    /// references by path were refreshed.
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

    let candidates = MapDocument {
        notes: notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts: vec![MapPart {
            name: "candidates".to_string(),
            objects: vec![map_area_outline(&area, &georef, &symbol_set, &project)?],
        }],
        templates: Vec::new(),
    };
    let crs_spec = format!("+init=epsg:{}", georef.epsg);
    let mut templates: Vec<TemplateRef> = [
        (ORTHOPHOTO_RELPATH, true, 1.0),
        (HILLSHADE_RELPATH, false, 0.5),
        (SLOPE_RELPATH, false, 0.5),
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
    let buildings_part = MapPart {
        name: "default part".to_string(),
        objects: stored_building_objects(dir, &project, &area, &georef, &symbol_set)?,
    };
    let content = PaperRect::of_bbox(&project.bbox()?, &georef)?;
    let mut attribution = vec![OSM_ATTRIBUTION_DE.to_string()];
    attribution.extend(lgb_attribution(&project));
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
            name: "peripheral information".to_string(),
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

    let map_path = map_path(dir);
    if map_path.exists() {
        if !force {
            return Err(ProjectError::MapExists(map_path));
        }
        let backup = map_path.with_extension("omap.bak");
        std::fs::copy(&map_path, &backup).map_err(|e| ProjectError::Io(backup, e))?;
    }
    write_document(&candidates, &dir.join(CANDIDATES_RELPATH))?;
    write_document(&map, &map_path)?;
    let attribution = dir.join("ATTRIBUTION.txt");
    std::fs::write(
        &attribution,
        format!("{attribution_text}\n\n{FREEMAPPER_CREDIT}\n"),
    )
    .map_err(|e| ProjectError::Io(attribution, e))?;
    Ok(map_path)
}

const CANDIDATES_RELPATH: &str = "templates/candidates.omap";
const OSM_ATTRIBUTION: &str = "Map area: © OpenStreetMap contributors, ODbL";
/// [`OSM_ATTRIBUTION`] for the German peripheral information.
const OSM_ATTRIBUTION_DE: &str = "Kartengebiet: © OpenStreetMap-Mitwirkende, ODbL";
/// [`FREEMAPPER_CREDIT`] for the German peripheral information.
const FREEMAPPER_CREDIT_DE: &str =
    concat!("Erstellt mit freemapper – ", env!("CARGO_PKG_REPOSITORY"));
/// Names the tool and where to get it, in every generated project.
const FREEMAPPER_CREDIT: &str = concat!("Created with freemapper – ", env!("CARGO_PKG_REPOSITORY"));

/// Attribution of every source the project uses: OSM, and LGB once tiles were fetched.
fn attribution_lines(project: &ProjectFile) -> Vec<String> {
    let mut lines = vec![OSM_ATTRIBUTION.to_string()];
    lines.extend(lgb_attribution(project));
    lines
}

/// The LGB attribution line, once tiles were fetched.
fn lgb_attribution(project: &ProjectFile) -> Option<String> {
    let year = project.tiles.iter().map(|t| t.published.year()).max()?;
    Some(attribution(year))
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
            buildings: Some(BuildingsRecord {
                bbox: [421_490.0, 5_821_580.0, 421_737.0, 5_821_897.0],
                fetched: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
                latest_update: NaiveDate::from_ymd_opt(2023, 5, 4),
                count: 18,
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
            !first.iter().any(|m| m.starts_with("warning: ")),
            "{first:?}"
        );

        serve_synthetic_tile(&fetcher, school_tile(), "2026-06-01");
        let (_, second) = fetch(dir.path(), &fetcher, &cache);
        let warnings: Vec<_> = second
            .iter()
            .filter(|m| m.starts_with("warning: "))
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
                templates.join("slope.png")
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
        assert_eq!(outcome.templates.len(), 3);
        assert_eq!(outcome.map, MapOutcome::Written(map_path(dir.path())));
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
        assert!(map.contains("<text>Kartengebiet: © OpenStreetMap-Mitwirkende, ODbL</text>"));
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
