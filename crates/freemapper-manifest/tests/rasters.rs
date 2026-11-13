//! The rasters a manifest declares as services: the services of the readers they build, the
//! backend that fetches them for a map area, and the backends a manifest names.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::NaiveDate;
use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::FetchMode;
use freemapper_adapter_api::project::{RasterProject, RasterRecords};
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::{NdsmRecord, TileRecord, WcsRecord, WmsRecord};
use freemapper_adapter_api::registry::Registry;
use freemapper_adapter_api::testing::MapFetcher;
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::model::DateSource;
use freemapper_manifest::rasters::{DeclaredRasters, height_service, orthophoto_source, query_of};
use freemapper_manifest::{ManifestError, dates, parse};
use freemapper_sources::raster::GridSpec;
use freemapper_sources::testing::{solid_jpeg, wcs_answer};
use freemapper_sources::wcs::WcsService;
use freemapper_sources::wms::{ORTHOPHOTO_PIXEL_M, OrthophotoSource, WmsService};

const WMS: &str =
    "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest";
const DGM: &str = "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1";
const DOM: &str = "https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1";
const INDEX: &str = "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis";

/// 20 m by 16 m at the Latina, Halle (EPSG:25832).
const SAMPLE: BoundingBox = BoundingBox {
    min_e: 706_440.0,
    min_n: 5_706_900.0,
    max_e: 706_460.0,
    max_n: 5_706_916.0,
};

fn repo(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn valid(id: &str) -> freemapper_manifest::Manifest {
    let text = repo(&format!("crates/freemapper-manifest/tests/valid/{id}.toml"));
    parse(&text, Path::new(id)).unwrap()
}

fn sample() -> String {
    repo("crates/freemapper-manifest/tests/invalid/unknown-key.toml")
        .replace("colour = \"red\"\n", "")
}

fn codes(findings: &[ManifestError]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.code).collect()
}

/// A project folder with its raster records.
struct TestProject {
    dir: PathBuf,
    tiles: Vec<TileRecord>,
    ndsm_tiles: Vec<NdsmRecord>,
    orthophoto: Option<WmsRecord>,
    terrain: Option<WcsRecord>,
    surface: Option<WcsRecord>,
    saves: usize,
}

impl TestProject {
    fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            tiles: Vec::new(),
            ndsm_tiles: Vec::new(),
            orthophoto: None,
            terrain: None,
            surface: None,
            saves: 0,
        }
    }
}

impl RasterProject for TestProject {
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

/// What the services of Saxony-Anhalt answer for the sample area.
fn serve(fetcher: &MapFetcher) {
    let grid = GridSpec::covering(&SAMPLE, 1.0);
    for (url, lift) in [(DGM, 80.0), (DOM, 92.0)] {
        let values: Vec<f32> = (0..grid.width * grid.height)
            .map(|i| (i % grid.width) as f32 * 0.01 + lift)
            .collect();
        fetcher.insert_prefix(url, wcs_answer(&grid, &values));
    }
    let photo = GridSpec::covering(&SAMPLE, ORTHOPHOTO_PIXEL_M);
    fetcher.insert_prefix(
        &format!("{WMS}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap"),
        solid_jpeg(photo.width, photo.height, [200, 0, 200]),
    );
    for (layer, fixture) in [
        (
            "truedop_kacheln",
            "tests/fixtures/lvermgeo-truedop-index.json",
        ),
        (
            "bdom_aktualitaet_zyklus",
            "tests/fixtures/lvermgeo-bdom-index.json",
        ),
    ] {
        fetcher.insert_prefix(
            &format!(
                "{INDEX}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=atkis_geobasis_bestand:{layer}&"
            ),
            repo(fixture).into_bytes(),
        );
    }
}

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
}

