//! The documents of a project and the files written from them: the starter project (map and
//! candidates template map), the OCAD files and the PDFs (ADR 0091: built once per command).
//!
//! What the build reads is stored under `data/` and `templates/`; none of the exports writes a
//! file the build reads.

use super::*;
use crate::build::barrier_gaps::move_points_off_barriers;
use crate::build::content::route;
use crate::build::line_cuts::{CONTOUR_CODES, NORTH_LINE_CODES, cut_lines_at_ways};
use crate::build::paved_borders::border_paved_areas;
use crate::build::points::way_footprints;
use crate::build::ticks::{orient_ticks, read_terrain};

/// The stored ALKIS buildings, read once per export: parsed, and the raw features (the Berlin
/// building parts are picked from these).
pub(super) struct StoredBuildings {
    pub(super) buildings: Vec<crate::sources::alkis::Building>,
    pub(super) features: Vec<serde_json::Value>,
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
pub(super) fn read_stored_buildings(dir: &Path) -> Result<Option<StoredBuildings>, ProjectError> {
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
pub(super) fn stored_building_objects(
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
pub(super) fn stored_osm_features(
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
pub(super) fn stored_osm_objects(
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
pub(super) fn stored_contour_objects(
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
    let provider = match &project.provider {
        Some(provider) => Some(provider.clone()),
        None => raster_provider(&project.region_bbox()?),
    };
    let source = terrain_source(provider.as_ref());
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
pub(super) const CONTOUR_CODE: &str = "101";
pub(super) const INDEX_CONTOUR_CODE: &str = "102";

/// The stored ALKIS buildings; none when no buildings were fetched.
#[cfg(test)]
pub(super) fn stored_buildings(
    dir: &Path,
) -> Result<Vec<crate::sources::alkis::Building>, ProjectError> {
    Ok(read_stored_buildings(dir)?.map_or_else(Vec::new, |s| s.buildings))
}

/// Whether the stored buildings give map objects that the map at `map_text` lacks.
pub(super) fn map_lacks_buildings(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if map_text.contains(ALKIS_SOURCE_TAG) {
        return Ok(false);
    }
    let project = ProjectFile::load(dir)?;
    let area = project.area()?;
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
pub(super) const ALKIS_SOURCE_TAG: &str = r#"<t k="freemapper:source">alkis</t>"#;

/// Whether the stored layers and buildings give 522 canopy map objects that the map at
/// `map_text` lacks (it was written before them, or without the layers).
pub(super) fn map_lacks_canopies(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if CANOPY_SOURCE_TAGS.iter().any(|tag| map_text.contains(tag)) {
        return Ok(false);
    }
    let project = ProjectFile::load(dir)?;
    let layers = stored_berlin_layers(dir, &project)?;
    let area = project.area()?;
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
pub(super) const CANOPY_SOURCE_TAGS: [&str; 3] = [
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
    write_starter_project(dir, &starter_documents(dir)?, Some(force))
}

/// [`export_starter_project`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_starter_project_from(
    dir: &Path,
    documents: &StarterDocuments,
    force: bool,
) -> Result<PathBuf, ProjectError> {
    write_starter_project(dir, documents, Some(force))
}

/// Writes the candidates template map and the attribution, and the map unless `map_force` is
/// `None` (the map is kept: it may contain the mapper's work). Candidates are a template that
/// always follows the current map area.
pub(super) fn write_starter_project(
    dir: &Path,
    documents: &StarterDocuments,
    map_force: Option<bool>,
) -> Result<PathBuf, ProjectError> {
    let StarterDocuments {
        map,
        candidates,
        attribution_text,
        ..
    } = documents;
    let map_path = map_path(dir);
    if let Some(force) = map_force
        && map_path.exists()
    {
        if !force {
            return Err(ProjectError::MapExists(map_path));
        }
        back_up(dir, &map_path, &map_path.with_extension("omap.bak"))?;
    }
    write_document(dir, candidates, &dir.join(CANDIDATES_RELPATH))?;
    if map_force.is_some() {
        forget_map_preview(dir)?;
        write_document(dir, map, &map_path)?;
    }
    let attribution = format!("{attribution_text}\n\n{FREEMAPPER_CREDIT}\n");
    write_inside(dir, &dir.join("ATTRIBUTION.txt"), attribution.as_bytes())?;
    Ok(map_path)
}

/// What to do when the map to back up is a link; ends the error of [`back_up`].
pub(super) const LINKED_MAP_HINT: &str =
    "replace the link with the real file and run the command again";

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
    /// Templates left out of the OCAD map.
    pub warnings: Vec<Note>,
}

/// The world file next to an image: `.jpg` → `.jgw`, `.png` → `.pgw`.
pub(super) fn world_file_path(image: &Path) -> Option<PathBuf> {
    let ext = image.extension()?.to_str()?;
    let mut chars = ext.chars();
    let (first, last) = (chars.next()?, ext.chars().last()?);
    Some(image.with_extension(format!("{first}{last}w")))
}

/// Writes the OCAD starting point into `ocad/`: `<name>.ocd` (OCAD 12) with the map objects,
/// `<name>-candidates.ocd`, and the raster templates with their world files as background maps.
/// An existing `<name>.ocd` is only replaced when `force` is set (kept as `.ocd.bak`).
pub fn export_ocad(dir: &Path, force: bool) -> Result<OcadOutcome, ProjectError> {
    export_ocad_from(dir, &starter_documents(dir)?, force)
}

/// [`export_ocad`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_ocad_from(
    dir: &Path,
    documents: &StarterDocuments,
    force: bool,
) -> Result<OcadOutcome, ProjectError> {
    let project = ProjectFile::load(dir)?;
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
pub(super) fn copy_backgrounds(
    dir: &Path,
    ocad_dir: &Path,
    templates: &[TemplateRef],
) -> Result<(Vec<Background>, Vec<Note>), ProjectError> {
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
            warnings.push(note!(
                "warning-ocad-no-background",
                error = Arg::error(ProjectError::Io(path, e))
            ));
            continue;
        }
        copy_inside(dir, &image, &ocad_dir.join(image_name))?;
        copy_inside(dir, &world, &ocad_dir.join(world_name))?;
        let text =
            std::fs::read_to_string(&world).map_err(|e| ProjectError::Io(world.clone(), e))?;
        let world_file = WorldFile::parse(&text).ok_or_else(|| {
            ProjectError::Layer(note!(
                "error-layer-not-a-world-file",
                path = world.display().to_string()
            ))
        })?;
        let (width_px, height_px) = image::image_dimensions(&image).map_err(|e| {
            ProjectError::Layer(note!(
                "error-layer-file",
                path = image.display().to_string(),
                detail = e.to_string()
            ))
        })?;
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
pub(super) const SHEET_PART: &str = freemapper_contract::tags::SHEET_PART;

/// The objects of a PDF: the map's own objects, with `candidates` also every candidate except
/// the map area outline (the first candidate) and the points hidden under a 520 candidate
/// (`freemapper:hidden`), and separately the sheet objects, which are not clipped.
pub(super) fn pdf_objects(
    documents: &StarterDocuments,
    candidates: bool,
) -> (Vec<&MapObject>, Vec<&MapObject>) {
    let (sheet, own): (Vec<&MapPart>, Vec<&MapPart>) = documents
        .map
        .parts
        .iter()
        .partition(|p| p.name == SHEET_PART);
    let candidate_objects = documents
        .candidates
        .parts
        .iter()
        .flat_map(|p| p.objects.iter().skip(1))
        .filter(|o| !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY))
        .take(if candidates { usize::MAX } else { 0 });
    let map = own
        .iter()
        .flat_map(|p| &p.objects)
        .chain(candidate_objects)
        .collect();
    (map, sheet.iter().flat_map(|p| &p.objects).collect())
}

/// Writes `<name>.pdf`, the printable map: the map objects as in the OOM map (base fill
/// included) and the peripheral information on the sheet, without any candidate.
pub fn export_pdf(
    dir: &Path,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    write_pdf_file(dir, &starter_documents(dir)?, false, scale_bar, sources)
}

/// [`export_pdf`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_pdf_from(
    dir: &Path,
    documents: &StarterDocuments,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    write_pdf_file(dir, documents, false, scale_bar, sources)
}

/// Writes `<name>-candidates.pdf`, the training map: the map objects and all candidates drawn
/// with the project's symbol set (this deliberately goes beyond "only reliable data as map
/// objects"), the peripheral information on the sheet.
pub fn export_candidates_pdf(
    dir: &Path,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    write_pdf_file(dir, &starter_documents(dir)?, true, scale_bar, sources)
}

/// [`export_candidates_pdf`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_candidates_pdf_from(
    dir: &Path,
    documents: &StarterDocuments,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    write_pdf_file(dir, documents, true, scale_bar, sources)
}

/// The file `export_pdf` or `export_candidates_pdf` writes.
pub fn pdf_path(dir: &Path, candidates: bool) -> PathBuf {
    let map = map_path(dir);
    if !candidates {
        return map.with_extension("pdf");
    }
    let stem = map
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    map.with_file_name(format!("{stem}-candidates.pdf"))
}

pub(super) fn write_pdf_file(
    dir: &Path,
    documents: &StarterDocuments,
    candidates: bool,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    use crate::render::{draw, pdf, symbols::RenderSymbols};
    let project = ProjectFile::load(dir)?;
    let symbols = RenderSymbols::from_set(&documents.map.symbol_set);
    let (map, _) = pdf_objects(documents, candidates);
    // The sheet is built again: its source block follows `sources`, the OOM map keeps the full list.
    let info = PeripheralInfo {
        attribution: pdf_attribution(&documents.info.attribution, &documents.content, sources),
        ..documents.info.clone()
    };
    let sheet_objects = info_block(&documents.content, &info, &documents.map.symbol_set);
    let sheet: Vec<&MapObject> = sheet_objects
        .iter()
        .filter(|o| scale_bar || !is_scale_bar(o))
        .collect();
    let render = |objects: &[&MapObject]| -> Vec<draw::Primitive> {
        objects
            .iter()
            .flat_map(|o| draw::primitives(o, &symbols))
            .collect()
    };
    let clip = paper_box(&PaperRect::of_bbox(
        &project.bbox()?,
        &documents.map.georef,
    )?);
    let bytes = pdf::write_pdf(
        &render(&map),
        &render(&sheet),
        clip,
        &symbols.colours,
        &project.display_title(dir),
    );
    let path = pdf_path(dir, candidates);
    write_inside(dir, &path, &bytes)?;
    Ok(path)
}

/// The preview of the map: a lossless WebP of what the map PDF draws in the map area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapImage {
    pub webp: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// What `data/map-preview.json` says about the preview next to the map: the project it was
/// exported for, without its content settings ([`project_fingerprint`]), and those settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapPreviewRecord {
    pub fingerprint: String,
    pub content: ContentSettings,
}

/// The record of the preview, in the internal store.
const MAP_PREVIEW_RECORD_RELPATH: &str = "data/map-preview.json";

/// The preview next to the map: `<name>-preview.webp` for `<name>.omap`.
pub fn map_preview_path(dir: &Path) -> PathBuf {
    let map = map_path(dir);
    let stem = map
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    map.with_file_name(format!("{stem}-preview.webp"))
}

/// The preview of the map in `documents`: the base part and the map part as the map PDF draws
/// them, clipped to the map area on paper (in paper orientation), without the sheet and without
/// any candidate. Texts are left out (see [`crate::render::raster`]).
pub fn map_preview_webp(
    documents: &StarterDocuments,
    dir: &Path,
) -> Result<MapImage, ProjectError> {
    use crate::render::{draw, raster, symbols::RenderSymbols};
    let project = ProjectFile::load(dir)?;
    let symbols = RenderSymbols::from_set(&documents.map.symbol_set);
    let (map, _) = pdf_objects(documents, false);
    let primitives: Vec<draw::Primitive> = map
        .iter()
        .flat_map(|o| draw::primitives(o, &symbols))
        .collect();
    let clip = paper_box(&PaperRect::of_bbox(
        &project.bbox()?,
        &documents.map.georef,
    )?);
    let image = raster::rasterize(
        &primitives,
        clip,
        &symbols.colours,
        raster::preview_px_per_mm(&clip),
    );
    let (width, height) = image.dimensions();
    let mut webp = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut webp)
        .encode(
            image.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| ProjectError::Io(map_preview_path(dir), std::io::Error::other(e)))?;
    Ok(MapImage {
        webp,
        width,
        height,
    })
}

/// Writes the preview of the map, `<name>-preview.webp`, and its record `data/map-preview.json`
/// (both or neither), from `documents`, which [`starter_documents`] built for `dir`. Called
/// where the map is written, never where an existing map is kept.
pub fn export_map_preview_from(
    dir: &Path,
    documents: &StarterDocuments,
) -> Result<PathBuf, ProjectError> {
    let image = map_preview_webp(documents, dir)?;
    let record = MapPreviewRecord {
        fingerprint: project_fingerprint(&ProjectFile::load(dir)?)?,
        content: documents.content_settings.clone(),
    };
    let path = map_preview_path(dir);
    let record_path = dir.join(MAP_PREVIEW_RECORD_RELPATH);
    let json = serde_json::to_vec_pretty(&record)
        .map_err(|e| ProjectError::Io(record_path.clone(), std::io::Error::other(e)))?;
    crate::fsutil::write_files_inside(
        dir,
        &[
            (path.as_path(), &image.webp),
            (record_path.as_path(), &json),
        ],
    )
    .map_err(|(path, e)| ProjectError::Io(path, e))?;
    Ok(path)
}

/// SHA-256 (hex) of the project file the library writes for `project` without its `[content]`
/// table: equal for two projects that differ only in their content settings, so the stored data
/// and a preview still fit the project.
pub fn project_fingerprint(project: &ProjectFile) -> Result<String, ProjectError> {
    use sha2::Digest as _;
    let without_content = ProjectFile {
        content: None,
        ..project.clone()
    };
    let text = toml::to_string(&without_content).map_err(|e| ProjectError::Toml(e.to_string()))?;
    Ok(sha2::Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Removes the record of the preview, then the preview, before the map is replaced: if the new
/// preview then fails, the folder has none, never the old one claiming to show the new map.
/// Without its record an image is no preview of any map, so the image goes on a best-effort
/// basis; the record must go.
fn forget_map_preview(dir: &Path) -> Result<(), ProjectError> {
    let record = dir.join(MAP_PREVIEW_RECORD_RELPATH);
    if freemapper_fs::stays_inside(dir, &record) {
        match std::fs::remove_file(&record) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(ProjectError::Io(record, e));
            }
            _ => {}
        }
    }
    freemapper_fs::remove_inside(dir, &map_preview_path(dir));
    Ok(())
}

/// The record of the preview in `dir`; none when there is none or it does not read.
pub fn read_map_preview_record(dir: &Path) -> Option<MapPreviewRecord> {
    let text = std::fs::read_to_string(dir.join(MAP_PREVIEW_RECORD_RELPATH)).ok()?;
    serde_json::from_str(&text).ok()
}

/// The paper rectangle `rect` (µm) in millimetres.
fn paper_box(rect: &PaperRect) -> crate::render::pdf::PaperBox {
    crate::render::pdf::PaperBox {
        min: (
            f64::from(rect.min_x) / 1000.0,
            f64::from(rect.min_y) / 1000.0,
        ),
        max: (
            f64::from(rect.max_x) / 1000.0,
            f64::from(rect.max_y) / 1000.0,
        ),
    }
}

/// Which source lines the PDF prints under the title block (ADR 0081). `ATTRIBUTION.txt` and the
/// OOM map always list every source, and the freemapper credit line is printed in every mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfSources {
    /// The full list if it fits the budget, else shortened (`dl-de/zero-2-0` lines merged first).
    #[default]
    Auto,
    /// The full list, never shortened.
    Full,
    /// No data-source lines, only the freemapper credit line.
    CreditOnly,
}

