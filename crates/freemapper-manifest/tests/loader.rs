//! Adapters from folders: what loads, in which order, and what is left out with its findings.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::BoundingBox;
use freemapper_manifest::adapter::{EmbeddedFiles, ManifestAdapter, Parse};
use freemapper_manifest::loader::{
    AdapterDir, AdapterWarning, Loaded, MAX_FILES, Origin, load_dirs,
};
use freemapper_manifest::model::Version;
use freemapper_manifest::parse;

const SQUARE: &str = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
    [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#;

const RUNNING: Version = Version(0, 13, 0);

/// The smallest valid manifest, for the adapter `id`.
fn manifest(id: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/invalid/unknown-key.toml");
    std::fs::read_to_string(path)
        .unwrap()
        .replace("colour = \"red\"\n", "")
        .replace("id = \"sample\"", &format!("id = \"{id}\""))
}

/// Writes the folder of the adapter `id` with `manifest` and its boundary into `dir`.
fn write(dir: &Path, id: &str, manifest: &str) -> PathBuf {
    let folder = dir.join(id);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("manifest.toml"), manifest).unwrap();
    std::fs::write(folder.join("boundary.geojson"), SQUARE).unwrap();
    folder
}

fn user(dir: &Path) -> Vec<AdapterDir> {
    vec![AdapterDir {
        path: dir.to_path_buf(),
        origin: Origin::User,
    }]
}

/// An adapter built into the program, made of the sample manifest with the id `id`.
fn built_in(id: &str, text: &str, backends: &Registry) -> Arc<dyn RegionAdapter> {
    static FILES: [(&str, &str); 1] = [("boundary.geojson", SQUARE)];
    let manifest = parse(text, Path::new(id)).unwrap();
    Arc::new(
        ManifestAdapter::new(
            manifest,
            &EmbeddedFiles(&FILES),
            Parse::OnFirstUse,
            backends,
        )
        .unwrap(),
    )
}

/// The one warning of a load, which left its adapter out, with the code of its first finding.
fn skipped(warnings: &[AdapterWarning]) -> (&'static str, &AdapterWarning) {
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].skipped, "{warnings:?}");
    (warnings[0].findings[0].code, &warnings[0])
}

#[test]
fn folder_adapters_load_after_the_built_ins_in_the_order_of_their_ids() {
    let dir = tempfile::tempdir().unwrap();
    let zeta = write(dir.path(), "zeta", &manifest("zeta"));
    write(dir.path(), "alpha", &manifest("alpha"));
    // A file beside the folders is nobody's adapter.
    std::fs::write(dir.path().join("README.txt"), "notes").unwrap();
    let mut registry = Registry::new();
    registry
        .add(built_in("sample", &manifest("sample"), &Registry::new()))
        .unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(warnings.is_empty(), "{warnings:?}");
    let ids: Vec<&str> = registry.adapters().map(|a| a.info().id.as_str()).collect();
    assert_eq!(ids, ["sample", "alpha", "zeta"]);
    assert_eq!(
        loaded[1],
        Loaded {
            id: "zeta".to_string(),
            origin: Origin::User,
            folder: zeta,
            revision: 1,
            hosts: vec!["geo.example.org".to_string()],
            last_verified: None,
            // The three adapters lie on the same square.
            overlaps: vec!["sample".to_string(), "alpha".to_string()],
        }
    );
    let zeta = registry.get("zeta").unwrap();
    assert!(zeta.alkis().is_some() && zeta.rasters().is_some());
    assert_eq!(zeta.rasters().unwrap().provider().id.as_str(), "zeta");
    assert!(!zeta.boundary().0.is_empty());
    // An adapter from a folder is held to the hosts it declares, the hosts of OSM and the
    // place search not among them; one made without that rule is held to nothing.
    let policy = zeta.policy().expect("a folder adapter has a policy");
    assert!(policy.check_url("https://geo.example.org/wfs").is_ok());
    assert!(policy.check_url("https://download.geofabrik.de/x").is_err());
    assert!(registry.get("sample").unwrap().policy().is_none());
    // A folder of adapters that is not there is no finding; a file in its place is one.
    let nowhere = load_dirs(&mut registry, &user(&dir.path().join("none")), RUNNING);
    assert!(nowhere.0.is_empty() && nowhere.1.is_empty());
    let file = load_dirs(
        &mut registry,
        &user(&dir.path().join("README.txt")),
        RUNNING,
    );
    assert_eq!(skipped(&file.1).0, "adapter.folder");
}

