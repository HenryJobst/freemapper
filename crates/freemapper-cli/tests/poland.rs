//! A map in Poland (ADR 0161): the region `polska` has OSM extracts only, so a project there
//! is made from OSM alone, without templates, and offline from a cached extract.

use std::path::Path;
use std::process::Command;

mod isolation;
use isolation::isolated;

const AREA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pl/schoolyard.geojson"
);
const EXTRACT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pl/lodzkie_2026-11-19.osm.pbf"
);

fn freemapper() -> Command {
    let mut command = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")));
    command.env("FREEMAPPER_LANG", "en");
    command
}

/// A cache folder that holds the extract of Łódź Voivodeship, as an earlier online run leaves it.
fn cache_with_extract(dir: &Path) -> std::path::PathBuf {
    let cache = dir.join("cache");
    std::fs::create_dir_all(cache.join("osm")).unwrap();
    std::fs::copy(EXTRACT, cache.join("osm/lodzkie_2026-11-19.osm.pbf")).unwrap();
    cache
}

#[test]
fn osm_only_region_makes_a_project_without_templates() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("Szkoła Podstawowa nr 5, Łódź");
    let cache = cache_with_extract(dir.path());
    let init = freemapper()
        .args(["init"])
        .arg(&project)
        .args(["--geojson", AREA, "--kind", "schoolyard"])
        .args(["--title", "Szkoła Podstawowa nr 5, Łódź"])
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let run = freemapper()
        .arg("run")
        .arg(&project)
        .args(["--offline", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{stdout}\n{stderr}");
    let omap = std::fs::read_dir(&project)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "omap"))
        .expect("a map was written");
    let text = std::fs::read_to_string(&omap).unwrap();
    // The zone of Łódź is 34, the title keeps its diacritics, and the credit names OSM only.
    assert!(text.contains("25834"), "{text}");
    assert!(text.contains("Szkoła Podstawowa nr 5, Łódź"), "{text}");
    let attribution = std::fs::read_to_string(project.join("ATTRIBUTION.txt")).unwrap();
    assert!(attribution.contains("OpenStreetMap"), "{attribution}");
    assert!(!attribution.contains("GUGiK"), "{attribution}");
    // No raster template: nothing of a terrain model or an orthophoto exists for this region yet.
    for template in [
        "orthophoto.jpg",
        "hillshade.png",
        "slope.png",
        "relief.png",
        "ndsm.png",
    ] {
        assert!(
            !project.join("templates").join(template).exists(),
            "{template}"
        );
    }
    assert!(
        stderr.contains("no terrain data for this region yet")
            || stdout.contains("no terrain data for this region yet"),
        "{stdout}\n{stderr}"
    );
}
