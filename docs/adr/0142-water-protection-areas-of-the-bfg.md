# 0142. Water protection areas of the BfG

Date: 2026-11-15
Status: Accepted

Builds on [0129](0129-permit-notes-for-hamburg-water-protection-and-forest-functions.md) (water protection in the notes), [0130](0130-permit-sources-as-tables.md)
and [0139](0139-permit-notes-for-saxony-asked-at-points.md) (sources asked at points).

## Context

Seven of the supported states give no water protection areas through a service of their own that freemapper may
read: Bavaria, Lower Saxony, Bremen, Mecklenburg-Vorpommern, Rhineland-Palatinate, Saxony-Anhalt, and Saxony
outside the forest. The Bundesanstalt für Gewässerkunde (BfG) gathers the areas the states report to WasserBLIcK and
serves them in the ArcGIS REST service `geoportal.bafg.de/arcgis1/rest/services/INSPIRE/AM/MapServer`: layer 1
drinking water protection areas, layer 2 medicinal spring protection areas, with `Name` but no zone, often one
feature per zone. The query answers GeoJSON in WGS 84 with paging (`exceededTransferLimit`). The catalogue entry
(`52d1b6ab-31c7-491b-96c8-be22a31d7313`) names CC BY 4.0 with the credit "WasserBLIcK/BfG und Zuständige Behörden
der Länder, Datum" and warns that the coverage is not complete. Spot checks: data in all seven states, none in
Baden-Württemberg. Names from Lower Saxony come as UTF-8 read as Windows-1252 ("BÃ¶rÃŸum").

## Decision

- A source `bfg` (`bfg_permits`) reads both layers by box, page by page, joins the features of one name into one
  area (so nested zones count once) and repairs names read as Windows-1252. A medicinal spring area is named
  "<Name> (Heilquellenschutzgebiet)". The findings are water protection areas without a zone; the hint then says
  that the data name no zone.
- The adapters of the seven states bring it; their manifests name the host `geoportal.bafg.de`. States with own
  water data keep theirs and do not ask the BfG.
- The credit carries the year of the fetch; the catalogue's sentence on the coverage is the source's note.
- An empty section is stated as empty when one source with areas fills it, even beside a source asked at points.

## Consequences

- Maps in these states get the water protection areas with their names; whether an event is possible depends on
  the zone, which only the lower water authority can say.
- Baden-Württemberg still has none; its own service names no licence.
- In Saxony the forest's zones from Sachsenforst and the whole areas of the BfG stand side by side, under
  different names for the same area at times.
