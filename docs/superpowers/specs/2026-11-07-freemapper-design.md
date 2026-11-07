# freemapper – Design

Status: for review · Date: 2026-11-07 · Terms follow [`docs/glossary.md`](../../glossary.md).

## 1. Intent

Orienteering mappers who don't use OCAD (Windows-only, commercial) draw in OpenOrienteering Mapper (OOM).
OOM is cross-platform and free, but it cannot generate base material from geodata: no point-cloud
processing, no contours from a DTM, no automatic symbolization of cadastre data. Mappers assemble this by
hand in QGIS or with scattered tools. Research (2026-11-06) found no tool that combines official
high-resolution state geodata (DTM, nDSM, orthophoto, ALKIS) with OSM into a georeferenced OOM file.

**freemapper** fills that gap. It produces a **starter project** for OOM: a georeferenced `.omap` with the
reliable map objects already symbolized, plus templates for everything a mapper still has to judge.
Drawing and field checking stay in OOM and in the terrain. freemapper replaces neither.

Users: the author, club mapping teams (often non-technical), the orienteering community at large, and
AI agents. **Non-technical users use freemapper through an AI agent via MCP**, so the MCP server is a primary
interface, not an add-on.

## 2. Goals, non-goals, anti-goals

**First deliverable (user request):** a **schoolyard map** at **1:1000** of the grounds of the
**Clara-Zetkin-Schule, Am Sportpark 1, Strausberg Vorstadt** (52.54024 N, 13.84434 E). The map area is
the OSM school polygon (way 992423119, ≈280 × 200 m) **without** the adjacent Sportpark. It lies entirely
in LGB tile `33421-5821` (EPSG:25833 ≈ E 421623, N 5821756). Sprint maps (ISSprOM 2019-2, 1:4000) for the
whole of Strausberg Vorstadt follow as the second map kind on the same pipeline.

**Goals (MVP)**
- Map kinds `Schoolyard` (IOF school symbol set 2019, 1:1000) first, then `Sprint` (ISSprOM 2019, 1:4000).
- One static binary per OS (Linux, macOS, Windows; x86_64 + arm64) with no GIS installation needed.
- Pipeline `init → fetch → derive → build → export`, re-runnable from the project file (see §4.7).
- Three interfaces over one core: CLI, MCP server (stdio), and a local browser page for area selection.
- Output policy **"reliable as objects, everything else as template"** (user decision):
  - map objects: ALKIS buildings (rules in §4.5) and, for sprint maps, ALKIS traffic areas and OSM paths that are explicitly unpaved;
  - templates only: out-of-bounds area (private land) candidates (user decision), impassable feature
    candidates, school furniture candidates, contours, hillshade, slope, nDSM, orthophoto.
- Every generated map object carries provenance tags (source, source id, data currency) so that
  reconciliation can be added later.

**Non-goals (MVP)**
- Forest maps / ISOM, vegetation classification, cliffs (later; karttapullautin is a candidate engine).
- Reconciliation with an already edited map (later; the provenance tags prepare for it).
- Berlin and other regions (next; the architecture must make a new source adapter cheap).
- Native desktop GUI, hosted service, `.mcpb` bundle, HTTP MCP transport, code signing/notarization.
- Editing maps. OOM does that.

**Anti-goals (failure even if it "works")**
- Wrong data presented as fact: an uncertain feature ending up as a map object the mapper has to find and delete.
- Bad georeferencing: an `.omap` that is offset or rotated, or that triggers declination warnings in OOM.
- Requiring GDAL/PROJ/Python or any other installation for normal use.
- Licence violations: missing attribution, or GPL-incompatible reuse of OOM symbol sets.

## 3. Constraints and decided choices