impl PdfSources {
    /// Whether the data sources need a credit when the map is passed on (ODbL, `dl-de/by-2-0`):
    /// the CLI warns when `CreditOnly` leaves them out.
    pub fn leaves_out_required_credits(self) -> bool {
        self == Self::CreditOnly
    }
}

/// The source block of the PDF: `lines` is the attribution of the sheet, the freemapper credit
/// line last; the credit line is in every result, as the last line.
pub(super) fn pdf_attribution(
    lines: &[String],
    content: &PaperRect,
    sources: PdfSources,
) -> Vec<String> {
    let (credit, body) = match lines.split_last() {
        Some((last, body)) if last == FREEMAPPER_CREDIT_DE => (last.clone(), body),
        _ => (FREEMAPPER_CREDIT_DE.to_string(), lines),
    };
    match sources {
        PdfSources::Full => body.iter().cloned().chain([credit]).collect(),
        PdfSources::CreditOnly => vec![credit],
        PdfSources::Auto => {
            let width_mm = f64::from(content.max_x - content.min_x) / 1000.0;
            source_block::compact(body, &credit, &source_block::Budget::for_content(width_mm))
        }
    }
}

/// The map and the candidates template map of a project, as written to OOM and OCAD files and
/// drawn into the PDFs. Building them reads every stored layer of the project, so a command that
/// writes several outputs builds them once with [`starter_documents`] and passes them to the
/// `export_*_from` functions (ADR 0091).
pub struct StarterDocuments {
    pub(super) map: MapDocument,
    pub(super) candidates: MapDocument,
    pub(super) attribution_text: String,
    /// The map content on paper and the texts of the sheet, from which the PDF builds its own.
    pub(super) content: PaperRect,
    pub(super) info: PeripheralInfo,
    /// The content settings the objects were placed with (ADR 0100).
    pub(super) content_settings: ContentSettings,
}

