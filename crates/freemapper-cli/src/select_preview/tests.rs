use std::path::PathBuf;

use freemapper::area::MapArea;
use freemapper::build::content::{ContentSettings, Profile};
use freemapper::fetch::Cache;
use freemapper::project::{
    MapKind, export_map_preview_from, init_project, set_content, set_map_area, starter_documents,
};

use super::*;
use crate::jobs::JobHub;
use crate::pipeline::RunOptions;
use crate::select::tests::{json_post, origin_of, request};
use crate::select::{Defaults, Selector};

fn school_area() -> MapArea {
    MapArea::from_geojson(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/clara-zetkin-schule.geojson"
        ))
        .unwrap(),
    )
    .unwrap()
}

/// A project in `dir/p`; with `exported` also its preview and record, as an export writes them.
fn project_in(dir: &Path, exported: bool) -> PathBuf {
    let project = dir.join("p");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
    init_project(
        &project,
        &school_area(),
        MapKind::Schoolyard,
        None,
        date,
        None,
    )
    .unwrap();
    if exported {
        let documents = starter_documents(&project).unwrap();
        export_map_preview_from(&project, &documents).unwrap();
    }
    project
}

async fn page_on(project: PathBuf, hub: Option<Arc<JobHub>>) -> Selector {
    let defaults = Defaults {
        kind: MapKind::Schoolyard,
        scale: None,
        title: None,
        from_agent: false,
        run: RunOptions::default(),
    };
    let cache = Cache::new(project.with_file_name("cache"));
    Selector::start(project, defaults, cache, 0, hub)
        .await
        .unwrap()
}

/// A live preview asked for in the browser language `accept`: status and body.
async fn ask(selector: &Selector, accept: &str) -> (u16, String) {
    let mut extra = json_post(&origin_of(&selector.url));
    extra.push(format!("Accept-Language: {accept}"));
    let (status, _, body) =
        request(&selector.url, "POST", "api/map-image", extra, "{}".into()).await;
    (status, body)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_live_preview_is_refused_while_a_run_of_the_page_goes_on() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let hub = Arc::new(JobHub::default());
    let selector = page_on(project.clone(), Some(Arc::clone(&hub))).await;
    // The run of another folder is no reason.
    let other = hub.register(&dir.path().join("other")).unwrap();
    assert_eq!(ask(&selector, "en").await.0, 200);
    hub.cancel(&other);
    hub.run_registered(&other, Box::new(|_, _| Err("cancelled".to_string())));
    hub.register(&project).unwrap();
    for (accept, expected) in [
        ("en-US,en", "A run is going on for this project."),
        (
            "de-DE,de;q=0.9",
            "Für dieses Projekt läuft gerade ein Lauf.",
        ),
    ] {
        let (status, body) = ask(&selector, accept).await;
        assert_eq!(status, 409, "{body}");
        assert!(body.contains(expected), "{body}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_live_preview_is_refused_while_another_process_holds_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let held = crate::lock::acquire(&project, crate::lock::Who::Cli).unwrap();
    let selector = page_on(project.clone(), None).await;
    let (status, body) = ask(&selector, "en").await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body.contains("another freemapper run is using this folder"),
        "{body}"
    );
    let (status, body) = ask(&selector, "de").await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body.contains("Ein anderer freemapper-Lauf verwendet diesen Ordner"),
        "{body}"
    );
    // Let go, and the preview comes; the page leaves no lock behind.
    drop(held);
    let (status, body) = ask(&selector, "en").await;
    assert_eq!(status, 200, "{body}");
    assert!(!project.join(crate::lock::LOCK_NAME).exists());
}

#[test]
fn the_build_is_kept_until_the_stored_data_changes() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let lang = Lang::english();
    let mut builds = Builds::default();
    let full = ContentSettings::with_profile(Profile::Full);
    let first = compute(&project, &mut builds, Some(full.clone()), &lang).unwrap();
    assert_eq!(first.content, full);
    assert_eq!(builds.built, 1);
    // Other settings, the same build.
    compute(&project, &mut builds, None, &lang).unwrap();
    assert_eq!(builds.built, 1);
    // Saved content settings are not part of the stamp, and a request without settings uses
    // them.
    let safe = ContentSettings::with_profile(Profile::Safe);
    set_content(&project, Some(safe.clone())).unwrap();
    let computed = compute(&project, &mut builds, None, &lang).unwrap();
    assert_eq!(computed.content, safe);
    assert_eq!(builds.built, 1);
    // New data (a fetch, a derive or an export) builds again, once.
    std::fs::write(project.join("data").join("new.geojson"), "{}").unwrap();
    compute(&project, &mut builds, None, &lang).unwrap();
    compute(&project, &mut builds, None, &lang).unwrap();
    assert_eq!(builds.built, 2);
    std::fs::create_dir_all(project.join("templates")).unwrap();
    std::fs::write(project.join("templates").join("t.png"), "x").unwrap();
    compute(&project, &mut builds, None, &lang).unwrap();
    assert_eq!(builds.built, 3);
    // Another area: the data is not that area's, and nothing is built.
    let moved = MapArea::from_geojson(
        r#"{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.541],[13.843,52.539]]]}"#,
    )
    .unwrap();
    set_map_area(&project, &moved).unwrap();
    match compute(&project, &mut builds, None, &lang) {
        Err(Refusal::Conflict(e)) => assert!(e.contains("Start a new run first."), "{e}"),
        other => panic!("{:?}", other.map(|c| c.content)),
    }
    assert_eq!(builds.built, 3);
}

#[test]
fn without_a_record_there_is_no_preview() {
    let dir = tempfile::tempdir().unwrap();
    let lang = Lang::english();
    let refused = |dir: &Path| match compute(dir, &mut Builds::default(), None, &lang) {
        Err(Refusal::Conflict(e)) => e,
        other => panic!("{:?}", other.map(|c| c.content)),
    };
    // No project, a project without an export.
    assert_eq!(refused(&dir.path().join("none")), "Create the map first.");
    let project = project_in(dir.path(), false);
    assert_eq!(refused(&project), "Create the map first.");
    // An image whose record is gone is no preview.
    let documents = starter_documents(&project).unwrap();
    export_map_preview_from(&project, &documents).unwrap();
    std::fs::remove_file(project.join("data").join("map-preview.json")).unwrap();
    assert_eq!(refused(&project), "Create the map first.");
}

#[test]
fn the_files_stamp_follows_changes_under_data_and_templates() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir_all(data.join("sub")).unwrap();
    std::fs::write(data.join("sub").join("a"), "1").unwrap();
    let before = FilesStamp::of(dir.path());
    assert_eq!(before, FilesStamp::of(dir.path()));
    // Same size, same count: the content changed, the size too.
    std::fs::write(data.join("sub").join("a"), "22").unwrap();
    let grown = FilesStamp::of(dir.path());
    assert_ne!(grown, before);
    std::fs::remove_file(data.join("sub").join("a")).unwrap();
    assert_ne!(FilesStamp::of(dir.path()), grown);
    // A file outside both folders does not count.
    let now = FilesStamp::of(dir.path());
    std::fs::write(dir.path().join("x.omap"), "map").unwrap();
    assert_eq!(FilesStamp::of(dir.path()), now);
}
