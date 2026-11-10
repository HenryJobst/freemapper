# Berlin plus: open data beyond what freemapper uses today

Verified live on 2026-11-08 (curl and Python urllib from macOS, no login). Every statement below was
observed unless it is marked **UNVERIFIED**. Samples live in this folder: `dl/` (GeoJSON, CityGML,
the first 2 MB of an ALS tile, a K5 map image), `probe/` (capabilities, catalogue JSON, ATOM feeds, ZIP
directory listings, logs). Scripts: `scripts/` (`sample.py` WFS sampler, `stats.py` value counts,
`remotezip.py` remote ZIP lister, `raster_cmp.py` area comparison, `utm.py`). About 66 MB were
downloaded in total.

Test areas (EPSG:25833, E/N):
- **Viktoriapark**, Kreuzberg (`examples/berlin-viktoriapark`, map area lon 13.37510–13.38690,
  lat 52.48531–52.49069): `389660,5816250,390480,5816840`.
- **Helene-Haeusler-Schule** and its kilometre in Mitte/Prenzlauer Berg (the school box used in
  `crates/freemapper/src/sources/berlin.rs` tests): `392000,5820000,393000,5821000`.

## 0. Result in one table (ranked by benefit/effort)

ISSprOM codes are the ones in `assets/symbol-sets/ISSprOM 2019_4000.omap` (OOM set). Note: in this set
the stairway is **532.6–532.9** (footprint lines), not 534; canopy 522; passage pillar 522.1; small monument 526;
prominent man-made feature 530/531; prominent large tree 417, bush or small tree 418; hedge 410.1;
step or edge of paved area 501.1; earth bank 104. Schoolyard codes from `school-2019-additions.xml`:
321 hydrant, 541 playground equipment, 542 table, 543 bench, 544 pole, 545 lamppost.

| # | Source (service · type) | Symbols | Reliable → object / candidate | Effort | Priority |
|---|---|---|---|---|---|
| 1 | **ALKIS Berlin structures** `gdi.berlin.de/services/wfs/alkis` · `alkis:bauwerkeflaechen`, `alkis:bauwerkelinien`, `alkis:relief`, `alkis:gewaesservegetationlinien` + building parts (`bezbat`) already in `alkis_gebaeude:gebaeude` | 522 canopy (Überdachung 100 414 citywide, Carport 6 581); 532.x stairs (Treppe 8 331, Freitreppe 259 + 185 lines); 515 walls (Mauerkante/Mauermitte/Mauer ≈ 9 300 lines + 1 915 areas); 513.2 retaining walls (Stützmauer ≈ 9 700 lines + 433 areas); 518 fences (Zaun 75 409); 522 passage candidates (Bauteil "Durchfahrt im Gebäude", "Arkade"); 104 earth bank (relief Oberkante); 526 monuments (2 725); 410.1 hedges (Heckenkante, 2 366) | **522 Überdachung/Carport: map object.** Everything else candidate (no height, no passability) | low–medium: reuse `berlin::fetch_wfs`, new rules file, a stairs builder | **1** |
| 2 | **K5 Karte von Berlin 1:5000** WMS `gdi.berlin.de/services/wms/k5_farbe` · layer `k5_farbe` | template only (shows park paths, stairs, walls, fences, monuments, spot heights, house numbers) | template | very low: existing WMS template path | **2** |
| 3 | **Baumbestand Berlin** `gdi.berlin.de/services/wfs/baumbestand` · `baumbestand:strassenbaeume` (434 765), `baumbestand:anlagenbaeume` (527 780) | 417 (crown ≥ 10 m) / 418 | candidate | low | **3** |
| 4 | **Straßenbefahrung 2014** `gdi.berlin.de/services/wfs/strassenbefahrung` (70 types) | 501.5 green cut-outs and extra paved candidates; 501.1 curbs (Bordstein 524 409); 532.x stairs (Treppe 1 824, with step count); 515 walls (Mauer 4 362); 518 railings (Geländer 2 492, Verkehrsschutzgitter); schoolyard 543 benches, 321 hydrants, 544 poles/bollards (Poller 344 666), 545 masts; 531 advertising columns; 522 bus shelters; 311 street pumps | candidates (2014–2016 survey); green cut is the one change to map objects | medium | **4** |
| 5 | **ALKIS land use `bezfkt = Kleingarten`** (already fetched, `alkis:tatsaechlichenutzungflaechen`) | 520 out of bounds | candidate (like residential parcels) | very low | **5** |
| 6 | **Öffentliche Beleuchtung** `…/wfs/beleuchtung` · `beleuchtung:beleuchtung`; **Zierbrunnen** `…/wfs/zierbrunnen`; **Trinkwasserbrunnen** `…/wfs/trinkwasserbrunnen` (242) | 545 (schoolyard), 311, 313 (schoolyard) | candidate | low | 6 |
| 7 | **Building currency** `…/wfs/ua_gebaeudehoehen` · `ua_gebaeudehoehen:gebaeudehoehen` (`grundrissaktualitaet`, joined by `gml_id` = ALKIS `uuid`) | fills `updated` of 521 objects (today `None` in Berlin) | metadata | low | 7 |
| 8 | **Biotoptypen 2024** `…/wfs/ua_biotoptypen_2024` · `ac_biotoptypen_f` | 401 lawn, 405/406/408 woody stands, 520 planted beds | candidate | medium (≈ 900 class names to map) | 8 |
| 9 | **ALS point cloud 2021** ATOM `gdi.berlin.de/data/a_als/atom/` (9 regional ZIPs, 249 GB) | classified nDSM template (vegetation vs structures), later 515/201/501.1 detection research | template | high | 9 (later) |
| 10 | LoD2 CityGML `gdi.berlin.de/data/a_lod2/atom/` (1 km zips, ~3.3 MB) | heights of 521/522 (no passages: solids reach the ground) | — | medium | 10 |
| 11 | ATKIS Basis-DLM `…/wfs/atkis` paths, Fußgängernetz, Detailnetz, Radverkehrsanlagen, Parken im Straßenraum | path centre lines (OSM covers this better) | — | low | 11 |
| 12 | Grünanlagen/Spielplätze, Friedhofsbestand, Kleingartenbestand (WFS), Denkmale (heritage parcels), Gewässerkarte | park/playground outlines, nothing ISSprOM-specific beyond ALKIS | — | low | 12 |
| — | Umweltatlas Versiegelung 2021, Grünvolumen 2020, Vegetationshöhen 2020 (WFS) | none: values are per block (`schluessel`), not per object | not usable at 1:4000 | — | drop |

