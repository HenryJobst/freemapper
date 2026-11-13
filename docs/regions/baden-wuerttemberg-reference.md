# Baden-Württemberg (LGL-BW) data sources for freemapper — live-verified reference

Verified live on 2026-11-08 (Europe/Berlin), evening (20:42–22:30 UTC). Provider: **Landesamt für Geoinformation und
Landentwicklung Baden-Württemberg (LGL)**, Stuttgart. Open data under **Datenlizenz Deutschland – Namensnennung 2.0
(dl-de/by-2-0)**, attribution required. The data come from four hosts:

- the file server **`https://opengeodata.lgl-bw.de/data/<product>/…`**: one zip per 2 km block (DGM1, DOM1, nDOM1,
  DOP20, …), one zip per Gemarkung (ALKIS NAS/Shape), statewide zips (Landnutzung, Hausumringe, Basis-DLM, …). No directory
  listing (`/data/` and `/data/alkis/` → **403**). The portal in front of it (`https://opengeodata.lgl-bw.de/`) is an
  Angular app. Its config `assets/config/local/odp-products.json` and its vector tiles
  `tiles/vts/{2x2Gitter,5x5Gitter,Gemarkungen}/{z}/{x}/{y}.pbf` name every file;
- the OWS proxy **`https://owsproxy.lgl-bw.de/owsproxy/{ows,wfs,wcs}/<service>`** (GeoServer behind an F5 WAF): the DOP
  WMS, the **open ALKIS WFS `WFS_LGL-BW_ALKIS`**, the currency WFS for DOP and DGM, the INSPIRE services;
- the OGC API Features **beta** `https://ogcapi.lgl-bw.de/beta/{ldproxy,pygeoapi}/` (same ALKIS views, labelled CC BY 4.0);
- the web site `www.lgl-bw.de` (licence text) and the metadata catalogue `metadaten.geoportal-bw.de` (GeoNetwork).

No login, token or form is needed for anything below. The F5 WAF sets `TS…` cookies, which can be ignored. All TLS
chains are complete (§12).

**The survey row in `docs/regions/states-survey.md` is out of date in four points** (all verified below):
1. there **is** an open ALKIS WFS with buildings **and actual land use** (`WFS_LGL-BW_ALKIS`, types
   `nora:v_al_gebaeude`, `nora:v_al_tatsaechliche_nutzung`, `nora:v_al_land`, dl-de/by-2-0), so no GPKG is needed;
2. the 2 km zips are **deflated (method 8)**, not stored. The central directory lists the members, and each member
   can be read by HTTP ranges and inflated on its own (`zip_range` works unchanged);
3. **DGM1 is XYZ text** (1 m cell centres, CRLF), not GeoTIFF; DOM1/nDOM1 are Float32 GeoTIFF;
4. the DOP flight date per 1 km tile is in an open WFS (`WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet`), and the
   WMS shows a **newer** vintage than the downloadable zips in about half the state.

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile / 2 km zip key |
|---|---|---|---|---|---|---|
| **Eberhard-Ludwigs-Gymnasium**, Herdweg 72, Stuttgart-Nord (primary) | way **150792563** | E 511852 / N 5403836 | E 71230 / N 5420285 | 48.78741 / 9.16135 | E 511790–511906, N 5403770–5403918 (10,035 m²) | **`511 5403`** / zip **`511_5402`** |
| **Rotteck-Gymnasium**, Lessingstraße 16, Freiburg im Breisgau (second school) | way **24740342** | E 413511 / N 5315890 | **E −33999** / N 5340090 | 47.99046 / 7.84076 | E 413359–413609, N 5315815–5315956 (15,724 m²) | `413 5315` / zip `413_5314` |
| **RP border probe**, Mannheim harbour / Rhine / Ludwigshafen | – | tile E 459000–460000, N 5483000–5484000 | | ≈49.49 / 8.45 | – | `459 5483` (59.2 % BW) / zip `459_5482` |
| **Hesse border probe**, Mannheim-Käfertal / Viernheim | – | tile E 469000–470000, N 5485000–5486000 | | ≈49.53 / 8.57 | – | `469 5485` (46.0 % BW) / zip `469_5484` |
| **Bavaria border probe**, Ulm / Danube / Neu-Ulm (Bavaria unsupported) | – | tile E 572000–573000, N 5359000–5360000 | | ≈48.39 / 9.99 | – | `572 5359` (52.7 % BW) / zip `571_5358` |
| ALKIS value samples: 2 × 2 km at Stuttgart school, Freiburg school, Mannheim centre (E 460500 N 5481000), Ulm centre (E 573000 N 5359000), Karlsruhe centre (E 455000 N 5428000), Titisee/Black Forest (E 431000 N 5303000), Heilbronn/Neckar (E 513500 N 5443000) (SW corners) | | | | | | |

(Coordinates from pyproj 3.8.0; school list from Overpass, `probe/osm/schools.json`, `school_ways.json`.)

**Note on 25833: Freiburg's easting in zone 33 is −34.0 km; 16.8 % of BW's area has negative 25833 eastings** (§3).

Samples (they were kept in a scratch folder `bw/` during the research and are not in the repository; the table
says what was looked at):

| Path | Content |
|---|---|
| `baden-wuerttemberg.geojson` | **the boundary asset** (§10) |
| `dl/ziptail/` | HEAD + last 64/256 KiB of the 2 km zips (DGM1/DOM1/nDOM1/DOP `513_5402`), the Stuttgart Gemarkung zips, `ln_bw.zip`, `ln_bw_shp.zip`, `hu_bw.zip`, `lb_bw.zip`, `Verw-Grenzen_ALKIS.zip`, `bdlm_bw_shp.zip` |
| `dl/meta/`, `dl/school_*`, `dl/fr_meta/` | per-tile CSV/TFW/Meta members read by range: DGM1 XYZ, DOM1 TIF, nDOM1 TIF of the school tile `511 5403`; Freiburg CSVs |
| `dl/border/` | DGM1/DOM1/nDOM1 + CSV of the three border tiles; `partial/` the half XYZ `573 5359` |
| `dl/dop_partial/` | first 6 MB of two DOP TIFF members (inflated head = top 400 rows), WMS strips, `compare_zip_top_wms_bottom.png` (Stuttgart, different vintages), `compare_fr.png` (Freiburg, same) |
| `dl/wms/` | WMS answers: school 25832, Freiburg 25833, size probes (`sz_*`, `bs_*`, `bt_*`), Mannheim border (`border_mannheim*.jpg`) |
| `dl/alkis_stuttgart/` | `ALKIS-oE_081460_Stuttgart_shp.zip` members `nutzung.*`, `gebaeudeBauwerke.*`, `nutzungFlurstueck.dbf`, `verwaltungseinheit.dbf` |
| `dl/ln/` | `ln_bw_informationsdatei.txt`, `221220_ln_kulturundunterhaltung.dbf` (the LN schema) |
| `dl/portal/`, `dl/portalcfg/`, `dl/vts/` | portal JS bundles (grepped only), `odp-products.json`, layer config, decoded vector tiles |
| `probe/caps/` | capabilities: DOP WMS, `WFS_LGL-BW_ALKIS`, DOP/DGM Aktualität WFS/WMS, INSPIRE buildings/land use/land cover/admin units, DGM1 WCS |
| `probe/wfs/` | currency WFS answers (school, **whole state**: `dop_bfk_all.xml` 36,653 tiles, `dgm_kacheln_all.xml`), ALKIS paging/error/25833 probes, INSPIRE samples |
| `probe/alkis/areas/` + `areas_stats.txt` | ALKIS WFS GeoJSON for seven 2 × 2 km areas (5,927 land-use objects, 24,451 buildings) and the value statistics |
| `probe/border/` | ALKIS buildings/land use in the three border tiles and in Viernheim / Neu-Ulm |
| `probe/ogcapi/` | OGC API landing pages, collections, queryables, items (buildings, parcels, Gemarkungen incl. all 3,380 names) |
| `probe/boundary/` | `land_25833.json`, `land_25832.json` (`nora:v_al_land`, full resolution) |
| `probe/geofabrik/`, `probe/osm/` | Geofabrik page, polys (state + 4 Regierungsbezirke); Overpass answers |
| `probe/csw/` | metadata-catalogue searches (found the currency services) |
| `probe/licence/` | `www.lgl-bw.de/Produkte/Open-Data/` page, Landinfo 2025 article PDF |
| `probe/tls/` | `openssl s_client -showcerts` per host |
| `probe/odp_products.txt`, `probe/gmk_names_stuttgart.txt`, `probe/stuttgart_nutzung_stats.txt` | portal product list; Gemarkung file names; Shape value stats |

---

## 1. Brandenburg → Baden-Württemberg at a glance