/// Runs `backend` for the sample area and returns how many notes it reported.
fn fetch(
    backend: &dyn RasterBackend,
    fetcher: &MapFetcher,
    project: &mut TestProject,
    mode: FetchMode,
    skip_surface: bool,
) -> Result<usize, AdapterError> {
    let registry = Registry::new();
    let cache = CacheDir::new("no-cache".into());
    let mut notes = 0;
    let mut env = AdapterEnv {
        fetcher,
        regions: &registry,
        cache: &cache,
        mode,
        today: today(),
        skip_surface,
        progress: &mut |_| notes += 1,
    };
    backend.fetch(&mut env, project, &SAMPLE)?;
    Ok(notes)
}

#[test]
fn the_raster_tables_build_the_services_of_the_readers() {
    let st = valid("saxonyanhalt");
    assert_eq!(
        orthophoto_source(st.orthophoto.as_ref().unwrap()),
        Some(OrthophotoSource::Wms(WmsService {
            url: Cow::Borrowed(WMS),
            layer: Cow::Borrowed("lsa_lvermgeo_dop20_2"),
            crs: Crs::Utm32,
            extra: Cow::Borrowed(""),
            max_px: 4000,
        }))
    );
    let envelope = BoundingBox {
        min_e: 606_000.0,
        min_n: 5_646_000.0,
        max_e: 790_000.0,
        max_n: 5_882_000.0,
    };
    let wcs = |url: &'static str| WcsService {
        url: Cow::Borrowed(url),
        coverage: Cow::Borrowed("Coverage1"),
        extra: Cow::Borrowed(""),
        multipart: true,
        envelope,
        max_m: 4000.0,
    };
    assert_eq!(height_service(st.terrain.as_ref().unwrap()), Some(wcs(DGM)));
    assert_eq!(height_service(st.surface.as_ref().unwrap()), Some(wcs(DOM)));
    // A table that names a backend builds no service.
    let hamburg = valid("hamburg");
    assert_eq!(
        orthophoto_source(hamburg.orthophoto.as_ref().unwrap()),
        None
    );
    assert_eq!(height_service(hamburg.terrain.as_ref().unwrap()), None);
    assert!(DeclaredRasters::new(&hamburg).is_none());
    // More parameters follow the request, their values encoded.
    let params = [
        ("TIME".to_string(), "2026".to_string()),
        ("note".to_string(), "a b&c=d/ä".to_string()),
    ];
    assert_eq!(query_of(&params), "&TIME=2026&note=a%20b%26c%3Dd%2F%C3%A4");
    let timed = sample().replace(
        "max_px = 4000\n",
        "max_px = 4000\nparams = { TIME = \"2026\" }\n",
    );
    let timed = parse(&timed, Path::new("sample")).unwrap();
    let Some(OrthophotoSource::Wms(wms)) = orthophoto_source(timed.orthophoto.as_ref().unwrap())
    else {
        panic!("no WMS");
    };
    assert_eq!(wms.extra, "&TIME=2026");
}

