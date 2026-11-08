# Schleswig-Holstein Adapter – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Maps in Schleswig-Holstein, above all along the Hamburg border, get orthophoto, terrain, nDSM, ALKIS
buildings and land use and OSM candidates, in EPSG:25832.

**Architecture:** The fourth zone-32 per-area provider, LVermGeo SH (`Provider::LvermGeoSh`).
- Orthophoto from the WMS; flight date from the DOP metadata WMS.
- DGM1 1 km GeoTIFFs from the INSPIRE ATOM data directory; tile names (with their per-tile year) come from the
  Downloadportal's bbox query.
- bDOM 0.2 m tiles come whole from the Downloadportal (no ranges). Each is reduced to 1 m and cached in that form.
- ALKIS from the VwG WFS through the shared `alkis_wfs` reader.
- The Downloadportal host omits its intermediate certificate, so a dedicated HTTP agent trusts the bundled D-TRUST
  chain for that host only.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1. Verified sources (2026-11-08):
`docs/regions/schleswig-holstein-reference.md` (§n). Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/sh/`.

## Rulings (the reference's open decisions, §13)

- **TLS (§13.1):**
  - Bundle the `D-TRUST BR CA 1-20-1 2020` intermediate and its root (`assets/tls/`) and use them as the only trust
    anchors of a second `ureq::Agent`.
  - That agent is used only for `geodaten.schleswig-holstein.de`.
  - Every other host keeps the default roots; nothing else is relaxed.
- **nDSM (§13.3):** the bDOM is fetched (≈ 105 MB and ≈ 90 s per km²).
  - Each tile is reduced to 1 m (the median of each 5 × 5 block, NaN ignored) and only the 1 m tile is cached.
  - The estimate states the download, and the surface warns instead of failing, as elsewhere.
- **DGM1 names (§13.6):** the Downloadportal AJAX `overview.php` for the area's bbox (≤ 200 objects; a 4 × 4 km area
  has ≤ 25 tiles) gives `kachel` and `datum`.
  - The GeoTIFF URL is `https://service.gdi-sh.de/INSPIRE_Atom/EL/data/<name>.tif`.
  - A 404 falls back to the XYZ through `massen.php`.
- **Orthophoto season (§13.4):** the record keeps the flight date. The manual explains the seasonal patchwork.

## Global Constraints

- **Region:**
  - `Region::SchleswigHolstein` (serde `"schleswigholstein"`), boundary `assets/regions/schleswig-holstein.geojson`
    (from the scratchpad `probe/boundary/schleswig-holstein_simplified25m.geojson`; 25833).
  - CRS `Utm32`, provider `Provider::LvermGeoSh` (serde `"lvermgeosh"`, `per_area()` true), object ids `DESH…`.
  - OSM extract `europe/germany/schleswig-holstein-latest.osm.pbf` (159 MB), cached as `schleswig-holstein_<date>.osm.pbf`.
  - The boundary shares ~131 km with Hamburg; a Norderstedt area is detected as SH, a Langenhorn one as Hamburg.
- **Orthophoto (§4.2):**
  - `https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD`, layer `sh_dop20_rgb`, `CRS=EPSG:25832`, `max_px` 4000.
  - Flight date via GetFeatureInfo on the DOP metadata WMS (`WMS_SH_MD_DOP`, §7).
- **DGM1 (§5.1, §5.2):** see the ruling. Float32 LZW with 256 px internal tiles (`decode_float_tiff` must read
  tiled TIFFs), NoData −9999, no TFW (grid from the tags).
- **bDOM (§6.1):**
  - `massen.php?file=bdom20nc_32_EEE_NNNN_1_sh_<year>.tif&id=6&live=<year>&km=<10 km block>&stack=32EEE_NNNN`,
    where the parameters come from `overview.php` (`link_data`).
  - Float32 uncompressed, 5000 × 5000, 256 px tiles, NoData NaN.
  - A `text/html` answer with status 200 is an error.
