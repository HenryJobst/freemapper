// SPDX-License-Identifier: AGPL-3.0-or-later

//! A map in Sweden from synthetic Lantmäteriet data (ADR 0203): the STAC catalogues, a Deflate
//! terrain tile and a land cover zip are answered from memory, the OSM extract is the empty one.
//! Nothing here is data of Lantmäteriet. The login comes from the two environment variables, which
//! a program cannot set in its own process without unsafe code, so the test runs itself again
//! with them set.

use std::io::Write as _;
use std::path::Path;
use std::process::Command;

use freemapper::area::MapArea;
use freemapper::fetch::{ByteRange, Cache, FetchError, FetchMode, Fetcher};
use freemapper::project::{
    MapKind, ProjectFile, derive_project, export_starter_project_from, fetch_project, init_project,
    starter_documents,
};
use freemapper_sources::raster::GridSpec;
use freemapper_sources::testing::tiled_float_tiff_with;

/// What the test puts in the variables; it must come out in no file and no message.
const USER: &str = "tester@example.org";
const PASSWORD: &str = "s3cret-Pw-4711";

const EMPTY_PBF: &[u8] = include_bytes!("../../../tests/golden/adapters/empty.osm.pbf");
const GPKG: &[u8] = include_bytes!("../../adapters/se/tests/fixtures/marktacke.gpkg");

const TILE_URL: &str = "https://dl1.lantmateriet.se/hojd/data/grid/mhm/61_3/m617_38.tif";
const ZIP_URL: &str = "https://dl1.lantmateriet.se/mark/marktacke/marktacke_kn0183.zip";

/// The terrain tile: 2 000 m square, west edge 384 000, north edge 6 178 500, 1 m cells, a slope.
fn tile() -> Vec<u8> {
    let grid = GridSpec {
        west: 384_000.0,
        north: 6_178_500.0,
        pixel_size: 1.0,
        width: 2000,
        height: 2000,
    };
    let values: Vec<f32> = (0..2000 * 2000)
        .map(|i| {
            let (x, y) = ((i % 2000) as f32, (i / 2000) as f32);
            100.0 + x * 0.02 + (y * 0.15).sin() * 2.0
        })
        .collect();
    tiled_float_tiff_with(&grid, &values, 256, 3, None, 8)
}

fn zipped() -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut out);
    zip.start_file(
        "marktacke_kn0183.gpkg",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(GPKG).unwrap();
    zip.finish().unwrap();
    out.into_inner()
}

/// The servers of the test.
struct Sweden {
    tile: Vec<u8>,
    zip: Vec<u8>,
    requests: std::sync::Mutex<Vec<String>>,
}

impl Sweden {
    fn answer(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.requests.lock().unwrap().push(url.to_string());
        let (dtm, cloud, land) = (
            "stac-hojd/v1/search?collections=dtm-cog",
            "stac-hojd/v1/search?collections=dsm-skoglig-copc",
            "stac-vektor/v1/search?collections=marktacke",
        );
        let item = |id: &str, href: &str| {
            format!(
                r#"{{"id":"{id}","collection":"c","properties":{{"datetime":"2025-09-29"}},"assets":{{"data":{{"href":"{href}"}}}}}}"#
            )
        };
        let body = |features: String| {
            format!(r#"{{"type":"FeatureCollection","features":[{features}],"links":[]}}"#)
                .into_bytes()
        };
        if url.ends_with("/state.txt") {
            Ok(b"timestamp=2026-11-26T21:21:00Z\nsequenceNumber=1\n".to_vec())
        } else if url.ends_with(".osm.pbf") {
            Ok(EMPTY_PBF.to_vec())
        } else if url.contains(dtm) {
            Ok(body(item("617_38", TILE_URL)))
        } else if url.contains(cloud) {
            Ok(body(String::new()))
        } else if url.contains(land) {
            Ok(body(item("0183", ZIP_URL)))
        } else if url == TILE_URL {
            Ok(self.tile.clone())
        } else if url == ZIP_URL {
            Ok(self.zip.clone())
        } else {
            Err(FetchError::NotFound(url.to_string()))
        }
    }
}

impl Fetcher for Sweden {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.answer(url)
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        let bytes = self.answer(url)?;
        let len = bytes.len() as u64;
        let (start, end) = match range {
            ByteRange::From { start, len: n } => (start.min(len), (start + n).min(len)),
            ByteRange::Last(n) => (len.saturating_sub(n), len),
        };
        Ok(bytes[start as usize..end as usize].to_vec())
    }
}