Recommended order: **Plan 1** (`2026-11-09-berlin-plus.md`) = sources 1, 2, 3, 5. **Plan 2**
(`2026-11-09-berlin-plus-2.md`) = sources 4, 6, 7. Sources 8 and 9 need their own plans later.

---

## 1. What freemapper uses in Berlin today (from the repo, read-only)

- `docs/regions/berlin-reference.md` §0, spec `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1:
  rasters from LGB (DOP20, DGM1, bDOM → nDSM), ALKIS from the Berlin WFS: buildings
  `alkis_gebaeude:gebaeude`, land use `alkis:tatsaechlichenutzungflaechen`, normalised to the LGB schema
  in `crates/freemapper/src/sources/berlin.rs` (`fetch_wfs`, `normalise_building`, `normalise_land_use`,
  `PAGE_COUNT = 5000`, `get_with_retries`), OSM from Geofabrik Brandenburg (contains Berlin).
- Build rules (`crates/freemapper/src/build/`): `buildings.rs` (521 for `Gebäude`, not underground;
  `AX_Bauteil` is dropped), `paved.rs` (501.5 for `Straßenverkehr`, Berlin squares are candidates via
  `bez = "ohne Unterart (Berlin)"`), `water.rs` (301.1/301.6), `out_of_bounds.rs` (520 candidates for
  `Wohnbaufläche`, `Industrie- und Gewerbefläche`, `Fläche gemischter Nutzung`, `Bahnverkehr`),
  `candidates.rs` + `assets/rules/osm-candidates.toml` (OSM fences 518, walls 515, retaining walls 513.2,
  hedges 410.1, trees 418, schoolyard furniture …).
- Spec §4.5 says "*candidate 522 canopy:* roofed open structures"; ADR 0003 "sure objects as map objects,
  the rest as templates".
- Manual `docs/manual/de.md` line 244: "In Berlin und Sachsen enthalten Straßenflächen auch Grün- und
  Baumstreifen am Rand." That is the known paved-area weakness source 4 can reduce.
- `zip_range.rs` reads ZIP32 directories (`count: u16`) and members stored or deflated; `Cargo.toml` has
  `zip` with feature `deflate` only and `las = "=0.11.1"` with `laz`.
- `OsmGeometry` is `Point` or `Line` (closed line = area; no holes). The candidate rule engine
  (`Rules::symbol_for`, `osm_objects`) works on string tags, so Berlin features can run through it if
  their properties become tags.

## 2. Portal basics (new findings)

- All services below are on the same GeoServer as the ALKIS WFS already in use
  (`https://gdi.berlin.de/services/wfs/<name>`). Each `GetCapabilities` `Fees` reads "Für die Nutzung der
  Daten ist die Datenlizenz Deutschland - Zero - Version 2.0 anzuwenden." (`probe/caps/*.xml`).
- `outputFormat=application/json`, `bbox=…,urn:ogc:def:crs:EPSG::25833`, coordinates E,N, `CountDefault`
  1 000 000, `links[rel=next]` paging: verified on `strassenbefahrung:av_poller` with `count=1000` in the
  school km → `numberMatched` 2392, `numberReturned` 1000, `links` `["next"]`. So `berlin::fetch_wfs`
  works unchanged for every layer here. Exception: its de-duplication key is `properties.uuid`, which
  the tree, Straßenbefahrung and lighting layers lack (they have `gisid`, `gis_id`, `id`); the top-level
  GeoServer feature `id` (e.g. `gebaeude.DEBE03YY600004AR`) is always present.
- No request failed during this session (≈ 250 requests), but the earlier reference saw ~1 in 20
  connection resets; keep the retries.
