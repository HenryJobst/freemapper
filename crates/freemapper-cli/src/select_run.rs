//! The run routes of the selection page: start a run, follow it, cancel it and open the project
//! folder (ADR 0090). The page sends the seven options and nothing else; the folder, the cache,
//! kind, scale and title are the ones the selector started with and the saved project holds.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use freemapper::fetch::{Cache, HttpFetcher};
use freemapper::layers::preview_area;
use freemapper::project::map_path;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::estimate::estimate_json;
use crate::jobs::{JobState, JobStatus, folder_key};
use crate::lock::{Who, acquire};
use crate::pipeline::{
    Event, RunControl, RunOptions, RunReport, RunRequest, describe, parse as parse_options,
};
use crate::select::{AppState, page_lang, saved_project};
use crate::text::{FluentValue, Lang, TEXTS, v};
use crate::tiles::TileProxy;

/// Largest body of the run, cancel and open-folder requests: seven options fit in far less.
pub(crate) const MAX_BODY_SMALL: usize = 4096;

/// What a run does. The real one is `pipeline::run`; tests pass a quick stand-in.
pub(crate) trait PipelineRunner: Send + Sync {
    fn run(
        &self,
        dir: &Path,
        options: &RunOptions,
        cache: &Cache,
        control: &RunControl,
        progress: &mut dyn FnMut(Event),
    ) -> Result<RunReport, anyhow::Error>;
}

/// Opens the project folder in the file manager. Tests pass a stand-in, so no test starts a
/// program.
pub(crate) trait FolderOpener: Send + Sync {
    /// `dir` is the absolute, canonical project folder. The error is a text for the user, in
    /// `lang`.
    fn open(&self, dir: &Path, lang: &Lang) -> Result<(), String>;
}

struct RealRunner;

impl PipelineRunner for RealRunner {
    fn run(
        &self,
        dir: &Path,
        options: &RunOptions,
        cache: &Cache,
        control: &RunControl,
        progress: &mut dyn FnMut(Event),
    ) -> Result<RunReport, anyhow::Error> {
        crate::pipeline::run(dir, options, cache, &HttpFetcher::new(), control, progress)
    }
}

/// The program and arguments that open `dir` in the file manager of this system: the folder is
/// the only argument, and no shell is involved.
fn opener_command(dir: &Path) -> (&'static str, Vec<OsString>) {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    (program, vec![dir.as_os_str().to_os_string()])
}

/// A Windows verbatim path without its prefix (`\\?\C:\x` becomes `C:\x`): `canonicalize`
/// returns that form, and the file manager does not take it.
fn strip_verbatim(path: &str) -> String {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = path.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        path.to_string()
    }
}

struct RealOpener;

impl FolderOpener for RealOpener {
    fn open(&self, dir: &Path, lang: &Lang) -> Result<(), String> {
        let text = |id: &str, args: &[(&str, FluentValue)]| TEXTS.text(lang, id, args);
        let shown = strip_verbatim(&dir.to_string_lossy());
        // Only an absolute folder goes on: a name that starts with `-` could be read as an
        // option by the program.
        if !Path::new(&shown).is_absolute() || shown.starts_with('-') {
            return Err(text("cli-open-folder-refused", &[]));
        }
        let (program, args) = opener_command(Path::new(&shown));
        let spawned = Command::new(program)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match spawned {
            Ok(mut child) => {
                // The program may run on after this call returns; collect its exit so it does
                // not stay a zombie. The answer to the page does not wait for it.
                let _ = std::thread::Builder::new()
                    .name("open-folder".to_string())
                    .spawn(move || {
                        let _ = child.wait();
                    });
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(text(
                "cli-open-folder-no-opener",
                &[("program", v(program))],
            )),
            Err(e) => Err(text("cli-open-folder-failed", &[("error", v(e))])),
        }
    }
}

/// The parts of the page that reach outside: the tile server, the pipeline, the file manager.
pub(crate) struct Services {
    pub(crate) tiles: TileProxy,
    pub(crate) runner: Arc<dyn PipelineRunner>,
    pub(crate) opener: Arc<dyn FolderOpener>,
    /// The shortest time between two served status requests.
    pub(crate) status_gap: Duration,
}

impl Services {
    pub(crate) fn real() -> Self {
        Self {
            tiles: TileProxy::new(),
            runner: Arc::new(RealRunner),
            opener: Arc::new(RealOpener),
            status_gap: Duration::from_millis(200),
        }
    }
}

/// The state name the page sees: `done` where MCP says `finished`.
fn state_name(state: JobState) -> &'static str {
    match state {
        JobState::Running => "running",
        JobState::Finished => "done",
        JobState::Failed => "failed",
        JobState::Cancelled => "cancelled",
    }
}

