# Lower Saxony (LGLN) data sources for freemapper — live-verified reference

Verified live on 2026-11-08 (Europe/Berlin), early morning (05:10–07:00 UTC). Provider: **Landesamt für Geoinformation und
Landesvermessung Niedersachsen (LGLN), Landesbetrieb Landesvermessung und Geobasisinformation**, Podbielskistraße 331,
Hannover (contact `geodatendienste@lgln.niedersachsen.de`). Open data ("OpenGeoData.NI") under **CC BY 4.0**. The data
come from three kinds of host:

- **STAC APIs** (stac-fastapi on uvicorn, one host per product): `dop.stac.…`, `dgm.stac.…`, `dom.stac.…`,
  `bdom.stac.lgln.niedersachsen.de` — the tile index with the flight/ALS date per tile and all historic vintages;
- **IBM Cloud Object Storage** buckets (`<bucket>.s3.eu-de.cloud-object-storage.appdomain.cloud`, server "Cleversafe"):
  `dop20-rgb`, `dop20-rgbi`, `dgm1`, `dom1`, `bdom20`, `arcgis-geojson` — every raster as a **Cloud-Optimized GeoTIFF**
  (HEAD, ranges, ETag), the buckets are even listable;
- **OGC services** behind the "doorman" gateway `opendata.lgln.niedersachsen.de/doorman/noauth/…` (DOP WMS = MapServer
  cascade, ALKIS WFS = AAA-suite XtraServer) and MapServer WCS on `opendata.geoservices.lgln.niedersachsen.de`
  (`dgm_wcs`, `dom_wcs`, `dop_wcs`).

No login, registration, token, cookie or form is needed for anything below (the gateway sets a session cookie
`AL_SESS-S`, but cookie-less requests work). Nothing returned 401/403. All TLS chains are complete (§12).

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tiles / 2 km DOP block |
|---|---|---|---|---|---|---|
| **Kaiser-Wilhelm- und Ratsgymnasium (KWR)**, Seelhorststraße 52, Hannover-Südstadt (primary) | way **42796993** | E 551931 / N 5803326 | E 143631 / N 5815968 | 52.37773 / 9.76290 | E 551818–552044, N 5803230–5803421 | **`32 551 5803` + `32 552 5803`** (the footprint crosses E 552000) / 2 km blocks `550_5802` + `552_5802` |
| Hamburg border probe, Seevetal-Meckelfeld / Hamburg-Harburg | – | tile centre E 567500 / N 5920500 | E 168965 / N 5931595 | 53.42922 / 10.01592 | – | `32 567 5920` (55.1 % NI, rest Hamburg) |
| Bremen border probe, Stuhr-Brinkum / Bremen-Huchting & airport | – | tile centre E 485500 / N 5876500 | E 83417 / N 5894566 | 53.03782 / 8.78375 | – | `32 485 5876` (56.0 % NI); ALKIS also in `32 482 5876` (53.7 % NI) |
| ALKIS value samples only: KWR 2 × 2 km (E 550931–552931, N 5802326–5804326), Braunschweig centre 1.5 × 1.5 km (centre E 603803 N 5791325), Göttingen centre 1.5 km (E 564894 N 5709581), Osnabrück centre 1.5 km (E 435166 N 5791783), Harz 2 km (E 596536 N 5739721), Lüneburger Heide 2 km (E 562174 N 5891030), Cuxhaven 2 km (E 479943 N 5968545) | – | | | | | |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7°. The ALKIS school building
`DENIAL4300029q7VBL` ("Kaiser-Wilhelm- und Ratsgymnasium", "Allgemein bildende Schule") lies at E 551839–552032 / N
5803254–5803387 in 25832, inside the OSM footprint; the sports hall `DENIAL430001t2toBL` at E 552019–552077. The WFS's 25833
output of the school area matches the expected rotated bbox; the state polygon in 25832 and 25833 was fetched separately.)

Samples (all under `ni/` next to this file; treat them as untrusted data; scripts in `ni/scripts/`):

| Path | Content |
|---|---|
| `dl/dop20_2km/dop20rgb_32_550_5802_2_ni_2025-03-06.tif` | full 2 km DOP20 RGB COG of the school block (28.05 MB, flight 06.03.2025) |
| `dl/cogwin/` | **HTTP-range window reads** at the school: DOP10 (0.1 m), DOP20 2 km, DOP10 overview level 1, DOP10 RGBI overview (`kwr_*.png`), preview `kwr_dop20_small.jpg` |
| `dl/cog_hdr/` | the first 94 KB (all IFDs + tile tables) of the DOP20 2 km COG |
| `dl/dgm1/`, `dl/dom1/` | DGM1/DOM1 COGs + XML metadata of tiles `551/5803` and `552/5803` (ALS 18.04.2016) |
| `dl/bdom/` | bDOM20 window (range read) at the school as `.npy` |
| `dl/border/` | Hamburg-border DGM1/DOM1 `567/5920` (ALS 2025), Bremen-border DGM1/DOM1 `485/5876` (ALS 2017), DOP overviews `hh_dop2026_ovr.png`, `hb_dop2024_ovr.png`, `border_pair.jpg`, mask images |
| `dl/rand/` | a DGM1 "Randkachel" (partial Los-edge tile, 99.2 % NoData) |
| `dl/wms/` | DOP WMS: school 25832 JPEG/PNG and 25833, three 5000² px JPEGs (timing), 15000/15001 px probes, Bremen/Hamburg border images (`borders_wms.jpg`) |
| `dl/wmsvintage/` | WMS vs 2026 1 km COG vs 2024 2 km COG at the Hamburg probe (which vintage the WMS serves) |
| `dl/wcs/` | DGM WCS clips (200 m, 2 km) |
| `dl/index/bdom20.geojson` | the 73.6 MB bDOM20 download index of the hub (one feature per tile) |
| `probe/stac/` | landing pages, collections, searches, and the **complete item lists** `all_dgm1.jsonl` (70,285), `all_dom1.jsonl` (70,289), `all_dop.jsonl` (100,106), `all_bdom.jsonl` (137,607) + `newest_*.json` (newest item per 1 km cell) |
| `probe/cog/` | remote COG structure dumps (`cog1.txt`, `cog2.txt`, `cog_bdom.txt`) |
| `probe/caps/` | capabilities: DOP WMS, ALKIS WFS, DGM/DOM/DOP WCS, DGM DescribeCoverage |
| `probe/gfi/` | DOP GetFeatureInfo answers (plain, HTML, GML; error variants) |
| `probe/alkis/` | WFS answers: school (buildings, NutzungFlurstueck, Nutzung, 25833, hits, paging, errors, 5 km limits), `areas/` (seven sample areas × three types, summaries `sum_*.txt`, dumps `dump_*.jsonl`) |
| `probe/border/` | STAC answers for both border tiles, ALKIS buildings/land use at the Hamburg and Bremen borders |
| `probe/boundary/` | `r1/land_25833.xml`, `r1/land_25832.xml` (state polygon DE03), `.wkt` versions, **`niedersachsen_simplified25m.geojson`** (copied to `ni/niedersachsen.geojson`) |
| `probe/s3/` | bucket listings (`?delimiter=/`) |
| `probe/tls/` | `openssl s_client -showcerts` per host |
| `probe/licence/agnb.html`, `probe/docs/` | LGLN AGNB page (licence §7), newsletters "LGLN Aktuell" 1/2025 and 4/2026 (+ text) |
| `probe/portal/` | OpenGeoData.NI page and its `main.js` (service list, old licence text) |
| `probe/geofabrik/`, `probe/osm/` | Geofabrik state/poly; Overpass answers (school list, KWR way) |

---

## 1. Brandenburg → Lower Saxony at a glance

