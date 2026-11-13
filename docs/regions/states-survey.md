# Open-data survey for freemapper: NI, HB, NRW, RP, SL, BW, BY

Checked live on 2026-11-08 (curl, GetCapabilities plus one 100 m bbox request per service; no large downloads).
Needs per `docs/regions/hessen-reference.md` section 1 and 14: (1) DOP <= 0.2 m with flight date, (2) DGM1 float32,
(3) surface model for an nDSM, (4) ALKIS-vereinfacht buildings + actual land use (WFS/OGC API), plus licence.
Samples (tiny test responses only) are in `work/<state>/` next to this file; all are untrusted data.

## Verdict summary

| State | Verdict | Effort | Why |
|---|---|---|---|
| NRW | ready | low | Same AAA-suite `ave:` WFS as Hesse/Saxony; DGM1/DOM1 GeoTIFF tiles by plain HTTP; WMS+WCS; dl-de/zero (no attribution) |
| Niedersachsen | ready | low-medium | Same AAA-suite `ave:` WFS; DOP/DGM1/DOM1 are Cloud-Optimized GeoTIFFs found via STAC (range reads); CC BY 4.0 |
| Rheinland-Pfalz | ready | medium | `ave:` WFS on a different server (no `NutzungFlurstueck`, uses `ave:Nutzung`); DGM1/DOM1 1 km tif tiles; DOP via WMS or 155 MB JP2; dl-de/by-2-0 |
| Baden-Wuerttemberg | done 2026-11-13 (ADR 0119, `baden-wuerttemberg-reference.md`) | medium-high | The row of 2026-11-08 was wrong in four points: there is an open ALKIS WFS with buildings and land use (GeoServer views, not the simplified schema), the 2 km zips are deflated and still readable by range per member, DGM1 is XYZ text, and the flight date of the orthophoto is in an open WFS. Orthophoto from the WMS, DGM1 and nDOM1 from the zips |
| Bayern | partly | high | DOP20/DGM1/DOM20 open, but buildings (Hausumringe) and land use (TN) only as bulk downloads (Shape per Regierungsbezirk, 5 GB GPKG); no WFS |
| Bremen | partly | medium | DOP20 WMS and Hausumringe WFS open; DGM1 only as 1.09 GB XYZ zip; DOM1 only as rendered WMS; no land-use service found |
| Saarland | done 2026-11-12 (ADR 0111, `saarland-reference.md`) | medium | All services answered on 2026-11-12. DOP20, DGM1, DOM1 and ALKIS simplified are dl-de/by-2-0 by the LVGL catalogue; heights come as district zips with 1 km tiles (range requests), ALKIS from a WFS with upper-case attribute names. The line of 2026-11-08 below is kept as found then |

Recommended order: NRW, Niedersachsen, Rheinland-Pfalz, Baden-Wuerttemberg, Bremen, Bayern, Saarland.

---

## Nordrhein-Westfalen (ready)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://www.wms.nrw.de/geobasis/wms_nw_dop` layer `nw_dop_rgb` (GetMap 100 m bbox: 200 image/jpeg 64 KB). Flight date: GetFeatureInfo on `nw_dop_utm_info` returned `Bildflugdatum = '07.04.2025'`, `Bodenaufloesung = '0.10'`. Tiles: `https://www.opengeodata.nrw.de/produkte/geobasis/lusat/akt/dop/dop_jp2_f10/dop10rgbi_32_345_5675_1_nw_2025.jp2` (34 MB, JP2 + index XML) | 25832 (25833 also offered) | dl-de/zero-2-0 | native 10 cm (RGBI); JP2 only as tiles, so WMS is the easy path |
| DGM1 | yes | Tiles `https://www.opengeodata.nrw.de/produkte/geobasis/hm/dgm1_tiff/dgm1_tiff/dgm1_32_345_5675_1_nw_2022.tif` (HEAD 200, 1.78 MB, Accept-Ranges). Directory listing is XML (`.../dgm1_tiff/dgm1_tiff/`). Also WCS `https://www.wcs.nrw.de/geobasis/wcs_nw_dgm` coverage `nw_dgm` (100 m GetCoverage: 200 image/tiff LZW float32) | 25832 | dl-de/zero-2-0 | file name carries the ALS year (2022); neighbour tiles may have other years |
| Surface | yes | DOM1 tiles `.../hm/dom1_tiff/dom1_tiff/dom1_32_345_5675_1_nw_2022.tif` (listed, 2.4 MB). Also nDOM50, bDOM50, 3D laser (LAZ) folders under `/produkte/geobasis/hm/` | 25832 | dl-de/zero-2-0 | DOM1 is ALS, same year as DGM1 (flag against DOP date) |
| Buildings | yes | WFS 2.0 `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht` type `ave:GebaeudeBauwerk` (GetFeature COUNT=1 in 100 m bbox: GML 3.2, `aktualit` 2024-02-26) | 25832 default | dl-de/zero-2-0 | none |
| Land use | yes | same WFS: `ave:NutzungFlurstueck`, `ave:Nutzung` listed | 25832 | dl-de/zero-2-0 | not field-tested for `bez` values |