/// A job as the page's status answer shows it. The lines are the engine's plain lines, the step
/// is a key (`step-fetch`, ...) that the page turns into its own text.
pub(crate) fn status_json(status: &JobStatus) -> Value {
    let files: Vec<Value> = status
        .report
        .iter()
        .flat_map(|report| &report.files)
        .map(|f| json!({ "kind": f.kind, "path": f.relative_path, "bytes": f.bytes }))
        .collect();
    json!({
        "state": state_name(status.state),
        "job_id": status.job_id,
        "phase": status.phase.map(|(step, index, total)| {
            json!({ "step": step.label_key(), "index": index, "total": total })
        }),
        "lines": status.lines,
        "next": status.next,
        "warnings": status.warnings,
        "files": files,
        "kept_map": status.report.as_ref().is_some_and(|r| r.kept_map),
        "summary": status.report.as_ref().map(|r| r.summary.as_str()),
        "error": status.error,
    })
}

/// The run of this page's folder that is going on now, for `api/project`; null when none runs.
pub(crate) fn running_json(state: &AppState) -> Value {
    state
        .hub
        .current_for(&state.dir)
        .filter(|status| status.state == JobState::Running)
        .map_or(Value::Null, |status| status_json(&status))
}

fn error_response(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

/// The JSON object of a request body; no body at all is an empty object.
fn body_value(body: &[u8]) -> Result<Value, String> {
    if body.iter().all(u8::is_ascii_whitespace) {
        return Ok(json!({}));
    }
    serde_json::from_slice(body).map_err(|e| e.to_string())
}

fn bad_request(lang: &Lang, message: String) -> Response {
    error_response(
        StatusCode::BAD_REQUEST,
        TEXTS.text(lang, "cli-run-bad-request", &[("error", v(message))]),
    )
}

/// The answer to a request whose task stopped unexpectedly.
fn internal_error(lang: &Lang, error: impl ToString) -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        TEXTS.text(lang, "cli-internal-error", &[("message", v(error))]),
    )
}

/// What a start needs once the checks passed: the options, and the folder lock that the job
/// keeps until it ends.
struct Prepared {
    options: RunOptions,
    lock: crate::lock::FolderLock,
    force_ignored: bool,
}

/// Why a run does not start: the status and the JSON body of the answer.
struct Refused(StatusCode, Value);

impl Refused {
    fn message(status: StatusCode, message: String) -> Self {
        Self(status, json!({ "error": message }))
    }
}

/// The checks before a run starts. The refusals are in `lang`.
fn prepare(state: &AppState, mut options: RunOptions, lang: &Lang) -> Result<Prepared, Refused> {
    let text = |id: &str, args: &[(&str, FluentValue)]| TEXTS.text(lang, id, args);
    let project = match saved_project(state, lang) {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Err(Refused::message(
                StatusCode::NOT_FOUND,
                text("cli-run-no-project", &[]),
            ));
        }
        Err(e) => return Err(Refused::message(StatusCode::BAD_REQUEST, e)),
    };
    if let Some(job) = state
        .hub
        .current_for(&state.dir)
        .filter(|job| job.state == JobState::Running)
    {
        let message = text("cli-run-already-running", &[("job", v(&job.job_id))]);
        return Err(Refused(
            StatusCode::CONFLICT,
            json!({ "error": message, "job_id": job.job_id }),
        ));
    }
    // `force` replaces an existing map; without one there is nothing to replace.
    let force_ignored = options.force && !map_path(&state.dir).exists();
    if force_ignored {
        options.force = false;
    }
    // The engine checks the disk again before it downloads; this answers before the job exists.
    if !options.offline
        && let Ok(area) = project.area()
    {
        let today = chrono::Local::now().date_naive();
        let preview = preview_area(&area, project.buffer_m, &state.cache, today);
        let estimate = estimate_json(&preview.estimate, &state.cache, &state.dir, lang);
        if estimate["disk"]["stops"] == true {
            let messages: Vec<&str> = estimate["disk"]["messages"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let message = text(
                "cli-run-disk-stops",
                &[("messages", v(messages.join("; ")))],
            );
            return Err(Refused(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": message, "disk": estimate["disk"] }),
            ));
        }
    }
    let lock = acquire(&state.dir, Who::Page)
        .map_err(|e| Refused::message(StatusCode::CONFLICT, crate::errors::message(&e, lang)))?;
    Ok(Prepared {
        options,
        lock,
        force_ignored,
    })
}

