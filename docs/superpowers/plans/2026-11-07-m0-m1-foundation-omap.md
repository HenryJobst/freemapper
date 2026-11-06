# M0 + M1: Foundation and OOM Writer – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper init` + `freemapper export` produce a georeferenced, empty 1:1000 schoolyard starter project (IOF school symbol set 2019) for the Clara-Zetkin-Schule that opens in OOM 0.9.6 without warnings.

**Architecture:** A Cargo workspace with the library crate `freemapper` (modules `crs`, `georef`, `omap`, `project`) and the binary crate `freemapper-cli` (binary name `freemapper`, clap). OOM files are written as XML with `quick-xml`. Colours and symbols come from OOM's embedded ISSprOM 2019 set, enlarged to 125 % at runtime for the school set.

**Tech Stack:** Rust 2024 (rustc 1.96), `quick-xml`, `clap` (derive), `serde` + `toml`, `geojson`, `chrono`, `world_magnetic_model` (WMM2025), `thiserror`, `anyhow` (CLI only).

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` (§3, §4.2, §4.3, §5, §6 M0/M1). Terms: `docs/glossary.md`.

## Global Constraints

- Source code entirely English. Identifiers follow the glossary "Code" column (`MapArea`, `MapObject`, `MapPart`, `Template`, `TemplateMap`, `SymbolSet`, `SymbolCode`, `Georeferencing`, `MapCoord`, `declination_deg`, `convergence_deg`, `grivation_deg`, `grid_scale_factor`, `scale_denominator`, `symbol_enlargement`, `MapKind::Schoolyard`, `MapKind::Sprint`).
- Pure Rust only: no crate that links GDAL/PROJ/PDAL/C libraries.
- Licence GPL-3.0-or-later.
- CRS: EPSG:25833 (ETRS89 / UTM 33N, GRS80, k0 = 0.9996, λ0 = 15°E, false easting 500000).
- OOM XML: namespace `http://openorienteering.org/apps/mapper/xml/v2`, `version="9"`. Coordinates are i32 in 1/1000 mm, y pointing south. Every ring ends with a repeat of its first point flagged 18. `coords count` must be exact.
- School set: all ISSprOM 2019 symbol dimensions × 1.25, map scale 1:1000.
- Gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` must pass before every commit.
- Commit after every task with the session attribution trailer.

## Review Focus

1. **Projected → map orientation sign** (grivation rotation and y-down): a point grid-north of the ref point must have negative y and, for grivation > 0, negative x. Pinned in Task 3.
2. **Combined scale factor**: OOM stores the *combined* factor in `grid_scale_factor`, and it must divide distances correctly (100 m on the ground = 100 mm at 1:1000 × factor). Pinned in Task 3.
3. **Enlargement must not touch non-dimension numbers** (colour indices, ids, counts, flags, angles, rotation, `dashes_in_group`, `mid_symbols_per_spot`, `pointed_cap_length`, pattern origin `<coord>`). Pinned in Task 6.
4. **Polygon ring closure** in written objects (last point == first point, flag 18), including input GeoJSON that is already closed. Pinned in Task 4.
5. **Non-ASCII text** (map notes with "©", "Daten geändert") must be written as valid UTF-8 XML with escaping. Pinned in Task 4.

---

### Task 1: Workspace, tooling, CI (M0)

**Files:**
- Create: `Cargo.toml` (workspace: members `crates/*`, `[workspace.package]` edition 2024 / license GPL-3.0-or-later / rust-version 1.85, `[workspace.lints.rust]` `unsafe_code = "forbid"`, `[workspace.lints.clippy]` `all = { level = "warn", priority = -1 }`)
- Create: `crates/freemapper/Cargo.toml`, `crates/freemapper/src/lib.rs`
- Create: `crates/freemapper-cli/Cargo.toml` (`[[bin]] name = "freemapper"`), `crates/freemapper-cli/src/main.rs`
- Create: `rustfmt.toml` (`edition = "2024"`, `max_width = 100`), `.gitignore` (`/target`, `*.omap.bak`), `LICENSE` (full GPL-3.0 text from https://www.gnu.org/licenses/gpl-3.0.txt)
- Create: `README.md` (German) and `README.en.md` (English): purpose, status, build, licence, attribution. They link each other and the glossary.
- Create: `lefthook.yml` (pre-commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`)
- Create: `.github/workflows/ci.yml` (matrix ubuntu-latest / macos-latest / windows-latest: fmt check, clippy, test)
- Create: `.claude/settings.json` with hooks: PostToolUse matcher `Edit|Write|MultiEdit` runs `rustfmt --edition 2024` on the edited file if it ends in `.rs` (read `tool_input.file_path` from the hook's stdin JSON); Stop hook runs `cargo clippy --workspace --all-targets -q -- -D warnings`.
- Test: `crates/freemapper-cli/tests/cli.rs`

- [ ] **Step 1: Failing test** `version_flag_prints_name`: run the binary (`env!("CARGO_BIN_EXE_freemapper")`) with `--version` and assert that stdout starts with `"freemapper "`.
- [ ] **Step 2:** `cargo test -p freemapper-cli` → FAIL (no binary/clap).
- [ ] **Step 3:** clap derive `Cli` with `#[command(version)]` and empty `Commands` enum placeholder; `lib.rs` has a crate doc comment.
- [ ] **Step 4:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check` → all pass.
- [ ] **Step 5:** Commit `chore: workspace, tooling, CI (M0)`.

### Task 2: `crs` – UTM 33 forward + grid convergence + point scale factor

**Files:** Create `crates/freemapper/src/crs.rs` (`pub mod crs;` in lib.rs). Tests are inline `#[cfg(test)]`.

**Interfaces – Produces:**
- `pub struct LatLon { pub lat: f64, pub lon: f64 }` (degrees, ETRS89 ≈ WGS84)
- `pub struct ProjectedPoint { pub easting: f64, pub northing: f64 }`
- `pub fn utm33_forward(p: LatLon) -> ProjectedPoint`
- `pub fn grid_convergence_deg(p: LatLon) -> f64` (OOM sign convention: negative west of 15°E)
- `pub fn point_scale_factor(p: LatLon) -> f64`
- `pub const EPSG_UTM33: u32 = 25833;`

- [ ] **Step 1: Failing tests** (`forward_reference_points`, `convergence_reference_points`, `scale_factor_reference_points`). Tolerances: 0.001 m, 1e-6°, 1e-8.

| lat | lon | easting | northing | convergence_deg | scale factor |
|---|---|---|---|---|---|
| 52.54024494927058 | 13.844335699781933 | 421622.9917 | 5821756.0323 | -0.917390 | 0.99967540 |
| 52.0 | 15.0 | 500000.0000 | 5761038.2125 | 0.0 | 0.99960000 |
| 53.0 | 12.0 | 298694.2000 | 5876481.1399 | -2.396705 | 1.00009740 |

- [ ] **Step 2:** `cargo test -p freemapper crs` → FAIL.
- [ ] **Step 3:** Implement the Krüger series to order n⁶ (Karney 2011, "Transverse Mercator with an accuracy of a few nanometers"). It gives easting/northing, γ = atan(τ'·tan λ / √(1+τ'²)) + atan2(q, p) and k from the same series terms. GRS80: a = 6378137, f = 1/298.257222101.
- [ ] **Step 4:** Tests pass. Gates pass.
- [ ] **Step 5:** Commit `feat(crs): UTM33 forward, grid convergence, scale factor`.

### Task 3: `georef` – Georeferencing, declination, projected ↔ map

**Files:** Create `crates/freemapper/src/georef.rs`.

**Interfaces:**
- Consumes: `crs::{LatLon, ProjectedPoint, utm33_forward, grid_convergence_deg, point_scale_factor, EPSG_UTM33}`
- Produces:
  - `pub fn magnetic_declination_deg(p: LatLon, date: chrono::NaiveDate) -> Result<f64, GeorefError>` (via `world_magnetic_model`; `GeorefError::DateOutsideModel` if the model rejects the date)
  - `pub struct Georeferencing { pub scale_denominator: u32, pub epsg: u32, pub ref_geographic: LatLon, pub ref_projected: ProjectedPoint, pub declination_deg: f64, pub convergence_deg: f64, pub grid_scale_factor: f64 }`
  - `impl Georeferencing { pub fn new(ref_point: LatLon, scale_denominator: u32, declination_date: NaiveDate) -> Result<Self, GeorefError>; pub fn grivation_deg(&self) -> f64; pub fn projected_to_map(&self, p: ProjectedPoint) -> MapCoord; pub fn map_to_projected(&self, c: MapCoord) -> ProjectedPoint }`
  - `MapCoord` is defined in Task 4 (`omap::MapCoord`). Implement Task 4's `MapCoord` struct first if needed, or define it here in `omap/coord.rs` and re-export it.

Rounding (written values, matching OOM's precision): `declination_deg` rounded to 0.01°, `grivation_deg() = round_0_01(declination_deg − convergence_deg)`, `grid_scale_factor` rounded to 1e-6. Auxiliary scale factor is 1.0 (combined = grid).

Transform (s = grid_scale_factor · scale_denominator / 1000 metres per map mm, g = grivation in radians, dE/dN relative to ref_projected):
`x_mm = (dE·cos g − dN·sin g) / s`, `y_mm = −(dE·sin g + dN·cos g) / s`, MapCoord = round(x_mm·1000), round(y_mm·1000).

- [ ] **Step 1: Failing tests**
  - `declination_strausberg_2026`: `magnetic_declination_deg(LatLon{52.5401, 13.8442}, 2026-11-07)` is in `4.0..7.0`.
  - `ref_point_maps_to_origin`: `projected_to_map(ref_projected) == MapCoord{x:0,y:0}`.
  - `north_is_up_without_grivation`: with a hand-built `Georeferencing` (declination = convergence = 0, factor 1.0, scale 1000), a point 100 m north → `MapCoord{x:0, y:-100_000}`; 50 m east → `{x:50_000, y:0}`.
  - `grivation_rotates_grid_north_left`: declination 5.0, convergence −1.0 (grivation 6.0) → point 100 m north gives `x < 0` and `y < 0`, and |x| ≈ 100_000·sin 6° (±2).
  - `combined_factor_scales`: factor 0.9996 → 100 m east gives x = round(100_000 / 0.9996).
  - `roundtrip_map_projected`: map_to_projected(projected_to_map(p)) is within 0.001 m of p for 3 points.
- [ ] **Step 2:** FAIL. **Step 3:** Implement. **Step 4:** PASS + gates. **Step 5:** Commit `feat(georef): georeferencing with WMM declination and map transform`.

### Task 4: `omap` – document model and XML writer

**Files:** Create `crates/freemapper/src/omap/mod.rs`, `omap/coord.rs`, `omap/model.rs`, `omap/writer.rs`; golden test file `crates/freemapper/tests/golden/minimal.xmap`.

**Interfaces – Produces:**
- `pub struct MapCoord { pub x: i32, pub y: i32, pub flags: u8 }` + `pub const FLAG_CURVE_START: u8 = 1, FLAG_CLOSE: u8 = 2, FLAG_HOLE: u8 = 16, FLAG_DASH: u8 = 32`
- `pub enum MapObjectKind { Point { rotation_rad: f64 }, Path }`
- `pub struct MapObject { pub symbol_id: u32, pub kind: MapObjectKind, pub coords: Vec<MapCoord>, pub tags: Vec<(String, String)> }`
- `pub fn closed_ring(points: &[(i32, i32)]) -> Vec<MapCoord>`: drops a duplicate closing input point, then appends the first point with flags 18. Multiple rings are concatenated by the caller.
- `pub struct MapPart { pub name: String, pub objects: Vec<MapObject> }`
- `pub enum TemplateKind { Gdal, Ogr, Map }` → XML `type` `GdalTemplate` / `OgrTemplate` / `TemplateMap`
- `pub struct TemplateRef { pub kind: TemplateKind, pub relpath: String, pub georeferenced: bool, pub crs_spec: Option<String>, pub visible: bool, pub opacity: f32 }`
- `pub struct MapDocument { pub notes: String, pub georef: Georeferencing, pub symbol_set: SymbolSet /* Task 5 */, pub parts: Vec<MapPart>, pub templates: Vec<TemplateRef> }`
- `pub enum XmlFormat { Compact, Pretty }`; `pub fn write_map(doc: &MapDocument, out: impl std::io::Write, format: XmlFormat) -> Result<(), OmapError>`

Writer layout (exact order): `<?xml version="1.0" encoding="UTF-8"?>`, `<map xmlns=… version="9">`, `<notes>`, `<georeferencing scale=… grid_scale_factor=… auxiliary_scale_factor="1" declination=… grivation=…>` with `<projected_crs id="EPSG"><spec language="PROJ.4">+init=epsg:25833</spec><parameter>25833</parameter><ref_point x=E y=N/></projected_crs>` and `<geographic_crs id="Geographic coordinates"><spec language="PROJ.4">+proj=latlong +datum=WGS84</spec><ref_point_deg lat=… lon=…/></geographic_crs>`, then the symbol set's raw `<colors>…</colors>` verbatim, `<barrier version="6" required="0.6.0">`, the raw `<symbols>…</symbols>`, `<parts count current="0">` (at least one part; an empty doc gets `default part`), `<templates count first_front_template="0">…<defaults use_meters_per_pixel="true" meters_per_pixel="0" dpi="0" scale="0"/></templates>`, `<view><grid …copied from symbol set file…/><map_view zoom="1" position_x="0" position_y="0"><map opacity="1" visible="true"/><templates count=N><ref template=i visible="true" opacity=…/>…</templates></map_view></view>`, `</barrier>`, `</map>`. Compact coords: `x y;` or `x y flags;` when flags ≠ 0. Pretty format writes `<coord x y flags/>` elements. Object XML: `<object type="0|1" symbol=id [rotation]>` + optional `<tags><t k=…>…</t></tags>` + `<coords count=N>` + (path only) `<pattern rotation="0"><coord x="0" y="0"/></pattern>`.

- [ ] **Step 1: Failing tests**
  - `closed_ring_appends_flagged_first_point` and `closed_ring_dedups_closed_input` (Review Focus 4).
  - `golden_minimal_pretty`: a doc with a test `SymbolSet` (1 colour, 0 symbols), one `MapPart` with one 4-point area object, and tags `freemapper:source=test`. Pretty output equals `tests/golden/minimal.xmap` byte for byte (create the golden file by hand from the layout above).
  - `notes_are_utf8_escaped`: notes `"© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert <x>"` → output contains `© GeoBasis-DE/LGB 2026` and `&lt;x&gt;`, and the output parses with `quick-xml` (Review Focus 5).
  - `template_refs_visible_in_view`: one `TemplateMap` ref → output contains `type="TemplateMap"`, `relpath="templates/candidates.omap"`, `georef="true"` and `<ref template="0" visible="true"`.
- [ ] **Step 2:** FAIL. **Step 3:** Implement with `quick_xml::Writer` (raw fragments written via `write_event(Event::Text(BytesText::from_escaped(raw)))` or equivalent). **Step 4:** PASS + gates. **Step 5:** Commit `feat(omap): document model and XML writer`.

### Task 5: `omap::symbol_set` – embedded ISSprOM 2019 and code → id lookup

**Files:**
- Create: `assets/symbol-sets/ISSprOM 2019_4000.omap`, downloaded from `https://raw.githubusercontent.com/OpenOrienteering/mapper/master/symbol%20sets/4000/ISSprOM%202019_4000.omap`
- Create: `assets/symbol-sets/README.md` with source URL, OOM commit/date, GPL-3.0-or-later, and IOF attribution for the specification
- Create: `crates/freemapper/src/omap/symbol_set.rs`

