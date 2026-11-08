# Mecklenburg-Vorpommern Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Mecklenburg-Vorpommern get orthophoto, terrain, nDSM, ALKIS buildings and land use and OSM
candidates. The map stays in EPSG:25833, like Brandenburg.

**Architecture:** A zone-33 per-area provider, LAiV M-V (`Provider::LaivMv`).
- **Orthophoto:** from a WCS returning JPEG. This is a new orthophoto source type beside the WMS, chosen because the
  WMS carries a watermark.
- **DGM1 and DOM1:** from the WCS, as plain TIFFs. The LVermGeo ST WCS code is generalised for this, not copied.
- **Masking:** both height models are masked to the MV 2 km tile coverage, because they carry no NoData.
- **ALKIS:** from LGB's own OGC API product at the MV endpoint; the LGB reader is unchanged.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/mecklenburg-vorpommern-reference.md` (§n). Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/mv/`.

## Rulings (from §13)

- **Orthophoto:** the DOP WCS (`mv_dop20`, JPEG, pieces ≤ 4000 px), because the WMS carries a watermark. The
  256-colour palette is accepted for a template, and the manual mentions it.
- **nDSM = DOM1 − DGM1**, both from the WCS. The ready nDOM is older and is not used.
- **Masking:** cells outside the MV tile set are NaN. The tile set comes from `overview.php` (2 km tiles), cached per
  area. Sea and Bodden at 0.0 m inside the set are real and stay.
- **CRS stays 25833**, so no zone switch. A Brandenburg ↔ MV move is a provider change, which drops the records.

## Global Constraints

- **Region:**
  - `Region::MecklenburgVorpommern` (serde `"mecklenburgvorpommern"`).
  - Boundary `assets/regions/mecklenburg-vorpommern.geojson`, from the scratchpad `mv/mecklenburg-vorpommern.geojson`.
    It is valid; `tests/region_assets.rs` checks that.
  - CRS `Utm33`; provider `Provider::LaivMv` (serde `"laivmv"`, `per_area()` true); object ids `DEMV…`.
  - OSM extract: `europe/germany/mecklenburg-vorpommern-latest.osm.pbf` (129 MB).
- **Orthophoto:**
  - Source (§4): `https://www.geodaten-mv.de/dienste/inspire_oi_dop_wcs`, `COVERAGEID=mv_dop20`, `FORMAT=image/jpeg`,
    `SUBSET=x(…)&SUBSET=y(…)` on the 0.2 m grid.
  - Flight date: from `overview.php type[]=dop20` (`aktualitaet`), on the host
    `laiv.geodaten-mv.de/afgvk/_apps/dladownload/_ajax/`.
- **DGM1/DOM1 (§5, §6):**
  - Services: `https://www.geodaten-mv.de/dienste/dgm_wcs` (`mv_dgm`) and `…/dom_wcs` (`mv_dom1`).
  - Request: `FORMAT=image/tiff&GEOTIFF:COMPRESSION=LZW`, 25833. The answer is a plain TIFF.
  - The record dates come from `overview.php` (`dgm1`, `dom1`).
- **ALKIS (§8):**
  - Endpoint: OGC API `https://www.geodaten-mv.de/api/geobasis/alkis-vereinf/v1`, collections `gebaeude_bauwerk` and
    `nutzung_flurstueck`. Use the LGB URL builder with the MV endpoint, 25833.
  - Spellings are LGB's.
  - The WFS `alkis_wfs_einfach` is the fallback when the OGC API fails, like Hamburg's.
- **Attribution (§9):** `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)`, plus the licence link in
  ATTRIBUTION.txt (as for SH).

## Review Focus

1. **Brandenburg border:**
   - a map across it masks the MV rasters outside the MV tiles;
   - it reads both ALKIS products and both OSM extracts without duplicates;
   - Brandenburg areas are unchanged.
2. **A WCS answer that is an XML exception** (status 200 or 400) is an error. Transient errors are retried, and
   empty `overview.php` answers are retried.
3. **The JPEG WCS pieces** stitch without seams on the exact grid.
4. **Masking:** sea at 0.0 m inside the tile set stays 0.0. Outside the set is NaN, so there are no cliffs in the
   hillshade.
5. **Existing regions unchanged:** Saxony-Anhalt's WCS behaves as before after the generalisation.

---

### Task 1: Region, extract, provider
Tests:
- `rostock_school_is_mecklenburg_vorpommern_in_zone_33` (Innerstädtisches Gymnasium, way/402355834)
- `brussow_border_has_both_regions`
- `mv_extract_urls`
- `mv_object_ids`

### Task 2: ALKIS
- The LGB OGC API reader with the MV endpoint.
- The WFS fallback.
- Fixtures from the Rostock samples.

Tests:
- `mv_buildings_parse`
- `mv_land_use_parses`
- `mv_alkis_falls_back_to_the_wfs`

### Task 3: WCS generalisation and MV height models
- The ST WCS code is generalised to `WcsService { url, coverage, multipart: bool, … }`.
- MV: plain TIFF, LZW.
- Tile coverage from `overview.php` (cached), with masking.

Tests:
- `st_wcs_unchanged`
- `mv_wcs_answer_decodes`
- `cells_outside_the_tile_set_are_nan`
- `sea_inside_stays_zero`
- `wcs_exception_is_an_error`

### Task 4: Orthophoto from the DOP WCS
- `WcsOrthophoto` beside the WMS: JPEG pieces ≤ 4000 px on the exact grid, stitched.
- Flight date from `overview.php`.

Tests:
- `dop_wcs_pieces_stitch_on_the_grid`
- `dop_flight_date_from_overview`

### Task 5: Project integration
- `fetch_mv_rasters`, derive, attribution with the licence link, estimate, selector layer + CSP, MCP text.
- Brandenburg ↔ MV provider change.

Tests:
- `mv_project_fetches_products`
- `mv_derive_writes_all_templates`
- `mv_attribution`
- `mv_estimate`
- `brandenburg_to_mv_drops_records`

### Task 6: Docs, example, live run, review
- README row, manual chapters 2, 12, 14 and 15 (the palette orthophoto, seasons), glossary, spec §4.1.
- Live runs:
  - Innerstädtisches Gymnasium Rostock;
  - a Brüssow border area.
- Demo example, e.g. the Stralsund old town or Schwerin castle.
