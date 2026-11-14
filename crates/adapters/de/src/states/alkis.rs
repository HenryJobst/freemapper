//! The ALKIS services of the states: which service serves which state, in which CRS, with which
//! fallback. The readers are those of `freemapper-sources`; Berlin keeps its own (Geoportal
//! Berlin WFS), and so does Baden-Württemberg (the views of the LGL). Bremen has no simplified
//! ALKIS: its full ALKIS is read as one. Bavaria has no service at all: its buildings and its
//! land use are read out of files.

use std::borrow::Cow;

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis, WfsAlkis};

use crate::crs::Crs;
use crate::sources::alkis_wfs;
use crate::states::State;
use crate::{berlin, geobremen, ldbv_alkis, lgl_bw, lgv};

/// LGB's OGC API, buildings and structures (`gebaeude_bauwerk`).
pub const BUILDINGS_ENDPOINT: &str =
    "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/gebaeude_bauwerk/items";
/// LAiV M-V's OGC API (the same LGB schema and spellings, EPSG:25833).
pub const MV_BUILDINGS_ENDPOINT: &str =
    "https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/gebaeude_bauwerk/items";
pub const MV_LAND_USE_ENDPOINT: &str =
    "https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/nutzung_flurstueck/items";
/// Land use per parcel part (`nutzung_flurstueck`).
pub const LAND_USE_ENDPOINT: &str =
    "https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/nutzung_flurstueck/items";

/// The ALKIS service of `state`.
pub fn source(state: State) -> Box<dyn AlkisSource> {
    let wfs =
        |service: alkis_wfs::WfsService| -> Box<dyn AlkisSource> { Box::new(WfsAlkis { service }) };
    match state {
        // LGB's OGC API (dl-de/by-2-0), which sets the schema and the spellings of all others.
        State::Brandenburg => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: None,
            normalise: Normaliser::Lgb,
        }),
        State::Berlin => Box::new(BerlinAlkis),
        State::Saxony => wfs(alkis_wfs::GEOSN),
        State::SaxonyAnhalt => wfs(alkis_wfs::LVERMGEO),
        State::Hamburg => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(lgv::BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(lgv::LAND_USE_ENDPOINT),
            crs: alkis_wfs::LGV.crs,
            fallback: Some(alkis_wfs::LGV),
            normalise: Normaliser::HamburgAscii,
        }),
        State::Hessen => wfs(alkis_wfs::HVBG),
        State::SchleswigHolstein => wfs(alkis_wfs::LVERMGEO_SH),
        State::MecklenburgVorpommern => Box::new(OgcApiAlkis {
            buildings: Cow::Borrowed(MV_BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(MV_LAND_USE_ENDPOINT),
            crs: alkis_wfs::LAIV_MV.crs,
            fallback: Some(alkis_wfs::LAIV_MV),
            normalise: Normaliser::Lgb,
        }),
        State::NordrheinWestfalen => wfs(alkis_wfs::GEOBASIS_NRW),
        State::Niedersachsen => wfs(alkis_wfs::LGLN),
        State::RheinlandPfalz => wfs(alkis_wfs::LVERMGEO_RP),
        State::Saarland => wfs(crate::lvgl::ALKIS),
        // The views of the LGL are not the simplified schema; the source turns them into it.
        State::BadenWuerttemberg => Box::new(lgl_bw::LglBwAlkis),
        // The state serves the full ALKIS only; the source turns it into the simplified schema.
        State::Bremen => Box::new(geobremen::alkis()),
        // The state has no service for its cadastre; the source reads the files it publishes.
        State::Bayern => Box::new(ldbv_alkis::LdbvAlkis {
            kreise: ldbv_alkis::built_in_kreise(),
        }),
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
    /// CRS of that service.
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
        let expected = |state: State| match state {
            State::Brandenburg => api(BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, Crs::Utm33),
            State::Berlin => api(
                berlin::BUILDINGS_SERVICE,
                berlin::LAND_USE_SERVICE,
                Crs::Utm33,
            ),
            State::Saxony => wfs(alkis_wfs::GEOSN),
            State::SaxonyAnhalt => wfs(alkis_wfs::LVERMGEO),
            State::Hamburg => api(lgv::BUILDINGS_ENDPOINT, lgv::LAND_USE_ENDPOINT, Crs::Utm32),
            State::Hessen => wfs(alkis_wfs::HVBG),
            State::SchleswigHolstein => wfs(alkis_wfs::LVERMGEO_SH),
            State::MecklenburgVorpommern => {
                api(MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT, Crs::Utm33)
            }
            State::NordrheinWestfalen => wfs(alkis_wfs::GEOBASIS_NRW),
            State::Niedersachsen => wfs(alkis_wfs::LGLN),
            State::RheinlandPfalz => wfs(alkis_wfs::LVERMGEO_RP),
            State::Saarland => wfs(crate::lvgl::ALKIS),
            State::BadenWuerttemberg => wfs(lgl_bw::ALKIS),
            State::Bremen => api(geobremen::ALKIS_URL, geobremen::ALKIS_URL, Crs::Utm32),
            State::Bayern => api(crate::ldbv::FILES, crate::ldbv::FILES, Crs::Utm32),
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
        // Bremen streams its answers through a file in the cache.
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let mut registry = freemapper_adapter_api::registry::Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        for state in State::ALL {
            // Bavaria has no service: it asks whether its files changed before it fetches
            // them, and a file that is not there is no error for the buildings. Its own tests
            // show the addresses (`ldbv_alkis`).
            if state == State::Bayern {
                continue;
            }
            let (buildings, land_use, crs) = expected(state);
            let source = source(state);
            // Nothing answers: the first request shows where the source asks.
            let first_request = |land: bool| {
                let fetcher = MapFetcher::default();
                let mut env = AdapterEnv {
                    fetcher: &fetcher,
                    regions: &registry,
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
                assert!(failed.is_err(), "{state:?}");
                fetcher.log.borrow()[0].clone()
            };
            for (url, endpoint) in [
                (first_request(false), buildings),
                (first_request(true), land_use),
            ] {
                assert!(url.starts_with(&endpoint), "{state:?}: {url}");
                assert!(
                    url.contains(&format!("{}", crs.epsg())),
                    "{state:?} asks in EPSG:{}: {url}",
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
