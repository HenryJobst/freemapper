# Thuringia (TLBG / GDI-Th) data sources for freemapper — live-verified reference

Verified live on 2026-11-08, 01:20–03:00 UTC (Sunday night, Europe/Berlin). Provider: **Thüringer Landesamt für
Bodenmanagement und Geoinformation (TLBG)**, Erfurt; the data infrastructure is run as **GDI-Th** (Geodateninfrastruktur
Thüringen) and the source note is always **"© GDI-Th"**. Older texts and metadata still say TLVermGeo (the former
Landesamt für Vermessung und Geoinformation). Open data are catalogued in **GeoMIS.Th** (GeoNetwork,
`geomis.geoportal-th.de`) and harvested into GovData. Four hosts matter:

- the services host **`www.geoproxy.geoportal-th.de`** ("Geoproxy", deegree 3.4 behind Oracle Access Manager): DOP WMS,
  ALKIS-vereinfacht WFS, boundary WFS, DGM hillshade WMS, INSPIRE services. **Works, no login.**
- the download host **`geoportal.geoportal-th.de`** (GAIA-light `dladownload` app, the same software as SH's Downloadportal;
  ATOM feeds `…/dienste/atom_th_*`; tile files under `…/hoehendaten/…`, `download.php?type=op&id=…`). **Down during the
  whole session**: every path answers with the same 970-byte HTML "Serviceinformation … Dieser Geodaten-Dienst ist im
  Moment leider noch nicht verfügbar" page, **HTTP 200 (or 206 to a Range request)**. Last good Wayback capture of its
  pages: 14.10.2026.
- the portal **`geoportal.thueringen.de`** (and `tlbg.thueringen.de`): **every request from a script gets a Link11 bot
  CAPTCHA page with HTTP 200** (also with a browser User-Agent, also via WebFetch). The CAPTCHA was not circumvented.
  The download pages were read from the Wayback Machine (captures up to 13.09.2026).
- the catalogue **`geomis.geoportal-th.de`** (CSW 2.0.2 and `/api/records/<uuid>/formatters/xml` work; the JSON search API
  needs a CSRF token).

No login, registration, token or form is needed for the services. **Blocker:** while the download host is down, there is
**no open source of DGM1, DOM1 or LiDAR for Thuringia** (no WCS exists; §5, §6). The orthophoto (WMS) and ALKIS (WFS)
paths are not affected.

Test locations:

| School / place | OSM | Centre EPSG:25832 | Centre EPSG:25833 | lat / lon | Footprint bbox 25832 | 1 km tile | 2 km tile |
|---|---|---|---|---|---|---|---|
| **Carl-Zeiss-Gymnasium**, Erich-Kuithan-Straße 7, Jena (primary) | way **330874575** | E 683250 / N 5647802 | E 261846 / N 5650036 | 50.95266 / 11.60906 | E 683147–683351, N 5647678–5647940 | `32 683 5647` (fully inside) | `32 682 5646` |
| Heinrich-Mann-Gymnasium, Heinrich-Mann-Straße 29, Erfurt (secondary) | way **925518704** | E 642668 / N 5647865 | E 221383 / N 5653399 | 50.96470 / 11.03174 | E 642601–642714, N 5647791–5647944 | `32 642 5647` | `32 642 5646` |
| Border probe: Billroda / Tauhardt (Saxony-Anhalt enclaves in Thuringia, Burgenlandkreis) | – | box E 670000–672000, N 5673600–5675600; centre of the largest enclave E 670895 / N 5674609 | E 251711 / N 5677770 | 51.1973 / 11.4460 | | `32 670…671 5673…5675` | |
| ALKIS value samples only: Jena 2 × 2 km (E 682250–684250, N 5646802–5648802), Erfurt centre 1.5 × 1.5 km (E 641313–642813, N 5648322–5649822), Thuringian Forest near Oberhof 2 × 2 km (E 620800–622800, N 5620554–5622554) | | | | | | | |

