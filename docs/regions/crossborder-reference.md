# Cross-border geodata reference (DE–PL, DE–CZ)

Research date: 2026-11-07. Everything marked **[live]** was probed from this machine on that day
(HTTP status, file downloaded and inspected). Anything marked **[doc]** comes from an official web page
or service metadata. **[unverified]** marks statements from memory or secondary sources.

Samples are in `samples/` next to this file, one subdirectory per source (`lgb/`, `alkis/`, `osm/`,
`pl/`, `cz/`, `sn/`, `mv/`). Treat them as untrusted data. The analysis scripts are in `scripts/`.

---

## 0. Short answer

* **Do the German state datasets cross the border?** Barely. The LGB Brandenburg tile sets stop at the
  first tile that contains the border. DGM1, bDOM, nDOM and ALS reach a few hundred metres into Poland,
  and DOP reaches about 1–3 km. GeoSN Saxony clips DGM1 to the state border plus about 100 m and writes
  nodata beyond it. The ALKIS OGC API has no features outside the state. For a map that really crosses
  the Oder/Neiße you need foreign data.
* **Is foreign data openly available?** Yes, for both countries, with no authentication.
  * **Poland (GUGiK)**: free since 31 July 2020. Static file URLs on `opendata.geoportal.gov.pl`, found
    through WFS sheet indexes. Products: NMT 1 m (ASC), NMPT 0.5 m (ASC), LiDAR LAZ (12 pt/m², 2024),
    orthophoto 0.25 m / 0.05 m (GeoTIFF), BDOT10k per powiat (GML/SHP/GPKG), and EGiB buildings via a
    national WFS that returns GeoJSON. CRS is EPSG:2180 and heights are PL-EVRF2007-NH.
  * **Czechia (ČÚZK)**: CC BY 4.0. ATOM feeds with static ZIP URLs on `openzu.cuzk.gov.cz`. Most raster
    and point-cloud products also come in **ETRS89/TM33N (EPSG:3045)** on a 2 × 2 km grid that matches
    UTM33 kilometres. That is the same projection freemapper already uses, so no reprojection is
    needed. Buildings come as INSPIRE BU per municipality in EPSG:4258, and ZABAGED via WFS in
    EPSG:5514.
* **Recommendation:** use OSM (Geofabrik Poland/Czech extracts) for the foreign side as a first step.
  Then add a raster-only DTM + orthophoto adapter for each country. Czechia is cheap because the CRS
  is already right. Poland is moderate: one more Transverse Mercator parameter set plus sheet lookup
  via WFS. Add official foreign vector data (buildings/land cover) last.

---

## 1. German data across the border

### 1.1 LGB Brandenburg (data.geobasis-bb.de)

Currency CSVs: `https://data.geobasis-bb.de/geobasis/information/aktualitaeten/` [live, 200]. Format
`sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate`, sheet `33EEE-NNNN`
(1 km). Relevant files: `bb_dgm_aktualitaet.csv` (31 291 tiles), `bb_bdom_aktualitaet.csv`,
`bb_dop_aktualitaet.csv`, `bb_laserscandaten_aktualitaet.csv`. The nDOM directory lists
`NDOM_33EEE-NNNN_LAS12.laz` directly (30 520 files) [live].

Border positions used (computed with pyproj, EPSG:25833):
Frankfurt (Oder) Stadtbrücke over the Oder ≈ E 469 590 / N 5 799 472; Słubice centre ≈ E 470 234;
Guben Neiße bridge ≈ E 480 963 / N 5 755 736; Gubin centre ≈ E 481 787.
(The E 474–477 km in the brief lies 4–7 km inside Poland, east of Słubice.)

Easternmost tile per row [live, from the CSVs]:

| N (km) | DGM1 max E | DOP max E | Border at |
|---|---|---|---|
| 5800–5803 (Frankfurt/Słubice) | 469 | 471 | ≈ 469.6 |
| 5753–5757 (Guben/Gubin) | 480 | 481 | ≈ 480.9 |

Statewide comparison of the eastern edge (max-E per row) [live]: DOP extends 0–5 km (median 2 tiles)
beyond the DGM edge, bDOM 0–4 km (median 1), and LAS equals DGM.

Direct probes [live] (`200` = exists, `404` = missing):

