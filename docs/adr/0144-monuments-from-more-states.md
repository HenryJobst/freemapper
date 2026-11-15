# 0144. Monuments from more states, and who looks after them

Date: 2026-11-15
Status: Accepted

Research: [`denkmale-2026-11-15.md`](../research/wald-und-sprint-behoerden/denkmale-2026-11-15.md). Builds on
[0130](0130-permit-sources-as-tables.md) (table sources) and [0137](0137-permit-notes-for-schleswig-holstein.md)
(GML services).

## Context

Monuments came only from Berlin, Hamburg and Saarland. Outside the city states no hint said whom to ask about
them: Berlin and Hamburg name the office of the Bezirk, the others named nobody. The research found open services
with a clear licence in seven more states:

| State | Service | Answer | Licence |
|---|---|---|---|
| Hesse | `geodienste.denkx.de` (DenkXweb) | GeoJSON, WGS 84 | dl-de/by-2-0, LfDH |
| Baden-Württemberg | `owsproxy.lgl-bw.de` (two WFS of the LAD) | GeoJSON, WGS 84 | dl-de/by-2-0, LAD |
| Bremen | `geodienste.bremen.de/wfs_kulturdenkmale` | GML 3.2, WGS 84 | CC BY, Senator für Kultur |
| Saxony | `cardomap3.idu.de` (`ogc.ashx?PKGID=5`) | GML 3.2, WGS 84, gzip without its end | CC BY 3.0 DE |
| Brandenburg | `gis-bldam-brandenburg.de` (Bau- and Bodendenkmale) | GML 3.2, only by a box in EPSG:25832 | © BLDAM; dl-de/by-2-0 |
| Rhineland-Palatinate | `www.geoportal.rlp.de` (owsproxy) | GeoJSON, only in EPSG:25832 | dl-de/by-2-0, GDKE RLP |
| North Rhine-Westphalia | `www.wfs.nrw.de/wfs/denkmal` | GML 3.2, only in EPSG:25832, still being gathered | dl-de/zero-2-0 |

Lower Saxony (licence contradicts itself), Bavaria (CC BY-ND), Schleswig-Holstein (a 52 MB weekly file only),
Mecklenburg-Vorpommern (district services only) and Saxony-Anhalt (empty service, contradicting licence) wait.

## Decision

- **Hint:** monuments on a map outside the city states (no district office names them) bring the hint of the lower
  monument authority, named like the nature authority: the kreisfreie Stadt itself, or the Landkreis with the
  largest share, with the Gemeinde, as some states make the Gemeinde the authority.
- **Sources:** `denkmal_permits` describes the seven services as tables of `wfs_permits`. A monument reading can
  take its kind from an attribute (`kind_from`, North Rhine-Westphalia's `kategorie`).
- **Own CRS:** a table source can be asked and answer in a projected CRS (`native_source`): the box goes in metres
  with `srsName` of the same CRS (without it Brandenburg's Bodendenkmale answer in EPSG:25833), and the positions are
  turned into WGS 84. `geojson_wfs::get_projected_features` reads such pages.
- **GML points:** the GML reader of the permit services (`parse_gml_with_points`) reads points and multipoints and
  finds the geometry by its GML element whatever the property is called (`SHAPE`, `PrimaryGeometry`,
  `msGeometry`). The cadastre keeps `parse_gml`, polygons only.
- **Addresses with a query:** a WFS address may carry a query of its own (`?PKGID=5`); the rest follows after `&`.
- **Gaps:** a source can say where its data are not complete (`PermitSourceInfo::gaps`: Potsdam's Baudenkmale,
  North Rhine-Westphalia's Gemeinden); the notes say it below the part, and the authority hint comes there without
  a finding too ("the data cannot tell for sure"). A Stadtkreis counts as kreisfrei.
- **Broken gzip:** a download whose gzip ends early is asked once more with `Accept-Encoding: identity`.
- The hosts join the manifests of Hesse, Saxony, Brandenburg and Rhineland-Palatinate; the others were named already.

## Consequences

- Sprint maps in the old towns of these states list their monuments with a count per kind and, where the service
  has one, the link into the state's list; the hint says whom to ask.
- North Rhine-Westphalia's data are incomplete; its description says so, and no monument there does not mean none.
- Hesse's archaeological monuments are points that stand for an area of about 300 m, as the service says; the
  count is what the notes can say.