- Catalogue: CKAN `https://datenregister.berlin.de/api/3/action/package_search|package_show` works, but
  started returning empty bodies after ~30 quick calls (`probe/pkg/*.json` failures); 2 s spacing fixed it.
- Typical response times for a 1 km² bbox: 0.1–0.7 s per type; sizes 1 KB–1.6 MB.

## 3. ALKIS Berlin full content (source 1)

Service `https://gdi.berlin.de/services/wfs/alkis` (catalogue `alkis-berlin-amtliches-liegenschaftskatasterinformationssystem-wfs-66e083ae`,
updated 2026-10-04, `dl-de-zero-2.0`). Feature types beyond the two in use (capabilities, `probe/caps/alkis.xml`):
`alkis:bauwerkeflaechen`, `alkis:bauwerkelinien`, `alkis:relief`, `alkis:gewaesservegetationflaechen`,
`alkis:gewaesservegetationlinien`, `alkis:vegetationpunkte`, `alkis:gebaeudelinien`, `alkis:flurstuecke`, …

Citywide `resultType=hits`: `bauwerkeflaechen` 178 946, `bauwerkelinien` 682 661, `relief` 133 629,
`gewaesservegetationlinien` 2 366, `vegetationpunkte` 34 688.

### 3.1 `alkis:bauwerkeflaechen` (areas)
Schema: `uuid, bwf, bezbwf, nam, art, bezart, spo, bezspo, atp, bezatp, bkt, bezbkt, bfk, bezbfk, des, bezdes, bezeich`.
Citywide `bezbwf` (one request with `propertyName=bezbwf`, `dl/bauwerkeflaechen_bezbwf_all.json`):
Überdachung 100 414, (none) 47 617, Treppe 8 331, Carport 6 581, Denkmal/Denkstein/Standbild 2 725,
Mauer 1 915, Brücke 1 817, Rampe 1 804, Spielfeld 1 694, Brückenpfeiler 1 340, Landebrücke 957,
Rohrdurchlass 924, Tunnel/Unterführung 535, Stützmauer 433, Freitreppe 259.
The `(none)` rows are other classes, e.g. `AX_WegPfadSteig` (`bezart` Fußweg, Rad- und Fußweg, with `nam`
like "Tivoliplatz") and `AX_Bahnverkehrsanlage`.
- Viktoriapark box: 101 features (Überdachung 54, Treppe 21, Carport 5, Tunnel 4, Brücke 4, Denkmal 1,
  Stadion 1, Spielfeld 1); **18 Treppe, 11 Überdachung, 4 Brücke inside the park core**.
- School km: 755 (Überdachung 435, Treppe 64, Rampe 16, Brückenpfeiler 14, Brücke 10, Spielfeld 8,
  Denkmal 4, Mauer 3, Springbrunnen 1).
- Sample: `{"uuid":"DEBE00YY11l000RG","bwf":"1610","bezbwf":"Überdachung","bezdes":"Aus Katasterkarten digitalisiert","bezeich":"AX_SonstigesBauwerkOderSonstigeEinrichtung"}`.

### 3.2 `alkis:bauwerkelinien` (lines)
Schema: `objectid, bwf, bezbwf, nam, art, bezart, fkt, bezfkt, bkt, bezbkt, bfk, bezbfk, des, bezdes, bezeich, uuid`.
Citywide (`propertyName=bezbwf,bezart`, 136 MB, 35 s; deleted after counting):
Fahrbahnbegrenzungslinie 526 342 (+ "überdeckt" 43 176), Zaun 75 409, Brückenpfeiler 9 543,
Widerlager 6 783, Mauerkante rechts 6 317 / links 676, Stützmauer 4 864, Stützmauer rechts 3 974 / links 799,
Spundwand 954, Zuschauertribüne 339, Mauermitte 292, Freitreppe 185, Steg 94, Mauer 80, Stützmauermitte 64.
- Viktoriapark: 636 (Fahrbahnbegrenzungslinie 550, Mauerkante rechts 28, Zaun 28, Widerlager 6, Stützmauer 7);
  park core: Mauerkante rechts 15, Stützmauer rechts 4, Zaun 1.
- "rechts/links" names the side of the wall body; pairs are rare (Viktoriapark 28 rechts / 0 links,
  school km 17 / 2), so each line can be used as is.
- `Fahrbahnbegrenzungslinie` is the ALKIS kerb line (road edge).

### 3.3 `alkis:relief` (`AX_Strukturlinie3D`)
`bezart`: Viktoriapark Sonstige Begrenzungskante 76, **Oberkante 45 / Unterkante 44** (embankment top/bottom;
88 lines in the park core around the Kreuzberg); `bezdes` "Digitalisierung analoger Vorlagen" (old map
source) — school km also "Terrestrische Aufnahme". Geometry is 2D here (MultiLineString).

### 3.4 Vegetation
- `alkis:gewaesservegetationflaechen`: `AX_Vegetationsmerkmal` `bezbws` Gras 303, Zierfläche 36,
  Baumbestand Laubholz 26 (school km); `AX_UntergeordnetesGewaesser` Bach/Teich (Viktoriapark waterfall: 4).
