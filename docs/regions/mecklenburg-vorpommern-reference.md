# Mecklenburg-Vorpommern (LAiV M-V) data sources for freemapper — live-verified reference

Verified live on 2026-11-08, 02:00–03:00 UTC (03:00–04:00 Europe/Berlin, Sunday night). Provider: **Landesamt für innere
Verwaltung Mecklenburg-Vorpommern (LAiV M-V), Amt für Geoinformation, Vermessungs- und Katasterwesen (AfGVK)**, Schwerin.
Open data ("OpenData-Geobasisdaten") under **CC BY 4.0** (portal terms "Stand 09. Juni 2024"). The data sit on four hosts:

- the services host **`www.geodaten-mv.de/dienste/<name>`**: MapServer WMS and **WCS** (DOP, DGM, DOM, nDOM), the
  AAA-suite (XtraServer) ALKIS WFS, and ATOM feeds whose files come from `…/dienste/<product>_download?…` (2 km tiles);
- **`www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1`**: an **OGC API Features** for ALKIS vereinfacht with the **same
  collection names, parameters and GeoJSON as Brandenburg's** `ogc-api.geobasis-bb.de/alkis-vereinfacht/v1`;
- the **Downloadportal** `laiv.geodaten-mv.de/afgvk/` (GAIA-light `dladownload`, the same software as SH and TH) with an
  undocumented AJAX tile index `…/_apps/dladownload/_ajax/overview.php` that gives the **date of every 2 km tile** per product;
- `www.laiv-mv.de` (product pages, yearly currency overviews as PDF/shapefile zip, licence PDF). The catalogue is
  GeoMIS.MV, with its CSW at `www.geodaten-mv.de/soapServices/api`.

No login, registration, token, form or CAPTCHA is needed for anything below. Only ALS point clouds, DOP10 and coloured bDOM10
are paid orders (€); freemapper does not need them. **TLS chains are complete on every host** (Telekom Security; all roots are
in webpki-roots 1.0.11, §12).

Test locations:

| School / place | OSM | Centre EPSG:25833 | Centre EPSG:25832 | lat / lon | Footprint bbox 25833 | 1 km tile | MV 2 km tile |
|---|---|---|---|---|---|---|---|
| **Innerstädtisches Gymnasium (ISG)**, Goetheplatz 5, Rostock (primary) | area way **402355834**; building relation **2203698** (`building=school`, multipolygon with 17 `building:part`s, track, 3 pitches) | E 312157 / N 5996317 | E 704637 / N 5997030 | 54.08073 / 12.12840 | E 312068–312250, N 5996240–5996397 | `33 312 5996` (fully inside); in 25832 it straddles `32 704 5996` / `32 704 5997` | `33_312_5996` (E 312000–314000, N 5996000–5998000) |
| Border probe: Brüssow (Brandenburg, Uckermark) / Vorpommern-Greifswald | – | box E 436007–438007, N 5917677–5919677 (30 % MV, 70 % BB); border point E 437007 / N 5918677 | E 835686 / N 5930156 | 53.41339 / 14.05226 | | | `33_436_5918` |
| ALKIS value samples only: Rostock 2 × 2 km around the school (E 311157–313157, N 5995317–5997317, incl. the old town), Schwerin centre 1.5 × 1.5 km (E 262202–263702, N 5947448–5948948), Rostocker Heide forest 2 × 2 km (E 318343–320343, N 6008874–6010874) | | | | | | | |
| DGM water probes: Warnemünde coast (E 308000–310000, N 6007000–6011000), Schweriner See (E 266000–268000, N 5950000–5952000), Müritz (E 346000–348000, N 5920000–5922000) | | | | | | | |

(Coordinates come from my own Krüger-series UTM code and agree with pyproj to 0.3 m. The ALKIS school building
`DEMVAL03000bhHpnBL` came back at E 312084–312205 / N 5996305–5996396 in 25833, inside the OSM area. The OSM school
building is mapped only as relation 2203698, not as a way; an Overpass `way[building]` query near the school finds nothing.)

Samples (all under `mv/`; treat them as untrusted data):

| Path | Content |
|---|---|
| **`mecklenburg-vorpommern.geojson`** (= `probe/boundary/mecklenburg-vorpommern_simplified25m.geojson`) | **the asset candidate** for `assets/regions/mecklenburg-vorpommern.geojson` (§10) |
| `dl/dgm1/dgm1_33_312_5996_2_gtiff.tif`, `dl/dom1/dom1_33_312_5996_2_gtiff.tif`, `dl/ndom/ndom_33_312_5996_2_gtiff.tif` | the school's 2 km DGM1, DOM1 and nDOM tiles from the ATOM download endpoints |
| `dl/dop/dop20rgb_33_312_5996_2_mv.tif` | the school's 2 km DOP20 RGB tile (**256-colour palette**, 141.6 MB) |
| `dl/rgbihead/rgbi_head.bin` | the first 1.06 MB of the RGBI tile (header + first 512 px internal tile; the full file is 419 MB) |
| `dl/wcs/` | WCS answers: DGM/DOM/nDOM 2 km at the school (plain, LZW, Deflate), DGM school bbox, edge probe, DOP20 school (TIFF, JPEG, JPEG-in-TIFF) |
| `dl/wms/` | DOP WMS GetMaps: school in 25833 and 25832, 4000/4192/4193 px probes, spring layer, sea probe |
| `dl/preview/` | small previews: school, watermark crops, border overlay, palette vs RGBI |
| `dl/akt/` | yearly currency shapefile zips from laiv-mv.de (DOP 2024/2025, DGM 2024/2025, DOM 2024, nDOM 2024, ALS 2025) |
| `dl/docs/vgb_agnb.pdf` | the AGNB terms (paid data) incl. the "© GeoBasis-DE/MV <Jahr…>" rule |
| `probe/alkis/` | WFS answers: school, 2 × 2 km Rostock, Schwerin centre, Rostocker Heide, paging/limit/error probes |
| `probe/ogcapi/` | OGC API answers: landing, collections, OpenAPI, school, 2 × 2 km, paging, errors |
| `probe/border/` | border probe: DGM/nDOM WCS, DOP WMS, ALKIS buildings/land use, DOP WCS edge |
| `probe/coast/` | DGM WCS at Warnemünde, Schweriner See, Müritz |
| `probe/dla/` | Downloadportal JS, `overview.php` answers, **statewide per-tile indexes** `dgm1_index.json`, `dom_index.json`, `ndom_index.json`, `dop20_index.json` (from 144 calls in `state/`) |
| `probe/atom/` | ATOM dataset feeds (DGM1 15.2 MB, DOM1 12.7 MB, DOP20 RGB 5.0 MB, RGBI 2.5 MB, nDOM 2.6 MB) |
| `probe/boundary/land_DE13_25833.xml` | the full state polygon from the ALKIS WFS (2.2 MB, 95,507 vertices) |
| `probe/caps/`, `probe/gfi/`, `probe/gfigrid/`, `probe/csw/`, `probe/web/`, `probe/geofabrik/`, `probe/osm/` | capabilities, GetFeatureInfo answers (incl. a 267-point statewide DOP date sample), ISO records, portal pages, Geofabrik state/poly, OSM API/Overpass answers |
| `scripts/` | helper scripts (UTM, TIFF tags, raster compare, GML attribute summary, shapefile-in-zip reader, boundary simplification and border checks, GetFeatureInfo grid) |

---

## 1. Brandenburg → Mecklenburg-Vorpommern at a glance

