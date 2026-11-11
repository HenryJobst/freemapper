//! The area selection page: a local web server for one project folder. The user draws or picks the
//! map area in the browser; saving writes it to `freemapper.toml`.
//!
//! The server listens on 127.0.0.1 only. Every path starts with a random token, and requests for
//! another host name are refused, so other websites cannot reach it (also not by DNS rebinding).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::jobs::JobHub;
use crate::pipeline::RunOptions;
use crate::select_run;
use crate::select_run::{FolderOpener, PipelineRunner, Services};
use crate::tiles::{TileError, TileProxy};
use axum::Json;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Path as UrlPath, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
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
pub use freemapper_web::open_browser;
use freemapper_web::{LocalServer, ServeOptions, embedded};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Notify;

#[derive(RustEmbed)]
#[folder = "web/"]
pub(crate) struct Assets;

/// Hosts can only be reached from this machine; the page tells the user so.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
     style-src 'self' 'unsafe-inline'; connect-src 'self'; \
     img-src 'self' data: https://isk.geobasis-bb.de https://geodienste.sachsen.de https://www.geodatenportal.sachsen-anhalt.de https://geodienste.hamburg.de https://www.gds-srv.hessen.de https://dienste.gdi-sh.de https://www.geodaten-mv.de https://www.wms.nrw.de https://opendata.lgln.niedersachsen.de https://geo4.service24.rlp.de; \
     frame-ancestors 'none'";

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
            ..ServeOptions::new(CONTENT_SECURITY_POLICY)
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

fn kind_name(kind: MapKind) -> &'static str {
    kind.name()
}

fn parse_kind(kind: &str) -> Result<MapKind, String> {
    MapKind::from_name(kind)
        .ok_or_else(|| format!("unknown map kind {kind:?}; use schoolyard or sprint"))
}

