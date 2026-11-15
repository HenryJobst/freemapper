# 0150. The training map, second half

Date: 2026-11-15
Status: Accepted, amends ADR 0140

## Context

The first half of the training map (ADR 0140, PR #139) left no apart conflict on Bremen Bürgerpark, Viktoriapark and
the Bastei. It got there only by cutting and dropping, and it left four things open:

- Carriageways, railway tracks and paths along roads still stood side by side. ADR 0140 counted these joined pairs
  and settled none of them. On the Bremen training map they add up to 1667 mm. Before the first half, 18 % of the
  length of the wide roads (502) there lay closer than 0.8 mm to another 502.
- Fences and walls were cut wherever the data puts them in the gap of a stream or a path. In Bremen, 20 % of the
  length of the fences (518) lay closer than 0.4 mm to a path. The check print has 386 mm of 518 and the training
  map kept 104 mm (89.4 mm before the final fixes of the first half). The conflict stretches of the 518 alone come to
  290 mm, most of them fences drawn on a stream or a path. Cutting is the only thing the first half can do there.
- Kerb lines (501.1) inside the street areas stayed. ISSprOM 2019-2, 501 says: "Edges within paved areas are
  generally not represented, unless they serve navigation", and "Differences in the brown colour shall be used to
  show differences in the traffic".
- A path that ended on a line the first half dropped or cut was left with a free end.

The user's order for the second half (spec 2026-11-14, "The second half, as designed on 2026-11-15"): generalize
what is alike, then shift, then cut.

## Decision

The training level (`build/training/`) gets four steps. The full order of `train` is now: join, widen openings
(ADR 0153), draw carriageways, street furniture, open ground, clear olive green, **merge** (with junctions recorded
before and repaired after, then join again), **shift** (with its own record and repair), move points, give way,
minimum sizes.

**Merge** (`merge.rs`, before the points move, so that they move off the merged line).

- Two lines of one class (Road, Railway or Path) in a Joined conflict merge where each lies within both half widths
  plus the gap of the other over at least the minimum length of the narrower symbol (`lines::beside`, from both
  sides). The line kept is the wider symbol. Of the same width it is the longer one, and of the same length the
  earlier. Its stretch becomes the middle of the two (`lines::middle`, samples every 0.25 mm). It leaves its own
  course over a ramp of four times the distance to the middle, where there is room.
- The kept line takes the tags of the longer line. The other line's source and id go into `freemapper:also`, as a
  join does, and `freemapper:generalized` gets `merged with <id>`. The other line keeps its pieces outside the
  stretch when each reaches its minimum length. Their cut ends get a vertex on the kept line, so a pair that parts
  makes a Y.
- An object merges once per round. The rounds measure again and stop when nothing merges, after 20 at the latest.
  Closed lines and lines of several parts do not merge.
- On the forest map only, before the rounds: a path, or a railway tagged `railway=tram`, in the gap zone of a road
  goes. The zone is where a Joined conflict is measured: the drawn half width of the one that stays plus the half
  width of the one that gives way plus the gap, from the road's centre line (0.675 mm for a 505 beside a 503 at
  1:10 000, so 6.75 m on the ground; 0.85 mm beside a 502, whose casing reaches 0.435 mm). A rider
  that lies more than half its length in the zone goes (`dropped_beside_roads`). Otherwise only its stretches in
  the zone are cut out, each at least 1.0 mm long, crossings left out. A piece below its minimum goes, and the cut
  end next to the road is drawn onto the road's centre line. The piece carries `cut: in the gap of a road`.
  *Ruling:* the zone is that of the conflict, not the road's drawn width. Read as the drawn width, sidewalks 4 to
  6 m from the road's centre stayed as joined conflicts that nothing settled. *Cost:* a real path that runs close
  beside a road over most of its length is dropped. ISOM does not map sidewalks and cycleways apart from their road.
  *Fix after the measurement:* the rule first took the road's half width, which for a 502 is the width of its fill
  (0.225 mm), not its casing; the measure saw sidewalks the rule did not, 260 mm of them in Bremen. When a pair
  merged, the rule runs once more after the merge rounds, against the merged roads; a piece cut before is not
  counted again.