/// Builds the documents of the project in `dir` from its stored data. They belong to that folder
/// and to the data as stored at this moment: build them again after a fetch or a derive.
pub fn starter_documents(dir: &Path) -> Result<StarterDocuments, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let area = project.area()?;
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
    // The same in an OSM fountain basin, which the basin names in `freemapper:also` (ADR 0087).
    let mut osm_candidates = osm_candidates;
    let cadastre_candidates = drop_fountains_in_osm_basins(
        cadastre_candidates,
        &mut osm_candidates,
        &georef,
        &symbol_set,
    );
    // A surveyed bus shelter under an ALKIS canopy is that canopy.
    let cadastre_candidates =
        drop_shelters_under_canopies(cadastre_candidates, &mut structures.canopies);
    // Walls, fences and hedges of any symbol that overlap on paper keep the stronger line, those
    // along a building outline go, and a tree in a hedge is the hedge (`build/barriers.rs`).
    let (osm_candidates, cadastre_candidates) = resolve_barrier_overlaps(
        osm_candidates,
        cadastre_candidates,
        &footprints,
        project.map_kind,
        &georef,
        &symbol_set,
    )?;
    // Parks, meadows and woods of the land use: 401, 403 and 405 (`build/open_land.rs`). Where
    // they are map objects, the OSM parks and lawns go.
    let open_land = open_land_objects(
        &land_use,
        &area.projected_polygon_in(project.crs()),
        &clip,
        project.map_kind,
        &georef,
        &symbol_set,
    )?;
    let osm_candidates = drop_open_land_under_alkis(osm_candidates, &open_land.map, &symbol_set);
    // The point symbols keep the norm's gap, dense tree groups become 501.3 on paving, else 402
    // or 405 (on a white base off open land with a 416 boundary; open land: the parks, the 401
    // green strips of the paved build, the OSM pitches among the candidates), cut at paving and
    // paths, and no point stands in a 520 area (`build/points.rs`). The OSM candidates stay
    // before the others.
    let mut out_of_bounds =
        stored_out_of_bounds_objects(&land_use, &project, &area, &georef, &symbol_set)?;
    let mut points = osm_candidates;
    points.extend(cadastre_candidates);
    // An area that two sources give (a passage in OSM and in ALKIS or the Berlin building parts,
    // a private strip in OSM inside an ALKIS area that shall not be entered, an OSM bridge over
    // the ALKIS one) is kept once, by rank (`build/dedup.rs`).
    let mut points = drop_areas_by_rank(
        points,
        &mut [&mut out_of_bounds, &mut structures.canopies],
        &symbol_set,
    );
    // A passage no runner can reach is left out of the training map (`build/passages.rs`).
    hide_unreachable_passages(
        &mut points,
        &out_of_bounds,
        &footprints,
        &water.union,
        &clip,
        &georef,
        &symbol_set,
    );
    // The firm-gravel ways (501.6 to 501.9) are map objects; where ALKIS or the survey draws the
    // paving they are the same object (ADR 0079).
    let ground: Vec<MapObject> = paved
        .candidates
        .iter()
        .chain(&open_land.map)
        .chain(&open_land.candidates)
        .cloned()
        .collect();
    let generalized = generalize_points(
        points,
        cut_firm_paths(osm_objects(Target::Map)?, &paved, &georef, &symbol_set),
        &out_of_bounds,
        &paved.map,
        &ground,
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
    let mut candidates = MapDocument {
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
                .chain(open_land.candidates)
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
    // Parks under everything but the walkable area: colour priority puts yellow low.
    map_objects.extend(open_land.map);
    map_objects.extend(paved.map);
    map_objects.extend(water.map);
    map_objects.extend(generalized.map);
    // What goes onto the map and what stays a candidate, by the project's content settings
    // (`build/content.rs`): the objects of the cadastre and the registers go to the map by
    // default.
    let settings = content_settings(&project);
    route(
        &settings,
        &mut map_objects,
        &mut candidates.parts[0].objects,
        &symbol_set,
    );
    // Contours and north lines stop at paving and paths (`build/line_cuts.rs`).
    let ways = way_footprints(
        &map_objects,
        &candidates.parts[0].objects,
        &map_objects,
        project.map_kind,
        &georef,
        &symbol_set,
    );
    let min_piece_m = f64::from(project.scale_denominator) / 1000.0;
    for list in [&mut candidates.parts[0].objects, &mut map_objects] {
        cut_lines_at_ways(
            list,
            &CONTOUR_CODES,
            &ways,
            min_piece_m,
            &georef,
            &symbol_set,
        );
    }
    cut_lines_at_ways(
        &mut map_objects,
        &NORTH_LINE_CODES,
        &ways,
        min_piece_m,
        &georef,
        &symbol_set,
    );
    // The border of the paved areas, 501.1 (`build/paved_borders.rs`).
    border_paved_areas(
        &mut [&mut map_objects, &mut candidates.parts[0].objects],
        &footprints,
        &water.union,
        &clip,
        &georef,
        &symbol_set,
    );
    // The last step on the lines: the ticks of fences point to the olive green area beside them,
    // those of earth banks down the slope (`build/ticks.rs`).
    orient_ticks(
        &mut [&mut map_objects, &mut candidates.parts[0].objects],
        read_terrain(dir).as_ref(),
        &georef,
        &symbol_set,
    );
    // Then the points keep the norm's gap to walls and fences, on the tick side with room for
    // the ticks (`build/barrier_gaps.rs`).
    move_points_off_barriers(
        &mut [&mut map_objects, &mut candidates.parts[0].objects],
        project.map_kind,
        &symbol_set,
    );
    let buildings_part = MapPart {
        name: "default part".to_string(),
        objects: map_objects,
    };
    let mut attribution = vec![OSM_ATTRIBUTION_DE.to_string()];
    attribution.extend(sheet_credit_lines(&project, &registers));
    attribution.push(FREEMAPPER_CREDIT_DE.to_string());
    let info = PeripheralInfo {
        title: project.display_title(dir),
        scale_denominator: project.scale_denominator,
        contour_interval_m: project.map_kind.contour_interval_m(),
        attribution,
    };
    // The walkable area (sprint only, ADR 0069) in its own part, written first.
    let mut parts: Vec<MapPart> = walkable_base(&content, project.map_kind, &symbol_set)
        .map(|base| MapPart {
            name: BASE_PART.to_string(),
            objects: vec![base],
        })
        .into_iter()
        .collect();
    parts.push(buildings_part);
    parts.push(MapPart {
        name: SHEET_PART.to_string(),
        objects: info_block(&content, &info, &symbol_set),
    });
    let sheet_info = info;
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
        content,
        info: sheet_info,
        content_settings: settings,
    })
}

