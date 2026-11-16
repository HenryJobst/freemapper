# Training map, clean base map and candidate layers (design)

Status: Accepted, first half merged (PR #139), second half built 2026-11-15, part 1 (candidate layers) built
2026-11-16 (ADR 0156), the light level of part 2 built 2026-11-16 (ADR 0157). The user asked for it on 2026-11-14 after looking at
the forest example Bremen Bürgerpark, where symbols are drawn on top of each other in heaps.

## Intent

freemapper has made one thing so far: a base map with a candidates template. The "training map"
(`<name>-candidates.pdf`) is that base map with every candidate drawn on it, nothing more. The two serve
different people and need different rules:

- The **base map** is working material for a mapper. Every object lies where its source has it, and nothing
  is missing. The mapper checks, corrects and copies.
- The **training map** is printed as it is and used without rework. It has to be readable: no symbols on top
  of each other, the norm's gaps and minimum sizes kept. For that it may merge, shift and leave out.

The user's decisions (2026-11-14):

1. The training map becomes a product of its own with its own map file, next to the base map.
2. Candidates of one kind go to separate layers, so that a mapper can look at one kind at a time.
3. The base map itself is cleaned lightly: objects that are duplicates or hidden by another object leave the
   map and go to the candidate layer of their kind. Nothing is shifted; what stays lies true to position.
4. The candidate layers start hidden; on opening, the map shows only the sure objects.
5. The readability problems found in the Bremen example must be solved at least for the training map. That
   makes the list below the acceptance of the training map, and the training map the first thing built.
6. The training map keeps every gap the norm asks for. Objects of the same kind are generalized first and
   shifted after; where a shift makes a new overlap, the objects are generalized instead. A shift stays within
   bounds.
7. Inside olive green areas (520) the training map draws no details: only buildings and water stay, and the
   contours run through. A way inside is drawn in full only where a crossing is possible. Of a way that allows
   none, only the beginning is drawn, so that a runner passing by is not surprised by a path or road branching
   off, and only where no impassable fence or wall stands in front of it; with one in front the map shows the
   barrier.
8. The ticks of fences and the half dots of retaining walls point inward where possible: towards the park or
   the olive green area, not towards a path or road. The ticks must not overdraw anything. Turning them to the
   other side is not a means against overdrawing, unless the same area lies on both sides; the rest is adjusted
   as far as the norm allows.
9. When everything is finished, the examples in the documentation show the training maps in general, as the
   best readable maps. Only where the text refers to another format specifically, the map of that format is
   shown.
10. Contours follow the norm on every map: they run as continuous lines through all symbols, buildings, water,
    ways and paved areas among them, and are cut out only where the norm says so (stairs and the small landform
    symbols). This takes back the contour cut at ways of ADR 0097, which the norm does not cover.
11. On the training map no single tree is drawn on a path; such a tree is shifted minimally, by at most 1.5 mm on
    paper (the bound of a point symbol; a line moves by at most 0.5 mm).
12. Street furniture (lamps, waste baskets, bicycle stands: the x of symbol 531) stays on the training map
    only away from street and square areas.
13. Inside a street area the kerb lines go; the carriageway gets the darker brown for heavy traffic, the
    pavement the light one.

## What the Bremen example shows

Measured in this session on `examples/bremen-buergerpark` (forest, 1:10 000, built with 0.20.0; lengths are
millimetres on paper). The numbers come from a throwaway script, not from freemapper, and are the yardstick the
conflict measure below has to reproduce in kind, not to the digit.

| Finding | Number |
|---|---|
| Earth banks (104, ALKIS `AX_Strukturlinie3D`, top edge) | 263 objects, 2209 mm |
| … closer than 0.4 mm to a ditch (305/306) | 30 % of their length |
| … closer than 0.4 mm to water (301) | 26 % |
| … closer than 0.5 mm to another earth bank | 31 % |
| … closer than 0.4 mm to a path or road | 14 % |
| OSM water areas (301) lying on ALKIS water (301.1) | 5 areas at 91 to 99 %, 613 of 781 mm² |
| Wide roads (502) closer than 0.8 mm to another 502 (carriageways) | 18 % of their length |
| Tram tracks (509) closer than 0.6 mm to a 502 | 13 % |
| The two railway tracks (509) | 0.39 mm apart, drawn as one band |
| Footpaths (505) | 416 pieces, 143 shorter than 2 mm, 15 % closer than 0.5 mm to another 505 |
| Stairways (532) | 53, 48 shorter than 1 mm; 6 of 23 OSM stairs lie on an ALKIS stair |
| Fences (518) closer than 0.4 mm to a path | 20 % of their length |
| Contours (2.5 m, flat land) | 160 pieces, 29 run through buildings, 9 % lie on an earth bank |
| Monument points (526) | 12, several in a row against buildings, 4 closer than 0.4 mm to a path |
| Open land (401) from the ALKIS land use "Park" in the Stadtwald | two areas, 13 % and 25 % open by the nDSM: forest drawn yellow |

All of these are **map objects** under the default profile `balanced`; the candidates of this example hold
almost only garden areas (520). The mess is in the base map, not only in the training map.

`drop_covered_water` (`build/water.rs`, threshold 0.5) should already drop the doubled OSM water. Why it leaves
it in Bremen is not known yet and is the first thing to find out in part 2.

## Goals

1. **Candidate layers.** The candidates are written as several template files, one per layer, each switched
   on and off on its own in OpenOrienteering Mapper; the OCAD output follows.
2. **A conflict measure.** One function says where drawn symbols come too close, by symbol pair and length. It
   is the yardstick of the tests and of the ADRs, and the two clean-up levels use it.
3. **A lightly cleaned base map.** Duplicates and hidden objects leave the map and go to their candidate
   layer. Map plus layers stay complete and true to position.
4. **The training map as a product.** `<name>-training.omap` and `<name>-training.pdf`, built from the map and
   all candidates by a stronger clean-up level that merges, shifts and leaves out until the conflict measure
   is quiet. freemapper-print and freemapper-courseplan read it like any map.

## Non-goals

- No new data sources and no new symbols.
- The base map keeps drawing a park as open land where ALKIS says so; only the training map looks at the
  trees (below). Whether the base map should follow is a question for later.
- No general cartographic displacement engine. Shifting stays limited to the cases named below.
- No change to what the content profiles mean (`safe`, `balanced`, `full`, `--content-set`).
- No rebuild of all 30 examples in parts 1 to 3; that is part 4.

## Anti-goals

- A base map whose objects no longer lie where the source has them. The light level never moves a vertex.
- Objects that vanish. Whatever leaves the map is in a candidate layer, tagged with the reason.
- A training map that hides its origin: it carries the same credit lines and the same tags
  (`freemapper:source`, `freemapper:source_id`) as the map, plus the reason where an object was changed.
- Output that differs between Linux, macOS and Windows (the float lesson of ADR 0097: compare with tolerance).

## Constraints

- The map assembly reads no file (ADR 0115). Both levels run on the built objects.
- Measures are types (`PaperMm`, `SymbolMm`, ADR 0118); gaps and minimum sizes come from the norm texts
  (ISOM 2017-2, ISSprOM 2019-2), not from estimates. The public IOF documents are the reference; the builder
  quotes paragraph and value in the ADR.
- The precedent is ADR 0126: `generalize_cliffs` runs in `Built::place` at a `Level` (map or training), what
  goes from the map goes to the candidates with `freemapper:generalized`, and is hidden from the training map.
  The new levels follow it, and the rock clean-up of the training level moves to the training map.
- OpenOrienteering Mapper draws all map parts at once and cannot hide one (checked 2026-11-14 in its manual);
  a layer that can be switched is a template file.
- Sheet objects keep `freemapper:source=sheet` (ADR 0080), so print and courseplan find them in the training
  map too.
- Every change that alters a drawn map goes through the golden-bless flow.
- User docs in German and English, terms in `docs/glossary.md` first: base map (Grundkarte), training map
  (Trainingskarte), candidate layer (Kandidaten-Layer), conflict (Konflikt), clean-up level
  (Bereinigungsstufe). An ADR per part, changelog entries under Unreleased (v0.21.0).

## Approaches

- **A, chosen: one conflict stage with two levels.** A step at the end of the placement looks at the drawn
  footprints, a table per symbol pair says who gives way and how. Level *map* only takes objects off the map;
  level *training* also merges, shifts and drops. One measure serves both and the tests. Cost: one new
  module that has to know every line and area symbol of both symbol sets.
- **B: every build module cleans its own kind** (water, earth banks, stairs, roads each get a level). The
  knowledge is close to the source, but conflicts between kinds (bank on ditch on path) have no home, more
  than ten modules change, and there is no common measure.
- **C: fix duplicates at the source and knock symbols out in the PDF.** Cheap, but it leaves the map file as
  it is and does not remove a single overlap.

The user left the choice to Claude; Claude took A. True duplicates between sources (the same lake from OSM and
ALKIS, the same stair twice) stay with the existing duplicate steps (`build/dedup.rs`, `build/water.rs`,
`build/stairs.rs`), which are repaired where Bremen shows a gap.

## Design

### Part 1: candidate layers

- Seven layers, by the colour families a mapper thinks in. Each object group (`Group`, `build/content.rs`)
  belongs to exactly one:

  | Layer | File | Groups |
  |---|---|---|
  | terrain | `templates/candidates-terrain.omap` | slopes, landforms, contours |
  | water | `templates/candidates-water.omap` | water-features |
  | vegetation | `templates/candidates-vegetation.omap` | vegetation, trees, hedges |
  | paths | `templates/candidates-paths.omap` | paths, streets, stairs, passages |
  | barriers | `templates/candidates-barriers.omap` | walls, fences |
  | features | `templates/candidates-features.omap` | monuments, furniture |
  | out-of-bounds | `templates/candidates-out-of-bounds.omap` | out-of-bounds |

  Candidates without a group (open land 401, paved areas 501, canopies 522, water areas, buildings) get their
  layer from a table by symbol code: 401 to vegetation, 501 to paths, 522 and buildings to features, 301 to
  water. A test asserts that every symbol code freemapper can write has a layer.
- All seven layers are always written, an empty one too, so that the template list of a map never depends on
  what a run found (Claude's call). The map file lists them as templates after the raster templates, in the
  order of the table.
- In OpenOrienteering Mapper each template is switched on and off on its own and has its own opacity; the map
  file stores both per template (`omap/writer.rs`, today `visible` and 0.7 for the one candidates file). Every
  candidate layer starts hidden, with an opacity of 0.7 for when it is switched on (the user's decision,
  2026-11-14: otherwise the map is overloaded; on opening it shows only the sure objects). The raster
  templates start as they do today. The manual says where the layers are switched on.
- `templates/candidates.omap` is no longer written for a new map, and `--force` removes one left by an older
  version together with the map it replaces. A map that is kept (a run without `--force`) is never edited: its
  template list still names `templates/candidates.omap`, so that file is refreshed as the union of the layers
  as long as the kept map lists it, the layer files are written beside it, and a note says that the layers
  show up after `--force` or when added by hand in OpenOrienteering Mapper (Claude's call).
- OCAD: `ocad/<name>-candidates-<layer>.ocd`, one per layer, in place of `<name>-candidates.ocd`.
- The candidates PDF (`--pdf-candidates`) and the hidden-candidate rules do not change: they see the union of
  the layers. The MCP tool `list_layers` lists the candidate layers by name; `preview_layer` is unchanged.
- The contract test in freemapper-core and the two sister programs are checked for the old file name.

### Part 2: conflict measure and the light level

**The measure** (`build/conflicts.rs`, new). Input: a list of map objects, the symbol set, map kind and scale.
Each object gets its drawn footprint from the symbol (stroke width with borders and ticks, area outline,
point extent; the renderer's primitives in `render/draw.rs` already know them). A table gives each pair of
symbol classes one of three relations:

- *free*: may overlap (a line over an area fill of another colour, a path over a bridge);
- *joined*: one network, may meet at ends and junctions but not run side by side inside the gap (path and
  path, road and path, ditch and ditch);
- *apart*: must keep the norm's gap (earth bank and ditch, earth bank and bank line, fence and path, track
  and track).

A conflict is a stretch of an object too close to another, longer than a minimum length, outside the
neighbourhood of a shared junction; for areas, the overlapping area. It has one of two grades: *overlap*, when
the drawn footprints cover each other, and *gap*, when they only come closer than the norm's gap. The result
lists conflicts with both objects, the pair, the grade, and length or area on paper. It is deterministic and
tolerant at the threshold.

**The light level** runs at the end of `Built::place`, before the ticks are oriented, on the objects routed to
the map. It only does three things, and none moves a vertex:

1. *Join.* Lines of one symbol and one source class that meet end to end are joined (as ADR 0126 does for
   cliffs), so the dash pattern runs through and short pieces stop being objects of their own. The joined
   object names the others in `freemapper:also`.
2. *Give way.* The principle, for every *apart* pair: where the drawn footprints cover each other (grade
   *overlap*), the lower-ranked object leaves the map for its candidate layer with
   `freemapper:generalized=<reason>`: whole, when more than half of it overlaps, otherwise only the stretch
   (the rest stays, if longer than the norm's minimum length). A *gap* conflict without overlap stays on the
   base map: both objects can still be told apart there. This is the user's "hidden by another object".
   Rank for the pairs Bremen shows: an earth bank gives way to a watercourse, a bank line, a railway, a road
   and a path; a fence gives way to a path or road it is drawn on; of two earth banks the shorter goes; a
   contour piece on an earth bank goes. *Joined* pairs (carriageways, tracks, parallel paths) are not touched.
3. *Too small to draw.* An object below the norm's minimum length or area for its symbol goes to its layer
   (stairs under the minimum, isolated ditch pieces, contour loops).

Parallel carriageways and tracks stay on the base map: they are real and true to position, and merging them
is the training map's business.

The duplicate steps are repaired first: why `drop_covered_water` leaves the Bremen lakes, and OSM stairs on
ALKIS stairs. A true duplicate, the same thing from two sources, is dropped as today and named in the kept
object's `freemapper:also`; it does not go to a layer (Claude's call: the anti-goal "objects that vanish"
means things that differ in the terrain, and a second outline of the same lake helps nobody).

What the light level takes off the map carries `freemapper:generalized` and no `freemapper:hidden`: the
candidates PDF draws it, because that print shows everything true to position. The cliffs of ADR 0126 follow:
their `freemapper:hidden=generalized` kept them out of the old training map and is no longer needed.

### Part 3: the training map

- **Files.** `<name>-training.omap` and `<name>-training.pdf` in the project folder; with `--ocad` also
  `ocad/<name>-training.ocd`. The map file lists the raster templates and no candidate layer.
- **Switch.** `--training` on `run` and `export`, `training: true` on the MCP tools `start_pipeline` and
  `open_area_selector`, a box on the area page. Not stored in `freemapper.toml`, like the other output
  switches. An existing training map is kept unless `--force` is given, the old one as `.bak`: the same rule
  as the map, because someone may have touched it.
- **Input.** Every object the content settings do not switch off: the map objects before the light level and
  all candidates that are not hidden (`freemapper:hidden`), as the candidates PDF takes them today.
- **The training level** ends with every gap of the norm kept: no conflict of either grade is left. It works
  in this order (the user's rule, 2026-11-14):
  1. *Generalize what is alike.* Objects of one kind are settled among themselves first: lines that meet end
     to end are joined; lines of one road class running side by side inside the gap become one line on their
     middle (carriageways, railway tracks, parallel paths); a tram track inside a drawn road and a footpath or
     cycleway inside the drawn width of a road are dropped; of two earth banks, fences or walls inside the gap
     the shorter goes; objects below the norm's minimum size go; points are thinned as today (ADR 0066).
  2. *Shift.* What is still inside the gap of an object of another kind is moved off by what is missing to the
     gap, the lower-ranked object moving (an earth bank beside a ditch, a fence beside a path, a point beside a
     wall as in `build/barrier_gaps.rs`). A shift stays within bounds: a line at most 0.5 mm on paper (5 m on
     the ground at 1:10 000, 2 m at 1:4000; Claude's ruling, to be tightened if the norm gives a value), and it
     keeps its shape; a point symbol at most 1.5 mm, which covers its own radius, the gap and half the widest
     way it is moved off.
  3. *Generalize where shifting fails.* Where the room is not there within the bound, or the shifted object
     would come inside the gap of a third one, it is not shifted: the stretch is cut out, or the object dropped
     when more than half of it is affected. A shift never makes a new conflict.
  The measure runs again after each step; the level stops when it is quiet, which the third step guarantees.
- **Inside olive green** (before the three steps, so that less is left to settle). The olive area is the union
  of the 520 objects of the training map. Buildings, canopies, water and watercourses stay, contours run through,
  and whatever lies on the edge stays (the fence or wall bounding the ground, the road along it). Everything
  else inside goes: areas are cut at the edge, points go, lines of barriers, earth banks, cliffs and stairs are
  cut at the edge. A way (road, path) inside stays in full where it is a crossing: its part inside meets the
  edge at two places or more, none of them barred by an impassable wall or fence. Otherwise its part inside goes,
  except a short beginning at each place where it meets the edge with no impassable wall or fence across it.
  The length of the beginning and how far inside "on the edge" reaches are the builder's rulings. A passage
  through a building (522 from a "Durchfahrt") follows the ways: it is drawn only where a way that stays runs
  through it; roofs and carports stay.
- **Contours by the norm** (for every map, taking back the contour part of ADR 0097). ISSprOM 2019-2, symbol
  101: "contour lines shall be represented as continuous lines through all symbols, also Building (521) and
  Canopy (522)", cut out only where they touch 105, 109, 110, 111, 112, 115 and Stairway (532). ISOM 2017-2 has
  no such sentence; contours are adapted or broken at 109 and 110 and broken around 108. The colour order of
  both symbol sets puts black lines, green and line blue over the contour brown, and building grey, paved brown
  and the blue of water areas under it: fences, walls and brooks lie on a contour, buildings, paved areas and
  lakes under it. So the contours and index contours are no longer cut at ways and paved areas; they stay cut
  at stairs, at the small landform symbols and below cliffs (ADR 0126). The north lines keep their cut at ways
  (ISSprOM §4.6).
- **Tick sides** (for every map, in `build/ticks.rs`, amending ADR 0097; turning a line moves nothing). The
  norm fixes two cases: the half dot of a retaining wall points to the lower level, and the tags of a fence that
  encloses an area stand inside. Beyond that the side follows the meaning, in this order: towards olive green
  beside the fence, away from a way running along it, towards the park. Only when none of these decides, which is
  when the same area lies on both sides, the side is taken whose ticks overdraw nothing. Where the ticks on the
  side given by the meaning overdraw something, the side stays, and on the training map the three steps above
  settle it. The norm lets tags be left out for earth banks close together and for vertical rock faces, not for
  fences, so fence ticks are never left out.
  - *Rock.* `Level::training` of ADR 0126 runs here. The candidates PDF goes back to the map level.
  - *Contours* follow the norm (below) and are not cut at buildings, water or ways. A contour piece on an
    earth bank goes, and a piece below the norm's minimum goes.
  - *Areas.* The minimum sizes hold for areas too: an area below the norm's minimum for its symbol goes
    (the small patches of open land and of the green shades).
  - *Trees.* A tree whose symbol touches a path line, a stair, a building, water or another tree is moved off
    by the least distance that frees it, within the bound of a shift; where that fails it goes. Trees on street
    and square areas stay: there is no room to move to, and the norm knows trees on paved areas.
  - *Street furniture* (531) on a street or square area goes; elsewhere it stays.
  - *Carriageways* (second half, with the road network): inside a street area the kerb lines (501.1) go, the
    carriageway is drawn as paved area with heavy traffic (501.10), the pavement as light traffic (501.5).
    Until that is built the kerb lines stay as they are.
  - *Open land under trees.* Open land that comes from a land use naming no ground cover (park, green space,
    leisure area) is drawn only where the nDSM shows no trees; the rest stays forest. The outline is smoothed
    and kept to the minimum sizes the vegetation areas use (ADR 0136). Without a surface model nothing
    changes.
  - *Points.* The point generalization (ADR 0066) runs on the points of the map and the candidates together,
    and a point keeps the norm's gap to buildings and ways as it does to walls and fences.
  - *Junctions survive.* An end that touched a line before touches the line that replaces it after: when
    lines are merged or shifted, the ends that met them are moved or extended to the new line. A path that
    ended on a dropped footway inside a road ends at that road. This is in scope; courseplan tells free ends
    from junctions by whether lines touch.
- **`--pdf-candidates` stays** and keeps its file name. It is the check print of the base map with every
  candidate, true to position; the docs no longer call it the training map.
- **Sister programs.** They take a map path and read `freemapper.toml` beside it; the training map needs no
  change there. The builder confirms it with one run each and tells their sessions.

### The findings on the training map

Every row of the Bremen table has its rule; the training map of Bremen Bürgerpark is accepted when each holds.

| Finding | Rule | Holds when |
|---|---|---|
| Earth banks on ditches, water, ways, each other | give way, shift, drop | no earth bank inside the gap of another object |
| Water twice | duplicate step repaired | no OSM water area on ALKIS water |
| Carriageways side by side | merge | one 502 per road; no 502 inside the gap of another outside junctions |
| Tram tracks in the road, footways along it | drop | no 509 and no 505 inside the drawn width of a 502 |
| Two railway tracks as one band | merge | one 509 per line |
| Footpaths in pieces | join | no 505 shorter than 2 mm that continues in another 505; the dash pattern runs through |
| Paths side by side | merge | no 505 inside the gap of another 505 outside junctions |
| Stairs too short, doubled | minimum size, duplicate step | no 532 below the norm's minimum, none twice |
| Fences on paths | shift, drop | no 518 inside the gap of a way |
| Monument points in a row | point generalization | every 526 keeps the norm's gap to points, buildings and ways |
| Contours as noise | drop on earth banks, minimum size (through buildings is the norm) | no contour on an earth bank, none below the minimum |
| Forest drawn yellow | open land under trees | the 401 of the Stadtwald covers only what the nDSM shows open |
| Details in the gardens around the park | inside olive green | nothing inside a 520 but buildings, water, contours and crossings |
| Fence ticks on paths | tick sides | no fence tick points to a way that runs along the fence, unless the norm fixes the side |

### What the Viktoriapark example adds

Measured on `examples/berlin-viktoriapark` (sprint, 1:4000), map and candidates together, which is what the
candidates PDF in the README draws.

| Finding | Number | Rule |
|---|---|---|
| Details inside olive green | 239 of 264 canopies and passages, 59 earth banks, 81 trees, 42 path pieces, 39 walls and fences | inside olive green |
| Passages into yards | 168 "Durchfahrt im Gebäude", nearly all leading into olive green | passages follow the ways |
| Trees on other symbols | 83 of 1286 touch a path line (16 centres on it), 15 on stairs, 10 at buildings, 9 at water, 17 touch another tree | trees are shifted |
| Street furniture | 144 x, 119 of them on street areas (91 lamps, 16 waste baskets, 12 pieces of furniture, 9 bicycle stands) | furniture only off street areas |
| Kerb lines | 270 lines, 7361 mm, two to four side by side in every street | carriageways (second half) |
| Small patches | under 4 mm²: 23 of 81 open land, 15 of 32 slow running, 6 of 6 walk, 13 of 96 forest | minimum sizes for areas |
| Stands of the stadium | rows of earth banks, stairs, fence and wall side by side | generalize what is alike, gaps |
| Contours | 5227 mm; 1456 mm through buildings, 2497 mm in olive green, 38 mm in water | by the norm: they stay |
| The PDF page | A3 portrait for a landscape map, half the sheet empty | not in this design: an issue of its own |

### Acceptance

Measured with the conflict measure on Bremen Bürgerpark (rebuilt in scratch) and on two other examples the
builder picks (one sprint, one rocky forest); the numbers go into the ADRs.

- Part 1: every candidate of today is in exactly one layer; the union equals today's `candidates.omap`.
- Part 2: no doubled water and no doubled stairs; the map has no *overlap* conflict of an *apart* pair left;
  the number of 505 objects falls by at least half; no vertex of a kept object has moved.
- Part 3: the training map has no conflict of either grade left and no object below the norm's minimum size;
  no object has moved by more than the bound; no line end that touched another line is left free.

The thresholds are Claude's proposal. If a rule needed to reach one costs more than it gives, the builder says
so in the ADR with the number reached.

### Part 4: afterwards

The examples are rebuilt with the layers and get a training map each. From then on the documentation shows the
training map wherever it shows an example in general: the previews in the READMEs and in `examples/`, the
example PDFs, the pictures of the manual. The base map, the candidate layers, the check print, the OCAD output
and the content profiles are shown where the text is about them; `examples/vergleich.md` puts base map,
candidates and training map side by side. The switch waits for the second half of the training map: until then
it still draws carriageways and tracks side by side. Further rules follow by what the measure shows on the other
examples.

## The second half, as designed on 2026-11-15

The first half is merged (PR #139, ADR 0140) and measured: no apart conflict is left on Bremen Bürgerpark,
Viktoriapark and Bastei, but only by cutting and dropping. What it leaves: carriageways, tracks and paths along
roads side by side (joined pairs), fences and walls cut where the data puts them in the gap of a stream or path
(Bremen keeps 104 of 386 mm of 518), kerb lines inside street areas, and path ends left free where a line was cut.
The second half settles these with the user's order: generalize what is alike, then shift, then cut.

- **Merge** (generalize what is alike, before the points move). Two lines of the same class (Road, Railway, Path)
  that run side by side inside the gap over a stretch of at least the class's minimum length become one line on
  their middle over that stretch; the wider symbol is kept, the tags of the longer line with the other in
  `freemapper:also`, and `freemapper:generalized=merged with <id>`. A railway line tagged `railway=tram` inside
  the drawn width of a Road goes. A Path inside the drawn width of a Road goes (sidewalks and cycleways drawn
  beside a road: the forest rules keep `footway=sidewalk`). The tram rule applies on the forest map only: on
  the sprint map a tram runs on the street area, which is Free.
- **Shift** (before cutting). For an apart conflict whose loser is a line, the conflicting stretch is moved away
  from the winner by what is missing to the gap, at most 0.5 mm on paper, with a ramp of four times the shift at
  each end of the stretch so the line keeps its shape and stays connected; a whole line is shifted when most of
  it is in conflict. The shifted line must make no new conflict with anything (lines, areas, points, contours,
  ticks): otherwise it is not shifted and give way cuts it as before. Walls and fences are shifted like other
  lines; their tick side does not change.
- **Junctions survive.** Before merge and shift, every line end that touches another line within `TOUCH_M` is
  recorded with the line and the position on it. After them, an end whose line moved or was replaced is moved to
  the nearest point of the line that replaced it, if that is within the shift bound plus the gap; a path that
  ended on a dropped sidewalk ends on the road. A free end is counted, not hidden.
- **Carriageways** (decision 13; ISSprOM 2019-2, 501: "Edges within paved areas are generally not represented,
  unless they serve navigation", "Differences in the brown colour shall be used to show differences in the
  traffic"). Only Berlin has a carriageway: the street survey (`cm_fahrbahn`) and its kerb lines
  (`sb-bordstein`). The carriageway areas inside a street parcel reach the training input as hidden candidates
  (`freemapper:hidden=carriageway`, so neither the map nor the check print draws them); on the training map they
  are drawn as 501.11 (paved area, heavy traffic, no border; 501.10 of the bundled set carries the border), cut
  to the 501.5 street area they lie in, and the survey kerb lines inside a street area go. The outer edge of the
  street area (the generated 501.1 border) stays. Elsewhere there is no carriageway in the data and nothing
  changes.

Acceptance on the same three areas: apart conflicts stay 0; joined conflicts (overlap and gap) fall by at least
80 %; 518 on the training map in Bremen keeps at least half of the check print's length; no line end that touched
a line before is free after; Viktoriapark has no survey kerb line inside a street area and shows the carriageways
in 50 % brown. ADR 0150 records it.

Ruling of 2026-11-15, after the final review: "joined conflicts fall by at least 80 %" is read as the joined pairs
the merge addresses, roads, railways and paths of one class (the way pairs). Over all joined pairs the second half
gives −58 %, −10 % and −53 %; the rest are pairs of other classes (water against watercourse, banks, barriers,
Berlin's stair steps) that no step of the second half touches. "No free end" is not met; ADR 0150 gives the counts
and names the follow-up.

## Left to the builder

- The names of types and functions, the shape of `Placed` and `StarterDocuments` for the third list, and
  whether the training list is built on request only.
- The rank order of the symbol classes beyond the pairs named, and the relation table, from the norm.
- How far a line may be shifted, and the minimum length of a conflict.
- Whether OCAD offers layers inside one file in the versions freemapper writes for; if so, they may replace
  the several candidate files. The OCAD output has not been checked in OCAD yet either way.
- The wording of the docs, the manual chapters and the area page.

## Order

The user's priority is the training map, so it comes first. It needs the conflict measure and the repaired
duplicate steps of part 2, which move with it, but neither the layers nor the light level: its input is the
map before the light level plus the candidates. Five pull requests, each merged after the user's yes:

1. Training map, first half: the conflict measure, the repaired duplicate steps, the files and the switch, and
   the rules that join, drop and cut (give way, minimum sizes, contours, open land under trees, points, rock).
   Between kinds it already keeps every gap, by cutting and dropping only.
2. Training map, second half: merge what is alike, shift before cutting, junctions survive. After it no
   conflict is left and the findings table holds.
3. Candidate layers (part 1).
4. The light level of the base map (rest of part 2).
5. Examples and comparison page (part 4).
