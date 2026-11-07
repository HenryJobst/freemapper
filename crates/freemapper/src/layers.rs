//! What a project has and will download: download estimate, layer list and PNG previews (for the
//! MCP server and the CLI).

use std::path::Path;

use crate::fetch::Cache;
use crate::project::{
    BUILDINGS_RELPATH, CANDIDATES_RELPATH, HILLSHADE_RELPATH, LAND_USE_RELPATH, NDSM_RELPATH,
    ORTHOPHOTO_RELPATH, ProjectError, ProjectFile, SLOPE_RELPATH, map_path,
};
use crate::sources::lgb::Product;
use crate::sources::osm::has_cached_extract;
use crate::tiles::tiles_covering;

/// Typical download sizes in MB.
const DOP_TILE_MB: f64 = 8.0;
const DGM_TILE_MB: f64 = 1.3;
const NDSM_TILE_MB: f64 = 15.0;
const OSM_EXTRACT_MB: f64 = 300.0;
const ALKIS_MB: f64 = 0.5;

/// One download of a project.
#[derive(Debug, Clone, PartialEq)]
pub struct EstimateItem {
    pub what: String,
    pub megabytes: f64,
    pub cached: bool,
}

/// What `fetch` will download for a project.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadEstimate {
    pub tiles: Vec<String>,
    pub items: Vec<EstimateItem>,
    /// Megabytes still to download (items not cached).
    pub total_megabytes: f64,
}

pub fn estimate_download(dir: &Path, cache: &Cache) -> Result<DownloadEstimate, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let tiles = tiles_covering(&project.bbox()?);
    let mut items = Vec::new();
    for &tile in &tiles {
        for (product, mb) in [
            (Product::Orthophoto, DOP_TILE_MB),
            (Product::Dtm, DGM_TILE_MB),
        ] {
            items.push(EstimateItem {
                what: format!("{product} tile {}", tile.name()),
                megabytes: mb,
                cached: cache.has_tile(product, tile),
            });
        }
        items.push(EstimateItem {
            what: format!("nDSM tile {}", tile.name()),
            megabytes: NDSM_TILE_MB,
            cached: cache.has_ndsm(tile),
        });
    }
    items.push(EstimateItem {
        what: "OSM extract Brandenburg (shared by all projects)".to_string(),
        megabytes: OSM_EXTRACT_MB,
        cached: has_cached_extract(cache),
    });
    items.push(EstimateItem {
        what: "ALKIS buildings and land use".to_string(),
        megabytes: ALKIS_MB,
        cached: dir.join(BUILDINGS_RELPATH).exists() && dir.join(LAND_USE_RELPATH).exists(),
    });
    let total_megabytes = items
        .iter()
        .filter(|i| !i.cached)
        .map(|i| i.megabytes)
        .sum();
    Ok(DownloadEstimate {
        tiles: tiles.iter().map(|t| t.name()).collect(),
        items,
        total_megabytes,
    })
}

/// What a layer is for the mapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerRole {
    /// The OOM map itself.
    Map,
    /// Candidate objects to check and copy (template map).
    Candidates,
    /// A raster template behind the map.
    Template,
}

/// One layer of a project.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: &'static str,
    pub file: String,
    pub role: LayerRole,
    pub ready: bool,
    /// The command that makes the file.
    pub made_by: &'static str,
    pub description: &'static str,
}

/// Raster templates that have previews.
const RASTER_LAYERS: [(&str, &str, &str); 4] = [
    (
        "orthophoto",
        ORTHOPHOTO_RELPATH,
        "aerial photo (DOP20), the base for drawing",
    ),
    (
        "hillshade",
        HILLSHADE_RELPATH,
        "shaded relief of the terrain model",
    ),
    (
        "slope",
        SLOPE_RELPATH,
        "slope of the terrain: banks and ramps",
    ),
    (
        "ndsm",
        NDSM_RELPATH,
        "height above ground: hedges, walls, trees, playground equipment",
    ),
];

pub fn list_layers(dir: &Path) -> Vec<Layer> {
    let map = map_path(dir);
    let mut layers = vec![
        Layer {
            name: "map",
            file: map.display().to_string(),
            role: LayerRole::Map,
            ready: map.exists(),
            made_by: "freemapper export",
            description: "the OOM map: buildings and peripheral information",
        },
        Layer {
            name: "candidates",
            file: dir.join(CANDIDATES_RELPATH).display().to_string(),
            role: LayerRole::Candidates,
            ready: dir.join(CANDIDATES_RELPATH).exists(),
            made_by: "freemapper export",
            description: "candidates to check: OSM features, contours, out-of-bounds areas",
        },
    ];
    for (name, relpath, description) in RASTER_LAYERS {
        layers.push(Layer {
            name,
            file: dir.join(relpath).display().to_string(),
            role: LayerRole::Template,
            ready: dir.join(relpath).exists(),
            made_by: "freemapper derive",
            description,
        });
    }
    layers
}