/// An adapter that lies on the ground of another says so: both draw objects there. A neighbour
/// that only shares a border does not.
#[test]
fn an_adapter_on_the_ground_of_another_names_it() {
    let dir = tempfile::tempdir().unwrap();
    let square = |min_e: u32, max_e: u32| {
        format!(
            r#"{{"type":"Feature","properties":{{}},"geometry":{{"type":"Polygon","coordinates":
            [[[{min_e},5800000],[{max_e},5800000],[{max_e},5801000],[{min_e},5801000],[{min_e},5800000]]]}}}}"#
        )
    };
    // `inner` lies inside the square of `sample`, `neighbour` beside it with a shared border,
    // `wide` around all of them.
    for (id, min_e, max_e) in [
        ("inner", 400_200, 400_400),
        ("neighbour", 401_000, 402_000),
        ("wide", 399_000, 403_000),
    ] {
        let folder = write(dir.path(), id, &manifest(id));
        std::fs::write(folder.join("boundary.geojson"), square(min_e, max_e)).unwrap();
    }
    let mut registry = Registry::new();
    registry
        .add(built_in("sample", &manifest("sample"), &Registry::new()))
        .unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(warnings.is_empty(), "{warnings:?}");
    let overlaps: Vec<(&str, Vec<&str>)> = loaded
        .iter()
        .map(|adapter| {
            let others = adapter.overlaps.iter().map(String::as_str).collect();
            (adapter.id.as_str(), others)
        })
        .collect();
    assert_eq!(
        overlaps,
        [
            ("inner", vec!["sample"]),
            ("neighbour", vec![]),
            ("wide", vec!["sample", "inner", "neighbour"]),
        ]
    );
}

/// Sharing ground is measured, not sampled: an adapter that reaches a third of the way into
/// another one shares its ground, although the middle of neither lies inside the other. Two
/// neighbours whose simplified borders overlap by a strip of a few metres do not.
#[test]
fn ground_is_shared_by_area_not_by_the_middle_of_a_region() {
    let dir = tempfile::tempdir().unwrap();
    let square = |min_e: u32, max_e: u32| {
        format!(
            r#"{{"type":"Feature","properties":{{}},"geometry":{{"type":"Polygon","coordinates":
            [[[{min_e},5800000],[{max_e},5800000],[{max_e},5801000],[{min_e},5801000],[{min_e},5800000]]]}}}}"#
        )
    };
    // `sample` is the square from 400 000 to 401 000. `partly` reaches 300 m into it from the
    // west, `strip` 5 m from the east.
    for (id, min_e, max_e) in [("partly", 399_500, 400_300), ("strip", 400_995, 402_000)] {
        let folder = write(dir.path(), id, &manifest(id));
        std::fs::write(folder.join("boundary.geojson"), square(min_e, max_e)).unwrap();
    }
    let mut registry = Registry::new();
    registry
        .add(built_in("sample", &manifest("sample"), &Registry::new()))
        .unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(warnings.is_empty(), "{warnings:?}");
    let overlaps: Vec<(&str, Vec<&str>)> = loaded
        .iter()
        .map(|adapter| {
            let others = adapter.overlaps.iter().map(String::as_str).collect();
            (adapter.id.as_str(), others)
        })
        .collect();
    assert_eq!(overlaps, [("partly", vec!["sample"]), ("strip", vec![])]);
}

/// A boundary whose ring crosses itself is not a region: the folder is left out with a finding
/// about the boundary, and the measure of shared ground never sees it.
#[test]
fn a_boundary_that_crosses_itself_is_left_out() {
    let dir = tempfile::tempdir().unwrap();
    // A bow tie over the west edge of `sample` (the square from 400 000 to 401 000).
    let bow_tie = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[399500,5800000],[400800,5801000],[400800,5800000],[399500,5801000],[399500,5800000]]]}}"#;
    let folder = write(dir.path(), "bowtie", &manifest("bowtie"));
    std::fs::write(folder.join("boundary.geojson"), bow_tie).unwrap();
    let mut registry = Registry::new();
    registry
        .add(built_in("sample", &manifest("sample"), &Registry::new()))
        .unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(loaded.is_empty(), "{loaded:?}");
    let (code, _) = skipped(&warnings);
    assert_eq!(code, "boundary.invalid");
}

