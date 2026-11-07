//! Project folder and project file (`freemapper.toml`).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::area::{AreaError, MapArea};
use crate::crs::{LatLon, utm33_forward};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{
    MapDocument, MapObject, MapObjectKind, MapPart, OmapError, SymbolSet, TemplateKind,
    TemplateRef, XmlFormat, closed_ring, write_map,
};

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

    pub fn symbol_set(self) -> SymbolSet {
        match self {
            Self::Schoolyard => SymbolSet::school_2019(),
            Self::Sprint => SymbolSet::issprom_2019(),
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
}

impl ProjectFile {
    pub fn load(dir: &Path) -> Result<Self, ProjectError> {
        let path = dir.join(PROJECT_FILE_NAME);
        let text = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io(path, e))?;
        toml::from_str(&text).map_err(|e| ProjectError::Toml(e.to_string()))
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
    };
    project.save(dir)?;
    Ok(project)
}

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
    let notes = format!("{OSM_ATTRIBUTION}\nfreemapper starter project");

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
    let map = MapDocument {
        notes,
        georef,
        symbol_set,
        parts: Vec::new(),
        templates: vec![TemplateRef {
            kind: TemplateKind::Map,
            relpath: CANDIDATES_RELPATH.to_string(),
            georeferenced: true,
            crs_spec: None,
            visible: true,
            opacity: 0.7,
        }],
    };

    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("map");
    let map_path = dir.join(format!("{name}.omap"));
    if map_path.exists() {
        if !force {
            return Err(ProjectError::MapExists(map_path));
        }
        let backup = dir.join(format!("{name}.omap.bak"));
        std::fs::copy(&map_path, &backup).map_err(|e| ProjectError::Io(backup, e))?;
    }
    write_document(&candidates, &dir.join(CANDIDATES_RELPATH))?;
    write_document(&map, &map_path)?;
    let attribution = dir.join("ATTRIBUTION.txt");
    std::fs::write(&attribution, format!("{OSM_ATTRIBUTION}\n"))
        .map_err(|e| ProjectError::Io(attribution, e))?;
    Ok(map_path)
}

const CANDIDATES_RELPATH: &str = "templates/candidates.omap";
const OSM_ATTRIBUTION: &str = "Map area: © OpenStreetMap contributors, ODbL";
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
    fn map_kind_defaults() {
        assert_eq!(MapKind::Schoolyard.default_scale(), 1000);
        assert_eq!(MapKind::Sprint.default_scale(), 4000);
        assert_eq!(MapKind::Schoolyard.default_buffer_m(), 20.0);
        assert_eq!(MapKind::Sprint.default_buffer_m(), 50.0);
        assert_eq!(MapKind::Schoolyard.symbol_set().symbol_count(), 191);
        assert_eq!(MapKind::Sprint.symbol_set().symbol_count(), 185);
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
        let project = init_project(dir.path(), &area, MapKind::Schoolyard, None, date).unwrap();
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
