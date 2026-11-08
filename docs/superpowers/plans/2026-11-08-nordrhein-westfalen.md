# Nordrhein-Westfalen Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in NRW get orthophoto, terrain, nDSM, ALKIS buildings and land use and OSM candidates, in
EPSG:25832.

**Architecture:** A zone-32 per-area provider, Geobasis NRW (`Provider::GeobasisNrw`).
- **Orthophoto:** WMS pieces of ≤ 4000 px, with the flight date by GetFeatureInfo.
- **DGM1/DOM1:** 1 km GeoTIFF tiles by plain HTTP. The names come from the product's `*_meta.zip` CSV, cached per
  tile.
- **ALKIS:** through the shared `alkis_wfs` reader.
- **OSM:** from the five Regierungsbezirk extracts (137–225 MB each) instead of the 917 MB statewide one.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/nordrhein-westfalen-reference.md` (§n). Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/nrw/`.

## Rulings (from §13)

- **OSM by Regierungsbezirk:**
  - The five Geofabrik extracts `nordrhein-westfalen/<rb>-regbez-latest.osm.pbf` (Arnsberg, Detmold, Düsseldorf,
    Köln, Münster).
  - Each has a bundled boundary: the Geofabrik `.poly`, converted to 25833 and simplified to 25 m
    (topology-preserving and valid).
  - An area reads every extract whose boundary lies within 100 m, the same rule as for states.
- **Tile names come from `dgm1_meta.zip` / `dom1_meta.zip`.** Each is a CSV with one row per tile, giving the exact
  name and date. Names are never computed and never taken from the directory listing, which holds stale `_2023`
  duplicates.
- **The orthophoto comes from the WMS,** because the tiles are JPEG 2000 only.
- **nDSM = DOM1 − DGM1** (laser, same date). nDOM50 is not used.

## Global Constraints

- **Region:**
  - `Region::NordrheinWestfalen` (serde `"nordrheinwestfalen"`); boundary
    `assets/regions/nordrhein-westfalen.geojson`, from the scratchpad
    `probe/boundary/nordrhein-westfalen_simplified25m.geojson`.
  - The boundary has negative eastings in 25833 (down to −139.6 km). It must be valid; `tests/region_assets.rs`
    checks that.
  - CRS `Utm32`, provider `Provider::GeobasisNrw` (serde `"geobasisnrw"`, `per_area()` true), object ids `DENW…`.
- **Orthophoto (§4):** `https://www.wms.nrw.de/geobasis/wms_nw_dop`, layer `nw_dop_rgb`, `CRS=EPSG:25832`, `max_px`
  4000. The flight date comes from GetFeatureInfo on `nw_dop_utm_info` (`Bildflugdatum`).
- **DGM1/DOM1 (§5, §6):**
  - Tiles: `https://www.opengeodata.nrw.de/produkte/geobasis/hm/dgm1_tiff/dgm1_tiff/<Kachelname>.tif` and the
    `dom1_tiff` sibling.
  - Format: Float32 LZW, NoData −9999, GeoKeys, no TFW.
  - Index: `<product>_meta.zip`, with the CSV `Kachelname` and date per tile.
- **ALKIS (§8):**
  - `WfsService` `GEOBASIS_NRW`: `https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht`, `ave:GebaeudeBauwerk`,
    `ave:NutzungFlurstueck`, `Utm32`, `square_bez: Some("ohne Unterart (Nordrhein-Westfalen)")`.
  - WFS errors are a bare HTTP 400.
- **Attribution (§9):** `Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) <year>, dl-de/zero-2-0`.

## Review Focus

1. **Areas across a Regierungsbezirk border** read both extracts, without duplicate OSM features.
2. **Hesse border:** both ALKIS services; rasters from the centre's provider.
3. **Tile index:** a tile that is in the CSV but missing on the server (404) is reported as missing. A stale listing
   name is never used.
4. **Negative 25833 eastings** in the west (Aachen): detection, estimate and selector work.
5. **Existing regions are unchanged.**

---

### Task 1: Region, provider, Regierungsbezirk extracts
Tests:
- `koeln_school_is_nrw_in_zone_32` (Hildegard-von-Bingen-Gymnasium, way/56752699)
- `aachen_negative_eastings_is_nrw`
- `nrw_extracts_by_regierungsbezirk`
- `regbez_border_reads_two_extracts`
- `nrw_object_ids`

### Task 2: ALKIS
- `alkis_wfs::GEOBASIS_NRW`, fixtures from the Köln samples.

Tests:
- `nrw_buildings_parse`
- `nrw_land_use_from_nutzung_flurstueck`
- `hesse_border_reads_both_states`

### Task 3: DGM1/DOM1 tiles and the meta index
`sources/geobasis_nrw.rs`:
- `fetch_model(…) -> HeightModel`;
- the meta-zip CSV index (cached per release);
- tiles cached by name.

Tests:
- `meta_csv_gives_names_and_dates`
- `tiles_mosaic_into_the_area`
- `listed_but_missing_tile_is_reported`
- `nrw_dgm1_decodes`

### Task 4: Orthophoto and project integration
- WMS + flight date, `fetch_nrw_rasters`, derive, attribution, estimate, selector + CSP, MCP text.

Tests:
- `nrw_project_fetches_products`
- `nrw_derive_writes_all_templates`
- `nrw_attribution`
- `nrw_estimate`

### Task 5: Docs, example, live run, review
- README row, manual chapters 2, 12, 14 and 15, glossary, spec §4.1.
- Live runs:
  - Köln school;
  - an Aachen area;
  - a Regierungsbezirk border.
- Demo example, e.g. the Aachen cathedral and old town, or Münster.