- On the sprint map the street space is a paved area (501.5), on which paths and trams are Free, so nothing on it is
  touched. Joined pairs of 501.x and 505.x lines of one class merge there as on the forest map.

**Shift** (`shift.rs`, before give way cuts).

- For every Apart conflict whose loser is a line of one part, the loser's stretch inside the winner's zone moves
  away from the winner. The zone is the winner's footprint grown by the loser's half width and the gap, with the
  crossing discs of the measure left out and stretches under 1.0 mm ignored. It moves by what it lacks to the gap,
  plus the tolerance, and by at most 0.5 mm on paper (5 m at 1:10 000, 2 m at 1:4000). Losers go by rank, lowest
  first, then the longer first. The winner stays.
- Each stretch moves with a ramp of four times the shift at each end. Stretches closer together than two ramps move
  as one. When the stretches cover more than 75 % of the line and lie on one side, the whole line moves, without
  ramps. A ramp that would pass an open end becomes a stretch to that end, and the end moves with it. The side is
  away from the nearest point of the winner's outline, or towards it when the middle lies inside the footprint. A
  cross product of exactly 0 means no shift. On a closed line, a stretch across the seam is one stretch.
- A shift is kept only when the moved line has no new conflict. The check looks at every object within 3 mm
  (lines, areas, points and contours) and at both relations. The moved line may also have no conflict of a worse
  grade and none with the winner. Otherwise it stays where it was, and give way cuts it as before.
- The order of the points is kept, so fences, walls and earth banks keep their tick side. A shifted line carries
  `freemapper:generalized=shifted: gap to <class>`. It is drawn again from its flattened centre line, so Bézier and
  dash flags go and the close flag stays.
- *Ruling:* every object keeps the arc ranges it has moved in, and a later shift that overlaps one of them is not
  made. So the 0.5 mm bound holds across rounds and a stretch never moves twice. *Cost:* a line that one shift has
  not freed waits for give way, even where a second small shift would have freed it.
- *Ruling:* the conflict measure reads the symbols once (`Footprints::with`) and measures one object
  (`conflicts_of`), and the pair logic lives in `pair_conflict`. This is a refactor and `conflicts` gives the same
  result.

**Junctions survive** (`junctions.rs`).

- Before the merge and again before the shift, every end of an open line (road, path, railway, stairway, barrier,
  watercourse, earth bank) that touches another such line within `TOUCH_M` (0.1 m) at the map scale is recorded.
  Afterwards, an end that no longer touches is moved to the nearest point of the line it touched, or of the line
  that replaced it (`replaced_by`, followed through every replacement). That point must lie within the shift bound
  plus the gap. Only the end vertex moves. An end that finds no line counts as free (`junctions_free`). An end the merge drew onto a line (a Y or a cut sidewalk) is recorded too
  (`Junctions::attach`), so it follows that line when it merges again.
- *Fix after the measurement:* give way and the minimum sizes cut and drop lines after both repairs, so ends were
  left free that no count saw. The keys recorded before the merge are set aside while the shift records its own
  (`set_aside`, `bring_back`), and after give way, the minimum sizes and the last join the ends recorded before the
  merge are repaired once more, onto what is left of the line they touched, with the same reach. What stays free is
  `junctions_free`, and the summary prints "N line ends left free" ("N Linienenden frei") when there are any. Give
  way names the lines each line gave way to: an end is not drawn onto a line that was cut for the end's own line
  (two lines that lay on each other, where the gap is meant), and it is not counted free when every line it could
  move onto was cut for its own.
- *Ruling:* the keys are stable. `record` tags every line with a temporary `freemapper:junction=<index>`, and
  `forget` removes the tag before `train` returns (a debug assertion checks it). A key that names several pieces of
  a split line resolves to the piece whose end lies nearest the recorded position. An end that lies further than the
  reach from its recorded position went with its stretch and is not counted. Keys that count objects by position
  would move an end onto the wrong line once an object before it is dropped. *Cost:* a tag lives on the objects during the
  level, and every return path must clear it.
