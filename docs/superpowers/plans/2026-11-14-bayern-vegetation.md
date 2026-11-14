# Bavaria: vegetation from the point clouds — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A map in Bavaria gets the vegetation candidates of ADR 0105 from the agency's laser point clouds.

**Architecture:** A sixth `PointCloudBackend` (`ldbv_las`) fetches the 1 km LAZ tiles by their computed address and keeps them with the freshness rules of Bavaria's other files (`ldbv_files`). The engine learns one thing: a provider can name classes that are no returns of vegetation. The day of the flight is read from the GPS time of the points.

**Tech Stack:** Rust workspace; crates `freemapper-adapter-api`, `freemapper` (engine), `freemapper-adapters-de`; `las` 0.11 with LAZ.

**Spec:** `docs/superpowers/specs/2026-11-14-bayern-vegetation-design.md`

## Global Constraints

- Address of a tile, exactly: `https://geodaten.bayern.de/odd_data/laser/<E>_<N>.laz` (`<E>`, `<N>`: south-west corner in km, EPSG:25832).
- Hosts of Bavaria after this plan, exactly: `geoservices.bayern.de`, `download1.bayernwolke.de`, `geodaten.bayern.de`. Manifest `revision = 2`.
- Cache: `<cache>/ldbv/las/<E>_<N>-<YYYYMMDD>.laz`; the note of a 404 is `<E>_<N>.missing`.
- `PointCloudInfo`: id `ldbv`, `cache_dir` `ldbv`, `source` `ldbv-las`, `epsg` 25832, `grid_km` 1, `left_out` `[22, 23]`.
- Estimate: 250 MB per tile. A download larger than 2 GiB is refused.
- No golden file changes for another state. A commit that changes a golden file carries `Golden-Bless: <reason>`.
- No new dependency. No request carries personal data; live checks use the program's own `User-Agent`.
- Source code and technical docs English, terms from `docs/glossary.md`; user docs German and English; German says "Bundesland".
- Disk is short: `CARGO_INCREMENTAL=0`, `cargo clean` after each build, `df` before a download.

## Review Focus

1. A tile with GPS week time (older lots): `published` must fall back to the day in its header and then to the day in the file name, never a date near 1980. (The header day was added during the build.)
2. A server that sends no `Last-Modified`: the copy is `<E>_<N>-00000000.laz`, `published` is `None`, and the tile is still used.
3. A download that is cut off or is an HTML page with status 200: nothing stays in the cache, the older copy serves.
4. Offline with nothing cached, and a 404 inside Bavaria: a note, never an error; the fetch of the map goes on.
5. A project whose recorded tile was replaced by a newer copy: `cached(record)` still finds a file.

---

### Task 1: A provider names the classes it leaves out

**Files:**
- Modify: `crates/freemapper-adapter-api/src/pointcloud.rs` (`PointCloudInfo`)
- Modify: `crates/freemapper/src/derive/vegetation.rs` (`read_tile`, `CellCounts::add_tile`, tests)
- Modify: `crates/freemapper/src/project.rs` (the derive step passes the list)
- Modify: `crates/adapters/de/src/{lgb_als,nrw_3dm,lvermgeo_rp_las,berlin_als,lvgl_las}.rs` (`left_out: Vec::new()`)

**Interfaces:**
- Produces: `PointCloudInfo::left_out: Vec<u8>`; `read_tile(path: &Path, dtm: &Raster<f32>, left_out: &[u8], each: &mut dyn FnMut(VegetationPoint))`; `CellCounts::add_tile(&mut self, path, dtm, mask, left_out: &[u8])`.

- [ ] **Step 1: Write the failing test** in `vegetation.rs`, next to `read_tile_measures_above_the_dtm_and_skips_buildings`:
  `read_tile_leaves_out_the_classes_the_provider_names`: a LAZ file (LAS 1.2, format 1) with three points 3 m above a flat DTM, classes 20, 22 and 23. `read_tile(.., &[], ..)` hands over 3 points, `read_tile(.., &[22, 23], ..)` hands over 1.
- [ ] **Step 2: Run it** — `cargo test -p freemapper --lib read_tile_leaves_out` — Expected: does not compile (`read_tile` takes 3 arguments).
- [ ] **Step 3: Implement.** Field `left_out` with the doc line "Classes of this provider that are no returns of what grows there, besides those `derive` leaves out for every provider." `read_tile` skips a point whose class number is in `left_out`. `add_tile` passes it through. In `project.rs` the derive step takes the list from `point_cloud_backend(..)`'s `info()` (empty without a backend). The five backends set `left_out: Vec::new()`.
- [ ] **Step 4: Run** `cargo test -p freemapper --lib vegetation` and `cargo test -p freemapper-adapters-de --lib point_cloud` — Expected: PASS.
- [ ] **Step 5: Commit** `feat(vegetation): a point-cloud provider names the classes it leaves out`.

### Task 2: A LAS download without a listed size, and the day of the flight

