# Quick Training Map (PDF) – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper export --pdf` (and `run --pdf`, MCP `start_pipeline {pdf: true}`) writes
`<name>.pdf`: a fully generated training map, ready to print. Candidates become map objects:
OSM furniture, barriers, paths, contours, out-of-bounds areas, ALKIS ways. The map is drawn with
the project's OOM symbol set the way OOM draws it, with the peripheral information and the
freemapper credit line on the sheet.

**Architecture:** New library module `render/`:
- `symbols.rs`: OOM symbol XML → render model.
- `geometry.rs`: path parts, Bézier flattening, arc length, dashes, mid-symbol placement, border
  offsets.
- `draw.rs`: objects → primitives tagged with a colour priority.
- `metrics.rs`: Helvetica widths and WinAnsi encoding.
- `pdf.rs`: primitives → PDF with `pdf-writer`.

Primitives are painted by colour priority, highest number first, so priority 0 is on top. The
export reuses `starter_documents` and merges the map and candidates parts, without the map area
outline.

**Tech Stack:** `pdf-writer = "0.15.0"` (MIT/Apache-2.0, 2026-05-27). Text uses the PDF standard
fonts Helvetica / Helvetica-Bold (metrically equal to Arial) with WinAnsi encoding, so no font is
embedded. Widths come from Adobe's Core 14 AFM files, notices kept in `metrics.rs`.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` "Later: Quick training map".
Rendering rules: `docs/render/oom-render-reference.md`, compiled from OOM's renderer sources; its
section numbers are cited below.

## Global Constraints

- **Output policy:** this switch deliberately overrides "Sicheres fertig, Rest Vorlage" (user,
  2026-11-07). Every candidate object except the map area outline is drawn.
- **Draw order:** by colour priority (= colour index), from the highest number down to 0. Fills of
  different objects are never merged. Areas fill even-odd.
- **Colours:** DeviceCMYK from the `c m y k` attributes; opacity 1.
- **Line rendering (reference §2):**
  - width, caps 0/1/2 → PDF 0/1/2, joins 0 bevel / 1 miter / 2 round → PDF 2/0/1, miter limit √5
  - dashed main lines (§2.2)
  - borders offset `line_width/2 + shift` with flat caps, dashed from the centre-line pattern (§2.3)
  - mid symbols placed by least deviation, local +y to the right of travel (§2.4)
  - start and end symbols (§2.6)
- **Point symbols (§3):**
  - the dot is a filled circle r = inner_radius
  - the ring is a stroked circle r = inner_radius + outer_width/2, width outer_width
  - elements are transformed by `x cos r − y sin r + X`, `y cos r + x sin r + Y`
  - r = −rotation for rotatable map points, 0 otherwise
- **Combined symbols:** every part draws the same path parts. Part `symbol=N` is the symbol index N.
- **Texts (§6):**
  - em = font size; Helvetica(-Bold) at `size_mm × 72/25.4` pt
  - baseline anchor for `v_align` 0; Top / VCenter / Bottom per §6
  - x offset from the measured width (0 / −w/2 / −w)
  - not rotated (the sheet symbols are not rotatable)
- **Bézier flattening:** chords ≤ 1 mm, error ≤ 0.005 mm.
- **Sheet:**
  - The page is the smallest of A4 and A3 (portrait or landscape) that holds all primitives plus a
    5 mm margin, else exactly that size.
  - The content is centred on the page.
  - Map objects are clipped to the content rectangle (map area + buffer); the peripheral objects
    are not.
- **PDF metadata:** Title = map title; Creator = "freemapper – https://github.com/HenryJobst/freemapper".
  The sheet already carries the credit line.

## Review Focus

1. **Closed lines and rings** (fences around yards, contour rings): dashes and mid symbols behave per
   §2.2/§2.4, no wrap-around artefacts.
2. **Very short lines** (< one dash period, 518 shorter than 2.775 mm): solid / no tick, no panic.
3. **Characters outside WinAnsi** (e.g. „€“ is fine, „Ω“ not): replaced by "?", width counted.
4. **Empty projects** (no buildings, no OSM): the PDF still shows the sheet.
5. **School map at 1:1000** (enlarged school set): sizes follow the enlarged XML.