/// Review focus 4: an id is taken once. A folder cannot replace an adapter the program has,
/// nor one loaded before it.
#[test]
fn folder_adapter_cannot_replace_a_builtin() {
    let dir = tempfile::tempdir().unwrap();
    let changed = manifest("sample").replace("Sample\", de", "Replaced\", de");
    assert_ne!(changed, manifest("sample"));
    let folder = write(dir.path(), "sample", &changed);
    let mut registry = Registry::new();
    registry
        .add(built_in("sample", &manifest("sample"), &Registry::new()))
        .unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(loaded.is_empty());
    let (code, warning) = skipped(&warnings);
    assert_eq!(code, "adapter.duplicate");
    assert_eq!(warning.folder, folder);
    assert!(
        warning.findings[0].message.contains("cannot replace"),
        "{warning:?}"
    );
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.get("sample").unwrap().info().names.en, "Sample");
    // The same folder in a second place (a project) is refused as well.
    let mut registry = Registry::new();
    let twice = [user(dir.path()), user(dir.path())].concat();
    let (loaded, warnings) = load_dirs(&mut registry, &twice, RUNNING);
    assert_eq!(loaded.len(), 1);
    assert_eq!(skipped(&warnings).0, "adapter.duplicate");
}

#[test]
fn broken_folder_adapter_is_skipped_with_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    let broken = manifest("broken").replace(
        "https://geo.example.org/dop/wms",
        "http://geo.example.org/dop/wms",
    );
    let folder = write(dir.path(), "broken", &broken);
    write(dir.path(), "fine", &manifest("fine"));
    // A boundary that crosses itself, a folder whose name is not the id, and one without a
    // manifest.
    let bow_tie = write(dir.path(), "bowtie", &manifest("bowtie"));
    std::fs::write(
        bow_tie.join("boundary.geojson"),
        SQUARE.replace(
            "[401000,5800000],[401000,5801000]",
            "[401000,5801000],[401000,5800000]",
        ),
    )
    .unwrap();
    write(dir.path(), "othername", &manifest("named"));
    std::fs::create_dir(dir.path().join("empty")).unwrap();
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["fine"]
    );
    let found: Vec<(String, &str, bool)> = warnings
        .iter()
        .map(|w| {
            (
                w.folder.file_name().unwrap().to_string_lossy().into_owned(),
                w.findings[0].code,
                w.skipped,
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("bowtie".to_string(), "boundary.invalid", true),
            ("broken".to_string(), "policy.url", true),
            ("empty".to_string(), "manifest.file-missing", true),
            ("othername".to_string(), "manifest.id", true),
        ]
    );
    let finding = &warnings[1].findings[0];
    assert_eq!(finding.path, folder.join("manifest.toml"));
    let line = broken.lines().nth(finding.line - 1).unwrap();
    assert_eq!(line, "url = \"http://geo.example.org/dop/wms\"");
    assert_eq!(finding.key, "orthophoto.url");
    // The boundary is named by its place on the disk.
    assert_eq!(
        warnings[0].findings[0].path,
        bow_tie.join("boundary.geojson")
    );
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

/// The sample manifest with the backend `shared` for its rasters.
fn with_backend(id: &str, provider: &str) -> String {
    let text = manifest(id).replace(
        "revision = 1\n",
        &format!("revision = 1\nprovider = \"{provider}\"\n"),
    );
    let cut = text.find("[orthophoto]").unwrap();
    format!(
        "{}[orthophoto]\nbackend = \"shared\"\n\n[terrain]\nbackend = \"shared\"\n",
        &text[..cut]
    )
}

/// Review focus 4: two adapters with one provider share its cache folder and its record. That
/// is right for a backend both name, and refused for services of an adapter's own.
#[test]
fn shared_provider_shares_the_cache_folder() {
    let mut registry = Registry::new();
    registry
        .add_backend(
            "shared",
            Arc::new(Named(ProviderInfo {
                id: "agency".parse().unwrap(),
                cache_dir: Some("agency_tiles".to_string()),
                terrain_source: "agency-dgm1".to_string(),
                kind: RasterKind::Tiles { grid_km: 1 },
            })),
        )
        .unwrap();
    let first = built_in("first", &with_backend("first", "agency"), &registry);
    registry.add(first).unwrap();
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "second", &with_backend("second", "agency"));
    // Services of its own under the provider of another adapter.
    let own = manifest("third").replace("revision = 1\n", "revision = 1\nprovider = \"agency\"\n");
    write(dir.path(), "third", &own);
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["second"]
    );
    assert_eq!(skipped(&warnings).0, "adapter.provider-shared");
    let provider = |id: &str| {
        registry
            .get(id)
            .unwrap()
            .rasters()
            .unwrap()
            .provider()
            .clone()
    };
    assert_eq!(provider("first"), provider("second"));
    assert_eq!(
        provider("second").cache_dir.as_deref(),
        Some("agency_tiles")
    );
}

