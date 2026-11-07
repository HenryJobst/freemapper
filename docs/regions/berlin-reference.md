# Berlin geodata reference for a freemapper Berlin adapter

Verified live on 2026-11-07 (HTTP requests from macOS, curl and Python urllib, no login).
Everything below was observed unless it is marked **UNVERIFIED**. Samples are in
(research scratch folder, not in the repository)
(about 165 MB were downloaded in total; whole datasets were not downloaded).

## 0. Result in one table

| freemapper product | Recommended source for Berlin | Why | New code? |
|---|---|---|---|
| Orthophoto template | **LGB DOP20 tiles as today** (`dop/rgb_jpg/dop_33EEE-NNNN.zip`), which already cover all of Berlin. Optional upgrade: Berlin **TrueDOP 2026 WMS** (`truedop_2026`) for tiles where the LGB tile is older than the Berlin flight of 2026-03-05/06/09 | LGB tiles exist for 1188 of 1188 1 km tiles under the Berlin grid. 548 of them are the Berlin 2026 TrueDOP; the outer districts carry LGB flights from 2023 to 2026 | none (optional WMS fallback) |
| DTM (hillshade, slope, contours) | **LGB DGM1 tiles as today** (`dgm/tif/dgm_33EEE-NNNN.zip`) | Same values as the Berlin DGM1 (checked at 3 points, identical to the cm), but in GeoTIFF f32 with the LGB currency CSV, 1 km tiles, about 1.3 MB per tile instead of an 18 MB XYZ zip | none |
| nDSM template | **LGB bDOM 0.2 m GeoTIFF** (`bdom/tif/bdom_33EEE-NNNN.zip`, currency in `bb_bdom_aktualitaet.csv`) minus LGB DGM1 | The LGB nDOM LAZ is **missing inside Berlin** (404), but the LGB bDOM covers all Berlin tiles (974 of 974 sampled), 666 of them from the Berlin flight of March 2026 | yes: an nDSM path from a GeoTIFF DSM minus the DTM, used where the nDOM LAZ is 404 |
| ALKIS buildings | **Berlin WFS** `https://gdi.berlin.de/services/wfs/alkis_gebaeude`, type `alkis_gebaeude:gebaeude`, `outputFormat=application/json` | The LGB OGC API returns 0 features in Berlin | yes: Berlin adapter with an attribute mapping (section 7) |
| ALKIS land use | **Berlin WFS** `https://gdi.berlin.de/services/wfs/alkis`, type `alkis:tatsaechlichenutzungflaechen` | Not in the LGB API for Berlin | yes: class mapping (section 8); fewer refinements than Brandenburg |
| OSM | **Geofabrik `brandenburg-latest.osm.pbf` as today**: it is "Brandenburg (mit Berlin)" and contains Berlin | Verified on the Geofabrik page and the `.poly` file | none (optional: `berlin-latest.osm.pbf`, 95 MB, for areas only in Berlin) |

**The user's starting assumption is wrong in two places:** (1) the Geofabrik Brandenburg extract *includes*
Berlin, and (2) LGB already serves DOP, DGM and bDOM for all of Berlin in the familiar 1 km scheme with
per-tile currency. What is missing for Berlin is only ALKIS (buildings, land use) and the nDOM.
A full Berlin raster adapter (ATOM, 2 km tiles, dl-de/zero-2-0) is possible, but it is not needed for the
MVP products.

Freemapper today in a Berlin area (from code reading, not run): `check_area` accepts it
(`LGB_COVERAGE` = 250000..500000 E, 5650000..6000000 N); DOP and DGM download fine; the nDOM is
skipped on 404 (`FetchError::NotFound` → "skipped", `fetch.rs` ~l.475); **ALKIS buildings and land use come
back as an empty FeatureCollection without an error**, so a Berlin project is silently left without buildings.
That is the first thing a Berlin adapter must fix (anti-goal "wrong data presented as fact").

---

## 1. What freemapper takes from Brandenburg (baseline)

From `crates/freemapper/src/sources/lgb.rs`, `alkis.rs`, `osm.rs`, `tiles.rs`, `fetch.rs`, `project.rs`, spec §4.1:

- Grid: 1×1 km, EPSG:25833, tile id `33EEE-NNNN` = SW corner in km (`tiles.rs`).
- DOP20 RGB: `https://data.geobasis-bb.de/geobasis/daten/dop/rgb_jpg/dop_33EEE-NNNN.zip` → `dop_….jpg` + `.jgw`.
- DGM1: `…/dgm/tif/dgm_33EEE-NNNN.zip` → `dgm_….tif` (f32, pixel-centre) + `.tfw`.
- nDOM: `…/ndom/NDOM_33EEE-NNNN_LAS12.laz` (no zip; currency = HTTP `Last-Modified`).
- Currency: `…/information/aktualitaeten/bb_dop_aktualitaet.csv`, `bb_dgm_aktualitaet.csv`
  (`sheetnr;sheetname;creationdate;revisiondate;publicationdate;…`); `published` names the cache file; a change
  is reported as "changed upstream".
- ALKIS: OGC API Features `https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/{gebaeude_bauwerk|nutzung_flurstueck}/items`,
  `f=json&bbox=…&bbox-crs=…25833&crs=…25833&limit=10000`, paging through `links[rel=next]`, dedup by `oid`.
  Buildings use `oid`, `gebnutzbez` (`Gebäude`/`Bauteil`), `funktion`, `rellage == "Unter der Erdoberfläche"`, `aktualit`.
  Land use uses `oid`, `nutzart`, `bez`, `aktualit`. Rules: `build/buildings.rs` (only `Gebäude`, not underground),
  `build/paved.rs` (`Straßenverkehr` except `bez = Begleitfläche Straßenverkehr`; `Platz` with no `bez` or with
  `Parkplatz|Marktplatz|Fußgängerzone`; `Weg` → candidate), `build/out_of_bounds.rs` (private: `Wohnbaufläche`,
  `Industrie- und Gewerbefläche`, `Fläche gemischter Nutzung`, `Bahnverkehr`; uses parcel parts, `OWN_LAND_SHARE`).
- OSM: `https://download.geofabrik.de/europe/germany/brandenburg-latest.osm.pbf` + `brandenburg-updates/state.txt`.
- Attribution: `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert`, `© OpenStreetMap contributors, ODbL`.

---

## 2. Main finding: LGB already serves Berlin rasters