/// The backend of a manifest without code: the flight date from the index, the orthophoto from
/// the WMS, the terrain model, the surface model and its date, in this order, each stored in
/// the project folder and recorded.
#[test]
fn a_declared_backend_fetches_orthophoto_terrain_and_surface() {
    let backend = DeclaredRasters::new(&valid("saxonyanhalt")).unwrap();
    assert_eq!(
        backend.provider(),
        &ProviderInfo {
            id: "lvermgeo".parse().unwrap(),
            cache_dir: None,
            terrain_source: "lvermgeo-st-dgm1".to_string(),
            kind: RasterKind::PerArea,
        }
    );
    let dir = tempfile::tempdir().unwrap();
    let fetcher = MapFetcher::default();
    serve(&fetcher);
    let mut project = TestProject::new(dir.path());
    fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
    for file in ["data/orthophoto-wms.jpg", "data/dgm1.tif", "data/dom1.tif"] {
        assert!(dir.path().join(file).exists(), "{file}");
    }
    let orthophoto = project.orthophoto.clone().unwrap();
    assert_eq!(orthophoto.flight.as_deref(), Some("29.04.2026"));
    assert_eq!(orthophoto.fetched, today());
    assert_eq!(project.terrain.clone().unwrap().date, None);
    assert_eq!(
        project.surface.clone().unwrap().date.as_deref(),
        Some("13.05.2024")
    );
    assert_eq!(project.saves, 1);
    let log = fetcher.log.borrow().clone();
    let asked: Vec<&str> = log
        .iter()
        .map(|url| match url {
            url if url.contains("truedop_kacheln") => "flight date",
            url if url.contains("bdom_aktualitaet_zyklus") => "surface date",
            url if url.starts_with(WMS) => "orthophoto",
            url if url.starts_with(DGM) => "terrain",
            url if url.starts_with(DOM) => "surface",
            other => panic!("unexpected request {other}"),
        })
        .collect();
    assert_eq!(
        asked,
        [
            "flight date",
            "orthophoto",
            "terrain",
            "surface",
            "surface date"
        ]
    );
    assert_eq!(
        log[0],
        format!(
            "{INDEX}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
             &TYPENAMES=atkis_geobasis_bestand:truedop_kacheln&OUTPUTFORMAT=GEOJSON\
             &BBOX=706440,5706898,706460,5706918,urn:ogc:def:crs:EPSG::25832"
        )
    );
    // A second fetch with the same flight date keeps the orthophoto and asks the heights again.
    let before = fetcher.log.borrow().len();
    fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
    let again = fetcher.log.borrow()[before..].to_vec();
    assert!(!again.iter().any(|url| url.starts_with(WMS)), "{again:?}");
    assert_eq!(again.len(), 4);
    // Offline nothing is asked; what is stored covers the area.
    let before = fetcher.log.borrow().len();
    fetch(&backend, &fetcher, &mut project, FetchMode::Offline, false).unwrap();
    assert_eq!(fetcher.log.borrow().len(), before);
}

/// The surface model is a template: when its service fails the fetch goes on with a warning.
/// Without room for it, it is not asked. Offline without a terrain model is an error.
#[test]
fn the_surface_model_may_be_missing_and_the_terrain_model_may_not() {
    let backend = DeclaredRasters::new(&valid("saxonyanhalt")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let fetcher = MapFetcher::default();
    serve(&fetcher);
    fetcher.refusing.borrow_mut().push(DOM.to_string());
    let mut project = TestProject::new(dir.path());
    let notes = fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
    assert!(project.terrain.is_some() && project.surface.is_none());
    assert!(notes >= 3, "{notes} notes");
    // No room for the surface model: its service is not asked at all.
    let dir = tempfile::tempdir().unwrap();
    let fetcher = MapFetcher::default();
    serve(&fetcher);
    let mut project = TestProject::new(dir.path());
    fetch(&backend, &fetcher, &mut project, FetchMode::Online, true).unwrap();
    assert!(!fetcher.log.borrow().iter().any(|url| url.starts_with(DOM)));
    assert!(project.terrain.is_some() && project.surface.is_none());
    // Offline with nothing stored.
    let dir = tempfile::tempdir().unwrap();
    let mut project = TestProject::new(dir.path());
    let offline = fetch(
        &backend,
        &MapFetcher::default(),
        &mut project,
        FetchMode::Offline,
        false,
    );
    assert!(
        matches!(offline, Err(AdapterError::OrthophotoNotFetched)),
        "{offline:?}"
    );
    // A terrain model that fails is an error, and nothing is recorded for it.
    let dir = tempfile::tempdir().unwrap();
    let fetcher = MapFetcher::default();
    serve(&fetcher);
    fetcher.refusing.borrow_mut().push(DGM.to_string());
    let mut project = TestProject::new(dir.path());
    assert!(fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).is_err());
    assert!(project.terrain.is_none());
}