| freemapper product | Brandenburg (today) | Mecklenburg-Vorpommern equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25833 for every product** (tiles, WCS, WFS, OGC API). MV spans 10.59–14.41°E; in 25833 E 206,889–460,853. Convergence −3.4° (west edge) to −0.7° (east edge). Same zone as Brandenburg: a cross-border BB–MV map needs no reprojection. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **DOP20 TrueDOP**. Best path: **WCS** `inspire_oi_dop_wcs`, coverage `mv_dop20`, `FORMAT=image/jpeg` (4000 × 4000 px in 5.7 s, **no watermark**). The WMS `adv_dop`, layer `mv_dop` (max **4192 px**) burns **"© GeoBasis-DE/M-V" into the top-left corner of every image**. All three (WMS, WCS, RGB tiles) serve a **256-colour, dithered palette** version; full-colour pixels exist only in the 419 MB RGBI tiles. Vintages per 2 km tile 2021/2023/2024/2025, March–October (§4). | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1**, ALS. **WCS** `dgm_wcs`, coverage `mv_dgm` (Float32, bit-identical to the tiles, `GEOTIFF:COMPRESSION=LZW` works, no size limit seen). Or ATOM 2 km GeoTIFF (Float32, uncompressed, no ranges). **No NoData: sea, Bodden and areas without tiles are 0.0.** Vintages per tile 2012–2025. | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **DOM1 − DGM1, both via WCS** (1 m, Float32; DOM1 is dated like the DOP in 57 % of tiles, school: DOM 08.2024, DOP 14.08.2024). A ready **nDOM** (WCS `mv_ndom`, 1 m) exists but is older (2020–2023; school 06.2022). bDOM20/bDOM10light (LAZ) are open, not examined. ALS point clouds are paid. | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **OGC API** `https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/gebaeude_bauwerk/items` — **freemapper's LGB URL works unchanged** (`f=json&bbox=…&bbox-crs=…25833&crs=…25833&limit=10000`, `next` links). Also the WFS `alkis_wfs_einfach` (`ave:GebaeudeBauwerk`, GML, 25833 only). LGB spellings, `rellage` and `Bauteil` present, oid prefix **`DEMV`**. | verified |
| Land use | OGC API `nutzung_flurstueck` | same OGC API, **`nutzung_flurstueck`** (and `nutzung`). LGB spellings; school parcel `bez` **"Bildung und Wissenschaft"** (as LGB). `Weg` never has a `bez`; `Begleitfläche Straßenverkehr` on 1–4 % of Straßenverkehr; Platz mostly "Parkplatz"/"Fußgängerzone". | verified |
| Currency | `bb_*_aktualitaet.csv` | **`overview.php`** per 2 km tile (DOP flight date, DGM/DOM/nDOM month, bDOM, ALS), ≤ 500 tiles per call; WMS GetFeatureInfo `mv_dop_info` (flight date); `aktualit` per ALKIS feature | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `mecklenburg-vorpommern-latest.osm.pbf` (128.8 MB) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **CC BY 4.0**. Portal terms: **"©GeoBasis-DE/MV/CC BY 4.0"**, changed data **"©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)"**. Services, feeds and ISO records instead say "© GeoBasis-DE/M-V <Jahr der letzten Datenlieferung>" (§9). | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **WMS / WCS** | `https://www.geodaten-mv.de/dienste/<name>` | WMS `adv_dop`, `adv_dop_fruehjahr`, `adv_dop_sommer`, `adv_bfue_dop` (flight blocks), `gdimv_dopcir`, `doparchiv_wms`, `dgm_wms`, `dom_wms`, `ndom_wms`, `tdom_wms`; WCS `dgm_wcs`, `dom_wcs`, `ndom_wcs`, `inspire_oi_dop_wcs`, `tdom_wcs` | **MapServer** (error texts `msWMSLoadGetMapParams()`, `msWCSGetCoverage20()`). WMS 1.3.0, MaxWidth/Height 4192; WCS 1.0.0/1.1.1/2.0.1. Apache, CORS `*`, HSTS. Capabilities in 0.6–1.1 s |
| **ALKIS WFS** | same host, `alkis_wfs_einfach` (vereinfacht), `alkis_wfs_sf` (simple features, `adv:AX_*`), `alkis_wfs_nas` | ALKIS | AAA-suite / XtraServer (`content-disposition: … XtraServerGetFeature.xml`, schemas under `https://www.geodaten-mv.de/aaa-suite/schema/`). The `alkis_e_*` siblings (owner data) need a proven interest — not needed |
| **OGC API Features** | `https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1` | ALKIS vereinfacht: `flurstueck`, `flurstueck_punkt`, `gebaeude_bauwerk`, `katasterbezirk`, `nutzung`, `nutzung_flurstueck`, `verwaltungseinheit` | GeoJSON (`application/geo+json`), `Content-Crs` header, OpenAPI at `…/v1/api?f=json`; the same product as LGB's (inferred from identical collection ids, parameters and paging). ISO record `669dad4a-78e3-40b4-91ae-fd841402d3f0` |
| **ATOM + downloads** | `https://www.geodaten-mv.de/dienste/<p>_atom` → `…/<p>_download?index=<i>&dataset=<uuid>&file=<name>` | DOP20, DGM, DOM, nDOM, ALKIS, DVG, ALS (paid), … | Feeds list every 2 km tile × format. Downloads: **no Content-Length, no ETag/Last-Modified, Range ignored (200 + full body)**, chunked; a missing file → **404** `text/html` "Ungueltiger Wert '…' fuer Parameter 'file'." |
| **Downloadportal** | `https://laiv.geodaten-mv.de/afgvk/` | product pages, GAIA-light download maps; AJAX `…/_apps/dladownload/_ajax/overview.php`, `details.php`, `kachelsuche.php`, `polygon.php`, `date.php` | the download buttons simply link to the `…/dienste/<p>_download` URLs (from `js/dladownload-config-download-<type>.js`). "Anmelden / Registrieren" is only for paid orders |
| Product/licence pages | `https://www.laiv-mv.de/Geoinformation/…`, `…/static/LAIV/Geoinformation/Dateien/…` | product info, yearly currency PDFs and shapefile zips, terms PDFs | static files |
| Catalogue | `https://www.geodaten-mv.de/soapServices/api` (CSW 2.0.2), GeoMIS.MV `https://www.geoportal-mv.de/geomis/` | ISO records | GetRecordById 0.25 s; GetRecords with CQL works (4.7 s) |

### 2.1 Tile scheme

- **2 km × 2 km**, EPSG:25833, SW corner at **even** km, for DOP20, DGM1, DOM1, nDOM. Tile key `33_EEE_NNNN` (e.g. `33_312_5996`),
  the same string as the WMS metadata `Name`/`kachel_nr` and the `overview.php` `kachel_nr`. (ALS uses 1 km tiles in
  `overview.php`: `33_312_5996` covered E 312000–313000.)
- File names have **no year** (unlike SH): `dop20rgb_33_312_5996_2_mv.tif`, `dop20rgbi_…_2_mv.tif`, `dop20rgb_…_2_mv.ecw`,
  `dgm1_33_312_5996_2_gtiff.tif`, `dom1_…_2_gtiff.tif`, `ndom_33_312_5996_2_gtiff.tif`. Only `overview.php`'s `dop_name`
  carries the year (`dop20rgbi_33_312_5996_2_mv_2024`).
- Grid (from the ATOM feeds): E 206…460, N 5890…6060; **DOP20 6,627 tiles**, DGM1/DOM1 **6,407**, nDOM 6,406. All SW
  corners even.
- **Tiles are whole 2 km squares that touch MV, not clipped to the state.** At the Brüssow probe the tile row N 5918 exists
  and reaches 1.39 km into Brandenburg with full data (DGM, nDOM, DOP); the row N 5916 (fully in BB) does not exist.
- Download endpoints (`index` = position of the format in the dataset feed):

  | Product | URL base | `dataset` | index → file |
  |---|---|---|---|
  | DOP20 RGB (palette) | `…/dienste/dop20_download` | `k34d45fr-f78c-44g7-n2b6-6f67k7ahl92g` | 0 `dop20rgb_…_2_mv.tif`, 1 `.ecw` |
  | DOP20 RGBI | `…/dienste/dop20_download` | `f94d17fa-b29b-41f7-a4b8-6e10f1aae38e` | 0 `dop20rgbi_…_2_mv.tif` |
  | DGM1 | `…/dienste/dgm_download` | `ca268792-s2q1-4a39-b34c-9ec5bf9a4469` | 0 `_mix.tif`, 1 `_xyz.zip`, 2 `_schum_NW.tif`, 3 `_zcode.tif`, **4 `_gtiff.tif`**, 5 `_isoli.zip` |
  | DOM1 | `…/dienste/dom_download` | `us214578-a1n5-4v12-v31c-5tg2az3a2164` | 0 `_xyz.zip`, 1 `_zcode.tif`, 2 `_schum.tif`, 3 `_mix.tif`, **4 `_gtiff.tif`** |
  | nDOM | `…/dienste/ndom_download` | `66b85dc1-404c-4fab-b652-d2501962912c` | 0 `ndom_…_2_gtiff.tif` |

  The dataset ids are not UUIDs (letters beyond f); copy them verbatim. Example:
  ```
  https://www.geodaten-mv.de/dienste/dgm_download?index=4&dataset=ca268792-s2q1-4a39-b34c-9ec5bf9a4469&file=dgm1_33_312_5996_2_gtiff.tif
  ```

### 2.2 Downloadportal tile index (`overview.php`)

- `GET https://laiv.geodaten-mv.de/afgvk/_apps/dladownload/_ajax/overview.php?bbox=E1,N1,E2,N2&crs=EPSG:25833&type[]=<type>`
  → 200 `application/json` in 0.25–1.0 s: `{"success":true,"result":{FeatureCollection}}`, one polygon per tile (25833) with
  properties. `bbox[]=E1&bbox[]=N1&…` works too.
- Types that work: `dgm1`, `dgm5`, `dom`, `ndom`, `dop20`, `bdom20`, `bdom`, `bdom10light`, `als`. `dop`, `dom1` →
  `{"success":false,"message":"Interner Fehler"}`.
- Properties at the school tile:

  | type | properties |
  |---|---|
  | `dop20` | `gid 2236`, `aktualitaet 2024-08-14`, `meta_aktualitaet 2024-08-14`, `aktualitaet_d 14.08.2024`, `kachel_nr 33_312_5996`, `dop_name dop20rgbi_33_312_5996_2_mv_2024`, `image_org /daten/geodaten/geobroker/dop20_aktuell/rgbi/tiff/…_2024.tif` (+ `image_gaia_rgb`, `image_gaia_cir`) |
  | `dgm1` | `aktualitaet 2024-03-01`, `aktualitaet_d 03.2024`, `jahrgang 2024` |
  | `dom` | `aktualitaet 2024-08-01`, `08.2024` |
  | `ndom` | `aktualitaet 2022-06-01`, `06.2022` |
  | `bdom20` | `aktualitaet 2024-08-14` |
  | `als` | `aktualitaet 2024-03-01` (1 km tile) |

- **At most 500 objects** per call; more → HTTP 200 `{"success":false,"message":"Too many objects returned - max 500"}`.
  A statewide index needs ≈ 36 calls of 46 × 34 km per type (done: `probe/dla/*_index.json`, all `aktualitaet` ISO dates).
