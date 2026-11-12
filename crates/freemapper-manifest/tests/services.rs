//! The services a manifest declares, as the readers ask them: ALKIS, the full ALKIS (NAS), tree
//! cadastres, and what the declared rasters download.

use std::borrow::Cow;
use std::path::Path;

use chrono::NaiveDate;
use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::credit::{AlkisUse, CreditLine, UsedSources};
use freemapper_adapter_api::estimate::EstimateArea;
use freemapper_adapter_api::extras::{TreeFields, TreeFormat};
use freemapper_adapter_api::records::{Part, Store};
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::model::Alkis;
use freemapper_manifest::{ManifestError, alkis, estimate, extras, parse};
use freemapper_sources::alkis_source::{Normaliser, OgcApiAlkis};
use freemapper_sources::wfs::WfsService;

fn repo(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn valid(id: &str) -> String {
    repo(&format!("crates/freemapper-manifest/tests/valid/{id}.toml"))
}

/// The smallest valid manifest (id `sample`, CRS EPSG:25832).
fn sample() -> String {
    repo("crates/freemapper-manifest/tests/invalid/unknown-key.toml")
        .replace("colour = \"red\"\n", "")
}

fn codes(findings: &[ManifestError]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.code).collect()
}

/// A backend that only says who it is, for the manifests that name one.
struct Named(freemapper_adapter_api::rasters::ProviderInfo);

impl freemapper_adapter_api::rasters::RasterBackend for Named {
    fn provider(&self) -> &freemapper_adapter_api::rasters::ProviderInfo {
        &self.0
    }

    fn fetch(
        &self,
        _: &mut freemapper_adapter_api::env::AdapterEnv<'_>,
        _: &mut dyn freemapper_adapter_api::project::RasterProject,
        _: &freemapper_georef::crs::BoundingBox,
    ) -> Result<(), freemapper_adapter_api::error::AdapterError> {
        unreachable!("the test fetches nothing")
    }
}

/// The backends the sample manifests name.
fn backends() -> freemapper_adapter_api::registry::Registry {
    let mut registry = freemapper_adapter_api::registry::Registry::new();
    for (name, provider) in [("lgv", "lgv"), ("geobasis-nrw", "geobasisnrw")] {
        let info = freemapper_adapter_api::rasters::ProviderInfo {
            id: provider.parse().unwrap(),
            cache_dir: None,
            terrain_source: format!("{provider}-dgm1"),
            kind: freemapper_adapter_api::rasters::RasterKind::PerArea,
        };
        registry
            .add_backend(name, std::sync::Arc::new(Named(info)))
            .unwrap();
    }
    registry
}

/// One boundary for every file a manifest names.
struct Square;

impl AdapterFiles for Square {
    fn read(&self, _: &str) -> Result<Cow<'static, str>, String> {
        Ok(Cow::Borrowed(
            r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
            [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#,
        ))
    }
}

const NAS: &str = "\n[[extra.nas]]\nname = \"Sample ALKIS (NAS)\"\nurl = \"https://geo.example.org/nas\"\n\
                   max_count = 100000\nmax_bytes = 268435456\n";
const TREES: &str = "\n[[extra.trees]]\nid = \"city\"\nname = \"City trees\"\n\
                     envelope = [600000, 5600000, 610000, 5610000]\nformat = \"ods-geojson-export\"\n\
                     geo_field = \"geo_point_2d\"\n\
                     endpoints = [\"https://geo.example.org/trees/1\", \"https://geo.example.org/trees/2\"]\n\
                     fields = { kind = \"art\", id = \"nr\", date = \"datum\", crown = \"krone\" }\n\
                     skipped_kinds = [\"Baumstumpf\"]\nattribution = \"City, dl-de/zero-2-0\"\nmax_bytes = 1000000\n";

#[test]
fn a_wfs_table_builds_the_service() {
    let manifest = parse(
        &valid("nordrheinwestfalen"),
        Path::new("nordrheinwestfalen"),
    )
    .unwrap();
    let Some(Alkis::Wfs(wfs)) = &manifest.alkis else {
        panic!("{:?}", manifest.alkis);
    };
    assert_eq!(
        alkis::wfs_service(wfs),
        WfsService {
            name: Cow::Borrowed("Geobasis NRW ALKIS"),
            url: Cow::Borrowed("https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht"),
            buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
            land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
            crs: Crs::Utm32,
            square_bez: Some(Cow::Borrowed("ohne Unterart (Nordrhein-Westfalen)")),
        }
    );
    assert_eq!(alkis::ogc_api(manifest.alkis.as_ref().unwrap()), None);
    // A service without a name of its own is called after the region.
    let plain = parse(&sample(), Path::new("sample")).unwrap();
    let Some(Alkis::Wfs(wfs)) = &plain.alkis else {
        panic!("{:?}", plain.alkis);
    };
    let service = alkis::wfs_service(wfs);
    assert_eq!(
        (service.name.as_ref(), service.crs),
        ("Sample ALKIS", Crs::Utm32)
    );
    assert_eq!(service.square_bez, None);
}

