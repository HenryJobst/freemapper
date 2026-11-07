//! What a project has and will download: download estimate, layer list and PNG previews (for the
//! MCP server and the CLI).

use std::path::Path;

use chrono::NaiveDate;

use crate::area::MapArea;
use crate::crs::{LatLon, ProjectedPoint, utm33_inverse};
use crate::fetch::Cache;
use crate::project::{
    BUILDINGS_RELPATH, CANDIDATES_RELPATH, HILLSHADE_RELPATH, LAND_USE_RELPATH, NDSM_RELPATH,
    ORTHOPHOTO_RELPATH, ProjectError, ProjectFile, SLOPE_RELPATH, check_area_size, map_path,
};
use crate::sources::lgb::Product;
use crate::sources::osm::{MAX_EXTRACT_AGE_DAYS, cached_extract_date};
use crate::tiles::{TileId, tiles_covering};

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

/// `today` decides whether the cached OSM extract is old enough to be downloaded again.
pub fn estimate_download(
    dir: &Path,
    cache: &Cache,
    today: NaiveDate,
) -> Result<DownloadEstimate, ProjectError> {
    let project = ProjectFile::load(dir)?;
    let bbox = project.bbox()?;
    check_area_size(&bbox)?;
    let alkis_cached = dir.join(BUILDINGS_RELPATH).exists() && dir.join(LAND_USE_RELPATH).exists();
    Ok(estimate_tiles(
        &tiles_covering(&bbox),
        cache,
        today,
        alkis_cached,
    ))
}

fn estimate_tiles(
    tiles: &[TileId],
    cache: &Cache,
    today: NaiveDate,
    alkis_cached: bool,
) -> DownloadEstimate {
    let mut items = Vec::new();
    for &tile in tiles {
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
        cached: cached_extract_date(cache)
            .is_some_and(|date| (today - date).num_days() <= MAX_EXTRACT_AGE_DAYS),
    });
    items.push(EstimateItem {
        what: "ALKIS buildings and land use".to_string(),
        megabytes: ALKIS_MB,
        cached: alkis_cached,
    });
    let total_megabytes = items
        .iter()
        .filter(|i| !i.cached)
        .map(|i| i.megabytes)
        .sum();
    DownloadEstimate {
        tiles: tiles.iter().map(|t| t.name()).collect(),
        items,
        total_megabytes,
    }
}

/// An LGB tile as drawn on a web map: corners south-west, south-east, north-east, north-west.
#[derive(Debug, Clone, PartialEq)]
pub struct TileOutline {
    pub name: String,
    pub corners: [LatLon; 4],
}

/// What an area that is not saved yet would mean: its tiles, size and download.
#[derive(Debug, Clone, PartialEq)]
pub struct AreaPreview {
    /// Tiles covering the area plus buffer; empty when the area is too large.
    pub tiles: Vec<TileOutline>,
    /// Size of the area plus buffer in metres.
    pub width_m: f64,
    pub height_m: f64,
    pub too_large: bool,
    /// Empty when the area is too large.
    pub estimate: DownloadEstimate,
}

