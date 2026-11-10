# Baden-Württemberg Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Baden-Württemberg get an orthophoto, terrain, nDSM, ALKIS buildings and land use, and OSM candidates,
in EPSG:25832.

**Architecture:** A zone-32 per-area provider, LGL-BW (`Provider::LglBw`).
- **Orthophoto:** from the WMS. The flight date comes from the Bildflugkacheln WFS per 1 km tile.
- **DGM1/nDOM1:** members of the 2 km zips, read by HTTP range with `zip_range`. The zip key is computed, and member
  names (with their year) come from the zip's central directory. DGM1 is XYZ text, nDOM1 a Float32 GeoTIFF; the dates
  come from the CSV member of each tile.
- **ALKIS:** from the open GeoServer WFS `WFS_LGL-BW_ALKIS` (`nora:v_al_gebaeude`, `nora:v_al_tatsaechliche_nutzung`)
  through the shared `alkis_wfs` reader. A `nora` schema maps its names to the `ave:` keys.
- **OSM:** the four Regierungsbezirk extracts, using the NRW mechanism.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/baden-wuerttemberg-reference.md` (§n; scratchpad copy `bw/BW-REFERENCE.md`). Live samples (untrusted,
fixtures only):
`<Arbeitsverzeichnis>`.

## Rulings (from §13/§14)

- **Land use comes from the ALKIS WFS `nora:v_al_tatsaechliche_nutzung`, not from `ln_bw.zip`.** The GPKG member is deflated (no range access, 2.39 GB to download) and uses the AdV LN model.
- **Buildings come from the same WFS (`nora:v_al_gebaeude`), not from the OGC API beta or the Gemarkung Shape zips.** It has the same data, an open dl-de/by-2-0 licence, and is not beta.
- **`bez` is `unterart_name` only when `unterart_id` is set,** because without one the name repeats `objektname`.
- **`gebnutzbez` is the constant "Gebäude",** because the view holds AX_Gebaeude only. Underground comes from `lage_oberflaeche_name`.
- **The orthophoto comes from the WMS, dated by `befliegungsdatum` of the centre tile.** The zip TIFFs are 63 MB uncompressed per km² and can be a year older.
- **WMS pieces are capped at 4000 px.** The server returns a blank 200 JPEG from 6144 px on.
- **An all-white piece inside the BW polygon is an error.**
- **The nDSM is the ready-made nDOM1, not DOM1 − DGM1.** It is leaf-on like the DOP, one file instead of two, and already clamped.
- **A tile missing from a zip's directory, and a 404 zip, mean "outside BW" when the tile lies outside the BW polygon.** Inside the polygon they are reported missing.
- **Partial XYZ tiles are accepted.** Absent rows become NaN; a point off the grid is still an error.
- **OSM is read per Regierungsbezirk** (four extracts, 128–212 MB) instead of the 653 MB state extract, as in ADR 0028.
- **Provider name `LglBw` (serde `"lglbw"`), attribution `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten bearbeitet, <year>)`.**

## Global Constraints

- **Region:**
  - `Region::BadenWuerttemberg` (serde `"badenwuerttemberg"`).
  - Boundary `assets/regions/baden-wuerttemberg.geojson`, from the scratchpad `bw/baden-wuerttemberg.geojson` (6,561
    vertices, 16 parts incl. Büsingen, 1 hole for the Hessian enclave, valid in shapely). `tests/region_assets.rs` checks it.
  - CRS `Utm32`; provider `Provider::LglBw` (`per_area()` true); object ids `DEBW…` (`of_object_id` arm `"DEBW"`).
  - 16.8 % of the state has negative 25833 eastings (Freiburg school E −34.0 km).
- **OSM (§11):**
  - `europe/germany/baden-wuerttemberg/{freiburg,karlsruhe,stuttgart,tuebingen}-regbez-latest.osm.pbf` + `-updates/state.txt`.
  - Four poly assets from Geofabrik's `.poly`, converted to 25833 and simplified to 25 m.
- **Orthophoto (§4):**
  - `https://owsproxy.lgl-bw.de/owsproxy/ows/WMS_LGL-BW_ATKIS_DOP_20_C`, layer `IMAGES_DOP_20_RGB`, `CRS=EPSG:25832`,
    `FORMAT=image/jpeg`, `max_px` 4000. White outside BW.
  - Date: `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ATKIS_DOP_20_Bildflugkacheln_Aktualitaet`,
    `TYPENAMES=verm:v_dop_20_bildflugkacheln`, key `dop_kachel = "32" + EEE + NNNN`, field `befliegungsdatum`
    (`YYYY-MM-DD`). Text fields are blank-padded.
