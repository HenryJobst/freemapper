# Extra open-data survey for freemapper sprint maps: BB, SN, ST, HH, HE, SH, MV, NW, NI, RP

Checked live on 2026-11-08 (curl and Python urllib: GetCapabilities plus small GetFeature/items requests in each
state's freemapper test area; HEAD or 4-byte range reads for files; no bulk downloads). Berlin is out of scope.
freemapper already uses, per state: DOP (+date), DGM1, DOM1/nDSM, ALKIS vereinfacht buildings + land use, OSM candidates.
This survey looks for sources that add sprint content (ISSprOM 2019-2): walls 513/515, fences 516/518, stairs 532,
canopies 522, passages, hedges 410.1, distinctive trees 417/418, small watercourses 304-306, earth banks 104/105,
bridges 512, fountains 311, monuments 526.
Probe responses (untrusted) are in `probe/` next to this file, the request log is `probe/_log.txt`, the helper scripts
are in `scripts/`.

## Key findings

1. **Passages and canopies need no new source.** ALKIS vereinfacht `GebaeudeBauwerk`, which freemapper already
   fetches, contains `Bauteil` objects with `funktion` "Durchfahrt im Gebäude", "Durchfahrt an überbautem Verkehrsweg",
   "Arkade", "Auskragender Geschossteil" and `Sonstiges Bauwerk …` objects with "Überdachung" (the reference docs count
   e.g. RP 1,225 Durchfahrt + 1,775 Überdachung, NI 1,864 Überdachung, MV 398 Durchfahrt + 562 Überdachung). Today
   `build/buildings.rs` keeps only `Gebäude`. A rule change gives passages and 522 canopies in every state.
2. **Full ALKIS (NAS-konform WFS) is open in BB, HE, MV, NW and NI**, all with the same AdV `adv:` schema, the same
   AAA-suite server family and nationwide AdV code lists. It carries the surveyed (cm-level) lines a sprint map needs:
   `AX_SonstigesBauwerkOderSonstigeEinrichtung` (Mauer 1700-1703, Stützmauer 1720-1723, Zaun 1740, Treppe 1620/1621,
   Treppenunterkante 1630, Überdachung 1610/1611, Rampe 1650, Denkmal 1750, Brunnen 1780-1783), `AX_Vegetationsmerkmal`
   (Laubbaum 1012/Nadelbaum 1011 points, Hecke 1100-1103, Baumreihe 1210-1230), `AX_Bauteil` (Durchfahrt 2610/2620,
   Arkade 2400, auskragend 2510), `AX_BauwerkImVerkehrsbereich` (Brücke 1800, Steg 1820, Tunnel 1870),
   `AX_UntergeordnetesGewaesser` (Graben 1010-1013, Bach 1030), `AX_WegPfadSteig`, `AX_Strukturlinie3D`
   (Oberkante 1220 / Unterkante 1230 → earth bank). How much a state actually captures differs a lot (see per-state
   samples): MV and BB are rich, NW has single trees, HE and NI mostly canopies and building parts. Saxony's NAS WFS
   is open but holds no topographic lines at all.
3. **ATKIS Basis-DLM is open in all ten states**, and the AdV publishes it for every state as one uniform GeoPackage
   per state on basemap.de (`basisviews_bdlm_<LAND>_EPSG4326_<date>.gpkg`, refreshed daily, R-tree indexed, HTTP
   Range supported). Basis-DLM is a 1:10,000 model (±3 m) and sparse in towns (Lübeck old town: 6 tree rows; Marburg:
   1 single tree; Halle centre: no vegetation lines), so it is a fallback for water axes (with width and "nicht ständig
   Wasser führend" → 304/305/306), embankments/dams, tree rows and hedges outside the ALKIS-rich states.
4. **LoD2 is open everywhere but adds little.** LoD2-DE buildings are footprint prisms with standard roofs; passages
   and canopies are not modelled (Hamburg sample: only Wall/Roof/GroundSurface). Useful only for heights (e.g. towers
   524/525), which the nDSM already gives. Not recommended.
5. **Tree cadastres** (positions, often crown diameter) are open for Leipzig, Dresden, Potsdam, Frankfurt (Oder),
   Hamburg, Frankfurt am Main, Kiel, Lübeck, Köln, Dortmund, Münster, Rostock. They are the best 417/418 source in
   street space. Not found for Hannover, Mainz, Koblenz, Halle, Magdeburg (no verifiable open endpoint).
6. **Hamburg** has a unique detailed street and park inventory (Feinkartierung Straße/Grün via OGC API): curbs,
   railings, fences, bollards, sidewalk/cycle-way/parking polygons with surface material, park paths, stairs and walls.

## Summary table (state × source)

Legend: ✔ open and verified live; (✔) open, listed/indexed but not sampled; ✗ not open / not found. Effort is for
freemapper: low = reuse an existing reader with a new rule or URL, medium = one new reader shared by several states,
high = bulk download or a new format just for this state.

| State | Full ALKIS (NAS WFS) | Basis-DLM | LoD2 | Tree cadastre (cities) | Water network | State-specific | Top picks (benefit/effort) |
|---|---|---|---|---|---|---|---|
| Brandenburg | ✔ `isk.geobasis-bb.de/ows/alkis_nas_wfs`, rich | ✔ GPKG 598 MB; INSPIRE hy-n/hy-p/tn WFS; NAS/Shape via Geobroker | ✔ 1 km CityGML zips + INSPIRE WFS | ✔ Potsdam (284k pts), Frankfurt (Oder) WFS (36.7k) | ✔ INSPIRE hy-p `Watercourse` (Basis-DLM) | — | ALKIS NAS WFS; vereinfacht passages/canopies; Potsdam trees |
| Saxony | ✔ open but no topography (land use, buildings only) | ✔ GPKG 879 MB; NAS/Shape zip 1.1-1.2 GB (geocloud) | (✔) 2 km CityGML index (tile host 503 today) | ✔ Leipzig WFS (187.6k), Dresden WFS | ✔ INSPIRE hy-p WFS | — | Basis-DLM GPKG; Leipzig/Dresden trees; vereinfacht passages/canopies |
| Saxony-Anhalt | ✗ only vereinfacht | ✔ NAS WFS `ST_LVermGeo_ATKIS_WFS_BDLM_OpenData`; Shape zip | ✔ simplified WFS (footprint+height); tiles via map downloader | ✗ not found | ✔ INSPIRE hy-p/hy-n WFS (listed) | — | Basis-DLM NAS WFS; vereinfacht passages/canopies |
| Hamburg | ✗ (WMS only) | ✔ GPKG 59 MB | ✔ OGC API CityJSON `lod2_hamburg` | ✔ Straßenbaumkataster OGC API/WFS (crown Ø) | ✔ Basis-Gewässernetz WFS (listed) | ✔ Feinkartierung Straße + Grün (OGC API) | Feinkartierung Straße/Grün; Straßenbaumkataster; Basis-DLM GPKG |
| Hesse | ✔ `gds.hessen.de/…/alkis/nas/wfs` | ✔ NAS WFS `…/atkis-bdlm/nas/wfs` (+ OGC API 694); GPKG 801 MB | (✔) INSPIRE WFS 831 | ✔ Frankfurt am Main CSV | (✔) INSPIRE hy-p ATKIS | — | ALKIS NAS + Basis-DLM NAS (same host); Frankfurt trees |
| Schleswig-Holstein | (✔) per-Flur GML download only | ✔ NAS WFS `WFS_SH_ATKIS_BDLM_SF_OpenGBD`; GPKG (stale: 2025-11-03) | (✔) Massendownload GeoJSON index | ✔ Kiel WFS, Lübeck CSV | ✔ INSPIRE hy-n/hy-p ATKIS + ALKIS WFS (listed) | (✔) Knicks/biotope lines, INVEKOS landscape elements | Basis-DLM WFS; Kiel/Lübeck trees |
| Mecklenburg-Vorpommern | ✔ `geodaten-mv.de/dienste/alkis_wfs_nas`, very rich | ✔ NAS WFS `atkis_bdlm_wfs_nas`; GPKG 367 MB | (✔) free download portal, ATOM | (✔) Rostock CSV/JSON (cc-zero; host unreachable from here) | via Basis-DLM | — | ALKIS NAS WFS; Basis-DLM NAS WFS; Rostock trees |
| North Rhine-Westphalia | ✔ `wfs_nw_alkis_aaa-modell-basiert` (single trees!) + GRU NAS per Kreis | ✔ NAS zip 1.19 GB, Shape 4.7 GB; GPKG 2.1 GB | ✔ 1 km CityGML tiles (35k) + OGC API `3dg` | ✔ Köln WFS, Dortmund, Münster | via Basis-DLM | ABK (raster only), INVEKOS landscape elements OGC API | ALKIS AAA WFS; Köln/Dortmund/Münster trees |
| Lower Saxony | ✔ `alkis_wfs_nas` (canopies, building parts, bridges; no walls) | ✔ FileGDB/Shape on LGLN hub; GPKG 1.4 GB | (✔) LoD2 index on LGLN hub | ✗ Hannover not found | via Basis-DLM | — | ALKIS NAS WFS; Basis-DLM GPKG |
| Rhineland-Palatinate | ✗ (vereinfacht + TN-Einzellayer only) | ✔ Shape-schema WFS `basisdlm_shape_rp.fcgi`; Shape zip 336 MB; GPKG 1.0 GB | ✔ 2 km CityGML tiles (unzipped, up to ~100 MB) | ✗ not found | via Basis-DLM | — | vereinfacht passages/canopies; Basis-DLM |

---

## Brandenburg

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes | WFS 2.0 `https://isk.geobasis-bb.de/ows/alkis_nas_wfs` "WFS BB ALKIS NAS-konform", 139 types (GID 7.1, `adv:`). Sanssouci bbox `366200,5807100,367100,5807700` (EPSG:25833): `AX_SonstigesBauwerkOderSonstigeEinrichtung` 295 (Zaun 1740 ×136, Mauerkante 1701/1702 ×153, Überdachung 1610 ×6), `AX_Vegetationsmerkmal` 96 (all Hecke 1100, `gml:Curve`), `AX_Bauteil` 35 (Durchfahrt 2610 ×34), `AX_BauwerkImVerkehrsbereich` 4 (Brücke), `AX_BauwerkImGewaesserbereich` 1. 337 KB for the walls/fences response, 0.4 s. Strausberg edge bbox: 4 wall features. | 25833 (also 25832, 4258 …) | dl-de/by-2-0, "© GeoBasis-DE/LGB, dl-de/by-2-0, (Daten geändert)" | numberMatched given; same text as the existing LGB products. Bulk alternative: NAS/Shape per Kreis `https://data.geobasis-bb.de/geobasis/daten/alkis/Vektordaten/{nas,shape}/` (44-236 MB, 2026-10-01) |
| Basis-DLM | yes | GPKG `https://basemap.de/dienste/opendata/basisviews/basisviews_bdlm_BB_EPSG4326_2026-11-08.gpkg` (598 MB, Range OK, data 07.11.2026). INSPIRE WFS from Basis-DLM: `https://inspire.brandenburg.de/services/hy-n_bdlm_wfs` (`hy-n:WatercourseLink`, 7 links in a 1 km Strausberg box, geometry + name only), `…/hy-p_bdlm_wfs` (`hy-p:Watercourse` with `origin`, width range 0-3 m; `Embankment` 0 in test box), `…/tn_bdlm_wfs` (40 transport types). NAS/Shape order via Geobroker (free, "Selbstentnahme") | 25833 | dl-de/by-2-0 (Geobroker product page, WFS AccessConstraints) | no plain-HTTP Basis-DLM folder on `data.geobasis-bb.de`; a guessed `isk.geobasis-bb.de/ows/dlm_nas_wfs` is 404 |
| LoD2 | yes | `https://data.geobasis-bb.de/geobasis/daten/3d_gebaeude/lod2_gml/lod2_33366-5807.zip` (1.99 MB, Last-Modified 2026-01-22; 1 km tiles, 19,299 listed); INSPIRE `https://inspire.brandenburg.de/services/bu-core3d_lod2_wfs` | 25833 | dl-de/by-2-0 | currency PDF `…/information/aktualitaeten/bb_3d-gebaeude_lod2_aktualitaet.pdf` |
| Trees | yes | Potsdam (city map points): `https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/baeume-{1,2,3}-stadtkarte-potsdam/records` (100,000 + 100,000 + 83,772; `ggartbez` Laub-/Nadelbaum, `pktx/pkty` 25833, `datum` survey date). Frankfurt (Oder): WFS `https://geoportal.frankfurt-oder.de/wss/service/WFS_Baumkataster/guest` type `ffo:Baeume` (36,658; crown Ø `KR_DURCHM`, height, species). Smaller: Fürstenwalde, Beelitz, Trebbin, Nuthetal, Kloster Lehnin WFS (GovData) | 25833 | Potsdam dl-de/zero-2-0; Frankfurt (Oder) dl-de/by-2-0 "Quelle: © GeoDaten-KVA / Stadt Frankfurt (Oder), dl-de/by-2-0" | Cottbus not found |
| Water network | yes | INSPIRE hy-p/hy-n above (from Basis-DLM); LfU Gewässernetz WFS not found | 25833 | dl-de/by-2-0 | ALKIS `AX_UntergeordnetesGewaesser` is in the NAS WFS (0 in both test boxes) |

**Recommendation:** (1) ALKIS NAS WFS: walls, fences, hedges, canopies, passages, bridges straight from the cadastre
(medium effort once; shared reader). (2) ALKIS vereinfacht Bauteil/Überdachung rule (low). (3) Potsdam tree points for
417/418 candidates (low, city-limited). Basis-DLM GPKG for water axes outside towns.

## Saxony

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes, but no topography | WFS `https://geodienste.sachsen.de/aaa/public_alkis/nas/wfs` "WFS SN ALKIS NAS-konform" (124 types, GID 6.0). A type scan in Dresden (`410000,5654000,414000,5658000`, EPSG:25833) returned features only for parcels, buildings, land use, `AX_Weg`, `AX_Gehoelz`; 0 for `AX_SonstigesBauwerk…`, `AX_Vegetationsmerkmal`, `AX_Bauteil`, `AX_BauwerkImVerkehrsbereich`, `AX_UntergeordnetesGewaesser` (also 0 in Bautzen) | 25833 | dl-de/by-2-0, "Quelle: GeoSN" | Saxony's cadastre does not carry these objects; no gain over vereinfacht |
| Basis-DLM | yes | NAS `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/DtPWngtLEJP8K3k/basisdlm_sn_nas.zip` (page: 1130 MB) and Shape `…/basisdlm_sn_shape.zip` (1,274,435,077 B, Range 206, Last-Modified 2026-11-04; quarterly); page `https://www.geodaten.sachsen.de/downloadbereich-basis-dlm-4168.html`. GPKG `…/basisviews_bdlm_SN_EPSG4326_2026-11-08.gpkg` (879 MB, data 26.10.2026). INSPIRE hy-p WFS `https://geodienste.sachsen.de/aaa/public_inspire/atkis-bdlm/hy-p/dls/wfs` (50 `Watercourse` in the Dresden box, `origin natural`, width ≤3 m) | 25833 | dl-de/by-2-0 ("Quelle: Geodaten Sachsen") | no full Basis-DLM WFS found (guessed paths 404); HEAD on geocloud returns 401, GET/Range works |
| LoD2 | yes | Index `https://geodienste.sachsen.de/ags-relay/ArcGISServer/guest/arcgis/rest/services/geosn/rest_geosn_downloadlinks/MapServer/3/query` → `https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/GVzwbSyp7Yl7mBD/lod2_33414_5654_2_sn_citygml.zip` (2 km; `Stand` "2023 (LSC 2016, Basis-DLM 2020, DGM 2016)"); also DXF/Shape | 25833 | dl-de/by-2-0 | the tile host answered 503 twice today |
| Trees | yes | Leipzig WFS `https://geodienste.leipzig.de/l3/OpenData/Baeume/wfs` type `OpenData:Baeume` (187,559; species, planting year, trunk Ø, sometimes crown Ø; `fme_tstamp` 2026-11-06; GeoJSON output works). Dresden WFS `https://kommisdd.dresden.de/net3/public/ogc.ashx?NodeId=1633&Service=WFS` type `cls:L1261` "Stadtbäume" | 25833 | Leipzig dl-de/by-2-0 ("Stadt Leipzig"); Dresden dl-de/by-2-0 (GovData), Capabilities Fees "none" | Dresden returns `numberMatched=unknown`; Chemnitz not found |
| Water network | yes | INSPIRE hy-p above; WMS `iwms_gsz_gewaessernetz` (rendered) | 25833 | dl-de/by-2-0 | |

**Recommendation:** (1) Basis-DLM from the GPKG (or the Shape zip) for water axes, tree rows, hedges, dams (medium,
shared). (2) Leipzig and Dresden tree WFS (low). (3) vereinfacht Bauteil/Überdachung rule (low).

## Saxony-Anhalt

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | no | only `ST_LVermGeo_ALKIS_WFS_OpenData` (vereinfacht: Flurstueck, GebaeudeBauwerk, Nutzung) | 25832 | dl-de/by-2-0 | |
| Basis-DLM | yes | WFS 2.0 `https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ATKIS_WFS_BDLM_OpenData/guest` "ATKIS-BDLM NAS" (84 types, `adv:`). Halle box `705800,5706300,707000,5707500` (EPSG:25832): `AX_WegPfadSteig` 25 (Fußweg 1103 ×23), `AX_BauwerkImVerkehrsbereich` 5 (Durchfahrt 1900, Hochstraße 1830, Tunnel 1870), `AX_SonstigesBauwerk…` 7 (Denkmal 1750), `AX_Vegetationsmerkmal` 0. Shape zip `https://www.geodatenportal.sachsen-anhalt.de/gfds_webshare/download/LVermGeo/Geodatenportal/externedaten/Basis-DLM_20260712_shp.zip` (GovData). GPKG 637 MB (data 03.11.2026) | 25832 (and 25833, 4258 …) | dl-de/by-2-0, "© GeoBasis-DE / LVermGeo ST" (Fees text still cites the Kostenverordnung, as for the other open WFS) | INSPIRE `ows_INSPIRE_LVermGeo_ATKIS_HY-P_WFS` / `HY-N_WFS` listed |
| LoD2 | yes | WFS `https://www.geodatenportal.sachsen-anhalt.de/ows_ST_LVermGeo_LoD2_WFS` types `ALKIS_LOD2_BU:BU.Building`, `BU.BuildingPart` (footprint MultiSurface in EPSG:4258 + `ELEVATION`, `HEIGHTABOVEGROUND`; a 25832 bbox returned 0, no bbox returned features). CityGML via map downloader `…/gfds/de/mod/4,1965,501/ajax/1/prepare/?` (max 5 tiles, no Range) | 4258/25832 | dl-de/by-2-0 | not CityGML via WFS |
| Trees | not found | no Halle/Magdeburg endpoint on GovData; a 2023 survey says both cities publish tree data (unverified) | | | |

**Recommendation:** (1) Basis-DLM NAS WFS (shared NAS reader; low-medium). (2) vereinfacht Bauteil/Überdachung rule
(low). LoD2 and trees: skip.

## Hamburg

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Feinkartierung Straße | yes | OGC API `https://api.hamburg.de/datasets/v1/feinkartierung_strasse/collections/{linien,punkte,strassenflaechen}/items`. Eppendorf box `564950,5937650,565350,5937950` (EPSG:25832): `linien` 128 (Bordstein 103, Geländer 15, Rinne 9, Zaun 1), `punkte` 335 (Poller 140, Bügel 134, Lampe 41, Ampel 15, Bank 2), `strassenflaechen` 553 polygons with `nutzung` (Gehweg 241, Grünfläche 112, Parkplätze 93, Radweg 41, Fahrbahn 19 …) and surface `inhalt` (Platten, Asphalt, Natursteinpflaster, **wassergebunden** 49, Baumscheibe …) | 25832 (CRS84/4326/3857) | dl-de/by-2-0, "Freie und Hansestadt Hamburg, Behörde für Verkehr und Mobilitätswende" | covers district and main roads; `stand` 2019-2026 per object |
| Feinkartierung Grün | yes | `…/feinkartierung_gruen/collections/{flaechenobjekte,linienobjekte,punktobjekte}/items`: same box: 27 areas (Wege, Treppenanlage, Mauer freistehend, Strauchflächen, Steg), 3 lines (Zäune, Handläufe), 1 point | 25832 | dl-de/zero-2-0 | public parks and playgrounds only |
| Trees | yes | OGC API `…/strassenbaumkataster/collections/strassenbaumkataster_hh/items` (133 in the box; `kronendurchmesser` 5-20 m, species); WFS `https://geodienste.hamburg.de/HH_WFS_Strassenbaumkataster`; port: `strassenbaumkataster_hafen` | 25832 | dl-de/by-2-0, "Freie und Hansestadt Hamburg, Behörde für Umwelt, Klima, Energie und Agrarwirtschaft" | street trees only |
| LoD2 | yes | OGC API `…/lod2_hamburg/collections/building/items?f=cityjson&bbox=…&bbox-crs=…25832` (12 buildings, 26 KB, 5 s; WallSurface/RoofSurface/GroundSurface, `measuredHeight`, `roofType`) | 25832 | dl-de/by-2-0 (LGV; not read from the API) | no passages/canopies |
| Basis-DLM | yes | GPKG `…/basisviews_bdlm_HH_EPSG4326_2026-11-08.gpkg` (59 MB, data 31.10.2026); INSPIRE HH Verkehrsnetze/Gebäude ATKIS WFS (Transparenzportal) | 4326 | dl-de/by-2-0 | no Basis-DLM WFS found (guesses 404) |
| Full ALKIS | no | only WMS "ALKIS - ausgewählte Daten"; guessed NAS WFS names 404 | | | |
| Water / ditches | yes | "Basis-Gewässernetz für Hamburg" WFS (Transparenzportal, dl-de/by-2-0, not sampled); district sets `strassengraeben_eimsbuettel`, `graeben_maehflaechen_hh_mitte`, `gewaesser_2_eimsbuettel`, `trockenfallende_gewaesser`, hedges `hecken_mitte` (OGC API) | 25832 | dl-de/by-2-0 | district coverage only |

**Recommendation:** (1) Feinkartierung Straße + Grün: railings/fences (516/518), park walls and stairs (513/532),
unpaved vs paved surfaces, bollards (medium: one OGC API GeoJSON reader, freemapper's LGB reader already pages
`next` links). (2) Straßenbaumkataster with crown diameter (low). (3) Basis-DLM GPKG for water axes (shared).

## Hesse

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes | WFS `https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/nas/wfs` "WFS HE ALKIS NAS-konform" (138 types). Marburg box `476359,5552444,476759,5552744` (EPSG:25832): `AX_SonstigesBauwerk…` 9 (Freitreppe 1621 ×4, Überdachung 1610 ×4, Rampe 1650), `AX_Bauteil` 29 (auskragend 2510, Tiefgarage 2100, Arkade 2400, Durchfahrt 2610), `AX_BesondereGebaeudelinie` 206, `AX_Vegetationsmerkmal` 0 | 25832, 25833 | "Jede Nutzung … ohne Einschränkung oder Bedingung" (= dl-de/zero-2-0) | `numberMatched=unknown`; same host as the vereinfacht WFS in use |
| Basis-DLM | yes | WFS `https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/atkis-bdlm/nas/wfs` (registry `https://www.geoportal.hessen.de/registry/wfs/695`, 112 types; OGC API `https://www.geoportal.hessen.de/spatial-objects/694`). Marburg 1.5 km box: `AX_Vegetationsmerkmal` 17 (Gehölz, Gras, 1 Nadelbaum), `AX_WegPfadSteig` 32, `AX_BauwerkImVerkehrsbereich` 6 (Steg, Tunnel, Brücke), `AX_SonstigesBauwerk…` 2 (Mauer, Denkmal). GPKG 801 MB (data 05.11.2026) | 25832 | dl-de/zero-2-0 (same text) | `AX_BoeschungKliff` has no geometry property (exception) |
| LoD2 | yes | INSPIRE WFS `https://www.geoportal.hessen.de/registry/wfs/831` "INSPIRE-WFS HE Gebäude - 3D LoD2" (caps 200); CKAN `3D-Gebäudemodell HE LoD2` | 25832 | WFS dl-de/zero-2-0; CKAN dataset dl-de/by-2-0 | not sampled |
| Trees | yes | Frankfurt am Main CSV `https://offenedaten.frankfurt.de/dcat/dataset/de-he-frankfurtam-baumkataster_op/content.csv` (`ETRS 89 Hochwert/Rechtswert`, crown Ø, height, species); WMS `https://geowebdienste.frankfurt.de/Baumkataster` (no WFS) | 25832 | dl-de/by-2-0 | Wiesbaden, Darmstadt, Kassel, Marburg not found |
| Water | yes | Basis-DLM / INSPIRE HE Hydro physische Gewässer ATKIS (CKAN) | 25832 | dl-de/zero-2-0 | |

**Recommendation:** (1) ALKIS NAS + Basis-DLM NAS WFS from the same host (shared NAS reader; stairs, canopies,
building parts, footpaths). (2) Frankfurt tree CSV (low, one city). (3) vereinfacht Bauteil rule (low).

## Schleswig-Holstein

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Basis-DLM | yes | WFS 2.0 `https://dienste.gdi-sh.de/WFS_SH_ATKIS_BDLM_SF_OpenGBD` (112 types, `adv:` GID 7.1). Lübeck box `610200,5969600,611400,5970800` (EPSG:25832): `AX_WegPfadSteig` 129, `AX_SonstigesBauwerk…` 10 (Spundwand 1790 ×8, Treppe, Denkmal), `AX_BauwerkImVerkehrsbereich` 10 (Brücke ×8), `AX_Vegetationsmerkmal` 6 (Baumreihe 1210), `AX_Gewaesserachse` 1. GPKG `…_SH_…gpkg` 586 MB but data date **03.11.2025** | 25832 | CC BY 4.0, "© GeoBasis-DE/LVermGeo SH/CC BY 4.0" | prefer the WFS (fresher than the GPKG) |
| Full ALKIS | partly | CKAN "ALKIS Schleswig-Holstein (ohne Eigentümerangaben)": GML per Flur via `https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload/single.php?file=ALKIS_SH_Massendownload.geojson&id=4` (16,885 Flur entries, incl. "Bauwerke und Einrichtungen", "Relief"); no NAS WFS (only `WFS_SH_ALKIS_vereinf_OpenGBD`, `…_VWG_…`) | 25832 | CC BY 4.0 | per-Flur zips; the portal certificate chain fails without `-k` (known from the reference doc) |
| LoD2 | yes | index `…/dladownload/single.php?file=LOD2_SH_Massendownload.geojson&id=4` (page `dl-lod2.html`) | 25832 | CC BY 4.0 | not sampled |
| Trees | yes | Kiel WFS `https://ims.kiel.de/geodatenextern/services/Stadtplan/LHKielWmsWfs/MapServer/WFSServer` type `Baeume` (crown Ø, species); Lübeck CSV `https://opendata.luebeck.de/bereich/5.660/stadtgruen/baumkataster/baumkataster.csv` (25832 coords, species, year); Norderstedt WFS `https://geoservice.norderstedt.de/geoserver/gru/wfs` (`gru:gruen_baum`), Elmshorn | 25832 | Kiel CC BY 4.0; Lübeck CC0; Norderstedt dl-de/by-2-0 | |
| Water | yes | INSPIRE `https://service.gdi-sh.de/SH_INSPIREDOWNLOAD_AI_HY-P_ATKIS`, `…_HY-N_ATKIS`, `…_HY-P_ALKIS` (CKAN, CC BY 4.0) | 25832 | CC BY 4.0 | not sampled |
| Knicks / landscape | yes | Biotope lines `https://opendata.schleswig-holstein.de/data/lfu51/SH4_BKSH_Linien_gesamt.zip` (Knicks, Feldhecken, Baumreihen; CC BY 4.0, older copy "other-closed"); INVEKOS `…/Landschaftselemente_2026_GPKG.zip` | 25832 | CC BY 4.0 | rural; low sprint value |

**Recommendation:** (1) Basis-DLM WFS (shared NAS reader; low-medium). (2) Kiel WFS and Lübeck CSV trees (low).
(3) Full ALKIS per Flur only if walls/fences prove valuable (high: bulk GML).

## Mecklenburg-Vorpommern

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes | WFS `https://www.geodaten-mv.de/dienste/alkis_wfs_nas` "WFS_MV_ALKIS_NAS" (139 types). Rostock box `311957,5996167,312757,5996967` (EPSG:25833): `AX_SonstigesBauwerk…` 762 (Zaun 1740 ×152, Stützmauer 1721/1722 ×136, Überdachung 1610 ×73, Mauerkante 1702 ×49, Treppenunterkante 1630 ×48, Sonstiges 9999 ×169), `AX_Bauteil` 342 (Durchfahrt 11, Arkade 7, auskragend 64), `AX_WegPfadSteig` 203 (3.2 MB), `AX_Vegetationsmerkmal` 13 (Hecke), `AX_BauwerkImVerkehrsbereich` 2 | 25833 (5650) | CC BY 4.0, "© GeoBasis-DE/M-V <Jahr der letzten Datenlieferung>" (Fees "Es gelten keine Bedingungen") | numberMatched given; responses are large (1 MB for 800 m) |
| Basis-DLM | yes | WFS `https://www.geodaten-mv.de/dienste/atkis_bdlm_wfs_nas` (117 types; URL from LAiV short description of 22.08.2024). Same box: `AX_Vegetationsmerkmal` 8 (Laubbaum 1012 ×5, Baumbestand, Gehölz), `AX_WegPfadSteig` 26, Denkmal 3. A simple-feature variant `WFS_MV_Basis-DLM_SF` exists. GPKG 367 MB (data 05.11.2026) | 25833 | CC BY 4.0 | |
| LoD2 | yes | "Kostenfreie Datenbereitstellung (Downloadportal)" and ATOM per `https://www.laiv-mv.de/Geoinformation/Geobasisdaten/gebaeude–modelle/` (~1.3 M buildings, footprints of the previous year) | 25833 | CC BY 4.0 | not sampled |
| Trees | yes | Rostock `https://geo.sv.rostock.de/download/opendata/baeume/baeume.{csv,json,gml}` (GovData "Bäume", OpenData.HRO, cc-zero, modified 2026-11-05) | | CC0 | host unreachable from this machine ("Network is unreachable"); not verified live |

**Recommendation:** (1) ALKIS NAS WFS: the richest cadastre in this survey for fences, retaining walls, stairs,
hedges and paths (shared reader). (2) Basis-DLM NAS WFS from the same host (shared). (3) Rostock trees (low).

## North Rhine-Westphalia

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes | WFS `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_aaa-modell-basiert` "WFS NW ALKIS AAA-Modell-basiert" (139 types). Box `354504,5642507,354904,5642807` (EPSG:25832): `AX_Vegetationsmerkmal` 110 (all Laubbaum 1012 points), `AX_SonstigesBauwerk…` 40 (Zaun ×17, Überdachung ×10+3 Carport, Mauerkante ×8, Stützmauer ×2), `AX_Bauteil` 21 (Durchfahrt ×5), `AX_WegPfadSteig` 16, `AX_Strukturlinie3D` 14 (Oberkante 1220, Unterkante 1230, sonstige 1240). Bulk: "Grundrissdaten (NAS)" per Kreis `https://www.opengeodata.nrw.de/produkte/geobasis/lk/akt/gru_xml/gru_05314000_Bonn_EPSG25832_NAS.zip` (217 MB, 2026-07-28) | 25832 | dl-de/zero-2-0 (no attribution) | numberMatched given |
| Basis-DLM | yes | `https://www.opengeodata.nrw.de/produkte/geobasis/lm/akt/basis-dlm/basis-dlm_EPSG25832_NAS.zip` (1,193,134,228 B) and `…_Shape.zip` (4.69 GB), 2026-10-21; GPKG 2.1 GB (data 01.11.2026) | 25832 | dl-de/zero-2-0 | no Basis-DLM WFS found (guesses 404) |
| LoD2 | yes | `https://www.opengeodata.nrw.de/produkte/geobasis/3dg/lod2_gml/lod2_gml/LoD2_32_354_5641_1_NW.gml` (1 km, 7-23 MB, 35,022 files listed); OGC API `https://ogc-api.nrw.de/3dg` | 25832 | dl-de/zero-2-0 | |
| Trees | yes | Köln WFS `https://geoportal.stadt-koeln.de/wss/service/baumkataster_extern_wfs/guest` type `ms:baumkataster` (dl-de/zero-2-0); Dortmund `https://open-data.dortmund.de/api/v2/catalog/datasets/baumkataster/exports/csv` (dl-de/zero-2-0); Münster (dl-de/by-2-0); also Aachen, Bochum (viewer), Krefeld, Duisburg, Essen, Wesel | 25832 | as listed | Düsseldorf, Bonn not found on GovData |
| Other | yes | ABK `…/lk/akt/abk_tiff/` (raster only, template candidate); landscape elements OGC API `https://ogc-api.nrw.de/inspire-lc-le` (rural hedges/tree rows) | 25832 | dl-de/zero-2-0 | |

**Recommendation:** (1) ALKIS AAA WFS: single trees as points (417/418), fences, walls, embankment edges, canopies
(shared reader). (2) Köln/Dortmund/Münster trees for street space (low). (3) Basis-DLM via GPKG for water axes.

## Lower Saxony

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Full ALKIS | yes | WFS `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_nas` "WFS NI ALKIS NAS-konform" (133 types). Hannover box `551330,5802775,552530,5803875` (EPSG:25832): `AX_SonstigesBauwerk…` 153 (Überdachung 1610 ×151, Denkmal ×2), `AX_Bauteil` 41 (Durchfahrt 2610 ×23, Arkade ×16), `AX_BauwerkImVerkehrsbereich` 7 (Brücke), `AX_Vegetationsmerkmal` 0, no walls/fences | 25832 | CC BY 4.0, "LGLN (<Jahr>)" | `numberMatched=unknown` |
| Basis-DLM | yes | LGLN hub `https://ni-lgln-opengeodata.hub.arcgis.com/documents/lgln-opengeodata::basis-dlm-shapefile/about` (FileGDB/Shape, free); GPKG 1.4 GB (data 05.11.2026) | 25832 / 4326 | CC BY 4.0 | no Basis-DLM WFS found (guessed doorman names → 400) |
| LoD2 | yes | LoD2 GeoJSON tile index on the LGLN hub (`…/pro-download-indices/lod2/…`, per the reference doc) | 25832 | CC BY 4.0 | not sampled |
| Trees | not found | Hannover publishes reports, no download found; city Open GeoData default licence CC BY 4.0 | | | |

**Recommendation:** (1) ALKIS NAS WFS for canopies, passages and bridges (shared reader; but these are mostly in
vereinfacht already, so the gain is small). (2) Basis-DLM GPKG for water axes, tree rows, hedges (shared).
(3) vereinfacht Bauteil/Überdachung rule (low). Skip LoD2.

## Rhineland-Palatinate

| Source | Open? | Verified service / URL | CRS | Licence | Notes |
|---|---|---|---|---|---|
| Basis-DLM | yes | WFS `https://geo5.service24.rlp.de/wfs/basisdlm_shape_rp.fcgi` "WFS RP Basis-DLM (Shape)" (45 types in the AdV Shape schema: `vermkv:Vegetation_VEG04_{L,P,F}`, `Gewaesser_GEW01_L`, `Verkehr_VER03_L`, `Siedlungen_SIE03_{L,P,F}`, `Relief_REL01_L` …). Koblenz box `399000,5578000,401500,5580500` (EPSG:25832): VEG04_L 2, VEG04_P 37, GEW01_L 9, VER03_L 63, SIE03_L 73, REL01_L 5. Bulk `https://geobasis-rlp.de/data/basisdlm/current/shp/BasisDLM_SHP_RP.zip` (336 MB, 2026-03-31). GPKG 1.0 GB (data 01.11.2026) | 25832 (many) | dl-de/by-2-0, "©GeoBasis-DE / LVermGeoRP<Jahr>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]" | Shape-schema attribute names differ from NAS; the GPKG is the uniform path |
| Full ALKIS | no | only vereinfacht (`alkis_rp.fcgi`, OGC API 519), TN-Einzellayer WFS 451 (land use only), Vermessungspunkte OGC API 669 | 25832 | dl-de/by-2-0 | vereinfacht already has 1,775 Überdachung + 1,225 Durchfahrt in the test area |
| LoD2 | yes | `https://geobasis-rlp.de/data/geb3dlo/current/gml/LoD2_32_400_5570_2_RP.gml` (2 km, plain GML, 47-99 MB in Koblenz; 10,533 listed incl. `.txt` metadata, 2026-04-27) | 25832 | dl-de/by-2-0 | large tiles |
| Trees | not found | Mainz, Koblenz, Trier, Kaiserslautern, Ludwigshafen: nothing on GovData | | | |

**Recommendation:** (1) vereinfacht Bauteil/Überdachung rule (low, high yield here). (2) Basis-DLM via the GPKG
(shared) or the Shape WFS (separate schema). LoD2 and trees: skip.

---

## Cross-state shared implementations

| # | Implementation | States | Gives | Effort |
|---|---|---|---|---|
| 1 | **Rule only:** ALKIS vereinfacht `Bauteil` (Durchfahrt, Arkade, auskragend) → passage/522, `… Überdachung` → 522 | all ten (and Berlin) | passages, canopies | low |
| 2 | **AAA NAS WFS reader** (`adv:` GID 6.0/7.1, GML 3.2, bbox in 25832/25833, `COUNT`/`STARTINDEX` paging, AdV code lists → symbols) | full ALKIS: BB, HE, MV, NW, NI; Basis-DLM: HE, MV, SH, ST | walls, retaining walls, fences, stairs, hedges, single trees, tree rows, ditches, bridges, embankment edges, footpaths | medium (one reader, a per-state service table, one rule file like `osm-candidates.toml`) |
| 3 | **basemap.de Basis-DLM GPKG reader** (`https://basemap.de/dienste/opendata/basisviews/` listing, file name carries the date; SQLite with R-tree, read by HTTP Range or cached per state like the Geofabrik PBF) | all ten (fallback where no Basis-DLM WFS: BB, SN, HH, NW, NI, RP) | water axes with width and "nicht ständig Wasser führend", tree rows, hedges, single trees, walls/fences/stairs (DLM level), dams, embankments, way categories | medium; one schema for all states, EPSG:4326 |
| 4 | **Tree point reader** (WFS GML/GeoJSON, OGC API, CSV; config: URL, x/y or geometry, crown-Ø field, licence line) | BB (Potsdam, FFO), SN (Leipzig, Dresden), HH, HE (Frankfurt), SH (Kiel, Lübeck, Norderstedt), MV (Rostock), NW (Köln, Dortmund, Münster) | 417/418 candidates, crown size | low-medium |
| 5 | OGC API Features GeoJSON reader (exists for LGB/MV/HH ALKIS) reused for Hamburg Feinkartierung and Straßenbaumkataster | HH (also NRW `3dg`, `inspire-lc-le`) | railings, fences, park walls/stairs, surfaces | low |
| 6 | INSPIRE `hy-p:Watercourse` reader | BB, SN, ST, SH, HE | water lines (geometry, width range, origin) | low, but superseded by 2/3 |
| — | Not recommended: LoD2 (CityGML/CityJSON) and basemap.de vector tiles (z15 maximum; one Sanssouci tile held 1 `Zaun` where ALKIS has 136 fences) | | | |

Code → ISSprOM mapping for 2/3 (candidates unless noted): Mauer 1700-1703 → 513.1/515 (height from nDSM decides),
Stützmauer 1720-1723 → 513.2, Zaun 1740 → 516/518, Treppe 1620/1621/1630 → 532.x, Überdachung 1610/1611 → 522,
Brunnen 1780-1783 → 311, Denkmal 1750 → 526, Hecke 1100-1103 → 410.1, Laub-/Nadelbaum 1011/1012 → 417/418,
Baumreihe 1210-1230 → row of 418, Graben 1010-1013 / Gewässerachse (width < 2 m) → 305/306, Brücke/Steg 1800/1820 →
512, Strukturlinie3D 1220/1230 and DammWallDeich → 104/105. Code lists: `https://registry.gdi-de.org/codelist/de.adv-online.gid/<Name>/<Name>.en.json`.

## Method notes and gaps

- Test boxes are the freemapper reference areas (Sanssouci/Strausberg, Dresden/Bautzen, Halle, Hamburg-Eppendorf,
  Marburg, Lübeck, Rostock, west of Köln, Hannover, Koblenz). Counts are for those boxes only; capture density
  varies by municipality.
- GovData CKAN (`https://ckan.govdata.de/api/3/action/package_search`) was the most reliable catalogue; licences come
  from resource `license` fields or service Capabilities. `gdk.gdi-de.org` was unreachable.
- Not verified live: Saxony LoD2 tiles (503), Rostock trees (host unreachable), NI/MV/SH/HE LoD2 files, SH full ALKIS
  per-Flur files, Hamburg Basis-Gewässernetz WFS, ST/SH INSPIRE hydro WFS, Halle/Magdeburg trees.
- The basemap.de GPKG licence is per state (the page links each state's licence; dl-de/zero or dl-de/by); attribution
  in basemap.de products: "© <year> basemap.de / BKG | Datenquellen: © GeoBasis-DE". Schleswig-Holstein's GPKG data
  is a year old; use the state WFS there.
- Largest response read: 3.3 MB (MV `AX_WegPfadSteig`). No bulk files were downloaded (GPKG: 512 KB header range only).
