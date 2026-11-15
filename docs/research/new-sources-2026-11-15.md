# New data sources not yet used (survey of 2026-11-15)

Three research passes (city and municipal data; water and forest; terrain, history, federal data) against what freemapper already reads: ALKIS (15 states), Basis-DLM (HE, MV, BY, RP, SH, ST), OSM extracts, DGM/DOM/orthophoto, point clouds, tree cadastres of 10 cities, permit sources. Services were queried anonymously with the user agent `freemapper-research (https://github.com/HenryJobst/freemapper)`; no personal data.

**Status words.** *fetched by me* = I read the answer myself. *fetched by sub-pass* = a research agent read it, I did not repeat it. *web only* = catalogue or web page. *unclear* = service unreachable or contradictory. Test boxes were small; hit counts are samples, not completeness.

Already decided the same day: Copernicus HRL (not now, idea for Thuringia, see memory `copernicus-hrl`), OpenFreeMap, Natural Earth, ESA MIRAVI and Esri World Imagery rejected.

## Ranking (gain for the maps against effort)

| # | Source | Where | Gain | Effort | Note |
|---|---|---|---|---|---|
| 1 | Forest road network with skidding lanes (`Forstwirtschaft:Forstwegenetz`, WFS, GeoJSON, `WKLASSE`, `WEGTYP`) plus geotopes and spoil heaps on the same host | SL | high for forest maps | low | Fetched by me (GetFeature with `SRSNAME=EPSG:25832`). Only open set with skidding lanes (OSM lacks them). Licence: capabilities say "no conditions", catalogue says CC BY 4.0 (web only). |
| 2 | Tree cadastres with crown diameter through the existing `wfs-geojson` format: Freiburg, Aachen, Krefeld (KRZN), Münster (genus only) | city adapters | high (symbols 417/418) | manifest only | Freiburg 46,048 trees, dl-de/by-2-0. Aachen 121,561, GovData dl-by-de/2.0 but the service says NONE. KRZN dl-zero. |
| 3 | New reader `wfs-gml` (WFS 2.0 with GML, paging): Rostock, Dresden, Bielefeld, Brandenburg municipalities | cities | medium to high | low to medium | Rostock bundles trees, benches, bins, lights, playgrounds (CC0). About 80 % of the found services fit `wfs-geojson` plus `wfs-gml`. |
| 4 | Forest stands with species and age: Berlin `ua_forstbetriebskarte_2014` (WFS, dl-de/zero), Thuringia `BAUMARTEN.zip` (90 MB, CC BY 4.0, whole forest), Baden-Württemberg ForstBW stands (state forest only, dl-de/by-2-0), Saarland state forest stands (no age) | BE, TH, BW, SL | high where no point cloud (BW, TH, BE) | medium | Feeds the vegetation layer and the green symbols 405–415. Brandenburg only as WMS with feature info. |
| 5 | DLR tree species and canopy height (`geoservice.dlr.de/eoc/land/wms`/`wcs`, CC BY 4.0, 10 m) | nationwide | medium for states without point cloud | low | Fetched by sub-pass and by me earlier for the Copernicus layers only; the DLR layer itself is sub-pass. One service, one licence. |
| 6 | Street space with surfaces: Hamburg Feinkartierung Straße (`HH_WFS_Feinkartierung_Strasse`), Berlin Straßenbefahrung, Aachen `logo_flaechen`, Kiel `Strassenbestandsflaechen` | HH, BE, AC, KI | medium for sprint (501, kerbs, rails) | medium | ALKIS street areas lack surface and green-strip separation in Berlin and Saxony. Hamburg data are from 2019. |
| 7 | Ditches and water networks with permanence and pipes: SN LfULG (`ROHR_ERL`, `WASSERF_ERL`), NI Hydro layer 29, MV LUNG, SH DAV, BW AWGN, SL and TH INSPIRE Hydro | several | medium for forest maps (305/306) | low per state | Bremen Gewässerkataster is CC BY-ND: clarify before use. |
| 8 | Moor type and thickness: BB Moorbodenkarte (not for scales larger than 1:5000), NRW IS Moor, HH Moorkartierung, BE Moore, ST LAGB, SH moorboden | BB, NRW, HH, BE, ST, SH | medium (308–313) | low to medium | More precise than AX_Moor. |
| 9 | Historic maps and aerials as templates: NRW (Tranchot, Uraufnahme, Neuaufnahme, TK25 1936–45, hist. DOP; dl-de/zero for Tranchot), BB/BE (DR25, Schmettau, DOP 1953/1992–2009), RP, BY, HE, HB | many | template only | low (WMS) | Template is passed on with the map: each licence needs a look. SH Chronologen: capabilities say CC BY 4.0, web page says private use only. |
| 10 | NRW geotopes (3155 polygons) and spring cadastre (21,575 points) | NRW | medium (rock, boulder, spring) | low (OGC API) | Geotope type only in the name. Geotopes fetched by me. |
| 11 | `left_out = [14]` in the Saarland (and probably NRW) point-cloud reader | SL | small | very small | Class 14 is most likely power line wire (pattern in my measurement, not documented). It would count as high vegetation today. |
| 12 | Schoolyard furniture (benches, bins, lights, play equipment): Rostock, Dortmund, Freiburg, Chemnitz | cities | schoolyard only | low once 3 stands | OSM covers most schoolyards. |