| freemapper product | Brandenburg (today) | Lower Saxony equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** NI spans 6.65–11.60°E; in 25833 it reaches E −63.5 km (12.1 % of the area at negative eastings) with −4.2° grid convergence and +1.2 m/km scale in Hannover (−6.7° / +3.4 m/km on Borkum). WMS and WFS reproject to 25833 on request. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **COG tiles found via STAC**, JPEG-in-TIFF (YCbCr 4:2:0, q75), 128 px internal tiles, 7 overviews → a map window can be read with **a few HTTP ranges** (school 400 × 300 m at 0.2 m: 15 requests, 0.59 MB, 4.5 s). Two tile schemes: **2 km** `dop20…_2_ni_<date>` (2012–2025) and **1 km** `dop20/dop10…_1_ni_<date>` (2024–2026); **all historic vintages** are kept (up to 9 per 2 km block). Newest per 1 km cell: 2026 37 %, 2025 53 %, 2024 10 %; **75 % leaf-off (Feb–Apr), 25 % leaf-on**. School: **06.03.2025** (leaf-off). Alternative: **WMS** `ni_dop20` (max **15000** px, ≈13.6 s per km² at 0.2 m, the same pixels). | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1 COG** Float32, LZW (no predictor), 512 px tiles + 1 overview, NoData −9999, 1 km, 3.5–4.1 MB, **no TFW**. ALS **2010–2025**, newest per tile mostly **2016/2017** (55 %), 2025 for 15 % (coast, Elbe, Weser). URL contains the ALS lot (`L1603/Vollkacheln/…`) → must come from STAC. Also WCS `ni_dgm1` (byte-identical, no size limit found up to 5 km). | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | Two ready options: **DOM1 − DGM1** (ALS, same lot/date/grid/format; **median 8–9 years older than the DOP**) or **bDOM20 − DGM1** (image-matching DSM, 0.2 m Float32 COG, **same date as the DOP on 96.7 % of tiles**, but leaf-off crowns are missing and the DGM is old). No nDOM, no open LAZ found. | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `alkis_wfs_einfach`, `ave:GebaeudeBauwerk` (AAA-suite XtraServer like Hesse/NRW/SH, GML 3.2 only, no GeoJSON, no OGC API found). LGB spellings, detailed `funktion` ("Allgemein bildende Schule"), `rellage`. **`COUNT` ≤ 10000 (more → 400), and without `COUNT` the answer is silently cut at 10000.** **Underground garages are `Gebäude` + `rellage "Unter der Erdoberfläche"`**, never `Bauteil`. | verified |
| Land use | OGC API `nutzung_flurstueck` | WFS **`ave:NutzungFlurstueck`** (parcel split) and `ave:Nutzung` (**has `nutzart`**). LGB spellings. **City-centre blocks are `Industrie- und Gewerbefläche` / "Handel und Dienstleistung"** (4,582 of 20,582 parcels), `Fläche gemischter Nutzung` is almost absent (2). `Straßenverkehr` "Begleitfläche" on 19 %, `Platz` without `bez` 55 %. School parcel "Bildung und Wissenschaft". | verified |
| Currency | `bb_*_aktualitaet.csv` | **STAC item `datetime`** per tile and vintage (DOP flight date, DGM/DOM ALS date + `los`, bDOM date), item XML metadata next to every COG, DOP WMS GetFeatureInfo (`BF_DATUM`), ETag/Last-Modified on every object, `aktualit` per ALKIS feature | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `niedersachsen-latest.osm.pbf` = **509.9 MB**; its poly **includes Bremen and Bremerhaven** (and Neuwerk), excludes Hamburg | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **CC BY 4.0**. AGNB §7.2 (Stand 01.10.2024): **"© GeoBasis-DE/LGLN Jahr, Daten geändert"** (year = download year); services' AccessConstraints give the shorter example "LGLN (2024) Creative Commons Namensnennung – 4.0 International (CC BY 4.0)". | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **STAC APIs** | `https://dop.stac.lgln.niedersachsen.de` (collection `DOP`), `https://dgm.stac.lgln.niedersachsen.de` (`dgm1`), `https://dom.stac.lgln.niedersachsen.de` (`dom1`), `https://bdom.stac.lgln.niedersachsen.de` (`BDOM`) | tile index, all vintages, asset hrefs | STAC 1.0.0, stac-fastapi (`server: uvicorn`, HTTP/2). Conformance: core, item-search, **fields, sort, query, filter (CQL2 text/JSON)**, transaction (advertised). **`limit` silently capped at 1000**; paging via `next` link; **no `numberMatched`/`context`**. 0.2–0.3 s per search (first call after idle 3–4 s). Licence `CC-BY-4.0` in collection and items. No `ndom`/`laz`/`las` STAC (404). |
| **COG buckets** | `https://{dop20-rgb,dop20-rgbi,dgm1,dom1,bdom20}.s3.eu-de.cloud-object-storage.appdomain.cloud/…` | the rasters + per-tile XML metadata (+ `.md5`, empty `.json`) | IBM COS ("Cleversafe", S3 API 2.5). HEAD with Content-Length, **`Accept-Ranges: bytes`**, ETag, Last-Modified, `x-amz-version-id`; Range → 206; suffix range (`bytes=-1000`) → 206; beyond EOF → **416** XML `InvalidRange`; **multi-range → 206 with only the first range** (not multipart); missing key → **404** XML `NoSuchKey`. **Buckets are publicly listable** (`?delimiter=/&prefix=…`, S3 ListBucketResult). Content-Type varies (`image/tiff` or `application/octet-stream`). |
| Index GeoJSONs (hub) | `https://arcgis-geojson.s3.eu-de.cloud-object-storage.appdomain.cloud/{dop20,bdom20,dgm1,dom1}/lgln-opengeodata-<p>.geojson` | statewide newest-tile index (EPSG:25832, one feature per 1 km tile: `Aktualitaet`, asset URLs, `tile_id`) | 83.2 / 73.6 / 36.2 / 36.0 MB (Last-Modified 2026-09-03 … 2026-11-01). The STAC is the better index (smaller answers, all vintages). |
| **DOP WMS** | `https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms` | `ni_dop20` (RGB), `ni_dop20_info` (queryable tile info) | "WMS NI DOP20", AdV-WMS-Profil; UMN MapServer cascade behind the gateway (`x_2_lb: … balancer://prod_umn`); **MaxWidth/MaxHeight 15000** |
| **ALKIS WFS** | `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach` | `ave:Flurstueck`, `ave:FlurstueckPunkt`, `ave:GebaeudeBauwerk`, `ave:KatasterBezirk`, `ave:Nutzung`, `ave:NutzungFlurstueck`, `ave:VerwaltungsEinheit` | "WFS NI ALKIS Vereinfacht", WFS 2.0.0/1.1.0, AAA-suite XtraServer (`Content-Disposition: inline; filename="XtraServerGetFeature.xml"`, `balancer://prod_xtraserver`) |
| **WCS** | `https://opendata.geoservices.lgln.niedersachsen.de/{dgm_wcs,dom_wcs,dop_wcs}` | coverages `ni_dgm1`, `ni_dom1`, `ni_dop` | WCS 2.0.1, MapServer. Unknown service names → **HTTP 200 HTML** "MapServer Message … CGI variable "map" not found". Also a rendered `dgm_wms` (`ni_dgm1_farbe`, `ni_dgm1_grau`, `ni_dgm1_info`). |
| Portals | `https://opengeodata.lgln.niedersachsen.de/` (Angular app, old), `https://ni-lgln-opengeodata.hub.arcgis.com` (ArcGIS Hub, named in the AGNB) | product pages, downloads | the old portal's `main.js` still carries a dl-de/by-2-0 licence text (§9); not used by freemapper |
| ISO metadata | `https://ni-harvest-prod.geocat.live/catalogue/srv/ger/csw?…&id=<uuid>` (from WMS caps), `https://geoportal.geodaten.niedersachsen.de/harvest/srv/ger/catalog.search` | | not examined |

Not used / not examined: the ArcGIS Hub app itself, ALKIS Landnutzung GeoPackage, LoD1/LoD2, Basis-DLM, the WCS `ni_dop`,
`single-datasets.opengeodata.lgln.niedersachsen.de` (ALKIS/ATKIS Verwaltungsgrenzen zips of 2024, 97.7/41.7 MB).

### 2.1 Tile scheme

- **EPSG:25832, SW corner at whole km.** Products and names (the **date or year is part of the name**, so names come from
  STAC, never from arithmetic):

  | Product | Item id = file stem | Tile | Asset href pattern |
  |---|---|---|---|
  | DOP 2 km (2012–2025) | `dop20rgbi_32_EEE_NNNN_2_ni_YYYY-MM-DD` (EEE, NNNN even) | 2 × 2 km, 10000 px at 0.2 m | `https://dop20-rgb.s3…/32EEENNNN/YYYY-MM-DD/dop20rgb_32_EEE_NNNN_2_ni_YYYY-MM-DD.tif` (+ `dop20-rgbi` bucket `…dop20rgbi_…`) |
  | DOP 1 km 20 cm (2024–2026) | `dop20rgbi_32_EEE_NNNN_1_ni_YYYY-MM-DD` | 1 km, 5000 px | same pattern, `_1_` |
  | DOP 1 km 10 cm (2024–2026, cities/blocks) | `dop10rgbi_32_EEE_NNNN_1_ni_YYYY-MM-DD` | 1 km, 10000 px at 0.1 m | `…/dop10rgb_32_EEE_NNNN_1_ni_…tif` **in the `dop20-rgb` bucket** |
  | DGM1 / DOM1 | `dgm1_32_EEE_NNNN_1_ni_YYYY` / `dom1_…` | 1 km, 1000 px | `https://dgm1.s3…/<Los>/Vollkacheln/dgm1_32_EEE_NNNN_1_ni_YYYY.tif` (Los e.g. `L1603`, `L2502`, `L1503-1`, `L2203_Wattbereiche_Weser`) |
  | bDOM 20 / 10 cm | `bdom20_32_EEE_NNNN_1_ni_YYYY-MM-DD` / `bdom10_…` | 1 km, 5000 / 10000 px | `https://bdom20.s3…/32EEENNNN/YYYY-MM-DD/bdom20_…tif` |

  The 9-digit folder is `32` + EEE + NNNN (e.g. `325505802`). Every TIFF has a sibling `.xml` (metadata) and `.tif.md5`.
- Item counts and grid (from paging all four STACs, §7):

  | Collection | Items | Distinct tiles | Vintages per tile |
  |---|---|---|---|
  | `dgm1` | 70,285 | 49,708 (E 342…674, N 5682…5971) | 1: 29,497, 2: 19,845, 3: 366 |
  | `dom1` | 70,289 | 49,708 | same |
  | `DOP` | 100,106 (2 km: 64,462; 1 km 20 cm: 26,186; 1 km 10 cm: 9,458) | 13,133 2 km blocks + 23,883 1 km tiles = 52,641 1 km cells | 2 km: mostly 5 (10,729 blocks), up to 9 |
  | `BDOM` | 137,607 (20 cm: 126,740; 10 cm: 10,867) | 52,611 | 2021–2026 |

- **Tiles are complete squares, not clipped to the state:** the Hamburg-border DGM1/DOM1 `567/5920` (44.9 % Hamburg) and
  the Bremen-border `485/5876` (44 % Bremen) have **0 % NoData**; the DOP COGs show Harburg and Bremen airport
  (`dl/border/border_pair.jpg`). Some DOP COGs (2024 2 km blocks, 2026 1 km tiles) carry a 1-bit mask (deflate, NewSubfileType 4/5): **100 % valid**
  in all three tiles decoded. Tiles that lie entirely outside NI do not exist (Bremen Marktplatz, Bremerhaven, Hamburg
  Rathaus, Bielefeld, Kassel: no item), except a few ALS edge files under `…/Nicht_NI/` (e.g. `L1343/Nicht_NI/dgm1_32_551_5929_1_ni_2013.tif`,
  not in STAC).
- **Lots overlap:** `…/<Los>/Randkacheln/` holds partial tiles where a lot ends (e.g. `L1603/Randkacheln/dgm1_32_515_5812_1_ni_2016.tif`,
  99.2 % NoData, 55 KB); the STAC item with the same id points to the full tile in the neighbouring lot
  (`L1602/Vollkacheln/dgm1_32_515_5812_1_ni_2016.tif`). **Never build DGM/DOM URLs from the tile key; take the STAC href.**

### 2.2 Finding tiles: the STAC search

