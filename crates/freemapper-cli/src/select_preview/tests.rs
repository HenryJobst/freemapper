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
    let first = compute(&project, &mut builds, Some(full.clone()), &lang, &|| false).unwrap();
    assert_eq!(first.content, full);
    assert_eq!(builds.built, 1);
    // Other settings, the same build.
    compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    assert_eq!(builds.built, 1);
    // Saved content settings are not part of the stamp, and a request without settings uses
    // them.
    let safe = ContentSettings::with_profile(Profile::Safe);
    set_content(&project, Some(safe.clone())).unwrap();
    let computed = compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    assert_eq!(computed.content, safe);
    assert_eq!(builds.built, 1);
    // New data (a fetch, a derive or an export) builds again, once.
    std::fs::write(project.join("data").join("new.geojson"), "{}").unwrap();
    compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    assert_eq!(builds.built, 2);
    std::fs::create_dir_all(project.join("templates")).unwrap();
    std::fs::write(project.join("templates").join("t.png"), "x").unwrap();
    compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    assert_eq!(builds.built, 3);
    // Another area: the data is not that area's, and nothing is built.
    set_map_area(&project, &moved_area()).unwrap();
    match compute(&project, &mut builds, None, &lang, &|| false) {
        Err(Refusal::Conflict(e)) => {
            assert!(e.contains("Start a run that replaces the map first"), "{e}")
        }
        other => panic!("{:?}", other.map(|c| c.content)),
    }
    assert_eq!(builds.built, 3);
}

#[test]
fn without_a_record_there_is_no_preview() {
    let dir = tempfile::tempdir().unwrap();
    let lang = Lang::english();
    let refused = |dir: &Path| match compute(dir, &mut Builds::default(), None, &lang, &|| false) {
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

/// Another area, close to the school.
fn moved_area() -> MapArea {
    MapArea::from_geojson(
        r#"{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.541],[13.843,52.539]]]}"#,
    )
    .unwrap()
}

/// The conflict sentence of a refused computation.
fn conflict(result: Result<Computed, Refusal>) -> String {
    match result {
        Err(Refusal::Conflict(e)) => e,
        other => panic!("{:?}", other.map(|c| c.content)),
    }
}

#[test]
fn a_run_that_starts_during_a_live_preview_is_not_refused() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let lang = Lang::english();
    let mut builds = Builds::default();
    // A terminal starts a run while the page builds: it gets the folder.
    let mut run = None;
    let refused = conflict(compute_with(
        &project,
        &mut builds,
        None,
        &lang,
        &|| false,
        &mut || {
            run = Some(crate::lock::acquire(&project, crate::lock::Who::Cli).unwrap());
        },
    ));
    assert!(
        refused.contains("another freemapper run is using this folder"),
        "{refused}"
    );
    // What was built may have read half-written data: it is not kept.
    assert!(builds.cached.is_none());
    drop(run);
    compute(&project, &mut builds, None, &lang, &|| false).unwrap();
    assert_eq!(builds.built, 2);
    // The page's own run, started meanwhile, is named as such.
    let started = std::cell::Cell::new(false);
    let refused = conflict(compute_with(
        &project,
        &mut builds,
        None,
        &lang,
        &|| started.get(),
        &mut || started.set(true),
    ));
    assert!(
        refused.starts_with("A run is going on for this project."),
        "{refused}"
    );
    assert!(builds.cached.is_none());
}

#[test]
fn a_save_during_a_live_preview_is_not_mixed_into_it() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let lang = Lang::english();
    let mut builds = Builds::default();
    let mut saved = false;
    let refused = conflict(compute_with(
        &project,
        &mut builds,
        None,
        &lang,
        &|| false,
        &mut || {
            if !saved {
                set_map_area(&project, &moved_area()).unwrap();
                saved = true;
            }
        },
    ));
    assert!(
        refused.contains("Start a run that replaces the map first"),
        "{refused}"
    );
    assert!(builds.cached.is_none());
}

#[test]
fn new_data_during_a_live_preview_is_built_once_more() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), true);
    let lang = Lang::english();
    let mut builds = Builds::default();
    let mut writes = 0;
    let computed = compute_with(&project, &mut builds, None, &lang, &|| false, &mut || {
        if writes == 0 {
            std::fs::write(project.join("data").join("new.geojson"), "{}").unwrap();
        }
        writes += 1;
    });
    assert!(computed.is_ok());
    assert_eq!((writes, builds.built), (2, 2));
    // Data that keeps changing: the second change is refused, nothing is kept.
    let refused = conflict(compute_with(
        &project,
        &mut builds,
        None,
        &lang,
        &|| false,
        &mut || {
            writes += 1;
            std::fs::write(project.join("data").join("new.geojson"), "x".repeat(writes)).unwrap();
        },
    ));
    assert!(
        refused.starts_with("A run is going on for this project."),
        "{refused}"
    );
    assert!(builds.cached.is_none());
}

#[test]
fn the_preview_is_clipped_to_the_area_it_was_built_for() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_in(dir.path(), false);
    let documents = starter_documents(&project).unwrap();
    let before = map_preview_webp(&documents, &project).unwrap();
    set_map_area(&project, &moved_area()).unwrap();
    assert_eq!(map_preview_webp(&documents, &project).unwrap(), before);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_queued_live_preview_gives_up_its_turn_to_a_newer_one() {
    let images = Arc::new(MapImages::default());
    // A live preview is computed: it holds the builds.
    let computing = images.turn().await.expect("the first request's turn");
    // Two more are asked for meanwhile, one after the other, and wait.
    let older = tokio::spawn({
        let images = Arc::clone(&images);
        async move { images.turn().await.is_some() }
    });
    while images.asked.load(Ordering::SeqCst) < 2 {
        tokio::task::yield_now().await;
    }
    let newer = tokio::spawn({
        let images = Arc::clone(&images);
        async move { images.turn().await.is_some() }
    });
    while images.asked.load(Ordering::SeqCst) < 3 {
        tokio::task::yield_now().await;
    }
    drop(computing);
    // The older one returns as soon as it has the builds, without computing; the newer one
    // gets its turn.
    assert!(!older.await.unwrap(), "the older request computes nothing");
    assert!(newer.await.unwrap(), "the newest request computes");
    // Alone, a request gets its turn.
    assert!(images.turn().await.is_some());
}
