# Training map, first half: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `--training` writes `<name>-training.omap` and `<name>-training.pdf`, built from the map and all
candidates by a training level that joins, drops and cuts, measured by a conflict measure.

**Architecture:** A pure module `build/conflicts.rs` measures where drawn footprints come too close. A pure module
`build/training.rs` turns the objects of the map and the visible candidates into the objects of the training map
in a fixed order of steps. `project/export.rs` reads the rasters, runs the rock step it has and then the training
level, and writes the files; the assembly still
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
- Comparisons at a threshold use a tolerance of 2 map units (as the private `TOLERANCE` of
  `build/barrier_gaps.rs` does), so that Linux, macOS and Windows decide alike.
- The plan was compared with the existing code on 2026-11-14; each task names what exists. Existing steps under
  golden tests (`join_touching_lines`, `join_pieces`, `resolve_barrier_overlaps`, `generalize_points`) are used
  as models, not changed, unless a task says so.
- "Canopy" is the 522 roof in this codebase. Trees over the ground are "tree cover" (Baumbedeckung).
- The base map must not change in this plan except through Task 2 (duplicate steps). Every commit that changes a
  drawn map follows the golden-bless flow: `FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden --
  --ignored g3_trace g4_pipeline`, `scripts/golden.sh test`, commit trailer `Golden-Bless:`.
- Checks before every commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked`. Use `CARGO_INCREMENTAL=0`.
- No personal data in requests, file names or fixtures. Live runs use the program's fixed User-Agent.
- Changelog entries go under "Unveröffentlicht" / "Unreleased" (v0.20.0 is released).
- `--pdf-candidates` keeps its name and file; it is the check print, no longer called the training map.

## Order of execution

Tasks 10 and 11 were added after the user's decisions 7 and 8 of the spec. Run the tasks in this order:
1, 2, 3, 4, 5, 10, 11, 6, 7, 8, 9.

## Review Focus

1. A project without terrain, surface model or rock raster (schoolyard, a state without DOM): the training map is
   still written; the steps that need a raster do nothing. Test in Task 4 and Task 5.
2. A symbol set without a code a rule names (ISSprOM has no 502 to 505, the school set no 104): the rule skips,
   nothing panics. Test in Task 1 and Task 4.
3. Lines with a side (earth banks, cliffs, fences) and paths with flags through the join: ticks keep their
   side, a flagged object is left alone, a ring stays closed. Test in Task 3.
4. An existing `<name>-training.omap` without `--force`: kept, the command fails with the map-exists message
   before it writes anything else of the training map. Test in Task 6.
5. Two objects exactly at the gap: the same verdict on every system. Test in Task 1.

---

### Task 1: The conflict measure

**Files:**
- Create: `crates/freemapper/src/build/conflicts.rs`
- Modify: `crates/freemapper/src/build/mod.rs` (declare the module)

**What exists (checked against the code):**
- `crate::render::draw::primitives(object, &RenderSymbols) -> Vec<Primitive>` gives the drawn shapes in paper mm, y
  down (map units / 1000). A dashed line comes as one `Stroke` per dash, so the bare union has holes at the dash
  gaps.
- An object carries a `symbol_id`, and `SymbolSet` has no id → code lookup. The codebase goes code → id with
  `SymbolSet::id_for_code`, which applies the ISOM aliases (`assets/symbol-sets/isom-aliases.toml`: 501.6/501.7 →
  505, 501.8/501.9 → 503, 505.2 → 504, 509.1/509.2 → 509, 532.6 to 532.9 → 532, 513.1/513.2 → 513, 301.6 → 301.4,
  104.1 → 104); a code the set lacks or leaves out gives `None`.
- The gap is `LINE_GAP_MM: SymbolMm = SymbolMm(0.15)` in `build/symbol_measures.rs` (ISSprOM §3.3).
- Code lists to build the classes from: `PATH_CODES` (`build/content.rs`), `CONTOUR_CODES` (`build/line_cuts.rs`,
  101 and 102), `BARRIER_SYMBOLS` (`build/barriers.rs`: 515, 518, 410.1, 513.2), `BANK_CODES` and `FENCE_CODE`
  (`build/ticks.rs`), `FILL_CODE` / `BANK_CODE` / `OSM_AREA_CODE` (`build/water.rs`).
- A uniform grid over segments: `Segments` in `build/barriers.rs`, the grid of `drop_lines_within` in
  `build/dedup.rs`.

**Interfaces (produces):**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class { Building, WaterArea, BankLine, Railway, Road, Path, Watercourse, Stairway,
                 Barrier, Cliff, EarthBank, Contour, Point, OtherArea, Other }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation { Free, Joined, Apart }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade { Overlap, Gap }
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// Index of the object that gives way (lower rank; same rank: the shorter) and of the other.
    pub loser: usize, pub winner: usize,
    pub classes: (Class, Class),
    pub grade: Grade,
    /// Length of the loser's centre line in conflict, mm on paper (0 for areas and points).
    pub length: PaperMm,
}
/// The classes of the symbols of a set: symbol id → class.
pub struct Classes { /* private */ }
impl Classes {
    pub fn of(symbol_set: &SymbolSet) -> Self;
    pub fn class(&self, symbol_id: u32) -> Class;
    /// The ids of the line and area symbols of `class` in this set.
    pub fn ids(&self, class: Class) -> Vec<u32>;
}
/// Per object: its class, its centre lines and its drawn footprint, in paper mm (y down).
pub struct Footprints { /* private */ }
impl Footprints {
    pub fn of(objects: &[&MapObject], symbol_set: &SymbolSet) -> Self;
    pub fn class(&self, i: usize) -> Class;
    pub fn footprint(&self, i: usize) -> &MultiPolygon<f64>;
    /// Half the width of the stroke along the centre line of a line object; 0 for others.
    pub fn half_width(&self, i: usize) -> f64;
    /// Length of the centre line, mm on paper; 0 for areas and points.
    pub fn length(&self, i: usize) -> f64;
}
pub fn rank(class: Class) -> u8;                 // higher wins
pub fn relation(a: Class, b: Class) -> Relation;
pub fn gap(kind: MapKind, scale_denominator: u32) -> PaperMm;
pub fn conflicts(footprints: &Footprints, kind: MapKind, scale_denominator: u32) -> Vec<Conflict>;
pub const MIN_CONFLICT: PaperMm = PaperMm(1.0);
/// Comparisons at a threshold are tolerant by 2 map units.
pub const TOLERANCE_MM: f64 = 0.002;
```