- `alkis:gewaesservegetationlinien`: `bezbws` "Heckenkante, rechts" (hedges), 2 366 citywide, 0 in both boxes.
- `alkis:vegetationpunkte`: "Laubbaum" 315 in the school km (duplicates the tree cadastre).

### 3.5 Building parts (already downloaded today, currently dropped)
`alkis_gebaeude:gebaeude`, `bezeich = AX_Bauteil`, `bezbat`:
Viktoriapark 386 parts: Geringergeschossiger Gebäudeteil 206, **Durchfahrt im Gebäude 111**,
Auskragender Geschossteil / zurückspringendes Geschoss 41, Loggia 14, Keller 6, Arkade 1,
"Durchfahrt an überbautem Verkehrsweg" 1 (note the no-break space in the value).
School km: Durchfahrt im Gebäude 84, Arkade 6, Auskragender … 194.
ISSprOM 521: passages through buildings are shown with the canopy symbol. A passage can be gated, so
these are 522 candidates. "Auskragender Geschossteil / zurückspringendes Geschoss" mixes overhangs
(passable underneath) with set-back upper storeys (not), so it is left out.

### 3.6 ISSprOM mapping and reliability
| ALKIS value | Symbol | Map kinds | Target |
|---|---|---|---|
| `bauwerkeflaechen` Überdachung, Carport | 522 (area) | both | **map object** (cadastral survey; same trust as 521 buildings) |
| Bauteil Durchfahrt im Gebäude, Durchfahrt an überbautem Verkehrsweg, Arkade | 522 | both | candidate |
| Treppe, Freitreppe (areas) | 532.6/.7/.8/.9 by width; direction from DGM1 | both | candidate |
| Freitreppe (lines) | 532.7 | both | candidate |
| Mauer (area), Mauerkante, Mauermitte, Mauer (lines) | 515 | both | candidate (height unknown: 513.1 if low) |
| Stützmauer* (lines/areas) | 513.2 | both | candidate |
| Zaun | 518 | both | candidate (516 if passable) |
| Heckenkante | 410.1 | both | candidate |
| relief Oberkante | 104 | sprint | candidate (Unterkante skipped) |
| Denkmal, Denkstein, Standbild (area) | 526 at the centroid if < 3 m across, else 521-like outline left to the mapper | both | candidate |
| Spielfeld | none (already 501.10/401 via OSM `leisure=pitch`) | — | — |
| Brücke (area) | 501.5 | sprint | candidate (like OSM `man_made=bridge`) |
| Fahrbahnbegrenzungslinie | 501.1 | sprint | candidate (dropped if plan 2 brings Straßenbefahrung curbs) |

Data quality: `bezdes` mostly "Aus Katasterkarten digitalisiert" (digitised from the cadastral map,
sub-metre). No per-feature date in these layers.

## 4. K5 Karte von Berlin 1:5000 (source 2)

- WMS `https://gdi.berlin.de/services/wms/k5_farbe` (also `k5_sw`), layer `k5_farbe`, EPSG:25833, formats
  incl. image/png, image/jpeg; Fees dl-de/zero-2-0 (`probe/caps/wms_k5_farbe.xml`).
- Test: `…/k5_farbe?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=k5_farbe&STYLES=&CRS=EPSG:25833&BBOX=389660,5816250,390480,5816840&WIDTH=1640&HEIGHT=1180&FORMAT=image/png`
  → 200 `image/png`, 1 850 687 B, 1.9 s (`dl/viktoriapark/k5.png`, 0.5 m/px). The image shows every park
  path of the Viktoriapark, stairs, the waterfall, monuments, walls, fences, embankment hatching, spot
  heights and house numbers — the mapper's best single reference in Berlin besides the orthophoto.
- Catalogue `karte-von-berlin-1-5000-k5-farbausgabe-9f3ecf8d`: updated 2026-01-02, released 2026-04-01;
  ATOM `https://gdi.berlin.de/data/k5_farbe/atom/0.atom` with 4 regional TIFF ZIPs (not needed).
- Attribution suggested by the surveying offices: "Geoportal Berlin / Karte 1:5000 von Berlin (K5)"
  (quoted in `docs/regions/berlin-reference.md` §10).

## 5. Baumbestand Berlin (source 3)

- WFS `https://gdi.berlin.de/services/wfs/baumbestand`, types `baumbestand:strassenbaeume` (434 765) and
  `baumbestand:anlagenbaeume` (527 780). Catalogue `baumbestand-berlin-wfs-48ad3a23`: updated 2026-04-09,
  `dl-de-zero-2.0`.
- Points. Fields: `gisid, pitid, standortnr, kennzeich, namenr, art_dtsch, art_bot, gattung_deutsch, gattung,
  art_gruppe, (strnr, strname, hausnr, zusatz), pflanzjahr, standalter, kronedurch, stammumfg, baumhoehe,
  eigentuemer, bezirk`. No `uuid`.
- Viktoriapark: 308 street trees (crown <4 m 13, 4–8 m 79, 8–12 m 136, ≥12 m 80) and 1 397 park trees
  (crown missing 389, <4 m 178, 4–8 m 375, 8–12 m 271, ≥12 m 184). School km: 875 + 500.
  Sample: `{"art_dtsch":"Silber-Linde","namenr":"Viktoriapark","pflanzjahr":"1976","kronedurch":8,"stammumfg":130,"baumhoehe":20,"eigentuemer":"Land Berlin"}`.
