# North Rhine-Westphalia (Geobasis NRW) data sources for freemapper — live-verified reference

Verified live on 2026-11-08 (Europe/Berlin), early morning (04:30–05:10 UTC). Provider: **Bezirksregierung Köln,
Abteilung 7 „Geobasis NRW"** (contact `geobasis@bezreg-koeln.nrw.de`). Open data since 2017 under **Datenlizenz
Deutschland – Zero 2.0**. Data come from six hosts, all on Apache with complete Let's Encrypt chains:

- the file server **OpenGeodata.NRW** `www.opengeodata.nrw.de/produkte/geobasis/…` (1 km tiles as single files, a
  directory listing in JSON/XML/HTML, a per-product metadata CSV in a small zip);
- the WMS host `www.wms.nrw.de/geobasis/…` (MapServer);
- the WCS host `www.wcs.nrw.de/geobasis/…` (MapServer behind an "OGC Proxy");
- the WFS host `www.wfs.nrw.de/geobasis/…` (AAA-suite XtraServer behind the same proxy);
- the OGC API host `ogc-api.nrw.de/lika/v1` (ldproxy; ALKIS vereinfacht as GeoJSON);
- the catalogue `apps.geoportal.nrw.de/soapServices/CSWStartup` (ISO metadata).

No login, registration, token, cookie or form is needed for anything below. Nothing returned 401/403.

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile / 2 km block |
|---|---|---|---|---|---|---|
| **Hildegard-von-Bingen-Gymnasium**, Leybergstraße 1, Köln-Sülz (primary) | way **56752699** | E 354704 / N 5642657 | E −66680 / N 5671681 | 50.91724 / 6.93295 | E 354608–354801, N 5642559–5642753 | `32 354 5642` (fully inside) / block E 354–356, N 5642–5644 |
| Humboldt-Gymnasium, Kartäuserwall 40, Köln-Altstadt-Süd (second sample; crosses a tile edge) | way **80359448** | E 355904 / N 5643652 | E −65399 / N 5672578 | 50.92649 / 6.94961 | E 355777–356031, N 5643553–5643751 | `32 355 5643` + `32 356 5643` |
| Hesse border probe (Siegerland, Burbach/Haiger area) | – | border point E 441213 / N 5620807 | | 50.73608 / 8.16692 | – | `32 441 5620` (12 % NRW, 88 % Hesse) |
| ALKIS value samples only: school 2 × 2 km (E 353704–355704, N 5641657–5643657), Münster centre (E 404820–406320, N 5757009–5758509), Aachen centre (E 293659–295159, N 5628142–5629642), Arnsberger Wald (E 443409–445409, N 5699172–5701172) | – | | | | | |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7° also for negative eastings. The seven ALKIS
school buildings (`funktion` "Allgemein bildende Schule") lie at E 354629–354792 / N 5642571–5642711 in 25832, inside the OSM
footprint. The WFS's own 25833 output matches my 25832 → 25833 conversion within **0.7 mm**, see §3.)

**Note on 25833: the school's easting in zone 33 is −66.7 km.** NRW spans E −139.6 km … +118.7 km in EPSG:25833;
**59.5 % of the state's area has negative 25833 eastings** (§3).

Samples (all under `nrw/` next to this file; treat them as untrusted data):

| Path | Content |
|---|---|
| `dl/hvb_dgm1/`, `dl/hvb_dom1/`, `dl/hvb_ndom50/`, `dl/hvb_dop/` | the primary school's tile `354/5642`: DGM1, DOM1 (ALS 12.02.2022), nDOM50 (bDOM 11.05.2025), DOP JP2 (11.05.2025) + `tile_0.2m.jpg`, `school_crop.jpg` |
| `dl/dgm1/`, `dl/dom1/`, `dl/ndom50/`, `dl/dop/` | the same four products for tile `355/5643` (Humboldt-Gymnasium), with response headers |
| `dl/border_dgm1/`, `dl/border_ndom/`, `dl/border_hewcs/`, `dl/border_wms/` | Hesse border: NRW DGM1 `441/5620`, `441/5619`, nDOM50 `441/5620`, Hesse's own DGM1 of `441/5620` via the HVBG WCS, NRW WMS 3 × 3 km JPEG/PNG and an image deep in Hesse |
| `dl/wms/` | DOP WMS: school 25832 and **25833** (`cmp_32_33.jpg`), native-grid PNG at 0.1 m + comparison with the JP2 (`native_0.1m.png.cmp.jpg`), 5000/5001 px probes |
| `dl/wmstime/` | five 5000 × 5000 px JPEGs (1 km² each, timing test) |
| `dl/wcs/` | WCS clips: DGM 200 m, 2 km, fractional bbox; DOM 200 m; nDOM 200 m |
| `dl/meta/`, `dl/meta_zips/` | the six `*_meta.zip` and their CSVs: `dgm1_nw.csv`, `dom1_nw.csv`, `ndom_nw.csv`, `bdom_nw.csv`, `3dm_nw.csv`, `dop_nw.csv` (one row per tile) |
| `dl/j2w/dop_j2w.zip` | world files of all DOP tiles (37,304 members, includes stale names) |
| `dl/laz_head/` | first 64 KB of the 3D-Messdaten LAZ and the bDOM50 LAZ of tile `354/5642` |
| `probe/listing/` | directory listings: `/produkte/…` folders (JSON), leaf listings `dgm1_tiff`, `dom1_tiff`, `ndom50_tiff`, `bdom50_las`, `3dm_l_las` (`index.json` + `index.xml`), `dop_jp2_f10.index.xml` |
| `probe/caps/` | capabilities: WMS DOP, WFS ALKIS, WCS DGM/DOM/nDOM/DOP, DGM DescribeCoverage |
| `probe/gfi/` | DOP GetFeatureInfo answers (plain, HTML, GML; inside and outside) |
| `probe/alkis/` | WFS answers: school (25832/25833, buildings, NutzungFlurstueck, Nutzung), four sample areas, paging, limits, error probes, border |
| `probe/ogcapi/` | OGC API landing page, collections, conformance, items (school 25832/25833, 2 km, paging), an LGB item for comparison |
| `probe/boundary/` | `VerwaltungsEinheit` answers (25833 `r1/`, 25832 `r2/`), the full state polygon as WKT (both CRS), **`nordrhein-westfalen_simplified25m.geojson`** |
| `probe/tls/` | `openssl s_client -showcerts` output per host |
| `probe/licence/`, `probe/iso/` | opengeodata listing page, product page, `lizenzbedingungen_geobasis_nrw.pdf` (+ text), ISO records of DGM1 and DOP |
| `probe/geofabrik/`, `probe/osm/` | Geofabrik state/poly/index page; Overpass answers |
| `scripts/` | helpers (UTM, listing summary, CSV summary, TIFF tags, JP2 boxes/decoding, DOP and WCS comparisons, nDSM, GML attribute summary, boundary extraction/simplification/gap checks) |

---

## 1. Brandenburg → NRW at a glance