```
GET https://dgm.stac.lgln.niedersachsen.de/search?bbox=9.7615,52.3770,9.7645,52.3785&limit=50
→ 200 application/geo+json, 3,140 B, 0.2–3.2 s
{"type":"FeatureCollection","features":[
  {"id":"dgm1_32_552_5803_1_ni_2016","collection":"dgm1","bbox":[9.7638608,52.3747005,9.7787071,52.3837859],
   "properties":{"los":"L1603","datetime":"2016-04-18T00:00:00Z","genauigkeit":"0.3"},
   "assets":{"dgm1-tif":{"href":"https://dgm1.s3.eu-de.cloud-object-storage.appdomain.cloud/L1603/Vollkacheln/dgm1_32_552_5803_1_ni_2016.tif",
                         "type":"image/tiff; application=geotiff; profile=cloud-optimized","roles":["data"]},
             "dgm1-metadata":{"href":"…/dgm1_32_552_5803_1_ni_2016.xml","type":"text/plain","roles":["metadata"]}}, …},
  {"id":"dgm1_32_551_5803_1_ni_2016", …}],
 "links":[{"rel":"root",…},{"rel":"self",…}], "numberReturned":2}
```

- `bbox` is **lon/lat (WGS84)**, not 25832 (convert the map bbox; the geometry is the tile square in lon/lat).
  `intersects` (POST, GeoJSON in 4326) works too.
- Property and asset keys per collection:
  - **DOP**: `datetime` (flight date), `kachelname`, `bildflugname` ("Hannover"), `bildflugnummer`, `camera`,
    `bodenpixelgroesse` ("20"/"10"), `bezugsflaeche` ("bDOM" everywhere = TrueDOP), `license`. Assets **`dop20_rgb`**,
    `dop20_rgbi`, `preview` (= the RGB COG), `metadata_rgb`, `metadata_rgbi`.
  - **dgm1**: `datetime` (ALS date), `los`, `genauigkeit` ("0.3"). Assets **`dgm1-tif`**, `dgm1-metadata`.
  - **dom1**: `datetime`, `los`, `genauigkeit` ("0.5"), `erfassungsmethode` ("5020"). Assets **`dom1-tif`**, `dom1-metadata`.
  - **BDOM**: `datetime`, `aufloesung` ("20"/"10"), `kachelname`, `bildflugname`, `bildflugnummer`, `camera`. Assets
    **`bdom20`** (also for the 10 cm items), `metadata`.
- **A DOP search returns every vintage** (12 items for the school bbox: 2 km blocks 2013, 2016, 2019, 2022, 2025 + two
  1 km DOP10 2025). Pick the newest by `datetime`; at equal dates prefer the wanted resolution. `sortby=-properties.datetime`
  and `datetime=2024-01-01T00:00:00Z/..` work; CQL2 text works (`filter-lang=cql2-text&filter=bodenpixelgroesse='20'`).
- **Paging:** `limit` > 1000 is silently reduced to 1000; the `next` link carries an opaque cursor parameter
  (`…&limit=1000&<cursor>=next:dgm1:<last id>`). The fields extension shrinks pages ≈12× (`fields=id,properties.datetime,
  -geometry,-assets,-links`: 120 KB instead of 1.44 MB per 1000 items). Whole collections: dgm1 71 pages in 132 s,
  DOP 101 pages in 210 s, BDOM 138 pages in 217 s (≈1.2 s per page).
- Errors: unknown item → **404** `{"code":"NotFoundError","description":"Item … does not exist."}`; bad bbox → **400**
  `{"detail":"invalid bbox: foo"}`; a bbox outside NI → 200 with an empty list.
- The hub's index GeoJSONs (`arcgis-geojson` bucket, §2) list only the newest tile per cell and are 36–83 MB.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - COGs: GeoKeys `ProjectedCSType 25832`, GeoAscii "ETRS89 / UTM zone 32N|ETRS89|", `GTRasterType 1` (PixelIsArea),
    ModelTiepoint at the **NW corner** of the tile (e.g. `551000, 5804000`), ModelPixelScale 1 / 0.2 / 0.1.
  - Item XML: DGM/DOM `KOORDINATENREFERENZSYSTEM_LAGE ETRS89_UTM32`, DOP/bDOM `25832`.
  - WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`; WCS native 25832; WMS layer `ni_dop20` lists only 25832 (the root
    layer adds 25833, 4258, 4326, 3857, 3034, 3035, 3044, 3045, 31466–31469, 32632, 32633, 4647).
- **Height datum DHHN2016** normal heights, quasigeoid GCG2016: DGM/DOM XML `DE_DHHN2016_NH`, `DE_AdV_GCG2016_QGH`;
  DOP/bDOM XML `koordinatenreferenzsystem_hoehe 7837`. No vertical GeoKey. Same as the other freemapper states.
- **Zone choice:** all data are in zone 32 and NI lies at 6.65–11.60°E (central meridian 9°E):

  | Place | Convergence zone 32 / 33 | Scale k zone 32 / 33 | 25833 easting |
  |---|---|---|---|
  | Hannover (school) | +0.60° / −4.15° | 0.99963 / 1.00116 | +143.6 km |
  | Osnabrück | −0.75° / −5.51° | 0.99965 / 1.00236 | +26.1 km |
  | Emden / Borkum (west edge) | −1.88° / −6.72° | 0.99989 / 1.00333 | −51.0 … −54.5 km |
  | Wolfsburg | +1.41° / −3.35° | 0.99978 / 1.00061 | +213.1 km |
  | Schnackenburg (east edge, 11.6°E) | +2.08° / −2.72° | 0.99997 / 1.00024 | +272.1 km |

  **NI maps must be 25832 projects** (`Region::crs() → Crs::Utm32`, like Saxony-Anhalt, Hamburg, Hesse, SH, NRW).
  12.1 % of the state area has negative 25833 eastings (the asset reaches E −63,523), so no 25833 `TileId` (`u32`) may
  ever be built for NI.
- **Extent** of the state polygon (`VerwaltungsEinheit` DE03, `aktualit 2026-11-06Z`, 62,958 vertices; includes the
  Wadden Sea / coastal waters and the East Frisian islands in one polygon):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 340071 – 674215, N 5682893 – 5986464 |
  | EPSG:25833 | E −63523 – 271928, N 5695828 – 6006482 |
  | lon / lat | ≈ lon 6.65 – 11.60, lat 51.30 – 54.01 |

  Area 51,201 km² in 25832 (official land area 47,710 km²; the rest is coastal water), 51,401 km² in 25833.

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- Product: **TrueDOP** ("Bezugsfläche bDOM") RGBI, 20 cm statewide, 10 cm for city cooperations and flight blocks
  ("Block100…600", "Hannover", "Göttingen", "Osnabrück", "LK Oldenburg", "Buchholz" …). Item XML `komprimierung "JPEG 75%"`.
- Flight programme (LGLN Aktuell 1/2025): **from 2025 a 2-year cycle** (≈30,000 km² per year), spring flights; 2025 the
  southern half incl. Göttingen, Hannover, Langenhagen, Osnabrück; the coast is flown yearly with NLWKN.
- Newest DOP per 1 km cell (52,641 cells; from `all_dop.jsonl`):

  | Newest flight | Cells |
  |---|---|
  | 2026-03 / 04 / 05 | 12,249 / 2,124 / 5,171 |
  | 2025-02 / 03 / 04 | 256 / 17,377 / 7,198 |
  | 2025-09 / 10 | 1,460 / 1,698 |
  | 2024-04 … 12 | 5,005 (mostly May, Aug, Sep) |
  | 2015–2023 | 103 (edge cells) |

  → **74.7 % leaf-off (Feb–Apr), 24.9 % leaf-on (May–Oct)**, a patchwork by region and year. School: **06.03.2025**
  (flight 3817 "Hannover", UltraCam Eagle), leaf-off, true orthophoto (`dl/cogwin/kwr_dop20_small.jpg`).
- Historic vintages stay online in STAC and in the buckets (2 km blocks back to 2012).
- **Scheme change:** 2012–2025 2 km blocks (`_2_`), from 2024 also 1 km tiles (`_1_`); 2026 only 1 km. In 2025 both exist
  for the same flight (the school: `dop20rgbi_32_550_5802_2_ni_2025-03-06` and `dop10rgbi_32_551_5803_1_ni_2025-03-06`).

### 4.2 COG tiles and window reads (recommended path)

- Structure (verified on five DOP COGs; GDAL COG layout `LAYOUT=IFDS_BEFORE_DATA`, `BLOCK_ORDER=ROW_MAJOR`,
  `BLOCK_LEADER=SIZE_AS_UINT4`, little-endian classic TIFF, first IFD at 192/226):

  | File | Size | IFD 0 | Compression | Internal tiles | Overviews |
  |---|---|---|---|---|---|
  | `dop20rgb_32_550_5802_2_ni_2025-03-06.tif` (2 km RGB) | 28,053,344 B | 10000², 3 × 8 bit | **7 (JPEG)**, Photometric **6 (YCbCr)**, subsampling 2×2, `JPEGTables` 142 B (quant tables only) | **128 × 128** (6,241) | 7 (5000 … 79), no mask (the 2024 blocks add 8 mask IFDs) |
  | `dop10rgb_32_551_5803_1_ni_2025-03-06.tif` (1 km 10 cm RGB) | 22,957,707 B | 10000², 0.1 m | same | 128 × 128 | 7 |
  | `dop20rgb_32_567_5920_1_ni_2026-05-02.tif` (1 km 20 cm RGB) | 4,898,208 B (typical 5.47 MB) | 5000² | same | 128 × 128 (1,600) | 6 + 6 mask IFDs |
  | `dop10rgbi_32_551_5803_1_ni_2025-03-06.tif` (RGBI bucket) | 565,192,710 B | 10000², 4 × 8 bit | **1 (none)**, RGB + ExtraSamples 0 | 512 × 512, 1 MB each | 5 |
  | `dop20rgbi_32_603_5903_1_ni_2026-05-23.tif` 1 km RGBI | 145,755,780 B | (not inspected; size = 5000² × 4 B uncompressed + overviews) | | | |

  Use the **`dop20_rgb` asset** (JPEG, 5–28 MB) — the RGBI asset is uncompressed (146–565 MB).
- **Reading a window by HTTP Range** (`scripts/cogwindow.py`, verified):
  1. `Range: bytes=0-16383` → IFD chain (all IFDs sit in the first 4.4 KB). The **tile-offset/byte-count tables** of a 2 km
     COG run to byte ≈70.7 KB (IFD 0: offsets at 3690, counts at 28654, 6,241 × 4 B each), of a 1 km 20 cm COG to
     ≈36.8 KB. So one `bytes=0-98303` request gets everything needed.
  2. Pick the IFD by resolution: IFD 0 full, IFD 1 = ½ (0.2 m from a 10 cm tile) …; skip mask IFDs (NewSubfileType 4/5,
     1 bit, deflate). Geotransform from IFD 0 (`ModelTiepoint` NW corner, `ModelPixelScale`), overview pixel size =
     scale × (W0 / Wk).
  3. Compute the tile columns/rows of the window; tiles of one row are contiguous in the file (row-major), so **one range
     per tile row**: `bytes=offset[first]-offset[last]+count[last]-1`.
  4. Decode each tile: `JPEGTables` without its EOI + tile data without its SOI → a standard JPEG (YCbCr); crop the mosaic.
  - School window E 551730–552000, N 5803175–5803475 (1350 × 1500 px at 0.2 m):
    - DOP20 2 km IFD 0: 156 tiles, **15 requests, 592,412 B, 4.46 s** (sequential);
    - DOP10 1 km IFD 1 (0.2 m overview): 156 tiles, 15 requests, 455,875 B, 3.75 s;
    - DOP10 1 km IFD 0 (0.1 m, 2700 × 3000 px): 528 tiles, 26 requests, 1.46 MB, 9.2 s;
    - RGBI 1 km IFD 1 (uncompressed): 12 tiles, 5 requests, **12.6 MB**, 12.2 s (avoid).
    - Per range request ≈0.25–0.4 s latency (sequential); parallel requests would cut this.
  - Full download of the 2 km RGB COG: 28.05 MB in 24.3 s (1.15 MB/s, the line speed).
- **freemapper's `tiff` crate (0.11.3, default features incl. `jpeg` via zune-jpeg) can decode these tiles** (source read):
  `CompressionMethod::ModernJPEG` with `JPEGTables` is supported, chroma subsampling is accepted for JPEG, and tiles are read
  with `Decoder::seek_to_image(ifd)` + `read_chunk(index)`. Two traps:
  - **The crate does not convert YCbCr to RGB** (it sets zune-jpeg's output colour space to the input colour space and
    reports `ColorType::YCbCr(8)`); freemapper must apply the JFIF/BT.601 full-range conversion itself.
  - `Decoder` needs `Read + Seek`; for range reads use a small reader that fetches the header range once and then serves
    tile ranges (freemapper already has `get_range_with_retries` / `ByteRange` in `fetch.rs`), or fetch the tile-row ranges
    and feed them via a sparse buffer. Edge tiles are padded (10000 = 78 × 128 + 16).
  - Not run in Rust; Pillow decoded every tile this way.
- Pixel check: the WMS PNG and the COG window agree (mean abs difference 4.6 of 255 at offset 0, 8–10 at ±1 px; mean
  luminance 106.24 vs 106.24) → same pixels, JPEG noise. The DOP10 0.2 m overview vs the DOP20 2 km tile: 5.3 (same flight,
  different processing path).

### 4.3 WMS (fallback)

- `https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms`, WMS 1.3.0, "WMS NI DOP20".
  - Layers: **`ni_dop20`** ("DOP20", cascaded, opaque, only 25832 listed on the layer), `ni_dop20_info` ("DOP20 Info",
    queryable). Root layer `WMS_NI_DOP20` lists 16 CRS incl. 25833. No TIME dimension.
  - Formats: image/jpeg, image/png, image/tiff, png 8 bit, vnd.jpeg-png …
  - **MaxWidth/MaxHeight 15000.** 15001 px → **HTTP 200 `application/vnd.ogc.se_xml`**
    `<ServiceException code="InvalidDimensionValue" locator="width">Parameter "width" zu groß: 15001 Limit=15000</ServiceException>`.
- Example (school, 25832, 0.2 m):
  ```
  https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&STYLES=&LAYERS=ni_dop20&CRS=EPSG:25832&BBOX=551730,5803175,552000,5803475&WIDTH=1350&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 414,487 B, 1.99 s (PNG 5.26 MB, 6.4 s). 25833 (`BBOX=143431,5815818,143831,5816118`, 2000 × 1500):
  513,135 B, 2.8 s.