- Covers street and park trees of the districts; trees on school grounds or private land are not
  included (**UNVERIFIED** beyond the samples).
- Separate dataset `baumbestand_gruen_berlin:anlagenbaum_gruenberlin` (31 298, Grün Berlin GmbH parks):
  0 in both boxes; licence conflict: CKAN `dl-de-by-2.0`, service Fees "Zero". Leave it out until clarified.
- OSM Berlin carries many `natural=tree` nodes (often imported from this cadastre, **UNVERIFIED**), so OSM
  tree candidates within 2 m of a cadastre tree should be dropped.

## 6. Straßenbefahrung 2014 (source 4)

- WFS `https://gdi.berlin.de/services/wfs/strassenbefahrung` (WMS same name), 70 feature types
  (`probe/caps/strassenbefahrung.xml`). Catalogue `strassenbefahrung-2014-wfs-a6b235f6`, `dl-de-zero-2.0`,
  creation 2014-01-01, publication 2019-05-22, frequency "asNeeded". Abstract: "In den Jahren 2014 und 2015
  wurde der gesamte öffentliche Straßenraum Berlins durch eine vermessungstechnische Straßenbefahrung erfasst
  … umfasst das Haupt- und Nebenstraßennetz und teilweise auch Wege in öffentlichen Parkanlagen."
- No newer survey exists (`strassenbefahrung_2021|2023|2024` → 404; catalogue search found only 2014).
- Survey dates are embedded in `gis_id` (`Fl_20150413_37`, `Li_25.09.2014_230`, `FL_16072014_03142`,
  `Pk_2015-05-02_3121`): Viktoriapark 2014-09-25 … 2016-04-01; school km 2014-07-16 … 2016-08-19.
- Citywide counts: Bordstein 524 409, Gehweg 713 598, Fahrbahn 63 175, Poller 344 666, Mauer 4 362,
  Geländer 2 492, Treppe 1 824.
- Viktoriapark box (`probe/sb_viktoriapark.txt`): Poller 764, Gehweg 541, Gebäudeeingang 377,
  Fahrbahnmarkierung 338, Bordstein 316 (9.5 km), Kabelschacht 248, Baumscheibe 243, Verkehrszeichen 181,
  Straßensinkkasten 160, Kanaldeckel 136, Parkfläche 65, Kabelkasten 60, Straßenbegrenzung 56, Fahrbahn 43,
  Hydrant 41, Gehwegüberfahrt 40, Mast_LSA 37, Abfallbehälter 33, Radweg 31, Mast 25, Trennstreifen 20,
  Durchfahrtshöhe 18, Fahrradständer 15, Haltestellenwartebereich 10, Sitzbank 9, Verkehrsschutzgitter 9,
  Werbesäule 6, Mauer 5, Fahrgastunterstand 5, Treppe 0, Geländer 0. In the park core only 42 path
  polygons (Gehweg, `elem_nr` "1.102"), no stairs or walls: the survey stops at the park edge.
- School km: Poller 2 392, Gehweg 1 689, Bordstein 1 246 (37 km), Baumscheibe 687, Öffentlicher Platz 597,
  Fahrbahn 263, Mast 239, Verkehrsschutzgitter 89, Werbesäule 71, Grünfläche 65, Sitzbank 46, **Treppe 38**,
  Geländer 15, Rampe 3, Mauer 1.
- Attributes are **codes without a published code list** (no docs: `…/data/strassenbefahrung/docs/` 403;
  JSON legends are single-symbol; GetFeatureInfo returns only raw values). Observed: Bordstein `bor` 1/2/3,
  `sre` 0/1/NULL; Mauer `hms` "2", `wer` 0; Poller `pot` 1/2/3/4; Gehweg `material` 01–17, `gws` 1/2/3/4/6;
  Treppe `trs` 2–11 (looks like the number of steps), `laenge`, `material`; Rampe `stg`.
- Area comparison on a 0.5 m grid (`scripts/raster_cmp.py`), ALKIS `AX_Strassenverkehr` vs the union of
  Fahrbahn, Gehweg, Radweg, Parkfläche, Öffentlicher Platz, Gehwegüberfahrt, Haltebereich,
  Haltestellenwartebereich, Fußgängerzone, Trennstreifen, Rampe, Treppe:
  - Viktoriapark: ALKIS 90 546 m², surfaces 108 412 m², both 81 361 m²; **ALKIS not surveyed 9 185 m² (10.1 %)**;
    surveyed green (Grünfläche + Baumscheibe) inside ALKIS 1 442 m²; surfaces outside ALKIS 27 051 m²
    (park paths, squares, forecourts).
  - School km: ALKIS 388 811 m², surfaces 343 103 m², both 284 930 m²; ALKIS not surveyed 103 881 m² (26.7 %);
    green inside ALKIS 9 800 m²; surfaces outside ALKIS 58 173 m².
  The ALKIS remainder is verges, planted strips and places rebuilt since 2014 (Alexanderplatz area), so
  replacing ALKIS by the survey would drop real paving; cutting only the surveyed green is safe.