/// When the service of the surface model fails, a stored model that covers the area stays, and
/// one of another area goes with its file.
#[test]
fn a_failing_surface_model_keeps_a_covering_one_and_drops_another() {
    let backend = DeclaredRasters::new(&valid("saxonyanhalt")).unwrap();
    for covers in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher);
        let mut project = TestProject::new(dir.path());
        fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
        let surface = dir.path().join("data/dom1.tif");
        assert!(project.surface.is_some() && surface.exists());
        if !covers {
            project.surface.as_mut().unwrap().bbox[2] -= 50.0;
        }
        fetcher.refusing.borrow_mut().push(DOM.to_string());
        fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
        assert_eq!(project.surface.is_some(), covers);
        assert_eq!(surface.exists(), covers);
    }
}

/// A manifest without a surface model: two rasters, no warning about a third.
#[test]
fn a_manifest_without_a_surface_model_fetches_two_rasters() {
    let text = repo("crates/freemapper-manifest/tests/valid/saxonyanhalt.toml");
    let cut = text.find("[surface]").unwrap();
    let end = text.find("[preview]").unwrap();
    let manifest = parse(
        &format!("{}{}", &text[..cut], &text[end..]),
        Path::new("saxonyanhalt"),
    )
    .unwrap();
    assert!(manifest.surface.is_none());
    let backend = DeclaredRasters::new(&manifest).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let fetcher = MapFetcher::default();
    serve(&fetcher);
    let mut project = TestProject::new(dir.path());
    fetch(&backend, &fetcher, &mut project, FetchMode::Online, false).unwrap();
    assert!(project.terrain.is_some() && project.surface.is_none());
    assert!(!fetcher.log.borrow().iter().any(|url| url.starts_with(DOM)));
    assert_eq!(project.saves, 1);
    fetch(&backend, &fetcher, &mut project, FetchMode::Offline, false).unwrap();
}

/// The date kinds: a STAC API gives the newest item at the centre of the area, the feature
/// info of the WMS a field of its answer, a WFS a field in one of two spellings.
#[test]
fn niedersachsen_stac_date_matches_parse_fixture() {
    let stac = "https://dop.stac.lgln.niedersachsen.de";
    let fetcher = MapFetcher::default();
    fetcher.insert_prefix(
        stac,
        repo("tests/fixtures/niedersachsen-stac-dop.json").into_bytes(),
    );
    let hannover = BoundingBox {
        min_e: 550_000.0,
        min_n: 5_802_000.0,
        max_e: 550_200.0,
        max_n: 5_802_050.0,
    };
    let newest = DateSource::StacNewest {
        url: stac.to_string(),
    };
    assert_eq!(
        dates::lookup(&newest, &fetcher, None, Crs::Utm32, &hannover).as_deref(),
        Some("06.03.2025")
    );
    assert!(fetcher.log.borrow()[0].starts_with(&format!("{stac}/search?bbox=9.")));
    // A STAC that does not answer gives no date.
    assert_eq!(
        dates::lookup(&newest, &MapFetcher::default(), None, Crs::Utm32, &hannover),
        None
    );
    assert_eq!(
        dates::lookup(&DateSource::None, &fetcher, None, Crs::Utm32, &hannover),
        None
    );
    // The feature info of the orthophoto's WMS.
    let info = DateSource::WmsFeatureInfo {
        layer: "nw_dop_utm_info".to_string(),
        field: "Bildflugdatum".to_string(),
    };
    let wms = "https://www.wms.nrw.de/geobasis/wms_nw_dop";
    let fetcher = MapFetcher::default();
    fetcher.insert_prefix(
        &format!("{wms}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=nw_dop_utm_info"),
        b"Layer 'nw_dop_utm_info'\n  Feature 1: \n    Bildflugdatum = '11.05.2025'\n".to_vec(),
    );
    assert_eq!(
        dates::lookup(
            &info,
            &fetcher,
            Some((wms, Crs::Utm32)),
            Crs::Utm32,
            &hannover
        )
        .as_deref(),
        Some("11.05.2025")
    );
    // Without a WMS to ask there is no date.
    assert_eq!(
        dates::lookup(&info, &fetcher, None, Crs::Utm32, &hannover),
        None
    );
    // A WFS that writes its dates as YYYY-MM-DD: the records keep DD.MM.YYYY.
    let fetcher = MapFetcher::default();
    fetcher.insert_prefix(
        "https://geo.example.org/index?",
        br#"{"features":[{"properties":{"flown":"2025-05-11T00:00:00","other":"11.05.2025"}}]}"#
            .to_vec(),
    );
    let wfs = |field: &str, format| DateSource::WfsGeoJson {
        url: "https://geo.example.org/index".to_string(),
        layer: "ns:tiles".to_string(),
        field: field.to_string(),
        format,
    };
    use freemapper_manifest::model::DateFormat;
    for (field, format, expected) in [
        ("flown", DateFormat::Iso, Some("11.05.2025")),
        ("other", DateFormat::DayMonthYear, Some("11.05.2025")),
        ("other", DateFormat::Iso, None),
        ("missing", DateFormat::Iso, None),
    ] {
        assert_eq!(
            dates::lookup(&wfs(field, format), &fetcher, None, Crs::Utm33, &hannover).as_deref(),
            expected,
            "{field}"
        );
    }
}

