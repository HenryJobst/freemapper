//! The map image routes of the selection page: the preview the last export wrote, and live
//! previews with the content settings the page shows, computed from a build of the project kept
//! in memory (`build_project` once, `place` for every change).
//!
//! `data/map-preview.json` is the authority: an image without a record is no preview, and a
//! record whose fingerprint differs from the saved project's belongs to another area, kind or
//! scale, so the stored data cannot show the saved project and a live preview is refused. The
//! fingerprint counts only the fields that decide the build ([`project_fingerprint`]): the
//! records a later fetch rewrites do not make a preview stale. Only a run that replaces the map
//! writes a new record.
//!
//! A live preview never takes the folder lock: a run must not fail because the page computes a
//! picture. It looks at the lock and the jobs before and after, and throws its result away when
//! anything changed in between.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use freemapper::build::content::ContentSettings;
use freemapper::project::{
    BuiltProject, ProjectFile, build_project, map_preview_webp, place, project_fingerprint,
    read_map_preview, stored_map_preview_record,
};
use serde_json::{Value, json};

use crate::errors::message;
use crate::select::{AppState, content_of, page_lang};
use crate::text::{Lang, TEXTS};

/// The live previews of one page: the cached build and the newest image.
#[derive(Default)]
pub(crate) struct MapImages {
    /// Held for the whole computation of a live preview, also when the page gave up waiting, so
    /// one is computed at a time; a request that comes meanwhile waits.
    builds: Arc<tokio::sync::Mutex<Builds>>,
    /// The newest live image; an older one is dropped.
    live: std::sync::Mutex<Option<LiveImage>>,
    /// How many live previews were asked for: the number of the newest request.
    asked: AtomicU64,
}

impl MapImages {
    /// Waits for the turn of a live preview asked for now. None when a newer one was asked for
    /// while it waited: the page shows only the newest answer, so this one gives up its turn
    /// without computing, and the newest comes sooner.
    async fn turn(&self) -> Option<tokio::sync::OwnedMutexGuard<Builds>> {
        let ticket = self.asked.fetch_add(1, Ordering::SeqCst) + 1;
        let builds = Arc::clone(&self.builds).lock_owned().await;
        (self.asked.load(Ordering::SeqCst) == ticket).then_some(builds)
    }
}

/// The cached build of the project and the number of the last live image.
#[derive(Default)]
struct Builds {
    cached: Option<(Stamp, BuiltProject)>,
    last_id: u64,
    /// How often the project was built, for the tests.
    built: usize,
}

/// What a build was made from: the fields of the project file that decide it
/// ([`project_fingerprint`]), and the state of the stored data and templates. Another stamp means
/// the build is out of date.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stamp {
    fingerprint: String,
    files: FilesStamp,
}

/// The newest modification time, the number and the total size of the files and folders under
/// `data/` and `templates/`. A fetch, a derive or an export changes at least one of them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct FilesStamp {
    newest: Option<SystemTime>,
    entries: u64,
    bytes: u64,
}

impl FilesStamp {
    /// Adds `path` and, for a folder, everything in it. Links are counted, not followed.
    fn add(&mut self, path: &Path) {
        let Ok(meta) = std::fs::symlink_metadata(path) else {
            return;
        };
        self.entries += 1;
        self.bytes += meta.len();
        if let Ok(modified) = meta.modified() {
            self.newest = self.newest.max(Some(modified));
        }
        if meta.is_dir()
            && let Ok(entries) = std::fs::read_dir(path)
        {
            for entry in entries.flatten() {
                self.add(&entry.path());
            }
        }
    }

    fn of(dir: &Path) -> Self {
        let mut stamp = Self::default();
        stamp.add(&dir.join("data"));
        stamp.add(&dir.join("templates"));
        stamp
    }
}

/// A live image and its number.
struct LiveImage {
    id: u64,
    webp: Bytes,
}

/// What a live preview computed.
struct Computed {
    webp: Vec<u8>,
    width: u32,
    height: u32,
    content: ContentSettings,
}

/// Why no live preview was computed.
#[derive(Debug)]
enum Refusal {
    /// A run holds the folder, there is no map yet, or the data is not the saved project's (409).
    Conflict(String),
    /// The project or its data could not be read (400).
    Failed(String),
}

/// A WebP image that the browser does not keep.
fn webp_response(webp: Bytes) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        webp,
    )
        .into_response()
}