**Interfaces – Produces:**
- `pub struct SymbolSet { pub name: String /* symbols@id */, pub scale_denominator: u32, pub colors_xml: String, pub symbols_xml: String, pub grid_xml: String, codes: Vec<(SymbolCode, u32)> }`
- `pub struct SymbolCode(pub String)`
- `impl SymbolSet { pub fn from_omap_xml(xml: &str) -> Result<Self, OmapError>; pub fn issprom_2019() -> Self /* include_str! */; pub fn id_for_code(&self, code: &str) -> Option<u32> /* first top-level symbol with that code */; pub fn symbol_count(&self) -> usize; pub fn max_id(&self) -> u32 }`

- [ ] **Step 1: Failing tests** `issprom_codes`: `id_for_code("101")==Some(0)`, `"102"→2`, `"103"→5`, `"501"→104`, `"501.5"→105`, `"501.1"→108`, `"520"→139`, `"521"→141`, `"522"→145`; `symbol_count()==185`; `max_id()==184`; `name=="ISSprOM 2019"`; `colors_xml` starts with `<colors count="32">`.
- [ ] **Step 2:** FAIL. **Step 3:** Parse with `quick-xml`. Capture the raw byte ranges of `<colors>…</colors>`, `<symbols>…</symbols>` and `<grid …/>`, and only top-level `<symbol>` children of `<symbols>` (nested private symbols have no `id`). **Step 4:** PASS + gates. **Step 5:** Commit `feat(omap): embedded ISSprOM 2019 symbol set`.

