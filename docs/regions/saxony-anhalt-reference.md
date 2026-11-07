# Saxony-Anhalt (LVermGeo ST) data sources for freemapper — live-verified reference

Verified live on 2026-11-07 (Europe/Berlin). Provider: **Landesamt für Vermessung und Geoinformation Sachsen-Anhalt (LVermGeo)**.
Portal: <https://www.lvermgeo.sachsen-anhalt.de/de/gdp-open-data.html> ("Open Data", part of the Geodatenportal
<https://geodatenportal.sachsen-anhalt.de>). Downloads need no login, no registration and no form. The only gate is a
confirmation checkbox, and it is switched off in the download pages' JavaScript.

Test locations:

| School | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 2 km tile |
|---|---|---|---|---|---|---|
| **Latina August Hermann Francke, Halle (Saale)** (primary) | way **1098639307** | E 706451 / N 5706913 | E 289795 / N 5707067 | 51.47565 / 11.97297 | E 706388–706514, N 5706790–5707036 | `32 706 5706` (fully inside) |
| Domgymnasium Magdeburg (currency checks + ALKIS only) | way **31109469** | E 680185 / N 5777944 | E 269457 / N 5780028 | 52.12262 / 11.63210 | E 680137–680233, N 5777893–5777996 | `32 680 5776` (fully inside) |

(25833 values come from my own Krüger-series UTM code, which round-trips to 1e-7°. ALKIS returned the Latina building at
E 289931 / N 5706902 etc. in 25833, which is consistent.)

Samples (all under `sachsen-anhalt/`; treat as untrusted data):

| Path | Content |
|---|---|
| `dl/dgm1/LVermGeo_DGM1_574055.zip` (+ `x/`) | full DGM1 tile 706/5706 from the map downloader (`.meta`, `.tif`, terms-of-use PDF) |
| `dl/dom1/LVermGeo_DOM1_738765.zip` (+ `x/`) | full DOM1 tile 706/5706 |
| `dl/dop20/head_2MB.bin`, `tif_head.bin`, `meta_*.bin` | first 2 MB of the map-downloader DOP20 zip (meta, TFW, TIFF start), and the meta of two more tiles |
| `dl/bdom20/head.bin` | first 1.5 MB of the bDOM20 zip (the LAS 1.4 header) |
| `dl/atom/` | ATOM DGM1 GeoTIFF (full, 8.9 MB); ATOM DOP20 header, IFD tail, and two internal 512 px tiles (as `.raw`/`.png`) |
| `dl/wms/dop_school_25833.jpg` | WMS GetMap 600 × 500 m at 0.2 m in **EPSG:25833** (visually checked: Franckesche Stiftungen with the sports field) |
| `dl/wcs/` | WCS DGM1/DOM1 clips (multipart responses, extracted `.x.tif`) in 25832 and 25833 |
| `dl/laz/halle_706_5706_head.bin` | LAS header of the Halle LiDAR tile, read from a 17 GB zip by HTTP range |
| `probe/gml_latina_*.xml`, `city_*.xml`, `mdb_*.xml` | ALKIS WFS answers (school bbox in 25833; Halle centre 2 × 2 km and Magdeburg centre 1.4 × 1.1 km in 25832) |
| `probe/tiles_gdp-*.json` | the map downloader's tile GeoJSON (label → internal id) per product |
| `probe/gb_*.json`, `gb_truedop_all.csv` | tile currency WFS answers; all 5465 TrueDOP tiles with flight date |
| `probe/dvg_land_25833.xml`, `probe/sachsen-anhalt_simplified25m.geojson` | state polygon (full) and a 25 m simplified version in the `assets/regions/*.geojson` format |
| `probe/zip_halle_laz.tsv` | member list of the Halle LAZ zip (read from its central directory) |
| `probe/nutzungsbedingungen.txt` | text of the terms of use (v5.0, 01.03.2024) |

---

## 1. Brandenburg → Saxony-Anhalt at a glance

| freemapper product | Brandenburg (today) | Saxony-Anhalt equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** WMS, WCS and WFS reproject to 25833 on request. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **TrueDOP20 RGB** as an ATOM GeoTIFF per 2 km tile (300 MB, uncompressed, **512 px internal tiles, HTTP ranges work**, so the needed part can be read without the whole file); or **WMS** (also in 25833); the map downloader zip is RGBI, 320 MB, and older | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1** GeoTIFF f32 LZW, 2 km tile, ≈9 MB; **no TFW** (GeoTIFF tags only). Map downloader or **WCS** (clip, optional 25833 output, nearest neighbour). The ATOM copy is stale. | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **bDOM20 LAZ** (0.2 m grid as points, absolute heights, 257 MB per 2 km tile) minus DGM1; or **DOM1 − DGM1** at 1 m (WCS or tile) | verified (header only for bDOM) |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `ave:GebaeudeBauwerk`, **GML 3.2 only**, includes `rellage` and `Bauteil` | verified |
| Land use | OGC API `nutzung_flurstueck` | WFS `ave:Nutzung` only (**no `NutzungFlurstueck`**). `nutzart` + `bez`; `bez` is missing on Platz and Straßenverkehr. | verified |
| Currency | `bb_*_aktualitaet.csv` | ArcGIS WFS `truedop_kacheln` (flight date per tile), `bdom_aktualitaet_zyklus` (DOM); `.meta` file inside every map downloader zip; ETag/Last-Modified on ATOM files | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `sachsen-anhalt-latest.osm.pbf` (176.9 MB) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | dl-de/by-2-0, provider **`© GeoBasis-DE / LVermGeo ST`** | verified |

---

## 2. Infrastructure: three download paths, tile scheme

LVermGeo has no single tile store. It has three paths, and they do **not** serve the same versions:

| Path | Host | Products | Ranges | Versions (Latina tile) |
|---|---|---|---|---|
| **Map downloader** (portal "zur Auswahl" pages) | `www.lvermgeo.sachsen-anhalt.de/de/mod/<module>/ajax/1/…` | DOP20 (RGBI), historic DOP20, DGM1, DOM1, bDOM20, DOP100, DTK, LoD … | **no** (`Range` ignored, 200 full body) | DOP **2024-05-13**; DGM1 current (revised 2024-05-13); DOM1 2024-05 |
| **INSPIRE ATOM** static files | `www.geodatenportal.sachsen-anhalt.de/gfds_webshare/sec-download/LVermGeo/…` | DOP20 RGB, DGM1 | **yes** (206, ETag, Last-Modified, 304) | DOP = **newest TrueDOP (2026 flight)**; DGM1 **older** (file from 2024-01-09) |
| **OGC services** (WMS/WCS/WFS) | `www.geodatenportal.sachsen-anhalt.de/…` and `geodatenportal.sachsen-anhalt.de/…` | DOP WMS, DGM1/DOM1 WCS, ALKIS/DVG WFS, tile index WFS | n/a | WMS DOP = the ATOM file pixel for pixel; WCS DGM1/DOM1 = the map downloader values |

### 2.1 Tile scheme