- **DGM1/nDOM1 (§2.1, §5, §6):**
  - Zip key `(E − (E+1) mod 2, N − N mod 2)` (odd E, even N).
  - URLs `https://opengeodata.lgl-bw.de/data/dgm/dgm1_32_{Ez}_{Nz}_2_bw.zip` and `…/data/ndom1/ndom1_32_{Ez}_{Nz}_2_bw.zip`.
  - Members `dgm1_32_{E}_{N}_1_bw_{YYYY}.xyz` (+ `.csv`) and `ndom1_32_{E}_{N}_1_bw.tif` (+ `.csv`). Members are
    deflated: ≤ 29 MB XYZ, ≤ 4.2 MB TIFF, inside `MAX_MEMBER_BYTES`.
  - XYZ: `E N Z`, cell centres, NW first, CRLF; partial rows possible at the border.
  - TIFF: Float32 in two layouts, uncompressed 128 px tiles (NoData −3.4028235e38) or LZW predictor 3 in 512 px tiles
    (NoData −9999). `NODATA_BELOW` −9998 catches both.
  - CSV dates are `YYYY-MM-DD` or `YYYY/MM/DD`. DGM currency = `Fortfuehrung`; nDSM currency = `bDOM20_Aktualitaet`.
- **ALKIS (§8):**
  - `WfsService` `LGL_BW`: `https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS`, `nora:v_al_gebaeude`,
    `nora:v_al_tatsaechliche_nutzung`, `Utm32`, `square_bez: Some("ohne Unterart (Baden-Württemberg)")`.
  - The schema has a geometry element `geom` and a sort key `gml_id`.
  - Mapping: `gml_id→oid`, `objektname→nutzart`, `unterart_name→bez` (only with `unterart_id`), `eigenname→name`,
    `beginn[..10]→aktualit`, `gebaeudefunktion_name→funktion`, `lage_oberflaeche_name→rellage`, `gebnutzbez="Gebäude"`.
  - A 200 `text/html` "Request Rejected" (the WAF) is an error.
- **Attribution (§9):** `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten bearbeitet, <year>)`.

## Review Focus

1. **The `nora` mapping in `alkis_wfs`:** geometry under `geom`, dedup and `SORTBY` by `gml_id`, the `bez` rule, the
   constant `gebnutzbez`, underground via `lage_oberflaeche_name`. GML of every other state stays byte-for-byte unchanged
   (existing fixtures).
2. **WMS blank detection and size cap:** pieces ≤ 4000 px; an all-white piece inside BW is an error, white outside BW is
   not; no false alarm at the Rhine.
3. **Zip-member tiles:** computed zip key, directory lookup by name prefix, 404 zip / absent member outside vs inside BW,
   partial XYZ rows as NaN, empty nDOM zips, CRC check on every member.
4. **Rhine border (Mannheim/Ludwigshafen) and Hesse (Viernheim):** both ALKIS services and the needed OSM extracts are
   read, without duplicates; rasters come from the centre's provider; the RP/BW asset overlap (p99 ≈ 31 m) does not
   double-count. Ulm/Neu-Ulm gets the beyond-coverage warning for the Bavarian part.
5. **Existing regions are unchanged:** NRW district extracts still work after adding BW districts; `LvermGeo`, `LvermGeoRp`
   and `LglBw` stay distinct; the 25833 path for negative eastings.

---

### Task 1: Region, provider, extracts
Tests:
- `stuttgart_school_is_bw_in_zone_32` (Eberhard-Ludwigs-Gymnasium, E 511852 N 5403836)
- `freiburg_school_has_negative_25833_easting`
- `ludwigshafen_stays_rp`
- `viernheim_stays_hessen`
- `neu_ulm_is_beyond_coverage`
- `bw_district_extract_urls`
- `bw_object_ids`

### Task 2: ALKIS through the `nora` schema
Tests:
- `bw_buildings_parse` (fixture: school box, 30 buildings)
- `bw_land_use_from_tatsaechliche_nutzung` (19 objects at the school; `Öffentliche Zwecke` as `bez`)
- `bez_only_with_unterart_id`
- `tiefgarage_is_underground`
- `waf_rejection_is_an_error`
- `other_states_gml_unchanged`

### Task 3: DGM1/nDOM1 from zip members
Tests:
- `zip_key_is_odd_east_even_north`
- `member_names_come_from_the_directory`
- `partial_xyz_rows_are_nan`
- `empty_ndom_zip_is_missing_inside_bw_only`
- `ndom_tiled_uncompressed_and_predictor3_decode`
- `csv_dates_in_both_formats`

### Task 4: Orthophoto and project integration
Tests:
- `flight_date_from_bildflugkacheln`
- `white_wms_piece_inside_bw_is_an_error`
- `bw_project_fetches_products`
- `bw_derive_writes_all_templates`
- `bw_attribution`
- `bw_estimate`

### Task 5: Docs, ADRs, changelog, example, live run, review
- Copy `BW-REFERENCE.md` to `docs/regions/baden-wuerttemberg-reference.md`, and update the BW row and section of
  `states-survey.md` (WFS found, zips deflated, DGM1 is XYZ).
- README row and manual chapters 2, 12, 14 and 15 (unslop + humanizer), glossary, spec §4.1.
- ADRs for the rulings above (land use from the WFS, WMS date from Bildflugkacheln, zip-member tiles, nDOM1,
  district extracts), and a changelog entry (DE + EN).
- Live runs:
  - the Stuttgart school;
  - the Freiburg school (negative 25833 easting);
  - the Mannheim/Ludwigshafen border.
- Demo example, e.g. Heidelberg old town and castle, or Freiburg Münsterplatz.