- Speed: 5000 × 5000 px JPEG (1 km² at 0.2 m): **13.5, 13.9, 13.7 s**, 4.7–5.0 MB each (≈0.35 MB/s — about a third of
  the COG throughput). 15000 × 100 px: 1.7 s. JPEG quality IJG q75.
- The WMS shows the **newest** vintage (at the Hamburg probe the 2026 1 km tile: mean abs diff 2.9 vs 30.7 to the 2024
  block).
- **Borders differ:** at the Hamburg probe the WMS shows imagery deep into Hamburg (to the edge of the NI tiles), but at
  the **Bremen probe it is clipped exactly at the state border** (Bremen white, 63.7 % of a 4 × 4 km image; the Bremen
  centre is all white) — although the COG tiles contain Bremen imagery (`dl/wms/borders_wms.jpg`). Outside any NI tile:
  white JPEG.
- Headers: no Content-Length, `cache-control: no-cache, no-store`, CORS `*`, session cookie set.
- **GetFeatureInfo gives the flight date — but only with an extra `FORMAT` parameter:**
  ```
  https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=ni_dop20_info&QUERY_LAYERS=ni_dop20_info&STYLES=&CRS=EPSG:25832&BBOX=551831,5803226,552031,5803426&WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=application/vnd.ogc.gml&FORMAT=image/png&FEATURE_COUNT=10
  → <PRODUKT>TrueDOP20 RGBI</PRODUKT><KACHELNAME>325505802</KACHELNAME><EDITION>2025</EDITION><BF_NAME>Hannover</BF_NAME>
    <BF_DATUM>20250306</BF_DATUM><BF_NUMMER>3817</BF_NUMMER><B_AUFL_CM>20</B_AUFL_CM><STAND>20260223</STAND><OWNER>LGLN</OWNER>
    (gml:Box 550000,5802000 552000,5804000)
  ```
  0.5 s. Without `FORMAT` every INFO_FORMAT → 200 `se_xml` "Parameter "format" fehlerhaft: null". `text/plain` returns only
  "Feature 2823:" without attributes; `text/html` has the attributes; `application/json` → 200 XML "Unsupported INFO_FORMAT".
  Where a 2 km block and a newer 1 km tile overlap, **both** are returned (Hamburg probe: `325665920` 20240502 and
  `325675920` 20260502) → take the newest `BF_DATUM`.

### 4.4 WCS `ni_dop`

- `https://opendata.geoservices.lgln.niedersachsen.de/dop_wcs` exists (caps: coverage `ni_dop`, "WCS NI DOP"). Not examined.

---

## 5. DGM1 (terrain model, 1 m)

- Source: ALS of 38 lots (`L1343` … `L2503`). The DGM1/DOM1 published in June 2024 had systematic errors; **the whole
  stock was re-derived and replaced in March 2026, and historic vintages 2013–2023 were added** (LGLN Aktuell 4/2026). ALS
  is now on a **6-year cycle** (from winter 2024/25; ≈8,000 km² at the coast, Elbe and Weser with ≥ 8 ground points/m²).
- Newest ALS per tile (49,708 tiles): 2010 79, 2011–2014 29, **2015 988, 2016 13,711, 2017 13,534**, 2018 1,529, 2019 905,
  2020 3,402, 2021 5,858, 2022 1,457, 2023 690, **2025 7,526**. Months: Jan–Apr 95 % (**leaf-off**), May 1,441, June 257,
  Sept 433, Dec 164. School tiles: **18.04.2016** (`L1603`); Hamburg probe 03.03.2025 (`L2502`); Bremen probe 14.02.2017
  (`L1702`).
- **Tile download:** asset `dgm1-tif`, e.g.
  ```
  HEAD https://dgm1.s3.eu-de.cloud-object-storage.appdomain.cloud/L1603/Vollkacheln/dgm1_32_551_5803_1_ni_2016.tif
  → 200, Content-Length: 3483298, ETag: "2a6f9f4e23c4cdc1f83a8a9067632b79", Last-Modified: Wed, 04 Mar 2026 10:42:13 GMT,
    Accept-Ranges: bytes, Content-Type: image/tiff, x-amz-meta-mtime: 1753703811.809243 (0.29 s)
  ```
  GET 3.5 MB in 3–5 s. Sizes 3.5–4.1 MB per tile.
- TIFF (verified on 551/5803, 552/5803, 567/5920 (2025), 485/5876 (2017)):
  - 1000 × 1000 **Float32** (SampleFormat 3), **Compression 5 (LZW), Predictor 1**, **512 × 512 internal tiles** (4 tiles,
    padded), **1 overview** (500 × 500), COG layout, IFD at 192;
  - ModelPixelScale 1, **ModelTiepoint (551000, 5804000)** (NW corner), GTRasterType 1, ProjectedCSType 25832,
    **`GDAL_NODATA` "-9999"**; no ImageDescription/Software/Copyright tags; **no TFW**.
  - Registration as LGB/NRW (tiepoint NW corner, value at pixel centre). freemapper's `tiff` crate reads tiled LZW Float32
    (not run). A whole tile is only 3.5 MB, so window reads are not worth it for the DTM.
- Item XML (`…_2016.xml`, 1,095 B, **UTF-8 BOM**): `AKTUALITAET_KACHELINFORMATIONEN 2025-12-29`, `VERSION_STANDARD 3.3`,
  `AKTUALITAET 2016-04-18`, `ERFASSUNGSMETHODE 5020`, `GENAUIGKEIT 0.3`, `…_HOEHE DE_DHHN2016_NH`, `LOS L1603`.