- Quirk: in the first pass 14 of 144 calls left no output file (no curl error, apparently an empty 200 body); all succeeded
  on retry. Retry on an empty or non-JSON body.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25833.**
  - GeoTIFF tiles and WCS output: `ProjectedCSType 25833`, GeoAscii "ETRS89 / UTM zone 33N", `GTRasterType 1` (PixelIsArea).
  - ALKIS WFS: DefaultCRS 25833, OtherCRS only **5650** (zone-prefixed 33). **25832 → HTTP 400** "The requested SRS
    "urn:ogc:def:crs:EPSG::25832" is not supported for FeatureType "ave:GebaeudeBauwerk"".
  - OGC API: `crs` CRS84 and 25833 only; `bbox-crs`/`crs` 25832 → **400 (HTML page)**.
  - WMS and WCS also serve 25832, 3857, 4326 and 15 more (not needed).
- **Height datum DHHN2016** (normal heights, EPSG:7837): product page "Höhe: DE_DHHN2016_NH, EPSG-Code: 7837"; ISO abstract
  "Höhenbezugssystem: DHHN2016 (Normalhöhennull (NHN), Amsterdamer Pegel)"; shapefile attribute `genau_hoeh 7837`. The TIFFs
  carry no vertical key.
- **Zone 33 fits.** MV lies 10.59–14.41°E, entirely west of the 15° central meridian (0.6° to 4.4° west of it), so the
  eastings stay positive and below 500,000. Grid convergence and scale:

  | Place | Convergence zone 33 / 32 | Scale k zone 33 / 32 |
  |---|---|---|
  | Rostock (school) | −2.33° / +2.53° | 1.00003 / 1.00011 |
  | Schwerin | −2.89° / +1.94° | 1.00029 / 0.99991 |
  | Boizenburg (west edge) | −3.44° / +1.38° | 1.00059 / 0.99976 |
  | Greifswald | −1.31° / +3.55° | 0.99974 / 1.00061 |
  | Ahlbeck (east edge) | −0.65° / +4.20° | 0.99963 / 1.00102 |

  Keep MV a **`Crs::Utm33` region, like Brandenburg** (no change to the default; no `Region::crs()` override).
- **Extent** of the state polygon (ALKIS `VerwaltungsEinheit` DE13, `aktualit 2026-11-06Z`):

  | CRS | Extent |
  |---|---|
  | EPSG:25833 | E 206889 – 460853, N 5890629 – 6060820 |
  | EPSG:25832 | E 606033 – 860288, N 5887211 – 6068689 |
  | lon / lat | lon 10.5936 – 14.4122, lat 53.1105 – 54.6845 |

  The WMS/WCS coverage envelope is wider: E 200000–465000, N 5886000–6075000 (WCS `boundedBy`).

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- Product page: DOP10 and DOP20, "Aktualitätszyklus 2,5 Jahre", "Die Aktualität für die Landesfläche liegt bei ca. drei
  Jahren", **TrueDOP since the 2018 flights**, since 2013 flown "sowohl innerhalb als auch außerhalb der belaubten
  Situation". DOP10 is order-only; DOP20 is the open product.
- Current DOP20 per 2 km tile (statewide `overview.php`, 6,627 tiles):

  | Year | Tiles | Flight months (tiles) |
  |---|---|---|
  | 2021 | 107 | 03 (64), 04 (43) |
  | 2023 | 1,369 | 04 (444), 05 (6), 06 (919) |
  | 2024 | 2,509 | 03 (194), 05 (1,177), 07 (47), 08 (413), 09 (678) |
  | 2025 | 2,642 | 03 (274), 04 (524), 05 (313), 06 (294), 07 (329), 08 (382), 09 (324), 10 (202) |

  So ≈ 23 % of tiles (1,543) are March/April (leaf-off or early), the rest leaf-on. The school tile is **14.08.2024** (leaf-on; flight
  "Bildflug 2024 Los2"), the Brüssow border tile 09.06.2023. The 2025 flights are already in the services.
- Seasonal services (useful for leaf-off templates):
  - `https://www.geodaten-mv.de/dienste/adv_dop_fruehjahr`, layer **`adv_dop_rgb_fruehjahr`** (MaxScale 1:15,001), metadata
    layer `adv_dop_fruehjahr_metadaten` (`meta_aktualitaet`, `meta_bildflugnr`, `kachel_nr`);
  - `…/adv_dop_sommer`, layer `adv_dop_rgb_sommer`, metadata `adv_dop_sommer_metadaten`.
  - At the school "Frühjahr" is **24.06.2022** (B2022_Los1, visually leaf-on) and "Sommer" 14.08.2024; so "Frühjahr" means the
    latest spring-programme flight, not necessarily leaf-off. A 267-point statewide sample of the spring layer: 2021 (42),
    2022 (51), 2023 (66), 2024 (47), 2025 (61); months 03–08.
  - **Quirk:** `adv_dop_rgb_fruehjahr` with `FORMAT=image/jpeg` answered with a **PNG** (2.2 MB, colormap) and the JPEG MIME
    type was not honoured; check the magic bytes.
- Flight-block shapefiles (per year, with dates) at
  `https://www.laiv-mv.de/static/LAIV/Geoinformation/Dateien/Aktualit%C3%A4t/Aktualit%C3%A4t%20DOP/DOP_2025.zip` (23 blocks,
  17.03.–20.10.2025), `…/DOP_2024.zip` (Los1 10 blocks March–May, Los2 13 blocks May–September). The `NAME`/`Befliegung`
  fields hold dates in mixed formats ("27.04.2025", "2.07.2025", "2024-09-07", one "2.05.204").
- Planned flights: WMS `adv_bfue_dop`, layer `dop_zukunft` → "2027" at the school (GetFeatureInfo).

### 4.2 WMS (`adv_dop`)

- `https://www.geodaten-mv.de/dienste/adv_dop`, WMS 1.3.0, "WMS Digitale Orthophotos MV (WMS_MV_DOP)", layer **`mv_dop`**
  (opaque, 25833 only listed on the sublayer; the parent `WMS_MV_DOP` lists 20 CRS), metadata layer `mv_dop_info`. No TIME
  dimension. Formats png, jpeg, png 24/8 bit, tiff. **MaxWidth/MaxHeight 4192**; 4193 → **HTTP 200 `text/xml`**
  "msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be between 1 and 4192 pixels."
  Unknown layer → 200 XML `LayerNotDefined`.
- Example (school, 400 × 300 m at 0.2 m):
  ```
  https://www.geodaten-mv.de/dienste/adv_dop?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=mv_dop&STYLES=&CRS=EPSG:25833&BBOX=311957,5996167,312357,5996467&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 496,588 B, **11.9 s** (no Content-Length, chunked). 4192² JPEG 2.8 MB in 32 s; 4000² JPEG 2.5–2.6 MB
  in 15–22 s; 4000² PNG 14.4 MB in 64 s. Slower than SH (4000² JPEG in 10.9 s there). 25832 also works.
- **Watermark:** every GetMap (also PNG, also the spring layer, also a 400 × 300 px sea tile) has **"© GeoBasis-DE/M-V"
  drawn at pixels x 9–95, y 9–17** (fixed size, independent of the image size). Apart from that the PNG is **pixel-identical
  to the palette tile** (4000² px compared, 0 differences outside the watermark, 8 grey levels mean error at ±1 px shift).
  freemapper's tiling into 4000 px pieces would stamp the text into every piece. Workarounds: use the WCS (§4.3), or
  request each piece ≥ 20 px larger on the top and left and crop.
- Outside the tile coverage: JPEG white, PNG transparent (except the watermark).

### 4.3 WCS (`inspire_oi_dop_wcs`, recommended)

- `https://www.geodaten-mv.de/dienste/inspire_oi_dop_wcs`, "INSPIRE-WCS MV Orthofotografie Digitales Orthophoto", coverage
  **`mv_dop20`**, grid origin (200000.1, 6074999.9) step 0.2 m, 1,325,000 × 945,000 px, 25833.
  AccessConstraints: "…CC BY 4.0… Die Namensnennung hat in folgender Weise zu erfolgen: © GeoBasis-DE/M-V <Jahr der letzten
  Datenlieferung>".
- Example:
  ```
  https://www.geodaten-mv.de/dienste/inspire_oi_dop_wcs?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=mv_dop20&FORMAT=image/jpeg&SUBSET=x(314000,314800)&SUBSET=y(5996000,5996800)
  ```
  → 200 `image/jpeg`, 4000 × 4000, 2,625,982 B, **5.7 s**. **No watermark.**
- `FORMAT=image/tiff`: 3 × 8 bit RGB, uncompressed strips; 2000 × 1500 = 9.0 MB in 46.8 s (under parallel load); tiepoint
  exactly the requested corner (311957, 5996467), pixel scale `0.2000000000000233` (float noise; round). With
  `&GEOTIFF:COMPRESSION=JPEG` → JPEG-in-TIFF, 4000² 7.5 MB in 35 s.
- Pixels **identical to the palette tile** (2000 × 1500 compared, mean abs diff 0.00 at offset 0, ≈ 8.6 at ±1 px).
- Outside the tile coverage the fill is **white (255,255,255)**; no alpha.
- freemapper: the existing WMS code path cannot be reused as is (different parameters), but the grid is exactly the request
  (verified with the TIFF tiepoint), so a JPEG answer can be placed like a WMS piece. Piece size limit not found (4000² OK).

