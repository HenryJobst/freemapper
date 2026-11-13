use std::process::Command;

mod isolation;
use isolation::isolated;

mod offline_school;
use offline_school::{offline_school, store_permit_inputs};

#[cfg(all(feature = "test-stub", debug_assertions))]
mod support;
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
use support::installable_release;
#[cfg(all(feature = "test-stub", debug_assertions))]
use support::{MANIFEST_PATH, UpdateStub, copied_program};

/// The program with English texts, whatever language the machine that runs the tests has.
fn freemapper() -> Command {
    let mut command = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")));
    command.env("FREEMAPPER_LANG", "en");
    command
}

/// The program with the language taken from `lang` alone.
fn freemapper_in(lang: &[(&str, &str)]) -> Command {
    let mut command = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")));
    for name in ["FREEMAPPER_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
        command.env_remove(name);
    }
    command.envs(lang.iter().copied());
    command
}

#[test]
fn texts_follow_lang_switch_and_environment() {
    let dir = tempfile::tempdir().unwrap();
    let init = |name: &str, lang: &[(&str, &str)], extra: &[&str]| {
        let output = freemapper_in(lang)
            .arg("init")
            .arg(dir.path().join(name))
            .args(["--geojson", FIXTURE])
            .args(extra)
            .output()
            .unwrap();
        assert!(output.status.success(), "{name}: {output:?}");
        String::from_utf8(output.stdout).unwrap()
    };
    assert!(init("a", &[], &[]).starts_with("Created project "));
    assert!(init("b", &[("LANG", "de_DE.UTF-8")], &[]).contains(" angelegt"));
    assert!(
        init(
            "c",
            &[("FREEMAPPER_LANG", "de"), ("LANG", "en_US.UTF-8")],
            &[]
        )
        .contains(" angelegt")
    );
    // The switch wins over the environment, before or after the subcommand.
    assert!(
        init("d", &[("LANG", "de_DE.UTF-8")], &["--lang", "en"]).starts_with("Created project ")
    );
    let output = freemapper_in(&[])
        .args(["--lang", "DE", "init"])
        .arg(dir.path().join("e"))
        .args(["--geojson", FIXTURE])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains(" angelegt"));
}

#[test]
fn an_unknown_language_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let output = freemapper_in(&[("LANG", "de_DE.UTF-8")])
        .args(["--lang", "fr", "init"])
        .arg(dir.path().join("a"))
        .args(["--geojson", FIXTURE])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fr") && stderr.contains("en, de"),
        "{stderr}"
    );
    assert!(
        stderr.contains("keine Texte"),
        "worded in the environment's language: {stderr}"
    );
    assert!(!dir.path().join("a").exists());
}

#[test]
fn version_flag_prints_name() {
    let output = freemapper().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("freemapper "), "stdout: {stdout}");
}

#[test]
fn no_subcommand_prints_help() {
    let output = freemapper().output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Usage:"), "stderr: {stderr}");
}

