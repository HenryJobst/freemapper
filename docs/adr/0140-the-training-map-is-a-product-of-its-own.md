# 0140. The training map is a product of its own

Date: 2026-11-15
Status: Accepted, amends ADR 0080, 0097, 0126 and 0134; amended by ADR 0151 (closed and enclosing fences have their ticks inside)

## Context

Until 0.21.0 the "training map" was `<name>-candidates.pdf`: the base map with every visible candidate drawn on
it, nothing more (ADR 0080). On the forest example Bremen Bürgerpark (1:10 000, 0.20.0) the user found symbols in
heaps. Measured on paper with a throwaway script (spec 2026-11-14):

| Finding | Number |
|---|---|
| Earth banks (104) | 263 objects; 30 % of their length closer than 0.4 mm to a ditch, 26 % to water, 31 % to another bank, 14 % to a way |
| OSM water (301) on ALKIS water (301.1) | 5 areas at 91 to 99 %, 613 of 781 mm² |
| Wide roads (502) closer than 0.8 mm to another 502 | 18 % of their length (carriageways) |
| Footpaths (505) | 416 pieces, 143 shorter than 2 mm, 15 % closer than 0.5 mm to another 505 |
| Stairways (532) | 53, 48 shorter than 1 mm; 6 of 23 OSM stairs on an ALKIS stair |
| Fences (518) closer than 0.4 mm to a path | 20 % of their length |
| Contours | 160 pieces, 29 through buildings, 9 % on an earth bank |
| Open land (401) from the land use "Park" in the Stadtwald | 13 % and 25 % open by the nDSM: forest drawn yellow |

The sprint example Viktoriapark (1:4000, map and candidates) adds details inside olive green (239 of 264 canopies
and passages, 59 earth banks, 81 trees, 42 path pieces), 168 passages into yards, 83 of 1286 trees touching a path
line (16 centred on it), 119 of 144 street furniture crosses on street areas, 270 kerb lines (7361 mm) and small
patches (23 of 81 open land areas under 4 mm²).

The user decided (2026-11-14): the training map becomes a product of its own, printed and run on as it is, and
keeps every gap of the norm; the base map stays the working material, true to position; contours follow the norm
on every map; fence ticks point inward; nothing but buildings and water is drawn inside olive green; trees move off
paths; street furniture stays only off street areas.

## Decision

**Files and switch.** `--training` on `run`, `export` and `select` (the page default), `training: true` on the MCP
tools `start_pipeline` and `open_area_selector`, and a box on the area page write `<name>-training.omap` and
`<name>-training.pdf`, with `--ocad` also `ocad/<name>-training.ocd`. The choice is not stored. The map file has the
base part, the trained objects and the sheet part of the map (sheet objects keep `freemapper:source=sheet`), the
raster templates and no candidates template. Every file is built before the first is written. `run` keeps an
existing training map with a note and exit code 0; `export --training` fails on one; `--force` replaces it and keeps
the old one as `.bak`.

**Input.** The objects of the map outside the base and sheet parts and every candidate without `freemapper:hidden`.
On forest and sprint maps the rock is generalized at `Level::TRAINING` first (ADR 0126, ADR 0134).

**The conflict measure** (`build/conflicts.rs`). Every object becomes its drawn footprint through the renderer. Each
symbol falls in one of 15 classes; a table gives each pair of classes a relation:

