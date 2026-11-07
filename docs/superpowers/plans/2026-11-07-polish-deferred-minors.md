# Polish: Deferred Minors from M2/M3a and M4a – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix every minor finding the final reviews of M2/M3a and M4a deferred (user, 2026-11-07: "die
Kleinigkeiten dann auch noch fixen"), each with a test that fails first.

**Architecture:** No new modules; small changes in `fetch.rs`, `raster.rs`, `derive/*`, `sources/alkis.rs`,
`build/buildings.rs`, `project.rs` and the CLI.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` (§4.7 reproducibility in particular).
The M4b review minors are added as a further task once that review is in.

## Global Constraints

- Every fix: failing test first, then the change, then `cargo test --workspace` and clippy green.
- `freemapper.toml` stays backward compatible (`#[serde(default)]` for new fields).
- Messages name the file or tile they are about and the command that resolves them.

## Review Focus

1. Offline runs after a partial online fetch (Task 1).
2. Corrupt data in the cache or the project (Tasks 1, 4).
3. Map areas larger than the guard (Task 2).
4. Changing map area or buffer after `fetch` (Tasks 2, 4).
5. Old projects (pre-M4a maps, toml files without the new records) (Task 4).

---

### Task 1: Fetch and cache robustness (M2/M3a minors 4, 5, 6, 11, 12)

**Files:** `crates/freemapper/src/fetch.rs`.

- [ ] Tests, then fixes:
  - `all_uncovered_tiles_reported_before_downloading`: tiles A (covered) and B, C (not covered) → error
    names both B and C, and no tile archive was requested.
    Change: resolve all currencies first; `FetchError::TilesNotCovered(Vec<TileId>)` replaces
    `TileNotCovered` ("tiles … are not covered by LGB Brandenburg data").
  - `offline_uses_older_cached_publication`: cache holds `<tile>_2026-01-26.zip`, cached CSV names
    2026-06-01 → offline fetch returns the older archive and emits
    `"warning: using cached 33421-5821 from 2026-01-26 (newer 2026-06-01 not downloaded)"`; the
    returned `currency.published` is 2026-01-26.
  - `corrupt_member_data_is_not_cached`: a zip whose raster member has a broken CRC → `CorruptArchive`.
    Change: `check_archive` reads both members to the end.
  - `stale_part_files_are_removed`: a leftover `<tile>_<date>.zip.part` disappears after a successful
    download of that tile; `.part` names carry the process id (`.zip.<pid>.part`) so two processes
    do not share one.
  - `offline_fetch_succeeds_from_cache` (test gap).
- [ ] Commit `fix(fetch): report uncovered tiles together, offline fallback, CRC check, .part hygiene`.

### Task 2: Raster and derive robustness (M2/M3a minors 7, 8, 9, 10)

**Files:** `crates/freemapper/src/raster.rs`, `crates/freemapper/src/derive/*.rs`,
`crates/freemapper/src/project.rs`.