| Tile | dgm/tif | bdom/tif | ndom (LAZ) | als/laz | dop/rgbi_tif | dop/rgb_jpg |
|---|---|---|---|---|---|---|
| 33469-5801 | 200 | 200 | 200 | 200 | 200 | 200 |
| 33470-5801 | 404 | 404 | 404 | 404 | 200 | 200 |
| 33471-5801 | 404 | 404 | 404 | 404 | 200 | 200 |
| 33474-5801 / 33476-5802 | 404 | 404 | – | – | 404 | – |
| 33480-5755 | 200 | – | 200 | – | – | – |
| 33481-5755 | 404 | – | – | – | 200 | 200 |
| 33482-5755 | 404 | – | – | – | 404 | 404 |

Sample URLs: `https://data.geobasis-bb.de/geobasis/daten/dgm/tif/dgm_33469-5801.zip`,
`.../bdom/tif/bdom_33469-5801.zip`, `.../dop/rgbi_tif/dop_33471-5801.zip`,
`.../dop/rgb_jpg/dop_33481-5755.zip`, `.../ndom/NDOM_33469-5801_LAS12.laz`.

Content of the border tiles [live, downloaded and analysed]:
* `dgm_33469-5801.tif` (1000 × 1000, float32, nodata −9999): **0 % nodata**. The tile spans the Oder,
  and the ~300 m on the Polish bank hold real LiDAR terrain (floodplain structures are visible).
  `dgm_33480-5755` (Guben) is also 0 % nodata.
* `bdom_33469-5801.tif` (5000 × 5000, 0.2 m): 0 % nodata.
* `dop_33471-5801.tif` (5000 × 5000 RGBI, 87 MB): full image of the Słubice outskirts, no nodata.

Conclusion: LGB tiles are **not cut at the border**. Whole 1 km tiles are delivered, so DGM/bDOM/nDOM
end 0–1 km past the border and DOP typically 1–3 km past it. Anything deeper into Poland needs
Polish data.

### 1.2 LGB ALKIS OGC API

`https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections` [live] lists `flurstueck`,
`flurstueck_punkt`, `gebaeude_bauwerk`, `katasterbezirk`, `nutzung`, `nutzung_flurstueck`,
`verwaltungseinheit`. Bbox queries (`/collections/{id}/items?f=json&limit=1000&bbox=lon,lat,lon,lat`)
[live]:

| bbox | gebaeude_bauwerk | nutzung |
|---|---|---|
| Słubice 14.565,52.345,14.580,52.355 | 0 | 0 |
| Gubin 14.735,51.945,14.750,51.955 | 0 | 0 |
| Frankfurt (control) 14.535,52.340,14.548,52.350 | 771 | 515 |

Confirmed: there are no ALKIS features on the Polish side.

### 1.3 GeoSN Saxony

Download mechanism [live]: `https://www.geodaten.sachsen.de/batch-download-4719.html` builds Nextcloud
public-share URLs `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/{share_id}/{file}`.
Share IDs from the page's product config:

| Product | share_id | file pattern (2 km tiles) |
|---|---|---|
| DGM1 GeoTIFF | `JCcXyifaNdLDnxZ` | `dgm1_33{EEE}_{NNNN}_2_sn_tiff.zip` |
| DOM1 GeoTIFF | `S6wwnFwX7882sZm` | `dom1_33{EEE}_{NNNN}_2_sn_tiff.zip` |
| DOP20 RGB | `QQFLq6nkoSnqB5g` | `dop20rgb_33{EEE}_{NNNN}_2_sn_tiff.zip` |
| DOP20 RGBI | `sX3GPcdBMGrfXT9` | `dop20rgbi_33{EEE}_{NNNN}_2_sn_tiff.zip` |
| ALKIS per municipality | `GsgZjSdwH45x8PJ` | `{Gebietsschlüssel}.xml.zip` |

`HEAD` and `PROPFIND` return **401**, but plain `GET` works (200/206). An adapter must use GET, not
HEAD, to check tile existence. Range GETs of large DOP files were very slow.

Probes [live]:
* `dgm1_33500_5668_2_sn_tiff.zip` (Görlitz, on the Neiße, 2.9 MB): GeoTIFF 2000 × 2000, EPSG:25833,
  nodata −9999, **80 % nodata**. Data ends with a smooth rounded edge about **100 m east of the
  Neiße**, so the clip is at the state border plus a small buffer.
* `dgm1_33488_5636_2_sn_tiff.zip` (Zittau, near the DE/PL/CZ tripoint): 99 % nodata.
* The page's district→1 km-cell mapping puts the eastern edge at E 500 km at Görlitz (Neiße ≈ E 500)
  and the southern edge at N 5629–5637 near Zittau/Hřensko. Saxony provides essentially **no data
  beyond the border**.

Licence [doc, via search, page text not fetched]: Datenlizenz Deutschland – Namensnennung 2.0, source
"Geodaten Sachsen" / GeoSN.