| freemapper product | Brandenburg (today) | NRW equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** NRW lies at 5.87–9.46°E. In 25833 it spans **E −139,634 … 118,732** (59.5 % of the area negative), with −6.3° grid convergence and +3.5 m/km scale in Köln (+4.7 m/km at Aachen). WMS, WFS, OGC API and WCS reproject to 25833 on request. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **DOP10 (TrueDOP on a bDOM), RGBI, 1 km tiles as JPEG 2000 only** (10000 × 10000 px, 30–80 MB). freemapper has no JP2 decoder, so use the **WMS** `wms_nw_dop`, layer `nw_dop_rgb`: **max 5000 px**, ≈6 s per 1 km² at 0.2 m, pixel-identical to the tiles, 25833 works. Flights 2024–2026 (396 tiles still 2022), **leaf-off and leaf-on mixed by region**; the school tile is from **11.05.2025** (leaf-on). | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1** GeoTIFF Float32, **LZW, no predictor**, 1 km, single files over plain HTTP (≈2.2 MB, HEAD/Range/ETag), **no TFW**, GeoKeys 25832, NoData −9999. ALS **2020-11 … 2025-02** (all winter flights). File names carry the year → index needed. WCS `nw_dgm` byte-identical (≤ 2 km per request). | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | Two ready options: **DOM1 − DGM1** (ALS, same flight, same grid and format; leaf-off; −1 … 5 years older than the DOP) or **nDOM50** (GeoTIFF Float32 0.5 m, already normalised; image-based bDOM − DGM1; its bDOM date equals the current DOP date for 52 % of tiles). Also open: bDOM50 LAZ (0.5 m grid as points, like LGB's nDOM LAZ) and the raw ALS point clouds (3D-Messdaten, ≈10 pts/m², 80 MB median per km²). | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | Two services with the same data: **ALKIS-vereinfacht WFS 2.0** `wfs_nw_alkis_vereinfacht`, `ave:GebaeudeBauwerk` (AAA-suite like Hesse/Saxony, GML 3.2, also GeoPackage), and an **OGC API Features** `ogc-api.nrw.de/lika/v1/collections/gebaeude_bauwerk` (GeoJSON, 25833, `limit` ≤ 10000, `next` links) whose properties lack `oid` (it is only the feature `id`) and say `fktkurz` instead of `gfkzshh`. LGB spellings ("Gebäude"), `rellage`, **detailed `funktion`** ("Allgemein bildende Schule"). | verified |
| Land use | OGC API `nutzung_flurstueck` | WFS **`ave:NutzungFlurstueck`** / OGC API `nutzung_flurstueck` (parcel split, like LGB). **`ave:Nutzung` has `nutzart` in NRW** (unlike Hesse). LGB spellings; `Straßenverkehr` has `bez` "Begleitfläche Straßenverkehr" on 19 % of parcels (never "Fahrbahn"); 23 % of `Platz` without `bez`. | verified |
| Currency | `bb_*_aktualitaet.csv` | **per-product `*_meta.zip`** (CSV, one row per tile: exact file name incl. year, flight/ALS date, camera, leaf state …; 110–300 KB zipped), DOP WMS GetFeatureInfo `nw_dop_utm_info` (`Bildflugdatum`), ETag/Last-Modified on every file, `aktualit` per ALKIS feature | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `nordrhein-westfalen-latest.osm.pbf` = **917.1 MB** (2.6 × Hesse). Five Regierungsbezirk extracts (137–225 MB) exist. | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **dl-de/zero-2-0**: "Jede Nutzung ist ohne Einschränkungen oder Bedingungen zulässig." **No attribution and no change note required** (the ISO licence JSON has `"quelle": ""`). | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **File server** | `https://www.opengeodata.nrw.de/produkte/geobasis/<theme>/<product>/<product>/` | DGM1 (`hm/dgm1_tiff/dgm1_tiff/`), DOM1 (`hm/dom1_tiff/dom1_tiff/`), nDOM50 (`hm/ndom50_tiff/ndom50_tiff/`), bDOM50 LAZ (`hm/bdom50_las/bdom50_las/`), ALS points (`hm/3dm_l_las/3dm_l_las/`), DOP (`lusat/akt/dop/dop_jp2_f10/`) | Apache; static files; **HEAD 200 with Content-Length**, `Accept-Ranges: bytes` (Range → 206), `ETag`, `Last-Modified`, `Content-Disposition: attachment`, CORS `*`. Wrong name → **404** `text/html`. Note the doubled product folder. |
| Directory listings | same host, folder URL (content negotiation, `TCN: choice`) or explicit `index.json` / `index.xml` / `index.html` | folder trees and per-leaf file lists | default answer is **JSON** (`Content-Location: index.json`); `Accept: application/xml` → **406** (the XML variant is `text/xml`); ask for `index.json` or `index.xml` explicitly. No gzip. Leaf listings 3.3–3.8 MB, ≈3 s. |
| **DOP WMS** | `https://www.wms.nrw.de/geobasis/wms_nw_dop` | `nw_dop_rgb`, `nw_dop_cir`, `nw_dop_nir`, `nw_dop_utm_info` (queryable) | MapServer; **MaxWidth/Height 5000** |
| **WCS** | `https://www.wcs.nrw.de/geobasis/wcs_nw_{dgm,dom,ndom,dop}` | coverages `nw_dgm`, `nw_dom`, `nw_ndom`, `nw_dop` | WCS 2.0.1, MapServer behind an "OGC Proxy"; **≈2 km per axis max** (§5) |
| **ALKIS WFS** | `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht` | `ave:Flurstueck`, `ave:FlurstueckPunkt`, `ave:GebaeudeBauwerk`, `ave:KatasterBezirk`, `ave:Nutzung`, `ave:NutzungFlurstueck`, `ave:VerwaltungsEinheit` | AAA-suite XtraServer (`Content-Disposition: inline; filename="XtraServerGetFeature.xml"`, header `X-Balancer-Status`) |
| **ALKIS OGC API** | `https://ogc-api.nrw.de/lika/v1` | collections `flurstueck`, `flurstueck_punkt`, `gebaeude_bauwerk`, `katasterbezirk`, `nutzung`, `nutzung_flurstueck`, `verwaltungseinheit` | ldproxy (profiles `def.ldproxy.net`), HTTP/2, GeoJSON/JSON-FG/FlatGeobuf/HTML, CQL2 filter, vector tiles |
| ISO metadata | `https://apps.geoportal.nrw.de/soapServices/CSWStartup?Service=CSW&Request=GetRecordById&Version=2.0.2&outputSchema=http://www.isotc211.org/2005/gmd&elementSetName=full&id=<uuid>` | per dataset | DGM1 `0c6796e5-9eca-4ae6-8b32-1fcc5ae5c481` (dateStamp 2026-02-20), DOP `56fb584b-10cf-4009-a405-0bef06bb3e00` (2026-06-30); 0.15–0.2 s |
| Product pages | `https://www.bezreg-koeln.nrw.de/geobasis-nrw/produkte-und-dienste/…` | descriptions, licence PDF | the DGM page lists the DGM hillshade/height-layer WCS too (not examined) |

Not used / not examined: the "Open Data Downloadclient der GDI-NW" (`www.geoportal.nrw/?activetab=map&openDownloadclient=true`,
custom areas), historic DOP (`lusat/hist/hist_dop/hist_dop_jp2_f10/`), InVeKoS DOP (`lusat/akt/idop/idop_jp2_f2/`), 3D-Mesh,
ALKIS NAS bulk (`lk/akt/`), DVG (`vkg/dvg/`). No `wcs_nw_bdom` (404).

### 2.1 Tile scheme

- **1 km × 1 km, EPSG:25832, SW corner at whole km**, the same tile ids for every product. There is no 2 km product.
- File names (the **year suffix is part of the name and differs per tile**, so names must come from an index):
  - DGM1: `dgm1_32_EEE_NNNN_1_nw_YYYY.tif` (YYYY = ALS year, e.g. `dgm1_32_354_5642_1_nw_2022.tif`)
  - DOM1: `dom1_32_EEE_NNNN_1_nw_YYYY.tif` (same year as the DGM1 for every tile)
  - nDOM50: **`ndom50_32EEE_NNNN_1_nw_YYYY.tif`** (no underscore after `32`; YYYY = bDOM year)
  - bDOM50: **`bdom50_32EEE_NNNN_1_nw_YYYY.laz`**
  - DOP: `dop10rgbi_32_EEE_NNNN_1_nw_YYYY.jp2` (YYYY = flight year), world files in `dop_j2w.zip`
  - ALS points: `3dm_32_EEE_NNNN_1_nw.laz` (no year)
- Grid: SW corners **E 280 … 531, N 5576 … 5821**, **35,860 tiles** for every product. That is **every 1 km square that
  touches NRW plus 839 squares up to 1.4 km outside** (checked against the ALKIS state polygon; none missing).
- **Tiles are complete squares, not clipped:** the border tile `441/5620` (88 % Hesse) has DGM1, nDOM50 and DOP over the full
  square without NoData (§5, §4.2).
- The DOP and bDOM50 listings carry **stale duplicates**: 756 DOP tiles and 1,196 bDOM50 tiles appear twice, with a `_2023`
  file (timestamp 2024-03) beside the current `_2025` file. The metadata CSV lists only the current one.

### 2.2 Finding tile names and dates

Two machine-readable indexes per product, both on the file server:

| Index | Size | Time | Content |
|---|---|---|---|
| **`<product>_meta.zip`** (e.g. `…/dgm1_tiff/dgm1_tiff/dgm1_meta.zip`) | 111,742 B (DGM1), 111,744 (DOM1), 206,732 (nDOM50), 298,283 (bDOM50), 110,500 (3DM), 280,193 (DOP) | 0.2–0.35 s | one CSV (`dgm1_nw.csv` 3.98 MB …), **one row per tile, the exact file name without extension** + dates (§7) |
| leaf listing `index.json` | 3.4–3.8 MB | ≈3 s | `{"metafiles":[{name,size,timestamp}],"datasets":[{"name":"dgm1_tiff_kacheln","title":…,"files":[{"name":"dgm1_32_280_5652_1_nw_2022.tif","size":"2393341","timestamp":"2023-05-22T08:43:46"},…]}]}` (sizes are **strings**). Last-Modified/ETag on the listing (`dgm1`: Tue, 03 Nov 2026 21:15:11 GMT, `"35a122-65f65c1ff6abb"`) |
| leaf listing `index.xml` | same | ≈3 s | `<opengeodata><metafiles><metafile name size timestamp/>…<datasets><dataset name title><files><file name size timestamp/>` (DOP also has `<geometafiles>` with `dop_j2w.zip`) |

- The CSV names match the listing exactly for DGM1, DOM1, nDOM50 and 3DM (0 differences); for DOP and bDOM50 the listing has
  the extra stale `_2023` files (§2.1).
- **Recommended:** fetch `<product>_meta.zip` (≈110–300 KB), map tile → `Kachelname` (+ `.tif`/`.jp2`/`.laz`) and date;
  cache it by ETag. The zip date in the listing (`2026-11-03T22:05:03` for DGM1) shows that it is rebuilt with the data.
- Tile URL example (school):
  ```
  HEAD https://www.opengeodata.nrw.de/produkte/geobasis/hm/dgm1_tiff/dgm1_tiff/dgm1_32_355_5643_1_nw_2022.tif
  → 200, Content-Length: 2051153, Last-Modified: Mon, 17 Jul 2023 18:14:01 GMT, ETag: "1f4c51-600b2c34c4c40",
    Accept-Ranges: bytes, Content-Type: image/tiff, Content-Disposition: attachment
  GET  … Range: bytes=0-99 → 206, Content-Length: 100
  GET  …/dgm1_32_354_5642_1_nw_2021.tif (wrong year) → 404 text/html
  ```

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DGM1/DOM1/nDOM50 GeoTIFF: GeoKeys `ProjectedCSType 25832`, GeoAscii "ETRS89 / UTM zone 32N|ETRS89|", `GTRasterType 1`.
  - DOP JP2: GMLJP2 `srsName="urn:ogc:def:crs:EPSG::25832"` and a GeoJP2 box; CSV `Koordinatenreferenzsystem_Lage 25832`.
  - LAZ: GeoKey VLR (34735).
  - WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`; OGC API `storageCrs …/EPSG/0/25832`; WCS native 25832.
- **Height datum DHHN2016** normal heights, quasigeoid GCG2016: CSV `Koordinatenreferenzsystem_Hoehe DE_DHHN2016_NH`,
  `Hoehenanomalie DE_AdV_GCG2016_QGH`; DOP/bDOM CSV `7837`. No vertical GeoKey in the TIFFs. The same as the other freemapper
  states.
- **Zone 33 is a very poor fit:**

  | Place | Convergence zone 32 / 33 | Scale k zone 32 / 33 | 25833 easting |
  |---|---|---|---|
  | Köln (school) | −1.60° / −6.28° | 0.99986 / 1.00354 | −66.7 km |
  | Aachen west edge (5.87°E) | −2.42° / −7.09° | 1.00020 / 1.00469 | −143.5 km |
  | Höxter east edge (9.45°E) | +0.35° / −4.37° | 0.99961 / 1.00140 | +117.2 km |

  NRW maps must be **25832 projects** (`Region::crs() → Crs::Utm32`, like Saxony-Anhalt, Hamburg, Hesse and SH). Any 25833
  tile arithmetic (`TileId` is `u32`) must never run for NRW: the asset itself has eastings down to −139,634.
- **Extent** of the state polygon (`VerwaltungsEinheit` DE05, `aktualit 2026-09-30Z`, 72,228 vertices, 6 polygons, no holes):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 280375 – 531799, N 5577679 – 5820212 |
  | EPSG:25833 | E −139634 – 118732, N 5608984 – 5839628 |
  | lon / lat | lon 5.8663 – 9.4613, lat 50.3227 – 52.5315 |

  Area 34,096.7 km² in 25832 (official 34,112 km²), 34,312.6 km² in 25833 (zone 33 scale error).
- 25833 availability (all verified):
  - DOP WMS: root and layers list 25833 (also 25831, 28992, 3857, 4258 …); GetMap in 25833 at the school returned the
    expected, rotated image (`dl/wms/cmp_32_33.jpg`), 1.1 s.
  - ALKIS WFS: `OtherCRS` 25833 (+ 19 more); the 25833 output of the state polygon matches my 25832 → 25833 transform within
    **0.7 mm** (every 50th of 70,211 vertices compared). The service abstract warns that only 25832 coordinates are official.
  - OGC API: `crs=…/EPSG/0/25833` and `bbox-crs=…/25833` work (152 buildings at the school, 0.33 s).
  - WCS: lists many CRS; output in 25833 not tested (would resample, as in Hesse).

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- WMS abstract: "liegen flächendeckend vor und werden in einem **2-jährigen Zyklus** erneuert … Bodenauflösung von **10cm/Pixel**
  … 4-Kanal Multispektralbilder … **RGBI** … Mit dem Bildflug 2018 wurde die Prozessierung des DOP als **True Orthophoto**
  eingeführt". Every current tile has `Bezugsflaeche bDOM`.
- The current mosaic (`dop_nw.csv`, "Stand 2026-10-21", 35,860 rows) by flight month:

  | Flight month | Tiles | Leaf state code (`Belaubungszustand`) |
  |---|---|---|
  | 2022-06 | 396 (flight 1387/22 Herford Detmold) | 3 |
  | 2024-03 / 04 / 05 / 06 / 08 / 09 | 1,652 / 1,756 / 2,624 / 1,956 / 2,912 / 896 | 1 in March–April; **both 1 and 3 in June–Sept** (939 tiles coded 1 in summer) |
  | 2025-03 / 04 | 5,464 / 2,036 | 1 (196 coded 3) |
  | 2025-05 … 10 | 1,124 / 4,040 / 1,292 / 1,188 / 792 / 464 | 3 |
  | 2026-02 / 03 / 04 / 05 | 528 / 4,680 / 1,260 / 800 | 1, 1, 1/2/3, 3 |

  So the state is a patchwork of 2024, 2025 and 2026 flights (plus 396 tiles from 2022), **leaf-off (Feb–April) and leaf-on
  (May–Oct) by region**. The code meaning 1 = unbelaubt, 2 = teilbelaubt, 3 = belaubt is my reading of the months, and the
  summer tiles coded 1 make it unreliable; use the flight month.
- The school tile `354/5642`: flight **11.05.2025** (1450/25 Köln Bonn, UltraCam Eagle 4.1), code 3; visually full foliage
  (`dl/hvb_dop/school_crop.jpg`), true orthophoto (no building lean).
- Only the current DOP is served by the WMS. Historic DOPs are a separate product (not examined).

### 4.2 WMS (recommended path)

- `https://www.wms.nrw.de/geobasis/wms_nw_dop`, WMS 1.3.0, title "WMS NW DOP", MapServer.
  - Layers: **`nw_dop_rgb`** ("DOP Farbe"), `nw_dop_cir`, `nw_dop_nir`, `nw_dop_utm_info` ("DOP Metadaten", queryable).
    No scale limits on `nw_dop_rgb`, no TIME dimension.
  - Formats: png, jpeg, tiff, png 8 bit. GetFeatureInfo: text/html, application/vnd.ogc.gml, text/plain.
  - **MaxWidth/MaxHeight 5000** (1 km at 0.2 m, 500 m at 0.1 m). 5001 px → **HTTP 200 `text/xml`**:
    `msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be between 1 and 5000 pixels.`
    freemapper's usual `max_px: 4000` fits.
- Example (school, 25832, 0.2 m):
  ```
  https://www.wms.nrw.de/geobasis/wms_nw_dop?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&STYLES=&LAYERS=nw_dop_rgb&CRS=EPSG:25832&BBOX=354504,5642507,354904,5642807&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 593,899 B, **1.04 s**. In 25833 (`CRS=EPSG:25833&BBOX=-66880,5671531,-66480,5671831`): 600,608 B, 1.10 s.
- Speed: 5000 × 5000 px JPEG (1 km² at 0.2 m): **5.9, 6.0, 6.2, 6.3, 6.6 s and once 21.8 s** (Münster), 4.6–5.4 MB each
  (≈1 MB/s, which is the speed of this test line; Geofabrik gave the same). No hang, no error in ≈20 GetMaps. PNG at 5000² px:
  64.7 MB, 57 s (avoid).
- JPEG quality: IJG **q 75** (quantisation tables).
- **The WMS returns the tile's pixels:** native grid PNG at 0.1 m (200 × 200 m, 2000 px) against the full-resolution JP2 decode:
  mean abs difference **3.0** (0–255) at offset 0, 4.7–6.0 at ±1 px; mean luminance 126.1 vs 125.9.
- Outside the tile set: JPEG **white** (996 B for a 400 × 300 image in Frankfurt), PNG with `TRANSPARENT=TRUE` transparent.
  The border probe `440000–443000 / 5619000–5622000` shows imagery to the east edge of the NRW tiles (E 442000) — far into
  Hesse — and white beyond (33.3 % white = the 442 column; `dl/border_wms/border_small.jpg`).
- Headers: chunked, **no Content-Length**, no ETag/Last-Modified, `Vary: Origin`.
- **GetFeatureInfo gives the flight date per tile:**
  ```
  https://www.wms.nrw.de/geobasis/wms_nw_dop?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=nw_dop_utm_info&QUERY_LAYERS=nw_dop_utm_info&STYLES=&CRS=EPSG:25832&BBOX=354604,5642557,354804,5642757&WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain
  → Layer 'nw_dop_utm_info'
      Feature 20301:
        Kachelname = '32354_5642'
        Bildflugdatum = '11.05.2025'
        Bodenauflösung = '0.10'
        Photometrie = 'RGBI'
        Bit_pro_Kanal = '8'
  ```
  0.16–0.23 s. The GML variant adds the tile box (`354000,5642000 355000,5643000`). Outside the tiles: "Search returned no
  results." The date agrees with the CSV (`2025-05-11`).

### 4.3 Tile downloads (JPEG 2000)

- `https://www.opengeodata.nrw.de/produkte/geobasis/lusat/akt/dop/dop_jp2_f10/dop10rgbi_32_354_5642_1_nw_2025.jp2`
  → 200, 33,172,901 B (the 355/5643 tile 32,550,024 B in 53 s alone, 0.61 MB/s; in parallel with three others 47 s).
  Listing: 36,616 files (35,860 current), median 31.4 MB, max 81.1 MB, **1.17 TB** in total.
- File (school tile, verified):
  - JP2 boxes: `ihdr` 10000 × 10000, **4 components (RGBI)**, 8 bit; `asoc` with **GMLJP2** (`RectifiedGrid`, origin
    `355000.05 5643999.95` = pixel centre, offset vectors 0.1/−0.1, envelope 355000 5643000 … 356000 5644000); a **GeoJP2**
    `uuid` box; one codestream with **a single 10000 × 10000 image tile**, RPCL order, 1 quality layer, 8 resolution levels,
    64 × 64 code blocks. CSV `Komprimierung "GDAL_JP2ECW, 90"`.
  - World file in `dop_j2w.zip` (6.9 MB, 37,304 members, also stale `…_2021.j2w` names): `0.1 / 0 / 0 / -0.1 / 355000.05 /
    5643999.95` (pixel centres; the LGB JGW convention at 0.1 m).
- Decoding (Pillow/OpenJPEG) at `reduce=1` (5000 × 5000, 0.2 m): 5.5–5.7 s, 0.85 GB peak memory. The single image tile means
  no partial (range) decode; a reduced resolution is possible.
- **freemapper has no JPEG 2000 decoder** (no `jpeg2k`/`openjpeg` dependency). Not recommended; the WMS gives the same pixels.

### 4.4 WCS `nw_dop`

- `https://www.wcs.nrw.de/geobasis/wcs_nw_dop?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=nw_dop&FORMAT=image/tiff&SUBSET=x(…)&SUBSET=y(…)`
  → uncompressed 4-band uint8 TIFF at 0.1 m (1000 × 1 m strip: 400,840 B). A 1 km² tile would be 400 MB. 1000 m width OK,
  2000 m → 400 (§5). Not examined further; the WMS is the better path.

---

## 5. DGM1 (terrain model, 1 m)

- Product page: "Als Datengrundlage werden die aus dem Ergebnis des flugzeuggestützten Laserscanning (Airborne Laserscanning,
  ALS) gewonnenen 3D-Messdaten verwendet. Die Höhenwerte des DGM beziehen sich auf die Rasterelementpositionen. Diese befinden
  sich im Zentrum der 1 x 1 m² Rasterelemente." **"Die Laserdatenerhebung in NRW wurde für den Erhebungszeitraum Ende 2025 –
  Anfang 2026 ausgesetzt. Ab dem kommenden Erhebungszeitraum Ende 2026 – Anfang 2027 wird sie mit einer erhöhten Punktdichte
  von mind. 8 Pkt./m² fortgeführt."** So the DGM1 will not get newer than February 2025 before spring 2027.
