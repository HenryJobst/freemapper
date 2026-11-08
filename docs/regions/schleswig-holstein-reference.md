# Schleswig-Holstein (LVermGeo SH) data sources for freemapper — live-verified reference

Verified live on 2026-11-08 (Europe/Berlin). Provider: **Landesamt für Vermessung und Geoinformation Schleswig-Holstein
(LVermGeo SH)**, Kiel. Open data is catalogued in the state CKAN portal `opendata.schleswig-holstein.de` (organisation
`lvermgeo`, 578 packages, almost all CC BY 4.0) and harvested from the metadata system SH-MIS. The data sit on three hosts,
all run by Dataport:

- the **Downloadportal** `geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/` (PHP; 1 km tiles of DOP20,
  DGM1, bDOM and ALKIS NAS per Flur, plus a statewide tile index per product as GeoJSON);
- the services host `dienste.gdi-sh.de` (DOP WMS, ALKIS WFS, ATOM feeds for ALKIS);
- the services host `service.gdi-sh.de` (metadata WMS per tile, INSPIRE services, **INSPIRE ATOM with the DGM1 as
  GeoTIFF**).

No login, registration, token or form is needed anywhere. **One host is a TLS trap:** `geodaten.schleswig-holstein.de`
does not send its intermediate certificate (§12). freemapper (ureq + rustls + webpki-roots) will fail there unless the
intermediate is bundled.

Test locations:

| School | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile |
|---|---|---|---|---|---|---|
| **Coppernicus-Gymnasium Norderstedt**, Coppernicusstraße 1 (primary; 1.07 km from the Hamburg border) | way **26986780** | E 565214 / N 5949250 | E 169108 / N 5960473 | 53.68789 / 9.98753 | E 565115–565312, N 5949136–5949365 | `32 565 5949` (fully inside) |
| Kieler Gelehrtenschule, Feldstraße 19, Kiel (currency/WMS only) | way **18616769** | E 573992 / N 6020961 | E 183946 / N 6031270 | 54.33113 / 10.13787 | E 573854–574130, N 6020851–6021071 | `32 573 6020` |
| Border probe: Norderstedt-Garstedt / Hamburg-Langenhorn | – | tile `32 565 5947` (52 % Hamburg), bbox E 565000–566000, N 5946500–5948000 | | ≈53.67 / 9.99 | | |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7°. The ALKIS main school building
`DESHPDHK00019nnABL` came back at E 565146–565217 / N 5949185–5949246 in 25832, consistent with OSM. The VwG WFS's own
25833 output matches my 25832 → 25833 conversion within **0.74 mm**, see §8.) The 2 × 2 km area used for ALKIS samples is
E 564214–566214 / N 5948250–5950250 (25832), i.e. tiles 564–565 × 5948–5949 plus edges. Other candidates checked for the
border distance: Grundschule Am Hagen, Ahrensburg 0.72 km; Schulzentrum Süd, Norderstedt 0.92 km; Sachsenwaldschule Reinbek
1.86 km; Johann-Rist-Gymnasium Wedel 2.36 km.

Samples (all under `sh/`; treat as untrusted data):

| Path | Content |
|---|---|
| `dl/dop/dop20rgbi_32_565_5949_1_sh_2025.tif` | the school's full 1 km DOP20 tile (RGBI, uncompressed, 104.9 MB), flight 17.03.2025 |
| `dl/dgm1/dgm1_32_565_5949_1_sh_2022.bin`, `dgm1_32_565_5947_1_sh_2022.bin` | DGM1 school tile and the border tile as **XYZ ASCII** (28 MB each) from the Downloadportal |
| `dl/dgm1tif/dgm1_32_565_5949_1_sh_2022.tif` | the same DGM1 tile as **GeoTIFF** from the INSPIRE ATOM data directory (1.48 MB) |
| `dl/bdom/bdom20nc_32_565_5949_1_sh_2023.tif` | bDOM 0.2 m school tile (Float32, uncompressed, 104.9 MB), flight 07.04.2023 |
| `dl/hhcmp/dgm1_32_565_5947_1_hh_2022.tif` | Hamburg's DGM1 for the shared border tile (range-read from the LGV zip) |
| `dl/massen/{DGM1_SH__,DOP20_SH__,bDOM_SH_}Massendownload.geojson` | statewide tile indexes (date + download link per 1 km tile); `ALKIS_SH_Massendownload_Norderstedt_excerpt.geojson` (56 of 16,885 Flur entries; the 243 MB original was deleted) |
| `dl/wms/` | DOP WMS GetMaps: school in 25832 and **25833**, native-grid PNG vs tile (`cmp65.png`), 4000/5000/5001 px probes, 2 × 3 km border image (`cmp_border.jpg`), deep-Hamburg probes, historic layers (`cmp_hist.jpg`), Kiel, Hamburg's WMS on the SH side |
| `dl/docs/smartmapping.pdf` | GDI-SH-Tag 2023 talk (OGC API plans; no URLs) |
| `probe/alkis/` | ALKIS answers: stored query `ave-by-bbox` (school, 2 × 2 km, border), the VwG WFS (school 25832/25833, 2 × 2 km 25833), **all Norderstedt buildings (49,761) and NutzungFlurstueck (44,800)**, error probes, stored-query descriptions |
| `probe/boundary/land_DE01_25833.xml`, **`schleswig-holstein_simplified25m.geojson`** | the state polygon (full) and the 25 m simplified asset candidate |
| `probe/caps/`, `probe/dla/`, `probe/atom/`, `probe/ckan/`, `probe/geofabrik/`, `probe/osm/` | capabilities, GetFeatureInfo answers, Downloadportal pages/JS/AJAX answers, the DGM1 ATOM feed (37.7 MB), CKAN package lists, Geofabrik state/poly, Overpass answers |
| `tls/bundle.pem` | system roots + the missing `D-TRUST BR CA 1-20-1 2020` intermediate (used by `scripts/c.sh`) |
| `scripts/` | helper scripts (UTM, TIFF tags, XYZ reader, nDSM, tile index summary, GML attribute summary, boundary simplification, border comparison) |

---

## 1. Brandenburg → Schleswig-Holstein at a glance

