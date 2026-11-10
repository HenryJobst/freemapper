# Region adapters implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `Region` and `Provider` enums and every `match` on them with a `RegionAdapter` trait, a registry, TOML manifests and adapter crates, so that a new country or state is a folder (or one small crate), with all eleven existing states producing identical output.

**Architecture:** First a five-layer offline golden net recorded from v0.9.0 (stage A). Then the shared types and readers move into `freemapper-adapter-api` and `freemapper-sources` (B). A registry with a shim over the old enums takes over one concern at a time until the enums can be deleted (C). The manifest crate, the URL policy and the loader come next (D), then `freemapper adapter check` and friends (E) and the bilingual guide (F). New states are separate plans (G).

**Tech Stack:** Rust 1.88 (pinned toolchain), `toml`, `serde` with `deny_unknown_fields`, `ureq` 3.4.2, `geo`, `freemapper-fs` and the new `freemapper-georef` from freemapper-core, GitHub Actions on three operating systems.

**Spec:** [`docs/superpowers/specs/2026-11-10-region-adapters-design.md`](../specs/2026-11-10-region-adapters-design.md), ADR [0082](../../adr/0082-region-adapters-trait-and-manifests.md). Section numbers below refer to the spec.

## Global Constraints

- Every task before stage D changes no output: `cargo test --workspace --locked` and the golden layers G1 to G5 stay green, and no file under `tests/golden/adapters/` changes in a `refactor(adapters):` commit (the CI guard of A5 enforces it).
- Manifest format is the integer `format = 1`; `deny_unknown_fields`; at most 64 KiB per manifest, 2 MiB per boundary file, 64 URLs per manifest, 64 files per adapter folder.
- Adapter id matches `[a-z][a-z0-9]{1,31}`. Legacy ids stay: regions `brandenburg`, `berlin`, `saxony`, `saxonyanhalt`, `hamburg`, `hessen`, `schleswigholstein`, `mecklenburgvorpommern`, `nordrheinwestfalen`, `niedersachsen`, `rheinlandpfalz`; providers `lgb`, `geosn`, `lvermgeo`, `lgv`, `hvbg`, `lvermgeosh`, `laivmv`, `geobasisnrw`, `lgln`, `lvermgeorp`.
- URLs in manifests: `https`, port 443, DNS names only, host listed in `[net] hosts`. Redirects at most 3, to declared hosts only. Loopback, link-local, private, unique-local, multicast and unspecified addresses are refused at connect time.
- Names from a server or a manifest reach a path only through a fixed table or a strict pattern (ADR 0059). All writes go through `freemapper-fs` (ADR 0060).
- Project-local adapters load only with `--adapters-from-project`, never through MCP.
- No new third-party dependency without the two-week rule (ADR 0012). Semver comparison for `min_freemapper` is hand-written (three integers).
- Shared code goes to freemapper-core, never copied (ADR 0067). freemapper-core pulls are pinned by tag.
- No e-mail address or other personal data in any request, `User-Agent` or fixture. Recording runs say so in their brief.
- English identifiers and docs; glossary terms; German text says "Bundesland". Commit messages end with the two trailer lines (`Co-Authored-By` and `Claude-Session`) given by the session. Run unslop and humanizer over prose.
- After every merge to main: watch `ci.yml` on three operating systems; `cargo clean` when the disk is tight (`df` first).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` before each commit.

## Review Focus

Failure modes the tasks' main tests do not cover, most likely first. Each has a test in the owning task.

1. A map area whose centre is in one adapter and whose edge is in another with a different UTM zone (Saxony-Anhalt and Brandenburg, zone 32 against 33): CRS and rasters follow the centre, ALKIS comes from both. Test in C2 (`border_centre_decides_crs`).
2. A project file that names an adapter that is not installed (a project from a colleague with a Czech adapter): a clear error with the id and the search folders, no silent re-fetch, no panic. Test in C14 and G5.
3. A manifest whose declared host resolves to `127.0.0.1`, redirects to an undeclared host or to an IP literal, or whose answer is larger than the call's cap. Test in D2 with a stub resolver.
4. Two adapters with the same id, or two adapters sharing a provider: a folder adapter cannot replace a built-in one; shared providers share the cache folder and the record. Test in D7.
5. A partial or hostile adapter folder: all capabilities absent, a boundary in the wrong CRS or self-crossing, a file name with `..`, a BOM or CRLF in the TOML, a symlinked folder. Test in D1 and E2.

---

## Stage A: the net

### Task A1: Glossary rows and golden G1 (registry data)

**Files:**
- Modify: `docs/glossary.md` (rows Regionsadapter, Adaptermanifest, Rasterbackend, Adapterprüfung; row "Quelladapter" points to Regionsadapter)
- Create: `crates/freemapper/tests/adapters_golden.rs`, `tests/golden/adapters/g1-regions.json`
- Create: `crates/freemapper/src/sources/region.rs` test-only helper `pub fn legacy_facts() -> Vec<RegionFacts>` behind `#[cfg(any(test, feature = "golden"))]`

**Interfaces:**
- Produces: `RegionFacts { id, provider, terrain_source, per_area, crs_epsg, object_id_prefix, boundary_sha256, osm: Vec<OsmFacts>, credit: Vec<(year, text)> }`, serialisable; the JSON file is its pretty print.

