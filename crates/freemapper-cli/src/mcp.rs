//! `freemapper mcp`: the MCP server on stdio, so an AI agent can make a map for a non-technical
//! user. The tools are thin wrappers over the library; the pipeline runs as a job the agent polls.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use base64::Engine;
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, FetchMode, HttpFetcher};
use freemapper::layers::{LayerRole, estimate_download, list_layers, preview_png};
use freemapper::places::{find_places, place_area};
use freemapper::project::{
    MapKind, MapOutcome, ProjectFile, RunOutcome, init_project, run_project,
};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::select::{Busy, Defaults, Selector, open_browser};
use serde_json::{Value, json};

/// Progress lines kept per job.
const JOB_LINES: usize = 50;
/// Longest side of a preview image in pixels.
const PREVIEW_PX: u32 = 800;

const INSTRUCTIONS: &str = "freemapper makes OpenOrienteering Mapper starter projects for \
orienteering maps (schoolyard 1:1000, sprint 1:4000) from official Brandenburg geodata and OSM. \
Workflow: find_place (search the school or area by name; pick a result with has_area) → \
create_project (a new folder, absolute path, with the osm_id) → estimate_download (tell the user \
how much will be downloaded) → start_pipeline (returns a job id; downloading can take minutes) → \
project_status (poll every few seconds until the state is finished or failed) → list_layers and \
preview_layer (show the user the templates). The user then opens the map file in OpenOrienteering \
Mapper. When no place fits or the user wants to draw the area, open_area_selector opens a page \
in the browser where the user draws or picks the area and saves it (this creates the project); \
continue with estimate_download once the user says it is saved. Paths must be absolute or start \
with ~. One pipeline runs at a time.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JobState {
    Running,
    Finished,
    Failed,
}

impl JobState {
    fn name(self) -> &'static str {
        match self {
            JobState::Running => "running",
            JobState::Finished => "finished",
            JobState::Failed => "failed",
        }
    }
}

#[derive(Debug)]
struct Job {
    dir: PathBuf,
    state: JobState,
    lines: VecDeque<String>,
    warnings: Vec<String>,
    result: Option<String>,
    error: Option<String>,
}

/// The pipeline jobs of this server's lifetime.
#[derive(Debug, Default)]
struct Jobs {
    next: u64,
    jobs: HashMap<String, Job>,
}

impl Jobs {
    /// Registers a running job for `dir`; refused while any job runs, because all projects
    /// share the download cache.
    fn start(&mut self, dir: &Path) -> Result<String, String> {
        if let Some((id, job)) = self
            .jobs
            .iter()
            .find(|(_, job)| job.state == JobState::Running)
        {
            return Err(format!(
                "{id} is still running for {}; one pipeline runs at a time (projects share the \
                 download cache): start this one when project_status reports finished or failed",
                job.dir.display()
            ));
        }
        self.next += 1;
        let id = format!("job-{}", self.next);
        self.jobs.insert(
            id.clone(),
            Job {
                dir: dir.to_path_buf(),
                state: JobState::Running,
                lines: VecDeque::new(),
                warnings: Vec::new(),
                result: None,
                error: None,
            },
        );
        Ok(id)
    }

    /// Whether a job runs for the folder `key` (see [`folder_key`]).
    fn running_for(&self, key: &Path) -> bool {
        self.jobs
            .values()
            .any(|job| job.state == JobState::Running && folder_key(&job.dir) == key)
    }

    fn progress(&mut self, id: &str, line: &str) {
        let Some(job) = self.jobs.get_mut(id) else {
            return;
        };
        if line.starts_with("warning:") {
            job.warnings.push(line.to_string());
        }
        if job.lines.len() == JOB_LINES {
            job.lines.pop_front();
        }
        job.lines.push_back(line.to_string());
    }

    fn finish(&mut self, id: &str, outcome: Result<String, String>) {
        let Some(job) = self.jobs.get_mut(id) else {
            return;
        };
        match outcome {
            Ok(result) => {
                job.state = JobState::Finished;
                job.result = Some(result);
            }
            Err(error) => {
                job.state = JobState::Failed;
                job.error = Some(error);
            }
        }
    }

