# 0070. OSM multipolygon relations are read

Date: 2026-11-10
Status: Accepted

## Context

The OSM reader took tagged nodes and tagged ways. A `type=multipolygon` relation carries the tags of an area
(`leisure=park`, `natural=water`, `building=yes`) and lists untagged ways as its rings. Those areas never reached
the rules: lakes with islands, parks cut by a road, buildings with a courtyard, large woods. ADR 0036 had named the
gap ("freemapper reads no OSM relations, so OSM multipolygon rivers are lost").

## Decision

- `read_extract` reads relations with `type=multipolygon` whose own tags pass the same `wanted` filter as ways. Other
  types, such as `boundary` and `route`, stay unread. The relations are collected in the pass that reads the nodes, so
  the extract is not read once more. The member ways come through the indexed reader with their nodes, tagged or not.
- A relation becomes one feature, `relation/<id>`, with the new geometry `OsmGeometry::Area`: a list of polygons, each
  with an outer ring and its holes. The ways of one role are chained into closed rings by their shared end nodes, in
  either direction. A role that is empty counts as outer. An inner ring goes to the smallest outer ring that contains
  it. An unclosed chain, a hole outside every outer ring and a way with a missing node are dropped, and a debug build
  says so.
- The area rule of ways holds for relations: a relation counts when a ring has a vertex in the area box or crosses it,
  or when an outer ring contains the box. The rings keep all their nodes.
- An outer way that carries only tags the relation repeats (old-style tagging) is not read as a feature, so the area is
  not drawn twice. An outer way with a tag of its own, such as a footway along the edge, stays. A relation without tags
  of its own, only `type=multipolygon`, is no feature, and the tagged outer way is read as before; its holes stay
  unknown there.
- The rules draw an `Area` as an area (`DrawKind::Area`) and as nothing else. One map object is made per polygon: the
  outer ring first, then each hole, every ring closed with the hole and close flags, as OOM expects. The later steps
  read the object through `object_polygon`, so a courtyard stays out of a building and a pond stays out of a park.
- The GeoJSON of the stored candidates writes an `Area` as a `MultiPolygon` with the holes as further rings, and reads it
  back unchanged.

## Consequences

- Parks, water, woods and buildings that are only relations now appear as candidates or map objects. The count of
  OSM features per map grows; the numbers for the two largest examples are in the changelog.
- Maps made before this change get the relations only with `run --force`, because the stored OSM candidates are not
  read again.
- A relation pulls in all its member ways of the state extract, tagged or not. The memory use of the way pass grows by
  the ways of the matching relations.
- Rings that touch at a shared node with three or more ways can be chained wrongly; such a relation is rare and the
  result is a ring that is still closed.
- Cost if wrong: a relation that is mapped badly (a gap in the outer ring) is silently missing, as before.

ADR 0036 said freemapper reads no OSM relations; this ADR replaces that sentence. Rivers as relations
(`waterway=river`, type `waterway`), the coastline and marsh stay out of scope.
