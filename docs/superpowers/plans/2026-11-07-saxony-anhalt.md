# Saxony-Anhalt Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Saxony-Anhalt and across its borders get orthophoto, terrain, nDSM, ALKIS buildings and land use,
and OSM candidates, in the state's own CRS (EPSG:25832).

**Architecture:** freemapper learns the second UTM zone.
- Every project gets the CRS of its raster region: EPSG:25832 for Saxony-Anhalt (and later Thuringia and Hesse),
  EPSG:25833 elsewhere.
- Rasters stay on their native pixel grid. Vectors from a service in the other zone (LGB, Berlin, GeoSN ALKIS near
  the border) are reprojected through latitude/longitude.
- Region detection stays in EPSG:25833, where the boundaries are stored; the area's 25833 box is computed from its
  latitude/longitude polygon.
- LVermGeo rasters are fetched per area, like the GeoSN orthophoto:
  - orthophoto from the WMS;
  - DGM1 and DOM1 from the WCS (one request each, ≤ 4000 px = 4 km).

  There are no tile records for Saxony-Anhalt.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1 and the later-work list.
Verified sources (2026-11-07): `docs/regions/saxony-anhalt-reference.md` (cited as §n).

## Global Constraints

- **CRS:**
  - `crs::Crs::{Utm32, Utm33}`: EPSG 25832/25833, central meridian 9°/15°; forward, inverse, convergence and
    scale factor per zone.
  - OCAD grid code `i8007`/`i8008`; the OOM `projected_crs` parameter is the EPSG code.
  - `ProjectFile.crs` (serde `crs`, default 25833, so older projects load unchanged).
  - The project bbox, the stored ALKIS files and records, the rasters and the georeferencing are all in the project
    CRS.
  - A change of map area that changes the CRS drops the stored tile, ALKIS, OSM and raster records, with a warning.
- **Region:**
  - `Region::SaxonyAnhalt` (serde `"saxonyanhalt"`), boundary `assets/regions/saxony-anhalt.geojson` (DE15,
    simplified 25 m, in 25833).
  - Provider `Provider::LvermGeo`, CRS `Utm32`, object ids `DEST…`.
  - The raster region decides the project CRS.
- **LVermGeo orthophoto (§4.4):**
  - `https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest`, layer
    `lsa_lvermgeo_dop20_2`, `CRS=EPSG:25832`, JPEG.
  - Pieces of at most 4000 px, the same stitching as GeoSN (the WMS code is shared, parametrised by service).
  - Flight date from the tile index WFS `atkis_geobasis_bestand:truedop_kacheln` (`BFDATUM`, GeoJSON) at the
    centre (§7).
  - The orthophoto is reused when the flight date is unchanged.
- **LVermGeo DGM1/DOM1 (§5, §6.1):**
  - WCS 2.0.1 `https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1` and
    `https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1`, coverage `Coverage1`, `FORMAT=image/tiff`,
    `SUBSET=x(E1,E2)&SUBSET=y(N1,N2)` on whole metres covering the bbox.
  - The answer is `multipart/related`: the TIFF part is taken out, and its grid comes from ModelTiepoint and
    ModelPixelScale (no GeoKeys).
  - Stored as `data/dgm1.tif` and `data/dom1.tif`, each with a record `WcsRecord { fetched, bbox }`.
  - The DOM date comes from `bdom_aktualitaet_zyklus` (`BFDATUM`).
  - nDSM = DOM1 − DGM1 (`fill_from_dsm`).
- **LVermGeo ALKIS (§8):**
  - WFS `https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest`, types
    `ave:GebaeudeBauwerk` and `ave:Nutzung`.
  - `SRSNAME`/`BBOX` in EPSG:25832.
  - GML 3.2 is read by the shared WFS reader (local names, default namespace). Errors come with status 200.
  - Squares without `bez` get `bez = "ohne Unterart (Sachsen-Anhalt)"`. `rellage` works as in Brandenburg.
  - The GeoSN WFS code becomes `sources/alkis_wfs.rs` with a `WfsService` per state.
  - Each ALKIS source reports its CRS, and the merged fetch reprojects into the project CRS.
- **OSM:** `europe/germany/sachsen-anhalt-latest.osm.pbf` (≈177 MB), cached as `sachsen-anhalt_<date>.osm.pbf`.
- **Attribution:** `© GeoBasis-DE / LVermGeo ST <year>, dl-de/by-2-0, Daten geändert` once LVermGeo data was
  fetched (the year is the fetch year).

## Review Focus

1. **Brandenburg, Berlin and Saxony projects are unchanged:**
   - EPSG 25833 is written to OOM and OCAD;
   - golden files are unchanged;
   - old `freemapper.toml` files without `crs` load.
2. **Zone-32 georeferencing is right:**
   - a 25832 map point round-trips to latitude/longitude and back;
   - declination/grivation use the zone-32 convergence;
   - OCAD writes `i8007`.
3. **A Saxony-Anhalt/Brandenburg border area (zone change):**
   - LGB ALKIS in 25833 is reprojected into the 25832 project;
   - buildings lie on the orthophoto;
   - there are no duplicates.