### 1.4 LAiV Mecklenburg-Vorpommern

ATOM [live]: `https://www.geodaten-mv.de/dienste/dgm_atom` → DGM1 dataset feed
`https://www.geodaten-mv.de/dienste/dgm_atom?type=dataset&id=ca268792-s2q1-4a39-b34c-9ec5bf9a4469`
(15 MB, 6407 tiles × 6 formats: gtiff, xyz, mix, isoli, schum, zcode). Download pattern:
`https://www.geodaten-mv.de/dienste/dgm_download?index=4&dataset=ca268792-s2q1-4a39-b34c-9ec5bf9a4469&file=dgm1_33_EEE_NNNN_2_gtiff.tif`
(uncompressed GeoTIFF, EPSG:25833, 2 km). Rights text in the feed: "© GeoBasis-DE/M-V (Jahr der
letzten Datenlieferung)".

Eastern edge [live, from the feed]: at N 5930 the last tile is 454 (covers to E 456 km), and at
N 5974–5976 it is 448 (to E 450). The Geofabrik MV polygon crosses N 5931 at ≈ E 455.4 and Ahlbeck/
Świnoujście is at ≈ E 449.2. Probe `dgm1_33_454_5930_2_gtiff.tif` (16 MB): **0 % nodata** and real
LiDAR detail across the whole tile. MV therefore behaves like Brandenburg (whole tiles at the border,
at most about 1 km into Poland). The exact Polish share of that tile was not measured against an
official border line **[partly unverified]**.

### 1.5 OSM / Geofabrik

Sizes [live, HEAD after 307 redirect, Last-Modified 6/7 Nov 2026]:

| Extract | URL | Size |
|---|---|---|
| Brandenburg | https://download.geofabrik.de/europe/germany/brandenburg-latest.osm.pbf | 297 MB |
| Berlin | .../europe/germany/berlin-latest.osm.pbf | 100 MB |
| Sachsen | .../europe/germany/sachsen-latest.osm.pbf | 270 MB |
| Mecklenburg-Vorpommern | .../europe/germany/mecklenburg-vorpommern-latest.osm.pbf | 129 MB |
| Poland | .../europe/poland-latest.osm.pbf | 2.12 GB |
| PL lubuskie | .../europe/poland/lubuskie-latest.osm.pbf | 63 MB |
| PL zachodniopomorskie | .../europe/poland/zachodniopomorskie-latest.osm.pbf | 98 MB |
| PL dolnośląskie | .../europe/poland/dolnoslaskie-latest.osm.pbf | 182 MB |
| PL wielkopolskie | .../europe/poland/wielkopolskie-latest.osm.pbf | 160 MB |
| Czech Republic | .../europe/czech-republic-latest.osm.pbf | 952 MB |
| CZ ústecký | .../europe/czech-republic/ustecky-latest.osm.pbf | 112 MB |
| CZ liberecký | .../europe/czech-republic/liberecky-latest.osm.pbf | 47 MB |
| CZ karlovarský | .../europe/czech-republic/karlovarsky-latest.osm.pbf | 31 MB |

Poland has 16 voivodeship sub-extracts. **Czechia now also has 14 kraj sub-extracts** [live]:
jihocesky, jihomoravsky, karlovarsky, kralovehradecky, liberecky, moravskoslezsky, olomoucky,
pardubicky, plzensky, praha, stredocesky, ustecky, vysocina, zlinsky.

Clip polygons (`.poly`) [live]: `brandenburg.poly` has 200 vertices, `sachsen.poly` 328,
`mecklenburg-vorpommern.poly` 137. They are simplified outlines with a small, uneven buffer. At the
Frankfurt Stadtbrücke latitude the Brandenburg polygon reaches 14.5621° E, about 580 m east of the
bridge (14.5536°). At Guben it lies on the Neiße (14.7222° vs bridge 14.7230°). No documented buffer
size was found. Do not count on the German state extracts for anything on the foreign side. Use the
PL/CZ extracts, or a sub-extract plus `osmium merge`. Geofabrik's way-completion strategy at the
boundary was not checked **[unverified]**.

---

## 2. Poland (GUGiK)

Licence [doc]: geoportal.gov.pl, NMT page: "Dane NMT są dostępne bezpłatnie i możliwe do dowolnego
wykorzystania." (free, any use). Legal basis: art. 40a ust. 2 Prawo geodezyjne i kartograficzne, as
amended by the act of 16 April 2020 (Dz.U. 2020 poz. 782), in force since **31 July 2020**. The EGiB WFS
capabilities give `Fees: none`, `AccessConstraints: none`. No mandatory attribution string was found.
Suggested: "Źródło: Główny Urząd Geodezji i Kartografii (geoportal.gov.pl)" **[wording is our
suggestion]**. **No authentication** is needed anywhere.