- [ ] **Step 1:** Add the glossary rows (German and English columns, `code` column names `RegionAdapter`, `ManifestAdapter`, `RasterBackend`, `adapter check`).
- [ ] **Step 2:** Write test `g1_registry_data_is_unchanged` that builds `legacy_facts()` from the enums and compares with `tests/golden/adapters/g1-regions.json`; with `FREEMAPPER_BLESS=1` it writes the file.
- [ ] **Step 3:** Implement `legacy_facts()` from `Region::ALL`, `Region::provider`, `Provider::terrain_source`, `Provider::per_area`, `Region::crs`, `ExtractSource::{url, state_url, prefix, megabytes, label}` (NRW per district), SHA-256 of each boundary constant, and the credit lines of the ten `*_attribution` functions for 2025 and 2026 (build a `ProjectFile` with records dated in that year).
- [ ] **Step 4:** Run `FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden g1` once on the v0.9.0 tree, check the JSON by eye (11 regions, 10 providers, 5 NRW extracts), run again without the variable. Expected: PASS.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 1, registry data` with the glossary.

**Net:** this task creates G1. **Outcome:** a file that says what the enums answered.

### Task A2: Golden G2 (geometry and dispatch)

**Files:**
- Modify: `crates/freemapper/tests/adapters_golden.rs`
- Create: `tests/golden/adapters/g2-geometry.json`, `tests/golden/adapters/g2-boxes.json` (the 211 boxes: 11 example areas read from `examples/*/freemapper.toml`, 200 border and coastal boxes)

**Interfaces:** consumes `regions_for`, `raster_region`, `provider_tiles(bbox, provider, km)`, `reaches_beyond_coverage`, `tile_touches`, `tile_centre_in`, `osm::extracts_for` (all public today).

- [ ] **Step 1:** Write `g2_geometry_is_unchanged`: for each box, record `regions_for` ids, `raster_region` id, `provider_tiles` for km 1 and 2 (names), `reaches_beyond_coverage`, `extracts_for` labels. Compare with the JSON.
- [ ] **Step 2:** Add `g2_dispatch_is_fast_enough`: all 211 boxes complete within 2 seconds in release-like test profile (twice the v0.9.0 measurement, written into the test as a constant after measuring).
- [ ] **Step 3:** Generate the 200 boxes deterministically in the test (seeded; points along the borders BB/BE, ST/NI, NI/Bremen, HE/RP, NW/NI, MV/SH, MV/BB, SN/BB, HH/SH, HH/NI, sizes 150 m to 1.5 km); bless once on v0.9.0.
- [ ] **Step 4:** Run the test. Expected: PASS twice (second run without bless).
- [ ] **Step 5:** Commit `test(adapters): golden net layer 2, geometry and dispatch`.

### Task A3: Replay and recording fetchers, golden G3 (request trace)

**Files:**
- Create: `crates/freemapper/tests/support/replay.rs` (`RecordingFetcher`, `ReplayFetcher`), `tests/golden/adapters/README.md` (how to record), `tests/golden/adapters/empty.osm.pbf` (a valid PBF with a header block and no data, under 300 bytes)
- Create per state: `tests/golden/adapters/<id>/responses/index.toml` and `<sha256-of-url-and-range>.bin`, `<id>/trace.txt`
- Modify: `crates/freemapper/tests/adapters_golden.rs`

**Interfaces:**
- Produces: `ReplayFetcher::new(dir: &Path) -> Self` implementing `Fetcher` (including `get_range`, `last_modified`, `download_to`; OSM `.pbf` URLs answer with `empty.osm.pbf`); `RecordingFetcher::new(inner: HttpFetcher, dir: &Path)` active only when `FREEMAPPER_GOLDEN_RECORD` is set. Both log `METHOD url [range]` lines to a `Vec<String>` readable as `trace()`.

- [ ] **Step 1:** Write `replay_returns_recorded_bytes_and_logs_requests` (a two-URL index, one range request, one unknown URL returns `FetchError::Http` with status 404 and appears in the trace).
- [ ] **Step 2:** Implement both fetchers. The key is SHA-256 of the URL plus the range; the user agent and headers are not recorded. The recorder refuses to run when the environment holds `FREEMAPPER_CONTACT` or any URL contains `@` (no personal data in requests).
- [ ] **Step 3:** Record one online `fetch_project` plus `estimate_download` per state on the golden area (a 150 m box in the example's area) with the maintainer's network. Brief for whoever runs it: no e-mail address in any request. Write `trace.txt` from the replay of the recording.
- [ ] **Step 4:** Write `g3_trace_<id>` for each of the 11 states (one test function generated by a macro): replay the fetch against a fresh project folder, compare the trace and the `warning:` lines with `trace.txt`.
- [ ] **Step 5:** Run `cargo test -p freemapper --test adapters_golden g3`. Expected: 11 PASS. Check the per-state fixtures stay under 2 MB each.
- [ ] **Step 6:** Commit `test(adapters): golden net layer 3, request traces`.

### Task A4: Golden G4 (offline pipeline per state)

**Files:**
- Create per state: `tests/golden/adapters/<id>/project/` (`freemapper.toml`, `data/`, `templates/` inputs, `cache/` tiles and models as the recording produced them, `data/osm-candidates.geojson` from the real extract), `<id>/expected.sha256`
- Modify: `crates/freemapper/tests/adapters_golden.rs`

**Interfaces:** consumes `derive_project`, `export` (the function `freemapper run` calls after fetch), `FetchMode::Offline`.

- [ ] **Step 1:** Write `g4_pipeline_<id>`: copy the stored project to a temp folder, point the cache at its `cache/`, run `fetch_project` offline, `derive_project`, then the map export with a fixed declination date; compute SHA-256 of `<name>.omap`, `templates/candidates.omap`, `ATTRIBUTION.txt`, `data/contours.geojson`, `templates/hillshade.png`, `slope.png`, `orthophoto.jpg`; compare with `expected.sha256`.
- [ ] **Step 2:** Produce the projects from the A3 recording session (the same run that wrote the responses leaves `data/` and the cache behind); prune the cache to the golden area; check each folder is under 2 MB.
- [ ] **Step 3:** Bless once on the v0.9.0 tree. Run twice to prove determinism. Expected: PASS twice on the same machine; also on the CI runners (check in step 4).
- [ ] **Step 4:** Add the job `golden` to `.github/workflows/ci.yml` (runs `cargo test -p freemapper --test adapters_golden` on Linux, macOS and Windows). Expected: green on all three; if PNG encoding differs across systems, compare decoded pixels instead and say so in the test.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 4, offline pipeline`.

### Task A5: Golden G5 (persisted format) and the CI guard

**Files:**
- Modify: `crates/freemapper/tests/adapters_golden.rs`, `.github/workflows/ci.yml`
- Create: `.github/scripts/check-golden-untouched.sh`

- [ ] **Step 1:** Write `g5_examples_roundtrip`: for each `examples/*/freemapper.toml`, `ProjectFile::load` then `save` to a temp folder; bytes equal the original.
- [ ] **Step 2:** Write `g5_unknown_provider_is_a_clear_error` (ignored until C14): a toml with `provider = "czfoo"` loads with a message that contains `czfoo` and the word `adapter`.
- [ ] **Step 3:** The script fails when the pull request title starts with `refactor(adapters):` and `git diff --name-only origin/main...HEAD` lists a path under `tests/golden/adapters/`. Add the step to the workflow for `pull_request` events.
- [ ] **Step 4:** Run G5 locally. Expected: roundtrip PASS, the second test ignored.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 5 and the guard`.

---

## Stage B: crates

### Task B1: `freemapper-georef` in freemapper-core (separate repository)