| Check | Result |
|---|---|
| `HEAD https://data.geobasis-bb.de/geobasis/daten/dop/rgb_jpg/dop_33392-5820.zip` (Berlin-Mitte) | 200, 7 730 897 B, Last-Modified 2026-08-05 |
| `HEAD …/dgm/tif/dgm_33392-5820.zip` | 200, 1 267 895 B, Last-Modified 2026-03-18 |
| `HEAD …/bdom/tif/bdom_33392-5820.zip` | 200, 29 681 774 B, Last-Modified 2026-09-10 (0.2 m GeoTIFF + `.tfw` + `_meta.html`) |
| `HEAD …/ndom/NDOM_33392-5820_LAS12.laz` | **404** |
| LGB OGC API `gebaeude_bauwerk` with bbox 392700,5820750,392900,5820950 | 200, **0 features** |
| `bb_dop_aktualitaet.csv` / `bb_dgm_aktualitaet.csv` | contain all 1188 1 km tiles under the Berlin 2 km DGM grid |
| `bb_bdom_aktualitaet.csv` | contains all 974 1 km tiles whose centre lies in Berlin (Geofabrik `berlin.poly`) |

Content checks:
- `dop_33392-5820.html` in the zip: "Bildflugdatum: 2026-03-05", "Veröffentlichung: 2026-07-29"; 5000×5000 RGB JPEG,
  `.jgw` 0.2 / 392000.1 / 5820999.9. Correlation with Berlin `truedop_2026` WMS (same 500 m box, 1 m/px): **0.98**,
  so it is the Berlin TrueDOP 2026.
- `dgm_33392-5820.tif`: f32 LZW 1000×1000, `.tfw` 1 / 392000.5 / 5820999.5. Values at (392000.5, 5820000.5) = 35.96,
  (392000.5, 5820999.5) = 37.30, (392500.5, 5820500.5) = 37.64: **identical** to the Berlin DGM1 XYZ.
  CSV: creation 2021-02-25 (Berlin ALS flight), revision 2025-12-01, published 2026-01-26.
- LGB DOP source dates over Berlin (1 km tiles with centre inside Berlin):
  2026-03-05/06/09 (Berlin flight) for **548** tiles in a central block (about E 376–405 km, N 5810–5829 km);
  the outer districts are older or Brandenburg flights: 2023-05-04 (south-east, e.g. `33406-5812`, which correlates
  only 0.61 with Berlin 2026, so it is different imagery), 2024-04-30/05-01 (west/south-west), 2025-08-12 (south-east,
  probably Berlin "TrueDOP Sommer 2025", **UNVERIFIED**), 2026-04-22/05-01 (north).
- LGB nDOM in Berlin: sampled 12 tiles. All 6 inner tiles → 404. The 6 border tiles (centre inside Berlin, a corner
  outside) and `33378-5809` → 200, 11–19 MB. **UNVERIFIED** whether those cover the Berlin part of the tile or only
  the Brandenburg part (LAZ not decoded).

Attribution note inside the LGB zips (verbatim):
- DOP: "Hinweis: werden auch Daten des Bundeslandes Berlin genutzt, ist der Copyright-Vermerk um den Hinweis
  “© Geoportal Berlin, dl-de/zero-2-0 “ zu ergänzen. Beispiel: „© GeoBasis-DE/LGB, dl-de/by-2-0; © Geoportal Berlin,
  dl-de/zero-2-0 (Daten geändert)“"
- bDOM: same sentence but with "© Geoportal Berlin, dl-de/by-2-0" (inconsistent with the DOP note).
- DGM: no Berlin note.

---

## 3. Berlin portal basics

- Geoportal: `https://gdi.berlin.de/` redirects (meta refresh) to `https://gdi.berlin.de/viewer/main/`.
  The old FIS-Broker `https://fbinter.stadt-berlin.de/fb/` → 301 to the same viewer, so **FIS-Broker URLs are dead**.
- Downloads: `https://gdi.berlin.de/data/<dataset>/atom/` (INSPIRE service feed) → `…/atom/0.atom` (dataset feed),
  static Apache files (`x-be: static`, `accept-ranges: bytes`, `ETag`, `Last-Modified`, `access-control-allow-origin: *`).
  Directory listings are 403 (`/data/`, `/data/truedop_2026/`).
- Services: `https://gdi.berlin.de/services/wms/<name>`, `https://gdi.berlin.de/services/wfs/<name>` (GeoServer behind
  the proxy; exception reports reference `http://10.10.212.83/geoserver/…`). No WCS: `/services/wcs/dgm1`,
  `/services/wcs/bdom`, `/services/wcs/truedop_2026` → 404.
- Catalogue: `https://datenregister.berlin.de/api/3/action/package_search?q=…` and `package_show?id=…` work without auth
  (CKAN; `status_show` needs auth). The `daten.berlin.de` website answers 403 to curl (any UA tried).
  ISO metadata: `https://gdi.berlin.de/geonetwork/srv/ger/csw?REQUEST=GetRecordById&SERVICE=CSW&VERSION=2.0.2&ID=<id>&ELEMENTSETNAME=full&outputSchema=http://www.isotc211.org/2005/gmd`.
- Product docs: `https://gdi.berlin.de/data/dgm1/docs/dgm1.pdf`, `https://gdi.berlin.de/data/bdom/docs/dom.pdf` (200).
- **Reliability:** about 10 of roughly 200 requests to `gdi.berlin.de` failed with "connection reset by peer" or a
  timeout (60–120 s). This hit static files (range reads on ALS ZIPs), the WFS, the WMS and the WCS probes, and was
  worst during bursts of quick requests. Every failed request worked when retried. No rate-limit headers were seen.
  A Berlin fetcher needs retries with backoff and a connect timeout well below 120 s.
- Authentication: none for anything listed here.

---

## 4. Orthophoto

### 4.1 Berlin TrueDOP 2026 (newest; 20 cm RGBI)
- Catalogue: `digitale-farbige-trueorthophotos-2026-dop20rgbi-9ff3159b` ("Digitale farbige TrueOrthophotos 2026 (DOP20RGBI)"),
  licence `dl-de-zero-2.0`, creation 2026-03-05, publication/revision 2026-05-25, "annually".
  Lineage: "Bildflug 05., 06. und 09. März 2026". 0.20 m, positional accuracy ±0.4 m.
