# Niedersachsen Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Niedersachsen get orthophoto, terrain, nDSM, ALKIS buildings and land use and OSM candidates, in
EPSG:25832.

**Architecture:** A zone-32 per-area provider, LGLN (`Provider::Lgln`).
- Orthophoto from the WMS (pieces ≤ 4000 px).
- DGM1 and DOM1 from the WCS through the shared `WcsService` (Saxony-Anhalt, MV), per area.
- Dates from the STAC searches (newest item at the area's centre).
- ALKIS through the shared `alkis_wfs` reader.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/niedersachsen-reference.md` (§n). Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/ni/`.

## Rulings (from §13/§14)

- **Rasters per area by WMS/WCS, not COG windows.** The WCS values are byte-identical to the COGs (§5). The WMS
  pixels are the same as the tiles (§4). This saves a COG range reader and YCbCr conversion; the cost is about 3×
  slower orthophotos. COG window reads stay a later option.
- **nDSM = DOM1 − DGM1.** It is laser-based and cheap, and it keeps the crowns.
  - The DOM is often 5–9 years older than the orthophoto. freemapper warns with both dates.
  - bDOM20 (146 MB/km², leaf-off crowns lost) is not used.
- **Dates from STAC.**
  - The newest DOP item at the centre gives the flight date and the leaf state by month.
  - The newest DGM/DOM item at the centre gives the record date.
  - The STAC answers are not cached beyond the project record.
- **The OSM extract `niedersachsen-latest` includes Bremen.** Bremen gets its own region later.
  Until then, an area in Bremen is "outside the supported states".

## Global Constraints

- **Region:**
  - `Region::Niedersachsen` (serde `"niedersachsen"`).
  - Boundary `assets/regions/niedersachsen.geojson`, from the scratchpad `ni/niedersachsen.geojson`. It is valid,
    with Bremen and Bremerhaven as holes, and `tests/region_assets.rs` checks it.
  - CRS `Utm32`; provider `Provider::Lgln` (serde `"lgln"`, `per_area()` true); object ids `DENI…`.
  - OSM extract: `europe/germany/niedersachsen-latest.osm.pbf` (510 MB).
- **Orthophoto (§4.2):**
  - Source: `https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms`, layer `ni_dop20`, `CRS=EPSG:25832`,
    `max_px` 4000.
  - Flight date: the STAC `https://dop.stac.lgln.niedersachsen.de/search?bbox=<lon/lat>` newest item `datetime`
    at the centre.
- **DGM1/DOM1 (§5, §6):**
  - Service: WCS `https://opendata.geoservices.lgln.niedersachsen.de/{dgm_wcs,dom_wcs}`, coverages `ni_dgm1` and
    `ni_dom1`, Float32, NoData −9999.
  - A **HTTP 200 HTML answer** ("MapServer Message") is an error.
  - Dates come from STAC `dgm.stac…` / `dom.stac…` (newest item at the centre).
- **ALKIS (§8):**
  - `WfsService` `LGLN`: `https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach`,
    `ave:GebaeudeBauwerk`, `ave:NutzungFlurstueck`, `Utm32`, `square_bez: Some("ohne Unterart (Niedersachsen)")`.
  - `COUNT` must be ≤ 10000; without `COUNT` the answer is cut silently.
  - Names are trimmed.
- **Attribution (§9):** `© GeoBasis-DE/LGLN <year>, Daten geändert, CC BY 4.0`, plus the licence link (the SH/MV
  mechanism).

## Review Focus

1. **Hamburg border (Harburg) and Bremen border:**
   - areas outside Niedersachsen are handled correctly (Bremen = outside, not Niedersachsen);
   - both ALKIS and both OSM extracts are read where applicable;
   - there are no duplicates.
2. **WCS:** an HTML answer with status 200 is an error. Areas up to 4 km work.
3. **STAC:** several vintages per cell; the newest is chosen. Paging works, and an empty answer gives no date and a
   warning.
4. **The DOM/DOP age gap** is warned with both dates.
5. **Existing regions are unchanged.**

---

### Task 1: Region, provider, extract
Tests:
- `hannover_school_is_niedersachsen_in_zone_32` (Kaiser-Wilhelm- und Ratsgymnasium, way/42796993)
- `bremen_is_not_niedersachsen`
- `harburg_border_has_both_regions`
- `ni_extract_urls`
- `ni_object_ids`

### Task 2: ALKIS
- `alkis_wfs::LGLN`, fixtures from the Hannover samples.
- Names are trimmed.

Tests:
- `lgln_buildings_parse`
- `lgln_land_use_from_nutzung_flurstueck`
- `lgln_underground_buildings_by_rellage`

### Task 3: WCS height models and STAC dates
`sources/lgln.rs`:
- the `WcsService` for DGM1/DOM1;
- STAC search (bbox in lon/lat, newest item);
- `fetch_model(…) -> HeightModel`.

Tests:
- `stac_picks_the_newest_item`
- `stac_pages_are_followed`
- `wcs_html_answer_is_an_error`
- `lgln_dgm1_from_the_wcs`

### Task 4: Orthophoto and project integration
- WMS + STAC flight date, `fetch_lgln_rasters`.
- The DOM/DOP age warning.
- Derive, attribution, estimate, selector + CSP, MCP text.

Tests:
- `ni_project_fetches_products`
- `ni_derive_writes_all_templates`
- `ni_attribution`
- `ni_estimate`
- `dom_older_than_dop_is_warned`

### Task 5: Docs, example, live run, review
- README row, manual chapters 2, 12, 14 and 15, glossary, spec §4.1.
- Live runs:
  - the Hannover school;
  - the Harburg border.
- Demo example, e.g. the Goslar old town or Lüneburg.