/// The help and the parse errors follow the language (ADR 0092, stage 4).
#[test]
fn help_and_parse_errors_follow_the_language() {
    let german = &[("FREEMAPPER_LANG", "de")];
    let help = freemapper_in(german).arg("--help").output().unwrap();
    assert!(help.status.success());
    let stdout = String::from_utf8(help.stdout).unwrap();
    assert!(stdout.contains("Aufruf: freemapper"), "{stdout}");
    // On Windows the usage line says `freemapper.exe`.
    assert!(
        stdout.contains("Einen Projektordner für ein Kartengebiet anlegen"),
        "{stdout}"
    );
    // `--lang` wins over the environment, wherever it stands.
    let english = freemapper_in(german)
        .args(["run", "--help", "--lang", "en"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(english.stdout).unwrap();
    assert!(stdout.contains("Usage: freemapper"), "{stdout}");
    assert!(stdout.contains(" run [OPTIONS] <DIR>"), "{stdout}");
    let of_run = freemapper_in(german)
        .args(["help", "run"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(of_run.stdout).unwrap();
    assert!(stdout.contains("Aufruf: freemapper"), "{stdout}");
    assert!(stdout.contains(" run [OPTIONS] <DIR>"), "{stdout}");
    let wrong = freemapper_in(german).arg("frobnicate").output().unwrap();
    assert_eq!(wrong.status.code(), Some(2));
    let stderr = String::from_utf8(wrong.stderr).unwrap();
    assert!(
        stderr.starts_with("Fehler: Unbekannter Befehl frobnicate"),
        "{stderr}"
    );
}

#[test]
fn init_rejects_scale_zero() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let output = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--scale",
            "0",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!project.join("freemapper.toml").exists());
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/clara-zetkin-schule.geojson"
);

fn init_and_export(project: &std::path::Path) {
    let init = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--kind",
            "schoolyard",
        ])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let export = freemapper()
        .args(["export", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
}

#[test]
fn init_then_export_creates_starter_project() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("clara-zetkin-schule");
    init_and_export(&project);

    for file in [
        "freemapper.toml",
        "clara-zetkin-schule.omap",
        "templates/candidates.omap",
        "ATTRIBUTION.txt",
        "clara-zetkin-schule-preview.webp",
        "data/map-preview.json",
    ] {
        assert!(project.join(file).exists(), "{file} missing");
    }
    let map = std::fs::read_to_string(project.join("clara-zetkin-schule.omap")).unwrap();
    assert!(map.contains(r#"scale="1000""#));
    assert!(map.contains("<parameter>25833</parameter>"));
    assert!(map.contains(r#"<symbols count="195""#));
    assert!(map.contains(r#"type="TemplateMap""#));
    assert!(map.contains("© OpenStreetMap contributors"));
}

#[test]
fn export_whose_preview_fails_leaves_no_preview_of_the_old_map() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("clara-zetkin-schule");
    init_and_export(&project);
    // A folder where the new image goes: the map is written, its preview is not.
    let preview = project.join("clara-zetkin-schule-preview.webp");
    std::fs::remove_file(&preview).unwrap();
    std::fs::create_dir(&preview).unwrap();
    std::fs::write(preview.join("keep"), "").unwrap();
    let pdf = project.join("clara-zetkin-schule.pdf");
    assert!(!pdf.exists());
    let export = freemapper()
        .args(["export", project.to_str().unwrap(), "--force", "--pdf"])
        .output()
        .unwrap();
    let (stdout, stderr) = (
        String::from_utf8_lossy(&export.stdout),
        String::from_utf8_lossy(&export.stderr),
    );
    // A warning, not a failure: the map is written, and the PDF after it too.
    assert_eq!(export.status.code(), Some(0), "{stdout}{stderr}");
    assert!(
        stderr
            .lines()
            .any(|l| l.starts_with("warning: the map is written, but its preview is missing: ")),
        "{stderr}"
    );
    assert!(!stdout.contains("preview.webp"), "{stdout}");
    assert!(project.join("clara-zetkin-schule.omap.bak").exists());
    assert!(pdf.exists(), "{stdout}{stderr}");
    assert!(!project.join("data/map-preview.json").exists());
}

#[test]
fn init_refuses_existing_project() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_and_export(&project);
    let again = freemapper()
        .args(["init", project.to_str().unwrap(), "--geojson", FIXTURE])
        .output()
        .unwrap();
    assert!(!again.status.success());
}

#[test]
fn outline_vertices_map_inside_paper() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("clara-zetkin-schule");
    init_and_export(&project);
    let candidates = std::fs::read_to_string(project.join("templates/candidates.omap")).unwrap();
    assert!(candidates.contains(r#"<t k="freemapper:source_id">way/992423119</t>"#));
    let parts = candidates.find("<parts ").unwrap();
    let start = parts + candidates[parts..].find("<coords count=\"").unwrap();
    let text_start = start + candidates[start..].find('>').unwrap() + 1;
    let text_end = text_start + candidates[text_start..].find("</coords>").unwrap();
    let coords: Vec<(i32, i32)> = candidates[text_start..text_end]
        .split(';')
        .filter(|c| !c.is_empty())
        .map(|c| {
            let mut parts = c.split_whitespace().map(|v| v.parse::<i32>().unwrap());
            (parts.next().unwrap(), parts.next().unwrap())
        })
        .collect();
    assert_eq!(coords.len(), 9, "ring of 8 vertices plus closing point");
    for (x, y) in coords {
        assert!(x.abs() < 200_000 && y.abs() < 200_000, "({x}, {y})");
    }
}

#[test]
fn export_refuses_to_overwrite_map_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_and_export(&project);
    let map = project.join("p.omap");
    std::fs::write(&map, "edited in OOM").unwrap();

    let again = freemapper()
        .args(["export", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!again.status.success());
    assert_eq!(std::fs::read_to_string(&map).unwrap(), "edited in OOM");

    let forced = freemapper()
        .args(["export", project.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    assert!(
        forced.status.success(),
        "{}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(project.join("p.omap.bak")).unwrap(),
        "edited in OOM"
    );
    assert!(
        std::fs::read_to_string(&map)
            .unwrap()
            .contains("<map xmlns=")
    );
}

fn init_school(project: &std::path::Path) {
    let init = freemapper()
        .args(["init", project.to_str().unwrap(), "--geojson", FIXTURE])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
}

#[test]
fn derive_before_fetch_explains_next_step() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let cache = dir.path().join("cache");
    let output = freemapper()
        .args([
            "derive",
            project.to_str().unwrap(),
            "--cache-dir",
            cache.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("freemapper fetch"), "stderr: {stderr}");
}

#[test]
fn offline_fetch_with_empty_cache_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let cache = dir.path().join("cache");
    let output = freemapper()
        .args([
            "fetch",
            project.to_str().unwrap(),
            "--offline",
            "--cache-dir",
            cache.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("33421-5821"), "stderr: {stderr}");
}

#[test]
#[ignore = "network"]
fn live_run_school() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("clara-zetkin-schule");
    init_school(&project);
    let run = freemapper()
        .args(["run", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    for file in ["orthophoto.jpg", "hillshade.png", "slope.png"] {
        assert!(
            project.join("templates").join(file).exists(),
            "{file} missing"
        );
    }
    let map = std::fs::read_to_string(project.join("clara-zetkin-schule.omap")).unwrap();
    assert!(map.contains(r#"<templates count="5" first_front_template="4">"#));
    let buildings = map.matches(r#"<t k="freemapper:source">alkis</t>"#).count();
    assert!(buildings >= 5, "{buildings} ALKIS buildings");
    let candidates = std::fs::read_to_string(project.join("templates/candidates.omap")).unwrap();
    assert!(
        candidates.contains("way/1028507308"),
        "OSM tartan pitch as a candidate"
    );
    assert!(candidates.contains("lgb-dgm1"), "contour candidates");
    assert!(
        candidates.contains("alkis-nutzung"),
        "out-of-bounds candidates"
    );
    assert!(
        candidates.contains("Industrie- und Gewerbefläche"),
        "Am Sportpark 2"
    );
    assert!(project.join("templates/ndsm.png").exists(), "nDSM template");
}

#[test]
fn init_with_title() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let init = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--title",
            "Clara-Zetkin-Schule Strausberg",
        ])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let export = freemapper()
        .args(["export", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let map = std::fs::read_to_string(project.join("p.omap")).unwrap();
    assert!(map.contains("<text>Clara-Zetkin-Schule Strausberg</text>"));
}

#[test]
fn init_requires_exactly_one_area_source() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let none = freemapper()
        .args(["init", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!none.status.success());
    let both = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--osm",
            "way/1",
        ])
        .output()
        .unwrap();
    assert!(!both.status.success());
    assert!(String::from_utf8_lossy(&both.stderr).contains("cannot be used with"));
}

#[test]
fn init_part_picks_a_part_of_an_osm_area_only() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    // --part belongs to --osm.
    let with_geojson = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--part",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!with_geojson.status.success());
    let stderr = String::from_utf8_lossy(&with_geojson.stderr);
    assert!(stderr.contains("--osm"), "{stderr}");
    assert!(!project.join("freemapper.toml").exists());
    // With --osm the flag is taken (the bad id fails before any request).
    let with_osm = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--osm",
            "992423119",
            "--part",
            "2",
        ])
        .output()
        .unwrap();
    assert!(!with_osm.status.success());
    let stderr = String::from_utf8_lossy(&with_osm.stderr);
    assert!(stderr.contains("is not an OSM id"), "{stderr}");
}

#[test]
fn init_with_a_bad_osm_id_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let output = freemapper()
        .args(["init", project.to_str().unwrap(), "--osm", "992423119"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("is not an OSM id"), "{stderr}");
    assert!(!project.join("freemapper.toml").exists());
}

#[test]
#[ignore = "network"]
fn live_init_by_place() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let output = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--place",
            "Clara-Zetkin-Schule Strausberg",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let toml = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(toml.contains("way/992423119"), "{toml}");
}

#[test]
fn find_without_a_name_is_a_usage_error() {
    let output = freemapper().arg("find").output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
}

#[test]
#[ignore = "network"]
fn live_find_lists_the_school() {
    let output = freemapper()
        .args(["find", "Clara-Zetkin-Schule Strausberg"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("way/992423119  Clara-Zetkin-Schule (amenity=school): "));
    assert!(stdout.ends_with("freemapper init <dir> --osm way/992423119\n"));
}

/// `find` against a search answer from a file (`FREEMAPPER_FIND_TEST_ANSWER`), which only a
/// build with the `test-stub` feature reads.
#[cfg(all(feature = "test-stub", debug_assertions))]
mod find_cli {
    use super::{Command, freemapper};

    fn fixture(name: &str) -> String {
        format!("{}/../../tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    fn find(answer: &str) -> Command {
        let mut command = freemapper();
        command
            .env("FREEMAPPER_FIND_TEST_ANSWER", answer)
            .args(["find", "Clara-Zetkin-Schule Strausberg"]);
        command
    }

    #[test]
    fn lists_areas_first_then_the_hint() {
        let output = find(&fixture("nominatim-search-mixed.json"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 3, "{stdout}");
        assert!(lines[0].starts_with("way/992423119  Clara-Zetkin-Schule (amenity=school): "));
        assert!(lines[1].contains("(highway=bus_stop; no area, cannot be a map area)"));
        assert_eq!(
            lines[2],
            "Create the project with: freemapper init <dir> --osm way/992423119"
        );
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn json_is_the_array_alone() {
        let output = find(&fixture("nominatim-search-mixed.json"))
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let places: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(places[0]["osm_id"], "way/992423119");
        assert_eq!(places[1]["has_area"], false);
    }

    #[test]
    fn nothing_found_exits_with_one_in_text_and_prints_an_empty_array_in_json() {
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty.json");
        std::fs::write(&empty, "[]").unwrap();
        let output = find(empty.to_str().unwrap()).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.starts_with("error: no place found for \"Clara-Zetkin-Schule Strausberg\""),
            "{stderr}"
        );
        // With --json an empty array, as the MCP tool find_place.
        let output = find(empty.to_str().unwrap())
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "[]\n");
        // A failed request is an error in both modes.
        for extra in [&[][..], &["--json"][..]] {
            let output = find(dir.path().join("missing.json").to_str().unwrap())
                .args(extra)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .starts_with("error: ")
            );
        }
    }

    /// `freemapper find x | head -1`: a reader that is gone ends the output quietly.
    #[test]
    #[cfg(unix)]
    fn a_closed_pipe_ends_quietly() {
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);
        let output = find(&fixture("nominatim-search-mixed.json"))
            .stdout(writer)
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.is_empty(), "{stderr}");
        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn german_texts() {
        let output = find(&fixture("nominatim-search-mixed.json"))
            .args(["--lang", "de"])
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("(highway=bus_stop; keine Fläche, kann kein Kartengebiet sein)"));
        assert!(
            stdout.ends_with("Projekt anlegen mit: freemapper init <ordner> --osm way/992423119\n")
        );
    }
}

#[test]
fn export_ocad_keeps_an_existing_oom_map() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("schule");
    init_and_export(&project);
    let map = project.join("schule.omap");
    std::fs::write(&map, "edited in OOM").unwrap();
    let export = freemapper()
        .args(["export", project.to_str().unwrap(), "--ocad"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&export.stdout);
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert_eq!(std::fs::read_to_string(&map).unwrap(), "edited in OOM");
    let ocd = project.join("ocad/schule.ocd");
    assert_eq!(&std::fs::read(&ocd).unwrap()[0..2], &[0xAD, 0x0C]);
    assert!(project.join("ocad/schule-candidates.ocd").exists());
    assert!(
        stdout.contains("schule.ocd") && stdout.contains("Load"),
        "{stdout}"
    );
}

#[test]
fn export_pdf_writes_the_map_without_candidates() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("schule");
    init_and_export(&project);
    let export = freemapper()
        .args(["export", project.to_str().unwrap(), "--pdf"])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let pdf = std::fs::read(project.join("schule.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    let stdout = String::from_utf8_lossy(&export.stdout);
    assert!(stdout.contains("schule.pdf"), "{stdout}");
    assert!(
        !project.join("schule-candidates.pdf").exists(),
        "--pdf alone writes no candidates PDF"
    );
}

#[test]
fn export_pdf_candidates_writes_the_second_file() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("schule");
    init_and_export(&project);
    let only = freemapper()
        .args(["export", project.to_str().unwrap(), "--pdf-candidates"])
        .output()
        .unwrap();
    assert!(only.status.success());
    assert!(project.join("schule-candidates.pdf").exists());
    assert!(!project.join("schule.pdf").exists());
    let both = freemapper()
        .args([
            "export",
            project.to_str().unwrap(),
            "--pdf",
            "--pdf-candidates",
        ])
        .output()
        .unwrap();
    assert!(both.status.success());
    let plain = std::fs::read(project.join("schule.pdf")).unwrap();
    let cand = std::fs::read(project.join("schule-candidates.pdf")).unwrap();
    assert!(plain.starts_with(b"%PDF-") && cand.starts_with(b"%PDF-"));
    let stdout = String::from_utf8_lossy(&both.stdout);
    assert!(stdout.contains("schule-candidates.pdf"), "{stdout}");
}

#[test]
fn export_pdf_scale_bar_is_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("schule");
    init_and_export(&project);
    let write = |extra: &[&str]| {
        let mut args = vec!["export", project.to_str().unwrap(), "--pdf"];
        args.extend_from_slice(extra);
        assert!(freemapper().args(&args).output().unwrap().status.success());
        std::fs::read(project.join("schule.pdf")).unwrap()
    };
    let plain = write(&[]);
    let with_bar = write(&["--pdf-scale-bar"]);
    assert_ne!(plain, with_bar, "the flag adds the bar");
    assert!(with_bar.len() > plain.len());
}

#[test]
fn export_pdf_sources_modes_and_the_notice() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("schule");
    init_and_export(&project);
    let run = |extra: &[&str]| {
        let mut args = vec!["export", project.to_str().unwrap(), "--pdf"];
        args.extend_from_slice(extra);
        let out = freemapper().args(&args).output().unwrap();
        assert!(out.status.success());
        let pdf = std::fs::read(project.join("schule.pdf")).unwrap();
        (String::from_utf8_lossy(&out.stdout).into_owned(), pdf)
    };
    let (auto_out, auto) = run(&[]);
    let (full_out, full) = run(&["--pdf-sources", "full"]);
    let (credit_out, credit) = run(&["--pdf-sources", "credit-only"]);
    // A short block is the same in auto and full; credit-only has fewer lines.
    assert_eq!(auto, full);
    assert!(credit.len() < auto.len());
    assert!(!auto_out.contains("without source credits"));
    assert!(!full_out.contains("without source credits"));
    assert!(credit_out.contains("PDF without source credits"));
    assert!(credit_out.contains("ATTRIBUTION.txt"));
    // The full list stays in ATTRIBUTION.txt.
    let list = std::fs::read_to_string(project.join("ATTRIBUTION.txt")).unwrap();
    assert!(list.contains("ODbL") && list.contains("freemapper"));
    // Unknown modes are refused.
    let bad = freemapper()
        .args([
            "export",
            project.to_str().unwrap(),
            "--pdf",
            "--pdf-sources",
            "none",
        ])
        .output()
        .unwrap();
    assert!(!bad.status.success());
}

#[test]
fn init_refuses_unusable_geojson_areas_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let cases = [
        ("empty", r#"{"type":"Polygon","coordinates":[]}"#),
        (
            "bowtie",
            r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.54],[13.85,52.53],[13.84,52.54],[13.84,52.53]]]}"#,
        ),
    ];
    for (name, text) in cases {
        let file = dir.path().join(format!("{name}.geojson"));
        std::fs::write(&file, text).unwrap();
        let project = dir.path().join(name);
        let output = freemapper()
            .args([
                "init",
                project.to_str().unwrap(),
                "--geojson",
                file.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!output.status.success(), "{name}");
        assert!(!stderr.contains("panicked"), "{name}: {stderr}");
        assert!(stderr.contains("not a usable map area"), "{name}: {stderr}");
        assert!(!project.join("freemapper.toml").exists(), "{name}");
    }
}