/// The saved project, if there is one; an error when its file cannot be read.
pub(crate) fn saved_project(state: &AppState) -> Result<Option<ProjectFile>, String> {
    if !state.dir.join("freemapper.toml").exists() {
        return Ok(None);
    }
    ProjectFile::load(&state.dir)
        .map(Some)
        .map_err(|e| crate::pipeline::describe(&e))
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

async fn project(State(state): State<Arc<AppState>>) -> Response {
    let saved = match saved_project(&state) {
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
    answer["run_defaults"] = run_defaults_json(&state.defaults.run);
    answer["allowed_scales"] = MapKind::ALL
        .into_iter()
        .map(|kind| (kind.name().to_string(), json!(kind.allowed_scales())))
        .collect::<serde_json::Map<_, _>>()
        .into();
    // The program language when `--lang` or the environment named one; else null, and the page
    // follows the browser (ADR 0092).
    answer["lang"] = crate::text::named_lang().map_or(Value::Null, |lang| Value::from(lang.code()));
    answer["running"] = crate::select_run::running_json(&state);
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

/// The map area in a request body: `area` (GeoJSON) and optionally `source_id` (`way/…`).
fn area_of(body: &Value) -> Result<MapArea, String> {
    let mut area = MapArea::from_geojson(&body["area"].to_string())
        .map_err(|e| crate::pipeline::describe(&e))?;
    area.validate().map_err(|e| crate::pipeline::describe(&e))?;
    if let Some(source_id) = body["source_id"].as_str() {
        lookup_url(source_id).map_err(|e| crate::pipeline::describe(&e))?;
        area.source_id = Some(source_id.to_string());
    }
    Ok(area)
}

/// The download estimate as the page and the MCP tool show it: each download with its size,
/// whether it is cached and whether it fits on the disk, and under `disk` the free space of the
/// volumes of the cache and of the project folder `dir` with what the fetch needs there, the
/// messages for what does not fit and whether the fetch stops; `error` when the free space
/// cannot be read.
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
            // A part the run needs does not fit: the fetch stops before it downloads anything.
            "stops": plan.missing().is_some(),
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
        Ok(Err(e)) => bad_request(crate::pipeline::describe(&e)),
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
        tokio::task::spawn_blocking(move || place_area(&HttpFetcher::new(), &query.osm_id, None))
            .await;
    match found {
        Ok(Ok(area)) => {
            Json(json!({ "area": area.to_geojson_value(), "source_id": osm_id })).into_response()
        }
        Ok(Err(e)) => bad_request(crate::pipeline::describe(&e)),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

async fn save(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Response {
    if state.hub.running_for(&state.dir) {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "a pipeline is running for this project; save again when it has finished" })),
        )
            .into_response();
    }
    let _saving = state.saving.lock().await;
    let task_state = Arc::clone(&state);
    let saved = tokio::task::spawn_blocking(move || -> Result<Value, SaveError> {
        let state = task_state;
        // A folder that exists may be in use by a terminal or the MCP server; a new one cannot be.
        let _lock = if state.dir.is_dir() {
            Some(
                crate::lock::acquire(&state.dir, crate::lock::Who::Page)
                    .map_err(|e| SaveError::Locked(e.to_string()))?,
            )
        } else {
            None
        };
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
            init_project(dir, &area, kind, scale, today, title)
                .map_err(|e| crate::pipeline::describe(&e))?;
        } else {
            let before = ProjectFile::load(dir)
                .map_err(|e| crate::pipeline::describe(&e))?
                .crs();
            let after = set_map_area(dir, &area)
                .map_err(|e| crate::pipeline::describe(&e))?
                .crs();
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
        Ok(Err(SaveError::Locked(e))) => {
            (StatusCode::CONFLICT, Json(json!({ "error": e }))).into_response()
        }
        Ok(Err(SaveError::Other(e))) => bad_request(e),
        Err(e) => bad_request(format!("internal error: {e}")),
    }
}

/// Why a save did not happen.
enum SaveError {
    /// Another freemapper run holds the folder.
    Locked(String),
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
            in_place: Vec::new(),
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
        // Leaving out the surface model lets the fetch go on.
        assert_eq!(disk["stops"], false);
        // Without room for the OSM extract it stops.
        let full = Cache::new(PathBuf::from("/ssd/cache")).with_disk_space(Arc::new(TwoDisks {
            cache: PathBuf::from("/ssd"),
            cache_mb: 1024 + 100,
            project_mb: 5000,
        }));
        let json = estimate_json(&estimate, &full, Path::new("/usb/school"));
        assert_eq!(json["disk"]["stops"], true);
        assert!(
            json["disk"]["messages"][0]
                .as_str()
                .unwrap()
                .starts_with("not enough disk space for the ALKIS and OSM map data."),
            "{json}"
        );
    }

    #[test]
    fn estimate_says_when_free_space_cannot_be_read() {
        #[derive(Debug)]
        struct Unreadable;
        impl DiskSpace for Unreadable {
            fn volume(&self, _path: &Path) -> std::io::Result<Volume> {
                Err(std::io::Error::other("not supported"))
            }
        }
        let estimate = DownloadEstimate {
            tiles: Vec::new(),
            items: Vec::new(),
            total_megabytes: 0.0,
            in_place: Vec::new(),
        };
        let cache = Cache::new(PathBuf::from("/net/cache")).with_disk_space(Arc::new(Unreadable));
        let json = estimate_json(&estimate, &cache, Path::new("/net/school"));
        assert_eq!(
            json["disk"]["error"],
            "cannot read the free disk space of /net/cache (not supported)"
        );
    }

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
        let (status, text) = tokio::task::spawn_blocking(move || {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let area = r#"{"area":{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.539]]]}}"#;
            let mut response = agent
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Origin", &origin)
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
        let post = move |url: String, origin: String| {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            let body = r#"{"area":{"type":"Polygon","coordinates":[[[13.843,52.539],[13.846,52.539],[13.846,52.541],[13.843,52.539]]]}}"#;
            let mut response = agent
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Origin", &origin)
                .send(body)
                .unwrap();
            (
                response.status().as_u16(),
                response.body_mut().read_to_string().unwrap(),
            )
        };
        let (u, o) = (url.clone(), origin.clone());
        let (status, text) = tokio::task::spawn_blocking(move || post(u, o))
            .await
            .unwrap();
        assert_eq!(status, 409, "{text}");
        assert!(
            text.contains("another freemapper run is using this folder"),
            "{text}"
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
        let (status, text) = tokio::task::spawn_blocking(move || post(url, origin))
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
        // No language was named in this test process, so the page follows the browser.
        assert!(answer["lang"].is_null(), "{answer}");
        assert!(answer.as_object().unwrap().contains_key("lang"));
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