| freemapper product | Brandenburg (today) | Schleswig-Holstein equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** SH spans 7.87–11.31°E. In 25833 it lies at E 34,833–260,730 (no negative eastings), but with ≈ −4.0° grid convergence and +0.9 m/km scale in Norderstedt (+1.9 m/km on Sylt). WMS and the VwG WFS reproject to 25833. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **DOP20 (TrueDOP, on a bDOM)**. **WMS** `WMS_SH_DOP20col_OpenGBD`, layer `sh_dop20_rgb`, **max 5000 px**, also 25833 (same geometry as the tiles, but brighter tones). Tiles: 1 km RGBI GeoTIFF, **uncompressed, 105 MB each, no HTTP ranges, ≈90 s per tile**. State flown in two halves: 2024 (May/Sept, leaf-on) and 2025 (March–June, **mixed leaf-off/leaf-on**). The school tile is from 17.03.2025 (leaf-off). | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1** from ALS, 1 km tiles. Two copies: **INSPIRE ATOM GeoTIFF** (Float32 LZW, 256 px tiles, NoData −9999, ≈1.5 MB, ranges, ETag) — recommended; or the Downloadportal **XYZ ASCII** (28 MB, no ranges). Vintages per tile 2005–2025 (Hamburg fringe: 2021/2022). No WCS. | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **bDOM 0.2 m** (image-based, Float32 GeoTIFF, uncompressed, 105 MB per tile, no ranges). Vintages 2023 and 2024 only; for the 2025 DOP half the bDOM is **two years older** than the orthophoto. No nDOM, no DOM1, no open LiDAR. | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0 (AAA-suite/XtraServer), GML 3.2 only.** The advertised `WFS_SH_ALKIS_vereinf_OpenGBD` accepts **stored queries only**; the sibling **`WFS_SH_ALKIS_VWG_OpenGBD`** accepts normal `TYPENAMES`/`BBOX` queries for all seven types, with 25833, `COUNT` ≤ 10000, `SORTBY`. LGB spellings (`Gebäude`), `rellage`, detailed `funktion`. An OGC API Features service is announced in the metadata, but its URL could not be resolved. | verified |
| Land use | OGC API `nutzung_flurstueck` | WFS **`ave:NutzungFlurstueck`** (parcel split, like LGB) and `ave:Nutzung`. LGB spellings; `bez` partly set (Parkplatz 14 % of Platz, Begleitfläche 7 % of Straßenverkehr in Norderstedt). School parcel `bez` "Öffentliche Zwecke". | verified |
| Currency | `bb_*_aktualitaet.csv` | per-product **Massendownload GeoJSON** (date per tile), the Downloadportal AJAX `overview.php` (flight and production dates, height system), metadata WMS `WMS_SH_MD_DOP` (GetFeatureInfo: flight date, camera, next planned flight), ETag/Last-Modified on the ATOM GeoTIFFs, `aktualit` per ALKIS feature. The DGM metadata WMS is stale. | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `schleswig-holstein-latest.osm.pbf` (159.3 MB; excludes Hamburg) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **CC BY 4.0**, Quellenvermerk **"©GeoBasis-DE/LVermGeo SH/CC BY 4.0"**, for changed data **"©GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)"** | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **CKAN portal API** | `opendata.schleswig-holstein.de/api/3/action/{package_search,package_show}` | catalogue | fast (0.6 s); packages `digitale-orthophotos-dop20`, `digitales-gelandemodell-1-dgm1`, `bildbasiertes-digitales-oberflachenmodell-bdom`, `alkis-schleswig-holstein-ohne-eigentumerangaben`. Extra `licenseAttributionByText` = `© GeoBasis-DE/LVermGeo SH/CC BY 4.0` on all of them. Valid TLS chain (Let's Encrypt) |
| **Downloadportal** | `https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/` | pages `dl-dop20.html`, `dl-dgm1.html`, `dl-bdom.html`, `dl-alkis.html`; indexes `single.php?file=<X>_Massendownload.geojson&id=4`; files `massen.php?file=…`; AJAX `_ajax/overview.php`, `_ajax/details.php` | Apache + PHP 8.2. **Missing TLS intermediate.** No Content-Length, ETag, Last-Modified or ranges on data. Errors as HTTP 200 HTML |
| **WMS / WFS / ALKIS ATOM** | `https://dienste.gdi-sh.de/<Service>` | `WMS_SH_DOP20col_OpenGBD`, `WMS_SH_DOP20col_hist_OpenGBD`, `WFS_SH_ALKIS_vereinf_OpenGBD`, `WFS_SH_ALKIS_VWG_OpenGBD`, `WMS_SH_ALKIS_OpenGBD`, `WMTS_SH_ALKIS_OpenGBD`, `SH_OpenGBD/feeds/Atom_SH_ALKIS_vereinf_OpenGBD/…` | WMS is MapProxy-like (error wording "image size too large", "unknown layer: …"; inferred); WFS is the AAA-suite (XtraServer; schema location `xtraprodaaa.dpaorinp.de/aaa-suite`). CORS headers present |
| **Metadata WMS, INSPIRE, DGM ATOM** | `https://service.gdi-sh.de/<Service>` | `WMS_SH_MD_DOP`, `WMS_SH_MD_DGM`, `WMS_SH_MD`, `SH_INSPIREDOWNLOAD_*`, **`INSPIRE_Atom/EL/SH_INSPIREDOWNLOAD_AII_EL_DGM1.xml`** and its `INSPIRE_Atom/EL/data/*.tif` | ArcGIS-style metadata WMS (`application/vnd.esri.wms_featureinfo_xml`); static Apache files for the ATOM data with ranges and ETag |
| SH-MIS (metadata) | `sh-mis.gdi-sh.de` | CSW | **broken**: `smartfinder-csw/api … GetRecordById` hung > 120 s and later reset the connection; `csw/api` → 302 → 404. So the registry links (`registry.gdi-de.org/id/de.sh/<uuid>`, which 302 to that CSW) do not resolve |

### 2.1 Tile scheme

- **1 km × 1 km**, EPSG:25832, SW corner at whole km, for DOP20, DGM1 and bDOM. Tile key `32EEENNNN` (e.g. `325655949`);
  the index also carries a 10 km block `kachel10km` / `km=32560_5940`.
- File names (the **year suffix is part of the name and differs per tile**, so names must come from the index):
  - DOP: `dop20rgbi_32_565_5949_1_sh_2025.tif` (317 tiles with suffix `_bw`, remark "BW: Teilbereiche in 60cm Bodenauflösung");
  - DGM1: `dgm1_32_565_5949_1_sh_2022.xyz` (Downloadportal) or `.tif` (ATOM);
  - bDOM: `bdom20nc_32_565_5949_1_sh_2023.tif` (the AJAX metadata calls it `bdom20rgbi_…`, the file is `bdom20nc_…`).
- Grid (from the indexes): DOP/bDOM **17,614 tiles**, SW corners E 425…650, N 5913…6101; DGM1 **18,685 tiles**, E 424…650,
  N 5912…6102. No duplicates.
- **Tiles are whole 1 km squares that intersect SH, not clipped to the state.** Around the school, tiles 563–567 × 5946–5950
  all exist except 566/5946, 566/5947, 567/5946, 567/5947 (entirely in Hamburg). The tile 565/5947 is 52 % Hamburg and is
  delivered complete (DGM1: 1,000,000 valid points; DOP via WMS: imagery to the tile edge). This is the opposite of Hamburg,
  which clips its tiles to the state (and the DOP to districts).

### 2.2 Downloadportal: tile indexes and file URLs

- Index per product (one statewide GeoJSON, EPSG:25832, `MultiPolygon` per tile with float noise like `6003000.000000001862645`):

  | File | Size | Time | Properties |
  |---|---|---|---|
  | `single.php?file=DOP20_SH__Massendownload.geojson&id=4` | 9,888,803 B | 8.4 s | `kachel`, `datum` (flight date), `kachel10km`, `bemerkung`, `link_data` |
  | `single.php?file=DGM1_SH__Massendownload.geojson&id=4` | 8,987,251 B | 8.0 s | `kachel`, `datum`, `link_data` |
  | `single.php?file=bDOM_SH_Massendownload.geojson&id=4` | 8,942,576 B | 7.5 s | `kachel` (**a number here, a string in the others**), `datum`, `link_data` |
  | `single.php?file=ALKIS_SH_Massendownload.geojson&id=4` | 242,982,955 B | 217 s | per **Flur**: `SCHLKRS`, `KREIS`, `SCHLGMD`, `GEMEINDE`, `GEMARKUNG`, `FLUR`, `DATUM` (2026-10-02), `LINK_DATA` → `massen.php?file=<flur>.xml.gz&id=1&live=10_2026` (NAS, monthly) |

  Served as `Content-Type: application/txt`, `Content-Disposition: attachment`, chunked, no ETag/Last-Modified (no change
  detection except by content). `bemerkung` is **double-encoded UTF-8** ("BodenauflÃ¶sung").
- `link_data` examples (the parameters `id`, `live`, `km`, `stack` are all required; dropping `km` fails):
  ```
  https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/massen.php?file=dop20rgbi_32_565_5949_1_sh_2025.tif&id=7&live=2025&km=32560_5940&stack=32565_5949
  https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/massen.php?file=dgm1_32_565_5949_1_sh_2022.xyz&id=2&live=2022&km=32560_5940
  https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/massen.php?file=bdom20nc_32_565_5949_1_sh_2023.tif&id=6&live=2023&km=32560_5940&stack=32565_5949
  ```
- `massen.php` behaviour:
  - `HEAD` → 200 with `Content-Disposition` and `Content-Type: image/tiff`, **no Content-Length** (1.1 s).
  - `Range: bytes=0-15` → **200 with the full body** (104.9 MB for a DOP tile). Ranges are ignored on data (but an error page
    answered a range with 206).
  - Throughput ≈1.2 MB/s (DOP 104,861,201 B in 89.5 s; bDOM in 90.6 s; DGM XYZ 28,000,001 B in 24–28 s). `Accept-Encoding:
    gzip` is ignored.
  - A wrong name or year → **HTTP 200 `text/html`** "Folgender Datensatz: dgm1_32_565_5949_1_sh_2021.xyz konnte nicht
    heruntergeladen werden. Die verwendete Massendownload-Datei ist veraltet. Bitte die aktuelle GeoJSON-Datei herunterladen."
    Check the content type.
- AJAX (undocumented, used by the page; base `…/dladownload/_ajax/`):
  - `overview.php?bbox=565100,5949100,565300,5949300&crs=EPSG:25832&type[]=dgm1` → 200 JSON (0.2 s), a FeatureCollection
    of the tiles in the bbox with all attributes (§7). **At most 200 objects**, else `{"success":false,"reason":"tooManyObjects","max":200}`.
  - `details.php?type=dgm1&id=<ogc_fid>` → the same attributes for one tile.
  - `kachelsuche.php` → 404 (dead).

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DOP/bDOM/DGM1 GeoTIFF: `ProjectedCSType 25832`, GeoAscii "ETRS89 / UTM zone 32N".
  - XYZ: 25832 coordinates (cell centres).
  - WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`; WMS/indexes 25832.
- **Height datum DHHN2016** (normal heights), quasigeoid GCG2016: the DGM1 tile metadata (`overview.php`) says
  `hoehenbezu DHHN2016_NH`, `quasigeoid DE_AdV_GCG2016_QGH`; the DOP metadata `BEZ_SYS_H 7837`. The TIFFs carry no vertical
  key. The DGM **metadata WMS** still says `DE_DHHN92_NH` / `GCG05` with a 2006 date for the same tile (stale; ignore it).
- **Zone 33 is a poor fit.** In 25833 SH lies at E 34,833–260,730 (Sylt is 6.7° west of 15°E). It stays positive, unlike Hesse.

  | Place | Convergence zone 32 / 33 | Scale k zone 32 / 33 |
  |---|---|---|
  | Norderstedt | +0.80° / −4.04° | 0.99965 / 1.00094 |
  | Kiel | +0.92° / −3.95° | 0.99967 / 1.00083 |
  | Sylt (west) | −0.59° / −5.51° | 0.99963 / 1.00188 |
  | Lauenburg (east) | +1.53° / −3.30° | 0.99979 / 1.00051 |

  The grids differ by ≈4.8°. **SH maps must be 25832 projects** (`Region::crs() → Crs::Utm32`, like Hamburg, Saxony-Anhalt
  and Hesse). This also keeps SH–Hamburg cross-border maps in one CRS.
- **Extent** of the state polygon (ALKIS `VerwaltungsEinheit` DE01, `aktualit 2026-11-06Z`, 39,721 vertices, 19 polygons,
  27 interiors):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 425950 – 650150, N 5913461 – 6101206 |
  | EPSG:25833 | E 34833 – 260730, N 5921411 – 6120909 |
  | lon / lat | lon 7.8651 – 11.3132, lat 53.3598 – 55.0562 |

- 25833 is available only through services: DOP WMS (CRS list 3034, 3035, 3044, 3045, 3857, 4326, 4258, 25832, **25833**,
  31467, 31468, 4647; GetMap verified), and the VwG WFS (`OtherCRS` 25833 and 13 more; verified, E N order). The main ALKIS
  WFS (`…_vereinf_OpenGBD`) rejects 25833: 400 "The requested SRS "urn:ogc:def:crs:EPSG::25833" is not supported for
  FeatureType "ave:Flurstueck"". No raster download is offered in 25833.

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- The state is flown **in two halves on a two-year cycle**; the current DOP is a mosaic of both:

  | Flight months (index `datum`) | Tiles | Season |
  |---|---|---|
  | 2024-05 | 7,576 | early leaf-on (mid May) |
  | 2024-09 | 1,293 | leaf-on |
  | 2025-03 | 1,522 | **leaf-off** (includes the Hamburg fringe: Norderstedt 17.03.2025) |
  | 2025-04 | 1,430 | mostly leaf-off |
  | 2025-05 | 2,413 | leaf-on (Kiel 13.05.2025, visually full foliage) |
  | 2025-06 | 3,380 | leaf-on |

- The metadata field `BELAUB` (leaf state) is **0 for both** the March Norderstedt tile and the May Kiel tile, so it says
  nothing. Use the flight month.
- The metadata service gives `BF_PLAN 2027` (next planned flight) for both test tiles.
- Every current tile is a **TrueDOP**: `DOP_BZFL bDOM`. Camera "UltraCamEagle_M3", flight height ≈2490 m, `BOD_PIX 20`.
- **Historic WMS** `https://dienste.gdi-sh.de/WMS_SH_DOP20col_hist_OpenGBD`, layers `SH_DOP20_1` (title SH_DOP20_2023),
  `SH_DOP20_2` (2022), `SH_DOP20_3` (2021). At the school: 2023 = leaf-off (a construction site), 2022 = **empty white**
  (that half was not flown in 2022), 2021 = leaf-on (`dl/wms/cmp_hist.jpg`). Its abstract calls it a "kostenpflichtigen Web
  Map Service", but its Fees are CC BY 4.0 like the others.
- Norderstedt city also publishes its own orthophotos (CKAN `orthofotos-2025-20cm` etc., organisation `norderstedt`,
  dl-de/by-2-0); not examined.

### 4.2 WMS (recommended path)

- `https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD`, WMS 1.3.0, title "WMS SH DOP20".
  - Layer **`sh_dop20_rgb`** ("SH DOP20 RGB", 3-band). No TIME dimension; it shows the current mosaic.
  - **MaxWidth/MaxHeight 5000** (1000 m at 0.2 m). 5001 px → **HTTP 200 `text/xml`** `ServiceExceptionReport` "image size too
    large". freemapper's `WMS_MAX_PX` 4000 fits.
  - Formats: png, gif, jpeg. BoundingBox CRS:84 7.82–11.42 / 53.30–55.15.
  - Fees: "Für die Nutzung der Daten ist die Creative Commons (CC BY 4.0) – Namensnennung 4.0 International anzuwenden. …
    Der Quellenvermerk lautet "© GeoBasis-DE/LVermGeo SH/CC BY 4.0"".
- Example (school, 400 × 300 m at 0.2 m):
  ```
  https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=sh_dop20_rgb&STYLES=&CRS=EPSG:25832&BBOX=565014,5949100,565414,5949400&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 673,046 B, 2.4 s (Content-Length set). 4000² px: 3.77 MB, 10.9 s. 5000² px: 6.09 MB, 22.3 s. The same
  box in **25833** (`CRS=EPSG:25833&BBOX=168908,5960323,169308,5960623`): 200, 676 KB, 0.9 s, visually checked (school and
  sports field centred).
- **Against the tile (native grid, PNG, internal tile 65):** same scene, same cars, geometrically aligned (normalised
  cross-correlation 0.969 at offset 0, ≤ 0.959 at ±1 px), but **different tones**: mean luminance 122 (WMS) vs 101 (tile),
  mean abs RGB difference 22. The WMS is a colour-processed derivative ("abgeleitete Rasterbilder"), not the tile pixels.
- Outside the tile coverage: PNG with `TRANSPARENT=TRUE` is fully transparent, JPEG is white (deep in Hamburg, Eppendorf).
- Unknown layer → 200 `text/xml` "unknown layer: nope".

### 4.3 Tile downloads (Downloadportal)

- One GeoTIFF per 1 km tile via `massen.php` (§2.2). No zip.
- TIFF (school tile, verified):
  - 5000 × 5000, **4 bands RGBI** 8-bit, Photometric RGB, ExtraSamples 0, **Compression 1 (none)**;
  - **256 × 256 internal tiles** (400), each 262,144 B, starting at offset 8;
  - **IFD at the end** (offset 104,857,608 of 104,861,201);
  - ModelPixelScale 0.2, **ModelTiepoint exactly (565000, 5950000)**, `GTRasterType 1` (PixelIsArea), ProjectedCSType 25832;
    no TFW, no NoData tag.
- Because the layout is uncompressed and tiled, a range reader could fetch single 256 px tiles in 260 KB, **but the server
  ignores ranges**. Each tile is a full 105 MB at ≈1.2 MB/s. Statewide that is ≈1.85 TB. **Not recommended**; use the WMS.

---

## 5. DGM1 (terrain model, 1 m)

- From ALS ("3 bis 4 Messpunkte" per m², height ≤ 15 cm on flat ground). The tile vintage varies:

  | Year | Tiles | Year | Tiles |
  |---|---|---|---|
  | 2005 | 3,550 | 2021 | 548 |
  | 2006 | 483 | 2022 | 2,671 |
  | 2007 | 509 | 2023 | 3,501 |
  | 2011–2019 | 30 | 2024 | 3,331 |
  | 2020 | 99 | 2025 | 3,963 |

  Around Hamburg (E 548–590, N 5916–5965): 868 tiles from 2022, 210 from 2021, 135 from 2005, 90 from 2007, 1 from 2020.
  The school tile is ALS **10.03.2022**.

### 5.1 INSPIRE ATOM GeoTIFF (recommended)

- Feed `https://service.gdi-sh.de/INSPIRE_Atom/EL/SH_INSPIREDOWNLOAD_AII_EL_DGM1.xml` (37.7 MB, 40 s, 18,573 entries,
  `updated 2026-08-04`). Each entry has a dataset feed whose `alternate` link is the file:
  ```
  https://service.gdi-sh.de/INSPIRE_Atom/EL/data/dgm1_32_565_5949_1_sh_2022.tif
  ```
  So the URL is `…/INSPIRE_Atom/EL/data/<name from the DGM1 index>.tif`.
