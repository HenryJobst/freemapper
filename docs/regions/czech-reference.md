# Czechia: reference of the ČÚZK services

Probed live on 2026-11-26 (issue #356). No key, no login, only https. The data holder is the
Czech Office for Surveying, Mapping and Cadastre (ČÚZK; its Land Survey Office, Zeměměřický úřad,
runs the products below).

## Licence

ČÚZK gives its open data free of charge under **Creative Commons CC BY 4.0**
(<https://cuzk.gov.cz/Predpisy/Podminky-poskytovani-prostor-dat-a-sitovych-sluzeb/Podminky-poskytovani-prostorovych-dat-CUZK.aspx>).
The list names the RÚIAN exchange format, the INSPIRE GML files, and, since 1 July 2023 (amendment of
act 200/1994), **ZABAGED, the orthophotos of the Czech Republic, the state map series, the geographic
names and the control points**. The terrain and surface models (DMR 4G/5G, DMP 1G) are published as open
data on the same terms (ATOM services in the national open-data catalogue).

- Credit: `© ČÚZK`, as the services themselves state (`copyrightText`, `accessInformation`). The licence
  asks for the source, a link to the licence and a note when the work was changed; ATTRIBUTION.txt and
  the PDF credit line carry all three.
- Open question: in 2025 a few people on the OSM mailing list `talk-cz` doubted that the orthophoto
  **WMS** falls under CC BY 4.0. The official terms above name the orthophoto explicitly, so the open
  orthophoto data are covered; the tiles we ask for are the same data. We keep the doubt on record and
  ask the user to confirm the credit text before the first release.

## Services (all probed)

| Product | Service | Notes |
|---|---|---|
| Terrain model DMR 5G | `https://ags.cuzk.gov.cz/arcgis2/rest/services/dmr5g/ImageServer` | `exportImage`, `pixelType=F32`, `format=tiff`, EPSG:5514 (S-JTSK Krovak EN); native pixel 2 m; max 15000 × 4100 px per request; Bpv heights; mean error 0.18 m open, 0.30 m forest |
| Surface model DMP 1G | `https://ags.cuzk.gov.cz/arcgis2/rest/services/dmp1g/ImageServer` | same interface; gives the nDSM with DMR 5G |
| Orthophoto | `https://ags.cuzk.gov.cz/arcgis1/rest/services/ORTOFOTO/MapServer` | `export`, jpg, up to 4096 × 4096 px, EPSG:5514 and others; tile cache too (WMTS) |
| ZABAGED vector (149 layers) | `https://ags.cuzk.gov.cz/arcgis/rest/services/ZABAGED_POLOHOPIS/MapServer` | `Query` works: buildings (layer 99 area, 23 point), roads (79, 80, 83), railways (75, 76), water (93, 132), walls (39), forest land (140 to 144), grassland (139), lone trees (14) |
| ZABAGED contours | `…/ZABAGED_VRSTEVNICE/MapServer` | for comparison, not for the map |
| RÚIAN | `…/RUIAN/MapServer` | layers incl. `StavebniObjekt` (3), `Parcela` (5), `KatastralniUzemi` (7), `Obec` (12), protected areas (`NarodniPark`, `ChranenaKrajinnaOblast`, …) |
| Point cloud DMR 5G | ATOM `https://atom.cuzk.gov.cz/DMR5G-ETRS89/DMR5G-ETRS89.xml` (LAZ, ETRS89/TM33N) | 28 MB feed; tiles for the vegetation density of forest maps |
| OSM | `https://download.geofabrik.de/europe/czech-republic/czech-republic-latest.osm.pbf` | redirect to a dated file |
| Cadastral map WMS | `https://services.cuzk.gov.cz/wms/wms.asp` | view only, not used |

Hosts to name: `ags.cuzk.gov.cz`, `atom.cuzk.gov.cz`, `services.cuzk.gov.cz`, `download.geofabrik.de`
(the old host name `ags.cuzk.cz` redirects; metadata records still quote it).

## Checked with a request

- `dmr5g` `exportImage`, 1 km × 1 km at S-JTSK −760000, −1060000: 1000 × 1000 px Float32 GeoTIFF, 4.2 MB, 200.
- `ORTOFOTO` `export`, the same square at 2000 × 2000 px: JPEG, 1.1 MB, 200.
- `ZABAGED_POLOHOPIS` and `RUIAN` answer with `Map,Query,Data`; `ZABAGED_POLOHOPIS` on the older path
  `arcgis/rest/services/zabaged_polohopis` answers "Service not started", so the host path needs the
  upper-case name above.
- `ruian_wfs` on the old path answers 404; RÚIAN comes through the MapServer `Query` or the VFR files.

## What this means for the build

- **One service stack for everything**: heights, orthophoto, buildings, land cover and roads come from
  the same host and the same licence; no sheet index is needed (the services mosaic). That is simpler
  than Poland, whose reader has to pick sheets per year.
- The DMR 5G service is a 2 m raster resampled from the TIN. The LAZ point cloud gives the 1 m local
  relief and the vegetation density; both are reachable.
- RÚIAN parcels have been outdated since 10 December 2021 (no transfer from the cadastre, ČÚZK notice).
  Use RÚIAN for buildings and administrative units, not for parcels.
- Not yet checked: the flight years of the orthophoto. The permit sources are in the section "Permits probed" below.

## Built in stage 1 (ADR 0197)

- Buildings: RÚIAN layer `StavebniObjekt` (polygons, 334 in the 0.2 km² test area of Prague's Old Town); the code domain of
  `zpusobvyuzitikod` (1 to 30) maps to the German function words; 22 to 29 are water works and no buildings.
- Land use: the ZABAGED polygon layers 142/144 (forest), 140/141 (scrub), 139 (grassland), 138 (arable), 135 (orchards,
  gardens), 136 (vineyards), 137 (hops), 134 (maintained green), 116 (cemeteries), 131 (marsh), 18 (peat), 118 (quarries),
  114 (industrial sites) and 132 (water, `stojvoda` `A` standing, `N` flowing).
- Query: `MapServer/<layer>/query` with `f=geojson`, `inSR=outSR=25833`, `orderByFields=OBJECTID`, 1000 objects a page.
- Still open: the flight year of the orthophoto
  (the service names none), contours that wander in small loops under dense buildings (DMR 5G is interpolated bare earth).

## Permits probed (27 November 2026, ADR 0201)

All over https, no login, with the fixed research User-Agent. Query as in the cadastre: `MapServer/<layer>/query`
with the box in WGS 84 (`inSR=4326`, `outSR=4326`), `f=geojson`, `maxAllowableOffset=0.00005` for polygons.
A wrong attribute in `outFields` gives `{"error":{"code":400,"message":"Failed to execute query."}}` with status 200.

| Source | Service | Layers read | Licence |
|---|---|---|---|
| AOPK ČR, protected areas | `https://gis.nature.cz/arcgis/rest/services/UzemniOchrana/ChranUzemi/MapServer` (max 1000 records) | 0 small areas (`KAT` NPR, NPP, PR, PP), 1 their protective zones, 2 zones of NP and CHKO (`ZONA`), 3 large areas (`KAT` NP, CHKO, OP) | metadata of the INSPIRE edition: CC BY 4.0; `copyrightText` "© AOPK ČR" |
| AOPK ČR, Natura 2000 | `…/UzemniOchrana/Natura2000/MapServer` | 1 EVL, 0 PO (`SITECODE`, `NAZEV`) | as above |
| AOPK ČR, memorial trees | `…/PamatneStromy/PamatneStromy/MapServer` | 1 trees (points, `NAZEV`, `DRUH`), 3 groups without positions (polygons) | as above |
| RÚIAN, units | `https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer` (max 1 000 000) | 12 obec (`okres`, `pou`), 13 POU (`orp`), 14 ORP (`vusc`, only for the capital), 15 okres (`vusc`), 17 kraj | CC BY 4.0 |
| NPÚ, monuments | `https://geoportal.npu.cz/arcgis/rest/services/INSPIRE/ProtectedSites/MapServer` (max 2000) | 0 protected sites: `SD_DESIGNATION` `monument`, `settlement`, `cultural` | metadata record: CC BY-SA 4.0 |

- Adršpach box 16.10 to 16.14 E, 50.60 to 50.63 N: layer 0 gives the NPR "Adršpašsko-teplické skály" (KOD 2401) and the
  PR "Křížová cesta"; layer 3 the CHKO "Broumovsko"; layer 2 its zones I to IV; Natura 2000 the EVL "Adršpašsko-teplické
  skály" and "Metuje a Dřevíč" and the PO "Broumovsko". RÚIAN: the obec Adršpach (okres Náchod, 3605, kraj 86), POU 2283,
  ORP Broumov (1031). The NPÚ gives 381 sites in a box of about 1.5 km by 1 km in Prague's old town.
- The capital: obec Praha (554782) has `okres` empty; its ORP (19) carries `vusc` 19.
- ÚHÚL (`geoportal.uhul.cz`, `www.uhul.cz`, forest management plans): the first answers with a certificate for another host
  name, the second with a chain that does not verify. Not usable by a client that checks certificates.
- The old NPÚ host `gis.npu.cz` does not resolve; the geoportal is `geoportal.npu.cz`. The richer ÚAP service
  (`Tematicke/CP_UAP_PVO`, with register numbers and links) has no licence in its metadata and is not used.
- Law quoted in the hints (read on `zakonyprolidi.cz`): 114/1992 Sb. § 16 odst. 1 písm. q, § 29 písm. d, § 37 odst. 3
  písm. d, § 75 odst. 1; 20/1987 Sb. § 14 odst. 1 and 2; 289/1995 Sb. § 20 odst. 5; 13/1997 Sb. § 25 odst. 1 and 6 písm. d,
  § 40 odst. 4 and 5.