**Files (in `../freemapper-core`):**
- Create: `crates/freemapper-georef/` (`Cargo.toml`, `src/lib.rs`, `src/crs.rs`, `src/tiles.rs`), `docs/adr/0015-the-georef-crate.md`
- Modify: workspace `Cargo.toml`, README tables (status "enthalten"), `CHANGELOG.md`/`.en.md`

**Interfaces:** produces `Crs`, `LatLon`, `ProjectedPoint`, `BoundingBox`, `TileId` with the exact public items of `crates/freemapper/src/crs.rs` and `tiles.rs` (no imports beyond `std`); the `georef.rs` declination part stays in freemapper (it needs `omap::MapCoord`).

- [ ] **Step 1:** Copy the tests of `crs.rs` and `tiles.rs` into the new crate unchanged; run them red (module missing).
- [ ] **Step 2:** Move the code. The move keeps names and behaviour. Add a README paragraph.
- [ ] **Step 3:** Run `cargo test --workspace --locked` in core. Expected: PASS.
- [ ] **Step 4:** Release core `v0.4.0` after the core CI is green (core ADR 0006). Tag only after review.
- [ ] **Step 5:** Commit `feat(georef): crs and tile types from freemapper`.

### Task B2: freemapper uses `freemapper-georef`

**Files:** Modify `Cargo.toml` (pin `tag = "v0.4.0"` for `freemapper-fs` and add `freemapper-georef`), `crates/freemapper/Cargo.toml`, `crates/freemapper/src/crs.rs` and `tiles.rs` become `pub use freemapper_georef::*;`-style re-exports, ADR 0067 gets a line (new ADR not needed).

- [ ] **Step 1:** Change the pin; `cargo build --locked` fails on the lock file; run `cargo update -p freemapper-fs -p freemapper-georef` and check that the lock diff touches only core crates and what they bring in (ADR 0067).
- [ ] **Step 2:** Replace the two modules by re-exports; delete the moved tests.
- [ ] **Step 3:** Run `cargo test --workspace --locked` and the golden tests. Expected: PASS, G1 to G5 unchanged.
- [ ] **Step 4:** Commit `refactor(adapters): crs and tiles from freemapper-georef`.

If core v0.4.0 is not out, do B3 first with the types in the api crate and redo this task later (D4).

### Task B3: `freemapper-adapter-api`

**Files:**
- Create: `crates/freemapper-adapter-api/{Cargo.toml,src/lib.rs,src/fetch.rs,src/records.rs,src/error.rs}`
- Modify: `crates/freemapper/src/fetch.rs` (the trait and error types become `pub use freemapper_adapter_api::fetch::*`), `crates/freemapper/src/project.rs` (record types re-exported)

**Interfaces:**
- Produces in the api crate: `trait Fetcher`, `enum FetchError`, `enum FetchMode`, `struct ByteRange`, `FetchedTile`, `TileCurrency`, `TileRecord`, `WcsRecord`, `OrthophotoRecord`, `RasterRecords { tiles, ndsm_tiles, orthophoto, terrain, surface }`, `EstimateItem`, `Part`, `Store` (moved from `space.rs`), `AdapterId(String)` with `FromStr` validating `[a-z][a-z0-9]{1,31}`, `CacheDir(PathBuf)` with `join_checked`.
- `HttpFetcher`, `Cache`, `DiskSpace` stay in `freemapper`.

- [ ] **Step 1:** Write `adapter_id_accepts_legacy_ids_and_rejects_others` (all 21 legacy ids pass; `""`, `"A"`, `"a"`, `"a b"`, `"../x"`, 33 characters fail).
- [ ] **Step 2:** Create the crate, move the items, re-export from the old paths so no call site changes.
- [ ] **Step 3:** Build and run the whole suite. Expected: PASS, goldens unchanged. Check that `cargo tree -p freemapper-adapter-api` shows no `ureq`, `toml` or `axum`.
- [ ] **Step 4:** Commit `refactor(adapters): freemapper-adapter-api with fetcher and record types`.

### Task B4: `freemapper-sources` (the generic readers)

**Files:**
- Create: `crates/freemapper-sources/` with modules `wms`, `wcs`, `wfs` (from `alkis_wfs.rs`), `ogc_api` (from the reader half of `alkis.rs`), `nas` (from `alkis_nas.rs`), `trees`, `zip_range`, `xlsx`, `float_tiff`, `height_model`, `raster`, `metalink` (extracted from `lvermgeo_rp.rs`)
- Modify: `crates/freemapper/src/sources/*.rs` (re-exports at the old paths), `Cargo.toml`

**Interfaces:** `WmsService`, `WcsService`, `WcsOrthophoto`, `WfsService`, `NasService`, `TreeDataset` change their `&'static str` fields to `Cow<'static, str>` so manifests can build them at run time; the constants stay `const` (`Cow::Borrowed`). `NasService.region` and `TreeDataset.region` become `AdapterId` (a `&'static str`-backed constant for the built-ins).

- [ ] **Step 1:** Move one reader at a time in the order `zip_range`, `xlsx`, `float_tiff`, `height_model`, `raster`, `wms`, `wcs`, `wfs`, `ogc_api`, `nas`, `trees`, `metalink`; after each, `cargo test --workspace --locked`.
- [ ] **Step 2:** Switch the service structs to `Cow<'static, str>`. Write `service_constants_are_unchanged` (URL strings produced by `piece_url`, `get_coverage_url` for 3 sample boxes equal the v0.9.0 strings, which the test holds as literals).
- [ ] **Step 3:** Check `cargo build -p freemapper-sources` alone compiles without `freemapper`, and measure the clean build time of the workspace before and after (record both numbers in the commit message).
- [ ] **Step 4:** Full suite and goldens. Expected: PASS, no golden change.
- [ ] **Step 5:** Commit in two commits: `refactor(adapters): move the readers to freemapper-sources`, `refactor(adapters): owned service fields`.

---

## Stage C: the trait and the registry

Recipe for every C task: (1) write the test that pins the concern through the trait; (2) add the trait method or data; (3) implement it for the built-in states by moving the existing code (no edits to bodies beyond signatures); (4) delete the old `match`; (5) goldens and the full suite stay green; (6) commit `refactor(adapters): ...`. If a golden file would have to change, stop: the move is not behaviour-preserving.

### Task C1: `RegionAdapter`, `Registry` and the legacy shim

**Files:**
- Create: `crates/freemapper-adapter-api/src/{adapter.rs,registry.rs,info.rs}`, `crates/freemapper/src/adapters/legacy.rs`
- Modify: `crates/freemapper/src/lib.rs`

