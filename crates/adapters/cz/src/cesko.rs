// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapter `cesko`: Czechia as one region. Its boundary is the state border of OpenStreetMap
//! (relation 51684, ODbL, simplified to about 40 m); its one OSM extract is the clipping polygon
//! of Geofabrik for the Czech Republic, which reaches past the border and only chooses the
//! extract. Both are embedded as GeoJSON in WGS 84 and projected to EPSG:25833 once on first use.
//! It gives the rasters of ČÚZK (terrain model, surface model and orthophoto,
//! [`crate::rasters::CuzkRasters`]) and the cadastre ([`crate::cadastre`]: the buildings of RÚIAN and
//! the land use of ZABAGED). The adapter asks the host of the ČÚZK services and no other ([`HOSTS`]).

use std::collections::BTreeMap;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, Remedy};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::info::{
    Country, Coverage, Declared, Names, OsmExtractInfo, PreviewInfo, RegionInfo,
};
use freemapper_adapter_api::policy::{Owner, PolicyFetcher, UrlPolicy};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, ProviderInfo, RasterArea, RasterBackend, StoredRasters,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use freemapper_adapter_api::registry::{DuplicateAdapter, Registry};
use freemapper_georef::crs::{BoundingBox, Crs, LatLon};
use geo::MapCoords;
use geo_types::MultiPolygon;

use crate::cadastre::CzCadastre;
use crate::rasters::{CuzkRasters, HOST, PROVIDER};

/// The id of the adapter.
pub const CESKO: &str = "cesko";

/// The credit line of the data of ČÚZK (terrain model, surface model, orthophoto), and the text
/// of the WMS the area page shows. The licence is CC BY 4.0 (open data of ČÚZK since 1 July 2023).
pub const CREDIT: &str = "© ČÚZK (CC BY 4.0)";

/// The hosts the adapter asks besides what a person allowed: the image services, the orthophoto
/// and the vector services of ČÚZK. The OSM extract comes through the core
/// hosts, and the area page asks the WMS of the same host itself, in the browser.
pub const HOSTS: [&str; 1] = [HOST];

/// The state border of Czechia (OSM relation 51684), WGS 84.
const CZECHIA_BOUNDARY: &str = include_str!("../data/czechia.geojson");

/// The clipping polygon of the Geofabrik extract of the Czech Republic, WGS 84.
const EXTRACT_BOUNDARY: &str = include_str!("../data/extract.geojson");

/// The extract of Geofabrik: the path without `-latest.osm.pbf`, and its size in MB at the time
/// of writing (checked 26 November 2026).
const EXTRACT: (&str, f64) = ("europe/czech-republic/czech-republic", 910.0);

/// The first geometry of an embedded GeoJSON feature in WGS 84, projected to EPSG:25833.
fn project(text: &str) -> MultiPolygon<f64> {
    let geojson: geojson::GeoJson = text.parse().expect("embedded boundary is GeoJSON");
    let geojson::GeoJson::Feature(feature) = geojson else {
        panic!("embedded boundary is a feature");
    };
    let geometry = feature.geometry.expect("embedded boundary has a geometry");
    let wgs84 =
        MultiPolygon::try_from(geometry.value).expect("embedded boundary is a MultiPolygon");
    wgs84.map_coords(|c| {
        let p = Crs::Utm33.forward(LatLon { lat: c.y, lon: c.x });
        geo_types::Coord {
            x: p.easting,
            y: p.northing,
        }
    })
}

/// Czechia as one region.
pub struct Cesko {
    info: RegionInfo,
    declared: Declared,
    policy: UrlPolicy,
    rasters: GuardedRasters,
    cadastre: CzCadastre,
    boundary: MultiPolygon<f64>,
    extract: MultiPolygon<f64>,
}