/// The content settings of a project: `balanced` when it has none (ADR 0100). The unit tests of
/// the build look at where the build puts each object, so they run with `safe`; a test of the
/// default sets `balanced` itself.
fn content_settings(project: &ProjectFile) -> crate::build::content::ContentSettings {
    use crate::build::content::{ContentSettings, Profile};
    project.content.clone().unwrap_or_else(|| {
        if cfg!(test) {
            ContentSettings::with_profile(Profile::Safe)
        } else {
            ContentSettings::default()
        }
    })
}

pub(crate) const CANDIDATES_RELPATH: &str = "templates/candidates.omap";

/// Line symbol used for the map area outline in the candidates template map.
pub(super) const OUTLINE_SYMBOL_CODE: &str = "501.1";

/// The exterior ring of the map area as a closed path object.
pub(super) fn map_area_outline(
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

pub(super) fn write_document(
    dir: &Path,
    doc: &MapDocument,
    path: &Path,
) -> Result<(), ProjectError> {
    // Written whole and then renamed: a full disk leaves the previous map as it was.
    let mut bytes = Vec::new();
    write_map(doc, &mut bytes, XmlFormat::Compact)?;
    Ok(write_inside(dir, path, &bytes)?)
}

/// [`crate::fsutil::copy_inside`] with the error as [`ProjectError::Io`].
pub(super) fn copy_inside(dir: &Path, from: &Path, to: &Path) -> Result<(), ProjectError> {
    crate::fsutil::copy_inside(dir, from, to).map_err(|(path, e)| ProjectError::Io(path, e))
}

/// Copies the map `map` to its backup `backup` before it is replaced. A map that is a link is
/// not copied; the error then says what to do.
pub(super) fn back_up(dir: &Path, map: &Path, backup: &Path) -> Result<(), ProjectError> {
    crate::fsutil::copy_inside(dir, map, backup).map_err(|(path, e)| {
        if e.to_string() == crate::fsutil::NOT_COPIED {
            let message = format!("{}; {LINKED_MAP_HINT}", crate::fsutil::NOT_COPIED);
            ProjectError::Io(path, std::io::Error::other(message))
        } else {
            ProjectError::Io(path, e)
        }
    })
}