Decisions:
- `Classes::of` resolves a table of `(code, class)` rows through `id_for_code`; a code the set has not is
  skipped, and when two rows give the same id the first row wins. Rows, in this order: 521 Building; 301, 301.1,
  302 (and the water codes of `build/water.rs`) WaterArea; 301.4 and 301.6 BankLine; 509, 509.1, 509.2 Railway;
  502, 503, 504 Road; 505, 506, 507, then the codes of `PATH_CODES` Path (on a forest map 501.8 and 505.2 are
  already Road by their alias: the first row wins); 304, 305, 306 Watercourse; 532, 532.6 to 532.9 Stairway; 513,
  515, 516, 518, then the codes of `BARRIER_SYMBOLS` Barrier; 201, 202 Cliff; 104, 104.1 EarthBank;
  `CONTOUR_CODES` Contour. A symbol id not in the table: Point when `symbol_type` says point symbol, OtherArea when
  area symbol (522 canopies and 501 paved areas among them: ways pass under and over them), else Other.
- Footprint of a line object = the union of its primitives (`Stroke` buffered by half its width with butt caps,
  `Fill` as it is) and of its centre line buffered by `half_width`, which closes the dash gaps. `half_width` is
  half the width of the widest stroke whose points all lie on the centre line. Ticks and borders are in the union.
  Footprint of an area or point object = the union of its primitives. Text draws nothing.
- Rank: Building 9, WaterArea 8, BankLine 8, Railway 7, Road 6, Path 5, Watercourse 4, Stairway 4, Barrier 3,
  Cliff 3, EarthBank 2, Contour 1, Point 0. OtherArea and Other take part in no conflict.
- Relation, one symmetric `match`: any pair with OtherArea or Other is Free. Contour is Free with everything
  except EarthBank, Cliff and Building (Apart). Road, Path, Railway and Stairway among each other are Joined.
  Watercourse with Watercourse, WaterArea or BankLine is Joined. WaterArea with BankLine is Free. Road, Path or
  Railway with WaterArea or BankLine is Free (bridges), and with Building is Free (a network is never cut by a
  building; at forest scale a road at its minimum width touches the houses along it). Building, WaterArea,
  BankLine, Cliff and Contour are Free with their own class. Every other pair is Apart, EarthBank with EarthBank,
  Barrier with Barrier and Point with Point among them.
- `gap` is `LINE_GAP_MM.on_paper(SymbolScale::new(kind, scale_denominator))`.
- Grade Overlap: the loser's centre line inside the winner's footprint grown by the loser's `half_width`. Grade
  Gap: inside that grown by `gap` more, minus the Overlap part. Two lines that cross or meet are no conflict
  there, whatever their relation: the neighbourhood of every point where the two centre lines meet or cross is
  left out, radius = both half widths + `gap`. A stretch counts from `MIN_CONFLICT`. A point or an area as loser:
  Overlap when the footprints intersect, Gap when they are closer than `gap`; length 0.
- At a threshold the verdict is "no conflict" within `TOLERANCE_MM`.
- One `Conflict` per pair and grade. Pairs come from a uniform grid over the bounding boxes, not from all pairs.