- Service feed: `https://gdi.berlin.de/data/truedop_2026/atom/` (200, `application/atom+xml`, 2848 B, `<updated>2026-05-27T11:54:59Z`).
- Dataset feed: `https://gdi.berlin.de/data/truedop_2026/atom/0.atom` (200, 3166 B). One `<entry>` with **9 regional ZIPs**
  (`rel="section"`, `type="application/x-jp2"`) plus `Blattschnitt2x2km.gif`:

  | Region URL (`…/truedop_2026/atom/…`) | Content-Length | Last-Modified | members |
  |---|---|---|---|
  | `Mitte.zip` | 3 201 636 189 | 2026-05-20 | 40 |
  | `Nord.zip` | 2 763 687 284 | 2026-05-20 | 40 |
  | `Nordost.zip` | 122 369 563 | 2026-05-20 | 4 |
  | `Nordwest.zip` | 1 206 912 397 | 2026-05-20 | 19 |
  | `Ost.zip` | 1 470 549 867 | 2026-05-20 | 21 |
  | `S%C3%BCd.zip` (Süd) | 3 183 168 066 | 2026-05-20 | 43 |
  | `S%C3%BCdost.zip` (Südost) | 3 472 803 667 | **2026-06-23** | 53 (with a `Südost/` folder prefix) |
  | `S%C3%BCdwest.zip` | 2 216 927 955 | 2026-05-20 | 35 |
  | `West.zip` | 2 232 262 923 | 2026-05-20 | 29 |

  URL spelling changes from year to year: 2026 uses UTF-8 umlauts (`Süd.zip`); `truedop_2024` uses `Sued.zip`, `Suedost.zip`.
  The size titles in the feed ("Mitte (3,13 GB)") are copied between years and are not reliable.
- Tiles: **283** members `dop20rgbi_33_EEE_NNNN_2_be_2026.jp2` (2 km × 2 km, SW corner in km, EEE/NNNN even),
  E 368–414, N 5798–5836, so the grid is 368 000–416 000 E × 5 798 000–5 838 000 N. No duplicates between regions.
  The ZIPs contain **no world files**.
- Format (from `dop20rgbi_33_400_5828_2_be_2026.jp2` and the header of `dop20rgbi_33_398_5824_2_be_2026.jp2`):
  JPEG 2000 (JP2), 10000×10000 px, 4 components (R,G,B,NIR) 8-bit, **one single 10000×10000 codestream tile**,
  12 quality layers, 9/7 irreversible wavelet (lossy). Georeferencing is GMLJP2 (`asoc`/`gml.root-instance`:
  envelope `urn:ogc:def:crs:EPSG::25833` 400000 5828000 / 402000 5830000, origin `400000.1 5829999.9` = pixel centre,
  offset vectors `0.2 0` and `0 -0.2`) plus a GeoJP2 `uuid` box (GeoTIFF tags, "ETRS_1989_UTM_Zone_33N").
  A full tile is about **80 MB** (deflate in the ZIP saves almost nothing; average over all 283 = 70 MB); edge tiles
  are clipped (e.g. 228 KB).
- Per-tile access without downloading the region: **verified** with HTTP range requests (`Range: bytes=…` → 206).
  Read the end of central directory (last 64 KB), then the central directory (ZIP32 here; the ALS ZIPs are ZIP64),
  then the local header and the deflated member. Script: `(scratch)/scripts/remotezip.py`.
  `If-Range` with the ETag works (206 if unchanged, 200 if not), so a change to the region ZIP (Südost was repacked on
  2026-06-23 and every offset moved) can be detected and handled safely.
- Pure-Rust risk: decoding a 10000×10000 single-tile lossy JP2 needs a JPEG 2000 decoder (OpenJPEG is C; pure-Rust
  decoders exist but were not tested here, **UNVERIFIED**) and about 400 MB of memory per tile. This is the main
  reason to prefer the LGB JPEG or the WMS.

### 4.2 Berlin TrueDOP 2026 WMS (good for clipping)
- `https://gdi.berlin.de/services/wms/truedop_2026?request=GetCapabilities&service=WMS` (200, 11 802 B, WMS 1.3.0).
  Layer **`truedop_2026`** (queryable=0, opaque=0). CRS EPSG:25833, 25832, 4326, 4258, 3857, CRS:84.
  EPSG:25833 bbox 366919.0, 5796949.6, 416682.6, 5839064.1. No `MaxWidth`/`MaxHeight` advertised.
  GetMap formats include `image/jpeg`, `image/png`, `image/geotiff`, `image/tiff`.
  Fees: "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - Version 2.0 anzuwenden …";
  AccessConstraints: "Es gelten keine Zugriffsbeschränkungen."
- Working example (1 km freemapper tile at native 0.2 m, aligned to the source pixel grid):
  `https://gdi.berlin.de/services/wms/truedop_2026?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=truedop_2026&STYLES=&CRS=EPSG:25833&BBOX=392000,5820000,393000,5821000&WIDTH=5000&HEIGHT=5000&FORMAT=image/jpeg`
  → 200 `image/jpeg`, 4 425 223 B, 18.5 s (progressive JPEG, 3 components). In EPSG:25833 the BBOX order is E,N.
- Limits tested: 500, 1000, 2000, 2048, 2049, 2500, 4096, 5000, 6000 and **8000×8000** px (2 km box, 13.2 MB JPEG, 34 s): all 200.
  `image/png` comes back as RGBA (transparent outside the data); `image/geotiff` is an uncompressed RGBA TIFF
  (2500² = 25 MB, 26 s), so it is not worth using.
- Pitfall: one `image/jpeg` request at 2500 px returned **HTTP 200 with `text/xml`** (ServiceExceptionReport
  "java.awt.image.RasterFormatException: MultiPixelPackedSampleModel has only one band."); the same request worked
  when repeated. Check the Content-Type, never only the status code.
- A world file has to be written by freemapper (0.2, 0, 0, −0.2, minE+0.1, maxN−0.1).

### 4.3 Other orthophoto datasets (all ATOM + regional ZIPs of JP2, same pattern)
`truedop_2025_sommer` (flown 2025-08, `…/truedop_2025_sommer/atom/0.atom`), `dop_2025_fruehjahr` (non-true DOP 2025),
`truedop_2024` (regional names `Sued.zip` …). `truedop_2025` → 404. Only the `truedop_2026` WMS was verified.
A GetMap to `…/services/wms/truedop_2025_sommer` with `LAYERS=truedop_2025_sommer` returned 200 with an XML
body, so either the service or the layer name is different (**UNVERIFIED**; read it from the CKAN `…-wms-…` entries).

---

## 5. DTM: Berlin DGM1

- Catalogue `atkis-dgm-digitales-gelandemodell-fa02f9e1`, `dl-de-zero-2.0`, revision 2025-12-18, publication 2026-01-29.
  Lineage (CSW dataset record `f0e8ff09-2887-3446-9c53-81dbc45af03c`): "Airborne Laserscanning Flug 24.02.2021, 25.02.2021
  und 02.03.2021; Aktualisierung mittels stereoskopischer Auswertung des Bildflugs 03/2022, 04/2023, 03/2025 und 08/2025".