/// The preview of `area` with `buffer_m` around it; ALKIS data counts as not downloaded yet.
pub fn preview_area(area: &MapArea, buffer_m: f64, cache: &Cache, today: NaiveDate) -> AreaPreview {
    let bbox = area.projected_bbox().buffered(buffer_m);
    let too_large = check_area_size(&bbox).is_err();
    let tiles = if too_large {
        Vec::new()
    } else {
        tiles_covering(&bbox)
    };
    let outlines = tiles
        .iter()
        .map(|tile| {
            let b = tile.bounds();
            let corner = |easting, northing| utm33_inverse(ProjectedPoint { easting, northing });
            TileOutline {
                name: tile.name(),
                corners: [
                    corner(b.min_e, b.min_n),
                    corner(b.max_e, b.min_n),
                    corner(b.max_e, b.max_n),
                    corner(b.min_e, b.max_n),
                ],
            }
        })
        .collect();
    let estimate = if too_large {
        DownloadEstimate {
            tiles: Vec::new(),
            items: Vec::new(),
            total_megabytes: 0.0,
        }
    } else {
        estimate_tiles(&tiles, cache, today, false)
    };
    AreaPreview {
        tiles: outlines,
        width_m: bbox.width(),
        height_m: bbox.height(),
        too_large,
        estimate,
    }
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

/// The steps that make the layers, for CLI and MCP users.
const EXPORT_STEP: &str = "freemapper export or run (MCP: start_pipeline)";
const DERIVE_STEP: &str = "freemapper derive or run (MCP: start_pipeline)";

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
            made_by: EXPORT_STEP,
            description: "the OOM map: buildings and peripheral information",
        },
        Layer {
            name: "candidates",
            file: dir.join(CANDIDATES_RELPATH).display().to_string(),
            role: LayerRole::Candidates,
            ready: dir.join(CANDIDATES_RELPATH).exists(),
            made_by: EXPORT_STEP,
            description: "candidates to check: OSM features, contours, out-of-bounds areas",
        },
    ];
    for (name, relpath, description) in RASTER_LAYERS {
        layers.push(Layer {
            name,
            file: dir.join(relpath).display().to_string(),
            role: LayerRole::Template,
            ready: dir.join(relpath).exists(),
            made_by: DERIVE_STEP,
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
    let project = ProjectFile::load(dir)?;
    let path = dir.join(relpath);
    if !path.exists() {
        if layer == "ndsm" && !project.tiles.is_empty() && project.ndsm_tiles.is_empty() {
            return Err(ProjectError::Layer(
                "no nDSM is published for the tiles of this map area, so there is no ndsm template"
                    .to_string(),
            ));
        }
        return Err(ProjectError::Layer(format!(
            "{layer} is not made yet; run {DERIVE_STEP}"
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
    use chrono::NaiveDate;

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

    fn today() -> NaiveDate {
        "2026-11-07".parse().unwrap()
    }

    fn fixture_area(name: &str) -> crate::area::MapArea {
        let path = format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        crate::area::MapArea::from_geojson(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn preview_lists_tile_outlines() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let area = fixture_area("clara-zetkin-schule.geojson");
        let preview = preview_area(&area, 20.0, &cache, today());
        assert!(!preview.too_large);
        assert_eq!(preview.estimate.tiles, ["33421-5821"]);
        assert_eq!(preview.tiles.len(), 1);
        assert_eq!(preview.tiles[0].name, "33421-5821");
        let sw = crate::crs::utm33_forward(preview.tiles[0].corners[0]);
        assert!(
            (sw.easting - 421_000.0).abs() < 0.01 && (sw.northing - 5_821_000.0).abs() < 0.01,
            "{sw:?}"
        );
        let ne = crate::crs::utm33_forward(preview.tiles[0].corners[2]);
        assert!((ne.easting - 422_000.0).abs() < 0.01, "{ne:?}");
        assert!(
            preview.width_m > 100.0 && preview.width_m < 400.0,
            "{}",
            preview.width_m
        );
        assert!(preview.estimate.total_megabytes > 300.0);
    }

    #[test]
    fn preview_flags_too_large() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let area = crate::area::MapArea::from_geojson(
            r#"{"type":"Polygon","coordinates":[[[13.7,52.5],[13.9,52.5],[13.9,52.6],[13.7,52.6],[13.7,52.5]]]}"#,
        )
        .unwrap();
        let preview = preview_area(&area, 50.0, &cache, today());
        assert!(preview.too_large);
        assert!(preview.tiles.is_empty() && preview.estimate.items.is_empty());
        assert!(preview.width_m > 10_000.0);
    }

    #[test]
    fn estimate_counts_an_old_extract_as_download() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        fetched(dir.path(), &cache);
        let later = "2026-11-20".parse().unwrap();
        let estimate = estimate_download(dir.path(), &cache, later).unwrap();
        let osm = estimate
            .items
            .iter()
            .find(|i| i.what.contains("OSM"))
            .unwrap();
        assert!(!osm.cached, "{osm:?}");
        assert_eq!(estimate.total_megabytes, OSM_EXTRACT_MB);
    }

    #[test]
    fn estimate_refuses_an_area_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let mut project = ProjectFile::load(dir.path()).unwrap();
        project.buffer_m = 10_000.0;
        project.save(dir.path()).unwrap();
        let err = estimate_download(dir.path(), &cache, today()).unwrap_err();
        assert!(matches!(err, ProjectError::AreaTooLarge { .. }), "{err:?}");
    }

    #[test]
    fn layers_name_the_pipeline_step() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        for layer in list_layers(dir.path()) {
            assert!(layer.made_by.contains("start_pipeline"), "{layer:?}");
        }
        let err = preview_png(dir.path(), "orthophoto", 800).unwrap_err();
        assert!(err.to_string().contains("start_pipeline"), "{err}");
    }

    #[test]
    fn preview_of_a_folder_without_project_fails() {
        let dir = tempfile::tempdir().unwrap();
        let err = preview_png(dir.path(), "orthophoto", 800).unwrap_err();
        assert!(!matches!(err, ProjectError::Layer(_)), "{err:?}");
    }

    #[test]
    fn preview_explains_a_missing_ndsm() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let fetcher = MapFetcher::default();
        serve_synthetic_tile(&fetcher, TileId::parse("33421-5821").unwrap(), "2026-01-26");
        fetch_project(dir.path(), &fetcher, &cache, FetchMode::Online, &mut |_| {}).unwrap();
        derive_project(dir.path(), &cache, &mut |_| {}).unwrap();
        let err = preview_png(dir.path(), "ndsm", 800).unwrap_err();
        assert!(err.to_string().contains("no nDSM"), "{err}");
    }

    #[test]
    fn estimate_before_and_after_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        school_project(dir.path());
        let before = estimate_download(dir.path(), &cache, today()).unwrap();
        assert_eq!(before.tiles, ["33421-5821"]);
        assert!(before.items.iter().all(|i| !i.cached));
        assert!(
            before.total_megabytes > 300.0,
            "OSM extract included: {}",
            before.total_megabytes
        );
        fetched(dir.path(), &cache);
        let after = estimate_download(dir.path(), &cache, today()).unwrap();
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