| Topic | Decision | By |
|---|---|---|
| Language | Rust (edition 2024), source code entirely English | user |
| Docs | README + user docs bilingual DE/EN; technical specs English; glossary first | user |
| Tech alternatives | Go and Elixir/Phoenix/Ash considered and rejected | user + Claude |
| First map | schoolyard map 1:1000, Clara-Zetkin-Schule, school grounds only | user |
| Schoolyard symbols | "IOF Symbol Set for School Orienteering Maps 2019": all ISSprOM 2019 symbols at 125 %, 501 redefined, 6 additional symbols (§4.3) | user asked for the standard; Claude found the IOF document |
| Dependencies | pure-Rust stack; C libraries (GDAL/PROJ/PDAL) only as later optional features | Claude |
| Core as library | core is a library crate so a later hosted service can embed it | Claude |
| Licence | GPL-3.0-or-later (OOM symbol sets and CRT files are GPL) | Claude |
| MCP | `rmcp` (official SDK), stdio transport; clients: Claude Code / CLI agents, any standard MCP client | user |
| Re-runs | MVP produces a starter project only; reconciliation later | user |
| Private land | template only | user |
| Process | Superpowers workflow (plan → subagent-driven development, TDD for parsers/writers/algorithms). Speed first: light gates now, ub-mode-style gates later | user |
| Formatting/lint | automatic: Claude Code hooks (rustfmt after edits, clippy at stop), lefthook pre-commit, CI on 3 OSes | user |

## 4. Architecture

Cargo workspace, kept small for speed:

```
crates/
  freemapper/          library: all domain logic
    project/           ProjectFile (freemapper.toml), Project folder layout
    area/              MapArea, BoundingBox, buffer, geocoding (Nominatim), OSM polygon lookup
    crs/               EPSG:25833 <-> EPSG:4326, grid convergence, scale factor, declination (WMM)
    cache/             tile cache in dirs::cache_dir()/freemapper, shared across projects
    sources/           SourceAdapter trait + adapters: lgb_brandenburg, osm
    raster/            simple georeferenced raster type, world files, mosaic + clip
    derive/            dtm (from DGM1 tiles), hillshade, slope, contours, ndsm (from nDOM LAZ)
    build/             symbol mapping rules -> MapObjects, Candidates, Templates
    omap/              OOM XML writer (+ minimal reader for tests), symbol sets embedded,
                       symbol enlargement, school additions
    preview/           PNG rendering of a layer
  freemapper-cli/      binary `freemapper`: CLI subcommands, `mcp` (stdio server), `select` (axum web page)
```

Long-running steps report progress through a `Progress` callback trait. The CLI shows a progress bar; the MCP
server sends MCP progress notifications (see §4.6 for jobs).

### 4.1 Data sources (Brandenburg, verified live 2026-11-06/07)

LGB open data, licence dl-de/by-2-0, no login: `https://data.geobasis-bb.de/geobasis/daten/`.
Tiles are 1×1 km in EPSG:25833, named `<prod>_33EEE-NNNN`, where EEE/NNNN give the SW corner in km.

| Product | URL pattern | Use |
|---|---|---|
| DGM1 | `dgm/tif/dgm_33EEE-NNNN.zip` (GeoTIFF f32, 1 m, pixel-centre registration) | hillshade, slope, contours |
| nDOM | `ndom/NDOM_33EEE-NNNN_LAS12.laz` (0.2 m grid as LAS 1.2 points, no CRS VLR → assign 25833) | nDSM template (hedges, walls, trees, equipment) |
| DOP20 RGB | `dop/rgb_jpg/dop_33EEE-NNNN.zip` (JPEG + world file, flown 2026-05) | orthophoto template |
| ALKIS | OGC API Features `https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1`, collections `gebaeude_bauwerk`, `nutzung`, `flurstueck`, GeoJSON by bbox | buildings, land use, parcels |
| Currency | `https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_*_aktualitaet.csv` | data currency per tile |

ALS LAZ (≈100 MB/tile, flown 2018), bDOM and DOP RGBI are not needed for the MVP.

OSM: Geofabrik extract `brandenburg-latest.osm.pbf`, cached and read with `osmpbf`, filtered to the bbox.
Public Overpass instances are not a backend. One exception: `area/` may resolve a single OSM element
(e.g. the school polygon) through the OSM API or Nominatim when the user names a place.

Attribution, written to the map notes, a text object outside the map area, and `ATTRIBUTION.txt`:
`© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` and `© OpenStreetMap contributors, ODbL`.
Every generated project (map notes, template map notes, `ATTRIBUTION.txt`) and every generated PDF map
also carries `Created with freemapper – https://github.com/HenryJobst/freemapper` (user, 2026-11-07).

### 4.2 Project folder

