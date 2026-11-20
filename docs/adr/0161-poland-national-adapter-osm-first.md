# 0161. Poland: one national adapter, OSM first

Date: 2026-11-20
Status: Accepted

## Context

freemapper is to make maps anywhere in Poland, for Polish users. The Polish sources (GUGiK) are national and open, so
one adapter serves the whole country; only the OSM extracts differ by voivodeship. The research is
`docs/regions/crossborder-reference.md` (§2, the border strip) and the design
`docs/superpowers/specs/2026-11-20-poland-design.md`. The plan builds Poland in five stages that each ship:
foundation and OSM, terrain and orthophoto, buildings and land use, Polish, permit notes.

## Decision

- One adapter `polska` in the crate `freemapper-adapters-pl`, with the 16 Geofabrik extracts of the voivodeships
  (`europe/poland/<id>`, which Geofabrik redirects to the folder of the voivodeship, also for the update state). Its
  boundary and the boundaries of the extracts are the clip polygons of Geofabrik, embedded in WGS 84 and projected to
  EPSG:25833 on first use.
- `RegionInfo` has a `country` and a `coverage`. `Coverage::OsmOnly` means the region supplies OSM extracts and
  nothing else: `fetch` and `derive` skip the rasters, the offline fetch stores an empty cadastre, the estimate has no
  ALKIS item, and the map has no templates and no contours. A warning says so (`warning-osm-only-no-terrain`).
- The project grid stays UTM. Zone 34 is new in freemapper-core (ADR 0040 there); `RegionAdapter::crs_for(centre)`
  gives a map the zone of its centre. EPSG:2180 is a source grid for the Polish rasters, not a grid of a project.
- An area whose centre lies in a region with OSM only and that reaches into a region of another country is refused
  (`area-crosses-country`), because it would be made half from each. A map centred in Germany that reaches into Poland
  keeps working and warns that the part beyond has no terrain model.
- The area page shows the aerial photo of the GUGiK WMS (`ORTO`, layer `Raster`). The service allows viewing; freemapper
  never harvests it.
- The PDF sheet text uses Helvetica with WinAnsi: the Polish letters it lacks print as the letter they are written with
  (`ł` as `l`). Stage 4 embeds a font.
- The lists of supported regions in messages name Poland; the sentence "official geodata of …" lists the regions with
  rasters only.
- Poland is a preview in the READMEs, the manual and the release notes until Polish is a program language (stage 4).

## Consequences

- Maps in Poland have the OSM objects only until stage 2 and 3 bring the terrain, the orthophoto and the official
  vectors.
- German border maps next to Poland now list `polska` in `regions_for`; the golden g2 is blessed for it.
- The ids of the adapters count one more in the registry tests, the `adapters list` output and the CSP of the area page.