| freemapper product | Brandenburg (today) | Baden-Württemberg equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product.** BW spans 7.51–10.50°E. Zone 32 is the right zone everywhere (convergence −1.1° … +1.1°); in 25833 the west reaches E −61.7 km (**16.8 % of the area negative**) | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **WMS** `WMS_LGL-BW_ATKIS_DOP_20_C`, layer `IMAGES_DOP_20_RGB` (clipped at the state border). Announces MaxWidth 15000, but **≥ 6144 px answers a blank white JPEG with HTTP 200**; ≤ 6000 px works. **No GetFeatureInfo.** Flight date per 1 km tile from the open **WFS `…DOP_20_Bildflugkacheln_Aktualitaet`** (`befliegungsdatum`), all May–September (leaf-on). The WMS shows the WFS vintage (2026 in 48 % of tiles); the 2 km zip tiles (uncompressed TIFF, 224–256 MB) lag behind | verified |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1 as XYZ text** inside 2 km zips (`dgm1_32_EEE_NNNN_2_bw.zip`, 13.5–13.7 MB, deflated), four 1 km members `dgm1_32_EEE_NNNN_1_bw_YYYY.xyz` (3.2–3.5 MB compressed → 28–29 MB) + one CSV per tile. Read per member by HTTP range (`zip_range`). Same XYZ layout as SH (`parse_xyz`), **but border tiles can hold only part of the rows** | verified |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **nDOM1 ready-made** (`ndom1_32_EEE_NNNN_2_bw.zip`, 11 MB; members `ndom1_32_EEE_NNNN_1_bw.tif`, no year), Float32 GeoTIFF, = bDOM20 (image matching, **leaf-on, flight date in the CSV**) − DGM1, clamped at 0. Alternative DOM1 − DGM1 (ALS, leaf-off, 2016–2024) | verified |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **WFS `WFS_LGL-BW_ALKIS`, type `nora:v_al_gebaeude`** (GeoServer, GML 3.2/GeoJSON, 25832/25833). Not the `ave:` schema: `gml_id`, `gebaeudefunktion_name`, `lage_oberflaeche_name` ("Unter der Erdoberfläche" on all 625 Tiefgaragen), `eigenname`, `beginn`. AX_Gebaeude only (no Bauteil/Überdachung) | verified |
| Land use | OGC API `nutzung_flurstueck` | **same WFS, `nora:v_al_tatsaechliche_nutzung`**: unsplit TN objects, `objektname` (= LGB `nutzart` spelling, incl. "Unland/Vegetationslose Fläche"), `unterart_name` (= `bez` when `unterart_id` is set). ≈2 MB and ≈1 s per 2 × 2 km | verified |
| Currency | `bb_*_aktualitaet.csv` | DOP: Bildflugkacheln WFS per 1 km tile; DGM/DOM: CSV member per tile (`Aktualitaet`, `Fortfuehrung`) and the DGM Aktualität WFS; nDOM: CSV (`bDOM20_Aktualitaet`); ALKIS `beginn` per object | verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `baden-wuerttemberg-latest.osm.pbf` = **652.8 MB**; four **Regierungsbezirk** extracts 128–212 MB (Freiburg, Karlsruhe, Stuttgart, Tübingen), as for NRW | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **dl-de/by-2-0**. Required text (open-data page): **„Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0“**; capabilities show a year form „LGL-BW (2024) Datenlizenz Deutschland - Namensnennung - Version 2.0, www.lgl-bw.de“ | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **File server** | `https://opengeodata.lgl-bw.de/data/` | `dgm/`, `dom1/`, `ndom1/`, `dop20/`, `dgm025/`, `dom/` (DOM5 XYZ), `lod1/`, `lod2/`, `dtk*/`, `alkis/` (per Gemarkung), `ln/`, `lb/`, `hu/`, `hk/`, `vg/`, `bdlm/` | HEAD 0.08 s, `Content-Length`, `ETag`, `Last-Modified`, **`Accept-Ranges: bytes`**, Range → **206**; missing file → **404**; directories → **403**. Small range reads 0.16–0.3 s; a 3.4 MB member in 1.2 s, 9.5 MB in 3.1 s (≈3 MB/s). Sets `MyCookie` and `TS…` cookies (ignorable). CSP lists `owsproxy.lgl-bw.de` |
| Portal config | `https://opengeodata.lgl-bw.de/assets/config/local/odp-products.json` (82,678 B) | the 25 products, URL patterns, sizes, formats, update cycles (`probe/odp_products.txt`) | e.g. DGM1 "Datenformat xyz, 11 MB, 2 km × 2 km, 4 mal pro Jahr"; Landnutzung "Geopackage, Shape, 4,8 GB, landesweit, 2 mal pro Jahr"; ALKIS "NAS und Shape, 1 bis 80 MB pro Gemarkung, 4 mal pro Jahr" |
| Portal vector tiles | `https://opengeodata.lgl-bw.de/tiles/vts/2x2Gitter/{z}/{x}/{y}.pbf` (also `5x5Gitter`, `Gemarkungen`) | gzip MVT; each feature has a JSON `metadata` with the exact file names | `2x2Gitter/12/2152/1410.pbf` → 200, 2,000 B, 19 cells, e.g. `{"name":"513-5406","products":[{"name":"DOP","types":[{"type":"DOP20_RGB","fileName":"dop20rgb_32_513_5406_2_bw.zip","downloadURL":"/data/dop20/dop20rgb_32_513_5406_2_bw.zip"} …` |
| **OWS proxy** | `https://owsproxy.lgl-bw.de/owsproxy/ows/<WMS>`, `/wfs/<WFS>`, `/wcs/<WCS>` | DOP WMS; ALKIS WFS; currency WFS; INSPIRE | GeoServer (`via: 1.1 geoserver:10360`) behind an **F5 WAF**; WAF rejections are **200 `text/html` "Request Rejected … Your support ID is …"** (§12) |
| OGC API (beta) | `https://ogcapi.lgl-bw.de/beta/ldproxy/<dataset>`, `…/beta/pygeoapi/` | ALKIS-BW Gebäude, Flurstücke, Gemarkungen, Fluren, points; Basis-DLM Bundesland/Kreis/Gemeinde | landing pages link `license` **CC BY 4.0** (`https://creativecommons.org/licenses/by/4.0/`); "Beta-Version … in zwei Versionen" (portal text). No land use |
| Metadata | `https://metadaten.geoportal-bw.de/geonetwork/srv/api/search/records/_search` | ES search found the currency services ("Bildflugkacheln Aktualität", "DGM Aktualität") | each zip also has a `Meta-*.txt` member with a record URL |

Not used / not examined in depth: GeoShop, LoD1/LoD2 CityGML, DTK rasters, DGM025, DOM5, Basis-DLM, Hauskoordinaten, the
INSPIRE download services, the WMTS `WMTS_LGL-BW_ATKIS_DOP_20_C`.

### 2.1 Tile scheme

- **EPSG:25832, 1 km tiles named by their SW corner in km, bundled in 2 km zips whose key is the SW tile with
  odd easting and even northing.** Verified: zips `511_5402`, `413_5314`, `513_5402`, `459_5482`, `469_5484`, `571_5358`
  exist (HEAD 200); `512_5402` and `511_5403` → 404. Zip key for tile (E, N): `E − (E+1) mod 2`, `N − N mod 2`.
- **The zip names carry no year; the member names do** (the ALS/flight year), so names come from the zip's central
  directory, never from guessing:

  | Product | Zip (2 km) | Members per 1 km tile | Zip size | Member size (compressed → raw) |
  |---|---|---|---|---|
  | DGM1 | `data/dgm/dgm1_32_EEE_NNNN_2_bw.zip` | `dgm1_32_eee_nnnn_1_bw_YYYY.xyz` + `.csv` | 13.5–13.7 MB | 2.8–3.5 MB → 28.0–29.0 MB |
  | DOM1 | `data/dom1/dom1_32_EEE_NNNN_2_bw.zip` | `dom1_32_eee_nnnn_1_bw_YYYY.tif` + `.tfw` + `.csv` | 9.1–13.8 MB | 2.1–3.1 MB → 2.2–3.3 MB |
  | nDOM1 | `data/ndom1/ndom1_32_EEE_NNNN_2_bw.zip` | `ndom1_32_eee_nnnn_1_bw.tif` + `.csv` (**no year**) | 11.0–11.3 MB | 1.8–3.7 MB → 3.8–4.2 MB |
  | DOP20 RGB | `data/dop20/dop20rgb_32_EEE_NNNN_2_bw.zip` (RGBI: `dop20rgbi_…`) | `dop20rgb_32_eee_nnnn_1_bw_YYYY.tif` + `.tfw` + `.csv` | 224–256 MB | 62–64 MB → 75.0 MB (**uncompressed TIFF**) |

  Every zip also holds a folder entry, `GOVDATA-Datenlizenz_Deutschland.pdf` (378 KB) and `Meta-ATKIS_<product>.txt` (one
  metadata URL). Example central directory (`dgm1_32_513_5402_2_bw.zip`, 13,727,337 B, directory 1,443 B at the end):
  ```
  dgm1_32_513_5402_2_bw/dgm1_32_513_5402_1_bw_2023.csv  method=8 comp=198      uncomp=278      lho=52
  dgm1_32_513_5402_2_bw/dgm1_32_513_5402_1_bw_2023.xyz  method=8 comp=3274713  uncomp=29000000 lho=332
  … (513_5403, 514_5402, 514_5403) …
  dgm1_32_513_5402_2_bw/GOVDATA-Datenlizenz_Deutschland.pdf, Meta-ATKIS_DGM1_ALS2.txt
  ```
  No zip64, no encryption, general-purpose flag 0 (ALKIS zips: `0x808` = data descriptor + UTF-8 names; the central
  directory still has the sizes). Every member read in this research passed its CRC (`scripts/zipfetch.py`).
- **Border zips hold only the 1 km tiles that touch BW**, and a zip whose four tiles all lie outside is 404:
  `dgm1_32_457_5484_2_bw.zip` has one XYZ (`458_5485`), `…471_5488…` three, `…573_5358…` one; `…455_5480`, `457_5480`,
  `579_5358`, `577_5360` → 404. **`ndom1_32_573_5358_2_bw.zip` exists but holds no tile at all** (only PDF + Meta).
- **Border XYZ can be partial:** `dgm1_32_573_5359_1_bw_2021.xyz` = 14,500,000 B, **500,000 points = the northern 500 rows
  only** (N 5359500.5–5359999.5, all 1000 columns); `dgm1_32_471_5489_1_bw_2022.xyz` 16.6 MB likewise. Full tiles across the
  border have no NoData at all (§5).

### 2.2 Finding tile names and dates

