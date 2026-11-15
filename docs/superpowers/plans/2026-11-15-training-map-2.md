# Training map, second half: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the training level merges what is alike, shifts before it cuts, keeps junctions, and draws the Berlin
carriageways in 50 % brown, so that joined pairs are settled and fewer walls and fences are lost.

**Architecture:** three new steps in `build/training/` (merge, shift, junctions) around the existing give way,
plus a carriageway step; one new geometry module for middle lines and offsets with sqrt-only arithmetic. The
assembly passes the Berlin carriageway surfaces as hidden candidates.

**Tech Stack:** Rust, `geo` 0.33 (`LineLocatePoint`, `LineInterpolatePoint`, `Simplify`, `Buffer`), the measure
types of `measure.rs`, `build/conflicts.rs`.

**Spec:** `docs/superpowers/specs/2026-11-14-training-map-design.md`, section "The second half, as designed on
2026-11-15", and Part 3; ADR 0140.

## Global Constraints

- English source and comments; measures as `PaperMm`/`SymbolMm`; norm values quoted with symbol number.
- Deterministic on Linux, macOS and Windows: no `acos`/`atan2`/`tan`/`hypot` in a decision; `sqrt` only; sorted
  iteration; tolerance 2 map units (`TOLERANCE_MM`). The training map is in the goldens (g4) of all four states.
- The base map changes only through Task 5 (hidden carriageway candidates in Berlin); bless with a
  `Golden-Bless:` trailer wherever goldens change (the training map hashes will change in every task that alters
  the training level — bless them in that task).
- Bounds: a line shifts at most 0.5 mm on paper (`MAX_SHIFT_MM`, exists in `training/points.rs`).
- `export CARGO_INCREMENTAL=0`; no shared CARGO_TARGET_DIR; checks fmt, clippy `-D warnings`, focused tests, once
  `cargo test --workspace --locked` with its exit code; `scripts/golden.sh test`; `check-golden-untouched.sh`.
- Commit trailers: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and the Claude-Session line.

## Review Focus

1. A merge or shift next to a point symbol already moved: no new conflict (Task 3/4 tests).
2. A shifted earth bank onto a contour: refused (Task 4).
3. A dual carriageway that splits at a junction: the merged middle stops where the pair parts; side roads stay
   connected (Task 3 with Task 2).
4. Sided lines (fences, earth banks): direction and tick side kept after shift and merge (Task 4).
5. Sprint maps: trams on street areas are untouched; paths beside 501.5 areas are not "inside a road" (Task 3).

## Order of execution

1, 2, 3, 4, 5, 6, 7.

---

### Task 1: Middle lines and offsets

**Files:** create `crates/freemapper/src/build/training/lines.rs` (declare in `training/mod.rs`).

**What exists:** `olive::Polyline` (private: `new`, `length`, `point(s)`, `locate(c)`, `between(a, b)`),
`render::geometry::offset` (uses `acos`/`tan`: do not use in decisions), `cliffs::resample`/`smooth` (private),
`geo` `LineLocatePoint`/`LineInterpolatePoint`/`Simplify`.

**Produces (paper mm, y down, as `conflicts::Footprints` uses):**

```rust
/// A polyline with arc-length access; moved here from olive.rs and made pub(super) (olive uses it from here).
pub(super) struct Polyline { .. }
/// The stretch of `a` that lies within `within` mm of `b`, as arc-length ranges on `a` (sorted, merged, each
/// at least `min_len` long).
pub(super) fn beside(a: &Polyline, b: &Polyline, within: f64, min_len: f64) -> Vec<(f64, f64)>;
/// The middle of `a` and `b` over `range` of `a`: samples every `step` mm on `a`, each paired with the
/// closest point of `b`, the midpoints simplified with tolerance `step / 4`.
pub(super) fn middle(a: &Polyline, b: &Polyline, range: (f64, f64), step: f64) -> Vec<(f64, f64)>;
/// `line` with the stretch `range` moved by `shift` mm to the side `side` (+1 right, -1 left of the
/// direction), with linear ramps of length `ramp` before and after; points are resampled every `step` in the
/// moved part. Normals from neighbouring points with sqrt only.
pub(super) fn shifted(line: &Polyline, range: (f64, f64), shift: f64, side: f64, ramp: f64, step: f64) -> Vec<(f64, f64)>;
```