- **2 km × 2 km**, EPSG:25832, SW corners at **even km** (all 5466–5469 labels checked: 0 odd E, 0 odd N).
- Label `32EEENNNN` (zone + E km, 3 digits + N km, 4 digits), e.g. `327065706` = E 706000–708000, N 5706000–5708000.
- File names:
  - map downloader: `dgm1_32_706_5706_2_st_2024.tif`, `dom1_32_706_5706_2_st_2024.tif`, `dop20rgbi_32_706_5706_2_st_2024.tif`, `bdom20rgbi_32_706_5706_2_st_2024.laz`. The suffix `_2_st_<year>` carries a year, so member names change on update; read whatever `.tif`/`.laz` the zip holds.
  - ATOM: `DOP20/327065706.tif`, `DGM1/DGM_327065706_TIFF.tif`.
- Grid extent (SW corners E 606…788 km, N 5646…5880 km): **E 606000–790000, N 5646000–5882000 (25832)**.
  This matches the WMS `BoundingBox` 25832 (540601–855399 is the padded service box, not the grid).
- Tile counts: DGM1 5469, DOP20/DOM1/bDOM20 5466, ATOM OI 5473 entries, ATOM EL 5451 entries, TrueDOP index 5465.

### 2.2 Map downloader (portal tile download)

