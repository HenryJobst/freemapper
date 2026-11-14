# Training map, first half: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `--training` writes `<name>-training.omap` and `<name>-training.pdf`, built from the map and all
candidates by a training level that joins, drops and cuts, measured by a conflict measure.

**Architecture:** A pure module `build/conflicts.rs` measures where drawn footprints come too close. A pure module
`build/training.rs` turns the objects of the map and the visible candidates into the objects of the training map
in a fixed order of steps. `project/export.rs` reads the rasters, calls it and writes the files; the assembly still
reads no file (ADR 0115). Between objects of different kinds this half already keeps every gap of the norm, by
cutting and dropping only. Merging what is alike, shifting before cutting (bounded, never making a new conflict)
and keeping junctions are the second half (another plan); until then lines of one network that run side by side
stay as they are.

**Tech Stack:** Rust (toolchain pinned), `geo` 0.x with `Buffer` and `BooleanOps`, the renderer of
`freemapper-render` (`crate::render::draw::primitives`), the measure types of `measure.rs`.

**Spec:** `docs/superpowers/specs/2026-11-14-training-map-design.md` (parts "Part 2: the measure", "Part 3", "The
findings on the training map", "Order" item 1).

## Global Constraints

- Source code, comments, ADR and plan in English; user docs in German and English; terms from `docs/glossary.md`,
  new ones added there first: Grundkarte / base map, Trainingskarte / training map, Konflikt / conflict,
  Bereinigungsstufe / clean-up level. German text says "Bundesland".
- Gaps and minimum sizes come from ISOM 2017-2 and ISSprOM 2019-2 (public IOF documents); the ADR quotes
  paragraph and value. Measures are `PaperMm` / `SymbolMm` (ADR 0118), never bare numbers at an interface.
- Comparisons at a threshold use a tolerance of 2 map units (`TOLERANCE` in `build/barrier_gaps.rs`), so that
  Linux, macOS and Windows decide alike.
- The base map must not change in this plan except through Task 2 (duplicate steps). Every commit that changes a
  drawn map follows the golden-bless flow: `FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden --
  --ignored g3_trace g4_pipeline`, `scripts/golden.sh test`, commit trailer `Golden-Bless:`.
- Checks before every commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`. Use `CARGO_INCREMENTAL=0`.
- No personal data in requests, file names or fixtures. Live runs use the program's fixed User-Agent.
- Changelog entries go under "Unveröffentlicht" / "Unreleased" (v0.20.0 is released).
- `--pdf-candidates` keeps its name and file; it is the check print, no longer called the training map.

## Review Focus

1. A project without terrain, surface model or rock raster (schoolyard, a state without DOM): the training map is
   still written; the steps that need a raster do nothing. Test in Task 4 and Task 5.
2. A symbol set without a code a rule names (ISSprOM has no 502 to 505, the school set no 104): the rule skips,
   nothing panics. Test in Task 1 and Task 4.
3. Paths with flags (curve start, hole point, dash point, closed rings) through join and cut: the coordinates
   stay a valid object, a ring stays closed. Test in Task 3.
4. An existing `<name>-training.omap` without `--force`: kept, the command fails with the map-exists message
   before it writes anything else of the training map. Test in Task 6.
5. Two objects exactly at the gap: the same verdict on every system. Test in Task 1.

---

### Task 1: The conflict measure

**Files:**
- Create: `crates/freemapper/src/build/conflicts.rs`
- Modify: `crates/freemapper/src/build/mod.rs` (declare the module)

**Interfaces:**
- Consumes: `crate::render::{draw::primitives, symbols::RenderSymbols}`, `SymbolSet::id_for_code`,
  `SymbolScale`, `PaperMm`, `MapKind`.
- Produces:

```rust
/// The classes the relation table knows.
pub enum Class { Building, WaterArea, BankLine, Railway, Road, Path, Watercourse, Stairway,
                 Barrier, Cliff, EarthBank, Contour, Point, OtherArea, Other }
pub enum Relation { Free, Joined, Apart }
pub enum Grade { Overlap, Gap }
pub struct Conflict {
    /// Index of the object that gives way (lower rank; same rank: the shorter) and of the other.
    pub loser: usize, pub winner: usize,
    pub classes: (Class, Class),
    pub grade: Grade,
    /// Length of the loser's centre line in conflict, mm on paper (0 for areas and points).
    pub length: PaperMm,
}
pub struct Footprints { /* per object: class, centre line(s) in paper mm, drawn footprint */ }
impl Footprints {
    pub fn of(objects: &[&MapObject], symbol_set: &SymbolSet, kind: MapKind) -> Self;
    /// The drawn footprint of object `i`, paper mm; empty when its symbol draws nothing.
    pub fn footprint(&self, i: usize) -> &MultiPolygon<f64>;
    pub fn class(&self, i: usize) -> Class;
}
pub fn class_of(code: &str, kind: MapKind) -> Class;
pub fn rank(class: Class) -> u8;                 // higher wins
pub fn relation(a: Class, b: Class) -> Relation;
pub fn gap(kind: MapKind, scale_denominator: u32) -> PaperMm;
pub fn conflicts(footprints: &Footprints, kind: MapKind, scale_denominator: u32) -> Vec<Conflict>;
pub const MIN_CONFLICT: PaperMm = PaperMm(1.0);
```

Decisions:
- Footprint = union of the object's render primitives: every `Shape::Stroke` buffered by half its width (butt
  caps), every `Shape::Fill`. Ticks and borders are part of it. Text draws nothing here.
- Classes by code: 521/522 Building; 301, 301.1, 302 WaterArea; 301.4 BankLine; 509 Railway; 502, 503, 504, 501
  areas' lines Road; 505, 506, 507, 501.6 to 501.9 Path; 304 to 306 Watercourse; 532 and 532.x Stairway; 513, 515,
  516, 518 and their variants, 410.1 Barrier; 201 to 203 Cliff; 104, 104.1 EarthBank; 101 to 103 Contour; point
  symbols (`symbol_type` point) Point; area symbols not named OtherArea; the rest Other. A code with a suffix the
  table does not name takes the class of its stem.
- Rank: Building 9, WaterArea 8, BankLine 8, Railway 7, Road 6, Path 5, Watercourse 4, Stairway 4, Barrier 3,
  Cliff 3, EarthBank 2, Contour 1, Point 0. OtherArea and Other take part in no conflict.
- Relation: any pair with OtherArea or Other is Free; Contour is Free with everything except EarthBank, Cliff and
  Building (Apart); Road, Path, Railway and Stairway among each other are Joined; Watercourse with Watercourse and
  with WaterArea or BankLine is Joined; WaterArea with BankLine is Free; a Road, Path or Railway with WaterArea or
  BankLine is Free (bridges); every other pair is Apart. The table is one `match`, symmetric by construction.
- `gap`: 0.15 mm (minimum gap between two lines of the same colour, ISOM 2017-2 and ISSprOM 2019-2; confirm the
  paragraph) times the symbol enlargement of `SymbolScale::new(kind, scale_denominator)`.
- A conflict of grade Overlap: the loser's centre line inside the winner's footprint grown by the loser's half
  width; grade Gap: inside that grown by `gap` more, minus the Overlap part. For Joined pairs the neighbourhood of
  every point where the two centre lines meet or cross is left out, radius = both half widths + `gap`. A stretch
  counts from `MIN_CONFLICT`. Points and areas as losers: Overlap when the footprints intersect, length 0.
- One `Conflict` per pair and grade. Pairs are found with a uniform grid over the bounding boxes, not all pairs.

- [ ] **Step 1: Write the failing tests** in `conflicts.rs` (`#[cfg(test)]`, fixture as in `build/line_cuts.rs`
  tests: ISOM set at 1:10 000):
  - `an_earth_bank_on_a_ditch_is_an_overlap`: a 306 line of 20 mm and a 104 line parallel 0.2 mm beside it give one
    conflict, `loser` the 104, `grade == Overlap`, `length` within 1 mm of 20 mm.
  - `an_earth_bank_beside_a_ditch_is_a_gap`: the same at a distance where the footprints are 0.1 mm apart gives
    `Gap`; 1 mm apart gives none.
  - `two_paths_that_cross_are_no_conflict` and `two_paths_side_by_side_are`: 505 crossing at right angles: none;
    505 parallel 0.3 mm apart over 10 mm: one, about 10 mm.
  - `a_path_over_water_is_free`, `a_contour_under_a_path_is_free`, `a_contour_through_a_building_is_an_overlap`.
  - `a_point_on_a_building_is_an_overlap` (526 inside a 521).
  - `the_verdict_at_the_gap_is_tolerant`: two lines whose footprints are exactly `gap` apart, and the same moved
    by 1 map unit either way, give the same result (no conflict).
  - `a_set_without_the_code_gives_no_conflict`: the ISSprOM set and objects of codes it lacks: empty, no panic.
  - `the_relation_is_symmetric`: for all pairs of `Class`, `relation(a, b) == relation(b, a)`.
