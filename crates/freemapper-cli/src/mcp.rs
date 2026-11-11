//! `freemapper mcp`: the MCP server on stdio, so an AI agent can make a map for a non-technical
//! user. The tools are thin wrappers over the library; the pipeline runs as a job the agent polls.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use freemapper::area::MapArea;
use freemapper::fetch::{Cache, HttpFetcher};
use freemapper::layers::{LayerRole, estimate_download, list_layers, preview_png};
use freemapper::places::{find_places, place_area};
use freemapper::project::{MapKind, ProjectFile, init_project};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;

use crate::jobs::{JobHub, folder_key};
use crate::pipeline::{self, describe};
use crate::select::{Defaults, Selector, estimate_json, open_browser};
use serde_json::{Value, json};

/// Longest side of a preview image in pixels.
const PREVIEW_PX: u32 = 800;

const INSTRUCTIONS: &str = "freemapper makes OpenOrienteering Mapper starter projects for \
orienteering maps (schoolyard 1:1000, sprint 1:4000) from official geodata of Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony and Rhineland-Palatinate, and from OSM. \
Workflow: find_place (search the school or area by name; pick a result with has_area) → \
create_project (a new folder, absolute path, with the osm_id) → estimate_download (tell the user \
how much will be downloaded) → start_pipeline (returns a job id; downloading can take minutes) → \
project_status (poll every few seconds until the state is finished or failed) → list_layers and \
preview_layer (show the user the templates). The user then opens the map file in OpenOrienteering \
Mapper. When no place fits or the user wants to draw the area, open_area_selector opens a page \
in the browser where the user draws or picks the area and saves it (this creates the project); \
continue with estimate_download once the user says it is saved. Paths must be absolute or start \
with ~. One pipeline runs at a time.";

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
    /// With osm_id: which part of an area of several separate parts to use (1 is the largest).
    part: Option<usize>,
    /// "schoolyard" (default, 1:1000) or "sprint" (1:4000).
    kind: Option<String>,
    /// Scale denominator: schoolyard 1000 (default); sprint 4000 (default) or 5000. Other values
    /// are refused; for 1:3000 draw at 4000, freemapper-print enlarges when printing.
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
    /// Replace an existing map file and keep the old one as .omap.bak. Default false.
    force: Option<bool>,
    /// Use the download cache only, without network access. Default false.
    offline: Option<bool>,
    /// Also write an OCAD starting point into the folder ocad/ (OCAD 12 format, for users who
    /// edit in OCAD). Default false.
    ocad: Option<bool>,
    /// Also write <name>.pdf, the map ready to print (the map objects only, no candidates).
    /// Default false.
    pdf: Option<bool>,
    /// Also write <name>-candidates.pdf, the training map (the map plus every candidate drawn as
    /// a map object; check them in the terrain). Default false.
    pdf_candidates: Option<bool>,
    /// Draw the scale bar on the PDFs. Default false (German maps usually have none; the scale
    /// number stays in the title block).
    pdf_scale_bar: Option<bool>,
    /// Source lines on the PDFs: "auto" (default; the full list, shortened only when it takes
    /// more than 5 lines), "full" (never shortened) or "credit-only" (only the freemapper credit
    /// line; ODbL and dl-de/by-2-0 then require naming the sources when the map is passed on).
    /// ATTRIBUTION.txt always lists every source.
    pdf_sources: Option<String>,
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
    /// Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default) or
    /// 5000. Other values are refused.
    scale: Option<u32>,
    /// Map title of a new project.
    title: Option<String>,
    /// Also open the page in the default browser of this computer. Default true.
    open_browser: Option<bool>,
    /// The run panel starts with "cache only, no network" ticked. Default false.
    offline: Option<bool>,
    /// The run panel starts with the OCAD starter project ticked. Default false.
    ocad: Option<bool>,
    /// The run panel starts with the printable PDF ticked. Default false.
    pdf: Option<bool>,
    /// The run panel starts with the training PDF (every candidate) ticked. Default false.
    pdf_candidates: Option<bool>,
    /// The run panel starts with the scale bar on the PDFs ticked. Default false.
    pdf_scale_bar: Option<bool>,
    /// Source lines on the PDFs the panel starts with: "auto" (default), "full" or
    /// "credit-only", as for start_pipeline.
    pdf_sources: Option<String>,
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
    jobs: Arc<JobHub>,
    /// Area selection pages by [`folder_key`], one per folder.
    selectors: Arc<tokio::sync::Mutex<HashMap<PathBuf, Selector>>>,
}

/// The id of a tool's description in the catalog.
fn tool_id(tool: &str) -> String {
    format!("mcp-tool-{}", tool.replace('_', "-"))
}

/// The ids a parameter's description is looked up under, the tool's own first; the second is
/// for a parameter that reads the same in every tool.
fn param_ids(tool: &str, param: &str) -> [String; 2] {
    let param = param.replace('_', "-");
    [
        format!("mcp-param-{}-{param}", tool.replace('_', "-")),
        format!("mcp-param-{param}"),
    ]
}

