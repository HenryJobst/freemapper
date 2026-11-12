//! The manifests of the German states against the constants they were written from: the
//! services a manifest builds are the services the code names.

use std::borrow::Cow;
use std::path::Path;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapters_de::states::alkis::{
    BUILDINGS_ENDPOINT, LAND_USE_ENDPOINT, MV_BUILDINGS_ENDPOINT, MV_LAND_USE_ENDPOINT,
};
use freemapper_adapters_de::states::{State, StateAdapter};
use freemapper_adapters_de::{lgv, states};
use freemapper_georef::crs::Crs;
use freemapper_manifest::model::Alkis;
use freemapper_manifest::{Manifest, adapter, alkis, extras, parse};
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis};
use freemapper_sources::nas::LGB_NAS;
use freemapper_sources::trees::POTSDAM;
use freemapper_sources::wfs::{self, WfsService};

fn manifest(id: &str) -> Manifest {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("manifests")
        .join(id)
        .join("manifest.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse(&text, Path::new(id)).unwrap_or_else(|findings| {
        let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
        panic!("{}", lines.join("\n"))
    })
}

/// The states that have a manifest so far (Berlin follows with its code adapter).
const WITH_MANIFEST: [State; 10] = [
    State::Brandenburg,
    State::Saxony,
    State::SaxonyAnhalt,
    State::Hamburg,
    State::Hessen,
    State::SchleswigHolstein,
    State::MecklenburgVorpommern,
    State::NordrheinWestfalen,
    State::Niedersachsen,
    State::RheinlandPfalz,
];

/// The WFS of a manifest: its ALKIS service, or the fallback of its OGC API.
fn wfs_of(manifest: &Manifest) -> WfsService {
    match manifest.alkis.as_ref().unwrap() {
        Alkis::Wfs(wfs) => alkis::wfs_service(wfs),
        Alkis::OgcApi {
            fallback: Some(wfs),
            ..
        } => alkis::wfs_service(wfs),
        other => panic!("{}: no WFS in {other:?}", manifest.adapter.id),
    }
}

#[test]
fn manifest_wfs_rows_equal_the_nine_constants() {
    let rows: [(&str, WfsService); 9] = [
        ("saxony", wfs::GEOSN),
        ("saxonyanhalt", wfs::LVERMGEO),
        ("hamburg", wfs::LGV),
        ("hessen", wfs::HVBG),
        ("schleswigholstein", wfs::LVERMGEO_SH),
        ("mecklenburgvorpommern", wfs::LAIV_MV),
        ("nordrheinwestfalen", wfs::GEOBASIS_NRW),
        ("niedersachsen", wfs::LGLN),
        ("rheinlandpfalz", wfs::LVERMGEO_RP),
    ];
    for (id, constant) in rows {
        assert_eq!(wfs_of(&manifest(id)), constant, "{id}");
    }
}

