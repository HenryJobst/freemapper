// SPDX-License-Identifier: AGPL-3.0-or-later

//! A map in Poland (ADR 0164, 0170): the region `polska` has OSM extracts and the rasters of
//! GUGiK, and no cadastre.

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

/// A project in Łódź keeps its zone and its title, and an offline run with only the OSM extract in
/// the cache stops where the sheets of GUGiK are missing, and says to fetch online first.
#[test]
fn a_polish_project_needs_the_sheets_of_gugik_for_an_offline_run() {
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
    let toml = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    // The zone of Łódź is 34 and the title keeps its diacritics.
    assert!(toml.contains("25834"), "{toml}");
    assert!(toml.contains("Szkoła Podstawowa nr 5, Łódź"), "{toml}");
    let run = freemapper()
        .arg("run")
        .arg(&project)
        .args(["--offline", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(1), "{run:?}");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("no terrain model stored") && stderr.contains("freemapper fetch"),
        "{stderr}"
    );
}

/// The estimate of a map in Łódź names the one OSM extract of its voivodeship, the sheets of
/// GUGiK and the buildings and land use of its cadastre, and the extract of a recent run in the
/// cache is not downloaded again; the rest still is.
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
    let (extracts, sheets): (Vec<_>, Vec<_>) = estimate
        .items
        .iter()
        .partition(|item| item.what.contains("Voivodeship"));
    // The cadastre is no sheet of GUGiK's rasters: its item names EGiB and not ALKIS.
    let (cadastre, sheets): (Vec<_>, Vec<_>) = sheets
        .into_iter()
        .partition(|item| item.what.contains("EGiB"));
    assert_eq!(cadastre.len(), 1, "{:?}", estimate.items);
    assert!(cadastre.iter().all(|item| !item.what.contains("ALKIS")));
    assert_eq!(extracts.len(), 1, "{:?}", estimate.items);
    assert!(
        extracts[0].what.contains("Łódź Voivodeship"),
        "{}",
        extracts[0].what
    );
    assert!(
        (extracts[0].megabytes - 132.9).abs() < 0.05,
        "{}",
        extracts[0].megabytes
    );
    assert!(!extracts[0].cached);
    assert!(
        !sheets.is_empty() && sheets.iter().all(|item| item.what.contains("GUGiK")),
        "{:?}",
        estimate.items
    );
    let warm = Cache::new(cache_with_extract(dir.path()));
    let estimate = freemapper::layers::estimate_download(&project, &warm, today).unwrap();
    for item in &estimate.items {
        assert_eq!(item.cached, item.what.contains("Voivodeship"), "{item:?}");
        assert!(!item.what.contains("ALKIS"), "{item:?}");
    }
    let sheets: f64 = estimate
        .items
        .iter()
        .filter(|item| !item.cached)
        .map(|item| item.megabytes)
        .sum();
    assert!((estimate.total_megabytes - sheets).abs() < 0.05);
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

/// The permit notes of a map in Poland come from Polish services (ADR 0175) and name no German
/// unit or authority. Offline with nothing stored, the run says what is missing: OSM alone, as
/// VG250 and the BfN are not asked there.
#[test]
fn permit_notes_of_a_polish_map_need_stored_inputs_offline() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_over(dir.path(), "lodz", 19.450, 51.760, 19.460, 51.766);
    let out = freemapper()
        .arg("permits")
        .arg(&project)
        .arg("--offline")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no source of the permit notes could be read"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("not available for this region"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("vg250") && !stderr.contains("bfn"),
        "{stderr}"
    );
    assert!(!project.join("permits.md").exists());
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
