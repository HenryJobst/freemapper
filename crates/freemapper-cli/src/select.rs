//! The area selection page: a local web server for one project folder. The user draws or picks the
//! map area in the browser; saving writes it to `freemapper.toml`.
//!
//! The server listens on 127.0.0.1 only. Every path starts with a random token, and requests for
//! another host name are refused, so other websites cannot reach it (also not by DNS rebinding).

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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
use freemapper::layers::{AreaPreview, DownloadEstimate, preview_area};
use freemapper::places::{find_places, lookup_url, place_area};
use freemapper::project::{
    MAX_AREA_SIDE_M, MapKind, ProjectFile, init_project, map_path, set_map_area,
};
use freemapper::space::RESERVE_BYTES;
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
     img-src 'self' data: https://tile.openstreetmap.org https://isk.geobasis-bb.de https://geodienste.sachsen.de https://www.geodatenportal.sachsen-anhalt.de https://geodienste.hamburg.de https://www.gds-srv.hessen.de https://dienste.gdi-sh.de https://www.geodaten-mv.de https://www.wms.nrw.de https://opendata.lgln.niedersachsen.de https://geo4.service24.rlp.de; \
     frame-ancestors 'none'";

/// Settings for a project the page creates (an existing project keeps its own).
#[derive(Debug, Clone)]
pub struct Defaults {
    pub kind: MapKind,
    pub scale: Option<u32>,
    pub title: Option<String>,
    /// The page was opened by an AI agent: after "done" the user goes back to the agent.
    pub from_agent: bool,
}

/// Whether a pipeline is running for a folder; saving is refused then, because the pipeline
/// would write its own copy of the project file back.
pub type Busy = Arc<dyn Fn(&Path) -> bool + Send + Sync>;

