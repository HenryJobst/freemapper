# 0008. OSM from Geofabrik extracts, not Overpass

Date: 2026-11-06
Status: Accepted

## Context

freemapper needs OSM features for candidates and sprint paths. Public Overpass instances are shared services
with usage limits. Geofabrik publishes a PBF extract per German state.

## Decision

OSM data comes from the Geofabrik extract of each touched region, cached and read with `osmpbf`, filtered to the
area. Public Overpass instances are not a backend. One exception: resolving a single named place or OSM element
through Nominatim or the OSM API.

## Consequences

- Runs work offline from the cached extract, and repeated runs do not load a shared server.
- The first run in a state downloads the whole extract, which is 917 MB for North Rhine-Westphalia (ADR 0028).
- Cost if wrong: large first downloads; switching to another backend would touch every OSM rule test.

Source: spec §4.1 (OSM); plan-mode notes "OSM-Quelle"; commit e4c716d.
