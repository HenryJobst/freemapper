# Hesse (HVBG) data sources for freemapper — live-verified reference

Verified live on 2026-11-07/08 (Europe/Berlin). Provider: **Hessische Verwaltung für Bodenmanagement und Geoinformation
(HVBG)**, the umbrella of the **Hessisches Landesamt für Bodenmanagement und Geoinformation (HLBG)** and the Ämter für
Bodenmanagement. HLBG is the publishing organisation in the open data portal (CKAN `opendata.hessen.de`, organisation
`hessisches-landesamt-fuer-bodenmanagement-und-geoinformation`) and the contact in the service capabilities; the
copyright and licence texts name HVBG. Open data since 01.02.2022 (Gesetz über die Bereitstellung offener Geobasisdaten
of 30.09.2021, § 18 HVGG). Data come from four hosts:

- the "Geodaten online" **Downloadcenter** `gds.hessen.de` (Intershop shop with a JSON REST API; 1 km tiles in one zip per
  municipality);
- the free WMS `www.gds-srv.hessen.de/cgi-bin/lika-services/…` (MapServer);
- the INSPIRE WMS/WCS `inspire-hessen.de/raster/…` (MapServer);
- the AAA-suite WFS `www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/…` (the same product as GeoSN's ALKIS vereinfacht WFS).

No login, registration, token or form is needed for anything below. Only the shop path (custom extracts, LiDAR point
clouds) has a cart and an account; freemapper does not need it.

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile |
|---|---|---|---|---|---|---|
| **Lessing-Gymnasium**, Fürstenbergerstraße 166, Frankfurt am Main (primary) | way **4795027** | E 476559 / N 5552594 | E 47838 / N 5571740 | 50.12512 / 8.67207 | E 476494–476624, N 5552532–5552656 | `32 476 5552` (fully inside) |
| Mainz cathedral area (Rhineland-Palatinate, border probe) | – | E 447940 / N 5538806 | E 18125 / N 5560266 | 49.9993 / 8.2736 | – | `32 447 5538` |
| ALKIS value samples only: Frankfurt centre (E 477500–479000, N 5550500–5552000), Marburg (E 483000–485000, N 5629000–5631000), Taunus forest near Idstein (E 458000–460000, N 5562000–5564000) | – | | | | | |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7°. The ALKIS school building
`DEHE06200002dyI4BL` came back at E 476519–476614 / N 5552562–5552634 in 25832, consistent with OSM. The WFS's own 25833
output matches my 25832 → 25833 conversion within **0.65 mm**, see §8.)

**Note on the Frankfurt school's eastings in zone 33: E ≈ 47.8 km.** Hesse spans E −17 km … +167 km in EPSG:25833, so
part of the state has **negative 25833 eastings** (§3).

Samples (all under `hessen/`; treat them as untrusted data):

| Path | Content |
|---|---|
| `dl/dop/DOP20_32_476_5552_1_he.jpg` + `.jgw`, `DOP20_32_462_5548_1_he.jpg`, `school_crop.jpg` | the school's 1 km DOP20 tile (flight 19.08.2025) and a Frankfurt border tile, range-read out of the 1.2 GB Frankfurt zip |
| `dl/dgm1/DGM1_32_476_5552_1_he.tif` + `.tfw`, `DGM1_32_462_5548_1_he.tif` | DGM1 school tile and a Frankfurt border tile, range-read from the 630 MB Frankfurt zip |
| `dl/dom1/DOM1_32_476_5552_1_he.tif` + `.tfw` | DOM1 school tile, range-read from the 836 MB Frankfurt zip |
| `dl/border/` | Wiesbaden DGM1 tile `dgm1_32_448_5538` (Rhine, older build without GeoKeys), WCS DGM1 at Mainz (`wcs_mainz.tif`, mask PNG), WMS DOP at Mainz (`wms_mainz.jpg/png`) and far outside (`wms_far.jpg`), ALKIS hits at Mainz |
| `dl/wcs/` | WCS GetCoverage clips: DGM1 200 m (school), 2 km, fractional bbox, 25833 output; DOM1 200 m; `dom_compare.png` (DOM tile vs WCS DOM) |
| `dl/wms/` | DOP WMS GetMaps: school in 25832 and **25833**, native-grid PNG, 3000/3001/4000 px probes, layer `he_dop_rgb` vs `he_dop20_rgb`, qTrueDOP comparison (`compare_dop20_qtrue.jpg`), INSPIRE DOP20 WMS probes, GetFeatureInfo answers |
| `dl/wmstime/` | four 3000 × 3000 px WMS pieces (timing test) |
| `dl/meta/` | `Metadaten_DOP.xlsx` (flight date per DOP tile), `Metadaten_DGM-DOM_August-2026.xlsx` (flight date per DGM/DOM tile), both converted to TSV; the two "Aktualitätsübersicht" PDFs; the 1 km tile index shapefile |
| `dl/docs/agb.txt` | text of the Geodaten-online terms ("Bedingungen für die Bereitstellung und Nutzung …") |
| `probe/gds/` | Downloadcenter REST answers (tree, per-district listings, paging), the page's JS (to find the REST endpoint) |
| `probe/zip/zip_*.tsv` | member lists (name, size, compressed, method, date, local offset) of the Frankfurt DGM1/DOM1/DOP20 zips and the Wiesbaden DGM1 zip, read from their central directories |
| `probe/alkis/` | ALKIS WFS answers: school bbox (25832 and 25833), 2 × 2 km around the school, Frankfurt centre, Marburg, Taunus forest, paging tests, error probes |
| `probe/boundary/` | `VerwaltungsEinheit` answers (25832/25833), the extracted state polygon `land_DE06_25833.xml`, and **`hessen_simplified25m.geojson`** |
| `probe/caps/`, `probe/iso/`, `probe/ckan/`, `probe/ogcapi/`, `probe/geofabrik/`, `probe/osm/` | capabilities (WMS, WCS, WFS), ISO metadata, CKAN package lists, the broken geoportal OGC API proxies, Geofabrik state/poly, Overpass answers |
| `scripts/` | helper scripts (UTM, zip-by-range, TIFF tags, xlsx → TSV, GML attribute summary, boundary simplification) |

---

## 1. Brandenburg → Hesse at a glance

