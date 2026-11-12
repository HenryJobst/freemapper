//! The raster downloads of the eleven states for the estimate, as the `*_items` functions of
//! `layers.rs` gave them (plan C5): the texts, the sizes and what counts as cached are
//! unchanged, the items are asked through
//! [`RegionAdapter::estimate`](freemapper_adapter_api::adapter::RegionAdapter).

use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::estimate::{
    EstimateArea, EstimateEnv, ORTHOPHOTO_MB_PER_KM2, WCS_MB_PER_KM2, per_km2_item,
};
use freemapper_adapter_api::records::{EstimateItem, Part, Store};

use crate::crs::BoundingBox;
use crate::fetch::TilePaths as _;
use crate::sources::lgb::{Product, ProductTiles as _};
use crate::sources::region::{Provider, Region, provider_tiles, regions_for, tile_touches};
use crate::tiles::{TileId, tiles_covering};

/// Typical download sizes in MB.
const DOP_TILE_MB: f64 = 8.0;
const DGM_TILE_MB: f64 = 1.3;
const NDSM_TILE_MB: f64 = 15.0;
/// The bDOM GeoTIFF that replaces the nDOM inside Berlin.
pub(crate) const BDOM_TILE_MB: f64 = 30.0;
/// GeoSN's 2 km tiles.
pub(crate) const GEOSN_DGM1_TILE_MB: f64 = 14.0;
const GEOSN_DOM1_TILE_MB: f64 = 15.0;
/// LVermGeo SH per whole 1 km tile: the DGM1 GeoTIFF and the bDOM 0.2 m (reduced to 1 m after).
const SH_DGM1_TILE_MB: f64 = 1.5;
const SH_BDOM_TILE_MB: f64 = 105.0;
/// A Geobasis NRW DGM1 tile (Float32, LZW) and a DOM1 tile.
const NRW_DGM1_TILE_MB: f64 = 2.2;
const NRW_DOM1_TILE_MB: f64 = 3.0;
/// An LVermGeo RP DGM1 tile (Float32, LZW, 1.3-1.8 MB) and a DOM1 tile (2-3.5 MB).
const RP_DGM1_TILE_MB: f64 = 1.5;
const RP_DOM1_TILE_MB: f64 = 3.0;

/// The raster downloads of a map of `area` whose rasters come from `region`.
pub(crate) fn items(
    region: Region,
    area: &EstimateArea,
    env: &EstimateEnv<'_>,
) -> Vec<EstimateItem> {
    let (bbox, cache) = (&area.bbox_25833, env.cache);
    let provider = region.provider();
    // The tiles of a tile provider's grid (LGB 1 km, GeoSN 2 km) that touch its regions.
    let tiles = |km: u32| provider_tiles(bbox, provider, km);
    match provider {
        Provider::Lgb => lgb_items(&tiles(1), cache),
        Provider::GeoSn => geosn_items(&tiles(Product::SaxonyDtm.grid_km()), bbox, cache),
        Provider::LvermGeo => lvermgeo_items(bbox),
        Provider::Lgv => lgv_items(bbox, cache),
        Provider::Hvbg => hvbg_items(bbox, cache),
        Provider::LvermGeoSh => schleswig_holstein_items(bbox, cache),
        Provider::LaivMv => laiv_mv_items(bbox),
        Provider::GeobasisNrw => nrw_items(bbox, cache),
        Provider::Lgln => lgln_items(bbox),
        Provider::LvermGeoRp => rp_items(bbox, &area.map_bbox, cache),
    }
}

/// GeoSN: DGM1 and DOM1 tiles, and the WMS orthophoto of the area (fetched for every project).
fn geosn_items(tiles: &[TileId], bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
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
pub(crate) fn zone_32_tiles(bbox: &BoundingBox) -> Vec<TileId> {
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
fn lgv_items(bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
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
fn hvbg_items(bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
    use crate::sources::hvbg::Model;
    let list_cached = std::fs::read_dir(cache.provider_dir("hvbg").join("metadata"))
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
                    .provider_dir("hvbg")
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
fn nrw_items(bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
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
fn rp_items(bbox: &BoundingBox, map_bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
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
fn schleswig_holstein_items(bbox: &BoundingBox, cache: &CacheDir) -> Vec<EstimateItem> {
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

/// LGB: orthophoto, DTM and nDSM (or bDOM inside Berlin) per 1 km tile.
pub(crate) fn lgb_items(tiles: &[TileId], cache: &CacheDir) -> Vec<EstimateItem> {
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