/// The descriptions of the parameters in a tool's input schema, by parameter name.
fn param_descriptions(schema: &mut serde_json::Map<String, Value>) -> Vec<(String, &mut Value)> {
    let Some(Value::Object(properties)) = schema.get_mut("properties") else {
        return Vec::new();
    };
    properties
        .iter_mut()
        .filter_map(|(name, property)| {
            let description = property.as_object_mut()?.get_mut("description")?;
            Some((name.clone(), description))
        })
        .collect()
}

/// Replaces the descriptions of the tools and their parameters by the catalog's texts in
/// `lang`. The macros hold the English texts; a test compares them with the English catalog.
fn localize_router(router: &mut ToolRouter<FreemapperServer>, lang: &crate::text::Lang) {
    let english = crate::text::TEXTS.message_ids("en");
    let text = |id: &str| crate::text::TEXTS.text(lang, id, &[]);
    for (name, route) in &mut router.map {
        if english.contains(&tool_id(name)) {
            route.attr.description = Some(text(&tool_id(name)).into());
        }
        let mut schema = (*route.attr.input_schema).clone();
        for (param, description) in param_descriptions(&mut schema) {
            if let Some(id) = param_ids(name, &param).iter().find(|id| english.contains(*id)) {
                *description = Value::String(text(id));
            }
        }
        route.attr.input_schema = Arc::new(schema);
    }
}

