# Bremen: services, licences and what the adapter uses

Research of 2026-11-13, the third state added on the adapter scheme. All requests were made with `curl` and
Python's `urllib`, with freemapper's own `User-Agent` and no personal data. The survey of 2026-11-08
(`states-survey.md`) listed Bremen as "partly": no land use service, the terrain only as one 1 GB archive, the
surface model only as a rendered map. Three of those points were wrong; this file replaces the row.

Agency: Landesamt GeoInformation Bremen (its services call it "GeoBremen"). The state is two cities 55 km apart,
Bremen and Bremerhaven, each with its own flights and its own archives. CRS: EPSG:25832. ALKIS object ids start
with `DEHB`. Geofabrik extract: `europe/germany/bremen` (20.3 MB on 2026-11-13).

## Licence

Open data since 9 June 2024. The open-data page of the agency
(`https://www.geo.bremen.de/sixcms/detail.php?gsid=bremen04.c.15613.de`) says: the open data are under
**CC BY 4.0**, and a use outside one's own house needs the source note
`© GeoBasis-DE / Landesamt GeoInformation Bremen (Jahr)`, clearly visible. Its product overview
(`https://www.geo.bremen.de/produkte/open-data-produktuebersicht-15654`) lists as free of charge, among others:
ALKIS FHB, ATKIS-DGM 1, DOM 1, ATKIS-DOP 10, Hausumringe, LoD2. The page adds that data counts as free only when
the user fetches it from a portal or service; a delivery on request costs a fee.

The services repeat the licence in their own words:

| Service | Text in its capabilities or catalogue record |
|---|---|
| WMS `wms_dop_lb` | `Fees`: "Creative Commons Namensnennung (CC-BY), Quellenvermerk: Landesamt GeoInformation Bremen"; no access restrictions |
| DGM1 and DOM1 downloads (MetaVer records) | "Creative Commons Namensnennung 4.0 International, Quellenvermerk: Landesamt GeoInformation Bremen" |
| WFS of the full ALKIS (on the LGLN's server) | CC BY 4.0; "Als Rechteinhaber und Bereitsteller ist "GeoBremen", sowie das Jahr des Datenbezugs in Klammern anzugeben. Beispiel für Quellenvermerk: GeoBremen (2024) Creative Commons Namensnennung – 4.0 International (CC BY 4.0)" |

freemapper writes the line of the agency's own page, with the licence and the note of changes:
`© GeoBasis-DE / Landesamt GeoInformation Bremen (<year>), CC BY 4.0, Daten geändert`.

Not open, by its catalogue record: the image-based surface model (bDOM, flight of March 2025), "gebührenpflichtig
und gesetzlich geschützt". The adapter does not use it.

The catalogue is MetaVer (`https://metaver.de`, full-text search `…/freitextsuche?q=…`, a record at
`…/trefferanzeige?docuuid=<id>`). The catalogues `gdk.gdi-de.org` and `inspire-geoportal.ec.europa.eu` reset the
connection from this host, as on 2026-11-08.

## Orthophoto

- WMS `https://geodienste.bremen.de/wms_dop_lb` ("DOP Luftbilder Land Bremen"): one layer per flight and city,
  back to 1951 in Bremen and 1937 in Bremerhaven. The newest are `dop10_2025_HB` and `dop10_2025_BHV` (10 cm);
  before them `dop10_2023_*`, `dop10_2021_*`. Infrared images are `dop10_cir_<year>_<city>`, scans of old photos
  `luftbild_<year>_<city>`, and two old layers write the resolution with an underscore
  (`dop12_5_2004_BHV`). The two cities were not always flown in the same year (`dop10_2019_HB`,
  `dop20_2018_BHV`).
- EPSG:25832, 25833, 3857 and others; JPEG and PNG; `MaxWidth` and `MaxHeight` 14 276. A request with both layers
  (`LAYERS=dop10_2025_HB,dop10_2025_BHV`) answers the image of whichever city the box lies in: 2 500 px in 1.2 s,
  4 000 px in 2.4 s. Outside the state the image is white, with status 200.
- The date is the year in the layer name. The service names no day: `GetFeatureInfo` is refused ("Layer(s)
  specified in QUERY_LAYERS parameter is not offered"). The catalogue record of the DOP10 gives 6 to 8 March 2025
  for the flight of Bremen.
- The survey named `wms_dop20_2023`. It still answers, with the flight of 2023 only.
- An INSPIRE WMS `wms_inspire_oi_dop10` is in the catalogue and answers 404.

## Terrain and surface model

File server `https://gdi2.geo.bremen.de/inspire/download`, no directory listing (403); the addresses come from
the MetaVer records "ATKIS - DGM1 Land Bremen - Downloaddienst" and "DOM1 Land Bremen". Four archives, each with
all 1 km tiles of one city as deflated XYZ text, and all answer range requests:

| Archive | Size | Tiles | A tile |
|---|---|---|---|
| `DGM/data/Gitternetz_DGM1_2017_HB_ASCII_XYZ.zip` | 1.09 GB | 395 | `dgm1_32487_5881_1_hb.xyz`, 21 MB, 2.8 MB deflated |
| `DGM/data/Gitternetz_DGM1_2015_BHV_ASCII_XYZ.zip` | 607 MB | 188 | `dgm1_32471__5933_1_hb.xyz` (two underscores), 28 MB, 3.4 MB deflated |
| `DOM/data/Gitternetz_DOM1_2017_HB_ASCII_XYZ.zip` | 1.24 GB | 395 | `dom1_32487_5881_1_hb.xyz`, 21 MB, 3.2 MB deflated |
| `DOM/data/Gitternetz_DOM1_2015_BHV_ASCII_XYZ.zip` | 325 MB | 114 | `dom1_32471_5933_1_hb.xyz`, 24 MB, 3.5 MB deflated |

The directory of an archive is 15 to 50 KB and takes three requests; a tile is one more. The archive of Bremen
also holds a `Thumbs.db`.

Every file starts with the line `x y z` and ends its lines with CRLF. The points are written in three ways:

| Archive | First lines | Meaning |
|---|---|---|
| DGM1 and DOM1 of Bremen | `487000 5881999 1.11` | whole metres, no zone number, from the north; 1000 × 1000 points from E to E + 999 |
| DGM1 of Bremerhaven | `32471000.5 5933999.5 -2.714` | cell centres, the zone number 32 in front, from the north |
| DOM1 of Bremerhaven | `32471000 5933000 3.3` | whole metres, the zone number in front, from the south; 1001 columns, the last one is the first of the next tile |

A point at whole metres is a grid point, not the centre of a cell. freemapper takes it for the cell to its
north-east, so three of the four models lie half a metre off in each direction (ADR 0121).

Both models come from the laser scans of 2017 (Bremen; the record of the DOM1 dates the data 15 March 2017) and
2015 (Bremerhaven), leaf-off. The catalogue gives an update every ten years. The surface model has outliers
(-12 m in a tile of Bremen, -61 m in one of Bremerhaven).

Also there, not used: `DGM5` and `DOM5` archives, a WFS `wfs_dom5` with the 5 m points, the rendered WMS
`wms_dgm1` and `wms_dom1`, the orthophoto as archives (`DOP/data/DOP10_RGB_JPG_HB.zip`), LoD2 as CityGML
(`LoD/data/LOD2_CITYGML_HB.zip`, 403 MB).

## Buildings and land use

Bremen serves no "ALKIS vereinfacht". Its full ALKIS is open, as a WFS that the LGLN of Lower Saxony runs
(XtraServer, AAA Suite, AdV schema GID 7.1):

- `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkishb_wfs_sf` ("WFS HB ALKIS AAA-Modell-basiert",
  simple features) and `…/alkishb_wfs_nas` ("NAS-konform"). 139 feature types each, EPSG:25832, GML 3.2 only.
  Both answer the same objects; the simple-feature form writes polygons and lines plainly and is a half to a
  ninth of the size. No paging (`ImplementsResultPaging` FALSE), no `CountDefault`, `numberMatched` "unknown",
  `SORTBY` refused. Several types in one request are refused ("Join not yet supported"), also in brackets.
- One feature type per land use class (`adv:AX_Wohnbauflaeche`, `adv:AX_Strassenverkehr`, … 26 of them),
  `adv:AX_Gebaeude`, `adv:AX_Bauteil`, `adv:AX_SonstigesBauwerkOderSonstigeEinrichtung`. Properties are AdV
  codes (`gebaeudefunktion` 2463, `funktion` 5310), the date is `lebenszeitintervall/beginnt`, the geometry is
  `position`.
- 4 km² around the main station (E 486 500 to 488 500, N 5 881 000 to 5 883 000): 5 335 buildings in 1.9 s
  (10.8 MB), 2 357 building parts, 965 structures, and 3 817 areas of the 26 land use types; the 28 requests
besides the buildings took 12 s together.

**From the full ALKIS to the simplified schema.** The simplified schema is derived from these objects, and the
LGLN serves both for Lower Saxony (`alkis_wfs_sf`, `alkis_wfs_einfach`), so the derivation could be compared
object by object. In five areas of Lower Saxony (Delmenhorst, Oldenburg, Hannover airport, Emden, Cuxhaven)
the rule below gave the class and the refinement of every land use area of the simplified service (7 463 of
7 463) and the kind, function and position of every building, building part and roof it holds (22 585); no
object differed. What the rule leaves out is listed after the table.

| Simplified | From |
|---|---|
| `oid` | `gml:id` + `BL` (buildings and structures) or `TN` (land use) |
| `aktualit` | the day of `beginnt` |
| `gebnutzbez` "Gebäude", `funktion` | `AX_Gebaeude`, label of `gebaeudefunktion` |
| `rellage` | label of `lageZurErdoberflaeche` (1200 "Unter der Erdoberfläche", 1400 "Aufgeständert") |
| `gebnutzbez` "Bauteil", `funktion` | `AX_Bauteil`, label of `bauart` |
| `gebnutzbez` "Sonstiges Bauwerk oder sonstige Einrichtung" | `AX_SonstigesBauwerkOderSonstigeEinrichtung` with `bauwerksfunktion` 1610 or 1611 (Überdachung, Carport) and an area |
| `nutzart` | the class of the feature type |
| `bez` | label of `funktion`; of `vegetationsmerkmal` for farmland and forest; of `bahnkategorie` for railways; of `art` for airfields; none for housing, heaps, mining, groves, heath, moor and swamp |

Left out, as in the simplified service: a land use area with `istWeitereNutzung` 1000 (a use that lies over
another one, mostly underground car parks). Left out by freemapper: towers, tanks, masts and chimneys (the
simplified service has them as further kinds; freemapper draws none of them) and the names of buildings (Bremen
writes file numbers there).

The labels are the AdV code lists of the GDI-DE registry
(`https://registry.gdi-de.org/codelist/de.adv-online.gid/<list>`, JSON under `…/<list>/<list>.de.json`), 21
lists with 502 codes, read 2026-11-13 into `assets/rules/adv-codes.toml`. They are the texts the simplified
services write: `Begleitfläche Straßenverkehr`, `Parkplatz`, `Grünanlage`.

Squares: 30 of 43 squares in the sample have a function, mostly `Parkplatz`. One without gets
`ohne Unterart (Bremen)` and stays a candidate, as in Lower Saxony.

**Surveyed lines.** The same service is the "full ALKIS (NAS)" source of the manifest (`[[extra.nas]]`): in the
sample, 136 stairs with 142 bottom edges, 45 ramps, 5 retaining walls, 18 monuments, 190 embankment tops,
20 bridges, 14 ditches. No hedges and no single trees there.

Also there, not used: the WFS `wfs_alkis_hausumringe` (outlines with the function code only), the WFS of parcels
`wfs_hduk2958loah3976niun`, the ALKIS as archives per city (`ADV-Shape/data/ALKIS_ADV_SHP_HB.zip`: the
simplified schema as shape files, whole city only; `ALKIS_o_E_o_KS/data/ALKIS_OE_OKS_NAS_HB.zip`: NAS).

## Boundary

`adv:AX_KommunalesGebiet` of the same WFS: three objects with five polygons, the city of Bremen with two
exclaves, one of them the port area inside Bremerhaven, and Bremerhaven. Their union is 419.6 km². Simplified to
25 m in EPSG:25833 it has four parts.

The municipal area of Bremerhaven reaches 8 km² into the Weser, and the boundary of Lower Saxony holds the same
water (both cadastres record the river there). Bremen's boundary file leaves it to Lower Saxony, so no two
states share ground; what is left in common are the strips where the two simplified borders cross.

## As built (2026-11-13, ADR 0121)

The adapter is `crates/adapters/de/manifests/bremen/` plus `crates/adapters/de/src/geobremen.rs`: id `bremen`,
provider `geobremen`, backend `geobremen`, EPSG:25832, object ids `DEHB`.

- **ALKIS** is code, but no code of Bremen: `freemapper_sources::aaa::AaaAlkis` reads the full ALKIS of any
  AdV service as the simplified schema; the adapter names the service and the `bez` of its squares.
- **Orthophoto**: the capabilities name the newest colour layer of each city; both are asked for in one request,
  in pieces of at most 4 000 px. The date of the image is the flight year of the city at the centre of the map
  area, recorded as `Bildflugjahr`. When the capabilities cannot be read, the layers of 2025 are asked for.
- **Terrain and surface**: the tiles of the area out of the archive of their city (north of N 5 910 000:
  Bremerhaven), each kept in the cache as a GeoTIFF named after the tile and the CRC-32 of its member. The date
  of both records is the year of the scan. The fetch warns that the surface model is older than the image.
- **OSM**: one extract.
- **The layers of the full ALKIS** come by the manifest, from the service the buildings come from.
- **Area page**: a preview is one layer, so the page shows the aerial photo of the city of Bremen and none of
  Bremerhaven.
- **Not built**: a check for a white image, a search for a newer archive when one is gone (its name carries the
  year of the scan; the fetch then fails with the address), the day of the flight.