- [ ] **Step 1: Write the failing tests** in `conflicts.rs` (`#[cfg(test)]`; fixture as the tests of
  `build/line_cuts.rs` build theirs; ISOM set `SymbolSet::isom_2017_2` at 1:10 000 unless said):
  - `an_earth_bank_on_a_ditch_is_an_overlap`: a 306 line of 20 mm and a 104 line parallel 0.2 mm beside it give one
    conflict, `loser` the 104, `grade == Overlap`, `length` within 1 mm of 20 mm.
  - `an_earth_bank_beside_a_ditch_is_a_gap`: the same with the footprints 0.1 mm apart gives `Gap`; 1 mm apart
    gives none.
  - `two_paths_that_cross_are_no_conflict` and `two_paths_side_by_side_are`: 505 crossing at right angles: none;
    505 parallel 0.3 mm apart over 10 mm: one, about 10 mm.
  - `a_dashed_path_has_no_holes`: a fence 518 of 10 mm lying on a 505 gives one conflict of about 10 mm, not
    several short ones.
  - `a_path_over_water_is_free`, `a_contour_under_a_path_is_free`, `a_contour_through_a_building_is_an_overlap`.
  - `a_ditch_that_crosses_a_path_is_no_conflict` (right angle and 30°); `a_road_along_a_house_is_free`;
    `row_houses_are_free`; `a_path_under_a_canopy_is_free`.
  - `a_point_on_a_building_is_an_overlap` (526 inside a 521).
  - `the_verdict_at_the_gap_is_tolerant`: two lines whose footprints are exactly `gap` apart, and the same moved
    by 1 map unit either way, give no conflict.
  - `the_classes_follow_the_aliases`: with the ISOM set the id of 501.8 is Road and that of 501.6 is Path; with
    `SymbolSet::issprom_2019()` the ids of 501.6 and 501.8 are Path, of 532.6 Stairway, of 509.1 Railway, of
    301.6 BankLine.
  - `a_set_without_the_code_gives_no_conflict`: the school set `SymbolSet::school_2019()` and an empty object
    list, then objects of ids the table does not know: empty result, no panic.
  - `the_relation_is_symmetric`: for all pairs of `Class`, `relation(a, b) == relation(b, a)`.
- [ ] **Step 2: Run** `cargo test -p freemapper conflicts::` and see them fail to compile.
- [ ] **Step 3: Implement** the module as specified.
- [ ] **Step 4: Run** the tests and `cargo clippy --workspace --all-targets -- -D warnings`, see them pass.
- [ ] **Step 5: Commit** `feat(build): a conflict measure for drawn symbols`.

### Task 2: Repair the duplicate steps (water, stairs)

**Files:**
- Modify: `crates/freemapper/src/build/water.rs` (`drop_covered_water`, `projected_area`),
  `crates/freemapper/src/build/assembly.rs` (`resolve`: the call in `off_water`, and a new stair step),
  `crates/freemapper/src/build/dedup.rs` or `build/stairs.rs` (the new stair comparison).

**What exists (read, not yet run on the data):**
- `drop_covered_water` is called in `resolve` → `off_water` (`assembly.rs`). `projected_area` (`water.rs`) puts all
  coordinates of the object into one ring. An OSM multipolygon relation is written as an outer ring plus hole
  rings (`build/candidates.rs`), so a lake with an island becomes a self-overlapping ring, `!area.is_valid()` is
  true and the guard keeps the lake. The doc comment "no holes: OSM areas come from closed ways" is out of date.
  `build::geometry::object_polygon` followed by `points::projected` reads the rings properly.
- The `union` passed in is `WaterObjects::union`: only the drawn pieces of the wide ALKIS water parts (mean width
  at least 2 m), clipped, slivers removed. Narrow "Graben" parts are not in it; a lake lying on those is not
  covered.
- OSM stairs are never compared with ALKIS stairs. On a forest map OSM steps have `target = "map"`
  (`osm-candidates.toml`), so they are in `osm_map`, which `drop_osm_lines_near_cadastre` never sees. The only
  stair comparison is survey against ALKIS by area share (`DUPLICATE_STAIR_SHARE` 0.5, `build/stairs.rs`).
  `add_also` (`build/dedup.rs`) writes `freemapper:also`.

Measured on Bremen Bürgerpark (forest, 1:10 000): five OSM `natural=water` relations (`relation/372786`,
`3046999`, `4505380`, `5482048`, and `3101282` at only 19 %) against ALKIS water; 6 of 23 OSM `highway=steps`
within 0.3 mm of an ALKIS "Treppe" 532.

- [ ] **Step 1: Failing test, water:** next to the existing `drop_covered_water` tests:
  `an_osm_lake_with_an_island_on_alkis_water_is_dropped` (outer ring and one hole ring, the union covering the
  lake) and `…_beside_alkis_water_is_kept`.
- [ ] **Step 2: Fix** `projected_area` to read the rings; update the doc comment. Run the water tests.
- [ ] **Step 3: Confirm on the data.** Build Bremen in scratch (`freemapper init <scratch>/bb --osm way/664446849
  --kind forest --scale 10000 --lang en`, `run`); count the OSM 301 objects that lie more than half on 301.1. If
  lakes are left because their ALKIS water is narrow, pass the union of all drawn ALKIS water areas to the step
  (wide and narrow) and add a test for it; write what was found into the commit message.
