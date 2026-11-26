// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapter `polska`: Poland as one region. Its boundary is the state border of OpenStreetMap
//! (relation 49715, ODbL, simplified to about 40 m). The boundaries of its OSM extracts are the
//! clipping polygons Geofabrik publishes (from `index-v1.json`); they reach past the border and
//! only choose the extract. All are embedded as GeoJSON in WGS 84 and projected to EPSG:25833
//! once on first use. Stage 2 adds the rasters of GUGiK (terrain model, surface model and
//! orthophoto, [`crate::rasters::PolskaRasters`]) and stage 3 the cadastre EGiB ([`crate::egib`]).
//! The permit notes come from the GDOŚ and the PRG ([`crate::permits`], ADR 0175). The adapter
//! asks the two hosts of GUGiK, the host of the GDOŚ and the host of the NID and no other ([`HOSTS`]).

use std::collections::BTreeMap;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, Remedy};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::extras::ExtraSet;
use freemapper_adapter_api::fetch::FetchMode;
use freemapper_adapter_api::info::{
    Country, Coverage, Declared, Names, OsmExtractInfo, PreviewInfo, RegionInfo,
};
use freemapper_adapter_api::permits::PermitSource;
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

use crate::bdot10k::{self, Gap};
use crate::egib;
use crate::gugik_index::{DOWNLOAD_HOST, GugikIndex, INDEX_HOST};
use crate::permits::{GDOS_HOST, NID_HOST};
use crate::rasters::{PROVIDER, PolskaRasters};

/// The id of the adapter.
pub const POLSKA: &str = "polska";

/// The credit line of the data of GUGiK (terrain model, surface model, orthophoto), and the text
/// of the WMS the area page shows.
pub const CREDIT: &str = "© GUGiK, geoportal.gov.pl";

/// The hosts the adapter asks besides what a person allowed: the sheet index and the services of
/// GUGiK ([`INDEX_HOST`]: rasters, cadastre and the boundaries of the PRG), the download of its
/// sheets ([`DOWNLOAD_HOST`]) and the services of the GDOŚ ([`GDOS_HOST`]) and the register of the NID ([`NID_HOST`]). The OSM extracts
/// come through the core hosts, and the area page asks the WMS of `mapy.geoportal.gov.pl` itself,
/// in the browser.
pub const HOSTS: [&str; 4] = [INDEX_HOST, DOWNLOAD_HOST, GDOS_HOST, NID_HOST];

/// The state border of Poland (OSM relation 49715), WGS 84.
const POLAND_BOUNDARY: &str = include_str!("../data/poland.geojson");

/// The voivodeships as Geofabrik extracts them: id, English name, size of the extract in MB at the
/// time of writing, boundary (WGS 84). The extract of `<id>` lies in the folder of the voivodeship,
/// `europe/poland/<id>/<id>-latest.osm.pbf`; the old address `europe/poland/<id>-latest.osm.pbf`
/// redirects to a place that does not exist (checked 21 November 2026).
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
    rasters: GuardedRasters,
    cadastre: PolskaCadastre,
    permits: Vec<Box<dyn PermitSource>>,
    extras: Vec<Box<dyn ExtraSet>>,
    boundary: MultiPolygon<f64>,
    extracts: Vec<MultiPolygon<f64>>,
}