- Doc `dgm1.pdf`: "Koordinatensystem: ETRS89 ⁄ UTM Zone 33N (EPSG 25833)", "**Höhenbezug: Deutsches Haupthöhennetz 2016
  (DHHN2016)**", "xyz-Datei in der Form Rechtswert⁄Hochwert⁄Höhe, gekachelt in einem Zuschnitt von jeweils 2x2 km",
  buffer of about 250 m into Brandenburg. Accuracy up to ±10 cm + 5 % of the grid spacing (flat open ground).
  (LGB's DGM is DHHN2016 too; the values match, see section 2.)
- Service feed `https://gdi.berlin.de/data/dgm1/atom/` → dataset feed `https://gdi.berlin.de/data/dgm1/atom/0.atom`
  (200, 44 377 B, Last-Modified 2026-02-16, `<updated>2025-12-18T11:54:57Z`): one entry, **297 per-tile links**
  `https://gdi.berlin.de/data/dgm1/atom/DGM1_EEE_NNNN.zip` (`type="text/csv"`), plus `2X2_EPSG_25833.gif`.
  Tiles E 368–416, N 5798–5836 (2 km, even km, aligned with the LGB 1 km grid: one Berlin tile = 4 LGB tiles).
- Example: `https://gdi.berlin.de/data/dgm1/atom/DGM1_392_5820.zip` → 200 `application/zip`, 18 048 778 B,
  Last-Modified 2026-01-23 09:27:01, ETag `"113670a-6490ac272b1aa"`. Member `dgm1_33_392_5820_2_be.xyz`
  (120 000 000 B, ZIP date 2025-12-16), 4 000 000 lines:
  `392000.500 5820000.500 35.96\r\n` (space-separated, 3/3/2 decimals, CRLF, pixel centres at .5 m, z range 30.82–78.37).
  Order in full tiles: from SW, x fastest, y increasing.
- **Edge tiles are sparse and ordered differently:** `DGM1_416_5810.zip` (21 535 B) holds 6272 points, sorted by x then y
  (`416000.500 5810302.500 32.30`), with no nodata values: missing cells are absent. The parser must place each point
  by its coordinates.
- WMS (`https://gdi.berlin.de/services/wms/dgm1`) layers: `dgm1`, `c_dgm1`, `dgm1_schummerung`, `b_dgm1_schummerung`,
  `dgm1_hoehenlinien`, `a_dgm1_hoehenlinien`. These are rendered images, not heights, and there is no WCS.

**Recommendation:** use the LGB DGM GeoTIFF tiles (same data, already supported). If a Berlin-native path is ever
needed, parse XYZ into a 2000×2000 f32 grid (NaN for absent points), split it into 4 LGB tiles and take currency
from `Last-Modified`.

---

## 6. Surface models, LiDAR and nDSM

| Dataset | Feed / URL pattern | Format | Date | Example |
|---|---|---|---|---|
| bDOM (image-based DSM) | `https://gdi.berlin.de/data/bdom/atom/0.atom` → `https://gdi.berlin.de/data/bdom/atom/EEE_NNNN.zip` (282 tiles, **no prefix**) | zip → `dom1_33_EEE_NNNN_2_be_2026.xyz`, 1 m | flown 05/06/09 Mar 2026 (metadata; `dom.pdf` still says summer 2025) | `392_5820.zip`: 20 397 437 B, Last-Modified 2026-05-27; 4 000 000 points `392000.50 5821999.50 54.95 \r\n` (2 decimals, **trailing space**, order **from NW**, y decreasing), z 30–302 |
| DOM (ALS DSM) | `https://gdi.berlin.de/data/dom/atom/0.atom` → `…/dom/atom/DOM1_EEE_NNNN.zip` (297 tiles) | zip → `dom1_33_EEE_NNNN_2_be.txt`, 1 m, same style as the DGM | ALS Feb/Mar 2021, not updated | `DOM1_392_5820.zip`: 20 530 185 B, member 120 006 998 B |
| ALS point cloud | `https://gdi.berlin.de/data/a_als/atom/0.atom` → 9 regional ZIPs (`Mitte.zip` 36.7 GB, `Nord.zip` 32.0 GB, `Nordost.zip` 0.98 GB, `Nordwest.zip` 16.1 GB, `Ost.zip` 15.7 GB, `Sued.zip` 38.3 GB, `Suedost.zip` 51.3 GB, `Suedwest.zip` 31.4 GB, `West.zip` 26.9 GB) | **uncompressed LAS 1.4** (`3dm_33_EEE_NNNN_1_be.las`, **1 km** tiles, 1066 in total), point format 6, scale 0.01, **no CRS VLR** (only a RIEGL extra-bytes VLR), TerraScan; classes 2/3/4/5/7/0 (+32 seen); about 9.8 pts/m² (13.7 M points per km² in the sample); members are deflate (method 8) or **Deflate64 (method 9: Nordost, Südost)**; the large ZIPs are ZIP64 | flown 24/25 Feb and 2 Mar 2021 | `3dm_33_384_5820_1_be.las` in `Mitte.zip`: 221 763 261 B compressed / 522 133 625 B; average 234 MB / 525 MB per tile |
| Vegetation heights 2020 (Umweltatlas) | `https://gdi.berlin.de/data/ua_vegetationshoehen_2020/atom/0.atom` → `veghoehe_2020.zip` (title "785 MB", one file for the whole city, `image/tiff`) | GeoTIFF | 2020 | not downloaded |
| bDOM/DOM WMS | `https://gdi.berlin.de/services/wms/bdom` (caps 200) | images only | | |

No Berlin nDOM product was found (catalogue searches: "Oberflächenmodell", "Laserscan", "Punktwolke", "Geländemodell").
LGB's nDOM LAZ is 404 inside Berlin (section 2).

**Recommendation for the nDSM template:** LGB `bdom/tif/bdom_33EEE-NNNN.zip` (f32 GeoTIFF 0.2 m, `.tfw` 0.2 / E+0.1 /
N+1000−0.1, about 30 MB per km²; `bb_bdom_aktualitaet.csv` gives the flight date, e.g. `33392-5820;;2026-03-05;;2026-09-10;`)
minus LGB DGM1 (bilinear to 0.2 m), clamped at 0. This keeps 0.2 m like the Brandenburg nDOM and stays in the 1 km scheme.
Caveats: image matching smooths thin objects (the bDOM tile maximum is 302 m although the TV tower in the same tile is 368 m
tall; that is plausible for image matching, but the cause is **UNVERIFIED**), and moving traffic is in the DSM
("Der … ruhende und fließende Verkehr wird auch dargestellt", `dom.pdf`). Fallback: Berlin bDOM 1 m XYZ minus Berlin DGM1
(both 2 km, about 20 MB each), which is coarser. The ALS LAS is not practical (234 MB per km², Deflate64, no CRS VLR).

---

## 7. ALKIS buildings

### 7.1 Endpoints
- `https://gdi.berlin.de/services/wfs/alkis_gebaeude?service=WFS&request=GetCapabilities` → 200, 95 453 B, WFS 2.0.0,
  title "ALKIS Berlin Gebäude", Fees "… Datenlizenz Deutschland - Zero - Version 2.0 …", AccessConstraints
  "Es gelten keine Zugriffsbeschränkungen". One feature type **`alkis_gebaeude:gebaeude`** (title "Gebäude"),
  DefaultCRS `urn:ogc:def:crs:EPSG::25833` (others 25832, 4326, 3857, 4258).
  Capabilities advertise only `application/gml+xml; version=3.2`, but **`outputFormat=application/json` works**
  (also `json`; `application/geo+json` and `geojson` → 400). `ImplementsResultPaging TRUE`, `CountDefault 1000000`.
- The same building schema is also in `https://gdi.berlin.de/services/wfs/alkis` as `alkis:gebaeudeflaechen`.
  The combined ALKIS WFS lists: `bauwerkeflaechen`, `bauwerkelinien`, `besondereflurstuecksgrenzen`, `bezirk`, `flur`,
  `flurstuecke`, `gebaeudeflaechen`, `gebaeudelinien`, `gemarkung`, `gewaesservegetationflaechen`, `land`, `ortsteile`,
  `festlegungenflaechen`, `relief`, `tatsaechlichenutzungflaechen`, `gewaesservegetationlinien`, `vegetationpunkte`.
- Example (school area, Helene-Haeusler-Schule, Mendelssohnstraße 10/11, 10405 Berlin-Prenzlauer Berg, about E 392800 N 5820850):
  `https://gdi.berlin.de/services/wfs/alkis_gebaeude?service=WFS&version=2.0.0&request=GetFeature&typeNames=alkis_gebaeude:gebaeude&outputFormat=application/json&bbox=392000,5820000,393000,5821000,urn:ogc:def:crs:EPSG::25833`
  → 200 `application/json;charset=UTF-8`, 1 104 821 B, 2.2 s, 1111 features (356 `AX_Gebaeude`, 755 `AX_Bauteil`),
  all MultiPolygon, coordinates **E,N** in EPSG:25833 (`crs` member `urn:ogc:def:crs:EPSG::25833`).
  A 4×4 km box (390000,5818000,394000,5822000): 26 754 features, 24.8 MB, 24.7 s, uuids unique.
- Paging: with `count=400` the response carries `numberMatched` 1111, `numberReturned`, `totalFeatures`, `timeStamp`
  and `links` `rel=next` / `rel=previous` (absolute GeoServer URLs with `STARTINDEX=`). Three pages: 400+400+311,
  no duplicates. **freemapper's `fetch_collection` paging through `links[rel=next]` works unchanged**; without `count`
  the server returns everything in one response (no `links`).
- `CQL_FILTER` (GeoServer vendor parameter) works for simple equality (`bezeich='AX_Platz'`, citywide 1751 features in
  0.56 s), but a `BBOX(...) AND bezgfk LIKE '%Schule%'` request timed out (120 s). Filter locally.
- Outside Berlin the WFS returns 0 (Strausberg bbox 421400,5821500,421800,5821900 → `numberMatched` 0).

### 7.2 Schema (`DescribeFeatureType`) and mapping to freemapper's `Building`

Fields: `uuid` (string, required), `gfk`/`bezgfk` (building function code/text), `ofl`/`bezofl` (position relative to the
ground), `aog`/`aug` (storeys above/below ground), `hoh` ("true"/""), `bat`/`bezbat` (Bauart of parts), `nam` (name),
`baw`/`bezbaw` (Bauweise), `zus`/`bezzus` (condition), `gkn`, `des`/`bezdes` (data capture), `lag`, `namlag` (address), `hnr`,
`pnr`, `lnr`, `bezeich` (AAA class), `shape_area`, `geom` (MultiSurface). Feature id = `gebaeude.<uuid>`.

| freemapper (`alkis.rs`) | Brandenburg (LGB) | Berlin (`alkis_gebaeude:gebaeude`) |
|---|---|---|
| `oid` | `oid` (`DEBBAL…`) | **`uuid`** (`DEBE…`, e.g. `DEBE03YY600004AR`) |
| `kind` (`Gebäude`/`Bauteil`) | `gebnutzbez` | **`bezeich`**: `AX_Gebaeude` ↔ `Gebäude`, `AX_Bauteil` ↔ `Bauteil` |
| `function` | `funktion` | **`bezgfk`** (same AAA text, e.g. `Wohnhaus`, `Allgemein bildende Schule`; code `gfk` 1010, 3021 …). Every `AX_Gebaeude` has it; `AX_Bauteil` has null |
| `underground` | `rellage == "Unter der Erdoberfläche"` | **`bezofl == "Unter der Erdoberfläche"`** (`ofl` 1200; 787 of 26 754). Also `Aufgeständert` (1400, 3). Normal = `""` (empty string, not null) |
| `updated` | `aktualit` | **none**: no per-feature date in the building schema |

Typical values (4×4 km, Mitte/Prenzlauer Berg): `bezgfk` Wohnhaus 4162, Wohngebäude mit Handel und Dienstleistungen 1215,
Gebäude für Wirtschaft oder Gewerbe 752, Tiefgarage 411, Bürogebäude 348, Garage 251, Allgemein bildende Schule 125,
Hochschulgebäude 68, Berufsbildende Schule 5. `bezbat` of parts: Geringergeschossiger Gebäudeteil, Auskragender
Geschossteil / zurückspringendes Geschoss, Durchfahrt im Gebäude, Hochhausgebäudeteil, Keller, Loggia, Arkade, Schornstein im Gebäude.

Sample features (geometry shortened):
```json
{"type":"Feature","id":"gebaeude.DEBE03YY600004AR","geometry":{"type":"MultiPolygon","coordinates":"[[[[392781.192,5820824.079], ... ]]]"},
 "properties":{"uuid":"DEBE03YY600004AR","gfk":3021,"bezgfk":"Allgemein bildende Schule","ofl":null,"bezofl":"","aog":4,"aug":null,"hoh":"",
 "bat":null,"bezbat":null,"nam":"Helene-Haeusler-Schule Turnhalle","baw":null,"bezbaw":"","zus":null,"bezzus":"","gkn":"","des":"4200",
 "bezdes":"Aus Katasterkarten digitalisiert","lag":"41942","namlag":"Mendelssohnstraße 10","hnr":"10","pnr":null,"lnr":null,
 "bezeich":"AX_Gebaeude","shape_area":2245.86}}
{"type":"Feature","id":"gebaeude.DEBE03AL1Kw0000Q", ... "properties":{"uuid":"DEBE03AL1Kw0000Q","gfk":3021,"bezgfk":"Allgemein bildende Schule",
 "aog":3,"nam":"Filiale der Helene- Haeusler Schule (HausB)","namlag":"Mendelssohnstraße 11","bezeich":"AX_Gebaeude","shape_area":849.1, ...}}
{"type":"Feature","id":"gebaeude.DEBE00YY13U0001t", ... "properties":{"uuid":"DEBE00YY13U0001t","gfk":null,"bezgfk":null,"bezofl":"","aog":2,
 "bat":1100,"bezbat":"Geringergeschossiger Gebäudeteil","des":"4200","bezeich":"AX_Bauteil","shape_area":5421.52, ...}}
```

---

## 8. ALKIS land use ("Tatsächliche Nutzung")

- Endpoint: `https://gdi.berlin.de/services/wfs/alkis`, type **`alkis:tatsaechlichenutzungflaechen`**
  ("ALKIS Berlin Tatsächliche Nutzung"), same WFS behaviour as section 7 (JSON, paging, CRS).
  There is **no** `…/services/wfs/alkis_tatsaechlichenutzung` (404).
- Example: `https://gdi.berlin.de/services/wfs/alkis?service=WFS&version=2.0.0&request=GetFeature&typeNames=alkis:tatsaechlichenutzungflaechen&outputFormat=application/json&bbox=392700,5820750,392900,5820950,urn:ogc:def:crs:EPSG::25833`
  → 200, 29 499 B, 19 features. 1 km box → 366 features, 0.6 s; 4×4 km → 4700 features, 5.9 MB, 5.5 s.
- Schema: `objectid` (int; feature id `tatsaechlichenutzungflaechen.<objectid>`), `nam`, `fkt`/`bezfkt`, `zus`/`bezzus`,
  `veg`/`bezveg`, `agt`/`bezagt`, `lgt`/`bezlgt`, `hyd`/`bezhyd`, `art`/`bezart`, `bkt`/`bezbkt`, `bezeich` (AAA class),
  `uuid` (ALKIS id), `geom` (MultiSurface). **No date field.**
- **These are TN objects, not parcel parts.** Brandenburg's `nutzung_flurstueck` is land use cut by parcels (it has a
  `flstkennz`); Berlin gives the uncut TN polygons. A Berlin `Wohnbaufläche` can span several parcels, so the
  `OWN_LAND_SHARE` logic in `out_of_bounds.rs` behaves differently. To get parcel parts, intersect with
  `alkis:flurstuecke` (fields `uuid`, `bezeich`=`AX_Flurstueck`, `afl` area, `fsko` parcel id e.g. `11000101806044____`,
  `zae`, `nen`, `gmk`, `namgmk`, `fln`, `gdz`, `namgem`, `zde` date of origin, `dst`, `beg` lifespan start e.g. `2026-10-05`).

