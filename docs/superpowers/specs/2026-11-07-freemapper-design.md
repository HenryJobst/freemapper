# freemapper – Design

Status: for review · Date: 2026-11-07 · Terms follow [`docs/glossary.md`](../../glossary.md).

Decisions are recorded as ADRs in `docs/adr/`.

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
| AI features | optional: a feature that calls a model runs only when a model is configured, and freemapper does every job in full without one. The MCP server is not affected, because an outside assistant drives the program there (ADR 0037) | user (suite rule) |

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

**Berlin (verified live 2026-11-07, details `docs/regions/berlin-reference.md`):** LGB serves DOP20, DGM1 and
bDOM for all of Berlin in the same tile scheme; the nDOM LAZ is 404 inside Berlin, so there the nDSM template is
bDOM (0.2 m GeoTIFF, `bdom/tif/bdom_33EEE-NNNN.zip`, currency `bb_bdom_aktualitaet.csv`) minus DGM1 (bilinear),
clamped at 0. The LGB ALKIS service returns nothing inside Berlin and the Berlin WFS nothing outside, so ALKIS
is per region (`sources/region.rs`, boundary `assets/regions/berlin.geojson`, 100 m margin; a border area
queries both): buildings `https://gdi.berlin.de/services/wfs/alkis_gebaeude` (`alkis_gebaeude:gebaeude`), land
use `https://gdi.berlin.de/services/wfs/alkis` (`alkis:tatsaechlichenutzungflaechen`), GeoJSON, normalised to
the LGB property schema; up to 3 attempts per page (the server drops about 1 request in 20). Berlin squares
(`AX_Platz`, never refined) get `bez = "ohne Unterart (Berlin)"` and stay candidates. Attribution: for an area touching
Berlin the LGB line becomes `© GeoBasis-DE/LGB <year>, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten
geändert` (LGB's licence note); once Berlin ALKIS objects were fetched (`AlkisRecord.regions`),
`© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert` is added. bDOM tiles are fetched one by one,
so one failing tile costs only its part of the nDSM.

**Saxony (verified live 2026-11-07, details `docs/regions/saxony-reference.md`):** regions are data
(`sources/region.rs`, boundaries `assets/regions/{brandenburg,berlin,saxony}.geojson`). ALKIS comes from every region
the widened bbox touches; rasters from the provider of the region holding the bbox centre (LGB for Brandenburg and
Berlin, GeoSN for Saxony); OSM from every touched region's Geofabrik extract, merged by id. GeoSN: DGM1/DOM1 2 km
tiles (currency and download links from the ArcGIS tile index, `Stand`), orthophoto from the WMS `sn_dop_020` at
0.2 m stitched from ≤ 4000 px pieces (the 230–270 MB DOP tiles are not used), nDSM = DOM1 − DGM1 at 1 m (no open
nDOM/LiDAR), ALKIS from the WFS `ave:GebaeudeBauwerk` / `ave:NutzungFlurstueck` as GML 3.2 (paged, 3 attempts per
page); squares without `bez` stay candidates. Attribution `Quelle: GeoSN, dl-de/by-2-0, Daten geändert`.

**Saxony-Anhalt (verified live 2026-11-07, details `docs/regions/saxony-anhalt-reference.md`):** all LVermGeo data is
EPSG:25832, so freemapper works in two UTM zones (`crs::Crs`): each project takes the CRS of its raster region
(`ProjectFile.crs`, default 25833), rasters stay on their native grid, ALKIS from a service in the other zone is
reprojected through lat/lon, and region tests stay in 25833. LVermGeo rasters are fetched per area: orthophoto from the
WMS `lsa_lvermgeo_dop20_2`, DGM1/DOM1 from the WCS (`multipart/related`, ≤ 4000 m per request), flight dates from the
tile index WFS (`truedop_kacheln`, `bdom_aktualitaet_zyklus`); nDSM = DOM1 − DGM1 at 1 m. ALKIS from the WFS
`ave:GebaeudeBauwerk` / `ave:Nutzung` (GML, shared reader with GeoSN; `rellage` present). OCAD grid code 8007 for zone 32.
Attribution `© GeoBasis-DE / LVermGeo ST <year>, dl-de/by-2-0, Daten geändert`.

**Hamburg (verified live 2026-11-07, details `docs/regions/hamburg-reference.md`):** LGV, EPSG:25832 (second zone-32
provider; a provider change inside a zone drops the records like a zone change). Orthophoto from the WMS
`dop_zeitreihe_unbelaubt` pinned to `TIME=2026` (2023 is a closed-licence satellite vintage), flight date from the AdV
metadata CSV. DGM1 and bDOM (2022, 1 m) read per 1 km tile by HTTP range out of the statewide zips (central directory
from the archive's end, deflated members checked against their CRC, cached per tile and archive release; a
server ignoring `Range` is refused, a clamped range is an error). ALKIS from the OGC API
`alkis_vereinfacht` (`GebaeudeBauwerk`, `NutzungFlurstueck`, 25832) with the ASCII spellings normalised to the LGB ones. When the OGC API fails, the classic WFS
`WFS_HH_ALKIS_vereinfacht` (the same data, GML) is asked instead.
Attribution `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <year>, dl-de/by-2-0, Daten
geändert`.

**Hesse (verified live 2026-11-08, details `docs/regions/hessen-reference.md`):** HVBG, EPSG:25832. Orthophoto from the
WMS (images ≤ 3000 px; flight date by GetFeatureInfo; summer flights). DGM1 and DOM1 (laser scan, 1 m) from the
Downloadcenter's municipal zips by HTTP range: tile → municipality from the metadata xlsx (which also dates the
tiles), file URLs resolved before each download, unresolvable places reported. ALKIS from the WFS
(`ave:NutzungFlurstueck` etc.). Attribution `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation
(HVBG) <year>, dl-de/zero-2-0, verändert`.