### 4.4 Tile downloads (ATOM)

- RGB (`dop20rgb_…_2_mv.tif`): 10000 × 10000, **1 band, Photometric 3 (palette), 256-entry colormap**, uncompressed, 512 px
  internal tiles, **6 internal overviews** (5000 … 157 px), tiepoint exactly (312000, 5998000), 0.2 m, 25833, no NoData.
  **141,574,244 B**; 121 s alone, 281 s with parallel requests (0.5–1.2 MB/s). Also as ECW.
- RGBI (`dop20rgbi_…`): 10000 × 10000 × **4 × 8 bit**, RGB + ExtraSamples 0, uncompressed, 512 px tiles, Trimble
  "OrthoVistaProcessing ImageWriter", IFD at the start; **≈ 419.4 MB** (the 400 tile byte counts sum to 419,430,400 B). Only
  the first 1.06 MB were read.
- No ranges, no Content-Length, so neither tile can be read partially in a controlled way (a client can stop reading early,
  as done for the RGBI header). **Not recommended.**

### 4.5 Colour reduction

- The palette data (tiles, WMS, WCS) use **242–251 distinct colours with dithering**. Against the RGBI pixels (first 512² px of
  the school tile): mean abs diff 3.7 grey levels, max 55, **PSNR 33.8 dB**, 62,437 → 242 colours. Visible as fine noise on
  roofs and lawns (`dl/preview/rgbi_vs_palette.png`), fine for a mapping template. The CIR WMS and the RGBI tiles are the
  only full-colour sources.

---

## 5. DGM1 (terrain model, 1 m)

- From ALS, "Aktualitätszyklus ca. 10 Jahre". Vintage per 2 km tile (`overview.php`, 6,407 tiles):

  | Year | Tiles | Year | Tiles |
  |---|---|---|---|
  | 2012 | 7 | 2019 | 697 |
  | 2015 | 91 | 2020 | 475 |
  | 2016 | 1,006 | 2022 | 805 |
  | 2017 | 568 | 2023 | 1,010 |
  | 2018 | 671 | 2024 | 1,012 |
  | | | 2025 | 65 |

  School tile **03.2024**, border tile 03.2017. The portal "Datenaktualisierungen" page says "DGM1 21.01.2026".

### 5.1 WCS (`dgm_wcs`, recommended)

- `https://www.geodaten-mv.de/dienste/dgm_wcs`, "WCS Digitales Geländemodell MV (WCS_MV_DGM)", coverages `mv_dgm` (1 m),
  `mv_dgm5`, `mv_dgm25`. Grid origin (200000.5, 6074999.5) = cell centres, 265,000 × 189,000 cells, range Float32
  (`swe:interval -3.4028e+38 3.4028e+38`, `nilValues` empty).
- Example:
  ```
  https://www.geodaten-mv.de/dienste/dgm_wcs?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=mv_dgm&FORMAT=image/tiff&SUBSET=x(311957,312357)&SUBSET=y(5996167,5996467)
  ```
  → 200 **`image/tiff` (a plain TIFF, not `multipart/related`)**, 400 × 300 Float32, 480,772 B, 0.8 s; tiepoint exactly
  (311957, 5996467), pixel 1 m, 25833 GeoKeys, **no GDAL_NODATA**. Uncompressed strips.
  - Compression: `&GEOTIFF:COMPRESSION=LZW` → LZW (2 km: 10.6 MB in 10.6 s instead of 16.0 MB in 14.0 s);
    `&GEOTIFF:COMPRESSION=Deflate&GEOTIFF:PREDICTOR=FloatingPoint` → 8.3 MB. Both decode to identical values.
  - Size: 4000² (64 MB, 55 s), 5000² (100 MB, 86 s), **8000² (256 MB, 369 s)** all answered; no limit found.
  - Outside the coverage envelope → **400** `ows:ExceptionReport` `ExtentError` "msWCSGetCoverage20(): WCS server error. Image
    extent does not intersect with desired region."; a request half outside the envelope is **clipped** (1000 m requested,
    500 px returned), so always use the returned tiepoint and size.
  - **Inside the envelope but without a tile (e.g. Brandenburg beyond the MV tiles, open sea) the WCS returns 0.0**, not NoData.
- Values are **bit-identical to the ATOM tiles** (2000 × 2000 compared, max diff 0).
- No gzip (`Accept-Encoding: gzip` ignored on the WCS). Throughput ≈ 1.1–1.5 MB/s.
- Differences from freemapper's LVermGeo WCS code (`lvermgeo.rs`): coverage id `mv_dgm` instead of `Coverage1`; the answer is a
  plain TIFF, so `multipart_tiff` would refuse it; the NoData rule (−9999) never fires; axis labels `x`/`y` match.

### 5.2 ATOM GeoTIFF (alternative)

- `…/dgm_download?index=4&dataset=ca268792-s2q1-4a39-b34c-9ec5bf9a4469&file=dgm1_33_EEE_NNNN_2_gtiff.tif` → 200 `image/tiff`,
  `Content-Disposition: attachment`, **16,024,498 B**, 13.8 s alone (45 s in parallel with others).
- TIFF: 2000 × 2000 **Float32** (SampleFormat 3), **uncompressed, 2000 strips of 1 row**, **IFD at the end** (offset
  16,012,146), ModelPixelScale 1, **tiepoint exactly (312000, 5998000)**, PixelIsArea, 25833, **no NoData tag, no TFW**. The
  equivalent TFW is `1/0/0/-1/312000.5/5997999.5`: the same registration as LGB/GeoSN DGM1.
- Feed: `https://www.geodaten-mv.de/dienste/dgm_atom?type=dataset&id=ca268792-s2q1-4a39-b34c-9ec5bf9a4469` (15.2 MB, 27.8 s,
  38,442 links = 6,407 tiles × 6 formats, `updated 2025-02-01`). Names are predictable from the tile key, so the feed is not
  needed.

### 5.3 Values, water and the missing NoData

- School tile: −0.10 … 24.73 m, median 4.60 m; **18.7 % of the cells are exactly 0.0** (the Warnow). School site 9.93–16.44 m.
- **Sea, Bodden and the tidal Warnow are 0.000** (Warnemünde probe: 99.8 % zeros in the coastal tile). **Inland lakes are
  flattened to their water level**: Schweriner See 37.50 m, Müritz 61.80 m (whole 2 km all 61.80).
- So 0.0 is a legitimate height on the coast, and also the WCS fill outside the tiles. freemapper must **mask cells outside
  the MV tile coverage** (the 2 km tiles from the ATOM feed or `overview.php`, or the state polygon plus one tile) instead of
  treating 0 as NoData. The ATOM tiles have no NoData at all (0 % in the school and border tiles).
- GetFeatureInfo on `dgm_wms` (`hoehendarstellung`) returns the same value (`value_0 = '12.542'` at E 312157.5 / N 5996316.5,
  the tile has 12.542).

### 5.4 Border with Brandenburg

- At the Brüssow probe the WCS gives full DGM values for the MV tile row N 5918–5920, also on its 70 % Brandenburg part (valid
  cells reach 1,389 m from the MV border, median 542 m), and **0.0 for the row south of N 5918000** (no tile). A BB–MV map must
  take LGB's DGM1 south of the MV tile edge (and should prefer it inside BB).
- LGB and MV DGM1 on the overlapping cells were not compared.

---

## 6. nDSM sources: DOM1, nDOM, bDOM, LiDAR

### 6.1 DOM1 (1 m) — recommended together with DGM1

- Product: "DOM1", 1 m, "Grundaktualität von 3 Jahren", AdV DOM standard; produced from ALS or image matching. Per-tile
  vintages (`overview.php`): **2022: 1,225, 2023: 2,451, 2024: 2,731**. The DOM month equals the DOP flight month in 3,776 of
  6,627 tiles; the 2024 coverage shapefile has `erf_method 5040` where DGM/ALS have `5020`. **So DOM1 is most likely
  image-matched from the DOP flights** (inferred). School: DOM 08.2024, DOP 14.08.2024.
- WCS `https://www.geodaten-mv.de/dienste/dom_wcs`, coverage **`mv_dom1`** (also `mv_dom25`), same grid and behaviour as
  `mv_dgm` (2 km Float32 16.0 MB in 14.0 s). Also ATOM `dom_download?index=4&dataset=us214578-a1n5-4v12-v31c-5tg2az3a2164&file=dom1_33_EEE_NNNN_2_gtiff.tif`
  (16,024,380 B, Float32, uncompressed, identical to the WCS), and `index=0` `_xyz.zip` (17.9 MB, one 160 MB `.xyz`,
  **unordered lines** `  312000.500  5996001.500       16.516\r\n`).
- **DOM1 − DGM1** at the school tile: 9.7 % of cells < 0 (clamp), 37.5 % > 2 m; school building bbox median 2.4 m, p90 15.5 m.

### 6.2 nDOM (1 m, ready-made)

- WCS `https://www.geodaten-mv.de/dienste/ndom_wcs`, coverage **`mv_ndom`**; ATOM
  `ndom_download?index=0&dataset=66b85dc1-404c-4fab-b652-d2501962912c&file=ndom_33_EEE_NNNN_2_gtiff.tif`.
- ATOM TIFF: 2000 × 2000 Float32, **LZW, 64 × 64 internal tiles**, `GDAL_NODATA 3.4028234663852886e+38` (FLT_MAX; tifffile
  warns it cannot cast it), Software "opalsDiff by Department of Geodesy and Geoinformation, E120.7 TU Wien", GDAL statistics
  in `GDAL_METADATA`; 13,374,079 B. The WCS returns the same values (uncompressed, no NoData tag).
