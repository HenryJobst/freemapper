//! What a project has and will download: download estimate, layer list and PNG previews (for the
//! MCP server and the CLI).

use std::path::Path;

use chrono::NaiveDate;

use crate::area::MapArea;
use crate::crs::{BoundingBox, LatLon, ProjectedPoint, utm33_inverse};
use crate::fetch::Cache;
use crate::project::{
    BUILDINGS_RELPATH, CANDIDATES_RELPATH, HILLSHADE_RELPATH, LAND_USE_RELPATH, NDSM_RELPATH,
    ORTHOPHOTO_RELPATH, ProjectError, ProjectFile, SLOPE_RELPATH, check_area, check_size, map_crs,
    map_path,
};
use crate::sources::lgb::Product;
use crate::sources::osm::{ExtractSource, MAX_EXTRACT_AGE_DAYS, cached_extract_date};
use crate::sources::region::{Provider, Region, provider_tiles, raster_region, regions_for};
use crate::tiles::TileId;

/// Typical download sizes in MB.
const DOP_TILE_MB: f64 = 8.0;
const DGM_TILE_MB: f64 = 1.3;
const NDSM_TILE_MB: f64 = 15.0;
/// The bDOM GeoTIFF that replaces the nDOM inside Berlin.
const BDOM_TILE_MB: f64 = 30.0;
/// GeoSN 2 km tiles and the WMS orthophoto per km² of map area plus buffer.
const GEOSN_DGM1_TILE_MB: f64 = 14.0;
const GEOSN_DOM1_TILE_MB: f64 = 15.0;
const GEOSN_WMS_MB_PER_KM2: f64 = 8.0;
/// A 1 m Float32 height model from the LVermGeo WCS per km².
const WCS_MB_PER_KM2: f64 = 4.0;
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
    // Regions and sizes are judged in EPSG:25833, where the boundaries are.
    let bbox = project.region_bbox()?;
    check_area(&bbox)?;
    let alkis_cached = dir.join(BUILDINGS_RELPATH).exists() && dir.join(LAND_USE_RELPATH).exists();
    Ok(estimate_area(&bbox, cache, today, alkis_cached))
}

/// The tiles a map of `bbox` uses, on its raster provider's grid, and the grid size in km.
fn area_tiles(bbox: &BoundingBox) -> (Vec<TileId>, u32) {
    let provider = raster_region(bbox).map_or(Provider::Lgb, Region::provider);
    match provider {
        Provider::Lgb => (provider_tiles(bbox, provider, 1), 1),
        Provider::GeoSn => {
            let km = Product::SaxonyDtm.grid_km();
            (provider_tiles(bbox, provider, km), km)
        }
        // LVermGeo rasters are fetched per area (WMS, WCS).
        // HVBG rasters are fetched per area, too (no 25833 grid: Hesse has negative eastings).
        Provider::LvermGeo | Provider::Lgv | Provider::Hvbg => (Vec::new(), 1),
    }
}