### Task 6: `omap::enlarge` – symbol enlargement (OOM-compatible)

**Files:** Create `crates/freemapper/src/omap/enlarge.rs`.

**Interfaces – Produces:** `pub fn enlarge_symbols(symbols_xml: &str, factor: f64) -> Result<String, OmapError>`. It rewrites XML events and leaves everything not listed below byte-identical in value.

Scale rules, exactly matching OOM `Symbol::scale` (`src/core/symbols/*.cpp`), with `r(v) = (v·factor).round()` (half away from zero, as `qRound`):
- `line_symbol`: `line_width, minimum_length, start_offset, end_offset, segment_length, end_length, minimum_mid_symbol_count, minimum_mid_symbol_count_when_closed, dash_length, break_length, in_group_break_length, mid_symbol_distance`. Not `pointed_cap_length`. OOM scales the two "count" fields too, so we do as well.
- `border` (inside line_symbol): `width, shift, dash_length, break_length`
- `point_symbol`: `inner_radius, outer_width`. Element objects (`element > object > coords`): x and y of every coordinate; flags unchanged.
- `area_symbol`: `min_area` scaled by factor² (`(v·f·f).round()`)
- `pattern` inside `area_symbol`: `line_spacing, line_width, line_offset, offset_along_line, point_distance` (not `angle`, `type`, `color`, `rotation`)
- `text_symbol` → `font`: `size = max(10, r(size))`; `framing`: `line_half_width, shadow_x_offset, shadow_y_offset`
- Nested `<symbol>` elements (mid/start/end/dash symbols, point elements, private combined parts) follow the same rules. Public combined parts (`<part symbol="N"/>`) are references and are left alone.
- Not touched: `object > pattern > coord` (pattern origin), all ids, colour indices, counts of elements/parts/patterns, `code`, `name`, descriptions.

