# Saxony (GeoSN) data sources for freemapper — live-verified reference

Verified live on 2026-11-07 (Europe/Berlin). Provider: **Landesamt für Geobasisinformation Sachsen (GeoSN)**. This is the current name, shown in the site footer and in the WFS `ProviderName`; the former "Staatsbetrieb Geobasisinformation und Vermessung Sachsen" no longer appears.
Portal: <https://www.geodaten.sachsen.de/> ("Offene Geodaten"). Downloads need no login, no registration and no form.

Test location: Evangelisches Kreuzgymnasium Dresden (OSM way 23838018), centre E 415973 / N 5655745 (EPSG:25833),
footprint bbox ≈ E 415878–416069, N 5655655–5655834. This is 2 km tile **`33414_5654`** (Kachel id `4145654`). The school
straddles the tile edge E 416000, so a real project there needs tiles 414/5654 and 416/5654.
A second check used the Thomasschule Leipzig (E 316069 / N 5690555).

Samples (all under ``):

| Path | Content |
|---|---|
| `dl/dgm1/dgm1_33414_5654_2_sn_tiff.zip` (+ `x/` unpacked) | full DGM1 tile |
| `dl/dom1/dom1_33414_5654_2_sn_tiff.zip` (+ `x/`) | full DOM1 tile |
| `dl/dop20rgb/` | TFW, `_akt.csv` and the first 336 KB of the TIFF (read by HTTP range + partial inflate, no full download) |
| `dl/qdop20rgb/` | Quick-DOP `_akt.csv` and the TIFF header |
| `dl/wms/dop_school.jpg` | WMS GetMap 600×500 m at 0.2 m |
| `probe/gml_*.xml`, `probe/city_*.xml`, `probe/lpz_*.xml` | ALKIS WFS answers (school bbox, Dresden centre 1 km², Leipzig 2×2 km) |
| `probe/list_*.xml` | WebDAV listings of the GeoCloud shares (all tiles with size, Last-Modified, ETag) |
| `probe/products.json`, `probe/pd_app.json` | the portal's own product/share configuration |

---

## 1. Brandenburg → Saxony at a glance