```
clara-zetkin-schule/
  freemapper.toml            map area (GeoJSON polygon, EPSG:4326), map kind, scale, buffer, parameters,
                             per-tile URL + data currency, OSM extract timestamp, declination date
  clara-zetkin-schule.omap   the starter project
  templates/
    orthophoto.jpg  hillshade.png  slope.png  ndsm.png   (each with a world file .jgw/.pgw)
    candidates.omap          template map with candidates (contours, out-of-bounds, impassable features,
                             school furniture …), symbolized with the same symbol set
  data/
    alkis-buildings.geojson  ALKIS buildings as fetched (GeoJSON, EPSG:25833), input of the export
    osm-candidates.geojson   OSM features matching the candidate rules (GeoJSON, EPSG:4326)
    contours.geojson         contour lines from derive (GeoJSON, EPSG:25833, property height)
    alkis-land-use.geojson   ALKIS land use per parcel part (nutzung_flurstueck), source of 520 candidates
  ATTRIBUTION.txt
```

Vector templates are a **template map** (`TemplateMap`) rather than GeoPackage. It needs no SQLite, it
renders with the right symbols, and the mapper can copy objects from it into the main map.
*If OOM turns out not to georeference a `TemplateMap` reliably, fall back to Shapefile + `.prj` as an
`OgrTemplate`.* Verify this in M1.

### 4.3 OOM file writing and symbol sets

Facts from the OOM sources (master, v0.9.6):
- Write `.omap` (compact XML, namespace `http://openorienteering.org/apps/mapper/xml/v2`, `version="9"`).
  The writer can also emit `.xmap` (pretty) for debugging and golden tests.
- Coordinates are i32 in 1/1000 mm on paper, with y pointing south. Every ring ends with a repeat of its first
  point flagged 18 (hole + close), and `coords count` must be exact.
- Georeferencing: `projected_crs id="EPSG"` with parameter 25833 and ref point = centroid of the map area.
  `scale`, `grid_scale_factor` (the *combined* factor), `declination` (WMM at the ref point on the declination
  date) and `grivation = declination − grid convergence` must be written consistently.
- Objects reference symbols by **id**, so freemapper keeps a code → id table generated from the embedded set.
- Templates are hidden by default in OOM, so write `<map_view>` refs with `visible="true"` and sensible opacity.
- Objects carry `<tags>`: `freemapper:source`, `freemapper:source_id`, `freemapper:currency`.
- Evaluate the crate `omap` (yvind/omap-rs, MIT) first. Writing our own is acceptable and probably simpler.

**Sprint symbol set:** embed OOM's `symbol sets/4000/ISSprOM 2019_4000.omap` and copy `<colors>` and `<symbols>`
verbatim.

**Schoolyard symbol set** (IOF "Symbol Set for School Orienteering Maps 2019", 1:1000; PDF:
`https://old.orienteering.sport/wp-content/uploads/2019/02/Symbol-set-for-school-orienteering-maps-2019-1.pdf`):
- all ISSprOM 2019 symbols enlarged to **125 %**;
- 501 paved area: brown 30 % for light traffic, 50 % for heavy traffic and tartan track/field, black border
  with the line width of 505;
- 6 added symbols: 321 hydrant (P), 541 playground equipment (L), 542 table (P), 543 bench (L),
  544 pole (P), 545 lamppost (P, north-oriented). Dimensions are in the PDF.
- Peripheral information: map scale and contour interval are mandatory. Also add title, copyright/attribution,
  a red bar on the north edge, and a bar scale, all as objects outside the map area. A legend is optional (later).
  *Implemented in M4b:* magnetic north lines (601, 30 mm × enlargement) over the map content; title,
  `Maßstab 1:… · Äquidistanz … m`, a 50 mm bar scale, attribution and credit line below it, with four
  sheet symbols 991–994 appended to both sets. The red north bar is left out for now: ISSprOM has no
  red colour and the bar has no norm source.

Producing the school set: first look for an existing GPL-compatible OOM school set (OOM forum, club
repos). If none exists, generate it once from `ISSprOM 2019_4000.omap`. Write a symbol enlargement in Rust that scales
every dimension in the symbol XML by 1.25: line widths, dash/gap lengths, mid-symbol distances, point element
coordinates and radii, area pattern spacing/widths, text sizes. Then add the 6 symbols as hand-written XML and
patch 501. Check the result in OOM against OOM's own "scale symbols" result. Commit the generated set as an
asset (`assets/symbol-sets/school-2019_1000.omap`) so the binary embeds a fixed file.

### 4.4 Derivation (MVP)

