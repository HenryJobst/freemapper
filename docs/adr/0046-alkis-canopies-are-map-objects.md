# 0046. ALKIS canopies in Berlin are map objects, not candidates

Date: 2026-11-09
Status: Accepted

## Context

Spec §4.5 made every 522 canopy a candidate, because OSM and the building attributes say little about roofed open
structures. The Geoportal Berlin layer `alkis:bauwerkeflaechen` has the ALKIS Bauwerke `Überdachung` and `Carport`
as surveyed polygons (`bezbwf`), from the same cadastre as the 521 buildings. A canopy often lies against a
building or partly under it, and some ALKIS canopies repeat a building outline.

## Decision

- User decision of 2026-11-08: `Überdachung` and `Carport` become 522 map objects on both map kinds
  (`build/canopy.rs`). This amends spec §4.5 ("candidate 522") for Berlin and adds them to the map objects of
  ADR 0003.
- They go into the map part right after the 521 buildings. Paint order comes from the colour priorities: the 522
  fill lies under the 521 fill (test `canopy_fill_lies_under_the_building_fill`).
- A canopy with at least 90 % of its area inside 521 footprints (`INSIDE_BUILDING_SHARE`) is dropped, so it is
  not drawn twice. Otherwise the nearby footprints are cut out, so no black 522 outline crosses a building. A
  canopy that only touches a building stays whole.
- Each remaining part must reach `(CANOPY_MIN_SIDE_MM = 0.5 mm × scale)²`; outlines are simplified at 0.1 mm.
  Both values come from `buildings.rs`.
- Tags: `freemapper:source=geoportal-berlin/bauwerke-flaechen`, `freemapper:source_id=<uuid>`,
  `freemapper:currency=<fetch date>`, `freemapper:alkis=Überdachung|Carport`.
- Every other Berlin structure stays a candidate, because ALKIS records neither height nor passability for it.
  Building passages are 522 candidates (ADR 0047).

## Consequences

- Berlin sprint and schoolyard maps start with the canopies drawn. An existing map gets them only with
  `run --force`.
- The mapper still checks the outlines in the terrain; the cut at a building can leave a small sliver part.
- Cost if wrong: a demolished or mis-surveyed canopy is a map object the mapper has to delete, not a candidate
  to ignore.

Source: user decision 2026-11-08; `docs/regions/berlin-plus-reference.md` §3; plan
`docs/superpowers/plans/2026-11-09-berlin-plus.md`; branch `berlin-plus`, review fix round of Task 3.