struct Square;

impl AdapterFiles for Square {
    fn read(&self, _: &str) -> Result<Cow<'static, str>, String> {
        Ok(Cow::Borrowed(
            r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
            [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#,
        ))
    }
}

/// A backend that only says who it is.
struct Named(ProviderInfo);

impl RasterBackend for Named {
    fn provider(&self) -> &ProviderInfo {
        &self.0
    }

    fn fetch(
        &self,
        _: &mut AdapterEnv<'_>,
        _: &mut dyn RasterProject,
        _: &BoundingBox,
    ) -> Result<(), AdapterError> {
        unreachable!("the test fetches nothing")
    }
}

fn named(provider: &str) -> Arc<dyn RasterBackend> {
    Arc::new(Named(ProviderInfo {
        id: provider.parse().unwrap(),
        cache_dir: Some(provider.to_string()),
        terrain_source: format!("{provider}-dgm1"),
        kind: RasterKind::PerArea,
    }))
}

/// A manifest names the code that fetches its rasters; the registry of the program has it or
/// the adapter cannot be used.
#[test]
fn unknown_backend_names_the_missing_feature() {
    let hamburg = || valid("hamburg");
    let findings = ManifestAdapter::new(hamburg(), &Square, Parse::Now, &Registry::new())
        .err()
        .unwrap();
    assert_eq!(codes(&findings), ["manifest.backend-missing"]);
    assert_eq!(findings[0].key, "orthophoto.backend");
    assert!(
        findings[0].message.contains("`lgv`"),
        "{}",
        findings[0].message
    );
    assert!(
        findings[0].message.contains("feature"),
        "{}",
        findings[0].message
    );
    // With the backend registered the adapter hands it out.
    let mut registry = Registry::new();
    registry.add_backend("lgv", named("lgv")).unwrap();
    assert!(registry.add_backend("lgv", named("lgv")).is_err());
    assert!(registry.backend("lgb").is_none());
    let adapter = ManifestAdapter::new(hamburg(), &Square, Parse::Now, &registry).unwrap();
    assert_eq!(
        adapter.rasters().unwrap().provider().cache_dir.as_deref(),
        Some("lgv")
    );
    // A backend of another provider than the manifest's is a mistake in the manifest.
    let mut other = Registry::new();
    other.add_backend("lgv", named("lgb")).unwrap();
    let findings = ManifestAdapter::new(hamburg(), &Square, Parse::Now, &other)
        .err()
        .unwrap();
    assert_eq!(codes(&findings), ["manifest.value"]);
    assert_eq!(findings[0].key, "adapter.provider");
}

