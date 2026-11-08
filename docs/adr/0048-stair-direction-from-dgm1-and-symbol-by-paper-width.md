# 0048. Stair direction from the DGM1, stair symbol by paper width

Date: 2026-11-09
Status: Accepted

## Context

ALKIS Berlin has stairs (`Treppe`, `Freitreppe`) as polygons. ISSprOM draws a stairway as a line along the
walking direction (532.x), with the footprint symbol chosen by the width (532.6 1.6 m to 532.9 4 m at 1:4000).
The polygon does not say which way is up. The export runs offline and has no cache, but the LGB DGM tiles live
only in the cache.

## Decision

- `build/stairs.rs`: the oriented minimum rectangle of a stair polygon gives two axes. The midpoints of the
  rectangle's sides are sampled bilinearly in the DTM, in map CRS. The axis with the larger height difference wins
  when it rises at least 0.3 m (`STAIR_MIN_RISE_M`); the line runs uphill, low end first.
- Otherwise, or without terrain: if an axis end lies within 0.5 m of a 521 footprint (`FACADE_DISTANCE_M`), that
  axis is used, running towards the building; else the long axis. Both cases are tagged
  `freemapper:direction=guessed`.
- Ruling (Task 3 review): the footprint symbol is chosen by paper width, not ground metres. The ground width is
  divided by (scale/1000 × symbol enlargement) and compared with 0.4/0.5/0.75/1.0 mm; the nearest wins (bounds
  0.45/0.625/0.875 mm). Sprint 1:4000 keeps 1.8/2.5/3.5 m; the schoolyard (1:1000, 125 %) uses 0.56/0.78/1.09 m,
  so most schoolyard stairs become 532.9. With the ground-metre bounds a 3 m schoolyard stair became 532.8, drawn
  less than 1 m wide on the ground.
- Stairs are candidates in `candidates.omap`. `Freitreppe` lines of `bauwerke-linien` are 532.7 rule candidates.
- Terrain: the per-area `data/dgm1.tif` when it covers the bbox; for tile providers `derive` keeps the DTM mosaic
  in `data/terrain.tif` (+ `.tfw`). Ruling (Task 3 review): the file is written only when a stored Berlin layer has
  a `Treppe` or `Freitreppe` area, and removed otherwise. Rejected: a cache argument on export (CLI and MCP
  change) and per-stair height files (splits the stair logic between derive and export).

## Consequences

- Stairs come with a direction the mapper can mostly trust; guessed ones are tagged and need a look.
- DGM1 at 1 m is smoothed, so short stairs (under about 2 m) often read flat and get `guessed`.
- `data/terrain.tif` costs about 4 MB/km² in Berlin projects with stairs.
- Cost if wrong (paper width): schoolyard stairs drawn wider than the field mapper would choose.

Source: ISSprOM 2019-2 symbol 532; `docs/regions/berlin-plus-reference.md` §3.6; branch `berlin-plus`, Task 3 and
its fix round.
