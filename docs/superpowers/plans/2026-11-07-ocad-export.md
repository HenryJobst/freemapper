# OCAD Export – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper export --ocad` (and `run --ocad`, MCP `start_pipeline {ocad: true}`) also writes an
OCAD starting point. It contains `ocad/<name>.ocd` (OCAD 12 format: colours, placeholder symbols,
map objects, georeferencing, background images), `ocad/<name>-candidates.ocd` and the images with
their world files next to them. In OCAD the user loads the official ISSprOM 2019 / school symbol
set ("Load colors and symbols from"), which replaces the placeholders by number.

**Architecture:** New library module `ocd/`:
- `file.rs`: the binary OCD 12 builder (header, index chains, parameter strings, symbol and
  object records, TDPoly).
- `symbols.rs`: colours and placeholder symbols from the OOM symbol set XML, plus the explicit
  freemapper code → OCAD number table.
- `convert.rs`: `MapDocument` → OCD. Paper coordinates are re-projected exactly, areas get holes,
  lines are split, texts get boxes, templates are positioned from their world files.

A small reader exists for tests only. The OOM writer is unchanged; export builds the same
`MapDocument`s and hands them to both writers.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` "Later: OCAD starter project".
User decision (2026-11-07): OCD 12 with placeholder symbols plus the official OCAD set; the user tests
in a current OCAD, optionally OCAD 12. Byte-level reference: `docs/ocad/ocd12-reference.md`. It is
compiled from the OCAD 12 spec, OOM's exporter, Purple Pen's exporter, files written by OCAD 12 and
OCAD's official ISSprOM 2019-2 set.

## Global Constraints

- **Binary layout:** little-endian, packed records. Header 60 bytes; symbol index block 1028; object
  index block 10244; string index block 4100. Records: base symbol 796, point 800 + 8·DataSize,
  line 872, area 832, text 1036; object 56 + 8·(nItem + nText). The index `Len` is **bytes**.
- **Coordinates:** 0.01 mm, y up, `(v << 8) | flags`. Hole = y-bit 2 on the first point of every
  hole ring. The bbox covers all coordinates ± the symbol `Extent`. All coordinates within ±2000 mm.
- **Georeferencing:** string 1039
  `\tm{scale}\tg{grid_mm}\tr1\tx{x0}\ty{y0}\ta{grivation}\td{grid_m}\ti8008\tb0.00\tc0.00`.
  - x0/y0 are the ref point rounded to whole metres.
  - Every coordinate is re-projected: `map_to_projected`, then the OCAD inverse
    `X = (cos a·ΔE − sin a·ΔN)/s`, `Y = (sin a·ΔE + cos a·ΔN)/s` (`s = scale/1000`). This avoids
    the 0.3 m/km drift.
- **Colours:** string 9 `Name\tn{i}\tc\tm\ty\tk\to0\tt{opacity%}` in OOM priority order (first on
  top), CMYK in percent, ASCII names (ä→ae, ö→oe, ü→ue, ß→ss, other non-ASCII dropped).
- **Symbol numbers:** OCAD = `number × 1000`, where the sub-number is a three-place decimal.
  - Explicit table: 101→101000, 102→102000, 201→201000, 321→321000, 401→401000,
    410.1→411002 (sprint) / 410002 (schoolyard), 418→418000, 501→501000, 501.1→501100,
    501.5→501001, 501.10→501020, 505.1→505000, 505.2→505001, 506→506000, 513.2→513200,
    515→515000, 518→518000, 520→520000, 521→521000, 521.1→521003, 541–545→541000–545000,
    601→601000, 991–994→991000–994000.
  - Other codes: `a.b` → `a·1000 + b` right-padded to three digits, bumped to the next free number
    on a clash.
- **Placeholders:**
  - line: OOM colour, width, simple dash
  - area: fill colour
  - point: dot (inner colour, Ø = 2·inner radius) or a 0.5 mm dot in the first colour found
  - combined: its area part, else its first line part
  - text: Arial, size in pt × 10, bold → 700, one OCAD symbol per used horizontal alignment
    (extra ones +1)
  - Name: the OOM name, ASCII, ≤ 63 characters. Icon: white (124) with a glyph in the main colour.
  - Only symbols used by objects are written.
- **Texts:** Otp 4, five coordinates (anchor on the first baseline, then box BL, BR, TR, TL). Box
  estimate: width 0.6·em·characters, ascent 0.9·em, descent 0.25·em. UTF-16LE, CR LF, NUL, padded to
  64 bytes.
- **Templates:**
  - Images and world files are copied next to the .ocd and referenced by bare file name.
  - String 8: `{file}\ts{0|1}\tr1\tu{u}\tv{v}\tx{x}\ty{y}\ta{grivation}\tb{grivation}\td{dim}`.
    x/y is the image centre on OCAD paper in mm; u/v are mm per pixel; dim = 100·(1 − opacity).
  - The candidates map `<name>-candidates.ocd` is referenced the same way (bare name, `s1\tr1`).
- **Notes and view:** map notes string 1061 with the attribution and the freemapper credit line
  (ASCII, CR LF). View string 1030 centred on the sheet.
- **Overwriting:** an existing `ocad/<name>.ocd` is replaced only with `--force` (kept as `.ocd.bak`),
  like the OOM map. The candidates file and the copied images are always refreshed.

## Review Focus

1. **Objects outside ±2000 mm or empty geometry** (1-point lines, < 3-point areas): skipped with a
   count in the progress, never written.
2. **A symbol used with both text alignments** gets two OCAD symbols; the numbers stay unique.
3. **Projects without templates, buildings or OSM data** – the OCD is still valid.
4. **The user's edited `.ocd`** is never overwritten without `--force`.
5. **Non-ASCII in names/notes** (Straßenverkehr, Äquidistanz) – ASCII in parameter strings, UTF-16 in
   object texts.

---

### Task 1: OCD 12 binary builder and test reader
`ocd/file.rs`:
- `OcdBuilder::new()`, `add_string(type, &str)`, `add_symbol(&OcdSymbol) -> i32`,
  `add_object(&OcdObject)`, `finish() -> Vec<u8>`.
- `OcdSymbol { number, name, kind: Point{dot_diameter, color} | Line{color, width, dash: Option<(i16, i16)>} | Area{color} | Text{color, size_pt10, bold, align}, colors }`.
- `OcdObject { symbol, kind: Point{angle} | Line | Area | Text{angle, text, box}, coords: Vec<OcdCoord{x, y, xflags, yflags}> }`.
- `ocd/read.rs` (`#[cfg(test)]`): walks the chains and decodes strings, symbols and objects.

