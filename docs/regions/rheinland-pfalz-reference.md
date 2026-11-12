# Rhineland-Palatinate (LVermGeo RP) data sources for freemapper — live-verified reference

Verified live on 2026-11-08 (Europe/Berlin), early morning (05:51–06:35 UTC). Provider: **Landesamt für Vermessung und
Geobasisinformation Rheinland-Pfalz (LVermGeo)**, Von-Kuhl-Straße 49, 56070 Koblenz (contact
`vertrieb-geodienste@vermkv.rlp.de`, `lvermgeo@vermkv.rlp.de`), for the "Vermessungs- und Katasterverwaltung Rheinland-Pfalz".
Open data under **Datenlizenz Deutschland – Namensnennung 2.0 (dl-de/by-2-0)**, attribution required. The data come from four
kinds of host:

- the file server **`https://geobasis-rlp.de/data/<product>/current/…`** (Apache with an "abba" directory listing in HTML,
  one file per tile, per-tile ISO 19139 XML metadata, Metalink `.meta4` files per municipality);
- the MapServer hosts **`geo4.service24.rlp.de`** (WMS, e.g. `wms/rp_dop20.fcgi`) and **`geo5.service24.rlp.de`** (WFS:
  `wfs/alkis_rp.fcgi`, `wfs/verwaltungsgrenzen_rp.fcgi`, `wfs/blattschnitt_rp.fcgi`, …);
- the **GeoPortal.rlp registry** `https://www.geoportal.rlp.de/registry/wfs/<id>` (a Mapbender proxy in front of the geo5
  WFS; this is the URL the open-data page publishes) and its OGC API façade `https://www.geoportal.rlp.de/spatial-objects/<id>`;
- the web site `lvermgeo.rlp.de` (licence text, product sheets).

No login, registration, token, cookie or form is needed for anything below (the registry sets a `PHPSESSID` cookie, which
can be ignored). The only 401 seen was a guessed WCS path (§5). All TLS chains are complete (§12).

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile / 2 km DOP block |
|---|---|---|---|---|---|---|
| **Görres-Gymnasium**, Gymnasialstraße 3, Koblenz-Altstadt (primary) | way **89826701** | E 400360 / N 5579556 | E −26154 / N 5604852 | 50.35964 / 7.59917 | E 400283–400415, N 5579490–5579632 | **`32 400 5579`** (fully inside) / block **`400_5578`** (E 400–402, N 5578–5580) |
| Frauenlob-Gymnasium, Adam-Karrillon-Straße 35, Mainz-Neustadt (second school, location only) | way **60400704** | E 447388 / N 5539898 | E 17661 / N 5561403 | 50.00908 / 8.26574 | E 447336–447443, N 5539850–5539943 | `32 447 5539` / block `446_5538` (DOP 09.07.2024) |
| **Hesse border probe**, Mainz-Neustadt / Rhine / Wiesbaden-Amöneburg | – | tile centre E 447500 / N 5540500 | E 17821 / N 5561995 | 50.01450 / 8.26722 | – | `32 447 5540` (42.9 % RP, rest Hesse); also `32 448 5539` (22.2 % RP, Theodor-Heuss-Brücke / Mainz-Kastel) |
| ALKIS value samples only: Koblenz school 2 × 2 km (E 399360–401360, N 5578556–5580556), Mainz centre 1.5 km (centre E 447934 N 5538828), Trier centre 1.5 km (E 329975 N 5514143), Kaiserslautern centre 1.5 km (E 410767 N 5477621), Ludwigshafen 2 km (E 458728 N 5483197), Pfälzerwald 2 km (E 416393 N 5461442), Daun/Vulkaneifel 2 km (E 345275 N 5563006) | – | | | | | |