- [ ] **Step 2: Run** `cargo test -p freemapper conflicts::` and see them fail to compile.
- [ ] **Step 3: Implement** the module as specified.
- [ ] **Step 4: Run** the tests, `cargo clippy`, see them pass.
- [ ] **Step 5: Commit** `feat(build): a conflict measure for drawn symbols`.

### Task 2: Repair the duplicate steps (water, stairs)

**Files:**
- Modify: `crates/freemapper/src/build/water.rs` (`drop_covered_water`, `OSM_WATER_COVERED`), its call in
  `build/assembly.rs`; `crates/freemapper/src/build/stairs.rs` or `build/dedup.rs` (where OSM stairs meet ALKIS
  stairs).

Facts to explain first (measured on Bremen Bürgerpark, forest, 1:10 000): five OSM `natural=water` multipolygon
relations (`relation/372786`, `3046999`, `4505380`, `5482048` and one more) lie 91 to 99 % on ALKIS water areas
(301.1, land use "Fließgewässer / Graben", "Stehendes Gewässer / See") and are still drawn as 301; 6 of 23 OSM
`highway=steps` lie within 0.3 mm of an ALKIS "Treppe" 532.

- [ ] **Step 1: Reproduce.** Build the Bremen project in scratch (`freemapper init <scratch>/bb --osm way/664446849
  --kind forest --scale 10000`, `fetch`, `derive`), then write a test or a debug run that shows which guard of
  `drop_covered_water` keeps each lake (candidates: the union passed in holds only part of the ALKIS water, the
  OSM relation has holes, the forest kind takes another path, the share is taken of the unclipped area). Write
  the cause into the commit message.