Tests:
- `header_and_chains`
- `chains_grow_past_256_entries`
- `symbol_record_sizes`
- `object_len_is_bytes`
- `bbox_covers_points_and_extent`
- `text_is_utf16_padded`
- `strings_are_nul_terminated`

### Task 2: Colours, placeholder symbols, numbers
`ocd/symbols.rs`:
- `ocad_number(code, kind) -> i32`, plus the fallback with clash bumping.
- `colours(&SymbolSet) -> Vec<OcdColour>`.
- `placeholder(&SymbolSet, symbol_id) -> Option<Placeholder>` (OOM XML → kind, main colour, sizes).

Tests:
- `numbers_follow_ocads_decimal_rule`
- `hedge_number_depends_on_kind`
- `unknown_codes_get_unique_numbers`
- `colours_keep_priority_order`
- `names_are_ascii`
- `line_area_point_text_placeholders`
- `combined_symbols_use_their_area_part`

### Task 3: Map document → OCD files
`ocd/convert.rs`:
- `write_ocd(doc: &MapDocument, kind: MapKind, dir: &Path, out: &Path, templates: &[OcdTemplate]) -> Result<OcdReport, OcdError>`.
- Georeferencing re-projection; areas with holes; line splitting; point rotation; texts; templates
  from world files; notes; view.

`project::export_ocad(dir, force) -> Result<OcadOutcome, ProjectError>` writes `ocad/`.

Tests:
- `coordinates_reproject_exactly` (< 1 cm against `map_to_projected`)
- `area_holes_are_flagged`
- `multipart_lines_are_split`
- `degenerate_objects_are_skipped`
- `template_position_from_world_file`
- `export_ocad_writes_map_candidates_and_images`
- `export_ocad_keeps_an_edited_ocd`
- `notes_carry_the_credit_line`

### Task 4: CLI, MCP, docs, acceptance
- `export --ocad`, `run --ocad`, MCP `start_pipeline {ocad}`.
- README DE/EN: "Load colors and symbols from" with the official set; the open points for the user's
  OCAD test.
- Glossary (`OCAD-Datei` / OCD file, `Platzhaltersymbol` / placeholder symbol).
- Live export of the school and sprint sample projects; the files are handed to the user for the
  OCAD check.