- ALS dates (`dgm1_nw.csv`): 2020 1,532 tiles, 2021 7,142, 2022 8,770, 2023 5,004, 2024 7,772, 2025 5,640; months Nov–Mar only
  (**leaf-off**). School tiles 354/5642 and 355/5643: **12.02.2022**; border tile 441/5620: 02.03.2021.
- **Tile download:** `https://www.opengeodata.nrw.de/produkte/geobasis/hm/dgm1_tiff/dgm1_tiff/dgm1_32_354_5642_1_nw_2022.tif`
  → 200, 1,770,350 B (alone: 2.05 MB in 1.8 s). Sizes 0.6–4.0 MB, median 2.16 MB, 78.8 GB in total.
- TIFF (verified on 354/5642, 355/5643, 441/5620, 441/5619):
  - 1000 × 1000 **Float32** (SampleFormat 3), **Compression 5 (LZW), Predictor 1 (none)**, **500 strips of 2 rows**, IFD at
    the **start** (offset 8);
  - ModelPixelScale 1, **ModelTiepoint exactly (354000, 5643000)** (NW corner), GTRasterType 1 (PixelIsArea),
    ProjectedCSType 25832, GeoAscii "ETRS89 / UTM zone 32N|ETRS89|", **`GDAL_NODATA` "-9999"**;
  - ImageDescription "Gelaendemodell, Rasterweite 1 Meter, 32_355_5643", Software "LAStools is a product of rapidlasso GmbH,
    Germany", Copyright (315) "Bezirksregierung Koeln, Geobasis NRW", DateTime 2023-07/2023-10 (build date);
  - **no TFW**. The registration is the LGB/GeoSN/HVBG one (tiepoint at the NW corner, value at the pixel centre).
  - Pillow decodes it; freemapper's `tiff` crate reads LZW Float32 without predictor (the simplest case; not run).