Mapping `bezeich` → Brandenburg `nutzart` (the LGB values, checked in a 5×5 km Strausberg box):

| Berlin `bezeich` | LGB `nutzart` | Berlin `bezfkt` values seen (citywide where marked) |
|---|---|---|
| `AX_Wohnbauflaeche` | Wohnbaufläche | none |
| `AX_IndustrieUndGewerbeflaeche` | Industrie- und Gewerbefläche | Handel und Dienstleistung, Gebäude- und Freifläche Industrie und Gewerbe, … Versorgungsanlage, … Entsorgungsanlage, Betriebsfläche Versorgungsanlage, Lagerfläche |
| `AX_FlaecheGemischterNutzung` | Fläche gemischter Nutzung | Gebäude- und Freifläche Mischnutzung mit Wohnen, … Land- und Forstwirtschaft |
| `AX_FlaecheBesondererFunktionalerPraegung` | Fläche besonderer funktionaler Prägung | citywide: Öffentliche Zwecke (5177), Historische Anlage (56), none (13). **No "Bildung und Wissenschaft"**: the school site is "Öffentliche Zwecke" |
| `AX_SportFreizeitUndErholungsflaeche` | Sport-, Freizeit- und Erholungsfläche | Grünanlage, Spielplatz/Bolzplatz, Erholungsfläche, Gebäude- und Freifläche Sport, Freizeit und Erholung, Sportanlage, Kleingarten, Freizeitanlage |
| `AX_Friedhof` | Friedhof | (Gebäude- und Freifläche Friedhof) |
| `AX_Strassenverkehr` | Straßenverkehr | citywide only: Gebäude- und Freifläche zu Verkehrsanlagen, Straße (2311; 1055) and Fußgängerzone (5130; 118). **No "Begleitfläche Straßenverkehr"** |
| `AX_Weg` | Weg | citywide: none |
| `AX_Platz` | Platz | citywide (1751): **always empty, so no "Parkplatz"/"Marktplatz"** |
| `AX_Bahnverkehr` | Bahnverkehr | Gebäude- und Freifläche zu Verkehrsanlagen, Schiene |
| `AX_Wald`, `AX_Gehoelz`, `AX_Landwirtschaft`, `AX_Moor` | Wald, Gehölz, Landwirtschaft, Moor | none in the samples (no Laub-/Nadelholz, Grünland …) |
| `AX_Fliessgewaesser`, `AX_StehendesGewaesser`, `AX_Hafenbecken` | Fließgewässer, Stehendes Gewässer, (Hafenbecken) | Fluß, Graben, Kanal / See, Teich |