/// `POST api/run`: starts the pipeline for the project folder with the seven options.
pub(crate) async fn run(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let lang = page_lang(&headers);
    let request = match body_value(&body)
        .and_then(|value| serde_json::from_value::<RunRequest>(value).map_err(|e| e.to_string()))
    {
        Ok(request) => request,
        Err(e) => return bad_request(&lang, e),
    };
    let options = match parse_options(request, &lang) {
        Ok(options) => options,
        Err(e) => return bad_request(&lang, e),
    };
    let task_state = Arc::clone(&state);
    let task_lang = lang.clone();
    let prepared =
        tokio::task::spawn_blocking(move || prepare(&task_state, options, &task_lang)).await;
    let Prepared {
        options,
        lock,
        force_ignored,
    } = match prepared {
        Ok(Ok(prepared)) => prepared,
        Ok(Err(Refused(status, body))) => return (status, Json(body)).into_response(),
        Err(e) => return internal_error(&lang, e),
    };
    let runner = Arc::clone(&state.runner);
    let cache = state.cache.clone();
    let dir = state.dir.clone();
    let started = state.hub.start(
        &state.dir,
        Box::new(move |control, progress| {
            // Held for as long as the job runs, also against a terminal or an agent.
            let _lock = lock;
            runner
                .run(&dir, &options, &cache, control, progress)
                .map_err(|e| describe(e.as_ref()))
        }),
    );
    match started {
        Ok(job_id) => (
            StatusCode::ACCEPTED,
            Json(json!({ "job_id": job_id, "force_ignored": force_ignored })),
        )
            .into_response(),
        // Only a run of another project can be the reason: this folder was checked before.
        Err(_) => error_response(
            StatusCode::CONFLICT,
            TEXTS.text(&lang, "cli-run-busy-elsewhere", &[]),
        ),
    }
}

#[derive(Deserialize)]
pub(crate) struct StatusQuery {
    job: Option<String>,
    since: Option<usize>,
}

/// `GET api/status?job=ID&since=N`: the run of this folder (the given one, else the current or
/// latest) with the lines from number `since` on.
pub(crate) async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<StatusQuery>,
) -> Response {
    let lang = page_lang(&headers);
    {
        let mut last = state
            .last_status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        if last.is_some_and(|at| now.duration_since(at) < state.status_gap) {
            return error_response(
                StatusCode::TOO_MANY_REQUESTS,
                TEXTS.text(&lang, "cli-status-too-fast", &[]),
            );
        }
        *last = Some(now);
    }
    let since = query.since.unwrap_or(0);
    let found = match &query.job {
        Some(id) => {
            let key = folder_key(&state.dir);
            match state
                .hub
                .status(id, since)
                .filter(|job| folder_key(&job.dir) == key)
            {
                Some(job) => Some(job),
                None => {
                    return error_response(
                        StatusCode::NOT_FOUND,
                        TEXTS.text(&lang, "cli-run-unknown-job", &[("job", v(id))]),
                    );
                }
            }
        }
        None => state
            .hub
            .current_for(&state.dir)
            .and_then(|current| state.hub.status(&current.job_id, since)),
    };
    match found {
        Some(job) => Json(status_json(&job)).into_response(),
        None => Json(json!({
            "state": "idle",
            "job_id": null,
            "phase": null,
            "lines": [],
            "next": 0,
            "warnings": [],
            "files": [],
            "kept_map": false,
            "summary": null,
            "error": null,
        }))
        .into_response(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CancelRequest {
    job_id: Option<String>,
}

/// `POST api/cancel`: asks the run of this folder to stop at the next boundary.
pub(crate) async fn cancel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let lang = page_lang(&headers);
    let request = match body_value(&body)
        .and_then(|value| serde_json::from_value::<CancelRequest>(value).map_err(|e| e.to_string()))
    {
        Ok(request) => request,
        Err(e) => return bad_request(&lang, e),
    };
    let key = folder_key(&state.dir);
    let target = match &request.job_id {
        Some(id) => state.hub.status(id, 0),
        None => state.hub.current_for(&state.dir),
    }
    .filter(|job| job.state == JobState::Running && folder_key(&job.dir) == key);
    match target {
        Some(job) if state.hub.cancel(&job.job_id) => {
            Json(json!({ "cancelling": true, "job_id": job.job_id })).into_response()
        }
        _ => error_response(
            StatusCode::CONFLICT,
            TEXTS.text(&lang, "cli-run-nothing-to-cancel", &[]),
        ),
    }
}

/// `POST api/open-folder`: opens the project folder, the one the page started with, in the file
/// manager. The body is not read.
pub(crate) async fn open_folder(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _body: Bytes,
) -> Response {
    let lang = page_lang(&headers);
    let task_state = Arc::clone(&state);
    let task_lang = lang.clone();
    let opened =
        tokio::task::spawn_blocking(move || match std::fs::canonicalize(&task_state.dir) {
            Ok(dir) => {
                let result = task_state.opener.open(&dir, &task_lang);
                (dir, result)
            }
            Err(_) => (
                task_state.dir.clone(),
                Err(TEXTS.text(&task_lang, "cli-open-folder-missing", &[])),
            ),
        })
        .await;
    match opened {
        Ok((dir, result)) => Json(json!({
            "opened": result.is_ok(),
            "dir": dir,
            "message": result.err(),
        }))
        .into_response(),
        Err(e) => internal_error(&lang, e),
    }
}

#[cfg(test)]
mod tests;
