# 0148. Permit sources from ArcGIS REST services

Date: 2026-11-15
Status: Accepted

Builds on [0142](0142-water-protection-areas-of-the-bfg.md) (the BfG's REST service) and
[0146](0146-natural-monuments-and-protected-biotopes-of-the-states.md) (protected objects).

## Context

Lower Saxony publishes its Naturdenkmale and geschützte Landschaftsbestandteile only in the ArcGIS REST map service
of the NLWKN (`umweltkarten-niedersachsen.de/arcgis/rest/services/Natur_wms/MapServer`, layers 190 and 7 for the
Naturdenkmale as areas and points, 10 and 12 for the Landschaftsbestandteile), under dl-de/by-2-0 with the NLWKN as
source. There is no WFS. The BfG's source of ADR 0142 already read such a service, with its own paging.

## Decision

- `arcgis_permits` holds the REST reading: the query of a layer by a box in WGS 84, answered as GeoJSON in WGS 84,
  page by page while `exceededTransferLimit` says more follow. The BfG's source uses it.
- A REST table (`arcgis_permits::source`) reads layers by number with the readings of the WFS tables, asking only for
  the attributes its readings keep.
- Lower Saxony's Naturdenkmale and Landschaftsbestandteile come from it; the source names its gap, the protected
  biotopes, which the Landkreise keep. The host joins Lower Saxony's manifest.

## Consequences

- A state whose services are REST only needs a table, no code of its own.
- The examples Bad Harzburg and Herrenhausen have no Naturdenkmal in their small areas; the nearest lie just outside.