Licence source (WMS/WCS/WFS Capabilities `Fees`): "Es gelten die durch den IT-Planungsrat im Datenportal fuer Deutschland (GovData) veroeffentlichten einheitlichen Lizenzbedingungen 'Datenlizenz Deutschland - Zero' (https://www.govdata.de/dl-de/zero-2-0). Jede Nutzung ist ohne Einschraenkungen oder Bedingungen zulaessig." `AccessConstraints: NONE`. No attribution required.
Effort low: `alkis_wfs` reuse (identical `ave:` schema, AAA-suite like Hesse), tiles by HTTP like Brandenburg/Hesse.

## Niedersachsen (ready)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms` layer `ni_dop20` (GetMap 100 m: 200 image/jpeg 50 KB; info layer `ni_dop20_info`; GetFeatureInfo needs a valid INFO_FORMAT, text/plain/xml gave a format error, not probed further). STAC `https://dop.stac.lgln.niedersachsen.de/search?bbox=...` returned item `dop20rgbi_32_550_5802_2_ni_2025-03-06`, `datetime 2025-03-06`, COG `https://dop20-rgb.s3.eu-de.cloud-object-storage.appdomain.cloud/325505802/2025-03-06/dop20rgb_32_550_5802_2_ni_2025-03-06.tif`, `license CC-BY-4.0` | 25832 | CC BY 4.0 | flight date is in the STAC item and file name; no login |
| DGM1 | yes | STAC `https://dgm.stac.lgln.niedersachsen.de/search?collections=dgm1&bbox=...`; COG `https://dgm1.s3.eu-de.cloud-object-storage.appdomain.cloud/L1603/Vollkacheln/dgm1_32_550_5803_1_ni_2016.tif` (Range GET -> 206). WCS `https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs` coverage `ni_dgm1` (100 m: 200 image/tiff, LZW, 100x100) | 25832 | CC BY 4.0 | the sample tile is ALS 2016-04-18 (older than DOP 2025); newer re-flights exist per `los`; LGLN announced a DGM1/DOM1 refresh |
| Surface | yes | STAC `https://dom.stac.lgln.niedersachsen.de/search?...` item `dom1_32_550_5803_1_ni_2016`, COG `https://dom1.s3.eu-de.cloud-object-storage.appdomain.cloud/L1603/Vollkacheln/dom1_32_550_5803_1_ni_2016.tif`. Also bDOM20 and LAZ point clouds on the hub | 25832 | CC BY 4.0 | nDSM = DOM1 - DGM1 from the same `los`; DOP is a different, newer flight |
| Buildings | yes | WFS 2.0 `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach` (`ave:GebaeudeBauwerk`) | 25832 | CC BY 4.0 | none |
| Land use | yes | same WFS `ave:NutzungFlurstueck` (GetFeature returned `nutzart` "Sport-, Freizeit- und Erholungsflaeche", `bez` "Siedlungsgruenflaeche") | 25832 | CC BY 4.0 | none |

Licence source (WMS/WFS/WCS `AccessConstraints`): "Es gelten die Lizenzbedingungen 'Creative Commons Namensnennung - 4.0 International (CC BY 4.0)' ... Als Rechteinhaber und Bereitsteller ist 'LGLN', sowie das Jahr des Datenbezugs in Klammern anzugeben. Beispiel fuer Quellenvermerk: LGLN (2024) Creative Commons Namensnennung - 4.0 International (CC BY 4.0)". Fees: None/NONE.
Effort low-medium: WFS identical to Hesse/NRW; new part is a STAC client + COG range reads (the human-facing `opengeodata.lgln.niedersachsen.de` site is a JS app with no directory listing, so STAC is the way).