Each "zur Auswahl" page (e.g. <https://www.lvermgeo.sachsen-anhalt.de/de/gdp-dgm1.html>, 2.9 MB of HTML, about 15 s) embeds
`new gc.mod.MapDownloadSelector('mapdownloader_content', '<GeoJSON>', 'EPSG:4647', ['zip','rar','tar.gz','tar.bz2'], '0', '', '', '', '<ajax_url>', '', …, 5, <dataCycles>)`.
The GeoJSON holds every tile as a polygon in EPSG:4647 (zone-prefixed 25832) with `properties: {id, label}`. The **`id` is an
internal number that differs per product**:

| Product | Page | `ajax_url` module | id range | Latina id | Domgymnasium id |
|---|---|---|---|---|---|
| DGM1 | `gdp-dgm1.html` | `2,2913,501` | 571553–577021 | **574055** | 575558 |
| DOP20 (current) | `gdp-dop20-auswahl.html` | `4,1962,501` | 727015–732480 | **732016** | 730705 |
| DOP20 historic | `gdp-dop20-auswahl-historie.html` | `4,1963,501` (+ `dataCycles` `2018-2019`, `2016-2017`, `2014-2015`) | same as DOP20 | 732016 | 730705 |
| DOM1 | `gdp-dom1.html` | `3,2926,501` | 737947–743412 | **738765** | 741280 |
| bDOM20 | `gdp-bdom20.html` | `4,1974,501` | same as DOM1 | **738765** | 741280 |

Protocol (from `res/modules/mapdownloader/res/index.js`, verified):

```
GET https://www.lvermgeo.sachsen-anhalt.de/de/mod/2,2913,501/ajax/1/prepare/?items=574055&format=zip
→ 200 text/plain, body = https://www.lvermgeo.sachsen-anhalt.de/de/mod/2,2913,501/ajax/1/download/?file=mapdownloader-1143c0df351a606e99721cdd0becffc6.zip
GET <that URL>
→ 200 application/octet-stream, Content-Disposition: attachment; filename=LVermGeo_DGM1.zip, 9,400,400 B (8.2 s, ≈1.15 MB/s)
```

Behaviour:
- `prepare` takes 0.3–1.9 s. Every call creates a **new random file name**, even for the same items.
- `items=a,b,…` builds one archive. The 5-tile limit is client-side only: 6 items were accepted (48 MB).
- `format` can be `zip`, `rar`, `tar.gz` or `tar.bz2`.
- Unknown item: **500** with the body `Daten nicht verf&uuml;gbar.` Unknown file name: **500** HTML.
- **`Range` is ignored**: the server answers 200 with the full body.
- **`HEAD`** works (Content-Length, Content-Disposition), but took 0.2 s right after `prepare` and **39–45 s** later in the session, for old and new links alike. Do not rely on it; GET directly.
- A link was still valid after 80 min. The real lifetime is unknown.
- The archive is assembled on demand: zip members carry the request time. Every zip also contains `3567_nutzungsbedingungenv5.0_b.pdf` (550 KB).
- The server sets a session cookie; it is not needed.
- **ids are not documented.** They come only from the HTML page. Cache the label → id map per product, and re-scrape on a 500 "Daten nicht verfügbar".

### 2.3 ATOM static files

- Service feeds:
  - <https://geodatenportal.sachsen-anhalt.de/arcgisinspire/rest/directories/web/INSPIRE_ALKIS/ALKIS_OI_DOP20_MapServer/serviceoi.xml>
  - `…/ALKIS_EL_MapServer/serviceel.xml`
- Dataset feeds `datasetoi.xml` (3.4 MB) and `datasetel.xml` (3.7 MB) have one entry per tile. Each entry links a coverage GML, e.g. `…/ALKIS_OI_DOP20_MapServer/OrthoimageCoverage_327065706.tif.gml`, whose `<fileReference>` is the file:
  ```
  https://www.geodatenportal.sachsen-anhalt.de/gfds_webshare/sec-download/LVermGeo/DOP20/327065706.tif
  https://www.geodatenportal.sachsen-anhalt.de/gfds_webshare/sec-download/LVermGeo/DGM1/DGM_327065706_TIFF.tif
  ```
  The pattern is fixed, so URLs can be built directly from the label.
- HTTP:
  - `HEAD` → 200 with `Content-Length`, `ETag`, `Last-Modified`, `Accept-Ranges: bytes`.
  - `Range: bytes=0-0` → **206, 1 byte** (correct, unlike GeoCloud).
  - `If-None-Match` / `If-Modified-Since` → **304**.
  - Missing tile → 404 HTML. Directory listing → 404 (the parent gives 401).
- Feed `<updated>` dates are stale (OI: all 2020, EL: all 2024). Use the file's `Last-Modified`/`ETag` instead.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832** (UTM 32), with no exception found:
  - GeoKeys `ProjectedCSType 25832` in DGM1, DOM1, DOP20 (both sources) and the WCS;
  - LAS GeoKey/WKT 25832 in bDOM20 and the LiDAR;
  - ALKIS WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`;
  - meta files `Koordinatenreferenzsystem_Lage: ETRS89_UTM32` / `25832`;
  - Basis-DLM, DVG and basemap downloads are "ETRS89/UTM32".
- **Height datum DHHN2016** normal heights: `Koordinatenreferenzsystem_Hoehe: DE_DHHN2016_NH`, `Hoehenanomalie: DE_AdV_GCG2016_QGH` (quasigeoid GCG2016); DOP meta `Koordinatenreferenzsystem_Hoehe: 7837`. The TIFFs carry **no vertical GeoKey**. This is the same height system as Brandenburg and Saxony.
- 25833 is available **only through services**, as server-side reprojection:
  - **DOP WMS**: CRS list includes 25833 (and 25832, 4647, 3857, 4326, 31467–9, 2166–8, 2397–9). Verified GetMap in 25833.
  - **WCS DGM1/DOM1**: `SUBSETTINGCRS` + `OUTPUTCRS=http://www.opengis.net/def/crs/EPSG/0/25833` works. Resampling is **nearest neighbour**: 2492 of 2494 sampled pixels equal the nearest source pixel; bilinear would differ by 2 cm on average.
  - **ALKIS WFS** and **DVG WFS**: `OtherCRS` 25833 (also 3034, 3035, 3044, 3045, 3857, 4258, 4326). Verified: `SRSNAME`/`BBOX` in `urn:ogc:def:crs:EPSG::25833` returns 25833 coordinates (E N axis order, 9 decimals).
  - **Tile index WFS** (ArcGIS): `SRSNAME=urn:ogc:def:crs:EPSG::25833` works (verified on `bundeslandgrenze`).
- **Extent** of the state polygon (DVG `DE15`, `aktualit` 2026-11-06, 41,098 vertices, 12 polygons, 7 holes, area ≈ 20,565 km²):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 607140 – 789178, N 5647834 – 5880142 |
  | EPSG:25833 | E **194937** – 374737, N 5646608 – 5882250 |
  | lon / lat | lon 10.5608 – **13.1867**, lat 50.9378 – 53.0418 |

- **The zone line at 12°E cuts the state.** 18,521 of 41,098 boundary vertices lie east of 12°E, 1,298 east of 13°E.
  - Halle (11.97°E) sits on the line; Magdeburg (11.63°E) is west of it; Dessau (12.24°E) and Wittenberg (12.65°E) are east.
  - LVermGeo uses zone 32 statewide, even up to 13.19°E. Saxony does the mirror image: zone 33 statewide, including its part west of 12°E.
- **Consequences for freemapper** (currently 25833 only; `raster.rs` rejects rotated or misaligned grids and has no warp):
  - A 25832 raster is **rotated** against the 25833 grid. Meridian convergence at Halle is about −2.37° in zone 33 and +2.33° in zone 32, so the two grids differ by **≈4.7°**. Rasters cannot be pasted; they must be **warped** (resampled).
  - In the far west (10.56°E, 4.44° from the zone-33 central meridian), the zone-33 scale factor is ≈1.0007 (+0.7 m per km). OOM's combined grid scale factor absorbs this, but it is far outside the intended zone.
  - Vectors (ALKIS, OSM, DVG) are no problem: the WFS reprojects, and OSM is lat/lon anyway. LAS points can be reprojected point by point before gridding.

---

## 4. Orthophoto

### 4.1 Which DOP is current (important)

- **Tile index WFS** `atkis_geobasis_bestand:truedop_kacheln` (§7) for the Latina tile: `NAME 327065706, BFLOS Halle, BFDATUM 29.04.2026`; Domgymnasium tile `326805776`: Magdeburg, 01.05.2026.
  Statewide: 2760 tiles from flights in 2025 (April–June) and 2705 tiles from 2026 (April–June). The state is flown in two halves on a 2-year cycle, and every DOP since the 2019 flight is a **TrueDOP** (rectified on the image-based bDOM).
- **ATOM file** `DOP20/327065706.tif`: `Last-Modified: Tue, 25 Aug 2026`. **WMS GetMap in 25832 is pixel-identical to it** (PNG, mean abs diff 0.0 over a 512² tile).
- **Map downloader** "aktuelle DOP" for the same tile: `dop20rgbi_32_706_5706_2_st_2024`, `Aktualitaet 2024-05-13`, flight `82_24_Halle`. Its pixels **differ** from ATOM/WMS (mean abs diff 31 on 0–255; a visibly different scene).
- Cross-check:

  | Tile | Flight (index) | ATOM `Last-Modified` | Map downloader meta |
  |---|---|---|---|
  | 327245646 (Zeitz) | 2025-06-14 | 2025-11-03 | `_2025`, 2025-06-14 (same flight) |
  | 326145744 (Wernigerode) | 2026-04-22 | 2026-10-08 | `_2024`, 2024-08-28 |

- **Conclusion (inferred, not stated by the provider):** ATOM and WMS carry the newest TrueDOP, including the 2026 flight. The map downloader still serves the previous cycle where 2026 data exists. The release lag is about 4–6 months after the flight.

### 4.2 ATOM TrueDOP20 GeoTIFF — primary candidate

- URL: `https://www.geodatenportal.sachsen-anhalt.de/gfds_webshare/sec-download/LVermGeo/DOP20/32EEENNNN.tif`
- Example: `…/DOP20/327065706.tif` → 200 `image/tiff`, **314,580,224 B**, `ETag "12c01d00-659e03252c857"`, `Last-Modified: Tue, 25 Aug 2026 14:48:32 GMT`.
- TIFF (classic, little-endian):
  - 10000 × 10000 px, **RGB** 8-bit (3 bands, no IR), **Compression 1 (none)**;
  - **tiled, 512 × 512** (20 × 20 = 400 tiles, each 786,432 B, the edge tiles padded);
  - Software "Trimble Germany GmbH", ImageDescription "OrthoVistaProcessing ImageWriter";
  - **IFD at the end** (offset 314,576,516, a tile size + 3,708 B before EOF);
  - ModelPixelScale 0.2, ModelTiepoint (706000.0, 5708000.0), `GTRasterType 1` (PixelIsArea), ProjectedCSType 25832;
  - **no TFW**. Georeferencing comes from the tags or from the label. The equivalent world file would be `0.2/0/0/-0.2/706000.1/5707999.9`.
- **Range access works and is fast**: header 64 KB, IFD tail 80 KB (0.18 s), one internal tile 786 KB (0.76 s). Verified by reading tile 204 (the school) and tile 0 and rendering them.
  A 300 × 300 m schoolyard needs about 4 × 4–5 × 5 internal tiles (**≈13–20 MB**), instead of 300 MB.
  Two ranged requests (head + tail) give the tile offset table.
- Size per 2 km tile: 300 MB (uncompressed). The portal says "ca. 300 MB" for 3-band DOP20.

### 4.3 Map downloader DOP20 (RGBI zip)

- `prepare` on module `4,1962,501`, item 732016 → `LVermGeo_DOP20.zip`, **319,583,153 B**. No ranges, so it is always a full download.
- Members (deflate):

  | Member | Compressed | Uncompressed |
  |---|---|---|
  | `dop20rgbi_32_706_5706_2_st_2024.meta` | 942 | 2,133 (**Latin-1 encoded**) |
  | `….tfw` | 40 | 72; `0.2 / 0 / 0 / -0.2 / 706000.100001 / 5707999.900001`, CRLF |
  | `….tif` | 319,033,258 | **419,434,160** (RGBI, uncompressed, IFD at offset 419,430,408, i.e. at the end) |

  The pixel data starts with a 512 × 512 × 4 block at offset 8, so this file is tiled as well (inferred from the pixel layout; its IFD was not read).
- The meta has useful fields (`Aktualitaet: 2024-05-13`, `Bildflugnummer: 82_24_Halle`, `Kamera_Sensor`, `Bezugsflaeche: bDOM` = TrueDOP, `Belaubungszustand: 3`, `Spektralkanaele: RGBI`). It is the first zip member, so streaming the first ≈3 KB and aborting yields it (verified for 3 tiles).
- Not recommended: it is older where a 2026 flight exists, 4 bands, 320 MB per tile, and has no ranges.

### 4.4 DOP WMS (clipped image, works in 25833)

- `https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest` (WMS 1.3.0).
  - Layers `lsa_lvermgeo_dop20_2` (DOP20) and `lsa_lvermgeo_dop100_unbesch`. The layer is **not queryable**, so there is no GetFeatureInfo date.
  - **MaxWidth/MaxHeight 4096** (819 m at 0.2 m).
  - Formats: jpeg, png, png8, gif, `image/vnd.jpeg-png` …
- Example (school, 600 × 500 m at 0.2 m, **EPSG:25833**):
  ```
  https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=lsa_lvermgeo_dop20_2&STYLES=&CRS=EPSG:25833&BBOX=289500,5706820,290100,5707320&WIDTH=3000&HEIGHT=2500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 3000 × 2500, 5,186,237 B, 5.5 s. Visually checked. Headers: `Cache-Control: no-store`, no ETag.
- In 25832 on the native pixel grid with PNG, the WMS returns the **exact** ATOM pixels. In 25833 the server resamples (method not tested for the WMS).
- Currency is not in the WMS. Take it from `truedop_kacheln` (§7).

---

## 5. DGM1 (terrain model, 1 m)

- Info: <https://www.lvermgeo.sachsen-anhalt.de/de/gdp-dgm-dom-lsa.html>. Downloads: per tile (map downloader), statewide in four parts (`…/Online-Bereitstellung-LVermGeo/DGM/DGM1_{1..4}.zip`, not probed), WMS, WCS, ATOM.
- **Map downloader** (current version). Example `items=574055` → `LVermGeo_DGM1.zip` 9,400,400 B. Members:
  `dgm1_32_706_5706_2_st.meta` (1,531 B, UTF-8), `dgm1_32_706_5706_2_st_2024.tif` (8,860,715 B), and the terms PDF. **No TFW, no aux.xml.**
  - Meta excerpt:
    ```
    Aktualitaet_Kachelinformation:   2025-12-03
    Version_Standard:                3.3
    Kachelname:                      dgm1_32_706_5706_2_st
    Aktualitaet:                     2023-12-18
    Erfassungsmethode:               5022
    Fortfuehrung:                    2024-05-13
    Fortfuehrungsmethode:            5040
    Genauigkeit:                     0.15
    Koordinatenreferenzsystem_Lage:  ETRS89_UTM32
    Koordinatenreferenzsystem_Hoehe: DE_DHHN2016_NH
    Hoehenanomalie:                  DE_AdV_GCG2016_QGH
    Copyright-Vermerk:               ©LVermgeo Sachsen-Anhalt 2025
    ```
  - TIFF:
    - 2000 × 2000, **Float32** (SampleFormat 3), **LZW** (Compression 5, no Predictor tag), strips of 20 rows (100 strips), **IFD at the end** (offset 8,859,358 of 8,860,715);
    - Software "LAStools";
    - `GTRasterType 1` (PixelIsArea), tiepoint (706000.0, 5708000.0), pixel 1 m, `GDAL_NODATA -9999`, ProjectedCSType 25832, no vertical key.
    - The equivalent TFW is `1/0/0/-1/706000.5/5707999.5`, the **same registration convention as LGB/GeoSN DGM1**.
    - Values: 78.86–118.80 m, no NoData pixels, 90.77 m at the school centre. Pillow decodes it.
- **WCS** `https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1` (ArcGIS, WCS 1.0 / 2.0.1, coverage `Coverage1`, formats `image/tiff`, `image/netcdf`; Fees "Datenlizenz Deutschland – Namensnennung, Version 2.0").
  ```
  …/ows_WCS_ST_DGM1?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=Coverage1&FORMAT=image/tiff&SUBSET=x(706300,706600)&SUBSET=y(5706700,5707100)
  ```
  → 200 **`multipart/related; boundary="wcs"`**: a GML part, then the TIFF part.
  - The TIFF is uncompressed Float32 in 128 px tiles.
  - Values are **identical to the map downloader tile** (max diff 0.0).
  - The **25832 output has no GeoKeyDirectory** (only tiepoint and scale). With `OUTPUTCRS=…/25833` the TIFF carries ProjectedCSType 25833.
  - Limit: **4000 px per axis** (4000 m: 200, 33.6 MB, 30 s; 4100 m: 400 HTML "ArcGIS Server Error"). It is uncompressed, so about 4 MB per km².
  - **No per-tile currency.**
- **ATOM** `…/sec-download/LVermGeo/DGM1/DGM_32EEENNNN_TIFF.tif`. Example: 8,879,520 B, `Last-Modified: Tue, 09 Jan 2024`, same TIFF layout.
  It is an **older version**: mean abs diff 0.13 m from the current tile, 95 % of pixels differ by > 1 cm, max 13.3 m. Do not use it.
- Currency: only the `.meta` inside the map downloader zip (`Aktualitaet` = acquisition, `Fortfuehrung` = last revision). There is no DGM tile-index WFS layer.
- Statewide currency: no DGM1 layer exists in the tile index. The LiDAR flight programme PDF `/datei/anzeigen/id/416049,501/Laserscan_Befliegungen_ST.pdf` was not read; the portal says the ALS cycle is six years.

---

## 6. nDSM sources: DOM1, bDOM20, LiDAR

### 6.1 DOM1 (1 m)

- Map downloader `items=738765` (module `3,2926,501`) → `LVermGeo_DOM1.zip` 10,887,950 B. Members: `dom1_32_706_5706_2_st_2024.meta`, `.tif` (10,380,287 B), and the PDF.
- Same TIFF layout as DGM1 (Float32 LZW 2000², PixelIsArea, tiepoint 706000/5708000, NoData −9999, 25832). The grids align pixel for pixel.
- Meta: `Aktualitaet: 2024-05`, `Erfassungsmethode: 5040`, `Genauigkeit Hoehe(m): 0,5`. Tile index: `bdom_aktualitaet_zyklus` → `BFDATUM 13.05.2024`, `Dateiname dom1_32_706_5706_2_st_2024`.
  The date equals the **2024 image flight**, and the accuracy is 0.5 m, so DOM1 is very likely **image-matching based** (resampled bDOM), not LiDAR (inferred).
- WCS `https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1` (same API as DGM1). Its values are identical to the tile (max diff 0.0).
- **DOM1 − DGM1** over the Latina footprint (26,076 px): min −0.86, p10 0.28, median 6.11, p90 17.31, max 31.87 m; 62 % > 2 m. Clamp < 0 to 0. As in Saxony it is 1 m, but image-based: surfaces are smoother, and thin objects and the area under tree crowns are poor.

### 6.2 bDOM20 (0.2 m, LAZ) — the high-resolution option

- Map downloader `items=738765` (module `4,1974,501`) → `LVermGeo_bDOM20.zip`, **257,358,190 B**. The first member is `bdom20rgbi_32_706_5706_2_st_2024.laz` (256,809,673 compressed, 257,379,297 B uncompressed). **No ranges.**
- LAS header, read by streaming 1.5 MB and inflating:
  - **LAS 1.4, PDRF 8** (RGB + NIR), 38 B per point, laszip;
  - **100,000,000 points** = the 10000 × 10000 grid at 0.2 m;
  - bbox E 706000.1–707999.9, N 5706000.1–5707999.9 (pixel centres), Z 76.68–181.86 m, scale 0.001/0.001/0.01;
  - VLRs: GeoKeys 25832 **and** an OGC WKT (2112);
  - creation day 240/2024; software "lascolor (201003)".
- This is a **surface** model (absolute heights), not a normalised nDOM. nDSM = bDOM20 − DGM1 (bilinear), the same recipe as freemapper's Berlin bDOM path, but delivered as points like LGB's nDOM LAZ.
- It is 257 MB per 2 km tile, against Brandenburg's few MB per 1 km. A corner location needs up to 4 tiles (≈1 GB), and the date is 2024 (older than the 2026 TrueDOP).
- **The point content was not decoded** (no LAZ decoder in this environment). Classification, RGB/NIR use and gaps are unverified.

### 6.3 nDOM / LiDAR

- **No nDOM** product exists anywhere on the open data page.
- LiDAR ("3D-Messdaten", LAS 1.2 PDRF 3, 4–8 pts/m², ground/non-ground classes) is openly available **only for two areas, as huge zips**:
  - `…/externedaten/Gemeinde_HalleSaale.zip`: **17,431,294,330 B**, Last-Modified 2023-06-16, `Accept-Ranges: bytes`. 51 members `Gemeinde_HalleSaale/3dm_32_EEE_NNNN_2_st_2017.laz` (2 km tiles, 245–610 MB each, deflated inside the zip; one tile is from 2021).
    The school tile `3dm_32_706_5706_2_st_2017.laz`: 329 MB; LAS 1.2, PDRF 3, 49,089,195 points (≈12 pts/m²), CRS GeoKey 25832, flight day 131/2017.
    A single tile could be range-read out of the zip (its local offset is in the central directory), but it is 7 years old.
  - `…/externedaten/Hakel.zip`: 2,949,239,207 B (2025-11-19).
  - Elsewhere: "auf Antrag", 190 € per data set.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **Tile index WFS** `https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis` (ArcGIS; `ImplementsResultPaging TRUE`; GeoJSON, CSV, GML, Shape, GPKG) | `truedop_kacheln` / `truedop_aktualitaet_zyklus` (`NAME`, `LOSNR`, `BFLOS`, `BFDATUM` as `DD.MM.YYYY` in GeoJSON, ISO in CSV); `bdom_aktualitaet_zyklus` (DOM: `BFDATUM`, `Dateiname`, `Aktualitae`, `GENAUIGKEI`, `CRS_*`) | one BBOX query per product. Example: `…&REQUEST=GetFeature&TYPENAMES=atkis_geobasis_bestand:truedop_kacheln&OUTPUTFORMAT=GEOJSON&BBOX=706440,5706900,706460,5706920,urn:ogc:def:crs:EPSG::25832` → 436 B, 0.2 s. GeoJSON output is **EPSG:4326** unless `SRSNAME` is given. The whole TrueDOP index as CSV is 1.4 MB, 1.9 s. **Not verified:** whether the bdom layer dates the bDOM20 as well (its `Dateiname` names DOM1) |
| `.meta` (first zip member) | every map downloader product | key/value text; DOP meta is Latin-1, DGM/DOM meta UTF-8 |
| `ETag` / `Last-Modified` | ATOM files | HEAD or conditional GET (304). DOP: Last-Modified ≈ release date (4–6 months after the flight) |
| ALKIS `aktualit` | per feature | `YYYY-MM-DDZ`. At the Latina bbox, buildings are mostly 2026 and land use 2023–2024 |
| DOP Shape `dop_aktualitaet.zip` | DOP | `/datei/anzeigen/id/250488,501/dop_aktualitaet.zip` (154 KB, not downloaded) |

Suggested mapping:
- orthophoto `currency` = `BFDATUM`, `published` = ATOM `Last-Modified`/`ETag`;
- DGM1 `currency` = meta `Fortfuehrung` (else `Aktualitaet`);
- DOM1/bDOM `currency` = `bdom_aktualitaet_zyklus.BFDATUM` or meta.

The map downloader gives no stable published marker (no ETag, fresh name per request).

---

## 8. ALKIS: buildings and actual land use

### 8.1 Service

- Endpoint **`https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest`**. It is an XtraServer-style AdV "WFS ST ALKIS Vereinfacht", namespace `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0`.
  - Provider "Landesamt fuer Vermessung und Geoinformation Sachsen-Anhalt"; `Fees` "Kostenverordnung …" (but it is listed under Open Data with dl-de/by-2-0); AccessConstraints → <https://www.lvermgeo.sachsen-anhalt.de/de/nutzungsbedingungen.html>.
- **Feature types: only `ave:Flurstueck`, `ave:GebaeudeBauwerk`, `ave:Nutzung`.** There is **no `NutzungFlurstueck`**, and `VerwaltungsEinheit` is in a separate DVG WFS (§10).
- CRS: Default 25832; Other 25833, 3034, 3035, 3044, 3045, 3857, 4258, 4326 (and 4647).
- Output: GML 3.2 (default) and GML 3.1.1 only. **No GeoJSON**: `OUTPUTFORMAT=application/json` returns **HTTP 200** with `Content-Type: application/vnd.ogc.se_xml` and a `ServiceExceptionReport` "Could not process client request". Errors arrive with status 200, so check the content type.
  An invalid `PROPERTYNAME` gives 200 + `ows:ExceptionReport`.
- Paging and limits:
  - `CountDefault` **100000**; `ImplementsResultPaging FALSE`; `numberMatched="unknown"` on normal requests; no `next` links.
  - `RESULTTYPE=hits` gives an exact `numberMatched` (4080 buildings in 2 × 2 km of Halle centre, 0.5 s).
  - `COUNT=3&STARTINDEX=0/3` returned disjoint, consecutive oids, so paging works de facto.
  - Behaviour above 100000 matches was not tested.
- Speed: school bbox 0.27–0.35 s; 2 × 2 km of Halle centre: buildings 4080 features / 4.4 MB / 4.5 s, land use 3815 / 4.0 MB / 4.1 s. Much faster than Saxony.
- Geometry: always `gml:MultiSurface/surfaceMember/Polygon/exterior|interior/LinearRing/posList`, `srsDimension="2"`, E N order in 25832 and 25833. **Not clipped** to the bbox. Native coordinates have mm precision; reprojected ones have 9 decimals.
- GML uses a **default namespace** (no `ave:` prefix on feature elements). A parser written for Saxony's prefixed output must match on local names.
- `DescribeFeatureType` (all elements as in the AdV schema; `minOccurs=0` where noted):
  - `GebaeudeBauwerk`: `oid`, `aktualit`, `geometrie`, `gebnutzbez`, `funktion`, `gfkzshh`?, **`rellage`?**, `name`?, `anzahlgs`?, `gmdschl`?, `lagebeztxt`?
  - `Nutzung`: `oid`, `aktualit`, `geometrie`, `nutzart`, **`bez`?**, `name`?
  - `Flurstueck`: `idflurst`, `flstkennz`, `land`, …, `flaeche`, `lagebeztxt`, `tntxt`?

Working example (Latina, 25833):

```
https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=289650,5706880,289950,5707250,urn:ogc:def:crs:EPSG::25833
```
→ 200 `application/gml+xml;version=3.2.1`, 60 features, 80,608 B. With `TYPENAMES=ave:Nutzung`: 62 features, 85,189 B.

### 8.2 Sample features (geometry elided)

```xml
<GebaeudeBauwerk gml:id="DESTLIKA0001MuQqBL">
  <oid>DESTLIKA0001MuQqBL</oid>
  <aktualit>2026-04-17Z</aktualit>
  <geometrie>…MultiSurface srsName="urn:ogc:def:crs:EPSG::25833"…</geometrie>
  <gebnutzbez>Gebäude</gebnutzbez>
  <funktion>Allgemein bildende Schule</funktion>
  <gfkzshh>31001_3021</gfkzshh>
  <name>Latina August Hermann Francke</name>
  <gmdschl>15002000</gmdschl>
  <lagebeztxt>Franckeplatz 1 Haus 42</lagebeztxt>
</GebaeudeBauwerk>

<GebaeudeBauwerk gml:id="DESTLIKA0007rKwQBL">          <!-- Halle centre -->
  <oid>DESTLIKA0007rKwQBL</oid><aktualit>2019-01-08Z</aktualit>
  <gebnutzbez>Bauteil</gebnutzbez><funktion>Tiefgarage</funktion>
  <rellage>Unter der Erdoberfläche</rellage><gmdschl>15002000</gmdschl>
</GebaeudeBauwerk>

<Nutzung gml:id="DESTLIKA0001MuqDTN">                    <!-- school grounds -->
  <oid>DESTLIKA0001MuqDTN</oid><aktualit>2023-09-20Z</aktualit>
  <nutzart>Fläche besonderer funktionaler Prägung</nutzart><bez>Öffentliche Zwecke</bez>
</Nutzung>

<Nutzung gml:id="DESTLIKA0006LikiTN">
  <oid>DESTLIKA0006LikiTN</oid><aktualit>2023-09-20Z</aktualit><nutzart>Platz</nutzart>   <!-- no bez -->
</Nutzung>
```

`oid` has 18 characters (`DESTLIKA…BL` for buildings, `…TN` for land use). It is unique, so dedup by oid works.

### 8.3 Observed values (Halle centre 2 × 2 km, Magdeburg centre 1.4 × 1.1 km, Latina bbox)

- `gebnutzbez`: `Gebäude` (4051 + 770) and **`Bauteil`** (29 + 11).
- **`rellage = "Unter der Erdoberfläche"`** occurs (Halle 29, Magdeburg 12), always on `Bauteil` with `funktion` `Keller` or `Tiefgarage`. Brandenburg's underground test therefore works unchanged (unlike Saxony).
- `funktion`:
  - Wohngebäude, Gebäude für Wirtschaft oder Gewerbe, Gemischt genutztes Gebäude mit Wohnen, Garage;
  - Gebäude für öffentliche Zwecke / soziale Zwecke (generic, frequent), Krankenhaus, Hochschulgebäude (Fachhochschule, Universität), Keller, Verwaltungsgebäude;
  - **Allgemein bildende Schule**, Sport-, Turnhalle;
  - Theater, Oper, Bibliothek, Bücherei, Museum, Kirche, Parkhaus, Tiefgarage, Parlament, Gericht, Polizei …
  - No `;`-joined multi-values were seen.
- `name` is present on landmarks (`Latina August Hermann Francke`, `Domgymnasium`, `Franckesche Stiftungen`, …). `anzahlgs` was not seen.
- `Nutzung.nutzart`: Wohnbaufläche, Fläche gemischter Nutzung, Straßenverkehr, Industrie- und Gewerbefläche, **Platz**, Fläche besonderer funktionaler Prägung, Sport-, Freizeit- und Erholungsfläche, Bahnverkehr, Weg, Gehölz, Friedhof, Fließgewässer. All are Brandenburg spellings.
- `bez` (Halle, 1297 of 3815 objects):
  - Gebäude- und Freifläche Mischnutzung mit Wohnen (on Fläche gemischter Nutzung), Handel und Dienstleistung, Industrie und Gewerbe, Tankstelle, Funk- und Fernmeldeanlage (on Industrie- und Gewerbefläche);
  - **Öffentliche Zwecke**, Kultur, Medien und Kommunikation (on Fläche besonderer funktionaler Prägung);
  - Grünanlage (on Sport-, Freizeit- und Erholungsfläche).
  - Magdeburg adds Umspannstation, Wasserwerk, Schwimmen, **Festplatz**.
- **Missing `bez`:**
  - `Platz` 238 of 238 (Halle), so no `Parkplatz`/`Marktplatz`/`Fußgängerzone` and `PAVED_SQUARES` never matches;
  - `Straßenverkehr` 539 of 539, so `Begleitfläche Straßenverkehr` never fires;
  - `Wohnbaufläche`, `Weg` and `Bahnverkehr` have none.
- The school parcel has `bez` **"Öffentliche Zwecke"**, not Brandenburg's "Bildung und Wissenschaft". This matters wherever freemapper keys on that value (test fixtures in `fetch.rs`/`alkis.rs`).
- `Nutzung` is **not split per parcel** (Brandenburg's `nutzung_flurstueck` is). freemapper's `data/alkis-land-use.geojson` then holds whole land-use objects; that is fine for `nutzart`-based rules.

### 8.4 Alternatives (not needed while the WFS works)

- Amtliche **Hausumringe** (statewide Shape): `https://www.geodatenportal.sachsen-anhalt.de/gfds_webshare/download/LVermGeo/Geodatenportal/content/Hausumringe.zip` → 200, 141,886,814 B, Last-Modified 2026-10-16, "Stand 09/2026". Not downloaded; HU-DE format without freemapper's function attribute.
- INSPIRE WFS: buildings `https://geodatenportal.sachsen-anhalt.de/ows_INSPIRE_LVermGeo_ALKIS_BU_WFS`, land use `…_ALKIS_ELU_WFS`. Not examined.
- Basis-DLM WFS / Shape (`Basis-DLM_20261012_shp.zip`, 365,868,949 B). Generalised, so unsuitable for parcel-exact out-of-bounds areas.
- LoD2 WFS `https://www.geodatenportal.sachsen-anhalt.de/ows_ST_LVermGeo_LoD2_WFS` (not examined).

---

## 9. Licence and attribution

- Terms of use: <https://www.lvermgeo.sachsen-anhalt.de/de/datei/anzeigen/id/3567,501/nutzungsbedingungen_b.pdf> (v5.0, 01.03.2024; the same PDF is shipped in every map downloader zip). § 2 "Offene Daten":
  > Offene Daten des LVermGeo dürfen frei genutzt werden unter der Lizenz „Datenlizenz Deutschland - Namensnennung - Version 2.0“ (www.govdata.de/dl-de/by-2-0). Der Bereitsteller nach Absatz 2 Nr. 1 dieser Lizenz ist mit **„© GeoBasis-DE / LVermGeo ST“** zu bezeichnen.
- The open data page says the same ("… unter der Open-Data-Lizenz Datenlizenz Deutschland – Namensnennung – Version 2.0").
  Exceptions: satellite data, basemap.de and basemap.de P10/TK10 have their own terms.
  The portal's own image credits read "© GeoBasis-DE/LVermGeo ST". The `.meta` files say `Copyright-Vermerk: ©LVermGeo Sachsen-Anhalt 2024/2025`.
- **Recommended freemapper line:** `© GeoBasis-DE / LVermGeo ST <year>, dl-de/by-2-0, Daten geändert`. dl-de/by-2-0 requires marking changes; the year is optional, by analogy with the LGB line.
- The § 3 source note with "[Jahr der letzten Abgabe, Aktenzeichen]" applies only to non-open data (data supplied on application), not to these downloads.

---

## 10. State boundary (for `assets/regions/sachsen-anhalt.geojson`)

- **Polygon source (recommended):** the DVG ALKIS WFS `ave:VerwaltungsEinheit` with `art = Bundesland` (oid `DE15`, `name` Sachsen-Anhalt). It returns 1,393,999 B in 1.7 s. `SRSNAME` 25833 is shown; 25832 is the default.
  ```
  https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DVG_ALKIS_WFS_OpenData/guest?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&FILTER=<fes:Filter xmlns:fes="http://www.opengis.net/fes/2.0" xmlns:ave="http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0"><fes:PropertyIsEqualTo><fes:ValueReference>ave:art</fes:ValueReference><fes:Literal>Bundesland</fes:Literal></fes:PropertyIsEqualTo></fes:Filter>
  ```
  (URL-encode the FILTER.)
  - The service holds 355 units: 1 `Bundesland`, 14 `Kreis / kreisfreie Stadt`, 122 `Verwaltungsgemeinschaft / Einheitsgemeinde`, 218 `Gemeinde`. Fetching all of them is 32 MB, 31 s.
  - The literal `Land` matches nothing.
  - Fields: `oid`, `aktualit`, `art`, `name`, `rs`, `ags`, `uebobjekt`, `ueboname`.
- A **25 m Douglas–Peucker simplification** in freemapper's format (`crs` `urn:ogc:def:crs:EPSG::25833`, `properties.simplified_m`) is in `probe/sachsen-anhalt_simplified25m.geojson`.
  - 4,376 vertices, 75 KB; the Saxony asset is 94.5 KB.
  - The main polygon (with its 7 holes) and 7 small exclaves (0.08–9.4 ha) are kept; 4 polygons that fell below 4 points after simplification were dropped.
- Alternative: the tile index WFS `atkis_geobasis_bestand:bundeslandgrenze` (`OUTPUTFORMAT=GEOJSON`, 0.9–1.2 MB). These are **lines** (14 `MultiLineString` segments with `Grenzregion` Brandenburg / Niedersachsen / Sachsen / Thüringen), not a polygon. The per-neighbour split is useful for checking overlap.
- Statewide DVG Shape: `…/externedaten/DVG_ALKIS.zip` (8,403,203 B, updated daily, 25832).

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/sachsen-anhalt-latest.osm.pbf` → **307** → `sachsen-anhalt-261106.osm.pbf` → 200, **176,879,475 B**, `Last-Modified: Sat, 07 Nov 2026 02:24:15 GMT`, `Accept-Ranges: bytes`.
- State: `https://download.geofabrik.de/europe/germany/sachsen-anhalt-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4956
  ```
  This is the same format and sequence as the Saxony and Brandenburg extracts.
- Poly: `https://download.geofabrik.de/europe/germany/sachsen-anhalt.poly` (buffered: lon 10.5473–13.2127, lat 50.9289–53.0566).

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| Map downloader (`www.lvermgeo.sachsen-anhalt.de/de/mod/…`) | Two steps (`prepare` → URL as text/plain, then GET). New random name per call. **No ranges** (200 full). HEAD works but can block for 40–45 s. 500 + "Daten nicht verf&uuml;gbar." for unknown ids; 500 HTML for unknown file names. Sets a cookie (not needed). ≈1.1 MB/s. Tile ids only from a 2.9 MB HTML page (≈15 s) |
| ATOM files (`www.geodatenportal…/gfds_webshare/sec-download/…`) | Correct ranges (also `bytes=0-0`), ETag, Last-Modified, 304. ≈0.7 MB/s for a full GET, fast small ranges (0.2–0.8 s). 404 for missing tiles |
| Statewide zips (`…/gfds_webshare/download/…`) | Ranges + Last-Modified; huge files (LiDAR Halle 17.4 GB) |
| WMS DOP | Max 4096 px; no cache headers (`no-store`); no GetFeatureInfo |
| WCS DGM1/DOM1 (ArcGIS) | Always `multipart/related` (GML + TIFF); **25832 output has no GeoKeys**; ≤ 4000 px per axis (else 400 HTML); uncompressed Float32; nearest-neighbour reprojection |
| ALKIS / DVG WFS | **Errors with HTTP 200** (`application/vnd.ogc.se_xml` or `ows:ExceptionReport`); GML only; default namespace; fast (0.3–4.5 s) |
| Tile index WFS (ArcGIS) | GeoJSON in 4326 by default; paging supported; CSV available |

No authentication, no rate-limit headers, no throttling and no failed requests were seen in roughly 120 requests. No fair-use rules were found on the pages read (not exhaustive).
Two hosts are in use: `www.geodatenportal.sachsen-anhalt.de` and `geodatenportal.sachsen-anhalt.de`. Both answer; the portal mixes them.

---

## 13. What makes the Saxony-Anhalt adapter differ (risks first)

1. **CRS EPSG:25832 everywhere, and the state straddles 12°E (10.56–13.19°E).** freemapper is 25833-only and cannot warp rasters (`raster.rs` rejects rotated or misaligned grids). The 25832 and 25833 grids differ by ≈4.7° around Halle. Options:
   - (a) make the project CRS per region (or per map), 25832 for Saxony-Anhalt (`crs.rs`, OOM georef, `tiles.rs`, ALKIS/OSM projection);
   - (b) ask the services for 25833 (WMS DOP, WCS DGM1/DOM1 with nearest neighbour, WFS), which gives up exact tiles and currency for the rasters;
   - (c) add a local warp.
   - Cross-border maps (Halle–Leipzig with Saxony, Dessau–Wittenberg with Brandenburg) mix zones in any case.
   - **A user decision is needed.**
2. **Three download paths with different versions.** The newest TrueDOP (2026) is only in ATOM/WMS; the map downloader DOP lags one cycle. ATOM DGM1 is **stale**; the current DGM1 comes only from the map downloader or WCS.
3. **The map downloader is an undocumented two-step AJAX API.** Product-specific internal ids come from scraping a 2.9 MB page; there are no ranges, slow HEAD, random file names and no ETag. Treat it as fragile; cache the id map and re-scrape on a 500.
4. **No TFW** for DGM1, DOM1 or the ATOM DOP. Georeferencing must come from the GeoTIFF tags (PixelIsArea tiepoint at the NW corner) or from the tile label. freemapper's `GridSpec::from_world_file` is not enough; it needs a GeoTIFF tag reader or label-derived grids.
5. **TIFF layouts:**
   - DOP is uncompressed, 512 px tiled, with the IFD at the end. Good for ranges, but it needs tiled-TIFF reading.
   - DGM/DOM are LZW strips with the IFD at the end, so the whole file is needed (8–11 MB, fine).
   - WCS output is multipart.
6. **High-resolution nDSM is heavy.** bDOM20 is 257 MB LAZ per 2 km tile without ranges. The light DOM1 is 1 m and image-based. There is no nDOM. LiDAR exists only for Halle (2017) and the Hakel.
7. **ALKIS:**
   - GML only, default namespace, errors with HTTP 200;
   - no `NutzungFlurstueck` (only `Nutzung`);
   - `bez` missing on Platz and Straßenverkehr (paved rules degrade as in Saxony);
   - school parcel `bez` is "Öffentliche Zwecke".
   - Positive: `rellage`/`Bauteil` present, so underground detection works; `name` present; fast.
8. **2 km tiles, labelled `32EEENNNN`.** File names carry a year suffix (`_2_st_2024`). The meta encoding differs between products (Latin-1 for DOP, UTF-8 for DGM/DOM).
9. **Currency sources differ per product:** TrueDOP and DOM come from the WFS index, DGM1 only from the meta inside the zip, and the ATOM feed dates are stale.
10. Unchanged from Brandenburg: dl-de/by-2-0 (provider note differs), DHHN2016, no login, DGM1 pixel registration (`E0+0.5` centre convention), Float32 with NoData −9999, Geofabrik layout.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Saxony-Anhalt | Notes |
|---|---|---|
| **Orthophoto template** | **ATOM TrueDOP20** `…/gfds_webshare/sec-download/LVermGeo/DOP20/32EEENNNN.tif`, reading only the needed 512 px internal tiles by HTTP range; currency `truedop_kacheln.BFDATUM`; published = `ETag`/`Last-Modified` | Newest flight (2026), RGB, exact pixels, ≈15–20 MB per schoolyard instead of 300 MB. Needs a small tiled-TIFF range reader (header + IFD tail + tiles). Fallback/fast mode: **WMS** `lsa_lvermgeo_dop20_2` (≤ 4096 px per request, stitched). In 25832 on the native grid it gives exact pixels; it also serves 25833. **Decision:** 25832 project vs 25833 resampling (§13.1). |
| **DTM** (hillshade, slope, contours) | **DGM1 from the map downloader** (module `2,2913,501`, id from `gdp-dgm1.html`) with currency from `.meta` (`Fortfuehrung`/`Aktualitaet`); or **WCS** `ows_WCS_ST_DGM1` for a clip (≤ 4000 px, optional `OUTPUTCRS` 25833, nearest neighbour) | **Do not use the ATOM DGM1** (stale). WCS is simpler and more stable but has no currency; the map downloader has currency but an undocumented API. **Decision:** WCS + "currency unknown", or map downloader. |
| **nDSM template** | Default **DOM1 − DGM1** (WCS `ows_WCS_ST_DOM1` or map downloader `3,2926,501`), clamp < 0; currency `bdom_aktualitaet_zyklus.BFDATUM` | 1 m, image-based (2024). Optional high-res mode: **bDOM20 LAZ** (`4,1974,501`, 257 MB per tile) minus DGM1, reusing the LAZ→grid code of the LGB nDOM path and reprojecting points if the project is 25833. |
| **Buildings** | **ALKIS WFS** `ave:GebaeudeBauwerk`, GML 3.2, `SRSNAME`/`BBOX` in the project CRS | Same attributes as LGB, including `rellage` and `Bauteil`. Page with `COUNT`/`STARTINDEX` (works although not advertised); check `RESULTTYPE=hits`; retry on non-GML 200 answers. |
| **Land use** (out-of-bounds, paved) | **ALKIS WFS** `ave:Nutzung` | No parcel split. Squares and verges have no `bez`: keep squares as candidates (like Berlin/Saxony). The school parcel is "Öffentliche Zwecke". |
| **Region boundary** | DVG WFS `VerwaltungsEinheit` `art=Bundesland` (DE15), simplified 25 m | Sample in `probe/sachsen-anhalt_simplified25m.geojson`. Overlaps with the Saxony and Brandenburg assets need the usual margin. |
| **OSM** | Geofabrik `europe/germany/sachsen-anhalt-latest.osm.pbf` + `sachsen-anhalt-updates/state.txt` | 177 MB; follows the 307 redirect to `sachsen-anhalt-YYMMDD.osm.pbf`. |
| **Attribution** | `© GeoBasis-DE / LVermGeo ST <year>, dl-de/by-2-0, Daten geändert` + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- That the ATOM/WMS DOP for the Halle tile is the 2026-04-29 flight. This is inferred from Last-Modified ordering against the index for three tiles and from the pixel difference to the 2024 map downloader DOP; the provider does not state it.
- That DOM1 is derived from the image-based bDOM (inferred from the date, which equals the image flight, and the 0.5 m accuracy).
- bDOM20 point content (classes, NIR/RGB, holes). The header only was read, because no LAZ decoder was available. Whether `bdom_aktualitaet_zyklus` also dates the bDOM20.
- The map downloader DOP20's own IFD (only its pixel layout was inspected); the RGBI band order.
- The WMS's resampling method in 25833 and its JPEG quality; the WMS at the 4096 px limit.
- ALKIS WFS behaviour above `CountDefault` 100000; whether `STARTINDEX` ordering is stable across requests; whether the missing `bez` holds statewide (checked in Halle and Magdeburg centres only).
- INSPIRE BU/ELU WFS, the LoD2 WFS, Hausumringe content, `dop_aktualitaet.zip`, the statewide DGM1/DOM1 parts and the LiDAR flight programme PDF were not examined or downloaded.
- How long the map downloader's generated files live (≥ 80 min seen); whether the server enforces any per-request tile cap (6 accepted).
- Rate limits or fair-use rules (none seen, none documented on the pages read).
- No raster was downloaded for the Domgymnasium tile (only currency and ALKIS).

Network note: total transfer was roughly **215 MB**. The largest items were WCS size-limit probes (≈75 MB), the full DVG unit list (32 MB), DGM1 twice (the first time through the range-ignoring map downloader), DOM1 and the ATOM DGM1. No DOP, bDOM or LiDAR tile was downloaded in full. Disk usage under `sachsen-anhalt/` is about 129 MB.