CRS: PL-1992 = **EPSG:2180** (ETRS89 / Poland CS92: TM, lon₀ 19°, k 0.9993, FE 500 000,
FN −5 300 000, GRS80). Heights are **PL-EVRF2007-NH**; older KRON86-NH versions also exist.

**Axis-order pitfalls** [live]: WFS 2.0 `BBOX` with `urn:ogc:def:crs:EPSG::2180` takes **N,E**
(e.g. `507000,198000,507900,198900` for Słubice). The data itself comes back as **E,N**: BDOT10k GML
posList, EGiB GeoJSON, and ASC `xllcenter` = easting.

### 2.1 Sheet indexes (WFS), the main entry point

Each feature has `godlo` (sheet ID), `akt_rok`, `format`, `char_przestrz` (resolution), `uklad_h`,
`zrodlo_danych` (LiDAR vs photogrammetry), `czy_ark_wypelniony` and **`url_do_pobrania`** (direct
file URL). One layer per year (2018…2026) [live]:

| Product | WFS | Layers |
|---|---|---|
| NMT (DTM) | https://mapy.geoportal.gov.pl/wss/service/PZGIK/NumerycznyModelTerenuEVRF2007/WFS/Skorowidze | `gugik:SkorowidzNMT2018`…`2026` |
| NMPT (DSM) | https://mapy.geoportal.gov.pl/wss/service/PZGIK/NumerycznyModelPokryciaTerenuEVRF2007/WFS/Skorowidze | `gugik:SkorowidzNMPT2019`…`2026` |
| LiDAR LAZ | https://mapy.geoportal.gov.pl/wss/service/PZGIK/DanePomiaroweLidarEVRF2007/WFS/Skorowidze | `gugik:SkorowidzDanychPomiarowychLIDAR2018`…`2026` |
| Orthophoto | https://mapy.geoportal.gov.pl/wss/service/PZGIK/ORTO/WFS/Skorowidze | `gugik:SkorowidzOrtofomapy1957`…`2026` |
| BDOT10k packages | https://mapy.geoportal.gov.pl/wss/service/PZGIK/BDOT/WFS/PobieranieBDOT10k | `ms:BDOT10k_powiaty` (380 powiats) |
| EGiB (cadastre) | https://mapy.geoportal.gov.pl/wss/service/PZGIK/EGIB/WFS/UslugaZbiorcza | `ms:budynki`, `ms:dzialki` |

KRON86 variants exist with `...KRON86...` in the path. Sample index query [live]:
```
https://mapy.geoportal.gov.pl/wss/service/PZGIK/NumerycznyModelTerenuEVRF2007/WFS/Skorowidze?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=gugik:SkorowidzNMT2024&BBOX=507000,198000,507900,198900,urn:ogc:def:crs:EPSG::2180&COUNT=20
```

Sheet scheme: 1:5000 sheets of the "1992" division (e.g. `N-33-126-C-c-4-3`, about 2.27 × 2.45 km).
The 1:1250 sub-sheets (`…-4-3-1-1`, about 0.6 × 0.6 km) are used for LAZ. Sheets are not aligned
with the PL-1992 grid, so the ASC/TIF holds the sheet's bounding box with nodata outside the sheet.

### 2.2 Results for Słubice (sheet N-33-126-C-c-4-3) and its western neighbour [live]

| Year | Product | Res. | Source | URL | Size |
|---|---|---|---|---|---|
| 2020 | NMT ASC | 1 m | LiDAR (σh 0.10) | https://opendata.geoportal.gov.pl/NumDaneWys/NMT/73426/73426_993520_N-33-126-C-c-4-3.asc | 33 MB |
| 2020 | NMT XYZ | 1 m | aerial images | …/NMT/73778/73778_1030711_N-33-126-C-c-4-3.xyz.zip | – |
| 2022 | NMT ASC | 1 m | aerial images (σh 0.15) | …/NMT/76153/76153_1192533_N-33-126-C-c-4-3.asc | 33 MB |
| 2022/2025 | NMT ASC | 5 m | aerial images | …/NMT/83845/83845_1820794_N-33-126-C-c-4-3.asc | – |
| 2024 | NMT ASC (c-3-4) | 1 m | – | …/NMT/80548/80548_1546831_N-33-126-C-c-3-4.asc | 32 MB |
| 2024 | NMPT ASC | **0.5 m** | LiDAR | https://opendata.geoportal.gov.pl/NumDaneWys/NMPT/80547/80547_1546806_N-33-126-C-c-4-3.asc | 133 MB |
| 2024 | LAZ | **12 pt/m²** (1:1250 sheets) | LiDAR | https://opendata.geoportal.gov.pl/NumDaneWys/DanePomiaroweLAZ/80545/80545_1546932_N-33-126-C-c-4-3-1-1.laz | 65 MB |
| 2024 | Ortho RGB | **0.05 m** | – | https://opendata.geoportal.gov.pl/ortofotomapa/80550/80550_1321875_N-33-126-C-c-4-3.tif | **1.43 GB** |
| 2025 | Ortho RGB | 0.25 m | – | https://opendata.geoportal.gov.pl/ortofotomapa/83847/83847_1549315_N-33-126-C-c-4-3.tif | 44 MB |
| 2025 | Ortho CIR | 0.25 m | – | …/ortofotomapa/83846/83846_1553252_N-33-126-C-c-4-3.tif | 45 MB |

