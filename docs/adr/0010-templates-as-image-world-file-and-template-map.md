# 0010. Raster templates as image plus world file, vector templates as a template map

Date: 2026-11-07
Status: Accepted

## Context

OOM reads raster templates as `TemplateImage` and vector templates as `OgrTemplate` or as another map. A GeoTIFF
writer and a GeoPackage writer would both need extra code, and GeoPackage needs SQLite.

## Decision

- Raster templates (orthophoto, hillshade, slope, nDSM) are JPEG or PNG files with a world file. OOM takes the
  CRS from the template's `<crs_spec>`. OCAD reads world files too.
- Vector templates are a template map, `templates/candidates.omap`, symbolized with the project's symbol set.
  The fallback, if OOM does not georeference it reliably, is Shapefile with `.prj`.

## Consequences

- No GeoTIFF writer and no SQLite are needed.
- Candidates render with the right symbols, and the mapper can copy them into the main map.
- Cost if wrong: a template that OOM places wrongly, which would force the Shapefile fallback.

Source: spec §4.2, §4.4; plan `2026-11-07-m2-m3a-orthophoto-hillshade.md` (Architecture); commit ff7fce4.
