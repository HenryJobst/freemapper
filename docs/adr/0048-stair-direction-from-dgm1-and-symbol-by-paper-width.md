# 0048. Stair direction from the DGM1, stair symbol by paper width

Date: 2026-11-09
Status: Accepted

## Context

ALKIS Berlin has stairs (`Treppe`, `Freitreppe`) as polygons. ISSprOM draws a stairway as a line along the
walking direction (532.x), with the footprint symbol chosen by the width (532.6 1.6 m to 532.9 4 m at 1:4000).
The polygon does not say which way is up. The export runs offline and has no cache, but the LGB DGM tiles live
only in the cache.

`bauwerke-linien` also has `Freitreppe` lines. The live run of 2026-11-09 found 185 of them citywide, none
within 3 m of a stair area. They are step edges: 19 groups of parallel lines about 1.5 m apart, 0.6 to 7.8 m
long, some of them twice at the same place.

## Decision

- `build/stairs.rs`: the oriented minimum rectangle of a stair polygon gives two axes. The midpoints of the
  rectangle's sides are sampled bilinearly in the DTM, in map CRS. The axis with the larger height difference wins
  when it rises at least 0.3 m (`STAIR_MIN_RISE_M`); the line runs uphill, low end first.
- Otherwise, or without terrain: if an axis end lies within 0.5 m of a 521 footprint or an ALKIS monument area
  (`Denkmal, Denkstein, Standbild`, `FACADE_DISTANCE_M`), that axis is used, running towards it; else the short
  axis. Both cases are tagged `freemapper:direction=guessed`. ALKIS draws the Nationaldenkmal as one area over
  the plinth and its stairs, so every end of a plinth stair touches it. Ends within 5 cm of the nearest one are
  then told apart by their distance to the centre of that area, and the stair leads up to the monument.
- Ruling (final review): the guessed default is the short axis, not the long one. A flight long enough to be
  the long axis of a stair 3 m or more wide rises at least 1 m and shows in DGM1. A stair that reads flat is
  most likely wide. Before the change, the 8 plinth stairs of the Nationaldenkmal in Viktoriapark (14.6 × 1.7 m)
  ran along the plinth instead of up to it.
- A stair wider than 3 × the 532.9 width on paper (`WIDE_STAIR_FACTOR`, 12 m at 1:4000) is tagged
  `freemapper:size=wide` and drawn as parallel 532.9 lines across its width, one 532.9 width apart and centred
  (at most 100). A 134 m stadium stand gets 33 lines instead of one 5 m line.
- Ruling (Task 3 review): the footprint symbol is chosen by paper width, not ground metres. The ground width is
  divided by (scale/1000 × symbol enlargement) and compared with 0.4/0.5/0.75/1.0 mm; the nearest wins (bounds
  0.45/0.625/0.875 mm). Sprint 1:4000 keeps 1.8/2.5/3.5 m; the schoolyard (1:1000, 125 %) uses 0.56/0.78/1.09 m,
  so most schoolyard stairs become 532.9. With the ground-metre bounds a 3 m schoolyard stair became 532.8, drawn
  less than 1 m wide on the ground.
- Stairs are candidates in `candidates.omap`.
- Ruling (final review): `Freitreppe` lines are no rule candidates. Each line drew a 532.7 across the walking
  direction, so one stair became 8 to 37 parallel stair symbols. Now `build/stairs.rs` drops duplicate edges
  (both ends within 5 cm) and joins edges into one flight when they are parallel within about 15°, the middle
  of one lies within 2 m of the other's line (`STEP_GAP_M`) and they overlap along their length. A flight of
  two or more edges is one stair. It runs across the edges, through the middle of their extent, as wide as that
  extent and as long as the flight. The direction comes from the edges, so the stair is not `guessed`. The DGM1
  only decides which end is up, when it rises 0.3 m along the flight. A lone edge gives no stair.
- Terrain: the per-area `data/dgm1.tif` when it covers the bbox; for tile providers `derive` keeps the DTM mosaic
  in `data/terrain.tif` (+ `.tfw`). Ruling (Task 3 review): the file is written only when a stored Berlin layer has
  a `Treppe` or `Freitreppe` feature, and removed otherwise. Rejected: a cache argument on export (CLI and MCP
  change) and per-stair height files (splits the stair logic between derive and export).

## Consequences

- Stairs come with a direction the mapper can mostly trust; guessed ones are tagged and need a look.
- DGM1 at 1 m is smoothed, so short stairs (under about 2 m) often read flat and get `guessed`. A flat stair
  that is in fact long and narrow now gets the wrong axis; the tag points the mapper to it.
- Wide stairs come as many lines; the mapper may join them into an area or keep a few.
- `data/terrain.tif` costs about 4 MB/km² in Berlin projects with stairs. The disk plan counts it as the derived
  share of the optional `Part::BerlinLayers` (as large as the layers, 4 MB/km²), not of the terrain part.
- Cost if wrong (paper width): schoolyard stairs drawn wider than the field mapper would choose.
- NAS stairs (AdV 1620/1621 areas, states plus 1) use the same builder. Their bottom edge (1630, linked by
  `gehoert_zu`) gives the direction, which is then not `guessed`; without an edge the DGM1 decides as above, and
  `data/terrain.tif` is kept for it.

Source: ISSprOM 2019-2 symbol 532; `docs/regions/berlin-plus-reference.md` §3.6; branch `berlin-plus`, Task 3 and
its fix round, the Task 5b live run and the final review.