- Values: school tile 51.56–57.09 m, centre 53.00 m; no NoData. Border tiles full (§2.1).
- **WCS:** `https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs`, coverage **`ni_dgm1`**, envelope 341000 5681992 …
  676040 5973000, 1 m grid origin (341000.5, 5972999.5), formats image/tiff, png, jpeg …
  ```
  https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=ni_dgm1&FORMAT=image/tiff&SUBSET=x(551800,552000)&SUBSET=y(5803200,5803400)
  ```
  → 200 `image/tiff`, 110,159 B, 1.2 s; LZW **Predictor 3 (floating point)** (supported by the `tiff` crate), strips of 10
  rows, tiepoint (551800, 5803400); **byte-identical to the COG** (max |Δ| 0.0). 2 km: 10.6 MB, 17.8 s; 4 km: 43 MB, 48.6 s;
  5 km: 67.9 MB, 77 s — **no size limit found**. It serves one mosaic (presumably the newest lot), no vintage choice.

---

## 6. nDSM sources: DOM1, bDOM20/10, LiDAR

### 6.1 DOM1 (1 m, ALS)

- STAC `https://dom.stac.lgln.niedersachsen.de`, asset `dom1-tif`, e.g.
  `https://dom1.s3.eu-de.cloud-object-storage.appdomain.cloud/L1603/Vollkacheln/dom1_32_551_5803_1_ni_2016.tif`
  (4,279,511 B). Format identical to DGM1 (LZW, 512 px tiles, 1 overview, NoData −9999). XML `GENAUIGKEIT 0.5`,
  `VERSION_STANDARD 1.2`.
- **DOM1 and DGM1 have the same lot and date per tile** (identical tile and vintage counts; 4 extra DOM items).
- **DOM1 − DGM1** over the school footprint in tile 551 (E 551818–552000 × N 5803230–5803421): p10 0.01, **median 6.21**,
  p90 15.18, max 33.57 m; 63.8 % > 2 m; 1.1 % < 0 (clamp).
- **Age gap to the DOP** (newest DOP year − newest DGM1 year per tile, 49,708 tiles): −1: 211, 0: 82, 1: 7,233, 2: 695,
  3: 711, 4: 1,553, 5: 7,823, 6: 1,531, 7: 2,870, **8: 10,828, 9: 14,448**, 10–15: 1,723. **79 % of tiles ≥ 5 years,
  54 % ≥ 8 years.** The school: DOM 2016 vs DOP 2025.
- WCS `dom_wcs` / `ni_dom1` exists (caps verified, not compared).

### 6.2 bDOM20 / bDOM10 (image-based DSM, same date as the DOP)

- STAC `https://bdom.stac.lgln.niedersachsen.de`, collection `BDOM` (2021-03-29 … 2026-05-25), asset `bdom20`:
  `https://bdom20.s3.eu-de.cloud-object-storage.appdomain.cloud/325515803/2025-03-06/bdom20_32_551_5803_1_ni_2025-03-06.tif`
- 2025 COG: 5000 × 5000 **Float32, uncompressed**, 512 px tiles (1 MB each), 4 overviews, **145.8 MB per km²**; no NoData
  tag. The 2022 vintage of the same tile is **LZW with `GDAL_NODATA nan`** (133.3 MB) — encoding differs by vintage.
  bDOM10: 10000², 565 MB.
- Item XML: `software SURE 2024.3.4`, `quelldaten_gsd 6`, overlap 80/50, `lagegenauigkeit 40`, `hoehengenauigkeit 60`,
  `erfassungsmethode 5040`, `belaubungszustand 0`.
- Window read at the school (E 551818–552000, N 5803230–5803421; `scripts/bdomwindow.py`): 4 range requests, 9.45 MB,
  10.4 s, 955 × 910 px, 52.56–83.23 m, no NaN.
- **bDOM (2025-03-06, averaged to 1 m) − DGM1 (2016)**: p1 −0.25, p10 −0.05, **median 0.26**, p90 13.16, max 21.99 m;
  only **35.8 % > 2 m** (DOM1 − DGM1: 63.8 %), 23.9 % < 0. The March flight is leaf-off: image matching sees through the
  bare crowns, so trees nearly vanish; buildings remain.
- **Currency:** the newest bDOM date equals the newest DOP date on **50,865 of 52,611 tiles (96.7 %)**; the rest differ by
  ±1–3 years. So bDOM − DGM1 matches the orthophoto (buildings, construction sites), DOM1 − DGM1 does not.

### 6.3 LiDAR / nDOM

