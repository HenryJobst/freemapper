//! The static checks of `freemapper adapters check`: what loads, and what its author is told.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::NaiveDate;
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::BoundingBox;
use freemapper_manifest::adapter::{EmbeddedFiles, ManifestAdapter, Parse};
use freemapper_manifest::check::{check_folder, covers};
use freemapper_manifest::model::Version;
use freemapper_manifest::parse;

const SQUARE: &str = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
    [[[400000,5800000],[410000,5800000],[410000,5810000],[400000,5810000],[400000,5800000]]]}}"#;

const RUNNING: Version = Version(0, 13, 0);

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
}

/// The smallest valid manifest, for the adapter `id`.
fn manifest(id: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(path)
        .unwrap()
        .replace("colour = \"red\"\n", "")
        .replace("id = \"sample\"", &format!("id = \"{id}\""))
}

/// Writes the folder of the adapter `id` with `manifest` and `boundary` into `dir`.
fn write(dir: &Path, id: &str, manifest: &str, boundary: &str) -> PathBuf {
    let folder = dir.join(id);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("manifest.toml"), manifest).unwrap();
    std::fs::write(folder.join("boundary.geojson"), boundary).unwrap();
    folder
}

/// The adapters of the program: one, `sample`, far from the square of the tests.
fn built_in() -> Registry {
    static FILES: [(&str, &str); 1] = [(
        "boundary.geojson",
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[600000,5600000],[610000,5600000],[610000,5610000],[600000,5610000],[600000,5600000]]]}}"#,
    )];
    let text =
        manifest("sample").replace("[adapter]\n", "[adapter]\nobject_id_prefix = \"DEXX\"\n");
    let manifest = parse(&text, Path::new("sample")).unwrap();
    let mut registry = Registry::new();
    registry
        .add(Arc::new(
            ManifestAdapter::new(
                manifest,
                &EmbeddedFiles(&FILES),
                Parse::OnFirstUse,
                &Registry::new(),
            )
            .unwrap(),
        ))
        .unwrap();
    registry
}

fn codes(folder: &Path) -> Result<Vec<&'static str>, Vec<&'static str>> {
    check_folder(folder, Registry::new(), &built_in(), RUNNING, today())
        .map(|checked| checked.warnings.iter().map(|w| w.code).collect())
        .map_err(|findings| findings.iter().map(|f| f.code).collect())
}

/// A folder that loads and was checked lately has nothing to say; the adapter is in the
/// registry alone, and a box is inside its boundary or not.
#[test]
fn a_fresh_folder_passes_without_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let text =
        manifest("mine").replace("[adapter]\n", "[adapter]\nlast_verified = \"2026-03-01\"\n");
    let folder = write(dir.path(), "mine", &text, SQUARE);
    let checked = check_folder(&folder, Registry::new(), &built_in(), RUNNING, today()).unwrap();
    assert!(checked.warnings.is_empty(), "{:?}", checked.warnings);
    assert_eq!(checked.registry.len(), 1);
    assert_eq!(checked.adapter().info().id.as_str(), "mine");
    let inside = BoundingBox {
        min_e: 404_000.0,
        min_n: 5_804_000.0,
        max_e: 404_150.0,
        max_n: 5_804_150.0,
    };
    assert!(covers(checked.adapter(), &inside));
    let across = BoundingBox {
        max_e: 410_100.0,
        ..inside
    };
    assert!(!covers(checked.adapter(), &across));
}

/// What the author is told about a folder that loads.
#[test]
fn warnings_name_what_the_author_should_know() {
    let dir = tempfile::tempdir().unwrap();
    // No day of the last check.
    let folder = write(dir.path(), "mine", &manifest("mine"), SQUARE);
    assert_eq!(codes(&folder), Ok(vec!["check.never-verified"]));
    // A check more than twelve months ago; one exactly twelve months ago still counts.
    for (day, expected) in [
        ("2025-11-11", vec!["check.verified-long-ago"]),
        ("2025-11-12", vec![]),
    ] {
        let text = manifest("mine").replace(
            "[adapter]\n",
            &format!("[adapter]\nlast_verified = \"{day}\"\n"),
        );
        let folder = write(dir.path(), "mine", &text, SQUARE);
        assert_eq!(codes(&folder), Ok(expected), "{day}");
    }
    let fresh = |id: &str| {
        manifest(id).replace("[adapter]\n", "[adapter]\nlast_verified = \"2026-03-01\"\n")
    };
    // The id and the object id prefix of an adapter of the program, and rules nobody reads.
    let folder = write(dir.path(), "sample", &fresh("sample"), SQUARE);
    std::fs::write(folder.join("rules.toml"), "").unwrap();
    assert_eq!(
        codes(&folder),
        Ok(vec!["adapter.rules-not-read", "check.built-in-id"])
    );
    let text = fresh("other").replace("[adapter]\n", "[adapter]\nobject_id_prefix = \"DEXX\"\n");
    let folder = write(dir.path(), "other", &text, SQUARE);
    assert_eq!(codes(&folder), Ok(vec!["check.prefix-shared"]));
    // A boundary of a few metres, and one on the ground of the built-in adapter.
    let tiny = SQUARE
        .replace("410000", "400010")
        .replace("5810000", "5800010");
    let folder = write(dir.path(), "tiny", &fresh("tiny"), &tiny);
    assert_eq!(codes(&folder), Ok(vec!["check.boundary-area"]));
    let on_sample = SQUARE
        .replace("4000", "6020")
        .replace("4100", "6080")
        .replace("58", "56");
    let folder = write(dir.path(), "inner", &fresh("inner"), &on_sample);
    assert_eq!(codes(&folder), Ok(vec!["check.shares-ground"]));
}

/// What keeps a folder out of the adapter folder of a person fails the check with the same
/// code.
#[test]
fn a_folder_that_would_not_load_fails_with_the_code_of_the_loader() {
    let dir = tempfile::tempdir().unwrap();
    // Longitude and latitude where the manifest says EPSG:25833.
    let degrees = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[9.7,53.4],[10.3,53.4],[10.3,53.7],[9.7,53.7],[9.7,53.4]]]}}"#;
    let folder = write(dir.path(), "degrees", &manifest("degrees"), degrees);
    assert_eq!(codes(&folder), Err(vec!["boundary.crs"]));
    // A folder whose name is not the id.
    let folder = write(dir.path(), "elsewhere", &manifest("mine"), SQUARE);
    assert_eq!(codes(&folder).unwrap_err().len(), 1);
    // No folder at all.
    assert_eq!(
        codes(&dir.path().join("nothing")),
        Err(vec!["adapter.folder"])
    );
}

#[cfg(unix)]
#[test]
fn a_linked_folder_fails_the_check() {
    let dir = tempfile::tempdir().unwrap();
    let real = write(dir.path(), "mine", &manifest("mine"), SQUARE);
    let link = dir.path().join("links").join("mine");
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    assert_eq!(codes(&link), Err(vec!["adapter.folder"]));
}