All return `200 application/octet-stream` [live]. File inspection [live]:
* NMT ASC: plain ESRI ASCII grid, `xllcenter/yllcenter` (pixel centres on integer metres),
  `nodata_value -9999`. The 2020 LiDAR sheet has 11 % nodata (sheet-shape corners).
* **Polish NMT reaches into Germany**: sheet c-3-4 (2024) is clipped with a smooth edge about
  **300 m west of the Oder**, so a few hundred metres of Frankfurt (Oder) are included.
* Ortho 0.25 m: tiled JPEG GeoTIFF with 10 pages (overviews), GeoKey ProjectedCRS 2180, citation
  "ETRF2000-PL/CS92". A full rectangle with no nodata.
* LAZ header: LAS 1.2, point format 3 (RGB), sensor VQ-780II, TerraScan, 8.97 M points on about
  0.35 km² (≈ 26 pt/m² including overlap). Z −3…66 m.

### 2.3 WCS (on-the-fly GeoTIFF, alternative to sheets)

`https://mapy.geoportal.gov.pl/wss/service/PZGIK/NMT/GRID1/WCS/DigitalTerrainModelFormatTIFF` [live]:
WCS 2.0.1/1.1.1/1.0.0 with **one coverage only, `DTM_PL-KRON86-NH_TIFF`** (old height datum), axes
`y x`, EPSG:2180. Sample:
```
...?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=DTM_PL-KRON86-NH_TIFF&FORMAT=image/tiff&SUBSET=x(198000,198500)&SUBSET=y(507000,507500)
```
→ `200 image/tiff`, 500 × 500 float32, 1 m, 1.0 MB, no nodata tag. The page states a limit of about
7 km² per request [doc]. The obvious NMPT WCS URL pattern returned 404. Because of KRON86 and the size
limit, prefer the EVRF2007 sheets.

### 2.4 Vector: BDOT10k and EGiB buildings

* BDOT10k per powiat (TERYT) [live]: `https://opendata.geoportal.gov.pl/bdot10k/schemat2021/{WW}/{TTTT}_GML.zip`,
  also `/SHP/{WW}/{TTTT}_SHP.zip` and `/GPKG/{WW}/{TTTT}_GPKG.zip`. Border powiats: 0805 słubicki
  (GML 54 MB / SHP 79 MB / GPKG 81 MB), 0802 krośnieński (Gubin), 0811 żarski, 0225 zgorzelecki,
  0210 lubański, 3206 gryfiński, 3211 policki, 3262 Szczecin, 3263 Świnoujście. Content of 0805:
  one file per class, `OT_BUBD_A` (buildings, 21 940 objects, geometry source "EGiB",
  floors, function), `OT_PTLZ_A` (forest), `OT_PTTR_A` (grassland), `OT_PTWP_A` (water),
  `OT_PTZB_A` (built-up), `OT_SWRS_L` (watercourses) etc. GML posList is E,N in EPSG:2180.
* EGiB national WFS [live]:
  ```
  https://mapy.geoportal.gov.pl/wss/service/PZGIK/EGIB/WFS/UslugaZbiorcza?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ms:budynki&BBOX=507300,198300,507500,198500,urn:ogc:def:crs:EPSG::2180&COUNT=500&OUTPUTFORMAT=application/json
  ```
  → `200 application/json`, 42 buildings in 200 × 200 m of Słubice, properties `ID_BUDYNKU`,
  `RODZAJ`, `KONDYGNACJE_NADZIEMNE/PODZIEMNE`, coordinates [E, N]. This is the closest Polish
  counterpart to the ALKIS API. It was announced on 2026-07-16 [doc], and its limits are
  undocumented.