- [ ] **Step 2: Write the failing test** for the cause found, next to the existing `drop_covered_water` tests,
  with a synthetic lake of that shape: the OSM area is dropped.
- [ ] **Step 3: Fix**, run the water tests.
- [ ] **Step 4: Stairs.** Failing test: an OSM stair whose line lies within the drawn width of an ALKIS stair for
  more than half its length is dropped and named in the ALKIS stair's `freemapper:also`; one that only crosses
  stays. Then the fix in the step that already compares stairs of two sources.
- [ ] **Step 5: Bless the goldens** if they change, run all checks.
- [ ] **Step 6: Commit** `fix(build): OSM water and stairs that double the cadastre are dropped` with the cause.

### Task 3: Join lines

**Files:**
- Create: `crates/freemapper/src/build/line_join.rs`; declare in `build/mod.rs`.

**Interfaces:**
- Produces:

```rust
/// Joins the path objects of each symbol in `codes` that meet end to end. Returns how many objects went.
pub fn join_lines(objects: &mut Vec<MapObject>, codes: &[&str], georef: &Georeferencing,
                  symbol_set: &SymbolSet) -> usize;
/// Ends closer than this touch (ground metres), as `JOIN_TOUCH_M` in `build/candidates.rs`.
pub const TOUCH_M: f64 = 0.1;
/// At a junction of three or more ends the straightest pair is joined when it turns less than this.
pub const MAX_TURN_DEG: f64 = 30.0;
```

Decisions: only objects of the same symbol id join. Where exactly two ends meet, they join whatever the angle;
where more meet, the pair with the smallest turn joins if under `MAX_TURN_DEG`, the others stay. The joined
object keeps the tags of the longer one and gets `freemapper:also` with the `freemapper:source_id` of each other
one (the form the duplicate steps write: `<source>/<id>`, joined by `; `). A line that closes on itself becomes a
closed path. Flags inside a line are kept; the flags of the two joined end points follow what a single line
would carry there. Order of the result: the joined object takes the place of its first member.

- [ ] **Step 1: Failing tests:** `two_pieces_become_one` (3 + 4 points → 6 points, one object, `also` set);
  `three_ends_join_the_straight_pair` (a T: the bar is one object, the stem stays); `a_fork_of_60_degrees_joins
  nothing`; `different_symbols_do_not_join`; `a_ring_of_pieces_is_closed`; `curve_flags_survive` (a piece with a
  curve start flag joined to a straight one keeps its three control points and flags); `ends_one_metre_apart_do
  not_join`.