1. **Compute the zip key, read the zip's central directory by range** (last 64 KiB → directory, 1.4–2.1 KB), pick the
   member whose name starts with `dgm1_32_{E}_{N}_1_bw_` (resp. `dom1_…`, `ndom1_32_{E}_{N}_1_bw.tif`). Two requests per
   zip, ≈0.2 s each. A tile not in the directory is outside BW (or missing); a 404 zip means no BW tile there.
2. The portal's 2 km grid vector tiles give the zip URLs (MVT + JSON) — not needed, the key is computable.
3. Per-tile **CSV member** (278–988 B) gives the dates (§7).
4. Statewide currency WFS (§7) — one small bbox query, no file names.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DGM1 XYZ: cell centres `E.50 N.50 Z` (two decimals), row by row from the NW corner, E fastest, **CRLF**; CSV
    `Koordinatenreferenzsystem_Lage ETRS89_UTM32`.
  - DOM1/nDOM1 GeoTIFF: ModelTiepoint at the **NW corner** (`511000, 5404000`; nDOM `5404000.0001` — float noise,
    freemapper rounds tiepoints to mm), ModelPixelScale 1. DOM1 GeoKeys: `ProjectedCSType 25832`, `GTModelType 32767`,
    `VerticalCSType 7837`; nDOM1 (bDOM-made) has `GeographicType 4258` and an ESRI PE string but no projected EPSG code.
    freemapper reads only tiepoint/scale.
  - DOP TFW `0.2 / 0 / 0 / -0.2 / 511000.1 / 5402999.9` (pixel centre).
  - ALKIS WFS: `SRSNAME=urn:ogc:def:crs:EPSG::25832` and `::25833` both work (25833 at Freiburg returned eastings
    −33,890 … −34,060; E/N order). GeoJSON output carries `crs` `urn:ogc:def:crs:EPSG::25832` and E/N order.
  - WMS layer CRS list: 31467, 31466, 31468, **25832**, **25833**, 5649, 5650, 4647, 4326, 4258, 3857.
- **Height datum DHHN2016** normal heights, quasigeoid GCG2016 (CSV `DE_DHHN2016_NH;DE_AdV_GCG2016_QGH`; one older
  CSV spells it `DE_AdV_GC-20-6_QGH`). Same as the other freemapper states.
- **Zone choice** (pyproj factors):

  | Place | Convergence zone 32 / 33 | Scale k zone 32 / 33 | 25833 easting |
  |---|---|---|---|
  | Stuttgart (school) | +0.12° / −4.40° | 0.99960 / 1.00186 | +71.2 km |
  | Freiburg (school) | −0.86° / −5.33° | 0.99969 / 1.00311 | **−34.0 km** |
  | Mannheim | −0.41° / −4.98° | 0.99962 / 1.00235 | +26.9 km |
  | Ulm | +0.74° / −3.75° | 0.99967 / 1.00129 | +129.4 km |
  | West edge (Rhine, 7.51°E) | −1.10° / −5.55° | 0.99975 / 1.00350 | −62.9 km |
  | East edge (10.50°E) | +1.13° / −3.39° | 0.99975 / 1.00094 | +170.2 km |

  **BW maps must be 25832 projects** (`Region::crs() → Crs::Utm32`). 16.8 % of the area has negative 25833 eastings
  (the whole Upper Rhine strip west of ≈8.0°E), so no 25833 `TileId` (`u32`) may be built for BW.
- **Extent** (`nora:v_al_land`, full resolution, §10):

  | CRS | Extent |
  |---|---|
  | EPSG:25832 | E 388339 – 610073, N 5265158 – 5515628 |
  | EPSG:25833 | E −61696 – 168532, N 5283662 – 5530172 |
  | lon / lat | lon 7.512 – 10.496, lat 47.532 – 49.791 |

  Area 35,721.6 km² in 25832 (35,891.8 km² in 25833). The DGM currency WFS covers tiles E 388–610, N 5265–5515 (36,652
  tiles).

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- WMS abstract (capabilities): "Landesweite Darstellung der Orthophotos in Farbe mit einer Bodenauflösung von 20 cm …
  Jährliche Aktualisierung von ca. 50 % der Landesfläche." (The layer abstract says "ca. 30 %".) True orthophotos
  (`hoehenmodell bDOM`).
- **Flight date per 1 km tile** — open WFS (found through the metadata catalogue, record `6158a399-01b5-cf84-e738-e7aae338c99c`):
  ```
  https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=verm:v_dop_20_bildflugkacheln&BBOX=511500,5403500,512200,5404100,urn:ogc:def:crs:EPSG::25832&COUNT=20
  → 200, 6,625 B, 0.17 s, numberMatched=4
  <verm:dop_kachel>325115403</verm:dop_kachel> <verm:befliegungsdatum>2026-05-24</verm:befliegungsdatum>
  <verm:bearbeitungsjahr>2026</verm:bearbeitungsjahr> <verm:kamera>UltraCam Eagle </verm:kamera> <verm:linse>100mm </verm:linse>
  <verm:flughoehe>5180m </verm:flughoehe> <verm:filmmaterial>digital </verm:filmmaterial> <verm:hoehenmodell>bDOM   </verm:hoehenmodell>
  <verm:farbtiefe>RGBI        </verm:farbtiefe> <verm:letztes_update>2026-10-20T11:32:07.121Z</verm:letztes_update>
  ```
  `dop_kachel` = `32` + EEE + NNNN of the 1 km tile; text fields are **blank-padded** (trim them). Fees/licence as §9.
- **Whole state** (`PROPERTYNAME=verm:dop_kachel,verm:befliegungsdatum,verm:bearbeitungsjahr&COUNT=100000` → 200, 21.6 MB,
  12.7 s, 36,653 tiles, `probe/wfs/dop_bfk_all.xml`):

  | `befliegungsdatum` month | Tiles |
  |---|---|
  | 2019-06 / 2020-08 / 2023-09 | 1 / 1 / 1 |
  | 2024-08 | 269 |
  | 2025-05 / 06 / 08 | 1,634 / 6,493 / 10,493 |
  | 2026-05 / 07 | 13,415 / 4,346 |

  → **100 % May–September (leaf-on)**; 48 % of tiles were flown in 2026. Test tiles: Stuttgart school **2026-05-24**;
  Freiburg school 2025-08-25; Mannheim `460 5484` 2026-07-29; Ulm `573 5360` 2025-08-10.
- **The WMS shows the WFS vintage; the zip tiles can be a year older.**
  - Stuttgart `511 5403`: zip CSV `Aktualitaet 2025-11-08`, flight 720; WFS 2026-05-24. The zip TIFF's top 400 rows (first
    6 MB of the member inflated, `scripts/zippartial.py`) vs the WMS strip `BBOX=511000,5403920,512000,5404000` 5000 × 400:
    **mean abs difference 23.0** of 255, at every ±1 px offset too. Cars and a pool differ
    (`dl/dop_partial/compare_zip_top_wms_bottom.png`) → **different flights**.
  - Freiburg `413 5315`: zip CSV `Aktualitaet 2025-08-25`, WFS 2025-08-25; same comparison → **mean abs difference 3.28**
    (JPEG noise) → **same flight**.
  - So the WMS follows `befliegungsdatum` (inferred from two tiles; no field ties them formally).
- The zip CSV date is not always the flight date: Stuttgart's DOP CSV says `2025-11-08` while the nDOM CSV of the same
  `Bildflugnr` 720 says `bDOM20_Aktualitaet 2025-08-11` (a November flight is implausible for leaf-on imagery). Use the WFS.

### 4.2 WMS (recommended path)

- `https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C`, WMS 1.3.0, "WMS LGL-BW ATKIS Digitale Orthophotos in Farbe
  20 cm Bodenauflösung" (keyword `WMS_LGL-BW_ATKIS_DOP_20_C_V06`, `opendata`).
  - One layer **`IMAGES_DOP_20_RGB`** "Digitale Orthophotos 20 cm Farbe grenzscharf - Landesgrenze", `queryable="0"`.
    **Only GetCapabilities and GetMap** are offered (no GetFeatureInfo); formats `image/png`, `image/png8`, `image/jpeg`
    (PNG came back as an 8-bit palette image).
  - `<MaxWidth>15000</MaxWidth><MaxHeight>15000</MaxHeight>` — **not true.** Width bisection at 0.2 m/px (height 100):
    4000, 5000, 5500, 6000 px → imagery; **6144, 6145, 6400, 6500, 7000, 7500, 8000, 10000, 12000, 15000, 15001, 20000,
    40000 px → HTTP 200 `image/jpeg` that is 100 % white**. 6000 × 6000 → imagery (5.35 MB, 4.3 s); 8000 × 8000 → white.
    There is **no exception** above the limit (even 40000 px) — only a blank image.
- Example (school, 25832, 0.2 m):
  ```
  https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=IMAGES_DOP_20_RGB&STYLES=&CRS=EPSG:25832&BBOX=511690,5403670,512010,5404010&WIDTH=1600&HEIGHT=1700&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 411,703 B, 0.89 s. 25833 at Freiburg (`BBOX=-34200,5339890,-33800,5340290`, 2000²) → 659,460 B, 2.3 s,
  0 % white — negative eastings accepted.
- Speed: 5000 × 5000 (1 km² at 0.2 m) 3.73 MB in **4.9 / 5.5 s**; 6000² 5.35 MB in 4.3 s (≈0.8–1.2 MB/s).
- **Clipped at the state border:** Mannheim/Ludwigshafen 4 × 4 km (`BBOX=457000,5479000,461000,5483000`) → 85.6 % white,
  cut exactly along the Rhine (`dl/wms/border_mannheim_small.jpg`), although DGM/DOM/nDOM tiles cover the RP bank.
- Headers: chunked (no Content-Length), `cache-control: public, max-age=32400`, CORS `*`.

### 4.3 Tile downloads

- `https://opengeodata.lgl-bw.de/data/dop20/dop20rgb_32_511_5402_2_bw.zip`: HEAD 200, **255,823,175 B**; `…513_5402…`
  253,104,740 B; `…413_5314…` 224,135,573 B. Members: **uncompressed** baseline TIFF 5000 × 5000 × RGB 8 bit, 1-row strips,
  (75,030,278 B), deflated to 62–64 MB each; TFW and CSV beside. A window could be read by inflating the member up to the
  needed rows (the north edge is first), but the south edge needs the whole 63 MB stream, and the zips are older than
  the WMS (§4.1). **Not worth it; use the WMS.**