- DTM: mosaic DGM1 tiles over map area + buffer (default 20 m for schoolyard, 50 m for sprint).
- Hillshade (azimuth 315°, altitude 45°) and slope with 3×3 kernels, written as PNG + world file.
- Contours: smooth the DTM (Gaussian, configurable σ), then marching squares (`contour` crate), simplify, clip.
  Interval: schoolyard 1 m (index 5 m), sprint 2 m (index 10 m). Contours are candidates only.
- nDSM: rasterize nDOM points (max Z per 0.2 m cell), written as PNG + world file with a height colour ramp.
  LGB publishes no currency for nDOM tiles; the cache is keyed by the server's `Last-Modified` date (a
  server without that header gives an undated file that is not refreshed until it is deleted).
- Orthophoto: mosaic + clip DOP20 RGB JPEG tiles, written as JPEG + world file.
- Raster templates use image + world file instead of GeoTIFF (decided in M2/M3a): OOM's `TemplateImage`
  reads world files and takes the CRS from the template's `<crs_spec>`, OCAD reads world files too, and
  no GeoTIFF writer is needed.

### 4.5 Symbol mapping

Mapping rules live in a TOML data file embedded in the binary. A project can override it, following the
idea of OOM's `.crt` files. OOM's `OSM-ISSprOM 2019.crt` is the source for OSM tag knowledge, **filtered by the
reliability policy below**. It is not taken over literally.

**Buildings (ALKIS `gebaeude_bauwerk`):**
- *object 521:* existing, above-ground buildings (`rellage` not underground) above the minimum size of the
  map kind. Sheds and garages count.
- *candidate 522 canopy:* roofed open structures (carports, shelters, canopies, identified by `funktion`/`gebnutzbez`).
- *dropped:* underground buildings, buildings under construction or planned, ruins, non-building structures
  (bridges, towers, tanks, walls). Walls may become impassable-feature candidates if the attribute is clear.
- Verify the exact attribute values against live data for tile `33421-5821` and list them in the rule file.

**Paths and areas (sprint only in the MVP):**
- *object 501 paved area:* ALKIS `nutzung` traffic classes (the exact `nutzart` values are verified live).
- *object 505:* OSM `highway=track|path|footway|bridleway` with an explicit unpaved `surface`
  (`unpaved, dirt, earth, ground, grass, gravel, fine_gravel, compacted, sand, woodchips`). Object 506 when
  additionally `width < 1.5` or `highway=path` with `trail_visibility` below good. 507 is not used.
- *candidate:* OSM paths with no `surface` tag.
- *dropped:* `footway=sidewalk|crossing` and paved OSM ways (ALKIS areas and the orthophoto cover them).

**Candidates (all map kinds), in `candidates.omap`:**
- out-of-bounds 520: residential parcels minus buildings/traffic areas (sprint); for the schoolyard, the
  non-school parcels inside map area + buffer;
- impassable features: OSM `barrier=fence|wall|hedge|retaining_wall`;
- schoolyard: OSM `leisure=pitch|track` (501 50 % / tartan), `playground=*` (541), `leisure=picnic_table`
  and `amenity=table` (542), `amenity=bench` (543), `highway=street_lamp` (545), `flagpole`/`man_made=mast` (544),
  `emergency=fire_hydrant` (321), `natural=tree` (ISSprOM tree symbol), `amenity=school` paved surfaces (501);
- contours 101/102 (see §4.4).

Generalization: drop buildings below the minimum size and simplify outlines (≈0.1 mm map tolerance; at 1:1000
that is 0.1 m).

### 4.6 Interfaces

CLI (`freemapper`):
- `init <dir> --place "Clara-Zetkin-Schule Strausberg" | --osm way/992423119 | --bbox … | --geojson file
  [--kind schoolyard|sprint] [--scale 1000]`
- `select <dir>`: opens the browser page
- `fetch`, `derive`, `build`, `export`, `run` (all steps), `status`, `preview <layer> -o out.png`
- `mcp`: MCP server on stdio

MCP tools mirror the CLI. They are self-describing and written for agents serving non-technical users:
`find_place`, `create_project`, `open_area_selector` (returns the URL), `estimate_download`, `start_pipeline`
(returns a job id immediately), `project_status` (job state, progress, warnings; agents poll it),
`preview_layer` (returns a PNG image), `list_layers`. Progress notifications go out as well, but agents
don't depend on them, so client timeouts on long downloads don't matter.