* The `PobieranieBDOT10k` WFS ignored our bbox (0 features). Fetch it without a bbox using
  `PROPERTYNAME=TERYT,NAZWA_POWIATU,Data_aktualizacji,URL_GML` (136 kB) and pick the powiat.

---

## 3. Czechia (ČÚZK)

Licence [doc/live]: ATOM portal footer "Data poskytována pod licencí: Creative Commons CC-BY 4.0".
Feed `<rights>`: "žádné podmínky neplatí" (no conditions). ZABAGED WFS `Fees`: "© ČÚZK". Suggested
attribution: "© ČÚZK, CC BY 4.0". The date of 1 July 2023 for the opening comes from the brief and
was not re-checked **[unverified]**. No authentication.

Service catalogue [live]: `https://atom.cuzk.gov.cz/getcapabilities.ashx` (JSON, 115 kB; the HTML
portal loads it via JS). Top-level feed: `https://atom.cuzk.gov.cz/{NAME}/{NAME}.xml`. Per-tile
dataset feed: `https://atom.cuzk.gov.cz/{NAME}/datasetFeeds/CZ-00025712-CUZK_{NAME}_{EEE}_{NNNN}.xml`.
The download link in it is `<link rel="alternate" length=…>`. Both `cuzk.cz` and `cuzk.gov.cz` work.

**Key finding: the ETRS89/TM33N (EPSG:3045) variants use a 2 × 2 km grid named `EEE_NNNN` in UTM33
km.** EPSG:3045 is the same projection as EPSG:25833 with N,E axis order in the EPSG definition, and
the files use E,N. 20 308 tiles per product.

| Name | Content | Format | Grid | Sample download (tile 488_5634, Hrádek nad Nisou) |
|---|---|---|---|---|
| DMR5G-ETRS89 | DTM 5th gen. = thinned ground points | LAZ | 2 km | https://openzu.cuzk.gov.cz/opendata/DMR5G/epsg-3045/488_5634.zip (1.8 MB) |
| DMR4G-ETRS89-TIFF | DTM 4th gen., 5 m grid | GeoTIFF | 2 km | https://openzu.cuzk.gov.cz/opendata/DMR4G-TIFF/epsg-3045/488_5634.zip (0.46 MB) |
| DMP1G-ETRS89 | DSM 1st gen. (LiDAR) | LAZ | 2 km | https://openzu.cuzk.cz/opendata/DMP1G/epsg-3045/488_5634.zip (1.5 MB) |
| DMPOK-ETRS89-TIFF | DSM from image correlation | GeoTIFF | 2 km | https://openzu.cuzk.gov.cz/opendata/DMPOK-TIFF/epsg-3045/488_5634.zip (38 MB) |
| DMPOK-ETRS89-LAZ | same, point cloud | LAZ | 2 km | (feed exists) |
| OI | INSPIRE orthophoto | JPEG2000 + .j2w + GML | 2 km | https://openzu.cuzk.gov.cz/opendata/OI/488_5634.zip (77 MB) |
| DMR5G-SJTSK / DMP1G-SJTSK / DMPOK-SJTSK-* / DMR4G-SJTSK | same in S-JTSK | LAZ/TIFF | SM5 sheets | e.g. `...DMR5G-SJTSK_BENE09` |
| ORTOFOTO | orthophoto JPEG, S-JTSK + extra JGW for TM33/TM34 | JPEG | SM5 sheets | – |
| ZABAGED-GPKG | whole country | GPKG | – | `.../ZABAGED-GPKG/epsg-3045/ZABAGED-3045-gpkg-20261004.zip` (**6.7 GB**), 5514 version 5.8 GB |
| ZABAGED-vyskopis | contour lines | SHP | SM5 | – |
| BU (INSPIRE buildings) | per municipality | GML 3.2.1 | obec | https://services.cuzk.gov.cz/gml/inspire/bu/epsg-4258/{obec}.zip |
| RUIAN-S-K-U | RÚIAN per municipality (VFR) | XML | obec | https://services.cuzk.cz/atom-index/RUIAN-S-K-U/5514 |

Probes [live]:
* DMR5G `488_5634.laz`: LAS 1.4, point format 6, LAStools `lasclip64`, **no CRS VLR**, 296 693 points
  on about 3.6 km², all class 8 (key points), so about **0.08 pt/m²**. It is a thinned TIN model,
  not a full point cloud. It must be triangulated/interpolated to a 1 m grid. Z 238–306 m.