## Rheinland-Pfalz (ready)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://geo4.service24.rlp.de/wms/rp_dop20.fcgi` layer `rp_dop20` (GetMap 100 m: 200 image/jpeg 64 KB); GetFeatureInfo `rp_dop20_info` returned metadata (`erstellung = 09.07.2024`, file `dop20rgbi_32444_5538`; this is a production date, not necessarily the flight date). Tiles: `https://geobasis-rlp.de/data/dop20rgb/current/jp2/dop20rgb_32_292_5548_2_rp_2025.jp2` (2 km, 155 MB + `.j2w`) | 25832 (also 25831) | dl-de/by-2-0 | JP2 tiles are heavy; WMS is practical; confirm flight date source in `.../metadata/` |
| DGM1 | yes | `https://geobasis-rlp.de/data/dgm1/current/tif/dgm1_32_292_5548_1_rp_2025.tif` (+ `.tfw`; HEAD 200, 1.58 MB, Accept-Ranges). Directory listing is plain HTML; `meta4/` for bulk | 25832 | dl-de/by-2-0 | none |
| Surface | yes | `https://geobasis-rlp.de/data/dom1/current/tif/dom1_32_292_5548_1_rp_2025.tif` (listed); also bDOM20, LAZ (`/data/las/`) | 25832 | dl-de/by-2-0 | none |
| Buildings | yes | WFS 2.0 `https://www.geoportal.rlp.de/registry/wfs/519` (ALKIS vereinfacht): `ave:GebaeudeBauwerk` (GetFeature COUNT=1: 200, GML 3.2); OGC API Features at `https://www.geoportal.rlp.de/spatial-objects/519` | 25832 | dl-de/by-2-0 | server is not the AAA-suite (MapServer-like, `numberMatched=unknown`, paging via `next`) |
| Land use | yes | same WFS `ave:Nutzung` (no `NutzungFlurstueck`); returned `nutzart` "Strassenverkehr", `bez` "Begleitflaeche Strassenverkehr" (verge class present). Also detailed WFS 451 "ALKIS Tatsaechliche Nutzung" (AX_* types) | 25832 | dl-de/by-2-0 | type name differs from Hesse |

Licence source (open-data page `https://lvermgeo.rlp.de/geodaten-geoshop/open-data/`): "Datenlizenz Deutschland - Namensnennung - Version 2.0 ... Namensnennung ... `(c)GeoBasis-DE / LVermGeoRP<Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]`". WMS Fees text: "geldleistungsfrei, Datenlizenz Deutschland - Namensnennung - Version 2.0".
Effort medium: WFS schema is `ave:` but the server and Nutzung type differ (paging, no oid-sort assumption), tiles are plain HTTP like Brandenburg.

## Baden-Wuerttemberg (done; the findings below are of 2026-11-08 and partly superseded, see `baden-wuerttemberg-reference.md`)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C` layer `IMAGES_DOP_20_RGB` (GetMap 100 m: 200 image/jpeg 39 KB). WCS `.../wcs/WCS_INSP_BW_Orthofoto_DOP20` (caps 200). Tiles `https://opengeodata.lgl-bw.de/data/dop20/dop20rgb_32_513_5402_2_bw.zip` (HEAD 200, 253 MB per 2 km tile) | 25832 (25833 also) | dl-de/by-2-0 | tiles too large to use; flight date not found in WMS caps (not verified; `Open_GeoData` product text / tile metadata would need checking) |
| DGM1 | yes | `https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_513_5402_2_bw.zip` (HEAD 200, 13.7 MB; Range GET 206, zip method `store`). WCS `https://owsproxy.lgl-bw.de/owsproxy/wcs/WCS_INSP_BW_Hoehe_Coverage_DGM1` coverage `EL.ElevationGridCoverage` (100 m: 200 image/tiff but 16-bit, uncompressed, big-endian; not the float32 source) | 25832 | dl-de/by-2-0 | inner format not unpacked; tile index is 2 km grid (even-km names such as 513_5402 hit) |
| Surface | yes | `.../data/dom1/dom1_32_513_5402_2_bw.zip` (9.1 MB) and ready-made `.../data/ndom1/ndom1_32_513_5402_2_bw.zip` (11 MB, from bDOM - DGM) | 25832 | dl-de/by-2-0 | nDOM1 is image-based (bDOM), a leaf-on canopy |
| Buildings | partly | INSPIRE WFS `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_INSP_BW_Gebaeude_ALKIS?VERSION=2.0.0` type `bu-core2d:Building` (GetFeature: 200 GML 3.2, features present). Bulk: `.../data/hu/hu_bw.zip` (Hausumringe Shape, 515 MB) | 25832 | dl-de/by-2-0 | INSPIRE schema, not `ave:`; new parser |
| Land use | partly | Only bulk: `.../data/ln/ln_bw.zip` (GPKG, 2.39 GB; HEAD 200) and `ln_bw_shp.zip`. The ALKIS land-cover WFS `WFS_INSP_BW_Bodenbedeckung_ALKIS` is "Kostenpflichtige Nutzung" (needs a Nutzungsvereinbarung) | 25832 | LN: open (HVD list) | no open WFS for land use; workaround: ln_bw.zip (cached, range-read) or OSM |