- **Older than DOM1/DGM1**: nDOM vintages 2012 (3), 2020 (1,081), 2021 (930), 2022 (1,668), 2023 (2,724). School **06.2022**.
  Against DOM1 − DGM1 at the school tile: ground agrees (median 0.002 m), but 17.6 % of cells differ by > 1 m (trees, changes);
  1.0 % are objects now but not in the nDOM, 1.8 % the reverse. Values −5.98 … 97.81 m.
- The nDOM ISO record (`6230a8b1-…`) is stale ("Es gelten Zugriffseinschränkungen. Für die Nutzung können Kosten entstehen.",
  link to an old AGNB PDF), while the portal, the WMS/WCS (`Fees`: "Es gelten keine Bedingungen") and the open-data page list
  it as free. Treat it as CC BY 4.0 like the rest (it is on the open download page).

### 6.3 bDOM and LiDAR

- bDOM (image matching, 0.1 m point cloud): **bDOM20** (LAZ, 0.2 m) and **bDOM10light** (LAZ without RGB) are "Zum Download"
  (open), bDOM10 with RGB is "Zur Bestellung". `overview.php` `bdom20` at the school: 14.08.2024. Not downloaded; LAZ would need
  a new reader.
- **ALS point clouds are paid** ("Hinweis: Dieses Produkt ist entgeltpflichtig", ATOM `als_atom` with access restrictions).
  Not needed.
- tDOM (difference models) exists (WMS/WCS); `tdom_wcs` says "Für die Nutzung können Kosten anfallen". Not needed.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **`overview.php`** (`https://laiv.geodaten-mv.de/afgvk/_apps/dladownload/_ajax/overview.php?bbox=…&crs=EPSG:25833&type[]=<t>`, ≤ 500 tiles, 0.25–1 s) | `dop20`, `dgm1`, `dom`, `ndom`, `bdom20`, `als` | `aktualitaet` (ISO date; DOP = flight day, DGM/DOM/nDOM = first of the month), `aktualitaet_d`, `jahrgang`, `kachel_nr`; DOP also `meta_aktualitaet` (differs from `aktualitaet` in 557 tiles) and the file name with year. Undocumented |
| WMS GetFeatureInfo `adv_dop` / `mv_dop_info` | DOP20 | `Name 33_312_5996`, `Nummer 'Bildflug 2024 Los2'`, `Befliegungsjahr`, **`Befliegungszeitraum`** (flight date), CSW links. INFO_FORMAT `text/plain`, `application/vnd.ogc.gml` (msGMLOutput with the 2 km tile box); `application/json` → 200 `text/xml`. 0.4–2.5 s. **Date format mixed**: ISO `2024-08-14` in most tiles, `22.04.2023` (D.MM.YYYY) in 27 of 267 sampled tiles |
| GetFeatureInfo `adv_dop_fruehjahr_metadaten`, `adv_dop_sommer_metadaten` | spring/summer DOP | `meta_aktualitaet` (also mixed formats), `meta_bildflugnr`, `kachel_nr` |
| Yearly coverage shapefiles (laiv-mv.de, `…/Aktualit%C3%A4t/Aktualit%C3%A4t%20<P>/<P>_<year>.zip`) | DOP (flight blocks with dates), DGM, DOM, nDOM, ALS, bDOM | one dissolved polygon per year for DGM/DOM/nDOM/ALS (fields `kachel_nr` of one sample tile, `aktualitae`, `erf_method`, `jahrgang`); `DGM_2024.zip` has **zone-prefixed eastings** (33220000…) and a different field set; `DGM_2025.zip` contains a stray 7.9 MB `nDOM_2023.pdf`; `nDOM_2024.zip` is a copy of `DOM_2024.zip` (same `gid`, `produkt DOM`) |
| Portal "Datenaktualisierungen" (`https://laiv.geodaten-mv.de/afgvk/Sonstiges/Datenaktualisierungen`) | all | product-level dates: DOP20 05.05.2025, DGM1 21.01.2026, DOM 04.07.2024, nDOM 16.04.2025, bDOM 20.10.2025, ALS 17.12.2025, ALKIS 26.10.2026 (these lag behind the tile dates, e.g. 2025 DOP flights are online) |
| ATOM `updated` | feeds | DOP 2025-10-20, DGM 2025-02-01, DOM 2024-09-01, nDOM 2023-06-01 (stale) |
| HTTP | downloads | **no ETag, no Last-Modified** on any download or WCS answer |
| ALKIS `aktualit` | per feature | WFS `2024-04-22Z`, OGC API `2020-03-31T07:08:53Z`. Rostock+Schwerin buildings: 3,414 of 9,777 from 2014 (migration), the rest 2015–2026 |

Suggested mapping:
- orthophoto `currency` = `overview.php` `dop20.aktualitaet` per tile under the map (mixed years and seasons are normal);
- DGM1 `currency` = `dgm1.aktualitaet` (month precision); `published` = none available (no ETag);
- nDSM: `dom.aktualitaet` and `dgm1.aktualitaet`; warn when the DOM month differs from the DOP flight month (43 % of tiles).

---

## 8. ALKIS: buildings and actual land use

### 8.1 Services

**(a) OGC API Features (recommended): `https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1`**
- Title "Daten des Liegenschaftskatasters - ALKIS Vereinfacht"; collections `flurstueck`, `flurstueck_punkt`,
  `gebaeude_bauwerk`, `katasterbezirk`, `nutzung`, `nutzung_flurstueck`, `verwaltungseinheit`; `storageCrs` 25833; CRS list
  `OGC/1.3/CRS84`, `OGC/0/CRS84`, `EPSG/0/25833`.
- **freemapper's `alkis::collection_url_in` URL works unchanged** with this endpoint:
  ```
  https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/gebaeude_bauwerk/items?f=json&bbox=311957,5996167,312357,5996467&bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25833&crs=http://www.opengis.net/def/crs/EPSG/0/25833&limit=10000
  ```
  → 200 `application/geo+json`, `Content-Crs: <http://www.opengis.net/def/crs/EPSG/0/25833>`, 131 features (= WFS), 74 KB,
  5.5 s cold / 0.45 s warm. `numberMatched`/`numberReturned` present; paging by `rel=next` links (`…&limit=3&offset=3`).
  OpenAPI: `limit` maximum **10000** (default 10); `limit=20000` is accepted silently.
- 2 × 2 km buildings: 5,885 features, 3.47 MB, 3.5–24.6 s (server load varies); with `Accept-Encoding: gzip` 0.63 MB in
  1.6–2.3 s. NutzungFlurstueck 2 × 2 km: 6,526 features, 3.84 MB, 14.8 s.
- Errors: **400 with an HTML page** (bad bbox, 25832 CRS), even with `f=json`.
- Feature: `id` = oid, `geometry` MultiPolygon, properties as below (`anzahlgs` is a number here, a string in the WFS).

**(b) WFS `https://www.geodaten-mv.de/dienste/alkis_wfs_einfach`** — "WFS_MV_ALKIS_Vereinfacht", AAA-suite, WFS 2.0.0.
- Types `ave:Flurstueck`, `ave:FlurstueckPunkt`, `ave:GebaeudeBauwerk`, `ave:KatasterBezirk`, `ave:Nutzung`,
  `ave:NutzungFlurstueck`, `ave:VerwaltungsEinheit`. **Ad-hoc `TYPENAMES`/`BBOX` queries work** (no stored-query trap as in SH).
- CRS 25833 (+5650); **25832 → 400**. Outputs GML 3.1/3.2 and `application/x-zip-shapefile`; `application/json` → 400 "The
  output format "application/json" is unknown."
- `CountDefault 30000`; `COUNT=40000` → 400 "Parameter 30000 exceeds configurated limit (MaxFeaturesLimit or CountLimit: )".
  `ImplementsResultPaging FALSE`, but `SORTBY=oid&COUNT=3&STARTINDEX=0/3` gave consecutive pages and
  `STARTINDEX=12000` of 31,867 (6 × 6 km) worked (1.9 s). **SORTBY oid uses a case-insensitive collation**
  (`…eboeb, …eboeC, …eboEd, …eboEE, …eboeg`); ties between oids that differ only in case could reorder across pages (not seen).
- `RESULTTYPE=hits` works (0.3 s). gzip supported (`content-encoding: gzip`; 6.2 MB → 0.72 MB).
- Errors: **HTTP 400** + `ows:ExceptionReport`.
- Speed: school buildings 135 KB 0.5 s; 2 × 2 km buildings 6.2 MB in 6.3–30 s; NutzungFlurstueck 7.1 MB 10.9 s; Schwerin
  1.5 × 1.5 km buildings 4.0 MB 21.7 s, NF 5.3 MB 32.5 s; Rostocker Heide NF (287 big polygons) 14.1 s.
- GML: default namespace `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0` (no `ave:` prefix on the
  feature elements), `gml:MultiSurface` → `surfaceMember` → `Polygon` → `exterior`/`interior` → `LinearRing` → `posList`
  **without `srsDimension`**, E N order, 3 decimals. Works with freemapper's `alkis_wfs` reader shape (inferred; not run).