- *Ruling:* an end touches only where it meets the body of a line. Meeting another recorded end alone does not
  count, and body contact ranks before end contact. Taken literally, two paths that ended at one point on a road
  that shifted would each still touch the other, and both would be left off the road. *Cost:* the second end may
  attach to the first path's moved end segment instead of the road. It is still connected, but it may not lie on the
  road.

**Carriageways** (`carriageway.rs`, right after join and the widened openings; decision 13 of the spec).

- Only Berlin has a carriageway in the data: the street survey's `cm_fahrbahn` and its kerb lines `sb-bordstein`,
  on the sprint map. The build cuts each carriageway to the 501.5 pieces of the street parcels (nutzart
  `Straßenverkehr` or `Platz`, green strips taken out) and passes it as a hidden 501.11 candidate
  (`freemapper:hidden=carriageway`). Neither the map nor the check print draws it. `training_candidates` lets
  exactly these hidden candidates into the training input.
- On the training map the carriageway loses its hidden tag, is cut to the union of the street areas, and is drawn as
  501.11, paved area with heavy traffic, in 50 % brown and with no border. Pieces below 501.11's minimum area of the
  bundled set (0.5 mm²) go. ADR 0140 named 501.10, but 501.10 of the bundled set carries the border. A survey kerb
  line (501.1) goes when more than half its length lies inside the street area shrunk by 0.75 m. A kerb within
  0.75 m of the edge is the edge the generated border leaves to it, so it stays, as the generated border does
  (`freemapper:source=generalization`).
- A hidden carriageway never stays hidden on the training map: with no street area, or no 501.11 in the symbol set,
  it goes. Outside Berlin nothing changes.
- *Ruling:* the hidden carriageways appear in `templates/candidates.omap` and `ocad/<name>-candidates.ocd` as 501.11
  areas, as the other hidden candidates do, because they are real survey data a mapper can use. *Cost:* dark brown
  areas in the candidates template when it is opened in OOM or OCAD. The manual says so.
- Point generalization ignores the hidden paved candidates (`way_footprints`), so the base map stays as it was.

**The report.** `TrainingReport` gets `merged`, `dropped_beside_roads`, `shifted`, `junctions_repaired`,
`junctions_free`, `carriageways` and `kerbs`. The summary line reads "N joined, N shifted, N left out or cut"
(German "zusammengelegt", "verschoben", "weggelassen oder gekürzt"). Merged pairs count as joined. Riders dropped
beside roads and kerb lines count as left out. Give way now adds to `cut` instead of overwriting the cuts of the
merge.

## Consequences

- The training maps of Berlin, Brandenburg, Hessen and Saxony-Anhalt in the goldens change (shift, carriageways).
  Berlin's `templates/candidates.omap` and `ocad/berlin-candidates.ocd` gain three hidden 501.11 candidates. The base
  maps, the check prints and the g3 traces stay as they were. Existing projects change only when their training map
  is written again.
- On the training map, lines move by up to 0.5 mm on paper and points by up to 1.5 mm. Merged roads and paths lie on
  the middle of their sources. Mapping goes on in the base map and the candidates, which keep the measured positions.
- The training export takes longer: in the release build 37 s instead of 23 s for Bremen Bürgerpark, 11 s instead
  of 8 s for Viktoriapark and 6 s instead of 4 s at the Bastei (CPU time, see "Measured"). As first written, the
  shift step drew every footprint again each round and round each attempt, and Bremen took 315 s. It now keeps the
  footprints and the verdict on each pair until one of the objects moves, finds the neighbours of a line in a grid,
  and does not try a refused shift again while nothing near it moved; the result is byte for byte the same.
- Cost if wrong: the bounds are constants (`MAX_SHIFT_MM`, `RAMP_FACTOR`, `WHOLE_SHARE`, `REACH_MM` in `shift.rs`,
  `MIDDLE_STEP_MM` and `RAMP_FACTOR` in `merge.rs`), and each step is one function called from `train`.

## What is left

- A fork that leaves a road at less than 20° is not a crossing for the measure, so its first stretch counts as
  inside the gap zone and is cut.
