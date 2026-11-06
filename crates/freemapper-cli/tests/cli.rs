use std::process::Command;

fn freemapper() -> Command {
    Command::new(env!("CARGO_BIN_EXE_freemapper"))
}

#[test]
fn version_flag_prints_name() {
    let output = freemapper().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("freemapper "), "stdout: {stdout}");
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
    assert!(map.contains(r#"<symbols count="191""#));
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