| freemapper product | Brandenburg (today) | Hesse equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** Hesse lies at 7.77–10.24°E. In 25833 it spans **E −17 106 … 167 166** (negative eastings), with ≈ −4.9° grid convergence and scale +2.1 m/km in Frankfurt. WMS and WFS reproject to 25833 on request; the WCS resamples. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **DOP20 RGB (TrueDOP, on a bDOM), 1 km JPEG (q 75) + JGW**, in **one zip per municipality** from the Downloadcenter; members are *stored*, so a tile is one HTTP range (≈4 MB). Or the **WMS** `ogc-free-images.ows` layer `he_dop20_rgb` (**max 3000 px**, 10–34 s per piece), the same pixels. Flights 2023/2024/2025 by region, **leaf-on (summer)**; the school tile is from 19.08.2025. | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1** GeoTIFF Float32, **LZW + horizontal predictor**, 1 km, + TFW, NoData −9999 (ALS 2019–2025); zip per municipality (≈2 MB per tile, deflate). Or **WCS 2.0** `he_dgm1`, byte-identical to the tiles, but slow (16 MB in 63 s). | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **DOM1 (ALS, 1 m)** − DGM1, same flight, same grid and format. No nDOM, no bDOM download; LiDAR point clouds only via the shop on request. | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `ave:GebaeudeBauwerk` (AAA-suite, like GeoSN), **GML 3.2 only**. **LGB spellings** ("Gebäude"), `rellage` present; `funktion` is only the coarse class ("Gebäude für öffentliche Zwecke"). | verified |
| Land use | OGC API `nutzung_flurstueck` | WFS **`ave:NutzungFlurstueck`** (LGB spellings, `bez` set). **`ave:Nutzung` has no `nutzart` in Hesse**, so it is unusable. `bez` is never "Fahrbahn"/"Begleitfläche" on Straßenverkehr. | verified |
| Currency | `bb_*_aktualitaet.csv` | `Metadaten_DOP.xlsx` (flight date per DOP tile and municipality), `Metadaten_DGM-DOM_<Monat>-<Jahr>.xlsx` (ALS date per tile), WMS GetFeatureInfo layer `he_dop_rgb_info` (`bildflug`), ETag/Last-Modified on the zips, `aktualit` per ALKIS feature | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `hessen-latest.osm.pbf` (347.6 MB) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **dl-de/zero-2-0** (§ 18 HVGG: any use without condition). **No attribution is required**; *if* a source note is given, it must say that the data were changed. Copyright text in the services: "Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation". | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **Downloadcenter REST** | `https://gds.hessen.de/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter?path=<Themengebiet>/<Produkt>/<Kreis>[&page=N]` | the tree and the file list per district | JSON; 0.08–0.2 s; found in the page's JS chunk `downloadCenter.*.js`. Not documented. |
| **Downloadcenter files** | `https://gds.hessen.de/downloadcenter/<YYYYMMDD>/<Themengebiet>/<Produkt>/<Kreis>/<Datei>.zip` | DOP20, DGM1, DOM1, documents | Apache; **HEAD → 405**; GET with `Range` → 206, `Content-Range` with the total, ETag, Last-Modified. The date segment is a validity window (§2.2). |
| **Static documents** | `https://gds.hessen.de/INTERSHOP/static/WFS/HLBG-Geodaten-Site/-/HLBG-Geodaten/de_DE/Downloadcenter/Daten/…` | metadata xlsx, overview PDFs | `Last-Modified` is regenerated per request (useless for change detection) |
| **Free WMS** | `https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows` (DOP), `…/ogc-free-maps.ows` (maps, DGM hillshades) | DOP RGB/CIR | MapServer; MaxWidth/Height **3000** |
| **INSPIRE raster** | `https://inspire-hessen.de/raster/{wms,}/{dop20,qtruedop,dgm1,dom1}/ows` | WMS (`OI.OrthoimageCoverage`, …) and **WCS 2.0.1/2.1.0** | MapServer; WMS max 4000 px; WCS output 25832 or 4326 |
| **ALKIS WFS** | `https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs` (also `…/alkis/nas/wfs`, `…/alkis/sf/wfs`) | ALKIS vereinfacht | AAA-suite (the GeoSN software), GML 3.1/3.2 and zipped shapefile |
| **Boundary shapefile** | `https://gds-srv.hessen.de/atomfeed/DigVGr-epsg25832-shp.zip` | Verwaltungsgrenzen (Land, RB, Kreise, Gemeinden, Gemarkungen, Fluren) | 67,733,515 B, Last-Modified 22 Jul 2026, ranges OK |
| CKAN | `https://opendata.hessen.de/api/3/action/package_search` | catalogue | 37 HLBG packages, licence `dl-zero-de/2.0` (LoD2 says `dl-by-de/2.0`) |
| ISO metadata | `https://www.geoportal.hessen.de/mapbender/php/mod_dataISOMetadata.php?outputFormat=iso19139&id=<uuid>` | per dataset | DOP20 `0b30f537-3bd0-44d4-83b0-e3c1542ca265`, DGM1 `dbf48a95-…`, DOM1 `4540651c-…`, ALKIS `a913ebfb-…` |

Dead or misleading endpoints:
- The CKAN "OGC API Features (REST)" links `https://www.geoportal.hessen.de/spatial-objects/{691,696,698,709,711}/api` all answer
  200 `{"success":false,"message":"WFS with id does not exists!"}`. There is **no working OGC API Features** for ALKIS.
- The geoportal "registry" WFS URLs (`…/registry/wfs/691?…`) return the capabilities of the gds.hessen.de WFS. Use the
  original URL.
- The ATOM "Rasterdownload" feeds (`mod_inspireDownloadFeed.php?…&generateFrom=wmslayer`) are generated from the WMS
  (GetMap tiles), not real files. Not examined further.
- `…/HLBG-Site/Geodaten/…/Metadaten_DOP.xlsx` (the link in the DOP product text) → **403** with an `image/jpeg` body.
  The working copy is under `…/HLBG-Geodaten-Site/-/HLBG-Geodaten/…` (the `Dokumente Luftbildinformationen` entry).

### 2.1 Tile scheme

- **1 km × 1 km**, EPSG:25832, SW corner at whole km, for DOP20, DGM1 and DOM1. The same tile ids for all three.
  - DOP: `DOP20_32_EEE_NNNN_1_he.jpg` + `.jgw` (e.g. `DOP20_32_476_5552_1_he.jpg`).
  - DGM1: `DGM1_32_EEE_NNNN_1_he.tif` + `.tfw` in the 2026 builds (Frankfurt, members dated 2026-07-28); **`dgm1_32_…` in
    lower case** in older builds (Wiesbaden, members dated 2025-05-08). Match case-insensitively.
  - DOM1: `DOM1_32_EEE_NNNN_1_he.tif` + `.tfw`.
  - Metadata names: DOP xlsx `DOP20_32_476_5552_1_he`; DGM/DOM xlsx a 7-digit `4765552` (EEE + NNNN).
- Grid: SW corners E 412…586, N 5471…5723. The DOP metadata has 21,868 tiles; the tile index shapefile 22,941.
- **Tiles are complete squares, not clipped** to the municipality or the state (DGM1 border tiles 462/5548 and
  448/5538 have no NoData; the DOP border tile 462/5548 has no fill). A tile that touches several municipalities is in
  **each** of their zips (9,548 duplicate rows in the metadata: e.g. 445/5555 is in Niedernhausen, Taunusstein and
  Wiesbaden).
- The tile set covers every 1 km square that touches Hesse. Outside it the WCS returns −9999 (§5) and the WMS returns
  white (§4.2). The DOP was flown over the border tiles too (e.g. 447/5538 in Mainz, flight 07.07.2023, `land = 'he'`).

### 2.2 Downloadcenter: tree, files, date window

- Tree (`…/downloadcenter?path=<path>&navigation=all`): `Luftbildinformationen/Digitale Orthophotos DOP20/<Kreis>`,
  `3D-Daten/Digitales Geländemodell (DGM1)/<Kreis>`, `3D-Daten/Digitales Oberflächenmodell (DOM1)/<Kreis>`. 27 districts
  (21 Landkreise + 5 kreisfreie Städte + Wiesbaden). No LiDAR node (3D-Daten has only 3D-Gebäudemodelle, DGM1, DOM1,
  Dokumente).
- A district listing returns `searchresult.downloads[]` with `name`, `fileExtension`, `fileSize` ("600,4 MB"),
  `creationDate` ("05.08.2026"), `downloadType` (`File`/`Package`/`ExternalLink`) and `downloadLink.uri`. One file per
  municipality; `paging.pageSize` 20, more with `&page=2` (Marburg-Biedenkopf: 22 municipalities on 2 pages). Some
  districts also list a `PKT_<Kreis>.zip` "Alle Dateien" package.
- Example (Frankfurt, DGM1):
  ```
  GET https://gds.hessen.de/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter?path=3D-Daten%2FDigitales+Gel%C3%A4ndemodell+%28DGM1%29%2FKreisfreie+Stadt+Frankfurt+am+Main
  → {"name":"Frankfurt_am_Main-DGM1","fileSize":"600,4 MB","creationDate":"05.08.2026","downloadType":"File",
     "downloadLink":{"uri":"/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Kreisfreie Stadt Frankfurt am Main/Frankfurt_am_Main-DGM1.zip"}}
  ```
