//! The area selection page: a local web server for one project folder. The user draws or picks the
//! map area in the browser; saving writes it to `freemapper.toml`.
//!
//! The server listens on 127.0.0.1 only. Every path starts with a random token, and requests for
//! another host name are refused, so other websites cannot reach it (also not by DNS rebinding).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path as UrlPath, Query, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, HttpFetcher};
use freemapper::layers::{AreaPreview, preview_area};
use freemapper::places::{find_places, lookup_url, place_area};
use freemapper::project::{
    MAX_AREA_SIDE_M, MapKind, ProjectFile, init_project, map_path, set_map_area,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Notify;

#[derive(RustEmbed)]
#[folder = "web/"]
struct Assets;

/// Hosts can only be reached from this machine; the page tells the user so.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
     style-src 'self' 'unsafe-inline'; connect-src 'self'; \
     img-src 'self' data: https://tile.openstreetmap.org https://isk.geobasis-bb.de; \
     frame-ancestors 'none'";

/// Settings for a project the page creates (an existing project keeps its own).
#[derive(Debug, Clone)]
pub struct Defaults {
    pub kind: MapKind,
    pub scale: Option<u32>,
    pub title: Option<String>,
}

struct AppState {
    dir: PathBuf,
    defaults: Defaults,
    cache: Cache,
    allowed_hosts: [String; 2],
    done: Notify,
}

/// A running area selection page.
pub struct Selector {
    pub url: String,
    server: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl Selector {
    /// Starts the page for `dir` on 127.0.0.1 (`port` 0 picks a free one).
    pub async fn start(
        dir: PathBuf,
        defaults: Defaults,
        cache: Cache,
        port: u16,
    ) -> anyhow::Result<Self> {
        let listener =
            tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await?;
        let port = listener.local_addr()?.port();
        let token = token()?;
        let state = Arc::new(AppState {
            dir,
            defaults,
            cache,
            allowed_hosts: [format!("127.0.0.1:{port}"), format!("localhost:{port}")],
            done: Notify::new(),
        });
        let app = router(&token, Arc::clone(&state));
        let shutdown = Arc::clone(&state);
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { shutdown.done.notified().await })
                .await
        });
        Ok(Self {
            url: format!("http://127.0.0.1:{port}/{token}/"),
            server,
        })
    }

    /// Waits until the user presses "done" on the page.
    pub async fn finished(self) -> anyhow::Result<()> {
        self.server.await??;
        Ok(())
    }
}