#[test]
fn an_ogc_api_table_builds_the_source_with_its_fallback() {
    let manifest = parse(&valid("hamburg"), Path::new("hamburg")).unwrap();
    assert_eq!(
        alkis::ogc_api(manifest.alkis.as_ref().unwrap()),
        Some(OgcApiAlkis {
            buildings: Cow::Borrowed(
                "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items"
            ),
            land_use: Cow::Borrowed(
                "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/NutzungFlurstueck/items"
            ),
            crs: Crs::Utm32,
            fallback: Some(WfsService {
                name: Cow::Borrowed("LGV ALKIS (WFS)"),
                url: Cow::Borrowed("https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht"),
                buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
                land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
                crs: Crs::Utm32,
                square_bez: None,
            }),
            normalise: Normaliser::HamburgAscii,
        })
    );
}

#[test]
fn nas_and_trees_build_their_services() {
    let text = format!("{}{NAS}layers = [\"sonstiges-bauwerk\"]\n{TREES}", sample());
    let manifest = parse(&text, Path::new("sample")).unwrap();
    let region = "sample".parse().unwrap();
    let nas = extras::nas_service(&manifest.nas[0], &region);
    assert_eq!(nas.name, "Sample ALKIS (NAS)");
    assert_eq!(nas.url, "https://geo.example.org/nas");
    assert_eq!((nas.region.as_str(), nas.crs), ("sample", Crs::Utm32));
    assert_eq!((nas.max_count, nas.max_bytes), (100_000, 256 << 20));
    assert!(nas.serves("sonstiges-bauwerk") && !nas.serves("strukturlinie"));
    assert_eq!(nas.namespaces, None);
    let trees = extras::tree_dataset(&manifest.trees[0], &region);
    assert_eq!(
        (trees.id.as_ref(), trees.name.as_ref()),
        ("city", "City trees")
    );
    assert_eq!(trees.region.as_str(), "sample");
    assert_eq!(
        trees.envelope,
        BoundingBox {
            min_e: 600_000.0,
            min_n: 5_600_000.0,
            max_e: 610_000.0,
            max_n: 5_610_000.0
        }
    );
    assert_eq!(
        trees.format,
        TreeFormat::OdsGeoJsonExport {
            geo_field: Cow::Borrowed("geo_point_2d")
        }
    );
    assert_eq!(
        trees.endpoints.as_ref(),
        [
            "https://geo.example.org/trees/1",
            "https://geo.example.org/trees/2"
        ]
    );
    assert_eq!(
        trees.fields,
        TreeFields {
            kind: Cow::Borrowed("art"),
            id: Cow::Borrowed("nr"),
            date: Cow::Borrowed("datum"),
            species: None,
            crown: Some(Cow::Borrowed("krone")),
        }
    );
    assert_eq!(trees.skipped_kinds.as_ref(), ["Baumstumpf"]);
    assert_eq!(trees.attribution, "City, dl-de/zero-2-0");
    assert_eq!(trees.max_bytes, 1_000_000);
}

/// A layer the NAS reader does not know would be left out silently, and the readers have room
/// for one NAS service per region.
#[test]
fn a_nas_table_names_known_layers_and_there_is_one() {
    let unknown = format!(
        "{}{NAS}layers = [\"strukturlinie\", \"vegetationsmerkmale\"]\n",
        sample()
    );
    let findings = parse(&unknown, Path::new("sample")).unwrap_err();
    assert_eq!(codes(&findings), ["manifest.value"], "{findings:?}");
    assert_eq!(findings[0].key, "extra.nas.layers");
    assert!(
        findings[0].message.contains("vegetationsmerkmale"),
        "{}",
        findings[0].message
    );
    let two = format!("{}{NAS}{NAS}", sample());
    let findings = parse(&two, Path::new("sample")).unwrap_err();
    assert_eq!(codes(&findings), ["manifest.value"], "{findings:?}");
    assert_eq!(findings[0].key, "extra.nas");
}