- HTTP: `HEAD` → 200 with **Content-Length, ETag** (`"16a695-658d4755641ce"`), **Last-Modified** (Wed, 12 Aug 2026),
  `Accept-Ranges: bytes`; `Range: bytes=0-0` → **206, 1 byte**. Full GET 1,484,437 B in 1.3 s. Valid TLS chain. A wrong year →
  404.
- **The feed XML is stale; the data directory is current.** 3,162 index names are missing from the feed (2,430 of them
  2025 tiles) and the feed lists 3,050 old names instead. All 25 sampled index-only names exist as `.tif`, and 39 of 40 random
  index names exist. The one miss (`dgm1_32_646_6043_1_sh_2005`) has no `.tif` under any year. **Build the URL from the
  Downloadportal index, and fall back to the XYZ on 404.**
- TIFF:
  - 1000 × 1000 **Float32** (SampleFormat 3), **LZW** (Compression 5, Predictor 1), **256 × 256 internal tiles** (16), IFD at
    the start;
  - ModelPixelScale 1, **ModelTiepoint exactly (565000, 5950000)**, `GTRasterType 1` (PixelIsArea), ProjectedCSType 25832,
    **`GDAL_NODATA -9999`**, no vertical key, **no TFW**;
  - the equivalent TFW is `1/0/0/-1/565000.5/5949999.5`, the **same registration as LGB/GeoSN/LVermGeo DGM1**.
  - Values identical to the XYZ (max abs diff 1.8e-6, float rounding). School tile 24.26–34.09 m, centre (E 565214.5 /
    N 5949250.5) **29.60 m**, no NoData. Pillow decodes it.