fn estimate_area(
    bbox: &BoundingBox,
    cache: &Cache,
    today: NaiveDate,
    alkis_cached: bool,
) -> DownloadEstimate {
    let (tiles, _) = area_tiles(bbox);
    let mut items = match raster_region(bbox).map(Region::provider) {
        Some(Provider::GeoSn) => geosn_items(&tiles, bbox, cache),
        Some(Provider::LvermGeo) => lvermgeo_items(bbox),
        Some(Provider::Lgv) => lgv_items(bbox),
        // Task 5: the HVBG download items.
        Some(Provider::Hvbg) => Vec::new(),
        _ => lgb_items(&tiles, cache),
    };
    let mut sources: Vec<ExtractSource> = Vec::new();
    for region in regions_for(bbox) {
        let source = ExtractSource::for_region(region);
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    for source in sources {
        items.push(EstimateItem {
            what: format!("OSM extract {} (shared by all projects)", source.label()),
            megabytes: source.megabytes(),
            cached: cached_extract_date(cache, source)
                .is_some_and(|date| (today - date).num_days() <= MAX_EXTRACT_AGE_DAYS),
        });
    }
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

/// GeoSN: DGM1 and DOM1 tiles, and the WMS orthophoto of the area (fetched for every project).
fn geosn_items(tiles: &[TileId], bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    let mut items = Vec::new();
    for &tile in tiles {
        for (product, mb) in [
            (Product::SaxonyDtm, GEOSN_DGM1_TILE_MB),
            (Product::SaxonyDsm, GEOSN_DOM1_TILE_MB),
        ] {
            items.push(EstimateItem {
                what: format!("{product} tile {}", tile.name()),
                megabytes: mb,
                cached: cache.has_tile(product, tile),
            });
        }
    }
    items.push(EstimateItem {
        what: "orthophoto (GeoSN WMS)".to_string(),
        megabytes: (bbox.width() * bbox.height() / 1e6 * GEOSN_WMS_MB_PER_KM2).max(0.1),
        cached: false,
    });
    items
}

/// LVermGeo: the WMS orthophoto and the WCS height models of the area, fetched for every project.
fn lvermgeo_items(bbox: &BoundingBox) -> Vec<EstimateItem> {
    use crate::sources::lvermgeo::Model;
    let km2 = bbox.width() * bbox.height() / 1e6;
    let item = |what: String, mb_per_km2: f64| EstimateItem {
        what,
        megabytes: (km2 * mb_per_km2).max(0.1),
        cached: false,
    };
    vec![
        item(
            "orthophoto (LVermGeo WMS)".to_string(),
            GEOSN_WMS_MB_PER_KM2,
        ),
        item(Model::Dgm1.label().to_string(), WCS_MB_PER_KM2),
        item(Model::Dom1.label().to_string(), WCS_MB_PER_KM2),
    ]
}

/// LGV: the WMS orthophoto and the height model tiles of the area, read out of the statewide
/// archives (≈2 MB DGM1 and ≈4 MB bDOM per km²).
fn lgv_items(bbox: &BoundingBox) -> Vec<EstimateItem> {
    use crate::sources::lgv::Model;
    let km2 = bbox.width() * bbox.height() / 1e6;
    let item = |what: String, mb_per_km2: f64| EstimateItem {
        what,
        megabytes: (km2 * mb_per_km2).max(0.1),
        cached: false,
    };
    vec![
        item("orthophoto (LGV WMS)".to_string(), GEOSN_WMS_MB_PER_KM2),
        item(Model::Dgm1.label().to_string(), 2.0),
        item(Model::Bdom.label().to_string(), 4.0),
    ]
}

/// LGB: orthophoto, DTM and nDSM (or bDOM inside Berlin) per 1 km tile.
fn lgb_items(tiles: &[TileId], cache: &Cache) -> Vec<EstimateItem> {
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
        // LGB publishes no nDOM inside Berlin; the bDOM stands in for it.
        let inside_berlin = regions_for(&tile.bounds()) == [Region::Berlin];
        items.push(if inside_berlin {
            EstimateItem {
                what: format!("nDSM tile {} ({})", tile.name(), Product::ImageDsm),
                megabytes: BDOM_TILE_MB,
                cached: cache.has_tile(Product::ImageDsm, tile),
            }
        } else {
            EstimateItem {
                what: format!("nDSM tile {}", tile.name()),
                megabytes: NDSM_TILE_MB,
                cached: cache.has_ndsm(tile),
            }
        });
    }
    items
}

/// A data tile (LGB 1 km, GeoSN 2 km) as drawn on a web map: corners south-west, south-east,
/// north-east, north-west.
#[derive(Debug, Clone, PartialEq)]
pub struct TileOutline {
    pub name: String,
    pub corners: [LatLon; 4],
}

/// What an area that is not saved yet would mean: its tiles, size and download.
#[derive(Debug, Clone, PartialEq)]
pub struct AreaPreview {
    /// Tiles covering the area plus buffer; empty when the area is too large or outside.
    pub tiles: Vec<TileOutline>,
    /// Size of the area plus buffer in metres.
    pub width_m: f64,
    pub height_m: f64,
    pub too_large: bool,
    /// The area lies outside the LGB data (Brandenburg and Berlin).
    pub outside_coverage: bool,
    /// Empty when the area is too large or outside the LGB data.
    pub estimate: DownloadEstimate,
}

/// The preview of `area` with `buffer_m` around it; ALKIS data counts as not downloaded yet.
pub fn preview_area(area: &MapArea, buffer_m: f64, cache: &Cache, today: NaiveDate) -> AreaPreview {
    let bbox = area.projected_bbox().buffered(buffer_m);
    // The size counts in the map's CRS, where the templates are built.
    let map_bbox = area.projected_bbox_in(map_crs(&bbox)).buffered(buffer_m);
    let check = check_area(&bbox).and_then(|()| check_size(&map_bbox));
    let outside_coverage = matches!(check, Err(ProjectError::OutsideCoverage));
    let too_large = matches!(check, Err(ProjectError::AreaTooLarge { .. }));
    let usable = check.is_ok();
    let (tiles, km) = if usable {
        area_tiles(&bbox)
    } else {
        (Vec::new(), 1)
    };
    let outlines = tiles
        .iter()
        .filter_map(|tile| {
            let b = tile.bounds_km(km);
            let corner = |easting, northing| utm33_inverse(ProjectedPoint { easting, northing });
            Some(TileOutline {
                name: tile.name(),
                corners: [
                    corner(b.min_e, b.min_n)?,
                    corner(b.max_e, b.min_n)?,
                    corner(b.max_e, b.max_n)?,
                    corner(b.min_e, b.max_n)?,
                ],
            })
        })
        .collect();
    let estimate = if !usable {
        DownloadEstimate {
            tiles: Vec::new(),
            items: Vec::new(),
            total_megabytes: 0.0,
        }
    } else {
        estimate_area(&bbox, cache, today, false)
    };
    AreaPreview {
        tiles: outlines,
        width_m: map_bbox.width(),
        height_m: map_bbox.height(),
        too_large,
        outside_coverage,
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
        let no_dsm = !project.tiles.iter().any(|r| r.product == Product::ImageDsm);
        if layer == "ndsm" && !project.tiles.is_empty() && project.ndsm_tiles.is_empty() && no_dsm {
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

    fn square(lon: f64, lat: f64, size: f64) -> crate::area::MapArea {
        crate::area::MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[[{lon},{lat}],[{},{lat}],[{},{}],[{lon},{}],[{lon},{lat}]]]}}"#,
            lon + size,
            lon + size,
            lat + size,
            lat + size
        ))
        .unwrap()
    }

    #[test]
    fn saxony_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let dresden = square(13.818, 51.043, 0.002);
        let preview = preview_area(&dresden, 20.0, &cache, today());
        let what: Vec<&str> = preview
            .estimate
            .items
            .iter()
            .map(|i| i.what.as_str())
            .collect();
        assert!(
            what.contains(&"terrain model (GeoSN DGM1) tile 33416-5654"),
            "{what:?}"
        );
        assert!(
            what.contains(&"surface model (GeoSN DOM1) tile 33416-5654"),
            "{what:?}"
        );
        assert!(what.contains(&"orthophoto (GeoSN WMS)"), "{what:?}");
        assert!(
            what.contains(&"OSM extract Saxony (shared by all projects)"),
            "{what:?}"
        );
        assert!(
            !what
                .iter()
                .any(|w| w.contains("DOP20") || w.contains("Brandenburg")),
            "{what:?}"
        );
        let dgm = preview
            .estimate
            .items
            .iter()
            .find(|i| i.what.contains("DGM1"))
            .unwrap();
        assert_eq!(dgm.megabytes, GEOSN_DGM1_TILE_MB);
        // The outline is the 2 km tile.
        let corners = &preview.tiles[0].corners;
        let sw = crate::crs::utm33_forward(corners[0]);
        let ne = crate::crs::utm33_forward(corners[2]);
        assert!(
            (ne.easting - sw.easting - 2000.0).abs() < 1.0,
            "{sw:?} {ne:?}"
        );
    }

    #[test]
    fn saxony_anhalt_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Latina, Halle.
        let halle = square(11.9716, 51.4783, 0.002);
        let preview = preview_area(&halle, 20.0, &cache, today());
        assert!(!preview.outside_coverage);
        let what: Vec<&str> = preview
            .estimate
            .items
            .iter()
            .map(|i| i.what.as_str())
            .collect();
        assert!(what.contains(&"orthophoto (LVermGeo WMS)"), "{what:?}");
        assert!(what.contains(&"terrain model (LVermGeo DGM1)"), "{what:?}");
        assert!(what.contains(&"surface model (LVermGeo DOM1)"), "{what:?}");
        assert!(
            what.contains(&"OSM extract Saxony-Anhalt (shared by all projects)"),
            "{what:?}"
        );
        assert!(preview.tiles.is_empty(), "no data tiles in Saxony-Anhalt");
    }

    #[test]
    fn preview_checks_the_size_in_the_map_crs() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // 3950 m along the zone-33 axes at Halle: with its buffer it fits in 25833, not in 25832.
        let corner = |de: f64, dn: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: 289_795.0 + de,
                northing: 5_707_067.0 + dn,
            })
            .unwrap();
            format!("[{},{}]", p.lon, p.lat)
        };
        let h = 1975.0;
        let ring = [
            corner(-h, -h),
            corner(h, -h),
            corner(h, h),
            corner(-h, h),
            corner(-h, -h),
        ];
        let area = crate::area::MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            ring.join(",")
        ))
        .unwrap();
        assert!(area.projected_bbox().buffered(20.0).width() <= crate::project::MAX_AREA_SIDE_M);
        let preview = preview_area(&area, 20.0, &cache, today());
        assert!(
            preview.too_large,
            "{} × {}",
            preview.width_m, preview.height_m
        );
        let max = crate::project::MAX_AREA_SIDE_M;
        assert!(preview.width_m > max || preview.height_m > max);
    }

    #[test]
    fn hamburg_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let eppendorf = square(9.9820, 53.5880, 0.002);
        let preview = preview_area(&eppendorf, 20.0, &cache, today());
        let what: Vec<&str> = preview
            .estimate
            .items
            .iter()
            .map(|i| i.what.as_str())
            .collect();
        assert!(what.contains(&"orthophoto (LGV WMS)"), "{what:?}");
        assert!(what.contains(&"terrain model (LGV DGM1 2022)"), "{what:?}");
        assert!(what.contains(&"surface model (LGV bDOM 2022)"), "{what:?}");
        assert!(
            what.contains(&"OSM extract Hamburg (shared by all projects)"),
            "{what:?}"
        );
    }

    #[test]
    fn border_estimate_counts_both_extracts() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // At the Brandenburg/Saxony boundary vertex E 459 509 N 5 710 017.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 459_400.0,
            northing: 5_709_900.0,
        })
        .unwrap();
        let preview = preview_area(&square(p.lon, p.lat, 0.003), 20.0, &cache, today());
        let osm: Vec<&str> = preview
            .estimate
            .items
            .iter()
            .map(|i| i.what.as_str())
            .filter(|w| w.starts_with("OSM extract"))
            .collect();
        assert_eq!(osm.len(), 2, "{osm:?}");
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
    fn preview_flags_outside_coverage() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        for (lon, lat) in [(11.57, 48.13), (151.2, -33.87), (-170.0, 52.5), (3.0, 52.5)] {
            let text = format!(
                r#"{{"type":"Polygon","coordinates":[[[{lon},{lat}],[{},{lat}],[{},{}],[{lon},{}],[{lon},{lat}]]]}}"#,
                lon + 0.005,
                lon + 0.005,
                lat + 0.005,
                lat + 0.005
            );
            let area = crate::area::MapArea::from_geojson(&text).unwrap();
            let preview = preview_area(&area, 20.0, &cache, today());
            assert!(preview.outside_coverage, "{lon} {lat}");
            assert!(preview.tiles.is_empty() && preview.estimate.items.is_empty());
        }
        let school = fixture_area("clara-zetkin-schule.geojson");
        assert!(!preview_area(&school, 20.0, &cache, today()).outside_coverage);
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
        assert_eq!(
            estimate.total_megabytes,
            ExtractSource::Brandenburg.megabytes()
        );
    }

    #[test]
    fn berlin_estimate_counts_the_bdom() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let mitte = TileId::parse("33392-5820").unwrap();
        let strausberg = TileId::parse("33421-5821").unwrap();
        let items = lgb_items(&[mitte, strausberg], &cache);
        let what: Vec<&str> = items.iter().map(|i| i.what.as_str()).collect();
        assert!(
            what.contains(&"nDSM tile 33392-5820 (surface model (bDOM))"),
            "{what:?}"
        );
        assert!(what.contains(&"nDSM tile 33421-5821"), "{what:?}");
        let bdom = items.iter().find(|i| i.what.contains("bDOM")).unwrap();
        assert_eq!(bdom.megabytes, BDOM_TILE_MB);
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
