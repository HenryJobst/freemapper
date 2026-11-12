# Park terrain implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sprint and schoolyard maps with park parts get three new aids from the DGM1: a local relief template, candidates for small knolls and depressions, and candidates for earth banks and cliffs (height jumps).

**Architecture:** Two new derive modules work on the terrain raster that `derive_project` already holds: `derive/relief.rs` writes `templates/relief.png`, `derive/landforms.rs` writes `data/landforms.geojson`. The export reads the GeoJSON into ISSprOM candidate objects (109, 110, 111, 104, 201) in `project/export.rs`, beside the contours, and drops those on buildings, in water and next to walls, fences and banks already drawn. Ideas come from karttapullautin (GPL-3.0) and omapmaker (ideas only, licence not GPL-compatible); no code is copied.

**Tech Stack:** Rust, `geo`, existing `Raster<f32>`, `contour_lines`, `gaussian_smooth`, `write_gray_png`.

**Spec:** this plan (research in the session of 2026-11-11/12; memory note "park terrain ideas"). Spec §4.4 (contours are candidates) and §4.5 (candidates map) apply.

## Global Constraints

- Source code and docs English, terms from `docs/glossary.md`; new terms get glossary rows (DE/EN).
- Everything new is a candidate or a template; nothing enters the map itself.
- Thresholds on paper are in mm × `scale_denominator / 1000`; heights per map kind: schoolyard 0.5 m, sprint 1.0 m.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` green; golden net updated where outputs change.
- ADR, changelog (DE+EN), manual (DE+EN), AGENTS.md, README template lists updated.

## Review Focus

1. A DTM with `NaN` cells (no data at the area edge, water): no candidates from or next to no-data cells.
2. Flat noisy terrain (DGM1 noise of a few cm): no knolls, no banks.
3. A wide hill already shown by contours: no knoll candidate (size limit).
4. A plain steep slope (a hillside, not a jump): no bank (elongation and drop limits).
5. Banks: the ticks point down (lower side on the right of the line direction).

---

### Task 1: Local relief template

**Files:** Create `crates/freemapper/src/derive/relief.rs`; modify `derive/mod.rs`, `project.rs` (`RELIEF_RELPATH`, `MapKind::relief_sigma_m`, call in `derive_project_cancellable`), `project/export.rs` (template list), `layers.rs` (`RASTER_LAYERS`), CLI `mcp.rs` + i18n parameter text.

**Interfaces:** `pub fn local_relief(dtm: &Raster<f32>, sigma_m: f64) -> Raster<Option<f32>>` (DTM minus its Gaussian mean, `None` on `NaN`); `pub fn relief_shade(relief: &Raster<Option<f32>>, span_m: f32) -> Raster<Option<u8>>` (128 at 0, 255 at `+span_m` and above, 0 at `-span_m` and below). `MapKind::relief_sigma_m`: schoolyard 3.0, sprint 5.0. Span 1.0 m for both.

- [ ] Tests: `local_relief_of_a_plane_is_zero`, `a_bump_is_positive_and_a_pit_negative`, `relief_shade_maps_span_to_full_range`, `nan_stays_none`; layer test lists `relief`.
- [ ] Implement, wire into derive (after slope), export template list (`relief.png`, hidden, opacity 0.5), `RASTER_LAYERS` ("relief", "local relief: banks, knolls, hollows and ditches").
- [ ] Commit `feat(derive): local relief template`.

### Task 2: Knolls and depressions

**Files:** Create `crates/freemapper/src/derive/landforms.rs`.

**Interfaces:** `pub enum LandformKind { Knoll, ElongatedKnoll, Depression, Bank, Cliff }` (GeoJSON `kind`: `knoll`, `elongated-knoll`, `depression`, `bank`, `cliff`); `pub struct Landform { pub kind: LandformKind, pub points: Vec<(f64, f64)>, pub rotation_deg: f64, pub height_m: f64 }`; `pub fn knolls_and_depressions(dtm: &Raster<f32>, scale_denominator: u32, min_height_m: f64) -> Vec<Landform>`; `landforms_to_geojson`, `landforms_from_geojson`.

Algorithm: smooth with σ 1 m; every strict local maximum (minimum for depressions, on the negated raster) of valid cells gets a window of half-size `max_diameter + 2` cells; `contour_lines` at 0.25 m in the window; the lowest closed ring around the extremum whose area is at most `π (d/2)²` with `d` = 5 mm on paper and which keeps 2 cells away from no-data cells; height = extremum minus ring height ≥ `min_height_m`. Greedy by height, an extremum inside an accepted ring of the same kind is dropped. Elongated when the ring's principal axes have an aspect ≥ 1.5 (knolls only); rotation = major axis, degrees counter-clockwise from east.

- [ ] Tests: `a_two_metre_bump_is_a_knoll`, `a_low_bump_is_no_knoll`, `a_wide_hill_is_no_knoll`, `an_elongated_bump_has_its_rotation`, `a_pit_is_a_depression`, `two_bumps_are_two_knolls`, `noise_gives_nothing`, `geojson_round_trip`.
- [ ] Commit `feat(derive): knoll and depression candidates from the terrain`.

### Task 3: Height jumps

**Files:** `crates/freemapper/src/derive/landforms.rs`.

**Interfaces:** `pub fn height_jumps(dtm: &Raster<f32>, scale_denominator: u32, min_height_m: f64) -> Vec<Landform>`.

Algorithm: gradient by central differences; steep cells have a gradient ≥ 0.8 (about 39°). 8-connected components; drop per cell = height 2 m uphill minus 2 m downhill along the gradient; component drop = median ≥ `min_height_m`; principal axis length ≥ max(2 mm on paper, 4 m) and ≥ 2 × the mean width (cells / length). Centre line: cells binned per pixel along the axis, mean position per bin, simplified at 0.5 m. Direction: the lower side lies on the right. Drop ≥ 3 m → `Cliff`, else `Bank`.

- [ ] Tests: `a_step_is_a_bank_with_the_low_side_right`, `a_high_step_is_a_cliff`, `a_gentle_slope_gives_nothing`, `a_wide_steep_hillside_gives_nothing`, `nan_cells_give_nothing`.
- [ ] Commit `feat(derive): bank and cliff candidates from height jumps`.

### Task 4: Derive and export

**Files:** `project.rs` (`LANDFORMS_RELPATH = "data/landforms.geojson"`, `MapKind::landform_min_height_m`, derive call), `project/export.rs` (`stored_landform_objects`).

Export: point symbols 109/110/111 (110 with rotation), line symbols 104/201; tags `freemapper:source`, `freemapper:height` (rounded to 0.1 m), `freemapper:currency`. Dropped: outside map area + buffer, within 2 m of a building footprint, inside the drawn water, lines within 2 m of a wall, fence or bank already among the map or candidate objects (codes 104, 104.1, 201, 513.1, 513.2, 515, 518); missing file → none.

- [ ] Tests: `derive_writes_landforms`, `export_adds_landform_candidates` (hand-written landforms GeoJSON → 109, 110, 111, 104 in `candidates.omap`, none on a building).
- [ ] Golden net: run `tests/golden-guard.sh`, re-record what the new files change.
- [ ] Commit `feat(export): terrain candidates in the candidates map`.

### Task 5: Docs

- [ ] Glossary rows: local relief (lokales Relief), landform candidate (Geländeform-Kandidat), height jump (Höhensprung).
- [ ] ADR 0104 "Terrain candidates from the DGM1"; changelog DE/EN; manual DE/EN (templates table, candidates section); AGENTS.md and README template lists; MCP layer text.
- [ ] unslop + humanizer over user texts; commit `docs: terrain candidates and local relief`.