impl Polska {
    /// The adapter, held to the hosts of GUGiK ([`HOSTS`]) and to `allowed`, the hosts a person
    /// allowed for it (`freemapper adapters allow-host polska <host>`). The OSM extracts come
    /// from the engine, which asks Geofabrik itself.
    pub fn new(allowed: &[String]) -> Self {
        let osm = VOIVODESHIPS
            .iter()
            .map(|(id, label, megabytes, _)| OsmExtractInfo {
                geofabrik_path: format!("europe/poland/{id}/{id}"),
                megabytes: *megabytes,
                label: (*label).to_string(),
                prefix: format!("{id}_"),
            })
            .collect();
        let id = AdapterId::built_in(POLSKA);
        let mut policy = UrlPolicy::new(HOSTS).expect("the hosts of GUGiK are DNS names");
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
                revision: 13,
                hosts: HOSTS.iter().map(ToString::to_string).collect(),
                last_verified: chrono::NaiveDate::from_ymd_opt(2026, 11, 24),
            },
            rasters: GuardedRasters {
                inner: PolskaRasters::new(Arc::new(GugikIndex::new())),
                policy: policy.clone(),
            },
            cadastre: PolskaCadastre {
                policy: policy.clone(),
            },
            policy,
            permits: crate::permits::sources(),
            extras: vec![Box::new(crate::bdot_roads::Roads)],
            info: RegionInfo {
                id: id.clone(),
                // The rasters are those of GUGiK ([`PolskaRasters`]).
                provider: AdapterId::built_in(PROVIDER),
                names: Names {
                    en: "Poland".into(),
                    de: "Polen".into(),
                    pl: Some("Polska".into()),
                    cs: Some("Polsko".into()),
                },
                // The zone of a map is chosen by its centre ([`RegionAdapter::crs_for`]).
                crs: Crs::Utm33,
                country: Country::Pl,
                // Rasters and the cadastre (EGiB) of GUGiK (ADR 0170, 0171).
                coverage: Coverage::Full,
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
                    attribution: CREDIT.into(),
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

    /// The protected areas of the GDOŚ and the forest districts of the PRG (ADR 0175).
    fn permit_sources(&self) -> &[Box<dyn PermitSource>] {
        &self.permits
    }

    /// The roads and paths of BDOT10k for a forest map (ADR 0187).
    fn extras(&self) -> &[Box<dyn ExtraSet>] {
        &self.extras
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        self.extracts.get(index)
    }

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        Some(&self.cadastre)
    }

    /// The credit of GUGiK for a project whose rasters came from it and holds a terrain model
    /// or an orthophoto, or that has buildings or land use of the cadastre or of BDOT10k (the
    /// same office and the same credit line).
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        let holds = used.terrain.is_some() || used.orthophoto.is_some();
        let cadastre = used.alkis_fetches(POLSKA).next().is_some();
        // The roads and paths of BDOT10k are of the same office.
        let roads = used.extra_layers.iter().any(|(name, count)| {
            *count > 0
                && crate::bdot_roads::CLASSES
                    .iter()
                    .any(|(layer, _)| layer == name)
        });
        if (used.rasters_from(PROVIDER) && holds) || cadastre || roads {
            vec![CreditLine::new(CREDIT)]
        } else {
            Vec::new()
        }
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        Some(&self.rasters)
    }

    /// The estimate of [`PolskaRasters`]: no raster item would be wrong while the region has
    /// rasters, and the engine adds none of the cadastre for it.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        match self.info.coverage {
            Coverage::Full | Coverage::Rasters => self.rasters.estimate(area, env),
            Coverage::OsmOnly => Vec::new(),
        }
    }

    fn crs_for(&self, centre: LatLon) -> Crs {
        Crs::for_position(centre)
    }
}

/// The cadastre of GUGiK (EGiB, [`crate::egib`]) with every request held to the hosts of the
/// adapter's policy, as [`GuardedRasters`] holds the rasters.
struct PolskaCadastre {
    policy: UrlPolicy,
}

impl PolskaCadastre {
    /// The features of `layer` in the area as EGiB gives them, with the box (EPSG:2180).
    fn egib(
        fetcher: &PolicyFetcher<'_>,
        area: &FetchArea,
        layer: egib::Layer,
    ) -> Result<(Vec<egib::Feature>, BoundingBox), AdapterError> {
        let bbox = egib::area_2180(area)?;
        let features = egib::fetch_all(fetcher, layer, &bbox).map_err(AdapterError::from);
        features.map(|f| (f, bbox))
    }

    /// The parcels of EGiB that give a land use, as polygons (`x` east, `y` north).
    fn covered(features: &[egib::Feature]) -> Vec<geo_types::Polygon<f64>> {
        features
            .iter()
            .filter(|f| egib::parcel_land_use(&f.kind).is_some())
            .flat_map(|f| f.polygons.iter().filter_map(|p| bdot10k::to_polygon(p)))
            .collect()
    }

