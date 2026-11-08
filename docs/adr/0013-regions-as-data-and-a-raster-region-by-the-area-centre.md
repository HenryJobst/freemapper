# 0013. Regions as data, and a raster region by the area centre

Date: 2026-11-07
Status: Accepted

## Context

Berlin showed that one state's ALKIS service returns nothing on the other side of the border. Saxony added a
second raster provider. More states were to follow, and maps near a border touch two of them.

## Decision

Regions are data in `sources/region.rs`. Each region brings its official boundary
(`assets/regions/<region>.geojson`), its ALKIS source, its raster provider, its OSM extract and its attribution.

- ALKIS comes from every region the widened bbox touches, with a 100 m margin (ADR 0015).
- OSM comes from the extract of every touched region, merged by OSM id.
- Rasters and tile products come from one provider: the one of the region that holds the bbox centre, the
  raster region.

## Consequences

- A new state adds a boundary, a provider and an ALKIS source.
- Region boundaries must be valid geometries (ADR 0025).
- Cost if wrong: on a border map the templates stay empty on the far side of the border, for example south of
  the MV tile edge.

Source: spec §4.1 (Berlin, Saxony); plans `2026-11-07-berlin.md`, `2026-11-07-saxony.md` (Architecture, Review
Focus 2); commits 8597990, dfec985; MV rulings (Task 5); glossary "Rasterregion".
