//! The area selection page: a local web server for one project folder. The user draws or picks the
//! map area in the browser; saving writes it to `freemapper.toml`.
//!
//! The server listens on 127.0.0.1 only. Every path starts with a random token, and requests for
//! another host name are refused, so other websites cannot reach it (also not by DNS rebinding).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use crate::errors::message;
use crate::estimate::estimate_json;
use crate::jobs::JobHub;
use crate::pipeline::RunOptions;
use crate::select_run;
use crate::select_run::{FolderOpener, PipelineRunner, Services};
use crate::text::{Lang, TEXTS, v};
use crate::tiles::{TileError, TileProxy};
use axum::Json;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Path as UrlPath, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use freemapper::area::MapArea;
use freemapper::build::content::{ContentSettings, Group, Place, Profile};
use freemapper::fetch::{Cache, HttpFetcher};
use freemapper::layers::{AreaPreview, preview_area};
use freemapper::places::{find_places, lookup_url, place_area};
use freemapper::project::{
    MAX_AREA_SIDE_M, MapKind, ProjectFile, init_project_with_content, map_path,
    set_map_area_with_content,
};
pub use freemapper_web::open_browser;
use freemapper_web::{LocalServer, ServeOptions, embedded};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Notify;

#[derive(RustEmbed)]
#[folder = "web/"]
pub(crate) struct Assets;

/// The content security policy of the page. Hosts can only be reached from this machine; the
/// page tells the user so. Images may come from the page itself and from the aerial photo
/// services the adapters name ([`preview_hosts`]), nothing else.
fn content_security_policy() -> &'static str {
    static POLICY: OnceLock<String> = OnceLock::new();
    POLICY.get_or_init(|| {
        format!(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
             connect-src 'self'; img-src 'self' data: {}; frame-ancestors 'none'",
            preview_hosts().join(" ")
        )
    })
}

/// The origins (`https://host`) of the aerial photo services of the adapters, each once, in the
/// order of the registry. Only an `https` address with a plain host name counts: the list goes
/// into the content security policy as it is.
fn preview_hosts() -> Vec<String> {
    let mut hosts: Vec<String> = Vec::new();
    for adapter in freemapper::adapters::registry().adapters() {
        let Some(preview) = &adapter.info().preview else {
            continue;
        };
        let host = preview
            .wms
            .strip_prefix("https://")
            .and_then(|rest| rest.split(['/', '?', '#']).next())
            .filter(|host| {
                !host.is_empty()
                    && host
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            });
        if let Some(host) = host {
            let origin = format!("https://{host}");
            if !hosts.contains(&origin) {
                hosts.push(origin);
            }
        }
    }
    hosts
}

/// The regions freemapper supports, for the page: `[{ id, names: { en, de }, preview }]` in the
/// order of the registry. `preview` is the aerial photo of the region for the map (`wms`,
/// `layer`, `format`, `params`, `attribution`, `agency`), or null.
async fn adapters() -> Response {
    let list: Vec<Value> = freemapper::adapters::registry()
        .adapters()
        .map(|adapter| {
            let info = adapter.info();
            json!({
                "id": info.id.as_str(),
                "names": { "en": info.names.en, "de": info.names.de },
                "preview": info.preview.as_ref().map(|preview| json!({
                    "wms": preview.wms,
                    "layer": preview.layer,
                    "format": preview.format,
                    "params": preview
                        .params
                        .iter()
                        .map(|(name, value)| (name.clone(), Value::from(value.as_str())))
                        .collect::<serde_json::Map<String, Value>>(),
                    "attribution": preview.attribution,
                    "agency": preview.agency,
                })),
            })
        })
        .collect();
    Json(list).into_response()
}

/// Settings for a project the page creates (an existing project keeps its own).
#[derive(Debug, Clone)]
pub struct Defaults {
    pub kind: MapKind,
    pub scale: Option<u32>,
    pub title: Option<String>,
    /// The page was opened by an AI agent: after "done" the user goes back to the agent.
    pub from_agent: bool,
    /// What the options of the run panel start with (the flags of `select`, ADR 0090). The
    /// browser stores nothing: its address changes with every start. `force` is always false.
    pub run: RunOptions,
}

pub(crate) struct AppState {
    pub(crate) dir: PathBuf,
    defaults: Defaults,
    pub(crate) cache: Cache,
    /// Saving is refused while a pipeline runs for the folder, because the pipeline would write
    /// its own copy of the project file back.
    pub(crate) hub: Arc<JobHub>,
    /// Saves one after the other.
    saving: tokio::sync::Mutex<()>,
    /// The OpenStreetMap base layer, fetched here with an identifying `User-Agent`.
    tiles: TileProxy,
    /// What a run does and how the folder is opened; replaced in tests.
    pub(crate) runner: Arc<dyn PipelineRunner>,
    pub(crate) opener: Arc<dyn FolderOpener>,
    /// When the last status request was served, and the shortest gap between two.
    pub(crate) last_status: std::sync::Mutex<Option<std::time::Instant>>,
    pub(crate) status_gap: std::time::Duration,
}

/// Largest request body of the page: a polygon of 5000 vertices is about 150 KB, and the shared
/// crate's default of 1 MiB leaves room for the places and options to come.
const MAX_BODY: usize = 1 << 20;

/// A running area selection page.
pub struct Selector {
    pub url: String,
    server: LocalServer,
    /// Set when the user presses "done".
    stopped: Arc<AtomicBool>,
}

impl Selector {
    /// Starts the page for `dir` on 127.0.0.1 (`port` 0 picks a free one).
    pub async fn start(
        dir: PathBuf,
        defaults: Defaults,
        cache: Cache,
        port: u16,
        hub: Option<Arc<JobHub>>,
    ) -> anyhow::Result<Self> {
        Self::start_with(dir, defaults, cache, port, hub, Services::real()).await
    }

