//! What a project has and will download: download estimate, layer list and PNG previews (for the
//! MCP server and the CLI).

use std::path::Path;

use chrono::NaiveDate;

use crate::area::MapArea;
use crate::crs::{BoundingBox, LatLon, ProjectedPoint, utm33_inverse};
use crate::fetch::Cache;
use crate::project::{
    BUILDINGS_RELPATH, CANDIDATES_RELPATH, HILLSHADE_RELPATH, K5_RELPATH, LAND_USE_RELPATH,
    NDSM_RELPATH, ORTHOPHOTO_RELPATH, ProjectError, ProjectFile, SLOPE_RELPATH, check_area,
    check_size, map_crs, map_path,
};
use crate::sources::lgb::Product;
use crate::sources::osm::{MAX_EXTRACT_AGE_DAYS, cached_extract_date, extracts_for};
use crate::sources::region::{
    Provider, Region, provider_tiles, raster_region, regions_for, tile_touches,
};
use crate::space::{Part, Store};
use crate::tiles::{TileId, tiles_covering};

/// Typical download sizes in MB.
const DOP_TILE_MB: f64 = 8.0;
const DGM_TILE_MB: f64 = 1.3;
const NDSM_TILE_MB: f64 = 15.0;
/// The bDOM GeoTIFF that replaces the nDOM inside Berlin.
const BDOM_TILE_MB: f64 = 30.0;
/// The WMS orthophoto per km² of map area plus buffer (all states), and GeoSN's 2 km tiles.
const GEOSN_DGM1_TILE_MB: f64 = 14.0;
const GEOSN_DOM1_TILE_MB: f64 = 15.0;
const ORTHOPHOTO_MB_PER_KM2: f64 = 8.0;
/// A 1 m Float32 height model from a WCS (LVermGeo, LAiV M-V, LGLN) per km².
const WCS_MB_PER_KM2: f64 = 4.0;
const ALKIS_MB: f64 = 0.5;
/// The Berlin ALKIS structure layers, relief, hedges and the tree cadastre, per km² (measured
/// 3.7 MB in Kreuzberg and 4.6 MB in Prenzlauer Berg, 2026-11-09).
const BERLIN_LAYERS_MB_PER_KM2: f64 = 4.0;
/// The K5 map (1:5000) PNG template at 0.25 m per pixel, per km² (measured 9.1 MB around a park
/// and 17 MB in dense blocks, 2026-11-09).
const K5_MB_PER_KM2: f64 = 15.0;
/// Names of the Berlin items in the estimate.
pub(crate) const BERLIN_LAYERS_ITEM: &str = "Berlin ALKIS structures and tree cadastre";
pub(crate) const BERLIN_K5_ITEM: &str = "Berlin K5 map (1:5000) template";
/// LVermGeo SH per whole 1 km tile: the DGM1 GeoTIFF and the bDOM 0.2 m (reduced to 1 m after).
const SH_DGM1_TILE_MB: f64 = 1.5;
const SH_BDOM_TILE_MB: f64 = 105.0;
/// A Geobasis NRW DGM1 tile (Float32, LZW) and a DOM1 tile.
const NRW_DGM1_TILE_MB: f64 = 2.2;
const NRW_DOM1_TILE_MB: f64 = 3.0;
/// An LVermGeo RP DGM1 tile (Float32, LZW, 1.3-1.8 MB) and a DOM1 tile (2-3.5 MB).
const RP_DGM1_TILE_MB: f64 = 1.5;
const RP_DOM1_TILE_MB: f64 = 3.0;

/// One download of a project.
#[derive(Debug, Clone, PartialEq)]
pub struct EstimateItem {
    pub what: String,
    pub megabytes: f64,
    pub cached: bool,
    /// The part of the fetch it belongs to, kept or left out as a whole when space runs short.
    pub part: Part,
    /// Where the download is stored.
    pub store: Store,
}

/// What `fetch` will download for a project.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadEstimate {
    pub tiles: Vec<String>,
    pub items: Vec<EstimateItem>,
    /// Megabytes still to download (items not cached).
    pub total_megabytes: f64,
    /// Parts whose files in the project folder (stored data, templates) already cover the area:
    /// a fetch replaces them in place, so the files derived from them need no new room.
    pub in_place: Vec<Part>,
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
    let map_bbox = project.bbox()?;
    let mut estimate = estimate_area(&bbox, &map_bbox, cache, today, alkis_cached);
    crate::project::mark_stored(&mut estimate, dir, &project, &map_bbox);
    Ok(estimate)
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
        Provider::LvermGeo
        | Provider::Lgv
        | Provider::Hvbg
        | Provider::LvermGeoSh
        | Provider::LaivMv
        | Provider::GeobasisNrw
        | Provider::Lgln
        | Provider::LvermGeoRp => (Vec::new(), 1),
    }
}

/// `bbox` is the map area plus buffer in EPSG:25833, `map_bbox` the same in the map's CRS.
pub(crate) fn estimate_area(
    bbox: &BoundingBox,
    map_bbox: &BoundingBox,
    cache: &Cache,
    today: NaiveDate,
    alkis_cached: bool,
) -> DownloadEstimate {
    let (tiles, _) = area_tiles(bbox);
    let mut items = match raster_region(bbox).map(Region::provider) {
        Some(Provider::GeoSn) => geosn_items(&tiles, bbox, cache),
        Some(Provider::LvermGeo) => lvermgeo_items(bbox),
        Some(Provider::Lgv) => lgv_items(bbox, cache),
        Some(Provider::Hvbg) => hvbg_items(bbox, cache),
        Some(Provider::LvermGeoSh) => schleswig_holstein_items(bbox, cache),
        Some(Provider::LaivMv) => laiv_mv_items(bbox),
        Some(Provider::GeobasisNrw) => nrw_items(bbox, cache),
        Some(Provider::Lgln) => lgln_items(bbox),
        Some(Provider::LvermGeoRp) => rp_items(bbox, map_bbox, cache),
        Some(Provider::Lgb) | None => lgb_items(&tiles, cache),
    };
    // The estimate stays quiet; the fetch prints any extract warning.
    for source in extracts_for(bbox, &mut |_| {}) {
        let date = cached_extract_date(cache, source);
        let cached = date.is_some_and(|date| (today - date).num_days() <= MAX_EXTRACT_AGE_DAYS);
        items.push(EstimateItem {
            what: format!("OSM extract {} (shared by all projects)", source.label()),
            megabytes: source.megabytes(),
            cached,
            // An older cached extract serves when the newer one does not fit.
            part: if date.is_some() && !cached {
                Part::OsmUpdate
            } else {
                Part::MapData
            },
            store: Store::Cache,
        });
    }
    items.push(EstimateItem {
        what: "ALKIS buildings and land use".to_string(),
        megabytes: ALKIS_MB,
        cached: alkis_cached,
        part: Part::MapData,
        store: Store::Project,
    });
    // Areas that touch Berlin also get the extra Berlin layers and the K5 template.
    if regions_for(bbox).contains(&Region::Berlin) {
        items.push(per_km2_item(
            bbox,
            BERLIN_LAYERS_ITEM,
            BERLIN_LAYERS_MB_PER_KM2,
            Part::BerlinLayers,
        ));
        items.push(per_km2_item(bbox, BERLIN_K5_ITEM, K5_MB_PER_KM2, Part::K5));
    }
    let total_megabytes = items
        .iter()
        .filter(|i| !i.cached)
        .map(|i| i.megabytes)
        .sum();
    DownloadEstimate {
        tiles: tiles.iter().map(|t| t.name()).collect(),
        items,
        total_megabytes,
        in_place: Vec::new(),
    }
}