### 5.2 Downloadportal XYZ (fallback)

- `massen.php?file=dgm1_32_EEE_NNNN_1_sh_<year>.xyz&id=2&live=<year>&km=<10 km block>` → 200 `text/plain;charset=UTF-8`,
  **28,000,001 B** per tile (24–28 s).
- Format: exactly 1,000,000 lines `565000.50 5949999.50 29.85\r\n` (fixed 26 characters + CRLF), cell centres at .50, heights
  with 2 decimals, row-major from the NW corner going east, then south; **one stray `\n` at the end**. No NoData lines in the
  two tiles read (full grid, also on the Hamburg side).

### 5.3 What is not there

- **No WCS** (CKAN search "WCS" finds only the historic Preußische Landesaufnahme). No DGM WMS with heights was looked for.
- The DGM **metadata WMS** `https://service.gdi-sh.de/WMS_SH_MD_DGM` (layers `1` DGM1 Kacheln, `2` Flugstreifen, `3`
  Flugpunkte) returns, for the school tile, `Ersterfassungsdatum 28.01.2006`, `DE_DHHN92_NH`, `GCG05` and a 2007 flight
  strip. That contradicts the 2022 tile; the service is stale.

### 5.4 Border cross-check with Hamburg

Tile 565/5947 (52 % Hamburg): SH delivers all 1,000,000 cells; Hamburg's own DGM1 2022 tile has 624,774 valid cells (the
rest −9999). On the common cells SH − HH: median −0.03 m, mean abs 0.073 m, p1 −0.51, p99 +0.38, extremes ±3.05 m; only 9 %
agree within 1 cm. Both are 2022 ALS, but they are processed independently. A cross-border DTM must pick one source per
cell (e.g. SH inside SH, HH inside HH) to avoid a step.

---

## 6. nDSM sources: bDOM, LiDAR

### 6.1 bDOM 0.2 m (image-based DSM)

- CKAN `bildbasiertes-digitales-oberflachenmodell-bdom`: "Gitterweite von 20 cm … Höhengenauigkeit beträgt ± 0,5 m",
  contains moving objects. Only from the Downloadportal (`id=6`), 1 km tiles, vintages **2023 (8,745 tiles)** and **2024
  (8,869)**.
- Pairing with the DOP: every tile with a 2024 DOP has a 2024 bDOM (same flight); **every tile with a 2025 DOP has a 2023
  bDOM** (the 2025 TrueDOPs were rectified on a bDOM that is not published yet). Norderstedt: DOP 17.03.2025, bDOM
  07.04.2023.
- TIFF (school tile, 104,861,185 B, 90.6 s, no ranges):
  - 5000 × 5000, **Float32**, 1 band, **uncompressed**, **256 px internal tiles**, IFD at the end;
  - ModelPixelScale 0.2, tiepoint exactly (565000, 5950000), PixelIsArea, 25832, **`GDAL_NODATA nan`**;
  - 0 NaN pixels in the school tile; values up to 94.6 m. The grid is 5 × 5 sub-cells per DGM1 cell.
- **bDOM − DGM1** (DGM1 nearest cell):
  - school site bbox (1.13 M px): min −2.14, p10 0.06, **median 0.14**, p90 7.62, max 34.16 m; 25.1 % > 2 m (the site is mostly
    sports field and yard);
  - main building wing (E 565150–565165, N 5949200–5949240): median **7.37 m**, all > 2 m;
  - sports field: median 0.09 m.
  - Clamp < 0 to 0. Expect smoothing at building edges and none of the area under tree crowns (image matching).
- Cost: 105 MB per km² without ranges, ≈90 s per tile, so a 2 × 2 km map needs ≈420 MB and ≈6 min.

### 6.2 LiDAR / nDOM / DOM1