Licence source (Capabilities): "Es gelten die Lizenzbedingungen 'Datenlizenz Deutschland - Namensnennung - Version 2.0' bzw. 'dl-de/by-2-0' ... Beispiel fuer Quellenvermerk: LGL-BW (2024) Datenlizenz Deutschland - Namensnennung - Version 2.0, www.lgl-bw.de". Fees: "Unentgeltliche Nutzung nach Open Data Lizenz". High-value dataset list (LGL, 29.05.2024): DGM1, DOM1, DOP20, Hausumringe, Landnutzung are open; bDOM free.
Effort medium-high: needs a zip-tile reader with ranges, an INSPIRE building parser, and an LN GPKG extractor.

## Bayern (partly)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://geoservices.bayern.de/od/wms/dop/v1/dop20` layer `by_dop20c` (GetMap 100 m: 200 image/jpeg 51 KB). Flight date: GetFeatureInfo `by_dop20_info` returned `ua = '24.08.2024'`. Tile `https://download1.bayernwolke.de/a/dop20/data/32690_5334.tif` (HEAD 200, 72.8 MB, 1 km) | 25832 (25833 also) | CC BY 4.0 | tile is large; WMS is practical |
| DGM1 | yes | `https://download1.bayernwolke.de/a/dgm/dgm1/520_5502.tif` (HEAD 200, 2.6 MB, 1 km; URL taken from the live Metalink `https://geodaten.bayern.de/odd/a/dgm/dgm1/meta/metalink/09.meta4`, 21 MB, ~240 GB total). XYZ variant offered | 25832 | CC BY 4.0 | Metalink index is large; polygon-to-metalink service `geoservices.bayern.de/services/poly2metalink/...` is POST-only (405 on GET) |
| Surface | yes | DOM20 GeoTIFF 30-50 MB per tile (`dom20_download_einzel` via `https://geoservices.bayern.de/od/wms/grid/v1/opendatagrid?...layers=dom20_DOM`); newsletter 03/2026: DOM20 for northern Bavaria from Bayernbefliegung 2025. LAZ laser data also open (up to 1 GB per tile) | 25832 | CC BY 4.0 | direct tile URL pattern for DOM20 not verified (my guesses returned 404); no 1 m DOM |
| Buildings | partly | Hausumringe only as Shape zip per Regierungsbezirk, e.g. `https://geodaten.bayern.de/odd/m/3/daten/hausumringe/bezirk/data/091_Oberbayern_Hausumringe.zip` (~100 MB each; URLs from `.../hausumringe/bezirk/kml/HU_regierungsbezirk.kml`). No WFS/OGC-API found (probed several guessed paths: 404) | 25832 | CC BY 4.0 | bulk only; no attributes beyond outline |
| Land use | partly | ALKIS Tatsaechliche Nutzung as GPKG `https://geodaten.bayern.de/odd/m/3/daten/tn/Nutzung_kreis.gpkg` (HEAD 200, 5.39 GB) or per-Landkreis Shape; WMS-only service `https://geoservices.bayern.de/od/wms/alkis/v1/tn` (200). Also "Landnutzung" GPKG 5.5 GB | 25832 | CC BY 4.0 | bulk only; WMS gives pictures, not features |

Licence source (portal FAQ `https://www.geodaten.bayern.de/odd/m/3/html/faq.html`, WMS `AccessConstraints: CC BY 4.0`, Fees: kostenfrei): attribution "Geobasisdaten: Bayerische Vermessungsverwaltung - www.geodaten.bayern.de (Daten veraendert), Lizenz: CC BY 4.0" (the "Daten veraendert" part when modified).
Effort high: needs a Metalink/tile index, big bulk downloads (build a pre-cut regional extract of Hausumringe and TN, as done for OSM), and no WFS. DOP date works through the WMS info layer.

