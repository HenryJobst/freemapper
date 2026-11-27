# 0200. Steep patches are rock faces in plan shape

Date: 2026-11-27
Status: Accepted, extends ADR 0198 and ADR 0199

## Context

Real maps of rock towns (the Adršpach sprint map of 2009, the Nikolsdorfer Wände) draw the rock faces as large black
areas in plan shape. ISOM 2017-2, 206: "A gigantic boulder, rock pillar or massive cliff shall be represented in plan
shape", minimum width 0.25 mm, minimum area 0.3 mm², black. freemapper drew only lines (201) and, after ADR 0199, boulders;
the plan shape existed for OpenStreetMap cliffs (`build/cliffs.rs`) but never from the terrain. Measured on the 1 m
terrain model of Adršpach (1.6 km², 2026-11-27): ground at least 70° steep is 5.2 % of the area, in about 500 patches of
25 m² or more; at 65° it is 8.4 %, at 75° 2.5 %, and the largest patch grows from 2 000 m² (70°) to 9 000 m² (65°).

## Decision

- `derive::landforms::rock_faces` traces the contour of the slope raster at 70° (`DEFAULT_FACE_SLOPE_DEG`). A closed
  ring is a face when its area is positive: the contour rings run with the steeper ground on the left, so a ring around
  less steep ground (a hollow inside a face) has a negative area and is dropped, which fills the hollow.
- A face needs at least 0.3 mm² and a width of 0.25 mm on the map (twice the area over the perimeter), then it is
  simplified by 0.5 m. A ring cut by the raster border or by cells without data is not closed and is left out.
- The landform kind `rock-face` is an area (`LandformKind::is_area`, a GeoJSON polygon of its outer ring). `build`
  draws it as 206, tagged like the other landforms: a candidate of the terrain layer in the group `landforms`.
- A bank or cliff that runs mostly within 2 m of a face is dropped (`in_a_face`): the area shows it already.
- Not built: contours broken at the black area (ISOM 0.15 mm gap), and white lanes through a face.

### Addendum 2026-11-27: the setting

The limit is a setting of the project, built like the steepness threshold of ADR 0198: `face_slope_deg` in
`freemapper.toml` (55° to 85°; none: the default of 70°), `--face-slope <degrees>|auto` on `init`, `derive` and `run`,
`face_slope` in the MCP tools `create_project` and `start_pipeline` (0: default), the field "Face threshold" on the
area page (automatic 70°, 65°, 75°). It counts for the project fingerprint. `DEFAULT_FACE_SLOPE_DEG` stays the default,
so existing maps and the golden files do not change.

## Addendum 2026-11-27: contours stop at the faces

Contours are cut out where they run through a rock face (206 is one of `CONTOUR_CUT_OUT_CODES` in
`build/line_cuts.rs`), as at small knolls and stairs; pieces shorter than 1 mm on paper are dropped. On Adršpach the
contour objects of the base map grow from 280 to 666 (the lines are cut into pieces) while the drawing hardly
changes, because the black area covered them; the brown lines no longer run into the black, as the norm wants
(contours and cliffs interplay, no brown line over black). Thinning of crowded contours is not built on purpose: the norm allows one interval per map, and with the
interval of 10 m on steep forest maps (ADR 0125) the contours come closer than the minimum gap of 0.29 mm only from
about 74° on, which the faces cover from 70° (see `docs/research/orienteering-specifications.md`).

## Consequences

- Adršpach: 310 faces; the training map shows black wall areas between the boulders, close to the real maps.
- A steep embankment of a quarry or a road cutting is a face too; the candidate has to be checked like every other one.
- The 1 m raster cannot show a wall narrower than about 2.5 m, so a narrow crack or a thin wall stays a line (201) or is
  lost.