- Values: school tile 47.10–56.87 m, centre **51.16 m**; no NoData in any of the four tiles read.
- **Border:** tile 441/5620 (88 % in Hesse) is complete (0 % NoData, 337.15–417.81 m). Against Hesse's DGM1 for the same
  square (HVBG WCS `he_dgm1`): 1,000,000 common pixels, mean Δ +2.3 cm, mean |Δ| 4.4 cm, p1/p99 −0.19/+0.19 m — the same
  terrain from different flights, so a cross-border DGM can be taken from either state.
- **WCS:** `https://www.wcs.nrw.de/geobasis/wcs_nw_dgm`, coverage **`nw_dgm`**, axis labels `x y`, grid origin
  (278000.5, 5827999.5), envelope 278000 5560000 … 536000 5828000, formats tiff/png/jpeg/…, interpolation NEAREST/AVERAGE/BILINEAR.
  ```
  https://www.wcs.nrw.de/geobasis/wcs_nw_dgm?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=nw_dgm&FORMAT=image/tiff&SUBSET=x(354600,354800)&SUBSET=y(5642500,5642700)
  ```
  → 200 `image/tiff`, 57,991 B, 0.27 s; LZW Float32, tiepoint exactly (354600, 5642700), GDAL_NODATA −9999. **Byte-identical
  to the tile** (max |Δ| 0.000; a 1 px shift gives 1.8–2.9 m). 2 × 2 km: 7.97 MB in 7.9 s.
  - **Size limit:** an axis of more than ≈2 km → **HTTP 400** `text/xml` `OGC Proxy: InvalidParameterValue`, no reason given.
    Deterministic: DGM x 2,020 m → 200, 2,021/2,030 m → 400 (repeated twice); nDOM 2,005 m → 200, 2,010 m → 400; DOP 1,000 m
    → 200, 2,000 m → 400. 2,000 × 2,000 m DGM → 200. Request ≤ 2 km pieces (or 1 km tiles).
  - **A fractional bbox is resampled**, not snapped: `x(354600.3,354800.7)` → 200 px of 1.002 m, tiepoint 354600.3. Request
    whole metres.

---

## 6. nDSM sources: DOM1, nDOM50, bDOM50, LiDAR

### 6.1 DOM1 (1 m, ALS)

- `…/hm/dom1_tiff/dom1_tiff/dom1_32_354_5642_1_nw_2022.tif` → 2,436,575 B. Format identical to DGM1 (LZW, no predictor,
  2-row strips, tiepoint (354000, 5643000), GDAL_NODATA −9999), ImageDescription "Oberflaechenmodell, Rasterweite 1 Meter, …".
  Values 46.53–99.99 m. 85.1 GB in total.
- **DOM1 and DGM1 have the same date and the same year in the name for all 35,860 tiles** (checked in the CSVs).
- **DOM1 − DGM1** over the school footprint bbox (37,442 px): min −4.65, p10 0.00, **median 3.17**, p90 14.51, max 27.75 m;
  52.7 % > 2 m. Clamp < 0. Leaf-off ALS (Feb 2022), so deciduous crowns are thin; DOP is May 2025.
- WCS `nw_dom` is byte-identical to the tile (max |Δ| 0.000 on 200 × 200 m; uncompressed TIFF, no NoData tag), unlike
  Hesse's WCS DOM.

### 6.2 nDOM50 (0.5 m, image-based, already normalised)

- `…/hm/ndom50_tiff/ndom50_tiff/ndom50_32354_5642_1_nw_2025.tif` → 16,024,810 B (every tile 16.02–16.04 MB, **574.7 GB** in
  total), 34 s in parallel / 14 s alone.
- TIFF: 2000 × 2000 Float32, **uncompressed**, 1-row strips, **IFD at the end** (offset 16,012,378), ModelPixelScale 0.5,
  tiepoint (354000, 5643000), GeoKeys 25832, GDAL_NODATA −9999, Copyright "(c) Geobasis NRW", DateTime = bDOM date
  (2025:05:11). QGIS style file in the listing.
- Meaning (`ndom_nw.csv`: `Kachelname;Aufnahmedatum_DGM;Aufnahmedatum_bDOM;…;Aufloesung_cm`): bDOM (image matching of the DOP
  flight) minus DGM1. School tile: DGM 2022-02-12, bDOM **2025-05-11 = the DOP date**.
- **Not clamped:** −5.72 … 69.37 m in the school tile. Over the school bbox (149,768 px): p10 0.11, **median 3.81**, p90 14.91,
  max 28.69; 55.2 % > 2 m. Against DOM1 − DGM1 over the whole tile (nDOM averaged to 1 m): median +0.18 m, mean |Δ| 1.20 m,
  p5/p95 −2.21/+4.20 m (leaf-on crowns are higher in the nDOM).
