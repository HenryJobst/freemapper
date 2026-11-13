# 0122. A sliver is measured by its short side

Date: 2026-11-13
Status: Accepted

## Context

The areas drawn from the land use drop slivers: 520 areas, paved areas and the pieces cut from them, water
pieces and 401 open land. A piece is a sliver when it is smaller than (1 mm)² or narrower than 1 mm, both
enlarged like the symbols (ADR 0118); survey pieces count down to 2 m on the ground (ADR 0053).

"Narrower" was the mean width, 2·area/perimeter (ADR 0036). That is the width of a long strip but half the side
of a square, so a square piece needed a side of 2 mm to count as 1 mm wide. On a sprint map the gap is 4 to 8 m
and showed little. On a forest map 1 mm of the norm is 15 m (1:15 000, and 1:10 000 with the symbols at 150 %):
house plots up to 30 × 30 m fell out of the 520 areas and left white holes in the settlements, and white reads
as forest. The Dammheide example, rebuilt after ADR 0118, lost 33 plots of 430 to 2500 m² and 8 areas of open
land this way.

## Decision

- The width of a piece in the sliver test is its short side: the short side of the rectangle with the area and
  the perimeter of the piece, holes included (`short_side` in `build/out_of_bounds.rs`). That is the width of a
  strip and the side of a square. A shape rounder than a square has no such rectangle and gets the side of the
  square with its area.
- `LandClip::is_sliver` and the open-land rule (`drop_open_land_under_alkis`) use it. The limits stay: (1 mm)²
  and 1 mm as symbol measures, and the 2 m cap of ADR 0053.
- The mean width stays for the water rule of ADR 0036, where a part under 2 m mean width is a candidate. It
  sorts watercourses, which are strips; a pond of 3 × 3 m stays a candidate.

## Consequences

- A compact piece with a side between 1 and 2 mm is kept where it was dropped. The short side is never below
  the mean width and at most twice it, so no piece that was kept is dropped now.
- Forest maps: the settlements are closed 520 areas again. Dammheide at 1:10 000 keeps 144 of
  its 146 areas of 520 (113 without this decision) and 9 of its 10 open-land areas (2 without).
- Sprint maps change a little. The three golden maps that change gain 16 small 520 candidates of 39 to 98 m²
  (Berlin 1, Hessen 2, Saxony-Anhalt 13), 4 paved areas (501.5) and 1 open-land area; contours, paved borders
  and north lines are cut at the new pieces.
- A ragged outline still narrows a piece, because the perimeter counts.
- Cost if wrong: a few small pieces more to delete.
