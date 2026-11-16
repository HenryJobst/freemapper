# 0157. The light level of the base map

Date: 2026-11-16
Status: Accepted, amends ADR 0126 and 0140

## Context

The base map is the mapper's working material: every object lies where its source has it, and the mapper checks,
corrects and copies. On the forest example Bremen Bürgerpark it also carried what nobody can read on it: under the
default profile `balanced` the measure of ADR 0140 finds 340 *overlap* conflicts of pairs that keep apart on the
map alone, 2400 mm on paper, most of them earth banks drawn on a ditch, a bank line or a path. The footpaths came
in 416 pieces.

The user decided on 2026-11-14 (spec `docs/superpowers/specs/2026-11-14-training-map-design.md`, decision 3): the
base map is cleaned lightly. Duplicates and objects hidden by another object leave the map for the candidate layer
of their kind (ADR 0156), and nothing is shifted. The spec, "Part 2: the light level", lists three steps and the
acceptance: no doubled water and no doubled stairs, no *overlap* conflict of an *apart* pair left on the map, at
least half fewer 505 objects, no vertex of a kept object moved. The training map starts from the map before the
light level and must not change.

## Decision

The light level (`build/light.rs`, `light_level`) runs on the objects routed to the map, as the last step of
`Built::place`. It does three things, and none moves a vertex:

1. *Join.* Lines of one symbol and one source (the value of `freemapper:source`) that meet end to end within 0.1 m
   are joined, by the rules of the training level's join (ADR 0140, step 1: the same symbol classes, sided lines
   never reversed, at a junction of three or more ends only the pair that turns least, below 30°). The joined
   line keeps the tags of the longest piece and names the others in `freemapper:also`
   (`line_join::join_lines_by`). Unlike the training level's join it loses no vertex: where the end of one piece
   and the start of the next only touch, both stay, and so does the last vertex of a closed chain; only a vertex
   identical to the one before goes (`Joint::Kept`; the training level keeps `Joint::Merged`).
2. *Give way.* For every pair that keeps apart (`Relation::Apart`) whose drawn footprints cover each other
   (`Grade::Overlap`), the line of lower rank gives way, with the ranks of ADR 0140: an earth bank to a
   watercourse, a bank line, a railway, a road and a path, a fence or wall to a path or road, a contour to an
   earth bank, the shorter of two earth banks. Its stretches inside the winner's footprint grown by its own half
   width (the crossing discs of the measure left out), each at least 1.0 mm long, leave the map. The whole line
   leaves when those stretches are more than half of it, or when no piece between them reaches the minimum length
   of its symbol; a piece between them below the minimum leaves with them. A *gap* conflict without overlap
   stays: both objects can still be told apart. Pairs of one network (`Relation::Joined`) are not touched. A
   loser waits while one of its winners gives way itself; at most 20 rounds, each measuring only the pairs of
   what changed (`PairMemo`, `Footprints::push`, `Footprints::undraw`).
3. *Too small.* Lines and areas below the norm's minimum size leave, by the rules of the training level's step 7
   (`training::too_small`): lines of the classes of the measure (not the north lines and the borders of paved
   areas, class `Other`), the areas 401 to 410. A line that ends on a line that stays stays too.

What leaves is appended to the candidates with `freemapper:generalized` and no `freemapper:hidden`: `hidden by
<classes>` (whole, or the stretch), `below the minimum size`. The pieces of a cut line that stay carry `cut where
hidden by <classes>`. The candidate layers put each object by its group (ADR 0156); the check print draws it. A
cut keeps every vertex of the line in place and in order, so the tick side stays; a piece gets a new vertex only
at a cut end, on the line's course. A piece of a closed line is open.

*Rulings:*

- *After the ticks, not before.* The spec puts the light level before the ticks are oriented. It runs after them
  and after the points are moved off the barriers (`move_points_off_barriers`, ADR 0097), so that one orientation
  serves the base map and the training input. It turns no line and moves nothing, so the ticks it leaves stay
  right. *Cost:* two pieces of a fence or bank that the orientation turned to opposite sides do not join, and a
  point that was moved off a fence stays moved when the light level then takes that fence off the map.
- *Points stay on the base map* (the controller's ruling, 2026-11-16). A point on a line hides nothing, and its
  true position is what a mapper needs. The user's rule for the base map is that only duplicates and hidden
  objects go to the layers and nothing is shifted. So only lines give way here; the acceptance line "no overlap
  conflict of an apart pair left" holds for lines, and the points are settled on the training map (ADR 0140,
  step 5). Areas never lose to a line in the rank table. *Cost:* the base map keeps overlap conflicts of points
  (220 at Viktoriapark, 150 of them trees on paths; 13 in Bremen).
- *One source is one value of `freemapper:source`.* OSM and the cadastre, or two layers of the full ALKIS, are
  never joined: the joined line keeps one source's tags. *Cost:* fewer joins; at the Bastei the 505 objects fall
  by 39 %, not by half.
- *A short stairway between two ways stays a stairway.* The training map draws it as the path it lies in; on the
  base map that would change what the object is. It stays where both its ends continue on a way that stays,
  otherwise it leaves. *Cost:* stairs below three steps remain on the base map where they link two ways.