#[test]
fn init_refuses_scales_print_would_refuse() {
    let dir = tempfile::tempdir().unwrap();
    for (kind, scale, hint) in [
        ("schoolyard", "2000", "1:1000"),
        ("sprint", "3000", "freemapper-print enlarges to 1:3000"),
    ] {
        let project = dir.path().join(format!("{kind}{scale}"));
        let output = freemapper()
            .args([
                "init",
                project.to_str().unwrap(),
                "--geojson",
                FIXTURE,
                "--kind",
                kind,
                "--scale",
                scale,
            ])
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!output.status.success(), "{kind} {scale}");
        assert!(stderr.contains(hint), "{kind} {scale}: {stderr}");
        assert!(!project.join("freemapper.toml").exists());
    }
    let project = dir.path().join("sprint5000");
    let output = freemapper()
        .args([
            "init",
            project.to_str().unwrap(),
            "--geojson",
            FIXTURE,
            "--kind",
            "sprint",
            "--scale",
            "5000",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("not in the sprint norm"), "{stdout}");
}

#[test]
fn init_writes_a_forest_project_and_refuses_sprint_scales_for_it() {
    let dir = tempfile::tempdir().unwrap();
    let init = |name: &str, scale: &str| {
        let project = dir.path().join(name);
        let output = freemapper()
            .args(["init", project.to_str().unwrap(), "--geojson", FIXTURE])
            .args(["--kind", "forest", "--scale", scale, "--lang", "en"])
            .output()
            .unwrap();
        (project, output)
    };
    let (project, output) = init("forest", "10000");
    assert!(output.status.success(), "{output:?}");
    let written = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(written.contains(r#"map_kind = "forest""#), "{written}");
    assert!(written.contains("scale_denominator = 10000"), "{written}");
    let (project, output) = init("forest4000", "4000");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(stderr.contains("1:7500, 1:10000, 1:15000"), "{stderr}");
    assert!(!project.join("freemapper.toml").exists());
}

#[test]
fn init_saves_the_contour_interval_and_refuses_a_bad_one() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("forest");
    let output = freemapper()
        .args(["init", project.to_str().unwrap(), "--geojson", FIXTURE])
        .args([
            "--kind",
            "forest",
            "--contour-interval",
            "2,5",
            "--lang",
            "en",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Contour interval saved in freemapper.toml: 2.5 m"),
        "{stdout}"
    );
    let written = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(written.contains("contour_interval_m = 2.5"), "{written}");
    let bad = dir.path().join("bad");
    let output = freemapper()
        .args(["init", bad.to_str().unwrap(), "--geojson", FIXTURE])
        .args([
            "--kind",
            "forest",
            "--contour-interval",
            "0",
            "--lang",
            "en",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(stderr.contains("invalid contour interval"), "{stderr}");
    assert!(!bad.exists(), "a refused interval creates no project");
}

/// A lock file as another freemapper process would have left it.
fn write_lock(project: &std::path::Path, who: &str) -> std::path::PathBuf {
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let path = project.join(".freemapper-run.lock");
    std::fs::write(
        &path,
        format!(r#"{{"pid":1,"started":{started},"who":"{who}"}}"#),
    )
    .unwrap();
    path
}

#[test]
fn run_fetch_derive_and_export_refuse_a_folder_another_run_holds() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let cache = dir.path().join("cache");
    let lock = write_lock(&project, "mcp");
    let held = std::fs::read_to_string(&lock).unwrap();
    for command in ["run", "fetch", "derive", "export"] {
        let mut args = vec![command, project.to_str().unwrap()];
        if command != "export" {
            args.extend(["--cache-dir", cache.to_str().unwrap()]);
        }
        let output = freemapper().args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{command}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("another freemapper run is using this folder (since ")
                && stderr.contains("started by an AI agent (MCP)"),
            "{command}: {stderr}"
        );
        assert_eq!(std::fs::read_to_string(&lock).unwrap(), held, "{command}");
    }
    // Nothing was written beside the project file and the lock.
    let mut names: Vec<String> = std::fs::read_dir(&project)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, [".freemapper-run.lock", "freemapper.toml"]);
}

#[test]
fn the_refusal_is_worded_in_german_when_asked_for() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    write_lock(&project, "page");
    let output = freemapper_in(&[("FREEMAPPER_LANG", "de")])
        .args(["export", project.to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Ein anderer freemapper-Lauf verwendet diesen Ordner")
            && stderr.contains("von der Gebietsauswahl-Seite"),
        "{stderr}"
    );
}

#[test]
fn a_stale_lock_does_not_block_and_a_finished_command_leaves_none() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let lock = write_lock(&project, "cli");
    let file = std::fs::OpenOptions::new().write(true).open(&lock).unwrap();
    file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(120))
        .unwrap();
    drop(file);
    let output = freemapper()
        .args(["export", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!lock.exists(), "released at the end");
    // A failing command releases it too.
    let output = freemapper()
        .args([
            "derive",
            project.to_str().unwrap(),
            "--cache-dir",
            dir.path().join("cache").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!lock.exists(), "released after an error");
}

#[test]
fn a_command_on_a_missing_folder_reports_the_usual_error_not_a_lock_error() {
    let dir = tempfile::tempdir().unwrap();
    let output = freemapper()
        .args(["export", dir.path().join("nope").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("lock"), "{stderr}");
}

#[test]
fn errors_follow_the_language_and_exit_with_one() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let cache = dir.path().join("cache");
    let derive = |lang: &[(&str, &str)]| {
        let output = freemapper_in(lang)
            .args([
                "derive",
                project.to_str().unwrap(),
                "--cache-dir",
                cache.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        String::from_utf8(output.stderr).unwrap()
    };
    assert_eq!(
        derive(&[]).trim_end(),
        "error: no tiles fetched yet; run `freemapper fetch` first"
    );
    assert_eq!(
        derive(&[("FREEMAPPER_LANG", "de")]).trim_end(),
        "Fehler: Es sind noch keine Kacheln geladen; rufen Sie zuerst `freemapper fetch` auf"
    );
    // A second init is refused in German too.
    let again = freemapper_in(&[("FREEMAPPER_LANG", "de")])
        .arg("init")
        .arg(&project)
        .args(["--geojson", FIXTURE])
        .output()
        .unwrap();
    let stderr = String::from_utf8(again.stderr).unwrap();
    assert!(
        stderr.starts_with("Fehler: Das Projekt gibt es schon: "),
        "{stderr}"
    );
}

/// The engine's warnings come in the program's language too (ADR 0092, stage 3).
#[test]
fn warnings_of_the_engine_follow_the_language() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let file = project.join("freemapper.toml");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        text.replace("scale_denominator = 1000", "scale_denominator = 1234"),
    )
    .unwrap();
    let cache = dir.path().join("cache");
    let first_line = |lang: &[(&str, &str)]| {
        let output = freemapper_in(lang)
            .args([
                "fetch",
                project.to_str().unwrap(),
                "--offline",
                "--cache-dir",
                cache.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        stderr.lines().next().unwrap_or_default().to_string()
    };
    assert_eq!(
        first_line(&[]),
        "warning: freemapper.toml: scale 1:1234 is none of the scales for schoolyard maps (1:1000); freemapper-print will refuse the map"
    );
    assert_eq!(
        first_line(&[("FREEMAPPER_LANG", "de")]),
        "Warnung: freemapper.toml: Der Maßstab 1:1234 gehört nicht zu den Maßstäben für Schulhofkarten (1:1000); freemapper-print wird die Karte ablehnen"
    );
}

// The permit notes (ADR 0117). The fixture `offline-school` is a fetched synthetic school
// project with its cache, so a run works offline; the freemapper crate writes it
// (`write_the_offline_cli_fixture`).

/// Without stored data `permits --offline` says what is missing and exits with 1.
#[test]
fn permits_offline_without_data_exits_with_1() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    init_school(&project);
    let cache = dir.path().join("cache");
    let permits = |command: &mut Command| {
        command
            .arg("permits")
            .arg(&project)
            .arg("--offline")
            .arg("--cache-dir")
            .arg(&cache)
            .output()
            .unwrap()
    };
    let output = permits(&mut freemapper());
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "error: no source of the permit notes could be read (vg250: not read; bfn: not \
             read; osm: not read); fetch them online first"
        ),
        "{stderr}"
    );
    assert!(!project.join("permits.md").exists());
    assert!(!project.join("permits.json").exists());
    let german = permits(&mut freemapper_in(&[("FREEMAPPER_LANG", "de")]));
    assert_eq!(german.status.code(), Some(1));
    let stderr = String::from_utf8(german.stderr).unwrap();
    assert!(
        stderr.contains("Keine Quelle der Genehmigungshinweise ließ sich lesen"),
        "{stderr}"
    );
    // The folder lock is gone again.
    assert!(!project.join(".freemapper-run.lock").exists());
}

/// The permit notes never fail a run: without their data the map is still written, a warning
/// says why the notes are missing, and the run exits with 0.
#[test]
fn run_with_permits_offline_still_writes_the_map() {
    let dir = tempfile::tempdir().unwrap();
    let (project, cache) = offline_school(dir.path());
    let output = freemapper()
        .arg("run")
        .arg(&project)
        .args(["--offline", "--permits", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(project.join("school.omap").exists());
    assert!(
        stderr.contains(
            "warning: the permit notes could not be written: no source of the permit notes \
             could be read (vg250: not read; bfn: not read; osm: not read); fetch them online \
             first"
        ),
        "{stderr}"
    );
    assert!(!project.join("permits.md").exists());
    // Without the switch nothing about the notes is said.
    let output = freemapper()
        .arg("run")
        .arg(&project)
        .args(["--offline", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!String::from_utf8(output.stderr).unwrap().contains("permit"));
}

/// With stored data the run writes the notes after the map and names them with the files it
/// wrote; a source that was down is a warning, not a failure.
#[test]
fn permits_appears_in_the_written_files() {
    let dir = tempfile::tempdir().unwrap();
    let (project, cache) = offline_school(dir.path());
    store_permit_inputs(&project);
    let output = freemapper()
        .arg("run")
        .arg(&project)
        .args(["--offline", "--permits", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let markdown = project.join("permits.md");
    let map_line = format!("Wrote {}", project.join("school.omap").display());
    let notes_line = format!("Wrote the permit notes {}", markdown.display());
    let map_at = stdout.find(&map_line).expect(&stdout);
    let notes_at = stdout.find(&notes_line).expect(&stdout);
    assert!(map_at < notes_at, "the notes come after the map: {stdout}");
    for line in [
        "warning: the permit notes leave out BfN: HTTP 503 Service Unavailable",
        "warning: the permit notes leave out OpenStreetMap: it was not read",
    ] {
        assert!(stderr.contains(line), "{stderr}");
    }
    let text = std::fs::read_to_string(&markdown).unwrap();
    assert!(
        text.starts_with("# Permit notes\n\nA hint, not legal advice."),
        "{text}"
    );
    assert!(text.contains("- State: Land Brandenburg, 100%"), "{text}");
    assert!(text.contains("Landkreis Märkisch-Oderland"), "{text}");
    assert!(
        text.contains("Ordnungsamt (public order office) of the Gemeinde Strausberg"),
        "{text}"
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join("permits.json")).unwrap())
            .unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["sources"][1]["state"], "unreachable");

    // `freemapper permits` alone writes the same notes, here in German, and exits with 0.
    std::fs::remove_file(&markdown).unwrap();
    let output = freemapper_in(&[("FREEMAPPER_LANG", "de")])
        .arg("permits")
        .arg(&project)
        .args(["--offline", "--cache-dir"])
        .arg(&cache)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(
        stdout.contains(&format!(
            "Genehmigungshinweise {} geschrieben",
            markdown.display()
        )),
        "{stdout}"
    );
    assert!(
        stderr.contains("Warnung: Die Genehmigungshinweise lassen BfN aus: HTTP 503"),
        "{stderr}"
    );
    let text = std::fs::read_to_string(&markdown).unwrap();
    assert!(
        text.starts_with("# Hinweise zu Genehmigungen\n\nHinweis, keine Rechtsauskunft."),
        "{text}"
    );
}

// `freemapper update` (spec 3.5). The tests that reach the update code need a build with the
// `test-stub` feature and talk to the stub of `tests/support`.

/// `freemapper update` of `program` in English, without the update switches of the environment.
fn update_of(program: &std::path::Path) -> Command {
    let mut command = isolated(Command::new(program));
    command
        .env("FREEMAPPER_LANG", "en")
        .env_remove("FREEMAPPER_UPDATE")
        .env_remove("FREEMAPPER_UPDATE_TEST_BASE")
        .env_remove("FREEMAPPER_CONFIG_DIR")
        .arg("update");
    command
}

#[test]
fn update_in_source_build_says_how_to_update() {
    for extra in [&[][..], &["--check"], &["--yes"], &["--rollback"]] {
        let output = update_of(std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")))
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{extra:?}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout.trim_end(),
            "This freemapper was built from source. Update it with git pull and cargo build --release --locked.",
            "{extra:?}"
        );
    }
}

#[test]
fn update_mode_writes_settings() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let set = |mode: &str| {
        let output = update_of(std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")))
            .env("FREEMAPPER_CONFIG_DIR", &config)
            .args(["--mode", mode])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains(&format!("Update mode set to {mode}")),
            "{stdout}"
        );
        std::fs::read_to_string(config.join("settings.toml")).unwrap()
    };
    assert!(set("install").contains("mode = \"install\""));
    assert!(set("off").contains("mode = \"off\""));
    // An unknown mode is a usage error and leaves the file alone.
    let output = update_of(std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")))
        .env("FREEMAPPER_CONFIG_DIR", &config)
        .args(["--mode", "sometimes"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        std::fs::read_to_string(config.join("settings.toml"))
            .unwrap()
            .contains("mode = \"off\"")
    );
}

/// `freemapper update` of `program`, talking to `stub`.
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_with(program: &std::path::Path, stub: &UpdateStub) -> Command {
    let mut command = update_of(program);
    command.env("FREEMAPPER_UPDATE_TEST_BASE", &stub.base);
    command
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_with_stub_reports_available_exit_10() {
    let stub = UpdateStub::with_fixture();
    let output = update_with(
        std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")),
        &stub,
    )
    .arg("--check")
    .output()
    .unwrap();
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let current = env!("CARGO_PKG_VERSION");
    for expected in [
        format!("freemapper 99.0.0 is available (you have {current}), released 2026-11-20."),
        "Release notes: https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0".to_string(),
        // 12345678 bytes.
        "Download: 11.8 MB".to_string(),
    ] {
        assert!(stdout.contains(&expected), "{expected}\n{stdout}");
    }
    assert_eq!(
        stub.paths(),
        [
            MANIFEST_PATH.to_string(),
            format!("{MANIFEST_PATH}.minisig")
        ]
    );
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_with_a_bad_signature_is_an_error() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let mut manifest = std::fs::read(dir.join("manifest-99.json")).unwrap();
    manifest.extend_from_slice(b"\n");
    let stub = UpdateStub::serving(
        manifest,
        std::fs::read(dir.join("manifest-99.json.minisig")).unwrap(),
        Vec::new(),
    );
    let output = update_with(
        std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")),
        &stub,
    )
    .arg("--check")
    .output()
    .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "error: the release could not be verified (the update signature is not valid); nothing was installed"
        ),
        "{stderr}"
    );
}

/// A release without `freemapper-update.json` (the manifest URL answers 404) cannot install
/// itself; the message says so and links the release page, in the language of the run.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_of_a_release_without_a_manifest_says_so() {
    let stub = UpdateStub::start(Vec::new());
    let program = std::path::Path::new(env!("CARGO_BIN_EXE_freemapper"));
    let no_manifest = "error: the newest release cannot install itself (it has no update manifest); \
                       download it from https://github.com/HenryJobst/freemapper/releases/latest\n";
    for extra in [&["--check"][..], &["--yes"]] {
        let output = update_with(program, &stub).args(extra).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{extra:?}: {output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap(), no_manifest);
    }
    let output = update_with(program, &stub)
        .env("FREEMAPPER_LANG", "de")
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Fehler: Die neueste Version kann sich nicht selbst installieren (ihr fehlt das \
         Aktualisierungsmanifest). Laden Sie sie von \
         https://github.com/HenryJobst/freemapper/releases/latest herunter.\n"
    );
    // A release asked for by its version: it may not exist at all.
    let output = update_with(program, &stub)
        .args(["--version", "0.9.9"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: release 0.9.9 does not exist or cannot install itself (it has no update manifest); \
         see https://github.com/HenryJobst/freemapper/releases\n"
    );
    assert!(!stub.paths().iter().any(|path| path.ends_with(".minisig")));
}

/// Only the manifest's 404 means "no manifest": a missing signature next to it is another error.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_with_a_missing_signature_is_no_missing_manifest() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let stub = UpdateStub::start(vec![(
        MANIFEST_PATH.to_string(),
        std::fs::read(dir.join("manifest-99.json")).unwrap(),
    )]);
    let output = update_with(
        std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")),
        &stub,
    )
    .arg("--check")
    .output()
    .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error: the update failed: "), "{stderr}");
    assert!(stderr.contains(".minisig: HTTP 404"), "{stderr}");
}