    /// Like [`Selector::start`] with the parts that touch the network, the pipeline and the
    /// operating system given (tests pass stand-ins).
    pub(crate) async fn start_with(
        dir: PathBuf,
        defaults: Defaults,
        cache: Cache,
        port: u16,
        hub: Option<Arc<JobHub>>,
        services: Services,
    ) -> anyhow::Result<Self> {
        let state = Arc::new(AppState {
            dir,
            defaults,
            cache,
            hub: hub.unwrap_or_default(),
            saving: tokio::sync::Mutex::new(()),
            tiles: services.tiles,
            runner: services.runner,
            opener: services.opener,
            last_status: std::sync::Mutex::new(None),
            status_gap: services.status_gap,
        });
        let options = ServeOptions {
            port,
            strict_post: true,
            max_body: MAX_BODY,
            ..ServeOptions::new(content_security_policy())
        };
        let stopped = Arc::new(AtomicBool::new(false));
        let route_stopped = Arc::clone(&stopped);
        let server = freemapper_web::serve(
            |prefix, done| router(prefix, state, done, route_stopped),
            options,
        )
        .await
        .map_err(|e| {
            if port == 0 {
                anyhow::anyhow!("{e}")
            } else {
                anyhow::anyhow!("{e}; leave out --port to pick a free port")
            }
        })?;
        Ok(Self {
            url: server.url.clone(),
            server,
            stopped,
        })
    }

    /// Whether the page still serves (the user has not pressed "done").
    pub fn is_running(&self) -> bool {
        !self.stopped.load(Ordering::SeqCst)
    }

    /// Waits until the user presses "done" on the page.
    pub async fn finished(self) -> anyhow::Result<()> {
        self.server.finished().await?;
        Ok(())
    }
}

/// The routes of the page below `prefix` (`/{token}`); the shared crate adds the guards.
/// "Done" notifies `done`, which stops the server.
fn router(
    prefix: &str,
    state: Arc<AppState>,
    done: Arc<Notify>,
    stopped: Arc<AtomicBool>,
) -> Router {
    Router::new()
        .route(&format!("{prefix}/"), get(index))
        .route(&format!("{prefix}/assets/{{*path}}"), get(asset))
        .route(
            &format!("{prefix}/tiles/osm/{{z}}/{{x}}/{{y}}"),
            get(osm_tile),
        )
        .route(&format!("{prefix}/api/texts"), get(texts))
        .route(&format!("{prefix}/api/adapters"), get(adapters))
        .route(&format!("{prefix}/api/project"), get(project))
        .route(&format!("{prefix}/api/update"), get(update_notice))
        .route(&format!("{prefix}/api/preview"), post(preview))
        .route(&format!("{prefix}/api/places"), get(places))
        .route(&format!("{prefix}/api/place"), get(place))
        .route(&format!("{prefix}/api/save"), post(save))
        .route(
            &format!("{prefix}/api/run"),
            post(select_run::run).layer(DefaultBodyLimit::max(select_run::MAX_BODY_SMALL)),
        )
        .route(&format!("{prefix}/api/status"), get(select_run::status))
        .route(
            &format!("{prefix}/api/cancel"),
            post(select_run::cancel).layer(DefaultBodyLimit::max(select_run::MAX_BODY_SMALL)),
        )
        .route(
            &format!("{prefix}/api/open-folder"),
            post(select_run::open_folder).layer(DefaultBodyLimit::max(select_run::MAX_BODY_SMALL)),
        )
        .route(
            &format!("{prefix}/api/done"),
            post(move || async move {
                stopped.store(true, Ordering::SeqCst);
                done.notify_one();
                Json(json!({ "done": true }))
            }),
        )
        .with_state(state)
}

/// A JSON error the page shows to the user.
fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

/// The answer to a request whose task stopped unexpectedly, in the page's language.
fn internal_error(lang: &Lang, error: impl ToString) -> Response {
    bad_request(TEXTS.text(lang, "cli-internal-error", &[("message", v(error))]))
}

/// One tile of the OpenStreetMap base layer. The address is checked before anything is fetched.
async fn osm_tile(
    State(state): State<Arc<AppState>>,
    UrlPath((z, x, y)): UrlPath<(String, String, String)>,
) -> Response {
    match state.tiles.get(&z, &x, &y).await {
        Ok(tile) => {
            let max_age = tile.remaining.as_secs();
            let mut response =
                ([(header::CONTENT_TYPE, tile.content_type)], tile.bytes).into_response();
            if let Ok(value) = HeaderValue::from_str(&format!("private, max-age={max_age}")) {
                response.headers_mut().insert(header::CACHE_CONTROL, value);
            }
            response
        }
        Err(TileError::BadRequest(message)) => bad_request(message),
        Err(TileError::Upstream(message)) => {
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": message }))).into_response()
        }
    }
}

async fn index() -> Response {
    embedded::<Assets>("index.html")
}

async fn asset(UrlPath(path): UrlPath<String>) -> Response {
    embedded::<Assets>(&path)
}

/// The page's language from the request's `Accept-Language`, unless `--lang` or the environment
/// named one (ADR 0092, core ADR 0030).
pub(crate) fn page_lang(headers: &HeaderMap) -> Lang {
    let accept = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok());
    crate::text::page_lang(accept)
}

/// Every word of the page: `{ "lang": "de" | "en", "texts": { id: text } }` (core ADR 0030).
async fn texts(headers: HeaderMap) -> Response {
    let lang = page_lang(&headers);
    Json(json!({ "lang": lang.code(), "texts": crate::text::page_texts(&lang) })).into_response()
}

fn kind_name(kind: MapKind) -> &'static str {
    kind.name()
}