- Dates: the nDOM's bDOM date equals the **current DOP date for 18,484 of 35,860 tiles**; 2,408 lag one year, 14,772 two
  years, 196 four years (the nDOM is rebuilt later than the DOP). Its DGM date always equals the DGM1 date.
- WCS `nw_ndom` byte-identical to the tile (0.5 m, uncompressed).

### 6.3 bDOM50 LAZ and ALS point clouds

- **bDOM50** `…/hm/bdom50_las/bdom50_las/bdom50_32354_5642_1_nw_2025.laz` (15.8 MB; median 14.2 MB, 525 GB total): LAS 1.2,
  point format 2 (RGB), laszip, **4,000,000 points = a 0.5 m grid as points** (x 354000.25 … 354999.75), VLR "tile without
  buffer". Dates = DOP dates (`bdom_nw.csv`, SURE 2025.2.1 or ArcGIS Reality). This is the format of LGB's nDOM LAZ, but
  not normalised.
- **3D-Messdaten Laserscanning** `…/hm/3dm_l_las/3dm_l_las/3dm_32_354_5642_1_nw.laz` (56.4 MB; median 79.6 MB, max 553 MB,
  **3.49 TB** total): LAS 1.2, point format 1, 10,131,111 points per km² at the school, classes 1, 2, 9, 14, 17, 18, 20, 24,
  26 (CSV header `Punktklassenbelegung`). Same dates as the DGM1. Not needed for freemapper.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **`dgm1_meta.zip` → `dgm1_nw.csv`** (CRLF, ASCII) | DGM1 | 5 header lines (`Aktualitaet_Kachelinformationen;2026-10-26`, `Version_Standard;3.3`), then `Kachelname;Aktualitaet;Erfassungsmethode;Fortfuehrung;Fortfuehrungsmethode;Genauigkeit;Koordinatenreferenzsystem_Lage;Koordinatenreferenzsystem_Hoehe;Hoehenanomalie`, e.g. `dgm1_32_354_5642_1_nw_2022;2022-02-12;5020;2022-02-12;5020;0.2;ETRS89_UTM32;DE_DHHN2016_NH;DE_AdV_GCG2016_QGH`. ISO dates. |
| `dom1_meta.zip` → `dom1_nw.csv` | DOM1 | same layout (`Version_Standard;1.2`) |
| `ndom50_meta.zip` → `ndom_nw.csv` | nDOM50 | no preamble; `Kachelname;Aufnahmedatum_DGM;Aufnahmedatum_bDOM;Koordinatenursprung_Rechts;Koordinatenursprung_Hoch;Aufloesung_cm` |
| **`dop_meta.zip` → `dop_nw.csv`** (UTF-8) | DOP | preamble `Stand;2026-10-21`, `Version Regelwerk;V4.0`; columns `Kachelname;Erfassungsmethode;Aktualitaet;Bildflugnummer;Kamera_Sensor;Bodenpixelgroesse;Spektralkanaele;Koordinatenreferenzsystem_Lage;Koordinatenreferenzsystem_Hoehe;Bezugsflaeche;Koordinatenursprung_East;Koordinatenursprung_North;Anzahl_Spalten;Anzahl_Zeilen;Farbtiefe;Standardabweichung;Dateiformat;Hintergrund;Quelldatenqualitaet;Kompression;Komprimierung;Belaubungszustand;Bemerkungen`; `Aktualitaet` = flight date (ISO) |
| `bdom50_meta.zip` → `bdom_nw.csv` | bDOM50 | like the DOP CSV plus software, LAS version |
| WMS GetFeatureInfo `nw_dop_utm_info` | DOP | `Bildflugdatum` DD.MM.YYYY per tile, 0.2 s |
| ETag / Last-Modified | every file, listing and meta zip | real file dates (DGM1 355/5643: Mon, 17 Jul 2023 18:14:01 GMT) — usable for change detection |
| ALKIS `aktualit` | per feature | WFS `YYYY-MM-DDZ`, OGC API `YYYY-MM-DDThh:mm:ssZ`. Samples: buildings 2023 2,618, **2024 12,076**, 2025 1,518, 2026 966 (2024 looks like a migration date); land-use parcels 2023–2026 |

Gap between DOP and ALS (DOP year − DGM1/DOM1 year, all tiles): −1 (ALS newer): 3,996; 0: 3,772; 1: 6,328; 2: 6,276;
3: 8,378; 4: 4,248; 5: 2,862.

Suggested mapping:
- orthophoto `currency` = DOP `Aktualitaet` of the tile(s) under the map (CSV or GetFeatureInfo); `published` = ETag/date of
  `dop_meta.zip` (WMS path);
- DGM1/DOM1 `currency` = `Aktualitaet` from the DGM1 CSV (the same for both); nDOM50 = its `Aufnahmedatum_bDOM`.

---

## 8. ALKIS: buildings and actual land use

### 8.1 WFS

- `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht`, title "WFS NW ALKIS Vereinfacht", WFS 2.0.0 (and 1.1.0).
  - Types (§2). DefaultCRS 25832; OtherCRS 25831, **25833**, 28992, 3034, 3035, 3043–3045, 31466, 31467, 3857, 4258, 4326,
    4647, 5649–5653.
  - Output: GML 3.1/3.2 (default `application/gml+xml; version=3.2.1`), `application/x-zip-shapefile`,
    **`application/geopackage+sqlite3`** (verified: 102 KB GeoPackage for a 100 m bbox). **No GeoJSON.**
  - Constraints: `ImplementsResultPaging FALSE`, `ImplementsSorting TRUE`, no `CountDefault`.
  - Elements in the default namespace `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/2.0`
    (`<GebaeudeBauwerk gml:id="…"><oid>…`), schema location `https://www.wfs.nrw.de/aaa-suite/schema/…` — the same as Hesse.
- freemapper's `alkis_wfs` request shape works unchanged:
  ```
  https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=354504,5642507,354904,5642807,urn:ogc:def:crs:EPSG::25832&SORTBY=oid&COUNT=10000&STARTINDEX=0
  ```
  → 200, 155 features, 159,589 B, **8.7 s (first request; later ones 0.3–0.4 s)**. `RESULTTYPE=hits` → `numberMatched="155"`
  (1.9 s); the results say `numberMatched="unknown"`.
- Speed (no gzip: `Vary: Accept-Encoding` but never `Content-Encoding`; `Accept-Ranges: none`): 2 × 2 km at the school:
  7,352 buildings 7.3 MB **6.9 s**, 5,284 NutzungFlurstueck 6.4 MB 6.5 s, 1,463 Nutzung 2.4 MB 2.5 s. Aachen 1.5 × 1.5 km:
  6,318 buildings 9.9 s. 5 × 5 km Köln: hits 41,697 (5.9 s); `COUNT=25000` → 25,000 (23.6 MB, 24 s); no COUNT → all 41,697
  (40.9 MB, 36.6 s). No server cap seen.
- Paging: `COUNT=1000&SORTBY=oid` with `STARTINDEX=0/1000/7000` → 1000/1000/352, disjoint, ordered case-insensitively (as in
  Hesse).
- 25833: `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833` → 157 features at the school (rotated bbox), 0.36 s, E N order.
- **Errors are generic:** unknown type, unsupported SRS (`EPSG::2056`), `OUTPUTFORMAT=application/json` all → **HTTP 400
  `text/xml`** `<ows:ExceptionText>OGC Proxy: InvalidParameterValue</ows:ExceptionText>` with no detail.
- Outside NRW (Frankfurt): `numberMatched="0"`. Border tile 441/5620: 0 buildings, 18 land-use parcels (the NRW part).
- Geometries are `MultiSurface` of `Polygon`s with `posList` (srsDimension on the MultiSurface), whole features across the bbox.

### 8.2 OGC API Features (`ogc-api.nrw.de/lika/v1`)

- ldproxy; conformance Features Part 1/2/3, CQL2, JSON-FG, tiles. Licence link dl-de/zero-2-0.
  ```
  https://ogc-api.nrw.de/lika/v1/collections/gebaeude_bauwerk/items?f=json&bbox=354504,5642507,354904,5642807&bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25832&crs=http://www.opengis.net/def/crs/EPSG/0/25832&limit=10000
  ```
  → 200 `application/geo+json`, 155 features (= WFS), 122 KB, 5.5 s (cold; 0.3 s warm). Headers `OGC-NumberMatched`,
  `OGC-NumberReturned`, `Content-Crs`; body `numberMatched`/`numberReturned`. **gzip works** (2 km: 5.47 MB → 0.79 MB, 4.4 s).
- **`limit` is capped at 10000** (`limit=50000` on 5 km → 10,000 features + a `next` link with `offset=10000`); `limit=1000`
  → `next` `…&limit=1000&offset=1000`. The request shape is freemapper's LGB one (`f=json&bbox&bbox-crs&crs&limit`).
- **Property differences to LGB's OGC API (same ldproxy):**
  - **no `oid` property** — only the feature `id` (`DENW37AL1000HpNiBL`); freemapper's OGC API reader takes
    `properties.oid` (`sources/alkis.rs:344, 402`) and would drop every NRW feature;
  - `fktkurz` instead of `gfkzshh`; `aktualit` with time; extra `flurstref.N.title/href`, `bauteilref.N.*`,
    `gebaeuderef.N.*` (flattened links).
  - Otherwise the same names and values as the WFS (`gebnutzbez`, `funktion`, `rellage`, `name`, `lagebeztxt`, `nutzart`,
    `bez`, `flstkennz`).

