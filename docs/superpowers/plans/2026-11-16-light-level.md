# The light level of the base map (plan)

The rest of Part 2 of `docs/superpowers/specs/2026-11-14-training-map-design.md`: "The light level". The
conflict measure (`build/conflicts.rs`), the repaired duplicate steps (ADR 0140: OSM water with islands, OSM
stairs on cadastre stairs) and the candidate layers (ADR 0156) exist already. The ADR of this plan amends ADR
0140 and 0126.

## Shape

- The light level runs on the objects routed to the map, as the last step of `Built::place`, after the ticks
  are turned and the points are moved off the barriers. The spec puts it before the ticks. It runs after
  them so that one tick orientation serves the base map and the training input: the training map must not
  change, and its input is the map before the light level (spec, "Order"). The light level moves no vertex
  and turns no line, so the ticks it leaves stay right.
- `Placed` gets a third list, `training`: the input of the training level, the map before the light level and
  the candidates the training map draws (every candidate but the outline and the hidden ones, the hidden
  carriageways kept). `StarterDocuments.training` carries it to `export_training`, which no longer reads the
  map parts and the candidates for it.
- What leaves the map is appended to the candidates with `freemapper:generalized=<reason>` and no
  `freemapper:hidden`. The candidate layers put it by its group (ADR 0156), and the check print draws it.
- The cliffs that `generalize_cliffs` takes off the map at `Level::map` keep `freemapper:hidden=generalized`
  inside `place` (the steps there read it as before), and the tag is taken off once the training input is set
  aside: the check print draws them, the training map stays as it was.

## Files and signatures

- `crates/freemapper/src/build/light.rs` (new)
  - `pub struct LightReport { joined, gave_way, cut, too_small }`
  - `pub fn light_level(map: &mut Vec<MapObject>, kind: MapKind, georef: &Georeferencing, symbol_set:
    &SymbolSet) -> (Vec<MapObject>, LightReport)`: returns what left the map.
  - step 1 `join`: lines of one symbol and one source (`freemapper:source`) that meet end to end, the classes
    the training level joins (`training::join_ids`), by `line_join::join_lines_by`.
  - step 2 `give_way`: every Apart pair of grade Overlap whose loser is a line; the loser's stretch inside the
    winner's footprint grown by its half width (`training::conflict_zones`, crossing discs left out) leaves the
    map, the whole line when that stretch is more than half its length; a rest below the minimum length
    (`training::min_length`) leaves with it. Points and areas are not touched. Rounds as the training level
    (winners first, at most 20); the objects keep their slots across the rounds and `PairMemo` measures
    only the pairs of what changed (added after the first measurement, where the plain rounds cost 10 s of
    release time on Bremen Bürgerpark; `Footprints::push` and `Footprints::undraw` are new). The cut keeps every vertex of the line as it is: own split by arc length in
    map units (`split`), new vertices only at the cut ends.
  - step 3 `too_small`: `training::too_small` without its stair-as-path step; lines of the classes of the
    measure (not `Other`, so north lines and paved borders stay) and the areas with a minimum size.
- `crates/freemapper/src/build/line_join.rs`: `pub fn join_lines_by(objects, ids, sided, georef, group:
  impl Fn(&MapObject) -> Option<String>)`; `join_lines_keyed` unchanged in behaviour (one group).
- `crates/freemapper/src/build/training/mod.rs`: `pub(crate) fn join_ids(&Classes) -> (Vec<u32>, Vec<u32>)`,
  `pub(crate) fn conflict_zones(...)` (out of `cut_out`), `pub(crate) fn too_small(...) -> TooSmall` (out of
  `drop_too_small`, which applies it), `pub(crate)` for `min_length`, `class_name`, `add_generalized`, and
  `pub struct TrainingInput { map, candidates }` with `TrainingInput::of(map, candidates)` and `objects()`
  (the filter out of `export.rs`; first `pub fn input`, made a struct so that the measure tells the map's
  part from the candidates').
- `crates/freemapper/src/build/cliffs.rs`: `pub fn reveal_generalized(candidates: &mut [MapObject])`.
- `crates/freemapper/src/build/assembly.rs`: `Placed { map, candidates, training, light }`; `place` ends with the
  training input, the light level and the revealed cliffs.
- `crates/freemapper/src/project/export.rs`: `StarterDocuments.training`; `export_training_from` reads it.

## Tests (first, red)

In `build/light.rs` on the ISOM set at 1:10 000 (the fixtures of `build/training`):

1. two 505 of one source that meet end to end become one, naming the other in `freemapper:also`; of two
   sources they stay two; a 505 and a 506 stay two.
2. an earth bank on a path for most of its length leaves whole, with `freemapper:generalized`, no
   `freemapper:hidden`; one with a short stretch on the path loses only the stretch, the rest stays with its
   vertices; a rest below the minimum leaves too.
3. an earth bank beside a path inside the gap but not on it stays (gap conflicts stay).
4. ranks: earth bank under watercourse, bank line, railway, road and path; fence under path and road; the
   shorter of two earth banks; a contour piece on an earth bank.
5. two paths on each other (joined pair) stay; a tree on a path stays.
6. a stair below the minimum with no way at its ends leaves; a short contour loop leaves; a 401 below its
   minimum area leaves.
7. no vertex of a kept object moves (every vertex of every kept object is a vertex of the input).
8. no overlap conflict of an Apart pair is left after the level.

In `build/assembly.rs` / `project.rs`:

9. the light level's objects land in the candidates, in the layer of their group, and in the check print;
   the training input is the map before the light level (`Placed.training`).
10. a cliff taken off the map has no `freemapper:hidden` in the candidates and is not in the training input.

## Measure

`examples/bremen-buergerpark`, `examples/berlin-viktoriapark`, `examples/sachsen-bastei` copied to the
scratchpad, `run --force --training`, with the main binary and the branch binary. A throwaway test (not
committed) reads the documents of each copy and counts on the map before and after the light level: overlap
conflicts of Apart pairs, 505 objects, OSM water on other water, OSM stairs on other stairs, moved vertices.
Times of `export` both ways. Training map files compared byte for byte.