**Files:**
- Modify: `crates/adapters/de/src/las_files.rs`
- Modify: the callers of `download_las` (`lgb_als.rs`, `nrw_3dm.rs`, `lvermgeo_rp_las.rs`) if its signature changes

**Interfaces:**
- Produces: `pub(crate) fn download_las(env, url: &str, bytes: Option<u64>, sha256: Option<&str>, path: &Path) -> Result<(), FetchError>` (`None`: no listed size, at most 2 GiB, the completeness check decides); `pub(crate) fn flight_day(path: &Path, today: NaiveDate) -> Option<NaiveDate>`.

- [ ] **Step 1: Write the failing tests** in `las_files.rs`:
  - `flight_day_is_the_newest_day_of_the_points`: a LAZ file with adjusted standard GPS time whose 40 points lie on 2021-12-19 and 2021-12-20 (GPS seconds since 1980-01-06 minus 10⁹) gives `2021-12-20`.
  - `flight_day_is_unknown_with_week_time`: the same points with GPS week time in the header give `None`.
  - `flight_day_is_unknown_outside_the_plausible_years`: points at GPS time 0 (adjusted: 2011-09-14) with `today` = 2011-01-01 give `None`; a file without points gives `None`.
  - `a_download_without_a_listed_size_is_checked_for_completeness`: `download_las(.., None, None, ..)` keeps a complete LAZ file; for the bytes `<html>not a tile</html>` it returns `CorruptArchive` and leaves neither the file nor a `.part`.
- [ ] **Step 2: Run** `cargo test -p freemapper-adapters-de --lib las_files` — Expected: does not compile (`flight_day` missing).
- [ ] **Step 3: Implement.** `flight_day`: header `gps_time_type()` must be `Standard`; 16 positions `i * (n − 1) / 15`, `seek` + `read_points(1)`; day = 1980-01-06 + floor((t + 10⁹) / 86 400) days; keep days from 1995-01-01 to `today`; the newest. Any read error gives `None`.
- [ ] **Step 4: Run** the tests of step 1 and `cargo test -p freemapper-adapters-de --lib` — Expected: PASS.
- [ ] **Step 5: Commit** `feat(adapters): a LAS download without a listed size; the flight day from the GPS time`.

### Task 3: The backend of Bavaria

**Files:**
- Create: `crates/adapters/de/src/ldbv_las.rs`
- Modify: `crates/adapters/de/src/lib.rs` (module), `crates/adapters/de/src/states/mod.rs` (`point_cloud`), `crates/adapters/de/src/lgb_als.rs` (test `five_states_have_point_clouds` becomes `six_states_have_point_clouds`), `crates/adapters/de/manifests/bayern/manifest.toml` (host, revision)
- Modify: whatever test names the hosts or the revision of Bavaria (`crates/adapters/de/tests/manifests.rs`, `crates/freemapper-cli/tests/*.rs`, golden files of the adapters)

**Interfaces:**
- Consumes: `ldbv_files::{Published, Checks, Current, current, cached, drop_older}`, `las_files::{download_las, flight_day, utm32_tile_name, parse_utm32_tile_name, check_las_file}`, `boundaries::{tile_centre_in, BAYERN}`, `PointCloudInfo::left_out`.
- Produces: `pub fn backend() -> &'static dyn PointCloudBackend`, `pub fn tile_url(tile: TileId) -> String`, `pub const MB_PER_TILE: f64 = 250.0`. (The planned `test_support` module was not built: no engine test uses it; the helpers are `las_files::testing`.)