fn error_response(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

/// Whether `record` describes `project`: the export that wrote it used a project file with the
/// same fields that decide the build ([`project_fingerprint`]).
fn fits(record_fingerprint: &str, project: &ProjectFile) -> bool {
    project_fingerprint(project).is_ok_and(|fingerprint| fingerprint == record_fingerprint)
}

/// `GET api/map-image`: the preview the last export wrote, 404 when there is none.
pub(crate) async fn stored(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let lang = page_lang(&headers);
    let dir = state.dir.clone();
    let image = tokio::task::spawn_blocking(move || read_map_preview(&dir))
        .await
        .ok()
        .flatten();
    match image {
        Some(webp) => webp_response(Bytes::from(webp)),
        None => error_response(
            StatusCode::NOT_FOUND,
            TEXTS.text(&lang, "cli-map-image-none", &[]),
        ),
    }
}

/// `GET api/map-image/state`: `{ stored, fresh, content }`. `stored`: the folder has a preview;
/// `fresh`: it was exported for the saved project (by the fields that decide the build), so live
/// previews work; `content`: the settings the stored preview shows, or null.
pub(crate) async fn stored_state(State(state): State<Arc<AppState>>) -> Response {
    let dir = state.dir.clone();
    let answer = tokio::task::spawn_blocking(move || {
        let record = stored_map_preview_record(&dir);
        let fresh = record.as_ref().is_some_and(|record| {
            ProjectFile::load(&dir).is_ok_and(|project| fits(&record.fingerprint, &project))
        });
        json!({
            "stored": record.is_some(),
            "fresh": fresh,
            "content": record.map_or(Value::Null, |record| {
                serde_json::to_value(record.content).unwrap_or(Value::Null)
            }),
        })
    })
    .await
    .unwrap_or_else(|_| json!({ "stored": false, "fresh": false, "content": null }));
    Json(answer).into_response()
}

/// `POST api/map-image` with `{ "content"?: <settings> }` (the shape of `api/save`; absent means
/// the saved project's): computes a live preview and answers `{ id, width, height, content }`;
/// `GET api/map-image/live/<id>` serves it.
pub(crate) async fn live(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let lang = page_lang(&headers);
    let body = match crate::select_run::body_value(&body) {
        Ok(body @ Value::Object(_)) => body,
        _ => {
            return error_response(
                StatusCode::BAD_REQUEST,
                TEXTS.text(&lang, "cli-map-image-bad-request", &[]),
            );
        }
    };
    let content = match content_of(&body, &lang, "cli-map-image-bad-content") {
        Ok(content) => content,
        Err(e) => return freemapper_web::refused(&e),
    };
    // Answered at once; checked again once it is this request's turn.
    if state.hub.running_for(&state.dir) {
        return error_response(
            StatusCode::CONFLICT,
            TEXTS.text(&lang, "cli-map-image-running", &[]),
        );
    }
    // Owned, so it stays held until the computation ends, also when the page stops waiting.
    let Some(mut builds) = state.map_images.turn().await else {
        // The page dropped this request when it asked for the newer one.
        return error_response(
            StatusCode::CONFLICT,
            TEXTS.text(&lang, "cli-map-image-superseded", &[]),
        );
    };
    let task_state = Arc::clone(&state);
    let task_lang = lang.clone();
    let answer = tokio::task::spawn_blocking(move || -> Result<Value, Refusal> {
        let (state, lang) = (task_state, task_lang);
        let running = || state.hub.running_for(&state.dir);
        let computed = compute(&state.dir, &mut builds, content, &lang, &running)?;
        builds.last_id += 1;
        let id = builds.last_id;
        *state
            .map_images
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(LiveImage {
            id,
            webp: Bytes::from(computed.webp),
        });
        Ok(json!({
            "id": id,
            "width": computed.width,
            "height": computed.height,
            "content": computed.content,
        }))
    })
    .await;
    match answer {
        Ok(Ok(answer)) => Json(answer).into_response(),
        Ok(Err(Refusal::Conflict(e))) => error_response(StatusCode::CONFLICT, e),
        Ok(Err(Refusal::Failed(e))) => error_response(StatusCode::BAD_REQUEST, e),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            TEXTS.text(
                &lang,
                "cli-internal-error",
                &[("message", crate::text::v(e))],
            ),
        ),
    }
}

/// `GET api/map-image/live/<id>`: the newest live image, when `id` is its number; 404 otherwise.
pub(crate) async fn live_image(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let newest = state
        .map_images
        .live
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|image| id.parse() == Ok(image.id))
        .map(|image| image.webp.clone());
    match newest {
        Some(webp) => webp_response(webp),
        None => error_response(
            StatusCode::NOT_FOUND,
            TEXTS.text(&page_lang(&headers), "cli-map-image-gone", &[]),
        ),
    }
}

/// What the checks before a computation found: the saved settings and what the build must be
/// made from.
struct Checked {
    saved_content: ContentSettings,
    stamp: Stamp,
}

/// Why the folder no longer looks as it did before the computation.
enum Moved {
    /// A run started or another process holds the folder: the refusal that says so.
    Refused(Refusal),
    /// The project file or the stored data changed: compute again.
    Changed,
}