Consequences for the rules: in Berlin every `AX_Platz` reaches `("Platz", None) => true` and becomes a map object
(in Brandenburg festival squares and similar are candidates through `bez`). Kerbside verges are not separated from roads,
so they are part of the `Straßenverkehr` area. Decide whether Berlin `Platz` should be a candidate, or check it against
the OSM surface.

Sample (school):
```json
{"type":"Feature","id":"tatsaechlichenutzungflaechen.50733","properties":{"objectid":50733,"nam":"","fkt":1100,"bezfkt":"Öffentliche Zwecke",
 "zus":null,"bezzus":"","veg":null,"bezveg":null,"agt":null,"bezagt":null,"lgt":null,"bezlgt":null,"hyd":null,"bezhyd":null,"art":null,
 "bezart":null,"bkt":null,"bezbkt":null,"bezeich":"AX_FlaecheBesondererFunktionalerPraegung","uuid":"DEBE03YY600026PO"}}
{"type":"Feature","id":"tatsaechlichenutzungflaechen.87632","properties":{"objectid":87632,"nam":"hinter Otto-Braun-Str. 77, 79 und hinter Mollstr. 33, 34, 35, 36",
 "fkt":null,"bezfkt":"","bezeich":"AX_Strassenverkehr","uuid":"DEBE03YY600026xk", ...}}
```
Other TN features in that box: AX_Wohnbauflaeche ×5, AX_FlaecheGemischterNutzung, AX_Friedhof "St. Marien-und St. Nikolai-Friedhof I",
AX_SportFreizeitUndErholungsflaeche/Grünanlage, AX_Strassenverkehr ×8.