- DOP CSV (one line per tile; `;`-separated, 24 fields): `Kachelname;Aktualitaet;Erfassungsmethode;Bildflugnummer;Kamera_Sensor;
  Bodenpixelgroesse;Spektralkanaele;Koordinatenreferenzsystem_Lage;…;Bezugsflaeche;Koordinatenursprung_East;…;Belaubungszustand;Bemerkungen`.
  Quality is uneven: Freiburg `Koordinatenursprung_East 41300` (a digit missing), Viernheim CRS `2582`, Mannheim 2024 tile
  with empty camera and `Bezugsflaeche`. `Belaubungszustand` is `3` on every sample (summer flights; code meaning not looked up).

### 4.4 WCS

- `WCS_INSP_BW_Orthofoto_DOP20` (survey: caps 200) and `WCS_INSP_BW_Hoehe_Coverage_DGM1` (caps 200, 10,265 B today; survey:
  GetCoverage gives 16-bit big-endian, not the source) — not needed.

---

## 5. DGM1 (terrain model, 1 m)

- Source: ALS. The 1 km tile carries two dates (CSV): `Aktualitaet` (first capture) and `Fortfuehrung` (last update,
  method 5020 both). **The file-name year is the `Fortfuehrung` year.**

  | Tile | CSV (Kachelname; Aktualitaet; …; Fortfuehrung) | Name year |
  |---|---|---|
  | `511 5403` (Stuttgart school) | `2016-04-06` … `2023-02-11` | 2023 |
  | `413 5315` (Freiburg) | `2017-03-23` … `2017-03-23` | 2017 |
  | `459 5483` (Mannheim) | `2016-01-27` … `2022-02-14` | 2022 |
  | `469 5485` (Viernheim border) | `2016-02-27` … `2022-02-15` | 2022 |
  | `572 5359` (Ulm) | `2018-04-14` … `2018-04-14` | 2018 |
  | `573 5359` (Neu-Ulm side) | – | 2021 |

- **DGM Aktualität WFS** `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ATKIS_DGM_Aktualitaet` (dl-de/by-2-0):
  `verm:v_dgm_kacheln` (`dgm_kachel 325115403`, `fortfuehrungsdatum 2023-02-11`, `genauigkeit +/-15cm`, CRS fields),
  `verm:v_dgm_laserflug_durchgefuehrt` (school: `los 27-1_2023`, **`flugdatum 09.02. - 13.02.2023`**, `punktdichte 8 Punkte/m²`,
  `scanner Riegl VQ-780 II`, `flughoehe 547m - 665m`), `verm:v_dgm_laserflug_planung`. Whole state: 36,652 tiles, 16.6 MB,
  4.5 s; **only 10,668 have a `fortfuehrungsdatum`** (2022: 4,099, 2023: 4,380, 2024: 2,189; months Jan–Mar, leaf-off);
  the rest are empty (first ALS campaign only; the CSV then has `Fortfuehrung = Aktualitaet`).
- **XYZ format** (verified on `511 5403`, `459 5483`, `469 5485`, `572 5359`, `573 5359`):
  - `511000.50 5403999.50 357.68\r\n` — E N Z with two decimals, single blanks, **CRLF**, cell centres row by row from the
    NW corner, E fastest; 1,000,000 points (`unique E 1000, unique N 1000`); 28.0–29.0 MB depending on the height digits.
  - **No NoData values** in any sample (no −9999); full tiles across the state border are complete (0 % gaps on the RP,
    Hesse and Bavaria sides).
  - **Partial border tiles** hold whole rows only (`573 5359`: northern 500 rows). freemapper's `parse_xyz` (SH) checks
    every point against the grid and requires all `cells²` points → needs a "missing rows are NaN" mode for BW.
  - Values: school tile 266.07–384.11 m; Mannheim 87.11–96.69 m; Ulm 466.54–512.47 m.
- Reuse: `lvermgeo_sh::parse_xyz` (same layout incl. CRLF) once it accepts a partial grid; members ≤ 29 MB are within
  `zip_range::MAX_MEMBER_BYTES` (64 MiB).

---

## 6. nDSM sources: nDOM1, DOM1, LiDAR

### 6.1 nDOM1 (ready-made, recommended)

- `https://opengeodata.lgl-bw.de/data/ndom1/ndom1_32_511_5402_2_bw.zip` (HEAD 200, 11,285,814 B). Member
  `ndom1_32_511_5403_1_bw.tif` 2,736,375 → 4,195,998 B (1.6 s). Product text (portal): "Normalisiertes Oberflächenmodell …
  GeoTIFF, 1 bis 4 MB, 2 km × 2 km, 4 mal pro Jahr".
- **CSV** (32 fields) carries both inputs, e.g. school:
  `ndom1_32_511_5403_1_bw;2016-04-06;5020;2023-02-11;5020;0.15;…;2025-08-11;5040;RealityStud 2025.1.1;720;UCE M3 …;20cm;RGBI;…;3;keine`
  → DGM part (Aktualitaet, Fortfuehrung) and **`bDOM20_Aktualitaet` 2025-08-11** (image matching, `bDOM20_Bildflugnr` 720,
  `bDOM20_Belaubung 3`). Freiburg: bDOM 2025-08-25 (= its DOP date); Mannheim: bDOM **2024/06/26** — **date format
  `YYYY/MM/DD` in older CSVs**, `YYYY-MM-DD` in newer ones.
- **Two encodings:**

  | Producer | Compression | Layout | NoData | Tiepoint |
  |---|---|---|---|---|
  | newer (school, Viernheim, Ulm; no Software tag) | **none** | 128 × 128 tiles (1000 not a multiple → padded) | **−3.4028235e38** (`GDAL_NODATA`) | `…, 5404000.0001` |
  | older (Mannheim 2024; "LAStools") | LZW, **predictor 3** (floating point) | 512 × 512 tiles, IFD at 192 | −9999 | whole metres |

  Values clamped at 0 (min 0.00). Ulm `572 5359` has 0.1 % NoData (−3.4e38) outside BW; freemapper's `NODATA_BELOW −9998`
  catches both NoData values.
- School footprint (E 511790–511906, N 5403770–5403918): nDOM p10 0.14, **median 6.69**, p90 15.03, max 29.61 m, 64.1 % > 2 m.
- **Some zips hold no nDOM tile even where DGM exists** (`ndom1_32_573_5358_2_bw.zip`: PDF + Meta only).

### 6.2 DOM1 (ALS, leaf-off) and DOM1 − DGM1

- `…/data/dom1/dom1_32_511_5402_2_bw.zip` (9,155,109 B); member `dom1_32_511_5403_1_bw_2023.tif` (2.29 MB) + `.tfw` + CSV
  (same dates as the DGM: ALS 2016 / update 2023-02-11). **Three TIFF encodings seen:** 2023 tile: LZW, **predictor 2**,
  2-row strips, IFD at 8; 2018/2022 tiles: LZW, **predictor 3**, 512 × 512 tiles, IFD at 192. All Float32, NoData −9999,
  "LAStools".
- DOM1 − DGM1 at the school tile: p50 3.99 m, 3.58 % < 0, only 0.108 % < −0.05 m (rounding) → clamp at 0. Over the school
  footprint: median 7.16, p90 15.21 m, 68.6 % > 2 m. **nDOM vs DOM1 − DGM1: mean abs 1.63 m, correlation 0.913** — they
  agree on buildings and differ on vegetation (leaf-on summer matching vs leaf-off spring ALS) and in time (2025 vs 2023).

### 6.3 LiDAR

- "Laserscandaten 2000-2005 als1" is `active: false` in the portal. The bDOM20 point clouds were not looked for. No
  current ALS point cloud on the portal.

### 6.4 Recommendation

**Use the ready-made nDOM1.** It is one file instead of two, already normalised and clamped, and leaf-on like the
DOP. Its bDOM date is the flight date of the zip DOP, which is at most one year older than the WMS. DOM1 − DGM1 would mix
a leaf-off ALS surface of 2016–2024 with a summer orthophoto. Fallback: if a tile is in the DGM zip but not in the nDOM
zip, report the nDSM tile as missing; do not mix sources inside one map.

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **WFS `…DOP_20_Bildflugkacheln_Aktualitaet`** | DOP (= what the WMS shows) | `befliegungsdatum YYYY-MM-DD` per 1 km tile `dop_kachel 32EEENNNN`; one bbox query, 0.17 s |
| **CSV member per 1 km tile** | DGM1, DOM1 | `Aktualitaet` (first capture), `Fortfuehrung` (last update = name year); `YYYY-MM-DD` or `YYYY/MM/DD` |
| CSV member | nDOM1 | `DGM_*` dates + `bDOM20_Aktualitaet` (flight date of the surface, leaf-on) + `bDOM20_Bildflugnr` |
| CSV member | DOP zip | `Aktualitaet` (not always the flight date, §4.1), `Bildflugnummer`, camera |
| **WFS `…DGM_Aktualitaet`** | DGM1 (DOM1) | `v_dgm_kacheln.fortfuehrungsdatum` (10,668 of 36,652 tiles), `v_dgm_laserflug_durchgefuehrt.flugdatum` per lot |
| ETag / Last-Modified | every zip | DGM `513_5402` 2026-07-17, DOM1 2026-09-30, nDOM1 2026-10-19, DOP 2026-03-27 |
| ALKIS `beginn` | per object | ISO timestamp (`2018-02-16T07:36:27Z`); buildings 1995–2026, land use mostly 2011+ |

