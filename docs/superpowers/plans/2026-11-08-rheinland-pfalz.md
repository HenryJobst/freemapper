# Rheinland-Pfalz Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Rheinland-Pfalz get orthophoto, terrain, nDSM, ALKIS buildings and land use and OSM candidates,
in EPSG:25832.

**Architecture:** A zone-32 per-area provider, LVermGeo RP (`Provider::LvermGeoRp`).
- **Orthophoto:** from the WMS. The flight date comes from GetFeatureInfo.
- **DGM1/DOM1:** 1 km GeoTIFF tiles by plain HTTP. Their names carry the laser-scan year and come from the
  `Laserbefliegung` WFS layer. Tiles are cached by name.
- **ALKIS:** from the MapServer WFS (`ave:GebaeudeBauwerk`, `ave:Nutzung`) through the shared `alkis_wfs` reader.
  The reader learns `aktualit` inside `gml:timePosition`.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/rheinland-pfalz-reference.md` (§n). Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/rp/`.

## Rulings (from §13/§14)

- **The orthophoto comes from the WMS.** The tiles are lossless JPEG 2000 of 105–155 MB, with no index.
  - The GetFeatureInfo `erstellung` date is taken as the flight date. All 5,265 blocks fall in May–September.
  - It is labelled "Aktualität (Erstellung)" in the record, and the ADR names the inference.
- **Tile names come from the `Laserbefliegung` WFS** (`blattschnitt_rp.fcgi`). It gives the flight days per
  tile, and the year in the name follows from them.
  - Names are never guessed by trying years.
  - A tile the index names but the server lacks (404) is reported missing.
- **Origin hosts, not the registry proxy.** The registry returns HTML with status 200 on errors.
- **`ave:Nutzung` is the land-use type,** because NutzungFlurstueck does not exist here. It carries
  `nutzart`/`bez`, and `;`-joined names are kept as they are.
- **Provider name `LvermGeoRp`,** because `LvermGeo` is Saxony-Anhalt.

## Global Constraints

- **Region:**
  - `Region::RheinlandPfalz` (serde `"rheinlandpfalz"`).
  - Boundary `assets/regions/rheinland-pfalz.geojson`, from the scratchpad `rp/rheinland-pfalz.geojson`. It is
    valid and has negative 25833 eastings; `tests/region_assets.rs` checks it.
  - CRS `Utm32`; provider `Provider::LvermGeoRp` (serde `"lvermgeorp"`, `per_area()` true); object ids `DERP…`.
  - OSM extract: `europe/germany/rheinland-pfalz-latest.osm.pbf` (275 MB). Its poly excludes Wiesbaden and
    Mainz-Kastel, which belong to Hesse.
- **Orthophoto (§4):**
  - Source: `https://geo4.service24.rlp.de/wms/rp_dop20.fcgi`, layer `rp_dop20`, `CRS=EPSG:25832`, `max_px` 4000.
  - Flight date: GetFeatureInfo `rp_dop20_info` (`erstellung`).
  - The WMS is clipped at the state border (white outside).
- **DGM1/DOM1 (§5, §6):**
  - Tiles: `https://geobasis-rlp.de/data/{dgm1,dom1}/current/tif/<name>.tif` (`…_1_rp_<year>.tif`).
  - Format: Float32 LZW with Predictor 2. NoData is in §5. DOM1 tiles from 2022–2024 have no TFW, so freemapper
    uses the GeoTIFF tags.
  - Index: WFS `https://geo5.service24.rlp.de/wfs/blattschnitt_rp.fcgi`, layer `Laserbefliegung`.
- **ALKIS (§8):**
  - `WfsService` `LVERMGEO_RP`: `https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi`, `ave:GebaeudeBauwerk`,
    `ave:Nutzung`, `Utm32`, `square_bez: Some("ohne Unterart (Rheinland-Pfalz)")`.
  - `parse_gml` reads `aktualit` inside `gml:timePosition`.
  - The spelling "Unland, Vegetationslose Fläche" maps to the LGB form if the build rules use it.
- **Attribution (§9):** `©GeoBasis-DE / LVermGeoRP<year>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]`.

## Review Focus

1. **Hesse border (Mainz/Wiesbaden):**
   - both ALKIS services and both OSM extracts, without duplicates;
   - rasters from the centre's provider;
   - the WMS is white beyond the border, while the DGM/DOM cover it.
2. **The tile index:** a listed tile that is 404 is reported missing; no tile at all is an error; names are
   validated.
3. **GML `aktualit` in `gml:timePosition`** is read, and the other states' GML is unchanged.
4. **Negative 25833 eastings** (84.5 % of the state): detection, estimate and selector work.
5. **Existing regions are unchanged** (the `LvermGeo` vs `LvermGeoRp` names).

---

### Task 1: Region, provider, extract
Tests:
- `mainz_school_is_rp_in_zone_32` (the school from §0)
- `wiesbaden_stays_hessen`
- `rp_extract_urls`
- `rp_object_ids`

### Task 2: ALKIS (+ `gml:timePosition`)
Tests:
- `rp_buildings_parse`
- `rp_land_use_from_nutzung`
- `aktualit_in_time_position_is_read`
- `mainz_border_reads_both_states`

### Task 3: DGM1/DOM1 tiles by the Laserbefliegung index
Tests:
- `laserbefliegung_gives_names_and_dates`
- `rp_tiles_mosaic_into_the_area`
- `listed_but_missing_tile_is_reported`
- `predictor2_tile_decodes`

### Task 4: Orthophoto and project integration
Tests:
- `rp_project_fetches_products`
- `rp_derive_writes_all_templates`
- `rp_attribution`
- `rp_estimate`
- `flight_date_from_get_feature_info`

### Task 5: Docs, ADRs, changelog, example, live run, review
- README row and manual chapters 2, 12, 14 and 15 (unslop + humanizer), glossary, spec §4.1.
- ADRs for the rulings above, and a changelog entry.
- Live runs:
  - the Mainz school;
  - the Mainz/Wiesbaden border.
- Demo example, e.g. the Trier old town and Porta Nigra, or the Koblenz Deutsches Eck.