---

### Task 1: Render model from the OOM symbol XML
`render/symbols.rs`:
- `RenderSymbols::from_set(&SymbolSet) -> RenderSymbols` (by symbol index), plus
  `colours: Vec<RenderColour { priority, cmyk }>`.
- `Symbol = Line(LineDef) | Area(AreaDef) | Point(PointDef) | Text(TextDef) | Combined(Vec<Part>)`.
  - `LineDef { color, width, cap, join, dash: Option<Dash{dash, gap, group, in_group_gap, half_outer}>, borders: Option<Border{color, width, shift, dash}>, mid: Option<Mid{symbol: Box<PointDef>, segment, end, per_spot, distance, at_least_one}>, start, end: Option<Box<PointDef>> }`
  - `PointDef { rotatable, inner: Option<(color, radius)>, ring: Option<(color, radius, width)>, elements: Vec<(Symbol, Vec<MapCoord>, is_point)> }`
  - `TextDef { color, size, bold, italic, line_spacing }`

Tests:
- `colours_have_priority_and_cmyk`
- `line_with_dashed_borders` (505.1)
- `dashed_main_line` (506)
- `mid_symbol_of_fence` (518)
- `start_and_end_symbols` (543)
- `point_with_ring_and_elements` (418, 545)
- `combined_parts_resolve` (521, 501)
- `text_symbol` (991)

### Task 2: Geometry
`render/geometry.rs`:
- `parts(coords) -> Vec<Part{points: Vec<(f64, f64)>, closed}>` (mm, Béziers flattened).
- `length`, `at(part, s) -> (point, tangent)`.
- `dashes(part, dash) -> Vec<Vec<(f64, f64)>>` (§2.2).
- `mid_positions(part, mid) -> Vec<(point, angle)>` (§2.4).
- `offset(part, d, join) -> Vec<(f64, f64)>` (§2.3, miter points with the 126.87° limit).

Tests:
- `bezier_is_flattened_within_tolerance`
- `dashes_are_stretched_to_whole_dashes` (506, 10 mm)
- `short_dashed_line_is_solid`
- `closed_dash_halves_meet`
- `mid_symbols_fill_the_line` (518, 20 mm)
- `short_fence_has_no_tick`
- `offset_of_a_right_angle_is_mitred`
- `tangent_at_vertex_is_the_bisector`

### Task 3: Primitives
`render/draw.rs`:
- `Primitive { priority, kind: Fill{rings, even_odd} | Stroke{path, closed, width, cap, join} | Text{origin, size, bold, text} }`.
- `primitives(doc_objects, symbols) -> Vec<Primitive>`.

Tests:
- `paved_area_edge_lies_below_its_fill` (501: fill 14, stroke 15)
- `bush_is_dot_and_ring`
- `fence_ticks_on_the_right`
- `building_has_fill_and_outline`
- `text_alignment_uses_widths`
- `rotation_of_rotatable_points`

### Task 4: PDF writer
`render/pdf.rs`, `render/metrics.rs`:
- `write_pdf(primitives, clip, sheet, title) -> Vec<u8>`: page choice, sorting, DeviceCMYK paths,
  clip, Helvetica fonts.
- `metrics::win_ansi(char) -> u8`, `metrics::width(text, bold) -> f64` (em).

Tests:
- `page_size_is_a4_or_a3`
- `primitives_are_painted_by_priority`
- `text_is_win_ansi_with_helvetica`
- `pdf_structure_is_valid` (header, xref offsets match, trailer)

### Task 5: Full map export, CLI, MCP, docs
- `project::export_training_pdf(dir) -> Result<PathBuf, ProjectError>`: map + candidates without the
  outline; the sheet unclipped.
- `export --pdf`, `run --pdf`, MCP `start_pipeline {pdf}`.
- README DE/EN; glossary (`Trainingskarte` / training map).
- Live: render the school and sprint samples, convert to PNG (`sips`) and check visually.