- [ ] **Step 1: Failing tests** (factor 1.25):
  - `line_attributes_scaled`: `line_width="210"`→`"263"`, `dash_length="4000"`→`"5000"`; `dashes_in_group="1"`, `color="8"`, `pointed_cap_length="750"` unchanged.
  - `border_scaled`: `width="100" shift="50"` → `"125" "63"`.
  - `point_element_coords_scaled`: `<coords count="2">0 0;0 -750;</coords>` → `0 0;0 -938;`; flags kept (`-12797 -557 1;` → `-15996 -696 1;`); `<pattern rotation="0"><coord x="0" y="0"/>` unchanged.
  - `area_min_area_quadratic`: `min_area="500"` → `"781"`.
  - `font_size_min_10`: `size="4"` → `"10"`; `size="2117"` → `"2646"`.
  - `issprom_full_set_roundtrips`: enlarging `SymbolSet::issprom_2019().symbols_xml` by 1.0 returns XML in which every attribute value equals the input's (compare via parsed attribute lists) (Review Focus 3).
- [ ] **Step 2:** FAIL. **Step 3:** Implement as a streaming `quick-xml` reader→writer that tracks an element stack to decide context. **Step 4:** PASS + gates. **Step 5:** Commit `feat(omap): OOM-compatible symbol enlargement`.