impl Cesko {
    /// The adapter, held to the host of ČÚZK ([`HOSTS`]) and to `allowed`, the hosts a person
    /// allowed for it (`freemapper adapters allow-host cesko <host>`). The OSM extract comes
    /// from the engine, which asks Geofabrik itself.
    pub fn new(allowed: &[String]) -> Self {
        let id = AdapterId::built_in(CESKO);
        let mut policy = UrlPolicy::new(HOSTS).expect("the host of ČÚZK is a DNS name");
        for host in allowed {
            if let Ok(wider) = policy.clone().with_hosts([host.as_str()]) {
                policy = wider;
            }
        }
        let policy = policy.owned_by(Owner {
            adapter: CESKO.into(),
            name: "Czechia".into(),
            remedy: Remedy::AllowHost,
        });
        Self {
            declared: Declared {
                revision: 1,
                hosts: HOSTS.iter().map(ToString::to_string).collect(),
                last_verified: chrono::NaiveDate::from_ymd_opt(2026, 11, 26),
            },
            rasters: GuardedRasters {
                inner: CuzkRasters::new(),
                policy: policy.clone(),
            },
            cadastre: CzCadastre {
                policy: policy.clone(),
            },
            policy,
            info: RegionInfo {
                id: id.clone(),
                // The rasters are those of ČÚZK ([`CuzkRasters`]).
                provider: AdapterId::built_in(PROVIDER),
                names: Names {
                    en: "Czechia".into(),
                    de: "Tschechien".into(),
                    pl: Some("Czechy".into()),
                    cs: Some("Česko".into()),
                },
                // Czechia lies between 12° and 19° E; one zone serves it (the east is past the
                // border of zone 33 by less than a degree, where the scale is still 1:1.0004).
                crs: Crs::Utm33,
                country: Country::Cz,
                // The rasters and the cadastre of ČÚZK: RÚIAN and ZABAGED (ADR 0197).
                coverage: Coverage::Full,
                object_id_prefix: None,
                osm: vec![OsmExtractInfo {
                    geofabrik_path: EXTRACT.0.to_string(),
                    megabytes: EXTRACT.1,
                    label: "Czechia".into(),
                    prefix: "czechia_".into(),
                }],
                credit_short: None,
                // The orthophoto the browser shows on the area page (ČÚZK, open data).
                preview: Some(PreviewInfo {
                    wms: crate::rasters::WMS.url.to_string(),
                    layer: crate::rasters::WMS.layer.to_string(),
                    format: "image/jpeg".into(),
                    params: Vec::new(),
                    attribution: CREDIT.into(),
                    agency: "ČÚZK".into(),
                }),
            },
            boundary: project(CZECHIA_BOUNDARY),
            extract: project(EXTRACT_BOUNDARY),
        }
    }
}

impl RegionAdapter for Cesko {
    fn info(&self) -> &RegionInfo {
        &self.info
    }

    fn declared(&self) -> Option<&Declared> {
        Some(&self.declared)
    }

    fn policy(&self) -> Option<&UrlPolicy> {
        Some(&self.policy)
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        &self.boundary
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        (index == 0).then_some(&self.extract)
    }

    /// The credit of ČÚZK for a project whose rasters came from it and holds a terrain model or
    /// an orthophoto, or that has buildings or land use of the cadastre (the same office and the
    /// same credit line).
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        let holds = used.terrain.is_some() || used.orthophoto.is_some();
        let cadastre = used.alkis_fetches(CESKO).next().is_some();
        if (used.rasters_from(PROVIDER) && holds) || cadastre {
            vec![CreditLine::new(CREDIT)]
        } else {
            Vec::new()
        }
    }

    /// The buildings of RÚIAN and the land use of ZABAGED ([`crate::cadastre`]).
    fn alkis(&self) -> Option<&dyn AlkisSource> {
        Some(&self.cadastre)
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        Some(&self.rasters)
    }

    /// The estimate of [`CuzkRasters`]: the engine adds the item of the cadastre itself.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.rasters.estimate(area, env)
    }

    fn crs_for(&self, _centre: LatLon) -> Crs {
        Crs::Utm33
    }
}

/// The rasters of ČÚZK with every request held to the hosts of the adapter's policy: the engine
/// hands the backend a fetcher that knows no adapter, so the backend is wrapped here, as the
/// manifest crate wraps the backends of the states.
struct GuardedRasters {
    inner: CuzkRasters,
    policy: UrlPolicy,
}

impl RasterBackend for GuardedRasters {
    fn provider(&self) -> &ProviderInfo {
        self.inner.provider()
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let fetcher = PolicyFetcher::new(env.fetcher, &self.policy);
        let mut guarded = AdapterEnv {
            fetcher: &fetcher,
            regions: env.regions,
            cache: env.cache,
            mode: env.mode,
            today: env.today,
            skip_surface: env.skip_surface,
            progress: &mut *env.progress,
        };
        self.inner.fetch(&mut guarded, project, bbox)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.inner.estimate(area, env)
    }