**Schleswig-Holstein (verified live 2026-11-08, details `docs/regions/schleswig-holstein-reference.md`):** LVermGeo SH,
EPSG:25832. Orthophoto from the WMS in 4000 px pieces, the flight date from the metadata WMS (the state is flown in two
halves, 2024 and 2025; no time choice). DGM1 from the INSPIRE ATOM GeoTIFFs (tile names from the portal's `overview.php`,
XYZ as fallback). The bDOM (0.2 m, image-based) as whole tiles of 105 MB, reduced to 1 m, only the 1 m copy is kept; the
nDSM is bDOM − DGM1, with a warning when the bDOM is older than the orthophoto. The Downloadportal host
`geodaten.schleswig-holstein.de` omits its intermediate certificate: the D-TRUST chain is pinned for that host only.
ALKIS from the VwG WFS (`ave:NutzungFlurstueck`). Attribution `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)`
plus the licence link.

**Mecklenburg-Vorpommern (verified live 2026-11-08, details `docs/regions/mecklenburg-vorpommern-reference.md`):** LAiV M-V,
EPSG:25833 like Brandenburg. Orthophoto from the DOP WCS (JPEG pieces of at most 4000 px on the grid; the WMS carries a
watermark), the flight date from `overview.php`. DGM1 and DOM1 from the WCS (plain TIFF, LZW), masked to the set of the
2 km tiles (the models carry no NoData); the `WcsService` is shared with Saxony-Anhalt. nDSM = DOM1 − DGM1. ALKIS from the
LGB's OGC API with a WFS fallback. Attribution `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)` plus the licence link.

**Nordrhein-Westfalen (verified live 2026-11-08, details `docs/regions/nordrhein-westfalen-reference.md`):** Geobasis NRW
(Bezirksregierung Köln), EPSG:25832 for the whole state (the map never uses zone 33). Orthophoto from the WMS (the tiles are
JPEG 2000 only), the flight date per tile. DGM1 and DOM1 as 1 km laser tiles of the same flight; file names and dates only
from the per-product tile list (the directory listing carries stale duplicates, and the name carries the year); nDSM =
DOM1 − DGM1. ALKIS from the WFS with the LGB spellings and a detailed `funktion`. OSM from the five Regierungsbezirk extracts
instead of the 917 MB statewide one (ADR 0028). At the Hesse border both ALKIS services are read and the rasters come from
the state of the centre. Licence dl-de/zero-2-0; attribution `Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) <year>,
dl-de/zero-2-0`.

