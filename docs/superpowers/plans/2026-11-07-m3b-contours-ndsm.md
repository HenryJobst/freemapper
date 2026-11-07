# M3b: Contours and nDSM – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper run` adds contour lines as candidates (101/102 in `templates/candidates.omap`) and an
nDSM template (`templates/ndsm.png`, heights of hedges, walls, trees, playground equipment above
ground) behind the map.

**Architecture:** Contours: `derive` smooths the DTM mosaic it already builds (Gaussian, σ in metres),
traces iso-lines with the `contour` crate, simplifies them and stores them as GeoJSON in EPSG:25833
(`data/contours.geojson`); `export` turns them into 101/102 objects in the candidates part. nDSM: LGB
publishes one LAZ point cloud per tile (`ndom/NDOM_33EEE-NNNN_LAS12.laz`, 25 million points on a 0.2 m
grid, z = height above ground). `fetch` caches the tiles keyed by the server's `Last-Modified` date (no
currency CSV exists); `derive` rasterizes the points over map area + buffer and writes a coloured PNG
with world file.

**Tech Stack:** Rust 2024; new crates `contour = "0.13.1"`, `las = { version = "=0.11.1", features = ["laz"] }`
(pure Rust; 0.11.2 is younger than two weeks).

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1 (nDOM), §4.4 (contours: smooth,
marching squares, simplify, clip; interval schoolyard 1 m / index 5 m, sprint 2 m / index 10 m;
contours are candidates only; nDSM max Z per 0.2 m cell with a height colour ramp).

## Global Constraints

- nDSM URL `https://data.geobasis-bb.de/geobasis/daten/ndom/NDOM_<tile name>_LAS12.laz` (≈ 15 MB,
  LAS 1.2, point format 0, scale 0.01, no CRS VLR → EPSG:25833 assumed; points at pixel centres,
  `x = 421000.1, 421000.3, …`). Cache `<cache>/lgb/ndom/<tile>_<Last-Modified date>.laz`.
- nDSM colours (RGBA): below 0.3 m transparent; 0.3–1 m `#f2d14b` (low: kerbs, benches, low walls);
  1–2.5 m `#f08a24` (hedges, walls, playground equipment, cars); 2.5–8 m `#4aa35a` (bushes, sheds,
  small trees); above 8 m `#1f6b3a` (trees, buildings). Template behind the map, hidden, opacity 0.6.
- Contours: σ = 1.5 m (schoolyard) / 2.0 m (sprint) Gaussian on the 1 m DTM; levels at multiples of
  the interval; index every 5th level (102), others 101; simplify with RDP at 0.1 mm on paper; drop
  lines shorter than 2 mm on paper. Tags: `freemapper:source = lgb-dgm1`, `freemapper:height = <m>`,
  `freemapper:currency = <newest DTM currency>`.
- Map kind gains `contour_sigma_m()`; `contour_interval_m()` exists.

## Review Focus

1. **Flat school grounds** – a DTM spanning less than one interval gives no contours, no error (Task 1).
2. **Nodata cells in the DTM** – no false contours along nodata (Task 1).
3. **Map area across tile edges** – contours and nDSM mosaic seamless (Tasks 1, 3).
4. **nDSM tile not published** (404) – fetch reports the tile, the rest of the project still works (Task 2).
5. **Projects fetched before M3b** – export works without `data/contours.geojson` and `ndsm.png` (Task 4).

---

### Task 1: Contour lines from the DTM

**Files:** create `crates/freemapper/src/derive/contours.rs`; modify `derive/mod.rs`, manifests.

**Interfaces:**
- `contours::gaussian_smooth(dtm: &Raster<f32>, sigma_m: f64) -> Raster<f32>` (NaN cells stay NaN and
  do not spread into neighbours: normalized convolution over valid cells).
