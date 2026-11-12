//! The adapter a manifest gives: identity, boundary, OSM extracts and credit.

use std::borrow::Cow;
use std::path::Path;

use chrono::NaiveDate;
use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::credit::{AlkisUse, CreditLine, UsedSources};
use freemapper_georef::crs::{Crs, LatLon, ProjectedPoint, utm33_inverse};
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::boundary::{MAX_BOUNDARY_BYTES, load_boundary, parse_boundary};
use freemapper_manifest::model::BoundaryCrs;
use freemapper_manifest::{ManifestError, parse};
use geo_types::MultiPolygon;

fn repo(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn manifest(id: &str) -> freemapper_manifest::Manifest {
    let path = format!("crates/freemapper-manifest/tests/valid/{id}.toml");
    parse(&repo(&path), Path::new(id)).unwrap()
}

/// The files of an adapter folder, by name.
struct Files(Vec<(&'static str, String)>);

impl AdapterFiles for Files {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        self.0
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, text)| Cow::Owned(text.clone()))
            .ok_or_else(|| "no such file".to_string())
    }
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

fn hamburg_files() -> Files {
    Files(vec![(
        "boundary.geojson",
        repo("assets/regions/hamburg.geojson"),
    )])
}

fn code(findings: &[ManifestError]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.code).collect()
}

/// A square of 1 km in EPSG:25833 as a boundary file, with `ring` as its outer ring.
fn file_of(ring: &[(f64, f64)]) -> String {
    let points: Vec<String> = ring.iter().map(|(x, y)| format!("[{x},{y}]")).collect();
    format!(
        r#"{{"type":"FeatureCollection","features":[{{"type":"Feature","properties":{{}},
        "geometry":{{"type":"Polygon","coordinates":[[{}]]}}}}]}}"#,
        points.join(",")
    )
}

const SQUARE: [(f64, f64); 5] = [
    (400_000.0, 5_800_000.0),
    (401_000.0, 5_800_000.0),
    (401_000.0, 5_801_000.0),
    (400_000.0, 5_801_000.0),
    (400_000.0, 5_800_000.0),
];

