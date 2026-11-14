# 0131. Permit sources from shapefiles of a whole state

Date: 2026-11-14
Status: Accepted

Builds on [0124](0124-forest-administrations-in-the-permit-notes.md) (NRW's forest shapefiles) and
[0130](0130-permit-sources-as-tables.md) (table sources).

## Context

Brandenburg and North Rhine-Westphalia publish their water protection areas not as a WFS but as one shapefile in a
zip for the whole state: the LfU Brandenburg on `data.geobasis-bb.de` (about 4 MB, 2027 zones, EPSG:25833, UTF-8,
CC BY 4.0 per GovData) and the open data portal of NRW (about 15 MB, 2522 zones, EPSG:25832, dl-de/zero-2-0 per
GovData). NRW's table has no `.cpg` and is Latin-1; it also holds planned zones (`areastatus = P`). The forest
sources of NRW (ADR 0124) already download, cut and convert such zips, with the code in their own module.

## Decision

- **Shapefile tables** (`shape_permits`): the download, the cut to the map box and the conversion to WGS 84 move out
  of the NRW forest module into one place, with the CRS of the file as a parameter. A `ShapeTableSource` reads one
  zip by a `Reading` of the table sources (ADR 0130) and keeps only its attributes.
- A water `Reading` can name the attribute and value of a planned zone; such a zone's text ends in "(geplant)".
  Planned zones stay in the notes: when in doubt, rather too much than too little (ADR 0127).
- The shapefile reader reads a field that is not UTF-8 as Latin-1.

## Consequences

- An online fetch in NRW downloads about 32 MB (three files), in Brandenburg about 4 MB more. A cache of these files
  across projects is still left for later.
- Further statewide shapefiles of polygons are a table of a few lines. Point layers (Bremen's monuments) need the
  reader to read points first.