**Niedersachsen (verified live 2026-11-08, details `docs/regions/niedersachsen-reference.md`):** LGLN, EPSG:25832 for the
whole state. Rasters per area (ADR 0029): orthophoto from the WMS `ni_dop20` in pieces of at most 4000 px; DGM1 and DOM1
from the WCS (`ni_dgm1`, `ni_dom1`, Float32, NoData −9999) through the shared `WcsService` in pieces of at most 2 km (a
4 km piece took 48.6 s against the 60 s timeout); an HTTP 200 HTML answer ("MapServer Message") is an error. Dates from
the STAC searches (`dop`, `dgm`, `dom.stac.lgln.niedersachsen.de`): the newest item at the area's centre, not cached
beyond the project record (ADR 0033); the flight month gives the leaf state. nDSM = DOM1 − DGM1, with a warning when the
DOM1 is more than 24 months older than the orthophoto (ADR 0032); bDOM20 is not used. ALKIS from the WFS
`alkis_wfs_einfach` (`ave:GebaeudeBauwerk`, `ave:NutzungFlurstueck`, `COUNT` ≤ 10000, names trimmed, underground by
`rellage`). OSM from `niedersachsen-latest` (510 MB), which includes Bremen; Bremen and Bremerhaven are holes in the
boundary and stay outside the supported states until Bremen gets its own region (ADR 0035). At the Hamburg border both
ALKIS services and both extracts are read. An area reaching more than 50 m beyond every supported state (abroad, sea,
or a German state not supported yet such as Bremen or Thuringia) is warned for every provider (ADR 0034). Attribution `© GeoBasis-DE/LGLN <year>, Daten geändert, CC BY 4.0` plus
the licence link.