    fn status(&self, id: &str) -> Option<Value> {
        let job = self.jobs.get(id)?;
        Some(json!({
            "job_id": id,
            "dir": job.dir,
            "state": job.state.name(),
            "lines": job.lines,
            "warnings": job.warnings,
            "result": job.result,
            "error": job.error,
        }))
    }
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct FindPlace {
    /// Name of the school or place, e.g. "Clara-Zetkin-Schule Strausberg".
    query: String,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct CreateProject {
    /// New project folder (absolute path or starting with ~).
    dir: String,
    /// OSM object of the map area from find_place, e.g. "way/992423119".
    osm_id: Option<String>,
    /// Instead of osm_id: a GeoJSON file with the map area polygon (EPSG:4326).
    geojson: Option<String>,
    /// "schoolyard" (default, 1:1000) or "sprint" (1:4000).
    kind: Option<String>,
    /// Scale denominator, e.g. 1000 for 1:1000 (default depends on the kind).
    scale: Option<u32>,
    /// Map title for the peripheral information (default: the folder name).
    title: Option<String>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct ProjectDir {
    /// Project folder (absolute path or starting with ~).
    dir: String,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct StartPipeline {
    /// Project folder (absolute path or starting with ~).
    dir: String,
    /// Replace an existing map file (the old one is kept as .omap.bak). Default false.
    force: Option<bool>,
    /// Use the download cache only, without network access. Default false.
    offline: Option<bool>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct ProjectStatus {
    /// Job id from start_pipeline, e.g. "job-1".
    job_id: String,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct OpenAreaSelector {
    /// Project folder (absolute path or starting with ~); created when the user saves.
    dir: String,
    /// Map kind of a new project: "schoolyard" (default, 1:1000) or "sprint" (1:4000).
    kind: Option<String>,
    /// Scale denominator of a new project.
    scale: Option<u32>,
    /// Map title of a new project.
    title: Option<String>,
    /// Also open the page in the default browser of this computer. Default true.
    open_browser: Option<bool>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct PreviewLayer {
    /// Project folder (absolute path or starting with ~).
    dir: String,
    /// "orthophoto", "hillshade", "slope" or "ndsm".
    layer: String,
}

#[derive(Clone)]
pub struct FreemapperServer {
    tool_router: ToolRouter<Self>,
    cache: Cache,
    jobs: Arc<Mutex<Jobs>>,
    /// Area selection pages by [`folder_key`], one per folder.
    selectors: Arc<tokio::sync::Mutex<HashMap<PathBuf, Selector>>>,
}

/// The job table; a panic elsewhere never makes it unusable.
fn lock(jobs: &Mutex<Jobs>) -> MutexGuard<'_, Jobs> {
    jobs.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Runs a job to its end and records the outcome; a panic is recorded as a failure, so the
/// job never stays running.
fn run_job(
    jobs: &Mutex<Jobs>,
    id: &str,
    job: impl FnOnce(&mut dyn FnMut(&str)) -> Result<String, String>,
) {
    let mut progress = |line: &str| lock(jobs).progress(id, line);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(&mut progress)))
        .unwrap_or_else(|panic| {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            Err(format!("internal error: {message}"))
        });
    lock(jobs).finish(id, outcome);
}

/// A tool result with one text block.
fn text(text: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// A tool error the agent can relay to the user.
fn failure(message: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![ContentBlock::text(message)]))
}

/// An error with its causes, e.g. "fetch failed: tile 33421-5821 is not in the cache".
fn describe(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let line = cause.to_string();
        if !message.contains(&line) {
            message = format!("{message}: {line}");
        }
        source = cause.source();
    }
    message
}

/// Resolves a path from the agent: absolute, or `~/…` (`~\…`) inside the home folder.
/// Relative paths are refused because the server's working folder is unknown to the agent; the
/// home folder itself is refused because a project would litter it.
fn resolve(path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    let home = dirs::home_dir();
    let resolved = if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
        let home = home
            .clone()
            .ok_or("no home folder on this system; use an absolute path")?;
        home.join(path[1..].trim_start_matches(['/', '\\']))
    } else {
        PathBuf::from(path)
    };
    if !resolved.is_absolute() {
        return Err(format!(
            "{path:?} is a relative path; use an absolute path or one starting with ~"
        ));
    }
    if home.is_some_and(|home| home == resolved) {
        return Err(format!(
            "{path:?} is the home folder itself; use a folder inside it, e.g. ~/maps/school"
        ));
    }
    Ok(resolved)
}

/// One spelling per folder (`..`, symlinks and letter case resolved as far as the folder or its
/// nearest existing parent exists), to find the page or job of a folder.
fn folder_key(dir: &Path) -> PathBuf {
    for ancestor in dir.ancestors() {
        if let Ok(canonical) = std::fs::canonicalize(ancestor) {
            let rest = dir.strip_prefix(ancestor).unwrap_or(Path::new(""));
            return canonical.join(rest);
        }
    }
    dir.to_path_buf()
}

/// The folder of an existing project.
fn project_dir(path: &str) -> Result<PathBuf, String> {
    let dir = resolve(path)?;
    if !dir.join("freemapper.toml").is_file() {
        return Err(format!(
            "no freemapper project in {}; use create_project first",
            dir.display()
        ));
    }
    Ok(dir)
}

fn map_kind(kind: Option<&str>) -> Result<MapKind, String> {
    match kind.unwrap_or("schoolyard") {
        "schoolyard" => Ok(MapKind::Schoolyard),
        "sprint" => Ok(MapKind::Sprint),
        other => Err(format!(
            "unknown kind {other:?}; use \"schoolyard\" or \"sprint\""
        )),
    }
}

fn role_name(role: LayerRole) -> &'static str {
    match role {
        LayerRole::Map => "map",
        LayerRole::Candidates => "candidates",
        LayerRole::Template => "template",
    }
}

/// What the pipeline did, in words for the user.
fn summary(outcome: &RunOutcome) -> String {
    let mut lines = Vec::new();
    match &outcome.map {
        MapOutcome::Written(map) => lines.push(format!("Wrote the map {}", map.display())),
        MapOutcome::Kept {
            map,
            references_images,
            missing_buildings,
        } => {
            lines.push(format!(
                "Kept the existing map {} (it may contain the mapper's work)",
                map.display()
            ));
            if !references_images {
                lines.push(
                    "The map does not reference the image templates yet: start the pipeline with \
                     force to replace it (the old one is kept as .omap.bak)."
                        .to_string(),
                );
            }
            if *missing_buildings {
                lines.push(
                    "The map has no ALKIS buildings yet: start the pipeline with force to add them \
                     (the old one is kept as .omap.bak)."
                        .to_string(),
                );
            }
        }
    }
    for template in &outcome.templates {
        lines.push(format!("Wrote {}", template.display()));
    }
    lines.join("\n")
}

/// Runs blocking library code off the async runtime.
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| format!("internal error: {e}"))?
}

