use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};

use freemapper::area::MapArea;
use freemapper::project::{MapKind, init_project, map_path};

use super::*;
use crate::jobs::JobHub;
use crate::pipeline::{ResultFile, Step};
use crate::select::tests::{json_post, origin_of, request};
use crate::select::{Defaults, Selector};

/// A run that reports two steps and waits for the test to let it finish (or for a cancel).
struct StubRunner {
    gate: Mutex<Receiver<()>>,
    seen: Mutex<Vec<RunOptions>>,
}

impl PipelineRunner for StubRunner {
    fn run(
        &self,
        _dir: &Path,
        options: &RunOptions,
        _cache: &Cache,
        control: &RunControl,
        progress: &mut dyn FnMut(Event),
    ) -> Result<RunReport, anyhow::Error> {
        self.seen.lock().unwrap().push(*options);
        let total = 3 + u8::from(options.ocad) + u8::from(options.pdf);
        progress(Event::Phase {
            step: Step::Fetch,
            index: 1,
            total,
        });
        progress(Event::Line("fetching tiles"));
        loop {
            if control.is_cancelled() {
                return Err(anyhow::anyhow!("cancelled"));
            }
            let waited = self
                .gate
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_millis(10));
            match waited {
                Err(RecvTimeoutError::Timeout) => {}
                Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        progress(Event::Phase {
            step: Step::Derive,
            index: 2,
            total,
        });
        progress(Event::Line("deriving templates"));
        progress(Event::Warning(
            "warning: the area touches a second Bundesland",
        ));
        Ok(RunReport {
            summary: "stub done".to_string(),
            warnings: vec!["warning: the area touches a second Bundesland".to_string()],
            files: vec![ResultFile {
                kind: "map",
                relative_path: "p.omap".to_string(),
                bytes: 10,
            }],
            kept_map: options.force,
        })
    }
}

/// Remembers the folders it was asked to open.
struct FakeOpener {
    opened: Mutex<Vec<PathBuf>>,
    fail: bool,
}

impl FolderOpener for FakeOpener {
    fn open(&self, dir: &Path) -> Result<(), String> {
        self.opened.lock().unwrap().push(dir.to_path_buf());
        if self.fail {
            Err("no opener here".to_string())
        } else {
            Ok(())
        }
    }
}

struct Page {
    selector: Selector,
    _root: tempfile::TempDir,
    project: PathBuf,
    release: Sender<()>,
    runner: Arc<StubRunner>,
    opener: Arc<FakeOpener>,
}

impl Page {
    async fn new(saved: bool) -> Self {
        Self::with(saved, false, Duration::ZERO, None).await
    }