### 8.3 Attributes compared with LGB

- `GebaeudeBauwerk`: `oid`, `aktualit`, `gebnutzbez`, `funktion`, `gmdschl` (always), `gfkzshh` (85 %), `lagebeztxt` (50 %),
  `name` (0.8 %), `rellage` (0.6 %). No `anzahlgs` in the samples. **LGB spellings** ("Gebäude").
- `NutzungFlurstueck`: `oid` (34 characters = Nutzung oid + Flurstück oid), `aktualit`, `nutzart`, `flstkennz` (always),
  `bez` (47 %), `name` (18 %).
- **`Nutzung` has `nutzart`** in NRW (4,863 of 4,863), with the same values; it is the unsplit land-use object.
  `ave:NutzungFlurstueck` stays the better match for LGB.
- oids start with **`DENW`** (`DENW37AL…` Köln, `DENW39AL…` Aachen, `DENW49AL…` Münster, `DENW11AL…` Arnsberger Wald;
  `…BL` buildings, `…TF` parcel land use), so `Region::of_object_id` gets a `"DENW"` arm.

### 8.4 Sample features at the Hildegard-von-Bingen-Gymnasium (WFS, geometry elided)

```json
{"oid":"DENW37AL1000H0F1BL","aktualit":"2024-10-19Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","gmdschl":"05315000","lagebeztxt":"Leybergstr. 1"}
{"oid":"DENW37AL1000GvzZBL","aktualit":"2024-10-19Z","gebnutzbez":"Gebäude","funktion":"Allgemein bildende Schule","gfkzshh":"31001_3021","gmdschl":"05315000"}
{"oid":"DENW37AL1000GvJxBL","aktualit":"2024-10-19Z","gebnutzbez":"Gebäude","funktion":"Sport-, Turnhalle","gfkzshh":"31001_3211","gmdschl":"05315000"}
{"oid":"DENW37AL1000Hjk3BL","aktualit":"2024-10-19Z","gebnutzbez":"Gebäude","funktion":"Gebäude für Wirtschaft oder Gewerbe","gfkzshh":"31001_2000","rellage":"Unter der Erdoberfläche","gmdschl":"05315000"}
{"oid":"DENW37AL1000HrivDENW37AL1000GwzdTF","aktualit":"2025-03-04Z","nutzart":"Fläche besonderer funktionaler Prägung","bez":"Bildung und Wissenschaft","flstkennz":"05495905600508______"}
{"oid":"DENW37AL1000H0BADENW37AL1000GwzdTF","aktualit":"2025-03-04Z","nutzart":"Sport-, Freizeit- und Erholungsfläche","bez":"Spielplatz, Bolzplatz","flstkennz":"05495905600508______"}
{"oid":"DENW37AL1000HrsiDENW37AL1000GzwRTF","aktualit":"2025-03-04Z","nutzart":"Straßenverkehr","name":"Leybergstr.","flstkennz":"05495905600353______"}
```

The school parcel's `bez` is **"Bildung und Wissenschaft"** and the buildings' `funktion` **"Allgemein bildende Schule"** —
as in Brandenburg, unlike Hesse's generalised values.

### 8.5 Observed values (school 2 × 2 km + Münster + Aachen + Arnsberger Wald: 17,178 buildings, 14,169 parcels, 4,863 Nutzung)

- **`gebnutzbez`:** `Gebäude` 13,717, `Bauteil` 2,495, `Sonstiges Bauwerk oder sonstige Einrichtung` 950, `Historisches
  Bauwerk oder historische Einrichtung` 9, `Bauwerk oder Anlage für Sport, Freizeit und Erholung` 3, `Bauwerk oder Anlage
  für Industrie und Gewerbe` 2, `Turm` 2.
