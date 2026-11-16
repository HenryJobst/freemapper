//! The documents of a project and the files written from them: the starter project (map and
//! candidate layers, ADR 0156), the OCAD files and the PDFs (ADR 0091: built once per command).
//!
//! What the build reads is stored under `data/` and `templates/`; none of the exports writes a
//! file the build reads.

use super::*;
use crate::build::assembly::{Buildings, Built, Frame, Layer, Stored, build};
use crate::build::buildings::object_footprints;
use crate::build::candidate_layers::{CandidateLayer, split};
use crate::build::contours::Contours;
use crate::build::landforms::Landforms;
use crate::build::ticks::read_terrain;
use crate::build::training::train;
use crate::build::vegetation::Vegetation;

/// The stored ALKIS buildings, read once per export: parsed, and the raw features (the Berlin
/// building parts are picked from these).
pub(super) struct StoredBuildings {
    pub(super) buildings: Vec<crate::sources::alkis::Building>,
    pub(super) features: Vec<serde_json::Value>,
}

impl StoredBuildings {
    /// The 521 footprints (buildings that become map objects), in the map's CRS.
    fn footprints(&self) -> Vec<geo_types::Polygon<f64>> {
        object_footprints(&self.buildings)
    }

    /// The footprints of every building, canopies and the other non-map buildings included.
    pub(super) fn all_footprints(&self) -> Vec<geo_types::Polygon<f64>> {
        self.buildings
            .iter()
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
    let mut features = freemapper_sources::layer_file::parse_layer(&text).map_err(|e| {
        ProjectError::StoredBuildings {
            path: path.clone(),
            message: e.to_string(),
        }
    })?;
    // A set may date the buildings from one of its layers, if stored. It is an optional source:
    // an unreadable file leaves the buildings undated.
    if let Some(dates) =
        registers_drawing().and_then(|(drawing, _)| drawing.footprint_dates.as_ref())
        && let Some(layer) = known_berlin_layer(dates.layer)
        && let Ok(text) = std::fs::read_to_string(berlin_layer_path(dir, layer))
        && let Ok(dating) = freemapper_sources::layer_file::parse_layer(&text)
    {
        (dates.join)(&mut features, &dating);
    }
    let buildings = parse_building_features(&features).buildings;
    Ok(Some(StoredBuildings {
        buildings,
        features,
    }))
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

/// The contour lines `derive` stored, with the interval they were traced at and the terrain
/// model they come from; none when there are none. The derive stores an interval that is not
/// the kind's; without one the lines are at the kind's, whatever freemapper.toml says now.
pub(super) fn read_stored_contours(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Option<Contours>, ProjectError> {
    let path = dir.join(CONTOURS_RELPATH);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let (lines, stored) =
        contours_from_geojson(&text).map_err(|e| ProjectError::StoredCandidates {
            path: path.clone(),
            message: e.to_string(),
        })?;
    let (source, currency) = terrain_provenance(project)?;
    Ok(Some(Contours {
        lines,
        interval: stored.unwrap_or_else(|| project.map_kind.contour_interval_m()),
        source,
        currency,
    }))
}

/// The terrain model the derived candidates come from (that of the area's raster provider) and
/// the newest currency of its tiles, as tag values.
fn terrain_provenance(project: &ProjectFile) -> Result<(String, Option<String>), ProjectError> {
    // The terrain model of the area's raster provider.
    let provider = match &project.provider {
        Some(provider) => Some(provider.clone()),
        None => raster_provider(&project.region_bbox()?),
    };
    let currency = project
        .tiles
        .iter()
        .filter(|t| t.product == Product::Dtm)
        .map(|t| t.currency)
        .max();
    Ok((
        terrain_source(provider.as_ref()),
        currency.map(|c| c.to_string()),
    ))
}

/// The vegetation areas `derive` stored (ADR 0105), with the source of their point cloud; none
/// when there are none or the area has no point cloud backend.
pub(super) fn read_stored_vegetation(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Option<Vegetation>, ProjectError> {
    use crate::derive::vegetation::vegetation_from_geojson;
    let path = dir.join(VEGETATION_RELPATH);
    if !path.exists() {
        return Ok(None);
    }
    let Some(backend) = point_cloud_backend(project, &project.region_bbox()?) else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let areas =
        vegetation_from_geojson(&text).map_err(|message| ProjectError::StoredCandidates {
            path: path.clone(),
            message,
        })?;
    Ok(Some(Vegetation {
        areas,
        source: backend.info().source.clone(),
        currency: vegetation_currency(&project.point_clouds),
    }))
}

/// The day the vegetation stands for: that of its oldest tile, among those that name one. An
/// area can lie on any of the tiles, so nothing newer can be said of it.
pub(super) fn vegetation_currency(tiles: &[PointCloudRecord]) -> Option<NaiveDate> {
    tiles.iter().filter_map(|tile| tile.published).min()
}

/// The landforms `derive` stored (ADR 0104), with the terrain model they come from; none when
/// there are none.
pub(super) fn read_stored_landforms(
    dir: &Path,
    project: &ProjectFile,
) -> Result<Option<Landforms>, ProjectError> {
    let path = dir.join(LANDFORMS_RELPATH);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path.clone(), e))?;
    let forms = landforms_from_geojson(&text).map_err(|e| ProjectError::StoredCandidates {
        path: path.clone(),
        message: e.to_string(),
    })?;
    let (source, currency) = terrain_provenance(project)?;
    Ok(Some(Landforms {
        forms,
        source,
        currency,
    }))
}

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
    let symbol_set = project.map_kind.symbol_set(project.scale_denominator);
    let Some(stored) = read_stored_buildings(dir)? else {
        return Ok(false);
    };
    if let Some(record) = &project.buildings
        && !record.covers(&project.bbox()?)
    {
        return Err(ProjectError::BuildingsNotFetched);
    }
    Ok(!building_objects(
        &stored.buildings,
        &area.projected_polygon_in(project.crs()),
        project.buffer_m,
        &georef,
        &symbol_set,
    )?
    .is_empty())
}

/// How a building object's provenance appears in the map file.
pub(super) const ALKIS_SOURCE_TAG: &str = r#"<t k="freemapper:source">alkis</t>"#;

/// Whether the stored layers and buildings give 522 canopy map objects that the map at
/// `map_text` lacks (it was written before them, or without the layers).
pub(super) fn map_lacks_canopies(dir: &Path, map_text: &str) -> Result<bool, ProjectError> {
    if canopy_source_tags()
        .iter()
        .any(|tag| map_text.contains(tag))
    {
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
    let symbol_set = project.map_kind.symbol_set(project.scale_denominator);
    let stored = read_stored_buildings(dir)?;
    let footprints = stored.as_ref().map_or_else(Vec::new, |b| b.footprints());
    let map_area = area.projected_polygon_in(project.crs());
    if let Some((drawing, crs)) = registers_drawing() {
        for layer in layers
            .iter()
            .filter(|l| drawing.canopy_areas.iter().any(|a| a.layer == l.layer.file))
        {
            if !canopy_objects(
                &[CanopyInput::Set {
                    drawing,
                    layer: layer.layer.file,
                    features: &layer.features,
                    crs,
                }],
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
        }
    }
    // The canopies among the ALKIS buildings of any state (`Überdachung`, `Carport`).
    let (Some(stored), Some(record)) = (&stored, &project.buildings) else {
        return Ok(false);
    };
    Ok(!canopy_objects(
        &[CanopyInput::Buildings(&stored.features)],
        &footprints,
        &map_area,
        project.buffer_m,
        &georef,
        &symbol_set,
        record.fetched,
    )?
    .is_empty())
}

/// How a canopy object's provenance appears in the map file: a canopy of a set, or an ALKIS
/// vereinfacht one (the buildings share `freemapper:source` = `alkis`, so the kind tells them
/// apart).
pub(super) fn canopy_source_tags() -> Vec<String> {
    let mut tags: Vec<String> = crate::build::sets::all_drawings()
        .flat_map(|d| {
            d.canopy_areas.iter().map(move |a| {
                format!(
                    r#"<t k="freemapper:source">{}/{}</t>"#,
                    d.source_prefix, a.layer
                )
            })
        })
        .collect();
    tags.extend(
        ["Überdachung", "Carport"].map(|kind| format!(r#"<t k="freemapper:alkis">{kind}</t>"#)),
    );
    tags
}

/// Writes the starter project (map, candidate layers, attribution) into `dir` and
/// returns the path of the map file.
///
/// An existing map file may already contain the mapper's work: it is only replaced when
/// `force` is set, and then kept as `<name>.omap.bak`.
pub fn export_starter_project(dir: &Path, force: bool) -> Result<PathBuf, ProjectError> {
    write_starter_project(dir, &starter_documents(dir)?, MapWrite::Write { force })
}

/// [`export_starter_project`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_starter_project_from(
    dir: &Path,
    documents: &StarterDocuments,
    force: bool,
) -> Result<PathBuf, ProjectError> {
    write_starter_project(dir, documents, MapWrite::Write { force })
}

/// What [`write_starter_project`] does with the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MapWrite {
    /// Writes it; an existing map only with `force`, kept as `<name>.omap.bak`.
    Write { force: bool },
    /// Keeps it, as it may contain the mapper's work. `old_candidates`: it names the candidates
    /// template of an older freemapper, [`OLD_CANDIDATES_RELPATH`].
    Keep { old_candidates: bool },
}

/// Writes the candidate layers and the attribution, and the map unless it is kept. Candidates
/// are templates that always follow the current map area. A written map names the layers, and
/// the candidates template of an older freemapper goes; a kept map that still names it gets it
/// refreshed with every candidate (ADR 0156).
pub(super) fn write_starter_project(
    dir: &Path,
    documents: &StarterDocuments,
    map_write: MapWrite,
) -> Result<PathBuf, ProjectError> {
    let StarterDocuments {
        map,
        candidates,
        attribution_text,
        ..
    } = documents;
    let map_path = map_path(dir);
    if let MapWrite::Write { force } = map_write
        && map_path.exists()
    {
        if !force {
            return Err(ProjectError::MapExists(map_path));
        }
        back_up(dir, &map_path, &map_path.with_extension("omap.bak"))?;
    }
    for (layer, doc) in documents.candidate_layers() {
        write_document(dir, &doc, &dir.join(layer.relpath()))?;
    }
    let old_candidates = dir.join(OLD_CANDIDATES_RELPATH);
    match map_write {
        MapWrite::Write { .. } => {
            forget_map_preview(dir)?;
            write_document(dir, map, &map_path)?;
            freemapper_fs::remove_inside(dir, &old_candidates);
        }
        MapWrite::Keep {
            old_candidates: true,
        } => write_document(dir, candidates, &old_candidates)?,
        MapWrite::Keep {
            old_candidates: false,
        } => {}
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
    /// The kept map names `<name>-candidates.ocd` of an older freemapper as a background map; the
    /// file was filled with every candidate, the layers were written beside it (ADR 0156).
    pub old_candidates: bool,
    /// `ocad/<name>-candidates-<layer>.ocd`, one per candidate layer in its order (ADR 0156).
    pub candidates: Vec<PathBuf>,
    pub objects: usize,
    /// Objects OCAD cannot take (no symbol for OCAD, no area, outside its drawing area).
    pub skipped: usize,
    /// Degenerate line parts and hole rings left out of written objects.
    pub dropped_parts: usize,
    /// Templates left out of the OCAD map.
    pub warnings: Vec<Note>,
    /// The kind of the map, which names the official symbol set to load in OCAD.
    pub map_kind: MapKind,
}

/// The world file next to an image: `.jpg` → `.jgw`, `.png` → `.pgw`.
pub(super) fn world_file_path(image: &Path) -> Option<PathBuf> {
    let ext = image.extension()?.to_str()?;
    let mut chars = ext.chars();
    let (first, last) = (chars.next()?, ext.chars().last()?);
    Some(image.with_extension(format!("{first}{last}w")))
}

/// Writes the OCAD starting point into `ocad/`: `<name>.ocd` (OCAD 12) with the map objects, one
/// `<name>-candidates-<layer>.ocd` per candidate layer, and the raster templates with their world
/// files as background maps; the candidate layers are hidden background maps (ADR 0156). An
/// existing `<name>.ocd` is only replaced when `force` is set (kept as `.ocd.bak`); then
/// `<name>-candidates.ocd` of an older freemapper goes, while a kept map that names it as a
/// background map gets it refreshed with every candidate.
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
    let name = ocad_name(dir);
    let (mut backgrounds, warnings) = copy_backgrounds(dir, &ocad_dir, &documents.map.templates)?;
    let mut candidates = Vec::new();
    for (layer, doc) in documents.candidate_layers() {
        let file_name = format!("{name}-candidates-{}.ocd", layer.name());
        backgrounds.push(Background::Map {
            file_name: file_name.clone(),
            visible: false,
        });
        let path = ocad_dir.join(&file_name);
        let (bytes, _) = map_to_ocd(&doc, project.map_kind, &[])?;
        write_inside(dir, &path, &bytes)?;
        candidates.push(path);
    }
    let old_name = format!("{name}-candidates.ocd");
    let old_candidates = ocad_dir.join(&old_name);
    let map = ocad_dir.join(format!("{name}.ocd"));
    let kept = map.exists() && !force;
    let (bytes, report) = map_to_ocd(&documents.map, project.map_kind, &backgrounds)?;
    // A kept map of an older freemapper names the one candidates file as its background map;
    // a map restored from its backup too. A map that cannot be read names nothing.
    let kept_names_old = kept
        && std::fs::read(&map)
            .map(|b| crate::ocd::file::background_names(&b).contains(&old_name))
            .unwrap_or(false);
    if kept {
        if kept_names_old {
            let (bytes, _) = map_to_ocd(&documents.candidates, project.map_kind, &[])?;
            write_inside(dir, &old_candidates, &bytes)?;
        }
    } else {
        if map.exists() {
            back_up(dir, &map, &map.with_extension("ocd.bak"))?;
        }
        write_inside(dir, &map, &bytes)?;
        freemapper_fs::remove_inside(dir, &old_candidates);
    }
    Ok(OcadOutcome {
        map,
        kept,
        old_candidates: kept_names_old,
        candidates,
        objects: report.objects,
        skipped: report.skipped,
        dropped_parts: report.dropped_parts,
        warnings,
        map_kind: project.map_kind,
    })
}

/// The name of the OCAD files of the project in `dir`: OCAD finds background maps by the ASCII
/// names in the parameter strings, so the files get the same ASCII names ("Straußberg" →
/// "Straussberg").
fn ocad_name(dir: &Path) -> String {
    map_path(dir)
        .file_stem()
        .map(|s| {
            crate::ocd::symbols::ascii(&s.to_string_lossy())
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "map".to_string())
}

/// Copies the image `templates` with their world files into `ocad_dir` and returns them as
/// OCAD background maps. A template that is a link or lies outside the project folder is left
/// out with a warning, so no file from elsewhere is copied into the project.
pub(super) fn copy_backgrounds(
    dir: &Path,
    ocad_dir: &Path,
    templates: &[TemplateRef],
) -> Result<(Vec<Background>, Vec<Note>), ProjectError> {
    let planned = plan_backgrounds(dir, ocad_dir, templates)?;
    planned.copy(dir)?;
    Ok((planned.backgrounds, planned.warnings))
}

/// The OCAD background maps of some templates before anything is copied ([`plan_backgrounds`]).
pub(super) struct PlannedBackgrounds {
    pub(super) backgrounds: Vec<Background>,
    /// The files to copy into the OCAD folder, from and to.
    copies: Vec<(PathBuf, PathBuf)>,
    /// Templates left out.
    pub(super) warnings: Vec<Note>,
}

impl PlannedBackgrounds {
    /// Copies the images and world files into the OCAD folder.
    fn copy(&self, dir: &Path) -> Result<(), ProjectError> {
        for (from, to) in &self.copies {
            copy_inside(dir, from, to)?;
        }
        Ok(())
    }
}

/// [`copy_backgrounds`] without the copying: the background maps, the files to copy and the
/// warnings, so the OCAD map can be built before anything is written.
pub(super) fn plan_backgrounds(
    dir: &Path,
    ocad_dir: &Path,
    templates: &[TemplateRef],
) -> Result<PlannedBackgrounds, ProjectError> {
    let (mut backgrounds, mut copies, mut warnings) = (Vec::new(), Vec::new(), Vec::new());
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
        copies.push((image.clone(), ocad_dir.join(image_name)));
        copies.push((world.clone(), ocad_dir.join(world_name)));
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
    Ok(PlannedBackgrounds {
        backgrounds,
        copies,
        warnings,
    })
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
    let candidate_objects =
        visible_candidates(documents).take(if candidates { usize::MAX } else { 0 });
    let map = own
        .iter()
        .flat_map(|p| &p.objects)
        .chain(candidate_objects)
        .collect();
    (map, sheet.iter().flat_map(|p| &p.objects).collect())
}

/// The candidates a PDF draws: all but the map area outline (the first candidate) and the hidden
/// ones (`freemapper:hidden`: points under a 520 candidate, rock and passages taken off, the
/// carriageways of the street survey). What the light level took off the map is among them.
fn visible_candidates(documents: &StarterDocuments) -> impl Iterator<Item = &MapObject> {
    documents
        .candidates
        .parts
        .iter()
        .flat_map(|p| p.objects.iter().skip(1))
        .filter(|o| !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY))
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

/// Writes `<name>-candidates.pdf`, the check print: the map objects and all candidates drawn
/// as they are with the project's symbol set (this deliberately goes beyond "only reliable data
/// as map objects"), the peripheral information on the sheet. The training map
/// ([`export_training`]) is the generalized print of the same objects.
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

/// The objects of the training map of a forest or sprint map, its rock generalized at the
/// training level:
/// the cliffs of the map and the visible candidates together, the contours cut below the rock
/// drawn, and the knolls and depressions on the rock left out.
fn training_rock(
    dir: &Path,
    kind: MapKind,
    objects: &[&MapObject],
    documents: &StarterDocuments,
) -> Vec<MapObject> {
    use crate::build::cliffs::{Ground, Level, generalize_cliffs, hide_landforms_on_rock};
    use crate::build::line_cuts::{CONTOUR_CODES, cut_lines_at_ways};
    use crate::derive::contours::MIN_LENGTH_MM as CONTOUR_MIN_LENGTH_MM;
    let georef = &documents.map.georef;
    let symbol_set = &documents.map.symbol_set;
    let terrain = read_terrain(dir).or_else(|| {
        crate::derive::terrain::read_float_raster(&dir.join(TERRAIN_MOSAIC_RELPATH)).ok()
    });
    let mut list: Vec<MapObject> = objects.iter().map(|o| (*o).clone()).collect();
    let mut gone = Vec::new();
    let rock_seen = crate::derive::terrain::read_float_raster(&dir.join(ROCK_SEEN_RELPATH)).ok();
    let rock = generalize_cliffs(
        &mut list,
        &mut gone,
        Ground {
            terrain: terrain.as_ref(),
            rock_seen: rock_seen.as_ref(),
        },
        kind,
        georef,
        symbol_set,
        &Level::training(kind),
    );
    if !rock.stops.is_empty() {
        let min_contour_m =
            CONTOUR_MIN_LENGTH_MM.metres(SymbolScale::new(kind, georef.scale_denominator));
        cut_lines_at_ways(
            &mut list,
            &CONTOUR_CODES,
            &rock.stops,
            min_contour_m,
            georef,
            symbol_set,
        );
    }
    hide_landforms_on_rock(&mut list, &mut gone, &rock.mask, georef);
    list
}

/// The file `export_pdf` or `export_candidates_pdf` writes.
pub fn pdf_path(dir: &Path, candidates: bool) -> PathBuf {
    if candidates {
        beside_map(dir, "-candidates.pdf")
    } else {
        map_path(dir).with_extension("pdf")
    }
}

/// `<name>-training.omap`, the training map [`export_training`] writes.
pub fn training_map_path(dir: &Path) -> PathBuf {
    beside_map(dir, "-training.omap")
}

/// `<name>-training.pdf`, the print of the training map.
pub fn training_pdf_path(dir: &Path) -> PathBuf {
    beside_map(dir, "-training.pdf")
}

/// `ocad/<name>-training.ocd`, the OCAD file of the training map.
pub fn training_ocad_path(dir: &Path) -> PathBuf {
    dir.join(OCAD_DIR)
        .join(format!("{}-training.ocd", ocad_name(dir)))
}

/// The file beside the map whose name is the map's followed by `suffix`.
fn beside_map(dir: &Path, suffix: &str) -> PathBuf {
    let map = map_path(dir);
    let stem = map
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    map.with_file_name(format!("{stem}{suffix}"))
}

/// What [`export_training`] wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainingOutcome {
    pub map: PathBuf,
    pub pdf: PathBuf,
    /// `ocad/<name>-training.ocd`, when OCAD output was asked for.
    pub ocad: Option<PathBuf>,
    /// What the training level did.
    pub report: TrainingReport,
    /// Templates left out of the OCAD training map, and a missing tree cover where the project
    /// has an nDSM.
    pub warnings: Vec<Note>,
}

/// Writes the training map, its PDF and, with `ocad`, `ocad/<stem>-training.ocd`. An existing
/// training map is kept unless `force` (then `<stem>-training.omap.bak`).
///
/// The training map is the map and every visible candidate generalized at the training level
/// (`build/training`), a map of its own that is printed as it is. Its OCAD file is kept as the
/// OOM one (backup `.ocd.bak`); both are checked before anything is written, and every file is
/// built before the first is written, so a failure leaves none of them half done.
pub fn export_training(
    dir: &Path,
    force: bool,
    scale_bar: bool,
    sources: PdfSources,
    ocad: bool,
) -> Result<TrainingOutcome, ProjectError> {
    export_training_from(
        dir,
        &starter_documents(dir)?,
        force,
        scale_bar,
        sources,
        ocad,
    )
}

/// [`export_training`] from `documents`, which [`starter_documents`] built for `dir`.
pub fn export_training_from(
    dir: &Path,
    documents: &StarterDocuments,
    force: bool,
    scale_bar: bool,
    sources: PdfSources,
    ocad: bool,
) -> Result<TrainingOutcome, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let map_path = training_map_path(dir);
    let ocad_path = ocad.then(|| training_ocad_path(dir));
    // Either file may hold the mapper's work.
    if !force
        && let Some(path) = std::iter::once(&map_path)
            .chain(&ocad_path)
            .find(|path| path.exists())
    {
        return Err(ProjectError::MapExists(path.clone()));
    }
    if ocad {
        check_folders(dir, &[OCAD_DIR])?;
    }
    let kind = project.map_kind;
    let (georef, symbol_set) = (&documents.map.georef, &documents.map.symbol_set);
    // The map's own objects before the light level, the visible candidates and the hidden
    // carriageways (`build::training::TrainingInput`); the walkable area and the sheet stay as
    // they are.
    let input: Vec<&MapObject> = documents.training.objects().collect();
    // The rock of a forest or sprint map is generalized more than on the map (ADR 0126): the
    // training map is printed as it is and must stay readable; the candidates keep every cliff.
    let objects = if matches!(kind, MapKind::Forest | MapKind::Sprint) {
        training_rock(dir, kind, &input, documents)
    } else {
        input.into_iter().cloned().collect()
    };
    let tree_cover = read_float_raster(&dir.join(TREE_COVER_RELPATH)).ok();
    let mut warnings = Vec::new();
    // A derive of an older freemapper wrote the nDSM but no tree cover: the parks stay whole.
    if tree_cover.is_none() && dir.join(NDSM_RELPATH).exists() {
        warnings.push(note!("warning-training-no-tree-cover"));
    }
    let (trained, report) = train(objects, tree_cover.as_ref(), kind, georef, symbol_set);
    let part = |name: &'static str| documents.map.parts.iter().filter(move |p| p.name == name);
    let mut parts: Vec<MapPart> = part(BASE_PART).cloned().collect();
    parts.push(MapPart {
        name: "default part".to_string(),
        objects: trained,
    });
    parts.extend(part(SHEET_PART).cloned());
    let training = MapDocument {
        notes: documents.map.notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts,
        templates: documents
            .map
            .templates
            .iter()
            .filter(|t| t.kind != TemplateKind::Map)
            .cloned()
            .collect(),
    };

    // Every file is built before the first is written.
    let map_bytes = document_bytes(&training)?;
    let drawn: Vec<&MapObject> = training
        .parts
        .iter()
        .filter(|p| p.name != SHEET_PART)
        .flat_map(|p| &p.objects)
        .collect();
    let pdf_bytes = pdf_bytes(dir, &project, documents, &drawn, scale_bar, sources)?;
    let ocad_dir = dir.join(OCAD_DIR);
    let ocad_files = match &ocad_path {
        Some(path) => {
            let planned = plan_backgrounds(dir, &ocad_dir, &training.templates)?;
            let (bytes, _) = map_to_ocd(&training, kind, &planned.backgrounds)?;
            Some((path, bytes, planned))
        }
        None => None,
    };

    if map_path.exists() {
        back_up(dir, &map_path, &map_path.with_extension("omap.bak"))?;
    }
    write_inside(dir, &map_path, &map_bytes)?;
    let pdf = training_pdf_path(dir);
    write_inside(dir, &pdf, &pdf_bytes)?;
    if let Some((path, bytes, planned)) = ocad_files {
        std::fs::create_dir_all(&ocad_dir).map_err(|e| ProjectError::Io(ocad_dir.clone(), e))?;
        if path.exists() {
            back_up(dir, path, &path.with_extension("ocd.bak"))?;
        }
        write_inside(dir, path, &bytes)?;
        planned.copy(dir)?;
        warnings.extend(planned.warnings);
    }
    Ok(TrainingOutcome {
        map: map_path,
        pdf,
        ocad: ocad_path,
        report,
        warnings,
    })
}

pub(super) fn write_pdf_file(
    dir: &Path,
    documents: &StarterDocuments,
    candidates: bool,
    scale_bar: bool,
    sources: PdfSources,
) -> Result<PathBuf, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let (map, _) = pdf_objects(documents, candidates);
    let bytes = pdf_bytes(dir, &project, documents, &map, scale_bar, sources)?;
    let path = pdf_path(dir, candidates);
    write_inside(dir, &path, &bytes)?;
    Ok(path)
}

/// The PDF of `map` clipped to the map area of `project` (in `dir`) and the sheet of
/// `documents`, its source block by `sources` and the scale bar only with `scale_bar`.
pub(super) fn pdf_bytes(
    dir: &Path,
    project: &ProjectFile,
    documents: &StarterDocuments,
    map: &[&MapObject],
    scale_bar: bool,
    sources: PdfSources,
) -> Result<Vec<u8>, ProjectError> {
    use crate::render::{draw, pdf, symbols::RenderSymbols};
    let symbols = RenderSymbols::from_set(&documents.map.symbol_set);
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
    Ok(pdf::write_pdf(
        &render(map),
        &render(&sheet),
        clip,
        &symbols.colours,
        &project.display_title(dir),
    ))
}

/// The preview of the map: a lossless WebP of what the map PDF draws in the map area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapImage {
    pub webp: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// What `data/map-preview.json` says about the preview next to the map: the project it was
/// exported for, by the fields that decide the build ([`project_fingerprint`]), and the content
/// settings it was placed with.
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
/// any candidate. Texts are left out (see [`crate::render::raster`]). The map area is the one
/// the documents were built for (`documents.content`, the rectangle of the project's bounding
/// box on paper), not a new read of `freemapper.toml`: a save meanwhile cannot mix two areas.
/// `dir` only names the file in an error.
pub fn map_preview_webp(
    documents: &StarterDocuments,
    dir: &Path,
) -> Result<MapImage, ProjectError> {
    use crate::render::{draw, raster, symbols::RenderSymbols};
    let symbols = RenderSymbols::from_set(&documents.map.symbol_set);
    let (map, _) = pdf_objects(documents, false);
    let primitives: Vec<draw::Primitive> = map
        .iter()
        .flat_map(|o| draw::primitives(o, &symbols))
        .collect();
    let clip = paper_box(&documents.content);
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

/// [`export_map_preview_from`] right after the map was written: a failure is the warning
/// `warning-map-preview-failed`, not an error, because the map is written and the outputs after
/// it (OCAD, PDFs) still run. The old preview went before the map was replaced, so the folder
/// then has none rather than a wrong one. The path of the preview when it was written.
pub fn export_map_preview_or_warn(
    dir: &Path,
    documents: &StarterDocuments,
    progress: &mut dyn FnMut(Note),
) -> Option<PathBuf> {
    match export_map_preview_from(dir, documents) {
        Ok(path) => Some(path),
        Err(e) => {
            progress(note!("warning-map-preview-failed", error = Arg::error(e)));
            None
        }
    }
}

/// SHA-256 (hex) of the fields of `project` that decide which map the stored data builds: map
/// kind, scale, declination date, buffer, the map area and its source, CRS and raster provider.
/// Equal for two projects that differ only in what does not decide that, so the stored data and
/// a preview still fit the project after such a change:
/// - the content settings, which [`place`] applies to the build;
/// - the title, which only the sheet shows, and the preview never draws the sheet;
/// - the records of the fetch (tiles, ALKIS, registers, OSM, rasters, nDSM), which every online
///   fetch rewrites, at least with today's date. Data that really changed changes the files
///   under `data/` and `templates/`, and whoever caches a build watches those.
///
/// An allow-list, not "everything but": the fields are taken apart without `..`, so a field
/// added to [`ProjectFile`] does not compile until it is sorted in here, and a new record cannot
/// silently start to count.
pub fn project_fingerprint(project: &ProjectFile) -> Result<String, ProjectError> {
    use sha2::Digest as _;
    /// The fields that count, in the TOML the library writes for them.
    #[derive(Serialize)]
    struct Counted<'a> {
        map_kind: &'a MapKind,
        scale_denominator: u32,
        declination_date: &'a chrono::NaiveDate,
        buffer_m: f64,
        map_area_source_id: &'a Option<String>,
        map_area: &'a serde_json::Value,
        crs: u32,
        provider: &'a Option<AdapterId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contour_interval_m: &'a Option<f64>,
    }
    let ProjectFile {
        map_kind,
        scale_denominator,
        declination_date,
        buffer_m,
        map_area_source_id,
        map_area,
        crs,
        provider,
        contour_interval_m,
        title: _,
        tiles: _,
        buildings: _,
        land_use: _,
        extra_sets: _,
        nas_layers: _,
        tree_layers: _,
        osm: _,
        orthophoto: _,
        k5: _,
        terrain: _,
        surface: _,
        ndsm_tiles: _,
        point_clouds: _,
        content: _,
        contours: _,
    } = project;
    let counted = Counted {
        map_kind,
        scale_denominator: *scale_denominator,
        declination_date,
        buffer_m: *buffer_m,
        map_area_source_id,
        map_area,
        crs: *crs,
        provider,
        contour_interval_m,
    };
    let text = toml::to_string(&counted).map_err(|e| ProjectError::Toml(e.to_string()))?;
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

/// The record of the preview in `dir`; none when there is none, it does not read, or it is not a
/// file of the project folder ([`read_inside`]).
pub fn read_map_preview_record(dir: &Path) -> Option<MapPreviewRecord> {
    let bytes = read_inside(dir, &dir.join(MAP_PREVIEW_RECORD_RELPATH))?;
    serde_json::from_slice(&bytes).ok()
}

/// The record of the stored preview: [`read_map_preview_record`] when the image is there too,
/// as a file of the project folder. Without both there is no preview.
pub fn stored_map_preview_record(dir: &Path) -> Option<MapPreviewRecord> {
    let record = read_map_preview_record(dir)?;
    let image = map_preview_path(dir);
    (!is_link(&image) && freemapper_fs::stays_inside(dir, &image) && image.is_file())
        .then_some(record)
}

/// The stored preview, `<name>-preview.webp`, when it has its record; none otherwise.
pub fn read_map_preview(dir: &Path) -> Option<Vec<u8>> {
    read_map_preview_record(dir)?;
    read_inside(dir, &map_preview_path(dir))
}

/// The file `path` of the project folder `dir`, read only when neither it nor a folder on the
/// way is a link that leads elsewhere: the area page serves what it reads, and a planted link
/// must not make it serve another file. Writing is guarded the same way (`write_files_inside`).
fn read_inside(dir: &Path, path: &Path) -> Option<Vec<u8>> {
    if is_link(path) || !freemapper_fs::stays_inside(dir, path) {
        return None;
    }
    std::fs::read(path).ok()
}

/// Whether `path` itself is a link.
fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
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
    /// The objects the training map starts from: the map's own objects before the light level
    /// and the candidates it draws.
    pub(super) training: crate::build::training::TrainingInput,
    /// What the light level took off the map (ADR 0157).
    pub(super) light: crate::build::light::LightReport,
}

impl StarterDocuments {
    /// What the light level did to the map of these documents (ADR 0157).
    pub fn light(&self) -> &crate::build::light::LightReport {
        &self.light
    }

