# Hamburg (LGV) data sources for freemapper — live-verified reference

Verified live on 2026-11-07 (Europe/Berlin). Provider: **Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und
Vermessung (LGV)**. Open data is published through the Transparenzportal (CKAN, <https://suche.transparenz.hamburg.de>),
the Urban Data Platform services on `geodienste.hamburg.de` (WMS/WFS) and `api.hamburg.de/datasets/v1` (OGC API
Features), and file downloads on `www.daten-hamburg.de` and `archiv.transparenz.hamburg.de`. Metadata lives in MetaVer
(<https://metaver.de>). No login, registration, token or form is needed anywhere.

Test locations:

| School | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile |
|---|---|---|---|---|---|---|
| **Gymnasium Eppendorf**, Hegestraße 35 (primary) | way **27470483** | E 565148 / N 5937825 | E 168078 / N 5949079 | 53.58522 / 9.98414 | E 565089–565212, N 5937781–5937883 | `32 565 5937` (fully inside) |
| Gelehrtenschule des Johanneums, Maria-Louisen-Straße 114 (not probed further) | way **31135163** | E 566642 / N 5938392 | E 169616 / N 5949519 | 53.59013 / 10.00682 | E 566582–566757, N 5938319–5938430 | `32 566 5938` |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7°. The ALKIS school building came back at
E 565101–565200 / N 5937787–5937855 in 25832, which is consistent. The WFS's own 25833 output matches my code within 1.4 mm,
see §10.)

Samples (all under `hamburg/`; treat them as untrusted data):

| Path | Content |
|---|---|
| `dl/dop/dop20rgbi_32_565_5937_1_hh_2026_4.tif`, `…_2026_3.tif` | the school's 1 km DOP 2026 tile, **twice**: the Hamburg-Nord copy (`_4`, 27.5 % valid) and the Eimsbüttel copy (`_3`, 72.5 % valid), each range-read out of a 2 GB district zip and inflated |
| `dl/dgm1/dgm1_32_565_5937_1_hh_2022.tif`, `dgm1_32_548_5935.tif`, `dgm1_hh_2023-02-09.csv` | DGM1 school tile, a border tile and the per-tile metadata CSV, range-read from the 1.36 GB statewide zip |
| `dl/bdom/dom1_32_565_5937_1_hh_2022.tif`, `dom1_hh_2022-11-21.csv` | bDOM 1 m school tile and its CSV, range-read from the 1.34 GB statewide zip |
| `dl/dopmeta/` (`x/`, `y/`) | AdV per-tile DOP metadata zips (old snapshot, and the new one with 2026), with the `Spezifikation_DOP_<year>_unbelaubt_hh.pdf` files |
| `dl/wms/` | DOP WMS GetMaps: school for TIME default/2026/2025/2024/2023/2021 and leaf-on (`compare.jpg` side by side), school in **EPSG:25833**, a 4096 px request, native-grid PNG `native_k62.png`, outside-Hamburg probes |
| `dl/wmsdgm/dgm_wms.tif` | DGM WMS `image/tiff` output (rendered RGB, not heights) |
| `dl/docs/dop_faq.pdf` (+ `.txt`) | "DOP Zeitreihen – Beschreibung und FAQ" (Stand 27.11.2025) |
| `probe/eppe_*`, `city_*`, `centre_*`, `wfs_eppe_*`, `page*.json`, `pg*.json` | ALKIS OGC API / WFS answers: school bbox (25832, and 25833 via the WFS), 2 × 2 km around the school, 1.5 × 1.5 km of the city centre, paging tests |
| `probe/land_ve_25832.json`, `landesgrenze_25832.json`, `landesgrenze_wfs_25833.{gml,json}` | state polygon from the OGC API (25832) and from the WFS (25833) |
| `probe/hamburg_simplified25m.geojson` | the 25 m simplified boundary in the `assets/regions/*.geojson` format |
| `probe/zip_*.tsv` | member lists (name, size, compressed, method, date, local offset) of the DOP 2024/2025/2026 district zips, DGM1 2022 and bDOM 2022 zips, read from their central directories |
| `probe/pkg_*.json`, `ckan_*.json`, `api_*.json`, `caps_*.xml`, `wfs_*caps.xml` | portal (CKAN) answers, OGC API landing/collections/OpenAPI, WMS/WFS capabilities |
| `scripts/` | the helper scripts used (UTM, zip-by-range, TIFF tags, DOP mask/JPEG decoding, simplification) |

---

## 1. Brandenburg → Hamburg at a glance

| freemapper product | Brandenburg (today) | Hamburg equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** Hamburg sits at 8.4–10.3°E, entirely in zone 32. The WMS and the classic WFS reproject to 25833; the **OGC API does not offer 25833**. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **True-DOP20 RGBI, 2026 leaf-off flight (09.03.–08.04.2026)**. Two paths: the **WMS** `wms_dop_zeitreihe_unbelaubt` with a `TIME` dimension (2001–2026, ≤ 4096 px, also 25833), or 1 km GeoTIFF tiles (JPEG q90, 256 px internal tiles) inside **2–6 GB zips, one per district, each clipped to the district**. HTTP ranges work. | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1 2022** (ALS 2022), GeoTIFF Float32 LZW, 1 km, **no TFW**, NoData pixels −9999 (the tag says −3.4e38). Only as a **1.36 GB statewide zip** on an archive URL; members can be range-read (≈2 MB per tile). **No WCS.** | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **bDOM 2022, 1 m** (image-based, from the March 2022 True-DOP flight), Float32 uncompressed, 1 km, in a 1.34 GB statewide zip (range-readable). nDSM = bDOM − DGM1. **No nDOM, no LiDAR point cloud, no higher-resolution DOM is open.** | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **OGC API** `alkis_vereinfacht/collections/GebaeudeBauwerk` (ldproxy, the same API family as LGB, `limit` ≤ 10000, `next` links). Alternatively the deegree WFS `WFS_HH_ALKIS_vereinfacht` (GeoJSON, 25833). The attributes have LGB names, but **`gebnutzbez` is "Gebaeude"** (ASCII). | verified |
| Land use | OGC API `nutzung_flurstueck` | **OGC API `NutzungFlurstueck`** (parcel split, like LGB) and `Nutzung`. **`nutzart` uses ASCII class-name spellings** ("Wohnbauflaeche", "Strassenverkehr", "Industrie Und Gewerbeflaeche"); `bez` is proper German and is set on `Platz` and `Straßenverkehr`. | verified |
| Currency | `bb_*_aktualitaet.csv` | AdV metadata CSV per DOP vintage (flight date per tile, 1 MB zip); DGM/bDOM CSV inside the statewide zip; `aktualit` per ALKIS feature; ETag/Last-Modified on all zips | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `hamburg-latest.osm.pbf` (54.1 MB, includes Neuwerk) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | dl-de/by-2-0, Quellenvermerk **"Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV)"**. Exception: DOP vintages **2023 leaf-off (Maxar)** and **2022 leaf-on (Pléiades)** in the WMS are under closed licences. | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **CKAN portal API** | `suche.transparenz.hamburg.de/api/3/action/{package_search,package_show,resource_search}` | catalogue of everything | Every new release becomes a **new package** with a numeric suffix (`…-dgm-16`, `…-unbelaubt7`). Several packages share a title, and two may share a modification time. Resource URLs in newer packages often point at **archive snapshots** (`archiv.transparenz.hamburg.de/hmbtgarchive/HMDK/<name>_<id>_snap_<n>.<ext>`), not at the live service |
| **OGC API Features** | `api.hamburg.de/datasets/v1/<dataset>` | ALKIS vereinfacht, Verwaltungsgrenzen, tile indexes, LoD2 … | ldproxy; GeoJSON; CRS84/25832/4326/3857 only |
| **WMS / WFS** | `geodienste.hamburg.de/<service>` | DOP time series, DGM WMS, ALKIS WFS (deegree), Verwaltungsgrenzen WFS | MapServer-style WMS (`vnd.jpeg-png`, MVT formats), deegree WFS 2.0. Both offer 25833 |
| **Downloads** | `www.daten-hamburg.de/opendata/…` (the bare host `daten-hamburg.de` sends **301** to `www.`) | DOP zips, bDOM zip, DOP metadata | HTTP/2, `Accept-Ranges`, ETag, Last-Modified |
| **Archive downloads** | `archiv.transparenz.hamburg.de/hmbtgarchive/HMDK/…_snap_<n>.zip` | DGM1 2022 (the original `daten-hamburg.de/…/DGM1/dgm1_hh_2022-04-30.zip` is **404**), older metadata zips | IIS-style; ranges, ETag, Last-Modified |

Dead or changed endpoints (still linked from older packages):
- `https://geodienste.hamburg.de/HH_WMS_DOP` → **404**. It was replaced by `wms_dop_zeitreihe_unbelaubt` / `wms_dop_zeitreihe_belaubt`.
- `https://daten-hamburg.de/geographie_geologie_geobasisdaten/Digitales_Hoehenmodell/DGM1/*` → **404**. Only the archive copies of the DGM1 zips remain.
- The metaver CSW (`https://metaver.de/csw?…GetRecords`) answered **403**; the HTML `trefferanzeige` pages work.

### 2.1 Tile scheme

- **1 km × 1 km**, EPSG:25832, SW corner at whole km, for DOP, DGM1 and bDOM. File names:
  - DOP: `dop20rgbi_32_EEE_NNNN_1_hh_YYYY_<bezirk>.tif` (e.g. `dop20rgbi_32_565_5937_1_hh_2026_4.tif`). The last digit is the **district id** (1 Hamburg-Mitte … 4 Hamburg-Nord, 3 Eimsbüttel …). The spec gives the pattern as `dop<cm><bands>_<zone>_<E km>_<N km>_<edge km>_<land>_<year>_<bezirks-id>`. The metadata CSV names the tile without the district suffix.
  - DGM1: `dgm1_32_EEE_NNNN_1_hh_2022.tif`, in zip folders `S32_EEE/` **or** `s32_EEE/` (the case varies).
  - bDOM: `dom1_32_EEE_NNNN_1_hh_2022.tif`, in folders `S32_EEE/` or `s32_EEE/`.
- Grid, main area: SW corners E 548…588, N 5916…5955 (874 DOP tiles); E 548000–589000, N 5916000–5956000. DGM1/bDOM add Neuwerk/Scharhörn (E 466–468, N 5973–5976): 880 DGM1 and 883 bDOM tiles. The DOP time series does **not** cover Neuwerk; a separate "DOP Zeitreihe Wattenmeer" dataset exists (not examined).
- The tile index API `api.hamburg.de/datasets/v1/uebersicht_kachelbezeichnungen` (collections `lgv_kachel_dop20_1km_utm`, `lgv_kachel_dgm_2km_utm`, …) is **stale**. It gives names like `dop20rgb_32565_5937_1_hh_JAHR` and 2 km `DGM1_32564_5936_2_FHH.xyz`, which do not match the current files. Build names from the grid instead.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DOP: GeoKeys `ProjectedCSType 25832`; meta `Koordinatenreferenzsystem_Lage 25832`.
  - DGM1/bDOM: GeoKeys 25832; CSV `ETRS89_UTM32`.
  - ALKIS: `storageCrs EPSG/0/25832`.
  - WMS/WFS DefaultCRS is 25832.
- **Height datum DHHN2016** normal heights: DGM/bDOM CSV `DE_DHHN2016_NH`, `Hoehenanomalie DE_AdV_GCG2016_QGH`; DOP CSV `Koordinatenreferenzsystem_Hoehe 7837`. (The 2024 leaf-on CSV says `7873`, presumably a typo.) The TIFFs carry no vertical GeoKey. This is the same height system as Brandenburg, Saxony and Saxony-Anhalt.
- **Zone 33 is a poor fit for Hamburg.** In 25833 the state lies at E 67963–189648, i.e. 4.7–6.6° west of the zone-33 central meridian (15°E):
  - grid convergence ≈ −4.0° (city) against +0.8° in zone 32, so the grids differ by **≈4.8°**;
  - the zone-33 scale factor is ≈1.0010 in the city (+1 m per km) and ≈1.0019 at Neuwerk.
  - Hamburg maps must be **25832 projects**, like Saxony-Anhalt (`Region::crs() → Crs::Utm32`).
- **Extent** of the state polygon (`landesgrenze`, 8517 vertices, 4 polygons, 1 hole, 754.6 km² in 25832):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 461485 – 588010, N 5916918 – 5979762 |
  | EPSG:25833 | E 67963 – 189648, N 5926818 – 5999561 |
  | lon / lat | lon 8.4132 – 10.3253, lat 53.3950 – 53.9649 |

- 25833 is available through services only:
  - DOP WMS and DGM WMS: CRS list CRS:84, 25832, **25833**, 3044, 31467, 3857, 4258, 4326, 4647. GetMap in 25833 verified.
  - WFS `WFS_HH_ALKIS_vereinfacht` and `HH_WFS_Verwaltungsgrenzen`: Other CRS 25833, 3034, 3035, 3044, 31467, 3857, 4258, 4326. Verified with `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833`; output is in E N order.
  - **OGC API: no 25833.** `crs=…/25833` → 400 (`does not have a value in the enumeration [CRS84, CRS84, 25832, 4326, 3857]`).

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

From the FAQ (Stand 27.11.2025), the metadata CSVs and the WMS:

| Year | Leaf-off (`unbelaubt`) | Leaf-on (`belaubt`) |
|---|---|---|
| 2021 | DOP 20 cm | – |
| 2022 | **True-DOP** 20 cm (flight 07.–23.03.2022) | **50 cm Pléiades satellite, closed licence, no download** |
| 2023 | **30 cm Maxar satellite, closed licence, no download** | – |
| 2024 | – | True-DOP 20 cm (01.–15.05.2024) |
| 2025 | True-DOP 20 cm (17.–20.02.2025) | – |
| **2026** | **True-DOP 20 cm (09.03.–08.04.2026)**, released 2026-09 (zips Last-Modified 2026-09-09 / 2026-10-27; metadata 2026-08-10) | planned 2027 |

The school tile `565/5937` was flown on **19.03.2026** (camera UltraCam-Osprey 4.1). Previous flights of this tile: 18.02.2025, 08.05.2024 (leaf-on), 10.03.2022.

### 4.2 WMS (recommended path)

- Leaf-off: `https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt`. Leaf-on: `https://geodienste.hamburg.de/wms_dop_zeitreihe_belaubt`. Both WMS 1.3.0.
  - Layers `dop_zeitreihe_unbelaubt` and `dop_zeitreihe_belaubt`, `queryable="0"` (no GetFeatureInfo).
  - **`<Dimension name="time" default="2026">2001/2026/P1Y`** (leaf-on: `default="2024">2005/2024/P1Y`).
  - MaxWidth/MaxHeight **4096** (819 m at 0.2 m). 4097 px → **HTTP 200 `text/xml` ServiceExceptionReport**.
  - Formats: png, jpeg, png8, `vnd.jpeg-png(8)`, tiff, pdf, svg, kml(z), MVT, json.
  - BoundingBox 25832: 547999–589000 / 5916000–5956000, so **no Neuwerk**. `MaxScaleDenominator 5000`, but 2 m/px still rendered.
- `TIME` behaviour (school, 400 × 300 m at 0.2 m; `dl/wms/compare.jpg`):
  - no `TIME` = `TIME=2026` = `TIME=2030`, all byte-identical. Per the FAQ, a time outside the range returns the default (newest).
  - `TIME=2025`: a different, earlier leaf-off image. `TIME=2021`: classic DOP.
  - `TIME=2024` (no leaf-off vintage that year): **a blue fill pattern, not transparent, HTTP 200**.
  - `TIME=2023`: the Maxar satellite scene (closed licence).
  - **Always send an explicit `TIME` with an open vintage. Never 2023 leaf-off or 2022 leaf-on.**
- Example (school, 25832, 0.2 m):
  ```
  https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=dop_zeitreihe_unbelaubt&STYLES=&CRS=EPSG:25832&BBOX=564950,5937650,565350,5937950&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg&TIME=2026
  ```
  → 200 `image/jpeg`, 667,241 B, 0.8–1.1 s. 4096² JPEG (819 m): 3.7 MB, 4.5 s. 25833 example (400 × 360 m): `CRS=EPSG:25833&BBOX=167880,5948900,168280,5949260` → 200, 812 KB, visually checked (rotated, school centred).
- **On the native grid in 25832 (PNG), the WMS returns the 2026 tile's pixels.** Against internal tile 62 of the Hamburg-Nord tile the mean abs difference was **2.4** on 0–255, which is JPEG re-quantisation; a 1 px shift gives ≈93. The WMS merges the district pieces seamlessly.
- Outside Hamburg: PNG with `TRANSPARENT=TRUE` is fully transparent; JPEG is **white**.
- Headers: `Cache-Control: max-age=0, must-revalidate, no-cache, no-store, max-age=86400`, `Expires` +1 day, no ETag.
- Fees text in the capabilities: "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV), **Andere geschlossene Lizenz, Quellenvermerk: Maxar Products. Dynamic Product © 2023 Maxar Technologies.**" (leaf-on: "…Pléiades 1A/B © CNES (2022), Distribution Airbus DS").

### 4.3 Tile downloads (district zips)

- URL: `https://www.daten-hamburg.de/opendata/fernerkundung_luftbilder/dop_unbelaubt/DOP<YYYY>_unbelaubt_Hamburg_<Bezirk>.zip`, with `<Bezirk>` ∈ Altona, Bergedorf, Eimsbuettel, Hamburg-Mitte, Hamburg-Nord, Harburg, Wandsbek. Leaf-on: `…/dop_belaubt/DOP2024_belaubt_Hamburg_<Bezirk>.zip`.
  - Spellings vary by year: 2022 uses `DOP2022_unbelaubt_Hamburg-Altona.zip`, `…_Hamburg_Hamburg-Mitte.zip`, `…Hamburg-Eimsbüttel.zip` with an umlaut. Take the URLs from the CKAN package instead of building them.
  - The CKAN packages listing 2026 are `luftbilder-hamburg-dop-zeitreihe-unbelaubt7` (and `…6`, which lists only up to 2025 although both have the same `metadata_modified`).
- Sizes, 2026: Eimsbüttel 1.83 GB, Hamburg-Nord 2.01 GB, Altona 2.70 GB, Hamburg-Mitte 3.89 GB, Harburg 4.35 GB, Bergedorf 5.02 GB, Wandsbek 5.42 GB (**≈25 GB per vintage**). 2025: 2.0–6.3 GB each (≈29 GB).
- HTTP: `HEAD` 200 with Content-Length, ETag, Last-Modified, `Accept-Ranges: bytes`; `Range: bytes=0-0` → **206, 1 byte**.
  The central directory sits at the end (7–14 KB), and members are **deflated** (method 8), not stored. One tile can be read by two ranged requests (local header + compressed member, 0.5–36 MB) plus inflate. Verified for two tiles.
- **Tiles are clipped to the district boundary (spec: "Clipping Hamburg – Amtliche Bezirksgrenzen", "Maske intern").** The school tile exists in **both** the Hamburg-Nord zip (`…_2026_4.tif`, 10.3 MB, mask 27.5 % valid) and the Eimsbüttel zip (`…_2026_3.tif`, 26.0 MB, 72.5 % valid). The masks are exact complements (union 100 %, overlap 0 %). A complete 1 km tile near a district border needs both pieces, merged by mask. The zip listings do not say which tiles are partial.
- TIFF layout (2026; verified on both pieces):
  - 5000 × 5000, **4 bands RGBI** 8-bit, **Compression 7 (JPEG, q 90)** with shared `JPEGTables`, Photometric RGB, ExtraSamples 0;
  - **256 × 256 internal tiles** (20 × 20);
  - IFD 0 at the **end** of the file, plus IFD 1 = **internal 1-bit transparency mask** (`NewSubfileType 4`, deflate, 256 px tiles); no overviews; ImageDescription "Digitales Orthophoto (DOP) 2026 Hamburg, Nutzerdaten_Open_Data";
  - ModelPixelScale 0.2, **ModelTiepoint (564999.9999994638, 5937999.999975801)**, i.e. float noise up to 2.4e-5 m (round to mm), `GTRasterType 1` (PixelIsArea), ProjectedCSType 25832; **no TFW**.
  - Decoding quirk: a 4-component JPEG without an Adobe marker is decoded by Pillow as "CMYK" with **inverted values**. The true RGB is 255 − Pillow's value; with that, it matched the WMS. Masked pixels are 0 (black) under that interpretation.
  - Contrast with the 2025 tiles: similar names (`…_hh_2025_4.tif`), compressed similarly (not opened).
- **Not recommended** for freemapper: 25 GB per vintage, deflate inside the zip, IFD at the end of each member (so the whole member must be inflated), district clipping, 4-channel JPEG decoding. The WMS gives the same pixels.

---

## 5. DGM1 (terrain model, 1 m)

- Product: "Geländemodelle Hamburg – DGM" (CKAN `gelaendemodelle-hamburg-dgm2`, modified 2026-10-28) and the older "Digitales Höhenmodell Hamburg DGM 1" (`…-dgm-16`).
  - Source: ALS flights 2010, 2020 and **2022**. Neuwerk 2022 is derived from an image-based DSM (no ALS).
  - Newest vintage **2022**; no newer DGM is open (checked on 2026-11-07).
  - Accuracy ±15 cm on clear surfaces; CSV `Genauigkeit 0.3` (Neuwerk 0.6).
- **Download (only as a statewide zip):** `https://archiv.transparenz.hamburg.de/hmbtgarchive/HMDK/dgm1_hh_2022-04-30_258334_snap_1.zip` → 200, **1,364,773,543 B**, `Last-Modified: Sat, 08 Aug 2026`, `ETag "884038837d27dd1:0"`, ranges OK.
  A second identical-size copy is `…_243830_snap_2.zip`. The original `daten-hamburg.de` path is 404, so the URL is a **snapshot id that can change with the next portal release**: resolve it via CKAN (newest "Geländemodelle Hamburg - DGM" package, resource format `GEOTIFF`).
  - Central directory 106 KB at the end; 926 entries: 880 tiles, 1 CSV `dgm1_hh_2023-02-09.csv` (93 KB), and folders.
  - Example member: `dgm1_hh_2022-04-30/dgm1_hh_2022/s32_565/dgm1_32_565_5937_1_hh_2022.tif`, 1,920,737 B (deflate 1,919,190). Fetched by range in ≈1–2 s.
  - Older vintages (2013–2021) are XYZ ASCII in 2 km tiles; 2022 is GeoTIFF.
- TIFF:
  - 1000 × 1000 **Float32** (SampleFormat 3), **LZW** (Compression 5, Predictor 1), one strip, IFD at the start;
  - ModelPixelScale 1, **ModelTiepoint (564999.9999999999, 5938000.000000001)** (float noise, round), `GTRasterType 1` (PixelIsArea), ProjectedCSType 25832, no vertical key, **no TFW**;
  - the equivalent TFW is `1/0/0/-1/565000.5/5937999.5`, the **same registration as LGB/GeoSN/LVermGeo DGM1**.
  - **NoData:** the `GDAL_NODATA` tag says `-3.4028234663852886e+38`, but the pixels outside Hamburg are **−9999**. Border tile 548/5935: 53.1 % −9999, no −3.4e38 at all. The school tile has none.
  - **Tiles are clipped to the state boundary.** Treat both values (≤ −9999 and the tag value) as NoData.
  - Values at the school: tile 1.64–19.78 m, school centre **6.15 m** (pixel E 565150.5 / N 5937824.5; the WMS GetFeatureInfo gives the same 6.1500001). Pillow decodes it.
- Metadata CSV (cp1252 with a stray `\x81`; read as Latin-1): `Kachelname;Aktualitaet;Erfassungsmethode;Fortfuehrung;Fortfuehrungsmethode;Genauigkeit;…`
  - school tile: `dgm1_32_565_5937_1_hh_2022;2022-03;5020;2022-03;5020;0.3;ETRS89_UTM32;DE_DHHN2016_NH;DE_AdV_GCG2016_QGH`;
  - 871 tiles are `2022-03 / 5020 / 0.3`, 11 Neuwerk tiles `5040 / 0.6`.
- **WMS** `https://geodienste.hamburg.de/HH_WMS_DGM1` (layers `WMS_DGM1_HAMBURG`, `WMS_DGM1_farbig`, `…_10000`) and `https://geodienste.hamburg.de/wms_gelaendemodelle_dgm` (`gelaendemodelle_dgm`, `…_farbig`, `…_10000`).
  - `FORMAT=image/tiff` returns **rendered 8-bit RGB**, not heights.
  - `GetFeatureInfo` (`text/plain`) returns `value_0` = height per pixel. That is usable for spot checks only.
- **No WCS:** every guessed name was 404, CKAN `resource_search url:wcs` → 0 hits, and no WCS is listed in the packages.

---

## 6. nDSM sources: bDOM, LiDAR

### 6.1 bDOM 1 m (image-based DSM)

- CKAN `digitales-hoehenmodell-hamburg-bdom8` (2026-06-05). Vintages 2018, 2020, 2021 (XYZ) and **2022 (GeoTIFF, newest)**. It is a by-product of the True-DOP production (dense image matching); the package says it excludes the Wadden Sea area, but the zip does contain Neuwerk tiles.
- `https://www.daten-hamburg.de/opendata/Digitales_Hoehenmodell_bDOM/dom1_hh_2022-11-21.zip` → 200, **1,344,475,498 B**, `Last-Modified: Wed, 06 Mar 2024`, ETag, ranges OK. 884 entries (883 tiles + `dom1_hh_2022-11-21.csv`); folders `S32_EEE/` or `s32_EEE/`.
  - Example: `S32_565/dom1_32_565_5937_1_hh_2022.tif`, 4,003,566 B (deflate 1,871,784).
- TIFF: 1000 × 1000 Float32, **uncompressed** (strips of 2 rows), tiepoint exactly (565000, 5938000), PixelIsArea, 25832, **no NoData tag**, no −9999 at the school. The grid is pixel-aligned with DGM1.
- CSV (Latin-1): `dom1_32_565_5937_1_hh_2022;2022-11;5040;2022-11;5020;0.5;…`. 874 tiles `2022-11`, 9 tiles `2022-7`; accuracy 0.5 m.
  `2022-11` is evidently the production date. The imagery is the 2022 leaf-off True-DOP flight (07.–23.03.2022; inferred).
- **bDOM − DGM1** over the school footprint bbox (12,546 px): min −1.35, p10 −0.01, **median 9.59**, p90 23.01, max 26.38 m; 62.8 % > 2 m. Clamp < 0 to 0.
  The 2022 bDOM is **4 years older** than the 2026 orthophoto, so new buildings will mismatch.

### 6.2 LiDAR / nDOM

- **No open LiDAR point cloud (LAZ/LAS) and no nDOM** were found: CKAN searches for Punktwolke, LAZ, Laserscanning, nDOM and Oberflächenmodell found only derived products (DGM, LoD1/LoD2/LoD3, 3D-Mesh, street-level "360° Panoramabilder mit LiDAR").
- The DGM page offers TIN, contours etc. "als Dienstleistung" (on request).
- bDOM 2025/2026 surely exist internally (the 2025 and 2026 DOPs are True-DOPs on a bDOM) but are not published (as of 2026-11-07).

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **AdV metadata CSV** `https://www.daten-hamburg.de/opendata/fernerkundung_luftbilder/dop_unbelaubt/DOP_unbelaubt_AdV-Metadaten_Hamburg.zip` (1,082,541 B, Last-Modified 2026-09-10, ETag) | DOP leaf-off, all vintages 2016–2026 (`AdV-Metadaten_DOP_<year>_unbelaubt_hh.csv` + `Spezifikation_DOP_<year>_unbelaubt_hh.pdf`) | UTF-8 with BOM (2025/2026) or Latin-1 (older), `;`-separated, 5 header lines + column header. Columns `Kachelname;Aktualitaet` (= **flight date DD.MM.YYYY**)`;Erfassungsmethode;Bildflugnummer;Kamera_Sensor;Bodenpixelgroesse;Spektralkanaele;…;Bezugsflaeche` (`bDOM` = True-DOP)`;Koordinatenursprung_East/North;…;Belaubungszustand`. 874 rows per vintage. The leaf-on equivalent is `…/dop_belaubt/…` (the old archive snapshot `dop_belaubt_adv-metadaten_hamburg_258167_snap_1.zip`, 33,857 B, has 2013–2024) |
| DGM/bDOM CSV inside the statewide zip | DGM1, bDOM | `Aktualitaet` / `Fortfuehrung` as `YYYY-MM`. Range-read the member (3 KB) |
| ETag / Last-Modified | every zip | HEAD; for change detection only (zips are re-published with district re-cuts, e.g. 2026 Hamburg-Mitte/Bergedorf/Harburg zips re-dated 2026-10-27) |
| WMS `TIME` | DOP | the vintage only (a year), not a date |
| ALKIS `aktualit` | per feature | `YYYY-MM-DDZ`. 2 × 2 km at the school: buildings mostly 2012 (1498 of 4441, the initial migration) and 2020–2024; land-use parcels mostly 2022–2026 (1169 from 2026) |

Suggested mapping:
- orthophoto `currency` = CSV `Aktualitaet` for the tile(s) under the map, `published` = metadata zip ETag;
- DGM1 `currency` = CSV `Fortfuehrung` (2022-03);
- bDOM `currency` = 2022 (CSV `2022-11` is the production date).

---

## 8. ALKIS: buildings and actual land use

### 8.1 OGC API Features (recommended)

- Landing page `https://api.hamburg.de/datasets/v1/alkis_vereinfacht` ("ALKIS – Liegenschaften und Verwaltungseinheiten", ldproxy; metadata MetaVer `18E405F7-4566-42DF-BFD7-EC777C97923B`).
  - Collections: **`GebaeudeBauwerk`**, **`NutzungFlurstueck`**, `Nutzung`, `Flurstueck`, `VerwaltungsEinheit`, `KatasterBezirk`. storageCrs 25832; `crs`/`bbox-crs` ∈ CRS84, 25832, 4326, 3857.
  - Conformance: Features 1/2/3/6/8, CQL2 (text/json), queryables, sortables (`sortby`), properties (`properties`, `exclude-properties`, `skipGeometry`).
- **Limits:** OpenAPI `limit` default 10, **maximum 10000** (the same as freemapper's `PAGE_LIMIT`). Larger values are accepted and clamped silently (`limit=100000` → 200).
  - Paging by `offset` with **`next`/`prev` links** in `links` (rel `next`). `limit=1000` over 4441 buildings gave disjoint, consecutive pages.
  - `numberMatched` and `numberReturned` are present on every page (so a separate hits request is unnecessary).
- Speed: school bbox (400 × 350 m) 0.5–0.9 s. 2 × 2 km: buildings 4441 features / 3.6 MB raw (0.8 MB gzip) / **11.8 s cold**, 0.9 s on repeat; NutzungFlurstueck 3886 / 4.7 MB / 16.6 s; Nutzung 1647 / 3.4 MB / 14.0 s. The first `limit=1000` page took 22.6 s cold. `Accept-Encoding: gzip` works.
- Working example (the same shape as freemapper's `collection_url`, but with 25832):
  ```
  https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items?f=json&bbox=564950,5937650,565350,5938000&bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25832&crs=http://www.opengis.net/def/crs/EPSG/0/25832&limit=10000
  ```
  → 200 `application/geo+json`, `Content-Crs: <…/EPSG/0/25832>`, 139 features, 131 KB. Same with `NutzungFlurstueck`: 193 features (313 KB); `Nutzung`: 105.
- Geometry: `MultiPolygon` in E N order; coordinates carry float noise (`565209.4840000002`). Feature `id` is an integer **gid**, not the oid; dedup by `properties.oid`, as freemapper already does.
- Errors: 400/404 come back as **HTML** unless `Accept: application/problem+json` is sent (then `application/problem+json` with a readable message). No ETag on items, so `If-None-Match` → 200.

### 8.2 Classic WFS (alternative, offers 25833)

- `https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht` (deegree, WFS 2.0.0/1.1.0).
  - Types: `ave:Flurstueck`, `ave:GebaeudeBauwerk`, `ave:KatasterBezirk`, `ave:Nutzung`, `ave:NutzungFlurstueck`, `ave:VerwaltungsEinheit`.
  - Output: GML 3.1/3.2, `text/csv`, **`application/geo+json`** (`application/json` is also accepted and returns geo+json).
  - CRS: 25832 default, plus 25833, 3034, 3035, 3044, 31467, 3857, 4258, 4326. Fees "Quellenvermerk: Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV)".
- `ImplementsResultPaging FALSE` in the capabilities, but `COUNT=3&STARTINDEX=0/3&SORTBY=oid` returned disjoint consecutive oids. `ImplementsSorting TRUE`. Without SORTBY the order differs between requests.
- `RESULTTYPE=hits` → `numberMatched="4441"` (2 × 2 km, 0.18 s).
- 25833 example: `…&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=167880,5948900,168280,5949260,urn:ogc:def:crs:EPSG::25833&OUTPUTFORMAT=application/geo%2Bjson` → 200, 170 features. 15.4 s cold; NutzungFlurstueck afterwards 1.2 s. The GeoJSON has no `crs` member, and ids look like `AVE_GEBAEUDEBAUWERK_341242`.
- It holds **the same data** (the same oids and the same transliterated values) as the OGC API.

### 8.3 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid`, `aktualit`, `gebnutzbez`, `funktion`, `gfkzshh`, `rellage`?, `name`?, `anzahlgs`? (3695 of 4441), `gmdschl`?, `lagebeztxt`?, geometry `geometrie`. These are **LGB names**.
- `NutzungFlurstueck`: `oid` (34 characters: Nutzung oid + Flurstück oid, e.g. `DEHHALKAP000Bw4tDEHHALKAP0006mmnTF`), `aktualit`, `nutzart`, `bez`?, `name`?, `flstkennz`.
- `Nutzung`: `oid` (18 characters, `…TN`), `aktualit`, `nutzart`, `bez`?, `name`?.
- oids start with **`DEHH`** (`DEHHALKA…BL` buildings, `…TN` land use), so `Region::of_object_id` gets a `"DEHH" => Hamburg` arm.

### 8.4 Sample features at the Gymnasium Eppendorf (geometry elided)

```json
{"oid":"DEHHALKAP0007dHTBL","aktualit":"2020-06-04Z","gebnutzbez":"Gebaeude","funktion":"Allgemein bildende Schule",
 "gfkzshh":"31001_3021","name":"Gymnasium Eppendorf","anzahlgs":4,"gmdschl":"02000000","lagebeztxt":"Hegestieg 3; Hegestraße 35"}
{"oid":"DEHHALKAP000At5gBL","aktualit":"2010-02-26Z","gebnutzbez":"Bauteil","funktion":"Arkade","gfkzshh":"31002_2400"}
{"oid":"DEHHALKA45I000SEBL","aktualit":"2018-07-18Z","gebnutzbez":"Bauteil","funktion":"Keller","gfkzshh":"31002_2000",
 "rellage":"Unter der Erdoberfläche"}
{"oid":"DEHHALKAP000Bw4tDEHHALKAP0006mmnTF","aktualit":"2026-02-19Z","nutzart":"Flaeche Besonderer Funktionaler Praegung",
 "bez":"Bildung und Wissenschaft","name":"Gymnasium Eppendorf","flstkennz":"020426___01593______"}
{"oid":"DEHHALKA2QL0006qDEHHALKAP0006mI0TF","aktualit":"2026-06-05Z","nutzart":"Strassenverkehr","bez":"Begleitfläche Straßenverkehr",
 "flstkennz":"020426___00213______"}
{"oid":"DEHHALKAP000BvIZDEHHALKAP0006lZ9TF","aktualit":"2010-02-26Z","nutzart":"Wohnbauflaeche","flstkennz":"020426___02236______"}
```

The school parcel has `bez` **"Bildung und Wissenschaft"**, as in Brandenburg.

### 8.5 Observed values (2 × 2 km around the school + 1.5 × 1.5 km city centre)

- **`gebnutzbez` (ASCII, Title Case):** `Gebaeude` 4617, `Bauteil` 1399, `Sonstiges Bauwerk Oder Sonstige Einrichtung` 159, `Bauwerk Oder Anlage Fuer Industrie Und Gewerbe` 5, `Bauwerk Oder Anlage Fuer Sport Freizeit Und Erholung` 4, `Vorratsbehaelter Speicherbauwerk` 3.
  **freemapper's `buildings::classify` keeps only `kind == "Gebäude"`, so every Hamburg building would be dropped without normalisation.**
- `rellage` (proper German):
  - `Unter der Erdoberfläche` on Bauteil (Keller 87, Tiefgarage 136) **and on `Gebaeude`** (Tiefgarage 177, Schutzbunker 14, a few others);
  - `Aufgeständert` on Arkade, Auskragender Geschossteil, Durchfahrt …
  - The underground test works unchanged (it ignores the kind).
- `funktion` (proper German, the AdV names): Wohnhaus 1981, Wohngebäude mit Handel und Dienstleistungen 746, Geringergeschossiger Gebäudeteil 449, Gebäude für Handel und Dienstleistungen 372, Durchfahrt im Gebäude 328, Tiefgarage 315, Arkade 259, Bürogebäude 258, Garage 207, Überdachung 157, … **Allgemein bildende Schule** 21, Sport-, Turnhalle 6. No `;`-joined values were seen.
- **`nutzart` (ASCII class names; mapping to the LGB spelling):**

  | Hamburg | LGB / freemapper spelling |
  |---|---|
  | `Wohnbauflaeche` | Wohnbaufläche |
  | `Flaeche Gemischter Nutzung` | Fläche gemischter Nutzung |
  | `Industrie Und Gewerbeflaeche` | Industrie- und Gewerbefläche |
  | `Flaeche Besonderer Funktionaler Praegung` | Fläche besonderer funktionaler Prägung |
  | `Sport Freizeit Und Erholungsflaeche` | Sport-, Freizeit- und Erholungsfläche |
  | `Strassenverkehr` | Straßenverkehr |
  | `Weg`, `Platz`, `Bahnverkehr`, `Hafenbecken`, `Schiffsverkehr` | unchanged |
  | `Fliessgewaesser`, `Stehendes Gewaesser` | Fließgewässer, Stehendes Gewässer |
  | `Unland Vegetationslose Flaeche` | Unland/Vegetationslose Fläche |

  Classes not in the sample (Wald, Landwirtschaft, Gehölz, Friedhof, Heide, Moor, Sumpf, Flugverkehr, Hafen, Halde, Tagebau …) will follow the same pattern. A robust rule: compare on a key that lower-cases, maps ä/ö/ü/ß → ae/oe/ue/ss and drops `-`, `,` and `/`. "Industrie- und Gewerbefläche" and "Industrie Und Gewerbeflaeche" both become `industrie und gewerbeflaeche`.
- **`bez` (proper German) is present where Saxony/Saxony-Anhalt lack it:**
  - `Strassenverkehr`: `Begleitfläche Straßenverkehr` 1373, `Fahrbahn` 369, none 101;
  - `Platz`: `Fußgängerzone` 82, `Parkplatz` 17, `Busbahnhof` 5, none 4;
  - `Weg`: `Begleitfläche Weg` 60, `Fahrweg` 35, none 57;
  - FBFP: Bildung und Wissenschaft, Gesundheit Kur, Kultur, Regierung und Verwaltung, Soziales, Religiöse Einrichtung, Öffentliche Zwecke …;
  - Gemischt: `Wohnen mit Handel und Dienstleistungen` 692 …;
  - IuG: `Handel und Dienstleistung` 733, Handel 50, Tankstelle …;
  - SFE: Grünanlage, Park, Spielplatz/Bolzplatz, Sportanlage …;
  - `Wohnbauflaeche`: never.
  - So `PAVED_SQUARES` and `VERGE` fire as in Brandenburg once `nutzart` is normalised.
- `name` is present on landmarks (Gymnasium Eppendorf, Universitätsklinikum Hamburg-Eppendorf, "B+R" shelters).

### 8.6 What is missing or different

- No 25833 on the OGC API (use 25832, or the WFS).
- Spellings of `nutzart` and `gebnutzbez` (§8.5).
- `Bauwerk…` objects are mixed into `GebaeudeBauwerk` (dropped by the kind rule anyway).
- ALKIS covers Hamburg only, including Neuwerk (collection extent lon 8.41–10.34).
- INSPIRE BU/ELU WFS, LoD2 and Hausumringe were not examined (not needed).

---

## 9. Licence and attribution

- Licence: **Datenlizenz Deutschland – Namensnennung – Version 2.0** (`dl-de-by-2.0`, <http://www.govdata.de/dl-de/by-2-0>) on every package used. The 2026 DOP spec says "Nutzungsbedingungen: Öffentlich, Datenlizenz Deutschland - Namensnennung - Version 2.0".
- Source note (verbatim from the WMS/WFS `Fees` and MetaVer): **"Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV)"**. The DGM WMS and the Verwaltungsgrenzen WFS omit the "(LGV)".
  The DOP GeoTIFFs carry tag 33432 (Copyright) = "Herausgeber: Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV)".
- **Exceptions:** WMS vintages **2023 leaf-off** ("Maxar Products. Dynamic Product © 2023 Maxar Technologies.") and **2022 leaf-on** ("Pléiades 1A/B © CNES (2022), Distribution Airbus DS") are under "Andere geschlossene Lizenz" and must not go into freemapper projects.
- **Recommended freemapper line:** `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <year>, dl-de/by-2-0, Daten geändert`. (Hamburg does not use a "© GeoBasis-DE/…" form on these pages; dl-de/by-2-0 requires marking changes; the year is optional.)

---

## 10. State boundary (for `assets/regions/hamburg.geojson`)

- **Polygon source (recommended):** the WFS `https://geodienste.hamburg.de/HH_WFS_Verwaltungsgrenzen`, type **`app:landesgrenze`** (deegree; other types `bezirke`, `stadtteile`, `ortsteile`, `gemarkungen`, `baubloecke`). It returns **EPSG:25833 directly**:
  ```
  https://geodienste.hamburg.de/HH_WFS_Verwaltungsgrenzen?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=app:landesgrenze&SRSNAME=urn:ogc:def:crs:EPSG::25833
  ```
  → 200 `application/gml+xml; version=3.2`, 198,873 B, 0.28 s (`OUTPUTFORMAT=application/geo%2Bjson`: 874,639 B, pretty-printed).
  - One feature (`fhh = FHH`), a MultiSurface of 4 polygons, 1 hole, 8517 vertices.
  - The server's 25833 coordinates equal my own 25832 → 25833 conversion of the OGC API copy within **1.4 mm** (median 0.8 mm).
- The same polygon is available from the OGC API: `https://api.hamburg.de/datasets/v1/verwaltungsgrenzen/collections/landesgrenze/items?f=json&crs=…/25832` (8517 vertices), and `alkis_vereinfacht/collections/VerwaltungsEinheit/items?art=Bundesland` (oid `DE02000000`, `aktualit 2026-09-23Z`, 8190 vertices). Both are 25832 only.
- Parts (25833 areas):
  - main area 748.9 km² with one **5,616 m² hole** (a Schleswig-Holstein enclave near Duvenstedt, 53.714°N 10.116°E);
  - **Neuwerk** 3.56 km²;
  - **Scharhörn/Nigehörn** 4.09 km²;
  - an **8,131 m² exclave** near Volksdorf (53.669°N 10.169°E).
- **25 m Douglas–Peucker simplification:** `probe/hamburg_simplified25m.geojson`. FeatureCollection, `crs` `urn:ogc:def:crs:EPSG::25833`, properties `name: Hamburg`, `source: LGV Hamburg WFS ALKIS Verwaltungsgrenzen (app:landesgrenze), dl-de/by-2-0`, `retrieved: 2026-11-07`, `simplified_m: 25`; MultiPolygon with integer coordinates.
  - **771 vertices, 13.5 KB** (the Berlin asset is 13.3 KB). All 4 polygons and the hole survive (main 715 + hole 5, Neuwerk 18, Scharhörn 28, exclave 5); no ring fell below 4 points.
  - The smallest parts (5-point exclave and hole) are within the tolerance of being dropped; keep or drop by taste.
- Neighbours are Schleswig-Holstein and Lower Saxony, neither of which is a freemapper region yet. No overlap with the existing assets.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf` → **307** → `hamburg-261106.osm.pbf` → 200, **54,130,091 B**, `Last-Modified: Sat, 07 Nov 2026 02:08:45 GMT`, ETag, `Accept-Ranges: bytes`.
- State: `https://download.geofabrik.de/europe/germany/hamburg-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4954
  ```
- Poly: `https://download.geofabrik.de/europe/germany/hamburg.poly`, **2 rings** (main + Neuwerk), lon 7.9678–10.3320, lat 53.3918–54.0626. The extract includes Neuwerk.
- Overpass needs a `User-Agent`: overpass-api.de answered **406** to curl's default UA.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `api.hamburg.de` OGC API | `limit` ≤ 10000 (silently clamped); `next` links; **no 25833**; errors are HTML unless `Accept: application/problem+json`; no ETag on items; **cold queries 11–23 s**, warm ≈1 s; gzip OK |
| `geodienste.hamburg.de` WMS (DOP, DGM) | Max 4096 px; oversize → **HTTP 200 `text/xml` exception**; `TIME` default = newest, a missing year inside the range → **blue fill image, HTTP 200**; DOP not queryable; DGM `image/tiff` = rendered RGB |
| `geodienste.hamburg.de` WFS (deegree) | GeoJSON/GML/CSV, 25833 OK; paging works although `ImplementsResultPaging FALSE`; cold 15 s; `numberMatched` only with `RESULTTYPE=hits` |
| `daten-hamburg.de` | bare host → **301** to `www.daten-hamburg.de` (follow redirects); HTTP/2; HEAD, ranges (`bytes=0-0` → 206), ETag, Last-Modified. Directory listing 403; old `geographie_geologie_geobasisdaten/…/DGM1/*` → 404 |
| `archiv.transparenz.hamburg.de` | snapshot files `…_<id>_snap_<n>.<ext>`; HEAD, ranges (206), ETag, Last-Modified. The URL changes when a package is republished; resolve via CKAN |
| `suche.transparenz.hamburg.de` CKAN | fast (0.3–1 s); one package per release with a numeric suffix; two packages can share a title and a modification time, so pick by content (resources), not only by the newest date |
| `metaver.de` | HTML pages OK; CSW `GetRecords` → 403 |

- No authentication anywhere, and no rate-limit or `Retry-After` headers. No throttling and no failed request (other than the deliberate error probes) in roughly 170 requests. No fair-use rules were found on the pages read (not exhaustive).
- The OGC API exposes an optional `access_token` parameter in its OpenAPI; it was not needed.

---

## 13. What makes the Hamburg adapter differ (risks first)

1. **ALKIS spellings differ from LGB although the API and field names are the same.** `nutzart` comes as ASCII class names ("Wohnbauflaeche", "Strassenverkehr", "Industrie Und Gewerbeflaeche") and `gebnutzbez` as "Gebaeude".
   - Without normalisation, `buildings::classify` drops **every** building ("not a building"), and the out-of-bounds (`PRIVATE_LAND_USES`) and paved rules (`paved.rs`) match nothing.
   - Fix in the Hamburg reader: map to the LGB spellings, or compare on a transliterated lower-case key (§8.5).
   - `bez`, `funktion` and `rellage` are proper German and need no change.
2. **No 25833 in the OGC API**, and freemapper's `alkis::collection_url` hard-codes `CRS_25833` for `bbox-crs`/`crs`.
   - Hamburg maps are 25832 (§3), so pass the map CRS (25832) to the OGC API;
   - or use the deegree WFS (GeoJSON, 25833 available) through the shared `alkis_wfs` path.
   - **Decision needed:** OGC API (LGB-like JSON and paging, 25832 only) vs WFS (25833, paging de facto).
3. **DOP tiles are clipped to districts and packed in 2–6 GB zips** (deflated members, IFD at the end, 4-channel JPEG, internal mask, float-noise tiepoints).
   - Exact tiles near a district border need two pieces merged by mask.
   - The **WMS returns the same pixels** on the native grid, with a `TIME` dimension, so the WMS is the practical path. It must always send an explicit open `TIME` (never 2023 leaf-off or 2022 leaf-on, which are closed-licence satellite images; a missing year returns a blue pattern).
4. **DGM1 exists only as a 1.36 GB statewide zip on a snapshot URL** (no WCS, no tile download; the original URL is 404).
   - It is feasible by HTTP range: read the 106 KB central directory once (cache it per ETag), then ≈2 MB per 1 km tile.
   - The URL must be rediscovered via CKAN when the snapshot id changes.
   - NoData is **−9999 in the pixels** while the tag says −3.4e38; tiles are clipped to the state boundary.
   - The vintage is 2022 (ALS); 4 years older than the DOP.
5. **nDSM is only the 1 m image-based bDOM 2022** (1.34 GB statewide zip, uncompressed Float32 members). There is no nDOM, LiDAR or 0.2 m surface model. It is four years older than the 2026 orthophoto.
6. **Two DOP seasons.** Leaf-off is annual (2025, 2026 open); leaf-on is every few years (2024). **Decision needed:** which season to default to (leaf-off recommended for orienteering).
7. **Portal versioning:** a new CKAN package per release; resource URLs point at archive snapshots; old service names are gone (`HH_WMS_DOP` 404). Pin service URLs (`geodienste…/wms_dop_zeitreihe_unbelaubt`, `api.hamburg.de/…/alkis_vereinfacht`), and resolve only the zips via CKAN.
8. **Neuwerk/Scharhörn** (≈120 km west of the city, in the Wadden Sea) are part of the state: in ALKIS, DGM1, bDOM and the boundary, but **not in the DOP time series WMS** (a separate "DOP Zeitreihe Wattenmeer" exists, not examined). The region's bbox spans E 461–588 km.
9. **Cold-start latency:** the OGC API and WFS can take 11–23 s for the first query of an area. Use generous timeouts (≥ 60 s) and one retry.
10. Unchanged from the other states: dl-de/by-2-0 (provider text differs), DHHN2016, DGM1 pixel registration (PixelIsArea at the NW corner, centres at +0.5), Float32, no login, and the Geofabrik layout (`hamburg-latest`, `hamburg-updates/state.txt`). 1 km tiles as in Brandenburg (not 2 km as in Saxony-Anhalt).

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Hamburg | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt`, layer `dop_zeitreihe_unbelaubt`, `CRS=EPSG:25832` on the native 0.2 m grid, ≤ 4096 px per request (stitched with the shared WMS stitching), explicit **`TIME=2026`** (an allow-list of open vintages: 2026, 2025, 2022, 2021, … leaf-off; 2024 leaf-on) | Currency: the flight date from the AdV metadata CSV (`DOP_unbelaubt_AdV-Metadaten_Hamburg.zip`, 1 MB, per tile; `published` = its ETag). Exact-tile fallback: the district zip by range + inflate + JPEG-RGBI decode + mask merge (heavy; not recommended). **Decision:** leaf-off (default) vs leaf-on. |
| **DTM** (hillshade, slope, contours) | **DGM1 2022** members from the statewide zip by HTTP range (`archiv.transparenz.hamburg.de/…/dgm1_hh_2022-04-30_258334_snap_1.zip`, URL resolved via CKAN), 1 km GeoTIFF Float32 LZW | Read and cache the central directory (106 KB) by ETag. NoData = −9999 (and the tag value). Round the tiepoints to mm. Currency from the CSV member (`Fortfuehrung 2022-03`). |
| **nDSM template** | **bDOM 2022 − DGM1 2022** at 1 m, the bDOM member from `www.daten-hamburg.de/opendata/Digitales_Hoehenmodell_bDOM/dom1_hh_2022-11-21.zip` by range; clamp < 0 | Image-based and 2022; no high-resolution option exists. Flag the age difference against the 2026 DOP. |
| **Buildings** | **OGC API** `…/alkis_vereinfacht/collections/GebaeudeBauwerk/items` with `bbox-crs`/`crs` = 25832, `limit=10000`, follow `next` | Reuse the LGB reader; **normalise `gebnutzbez` "Gebaeude" → "Gebäude"**; underground via `rellage` (also on Gebaeude/Tiefgarage). Alternative: WFS `ave:GebaeudeBauwerk` GeoJSON (25833 possible). |
| **Land use** (out-of-bounds, paved) | **OGC API `NutzungFlurstueck`** (parcel split, like LGB `nutzung_flurstueck`) | **Normalise `nutzart`** (table in §8.5). `bez` is complete enough for verges and paved squares; the school parcel is "Bildung und Wissenschaft". |
| **Region boundary** | WFS `HH_WFS_Verwaltungsgrenzen` `app:landesgrenze` in 25833, simplified 25 m | Ready in `probe/hamburg_simplified25m.geojson` (771 vertices, 4 polygons incl. Neuwerk/Scharhörn, 1 hole). Region CRS Utm32; oid prefix `DEHH`. |
| **OSM** | Geofabrik `europe/germany/hamburg-latest.osm.pbf` + `hamburg-updates/state.txt` | 54 MB; follow the 307 to `hamburg-YYMMDD.osm.pbf`. |
| **Attribution** | `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <year>, dl-de/by-2-0, Daten geändert` + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. Never with the 2023 leaf-off or 2022 leaf-on WMS vintages. |

---

## 15. Not verified, or open

- That the bDOM 2022 imagery is the March 2022 flight (inferred from the True-DOP 2022 vintage; the CSV says only `2022-11`).
- Pillow's CMYK inversion for the 4-channel JPEG tiles: that the stored values are inverted relative to Pillow's output, and that masked pixels are 0. This was inferred from the match with the WMS (mean diff 2.4), not checked with GDAL.
- The 2025 DOP tiles' TIFF layout (only the zip listing was read), and the leaf-on 2024 tile layout.
- Which DOP tiles are split between districts (only the school tile was checked). The re-dated 2026-10-27 zips (Hamburg-Mitte, Bergedorf, Harburg) suggest re-cuts.
- The WMS resampling method in 25833 and its JPEG quality; the effect of `MaxScaleDenominator 5000` (2 m/px still rendered).
- The "DOP Zeitreihe Wattenmeer" dataset (Neuwerk orthophotos) and its licence.
- OGC API behaviour above 10000 matches per page beyond `next` links (only 4441-feature sets tested). Stability of `offset` ordering without `sortby`.
- `nutzart` spellings of classes absent from the two urban samples (Wald, Landwirtschaft, Gehölz, Friedhof …). The pattern is inferred.
- INSPIRE BU/ELU WFS, LoD2, the 3D-Mesh, older XYZ DGM vintages and the DGM WMS `wms_gelaendemodelle_dgm` vs `HH_WMS_DGM1` differences were not examined.
- Rate limits or fair-use rules (none seen, none documented on the pages read).
- The Johanneum location was only located (OSM id and coordinates); no data was fetched there separately. It lies in the 2 × 2 km ALKIS sample area (E 564–566 km) only partly (its E 566.6 km is outside).

Network note: total transfer was roughly **90 MB**. The largest items were the two DOP 2026 tile members (35 MB), the WMS scale/size probes (≈25 MB), the ALKIS area samples (≈10 MB gzip), the DGM1/bDOM members (≈6 MB) and the metadata zips (1.2 MB). No full zip was downloaded. Disk usage under `hamburg/` is about 84 MB.