(Coordinates come from my own Krüger-series UTM code, which round-trips to 1e-7°. The ALKIS school buildings
`DETHL55P0000huPsBL`/`…huPwBL`/`…huPxBL`/`…huPyBL` came back at E 683241–683327 / N 5647714–5647858 in 25832, inside the OSM
footprint. The WFS's own 25833 output matches my 25832 → 25833 conversion within **0.67 mm**, see §8.)

Samples (all under `th/`; treat them as untrusted data):

| Path | Content |
|---|---|
| `thueringen.geojson` (= `probe/boundary/thueringen_simplified25m.geojson`) | **the asset candidate** for `assets/regions/thueringen.geojson` (§10) |
| `dl/wms/` | DOP WMS GetMaps at the school: 25832 (`school_25832.jpg`, preview `school_preview.jpg`), **25833**, 5000 px (1 km²), the 5001 px error, outside-coverage probes (`far.jpg`, `far.png`) |
| `dl/wmstime/` | four 5000 × 5000 px pieces in Erfurt (timing) |
| `dl/docs/README_Hoehendaten_2020-2025.pdf`, `README_Hoehendaten_2014-2019.pdf` | the TLBG height data descriptions (from the Wayback Machine, Nov 2024 captures) |
| `dl/fossgis/slides.pdf` | FOSSGIS 2026 talk on the GDI-Th re-engineering (DOP to MapServer/COG in the state data centre) |
| `probe/alkis/` | ALKIS WFS answers: school bbox (25832, and 25833 output), Jena 2 × 2 km, Erfurt centre, forest, Erfurt school; `paging/` (STARTINDEX tests, hits of a 12 × 12 km box); `err/` (500 "empty String", 0-feature 25833 BBOX, unknown type, JSON format) |
| `probe/border/` | TH and ST ALKIS at the Billroda/Tauhardt enclaves, the TH WMS image there, `overlay_border.jpg` (boundary asset magenta, TH land use yellow, ST buildings red) |
| `probe/boundary/` | `ve_point.gml` (VerwaltungsEinheit at the school: Gemeinde, Kreis, Land), `land_DE16_25833.xml` (the full state polygon), `land_25833.gml` (the boundary WFS's version) |
| `probe/gfi/` | GetFeatureInfo answers of `th_dop_info` (all INFO_FORMATs) |
| `probe/zip/` | member list of the archived LAS tile `las_628_5611_1_th_2014-2019.zip` and its `.meta` (read by ranges from the Wayback Machine) |
| `probe/caps/`, `probe/iso/`, `probe/geomis/`, `probe/ckan/`, `probe/wayback/`, `probe/atom/`, `probe/dla/`, `probe/portal/`, `probe/geofabrik/`, `probe/osm/`, `probe/poll/` | capabilities, ISO records, CSW searches, GovData package lists, archived download pages/app JS/ATOM service feeds/RSS, the live outage pages, the CAPTCHA page, Geofabrik state/poly, OSM ways, the download-host poll log |
| `scripts/` | helper scripts (UTM, GML attribute summary, CSW summary, WMS/WFS caps, zip-by-range, TIFF tags, 25833 comparison, ring info, boundary simplification, border overlap, overlay) |

---

## 1. Brandenburg → Thuringia at a glance

| freemapper product | Brandenburg (today) | Thuringia equivalent | Status |
|---|---|---|---|
| CRS | EPSG:25833 | **EPSG:25832 for every product** (tiles, WMS default, WFS default). Thuringia spans 9.86–12.66°E and straddles the 12° zone border; in 25833 it lies at E 137,846–335,101 (no negative eastings), with −2.6° convergence and +0.3 m/km scale in Jena (west edge −4.0°, +1.2 m/km). WMS and WFS output 25833 on request, but the **WFS BBOX in 25833 silently returns 0 features**. | verified |
| Orthophoto template | DOP20 RGB JPEG + JGW, 1 km zip | **WMS** `https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP`, layer **`th_dop`** (DOP20 RGB, TrueDOP on a bDOM), **max 5000 px**, 12–15 s per 1 km² at 0.2 m, 25832 and 25833. Tiles (1 km RGBI GeoTIFF, `download.php`) are on the download host, which is down. State flown in two halves: centre **spring 2024** (Jena 07.04.2024, Erfurt 01.05.2024), rest **March–June 2025**: mixed leaf-off/leaf-on. | WMS verified; tiles not reachable |
| DTM | DGM1 GeoTIFF f32 + TFW, 1 km | **DGM1** 1 km tiles `dgm1_32_EEE_NNNN_1_th_2020-2025.tif` (Float32, LZW, NoData −9999, + XYZ, + `.meta`), ALS 2020–2025, on the **download host (down)**. **No WCS**; geoproxy has only hillshade WMS (DGM2/DGM5) and an INSPIRE 5 m WMS. | documented only; **blocked** |
| nDSM template | nDOM LAZ (0.2 m grid as points) | **DOM1** (ALS, 1 m, same tile scheme and format as DGM1) − DGM1, on the download host (down). bDOM 0.2 m exists as a product (GeoMIS) but has no open download link. LAZ 1 km tiles (LAS 1.4, PDRF 1, ≈ 120 MB) also on the download host. No nDOM product. | documented only; **blocked** |
| Buildings | OGC API `gebaeude_bauwerk`, GeoJSON | **ALKIS-vereinfacht WFS 2.0** `https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs`, type `ave:GebaeudeBauwerk`, **GML 3.2 only**, elements **prefixed `ave:`**, bare `gml:Polygon`. **`gebnutzbez` in ASCII title case** ("Gebaeude", "Sonstiges Bauwerk Oder Sonstige Einrichtung"); **no `rellage`**: underground garages are `Gebaeude` + `funktion` "Tiefgarage". `funktion` is specific ("Allgemein bildende Schule"). | verified |
| Land use | OGC API `nutzung_flurstueck` | same WFS, **`ave:Nutzung`** (no `NutzungFlurstueck`), `nutzart` **in ASCII title case** ("Strassenverkehr", "Industrie Und Gewerbeflaeche"), `bez` in normal spelling; `Begleitfläche Straßenverkehr` does occur. | verified |
| Currency | `bb_*_aktualitaet.csv` | WMS GetFeatureInfo `th_dop_info` (`Erstellung` = flight date, `Publikation`, tile name) per 1 km tile; per-tile `.meta` files (`Erfassungsdatum`) in the height zips (download host); `aktualit` per ALKIS feature | GFI verified |
| OSM | `brandenburg-latest.osm.pbf` | Geofabrik `thueringen-latest.osm.pbf` (160.3 MB) | verified |
| Licence | dl-de/by-2-0, `© GeoBasis-DE/LGB <year>` | **CC BY 4.0**, attribution **"© GDI-Th"** (service capabilities and every ISO record checked). Changed from dl-de/by-2-0 for data published after **14.09.2026**. | verified |

---

## 2. Infrastructure: hosts, portal, tile scheme

| Path | Host | Products | Notes |
|---|---|---|---|
| **DOP WMS** | `https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP` | DOP 0.2 m and 2 m, RGB/CIR/PAN, metadata layer | deegree 3.4-pre16-TH-1.22; MaxWidth/Height **5000**; fast (§4.2) |
| **ALKIS WFS** | `https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs` | ALKIS vereinfacht (Flurstueck, GebaeudeBauwerk, KatasterBezirk, Nutzung, VerwaltungsEinheit) | WFS 2.0.0, GML only, CountDefault 25000 |
| **Boundary WFS** | `https://www.geoproxy.geoportal-th.de/geoproxy/services/GRENZUEB_wfs` | Land, Landkreise, VG, Gemeinden, Gemarkungen, Fluren (`tlvermgeo:th_etrs89utm32_*`) | WFS 1.1.0, also `application/json` |
| DGM WMS | `https://www.geoproxy.geoportal-th.de/geoproxy/services/DGM` | hillshades `DGM2`, `DGM5` | answers `SERVICE=WCS&REQUEST=GetCapabilities` with its **WMS** capabilities: no WCS |
| INSPIRE elevation | `https://www.geoproxy.geoportal-th.de/geoproxy/services/INSPIREel` | `EL.GridCoverage` (from DGM5) | WMS only; `SERVICE=WCS` → 200 `text/xml` 370 B exception |
| **Download app** (down) | `https://geoportal.geoportal-th.de/gaialight-th/_apps/dladownload/dl-dhm.html` (heights), `dl-lbop.html` (aerial images/DOP) | GAIA-light: `_ajax/overview.php?bbox[]=…&crs=EPSG:25832&type[]=…` (GeoJSON tile index, max 200 objects), `_ajax/details.php`, `download.php?type=op&id=<id>`, `rss.php?obj=op` | same software as SH; read from the Wayback Machine only |
| **Height files** (down) | `https://geoportal.geoportal-th.de/hoehendaten/{DGM,DOM,LAS}/<prod>_<period>/…zip`, `…/hoehendaten/Uebersichten/README_*.pdf`, `Stand_<period>.zip` | DGM1, DOM1, LAZ per 1 km | one archived URL: `…/hoehendaten/LAS/las_2014-2019/las_628_5611_1_th_2014-2019.zip` (119,052,248 B) |
| **ATOM feeds** (down) | `https://geoportal.geoportal-th.de/dienste/atom_th_hoehendaten_{dgm,dom,las}` (service feeds; dataset feeds `?type=dataset&id=<uuid>`), `atom_th_alkis`, `atom_th_gebaeude`, `atom_th_dtk10`, … | INSPIRE ATOM | service feeds archived (last 2026-04-13 / 2026-08-30); dataset feeds never archived |
| Portal (CAPTCHA) | `https://geoportal.thueringen.de/gdi-th/download-offene-geodaten/download-{hoehendaten,luftbilder-und-orthophotos,liegschaftskataster,…}` | pages that embed the download app in an iframe | **Link11 CAPTCHA, HTTP 200**; `www.geoportal-th.de` 301 → there |
| Catalogue | `https://geomis.geoportal-th.de/geonetwork/srv/ger/csw` | ISO 19139 per dataset | DOP `3940ce6d-a2fc-44cf-9b30-916f999750ae`, DGM `14418d25-fcd7-4a3f-99a9-e3059a2772af`, DOM `3b5d8d9c-775d-4617-8dfe-71480d6472a6`, LiDAR `c8363eb8-7f2a-49b5-bb59-a1571f40a21f`, bDOM `1c2a4b6b-7938-4f42-8e24-2d48ce4fe594` |
| GovData | `https://www.govdata.de/ckan/api/3/action/package_search?q=TLBG` | 98 packages | `license_id` empty; links still point to the old ATOM/portal URLs |

Dead or misleading endpoints:
- Everything on `geoportal.geoportal-th.de`, including the URLs in GovData and the ISO records (ATOM feeds, `hausko_umr/HK-TH.zip`,
  `dlm/*.zip`): 200/206 `text/html`, 970 B, `Last-Modified: Thu, 24 Nov 2016 16:08:43 GMT` (the static outage page).
- `geoportal.thueringen.de/media/Download/…` (catalogues, legends): behind the same CAPTCHA.
- easygeodata.de (a third-party mirror that links Thuringian DOP/LiDAR tiles "direkt bei der Landesbehörde") needs an API
  key (401); not used.
- `vmwgis3sdi1.tlvermgeo.thlv.de` (historic DOP WMS in GovData) is an internal host name.
- The re-engineering of GDI-Th (FOSSGIS 2026 talk by grit GmbH for TMDI/TLBG): the DOP are being moved to MapServer WMS
  **and WCS** on COGs (ZSTD, 512 px blocks) in the Thüringer Landesrechenzentrum ("Geoproxy 4.0"). No public URL was
  found; the download-host outage may be part of this migration (not confirmed).

### 2.1 Tile scheme

- **1 km × 1 km**, EPSG:25832, SW corner at whole km, the same ids for DOP, DGM1, DOM1 and LAZ.
  - DOP tile name in the WMS metadata: `32_683_5647` (the school tile). The RSS of updates lists `202506 32721_5638`
    (flight month + tile). Source files (FOSSGIS slides): ≈ 17,000 GeoTIFFs, 5000 × 5000 px, RGBI, 25832.
  - DGM1/DOM1 (README 2020–2025, §5): `dgm1_32_636_5621_1_th_2020-2025.tif` / `.xyz` / `.meta`,
    `dom1_32_636_5621_1_th_2020-2025.*`, `las_32_636_5621_1_th_2020-2025.laz`. The period suffix is part of the name
    (`2010-2013`, `2014-2019`, `2020-2025`), so the same tile exists in up to three vintages.
  - The archived 2014–2019 LAZ zip is named **without** the zone: `las_628_5611_1_th_2014-2019.zip`. Whether the
    2020–2025 zips have `_32_` is unknown.
- WMS root extent (25832): E 557000–769000, N 5561000–5727000; the 0.2 m RGB layer E 561000–757000, N 5562000–5724000.
- The WMS covers whole 1 km tiles beyond the border: at the Saxony-Anhalt enclaves the TH WMS shows imagery over Saxony-Anhalt
  (§10.3); 2 km further out it is white.

---

## 3. CRS, height datum, extent (read this first)

- **Every product is EPSG:25832.**
  - DOP WMS layers list 25832 first; the root lists 25832, 25833, 31467, 31468, 4326, 4258, 3034, 3035, 3038–3047, 3857.
  - ALKIS WFS `DefaultCRS urn:ogc:def:crs:EPSG::25832`, OtherCRS 4326, 25833, 31467, 31468, 4258, 3038–3040 (3035 also works).
  - Height data (README): "ETRS89, Mittelmeridian der UTM-Zone 32 … ohne Zonenkennziffer" (the eastings in the files
    have no `32` prefix); LAS `.meta` `EPSG-Code Lage: 25832`.
- **Height datum DHHN2016** normal heights (README "Deutsches Haupthöhennetz 2016 (DHHN16); Höhen in NHN"; `.meta`
  `EPSG-Code Hoehe: 7837`, `Quasigeoid: GCG2016`). The same as the other freemapper states.
- **Zone 33 would be acceptable for Thuringia** (unlike Hesse), but the data are 25832:

  | Point | zone 32 | zone 33 |
  |---|---|---|
  | Jena school (11.61°E) | E 683250, convergence +2.03°, scale +0.01 m/km | E 261846, −2.63°, +0.30 m/km |
  | Erfurt (11.03°E) | E 642668, +1.58°, −0.15 m/km | E 221383, −3.08°, +0.55 m/km |
  | west edge (9.87°E) | E 561113, +0.68°, −0.35 m/km | E 139743, −3.98°, +1.19 m/km |
  | east edge, Altenburg (12.66°E) | E 756840, +2.84°, +0.41 m/km | E 335777, −1.82°, −0.07 m/km |

  Recommendation: **25832 projects** (`Region::crs() → Crs::Utm32`), like Saxony-Anhalt, Hamburg, Hesse, SH: native tile
  grid, WFS BBOX works only in 25832 (§8.1).
- **Extent** of the state polygon (ALKIS `VerwaltungsEinheit` DE16, 63,327 vertices in the main ring):

  | CRS | Extent |
  |---|---|
  | EPSG:25833 | E 137846 – 335101, N 5570126 – 5730537 |
  | EPSG:25833 (boundary WFS envelope) | E 134006 – 341328, N 5559341 – 5735465 |
  | EPSG:25832 (WMS root bbox) | E 557000 – 769000, N 5561000 – 5727000 |
  | lon / lat (Geofabrik poly) | lon 9.863 – 12.661, lat 50.200 – 51.652 |

- 25833 availability: DOP WMS GetMap in 25833 works (school, 483,580 B, 1.5 s); ALKIS WFS **output** in 25833 works
  (0.67 mm from my transform), **BBOX in 25833 does not** (0 features, §8.1); boundary WFS outputs 25833.

---

## 4. Orthophoto

### 4.1 Vintages and which one is current

- TLBG flies half of the state each year; the WMS serves the newest TrueDOP per tile (`Bezug=bDOM` everywhere sampled). Flight
  dates from GetFeatureInfo `th_dop_info` (`Erstellung`), publication month (`Publikation`):

  | Place | Tile | Flight | Published |
  |---|---|---|---|
  | **Jena school** | `32_683_5647` | **07.04.2024** | 2024-10 |
  | Erfurt (HMG) | `32_642_5647` | 01.05.2024 | 2024-10 |
  | Weimar | `32_663_5649` | 02.05.2024 | 2024-11 |
  | Suhl | `32_619_5607` | 15.05.2024 | 2025-02 |
  | Billroda border probe | `32_671_5674`, `32_671_5675` | 01.05.2024 | 2025-01 |
  | Eisenach | `32_592_5647` | 05.03.2025 | 2025-09 |
  | Mühlhausen | `32_601_5674` | 08.03.2025 | 2025-07 |
  | Nordhausen | `32_624_5707` | 01.05.2025 | 2025-09 |
  | Sonneberg | `32_654_5580` | 12.05.2025 | 2026-01 |
  | Saalfeld | `32_667_5613` | 21.06.2025 | 2026-02 |
  | Gera, Altenburg | `32_716_5641`, `32_740_5653` | 22.06.2025 | 2026-03 |

  So the centre (Erfurt–Weimar–Jena, Suhl) is **2024**, the west, north, east and south are **2025**. Flights run from early
  March (leaf-off) to late June (full leaf). The Jena school image (`dl/wms/school_preview.jpg`) shows early leaf-out with
  blossoming trees; the Billroda image (May 2024) full foliage. The download app said in 2026 "DOP von 2025 erst
  teilweise veröffentlicht"; by now the 2025 half is published (latest `Publikation` 2026-03).
- Historic DOP/aerial images go back to 1945/1953 (archived `dl-lbop.html`, overview PDFs per year); not needed.

### 4.2 WMS (recommended path)

- `https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP`, WMS 1.1.1/1.3.0, title "WMS TH DOP".
  - Layers: **`th_dop`** ("DOP Farbe", RGB 0.2 m; MinScaleDenominator 44), `th_dop20cir`, `th_dop20pan`, the 2 m layers
    `th_dop200rgb`/`th_dop200cir`/`th_dop200pan` (MinScale 24999), and **`th_dop_info`** (queryable metadata).
  - Formats: jpeg, png, png 8 bit, tiff, gif. No TIME dimension.
  - **MaxWidth/MaxHeight 5000**. 5001 px → **HTTP 200 `text/xml`**:
    `<ServiceException>Map width out of range.</ServiceException>`. freemapper's `max_px` can be 5000.
- Example (school, 25832, 0.2 m):
  ```
  https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=th_dop&STYLES=&CRS=EPSG:25832&BBOX=683050,5647652,683450,5647952&WIDTH=2000&HEIGHT=1500&FORMAT=image/jpeg
  ```
  → 200 `image/jpeg`, 475,299 B, **1.4 s**. 25833 (`BBOX=261646,5649886,262046,5650186`): 483,580 B, 1.5 s.
- **Fast:** 5000 × 5000 px (1 km² at 0.2 m): 12.4 s at the school; four Erfurt km²: 14.8, 12.4, 11.9, 12.0 s (4.5–4.6 MB each).
  No hangs or errors in ≈ 12 GetMaps. A 2 × 2 km map at 0.2 m = 4 pieces ≈ 50 s.
- JPEG: IJG **quality 75**, baseline, RGB. The source tiles are RGBI GeoTIFFs; the WMS returns RGB only.
- Outside the coverage (Leipzig, E 730000 N 5690000): JPEG **white** (255), PNG with `TRANSPARENT=TRUE` fully transparent;
  0.13 s.
- Headers: `Cache-Control: no-cache, no-store`, chunked, no ETag, no Last-Modified, `Access-Control-Allow-Origin: *`,
  `Set-Cookie: OAMAuthnCookie_…` (Oracle Access Manager; not needed for the next request).
- Not checked: whether the WMS pixels equal the tile pixels on the native grid (no tile could be downloaded).
- **GetFeatureInfo gives the flight date per tile:**
  ```
  https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=th_dop_info&QUERY_LAYERS=th_dop_info&STYLES=&CRS=EPSG:25832&BBOX=683200,5647752,683300,5647852&WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain
  → <xml version="1.0" encoding="UTF-8">
    _featuremember_$#$Name=32_683_5647$#$Titel=n/a$#$Erstellung=07.04.2024$#$Publikation=2024-10$#$CSW-Anfrage=https://geomis…&ID=3940ce6d-…$#$Bezug=bDOM
  ```
  0.13 s. `application/vnd.ogc.gml` / `text/xml` give a WFS 1.0 GML (`tlvermgeo:DOP20RGBI_UTM` with `Name`, `Titel`,
  `Erstellung` DD.MM.YYYY, `Publikation` YYYY-MM, `CSW-Anfrage`, `Bezug`, and the tile box in `gml:Box`) — easier to parse
  than the odd `$#$` text format. `application/json` → 200 `text/xml` "InvalidFormat". Outside the coverage: no feature.

### 4.3 Tile downloads (download host; not reachable)

- From the archived app and RSS: DOP tiles were served per tile through
  `https://geoportal.geoportal-th.de/gaialight-th/_apps/dladownload/download.php?type=op&id=<numeric id>` (ids from
  `_ajax/overview.php`, which refuses > 200 objects: `{"success":false,"message":"Too many objects returned - max 200",…}`);
  an invalid id answered `No object found for type op id 100000`. An archived `type=lb` (aerial image) answer was 175 MB, so
  the DOP tile (5000 × 5000 RGBI, presumably uncompressed like SH) is likely ≈ 100 MB.
- RSS `rss.php?obj=op` lists updated tiles as `202506 32721_5638` (flight YYYYMM + tile).
- Format, compression, HTTP ranges: **not verified**. The WMS is the only working orthophoto path now, and it is good enough
  (fast, 5000 px).

---

## 5. DGM1 (terrain model, 1 m)

- Product "Digitales Geländemodell", ISO `14418d25-…`, licence CC BY 4.0 "© GDI-Th". Vintages per tile: 2010–2013, 2014–2019,
  **2020–2025** (the app's radio buttons `dhm2`, `dhm1`, `dhm1n`).
- **From `README_Hoehendaten_2020-2025.pdf`** (TLBG, Stand 10.11.2023; Wayback copy in `dl/docs/`):
  - "Format tif = GEOTIFF, 32 bit, Float, Komprimierung LZW, Hintergrund-/NoData-Wert -9999"; also XYZ ASCII (cell centres,
    2 decimals, e.g. `578227.50 5625401.50 353.90`).
  - AdV PQS-DGM 3.2 / PQS-DOM 1.1: raster 1 m, 1 × 1 km, "nicht redundant"; the PQS places the cell centres at .5 m, like
    LGB/GeoSN/HVBG (`.xyz` example coordinates end in `.50`).
  - Names `dgm1_32_636_5621_1_th_2020-2025.tif|.xyz|.meta`; one `.meta` per tile with `Datei, Erfassungsdatum,
    Erfassungsmethode, Lasergebiet, EPSG-Code Lage, EPSG-Code Hoehe, Quasigeoid, Genauigkeit Hoehe, Qualitaetsstufe,
    ADV-Standard, Copyright`.
- **Download (not reachable):** `https://geoportal.geoportal-th.de/hoehendaten/…`, zip per tile (by analogy with the
  archived LAS URL: `hoehendaten/DGM/dgm_2020-2025/dgm1_…_1_th_2020-2025.zip`; the exact path was not seen), and the ATOM
  feed `…/dienste/atom_th_hoehendaten_dgm?type=dataset&id=14418d25-fcd7-4a3f-99a9-e3059a2772af`. Overviews of the vintage per
  tile: `…/hoehendaten/Uebersichten/Stand_2020-2025.zip` etc.
- **No WCS**, no other open 1 m source. `DGM` WMS = hillshades of DGM2/DGM5; `INSPIREel` = 5 m, WMS only.
- Not verified (host down): TIFF tags (tiepoint, GeoKeys, tiling, predictor), zip layout and compression, Range support,
  ETag/Last-Modified, real file sizes, whether the 2020–2025 vintage is statewide.

---

## 6. nDSM sources: DOM1, bDOM, LiDAR

### 6.1 DOM1 (1 m, from ALS)

- ISO `3b5d8d9c-…`; per README the same format and naming as DGM1 (`dom1_32_…_1_th_2020-2025.tif`, Float32 LZW, −9999),
  from the same ALS flight as the DGM1 of the same period, so DOM1 − DGM1 of a tile share one date.
- Download only via the download host / ATOM `atom_th_hoehendaten_dom` (down). Not verified.

### 6.2 bDOM 0.2 m (image-based)

- GeoMIS record `1c2a4b6b-7938-4f42-8e24-2d48ce4fe594` "Bildbasiertes Digitales Oberflächenmodell - bDOM", keyword
  `opendata`, licence CC BY 4.0, "Punktgitterweite … 20 cm", format TIFF. **No download or service link** in the record
  (only TLBG pages and the fee regulation ThürVwKostOVerm); the archived download app has no bDOM category. It is the
  surface the TrueDOP is rectified on (`Bezug=bDOM`). Treat as not openly available.

### 6.3 LiDAR / nDOM

- LAZ per 1 km tile on the download host (ATOM `atom_th_hoehendaten_las`). The archived
  `las_628_5611_1_th_2014-2019.zip` (Wayback, ranges worked through `web.archive.org`): **2 members, deflated**:
  `las_628_5611_1_th_2014-2019.laz` 119,032,929 B and `.meta` 292 B; `x-archive-orig-accept-ranges: bytes`,
  `Last-Modified: Thu, 01 Aug 2019`. The `.meta`:
  ```
  Datei: las_628_5611_1_th_2014-2019.laz
  Erfassungsdatum: 2018-04
  Erfassungsmethode: Airborne Laserscanning
  Lasergebiet: Laser_225_2018
  EPSG-Code Lage: 25832
  EPSG-Code Hoehe: 7837
  Quasigeoid: GCG2016
  Genauigkeit Lage: 0,150m
  Genauigkeit Hoehe: 0,090m
  (c) GDI-Th, Freistaat Thueringen
  ```
  The date has **month precision** (`2018-04`).
- README classes: 1 other, 2 ground, 18 added ground points, 20 non-ground ("relevant für DOM"); LAS 1.4, PDRF 1.
- **No nDOM** product (CSW search "nDOM"/"Normalisiert": 0 records).

---

## 7. Data currency per tile / feature

| Mechanism | Products | How |
|---|---|---|
| **WMS GetFeatureInfo `th_dop_info`** | DOP (WMS path) | `Erstellung` = flight date (DD.MM.YYYY), `Publikation` (YYYY-MM), `Name` = tile id, per 1 km tile, 0.13 s per query; GML format recommended |
| `.meta` per tile (download host) | DGM1, DOM1, LAZ | `Erfassungsdatum` (month precision, e.g. `2018-04`), `Lasergebiet` |
| `Stand_<period>.zip` (download host) | heights | overview of the vintage per tile (not read) |
| RSS `rss.php?obj=op` (download host) | DOP | updated tiles `YYYYMM EEE_NNNN` |
| ALKIS `aktualit` | per feature | `YYYY-MM-DD` (**no `Z`**, unlike GeoSN/HVBG). Buildings: 5,185 of 8,416 from 2020 (a migration), then 2021–2026; land use spread 2013–2026 |
| Geoproxy WMS headers | – | no ETag / Last-Modified |

Suggested mapping: orthophoto `currency` = `Erstellung` of the tile(s) under the map, `published` = `Publikation`;
DGM1/DOM1 `currency` = `.meta` `Erfassungsdatum` (once downloadable).

---

## 8. ALKIS: buildings and actual land use

### 8.1 Service

- `https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs`, title "WFS TH ALKIS Vereinfacht", WFS 2.0.0 (deegree).
  - Types: `ave:Flurstueck`, **`ave:GebaeudeBauwerk`**, `ave:KatasterBezirk`, **`ave:Nutzung`**, `ave:VerwaltungsEinheit`.
    **No `ave:NutzungFlurstueck`** (asking for it → HTTP 200 `text/xml` "Missing WFS:* right for object(s)
    {}:NutzungFlurstueck.").
  - Namespace `http://repository.gdi-de.org/schemas/adv/produkt/alkis-vereinfacht/1.0` (**1.0**; ST uses 2.0), elements
    **with the `ave:` prefix** (`<ave:GebaeudeBauwerk gml:id="GebaeudeBauwerk_DETHL55P0000huUTBL"><ave:oid>…`). freemapper's
    `parse_gml` strips prefixes (`local()`), so this works.
  - Geometry: **bare `gml:Polygon`** (no MultiSurface, although the schema says `MultiSurfacePropertyType`), `posList`, 2D,
    no `srsDimension`; 2 polygons with holes in 3,826 buildings; no arcs seen. Supported by `parse_gml`.
  - Output: GML 2.1.2–3.2.1 only. `OUTPUTFORMAT=application/json` / `application/geo+json` → **400** "This WFS is not
    configured to handle the output/input format 'application/json'".
  - Constraints: `ImplementsResultPaging FALSE`, **`CountDefault 25000`**.
- Freemapper's request shape works:
  ```
  https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=683050,5647652,683450,5647952,urn:ogc:def:crs:EPSG::25832&SORTBY=oid&COUNT=10000&STARTINDEX=0
  ```
  → 200 `application/gml+xml; version=3.2`, 125 features, 155 KB, 0.5 s. `RESULTTYPE=hits` → `numberMatched="125"` (0.13 s).
- Speed (no gzip: `Accept-Encoding: gzip` is ignored): Jena 2 × 2 km 3,826 buildings 4.7 MB **4.5 s**, 770 land use 1.5 MB
  1.7 s; Erfurt centre 1.5 × 1.5 km 4,581 buildings 6.1 MB 6.0 s; 12 × 12 km Erfurt (77,718 buildings by hits) 25,000
  features 31 MB in 33 s.
- **Paging works, `SORTBY` is ignored:** `COUNT=1000&STARTINDEX=0/1000/2000/3000` gave 1000/1000/1000/826 disjoint features
  whose union equals the full answer (3,826), but the order is neither ASCII nor case-insensitive by `oid`, and identical
  with and without `SORTBY=oid`. Two identical requests returned the same order. A paged answer's `numberMatched` is the
  **page size** (`numberMatched="1000" numberReturned="1000"`), so the total must come from `RESULTTYPE=hits` (which is what
  `fetch_wfs` does).
- **Silent cap:** without COUNT or with `COUNT=30000`, at most **25,000** features come back, with `numberMatched="25000"`.
  freemapper's `PAGE_COUNT` 10,000 stays below it.
- **25833:** `SRSNAME=urn:ogc:def:crs:EPSG::25833` with a **25832 BBOX** → 125 features in 25833, E N order, within 0.67 mm of
  my transform. But a **BBOX in 25833** (`…,urn:ogc:def:crs:EPSG::25833`, `EPSG:25833`, or swapped axes) → **200, 0
  features**. So the WfsService must use `Crs::Utm32`.
- Errors: a malformed BBOX → **500** `ows:ExceptionReport` "empty String"; unknown type → **200** `text/xml` ExceptionReport
  (from the access proxy); unknown output format → 400. `parse_gml` refuses any root other than `FeatureCollection`, so all
  of these become errors.
- Outside Thuringia (E 700000 N 5700000): `numberReturned="0"`. Features crossing the bbox are returned whole.

### 8.2 Attributes compared with LGB

- Schema (`DescribeFeatureType`):
  - `GebaeudeBauwerk`: `oid, aktualit, geometrie, gebnutzbez, funktion, fktkurz, name, anzahlgs, lagebeztxt`. **No `rellage`,
    no `gmdschl`, no `gfkzshh`.** `fktkurz` never occurred.
  - `Nutzung`: `oid, aktualit, geometrie, nutzart, bez, name`. Not split by parcel (like Saxony-Anhalt).
- **Value spellings:**
  - `gebnutzbez` and `nutzart` are **ASCII transliterations in title case** of the AdV names: `Gebaeude`,
    `Sonstiges Bauwerk Oder Sonstige Einrichtung`, `Bauwerk Oder Anlage Fuer Industrie Und Gewerbe`,
    `Vorratsbehaelter Speicherbauwerk`, `Turm`; `Wohnbauflaeche`, `Strassenverkehr`, `Industrie Und Gewerbeflaeche`,
    `Flaeche Gemischter Nutzung`, `Sport Freizeit Und Erholungsflaeche`, `Flaeche Besonderer Funktionaler Praegung`,
    `Unland Vegetationslose Flaeche`, `Fliessgewaesser`, `Gehoelz`, `Stehendes Gewaesser`.
  - `funktion`, `bez` and `name` use the **normal spelling** (`Wohngebäude`, `Begleitfläche Straßenverkehr`,
    `Öffentliche Zwecke`).
  - Without normalisation **every building is dropped** (`buildings.rs` `BUILDING_KIND = "Gebäude"`) and no land use is
    private (`PRIVATE_LAND_USES` has `Wohnbaufläche` …). Hamburg's `lgv::normalise_building` / `normalise_land_use` fix both:
    `spelling_key` lower-cases, maps ä/ö/ü/ß to ae/oe/ue/ss and `-`, `,`, `/` to spaces, so `Industrie Und Gewerbeflaeche` →
    `Industrie- und Gewerbefläche`, `Sport Freizeit Und Erholungsflaeche` → `Sport-, Freizeit- und Erholungsfläche`,
    `Unland Vegetationslose Flaeche` → `Unland/Vegetationslose Fläche` (checked by hand for all 16 observed values).
- **No building parts:** no `Bauteil` at all in 8,416 buildings. **Underground garages are `gebnutzbez` "Gebaeude" with
  `funktion` "Tiefgarage"** (28 in the samples, one per Erfurt school box) and no `rellage`, so freemapper would draw them as
  surface buildings. The adapter must set `underground` from `funktion == "Tiefgarage"`.
- oids: 18 characters, **`DETHL5xP…`** (`DETHL51P` Erfurt, `DETHL55P` Jena, `DETHL53P`, `DETHL57P`); `…BL` buildings, `…TN`
  land use. `Region::of_object_id` gets a `"DETH" => Thuringia` arm.

### 8.3 Sample features at the Carl-Zeiss-Gymnasium (geometry elided)

```json
{"oid":"DETHL55P0000huPsBL","aktualit":"2020-11-03","gebnutzbez":"Gebaeude","funktion":"Allgemein bildende Schule",
 "name":"Carl-Zeiss-Gymnasium","anzahlgs":"3","lagebeztxt":"Erich-Kuithan-Straße 7"}
{"oid":"DETHL55P0000huQIBL","aktualit":"2020-11-03","gebnutzbez":"Gebaeude","funktion":"Sport-, Turnhalle","anzahlgs":"2",
 "lagebeztxt":"Erich-Kuithan-Straße 7"}
{"oid":"DETHL55P0000hwm5BL","aktualit":"2020-08-13","gebnutzbez":"Sonstiges Bauwerk Oder Sonstige Einrichtung",
 "funktion":"Überdachung","lagebeztxt":"Erich-Kuithan-Straße 7"}
{"oid":"DETHL55P0000huHxBL","aktualit":"2020-11-03","gebnutzbez":"Gebaeude","funktion":"Tiefgarage",
 "lagebeztxt":"Naumburger Straße 4; Merseburger Straße 19a; Altenburger Straße 15,19; …"}
{"oid":"DETHL55P0000Sn0HTN","aktualit":"2021-12-20","nutzart":"Flaeche Besonderer Funktionaler Praegung","bez":"Öffentliche Zwecke"}
{"oid":"DETHL55P0000Sn0KTN","aktualit":"2021-12-20","nutzart":"Sport Freizeit Und Erholungsflaeche","bez":"Sportanlage"}
{"oid":"DETHL55P0000VUV2TN","aktualit":"2022-03-14","nutzart":"Strassenverkehr","bez":"Begleitfläche Straßenverkehr"}
{"oid":"DETHL55P0001kB2wTN","aktualit":"2026-07-29","nutzart":"Platz"}
```

The school land (15,713 m²) is FBFP "Öffentliche Zwecke" (Brandenburg: "Bildung und Wissenschaft"); the sports ground next to
it SFE "Sportanlage". The Erfurt school buildings: `funktion` "Allgemein bildende Schule", `name` "Heinrich-Mann-Gymnasium
Staatliches Gymnasium Zur Himmelspforte,Staatliches Gymnasium 5".

### 8.4 Observed values (Jena 2 × 2 km + Erfurt centre + forest: 8,416 buildings, 1,813 land use objects)

- **`gebnutzbez`:** `Gebaeude` 7,806, `Sonstiges Bauwerk Oder Sonstige Einrichtung` 603 (all `funktion` Überdachung),
  `Bauwerk Oder Anlage Fuer Industrie Und Gewerbe` 3 (Schornstein, Funkmast), `Turm` 3 (Stadt-/Torturm, Kirchturm),
  `Vorratsbehaelter Speicherbauwerk` 1 (Tank). No `Bauteil`.
- **`funktion` is specific** (50 values): Wohngebäude 4,227, Gemischt genutztes Gebäude mit Wohnen 939, Gebäude zur
  Freizeitgestaltung 757, Nach Quellenlage nicht zu spezifizieren 634, Überdachung 603, Gebäude für Wirtschaft oder
  Gewerbe 229, Garage 219, Gebäude für Handel und Dienstleistungen 168, Gebäude für öffentliche Zwecke 104, … **Allgemein
  bildende Schule 31**, **Tiefgarage 28**, Kirche 36, Museum 24, Sport-, Turnhalle 12, Hochschulgebäude 8 …
- **`name`** on 260 buildings, clean text (no `;`-split fragments as in Hesse); multi-part names joined by `,`
  ("Königin-Luise-Gymnasium,Staatliches Gymnasium 6").
- **`anzahlgs`** on 7,375 buildings. `lagebeztxt` on every building.
- **`nutzart`** (TH spelling → count): Strassenverkehr 475, Flaeche Gemischter Nutzung 236, Wohnbauflaeche 229, Weg 189,
  Sport Freizeit Und Erholungsflaeche 149, Flaeche Besonderer Funktionaler Praegung 125, Industrie Und Gewerbeflaeche 109,
  Wald 91, Fliessgewaesser 63, Landwirtschaft 41, Platz 39, Bahnverkehr 27, Unland Vegetationslose Flaeche 22, Gehoelz 14,
  Friedhof 2, Stehendes Gewaesser 2.
- **`bez` per `nutzart`:**
  - `Strassenverkehr`: none 455, **`Begleitfläche Straßenverkehr` 20** (so `paved.rs`'s `VERGE` rule fires, unlike Saxony and
    Hesse). Never `Fahrbahn`.
  - `Platz`: `Parkplatz` 20, **none 19** → needs `square_bez` (e.g. "ohne Unterart (Thüringen)").
  - `Weg`: none 189 (no Fußweg/Rad- und Fußweg).
  - FBFP: Öffentliche Zwecke 111, Kultur 12, Historische Anlage 2.
  - IuG: Handel und Dienstleistung 65, Industrie und Gewerbe 28, Umspannstation 5, Lagerplatz 4, none 2, Abfallbehandlungsanlage,
    Entsorgung, Heizwerk, Wasserwerk, Gaswerk 1 each.
  - SFE: Garten 46, Grünanlage 34, Kleingarten 27, Siedlungsgrünfläche 20, Park 13, Wochenend- und Ferienhausfläche 6,
    Sportanlage 2, Freizeitanlage 1.
  - Wald: Nadelholz 66, Laubholz 18, Laub- und Nadelholz 7. Landwirtschaft: Grünland 34, Ackerland 7. Unland: Naturnahe
    Fläche 13, Gewässerbegleitfläche 9.
  - `Wohnbauflaeche`, `Flaeche Gemischter Nutzung`, `Bahnverkehr`, `Fliessgewaesser`, `Gehoelz`, `Friedhof`: no `bez`.

### 8.5 What is missing or different

- No GeoJSON, no OGC API Features; GML 3.2 with `ave:` prefixes.
- No `NutzungFlurstueck` (use `ave:Nutzung`, like LVermGeo ST).
- ASCII title-case `gebnutzbez`/`nutzart` (normalise as for Hamburg).
- No `rellage`, no `Bauteil`: underground garages only recognisable by `funktion` "Tiefgarage".
- `SORTBY` ignored; BBOX only in 25832; cap 25,000; no gzip.
- INSPIRE BU/LU, LoD1/LoD2, Hausumringe were not examined.

---

## 9. Licence and attribution

- Licence: **Creative Commons BY 4.0** (<https://creativecommons.org/licenses/by/4.0/deed.de>). Verbatim, in the `Fees` of
  the DOP WMS, DGM WMS, INSPIRE WMS, ALKIS WFS and boundary WFS capabilities and in `otherConstraints` of the ISO records
  of DOP, DGM, DOM, ALS and bDOM:
  > Lizenz: Creative Commons BY 4.0 (https://creativecommons.org/licenses/by/4.0/deed.de) Namensnennung: "© GDI-Th"

  plus the machine-readable `{"id":"cc-by/4.0","name":"Creative Commons BY 4.0","url":"https://creativecommons.org/licenses/by/4.0/deed.de","quelle": "© GDI-Th"}`.
  `AccessConstraints`: "NONE. Es gelten keine Beschränkungen."
- **Change of licence:** until September 2026 the open data were under dl-de/by-2-0 (archived RSS of April 2026: "… gelten
  die Nutzungsbedingungen nach der Datenlizenz Deutschland - Namensnennung, zurzeit in der Version 2.0 unter Verwendung des
  Quellenvermerks gemäß Absatz 2 Nummer 1: © GDI-Th"). A geoportal notice (news id 92, quoted in the OSM forum thread
  "Fragen zum Lizenzwechsel DOP in Thüringen", 17.09.2026) replaces it with CC BY 4.0 for data published after **14.09.2026**.
  The notice itself is behind the CAPTCHA and was not read. Data downloaded earlier stay under dl-de/by-2-0; both
  licences require the source note "© GDI-Th".
- CC BY 4.0 requires: the attribution "© GDI-Th", the licence name and link, and an **indication of changes** ("verändert").
  It also forbids adding technical restrictions (DRM) to the shared material — irrelevant for OMAP/OCAD files.
- **Recommended freemapper line:** `Geobasisdaten: © GDI-Th, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/), verändert`
  + `© OpenStreetMap contributors, ODbL` + the freemapper credit line.
- The tile `.meta` files carry "(c) GDI-Th, Freistaat Thueringen".

---

## 10. State boundary (for `assets/regions/thueringen.geojson`)

### 10.1 Source

- **Recommended:** ALKIS WFS `ave:VerwaltungsEinheit`, `art` "Bundesland", oid **`DE16`**, `name` Thüringen, `schluessel` 16,
  `aktualit` 2024-01-22. Fetched by a 10 m bbox at the school (in 25832; output in 25833), which returns Gemeinde (DE16053000
  Jena), Kreis (DE16053) and Land:
  ```
  https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ave:VerwaltungsEinheit&SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX=683245,5647797,683255,5647807,urn:ogc:def:crs:EPSG::25832
  ```
  → 200, 2.3 MB, **3.0 s**.
- Alternative: boundary WFS `tlvermgeo:th_etrs89utm32_land` (`…/GRENZUEB_wfs?SERVICE=WFS&VERSION=1.1.0&REQUEST=GetFeature&TYPENAME=tlvermgeo:th_etrs89utm32_land&SRSNAME=EPSG:25833`,
  1.09 MB, 1.4 s; GML 3.1.1 `MultiPolygon`, 5 polygons, 7 holes; JSON possible). Not used.

### 10.2 Geometry and the asset

- DE16: MultiSurface of **5 polygons**, main ring 63,327 vertices, 16,221.28 km² (25833 area):
  - 10 holes: 3 degenerate (0 m²), the rest 492 m² … 0.094 km². The real ones are **Saxony-Anhalt enclaves**: four at Billroda/
    Tauhardt (51.197°N 11.446°E; 0.094 km², 0.019 km², 8,922 m², 821 m²) and one at 51.334°N 11.416°E (0.094 km²); two small
    ones near 51.54°N 10.91°E (492 and 1,752 m²; not covered by any freemapper asset, presumably Lower Saxony or simplification noise).
  - 4 exclaves: 37,149 m² (51.614°N 10.920°E) and 10,506 m² (51.579°N 10.884°E) in the Harz, 3,760 m² (51.210°N 11.439°E),
    4,481 m² (50.631°N 12.218°E, Saxony border).
- **Simplification:** plain Douglas–Peucker per ring (the script used for Hesse/SH, `scripts/simplify_th.py`) produced an
  **invalid** MultiPolygon here (self-intersection at E 252671 N 5686873, two overlapping holes, the 50.63°N exclave touching
  the main polygon). The asset was therefore made with **topology-preserving** simplification (shapely `simplify(25,
  preserve_topology=True)`, then `set_precision(1 m)`), `scripts/simplify_th_topo.py`:
  - `thueringen.geojson`: FeatureCollection, `crs` `urn:ogc:def:crs:EPSG::25833`, properties `name: Thüringen`, `source:
    TLBG ALKIS vereinfacht WFS adv_alkis_wfs (VerwaltungsEinheit DE16), © GDI-Th, CC BY 4.0`, `retrieved: 2026-11-08`,
    `simplified_m: 25`; MultiPolygon with integer coordinates; **valid** (GEOS).
  - **5,955 vertices, 101.7 KB** (Hesse 82.7 KB, Saxony 94.5 KB). Main ring 5,881 vertices + 7 holes (the 3 degenerate holes
    dropped); exclaves 8/5/4/5 vertices.
  - Area 16,221.25 km² (−0.03 km² vs the original), symmetric difference 7.1 km² over ≈ 1,000 km of border.
  - Extent E 137,848 … 335,100, N 5,570,126 … 5,730,537 (no negative eastings).
- **Side finding:** the existing assets `brandenburg`, `berlin`, `saxony`, `saxony-anhalt` and `hessen` are **invalid**
  GEOS geometries (self-intersections, e.g. Saxony at E 378077 N 5695102, Saxony-Anhalt at E 304398 N 5694982, Hesse at
  E 43434 N 5514915); only `hamburg` is valid. `geo`'s `Contains`/`Intersects` still give answers, but a ring that crosses
  itself can misclassify points near the crossing.

### 10.3 Shared borders (simplified assets, after `make_valid` on the neighbours)

| Neighbour asset | Shared border | Overlap | Gap (excluding tripoint artefacts) | Shared-vertex distance to the neighbour's boundary |
|---|---|---|---|---|
| `saxony.geojson` | 274.9 km | 198 pieces, **0.24 km²**, largest 10,634 m², mean width ≤ 19 m | ≈ 0.17 km², pieces ≤ 6,000 m² | median 0.0 m, p95 12.0 m |
| `saxony-anhalt.geojson` | 305.6 km | 57 pieces, 0.062 km², largest 10,122 m², ≤ 14 m wide | ≈ 0.076 km², pieces ≤ 7,754 m² | median 0.0 m, p95 2.0 m |
| `hessen.geojson` | 265.9 km | 25 pieces, 0.032 km², largest 3,978 m², ≤ 11 m wide | ≈ 0.028 km², pieces ≤ 5,987 m² | median 0.0 m, p95 0.0 m |

- The large "gap" pieces (0.08–0.13 km², 120–145 m wide) sit at the tripoints TH–SN–ST (51.09°N 12.28°E), TH–SN–Bavaria
  (50.42°N 11.92°E), TH–ST–Lower Saxony (51.64°N 10.70°E), TH–HE–Bavaria (50.52°N 10.04°E) and TH–HE–Lower Saxony
  (51.38°N 9.93°E), where the third state fills them; they are artefacts of the measurement.
- All differences are below `region.rs`'s `MARGIN_M` (100 m): an area at the border queries both states' ALKIS.
- The Saxony-Anhalt enclaves at Billroda/Tauhardt and 51.33°N 11.42°E are 95–100 % covered by the ST asset; the 3,760 m² TH
  exclave at 51.21°N 11.44°E is 23 % inside the ST asset, the 4,481 m² exclave at 50.63°N 12.22°E 31 % inside the Saxony asset
  (both below the simplification noise).
- **Border probe (Billroda/Tauhardt, 2 × 2 km, `probe/border/overlay_border.jpg`):** TH ALKIS returns **1 building** and 124
  land use objects (fields and forest, ending at the border); ST ALKIS (`ST_LVermGeo_ALKIS_WFS_OpenData`) returns **390
  buildings** (the villages are in Saxony-Anhalt). The TH DOP WMS shows full imagery over the Saxony-Anhalt villages (tiles
  `32_671_5674/5675`, flight 01.05.2024), and the ST buildings line up with it. Each state's WFS returns only its own
  features, as `region.rs` expects.

---

## 11. OSM (Geofabrik)

- `https://download.geofabrik.de/europe/germany/thueringen-latest.osm.pbf` → **307** → `thueringen-261106.osm.pbf` → 206 to a
  range, **160,289,548 B**, `Last-Modified: Sat, 07 Nov 2026 02:15:55 GMT`, `ETag "98dd30c-65fa64f0b76b7"`, `Accept-Ranges: bytes`.
- State: `https://download.geofabrik.de/europe/germany/thueringen-updates/state.txt` →
  ```
  # original OSM minutely replication sequence number 7361559
  timestamp=2026-11-06T21\:21\:21Z
  sequenceNumber=4957
  ```
- Poly: `https://download.geofabrik.de/europe/germany/thueringen.poly`, one ring, 251 points, lon 9.863–12.661, lat 50.200–51.652.
- Overpass (overpass-api.de): one area query with `freemapper-research/0.1 (…)` answered (32 s), later small `around:`
  count queries got non-JSON answers (rate limiting); the OSM API `/api/0.6/way/<id>/full.json` worked.

---

## 12. HTTP quirks, reliability, limits

| Host / service | Quirks |
|---|---|
| `geoportal.geoportal-th.de` (download app, ATOM, files) | **Down**: every path → **200 / 206 `text/html` 970 B** outage page (`Last-Modified` 2016), even for `.zip`/`.pdf` URLs and Range requests. A client must check `Content-Type` and size, not only the status. `Server: Apache/2.4.6 (Linux/SUSE)`. |
| `geoportal.thueringen.de`, `tlbg.thueringen.de` | **Link11 CAPTCHA with HTTP 200** (`Content-Type: text/html`, 45.8 KB, `Cache-Control: no-cache, no-store`) for scripts, browser UAs and WebFetch. Never use these hosts in freemapper. The certificate CN is `100tage-programm.thueringen.de` (SAN covers the host). |
| `www.geoproxy.geoportal-th.de` WMS | max 5000 px, oversize → **HTTP 200 `text/xml`** ServiceExceptionReport; JPEG q75; white outside; 12–15 s per 5000² piece; `OAMAuthnCookie` set on every answer (ignore); no ETag/Last-Modified |
| `www.geoproxy.geoportal-th.de` WFS | GML only; unknown type → **HTTP 200** ExceptionReport; bad BBOX → 500; JSON → 400; `SORTBY` ignored; paged `numberMatched` = page size (use hits); cap 25,000; **25833 BBOX → 0 features**; no gzip; 0.1–6 s per request |
| `www.geoproxy.geoportal-th.de` unknown service | `…/services/<name>` → 500 `application/xml` "No service with identifier '<name>' available." |
| `geomis.geoportal-th.de` | CSW GET works (0.4–6 s); the GeoNetwork JSON search API → 403 (CSRF token) |
| TLS | All hosts send complete chains: geoproxy/geoportal-th/geomis `*.geoportal-th.de`/`*.geoproxy…` → Sectigo DV R36 → Sectigo Root R46 (+ cross-sign by USERTrust RSA, in webpki-roots); geoportal.thueringen.de and Geofabrik Let's Encrypt. No SH-style missing intermediate. |
| Range support | Wayback copies of the download host had `Accept-Ranges: bytes` and ETag; the live host could not be tested. Geofabrik: ranges OK. |

- No authentication, no rate-limit headers, no throttling seen on geoproxy in ≈ 120 requests.
- The download host was polled every 5 minutes from 01:29 UTC (`probe/poll/poll.log`); it stayed down for the whole session.

---

## 13. What makes the Thuringia adapter differ (risks first)

1. **DGM1 and DOM1 are unreachable right now.** The only open copies are the 1 km tile zips/ATOM feeds on
   `geoportal.geoportal-th.de`, which serves a maintenance page (HTTP 200/206) to every request; there is no WCS and no other
   1 m source. Until it returns (or a new host of the GDI-Th re-engineering appears), freemapper can build Thuringian maps
   only with the orthophoto and ALKIS (no hillshade/contours, no nDSM). The outage page has status 200, so the fetcher must
   reject `text/html` answers for tile URLs. The exact file paths, TIFF layout and Range support still need a check once the
   host is back.
2. **ALKIS spellings:** `gebnutzbez`/`nutzart` in ASCII title case. Without `lgv::normalise_building`/`normalise_land_use`
   (or an equivalent), **all buildings vanish** and nothing is private land. The Hamburg `spelling_key` maps every observed
   value correctly.
3. **No `rellage`, no `Bauteil`:** underground garages are ordinary `Gebaeude` with `funktion` "Tiefgarage" and would be
   drawn as buildings (28 in the samples, often under courtyards). Map `funktion == "Tiefgarage"` to `underground`.
4. **WFS quirks:** BBOX only in 25832 (25833 → silently 0 features, which would look like "no buildings"); `SORTBY`
   ignored (paging still stable, `fetch_wfs`'s hits + dedupe covers it); cap 25,000; no `NutzungFlurstueck`. The
   `WfsService` is `{url: …/adv_alkis_wfs, buildings_type: "ave:GebaeudeBauwerk", land_use_type: "ave:Nutzung", crs: Utm32,
   square_bez: "ohne Unterart (Thüringen)"}` plus the normalisation hooks.
5. **Licence changed in September 2026** to CC BY 4.0 ("© GDI-Th"); the metadata of older datasets and third parties still
   say dl-de/by-2-0. CC BY 4.0 wants the licence link and a change note in the attribution.
6. **Orthophoto vintages differ by half of the state** (2024 centre vs 2025 rest) and by season (March leaf-off to June
   leaf-on); flag the flight date per map from GetFeatureInfo.
7. **The portal and the TLBG web site are CAPTCHA-walled** (HTTP 200 HTML). Nothing in the adapter may depend on
   `geoportal.thueringen.de` or `tlbg.thueringen.de`; links in GovData/GeoMIS to them or to the download host are not usable.
8. **The boundary asset needs topology-preserving simplification**; the plain DP script made an invalid polygon. Thuringia
   contains Saxony-Anhalt enclaves (holes) and has four tiny exclaves.
9. Unchanged from other states: 1 km tiles in 25832, DHHN2016, deegree/AAA-vereinfacht GML handled by `parse_gml`, Geofabrik
   layout, no login, complete TLS chains. The WMS is the fastest orthophoto WMS of all freemapper states (5000 px, ≈ 12 s/km²).

---

## 14. Recommendation per freemapper product

| freemapper product | Use in Thuringia | Notes |
|---|---|---|
| **Orthophoto template** | **WMS** `https://www.geoproxy.geoportal-th.de/geoproxy/services/DOP`, layer `th_dop`, `CRS=EPSG:25832`, 0.2 m, **`max_px` 5000** | Currency: GetFeatureInfo `th_dop_info` (`Erstellung`, `Publikation`, tile `Name`); prefer `INFO_FORMAT=application/vnd.ogc.gml`. Tiles via `download.php` only if the download host returns. |
| **DTM** (hillshade, slope, contours) | **DGM1 tiles** `dgm1_32_EEE_NNNN_1_th_2020-2025.tif` from the download host / ATOM `atom_th_hoehendaten_dgm` — **currently unavailable** | Float32 LZW, NoData −9999 (README). No WCS. Re-check the host before implementing. |
| **nDSM template** | **DOM1 − DGM1** (ALS, same flight, `dom1_32_…_2020-2025.tif`) from the same host — **currently unavailable** | bDOM 0.2 m has no open download; LAZ (≈ 120 MB/km²) as fallback, same host. |
| **Buildings** | **WFS** `https://www.geoproxy.geoportal-th.de/geoproxy/services/adv_alkis_wfs`, `ave:GebaeudeBauwerk`, 25832, hits + `COUNT=10000` pages | Reuse `alkis_wfs` (GML) + Hamburg's spelling normalisation; `Tiefgarage` → underground. |
| **Land use** (out-of-bounds, paved) | the same WFS, **`ave:Nutzung`** | normalise `nutzart`; `square_bez` "ohne Unterart (Thüringen)"; `VERGE` works (`Begleitfläche Straßenverkehr`). |
| **Region boundary** | WFS `ave:VerwaltungsEinheit` DE16 (25833 output, 25832 bbox), topology-preserving 25 m simplification | Ready: `th/thueringen.geojson` (5,955 vertices, 5 polygons, 7 holes, valid). Region CRS `Utm32`; oid prefix `DETH`. |
| **OSM** | Geofabrik `europe/germany/thueringen-latest.osm.pbf` + `thueringen-updates/state.txt` | 160 MB; follow the 307 to `thueringen-YYMMDD.osm.pbf`. |
| **Attribution** | `Geobasisdaten: © GDI-Th, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/), verändert` + `© OpenStreetMap contributors, ODbL` + the freemapper credit line | §9 |

---

## 15. Not verified, or open

- **Everything on the download host:** DGM1/DOM1/LAZ/DOP tile URLs (only one archived LAZ URL is known; the README names
  the files inside the zips), zip layout, TIFF tags (tiepoint, GeoKeys, tiling, predictor), Range/ETag support, file sizes,
  coverage of the 2020–2025 vintage, the `Stand_*.zip` overviews, the ATOM dataset feeds, and whether the outage is
  maintenance or a migration. Re-test `https://geoportal.geoportal-th.de/hoehendaten/Uebersichten/README_Hoehendaten_2020-2025.pdf`
  (a PDF answer means the host is back).
- Whether the GDI-Th re-engineering will publish the planned MapServer **WCS** for DOP (FOSSGIS 2026) and perhaps for heights
  at a new host.
- The geoportal licence notice (news id 92) itself (CAPTCHA); the exact cut-off rule for data published before 14.09.2026.
- Whether the WMS pixels equal the DOP tile pixels on the native grid; RGBI tiles vs RGB WMS.
- bDOM 0.2 m availability (product exists; no open link found).
- WMS behaviour under sustained load (≈ 12 GetMaps were made); the 2 m layers; CIR.
- `nutzart`/`bez` values of rarer classes (Heide, Moor, Sumpf, Flugverkehr, Tagebau, Halde …) and of rural areas; the forest
  sample had only 9 buildings. `fktkurz` never occurred.
- Whether `funktion` "Tiefgarage" objects are always fully underground (some could be partly above ground).
- The boundary WFS polygon (7 holes vs ALKIS's 10) was not compared in detail.
- Overpass rate limits from this IP.

Network note: total transfer ≈ 130 MB, mostly ALKIS (two 31 MB cap tests, deleted; ≈ 25 MB samples), WMS probes (≈ 25 MB), Wayback archives
and the FOSSGIS slides (3 MB). No tile was downloaded (host down). Disk usage under `th/` ≈ 62 MB (plus a shapely venv under
`scratchpad/venv`).