## Facts per topic

### City and municipal data
- Tree cadastres with crown: Freiburg `geoportal.freiburg.de/wfs/gut_pit/gut_pit` (fetched by me), Aachen `geoserver.aachen.de/opendata/ows` (fetched by me), Krefeld `geoservices.krzn.de/…/wfs_verb_baum` (sub-pass), Dresden `kommisdd.dresden.de/…` (sub-pass, service says none, catalogue dl-by-de/2.0), Rostock `geo.sv.rostock.de/geodienste/baeume/wfs` (CC0, sub-pass), Bielefeld (cc-by/4.0, fetched by me), Duisburg ArcGIS (CC BY 3.0 DE).
- Not usable: Bochum ("andere"), Bonn (licences contradict, service gave 502/504), Karlsruhe has no crown, Münster genus only, Bremen/Hannover/Stuttgart/Nuremberg/Munich nothing usable.
- Furniture: Rostock, Dortmund, Freiburg, Chemnitz, Berlin (`gruenanlagen`, bicycle racks, dl-de/zero).
- OSM covers trees, fences, walls, stairs, hedges, benches, lights; the gaps are crown diameter (trees) and surface (streets). For sprint the OSM census rates benches, bins, bollards, lights as no ISSprOM symbol.
- Federal: BKG TopPlusOpen (WMS, dl-de/by-2-0, fetched by me, generalised raster, reference only); DLM250 1:250,000 unusable; DGM1/DTK25/DTK50 need a UUID (403); LoD2-DE no service; Overture ODbL, adds nothing over OSM/ALKIS; Mapillary needs a token.
- Licence doubts before any adapter: Aachen, Dresden, Frankfurt (Oder) Stadtkarte (terms not read), Brandenburg municipalities (none stated), Duisburg and Karlsruhe (none), Münster, Bonn, Bochum.
- Not reached: Hamburg `api.hamburg.de` timed out (the `geodienste.hamburg.de` WFS works); many `opendata.*` hosts of other cities unreachable, which is no proof that nothing exists.

### Water and forest
- Stands and species: see ranking 4; Brandenburg Waldbestandeskarte (`brandenburg-forst.de/ogc/wb`, WMS with feature info only), Saxony Waldbiotope (WMS) and BTLNK 2005 (REST, dl-de/by-2-0), Thuringia Standort and Waldbiotope (CC BY 4.0), NRW FSK5 (WFS, dl-de/by-2-0).
- No open stand, site or road source found: HE, ST, NI, RP, BY (RP forest host answered 403).
- Moor and ditches: see ranking 7 and 8.
- Contradiction to check: the sub-pass says the Saarland Basis-DLM is offered as a Shape ZIP under dl-de/by-2-0 (`…/OD_BDLM_shp_SL/BDLM-SHP-EPSG-25832.zip`, 84.5 MB, directory read). The repo's Saarland reference does not say the Basis-DLM is paid, only that DGM1 2006 and some WCS are. Treat as open question, not a finding.

### Terrain, history, point-cloud classes
- Measured by me on cached tiles (3 per state): BB 1, 2, 20; BE 2, 3, 4, 5, 7; NRW 1, 2, 9, 17, 18, 20, 24, 26; BY 2, 6, 7, 20, 22; RP only 2 and 20; SL 1, 2, 6, 8, 9, 14, 17, 20, 24, 30. Only Saarland has a plausible wire class (14, 399 points along a 200 m line, first to fourth echoes). Powerlines and masts come from OSM today (ADR 0083); AX_Leitung from the Basis-DLM stays the open follow-up.
- Unknown class tables: HE, SH, HH, HB; names without codes: MV, ST, BW (BW newsletter mentions wire and bridge classes).
- Geotopes and springs: SL, NRW, HH Geotourismus (small), BW LGRB `ms:gtk` (type attribute not retrievable), ST mining. Nothing usable found in BE, MV, HB, TH.
- Footpaths and cycle routes: nothing beyond OSM except surface and width (HH, BW).

## Not reached
- Thuringia (except forest stands and INSPIRE Hydro), MV geotopes and moors, Berlin geotopes, the GDI-DE catalogue.
- Licence texts read only from web pages: Saarland forest roads (catalogue CC BY 4.0), BW RadVIS, BW Geotopkataster, Thünen moor map.
- A 78 MB Bavarian hiking-trail ZIP was downloaded by a sub-pass by mistake (larger than the few MB the brief allowed).
