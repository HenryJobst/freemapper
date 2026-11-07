# M4b: Peripheral Information – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The exported map carries its peripheral information as map objects: magnetic north lines over
the map content and, below it, title, scale, contour interval, bar scale, attribution and the freemapper
credit line — the first usable schoolyard map.

**Architecture:** The OOM writer learns text objects. Four sheet symbols (three text sizes, one bar scale
line) are appended to both symbol sets as a fixed XML asset, after the school enlargement so their sizes
are absolute. A layout module computes, in map (paper) coordinates, the north lines across the content
rectangle (map area + buffer projected to paper) and an info block below it. The export writes them into
a part `peripheral information`, next to the buildings in `default part`.

**Tech Stack:** Rust 2024, existing crates only.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.3 (peripheral information),
§4.1 (attribution), §4.4 (contour intervals).

## Global Constraints

- Source code English; generated map text German (Brandenburg first): `Maßstab 1:{scale}`,
  `Äquidistanz {interval} m`; attribution lines and the credit line verbatim from the project.
- OOM text objects: `<object type="4" symbol="N" h_align="H" v_align="V"><coords count="1">x y;</coords><text>…</text></object>`
  (`h_align` 0 left / 1 centre / 2 right, `v_align` 0 baseline / 1 top / 2 centre / 3 bottom); tags come
  before `<coords>` as for other objects. Text symbols:
  `<text_symbol icon_text="…"><font family="Arial" size="S" [bold="true"]/><text color="2" line_spacing="1" paragraph_spacing="0" character_spacing="0" kerning="true"/></text_symbol>`
  (`S` in 1/1000 mm).
- Sheet symbols (codes, names, sizes on paper, colour 2 = Black 100%): `991` Map title (Arial bold 5 mm),
  `992` Map text (Arial 2.5 mm), `993` Map small text (Arial 1.8 mm), `994` Bar scale line (line width
  0.25 mm). Appended after all existing symbols; existing ids stay unchanged (school 321/541–545 keep
  185–190).
- Magnetic north lines: ISSprOM 601 (black), spacing 30 mm × symbol enlargement (sprint 30 mm,
  schoolyard 37.5 mm), vertical in map coordinates (the map is oriented to magnetic north), one line
  through the map reference point (x = 0), covering the content rectangle.
- Contour interval: schoolyard 1 m, sprint 2 m (spec §4.4).
- Bar scale: 50 mm on paper with ticks at 0, ½ and 1, labelled with ground metres (`0`, `25`, `50 m` at
  1:1000; `0`, `100`, `200 m` at 1:4000).
- The red bar on the north edge listed in spec §4.3 is **not** implemented: ISSprOM has no red colour and
  the bar has no norm source; north lines carry the orientation (ruling, see Task 2).

## Review Focus

1. **Text with XML special characters** (`&`, `<`, quotes in a title) – escaped, the file still parses
   (Task 1: `text_object_escapes_content`).
2. **Very small or very large map areas** – north lines and info block still cover / sit below the
   content (Task 2: `north_lines_cover_content`, `info_block_sits_below_content`).
3. **Projects created before M4b** (no `title` in `freemapper.toml`) – a title derived from the folder
   name (Task 3: `project_without_title_loads`).
4. **Sprint vs schoolyard** – spacing, interval and bar labels follow the map kind (Task 2 tests for
   both kinds).
5. **Re-export** – peripheral objects are not duplicated, there is exactly one part of that name (Task 3:
   `export_has_one_peripheral_part`).

---

### Task 1: Text objects

**Files:** `crates/freemapper/src/omap/model.rs`, `crates/freemapper/src/omap/writer.rs`,
`crates/freemapper/tests/omap_writer.rs`.

**Interfaces:** `MapObjectKind::Text { text: String, h_align: u8, v_align: u8 }`; the writer writes type 4
as in Global Constraints. `MapObjectKind` loses `Copy`-like uses: match by reference.

- [ ] **Step 1: Tests** (`tests/omap_writer.rs`): `text_object_written_like_oom` — one text object at
  (1000, 2000), h_align 1, v_align 0, text `Clara-Zetkin-Schule` → contains
  `<object type="4" symbol="0" h_align="1" v_align="0"><coords count="1">1000 2000;</coords><text>Clara-Zetkin-Schule</text></object>`
  and parses; `text_object_escapes_content` — text `A & B <C> "D"` → escaped, parses. The golden file
  stays byte-identical.
- [ ] **Step 2:** run → fail. **Step 3:** implement. **Step 4:** `cargo test --workspace` → pass; commit
  `feat(omap): text objects`.

### Task 2: Sheet symbols and layout

**Files:** create `assets/symbol-sets/sheet-additions.xml`, `crates/freemapper/src/build/peripheral.rs`;
modify `crates/freemapper/src/omap/symbol_set.rs`, `crates/freemapper/src/project.rs` (`MapKind`),
`crates/freemapper/src/build/mod.rs`.

**Interfaces:**
- `SymbolSet::with_additions(&self, additions_xml: &str) -> Self` (refactored out of `school_2019`:
  replaces `id="@"` with consecutive ids after `max_id`, updates `<symbols count>`).