- *Free* (may lie on each other): every pair with a paved area, canopy or other area (OtherArea, Other); a contour
  with everything but earth banks and cliffs (ISSprOM 2019-2, 101: "contour lines shall be represented as continuous
  lines through all symbols, also Building (521) and Canopy (522)"); ways and railways with water and buildings
  (bridges); buildings with water; water area with bank line; barriers with stairways (ISSprOM 2019-2, 532: a
  stairway between impassable objects may drop its border lines, so a railing or fence along it is drawn with it);
  two buildings, two water areas, two bank lines, two cliffs.
- *Joined* (one network): roads, paths, railways and stairways among each other; watercourses with watercourses and
  water.
- *Apart* (keep the gap): every other pair.

Where two meet, the lower rank gives way: building 9, water area and bank line 8, railway 7, road 6, path 5,
watercourse and stairway 4, barrier and cliff 3, earth bank 2, contour 1, point and other 0; of one rank the shorter
line (lengths within the tolerance are equal, then the later object). The gap is the gap between two line symbols,
0.15 mm at 100 % (ISSprOM §3.3), grown with the symbols. A conflict is a stretch of the loser's centre line inside
the winner's footprint grown by the loser's half width and the gap, at least 1.0 mm long on paper; *overlap* when the
stretch inside the footprint alone reaches 1.0 mm, else *gap*; one per pair, judged over the whole zone, so that a
line through the ticks of an earth bank is one conflict. Where centre lines cross at 20° or more there is none.
Comparisons at a threshold are tolerant by 2 map units, so a difference in the last bits of a length or a distance (the measure takes them from `geo`, whose `Euclidean` uses `hypot`, and some training steps call `hypot` themselves) does not change a decision from one system to another; the golden net of the adapters covers the training map of its four states.

**The training level** (`build/training/`) runs these steps in order:

1. Ways, stairways, railways, watercourses, earth banks, barriers and contours of one symbol that meet end to end within 0.1 m
   are joined; at a junction of three or more ends only the pair that turns least, if below 30°. Sided lines
   (banks, cliffs, barriers) are never reversed. The joined line names the others in `freemapper:also`. Repeated
   until nothing joins.
2. Street furniture (531) whose centre lies on a paved area goes; in a park it stays.
3. Open land (401) from a park land use (ALKIS "Sport-, Freizeit- und Erholungsfläche", OSM `leisure=park`) keeps
   only its open ground: `derive` writes `data/tree-cover.tif`, the share of the nDSM cells of each 1 m cell that are at
   least 3 m high; a share below 0.5 is open. The open cells are taken to the 3 m cells of the vegetation (a cell is
   open when at least half of its 1 m cells are), and the outline is traced on them as the vegetation areas are
   (ADR 0136) and kept to their minimum areas. Without a surface model nothing changes.
4. Inside olive green (520) only buildings, roofs and carports, water, watercourses, railways and contours stay. The
   inside is the union of the 520 shrunk by 0.4 mm on paper (ruling: what lies on the edge stays). A way (road, path,
   stairway) stays in full where its part inside meets the edge at two places or more that no wall (515) or fence (518)
   bars: a crossing. Otherwise only a stub of 1.0 mm (or the symbol's minimum length) stays at each unbarred place. A
   passage (522 "Durchfahrt", "Arkade", OSM `tunnel=building_passage` or `covered=*`) stays only where a kept way runs
   through it. Areas are cut at the edge, points go, other lines are cut; a cut piece carries
   `freemapper:generalized=cut: inside olive green`.
5. A point symbol closer than the gap to any symbol a point keeps apart from is moved to the nearest spot that
   keeps every gap and whose symbol does not overlap olive green (520), at most 1.5 mm on paper (ruling: a point
   symbol stands for an object beside the way within its own size; a tree centred on a 505 at 1:10 000 needs 0.79 to
   1.24 mm; the lines keep 0.5 mm for the second half). The larger symbol goes first, then the earlier object, so of
   two points too close the smaller moves. It carries `freemapper:moved`. Where no spot is free it is left for step
   6. Trees on street and square areas stay.
6. Of every Apart pair in conflict the lower-ranked object gives way: the stretch is cut out (the winner's footprint
   grown by the loser's half width, the gap and the tolerance, crossings kept), the object goes whole when more than
   half of its length is in conflict, measured over the union of the zones of all its winners. Walls (513, 515) and
   impassable fences (518) never go whole there, only their stretch in conflict is cut (ruling: a training map is run
   on as it is, a missing barrier changes the route choice); pieces below the minimum still go in step 7. A cut piece
   carries `freemapper:generalized=cut: gap to <classes>`. A loser waits while one of its winners still has to give way; at most 20
   rounds, and what is left then is reported in mm.
7. What is below the norm's minimum goes. Lines: the minimum length of the symbol set, else 1.0 mm; a stairway 1.0
   mm ("Minimum length: 3 (graphical) steps" in both norms, steps 0.5 mm apart in both sets); a line that ends on
   a line that stays, stays too: for roads, paths, railways and stairways any of these, for other lines one of their
   class; a chain of short pieces stays when one end meets a line long enough, two short pieces do not keep each
   other. A stairway below its minimum with a staying road or path at both ends is drawn with the symbol of the
   longer of them (`freemapper:generalized=stair below the minimum drawn as path`) and joined into it, unless a barrier
   runs along it so close that as a path it would keep no gap (then it stays a stairway); with one at one end only,
   or none, it goes. Areas: 401 0.5 mm² (ISSprOM) or 0.55 mm × 0.55 mm (ISOM), 402 6.25 mm², 403 1 mm², 405
   1.0 mm², 406 to 410 1.5 mm × 1.5 mm.

Joined pairs side by side are counted, not settled.

**Left to the second half.** Merging lines of one class that run side by side inside the gap onto their middle
(carriageways, railway tracks, parallel paths), dropping a tram track or a footway inside a drawn road, shifting
lines by at most 0.5 mm before cutting them, keeping junctions when lines merge or shift, and colouring the
carriageway (501.10) and the pavement (501.5) instead of the kerb lines (501.1). Until then the training map still
draws them side by side. The candidate layers, the light clean-up level of the base map and the rebuilt examples are
later pull requests; so is the page orientation of the PDF.

**For every map** (amending ADR 0097):

- Contours are no longer cut at ways and paved areas. They are cut out at stairways (532) and at the small landform
  symbols 105, 109, 110, 111, 112 and 115 (ISSprOM 2019-2, 101), not at 105 on a forest map (ISOM's "Earth wall",
  which contours may cross), and still below cliffs (ADR 0126). Hidden objects cut nothing. North lines keep their cut
  at ways (ISSprOM §4.6). Their end at the shore of drawn ALKIS water stays as it was.
- Fence ticks: a closed fence keeps them inside (ISSprOM 516, 518: "If the fence forms an enclosed area, tags should
  be placed inside"), then they point to olive green beside it, away from a way along it (a way on one side at half
  the samples or more and on the other at less than half, within the tick length and the gap), towards the park.
  Only when none decides, with the same area on both sides, they take the side where they overdraw nothing. A map
  fence looks at the ways of the map only.
- An OSM water area whose rings include islands is read with its holes, so `drop_covered_water` now drops one that lies
  more than half on ALKIS water. An OSM stair that lies with at least half its length within the drawn width of a
  cadastre or survey stair is dropped, and the stair names it in `freemapper:also`.

**The check print** (amending ADR 0080 and 0126). `--pdf-candidates` keeps its file `<name>-candidates.pdf` and is
called the check print: the base map with every visible candidate, true to position, at the map level. It no longer
runs `Level::TRAINING`; that moves to the training map.

## Consequences

- Drawn base maps change: contours across ways and paved areas, some fence ticks, fewer doubled lakes and stairs. The
  goldens of Berlin, Brandenburg, Hessen and Saxony-Anhalt are blessed; the check prints of Berlin, Hessen and
  Saxony-Anhalt change because the training rock left them. Existing projects change after `export --force`.
- A project with an nDSM stores `data/tree-cover.tif` and `.tfw` (1 m cells).
- The training map moves up to 1.5 mm what it shows as a point symbol (15 m on the ground at 1:10 000) and leaves out what
  the base map keeps; mapping goes on in the base map and the candidates.
- Until the second half, carriageways, tracks and parallel paths stay side by side, and a line end that touched a
  dropped line may be left free.
- Cost if wrong: the rank table and the relation table are one function each in `build/conflicts.rs`; the bounds are
  constants in `build/training/`.

## Measured

On 2026-11-15, with the release build of this branch after the fixes of the final review, on three examples rebuilt in scratch from their
`freemapper.toml`: Bremen Bürgerpark (forest 1:10 000), Viktoriapark in Berlin (sprint 1:4000) and the Bastei in
Saxony (rocky forest 1:10 000). The conflicts are counted with the program's own measure (`build/conflicts.rs`),
on the check print (the map with every visible candidate, the input of the training level) and on the training
map. Lengths are mm on paper. Stairs count as below the minimum under 1.5 mm at 1:10 000 (150 %) and under
1.0 mm at 1:4000.

| | Bremen Bürgerpark | Viktoriapark | Bastei |
|---|---|---|---|
| Apart conflicts, check print → training map | 3958 mm (658) → 0 | 550 mm (567) → 0 | 639 mm (256) → 0 |
| Training summary | 456 joined, 368 left out or cut | 91 joined, 535 left out or cut | 220 joined, 194 left out or cut |
| Objects | 2196 → 1472 | 3740 → 3205 | 1463 → 991 |
| 505 objects | 416 → 203 | none (sprint path symbols: 215 → 140) | 99 → 45 |
| Path-class length | 4345 → 4318 mm | 2400 → 2019 mm | 1816 → 1804 mm |
| 526 points | 12 → 10 | 3 → 2 | 2 → 2 |
| Stairs below the minimum | 46 of 47 → 0 of 1 | 20 of 142 → 0 of 116 | 74 of 95 → 0 of 19 |
| Points moved | 9 (526) | 278 (269 trees, 5 of 531, 2 of 312, 1 each of 311 and 526) | 5 (2 of 526, 109, 110, 111) |
| Barriers 518 / 515 / 513 (513.2 in Berlin), check print → training map | 386 / 11 / 66 mm → 104 / 5 / 0 | 654 / 300 / 131 mm → 573 / 190 / 89 | 154 / 48 / 58 mm → 50 / 12 / 8 |
| Moved points on olive green | 0 | 0 | 0 |
| `export --training`: real, user, maximum resident set | 24 s, 23 s, 350 MiB | 8 s, 8 s, 195 MiB | 4 s, 4 s, 100 MiB |
| `export` (before the fixes, busy machine): real, user, peak | 8 s, 6 s, 156 MiB | 4 s, 4 s, 121 MiB | 6 s, 5 s, 58 MiB |

In Bremen the two ALKIS park areas of the Stadtwald (464 658 m², 21 % of it open by the nDSM) keep 58 505 m² of
open land, 93 % of it open (60 924 m² before the open ground was outlined on the 3 m cells). No run printed the
warning that conflicts were left.

The barriers (518 / 515 / 513, mm): before the fixes, measured with the binary of the branch before them, the
training map kept 89.4 / 5.2 / 0 in Bremen, 573.0 / 190.3 / 89.2 (513.2) in Viktoriapark and 50.4 / 12.3 / 5.6 at the
Bastei; give way had counted a stretch near two winners twice and dropped walls and fences whole. The fixes bring
back 15 mm of 518 in Bremen and 2.7 mm of 513 at the Bastei; everything else is unchanged.
What is still missing lies in the gap of a way, a watercourse or another fence in the data (in Bremen the conflict
stretches of the 518 alone add up to 290 mm, most of them fences drawn on a stream or a path) and is cut there; only
shifting the line beside it, left to the second half, keeps it.

The memory: the open-land step gave its open mask on the 1 m cells of the tree cover to the outline of the
vegetation areas, which refines its grid threefold and blurs it, so every park was traced on a grid of 1/3 m. On
the 3 m cells of the vegetation the Bremen training export peaks at 329 to 350 MiB (two runs) instead of 1598 MiB
(maximum resident set, measured on 2026-11-15 on a quiet machine, load average below 5; 25.7 s before, 24 s after).
The Nationaldenkmal's point in Viktoriapark and a fountain at the Meierei in Bremen still go: no spot within 1.5 mm keeps the gap.

Left to the second half: carriageways, tram and railway tracks, paths along roads and kerb lines that run side by
side (in Bremen 1667 mm of joined pairs on the training map).

Source: spec `docs/superpowers/specs/2026-11-14-training-map-design.md` (decisions 1 to 13, Part 3), plan
`docs/superpowers/plans/2026-11-14-training-map-1.md`, the rulings ledger of that plan; ISSprOM 2019-2 101, 516,
518, 532, §3.3, §4.6; ISOM 2017-2 105, 401, 532.