#[test]
fn an_adapter_for_a_newer_freemapper_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "future",
        &manifest("future").replace("\"0.13.0\"", "\"0.14.0\""),
    );
    write(dir.path(), "current", &manifest("current"));
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(loaded.len(), 1);
    let (code, warning) = skipped(&warnings);
    assert_eq!(code, "adapter.needs-newer");
    assert_eq!(
        warning.findings[0].message,
        "adapter future needs freemapper 0.14.0 or newer (this is 0.13.0)"
    );
    // The same folder with a newer program.
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), Version(0, 14, 0));
    assert_eq!(loaded.len(), 2);
    assert!(warnings.is_empty());
}

/// A person may keep their adapter folder under version control or in a synced folder. What
/// such tools put there (`.git`, `.stfolder`) starts with a dot and is nobody's adapter: it is
/// passed over without a warning.
#[test]
fn a_dot_folder_in_the_adapter_folder_is_passed_over() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "current", &manifest("current"));
    std::fs::create_dir_all(dir.path().join(".git/objects")).unwrap();
    std::fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["current"]
    );
    assert!(warnings.is_empty(), "{warnings:?}");
}

/// The version of a program built before its release (`0.15.0-rc.1`) counts as that release.
#[test]
fn a_prerelease_of_the_program_counts_as_its_release() {
    assert_eq!(Version::of_program("0.15.0"), Some(Version(0, 15, 0)));
    assert_eq!(Version::of_program("0.15.0-rc.1"), Some(Version(0, 15, 0)));
    assert_eq!(
        Version::of_program("0.15.0+build.7"),
        Some(Version(0, 15, 0))
    );
    assert_eq!(Version::of_program("0.15"), None);
    // A manifest states a release, never a prerelease.
    assert_eq!(Version::parse("0.15.0-rc.1"), None);
}

/// A manifest written for a newer freemapper may use keys and kinds this one does not know.
/// What it says is that the adapter needs the newer program, not that a key is unknown: the
/// version is read before the strict schema.
#[test]
fn a_manifest_of_a_newer_format_1_says_which_freemapper_it_needs() {
    let dir = tempfile::tempdir().unwrap();
    let newer = manifest("future")
        .replace("\"0.13.0\"", "\"0.15.0\"")
        .replace("[net]", "key_of_tomorrow = true\n\n[net]");
    assert!(newer.contains("key_of_tomorrow") && newer.contains("0.15.0"));
    write(dir.path(), "future", &newer);
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(loaded.is_empty());
    let (code, warning) = skipped(&warnings);
    assert_eq!(code, "adapter.needs-newer");
    assert_eq!(warning.findings.len(), 1);
    assert_eq!(
        warning.findings[0].message,
        "adapter future needs freemapper 0.15.0 or newer (this is 0.13.0)"
    );
    // The program it was written for reads the version and then finds what it cannot read.
    let mut registry = Registry::new();
    let (_, warnings) = load_dirs(&mut registry, &user(dir.path()), Version(0, 15, 0));
    assert_eq!(skipped(&warnings).0, "manifest.unknown-key");
}