ISSprOM use:
| Layer | Symbol | Target |
|---|---|---|
| `ce_gruenflaeche` | cut out of the 501.5 objects (min side 1 mm on paper) and 401 candidate | object change + candidate |
| surfaces outside ALKIS Straßenverkehr (`cm_fahrbahn`, `cl_gehweg`, `ch_radweg`, `ck_parkflaeche`, `ci_oeffentlicher_platz`, `cj_fussgaengerzone`, `by_gehwegueberfahrt`) | 501.5 | candidate (sprint) |
| `bd_bordstein` | 501.1 | candidate (sprint) |
| `cc_treppe` | 532.6–.9 (width; `trs` kept as tag) | candidate |
| `bh_mauer` | 515 | candidate |
| `bf_gelaender`, `bb_verkehrsschutzgitter`, `bg_leitplanke` | 518 | candidate |
| `bj_sitzbank` | 543 | candidate (schoolyard) |
| `aq_hydrant` | 321 | candidate (schoolyard) |
| `av_poller`, `as_mast`, `am_fahnenmast` | 544 | candidate (schoolyard) |
| `ag_werbesaeule` | 531 | candidate (sprint) |
| `br_fahrgastunterstand` | 522 | candidate |
| `af_wasserpumpen_brunnen`, `bv_springbrunnen_zierbrunnen` | 311 | candidate |
| `ac_trinkwasserbrunnen_wasserspender` | 313 | candidate (schoolyard) |
| `aw_spielgeraet` | 541 | candidate (schoolyard) |
| all others (signs, markings, manholes, cable boxes, …) | none | — |

## 7. Small point layers (source 6)

- **Öffentliche Beleuchtung** `https://gdi.berlin.de/services/wfs/beleuchtung` · `beleuchtung:beleuchtung`,
  catalogue `offentliche-beleuchtung-wfs-77fc5415` updated 2026-09-30. MultiPoint; Viktoriapark 178:
  "Lichtmast mit Auslegerleuchte" 103, "… Aufsatzleuchte" 61, "Gas-Aufsatzleuchte" 7, status "In Betrieb" 173.
  → 545 lamppost (schoolyard), drop OSM `highway=street_lamp` within 2 m.
- **Zierbrunnen** `…/wfs/zierbrunnen` · `zierbrunnen:bez_zierbrunnen` (updated 2026-04-08): Viktoriapark 3
  (e.g. `{"kategorie":"Tiefbrunnen","brunnenbezeichnung":"Viktoriapark","betreut_durch":"Berliner Wasserbetriebe"}`) → 311.
- **Trinkwasserbrunnen** `…/wfs/trinkwasserbrunnen` · `trinkwasserbrunnen:trinkwasserbrunnen`
  (242 citywide, 2025-06-15) → 313 schoolyard.
- **Schulen** `…/wfs/schulen` · `schulen:schulen` (points with `bsn`, `schulname`, `schulart`; updated
  2026-09-23): Viktoriapark 1 (Charlotte-Salomon-Grundschule). Could name the school or seed the schoolyard
  area picker; no grounds polygon.

## 8. Building currency (source 7)

- `https://gdi.berlin.de/services/wfs/ua_gebaeudehoehen` · `ua_gebaeudehoehen:gebaeudehoehen` ("Gebäudehöhen 2022",
  catalogue updated 2023-08-23): fields `gml_id` (= ALKIS uuid), `hoehe`, `geschosse`, `grundrissaktualitaet`,
  `quelledachhoehe_txt` (Laserscan 934, Photogrammetrie 78). Viktoriapark: 418 of 430 ALKIS `AX_Gebaeude` uuids
  joined; dates 2015-08-27 (466), 2021-03-11 (329), 2018-09-18 (55) …
- The LoD2 tiles carry the same `Grundrissaktualitaet` per building and are newer (creationDate 2026-03-02),
  but cost a 3.3 MB download per km² and a CityGML reader.

## 9. Biotoptypen 2024 (source 8)

`https://gdi.berlin.de/services/wfs/ua_biotoptypen_2024` · `ua_biotoptypen_2024:ac_biotoptypen_f` (lines and points
types were empty here). Viktoriapark 84 polygons, `kart_jahr` 2023, `kart_methode` L (aerial interpretation) 82:
"mehrschichtige Gehölzbestände" 17, "Zier-/Scherrasen, mit lockerstehenden Bäumen" 5, "… weitgehend ohne Bäume" 2,
"Gräben, weitgehend naturfern …" 2, block and street classes. Useful for park vegetation candidates
(401/405/406/408) after a class table; medium effort.

## 10. ALS point cloud (source 9; coordinator question)

- ATOM `https://gdi.berlin.de/data/a_als/atom/0.atom` (`<updated>2025-08-07`): **9** regional ZIPs (the
  mapant-germany note `processing_pipeline/lidar_open_data_germany.md` says 8 and "~200 GB"; the feed lists
  Mitte 37.3, Nord 32.5, Nordost 1, Nordwest 16.3, Ost 16.1, Süd 35.3, Südost 50, Südwest 32, West 27.3 GB
  ≈ 248 GB). Rights: dl-de/zero-2-0.
