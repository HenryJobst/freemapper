// SPDX-License-Identifier: AGPL-3.0-or-later

//! A map in Poland (ADR 0164): the region `polska` has OSM extracts only, so a project there
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

/// The estimate of a map in Łódź names the one OSM extract of its voivodeship and no raster, and
/// the extract of a recent run in the cache is not downloaded again.
#[test]
fn estimate_lists_one_extract_for_lodz() {
    use freemapper::fetch::Cache;
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let init = freemapper()
        .arg("init")
        .arg(&project)
        .args(["--geojson", AREA])
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let today = chrono::NaiveDate::from_ymd_opt(2026, 11, 20).unwrap();
    let empty = Cache::new(dir.path().join("empty-cache"));
    let estimate = freemapper::layers::estimate_download(&project, &empty, today).unwrap();
    assert_eq!(estimate.items.len(), 1, "{:?}", estimate.items);
    let item = &estimate.items[0];
    assert!(item.what.contains("Łódź Voivodeship"), "{}", item.what);
    assert!((item.megabytes - 132.9).abs() < 0.05, "{}", item.megabytes);
    assert!(!item.cached);
    assert!((estimate.total_megabytes - 132.9).abs() < 0.05);
    let warm = Cache::new(cache_with_extract(dir.path()));
    let estimate = freemapper::layers::estimate_download(&project, &warm, today).unwrap();
    assert!(
        estimate.items.iter().all(|item| item.cached),
        "{:?}",
        estimate.items
    );
    assert!(estimate.total_megabytes < 0.01);
}

/// A GeoJSON rectangle (lon/lat) in `dir`, and the project made from it.
fn project_over(
    dir: &Path,
    name: &str,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
) -> std::path::PathBuf {
    let area = dir.join(format!("{name}.geojson"));
    std::fs::write(
        &area,
        format!(
            r#"{{"type":"Feature","properties":{{}},"geometry":{{"type":"Polygon","coordinates":[[[{west},{south}],[{east},{south}],[{east},{north}],[{west},{north}],[{west},{south}]]]}}}}"#
        ),
    )
    .unwrap();
    let project = dir.join(name);
    let init = freemapper()
        .arg("init")
        .arg(&project)
        .arg("--geojson")
        .arg(&area)
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    project
}

fn run_offline(project: &Path, cache: &Path) -> std::process::Output {
    freemapper()
        .arg("run")
        .arg(project)
        .args(["--offline", "--cache-dir"])
        .arg(cache)
        .output()
        .unwrap()
}

/// Słubice on the east bank of the Oder with a strip of Frankfurt (Oder) on the west bank: the
/// centre lies in Poland, which has no data of Germany's kind, so the map would be half made.
/// `init` already says so.
#[test]
fn an_area_over_the_oder_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let area = dir.path().join("oder.geojson");
    std::fs::write(
        &area,
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":[[[14.550,52.346],[14.580,52.346],[14.580,52.352],[14.550,52.352],[14.550,52.346]]]}}"#,
    )
    .unwrap();
    let init = freemapper()
        .arg("init")
        .arg(dir.path().join("oder"))
        .arg("--geojson")
        .arg(&area)
        .output()
        .unwrap();
    assert_eq!(init.status.code(), Some(1), "{init:?}");
    let stderr = String::from_utf8_lossy(&init.stderr);
    assert!(
        stderr.contains("both sides of a country border"),
        "{stderr}"
    );
    assert!(!dir.path().join("oder").join("freemapper.toml").exists());
}

/// A box across the Olza at Cieszyn reaches into Czechia, for which freemapper has no source: it
/// is refused as well, not mapped up to the border only.
#[test]
fn an_area_over_the_olza_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let area = dir.path().join("olza.geojson");
    std::fs::write(
        &area,
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":[[[18.620,49.746],[18.650,49.746],[18.650,49.752],[18.620,49.752],[18.620,49.746]]]}}"#,
    )
    .unwrap();
    let init = freemapper()
        .arg("init")
        .arg(dir.path().join("olza"))
        .arg("--geojson")
        .arg(&area)
        .output()
        .unwrap();
    assert_eq!(init.status.code(), Some(1), "{init:?}");
    let stderr = String::from_utf8_lossy(&init.stderr);
    assert!(
        stderr.contains("both sides of a country border"),
        "{stderr}"
    );
}

/// The permit notes ask German services and name German authorities: for a map in Poland they
/// would say that no protected area lies there. They are refused until Poland has its own.
#[test]
fn permit_notes_are_refused_for_a_polish_map() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_over(dir.path(), "lodz", 19.450, 51.760, 19.460, 51.766);
    for offline in [true, false] {
        let mut cmd = freemapper();
        cmd.arg("permits").arg(&project);
        if offline {
            cmd.arg("--offline");
        }
        let out = cmd.output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("not available for this region yet"),
            "{stderr}"
        );
        assert!(!project.join("permits.md").exists());
    }
}

#[test]
fn an_area_wholly_in_slubice_is_not_refused_for_its_border() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let project = project_over(dir.path(), "slubice", 14.566, 52.346, 14.580, 52.352);
    let run = run_offline(&project, &cache);
    let stderr = String::from_utf8_lossy(&run.stderr);
    // No extract is cached, so the run stops there, not at the border.
    assert!(
        !stderr.contains("both sides of a country border"),
        "{stderr}"
    );
}

/// The same strip with its centre in Germany keeps working as before: Brandenburg maps it.
#[test]
fn a_border_area_centred_in_germany_still_belongs_to_germany() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let project = project_over(dir.path(), "ffo", 14.520, 52.340, 14.570, 52.346);
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("provider = \"lgb\""), "{text}");
    let run = run_offline(&project, &cache);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        !stderr.contains("both sides of a country border"),
        "{stderr}"
    );
    // The strip east of the Oder is Poland's: no terrain model there, and the map says so.
    assert!(
        stderr.contains("lies outside the supported states"),
        "{stderr}"
    );
}