    async fn with(
        saved: bool,
        opener_fails: bool,
        status_gap: Duration,
        cache: Option<Cache>,
    ) -> Self {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("p");
        if saved {
            let area = MapArea::from_geojson(
                &std::fs::read_to_string(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/fixtures/clara-zetkin-schule.geojson"
                ))
                .unwrap(),
            )
            .unwrap();
            let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
            init_project(&project, &area, MapKind::Schoolyard, None, date, None).unwrap();
        }
        let (release, gate) = channel();
        let runner = Arc::new(StubRunner {
            gate: Mutex::new(gate),
            seen: Mutex::new(Vec::new()),
        });
        let opener = Arc::new(FakeOpener {
            opened: Mutex::new(Vec::new()),
            fail: opener_fails,
        });
        let defaults = Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: false,
            run: RunOptions::default(),
        };
        let cache = cache.unwrap_or_else(|| Cache::new(root.path().join("cache")));
        let services = Services {
            tiles: TileProxy::new(),
            runner: runner.clone(),
            opener: opener.clone(),
            status_gap,
        };
        let selector = Selector::start_with(
            project.clone(),
            defaults,
            cache,
            0,
            Some(Arc::new(JobHub::default())),
            services,
        )
        .await
        .unwrap();
        Self {
            selector,
            _root: root,
            project,
            release,
            runner,
            opener,
        }
    }

    async fn post(&self, route: &str, body: &str) -> (u16, Value) {
        let origin = origin_of(&self.selector.url);
        let (status, _, text) = request(
            &self.selector.url,
            "POST",
            route,
            json_post(&origin),
            body.to_string(),
        )
        .await;
        let json = serde_json::from_str(&text).unwrap_or(Value::String(text));
        (status, json)
    }

    async fn status(&self, query: &str) -> (u16, Value) {
        let (status, _, text) = request(
            &self.selector.url,
            "GET",
            &format!("api/status{query}"),
            vec![],
            String::new(),
        )
        .await;
        let json = serde_json::from_str(&text).unwrap_or(Value::String(text));
        (status, json)
    }

    /// Polls the status until `done` accepts it; gives up after five seconds.
    async fn wait_until(&self, done: impl Fn(&Value) -> bool) -> Value {
        for _ in 0..500 {
            let (status, answer) = self.status("").await;
            assert_eq!(status, 200, "{answer}");
            if done(&answer) {
                return answer;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("the status never reached the expected answer");
    }

    /// Polls the status until it shows `state`.
    async fn wait_for(&self, state: &str) -> Value {
        self.wait_until(|answer| answer["state"] == state).await
    }

    fn calls(&self) -> usize {
        self.runner.seen.lock().unwrap().len()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn run_starts_a_job_and_status_follows_it_to_done() {
    let page = Page::new(true).await;
    let (status, answer) = page.post("api/run", r#"{"ocad":true,"pdf":true}"#).await;
    assert_eq!(status, 202, "{answer}");
    assert_eq!(answer["job_id"], "job-1");
    let running = page.wait_for("running").await;
    assert_eq!(running["job_id"], "job-1");
    // The step is a key and a count; the page words it.
    let running = page.wait_until(|answer| answer["phase"].is_object()).await;
    assert_eq!(
        running["phase"],
        json!({ "step": "step-fetch", "index": 1, "total": 5 })
    );
    assert_eq!(running["files"], json!([]));
    page.release.send(()).unwrap();
    let done = page.wait_for("done").await;
    assert_eq!(done["files"][0]["kind"], "map");
    assert_eq!(done["files"][0]["path"], "p.omap");
    assert_eq!(done["files"][0]["bytes"], 10);
    assert_eq!(done["kept_map"], false);
    assert_eq!(
        done["warnings"],
        json!(["warning: the area touches a second Bundesland"])
    );
    assert_eq!(done["error"], Value::Null);
    let options = page.runner.seen.lock().unwrap()[0];
    assert!(options.ocad && options.pdf && !options.pdf_candidates && !options.offline);
    // The project answer shows no run any more.
    let (_, _, text) = request(
        &page.selector.url,
        "GET",
        "api/project",
        vec![],
        String::new(),
    )
    .await;
    let project: Value = serde_json::from_str(&text).unwrap();
    assert!(project["running"].is_null());
}

#[tokio::test(flavor = "multi_thread")]
async fn run_refuses_unknown_fields_and_the_ones_the_project_fixes() {
    let page = Page::new(true).await;
    for field in [
        "kind", "scale", "title", "dir", "path", "folder", "token", "cache",
    ] {
        let body = format!(r#"{{"{field}":"x"}}"#);
        let (status, answer) = page.post("api/run", &body).await;
        assert_eq!(status, 400, "{field}: {answer}");
        assert!(
            answer["error"].as_str().unwrap().contains(field),
            "{field}: {answer}"
        );
    }
    // Not an object, not even JSON, wrong types.
    for body in [
        "[]",
        "7",
        "not json",
        r#"{"pdf":"yes"}"#,
        r#"{"pdf_sources":3}"#,
    ] {
        let (status, _) = page.post("api/run", body).await;
        assert_eq!(status, 400, "{body}");
    }
    assert_eq!(page.calls(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn run_refuses_a_bad_pdf_sources_with_the_mcp_text() {
    let page = Page::new(true).await;
    let (status, answer) = page.post("api/run", r#"{"pdf_sources":"x"}"#).await;
    assert_eq!(status, 400);
    assert!(
        answer["error"]
            .as_str()
            .unwrap()
            .contains(r#"pdf_sources must be auto, full or credit-only, not "x""#),
        "{answer}"
    );
    assert_eq!(page.calls(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn run_accepts_an_empty_body_as_all_defaults() {
    let page = Page::new(true).await;
    let (status, answer) = page.post("api/run", "").await;
    assert_eq!(status, 202, "{answer}");
    page.release.send(()).unwrap();
    page.wait_for("done").await;
    assert_eq!(page.runner.seen.lock().unwrap()[0], RunOptions::default());
}

#[tokio::test(flavor = "multi_thread")]
async fn run_without_a_saved_project_is_404() {
    let page = Page::new(false).await;
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 404, "{answer}");
    assert!(
        answer["error"].as_str().unwrap().contains("Save the area"),
        "{answer}"
    );
    assert_eq!(page.calls(), 0);
    assert!(!page.project.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn second_run_is_409_and_names_the_job() {
    let page = Page::new(true).await;
    let (status, _) = page.post("api/run", "{}").await;
    assert_eq!(status, 202);
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 409, "{answer}");
    assert!(
        answer["error"].as_str().unwrap().contains("job-1"),
        "{answer}"
    );
    assert_eq!(answer["job_id"], "job-1");
    page.release.send(()).unwrap();
    page.wait_for("done").await;
    assert_eq!(page.calls(), 1);
    // The lock of the first run is gone with it: a new run starts.
    assert!(!page.project.join(crate::lock::LOCK_NAME).exists());
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 202, "{answer}");
    assert_eq!(answer["job_id"], "job-2");
    page.release.send(()).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn force_counts_only_when_a_map_exists() {
    let page = Page::new(true).await;
    let (status, answer) = page.post("api/run", r#"{"force":true}"#).await;
    assert_eq!(status, 202, "{answer}");
    assert_eq!(answer["force_ignored"], true);
    page.release.send(()).unwrap();
    page.wait_for("done").await;
    assert!(!page.runner.seen.lock().unwrap()[0].force);

    std::fs::write(map_path(&page.project), "old map").unwrap();
    let (status, answer) = page.post("api/run", r#"{"force":true}"#).await;
    assert_eq!(status, 202, "{answer}");
    assert_eq!(answer["force_ignored"], false);
    page.release.send(()).unwrap();
    let done = page.wait_for("done").await;
    assert!(page.runner.seen.lock().unwrap()[1].force);
    assert_eq!(done["kept_map"], true);
}

#[derive(Debug)]
struct Tiny;

impl freemapper::space::DiskSpace for Tiny {
    fn volume(&self, _path: &Path) -> std::io::Result<freemapper::space::Volume> {
        Ok(freemapper::space::Volume {
            id: "tiny".to_string(),
            available: 1 << 20,
        })
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn run_is_422_when_the_disk_stops_the_fetch() {
    let root = tempfile::tempdir().unwrap();
    let cache = Cache::new(root.path().join("cache")).with_disk_space(Arc::new(Tiny));
    let page = Page::with(true, false, Duration::ZERO, Some(cache)).await;
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 422, "{answer}");
    assert_eq!(answer["disk"]["stops"], true, "{answer}");
    assert!(
        answer["error"].as_str().unwrap().contains("disk space"),
        "{answer}"
    );
    assert_eq!(page.calls(), 0);
    assert!(!page.project.join(crate::lock::LOCK_NAME).exists());
    // Offline nothing is downloaded, so the disk does not stop it.
    let (status, answer) = page.post("api/run", r#"{"offline":true}"#).await;
    assert_eq!(status, 202, "{answer}");
    page.release.send(()).unwrap();
    page.wait_for("done").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn lock_held_by_another_process_is_409_with_the_message() {
    let page = Page::new(true).await;
    let held = crate::lock::acquire(&page.project, crate::lock::Who::Cli).unwrap();
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 409, "{answer}");
    let message = answer["error"].as_str().unwrap();
    assert!(
        message.contains("another freemapper run is using this folder"),
        "{message}"
    );
    assert!(
        !message.contains(page.project.to_str().unwrap()),
        "{message}"
    );
    assert_eq!(page.calls(), 0);
    drop(held);
    let (status, _) = page.post("api/run", "{}").await;
    assert_eq!(status, 202);
    page.release.send(()).unwrap();
    page.wait_for("done").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn status_is_idle_before_any_run_and_unknown_jobs_are_404() {
    let page = Page::new(true).await;
    let (status, answer) = page.status("").await;
    assert_eq!(status, 200);
    assert_eq!(answer["state"], "idle");
    assert_eq!(answer["lines"], json!([]));
    assert_eq!(answer["next"], 0);
    assert_eq!(answer["files"], json!([]));
    let (status, answer) = page.status("?job=job-9").await;
    assert_eq!(status, 404, "{answer}");
    let (status, _) = page.status("?since=abc").await;
    assert_eq!(status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn status_cursor_returns_only_new_lines() {
    let page = Page::new(true).await;
    page.post("api/run", "{}").await;
    // First line: "fetching tiles".
    let first = page.wait_until(|answer| answer["next"] == 1).await;
    assert_eq!(first["lines"], json!(["fetching tiles"]));
    page.release.send(()).unwrap();
    let done = page.wait_for("done").await;
    assert_eq!(done["next"], 3);
    assert_eq!(done["lines"].as_array().unwrap().len(), 3);
    let (_, rest) = page.status("?since=1").await;
    assert_eq!(
        rest["lines"],
        json!([
            "deriving templates",
            "warning: the area touches a second Bundesland"
        ])
    );
    assert_eq!(rest["next"], 3);
    // Asking from the end, or beyond it, returns nothing new and the same cursor.
    for since in [3, 99] {
        let (_, none) = page.status(&format!("?job=job-1&since={since}")).await;
        assert_eq!(none["lines"], json!([]), "{since}");
        assert_eq!(none["next"], 3, "{since}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn status_faster_than_the_gap_is_429() {
    let page = Page::with(true, false, Duration::from_millis(400), None).await;
    let (status, _) = page.status("").await;
    assert_eq!(status, 200);
    let (status, answer) = page.status("").await;
    assert_eq!(status, 429, "{answer}");
    tokio::time::sleep(Duration::from_millis(450)).await;
    let (status, _) = page.status("").await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn cancel_ends_the_run_as_cancelled() {
    let page = Page::new(true).await;
    let (status, answer) = page.post("api/cancel", "{}").await;
    assert_eq!(status, 409, "{answer}");
    page.post("api/run", "{}").await;
    page.wait_for("running").await;
    let (status, answer) = page.post("api/cancel", "").await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["cancelling"], true);
    let ended = page.wait_for("cancelled").await;
    assert_eq!(ended["error"], "cancelled");
    assert_eq!(ended["files"], json!([]));
    // A cancelled run leaves the folder free for the next one.
    let (status, answer) = page.post("api/run", "{}").await;
    assert_eq!(status, 202, "{answer}");
    page.release.send(()).unwrap();
    page.wait_for("done").await;
    let (status, _) = page.post("api/cancel", r#"{"job_id":"job-1"}"#).await;
    assert_eq!(status, 409, "a finished run cannot be cancelled");
    let (status, _) = page.post("api/cancel", r#"{"dir":"/"}"#).await;
    assert_eq!(status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn open_folder_opens_the_fixed_folder_whatever_the_body_says() {
    let page = Page::new(true).await;
    let (status, answer) = page
        .post("api/open-folder", r#"{"dir":"/etc","path":"/","cmd":"rm"}"#)
        .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["opened"], true);
    let fixed = std::fs::canonicalize(&page.project).unwrap();
    assert_eq!(*page.opener.opened.lock().unwrap(), vec![fixed.clone()]);
    assert_eq!(answer["dir"], fixed.display().to_string());
    assert_eq!(answer["message"], Value::Null);
    // No body at all works too.
    let (status, _) = page.post("api/open-folder", "").await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn open_folder_fails_softly() {
    let page = Page::with(true, true, Duration::ZERO, None).await;
    let (status, answer) = page.post("api/open-folder", "{}").await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["opened"], false);
    assert_eq!(answer["message"], "no opener here");
    assert!(
        answer["dir"].as_str().is_some(),
        "the page shows the path as text"
    );
    // A folder that does not exist yet is not opened at all.
    let page = Page::new(false).await;
    let (status, answer) = page.post("api/open-folder", "{}").await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["opened"], false);
    assert!(
        answer["message"]
            .as_str()
            .unwrap()
            .contains("does not exist"),
        "{answer}"
    );
    assert!(page.opener.opened.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_guards_cover_the_new_routes() {
    let page = Page::new(true).await;
    let origin = origin_of(&page.selector.url);
    for route in ["api/run", "api/cancel", "api/open-folder"] {
        let variants: [(&str, Vec<String>); 5] = [
            ("no origin", vec!["Content-Type: application/json".into()]),
            (
                "foreign origin",
                vec![
                    "Origin: http://evil.example".into(),
                    "Content-Type: application/json".into(),
                ],
            ),
            (
                "text/plain",
                vec![
                    format!("Origin: {origin}"),
                    "Content-Type: text/plain".into(),
                ],
            ),
            (
                "form",
                vec![
                    format!("Origin: {origin}"),
                    "Content-Type: application/x-www-form-urlencoded".into(),
                ],
            ),
            (
                "cross-site",
                vec![
                    format!("Origin: {origin}"),
                    "Content-Type: application/json".into(),
                    "Sec-Fetch-Site: cross-site".into(),
                ],
            ),
        ];
        for (what, extra) in variants {
            let (status, ..) =
                request(&page.selector.url, "POST", route, extra, "{}".to_string()).await;
            assert_eq!(status, 403, "{route}: {what}");
        }
    }
    // GET on a POST route and POST on the GET route are not served.
    let (status, ..) = request(&page.selector.url, "GET", "api/run", vec![], String::new()).await;
    assert_eq!(status, 405);
    let (status, ..) = request(
        &page.selector.url,
        "POST",
        "api/status",
        json_post(&origin),
        "{}".to_string(),
    )
    .await;
    assert_eq!(status, 405);
    // Another host name is refused for the GET route too.
    let foreign = vec!["Host: evil.example".to_string()];
    let (status, ..) = request(
        &page.selector.url,
        "GET",
        "api/status",
        foreign,
        String::new(),
    )
    .await;
    assert_eq!(status, 403);
    assert_eq!(page.calls(), 0);
    assert!(page.opener.opened.lock().unwrap().is_empty());
    assert!(page.selector.is_running());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_new_post_routes_take_at_most_4_kib() {
    let page = Page::new(true).await;
    let big = format!(r#"{{"pdf":true,"x":"{}"}}"#, "a".repeat(5000));
    for route in ["api/run", "api/cancel", "api/open-folder"] {
        let (status, _) = page.post(route, &big).await;
        assert_eq!(status, 413, "{route}");
    }
    assert_eq!(page.calls(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn answers_never_contain_the_token() {
    let page = Page::new(true).await;
    let token = page
        .selector
        .url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap()
        .to_string();
    let (_, run) = page.post("api/run", "{}").await;
    let (_, status) = page.status("").await;
    let (_, open) = page.post("api/open-folder", "{}").await;
    page.release.send(()).unwrap();
    for answer in [run, status, open] {
        assert!(!answer.to_string().contains(&token), "{answer}");
    }
}

#[test]
fn the_opener_command_is_the_folder_alone() {
    let dir = Path::new(if cfg!(windows) {
        r"C:\maps\school"
    } else {
        "/maps/school"
    });
    let (program, args) = opener_command(dir);
    let expected = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    assert_eq!(program, expected);
    assert_eq!(args, vec![dir.as_os_str().to_os_string()]);
}

#[test]
fn the_windows_verbatim_prefix_is_dropped_for_the_file_manager() {
    assert_eq!(strip_verbatim(r"\\?\C:\maps\school"), r"C:\maps\school");
    assert_eq!(
        strip_verbatim(r"\\?\UNC\server\share\x"),
        r"\\server\share\x"
    );
    assert_eq!(strip_verbatim("/maps/school"), "/maps/school");
}

#[test]
fn a_folder_that_looks_like_an_option_is_never_passed_on() {
    assert!(RealOpener.open(Path::new("-rf")).is_err());
    assert!(RealOpener.open(Path::new("relative")).is_err());
}