- `contours::ContourLine { height: f64, points: Vec<(f64, f64)> /* EPSG:25833 */ }`.
- `contours::contour_lines(dtm: &Raster<f32>, interval_m: f64) -> Vec<ContourLine>` (levels between the
  DTM's min and max; cells next to NaN excluded by tracing a field where NaN is replaced per level and
  dropping segments that touch nodata).
- `contours::contours_to_geojson(&[ContourLine]) -> String`, `contours_from_geojson(&str) -> Result<Vec<ContourLine>, ContourError>`.
- [ ] Tests: `plane_gives_straight_contours` (z = e/10 on 50 × 50 px → levels 1…4 m, each a straight
  north–south line at e = 10, 20, 30, 40 m ± 0.01), `flat_dtm_gives_no_contours`,
  `smoothing_keeps_a_plane`, `nodata_does_not_create_contours`, `geojson_roundtrip`.
- [ ] Commit `feat(derive): contour lines from the DTM`.

### Task 2: nDSM tiles

**Files:** `crates/freemapper/src/sources/lgb.rs`, `crates/freemapper/src/fetch.rs`.

**Interfaces:**
- `lgb::ndsm_tile_url(t: TileId) -> String`.
- `Fetcher::last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError>` (default `None`;
  `HttpFetcher`: HEAD request, `Last-Modified` header); test fetcher: a map of dates.
- `fetch::fetch_ndsm_tiles(fetcher, cache, tiles, mode, progress) -> Result<Vec<NdsmTile { tile, url, published: Option<NaiveDate>, path }>, FetchError>`;
  a tile answered with 404 is reported (`"warning: no nDSM for tile …"`) and skipped.
- [ ] Tests: `ndsm_url_matches_lgb_layout`, `ndsm_downloaded_then_reused`, `ndsm_republished_replaces_old`,
  `missing_ndsm_tile_is_skipped_with_warning`, `offline_uses_cached_ndsm`, `not_a_las_file_is_rejected`.
- [ ] Commit `feat(fetch): nDSM point cloud tiles`.

### Task 3: nDSM template

**Files:** create `crates/freemapper/src/derive/ndsm.rs`.

**Interfaces:**
- `ndsm::rasterize(tiles: &[PathBuf], bbox: &BoundingBox) -> Result<Raster<f32>, DeriveError>` (0.2 m
  grid covering `bbox`; max z per cell; cells without points NaN).
- `ndsm::colour(height: f32) -> [u8; 4]`; `ndsm::write_ndsm_png(r: &Raster<f32>, out: &Path)` (+ `.pgw`).
- [ ] Tests (LAZ written with `las::Writer` in the test): `rasterize_keeps_max_height`,
  `points_outside_bbox_are_ignored`, `two_tiles_are_seamless`, `colour_ramp`, `png_has_world_file`.
- [ ] Commit `feat(derive): nDSM template`.

### Task 4: Project integration

**Files:** `crates/freemapper/src/project.rs`.

**Interfaces:** `ProjectFile.ndsm_tiles: Vec<NdsmRecord { tile, url, published: Option<NaiveDate> }>`;
`fetch_project` fetches nDSM tiles; `derive_project` writes `templates/ndsm.png` (when nDSM tiles exist)
and `data/contours.geojson`; `export` references `ndsm.png` (behind, hidden, 0.6) and adds contour
candidates. Attribution unchanged (LGB line covers nDOM).
- [ ] Tests: `derive_writes_contours_and_ndsm`, `export_adds_contour_candidates`,
  `export_references_ndsm_template`, `export_without_contours_still_works`.
- [ ] Commit `feat: contours and nDSM in derive and export`.

### Task 5: Docs, live run, acceptance

- [ ] README DE/EN, spec §4.2 (`ndsm.png`, `data/contours.geojson`), glossary (`Höhenlinie`,
  `Zählkurve` if missing). Live test: real nDSM tile for 33421-5821; sample regenerated; overlay
  checked visually. Commit `docs: M3b contours and nDSM`.