4. **WCS answers:**
   - multipart with a TIFF → a raster on the right grid;
   - an HTML/XML error with status 200 or 400 → a clear error, retried for 5xx/resets only.
5. **Moving an area across the zone line** drops the stale records, so nothing in the old CRS is reused.

---

### Task 1: Two UTM zones
`crs.rs`:
- `Crs { Utm32, Utm33 }` with `epsg()`, `forward(LatLon)`, `inverse(ProjectedPoint)`, `convergence_deg(LatLon)`
  and `scale_factor(LatLon)`.
- `utm33_*` stay as thin wrappers.

`georef.rs`:
- `Georeferencing` carries `crs`.
- OOM writer: the EPSG code.
- OCD writer: `i8007`/`i8008`.

Tests:
- `zone32_round_trip_and_reference_points` (Halle Latina: 25832 E 706451 N 5706913 ↔ 25833 E 289795 N 5707067
  through lat/lon)
- `zone32_convergence_is_positive_west_of_the_meridian`
- `georef_writes_the_project_epsg`
- `ocad_grid_code_follows_the_zone`
- existing golden tests unchanged

### Task 2: Saxony-Anhalt region and project CRS
`region.rs`, `area.rs`, `project.rs`:
- `Region::SaxonyAnhalt`, `Provider::LvermGeo`, `Region::crs()`.
- `MapArea::projected_bbox_in(crs)`.
- `ProjectFile.crs`, `ProjectFile::bbox()` in the project CRS.
- `init_project`/`set_map_area` pick the CRS from the raster region of the area's 25833 box and drop stale records
  on a change.
- `check_area` and the region tests use the 25833 box.

Tests:
- `halle_is_saxony_anhalt_in_zone_32`
- `old_project_without_crs_is_25833`
- `moving_across_the_zone_line_drops_records`
- `brandenburg_project_stays_25833`

### Task 3: Shared ALKIS WFS and reprojection
`sources/alkis_wfs.rs` (from `geosn_alkis.rs`), `sources/alkis.rs`:
- `WfsService { name, url, buildings_type, land_use_type, crs, square_bez }` for GeoSN and LVermGeo.
- The merged fetch takes `(bbox_25833, project_crs)` and reprojects each source's features into the project CRS.
- Before relying on `SORTBY`, check that the LVermGeo WFS accepts it (live; without it, paging relies on hits).

Fixtures `tests/fixtures/saxony-anhalt-buildings.gml`, `saxony-anhalt-land-use.gml` (Latina, live).

Tests:
- `lvermgeo_buildings_parse` (Bauteil + rellage Tiefgarage)
- `lvermgeo_land_use_squares_stay_candidates`
- `exception_with_status_200_is_an_error`
- `border_features_are_reprojected_into_the_project_crs`
- the existing GeoSN tests

### Task 4: LVermGeo rasters
`sources/lvermgeo.rs`, `sources/wms.rs` (from `geosn.rs`):
- `wms::fetch_orthophoto(service, …)`.
- `lvermgeo::wcs_url(product, bbox)`; `multipart_tiff(bytes, content_type) -> Vec<u8>`; `fetch_wcs(…, out)`.
- `flight_date(bbox)` / `dom_date(bbox)` from the tile index.
- `derive`: `read_geotiff_raster(path) -> Raster<f32>` from the tiepoint/scale tags.

Tests:
- `wcs_url_subsets_whole_metres`
- `multipart_answer_yields_the_tiff`
- `wcs_error_page_is_an_error`
- `geotiff_tags_give_the_grid`
- `tile_index_gives_the_flight_date`

### Task 5: Project integration
- `fetch_lvermgeo_rasters`:
  - WMS orthophoto (reused when the flight is unchanged);
  - WCS DGM1, required;
  - WCS DOM1, a template, so a failure only warns.
- Records `WcsRecord`.
- `derive`:
  - LVermGeo branch: DTM from `data/dgm1.tif`, nDSM = DOM1 − DGM1, orthophoto from the WMS file;
  - `NotFetched` and outdated checks per provider.
- OSM extract `SaxonyAnhalt`; attribution; estimate (WMS 8 MB/km², WCS 4 MB/km² each, extract 177 MB).
- Area selector: LVermGeo aerial layer + CSP; MCP text.

Tests:
- `saxony_anhalt_project_fetches_lvermgeo_products`
- `saxony_anhalt_derive_writes_all_templates`
- `saxony_anhalt_attribution`
- `saxony_anhalt_estimate`
- `zone32_map_is_georeferenced_in_25832` (OOM file and OCAD)
- `lgb_border_buildings_in_a_25832_project`

### Task 6: Docs, live run, acceptance
- README DE/EN, manual chapter 12 and the messages table, glossary (`Koordinatensystem`/`Zone`), spec §4.1.
- Live runs:
  - Latina August Hermann Francke, Halle (way/1098639307);
  - a border area with Brandenburg.
- Overlay of buildings on the orthophoto, nDSM, PDF and OCAD georeferencing.