- [ ] **Step 4: Failing test, stairs:** `an_osm_stair_on_a_cadastre_stair_is_dropped` (an OSM stair line with at
  least half its length within the drawn width of a cadastre stair object: gone, and the cadastre stair carries
  its id in `freemapper:also`); `an_osm_stair_that_only_crosses_one_stays`; `…_on_a_map_without_cadastre_stairs
  stays`.
- [ ] **Step 5: Implement** the comparison as a step of `resolve`, for the OSM stairs of the map and of the
  candidates, share 0.5 as `DUPLICATE_STAIR_SHARE`.
- [ ] **Step 6: Bless the goldens** if they change, run all checks.
- [ ] **Step 7: Commit** `fix(build): OSM water and stairs that double the cadastre are dropped`.

### Task 3: Join lines

**Files:**
- Create: `crates/freemapper/src/build/line_join.rs`; declare in `build/mod.rs`.

**What exists:** two joins, neither on finished map objects of any symbol: `join_touching_lines`
(`build/candidates.rs`: on `LatLon` features, skips OSM, 0.1 m, at most 30° at a joint, keeps sides with
`line_side` / `SIDED_CODES`, tags through `chain_tags` → `add_also`) and `join_pieces` (`build/cliffs.rs`: end to
start, closest first, 1 m, 60°, closes rings). Neither is changed here (their output is under golden tests); the
new function takes the method of `join_pieces` and the side rule of `SIDED_CODES`. No source writes curve flags.

**Interfaces (produces):**

```rust
/// Joins the path objects of each symbol id in `ids` that meet end to end. Returns how many objects went.
pub fn join_lines(objects: &mut Vec<MapObject>, ids: &[u32], sided: &[u32], georef: &Georeferencing) -> usize;
/// Ends closer than this touch (ground metres), as `JOIN_TOUCH_M` in `build/candidates.rs`.
pub const TOUCH_M: f64 = 0.1;
/// At a junction of three or more ends the straightest pair is joined when it turns less than this.
pub const MAX_TURN_DEG: f64 = 30.0;
```

Decisions: only objects of the same symbol id join. Where exactly two ends meet, they join whatever the angle;
where more meet, the pair with the smallest turn joins if under `MAX_TURN_DEG`, the others stay. An object whose
symbol id is in `sided` (ticks on one side: 104, 201, 513.2, 518 and what `SIDED_CODES` names) is never reversed:
it joins only end to start; others may be reversed. An object with a coordinate flag other than the closing one
is left alone. The joined object keeps the tags of the longer one and gets the `freemapper:source_id` of the
others through `add_also`. A chain that closes on itself becomes a closed path. The joined object takes the place
of its first member.

- [ ] **Step 1: Failing tests:** `two_pieces_become_one` (3 + 4 points → 6 points, one object, `also` set);
  `three_ends_join_the_straight_pair` (a T: the bar is one object, the stem stays); `a_fork_of_60_degrees_joins
  nothing`; `different_symbols_do_not_join`; `a_ring_of_pieces_is_closed`; `a_sided_line_is_not_reversed` (two
  104 pieces head to head stay two; head to tail become one with the direction kept); `an_unsided_line_may_be
  reversed`; `an_object_with_flags_is_left_alone`; `ends_one_metre_apart_do_not_join`.
- [ ] **Step 2: Run and see them fail. Step 3: Implement. Step 4: Run and pass**, clippy.
- [ ] **Step 5: Commit** `feat(build): lines of one symbol that meet end to end are joined`.

### Task 4: The training level (join, give way, cut, minimum sizes)

**Files:**
- Create: `crates/freemapper/src/build/training.rs`; declare in `build/mod.rs`.
- Modify: `crates/freemapper/src/omap/symbol_set.rs` (getter for the minimum length).

**What exists:**
- `cut_lines_at_ways(objects, codes: &[&str], ways: &[Polygon<f64>], min_piece_m: f64, georef, symbol_set)`
  (`build/line_cuts.rs`) cuts lines of the given codes at polygons. The polygons are in **projected metres**, the
  minimum piece in **ground metres**; it drops flags. `build::geometry::object_polygon` gives an object's polygon
  in map units, `build::points::projected` turns it into projected metres.
- Contour rings on buildings are already dropped (`build/contours.rs`); open contours are not cut at buildings.
  Minimum contour length: `MIN_LENGTH_MM = SymbolMm(2.0)` (`derive/contours.rs`).
- Candidate points other than trees inside a building are already dropped on sprint and forest maps
  (`build/points.rs`); map points are not.
- `resolve_barrier_overlaps` (`build/barriers.rs`) is the precedent for giving way among barriers: pieces of at
  least `PIECE_MIN_MM = SymbolMm(1.0)`. It runs in `resolve` only.
- The symbol's minimum length is in the set's XML (`minimum_length`, ISOM 104 and 201: 600) and already enlarged
  by `omap/enlarge.rs`; nothing reads it. Existing minimums elsewhere: `MIN_LINE_MM` 1.0 mm on paper
  (`build/candidates.rs`), `norm_min_line_mm` (509.1: 4 mm, ISSprOM 4.5).
- The rock step of the training map (`training_rock` in `project/export.rs`) reads its own rasters and stays
  where it is; Task 6 calls it before `train`.