/// Spec 3.3: `--check` exits 10 whenever a newer release exists, also one that needs a manual
/// install; the message links its release page.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_of_a_release_that_needs_a_manual_install_exits_10() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let stub = UpdateStub::serving(
        std::fs::read(dir.join("manifest-99-manual.json")).unwrap(),
        std::fs::read(dir.join("manifest-99-manual.json.minisig")).unwrap(),
        Vec::new(),
    );
    let output = update_with(
        std::path::Path::new(env!("CARGO_BIN_EXE_freemapper")),
        &stub,
    )
    .arg("--check")
    .output()
    .unwrap();
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!(
            "freemapper 99.0.0 cannot be installed by the self-update of {}; install 98.0.0 or newer by hand, see https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0",
            env!("CARGO_PKG_VERSION")
        )),
        "{stdout}"
    );
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_without_yes_and_no_terminal_refuses() {
    let dir = tempfile::tempdir().unwrap();
    let program = copied_program(dir.path());
    let stub = UpdateStub::with_fixture();
    let output = update_with(&program, &stub)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("freemapper 99.0.0 is available"),
        "{stdout}"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("freemapper update --yes"), "{stderr}");
    assert!(!stub.asked_for_an_asset());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

/// Review focus 1: a program folder the user cannot write is named, and nothing is downloaded.
#[test]
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
fn update_in_a_read_only_folder_names_it() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("bin");
    std::fs::create_dir(&folder).unwrap();
    let program = copied_program(&folder);
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
    let stub = UpdateStub::with_fixture();
    let output = update_with(&program, &stub).arg("--yes").output().unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let canonical = std::fs::canonicalize(&folder).unwrap();
    assert!(
        stderr.contains(&format!(
            "cannot write to the program folder {}",
            canonical.display()
        )),
        "{stderr}"
    );
    assert!(!stub.asked_for_an_asset());
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
}

