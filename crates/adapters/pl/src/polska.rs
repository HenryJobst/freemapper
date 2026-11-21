// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapter `polska`: Poland as one region. Its boundary is the state border of OpenStreetMap
//! (relation 49715, ODbL, simplified to about 40 m). The boundaries of its OSM extracts are the
//! clipping polygons Geofabrik publishes (from `index-v1.json`); they reach past the border and
//! only choose the extract. All are embedded as GeoJSON in WGS 84 and projected to EPSG:25833
//! once on first use. Stage 1 has the OSM extracts only: no rasters, no ALKIS
//! (`Coverage::OsmOnly`).

use std::collections::BTreeMap;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::error::Remedy;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::info::{
    Country, Coverage, Declared, Names, OsmExtractInfo, PreviewInfo, RegionInfo,
};
use freemapper_adapter_api::policy::{Owner, UrlPolicy};
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use freemapper_adapter_api::registry::{DuplicateAdapter, Registry};
use freemapper_georef::crs::{Crs, LatLon};
use geo::MapCoords;
use geo_types::MultiPolygon;

/// The id of the adapter.
pub const POLSKA: &str = "polska";

/// The state border of Poland (OSM relation 49715), WGS 84.
const POLAND_BOUNDARY: &str = include_str!("../data/poland.geojson");

/// The voivodeships as Geofabrik extracts them: id, English name, size of the extract in MB at the
/// time of writing, boundary (WGS 84). The extract of `<id>` lies at
/// `europe/poland/<id>-latest.osm.pbf`, which Geofabrik redirects to the folder of the voivodeship.
const VOIVODESHIPS: [(&str, &str, f64, &str); 16] = [
    (
        "dolnoslaskie",
        "Lower Silesian Voivodeship",
        182.5,
        include_str!("../data/voivodeships/dolnoslaskie.geojson"),
    ),
    (
        "kujawsko-pomorskie",
        "Kuyavian-Pomeranian Voivodeship",
        105.8,
        include_str!("../data/voivodeships/kujawsko-pomorskie.geojson"),
    ),
    (
        "lodzkie",
        "Łódź Voivodeship",
        132.9,
        include_str!("../data/voivodeships/lodzkie.geojson"),
    ),
    (
        "lubelskie",
        "Lublin Voivodeship",
        139.8,
        include_str!("../data/voivodeships/lubelskie.geojson"),
    ),
    (
        "lubuskie",
        "Lubusz Voivodeship",
        63.3,
        include_str!("../data/voivodeships/lubuskie.geojson"),
    ),
    (
        "malopolskie",
        "Lesser Poland Voivodeship",
        203.9,
        include_str!("../data/voivodeships/malopolskie.geojson"),
    ),
    (
        "mazowieckie",
        "Mazovian Voivodeship",
        302.5,
        include_str!("../data/voivodeships/mazowieckie.geojson"),
    ),
    (
        "opolskie",
        "Opole Voivodeship",
        55.1,
        include_str!("../data/voivodeships/opolskie.geojson"),
    ),
    (
        "podkarpackie",
        "Subcarpathian Voivodeship",
        155.6,
        include_str!("../data/voivodeships/podkarpackie.geojson"),
    ),
    (
        "podlaskie",
        "Podlaskie Voivodeship",
        78.0,
        include_str!("../data/voivodeships/podlaskie.geojson"),
    ),
    (
        "pomorskie",
        "Pomeranian Voivodeship",
        118.7,
        include_str!("../data/voivodeships/pomorskie.geojson"),
    ),
    (
        "slaskie",
        "Silesian Voivodeship",
        187.0,
        include_str!("../data/voivodeships/slaskie.geojson"),
    ),
    (
        "swietokrzyskie",
        "Świętokrzyskie Voivodeship",
        88.8,
        include_str!("../data/voivodeships/swietokrzyskie.geojson"),
    ),
    (
        "warminsko-mazurskie",
        "Warmian-Masurian Voivodeship",
        85.8,
        include_str!("../data/voivodeships/warminsko-mazurskie.geojson"),
    ),
    (
        "wielkopolskie",
        "Greater Poland Voivodeship",
        159.9,
        include_str!("../data/voivodeships/wielkopolskie.geojson"),
    ),
    (
        "zachodniopomorskie",
        "West Pomeranian Voivodeship",
        98.1,
        include_str!("../data/voivodeships/zachodniopomorskie.geojson"),
    ),
];

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