/// A 150 m square in the land cover of the fixture (EPSG:25833 385 030 to 385 180 by
/// 6 177 550 to 6 177 700), as a GeoJSON polygon in degrees.
fn area() -> MapArea {
    let corner = |e: f64, n: f64| {
        let at = freemapper::crs::utm33_inverse(freemapper::crs::ProjectedPoint {
            easting: e,
            northing: n,
        })
        .unwrap();
        [at.lon, at.lat]
    };
    let ring = [
        corner(385_030.0, 6_177_550.0),
        corner(385_180.0, 6_177_550.0),
        corner(385_180.0, 6_177_700.0),
        corner(385_030.0, 6_177_700.0),
        corner(385_030.0, 6_177_550.0),
    ];
    let polygon = serde_json::json!({"type": "Polygon", "coordinates": [ring]});
    MapArea::from_geojson(&polygon.to_string()).unwrap()
}

fn every_file(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_file(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// The work itself; it needs the two variables.
fn a_swedish_map_is_made_from_synthetic_lantmateriet_data() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    std::fs::create_dir_all(&project).unwrap();
    let cache = Cache::new(dir.path().join("cache"));
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
    init_project(&project, &area(), MapKind::Forest, None, date, None).unwrap();
    let server = Sweden {
        tile: tile(),
        zip: zipped(),
        requests: Default::default(),
    };
    let mut lines = Vec::new();
    fetch_project(&project, &server, &cache, FetchMode::Online, &mut |note| {
        lines.push(note.text(&freemapper::note::Lang::english(), &|e, _| e.to_string()));
    })
    .unwrap();
    let loaded = ProjectFile::load(&project).unwrap();
    assert!(project.join("data/dgm1.tif").exists(), "{lines:?}");
    assert!(loaded.terrain.is_some());
    assert!(loaded.orthophoto.is_none());
    derive_project(&project, &cache, &mut |note| {
        lines.push(note.text(&freemapper::note::Lang::english(), &|e, _| e.to_string()));
    })
    .unwrap();
    assert!(project.join("templates/hillshade.png").exists());
    assert!(!project.join("templates/orthophoto.jpg").exists());
    assert!(
        lines.iter().any(|l| l.contains("has no orthophoto")),
        "{lines:?}"
    );
    // The map is written without an orthophoto template.
    let documents = starter_documents(&project).unwrap();
    export_starter_project_from(&project, &documents, true).unwrap();
    let omap = std::fs::read_dir(&project)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "omap"))
        .expect("a map was written");
    let text = std::fs::read_to_string(omap).unwrap();
    assert!(
        !text.contains("orthophoto.jpg"),
        "the map names no orthophoto"
    );
    // The land cover came from the zip: the forest polygon is stored with its LGB class.
    let mut files = Vec::new();
    every_file(&project, &mut files);
    let stored_forest = files.iter().any(|path| {
        std::fs::read_to_string(path)
            .is_ok_and(|text| text.contains("id-forest") && text.contains("Wald"))
    });
    assert!(stored_forest, "{files:?}");
    // The login is in no file of the project and in no message.
    for path in &files {
        let bytes = std::fs::read(path).unwrap();
        assert!(
            !bytes
                .windows(PASSWORD.len())
                .any(|w| w == PASSWORD.as_bytes()),
            "{}",
            path.display()
        );
    }
    assert!(
        lines
            .iter()
            .all(|l| !l.contains(PASSWORD) && !l.contains(USER))
    );
}

#[test]
fn a_swedish_map_runs_with_the_login_in_the_environment() {
    if std::env::var("FREEMAPPER_LM_USER").is_ok() {
        a_swedish_map_is_made_from_synthetic_lantmateriet_data();
        return;
    }
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "a_swedish_map_runs_with_the_login_in_the_environment",
            "--nocapture",
        ])
        .env("FREEMAPPER_LM_USER", USER)
        .env("FREEMAPPER_LM_PASSWORD", PASSWORD)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}

#[test]
fn without_the_login_the_fetch_stops_before_a_request_and_names_the_variables() {
    if std::env::var("FREEMAPPER_LM_USER").is_ok() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    std::fs::create_dir_all(&project).unwrap();
    let cache = Cache::new(dir.path().join("cache"));
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
    init_project(&project, &area(), MapKind::Forest, None, date, None).unwrap();
    let server = Sweden {
        tile: tile(),
        zip: zipped(),
        requests: Default::default(),
    };
    let error =
        fetch_project(&project, &server, &cache, FetchMode::Online, &mut |_| {}).unwrap_err();
    let text = error.to_string();
    assert!(
        text.contains("FREEMAPPER_LM_USER") && text.contains("FREEMAPPER_LM_PASSWORD"),
        "{text}"
    );
    assert!(text.contains("Laserdata Skog"), "{text}");
    let asked: Vec<String> = server.requests.lock().unwrap().clone();
    assert!(
        asked.iter().all(|u| !u.contains("lantmateriet")),
        "{asked:?}"
    );
}