/// A PNG of a raster template, its longest side at most `max_px`.
pub fn preview_png(dir: &Path, layer: &str, max_px: u32) -> Result<Vec<u8>, ProjectError> {
    let Some((_, relpath, _)) = RASTER_LAYERS.iter().find(|(name, _, _)| *name == layer) else {
        let names: Vec<&str> = RASTER_LAYERS.iter().map(|(name, _, _)| *name).collect();
        return Err(ProjectError::Layer(format!(
            "no preview for {layer:?}; layers with previews: {}",
            names.join(", ")
        )));
    };
    let path = dir.join(relpath);
    if !path.exists() {
        return Err(ProjectError::Layer(format!(
            "{layer} is not made yet; run `freemapper derive` (or start the pipeline)"
        )));
    }
    let image =
        image::open(&path).map_err(|e| ProjectError::Layer(format!("{}: {e}", path.display())))?;
    let small = image.thumbnail(max_px, max_px);
    let mut png = std::io::Cursor::new(Vec::new());
    small
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| ProjectError::Layer(e.to_string()))?;
    Ok(png.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, serve_synthetic_ndsm, serve_synthetic_tile};
    use crate::fetch::{Cache, FetchMode};
    use crate::project::{MapKind, derive_project, fetch_project, init_project};
    use crate::tiles::TileId;

    fn school_project(dir: &Path) {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/clara-zetkin-schule.geojson"
        ))
        .unwrap();
        let area = crate::area::MapArea::from_geojson(&text).unwrap();
        let date = "2026-11-07".parse().unwrap();
        init_project(dir, &area, MapKind::Schoolyard, None, date, None).unwrap();
    }

    fn fetched(dir: &Path, cache: &Cache) {
        let fetcher = MapFetcher::default();
        let tile = TileId::parse("33421-5821").unwrap();
        serve_synthetic_tile(&fetcher, tile, "2026-01-26");
        serve_synthetic_ndsm(&fetcher, tile);
        fetch_project(dir, &fetcher, cache, FetchMode::Online, &mut |_| {}).unwrap();
    }

    #[test]
    fn estimate_before_and_after_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let before = estimate_download(dir.path(), &cache).unwrap();
        assert_eq!(before.tiles, ["33421-5821"]);
        assert!(before.items.iter().all(|i| !i.cached));
        assert!(
            before.total_megabytes > 300.0,
            "OSM extract included: {}",
            before.total_megabytes
        );
        fetched(dir.path(), &cache);
        let after = estimate_download(dir.path(), &cache).unwrap();
        assert!(after.items.iter().all(|i| i.cached), "{:?}", after.items);
        assert_eq!(after.total_megabytes, 0.0);
    }

    #[test]
    fn layers_report_what_is_ready() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let names: Vec<&str> = list_layers(dir.path()).iter().map(|l| l.name).collect();
        assert_eq!(
            names,
            [
                "map",
                "candidates",
                "orthophoto",
                "hillshade",
                "slope",
                "ndsm"
            ]
        );
        assert!(list_layers(dir.path()).iter().all(|l| !l.ready));
        fetched(dir.path(), &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let ready: Vec<&str> = list_layers(dir.path())
            .into_iter()
            .filter(|l| l.ready)
            .map(|l| l.name)
            .collect();
        assert_eq!(ready, ["orthophoto", "hillshade", "slope", "ndsm"]);
    }

    #[test]
    fn preview_is_a_small_png() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let err = preview_png(dir.path(), "orthophoto", 800).unwrap_err();
        assert!(err.to_string().contains("freemapper derive"), "{err}");
        assert!(
            preview_png(dir.path(), "street view", 800)
                .unwrap_err()
                .to_string()
                .contains("orthophoto")
        );
        fetched(dir.path(), &cache);
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let png = preview_png(dir.path(), "ndsm", 64).unwrap();
        let image = image::load_from_memory(&png).unwrap();
        assert!(
            image.width() <= 64 && image.height() <= 64,
            "{}×{}",
            image.width(),
            image.height()
        );
        assert_eq!(&png[1..4], b"PNG");
    }
}