- **File names are irregular** and must be taken from the listing: `Frankfurt_am_Main-DGM1.zip`, `Wiesbaden - DGM1.zip`,
  `Amöneburg - DOP20.zip`, `Kirchhain-DOP20.zip`, `Neustadt_(Hessen)-DOP20.zip`, `Weimar (Lahn) - DOP20.zip`. The
  district node names also differ by product (DOP: `Main-Taunus_Kreis`, DGM1: `Main-Taunus-Kreis`) and from the metadata
  xlsx (`kreis_bz` "Main-Taunus", "Landeshauptstadt Wiesbaden" vs node "Kreisfreie Stadt Wiesbaden").
- **Date window:** the `downloadLink.uri` carries a date (`/downloadcenter/20261108/…` on 07.11.2026, 23:55 CET).
  Tested at the same moment: 20261107 → 206, 20261108 → 206, 20261105/06/09/10 → **404**, 20250101 → 404. A cached URL
  therefore dies after a day or two. **Resolve the file URL from the REST API right before each download.**
- Sizes: Frankfurt DGM1 629,542,574 B, DOM1 835,872,036 B, DOP20 1,218,993,433 B (310 tiles each). Marburg-Biedenkopf DOP
  zips 112–577 MB. Largest seen 1.2 GB, so no zip64 (not checked statewide).
- Zip layout (Frankfurt; flat, no folders; read from the central directory at the end, 44–67 KB):
  - DOP: `.jpg` **stored (method 0)**, 3.9 MB average; `.jgw` deflated (90 B).
  - DGM1/DOM1: `.tif` **deflated (method 8)** but the TIFF is LZW inside (2.1 MB → 2.1 MB); `.tfw` deflated.
  - Reading one tile = 1 ranged GET for the 70 KB tail (once per zip, cache by ETag) + 1 for the local header + 1 for the
    member: 2–6 s for 2–4.5 MB.
- HTTP: `Last-Modified` (Frankfurt DGM1: Wed, 05 Aug 2026 10:25:04 GMT), `ETag "25860eae-6584a2f457286"`, `Accept-Ranges:
  bytes`, `Cache-Control: no-cache, no-store`. **One ranged GET came back empty** (0 bytes, curl gave no error); the retry
  worked. Check the length of every range answer.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DOP: metadata `Koordinatenreferenzsytem Lage 25832`; JGW in 25832 (no GeoTIFF).
  - DGM1/DOM1: GeoKeys `ProjectedCSType 25832` in the 2026 builds; **no GeoKeys at all** in older builds (Wiesbaden
    2025-05). Only tiepoint and pixel scale are always there, and they are all freemapper's decoder reads.
  - WFS DefaultCRS `urn:ogc:def:crs:EPSG::25832`; WCS native 25832; WMS layers list only 25832, the root layer adds 25833 …
- **Height datum DHHN2016** normal heights (HVBG DOM page: "ETRS89 / UTM 32N mit DHHN2016_NH"; DOP metadata
  `Koordinatenreferenzsytem Hoehe 7837`). No vertical GeoKey in the TIFFs. The same system as the other freemapper states.
- **Zone 33 is very poor for Hesse:**
  - Frankfurt: convergence −4.86° (zone 32: −0.25°), scale factor 1.00211 (+2.1 m per km);
  - west edge (7.77°E): convergence −5.55°, scale 1.0029 (+2.9 m per km);
  - in 25833 the state spans **E −17 106 … 167 166**, so the west (Rheingau, Wiesbaden, Limburg) has **negative
    eastings**.
  - Hesse maps must be **25832 projects**, like Saxony-Anhalt and Hamburg (`Region::crs() → Crs::Utm32`).
- **Extent** of the state polygon (`VerwaltungsEinheit` DE06, 38,134 vertices, 2 polygons):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 412134 – 586409, N 5471409 – 5723092 |
  | EPSG:25833 | E −17106 – 167166, N 5489603 – 5737357 |
  | lon / lat (Geofabrik poly) | lon 7.768 – 10.246, lat 49.393 – 51.659 |

