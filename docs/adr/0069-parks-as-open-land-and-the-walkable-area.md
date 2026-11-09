# 0069. Parks are open land, and the walkable area lies under the sprint map

Date: 2026-11-09
Status: Accepted

## Context

User feedback on the Viktoriapark and Sanssouci training maps (2026-11-09): white on white is hard to see,
the tree group forests must not cover the paths, and a park should be yellow open land with the white forest
or single trees on it. freemapper drew no open land from the land use. A park was the white paper, a forest
of a tree group (ADR 0066) was white too and needed a dotted 416 to show, and that 416 crossed the paths.

A second user decision of the same day: on the sprint map the whole area is first covered by the
"belaufbare Fläche" (walkable area) in the lower brown, and everything else lies on it. The colour comes
from the user's reference OCAD file `srb-sep_trc.ocd`: "Unteres Braun 30%", the colour with the lowest
priority in that file, CMYK 6/23/33/0, spot colour PMS 471 Braun 30%. These are exactly the values of
"Lower brown 30%" in our ISSprOM set (IOF colour definitions Rev. 4, ADR 0044).

What the norm says: ISSprOM 2019-2 knows no such base. Brown 30% is the colour of 501 paved area for light
traffic ("Colour: brown 30% for light traffic of vehicles or pedestrians"), and 505 unpaved footpath takes
"the same as for the symbol Paved area (501)". White is 405 forest. The walkable area is therefore a
freemapper rule after mapping practice, not a norm symbol.

In the bundled colour table (32 colours, priorities 0 to 31) Yellow 100% is 30 and Yellow 50% is 31, the
lowest. Lower brown 30% is 14 and lies above all yellow, green and olive colours and the water fills. A base
in colour 14 would cover parks, forests, out-of-bounds areas and water.

## Decision

Open land from the land use (`build/open_land.rs`, both map kinds):

| ALKIS land use | symbol | target |
|---|---|---|
| Sport-, Freizeit- und Erholungsfläche / Grünanlage, Park, Siedlungsgrünfläche | 401 | map |
| Landwirtschaft / Grünland | 401 | map |
| Landwirtschaft, other or none | 401 | candidate |
| Friedhof | 401 | candidate |
| Wald, Gehölz | 405 | map |
| Heide | 403 | map |
| Moor, Sumpf | 403 | candidate |

- Names are compared with `spelling_key`, so the Hamburg spellings match. Sports grounds, playgrounds and
  allotment gardens get nothing here (allotments stay 520, ADR 0049).
- The areas are clipped to map area plus buffer like the other land use objects. No cut at buildings,
  paving or water: their colours lie above yellow and white in OOM, OCAD and the PDF.
- On the schoolyard map a parcel off the school grounds is a 520 candidate and gets no open land.
- OSM `leisure=park` and `landuse=grass|meadow|village_green|recreation_ground` (closed ways) are 401
  candidates. Where an ALKIS 401, 403 or 405 map object lies, the OSM area is cut away; pieces under (1 mm)²
  or 1 mm wide go.

Tree groups (amends ADR 0066):

- The open land above counts for the 405/402 decision as the pitches and green strips did.
- Off paving a 402 or 405 is also cut at the paved candidates (501, 501.3, 501.5, 501.10, 501.11) and at
  the paths: 505.1 (0.55 mm), 505.2 (0.7 mm), 506 and 507 (0.27 mm), 501.6 to 501.9, each buffered by half
  its width on paper, map objects and candidates. No group area covers a path. A piece below the minimum
  goes back to the thinning.
- The 416 outline is drawn only where the base is white, on a map without the walkable area (the schoolyard).
  On the sprint map the white forest shows on yellow or brown. §416 wants the line for "a distinct forest
  edge"; for indistinct boundaries "the area edges are shown only by the change in colour".

The walkable area (sprint map only):

- A new colour "Lower brown 30% for walkable area" with the CMYK, RGB and spot colour mix of Lower brown 30%
  (0.3 × Brown 100%, knockout), appended at priority 32, behind all others. The table stays contiguous and no
  existing priority or symbol id changes. In OCAD it is colour 32, the last one.
- A new area symbol 990 "Walkable area (base fill)" (German: Belaufbare Fläche (Grundfläche)) in that colour,
  after the sheet symbols (id 189 in the sprint set). OCAD number 990.000.
- One 990 object over the paper rectangle that encloses map area plus buffer (the window the training PDF
  shows), so no white wedge is left where the grivation turns the buffer's box. It is tagged
  `freemapper:source=base` and is the only object of the map part `base`, written first. OOM opens with the
  default part as the current part.
- It is always drawn. It fills every gap that nothing else maps; a sprint map without it is incomplete. There
  is no switch. Deleting it in OOM is possible but not intended. freemapper-print only leaves the object out
  of its page centring, overhang and north-line bounds, by the tag and the part.
- The ALKIS 501.5 parcels stay map objects. They carry the provenance and lie above the yellow and green
  where the two overlap; on the base they share its tone. Curbs (501.1), edges and buildings stay above.
- Print sprint maps in colour: in greyscale the 30 % tint is close to the path areas and to paper white.
- The schoolyard map gets no walkable area. The user asked for the sprint map; the school set follows the
  white base of ISSprOM.

## Consequences

Examples, candidates and map together (the online examples fetched again on 2026-11-09, unchanged except
for the symbols below):

| | before | after |
|---|---:|---:|
| Viktoriapark 401 map objects / candidates | 0 / 6 | 5 / 29 |
| Viktoriapark 405, 402, 416 group objects | 43, 9, 49 | 41, 3, 0 |
| Viktoriapark cadastre tree points / trees in group areas | 835 / 994 | 1 128 / 634 |
| Sanssouci 401 map objects / candidates | 0 / 0 | 27 / 23 |
| Sanssouci 402 group areas | 48 | 38 |
| Sanssouci cadastre tree points / trees in group areas | 688 / 789 | 774 / 676 |
| Monschau 405 map objects | 0 | 153 |

- The parks of all eleven examples are yellow, the woods white, and the rest of the sheet brown.
- More trees stay points: the path cuts split many groups into pieces below the minimum.
- Sanssouci stays mostly yellow: ALKIS calls the whole park "Park", and the Potsdam trees have no crown, so
  their groups are 402. Wooded parts without tree data are yellow until the mapper draws them.
- Every unmapped gap now prints as paved-looking brown, for example a Fläche besonderer funktionaler Prägung
  or an unpaved Weg parcel. That is the user's rule; the mapper corrects it in the terrain.
- Cost if wrong: a park parcel with a wood in it is yellow, a meadow fenced off is open, a path cut through a
  forest leaves a gap where the canopy is closed, or a brown area looks paved where it is not.

Source: user decisions of 2026-11-09 (yellow parks, walkable area, not switchable, the reference file);
ISSprOM 2019-2 §4.4 (401, 403, 405, 416), §4.5 (501, 505); IOF Printing and Colour Definitions Rev. 4;
branch `park-open-land`.
