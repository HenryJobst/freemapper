# 0136. Vegetation outlines traced on a blurred raster

Date: 2026-11-14
Status: Accepted

## Context

The vegetation from laser point clouds (ADR 0105) is classified in cells of 3 m, and its areas were the 0.5 contour
of a 0/1 raster of those cells, simplified by 1.5 m. Their outlines ran in stairs of a whole cell: measured on the
forest examples, the outlines of the point-cloud vegetation were 16 to 22 % longer than the same lines smoothed by
4.5 m, while contours, paths and water stayed under 4 % (the measure of ADR 0126). Karttapullautin smooths its
vegetation in the raster too (a median filter, `yellow_smoothing`). The user chose to fix it after the rock
generalisation, and freemapper-d3 left `derive/vegetation.rs` to this change.

## Decision

- `outlines` traces each class on a raster three times finer than the classification (1 m), the 0/1 raster of the
  class blurred with a Gaussian of half a cell (1.5 m, ruling), mapped to 0.25 to 0.75 so that the contours at
  every 0.5 cross only the edge, and padded with empty cells as far as the blur reaches.
- Half a cell: the stairs repeat every cell, and a Gaussian of half that damps them to under 1 %. A whole cell
  (tried first) shrinks a block of four cells (6 m × 6 m, the smallest area at 1:4000) below the 0.5 contour.
- The classification, the majority filter, the minimum area of 1.5 mm × 1.5 mm and the simplification by 1.5 m
  stay. The classes are traced one by one, so at a junction of three classes the areas may overlap or leave a gap
  of a metre; they are candidates.

## Consequences

- Corners of vegetation areas are rounded: a thicket of 36 m × 36 m loses about 80 m² of its 1152 m².
- Fewer areas stay: rounded spurs fall below the minimum area. Sanssouci keeps 264 instead of 305 vegetation
  areas, Monschau 681 instead of 758, Cochem 449 instead of 519, Saarlouis 412 instead of 482, the Viktoriapark
  138 instead of 154. The outlines of the forest areas of Geobasis NRW measure 13 % longer than their course
  smoothed by 4.5 m instead of 16 %; what remains is the shape of the classes at 10 to 20 m, not the stairs.
- The nine examples with point-cloud vegetation are rebuilt: the sprint maps Sanssouci, Viktoriapark, Monschau,
  Cochem and Saarlouis, the forest maps Rauener Berge, Siebengebirge, Cloef and Schmausenbuck.