struct AppState {
    dir: PathBuf,
    defaults: Defaults,
    cache: Cache,
    allowed_hosts: [String; 2],
    allowed_origins: [String; 2],
    busy: Option<Busy>,
    /// Saves one after the other.
    saving: tokio::sync::Mutex<()>,
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
        busy: Option<Busy>,
    ) -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
            .await
            .map_err(|e| {
                if port == 0 {
                    anyhow::anyhow!("cannot listen on 127.0.0.1: {e}")
                } else {
                    anyhow::anyhow!(
                        "cannot listen on 127.0.0.1:{port} ({e}); leave out --port to pick a free port"
                    )
                }
            })?;
        let port = listener.local_addr()?.port();
        let token = token()?;
        let hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
        let state = Arc::new(AppState {
            dir,
            defaults,
            cache,
            allowed_origins: hosts.clone().map(|host| format!("http://{host}")),
            allowed_hosts: hosts,
            busy,
            saving: tokio::sync::Mutex::new(()),
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

    /// Whether the page still serves (the user has not pressed "done").
    pub fn is_running(&self) -> bool {
        !self.server.is_finished()
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
                // Browsers send Origin on cross-site requests: only the page itself may call.
                if let Some(origin) = request.headers().get(header::ORIGIN)
                    && !state
                        .allowed_origins
                        .iter()
                        .any(|allowed| allowed == origin)
                {
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
                // The token is in the URL: never send it to the tile servers.
                headers.insert(
                    header::REFERRER_POLICY,
                    HeaderValue::from_static("no-referrer"),
                );
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

/// The saved project, if there is one; an error when its file cannot be read.
fn saved_project(state: &AppState) -> Result<Option<ProjectFile>, String> {
    if !state.dir.join("freemapper.toml").exists() {
        return Ok(None);
    }
    ProjectFile::load(&state.dir)
        .map(Some)
        .map_err(|e| e.to_string())
}

async fn project(State(state): State<Arc<AppState>>) -> Response {
    let saved = match saved_project(&state) {
        Ok(saved) => saved,
        Err(e) => return bad_request(e),
    };
    let answer = match saved {
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

/// The download estimate as the page and the MCP tool show it: each download with its size,
/// whether it is cached and whether it fits on the disk, and under `disk` the free space of the
/// volumes of the cache and of the project folder `dir` with what the fetch needs there and the
/// warnings for what does not fit.
pub(crate) fn estimate_json(estimate: &DownloadEstimate, cache: &Cache, dir: &Path) -> Value {
    let plan = freemapper::space::plan(estimate, cache.root(), dir, cache.disk_space());
    let items: Vec<Value> = estimate
        .items
        .iter()
        .map(|i| {
            let fits = i.cached || plan.as_ref().map_or(true, |plan| plan.fits(i.part));
            json!({ "what": i.what, "megabytes": i.megabytes, "cached": i.cached, "fits": fits })
        })
        .collect();
    let megabytes = |bytes: u64| bytes.div_ceil(1 << 20);
    let disk = match &plan {
        Ok(plan) => json!({
            "reserve_megabytes": megabytes(RESERVE_BYTES),
            "volumes": plan.volumes.iter().map(|v| {
                let holds: Vec<&str> = [(v.holds_cache, "cache"), (v.holds_project, "project")]
                    .into_iter()
                    .filter_map(|(holds, what)| holds.then_some(what))
                    .collect();
                json!({
                    "path": v.path,
                    "holds": holds,
                    "free_megabytes": v.available / (1 << 20),
                    "needed_megabytes": megabytes(v.needed),
                })
            }).collect::<Vec<_>>(),
            "messages": plan.short.iter().map(ToString::to_string).collect::<Vec<_>>(),
        }),
        Err((path, e)) => json!({
            "error": format!("cannot read the free disk space of {} ({e})", path.display()),
        }),
    };
    json!({
        "tiles": estimate.tiles,
        "items": items,
        "total_megabytes": estimate.total_megabytes,
        "disk": disk,
    })
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

async fn preview(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Response {
    let today = chrono::Local::now().date_naive();
    // Validating a large outline takes a while: off the async workers.
    let answer = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let area = area_of(&body)?;
        // An existing project keeps its buffer; a new one gets the buffer of the kind chosen
        // on the page.
        let buffer_m = match saved_project(&state)? {
            Some(project) => project.buffer_m,
            None => match body["kind"].as_str() {
                Some(kind) => parse_kind(kind)?.default_buffer_m(),
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
    if state.busy.as_ref().is_some_and(|busy| busy(&state.dir)) {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "a pipeline is running for this project; save again when it has finished" })),
        )
            .into_response();
    }
    let _saving = state.saving.lock().await;
    let task_state = Arc::clone(&state);
    let saved = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let state = task_state;
        let area = area_of(&body)?;
        let dir = &state.dir;
        let created = !dir.join("freemapper.toml").exists();
        let mut zone_changed = false;
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
            let before = ProjectFile::load(dir).map_err(|e| e.to_string())?.crs();
            let after = set_map_area(dir, &area).map_err(|e| e.to_string())?.crs();
            zone_changed = before != after;
        }
        Ok(json!({
            "saved": true,
            "created": created,
            // Everything fetched for the old area was in the other UTM zone and is dropped.
            "zone_changed": zone_changed,
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

/// Opens `url` in the default browser; an error when no opener can be started (e.g. on a server).
/// Does not wait for the opener: some keep running as long as the browser.
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
    let mut child = command
        .arg(url)
        // The MCP server's stdin is the protocol pipe.
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper::layers::EstimateItem;
    use freemapper::space::{DiskSpace, Part, Store, Volume};

    /// The cache on one volume with `cache_mb` free, everything else on another with
    /// `project_mb`.
    #[derive(Debug)]
    struct TwoDisks {
        cache: PathBuf,
        cache_mb: u64,
        project_mb: u64,
    }

    impl DiskSpace for TwoDisks {
        fn volume(&self, path: &Path) -> std::io::Result<Volume> {
            let (id, mb) = if path.starts_with(&self.cache) {
                ("ssd", self.cache_mb)
            } else {
                ("usb", self.project_mb)
            };
            Ok(Volume {
                id: id.to_string(),
                available: mb << 20,
            })
        }
    }

    #[test]
    fn estimate_shows_free_space() {
        let item = |what: &str, megabytes, cached, part, store| EstimateItem {
            what: what.to_string(),
            megabytes,
            cached,
            part,
            store,
        };
        let estimate = DownloadEstimate {
            tiles: Vec::new(),
            items: vec![
                item("OSM extract", 300.0, false, Part::MapData, Store::Cache),
                item("orthophoto", 20.0, false, Part::Orthophoto, Store::Project),
                item("surface model", 200.0, false, Part::Surface, Store::Cache),
                item("terrain model", 10.0, true, Part::Terrain, Store::Cache),
            ],
            total_megabytes: 520.0,
        };
        let cache = Cache::new(PathBuf::from("/ssd/cache")).with_disk_space(Arc::new(TwoDisks {
            cache: PathBuf::from("/ssd"),
            cache_mb: 1024 + 400,
            project_mb: 5000,
        }));
        let json = estimate_json(&estimate, &cache, Path::new("/usb/school"));
        let disk = &json["disk"];
        assert_eq!(disk["reserve_megabytes"], 1024);
        assert_eq!(
            disk["volumes"],
            json!([
                { "path": "/ssd/cache", "holds": ["cache"], "free_megabytes": 1424, "needed_megabytes": 500 },
                { "path": "/usb/school", "holds": ["project"], "free_megabytes": 5000, "needed_megabytes": 140 },
            ])
        );
        let fits: Vec<bool> = json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["fits"].as_bool().unwrap())
            .collect();
        assert_eq!(fits, [true, true, false, true]);
        assert_eq!(
            disk["messages"],
            json!([
                "freemapper skips the surface model for the nDSM template for lack of disk space. It needs 200 MB, but /ssd/cache has only 100 MB free beyond the 1.0 GB reserve. Free some space and run `freemapper fetch` again"
            ])
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn save_is_refused_while_a_pipeline_runs() {
        let dir = tempfile::tempdir().unwrap();
        let busy: Busy = Arc::new(|_| true);
        let defaults = Defaults {
            kind: MapKind::Schoolyard,
            scale: None,
            title: None,
            from_agent: true,
        };
        let cache = Cache::new(dir.path().join("cache"));
        let selector = Selector::start(dir.path().join("p"), defaults, cache, 0, Some(busy))
            .await
            .unwrap();
        let url = format!("{}api/save", selector.url);
        let (status, text) = tokio::task::spawn_blocking(move || {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let area = r#"{"area":{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.539]]]}}"#;
            let mut response = agent
                .post(&url)
                .header("Content-Type", "application/json")
                .send(area)
                .unwrap();
            let status = response.status().as_u16();
            (status, response.body_mut().read_to_string().unwrap())
        })
        .await
        .unwrap();
        assert_eq!(status, 409, "{text}");
        assert!(text.contains("pipeline"), "{text}");
        assert!(!dir.path().join("p").exists());
    }
}
