# 0068. Overlapping walls, fences and hedges keep the stronger line

Date: 2026-11-10
Status: Accepted

## Context

In the Sanssouci training PDF thick black lines lay on top of or beside each other. The full ALKIS (NAS), its
paired wall edges (ADR 0063), OSM and the Berlin survey each give lines for one wall, fence or hedge, often
with different symbols. The duplicate rule of ADR 0063 only drops a line with the same symbol (80 % of its
length within 1.5 m). Measured on Sanssouci (candidates, parallel stretches of at least 2 m closer than 1 m):
162 line pairs, 71 of them an OSM `barrier=retaining_wall` (513.2) on an ALKIS wall (515), the rest OSM walls
and fences on ALKIS walls, ALKIS fences and hedges beside ALKIS walls. 486 m of wall and fence ran within
0.5 m of a building outline, and 34 trees stood within 1 m of a hedge.

ISSprOM 2019-2: §3.3 gap between line symbols of one colour 0.15 mm, the opening of fences and walls 1.0 mm;
§515 "An uncrossable wall or retaining wall"; §521 minimum gap between buildings and uncrossable features
0.4 mm.

## Decision

- `resolve_barrier_overlaps` (`build/barriers.rs`) runs in `starter_documents` after the duplicate chain and
  before the point generalization, on the OSM and cadastre candidates. It works on 515, 518, 410.1 and 513.2.
- Order: higher rank first (register > ALKIS > survey > OSM), then the stronger symbol: uncrossable wall 515 >
  uncrossable fence 518 > hedge 410.1 > passable retained wall 513.2. A solid construction beats a fence, a
  fence beats vegetation, and an uncrossable line beats a passable one, so a runner never gets an opening a
  more trusted line closes. Then the longer line, then the name.
- Each line, in that order, loses the sections that run parallel (at most 30° off) to a line taken before it
  and closer than the two symbols may come on paper: half their widths (515, 518, 410.1 0.4 mm, 513.2 0.6 mm)
  plus the 0.15 mm gap for two black lines; a green hedge and a black line only must not overlap. Lines of two
  ranks are one object seen by two sources, so there the reach is at least 1.5 m, the tolerance of ADR 0063.
  At 1:4000: 515 and 518 of two sources 2.2 m, OSM 513.2 and ALKIS 515 2.6 m, hedge and fence 1.6 m.
- Two lines of one rank with one symbol stay: two ALKIS walls 2 m apart are two walls with a passage between
  them, and the edges of one wall are paired before (ADR 0063).
- A section parallel to a building outline (521 map object) within 0.5 m goes: the outline draws that wall.
- The pieces left must be 1.0 mm long (the norm's opening of fences and walls). They keep the line's tags. The
  winner names the cut line in `freemapper:also`; a cut along a building is noted in `freemapper:generalized`.
- A tree point within 1 m of one of these lines goes, and the nearest line names it in
  `freemapper:generalized`. In a hedge the tree is part of the hedge; on a wall or fence its 0.8 mm symbol
  covers the line, and the uncrossable line matters more to a runner.
- The cut works on whole sections, never on a perpendicular meeting: a fence that ends at a wall stays whole.

## Consequences

Sanssouci, candidates and map, offline from the stored data:

| | before | after |
|---|---:|---:|
| parallel line pairs closer than 1 m over at least 2 m | 162 | 4 |
| wall, fence and hedge length | 27.9 km | 21.8 km |
| OSM 513.2 length | 3.1 km | 0.2 km |
| length within 0.5 m of a building outline | 486 m | 79 m |
| trees within 1 m of a hedge | 34 | 0 |

Viktoriapark: 5 pairs before, 1 after. The 4 Sanssouci pairs left are ALKIS walls of one rank, edges the
pairing did not join.

- The OSM retaining walls on the Sanssouci terraces give way to the ALKIS walls. ALKIS calls them "Mauer", so
  the terraces print as uncrossable walls (515); `freemapper:also` names the OSM way. The mapper decides on
  the spot which terrace walls a runner can jump down (513.2).
- Cost if wrong: a real fence 2 m beside a wall of another source disappears, a hedge planted along a fence
  is cut, a tree at a wall is missing, or a wall that only runs close to a building is shortened. The norm
  wants such pairs displaced, which is the mapper's job.

Source: user request of 2026-11-10; ISSprOM 2019-2 §3.3, §515, §521; branch `dense-trees-walls`.
