# 0073. Access restrictions and firm gravel from OSM

Date: 2026-11-10
Status: Accepted

## Context

The OSM census (2026-11-10) found about 38 private or closed footways and service ways in the Viktoriapark box and about
35 private plus 29 `foot=no` ways in the Lübeck Altstadt. freemapper read none of the access tags. A private courtyard
path was drawn like a public one, as a 505.1 candidate or even as a map object. Areas with `access=private` (company
yards, private gardens, school grounds) got the ground symbol of their land use.

The census also found that `surface=compacted` made a 505.1 map object, which is the largest source of false lines in a
park: Berlin has 43 compacted footways in the Viktoriapark box.

What the norm says (ISSprOM 2019-2, section 4.5):

- 520 Area that shall not be entered: "An area that shall not be entered such as a private area, a flower bed, a railway
  area etc. No feature shall be represented in this area, except very prominent features [...]". Minimum width 0.25 mm
  (1 m at 1:4000), minimum area 0.25 mm² (4 m²).
- 501 Paved area: "an area with a firm surface such as asphalt, hard gravel, tiles, concrete or the like".
  501.6 to 501.9 are the line forms for footprints of 1.4, 2, 3 and 4 m.
- 505 Unpaved footpath or track: "a way for passing mainly by foot, without a smooth, hard surface".
- 506 Small unpaved footpath or track.

## Decision

Access (sprint maps, candidates; rules in `osm-candidates.toml` before every path and ground rule):

- `highway=footway|path|pedestrian|steps|bridleway` with `access=private|no`, or with `foot=private|no`, and
  `highway=service|track|cycleway` with `access=private|no`, or with `foot=private`, become a 520 candidate: a strip along
  the way (the `corridor_m` field of ADR 0072), 2 m wide for footways and 3 m for service roads, tracks and cycleways, or
  the `width` tag. The passable-path rules after it no longer see the way, so the 505.1 line is gone.
- `foot=yes|permissive|designated` overrides `access` and leaves the way as it was. `access=destination`, `customers`,
  `permit` and `delivery` let a runner in and change nothing. `foot=no` alone on a service road, track or cycleway stays
  as it was: it is a paved road a runner can cross, not a closed area.
- `access=private|no` on a closed way or relation with `landuse` (except `reservoir` and `basin`), a leisure tag
  (garden, pitch, playground, sports centre, golf course, track, stadium, park, recreation ground, dog park, miniature
  golf, horse riding, ice rink) or an amenity tag (parking, school, kindergarten, childcare, college, university,
  hospital, graveyard, social facility, fuel, car wash, marketplace) becomes a 520 candidate of its own shape. A private
  swimming pool stays water (301).
- A 520 candidate hides the candidate points inside it (ADR 0066), so a bollard in a private yard stays out of the
  training map.
- `symbol_for` now skips a rule that has no symbol for the map kind. Before, such a rule still ended the search, so a
  sprint-only rule placed before a schoolyard rule would have hidden it. The schoolyard maps are unchanged: they
  have no 520 rule.

Surface and width:

- `surface=compacted|fine_gravel` is firm ("hard gravel" in 501). On `footway`, `path`, `bridleway` and `track` it
  becomes a **501.6 to 501.9 candidate** by `width` (under 2 m or none: 501.6; a track without width: 501.7; 2 m: 501.7;
  3 m: 501.8; 4 m: 501.9), no longer a 505 map object. The mapper compares it with the orthophoto: a wide firm path is
  also a paved area from ALKIS or the survey.
- `surface=unpaved|dirt|earth|ground|grass|gravel|sand|woodchips` stays a map object: 506 under 1.5 m wide or with
  `trail_visibility` intermediate or worse, 505.2 from 3 m (new), else 505.1 (tracks 505.2). `gravel` is loose gravel.
- `tracktype` decides a track without `surface`: grade1 is paved (no candidate), grade2 a firm track (501.7 candidate),
  grade3 to grade5 an unpaved track (505.2 candidate). A `surface` tag wins.
- `width` is read as before for footways, paths, bridleways and tracks. It is not read for steps and paved service
  roads, because the ALKIS and survey areas give their width.

## Consequences

- In the Viktoriapark the 505.1 map lines fall from 58 to 9, and the 45 compacted footways and 4 wide ways appear as
  501.6 and 501.9 candidates. In Lübeck the 505.1 map lines fall from 16 to 3. They are 501.6 candidates now, and still cut the tree groups
  (`PATH_WIDTHS_MM` lists 501.6 to 501.9).
- A 520 strip along a private way overlaps ALKIS private land where both know it. Both are candidates.
- A tagging mistake (`access=no` on a public path) now removes a path from the candidates and replaces it with a
  520 strip. The strip carries `freemapper:osm=highway=...; access=no` so the mapper sees why.
- Existing maps get all of this with `run --force`.