fn parse_kind(kind: &str, lang: &Lang) -> Result<MapKind, String> {
    MapKind::from_name(kind)
        .ok_or_else(|| TEXTS.text(lang, "cli-save-unknown-kind", &[("kind", v(kind))]))
}

/// The saved project, if there is one; an error in `lang` when its file cannot be read.
pub(crate) fn saved_project(state: &AppState, lang: &Lang) -> Result<Option<ProjectFile>, String> {
    if !state.dir.join("freemapper.toml").exists() {
        return Ok(None);
    }
    ProjectFile::load(&state.dir)
        .map(Some)
        .map_err(|e| message(&e, lang))
}

/// The options of the run panel as the page starts them.
fn run_defaults_json(options: &RunOptions) -> Value {
    json!({
        "force": options.force,
        "offline": options.offline,
        "ocad": options.ocad,
        "pdf": options.pdf,
        "pdf_candidates": options.pdf_candidates,
        "pdf_scale_bar": options.pdf_scale_bar,
        "pdf_sources": crate::pipeline::pdf_sources_name(options.pdf_sources),
    })
}

/// The stored content settings (ADR 0100) as the page reads them, or null while the file has none.
fn content_json(content: Option<&ContentSettings>) -> Value {
    content.map_or(Value::Null, |c| {
        serde_json::to_value(c).unwrap_or(Value::Null)
    })
}

/// The profiles, groups and places the content section lists, by their names in `freemapper.toml`
/// and in the library's order. The page names them with its own texts (ADR 0101).
fn content_options_json() -> Value {
    json!({
        "profiles": Profile::ALL.map(Profile::name),
        "groups": Group::ALL.map(Group::name),
        "places": Place::ALL.map(Place::name),
    })
}

/// The `content` of a save request: none when the page sends none (the stored settings stay), an
/// error sentence when it is not a valid choice.
fn content_of(body: &Value, lang: &Lang) -> Result<Option<ContentSettings>, SaveError> {
    match body.get("content") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|_| SaveError::Refused(TEXTS.text(lang, "cli-save-bad-content", &[]))),
    }
}

/// The revision of `freemapper.toml` in `dir` (core ADR 0030), or `None` while there is none.
fn project_revision(dir: &Path) -> Option<String> {
    std::fs::read(dir.join("freemapper.toml"))
        .ok()
        .map(|bytes| freemapper_web::revision(&bytes))
}

async fn project(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    // Read before the project: should the file change in between, the page holds the older
    // revision, and its next save is refused instead of overwriting.
    let revision = project_revision(&state.dir);
    let saved = match saved_project(&state, &page_lang(&headers)) {
        Ok(saved) => saved,
        Err(e) => return bad_request(e),
    };
    let mut answer = match saved {
        Some(project) => json!({
            "exists": true,
            "dir": state.dir,
            "kind": kind_name(project.map_kind),
            "scale": project.scale_denominator,
            "title": project.title,
            "buffer_m": project.buffer_m,
            "area": project.map_area,
            "source_id": project.map_area_source_id,
            "has_map": map_path(&state.dir).exists(),
            "max_side_m": MAX_AREA_SIDE_M,
            "from_agent": state.defaults.from_agent,
            "content": content_json(project.content.as_ref()),
        }),
        None => json!({
            "exists": false,
            "dir": state.dir,
            "kind": kind_name(state.defaults.kind),
            "scale": state.defaults.scale.unwrap_or(state.defaults.kind.default_scale()),
            "title": state.defaults.title,
            "buffer_m": state.defaults.kind.default_buffer_m(),
            "area": null,
            "source_id": null,
            "has_map": false,
            "max_side_m": MAX_AREA_SIDE_M,
            "from_agent": state.defaults.from_agent,
            "content": null,
        }),
    };
    answer["content_options"] = content_options_json();
    answer["run_defaults"] = run_defaults_json(&state.defaults.run);
    answer["allowed_scales"] = MapKind::ALL
        .into_iter()
        .map(|kind| (kind.name().to_string(), json!(kind.allowed_scales())))
        .collect::<serde_json::Map<_, _>>()
        .into();
    answer["running"] = crate::select_run::running_json(&state);
    answer["revision"] = revision.map_or(Value::Null, Value::from);
    Json(answer).into_response()
}

/// The update notice of the page (spec 3.7): the newer version that the record of the daily check
/// holds, or null. Reads the record only and sends no request.
async fn update_notice(State(state): State<Arc<AppState>>) -> Response {
    use crate::update::background::{CheckContext, recorded_update};
    let ctx = CheckContext::of_command(Some(state.cache.root().to_path_buf()), false);
    // The record is a file in the cache; only a link to the release page reaches the page.
    let available = recorded_update(&ctx).map(|(version, notes)| {
        let notes = notes.starts_with("https://github.com/").then_some(notes);
        json!({ "version": version.to_string(), "notes": notes })
    });
    Json(json!({ "available": available })).into_response()
}

/// The map area in a request body: `area` (GeoJSON) and optionally `source_id` (`way/…`). The
/// error is in `lang`.
fn area_of(body: &Value, lang: &Lang) -> Result<MapArea, String> {
    let mut area =
        MapArea::from_geojson(&body["area"].to_string()).map_err(|e| message(&e, lang))?;
    area.validate().map_err(|e| message(&e, lang))?;
    if let Some(source_id) = body["source_id"].as_str() {
        lookup_url(source_id).map_err(|e| message(&e, lang))?;
        area.source_id = Some(source_id.to_string());
    }
    Ok(area)
}

fn preview_json(preview: &AreaPreview, cache: &Cache, dir: &Path) -> Value {
    let tiles: Vec<Value> = preview
        .tiles
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "corners": t.corners.iter().map(|c| [c.lat, c.lon]).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "tiles": tiles,
        "width_m": preview.width_m,
        "height_m": preview.height_m,
        "too_large": preview.too_large,
        "outside_coverage": preview.outside_coverage,
        "max_side_m": MAX_AREA_SIDE_M,
        "estimate": estimate_json(&preview.estimate, cache, dir),
    })
}