/// The whole install against the stub (see `installable_release`). Without the `minisign` CLI
/// the test says so and passes.
#[test]
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
fn update_yes_installs_over_a_copied_binary() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("bin");
    let work = dir.path().join("work");
    std::fs::create_dir(&folder).unwrap();
    std::fs::create_dir(&work).unwrap();
    let program = copied_program(&folder);
    let Some(stub) = installable_release(&work) else {
        return;
    };

    let output = update_with(&program, &stub).arg("--yes").output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let current = env!("CARGO_PKG_VERSION");
    // Progress goes to stdout like the other progress lines.
    assert!(
        stdout.contains("downloading and checking freemapper 99.0.0"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "freemapper updated from {current} to 99.0.0. Release notes: https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0"
        )),
        "{stdout}"
    );
    assert!(stdout.contains("MCP server"), "{stdout}");
    let version = |path: &std::path::Path| {
        let output = Command::new(path).arg("--version").output().unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    assert_eq!(version(&program), "freemapper 99.0.0");
    assert_eq!(
        version(&folder.join("freemapper.previous")),
        format!("freemapper {current}")
    );
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_rollback_restores_previous() {
    let dir = tempfile::tempdir().unwrap();
    let program = copied_program(dir.path());
    let previous = dir.path().join(if cfg!(windows) {
        "freemapper.previous.exe"
    } else {
        "freemapper.previous"
    });
    std::fs::copy(&program, &previous).unwrap();
    // Both are the same program; the time stamp tells them apart.
    let marked = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    std::fs::File::options()
        .write(true)
        .open(&previous)
        .unwrap()
        .set_modified(marked)
        .unwrap();
    let modified = |path: &std::path::Path| std::fs::metadata(path).unwrap().modified().unwrap();
    let stub = UpdateStub::start(Vec::new());
    let output = update_with(&program, &stub)
        .arg("--rollback")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!(
            "Went back to freemapper {}",
            env!("CARGO_PKG_VERSION")
        )),
        "{stdout}"
    );
    assert_eq!(modified(&program), marked);
    assert_ne!(modified(&previous), marked);
    assert!(stub.paths().is_empty(), "a rollback sends no request");
    // Without a previous version there is nothing to go back to.
    std::fs::remove_file(&previous).unwrap();
    let output = update_with(&program, &stub)
        .arg("--rollback")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("there is no previous version to go back to"),
        "{stderr}"
    );
}

/// The daily check after `init`, `select`, `fetch` and `run` (spec 3.3), against the stub. Only a
/// build with the `test-stub` feature and `FREEMAPPER_UPDATE_TEST_BASE` counts as a release build,
/// so no other test here ever checks. The module name puts these tests under CI's `update` filter.
#[cfg(all(feature = "test-stub", debug_assertions))]
mod update_check {
    use std::path::Path;
    use std::time::{Duration, Instant};

    use freemapper_update::{RecordResult, load_record};

    use super::{Command, MANIFEST_PATH, UpdateStub, copied_program, isolated};

    /// `program` in English, talking to `stub`, without `CI` and the update switches of the
    /// environment, with its settings and its home folder (and so the default cache) in `dir`.
    fn checking(program: &Path, stub: &UpdateStub, dir: &Path) -> Command {
        let mut command = isolated(Command::new(program));
        command
            .env("FREEMAPPER_LANG", "en")
            .env_remove("CI")
            .env_remove("FREEMAPPER_UPDATE")
            .env("FREEMAPPER_CONFIG_DIR", dir.join("config"))
            .env("FREEMAPPER_UPDATE_TEST_BASE", &stub.base)
            .env("HOME", dir.join("home"))
            .env("XDG_CACHE_HOME", dir.join("home/.cache"));
        command
    }

