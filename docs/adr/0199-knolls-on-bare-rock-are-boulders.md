# 0199. A knoll on bare rock is a boulder

Date: 2026-11-26
Status: Accepted, extends ADR 0104 and ADR 0198

## Context

`derive` finds knolls (109, 110) as small rises of the terrain model. In a rock town each tower is such a rise, and
the map showed them as brown knolls on grey bare rock. A real map of the same country (the Nikolsdorfer Wände, Saxonia
Festival 2026, 1:10 000, 5 m contours) draws them as very many small black symbols. In ISOM 2017-2 a boulder (204) is
higher than 1 m and a large boulder (205) higher than 2 m; the gigantic boulder or rock pillar (206) is an area in plan
shape; bare rock (214) is for "a runnable area of rock" and is grey. Counted on the examples: of 640 knolls on Adršpach
522 lie in the OpenStreetMap rock areas (456 of them 2 m or higher), of 239 on the Nikolsdorfer Wände 81 (40 of 2 m or higher).

## Decision

- A knoll or an elongated knoll of 1 m or more whose point lies in a bare-rock area drawn so far (symbol 214 of the
  objects of the map and the candidates, which OpenStreetMap supplies as `natural=bare_rock`) becomes a boulder (204)
  from 1 m and a large boulder (205) from 2 m (`build/landforms.rs`). It keeps the tag of the landform, so the content
  settings place it in the group `landforms`, a candidate of the terrain layer by default, and it has no rotation.
- Depressions and knolls under 1 m stay as they were. A symbol set without 204 or 205 keeps the knoll.
- The height of a knoll is cut off at 6 m (`MAX_FORM_HEIGHT_M`), so a higher tower is a large boulder either way.
- Not built: the gigantic boulder (206) in plan shape, and rock areas from the orthophoto. Where OpenStreetMap has no rock
  area, the brown knolls stay.

## Consequences

- Adršpach: 520 of 522 knolls on rock become 65 boulders and 455 large boulders; the training map keeps about 500 and
  looks like a rock field. Nikolsdorf: 81 become 41 boulders and 40 large boulders, 11 survive the clean-up of the
  training map. The golden maps of the four states do not change (they have no bare rock).
- A tower of 2 m is not a large boulder in the strict sense of the norm (more than 2 m); the model of 1 to 2 m is
  rough. The candidates are to be checked in the terrain.

## Addendum 2026-11-27: boulder clusters (207)

Boulders closer together than their symbols allow cannot be drawn singly: 0.4 mm for the boulder (0.6 mm for the large one)
and 0.15 mm between symbols of one colour (ISOM 2017-2, gap rule) make 0.55 mm from centre to centre. Three or more
boulders in a chain of that distance or less become one boulder cluster (207) at their centre, as high as the highest of
them, tagged `freemapper:boulders` with their number (`build/landforms.rs`, `boulders_and_clusters`). One or two stay
single. The distance is taken on paper, so it is 8.25 m at 1:15 000 and 5.5 m at 1:10 000. Without the symbol 207 in the
set the boulders stay single. ISOM asks for boulders higher than 1 m in a cluster; every boulder here is.

Two real maps of the area gave the reason: the Nikolsdorfer Wände of the Saxonia Festival 2026 and the Adršpach map of 2014
draw a rock field as black dots and short black strokes, with small triangles where boulders stand too close. A map of
Adršpach of 2009 (ISSprOM, 1:5000, 5 m contours) draws the rock town as large black plan-shape areas instead (206); that
is the next step and not built here.

Measured effect: none at 1:10 000, little at 1:15 000. The knoll finder keeps its points at least 5.4 m apart (Adršpach: nearest
neighbour 5.4 m at the least, 18 m median; Nikolsdorf 6.4 m and 17 m), so no pair is closer than the 5.5 m of 1:10 000; at
1:15 000 (8.25 m) 2 to 4 % of the points have a neighbour that close, and three in a chain are rarer. The clusters wait for data
that separates single towers more densely, such as a point cloud. Kept because the rule is the norm's and cheap.

