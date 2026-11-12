//! The raster backends of the ten providers behind the legacy adapters (plan C7): the fetch
//! goes to the provider's function in `project.rs`, and `derive` learns from here what a tile
//! provider keeps in the cache. The backends move into the adapter crate provider by provider.

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::fetch::FetchedTile;
use freemapper_adapter_api::geometry::provider_tiles_in;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, HeightInput, NdsmInput, OrthophotoInput, ProviderInfo, RasterArea,
    RasterBackend, RasterKind, StoredRasters,
};
use freemapper_adapter_api::records::AdapterId;

use crate::crs::BoundingBox;
use crate::fetch::TilePaths as _;
use crate::sources::lgb::{Product, ProductTiles as _, TileCurrency};
use crate::sources::region::Provider;
use crate::tiles::TileId;

/// The tile products of a tile provider.
struct TileProducts {
    /// The products every tile of the area must be recorded with.
    required: &'static [Product],
    /// The orthophoto tiles; `None`: the orthophoto is an image fetched for the area.
    orthophoto: Option<Product>,
    dtm: Product,
    dsm: Product,
}

/// The products LGB records for every tile.
pub(crate) const LGB_PRODUCTS: [Product; 2] = [Product::Orthophoto, Product::Dtm];

/// The raster backend of a provider, as the enums describe it.
pub(crate) struct LegacyBackend {
    provider: Provider,
    info: ProviderInfo,
}

impl LegacyBackend {
    pub(crate) fn new(provider: Provider) -> Self {
        let products = tile_products(provider);
        Self {
            provider,
            info: ProviderInfo {
                id: AdapterId::built_in(provider.id()),
                cache_dir: cache_dir(provider).map(str::to_string),
                terrain_source: provider.terrain_source().to_string(),
                kind: match products {
                    Some(products) => RasterKind::Tiles {
                        grid_km: products.dtm.grid_km(),
                    },
                    None => RasterKind::PerArea,
                },
            },
        }
    }
}

/// The folder `provider` keeps its downloads under in the shared cache: the name its module
/// passes to `CacheDir::provider_dir`.
fn cache_dir(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::Lgb => Some("lgb"),
        Provider::GeoSn => Some("geosn"),
        Provider::Lgv => Some("lgv"),
        Provider::Hvbg => Some("hvbg"),
        Provider::LvermGeoSh => Some("lvermgeo_sh"),
        Provider::GeobasisNrw => Some("geobasis_nrw"),
        Provider::LvermGeoRp => Some("lvermgeo_rp"),
        Provider::LaivMv => Some("laiv_mv"),
        // Everything is requested for the area and stored in the project folder.
        Provider::LvermGeo | Provider::Lgln => None,
    }
}

/// The tile products of a tile provider: required, terrain and surface model. The per-area
/// providers (LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V, Geobasis NRW, LGLN, LVermGeo RP) store
/// their height models per area and have none.
fn tile_products(provider: Provider) -> Option<TileProducts> {
    match provider {
        Provider::Lgb => Some(TileProducts {
            required: &LGB_PRODUCTS,
            orthophoto: Some(Product::Orthophoto),
            dtm: Product::Dtm,
            dsm: Product::ImageDsm,
        }),
        Provider::GeoSn => Some(TileProducts {
            required: &[Product::SaxonyDtm],
            orthophoto: None,
            dtm: Product::SaxonyDtm,
            dsm: Product::SaxonyDsm,
        }),
        Provider::LvermGeo
        | Provider::Lgv
        | Provider::Hvbg
        | Provider::LvermGeoSh
        | Provider::LaivMv
        | Provider::GeobasisNrw
        | Provider::Lgln
        | Provider::LvermGeoRp => None,
    }
}

impl RasterBackend for LegacyBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        crate::project::fetch_rasters(self.provider, env, project, bbox)
    }

    fn derive_inputs(
        &self,
        env: &DeriveEnv<'_>,
        stored: &StoredRasters<'_>,
        area: &RasterArea,
    ) -> DeriveInputs {
        match tile_products(self.provider) {
            Some(products) => tile_inputs(self.provider.id(), &products, env, stored, area),
            None => DeriveInputs::stored(),
        }
    }
}

/// What `derive` reads of a tile provider: the recorded tiles as archives in the cache.
fn tile_inputs(
    provider: &str,
    products: &TileProducts,
    env: &DeriveEnv<'_>,
    stored: &StoredRasters<'_>,
    area: &RasterArea,
) -> DeriveInputs {
    let needed = provider_tiles_in(env.regions, &area.bbox, provider, products.dtm.grid_km());
    let outdated: Vec<TileId> = needed
        .into_iter()
        .filter(|tile| {
            products.required.iter().any(|product| {
                !stored
                    .tiles
                    .iter()
                    .any(|r| r.product == *product && r.tile == tile.name())
            })
        })
        .collect();
    let tiles = |product: Product| -> Result<Vec<FetchedTile>, AdapterError> {
        let mut fetched = Vec::new();
        let mut missing = Vec::new();
        for record in stored.tiles.iter().filter(|r| r.product == product) {
            let tile = TileId::parse(&record.tile).ok_or_else(|| {
                AdapterError::ProjectFile(format!("invalid tile id {:?}", record.tile))
            })?;
            let zip = env.cache.tile_path(product, tile, record.published);
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
    DeriveInputs {
        outdated,
        orthophoto: match products.orthophoto {
            Some(product) => tiles(product).map(OrthophotoInput::Tiles),
            None => Ok(OrthophotoInput::Image),
        },
        dtm: tiles(products.dtm).map(HeightInput::Tiles),
        dsm: stored
            .tiles
            .iter()
            .any(|r| r.product == products.dsm)
            .then(|| tiles(products.dsm).map(HeightInput::Tiles)),
        ndsm: stored
            .ndsm_tiles
            .iter()
            .map(|record| NdsmInput {
                tile: record.tile.clone(),
                // The tile comes from the project file: only a valid tile id names a cache file.
                path: TileId::parse(&record.tile)
                    .map(|tile| env.cache.ndsm_path(tile, record.published)),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::region::golden::legacy_facts;

    #[test]
    fn provider_info_matches_g1() {
        for facts in legacy_facts() {
            let provider = crate::sources::region::Region::from_id(&facts.id)
                .unwrap()
                .provider();
            let info = LegacyBackend::new(provider).info;
            assert_eq!(info.id.as_str(), facts.provider, "{}", facts.id);
            assert_eq!(info.cache_dir, facts.cache_dir, "{}", facts.id);
            assert_eq!(info.terrain_source, facts.terrain_source, "{}", facts.id);
            assert_eq!(
                info.kind == RasterKind::PerArea,
                facts.per_area,
                "{}",
                facts.id
            );
        }
        assert_eq!(
            LegacyBackend::new(Provider::Lgb).info.kind,
            RasterKind::Tiles { grid_km: 1 }
        );
        assert_eq!(
            LegacyBackend::new(Provider::GeoSn).info.kind,
            RasterKind::Tiles { grid_km: 2 }
        );
    }
}