Related: `alkis:gewaesservegetationflaechen` holds `AX_UntergeordnetesGewaesser` (e.g. "Barssee", `bezfkt` Teich) and
`AX_Vegetationsmerkmal` (`bezzus` "Nass") and could feed candidates later.

---

## 9. Data currency

| Product | Per tile/feature? | Mechanism (verified) |
|---|---|---|
| LGB tiles in Berlin (DOP, DGM, bDOM) | **yes** | Existing LGB CSVs (`bb_dop_…`, `bb_dgm_…`, `bb_bdom_aktualitaet.csv`) list the Berlin tiles with creation/revision/publication |
| Berlin DGM1/DOM/bDOM per-tile ZIPs | no list | ATOM `<updated>` exists only per dataset (dgm1 2025-12-18, bdom 2026-05-27, dom 2025-08-07); no per-tile `<entry>`. Per tile: HTTP `Last-Modified` + `ETag` + `Content-Length` (e.g. all DGM1 tiles 2026-01-23) and the ZIP member date (2025-12-16). `If-None-Match` and `If-Modified-Since` → **304** (verified). Survey dates are only in the metadata lineage, citywide |
| Berlin TrueDOP regional ZIPs | no list | One `Last-Modified`/`ETag` per region (9 values); member dates 2026-05-19; flight date only in the metadata ("Bildflug 05., 06. und 09. März 2026"). Repacking changes the ETag and all offsets (Südost 2026-06-23) |
| Berlin WMS | no | Use the dataset's flight date |
| ALKIS buildings / land use (Berlin WFS) | **no** | No `aktualit`. Dataset metadata: revision 2026-10-04, publication 2026-10-27, "monthly". The GeoJSON `timeStamp` gives the query time. Only `flurstuecke` carry dates (`zde`, `beg`) |
| OSM | per extract | `state.txt` |

Change detection for a Berlin adapter: keep (URL, ETag, Last-Modified, Content-Length) per tile and revalidate with a
conditional GET (cheap 304). Use the dataset/lineage date as the "currency" shown to the user (flight year), and
`Last-Modified` as "published" for cache naming, like `ndsm_path` does today.
For ALKIS, record the fetch time plus the metadata revision date, and treat a changed feature set (uuids, geometry hash)
as "changed upstream".

---

## 10. Licence and attribution

- Every Berlin dataset used here: **Datenlizenz Deutschland – Zero – Version 2.0** (`dl-de/zero-2-0`). Evidence: ATOM `<rights>`
  "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - Version 2.0 anzuwenden. Die Lizenz ist über
  https://www.govdata.de/dl-de/zero-2-0 abrufbar.", WMS/WFS `Fees` (same text), CKAN `license_id: dl-de-zero-2.0`, ISO
  `{"id":"dl-zero-de/2.0",…}`. The "Geobasisdaten online" terms (V 2.0, from 07.03.2025, §3.4) also state Zero for ALKIS,
  ATKIS and AFIS Berlin (excluding owner data and survey figures).
- Attribution is **optional**. Berlin's surveying offices: "Für die Nutzung der Daten des Landes Berlin ist die Datenlizenz
  Deutschland – Zero – Version 2.0 anzuwenden. Der Quellenvermerk für diese Daten ist somit optional und lautet
  'Geoportal Berlin / Karte 1:5000 von Berlin (K5)'" (https://www.berlin.de/vermessungsaemter/geoinformation/artikel.675359.php),
  i.e. the pattern **"Geoportal Berlin / [Titel des Datensatzes]"**.