- **`funktion` is detailed** (73 values): Wohngebäude 5,290, Gemischt genutztes Gebäude mit Wohnen 4,016, Gebäude für
  Wirtschaft oder Gewerbe 1,966, Durchfahrt im Gebäude 1,104, Gebäude zum Parken 1,042, Überdachung 879, Sonstiges 635,
  Auskragender Geschossteil 558, Gebäude für Land- und Forstwirtschaft 286, Gebäude zur Freizeitgestaltung 172, Gebäude für
  Bildung und Forschung 145, **Allgemein bildende Schule 113**, Gebäude für soziale Zwecke 95, Arkade 87, Tiefgarage 76,
  Hochschulgebäude 68, Carport 65, Kirche 35, Sport-, Turnhalle 33, Krankenhaus 28, Berufsbildende Schule 11 …
  **11 values are multi-valued with `;`** ("Gebäude für Wirtschaft oder Gewerbe;Parkdeck", "Gemischt genutztes Gebäude mit
  Wohnen;Theater"), as the WFS abstract announces ("durch definierte Trennzeichen getrennt").
- **`rellage`:** `Unter der Erdoberfläche` 75, `Aufgeständert` 29. Of 76 `Bauteil`/`Tiefgarage`, 62 have "Unter der
  Erdoberfläche" and **14 have no `rellage`**.
- **`name`** rare (140) and clean (no split words, no mojibake seen); several names joined by `;`
  ("Universitätskliniken Köln;Frauenklinik", "Geomuseum;Landsberg´sche Kurie").
- **`nutzart`** (NutzungFlurstueck; LGB spellings): Wohnbaufläche 5,104, Fläche gemischter Nutzung 3,957, Straßenverkehr
  1,351, Weg 1,137, Fläche besonderer funktionaler Prägung 846, Industrie- und Gewerbefläche 711, Sport-, Freizeit- und
  Erholungsfläche 278, Wald 269, Platz 149, Bahnverkehr 134, Fließgewässer 107, Unland/Vegetationslose Fläche 88, Stehendes
  Gewässer 15, Landwirtschaft 14, Friedhof 5, Gehölz 4. `PRIVATE_LAND_USES` matches as in Brandenburg.
- **`bez` per `nutzart`:**
  - `Straßenverkehr`: none 942, **`Begleitfläche Straßenverkehr` 252**, `Fußgängerzone` 157. Never `Fahrbahn`. The `VERGE`
    rule (`build/paved.rs`) fires for the 19 % with the verge value; the rest becomes paved.
  - `Weg`: none 1,128, `Begleitfläche Weg` 9.
  - `Platz`: `Parkplatz` 85, **none 35**, `Fußgängerzone` 28, `Marktplatz` 1 → `square_bez` needed.
  - FBFP: Bildung und Wissenschaft 224, none 168, Religiöse Einrichtung 147, Soziales 125, Regierung und Verwaltung 81,
    Kultur 41, Gesundheit, Kur 22, Sicherheit und Ordnung 22, Öffentliche Zwecke 9, Historische Anlage 6, Medien und
    Kommunikation 1.
  - Fläche gemischter Nutzung: **`Gebäude- und Freifläche Mischnutzung mit Wohnen` 3,956** (a `bez` on almost every parcel).
  - IuG: Handel und Dienstleistung 626, Versorgungsanlage 32, Industrie und Gewerbe 22, Umspannstation 15, Lagerfläche 8,
    Tankstelle 4 …
  - SFE: Grünanlage 86, Park 85, Spielplatz, Bolzplatz 61, Kleingarten 14, Siedlungsgrünfläche 13, Garten 11, Sportanlage 5,
    Schwimmen 3.
  - Wald: Laub- und Nadelholz 206, Nadelholz 35, Laubholz 28. Fließgewässer: Bach 84, Fluss 22, Graben 1. Unland:
    Gewässerbegleitfläche 88. Stehendes Gewässer: Teich 14, See 1. Landwirtschaft: Grünland 14.
  - `Wohnbaufläche`, `Bahnverkehr`, `Friedhof`, `Gehölz`: no `bez`.

### 8.6 What is missing or different

- WFS: GML only (GeoPackage/shape too), generic error texts, no gzip, slow first request.
- OGC API: no `oid` property, `fktkurz` for `gfkzshh`, limit ≤ 10000.
- `;`-joined multi-values in `funktion` and `name`.
- No carriageway `bez`; verges marked on part of the streets only.
- Not examined: Flurstück, Gebäude 3D (LoD2, `3dg`), INSPIRE services.

---

## 9. Licence and attribution

- Licence: **Datenlizenz Deutschland – Zero – Version 2.0** (<https://www.govdata.de/dl-de/zero-2-0>) on every product used:
  WMS/WCS/WFS `Fees`, ISO `otherConstraints` (with the JSON `{"id": "dl-zero-de/2.0", "name": "Datenlizenz Deutschland – Zero
  – Version 2.0", "url": "https://www.govdata.de/dl-de/zero-2-0", "quelle": ""}` — **an empty source note**), the OGC API
  `license` link, the OpenGeodata listing page ("Geobasis NRW stellt diese Daten unter der Datenlizenz Deutschland - Zero -
  Version 2.0 bereit; jede Nutzung ist ohne Einschränkungen oder Bedingungen zulässig.") and the licence PDF
  `https://www.bezreg-koeln.nrw.de/system/files/media/document/file/lizenzbedingungen_geobasis_nrw.pdf` ("Stand: 3/2020").
- Verbatim (`Fees` of WMS, WCS and WFS; `AccessConstraints NONE`):
  > "Die Geobasisdaten des amtlichen Vermessungswesens werden als öffentliche Aufgabe gem. VermKatG NRW und gebührenfrei nach
  > Open Data-Prinzipien über online-Verfahren bereitgestellt. Nutzungsbedingungen: Es gelten die durch den IT-Planungsrat im
  > Datenportal für Deutschland (GovData) veröffentlichten einheitlichen Lizenzbedingungen „Datenlizenz Deutschland – Zero"
  > (https://www.govdata.de/dl-de/zero-2-0). Jede Nutzung ist ohne Einschränkungen oder Bedingungen zulässig. Eine Haftung für
  > die zur Verfügung gestellten Daten und Dienste wird ausgeschlossen. …"
- **No attribution and no change note are required.** Unlike Hesse (§ 18 HVGG), NRW has no rule that a voluntary source
  note must mention changes. No recommended Quellenvermerk was found (product pages, open-data page, PDF, ISO).
- Copyright wording in the data: TIFF tag "Bezirksregierung Koeln, Geobasis NRW" (DGM1/DOM1), "(c) Geobasis NRW" (nDOM50);
  CSV `Eigentuemer` "Land Nordrhein-Westfalen, Bezirksregierung Koeln, Geobasis NRW".
- **Recommended freemapper line** (voluntary, but freemapper names every source):
  `Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) <year>, dl-de/zero-2-0` — "verändert" may be appended for
  uniformity with the other states, but is not required.

---

## 10. State boundary (for `assets/regions/nordrhein-westfalen.geojson`)

- **Source:** the ALKIS WFS `ave:VerwaltungsEinheit`, `art = Bundesland`, oid **`DE05`**, `name Nordrhein-Westfalen`,
  `ags 05`, `aktualit 2026-09-30Z`. Fetched by a tiny bbox inside NRW (returns Gemeinde DE05315000, Kreis DE05315,
  Regierungsbezirk DE053, Land DE05):
  ```
  https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=-66685,5671676,-66675,5671686,urn:ogc:def:crs:EPSG::25833
  ```
  → 200, 4,301,928 B, **4.2 s** (the 25832 variant: 2,852,999 B, 7.9 s). `RESOURCEID` and `FILTER` were not tried.
- Geometry: MultiSurface with **6 polygons, no holes**, 72,228 vertices:
  - main polygon 34,287.4 km² (25833 area), 70,211 vertices;
  - **five exclaves west of the Vennbahn** (German territory surrounded by Belgium, near Monschau/Roetgen): 12.21, 10.10,
    1.90, 0.96 and 0.017 km² (representative points in 25833: (−122277, 5638239), (−119627, 5646189), (−122590, 5648756),
    (−123588, 5634193), (−118993, 5641731)).
  - valid (shapely).
- **25 m topology-preserving simplification:** `probe/boundary/nordrhein-westfalen_simplified25m.geojson`
  (script `scripts/simplify_nrw.py`: shapely 2.2 `simplify(25, preserve_topology=True)`, then `set_precision(grid_size=1)`
  for integer metres, which keeps the result valid).
  - FeatureCollection, `name` "Nordrhein-Westfalen (ALKIS, simplified 25 m)", `crs` `urn:ogc:def:crs:EPSG::25833`,
    properties `name: Nordrhein-Westfalen`, `source: Geobasis NRW ALKIS vereinfacht WFS (VerwaltungsEinheit DE05),
    dl-de/zero-2-0`, `retrieved: 2026-11-08`, `simplified_m: 25`; MultiPolygon with integer coordinates, parts sorted by area.
  - **5,606 vertices, 94.5 KB** (Hesse asset 5,166 vertices / 83 KB). Parts 5,416 / 76 / 72 / 22 / 15 / 5 vertices; every
    part kept. Rings clockwise, like the Hesse and SH assets.
  - **Valid** (shapely `is_valid`, "Valid Geometry"). The repo test `region_boundaries_are_valid` uses geo's
    `check_validation`; not run.
  - Area 34,312.32 km² (full 34,312.55), symmetric difference 7.53 km², Hausdorff 29.7 m.
  - Extent E −139634 … 118732, N 5608984 … 5839628: **negative eastings**, like the Hesse asset.
- **Against `assets/regions/hessen.geojson`** (`scripts/border_check.py`, `scripts/gap_border.py`):
  - The shared border is 273.5 km, from the NRW–Hesse–RLP tripoint (50.6857°N 8.1250°E) to the NRW–Hesse–Lower Saxony
    tripoint (51.6506°N 9.4401°E).
  - The full-resolution NRW polygon and the full Hesse polygon (HVBG ALKIS DE06) overlap by only **381 m²**: the two states'
    official borders agree.
  - Simplified NRW asset vs the Hesse asset: **overlaps 881 slivers, 27,445 m² in total**, largest 6,742 m², max width
    22.8 m; **gaps 10 slivers, 7,383 m²**, largest 2,128 m², max width 21.0 m. (The other "holes" in NRW ∪ Hesse are Hesse's
    own holes far from NRW.)
  - Sampled every 50 m along the true border: NRW asset max deviation 24.7 m (median 2.0 m); Hesse asset max 100 m at the
    RLP tripoint (Hesse's own simplification), median 2.0 m.
  - So a point within ≈25 m of the NRW–Hesse border can be in both or neither asset; freemapper's `regions_for` buffers by
    `MARGIN_M` anyway. A shared-edge simplification (e.g. `shapely.coverage_simplify` of both full polygons, then
    re-generating `hessen.geojson`) would remove the slivers; not done.
- Neighbours: Lower Saxony, Hesse, Rhineland-Palatinate, Belgium, Netherlands. Only Hesse is a freemapper region.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/nordrhein-westfalen-latest.osm.pbf` → **307** →
  `nordrhein-westfalen-261107.osm.pbf` → 200, **917,065,941 B** (874.6 MiB), `Last-Modified: Sun, 08 Nov 2026 02:02:03 GMT`.
  Hesse for comparison: 347,661,735 B. freemapper's `MAX_EXTRACT_BYTES` (2 GiB) allows it; the doc comment there ("the
  largest state extract is about 350 MB") becomes wrong.
- State: `https://download.geofabrik.de/europe/germany/nordrhein-westfalen-updates/state.txt`
  ```
  # original OSM minutely replication sequence number 7362973
  timestamp=2026-11-07T21\:21\:36Z
  sequenceNumber=4957
  ```
- Poly: `https://download.geofabrik.de/europe/germany/nordrhein-westfalen.poly` (21,715 B, one ring, 700 points).
- **Regierungsbezirk extracts** (each with its own `…-updates/state.txt`, e.g. `nordrhein-westfalen/koeln-regbez-updates/state.txt`
  sequence 4946):

  | Extract | Bytes |
  |---|---|
  | `nordrhein-westfalen/arnsberg-regbez-latest.osm.pbf` | 195,184,035 |
  | `nordrhein-westfalen/detmold-regbez-latest.osm.pbf` | 137,290,401 |
  | `nordrhein-westfalen/duesseldorf-regbez-latest.osm.pbf` | 217,243,228 |
  | `nordrhein-westfalen/koeln-regbez-latest.osm.pbf` | 225,223,763 |
  | `nordrhein-westfalen/muenster-regbez-latest.osm.pbf` | 140,927,794 |

  Using them would need a Regierungsbezirk boundary per extract (the ALKIS `VerwaltungsEinheit` DE051 … DE059 or Geofabrik's
  `.poly`) and two extracts for maps on a Regierungsbezirk border.
- Download speed on this test line: 1.17 MB/s (20 MB range), i.e. ≈13 minutes for the state extract.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `www.opengeodata.nrw.de` files | HEAD 200 + Content-Length, ranges 206, ETag/Last-Modified, `Content-Disposition: attachment`, wrong name → 404 (proper status). Names need the per-tile year (§2.2). Listings: content negotiation, JSON default, `Accept: application/xml` → 406; DOP/bDOM listings contain stale duplicates. No gzip. |
| `www.wms.nrw.de` DOP WMS | **max 5000 px**, oversize → **HTTP 200 `text/xml`** exception; ≈6 s per 5000² JPEG (once 22 s); q75 JPEG; white outside the tiles; no Content-Length, no caching headers |
| `www.wcs.nrw.de` WCS | **≈2 km per axis** (DGM ≤ 2,020 m, nDOM ≤ 2,005 m, DOP 2,000 m refused) → **HTTP 400 "OGC Proxy: InvalidParameterValue"** without reason; fractional bbox resampled; DGM output LZW, DOM/nDOM/DOP uncompressed (DOM/nDOM without NoData tag) |
| `www.wfs.nrw.de` WFS | GML (+ shape, GeoPackage); `numberMatched="unknown"` on results (use `RESULTTYPE=hits`); paging works with SORTBY; **all errors 400 with the same generic text**; no gzip; first request after idle 5–9 s; no feature cap up to 41,697 |
| `ogc-api.nrw.de` | GeoJSON, gzip, HTTP/2, `limit` ≤ 10000 with `next` links; no `oid` property |
| Geofabrik | 307 to the dated file; 917 MB |

- **TLS (checked with `openssl s_client -showcerts -CAfile <certifi bundle>`, which does no AIA fetching):** every host —
  `www.opengeodata.nrw.de`, `www.wms.nrw.de`, `www.wfs.nrw.de`, `www.wcs.nrw.de`, `apps.geoportal.nrw.de`,
  `www.geoportal.nrw.de` (and `download.geofabrik.de`) — sends **4 certificates** (leaf → Let's Encrypt YE1/YE2 → ISRG Root YE
  → ISRG Root X2 cross-signed by ISRG Root X1), all ECDSA P-384; **`Verify return code: 0 (ok)`**. ISRG Root X1 and X2 are in
  the Mozilla/webpki roots, so ureq + rustls + webpki-roots will connect. opengeodata negotiated TLS 1.2
  ECDHE-ECDSA-AES256-GCM-SHA384. No SH-style missing intermediate. (`ogc-api.nrw.de` was used over HTTPS without problems; its
  chain was not dumped.)
- Throughput: ≈0.94–1.13 MB/s per file from opengeodata, ≈1.2 MB/s total with 4 parallel connections — the same as Geofabrik
  on this line, so the line, not the server, was the limit.
- No authentication anywhere, no rate-limit or `Retry-After` headers, no throttling seen in ≈220 requests (≈0.5 GB). CORS `*`
  on the file server and the OGC API.

---

## 13. What makes the NRW adapter differ (risks first)

1. **Tile file names cannot be computed.** Every name ends in the flight/ALS year of that tile (`_nw_2022.tif`), nDOM50 and
   bDOM50 drop an underscore (`ndom50_32354_5642…`), 3DM has no year. The adapter needs an index: the per-product
   `*_meta.zip` (110–300 KB, one CSV row per tile with the exact name and date) is the cheapest; cache it by ETag. The DOP and
   bDOM listings also hold 756/1,196 stale `_2023` duplicates — never pick a name from the listing by pattern.
2. **The DOP tiles are JPEG 2000 only** (10 cm RGBI, 30–80 MB, one image tile, 0.85 GB to decode at 0.2 m). freemapper has no
   JP2 decoder. **Use the WMS** (`nw_dop_rgb`, ≤ 5000 px, ≈6 s per km² at 0.2 m, the tile's pixels, GetFeatureInfo date). A
   per-area WMS path like Hesse/SH/Hamburg (`per_area() → true` for the orthophoto).
3. **Which nDSM?** DOM1 − DGM1 (ALS, 1 m, same date, leaf-off, −1 … 5 years older than the DOP) is the drop-in for the
   existing DOM path. nDOM50 (0.5 m, normalised, 16 MB uncompressed per tile, not clamped, image-based, often the DOP date) is
   closer to the orthophoto but 7 × the bytes. Decision needed; DOM1 − DGM1 is the conservative default.
4. **ALKIS has two working services.** The WFS fits `alkis_wfs` unchanged (`WfsService { url:
   "https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht", land_use_type: "ave:NutzungFlurstueck", crs: Utm32, square_bez:
   "ohne Unterart (Nordrhein-Westfalen)" }`). The OGC API would fit the LGB reader only after a fallback from `properties.oid`
   to the feature `id` (and `fktkurz` ↔ `gfkzshh` if that is ever used). WFS errors are generic 400s; the first request can
   take 9 s.
5. **The OSM extract is 917 MB** (2.6 × Hesse); download and full read take minutes. Regierungsbezirk extracts (137–225 MB)
   would need region boundaries per extract and multi-extract maps.
6. **CRS:** everything 25832; in 25833 NRW reaches −139.6 km (59.5 % of the area negative), −6.3° convergence and +3.5 to
   +4.7 m/km scale. `Region::crs() → Utm32`; no 25833 `TileId` may ever be built for NRW.
7. **WCS limits:** ≈2 km per axis, generic 400, fractional bbox resampled. Only relevant if the WCS is used instead of tiles.
8. **Boundary:** 6 parts (5 Vennbahn exclaves in Belgium); slivers ≤ 23 m wide against the Hesse asset (§10).
9. **Currency:** the DOP is a 2022/2024/2025/2026 patchwork with leaf-off (Feb–Apr) and leaf-on (May–Oct) regions; the
   `Belaubungszustand` code is unreliable in 2024. The DGM1 stays at ≤ Feb 2025 until 2027 (ALS paused).
10. ALKIS values: detailed `funktion` with `;` multi-values; 14 underground garages without `rellage`; verges on 19 % of
    street parcels; 23 % of squares without `bez`.
11. Unchanged from the other states: DHHN2016, DGM1 pixel registration (tiepoint at the NW corner, PixelIsArea), Float32,
    NoData −9999, 1 km tiles, LGB spellings, Geofabrik layout. TIFF details are simpler than Hesse (LZW without predictor, IFD
    first), but there is **no TFW** (freemapper reads the GeoTIFF tags). Licence simpler than everywhere else.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in NRW | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `https://www.wms.nrw.de/geobasis/wms_nw_dop`, layer `nw_dop_rgb`, `CRS=EPSG:25832`, 0.2 m, JPEG, pieces ≤ 5000 px (`max_px: 4000` or 5000), timeout + retry | Currency: GetFeatureInfo `nw_dop_utm_info` (`Bildflugdatum`) or `dop_nw.csv` `Aktualitaet`. Not the JP2 tiles (no decoder). |
| **DTM** (hillshade, slope, contours) | **DGM1 tiles** `https://www.opengeodata.nrw.de/produkte/geobasis/hm/dgm1_tiff/dgm1_tiff/<Kachelname>.tif`, names from `dgm1_meta.zip` | LZW Float32, NoData −9999, no TFW; ≈2.2 MB per km². Fallback: WCS `nw_dgm` (identical values, ≤ 2 km). |
| **nDSM template** | **DOM1 − DGM1** from `…/hm/dom1_tiff/dom1_tiff/<Kachelname>.tif` (same names with `dom1_`) | Clamp < 0; flag the ALS date against the DOP date. Option: nDOM50 (`…/hm/ndom50_tiff/ndom50_tiff/`, 0.5 m, 16 MB) when the DOP and bDOM dates match. |
| **Buildings** | **WFS** `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht`, `ave:GebaeudeBauwerk`, 25832, `SORTBY=oid&COUNT=10000`, hits first | Reuse `alkis_wfs`; LGB spellings; `rellage`; oid prefix `DENW`. Alternative: OGC API `ogc-api.nrw.de/lika/v1` after an `id` fallback. |
| **Land use** (out-of-bounds, paved) | the same WFS, **`ave:NutzungFlurstueck`** | `square_bez` "ohne Unterart (Nordrhein-Westfalen)"; verge rule fires for "Begleitfläche Straßenverkehr". |
| **Region boundary** | WFS `ave:VerwaltungsEinheit` oid DE05 in 25833 (small-bbox query), simplified 25 m topology-preserving | Ready: `probe/boundary/nordrhein-westfalen_simplified25m.geojson` (5,606 vertices, 6 parts, valid). Region CRS Utm32. |
| **OSM** | Geofabrik `europe/germany/nordrhein-westfalen-latest.osm.pbf` + `nordrhein-westfalen-updates/state.txt` | 917 MB; follow the 307. |
| **Attribution** | `Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) <year>, dl-de/zero-2-0` + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9; voluntary, no change note required. |

---

## 15. Not verified, or open

- The meaning of the `Belaubungszustand` codes (1/2/3) and why 939 summer-2024 tiles are coded 1.
- Whether the stale `_2023` DOP/bDOM files are removed later, and how fast the listings/CSVs follow a new flight.
- That freemapper's `tiff` decoder reads the NRW DGM1 (LZW, 2-row strips, no predictor; simple, not run) and that geo's
  `check_validation` accepts the boundary asset (shapely says valid; the repo test was not run).
- WCS output in 25833; the exact rule behind the WCS size limit (per coverage, ≈2 km); the DOP WCS in general.
- WMS behaviour under sustained load (≈20 GetMaps made, no failure); the WMS `nw_dop_cir`/`nw_dop_nir` layers.
- The OGC API in depth (CQL2 filters, FlatGeobuf, `nutzung_flurstueck` paging beyond 10,000); its TLS chain dump.
- `nutzart`/`bez` values of rarer classes (Heide, Moor, Tagebau, Flugverkehr, Hafen …) and of rural areas beyond one forest.
- The "Open Data Downloadclient" (custom areas), historic and InVeKoS DOP, 3D-Mesh, LoD2.
- A shared-edge re-simplification of `hessen.geojson` and `nordrhein-westfalen.geojson` to remove the border slivers.
- Whether the Regierungsbezirk extracts are worth the extra region logic.

Network note: total transfer was roughly **0.5 GB**: DOP JP2s (66 MB), nDOM50 tiles (48 MB), WMS probes incl. one 65 MB PNG
(deleted), listings (38 MB), ALKIS samples incl. a 41 MB and a 24 MB GML (deleted), LAZ ranges (40 MB), DGM/DOM tiles, metadata
zips, Geofabrik heads. No Geofabrik extract was downloaded. Disk usage under `nrw/` is about 346 MB.