### Task 7: School symbol set 2019 (1:1000)

**Files:**
- Create: `assets/symbol-sets/school-2019-additions.xml`: 6 top-level `<symbol>` elements in the OOM XML dialect, ids filled in at runtime
- Modify: `crates/freemapper/src/omap/symbol_set.rs` (add `pub fn school_2019() -> Self`)

Build: `enlarge_symbols(issprom.symbols_xml, 1.25)`, then patch, then append the additions with ids `max_id+1..` (185–190), and update `<symbols count>` to 191. Keep `id="ISSprOM 2019"` so OOM's ISSprOM CRT files still apply. `scale_denominator = 1000`.
- Patch 501 (IOF school set "Extended definition"): in the combined symbols with code `501` (id 104) and `501.10` (id 113), set the private border line (code `501.1`) `line_width` to the enlarged `line_width` of the top-level symbol with code `505`. Fill colours stay lower brown 30 % (colour 14) and 50 % (colour 13).
- Additions. Sizes are in mm on the map as read from the IOF PDF (`https://old.orienteering.sport/wp-content/uploads/2019/02/Symbol-set-for-school-orienteering-maps-2019-1.pdf`) and are **not** enlarged again. 1 mm = 1000 units. Colours: black = 2, blue = 7.

| code | name | type | geometry |
|---|---|---|---|
| 321 | Hydrant | point | blue ring outer Ø 1.3 (line 0.13) + blue dot Ø 0.5 |
| 541 | Playground equipment | line | black line 0.12; start + end symbols: black ring Ø 0.6, line 0.18; minimum_length 1800 |
| 542 | Table | point | black square 1.5 × 1.5 outline 0.22 with both diagonals 0.12 |
| 543 | Bench, seat | line | black line 0.22; start + end symbols: perpendicular tick 0.8 long, 0.18 wide; minimum_length 1500 |
| 544 | Pole, small pylon | point | black ring Ø 1.1, line 0.18 |
| 545 | Lamppost | point | black ring Ø 1.4, line 0.18 + "×" of 2 lines 0.6 long, 0.12 wide; `is_rotatable="false"` (north-oriented) |