- [ ] **Step 2: Run and see them fail. Step 3: Implement. Step 4: Run and pass.**
- [ ] **Step 5: Commit** `feat(build): lines of one symbol that meet end to end are joined`.

### Task 4: The training level (join, give way, cut, minimum sizes)

**Files:**
- Create: `crates/freemapper/src/build/training.rs`; declare in `build/mod.rs`.

**Interfaces:**
- Consumes: Task 1 (`Footprints`, `conflicts`, `Class`, `Grade`), Task 3 (`join_lines`),
  `build::line_cuts::{cut_lines_at_ways, CONTOUR_CODES}`, `build::geometry::object_polygon`,
  `build::cliffs::{generalize_cliffs, hide_landforms_on_rock, Ground, Level}`.
- Produces:

```rust
/// What the rasters say, read by the project; each may be missing.
pub struct TrainingGround<'a> {
    pub terrain: Option<&'a Raster<f32>>,
    pub rock_seen: Option<&'a Raster<f32>>,
    /// 1 where trees stand (Task 5).
    pub canopy: Option<&'a Raster<f32>>,
}
/// What the training level did, for the summary line and the ADR.
#[derive(Debug, Default, PartialEq)]
pub struct TrainingReport {
    pub joined: usize, pub gave_way: usize, pub cut: usize, pub too_small: usize,
    pub open_land_cut: usize,
    /// Conflict length left, mm on paper, by grade.
    pub overlap_left: f64, pub gap_left: f64,
}
/// The objects of the training map from the objects of the map and the visible candidates.
pub fn train(objects: Vec<MapObject>, ground: TrainingGround<'_>, kind: MapKind,
             georef: &Georeferencing, symbol_set: &SymbolSet) -> (Vec<MapObject>, TrainingReport);
```

Order of steps inside `train`, each a private function with its own tests:
1. Rock: what `training_rock` in `project/export.rs` does today, moved here unchanged (forest and sprint only).
2. `join_lines` on the codes of the classes Road, Path, Railway, Watercourse, EarthBank, Barrier, Contour.
3. Contours are cut at the footprints of Building objects (`cut_lines_at_ways` with `CONTOUR_CODES`, minimum piece
   as `place` uses for contours).
4. Give way: for the conflicts of Apart pairs of both grades (the training map keeps every gap), losers by
   ascending rank: a loser with more than half its length in conflict goes whole; otherwise the stretch is cut
   out (winner's footprint grown by the loser's half width plus `gap`), pieces below the minimum length of step 5
   go. Points and areas that lose go whole. Among objects of one class the shorter loses. Dropped objects are
   gone from the training map; nothing is tagged hidden. The step repeats until `conflicts` finds no Apart pair
   (each round removes geometry, so it ends).
5. Minimum sizes: a line shorter than its symbol's minimum length goes. The minimum is the symbol's
   `minimum_length` in the set where it is above 0, else a table from the norm for 104 (0.6 mm), 532, 201, 101,
   513 to 518; every value times the symbol enlargement. A line that continues in another line of its class
   (an end within `TOUCH_M` of it) is not isolated and stays.
6. The report: counts, and the conflict length left by `conflicts` on the result.

- [ ] **Step 1: Failing tests** (ISOM set, 1:10 000, no rasters unless said):
  - `an_earth_bank_on_a_ditch_goes`: 306 of 20 mm and 104 on it → the 104 is gone, the 306 stays, `gave_way == 1`.
  - `an_earth_bank_that_crosses_a_path_is_cut`: 104 of 20 mm crossing a 505 at right angles → two 104 pieces, a
    gap around the path, `cut == 1`.
  - `a_fence_on_a_path_goes_where_it_lies_on_it`: 518 of 20 mm, 8 mm of it on a 505 → one 518 of about 12 mm.
  - `an_earth_bank_inside_the_gap_of_a_ditch_goes_too`: footprints 0.1 mm apart → the 104 is gone.
  - `of_two_earth_banks_inside_the_gap_the_shorter_goes`.
  - `no_apart_conflict_is_left`: a mixed scene (bank, ditch, fence, path, building, point) → `conflicts` on the
    result has no Apart pair of either grade.
  - `a_point_on_a_building_goes`; `a_point_beside_it_stays`.
  - `a_contour_stops_at_a_building`; `a_contour_on_an_earth_bank_goes`.
  - `a_stair_below_the_minimum_goes`; `a_short_path_that_continues_stays`.
  - `pieces_of_a_path_are_one_object_after` (5 pieces of 505 in a row → 1).
  - `carriageways_and_tracks_are_left_alone` (two parallel 502, two parallel 509: unchanged; they are the second
    half).
  - `without_rasters_nothing_fails` (schoolyard set, `TrainingGround` all `None`).
  - `the_report_counts_what_is_left`: two parallel 505 inside the gap (a Joined pair, left to the second half)
    → `gap_left` or `overlap_left` is above 0; after `an_earth_bank_on_a_ditch_goes` both are 0.