- No open LAZ point cloud and no nDOM product were found (no STAC hosts `ndom`, `laz`, `las`; the hub pages name "Laserscan-
  Punktwolke" only as a source of LoD2). LoD1/LoD2 indexes exist (`…/pro-download-indices/lod{1,2}/…geojson`), not examined.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **STAC item `properties.datetime`** | DOP (flight date), DGM1/DOM1 (ALS date + `los`), bDOM (flight date) | per tile and vintage; ISO `YYYY-MM-DDT00:00:00Z`. The file name carries the same date (DOP/bDOM full date, DGM/DOM year; name year = datetime year for all 70,285 DGM items) |
| Item XML next to each COG (asset `*metadata*`) | all | DOP: `aktualitaet 2025-03-06`, `aktualitaet_kachelinformationen 2026-02-13`, `bildflugname`, `kamera_sensor`, `komprimierung "JPEG 75%"`, `belaubungszustand 0`, `hintergrundwert 255`; DGM/DOM: `AKTUALITAET`, `LOS`, `GENAUIGKEIT` (UTF-8 BOM, upper-case tags) |
| DOP WMS GetFeatureInfo `ni_dop20_info` | DOP | `BF_DATUM YYYYMMDD`, `STAND`, `KACHELNAME` (needs `FORMAT=`; returns overlapping 2 km and 1 km tiles) |
| ETag / Last-Modified / `x-amz-version-id` | every object | object build date (DGM 2016 tile: 04 Mar 2026 = the re-derivation), usable for change detection |
| Hub index GeoJSON `Aktualitaet` | newest tile only | 36–83 MB, not needed |
| ALKIS `aktualit` | per feature | `YYYY-MM-DDZ`. Buildings in the samples: 2011 2,567 (migration date), 2017 1,164, 2022 3,464, 2025 3,865, 2026 3,242 …; land-use parcels mostly 2019–2026 |

Suggested mapping:
- orthophoto `currency` = `datetime` of the newest DOP item under the map (or `BF_DATUM`); `published` = its ETag/
  Last-Modified or the XML `aktualitaet_kachelinformationen`; report leaf-off/leaf-on from the month;
- DGM1/DOM1 `currency` = item `datetime` (same for both); bDOM = its `datetime` (= DOP date on 96.7 % of tiles).

---

## 8. ALKIS: buildings and actual land use

### 8.1 WFS

- `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach`, "WFS NI ALKIS Vereinfacht", WFS 2.0.0/1.1.0.
  - Types (§2), DefaultCRS 25832; OtherCRS 25833, 3034, 3857, 4258, 4326, 900913.
  - Output: GML 3.1/3.2 (default `application/gml+xml; version=3.2.1`), `application/x-zip-shapefile` (GetFeature only).
    **No GeoJSON, no GeoPackage.**
  - Constraints: `ImplementsResultPaging FALSE` (but STARTINDEX works; SORTBY works), **`CountDefault 10000`**.
  - Default namespace `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0`, schema location
    `http://download.lgln.niedersachsen.de/aaa-suite/schema/…` — the Hesse/NRW/SH layout.
- freemapper's `alkis_wfs` request shape works unchanged:
  ```
  https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=551730,5803175,552130,5803475,urn:ogc:def:crs:EPSG::25832&SORTBY=oid&COUNT=10000&STARTINDEX=0
  ```
  → 200, **169 features**, 199,013 B, 0.48 s (again 0.37 s). `RESULTTYPE=hits` → `numberMatched="169"` (0.18 s); results
  say `numberMatched="unknown"`.
- Speed (no gzip: `Vary: Accept-Encoding` but no `Content-Encoding`): KWR 2 × 2 km: 3,937 buildings 4.3 MB **4.6 s**,
  2,922 NutzungFlurstueck 3.8 MB 4.1 s, 1,164 Nutzung 1.9 MB 2.2 s; Cuxhaven 2 × 2 km: 5,993 buildings 6.0 MB 6.4 s.
  5 × 5 km Hannover: hits **25,254** (1.7 s).
- **Limits:** `COUNT=10000` → 200; **`COUNT=10001` / `25000` → HTTP 400** (Apache HTML "400 Internal Server Error … Your
  browser sent a request that this server could not understand"); **no `COUNT` on the 5 km box → 200 with exactly 10,000
  features (10.5 MB, 10.9 s) and no sign of truncation** → always ask `hits` first or page.
- Paging: `COUNT=1000&SORTBY=oid` with `STARTINDEX=0/1000/3000` → 1000/1000/937 (= 3,937), disjoint, ordered.
- 25833: `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833` → 183 features at the school (rotated bbox), E N order, 0.41 s.
- **Errors are generic Apache HTML 400s** (`text/html;charset=iso-8859-1`): unknown type, `OUTPUTFORMAT=application/json`,
  unsupported SRS (`EPSG::2056`) — no OGC exception, no detail.
- Outside NI (Bielefeld): `numberReturned="0"`. **ALKIS stops at the state border:** Hamburg probe 662 buildings, all
  `gmdschl 03353031` (Seevetal), none outside NI; Bremen tile 482/5876: 225 buildings / 243 parcels, none > 50 % outside NI
  (max 10.8 %, a building on the line).
- Geometries are `MultiSurface` of `Polygon`s with `posList`, whole features across the bbox. No `;`-joined `funktion`.

### 8.2 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid` (18 chars), `aktualit`, `gebnutzbez`, `funktion`, `gmdschl` (always), `gfkzshh` (92 %),
  `lagebeztxt` (50 %), `name` (1.5 %), `rellage` (0.85 %). **LGB spellings** ("Gebäude", "Allgemein bildende Schule").
- `NutzungFlurstueck`: `oid` (34 chars = Nutzung oid + Flurstück oid), `aktualit`, `nutzart` (always), `flstkennz`
  (always), `bez` (49 %), `name` (0.15 %).
- **`ave:Nutzung` has `nutzart`** (8,347 of 8,347) with the same values; unsplit objects. `NutzungFlurstueck` is the LGB
  match.
- oids start with **`DENI`** (`DENIAL43…` Hannover, `DENIAL01…` Braunschweig, `DENIAL55…` Göttingen, `DENIAL77…`
  Osnabrück, `DENIAL32…` Cuxhaven; also `DENIALP1…`); `…BL` buildings, `…TF` parcel land use → `Region::of_object_id`
  gets a `"DENI"` arm.

### 8.3 Sample features at the Kaiser-Wilhelm- und Ratsgymnasium (WFS, geometry elided)

```json
{"oid":"DENIAL4300029q7VBL","aktualit":"2025-09-09Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","name":"Kaiser-Wilhelm- und Ratsgymnasium","gmdschl":"03241001","lagebeztxt":"Seelhorststraße 52"}
{"oid":"DENIAL430001t2toBL","aktualit":"2019-04-12Z","gebnutzbez":"Gebäude","funktion":"Sport-, Turnhalle","gfkzshh":"31001_3211","name":"Sporthalle ","gmdschl":"03241001","lagebeztxt":"Lüerstraße 15"}
{"oid":"DENIAL4300029q81BL","aktualit":"2025-07-24Z","gebnutzbez":"Bauteil","funktion":"Arkade","gmdschl":"03241001"}
{"oid":"DENIAL430001Kmm1BL","aktualit":"2022-03-10Z","gebnutzbez":"Gebäude","funktion":"Gebäude für Wirtschaft oder Gewerbe","gfkzshh":"31001_2000","rellage":"Unter der Erdoberfläche","gmdschl":"03241001"}
{"oid":"DENIAL430000ae3EDENIAL4300009yH7TF","aktualit":"2019-04-12Z","nutzart":"Fläche besonderer funktionaler Prägung","bez":"Bildung und Wissenschaft","flstkennz":"034880016000110127__"}
{"oid":"DENIAL4300009BvtDENIAL4300009yepTF","aktualit":"2026-07-06Z","nutzart":"Sport-, Freizeit- und Erholungsfläche","bez":"Sportanlage","flstkennz":"034880016000110126__"}
{"oid":"DENIAL4300009BBEDENIAL4300009Av3TF","aktualit":"2021-01-04Z","nutzart":"Sport-, Freizeit- und Erholungsfläche","bez":"Spielplatz, Bolzplatz","flstkennz":"034880016000110125__"}
```

Note the trailing blank in `"Sporthalle "`.

### 8.4 Observed values (seven areas: 21,179 buildings, 20,582 parcels, 8,347 Nutzung)

- **`gebnutzbez`:** `Gebäude` 17,677, `Sonstiges Bauwerk oder sonstige Einrichtung` 1,870, `Bauteil` 1,588,
  `Vorratsbehälter, Speicherbauwerk` 17, `Bauwerk oder Anlage für Industrie und Gewerbe` 12, `Bauwerk oder Anlage für
  Sport, Freizeit und Erholung` 8, `Turm` 7.
- **`funktion` is detailed** (69 values): Wohngebäude 7,242, Gebäude für Wirtschaft oder Gewerbe 5,423, Überdachung 1,864,
  Wohngebäude mit Handel und Dienstleistungen 1,808, Gebäude für Handel und Dienstleistungen 1,234, Durchfahrt im Gebäude
  797, Gebäude für Handel und Dienstleistung mit Wohnen 518, Arkade 418, Auskragender Geschossteil / zurückspringendes
  Geschoss 358, Gebäude zur Versorgung 213, Gebäude für öffentliche Zwecke 208, Garage 109, Hochschulgebäude
  (Fachhochschule, Universität) 100, **Allgemein bildende Schule 65**, Kirche 39, Sport-, Turnhalle 36, Parkhaus 30,
  Berufsbildende Schule 23, Krankenhaus 13, Kinderkrippe, Kindergarten, Kindertagesstätte 8 … **No "Tiefgarage".**
- **`rellage`:** `Unter der Erdoberfläche` 130, `Aufgeständert` 50.
  - **Underground structures are `Gebäude`, not `Bauteil`:** the 130 underground objects are `Gebäude` with funktion
    "Gebäude für Wirtschaft oder Gewerbe" 123, Garage 3, Handel und Dienstleistungen 2 … . **No `Bauteil` is underground**
    (Bauteil = Durchfahrt im Gebäude 797, Arkade 418, Auskragender Geschossteil 358, Durchfahrt an überbautem Verkehrsweg 12,
    Turm im Gebäude 3; `rellage` only once, "Aufgeständert"). A filter on `rellage` covers them; a filter on
    `Bauteil`/`Tiefgarage` alone would draw them as buildings.
- **`name`** (316): mostly clean, but with **label fragments** — `"Ga"`, `"Ke"` (on underground objects in Braunschweig),
  a name split over two buildings ("Gymnasium Kleine Burg und Abendgymnasium" and `"und Abendgymnasium"` on a shop), and
  **7 names with trailing blanks** ("Sporthalle ", "Sambesi  ", "Yukon Bay  "). Trim, and ignore names < 3 characters.
- **`nutzart`** (NutzungFlurstueck; LGB spellings): Wohnbaufläche 7,338, **Industrie- und Gewerbefläche 5,106**,
  Straßenverkehr 3,105, Weg 2,370, Fläche besonderer funktionaler Prägung 796, Sport-, Freizeit- und Erholungsfläche 634,
  Bahnverkehr 344, Platz 341, Fließgewässer 188, Wald 126, Landwirtschaft 41, Heide 36, Stehendes Gewässer 34,
  Unland/Vegetationslose Fläche 32, Schiffsverkehr 31, Gehölz 28, Hafenbecken 23, Friedhof 7, **Fläche gemischter Nutzung
  2**. `PRIVATE_LAND_USES` matches unchanged.
- **`bez` per `nutzart`:**
  - `Industrie- und Gewerbefläche`: **Handel und Dienstleistung 4,582** (the city-centre blocks; NI does not use "Fläche
    gemischter Nutzung"/"Mischnutzung mit Wohnen" there), Industrie und Gewerbe 442, Gebäude- und Freifläche
    Versorgungsanlage, Elektrizität 51, Betriebsfläche … 8, Lagerfläche 3, Werft 3, Umspannstation 2 …
  - `Straßenverkehr`: none 2,306, **`Begleitfläche Straßenverkehr` 585 (19 %)**, `Fußgängerzone` 198, `Gebäude- und
    Freifläche zu Verkehrsanlagen, Straße` 16. Never `Fahrbahn`.
  - `Weg`: Fußweg 1,307, Rad- und Fußweg 587, none 352, Radweg 124.
  - `Platz`: **none 186 (55 %)**, Parkplatz 144, Marktplatz 9, Fußgängerzone 2 → `square_bez` needed.
  - FBFP: Bildung und Wissenschaft 228, Religiöse Einrichtung 119, Regierung und Verwaltung 118, Soziales 84, Kultur 78,
    Gesundheit, Kur 45, Öffentliche Zwecke 43, Sicherheit und Ordnung 37, Medien und Kommunikation 30, Historische Anlage 14.
  - SFE: Siedlungsgrünfläche 383, Park 135, Spielplatz, Bolzplatz 37, Sportanlage 31, Gebäude- und Freifläche Sport,
    Freizeit und Erholung 16, Kleingarten 10, Zoo 8, Schwimmen 5, Grünanlage 4, Botanischer Garten 3 …
  - Wald: Nadelholz 98, Laubholz 22, Laub- und Nadelholz 6. Fließgewässer: Graben 83, Fluss 80, Kanal 25. Stehendes
    Gewässer: Teich 28, Stausee 6. Landwirtschaft: Grünland 41. Unland: Gewässerbegleitfläche 32. Bahnverkehr: none 318,
    Eisenbahn 16, Stadtbahn 10. Friedhof: none 4, Historischer Friedhof 3.
  - `Wohnbaufläche`, `Heide`, `Gehölz`, `Hafenbecken`: no `bez`.

### 8.5 What is missing or different

- GML only, generic HTML 400s, no gzip, **silent 10,000 cut without COUNT**, COUNT > 10000 refused.
- Underground garages as `Gebäude` + `rellage`; label fragments and trailing blanks in `name`.
- City-centre land use is "Industrie- und Gewerbefläche / Handel und Dienstleistung" (private for out-of-bounds — the same
  effect as "Fläche gemischter Nutzung" elsewhere).
- No OGC API Features endpoint found. Not examined: Flurstück, KatasterBezirk, LoD2, ALKIS Landnutzung GPKG.

---

## 9. Licence and attribution

- Licence: **Creative Commons Namensnennung 4.0 International (CC BY 4.0)** — STAC collections and items
  (`"license": "CC-BY-4.0"`), WMS/WFS/WCS AccessConstraints, and the LGLN AGNB.
- **AGNB** (`https://www.lgln.niedersachsen.de/startseite/wir_uber_uns_amp_organisation/allgemeine_geschafts_und_nutzungsbedingungen_agnb/allgemeine-geschafts-und-nutzungsbedingungen-agnb-97401.html`,
  "Stand: 01.10.2024"), verbatim:
  > 7.1 Die auf der Homepage des LGLN unter: https://ni-lgln-opengeodata.hub.arcgis.com aufgeführten Geodaten (offene
  > Geodaten) können unter den Bedingungen der Lizenz „Creative Commons" (CC-BY 4.0)" kostenfrei intern und extern genutzt
  > werden.
  > 7.2 Bei der Nutzung von offenen Geodaten ist die Lizenznehmerin oder der Lizenznehmer verpflichtet, bei jeder
  > öffentlichen Wiedergabe, Verbreitung oder Präsentation der Geodaten sowie bei jeder Veröffentlichung oder externen
  > Nutzung einer Bearbeitung oder Umgestaltung den folgenden deutlich sichtbaren Quellenvermerk anzubringen:
  > **© GeoBasis-DE/LGLN Jahr 3), Daten geändert 4)**
  > 3) Jahr des Downloads der Geodaten durch die Lizenznehmerin oder den Lizenznehmer, z. B. 2024
  > 4) anzugeben, falls Daten verändert wurden
  > Bei Darstellungen im Format kleiner als DIN A5 genügt die Bildmarke. … Steht der Quellenvermerk nicht in unmittelbarem
  > Zusammenhang zur Darstellung, ist die Nutzung der Geodaten dem LGLN anzuzeigen.