- [ ] **Step 1: Write the failing tests** in `ldbv_las.rs` (tile 655/5479, a box inside it; `MapFetcher` with `Last-Modified` 2023-03-10; the synthetic tile's points measured on 2021-12-20):
  - `the_address_follows_the_agency`: `tile_url` gives `https://geodaten.bayern.de/odd_data/laser/655_5479.laz`.
  - `a_fetch_downloads_the_tile_and_records_the_flight_day`: one tile, record `{tile: "32655-5479", url, published: 2021-12-20}`, file `655_5479-20230310.laz` in `ldbv/las`.
  - `a_second_fetch_asks_only_for_the_day`: the log of the second fetch has no `GET` of the tile.
  - `a_newer_file_replaces_the_cached_one`: `Last-Modified` 2026-09-03 → the new copy is there, the old one gone.
  - `a_tile_with_week_time_is_dated_by_the_server`: `published` is 2023-03-10.
  - `a_server_without_a_day_still_gives_the_tile`: copy `655_5479-00000000.laz`, `published: None`.
  - `a_broken_download_falls_back_to_the_cached_copy`: the newer file is an HTML page → the tile of the old copy comes back, a warning `warning-point-cloud-unavailable-cached`, no new file.
  - `a_broken_download_without_a_copy_leaves_the_tile_out`: no tile, warning `warning-point-cloud-unavailable-skipped`, `Ok`.
  - `a_tile_the_server_lacks_is_a_note`: 404 inside Bavaria → no tile, `progress-point-cloud-not-published`, `Ok`; the next fetch asks again. For a tile beyond the border (500/5510) the second fetch within 30 days asks nothing.
  - `offline_serves_the_cached_tile_and_warns_without_one`: offline with a copy → the tile, no request; without → `warning-point-cloud-no-cached`, `Ok`.
  - `a_silent_server_costs_one_check`: two cached tiles, `HEAD` fails → both tiles served, one `warning-file-check-failed`, one `HEAD` in the log.
  - `the_recorded_tile_is_found_after_a_newer_copy_came`: `cached(record)` gives the newer file.
  - `the_estimate_counts_250_mb_a_tile`: two tiles under a box across a tile border, `cached` true for the one with a copy, part `PointCloud`, store `Cache`, what `LDBV point cloud 32655-5479`.
  - In `lgb_als.rs`: `six_states_have_point_clouds` with `State::Bayern` in the 25832 arm.
- [ ] **Step 2: Run** `cargo test -p freemapper-adapters-de --lib ldbv_las` — Expected: does not compile (module missing).
- [ ] **Step 3: Implement** the backend as the spec's section "The backend" says. One `Checks` per fetch. `missing_holds: !tile_centre_in(tile, BAYERN)`. A `Cancelled` from any step is returned as the error. After the loop `freemapper_fs::remove_stale_cache_parts`. Register it in `states/mod.rs`; add the host and `revision = 2` to the manifest; set `last_verified = "2026-11-14"` (unchanged).
- [ ] **Step 4: Run** `cargo test -p freemapper-adapters-de` and `cargo test --workspace --locked` (output to a file, read its tail) — Expected: PASS; a test that names Bavaria's hosts or revision is updated to the three hosts and revision 2. `scripts/golden.sh test` — Expected: PASS, or only golden files that hold Bavaria's manifest data differ; bless those in this commit with `Golden-Bless: Bavaria asks a third host for its point clouds`.
- [ ] **Step 5: Commit** `feat(bayern): vegetation from the point clouds of the LDBV`.

### Task 4: Live run and the example (gate)

**Files:**
- Modify: `examples/bayern-schmausenbuck/*` (rebuilt)
- Modify: `docs/regions/bayern-reference.md` (section "Live run": what the vegetation run gave)

- [ ] **Step 1:** `df -h /` — at least 6 GB free, else stop and ask. Build the release program, copy it to the scratch folder, `cargo clean`.
- [ ] **Step 2:** `CI=1 freemapper run examples/bayern-schmausenbuck --force --pdf --pdf-candidates --ocad --lang en --content-set vegetation=candidates`. Expected: exit 0, four `ldbv` point-cloud tiles in `freemapper.toml` with `published = "2021-12-20"` (or the day the points give), `data/vegetation.geojson` written.
- [ ] **Step 3: Gate.** Render the candidates PDF and compare it with the orthophoto template: forest (405) under the crowns, open land in the clearings and under the power lines, green classes in between; no class covers the whole forest; the power lines are no forest strips over open ground. Count the vegetation candidates by symbol. If the gate fails: stop, keep the pictures, report to the user.
- [ ] **Step 4:** A second run with `--offline --force` writes the same candidates. Delete `*.bak` and the map-only PDF, render `preview.webp` as for the other examples, run `cargo test -p freemapper --test adapters_golden`.
- [ ] **Step 5: Commit** `docs(examples): the Schmausenbuck with the vegetation from the point clouds`.

### Task 5: Texts

**Files:**
- Create: `docs/adr/0128-bavaria-vegetation-from-the-point-clouds.md`; Modify: `docs/adr/README.md`, `docs/adr/0105-vegetation-from-point-clouds.md` (one line that names the sixth source and points to 0128)
- Modify: `docs/manual/de.md`, `docs/manual/en.md` (section "Vegetation aus Laserpunktwolken" / its English twin)
- Modify: `README.md`, `README.en.md`, `AGENTS.md` if it lists the states with vegetation
- Modify: `crates/freemapper-cli/i18n/{de,en}/freemapper.ftl` (`page-content-group-vegetation`) and the tests that quote it
- Modify: `examples/README.md`, `examples/README.en.md` (row and paragraph of the forest examples)
- Modify: `CHANGELOG.md`, `CHANGELOG.en.md` (under Unveröffentlicht / Unreleased, "Hinzugefügt" / "Added")
- Modify: `docs/regions/bayern-reference.md` ("Open, and left for later" no longer lists the point clouds; done in the research commit)

- [ ] **Step 1:** Write the texts with the numbers of the live run (sizes, tiles, flight day, counts). The manual says what a tile costs in Bavaria (meist 100 bis 600 MB) and that the date on the candidates is the day of the flight.
- [ ] **Step 2:** Run `unslop` and `humanizer` over the new user texts.
- [ ] **Step 3:** `cargo test --workspace --locked`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/golden.sh test` — Expected: PASS.
- [ ] **Step 4: Commit** `docs: Bavaria's vegetation in the manual, the READMEs and the changelog (ADR 0128)`.
