# 0014. Two UTM zones, the map CRS from the raster region

Date: 2026-11-07
Status: Accepted

## Context

Brandenburg, Berlin and Saxony publish in EPSG:25833. All Saxony-Anhalt data is EPSG:25832, and so are Hamburg,
Hesse, Schleswig-Holstein, NRW and Lower Saxony. In 25833, NRW would reach eastings of -139.6 km and a grid
convergence of -6.3°.

## Decision

freemapper works in both zones (`crs::Crs`). Each project takes the CRS of its raster region
(`ProjectFile.crs`, default 25833 so older projects load unchanged).

- Rasters stay on their native grid.
- ALKIS from a service in the other zone is reprojected through lat/lon.
- Region checks stay in 25833.
- Georeferencing and the OCAD grid code (8007 or 8008) follow the zone.
- A change of map area that changes the zone, or the raster provider, drops the stored records with a warning.

## Consequences

- A map near the zone border stays in the zone of its centre.
- Code must not assume a zone from the provider kind. The MV plan had to audit "per area means zone 32" first.
- Cost if wrong: zone mix-ups in derive or in the download estimate.

Source: spec §4.1 (Saxony-Anhalt, Hamburg); plan `2026-11-07-saxony-anhalt.md` (Global Constraints); commits
5f67f9c, e81f0e1, 2223751; MV rulings; NRW reference §13 item 6; glossary "Koordinatensystem (Zone)".
