# 0016. Per-area raster providers and tile providers

Date: 2026-11-07
Status: Accepted

## Context

LGB and GeoSN publish rasters as fixed tiles with a published currency per tile. Saxony-Anhalt, Hamburg, Hesse,
Schleswig-Holstein and MV serve them through a WMS, a WCS or members of large zips, which freemapper cuts to the
map area.

## Decision

`Provider::per_area()` splits providers into two kinds. Tile providers (LGB, GeoSN) fetch recorded tiles into the
shared cache. Per-area providers (LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V) fetch a mosaic for the map area and
store it in the project as `data/dgm1.tif` and `data/dom1.tif`, each with a record of its bbox.

- On 2026-11-08 the MV review dropped a global per-area WCS mosaic cache: offline runs reuse the project's
  stored model when its record covers the area. A global cache would grow by 100 to 250 MB per area.
- MV is the first per-area provider in zone 33, so code must not tie `per_area` to zone 32.

## Consequences

- Moving a project to another provider drops its records (ADR 0014).
- Cost if wrong: offline runs need an earlier online fetch of the same area.

Source: spec §4.1 (Saxony-Anhalt, Hamburg); plan `2026-11-07-hamburg.md` (Architecture); commits 31a2920,
2223751, 857359c; MV rulings (zone audit, Task 3); `sources/region.rs` `Provider::per_area`.