/// Whether a run of this page (`running`) or of another process (the folder lock, only looked
/// at, never taken, so a run that starts now is not refused) uses the folder.
fn check_runs(dir: &Path, lang: &Lang, running: &dyn Fn() -> bool) -> Result<(), Refusal> {
    if running() {
        return Err(Refusal::Conflict(TEXTS.text(
            lang,
            "cli-map-image-running",
            &[],
        )));
    }
    crate::lock::check_free(dir).map_err(|e| Refusal::Conflict(message(&e, lang)))
}

/// The checks before a computation: no run, a project, a preview record that fits it.
fn check_before(dir: &Path, lang: &Lang, running: &dyn Fn() -> bool) -> Result<Checked, Refusal> {
    let text = |id: &str| TEXTS.text(lang, id, &[]);
    if !dir.join("freemapper.toml").is_file() {
        return Err(Refusal::Conflict(text("cli-map-image-none")));
    }
    check_runs(dir, lang, running)?;
    let project = ProjectFile::load(dir).map_err(|e| Refusal::Failed(message(&e, lang)))?;
    let record = stored_map_preview_record(dir)
        .ok_or_else(|| Refusal::Conflict(text("cli-map-image-none")))?;
    if !fits(&record.fingerprint, &project) {
        return Err(Refusal::Conflict(text("cli-map-image-stale")));
    }
    Ok(Checked {
        saved_content: project.content.unwrap_or_default(),
        stamp: Stamp {
            fingerprint: record.fingerprint,
            files: FilesStamp::of(dir),
        },
    })
}

/// The checks after a computation: still no run, and the project file and the stored data as
/// they were before it, so the build read neither half-written data nor another area.
fn check_after(
    dir: &Path,
    lang: &Lang,
    running: &dyn Fn() -> bool,
    before: &Stamp,
) -> Result<(), Moved> {
    check_runs(dir, lang, running).map_err(Moved::Refused)?;
    let unchanged = ProjectFile::load(dir).is_ok_and(|project| fits(&before.fingerprint, &project))
        && FilesStamp::of(dir) == before.files;
    if unchanged {
        Ok(())
    } else {
        Err(Moved::Changed)
    }
}

/// A live preview of the project in `dir` with `content` (none: the saved project's), from the
/// cached build in `builds`, which is built again when its stamp is out of date.
///
/// Nothing is locked while it computes, so a run that starts meanwhile is never refused. Instead
/// the folder is checked before and after: a run that started, a save or new data in between
/// throw the result away. A change without a run is computed once more (the checks before then
/// say what happened, for example that the area changed); a second change is refused.
fn compute(
    dir: &Path,
    builds: &mut Builds,
    content: Option<ContentSettings>,
    lang: &Lang,
    running: &dyn Fn() -> bool,
) -> Result<Computed, Refusal> {
    compute_with(dir, builds, content, lang, running, &mut || {})
}

/// [`compute`] with `meanwhile` called between the computation and the checks after it, where
/// tests change the folder.
fn compute_with(
    dir: &Path,
    builds: &mut Builds,
    content: Option<ContentSettings>,
    lang: &Lang,
    running: &dyn Fn() -> bool,
    meanwhile: &mut dyn FnMut(),
) -> Result<Computed, Refusal> {
    for _ in 0..2 {
        let checked = check_before(dir, lang, running)?;
        let stamp = checked.stamp;
        // The old build goes before the new one is made: two may not fit in memory.
        if builds
            .cached
            .as_ref()
            .is_some_and(|(known, _)| *known != stamp)
        {
            builds.cached = None;
        }
        let built = match &mut builds.cached {
            Some((_, built)) => built,
            slot @ None => {
                let built = build_project(dir).map_err(|e| Refusal::Failed(message(&e, lang)))?;
                builds.built += 1;
                &slot.insert((stamp.clone(), built)).1
            }
        };
        // The build reads the project file itself: it must be the one checked.
        let computed = if built.fingerprint() == stamp.fingerprint {
            let content = content.clone().unwrap_or(checked.saved_content);
            let documents =
                place(built, &content).map_err(|e| Refusal::Failed(message(&e, lang)))?;
            let image = map_preview_webp(&documents, dir)
                .map_err(|e| Refusal::Failed(message(&e, lang)))?;
            Some(Computed {
                webp: image.webp,
                width: image.width,
                height: image.height,
                content,
            })
        } else {
            None
        };
        meanwhile();
        match (computed, check_after(dir, lang, running, &stamp)) {
            (Some(computed), Ok(())) => return Ok(computed),
            (_, Err(Moved::Refused(refusal))) => {
                builds.cached = None;
                return Err(refusal);
            }
            _ => builds.cached = None,
        }
    }
    Err(Refusal::Conflict(TEXTS.text(
        lang,
        "cli-map-image-running",
        &[],
    )))
}

#[cfg(test)]
mod tests;
