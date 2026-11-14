# 0127. Permit notes follow the map area

Date: 2026-11-14
Status: Accepted

Builds on [0117](0117-permit-notes-from-national-sources.md) and
[0124](0124-forest-administrations-in-the-permit-notes.md). Amends 0124's rule that the forest hints are for
forest maps only, and 0117's rule that the school operator hint is for schoolyard maps only.

## Context

The user asked on 2026-11-14 that the notes give only what is relevant to the chosen map area, and when that is
unclear rather too much than too little. Two rules decided by the map kind instead:

- 0124 gave the forest hints to forest maps only, because a Revier in Brandenburg covers towns as well. A sprint
  map in a park with a wood, or on the edge of a forest, then missed the forest office, while the section with
  the Revier still appeared on a schoolyard in the middle of a town.
- 0117 named the operator of a school only on schoolyard maps; a sprint on a campus or around a school missed it.

## Decision

- **Forest by what lies in the area.** The forest hints and the section of the forest offices come when the map
  is a forest map, when OSM has forest or wood (`landuse=forest`, `natural=wood`) in the area with any share,
  or when OSM was not read and it cannot be told. Otherwise both are left out. A wood without operator or owner
  is a new kind of OSM finding (`forest`) for this; with one it stays an operated object, as before.
- **The school operator on every map kind**, when a school, kindergarten, college or university with an operator
  or owner lies on the map.
- The state forest, the protected areas and the regional sections already follow the area and stay as they are.

## Consequences

- A town without forest gets no Revier and no forest hint; a sprint map with a wood gets both.
- `osm.geojson` in `data/permits/` now also holds the woods of the map box, so it grows on forest maps; offline
  runs read it as before, and older stored files (without woods) count as "no forest" unless the map is a forest
  map.
- New hints of later steps follow the same rule: decided by what lies in the area, and given when unsure.