#[tool_router]
impl FreemapperServer {
    pub fn new(cache: Cache) -> Self {
        Self {
            tool_router: Self::tool_router(),
            cache,
            jobs: Arc::default(),
            selectors: Arc::default(),
        }
    }

    #[tool(
        description = "Search a school, park or other place by name (OpenStreetMap Nominatim). \
                       Returns up to 5 matches, areas first; only matches with has_area can be a \
                       map area. An empty list means nothing was found."
    )]
    async fn find_place(
        &self,
        Parameters(req): Parameters<FindPlace>,
    ) -> Result<CallToolResult, McpError> {
        crate::nominatim::wait_turn().await;
        let places = blocking(move || {
            find_places(&HttpFetcher::new(), &req.query).map_err(|e| describe(&e))
        })
        .await;
        match places {
            Ok(places) => {
                let places: Vec<Value> = places
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
                    .collect();
                text(Value::from(places).to_string())
            }
            Err(e) => failure(e),
        }
    }

    #[tool(
        description = "Create a project folder for a map area: give osm_id (from find_place) or a \
                       GeoJSON file. kind is schoolyard (1:1000, default) or sprint (1:4000)."
    )]
    async fn create_project(
        &self,
        Parameters(req): Parameters<CreateProject>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match resolve(&req.dir) {
            Ok(dir) => dir,
            Err(e) => return failure(e),
        };
        let kind = match map_kind(req.kind.as_deref()) {
            Ok(kind) => kind,
            Err(e) => return failure(e),
        };
        let area = match (req.osm_id, req.geojson) {
            (Some(osm_id), None) => {
                crate::nominatim::wait_turn().await;
                blocking(move || place_area(&HttpFetcher::new(), &osm_id).map_err(|e| describe(&e)))
                    .await
            }
            (None, Some(geojson)) => resolve(&geojson).and_then(|path| {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("reading {}: {e}", path.display()))?;
                MapArea::from_geojson(&text).map_err(|e| describe(&e))
            }),
            _ => Err("give exactly one of osm_id (from find_place) or geojson".to_string()),
        };
        let area = match area {
            Ok(area) => area,
            Err(e) => return failure(e),
        };
        let today = chrono::Local::now().date_naive();
        match init_project(&dir, &area, kind, req.scale, today, req.title) {
            Ok(project) => text(format!(
                "Created project {} for {} ({}, 1:{}). Next: estimate_download, then start_pipeline.",
                dir.display(),
                area.source_id.as_deref().unwrap_or("the given area"),
                match kind {
                    MapKind::Schoolyard => "schoolyard",
                    MapKind::Sprint => "sprint",
                },
                project.scale_denominator
            )),
            Err(e) => failure(describe(&e)),
        }
    }

    #[tool(
        description = "What start_pipeline will download for a project: the tiles, each download \
                       with its size in MB and whether it is cached, and the MB still to download."
    )]
    async fn estimate_download(
        &self,
        Parameters(req): Parameters<ProjectDir>,
    ) -> Result<CallToolResult, McpError> {
        let cache = self.cache.clone();
        let estimate = blocking(move || {
            let dir = project_dir(&req.dir)?;
            let today = chrono::Local::now().date_naive();
            estimate_download(&dir, &cache, today).map_err(|e| describe(&e))
        })
        .await;
        match estimate {
            Ok(estimate) => {
                let items: Vec<Value> = estimate
                    .items
                    .iter()
                    .map(
                        |i| json!({ "what": i.what, "megabytes": i.megabytes, "cached": i.cached }),
                    )
                    .collect();
                text(
                    json!({
                        "tiles": estimate.tiles,
                        "items": items,
                        "total_megabytes": estimate.total_megabytes,
                    })
                    .to_string(),
                )
            }
            Err(e) => failure(e),
        }
    }

    #[tool(
        description = "Download the data, compute the templates and write the starter project \
                       (fetch, derive, export). Runs in the background and returns a job_id; poll \
                       project_status. One pipeline runs at a time. An existing map is kept unless force \
                       is true."
    )]
    async fn start_pipeline(
        &self,
        Parameters(req): Parameters<StartPipeline>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match project_dir(&req.dir) {
            Ok(dir) => dir,
            Err(e) => return failure(e),
        };
        if let Err(e) = ProjectFile::load(&dir) {
            return failure(describe(&e));
        }
        let page_open = self
            .selectors
            .lock()
            .await
            .get(&folder_key(&dir))
            .is_some_and(Selector::is_running);
        if page_open {
            return failure(
                "the area selection page for this project is still open; ask the user to save \
                 the area and press Done on the page, then start the pipeline",
            );
        }
        let id = match lock(&self.jobs).start(&dir) {
            Ok(id) => id,
            Err(e) => return failure(e),
        };
        let jobs = Arc::clone(&self.jobs);
        let cache = self.cache.clone();
        let mode = if req.offline.unwrap_or(false) {
            FetchMode::Offline
        } else {
            FetchMode::Online
        };
        let force = req.force.unwrap_or(false);
        let job_id = id.clone();
        let job_dir = dir.clone();
        tokio::task::spawn_blocking(move || {
            run_job(&jobs, &job_id, |progress| {
                run_project(&job_dir, &HttpFetcher::new(), &cache, mode, force, progress)
                    .map(|outcome| summary(&outcome))
                    .map_err(|e| describe(&e))
            });
        });
        text(json!({ "job_id": id, "dir": dir, "state": "running" }).to_string())
    }

    #[tool(
        description = "State of a pipeline job: running, finished or failed, the last progress \
                       lines, warnings, and the result or the error."
    )]
    async fn project_status(
        &self,
        Parameters(req): Parameters<ProjectStatus>,
    ) -> Result<CallToolResult, McpError> {
        match lock(&self.jobs).status(&req.job_id) {
            Some(status) => text(status.to_string()),
            None => failure(format!(
                "no job {:?}; start_pipeline returns the job ids of this session",
                req.job_id
            )),
        }
    }

    #[tool(
        description = "The layers of a project: the map, the candidates and the raster templates, \
                       each with its file, whether it is made yet and which step makes it."
    )]
    async fn list_layers(
        &self,
        Parameters(req): Parameters<ProjectDir>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match project_dir(&req.dir) {
            Ok(dir) => dir,
            Err(e) => return failure(e),
        };
        let layers: Vec<Value> = list_layers(&dir)
            .iter()
            .map(|l| {
                json!({
                    "name": l.name,
                    "file": l.file,
                    "role": role_name(l.role),
                    "ready": l.ready,
                    "made_by": l.made_by,
                    "description": l.description,
                })
            })
            .collect();
        text(Value::from(layers).to_string())
    }

    #[tool(
        description = "Open a page in the browser where the user draws the map area (rectangle \
                       or polygon) or picks a place, sees the tiles and the download size, and \
                       saves it; saving creates the project folder or changes its area. Returns \
                       the URL to give the user. The page runs until the user presses Done; \
                       calling again for the same folder returns the same URL."
    )]
    async fn open_area_selector(
        &self,
        Parameters(req): Parameters<OpenAreaSelector>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match resolve(&req.dir) {
            Ok(dir) => dir,
            Err(e) => return failure(e),
        };
        let kind = match map_kind(req.kind.as_deref()) {
            Ok(kind) => kind,
            Err(e) => return failure(e),
        };
        let key = folder_key(&dir);
        let mut selectors = self.selectors.lock().await;
        selectors.retain(|_, selector| selector.is_running());
        let (url, note) = match selectors.get(&key) {
            Some(selector) => (
                selector.url.clone(),
                Some(
                    "The page for this folder is already open; it keeps the settings it was \
                     opened with.",
                ),
            ),
            None => {
                let defaults = Defaults {
                    kind,
                    scale: req.scale,
                    title: req.title,
                    from_agent: true,
                };
                let jobs = Arc::clone(&self.jobs);
                let busy: Busy =
                    Arc::new(move |dir: &Path| lock(&jobs).running_for(&folder_key(dir)));
                let started =
                    Selector::start(dir.clone(), defaults, self.cache.clone(), 0, Some(busy)).await;
                let selector = match started {
                    Ok(selector) => selector,
                    Err(e) => return failure(format!("cannot start the page: {e}")),
                };
                let url = selector.url.clone();
                selectors.insert(key, selector);
                (url, None)
            }
        };
        drop(selectors);
        let opened = req.open_browser.unwrap_or(true) && open_browser(&url).is_ok();
        text(
            json!({
                "url": url,
                "dir": dir,
                "opened_browser": opened,
                "note": note,
                "next": "Ask the user to choose the area on the page, press Save and then Done.",
            })
            .to_string(),
        )
    }

    #[tool(
        description = "A PNG preview (at most 800 px) of a raster template: orthophoto, \
                       hillshade, slope or ndsm."
    )]
    async fn preview_layer(
        &self,
        Parameters(req): Parameters<PreviewLayer>,
    ) -> Result<CallToolResult, McpError> {
        let layer = req.layer.clone();
        let png = blocking(move || {
            let dir = project_dir(&req.dir)?;
            preview_png(&dir, &req.layer, PREVIEW_PX).map_err(|e| describe(&e))
        })
        .await;
        match png {
            Ok(png) => Ok(CallToolResult::success(vec![
                ContentBlock::text(format!("Preview of {layer}")),
                ContentBlock::image(
                    base64::engine::general_purpose::STANDARD.encode(png),
                    "image/png",
                ),
            ])),
            Err(e) => failure(e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FreemapperServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("freemapper", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

/// Serves MCP on stdin/stdout until the client disconnects.
pub async fn serve(cache: Cache) -> anyhow::Result<()> {
    let service = FreemapperServer::new(cache)
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_pipeline_runs_at_a_time() {
        // Jobs share the download cache, so a second one waits even for another project.
        let mut jobs = Jobs::default();
        let first = jobs.start(Path::new("/projects/school")).unwrap();
        assert_eq!(first, "job-1");
        for dir in ["/projects/school", "/projects/other"] {
            let refused = jobs.start(Path::new(dir)).unwrap_err();
            assert!(refused.contains("job-1"), "{refused}");
        }
        jobs.finish(&first, Err("offline".into()));
        assert_eq!(jobs.start(Path::new("/projects/other")).unwrap(), "job-2");
    }

    #[test]
    fn panicking_job_is_reported_as_failed() {
        let jobs = Arc::new(Mutex::new(Jobs::default()));
        let id = lock(&jobs).start(Path::new("/p")).unwrap();
        run_job(&jobs, &id, |_| panic!("bad tile data"));
        let status = lock(&jobs).status(&id).unwrap();
        assert_eq!(status["state"], "failed");
        assert!(
            status["error"].as_str().unwrap().contains("bad tile data"),
            "{status}"
        );
    }

    #[test]
    fn job_progress_and_outcome_are_recorded() {
        let jobs = Arc::new(Mutex::new(Jobs::default()));
        let id = lock(&jobs).start(Path::new("/p")).unwrap();
        run_job(&jobs, &id, |progress| {
            progress("warning: no nDSM for tile 33421-5821");
            Ok("Wrote the map".into())
        });
        let status = lock(&jobs).status(&id).unwrap();
        assert_eq!(status["state"], "finished");
        assert_eq!(status["warnings"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn project_dir_needs_a_project() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("school");
        let err = project_dir(project.to_str().unwrap()).unwrap_err();
        assert!(err.contains("create_project"), "{err}");
        std::fs::create_dir(&project).unwrap();
        std::fs::write(project.join("freemapper.toml"), "").unwrap();
        assert_eq!(project_dir(project.to_str().unwrap()).unwrap(), project);
    }

    #[test]
    fn job_keeps_the_last_lines_and_the_warnings() {
        let mut jobs = Jobs::default();
        let id = jobs.start(Path::new("/p")).unwrap();
        jobs.progress(&id, "warning: no nDSM for tile 33421-5821");
        for i in 0..60 {
            jobs.progress(&id, &format!("line {i}"));
        }
        let status = jobs.status(&id).unwrap();
        assert_eq!(status["state"], "running");
        assert_eq!(status["lines"].as_array().unwrap().len(), JOB_LINES);
        assert_eq!(status["lines"][0], "line 10");
        assert_eq!(status["warnings"].as_array().unwrap().len(), 1);
        jobs.finish(&id, Ok("Wrote the map".into()));
        let status = jobs.status(&id).unwrap();
        assert_eq!(status["state"], "finished");
        assert_eq!(status["result"], "Wrote the map");
    }

    #[test]
    fn paths_must_be_absolute_or_home() {
        assert!(resolve("projects/school").unwrap_err().contains("relative"));
        let home = dirs::home_dir().unwrap();
        assert_eq!(resolve("~/maps/x").unwrap(), home.join("maps/x"));
        assert_eq!(resolve("~\\maps").unwrap(), home.join("maps"));
        assert!(resolve("~").unwrap_err().contains("home folder itself"));
        let absolute = std::env::temp_dir().join("x");
        assert_eq!(resolve(absolute.to_str().unwrap()).unwrap(), absolute);
    }

    #[test]
    fn describe_adds_causes_once() {
        #[derive(Debug)]
        struct Outer(std::io::Error);
        impl std::fmt::Display for Outer {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("fetch failed")
            }
        }
        impl std::error::Error for Outer {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
        let error = Outer(std::io::Error::other("tile 33421-5821 is not in the cache"));
        assert_eq!(
            describe(&error),
            "fetch failed: tile 33421-5821 is not in the cache"
        );
    }
}
