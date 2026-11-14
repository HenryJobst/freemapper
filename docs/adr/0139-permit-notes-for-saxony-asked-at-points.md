# 0139. Permit notes for Saxony, asked at points

Date: 2026-11-15
Status: Accepted

Research: [`nachrecherche-2026-11-14.md`](../research/wald-und-sprint-behoerden/nachrecherche-2026-11-14.md).
Builds on [0124](0124-forest-administrations-in-the-permit-notes.md) (forest administrations) and
[0130](0130-permit-sources-as-tables.md) (table sources).

## Context

The Staatsbetrieb Sachsenforst publishes its Forstbezirke and Reviere ("Wald nach SächsWaldG") and the functions of
the forest ("Waldfunktionen in Sachsen") only as ArcGIS WMS, with no WFS and no REST query. GetFeatureInfo answers
`application/geojson` with the attributes of the features at a point and `"geometry": null`. The capabilities name
dl-de/by-2-0, for the functions with the credit "Geodaten Sachsenforst"; the catalogue entries say "Es gelten keine
Bedingungen".

What the services hold:

- Reviere (`REV_NAME`, `FBZ_NAME`) cover all of Saxony, towns too. The field of the Forstbezirk holds at most 40
  characters and cuts the one office that is no place, the Nationalpark- und Forstverwaltung Sächsische Schweiz.
- In the state forest the Revier of Sachsenforst is in charge. For other forest the lower forest authority is the
  Landkreis or the kreisfreie Stadt.
- The functions carry their key `WFZ`: 7110 Erholungswald, 4151 Naturwaldzelle (name in `GNM`), 1110
  Bodenschutzwald nach SächsWaldG, 211xx the zones of drinking water protection areas, in the forest only (name in
  `GNM`, zone at the end of `KNM`, "§ TW GW Z III").
- At a border between two Reviere the server answers for both.

## Decision

- A new kind of source, `gfi_permits`, asks a WMS by GetFeatureInfo. It lays cells over the box of the map, about
  square on the ground, no side shorter than 50 m and at most 25 cells, asks at the centre of each with a box of
  about 2 m in CRS:84, and stores each answer with its cell as geometry. The engine measures the cells as it
  measures areas. A query can take one feature per point only (`one`), for layers that do not overlap such as the
  Reviere.
- `PermitSourceInfo` gets the flag `sampled`. For a section that a sampled source fills, the notes do not say that
  nothing lies there but "At the points asked, … names nothing here". Below the part of the adapter one sentence
  says that the source answers only at points, that the shares are estimates and that what lies between the points
  can be missing. `permits.json` carries the flag as `"sampled": true`.
- Saxony's source (`sbs_permits`) reads the Reviere with their Forstbezirk as forest units of the Staatsbetrieb
  Sachsenforst and, from the functions, Erholungswald, Naturwaldzellen, Bodenschutzwald (as protection forest) and
  the drinking water zones. Its description says that a Revier is in charge in the state forest and the lower
  forest authority of the Landkreis for other forest.
- The host `www.forsten.sachsen.de` joins Saxony's manifest.

## Consequences

- A map in Saxony costs up to 50 requests to Sachsenforst (two queries at up to 25 points), about half a minute.
- A small Naturwaldzelle or an inner water zone between two points can be missed; the notes say so. The water zones
  outside the forest are not in the service.
- The layer numbers of the WMS (7, 46, 101, 80 to 94) may change when Sachsenforst publishes the service again; the
  findings read the key `WFZ`, so a changed layer gives no finding rather than a wrong one, and the live test
  notices.
