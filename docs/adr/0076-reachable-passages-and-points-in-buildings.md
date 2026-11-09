# 0076. Passages must be reachable, and points inside buildings go

Date: 2026-11-10
Status: Accepted

## Context

User requirement (2026-11-10): grey passage areas matter only when a runner can reach them from walkable ground. After
ADRs 0061 and 0072 the Lübeck example has 280 522 candidates, and the Viktoriapark 211. Many lie inside a block, touching
only buildings: a Berlin building part between two wings, an OSM way through a rear building, a passage from one walled-in
courtyard to another. They show as grey outlines on the grey building and tell the runner nothing.

A second problem turned up in the Lübeck preview. Dark dots stood on the large church. They were 526 memorial candidates
(ADR 0075), OSM nodes whose position lies inside the building footprint. A memorial, lamp or bin inside a building can
be neither seen nor used on a map.

ISSprOM 2019-2 2.3 asks for passages "which are important for the competitors". A passage that leads nowhere is not
important to anyone.

## Decision

**Reachable passages.** `build/passages.rs` runs once per build on all candidates, after the cadastre, survey and OSM
candidates are joined:

- Walkable ground is the clip window (map area plus buffer) less the 521 buildings, the 520 areas (the land-use 520
  candidates and any 520 among the candidates, such as the strips of ADR 0073) and the water. The sprint base fill
  covers all of it, so streets, paved areas and open land need no separate test. Walls and fences do not block; a
  fence line rarely closes off a whole region.
- The ground falls into pieces. A piece that touches the edge of the window is part of the street network. A piece inside
  a block, a courtyard with no way out, is not.
- Each 522 passage is a node. A passage is joined to every other passage and every ground piece within 0.5 mm on paper
  (2 m at 1:4000, 2.5 m at 1:5000), which bridges a way that ends a little inside its wall. A chain of passages (the
  Lübecker Höfe und Gänge) is one component.
- A component that holds a piece of the street network is reachable. A passage in any other component gets the tag
  `freemapper:hidden=unreachable`.
- Hidden is not deleted. The passage stays in the candidates (template map, OCAD file) and is left out of the training
  map, as the points under a 520 area are (ADR 0066). A mapper who knows the courtyard has a gate can still find it.
  Both OSM passages (ADR 0072) and the cadastre ones (ADR 0061, Berlin building parts) go through the same test.

**Points inside buildings.** In `generalize_points` (`build/points.rs`), on sprint maps, a candidate point whose centre
lies inside a 521 footprint is removed. This applies to every point symbol that function sizes (the OSM and survey
street furniture, memorials, fountains, wells and springs), not only to memorials. It is removed, not hidden, because
no tag would help the mapper. Two things stay:

- Trees (417, 418). The cadastre footprint often takes in the trees along a wall, and tree groups are cut at buildings
  already. The existing behaviour stays.
- Map points, and everything on schoolyard maps.

Amends ADR 0074 (control sites inside a building are dropped) and ADR 0075 (so are memorials, the dots on the church).

## Consequences

- Lübeck: 16 of the 280 522 candidates are hidden (13 from ALKIS, 3 from OSM). 21 point candidates inside buildings go:
  526 from 10 to 4, 531 from 195 to 180. The candidates fall from 1859 to 1838.
- Viktoriapark: 52 of the 211 522 candidates are hidden (38 Berlin building parts, 14 OSM passages). 531 goes from 121 to 119
  (candidates 2327 to 2325).
- Not done: walls and fences as obstacles, level changes, and passages that a gate closes.
- Existing maps get this with `run --force`.