**Interfaces:**
- Consumes: Task 1 (`Classes`, `Footprints`, `conflicts`, `Class`, `Grade`, `gap`), Task 3 (`join_lines`).
- Produces:

```rust
// omap/symbol_set.rs
/// The minimum length of the line symbol `id` as the set gives it (already enlarged); `None` when 0 or unknown.
pub fn minimum_length(&self, id: u32) -> Option<PaperMm>;

// build/training.rs
/// What the training level did, for the summary line and the ADR.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TrainingReport {
    pub joined: usize, pub gave_way: usize, pub cut: usize, pub too_small: usize,
    pub open_land_cut: usize,
    /// Conflict length left, mm on paper, by grade (the pairs of one network, left to the second half).
    pub overlap_left: f64, pub gap_left: f64,
}
/// The objects of the training map from the objects of the map and the visible candidates.
/// `tree_cover` is the raster of Task 5 (share of ground under trees per cell), when there is one.
pub fn train(objects: Vec<MapObject>, tree_cover: Option<&Raster<f32>>, kind: MapKind,
             georef: &Georeferencing, symbol_set: &SymbolSet) -> (Vec<MapObject>, TrainingReport);
```

Order of steps inside `train`, each a private function with its own tests:
1. `join_lines` on the ids of the classes Road, Path, Railway, Watercourse, EarthBank, Barrier and Contour;
   `sided` = the ids of EarthBank, Cliff and Barrier.
2. Open contours are cut at the Building objects (`cut_lines_at_ways` with `CONTOUR_CODES`, minimum piece
   `MIN_LENGTH_MM` of `derive/contours.rs` in ground metres).
3. (Task 5 puts its step here.)
4. Give way: for the conflicts of Apart pairs of both grades (the training map keeps every gap), losers by
   ascending rank: a loser with more than half its length in conflict goes whole; otherwise the stretch is cut
   out with `cut_lines_at_ways` at the winner's footprint grown by the loser's half width plus `gap` (converted
   to projected metres), pieces below the loser's minimum length go. Points and areas that lose go whole. Among
   objects of one class the shorter loses. Dropped objects are gone from the training map; nothing is tagged
   hidden. The step repeats until `conflicts` finds no Apart pair (each round removes geometry, so it ends).
5. Minimum sizes: a line shorter than its minimum length goes: `SymbolSet::minimum_length` where the set gives
   one, else `MIN_LINE_MM` of `build/candidates.rs`; stairs take the norm's value for 532, quoted in the ADR. A
   line with an end within `TOUCH_M` of another line of its class continues there and stays.
6. The report: counts, and the conflict length left by `conflicts` on the result.

- [ ] **Step 1: Failing tests** (ISOM set, 1:10 000, no raster):
  - `an_earth_bank_on_a_ditch_goes`: 306 of 20 mm and 104 on it → the 104 is gone, the 306 stays, `gave_way == 1`.
  - `an_earth_bank_inside_the_gap_of_a_ditch_goes_too`: footprints 0.1 mm apart → the 104 is gone.
  - `an_earth_bank_that_crosses_a_path_stays_whole` (right angle and 30°).
  - `an_earth_bank_that_runs_into_a_path_is_cut`: 104 of 20 mm, its last 6 mm along a 505 inside the gap → one
    104 of about 14 mm, `cut == 1`.
  - `a_fence_on_a_path_goes_where_it_lies_on_it`: 518 of 20 mm, 8 mm of it on a 505 → one 518 of about 12 mm.
  - `of_two_earth_banks_inside_the_gap_the_shorter_goes`.
  - `a_point_on_a_building_goes`; `a_point_beside_it_stays`.
  - `a_contour_stops_at_a_building`; `a_contour_on_an_earth_bank_goes`.
  - `a_stair_below_the_minimum_goes`; `a_short_path_that_continues_stays`.
  - `pieces_of_a_path_are_one_object_after` (5 pieces of 505 in a row → 1, `joined == 4`).
  - `carriageways_and_tracks_are_left_alone` (two parallel 502, two parallel 509: unchanged).
  - `no_apart_conflict_is_left`: a mixed scene (bank, ditch, fence, path, building, point) → `conflicts` on the
    result has no Apart pair of either grade.
  - `the_school_set_does_not_fail` (`SymbolSet::school_2019()`, a few objects, no raster).
  - `the_report_counts_what_is_left`: two parallel 505 inside the gap → `gap_left + overlap_left > 0`; after
    `an_earth_bank_on_a_ditch_goes` both are 0.
  - In `omap/symbol_set.rs`: `the_minimum_length_of_an_earth_bank_is_read` (ISOM at 1:10 000: 0.6 mm times the
    enlargement the set already applied; a symbol with 0 gives `None`).
- [ ] **Step 2: Run and see them fail. Step 3: Implement. Step 4: Run and pass**, clippy.
- [ ] **Step 5: Commit** `feat(build): the training level joins, gives way and cuts`.

### Task 5: Open land under trees