#[test]
fn hamburg_manifest_equals_the_ogc_api_plus_wfs_fallback() {
    let api = |id: &str| alkis::ogc_api(manifest(id).alkis.as_ref().unwrap()).unwrap();
    assert_eq!(
        api("hamburg"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(lgv::BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(lgv::LAND_USE_ENDPOINT),
            crs: Crs::Utm32,
            fallback: Some(wfs::LGV),
            normalise: Normaliser::HamburgAscii,
        }
    );
    assert_eq!(
        api("mecklenburgvorpommern"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(MV_BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(MV_LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: Some(wfs::LAIV_MV),
            normalise: Normaliser::Lgb,
        }
    );
    assert_eq!(
        api("brandenburg"),
        OgcApiAlkis {
            buildings: Cow::Borrowed(BUILDINGS_ENDPOINT),
            land_use: Cow::Borrowed(LAND_USE_ENDPOINT),
            crs: Crs::Utm33,
            fallback: None,
            normalise: Normaliser::Lgb,
        }
    );
}

#[test]
fn potsdam_tree_dataset_from_manifest_equals_the_constant() {
    let brandenburg = manifest("brandenburg");
    let region = "brandenburg".parse().unwrap();
    assert_eq!(brandenburg.trees.len(), 1);
    assert_eq!(
        extras::tree_dataset(&brandenburg.trees[0], &region),
        POTSDAM
    );
    assert_eq!(brandenburg.nas.len(), 1);
    assert_eq!(extras::nas_service(&brandenburg.nas[0], &region), LGB_NAS);
    // No other state has such sources.
    for state in WITH_MANIFEST {
        if state != State::Brandenburg {
            let manifest = manifest(state.id());
            assert!(
                manifest.nas.is_empty() && manifest.trees.is_empty(),
                "{}",
                state.id()
            );
        }
    }
}

/// What a manifest says about its state is what the adapter of the state answers today: ids,
/// names, CRS, object-id prefix, the extracts and the preview.
#[test]
fn every_manifest_names_its_state() {
    for state in WITH_MANIFEST {
        let today = StateAdapter::new(state);
        assert_eq!(
            &adapter::info_of(&manifest(state.id())),
            today.info(),
            "{}",
            state.id()
        );
    }
    assert_eq!(states::adapters().len(), WITH_MANIFEST.len() + 1);
}

/// Saxony-Anhalt needs no code: the backend its manifest declares asks what `lvermgeo.rs` asks.
mod saxony_anhalt {
    use std::path::{Path, PathBuf};

    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::env::AdapterEnv;
    use freemapper_adapter_api::error::AdapterError;
    use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
    use freemapper_adapter_api::fetch::FetchMode;
    use freemapper_adapter_api::project::{RasterProject, RasterRecords};
    use freemapper_adapter_api::rasters::RasterBackend;
    use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_adapters_de::lvermgeo::{self, LvermgeoBackend, Model};
    use freemapper_adapters_de::states::{State, estimate};
    use freemapper_georef::crs::{BoundingBox, Crs};
    use freemapper_manifest::rasters::{DeclaredRasters, height_service, orthophoto_source};
    use freemapper_sources::raster::GridSpec;
    use freemapper_sources::testing::{solid_jpeg, wcs_answer};
    use freemapper_sources::wms::{ORTHOPHOTO_PIXEL_M, OrthophotoSource};

    use super::manifest;

    /// 20 m by 16 m at the Latina, Halle (EPSG:25832).
    const SAMPLE: BoundingBox = BoundingBox {
        min_e: 706_440.0,
        min_n: 5_706_900.0,
        max_e: 706_460.0,
        max_n: 5_706_916.0,
    };

    #[derive(Default)]
    struct Project {
        dir: PathBuf,
        tiles: Vec<TileRecord>,
        ndsm_tiles: Vec<NdsmRecord>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
        saves: usize,
    }

    impl RasterProject for Project {
        fn records(&mut self) -> RasterRecords<'_> {
            RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            self.saves += 1;
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            SAMPLE
        }

        fn crs(&self) -> Crs {
            Crs::Utm32
        }
    }

    fn fixture(name: &str) -> Vec<u8> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/fixtures")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// What the services answer for the sample area; `surface` says whether the DOM1 answers.
    fn served(surface: bool) -> MapFetcher {
        let fetcher = MapFetcher::default();
        let grid = GridSpec::covering(&SAMPLE, 1.0);
        for (model, lift) in [(Model::Dgm1, 80.0), (Model::Dom1, 92.0)] {
            let values: Vec<f32> = (0..grid.width * grid.height)
                .map(|i| (i % grid.width) as f32 * 0.01 + lift)
                .collect();
            fetcher.insert_prefix(model.wcs_endpoint(), wcs_answer(&grid, &values));
        }
        if !surface {
            fetcher
                .refusing
                .borrow_mut()
                .push(Model::Dom1.wcs_endpoint().to_string());
        }
        let photo = GridSpec::covering(&SAMPLE, ORTHOPHOTO_PIXEL_M);
        fetcher.insert_prefix(
            &format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap",
                lvermgeo::WMS.url
            ),
            solid_jpeg(photo.width, photo.height, [200, 0, 200]),
        );
        for (layer, name) in [
            ("truedop_kacheln", "lvermgeo-truedop-index.json"),
            ("bdom_aktualitaet_zyklus", "lvermgeo-bdom-index.json"),
        ] {
            fetcher.insert_prefix(
                &format!(
                    "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=atkis_geobasis_bestand:{layer}&",
                    lvermgeo::INDEX_URL
                ),
                fixture(name),
            );
        }
        fetcher
    }

    /// What a fetch leaves behind: the requests, the notes in English, the records and how
    /// often the project was saved.
    #[derive(Debug, PartialEq)]
    struct Outcome {
        result: Result<(), String>,
        requests: Vec<String>,
        notes: Vec<String>,
        orthophoto: Option<WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
        saves: usize,
    }

    /// Two fetches in a row (the second finds the orthophoto of the first), then one offline.
    fn run(backend: &dyn RasterBackend, surface: bool, skip_surface: bool) -> Vec<Outcome> {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = served(surface);
        let mut project = Project {
            dir: dir.path().to_path_buf(),
            ..Project::default()
        };
        let registry = Registry::new();
        let cache = CacheDir::new("no-cache".into());
        [FetchMode::Online, FetchMode::Online, FetchMode::Offline]
            .into_iter()
            .map(|mode| {
                let before = fetcher.log.borrow().len();
                let mut notes = Vec::new();
                let mut env = AdapterEnv {
                    fetcher: &fetcher,
                    regions: &registry,
                    cache: &cache,
                    mode,
                    today: NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
                    skip_surface,
                    progress: &mut |note| notes.push(format!("{note:?}")),
                };
                let result = backend
                    .fetch(&mut env, &mut project, &SAMPLE)
                    .map_err(|e| e.to_string());
                Outcome {
                    result,
                    requests: fetcher.log.borrow()[before..].to_vec(),
                    notes,
                    orthophoto: project.orthophoto.clone(),
                    terrain: project.terrain.clone(),
                    surface: project.surface.clone(),
                    saves: project.saves,
                }
            })
            .collect()
    }

    #[test]
    fn saxony_anhalt_manifest_builds_the_same_requests_as_lvermgeo_rs() {
        let st = manifest("saxonyanhalt");
        // The services of the tables are the constants of the module.
        assert_eq!(
            orthophoto_source(st.orthophoto.as_ref().unwrap()),
            Some(OrthophotoSource::Wms(lvermgeo::WMS))
        );
        assert_eq!(
            height_service(st.terrain.as_ref().unwrap()),
            Some(Model::Dgm1.wcs())
        );
        assert_eq!(
            height_service(st.surface.as_ref().unwrap()),
            Some(Model::Dom1.wcs())
        );
        let declared = DeclaredRasters::new(&st).unwrap();
        let coded = LvermgeoBackend::new();
        assert_eq!(declared.provider(), coded.provider());
        // A whole fetch: the same requests in the same order, the same notes, the same records;
        // also when the surface model fails, and when there is no room for it.
        for (surface, skip_surface) in [(true, false), (false, false), (true, true)] {
            let (ours, theirs) = (
                run(&declared, surface, skip_surface),
                run(&coded, surface, skip_surface),
            );
            assert_eq!(ours, theirs, "surface {surface}, skipped {skip_surface}");
            assert_eq!(ours[0].result, Ok(()));
            assert!(ours[0].requests.len() >= 3, "{:?}", ours[0].requests);
        }
        // The estimate lists the same downloads.
        let bbox = BoundingBox {
            min_e: 700_000.0,
            min_n: 5_700_000.0,
            max_e: 702_000.0,
            max_n: 5_703_000.0,
        };
        let area = EstimateArea {
            bbox_25833: bbox,
            map_bbox: bbox,
            today: NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
        };
        let cache = CacheDir::new("no-cache".into());
        let env = EstimateEnv { cache: &cache };
        assert_eq!(
            declared.estimate(&area, &env),
            estimate::items(State::SaxonyAnhalt, &area, &env)
        );
    }
}
