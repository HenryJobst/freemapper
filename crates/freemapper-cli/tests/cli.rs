use std::process::Command;

/// The program with English texts, whatever language the machine that runs the tests has.
fn freemapper() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_freemapper"));
    command.env("FREEMAPPER_LANG", "en");
    command
}

/// The program with the language taken from `lang` alone.
fn freemapper_in(lang: &[(&str, &str)]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_freemapper"));
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