Area selection page: axum serves a single embedded page (Leaflet + OSM tiles, assets embedded with
`rust-embed`). The user draws a rectangle or polygon or picks an OSM polygon. The page shows the LGB 1 km
tile grid and an estimated download size, and saving writes the map area to `freemapper.toml`.

Distribution: GitHub Releases with unsigned binaries. The README explains the Gatekeeper/SmartScreen
prompts and how to register the MCP server in Claude Code (`claude mcp add freemapper -- freemapper mcp`)
and in generic MCP client configs.

### 4.7 Reproducibility (Claude's call)

"Re-runnable and traceable", not byte-identical across time:
- `freemapper.toml` records the URL and data currency of every tile, the OSM extract timestamp, and the
  declination date, which is fixed at `init` and not "today" on each run.
- The cache is keyed by tile id + data currency, and the latest version per tile is kept. If upstream
  currency changed since `init`, `fetch` warns, uses the new data, and updates the record. `--offline` uses
  the cache only.
- Byte-identical output is guaranteed (and tested) only for fixed inputs, e.g. the fixture integration test.

## 5. Testing and quality

- TDD for the OOM writer (golden `.xmap` files), symbol enlargement (every dimension attribute scaled),
  coordinate transforms (reference points checked against PROJ output), tile id computation,
  contour/hillshade on synthetic rasters, and symbol mapping rules.
- Fixture data: a small clipped excerpt of tile `33421-5821` around the school (dl-de/by-2-0, attribution in
  the fixture folder) and a small OSM PBF excerpt (ODbL).
- Integration test: the full pipeline on fixtures with no network (sources behind the trait).
- Manual acceptance: open `clara-zetkin-schule.omap` in OOM 0.9.6. It must open without warnings. Buildings
  must line up with the orthophoto template, templates must be visible, and symbol sizes must match the IOF
  school set (125 %).
- Gates: `cargo fmt`, `cargo clippy -D warnings`, `cargo test` via Claude Code hooks, lefthook pre-commit, and
  GitHub Actions on Linux/macOS/Windows. Release builds for 6 targets.

## 6. Milestones

| # | Result |
|---|---|
| M0 | Repo: workspace, glossary, lint/format hooks, lefthook, CI, licence, bilingual README stub |
| M1 | `.omap` writer + georeferencing + school symbol set: empty 1:1000 school map for the Clara-Zetkin-Schule opens in OOM without warnings; `TemplateMap` georeferencing verified |
| M2 | Project file, map area (OSM polygon/bbox/GeoJSON/place), cache, LGB tile fetch with currency |
| M3 | Templates: orthophoto, nDSM, hillshade, slope, contours (candidates) |
| M4 | ALKIS buildings as map objects with provenance; peripheral information → **first usable schoolyard map** |
| M5 | OSM candidates (`candidates.omap`): school furniture, barriers, pitches, trees, out-of-bounds |
| M6 | MCP server: tools, jobs, preview PNGs |
| M7 | Sprint map kind (1:4000) for Strausberg Vorstadt: ALKIS traffic areas, OSM unpaved paths |
| M8 | Browser area selection page |

**Later:** Quick training map (user, 2026-11-07): an optional switch (e.g. `--full`) that generates
the most complete map possible and renders it to PDF. Unlike the default output policy, candidates
(OSM furniture, barriers, contours, nDSM-derived features) become map objects. This needs a symbol
renderer for the OOM symbol set (lines, dashes, area patterns, point symbols, text) plus PDF output, and
peripheral information on the sheet. The PDF names freemapper and its repository like every generated project (§4.1). OCAD starter project (user, 2026-11-07): the same content as an `.ocd` file for further
editing in OCAD. Options to check: write OCD directly (OCAD publishes its file format), or rely on OOM's
OCD export. Keep map objects, symbols and templates in a format-neutral model so a second writer can be
added; image templates with world files already work in OCAD. Berlin adapter (ATOM/WFS, 2 km tiles, dl-de/zero-2-0), forest maps (ISOM, vegetation, cliffs via
karttapullautin), reconciliation, other regions, optional GDAL feature, legend generation, stricter quality
gates (ub-mode style).

## 7. Left to the builder

Internal module boundaries beyond §4, crate choices within the pure-Rust constraint, smoothing and
simplification defaults, the exact ALKIS attribute values in the rule file (verified against live data),
preview styling, and the web page design.