| freemapper product | Brandenburg (today) | Saxony equivalent (recommended) | Status |
|---|---|---|---|
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip (a few MB) | **DOP20 RGB** uncompressed GeoTIFF + TFW, **2 km** zip, **≈230–270 MB per tile**; optionally the newer **Quick-DOP**; or **WMS** `sn_dop_020` for a clipped JPEG | verified |
| DTM (hillshade, slope, contours) | DGM1 GeoTIFF f32 + TFW, 1 km zip | **DGM1** GeoTIFF f32 LZW + TFW, 2 km zip, ≈14 MB | verified, full tile downloaded |
| nDSM template | nDOM LAZ (0.2 m grid as points), 1 km | **not openly available**: nDOM is "derzeit auf Antrag" and its share is empty; LSC (LAZ) is currently unavailable. Use **DOM1 − DGM1** (1 m) | verified (gaps) |
| Buildings | ALKIS OGC API Features `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `ave:GebaeudeBauwerk`, **GML 3.2 only** | verified |
| Land use (out-of-bounds, paved) | OGC API `nutzung_flurstueck` (`nutzart`, `bez`) | WFS `ave:NutzungFlurstueck`, the same schema (`nutzart`, `bez`) but `bez` is sparse | verified |
| Currency | `bb_*_aktualitaet.csv` per product | ArcGIS REST tile index (`Stand` per tile) + `*_akt.csv` inside every zip; ETag/Last-Modified for change detection | verified |
| OSM | Geofabrik `brandenburg-latest.osm.pbf` | Geofabrik `sachsen-latest.osm.pbf` (270 MB) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | dl-de/by-2-0, source note **"GeoSN"** (`Quelle: GeoSN, dl-de/by-2-0`) | verified |

---

## 2. Common infrastructure: GeoCloud shares, tile scheme, tile index

### 2.1 Download host (GeoCloud, a Nextcloud instance)

All tile products are files in **public Nextcloud link shares** on `geocloud.landesvermessung.sachsen.de`:

```
https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/<SHARE_TOKEN>/<FILE>
```

The portal builds the same URL in `createGeoCloudURL()` on <https://www.geodaten.sachsen.de/batch-download-4719.html>.
The legacy form `…/index.php/s/<TOKEN>/download?path=%2F&files=<FILE>` answers `303` with a redirect to the URL above.

HTTP behaviour, verified:

| Request | Result |
|---|---|
| `GET` (with or without `Range`) | `200`/`206`, `Content-Type: application/zip`, `ETag`, `OC-ETag`, `Last-Modified`, byte ranges supported (resumable) |
| `HEAD` without extra headers | **`401 Unauthorized`** (Sabre `NotAuthenticated`) |
| `HEAD` with **`X-Requested-With: XMLHttpRequest`** | `200`, with `Content-Length`, `Last-Modified`, `ETag` |
| `PROPFIND` (Depth 0/1) without that header | `401` |
| `PROPFIND` with `X-Requested-With: XMLHttpRequest` | `207`; Depth 1 on the share root lists **every tile** with size, Last-Modified and ETag (about 2 MB XML for about 5000 files, 2–4 s) |
| `Range: bytes=0-0` | **The server ignores the range and sends the whole file** (with status 206 and a full-length Content-Range). Use `bytes=0-1` or more. |
| Range near the end of a 267 MB file | works, but took about **15 s** (the server seeks slowly) |
| Missing file | `404`, Sabre XML `File with name /… could not be located` |
| Dead share token | `503` Sabre `ServiceUnavailable` / `NotFound`; the share page `index.php/s/<token>` says "Share not found" (404) |

Measured throughput is about **1–1.2 MB/s** (DGM1: 13 MB in 12.3 s; DOM1: 15 MB in 20.5 s; DOP: 70 MB in 60 s), so a 230 MB DOP tile takes about 3–4 minutes.
No rate-limit headers or throttling were seen. There is no authentication.

freemapper's `Fetcher::last_modified` sends `HEAD`. Against GeoCloud it must add `X-Requested-With: XMLHttpRequest`, or it will get a 401.

**Share tokens change.** The changelog (<https://www.geodaten.sachsen.de/changelog.xml>) reports "Änderung der Freigabelinks … Extern verwaltete Downloadlinks müssen entsprechend angepasst werden" on 2023-04-18, 2024-08-09 and 2025-09-09. Tokens must not be the only source of truth (see §2.4).

Current share tokens (2026-11-07):

| Product | Share token | Share name | File pattern | Files |
|---|---|---|---|---|
| DOP20 RGB | `QQFLq6nkoSnqB5g` | – | `dop20rgb_33EEE_NNNN_2_sn_tiff.zip` | 4967 |
| DOP20 RGBI | `sX3GPcdBMGrfXT9` | – | `dop20rgbi_33EEE_NNNN_2_sn_tiff.zip` | – |
| Quick-DOP RGB | `6MyPjbb3jzEHMfC` | – | `qdop20rgb_33EEE_NNNN_2_sn_tiff.zip` | 2453 |
| Quick-DOP RGBI | `gCRDjCCbZatfSJL` | – | `qdop20rgbi_33EEE_NNNN_2_sn_tiff.zip` | – |
| DGM1 | `JCcXyifaNdLDnxZ` | `DGM1_TIFF_2km` | `dgm1_33EEE_NNNN_2_sn_tiff.zip` | 4981 |
| DOM1 | `S6wwnFwX7882sZm` | – | `dom1_33EEE_NNNN_2_sn_tiff.zip` | 4981 |
| nDOM | `pcp2TnRYaCHmKN3` | `nDOM` | `ndom_33EEE_NNNN_2_sn_tiff.zip` (per app config) | **0, empty** |
| LSC (LAZ) | `EpkzyJHScGb5ndd` (app config) | `LSC` | `lsc_33EEE_NNNN_2_sn_laz.zip` | **0, empty** |
| LSC (LAZ) | `rqcqdt8QMcLFUvC` (tile index) | – | same pattern | **dead (503 / "Share not found")** |
| LoD2 Shape | `zFnkkAX9tmL2P7R` | – | `lod2_33EEE_NNNN_2_sn_shape.zip` | – |
| Hausumringe | `AcAqRn4k9Sz8ZZx` | – | `hu_sn_shape.zip` (statewide, 229,089,904 B) | 1 |
| Basis-DLM | `DtPWngtLEJP8K3k` | – | `basisdlm_sn_shape.zip` (1,274,435,077 B) / `basisdlm_sn_nas.zip` | 2 |

The tokens come from three portal sources, which agree for DOP, DGM1 and DOM1:
- `batchConfig.products` JSON, embedded inline in <https://www.geodaten.sachsen.de/batch-download-4719.html>; this source has no LSC or nDOM entry;
- `bundles["geobak-4x-productdownload"].ProductdownloadController.productsConfig` in <https://geoviewer.sachsen.de/mapviewer/resources/apps/produktdownload/app.json>, as `cloudId` + `pattern`;
- the `Download` field of the ArcGIS tile index (§2.3).

### 2.2 Tile scheme

- **2 km × 2 km tiles** for DOP, Quick-DOP, DGM1, DOM1, LSC, nDOM, LoD1/2 and DTK10. Brandenburg uses 1 km.
- CRS **EPSG:25833 statewide**, including the Vogtland west of 12°E (see §9). No product is offered in EPSG:25832.
- Name = product + `_33` + **E km (3 digits)** + `_` + **N km (4 digits)** + `_2_sn_` + format suffix, from the tile's **south-west corner**.
  Example: `dgm1_33414_5654_2_sn_tiff.zip` covers E 414000–416000, N 5654000–5656000.
  The separator is `_` (Brandenburg: `33EEE-NNNN`, with a hyphen).
- **SW corners are always even km.** In all listings, every one of the 4981 DGM1 E and N values is even. The portal derives the tile from a 1 km cell with `x = floor(E_km/2)*2`, `y = floor(N_km/2)*2`.
- Kachel id in the index and the `_akt.csv`: 7 digits `EEENNNN`, e.g. `4145654`. The FAQ example is `2805588` = E 33 280 km, N 5588 km.
- Tile SW corners in the listings: E 278…502 km, N 5560…5728 km. The grid therefore spans **E 278000–504000, N 5560000–5730000**. This matches the index layer `fullExtent` `{xmin 278000, ymin 5560000, xmax 504000, ymax 5730000}`.
- Border tiles exist where the state boundary plus a 200 m buffer reaches into them. Some border tiles are tiny, e.g. a 0.3 MB DOP or a 1 MB DGM1.
- Official grid files: <https://www.geodaten.sachsen.de/download/Shape_km2_33_UTM.zip> (also `…/GeoPackage_Kachelungen_33_UTM.zip`; links taken from the FAQ, not downloaded).

### 2.3 Tile index with per-tile currency (ArcGIS REST)

```
https://geodienste.sachsen.de/ags-relay/ArcGISServer/guest/arcgis/rest/services/geosn/rest_geosn_downloadlinks/MapServer
```

Layers: `0 ALKIS, 1 LSC, 2 LoD1, 3 LoD2, 4 DOM1, 5 DOM2, 6 DGM1, 7 DOP_RGB, 8 DTK25_Einzellayer, 9 DTK10_Einzellayer, 10 DGM2, 11 DTK10_Summenlayer, 12 DTK100_Einzellayer, 13 DTK100_Summenlayer, 14 DTK25_Summenlayer, 15 DTK50_Einzellayer, 16 DTK50_Summenlayer, 17 DOP_RGBI, 18 P10`.
The index has **no Quick-DOP layer and no nDOM layer.** Layer SR is 25833, `maxRecordCount` 2000, polygon geometry.
Fields: `Produkt`, `Kachel`, `Download` (full GeoCloud URL), `Stand` (alias "Aktualität", string), and `Boundingbox` (DOP, LSC).

Working example (DOP tile at the school):

```
GET https://geodienste.sachsen.de/ags-relay/ArcGISServer/guest/arcgis/rest/services/geosn/rest_geosn_downloadlinks/MapServer/7/query?f=json&geometry=415973,5655745&geometryType=esriGeometryPoint&inSR=25833&spatialRel=esriSpatialRelIntersects&outFields=*&returnGeometry=false
→ {"OBJECTID":3402,"Produkt":"DOP_RGB","Kachel":"4145654",
   "Download":"https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/QQFLq6nkoSnqB5g/dop20rgb_33414_5654_2_sn_tiff.zip",
   "Boundingbox":"414000 5654000 416000 5656000","Stand":"2024-03-19"}
