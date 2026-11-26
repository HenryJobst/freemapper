# 0197. Czechia: one national adapter with the rasters and the cadastre of ČÚZK

Date: 2026-11-26
Status: Accepted

## Context

freemapper is to make maps anywhere in Czechia, for all map kinds. The data are national and open: the Czech Office for
Surveying, Mapping and Cadastre (ČÚZK) gives its open data under CC BY 4.0 and, since 1 July 2023, also ZABAGED, the
orthophotos and the RÚIAN data. The research is `docs/regions/czech-reference.md`; the issue is #356 (part of #227).
Poland (ADR 0164) is the model: one national adapter. Czechia gets the rasters and the cadastre together, because a sprint map without buildings is of no use; the permit notes follow.

## Decision

- One adapter `cesko` in the crate `freemapper-adapters-cz`, one provider `cuzk`. The adapter is code, not a manifest:
  the height models come from ArcGIS image services, which no manifest table describes (ADR 0164 gave the same reason
  for Poland).
- The boundary is the state border of OpenStreetMap (relation 51684, ODbL, simplified to about 40 m). Geofabrik has one
  extract for the whole country (`europe/czech-republic/czech-republic`, about 910 MB); its clipping polygon only
  chooses the extract.
- The rasters come from `ags.cuzk.gov.cz`, one host: the terrain model DMR 5G and the surface model DMP 1G from the image
  services `dmr5g` and `dmp1g` (`exportImage`, Float32 GeoTIFF), the orthophoto from the WMS of the service `ORTOFOTO`
  (JPEG). All three are asked for in EPSG:25833, the grid of the map, so no Krovak (S-JTSK) projection is needed. The
  models are read at 2 m, their pixel, in pieces of at most 4000 pixels and laid on the 1 m grid of the area (bilinear;
  the nearest cell with a height where one neighbour has none). A request for an area without a height gives huge
  negative values; they become `NaN`, and an area without any height is refused with the words "covers Czechia only".
- Coverage is `Coverage::Full`. The buildings are the footprints of RÚIAN (layer `StavebniObjekt` of the map service
  `RUIAN`, about 330 buildings in 0.2 km² of Prague's Old Town, where ZABAGED has 71 blocks); a building that no longer
  stands (`platido`) or is marked wrong (`nespravny`) is left out, and so are the structures of water works (codes 22 to
  29 of the way of use). The land use is the polygon layers of ZABAGED (forest and scrub, grassland, arable land,
  gardens, vineyards, hops, maintained green, cemeteries, marsh, peat, quarries, industrial sites, water). Both are
  asked as GeoJSON in the CRS of the map, 1000 objects a page. The way of use and the kinds become the German words of
  the ALKIS schema that the engine reads. A pond and a river are told apart by `stojvoda`. The permit notes are not
  available for Czechia yet; the engine says so as it does for Poland before its permit stage. The progress lines name
  the cadastre RÚIAN.
- One zone serves the country: every map is drawn in EPSG:25833, also in the east of Moravia, which lies up to a degree
  past the zone border (scale error under 0.1 %).
- `RegionInfo` gets `Country::Cz` (code `cz`) and `Names` a Czech name (`cs`), as Poland has a Polish one. Czech is
  already a program language (machine drafted, ADR 0092); the region is named in it as `Česko`.
- The credit line is `© ČÚZK (CC BY 4.0)`, shown on the PDF and in `ATTRIBUTION.txt` when a project holds a terrain model
  or an orthophoto of ČÚZK. The licence asks for the source, a link to the licence and a note when the data were
  changed; the credit and the attribution file carry all three.
- The adapter asks the host `ags.cuzk.gov.cz` and no other; a person can allow more hosts as for the other adapters.
- The area page shows the orthophoto WMS of the same host.

## Consequences

- Three downloads, all stored in the project folder: the terrain model and the surface model at about 1 MB per km² each,
  the orthophoto at about 6.6 MB per km². The estimate says so.
- RÚIAN parcels are out of date since December 2021 (no transfer from the cadastre), so freemapper takes no parcel
  from it; the land use comes from ZABAGED.
- The terrain model DMR 5G is bare earth, interpolated under buildings. In a dense old town the contours there wander in
  small loops; a flat sprint map there looks better with a coarser interval than the 2.5 m of the map kind.
- In 2025 a few people on the OSM mailing list `talk-cz` doubted that the orthophoto WMS falls under CC BY 4.0. The terms
  of ČÚZK name the orthophotos of the Czech Republic among the open data, so freemapper uses the WMS; the doubt is
  recorded in the reference and settled when ČÚZK says otherwise.
- The flight year of the orthophoto is not known (the service names none); a stored image is renewed after 30 days.