- [ ] **Step 2: Run and see them fail. Step 3: Implement. Step 4: Run and pass**, clippy.
- [ ] **Step 5: Commit** `feat(build): the training level joins, gives way and cuts`.

### Task 5: Open land under trees

**Files:**
- Modify: the derive step that writes `data/rock-seen.tif` (`ROCK_SEEN_RELPATH` in `project.rs`) to also write
  `data/canopy.tif` (`CANOPY_RELPATH`); `build/training.rs` (step between 3 and 4); `build/open_land.rs` only if a
  helper is shared.

Decisions: `derive` writes the canopy raster where a surface model is stored, for both kinds of provider (per-area
model at `SURFACE_RELPATH`, tiled nDSM): cell value 1 where surface minus terrain is at least 3 m (ruling: a tree
that hides the ground from a runner's view of open land), else 0, at the cell size of the terrain. In `train`, a
401 or 403 object whose tag `freemapper:nutzart` names a use without ground cover (the classes `open_land.rs` maps
from "Park", "Grünanlage", "Sport-, Freizeit- und Erholungsfläche"; take the list from its table) is intersected
with the open cells: the canopy mask is smoothed and held to the minimum area and width the vegetation areas use
(the functions of `build/vegetation.rs`, ADR 0136), then subtracted. Pieces below the minimum area of 401 go. An
object without that tag, and every object when `canopy` is `None`, stays as it is.

- [ ] **Step 1: Failing tests:** `a_park_keeps_only_its_open_half` (a 401 square tagged "… / Park", canopy raster
  with trees on its left half → one 401 over the right half, `open_land_cut == 1`); `a_meadow_from_another_use_is
  left` (401 without the tag); `without_canopy_nothing_changes`; `a_glade_below_the_minimum_goes`. In the derive
  tests: `derive_writes_the_canopy_where_a_surface_model_is_stored` and `…_writes_none_without_one`.
- [ ] **Step 2 to 4:** fail, implement, pass.
- [ ] **Step 5: Commit** `feat: open land under trees stays forest on the training map`.

### Task 6: The training map files

**Files:**
- Modify: `crates/freemapper/src/project/export.rs` (replace `training_rock`; new functions), `project.rs` (re-exports,
  tests).

**Interfaces:**
- Consumes: `train`, `TrainingGround`, `pdf_objects`, `write_document`, `back_up`, `write_inside`, `info_block`.
- Produces:

```rust
pub struct TrainingOutcome { pub map: PathBuf, pub pdf: PathBuf, pub report: TrainingReport }
pub fn training_map_path(dir: &Path) -> PathBuf;   // <stem>-training.omap
pub fn training_pdf_path(dir: &Path) -> PathBuf;   // <stem>-training.pdf
/// Writes the training map and its PDF. An existing training map is kept unless `force` (then `.omap.bak`).
pub fn export_training_from(dir: &Path, documents: &StarterDocuments, force: bool, scale_bar: bool,
                            sources: PdfSources) -> Result<TrainingOutcome, ProjectError>;
pub fn export_training(dir: &Path, force: bool, scale_bar: bool, sources: PdfSources)
                       -> Result<TrainingOutcome, ProjectError>;
```

Decisions: the input of `train` is the first list of `pdf_objects(documents, true)` without the walkable-area part
and without sheet objects. The training document has the parts of the map in their order (walkable area if any,
`default part` with the trained objects, the sheet part), the notes of the map and its templates without those of
`TemplateKind::Map`. The PDF is drawn from the same objects by the code `write_pdf_file` uses. The check for an
existing training map comes first and returns `ProjectError::MapExists(path)`. With an OCAD export in the same
command, `export_ocad_from` gains the training map as `ocad/<stem>-training.ocd` (field `training:
Option<PathBuf>` in `OcadOutcome`, written only when the training documents are passed). `write_pdf_file` with
`candidates == true` no longer calls the rock step: the check print shows the map level.

- [ ] **Step 1: Failing tests** in `project.rs` beside `candidates_pdf_draws_map_and_candidates`:
  `the_training_map_is_written_beside_the_map` (both files exist, the `.omap` parses, lists no `candidates.omap`
  template, holds the sheet part); `the_training_map_takes_map_and_candidates` (a fixture with a candidate fence:
  it is in the training map); `an_existing_training_map_is_kept` (second call without force → `MapExists`, file
  unchanged, no PDF rewritten; with force → `.omap.bak`); `the_candidates_pdf_is_the_map_level_again` (adapt
  `the_training_map_draws_no_top_lines`-style assertions of the PDF tests that relied on the training rock).
- [ ] **Step 2 to 4:** fail, implement, pass; goldens blessed if the candidates PDF golden changes.
- [ ] **Step 5: Commit** `feat: the training map is a map file and a PDF of its own`.

### Task 7: The switch

**Files:** every place `pdf_candidates` is threaded through gets a `training` beside it:
`crates/freemapper-cli/src/main.rs` (`run`, `export`: `--training`), `pipeline.rs` (`PipelineOptions`, the request,
`Step::Training`, the result files of kind `training-map` and `training-pdf`, the summary), `mcp.rs`
(`start_pipeline`, `open_area_selector`: `training`), `select.rs`, `select_page.rs`, `select_run.rs`,
`web/index.html`, `web/run.js` (the box "Trainingskarte" / "Training map"), `i18n/de/freemapper.ftl`,
`i18n/en/freemapper.ftl`, `help.rs`, and the tests `tests/cli.rs`, `tests/mcp.rs`, `src/select_run/tests.rs`.

Decisions: `--training` is independent of `--pdf` and `--pdf-candidates`; `--pdf-scale-bar` and `--pdf-sources`
apply to its PDF; `--force` applies to it as to the map. Not stored in `freemapper.toml`. Messages: "Wrote
<path> (the training map: joined …, left out …)" from the report; when the training map exists and `--force` is
missing, the map-exists message names the training map. Texts for `--pdf-candidates` change from "the training
map" to "the check print: the map with every candidate, true to position" in help, MCP descriptions and i18n.

- [ ] **Step 1: Failing tests:** CLI: `run --training` writes both files and prints the line; `export --training`
  on an existing training map without `--force` exits 1 with the message; MCP: `start_pipeline` with `training:
  true` lists both files in `files`; the page: the box is in the form and reaches the run options.
- [ ] **Step 2 to 4:** fail, implement, pass.
- [ ] **Step 5: Commit** `feat(cli): --training writes the training map`.

### Task 8: Docs, ADR, changelog

**Files:** `docs/glossary.md`, `docs/adr/0137-the-training-map-is-a-product-of-its-own.md` (check the number is
free) and `docs/adr/README.md`, `AGENTS.md`, `README.md`, `README.en.md`, `docs/manual/de.md`, `docs/manual/en.md`,
`CHANGELOG.md`, `CHANGELOG.en.md`.

- [ ] **Step 1:** Glossary terms. **Step 2:** The ADR: context (the Bremen table of the spec), decision (product,
  switch, input, the steps of the training level, the relation table and ranks, the norm values with paragraph),
  what is left to the second half, consequences. Amends ADR 0080 and 0126. **Step 3:** AGENTS.md (commands, outputs,
  "What lands in the project folder", "What to tell the user"), READMEs, manual chapters in both languages,
  changelog under Unreleased. Run the skills `unslop` and `humanizer` on the user texts.
- [ ] **Step 4: Commit** `docs: the training map`.

### Task 9: Measure and open the pull request

- [ ] **Step 1:** Build Bremen Bürgerpark, one sprint example and one rocky forest example in scratch with
  `--training --pdf-candidates`. Record per example: conflict length by grade for the check print (the old
  training map) and for the training map, from a test-only or `--lang en` summary of the report; render both PDFs
  and look at them.
- [ ] **Step 2:** Tick the rows of "The findings on the training map" this half must reach: water twice, footpaths
  in pieces, stairs, contours, forest drawn yellow, monument points; write the numbers and the rows left for the
  second half into the ADR.
- [ ] **Step 3:** Full checks, push the branch to GitHub, open the pull request (label `all-systems`: file names
  and paths), wait for the user's yes.
