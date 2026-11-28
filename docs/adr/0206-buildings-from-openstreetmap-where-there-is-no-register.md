# 0206. Buildings from OpenStreetMap where there is no register

Date: 2026-11-28
Status: Accepted, narrows ADR 0204

## Context

The map draws buildings from the cadastre only: the build, the vegetation mask and the export read
`data/alkis-buildings.geojson`, the buildings of the region in the schema of the LGB. The first stage of the Swedish
adapter (ADR 0204) has no register of buildings, since Byggnad is applied for and not yet granted, so its maps had no
buildings at all. A sprint or school map without buildings is of little use, and a forest map shows them as
landmarks. OpenStreetMap has the buildings of Sweden, and the extract is read for the paths anyway.

## Decision

- `RegionAdapter::buildings_from_osm()` (default `false`) says that the buildings of a region come from
  OpenStreetMap. The adapter of Sweden answers `true`; the German states, Poland and Czechia do not change.
- The fetch of such a region asks no cadastre service for buildings. The read of the OSM extract keeps the
  `building` ways and relations too (any value but `no`), and `store_osm_buildings` writes them to
  `data/alkis-buildings.geojson` in the schema the build reads: `gebnutzbez = "Gebäude"`, `funktion` the value of
  `building`, `rellage` set for `location=underground` or a negative `layer`, the footprint in the map's CRS. The
  record of the buildings has the date, the box and the count, and names no region.
- A building that no candidate rule asks for is not stored as a candidate. A feature that is a building and a
  candidate stays in both.
- Everything after the fetch is unchanged: the same generalisation, minimum sizes, mask and export as for register
  buildings. `freemapper-adapter-api` has the new note `progress-osm-buildings` in 21 languages.

## Consequences

- Swedish maps have buildings as far as OpenStreetMap maps them. Its coverage in the country is good in towns and
  thinner in the countryside, and the credit line of OpenStreetMap names the source.
- OSM buildings are only footprints: no date of the survey, no use (`funktion` is the OSM tag, not the cadastre's).
- Another region without a register (a future country) sets the same switch. A manifest key for it can follow when a
  folder adapter needs it; until then the switch is code of the adapter, as `no_buildings_warning` is.
- Byggnad, when granted, replaces this in Sweden: the switch goes and the register is read like ALKIS.
