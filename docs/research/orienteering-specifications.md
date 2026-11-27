# Orienteering map specifications: reference list

The documents that say how an orienteering map is drawn, with what was read of each and what the
implementation is to be compared against. Started 2026-11-26 while looking for the rules on contours in
steep land (ADR 0198). The PDFs are not stored in the repository (the IOF holds the copyright); this list
names them, so that a later comparison reads the same editions.

Status of a row: **read** (the text was extracted and read in full for the passages named), **cited**
(an ADR of this repository quotes it, not read again here), **reported** (named by a person or a search
result, the wording not seen: treat every statement as unverified).

## The IOF specifications

| Document | Edition | Where | Status |
|---|---|---|---|
| ISOM 2017-2, International Specification for Orienteering Maps | adjusted version of January 2019, errata to 20.01.2020 | <https://o-sport.de/assets/dokumente/allgemein/kartenwesen/isom2017-2.pdf> (SHA-256 `31ec70bd72172a451249bf65c7014442bc0b28609cbbce178f5c6cecb5754650`) | read: §2.10, symbols 101, 102, 103, 104, 201, the general gap rules |
| ISOM 2017-2, Revision 6 | January 2024 | <https://baoc.org/wiki/images/d/db/ISOM_2017-2_%28Revision_6%2C_January_2024%29.pdf> (the address answered with the club's home page on 2026-11-26) | not read; a search result quotes the same bend and mouth rules |
| ISOM 2017-2 on the O-Map wiki | current | <https://omapwiki.orienteering.sport/specifications/isom/> | reported (the wiki refused automatic access, HTTP 403) |
| ISSprOM 2019-2, International Specification for Sprint Orienteering Maps | January 2022 | the PDF of the IOF; sections §3.1, §3.3, §4.4, §4.5 are quoted in ADR 0066, 0068, 0072, 0073 | cited |
| ISMTBOM 2022, International Specification for Mountain Bike Orienteering Maps | 2022 | <https://www.swiss-orienteering.ch/files/kommission_karten/ISMTBOM_2022_IOF.pdf> | reported: said to allow 10 m in steep land; not a foot-orienteering rule |
| ISOM 2000, Darstellungsvorschriften of the Swiss federation | 2000 | <https://www.swiss-orienteering.ch/files/kommission_karten/isom2000_solv.pdf> | reported: said to name 2.5 m and 10 m next to 5 m; an old edition |
| SOLV, Herstellung OL-Karten | n/a | <https://www.solv.ch/files/SO_Herstellung_OL-Karten_Web_opt.pdf> | reported: one interval on the whole map |
| FISO document on sprint maps | 2025 | <https://fiso.it/_files/f_media/2025/10/39251.pdf> | reported: sprint maps 2 or 2.5 m, 5 m in exceptions |
| Scale and contour interval (Orienteering Portugal) | n/a | <https://orienteering.natourtrail.pt/maps/scale-and-contour-interval/> | reported: foot orienteering 5 m and 2.5 m only |
| O-Map wiki, contour lines (glossary) and symbol pages | current | <https://omapwiki.orienteering.sport/glossary/contour-lines/> | reported (HTTP 403) |

A map of the Nikolsdorfer Wände (Saxonia Festival, middle distance of 2026-09-13, 1:10 000, contour interval 5 m, surveyed 2024 to 2026): <https://livelox.blob.core.windows.net/map-images/240886/thumbnails/1024/457b98e0dfef4d3a819e1797827f8851> (reported by the user and viewed on 2026-11-26; not stored in the repository, the map forbids reproduction without consultation). It shows a rock field drawn as very many small black symbols and 5 m contours. A freemapper map of the same area was built on 2026-11-26 for the comparison (not kept).