- None open: CKAN searches for Punktwolke, LAZ, DOM (only the bDOM), Laserscanning (only DGM1), Oberflächenmodell (only the
  bDOM). The INSPIRE ATOM has no bDOM/DOM feed (`…_EL_bDOM.xml`, `…_EL_DOM1.xml`, `…/EL/data/bdom20nc_….tif` → 404).

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **Massendownload GeoJSON** (`single.php?file=<P>_Massendownload.geojson&id=4`, 9–10 MB) | DOP20, DGM1, bDOM | `datum` per 1 km tile: DOP = flight date, bDOM = flight date, DGM1 = update date (= acquisition here). Also the file name with its year. No ETag; re-download for changes |
| **`overview.php`** (`…/dladownload/_ajax/overview.php?bbox=…&crs=EPSG:25832&type[]=<dgm1\|dop20\|bdom>`, ≤ 200 tiles, 0.2 s) | DOP20, DGM1, bDOM | DGM1: `a_datum`/`e_datum` 2022-03-10, `hoehenbezu DHHN2016_NH`, `quasigeoid DE_AdV_GCG2016_QGH`, `kaname`, `jahr`. DOP: `a_datum 2025-03-17` (flight), `e_datum 2025-06-03` (production), `bw_bemerk`. bDOM: `a_datum 2023-04-07`, `e_datum 2023-05-26`. Undocumented |
| **Metadata WMS** `https://service.gdi-sh.de/WMS_SH_MD_DOP`, layer `DOP20`, GetFeatureInfo (`text/plain`, `text/html`, `application/geojson`, …; max 4096 px) | DOP20 | 0.1 s. Fields include `KACHEL_N`, `A_DATUM` (flight), `E_DATUM` (production), `HERSTL_D`, `BF_HOEHE`, `KAM_SENS`, `DOP_BZFL bDOM`, `BEZ_SYS_H 7837`, `BELAUB` (always 0), `BF_PLAN 2027`. `text/plain` has a broken umlaut ("Landesamt f�r …") |
| ETag / Last-Modified | DGM1 ATOM GeoTIFFs | HEAD (school tile Last-Modified 2026-08-12); usable as `published` |
| ALKIS `aktualit` | per feature | `YYYY-MM-DDZ`. Norderstedt buildings: 17,224 of 49,761 from 2011 (the migration), the rest spread 2012–2026 (1,940 from 2026); NutzungFlurstueck: all 2025 (35,925) or 2026 (8,875) |
| ALKIS NAS per Flur | – | `DATUM 2026-10-02`, `live=10_2026` (monthly) |

Suggested mapping:
- orthophoto `currency` = index/overview `datum`/`a_datum` per tile under the map (mixed 2024/2025 is possible),
  `published` = `E_DATUM`;
- DGM1 `currency` = index `datum`, `published` = ATOM `ETag`;
- bDOM `currency` = index `datum` (warn when it is older than the DOP).

---

## 8. ALKIS: buildings and actual land use

### 8.1 Services

Both are the AdV "ALKIS vereinfacht" WFS 2.0 on the AAA-suite (XtraServer), namespace
`http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0`, provider "Landesamt für Vermessung und
Geoinformation Schleswig-Holstein (LVermGeo SH)", Fees CC BY 4.0 with "© GeoBasis-DE/LVermGeo SH/CC BY 4.0".