```

The same point query on the other layers gave:
- DGM1 (6): `Stand 2024-11-30`.
- DOM1 (4): `2024-11-30`.
- LSC (1): `2024-11-30`, but its URL points at the dead share `rqcqdt8QMcLFUvC`.
- DOP_RGBI (17): `2024-03-19`.

Envelope queries work (`geometryType=esriGeometryEnvelope&geometry=xmin,ymin,xmax,ymax`, 0.4 s). `esriSpatialRelIntersects` **also returns tiles that only touch an edge**: the bbox 415700,5655500,416300,5656000 returned 4 tiles, including the `…_5656` row. Compute the tile set locally with freemapper's `tiles_covering` logic for 2 km tiles, and use the index only for currency and URLs.
`f=geojson` also works (`application/geo+json`). `Cache-Control: max-age=0,must-revalidate`.

The `Stand` format varies:
- one date: `2024-11-30`;
- several dates: `2025-04-03, 2025-04-04` or `2020-11-24, 2020-11-25, 2020-11-30`;
- a range (DGM2/DOM2 20 km tiles): `2024-11-27, 2024-11-30 bis 2024-12-28`.

To mirror LGB's "currency = revision else creation", take the **latest date** in the string.

Distribution of `Stand` values: DOP_RGB uses flights from 2024 and 2025 only. DGM1 and DOM1 use LiDAR flights from 2018 to 2025 (99 distinct `Stand` values).

### 2.4 Resolving URLs robustly

Recommended order:
1. Compute the 2 km tiles locally.
2. Query the tile index (one envelope query per product), which returns `Download` + `Stand`.
3. If the GeoCloud URL answers 404/503, rebuild it from `app.json` (`cloudId` + `pattern`, `$Kachel_Ost$`/`$Kachel_Nord$` = 3/4-digit km) and retry.
4. Keep the current tokens as a last fallback.

Treat a 503 Sabre error or a 404 "Share not found" page as "share moved", not as "tile missing".

---

## 3. Orthophoto

### 3.1 DOP20 RGB (official, tile download) — primary candidate

- Product page: <https://www.geodaten.sachsen.de/downloadbereich-dop-4826.html> ("Kacheln der Größe 2 km x 2 km … Zip-Verzeichnis mit einer GeoTIFF-Datei (Bilddaten) und einer TFW-Datei (Worldfile, nur bei RGB) sowie einer Beschreibungsdatei mit Aktualitätsinformationen").
- URL pattern: `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/QQFLq6nkoSnqB5g/dop20rgb_33EEE_NNNN_2_sn_tiff.zip`
- Working example: `…/QQFLq6nkoSnqB5g/dop20rgb_33414_5654_2_sn_tiff.zip` → `206`, **267,114,281 bytes**, `Last-Modified: Wed, 23 Apr 2025 08:20:39 GMT`, `ETag "68ad705eb4c1c"`.
- Zip members (deflate, classic zip32):

  | Member | Compressed | Uncompressed |
  |---|---|---|
  | `dop20rgb_33414_5654_2_sn.tif` | 267,113,725 | **300,013,320** |
  | `dop20rgb_33414_5654_2_sn.tfw` | 39 | 90 |
  | `dop20rgb_33414_5654_2_sn_akt.csv` | 91 | 107 |

- TIFF: 10000 × 10000 px, RGB 8-bit, **no TIFF compression** (Compression 1), strips of 13 rows, PlanarConfig 1. Software "Trimble Germany GmbH", ImageDescription "OrthoVistaProcessing ImageWriter".
  - GeoKeys: ProjectedCSType **25833**, `GTRasterType = 2` (PixelIsPoint), ModelTiepoint at (414000.1, 5655999.9), pixel 0.2 m.
  - The header's first-IFD pointer is at **offset 300,006,664 (end of file)**. A TIFF reader needs random access, so extract the member to disk or memory (300 MB) first. An IFD-like block also sits at offset 8; that is the one parsed above.
- TFW (centre of the top-left pixel, the same convention freemapper's `GridSpec::from_world_file` expects): `0.2 / 0 / 0 / -0.2 / 414000.1 / 5655999.9`, which gives the NW corner 414000.0 / 5656000.0.
- `_akt.csv` (CRLF line endings, `;`-separated):
  ```
  Kachelname;Ausdehnung (xmin ymin xmax ymax);Aktualitaet
  4145654;414000 5654000 416000 5656000;2024-03-19
  ```
- Share statistics: 4967 files, median **228.6 MB**, max 270.1 MB, **1067 GB** total.
  - Last-Modified 2025-04 for 2423 tiles: flight "Frühjahrsbefliegung 2024", released 2025-04-24.
  - Last-Modified 2026-05 for 2544 tiles: "Frühjahrsbefliegung 2025", released 2026-05-22.
  - Saxony flies half the state per year (2-year cycle).
- Compared with Brandenburg: about 50× more bytes per km² (uncompressed TIFF instead of JPEG), and 4 km² per tile. One schoolyard tile is about 230–270 MB to download and 300 MB to decode; a corner location needs up to 4 tiles (≈1 GB).

### 3.2 Quick-DOP (newer, preliminary)

- `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/6MyPjbb3jzEHMfC/qdop20rgb_33EEE_NNNN_2_sn_tiff.zip`
- Example: `…/qdop20rgb_33414_5654_2_sn_tiff.zip` → 254,337,359 bytes, `Last-Modified: Tue, 20 Oct 2026 14:17:50 GMT`, `ETag "6ad8758d5db1b"`.
- Members: `qdop20rgb_33414_5654_2_sn.tif` (300,013,336 B uncompressed) and `qdop20rgb_33414_5654_2_sn_akt.csv` with `Aktualitaet` **2026-05-25**. There is **no TFW.**
- TIFF: same layout (10000², RGB, uncompressed, IFD pointer at the end). Software "Trimble Inpho 15.1.2". `GTRasterType = 1` (PixelIsArea), tiepoint (414000.0, 5656000.0).
  Georeferencing has to come from the GeoTIFF tags or from the tile name.
- Coverage: 2453 tiles, E 360…502 km, N 5606…5716 km (the eastern half: Lose Dresden, Hoyerswerda, Bautzen, Riesa, Seiffen, Weißwasser, Görlitz, Freiberg). Released 2026-09-28 … 2026-11-06 ("Sommerbefliegung 2026").
- Portal note: "Quick-DOP sind nur verfügbar, wenn diese für das betroffene Gebiet vorliegen und die darauf basierenden DOP noch nicht freigegeben wurden." The share is therefore temporary, and Quick-DOP tiles disappear when the DOP is released.
- Not in the ArcGIS tile index. Currency only comes from `_akt.csv` inside the zip; a range read is possible (see §7) but slow.

### 3.3 DOP WMS (light alternative: clipped JPEG, no 230 MB tiles)

- `https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest` (WMS 1.3.0), layer **`sn_dop_020`**, info layer `sn_dop_020_info`.
  - CRS includes EPSG:25833 and 25832.
  - Formats: `image/jpeg`, `image/png`, `image/tiff`, among others.
  - **MaxWidth/MaxHeight 4096** px, so at most 819 m per request at 0.2 m.
  - Fees "kostenfrei"; AccessConstraints → <https://geoportal.sachsen.de/nutzungsbedingungen.html>.
- Working example (school, 600 × 500 m at 0.2 m):
  ```
  https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=sn_dop_020&STYLES=&CRS=EPSG:25833&BBOX=415700,5655500,416300,5656000&WIDTH=3000&HEIGHT=2500&FORMAT=image/jpeg
  ```
  → `200 image/jpeg`, 3000×2500, 2,290,109 bytes, 10 s. Visually checked: the school is in the image centre.
- Flight date by GetFeatureInfo on `sn_dop_020_info` (`INFO_FORMAT=text/plain`): `Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP`. The WMS therefore shows the **released DOP (2024)**, not the 2026 Quick-DOP.
- Drawbacks: the JPEG is re-encoded by the server (quality unknown); the currency is only a text string; bboxes larger than 819 m must be stitched; there is no ETag-based change detection.
- No WMS for Quick-DOP was found: `wms_geosn_quick-dop`, `wms_geosn_qdop`, `wms_geosn_quickdop` and `wms_geosn_rohdop` all return 404. This list is **not exhaustive**.

### 3.4 ATOM

The INSPIRE ATOM feeds exist but are useless for automation:
- Service feeds <https://geodownload.sachsen.de/inspire/oi_atom/Service_oi.xml> and <https://geodownload.sachsen.de/inspire/el_atom/Service_el.xml>.
- Their dataset feeds (`Dataset_oi_dop.xml`, `Dataset_el_dhm.xml`) contain **no per-tile entries**, only links to the HTML download pages.

---

## 4. DGM1 (terrain model, 1 m)

- Download page: <https://www.geodaten.sachsen.de/downloadbereich-digitale-hoehenmodelle-4851.html>. Its text: "LSC: LAZ; DGM1, DOM1: GeoTIFF; nDOM: GeoTIFF (derzeit auf Antrag, Download in Vorbereitung)", "ETRS89_UTM33 (EPSG: 25833)", "Deutsches Haupthöhennetz 2016 (DHHN2016) (EPSG: 7837)".
- URL: `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/JCcXyifaNdLDnxZ/dgm1_33EEE_NNNN_2_sn_tiff.zip`
- Example (fully downloaded): `…/dgm1_33414_5654_2_sn_tiff.zip` → 12,993,914 bytes, `Last-Modified: Wed, 30 Sep 2026 09:59:44 GMT`, `ETag "6abcddc6312ff"`.
- Members:

  | Member | Size |
  |---|---|
  | `dgm1_33414_5654_2_sn.tif` | 13,338,406 |
  | `dgm1_33414_5654_2_sn.tfw` | 90 |
  | `dgm1_33414_5654_2_sn.tif.aux.xml` | 385 (GDAL PAM statistics: min 109.50, max 121.53) |
  | `dgm1_33414_5654_2_sn_akt.csv` | 107; `4145654;414000 5654000 416000 5656000;2024-11-30` |

- TIFF:
  - 2000 × 2000 px, **Float32** (SampleFormat 3), **LZW** (Compression 5, Predictor 1), one row per strip, header IFD at offset 8;
  - `GTRasterType = 1` (PixelIsArea), tiepoint (414000.0, 5656000.0), pixel 1 m;
  - `GDAL_NODATA = -9999`;
  - no vertical GeoKey (the datum is documented as DHHN2016).
- TFW: `1 / 0 / 0 / -1 / 414000.5 / 5655999.5`, **the same convention as LGB DGM1** (freemapper's test fixture uses `421000.5 / 5821999.5`). The raster grid is identical to Brandenburg's; only the tiles are 2 km and 2000 px.
- Share: 4981 tiles, median 14.3 MB, max 15.9 MB, 66.5 GB total. All files show Last-Modified 2026-09, although the zip members are dated 2026-01-22 and `Stand` is 2024-11-30. **Last-Modified tracks re-uploads, not data currency.**
- Height reference: DHHN2016 (normal heights), the same system Brandenburg uses (Brandenburg: assumed, not re-checked here).
- History: GeoTIFF 2 km tiles statewide since 2025-12-10. The XYZ ASCII variant was dropped from the direct download on 2026-02-04. 1 km GeoTIFF tiles were used before; they are now superseded.
- DGM2 (20 km packages, `index.php/s/PG2K3kG7BwYOUs0/download?path=%2F&files=dgm2_4005640.zip`) is listed in the index but not needed.

---

## 5. nDOM, DOM1 and LiDAR (LSC)

### 5.1 nDOM: not openly available

- The download page says "nDOM: GeoTIFF (derzeit auf Antrag, Download in Vorbereitung)".
- The share `pcp2TnRYaCHmKN3` (name "nDOM") exists but is **empty**: the PROPFIND Depth 1 lists only the root, `quota-used-bytes 0`, Last-Modified 2025-12-03.
- `…/pcp2TnRYaCHmKN3/ndom_33414_5654_2_sn_tiff.zip` → 404.
- The pattern `ndom_33EEE_NNNN_2_sn_tiff.zip` comes from `app.json` and is **unverified** (no file exists yet).

### 5.2 LSC (classified laser points, LAZ): currently not downloadable

- Pattern from `app.json`: `lsc_33EEE_NNNN_2_sn_laz.zip`, a LAZ inside a zip (Brandenburg: plain `.laz`).
- The app-config share `EpkzyJHScGb5ndd` (name "LSC") is **empty** (Last-Modified 2026-10-20).
- The tile index still points to `rqcqdt8QMcLFUvC`, which returns **503** ("Share not found").
- LSC is not offered in the batch download.
- `Stand` values in the index: flights 2020-11 to 2023-12; the changelog has a "Los Dresden" update on 2026-01-26.
- The LAZ version, point format, classes, CRS VLR and file size could **not be verified**.

### 5.3 DOM1 (surface model, 1 m): available, the nDSM substitute

- URL: `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/S6wwnFwX7882sZm/dom1_33EEE_NNNN_2_sn_tiff.zip`
- Example (fully downloaded): `…/dom1_33414_5654_2_sn_tiff.zip` → 15,033,535 bytes, `Last-Modified: Fri, 23 Jan 2026 09:19:28 GMT`, `ETag "6973732ccf6ed"`.
- Same layout as DGM1: tif (15,604,390 B, Float32 LZW 2000², PixelIsArea, NoData -9999), tfw, aux.xml, `_akt.csv` with `2024-11-30`.
- Share: 4981 tiles, median 14.8 MB, 68.6 GB total.
- **DOM1 − DGM1** on the test tile over the school (±50 m): n = 48160 px, min −3.41, p10 0.00, median 4.27, p90 17.15, max 36.01 m; 57 % of pixels are > 2 m.
  The grids align pixel for pixel (same tiepoint). The result is a usable nDSM, but at **1 m** instead of Brandenburg's 0.2 m, so hedges, walls and small equipment are blurred or lost.
  Small negative values occur and should be clamped at 0.

---

## 6. ALKIS: buildings and actual land use ("tatsächliche Nutzung")

ALKIS **is open** in Saxony: the AdV "ALKIS vereinfacht 2.0" schema, the same one behind Brandenburg's `alkis-vereinfacht` OGC API. Attribute names match.

### 6.1 Service

- Endpoint: **`https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs`** (XtraServer). The portal page is <https://www.geodaten.sachsen.de/alkis-wfs-4968.html>; it says the data is updated weekly. The interface description is <https://geoportal.sachsen.de/portal/dokumente/WFS_ALKIS_Anwendungsbeschreibung.pdf> (not read).
- `GetCapabilities` (WFS 2.0.0, 1.1.0 also accepted): `ows:Title` "WFS SN ALKIS Vereinfacht"; Fees "Der Dienst ist kostenfrei unter Angabe der Quelle „Landesamt für Geobasisinformation Sachsen (GeoSN)“ oder kurz "GeoSN" … nutzbar."; AccessConstraints none.
- Feature types (namespace `ave` = `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0`):
  `ave:Flurstueck`, `ave:FlurstueckPunkt`, **`ave:GebaeudeBauwerk`**, `ave:KatasterBezirk`, `ave:Nutzung`, **`ave:NutzungFlurstueck`**, `ave:VerwaltungsEinheit`.
- CRS: Default `urn:ogc:def:crs:EPSG::25833`; Other `EPSG::25832`, `EPSG::4258`.
  Axis order in 25833 is **E N**, both in `BBOX` and in `gml:posList` (verified). In 4258 it would be lat/lon (not tested).
- Output formats: `application/gml+xml; version=3.2` (default), GML 3.1.1, and `application/x-zip-shapefile` (works, returns a zip).
  **There is no GeoJSON**: `OUTPUTFORMAT=application/json` and `application/geo+json` return **400 with an HTML error page**.
- Limits: `CountDefault` **20000** features per request; the abstract says "bis maximal 20.000 Objekte". `ImplementsResultPaging = FALSE` and there are **no `next` links**.
  `COUNT`/`STARTINDEX` nevertheless work in practice: `COUNT=3&STARTINDEX=3` returned the next three oids.
  `RESULTTYPE=hits` gives `numberMatched` fast (0.5 s). For example, 8 × 8 km of central Leipzig has 47,138 buildings, which is over the cap. The behaviour above the cap (silent truncation or exception) was **not tested**.
- Geometry: always `gml:MultiSurface/surfaceMember/Polygon/exterior|interior/LinearRing/posList`, 2D, `srsDimension="2"`, coordinates with mm precision. No arcs or curves were seen in 2477 + 2729 features.
  Features are **not clipped** to the bbox: the returned envelope 415777.883 5655572.608 – 416192.975 5655938.59 is larger than the requested bbox.
- `DescribeFeatureType` (attributes; `minOccurs=0` unless marked required):
  - `GebaeudeBauwerk`: `oid`, `aktualit` (xs:date), `geometrie`, `gebnutzbez`, `funktion`, `gfkzshh`, `rellage`, `name`, `anzahlgs`, `gmdschl`, `lagebeztxt`.
  - `Nutzung`: `oid`, `aktualit`, `geometrie`, `nutzart`, `bez`, `name`.
  - `NutzungFlurstueck`: as `Nutzung` plus the required `flstkennz`.

Working example (school bbox):

```
https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=415800,5655600,416150,5655900,urn:ogc:def:crs:EPSG::25833
```

| Request | Features | Size | Time |
|---|---|---|---|
| School bbox (350 × 300 m), `GebaeudeBauwerk` | 123 | 136,732 B | 7.7 s |
| School bbox, `NutzungFlurstueck` | 136 | 138,510 B | 6.8 s |
| School bbox, `Nutzung` | 36 | 43,760 B | 34.6 s (slow) |
| Dresden centre 1 km², buildings | 243 | 448 KB | 19.8 s |
| Dresden centre 1 km², land use | 700 | 1.2 MB | 27.6 s |
| Leipzig 2 × 2 km, buildings | 2477 | 2.8 MB | 2.9 s |
| Leipzig 2 × 2 km, land use | 2729 | 3.7 MB | 3.7 s |

Response times vary a lot.

### 6.2 Sample features (school bbox, posList shortened)

```xml
<GebaeudeBauwerk gml:id="DESNALK0CrM0003UBL">
  <gml:identifier codeSpace="urn:adv:oid:">urn:adv:oid:DESNALK0CrM0003UBL</gml:identifier>
  <oid>DESNALK0CrM0003UBL</oid>
  <aktualit>2024-01-04Z</aktualit>
  <geometrie><gml:MultiSurface srsName="urn:ogc:def:crs:EPSG::25833" srsDimension="2"><gml:surfaceMember><gml:Polygon>
    <gml:exterior><gml:LinearRing><gml:posList>416159.372 5655892.09 416156.502 5655888.675 …</gml:posList></gml:LinearRing></gml:exterior>
  </gml:Polygon></gml:surfaceMember></gml:MultiSurface></geometrie>
  <gebnutzbez>Gebäude</gebnutzbez>
  <funktion>Allgemein bildende Schule</funktion>
  <gfkzshh>31001_3021</gfkzshh>
  <gmdschl>14612000</gmdschl>
  <lagebeztxt>Wägnerstraße 26</lagebeztxt>