**Interfaces:**
- Produces: `trait RegionAdapter` (section 3.3, with `rasters`, `alkis`, `extras`, `attribution`, `estimate` returning `None`, empty or default in this task), `struct RegionInfo { id: AdapterId, provider: AdapterId, names: Names, crs: Crs, object_id_prefix: Option<String>, osm: Vec<OsmExtractInfo>, credit_short: Option<String> }`, `struct Registry`, `fn registry() -> &'static Registry` (process-global, `OnceLock`, set once by `install_registry(Registry)`; default `Registry::builtin()`), `struct LegacyAdapter(Region)` implementing the trait from the enums.

- [ ] **Step 1:** Write `registry_lists_the_eleven_in_legacy_order` (ids equal `Region::ALL` order) and `legacy_info_matches_g1` (every `RegionInfo` field equals the G1 JSON).
- [ ] **Step 2:** Implement the types and `LegacyAdapter`; boundaries stay the `include_str!` constants parsed once.
- [ ] **Step 3:** Run G1. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C2: Geometry functions on the registry

**Files:** Modify `crates/freemapper/src/sources/region.rs` (the 16 production refs), callers in `layers.rs`, `osm.rs`, `alkis.rs`, `trees.rs`, `alkis_nas.rs`.

**Interfaces:** `regions_for`, `raster_region`, `provider_tiles`, `reaches_beyond_coverage`, `tile_touches`, `tile_centre_in`, `contains_point`, `touches_region` keep their names and add `_in(&Registry, …)` variants; the old names call `registry()`. They take `&AdapterId` (or `&dyn RegionAdapter`) instead of `Region`. A bounding-rectangle pre-filter sits in front of `intersects` (exact, so no result changes).

- [ ] **Step 1:** Write `border_centre_decides_crs` (Review Focus 1): a box with centre in Saxony-Anhalt and an edge in Brandenburg gives `Crs::Utm32`, ALKIS regions `[brandenburg, saxonyanhalt]` in registry order, and the reverse box gives `Utm33`.
- [ ] **Step 2:** Move the logic; keep `Region` only as the legacy adapter's id source.
- [ ] **Step 3:** Run G2 including the timing test. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C3: OSM extracts through the adapter

**Files:** Modify `crates/freemapper/src/sources/osm.rs` (`ExtractSource`, `extracts_for`, `GovernmentDistrict`), `layers.rs`.

**Interfaces:** `OsmExtractInfo { geofabrik_path, megabytes, label, prefix, boundary: Option<MultiPolygon> }` in `RegionInfo.osm`; `ExtractSource` becomes `ExtractRef { adapter: AdapterId, index: usize }` with `url()`, `state_url()`, `prefix()`, `megabytes()`, `label()` read from the info. The NRW rule (extracts whose boundary lies within the margin; nearest if none, with the existing warning) becomes the generic rule "when an adapter has several extracts, pick by boundary".

- [ ] **Step 1:** Write `nrw_district_border_reads_two_extracts_with_the_old_warning` and `single_extract_regions_ignore_boundaries`.
- [ ] **Step 2:** Move the five district boundaries to the NRW info (still `include_str!`).
- [ ] **Step 3:** G1, G2, G3 (the state-URL and PBF URLs appear in traces). Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C4: Credit lines through the adapter

**Files:** Modify `crates/freemapper/src/project.rs` (the ten `*_attribution` functions, `attribution_lines`, `raster_provider`, `per_area_attribution_year`), `build/source_block.rs` (no change expected).

**Interfaces:** `RegionAdapter::attribution(&self, used: &UsedSources) -> Vec<CreditLine>` where `UsedSources` carries the project's raster records, ALKIS records and `provider`; `CreditLine { text, licence }`. `per_area_attribution_year` becomes a helper in the api crate.

- [ ] **Step 1:** Write `credit_lines_match_for_all_year_and_record_combinations` using the G1 credit table plus three record combinations per state (rasters only, ALKIS only, both).
- [ ] **Step 2:** Move each function's text into its legacy adapter; Berlin's and Brandenburg's lines keep their special forms (`berlin_attribution`, `lgb_attribution_with_berlin`).
- [ ] **Step 3:** G1 and G4 (`ATTRIBUTION.txt` bytes). Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C5: Estimate through the adapter

**Files:** Modify `crates/freemapper/src/layers.rs` (`area_tiles`, `estimate_area`, ten `*_items`), `space.rs` (`Part`, `Store` already moved in B3).

**Interfaces:** `RegionAdapter::estimate(&self, area: &EstimateArea, env: &EstimateEnv) -> Vec<EstimateItem>` with `EstimateArea { bbox_25833, map_bbox, today }` and `EstimateEnv { cache: &CacheDir }`.

- [ ] **Step 1:** Write `estimate_items_equal_v090_for_the_example_areas` (item text, MB, cached flag, part, store for the 11 example boxes, golden file `g2-estimates.json` created in this task from v0.9.0 and exempt from the guard because it is created, not edited; note it in the commit).
- [ ] **Step 2:** Move the functions; the shared constants (`ORTHOPHOTO_MB_PER_KM2` and the others) move to the api crate as defaults.
- [ ] **Step 3:** Run goldens and `estimate_download` tests. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C6: ALKIS through the adapter

**Files:** Modify `crates/freemapper/src/sources/alkis.rs` (`fetch_buildings_in`, `fetch_land_use_in`: the two 11-arm matches), `crates/freemapper-sources/src/wfs.rs` (the nine `WfsService` constants become data in the legacy adapters).

**Interfaces:** `AlkisSource::{buildings, land_use}(&self, env, area) -> Result<AlkisPart, AdapterError>` with `AlkisPart { features: Vec<Value>, crs: Crs }`; helpers `WfsAlkis`, `OgcApiAlkis { endpoints, fallback: Option<WfsService>, normalise: Normaliser }` in `freemapper-sources`; `Normaliser::{Lgb, HamburgAscii}` (moves `lgv::normalise_building` and `normalise_land_use`). Berlin and Brandenburg keep code (`berlin::fetch_buildings`, `collection_features`).

- [ ] **Step 1:** Write `wfs_rows_equal_constants` (each of the nine rows built from the new struct equals the old constant field by field) and `hamburg_falls_back_to_wfs_when_the_ogc_api_fails` (uses the replay fetcher with the API URL returning 500).
- [ ] **Step 2:** Replace the two `match`es by `merged(area, |adapter| adapter.alkis()...)`; the merge by oid stays in `alkis.rs`.
- [ ] **Step 3:** G3 (requests), G4 (buildings and land use files). Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C7: Tile raster backends (LGB and GeoSN)

