//! The fixture `offline-school`: a fetched synthetic school project with its cache, so a run
//! works offline, and the stored inputs of the permit notes (ADR 0117).
#![allow(dead_code)] // Each test crate uses a part.

pub const OFFLINE_SCHOOL: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/offline-school");

pub fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// The offline school in `dir`: its project folder and its cache.
pub fn offline_school(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let fixture = std::path::Path::new(OFFLINE_SCHOOL);
    let (project, cache) = (dir.join("school"), dir.join("cache"));
    copy_tree(&fixture.join("project"), &project);
    copy_tree(&fixture.join("cache"), &cache);
    (project, cache)
}

/// Stored inputs of the permit notes as a fetch leaves them: VG250 read (Brandenburg and a
/// Gemeinde around the school), the BfN down, OSM not read.
pub fn store_permit_inputs(project: &std::path::Path) {
    let folder = project.join("data/permits");
    std::fs::create_dir_all(&folder).unwrap();
    let feature = |name: &str, kind: &str, ags: &str| {
        serde_json::json!({
            "type": "Feature",
            "properties": { "gen": name, "bez": kind, "ags": ags },
            "geometry": {
                "type": "Polygon",
                "coordinates": [[[13.8, 52.5], [13.9, 52.5], [13.9, 52.6], [13.8, 52.6], [13.8, 52.5]]]
            }
        })
    };
    let collection = |features: Vec<serde_json::Value>| {
        serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
    };
    for (level, features) in [
        ("land", vec![feature("Brandenburg", "Land", "12")]),
        (
            "kreis",
            vec![feature("Märkisch-Oderland", "Landkreis", "12064")],
        ),
        ("verwaltungsgemeinschaft", vec![]),
        ("gemeinde", vec![feature("Strausberg", "Stadt", "12064472")]),
    ] {
        std::fs::write(
            folder.join(format!("vg250-{level}.geojson")),
            collection(features),
        )
        .unwrap();
    }
    let sources = serde_json::json!({
        "schema": 1,
        "bbox": [13.84, 52.53, 13.85, 52.55],
        "vg250": { "state": "read", "fetched": "2026-11-13", "count": 3 },
        "bfn": { "state": "unreachable", "reason": "HTTP 503 Service Unavailable" },
        "osm": { "state": "not-read" }
    });
    std::fs::write(folder.join("sources.json"), sources.to_string()).unwrap();
}