**Files:**
- Modify: `crates/freemapper/src/project.rs` (the derive step around the nDSM, and `TREE_COVER_RELPATH`),
  `crates/freemapper/src/derive/vegetation.rs` (make the outline helper reachable), `build/training.rs` (step 3),
  `docs/glossary.md` (term).

**What exists:**
- "Canopy" means the 522 roof in this codebase (`build/canopy.rs`, glossary: "Nicht Baumkrone!"). The new thing
  is called **tree cover** (Baumbedeckung) everywhere: `TREE_COVER_RELPATH = "data/tree-cover.tif"`.
- The float nDSM exists only inside derive, merged from three inputs (nDSM LAZ tiles, a DSM tile mosaic, the
  per-area surface model at `SURFACE_RELPATH`) on a 0.2 m grid: `derive/ndsm.rs::rasterize`, `fill_from_dsm`; in
  `project.rs` the merged raster is complete just before `write_ndsm_png` is called. The PNG is a colour ramp and
  cannot be read back. `data/rock-seen.tif` shows how derive writes a float raster that the export reads with
  `derive::terrain::read_float_raster`.
- Smoothing and minimum area of raster classes (ADR 0136) are in `derive/vegetation.rs`: `majority`, the private
  `outlines`, `min_area_m2` (`MIN_AREA_MM2 = SymbolMm2(2.25)`). Slivers: `LandClip::is_sliver` / `short_side`
  (`build/out_of_bounds.rs`), `MIN_PIECE_SIDE_MM` 1.0 (`build/open_land.rs`).
- An area from the land use carries `freemapper:nutzart = "<nutzart> / <bez>"` (`build/out_of_bounds.rs`). A
  park never gives 403. Today a park on a forest map is cut only by ALKIS Wald/Gehölz and by OSM forest polygons
  (`build/open_land.rs`); the Stadtwald has neither.

Decisions: where derive has an nDSM it also writes the tree-cover raster: 1 m cells, value = share of the 0.2 m
cells in it that are at least 3 m high (ruling: what hides the ground of an open area from a runner). Without an
nDSM no file is written and an old one is removed. In `train`, a 401 object whose `freemapper:nutzart` starts
with "Sport-, Freizeit- und Erholungsfläche" (Park, Grünanlage) or whose `freemapper:osm` holds `leisure=park` is
reduced to its open ground: cells with a share below 0.5 are open; the open mask is smoothed with `majority` and
outlined as the vegetation classes are, intersected with the object, pieces below `min_area_m2` or thinner than
`MIN_PIECE_SIDE_MM` go. The pieces keep the tags of the object and get `freemapper:generalized=tree cover`. Every
other object, and every object when `tree_cover` is `None`, stays as it is.

- [ ] **Step 1: Failing tests:** in `training.rs`: `a_park_keeps_only_its_open_half` (a 401 square tagged
  "Sport-, Freizeit- und Erholungsfläche / Park", raster with share 1 on its left half and 0 on the right → one
  401 over the right half, `open_land_cut == 1`); `a_meadow_of_another_use_is_left`; `without_tree_cover_nothing
  changes`; `a_glade_below_the_minimum_goes`. In the derive tests of `project.rs`:
  `derive_writes_the_tree_cover_where_there_is_an_ndsm` and `…_writes_none_without_one`.
- [ ] **Step 2 to 4:** fail, implement, pass.
- [ ] **Step 5: Commit** `feat: open land under trees stays forest on the training map`.

### Task 6: The training map files

**Files:**
- Modify: `crates/freemapper/src/project/export.rs`, `crates/freemapper/src/project.rs` (re-exports, tests).

**What exists:** `pdf_objects` (its first list holds the walkable-area part `BASE_PART` too; only `SHEET_PART` is
split off), `write_pdf_file` (calls `training_rock` when `candidates`), `training_rock(dir, kind, objects,
documents) -> Vec<MapObject>`, `export_ocad_from(dir, documents, force) -> Result<OcadOutcome, _>`, `pdf_path`,
`back_up`, `write_document`, `write_inside`, `info_block`, `ProjectError::MapExists`, `read_float_raster`.

**Interfaces:**
- Consumes: `train`, `TrainingReport`, `TREE_COVER_RELPATH`.
- Produces:

```rust
pub struct TrainingOutcome { pub map: PathBuf, pub pdf: PathBuf, pub ocad: Option<PathBuf>,
                             pub report: TrainingReport }
pub fn training_map_path(dir: &Path) -> PathBuf;   // <stem>-training.omap
pub fn training_pdf_path(dir: &Path) -> PathBuf;   // <stem>-training.pdf
/// Writes the training map, its PDF and, with `ocad`, `ocad/<stem>-training.ocd`. An existing training map
/// is kept unless `force` (then `<stem>-training.omap.bak`).
pub fn export_training_from(dir: &Path, documents: &StarterDocuments, force: bool, scale_bar: bool,
                            sources: PdfSources, ocad: bool) -> Result<TrainingOutcome, ProjectError>;
pub fn export_training(dir: &Path, force: bool, scale_bar: bool, sources: PdfSources, ocad: bool)
                       -> Result<TrainingOutcome, ProjectError>;
```