- [ ] Tests, then fixes:
  - `misaligned_tiles_are_an_error`: `paste_with` returns `Result<(), RasterError>`;
    `RasterError::Misaligned` / `PixelSizeMismatch` instead of a panic.
  - `oversized_area_is_rejected`: `GridSpec::covering` callers check `MAX_RASTER_SIDE_PX = 20_000`;
    `DeriveError::AreaTooLarge { width, height }` ("map area too large for one template: …").
  - Orthophoto encodes from `mosaic.data.as_flattened()` (no copy; covered by the existing test).
  - `derive_detects_stale_tile_records`: after `buffer_m` grows so that a new tile is needed,
    `derive_project` fails with `ProjectError::TilesOutdated` ("the map area needs tiles … that were not
    fetched; run `freemapper fetch`").
  - `derived_images_are_written_atomically`: images are written via `write_atomically`; no `.part` left.
  - `cross_tile_orthophoto_and_dtm_are_seamless` (test gap): two synthetic tiles side by side through
    `build_orthophoto` and `mosaic_dtm`.
  - `fetch_not_covered_leaves_project_untouched` (test gap).
- [ ] Commit `fix(derive): errors instead of panics, size guard, stale records, atomic images`.

### Task 3: Slope template for flat ground (from the user's OOM check)

**Files:** `crates/freemapper/src/derive/terrain.rs`, `crates/freemapper/src/project.rs`.

- [ ] Test `slope_black_threshold_per_map_kind`: `MapKind::slope_black_deg` schoolyard 15°, sprint 30°;
  `slope_shade(slope, black_deg)` maps 7.5° to 128 at 15°.
- [ ] Commit `feat(derive): slope ramp per map kind`.

### Task 4: ALKIS robustness and traceability (M4a minors 1–9)

**Files:** `crates/freemapper/src/sources/alkis.rs`, `crates/freemapper/src/build/buildings.rs`,
`crates/freemapper/src/project.rs`, `crates/freemapper-cli/src/main.rs`.

- [ ] Tests, then fixes:
  - `fetch_stops_on_repeated_next_link` and `fetch_stops_on_empty_page`.
  - `duplicate_oids_across_pages_are_merged`.
  - `unusual_feature_is_skipped`: `parse_buildings` returns `ParsedBuildings { buildings, skipped:
    Vec<String> }`; a null geometry or missing oid is skipped and listed; `fetch` reports
    `"skipped {n} ALKIS features: …"`.
  - `corrupt_stored_buildings_name_the_file`: `ProjectError::StoredBuildings { path, message }`
    ("{path}: {message}; run `freemapper fetch` again").
  - `tiny_holes_are_dropped`: a 0.3 × 0.3 m light shaft at 1:1000 disappears; `MIN_SIDE_MM` doc says
    "area below (0.5 mm)² on paper".
  - `fetch_records_buildings`: `ProjectFile.buildings: Option<BuildingsRecord { bbox: [f64; 4],
    fetched: NaiveDate, latest_update: Option<NaiveDate>, count: usize }>`; a second fetch with a newer
    `aktualit` warns `"warning: ALKIS buildings changed upstream (… → …)"`; an offline fetch whose
    recorded bbox does not cover the current bbox fails with `BuildingsNotFetched`.
  - `run_hints_at_missing_buildings`: `MapOutcome::Kept` gains `missing_buildings: bool` (stored
    buildings exist, the kept map has no `alkis` objects); the CLI prints the `--force` hint.
  - `buildings_url_rounds_bbox_outward`: `bbox=421490,5821580,421737,5821897` for a fractional bbox.
- [ ] Commit `fix(alkis): paging guards, skipped features, traceability, tiny holes`.

### Task 5: M4b review findings (2 important, minors)

**Files:** `crates/freemapper/src/project.rs`, `crates/freemapper/src/build/peripheral.rs`,
`crates/freemapper/src/omap/symbol_set.rs`, `docs/glossary.md`.

- [ ] Tests, then fixes:
  - `empty_title_falls_back_to_folder_name`: `--title ""` / whitespace → `None`; a folder of only
    separators → `Karte`; control characters are removed from the title.
  - `display_title_of_dot_uses_the_real_folder`: `display_title(Path::new("."))` and `map_path(".")`
    use the canonical folder name.
  - `bar_scale_uses_round_lengths`: the bar is the largest 1/2/5 × 10ⁿ m not longer than 60 mm on paper
    (1:1000 → 50 m, 1:4000 → 200 m, 1:1500 → 50 m = 33.3 mm); labels use a decimal comma (`2,5`).
  - `bar_unit_is_its_own_text`: labels `0`, `25`, `50` centred on their ticks, `m` left-aligned after
    the bar end.
  - `sheet_text_is_german`: sheet lines `Kartengebiet: © OpenStreetMap-Mitwirkende, ODbL` and
    `Erstellt mit freemapper – https://github.com/HenryJobst/freemapper` (notes and ATTRIBUTION.txt
    keep the English lines).
  - `export_sprint_project`: north lines 30 mm apart, `Maßstab 1:4000 · Äquidistanz 2 m`.
  - `project_file_without_tiles_loads` also asserts `title == None` (hand-written old toml).
  - `north_line_spacing_sprint` uses the sprint set.
- [ ] `MapKind::symbol_enlargement` reuses the symbol set's constant; `bar_scale` becomes its own
  function (glossary code term).
- [ ] Commit `fix(sheet): title fallback, round bar scale, German sheet text`.