Czech map archive, for comparison with the Adršpach example (viewed 2026-11-26/27, not stored; the archive warns against use for events without the publisher's permission): the map `Adršpašské skalní město` of 2009 (<https://mapy.ceskyorientak.cz/map/adrspasske-skalni-mesto-2009>: ISSprOM, 1:5000, contour interval 5 m, KOB Dobruška, drawn by Tomáš Leštínský, 1.01 km², OCAD 7) draws the rock town as large black plan-shape areas with white lanes; a foot-orienteering map of the same rocks of 2014 (shared by the user) draws a field of black dots and strokes with 5 m contours.

Topographic maps, for comparison only (no orienteering rule): swisstopo contour lines
(<https://www.swisstopo.admin.ch/en/contour-lines>, 10 m at 1:10 000, 20 m in the Alps at 1:25 000), the
traditional maps of the German Alpine Club (<https://www.alpenverein.de/verband/huetten-und-wege/kartographie/traditionelle-alpenvereinskarten>, 20 m).

## What ISOM 2017-2 (January 2019) says, quoted

- §2.10 Contour interval: "The contour interval for orienteering maps is 5 m. In flat terrain where the slope
  is less than 5% (or the contours would be more than 7 mm apart) all over the area, 2.5 m contours may be
  used. Different contour intervals shall not be used on the same map." The text names no larger interval.
- Symbol 101 Contour: "The smallest bend in a contour line is 0.25 mm from centre to centre of the line
  (footprint 4 m). The mouth of a re-entrant or a spur must be wider than 0.5 mm from centre to centre of the
  line (footprint 8 m)." "Small details on contours should be avoided because they tend to hide the main
  features of the terrain." Line width 0.14 mm. A depression has at least one slope line and is at least 1 m
  deep; a contour knoll is at least 0.9 mm by 0.6 mm, a depression at least 1.1 mm by 0.7 mm.
- General gaps: "The minimum gap between line symbols including outlines of area symbols of the same colour is
  0.15 mm", with exceptions for joins and crossings of network symbols, for contours (101) crossing earth
  walls (105, 106) and erosion gullies (107), and for contours (101) and earth banks (104).
- Contours and cliffs: "Contours and cliffs should at least partly overlap" (an exception to the gap between
  line symbols of different colours); symbol 201: "An impassable cliff should interplay with the contour
  lines." The gap between two impassable cliffs, or a cliff and another impassable feature, "must exceed
  0.25 mm on the map".
- Symbol 103 Form line: only one between neighbouring contours; "the presence of a form line between contours
  makes the terrain appear nearly twice as steep".
- The text of this edition says nothing about contours that come too close on steep slopes and nothing that
  allows 10 m.

## Open

- Whether a later edition (Revision 6, January 2024) or the wiki allows 10 m in very steep terrain: the wiki is
  said to, the Portuguese source says it does not, the edition read here does not. Read Revision 6 before
  documenting 10 m as a norm. ADR 0125 chooses 10 m for steep forests on its own account.
- The rule for contours that merge in steep land is not found in the edition read. The 0.15 mm gap between
  line symbols of one colour and the contour width of 0.14 mm give a centre spacing of at least 0.29 mm.

## To compare with the implementation

| Rule | Where it lives |
|---|---|
| Contour interval 5 m, 2.5 m flat, one interval per map; 10 m for steep forests (ADR 0125) and 2 m sprint, 1 m school (ADR 0116) | `derive/contours.rs` (`contour_interval_for`, `CROWDED_SPACING` of 0.35 mm), `project.rs` |
| Smallest bend 0.25 mm, mouth of a re-entrant 0.5 mm, small details removed | `derive/contours.rs` (`generalize`, `MIN_LENGTH_MM`, `MIN_RING_AREA_MM2`) |
| Minimum gap 0.15 mm between lines of one colour, 0.25 mm between impassable cliffs | `build/line_cuts.rs`, `build/cliffs.rs`, ADR 0063, 0066, 0068 |
| Contours and cliffs overlap and interplay | `build/cliffs.rs`, `build/training/` |
| Knoll, depression and elongated knoll minimum sizes | `derive/landforms.rs`, `build/landforms.rs` |
| Index contour every fifth contour | `build/contours.rs`, ADR 0116 |
| Form lines used sparingly | not drawn by freemapper |