* **Clipped at the Czech border**: the point-density grid is empty north of about N 5 635 800, and
  tile `446_5638` (Hřensko/Schmilka) has only 1958 points in a sliver. The DMR4G 5 m GeoTIFF of
  the same tile is 24 % nodata in the north, and the OI orthophoto (16 000 × 16 000, **0.125 m**,
  .j2w origin 488000.0625 / 5635999.9375) shows the same smooth rounded cut. Any buffer is no more
  than about 100–200 m (not measured against the official border line). The ČÚZK side has
  **no data in Germany or Poland**.
* DMR4G TIFF: 400 × 400 float32 LZW, nodata −9999, **no GeoTIFF geokeys** (only `.tfw`, CRS
  implicit). The adapter must read the TFW and assume EPSG:3045.
* ZABAGED WFS [live]: `https://ags.cuzk.gov.cz/arcgis/services/ZABAGED_POLOHOPIS/MapServer/WFSServer`,
  149 layers, DefaultCRS **EPSG:5514**. Buildings layer
  `ZABAGED_POLOHOPIS:Budova_jednotlivá_nebo_blok_budov__plocha_` → 429 matched in 800 × 800 m of
  Hrádek (bbox `-702000,-963000,-701200,-962200,urn:ogc:def:crs:EPSG::5514`). Land-cover layers
  include `Lesní_půda_se_stromy`, `Orná_půda_a_ostatní…`, `Ovocný_sad__zahrada`, `Vodní_plocha`,
  `Rašeliniště`, and O-relevant point features (`Osamělý_balvan__skála__skalní_suk`,
  `Skupina_balvanů`, `Vstup_do_jeskyně`, `Lesní_průsek`). A request with `srsName=EPSG:25833`
  returned 0 features, so reprojection by the server is not available as tested.
* INSPIRE BU EPSG:4258 [live]: listing has about 6260 municipality ZIPs. `564028.zip` (0.7 MB → 33 MB
  GML) has 5275 buildings with 2D polygons in lat,lon order (EPSG:4258). Usable with the existing
  `utm33_forward`.

Height datum: Czech heights are **Bpv** (Baltic after adjustment, Kronstadt). freemapper uses DHHN2016
(Amsterdam). The offset is on the order of 0.1–0.2 m. The exact value and sign were not measured,
because the Czech data has no overlap with German data **[unverified]**. EPSG offers a Bpv→EVRF2007
transformation [doc, search].

---

## 4. Recommendation

### 4.1 What a cross-border orienteering map needs

* **Base map / vectors (roads, paths, water, buildings, vegetation):** the Geofabrik **Poland or Czech
  sub-extract** (lubuskie 63 MB, zachodniopomorskie 98 MB, dolnośląskie 182 MB; liberecký 47 MB,
  ústecký 112 MB) merged with the German state extract. This needs **zero new CRS code**, because OSM
  is WGS84 and freemapper already projects to 25833. OSM coverage of paths and forests near Słubice,
  Gubin and the Elbe Sandstone area is typically good, though not verified here.
  **OSM-only is an acceptable first step for the foreign side.**
* **Contours / DTM:** essential for O-maps and not covered by OSM. This should be the first official
  foreign adapter. On the border strip the German tiles already give 0.3–1 km (BB/MV) or about 0.1 km
  (SN) of foreign terrain, which is enough for a forest that only touches the border.
* **Orthophoto:** needed for mapping the foreign area. LGB already covers 1–3 km into Poland.
* **Vegetation height (nDSM):** PL NMPT 0.5 m minus NMT, or the PL LAZ directly. For CZ, DMP1G minus
  DMR5G.
* **Official buildings/land cover:** nice to have. PL EGiB WFS (GeoJSON, E,N) is easy. CZ: INSPIRE BU
  in EPSG:4258 per municipality, or ZABAGED WFS in 5514.

### 4.2 Adapter cost estimate

**Czechia (cheap):**
* CRS: use the `*-ETRS89` / `epsg-3045` products. They are on the 2 km UTM33 grid, so freemapper's
  tiling (snapped boxes) maps 1:1 and **no reprojection is needed**.
* Index: no index service is needed. Build `datasetFeeds/..._{EEE}_{NNNN}.xml` for each 2 km tile,
  read the `<link rel="alternate" href length>`, and treat 404 as no tile. (Or derive
  `https://openzu.cuzk.gov.cz/opendata/{PRODUCT}/epsg-3045/{EEE}_{NNNN}.zip` directly; the pattern
  is consistent for DMR5G, DMR4G-TIFF, DMPOK-TIFF and DMP1G.)
* DTM: DMR5G is thinned LAZ (about 0.1 pt/m²), so it needs TIN/IDW gridding to 1 m. The `las` crate with
  `laz` is already a dependency. Alternatively use DMR4G 5 m GeoTIFF (simpler, coarser) with the
  TFW-only georef.