**Files:** Create `crates/adapters/de/src/{lgb.rs,geosn.rs}`; modify `project.rs` (`fetch_lgb_rasters`, `fetch_geosn_rasters`, the derive arms for tile providers), `fetch.rs` (currency tables), `layers.rs`, `sources/lgb.rs` (`Product`).

**Interfaces:** `RasterBackend` as in section 3.3; `ProviderInfo { id, terrain_source, kind: Tiles { grid_km } | PerArea }`; `DeriveInputs` as in section 3.3. `Product` keeps its five variants and serde names, and moves to the DE adapter crate with the cache folder names (`lgb`, `geosn`).

- [ ] **Step 1:** Write `lgb_offline_fetch_reads_cached_tiles_only` and `geosn_offline_with_missing_index_is_the_old_error` (the old offline special case at `fetch.rs:889` keeps its message).
- [ ] **Step 2:** Move the two fetch functions into `RasterBackend::fetch` and the derive match arms into `derive_inputs`; the project code calls `adapter.rasters()` and no longer knows tile or per-area kinds by enum.
- [ ] **Step 3:** G1 to G4 for BB, BE and SN. Expected: PASS.
- [ ] **Step 4:** Commit. This is the largest C task (about 1 200 lines moved); split it into two commits, LGB then GeoSN.

### Task C8: Saxony-Anhalt and Lower Saxony backends

**Files:** Create `crates/adapters/de/src/{lvermgeo.rs,lgln.rs}`; modify `project.rs` (`fetch_lvermgeo_rasters`, `fetch_lgln_rasters`), `layers.rs` (`lvermgeo_items`, `lgln_items`).

- [ ] **Step 1:** Write `st_wcs_request_is_cut_to_the_envelope` (a box reaching past the envelope yields clipped `SUBSET`s) and `ni_stac_date_and_age_warning_text_unchanged`.
- [ ] **Step 2:** Move the code per the recipe; leave the dates logic in the modules (they become manifest `date` kinds in D5).
- [ ] **Step 3:** G1 to G4 for ST and NI. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C9: Hamburg and Hesse backends

**Files:** Create `crates/adapters/de/src/{lgv.rs,hvbg.rs}`; modify `project.rs` (`fetch_lgv_rasters`, `fetch_hvbg_rasters`), `layers.rs`, `sources/zip_range.rs` use sites.

- [ ] **Step 1:** Write `hesse_negative_eastings_return_no_tile_grid` (the `per_area` rule from `raster_tiles`) and `hamburg_missing_tile_message_unchanged`.
- [ ] **Step 2:** Move; `lgv::spelling_key` stays public in `freemapper-sources` because `build/state.rs` uses it.
- [ ] **Step 3:** G1 to G4 for HH and HE. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C10: Schleswig-Holstein backend and the trust anchors

**Files:** Create `crates/adapters/de/src/lvermgeo_sh.rs`, `crates/freemapper-adapter-api/src/trust.rs`; move `assets/tls/*.pem` use from `fetch.rs`.

**Interfaces:** `TrustAnchors { host: &'static str, pems: &'static [&'static [u8]] }`; `RasterBackend::trust(&self) -> Option<&TrustAnchors>`; `HttpFetcher` builds the pinned agent per registered `TrustAnchors` instead of the `PORTAL_HOST` constant. The no-redirect rule and the host comparison (`trust_for`) stay.

- [ ] **Step 1:** Write `pinned_host_matches_case_insensitively_with_one_trailing_dot_and_follows_no_redirects` (the existing tests of `trust_for` and `max_redirects()` move; they must pass unchanged) and `unpinned_hosts_use_default_roots`.
- [ ] **Step 2:** Move the code and the PEM `include_bytes!` to the SH adapter.
- [ ] **Step 3:** Full suite, G1 to G4 for SH. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C11: Mecklenburg-Vorpommern and North Rhine-Westphalia backends

**Files:** Create `crates/adapters/de/src/{laiv_mv.rs,geobasis_nrw.rs}`; modify `project.rs` (`fetch_mv_rasters`, `fetch_nrw_rasters`), `layers.rs`.

- [ ] **Step 1:** Write `mv_cells_outside_the_tile_set_are_nan_and_zeros_inside_stay` (the existing unit test moves with a tiny synthetic raster) and `nrw_tile_names_only_letters_digits_underscores`.
- [ ] **Step 2:** Move per the recipe.
- [ ] **Step 3:** G1 to G4 for MV and NW. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C12: Rhineland-Palatinate backend

**Files:** Create `crates/adapters/de/src/lvermgeo_rp.rs`; modify `project.rs` (`fetch_rp_rasters`), `layers.rs` (`rp_items`).

- [ ] **Step 1:** Write `rp_missing_tiles_are_checked_again_after_30_days` (the existing test with an injected date) and `rp_uses_origin_hosts_only` (asserts no URL in the G3 trace contains `geoportal.rlp.de`).
- [ ] **Step 2:** Move per the recipe; the Metalink code already lives in `freemapper-sources::metalink` (B4).
- [ ] **Step 3:** G1 to G4 for RP. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task C13: Extra layers and the user-visible state lists

**Files:** Create `crates/adapters/de/src/berlin.rs` (adapter with code `ExtraLayer`s); modify `project.rs` (Berlin layers, NAS layers, tree layers records and paths), `sources/alkis_nas.rs`, `sources/trees.rs` (service tables read from adapters), `crates/freemapper-cli/src/select.rs` (new `GET /api/adapters`), `crates/freemapper-cli/web/app.js` (replace the two state lists and the per-state aerial layers by the response), `project.rs:134` and `mcp.rs:31` (texts built from `names`).

**Interfaces:** `trait ExtraLayer { fn id(&self) -> &str; fn kinds(&self) -> &[MapKind]; fn fetch(...); }`; `NAS_SERVICES` and `TREE_DATASETS` become registry queries `nas_services_for(bbox)` and `tree_datasets_for(bbox)` with the same results; `/api/adapters` returns `[{id, names, preview: {wms, layer, attribution}}]`.