/// An adapter whose rasters are services brings its own backend and its estimate; one without
/// raster tables has none and is never the raster region.
#[test]
fn the_adapter_fetches_what_its_manifest_declares() {
    let registry = Registry::new();
    let st = ManifestAdapter::new(valid("saxonyanhalt"), &Square, Parse::Now, &registry).unwrap();
    let rasters = st.rasters().unwrap();
    assert_eq!(rasters.provider().id.as_str(), "lvermgeo");
    let bbox = BoundingBox {
        min_e: 700_000.0,
        min_n: 5_700_000.0,
        max_e: 701_000.0,
        max_n: 5_701_000.0,
    };
    let area = EstimateArea {
        bbox_25833: bbox,
        map_bbox: bbox,
        today: today(),
    };
    let cache = CacheDir::new("no-cache".into());
    let items = st.estimate(&area, &EstimateEnv { cache: &cache });
    let seen: Vec<(&str, f64)> = items
        .iter()
        .map(|i| (i.what.as_str(), i.megabytes))
        .collect();
    assert_eq!(
        seen,
        [
            ("orthophoto (LVermGeo WMS)", 8.0),
            ("terrain model (LVermGeo DGM1)", 4.0),
            ("surface model (LVermGeo DOM1)", 4.0)
        ]
    );
    let text = sample();
    let cut = text.find("[orthophoto]").unwrap();
    let alkis_only = parse(&text[..cut], Path::new("sample")).unwrap();
    let adapter = ManifestAdapter::new(alkis_only, &Square, Parse::Now, &registry).unwrap();
    assert!(adapter.rasters().is_none());
    assert!(
        adapter
            .estimate(&area, &EstimateEnv { cache: &cache })
            .is_empty()
    );
}

/// The raster tables belong together: an orthophoto and a terrain model, all of them services
/// or all of them one backend.
#[test]
fn the_raster_tables_fit_together() {
    let cases: [(&str, &str); 4] = [
        // An orthophoto without a terrain model.
        (
            "[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm/wcs\"\ncoverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n",
            "",
        ),
        // A service beside a backend.
        (
            "[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm/wcs\"\ncoverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n",
            "[terrain]\nbackend = \"sample\"\n",
        ),
        // Two backends.
        (
            "[orthophoto]\nkind = \"wms\"\nurl = \"https://geo.example.org/dop/wms\"\nlayer = \"dop20\"\nmax_px = 4000\n\n[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm/wcs\"\ncoverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n",
            "[orthophoto]\nbackend = \"one\"\n\n[terrain]\nbackend = \"two\"\n",
        ),
        // A surface model alone.
        (
            "[orthophoto]\nkind = \"wms\"\nurl = \"https://geo.example.org/dop/wms\"\nlayer = \"dop20\"\nmax_px = 4000\n\n[terrain]\nkind = \"wcs\"\nurl = \"https://geo.example.org/dgm/wcs\"\ncoverage = \"dgm1\"\nenvelope = [600000, 5600000, 700000, 5700000]\nmax_m = 4000\n",
            "[surface]\nbackend = \"one\"\n",
        ),
    ];
    for (old, new) in cases {
        assert_eq!(sample().matches(old).count(), 1, "{old}");
        let findings = parse(&sample().replace(old, new), Path::new("sample")).unwrap_err();
        assert_eq!(
            codes(&findings),
            ["manifest.rasters"],
            "{new}: {findings:?}"
        );
    }
    // A date is asked of a service, never of code.
    let coded = sample().replace(
        "max_px = 4000\n",
        "max_px = 4000\ndate = { backend = \"sample\" }\n",
    );
    assert_eq!(
        codes(&parse(&coded, Path::new("sample")).unwrap_err()),
        ["manifest.unknown-key"]
    );
}
