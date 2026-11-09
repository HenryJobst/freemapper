# 0075. Obvious memorials from OSM, without Stolpersteine and plaques

Date: 2026-11-10
Status: Accepted

## Context

freemapper read no OSM memorials. The census found 27 memorial nodes in the Viktoriapark box (23 of them Stolpersteine)
and 110 in the Lübeck Altstadt (95 Stolpersteine, 7 plaques). The task asked to keep memorials out unless a clear symbol
exists, and to exclude Stolpersteine and plaques wherever memorials are read.

What the norm says (ISSprOM 2019-2, 4.5): 526 Cairn, memorial, small monument or boundary stone (P): "Cairn, memorial,
small monument or boundary stone that is obvious. Large massive monuments shall be represented in plan shape with the
symbol Building (521)." Footprint 4 m. The word "obvious" decides: a Stolperstein is a 10 cm brass plate set flush into
the pavement, and a plaque hangs on a wall. Neither can be seen as an object from a few metres, so neither serves
navigation.

## Decision

- 526 exists and names memorials, so a clear symbol exists. `historic=memorial|monument|wayside_cross|boundary_stone` nodes are
  **526 candidates** on sprint maps (candidates only, no schoolyard symbol).
- Excluded in the rule: `memorial=stolperstein|stolperschwelle|plaque|plate|ground_plaque|blue_plaque|bench|ghost_bike|mural|tree`
  and the old tag `memorial:type=stolperstein|stolperschwelle|plaque`. The bench, the ghost bike, the mural and the tree are
  memorials by purpose, not objects with their own shape. A node with no `memorial` tag stays in.
- Only nodes. An area memorial is a building or a mapper job (521). `tourism=artwork` is not a memorial and stays out.
- The exclusion sits in the one place where OSM memorials are read, the rule file. The extract reader never keeps a
  Stolperstein, so none reaches the stored candidates (`wants` is false). The Berlin layer `Denkmal, Denkstein, Standbild`
  and the Brandenburg NAS monuments come from cadastres, not from OSM, and have no Stolpersteine.
- The 526 point has a size in `build/points.rs`, so the gap between point symbols holds for it.

## Consequences

- Where a cadastre also has the monument (Berlin, Brandenburg), the OSM node is a second 526. The mapper keeps one.
- Existing maps get them with `run --force`.
- Amendment (ADR 0076): a memorial whose node lies inside a building footprint is dropped, on sprint maps.
- A memorial tagged wrongly (a Stolperstein as plain `historic=memorial`) gets through. The census shows the tagging is
  consistent for Stolpersteine.