- [ ] **Step 1: Failing tests** `school_set`: `symbol_count()==191`; `scale_denominator==1000`; `id_for_code("545")==Some(190)` (ids in table order from 185); the line_symbol for code 101 has `line_width` = round(ISSprOM 101 width × 1.25); the 501.1 private border in id 104 has `line_width` == enlarged 505 width; output parses with quick-xml.
- [ ] **Step 2:** FAIL. **Step 3:** Implement. Write the additions XML by hand, modelled on existing ISSprOM point/line symbols (e.g. 531, 532, 512). **Step 4:** PASS + gates. **Step 5:** Commit `feat(omap): IOF school symbol set 2019 for 1:1000`.

### Task 8: Project file, map area, `init` + `export` CLI

**Files:**
- Create: `crates/freemapper/src/project.rs`, `crates/freemapper/src/area.rs`
- Create: `tests/fixtures/clara-zetkin-schule.geojson`: OSM way 992423119 as a GeoJSON Feature with a Polygon in EPSG:4326. Build it once from `https://api.openstreetmap.org/api/0.6/way/992423119/full.json` (node lat/lon in way order) and set `properties.source = "© OpenStreetMap contributors, ODbL"`.
- Modify: `crates/freemapper-cli/src/main.rs`; Test: `crates/freemapper-cli/tests/cli.rs`

