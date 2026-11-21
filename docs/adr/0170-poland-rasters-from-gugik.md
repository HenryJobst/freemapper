# ADR 0170: Poland gets its rasters from GUGiK, without a cadastre

## Status

Accepted, 21 November 2026. Builds on [ADR 0164](0164-poland-national-adapter-osm-first.md) and
[ADR 0167](0167-poland-height-and-orthophoto-readers.md).

## Context

The readers, the warp, the mosaic and `PolskaRasters` stood on main, but nothing asked the index
service of GUGiK and the engine treated Poland as OSM only. The service answers again
(`docs/regions/poland-reference.md`). A map of Słubice now gets terrain, surface model and
orthophoto from it.

## Decision

- **A sheet index client.** `GugikIndex` asks the WFS index of each product once, with every year
  layer of the product in one request, and the box north first, east second (EPSG:2180). The layer
  list comes from `DescribeFeatureType`, since a missing year layer fails the whole request.
- **Only what can be read.** Height sheets only as `.asc`; orthophotos only as `.tif` and only
  RGB. CIR sheets are false-colour infrared and never serve as the orthophoto. The 0.25 m rule of
  ADR 0167 stays.
- **A download only from `opendata.geoportal.gov.pl`,** over https. Another host leaves the sheet
  out with a warning.
- **A third coverage, `Rasters`.** The engine fetched no raster for `OsmOnly` and also drew no
  cadastre. Poland has rasters and no cadastre, so `Coverage::Rasters` fetches and derives the
  templates and keeps the buildings and land use from OSM. It still refuses an area that reaches
  over the border.
- **A sheet that cannot be read is left out.** Sheets of old flights with the old sheet numbers
  can carry a coordinate system the reader does not know (EPSG:32767); the fetch warns and uses
  the sheets of other flights, and fails only when nothing is left.
- **The OSM extracts lie in the folder of the voivodeship,**
  `europe/poland/<id>/<id>-latest.osm.pbf`. Geofabrik's redirect from the old address points to
  an address that does not exist.

## Consequences

- Poland stays a preview until the cadastre of Stage 3.
- The sheet index and the downloads depend on the service of GUGiK, which was down for days before.
  The estimate names the download sizes and a cap of 2 GB per project.
- A map of old and new flights may mix years; the choice per sheet number is the rule of
  `best_per_cell`, and the credit line names GUGiK.