/// A tool result with one text block.
fn text(text: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// A tool error the agent can relay to the user.
fn failure(message: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![ContentBlock::text(message)]))
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
        let mut tool_router = Self::tool_router();
        localize_router(&mut tool_router, &crate::text::lang());
        Self {
            tool_router,
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
        description = "Create a project folder for a map area. Give osm_id (from find_place) or a \
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
        let part = req.part;
        let area = match (req.osm_id, req.geojson) {
            (Some(osm_id), None) => {
                crate::nominatim::wait_turn().await;
                blocking(move || {
                    place_area(&HttpFetcher::new(), &osm_id, part).map_err(|e| describe(&e))
                })
                .await
            }
            (None, Some(geojson)) => resolve(&geojson).and_then(|path| {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("reading {}: {e}", path.display()))?;
                let area = MapArea::from_geojson(&text).map_err(|e| describe(&e))?;
                area.validate().map_err(|e| describe(&e))?;
                Ok(area)
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
                "Created project {} for {} ({}, 1:{}). Next: estimate_download, then start_pipeline.{}",
                dir.display(),
                area.source_id.as_deref().unwrap_or("the given area"),
                match kind {
                    MapKind::Schoolyard => "schoolyard",
                    MapKind::Sprint => "sprint",
                },
                project.scale_denominator,
                project
                    .map_kind
                    .scale_note(project.scale_denominator)
                    .map_or_else(String::new, |note| format!(" {note}"))
            )),
            Err(e) => failure(describe(&e)),
        }
    }

    #[tool(
        description = "What start_pipeline will download for a project: the tiles, each download \
                       with its size in MB, whether it is cached and whether it fits on the disk, \
                       the MB still to download, and under disk the free space of the cache and \
                       project volumes with messages for what does not fit (tell the user)."
    )]
    async fn estimate_download(
        &self,
        Parameters(req): Parameters<ProjectDir>,
    ) -> Result<CallToolResult, McpError> {
        let cache = self.cache.clone();
        let estimate = blocking(move || {
            let dir = project_dir(&req.dir)?;
            let today = chrono::Local::now().date_naive();
            let estimate = estimate_download(&dir, &cache, today).map_err(|e| describe(&e))?;
            Ok(estimate_json(&estimate, &cache, &dir))
        })
        .await;
        match estimate {
            Ok(estimate) => text(estimate.to_string()),
            Err(e) => failure(e),
        }
    }

    #[tool(
        description = "Download the data, compute the templates and write the starter project \
                       (fetch, derive, export). Runs in the background and returns a job_id; poll \
                       project_status. One pipeline runs at a time. It keeps an existing map unless force \
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
        let options = match pipeline::parse(pipeline::RunRequest {
            force: req.force,
            offline: req.offline,
            ocad: req.ocad,
            pdf: req.pdf,
            pdf_candidates: req.pdf_candidates,
            pdf_scale_bar: req.pdf_scale_bar,
            pdf_sources: req.pdf_sources,
        }) {
            Ok(options) => options,
            Err(e) => return failure(e),
        };
        // Held for as long as the job runs (the closure owns it), also against a terminal.
        let lock = match crate::lock::acquire(&dir, crate::lock::Who::Mcp) {
            Ok(lock) => lock,
            Err(e) => return failure(e.to_string()),
        };
        let cache = self.cache.clone();
        let job_dir = dir.clone();
        let started = self.jobs.start(
            &dir,
            Box::new(move |control, progress| {
                let _lock = lock;
                pipeline::run(
                    &job_dir,
                    &options,
                    &cache,
                    &HttpFetcher::new(),
                    control,
                    progress,
                )
                .map_err(|e| describe(e.as_ref()))
            }),
        );
        let id = match started {
            Ok(id) => id,
            Err(e) => return failure(e),
        };
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
        match self.jobs.status(&req.job_id, 0) {
            Some(status) => text(status.to_json().to_string()),
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
        let run = match pipeline::parse(pipeline::RunRequest {
            offline: req.offline,
            ocad: req.ocad,
            pdf: req.pdf,
            pdf_candidates: req.pdf_candidates,
            pdf_scale_bar: req.pdf_scale_bar,
            pdf_sources: req.pdf_sources,
            ..Default::default()
        }) {
            Ok(run) => run,
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
                    run,
                };
                let hub = Arc::clone(&self.jobs);
                let started =
                    Selector::start(dir.clone(), defaults, self.cache.clone(), 0, Some(hub)).await;
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
            .with_instructions(crate::text::text("mcp-instructions", &[]))
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

    /// Every tool and parameter description of the macros: the ids and the English text.
    fn described() -> Vec<([String; 2], String)> {
        let mut out = Vec::new();
        let mut router = FreemapperServer::tool_router();
        let mut names: Vec<String> = router.map.keys().map(ToString::to_string).collect();
        names.sort();
        for name in names {
            let route = router.map.get_mut(name.as_str()).unwrap();
            let description = route.attr.description.clone().unwrap_or_default();
            out.push(([tool_id(&name), tool_id(&name)], description.to_string()));
            let mut schema = (*route.attr.input_schema).clone();
            for (param, description) in param_descriptions(&mut schema) {
                let text = description.as_str().unwrap_or_default().to_owned();
                out.push((param_ids(&name, &param), text));
            }
        }
        out
    }

    #[test]
    fn the_english_descriptions_equal_the_catalog() {
        let english = crate::text::TEXTS.message_ids("en");
        for (ids, text) in described() {
            let id = ids
                .iter()
                .find(|id| english.contains(*id))
                .unwrap_or_else(|| panic!("no text for {} in the catalog", ids[0]));
            let catalog = crate::text::TEXTS.text(&crate::text::Lang::english(), id, &[]);
            // A doc comment keeps its line breaks; the catalog has one line.
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            assert_eq!(catalog, text, "{id}");
        }
        assert_eq!(
            crate::text::TEXTS.text(&crate::text::Lang::english(), "mcp-instructions", &[]),
            INSTRUCTIONS
        );
    }

    #[test]
    fn every_description_in_the_catalog_belongs_to_a_tool_or_a_parameter() {
        let reachable: std::collections::BTreeSet<String> =
            described().into_iter().flat_map(|(ids, _)| ids).collect();
        for id in crate::text::TEXTS.message_ids("en") {
            if id.starts_with("mcp-tool-") || id.starts_with("mcp-param-") {
                assert!(reachable.contains(&id), "{id} names no tool or parameter");
            }
        }
    }

    #[test]
    fn the_tools_are_described_in_german() {
        let german = crate::text::TEXTS.parse_lang_arg("de").unwrap();
        let mut router = FreemapperServer::tool_router();
        localize_router(&mut router, &german);
        let route = router.map.get("find_place").unwrap();
        let description = route.attr.description.clone().unwrap();
        assert!(description.contains("OpenStreetMap Nominatim"), "{description}");
        assert!(description.starts_with("Sucht"), "{description}");
        let schema = Value::Object((*route.attr.input_schema).clone());
        let query = schema["properties"]["query"]["description"].as_str().unwrap();
        assert!(query.starts_with("Name der Schule"), "{query}");
    }

    /// Prints the English catalog entries from the macros: `cargo test -p freemapper-cli
    /// print_the_english_descriptions -- --ignored --nocapture`.
    #[test]
    #[ignore = "a tool for the catalog, not a test"]
    fn print_the_english_descriptions() {
        let all = described();
        let mut texts: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
        let mut uses: HashMap<String, usize> = HashMap::new();
        for (ids, text) in &all {
            texts.entry(ids[1].clone()).or_default().insert(text.clone());
            *uses.entry(ids[1].clone()).or_default() += 1;
        }
        let mut printed = std::collections::BTreeSet::new();
        for (ids, text) in &all {
            let shared = ids[0] != ids[1] && texts[&ids[1]].len() == 1 && uses[&ids[1]] > 1;
            let id = if shared { &ids[1] } else { &ids[0] };
            if printed.insert(id.clone()) {
                println!("{id} = {text}");
            }
        }
        println!("mcp-instructions = {INSTRUCTIONS}");
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
    fn paths_must_be_absolute_or_home() {
        assert!(resolve("projects/school").unwrap_err().contains("relative"));
        let home = dirs::home_dir().unwrap();
        assert_eq!(resolve("~/maps/x").unwrap(), home.join("maps/x"));
        assert_eq!(resolve("~\\maps").unwrap(), home.join("maps"));
        assert!(resolve("~").unwrap_err().contains("home folder itself"));
        let absolute = std::env::temp_dir().join("x");
        assert_eq!(resolve(absolute.to_str().unwrap()).unwrap(), absolute);
    }
}