/// Poland as one region.
pub struct Polska {
    info: RegionInfo,
    declared: Declared,
    policy: UrlPolicy,
    boundary: MultiPolygon<f64>,
    extracts: Vec<MultiPolygon<f64>>,
}

impl Polska {
    /// The adapter, held to no host of its own besides `allowed`, the hosts a person allowed for
    /// it (`freemapper adapters allow-host polska <host>`). In stage 1 it asks no service: the
    /// OSM extracts come from the engine, which asks Geofabrik itself.
    pub fn new(allowed: &[String]) -> Self {
        let osm = VOIVODESHIPS
            .iter()
            .map(|(id, label, megabytes, _)| OsmExtractInfo {
                geofabrik_path: format!("europe/poland/{id}"),
                megabytes: *megabytes,
                label: (*label).to_string(),
                prefix: format!("{id}_"),
            })
            .collect();
        let id = AdapterId::built_in(POLSKA);
        let mut policy = UrlPolicy::new([]).expect("no host is a valid policy");
        for host in allowed {
            if let Ok(wider) = policy.clone().with_hosts([host.as_str()]) {
                policy = wider;
            }
        }
        let policy = policy.owned_by(Owner {
            adapter: POLSKA.into(),
            name: "Poland".into(),
            remedy: Remedy::AllowHost,
        });
        Self {
            declared: Declared {
                revision: 1,
                hosts: Vec::new(),
                last_verified: chrono::NaiveDate::from_ymd_opt(2026, 11, 20),
            },
            policy,
            info: RegionInfo {
                id: id.clone(),
                provider: id,
                names: Names {
                    en: "Poland".into(),
                    de: "Polen".into(),
                    pl: Some("Polska".into()),
                },
                // The zone of a map is chosen by its centre ([`RegionAdapter::crs_for`]).
                crs: Crs::Utm33,
                country: Country::Pl,
                coverage: Coverage::OsmOnly,
                object_id_prefix: None,
                osm,
                credit_short: None,
                // The aerial photo the browser shows on the area page (GUGiK, no fee; the service
                // forbids automated harvesting, which a map the person looks at is not).
                preview: Some(PreviewInfo {
                    wms: "https://mapy.geoportal.gov.pl/wss/service/PZGIK/ORTO/WMS/StandardResolution"
                        .into(),
                    layer: "Raster".into(),
                    format: "image/jpeg".into(),
                    params: Vec::new(),
                    attribution: "© GUGiK, geoportal.gov.pl".into(),
                    agency: "GUGiK".into(),
                }),
            },
            boundary: project(POLAND_BOUNDARY),
            extracts: VOIVODESHIPS
                .iter()
                .map(|(.., text)| project(text))
                .collect(),
        }
    }
}

impl RegionAdapter for Polska {
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
        self.extracts.get(index)
    }

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        None
    }

    fn attribution(&self, _used: &UsedSources<'_>) -> Vec<CreditLine> {
        Vec::new()
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        None
    }

    fn estimate(&self, _area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        Vec::new()
    }

    fn crs_for(&self, centre: LatLon) -> Crs {
        Crs::for_position(centre)
    }
}

/// Adds the adapter of Poland to `registry`, with the hosts a person allowed for it, by adapter
/// id (as the states take them).
pub fn register_with_hosts(
    registry: &mut Registry,
    extra: &BTreeMap<String, Vec<String>>,
) -> Result<(), DuplicateAdapter> {
    let allowed = extra.get(POLSKA).map_or(&[][..], Vec::as_slice);
    registry.add(Arc::new(Polska::new(allowed)))
}

/// [`register_with_hosts`] without a host allowed.
pub fn register(registry: &mut Registry) -> Result<(), DuplicateAdapter> {
    register_with_hosts(registry, &BTreeMap::new())
}