fn area() -> EstimateArea {
    // 2 km by 3 km.
    let bbox = BoundingBox {
        min_e: 700_000.0,
        min_n: 5_700_000.0,
        max_e: 702_000.0,
        max_n: 5_703_000.0,
    };
    EstimateArea {
        bbox_25833: bbox,
        map_bbox: bbox,
        today: NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
    }
}

/// A raster that is a service grows with the area; one that a backend fetches is the backend's
/// to estimate.
#[test]
fn declared_rasters_are_estimated_by_area() {
    let st = parse(&valid("saxonyanhalt"), Path::new("saxonyanhalt")).unwrap();
    let items = estimate::declared_items(&st, &area());
    let seen: Vec<(&str, f64, Part)> = items
        .iter()
        .map(|item| (item.what.as_str(), item.megabytes, item.part))
        .collect();
    assert_eq!(
        seen,
        [
            ("orthophoto (LVermGeo WMS)", 48.0, Part::Orthophoto),
            ("terrain model (LVermGeo DGM1)", 24.0, Part::Terrain),
            ("surface model (LVermGeo DOM1)", 24.0, Part::Surface),
        ]
    );
    assert!(
        items
            .iter()
            .all(|item| !item.cached && item.store == Store::Project)
    );
    // Other sizes in `[estimate]`.
    let other = valid("saxonyanhalt").replace("terrain_mb_per_km2 = 4", "terrain_mb_per_km2 = 1.5");
    let items =
        estimate::declared_items(&parse(&other, Path::new("saxonyanhalt")).unwrap(), &area());
    assert_eq!(items[1].megabytes, 9.0);
    // Hamburg: a backend fetches the three rasters, so the manifest declares none to estimate.
    let hamburg = parse(&valid("hamburg"), Path::new("hamburg")).unwrap();
    assert!(estimate::declared_items(&hamburg, &area()).is_empty());
    // Without a label of its own a raster is named after the region and its service.
    let plain = parse(&sample(), Path::new("sample")).unwrap();
    assert_eq!(
        estimate::declared_items(&plain, &area())[0].what,
        "orthophoto (Sample WMS)"
    );
    assert_eq!(
        estimate::declared_items(&plain, &area())[1].what,
        "terrain model (Sample WCS)"
    );
}

/// The adapter offers what the manifest declares, and names the tree cadastres that drew trees
/// after its own credit.
#[test]
fn the_adapter_offers_the_services_of_its_manifest() {
    let text = format!("{}{NAS}{TREES}", sample());
    let manifest = parse(&text, Path::new("sample")).unwrap();
    let adapter = ManifestAdapter::new(manifest, &Square, Parse::Now, &backends()).unwrap();
    assert!(adapter.alkis().is_some());
    assert_eq!(adapter.nas().unwrap().region.as_str(), "sample");
    assert_eq!(adapter.tree_datasets().len(), 1);
    assert_eq!(adapter.tree_datasets()[0].region.as_str(), "sample");
    let registers = ["city"];
    let used = UsedSources {
        alkis: vec![AlkisUse {
            regions: vec!["sample".parse().unwrap()],
            fetched: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        }],
        drawn_registers: &registers,
        ..UsedSources::default()
    };
    assert_eq!(
        adapter.attribution(&used),
        [
            CreditLine::new("© Sample 2026, dl-de/by-2-0"),
            CreditLine::new("City, dl-de/zero-2-0")
        ]
    );
    // Trees alone: their line alone. No trees drawn: no line of theirs.
    let trees_only = UsedSources {
        drawn_registers: &registers,
        ..UsedSources::default()
    };
    assert_eq!(
        adapter.attribution(&trees_only),
        [CreditLine::new("City, dl-de/zero-2-0")]
    );
    let none = UsedSources {
        alkis: used.alkis.clone(),
        ..UsedSources::default()
    };
    assert_eq!(adapter.attribution(&none).len(), 1);
    // A manifest without these tables offers none of it.
    let hamburg = parse(&valid("hamburg"), Path::new("hamburg")).unwrap();
    let hamburg = ManifestAdapter::new(hamburg, &Square, Parse::Now, &backends()).unwrap();
    assert!(hamburg.alkis().is_some());
    assert!(hamburg.nas().is_none() && hamburg.tree_datasets().is_empty());
}
