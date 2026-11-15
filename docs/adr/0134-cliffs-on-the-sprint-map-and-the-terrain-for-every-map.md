# 0134. Cliffs on the sprint map, and the terrain for every map

Date: 2026-11-14
Status: Accepted, amended by ADR 0140 (the training level runs on the training map, not on the candidates PDF)

## Context

ADR 0126 generalizes the OSM cliffs of a forest map, the map at one level and its training PDF at a stronger
one. The user asked whether the training map's own level holds for forest maps only (it did), and noted that
sprint maps have cliffs too. A measure over the 15 sprint examples, map and visible candidates together:

- The sprint training maps are not overloaded: at most 20 candidate objects on a square centimetre of paper,
  mostly areas that shall not be entered (520); at most 89 mm of wall, fence, bank or cliff line (Wismar: city
  walls of the cadastre).
- Cliffs occur in eight of them (Bautzen 10 on the map and 18 candidates, Marburg 33, Saarlouis 25, Heidelberg
  18, Blankenese and Cochem 10), at most 37 mm of cliff line on a square centimetre (Saarlouis, Heidelberg).

So on sprint maps the step is less about overload than about the same rules: the norm's minimum height and
length, the gaps, the smoothing, the side from the terrain, pillars in plan shape.

The step needs the terrain model at export. A tile provider's mosaic was kept only for forest maps and for Berlin
stairs (ADR 0126), so a sprint map in Saxony or Brandenburg had none: its cliffs would count as tall and keep
OSM's way round, and the ticks of its earth banks have never followed the terrain there.

## Decision

- `generalize_cliffs` takes the values of the map's norm (`Norm`): ISOM 2017-2 for forest maps as before;
  ISSprOM 2019-2 for sprint maps: 201 a line of 0.5 mm with tags to 0.75 mm (as the bundled symbol draws them),
  the gap between lines of one colour (§3.3, 0.15 mm; ruling, the norm names none for 201), 206 a gap of
  0.15 mm, 0.3 mm wide and 0.75 mm². The schoolyard keeps its cliffs.
- Levels by map kind: the sprint map takes the norm's minimum height 1.5 m and length 0.9 mm, pieces from 1.2 mm
  (ruling: the tags stand 0.75 mm apart); the sprint training map cliffs of 3 m, whole ones from 1.2 mm, pieces
  from 1.5 mm, no top lines (ruling). The forest levels stay.
- The training PDF of a sprint map runs the step at its training level, like a forest map's.
- `derive` keeps a tile provider's terrain mosaic (`data/terrain.tif`) for every map, and the export reads it
  when there is no `data/dgm1.tif`. The ticks of banks (104) and retained walls (513.2) on maps of tile
  providers follow the terrain from now on, as they did on maps of per-area providers.
- `derive` writes `data/rock-seen.tif` for sprint maps too (ADR 0126).

## Consequences

- The golden net changes in four states, re-recorded: Berlin (8 banks and 5 retained walls turned round, two
  trees moved off them, one candidate cliff turned), Brandenburg (only the new data files), Hessen and
  Saxony-Anhalt (the new data files and the training PDF). Each was compared object by object with main.
- A project of a tile provider stores the mosaic (about 4 MB per km²) whatever its kind.
- The sprint examples are rebuilt online.