**(a) `https://dienste.gdi-sh.de/WFS_SH_ALKIS_vereinf_OpenGBD`** — the service named in CKAN and the ATOM feed ("WFS SH
ALKIS Vereinfacht").
- Feature types `ave:Flurstueck`, `ave:FlurstueckPunkt`, `ave:GebaeudeBauwerk`, `ave:KatasterBezirk`, `ave:Nutzung`,
  `ave:NutzungFlurstueck`, `ave:VerwaltungsEinheit`; DefaultCRS 25832, **no OtherCRS**.
- **Ad-hoc queries are refused for every type**: `TYPENAMES=ave:GebaeudeBauwerk&BBOX=…` → **400** `ows:ExceptionReport`
  "The requested FeatureType "ave:GebaeudeBauwerk" is not available for the WFS-Query-Operation." (also with `NAMESPACES`,
  and for Flurstueck, Nutzung, NutzungFlurstueck, VerwaltungsEinheit …).
- Only **stored queries** (2,232): `…/alkis-vereinfacht/2.0/ave-by-bbox` (parameters `CRS`, `x1`, `y1`, `x2`, `y2`; returns
  Flurstueck + GebaeudeBauwerk + KatasterBezirk + Nutzung + VerwaltungsEinheit, **not** NutzungFlurstueck),
  `GetGebByGemeinde` (`gemeindeschluessel`), `GetNutzungFlurstueckFromGem<AGS>` (one per municipality, 1,108),
  `GetNutzungFromGem<AGS>`, `GetNutzungFlstByFlst`, `GetVerwaltungsEinheitART` (`art`), `GetFlstBy…`, `GetFeatureById`.
- `ave-by-bbox` at the school (400 × 300 m): 200, 191 features, **2,768,169 B**, 3.5 s, of which **2.47 MB are the five
  VerwaltungsEinheit polygons** (Gemeinde Norderstedt, Kreis Segeberg 21,771 vertices, Land and "Regierungsbezirk" 39,721
  vertices each). Every bbox query carries that overhead. 2 × 2 km: 11,834 features, 16.3 MB, 17.3 s.
  - Paging works (`COUNT`/`STARTINDEX`, `numberMatched` on every page); order is by type, then stable; `SORTBY` is accepted.
  - 25833 → 400 (see §3).
  - **`RESULTTYPE=hits` is ignored on stored queries**: `GetNutzungFlurstueckFromGem01060063&RESULTTYPE=hits` returned all
    44,800 features (49.0 MB, 64 s); `GetGebByGemeinde&gemeindeschluessel=01060063&RESULTTYPE=hits` returned 49,761 buildings
    (47.6 MB, 49 s).

**(b) `https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD`** (recommended) — titled "WFS_SH_ALKIS_VwG_OpenGBD", abstract
"…GeoObjekten zum Themenbereich Verwaltunggrenzen", but it serves **all seven types with normal queries**.
- CRS: Default 25832; Other **25833**, 3034, 3035, 3044, 3045, 31463, 31464, 31467, 31468, 31469, 3857, 4258, 4326, 4647,
  900913.
- Output GML 3.1/3.2 only. `OUTPUTFORMAT=application/json` → **400** XML. No gzip (`Vary: Accept-Encoding` but no encoding).
- **`CountDefault 10000`**; `COUNT=20000` → 400 "Parameter 10000 exceeds configurated limit". `ImplementsResultPaging FALSE`,
  but `SORTBY=oid&COUNT=3&STARTINDEX=0/3` gave disjoint consecutive oids. `ImplementsSorting TRUE`.
- `RESULTTYPE=hits` works: 2 × 2 km buildings `numberMatched="6687"` in 0.15 s. Normal pages also carry `numberMatched`.
- Errors: **HTTP 400** + `ows:ExceptionReport` (unlike Saxony-Anhalt's 200).
- Speed: school bbox buildings 25832 0.7 s, 92 KB, 84 features; **25833 cold 16.8 s** (96 features); NutzungFlurstueck 25833
  0.9 s, 91 features. 2 × 2 km in 25833: buildings 6,882 / 7.2 MB / 10.6 s; NutzungFlurstueck 4,583 / 5.5 MB / 6.3 s.
- Geometry: `gml:MultiSurface` → `Polygon` → `exterior`/`interior` `posList`, `srsDimension="2"`, E N order; native 3
  decimals, reprojected 9 decimals. Default namespace in the response (no `ave:` prefix on feature elements), like Saxony-Anhalt.
- 25833 accuracy: over 1,108 vertices of 79 common buildings the server's 25833 equals my conversion of its 25832 output
  within **0.74 mm** (constant offset).
- **Risk:** this endpoint is evidently meant for administrative boundaries; its full content may be unintended and could be
  restricted later. Keep `ave-by-bbox` on (a) as a fallback.

Working example (school, 25833, the shape of freemapper's `alkis_wfs` URL):
```
https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=168908,5960323,169308,5960623,urn:ogc:def:crs:EPSG::25833&SORTBY=oid&COUNT=10000&STARTINDEX=0
```

**(c) OGC API Features** — CKAN lists "Daten des Liegenschaftskatasters - ALKIS Vereinfacht (OGC API-Features Webservice)"
(GeoJSON/GML; collections Flurstücke, Flurstückspunkt, Gebäude, Nutzungen, Nutzung-Flurstück, Verwaltungseinheiten,
Katasterbezirke), but its only URL is `https://registry.gdi-de.org/id/de.sh/faead27a-3a25-4a00-81d7-3dc302dc3a77`, which
302s to the broken SH-MIS CSW. The GDI-DE catalogue reset the connection. Nine guessed paths on `dienste.gdi-sh.de` were 404;
`ogcapi.`/`api.`/`features.gdi-sh.de` do not resolve. **Not found.**

Other: INSPIRE BU/ELU WFS (`https://service.gdi-sh.de/SH_INSPIREDOWNLOAD_AIII_BU`, `…_AIII_ELU`), the WMS/WMTS ALKIS and the
NAS download per Flur were not examined.

### 8.2 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid`, `aktualit`, `gebnutzbez`, `funktion`, `gfkzshh`?, `rellage`?, `name`?, `anzahlgs`?, `gmdschl`,
  `lagebeztxt`?, geometry `geometrie`. **LGB names and LGB spellings.**
- `NutzungFlurstueck`: `oid` (34 characters: Nutzung oid + Flurstück oid, e.g. `DESHPDHK0001032CDESHPDHK0000Zt81TF`),
  `aktualit`, `nutzart`, `bez`?, `name`?, `flstkennz`.
- `Nutzung`: `oid` (18 characters, `…TN`), `aktualit`, `nutzart`, `bez`?, `name`?.
- oids start with **`DESH`** (`DESHPDHK…BL` buildings, `…TN` land use), so `Region::of_object_id` gets a `"DESH"` arm.
  (VerwaltungsEinheit oids are `DE01…` and are not region-tagged by that rule.)

### 8.3 Sample features at the Coppernicus-Gymnasium (geometry elided)

```json
{"oid":"DESHPDHK00019nnABL","aktualit":"2025-02-21Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule",
 "gfkzshh":"31001_3021","name":"Coppernicus - Gymnasium","anzahlgs":"1","gmdschl":"01060063","lagebeztxt":"Coppernicusstraße 1"}
{"oid":"DESHPDHK00019nnBBL","aktualit":"2016-11-25Z","gebnutzbez":"Gebäude","funktion":"Sport-, Turnhalle","gfkzshh":"31001_3211","anzahlgs":"1","gmdschl":"01060063"}
{"oid":"DESHPDHK0008WOHpBL","aktualit":"2024-01-19Z","gebnutzbez":"Bauteil","funktion":"Tiefgarage","rellage":"Unter der Erdoberfläche","gmdschl":"01060063"}
{"oid":"DESHPDHK00019no2BL","aktualit":"2024-01-19Z","gebnutzbez":"Gebäude","funktion":"Parkdeck","gfkzshh":"31001_2462","rellage":"Aufgeständert","gmdschl":"01060063"}
{"oid":"DESHPDHK00019tjYTN","aktualit":"2025-10-27Z","nutzart":"Fläche besonderer funktionaler Prägung","bez":"Öffentliche Zwecke"}
{"oid":"DESHPDHK00019tDFTN","aktualit":"2025-10-27Z","nutzart":"Weg","bez":"Rad- und Fußweg"}
{"oid":"DESHPDHK00019tulTN","aktualit":"2025-10-27Z","nutzart":"Straßenverkehr","name":"Coppernicusstraße"}
{"oid":"DESHPDHK00019tHMTN","aktualit":"2025-10-27Z","nutzart":"Platz"}
```

The school parcel's `bez` is **"Öffentliche Zwecke"** (as in Saxony-Anhalt), not Brandenburg's "Bildung und Wissenschaft"
(which occurs only 4 times in Norderstedt).

### 8.4 Observed values (all of Norderstedt: 49,761 buildings, 44,800 NutzungFlurstueck; plus the 2 × 2 km area)

- **`gebnutzbez`** (LGB spellings): `Gebäude` 35,747, `Sonstiges Bauwerk oder sonstige Einrichtung` 12,058 (Überdachung,
  Carport …), `Bauteil` 1,864, `Vorratsbehälter, Speicherbauwerk` 88, `Turm` 3, `Bauwerk oder Anlage für Industrie und
  Gewerbe` 1. freemapper's kind rule (`Gebäude`) works unchanged.
- `rellage`: `Unter der Erdoberfläche` 86 (Tiefgarage, Keller), `Aufgeständert` 8.
- `funktion` (142 values, detailed AdV names): Wohnhaus 18,607, Überdachung 8,605, Schuppen 5,805, Nach Quellenlage nicht zu
  spezifizieren 4,408, Garage 4,071, Carport 3,419, Sonstiges 939, Stall 520, … **Allgemein bildende Schule 63**, Sport-,
  Turnhalle 20, Gebäude zum U-Bahnhof (Garstedt). No `;`-joined values.
- `name` on 27 buildings (schools, Amtsgericht, Finanzamt …).
- **`nutzart`** (LGB spellings, 22 values): Wohnbaufläche 23,791, Straßenverkehr 5,791, Landwirtschaft 2,879, Fläche
  besonderer funktionaler Prägung 1,852, Weg 1,635, Industrie- und Gewerbefläche 1,597, Fließgewässer 1,547, Sport-, Freizeit-
  und Erholungsfläche 1,422, Wald 1,385, Fläche gemischter Nutzung 836, Platz 507, Gehölz 414, Bahnverkehr 319,
  Unland/Vegetationslose Fläche 264, Stehendes Gewässer 245, Moor 205, Tagebau, Grube, Steinbruch 61, Heide 17, Friedhof 12,
  Flugverkehr 10, Sumpf 8, Halde 3.
- **`bez` per `nutzart`** (the values freemapper's rules key on):

  | nutzart | bez |
  |---|---|
  | Straßenverkehr | none 5,360; **Begleitfläche Straßenverkehr 418**; Gebäude- und Freifläche zu Verkehrsanlagen, Straße 13 |
  | Platz | none 402; **Parkplatz 72**; Raststätte, Autohof 17; Rastplatz 16 (no Fußgängerzone/Marktplatz seen) |
  | Weg | Rad- und Fußweg 1,019; none 613; Fahrweg 3 |
  | Fläche besonderer funktionaler Prägung | none 1,608; Öffentliche Zwecke 213; Soziales 12; Historische Anlage 5; Bildung und Wissenschaft 4; … |
  | Wohnbaufläche | never |

  Others: Grünland 1,896, Grünanlage 1,201, Industrie und Gewerbe 1,039, Ackerland 873, Graben 815, Bach 732, Laubholz 607,
  Laub- und Nadelholz 458, Nadelholz 320, Sportanlage 124, Spielplatz, Bolzplatz 6 …
- `name` is set on many Straßenverkehr/Fließgewässer objects (street and stream names).

### 8.5 What is missing or different

- `bez` is only partly filled on Straßenverkehr and Platz, so `VERGE` and `PAVED_SQUARES` fire on a minority of objects (better
  than Saxony/Saxony-Anhalt, worse than Brandenburg/Hamburg).
- No GeoJSON, no gzip; the main WFS needs stored queries; the full-featured endpoint is the "VwG" one.
- **ALKIS stops at the state border** (verified from both sides): in the border bbox E 565000–566000 / N 5946500–5948000, the
  SH WFS returned 1,419 buildings whose east-most vertex is at E 565,535; Hamburg's OGC API returned 444 buildings (gmdschl
  02000000) starting at E 565,443. No feature from the other state.

---

## 9. Licence and attribution

- Licence: **Creative Commons Namensnennung 4.0 International (CC BY 4.0)** on every product used (CKAN `license_id`
  `http://dcat-ap.de/def/licenses/cc-by/4.0`; WMS/WFS Fees; Downloadportal terms).
- Downloadportal terms `https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/lizenz.html` ("Stand 01. August
  2022"), verbatim:
  > (3) Bei der Nutzung ist sicherzustellen, dass folgende Angaben als Quellenvermerk enthalten sind: 1. Bezeichnung des
  > Bereitstellers nach dessen Maßgabe, 2. der Vermerk „Creativ Commons – Namensnennung 4.0 International" oder „CC BY 4.0"mit
  > Verweis auf den Lizenztext unter http://creativecommons.org/licenses/by/4.0 sowie 3. einen Verweis auf den Datensatz
  > (URI). Dies gilt nur soweit die datenhaltende Stelle die Angaben 1. bis 3. zum Quellenvermerk bereitstellt.
  > (4) Veränderungen, Bearbeitungen, neue Gestaltungen oder sonstige Abwandlungen sind im Quellenvermerk mit Hinweis zu
  > versehen, dass die Daten geändert wurden.
  > … einen deutlich sichtbaren Quellenvermerk anzubringen, der wie folgt auszugestalten ist:
  > **©GeoBasis-DE/LVermGeo SH/CC BY 4.0**
  > **©GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)**
- Services and CKAN write it with a space: "© GeoBasis-DE/LVermGeo SH/CC BY 4.0".
- Inconsistencies: the ALKIS ATOM feed's `<rights>` (updated 2023) still says dl-de/by-2-0 "© GeoBasis-DE/LVermGeo SH" with
  a typo'd URL (`govdatn.de`); the historic DOP WMS abstract says "kostenpflichtig"; the LVermGeo "Dienste" web page still
  says WMS use needs a licence agreement. The CKAN records, service Fees and the portal terms all say CC BY 4.0.
- **Recommended freemapper line:** `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)`, with the licence link
  `https://creativecommons.org/licenses/by/4.0/` in the project notes. Optionally add the dataset URIs (e.g. the CKAN package
  pages); § 3 Nr. 3 only applies where the provider supplies them.

---

## 10. State boundary (for `assets/regions/schleswig-holstein.geojson`)

- **Polygon source (recommended):** the VwG WFS, `ave:VerwaltungsEinheit` filtered on `art = Bundesland`, directly in 25833:
  ```
  https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&FILTER=<fes:Filter xmlns:fes="http://www.opengis.net/fes/2.0" xmlns:ave="http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0"><fes:PropertyIsEqualTo><fes:ValueReference>ave:art</fes:ValueReference><fes:Literal>Bundesland</fes:Literal></fes:PropertyIsEqualTo></fes:Filter>
  ```
  (URL-encode the FILTER) → 200, 1 feature, 1,350,155 B, 2.7 s. oid **`DE01`**, `name Schleswig-Holstein`, `aktualit
  2026-11-06Z`. Alternative on the main WFS: stored query `GetVerwaltungsEinheitART&art=Bundesland` (25832 only). The
  "Regierungsbezirk" `DE010` (name "   ") carries the same geometry.
- Geometry: MultiSurface, 19 polygons, 39,721 vertices, 27 interiors (25 of them zero-area slivers). The main polygon (≈15,890
  km² in 25833) includes Sylt, Fehmarn and Nordstrand; separate polygons are Föhr 83.2 km², Pellworm 36.3, Amrum 20.5, the
  Halligen, Helgoland (4.24 km², 54.184°N 7.896°E) and:
  - a **5,616 m² exclave at 53.714°N 10.116°E** inside Hamburg (= the hole in Hamburg's polygon, same area);
  - one real hole of **8,131 m²** (= Hamburg's exclave near Volksdorf, same area).
- **25 m Douglas–Peucker simplification:** `probe/boundary/schleswig-holstein_simplified25m.geojson`. FeatureCollection,
  `crs` `urn:ogc:def:crs:EPSG::25833`, properties `name: Schleswig-Holstein`, `source: LVermGeo SH WFS_SH_ALKIS_VWG_OpenGBD
  (VerwaltungsEinheit DE01), © GeoBasis-DE/LVermGeo SH/CC BY 4.0`, `retrieved: 2026-11-08`, `simplified_m: 25`; MultiPolygon
  with integer coordinates.
  - **3,559 vertices, 58.3 KB** (Saxony-Anhalt asset 74.8 KB). 17 polygons kept (main 2,947 vertices + the 8,131 m² hole);
    dropped: the 25 degenerate holes, a 98 m² islet near Helgoland (54.182°N 7.914°E) and a 742 m² 144-vertex sliver near Kiel
    (54.391°N 10.194°E).
  - Extent E 34,838–260,730, N 5,921,411–6,120,909: **positive eastings**.
- **Shared border with `assets/regions/hamburg.geojson`** (both 25 m simplified from ALKIS):
  - shared length ≈131 km; distance between the two simplified lines: p50 0.0 m, p90 0.0 m, **p99 2.7 m**, max 95 m (the max is
    where the lines approach without being shared, e.g. across the Elbe);
  - by 5 m grid sampling within 125 m of the Hamburg line: **overlap ≈2,350 m²**, gap ≈20,100 m² (cells outside both within
    30 m of both lines), i.e. well under a metre of average width;
  - the SH exclave/Hamburg hole and the Hamburg exclave/SH hole match.
  - So `regions_for` will report both regions only for bboxes that really straddle; the usual margin suffices.
- Neighbours: Hamburg (a freemapper region), Lower Saxony (across the Elbe), Mecklenburg-Western Pomerania, Denmark.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf` → **307** →
  `schleswig-holstein-261106.osm.pbf` → 200, **159,295,422 B**, `Last-Modified: Sat, 07 Nov 2026 02:16:58 GMT`,
  `ETag "97ea7be-65fa652cc2a9f"`, `Accept-Ranges: bytes`. The `Location` header is an absolute URL.
- State: `https://download.geofabrik.de/europe/germany/schleswig-holstein-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4956
  ```
- Poly: `https://download.geofabrik.de/europe/germany/schleswig-holstein.poly`, one ring, 110 points, lon 7.465–11.799,
  lat 53.357–55.146. It includes Helgoland and **excludes Hamburg** (Eppendorf and the Rathaus test outside; the Coppernicus
  school inside). A cross-border SH–HH map needs both extracts.
- Overpass answered 200 with `freemapper-research/0.1 (github.com/HenryJobst/freemapper)` and `Accept: application/json`
  (31–41 s per query).

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `geodaten.schleswig-holstein.de` (Downloadportal) | **Missing TLS intermediate**: the server sends only the leaf (issuer `D-TRUST BR CA 1-20-1 2020`; AIA `http://www.d-trust.net/cgi-bin/D-TRUST_BR_CA_1-20-1_2020.crt`, which 302s to `www1.d-trust.net`). `openssl s_client` → "Verify return code: 21 (unable to verify the first certificate)"; Python/OpenSSL → `CERTIFICATE_VERIFY_FAILED`; curl works only with a bundle that contains the intermediate. rustls does no AIA fetching, so freemapper's ureq will fail (inferred; the root `D-TRUST BR Root CA 1 2020` is in webpki-roots 1.0.11, the intermediate is not). **Range ignored**, no Content-Length/ETag/Last-Modified, ≈1.2 MB/s, no gzip, errors as **200 text/html**, PHP AJAX limit 200 objects, index `bemerkung` double-encoded |
| `dienste.gdi-sh.de` WMS DOP | Max **5000** px; oversize → **200 `text/xml`** "image size too large"; unknown layer → 200 XML; outside coverage transparent PNG / white JPEG; Content-Length present; CORS |
| `dienste.gdi-sh.de` ALKIS WFS `…_vereinf_OpenGBD` | ad-hoc queries refused (**400**); stored queries only; `ave-by-bbox` adds ≈2.5 MB of admin polygons to every answer; no 25833; **`RESULTTYPE=hits` ignored on stored queries (full download)** |
| `dienste.gdi-sh.de` ALKIS WFS `…_VWG_OpenGBD` | full ad-hoc queries, 25833, `COUNT` ≤ 10000 (more → 400), SORTBY paging, hits OK; errors 400 + `ows:ExceptionReport`; GML only; no gzip; cold 25833 request 16.8 s, then 1–11 s |
| `service.gdi-sh.de` ATOM DGM1 | static files: HEAD, Content-Length, ETag, Last-Modified, ranges (206); the 37.7 MB feed XML is stale (use the Downloadportal index for names); missing file → 404 HTML |
| `service.gdi-sh.de` metadata WMS | fast (0.1 s); DOP fields current; **DGM fields stale** (2006/DHHN92); `text/plain` carries U+FFFD instead of umlauts |
| `sh-mis.gdi-sh.de` CSW | hangs or 404; registry links do not resolve |
| `opendata.schleswig-holstein.de` CKAN | fast, valid TLS; 578 LVermGeo packages (most of them TK25 sheets) |

- No authentication, no rate-limit or `Retry-After` headers, no throttling in roughly 200 requests. No fair-use rules were
  found on the pages read (not exhaustive).
- Total transfer for this research ≈ **700 MB** (two 105 MB tiles, the 243 MB ALKIS index, ≈100 MB of Norderstedt ALKIS,
  four 28 MB XYZ downloads incl. repeats, the 38 MB ATOM feed, WMS probes); disk under `sh/` ≈475 MB.

---

## 13. What makes the Schleswig-Holstein adapter differ (risks first)

1. **TLS on `geodaten.schleswig-holstein.de`.** Every Downloadportal URL (indexes, DOP/bDOM tiles, XYZ, AJAX) fails
   certificate verification in clients without AIA fetching, which includes rustls. Options: bundle the `D-TRUST BR CA 1-20-1
   2020` intermediate (valid to 2035-02-11) as an extra root for that host, or avoid the host. The DGM1 (ATOM), the WMS and
   the WFS are on hosts with correct chains. **Decision needed** for the bDOM, which exists only on this host.
2. **ALKIS needs the right endpoint.** The advertised WFS refuses normal queries. The VwG WFS behaves like GeoSN/Hesse
   (`alkis_wfs` path: `TYPENAMES`, `SRSNAME`/`BBOX` 25832 or 25833, `SORTBY=oid`, `COUNT=10000`), but it is labelled as an
   administrative-boundary service, so it may change. The OGC API is announced but unreachable. Fallback: `ave-by-bbox` stored
   query (mixed types, +2.5 MB per call, 25832 only, no NutzungFlurstueck; use `Nutzung` there).
3. **Raster downloads are heavy and range-less.** DOP and bDOM tiles are 105 MB uncompressed each at ≈1.2 MB/s (≈90 s), with
   no ranges, ETag or Content-Length. Use the **WMS** for the orthophoto. For the nDSM there is no alternative to the full bDOM
   tiles (≈6 min for a 2 × 2 km map).
4. **The orthophoto is a seasonal and yearly patchwork**: 2024 (May/Sept, leaf-on) and 2025 (March–June; March/April
   leaf-off, May/June leaf-on), and the WMS has no TIME. Leaf-off is luck of location (the Hamburg fringe is March 2025). The
   WMS tones differ from the tiles (brighter); geometry matches.
5. **The bDOM lags the DOP** for the 2025 half (bDOM 2023 vs DOP 2025): new buildings mismatch. The 2024 half is consistent.
6. **DGM1 has two copies with different metadata quality.** The ATOM GeoTIFF is the good one (1.5 MB, ranges, ETag), but its
   feed is stale, so names (with a per-tile year) must come from the Downloadportal index (on the TLS-broken host) or from a
   cached copy of it; the XYZ (28 MB ASCII) is the fallback. Vintages range from 2005 to 2025; the DGM metadata WMS is wrong.
7. **Tiles are not clipped to the state.** SH tiles run to the km edge into Hamburg; Hamburg clips its own. For cross-border
   maps, the SH WMS covers the Hamburg fringe; the two DGM1s differ by a few cm (mean 7 cm, up to ±3 m), so pick one per cell.
   ALKIS is clean on both sides.
8. **CRS 25832** for all products; zone 33 means ≈4° to 5.5° rotation and up to +1.9 m/km scale. Make SH a `Crs::Utm32`
   region (like Hamburg).
9. **ALKIS content:** LGB spellings everywhere (no normalisation, unlike Hamburg); `rellage` and `Bauteil` present; detailed
   `funktion`; `bez` only partly set on Straßenverkehr/Platz; the school parcel is "Öffentliche Zwecke". GML only, no gzip.
10. Unchanged from the other states: no login, DHHN2016, DGM1 pixel registration (PixelIsArea at the NW corner, centres at
    +0.5), Float32 with NoData −9999, 1 km tiles, Geofabrik layout. New: CC BY 4.0 instead of dl-de/by-2-0, and the oid
    prefix `DESH`.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Schleswig-Holstein | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `WmsService{url: "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD", layer: "sh_dop20_rgb", crs: Utm32, extra: ""}`, native 0.2 m grid, pieces ≤ 4000 px (server max 5000) | Currency: `datum` per tile from `DOP20_SH__Massendownload.geojson` or `WMS_SH_MD_DOP` GetFeatureInfo (`A_DATUM`, `E_DATUM`); report the season. Tile download (105 MB, no ranges) only as an exact-pixel fallback. |
| **DTM** (hillshade, slope, contours) | **DGM1 GeoTIFF** `https://service.gdi-sh.de/INSPIRE_Atom/EL/data/<name>.tif`, `<name>` from the DGM1 index (`dgm1_32_EEE_NNNN_1_sh_<year>`) | Float32 LZW, NoData −9999, no TFW (derive the grid from the tile key or the tags), ETag as `published`. On 404 fall back to the XYZ via `massen.php`. Ship or cache the index (9 MB) to avoid the TLS-broken host on every run. |
| **nDSM template** | **bDOM 0.2 m − DGM1**, bDOM tile from `massen.php?file=bdom20nc_…&id=6&…` (full 105 MB per tile), clamp < 0 | Needs the TLS fix. Warn when `bDOM datum` < `DOP datum` (all 2025-DOP tiles). Optional to skip in SH if download time matters. |
| **Buildings** | **WFS** `https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD`, `ave:GebaeudeBauwerk`, GML 3.2, `SRSNAME`/`BBOX` in the project CRS (25832), `SORTBY=oid&COUNT=10000&STARTINDEX=…` | Reuse the GeoSN/Hesse GML reader (default namespace). LGB spellings; underground via `rellage`. Fallback: stored query `ave-by-bbox` on `WFS_SH_ALKIS_vereinf_OpenGBD`. |
| **Land use** (out-of-bounds, paved) | Same WFS, **`ave:NutzungFlurstueck`** (parcel split, like LGB) | `bez` partial on Straßenverkehr/Platz; school parcel "Öffentliche Zwecke". |
| **Region boundary** | VwG WFS `VerwaltungsEinheit` `art=Bundesland` (DE01) in 25833, simplified 25 m | Ready in `probe/boundary/schleswig-holstein_simplified25m.geojson` (3,559 vertices, 17 polygons, 1 hole); matches the Hamburg asset to p99 2.7 m. Region CRS Utm32; oid prefix `DESH`. |
| **OSM** | Geofabrik `europe/germany/schleswig-holstein-latest.osm.pbf` + `schleswig-holstein-updates/state.txt` | 159 MB; 307 to `schleswig-holstein-YYMMDD.osm.pbf`. Excludes Hamburg. |
| **Attribution** | `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)` + licence link + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- The OGC API Features endpoint for ALKIS vereinfacht (announced in CKAN; the registry/CSW link is broken; not found by
  guessing).
- Whether the VwG WFS's full ALKIS content is intended and will stay; its behaviour above 10,000 matches per bbox (only
  ≤ 6,882 tested) and the stability of `STARTINDEX` ordering across data updates.
- That rustls actually fails on `geodaten.schleswig-holstein.de` (inferred from the missing intermediate and from OpenSSL and
  Python failing; no Rust client was run).
- The WMS's resampling in 25833 and its JPEG quality; why its tones differ from the tiles (colour processing assumed); whether
  the WMS uses the `_bw` (partly 60 cm) tiles.
- The 2024 DOP tile layout (only a 2025 tile was opened), the `_bw` tiles, and what "nc" in `bdom20nc` means.
- The meaning of the DOP metadata `BELAUB` (0 for both leaf-off and leaf-on tiles).
- Whether the bDOM has NaN gaps elsewhere (water, shadows); only the school tile was read (0 NaN).
- How many DGM1 index tiles lack an ATOM GeoTIFF (1 of 40 random samples missing); whether the ATOM data directory is updated
  at the same time as the Downloadportal.
- `bez` coverage outside Norderstedt (Kiel, Lübeck, rural areas); INSPIRE BU/ELU WFS, ALKIS NAS per Flur, WMTS, LoD2 and
  Hausumringe were not examined.
- The ALKIS of Kiel or Lübeck (only DOP currency and the WMS were checked in Kiel).
- Rate limits or fair-use rules (none seen, none documented on the pages read).