    /// Adds the cemeteries of BDOT10k that lie on parcels EGiB gives no land use (`Bi` other
    /// built-up land, which holds schools, offices, shops and cemeteries alike, ADR 0192), as
    /// `Friedhof`. Offline nothing is asked; a failure is a warning and the land use stays that of
    /// EGiB (ADR 0194).
    fn add_cemeteries(
        env: &mut AdapterEnv<'_>,
        fetcher: &PolicyFetcher<'_>,
        features: &[egib::Feature],
        bbox: &BoundingBox,
        crs: Crs,
        part: &mut AlkisPart,
    ) {
        if env.mode == FetchMode::Offline {
            return;
        }
        match bdot10k::cemeteries(fetcher, bbox) {
            Ok(filled) => {
                let extra = bdot10k::land_use(&filled.objects, crs, &Self::covered(features));
                if !extra.features.is_empty() {
                    (env.progress)(bdot10k::filled_note(
                        Gap::Cemeteries,
                        extra.features.len(),
                        &filled.powiats,
                    ));
                    part.features.extend(extra.features);
                }
            }
            Err(e) => (env.progress)(bdot10k::failed_note(Gap::Cemeteries, e)),
        }
    }

    /// What BDOT10k holds for `gap`, with the notes that tell the person. `None` when it holds
    /// nothing (or the run is offline); the failure of BDOT10k is a warning and no error,
    /// because the map without it is what it was before.
    fn bdot(
        env: &mut AdapterEnv<'_>,
        fetcher: &PolicyFetcher<'_>,
        bbox: &BoundingBox,
        gap: Gap,
    ) -> Option<bdot10k::Filled> {
        match bdot10k::fill(env, fetcher, bbox, gap) {
            Ok(filled) if !filled.objects.is_empty() => Some(filled),
            Ok(_) => None,
            Err(e) => {
                (env.progress)(bdot10k::failed_note(gap, e));
                None
            }
        }
    }
}

impl AlkisSource for PolskaCadastre {
    /// The buildings of EGiB; where it gives none (a municipality without a cadastre service) or
    /// fails, those of BDOT10k (ADR 0178).
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let fetcher = PolicyFetcher::new(env.fetcher, &self.policy);
        let first = Self::egib(&fetcher, area, egib::Layer::Buildings);
        match first {
            Ok((features, _)) if !features.is_empty() => Ok(egib::buildings(&features, area.crs)),
            Ok((empty, bbox)) => {
                let part = Self::bdot(env, &fetcher, &bbox, Gap::Buildings).map(|filled| {
                    let part = bdot10k::buildings(&filled.objects, area.crs);
                    (env.progress)(bdot10k::filled_note(
                        Gap::Buildings,
                        part.features.len(),
                        &filled.powiats,
                    ));
                    part
                });
                Ok(part.unwrap_or_else(|| egib::buildings(&empty, area.crs)))
            }
            Err(AdapterError::Fetch(error)) => {
                let Ok(bbox) = egib::area_2180(area) else {
                    return Err(AdapterError::Fetch(error));
                };
                let Some(filled) = Self::bdot(env, &fetcher, &bbox, Gap::Buildings) else {
                    return Err(AdapterError::Fetch(error));
                };
                let part = bdot10k::buildings(&filled.objects, area.crs);
                (env.progress)(bdot10k::instead_note(
                    Gap::Buildings,
                    error,
                    &filled.powiats,
                ));
                Ok(part)
            }
            Err(other) => Err(other),
        }
    }

    /// The land use of EGiB; where its parcels give a land use for less than
    /// [`bdot10k::MIN_LAND_USE_SHARE`] of the area (the big cities send most parcels without a
    /// class) or EGiB fails, that of BDOT10k beside it, left out where EGiB has one (ADR 0178).
    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let fetcher = PolicyFetcher::new(env.fetcher, &self.policy);
        let first = Self::egib(&fetcher, area, egib::Layer::Parcels);
        let (features, bbox, failure) = match first {
            Ok((features, bbox)) => {
                if !bdot10k::land_use_is_thin(&features, &bbox) {
                    let mut part = egib::land_use(&features, area.crs);
                    Self::add_cemeteries(env, &fetcher, &features, &bbox, area.crs, &mut part);
                    return Ok(part);
                }
                (features, bbox, None)
            }
            Err(AdapterError::Fetch(error)) => {
                let Ok(bbox) = egib::area_2180(area) else {
                    return Err(AdapterError::Fetch(error));
                };
                (Vec::new(), bbox, Some(error))
            }
            Err(other) => return Err(other),
        };
        let own = egib::land_use(&features, area.crs);
        let Some(filled) = Self::bdot(env, &fetcher, &bbox, Gap::LandUse) else {
            return failure.map_or(Ok(own), |error| Err(AdapterError::Fetch(error)));
        };
        let covered = Self::covered(&features);
        let mut part = own;
        let extra = bdot10k::land_use(&filled.objects, area.crs, &covered);
        (env.progress)(match failure {
            Some(error) => bdot10k::instead_note(Gap::LandUse, error, &filled.powiats),
            None => bdot10k::filled_note(Gap::LandUse, extra.features.len(), &filled.powiats),
        });
        part.features.extend(extra.features);
        Ok(part)
    }
}