/// Review focus 5: a folder with more files than an adapter has, and a `rules.toml`, which this
/// freemapper does not read: the adapter loads and the person hears about it.
#[test]
fn a_crowded_folder_is_skipped_and_rules_are_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let crowded = write(dir.path(), "crowded", &manifest("crowded"));
    for index in 0..MAX_FILES - 1 {
        std::fs::write(crowded.join(format!("file-{index}.txt")), "x").unwrap();
    }
    let ruled = write(dir.path(), "ruled", &manifest("ruled"));
    std::fs::write(ruled.join("rules.toml"), "[[rule]]\n").unwrap();
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["ruled"]
    );
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert_eq!(
        (warnings[0].findings[0].code, warnings[0].skipped),
        ("adapter.folder", true)
    );
    assert!(
        warnings[0].findings[0].message.contains("65 files"),
        "{warnings:?}"
    );
    assert_eq!(
        (warnings[1].findings[0].code, warnings[1].skipped),
        ("adapter.rules-not-read", false)
    );
    assert_eq!(warnings[1].findings[0].path, ruled.join("rules.toml"));
    // With one file less the folder is fine.
    std::fs::remove_file(crowded.join("file-0.txt")).unwrap();
    let (loaded, _) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["crowded"]
    );
}

#[test]
fn config_dir_with_spaces_and_unicode_works() {
    let dir = tempfile::tempdir().unwrap();
    let adapters = dir
        .path()
        .join("Anwendungs Daten")
        .join("Übung ö")
        .join("adapters");
    let folder = write(&adapters, "sample", &manifest("sample"));
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(&adapters), RUNNING);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(loaded[0].folder, folder);
    // A manifest saved by an editor on Windows.
    let windows = format!("\u{feff}{}", manifest("windows").replace('\n', "\r\n"));
    write(&adapters, "windows", &windows);
    let (loaded, warnings) = load_dirs(&mut registry, &user(&adapters), RUNNING);
    assert_eq!(
        loaded.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
        ["windows"]
    );
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].findings[0].code, "adapter.duplicate");
}

/// Review focus 5: a link is not followed, wherever it stands: the folder of adapters, the
/// folder of an adapter, its manifest or its boundary.
#[cfg(unix)]
#[test]
fn links_are_not_followed() {
    use std::os::unix::fs::symlink;
    let outside = tempfile::tempdir().unwrap();
    let real = write(outside.path(), "linked", &manifest("linked"));
    let dir = tempfile::tempdir().unwrap();
    // The folder of an adapter is a link.
    symlink(&real, dir.path().join("linked")).unwrap();
    // The manifest is a link.
    let by_manifest = dir.path().join("bymanifest");
    std::fs::create_dir(&by_manifest).unwrap();
    std::fs::write(outside.path().join("m.toml"), manifest("bymanifest")).unwrap();
    symlink(
        outside.path().join("m.toml"),
        by_manifest.join("manifest.toml"),
    )
    .unwrap();
    std::fs::write(by_manifest.join("boundary.geojson"), SQUARE).unwrap();
    // The boundary is a link.
    let by_boundary = dir.path().join("byboundary");
    std::fs::create_dir(&by_boundary).unwrap();
    std::fs::write(by_boundary.join("manifest.toml"), manifest("byboundary")).unwrap();
    symlink(
        real.join("boundary.geojson"),
        by_boundary.join("boundary.geojson"),
    )
    .unwrap();
    let mut registry = Registry::new();
    let (loaded, warnings) = load_dirs(&mut registry, &user(dir.path()), RUNNING);
    assert!(loaded.is_empty(), "{loaded:?}");
    let found: Vec<(String, &str)> = warnings
        .iter()
        .map(|w| {
            (
                w.folder.file_name().unwrap().to_string_lossy().into_owned(),
                w.findings[0].code,
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("byboundary".to_string(), "manifest.file-missing"),
            ("bymanifest".to_string(), "manifest.file-missing"),
            ("linked".to_string(), "adapter.folder"),
        ]
    );
    assert!(warnings.iter().all(|w| w.skipped));
    // The folder of adapters itself is a link.
    let link = outside.path().join("adapters");
    symlink(dir.path(), &link).unwrap();
    let (loaded, warnings) = load_dirs(&mut registry, &user(&link), RUNNING);
    assert!(loaded.is_empty());
    assert_eq!(skipped(&warnings).0, "adapter.folder");
}