**Interfaces – Produces:**
- `pub enum MapKind { Schoolyard, Sprint }` with `default_scale()` (1000 / 4000) and `symbol_set()` (`school_2019` / `issprom_2019`); serde lowercase names
- `pub struct MapArea { pub polygon: geo_types::Polygon<f64> /* lon/lat */ }` with `fn from_geojson(s: &str) -> Result<Self, AreaError>` (first Polygon feature/geometry) and `fn centroid(&self) -> LatLon` (area centroid, `geo` crate)
- `pub struct ProjectFile { pub map_kind: MapKind, pub scale_denominator: u32, pub declination_date: NaiveDate, pub buffer_m: f64 /* 20 schoolyard, 50 sprint */, pub map_area: serde_json::Value /* GeoJSON geometry */ }` with `load(dir)` and `save(dir)` to `freemapper.toml`
- `pub fn export_starter_project(dir: &Path) -> Result<PathBuf, ProjectError>`. It writes:
  - `<dir-name>.omap`: georeferencing at the map-area centroid; symbol set by kind; one empty part `"default part"`; a visible `TemplateMap` ref `templates/candidates.omap` (opacity 0.7); notes = attribution text (`© OpenStreetMap contributors, ODbL` for the map area)
  - `templates/candidates.omap`: same georeferencing and symbol set, part `"candidates"`, with the map-area outline as a closed path object using the symbol with code `501.1`, tagged `freemapper:source=osm`, `freemapper:source_id=way/992423119` if known, `freemapper:currency=<date>`
  - `ATTRIBUTION.txt`
- CLI: `freemapper init <DIR> --geojson <FILE> [--kind schoolyard|sprint] [--scale N]` (`declination_date` = today) and `freemapper export <DIR>`

- [ ] **Step 1: Failing tests**
  - `centroid_of_fixture`: centroid of the fixture is within 0.0005° of (52.54008, 13.84419).
  - `project_file_roundtrip`: save then load gives an equal `ProjectFile`.
  - CLI `init_then_export_creates_starter_project` (tempdir): exit 0; files `clara-zetkin-schule.omap`, `templates/candidates.omap`, `ATTRIBUTION.txt`, `freemapper.toml` exist; the omap contains `scale="1000"`, `<parameter>25833</parameter>`, `<symbols count="191"` and `type="TemplateMap"`.
  - `outline_vertices_map_inside_paper`: every outline coord satisfies |x|, |y| < 200_000 (±200 mm at 1:1000, i.e. the ±140 m extent fits).
- [ ] **Step 2:** FAIL. **Step 3:** Implement. **Step 4:** PASS + gates. **Step 5:** Commit `feat: init/export of empty starter project (M1)`.

### Task 9: Acceptance in OOM 0.9.6 + docs

- [ ] **Step 1:** `cargo run -p freemapper-cli -- init /tmp/…/clara-zetkin-schule --geojson tests/fixtures/clara-zetkin-schule.geojson --kind schoolyard && cargo run -p freemapper-cli -- export …` (use the scratchpad dir).
- [ ] **Step 2:** `open -a Mapper <omap>` (OOM 0.9.6 is installed). Checklist for the user: the file opens without warning dialogs; Map → Georeferencing shows EPSG 25833, scale 1:1000 and declination ≈ 5–6°; the candidates template is visible as an outline; hovering an outline corner shows projected coordinates matching the fixture (≈ E 4216xx, N 58217xx); symbol sizes are 125 % of ISSprOM.
- [ ] **Step 3:** Update `README.md` / `README.en.md`: build + `init`/`export` usage and status M1.
- [ ] **Step 4:** Commit `docs: M1 usage`.