/// The rasters of GUGiK with every request held to the hosts of the adapter's policy: the sheet
/// index, the redirects of a download and the sheet addresses the index names. The engine hands
/// the backend a fetcher that knows no adapter, so the backend is wrapped here, as the manifest
/// crate wraps the backends of the states.
struct GuardedRasters {
    inner: PolskaRasters,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::fetch::FetchMode;
    use freemapper_adapter_api::project::RasterRecords;
    use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
    use freemapper_adapter_api::testing::MapFetcher;
    use std::path::{Path, PathBuf};

    struct Project {
        dir: PathBuf,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
    }

    impl RasterProject for Project {
        fn records(&mut self) -> freemapper_adapter_api::project::RasterRecords<'_> {
            RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            area()
        }

        fn crs(&self) -> Crs {
            Crs::Utm33
        }
    }

    /// A 20 m box at Słubice in UTM 33.
    fn area() -> BoundingBox {
        BoundingBox {
            min_e: 483_400.0,
            min_n: 5_800_800.0,
            max_e: 483_420.0,
            max_n: 5_800_820.0,
        }
    }

    /// The adapter offers the roads of BDOT10k as its set of extra layers.
    #[test]
    fn the_adapter_offers_the_road_layers_as_an_extra_set() {
        let polska = Polska::new(&[]);
        let extras = polska.extras();
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0].info().key, crate::bdot_roads::KEY);
        assert!(extras[0].drawing().is_some());
    }

    /// The guarded rasters give the estimate of the backend: the three downloads of GUGiK.
    #[test]
    fn the_guarded_rasters_give_the_estimate_of_the_backend() {
        let polska = Polska::new(&[]);
        let rasters = polska.rasters().expect("the rasters of GUGiK");
        let cache = CacheDir::new(std::env::temp_dir().join("freemapper-pl-guarded-estimate"));
        let items = rasters.estimate(
            &EstimateArea {
                bbox_25833: area(),
                map_bbox: area(),
                today: NaiveDate::from_ymd_opt(2026, 11, 21).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(items.len(), 3, "{items:?}");
        assert!(items.iter().all(|item| item.what.contains("GUGiK")));
    }

    /// The guarded rasters hand the fetch to the backend and give its error back: with no answer
    /// from the index, a fetch fails and writes nothing.
    #[test]
    fn the_guarded_rasters_pass_the_fetch_on_and_its_error_back() {
        let polska = Polska::new(&[]);
        let rasters = polska.rasters().expect("the rasters of GUGiK");
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let regions = Registry::new();
        let fetcher = MapFetcher::default();
        let mut progress = |_| {};
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &regions,
            cache: &cache,
            mode: FetchMode::Online,
            today: NaiveDate::from_ymd_opt(2026, 11, 21).unwrap(),
            skip_surface: false,
            progress: &mut progress,
        };
        let mut project = Project {
            dir: scratch.path().join("project"),
            tiles: Vec::new(),
            ndsm_tiles: Vec::new(),
            orthophoto: None,
            terrain: None,
            surface: None,
        };
        std::fs::create_dir_all(&project.dir).unwrap();
        let result = rasters.fetch(&mut env, &mut project, &area());
        assert!(result.is_err(), "{result:?}");
        assert!(project.terrain.is_none());
    }
}
