# 0133. Classified roads in the permit notes

Date: 2026-11-14
Status: Accepted

Research: [`nachrecherche-2026-11-14.md`](../research/wald-und-sprint-behoerden/nachrecherche-2026-11-14.md).
Builds on [0117](0117-permit-notes-from-national-sources.md) and [0127](0127-permit-notes-follow-the-map-area.md).

## Context

A sprint that crosses or closes a road needs the road traffic authority's order, and the road's keeper
(Straßenbaulastträger) depends on its class: the Bund for a Bundesstraße (administered by the state), the state
for a Landes- or Staatsstraße, the Kreis for a Kreisstraße, the Gemeinde for its own roads, and in larger towns often
the town for the through roads. Open road networks with the legal class exist only in parts (Bavaria's ATKIS
`widmung`, the BKG's DLM250 at 1:250 000). OSM carries the class in the road's `ref` ("B 96", "L 30", "St 2045",
"S 177", "K 12", "A 9") everywhere, and freemapper reads the OSM extract for the notes already.

## Decision

- **Roads from OSM**: a way with a `highway` of a road (motorway to residential, with the links) and a `ref` of a
  class (A, B, L, S, St, K followed by a number) is a finding of the kind `road`, a line. Footways and tracks with a
  ref, European routes and a town's own numbering are not. `RoadClass::of` reads the class.
- **Section and hints**: the notes list the classified roads whose line touches the map area, by class, their refs
  sorted and each once. Each class gets a hint: who orders a closure or a crossing with marshals (the road traffic
  authority) and who usually keeps the road; an Autobahn may be crossed only on bridges or through underpasses.
  The wording says "usually" and names the larger towns as the common exception.
- `osm.geojson` stores a road as a LineString with its `road_ref`, so `--offline` gives the same notes.

## Consequences

- Every state gets the road hints without a new service. OSM can miss a `ref` or carry an old one; the notes are
  hints.
- Municipal roads are not named; the Ordnungsamt hint of the Gemeinde covers them.
- A legal road class from the states' networks (Bavaria's ATKIS) can replace or confirm OSM later.