    /// `fetch` of a folder that does not exist: it fails at once, without the network, and its
    /// cache folder is `<dir>/cache`.
    fn failing_fetch<'a>(command: &'a mut Command, dir: &Path) -> &'a mut Command {
        command
            .arg("fetch")
            .arg(dir.join("missing"))
            .arg("--cache-dir")
            .arg(dir.join("cache"))
    }

    fn record(dir: &Path) -> Option<RecordResult> {
        load_record(&dir.join("cache/update-check.json")).map(|record| record.result)
    }

    fn program() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_freemapper"))
    }

    fn notice() -> String {
        format!(
            "freemapper 99.0.0 is available (you have {}). Update with: freemapper update\n",
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Review focus 5: a check that is not done after 3 s is dropped, recorded as failed, and
    /// leaves the command's output and exit code as they are.
    #[test]
    fn slow_source_is_dropped_after_3_s_and_recorded_failed() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture_after(Duration::from_secs(5));
        let without = failing_fetch(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        assert!(stub.paths().is_empty(), "no check when CI is set");
        let started = Instant::now();
        let with = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        let took = started.elapsed();
        assert!(
            // The stub answers after 5 s twice: a check that was not dropped takes 10 s or more.
            took >= Duration::from_millis(2900) && took < Duration::from_millis(8000),
            "{took:?}"
        );
        assert_eq!(with.status.code(), Some(1), "{with:?}");
        assert_eq!(with.status.code(), without.status.code());
        assert_eq!(with.stdout, without.stdout);
        assert_eq!(with.stderr, without.stderr);
        assert_eq!(
            record(dir.path()),
            Some(RecordResult::Failed {
                reason: "timeout".into()
            })
        );
        assert_eq!(stub.paths(), [MANIFEST_PATH]);
    }

    #[test]
    fn server_error_is_recorded_failed_and_not_shown() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::start(Vec::new());
        let without = failing_fetch(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        let with = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(with.status.code(), without.status.code());
        assert_eq!(with.stdout, without.stdout);
        assert_eq!(with.stderr, without.stderr);
        let Some(RecordResult::Failed { reason }) = record(dir.path()) else {
            panic!("{:?}", record(dir.path()));
        };
        assert!(reason.contains("404"), "{reason}");
        assert_eq!(stub.paths(), [MANIFEST_PATH]);
    }

    #[test]
    fn notice_line_on_stderr_after_command_output() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let without = failing_fetch(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        let with = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(with.status.code(), Some(1), "{with:?}");
        assert_eq!(with.stdout, without.stdout);
        let stderr = String::from_utf8(with.stderr).unwrap();
        let before = String::from_utf8(without.stderr).unwrap();
        assert!(before.starts_with("error: "), "{before}");
        assert_eq!(stderr, format!("{before}{}", notice()));
        assert_eq!(
            record(dir.path()),
            Some(RecordResult::Available {
                version: "99.0.0".parse().unwrap(),
                notes: "https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0".into(),
            })
        );
        // The next command the same day neither asks nor tells.
        let again = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(again.stderr).unwrap(), before);
        assert_eq!(stub.paths().len(), 2);
    }

    /// Spec 3.2: a release whose signature does not verify is an error that says so. The daily
    /// check gives one warning line after the command's output, keeps the exit code, records the
    /// attempt as failed, and so says it at most once a day.
    #[test]
    fn bad_signature_gives_one_warning_line() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
        let manifest = std::fs::read(fixtures.join("manifest-99.json")).unwrap();
        let mut changed = manifest.clone();
        changed.extend_from_slice(b"\n");
        for (manifest, signature, detail) in [
            // Signed by a key the program does not know.
            (
                manifest,
                "manifest-99.json.wrong-key.minisig",
                "the update is signed with an unknown key",
            ),
            // Signed by the right key, but the manifest changed after.
            (
                changed,
                "manifest-99.json.minisig",
                "the update signature is not valid",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let stub = UpdateStub::serving(
                manifest,
                std::fs::read(fixtures.join(signature)).unwrap(),
                Vec::new(),
            );
            let without = failing_fetch(
                checking(program(), &stub, dir.path()).env("CI", "1"),
                dir.path(),
            )
            .output()
            .unwrap();
            let with = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
                .output()
                .unwrap();
            assert_eq!(with.status.code(), Some(1), "{with:?}");
            assert_eq!(with.status.code(), without.status.code());
            assert_eq!(with.stdout, without.stdout);
            let before = String::from_utf8(without.stderr).unwrap();
            assert_eq!(
                String::from_utf8(with.stderr).unwrap(),
                format!(
                    "{before}warning: the newest freemapper release could not be verified \
                     ({detail}); nothing was installed\n"
                )
            );
            let Some(RecordResult::Failed { reason }) = record(dir.path()) else {
                panic!("{:?}", record(dir.path()));
            };
            assert_eq!(reason, detail);
            // The next command the same day neither asks nor tells.
            let again = failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
                .output()
                .unwrap();
            assert_eq!(String::from_utf8(again.stderr).unwrap(), before);
            assert_eq!(stub.paths().len(), 2);
        }
    }

    #[test]
    fn no_check_with_offline_or_mode_off() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        failing_fetch(&mut checking(program(), &stub, dir.path()), dir.path())
            .arg("--offline")
            .output()
            .unwrap();
        failing_fetch(
            checking(program(), &stub, dir.path()).env("FREEMAPPER_UPDATE", "off"),
            dir.path(),
        )
        .output()
        .unwrap();
        // `derive` and `export` work offline and do not check either.
        checking(program(), &stub, dir.path())
            .arg("derive")
            .arg(dir.path().join("missing"))
            .arg("--cache-dir")
            .arg(dir.path().join("cache"))
            .output()
            .unwrap();
        assert!(stub.paths().is_empty(), "{:?}", stub.paths());
        assert_eq!(record(dir.path()), None);
    }

    /// `find` uses the network, so the check runs after it like after `init`, in the default
    /// cache folder.
    #[test]
    fn find_runs_the_check_after_its_output() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let answer = format!(
            "{}/../../tests/fixtures/nominatim-search.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let output = checking(program(), &stub, dir.path())
            .env("FREEMAPPER_FIND_TEST_ANSWER", answer)
            .args(["find", "Clara-Zetkin-Schule Strausberg"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.ends_with("--osm way/992423119\n"), "{stdout}");
        assert_eq!(String::from_utf8(output.stderr).unwrap(), notice());
    }

    /// Spec 3.4: mode `install` installs after the command, and only after one that succeeded.
    #[test]
    #[cfg(unix)]
    fn install_mode_installs_only_after_success() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        let work = dir.path().join("work");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&work).unwrap();
        let program = copied_program(&folder);
        let Some(stub) = super::installable_release(&work) else {
            return;
        };
        let project = dir.path().join("p");
        let output = checking(&program, &stub, dir.path())
            .env("FREEMAPPER_UPDATE", "install")
            .arg("init")
            .arg(&project)
            .args(["--geojson", super::FIXTURE])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let current = env!("CARGO_PKG_VERSION");
        let lines: Vec<&str> = stdout.lines().collect();
        let position = |start: &str| {
            lines
                .iter()
                .position(|line| line.starts_with(start))
                .unwrap_or_else(|| panic!("{start}\n{stdout}"))
        };
        // The command's own output comes first, then the install.
        assert!(
            position("Created project") < position("downloading and checking freemapper 99.0.0")
        );
        assert!(
            position("downloading and checking")
                < position(&format!("freemapper updated from {current} to 99.0.0"))
        );
        assert_eq!(
            lines.last(),
            Some(&"The new version applies from the next start of freemapper.")
        );
        assert!(String::from_utf8(output.stderr).unwrap().is_empty());
        let version = Command::new(&program).arg("--version").output().unwrap();
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            "freemapper 99.0.0"
        );
        // The record lies in the default cache folder, since `init` has no `--cache-dir`.
        let recorded = ["home/Library/Caches", "home/.cache"]
            .iter()
            .find_map(|cache| {
                load_record(&dir.path().join(cache).join("freemapper/update-check.json"))
            });
        assert!(matches!(
            recorded.map(|r| r.result),
            Some(RecordResult::Available { .. })
        ),);
    }

    #[test]
    fn install_mode_skips_after_failed_command() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        std::fs::create_dir(&folder).unwrap();
        let program = copied_program(&folder);
        let stub = UpdateStub::with_fixture();
        let output = failing_fetch(
            checking(&program, &stub, dir.path()).env("FREEMAPPER_UPDATE", "install"),
            dir.path(),
        )
        .output()
        .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(String::from_utf8(output.stdout).unwrap().is_empty());
        // Not installed, only told.
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.ends_with(&notice()), "{stderr}");
        assert!(!stub.asked_for_an_asset());
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
    }

    /// Spec 3.5 step 7: every start deletes the stale files an update left behind.
    #[test]
    fn stale_files_are_removed_at_start() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        std::fs::create_dir(&folder).unwrap();
        let program = copied_program(&folder);
        let stale = folder.join(if cfg!(windows) {
            "freemapper.stale-1700000000.exe"
        } else {
            "freemapper.stale-1700000000"
        });
        std::fs::write(&stale, "old").unwrap();
        let stub = UpdateStub::start(Vec::new());
        // A build from source leaves them.
        let mut source_build = checking(&program, &stub, dir.path());
        source_build.env_remove("FREEMAPPER_UPDATE_TEST_BASE");
        source_build
            .args(["update", "--mode", "notify"])
            .output()
            .unwrap();
        assert!(stale.exists());
        let output = checking(&program, &stub, dir.path())
            .args(["update", "--mode", "notify"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(!stale.exists());
        assert!(stub.paths().is_empty());
    }
}