## Bremen (partly)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | yes | WMS `https://geodienste.bremen.de/wms_dop20_2023` layer `DOP20_2023_HB` (GetMap at E485000 N5880000: 200 image/jpeg 50 KB) | 25832 (25833 also) | CC BY | only year 2023 verified (other years 2022/24/25 names gave 404); flight date not exposed, only the year |
| DGM1 | partly | Download `https://gdi2.geo.bremen.de/inspire/download/DGM/Gitternetz_DGM1_2017_HB_ASCII_XYZ.zip` (HEAD 200, 1.09 GB, Last-Modified 2025-07-24, XYZ ASCII, 2017); Bremerhaven 2015 service reportedly "nicht frei verfuegbar" per catalogue (not verified, catalogue hosts unreachable). WMS `https://geodienste.bremen.de/wms_dgm1` layer `DGM1_HB` is rendered only | 25832 | CC BY | single 1 GB zip, no tiles; directory listing 403 |
| Surface | partly | only rendered WMS `https://geodienste.bremen.de/wms_dom1` (caps 200; "stellt das DOM 1 von Bremen dar"). A state notice lists DOM1/DOM5, DOP10/20, LoD1/2 as high-value open datasets, but I found no download URL | 25832 | CC BY 4.0 | no raw heights found |
| Buildings | yes | WFS `https://geodienste.bremen.de/wfs_alkis_hausumringe` type `app:hausumringe` (deegree; GetFeature in 100 m bbox: 200 GML 3.2; attributes `ags`, `oi`, `gfk` only) | 25832 | CC BY | no `rellage` / underground flag (only `gfk`) |
| Land use | no | no open WFS/WMS found (about 25 guessed names under geodienste.bremen.de: 404) | - | - | workaround: OSM land use |

Licence source: WMS `Fees`: "Creative Commons Namensnennung (CC-BY), Quellenvermerk: Landesamt GeoInformation Bremen"; `AccessConstraints`: "Es gelten keine Zugriffsbeschraenkungen, Landesamt GeoInformation Bremen". Official notice (`geo.bremen.de/sixcms/media.php/13/2024-05-22_Verfuegung_BetroffeneGeobasisdaten_DVO_hvD.pdf`): high-value data (DGM1/5, DOM1/5, DOP10/20, LoD1/2, Hausumringe) under "die Lizenz Creative Commons BY 4.0".
Effort medium (small area: one 1 GB XYZ, DOP by WMS, simple WFS). Low priority; the missing DSM and land use need workarounds (no nDSM, OSM for land use).

## Saarland (blocked, unverified)

| Product | Open? | Verified service / URL | CRS | Licence | Blockers / notes |
|---|---|---|---|---|---|
| DOP | partly | WMS `https://geoportal.saarland.de/freewms/dop2025` (also `dop2022`, `dop2023`, `dop`), layer `sl_dop20_rgb` (2025) / `WMS_SL_DOP20` (2023): GetMap 100 m at E350000 N5455000: 200 image/jpeg 33-45 KB | 25832, 25833 | Capabilities: `Fees: None`, `AccessConstraints: noLimitations`; keyword "opendata"; no licence text | no flight date found; catalogue snippets say DOP20 use beyond viewing "is subject to fee and requires a contract" (not verified, catalogue unreachable) |
| DGM1 | no | only rendered WMS `https://geoportal.saarland.de/freewms/dgm` (layer `sl_dgm1_2006`, hillshade). Catalogue snippets claim DGM1 under dl-de/by-2.0 via ATOM/WCS but I could not reach them | - | unverified | `geoportal.saarland.de` main paths, `/freewfs/*`, `/freewcs/*`, `/spatial-objects/` all returned 503 ("Served by LVGL Saarland") |
| Surface | no | nothing found | - | - | - |
| Buildings | unverified | WFS paths returned 503 | - | - | host down or not public |
| Land use | unverified | as above | - | - | - |

Licence: no usable statement in Capabilities beyond "noLimitations". Re-test when the 503 clears; until then treat as OSM-only. Effort high (nothing standard; possible shop-only data).

## Method notes and gaps

- Catalogue sites (`gdk.gdi-de.org`, `inspire-geoportal.ec.europa.eu`) were unreachable from this host (connection reset / expired certificate), so some Saarland and Bremen details come from web-search snippets and are marked unverified above.
- Flight dates: verified for NRW (GFI `Bildflugdatum`), Niedersachsen (STAC `datetime`), Bayern (GFI `ua`). Not found for BW, HB, SL; RLP only a production date.
- NRW/NI/RLP/BW/BY DGM1 vintage is not the DOP vintage (e.g. NI sample DGM/DOM 2016 vs DOP 2025): the nDSM adapter must flag the date gap as it does for Hesse.
- No data were downloaded beyond tiny test responses (largest file: the 21 MB Bayern Metalink, deleted).
