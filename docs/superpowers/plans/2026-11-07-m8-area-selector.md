# M8: Browser Area Selection – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper select <dir>` opens a local page in the browser. On it the user draws a rectangle
or polygon or picks an OSM place, sees the LGB 1 km tile grid and the estimated download, and saves the
map area to `freemapper.toml`. It creates the project or changes the area of an existing one. The MCP
tool `open_area_selector` starts the same page and returns its URL.

**Architecture:** The library gains a UTM inverse, an area preview (tile outlines in WGS84, size,
too-large check, download estimate for an area that is not saved yet) and `set_map_area`. The CLI
crate hosts an axum server (`select.rs`) on 127.0.0.1 with a random port and a random token in every
path. It embeds the page (`web/`: `index.html`, `app.js`, `app.css`, Leaflet 1.9.4 vendored) with
rust-embed and exposes a small JSON API. The page is vanilla JS on Leaflet: click two corners for a
rectangle, click vertices for a polygon, or search a place. The CLI waits until the page says "done"
or Ctrl+C. The MCP server keeps the selector running in the background.

**Tech Stack:** `axum = "0.8.9"`, `rust-embed = "8.13.0"` (both older than two weeks), Leaflet 1.9.4
(BSD-2-Clause, vendored), `ureq` as a CLI dev-dependency for the HTTP tests.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.6 (select, open_area_selector,
area selection page).

## Global Constraints

- Bind 127.0.0.1 only. Every route lives under `/<token>/` (32 hex characters from the OS random
  source), and a request without the token gets 404. Requests whose `Host` is not
  `127.0.0.1:<port>` or `localhost:<port>` are refused with 403 (DNS rebinding).
- API (JSON, under the token):
  - `GET project`: kind, scale, title, area GeoJSON or null, `has_map`.
  - `POST preview {area}`: `tiles [{name, corners [[lat,lon]×4]}]`, `width_m`, `height_m`,
    `too_large`, `estimate`.
  - `GET places?q=`: like `find_place`.
  - `GET place?osm_id=`: area GeoJSON.
  - `POST save {area, kind?, scale?, title?}`: creates the project or sets its area. Refused when the
    area is too large.
  - `POST done`: stops the server.
- Backgrounds: OSM tiles (`https://tile.openstreetmap.org/{z}/{x}/{y}.png`, "© OpenStreetMap
  contributors") and LGB DOP20 WMS (`https://isk.geobasis-bb.de/mapproxy/dop20c/service/wms`, layer
  `bebb_dop20c`, EPSG:3857, "© GeoBasis-DE/LGB, dl-de/by-2-0"). The page names freemapper and
  `https://github.com/HenryJobst/freemapper`.
- UI text in German or English by `navigator.language`. Source code is English.
- Nominatim: same pacing (≥ 1 s) and User-Agent as the MCP server, via the library.
- `select` prints the URL to stdout and opens the browser (`open` / `xdg-open` / `cmd /c start`)
  unless `--no-browser`. `--port` is optional (default 0 = random).

## Review Focus

1. **Another website POSTs to the local server** – refused (token, Host check).
2. **Saving a too-large area, or saving into an existing project with a map** – refused with a
   reason; existing project: the area changes, the map keeps its georeferencing (page says so).
3. **A polygon with fewer than 3 points or self-intersecting input** – clear error, no panic.
4. **Browser cannot be opened (headless)** – URL printed, server keeps running.
5. **MCP: `open_area_selector` twice for the same folder** – the same URL, no second server.

---

### Task 1: Library – inverse, area preview, set_map_area
`crs::utm33_inverse(ProjectedPoint) -> LatLon` (series inverse matching the forward;
roundtrip < 1 mm). `layers::preview_area(area, kind, cache, today, alkis_cached) -> AreaPreview`
(shares the estimate items with `estimate_download`). `project::set_map_area(dir, area)` (keeps every
other field, checks the size). Tests: `utm33_roundtrip`, `preview_lists_tile_outlines`,
`preview_flags_too_large`, `set_map_area_keeps_the_rest`, `set_map_area_refuses_too_large`.

### Task 2: Server and `freemapper select`
`crates/freemapper-cli/src/select.rs` (router, token, Host check, embedded assets, shutdown);
`Select { dir, kind, scale, title, no_browser, port, cache }`. Tests (`tests/select.rs`, spawn the
binary with `--no-browser`, read the URL from stdout): `page_and_assets_are_served`,
`requests_without_token_are_404`, `foreign_host_is_403`, `preview_reports_tiles`,
`save_creates_the_project`, `save_changes_an_existing_area`, `save_refuses_too_large`,
`bad_area_is_a_400`, `done_stops_the_server`.

### Task 3: The page
`web/index.html`, `web/app.js`, `web/app.css`, `web/vendor/leaflet/` (+ licence). Checked in a real
browser (Playwright screenshot): draw, grid, estimate, save.

### Task 4: MCP tool, docs, acceptance
`open_area_selector {dir, kind?, scale?, title?}` → URL (the same server per folder; it opens the
browser too). README DE/EN, glossary (`Gebietsauswahl` / area selector). Live check with the school.