</GebaeudeBauwerk>

<NutzungFlurstueck gml:id="DESNALK090L00003DESNALK0q8002P3cTF">
  <oid>DESNALK090L00003DESNALK0q8002P3cTF</oid>
  <aktualit>2020-08-12Z</aktualit>
  <geometrie>…MultiSurface…</geometrie>
  <nutzart>Fläche besonderer funktionaler Prägung</nutzart>     <!-- the school parcel: no bez -->
  <flstkennz>140260___00225000o00</flstkennz>
</NutzungFlurstueck>

<NutzungFlurstueck gml:id="DESNALK06uT000GKDESNALK0q8002O4sTF">
  <oid>DESNALK06uT000GKDESNALK0q8002O4sTF</oid>
  <aktualit>2019-05-20Z</aktualit>
  <nutzart>Industrie- und Gewerbefläche</nutzart>
  <bez>Handel und Dienstleistung</bez>
  <flstkennz>140260___00211000201</flstkennz>
</NutzungFlurstueck>
```

Format notes:
- `aktualit` is `YYYY-MM-DDZ`, while Brandenburg uses `YYYY-MM-DDThh:mm:ssZ`. freemapper's `get(..10)` parse works for both.
- `GebaeudeBauwerk.oid` has 18 characters (`DESNALK…BL`).
- A `NutzungFlurstueck.oid` has 34 characters: the Nutzung oid (16) followed by the Flurstück oid (18). It is still unique, so dedup by oid works.

### 6.3 Observed values

Buildings (Dresden school bbox, Dresden centre 1 km², Leipzig 2×2 km, plus filtered counts over 16 × 16 km of Dresden, E 404000–420000, N 5648000–5664000):

- `gebnutzbez`: **always `Gebäude`.** `gebnutzbez=Bauteil` → 0 hits; `gebnutzbez != Gebäude` → 0 hits in 8 × 8 km of Dresden.
- `rellage`: **never set.** 0 hits for `NOT rellage IS NULL` over 16 × 16 km of Dresden. Brandenburg's underground test (`rellage = "Unter der Erdoberfläche"`) will never fire.
  Whether underground garages are missing or delivered as ordinary `Gebäude` is **unverified**. `funktion=Tiefgarage` → 0 hits.
  A Leipzig `Gebäude zum Parken` at "Burgplatz 6" could be the underground garage under Burgplatz (unverified).
- `anzahlgs`: not seen.
- `funktion` values seen:
  - Wohngebäude, Garage, Gemischt genutztes Gebäude mit Wohnen, Gebäude für Handel und Dienstleistungen, Gebäude für Gewerbe und Industrie;
  - **Allgemein bildende Schule**, Sport-, Turnhalle, Gebäude zur Versorgung, Gebäude zum Parken, Gebäude für Beherbergung, Gebäude für Bewirtung;
  - Museum, Gebäude für kulturelle Zwecke, Verwaltungsgebäude, Kirche, Theater, Oper, Schloss, Gericht, Hochschulgebäude (Fachhochschule, Universität), Polizei, Rathaus, Feuerwehr;
  - Kinderkrippe, Kindergarten, Kindertagesstätte; Gebäude für Bildung und Forschung; Nach Quellenlage nicht zu spezifizieren;
  - Leipzig often uses generic values: "Gebäude für öffentliche Zwecke" (188×), "Gebäude für Wirtschaft oder Gewerbe" (326×);
  - multi-valued, `;`-joined: **`Schloss;Museum`**.

Land use (`NutzungFlurstueck`; `Nutzung` gives the same values without the parcel split):

- `nutzart` seen:
  - **Wohnbaufläche, Straßenverkehr, Weg, Platz, Bahnverkehr, Industrie- und Gewerbefläche, Fläche gemischter Nutzung** (81 `Nutzung` objects in 16 × 16 km of Dresden);
  - Fläche besonderer funktionaler Prägung, Sport-, Freizeit- und Erholungsfläche, Fließgewässer, Stehendes Gewässer, Landwirtschaft, Wald, Gehölz, Unland/Vegetationslose Fläche.
  - All four values in freemapper's `PRIVATE_LAND_USES` occur with Brandenburg's spelling.
- `bez` seen **only** as:
  - `Handel und Dienstleistung`, `Industrie und Gewerbe`, `Versorgungsanlage`, `Entsorgung` (on Industrie- und Gewerbefläche);
  - `Grünanlage` (on Sport-, Freizeit- und Erholungsfläche).
- Counted over 16 × 16 km of Dresden (`Nutzung`):
  - `Platz` total 428, `Platz` with any `bez` **0**, `bez=Parkplatz` **0**;
  - `bez=Begleitfläche Straßenverkehr` **0**; `Straßenverkehr` with `bez` **0**;
  - `Fläche besonderer funktionaler Prägung` with `bez` **0**; `Bahnverkehr` 201.
- Effect on freemapper's `build/paved.rs`:
  - `("Platz", None) => true` makes **every Platz paved** (Festplatz, grass squares included);
  - `("Straßenverkehr", Some("Begleitfläche Straßenverkehr")) => continue` never fires, so **verges and tree strips count as carriageway**;
  - `PAVED_SQUARES` never matches.
  - Brandenburg's school parcel `bez` "Bildung und Wissenschaft" has no Saxony counterpart.

### 6.4 Open substitutes (not needed while the WFS works)

- **Hausumringe**: statewide Shape only. `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/AcAqRn4k9Sz8ZZx/hu_sn_shape.zip` → 200, 229,089,904 B, Last-Modified 30 Oct 2026. Quarterly updates ("Stand 22.10.2026"). It has no function attribute of freemapper's kind.
- **Basis-DLM** (ATKIS): statewide Shape/NAS, `…/DtPWngtLEJP8K3k/basisdlm_sn_shape.zip` → 200, 1,274,435,077 B, Last-Modified 04 Nov 2026. Quarterly; generalised (1:25,000 accuracy). Unsuitable for parcel-exact out-of-bounds areas.
- **ALKIS NAS** per municipality, with point information: `…/GsgZjSdwH45x8PJ/<Gemeindeschlüssel>.xml.zip` (pattern from the batch config, not downloaded). This is the full NAS, too heavy for freemapper.
- **LoD2** per 2 km tile (Shape/CityGML/DXF): `…/zFnkkAX9tmL2P7R/lod2_33414_5654_2_sn_shape.zip` → 206, 2,567,712 B, Last-Modified 05 Aug 2025. Footprints come from ATKIS and carry heights. It could replace buildings if the WFS fails (content not inspected).

---

## 7. Data currency ("Aktualität")

| Mechanism | Products | How |
|---|---|---|
| ArcGIS tile index `Stand` | DOP_RGB, DOP_RGBI, DGM1, DOM1, LSC, LoD1/2, DTK, P10 | one envelope query per product (§2.3); string with one or more dates. Matches `_akt.csv` (DOP 2024-03-19, DGM1/DOM1 2024-11-30 verified) |
| `<name>_akt.csv` inside each zip | DOP, Quick-DOP, DGM1, DOM1 (and presumably nDOM/LSC) | `Kachelname;Ausdehnung (xmin ymin xmax ymax);Aktualitaet`, CRLF, one row. Only reachable after download, or by a remote zip range read (≈4–60 s because the tail range is slow) |
| `ETag` / `Last-Modified` | every GeoCloud file | via `GET`, or `HEAD`/`PROPFIND` with `X-Requested-With: XMLHttpRequest`. **Upload date, not flight date** (DGM1: Stand 2024-11-30, members 2026-01-22, Last-Modified 2026-09-30). Good as the "published" marker for freemapper's "changed upstream" warning |
| PROPFIND listing of a share | all tiles of a product | one request (≈2 MB) returns size, Last-Modified and ETag of every tile, the equivalent of LGB's CSV `publicationdate` |
| WMS GetFeatureInfo `sn_dop_020_info` | WMS orthophoto | text such as `Datum: 19.03. + 07.04.2024` |
| ALKIS `aktualit` | per feature | as in Brandenburg |
| Changelog Atom | all products | <https://www.geodaten.sachsen.de/changelog.xml> (326 entries; release notes per product) |

A suggested freemapper mapping: `currency` = latest date in `Stand` (or `_akt.csv`); `published` = Last-Modified or ETag of the zip.

---

## 8. Licence and attribution

- Terms of use: <https://www.landesvermessung.sachsen.de/allgemeine-nutzungsbedingungen-8954.html>: "Die Erlaubnis entspricht dem Lizenzmuster der **Datenlizenz Deutschland – Namensnennung – Version 2.0** unter Angabe des Quellenvermerks **"Landesamt für Geobasisinformation Sachsen (GeoSN)" oder kurz "GeoSN"**." Download and use are free of charge.
- Official examples on that page:
  - "Elektronische Medien: **Quelle: GeoSN, dl-de/by-2-0**, [optional: URI des Datensatzes]"
  - "Druckmedien: **Quelle: GeoSN, dl-de/by-2-0**, [optional: Produktname (DTK, DGM)]"
- The WFS capabilities say "kostenfrei unter Angabe der Quelle „Landesamt für Geobasisinformation Sachsen (GeoSN)“ oder kurz "GeoSN"".
- The INSPIRE ATOM metadata says "Namensnennung: Geodaten Sachsen". This is an older wording; prefer the terms-of-use page.
- **Recommended freemapper line** (analogous to Brandenburg's): `Quelle: GeoSN, dl-de/by-2-0, Daten geändert`. Optionally add the year and the dataset URI, e.g. `Quelle: GeoSN <year>, dl-de/by-2-0 (https://www.govdata.de/dl-de/by-2-0), Daten geändert`. dl-de/by-2-0 requires marking changes.
- Restriction on the same page: "Die Nutzung der **Präsentationsausgaben** aus dem Liegenschaftskataster ist auf den eigenen Gebrauch beschränkt". This concerns rendered cadastral map products, not the WFS vector data freemapper converts. Owner data is never included in "vereinfacht". This is not legal advice; it is worth a one-line confirmation by mail to geodaten@geosn.sachsen.de if in doubt.

---

## 9. CRS and extent

- All raster tiles are EPSG:25833 (GeoKey ProjectedCSType 25833). The ALKIS WFS defaults to 25833 and can also deliver 25832 and 4258.
- The FAQ says: "Die bereitgestellten Geodaten werden … im amtlichen Koordinatenreferenzsystem ETRS89_UTM33 (EPSG: 25833) bereitgestellt."
- State boundary (`ave:VerwaltungsEinheit` `DE14` "Freistaat Sachsen", 76,402 vertices, `aktualit` 2026-10-27):
  - **EPSG:25833**: E 278330 – 502930, N 5561084 – 5728200
  - **EPSG:4258**: lon 11.87148 – 15.04201, lat 50.17130 – 51.68497
  - **EPSG:25832**: E 703461 – 921300, N 5562993 – 5732999 (for information only; nothing is delivered in 25832)
  - 5377 boundary vertices lie west of 12°E (Vogtland, strictly UTM zone 32), but the products still use zone 33. Easting there is about 278 km, so freemapper's zone-33 maths works unchanged.
- Tile-grid coverage box: **E 278000–504000, N 5560000–5730000** (2 km grid).
- freemapper's `LGB_COVERAGE` (E 250000–500000, N 5650000–6000000) overlaps northern Saxony (N 5650000–5730000) and does not reach the eastern edge (E > 500000). Adapter choice therefore needs a state polygon or per-adapter coverage, not one bbox. Near the Lausitz border both states apply.

---

## 10. OSM (Geofabrik Saxony)

- Extract: `https://download.geofabrik.de/europe/germany/sachsen-latest.osm.pbf` → **307** redirect to the dated file `sachsen-261106.osm.pbf` → 200, **269,824,406 bytes** (Geofabrik page: "257 MB"), `Last-Modified: Sat, 07 Nov 2026 02:15:03 GMT`.
  For comparison, `brandenburg-latest.osm.pbf` → `brandenburg-261106.osm.pbf`, 296,984,244 B.
- State: `https://download.geofabrik.de/europe/germany/sachsen-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4956
  ```
  This is the same format freemapper already parses.
- Poly: `https://download.geofabrik.de/europe/germany/sachsen.poly` (buffered: lon 11.8668–15.0508, lat 50.1666–51.6874).
- The redirect is new compared with what the current code expects. ureq follows redirects by default, but the "about 300 MB" message and the cache naming (`brandenburg_<date>.osm.pbf`) need a region parameter.

---

## 11. What makes the Saxony adapter differ from Brandenburg (risks first)

1. **Orthophoto volume.** 2 km uncompressed GeoTIFF in zip, 230–270 MB per tile, against a few MB of JPEG per 1 km tile in Brandenburg.
   - A single map can mean 0.25–1 GB of download (≈4 min per tile at ≈1 MB/s) and 300 MB of decode memory per tile.
   - The TIFF's first IFD sits at the end of the file, so it cannot be stream-decoded from the zip.
   - The decoder needs uncompressed RGB strips (the `tiff` crate is already a dependency). The template must be re-encoded to JPEG.
   - The WMS (§3.3) is the escape hatch; it gives up exact per-tile currency.
2. **No open high-resolution nDSM.** nDOM is "auf Antrag" and its share is empty; LSC is unavailable (empty share, dead index link). The fallback is DOM1 − DGM1 at 1 m, against Brandenburg's 0.2 m.
3. **GeoCloud HTTP quirks.**
   - `HEAD` and `PROPFIND` return 401 unless `X-Requested-With: XMLHttpRequest` is sent; freemapper's `last_modified()` must add it.
   - `Range: bytes=0-0` returns the whole file.
   - Tail ranges are slow.
4. **Share tokens rotate** (2023, 2024 and 2025 so far), and the tile index can hold stale links (LSC). Resolve URLs at runtime (§2.4) and treat 404/503 as "re-resolve".
5. **ALKIS WFS instead of OGC API.**
   - GML 3.2 only (no GeoJSON), so a small GML subset parser is needed (MultiSurface/Polygon/LinearRing/posList, 2D) or the zipped shapefile.
   - No `next` links; cap of 20,000 features per request; errors arrive as HTML with status 400.
   - Split large areas into ≤1 km cells, or use `COUNT`+`STARTINDEX`, which works but is not advertised. Check `RESULTTYPE=hits` first.
6. **ALKIS content gaps.**
   - No `rellage`, so freemapper cannot drop underground buildings; no `Bauteil`.
   - `bez` is missing on `Platz`, `Straßenverkehr` and `Fläche besonderer funktionaler Prägung`. The paved-area rules degrade: all squares become paved and verges are not excluded. Combine with OSM `surface`/`landuse` or the orthophoto, or mark these as candidates.
   - `funktion` is sometimes generic, and multi-valued with `;`.
7. **Tile id type.** 2 km, even SW km, `_`-separated names with a product prefix and the `_2_sn_<fmt>` suffix; the 7-digit Kachel id `EEENNNN`. Brandenburg: `33EEE-NNNN` at 1 km.
8. **Currency format.** A string with several dates, sometimes with "bis". Last-Modified is an upload date, not the flight date. Quick-DOP has no index entry.
9. **Two orthophoto generations** (DOP and the temporary Quick-DOP). Quick-DOP has no TFW and a different `GTRasterType` (PixelIsArea vs PixelIsPoint), although both describe the same grid.
10. **Coverage check.** It needs a Saxony box (E 278000–504000, N 5560000–5730000) or polygon. This box overlaps the current LGB box, and the east edge is beyond E 500000.
11. Unchanged from Brandenburg: EPSG:25833, DHHN2016 heights, dl-de/by-2-0, no login, world-file convention (TFW = centre of the top-left pixel), DGM1 grid registration (`E0+0.5`).

---

## 12. Recommendation per freemapper product

| freemapper product | Use in Saxony | Notes |
|---|---|---|
| **Orthophoto template** | **DOP20 RGB tiles** `…/QQFLq6nkoSnqB5g/dop20rgb_33EEE_NNNN_2_sn_tiff.zip` (TFW inside), currency from index layer 7 `Stand`, published = ETag/Last-Modified | Mirrors the Brandenburg model (cacheable, exact pixels, per-tile currency). Optionally prefer **Quick-DOP** (`6MyPjbb3jzEHMfC`, `_akt.csv` date) when it is newer and present, labelled as preliminary. Offer the **WMS** `sn_dop_020` (≤4096 px per request, stitched) as a fast mode or fallback when the tile download is too heavy; record its GetFeatureInfo date. **Decide** the default with the user: tiles for fidelity, WMS for speed. |
| **DTM** (hillshade, slope, contours) | **DGM1** `…/JCcXyifaNdLDnxZ/dgm1_33EEE_NNNN_2_sn_tiff.zip` | Float32 LZW, TFW identical in convention to Brandenburg, NoData −9999, DHHN2016; index layer 6 `Stand`. |
| **nDSM template** | **DOM1 − DGM1** from `…/S6wwnFwX7882sZm/dom1_33EEE_NNNN_2_sn_tiff.zip` + DGM1 (clamp < 0 to 0) | 1 m only. Switch to nDOM (`pcp2TnRYaCHmKN3`, `ndom_…_2_sn_tiff.zip`) or LSC LAZ once they are published; the changelog Atom will announce it. |
| **Buildings** | **ALKIS WFS** `ave:GebaeudeBauwerk`, GML 3.2, `SRSNAME`/`BBOX` in `urn:ogc:def:crs:EPSG::25833` | Same attributes as Brandenburg (`oid`, `aktualit`, `gebnutzbez`, `funktion`). Underground detection is impossible (no `rellage`). LoD2 tiles are the fallback. |
| **Land use** (out-of-bounds, paved) | **ALKIS WFS** `ave:NutzungFlurstueck` (or `ave:Nutzung` without the parcel split; slower in the test) | `nutzart` values match Brandenburg. Adapt the paved rules for the missing `bez` (Platz, verges). |
| **OSM** | Geofabrik `europe/germany/sachsen-latest.osm.pbf` + `sachsen-updates/state.txt` | 270 MB; follows the 307 redirect to `sachsen-YYMMDD.osm.pbf`. |
| **Attribution** | `Quelle: GeoSN, dl-de/by-2-0, Daten geändert` + `© OpenStreetMap contributors, ODbL` + freemapper credit line | See §8. |

---

## 13. Not verified, or open

- LSC LAZ content (LAS version, point format, classes, CRS VLR, tile size): no file is downloadable at the moment.
- nDOM format: the share is empty; only the name pattern from `app.json` is known.
- Whether underground structures (Tiefgaragen) are absent from `GebaeudeBauwerk` or present as plain `Gebäude`.
- Whether the missing `bez`/`rellage` hold statewide. Checked only in Dresden (16 × 16 km filtered counts, 1 km² full) and Leipzig (2 × 2 km full).
- WFS behaviour above 20,000 matches (truncation vs exception); stability of `STARTINDEX` ordering.
- Whether the IFD at the end of the DOP TIFF equals the IFD at offset 8 (only the latter was parsed, from a partial inflate).
- JPEG quality and exact resampling of the WMS; whether a Quick-DOP WMS exists under another name.
- Rate limits or fair-use rules for GeoCloud and the WFS (none seen; none documented on the pages read).
- `WFS_ALKIS_Anwendungsbeschreibung.pdf` was not read.
- The tile-grid files (`Shape_km2_33_UTM.zip`, GeoPackage) were not downloaded.

Network note: because of the `Range: bytes=0-0` quirk (§2.1), several probes transferred whole files to memory or `/dev/null`. Total network transfer was roughly 0.7–0.9 GB. Disk usage under `sachsen/` is about 81 MB.