- [ ] **Step 1:** Write `berlin_layers_paths_still_come_from_the_static_table` (ADR 0059: a key outside the table is dropped with the old warning) and `adapters_endpoint_lists_eleven_with_previews` (in `crates/freemapper-cli/tests/select.rs`).
- [ ] **Step 2:** Move the extras; keep `data/berlin`, `data/alkis-nas` and `data/trees` names.
- [ ] **Step 3:** Replace the hard-coded lists in the page; check it in the browser against the 11 states (manual check, note in the commit) and with the content security policy test.
- [ ] **Step 4:** G1 to G4 for BB and BE, `select` tests. Expected: PASS.
- [ ] **Step 5:** Commit in two commits (extras, then page and texts).

### Task C14: Persisted ids and deleting the enums

**Files:** Modify `project.rs` (`ProjectFile.provider: Option<AdapterId>`, record `regions: Vec<AdapterId>`), `region.rs` (delete `Region`, `Provider`, `ALL`, the boundary constants), `adapters/legacy.rs` (delete), all remaining `Region::`/`Provider::` sites.

**Interfaces:** `AdapterId` serialises as a plain lower-case string (the same bytes the enums produced).

- [ ] **Step 1:** Remove the `#[ignore]` of `g5_unknown_provider_is_a_clear_error` (message: `the project names the adapter "czfoo", which is not installed; adapters are searched in: …`).
- [ ] **Step 2:** Add the guard test `no_enum_matches_left`: a test that reads `crates/**/*.rs` outside `tests/` and fails on `Region::` or `Provider::` (the legacy id table lives in the DE adapter crate as string constants).
- [ ] **Step 3:** Delete the enums and the shim; fix the last call sites.
- [ ] **Step 4:** Full suite, G1 to G5. Expected: PASS, no golden change.
- [ ] **Step 5:** Commit; open the PR for stages A to C if not already merged. Watch CI on three systems.

---

## Stage D: manifests

### Task D1: Schema, parser and validator

**Files:**
- Create: `crates/freemapper-manifest/{Cargo.toml,src/lib.rs,src/schema.rs,src/validate.rs,src/error.rs}`, `crates/freemapper-manifest/tests/invalid/*.toml` (30 files), `tests/valid/{hamburg,nordrheinwestfalen,saxonyanhalt}.toml`

**Interfaces:**
- Produces: `fn parse(text: &str, folder: &Path) -> Result<Manifest, Vec<ManifestError>>`; `struct Manifest` (tables of section 3.4); `ManifestError { code: &'static str, path: PathBuf, line: usize, key: String, message: String }` with codes such as `manifest.unknown-key`, `manifest.format`, `manifest.id`, `manifest.file-name`, `manifest.too-large`, `policy.url`, `policy.host-not-declared`.