Working WFS example (the shape of freemapper's `alkis_wfs` URL):
```
https://www.geodaten-mv.de/dienste/alkis_wfs_einfach?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=311957,5996167,312357,5996467,urn:ogc:def:crs:EPSG::25833&SORTBY=oid&COUNT=10000&STARTINDEX=0
```

Other ALKIS offers (not needed): `alkis_wfs_sf` (simple features `adv:AX_*`, 126 KB capabilities), `alkis_wfs_nas`, WMS
`alkis_wms`, ATOM `alkis_flur_atom` (NAS per municipality), statewide shapefiles `AAA2Shape_LandMV.zip` (quarterly), INSPIRE
BU/CP/ELU views.

### 8.2 Attributes compared with LGB

- `gebaeude_bauwerk` / `GebaeudeBauwerk`: `oid`, `aktualit`, `gebnutzbez`, `funktion`, `gfkzshh`?, `anzahlgs`?,
  `lagebeztxt`?, `rellage`?, `name`?, `gmdschl`. **Same names and spellings as LGB.**
- `nutzung_flurstueck`: `oid` (34 characters: Nutzung oid + Flurstück oid), `aktualit`, `nutzart`, `bez`?, `name`?, `flstkennz`.
- oids start with **`DEMV`** (`DEMVAL03…` Rostock, `DEMVAL04…` Schwerin, `DEMVHR03…` for a few Rostock objects, suffix `BL`
  buildings, `TN`/`TF` land use), so `Region::of_object_id` gets a `"DEMV"` arm. The state VerwaltungsEinheit is `DE13`.

### 8.3 Sample features at the ISG (WFS, geometry elided)

```json
{"oid":"DEMVAL03000bhHpnBL","aktualit":"2020-03-31Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","anzahlgs":"4","gmdschl":"13003000","lagebeztxt":"Goetheplatz 5"}
{"oid":"DEMVAL03000AsBvGBL","aktualit":"2020-03-31Z","gebnutzbez":"Gebäude","funktion":"Sport-, Turnhalle","gfkzshh":"31001_3211","anzahlgs":"1","gmdschl":"13003000"}
{"oid":"DEMVAL03000AsBvrBL","aktualit":"2024-04-22Z","gebnutzbez":"Bauteil","funktion":"Abweichende Geschosshöhe","anzahlgs":"1","gmdschl":"13003000"}
{"oid":"DEMVAL03000bhHqABL","aktualit":"2014-11-28Z","gebnutzbez":"Gebäude","funktion":"Schuppen","gfkzshh":"31001_2723","gmdschl":"13003000","lagebeztxt":"Goetheplatz 5 2"}
{"oid":"DEMVAL03000xGDpGBL","aktualit":"2019-02-12Z","gebnutzbez":"Sonstiges Bauwerk oder sonstige Einrichtung","funktion":"Sonstiges","gfkzshh":"51009_9999","gmdschl":"13003000"}
{"oid":"DEMVAL03000bkdfbDEMVAL03Z00008UpTF","aktualit":"2021-08-25Z","nutzart":"Fläche besonderer funktionaler Prägung","bez":"Bildung und Wissenschaft","flstkennz":"13224100702893______"}
{"oid":"DEMVAL03000FbzckDEMVAL03Z00008TqTF","aktualit":"2021-08-25Z","nutzart":"Straßenverkehr","flstkennz":"13224100702889______"}
{"oid":"DEMVAL0300btrcapDEMVAL0300btrddrTF","aktualit":"2025-06-16Z","nutzart":"Bahnverkehr","bez":"Eisenbahn","flstkennz":"132241007027500090__"}
```

The same school building from the OGC API: `{"oid":"DEMVAL03000bhHpnBL","aktualit":"2020-03-31T07:08:53Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","anzahlgs":4,"gmdschl":"13003000","lagebeztxt":"Goetheplatz 5"}`.
The school building has no `name` (unlike SH); `name` is set on 43 of 9,777 buildings (Marienkirche, Steintor, Universität,
Schweriner Schloss, Mecklenburgisches Staatstheater, also "Fahrstuhl", "Tunnel").

### 8.4 Observed values (Rostock 2 × 2 km + Schwerin centre: 9,777 buildings, 11,321 NutzungFlurstueck; Rostocker Heide 287 NF; border probe 5 buildings / 163 NF)

- **`gebnutzbez`** (LGB spellings, umlauts): `Gebäude` 6,702, `Bauteil` 1,716, `Sonstiges Bauwerk oder sonstige Einrichtung`
  1,285, `Bauwerk oder Anlage für Sport, Freizeit und Erholung` 62, `Bauwerk oder Anlage für Industrie und Gewerbe` 8 (+4
  wind turbines at the border), `Vorratsbehälter, Speicherbauwerk` 3, `Turm` 1. freemapper's kind rule (`Gebäude`) works
  unchanged.
- **`rellage`** present: `Unter der Erdoberfläche` 49 (Tiefgarage etc.), `Aufgeständert` 6. LGB spelling.
- `funktion` (123 values, detailed AdV names): Wohnhaus 2,445, Wohngebäude mit Handel und Dienstleistungen 722, Geringergeschossiger
  Gebäudeteil 703, Schuppen 617, Sonstiges 598, Überdachung 562, Wohngebäude 456, Garage 407, Durchfahrt im Gebäude 398,
  Auskragender Geschossteil 301, … Tiefgarage 44, Hochschulgebäude 28, **Allgemein bildende Schule 23**, Sport-, Turnhalle 19.
  No `;`-joined values seen.
- **`nutzart`** (LGB spellings, 18 values): Wohnbaufläche 5,495, Straßenverkehr 1,451, Sport-, Freizeit- und Erholungsfläche
  1,151, Industrie- und Gewerbefläche 1,109, Weg 732, Fläche besonderer funktionaler Prägung 653, Platz 458, Bahnverkehr 180,
  Wald 159, Fließgewässer 59, Landwirtschaft 58, Stehendes Gewässer 55, Unland/Vegetationslose Fläche 13, Hafenbecken 10,
  Sumpf 9, Gehölz 7, Friedhof 6, Schiffsverkehr 3.
- **`bez` per `nutzart`** (Rostock / Schwerin):

  | nutzart | bez |
  |---|---|
  | Straßenverkehr | none 969 / 434; **Begleitfläche Straßenverkehr 43 / 5** (4 % / 1 %) |
  | Platz | **Parkplatz 136 / 136**; **Fußgängerzone 84 / 42**; none 31 / 29 |
  | Weg | **always none** (544 / 134; no "Rad- und Fußweg") |
  | Fläche besonderer funktionaler Prägung | Bildung und Wissenschaft 138 / 38, Regierung und Verwaltung 98 / 78, Soziales 83 / 46, Kultur, Religiöse Einrichtung, Gesundheit, Kur, Öffentliche Zwecke 9 / 6, Sicherheit und Ordnung |
  | Sport-, Freizeit- und Erholungsfläche | Grünanlage, Park, Garten, Kleingarten, Freizeitanlage, Spielplatz, Bolzplatz, Sportanlage, Erholungsfläche |
  | Industrie- und Gewerbefläche | Handel und Dienstleistung, Industrie und Gewerbe, Kraftwerk, Heizwerk, … |
  | Bahnverkehr | Straßenbahn, Eisenbahn |
  | Wald (Heide) | Nadelholz 88, Laubholz 59, Laub- und Nadelholz 12 |
  | Wohnbaufläche | never |

- `name` on NutzungFlurstueck: 2 of 11,608 ("Rosengarten"); streets carry no names in this product.

### 8.5 What is missing or different

- `Weg` has no `bez` at all and `Begleitfläche Straßenverkehr` is rare, so freemapper's `VERGE` rule fires seldom; `Platz` is
  well classified (better than SH/Saxony).
- No `name` on most buildings (schools unnamed).
- WFS: GML only, 25833 only; OGC API: GeoJSON, 25833/CRS84 only. MV maps are 25833 projects anyway.
- **ALKIS stops at the state border**: in the Brüssow box the 5 buildings and 163 land-use parcels all lie inside MV (max
  vertex distance outside the 25 m asset 0 m for buildings, 18 m for parcels = simplification tolerance); `gmdschl 13075032`.

---

## 9. Licence and attribution

- Licence: **Creative Commons Namensnennung 4.0 International (CC BY 4.0)** for the "OpenData-Geobasisdaten" (LAiV open-data
  page: "Die frei zugänglichen Geobasisdaten – die sogenannten OpenData-Geobasisdaten – unterliegen den Bedingungen der Lizenz
  „Creative Commons BY 4.0" (CC-Namensnennung, CC-BY)"); ISO records of DGM/DOM: `{"id":"cc-by/4.0", …, "quelle": "©
  GeoBasis-DE/M-V <Jahr der letzten Datenlieferung>"}`.
- Downloadportal terms `https://laiv.geodaten-mv.de/afgvk/Sonstiges/Nutzungsbedingungen` ("Stand 09. Juni 2024"), verbatim:
  > (2) Alle übrigen Datenbereitstellungen und –nutzungen sind unter den Bedingungen der "Creativ Commons – Namensnennung 4.0
  > International (CC-BY 4.0)" zulässig.
  > …
  > Bei der Nutzung ist sicherzustellen, dass folgende Angaben als Quellenvermerk enthalten sind: Bezeichnung des
  > Bereitstellers nach dessen Maßgabe, der Vermerk „Creativ Commons – Namensnennung 4.0 International" oder „CC BY 4.0"mit
  > Verweis auf den Lizenztext unter http://creativecommons.org/licenses/by/4.0 sowie einen Verweis auf den Datensatz (URI).
  > Dies gilt nur soweit die datenhaltende Stelle die Angaben 1. bis 3. zum Quellenvermerk bereitstellt.
  > Veränderungen, Bearbeitungen, neue Gestaltungen oder sonstige Abwandlungen sind im Quellenvermerk mit Hinweis zu versehen,
  > dass die Daten geändert wurden.
  > …
  > Quellenvermerk für lizenzfreie Datennutzungen
  > Quellenvermerk gemäß Abs. c Nr. 1 und Nr. 2
  > Der Lizenznehmer ist verpflichtet, bei jeder öffentlichen Wiedergabe, Verbreitung oder Präsentation der Daten, sowie bei
  > jeder Veröffentlichung oder externer Nutzung einer Bearbeitung oder Umgestaltung einen deutlich sichtbaren Quellenvermerk
  > anzubringen, der wie folgt auszugestalten ist:
  > **©GeoBasis-DE/MV/CC BY 4.0**
  > **©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)**
  The same page carries an OSM permission (26.09.2025): the OSM contributors page counts as adequate attribution and ODbL
  "Parallel Distribution" is accepted.