    fn derive_inputs(
        &self,
        env: &DeriveEnv<'_>,
        stored: &StoredRasters<'_>,
        area: &RasterArea,
    ) -> DeriveInputs {
        self.inner.derive_inputs(env, stored, area)
    }
}

/// Adds the adapter of Czechia to `registry`, with the hosts a person allowed for it, by adapter
/// id (as the states take them).
pub fn register_with_hosts(
    registry: &mut Registry,
    extra: &BTreeMap<String, Vec<String>>,
) -> Result<(), DuplicateAdapter> {
    let allowed = extra.get(CESKO).map_or(&[][..], Vec::as_slice);
    registry.add(Arc::new(Cesko::new(allowed)))
}

/// [`register_with_hosts`] without a host allowed.
pub fn register(registry: &mut Registry) -> Result<(), DuplicateAdapter> {
    register_with_hosts(registry, &BTreeMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use geo::Contains;
    use geo_types::Point;

    fn utm(lat: f64, lon: f64) -> Point<f64> {
        let p = Crs::Utm33.forward(LatLon { lat, lon });
        Point::new(p.easting, p.northing)
    }

    #[test]
    fn the_border_holds_czech_places_and_leaves_the_neighbours_out() {
        let cesko = Cesko::new(&[]);
        let inside = [
            ("Praha", 50.087, 14.421),
            ("Brno", 49.195, 16.608),
            ("Ostrava", 49.835, 18.292),
            ("Cheb", 50.080, 12.374),
        ];
        for (name, lat, lon) in inside {
            assert!(cesko.boundary().contains(&utm(lat, lon)), "{name}");
        }
        let outside = [
            ("Dresden", 51.050, 13.738),
            ("Wien", 48.208, 16.373),
            ("Wrocław", 51.108, 17.039),
            ("Bratislava", 48.149, 17.107),
        ];
        for (name, lat, lon) in outside {
            assert!(!cesko.boundary().contains(&utm(lat, lon)), "{name}");
        }
    }

    #[test]
    fn the_extract_reaches_past_the_border() {
        let cesko = Cesko::new(&[]);
        let extract = cesko.extract_boundary(0).expect("one extract");
        assert!(extract.contains(&utm(50.087, 14.421)));
        assert!(cesko.extract_boundary(1).is_none());
        assert_eq!(cesko.info().osm.len(), 1);
        assert_eq!(
            cesko.info().osm[0].url(),
            "https://download.geofabrik.de/europe/czech-republic/czech-republic-latest.osm.pbf"
        );
    }

    #[test]
    fn the_region_is_named_in_four_languages() {
        let names = Cesko::new(&[]).info().names.clone();
        assert_eq!(names.in_lang("en"), "Czechia");
        assert_eq!(names.in_lang("de"), "Tschechien");
        assert_eq!(names.in_lang("cs"), "Česko");
        assert_eq!(names.in_lang("pl"), "Czechy");
    }

    #[test]
    fn the_adapter_asks_the_host_of_cuzk_and_those_a_person_allowed() {
        let plain = Cesko::new(&[]);
        assert_eq!(plain.declared().unwrap().hosts, [HOST]);
        let wider = Cesko::new(&["atom.cuzk.gov.cz".to_string()]);
        let policy = wider.policy().unwrap();
        assert!(policy.check_url("https://atom.cuzk.gov.cz/x").is_ok());
        assert!(
            plain
                .policy()
                .unwrap()
                .check_url("https://atom.cuzk.gov.cz/x")
                .is_err()
        );
    }

    #[test]
    fn the_credit_names_cuzk_and_its_licence_where_its_rasters_are_used() {
        assert!(CREDIT.contains("ČÚZK") && CREDIT.contains("CC BY 4.0"));
    }

    /// The guarded rasters give the estimate of the backend: the three downloads of ČÚZK.
    #[test]
    fn the_guarded_rasters_give_the_estimate_of_the_backend() {
        let cesko = Cesko::new(&[]);
        let bbox = BoundingBox {
            min_e: 458_000.0,
            min_n: 5_548_000.0,
            max_e: 459_000.0,
            max_n: 5_549_000.0,
        };
        let cache = CacheDir::new(std::env::temp_dir().join("freemapper-cz-guarded-estimate"));
        let items = cesko.estimate(
            &EstimateArea {
                bbox_25833: bbox,
                map_bbox: bbox,
                today: NaiveDate::from_ymd_opt(2026, 11, 26).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(items.len(), 3, "{items:?}");
    }
}