* Ortho: JPEG2000 (OI) needs a JP2 decoder (e.g. an OpenJPEG binding). That is a new native
  dependency and the main cost. The alternative is the `ORTOFOTO` JPEG feed (SM5 sheets, S-JTSK with
  extra TM33 world files). That avoids JP2 but the sheets do not match the grid.
* Height datum: Bpv vs DHHN2016 about 0.1–0.2 m. Ignore for 2.5/5 m contours, or apply a constant.
* Buildings: INSPIRE BU (EPSG:4258 lat,lon) per municipality → existing `utm33_forward`.
* Rough effort: about 1–2 days for DTM + nDSM via LAZ/TIFF, plus about 1 day for orthophoto if JP2 is
  accepted.

**Poland (moderate):**
* CRS: EPSG:2180 is Transverse Mercator on GRS80/ETRS89, with no datum shift (ETRF2000-PL vs
  ETRS89 is at the cm level). freemapper's `crs.rs` already has its own TM, so this means one more
  parameter set (lon₀ 19°, k 0.9993, FE 500 000, FN −5 300 000). However, rasters must be
  **resampled** from 2180 to 25833, because the grids are rotated relative to each other by grid
  convergence difference of about 3° at 14.6° E / 52° N (computed, not measured). That needs a warp step (bilinear for DTM, nearest/bilinear for
  ortho).
* Index: WFS GetFeature per year layer, with BBOX in **N,E** order, then pick the newest
  `akt_rok` with `zrodlo_danych = Skaning laserowy` and `char_przestrz = 1.00 m`. Sheets are
  quadrilaterals, so take the union of all hits.
* Formats: ASC (text, 33 MB per sheet, trivial parser), GeoTIFF ortho (JPEG-compressed tiles; check
  that the `tiff` crate handles JPEG-in-TIFF, otherwise this is a new dependency). Avoid the 0.05 m
  ortho (1.4 GB per sheet) and pick 0.25 m.
* Height datum: PL-EVRF2007-NH vs DHHN2016 measured [live] on the Oder overlap at Frankfurt (LGB
  `dgm_33469-5799` vs PL NMT 2024 c-3-4, 109 014 pixels): **median +0.03 m**, p10 −0.06 m. This
  agrees within a few cm, so no correction is needed. Do not use KRON86: WCS KRON86 vs EVRF2007 ASC
  shows a median of 0.12 m, mixed with differences between survey years.
* Vectors: EGiB WFS GeoJSON buildings, or BDOT10k GPKG per powiat (forest/grassland/water polygons
  ≈ ALKIS actual land use).
* Rough effort: about 2–3 days (TM parameters + warp + WFS index + ASC reader), plus about 1 day for
  buildings/land cover.

**Border seams:** German, Polish and Czech coverage overlap by 0–1 km (BB/MV vs PL) or meet with a
gap of less than about 100 m (SN vs CZ/PL). Mosaic with priority (home state first) and feather the
DTM over the overlap. The measured 3 cm datum difference with Poland will not show as a contour step.

### 4.3 Suggested order

1. Done today: German tiles plus Geofabrik PL/CZ sub-extract merged into the OSM input. Add a region
   rule: if the area intersects PL/CZ, also fetch the neighbouring extract.
2. CZ DTM (DMR5G or DMR4G, EPSG:3045 tiles).
3. PL DTM (NMT 1 m ASC via WFS index, warp 2180 → 25833).
4. Orthophoto PL (0.25 m GeoTIFF) / CZ (OI JP2).
5. Official foreign buildings/land cover (PL EGiB/BDOT10k, CZ INSPIRE BU/ZABAGED).

Every output must credit the sources: "© GeoBasis-DE/LGB", "© GeoBasis-DE/M-V (year)", "Geodaten
Sachsen / GeoSN (dl-de/by-2-0)", "GUGiK (geoportal.gov.pl)", "© ČÚZK, CC BY 4.0", "© OpenStreetMap
contributors (ODbL)", plus the freemapper credit line.

---

## 5. Not verified live

* Geofabrik's documented buffer size (none found; only measured examples) and its way-completion
  strategy.
* GeoSN licence wording (from a search snippet) and the GeoSN DOP clipping (only DGM1 was probed).
* Exact Czech clip buffer against the official border line, and the Bpv–DHHN2016 offset value/sign.
* The ČÚZK open-data start date of 1 July 2023.
* Polish WMS ortho endpoints, and any rate limits on GUGiK/ČÚZK (none observed in about 40 requests each).
* OSM data quality on the foreign side.
