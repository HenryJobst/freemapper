# 0036. Water from ALKIS as fill and bank lines, water from OSM as candidates

Date: 2026-11-08
Status: Accepted, amended by ADR 0070

## Context

No map showed water. ALKIS land use has the water parcels (`Fließgewässer`, `Stehendes Gewässer`, `Hafenbecken`,
`Meer`) but no depth, and one river can lie in many parcels (33 in one Hannover area). Brandenburg, Saxony and
Saxony-Anhalt give no `bez`, so a `Graben` looks like a `Fluss` there. freemapper reads no OSM relations, so OSM
multipolygon rivers are lost. ALKIS-vereinfacht `GebaeudeBauwerk` has no bridges.

## Decision

- ALKIS water becomes uncrossable water (301) on both map kinds, never 302. The fill (301.1) and the bank line
  (301.6) are separate objects. freemapper dissolves the wide parts, clips the shape to the map area plus buffer and
  removes the bank line segments on the clip rectangle. Islands keep theirs.
- A part with a mean width (2·area/perimeter) under 2 m, the ISSprOM 305 limit, is a candidate. This replaces `bez`.
- OSM water areas, fountains, wells, springs, streams and ditches are candidates. freemapper drops an OSM area when at
  least 50 % of its clipped part lies in the ALKIS water (`OSM_WATER_COVERED`).
- Bridges come from OSM only: `man_made=bridge` areas are 501.5 candidates on sprint maps. OSM relations, river
  centre lines, marsh (307 to 310) and `natural=coastline` stay out of scope.

## Consequences

- No black lines cross rivers at parcel boundaries or run along the map edge, which the combined 301 would draw.
- The mapper fixes parcel edges that include the bank slope, bank lines across bridges and shallow water (302).
- Sea missing from ALKIS stays white. Existing maps get the water only with `run --force`.
- Cost if wrong: crossable ponds drawn as uncrossable; parcels that do not quite meet draw a bank line across a river.

Addendum (2026-11-08): the contour candidates (101/102) stop at the drawn ALKIS water, and OSM 311 points inside
that water or inside a kept OSM 301 area (mostly fountain jets in a basin) are dropped (commits 3593992, f3f37fb).

Addendum (2026-11-08, final review): holes get the same 2 m width cap as the water, so an island or mole of 2 m mean
width keeps its bank line at 1:4000 (f43bf6b). Parcel parts are cut to the clip window plus 5 m after the width test
and before the union, so a huge `Meer` parcel costs what the map needs. The cut keeps the vertices inside exact, so a
cut parcel still meets its neighbour (066a3ff, 51e2bb1). The water rules skip only the tunnel
values, so `tunnel=no` keeps a stream or pond (ee21c5d).

Addendum (2026-11-08, Mainz): at the Rhine the Rhineland-Palatinate and Hesse parcels meet along the state border
in mid-river without sharing their vertices, and the union kept the seam as a bank line through the river. The
"parcels that do not quite meet" cost above came true. After the union freemapper now closes every gap, inlet and
hole narrower than 0.5 m (`SEAM_TOLERANCE_M`), for parcels of one state as well. It takes the pieces that a
morphological closing (grow by 0.25 m, shrink by 0.25 m, mitre joins) adds and joins only those to the union, grown
by 1 cm to overlap both sides and cut back to the closing, so every other vertex stays where it was and the shore
stays straight where a gap meets it. The narrowest real gap, a bridge of 0.8 m, stays land, and so does an island.
Dropping bank segments near
other water was the alternative; it would have left the white gap in the fill and in the water that contours and OSM
duplicates are tested against (branch `sheet-colours`).

Source: plan `docs/superpowers/plans/2026-11-08-water.md` (Rulings, Global Constraints); spec §4.5 (Water); commits
71e3937, ee73610, 3a99a31, 599c414, 0cbbe35 and 19fe4c1 on branch `water`.

Addendum (2026-11-10): ADR 0070 replaces the sentence "freemapper reads no OSM relations". OSM multipolygon relations are read
now, so water areas that are relations become candidates like water ways. Water relations of other types (rivers as
`type=waterway`) stay out of scope.
