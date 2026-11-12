//! The raster backends behind the legacy adapters (plan C7). A provider whose backend lives in
//! the adapter crate gets that one. For the others the fetch goes to the provider's function in
//! `project.rs`; they move into the adapter crate provider by provider.

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, ProviderInfo, RasterArea, RasterBackend, RasterKind, StoredRasters,
};
use freemapper_adapter_api::records::AdapterId;
use freemapper_adapters_de::lgb::LgbBackend;
use freemapper_adapters_de::tile_fetch::{TileProducts, tile_inputs};

use crate::crs::BoundingBox;
use crate::project;
use crate::sources::lgb::{Product, ProductTiles as _};
use crate::sources::region::Provider;

/// A raster fetch: the signature of [`RasterBackend::fetch`] without the backend.
type RasterFetch =
    fn(&mut AdapterEnv<'_>, &mut dyn RasterProject, &BoundingBox) -> Result<(), AdapterError>;

/// The raster backend of `provider`.
pub(crate) fn backend(provider: Provider) -> Box<dyn RasterBackend> {
    let fetch: RasterFetch = match provider {
        Provider::Lgb => return Box::new(LgbBackend::new()),
        Provider::GeoSn => project::fetch_geosn_rasters,
        Provider::LvermGeo => project::fetch_lvermgeo_rasters,
        Provider::Lgv => project::fetch_lgv_rasters,
        Provider::Hvbg => project::fetch_hvbg_rasters,
        Provider::LvermGeoSh => project::fetch_sh_rasters,
        Provider::LaivMv => project::fetch_mv_rasters,
        Provider::GeobasisNrw => project::fetch_nrw_rasters,
        Provider::Lgln => project::fetch_lgln_rasters,
        Provider::LvermGeoRp => project::fetch_rp_rasters,
    };
    Box::new(LegacyBackend::new(provider, fetch))
}

/// The raster backend of a provider, as the enums describe it.
struct LegacyBackend {
    provider: Provider,
    fetch: RasterFetch,
    info: ProviderInfo,
}

impl LegacyBackend {
    fn new(provider: Provider, fetch: RasterFetch) -> Self {
        Self {
            provider,
            fetch,
            info: ProviderInfo {
                id: AdapterId::built_in(provider.id()),
                cache_dir: cache_dir(provider).map(str::to_string),
                terrain_source: provider.terrain_source().to_string(),
                kind: match tile_products(provider) {
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

/// The tile products of a tile provider that is still a legacy backend: required, terrain and
/// surface model. The per-area providers (LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V, Geobasis
/// NRW, LGLN, LVermGeo RP) store their height models per area and have none.
fn tile_products(provider: Provider) -> Option<TileProducts> {
    match provider {
        Provider::GeoSn => Some(TileProducts {
            required: &[Product::SaxonyDtm],
            orthophoto: None,
            dtm: Product::SaxonyDtm,
            dsm: Product::SaxonyDsm,
        }),
        Provider::Lgb
        | Provider::LvermGeo
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
        (self.fetch)(env, project, bbox)
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
            let backend = backend(provider);
            let info = backend.provider();
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
            backend(Provider::Lgb).provider().kind,
            RasterKind::Tiles { grid_km: 1 }
        );
        assert_eq!(
            backend(Provider::GeoSn).provider().kind,
            RasterKind::Tiles { grid_km: 2 }
        );
    }
}