async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let today = chrono::Local::now().date_naive();
    let lang = page_lang(&headers);
    let task_lang = lang.clone();
    // Validating a large outline takes a while: off the async workers.
    let answer = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let lang = task_lang;
        let area = area_of(&body, &lang)?;
        // An existing project keeps its buffer; a new one gets the buffer of the kind chosen
        // on the page.
        let buffer_m = match saved_project(&state, &lang)? {
            Some(project) => project.buffer_m,
            None => match body["kind"].as_str() {
                Some(kind) => parse_kind(kind, &lang)?.default_buffer_m(),
                None => state.defaults.kind.default_buffer_m(),
            },
        };
        let preview = preview_area(&area, buffer_m, &state.cache, today);
        Ok(preview_json(&preview, &state.cache, &state.dir))
    })
    .await;
    match answer {
        Ok(Ok(answer)) => Json(answer).into_response(),
        Ok(Err(e)) => bad_request(e),
        Err(e) => internal_error(&lang, e),
    }
}

#[derive(Deserialize)]
struct PlacesQuery {
    q: String,
}

async fn places(headers: HeaderMap, Query(query): Query<PlacesQuery>) -> Response {
    let lang = page_lang(&headers);
    crate::nominatim::wait_turn().await;
    let found =
        tokio::task::spawn_blocking(move || find_places(&HttpFetcher::new(), &query.q)).await;
    match found {
        Ok(Ok(places)) => Json(
            places
                .iter()
                .map(|p| {
                    json!({
                        "osm_id": p.osm_id,
                        "name": p.name,
                        "display_name": p.display_name,
                        "category": p.category,
                        "kind": p.kind,
                        "has_area": p.has_area,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Ok(Err(e)) => bad_request(message(&e, &lang)),
        Err(e) => internal_error(&lang, e),
    }
}

#[derive(Deserialize)]
struct PlaceQuery {
    osm_id: String,
}

async fn place(headers: HeaderMap, Query(query): Query<PlaceQuery>) -> Response {
    let lang = page_lang(&headers);
    crate::nominatim::wait_turn().await;
    let osm_id = query.osm_id.clone();
    let found =
        tokio::task::spawn_blocking(move || place_area(&HttpFetcher::new(), &query.osm_id, None))
            .await;
    match found {
        Ok(Ok(area)) => {
            Json(json!({ "area": area.to_geojson_value(), "source_id": osm_id })).into_response()
        }
        Ok(Err(e)) => bad_request(message(&e, &lang)),
        Err(e) => internal_error(&lang, e),
    }
}

async fn save(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    // The revision of freemapper.toml as the page read it (core ADR 0030); none before the file
    // exists.
    let seen = headers
        .get(freemapper_web::REVISION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let lang = page_lang(&headers);
    if state.hub.running_for(&state.dir) {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": TEXTS.text(&lang, "cli-save-running", &[]) })),
        )
            .into_response();
    }
    let _saving = state.saving.lock().await;
    let task_state = Arc::clone(&state);
    let task_lang = lang.clone();
    let saved = tokio::task::spawn_blocking(move || -> Result<Value, SaveError> {
        let (state, lang) = (task_state, task_lang);
        // A folder that exists may be in use by a terminal or the MCP server; a new one cannot be.
        let _lock = if state.dir.is_dir() {
            Some(
                crate::lock::acquire(&state.dir, crate::lock::Who::Page)
                    .map_err(|e| SaveError::Locked(message(&e, &lang)))?,
            )
        } else {
            None
        };
        let dir = &state.dir;
        // Another window or program wrote the file since the page read it: write nothing. The
        // check comes under the folder lock, so no run of this program writes in between.
        if let Some(current) = project_revision(dir)
            && seen.as_deref() != Some(current.as_str())
        {
            return Err(SaveError::Stale(current));
        }
        let content = content_of(&body, &lang)?;
        let area = area_of(&body, &lang)?;
        let created = !dir.join("freemapper.toml").exists();
        let mut zone_changed = false;
        let mut provider_changed = false;
        if created {
            let kind = match body["kind"].as_str() {
                Some(kind) => parse_kind(kind, &lang)?,
                None => state.defaults.kind,
            };
            let scale = body["scale"]
                .as_u64()
                .map(|s| {
                    u32::try_from(s).map_err(|_| {
                        TEXTS.text(&lang, "cli-save-scale-too-large", &[("scale", v(s))])
                    })
                })
                .transpose()?
                .or(state.defaults.scale);
            let title = body["title"]
                .as_str()
                .map(str::to_string)
                .or(state.defaults.title.clone());
            let today = chrono::Local::now().date_naive();
            init_project_with_content(dir, &area, kind, scale, today, title, content)
                .map_err(|e| message(&e, &lang))?;
        } else {
            let saved =
                set_map_area_with_content(dir, &area, content).map_err(|e| message(&e, &lang))?;
            zone_changed = saved.zone_changed;
            provider_changed = saved.provider_changed;
        }
        Ok(json!({
            "saved": true,
            "created": created,
            // The new area lies in another UTM zone: what was fetched for the old one is dropped.
            "zone_changed": zone_changed,
            // Another provider serves the new area: its data is fetched again as well.
            "provider_changed": provider_changed,
            "dir": dir,
            "has_map": map_path(dir).exists(),
            // The page compares it with the revision it reads next: another one means that
            // another window or program wrote the file in between.
            "revision": project_revision(dir),
        }))
    })
    .await;
    match saved {
        Ok(Ok(answer)) => Json(answer).into_response(),
        Ok(Err(SaveError::Locked(e))) => {
            (StatusCode::CONFLICT, Json(json!({ "error": e }))).into_response()
        }
        Ok(Err(SaveError::Stale(current))) => {
            freemapper_web::stale(&current, &TEXTS.text(&lang, "page-save-stale", &[]))
        }
        Ok(Err(SaveError::Refused(e))) => freemapper_web::refused(&e),
        Ok(Err(SaveError::Other(e))) => bad_request(e),
        Err(e) => internal_error(&lang, e),
    }
}

/// Why a save did not happen.
enum SaveError {
    /// Another freemapper run holds the folder.
    Locked(String),
    /// The file changed since the page read it; its revision now.
    Stale(String),
    /// The content settings are not a valid choice (422).
    Refused(String),
    Other(String),
}

impl From<String> for SaveError {
    fn from(message: String) -> Self {
        SaveError::Other(message)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use freemapper::project::init_project;

    #[tokio::test(flavor = "multi_thread")]
    async fn save_is_refused_while_a_pipeline_runs() {
        let dir = tempfile::tempdir().unwrap();
        let hub = Arc::new(JobHub::default());
        hub.register(&dir.path().join("p")).unwrap();
        let defaults = Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: true,
            run: RunOptions::default(),
        };
        let cache = Cache::new(dir.path().join("cache"));
        let selector = Selector::start(dir.path().join("p"), defaults, cache, 0, Some(hub))
            .await
            .unwrap();
        let url = format!("{}api/save", selector.url);
        let origin = origin_of(&selector.url);
        let (english, german) = tokio::task::spawn_blocking(move || {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let area = r#"{"area":{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.539]]]}}"#;
            // The refusal comes in the page's language; nothing names one in a unit test.
            let save = |accept: &str| {
                let mut response = agent
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .header("Origin", &origin)
                    .header("Accept-Language", accept)
                    .send(area)
                    .unwrap();
                let status = response.status().as_u16();
                (status, response.body_mut().read_to_string().unwrap())
            };
            (save("en-US,en"), save("de-DE,de;q=0.9"))
        })
        .await
        .unwrap();
        for ((status, text), expected) in [
            (english, "A run is going on for this project."),
            (german, "Für dieses Projekt läuft gerade ein Lauf"),
        ] {
            assert_eq!(status, 409, "{text}");
            assert!(text.contains(expected), "{text}");
        }
        assert!(!dir.path().join("p").exists());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn save_is_refused_while_another_process_holds_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("p");
        let area = freemapper::area::MapArea::from_geojson(
            &std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/clara-zetkin-schule.geojson"
            ))
            .unwrap(),
        )
        .unwrap();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
        init_project(&project, &area, MapKind::Schoolyard, None, date, None).unwrap();
        let before = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
        let held = crate::lock::acquire(&project, crate::lock::Who::Cli).unwrap();
        let defaults = Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: false,
            run: RunOptions::default(),
        };
        let cache = Cache::new(dir.path().join("cache"));
        let selector = Selector::start(project.clone(), defaults, cache, 0, None)
            .await
            .unwrap();
        let url = format!("{}api/save", selector.url);
        let origin = origin_of(&selector.url);
        // The page names the revision of the file it read (core ADR 0030).
        let seen = freemapper_web::revision(before.as_bytes());
        let post = |url: String, origin: String, seen: String, accept: &str| {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let body = r#"{"area":{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.539]]]}}"#;
            let mut response = agent
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Origin", &origin)
                .header(freemapper_web::REVISION_HEADER, &seen)
                .header("Accept-Language", accept)
                .send(body)
                .unwrap();
            (
                response.status().as_u16(),
                response.body_mut().read_to_string().unwrap(),
            )
        };
        let (u, o) = (url.clone(), origin.clone());
        let s = seen.clone();
        let (status, text) = tokio::task::spawn_blocking(move || post(u, o, s, "en"))
            .await
            .unwrap();
        assert_eq!(status, 409, "{text}");
        assert!(
            text.contains("another freemapper run is using this folder"),
            "{text}"
        );
        // In the page's language, also when the program's is another.
        let (u, o) = (url.clone(), origin.clone());
        let s = seen.clone();
        let (status, german) = tokio::task::spawn_blocking(move || post(u, o, s, "de-DE,de"))
            .await
            .unwrap();
        assert_eq!(status, 409, "{german}");
        assert!(
            german.contains("Ein anderer freemapper-Lauf verwendet diesen Ordner"),
            "{german}"
        );
        assert!(
            !text.contains(&dir.path().display().to_string()),
            "no path: {text}"
        );
        assert_eq!(
            std::fs::read_to_string(project.join("freemapper.toml")).unwrap(),
            before
        );
        // Let go, and the same save goes through; the page leaves no lock behind.
        drop(held);
        let (status, text) = tokio::task::spawn_blocking(move || post(url, origin, seen, "en"))
            .await
            .unwrap();
        assert_eq!(status, 200, "{text}");
        assert!(!project.join(crate::lock::LOCK_NAME).exists());
    }

    /// A stand-in for the tile server: answers every request with `status` and collects the
    /// request heads.
    struct StubTiles {
        base: String,
        heads: Arc<std::sync::Mutex<Vec<String>>>,
    }

    fn stub_tiles(status: u16, cache_control: &'static str) -> StubTiles {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let heads = Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen = Arc::clone(&heads);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                seen.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buf[..n]).to_string());
                let body = b"\x89PNG-stub";
                let head = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: image/png\r\nCache-Control: {cache_control}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        StubTiles { base, heads }
    }

    async fn selector_with(tiles: TileProxy) -> (Selector, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let defaults = Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: false,
            run: RunOptions::default(),
        };
        let cache = Cache::new(dir.path().join("cache"));
        let selector = Selector::start_with(
            dir.path().join("p"),
            defaults,
            cache,
            0,
            None,
            Services {
                tiles,
                ..Services::real()
            },
        )
        .await
        .unwrap();
        (selector, dir)
    }

    /// Status, content type, cache control, content security policy and body of a GET.
    async fn get(url: String) -> (u16, String, String, String, Vec<u8>) {
        tokio::task::spawn_blocking(move || {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let mut response = agent.get(&url).call().unwrap();
            let header = |name: &str| {
                response
                    .headers()
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default()
                    .to_string()
            };
            let content_type = header("content-type");
            let cache_control = header("cache-control");
            let csp = header("content-security-policy");
            let status = response.status().as_u16();
            let body = response.body_mut().read_to_vec().unwrap();
            (status, content_type, cache_control, csp, body)
        })
        .await
        .unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn tile_route_refuses_bad_addresses_without_asking_upstream() {
        let stub = stub_tiles(200, "max-age=3600");
        let (selector, _dir) = selector_with(TileProxy::with_base(&stub.base)).await;
        for path in [
            "tiles/osm/20/0/0.png",
            "tiles/osm/5/32/0.png",
            "tiles/osm/5/0/32.png",
            "tiles/osm/5/-1/0.png",
            "tiles/osm/5/0/0",
            "tiles/osm/5/0/0.jpg",
            "tiles/osm/x/0/0.png",
        ] {
            let (status, ..) = get(format!("{}{path}", selector.url)).await;
            assert_eq!(status, 400, "{path}");
        }
        assert!(stub.heads.lock().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn tile_route_fetches_with_an_identifying_user_agent_and_caches() {
        let stub = stub_tiles(200, "public, max-age=3600");
        let (selector, _dir) = selector_with(TileProxy::with_base(&stub.base)).await;
        let url = format!("{}tiles/osm/5/17/10.png", selector.url);
        let (status, content_type, cache_control, _, body) = get(url.clone()).await;
        assert_eq!(status, 200);
        assert_eq!(content_type, "image/png");
        assert!(
            cache_control.starts_with("private, max-age="),
            "{cache_control}"
        );
        assert_eq!(body, b"\x89PNG-stub");
        // The second request comes from the cache.
        let (status, ..) = get(url).await;
        assert_eq!(status, 200);
        let heads = stub.heads.lock().unwrap();
        assert_eq!(heads.len(), 1, "{heads:?}");
        let head = heads[0].to_ascii_lowercase();
        assert!(head.starts_with("get /5/17/10.png "), "{head}");
        let agent = head
            .lines()
            .find_map(|l| l.strip_prefix("user-agent: "))
            .unwrap();
        assert!(agent.starts_with("freemapper/0."), "{agent}");
        assert!(
            agent.contains("https://github.com/henryjobst/freemapper"),
            "{agent}"
        );
        assert!(!head.contains('@'), "{head}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn tile_route_maps_an_upstream_refusal_to_502() {
        let stub = stub_tiles(403, "no-store");
        let (selector, _dir) = selector_with(TileProxy::with_base(&stub.base)).await;
        let (status, _, _, _, body) = get(format!("{}tiles/osm/5/17/10.png", selector.url)).await;
        assert_eq!(status, 502);
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("HTTP 403"), "{text}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn page_does_not_load_tiles_from_the_tile_server_directly() {
        let stub = stub_tiles(200, "max-age=60");
        let (selector, _dir) = selector_with(TileProxy::with_base(&stub.base)).await;
        let (_, _, _, csp, _) = get(selector.url.clone()).await;
        assert!(!csp.contains("tile.openstreetmap.org"), "{csp}");
        assert!(csp.contains("img-src 'self'"), "{csp}");
        let app = Assets::get("app.js").unwrap();
        let app = String::from_utf8_lossy(&app.data);
        assert!(!app.contains("https://tile.openstreetmap.org"));
        assert!(app.contains("tiles/osm/{z}/{x}/{y}.png"));
    }

    /// `http://127.0.0.1:<port>` of a page address.
    pub(crate) fn origin_of(url: &str) -> String {
        let rest = url.strip_prefix("http://").unwrap();
        format!("http://{}", rest.split('/').next().unwrap())
    }

    /// The page address with its token replaced by `token`.
    fn with_token(url: &str, token: &str) -> String {
        let old = url.trim_end_matches('/').rsplit('/').next().unwrap();
        url.replace(old, token)
    }

    /// Status, lower-cased header block and body of a raw request. `extra` are header lines;
    /// `Host` is added unless one of them sets it.
    pub(crate) fn raw(
        url: &str,
        method: &str,
        path: &str,
        extra: &[String],
        body: &str,
    ) -> (u16, String, String) {
        use std::io::{Read, Write};
        let host = url
            .strip_prefix("http://")
            .unwrap()
            .split('/')
            .next()
            .unwrap();
        let prefix = &url[url.find(host).unwrap() + host.len()..];
        let mut head = format!("{method} {prefix}{path} HTTP/1.1\r\n");
        if !extra
            .iter()
            .any(|h| h.to_ascii_lowercase().starts_with("host:"))
        {
            head.push_str(&format!("Host: {host}\r\n"));
        }
        for line in extra {
            head.push_str(line);
            head.push_str("\r\n");
        }
        head.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        ));
        // The server may answer 403 or 413 and close before it has read the body; Windows then
        // resets the connection and drops the answer, so an empty answer is asked for again.
        let mut answer = Vec::new();
        for _ in 0..5 {
            let mut stream = std::net::TcpStream::connect(host).unwrap();
            stream.write_all(head.as_bytes()).unwrap();
            let _ = stream.write_all(body.as_bytes());
            answer.clear();
            let _ = stream.read_to_end(&mut answer);
            if !answer.is_empty() {
                break;
            }
        }
        let answer = String::from_utf8_lossy(&answer).to_string();
        let (head, body) = answer.split_once("\r\n\r\n").unwrap_or((&answer, ""));
        let status = head.split(' ').nth(1).unwrap().parse().unwrap();
        (status, head.to_ascii_lowercase(), body.to_string())
    }

    pub(crate) async fn request(
        url: &str,
        method: &'static str,
        path: &str,
        extra: Vec<String>,
        body: String,
    ) -> (u16, String, String) {
        let (url, path) = (url.to_string(), path.to_string());
        tokio::task::spawn_blocking(move || raw(&url, method, &path, &extra, &body))
            .await
            .unwrap()
    }

    pub(crate) fn json_post(origin: &str) -> Vec<String> {
        vec![
            format!("Origin: {origin}"),
            "Content-Type: application/json".to_string(),
        ]
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn every_embedded_asset_has_its_content_type_and_nosniff() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        let mut seen = std::collections::BTreeSet::new();
        let mut paths: Vec<String> = Assets::iter().map(|p| p.to_string()).collect();
        paths.sort();
        assert!(paths.len() >= 11, "{paths:?}");
        for path in paths {
            let url = if path == "index.html" {
                String::new()
            } else {
                format!("assets/{path}")
            };
            let (status, head, body) =
                request(&selector.url, "GET", &url, vec![], String::new()).await;
            assert_eq!(status, 200, "{path}");
            assert!(!body.is_empty(), "{path}");
            assert!(
                head.contains("x-content-type-options: nosniff"),
                "{path}: {head}"
            );
            let content_type = head
                .lines()
                .find_map(|l| l.strip_prefix("content-type: "))
                .unwrap_or_else(|| panic!("{path}: no content type in {head}"))
                .to_string();
            let extension = path.rsplit_once('.').map_or("", |(_, e)| e);
            let expected = match extension {
                "html" => "text/html; charset=utf-8",
                "js" => "text/javascript; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "png" => "image/png",
                // No extension (the Leaflet licence): anything but a type a browser would run.
                _ => {
                    assert!(
                        !content_type.contains("html") && !content_type.contains("script"),
                        "{path}: {content_type}"
                    );
                    continue;
                }
            };
            assert_eq!(content_type, expected, "{path}");
            seen.insert(extension.to_string());
        }
        assert_eq!(seen.len(), 4, "{seen:?}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn asset_paths_cannot_leave_the_embedded_folder() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        for path in [
            "assets/../Cargo.toml",
            "assets/%2e%2e/index.html",
            "assets/missing.js",
        ] {
            let (status, ..) = request(&selector.url, "GET", path, vec![], String::new()).await;
            assert!(status == 404 || status == 400, "{path}: {status}");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn post_without_origin_is_refused() {
        let (selector, dir) = selector_with(TileProxy::new()).await;
        let extra = vec!["Content-Type: application/json".to_string()];
        let (status, ..) = request(&selector.url, "POST", "api/save", extra, "{}".into()).await;
        assert_eq!(status, 403);
        assert!(!dir.path().join("p").exists());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn post_with_a_form_content_type_is_refused() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        let origin = origin_of(&selector.url);
        for content_type in ["text/plain", "application/x-www-form-urlencoded"] {
            let extra = vec![
                format!("Origin: {origin}"),
                format!("Content-Type: {content_type}"),
            ];
            let (status, ..) = request(&selector.url, "POST", "api/done", extra, "{}".into()).await;
            assert_eq!(status, 403, "{content_type}");
        }
        // Without a content type at all.
        let extra = vec![format!("Origin: {origin}")];
        let (status, ..) = request(&selector.url, "POST", "api/done", extra, String::new()).await;
        assert_eq!(status, 403);
        assert!(selector.is_running());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn post_with_a_foreign_origin_is_refused() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        for origin in ["http://evil.example", "http://127.0.0.1:1", "null"] {
            let (status, ..) = request(
                &selector.url,
                "POST",
                "api/done",
                json_post(origin),
                "{}".into(),
            )
            .await;
            assert_eq!(status, 403, "{origin}");
        }
        assert!(selector.is_running());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cross_site_fetch_metadata_is_refused_but_navigation_works() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        let cross = vec!["Sec-Fetch-Site: cross-site".to_string()];
        let (status, ..) = request(&selector.url, "GET", "", cross, String::new()).await;
        assert_eq!(status, 403);
        let mut post = json_post(&origin_of(&selector.url));
        post.push("Sec-Fetch-Site: cross-site".into());
        let (status, ..) = request(&selector.url, "POST", "api/done", post, "{}".into()).await;
        assert_eq!(status, 403);
        // Typing the address into the browser (or `open`) sends `none`; the page's own
        // requests send `same-origin`.
        for site in ["none", "same-origin"] {
            let extra = vec![
                format!("Sec-Fetch-Site: {site}"),
                "Sec-Fetch-Mode: navigate".to_string(),
            ];
            let (status, ..) = request(&selector.url, "GET", "", extra, String::new()).await;
            assert_eq!(status, 200, "{site}");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn requests_for_another_host_name_are_refused() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        for host in [
            "evil.example",
            "evil.example:80",
            "127.0.0.1",
            "127.0.0.1:1",
        ] {
            let extra = vec![format!("Host: {host}")];
            let (status, ..) =
                request(&selector.url, "GET", "api/project", extra, String::new()).await;
            assert_eq!(status, 403, "{host}");
        }
        let origin = origin_of(&selector.url);
        let port = origin.rsplit(':').next().unwrap();
        let extra = vec![format!("Host: localhost:{port}")];
        let (status, ..) = request(&selector.url, "GET", "api/project", extra, String::new()).await;
        assert_eq!(status, 200);
        // A wrong token is a 404, not a page.
        let wrong = with_token(&selector.url, &"0".repeat(32));
        let (status, ..) = request(&wrong, "GET", "", vec![], String::new()).await;
        assert_eq!(status, 404);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn done_stops_the_server() {
        let (selector, _dir) = selector_with(TileProxy::new()).await;
        let origin = origin_of(&selector.url);
        let (status, _, body) = request(
            &selector.url,
            "POST",
            "api/done",
            json_post(&origin),
            "{}".into(),
        )
        .await;
        assert_eq!(status, 200);
        assert!(body.contains("\"done\":true"), "{body}");
        tokio::time::timeout(std::time::Duration::from_secs(10), selector.finished())
            .await
            .expect("the server stops after done")
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn tile_route_is_protected_like_the_rest() {
        let stub = stub_tiles(200, "max-age=3600");
        let (selector, _dir) = selector_with(TileProxy::with_base(&stub.base)).await;
        let path = "tiles/osm/5/17/10.png";
        let foreign = vec!["Host: evil.example".to_string()];
        let (status, ..) = request(&selector.url, "GET", path, foreign, String::new()).await;
        assert_eq!(status, 403);
        let cross = vec!["Sec-Fetch-Site: cross-site".to_string()];
        let (status, ..) = request(&selector.url, "GET", path, cross, String::new()).await;
        assert_eq!(status, 403);
        let wrong = with_token(&selector.url, &"1".repeat(32));
        let (status, ..) = request(&wrong, "GET", path, vec![], String::new()).await;
        assert_eq!(status, 404);
        assert!(stub.heads.lock().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_polygon_of_5000_vertices_fits_the_body_limit() {
        let (selector, dir) = selector_with(TileProxy::new()).await;
        let mut ring: Vec<String> = (0..5000)
            .map(|i| {
                let a = f64::from(i) / 5000.0 * std::f64::consts::TAU;
                format!(
                    "[{:.7},{:.7}]",
                    13.8445 + 0.0006 * a.cos(),
                    52.5400 + 0.0004 * a.sin()
                )
            })
            .collect();
        ring.push(ring[0].clone());
        let body = format!(
            r#"{{"area":{{"type":"Polygon","coordinates":[[{}]]}}}}"#,
            ring.join(",")
        );
        assert!(
            body.len() > 100_000 && body.len() < 1 << 20,
            "{}",
            body.len()
        );
        let origin = origin_of(&selector.url);
        let (status, _, answer) = request(
            &selector.url,
            "POST",
            "api/save",
            json_post(&origin),
            body.clone(),
        )
        .await;
        assert_eq!(status, 200, "{answer}");
        assert!(dir.path().join("p").join("freemapper.toml").exists());
        // Above the limit the request is refused with 413, not read.
        let huge = format!("{body}{}", " ".repeat(1 << 20));
        let (status, ..) = request(
            &selector.url,
            "POST",
            "api/preview",
            json_post(&origin),
            huge,
        )
        .await;
        assert_eq!(status, 413);
    }

    /// A page for `defaults`, with the hub when given.
    async fn page_with(
        defaults: Defaults,
        hub: Option<Arc<JobHub>>,
    ) -> (Selector, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let selector = Selector::start(dir.path().join("p"), defaults, cache, 0, hub)
            .await
            .unwrap();
        (selector, dir)
    }

    fn plain_defaults() -> Defaults {
        Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: false,
            run: crate::pipeline::RunOptions::default(),
        }
    }

    async fn project_json(selector: &Selector) -> Value {
        let (status, _, body) =
            request(&selector.url, "GET", "api/project", vec![], String::new()).await;
        assert_eq!(status, 200, "{body}");
        serde_json::from_str(&body).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn project_answer_has_run_defaults() {
        let run = crate::pipeline::RunOptions {
            ocad: true,
            pdf: true,
            pdf_candidates: true,
            pdf_scale_bar: true,
            pdf_sources: freemapper::project::PdfSources::CreditOnly,
            offline: true,
            ..Default::default()
        };
        let defaults = Defaults {
            run,
            ..plain_defaults()
        };
        let (selector, _dir) = page_with(defaults, None).await;
        let answer = project_json(&selector).await;
        assert_eq!(
            answer["run_defaults"],
            json!({
                "force": false,
                "offline": true,
                "ocad": true,
                "pdf": true,
                "pdf_candidates": true,
                "pdf_scale_bar": true,
                "pdf_sources": "credit-only",
            })
        );
        // Without flags every option is off and the sources are automatic.
        let (selector, _dir) = page_with(plain_defaults(), None).await;
        let answer = project_json(&selector).await;
        assert_eq!(answer["run_defaults"]["pdf_sources"], "auto");
        assert_eq!(answer["run_defaults"]["pdf"], false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn allowed_scales_come_from_the_engine() {
        let (selector, _dir) = page_with(plain_defaults(), None).await;
        let answer = project_json(&selector).await;
        assert_eq!(
            answer["allowed_scales"],
            json!({
                "schoolyard": MapKind::Schoolyard.allowed_scales(),
                "sprint": MapKind::Sprint.allowed_scales(),
            })
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn running_is_null_without_a_job_and_describes_one_otherwise() {
        let hub = Arc::new(JobHub::default());
        let (selector, dir) = page_with(plain_defaults(), Some(Arc::clone(&hub))).await;
        let answer = project_json(&selector).await;
        assert!(answer["running"].is_null(), "{answer}");
        // A job of another project is not this page's run.
        let other = hub.register(&dir.path().join("other")).unwrap();
        assert!(project_json(&selector).await["running"].is_null());
        hub.cancel(&other);
        hub.run_registered(&other, Box::new(|_, _| Err("cancelled".to_string())));
        let id = hub.register(&dir.path().join("p")).unwrap();
        let answer = project_json(&selector).await;
        assert_eq!(answer["running"]["state"], "running", "{answer}");
        assert_eq!(answer["running"]["job_id"], id);
    }
}