Decisions: the check for an existing training map comes first and returns `ProjectError::MapExists(path)` before
anything is written. The input is the objects of the map's parts other than `BASE_PART` and `SHEET_PART` plus
the candidates that are not hidden (`HIDDEN_KEY`), as `pdf_objects(documents, true)` takes them; then
`training_rock` (forest and sprint, as today), then `train` with the raster at `TREE_COVER_RELPATH` if it reads.
The training document has the parts of the map in their order (walkable area if any, `default part` with the
trained objects, the sheet part), the notes of the map and its templates without those of `TemplateKind::Map`.
The PDF is drawn from the same objects by the code `write_pdf_file` uses (shared, not copied). The OCAD file goes
through the writer `export_ocad_from` uses. `write_pdf_file` with `candidates == true` no longer calls
`training_rock`: the check print shows the map level; the doc comments of `export_candidates_pdf*` say "check
print".

- [ ] **Step 1: Failing tests** in `project.rs` beside `candidates_pdf_draws_map_and_candidates`:
  `the_training_map_is_written_beside_the_map` (both files exist, the `.omap` parses, lists no template of kind
  map, holds the sheet part and, on a sprint map, the walkable area once); `the_training_map_takes_map_and
  candidates` (a fixture with a candidate fence: it is in the training map); `an_existing_training_map_is_kept`
  (second call without force → `MapExists`, both files unchanged; with force → `.omap.bak`);
  `the_training_map_goes_to_ocad_on_request`; `the_candidates_pdf_is_the_map_level_again` (adapt the PDF tests
  that relied on the training rock, such as `training_map_takes_all_but_the_outline`).
- [ ] **Step 2 to 4:** fail, implement, pass; goldens blessed if the candidates PDF golden changes.
- [ ] **Step 5: Commit** `feat: the training map is a map file and a PDF of its own`.

### Task 7: The switch

**Files:** every place `pdf_candidates` is threaded through gets a `training` beside it:
`crates/freemapper-cli/src/main.rs` (`run`, `export`: `--training`), `pipeline.rs` (`PipelineOptions`, the request,
`Step::Training`, the result files of kind `training-map` and `training-pdf`, the summary), `mcp.rs`
(`start_pipeline`, `open_area_selector`: `training`), `select.rs`, `select_page.rs`, `web/index.html`,
`web/run.js` (the box "Trainingskarte" / "Training map"), `i18n/de/freemapper.ftl`, `i18n/en/freemapper.ftl`, and
the tests `tests/cli.rs`, `tests/mcp.rs`, `src/select_run/tests.rs`.

Decisions: `--training` is independent of `--pdf` and `--pdf-candidates`; `--pdf-scale-bar` and `--pdf-sources`
apply to its PDF; `--force` applies to it as to the map; with `--ocad` the training map goes to OCAD too. Not
stored in `freemapper.toml`. Messages: "Wrote <path> (the training map: <n> joined, <n> left out or cut)" from
the report; when the training map exists and `--force` is missing, the map-exists message names the training
map. Texts for `--pdf-candidates` change from "the training map" to "the check print: the map with every
candidate, true to position" in help, MCP descriptions and i18n.

- [ ] **Step 1: Failing tests:** CLI: `run --training` writes both files and prints the line; `export --training`
  on an existing training map without `--force` exits 1 with the message; MCP: `start_pipeline` with `training:
  true` lists both files in `files`; the page: the box is in the form and reaches the run options.
- [ ] **Step 2 to 4:** fail, implement, pass.
- [ ] **Step 5: Commit** `feat(cli): --training writes the training map`.

### Task 8: Docs, ADR, changelog

**Files:** `docs/glossary.md`, `docs/adr/0140-the-training-map-is-a-product-of-its-own.md` (0137 to 0139
belong to other branches) and `docs/adr/README.md`, `AGENTS.md`, `README.md`, `README.en.md`, `docs/manual/de.md`, `docs/manual/en.md`,
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
- [ ] **Step 3:** One run each of freemapper-print and freemapper-courseplan on the Bremen training map (they
  take a map path and read `freemapper.toml` beside it): it loads, the sheet is found. Tell their sessions.
- [ ] **Step 4:** Full checks, push the branch to GitHub, open the pull request (label `all-systems`: file names
  and paths), wait for the user's yes.

### Task 10: Inside olive green

**Files:**
- Modify: `crates/freemapper/src/build/training.rs` (a step after the join and the contour cut, before "give way").

**What exists:** points under a 520 candidate are hidden (`HIDDEN_KEY = 520`, `build/points.rs`) and never reach
`train`; vegetation is cut at the 520 areas it is placed with (`cut_vegetation_at_out_of_bounds`,
`build/vegetation.rs`). `cut_lines_at_ways` cuts lines at polygons in projected metres. The 520 objects of the map
and of the candidates are both in the input of `train`. The impassable barriers are 515 (wall) and 518 (fence).

**Interfaces:** consumes `Classes`, `Footprints`, `gap` (Task 1), `train` and `TrainingReport` (Task 4). Produces a
field `pub inside_olive: usize` in `TrainingReport` (objects that went or were cut by this step) and