- Other spellings in use:
  - service capabilities (`AccessConstraints`), ATOM `<rights>`: "… deutlich sichtbaren Quellenvermerk anzubringen, der wie
    folgt auszugestalten ist: **© GeoBasis-DE/M-V <Jahr der letzten Datenlieferung>**" (with hyphen, with year);
  - the WMS watermark: "© GeoBasis-DE/M-V";
  - the AGNB (paid data, `240609_VGB_AGNB.pdf`): "© GeoBasis-DE/MV <Jahr der letzten Datenlieferung>".
- Service `Fees`: "Es gelten keine Bedingungen".
- **Recommended freemapper line:** `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)` (the terms' form for changed data), with the
  licence link `https://creativecommons.org/licenses/by/4.0/` in the project notes. Adding the year of the data (e.g.
  `© GeoBasis-DE/M-V 2026`, the service form) is optional; the terms do not ask for it. Dataset URIs are optional (the terms
  require them only where the provider supplies them).

---

## 10. State boundary (for `assets/regions/mecklenburg-vorpommern.geojson`)

- **Source:** the ALKIS WFS, `ave:VerwaltungsEinheit` filtered on `art = Bundesland`, directly in 25833:
  ```
  https://www.geodaten-mv.de/dienste/alkis_wfs_einfach?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&FILTER=<fes:Filter xmlns:fes="http://www.opengis.net/fes/2.0" xmlns:ave="http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0"><fes:PropertyIsEqualTo><fes:ValueReference>ave:art</fes:ValueReference><fes:Literal>Bundesland</fes:Literal></fes:PropertyIsEqualTo></fes:Filter>
  ```
  (URL-encode the FILTER) → 200, 1 feature, **2,204,747 B, 11.5 s**. oid **`DE13`**, `name Mecklenburg-Vorpommern`,
  `aktualit 2026-11-06Z`. Alternatives: OGC API `verwaltungseinheit`; DVG WFS `dvg_laiv_wfs` (`dvg:kreise`, GeoJSON output);
  BKG VG25.
- Geometry: MultiSurface, **106 polygons** (the main polygon 22,819.8 km² includes Rügen and Fischland-Darß-Zingst, because
  the Bodden waters and the Strelasund belong to municipalities; separate polygons are Usedom 371.6 km², Poel 36.1 km²,
  Ummanz 19.4 km², Hiddensee 18.7 km² and further islands and islets), 95,507 vertices, valid; 32 interior rings, **31 of them slivers ≤ 84 m²** and one real hole of **35,864 m² at
  54.3927°N 12.6750°E** (Bodden water near Zingst with no parcels: WFS Flurstueck/Nutzung/VerwaltungsEinheit there → 0).
  Area 23,291.8 km².
- **Asset:** `mv/mecklenburg-vorpommern.geojson` (script `scripts/simplify_mv.py`):
  1. drop interior rings < 1,000 m² (the 31 slivers);
  2. **`shapely.simplify(25, preserve_topology=True)`** on the whole MultiPolygon (topology-preserving);
  3. round to integer metres, remove repeated points, drop polygons < 100 m² (20 islets of 1–108 m²);
  4. re-check validity.
  Result: FeatureCollection, `crs` `urn:ogc:def:crs:EPSG::25833`, properties `name: Mecklenburg-Vorpommern`, `source: LAiV
  M-V alkis_wfs_einfach (VerwaltungsEinheit DE13), © GeoBasis-DE/M-V 2026, CC BY 4.0`, `retrieved 2026-11-08`,
  `simplified_m 25`; **MultiPolygon, 86 polygons, 1 hole, 9,231 vertices, 157.7 KB, `is_valid` True**; area 23,292.1 km²
  (+0.0012 %). Extent E 206,889–460,853, N 5,890,646–6,060,820. The original vertices of the kept polygons are within
  p50 4.7 m / p99 22.7 m of the simplified outline. The file is about twice the size of the Brandenburg asset (82.9 KB) because
  of the Bodden coast and islands; a 50 m tolerance would roughly halve it (not tested).
- **Shared border with `assets/regions/brandenburg.geojson`** (both 25 m simplified):
  - shared length **439 km**; distance from the MV line to the BB line along it: p50 0.0 m, p90 1.2 m, **p99 12.5 m**;
  - **overlap 0.11 km²** (1,441 slivers, largest 18,387 m² at E 301498 / N 5912606), **gap 0.17 km²** (cells within 30 m of
    both lines covered by neither; largest 23,136 m² at E 308616 / N 5915168) — i.e. well under 1 m average width;
  - against the **unsimplified** MV line the BB asset is off by p50 3.8 m / p99 21.6 m and overlaps 1.17 km²: most of the
    mismatch comes from the BB asset's own simplification, and both 25 m outlines agree better with each other than with the
    truth. `regions_for` with the usual margin is fine.
- **Shared border with SH** (`sh/probe/boundary/schleswig-holstein_simplified25m.geojson`): 134 km, p50 0.0 / p90 1.3 /
  p99 15.5 m, overlap 0.055 km², gap 0.048 km². **That SH candidate is not a valid geometry**: shapely reports
  "Self-intersection[91512.316 6058719.524]" (25833, North Frisia); it was compared after `make_valid`. It was simplified with
  plain Douglas–Peucker per ring; redo it topology-preserving before it goes into `assets/`. All six current assets in
  `assets/regions/` are valid.
- Neighbours: Brandenburg (freemapper region), Schleswig-Holstein, Lower Saxony (Elbe, short), Poland.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-latest.osm.pbf` → **307** →
  `mecklenburg-vorpommern-261107.osm.pbf` → 200, **128,786,435 B**, `Last-Modified: Sun, 08 Nov 2026 01:57:31 GMT`,
  `ETag "7ad2003-65fba2b0f0000"`, `Accept-Ranges: bytes`.
- State: `https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7362973
  timestamp=2026-11-07T21\:21\:36Z
  sequenceNumber=4955
  ```
- Poly: `https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern.poly`, one ring, 137 points, lon 10.588–14.423,
  lat 53.106–54.984; a buffer of median 0.5 km around MV (up to 35 km offshore). It contains the school, Arkona and Hiddensee,
  but **not** Brüssow, Prenzlau or Świnoujście. A BB–MV map near the border needs both extracts.
- The school building is a multipolygon **relation** (2203698), so the OSM reader must handle building relations (it does for
  LGB areas, inferred).
- Overpass (`overpass-api.de`) answered once (40 s) and then gave **502/504 "at capacity"** for 20 minutes; the OSM API
  `api.openstreetmap.org/api/0.6/map?bbox=…` (3.3 MB, 4.8 s) worked as a fallback.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `www.geodaten-mv.de` WMS (MapServer) | **Watermark "© GeoBasis-DE/M-V" in every GetMap** (x 9–95, y 9–17 px). Max **4192** px; oversize → **200 `text/xml`** ServiceExceptionReport; unknown layer → 200 XML. **Slow: 4000² JPEG 15–22 s.** No Content-Length (chunked). Spring layer ignores `FORMAT=image/jpeg` (returns PNG). GetFeatureInfo `application/json` → 200 `text/xml`. DOP is the 256-colour palette |
| `www.geodaten-mv.de` WCS (MapServer) | Plain `image/tiff` (no multipart); `GEOTIFF:COMPRESSION=LZW/Deflate/JPEG` honoured; no size limit seen (8000² = 256 MB); outside → **400** XML; partly outside → clipped; **no-data areas inside the envelope → 0.0 (DGM/DOM/nDOM) or white (DOP)**; no NoData tag; no gzip; ≈ 1.1–1.5 MB/s |
| `www.geodaten-mv.de` `*_download` (ATOM files) | **Range ignored** (200 + full body), no Content-Length/ETag/Last-Modified, chunked; 0.5–1.2 MB/s; **chunked transfers were cut off twice** ("transfer closed with outstanding read data remaining": the DOM1 dataset feed at 7.4 of 12.7 MB and a DOM1 XYZ zip at 6.9 of 17.9 MB, both while other downloads ran); both succeeded on retry. Without Content-Length a short body is detectable only by parsing (TIFF IFD at the end helps). Missing file → 404 HTML |
| `www.geodaten-mv.de` ALKIS WFS (XtraServer) | 25833 only; COUNT ≤ 30000; case-insensitive SORTBY; errors **400** XML; gzip on request; 0.3–30 s depending on load |
| `www.geodaten-mv.de/api/…` OGC API | errors **400 HTML**; 25833/CRS84 only; gzip on request; `limit` > 10000 accepted silently; cold requests up to 25 s |
| `laiv.geodaten-mv.de` `overview.php` | ≤ 500 objects (else 200 + `success:false`); a few empty 200 answers under rapid sequential calls; `Interner Fehler` for unknown types |
| `www.laiv-mv.de` static files | fine; URL paths contain `Aktualität` with a space (encode `%C3%A4`, `%20`); one link uses `Äktualität` |

- **TLS:** `www.geodaten-mv.de` and `laiv.geodaten-mv.de` send leaf + "Telekom Security OV RSA CA 26A" + "Telekom Security TLS
  RSA Root 2023" (cross-signed by T-TeleSec GlobalRoot Class 3); `www.laiv-mv.de` the same chain with the self-signed 2023
  root; `www.geoportal-mv.de` leaf + "Telekom Security ServerID OV Class 2 CA" (root T-TeleSec GlobalRoot Class 2).
  `openssl s_client` "Verify return code: 0 (ok)" on all four; all three roots are in webpki-roots 1.0.11 (freemapper's
  version). No TLS workaround needed (unlike SH).
- No authentication, no CAPTCHA, no `Retry-After` or rate-limit headers in ≈ 600 requests; no fair-use rules found on the pages
  read (not exhaustive). CORS `*` on the services and the portal.
- Total transfer for this research ≈ **950 MB** (two 141 MB DOP tile transfers, 420 MB of WCS size probes, other 2 km
  tiles, feeds, ALKIS); disk under `mv/` ≈ 450 MB plus a 104 MB Python venv.

---

## 13. What makes the Mecklenburg-Vorpommern adapter differ (risks first)

1. **No NoData in the height models.** DGM1/DOM1/nDOM via WCS (and the DGM/DOM tiles) mark nothing: the sea, the Bodden and
   the Warnow are 0.0 m, and the WCS fills areas without tiles (Brandenburg south of the MV tile edge, open sea) with 0.0 too.
   Cross-border maps would get a cliff to 0 m. freemapper must mask by the MV tile coverage (2 km tiles from the ATOM feed or
   `overview.php`, cacheable) and take LGB's DGM south of it. Inland lakes are flat at their water level (fine).
2. **WMS watermark.** Every DOP GetMap carries "© GeoBasis-DE/M-V" at the top-left; pieces stitched by the WMS code would show
   it every 800 m. Use the **DOP WCS** (`mv_dop20`, JPEG, no watermark, about 3× faster than the WMS) or overlap-and-crop pieces.
   This is a new source type for the orthophoto (WCS with JPEG instead of WMS).
3. **256-colour DOP.** WMS, WCS and the RGB tiles are a dithered palette (PSNR 33.8 dB against RGBI). Only the 419 MB RGBI tiles
   (no ranges) have full colour. Acceptable for a template; mention it in the notes.
4. **WCS details differ from LVermGeo ST.** Coverage ids `mv_dgm`/`mv_dom1`/`mv_ndom`, plain TIFF answers (no multipart),
   no −9999 NoData, compression by `GEOTIFF:COMPRESSION=LZW` (decoder must handle LZW; Deflate with floating-point predictor
   also possible), the answer is clipped to the coverage envelope. The ST code needs a small generalisation, not a copy.
5. **nDSM choice.** The ready nDOM is 1–4 years older than DOM1/DGM1 (school: 2022 vs 2024) and differs by > 1 m on 18 % of
   cells. DOM1 − DGM1 from two WCS requests is current and dated like the DOP in most tiles. DOM1 is probably image-matched
   (smooth edges, no ground under canopy; inferred).
6. **Seasonal and yearly patchwork.** DOP tiles are from 2021, 2023, 2024 and 2025, flown March to October; only ≈ 23 % are
   March/April. No TIME dimension; the separate spring/summer services help only where a leaf-off spring flight exists.
7. **Download hosts are slow and fragile**: 0.5–1.5 MB/s, no ranges, no Content-Length, occasional truncated chunked bodies,
   occasional empty `overview.php` answers. Retry with validation.
8. **ALKIS is easy**: the OGC API is LGB's product (same URL scheme, GeoJSON, paging) — only the endpoint changes. LGB spellings,
   `rellage`, `Bauteil`; oid prefix `DEMV`. `Weg` has no `bez`, `Begleitfläche` is rare.
9. **CRS 25833** like Brandenburg (no `Crs::Utm32` override). New: CC BY 4.0 with the "©GeoBasis-DE/MV/CC BY 4.0 (Quelle
   verändert)" line instead of dl-de/by-2-0.
10. Unchanged: no login, DHHN2016, DGM1 pixel registration (PixelIsArea at the NW tile corner, centres at +0.5 m), Float32,
    Geofabrik layout. Different: 2 km tiles instead of 1 km, no year in file names.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Mecklenburg-Vorpommern | Notes |
|---|---|---|
| **Orthophoto template** | **WCS** `https://www.geodaten-mv.de/dienste/inspire_oi_dop_wcs`, `COVERAGEID=mv_dop20`, `FORMAT=image/jpeg`, `SUBSET=x(…)&SUBSET=y(…)` on the 0.2 m grid, pieces ≤ 4000 px | No watermark, exact grid, 5.7 s per 4000² px. Fallback: WMS `WmsService{url: "https://www.geodaten-mv.de/dienste/adv_dop", layer: "mv_dop", crs: Utm33, extra: "", max_px: 4192}` with a ≥ 20 px top/left overlap cropped away. Currency: `overview.php type[]=dop20` (`aktualitaet` = flight date) or GetFeatureInfo `mv_dop_info` (normalise the two date formats). |
| **DTM** (hillshade, slope, contours) | **WCS** `https://www.geodaten-mv.de/dienste/dgm_wcs`, `COVERAGEID=mv_dgm`, `FORMAT=image/tiff&GEOTIFF:COMPRESSION=LZW`, 25833 | Float32, no NoData: mask cells outside the MV 2 km tile set (sea and Bodden at 0.0 are real). Fallback: ATOM `dgm_download?index=4&…&file=dgm1_33_EEE_NNNN_2_gtiff.tif` (16 MB per 4 km², uncompressed). Currency `overview.php type[]=dgm1`. |
| **nDSM template** | **DOM1 − DGM1**: WCS `https://www.geodaten-mv.de/dienste/dom_wcs`, `COVERAGEID=mv_dom1` (same parameters) minus the DGM above; clamp < 0 | Mask as for the DTM. Report `dom.aktualitaet` and warn if it differs from the DOP month. Ready nDOM (`ndom_wcs`, `mv_ndom`) only as a one-request fallback (older). |
| **Buildings** | **OGC API** `https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1/collections/gebaeude_bauwerk/items`, freemapper's LGB URL builder unchanged (25833, `limit=10000`, `next` links) | Send `Accept-Encoding: gzip` if the client can. Fallback: WFS `alkis_wfs_einfach`, `ave:GebaeudeBauwerk`, `SORTBY=oid&COUNT=10000&STARTINDEX=…` (GML reader). `Region::of_object_id`: `"DEMV"`. |
| **Land use** (out-of-bounds, paved) | Same OGC API, **`nutzung_flurstueck`** | LGB spellings; school parcels "Bildung und Wissenschaft"; `Weg` without `bez`. |
| **Region boundary** | ALKIS WFS `VerwaltungsEinheit` `art=Bundesland` (DE13), 25833, 25 m topology-preserving | Ready: `mv/mecklenburg-vorpommern.geojson` (86 polygons, 1 hole, 9,231 vertices, 157.7 KB, valid); matches the BB asset to p99 12.5 m. Region CRS Utm33. |
| **OSM** | Geofabrik `europe/germany/mecklenburg-vorpommern-latest.osm.pbf` + `mecklenburg-vorpommern-updates/state.txt` | 128.8 MB; 307 to the dated file. Border maps also need `brandenburg-latest`. |
| **Attribution** | `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)` + licence link + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- Whether freemapper's Rust code (ureq/rustls, `tiff` crate, the GML and GeoJSON readers) handles these answers; nothing was
  run in Rust. In particular: LZW Float32 decoding of the WCS answers, the DOP WCS JPEG path, and gzip in ureq.
- WCS behaviour beyond 8000² px and its timeout; WCS DOP piece limits beyond 4000² px.
- That DOM1 is image-matched (inferred from dates and `erf_method 5040`; no documentation read says so).
- The DGM1 of LGB vs MV on overlapping border tiles; MV data on the Polish side (the 2 km tiles reach about 1 km into Poland per
  `crossborder-reference.md`; not re-checked).
- bDOM20/bDOM10light LAZ (format, size, download URL), the `tdom` products, DGM5/DGM25.
- Whether the OGC API keeps a stable order across pages during data updates; behaviour above 10,000 matches per bbox in the
  OGC API (the WFS was paged to 12,000 of 31,867).
- ALKIS outside Rostock/Schwerin (rural `bez` coverage, other cadastral offices `DEMVAL0x`).
- Why ≈ 10 % of rapid `overview.php` calls returned an empty body, and whether the truncated chunked downloads are load-related.
- Rate limits or fair-use rules (none seen, none found).
- The meaning of `meta_aktualitaet` ≠ `aktualitaet` in 557 DOP tiles.