- Suggested freemapper lines (Berlin areas):
  - ALKIS: `Geoportal Berlin / ALKIS Berlin Gebäude, dl-de/zero-2-0` and
    `Geoportal Berlin / ALKIS Berlin (Amtliches Liegenschaftskatasterinformationssystem), dl-de/zero-2-0`
  - Berlin rasters, if used directly: `Geoportal Berlin / Digitale farbige TrueOrthophotos 2026 (DOP20RGBI), dl-de/zero-2-0`, etc.
  - LGB tiles with Berlin content (required by LGB's own note): `© GeoBasis-DE/LGB <year>, dl-de/by-2-0; © Geoportal Berlin,
    dl-de/zero-2-0 (Daten geändert)`. The LGB bDOM note says `© Geoportal Berlin, dl-de/by-2-0` instead; writing
    `© Geoportal Berlin` either way satisfies both notes.
  - OSM unchanged: `© OpenStreetMap contributors, ODbL`.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/brandenburg.html`: title "Download OpenStreetMap for **Brandenburg (mit Berlin)**";
  `brandenburg-latest.osm.pbf` "File size: 283 MB", data up to 2026-11-06T21:21:21Z. `brandenburg.poly`: **one outer ring,
  no hole**; points in Alexanderplatz, Prenzlauer Berg, Spandau and Köpenick lie inside it. **So the Brandenburg extract
  contains Berlin, and freemapper's OSM path needs no change.**
- Berlin: `https://download.geofabrik.de/europe/germany/berlin.html` ("File size: 95 MB"),
  `https://download.geofabrik.de/europe/germany/berlin-latest.osm.pbf` → **307** to `berlin-261106.osm.pbf`
  (200, 100 024 979 B, Last-Modified 2026-11-07 02:22:01). State: `https://download.geofabrik.de/europe/germany/berlin-updates/state.txt`
  → `timestamp=2026-11-06T21\:21\:21Z`, `sequenceNumber=4956` (identical to Brandenburg's).
  Brandenburg's `latest` also answers 307, so the fetcher has to follow redirects (ureq does this by default; the
  effect on freemapper was **UNVERIFIED**).
- `berlin.poly` bbox: lon 13.08283–13.76225, lat 52.33446–52.6783.

---

## 12. Extent and overlap

- Berlin (ALKIS `alkis:land`, feature `land.11`, 9581 vertices), EPSG:25833:
  **min E 370 000.825, min N 5 799 520.660, max E 415 786.553, max N 5 837 259.270**.
  As 1 km LGB tiles: E 370–415, N 5799–5837.
- Berlin product grids: DGM1/DOM 2 km tiles 368 000–418 000 E × 5 798 000–5 838 000 N (297 tiles, about 250 m buffer
  into Brandenburg); TrueDOP 2026 368 000–416 000 × 5 798 000–5 838 000 (283 tiles, edge tiles clipped); bDOM 282 tiles;
  ALS 1066 1 km tiles. All aligned to even km, so they nest in the LGB 1 km grid (Berlin `EEE_NNNN` = LGB `33EEE-NNNN`,
  `33(EEE+1)-NNNN`, `33EEE-(NNNN+1)`, `33(EEE+1)-(NNNN+1)`).
- LGB rasters (DOP, DGM, bDOM) cover Berlin completely, so **raster tiles never need two adapters**.
- ALKIS is strictly split: Berlin WFS returns nothing in Brandenburg, LGB returns nothing in Berlin. Border test box
  382016,5807103,382616,5807703 (Lichterfelde/Teltow): Berlin 70 buildings (`DEBE00…`, `DEBE06…`), LGB 87 (`DEBBAL…`),
  no shared ids. **An area that crosses the border needs both ALKIS sources merged**. Choose the source per feature
  (or per bbox via the `alkis:land` polygon) and keep provenance per feature.
- nDOM: LGB has it only on border tiles, so inside Berlin use the bDOM−DGM path.

---

## 13. How a Berlin adapter differs from the LGB one (risks first)

1. **Silent empty ALKIS in Berlin today**: LGB returns 0 features without an error, and the project ends up without buildings.
   Detect it (bbox intersects `alkis:land`), or route by polygon.
2. **Attribute schema**: `uuid` instead of `oid`, `bezeich` `AX_*` class names instead of German text, `bezgfk`/`bezofl`
   instead of `funktion`/`rellage`, empty strings instead of nulls, **no `aktualit`**. Land use comes as TN objects, not
   parcel parts, with fewer refinements (no Parkplatz, no Begleitfläche, no Bildung und Wissenschaft).
3. **Protocol**: WFS 2.0 KVP (`typeNames`, `bbox=…,urn:ogc:def:crs:EPSG::25833`, `count`, `startIndex`) instead of OGC API
   (`bbox-crs`, `limit`). GeoJSON is undocumented but works, and the `links[rel=next]` paging is compatible.
4. **Server flakiness**: connection resets and connect timeouts, plus HTTP 200 responses with an XML exception body
   (WMS). Retry, check the Content-Type, and use a short connect timeout.
5. **Rasters, if fetched from Berlin directly**: 2 km tiles; DGM/DOM/bDOM as zipped XYZ text (120 MB per tile
   unzipped, CRLF, varying decimals, trailing spaces, row/column order that changes per tile, sparse edge tiles);
   TrueDOP only as 0.1–3.5 GB regional ZIPs of 80 MB single-tile lossy JP2 without world files (range reads are possible;
   the JP2 decode is the problem); ALS as uncompressed LAS 1.4 in ZIP64/Deflate64 without a CRS VLR. None of this is needed
   if the LGB tiles are used.
6. **Currency**: no per-tile CSV on the Berlin side; flight dates only per dataset; ALKIS without feature dates.
7. **Height datum**: DHHN2016 in both states (Berlin `dgm1.pdf`, `dom.pdf`); the LGB DGM in Berlin equals the Berlin DGM1.
   CRS EPSG:25833 everywhere; axis order E,N in all responses tested.
8. **Licence**: Berlin is dl-de/zero-2-0 (no attribution needed), but the LGB tiles that carry Berlin content ask for the
   extra "© Geoportal Berlin" note.
9. **Catalogue**: `daten.berlin.de` blocks curl (403); use the CKAN API at `datenregister.berlin.de` or the CSW.
10. No authentication and no API keys anywhere. No rate-limit headers were seen.

---

## 14. Not verified

- Whether LGB nDOM tiles on the border contain points in the Berlin part.
- The exact WMS layer names for `truedop_2025_sommer`, `dop_2025_fruehjahr` and `truedop_2024` (only `truedop_2026` was used).
- Pure-Rust decoding of the Berlin JP2 tiles.
- The reason for the 302 m bDOM maximum in tile 392_5820 (TV tower 368 m).
- The source of the LGB DOP tiles dated 2025-08-12 in Berlin (probably Berlin TrueDOP Sommer 2025).
- Whether `CQL_FILTER` with `LIKE` over large boxes ever finishes (it timed out once).
- Server-side limits above 8000×8000 px (WMS) and above 26 754 features per WFS response.

## 15. Local samples (all under `(scratch)/berlin/`)

`dop/dop20rgbi_33_400_5828_2_be_2026.jp2` (edge tile, 228 KB), `dop/mitte_head.jp2` (first 66 KB of a full tile),
`dgm/DGM1_392_5820.zip`, `dgm/edge/DGM1_416_5810.zip`, `bdom/392_5820.zip`, `als/head_946366555-946466554.bin` (LAS header bytes),
`wms/test2.jpg`, `wms/cmp_*.png`, `lgb_in_berlin/` (LGB DOP/DGM tiles `33392-5820`, `33406-5812`, bDOM `.tfw`/meta),
`alkis/*.json` (buildings 1 km and 4 km, TN, parcels, border test, `land.json`), `alkis/caps_*.xml`, `alkis/dft_*.xsd`,
`lists/` (central-directory listings of all TrueDOP 2026 and ALS regional ZIPs, tile lists), `meta/` (ATOM feeds,
CKAN JSON, CSW records, WMS capabilities, LGB currency CSVs), `osm/` (`*.poly`, Geofabrik pages), `docs/` (`dgm1.pdf`, bDOM `dom.pdf`).
Scripts: `(scratch)/scripts/remotezip.py` (remote ZIP list/extract by range), `jp2boxes.py`, `lashdr.py`, `utm.py`.
