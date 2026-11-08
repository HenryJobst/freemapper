# 0050. The Berlin K5 map is a WMS template at 0.25 m per pixel

Date: 2026-11-09
Status: Accepted

## Context

The Karte von Berlin 1:5000 (K5) shows walls, stairs, fences and small structures that a sprint mapper wants to
compare with. Geoportal Berlin offers it as a WMS (`k5_farbe`) and as regional TIFF ZIPs, under dl-de/zero-2-0.
The plan asked for 0.25 m on sprint and 0.1 m on schoolyard maps.

## Decision

- WMS only, never the TIFF ZIPs: `https://gdi.berlin.de/services/wms/k5_farbe`, layer `k5_farbe`,
  `CRS=EPSG:25833`, PNG, `max_px` 4000 per GetMap, tiled like the orthophoto WMS. Output `templates/k5.png` +
  `.pgw`; currency is the dataset date 2026-01-02 from the catalogue.
- Ruling (Task 4 review): 0.25 m per pixel on both map kinds, and coarser when the longest side would pass
  6000 px (`K5_MAX_SIDE_PX`). For a 1:5000 map, 0.1 m needs six times the pixels of 0.25 m and up to about
  300 MB of memory while stitching a large area.
- Only near Berlin and for UTM 33 maps. A failed download is a warning and never fails the fetch. The disk plan
  treats K5 as an optional part (`Part::K5`, 1.5 MB/km² in the estimate). Offline keeps the stored file.
- The starter map lists it hidden, opacity 0.7, behind the map; the OCAD file gets it as a background map;
  `list_layers` and the preview know it as `k5`. Attribution: `Geoportal Berlin / Karte 1:5000 von Berlin (K5),
  dl-de/zero-2-0` when the template was written.

## Consequences

- Berlin projects get one more template; an area longer than 1.5 km on a side, buffer included, gets a coarser
  K5.
- Cost if wrong: a slightly softer K5 at schoolyard scale.

Source: `docs/regions/berlin-plus-reference.md` (K5); catalogue `karte-von-berlin-1-5000-k5-farbausgabe-9f3ecf8d`;
branch `berlin-plus`, Task 4 and its fix round.