- **Range reads work** (`scripts/remotezip.py`, `probe/als_*_list.txt`): `Accept-Ranges: bytes`, ETag,
  Last-Modified 2025-08-14.
  - `Nordost.zip` (982 934 068 B): ZIP32, 11 members, central directory 770 B at offset 982 933 276,
    **method 9 (Deflate64)** for all members.
  - `Mitte.zip` (36 734 670 085 B): **ZIP64**, 160 members, central directory 18 640 B,
    **method 8 (Deflate)** for all.
  - Members are uncompressed LAS 1.4 (`3dm_33_EEE_NNNN_1_be.las`, 1 km tiles), not LAZ.
- Read test: member `3dm_33_389_5816_1_be.las` (Viktoriapark) at offset 11 574 079 679, 229 770 899 B compressed
  / 526 243 477 B raw. The first 2 MB inflated with raw deflate to 4.46 MB: `LASF` 1.4, header 375 B,
  1 VLR (`LASF_Spec` 4, extra bytes, no CRS VLR), point format 6, 38 B/record, scale 0.01,
  bbox 389000–389999.99 / 5816000–5816999.99, z −23.86…178.69, **13 848 481 points** (≈ 13.8 pts/m²);
  classes in the first 117 k points: 2 ground 60 172, 5 high veg 38 064, 4 medium 11 481, 3 low 7 099,
  0 unclassified 516, 7 noise 5 — **no class 6 (buildings)**.
- Per-area use is feasible without a one-off 248 GB download: read the directory (≤ 19 KB), fetch the
  member for each km² (~230 MB compressed, ~525 MB raw), stream-inflate and bin points. Needed: ZIP64 in
  `zip_range.rs` (today ZIP32 only), Deflate64 for Nordost/Südost (the `zip` crate's pure-Rust `deflate64`
  feature, **UNVERIFIED** in freemapper's pinned version), a streaming reader (the member is too big to
  buffer twice), LAS 1.4 PDRF 6 via the `las` crate already in the tree.
- What it adds: a **classified** nDSM template (vegetation classes 3–5 only, without parked cars and
  without the image-matching smoothing of the bDOM) at 0.5 m, and a "structures" raster (class 0/1 above
  ground: walls, fences, cars, buildings). Detecting walls (515), steps (532), terrain steps (201/104) or
  hedges (410.1) as vector candidates from 10–14 pts/m² is research work. Survey date 2021 (Feb/Mar,
  leaf-off — good for walls under trees).
- Disk constraint (user, 2026-11-08: about 35 GB free, so no 248 GB bulk download): the checks the user asked
  for all pass. `Accept-Ranges: bytes` is set. The central directory sits at the end and was read by
  range (ZIP32 for Nordost, ZIP64 for Mitte). Every member has a known offset and size: 1 km tiles of about
  10–250 MB compressed (Mitte averages ~230 MB, edge tiles in Nordost 10–214 MB). They are not stored:
  they are deflated (method 8) or Deflate64 (method 9), and they are LAS, not LAZ. A per-area run reads
  one member per km² by range and inflates it as a stream. A typical sprint area of 1–2 km² transfers
  0.25–0.5 GB, and only the derived rasters (a few MB) need to stay on disk. So the cloud is **usable on
  this machine per area**. It still stays out of plans 1 and 2 (priority 9).
- Rating: benefit medium (templates; ALKIS lines already give walls, stairs and embankments as vectors),
  effort high (ZIP64 + Deflate64 + streaming + 0.5–1 GB transfer per km²). Priority 9: after plans 1 and 2.
  A one-off local download with a tile index is not suitable for lay users.

## 11. Others checked

- **LoD2** `https://gdi.berlin.de/data/a_lod2/atom/0.atom` (926 `LoD2_EEE_NNNN.zip`, 1 km, `application/x-gmz`,
  `<updated>2026-03-26`). `LoD2_389_5816.zip`: 3 292 655 B, Last-Modified 2026-06-01, one CityGML 1.0 file
  (29 MB) with 705 `bldg:Building`, 1 271 `BuildingPart`, `measuredHeight`, functions incl. `51009_1610`
  (Überdachung, 77) and `51009_1700`. Solids reach the ground, so no passages; canopies and building
  outlines are already in ALKIS. Value: heights only.
- **ATKIS Basis-DLM** `…/wfs/atkis`: 90 types; `atkis:b05_ax_wegpfadsteig_l` 148 "Fußweg" centre lines in
  Viktoriapark (`beg` 2024-12-09 / 2026-03-06) — the park paths; OSM already has them. `a11_…_p` monuments
  with names ("Ludwig Uhland").
- **Grünanlagen** `…/wfs/gruenanlagen` (`gruenanlagen:gruenanlagen`, `gruenanlagen:spielplaetze` 1 886):
  outlines with name and `widmung`; Viktoriapark playground "Viktoriapark, Spiel Bolz". Could rule public
  parks *out* of the 520 candidates; low benefit (ALKIS already says Grünanlage).