    /// The candidates cut into the template maps of the candidate layers, every layer in its
    /// order (ADR 0156): together they are [`Self::candidates`], with the outline of the map area
    /// in each.
    pub(super) fn candidate_layers(&self) -> Vec<(CandidateLayer, MapDocument)> {
        let objects: Vec<MapObject> = self
            .candidates
            .parts
            .iter()
            .flat_map(|p| p.objects.iter().cloned())
            .collect();
        split(&objects, &self.candidates.symbol_set)
            .into_iter()
            .map(|(layer, objects)| {
                let doc = MapDocument {
                    parts: vec![MapPart {
                        name: "candidates".to_string(),
                        objects,
                    }],
                    ..self.candidates.clone()
                };
                (layer, doc)
            })
            .collect()
    }
}

/// Builds the documents of the project in `dir` from its stored data. They belong to that folder
/// and to the data as stored at this moment: build them again after a fetch or a derive.
pub fn starter_documents(dir: &Path) -> Result<StarterDocuments, ProjectError> {
    let built = build_project(dir)?;
    place(&built, &built.saved_content)
}

/// Every object of a project built from its stored data, before the content settings place them
/// ([`place`]): the expensive part of [`starter_documents`] (reading the data, building and
/// deduplicating the objects), so the area page builds it once and places it again for every
/// change of the content settings. It belongs to the folder and the data as stored when it was
/// built, like the documents.
#[derive(Clone)]
pub struct BuiltProject {
    /// [`project_fingerprint`] of the project file it was built from.
    fingerprint: String,
    /// The content settings of that project file.
    saved_content: ContentSettings,
    /// The objects of the map and the candidates, as the build put them (`build/assembly.rs`).
    objects: Built,
    notes: String,
    templates: Vec<TemplateRef>,
    attribution_text: String,
    content: PaperRect,
    info: PeripheralInfo,
    /// The walkable area (sprint only) and the sheet, which no content setting changes.
    base_part: Option<MapPart>,
    sheet_part: MapPart,
}

impl BuiltProject {
    /// [`project_fingerprint`] of the project file it was built from.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// How many times the paved borders were computed for it (and its clones).
    #[cfg(test)]
    pub(super) fn borders_computed(&self) -> usize {
        self.objects.borders_computed()
    }
}

/// Builds every object of the project in `dir` from its stored data ([`BuiltProject`]).
pub fn build_project(dir: &Path) -> Result<BuiltProject, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let area = project.area()?;
    let georef = Georeferencing::new_in(
        project.crs(),
        area.centroid(),
        project.scale_denominator,
        project.declination_date,
    )?;
    let symbol_set = project.map_kind.symbol_set(project.scale_denominator);
    let stored = read_stored(dir, &project, &area)?;
    // The interval of the sheet: that of the stored contours; without lines it is the one the
    // next derive takes.
    let contour_interval_m = stored.contours.as_ref().map_or_else(
        || {
            project
                .contour_interval_m
                .unwrap_or_else(|| project.map_kind.contour_interval_m())
        },
        |contours| contours.interval,
    );
    let saved_content = content_settings(&project);
    // The objects of the map and the candidates (`build/assembly.rs`).
    let objects = build(
        Frame {
            map_kind: project.map_kind,
            georef,
            symbol_set,
            area,
            area_source_id: project.map_area_source_id.clone(),
            buffer_m: project.buffer_m,
            clip: project.bbox()?,
        },
        stored,
        &saved_content,
    )?;
    let (georef, symbol_set) = (objects.georef(), objects.symbol_set());
    // A register is named once its points (or trees merged into a group area) are among the
    // candidates; OSM is named anyway.
    let registers = drawn_registers(objects.point_candidates());
    let attribution_text = attribution_lines(&project, &registers).join("\n");
    let notes = format!("{attribution_text}\n{FREEMAPPER_CREDIT}");
    let crs_spec = format!("+init=epsg:{}", georef.epsg);
    let mut templates: Vec<TemplateRef> = [
        (ORTHOPHOTO_RELPATH, true, 1.0),
        (HILLSHADE_RELPATH, false, 0.5),
        (SLOPE_RELPATH, false, 0.5),
        (NDSM_RELPATH, false, 0.6),
        (K5_RELPATH, false, 0.7),
        (RELIEF_RELPATH, false, 0.5),
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
    // The candidate layers after the rasters, hidden: on opening the map shows only its own
    // objects (ADR 0156).
    templates.extend(CandidateLayer::ALL.iter().map(|layer| TemplateRef {
        kind: TemplateKind::Map,
        relpath: layer.relpath().to_string(),
        georeferenced: true,
        crs_spec: None,
        visible: false,
        opacity: CANDIDATE_LAYER_OPACITY,
        behind_map: false,
    }));
    let content = PaperRect::of_bbox(&project.bbox()?, georef)?;
    let mut attribution = vec![OSM_ATTRIBUTION_DE.to_string()];
    attribution.extend(sheet_credit_lines(&project, &registers));
    attribution.push(FREEMAPPER_CREDIT_DE.to_string());
    let info = PeripheralInfo {
        title: project.display_title(dir),
        scale_denominator: project.scale_denominator,
        contour_interval_m,
        attribution,
    };
    // The walkable area (sprint only, ADR 0069) in its own part, written first.
    let base_part = walkable_base(&content, project.map_kind, symbol_set).map(|base| MapPart {
        name: BASE_PART.to_string(),
        objects: vec![base],
    });
    let sheet_part = MapPart {
        name: SHEET_PART.to_string(),
        objects: info_block(&content, &info, symbol_set),
    };
    let fingerprint = project_fingerprint(&project)?;

    Ok(BuiltProject {
        fingerprint,
        saved_content,
        objects,
        notes,
        templates,
        attribution_text,
        content,
        info,
        base_part,
        sheet_part,
    })
}

/// Reads the stored data of the project in `dir`: the one place the build reads the folder. A
/// source that was not fetched or does not cover the map area is left empty (`fetch` reports
/// it); the buildings and the OSM features fail instead when their recorded fetch does not cover
/// the current map area plus buffer.
fn read_stored(dir: &Path, project: &ProjectFile, area: &MapArea) -> Result<Stored, ProjectError> {
    let buildings = read_stored_buildings(dir)?;
    if buildings.is_some()
        && let Some(record) = &project.buildings
        && !record.covers(&project.bbox()?)
    {
        return Err(ProjectError::BuildingsNotFetched);
    }
    let berlin_layers = stored_berlin_layers(dir, project)?;
    let nas_layers = stored_nas_layers(dir, project)?;
    let set_layers = stored_set_layers(dir, project)?;
    // The terrain for the stair direction, read only when a layer gives stairs.
    let stair_terrain = if stair_layers_stored(&berlin_layers, &nas_layers)
        || set_layers
            .iter()
            .any(|set| set.stair_layers().next().is_some())
    {
        stored_stair_terrain(dir, project)?
    } else {
        None
    };
    let land_use = stored_land_use(dir, project)?;
    let osm = stored_osm_features(dir, project, area)?;
    // Not read for a forest map, which draws no register trees.
    let tree_layers = if project.map_kind != MapKind::Forest {
        stored_tree_layers(dir, project)?
    } else {
        Vec::new()
    };
    Ok(Stored {
        buildings: buildings.map(|stored| Buildings {
            buildings: stored.buildings,
            features: stored.features,
            fetched: project.buildings.as_ref().map(|record| record.fetched),
        }),
        berlin: berlin_layers
            .into_iter()
            .map(|l| Layer {
                name: l.layer.file.to_string(),
                features: l.features,
                fetched: l.fetched,
            })
            .collect(),
        nas: nas_layers
            .into_iter()
            .map(|l| Layer {
                name: l.layer.file.to_string(),
                features: l.features,
                fetched: l.fetched,
            })
            .collect(),
        trees: tree_layers
            .into_iter()
            .map(|l| Layer {
                name: l.dataset.id.to_string(),
                features: l.features,
                fetched: l.fetched,
            })
            .collect(),
        sets: set_layers,
        land_use,
        osm,
        stair_terrain,
        // A forest map measures its cliffs on the terrain (`build/cliffs.rs`): of a tile provider
        // the mosaic `derive` kept for it.
        // The stored DGM1 of a per-area provider, else the mosaic `derive` kept of a tile
        // provider's (ADR 0126): the ticks of banks and the cliffs follow it.
        terrain: read_terrain(dir)
            .or_else(|| read_float_raster(&dir.join(TERRAIN_MOSAIC_RELPATH)).ok()),
        // Where the orthophoto shows rock: the cliffs it confirms go first (ADR 0126).
        rock_seen: matches!(project.map_kind, MapKind::Forest | MapKind::Sprint)
            .then(|| crate::derive::terrain::read_float_raster(&dir.join(ROCK_SEEN_RELPATH)).ok())
            .flatten(),
        contours: read_stored_contours(dir, project)?,
        landforms: read_stored_landforms(dir, project)?,
        vegetation: read_stored_vegetation(dir, project)?,
    })
}

/// The documents of `built` with its objects placed by `content` (ADR 0100): `route` sends them
/// to the map, to the candidates or leaves them out, and the steps that look at both lists
/// follow. [`starter_documents`] places with the project's own settings.
pub fn place(
    built: &BuiltProject,
    content: &ContentSettings,
) -> Result<StarterDocuments, ProjectError> {
    let placed = built.objects.place(content)?;
    let (georef, symbol_set) = (built.objects.georef(), built.objects.symbol_set());
    let candidates = MapDocument {
        notes: built.notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts: vec![MapPart {
            name: "candidates".to_string(),
            objects: placed.candidates,
        }],
        templates: Vec::new(),
    };
    let mut parts: Vec<MapPart> = built.base_part.iter().cloned().collect();
    parts.push(MapPart {
        name: "default part".to_string(),
        objects: placed.map,
    });
    parts.push(built.sheet_part.clone());
    let map = MapDocument {
        notes: built.notes.clone(),
        georef: georef.clone(),
        symbol_set: symbol_set.clone(),
        parts,
        templates: built.templates.clone(),
    };

    Ok(StarterDocuments {
        map,
        candidates,
        attribution_text: built.attribution_text.clone(),
        content: built.content,
        info: built.info.clone(),
        content_settings: content.clone(),
        training: placed.training,
        light: placed.light,
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

/// The one candidates template of freemapper before the candidate layers (ADR 0156): written
/// only while a kept map names it, removed when the map is replaced.
pub(crate) const OLD_CANDIDATES_RELPATH: &str = "templates/candidates.omap";

/// The opacity of a candidate layer when it is switched on.
pub(crate) const CANDIDATE_LAYER_OPACITY: f32 = 0.7;

pub(super) fn write_document(
    dir: &Path,
    doc: &MapDocument,
    path: &Path,
) -> Result<(), ProjectError> {
    // Written whole and then renamed: a full disk leaves the previous map as it was.
    Ok(write_inside(dir, path, &document_bytes(doc)?)?)
}

/// The OOM file of `doc`.
pub(super) fn document_bytes(doc: &MapDocument) -> Result<Vec<u8>, ProjectError> {
    let mut bytes = Vec::new();
    write_map(doc, &mut bytes, XmlFormat::Compact)?;
    Ok(bytes)
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