/// A valid adapter folder `id` in `dir`: a WFS on `geo.example.org` and a square boundary.
fn write_adapter(dir: &std::path::Path, id: &str) {
    let folder = dir.join(id);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join("manifest.toml"),
        format!(
            "format = 1\n\n[adapter]\nid = \"{id}\"\nrevision = 2\nmin_freemapper = \"0.13.0\"\ncrs = 25832\n\
             names = {{ en = \"Sample\", de = \"Beispiel\" }}\n\n[net]\nhosts = [\"geo.example.org\"]\n\n\
             [boundary]\nfile = \"boundary.geojson\"\ncrs = 25833\n\n[[osm.extract]]\n\
             geofabrik = \"europe/germany/hamburg\"\nmegabytes = 54\nlabel = \"Sample\"\n\n[attribution]\n\
             licence = \"dl-de/by-2-0\"\ntext = \"© Sample {{year}}\"\n\n[alkis]\nkind = \"wfs\"\n\
             url = \"https://geo.example.org/wfs\"\nbuildings_type = \"ave:GebaeudeBauwerk\"\n\
             land_use_type = \"ave:NutzungFlurstueck\"\n"
        ),
    )
    .unwrap();
    std::fs::write(
        folder.join("boundary.geojson"),
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#,
    )
    .unwrap();
}

/// `adapters list` shows the states built into the program, then the adapters of the folder
/// `FREEMAPPER_ADAPTER_DIR` names and, with `--project`, those of a project; a folder with a
/// defect is a warning on stderr and the list goes on.
#[test]
fn adapters_list_shows_built_in_user_and_project_adapters() {
    let user = tempfile::tempdir().unwrap();
    write_adapter(user.path(), "mine");
    std::fs::create_dir(user.path().join("broken")).unwrap();
    let project = tempfile::tempdir().unwrap();
    write_adapter(&project.path().join("adapters"), "theirs");
    let list = |args: &[&str]| {
        let output = freemapper()
            .env("FREEMAPPER_ADAPTER_DIR", user.path())
            .args(["adapters", "list"])
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let (out, err) = list(&["--json"]);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    let adapters = json["adapters"].as_array().unwrap();
    assert_eq!(adapters.len(), 14, "{out}");
    assert_eq!(adapters[0]["id"], "brandenburg");
    assert_eq!(adapters[0]["source"], "built-in");
    assert_eq!(adapters[13]["id"], "mine");
    assert_eq!(adapters[13]["source"], "user");
    assert_eq!(adapters[13]["revision"], 2);
    assert_eq!(json["warnings"][0]["skipped"], true);
    assert_eq!(
        json["warnings"][0]["findings"][0]["code"],
        "manifest.file-missing"
    );
    assert!(err.is_empty(), "{err}");
    // The project's adapters only when asked for.
    let (out, _) = list(&["--json", "--project", project.path().to_str().unwrap()]);
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["adapters"][14]["id"], "theirs");
    assert_eq!(json["adapters"][14]["source"], "project");
    // As text, with the warning on stderr.
    let (out, err) = list(&[]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 15, "{out}");
    assert!(
        lines[0].starts_with("brandenburg  Brandenburg  built in  revision 1  "),
        "{out}"
    );
    assert_eq!(
        lines[13],
        "mine  Sample  from your folder  revision 2  checked never  hosts: geo.example.org  shares ground with: Brandenburg"
    );
    assert!(
        lines[14].starts_with("Your adapters are read from "),
        "{out}"
    );
    assert!(err.starts_with("warning: the adapter in "), "{err}");
    assert!(err.contains("[manifest.file-missing]"), "{err}");
    let (out, err) = list(&["--lang", "de"]);
    assert!(out.contains("  eingebaut  Revision 1  "), "{out}");
    assert!(err.starts_with("Warnung: Der Adapter in "), "{err}");
}

/// `adapters check` reads the folder of one adapter: exit code 0 when it would load, 1 with
/// findings, 2 when the check cannot run. `adapter check` is the same command.
#[test]
fn adapters_check_passes_fails_and_refuses() {
    let hamburg = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../adapters/de/manifests/hamburg"
    );
    let check = |args: &[&str]| {
        let output = freemapper().args(args).output().unwrap();
        (
            output.status.code(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    // The folder of a built-in state passes, with the warning that its id is built in.
    let (code, out, err) = check(&["adapters", "check", hamburg]);
    assert_eq!(code, Some(0), "{out}{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].starts_with("adapter hamburg (Hamburg), revision 1, hosts: "),
        "{out}"
    );
    assert!(lines[1].contains("[check.built-in-id]"), "{out}");
    assert!(
        lines[2].ends_with("passes the check, with one warning"),
        "{out}"
    );
    assert!(err.is_empty(), "{err}");
    let (code, out, _) = check(&["adapter", "check", hamburg, "--json"]);
    assert_eq!(code, Some(0), "{out}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["passed"], true);
    assert_eq!(json["adapter"]["id"], "hamburg");
    assert_eq!(json["findings"][0]["code"], "check.built-in-id");
    assert_eq!(json["findings"][0]["severity"], "warning");
    let (_, out, _) = check(&["adapters", "check", hamburg, "--lang", "de"]);
    assert!(
        out.starts_with("Adapter hamburg (Hamburg), Revision 1, Hosts: "),
        "{out}"
    );
    assert!(
        out.contains("besteht die Prüfung, mit einer Warnung"),
        "{out}"
    );

    // A folder with a defect: every finding, and exit code 1.
    let dir = tempfile::tempdir().unwrap();
    write_adapter(dir.path(), "mine");
    let mine = dir.path().join("mine");
    std::fs::write(
        mine.join("boundary.geojson"),
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[9.7,53.4],[10.3,53.4],[10.3,53.7],[9.7,53.7],[9.7,53.4]]]}}"#,
    )
    .unwrap();
    let (code, out, _) = check(&["adapters", "check", mine.to_str().unwrap()]);
    assert_eq!(code, Some(1), "{out}");
    assert!(out.starts_with("error: "), "{out}");
    assert!(out.contains("[boundary.crs]"), "{out}");
    assert!(out.trim_end().ends_with("fails the check"), "{out}");
    let (code, out, _) = check(&["adapters", "check", mine.to_str().unwrap(), "--json"]);
    assert_eq!(code, Some(1), "{out}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["passed"], false);
    assert_eq!(json["findings"][0]["code"], "boundary.crs");

    // What cannot run: no folder, a live check without a box, a box outside the boundary.
    let nowhere = dir.path().join("nowhere");
    let (code, out, err) = check(&["adapters", "check", nowhere.to_str().unwrap()]);
    assert_eq!(code, Some(2), "{out}{err}");
    assert!(err.starts_with("error: there is no folder "), "{err}");
    let (code, _, err) = check(&["adapters", "check", hamburg, "--live"]);
    assert_eq!(code, Some(2), "{err}");
    assert!(err.contains("--bbox"), "{err}");
    let cache = dir.path().join("cache");
    let (code, out, err) = check(&[
        "adapters",
        "check",
        hamburg,
        "--live",
        "--bbox",
        "13.84,52.53,13.85,52.54",
        "--cache-dir",
        cache.to_str().unwrap(),
    ]);
    assert_eq!(code, Some(2), "{out}{err}");
    assert_eq!(
        err.trim_end(),
        "error: the box does not lie inside the boundary of the adapter hamburg; a live check \
         asks the services for a place the adapter covers"
    );
    assert!(out.is_empty(), "{out}");
    assert!(!cache.exists());
}

/// The file of the allowed hosts in the configuration folder that `home` stands for.
#[cfg(unix)]
fn allowed_hosts_file(home: &std::path::Path) -> std::path::PathBuf {
    let config = if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support")
    } else {
        home.join("xdg")
    };
    config.join("freemapper").join("allowed-hosts.toml")
}

/// freemapper with `home` as the home folder of the person, so its configuration folder is
/// a folder of the test.
#[cfg(unix)]
fn freemapper_at(home: &std::path::Path) -> Command {
    let mut command = freemapper();
    command
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env("FREEMAPPER_ADAPTER_DIR", home.join("no-adapters"));
    command
}