- `MapKind::symbol_set` returns the set *with* the sheet additions for both kinds.
- `MapKind::symbol_enlargement(self) -> f64` (1.25 / 1.0), `MapKind::contour_interval_m(self) -> u32`
  (1 / 2).
- `build::peripheral::PaperRect { min_x: i32, min_y: i32, max_x: i32, max_y: i32 }` (µm on paper),
  `PaperRect::of_bbox(bbox: &BoundingBox, georef: &Georeferencing) -> Result<Self, GeorefError>` (the
  four corners projected).
- `build::peripheral::SheetInfo { title: String, scale_denominator: u32, contour_interval_m: u32,
  attribution: Vec<String> }` (the credit line is the last attribution line).
- `build::peripheral::north_lines(content: &PaperRect, spacing_mm: f64, symbol_set: &SymbolSet) -> Vec<MapObject>`
- `build::peripheral::info_block(content: &PaperRect, info: &SheetInfo, symbol_set: &SymbolSet) -> Vec<MapObject>`

Layout of the info block (left edge = `content.min_x`, y grows southwards, values in mm on paper):
title baseline at `max_y + 12`; `Maßstab 1:… · Äquidistanz … m` baseline at `+ 6` below the title;
bar scale line at `+ 7` below that, ticks 1.5 mm upwards, labels (small text, centred) 1 mm above the
ticks; attribution lines (small text) from `+ 6` below the bar, 2.5 mm apart.

- [ ] **Step 1: Tests:**
  - `symbol_set.rs` `sheet_additions_extend_both_sets`: school set has 195 symbols, `321` still id 185,
    codes `991`–`994` with ids 191–194; sprint set (via `MapKind::Sprint.symbol_set()`) 189 symbols,
    `991` id 185; `994` is a line symbol with `line_width="250"`; title font `size="5000"` with
    `bold="true"`.
  - `peripheral.rs`:
    - `north_lines_cover_content`: content −100…+120 mm × −80…+60 mm, spacing 37.5 → lines at
      x = −75, −37.5, 0, 37.5, 75, 112.5 mm, each a 601 path from y = min to y = max (2 coords).
    - `north_line_spacing_sprint`: spacing 30 → a line at x = 30 mm.
    - `info_block_sits_below_content`: every coordinate of every object has `y > content.max_y`.
    - `info_block_texts_schoolyard`: texts in order `Clara-Zetkin-Schule`,
      `Maßstab 1:1000 · Äquidistanz 1 m`, `0`, `25`, `50 m`, then the attribution lines; title uses 991.
    - `bar_scale_sprint`: labels `0`, `100`, `200 m`; bar line is 50 mm long.
    - `paper_rect_of_bbox`: a bbox around the reference point maps to a rect containing (0, 0).
  - `project.rs` `map_kind_sheet_values`: enlargement 1.25 / 1.0, interval 1 / 2.
- [ ] **Step 2:** run → fail. **Step 3:** implement; ledger the red-bar ruling.
- [ ] **Step 4:** `cargo test --workspace` → pass; commit `feat: sheet symbols, north lines and info block`.

### Task 3: Project title and export

**Files:** `crates/freemapper/src/project.rs`, `crates/freemapper-cli/src/main.rs`,
`crates/freemapper-cli/tests/cli.rs`.

**Interfaces:**
- `ProjectFile.title: Option<String>` (`#[serde(default)]`); `ProjectFile::display_title(&self, dir: &Path) -> String`
  (title, else the folder name with each `-`/`_`-separated word capitalized, joined by `-`:
  `clara-zetkin-schule` → `Clara-Zetkin-Schule`).
- `init_project(dir, area, kind, scale, date, title: Option<String>)`; CLI `init --title <text>`.
- Export: part `peripheral information` with `north_lines` + `info_block` (content = `project.bbox()`
  projected); buildings stay in `default part`, which is now always written first.

- [ ] **Step 1: Tests:**
  - `project_without_title_loads` (M1-style toml) and `display_title_from_folder_name`.
  - `export_has_one_peripheral_part`: export twice with `--force` → the map has exactly one
    `<part name="peripheral information">`, it contains the title text and the credit line.
  - CLI `init_with_title`: `init … --title "Clara-Zetkin-Schule Strausberg"` then `export` → map text
    contains `<text>Clara-Zetkin-Schule Strausberg</text>`.
- [ ] **Step 2:** run → fail. **Step 3:** implement. **Step 4:** `cargo test --workspace` → pass; commit
  `feat: peripheral information in the export`.

### Task 4: Docs and acceptance

- [ ] README DE/EN (Randangaben / peripheral information, `--title`), spec §4.3 (red bar dropped, with
  reason), glossary (`Randangaben` exists; add `Nordlinien | magnetic north lines | … | north_lines`).
- [ ] `cargo test --workspace`, clippy, live run; regenerate the sample with `run --force` and hand it to
  the user for the OOM check. Commit `docs: M4b peripheral information`.