/// 128 random bits from the operating system, as hex.
fn token() -> anyhow::Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| anyhow::anyhow!("no random source: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn router(token: &str, state: Arc<AppState>) -> Router {
    let host_check = Arc::clone(&state);
    Router::new()
        .route(&format!("/{token}/"), get(index))
        .route(&format!("/{token}/assets/{{*path}}"), get(asset))
        .route(&format!("/{token}/api/project"), get(project))
        .route(&format!("/{token}/api/preview"), post(preview))
        .route(&format!("/{token}/api/places"), get(places))
        .route(&format!("/{token}/api/place"), get(place))
        .route(&format!("/{token}/api/save"), post(save))
        .route(&format!("/{token}/api/done"), post(done))
        .with_state(state)
        .layer(middleware::from_fn(move |request: Request, next: Next| {
            let state = Arc::clone(&host_check);
            async move {
                let host = request
                    .headers()
                    .get(header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or_default();
                if !state.allowed_hosts.iter().any(|allowed| allowed == host) {
                    return StatusCode::FORBIDDEN.into_response();
                }
                let mut response = next.run(request).await;
                let headers = response.headers_mut();
                headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
                headers.insert(
                    header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static(CONTENT_SECURITY_POLICY),
                );
                headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
                response
            }
        }))
}

/// A JSON error the page shows to the user.
fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

fn embedded(path: &str) -> Response {
    match Assets::get(path) {
        Some(file) => (
            [(header::CONTENT_TYPE, file.metadata.mimetype().to_string())],
            file.data.into_owned(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn index() -> Response {
    embedded("index.html")
}

async fn asset(UrlPath(path): UrlPath<String>) -> Response {
    embedded(&path)
}

fn kind_name(kind: MapKind) -> &'static str {
    match kind {
        MapKind::Schoolyard => "schoolyard",
        MapKind::Sprint => "sprint",
    }
}

fn parse_kind(kind: &str) -> Result<MapKind, String> {
    match kind {
        "schoolyard" => Ok(MapKind::Schoolyard),
        "sprint" => Ok(MapKind::Sprint),
        other => Err(format!(
            "unknown map kind {other:?}; use schoolyard or sprint"
        )),
    }
}

/// The saved project, if there is one.
fn saved_project(state: &AppState) -> Option<ProjectFile> {
    state
        .dir
        .join("freemapper.toml")
        .exists()
        .then(|| ProjectFile::load(&state.dir).ok())
        .flatten()
}

async fn project(State(state): State<Arc<AppState>>) -> Response {
    let answer = match saved_project(&state) {
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
        }),
    };
    Json(answer).into_response()
}

/// The map area in a request body: `area` (GeoJSON) and optionally `source_id` (`way/…`).
fn area_of(body: &Value) -> Result<MapArea, String> {
    let mut area = MapArea::from_geojson(&body["area"].to_string()).map_err(|e| e.to_string())?;
    area.validate().map_err(|e| e.to_string())?;
    if let Some(source_id) = body["source_id"].as_str() {
        lookup_url(source_id).map_err(|e| e.to_string())?;
        area.source_id = Some(source_id.to_string());
    }
    Ok(area)
}

fn preview_json(preview: &AreaPreview) -> Value {
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
    let items: Vec<Value> = preview
        .estimate
        .items
        .iter()
        .map(|i| json!({ "what": i.what, "megabytes": i.megabytes, "cached": i.cached }))
        .collect();
    json!({
        "tiles": tiles,
        "width_m": preview.width_m,
        "height_m": preview.height_m,
        "too_large": preview.too_large,
        "max_side_m": MAX_AREA_SIDE_M,
        "estimate": {
            "tiles": preview.estimate.tiles,
            "items": items,
            "total_megabytes": preview.estimate.total_megabytes,
        },
    })
}

async fn preview(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Response {
    let area = match area_of(&body) {
        Ok(area) => area,
        Err(e) => return bad_request(e),
    };
    let buffer_m = saved_project(&state)
        .map(|p| p.buffer_m)
        .unwrap_or(state.defaults.kind.default_buffer_m());
    let today = chrono::Local::now().date_naive();
    let answer = tokio::task::spawn_blocking(move || {
        preview_json(&preview_area(&area, buffer_m, &state.cache, today))
    })
    .await;
    match answer {
        Ok(answer) => Json(answer).into_response(),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

#[derive(Deserialize)]
struct PlacesQuery {
    q: String,
}

async fn places(Query(query): Query<PlacesQuery>) -> Response {
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
        Ok(Err(e)) => bad_request(e.to_string()),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

#[derive(Deserialize)]
struct PlaceQuery {
    osm_id: String,
}

async fn place(Query(query): Query<PlaceQuery>) -> Response {
    crate::nominatim::wait_turn().await;
    let osm_id = query.osm_id.clone();
    let found =
        tokio::task::spawn_blocking(move || place_area(&HttpFetcher::new(), &query.osm_id)).await;
    match found {
        Ok(Ok(area)) => {
            Json(json!({ "area": area.to_geojson_value(), "source_id": osm_id })).into_response()
        }
        Ok(Err(e)) => bad_request(e.to_string()),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

async fn save(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Response {
    let area = match area_of(&body) {
        Ok(area) => area,
        Err(e) => return bad_request(e),
    };
    let saved = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let dir = &state.dir;
        let created = !dir.join("freemapper.toml").exists();
        if created {
            let kind = match body["kind"].as_str() {
                Some(kind) => parse_kind(kind)?,
                None => state.defaults.kind,
            };
            let scale = body["scale"]
                .as_u64()
                .map(|s| u32::try_from(s).map_err(|_| format!("scale {s} is too large")))
                .transpose()?
                .or(state.defaults.scale);
            let title = body["title"]
                .as_str()
                .map(str::to_string)
                .or(state.defaults.title.clone());
            let today = chrono::Local::now().date_naive();
            init_project(dir, &area, kind, scale, today, title).map_err(|e| e.to_string())?;
        } else {
            set_map_area(dir, &area).map_err(|e| e.to_string())?;
        }
        Ok(json!({
            "saved": true,
            "created": created,
            "dir": dir,
            "has_map": map_path(dir).exists(),
        }))
    })
    .await;
    match saved {
        Ok(Ok(answer)) => Json(answer).into_response(),
        Ok(Err(e)) => bad_request(e),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

async fn done(State(state): State<Arc<AppState>>) -> Response {
    state.done.notify_one();
    Json(json!({ "done": true })).into_response()
}

/// Opens `url` in the default browser; an error when there is none (e.g. on a server).
pub fn open_browser(url: &str) -> std::io::Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else if cfg!(windows) {
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    } else {
        std::process::Command::new("xdg-open")
    };
    let status = command
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "browser start failed: {status}"
        )))
    }
}