- [ ] Tests first: `beside_finds_the_parallel_stretch` (two 20 mm lines 0.3 mm apart overlapping 12 mm → one
  range of ~12 mm); `middle_of_two_parallels_lies_between` (midpoints 0.15 mm from each); `shifted_moves_only
  _the_stretch` (ramp ends unmoved, plateau moved by `shift` within 0.002); `shifted_keeps_a_closed_line_closed`;
  `the_same_input_gives_the_same_points` (bitwise equal twice).
- [ ] Move `Polyline` from olive.rs, adapt olive.rs; implement; clippy; commit `feat(build): middle lines and
  offsets for the training map`.

### Task 2: Junctions

**Files:** create `crates/freemapper/src/build/training/junctions.rs`.

**Produces:**

```rust
/// Line ends that touch another line, recorded before a change.
pub(super) struct Junctions { .. }
impl Junctions {
    /// Every end of a line object (classes Road, Path, Railway, Stairway, Barrier, Watercourse, EarthBank) that
    /// lies within TOUCH_M (ground metres, as line_join) of another line object, keyed by a stable object key
    /// (`freemapper:source_id` + symbol id + index of first appearance).
    pub(super) fn record(objects: &[MapObject], classes: &Classes, georef: &Georeferencing) -> Self;
    /// After a change: every recorded end whose object still exists but no longer touches a line is moved to the
    /// nearest point of the line that touched it or of the line that replaced it (`replaced_by`), when that point
    /// is within `reach` mm; returns (repaired, still free).
    pub(super) fn repair(&self, objects: &mut [MapObject], replaced_by: &BTreeMap<Key, Key>, reach: f64,
                         georef: &Georeferencing) -> (usize, usize);
}
```

Decisions: the end moves (its last vertex is replaced), the touched line does not; a sided line keeps its
direction; ends of closed lines are not recorded.

- [ ] Tests: `an_end_follows_a_shifted_line`, `an_end_moves_to_the_replacing_middle_line`, `an_end_too_far
  _stays_free_and_is_counted`, `a_closed_line_has_no_ends`.
- [ ] Implement; commit `feat(build): line ends keep their junctions on the training map`.

### Task 3: Merge what is alike

**Files:** `crates/freemapper/src/build/training/merge.rs` (new); `training/mod.rs` (call it after `clear_olive`,
before `move_points`; rerun `join`).

