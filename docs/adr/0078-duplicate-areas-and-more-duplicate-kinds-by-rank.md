# 0078. Duplicate areas and more duplicate kinds, by rank

Date: 2026-11-10
Status: Accepted

## Context

`build/dedup.rs` ranks the sources (register, then ALKIS, then survey, then OSM, ADR 0063) and drops the lower one for
trees, a few lines and some street furniture. Passages, memorials, private strips and bridges had no rule, and they
come from several sources since ADRs 0061, 0072, 0073 and 0075.

A count of probable pairs in the finished candidates (objects of different sources with the same symbol: points within
4 m, lines with 80 % of their length within 1.5 m, areas with half of the smaller one covered by the other) found no
point or line duplicates left, and these:

| Symbol | Viktoriapark | Lübeck |
| --- | --- | --- |
| 522 passage: OSM `tunnel=building_passage` or `covered` over a Berlin building part, an ALKIS passage or an ALKIS canopy of the map | 19 | 62 |
| 520: OSM strip along a private road or car park inside an ALKIS 520 land use area | 23 | 14 |
| 501.5: a Berlin bridge deck beside a surveyed pavement | 1 (not a duplicate) | 0 |

Trees were checked too. The remaining OSM trees show no peak of nearest register trees just beyond the 2 m limit (none at
0 to 3 m, then 1 to 7 per metre with no peak), so the 2 m of ADR 0051 stays. Memorials, fountains, benches, lamps, walls,
fences and hedges had no pair within 8 m (points) or 3 m (lines) after the existing rules.

## Decision

**Areas** (`drop_areas_by_rank`, run in `project.rs` on all candidates once they are joined, before the unreachable
passages are hidden). Of the 522, 520 and 501.5 candidates, one is dropped when enough of its area lies under the
areas with the same symbol of a higher rank. The higher-ranked areas are the other candidates and the map objects
that carry the symbol (the canopies of ALKIS and the Berlin structures, the ALKIS land use 520).

- The share is measured against the union of the winners, so a passage that a cadastre splits in two still counts.
- The winner is widened by 0.25 mm on paper (`AREA_DEDUP_TOLERANCE_MM`; 1 m at 1:4000, 1.25 m at 1:5000), about one
  thin line width: two outlines of the same object that differ by that much still read as one line. The tolerance is
  a paper size, because it decides what the printed map shows twice.
- 522: 50 %. An OSM way is a strip of a default width along the way, the cadastre polygon is the real shape, so the
  strip can be wider or longer than the polygon.
- 520: 80 %. An OSM strip along a private road partly outside the land use area (a measured pair at 62 % and one at
  73 %) is news for the part outside, so it stays.
- 501.5: only an OSM bridge (`man_made=bridge`) can lose, and only to a Berlin bridge deck (`bauwerke-flaechen`). The other
  501.5 candidates are paved ground that lies under bridges and paths by design.
- Two areas of the same rank never drop each other. The winner names the dropped one in `freemapper:also`, also when
  the winner is a map object (a canopy).
- The land use candidates (`freemapper:source=alkis-nutzung`) rank as ALKIS. They had no rank before.

**Points.** A Berlin monument (`bauwerke-flaechen`, 526) or a NAS 526 beats an OSM memorial within 5 m
(`MEMORIAL_DEDUP_M`; the cadastre point is the centre of an area, the OSM node can stand anywhere on it). ADR 0077 added
the surveyed bollard against the OSM bollard (2 m). No other pair was found.

**Lines.** Kerb (501.1) and embankment top (104) join the symbols whose OSM lines are dropped along a cadastre line.

## Consequences

- Viktoriapark: 520 from 117 to 94, 522 from 211 to 192. In total 42 candidates less (2 354 to 2 312). Two 522 matches are
  map canopies. Lübeck: 520 from 1 125 to 1 112, 522 from 280 to 221, 72 fewer (the 531 and 313 changes
  come from ADR 0077). The finder drops from 43 to 2 pairs (Viktoriapark) and from 76 to 3 (Lübeck).
- Kept on purpose: an OSM strip much larger than the cadastre polygon it covers (3 pairs in Lübeck, a 40 m² strip over a
  3 to 8 m² polygon). It adds extent that the cadastre lacks.
- A distinct object under the 50 % or 80 % limit is never dropped. Crops of the dropped pairs (the red outlines lie
  inside blue cadastre areas) are in the pull request.
- Existing maps get this with `run --force`.

## Amendment 2026-11-10: only a surviving area wins

The first version let every higher-ranked area win, dropped ones included. In a chain A > B > C, B could go because of
A, and C then went because of B although only a small part of C lay under A. The `freemapper:also` note landed on the
dropped B, and the part of C outside A was missing from the map.

- `drop_areas_by_rank` now works from the highest rank down, and an area that is already dropped covers nothing. C
  stays when A alone does not reach the share; when A does, C goes and the note names C on A, the winner that is
  still on the map.
- Points already behaved this way.
- Existing maps get this with `run --force`.