- A tram that runs partly inside a road is cut there like a path. The spec only says that a tram inside a road goes.
- Merged and shifted lines are written as plain vertices: Bézier curves are flattened and the dash flags dropped.
- Three lines side by side merge in two rounds. The ends that the first merge drew onto the kept line are attached,
  but a second merge of that line can still loosen them by up to half the distance.
- Of two lines of one rank and one length in an Apart conflict, neither shifts: the pair falls to give way.
- Bremen keeps 48 % of the length of 518, not half: the enclosures at the Meierei are fences drawn inside the gap
  of a stream with no room to move them 0.5 mm off it, so give way cuts them there.
- End to end (every end of the check print that touched a line against the training map), more ends are free than
  the program counts: 43, 14 and 15 on the three examples, against 18, 6 and 9. The program does not count an end
  whose line was cut for its own line (the gap is meant), and the measure counts any end that lies within reach of
  where a touching end was. How the difference splits between the two was not sorted out.
- Smaller items: two pieces of one key whose ends of the same flag lie within reach are rare and not resolved
  further; after a stretch across the seam of a closed line, the arc positions of the stretches before it are off
  by up to about 0.06 mm; a survey kerb that lies more than half inside a street goes whole, even where part of it
  was the street's edge; `way_footprints` skips every hidden paved candidate, not only the carriageways.

## Measured

On 2026-11-15, with the release build of the fix wave (head 0665819b), on the three examples rebuilt in scratch from
their `freemapper.toml`, `export --force --training`, measured with the program's own conflict measure (check print →
training map, mm on paper). The machine was shared: load average 8 to 10 during the exports, so the times are upper
bounds. Way pairs are the joined pairs of roads, railways and paths, what merge settles. Free ends are the program's
`junctions_free`. Before the fix wave the first measurement (head 07783574) gave 315 s, 17 s and 8 s, way pairs
−72 %, −100 % and −96 %, 48.1 % of 518 in Bremen, and 0, 0 and 3 free ends, counted after the merge and the shift
only.

Bremen Bürgerpark (forest 1:10 000):

| | check print | training map |
|---|---|---|
| Apart conflicts | 3949.1 mm | 0 |
| Way pairs | 995.5 mm | 14.6 mm (−98.5 %) |
| Path–Road | 367.2 mm | 0 |
| All joined pairs | 1692.5 mm | 705.2 mm (−58.3 %) |
| 518 | 386.1 mm | 186.1 mm (48.2 %) |
| Line ends left free | | 18 |
| Training export: CPU / real / max RSS | | 37.0 s / 39.8 s / 292 MiB (first half 23 s / 24 s / 350 MiB) |

Viktoriapark (sprint 1:4000):

| | check print | training map |
|---|---|---|
| Apart conflicts | 550.0 mm | 0 |
| Way pairs | 15.1 mm | 0 (−100 %) |
| All joined pairs | 158.9 mm | 143.8 mm (−9.5 %) |
| 518 | 653.9 mm | 592.1 mm |
| Line ends left free | | 6 |
| Training export: CPU / real / max RSS | | 11.2 s / 11.7 s / 131 MiB (first half 8 s / 8 s / 195 MiB) |

Bastei (forest 1:10 000):

| | check print | training map |
|---|---|---|
| Apart conflicts | 639.1 mm | 0 |
| Way pairs | 264.2 mm | 11.1 mm (−95.8 %) |
| All joined pairs | 525.5 mm | 245.2 mm (−53.3 %) |
| 518 | 153.8 mm | 70.6 mm |
| Line ends left free | | 9 |
| Training export: CPU / real / max RSS | | 5.9 s / 6.2 s / 78 MiB (first half 4 s / 4 s / 100 MiB) |

The joined pairs left are mostly water against watercourse and Berlin's stair step lines, which no step of the second half touches. The Bremen way pairs left are 3.9 mm of Road–Road,
2.3 mm of Path–Path and 8.4 mm of Path–Railway.

Source: spec `docs/superpowers/specs/2026-11-14-training-map-design.md` ("The second half, as designed on
2026-11-15", decision 13), plan `docs/superpowers/plans/2026-11-15-training-map-2.md` and its rulings ledger;
ISSprOM 2019-2 501, §3.3; ISOM 2017-2 502 to 507.