fn vertices(boundary: &MultiPolygon<f64>) -> Vec<(f64, f64)> {
    boundary
        .0
        .iter()
        .flat_map(|polygon| {
            std::iter::once(polygon.exterior())
                .chain(polygon.interiors())
                .flat_map(|ring| ring.coords().map(|c| (c.x, c.y)))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A boundary may come in longitude and latitude. Hamburg's file, turned into EPSG:4326 and
/// read back, is the boundary of the EPSG:25833 file to a metre.
#[test]
fn boundary_in_4326_equals_the_25833_boundary_within_1_m() {
    let path = Path::new("hamburg/boundary.geojson");
    let text = repo("assets/regions/hamburg.geojson");
    let utm = parse_boundary(&text, BoundaryCrs::Utm33, path).unwrap();
    assert!(vertices(&utm).len() > 100);
    // The same file with every position in degrees.
    let mut collection: geojson::FeatureCollection = text.parse().unwrap();
    let degrees = utm.clone().0.into_iter().map(|polygon| {
        let ring = |ring: &geo_types::LineString<f64>| -> Vec<Vec<f64>> {
            ring.coords()
                .map(|c| {
                    let LatLon { lat, lon } = utm33_inverse(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .unwrap();
                    vec![lon, lat]
                })
                .collect()
        };
        std::iter::once(ring(polygon.exterior()))
            .chain(polygon.interiors().iter().map(ring))
            .collect::<Vec<_>>()
    });
    collection.features[0].geometry = Some(geojson::Geometry::new(
        geojson::GeometryValue::new_multi_polygon(degrees),
    ));
    let wgs = parse_boundary(&collection.to_string(), BoundaryCrs::Wgs84, path).unwrap();
    let (a, b) = (vertices(&utm), vertices(&wgs));
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(&b) {
        assert!((a.0 - b.0).hypot(a.1 - b.1) < 1.0, "{a:?} {b:?}");
    }
}

#[test]
fn self_crossing_boundary_is_rejected() {
    let path = Path::new("sample/boundary.geojson");
    assert!(parse_boundary(&file_of(&SQUARE), BoundaryCrs::Utm33, path).is_ok());
    // A bow tie: the ring crosses itself.
    let bow_tie = [SQUARE[0], SQUARE[2], SQUARE[1], SQUARE[3], SQUARE[0]];
    let refused = parse_boundary(&file_of(&bow_tie), BoundaryCrs::Utm33, path).unwrap_err();
    assert_eq!(refused.code, "boundary.invalid");
    assert_eq!(refused.path, path);
    assert!(refused.message.contains("valid"), "{}", refused.message);
}

/// Review focus 5: a boundary whose positions cannot be of the CRS the manifest states, and
/// files that are no boundary.
#[test]
fn a_boundary_in_the_wrong_crs_or_shape_is_rejected() {
    let path = Path::new("sample/boundary.geojson");
    let degrees = [
        (9.9, 53.5),
        (10.1, 53.5),
        (10.1, 53.6),
        (9.9, 53.6),
        (9.9, 53.5),
    ];
    let refused = |text: &str, crs| parse_boundary(text, crs, path).unwrap_err().code;
    // Degrees stated as metres, and metres stated as degrees.
    assert_eq!(
        refused(&file_of(&degrees), BoundaryCrs::Utm33),
        "boundary.crs"
    );
    assert_eq!(
        refused(&file_of(&SQUARE), BoundaryCrs::Wgs84),
        "boundary.crs"
    );
    assert!(parse_boundary(&file_of(&degrees), BoundaryCrs::Wgs84, path).is_ok());
    assert_eq!(refused("{", BoundaryCrs::Utm33), "boundary.format");
    assert_eq!(refused("[1, 2]", BoundaryCrs::Utm33), "boundary.format");
    let point = r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},
        "geometry":{"type":"Point","coordinates":[400000,5800000]}}]}"#;
    assert_eq!(refused(point, BoundaryCrs::Utm33), "boundary.format");
    let none = r#"{"type":"FeatureCollection","features":[]}"#;
    assert_eq!(refused(none, BoundaryCrs::Utm33), "boundary.format");
    // Two features: which of them is the region? One is asked for.
    let two = file_of(&SQUARE).replacen(
        r#"{"type":"Feature""#,
        r#"{"type":"Feature","properties":{},"geometry":null},{"type":"Feature""#,
        1,
    );
    assert_eq!(refused(&two, BoundaryCrs::Utm33), "boundary.format");
    let empty = r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},
        "geometry":{"type":"MultiPolygon","coordinates":[]}}]}"#;
    assert_eq!(refused(empty, BoundaryCrs::Utm33), "boundary.format");
}

#[test]
fn a_boundary_file_has_at_most_2_mib() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("boundary.geojson");
    std::fs::write(&file, file_of(&SQUARE)).unwrap();
    assert!(load_boundary(&file, BoundaryCrs::Utm33).is_ok());
    let padded = format!("{}{}", file_of(&SQUARE), " ".repeat(MAX_BOUNDARY_BYTES));
    std::fs::write(&file, padded).unwrap();
    let refused = load_boundary(&file, BoundaryCrs::Utm33).unwrap_err();
    assert_eq!((refused.code, &refused.path), ("boundary.too-large", &file));
    let missing = load_boundary(&dir.path().join("none.geojson"), BoundaryCrs::Utm33);
    assert_eq!(missing.unwrap_err().code, "manifest.file-missing");
}