Decisions (spec "Merge"): pairs of the same class Road, Railway or Path whose conflicts (`conflicts::conflicts`,
relation Joined) are of length ≥ the class's minimum length; the stretch via `lines::beside` with `within` =
both half widths + `gap`; replace the stretch of both lines by `lines::middle` (the pieces of both outside the
stretch stay and are connected to the middle's ends); the kept symbol is the wider (`Footprints::half_width`), on
a tie the longer object's; tags of the longer, the other's id in `freemapper:also`,
`freemapper:generalized=merged`. A Railway with `freemapper:osm` containing `railway=tram` within a Road's drawn
width (forest map only) goes; a Path within a Road's drawn width over more than half its length goes, else the
stretch is cut (as give way cuts). `Junctions::record` before, `repair` after with `replaced_by`. Report fields
`merged`, `dropped_beside_roads`, `junctions_repaired`, `junctions_free`.

- [ ] Tests (ISOM 1:10 000 and ISSprOM 1:4000): `two_carriageways_become_one_road`; `two_tracks_become_one_line`;
  `a_sidewalk_beside_a_road_goes` (forest); `a_tram_in_a_road_goes_on_the_forest_map_only`; `a_side_road_stays
  _connected_to_the_merged_road`; `a_pair_that_parts_merges_only_where_it_runs_together`; `paths_beside_a_street
  _area_are_left_on_the_sprint_map`; `nothing_new_conflicts_with_a_moved_point` (merge runs before points, so the
  points step sees the merged line — assert via `train`).
- [ ] Update `the_report_counts_what_is_left` (parallel 505 now merge).
- [ ] Bless goldens (training map hashes), commit `feat(build): the training map merges carriageways, tracks and
  parallel paths`.

### Task 4: Shift before cutting

**Files:** `crates/freemapper/src/build/training/shift.rs` (new); `training/mod.rs` (after merge, before
`move_points` and `give_way`).

Decisions (spec "Shift"): for each Apart conflict (by ascending loser rank, as give way orders) whose loser is a
line: the stretch from `lines::beside(loser, winner, …)`; the side away from the winner (the sign of the cross
product of the loser's direction and the vector to the winner's closest point, sqrt-free); `shift` = what is
missing to the gap plus `TOLERANCE_MM`; if `shift > MAX_SHIFT_MM` → skip; the whole line when the stretch covers
more than 75 % of it (no ramps), else the stretch with ramps of `4 * shift`; accept only if `conflicts` on the
result shows no new conflict of the shifted object with any object (Apart or Joined) and no conflict with a
point symbol; else keep the original. Barriers (513/515/518) shift like other lines. Junctions recorded before
and repaired after. Report `shifted`.

- [ ] Tests: `an_earth_bank_beside_a_ditch_moves_off`; `a_fence_beside_a_path_moves_off_and_keeps_its_ticks
  _side`; `a_shift_that_would_hit_a_contour_is_refused`; `a_shift_beyond_half_a_millimetre_is_refused_and_cut`;
  `a_partly_close_fence_moves_only_there` (ramps); `a_path_end_on_the_shifted_fence_follows` (junctions).
- [ ] Bless goldens; commit `feat(build): the training map shifts lines before it cuts them`.

### Task 5: Carriageways in Berlin

**Files:** `crates/freemapper/src/build/paved.rs` and `build/assembly.rs` (pass the survey `cm_fahrbahn`
surfaces inside street parcels as candidates tagged `freemapper:hidden=carriageway`, symbol 501.11);
`training/carriageway.rs` (new, called right after `join`).

Decisions (spec "Carriageways"): in `train`, a hidden carriageway object is un-hidden, cut to the union of the
501.5 objects whose `freemapper:nutzart` names a street (Straßenverkehr, Platz) and drawn as 501.11 (no border);
kerb lines (501.1 with `freemapper:source` = `geoportal-berlin/sb-bordstein`) that lie inside a street area by
more than half go; the generated border (`freemapper:source=generalization`) stays. Pieces below 501.11's minimum
area go. `pdf_objects` and the map never draw `freemapper:hidden` objects (check). Report `carriageways`.

- [ ] Tests: `the_survey_carriageway_is_a_hidden_candidate` (assembly); `the_training_map_draws_the_carriageway
  _darker`; `kerb_lines_inside_a_street_go_and_the_border_stays`; `without_a_survey_nothing_changes`.
- [ ] Bless goldens (Berlin candidates.omap and training map); commit `feat: the training map draws Berlin's
  carriageways in heavy-traffic brown`.

### Task 6: Report, CLI and docs

- CLI summary (`pipeline.rs`): "… <merged> merged, <shifted> shifted, …"; a `warning:` when `junctions_free > 0`?
  No: report it in the summary only.
- ADR 0150 (number reserved with session "freemapper"), amends 0140; ADR index; changelog (Unreleased); manual
  de/en (training map chapter); AGENTS.md "What to tell the user" (remove "Carriageways, tracks and paths that run
  side by side still stand side by side"); spec status.
- Commit `docs: the training map, second half`.

### Task 7: Measure and PR

- Rebuild Bremen Bürgerpark, Viktoriapark, Bastei in scratch with `export --force --training`; measure with the
  helper of the first half (apart, joined, 518/515/513 lengths, free ends, 501.1 inside street areas, time and
  memory); render crops (Bremen north-west railway and roads, Viktoriapark streets, Rathen); fill ADR 0150
  "Measured"; check the acceptance of the spec section.
- Merge `github/main`, checks, PR with label `all-systems`, merge on green CI, watch main CI.
