# M6: MCP Server – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper mcp` is an MCP server on stdio, so an AI agent can serve a non-technical user:
find the school by name, create the project, show what will be downloaded, start the pipeline as a job,
poll its status and show preview images of the layers.

**Architecture:** The library gains a place search (Nominatim, the single-element exception of spec
§4.1), a download estimate, a layer list and PNG previews — all synchronous and testable. The CLI crate
hosts the MCP server (`rmcp`, stdio, tokio only there): thin tools over the library; `start_pipeline`
runs `run_project` on a blocking thread and records progress in an in-memory job table that
`project_status` reads. `init` gains `--osm` and `--place`.

**Tech Stack:** `rmcp = { version = "=3.6.0", features = ["server", "transport-io", "macros"] }` (3.7+ are
younger than two weeks), `tokio = { version = "1.54.1", features = ["macros", "rt-multi-thread", "io-std"] }`,
`schemars = "1.2.3"` — CLI crate only.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.6 (MCP tools mirror the CLI:
`find_place`, `create_project`, `open_area_selector`, `estimate_download`, `start_pipeline` returning a job id,
`project_status`, `preview_layer` returning a PNG, `list_layers`; agents poll), §4.1 (Nominatim for a single
element). Scope cut: `open_area_selector` needs the browser page (M8); MCP progress notifications are
left out because agents poll `project_status`.

## Global Constraints

- Nominatim: `https://nominatim.openstreetmap.org/search?q=<query>&format=jsonv2&limit=5&polygon_geojson=1`
  and `…/lookup?osm_ids=W<id>&format=jsonv2&polygon_geojson=1` (W/R/N for way/relation/node). Usage policy:
  identifying User-Agent — `freemapper/<version> (+https://github.com/HenryJobst/freemapper)` for every
  request — and at most one request per second (one per tool call / CLI call).
- A place without a polygon (a node) cannot be a map area: error "… is a point, not an area; pick its
  grounds (a way or relation)".
- Tool names and descriptions in English (source code rule); results are plain facts an agent can
  relay to the user in any language.
- MCP server info: name `freemapper`, version from Cargo, instructions describing the workflow
  (find_place → create_project → estimate_download → start_pipeline → project_status → preview_layer).
- Jobs live in memory for the server's lifetime; ids `job-<n>`; state `running | finished | failed`, the
  last 50 progress lines, warnings (lines starting with `warning:`), the result (map path) or the error.
- Previews: PNG, longest side ≤ 800 px, layers `orthophoto`, `hillshade`, `slope`, `ndsm`.

## Review Focus

1. **A place search with several or no matches** – a ranked list, areas first; empty list is not an error.
2. **A job that fails** (offline, empty cache) – status `failed` with the error text, server keeps running.
3. **Two jobs for the same project at once** – the second is refused while the first runs.
4. **Paths from the agent** – relative paths, `~`, missing folders: clear errors, no panic.
5. **Layer not derived yet** – `preview_layer` explains which step makes it.

---

### Task 1: Place search (library)
`crates/freemapper/src/area/places.rs` (area becomes a folder module or a sibling file `places.rs`):
`PlaceCandidate { osm_id: String /* way/1 */, name, display_name, category, kind, has_area: bool }`,
`search_url(query)`, `parse_search(json) -> Result<Vec<PlaceCandidate>, AreaError>` (areas first, keep the
Nominatim order otherwise), `lookup_url(osm_id) -> Result<String, AreaError>`, `area_from_lookup(json, osm_id)
-> Result<MapArea, AreaError>`, `find_places(fetcher, query)`, `place_area(fetcher, osm_id)`; HttpFetcher
User-Agent as above. Tests on fixture JSON (way with polygon, node without, empty result, bad osm id).

### Task 2: `init --osm` / `--place`
CLI: exactly one of `--geojson`, `--osm way/…`, `--place "…"` (the best area match; prints the chosen
place). Tests: argument conflicts; `--osm` with a bad id fails cleanly (no network needed).

### Task 3: Estimate, layers, previews (library)
`project::estimate_download(dir, cache) -> DownloadEstimate { tiles, items: Vec<EstimateItem { what,
megabytes, cached }>, total_megabytes }` (DOP 8 MB, DGM 1.3 MB, nDSM 15 MB per tile; OSM extract 300 MB
unless cached; ALKIS ≈ 0); `project::list_layers(dir) -> Vec<Layer { name, file, role, ready, made_by }>`;
`project::preview_png(dir, layer, max_px) -> Result<Vec<u8>, ProjectError>`. Tests with the synthetic
fetcher.

### Task 4: MCP server
`crates/freemapper-cli/src/mcp.rs`: tools `find_place {query}`, `create_project {dir, osm_id | geojson,
kind?, scale?, title?}`, `estimate_download {dir}`, `start_pipeline {dir, force?, offline?}`,
`project_status {job_id}`, `list_layers {dir}`, `preview_layer {dir, layer}` (image content);
`freemapper mcp [--cache-dir]`. Tests: stdio integration test spawning the binary — initialize, the
tool list, `create_project` from the fixture GeoJSON, `list_layers`, `start_pipeline` offline with an
empty cache → `project_status` reports `failed` naming the tile, a second `start_pipeline` while one runs
is refused (a slow job is not needed: refuse while state is `running`, test via the job table).

### Task 5: Docs and acceptance
README DE/EN: register in Claude Code (`claude mcp add freemapper -- freemapper mcp`), generic MCP config
snippet, the workflow in plain words. Live test: `find_place("Clara-Zetkin-Schule Strausberg")` finds
`way/992423119`.