- **Kleingartenbestand** `…/wfs/kleingarten`, **Friedhofsbestand** `…/wfs/friedhofsbestand`: 0 in the boxes;
  ALKIS `bezfkt` already carries Kleingarten and Friedhof.
- **Fußgängernetz** (2014–2019), **Detailnetz** (`b_bauwerke`: e.g. "Viktoriapark-Brücke"),
  **Radverkehrsanlagen**, **Parken im Straßenraum 2023/2025**, **Fußgängerüberwege**: network/traffic data;
  no ISSprOM gain over OSM and ALKIS.
- **Umweltatlas** `ua_versiegelung_2021`, `ua_gruenvolumen_2020`, `ua_vegetationshoehen_2020`,
  `ua_flaechennutzung`: block aggregates (`schluessel`, `vg_2021`, `mean_vegh_bl2020` …), not objects.
  `ua_vegetationshoehen_2020` also has a citywide GeoTIFF (ATOM, "785 MB", see the old reference §6).
- **Gewässerkarte** `…/wfs/gewaesserkarte`: coarse water polygons (the whole Spree as one feature) and
  pipes; ALKIS water is better.
- **Denkmale** `…/wfs/denkmale`: heritage parcels/areas with a link; not map objects.

## 12. Licence and attribution

All services above: Datenlizenz Deutschland – Zero – 2.0 (capabilities `Fees`, CKAN `license_id`).
Attribution is optional; freemapper names Berlin data anyway. Suggested lines, added only when the layer
contributed objects or a template:
- `© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert` (exists; covers source 1 and 5)
- `Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0`
- `Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0`
- `Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert`
- `Geoportal Berlin / Öffentliche Beleuchtung, Zierbrunnen, Trinkwasserbrunnen, dl-de/zero-2-0`
- Not used: Grün Berlin tree layer (licence conflict, §5).

## 13. Risks

1. **Straßenbefahrung is 10–12 years old** (2014–2016). Streets were rebuilt since (26.7 % of ALKIS street
   area in the Alexanderplatz km is not covered). Only candidates, plus the green cut-out, and the
   currency tag must show the survey date.
2. **Undocumented codes** (material, wall height `hms`, bollard type `pot`, curb `bor`): no rule may depend
   on them until a code list is found.
3. **Candidate clutter**: 2 392 bollards, 1 375 trees, 1 246 curb lines in one km². Keep bollards and
   masts schoolyard-only; curbs and trees as separate, clearly named candidate layers if the OMAP
   candidates file grows too much (open question).
4. **Stair direction**: ALKIS stair polygons carry no direction; inference from DGM1 (1 m) fails on short
   flights (< 1 m rise). Mark guessed directions.
5. **522 as map object** changes spec §4.5 ("candidate 522"); needs an ADR. Overhangs/roofs over private
   courtyards are out of bounds anyway; the canopy sits below the 520 candidate.
6. **Kleingarten → 520** applies to every state whose land use has `bez = Kleingarten`; check the other
   states' fixtures and tests.
7. **Server load**: about 8–12 extra WFS requests per area (each paged); the server drops ~5 % of requests.
8. **Grün Berlin tree licence** conflict (by vs zero).
9. **ALS**: Deflate64 support and memory for 525 MB members; 2021 data.
10. Border areas: all these layers return nothing outside Berlin; route them like the ALKIS WFS
    (`AlkisRecord.regions` / Berlin region).

## 14. Open questions

1. Should ALKIS `Überdachung`/`Carport` really become 522 map objects (my recommendation), or stay
   candidates as spec §4.5 says?
2. Bauteil "Durchfahrt im Gebäude": candidate 522 only, or also cut the passage out of the 521 object?
3. Curbs: ISSprOM gives no curb symbol; is 501.1 (step or edge of paved area) the mapper's wish for kerbs,
   or should curbs be left out?
4. Tree threshold for 417 vs 418 (proposed crown ≥ 10 m), and whether street trees in rows belong in the
   candidates at all on sprint maps.
5. Is a code list for the Straßenbefahrung attributes obtainable (SenMVKU, VISS-Info@senmvku.berlin.de)?
6. Separate candidate templates per source (e.g. `templates/candidates-berlin.omap`) or one file?
7. ALS: worth a plan 3 for a classified nDSM template, given 0.5–1 GB transfer per km²?

## 15. Local samples

`dl/viktoriapark/{alkis,atkis,baum,gruen,misc,strassenbefahrung,ua}/*.json`, `dl/viktoriapark/k5.png`,
`dl/school1km/{alkis,baum,strassenbefahrung}/*.json`, `dl/lod2/LoD2_389_5816.zip`,
`dl/als/389_5816_head.deflate` (first 2 MB of the ALS member), `dl/bauwerkeflaechen_bezbwf_all.json`
(citywide `bezbwf` only). `probe/caps/` capabilities, `probe/pkg/` CKAN records, `probe/ckan*.json` searches,
`probe/*.atom`, `probe/als_*_list.txt`, `probe/sb_viktoriapark.txt`, `probe/misc_viktoriapark.txt`,
`probe/mapant_lidar.md`.