- [ ] **Step 1:** Write `valid_manifests_parse` (the three sketches of section 3.5, adjusted to the final schema) and `invalid_manifests_fail_with_their_code`, a table test over the 30 files (each file's first line is a comment `# expect: <code>`): unknown key, `format = 2`, bad id, `file = "../x"`, `file = "a/b"`, 65 KiB text, 65 URLs, `http://`, IP literal, `localhost`, user info, port 8443, host not in `[net]`, wildcard host, attribution with newline or over 400 characters, unknown licence, `crs = 4326` in `[adapter]`, empty `names.en`, BOM, CRLF (accepted), duplicated tables, `kind = "script"`, `date.kind = "x"`, `normalise = "nope"`, an adapter with no `[alkis]` and no `[orthophoto]`.
- [ ] **Step 2:** Implement `serde` structs with `deny_unknown_fields`, then the checks that serde cannot do. Line numbers come from `toml::Spanned`.
- [ ] **Step 3:** Run `cargo test -p freemapper-manifest`. Expected: PASS, 33 cases.
- [ ] **Step 4:** Commit.

### Task D2: `UrlPolicy` and the enforcing fetcher

**Files:**
- Create: `crates/freemapper-adapter-api/src/policy.rs`
- Modify: `crates/freemapper/src/fetch.rs` (`HttpFetcher`: resolver, redirect handling, per-call byte caps), `crates/freemapper-adapter-api/src/fetch.rs`

**Interfaces:**
- Produces: `struct UrlPolicy { hosts: BTreeSet<String>, max_redirects: u8 }`, `fn check_url(&self, url: &str) -> Result<(), PolicyError>`, `fn check_addr(addr: IpAddr) -> Result<(), PolicyError>`, `trait Resolver` (`std::net::ToSocketAddrs` by default, a stub in tests); `HttpFetcher::with_policy(UrlPolicy)`. The built-in adapters get a policy built from their declared hosts; before D6 the policy for built-ins is "address check only".

- [ ] **Step 1:** Write `declared_host_resolving_to_loopback_is_refused`, `redirect_to_undeclared_host_is_refused`, `redirect_to_ip_literal_is_refused`, `more_than_three_redirects_is_refused`, `answer_over_the_cap_is_too_large`, `private_ranges_are_refused` (10/8, 172.16/12, 192.168/16, 169.254/16, fc00::/7, fe80::/10, ::1, 127/8, 0.0.0.0, 224/4), with a local stub server on 127.0.0.1 and a stub resolver (Review Focus 3).
- [ ] **Step 2:** Implement with `ureq`'s resolver hook and per-redirect re-check; the pinned SH agent keeps `max_redirects(0)`.
- [ ] **Step 3:** Run the new tests and the whole suite. Expected: PASS; goldens unchanged (built-ins reach public addresses).
- [ ] **Step 4:** Commit `feat(adapters): url policy and address checks in the fetcher`.

### Task D3: `ManifestAdapter` (info, boundary, OSM, credit, preview)

**Files:** Create `crates/freemapper-manifest/src/adapter.rs`, `crates/freemapper-manifest/src/boundary.rs`.

**Interfaces:** `struct ManifestAdapter` implementing `RegionAdapter` for the declared parts; `fn load_boundary(path, crs) -> Result<MultiPolygon<f64>, ManifestError>` (reprojects 4326 to 25833, validates with `geo::Validation`, rejects a polygon over 1 000 000 vertices); `[[osm.extract]]` builds URLs `https://download.geofabrik.de/<path>-latest.osm.pbf` and `.../<path>-updates/state.txt`.

- [ ] **Step 1:** Write `boundary_in_4326_equals_the_25833_boundary_within_1_m` (reproject Hamburg's file both ways), `self_crossing_boundary_is_rejected`, `geofabrik_urls_equal_the_osm_rs_constants_for_hamburg_and_hessen`, `credit_text_substitutes_year_once`.
- [ ] **Step 2:** Implement; credits use `{year}` replaced by the newest fetch year; text length and control characters are checked at load. An optional `rules.toml` in the folder goes through the existing rule validator, may hold candidate rules only, and gets `freemapper:source=adapter/<id>` ranked below OSM (test `adapter_rules_cannot_draw_map_objects`).
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task D4: ALKIS, extras and estimate from a manifest

**Files:** Create `crates/freemapper-manifest/src/alkis.rs`, `src/extras.rs`, `src/estimate.rs`.

**Interfaces:** `[alkis]` builds `WfsAlkis` or `OgcApiAlkis` from `freemapper-sources`; `[[extra.nas]]` and `[[extra.trees]]` build `NasService` and `TreeDataset`; `[estimate]` overrides the defaults.

- [ ] **Step 1:** Write `manifest_wfs_rows_equal_the_nine_constants` (the nine manifests under `crates/adapters/de/manifests/`, written in this task from the constants, build services equal to the constants), `hamburg_manifest_equals_the_ogc_api_plus_wfs_fallback`, and `potsdam_tree_dataset_from_manifest_equals_the_constant`.
- [ ] **Step 2:** Implement the builders.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task D5: Rasters and date kinds from a manifest; named backends

**Files:** Create `crates/freemapper-manifest/src/rasters.rs`, `src/dates.rs`.

**Interfaces:** `[orthophoto]`, `[terrain]`, `[surface]` build `WmsService`, `WcsOrthophoto`, `WcsService` or resolve `backend = "<name>"` through `Registry::backend(name) -> Option<Arc<dyn RasterBackend>>`; date kinds `wfs-geojson`, `stac-newest`, `wms-featureinfo`, `none` implemented with the existing parsers (`lvermgeo::parse_index_date`, `lgln::newest_item`); a manifest naming an unknown backend gives `manifest.backend-missing` with the feature name.

- [ ] **Step 1:** Write `saxony_anhalt_manifest_builds_the_same_requests_as_lvermgeo_rs` (G3-style: URLs for a sample box equal), `niedersachsen_stac_date_matches_parse_fixture`, `unknown_backend_names_the_missing_feature`.
- [ ] **Step 2:** Implement. The date kinds reuse the parsers; they are moved, not rewritten.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task D6: Move the eleven states onto embedded manifests

**Files:** Create `crates/adapters/de/manifests/<id>/manifest.toml` and `boundary.geojson` (moved from `assets/regions/`) for 11 states, plus `osm-<district>.geojson` for NRW; modify `crates/adapters/de/src/lib.rs` (`register`), delete `adapters/legacy` remnants.

- [ ] **Step 1:** For each state in the order ST, NI (fully declarative), then HH, NW, SN, MV, HE, SH, RP, BB, and last BE (thin manifest plus code adapter): write the manifest, switch the registry entry from the code constants to `ManifestAdapter` plus the backend, run G1 to G5.
- [ ] **Step 2:** Add `builtin_manifests_pass_the_validator_and_policy` (every embedded manifest passes D1 and declares every host it uses).
- [ ] **Step 3:** One PR per three states (four PRs). Each must keep all goldens green with no golden edits.
- [ ] **Step 4:** After the last state: delete the now-unused constants, run `cargo clippy` for dead code. Expected: clean.
- [ ] **Step 5:** Commit per state group.

### Task D7: Discovery, precedence and the CLI

**Files:** Create `crates/freemapper-manifest/src/loader.rs`; modify `crates/freemapper-cli/src/main.rs` (`adapters list`, `--adapters-from-project`), `crates/freemapper-cli/src/adapters.rs` (registration list), `crates/freemapper-cli/src/mcp.rs` (never loads project adapters).

**Interfaces:** `Registry::with_dirs(dirs: &[AdapterDir]) -> (Registry, Vec<AdapterWarning>)`, `AdapterDir { path, origin: Builtin | User | Project }`; `freemapper adapters list [--json]`.

- [ ] **Step 1:** Write `folder_adapter_cannot_replace_a_builtin`, `broken_folder_adapter_is_skipped_with_file_and_line`, `shared_provider_shares_the_cache_folder` (Review Focus 4), `project_adapters_need_the_flag`, `mcp_ignores_project_adapters`, `adapter_dir_env_overrides_the_config_dir`, `config_dir_with_spaces_and_unicode_works`.
- [ ] **Step 2:** Implement; the config folder is `dirs::config_dir()/freemapper/adapters`.
- [ ] **Step 3:** Run, then try `freemapper adapters list` with a sample user folder on all three operating systems in CI.
- [ ] **Step 4:** Commit.

### Task D8: `tile-pattern` raster kind

**Files:** Modify `crates/freemapper-manifest/src/rasters.rs`, `crates/freemapper-sources/src/tile_pattern.rs`.

**Interfaces:** `[terrain] kind = "tile-pattern"`, keys `url` with `{zone}`, `{e}`, `{n}` placeholders, `grid_km`, `zone`, `format = "geotiff"`, `nodata`; fetched into the shared cache as a tile provider.

- [ ] **Step 1:** Write `pattern_expands_to_the_tile_urls_of_a_box` and `placeholder_values_are_percent_encoded`.
- [ ] **Step 2:** Implement over the existing tile cache and mosaic code.
- [ ] **Step 3:** Run the suite. Expected: PASS; no existing state uses the kind, so goldens are unchanged.
- [ ] **Step 4:** Commit. Optional: ship only if a first contributor needs it; otherwise leave it for stage G.

---

## Stage E: tooling

### Task E1: Fixtures format and the replay fetcher in the library

**Files:** Create `crates/freemapper-sources/src/replay.rs` (promoted from the test support of A3), `docs/adapters/fixtures.md` (outline only, filled in F1).

**Interfaces:** `ReplayFetcher::from_dir(dir)`; format: `fixtures/index.toml` with `[[response]] url, range, status, content_type, file`; files under `fixtures/` only, names `[0-9a-f]{64}.bin`.

- [ ] **Step 1:** Write `replay_rejects_index_entries_that_name_other_paths` and `replay_reports_unrecorded_requests_with_the_url`.
- [ ] **Step 2:** Move the code; A3's tests import it.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E2: `freemapper adapter check`

**Files:** Create `crates/freemapper-cli/src/adapter_check.rs`; modify `main.rs`.

**Interfaces:** `freemapper adapter check <dir> [--fixtures] [--live --bbox minLon,minLat,maxLon,maxLat] [--json]`; exit code 0 pass, 1 findings, 2 cannot run. Static checks: schema (D1), policy, boundary validity and CRS, boundary area plausible for the object-id prefix, credit present and licence known, `last_verified` not older than 12 months (warning), every `backend` resolvable, `rules.toml` valid. `--fixtures`: replay a fetch, derive and build of the golden area from `<dir>/fixtures/` and compare with `<dir>/expected.sha256` when present. `--live`: the same against the network on the given box, which must lie inside the boundary, printing one line per capability (`orthophoto ok 0.2 m, 1 piece`, `terrain FAILED: HTTP 404 …`).

- [ ] **Step 1:** Write `check_passes_the_hamburg_folder`, `check_fails_a_boundary_in_the_wrong_crs_with_code_boundary_crs`, `check_fails_a_symlinked_folder`, `check_json_output_lists_codes`, `live_box_outside_the_boundary_is_refused`.
- [ ] **Step 2:** Implement over D1, D3 to D5 and the replay fetcher.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E3: `freemapper adapter record`

**Files:** Create `crates/freemapper-cli/src/adapter_record.rs`.

**Interfaces:** `freemapper adapter record <dir> --bbox ...` runs a live fetch, derive and build on a 150 m box, writes `<dir>/fixtures/`, `<dir>/expected.sha256` and prints the sizes; refuses over 2 MB; sends no contact data (neutral user agent, as today).

- [ ] **Step 1:** Write `record_refuses_a_fixture_set_over_2_mb` and `recorded_fixtures_replay_to_the_same_hashes` (against the local stub server).
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E4: `adapter new` and `adapter boundary`

**Files:** Create `crates/freemapper-cli/src/adapter_new.rs`, `adapter_boundary.rs`.

**Interfaces:** `freemapper adapter new <id> [--kind manifest|backend|crate]` writes a commented manifest or a crate skeleton (`Cargo.toml`, `src/lib.rs` with the `RasterBackend` stub, a test, the line to add to `adapters.rs`); `freemapper adapter boundary <geojson> --out <file> [--crs 4326|25833]` simplifies to 25 m, reprojects to 25833 and validates.

- [ ] **Step 1:** Write `new_manifest_passes_check_after_filling_the_placeholders` (placeholders make `check` fail with `manifest.placeholder`), `new_crate_compiles` (in a temp workspace member), `boundary_output_is_valid_and_within_25_m_of_the_input`.
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

---

## Stage F: documentation

### Task F1: Contributor guide and manifest reference (bilingual)

**Files:** Create `docs/adapters/{en.md,de.md,manifest-v1.en.md,manifest-v1.de.md}`, `docs/adapters/tutorial.md` pair if length requires.

- [ ] **Step 1:** Write the ten sections of spec 3.17 in English, then German ("Bundesland" for states, glossary terms), with one runnable tutorial that adds a made-up state from the sample manifests and ends with `adapter check` green.
- [ ] **Step 2:** Add the pull-request checklist: manifest valid, `check` green with fixtures, credit text matches the licence page, `maintainers` and `last_verified` set, no personal data in fixtures, example project and manual row for a new state (house rule).
- [ ] **Step 3:** Run unslop and humanizer over the prose. Check every command in the guide by running it.
- [ ] **Step 4:** Commit `docs(adapters): contributor guide`.

### Task F2: Manual, README, weekly live workflow, release notes

**Files:** Modify `docs/manual/{de,en}.md` (regions chapter links to the guide), `README.md`, `README.en.md` (one sentence, link; re-check status per the pre-push rule), `CHANGELOG.md`, `CHANGELOG.en.md` (entry for the release that ships this, written when the work merges), `docs/adr/README.md` (status of ADR 0082 to Accepted); create `.github/workflows/adapters-live.yml` (weekly, runs `adapter check --live` for every built-in adapter, opens an issue on failure; neutral user agent).

- [ ] **Step 1:** Update the files; run the README re-check.
- [ ] **Step 2:** Dry-run the workflow with `workflow_dispatch` once.
- [ ] **Step 3:** Commit; after CI is green, release at a sensible point (auto-release rule).

---

## Stage G: new states (separate plans)

Not planned here. Each is a plan of its own because the service research comes first: Baden-Württemberg, Bremen (a hole in the Lower Saxony boundary today; its adapter removes the `reaches_beyond_coverage` special case), Bayern, Saarland. Each follows the house rules for a new state: manifest folder, example project, manual row, README row, changelog entry in German and English. They are the first test of the guide: if a stage-G task needs a file outside `crates/adapters/` or the registration line, the design failed and a finding goes back to the spec.

## Self-review

- Spec coverage: 3.2 crates (B1 to B4, C1), 3.3 trait (C1 to C13), 3.4 schema (D1, D3 to D5, D8), 3.5 samples (D1, D4, D5, D6), 3.6 limits (D5 backend errors, F1), 3.7 registry (C1, D7), 3.8 security (D2, D1, D7, C13 for the page), 3.9 normalisers (C6), 3.10 versions (D1, D7), 3.11 errors (D1, C14), 3.12 i18n (C13 names, D1), 3.13 ids (B3, C14, A5), 3.14 testing (A1 to A5, E1 to E3), 3.15 net and migration (A, C), 3.16 performance (A2, C2), 3.17 docs (F1, F2), 3.18 core (B1, B2). Gaps: none found; the `rules.toml` per adapter (3.8, D11) has a check in E2 but no loader task. Added to D3: load `rules.toml` through the existing rule validator and rank it below OSM.
- Type consistency: `AdapterId` (B3) is used in C1 to C14 and D; `RasterRecords` (B3) in C7; `Registry::backend` (D5) matches `RasterBackend` (C7); `UsedSources` (C4) and `EstimateArea` (C5) are defined where first used.
- Proportion: the plan is longer than the spec's staged table suggests because every task carries its tests; bodies are left to the implementer.

## Size and order

| Stage | Tasks | Estimated size | Can run in parallel |
|---|---:|---|---|
| A | 5 | 1 200 lines of tests, 8 to 12 MB of fixtures | A1, A2 together; A3 then A4 |
| B | 4 | 6 000 lines moved | B1 in core first |
| C | 14 | 4 000 lines moved, 1 500 new | C3 to C6 independent after C2; C8 to C12 independent after C7 |
| D | 8 | 2 500 lines new, 11 manifests | D1 and D2 together; D6 after D3 to D5 |
| E | 4 | 1 500 lines new | E1 then E2; E3, E4 after E2 |
| F | 2 | docs | after D |
| Total | 37 | | |

Recommended execution: subagent-driven. The tasks depend on each other's interfaces (B3's types, C1's trait), there are 37 of them, and a mistake in a move task changes example output silently, so an independent review per task earns its cost. Stage A is the one place where a human should look at the recorded fixtures before the rest starts.