/// `adapters allow-host` allows a built-in state one more host in the person's configuration
/// folder, `adapters list` shows it, and `disallow-host` takes it back. An address or a name
/// of the local network is refused when it is entered. Nothing else adds a host: not a file
/// in a project folder, not the variables that name other folders of freemapper.
#[cfg(unix)]
#[test]
fn a_host_is_allowed_in_the_persons_configuration_only() {
    let home = tempfile::tempdir().unwrap();
    let run = |command: &mut Command| {
        let output = command.output().unwrap();
        (
            output.status.code(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let allowed_with = |command: &mut Command, id: &str, more: &[&str]| -> serde_json::Value {
        let (code, out, err) = run(command.args(["adapters", "list", "--json"]).args(more));
        assert_eq!(code, Some(0), "{err}");
        let json: serde_json::Value = serde_json::from_str(&out).unwrap();
        let adapters = json["adapters"].as_array().unwrap();
        let adapter = adapters.iter().find(|a| a["id"] == id).unwrap();
        adapter["allowed_hosts"].clone()
    };
    let allowed = |command: &mut Command, id: &str| allowed_with(command, id, &[]);
    let at = || freemapper_at(home.path());
    assert_eq!(allowed(&mut at(), "hessen"), serde_json::json!([]));

    let (code, out, err) = run(at().args(["adapters", "allow-host", "hessen", "cdn.example.net"]));
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.starts_with("freemapper may now also ask cdn.example.net for Hesse. "),
        "{out}"
    );
    let file = allowed_hosts_file(home.path());
    assert!(file.exists(), "{out}");
    assert_eq!(
        allowed(&mut at(), "hessen"),
        serde_json::json!(["cdn.example.net"])
    );
    assert_eq!(allowed(&mut at(), "hamburg"), serde_json::json!([]));
    let (_, out, _) = run(at().args(["adapters", "list"]));
    let hessen = out
        .lines()
        .find(|line| line.starts_with("hessen "))
        .unwrap();
    assert!(
        hessen.ends_with("  allowed by you: cdn.example.net"),
        "{hessen}"
    );
    let (_, out, _) = run(at().args(["adapters", "list", "--lang", "de"]));
    assert!(
        out.contains("  von Ihnen freigegeben: cdn.example.net"),
        "{out}"
    );

    // What is no public name is refused when it is entered, in the language of the run.
    for (host, reason) in [
        ("192.168.0.1", "an IP address, not a DNS name"),
        ("localhost", "a name of this computer or without a domain"),
        ("fritz.box", "a name of the local network"),
    ] {
        let (code, _, err) = run(at().args(["adapters", "allow-host", "hessen", host]));
        assert_eq!(code, Some(1), "{host}");
        assert_eq!(
            err.trim_end(),
            format!("error: {host} cannot be allowed: {reason}")
        );
    }
    let (code, _, err) = run(at().args([
        "adapters",
        "allow-host",
        "hessen",
        "10.0.0.1",
        "--lang",
        "de",
    ]));
    assert_eq!(code, Some(1));
    assert_eq!(
        err.trim_end(),
        "Fehler: 10.0.0.1 lässt sich nicht freigeben: eine IP-Adresse, kein DNS-Name"
    );
    assert_eq!(
        allowed(&mut at(), "hessen"),
        serde_json::json!(["cdn.example.net"])
    );

    // A project that was passed on brings a file of the same name, in every place one might
    // look: it allows nothing, with the switch for its adapters or without.
    let project = tempfile::tempdir().unwrap();
    let theirs = "[allow]\nhamburg = [\"evil.example.net\"]\n";
    for folder in ["", "freemapper", "adapters", "adapters/freemapper"] {
        let folder = project.path().join(folder);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("allowed-hosts.toml"), theirs).unwrap();
    }
    let dir = project.path().to_str().unwrap();
    assert_eq!(
        allowed_with(
            at().current_dir(project.path()),
            "hamburg",
            &["--project", dir]
        ),
        serde_json::json!([])
    );
    // Nor do the variables that name other folders of freemapper move the file.
    assert_eq!(
        allowed(
            at().env("FREEMAPPER_ADAPTER_DIR", project.path())
                .env("FREEMAPPER_CONFIG_DIR", project.path())
                .env(
                    "FREEMAPPER_ALLOWED_HOSTS",
                    project.path().join("allowed-hosts.toml")
                ),
            "hamburg"
        ),
        serde_json::json!([])
    );

    let (code, out, _) = run(at().args(["adapters", "disallow-host", "hessen", "cdn.example.net"]));
    assert_eq!(code, Some(0));
    assert_eq!(
        out.trim_end(),
        "freemapper no longer asks cdn.example.net for Hesse"
    );
    assert_eq!(allowed(&mut at(), "hessen"), serde_json::json!([]));
    assert!(!file.exists());
}

/// An entry for a host that a release made part of the adapter is said to be redundant by
/// the next command, taken out of the file and not mentioned again.
#[cfg(unix)]
#[test]
fn a_redundant_allowed_host_is_said_once_and_removed() {
    let home = tempfile::tempdir().unwrap();
    let file = allowed_hosts_file(home.path());
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "[allow]\nhessen = [\"gds.hessen.de\"]\n").unwrap();
    let project = home.path().join("nothing-here");
    let fetch = || {
        let output = freemapper_at(home.path())
            .arg("fetch")
            .arg(&project)
            .arg("--offline")
            .output()
            .unwrap();
        String::from_utf8(output.stderr).unwrap()
    };
    let first = fetch();
    assert!(
        first.starts_with(
            "warning: gds.hessen.de is now one of the hosts the adapter of Hesse names; your entry in "
        ),
        "{first}"
    );
    assert!(!file.exists());
    assert!(!fetch().contains("gds.hessen.de"));
}

/// A command loads the adapters of its project folder only with `--adapters-from-project`,
/// and then says which adapter asks which hosts.
#[test]
fn project_adapters_need_the_flag() {
    let user = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    write_adapter(&project.path().join("adapters"), "theirs");
    let fetch = |args: &[&str]| {
        let output = freemapper()
            .env("FREEMAPPER_ADAPTER_DIR", user.path())
            .arg("fetch")
            .arg(project.path())
            .arg("--offline")
            .args(args)
            .output()
            .unwrap();
        // The folder holds no project file, so the fetch itself fails either way.
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        String::from_utf8(output.stderr).unwrap()
    };
    let line = "warning: the adapter theirs from the project folder is in use; it may ask these \
                hosts: geo.example.org";
    assert!(!fetch(&[]).contains("theirs"));
    assert!(fetch(&["--adapters-from-project"]).contains(line));
}

/// A project whose file names an adapter of its own folder `adapters` is read only with the
/// switch; without it the adapter is one this freemapper does not have.
#[test]
fn a_project_that_names_its_own_adapter_is_read_only_with_the_flag() {
    let user = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let init = freemapper()
        .env("FREEMAPPER_ADAPTER_DIR", user.path())
        .arg("init")
        .arg(&project)
        .args(["--geojson", FIXTURE])
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    write_adapter(&project.join("adapters"), "theirs");
    let file = project.join("freemapper.toml");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("provider = \"lgb\""), "{text}");
    std::fs::write(
        &file,
        text.replace("provider = \"lgb\"", "provider = \"theirs\""),
    )
    .unwrap();
    let export = |args: &[&str]| {
        let output = freemapper()
            .env("FREEMAPPER_ADAPTER_DIR", user.path())
            .arg("export")
            .arg(&project)
            .args(args)
            .output()
            .unwrap();
        (
            output.status.code(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let unknown = "error: the project file names the adapter \"theirs\", which this freemapper \
                   does not have";
    let (code, without) = export(&[]);
    assert_eq!(code, Some(1), "{without}");
    assert!(without.contains(unknown), "{without}");
    assert!(
        without.contains("(only for a project you trust)"),
        "{without}"
    );
    assert!(!without.contains("is in use"), "{without}");
    // With the switch the project is read, and the person is told about the adapter first.
    let (code, with) = export(&["--adapters-from-project"]);
    assert_eq!(code, Some(0), "{with}");
    assert!(
        with.starts_with("warning: the adapter theirs from the project folder is in use; "),
        "{with}"
    );
}

/// A manifest in a project folder whose hosts are an address or a wildcard is left out with a
/// warning; nothing panics, and the command goes on to its own error.
#[test]
fn a_project_adapter_with_a_hostile_host_list_is_left_out() {
    let user = tempfile::tempdir().unwrap();
    for host in ["127.0.0.1", "*", "localhost"] {
        let project = tempfile::tempdir().unwrap();
        let adapters = project.path().join("adapters");
        write_adapter(&adapters, "theirs");
        let manifest = adapters.join("theirs").join("manifest.toml");
        let text = std::fs::read_to_string(&manifest).unwrap();
        std::fs::write(
            &manifest,
            text.replace(
                "hosts = [\"geo.example.org\"]",
                &format!("hosts = [\"geo.example.org\", \"{host}\"]"),
            ),
        )
        .unwrap();
        let output = freemapper()
            .env("FREEMAPPER_ADAPTER_DIR", user.path())
            .arg("fetch")
            .arg(project.path())
            .args(["--offline", "--adapters-from-project"])
            .output()
            .unwrap();
        // No project file: the fetch fails with its own error, after the warning.
        assert_eq!(output.status.code(), Some(1), "{host}: {output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        let first = stderr.lines().next().unwrap_or_default();
        assert!(
            first.starts_with("warning: the adapter in "),
            "{host}: {stderr}"
        );
        assert!(first.contains(" is left out: "), "{host}: {stderr}");
        assert!(first.contains("[policy.host]"), "{host}: {stderr}");
        assert!(!stderr.contains("is in use"), "{host}: {stderr}");
        assert!(!stderr.contains("panicked"), "{host}: {stderr}");
    }
}