- *The training input is set aside first.* `Placed.training` (`training::TrainingInput`) holds the map before the
  light level and the candidates the training map draws (every candidate but the outline and the hidden ones,
  the hidden carriageways kept), and `StarterDocuments.training` carries it to `export_training`. *Cost:* one more
  copy of the map and the visible candidates in memory.
- *The cliffs of ADR 0126 follow.* A cliff `generalize_cliffs` takes off the map keeps `freemapper:hidden=
  generalized` inside `place`, where the steps after it read the tag as before, and loses it once the training
  input is set aside (`cliffs::reveal_generalized`). The check print now draws these cliffs; the training map
  leaves them out as before. The knolls on the rock keep `freemapper:hidden=rock`.
- *The duplicate steps need no repair.* With the fixes of ADR 0140 (OSM water with its islands, OSM stairs on
  cadastre stairs) no OSM water area lies more than half on water of another source and no OSM stair lies with
  half its length on a stair of another source, before and after the light level, on all three examples.

## Consequences

- Drawn base maps change: fewer and longer lines, earth banks and fences off ways and water, small pieces gone.
  Existing projects change after `export --force`. The candidate layers and the check print gain what the map
  lost; the map and its layers together hold every object.
- The training map, its PDF and OCAD file do not change: on the three examples below they are byte for byte the
  same as with the binary of `main`.
- The golden net (G4) changes in the base map, its PDF, the OCAD map, the check print and the candidate layers
  that gained objects, in all four of its states. The check print draws the same objects as before where no cliff
  was taken off, now partly as candidates and in another order, so its bytes change too. The training maps and
  their PDFs keep their hashes.
- `export` takes longer: on Bremen Bürgerpark 3.5 s of 35 s.
- After writing the map, `run` and `export` print one line on what the light level did (`cli-map-light`), and the
  MCP summary has it too.
- A run without `--force` keeps an existing map but writes the candidate layers anew. They then hold what the
  light level takes off the new map while the kept map still has it, so these objects are there twice. The run
  says so (`cli-map-light-kept`, in the MCP summary `cli-summary-map-light-kept`) when the light level took
  anything; `--force` replaces the map.
- Cost if wrong: the steps are three functions in `build/light.rs`; the ranks and relations stay those of
  `build/conflicts.rs`. Taking the light level out is one call in `Built::place`.

## Measured

On 2026-11-16, release builds of `main` (6dcf3745) and of this branch, on Bremen Bürgerpark (forest 1:10 000),
Viktoriapark in Berlin (sprint 1:4000) and the Bastei in Saxony (rocky forest 1:10 000), copied from `examples/`
and run with `run --force --training` (cache warm). The counts come from a throwaway test with the program's own
measure on the base map before and after the light level, under each project's content settings (`balanced`).
Lengths are mm on paper.

| | Bremen Bürgerpark | Viktoriapark | Bastei |
|---|---|---|---|
| Map objects, before → after | 1779 → 1169 | 3188 → 3061 | 1318 → 1039 |
| Apart overlap conflicts of lines | 327 (2400 mm) → 0 | 44 (272 mm) → 0 | 105 (345 mm) → 0 |
| Apart overlap conflicts of points (stay) | 13 → 13 | 227 → 220 | 1 → 1 |
| Apart gap conflicts | 312 (1487 mm) → 186 (1006 mm) | 192 (73 mm) → 177 (102 mm) | 120 (198 mm) → 55 (105 mm) |
| 505 objects | 416 → 208 (−50 %) | none (path class 215 → 172) | 99 → 60 (−39 %) |
| Stairways | 47 → 15 | 142 → 121 | 95 → 61 |
| Joined / gave way whole / cut / too small | 439 / 127 / 46 / 78 | 86 / 12 / 14 / 40 | 155 / 42 / 23 / 95 |
| Objects to the layers: terrain, water, paths, barriers | 167, 26, 43, 34 | 15, 0, 20, 35 | 55, 0, 61, 52 |
| OSM water on other water, OSM stairs on other stairs | 0, 0 → 0, 0 | 0, 0 → 0, 0 | 0, 0 → 0, 0 |
| Vertices moved / lost (map and what left it) | 0 / 0 (103 new cut ends) | 0 / 0 (30) | 0 / 0 (45) |
| Training map `.omap` and `.pdf` | same bytes | same bytes | same bytes |
| `export --training`: real, maximum resident set, `main` → branch | 31.9 s, 359 MiB → 35.4 s, 327 MiB | 9.0 s, 199 MiB → 9.3 s, 194 MiB | 5.3 s, 103 MiB → 5.8 s, 103 MiB |

The gap conflicts after a cut are the cut ends: the stretch on the other symbol went, the piece beside it stays
inside the gap, as the spec wants. The first version measured again from scratch in every round and cost 10.6 s on
Bremen; keeping the slots and the verdicts of unchanged pairs brought it to the 3.5 s above with the same result.

Not met: at the Bastei the 505 objects fall by 39 % (99 → 60). Most of the rest meet at junctions of three or more
ways, where only the straightest pair joins, or come from two sources. Viktoriapark has no 505; its path class
falls by 20 %.

Source: spec `docs/superpowers/specs/2026-11-14-training-map-design.md` (decision 3, "Part 2: the light level",
Acceptance); plan `docs/superpowers/plans/2026-11-16-light-level.md`; ADR 0126, 0140, 0150, 0156.
