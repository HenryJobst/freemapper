//! The ALKIS services of the eleven states, as the two matches of `sources/alkis.rs` named them
//! (plan C6): which service serves which state, in which CRS, with which fallback. The readers
//! are those of `freemapper-sources`; Berlin keeps its own (Geoportal Berlin WFS).

use std::borrow::Cow;

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis, WfsAlkis};

use crate::crs::Crs;
use crate::sources::alkis::{
    BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT,
};
use crate::sources::region::Region;
use crate::sources::{alkis_wfs, berlin, lgv};

/// The ALKIS service of `region`.
pub(crate) fn source(region: Region) -> Box<dyn AlkisSource> {
    let wfs =
        |service: alkis_wfs::WfsService| -> Box<dyn AlkisSource> { Box::new(WfsAlkis { service }) };
    match region {
        // LGB's OGC API (dl-de/by-2-0), which sets the schema and the spellings of all others.
        Region::Brandenburg => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: None,
            normalise: Normaliser::Lgb,
        }),
        Region::Berlin => Box::new(BerlinAlkis),
        Region::Saxony => wfs(alkis_wfs::GEOSN),
        Region::SaxonyAnhalt => wfs(alkis_wfs::LVERMGEO),
        Region::Hamburg => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(lgv::BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(lgv::LAND_USE_ENDPOINT),
            crs: alkis_wfs::LGV.crs,
            fallback: Some(alkis_wfs::LGV),
            normalise: Normaliser::HamburgAscii,
        }),
        Region::Hessen => wfs(alkis_wfs::HVBG),
        Region::SchleswigHolstein => wfs(alkis_wfs::LVERMGEO_SH),
        Region::MecklenburgVorpommern => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(MV_BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(MV_LAND_USE_ENDPOINT),
            crs: alkis_wfs::LAIV_MV.crs,
            fallback: Some(alkis_wfs::LAIV_MV),
            normalise: Normaliser::Lgb,
        }),
        Region::NordrheinWestfalen => wfs(alkis_wfs::GEOBASIS_NRW),
        Region::Niedersachsen => wfs(alkis_wfs::LGLN),
        Region::RheinlandPfalz => wfs(alkis_wfs::LVERMGEO_RP),
    }
}

/// Berlin: the Geoportal Berlin WFS, normalised to the LGB schema by `sources/berlin.rs`.
struct BerlinAlkis;

impl AlkisSource for BerlinAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        Ok(AlkisPart {
            features: berlin::fetch_buildings(env.fetcher, &area.bbox_for(Crs::Utm33))?,
            crs: Crs::Utm33,
        })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        Ok(AlkisPart {
            features: berlin::fetch_land_use(env.fetcher, &area.bbox_for(Crs::Utm33))?,
            crs: Crs::Utm33,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every state asks its own service first, for the buildings and for the land use, in the
    /// CRS of that service. The table is the two matches `sources/alkis.rs` had.
    #[test]
    fn every_state_asks_its_service() {
        use crate::fetch::FetchMode;
        use crate::fetch::test_support::MapFetcher;
        use crate::sources::alkis::FetchArea;
        use freemapper_adapter_api::cache::CacheDir;

        let wfs = |service: alkis_wfs::WfsService| {
            (
                service.url.to_string(),
                service.url.to_string(),
                service.crs,
            )
        };
        let api = |buildings: &str, land_use: &str, crs: Crs| {
            (buildings.to_string(), land_use.to_string(), crs)
        };
        let expected = |region: Region| match region {
            Region::Brandenburg => api(BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, Crs::Utm33),
            Region::Berlin => api(
                berlin::BUILDINGS_SERVICE,
                berlin::LAND_USE_SERVICE,
                Crs::Utm33,
            ),
            Region::Saxony => wfs(alkis_wfs::GEOSN),
            Region::SaxonyAnhalt => wfs(alkis_wfs::LVERMGEO),
            Region::Hamburg => api(lgv::BUILDINGS_ENDPOINT, lgv::LAND_USE_ENDPOINT, Crs::Utm32),
            Region::Hessen => wfs(alkis_wfs::HVBG),
            Region::SchleswigHolstein => wfs(alkis_wfs::LVERMGEO_SH),
            Region::MecklenburgVorpommern => {
                api(MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT, Crs::Utm33)
            }
            Region::NordrheinWestfalen => wfs(alkis_wfs::GEOBASIS_NRW),
            Region::Niedersachsen => wfs(alkis_wfs::LGLN),
            Region::RheinlandPfalz => wfs(alkis_wfs::LVERMGEO_RP),
        };
        // Kassel, a place both UTM zones can hold.
        let area = FetchArea::new(
            crate::crs::BoundingBox {
                min_e: 100_000.0,
                min_n: 5_700_000.0,
                max_e: 100_200.0,
                max_n: 5_700_200.0,
            },
            Crs::Utm33,
        );
        let cache = CacheDir::new("no-cache".into());
        for region in Region::ALL {
            let (buildings, land_use, crs) = expected(region);
            let source = source(region);
            // Nothing answers: the first request shows where the source asks.
            let first_request = |land: bool| {
                let fetcher = MapFetcher::default();
                let mut env = AdapterEnv {
                    fetcher: &fetcher,
                    cache: &cache,
                    mode: FetchMode::Online,
                    today: crate::clock::today(),
                    skip_surface: false,
                    progress: &mut |_| {},
                };
                let failed = if land {
                    source.land_use(&mut env, &area)
                } else {
                    source.buildings(&mut env, &area)
                };
                assert!(failed.is_err(), "{region:?}");
                fetcher.log.borrow()[0].clone()
            };
            for (url, endpoint) in [
                (first_request(false), buildings),
                (first_request(true), land_use),
            ] {
                assert!(url.starts_with(&endpoint), "{region:?}: {url}");
                assert!(
                    url.contains(&format!("{}", crs.epsg())),
                    "{region:?} asks in EPSG:{}: {url}",
                    crs.epsg()
                );
            }
        }
    }

    /// The fallback of an OGC API is asked with the box and the CRS of the API, so the two must
    /// agree: Hamburg in EPSG:25832, Mecklenburg-Vorpommern in EPSG:25833.
    #[test]
    fn the_fallback_services_answer_in_the_crs_of_their_api() {
        assert_eq!(alkis_wfs::LGV.crs, Crs::Utm32);
        assert_eq!(alkis_wfs::LAIV_MV.crs, Crs::Utm33);
    }
}