- **ALKIS (§8):**
  - `WfsService` `LVERMGEO_SH`: `https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD`, `ave:GebaeudeBauwerk`,
    `ave:NutzungFlurstueck`, `Utm32`, `square_bez: "ohne Unterart (Schleswig-Holstein)"`.
  - LGB spellings.
- **Attribution (§9):** `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)`; the project notes add
  `https://creativecommons.org/licenses/by/4.0/`.

## Review Focus

1. **Hamburg border:**
   - areas on both sides are detected correctly;
   - a cross-border map reads both ALKIS services and both OSM extracts without duplicates;
   - rasters come from the centre's provider.
2. **TLS:** only the Downloadportal host uses the bundled anchors. A certificate that does not chain to them fails,
   and no other host's verification changes.
3. **bDOM:** a 105 MB tile is streamed to disk, not held twice in memory. An HTML answer, NaN blocks and a missing
   tile are each handled with a message.
4. **DGM1:** a tile missing from the ATOM directory (404) uses the XYZ. A tiled LZW GeoTIFF decodes to the right grid.
5. **Existing regions are unchanged** (Hamburg above all: detection at the shared border).

---

### Task 0: Carry-overs from the Hesse review
- The LGV estimate marks tiles that are already cached. The release directory is known from `lgv::Model::release()`.
- A failed download of the HVBG metadata list (not only a failed parse) falls back to the kept list, with a warning.

Tests:
- `hamburg_estimate_marks_cached_tiles`
- `metadata_list_download_failure_uses_the_kept_list`

### Task 1: Region, extract, provider
Tests:
- `norderstedt_school_is_schleswig_holstein_in_zone_32` (Coppernicus-Gymnasium, way/26986780)
- `langenhorn_stays_hamburg`
- `schleswig_holstein_extract_urls`
- `schleswig_holstein_object_ids`

### Task 2: ALKIS
- `alkis_wfs::LVERMGEO_SH`, the merged-fetch arm, fixtures from the Norderstedt samples.

Tests:
- `lvermgeo_sh_buildings_parse`
- `lvermgeo_sh_land_use_from_nutzung_flurstueck`
- `cross_border_alkis_reads_both_states`

### Task 3: Downloadportal agent and tile lookup
- The pinned-anchor agent for `geodaten.schleswig-holstein.de` (the `Fetcher` routes by host).
- `overview.php` → tiles with name, year, date and `link_data`.

Tests:
- `portal_host_uses_the_bundled_anchors`
- `other_hosts_keep_the_default_roots`
- `overview_lists_dgm1_and_bdom_tiles`
- `too_many_objects_is_an_error`

### Task 4: DGM1 and bDOM
- DGM1 GeoTIFF per tile (tiled LZW), XYZ fallback on 404.
- bDOM tiles streamed to a temporary file, reduced to 1 m (5 × 5 median), cached.
- `fetch_model`-style API returning missing tiles and the centre date.

Tests:
- `tiled_lzw_dgm1_decodes`
- `missing_atom_tile_falls_back_to_xyz`
- `bdom_is_reduced_to_one_metre`
- `html_answer_is_an_error`

### Task 5: Orthophoto and project integration
- WMS, flight date, `fetch_sh_rasters`, derive, attribution (+ licence link in notes), estimate (bDOM download
  stated), selector layer + CSP (`dienste.gdi-sh.de`), MCP text.

Tests:
- `schleswig_holstein_project_fetches_products`
- `schleswig_holstein_derive_writes_all_templates`
- `schleswig_holstein_attribution`
- `schleswig_holstein_estimate`

### Task 6: Docs, example, live run, review
- README row, manual chapters 2, 12, 14 and 15 (seasonal orthophoto, bDOM older than the orthophoto in the 2025
  half), glossary, spec §4.1.
- Live runs:
  - Coppernicus-Gymnasium Norderstedt;
  - a Hamburg/Norderstedt border area.
- Demo example for `examples/`, e.g. Lübeck old town or Kiel Förde.