/// GeoSN: DGM1 and DOM1 tiles, and the WMS orthophoto of the area (fetched for every project).
fn geosn_items(tiles: &[TileId], bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    let mut items = Vec::new();
    for &tile in tiles {
        for (product, mb, part) in [
            (Product::SaxonyDtm, GEOSN_DGM1_TILE_MB, Part::Terrain),
            (Product::SaxonyDsm, GEOSN_DOM1_TILE_MB, Part::Surface),
        ] {
            items.push(EstimateItem {
                what: format!("{product} tile {}", tile.name()),
                megabytes: mb,
                cached: cache.has_tile(product, tile),
                part,
                store: Store::Cache,
            });
        }
    }
    items.push(per_km2_item(
        bbox,
        "orthophoto (GeoSN WMS)",
        ORTHOPHOTO_MB_PER_KM2,
        Part::Orthophoto,
    ));
    items
}

/// LVermGeo: the WMS orthophoto and the WCS height models of the area, fetched for every project.
fn lvermgeo_items(bbox: &BoundingBox) -> Vec<EstimateItem> {
    use crate::sources::lvermgeo::Model;
    let km2 = bbox.width() * bbox.height() / 1e6;
    let item = |what: String, mb_per_km2: f64, part: Part| EstimateItem {
        what,
        megabytes: (km2 * mb_per_km2).max(0.1),
        cached: false,
        part,
        store: Store::Project,
    };
    vec![
        item(
            "orthophoto (LVermGeo WMS)".to_string(),
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        item(
            Model::Dgm1.label().to_string(),
            WCS_MB_PER_KM2,
            Part::Terrain,
        ),
        item(
            Model::Dom1.label().to_string(),
            WCS_MB_PER_KM2,
            Part::Surface,
        ),
    ]
}

/// The 1 km tiles of EPSG:25832 touched by `bbox` (EPSG:25833): what the per-area providers of
/// zone 32 (LGV, HVBG) read, whole, out of their archives.
fn zone_32_tiles(bbox: &BoundingBox) -> Vec<TileId> {
    use crate::crs::{Crs, ProjectedPoint};
    let corners = [
        (bbox.min_e, bbox.min_n),
        (bbox.max_e, bbox.min_n),
        (bbox.max_e, bbox.max_n),
        (bbox.min_e, bbox.max_n),
    ]
    .map(|(easting, northing)| {
        Crs::Utm32.reproject(Crs::Utm33, ProjectedPoint { easting, northing })
    });
    let Some(corners) = corners.into_iter().collect::<Option<Vec<_>>>() else {
        return Vec::new();
    };
    let fold = |pick: fn(&ProjectedPoint) -> f64, init: f64, f: fn(f64, f64) -> f64| {
        corners.iter().map(pick).fold(init, f)
    };
    tiles_covering(&BoundingBox {
        min_e: fold(|p| p.easting, f64::INFINITY, f64::min),
        min_n: fold(|p| p.northing, f64::INFINITY, f64::min),
        max_e: fold(|p| p.easting, f64::NEG_INFINITY, f64::max),
        max_n: fold(|p| p.northing, f64::NEG_INFINITY, f64::max),
    })
}

/// LGV: the WMS orthophoto and the height model tiles of the area, read out of the statewide
/// archives (≈2 MB DGM1 and ≈4 MB bDOM per 1 km tile touched, those already cached left out).
fn lgv_items(bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    use crate::sources::lgv::Model;
    // Only tiles that can exist: every square that touches Hamburg has a (clipped) tile.
    let tiles: Vec<TileId> = zone_32_tiles(bbox)
        .into_iter()
        .filter(|t| tile_touches(*t, Region::Hamburg))
        .collect();
    let model_item = |model: Model, mb_per_tile: f64, part: Part| {
        let open = tiles
            .iter()
            .filter(|t| !model.tile_cache_file(cache, **t).exists())
            .count();
        EstimateItem {
            what: model.label().to_string(),
            megabytes: open as f64 * mb_per_tile,
            cached: open == 0,
            part,
            store: Store::Cache,
        }
    };
    vec![
        per_km2_item(
            bbox,
            "orthophoto (LGV WMS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        model_item(Model::Dgm1, 2.0, Part::Terrain),
        model_item(Model::Bdom, 4.0, Part::Surface),
    ]
}

/// HVBG: the WMS orthophoto, the height models read out of the municipal zips (≈2.1 MB DGM1
/// and ≈3 MB DOM1 per 1 km tile touched, those already cached left out) and the metadata list
/// that finds the zips. The list's file name is only known after the product description is
/// read, so it counts as cached once any list is.
fn hvbg_items(bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    use crate::sources::hvbg::Model;
    let list_cached = std::fs::read_dir(cache.hvbg_dir().join("metadata"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().ends_with(".xlsx"));
    let tiles = zone_32_tiles(bbox);
    let model_item = |model: Model, mb_per_tile: f64, part: Part| {
        let open = tiles
            .iter()
            .filter(|t| {
                !cache
                    .hvbg_dir()
                    .join(model.code())
                    .join(format!("32_{:03}_{:04}.tif", t.east_km, t.north_km))
                    .exists()
            })
            .count();
        EstimateItem {
            what: model.label().to_string(),
            megabytes: open as f64 * mb_per_tile,
            cached: open == 0,
            part,
            store: Store::Cache,
        }
    };
    vec![
        per_km2_item(
            bbox,
            "orthophoto (HVBG WMS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        model_item(Model::Dgm1, 2.1, Part::Terrain),
        model_item(Model::Dom1, 3.0, Part::Surface),
        // Both models need the list; it goes with the terrain model, which the run needs.
        EstimateItem {
            what: "tile metadata list (HVBG)".to_string(),
            megabytes: 1.0,
            cached: list_cached,
            part: Part::Terrain,
            store: Store::Cache,
        },
    ]
}

/// Geobasis NRW: the WMS orthophoto, and the DGM1 (≈2.2 MB) and DOM1 (≈3 MB) per whole 1 km tile
/// that touches the state, those already cached left out.
fn nrw_items(bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    use crate::sources::geobasis_nrw::{Model, cached_tiles};
    let tiles: Vec<TileId> = zone_32_tiles(bbox)
        .into_iter()
        .filter(|t| tile_touches(*t, Region::NordrheinWestfalen))
        .collect();
    let model_item = |model: Model, mb_per_tile: f64, part: Part| {
        let cached = cached_tiles(cache, model, &tiles);
        let open = tiles.iter().filter(|t| !cached.contains(t)).count();
        EstimateItem {
            what: model.label().to_string(),
            megabytes: open as f64 * mb_per_tile,
            cached: open == 0,
            part,
            store: Store::Cache,
        }
    };
    vec![
        per_km2_item(
            bbox,
            "orthophoto (Geobasis NRW WMS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        model_item(Model::Dgm1, NRW_DGM1_TILE_MB, Part::Terrain),
        model_item(Model::Dom1, NRW_DOM1_TILE_MB, Part::Surface),
    ]
}

/// LVermGeo RP: the WMS orthophoto, and the DGM1 (≈1.5 MB) and DOM1 (≈3 MB) per whole 1 km tile
/// the run will download (those the kept tile list of the area names, else those touching the
/// state; `map_bbox` in EPSG:25832), those already cached left out, and the Metalink of a model
/// (11 MB) when the kept tile list gives one of the tiles no year.
fn rp_items(bbox: &BoundingBox, map_bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    use crate::sources::lvermgeo_rp::{
        METALINK_MB, Model, cached_tiles, metalink_to_fetch, tiles_to_fetch,
    };
    let tiles = tiles_to_fetch(cache, map_bbox);
    let mut items = vec![per_km2_item(
        bbox,
        "orthophoto (LVermGeo RP WMS)",
        ORTHOPHOTO_MB_PER_KM2,
        Part::Orthophoto,
    )];
    for (model, mb_per_tile, part) in [
        (Model::Dgm1, RP_DGM1_TILE_MB, Part::Terrain),
        (Model::Dom1, RP_DOM1_TILE_MB, Part::Surface),
    ] {
        let cached = cached_tiles(cache, model, map_bbox, &tiles);
        let open = tiles.iter().filter(|t| !cached.contains(t)).count();
        items.push(EstimateItem {
            what: model.label().to_string(),
            megabytes: open as f64 * mb_per_tile,
            cached: open == 0,
            part,
            store: Store::Cache,
        });
        if metalink_to_fetch(cache, model, map_bbox, &tiles) {
            items.push(EstimateItem {
                what: format!("Metalink of the {}", model.label()),
                megabytes: METALINK_MB,
                cached: false,
                part,
                store: Store::Cache,
            });
        }
    }
    items
}

/// LVermGeo SH: the WMS orthophoto, the DGM1 (a 1.5 MB GeoTIFF) and the bDOM (a 105 MB full-tile
/// download, reduced to 1 m after) per whole 1 km tile that can exist inside the state, those
/// already cached left out.
fn schleswig_holstein_items(bbox: &BoundingBox, cache: &Cache) -> Vec<EstimateItem> {
    use crate::sources::lvermgeo_sh::{Model, cached_tile_files};
    // Every square that touches the state has a (whole) tile.
    let tiles: Vec<TileId> = zone_32_tiles(bbox)
        .into_iter()
        .filter(|t| tile_touches(*t, Region::SchleswigHolstein))
        .collect();
    // A tile counts as cached only under the name its current vintage has.
    let model_item = |model: Model, mb_per_tile: f64, what: String, part: Part| {
        let files = cached_tile_files(cache, model);
        let open = tiles
            .iter()
            .filter(|t| !files.get(t).is_some_and(|file| file.exists()))
            .count();
        EstimateItem {
            what,
            megabytes: open as f64 * mb_per_tile,
            cached: open == 0,
            part,
            store: Store::Cache,
        }
    };
    vec![
        per_km2_item(
            bbox,
            "orthophoto (LVermGeo SH WMS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        model_item(
            Model::Dgm1,
            SH_DGM1_TILE_MB,
            Model::Dgm1.label().to_string(),
            Part::Terrain,
        ),
        model_item(
            Model::Bdom,
            SH_BDOM_TILE_MB,
            "surface model (LVermGeo SH bDOM, full-tile download)".to_string(),
            Part::Surface,
        ),
    ]
}

/// LAiV M-V: the orthophoto, DGM1 and DOM1 all come from WCS requests for the area itself, so
/// nothing is shared between projects or cached.
fn laiv_mv_items(bbox: &BoundingBox) -> Vec<EstimateItem> {
    use crate::sources::laiv_mv::Model;
    vec![
        per_km2_item(
            bbox,
            "orthophoto (LAiV M-V WCS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        per_km2_item(bbox, Model::Dgm1.label(), WCS_MB_PER_KM2, Part::Terrain),
        per_km2_item(bbox, Model::Dom1.label(), WCS_MB_PER_KM2, Part::Surface),
    ]
}

/// LGLN: the WMS orthophoto and the DGM1 and DOM1 from the WCS, all requested for the area
/// itself (the WCS in pieces of at most 2 km), so nothing is shared between projects or cached.
fn lgln_items(bbox: &BoundingBox) -> Vec<EstimateItem> {
    use crate::sources::lgln::Model;
    vec![
        per_km2_item(
            bbox,
            "orthophoto (LGLN WMS)",
            ORTHOPHOTO_MB_PER_KM2,
            Part::Orthophoto,
        ),
        per_km2_item(bbox, Model::Dgm1.label(), WCS_MB_PER_KM2, Part::Terrain),
        per_km2_item(bbox, Model::Dom1.label(), WCS_MB_PER_KM2, Part::Surface),
    ]
}

/// A download that grows with the area: `mb_per_km2` over `bbox` (at least 0.1 MB), requested
/// for the area itself and stored in the project folder.
fn per_km2_item(bbox: &BoundingBox, what: &str, mb_per_km2: f64, part: Part) -> EstimateItem {
    EstimateItem {
        what: what.to_string(),
        megabytes: (bbox.width() * bbox.height() / 1e6 * mb_per_km2).max(0.1),
        cached: false,
        part,
        store: Store::Project,
    }
}

/// LGB: orthophoto, DTM and nDSM (or bDOM inside Berlin) per 1 km tile.
fn lgb_items(tiles: &[TileId], cache: &Cache) -> Vec<EstimateItem> {
    let mut items = Vec::new();
    for &tile in tiles {
        for (product, mb, part) in [
            (Product::Orthophoto, DOP_TILE_MB, Part::Orthophoto),
            (Product::Dtm, DGM_TILE_MB, Part::Terrain),
        ] {
            items.push(EstimateItem {
                what: format!("{product} tile {}", tile.name()),
                megabytes: mb,
                cached: cache.has_tile(product, tile),
                part,
                store: Store::Cache,
            });
        }
        // LGB publishes no nDOM inside Berlin; the bDOM stands in for it.
        let inside_berlin = regions_for(&tile.bounds()) == [Region::Berlin];
        items.push(if inside_berlin {
            EstimateItem {
                what: format!("nDSM tile {} ({})", tile.name(), Product::ImageDsm),
                megabytes: BDOM_TILE_MB,
                cached: cache.has_tile(Product::ImageDsm, tile),
                part: Part::Surface,
                store: Store::Cache,
            }
        } else {
            EstimateItem {
                what: format!("nDSM tile {}", tile.name()),
                megabytes: NDSM_TILE_MB,
                cached: cache.has_ndsm(tile),
                part: Part::Surface,
                store: Store::Cache,
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
            in_place: Vec::new(),
        }
    } else {
        estimate_area(&bbox, &map_bbox, cache, today, false)
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
/// The K5 template is downloaded by `fetch`, not derived.
const FETCH_STEP: &str = "freemapper fetch or run (MCP: start_pipeline); Berlin only";

/// Raster templates that have previews.
const RASTER_LAYERS: [(&str, &str, &str); 5] = [
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
    (
        "k5",
        K5_RELPATH,
        "Berlin map 1:5000 (K5), to draw on or compare with",
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
            made_by: if name == "k5" {
                FETCH_STEP
            } else {
                DERIVE_STEP
            },
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
        let step = if layer == "k5" {
            FETCH_STEP
        } else {
            DERIVE_STEP
        };
        return Err(ProjectError::Layer(format!(
            "{layer} is not made yet; run {step}"
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
    use crate::sources::osm::ExtractSource;
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
    fn hessen_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Lessing-Gymnasium in Frankfurt, 25833 E 47838 N 5571740.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 47_838.0,
            northing: 5_571_740.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let preview = preview_area(&area, 20.0, &cache, today());
        let items = &preview.estimate.items;
        let find = |what: &str| {
            items
                .iter()
                .find(|i| i.what == what)
                .unwrap_or_else(|| panic!("{what} in {items:?}"))
        };
        // The estimate counts in the region CRS (25833).
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(near(find("orthophoto (HVBG WMS)").megabytes, km2 * 8.0));
        // The Downloadcenter is read in whole 1 km tiles, however small the area: the 200 m
        // school yard lies in one.
        assert!(near(find("terrain model (HVBG DGM1)").megabytes, 2.1));
        assert!(near(find("surface model (HVBG DOM1)").megabytes, 3.0));
        assert!(near(find("tile metadata list (HVBG)").megabytes, 1.0));
        assert!(near(
            find("OSM extract Hesse (shared by all projects)").megabytes,
            348.0
        ));
        assert!(
            items
                .iter()
                .filter(|i| i.what.contains("HVBG"))
                .all(|i| !i.cached)
        );
    }

    #[test]
    fn hvbg_estimate_counts_tiles_and_skips_cached_ones() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 47_838.0,
            northing: 5_571_740.0,
        })
        .unwrap();
        // About 2 km wide: crosses at least one tile edge.
        let area = square(p.lon, p.lat, 0.012);
        let bbox = area.projected_bbox().buffered(20.0);
        let tiles = zone_32_tiles(&bbox);
        assert!(tiles.len() >= 2, "{tiles:?}");
        let n = tiles.len() as f64;
        let item = |what: &str, cache: &Cache| {
            let preview = preview_area(&area, 20.0, cache, today());
            preview
                .estimate
                .items
                .iter()
                .find(|i| i.what == what)
                .unwrap()
                .clone()
        };
        let dgm1 = item("terrain model (HVBG DGM1)", &cache);
        assert!((dgm1.megabytes - 2.1 * n).abs() < 1e-9 && !dgm1.cached);
        // One tile of the DGM1 already in the cache.
        let dir = cache.hvbg_dir().join("dgm1");
        std::fs::create_dir_all(&dir).unwrap();
        let t = tiles[0];
        std::fs::write(
            dir.join(format!("32_{:03}_{:04}.tif", t.east_km, t.north_km)),
            b"x",
        )
        .unwrap();
        let dgm1 = item("terrain model (HVBG DGM1)", &cache);
        assert!((dgm1.megabytes - 2.1 * (n - 1.0)).abs() < 1e-9 && !dgm1.cached);
        let dom1 = item("surface model (HVBG DOM1)", &cache);
        assert!((dom1.megabytes - 3.0 * n).abs() < 1e-9);
    }

    /// A square map area (0.002 degrees) whose corner is at EPSG:25832 `easting`/`northing`.
    fn utm32_area(easting: f64, northing: f64) -> crate::area::MapArea {
        let p = crate::crs::Crs::Utm32
            .inverse(crate::crs::ProjectedPoint { easting, northing })
            .unwrap();
        square(p.lon, p.lat, 0.002)
    }

    fn find_item<'a>(items: &'a [EstimateItem], what: &str) -> &'a EstimateItem {
        items
            .iter()
            .find(|i| i.what == what)
            .unwrap_or_else(|| panic!("{what} in {items:?}"))
    }

    #[test]
    fn nrw_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Hildegard-von-Bingen-Gymnasium, Köln: inside the tile 354/5642.
        let area = utm32_area(354_600.0, 5_642_550.0);
        let preview = preview_area(&area, 20.0, &cache, today());
        let items = &preview.estimate.items;
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(near(
            find_item(items, "orthophoto (Geobasis NRW WMS)").megabytes,
            km2 * 8.0
        ));
        // Whole 1 km tiles, however small the area: the school yard lies in one.
        let dgm1 = find_item(items, "terrain model (Geobasis NRW DGM1)");
        assert!(near(dgm1.megabytes, 2.2) && !dgm1.cached, "{dgm1:?}");
        let dom1 = find_item(items, "surface model (Geobasis NRW DOM1)");
        assert!(near(dom1.megabytes, 3.0) && !dom1.cached, "{dom1:?}");
        assert!(near(
            find_item(
                items,
                "OSM extract North Rhine-Westphalia, Köln (shared by all projects)"
            )
            .megabytes,
            225.2
        ));
        assert!(preview.estimate.tiles.is_empty());
        assert!(!items.iter().any(|i| i.what.contains("LGB")));
    }

    #[test]
    fn nrw_estimate_in_the_west_with_negative_eastings() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Aachen cathedral: 25833 E -128141, so far west that the eastings are negative.
        let c = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: -128_141.0,
                    northing: 5_662_838.0,
                },
            )
            .unwrap();
        let area = utm32_area(c.easting, c.northing);
        assert!(area.projected_bbox().min_e < 0.0);
        let preview = preview_area(&area, 20.0, &cache, today());
        assert!(
            !preview.too_large && !preview.outside_coverage,
            "{preview:?}"
        );
        let items = &preview.estimate.items;
        assert!(find_item(items, "terrain model (Geobasis NRW DGM1)").megabytes >= 2.2);
        assert!(find_item(items, "surface model (Geobasis NRW DOM1)").megabytes >= 3.0);
        assert!(
            items
                .iter()
                .any(|i| i.what.starts_with("OSM extract North Rhine-Westphalia")),
            "{items:?}"
        );
    }

    #[test]
    fn nrw_estimate_on_a_district_border_lists_both_extracts() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Köln-Worringen on the Köln / Düsseldorf district border, 25833 E -70550 / N 5689830.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: -70_550.0,
            northing: 5_689_830.0,
        })
        .unwrap();
        let area = square(p.lon - 0.006, p.lat - 0.004, 0.012);
        let preview = preview_area(&area, 20.0, &cache, today());
        let items = &preview.estimate.items;
        for (label, megabytes) in [("Köln", 225.2), ("Düsseldorf", 217.2)] {
            let what =
                format!("OSM extract North Rhine-Westphalia, {label} (shared by all projects)");
            let item = find_item(items, &what);
            assert!(
                (item.megabytes - megabytes).abs() < 1e-9 && !item.cached,
                "{item:?}"
            );
        }
        assert_eq!(
            items
                .iter()
                .filter(|i| i.what.starts_with("OSM extract"))
                .count(),
            2
        );
    }

    #[test]
    fn nrw_estimate_counts_tiles_and_marks_cached_ones() {
        use crate::sources::geobasis_nrw::{Model, cached_tiles};
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // Two tiles side by side: the tile border 355 km east lies inside the area.
        let area = utm32_area(354_900.0, 5_642_400.0);
        let item = |what: &str, cache: &Cache| {
            let preview = preview_area(&area, 20.0, cache, today());
            find_item(&preview.estimate.items, what).clone()
        };
        let before = item("terrain model (Geobasis NRW DGM1)", &cache);
        assert!(before.megabytes > 2.2 * 1.9 && !before.cached, "{before:?}");
        // Both tiles of the DGM1 in the cache, under the names the kept list gives.
        let tiles = [(354, 5642), (355, 5642)];
        let names: Vec<(String, &str)> = tiles
            .iter()
            .map(|(e, n)| (format!("dgm1_32_{e}_{n}_1_nw_2022"), "2022-02-12"))
            .collect();
        let dir = cache.geobasis_nrw_dir();
        std::fs::create_dir_all(dir.join("meta")).unwrap();
        std::fs::create_dir_all(dir.join("dgm1")).unwrap();
        std::fs::write(
            dir.join("meta/dgm1_2026-10-26.zip"),
            crate::fetch::test_support::nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &names),
        )
        .unwrap();
        let grid = |e: u32, n: u32| crate::raster::GridSpec {
            west: f64::from(e) * 1000.0,
            north: f64::from(n + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let (e, n) = tiles[0];
        std::fs::write(
            dir.join(format!("dgm1/dgm1_32_{e}_{n}_1_nw_2022.tif")),
            crate::fetch::test_support::geotiff(&grid(e, n), &vec![1.0; 1_000_000]),
        )
        .unwrap();
        assert_eq!(
            cached_tiles(&cache, Model::Dgm1, &[]).len(),
            0,
            "only the asked tiles count"
        );
        let asked: Vec<TileId> = tiles
            .iter()
            .map(|&(east_km, north_km)| TileId { east_km, north_km })
            .collect();
        assert_eq!(cached_tiles(&cache, Model::Dgm1, &asked).len(), 1);
        let one_open = item("terrain model (Geobasis NRW DGM1)", &cache);
        assert!(
            (one_open.megabytes - 2.2).abs() < 1e-9 && !one_open.cached,
            "{one_open:?}"
        );
        let (e, n) = tiles[1];
        std::fs::write(
            dir.join(format!("dgm1/dgm1_32_{e}_{n}_1_nw_2022.tif")),
            crate::fetch::test_support::geotiff(&grid(e, n), &vec![1.0; 1_000_000]),
        )
        .unwrap();
        let none_open = item("terrain model (Geobasis NRW DGM1)", &cache);
        assert!(
            none_open.megabytes == 0.0 && none_open.cached,
            "{none_open:?}"
        );
        // The DOM1 is still open.
        assert!(!item("surface model (Geobasis NRW DOM1)", &cache).cached);
    }

    #[test]
    fn nrw_estimate_does_not_list_a_tile_the_server_lacks_as_a_download() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let area = utm32_area(354_900.0, 5_642_400.0);
        let dir = cache.geobasis_nrw_dir();
        std::fs::create_dir_all(dir.join("meta")).unwrap();
        std::fs::create_dir_all(dir.join("dgm1")).unwrap();
        let names: Vec<(String, &str)> = [354, 355]
            .iter()
            .map(|e| (format!("dgm1_32_{e}_5642_1_nw_2022"), "2022-02-12"))
            .collect();
        std::fs::write(
            dir.join("meta/dgm1_2026-10-26.zip"),
            crate::fetch::test_support::nrw_meta_zip("dgm1_nw.csv", "2026-10-26", &names),
        )
        .unwrap();
        // Both tiles were listed, and the server answered 404 for both.
        for (name, _) in &names {
            std::fs::write(dir.join(format!("dgm1/{name}.missing")), b"").unwrap();
        }
        let preview = preview_area(&area, 20.0, &cache, today());
        let item = find_item(&preview.estimate.items, "terrain model (Geobasis NRW DGM1)");
        assert!(item.megabytes == 0.0 && item.cached, "{item:?}");
    }

    /// 25832 E 565500 / N 5947100 as a map centre: Hamburg-Langenhorn, 600 m south of the border
    /// to Norderstedt.
    fn langenhorn_area(size: f64) -> crate::area::MapArea {
        let p = crate::crs::Crs::Utm33
            .reproject(
                crate::crs::Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 565_500.0,
                    northing: 5_947_100.0,
                },
            )
            .unwrap();
        let p = crate::crs::utm33_inverse(p).unwrap();
        square(p.lon, p.lat, size)
    }

    #[test]
    fn rp_estimate() {
        use crate::fetch::test_support::rp_tile_list;
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Görres-Gymnasium, Koblenz: inside the tile 400/5579, with negative 25833 eastings.
        let area = utm32_area(400_300.0, 5_579_500.0);
        assert!(area.projected_bbox().max_e < 0.0);
        let item = |cache: &Cache| {
            let preview = preview_area(&area, 20.0, cache, today());
            assert!(
                !preview.too_large && !preview.outside_coverage,
                "{preview:?}"
            );
            assert!(preview.estimate.tiles.is_empty());
            preview.estimate.items
        };
        let items = item(&cache);
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(near(
            find_item(&items, "orthophoto (LVermGeo RP WMS)").megabytes,
            km2 * 8.0
        ));
        // Whole 1 km tiles, however small the area: the school yard lies in one.
        let dgm1 = find_item(&items, "terrain model (LVermGeo RP DGM1)");
        assert!(near(dgm1.megabytes, 1.5) && !dgm1.cached, "{dgm1:?}");
        let dom1 = find_item(&items, "surface model (LVermGeo RP DOM1)");
        assert!(near(dom1.megabytes, 3.0) && !dom1.cached, "{dom1:?}");
        assert!(near(
            find_item(
                &items,
                "OSM extract Rhineland-Palatinate (shared by all projects)"
            )
            .megabytes,
            275.0
        ));
        assert!(
            !items.iter().any(|i| i.what.contains("Metalink")),
            "{items:?}"
        );
        assert!(!items.iter().any(|i| i.what.contains("LGB")));

        // The DGM1 tile in the cache, under the name its year gives.
        let dir = cache.lvermgeo_rp_dir();
        std::fs::create_dir_all(dir.join("dgm1")).unwrap();
        std::fs::write(dir.join("dgm1/dgm1_32_400_5579_1_rp_2024.tif"), b"II*\0").unwrap();
        let items = item(&cache);
        let dgm1 = find_item(&items, "terrain model (LVermGeo RP DGM1)");
        assert!(dgm1.megabytes == 0.0 && dgm1.cached, "{dgm1:?}");
        assert!(!find_item(&items, "surface model (LVermGeo RP DOM1)").cached);

        // A kept tile list that gives the tile no year: the Metalink (11 MB a model) will name
        // its file, which is then downloaded.
        std::fs::create_dir_all(dir.join("tile_list")).unwrap();
        let tile = TileId {
            east_km: 400,
            north_km: 5579,
        };
        std::fs::write(
            crate::sources::lvermgeo_rp::list_path(
                &dir,
                &area
                    .projected_bbox_in(crate::crs::Crs::Utm32)
                    .buffered(20.0),
            ),
            rp_tile_list(&[(tile, "", "")]),
        )
        .unwrap();
        let items = item(&cache);
        let metalink = find_item(&items, "Metalink of the surface model (LVermGeo RP DOM1)");
        assert!(
            near(metalink.megabytes, 11.0) && !metalink.cached,
            "{metalink:?}"
        );
        let dom1 = find_item(&items, "surface model (LVermGeo RP DOM1)");
        assert!(near(dom1.megabytes, 3.0) && !dom1.cached, "{dom1:?}");
        // A fresh kept DOM1 Metalink that names no file for it: nothing to download.
        std::fs::create_dir_all(dir.join("meta4")).unwrap();
        std::fs::write(
            dir.join("meta4/dom1_tif_07.meta4"),
            "<metalink xmlns=\"urn:ietf:params:xml:ns:metalink\"></metalink>\n",
        )
        .unwrap();
        let items = item(&cache);
        let dom1 = find_item(&items, "surface model (LVermGeo RP DOM1)");
        assert!(dom1.megabytes == 0.0 && dom1.cached, "{dom1:?}");
        assert!(
            !items
                .iter()
                .any(|i| i.what.contains("Metalink of the surface")),
            "{items:?}"
        );
    }

    #[test]
    fn rp_estimate_counts_the_tiles_the_kept_list_names() {
        use crate::fetch::test_support::rp_tile_list;
        use crate::sources::lvermgeo_rp::list_path;
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // At the Hesse border in Mainz: the tile 447/5540 is 43 % RP, 448/5540 lies wholly in
        // Hesse (Wiesbaden). An area from E 446500 to 448100 covers 446 to 448.
        let at = |east_km, north_km| TileId { east_km, north_km };
        assert!(tile_touches(at(447, 5540), Region::RheinlandPfalz));
        assert!(!tile_touches(at(448, 5540), Region::RheinlandPfalz));
        let p = |easting, northing| {
            crate::crs::Crs::Utm32
                .inverse(crate::crs::ProjectedPoint { easting, northing })
                .unwrap()
        };
        let (sw, ne) = (p(446_500.0, 5_540_300.0), p(448_100.0, 5_540_700.0));
        let area = crate::area::MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[[{w},{s}],[{e},{s}],[{e},{n}],[{w},{n}],[{w},{s}]]]}}"#,
            w = sw.lon,
            s = sw.lat,
            e = ne.lon,
            n = ne.lat
        ))
        .unwrap();
        let region_bbox = area.projected_bbox().buffered(20.0);
        assert_eq!(
            raster_region(&region_bbox),
            Some(Region::RheinlandPfalz),
            "the centre lies in RP"
        );
        let map_bbox = area
            .projected_bbox_in(crate::crs::Crs::Utm32)
            .buffered(20.0);
        let area_tiles = tiles_covering(&map_bbox);
        let dgm1 = |cache: &Cache| {
            let preview = preview_area(&area, 20.0, cache, today());
            find_item(&preview.estimate.items, "terrain model (LVermGeo RP DGM1)").clone()
        };
        // Before a run: the tiles that touch the state.
        let touching = area_tiles
            .iter()
            .filter(|t| tile_touches(**t, Region::RheinlandPfalz))
            .count();
        assert!(touching < area_tiles.len(), "{area_tiles:?}");
        let before = dgm1(&cache);
        assert!(
            (before.megabytes - touching as f64 * 1.5).abs() < 1e-9,
            "{before:?}"
        );
        // The list kept for this area names every tile but 446/5540: those are what the run
        // downloads, the Hesse tile 448/5540 among them.
        let listed: Vec<(TileId, &str, &str)> = area_tiles
            .iter()
            .filter(|t| **t != at(446, 5540))
            .map(|t| (*t, "2024", "30.03.24, 06.04.24"))
            .collect();
        assert!(listed.iter().any(|(t, _, _)| *t == at(448, 5540)));
        let path = list_path(&cache.lvermgeo_rp_dir(), &map_bbox);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, rp_tile_list(&listed)).unwrap();
        let after = dgm1(&cache);
        assert!(
            (after.megabytes - listed.len() as f64 * 1.5).abs() < 1e-9,
            "{after:?}"
        );
    }

    #[test]
    fn hamburg_estimate_counts_every_tile_touching_hamburg() {
        use crate::sources::lgv::Model;
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let area = langenhorn_area(0.02);
        let all = zone_32_tiles(&area.projected_bbox().buffered(20.0));
        // 565/5948 lies mostly in Norderstedt but touches Hamburg (its centre is not in it);
        // 565/5949 is Schleswig-Holstein's only.
        let at = |east_km, north_km| TileId { east_km, north_km };
        assert!(
            all.contains(&at(565, 5948)) && all.contains(&at(565, 5949)),
            "{all:?}"
        );
        let touching = all
            .iter()
            .filter(|t| tile_touches(**t, Region::Hamburg))
            .count();
        assert!(touching < all.len());
        let preview = preview_area(&area, 20.0, &cache, today());
        let find = |what: &str| {
            preview
                .estimate
                .items
                .iter()
                .find(|i| i.what == what)
                .unwrap()
                .clone()
        };
        assert!((find(Model::Dgm1.label()).megabytes - 2.0 * touching as f64).abs() < 1e-9);
        assert!((find(Model::Bdom.label()).megabytes - 4.0 * touching as f64).abs() < 1e-9);
    }

    #[test]
    fn schleswig_holstein_estimate_counts_tiles_mostly_in_hamburg() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The area starts at 25832 E 567700 / N 5947700 (Norderstedt) and reaches 2 km north-east.
        let p = crate::crs::Crs::Utm33
            .reproject(
                crate::crs::Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 567_700.0,
                    northing: 5_947_700.0,
                },
            )
            .unwrap();
        let p = crate::crs::utm33_inverse(p).unwrap();
        let area = square(p.lon, p.lat, 0.02);
        let all = zone_32_tiles(&area.projected_bbox().buffered(20.0));
        let at = |east_km, north_km| TileId { east_km, north_km };
        // 568/5948 is mostly Hamburg's (its centre is not in Schleswig-Holstein) but touches it;
        // 567/5947 and 568/5947 do not.
        assert!(
            all.contains(&at(568, 5948)) && all.contains(&at(567, 5947)),
            "{all:?}"
        );
        let touching = all
            .iter()
            .filter(|t| crate::sources::region::tile_touches(**t, Region::SchleswigHolstein))
            .count();
        assert!(touching < all.len());
        let preview = preview_area(&area, 20.0, &cache, today());
        let find = |prefix: &str| {
            preview
                .estimate
                .items
                .iter()
                .find(|i| i.what.starts_with(prefix))
                .unwrap()
                .clone()
        };
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(near(find("terrain model").megabytes, 1.5 * touching as f64));
        assert!(near(
            find("surface model").megabytes,
            105.0 * touching as f64
        ));
    }

    #[test]
    fn schleswig_holstein_estimate_merges_all_kept_tile_lists() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 169_108.0,
            northing: 5_960_473.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let root = cache.lvermgeo_sh_dir();
        std::fs::create_dir_all(root.join("overview")).unwrap();
        std::fs::create_dir_all(root.join("dgm1")).unwrap();
        let list = include_str!("../../../tests/fixtures/schleswig-holstein-overview-dgm1.json");
        let dgm1 = || {
            preview_area(&area, 20.0, &cache, today())
                .estimate
                .items
                .into_iter()
                .find(|i| i.what == "terrain model (LVermGeo SH DGM1)")
                .unwrap()
        };
        let keep = |file: &str, text: &str| {
            std::fs::write(root.join("overview").join(file), text).unwrap();
            // The lists are ordered by modification time.
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        // The school tile 565/5949 is named only by the list of another project's area, and
        // its file is cached: the newest list (of other tiles) does not hide it.
        keep("dgm1_other.json", list);
        std::fs::write(
            root.join("dgm1").join("dgm1_32_565_5949_1_sh_2022.tif"),
            b"x",
        )
        .unwrap();
        keep(
            "dgm1_newest.json",
            &list
                .replace("565_5949", "570_5949")
                .replace("325655949", "325705949"),
        );
        assert!(dgm1().cached, "{:?}", dgm1());
        // An older list names an older vintage, the newer one the current: the newer wins, so
        // the cached old file does not count.
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let root = cache.lvermgeo_sh_dir();
        std::fs::create_dir_all(root.join("overview")).unwrap();
        std::fs::create_dir_all(root.join("dgm1")).unwrap();
        let keep = |file: &str, text: &str| {
            std::fs::write(root.join("overview").join(file), text).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let dgm1 = || {
            preview_area(&area, 20.0, &cache, today())
                .estimate
                .items
                .into_iter()
                .find(|i| i.what == "terrain model (LVermGeo SH DGM1)")
                .unwrap()
        };
        keep(
            "dgm1_before.json",
            &list.replace("565_5949_1_sh_2022", "565_5949_1_sh_2019"),
        );
        keep("dgm1_after.json", list);
        std::fs::write(
            root.join("dgm1").join("dgm1_32_565_5949_1_sh_2019.tif"),
            b"x",
        )
        .unwrap();
        assert!(!dgm1().cached && dgm1().megabytes == 1.5, "{:?}", dgm1());
        std::fs::write(
            root.join("dgm1").join("dgm1_32_565_5949_1_sh_2022.tif"),
            b"x",
        )
        .unwrap();
        assert!(dgm1().cached, "{:?}", dgm1());
    }

    #[test]
    fn schleswig_holstein_estimate_needs_the_current_vintage_of_a_cached_tile() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 169_108.0,
            northing: 5_960_473.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let tiles = zone_32_tiles(&area.projected_bbox().buffered(20.0));
        assert_eq!(tiles.len(), 1, "{tiles:?}");
        let t = tiles[0];
        let root = cache.lvermgeo_sh_dir();
        // The tile list kept from the last online run names the current files (2022).
        let name = format!("dgm1_32_{:03}_{:04}_1_sh_2022", t.east_km, t.north_km);
        let list = include_str!("../../../tests/fixtures/schleswig-holstein-overview-dgm1.json");
        assert!(list.contains(&name), "{name}");
        std::fs::create_dir_all(root.join("overview")).unwrap();
        std::fs::write(root.join("overview").join("dgm1_1_2_3_4.json"), list).unwrap();
        std::fs::create_dir_all(root.join("dgm1")).unwrap();
        // Only an older vintage is cached: the tile is downloaded again.
        std::fs::write(
            root.join("dgm1").join(format!(
                "dgm1_32_{:03}_{:04}_1_sh_2019.tif",
                t.east_km, t.north_km
            )),
            b"x",
        )
        .unwrap();
        let dgm1 = |cache: &Cache| {
            preview_area(&area, 20.0, cache, today())
                .estimate
                .items
                .into_iter()
                .find(|i| i.what == "terrain model (LVermGeo SH DGM1)")
                .unwrap()
        };
        assert!(!dgm1(&cache).cached && dgm1(&cache).megabytes == 1.5);
        std::fs::write(root.join("dgm1").join(format!("{name}.tif")), b"x").unwrap();
        assert!(dgm1(&cache).cached);
    }

    #[test]
    fn ni_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Kaiser-Wilhelm- und Ratsgymnasium, Hannover (25832 E 551900 / N 5803300).
        let area = utm32_area(551_800.0, 5_803_200.0);
        let preview = preview_area(&area, 20.0, &cache, today());
        assert!(
            !preview.too_large && !preview.outside_coverage,
            "{preview:?}"
        );
        let items = &preview.estimate.items;
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        // Everything comes from WMS and WCS requests for the area itself: nothing is cached.
        for (what, mb_per_km2) in [
            ("orthophoto (LGLN WMS)", 8.0),
            ("terrain model (LGLN DGM1)", 4.0),
            ("surface model (LGLN DOM1)", 4.0),
        ] {
            let item = find_item(items, what);
            assert!(
                near(item.megabytes, km2 * mb_per_km2) && !item.cached,
                "{item:?}"
            );
        }
        assert!(near(
            find_item(items, "OSM extract Lower Saxony (shared by all projects)").megabytes,
            510.0
        ));
        assert!(preview.estimate.tiles.is_empty());
        assert!(!items.iter().any(|i| i.what.contains("LGB")), "{items:?}");
        // A large area is estimated by its size, too (four 2 km GetCoverages per model).
        let p = |e: f64, n: f64| {
            crate::crs::Crs::Utm32
                .inverse(crate::crs::ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap()
        };
        let (sw, ne) = (p(550_000.0, 5_801_000.0), p(553_500.0, 5_804_500.0));
        let large = crate::area::MapArea::from_geojson(&format!(
            r#"{{"type":"Polygon","coordinates":[[[{w},{s}],[{e},{s}],[{e},{n}],[{w},{n}],[{w},{s}]]]}}"#,
            w = sw.lon,
            s = sw.lat,
            e = ne.lon,
            n = ne.lat
        ))
        .unwrap();
        let preview = preview_area(&large, 20.0, &cache, today());
        assert!(!preview.too_large, "{preview:?}");
        let bbox = large.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        assert!(km2 > 12.0, "{km2}");
        let dgm1 = find_item(&preview.estimate.items, "terrain model (LGLN DGM1)");
        assert!(near(dgm1.megabytes, km2 * 4.0), "{dgm1:?}");
    }

    #[test]
    fn mv_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Innerstaedtisches Gymnasium in Rostock, 25833 E 312157 N 5996317.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 312_157.0,
            northing: 5_996_317.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let preview = preview_area(&area, 20.0, &cache, today());
        let items = &preview.estimate.items;
        let find = |what: &str| {
            items
                .iter()
                .find(|i| i.what == what)
                .unwrap_or_else(|| panic!("{what} in {items:?}"))
        };
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(near(find("orthophoto (LAiV M-V WCS)").megabytes, km2 * 8.0));
        assert!(near(
            find("terrain model (LAiV M-V DGM1)").megabytes,
            km2 * 4.0
        ));
        assert!(near(
            find("surface model (LAiV M-V DOM1)").megabytes,
            km2 * 4.0
        ));
        assert!(near(
            find("OSM extract Mecklenburg-Vorpommern (shared by all projects)").megabytes,
            128.8
        ));
        assert!(
            items
                .iter()
                .filter(|i| i.what.contains("LAiV M-V"))
                .all(|i| !i.cached)
        );
    }

    #[test]
    fn schleswig_holstein_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Coppernicus-Gymnasium in Norderstedt, 25833 E 169108 N 5960473.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 169_108.0,
            northing: 5_960_473.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let preview = preview_area(&area, 20.0, &cache, today());
        let items = &preview.estimate.items;
        let find = |what: &str| {
            items
                .iter()
                .find(|i| i.what == what)
                .unwrap_or_else(|| panic!("{what} in {items:?}"))
        };
        let bbox = area.projected_bbox().buffered(20.0);
        let km2 = bbox.width() * bbox.height() / 1e6;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        let tiles = zone_32_tiles(&bbox).len() as f64;
        assert!(near(
            find("orthophoto (LVermGeo SH WMS)").megabytes,
            km2 * 8.0
        ));
        assert!(near(
            find("terrain model (LVermGeo SH DGM1)").megabytes,
            1.5 * tiles
        ));
        let bdom = find("surface model (LVermGeo SH bDOM, full-tile download)");
        assert!(near(bdom.megabytes, 105.0 * tiles), "{bdom:?}");
        assert!(near(
            find("OSM extract Schleswig-Holstein (shared by all projects)").megabytes,
            159.0
        ));
        assert!(
            items
                .iter()
                .filter(|i| i.what.contains("LVermGeo SH"))
                .all(|i| !i.cached)
        );
        // The school lies in one tile (565/5949); its current files are in the cache.
        assert_eq!(zone_32_tiles(&bbox).len(), 1);
        let root = cache.lvermgeo_sh_dir();
        std::fs::create_dir_all(root.join("overview")).unwrap();
        for (folder, list, file) in [
            (
                "dgm1",
                include_str!("../../../tests/fixtures/schleswig-holstein-overview-dgm1.json"),
                "dgm1_32_565_5949_1_sh_2022.tif",
            ),
            (
                "bdom",
                include_str!("../../../tests/fixtures/schleswig-holstein-overview-bdom.json"),
                "bdom20nc_32_565_5949_1_sh_2023_1m.tif",
            ),
        ] {
            std::fs::write(
                root.join("overview").join(format!("{folder}_1_2_3_4.json")),
                list,
            )
            .unwrap();
            std::fs::create_dir_all(root.join(folder)).unwrap();
            std::fs::write(root.join(folder).join(file), b"x").unwrap();
        }
        let preview = preview_area(&area, 20.0, &cache, today());
        for prefix in ["terrain model", "surface model"] {
            let item = preview
                .estimate
                .items
                .iter()
                .find(|i| i.what.starts_with(prefix))
                .unwrap();
            assert!(item.cached && item.megabytes == 0.0, "{item:?}");
        }
    }

    #[test]
    fn hamburg_estimate_marks_cached_tiles() {
        use crate::sources::lgv::Model;
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 168_078.0,
            northing: 5_949_079.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let tiles = zone_32_tiles(&area.projected_bbox().buffered(20.0));
        let item = |what: &str, cache: &Cache| {
            preview_area(&area, 20.0, cache, today())
                .estimate
                .items
                .into_iter()
                .find(|i| i.what == what)
                .unwrap()
        };
        let label = Model::Dgm1.label();
        assert!(!item(label, &cache).cached);
        // Every tile of the DGM1 in the cache, none of the bDOM.
        let dir = cache.lgv_dir().join("dgm1").join(Model::Dgm1.release());
        std::fs::create_dir_all(&dir).unwrap();
        for (i, t) in tiles.iter().enumerate() {
            std::fs::write(
                dir.join(format!("dgm1_32_{:03}_{:04}_.tif", t.east_km, t.north_km)),
                b"x",
            )
            .unwrap();
            let dgm1 = item(label, &cache);
            let open = (tiles.len() - i - 1) as f64;
            assert!((dgm1.megabytes - 2.0 * open).abs() < 1e-9, "{dgm1:?}");
            assert_eq!(dgm1.cached, open == 0.0);
        }
        let bdom = item(Model::Bdom.label(), &cache);
        assert!(!bdom.cached && (bdom.megabytes - 4.0 * tiles.len() as f64).abs() < 1e-9);
    }

    #[test]
    fn lgv_estimate_counts_whole_tiles() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        // The Gymnasium Eppendorf, Hamburg, 25833 E 168078 N 5949079: a 200 m yard.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 168_078.0,
            northing: 5_949_079.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let tiles = zone_32_tiles(&area.projected_bbox().buffered(20.0)).len() as f64;
        assert!((1.0..=4.0).contains(&tiles), "{tiles}");
        let preview = preview_area(&area, 20.0, &cache, today());
        let mb = |what: &str| {
            preview
                .estimate
                .items
                .iter()
                .find(|i| i.what == what)
                .unwrap()
                .megabytes
        };
        assert!((mb("terrain model (LGV DGM1 2022)") - 2.0 * tiles).abs() < 1e-9);
        assert!((mb("surface model (LGV bDOM 2022)") - 4.0 * tiles).abs() < 1e-9);
    }

    #[test]
    fn cached_hvbg_metadata_list_is_not_estimated() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 47_838.0,
            northing: 5_571_740.0,
        })
        .unwrap();
        let area = square(p.lon, p.lat, 0.002);
        let list = |cache: &Cache| {
            let preview = preview_area(&area, 20.0, cache, today());
            let item = preview
                .estimate
                .items
                .iter()
                .find(|i| i.what == "tile metadata list (HVBG)")
                .unwrap()
                .clone();
            (item.cached, preview.estimate.total_megabytes)
        };
        let (cached, total) = list(&cache);
        assert!(!cached);
        let dir = cache.hvbg_dir().join("metadata");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Metadaten_20260101.xlsx"), b"x").unwrap();
        let (cached, after) = list(&cache);
        assert!(cached);
        assert!((total - after - 1.0).abs() < 1e-9);
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
        // The older extract serves when the newer one does not fit.
        assert_eq!(osm.part, Part::OsmUpdate);
        assert_eq!(
            estimate.total_megabytes,
            ExtractSource::Brandenburg.megabytes()
        );
    }

    #[test]
    fn berlin_plus_estimate() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let mitte = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_500.0,
            max_n: 5_821_000.0,
        };
        let km2 = 1.5;
        let estimate = estimate_area(&mitte, &mitte, &cache, today(), false);
        let find = |what: &str| estimate.items.iter().find(|i| i.what == what).cloned();
        // +4 MB per km² for the vector layers, +15 MB per km² for the K5 template (live runs
        // 2026-11-09: Viktoriapark 3.7 and 9.1 MB per km², Helene-Haeusler-Schule 4.6 and 17).
        let layers = find(BERLIN_LAYERS_ITEM).expect("Berlin layers");
        assert!((layers.megabytes - km2 * 4.0).abs() < 1e-9, "{layers:?}");
        assert!(!layers.cached);
        let k5 = find(BERLIN_K5_ITEM).expect("K5 template");
        assert!((k5.megabytes - km2 * 15.0).abs() < 1e-9, "{k5:?}");
        assert_eq!((k5.part, k5.store), (Part::K5, Store::Project));
        // The K5 map and the layers are optional, as the fetch treats them.
        assert!(!k5.part.required());
        assert_eq!(
            (layers.part, layers.store),
            (Part::BerlinLayers, Store::Project)
        );
        assert!(!layers.part.required());
        let other: f64 = estimate
            .items
            .iter()
            .filter(|i| i.what != BERLIN_LAYERS_ITEM && i.what != BERLIN_K5_ITEM)
            .map(|i| i.megabytes)
            .sum();
        assert!((estimate.total_megabytes - other - km2 * 19.0).abs() < 1e-6);
        // Outside Berlin the estimate has neither.
        let strausberg = BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_000.0,
            max_e: 422_000.0,
            max_n: 5_822_000.0,
        };
        let estimate = estimate_area(&strausberg, &strausberg, &cache, today(), false);
        assert!(
            estimate
                .items
                .iter()
                .all(|i| i.what != BERLIN_LAYERS_ITEM && i.what != BERLIN_K5_ITEM)
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
    fn k5_template_has_a_layer_and_a_preview() {
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let k5 = |dir: &Path| {
            list_layers(dir)
                .into_iter()
                .find(|l| l.name == "k5")
                .unwrap()
        };
        assert!(!k5(dir.path()).ready);
        assert!(k5(dir.path()).made_by.contains("fetch"));
        let err = preview_png(dir.path(), "k5", 100).unwrap_err();
        assert!(err.to_string().contains("freemapper fetch"), "{err}");
        std::fs::create_dir_all(dir.path().join("templates")).unwrap();
        image::RgbImage::new(400, 200)
            .save(dir.path().join(K5_RELPATH))
            .unwrap();
        assert!(k5(dir.path()).ready);
        let png = preview_png(dir.path(), "k5", 100).unwrap();
        let small = image::load_from_memory(&png).unwrap();
        assert_eq!((small.width(), small.height()), (100, 50));
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
    fn estimate_items_name_their_part_and_store() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(cache_dir.path().to_path_buf());
        let dir = tempfile::tempdir().unwrap();
        school_project(dir.path());
        let lgb = estimate_download(dir.path(), &cache, today()).unwrap();
        let of = |items: &[EstimateItem], what: &str| {
            let item = find_item(items, what);
            (item.part, item.store)
        };
        assert_eq!(
            of(
                &lgb.items,
                &format!("{} tile 33421-5821", Product::Orthophoto)
            ),
            (Part::Orthophoto, Store::Cache)
        );
        assert_eq!(
            of(&lgb.items, &format!("{} tile 33421-5821", Product::Dtm)),
            (Part::Terrain, Store::Cache)
        );
        assert_eq!(
            of(&lgb.items, "nDSM tile 33421-5821"),
            (Part::Surface, Store::Cache)
        );
        assert_eq!(
            of(&lgb.items, "ALKIS buildings and land use"),
            (Part::MapData, Store::Project)
        );
        // No extract cached yet: the run needs it.
        let osm = "OSM extract Brandenburg (shared by all projects)";
        assert_eq!(of(&lgb.items, osm), (Part::MapData, Store::Cache));
        // A per-area provider stores what it requests in the project folder.
        let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
            easting: 47_838.0,
            northing: 5_571_740.0,
        })
        .unwrap();
        let hesse = preview_area(&square(p.lon, p.lat, 0.002), 20.0, &cache, today()).estimate;
        assert_eq!(
            of(&hesse.items, "orthophoto (HVBG WMS)"),
            (Part::Orthophoto, Store::Project)
        );
        assert_eq!(
            of(&hesse.items, "tile metadata list (HVBG)"),
            (Part::Terrain, Store::Cache)
        );
        assert_eq!(
            of(&hesse.items, "surface model (HVBG DOM1)"),
            (Part::Surface, Store::Cache)
        );
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
                "ndsm",
                "k5"
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