- 25833 availability:
  - DOP WMS: CRS list includes EPSG:25833 (root layer); GetMap in 25833 verified (school, visually checked, rotated).
  - ALKIS WFS: OtherCRS 25833, 4258, 4326. Verified; output in E N order; matches my transform within 0.65 mm.
  - WCS: `OUTPUTCRS=…/25833` works but **resamples** to 1.0025 m pixels (232 × 232 for 200 m). Not usable as a native
    grid.

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- HVBG renews the DOP every two years (product page: "Seit 2008 werden die Digitalen Orthophotos im 2-Jahresturnus
  erneuert"); since the 2020 flight they are processed as **TrueDOP** (on a bDOM; metadata `Bezugsflaeche BDOM`).
- The current statewide mosaic (overview "DOP-Übersicht, Aktualität 20.10.2026", metadata sheet
  `MetadatenGemeinden-BF_2025-3`) is a patchwork of three flight years:
  - 2025: the south and centre (Frankfurt, Wetterau, Main-Kinzig, Vogelsberg, Darmstadt, Bergstraße, Odenwald …),
    13,813 rows;
  - 2024: the north (Kassel, Waldeck, Marburg, Gießen, Lahn-Dill, Fulda, Hersfeld …), 14,383 rows;
  - 2023: the south-west (Rheingau-Taunus, Main-Taunus, Limburg, Wiesbaden), 3,220 rows.
- Flights are **leaf-on summer flights**: the school tile was flown on **19.08.2025** (UltraCam Eagle M3); the
  Rheingau-Taunus tiles on 07.07.2023. The orthophoto shows full foliage (`dl/dop/school_crop.jpg`). Hesse has no
  leaf-off DOP open.
- **qTrueDOP** ("quick TrueDOP", INSPIRE WMS/WCS `inspire-hessen.de/raster/{wms/,}qtruedop/ows`): a fast TrueDOP
  processing below AdV quality ("entspricht nicht den Anforderungen des Produkt- und Qualitätsstandards (PQS) der
  AdV"), alternating spring (March/April) and summer (May/June). **At the school it is a different, older image** (the
  school roof has no solar panels, which the 2025 DOP20 shows; `dl/wms/compare_dop20_qtrue.jpg`). Its `Metadaten`
  layer's GetFeatureInfo returned "Nothing found". Its date is unknown, so do not use it.
- Only the current DOP is served. Historic orthophotos are a separate product (CKAN `historische-digitale-orthofotos1`),
  not examined.

### 4.2 WMS (`ogc-free-images.ows`)

- `https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows`, WMS 1.3.0, title "WMS HE DOP".
  - Layers: `he_dop20_rgb` (**MaxScaleDenominator 9000**), `he_dop_rgb` (all scales; byte-identical to `he_dop20_rgb` at
    0.2 m), `he_dop20_cir`, `he_dop_cir`, `he_dop_alk_t` (with cadastral overlay), info layers `he_dop_rgb_info`,
    `he_dop_cir_info` (queryable).
  - CRS: 25832, 25833, 31466–31469, 4326, 4258, 3857, 3038–3047 … (root layer).
  - Formats: jpeg, png, png 8 bit, `vnd.jpeg-png(8)`, tiff, json.
  - **MaxWidth/MaxHeight 3000** (600 m at 0.2 m). 3001 px →
    **HTTP 200 `text/xml`**: `msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be
    between 1 and 3000 pixels.` freemapper's `wms::WMS_MAX_PX` (4000) is too large for this service.
  - No TIME dimension.
- Example (school, 25832, 0.2 m):
  ```
  https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=he_dop20_rgb&STYLES=&CRS=EPSG:25832&BBOX=476359,5552444,476759,5552744&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 545,436 B, **12.3 s** (repeat: 9.7 s). 25833 (`BBOX=47635,5571592,48035,5571892`): 550,469 B,
  14.5 s, visually checked.
- **Slow and occasionally hanging:**
  - 3000 × 3000 px pieces at the school: 28.7, 21.3, 34.0, 15.9, 24.6 s (1.56–1.73 MB each). A 1 km² map at 0.2 m (4 pieces of
    ≤ 3000 px) takes 1.5–2 minutes.
  - **Two requests hung with no byte**: a 2000 × 1500 px one for 300 s and a 3000 × 3000 px one (Taunus,
    E 470000–470600) for 180 s (curl timeouts, exit 28). Other requests before and after worked.
  - Empty areas are fast (0.3 s); Mainz 2 × 2 km at 1 m/px took 3.8 s.
  - Use a per-request timeout (≈ 90 s) and retries.
- **On the native grid in 25832 (PNG), the WMS returns the tile's pixels**: mean abs difference **5.4** (0–255) against the
  DOP20 tile, against 6.9–11.9 for 1 px shifts. The higher value than in Hamburg (2.4) comes from the double JPEG
  q75 path; the WMS JPEG uses the same q75 tables.
- Outside the DOP coverage: JPEG **white** (255), PNG with `TRANSPARENT=TRUE` transparent. Inside border tiles flown by
  HVBG (Mainz) the WMS shows imagery of the neighbouring state.
- Headers: `Cache-Control: no-cache, no-store`, chunked, no ETag, no Last-Modified.
- **GetFeatureInfo gives the flight date per tile:**
  ```
  …ogc-free-images.ows?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=he_dop_rgb_info&QUERY_LAYERS=he_dop_rgb_info&STYLES=&CRS=EPSG:25832&BBOX=476400,5552400,476600,5552600&WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain
  → name = 'dop_476500_5552000'  land = 'he'  aufloesung = '20'  farbe = 'rgb'  bezug = 'BDOM'
    flugjahr = '2025'  bildflug = '19-08-2025'  aenderung = '12-10-2026'
  ```
  3.6 s. `name` is not a reliable tile id (Mainz gave `dop_447500_5538500` for tile 447/5538, the school
  `dop_476500_5552000` for 476/5552). Outside the coverage: "Search returned no results."
- Fees/AccessConstraints of the capabilities: see §9.

### 4.3 INSPIRE WMS (`inspire-hessen.de/raster/wms/dop20/ows`)

- Layer `OI.OrthoimageCoverage`, MaxWidth/Height **4000**, CRS 4258, 4326, **25832**, 31467, CRS:84 (**no 25833**),
  formats png, jpeg, gif, GeoTIFF, tiff. Fees "Datenlizenz Deutschland – Zero – Version 2.0".
- Probes (`dl/wms/insp_*`, `compare_tile_inspire.jpg`):
  - 4000 × 4000 JPEG (800 m): 200, 3,977,274 B, **64.9 s**. 4001 px → HTTP 200 `text/xml` "image size too …".
  - `CRS=EPSG:25832` native-grid PNG (1000 × 1000, 0.2 m): 200, 12.1 s, an 8-bit **palette** PNG.
  - `CRS=EPSG:25833` → HTTP 200 `text/xml` ServiceExceptionReport.
  - **It is not the current DOP:** mean abs difference 33 against the 2025 tile at every shift, and visually an older,
    non-true orthophoto (buildings lean, the school roof has no solar panels). **Do not use the INSPIRE DOP20 WMS**
    (nor the INSPIRE DOP20 WCS `he_dop20`, presumably the same source; not tested).

### 4.4 Tile downloads (Downloadcenter)

- URL from the REST listing, e.g.
  `https://gds.hessen.de/downloadcenter/20261108/Luftbildinformationen/Digitale%20Orthophotos%20DOP20/Kreisfreie%20Stadt%20Frankfurt%20am%20Main/Frankfurt_am_Main-DOP20.zip`
  (1,218,993,433 B, zip `creationDate` 02.11.2026, members dated 2026-10-08).
- Tile `DOP20_32_476_5552_1_he.jpg`: **stored**, 4,553,494 B at local offset 724,941,014; read by range in 5.9 s.
  - JFIF baseline JPEG, **5000 × 5000, RGB** (3 channels; RGBI and CIR only on request, for a fee); quantisation tables
    = IJG **quality 75**; no EXIF, no georeferencing inside.
  - JGW (LF line ends):
    `0.2 / 0 / 0 / -0.2 / 476000.1 / 5552999.9000000004` (pixel centres; float noise in the last line, round to mm).
    The same convention as LGB's JGW.
  - Not clipped: the Frankfurt border tile 462/5548 has 0.013 % white pixels and no black fill.
- This is the same format as Brandenburg's DOP20 tiles (JPEG + JGW per 1 km), only packed per municipality and fetched
  by range.

---

## 5. DGM1 (terrain model, 1 m)

- Product "Digitales Geländemodell (DGM1)": "Höhenmäßige Beschreibung des Geländereliefs … in Rasterform (Pixel) mit der
  quadratischen Fläche von 1 m * 1 m … Der originäre Höhenbezug ist die Rastermitte." From statewide ALS (Riegl
  VQ-780 family), flights **2019–2025** by region (overview "3D-Übersicht, Aktualität 4.08.2026"). The school tile was
  flown on **28.02.2023**. HVBG aims at a six-year cycle.
- **Tile download** (zip per municipality, see §2.2). School tile from the Frankfurt zip: member
  `DGM1_32_476_5552_1_he.tif` 2,080,264 B (deflate 2,065,434) at offset 373,756,777, read in 2.1 s.