```rust
/// How far inside the olive green an object still lies on its edge.
pub const OLIVE_EDGE_MM: PaperMm = PaperMm(0.4);
/// The beginning drawn of a way that allows no crossing: this, or the symbol's minimum length if longer.
pub const WAY_STUB_MM: PaperMm = PaperMm(1.0);
```

Decisions: the olive area is the union of the objects of the symbol 520; `inner` is that union shrunk by
`OLIVE_EDGE_MM`. Stay untouched: Building, WaterArea, BankLine, Watercourse, Contour, the 520 objects, canopies
(522), text and sheet objects. Areas of other symbols lose their part inside `inner`; pieces below the minimum the
open-land step uses go. Points inside `inner` go. Lines of the classes Barrier, EarthBank, Cliff and Stairway are
cut at `inner` (pieces below their minimum length go). Ways (Road, Path): the parts inside `inner` are taken
together as they join; such a group is a crossing when it meets the boundary of `inner` at two places or more
that are not barred. A place is barred when a 515 or 518 object crosses the way within `OLIVE_EDGE_MM` of the
boundary. A crossing stays in full. Of a group that is no crossing, the part inside goes, except the first
`WAY_STUB_MM` (or the symbol's minimum length) of the way inward from each place that is not barred. Railways are
left alone. Access tags are not read: the rule is geometry only (ruling; OSM access tags are not carried by the
objects).

- [ ] **Step 1: Failing tests** (ISSprOM set at 1:4000 and ISOM at 1:10 000 where it matters):
  `a_fence_inside_goes_and_the_one_on_the_edge_stays`; `a_point_inside_goes`; `a_building_and_a_pond_inside_stay`;
  `a_contour_runs_through`; `open_land_is_cut_at_the_edge`; `a_path_that_crosses_stays_in_full` (enters on one
  side, leaves on the other); `a_driveway_keeps_only_its_beginning` (dead end: one piece of `WAY_STUB_MM` from
  the edge); `a_driveway_behind_an_impassable_fence_goes_whole` (518 across its entrance: no stub);
  `a_crossing_barred_at_one_end_is_no_crossing` (one entrance free → stub there, nothing at the barred end);
  `a_passable_fence_does_not_bar` (a 516 across the entrance: the stub is drawn); `without_olive_nothing
  changes`.
- [ ] **Step 2 to 4:** fail, implement, pass, clippy.
- [ ] **Step 5: Commit** `feat(build): no details inside olive green on the training map`.

### Task 11: Tick sides

**Files:**
- Modify: `crates/freemapper/src/build/ticks.rs`; `docs/adr/0097-…` gets no edit (the new ADR of Task 8 amends it).

**What exists:** `orient_ticks(lists, terrain, georef, symbol_set)` is the last step on the lines in `Built::place`,
for the map and the candidates. A fence (518) turns its ticks towards an olive green area beside it, else a
closed fence has them inside, else towards a park (401) beside it. Banks, cliffs and retaining walls (513.2) point
down the slope. Lines with curves or dashes are left alone. The ticks sit on the right of the line. Norm: 516/518
"If the fence forms an enclosed area, tags should be placed inside"; 513.2 "Half dot must point to the lower
level".

Decisions (the user's rule: the side follows the meaning; turning is no means against overdrawing unless the
same area lies on both sides):
- The order of the votes for a fence: olive green beside it; closed → inside; **away from a way** (a Road or Path of
  `conflicts::Classes` running along one side within the reach of the ticks plus the gap for at least half the
  fence's length, and not along the other); park beside it.
- When no vote decides, the same area lies on both sides: the ticks go to the side where their footprint (ticks
  only, without the line) meets no footprint of another object whose relation to Barrier is not Free; when both
  sides or neither are free, the direction of the source stays.
- A retaining wall, a bank and a cliff are decided by the terrain as today and never by a way or by room.
- Where the decided side overdraws something, it stays.
- The signature of `orient_ticks` stays; it applies to every map, so the goldens change: bless them.

- [ ] **Step 1: Failing tests** beside the existing ones in `ticks.rs`: `a_fence_along_a_path_turns_away_from_it`;
  `olive_green_wins_over_a_path_on_the_same_side` (olive green and a path both on the left: ticks left);
  `a_closed_fence_keeps_its_ticks_inside_with_a_path_inside`; `a_fence_between_two_paths_keeps_its_direction`;
  `in_the_forest_the_side_with_room_is_taken` (a boulder point on the right, nothing left: ticks left);
  `in_the_forest_with_no_room_the_direction_stays`; `a_retaining_wall_is_not_turned_by_a_path`;
  `the_side_given_by_olive_green_stays_when_its_ticks_overdraw` (olive green left, a point in it at the fence:
  ticks left).
- [ ] **Step 2 to 4:** fail, implement, pass; bless the goldens; all checks.
- [ ] **Step 5: Commit** `feat(build): fence ticks turn away from a way; with the same area on both sides they take the free side` with the `Golden-Bless:` trailer.
