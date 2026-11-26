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
