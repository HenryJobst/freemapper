# 0070. OSM multipolygon relations are read

Date: 2026-11-10
Status: Accepted

## Context

The OSM reader took tagged nodes and tagged ways. A `type=multipolygon` relation carries the tags of an area
(`leisure=park`, `natural=water`, `building=yes`) and lists untagged ways as its rings. Those areas never reached
the rules: lakes with islands, parks cut by a road, buildings with a courtyard, large woods. ADR 0036 named the gap
("freemapper reads no OSM relations, so OSM multipolygon rivers are lost").

## Decision

- `read_extract` reads relations with `type=multipolygon` whose own tags pass the same `wanted` filter as ways.
  Other types, such as `boundary` and `route`, stay unread. The relations are collected in the pass that reads the
  nodes, so the extract is not read an extra time. The member ways come through the indexed reader with their
  nodes, tagged or not.
- A relation becomes one feature, `relation/<id>`, with the new geometry `OsmGeometry::Area`: a list of polygons,
  each with an outer ring and its holes. The ways of one role are chained into closed rings by their shared end
  nodes, in either direction. An empty role counts as outer. An inner ring goes to the smallest outer ring that
  contains it. An unclosed chain, a hole outside every outer ring and a way with a missing node are dropped; a debug
  build prints a note.
- The area test of ways holds for relations. A relation counts when a ring has a vertex in the area box or crosses
  it, or when an outer ring contains the box. The rings keep all their nodes.
- An outer way whose tags the relation repeats (old-style tagging) is not read as a feature, so the area is drawn
  once. An outer way with a tag of its own, such as a footway along the edge, stays. A relation with no tags besides
  `type=multipolygon` is no feature. The tagged outer way is read as before, and its holes stay unknown.
- Only area rules draw an `Area`. Each polygon becomes one map object: the outer ring first, then each hole, every
  ring closed with the hole and close flags that OOM expects. Later steps read the object through `object_polygon`,
  so a courtyard stays out of its building and a pond out of its park.
- The stored candidates write an `Area` as a GeoJSON `MultiPolygon`, with the holes as further rings, and read it
  back unchanged.

## Consequences

- Parks, water, woods and buildings that exist only as relations now appear as candidates or map objects. The
  changelog gives the counts for the two examples that change.
- Maps made before this change get the relations only with `run --force`, because the stored OSM candidates are not
  read again.
- A relation pulls in all its member ways from the state extract, tagged or not, so the way pass uses more memory.
- Rings that meet at a node shared by three or more ways can be chained the wrong way round. The result is still a
  closed ring, and such relations are rare.
- A relation with a gap in its outer ring is dropped without a message, as it was before.

This ADR replaces the sentence "freemapper reads no OSM relations" in ADR 0036. Rivers as relations
(`type=waterway`), the coastline and marsh stay out of scope.