#[test]
fn geofabrik_urls_equal_the_osm_rs_constants_for_hamburg_and_hessen() {
    let adapter = ManifestAdapter::new(
        manifest("hamburg"),
        &hamburg_files(),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    let info = adapter.info();
    assert_eq!(info.osm.len(), 1);
    let extract = &info.osm[0];
    assert_eq!(
        extract.url(),
        "https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf"
    );
    assert_eq!(
        extract.state_url(),
        "https://download.geofabrik.de/europe/germany/hamburg-updates/state.txt"
    );
    assert_eq!(
        (
            extract.prefix.as_str(),
            extract.megabytes,
            extract.label.as_str()
        ),
        ("hamburg_", 54.0, "Hamburg")
    );
    // Hesse: the same manifest with another path, as the sample of the design says.
    let hessen = repo("crates/freemapper-manifest/tests/valid/hamburg.toml")
        .replace("europe/germany/hamburg", "europe/germany/hessen");
    let adapter = ManifestAdapter::new(
        parse(&hessen, Path::new("hamburg")).unwrap(),
        &hamburg_files(),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    assert_eq!(
        adapter.info().osm[0].url(),
        "https://download.geofabrik.de/europe/germany/hessen-latest.osm.pbf"
    );
    assert_eq!(adapter.info().osm[0].prefix, "hessen_");
}

/// What the manifest says about the region is what the adapter answers.
#[test]
fn the_adapter_has_the_facts_of_its_manifest() {
    let adapter = ManifestAdapter::new(
        manifest("hamburg"),
        &hamburg_files(),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    let info = adapter.info();
    assert_eq!(info.id.as_str(), "hamburg");
    assert_eq!(info.provider.as_str(), "lgv");
    assert_eq!(
        (info.names.en.as_str(), info.names.de.as_str()),
        ("Hamburg", "Hamburg")
    );
    assert_eq!(info.crs, Crs::Utm32);
    assert_eq!(info.object_id_prefix.as_deref(), Some("DEHH"));
    assert_eq!(info.credit_short, None);
    let preview = info.preview.as_ref().unwrap();
    assert_eq!(
        preview.wms,
        "https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt"
    );
    assert_eq!(preview.params, [("time".to_string(), "2026".to_string())]);
    assert_eq!(
        (preview.format.as_str(), preview.agency.as_str()),
        ("image/jpeg", "LGV")
    );
    // The boundary is the one of the file, and the box around it is kept.
    let boundary = parse_boundary(
        &repo("assets/regions/hamburg.geojson"),
        BoundaryCrs::Utm33,
        Path::new("x"),
    )
    .unwrap();
    assert_eq!(adapter.boundary(), &boundary);
    let bounds = adapter.bounds();
    let all = vertices(&boundary);
    let min_e = all.iter().map(|v| v.0).fold(f64::INFINITY, f64::min);
    let max_n = all.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
    assert_eq!((bounds.min_e, bounds.max_n), (min_e, max_n));
    assert!(adapter.extract_boundary(0).is_none());
    assert_eq!(adapter.manifest().adapter.revision, 1);
    // A manifest read on first use gives the same.
    let lazy = ManifestAdapter::new(
        manifest("hamburg"),
        &hamburg_files(),
        Parse::OnFirstUse,
        &backends(),
    )
    .unwrap();
    assert_eq!(lazy.boundary(), adapter.boundary());
    assert_eq!(lazy.info(), adapter.info());
    let st = manifest("saxonyanhalt");
    let files = Files(vec![(
        "boundary.geojson",
        repo("assets/regions/saxony-anhalt.geojson"),
    )]);
    let st = ManifestAdapter::new(st, &files, Parse::Now, &backends()).unwrap();
    assert_eq!(st.info().credit_short.as_deref(), Some("LVermGeo ST"));
}

/// North Rhine-Westphalia has five extracts, each with its own boundary.
#[test]
fn each_extract_has_its_boundary() {
    let districts = ["arnsberg", "detmold", "duesseldorf", "koeln", "muenster"];
    let mut files = vec![(
        "boundary.geojson",
        repo("assets/regions/nordrhein-westfalen.geojson"),
    )];
    let names = [
        "osm-arnsberg.geojson",
        "osm-detmold.geojson",
        "osm-duesseldorf.geojson",
        "osm-koeln.geojson",
        "osm-muenster.geojson",
    ];
    for (name, district) in names.into_iter().zip(districts) {
        files.push((
            name,
            repo(&format!("assets/regions/osm/nrw-{district}.geojson")),
        ));
    }
    let adapter = ManifestAdapter::new(
        manifest("nordrheinwestfalen"),
        &Files(files),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    assert_eq!(adapter.info().osm.len(), 5);
    assert_eq!(adapter.info().osm[4].prefix, "nrw-muenster_");
    assert_eq!(
        adapter.info().osm[3].url(),
        "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/koeln-regbez-latest.osm.pbf"
    );
    for index in 0..5 {
        let boundary = adapter.extract_boundary(index).unwrap();
        assert!(!boundary.0.is_empty(), "{index}");
        assert_ne!(boundary, adapter.boundary());
    }
    assert!(adapter.extract_boundary(5).is_none());
}

/// A folder that lacks a file, or holds a broken one, gives no adapter; every file is named.
#[test]
fn missing_and_broken_files_are_findings() {
    let missing = ManifestAdapter::new(
        manifest("hamburg"),
        &Files(Vec::new()),
        Parse::Now,
        &backends(),
    );
    let findings = missing.err().unwrap();
    assert_eq!(code(&findings), ["manifest.file-missing"]);
    assert_eq!(
        findings[0].path,
        Path::new("hamburg").join("boundary.geojson")
    );
    // On first use the file is still read when the adapter is made.
    let lazy = ManifestAdapter::new(
        manifest("hamburg"),
        &Files(Vec::new()),
        Parse::OnFirstUse,
        &backends(),
    );
    assert_eq!(code(&lazy.err().unwrap()), ["manifest.file-missing"]);
    let broken = Files(vec![("boundary.geojson", "{".to_string())]);
    let findings = ManifestAdapter::new(manifest("hamburg"), &broken, Parse::Now, &backends())
        .err()
        .unwrap();
    assert_eq!(code(&findings), ["boundary.format"]);
    // Both broken files of a region with several are reported.
    let files = Files(vec![
        ("boundary.geojson", "{".to_string()),
        ("osm-arnsberg.geojson", "[]".to_string()),
    ]);
    let findings = ManifestAdapter::new(
        manifest("nordrheinwestfalen"),
        &files,
        Parse::Now,
        &backends(),
    )
    .err()
    .unwrap();
    assert_eq!(
        code(&findings),
        [
            "boundary.format",
            "boundary.format",
            "manifest.file-missing",
            "manifest.file-missing",
            "manifest.file-missing",
            "manifest.file-missing"
        ]
    );
}

fn day(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 6, 15).unwrap()
}

/// The credit is due once something of the region was fetched, with the year of the newest
/// fetch in place of `{year}`.
#[test]
fn credit_text_substitutes_year_once() {
    let adapter = ManifestAdapter::new(
        manifest("hamburg"),
        &hamburg_files(),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    assert!(adapter.attribution(&UsedSources::default()).is_empty());
    let alkis = |region: &str, year| AlkisUse {
        regions: vec![region.parse().unwrap()],
        fetched: day(year),
    };
    let used = UsedSources {
        alkis: vec![
            alkis("hamburg", 2025),
            alkis("hamburg", 2026),
            alkis("hessen", 2027),
        ],
        ..UsedSources::default()
    };
    assert_eq!(
        adapter.attribution(&used),
        [CreditLine::new(
            "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) 2026, \
             dl-de/by-2-0, Daten geändert"
        )]
    );
    // Objects of another region alone: no line of this one.
    let other = UsedSources {
        alkis: vec![alkis("hessen", 2027)],
        ..UsedSources::default()
    };
    assert!(adapter.attribution(&other).is_empty());
    // A licence note goes with the line; a credit without {year} stays as it is.
    let text = repo("crates/freemapper-manifest/tests/valid/hamburg.toml").replace(
        "(LGV) {year}, dl-de/by-2-0, Daten geändert\"",
        "(LGV)\"\nnote = \"Lizenz: https://www.govdata.de/dl-de/by-2-0\"",
    );
    let adapter = ManifestAdapter::new(
        parse(&text, Path::new("hamburg")).unwrap(),
        &hamburg_files(),
        Parse::Now,
        &backends(),
    )
    .unwrap();
    assert_eq!(
        adapter.attribution(&used),
        [CreditLine::with_licence(
            "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV)",
            "Lizenz: https://www.govdata.de/dl-de/by-2-0"
        )]
    );
}
