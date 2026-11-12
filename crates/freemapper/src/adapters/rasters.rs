//! The raster backend of each provider behind the legacy adapters. All ten live in the adapter
//! crate (plan C7 to C12); this table names them by the provider enum until the enum goes.

use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapters_de::geobasis_nrw::GeobasisNrwBackend;
use freemapper_adapters_de::geosn::GeosnBackend;
use freemapper_adapters_de::hvbg::HvbgBackend;
use freemapper_adapters_de::laiv_mv::LaivMvBackend;
use freemapper_adapters_de::lgb::LgbBackend;
use freemapper_adapters_de::lgln::LglnBackend;
use freemapper_adapters_de::lgv::LgvBackend;
use freemapper_adapters_de::lvermgeo::LvermgeoBackend;
use freemapper_adapters_de::lvermgeo_rp::LvermgeoRpBackend;
use freemapper_adapters_de::lvermgeo_sh::LvermgeoShBackend;

use crate::sources::region::Provider;

/// The raster backend of `provider`.
pub(crate) fn backend(provider: Provider) -> Box<dyn RasterBackend> {
    match provider {
        Provider::Lgb => Box::new(LgbBackend::new()),
        Provider::GeoSn => Box::new(GeosnBackend::new()),
        Provider::LvermGeo => Box::new(LvermgeoBackend::new()),
        Provider::Lgv => Box::new(LgvBackend::new()),
        Provider::Hvbg => Box::new(HvbgBackend::new()),
        Provider::LvermGeoSh => Box::new(LvermgeoShBackend::new()),
        Provider::LaivMv => Box::new(LaivMvBackend::new()),
        Provider::GeobasisNrw => Box::new(GeobasisNrwBackend::new()),
        Provider::Lgln => Box::new(LglnBackend::new()),
        Provider::LvermGeoRp => Box::new(LvermgeoRpBackend::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::region::golden::legacy_facts;
    use freemapper_adapter_api::rasters::RasterKind;

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