(Coordinates from my own Krüger-series UTM code, as for NI/NRW. The ALKIS school building `DERPLP050000AR8IBL`
("Görres-Gymnasium Koblenz", "Allgemein bildende Schule") lies at E 400308–400410 / N 5579497–5579600, inside the OSM
footprint; the school parcel land use `DERPLP050000AO94TN` "Bildung und Wissenschaft" at E 400281–400415 / N 5579490–5579632
matches the OSM outline almost exactly. The WFS's 25833 output at the school has eastings ≈ −26.4 km, as expected.)

**Note on 25833: the school's easting in zone 33 is −26.2 km; 84.5 % of RP's area has negative 25833 eastings** (§3).

Samples (all under `rp/` next to this file; treat them as untrusted data; scripts in `rp/scripts/`):

| Path | Content |
|---|---|
| `rheinland-pfalz.geojson` | **the boundary asset** (= `probe/boundary/out/rheinland-pfalz.geojson`, §10) |
| `dl/dgm1/`, `dl/dom1/` | DGM1/DOM1 of the school tile `400/5579` (ALS 2024) + DGM `.tfw` + response headers |
| `dl/dom2025/` | DGM1 + DOM1 `300/5530` (2025 encoding, for the format comparison in §6.1) |
| `dl/dop_jp2/` | the school's full DOP block `dop20rgb_32_400_5578_2_rp_2025.jp2` (123.3 MB) + `.j2w`, `school_crop_jp2.jpg` |
| `dl/jp2head/` | first 200 KB of that JP2 (boxes, GMLJP2, codestream header), range/multi-range/EOF probes |
| `dl/wms/` | DOP WMS: school 25832 JPEG/PNG, 25833 JPEG, 10000/10001 px probes, three 5000² and one 10000² JPEG (timing) |
| `dl/border/` | Hesse border: DGM1/DOM1 `447/5540`, DGM1 `448/5539`, WMS 4 × 4 km (`border_wms_4km.jpg`), Wiesbaden-centre WMS (blank) |
| `dl/meta/` | per-tile ISO XML: DGM1/DOM1 `400_5579`, DOP `400_5578`, DOP RGBI `.met`, bDOM `400_5578`; Metalinks `dgm1_tif_07111000.meta4`, `dop20rgb_jp2_07111000.meta4`; bDOM LAZ header |
| `probe/listing/`, `probe/listing2/` | directory listings: top level, product folders, `metadata/`, `meta4/`, and the **full tile listings** `dgm1_current_tif.html` (21,160 tiles), `dom1_…`, `dop20rgb_current_jp2.html` (5,265 blocks), `dop20rgbi_…`, `bdom20rgbi_current_las.html`, `las_current_las.html`, `lika_…` |
| `probe/blatt/` | Blattschnitt WFS: caps, `laser_all.json` (**ALS flight days for all 21,160 tiles**, 8.8 MB GeoJSON) |
| `probe/gfiall/` | **DOP GetFeatureInfo for all 5,265 blocks** (`gfi_all.txt`, one line per block) |
| `probe/gfi/` | DOP GetFeatureInfo at the school (plain, HTML, GML) |
| `probe/caps/` | capabilities: DOP WMS, ALKIS WFS (registry 519 and origin geo5), 314/315/316/451/446/543, Verwaltungsgrenzen (geo5) |
| `probe/wfs/`, `probe/alkis/` | WFS answers: school (registry vs origin, buildings, Nutzung, 25833, hits, paging, COUNT limits, error probes, GeoJSON), `areas/` (seven areas × two types, `sum_*.txt`, `dump_*.jsonl`), Flurstuecksabschnitte (543), TN-Einzellayer (451) |
| `probe/ogcapi/` | OGC API landing pages (314, 315, 451, 519, 543) with `accessUrl`, items at the school |
| `probe/border/` | ALKIS buildings/land use in the Hesse border tile `447/5540` |
| `probe/boundary/` | `r1/lg_25833.gml`, `r1/lg_25832.gml` (Landesgrenze), `.wkt` versions, `out/rheinland-pfalz.geojson` |
| `probe/tls/` | `openssl s_client -showcerts` per host |
| `probe/licence/`, `probe/docs/` | open-data page (+ text), product sheets `WFS_ALKIS_Vereinfacht.pdf`, `DOP.pdf`, `Aktualitaet_Luftbildbefliegung.pdf` (+ `akt_lubi-1.png`) |
| `probe/geofabrik/`, `probe/osm/` | Geofabrik state/poly/page; Overpass answers (school list, school ways) |
| `rustcheck/` | a 40-line Rust program (`tiff` 0.11.3 + `ureq` 3.4.2, freemapper's pinned versions) that decoded the DGM/DOM tiles and connected to every host |

---

## 1. Brandenburg → Rhineland-Palatinate at a glance

| freemapper product | Brandenburg (today) | Rhineland-Palatinate equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** RP spans 6.11–8.51°E; in 25833 it reaches E −135.7 km (**84.5 % of the area at negative eastings**) with −5.7° grid convergence and +3.0 m/km scale in Koblenz (−6.8° / +4.6 m/km at the western edge). WMS and WFS reproject to 25833 on request. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **WMS** `rp_dop20` (MapServer, **max 10000 px**, 5000² px JPEG in 4–10 s, 10000² in 22 s; pixels identical to the tiles up to JPEG noise). Tiles exist only as **2 km lossless JPEG 2000** (`dop20rgb_32_EEE_NNNN_2_rp_YYYY.jp2`, 105–155 MB, 1024 px J2K tiles, no TLM) — impractical. Vintages 2022–2025, **2-year cycle, all summer flights (May–Sept, leaf-on)**; school **25.08.2025**. Date per block via WMS GetFeatureInfo `erstellung`. | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1 GeoTIFF + TFW**, 1 km, Float32, **LZW + Predictor 2 (horizontal) on float words**, 2-row strips, NoData −9999, written by LAStools, 1.2–1.8 MB. **The ALS year is in the file name** (`dgm1_32_400_5579_1_rp_2024.tif`); 4-year ALS cycle 2022–2025, leaf-off (Feb–Apr). No open WCS (a `wcs/rp_dgm1.fcgi` answers **401**). | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **DOM1 − DGM1** (same ALS flight, same grid, both GeoTIFF; DOM1 2.0–3.5 MB). Within ±3 years of the DOP everywhere (ALS spring, DOP summer). Alternative bDOM20 only as **2 km LAZ** (≈270 MB, 100 M points, same date as the DOP). No nDOM. | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi` (= registry `…/registry/wfs/519`), `ave:GebaeudeBauwerk`. **MapServer, not the AAA-suite:** `gml:Polygon` directly under `ave:geometrie`, **`aktualit` wrapped in `<gml:timePosition>`**, empty elements for empty attributes, an extra `gid`, per-feature `gml:boundedBy`. LGB spellings; **`funktion` "Tiefgarage"** (Gebäude *and* Bauteil, always with `rellage`). COUNT up to ≥ 200000 accepted, default 100000. | verified |
| Land use | OGC API `nutzung_flurstueck` | **`ave:Nutzung` only** (no `NutzungFlurstueck`, no `VerwaltungsEinheit`); unsplit objects, **has `nutzart` and `bez`**. LGB spellings except **"Unland, Vegetationslose Fläche"**; `bez` is finer than elsewhere (Wohnbaufläche "Offen"/"Geschlossen", Industrie "Handel", "Bank, Kredit" …), a few `;`-joined/terminated values. City blocks "Fläche gemischter Nutzung / Gebäude- und Freifläche Mischnutzung mit Wohnen". `Platz` without `bez` 27 %; verge `bez` on 13 % of `Straßenverkehr`. | verified |
| Currency | `bb_*_aktualitaet.csv` | DOP: GetFeatureInfo `erstellung` per 2 km block (= name year for all 5,265); DGM/DOM: **Blattschnitt WFS layer `Laserbefliegung`** (ALS flight days per 1 km tile, 8.8 MB GeoJSON for the whole state) + name year; per-tile ISO XML; ETag/Last-Modified; `aktualit` per ALKIS feature | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `rheinland-pfalz-latest.osm.pbf` = **274.8 MB**; poly excludes Wiesbaden/Mainz-Kastel, Saarland, Mannheim | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **dl-de/by-2-0** (same licence as Brandenburg). Required text (open-data page, WMS/WFS `Fees`, tile metadata): **"©GeoBasis-DE / LVermGeoRP<Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"** | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **File server** | `https://geobasis-rlp.de/data/` | folders `dgm1`, `dom1`, `dop20rgb`, `dop20rgbi`, `bdom20rgbi`, `las`, `hu` (Hausumringe zip 319 MB), `lb` (Landbedeckung zip 1.9 GB), `lika` (cadastral map raster), `dtk*`, `basisdlm`, `hk`, `geb3dlo`, `inspire-annex*`, `ln` (**502 Proxy Error** at the time) … each with `current/{tif,jp2,las,zip}/`, `current/metadata/`, `current/meta4/` | Apache, HTML listing (theme "abba"), sortable (`?C=N;O=D`), **no JSON/XML listing**. HEAD with Content-Length, ETag, Last-Modified, **`Accept-Ranges: bytes`**; Range → 206; multi-range → 206 `multipart/byteranges`; beyond EOF → 416; missing file → **404** HTML. CORS `*`. Large listings are slow: `dgm1/current/tif/` = 9.1 MB HTML in **52 s**. |
| **DOP WMS** | `https://geo4.service24.rlp.de/wms/rp_dop20.fcgi` | `rp_dop20` (RGB), `rp_dop20_info` (queryable metadata per block); root `wms_rp_dop20` | "WMS RP DOP20", MapServer, AdV-WMS-Profil; **MaxWidth/MaxHeight 10000**; squid in front (`X-Cache: MISS from geo4ap3.vermkv.rlp`) |
| **ALKIS WFS (origin)** | `https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi` | `ave:Flurstueck`, `ave:GebaeudeBauwerk`, `ave:Nutzung` | "WFS RP ALKIS Vereinfacht", MapServer WFS 2.0/1.1/1.0; the origin of registry 519 (`accessUrl` in the OGC API landing page); its capabilities reference itself |
| ALKIS WFS (registry) | `https://www.geoportal.rlp.de/registry/wfs/519` | same | Mapbender proxy (PHP); **byte-identical answers** to the origin (diff of the school answer: 0 lines apart from `timeStamp`/`schemaLocation`), but `Content-Type: application/xml`, **15 duplicate `Set-Cookie: PHPSESSID` headers**, ≈0.2 s slower |
| Other geo5 WFS | `…/wfs/verwaltungsgrenzen_rp.fcgi` (registry 314), `…/wfs/blattschnitt_rp.fcgi` (315), `…/wfs/tnel_rp.fcgi` (451, ALKIS TN "Einzellayer" with full AAA attributes), `…/wfs/fav_rp.fcgi` (543, Flurstücksabschnitte) | state boundary, tile/ALS index | **The registry proxy for 314 is broken**: every GetFeature → **HTTP 200 `text/html` "No wfs_conf data for featuretype vermkv:landesgrenze_rlp"** (56 B); the origin works. |
| OGC API façade | `https://www.geoportal.rlp.de/spatial-objects/<id>` | GeoJSON items in WGS84 | Mapbender; lon/lat only (`bbox-crs` 25832 → 400 "Parameter bbox not inside global WGS84 bbox"), coordinates rounded to 6 decimals, `aktualit` renamed `timePosition`, `gmdschl` as integer (`7111000`, leading zero lost), 11 s for 112 buildings, a `next` link even when everything was returned. **Not suitable.** |
| Portals / docs | `https://lvermgeo.rlp.de/geodaten-geoshop/open-data/`, `https://geoshop.rlp.de` (GeoShop), `https://www.geoportal.rlp.de` | licence, product sheets, map viewers | the open-data page links the registry WFS, the OGC API, GeoShop and the INSPIRE download feeds |
| ISO metadata | per tile under `…/current/metadata/*_meta.xml`; CSW `https://metadatenkatalog-vermkv.service24.rlp.de/extern/vertriebsprodukte?…` (link in GetFeatureInfo) | | CSW not examined |

Not used / not examined: GeoShop, INSPIRE download feeds, `hu` / `lb` bulk zips, `lika` raster, LoD (`geb3dlo`), Basis-DLM,
the WMS-T of historic orthophotos.

### 2.1 Tile scheme

- **EPSG:25832, SW corner at whole km** ("Kachelungssystematik: … glatten km-Werten an der jeweils linken unteren Ecke").
  **The year is part of the file name for DGM1/DOM1/DOP**, so names cannot be computed from the tile key alone:

  | Product | File name | Tile | Files | Size |
  |---|---|---|---|---|
  | DGM1 | `dgm1/current/tif/dgm1_32_EEE_NNNN_1_rp_YYYY.tif` + `.tfw` | 1 km, 1000 px | 21,160 tiles (E 292…465, N 5422…5645) | 1.2–1.8 MB |
  | DOM1 | `dom1/current/tif/dom1_32_EEE_NNNN_1_rp_YYYY.tif` (+ `.tfw` only for the 5,456 tiles of 2025) | 1 km | 21,160 | 2.0–3.5 MB |
  | DOP20 RGB | `dop20rgb/current/jp2/dop20rgb_32_EEE_NNNN_2_rp_YYYY.jp2` + `.j2w` (EEE, NNNN even) | 2 km, 10000 px | 5,265 blocks (E 292…464, N 5424…5644) | 105–155 MB |
  | DOP20 RGBI | `dop20rgbi/current/jp2/dop20rgbi_32_EEE_NNNN_2_rp_YYYY.jp2` + `.j2w` + `.met` (ISO XML) | 2 km | 5,265 | 155–220 MB |
  | bDOM20 | `bdom20rgbi/current/las/bdom20rgbi_32_EEE_NNNN_2_rp.laz` (no year) | 2 km | 5,266 | 240–360 MB |
  | ALS points | `las/current/las/lpolpg_32_EEE_NNNN_1_rp.laz` (no year) | 1 km | 21,207 | 80–350 MB |
  | Per-tile metadata | `<product>/current/metadata/<stem>_meta.xml` (e.g. `dgm1_32_400_5579_1_rp_2024_meta.xml`, 86 KB; DOP 72 KB) | | | ISO 19139 |
  | Metalinks | `<product>/current/meta4/<product>_{tif,jp2,md}_<AGS>.meta4` per Land (`07`, 6.1 MB), Kreis (`07111`), Verbandsgemeinde, Gemeinde (`07111000`) | | 4,999 files per product | file name, size, **SHA-256**, URL |

- Name year counts (from the listings): DGM1/DOM1 2022: 5,444, 2023: 4,804, 2024: 5,456, 2025: 5,456 tiles; DOP 2022: 88,
  2023: 1,655, 2024: 1,020, 2025: 2,502 blocks.
- **Tiles are complete squares across the state border:** the Hesse-border DGM1/DOM1 `447/5540` (57 % Hesse) and DGM1
  `448/5539` (78 % Hesse) have **0 % NoData** and show Wiesbaden-side terrain. (Border tiles towards France/Luxembourg/
  Belgium not checked.)

### 2.2 Finding tile names and dates

Four ways, from best to worst for an adapter:

1. **Blattschnitt WFS, layer `Laserbefliegung`** (for DGM1/DOM1): one feature per 1 km tile with `kachel` (`4005579` =
   EEE+NNNN), `flugjahr`, `los`, `jahr_los`, `flugtage`:
   ```
   https://geo5.service24.rlp.de/wfs/blattschnitt_rp.fcgi?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=Laserbefliegung&BBOX=399000,5578000,402000,5581000,urn:ogc:def:crs:EPSG::25832
   → 200, 25 features, 31.7 KB, 0.22 s
   <ms:kachel>4005579</ms:kachel><ms:flugjahr>2024</ms:flugjahr><ms:los>3</ms:los><ms:jahr_los>2024_3</ms:jahr_los>
   <ms:flugtage>30.03.24, 06.04.24</ms:flugtage>  <ms:betrifft>DGM, DOM, Laserpunkte</ms:betrifft>
   ```
   Whole state: `…&OUTPUTFORMAT=application/json;%20subtype=geojson` → 21,160 features, 8.8 MB, 8.4 s. **`flugjahr` equals
   the DGM1 and DOM1 file-name year for 21,147 of 21,160 tiles**; 13 tiles (E 333–337, N 5537–5539) have `flugjahr` null
   (file year 2025). So: tile key → `flugjahr` → URL.
2. **Try the years:** the name year is one of 2022–2025; a wrong year is a plain **404** (`dgm1_…_2025.tif` for a 2024 tile:
   404, 1,595 B HTML). At most four GETs per tile, but each 404 costs up to a few seconds on this server (§12).
3. **Metalink per municipality** (`…/meta4/dgm1_tif_<AGS>.meta4`, e.g. Koblenz `07111000`: 284 files, 81 KB): exact URLs,
   sizes and SHA-256. Needs the municipality key of the map area.
4. The HTML listing (9 MB, 52 s for DGM1) — only for a one-off index build.

For the **DOP** the block name year is in the GetFeatureInfo answer (§4.1) and equals the file-name year for all 5,265 blocks.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DGM1/DOM1 GeoTIFF: `ProjectedCSType 25832`, ModelTiepoint at the **NW corner** (`400000, 5580000`), ModelPixelScale 1,
    `GTRasterType 1` (PixelIsArea); TFW gives the pixel centre (`400000.5 / 5579999.5`). Odd GeoKeys: the DGM1 and the 2025
    DOM1 have `GTModelType 32767` (user-defined) plus `VerticalCSType 7837`; the 2022–2024 DOM1 have no `GTModelType` at all.
    freemapper reads only ModelTiepoint/ModelPixelScale (or the TFW), so this does not matter.
  - DOP JP2: GMLJP2 `RectifiedGrid` origin `400000.1 5579999.9` (pixel centre), offset vectors ±0.2, `urn:ogc:def:crs:EPSG::25832`;
    `.j2w` `0.2 / 0 / 0 / -0.2 / 400000.1 / 5579999.9`.
  - WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`; WMS layers list 25831, **25832**, **25833**, 4258, 4326, 5649–5653, 4647,
    9935832, 3043 (info layer).
- **Height datum DHHN2016** normal heights, quasigeoid GCG2016: DGM/DOM GeoAscii "ETRS89 / UTM zone 32N + DHHN2016 height",
  tile XML `VerticalCRS 7837 DE_DHHN2016_NH`, remark `DE_AdV_GCG2016_QGH`. Same as the other freemapper states.
- **Zone choice:** RP lies at 6.11–8.51°E, entirely west of 9°E:

  | Place | Convergence zone 32 / 33 | Scale k zone 32 / 33 | 25833 easting |
  |---|---|---|---|
  | Koblenz (school) | −1.08° / −5.71° | 0.99972 / 1.00300 | **−26.2 km** |
  | Mainz | −0.56° / −5.16° | 0.99963 / 1.00245 | +18.1 km |
  | Kaiserslautern | −0.94° / −5.51° | 0.99970 / 1.00297 | −24.0 km |
  | Trier | −1.80° / −6.40° | 0.99996 / 1.00405 | −101.9 km |
  | West edge (6.11°E, Eifel at Luxembourg) | −2.21° / −6.84° | 1.00012 / 1.00457 | −135.7 km |
  | East edge (8.51°E, Worms area) | −0.37° / −4.94° | 0.99962 / 1.00232 | +29.5 km |

  **RP maps must be 25832 projects** (`Region::crs() → Crs::Utm32`). **84.5 % of the state area has negative 25833
  eastings** (the asset reaches E −135,707), so no 25833 `TileId` (`u32`) may ever be built for RP.
- **Extent** of the state polygon (Landesgrenze incl. the German–Luxembourg condominium, §10):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 293316 – 464351, N 5423949 – 5644103 |
  | EPSG:25833 | E −135713 – 30574, N 5445770 – 5668185 |
  | lon / lat | lon 6.112 – 8.508, lat 48.966 – 50.942 |

  Area 19,848.9 km² in 25832 (official 19,858 km²), 19,986.2 km² in 25833.

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- Product: DOP20 RGB / RGBI, 20 cm, **true orthophoto since 2022** (tile XML `Bezugsfläche bDOM`; product sheet: "seit 2022
  aus dem bildbasierten Digitalen Oberflächenmodell (bDOM) abgeleitet"). Update cycle (product sheet `DOP.pdf`, Stand
  02/2026): "**zweijährig (jährlich für ca. 50% der Landesfläche)**". Map `Aktualitaet_Luftbildbefliegung.pdf` "Stand
  01/2026": north-west 2023, centre/east 2025, south-east 2024, a corner 2022.
- Only the current vintage is published (`…/current/…`); no historic DOP folders.
- **Flight date per block** — from the WMS info layer (all 5,265 blocks queried, `probe/gfiall/gfi_all.txt`, ≈10 min with
  three parallel requests, no error):

  | `erstellung` month | Blocks |
  |---|---|
  | 2022-07 | 88 |
  | 2023-05 / 06 / 09 | 471 / 308 / 876 |
  | 2024-07 / 08 / 09 | 352 / 374 / 294 |
  | 2025-06 / 08 / 09 | 963 / 1,439 / 100 |

  → **100 % leaf-on (May–September)**, 24 distinct dates in runs of consecutive days (e.g. 06.–09.09.2023, 19.–21.06.2025,
  09.–15.08.2025). The field is called `erstellung` (creation) and is also the ISO `CI_Date creation` of the tile; the
  date pattern and the leaf-on imagery make it the **flight date** (inferred — no field is labelled "Bildflugdatum").
  The file-name year equals the `erstellung` year for all 5,265 blocks. School: **25.08.2025** (leaf-on, true ortho,
  `dl/wms/school_25832.jpg`). Mainz school: 09.07.2024.

### 4.2 WMS (recommended path)

- `https://geo4.service24.rlp.de/wms/rp_dop20.fcgi`, WMS 1.3.0 (MapServer), "WMS RP DOP20".
  - Layers: **`rp_dop20`** ("DOP20", not queryable), **`rp_dop20_info`** ("Metadaten der DOP20", queryable). No TIME dimension.
  - Formats: image/jpeg, png, tiff, tiff-lzw, webp, vnd.jpeg-png(8), png;mode=24bit.
  - **MaxWidth/MaxHeight 10000.** 10000 × 100 px → 200 JPEG (0.37 s); 10001 px → **HTTP 200 `text/xml`**
    `<ServiceExceptionReport …><ServiceException>msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH
    and HEIGHT must be between 1 and 10000 pixels.</ServiceException>`.
- Example (school, 25832, 0.2 m):
  ```
  https://geo4.service24.rlp.de/wms/rp_dop20.fcgi?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&STYLES=&LAYERS=rp_dop20&CRS=EPSG:25832&BBOX=400180,5579380,400540,5579730&WIDTH=1800&HEIGHT=1750&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 759,137 B, 0.87 s (PNG 2.21 MB, 2.3 s). 25833 (`BBOX=-26354,5604652,-25954,5605052`, 2000²): 866,894 B,
  1.2 s — negative eastings are accepted.
- Speed: 5000 × 5000 px JPEG (1 km² at 0.2 m): **9.9, 4.0, 5.1 s** (5.4 / 3.8 / 5.1 MB); 10000 × 10000 (4 km²): 19.8 MB in
  **22.3 s**. ≈1 MB/s, about 3× faster than LGLN's WMS.
- **Same pixels as the tiles:** the school window from the decoded JP2 vs the WMS JPEG: mean abs difference **3.45** of 255
  at offset 0, 7.6 at ±1 px; mean luminance 115.03 vs 115.05 (`scripts/jp2cmp.py`).
- **Clipped at the state border:** at the Hesse probe (4 × 4 km, 43.3 % RP) the Hesse side is white (56.6 % white pixels),
  cut exactly along the mid-Rhine border (`dl/border/border_wms_4km.jpg`); Wiesbaden centre → an all-white 1.8 KB JPEG —
  although the DGM/DOM tiles cover the Hesse side.
- Headers: no Content-Length (chunked), `Cache-Control: max-age=86400` and `max-age=0` both present, CORS `*`.
- **GetFeatureInfo gives the date and the block name** (no extra parameters needed):
  ```
  https://geo4.service24.rlp.de/wms/rp_dop20.fcgi?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=rp_dop20_info&QUERY_LAYERS=rp_dop20_info&STYLES=&CRS=EPSG:25832&BBOX=400260,5579456,400460,5579656&WIDTH=100&HEIGHT=100&I=50&J=50&FEATURE_COUNT=10&INFO_FORMAT=text/plain
  → 200 text/plain, 901 B, 0.15 s
  Layer 'metadaten'
    Feature 5098:
      gid = '5098'
      publikation = '2026-01-11 02:15:10.166'
      name = 'dop20rgbi_32400_5578'
      titel = 'Digitale Orthophotos 0,2m Rheinland-Pfalz (DOP20RGB) - 4005578'
      erstellung = '25.08.2025'
      aktualisierung = 'n/a'
      csw-anfrage = 'https://metadatenkatalog-vermkv.service24.rlp.de/extern/vertriebsprodukte?request=GetRecords&…'
  ```
  `application/vnd.ogc.gml` (2.2 KB) gives the same fields plus the block square (`400000,5578000 402000,5580000`);
  `text/html` 3.7 KB. Date format **`DD.MM.YYYY`**; `aktualisierung` is `n/a` for all blocks; `publikation` = publication
  timestamp (2023-03 … 2026-01). One block per point (blocks do not overlap).

### 4.3 Tile downloads (JPEG 2000)

- `https://geobasis-rlp.de/data/dop20rgb/current/jp2/dop20rgb_32_400_5578_2_rp_2025.jp2`: HEAD 200, **123,320,680 B**,
  `Content-Type: image/jp2`, ETag, Last-Modified `Sun, 11 Jan 2026 01:09:57 GMT`, `Accept-Ranges: bytes`; full GET **121 s
  (1.02 MB/s)**; decode (Pillow/OpenJPEG 2.5.4) **20.9 s** for 10000² RGB.
- Structure (`scripts/j2kmain.py`): JP2 boxes `jP`, `ftyp`, `rreq`, `jp2h` (ihdr 10000 × 10000 × 3, 8 bit; `res `), a GeoTIFF
  `uuid` box, a GMLJP2 `asoc` (envelope 400000 5578000 / 402000 5580000), then `jp2c`. Codestream: **SIZ 1024 × 1024 tiles
  (10 × 10), COD LRCP, 1 quality layer, 3 decomposition levels, reversible 5/3 wavelet, COM "Created by OpenJPEG version
  2.4.0. LOSSLESS settings used"**, **no TLM/PLT** → tile offsets can only be found by hopping SOT headers (100 range
  requests), then ≈1.2 MB per 1024² tile. A 400 m window needs up to 9 tiles (≈10 MB) plus a Rust JPEG 2000 decoder
  (none in freemapper). **Not worth it; use the WMS.**
- RGBI JP2 155–220 MB, with an ISO `.met` (87 KB) next to it. ISO XML per block under `…/metadata/` (72 KB): `CI_Date creation
  2025-08-25`, `editionDate 2025`, `Bezugsfläche bDOM`, `Anzahl Spalten 10.000`, accuracy ±0.5 m.

### 4.4 WCS

- No DOP or DGM WCS is published on the open-data page. Guessed paths: `https://geo4.service24.rlp.de/wcs/rp_dgm1.fcgi` →
  **401 Unauthorized** (exists, protected — presumably a paid service), `…/wcs/dgm_rp.fcgi` and `geo5…/wcs/dgm_rp.fcgi` → 403.

---

## 5. DGM1 (terrain model, 1 m)

- Source: ALS, **4-year cycle in four lots per year** (Blattschnitt `Laserbefliegung`: 2022 5,444, 2023 4,804, 2024 5,456,
  2025 5,443 tiles + 13 without date; lots `2022_1` … `2025_4`). Flight days all in **February–April (leaf-off)**; tiles carry
  1–5 flight days (e.g. "07.-09.02.23, 15.-17.03.23, 10.04.23"). School tile: **30.03.24, 06.04.24** (lot `2024_3`); Hesse
  probe: 2023 (lot `2023_1`).
- **Tile download:**
  ```
  HEAD https://geobasis-rlp.de/data/dgm1/current/tif/dgm1_32_400_5579_1_rp_2024.tif
  → 200, Content-Length: 1271522, ETag: "1366e2-65199c653850e", Last-Modified: Tue, 12 May 2026 07:25:59 GMT,
    Accept-Ranges: bytes, Content-Type: image/tiff (0.69 s)
  ```
  GET 1.27 MB in 8.7 s on the first try, 1.0–1.4 MB in 1.5–3.4 s later (server latency varies, §12). All DGM1 files were
  rewritten in May 2026 (mtime 2026-05-12; TFWs 2026-06).
- TIFF (verified on `400/5579` 2024, `447/5540` 2023, `448/5539` 2023, `300/5530` 2025):
  - 1000 × 1000 **Float32**, **Compression 5 (LZW), Predictor 2 (horizontal differencing of the 32-bit words)**,
    **strips of 2 rows** (500 strips), first IFD at 8, no overviews, no tiling;
  - Software "LAStools is a product of rapidlasso GmbH, Germany"; `GDAL_NODATA "-9999"`; values rounded to **0.01 m**;
  - TFW: `1.0 / 0.0 / 0.0 / -1.0 / 400000.5 / 5579999.5` (78 B; pixel centre). The tiepoint is the NW corner.
  - **Rust check:** `tiff` 0.11.3 `Decoder::read_image()` returned `F32` with the same values as libtiff (min 61.17, max 75.56
    at the school; the crate supports Predictor 2 for IEEE float, source `decoder/image.rs` + `rev_hpredict_nsamp`).
    freemapper's `float_tiff` lenient path also handles predictor 2.
- Values: school tile 61.17–75.56 m (Rhine/Mosel level ≈61 m), no NoData. Border tiles full (§2.1). The tile XML gives a
  different vertical extent (57.98–75.96) and `Erfassung 2024-03-15` instead of the flight days 30.03./06.04.2024 —
  use the Blattschnitt flight days, not the XML.
- Item XML (`dgm1_32_400_5579_1_rp_2024_meta.xml`, 88 KB ISO 19139): title "Digitales Geländemodell Gitterweite 1 m
  Rheinland-Pfalz (DGM1) - 4005579", `CI_Date creation 2024-03-15`, `dateStamp 2026-03-11`, lineage "Daten der
  Laserscanbefliegungen", "Triangulation", accuracy text (±10 cm + 5 % / 20 % of the grid width).

---

## 6. nDSM sources: DOM1, bDOM20, LiDAR

### 6.1 DOM1 (1 m, ALS)

- `https://geobasis-rlp.de/data/dom1/current/tif/dom1_32_400_5579_1_rp_2024.tif` (3,002,399 B, 2.8 s). Same ALS flight and
  name year as the DGM1 (Blattschnitt `betrifft: "DGM, DOM, Laserpunkte"`).
- **Two encodings:**

  | Vintage | Strips | Predictor | IFD | TFW | GeoKeys | Precision |
  |---|---|---|---|---|---|---|
  | 2022–2024 (16,704 tiles, written 2025-04) | 20 rows (50 strips) | 1 (none) | **at the end of the file** (offset 3,001,470) | **none** (`.tfw` → 404) | no `GTModelType`, `GeographicType 4258`, no vertical key | 0.001 m |
  | 2025 (5,456 tiles, written 2026-06/07) | 2 rows | 2 | at 8 | yes | as DGM1 | 0.01 m |

  Both decoded in Rust (`tiff` 0.11.3) and Pillow. **freemapper must read the georeference from the GeoTIFF tags, not from a
  `.tfw`**, for DOM1 (the `float_tiff` decoder does).
- **DOM1 − DGM1** over the school footprint (9,423 cells): p10 0.00, **median 9.39**, p90 18.59, max 26.72 m; 63.1 % > 2 m;
  6.5 % < 0. Whole tile: 14.5 % < 0, but only 0.13 % below −0.05 m — rounding (DGM 0.01 m vs DOM 0.001 m) → clamp at 0.
- **Age gap to the DOP** (DOP block year − ALS year per 1 km tile, 21,060 tiles): −2: 3,504, −1: 1,884, 0: 2,300, **+1: 7,152**,
  +2: 4,132, +3: 2,088 (100 edge tiles without a DOP block). **Always within ±3 years**, but ALS is spring (leaf-off) and the
  DOP summer (leaf-on). 26 % of tiles have a DOM *newer* than the DOP. School: ALS 04/2024, DOP 08/2025.

### 6.2 bDOM20 (image-based DSM, same date as the DOP)

- Only as LAZ: `https://geobasis-rlp.de/data/bdom20rgbi/current/las/bdom20rgbi_32_400_5578_2_rp.laz` (272 MB). Header
  (first 375 B by range): LAS **1.2, point format 2 (RGB)**, LASzip 3.4, **100,000,000 points** (= 0.2 m grid over 2 × 2 km),
  scale 0.01, created day 339/2025, z 33.5–210.8 m. ISO XML `bdom20rgbi_32_400_5578_2_rp_meta.xml`: "Bildbasiertes digitales
  Oberflächenmodell 0,2 m", `CI_Date creation 2025-08-25` = the DOP date.
- freemapper reads LAZ with the `las` crate (Brandenburg nDOM), but **from an in-memory copy** — 270 MB per 2 km block is too
  much for a school map. Not recommended.

### 6.3 LiDAR / nDOM

- ALS point clouds `las/current/las/lpolpg_32_EEE_NNNN_1_rp.laz` (1 km, 80–350 MB, 21,207 files). **No nDOM product.**

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **WMS GetFeatureInfo `rp_dop20_info`** | DOP | `erstellung DD.MM.YYYY` (flight date, §4.1), `name dop20rgbi_32EEE_NNNN`, `publikation`; 0.15 s per point |
| **Blattschnitt WFS `Laserbefliegung`** | DGM1, DOM1, ALS points | `flugjahr`, `flugtage` (one or several days/ranges, `DD.MM.YY`), `los`; per 1 km tile; whole state 8.8 MB |
| File name year | DGM1, DOM1, DOP | = `flugjahr` (21,147/21,160) / = `erstellung` year (5,265/5,265) |
| ISO XML per tile (`…/metadata/`) | all | `CI_Date creation` (DOP/bDOM = flight date; DGM = 2024-03-15, not a flight day), `dateStamp`, `supplementalInformation` (timestamp) |
| ETag / Last-Modified | every file | build date (DGM1 2026-05-12, DOM1 2025-04 or 2026-06/07, DOP 2025-01 … 2026-01) |
| Metalink | every file | SHA-256 + size, `published` |
| ALKIS `aktualit` | per feature | `YYYY-MM-DD` inside `<gml:timePosition>`. Buildings: 2010 8,168 (migration), 2011–2023 7,619, 2024 4,622, 2025 2,764, 2026 2,478; land use 2024–2026 mostly ("Monatsaktualität") |

Suggested mapping:
- orthophoto `currency` = GFI `erstellung` at the map centre (one request; check the map corners if the map spans blocks),
  always leaf-on; `published` = GFI `publikation`;
- DGM1/DOM1 `currency` = the last `flugtage` day of the tile (or `flugjahr`), `published` = Last-Modified.

---

## 8. ALKIS: buildings and actual land use

### 8.1 WFS

- **Origin `https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi`** (registry `https://www.geoportal.rlp.de/registry/wfs/519`
  returns the same bytes), "WFS RP ALKIS Vereinfacht", WFS 2.0.0/1.1.0/1.0.0, **MapServer**.
  - Types: `ave:Flurstueck`, `ave:GebaeudeBauwerk`, `ave:Nutzung` — **no `NutzungFlurstueck`, no `VerwaltungsEinheit`,
    no `KatasterBezirk`** (although the abstract lists Verwaltungseinheiten).
  - DefaultCRS 25832; OtherCRS 25831, 25833, 3034, 3035, 3043–3045, 3857, 4258, 4326, 31466–31469, 4647, 5649–5653, 9935832.
  - Output: `application/gml+xml; version=3.2` (default; answered as `text/xml; subtype="gml/3.2.1"`), GML 2.1.2/3.1.1,
    **`application/json; subtype=geojson`**, shapefile zip, CSV.
  - Constraints: **`ImplementsResultPaging TRUE`, `CountDefault 100000`** (product sheet: "bis zu 100.000 Objekte einer
    Objektart"). No gzip (`Vary: Accept-Encoding` but no `Content-Encoding`).
- freemapper's `alkis_wfs` request shape works:
  ```
  https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=400180,5579380,400540,5579730,urn:ogc:def:crs:EPSG::25832&SORTBY=oid&COUNT=10000&STARTINDEX=0
  ```
  → 200, **351 features**, 611,946 B, 2.0 s (1.0 s on repeat). `ave:Nutzung` same box: 77 features, 168.7 KB, 0.6 s.
  `RESULTTYPE=hits` → `numberMatched="351" numberReturned="0"` (0.15 s); 5 × 5 km around the school: hits **24,364**.
- **The GML differs from the AAA-suite servers (HVBG/LGLN/NRW)** — checked against `parse_gml`:
  ```xml
  <wfs:member>
    <ave:GebaeudeBauwerk gml:id="GebaeudeBauwerk.517812">
      <gml:boundedBy><gml:Envelope srsName="urn:ogc:def:crs:EPSG::25832">…</gml:Envelope></gml:boundedBy>
      <ave:geometrie>
        <gml:Polygon gml:id="GebaeudeBauwerk.517812.1" srsName="urn:ogc:def:crs:EPSG::25832">
          <gml:exterior><gml:LinearRing><gml:posList srsDimension="2">… E N pairs …</gml:posList></gml:LinearRing></gml:exterior>
        </gml:Polygon>
      </ave:geometrie>
      <ave:oid>DERPLP050000AR8IBL</ave:oid>
      <ave:aktualit gml:id="GebaeudeBauwerk.517812.aktualit"><gml:timePosition>2026-07-20</gml:timePosition></ave:aktualit>
      <ave:gebnutzbez>Gebäude</ave:gebnutzbez>
      …
      <ave:rellage></ave:rellage>
      <ave:gid>517812</ave:gid>
  ```
  - geometry is a bare **`gml:Polygon`** (all 31,224 sampled features; interiors on 0.6 %; no MultiSurface, no curves) —
    in `GEOMETRY_ELEMENTS`, read fine;
  - **`aktualit` holds a child element `<gml:timePosition>`**: `parse_gml` clears the text buffer at each `Start` and takes
    the text only at the end of a direct child → **`aktualit` would be lost** (`updated = None`). Needs a small parser
    change (keep the text of a nested `timePosition`, or read a direct child's descendant text);
  - per-feature `gml:boundedBy` (ignored by the parser, verified by reading the code path), empty elements
    `<ave:rellage></ave:rellage>` (skipped as empty), an extra attribute **`gid`** (harmless).
- **Paging:** `COUNT=100&SORTBY=oid&STARTINDEX=0/100/200/300` → 100/100/100/51 = 351, disjoint and complete. Full pages say
  `numberMatched="unknown"`, the last page `numberMatched="351"`. **`SORTBY=oid` sorts case-insensitively** (database
  collation: `…AQJoBL` before `…AQJXBL`, `…AQKiBL` before `…AQKIBL`), not in byte order — harmless for freemapper (it
  dedups by oid and stops on a page without new features), but do not assume byte order.
- **Limits:** `COUNT=100001` and `COUNT=200000` → 200 with all features (no cap seen). Without COUNT the server default is
  100000; a box with > 100000 buildings (≈ 10 × 10 km of city) was not tried.
- 25833: `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833` with negative eastings → 200, 433 features (rotated bbox), E N order,
  1.1 s.
- **Errors are proper OGC exceptions with HTTP 400** (`text/xml`): unknown type → `InvalidParameterValue` locator `typename`
  "msWFSGetFeature(): WFS server error. TYPENAME 'Gebaeude' doesn't exist in this server." (5.5 s!); bad SRS → locator
  `srsname` "Invalid SRS"; `BBOX=foo` → locator `bbox` "Wrong number of arguments for BBOX.".
- Outside RP (Mannheim-side box E 455000 N 5545000): `numberMatched="unknown" numberReturned="0"`. **ALKIS stops at the
  state border:** Hesse border tile `447/5540`: 429 buildings, all `gmdschl 07315000` (Mainz), 0 % of any building outside
  the RP polygon; 95 land-use objects, none outside.
- Speed (2 × 2 km / 1.5 × 1.5 km): Koblenz 3,549 buildings 5.9 MB 6.1 s, 1,208 Nutzung 2.4 MB 6.2 s; Kaiserslautern 6,624
  buildings 10.5 MB 10.8 s; Mainz 4,305 / 7.3 MB / 7.5 s. ≈1 MB/s. One border-tile request took 12.9 s for 0.74 MB.

### 8.2 OGC API Features (`www.geoportal.rlp.de/spatial-objects/519`)

- Mapbender façade over the same WFS: `…/collections/ave:GebaeudeBauwerk/items?f=json&bbox=<lon/lat>&limit=500` → 200
  `application/vnd.geo+json`, 112 features, 11.3 s. **WGS84 only** (25832 `bbox-crs` → 400), coordinates rounded to 1e-6°
  (≈0.1 m), property names changed (`timePosition`, `gml_id`), `gmdschl` integer. The WFS is better in every respect.

### 8.3 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid` (18 chars, suffix `BL`), `aktualit` (in `timePosition`), `gebnutzbez`, `funktion`, `gmdschl`
  (always), `gid` (always), `gfkzshh` (93 %), `lagebeztxt` (45 %), **`anzahlgs`** (number of storeys, 8 %), `name` (1.4 %),
  `rellage` (0.8 %). LGB spellings ("Gebäude", "Allgemein bildende Schule").
- `Nutzung`: `oid` (18 chars, suffix **`TN`**, an object id, not oid+parcel), `aktualit`, `nutzart` (always), `bez` (82 %),
  `name` (0.5 %), `gid`. **No `flstkennz`** (not split by parcel) — the same type freemapper uses for Saxony-Anhalt
  (`LVERMGEO.land_use_type = "ave:Nutzung"`).
- **`ave:Nutzung` carries both `nutzart` and `bez`** (5,573 of 5,573 have `nutzart`) — unlike Hesse, where it has no `nutzart`.
- oids start with **`DERP`** (`DERPLP05…` Koblenz, `DERPLP20…`, `DERPLP11/12/14/15/16…`) → `Region::of_object_id` gets a
  `"DERP"` arm.
- Other ALKIS services, not needed: 451 "TN Einzellayer" (`tnel_rp.fcgi`, 25 `ave:AX_*` types with raw AAA attributes such as
  `funktion 2100`, `funktion_beschreibung`, processing steps), 543 "Flurstücksabschnitte" (`fav_rp.fcgi`, text fields like
  `tng "279 m² Gebäude- und Freifläche Mischnutzung mit Wohnen|36 m² Religiöse Einrichtung"`).

### 8.4 Sample features at the Görres-Gymnasium (WFS, geometry elided)

```json
{"oid":"DERPLP050000AR8IBL","aktualit":"2026-07-20","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","name":"Görres-Gymnasium Koblenz","gmdschl":"07111000","lagebeztxt":"Gymnasialstraße 3","gid":"517812"}
{"oid":"DERPLP050000AR35BL","aktualit":"2022-09-08","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","gmdschl":"07111000","gid":"516819"}
{"oid":"DERPLP050000LFomBL","aktualit":"2024-06-03","gebnutzbez":"Gebäude","funktion":"Tiefgarage","rellage":"Unter der Erdoberfläche","gmdschl":"07111000","gid":"623474"}
{"oid":"DERPLP050000AQMGBL","aktualit":"2010-07-29","gebnutzbez":"Bauteil","funktion":"Durchfahrt im Gebäude","gmdschl":"07111000","gid":"57497"}
{"oid":"DERPLP050000AO94TN","aktualit":"2026-02-05","nutzart":"Fläche besonderer funktionaler Prägung","bez":"Bildung und Wissenschaft","gid":"81684"}
{"oid":"DERPLP200003dWfbTN","aktualit":"2026-02-05","nutzart":"Platz","bez":"Fußgängerzone","gid":"648293"}
{"oid":"DERPLP200003FfKATN","aktualit":"2025-05-05","nutzart":"Straßenverkehr","bez":"Fußgängerzone","gid":"663062"}
```

The Tiefgarage `…LFomBL` lies exactly under the Platz `…dWfbTN` (Görresplatz) next to the school.

### 8.5 Observed values (seven areas: 25,651 buildings, 5,573 Nutzung objects)

- **`gebnutzbez`:** `Gebäude` 22,141, `Sonstiges Bauwerk oder sonstige Einrichtung` 1,775 (all "Überdachung"), `Bauteil` 1,735.
- **`funktion` is detailed** (51 values): Wohngebäude 12,594, Gebäude für Wirtschaft oder Gewerbe 7,960, Überdachung 1,775,
  Durchfahrt im Gebäude 1,225, Garage 595, Abweichende Geschosshöhe 312, **Tiefgarage 175**, Arkade 125, Verwaltungsgebäude 98,
  **Allgemein bildende Schule 96**, Umformer 81, Kirche 54, Kinderkrippe, Kindergarten, Kindertagesstätte 37, Parkhaus 32,
  Krankenhaus 31, Berufsbildende Schule 26, Hochschulgebäude (Fachhochschule, Universität) 24, Sport-, Turnhalle 22 …
- **`rellage`:** `Unter der Erdoberfläche` 200, `Aufgeständert` 5.
  - Underground: **Gebäude/Tiefgarage 132, Bauteil/Tiefgarage 43**, Gebäude/Gebäude für Wirtschaft oder Gewerbe 12,
    Gebäude/Garage 9, Gebäude/Wohngebäude 3, Gebäude/Parkhaus 1. **Every "Tiefgarage" has `rellage`**, so freemapper's
    `rellage` filter catches them all; a filter on `Bauteil` alone would miss 157 of 200.
  - Other Bauteile: Durchfahrt im Gebäude 1,225, Abweichende Geschosshöhe 312, Arkade 125, Durchfahrt an überbautem
    Verkehrsweg 22, Sonstiges 8 — none underground.
- **`name`** (364): clean, but **multiple names joined with `;`** ("Amtsgericht Koblenz;Landgericht Koblenz",
  "Deutschherrenhaus;Ludwig Museum;Ludwig Museum", up to seven names), one internal label
  `"AX_Bauteil (Bauart Tiefgarage 2100)"`, short names ("WC", "AOK"). No trailing blanks seen. `funktion` never has `;`.
- **`nutzart`** (LGB spellings except one): Wohnbaufläche 1,102, Straßenverkehr 1,054, Fläche gemischter Nutzung 1,023,
  Industrie- und Gewerbefläche 688, Fläche besonderer funktionaler Prägung 451, Weg 313, Sport-, Freizeit- und
  Erholungsfläche 214, Wald 182, Platz 178, Gehölz 109, Bahnverkehr 96, Landwirtschaft 62, Fließgewässer 39,
  **`Unland, Vegetationslose Fläche` 32** (comma; LGB/NI "Unland/Vegetationslose Fläche"), Schiffsverkehr 15, Friedhof 4,
  Stehendes Gewässer 4, Hafenbecken 4, Flugverkehr 2, Sumpf 1. `PRIVATE_LAND_USES` matches unchanged; no freemapper build
  rule uses the Unland spelling.
- **`bez` per `nutzart`:**
  - `Wohnbaufläche`: **`Offen` 729, `Geschlossen` 366** (Bauweise), none 7.
  - `Fläche gemischter Nutzung`: Gebäude- und Freifläche Mischnutzung mit Wohnen 1,013, Parken 6, Landwirtschaftliche/
    Forstwirtschaftliche Betriebsfläche 4.
  - `Industrie- und Gewerbefläche` (25 values): **Handel 190**, Lagerfläche 93, Freie Berufe und weitere Dienstleistungen 72,
    Bank, Kredit 61, Beherbergung 49, Handel und Dienstleistung 44, Restauration 43, Handwerk 37, Produktion 15, Parken 14,
    Vergnügung 13, Industrie und Gewerbe 13, Kraftwerk 11, Tankstelle 6 …
  - `Straßenverkehr`: none 806, **`Begleitfläche Straßenverkehr` 139 (13 %)**, `Fußgängerzone` 108, `Gebäude- und Freifläche zu
    Verkehrsanlagen, Straße` 1. Never `Fahrbahn`.
  - `Weg`: **`Fahrweg` 150**, Fußweg 120, Rad- und Fußweg 40, Radweg 2, none 1.
  - `Platz`: Parkplatz 88, **none 48 (27 %)**, Fußgängerzone 39, Busbahnhof 2, Caravan-, Wohnmobilstellplatz 1 → `square_bez`.
  - FBFP: Regierung und Verwaltung 102, Soziales 88, Bildung und Wissenschaft 69, Religiöse Einrichtung 68, Kultur 39,
    Sicherheit und Ordnung 32, Historische Anlage 16, Parken 16, Gesundheit, Kur 15, Öffentliche Zwecke 6.
  - SFE: Grünanlage 127, Spielplatz, Bolzplatz 35, Garten 20, Park 7, Sportanlage 6, Kleingarten 4, Sportplatz 3 …
  - `Bahnverkehr`: **`Begleitfläche Bahnverkehr;` 46 (trailing `;`)**, Eisenbahn 28, none 13, `Gebäude- und Freifläche zu
    Verkehrsanlagen, Schiene;` 7, Straßenbahn 2. `Unland`: Gewässerbegleitfläche 18, Naturnahe Fläche 13,
    **`Vegetationslose Fläche;Fels`** 1. `Flugverkehr`: `…Luftfahrt;Hubschrauberlandeplatz`.
  - Wald: Laub- und Nadelholz 78, Laubholz 71, Nadelholz 33. Landwirtschaft: Grünland 49, Brachland 9, Ackerland 2,
    Rebfläche 2. Fließgewässer: Bach 21, Fluss 17, Graben 1. `Gehölz`, `Hafenbecken`, `Sumpf`: no `bez`.
- The Pfälzerwald 2 km box has **0 buildings** and 136 Nutzung objects (forest).

### 8.6 What is missing or different

- MapServer GML: `aktualit` in `gml:timePosition` (parser change), bare `gml:Polygon`, extra `gid`, case-insensitive
  `SORTBY`, `numberMatched="unknown"` on full pages.
- No `NutzungFlurstueck` → `ave:Nutzung` (unsplit, no `flstkennz`, like Saxony-Anhalt). No `VerwaltungsEinheit` → the state
  boundary comes from the Verwaltungsgrenzen WFS (§10).
- `;`-joined names, `;`-terminated `bez` values, the comma in "Unland, Vegetationslose Fläche".
- Tiefgaragen as Gebäude and as Bauteil, always with `rellage`.
- Not examined: `ave:Flurstueck`, the 451 attributes in depth, LoD2, Hausumringe zip.

---

## 9. Licence and attribution

- Licence: **Datenlizenz Deutschland – Namensnennung – Version 2.0** (`https://www.govdata.de/dl-de/by-2-0`), "geldleistungsfrei".
- **Open-data page** (`https://lvermgeo.rlp.de/geodaten-geoshop/open-data/`), verbatim:
  > Die hier angebotenen Geodaten und -dienste der Vermessungs- und Katasterverwaltung Rheinland-Pfalz stehen unter der
  > Datenlizenz Deutschland – Namensnennung – Version 2.0 geldleistungsfrei zum Download und zur Online-Nutzung zur
  > Verfügung. Die Namensnennung der Vermessungs- und Katasterverwaltung Rheinland-Pfalz als Rechteinhaberin hat in
  > folgender Weise zu erfolgen: **©GeoBasis-DE / LVermGeoRP<Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de
  > [Daten bearbeitet]**
- **WMS/WFS `Fees`** (DOP WMS, ALKIS WFS, Verwaltungsgrenzen WFS; `AccessConstraints NONE`), verbatim:
  > geldleistungsfrei, Datenlizenz Deutschland – Namensnennung – Version 2.0; URL: https://www.govdata.de/dl-de/by-2-0 Die
  > Namensnennung der Vermessungs- und Katasterverwaltung Rheinland-Pfalz als Rechteinhaberin hat in folgender Weise zu
  > erfolgen: ©GeoBasis-DE / LVermGeoRP (Jahr des Datenbezugs), dl-de/by-2-0, http://www.lvermgeo.rlp.de [Daten bearbeitet];
  > Es gelten folgende Regelungen zu Gewährleistung und Haftung; URL:https://lvermgeo.rlp.de/geodaten-geoshop/open-data/gewaehrleistung-haftung/
- **Tile ISO XML** (`otherConstraints`): "©GeoBasis-DE / LVermGeoRP <Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de
  [Daten bearbeitet]" plus JSON `{"id":"dl-by-de/2.0","name":"Datenlizenz Deutschland Namensnennung 2.0","url":"https://www.govdata.de/dl-de/by-2-0","quelle":"© GeoBasis-DE / LVermGeoRP<Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"}`.
- The spacing of the year varies between sources ("LVermGeoRP<Jahr>", "LVermGeoRP (Jahr)", "LVermGeoRP <Jahr>"); the
  open-data page's form without blank is the reference. "[Daten bearbeitet]" applies because freemapper derives map objects.
- **Recommended freemapper line:** `©GeoBasis-DE / LVermGeoRP<download year>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]`
  (e.g. `©GeoBasis-DE / LVermGeoRP2026, …`), licence link `https://www.govdata.de/dl-de/by-2-0` in the project notes.
- Note: the OSM paragraph on the same page says RP data in OSM are attributed via the contributors list; not relevant here.

---

## 10. State boundary (for `crates/adapters/de/manifests/rheinlandpfalz/boundary.geojson`)

- **Source:** the Verwaltungsgrenzen WFS, type **`vermkv:landesgrenze_rlp`**, **from the origin** (the registry 314 answers
  "No wfs_conf data …"):
  ```
  https://geo5.service24.rlp.de/wfs/verwaltungsgrenzen_rp.fcgi?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=vermkv:landesgrenze_rlp&SRSNAME=urn:ogc:def:crs:EPSG::25833
  ```
  → 200 `text/xml; subtype="gml/3.2.1"`, **2 features**, 1,672,591 B, **1.8 s** (25832: 1,663,607 B, 1.7 s). The ALKIS WFS
  has no `VerwaltungsEinheit`.
- Geometry (shapely, 25833):
  - `gid 1`, `name "Rheinland-Pfalz"`: one Polygon, 45,280 vertices, **no holes**, 19,979.98 km², valid;
  - `gid 2`, `name "Deutsch-Lux. Hoheitsgebiet"`: the **German–Luxembourg condominium** (the border rivers Mosel, Sauer, Our),
    two Polygons, 4.880 km² and 1.377 km², valid, no overlap with the main polygon, sharing 81.0 km and 39.6 km of boundary
    with it. The union is one valid polygon of 19,986.24 km² (43,990 vertices).
  - The two features as a MultiPolygon are *invalid* (they touch along lines) → union them first.
- **25 m topology-preserving simplification:** `probe/boundary/out/rheinland-pfalz.geojson` = `rp/rheinland-pfalz.geojson`
  (script `scripts/simplify_rp.py`: union of both features, shapely 2.2 `simplify(25, preserve_topology=True)`,
  `set_precision(grid_size=1)`, `normalize`).
  - FeatureCollection, `name` "Rheinland-Pfalz (LVermGeo, simplified 25 m)", `crs` `urn:ogc:def:crs:EPSG::25833`, properties
    `name: Rheinland-Pfalz`, `source: LVermGeo RP WFS Verwaltungsgrenzen (vermkv:landesgrenze_rlp, incl. Deutsch-Lux.
    Hoheitsgebiet), ©GeoBasis-DE / LVermGeoRP2026, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]`, `retrieved:
    2026-11-08`, `simplified_m: 25`; MultiPolygon with integer coordinates.
  - **4,243 vertices, 70.1 KB**, one part, no holes, exterior clockwise (like the Hesse/SH/NI assets).
  - **Valid** (shapely). The repo test `region_boundaries_are_valid` (geo `check_validation`) was not run.
  - Area 19,986.33 km² (full 19,986.24), symmetric difference 6.0 km², **Hausdorff 25.3 m**.
  - Extent E −135,707 … 30,572, N 5,445,770 … 5,668,185: **84.5 % of the area at negative eastings** in 25833.
- **Against the neighbour assets** (`scripts/border_rp.py`, `scripts/gap_detail.py`; shared border = true RP boundary within
  150 m of the neighbour asset; deviation sampled every 50 m):

  | Neighbour asset | Shared border | Overlap (asset ∩ asset) | Gap between them (within 60 m of the border) | True border → RP asset p99 / max | → neighbour asset p99 |
  |---|---|---|---|---|---|
  | `crates/adapters/de/manifests/hessen/boundary.geojson` (repo) | 277.3 km | 162,163 m², 70 pieces, max width 24.5 m | 180,476 m², 82 pieces | 20.9 / 24.4 m | 21.2 m |
  | `nrw/probe/boundary/nordrhein-westfalen_simplified25m.geojson` (scratchpad) | 316.9 km | 35,474 m², 42 pieces, max width 25.8 m | 73,178 m², 40 pieces | 21.0 / 24.5 m | 21.0 m |

  - The wide gap pieces are at tripoints: 67.5 m near 49.583°N 8.423°E (Hesse/Baden-Württemberg/RP at Worms–Lampertheim; the
    area outside both is Baden-Württemberg), 60.0 / 78.8 m at 50.686°N 8.124°E (RP/Hesse/NRW tripoint), 63.2 m at 50.323°N
    6.403°E (RP/NRW/Belgium). Along the lines the slivers are ≲ 25 m wide, as for NRW–Hesse and NI.
  - **Full resolution:** RP Landesgrenze vs the HVBG ALKIS polygon of Hesse (`hessen/probe/boundary/land_DE06_25833.xml`):
    overlap 7,173 m² in 1,390 slivers; vs the NRW full polygon: 10,184 m² — the official borders agree to centimetres-to-decimetres.
  - So a point within ≈25 m of a border can be in both or neither asset; `regions_for` buffers by `MARGIN_M`.
- Neighbours: North Rhine-Westphalia, Hesse, Baden-Württemberg, Saarland, France, Luxembourg (condominium), Belgium. BW and
  Saarland have no freemapper asset yet (not compared); the foreign borders were not checked.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/rheinland-pfalz-latest.osm.pbf` → **307** →
  `rheinland-pfalz-261107.osm.pbf` → 200, **274,845,689 B** (262 MiB), `Last-Modified: Sun, 08 Nov 2026 02:05:17 GMT`,
  `ETag "1061cff9-65fba46d96bac"`, `Accept-Ranges: bytes`. (NI 510 MB, NRW 917 MB, Hesse 348 MB.) Within `MAX_EXTRACT_BYTES`.
- State: `https://download.geofabrik.de/europe/germany/rheinland-pfalz-updates/state.txt`
  ```
  # original OSM minutely replication sequence number 7362973
  timestamp=2026-11-07T21\:21\:36Z
  sequenceNumber=4957
  ```
- Poly: `https://download.geofabrik.de/europe/germany/rheinland-pfalz.poly` (20,940 B, one ring, 675 points, lon 6.109–8.511,
  lat 48.964–50.944). Contains Mainz and the Koblenz school and the Luxembourg bank at Wasserbillig (buffered); **excludes
  Wiesbaden, Mainz-Kastel (Hesse), Mannheim, Saarbrücken, Bonn**. A Mainz-Rhine map that reaches Kastel needs the Hesse extract too.
- Overpass answered 200 with `freemapper-research/0.1 (github.com/HenryJobst/freemapper)` (36–38 s per query).

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `geobasis-rlp.de/data` | HEAD/Range/ETag/Last-Modified OK; **Range → 206, multi-range → 206 multipart, beyond EOF → 416**, missing → **404 HTML**; **latency is erratic**: the same 78 B `.tfw` took 0.43, 0.57, 0.62, 1.5, 3.0 and **8.8 s** (TTFB) in six consecutive requests; a 1.27 MB DGM 8.7 s once, 1.5–3.4 s otherwise; throughput ≈1 MB/s; HTML listings up to 9 MB / 52 s; `ln/current/` → **502 Proxy Error** |
| `geo4.service24.rlp.de/wms/rp_dop20.fcgi` | **max 10000 px**, oversize → **200 `text/xml` ServiceExceptionReport**; outside RP → white image (200); 5000² in 4–10 s; GFI 0.13–0.15 s; 5,265 GFI in ≈10 min without an error |
| `geo5.service24.rlp.de/wfs/alkis_rp.fcgi` | GML 3.2 / GeoJSON; **COUNT ≥ 200000 accepted, default 100000**; errors are **400 OWS ExceptionReports**; unknown type 5.5 s; no gzip; ≈1 MB/s; one 0.74 MB answer took 12.9 s |
| `www.geoportal.rlp.de/registry/wfs/*` | proxy of geo5; 519 identical bytes but 15 `Set-Cookie` headers; **314 broken (200 text/html "No wfs_conf data …")** |
| `www.geoportal.rlp.de/spatial-objects/*` | WGS84-only OGC API façade, slow (11 s), lossy |
| `geo4.service24.rlp.de/wcs/rp_dgm1.fcgi` | **401** (protected) |
| Geofabrik | 307 to the dated file; 275 MB |

- **TLS (checked with `openssl s_client -showcerts -CAfile <certifi bundle> -verify_return_error`, which does no AIA
  fetching): every host verifies, `Verify return code: 0 (ok)`:**
  - `geobasis-rlp.de` (TLS 1.3): 2 certs, `*.geobasis-rlp.de` → Sectigo Public Server Authentication CA DV R36 → **Sectigo
    Public Server Authentication Root R46**;
  - `geo4.service24.rlp.de`, `geo5.service24.rlp.de`, `metadatenkatalog-vermkv.service24.rlp.de` (TLS 1.2): 3 certs,
    `*.service24.rlp.de` → GlobalSign GCC R6 AlphaSSL CA 2025 → **GlobalSign Root CA – R6** (root included);
  - `www.geoportal.rlp.de` (TLS 1.2): `*.geoportal.rlp.de` → Thawte TLS RSA CA G1 → **DigiCert Global Root G2**;
  - `lvermgeo.rlp.de` (TLS 1.3): `*.rlp.de` → Telekom Security OV RSA CA 26A → **Telekom Security TLS RSA Root 2023**;
  - `download.geofabrik.de`: Let's Encrypt YE1 → Root YE → ISRG Root X2 → ISRG Root X1.
  All five roots are in `webpki-roots` 1.0.11 (freemapper's lock file; grepped). **Rust check:** `ureq` 3.4.2 (rustls +
  webpki-roots, freemapper's pin) fetched from `geobasis-rlp.de`, `geo4…`, `geo5…`, `www.geoportal.rlp.de` and
  `lvermgeo.rlp.de` without error (`rustcheck/`). **No SH-style missing intermediate.**
- No authentication, no rate-limit or `Retry-After` headers, no throttling in ≈5,500 WMS and ≈100 other requests.
- Total transfer for this research ≈ **0.35 GB** (DOP JP2 123 MB, WMS ≈45 MB, listings ≈35 MB, ALKIS ≈70 MB, ALS index 9 MB,
  rest small); disk under `rp/` ≈430 MB.

---

## 13. What makes the Rhineland-Palatinate adapter differ (risks first)

1. **Tile URLs need the year.** DGM1/DOM1 names end in the ALS year, DOP names in the flight year. Use the Blattschnitt WFS
   `Laserbefliegung` (`flugjahr` per `kachel`, one small bbox query) for DGM/DOM, falling back to trying 2025…2022 on 404
   for the 13 tiles without `flugjahr`; the DOP comes from the WMS anyway. A plain "Brandenburg-style computed URL" adapter
   does not work.
2. **ALKIS GML from MapServer:** `aktualit` sits in `<gml:timePosition>` → `parse_gml` loses it today (needs a change and a
   fixture); otherwise compatible (bare `gml:Polygon`, extra `gid`, empty elements). `ave:Nutzung` instead of
   `NutzungFlurstueck` (like Saxony-Anhalt), case-insensitive `SORTBY`.
3. **Use the origin host** `geo5.service24.rlp.de`, not the published registry URL: the registry adds a Mapbender layer that is
   already broken for the boundary WFS (200 + HTML text) — `get_with_retries`' validator must treat a non-XML 200 as an error
   (it does for the WFS via `parse_gml`).
4. **Erratic file-server latency** (single small requests up to 9 s): generous timeouts and retries; avoid HTML listings.
5. **Orthophoto only via WMS** in practice (JP2 tiles lossless 105–155 MB, no TLM; no COG): `WmsService{max_px: 10000}`,
   pieces ≤ 4000 px work. The WMS is clipped at the state border (Hesse side white).
6. **DOM1 has two encodings** (2022–2024 without TFW, IFD at the end, no predictor; 2025 with TFW and predictor 2): read the
   GeoTIFF tags, never require a `.tfw`. DGM1 uses LZW + Predictor 2 on float32 (supported by `tiff` 0.11.3, verified).
7. **Currency:** DOP always leaf-on summer (2022–2025, 2-year cycle), ALS always leaf-off spring (2022–2025, 4-year cycle),
   within ±3 years of each other; DOM can be newer than the DOP (26 % of tiles).
8. **CRS 25832** everywhere; 84.5 % of RP at negative 25833 eastings. `Crs::Utm32`.
9. **Naming clash:** freemapper already has `Provider::LvermGeo` for Saxony-Anhalt; RP's agency has the same short name
   ("LVermGeo") → e.g. `Provider::LvermGeoRp`, oid prefix `DERP`.
10. ALKIS content: `;`-joined names, `;`-terminated `bez`, "Unland, Vegetationslose Fläche", Tiefgaragen with `rellage`,
    `Platz` without `bez` 27 % → `square_bez`.
11. Unchanged from the other states: no login, DHHN2016, NW-corner tiepoint / PixelIsArea, Float32 with NoData −9999, 1 km
    DGM tiles with TFW, LGB spellings, `ave:` WFS, Geofabrik layout, dl-de/by-2-0 (as Brandenburg).

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Rhineland-Palatinate | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `WmsService { url: "https://geo4.service24.rlp.de/wms/rp_dop20.fcgi", layer: "rp_dop20", crs: Crs::Utm32, extra: "", max_px: 10000 }`, JPEG pieces ≤ 4000 px | Date: one GFI on `rp_dop20_info` (`INFO_FORMAT=text/plain`, parse `erstellung = 'DD.MM.YYYY'`), leaf-on. Oversize → 200 XML report (the `is_ogc_report` check covers it). Outside RP white. |
| **DTM** (hillshade, slope, contours) | **DGM1 tiles** `https://geobasis-rlp.de/data/dgm1/current/tif/dgm1_32_{E}_{N}_1_rp_{flugjahr}.tif` (+ `.tfw`), `flugjahr` from `https://geo5.service24.rlp.de/wfs/blattschnitt_rp.fcgi` `TYPENAMES=Laserbefliegung&BBOX=…` | LZW + predictor 2 Float32, NoData −9999, 1.2–1.8 MB; currency = last `flugtage` day; ETag/Last-Modified as `published`. Timeouts ≥ 30 s. |
| **nDSM template** | **DOM1 − DGM1** (`…/dom1/current/tif/dom1_32_{E}_{N}_1_rp_{flugjahr}.tif`, same year as the DGM) | Read georeference from GeoTIFF tags (no TFW for 2022–2024); clamp < 0; warn "ALS <date> (leaf-off) vs DOP <date> (leaf-on)". bDOM20 LAZ (270 MB / 2 km) not recommended. |
| **Buildings** | **WFS** `https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi`, `ave:GebaeudeBauwerk`, 25832, `SORTBY=oid&COUNT=10000&STARTINDEX=…` | Reuse `alkis_wfs`; **fix `aktualit` in `gml:timePosition`**; underground via `rellage`; split/trim names on `;`. oid prefix `DERP`. |
| **Land use** (out-of-bounds, paved) | the same WFS, **`ave:Nutzung`** | Proposed: `WfsService { name: "LVermGeo RP ALKIS", url: "https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi", buildings_type: "ave:GebaeudeBauwerk", land_use_type: "ave:Nutzung", crs: Crs::Utm32, square_bez: Some("ohne Unterart (Rheinland-Pfalz)") }`. Trim a trailing `;` from `bez`. |
| **Region boundary** | Verwaltungsgrenzen WFS (origin) `vermkv:landesgrenze_rlp` in 25833, union with the condominium, simplified 25 m | Ready: `rp/rheinland-pfalz.geojson` (4,243 vertices, 1 part, valid). Region CRS Utm32. |
| **OSM** | Geofabrik `europe/germany/rheinland-pfalz-latest.osm.pbf` + `rheinland-pfalz-updates/state.txt` | 275 MB; follow the 307; Mainz–Wiesbaden maps need the Hesse extract too. |
| **Attribution** | `©GeoBasis-DE / LVermGeoRP<year>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]` + licence link + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- That `erstellung` is the flight date (inferred from 24 clustered summer dates and the leaf-on imagery; no field says
  "Bildflugdatum"); the CSW record linked from GFI was not opened.
- The behaviour of the ALKIS WFS above 100,000 features without COUNT (default cap; silent cut or not) — not tried.
- freemapper's actual `parse_gml` on an RP answer (only the code was read; the `aktualit` loss is a code-reading result);
  geo's `check_validation` on the asset; a full freemapper run.
- DGM/DOM border tiles towards France, Luxembourg, Belgium, Saarland and BW (NoData share); the WMS clipping at those borders.
- Why the DGM XML says `Erfassung 2024-03-15` while the ALS index says 30.03./06.04.2024.
- The 13 tiles without `flugjahr` (E 333–337, N 5537–5539) — their DGM/DOM names say 2025. The live index sample of 2026-11-08 used for the example has a 14th, `3335536` (E 333, N 5536).
- Rate limits / fair-use rules (none seen in ≈5,600 requests; none found on the open-data pages); the stability of the
  geo5 host (a GeoPortal.rlp news item about a geo5 maintenance outage turned up in a web search, not opened).
- `ln/current/` (Landnutzung) answered 502 at the time; `lb` (1.9 GB) and `hu` (319 MB) zips not examined.
- The 451 TN-Einzellayer and 543 Flurstücksabschnitte in depth; LoD2; historic DOP WMS-T.