Suggested mapping:
- orthophoto `currency` = `befliegungsdatum` of the tile at the map centre (check the corners when a map spans tiles; they
  can differ by a year), always leaf-on;
- DGM1 `currency` = CSV `Fortfuehrung` of the centre tile, `published` = the zip's Last-Modified;
- nDSM `currency` = CSV `bDOM20_Aktualitaet`; warn when it differs from the orthophoto date ("surface 2025-08-11 vs photo
  2026-05-24").

---

## 8. ALKIS: buildings and actual land use

### 8.1 WFS `WFS_LGL-BW_ALKIS` (recommended)

- `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS`, "WFS LGL-BW ALKIS", WFS 2.0.0 (GeoServer). Capabilities
  (201,672 B): **Fees "Unentgeltliche Nutzung nach Open Data Lizenz"**, AccessConstraints = dl-de/by-2-0 text (§9).
  - **31 types** `nora:v_al_*`: `flurstueck`, **`gebaeude`**, `grenzpunkt`, `sonstiger_punkt`, **`tatsaechliche_nutzung`**,
    `bauwerk_einrichtung_{p,l,f}`, `festlegung_recht`, `strasse_gewann`, `lagebezeichnung`, `zuordnung_lagebezeichnung`,
    `grenzlinie`, `flur`, `gemarkung`, `gemeinde`, `kreis`, `region`, `regierungsbezirk`, **`land`**, labels,
    `bodenbewertung`, `bodenschaetzung_f`, `grabloch`, `vergleichsstueck`, `tagesabschnitt`.
  - Output formats: GML 3.2 (default `application/gml+xml; version=3.2`), GML 2/3.1, **`application/json`**, SHAPE-ZIP,
    geopackage. **`CountDefault 1000000`, `ImplementsResultPaging TRUE`.** POST "wird derzeit noch nicht unterstützt".
- DescribeFeatureType (`probe/wfs/describe.xml`):
  - `v_al_tatsaechliche_nutzung`: `gml_id`, `objektart` (int), `objektname`, `unterart_typ`, `unterart_id`, `unterart_kuerzel`,
    `unterart_name`, `eigenname`, `pruefdatum`, `pruefergebnis_{id,name}`, `weitere_nutzung_{id,name}`, `klasse`, `beginn`,
    `ende`, `geom` (`gml:SurfacePropertyType`);
  - `v_al_gebaeude`: `gml_id`, `gebaeudefunktion_{id,kuerzel,name}`, `lage_id`, `adressen`, `hochhaus`,
    `lage_oberflaeche_{id,name}`, `nutzung_{id,name}`, `eigenname`, `qualitaet`, `klasse`, `beginn`, `ende`, `geom`
    (`gml:MultiSurfacePropertyType`);
  - `v_al_land`: `land_id`, `land_name`, `beginn`, `ende`, `geom`.
- Request (school, 140 × 170 m):
  ```
  https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=nora:v_al_tatsaechliche_nutzung&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=511780,5403760,511920,5403930,urn:ogc:def:crs:EPSG::25832&COUNT=1000
  → 200 application/gml+xml; version=3.2, 87,835 B, 0.12 s, numberMatched=19
  ```
  `nora:v_al_gebaeude` same box: 30 features, 51,966 B, 0.11 s. 2 × 2 km (GeoJSON): land use 1,015–1,538 objects,
  1.2–2.3 MB, 0.9–1.0 s; buildings 3,077–6,269, 2.5–5.1 MB, 0.9–1.9 s (§8.5). `RESULTTYPE=hits` over 10 × 10 km Stuttgart:
  **86,524 buildings**, 0.17 s.
- **GML shape** (GeoServer, not the AAA suite):
  ```xml
  <wfs:member>
    <nora:v_al_tatsaechliche_nutzung gml:id="v_al_tatsaechliche_nutzung.a8db9799-19f2-4ecd-a389-d05073768968">
      <gml:boundedBy><gml:Envelope srsName="urn:ogc:def:crs:EPSG::25832" srsDimension="2">…</gml:Envelope></gml:boundedBy>
      <nora:gml_id>DEBWL52210003t5E</nora:gml_id>
      <nora:objektart>41007</nora:objektart>
      <nora:objektname>Fläche besonderer funktionaler Prägung</nora:objektname>
      <nora:unterart_typ>FKT (Funktion)</nora:unterart_typ>
      <nora:unterart_id>1100</nora:unterart_id>
      <nora:unterart_kuerzel>ÖFZ</nora:unterart_kuerzel>
      <nora:unterart_name>Öffentliche Zwecke</nora:unterart_name>
      <nora:klasse>1</nora:klasse>
      <nora:beginn>2018-02-16T07:36:27Z</nora:beginn>
      <nora:geom><gml:Polygon srsName="…25832" srsDimension="2" gml:id="….geom"><gml:exterior><gml:LinearRing><gml:posList>…
  ```
  Buildings: `<nora:geom><gml:MultiSurface …><gml:surfaceMember><gml:Polygon …>`. Geometry elements: `Polygon`,
  `MultiSurface`, `surfaceMember`, `exterior`, `interior`, `LinearRing`, `posList` — all in freemapper's
  `GEOMETRY_ELEMENTS`. **But the geometry property is `geom`, not `geometrie`**, the id is `gml_id`, not `oid`, and the
  attribute names differ → `parse_gml` would lose the geometry (`in_geometry` tests `path[2] == "geometrie"`) and paging
  would not dedup (no `oid`). `SORTBY=oid` → **400** `InvalidParameterValue` "Illegal property name: oid for feature type
  nora:v_al_gebaeude"; use `SORTBY=gml_id`.
- **Paging** `SORTBY=gml_id&COUNT=10&STARTINDEX=0/10/20` → 10/10/10, disjoint (`DEBWL52210003vzR…J4L`, `…Jb4…jOy`,
  `…jQ1…NLT`), `numberMatched="30"` on every page.
- **Errors:** `BBOX=foo` → **400** `application/xml` OWS ExceptionReport `NoApplicableCode`; an **unknown type → 200
  `text/html` "Request Rejected … Your support ID is: 874019351812177928"** (the WAF, not GeoServer).
- **ALKIS stops at the state border:** RP tile `459 5483` 104 buildings / 53 land-use objects, Hesse tile `469 5485` 6 / 28,
  Bavaria tile `572 5359` 541 / 131 — **0 features outside the BW polygon** in each; Viernheim 1 km box 0 / 0, Neu-Ulm 0 / 1
  (a sliver inside BW), Ludwigshafen `456500,5480500,457500,5481500` hits 0.

### 8.2 Mapping to freemapper's model (`ave:` keys)

| freemapper key | `v_al_gebaeude` | `v_al_tatsaechliche_nutzung` |
|---|---|---|
| `oid` | `gml_id` (16 chars, `DEBWL5…`/`DEBWL0…`; no `BL`/`TN` suffix) | `gml_id` |
| `gebnutzbez` | constant **"Gebäude"** (the view holds AX_Gebaeude only: no Bauteil, no Überdachung, no Turm) | – |
| `funktion` | `gebaeudefunktion_name` (LGB spellings: "Allgemein bildende Schule", "Wohnhaus", "Tiefgarage") | – |
| `rellage` | `lage_oberflaeche_name` ("Unter der Erdoberfläche" = freemapper's `UNDERGROUND`; "Aufgeständert") | – |
| `name` | `eigenname` | `eigenname` |
| `aktualit` | `beginn[..10]` | `beginn[..10]` |
| `nutzart` | – | `objektname` |
| `bez` | – | `unterart_name` **only when `unterart_id` is set** (otherwise `unterart_name` repeats `objektname`: verified 0 exceptions either way over 5,927 objects) |

`Region::of_object_id` needs a **`"DEBW"`** arm. `ende` was empty on all 30,378 sampled objects.

### 8.3 Sample features at the Eberhard-Ludwigs-Gymnasium (WFS, geometry elided)

```json
{"gml_id":"DEBWL52210003t5E","objektart":41007,"objektname":"Fläche besonderer funktionaler Prägung","unterart_typ":"FKT (Funktion)","unterart_id":1100,"unterart_kuerzel":"ÖFZ","unterart_name":"Öffentliche Zwecke","klasse":1,"beginn":"2018-02-16T07:36:27Z"}
{"gml_id":"DEBWL52210005fAq","objektart":41002,"objektname":"Industrie- und Gewerbefläche","unterart_id":1400,"unterart_kuerzel":"HDL","unterart_name":"Handel und Dienstleistung","klasse":2,"beginn":"2018-02-16T07:36:27Z"}
{"gml_id":"DEBWL52210003xQ7","gebaeudefunktion_id":1010,"gebaeudefunktion_kuerzel":"Whs","gebaeudefunktion_name":"Wohnhaus","lage_id":"08111000221800074","hochhaus":false,"klasse":1,"beginn":"1997-09-29T23:00:00Z"}
```

The school parcel's land use (`DEBWL52210003t5E`, envelope E 511788.84–511905.46, N 5403768.77–5403920.14) matches the OSM
footprint (E 511790–511906, N 5403770–5403918) within 2 m.

### 8.4 Alternatives (not recommended, but verified)

- **OGC API beta** `https://ogcapi.lgl-bw.de/beta/ldproxy/gebaeude/collections/v_al_gebaeude/items?f=json&bbox=…&bbox-crs=http://www.opengis.net/def/crs/EPSG/0/25832&crs=…/25832&limit=10000`
  → same view (same property names). 1 × 1 km: 1,473 buildings, 1.02 MB, 2.4 s; 2 × 2 km `limit=20000`: 5,805, 3.76 MB,
  3.4 s, no `next` link. **No land use** (only `flurstueck`, `gemarkung`, … without Nutzung). Beta, labelled CC BY 4.0.
- **Per-Gemarkung ALKIS Shape** `https://opengeodata.lgl-bw.de/data/alkis/ALKIS-oE_08{gemarkung_id:04}_{name}_shp.zip`
  (name as in the Gemarkung list, UTF-8 percent-encoded, spaces and dots kept: `ALKIS-oE_081515_Weil%20der%20Stadt_shp.zip`,
  `ALKIS-oE_086020_St.%20Georgen_shp.zip`, `ALKIS-oE_080001_Freudenberg_shp.zip` → 200; a wrong name → 404). Stuttgart zip
  56.5 MB, members `nutzung.{shp,shx,dbf}`, `nutzungFlurstueck.*`, `gebaeudeBauwerke.*`, … = **the ALKIS-vereinfacht schema
  as Shape**, readable by range (nutzung.shp 9.5 MB compressed). But **`nutzart` and `gebnutzbez` are ASCII-folded and
  title-cased** ("Strassenverkehr", "Industrie Und Gewerbeflaeche", "Unland Vegetationslose Flaeche", "Gebaeude",
  "Sonstiges Bauwerk Oder Sonstige Einrichtung"), `bez` keeps umlauts; the buildings have **no `rellage`**; one Stuttgart
  `gebaeudeBauwerke.dbf` inflates to 70.5 MB (> `MAX_MEMBER_BYTES`); quarterly. A Gemarkung lookup is needed (OGC API
  `gemarkung` items: 3,380 Gemarkungen). Compared with the API over 1 km²: 1,469 of 1,470 buildings in common; the API flags
  74 underground, the Shape's "Tiefgarage" rule finds 59.
- **INSPIRE** `WFS_INSP_BW_Gebaeude_ALKIS` (`bu-core2d:Building`, open, `CountDefault 500000`, no paging; GetFeature 200 but
  `numberReturned="0"` with members inside) and `WFS_INSP_BW_Bodennutzung_ALKIS` (`elu:ExistingLandUseObject`, HILUCS
  codes, open). The land-cover WFS `WFS_INSP_BW_Bodenbedeckung_ALKIS` is **"Kostenpflichtige Nutzung"** (needs a
  Nutzungsvereinbarung). HILUCS/INSPIRE schemas would need new parsers and lose the `nutzart`/`bez` classes.
- **`ln_bw.zip`** (Landnutzung): see §8.7 — not usable per area.

### 8.5 Observed values (seven 2 × 2 km areas: 24,451 buildings, 5,927 land-use objects; `probe/alkis/areas_stats.txt`)

- **`objektname`** (all LGB spellings): Wohnbaufläche 1,489, Straßenverkehr 1,301, Industrie- und Gewerbefläche 1,046,
  Landwirtschaft 457, Fläche besonderer funktionaler Prägung 427, Weg 381, Sport-, Freizeit- und Erholungsfläche 370, Platz 109,
  Fließgewässer 103, Bahnverkehr 86, Wald 49, Fläche gemischter Nutzung 40, **Unland/Vegetationslose Fläche 38** (LGB form),
  Stehendes Gewässer 14, Gehölz 13, Friedhof 3, Hafenbecken 1. `PRIVATE_LAND_USES` matches unchanged.
- **`bez` (from `unterart_name` with `unterart_id`)** per class:
  - `Industrie- und Gewerbefläche`: Handel und Dienstleistung 751, Gebäude- und Freifläche Industrie und Gewerbe 184,
    … Versorgungsanlage 89, Lagerfläche 12, … Entsorgungsanlage 6, Umspannstation 4;
  - `Landwirtschaft` (VEG): Gartenbauland 374, Grünland 52, Ackerland 26, Rebfläche 5; `Wald` (VEG): Nadelholz 24, Laub- und
    Nadelholz 14, Laubholz 11; `Gehölz` none;
  - FBFP: Öffentliche Zwecke 421, Historische Anlage 4, Kultur 1, Medien und Kommunikation 1;
  - SFE: Grünanlage 310, Gebäude- und Freifläche Sport, Freizeit und Erholung 36, Sportanlage 16, Garten 5, Freizeitanlage 2,
    Botanischer Garten 1;
  - `Platz`: **Parkplatz 34, none 75 (69 %)** → `square_bez`;
  - `Fließgewässer`: Bach 35, **Kanal 34**, Fluss 26, Graben 8; `Stehendes Gewässer`: Teich 11, See 3; `Hafenbecken` none;
  - `Fläche gemischter Nutzung`: none 24, Gebäude- und Freifläche Land- und Forstwirtschaft 11, Landwirtschaftliche
    Betriebsfläche 5;
  - **never a `bez` on Wohnbaufläche, Straßenverkehr, Weg, Bahnverkehr, Unland, Friedhof** — in particular **no
    "Begleitfläche Straßenverkehr"** (verges) and no Fußgängerzone/Fahrweg: BW records no Funktion there.
- `unterart_typ`: "FKT (Funktion)" or "VEG (Vegetationsmerkmal)". `weitere_nutzung` empty. `eigenname` on 14 (parks,
  allotments, "Israelitischer Friedhof"). `pruefergebnis_name` on 97 ("Erfassung eines neuen Objektes", …).
- Geometry: land use all `Polygon` (217 with holes); buildings all `MultiPolygon` (18 with > 1 part).
- **Buildings `gebaeudefunktion_name`**: Wohnhaus 11,603, Garage 4,162, Wohn- und Geschäftsgebäude 1,926, Betriebsgebäude 1,373,
  Schuppen 934, **Tiefgarage 625**, Geschäftsgebäude 621, Bürogebäude 485, … **Allgemein bildende Schule 177**, … Schutzbunker 10.
- **`lage_oberflaeche`**: none 23,777, **1200 "Unter der Erdoberfläche" 672** (Tiefgarage 625 — every Tiefgarage —, Gebäude für
  Vorratshaltung 10, Geschäftsgebäude 7, Wohnhaus 7, Schutzbunker 6, …), 1400 "Aufgeständert" 2.
- `eigenname` on 263 buildings (schools, museums: "Bismarckgymnasium", "Staatl. Kunsthalle"); `hochhaus` boolean; `klasse` 1–4.
- The Titisee box (Black Forest) has 32 buildings and 113 land-use objects.

### 8.6 What is missing or different

- `nora:` schema instead of `ave:` (property mapping §8.2, geometry property `geom`, `SORTBY=gml_id`).
- Buildings view = AX_Gebaeude only; underground via `lage_oberflaeche`.
- No verge class on Straßenverkehr → every `Straßenverkehr` polygon (incl. green verges) counts as paved.
- `Platz` without Funktion 69 % → `square_bez: Some("ohne Unterart (Baden-Württemberg)")`.
- WAF rejection is 200 HTML.

### 8.7 Landnutzung GPKG `ln_bw.zip` (asked: can it be range-read?) — no

- `https://opengeodata.lgl-bw.de/data/ln/ln_bw.zip`: HEAD 200, **2,391,757,913 B**, Last-Modified 2026-07-07. Central directory
  (last 256 KiB): `bw_32_ln_2026_02.gpkg` **method 8 (deflated)**, 2,391,379,309 → **4,217,212,928 B**, plus the licence PDF,
  `ln_bw_informationsdatei.txt` ("Prozessierungsdatum: 2026-07-06"), `Meta-Landnutzung.txt`. The portal now says "4,8 GB".
- **A deflated member has no random access**: SQLite pages (R-tree, table b-trees) can only be reached by inflating the stream
  from the start, on average half of 4.2 GB, for every area. Range reading the GPKG is not possible; the whole 2.39 GB must
  be downloaded and 4.2 GB unpacked.
- It is also **not ALKIS TN**: the AdV "Landnutzung" model (`ln_bw_shp.zip`, 2,349,189,445 B, 20 class files
  `221100_ln_wohnnutzung`, `222100_ln_strassenundwegeverkehr` (961 MB compressed), `223200_ln_forstwirtschaft`, …; fields
  `uuid`, `modellart`, `fkt`, `zus`, `nam`, `dlu`, `edu`, `iwn`, `man`, `LN_ID`, `qid`) → a new class mapping would be needed.
- Other statewide zips (HEAD): `hu_bw.zip` 515,375,738 B (Hausumringe Shape, `Gebaeude-BW.shp` 1.26 GB raw), `lb_bw.zip`
  3,346,488,084 B (Landbedeckung GPKG 28.1 GB raw), `Verw-Grenzen_ALKIS.zip` 40,028,867 B, `bdlm_bw_shp.zip` 774,961,584 B.

### 8.8 Land use: options and costs

| Option | Per map area (2 × 2 km) | One-off | Schema fit | Verdict |
|---|---|---|---|---|
| **ALKIS WFS `nora:v_al_tatsaechliche_nutzung`** | **1–3 MB, ≈1 s** | none | `nutzart`/`bez` in LGB spelling after renaming | **recommended** |
| Gemarkung Shape zips | 1–10 MB per Gemarkung (range-read `nutzung.*`), + Gemarkung lookup | none | ASCII-folded `nutzart` → spelling map; quarterly | fallback only |
| `ln_bw.zip` GPKG | 0 after download | **2.39 GB download, 4.2 GB disk**, refresh twice a year | AdV LN codes, new mapping | no |
| `ln_bw_shp.zip` per class | streets class alone 961 MB | 2.35 GB | same | no |
| OSM only | 0 | extract | no ALKIS classes, no out-of-bounds rule | no |

---

## 9. Licence and attribution

- Licence: **Datenlizenz Deutschland – Namensnennung – Version 2.0** (`https://www.govdata.de/dl-de/by-2-0`); every zip
  holds `GOVDATA-Datenlizenz_Deutschland.pdf`.
- **Open-data page** `https://www.lgl-bw.de/Produkte/Open-Data/` (fetched 200, 339,779 B), verbatim:
  > Die offenen Geobasisdaten und Geodatendienste der Vermessungsverwaltung Baden-Württemberg können unter den Bedingungen
  > der Datenlizenz Deutschland - Namensnennung - Version 2.0 ( http://www.govdata.de/dl-de/by-2-0 ) unentgeltlich genutzt
  > werden. Die Namensnennung hat in folgender Weise zu erfolgen: **„Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0“**.
- **Capabilities** (DOP WMS, ALKIS WFS, currency WFS, INSPIRE open WFS) `Fees` "Unentgeltliche Nutzung nach Open Data Lizenz",
  `AccessConstraints`, verbatim:
  > Es gelten die Lizenzbedingungen „Datenlizenz Deutschland - Namensnennung - Version 2.0“ bzw. „dl-de/by-2-0” (
  > https://www.govdata.de/dl-de/by-2-0 ) mit den dort geforderten Angaben zum Quellenvermerk. Als Rechteinhaber und
  > Bereitsteller ist die Datenhaltende Stelle, sowie das Jahr des Datenbezugs in Klammern anzugeben. Beispiel für
  > Quellenvermerk: LGL-BW (2024) Datenlizenz Deutschland - Namensnennung - Version 2.0, www.lgl-bw.de
- The OGC API beta landing pages link **CC BY 4.0** instead — one more reason to stay on the WFS.
- dl-de/by-2-0 asks that changes be marked. **Recommended freemapper line:** `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0
  (Daten bearbeitet, <download year>)` — the open-data page's form, plus the year the capabilities ask for and the change note;
  licence link in the project notes. Combined with `© OpenStreetMap contributors, ODbL` and the freemapper credit line.

---

## 10. State boundary (for `assets/regions/baden-wuerttemberg.geojson`)

- **Source: the ALKIS WFS, type `nora:v_al_land`:**
  ```
  https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=nora:v_al_land&SRSNAME=urn:ogc:def:crs:EPSG::25833&OUTPUTFORMAT=application/json
  ```
  → 200, **1,977,090 B, 0.68 s**, 1 feature `{"land_id": 8, "land_name": "Baden-Württemberg", "beginn": "2026-07-01T00:00:00Z"}`
  (25832: 1,391,702 B, 0.80 s). Licence dl-de/by-2-0 (§9). Alternatives: OGC API `bundesland` (Basis-DLM, CC BY), `Verw-Grenzen_ALKIS.zip`.
- Geometry (shapely 2.2.0, 25833): **MultiPolygon of 16 parts, valid, 61,108 vertices, one hole**:
  - main part 35,879.0 km² with a **0.62 km² hole at 49.618°N 8.679°E** (an enclave of Hesse; 94.5 % of it lies in the repo's
    `hessen.geojson`);
  - **Büsingen am Hochrhein** 7.66 km² (exclave in Switzerland, 47.70°N 8.69°E), **Reichenau** 4.45 km², **Mainau** 0.45 km²,
    small Lake Constance islands/parts (0.0002–0.14 km²), and two pieces of 0.022 and 0.021 km² at 49.61°N 8.68°E (Odenwald).
  - Lake Constance itself is not inside (no agreed border).
- **25 m topology-preserving simplification** → `bw/baden-wuerttemberg.geojson` (script `scripts/simplify_bw.py`: shapely
  `simplify(25, preserve_topology=True)`, `set_precision(grid_size=1)`, `normalize`, exteriors oriented clockwise):
  - FeatureCollection, `name` "Baden-Württemberg (LGL-BW, simplified 25 m)", `crs` `urn:ogc:def:crs:EPSG::25833`, properties
    `name`, `source` ("LGL-BW WFS_LGL-BW_ALKIS (nora:v_al_land), LGL-BW (2026) Datenlizenz Deutschland - Namensnennung - Version 2.0,
    www.lgl-bw.de [Daten bearbeitet]"), `retrieved 2026-11-08`, `simplified_m 25`; integer coordinates — same layout as
    `assets/regions/rheinland-pfalz.geojson`.
  - **6,561 vertices, 109,086 B**, 16 parts, 1 hole, **valid (shapely)**, exteriors clockwise. The repo test
    `region_boundaries_are_valid` (geo `check_validation`) was not run.
  - Area 35,892.44 km² (full 35,891.82), symmetric difference 9.58 km², **Hausdorff 26.1 m**.
  - Extent E −61,679 … 168,532, N 5,283,662 … 5,530,172; 16.8 % at negative 25833 eastings.
- **Against the neighbour assets** (`scripts/border_bw.py`; shared border = BW boundary within 150 m of the neighbour):

  | Neighbour asset | Shared border | Overlap (asset ∩ asset) | Gap (within 60 m of both borders) |
  |---|---|---|---|
  | `assets/regions/hessen.geojson` | 181.3 km | 79,633 m² in 498 pieces | 135,291 m² |
  | `assets/regions/rheinland-pfalz.geojson` | 93.9 km | 359,087 m² in 87 pieces | 426,933 m² |

  **The official BW and RP Landesgrenzen disagree along the Rhine** (full BW `v_al_land` vs the full RP Landesgrenze from
  `rp/probe/boundary/r1/rp_full_25833.wkt`): overlap 150,181 m² in 69 pieces (largest 22,906 m²), gaps 369,008 m²; distance
  from the BW border to the RP border sampled every 50 m over 94.1 km: **p50 2.9 m, p90 14.8 m, p99 31.0 m** (max 200 m at the
  French tripoint, 48.966°N 8.230°E). Overlap pieces up to 18.8 m mean width (49.125°N 8.366°E; 49.454°N 8.442°E). So near
  the Rhine a point can lie in both or neither asset; `regions_for` buffers by `MARGIN_M`, which absorbs this.
- Neighbours: Hesse, Rhineland-Palatinate, Bavaria (no asset → beyond-coverage warning, ADR 0034), France, Switzerland
  (Büsingen enclave, Basel), Austria (Lake Constance).

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/baden-wuerttemberg-latest.osm.pbf` → redirect (followed with `curl -L`; code not recorded) → `baden-wuerttemberg-261107.osm.pbf`
  → 200, **652,827,459 B**, `Last-Modified: Sun, 08 Nov 2026 02:02:55 GMT`; `…/baden-wuerttemberg-updates/state.txt`
  `timestamp=2026-11-07T21\:21\:36Z sequenceNumber=4957`. Within `MAX_EXTRACT_BYTES` (2 GiB), but 0.7 × the NRW extract
  that ADR 0028 split by district.
- **Regierungsbezirk extracts** (all 200, `-261107` files, state.txt `sequenceNumber=4951`):

  | Extract | Size | Poly |
  |---|---|---|
  | `baden-wuerttemberg/freiburg-regbez-latest.osm.pbf` | 160,388,681 B | `freiburg-regbez.poly` 12,260 B |
  | `baden-wuerttemberg/karlsruhe-regbez-latest.osm.pbf` | 155,156,205 B | 13,785 B |
  | `baden-wuerttemberg/stuttgart-regbez-latest.osm.pbf` | 212,409,827 B | 10,989 B |
  | `baden-wuerttemberg/tuebingen-regbez-latest.osm.pbf` | 128,374,062 B | 8,571 B |

- Polys (shapely, valid): Stuttgart school → stuttgart; Freiburg school → freiburg; **Büsingen → freiburg**; Mannheim → karlsruhe;
  Ulm → tuebingen; **Viernheim, Ludwigshafen, Neu-Ulm → none** (the state poly `baden-wuerttemberg.poly`, 11,113 B, excludes
  them as well). A Mannheim map that reaches Ludwigshafen needs the RP extract; one reaching Viernheim the Hesse extract.
- Overpass answered 200 with `freemapper-research/0.1 (github.com/HenryJobst/freemapper)` (49 s for the school query).

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `opengeodata.lgl-bw.de/data` | HEAD/Range/ETag/Last-Modified OK; Range → 206; missing → 404; directories → **403**; ≈0.2 s per small range, ≈3 MB/s; `TS…`/`MyCookie` cookies; members deflated, CRC verified |
| `owsproxy.lgl-bw.de/…/WMS_LGL-BW_ATKIS_DOP_20_C` | **≥ 6144 px → 200 blank white JPEG** (announced 15000); no GFI; clipped white outside BW; 5000² in ≈5 s; CORS `*` |
| `owsproxy.lgl-bw.de/…/WFS_LGL-BW_ALKIS` | GeoServer; GML 3.2 / JSON; `CountDefault 1000000`; paging with `SORTBY=gml_id`; bad parameter → 400 OWS report; **unknown type → 200 text/html WAF page**; ≈2–3 MB/s; a `Range` header is honoured (206) |
| currency WFS (`…Bildflugkacheln_Aktualitaet`, `…DGM_Aktualitaet`) | small bbox 0.15–0.26 s; whole state 12.7 s / 4.5 s; padded text fields |
| `ogcapi.lgl-bw.de/beta` | beta; CC BY 4.0 label; `/beta/ldproxy/collections` → 404 (per-dataset landing pages only) |
| Geofabrik | redirect to the dated file; 653 MB state, 128–212 MB per district |

- **TLS** (`openssl s_client -showcerts -CAfile <certifi> -verify_return_error`): `owsproxy.lgl-bw.de`, `opengeodata.lgl-bw.de`,
  `ogcapi.lgl-bw.de`: TLS 1.3, 3 certs `*.lgl-bw.de` → Thawte TLS RSA CA G1 → **DigiCert Global Root G2**, `Verify return
  code: 0 (ok)`; `download.geofabrik.de`: Let's Encrypt YE1 → Root YE → ISRG Root X2 → ISRG Root X1. DigiCert Global Root G2
  is in `webpki-roots` 1.0.11 (freemapper's lock file; grepped the crate source). No Rust/ureq run was made.
- A `User-Agent` of `freemapper/0.1 (github.com/HenryJobst/freemapper)` or `ureq/3.4.2` passes the WAF (206 on both hosts).
- No authentication, no `Retry-After`, no throttling in ≈400 requests.
- Total transfer for this research ≈ 0.25 GB; disk under `bw/` ≈ 527 MB (inflated XYZ/DBF samples).

---

## 13. What makes the Baden-Württemberg adapter differ (risks first)

1. **WMS silent blank above ≈6000 px.** The capabilities promise 15000; ≥ 6144 px returns a valid white JPEG with status 200.
   freemapper must cap pieces (4000 px) and should treat an all-white piece *inside* BW as an error.
2. **The DOP date is not in the WMS.** It comes from a separate WFS per 1 km tile (`befliegungsdatum`); the WMS shows that
   vintage, the zip CSV may be a year older. The tie between WMS and WFS is inferred from two tiles (§4.1).
3. **ALKIS is a different schema** (`nora:` GeoServer views): geometry property `geom`, id `gml_id`, `SORTBY=gml_id`, names
   to map (`objektname`→`nutzart`, `unterart_name`→`bez` only with `unterart_id`, `lage_oberflaeche_name`→`rellage`,
   `gebaeudefunktion_name`→`funktion`, `beginn`→`aktualit`, constant `gebnutzbez "Gebäude"`). `parse_gml` today drops the
   geometry (`geometrie` hard-coded) and dedups by `oid`.
4. **WAF answers 200 HTML** for rejected requests → the WFS validator must reject non-GML bodies (it does via `parse_gml`'s
   root check; keep a test).
5. **DGM1 is XYZ inside zips, and border tiles are partial** (whole rows missing); nDOM zips can be empty where DGM exists;
   border zips list only BW-touching tiles. `parse_xyz` needs a partial-grid mode; "tile absent from the directory" must
   be a normal outcome (outside BW), not an error.
6. **Three Float32 TIFF encodings** (strips + predictor 2; 512 px tiles + LZW predictor 3; 128 px tiles uncompressed with
   NoData −3.4e38). Predictor 3 on the main `tiff` 0.11.3 path is not verified in Rust → fixture test.
7. **No verge class** (`Begleitfläche Straßenverkehr`) and no Funktion on Straßenverkehr/Weg → paved areas include
   verges; 69 % of `Platz` without `bez`.
8. **CSV date formats vary** (`YYYY-MM-DD`, `YYYY/MM/DD`) and DOP CSV fields are sloppy; parse leniently, prefer the WFS.
9. **Rhine border:** the official BW and RP lines differ by up to ≈30 m (p99) → overlaps/gaps between assets; Mannheim maps
   read both ALKIS services and two OSM extracts; the WMS is white across the Rhine although rasters cover it.
10. **OSM 653 MB** → per-Regierungsbezirk extracts (four polys to ship) like NRW.
11. CRS 25832 everywhere; 16.8 % of BW at negative 25833 eastings (Upper Rhine) → `Crs::Utm32`, no 25833 `TileId`.
12. Unchanged from other states: no login, DHHN2016, NW-corner tiepoint, dl-de/by-2-0, LGB spellings in the WFS.

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Baden-Württemberg | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `WmsService { url: "https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C", layer: "IMAGES_DOP_20_RGB", crs: Crs::Utm32, extra: "", max_px: 4000 }`, JPEG | Date: WFS `…/wfs/WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet`, `TYPENAMES=verm:v_dop_20_bildflugkacheln`, small bbox at the centre, `befliegungsdatum` of the tile containing the centre; leaf-on. Treat an all-white piece inside BW as an error. |
| **DTM** (hillshade, slope, contours) | **DGM1 XYZ members** of `https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_{Ez}_{Nz}_2_bw.zip` by range (`zip_range`), member `dgm1_32_{E}_{N}_1_bw_*.xyz` | Parse with `parse_xyz` in a partial-grid mode (absent rows NaN); currency = CSV `Fortfuehrung`; `published` = Last-Modified. |
| **nDSM template** | **nDOM1** members `ndom1_32_{E}_{N}_1_bw.tif` of `…/data/ndom1/ndom1_32_{Ez}_{Nz}_2_bw.zip` | `decode_float_tiff` (tiled, uncompressed or LZW pred 3); currency = CSV `bDOM20_Aktualitaet` (leaf-on); warn when ≠ orthophoto date. DOM1 − DGM1 not used. |
| **Buildings** | **WFS** `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS`, `nora:v_al_gebaeude`, 25832, `SORTBY=gml_id&COUNT=10000&STARTINDEX=…` | Through `alkis_wfs` with a `nora` schema mapping (§8.2); underground via `lage_oberflaeche_name`; oid prefix `DEBW`. |
| **Land use** (out-of-bounds, paved, water) | the same WFS, **`nora:v_al_tatsaechliche_nutzung`** | `WfsService { name: "LGL-BW ALKIS", url: "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS", buildings_type: "nora:v_al_gebaeude", land_use_type: "nora:v_al_tatsaechliche_nutzung", crs: Crs::Utm32, square_bez: Some("ohne Unterart (Baden-Württemberg)") }` + schema. Cost ≈1–3 MB / 1 s per 2 × 2 km; no bulk download. |
| **Region boundary** | `nora:v_al_land` in 25833, simplified 25 m | Ready: `bw/baden-wuerttemberg.geojson` (6,561 vertices, 16 parts, 1 hole, valid). Region CRS Utm32. |
| **OSM** | Geofabrik **Regierungsbezirk** extracts `baden-wuerttemberg/{freiburg,karlsruhe,stuttgart,tuebingen}-regbez-latest.osm.pbf` + their `state.txt` | 128–212 MB each; reuse the NRW district mechanism (ADR 0028) with four 25 m poly assets. |
| **Attribution** | `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten bearbeitet, <year>)` + licence link + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9. |

---

## 15. Not verified, or open

- That the WMS always shows the `befliegungsdatum` vintage: shown for two tiles only (Stuttgart differs from the zip, Freiburg
  equals it). No field links the WMS to the WFS. A third check in a 2024 tile would strengthen it.
- The exact WMS size limit between 6001 and 6143 px, and whether it depends on height or area (only width bisected, 6000² ok).
- `Belaubungszustand`/`bDOM20_Belaubung` code `3`: meaning not looked up (assumed "belaubt" from the summer dates).
- Rust checks: no ureq/`tiff` run was made. Open points are predictor 3 on 512 px LZW tiles through the main `read_image`
  path, and the uncompressed 128 px tiles padded beyond 1000 px. The `float_tiff` comments claim both work.
- The repo test `region_boundaries_are_valid` (geo `check_validation`) on the new asset; a full freemapper run.
- The WFS above 1,000,000 features without COUNT (default cap) — irrelevant for map areas (10 × 10 km Stuttgart: 86,524).
- Long-term stability of `WFS_LGL-BW_ALKIS` (it is not advertised on the open-data page; found through the catalogue). The
  OGC API beta over the same views is the documented successor and the Gemarkung Shape zips the bulk fallback.
- Border tiles towards France/Switzerland (DGM/nDOM completeness); WMS clipping there.
- Why some zip DOP CSV dates (`2025-11-08`) are later than the flight; the CSW record PDFs linked from `Meta-*.txt` were not
  opened.
- Rate limits / fair-use rules: none seen, none found on the open-data page.

---

## 16. As built (2026-11-13, ADR 0118)

Re-checked on 2026-11-13 before building: the ALKIS WFS (19 land use areas and 30 buildings in the school box, the
same counts as on 2026-11-08), the WMS, the WFS of the flight tiles, the archives `dgm1_32_511_5402_2_bw.zip` and
`ndom1_32_511_5402_2_bw.zip` and the four Geofabrik extracts answered as recorded above.

The adapter is `crates/adapters/de/manifests/badenwuerttemberg/` plus `crates/adapters/de/src/lgl_bw.rs`: id
`badenwuerttemberg`, provider `lglbw`, backend `lgl-bw`, EPSG:25832, object ids `DEBW`.

- **ALKIS** is code (`LglBwAlkis`), because the views are not the simplified schema. The WFS reader of
  `freemapper-sources` got two small things for it: a geometry named `geom` is read, and `fetch_wfs_by` pages and
  de-duplicates by a key of the caller (`gml_id`). The mapping is §8.2; attributes outside it are dropped.
- **Orthophoto**: the WMS in pieces of at most 4 000 px, the flight date from the WFS of the flight tiles for the
  tile at the centre of the area. The check "an all-white piece inside the state is an error" (§13.1) was not
  built: at 4 000 px the white answer does not occur.
- **Terrain**: the XYZ member of each 1 km tile out of the archive of its 2 km block; rows missing at the border
  stay empty. The cache keeps a tile as a GeoTIFF named after the tile and the CRC-32 of its member.
- **Surface**: the nDOM1 member of each tile. The project stores a surface model and `derive` subtracts the terrain,
  so the backend stores nDOM1 + DGM1. The date of the record is `bDOM20_Aktualitaet`.
- **Dates**: both CSV headers were read on 2026-11-13. The DGM header is `Kachelname;Aktualitaet;…;Fortfuehrung;…`,
  the nDOM header puts a blank after each semicolon (`Kachel; DGM_Aktualitaet; …; bDOM20_Aktualitaet; …`).
- **OSM**: four extracts, with boundaries from Geofabrik's `.poly` files of 2026-11-13 (25 m, EPSG:25833). Together
  they cover the state boundary without a gap; Ludwigshafen and Neu-Ulm lie in none.
- **Credit**: `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten geändert, <year>)`.
- **Not used**: DOM1 (laser scan, leaf-off, up to nine years old), the orthophoto archives, the OGC API beta, the
  Gemarkung archives, the INSPIRE services, `ln_bw.zip`.