- TIFF (2026 build):
  - 1000 × 1000 **Float32** (SampleFormat 3), **Compression 5 (LZW) with Predictor 2** (horizontal differencing on the
    32-bit words), 50 strips of 20 rows, IFD at the **end** of the file;
  - ModelPixelScale 1, **ModelTiepoint exactly (476000, 5553000)**, GTRasterType 1 (PixelIsArea), ProjectedCSType 25832,
    GeoAscii "ETRS89 / UTM zone 32N|ETRS89|", `GDAL_NODATA` "-9999", Software "LAStools is a product of rapidlasso GmbH";
  - TFW `1/0/0/-1/476000.5/5552999.5`, the **same registration as LGB/GeoSN/LVermGeo/LGV DGM1**.
  - The older Wiesbaden build (`dgm1_32_448_5538_1_he.tif`, 2025-05): the same, but **no GeoKeyDirectory**.
  - Pillow decodes both. The `tiff` crate 0.11.3 (freemapper's decoder) supports `Predictor::Horizontal` for
    `SampleFormat::IEEEFP` (checked in its source); not run against these files.
- Values: school tile 97.60–124.24 m, school centre **109.90 m** (pixel E 476559.5 / N 5552593.5); no NoData in the
  school tile, the Frankfurt border tile, or the Rhine tile 448/5538 (80.45–126.48 m, it includes the Mainz bank).
- **WCS** (INSPIRE): `https://inspire-hessen.de/raster/dgm1/ows`, WCS 2.0.1/2.1.0, coverage **`he_dgm1`** (+ `_2 … _32`
  overviews), axis labels `E N`, origin (411000.5, 5723999.5), NoData −9999.
  ```
  https://inspire-hessen.de/raster/dgm1/ows?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=he_dgm1&FORMAT=image/tiff&SUBSET=E(476400,476600)&SUBSET=N(5552400,5552600)
  ```
  → 200 `image/tiff`, 160,584 B, 5–8 s. Uncompressed Float32, tiepoint exactly (476400, 5552600), GeoKeys 25832,
  GDAL_NODATA −9999. **Byte-identical to the download tile** (max |Δ| 0.0; a 1 px shift gives 3–4.5 m).
  - A fractional bbox (`E(476400.3,476600.7)`) is snapped to whole metres (tiepoint 476401, 200 px).
  - **Slow:** 2 × 2 km (16 MB) took **63 s**; 6 × 6 km did not finish within 120 s. That is ≈ 0.25 MB/s, against ≈ 1 MB/s
    for the zip ranges.
  - Outside the tile set: −9999 (Mainz 1.5 × 1.5 km: 44 % −9999, in the shape of the missing tile 447/5538; 2 km outside:
    100 %).
- The DGM WMS layers in `ogc-free-maps.ows` (hillshades) were not examined.

---

## 6. nDSM sources: DOM1, LiDAR

### 6.1 DOM1 (1 m, from ALS)

- "Datengrundlage sind die durch Laserscanning gewonnen dreidimensionalen Messpunkte"; the same flights and the same
  metadata list as the DGM1, so **DOM1 and DGM1 of a tile have the same date**. GeoTIFF Float32 LZW, NoData −9999
  (HVBG DOM page). DOM5 and hillshades on request (fees); a bDOM is mentioned only as an internal basis.
- Frankfurt zip: `DOM1_32_476_5552_1_he.tif` 3,068,330 B (deflate 3,000,975); the format is identical to DGM1 (LZW +
  predictor 2, tiepoint (476000, 5553000), TFW `…/476000.5/5552999.5`, GeoKeys 25832, GDAL_NODATA −9999). Values
  99.63–214.48 m.
- **DOM1 − DGM1** over the school footprint bbox (16,120 px): min −0.96, p10 0.01, **median 3.90**, p90 16.82, max 31.44 m;
  55.1 % > 2 m. Clamp < 0 to 0. The ALS (28.02.2023, leaf-off) is 2.5 years older than the DOP (19.08.2025, leaf-on).
- **WCS `dom1`** (`https://inspire-hessen.de/raster/dom1/ows`, `COVERAGEID=dom1`) **is not the download DOM**: on the same
  grid only 0.2 % of the pixels are equal, mean |Δ| 0.80 m, 28 % differ by > 0.5 m, p1/p99 −6.2/+8.4 m. Buildings agree;
  the differences are in tree crowns (`dl/wcs/dom_compare.png`). Probably another flight or processing; which one is
  newer is unknown. **Use the download tiles**, whose date is in the metadata list.

### 6.2 LiDAR / nDOM

- No open point cloud download. The HVBG open data page lists "Airborne Laserscanning" under "Shop" only (order, account),
  and the Downloadcenter has no LiDAR node. No nDOM product exists.
- DOM1 (ALS) is a better nDSM source than Hamburg's image-based bDOM; no 0.2 m surface model is open.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **`Metadaten_DOP.xlsx`** `https://gds.hessen.de/INTERSHOP/static/WFS/HLBG-Geodaten-Site/-/HLBG-Geodaten/de_DE/Downloadcenter/Daten/Luftbildinformationen/Metadaten_DOP.xlsx` (1,206,127 B) | DOP20 | One sheet `MetadatenGemeinden-BF_2025-3`; row 1 = release date as an Excel serial (46315 = 2026-10-20); row 2 header `Kachelname, Bildflugdatum, gmde_bz, kreis_bz, regbez_bz, Kamera_Sensor, Koordinatenreferenzsytem Lage, Koordinatenreferenzsytem Hoehe, Bezugsflaeche`. **Bildflugdatum is an Excel serial date** (45888 = 2025-08-19). 31,416 rows = tile × municipality (21,868 tiles). Also gives the municipality of each tile, i.e. which zip(s) hold it. |
| **`Metadaten_DGM-DOM_<Monat>-<Jahr>.xlsx`** (now `…/3D-Daten/Metadaten_DGM-DOM_August-2026.xlsx`, 972,183 B) | DGM1, DOM1 | Sheet `2026-08-04_Metadaten`; row 1 "Aktualitä: 5.08.2026"; header `Kachelname` (7 digits, `4765552`), `Bildflugdatum` (Excel serial; 44985 = 2023-02-28), `Sensor`, `Gemeinde`, `Kreis`, `Regierungsbezirk`. 31,415 rows / 21,867 tiles. **The file name changes with each release**: take the URL from the DGM1 product description in the REST answer (`searchresult.product.description` contains the links). |
| WMS GetFeatureInfo `he_dop_rgb_info` | DOP20 | `bildflug` (DD-MM-YYYY), `flugjahr`, `aenderung` per tile, 3–4 s per query |
| Zip ETag / Last-Modified, REST `creationDate` | every zip | for change detection (Frankfurt DOP zip 02.11.2026, DGM1/DOM1 05.08.2026) |
| Static file headers | xlsx, PDF | `Last-Modified` changes per request (19:32 vs 19:55 for the same xlsx), no ETag. **Not usable.** |
| ALKIS `aktualit` | per feature | `YYYY-MM-DDZ`. Samples: buildings mostly 2009 (12,832 of 18,848, the migration), then spread to 2026; land-use parcels spread 2009–2026 (720 from 2026). |

Suggested mapping:
- orthophoto `currency` = `Bildflugdatum` of the tile(s) under the map (xlsx or GetFeatureInfo); `published` = zip ETag
  (download path) or the xlsx release date in row 1 (WMS path);
- DGM1/DOM1 `currency` = `Bildflugdatum` from the DGM/DOM xlsx (the same for both).

---

## 8. ALKIS: buildings and actual land use

### 8.1 Service

- `https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs`, title "WFS HE ALKIS Vereinfacht", WFS 2.0.0.
  - Types `ave:Flurstueck`, `ave:FlurstueckPunkt`, **`ave:GebaeudeBauwerk`**, `ave:KatasterBezirk`, `ave:Nutzung`,
    **`ave:NutzungFlurstueck`**, `ave:VerwaltungsEinheit`. DefaultCRS 25832; OtherCRS 25833, 4258, 4326.
  - Output: GML 3.1/3.2 (`application/gml+xml; version=3.2` is the default answer) and `application/x-zip-shapefile`.
    **No GeoJSON** (`OUTPUTFORMAT=application/json` → 400).
  - Constraints: `ImplementsResultPaging FALSE`, `CountDefault 100000`, `ImplementsSorting` not listed.
  - Elements use the default namespace (`<GebaeudeBauwerk gml:id="…"><oid>…`), as at GeoSN.
- Freemapper's `alkis_wfs` request shape works unchanged:
  ```
  https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=476359,5552444,476759,5552744,urn:ogc:def:crs:EPSG::25832&SORTBY=oid&COUNT=10000&STARTINDEX=0
  ```
  → 200, 281 features, 3.5 s. `RESULTTYPE=hits` → `numberMatched="281"` (2.1 s). The results answer says
  `numberMatched="unknown"` (as at GeoSN), so the separate hits request is needed.
- Speed (gzip works, `Content-Encoding: gzip`): 2 × 2 km around the school: 9,599 buildings, 9.7 MB GML (1.0 MB gzip),
  **1.9 s**; 4,756 land-use parcels 6.3 MB, 5.1 s. Without COUNT the same 9,599 came back (15 s). Frankfurt centre
  1.5 × 1.5 km: 4,982 buildings 9.4 s.
- Paging: `COUNT=1000&SORTBY=oid` with `STARTINDEX=0/1000/9000` gave 1000/1000/599 disjoint features (the oid order is
  case-insensitive: `…bjyv…` before `…bjYV…`). Without `SORTBY` two identical requests returned the same order.
- 25833: `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833` → 320 features, 0.5 s, coordinates in E N order, within 0.65 mm of
  my own transform of the 25832 answer.
- Errors: **400 `text/xml` ExceptionReport** with a readable reason (unknown type, unsupported SRS `EPSG::3035`, unknown
  output format).
- Outside Hesse (Mainz, 400 × 300 m): `numberMatched="0"`.
- Geometries are `MultiSurface` of `Polygon`s with `posList`, srsDimension 2. Features that cross the bbox are returned
  whole.

### 8.2 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid`, `aktualit`, `gebnutzbez`, `funktion`, `gmdschl` (always), `gfkzshh` (17,236 of 18,848),
  `lagebeztxt`?, `anzahlgs`?, `rellage`?, `name`?. **The LGB names and spellings**: `gebnutzbez` "Gebäude".
- `NutzungFlurstueck`: `oid` (34 characters = Nutzung oid + Flurstück oid, e.g. `DEHE06200001eASQDEHE06200001eA8zTF`),
  `aktualit`, `nutzart`, `bez`?, `name`?, `flstkennz`.
- **`Nutzung` has no `nutzart`** in Hesse: only `oid`, `aktualit`, `name`?, `bez`? (checked on the raw GML of 84 features).
  Freemapper's `LVERMGEO` configuration (`land_use_type: ave:Nutzung`) must **not** be copied; use
  `ave:NutzungFlurstueck`, like `GEOSN`.
- oids start with **`DEHE`** (`DEHE0620…` Frankfurt/Darmstadt, `DEHE0621…` Marburg; `…BL` buildings, `…TF` parcel
  land use, `…TN` land use), so `Region::of_object_id` gets a `"DEHE" => Hesse` arm.

### 8.3 Sample features at the Lessing-Gymnasium (geometry elided)

```json
{"oid":"DEHE06200002dyI4BL","aktualit":"2017-02-02Z","gebnutzbez":"Gebäude","funktion":"Gebäude für öffentliche Zwecke",
 "gfkzshh":"31001_3000","name":"Lessing-Gymnasium;IV","gmdschl":"06412000","lagebeztxt":"Fürstenbergerstraße 166"}
{"oid":"DEHE06200001emllBL","aktualit":"2009-11-10Z","gebnutzbez":"Gebäude","funktion":"Gebäude für öffentliche Zwecke",
 "gfkzshh":"31001_3000","name":"Elisabethenschule;Gymnasium","anzahlgs":"4","gmdschl":"06412000",
 "lagebeztxt":"Eschersheimer Landstraße 96, 102; Vogtstraße 33, 35, 37"}
{"oid":"DEHE06200001eBK5BL","aktualit":"2009-11-10Z","gebnutzbez":"Bauteil","funktion":"Tiefgarage",
 "rellage":"Unter der Erdoberfläche","gmdschl":"06412000"}
{"oid":"DEHE06200001eASQDEHE06200001eA8zTF","aktualit":"2024-02-07Z","nutzart":"Fläche besonderer funktionaler Prägung",
 "bez":"Öffentliche Zwecke","flstkennz":"060471285000080005__"}
{"oid":"DEHE06200001eAKHDEHE06200001eA0xTF","aktualit":"2009-11-10Z","nutzart":"Straßenverkehr","name":"Wolfsgangstraße",
 "flstkennz":"060471273000010001__"}
{"oid":"DEHE06200001NGJUDEHE06200001G0ffTF","aktualit":"2021-04-29Z","nutzart":"Platz","bez":"Parkplatz",
 "name":"Gisèle-Freund-Platz","flstkennz":"060471289000210002__"}
```

The school parcel's `bez` is **"Öffentliche Zwecke"**, not "Bildung und Wissenschaft" (Brandenburg). The school building's
`funktion` is "Gebäude für öffentliche Zwecke", not "Allgemein bildende Schule".

### 8.4 Observed values (school 2 × 2 km + Frankfurt centre + Marburg + Taunus forest: 18,848 buildings, 11,071 parcels)

- **`gebnutzbez`:** `Gebäude` 17,092, `Bauteil` 1,612, `Sonstiges Bauwerk oder sonstige Einrichtung` 137, `Turm` 7. LGB
  spellings, so `buildings::classify` works unchanged.
- **`funktion` is generalised** for ordinary buildings: `Wohngebäude` 13,475 (gfkzshh `31001_1000`), `Gebäude für
  Wirtschaft oder Gewerbe` 1,877 (`_2000`), `Gebäude für öffentliche Zwecke` 1,388 (`_3000`). Only a few are specific:
  Gebäude zum Parken 142, Sonstiges 148, Nach Quellenlage nicht zu spezifizieren 93, Umformer 62, Krankenhaus 18,
  Kirche 16, Gebäude für religiöse Zwecke 15, Parkhaus 5 … Bauteile: Auskragender Geschossteil 650, Durchfahrt im
  Gebäude 325, Tiefgarage 298, Überdachung 137, Arkade 136, Keller 46. Freemapper uses `funktion` only for labels
  (`build/buildings.rs`), so this only makes labels coarser.
- **`rellage`:** `Unter der Erdoberfläche` 412 (Tiefgarage, Keller), `Aufgeständert` 9. The underground test works
  unchanged.
- **`name`** holds the label text split by `;`, sometimes in a broken order: `Lessing-Gymnasium;IV`,
  `schule;Liebfrauen-`, `Parkh.;Taubenstr.;Parkhaus;str.;Tauben-`, `Alten-;heim`. One value is double-encoded UTF-8
  (`BÃ¼rgerhospital` next to `Bürgerhospital`). Do not use `name` for map text without cleaning.
- **`nutzart`** (LGB spellings throughout): Wohnbaufläche 5,116, Fläche gemischter Nutzung 1,629, Straßenverkehr 1,445,
  Industrie- und Gewerbefläche 1,074, Fläche besonderer funktionaler Prägung 611, Landwirtschaft 288, Weg 248, Sport-,
  Freizeit- und Erholungsfläche 244, Wald 148, Platz 92, Fließgewässer 65, Bahnverkehr 51, Unland/Vegetationslose Fläche
  38, Gehölz 8, Stehendes Gewässer 7, Friedhof 6, Halde 1. `PRIVATE_LAND_USES` matches as in Brandenburg.
- **`bez` per `nutzart`:**
  - `Straßenverkehr`: none 1,425, `Fußgängerzone` 20. **Never `Fahrbahn` or `Begleitfläche Straßenverkehr`**, so the
    `VERGE` rule never fires and every street parcel (with its green verges and tree rows) becomes paved, as in Saxony.
  - `Platz`: `Parkplatz` 69, none 19, `Fußgängerzone` 3, `Marktplatz` 1. Squares without `bez` need the
    `square_bez` treatment (candidate instead of paved), as for GeoSN/LVermGeo.
  - `Weg`: none 243, `Fußweg` 5.
  - FBFP: `Öffentliche Zwecke` 390, none 199, Historische Anlage 7, Gesundheit, Kur 6, Bildung und Wissenschaft 4, Soziales 2, Kultur 2,
    Religiöse Einrichtung 1.
  - IuG: Handel und Dienstleistung 800, Industrie und Gewerbe 210, Versorgungsanlage 22, Lagerfläche 22, Umspannstation 13 …
  - SFE: Park 104, Grünanlage 96, Sportanlage 29, Kleingarten 7, Freizeitanlage 4 …
  - Landwirtschaft: Gartenbauland 151, Grünland 72, Ackerland 65. Wald: Laub- und Nadelholz 60, Laubholz 58, Nadelholz 10,
    none 20. Fließgewässer: Fluss 27, Graben 15, Bach 12.
  - `Wohnbaufläche`, `Fläche gemischter Nutzung`, `Bahnverkehr`: no `bez`.

### 8.5 What is missing or different

- No GeoJSON and no working OGC API; GML only (the existing `alkis_wfs` GML reader fits).
- `ave:Nutzung` without `nutzart`.
- No verge/carriageway split in `Straßenverkehr`.
- Generalised `funktion`; messy `name`.
- The INSPIRE BU/ELU services, LoD2 (`dl-by-de/2.0` in CKAN!) and Hausumringe were not examined (not needed).

---

## 9. Licence and attribution

- Licence: **Datenlizenz Deutschland – Zero – Version 2.0** (`dl-zero-de/2.0`, <https://www.govdata.de/dl-de/zero-2-0>) on
  every product used (ISO metadata `otherConstraints`, CKAN `license_id`, WCS/INSPIRE WMS `Fees`).
- The legal text, verbatim from the WMS/WFS capabilities (`AccessConstraints`), § 18 HVGG:
  > "Jede Nutzung der Geobasisdaten und zugehörigen Metadaten ist ohne Einschränkung oder Bedingung erlaubt. Die
  > bereitgestellten Geobasisdaten und Metadaten dürfen für die kommerzielle und nicht kommerzielle Nutzung insbesondere
  > 1. vervielfältigt, ausgedruckt, präsentiert, verändert, bearbeitet sowie an Dritte übermittelt werden, 2. mit eigenen
  > Daten und Daten anderer zusammengeführt und zu selbstständigen neuen Datensätzen verbunden werden, 3. in interne und
  > externe Geschäftsprozesse, Produkte und Anwendungen in öffentlichen und nicht öffentlichen elektronischen Netzwerken
  > eingebunden werden. **Wird bei der Nutzung der Geobasisdaten oder deren Metadaten ein Quellenvermerk beigegeben, ist in
  > diesem auf Veränderungen, Bearbeitungen, neue Gestaltungen oder sonstige Abwandlungen der Daten oder Metadaten
  > hinzuweisen.** (§ 18 Hessisches Vermessungs- und Geoinformationsgesetz – HVGG)."

  `Fees`: "Der automatisierte Abruf und die Nutzung der Geobasisdaten und der zugehörigen Metadaten sind kostenfrei. (§ 24
  HVGG)". The Geodaten-online terms say the same ("Damit ist jede Nutzung dieser Daten ohne Einschränkung oder Bedingung
  erlaubt.") and add that products based on third-party data (e.g. BKG) carry their own source notes.
- Source wording used by HVBG itself (WMS `Abstract`): **"Geobasisdaten © Hessische Verwaltung für Bodenmanagement und
  Geoinformation"**. No recommended Quellenvermerk was found on hvbg.hessen.de/open-data, the product pages or in the ISO
  metadata.
- **Recommended freemapper line** (voluntary, but freemapper names every source):
  `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) <year>, dl-de/zero-2-0, verändert`.
  Because a source note is given, the change note ("verändert") is **mandatory** under § 18 HVGG.
- Exceptions: none for the products used. Note that the CKAN package `3d-gebaudemodell-he-lod2` says `dl-by-de/2.0`
  (not used).

---

## 10. State boundary (for `assets/regions/hessen.geojson`)

- **Polygon source (recommended):** the ALKIS WFS `ave:VerwaltungsEinheit`, `art = Bundesland`, oid **`DE06`**, `name
  Hessen`, `aktualit 2026-11-06Z`. Fields: `oid`, `aktualit`, `art`, `name`, `ags`, `uebobjekt`, `ueboname`. Fetched by a
  tiny bbox around a point inside Hesse, which returns the Gemeinde, Kreis, Regierungsbezirk and Land:
  ```
  https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=47830,5571730,47840,5571740,urn:ogc:def:crs:EPSG::25833
  ```
  → 200, 1.0 MB gzip, **1.2 s** (the 25832 equivalent took 9.4 s cold). **`RESOURCEID=DE06` hung for more than 10
  minutes** without an answer; avoid it. (The `FILTER` on `art`, as used for Saxony-Anhalt, was not tried.)
- Geometry: MultiSurface with **2 polygons**, 38,134 vertices:
  - main polygon 21,191.7 km² (25833 area) with 27 holes, **23 of them degenerate** (< 11 m², zero-width slivers);
    the real ones are 0.0205 km² (49.609°N 8.680°E), 0.0475 km² (51.561°N 9.607°E), 0.0014 km² (50.436°N 8.297°E) and one
    more of 1,415 m²;
  - an **exclave of 0.62 km²** at 49.617°N 8.671°E (on the Baden-Württemberg border near Heppenheim/Weinheim) with a
    0.022 km² hole.
- **25 m Douglas–Peucker simplification:** `probe/boundary/hessen_simplified25m.geojson`. FeatureCollection, `crs`
  `urn:ogc:def:crs:EPSG::25833`, properties `name: Hessen`, `source: HVBG ALKIS vereinfacht WFS (VerwaltungsEinheit DE06),
  dl-de/zero-2-0`, `retrieved: 2026-11-07`, `simplified_m: 25`; MultiPolygon with integer coordinates.
  - **5,144 vertices, 84.7 KB** (Brandenburg asset 82.9 KB). Main polygon 5,094 + holes 6/5/7, exclave 26 + hole 6.
  - The 23 degenerate holes were dropped (fewer than 4 points or < 1 m² after simplification).
  - Extent E −17106 … 167166, N 5489626 … 5737354: **negative eastings** in the asset (no other asset has them).
- Alternative: the statewide shapefile `https://gds-srv.hessen.de/atomfeed/DigVGr-epsg25832-shp.zip` (67.7 MB, 25832, not
  opened).
- Neighbours are North Rhine-Westphalia, Lower Saxony, Thuringia, Bavaria, Baden-Württemberg and Rhineland-Palatinate;
  none is a freemapper region yet, so there is no overlap with the existing assets.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/hessen-latest.osm.pbf` → **307** → `hessen-261106.osm.pbf` → 200,
  **347,632,276 B** (index page: 331 MB), `Last-Modified: Sat, 07 Nov 2026 02:26:54 GMT`, `ETag "14b87294-65fa67654d7c5"`,
  `Accept-Ranges: bytes`.
- State: `https://download.geofabrik.de/europe/germany/hessen-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4954
  ```
- Poly: `https://download.geofabrik.de/europe/germany/hessen.poly`, one ring, 294 points, lon 7.768–10.246, lat 49.393–51.659.
- Overpass (overpass-api.de) answered **406** to a UA starting with `Mozilla/5.0`; with
  `freemapper-research/0.1 (github.com/HenryJobst/freemapper)` and `Accept: application/json` it answered 200.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `gds.hessen.de` REST | undocumented JSON API (`/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter?path=`); `pageSize` 20, `&page=N`; empty `path` → 400; fast (0.1–0.2 s); sets session cookies (not needed) |
| `gds.hessen.de/downloadcenter/<date>/…` | **date segment valid ≈ 2 days**; **HEAD → 405**, use GET with `Range: bytes=0-0` for the size; ranges, ETag, Last-Modified; spaces, umlauts and parentheses in paths (percent-encode); irregular file names; one empty range answer seen |
| `gds.hessen.de/INTERSHOP/static/…` | `Last-Modified` regenerated per request; wrong `HLBG-Site/Geodaten` path in one product text → 403 |
| `www.gds-srv.hessen.de` WMS | **Max 3000 px**, oversize → **HTTP 200 `text/xml`** exception; **10–34 s per piece, two hangs (180 s, 300 s) in ≈ 15 GetMaps**; JPEG q75; white outside the coverage; no caching headers |
| `inspire-hessen.de` WCS | 2.0.1 `SUBSET=E(…)&SUBSET=N(…)`; uncompressed TIFF; **≈ 0.25 MB/s** (2 km² 63 s); 25833 output resampled; DOM coverage ≠ download DOM |
| `inspire-hessen.de` WMS | max 4000 px (4001 → HTTP 200 XML), 25833 → HTTP 200 XML exception, 65 s for 4000²; **DOP20 is an older vintage** (§4.3) |
| `www.gds.hessen.de` WFS | GML only; `numberMatched="unknown"` on results (use `RESULTTYPE=hits`); paging works with SORTBY although `ImplementsResultPaging FALSE`; proper 400 errors; gzip; 0.5–15 s; `RESOURCEID` query hung > 10 min |
| `geoportal.hessen.de` | OGC API proxies broken (`WFS with id does not exists!`) |

- No authentication anywhere, no rate-limit or `Retry-After` headers, no throttling seen in roughly 180 requests. No
  fair-use rules found on the pages read (not exhaustive).

---

## 13. What makes the Hesse adapter differ (risks first)

1. **The orthophoto WMS is slow, sometimes hangs, and caps at 3000 px.**
   - freemapper's `WMS_MAX_PX` (4000) is a constant and must become per-service (≤ 3000 here), or the oversize request
     fails with an HTTP 200 XML body.
   - 15–34 s per 3000 px piece, so 1.5–2 min per km²; two of ≈ 15 requests hung (180 s, 300 s). Needs a per-request
     timeout and retries.
   - The INSPIRE DOP20 WMS (4000 px) is no alternative: it serves an older, non-true orthophoto.
   - The **tile download is the better path**: the JPEG tiles are *stored* in the zip, so a tile is one ranged GET of ≈4 MB
     in a few seconds, the pixels are the WMS's pixels, and the georeferencing (JGW) is the LGB convention.
2. **Downloads are per municipality behind an undocumented REST API with expiring URLs.**
   - Tile → municipality: the metadata xlsx (`gmde_bz`, `kreis_bz`; a tile can be in several). Municipality → file: the
     REST listing of its district (names do not match the xlsx exactly: "Main-Taunus" vs "Main-Taunus-Kreis"/"_Kreis",
     "Landeshauptstadt Wiesbaden" vs "Kreisfreie Stadt Wiesbaden", "Neustadt (Hessen)" vs `Neustadt_(Hessen)-DOP20.zip`).
     Compare on a normalised key, or index all 27 district listings (≈ 30 REST calls per product) and the zips' central
     directories (one 70 KB range per zip, cached per ETag).
   - The `/downloadcenter/<YYYYMMDD>/` URL is only valid for about two days; resolve it before every download.
   - HEAD is refused (405); member names vary in case (`DGM1_` vs `dgm1_`).
   - **Decision needed:** this index (more code, fast and exact) vs the WCS for DGM1 (simple, identical data, slow) vs the
     WMS for the DOP (simple, slow).
3. **CRS:** everything is 25832; in 25833 Hesse has negative eastings (E −17 … 167 km), −4.9° convergence and +2–3 m/km
   scale error. Region CRS must be `Utm32`. `TileId` is `u32` and `tiles_covering_grid` clamps negative values to 0, so no
   25833 tile id may ever be computed for Hesse; the region asset itself contains negative coordinates.
4. **ALKIS land use must come from `ave:NutzungFlurstueck`** (`ave:Nutzung` has no `nutzart`). `Straßenverkehr` never has a
   verge/carriageway `bez`, so street parcels become paved including verges (as in Saxony); squares without `bez` need
   the `square_bez` candidate rule. Spellings are LGB's, so no normalisation (unlike Hamburg).
5. **DOP is leaf-on (summer) and 1–3 years old by region** (2023 south-west, 2024 north, 2025 south). The ALS (DGM1/DOM1)
   is 2019–2025 and need not match the DOP year (school: ALS Feb 2023, DOP Aug 2025). Flag the age difference.
6. **Two DOM1 versions:** the WCS `dom1` differs from the download (vegetation, mean 0.8 m). Use the download.
7. GML only (no GeoJSON); `numberMatched` only via hits; `name` labels are split and partly mis-encoded.
8. TIFF details: LZW with **Predictor 2 on Float32** (the `tiff` crate supports it; not yet run in freemapper); IFD at the
   end; older builds without GeoKeys; tiepoints exact (no float noise), JGW with float noise.
9. Unchanged from the other states: DHHN2016, DGM1 pixel registration (PixelIsArea at the NW corner, TFW centres at +0.5),
   Float32, NoData −9999, no login, 1 km tiles, Geofabrik layout (`hessen-latest`, `hessen-updates/state.txt`). The
   licence is simpler than everywhere else (dl-de/zero-2-0, no attribution duty).

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Hesse | Notes |
|---|---|---|
| **Orthophoto template** | **DOP20 tiles from the Downloadcenter by HTTP range** (zip per municipality; JPEG stored, ≈ 4 MB per km²; JGW). Fallback/simple path: **WMS** `https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows`, layer `he_dop20_rgb`, `CRS=EPSG:25832`, 0.2 m, **≤ 3000 px pieces**, timeout + retry. | Currency: `Bildflugdatum` from `Metadaten_DOP.xlsx` (Excel serial) or GetFeatureInfo `he_dop_rgb_info` (`bildflug`). Not qTrueDOP (older, unknown date). |
| **DTM** (hillshade, slope, contours) | **DGM1 tiles** from the same Downloadcenter index (deflate members, ≈ 2 MB), or the **WCS** `https://inspire-hessen.de/raster/dgm1/ows` coverage `he_dgm1` (identical values, 25832 only, slow: fetch per km tile) | NoData −9999; tiepoints exact; date from `Metadaten_DGM-DOM_*.xlsx`. |
| **nDSM template** | **DOM1 − DGM1** at 1 m from the download tiles (ALS, same flight as the DGM) | Not the WCS `dom1`. Clamp < 0. Flag the ALS date against the DOP date. |
| **Buildings** | **WFS** `https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs`, `ave:GebaeudeBauwerk`, 25832, `SORTBY=oid&COUNT=10000`, hits first | Reuse `alkis_wfs` (GML); LGB spellings; underground via `rellage`. |
| **Land use** (out-of-bounds, paved) | the same WFS, **`ave:NutzungFlurstueck`** | `square_bez` e.g. "ohne Unterart (Hessen)"; no verge `bez`. |
| **Region boundary** | WFS `ave:VerwaltungsEinheit` oid DE06 in 25833 (small-bbox query), simplified 25 m | Ready in `probe/boundary/hessen_simplified25m.geojson` (5,144 vertices, 2 polygons, 4 holes). Region CRS Utm32; oid prefix `DEHE`. |
| **OSM** | Geofabrik `europe/germany/hessen-latest.osm.pbf` + `hessen-updates/state.txt` | 348 MB; follow the 307 to `hessen-YYMMDD.osm.pbf`. |
| **Attribution** | `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) <year>, dl-de/zero-2-0, verändert` + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9; "verändert" is required once a source note is given. |

---

## 15. Not verified, or open

- Which of the two DOM1 versions (WCS vs download) is newer, and why they differ.
- The qTrueDOP date at the school (its metadata layer returned nothing).
- Whether the `/downloadcenter/<date>/` window is exactly "today and tomorrow" in Europe/Berlin time, and when it rolls
  over (tested at one moment only).
- That freemapper's `tiff` decoder reads the LZW + Predictor 2 Float32 tiles (checked in the crate source, not run); the
  older builds without GeoKeys.
- The largest municipal zips (zip64?) and member layout outside Frankfurt and Wiesbaden; whether all DOP zips store the
  JPEGs uncompressed.
- The `FILTER` (art = Bundesland) query for the boundary, and why `RESOURCEID=DE06` hangs.
- `ave:VerwaltungsEinheit`'s `aktualit` (2026-11-06Z) suggests daily updates of ALKIS; the WFS update cycle was not checked.
- `nutzart`/`bez` values of rarer classes (Heide, Moor, Sumpf, Flugverkehr, Hafen, Tagebau …) and of other regions; only
  four sample areas were read.
- WMS behaviour under sustained load (only ≈ 15 GetMaps were made); the DGM hillshade layers in `ogc-free-maps.ows`.
- LiDAR point clouds via the shop (terms, price, login), LoD2, Hausumringe, INSPIRE BU/ELU.
- The Mainz-side imagery in the WMS border tiles (HVBG flights, `land = 'he'`): its licence is presumably the same
  dl-de/zero-2-0, not checked.

Network note: total transfer was roughly **110 MB**: DOP/DGM/DOM members (≈ 17 MB), WMS probes (≈ 15 MB), the WCS 2 km
clip (16 MB) and an aborted 6 km clip (35 MB, deleted), ALKIS samples (≈ 4 MB gzip), the metadata xlsx and overview PDFs
(≈ 24 MB). No full zip was downloaded. Disk usage under `hessen/` is about 175 MB.
