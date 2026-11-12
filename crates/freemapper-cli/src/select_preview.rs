//! The map image routes of the selection page: the preview the last export wrote, and live
//! previews with the content settings the page shows, computed from a build of the project kept
//! in memory (`build_project` once, `place` for every change).
//!
//! `data/map-preview.json` is the authority: an image without a record is no preview, and a
//! record whose fingerprint differs from the saved project's belongs to another area, kind or
//! scale, so the stored data cannot show the saved project and a live preview is refused.

use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use freemapper::build::content::ContentSettings;
use freemapper::project::{
    BuiltProject, ProjectFile, build_project, map_preview_path, map_preview_webp, place,
    project_fingerprint, read_map_preview_record,
};
use serde_json::{Value, json};

use crate::errors::message;
use crate::jobs::JobState;
use crate::select::{AppState, page_lang};
use crate::text::{Lang, TEXTS};

/// The live previews of one page: the cached build and the newest image.
#[derive(Default)]
pub(crate) struct MapImages {
    /// Held for the whole computation of a live preview, also when the page gave up waiting, so
    /// one is computed at a time; a request that comes meanwhile waits.
    builds: Arc<tokio::sync::Mutex<Builds>>,
    /// The newest live image; an older one is dropped.
    live: std::sync::Mutex<Option<LiveImage>>,
}

/// The cached build of the project and the number of the last live image.
#[derive(Default)]
struct Builds {
    cached: Option<(Stamp, BuiltProject)>,
    last_id: u64,
    /// How often the project was built, for the tests.
    built: usize,
}

/// What a build was made from: the project file without its content settings, and the state of
/// the stored data and templates. Another stamp means the build is out of date.
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

/// Whether `record` describes `project`: the export that wrote it used the same project file,
/// apart from the content settings.
fn fits(record_fingerprint: &str, project: &ProjectFile) -> bool {
    project_fingerprint(project).is_ok_and(|fingerprint| fingerprint == record_fingerprint)
}

/// `GET api/map-image`: the preview the last export wrote, 404 when there is none.
pub(crate) async fn stored(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let lang = page_lang(&headers);
    let dir = state.dir.clone();
    let image = tokio::task::spawn_blocking(move || {
        read_map_preview_record(&dir)?;
        std::fs::read(map_preview_path(&dir)).ok()
    })
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
/// `fresh`: it was exported for the saved project (apart from the content settings), so live
/// previews work; `content`: the settings the stored preview shows, or null.
pub(crate) async fn stored_state(State(state): State<Arc<AppState>>) -> Response {
    let dir = &state.dir;
    let record = read_map_preview_record(dir).filter(|_| map_preview_path(dir).is_file());
    let fresh = record.as_ref().is_some_and(|record| {
        ProjectFile::load(dir).is_ok_and(|project| fits(&record.fingerprint, &project))
    });
    Json(json!({
        "stored": record.is_some(),
        "fresh": fresh,
        "content": record.map_or(Value::Null, |record| {
            serde_json::to_value(record.content).unwrap_or(Value::Null)
        }),
    }))
    .into_response()
}

/// `POST api/map-image` with `{ "content"?: <settings> }` (the shape of `api/save`; absent means
/// the saved project's): computes a live preview and answers `{ id, width, height, content }`;
/// `GET api/map-image/live/<id>` serves it.
pub(crate) async fn live(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let lang = page_lang(&headers);
    let content = match body.get("content") {
        None | Some(Value::Null) => None,
        Some(value) => match serde_json::from_value::<ContentSettings>(value.clone()) {
            Ok(content) => Some(content),
            Err(_) => {
                return freemapper_web::refused(&TEXTS.text(
                    &lang,
                    "cli-map-image-bad-content",
                    &[],
                ));
            }
        },
    };
    if running(&state) {
        return error_response(
            StatusCode::CONFLICT,
            TEXTS.text(&lang, "cli-map-image-running", &[]),
        );
    }
    // Owned, so it stays held until the computation ends, also when the page stops waiting.
    let mut builds = Arc::clone(&state.map_images.builds).lock_owned().await;
    let task_state = Arc::clone(&state);
    let task_lang = lang.clone();
    let answer = tokio::task::spawn_blocking(move || -> Result<Value, Refusal> {
        let (state, lang) = (task_state, task_lang);
        let computed = compute(&state.dir, &mut builds, content, &lang)?;
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

/// Whether this page's run of the folder is going on (a run of another process holds the folder
/// lock instead).
fn running(state: &AppState) -> bool {
    state
        .hub
        .current_for(&state.dir)
        .is_some_and(|job| job.state == JobState::Running)
}

/// A live preview of the project in `dir` with `content` (none: the saved project's), from the
/// cached build in `builds`, which is built again when its stamp is out of date. The stored data
/// is read under the folder lock only, so no run of another process writes it meanwhile.
fn compute(
    dir: &Path,
    builds: &mut Builds,
    content: Option<ContentSettings>,
    lang: &Lang,
) -> Result<Computed, Refusal> {
    let text = |id: &str| TEXTS.text(lang, id, &[]);
    if !dir.join("freemapper.toml").is_file() {
        return Err(Refusal::Conflict(text("cli-map-image-none")));
    }
    let lock = crate::lock::acquire(dir, crate::lock::Who::Page)
        .map_err(|e| Refusal::Conflict(message(&e, lang)))?;
    let project = ProjectFile::load(dir).map_err(|e| Refusal::Failed(message(&e, lang)))?;
    let record = read_map_preview_record(dir)
        .filter(|_| map_preview_path(dir).is_file())
        .ok_or_else(|| Refusal::Conflict(text("cli-map-image-none")))?;
    if !fits(&record.fingerprint, &project) {
        return Err(Refusal::Conflict(text("cli-map-image-stale")));
    }
    let stamp = Stamp {
        fingerprint: record.fingerprint,
        files: FilesStamp::of(dir),
    };
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
            &slot.insert((stamp, built)).1
        }
    };
    // Placing reads no stored data.
    drop(lock);
    let content = content.unwrap_or_else(|| project.content.clone().unwrap_or_default());
    let documents = place(built, &content).map_err(|e| Refusal::Failed(message(&e, lang)))?;
    let image =
        map_preview_webp(&documents, dir).map_err(|e| Refusal::Failed(message(&e, lang)))?;
    Ok(Computed {
        webp: image.webp,
        width: image.width,
        height: image.height,
        content,
    })
}

#[cfg(test)]
mod tests;