- **Service AccessConstraints** (WMS; WFS/WCS the same wording), verbatim:
  > Es gelten die Lizenzbedingungen "Creative Commons Namensnennung – 4.0 International (CC BY 4.0)" bzw. "cc-by/4.0"
  > (http://creativecommons.org/licenses/by/4.0/) mit den dort geforderten Angaben zum Quellenvermerk. Als Rechteinhaber und
  > Bereitsteller ist "LGLN", sowie das Jahr des Datenbezugs in Klammern anzugeben. Beispiel für Quellenvermerk: LGLN (2024)
  > Creative Commons Namensnennung – 4.0 International (CC BY 4.0)
  `Fees: None/NONE`.
- **Stale text:** the old OpenGeoData.NI app (`opengeodata.lgln.niedersachsen.de/main.js`) still says "Datenlizenz
  Deutschland – Namensnennung – Version 2.0" with "Auszug aus den Geodaten des Landesamtes für Geoinformation und
  Landesvermessung Niedersachsen, ©Jahr". The AGNB and every live service say CC BY 4.0.
- **Recommended freemapper line** (satisfies both the AGNB form and the CC BY change note):
  `© GeoBasis-DE/LGLN <download year>, Daten geändert, CC BY 4.0` (licence link `https://creativecommons.org/licenses/by/4.0/`
  in the project notes). Because the line sits in the map's credits, no notice to LGLN is needed.

---

## 10. State boundary (for `crates/adapters/de/manifests/niedersachsen/boundary.geojson`)

- **Source:** the ALKIS WFS `ave:VerwaltungsEinheit` with `art = Bundesland`, directly in 25833:
  ```
  https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&FILTER=<fes:Filter xmlns:fes="http://www.opengis.net/fes/2.0" xmlns:ave="http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0"><fes:PropertyIsEqualTo><fes:ValueReference>ave:art</fes:ValueReference><fes:Literal>Bundesland</fes:Literal></fes:PropertyIsEqualTo></fes:Filter>
  ```
  (URL-encode the FILTER) → 200, 1 feature, 2,135,729 B, **13.4 s** (25832: 1,437,844 B, 19.6 s). oid **`DE03`**,
  `name Niedersachsen`, `aktualit 2026-11-06Z`. A 10 m bbox query at the school returns 4 VerwaltungsEinheit (hits).
- Geometry: MultiSurface, **2 polygons**, 62,958 vertices, valid (shapely):
  - main polygon 51,401.04 km² (25833) incl. the islands and coastal waters, with **17 holes**:
    - **Bremen (city) 319.51 km²** (53.126°N 8.814°E), **Bremerhaven 93.36 km²** (53.543°N 8.600°E), two small Bremen
      pieces 0.579 km² (53.606°N 8.651°E, north of Bremerhaven) and 0.044 km² (53.196°N 8.614°E);
    - 13 zero-area slivers (< 1 m², e.g. at 25833 (112143, 5785189), (−21553, 5940570)).
  - a **0.048 km² exclave** at 51.561°N 9.606°E (Weser valley at the Hesse border).
- **25 m topology-preserving simplification:** `probe/boundary/niedersachsen_simplified25m.geojson` = `ni/niedersachsen.geojson`
  (script `scripts/simplify_ni.py`: drop holes < 1 m², shapely 2.2 `simplify(25, preserve_topology=True)`,
  `set_precision(grid_size=1)`, `normalize`).
  - FeatureCollection, `name` "Niedersachsen (ALKIS, simplified 25 m)", `crs` `urn:ogc:def:crs:EPSG::25833`, properties
    `name: Niedersachsen`, `source: LGLN ALKIS vereinfacht WFS (VerwaltungsEinheit DE03), © GeoBasis-DE/LGLN 2026, CC BY 4.0`,
    `retrieved: 2026-11-08`, `simplified_m: 25`; MultiPolygon with integer coordinates.
  - **5,927 vertices, 96.6 KB** (Hesse 5,166 / 83 KB, NRW 5,606 / 94.5 KB). Parts: main (5,240 exterior vertices,
    **4 holes: Bremen 319.517, Bremerhaven 93.369, 0.586, 0.044 km²**) and the 0.045 km² exclave. Exterior rings clockwise
    (like the Hesse/SH assets).
  - **Valid** (shapely). The repo test `region_boundaries_are_valid` (geo `check_validation`) was not run.
  - Area 51,401.39 km² (full 51,401.08), symmetric difference 9.0 km², **Hausdorff 28.7 m** (boundary to boundary).
  - Extent E −63,523 … 271,928, N 5,695,828 … 6,006,482: **negative eastings** in 25833.
- **Against the neighbour assets** (`scripts/border_ni.py`; shared border = true NI boundary within 150 m of the neighbour
  asset; deviation sampled every 50 m):

  | Neighbour asset | Shared border | Overlap (asset ∩ asset) | Gap between them (within 60 m of the border) | True border → NI asset p99 / max | → neighbour asset p99 |
  |---|---|---|---|---|---|
  | `hamburg.geojson` | 80.5 km | 67,560 m², 29 pieces, max width 22.7 m | 90,581 m², 30 pieces | 21.2 / 24.7 m | 21.4 m |
  | `schleswig-holstein.geojson` | 109.8 km | **89.0 km²** (see below) + slivers ≤ 15,024 m² | 110,483 m² | 20.6 / 24.6 m | (5,993 m, the estuary) |
  | `hessen.geojson` | 168.6 km | 72,523 m², 52 pieces, max width 23.9 m | 103,917 m² | 21.8 / 24.9 m | 22.0 m |
  | `saxony-anhalt.geojson` | 343.8 km | 4,085 m², 2 pieces | 37,918 m² | 20.0 / 24.4 m | 20.5 m |
  | `brandenburg.geojson` | 30.2 km | 34,046 m², 14 pieces, max width 21.5 m | 101,935 m² | 21.0 / 21.9 m | 24.5 m |
  | `mv/mecklenburg-vorpommern.geojson` (scratchpad) | 80.8 km | 29,841 m², 21 pieces | 84,948 m² | 22.2 / 24.3 m | 22.5 m |
  | `nrw/…/nordrhein-westfalen_simplified25m.geojson` (scratchpad) | 587.9 km | 34,833 m², 39 pieces | 74,803 m² | 21.1 / 24.8 m | 21.2 m |

  (The "gap" column counts the area within 60 m of the shared border that is in neither asset, including the cut ends near
  tripoints, so its piece widths are not meaningful; the slivers along the line are ≲ 25 m wide like the overlaps.)
  - **Elbe estuary overlap with Schleswig-Holstein: 89.0 km²** off Cuxhaven (53.84–53.97°N, 8.71–8.98°E). It is in the
    official data: the full-resolution ALKIS polygons of NI (DE03) and SH (DE01) overlap by **88.986 km²** — both states'
    cadastres claim the same part of the outer Elbe. On land the two borders follow the Elbe and agree.
  - Hamburg: the full NI polygon and the Hamburg asset overlap by 141,990 m² (Hamburg's own 25 m simplification). Hamburg's
    Neuwerk/Scharhörn parts (3.53, 4.08 km²) do not touch NI (no hole needed).
  - Bremen and Thuringia have no freemapper asset; the Bremen holes come straight from ALKIS (Bremen's own polygon not
    compared). Netherlands border (Dollart/Ems) not checked.
  - So, as for NRW–Hesse, a point within ≈25 m of a border can be in both or neither asset; `regions_for` buffers by
    `MARGIN_M`. The Elbe-estuary overlap is open water, irrelevant for school maps.
- Neighbours: Bremen (enclaves), Hamburg, Schleswig-Holstein (Elbe), Mecklenburg-Western Pomerania, Brandenburg,
  Saxony-Anhalt, Thuringia, Hesse, North Rhine-Westphalia, the Netherlands.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/niedersachsen-latest.osm.pbf` → **307** →
  `niedersachsen-261107.osm.pbf` → 200, **509,866,838 B** (486 MiB), `Last-Modified: Sun, 08 Nov 2026 02:02:37 GMT`,
  `ETag "1e63f356-65fba3d4bd30f"`, `Accept-Ranges: bytes`. (NRW 917 MB, SH 159 MB, Hesse 348 MB.) Within freemapper's
  `MAX_EXTRACT_BYTES` (2 GiB). No sub-region extracts listed for Niedersachsen.
- State: `https://download.geofabrik.de/europe/germany/niedersachsen-updates/state.txt`
  ```
  # original OSM minutely replication sequence number 7362973
  timestamp=2026-11-07T21\:21\:36Z
  sequenceNumber=4955
  ```
- Poly: `https://download.geofabrik.de/europe/germany/niedersachsen.poly` (18,305 B, **one ring, no holes**, 590 points,
  lon 6.295–11.607, lat 51.293–54.240). It **includes Bremen and Bremerhaven** (and Neuwerk), excludes Hamburg (Rathaus and
  Harburg outside). A Bremen-border map is covered by this one extract; a Hamburg-border map needs the Hamburg extract too.
  `bremen-latest.osm.pbf` is 21.3 MB.
- Overpass answered 200 with `freemapper-research/0.1 (github.com/HenryJobst/freemapper)` (28–36 s per query); the
  `area["name"="Hannover"]["admin_level"="6"]` query returned nothing — use a bbox.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `*.stac.lgln.niedersachsen.de` | `limit` capped at 1000 silently; cursor paging via `next`; no `numberMatched`; bbox in lon/lat; 0.2–0.3 s (cold 3–4 s); JSON errors with proper 400/404 |
| `*.s3.eu-de.cloud-object-storage.appdomain.cloud` | HEAD + Content-Length, ETag, Last-Modified; **Range → 206, suffix range OK, beyond EOF → 416, multi-range → only the first range (206, no multipart)**; missing key → 404 XML; buckets listable; **HEAD on large DOP objects took 3.9–5.0 s** (DGM 0.2–0.3 s; range GETs 0.25–0.45 s) — skip HEAD, GET a range directly; Content-Type `image/tiff` or `application/octet-stream`; DOP/DGM URLs not computable (date / lot in the path) |
| `opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms` | **max 15000 px**, oversize → **200 `se_xml`**; ≈13.6 s per 5000² JPEG (0.35 MB/s); GetFeatureInfo needs `FORMAT`; no Content-Length/caching headers; session cookie (not needed); **Bremen clipped, Hamburg not** |
| `…/doorman/noauth/alkis_wfs_einfach` | GML only; **COUNT ≤ 10000 (else 400), default 10000 with silent truncation**; all errors HTML 400 without detail; no gzip; 0.2–6.5 s |
| `opendata.geoservices.lgln.niedersachsen.de/*_wcs` | no size limit found up to 5 km (68 MB, 77 s); LZW + float predictor; unknown service → **200 HTML MapServer message** |
| Geofabrik | 307 to the dated file; 510 MB |

- **TLS (checked with `openssl s_client -showcerts -CAfile <certifi bundle> -verify_return_error`, which does no AIA
  fetching): every host verifies, `Verify return code: 0 (ok)`, TLS 1.3:**
  - `opendata.lgln.niedersachsen.de`: 3 certs, leaf → Let's Encrypt **YR2** → ISRG Root YR (cross-signed by **ISRG Root X1**);
  - `opendata.geoservices.lgln.niedersachsen.de`, `{dop,dgm,bdom}.stac.lgln.niedersachsen.de` (wildcards): 2 certs, leaf →
    RapidSSL TLS ECC CA G1 → **DigiCert Global Root G3**;
  - `*.s3.eu-de.cloud-object-storage.appdomain.cloud` (dop20-rgb, dgm1, arcgis-geojson): 3 certs, IBM leaf → DigiCert
    Global G2 TLS RSA SHA256 2020 CA1 → **DigiCert Global Root G2**;
  - `opengeodata.lgln.niedersachsen.de`, `single-datasets.opengeodata…`: Google Trust Services WE1 → GTS Root R4
    (cross-signed by GlobalSign Root CA);
  - `download.geofabrik.de`: Let's Encrypt YE1 → Root YE → ISRG Root X2 → ISRG Root X1.
  All roots are in the Mozilla/webpki set, so ureq + rustls + webpki-roots should connect (inferred; no Rust client run).
  **No SH-style missing intermediate.**
- Throughput: ≈1.15 MB/s per file from COS (the line; Geofabrik heads gave the same order), WMS ≈0.35 MB/s.
- No authentication, no rate-limit or `Retry-After` headers, no throttling in ≈450 STAC pages and ≈150 other requests.
  CORS `*` on the gateway.
- Total transfer for this research ≈ **0.6 GB** (STAC paging ≈0.1 GB, bDOM index 74 MB, WCS 2–5 km clips 120 MB (deleted),
  WMS probes ≈30 MB, DOP/DGM/DOM/bDOM tiles and ranges ≈90 MB, ALKIS ≈80 MB incl. a deleted 10 MB answer); disk under `ni/`
  ≈370 MB.

---

## 13. What makes the Lower Saxony adapter differ (risks first)

1. **A STAC client and a COG reader are new code.** Tile names and URLs cannot be computed (DOP/bDOM: flight date in folder
   and name; DGM/DOM: ALS lot folder + year, with lot-edge duplicates in `Randkacheln`). The adapter must search STAC by
   lon/lat bbox, pick the newest item (DOP: all vintages and two tile schemes come back), and take `assets.dop20_rgb.href` /
   `dgm1-tif` / `dom1-tif` / `bdom20`. Searches are fast (0.2–0.3 s) and small.
2. **Orthophoto decoding:** the DOP COGs are JPEG-in-TIFF, **YCbCr 4:2:0, 128 px tiles**. The `tiff` crate decodes them but
   **returns YCbCr samples** — freemapper must convert to RGB. Window reads need a range-backed `Read + Seek` (or tile-row
   ranges) and must skip mask IFDs and pick overviews by size. Alternative without new decoding: the **WMS** (15000 px max,
   but ≈3× slower than ranges and Bremen-clipped).
3. **nDSM choice:** DOM1 − DGM1 is the drop-in (same format as DGM1, 4 MB per km²) but **8–9 years older than the DOP on half
   the state** (79 % ≥ 5 years). bDOM20 − DGM1 matches the DOP date (96.7 %) but is 146 MB per km² uncompressed (window
   reads: ≈9.5 MB for a 200 m school), leaf-off crowns are missing (school: 36 % > 2 m vs 64 %), and its DGM is still old.
   **Decision needed**; DOM1 − DGM1 with a date warning is the conservative default.
4. **ALKIS WFS cut-off:** without `COUNT` the answer stops at 10,000 features without any marker, and `COUNT` > 10000 is
   rejected. Keep freemapper's `COUNT=10000&SORTBY=oid&STARTINDEX` paging and stop on `numberReturned < COUNT` (or ask
   `hits` first).
5. **ALKIS content:** underground garages are `Gebäude` + `rellage "Unter der Erdoberfläche"` (no `Bauteil`/`Tiefgarage`) —
   the underground filter must use `rellage`; label fragments ("Ga", "Ke", "und Abendgymnasium") and trailing blanks in
   `name`; city-centre blocks are "Industrie- und Gewerbefläche / Handel und Dienstleistung"; 55 % of `Platz` without `bez`.
6. **Currency is a patchwork:** DOP 2024/2025/2026, three quarters leaf-off; DGM/DOM mostly 2016–2017, 2025 at the coast.
   Report dates per tile.
7. **Borders:** Bremen is a hole (two big + two small pieces). Data tiles are full squares across the borders; the WMS is
   clipped at Bremen only; ALKIS stops exactly at the border; the Geofabrik extract includes Bremen. The NI and SH cadastres
   overlap by 89 km² in the Elbe estuary.
8. **CRS 25832** everywhere; in 25833 12 % of NI is at negative eastings and convergence reaches −6.7°. `Crs::Utm32`.
9. Unchanged from the other states: no login, DHHN2016, DGM1 registration (NW-corner tiepoint, PixelIsArea), Float32 with
   NoData −9999, 1 km DGM tiles, LGB spellings, `ave:` WFS, Geofabrik layout, CC BY 4.0 (as SH). New: oid prefix `DENI`,
   no TFW anywhere.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Lower Saxony | Notes |
|---|---|---|
| **Orthophoto template** | **COG window reads**: STAC `https://dop.stac.lgln.niedersachsen.de/search?bbox=<lon/lat>&limit=1000` → newest item per cell → asset **`dop20_rgb`** → header range `bytes=0-98303` + one range per tile row; IFD 0 of 20 cm tiles or IFD 1 of 10 cm tiles for 0.2 m | Convert YCbCr → RGB; skip mask IFDs. Currency = item `datetime` (+ leaf-off/on by month). Fallback/simple path: **WMS** `WmsService{url: "https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms", layer: "ni_dop20", crs: Utm32}`, pieces ≤ 4000 px (server 15000), timeout ≥ 60 s; date via GFI (`FORMAT` needed). |
| **DTM** (hillshade, slope, contours) | **DGM1 COG** via STAC `https://dgm.stac.lgln.niedersachsen.de/search?collections=dgm1&bbox=…` → newest item per tile → asset `dgm1-tif`, whole file (≈3.5–4 MB) | LZW Float32, NoData −9999, no TFW (GeoTIFF tags). ETag as `published`. Fallback: WCS `ni_dgm1` (identical values). |
| **nDSM template** | **DOM1 − DGM1** (STAC `dom.stac…`, asset `dom1-tif`, same item date as the DGM) | Clamp < 0; warn with DOM date vs DOP date (usually 5–9 years). Option: bDOM20 − DGM1 via window reads when the DOP is leaf-on and the DOM is old. |
| **Buildings** | **WFS** `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach`, `ave:GebaeudeBauwerk`, 25832, `SORTBY=oid&COUNT=10000&STARTINDEX=…` | Reuse `alkis_wfs` (GML reader of Hesse/SH/NRW). Underground via `rellage`; trim names. oid prefix `DENI`. |
| **Land use** (out-of-bounds, paved) | the same WFS, **`ave:NutzungFlurstueck`** | `square_bez` "ohne Unterart (Niedersachsen)"; verge rule fires on 19 % of streets. Proposed: `WfsService { name: "LGLN ALKIS", url: "https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach", buildings_type: "ave:GebaeudeBauwerk", land_use_type: "ave:NutzungFlurstueck", crs: Crs::Utm32, square_bez: "ohne Unterart (Niedersachsen)" }` |
| **Region boundary** | WFS `VerwaltungsEinheit` `art=Bundesland` (DE03) in 25833, simplified 25 m | Ready: `ni/niedersachsen.geojson` (5,927 vertices, 2 parts, 4 holes incl. Bremen/Bremerhaven, valid). Region CRS Utm32. |
| **OSM** | Geofabrik `europe/germany/niedersachsen-latest.osm.pbf` + `niedersachsen-updates/state.txt` | 510 MB; follow the 307; includes Bremen. |
| **Attribution** | `© GeoBasis-DE/LGLN <year>, Daten geändert, CC BY 4.0` + licence link + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- That freemapper's `tiff` crate decodes the DOP JPEG tiles in practice (source read: JPEG + JPEGTables + subsampling
  supported, output YCbCr) and the DGM1 COG (tiled LZW); that geo's `check_validation` accepts the boundary asset; that
  rustls connects to all hosts (openssl without AIA says yes).
- What the DOP internal masks mark elsewhere (three tiles were 100 % valid); whether 2026 1 km tiles will replace the 2 km
  blocks retroactively; the purpose of the "Block100–600" 10 cm flights (2024-07/09, 2025-09/10).
- Why the WMS clips Bremen but not Hamburg; WMS behaviour under sustained load (≈15 GetMaps made, no failure).
- The DOM WCS and DOP WCS content; which lot the DGM WCS uses where several vintages exist; the WCS limit above 5 km.
- `Nicht_NI` ALS tiles (in buckets, not in STAC) and whether they are licensed/maintained.
- Bremen-side and Hamburg-side DGM comparisons (not done); NL border of the boundary asset; the Elbe overlap with SH (an
  official-data conflict, not resolved).
- ALKIS `nutzart`/`bez` of rarer classes (Moor, Sumpf, Tagebau, Flugverkehr, Ackerland — no farmland in the samples beyond
  Grünland) and of rural areas beyond one Harz and one Heide box.
- The ArcGIS Hub, LoD1/LoD2, ALKIS Landnutzung GeoPackage, historic DOP use, the CSW catalogue.
- Rate limits or fair-use rules (none seen, none found in the AGNB).