**Rheinland-Pfalz (verified live 2026-11-08, details `docs/regions/rheinland-pfalz-reference.md`):** LVermGeo RP
(`Provider::LvermGeoRp`, oids `DERP…`, ADR 0042), EPSG:25832 for the whole state; 84.5 % of it has negative 25833 eastings,
so no zone-33 `TileId` is ever built there. Orthophoto from the WMS `rp_dop20` (`geo4.service24.rlp.de`) in pieces of at
most 4000 px, because the tiles are lossless JPEG 2000 of 105 to 155 MB per 2 km block; the WMS is white beyond the state
border. The date comes from one GetFeatureInfo on `rp_dop20_info` at the centre: `erstellung`, the block's date of
creation, taken as the flight date and recorded with `flight_label = "Aktualität (Erstellung)"` (ADR 0038). DGM1 and DOM1
as 1 km GeoTIFF tiles from `geobasis-rlp.de` (Float32, LZW with predictor 2; the 2022 to 2024 DOM1 have no TFW, so the
GeoTIFF tags give the georeference). Their names carry the scan year and come from the `Laserbefliegung` layer of the
Blattschnitt WFS, with the Land Metalink for the tiles it gives no year; no name is guessed (ADR 0039). The mosaic of these
list-named tiles is shared with NRW. A listed tile that answers 404 is reported missing and asked for again after 30 days
(ADR 0040). nDSM = DOM1 − DGM1, from the same flight. ALKIS from the origin WFS `geo5.service24.rlp.de/wfs/alkis_rp.fcgi`,
not the GeoPortal.rlp registry proxy: `ave:GebaeudeBauwerk`, and `ave:Nutzung` as the land-use type; `parse_gml` reads
`aktualit` inside `gml:timePosition` (ADR 0041). OSM from `rheinland-pfalz-latest` (275 MB), which leaves out Wiesbaden
and Mainz-Kastel. At the Hesse border both ALKIS services and both extracts are read, and the rasters come from the state
of the centre; the LVermGeo RP height tiles reach across the Rhine, its WMS does not. Attribution
`©GeoBasis-DE / LVermGeoRP<year>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]` plus the dl-de/by-2-0 licence note (ADR 0042).

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
- *object 501 paved area:* ALKIS `nutzung` traffic classes. Verified live 2026-11-07 (Strausberg
  Vorstadt): object 501.5 for `Straßenverkehr` without `bez = Begleitfläche Straßenverkehr` and for
  `Platz` (with or without `bez = Parkplatz`); candidate 501.5 for `Weg` (sidewalks and lanes in town,
  tracks in the woods); verges not drawn. 501.5 has no border (a parcel boundary is no visible edge);
  heavy-traffic symbols are not used (ALKIS has no traffic volume). Unpaved OSM ways: track 505.2,
  path/footway/bridleway 505.1 (OOM's ISSprOM set splits 505 by footprint).
- *object 505:* OSM `highway=track|path|footway|bridleway` with an explicit unpaved `surface`
  (`unpaved, dirt, earth, ground, grass, gravel, fine_gravel, compacted, sand, woodchips`). Object 506 when
  additionally `width < 1.5` or `highway=path` with `trail_visibility` below good. 507 is not used.
- *candidate:* OSM paths with no `surface` tag.
- *dropped:* `footway=sidewalk|crossing` and paved OSM ways (ALKIS areas and the orthophoto cover them).

**Water (both map kinds, `build/water.rs`; ADR 0036):**
- *object 301.1 + 301.6:* ALKIS `nutzung` `Fließgewässer`, `Stehendes Gewässer`, `Hafenbecken` and `Meer`, any `bez`
  (`WATER_LAND_USES`). ALKIS has no depth, so ALKIS water is always uncrossable (301), never 302. Parts with a mean
  width (2·area/perimeter) of at least 2 m (`NARROW_WATER_M`, the ISSprOM 305 limit) are dissolved into one shape and
  clipped to the map area plus buffer. Each piece becomes one 301.1 area (holes = islands) and its rings become 301.6
  bank lines, minus the segments on the clip rectangle. The fill and the bank line are separate symbols because the
  combined 301 would draw a line at every parcel boundary and along the map edge. Slivers under 1 mm on paper (2 m on
  the ground at most) and gap holes of the same width are dropped. Tags (prefix `freemapper:`): `source=alkis-nutzung`,
  `source_id` = sorted oids joined by `;`, `nutzart` = the distinct `nutzart / bez`, `currency` = the newest
  `updated`.
- *candidate 301.1 + 301.6:* ALKIS water parts with a mean width under 2 m, dissolved among themselves.
- *candidate 301 (combined, OSM areas are not clipped):* `natural=water` (not `covered=yes`, not in a tunnel or culvert; `tunnel=no` is kept),
  `waterway=riverbank`, `landuse=reservoir`, `landuse=basin` (not `basin=retention|infiltration|detention`),
  `leisure=swimming_pool` (not indoor, covered, underground or on a roof), `amenity=fountain` areas. An OSM 301 area
  is dropped when at least 50 % of its part inside the clip window lies in the ALKIS water (`OSM_WATER_COVERED`,
  `drop_covered_water`).
- *candidate points and lines:* 311 `amenity=fountain` nodes and `man_made=water_well`; 312 `natural=spring`; 313
  `amenity=drinking_water` (schoolyard only); 305 `waterway=stream|drain` without `intermittent=yes`; 306
  `waterway=ditch` and intermittent streams and drains. No line in a tunnel or culvert; `tunnel=no` is kept.
- *candidate 501.5 bridge (sprint only):* `man_made=bridge` areas. ALKIS-vereinfacht `GebaeudeBauwerk` has no
  bridges. The bank line runs across the bridge; the mapper cuts it.
- *paint order* comes from the colour priorities, no object order: bank line 301.6 (black, 2) on top, then buildings
  521.3 (9) and paved areas 501.5 (14), then water 301.1 (16), then out of bounds 520 (22). On the schoolyard,
  water off the grounds stays inside the 520 candidates and is painted over them.
- *OCAD numbers:* 301.1 → 301001 and 301.6 → 301002 (the decimal rule would give 301100 and 301600, which OCAD's set
  lacks); 301, 305, 306, 311, 312 and 313 follow the rule (301000 … 313000). See `docs/ocad/ocd12-reference.md` §9.
- *dropped / out of scope:* OSM relations (multipolygon rivers; ALKIS covers them), river and canal centre lines,
  marsh 307–310 (`Moor`, `Sumpf`), `natural=coastline` (sea missing from ALKIS stays white).

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
As built in M8: 127.0.0.1 only, a random 128-bit token in every path, a Host check against DNS
rebinding, CSP; backgrounds OSM tiles and the LGB DOP20 WMS (`bebb_dop20c`); the page is vanilla JS
(Leaflet 1.9.4 vendored), German or English by browser language; saving an existing project changes
only its map area (an existing map keeps its georeferencing).

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
added; image templates with world files already work in OCAD. Berlin adapter (done 2026-11-07: §4.1), Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony and Rhineland-Palatinate (done 2026-11-08: §4.1; Thuringia blocked for now by its height-data host; next the remaining states of `docs/regions/states-survey.md`: Baden-Württemberg, Bremen, Bavaria, Saarland; user, 2026-11-07), then forest maps; then the neighbouring countries; Polish and Czech data for border maps (timing open), forest maps (ISOM, vegetation, cliffs via
karttapullautin), reconciliation, other regions, optional GDAL feature, legend generation, stricter quality
gates (ub-mode style).

## 7. Left to the builder

Internal module boundaries beyond §4, crate choices within the pure-Rust constraint, smoothing and
simplification defaults, the exact ALKIS attribute values in the rule file (verified against live data),
preview styling, and the web page design.
