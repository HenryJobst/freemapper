# Region adapters implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `Region` and `Provider` enums and every `match` on them with a `RegionAdapter` trait, a registry, TOML manifests and adapter crates, so that a new country or state is a folder (or one small crate), with all eleven existing states producing identical output.

**Architecture:** First an injectable clock and a five-layer offline golden net recorded from v0.11.0 (stage A); the recorded answers of the services live in a fixture store outside the repository. Then the shared types, the notes and the readers move into `freemapper-adapter-api` and `freemapper-sources`, and the ten raster fetch functions get their adapter-shaped parameters while they still live in `project.rs` (B). A registry with a shim over the old enums takes over one concern at a time until the enums can be deleted (C). The manifest crate, the URL policy and the loader come next (D), then `freemapper adapter check` (E), one new state as the test of the design, and the bilingual guide (F). New states are separate plans (G).

**Tech Stack:** Rust 1.99.1 (pinned toolchain), `toml`, `serde` with `deny_unknown_fields`, `ureq` 3.4.2 (pinned exactly), `geo`, `chrono`, from freemapper-core (tag v0.18.0) `freemapper-fs`, `freemapper-georef`, `freemapper-text` and `freemapper-contract`, GitHub Actions on three operating systems.

**Spec:** [`docs/superpowers/specs/2026-11-10-region-adapters-design.md`](../specs/2026-11-10-region-adapters-design.md), ADR [0082](../../adr/0082-region-adapters-trait-and-manifests.md). Section numbers below refer to the spec.

**Revision 2026-11-11.** Reviewed against `main` at v0.10.0 (c393575), 98 commits after the first version. Changes: B1 and B2 are done (georef in core v0.5.0, freemapper pins v0.11.0); the goldens are recorded from v0.10.0, not v0.9.0; new task A0 makes the clock injectable, because about 25 calls of `chrono::Local::now()` and `SystemTime::now()` would make G3 and G4 change from day to day; G4 calls the library functions of ADR 0091 and also hashes the `.ocd`, the PDFs and the written `freemapper.toml`; the goldens pin the program language to English (ADR 0092); the golden guard checks commit subjects on every push, not pull-request titles; the trait keeps the cancel of ADR 0090; tasks that touch `select.rs`, `web/` or `mcp.rs` are agreed with the session that owns them; D8, E3, E4 and the weekly live workflow are deferred; one new state follows stage E before the guide. The worktree of the first attempt (`08a5fcd`, 98 commits behind) is not continued: start from `main`, cherry-pick `08a5fcd`, bless G1 again and read the diff; the half-written G2 test is dropped.

**Revision 3, 2026-11-11 (evening).** Reviewed against `main` at v0.11.0 (`3b95e7a`). The review measured the code and the cache instead of trusting the first numbers, and six things did not hold:

- The fixtures do not fit into the repository. One tile set of Brandenburg is about 37 MB (orthophoto 6.8, DGM 1.2, bDOM 28.6), of Saxony about 27 MB, and a tile cannot be cut to the golden area. The recorded answers therefore live in a fixture store outside the repository, pinned by SHA-256 (A3), and G4 runs on from the replayed fetch of G3 instead of keeping a second copy as a cache (A4).
- The clock task looked for the wrong calls. The two age checks of Rhineland-Palatinate read a file's age through `modified()` and `elapsed()`, not through `SystemTime::now()` (A0).
- Nothing showed that the outputs are byte-identical on three operating systems. A pilot with two states runs through the CI before the other nine are recorded (A4).
- The raster fetch functions cannot be moved as they are: they take the project folder and the project file, save in between, report through `note!`, get `skip_surface` from the disk-space plan and call ten shared helpers of `project.rs`. New task B5 gives them adapter-shaped parameters in place, so C7 to C12 are moves again. The notes move to the api crate in B3, because the readers of B4 need them too.
- `TileRecord` and `FetchedTile` carry a `Product`, so `Product` belongs to the api crate (B3), not to the DE adapter crate (C7). The map kind of `ExtraLayer` is the one of `freemapper-contract` (ADR 0095).
- Four providers keep their cache folder under a name that is not their id (`lvermgeo_sh`, `geobasis_nrw`, `lvermgeo_rp`, `laiv_mv`). The folder name is a field of its own and part of G1 (A1, B5).

Also updated: core pins (v0.18.0), the owners of the CLI files (the self-update session now), the five places that list the states and the content security policy (C13), the hosts and the redirect check of the self-update (D2), help texts, catalogs, AGENTS.md and manual for the new commands (D7, E2), the unit tests that name the enums (recipe of stage C), a guard by trailer instead of by subject prefix (A5), and how long the net stays (F2). 34 active tasks.

**Start condition.** Stage A starts after the self-update branch (ADR 0094, pull request 36) is on `main`: it edits `main.rs`, the catalogs, `mcp.rs`, `select.rs` and `web/`, and it brings an HTTP source of its own that D2 has to cover. Publishing the fixture store (A3, step 6) is a release on the public repository and is part of the user's go for stage A.

## Global Constraints

- Every task before stage D changes no output: `cargo test --workspace --locked` and the golden layers G1 to G5 stay green, and no file under `tests/golden/adapters/` changes in a commit whose subject starts with `refactor(` (the CI guard of A5 enforces it on every push and pull request).
- G1, G2 and G5 need only the repository and run with every `cargo test`. G3 and G4 need the fixture store and are `#[ignore]`d; `scripts/golden.sh test` fetches the store when it is missing and runs all five layers (`cargo test -p freemapper --test adapters_golden -- --include-ignored`). Run it before every commit of stages B to D. With the store missing G3 and G4 fail and name the script; they never pass by skipping.
- Once stage A is on `main`, a commit that changes output on purpose (a new symbol, a fix) blesses the goldens with `FREEMAPPER_BLESS=1` in the same commit, names the changed golden files in its message and carries the trailer `Golden-Bless: <reason>`. A commit that adds or changes a request records the states concerned again (`scripts/golden.sh record <id>`), publishes the next store version and moves `store.lock` in the same commit. The contract fixtures of freemapper-core (ADR 0089) are a second net with its own rule. Tell the other sessions when stage A lands.
- Stages B5 to C14 run in one stretch. While it runs, work on `main` that adds a data source or changes a request waits, or carries its own recording. Tell the other sessions when the stretch starts and when it ends.
- The goldens run with the fixed clock of A0 (`2026-11-11`) and render notes in English (`Lang::english()`); a golden never depends on the day, the time zone or `FREEMAPPER_LANG`.
- A cancel (`FetchError::Cancelled`, ADR 0090) passes through every new error type unchanged and still ends the run as a cancel.
- `crates/freemapper-cli/src/main.rs`, `mcp.rs`, `select.rs`, `select_run.rs`, `help.rs`, `web/` and the catalogs under `crates/freemapper-cli/i18n/` were last changed by the self-update session (session "freemapper"); the program-language work is finished. Before a task edits them, check `gh pr list` and the worktrees for open work on these files, send that session a message with the files and the planned change, and rebase on its latest merge.
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
6. A cancel from the browser page while a raster backend downloads: the run stops as a cancel, not as a fetch failure with a misleading message. Test in C7.
7. A fetch that fails halfway (a service down after the orthophoto, the network gone during the surface model): the project file records exactly the parts that finished, as today, so the next fetch goes on from there. Test in B5 (`g3-interrupted.json`).
8. A disk with room for the terrain model but not for the surface model: the surface is skipped with today's warning and no request for it goes out. Test in B5.

---

## Stage A: the net

### Task A0: An injectable clock

**Files:**
- Create: `crates/freemapper/src/clock.rs`
- Modify: `crates/freemapper/src/lib.rs`, `crates/freemapper/src/project.rs` (12 production calls of `chrono::Local::now()`; the other 11 are in its tests), `crates/freemapper/src/sources/lvermgeo_rp.rs` (`known_missing` and `fresh_metalink`: they read a file's age through `modified()` and `elapsed()`, which asks the system clock without naming it; the `SystemTime::now()` calls of that file are all in tests)

**Interfaces:**
- Produces in `freemapper::clock`: `pub fn today() -> NaiveDate`, `pub fn now() -> SystemTime` and `pub fn age_of(path: &Path) -> Option<Duration>` (the clock's now minus the file's modification time; `None` when the file or its time cannot be read). A file dated after the clock's now has age zero: a file a test writes under the fixed clock carries today's real time and must count as fresh. By default the three read the system clock. `pub fn with_fixed<R>(at: NaiveDateTime, f: impl FnOnce() -> R) -> R` (a process-wide override, because the fetch code may download on worker threads; `with_fixed` holds a static mutex, so tests that fix the clock run one after another) behind `#[cfg(any(test, feature = "golden"))]`.
- Left alone on purpose: the CLI crate's clock calls (run lock, `select_run.rs`, the self-update check; they write no project file); `freemapper_fs::remove_stale_cache_parts` in core, which only deletes abandoned partial downloads; `hvbg::kept_lists` and the overview lists of `lvermgeo_sh`, which order files by modification time and never compare with now (A3 checks that a replay leaves one list per kind, so the order cannot matter).

- [ ] **Step 1:** Write `fixed_clock_dates_the_records` (a fetch with a stub fetcher inside `with_fixed(2026-11-11T12:00)` writes `fetched = 2026-11-11` into the records), `rp_rechecks_missing_tiles_after_30_days_by_the_clock` (the existing 30-day tests of the 404 note and of the kept Metalink, driven by the clock: the file is written now and the clock is fixed 31 days later) and `a_file_dated_after_the_clock_is_fresh` (`age_of` gives zero). Run: FAIL (module missing).
- [ ] **Step 2:** Add the module and replace the production calls one by one. No other change; existing tests must pass unchanged.
- [ ] **Step 3:** Run `cargo test --workspace --locked`. Expected: PASS. Add a test `no_direct_clock_calls` that reads `crates/freemapper/src/**/*.rs` and fails on `Local::now(`, `Utc::now(`, `SystemTime::now(` or `.elapsed()` outside `clock.rs` and test modules (today `.elapsed()` occurs in production code only at the two sites of `lvermgeo_rp.rs`).
- [ ] **Step 4:** Commit `refactor(clock): one injectable clock for dates the project stores`.

### Task A1: Glossary rows and golden G1 (registry data)

**Files:**
- Modify: `docs/glossary.md` (rows Regionsadapter, Adaptermanifest, Rasterbackend, Adapterprüfung; row "Quelladapter" points to Regionsadapter)
- Create: `crates/freemapper/tests/adapters_golden.rs`, `tests/golden/adapters/g1-regions.json`
- Create: `crates/freemapper/src/sources/region.rs` test-only helper `pub fn legacy_facts() -> Vec<RegionFacts>` behind `#[cfg(any(test, feature = "golden"))]`

**Interfaces:**
- Produces: `RegionFacts { id, provider, cache_dir, terrain_source, per_area, crs_epsg, object_id_prefix, boundary_sha256, osm: Vec<OsmFacts>, credit: Vec<(year, text)> }`, serialisable; the JSON file is its pretty print. `cache_dir` is the provider's folder in the shared cache as `Cache` names it today: `lgb`, `geosn`, `lgv`, `hvbg`, `lvermgeo_sh`, `geobasis_nrw`, `lvermgeo_rp`, `laiv_mv`, and none for `lvermgeo` and `lgln`, which keep nothing there. Four of these names are not the provider id (`lvermgeosh`, which `Product::provider_dir_name` returns and no folder uses). A user's cache holds several GB under them, so they are recorded.

- [ ] **Step 1:** Add the glossary rows (German and English columns, `code` column names `RegionAdapter`, `ManifestAdapter`, `RasterBackend`, `adapter check`).
- [ ] **Step 2:** Write test `g1_registry_data_is_unchanged` that builds `legacy_facts()` from the enums and compares with `tests/golden/adapters/g1-regions.json`; with `FREEMAPPER_BLESS=1` it writes the file.
- [ ] **Step 3:** Implement `legacy_facts()` from `Region::ALL`, `Region::provider`, `Provider::terrain_source`, `Provider::per_area`, `Region::crs`, `ExtractSource::{url, state_url, prefix, megabytes, label}` (NRW per district), SHA-256 of each boundary constant, and the credit lines of the ten `*_attribution` functions for 2025 and 2026 (build a `ProjectFile` with records dated in that year).
- [ ] **Step 4:** Run `FREEMAPPER_BLESS=1 cargo test -p freemapper --test adapters_golden g1` once on `main` after A0, check the JSON by eye (11 regions, 10 providers, 5 NRW extracts), run again without the variable. Expected: PASS.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 1, registry data` with the glossary.

Head start: commit `08a5fcd` (branch `worktree-agent-a166cbc944727a3f2`) did steps 1 to 5 on the tree of 2026-11-10. Cherry-pick it onto a new branch from `main`, resolve conflicts in `project.rs`, `osm.rs` and `region.rs`, add the `cache_dir` field, bless again and read the JSON diff: besides the new field, a difference is a change made between the two trees and must be explained in the commit message before going on.

**Net:** this task creates G1. **Outcome:** a file that says what the enums answered.

### Task A2: Golden G2 (geometry and dispatch)

**Files:**
- Modify: `crates/freemapper/tests/adapters_golden.rs`
- Create: `tests/golden/adapters/g2-geometry.json`, `tests/golden/adapters/g2-boxes.json` (the 211 boxes: 11 example areas read from `examples/*/freemapper.toml`, 200 border and coastal boxes)

**Interfaces:** consumes `regions_for`, `raster_region`, `provider_tiles(bbox, provider, km)`, `reaches_beyond_coverage`, `tile_touches`, `tile_centre_in`, `osm::extracts_for` (all public today).

- [ ] **Step 1:** Write `g2_geometry_is_unchanged`: for each box, record `regions_for` ids, `raster_region` id, `provider_tiles` for km 1 and 2 (names), `reaches_beyond_coverage`, `extracts_for` labels. Compare with the JSON.
- [ ] **Step 2:** Add `g2_dispatch_is_fast_enough`, marked `#[ignore]` so it never runs on the shared CI runners: all 211 boxes complete within twice the v0.11.0 measurement on the maintainer's machine (written into the test as a constant after measuring). It is run by hand after C2 and C14 (`cargo test --release -p freemapper --test adapters_golden g2_dispatch -- --ignored`).
- [ ] **Step 3:** Generate the 200 boxes deterministically in the test (seeded; points along the borders BB/BE, ST/NI, NI/Bremen, HE/RP, NW/NI, MV/SH, MV/BB, SN/BB, HH/SH, HH/NI, sizes 150 m to 1.5 km); bless once on `main`.
- [ ] **Step 4:** Run the test. Expected: PASS twice (second run without bless).
- [ ] **Step 5:** Commit `test(adapters): golden net layer 2, geometry and dispatch`.

### Task A3: The fixture store, replay and recording fetchers, golden G3 (request trace)

The answers of the services are too large for the repository: a tile comes whole (Brandenburg about 37 MB for one square, Saxony about 27 MB, Rhineland-Palatinate two Metalinks of 11 MB each), and eight of the eleven states fetch tiles. The repository keeps what a reviewer reads (index, trace, hashes); the bodies live in a store.

**Files:**
- Create: `crates/freemapper/tests/support/replay.rs` (`RecordingFetcher`, `ReplayFetcher`), `tests/golden/adapters/README.md` (the store, how to record, how to bless), `tests/golden/adapters/empty.osm.pbf` (a valid PBF with a header block and no data, under 300 bytes), `tests/golden/adapters/store.lock` (TOML: `version`, `url`, `sha256`, `bytes` of the store archive), `scripts/golden.sh` (`fetch`, `test`, `record <id>`, `pack`), `tests/golden-store.sh` (test of `fetch`)
- Create per state, in the repository: `tests/golden/adapters/<id>/index.toml` (`[[response]]` with `url`, `range`, `status`, `content_type` and `body`, the SHA-256 of the answer), `<id>/trace.txt`
- In the store, not in the repository: `responses/<sha256-of-the-body>.bin` (one pool for all states, so an answer two states share is kept once), `<id>/osm-candidates.geojson` (for G4), `<id>/ATTRIBUTION.txt`
- Modify: `crates/freemapper/tests/adapters_golden.rs`

**The store:** one archive `golden-adapters-<version>.tar.gz`, the asset of a GitHub prerelease `golden-adapters-<version>` of this repository (`gh release create golden-adapters-1 --prerelease --latest=false`; `release.yml` reacts to `v*` tags only). `scripts/golden.sh fetch` downloads the archive named in `store.lock` with plain `curl` (no token, no extra header), refuses it when the SHA-256 differs and unpacks it to `target/golden-adapters/<version>/`; `FREEMAPPER_GOLDEN_STORE` names another folder. Expected size 120 to 180 MB; measure it at the recording and write the number into the README. A store version is never changed after it is published; a new recording is a new version. The prerelease carries the archive and nothing else, above all no file named `freemapper-update.json`: the self-update reads that name from `releases/latest/download/`.

**Interfaces:**
- Produces: `ReplayFetcher::new(index: &Path, store: &Path) -> Self` implementing `Fetcher` (including `get_range`, `last_modified`, `download_to`; OSM `.pbf` URLs answer with `empty.osm.pbf`); `RecordingFetcher::new(inner: HttpFetcher, index: &Path, store: &Path)` active only when `FREEMAPPER_GOLDEN_RECORD` is set. Both log `METHOD url [range]` lines to a `Vec<String>` readable as `trace()`.
- `g3_*` and `g4_*` tests carry `#[ignore = "needs the fixture store: scripts/golden.sh test"]`. Run with `--include-ignored` and no store, they fail with that sentence.
- Notes are compared as their English rendering (`Lang::english()`), so the goldens do not move when a plain line gets a message id with the same English text.

- [ ] **Step 1:** Write `replay_returns_recorded_bytes_and_logs_requests` (a two-URL index and a store folder made in the test, one range request, one unknown URL returns `FetchError::Http` with status 404 and appears in the trace; this test needs no published store and is not ignored) and `tests/golden-store.sh` (`fetch` unpacks a throwaway archive whose hash matches a throwaway lock and refuses one whose hash differs).
- [ ] **Step 2:** Implement both fetchers and the script. The index key is the URL plus the range; the user agent and headers are not recorded. The recorder refuses to run when any URL contains `@` or when the user agent it would send contains `@` (no personal data in requests).
- [ ] **Step 3:** Record one online `fetch_project` plus `estimate_download` per state on the golden area (a 150 m box in the example's area) with the maintainer's network, inside `clock::with_fixed(2026-11-11T12:00)`. Record Saxony-Anhalt and Hesse first and take them through the pilot of A4 (its step 2) before the other nine. Check `df` first. The recording starts with empty tile folders, so every tile is requested and recorded; it may start with the maintainer's cached OSM extracts, so no extract of 50 to 500 MB is loaded again (the index answers the extract URL with `empty.osm.pbf` anyway). Copy the `data/osm-candidates.geojson` of that run, read from the real extract, into the store. Put the Brandenburg and the Berlin area into the same 1 km square at the border if one with enough objects on both sides exists; they then share the LGB tiles. Brief for whoever runs it: no e-mail address in any request, header or file. If a state's service fails during the recording (Saxony-Anhalt's aerial WMS failed on 2026-11-11), retry once on another day; if it still fails, record what answers, mark the state in `tests/golden/adapters/README.md` as "partial, re-record" and go on: G3 then pins the failing request and the warning, which is behaviour too. Write `trace.txt` from the replay of the recording.
- [ ] **Step 4:** Write `g3_trace_<id>` for each of the 11 states (one test function generated by a macro): replay the fetch against an empty project folder and an empty cache under the fixed clock, compare the trace and the `warning:` lines with `trace.txt`. After the replay of Hesse and Schleswig-Holstein, assert that each list folder of the cache holds one list per kind; if a state keeps two, break the tie of equal modification times by file name in a `fix(...)` commit before blessing.
- [ ] **Step 5:** Run `scripts/golden.sh test`. Expected: 11 `g3` tests PASS. Put an `ATTRIBUTION.txt` into each `<id>/` folder of the store with the state's credit line and licence, and the OpenStreetMap line (ODbL) for the candidates file: the store republishes state data and OSM data.
- [ ] **Step 6:** `scripts/golden.sh pack`, publish the archive as the prerelease `golden-adapters-1` and write `store.lock`. Then check that the newest program release is still the latest one: `gh release view golden-adapters-1 --json isPrerelease` says true, and `https://github.com/HenryJobst/freemapper/releases/latest` still leads to the newest `v*` tag. The install steps of AGENTS.md and the self-update read `latest`.
- [ ] **Step 7:** Commit `test(adapters): golden net layer 3, request traces` with the trailer `Golden-Bless: first recording`.

### Task A4: Golden G4 (pipeline per state), with a pilot on three systems

**Files:**
- Create per state: `tests/golden/adapters/<id>/expected.sha256`
- Modify: `crates/freemapper/tests/adapters_golden.rs`, `.github/workflows/ci.yml` (job `golden`)

**Interfaces:** consumes the library functions the CLI pipeline (`crates/freemapper-cli/src/pipeline.rs`) calls: `fetch_project`, `derive_project`, `starter_documents` and the four `export_*_from` functions of ADR 0091 (`export_starter_project_from`, `export_ocad_from`, `export_pdf_from`, `export_candidates_pdf_from`). The test does not call the CLI crate; if `pipeline::run` does a step the library functions miss, the test says which and repeats it. G4 keeps no project folder and no cache of its own. It goes on from the replayed fetch of G3, so every tile is stored once, and every file the run reads was written by the run (no modification time comes from a checkout).

- [ ] **Step 1:** Write `g4_pipeline_<id>` (ignored like G3), all under `clock::with_fixed(2026-11-11T12:00)`: replay the online fetch into an empty project folder and an empty cache; run `fetch_project` with `FetchMode::Offline` on the result and assert that `freemapper.toml` keeps its bytes; replace `data/osm-candidates.geojson` by the store's file (the replay answered the extract with the empty PBF; the candidates of the real extract keep the OSM objects in the outputs, and the adapter work does not touch the PBF reader); `derive_project`; build the documents once and write all four outputs; compute SHA-256 of `freemapper.toml`, `<name>.omap`, `<name>.ocd`, the map PDF, the candidates PDF, `templates/candidates.omap`, `ATTRIBUTION.txt`, `data/contours.geojson`, `templates/hillshade.png`, `slope.png`, `orthophoto.jpg`; compare with `expected.sha256`. If an output turns out not to be deterministic on one machine, find the varying field and fix it (a production fix in its own `fix(...)` commit before the bless), never by leaving the output out.
- [ ] **Step 2 (pilot):** Do step 1 for Saxony-Anhalt (a per-area state) and Hesse (tiles, zip ranges, the metadata xlsx) only, add the job `golden` of step 4, publish these two as store version 0 and push the branch as a draft pull request. `derive` and `render` call `sin`, `cos`, `atan`, `exp` and `hypot` of each system's maths library, so equal bytes on Linux, macOS and Windows are a claim to test, not a given. Rule for a difference between the systems:
  - `freemapper.toml`, the two `.omap`, `.ocd`, `ATTRIBUTION.txt` and the PDFs must be byte-identical on all three. If they are not, round the value where it is written (a `fix(...)` commit in production code, before the bless).
  - `hillshade.png`, `slope.png`, `orthophoto.jpg` and `contours.geojson` may fall back, per file kind and only where the pilot shows a difference: `expected.sha256` holds the hash of the recording system (macOS), and on a system where the hash differs the test compares the decoded content (pixels within 1 of 255 per channel, coordinates within 1 mm). The test's header and the README name each kind that uses the fallback.
  - No output leaves the net.
- [ ] **Step 3:** Record the other nine states (A3, step 3), bless, and run twice to prove determinism. Expected: PASS twice on the same machine.
- [ ] **Step 4:** The job `golden` in `.github/workflows/ci.yml` runs on Linux, macOS and Windows: restore `target/golden-adapters/` with `actions/cache`, keyed by the hash of `store.lock`; `bash scripts/golden.sh test`. Expected: green on all three.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 4, pipeline` with the trailer `Golden-Bless: first recording`.

### Task A5: Golden G5 (persisted format) and the CI guard

**Files:**
- Modify: `crates/freemapper/tests/adapters_golden.rs`, `.github/workflows/ci.yml`
- Create: `.github/scripts/check-golden-untouched.sh`

- [ ] **Step 1:** Write `g5_examples_roundtrip`: for each `examples/*/freemapper.toml`, `ProjectFile::load` then `save` to a temp folder; bytes equal the original.
- [ ] **Step 2:** Write `g5_unknown_provider_is_a_clear_error` (ignored until C14): a toml with `provider = "czfoo"` loads with a message that contains `czfoo` and the word `adapter`.
- [ ] **Step 3:** The script walks every commit of the pushed range (`push`: `github.event.before..github.sha`; `pull_request`: `origin/main..HEAD`; a new branch with an all-zero `before`: the commits not on `origin/main`) and looks at the paths `git diff-tree --no-commit-id --name-only -r` lists under `tests/golden/adapters/` (`store.lock` lies there, so a new store version counts). It fails in two cases: the commit has such a path and no trailer line `Golden-Bless: <reason>`; or the commit has such a path and its subject starts with `refactor(`, with or without the trailer. So a move can never bless its own change, and a `feat` or `fix` has to say that it meant to change the output. A merge commit is skipped; its parents are checked on their own. Add the step to `ci.yml` for both events (half the recent merges reach `main` as local merges, not pull requests). Test the script with a throwaway repository in `tests/golden-guard.sh`: a refactor that leaves the goldens alone (passes), a refactor that edits one (fails), a fix that edits one with the trailer (passes) and without it (fails).
- [ ] **Step 4:** Run G5 locally. Expected: roundtrip PASS, the second test ignored.
- [ ] **Step 5:** Commit `test(adapters): golden net layer 5 and the guard`.

---

### Stage A as built (2026-11-11)

Stage A is done on the branch `region-adapters` (pull request 38). Where it differs from the tasks above, the code and `tests/golden/adapters/README.md` are right, and later stages read this list:

- **Recorded from v0.12.0**, the tree `main` had when the work started, not v0.11.0.
- **The clock override is per thread** (`clock::with_fixed`), not process-wide with a mutex. The library starts no thread, and a process-wide clock would reach the other unit tests that run at the same time. Code that moves fetch work to another thread must take the clock with it; G3 and G4 show it if it does not.
- **Test names**: `g3_trace::<id>` and `g4_pipeline::<id>` (a module per layer), not `g3_trace_<id>`.
- **G1** keeps the credit lines as (year, records, lines) for the record sets `alkis` and `rasters`. **G2** reads its 211 boxes from `g2-boxes.json` (whole metres) and asks `tile_touches` and `tile_centre_in` only for the zone-32 regions of a box; its timing is the best of five runs (1100 ms on v0.12.0).
- **The index** keeps what the `Fetcher` trait shows (body hash, size, `Last-Modified` date, or the error kind), not HTTP status and content type. `trace.txt` also holds the estimate before the fetch and the result line.
- **Golden areas**: the example centre, moved by 90 to 190 m in Berlin, Schleswig-Holstein, North Rhine-Westphalia and Rhineland-Palatinate so that the area needs one tile (`golden_centre`). Brandenburg and Berlin do not share a square.
- **The store has 206 MB** (version 1), not 120 to 180 MB: the surface model of Schleswig-Holstein is a whole tile of 105 MB. Version 0 was the pilot.
- **G4 hashes every file** of the project folder (34 per state), and all of them are the same bytes on Linux, macOS and Windows. No output needs the decoded-content fallback of A4. Two fixes in production code made it so (`71607e7`): the areas in `freemapper.toml` and the map's reference point are on a millimetre grid.
- **CI**: the net is the job `golden` on three systems; a failing G4 uploads the differing files. The guard is the job `golden-guard`; it does not count the Markdown files of the golden folder.
- `scripts/golden.sh test` skips `g5_unknown_provider_is_a_clear_error` by name until C14 takes the skip out; `g2_dispatch_is_fast_enough` and `record_state` return early in a debug build and without `FREEMAPPER_GOLDEN_RECORD`.
- The session "freemapper" converts the area page to the suite interface (freemapper-core ADR 0030). Tell it before C13 and D7 start.

## Stage B: crates

### Task B1: `freemapper-georef` in freemapper-core: done

Done outside this plan on 2026-11-10: core v0.5.0 has `freemapper-georef` (`crs`, `tiles`, `tm`; core ADR 0015). Nothing to do.

### Task B2: freemapper uses `freemapper-georef`: done

Done: `crates/freemapper/src/crs.rs` and `tiles.rs` re-export `freemapper_georef::crs::*` and `::tiles::*`; the workspace pins every core crate to tag v0.18.0 (v0.19.0 exists and changes nothing freemapper uses; a pin moves for all crates at once, ADR 0067). The api crate of B3 depends on `freemapper-georef` directly and copies no type.

### Task B3: `freemapper-adapter-api`

**Files:**
- Create: `crates/freemapper-adapter-api/{Cargo.toml,src/lib.rs,src/fetch.rs,src/records.rs,src/error.rs,src/note.rs}`, `crates/freemapper-adapter-api/i18n/{en,de}/notes.ftl` (moved from `crates/freemapper/i18n/`)
- Modify: `crates/freemapper/src/fetch.rs` (the trait and error types become `pub use freemapper_adapter_api::fetch::*`), `crates/freemapper/src/project.rs` (record types re-exported), `crates/freemapper/src/note.rs` (becomes `pub use freemapper_adapter_api::note::*`), `crates/freemapper/src/sources/lgb.rs` (`Product`, `TileCurrency`)

**Interfaces:**
- Produces in the api crate: `trait Fetcher`, `enum FetchError` (with its `Cancelled` variant), `enum FetchMode`, `struct ByteRange`, `FetchedTile`, `TileCurrency`, `TileRecord`, `NdsmRecord`, `WmsRecord`, `WcsRecord`, `EstimateItem`, `Part`, `Store` (moved from `space.rs`), `AdapterId(String)` with `FromStr` validating `[a-z][a-z0-9]{1,31}`.
- `Product` moves here too, with its five variants and serde names: `TileRecord` and `FetchedTile` have a field of that type, so it cannot live in a crate that depends on this one. Its methods that name `Provider` (`provider`, `provider_dir_name`) stay in `sources/lgb.rs` as free functions until C7. That an enum of LGB and GeoSN products sits in the general crate is the price of leaving the records alone in the first pass (spec D9); `Part::BerlinLayers` is the same kind of wart.
- The module `note` moves as a whole: `Note`, `Arg`, the macro `note!` (now `#[macro_export]`), `model`, and the catalog `NOTES` with both `notes.ftl`. The readers of B4 and every backend report through it, and `Display` and `PartialEq` of `Note` read the catalog, so the type cannot move without it. All messages of the engine stay in this one catalog; a code adapter adds its messages there in its pull request. The test helpers of `Note` (`contains`, `starts_with`, `ends_with`, comparison with `&str`) become public behind the feature `testing`, which dev-dependencies turn on. The completeness test (`the_catalogs_are_complete_and_use_the_same_variables`) moves along and reads the `src/` folders of every workspace crate that emits notes.
- The map kind in this crate is `freemapper_contract::MapKind` (core, ADR 0095); `freemapper::project::MapKind` converts with `contract()`.
- Dependencies: `freemapper-georef`, `freemapper-text`, `freemapper-contract` (no default features), `chrono`, `serde`, `geo-types`.
- `HttpFetcher`, `CancelCheck`, `Cache`, `DiskSpace` and the clock of A0 stay in `freemapper`. Adapters that need today's date get it through `AdapterEnv` (B5), not by calling the clock. `RasterRecords`, `CacheDir` and `AdapterError` come with B5, where their users are.

- [ ] **Step 1:** Write `adapter_id_accepts_legacy_ids_and_rejects_others` (all 21 legacy ids pass; `""`, `"A"`, `"a"`, `"a b"`, `"../x"`, 33 characters fail).
- [ ] **Step 2:** Create the crate, move the items, re-export from the old paths so no call site changes.
- [ ] **Step 3:** Build and run the whole suite. Expected: PASS, goldens unchanged (G3 compares the English notes). Check that `cargo tree -p freemapper-adapter-api` shows no `ureq`, `toml` or `axum`.
- [ ] **Step 4:** Commit in two commits: `refactor(adapters): freemapper-adapter-api with fetcher and record types`, `refactor(adapters): notes and their catalog in the api crate`.

### Task B4: `freemapper-sources` (the generic readers)

**Files:**
- Create: `crates/freemapper-sources/` with modules `wms`, `wcs`, `wfs` (from `alkis_wfs.rs`), `ogc_api` (from the reader half of `alkis.rs`), `nas` (from `alkis_nas.rs`), `trees`, `zip_range`, `xlsx`, `float_tiff`, `height_model`, `raster`, `metalink` (extracted from `lvermgeo_rp.rs`)
- Modify: `crates/freemapper/src/sources/*.rs` (re-exports at the old paths), `Cargo.toml`

**Interfaces:** `WmsService`, `WcsService`, `WcsOrthophoto`, `WfsService`, `NasService`, `TreeDataset` change their `&'static str` fields to `Cow<'static, str>` so manifests can build them at run time; the constants stay `const` (`Cow::Borrowed`). `NasService.region` and `TreeDataset.region` become `AdapterId` (a `&'static str`-backed constant for the built-ins).

What the readers take from the main crate today, and where it goes: `crs` and `tiles` are re-exports of `freemapper-georef`; `fetch` and `note` are in the api crate since B3; `raster` moves here. Three things need a step of their own:
- `height_model` calls `paste_tile`, `read_cached` and `whole_tile` of the state module `lvermgeo_sh` (and `geobasis_nrw` calls them too). They move into `height_model` first.
- `alkis_nas` takes `MapKind` from `project`. It takes `freemapper_contract::MapKind` instead; its callers pass `kind.contract()`.
- `alkis`, `alkis_nas` and `trees` name `Region` and `regions_for`. Only their reader halves move (requests, paging, parsing); the halves that pick services by region stay in `freemapper` until C6 and C13.

- [ ] **Step 1:** Move the three tile helpers into `height_model` (own commit, `refactor(adapters): tile helpers of the height models in one module`). Then move one reader at a time in the order `raster`, `zip_range`, `xlsx`, `float_tiff`, `height_model`, `wms` and `wcs` together (each uses the other), `wfs`, `ogc_api`, `nas`, `trees`, `metalink`; after each, `cargo test --workspace --locked`.
- [ ] **Step 2:** Switch the service structs to `Cow<'static, str>`. Write `service_constants_are_unchanged` (URL strings produced by `piece_url`, `get_coverage_url` for 3 sample boxes equal the strings of `main` before the move, which the test holds as literals).
- [ ] **Step 3:** Check `cargo build -p freemapper-sources` alone compiles without `freemapper`, and measure the clean build time of the workspace before and after (record both numbers in the commit message).
- [ ] **Step 4:** Full suite and goldens. Expected: PASS, no golden change.
- [ ] **Step 5:** Commit in two commits: `refactor(adapters): move the readers to freemapper-sources`, `refactor(adapters): owned service fields`.

### Task B5: Adapter-shaped parameters for the raster fetch, in place

The ten `fetch_*_rasters` functions of `project.rs` cannot move into a crate that does not depend on `freemapper` as they are. Each takes the project folder and `&mut ProjectFile`, saves the project file at fixed points (15 `project.save(dir)` calls in the ten), gets `skip_surface` from the disk-space plan, reports through `progress`, returns `ProjectError`, and calls shared helpers of `project.rs`: `fetch_wms_orthophoto` (10 calls), `warn_beyond_state` (9), `fetch_area_heights` (7), `drop_wms_orthophoto`, `keep_stored_surface`, `say_kept_surface_tiles`, `warn_missing_tiles`, `raster_tiles`, `tile_record`. This task changes the parameters and nothing else, with the functions still in `project.rs` and the dispatch still a `match` on `Provider`. After it C7 to C12 are moves.

**Files:**
- Create: `crates/freemapper-adapter-api/src/{env.rs,project.rs}`, `crates/freemapper-sources/src/raster_fetch.rs`, `tests/golden/adapters/g3-interrupted.json`
- Modify: `crates/freemapper/src/project.rs` (the ten functions, the helpers above, the dispatch in `fetch_project_parts`), `crates/freemapper/src/fetch.rs` (`Cache`, `fetch_tiles`, `fetch_tiles_each`), `crates/freemapper-adapter-api/src/error.rs`

**Interfaces:**
- `AdapterEnv<'a> { fetcher: &'a dyn Fetcher, cache: &'a CacheDir, mode: FetchMode, today: NaiveDate, skip_surface: bool, progress: &'a mut dyn FnMut(Note) }`. `today` comes from `clock::today()` at the one place that builds the env; `skip_surface` from the disk-space plan as today.
- `CacheDir`: the cache root with exactly the methods the ten functions, `fetch_tiles`, `fetch_tiles_each` and the state modules call on `Cache` today (list them with `rg "cache\.[a-z_]+\(" crates/freemapper/src` first: the folder getters, `tile_path`, `ndsm_path`, `has_tile`, `currency_csv_path` and what the tile download asks about free space). The folder getters become `provider_dir(name: &str)`, called with the names G1 records (`lvermgeo_sh`, not `lvermgeosh`). `Cache` in `freemapper` hands out a `CacheDir`.
- `trait RasterProject`: `records(&mut self) -> RasterRecords<'_>` (mutable borrows of `tiles`, `ndsm_tiles`, `orthophoto`, `terrain`, `surface`; a view, not a new struct inside `ProjectFile`, because a flattened struct would change the key order of `freemapper.toml`), `save(&mut self) -> Result<(), AdapterError>` (the checkpoint; the 15 calls stay where they are), `dir(&self) -> &Path`, `region_bbox(&self)`, `crs(&self) -> Crs`. `ProjectFile` with its folder implements it in `project.rs`. `dir()` exists because the state modules write through `write_inside(dir, …)` today. The `ProjectSink` of spec 3.3, which hides the folder, stays the target for adapters that are not built in and is not part of this plan's moves; a manifest adapter never sees the folder, because it reaches the project only through the readers.
- `AdapterError` gets one variant for every `ProjectError` variant the ten functions can return (`OrthophotoNotFetched`, `TerrainNotFetched`, the fetch, file and save errors), and `impl From<AdapterError> for ProjectError` maps each back to the variant it came from. The CLI words errors by variant (`errors.rs`, German and English), so the mapping must lose nothing.
- The shared helpers take `&mut AdapterEnv` and `&mut dyn RasterProject` and move to `freemapper_sources::raster_fetch`. `warn_beyond_state` takes the provider id as `&str` until C2 gives it the registry.
- Each of the ten functions becomes `fn fetch_<provider>_rasters(env: &mut AdapterEnv<'_>, project: &mut dyn RasterProject, bbox: &BoundingBox) -> Result<(), AdapterError>`.

- [ ] **Step 1:** Before any change, record `g3-interrupted.json`: for Saxony-Anhalt, Brandenburg and Hesse, replay the fetch with a fetcher that fails at the request after the orthophoto, at the first surface request and at the last request, and store the trace, the notes and the bytes of `freemapper.toml` left behind (nine cases). Commit it alone as `test(adapters): golden for interrupted fetches` with the trailer `Golden-Bless: new layer`.
- [ ] **Step 2:** Write `skip_surface_reaches_every_backend` (for each of the ten providers: a disk-space plan without room for the surface gives a trace without a surface request and the warning of today) and `adapter_errors_map_back_to_the_same_project_error` (find the variants by changing one function's return type and reading the compiler; for each, the round trip gives the same variant and the same English and German text).
- [ ] **Step 3:** Add the types. Change the helpers, then the ten functions, one commit per provider (`refactor(adapters): <provider> raster fetch takes the adapter parameters`). Bodies change only where a parameter is named.
- [ ] **Step 4:** Move the helpers to `raster_fetch`. `scripts/golden.sh test` and the whole suite after every commit. Expected: PASS, no golden change.

---

## Stage C: the trait and the registry

Recipe for every C task: (1) write the test that pins the concern through the trait; (2) add the trait method or data; (3) implement it for the built-in states by moving the existing code (no edits to bodies beyond signatures); (4) delete the old `match`; (5) goldens and the full suite stay green; (6) commit `refactor(adapters): ...`. If a golden file would have to change, stop: the move is not behaviour-preserving.

Unit tests are part of every move. About 290 of the 481 `Region::` and `Provider::` references are in test modules (`region.rs` alone has 152). Each C task rewrites the tests of the code it moves to adapter ids (a test helper `adapter("hamburg")` over the registry) and moves them with the code, so C14 finds only leftovers. The assertions stay as they are.

### Task C1: `RegionAdapter`, `Registry` and the legacy shim

**Files:**
- Create: `crates/freemapper-adapter-api/src/{adapter.rs,registry.rs,info.rs}`, `crates/freemapper/src/adapters/legacy.rs`
- Modify: `crates/freemapper/src/lib.rs`

**Interfaces:**
- Produces: `trait RegionAdapter` (section 3.3, with `rasters`, `alkis`, `extras`, `attribution`, `estimate` returning `None`, empty or default in this task; `AdapterEnv` exists since B5), `struct RegionInfo { id: AdapterId, provider: AdapterId, names: Names { en, de }, crs: Crs, object_id_prefix: Option<String>, osm: Vec<OsmExtractInfo>, credit_short: Option<String> }`, `struct Registry`, `fn registry() -> &'static Registry` (process-global, `OnceLock`, set once by `install_registry(Registry)`; default `Registry::builtin()`), `struct LegacyAdapter(Region)` implementing the trait from the enums.
- Where the built-ins come from: `Registry` is a type of the api crate and knows no adapter. `Registry::builtin()` is a function of `freemapper` (`freemapper::adapters::builtin()`), so the library and its tests work without the CLI. From C7 on `freemapper` depends on `crates/adapters/de`, which depends on the api crate and `freemapper-sources` only. The CLI starts from the built-ins, adds the crates its features enable and the folder adapters (D7), and calls `install_registry` once.

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

- [ ] **Step 1:** Write `estimate_items_equal_the_old_ones_for_the_example_areas` (item text, MB, cached flag, part, store for the 11 example boxes under the fixed clock, golden file `g2-estimates.json` blessed before the move). Commit the test and the file on their own as `test(adapters): estimate golden` with the trailer `Golden-Bless: new layer`, so the guard, which rejects any golden path in a `refactor(` commit, stays strict.
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

**Interfaces:** `RasterBackend::fetch(&self, env: &mut AdapterEnv<'_>, project: &mut dyn RasterProject, bbox: &BoundingBox) -> Result<(), AdapterError>`, the signature B5 gave the ten functions; `ProviderInfo { id, cache_dir: Option<&'static str>, terrain_source, kind: Tiles { grid_km } | PerArea }` with the folder names of G1 (a new adapter's folder is its provider id); `DeriveInputs` as in section 3.3. `Product` is in the api crate since B3; its free functions that named `Provider` (B3) become data of the two backends here.

- [ ] **Step 1:** Write `lgb_offline_fetch_reads_cached_tiles_only`, `geosn_offline_with_missing_index_is_the_old_error` (the offline special case for the GeoSN index in `fetch.rs` keeps its message), `area_without_a_raster_region_fetches_from_lgb_as_today` (the dispatch sends `None` to the LGB function; the registry keeps that) and `cancel_during_backend_fetch_ends_as_cancel` (Review Focus 6: a fetcher that returns `FetchError::Cancelled` on the second request; `fetch_project_cancellable` returns the same cancel error as today, and no "download failed" warning appears).
- [ ] **Step 2:** Move the two fetch functions into `RasterBackend::fetch` and the derive match arms into `derive_inputs`; the project code calls `adapter.rasters()` and no longer knows tile or per-area kinds by enum. `AdapterError::Fetch(FetchError::Cancelled)` maps back to the project's cancel error in one place; every later backend task (C8 to C12) reuses that mapping and adds no own.
- [ ] **Step 3:** G1 to G4 for BB, BE and SN, and `g3-interrupted.json`. Expected: PASS.
- [ ] **Step 4:** Commit. This is the largest C task (about 1 200 lines moved, their tests included); split it into two commits, LGB then GeoSN.

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

**Files:** Create `crates/adapters/de/src/berlin.rs` (adapter with code `ExtraLayer`s); modify `project.rs` (Berlin layers, NAS layers, tree layers records and paths; the "outside the supported states" error and the beyond-coverage warning, which name the states), `sources/alkis_nas.rs`, `sources/trees.rs` (service tables read from adapters), `crates/freemapper-cli/src/select.rs` (new `GET /api/adapters`; the `img-src` list of the content security policy at line 44 is built from the preview hosts), `crates/freemapper-cli/web/app.js` (the ten aerial layers at lines 206 to 290 and the state list in `TEXTS.de` and `TEXTS.en` come from the response; `run.js` names no state today), `crates/freemapper-cli/i18n/{en,de}/freemapper.ftl` (`error-project-outside-coverage` and `mcp-instructions` take the list as a value).

The states are listed in five places today: `project.rs:150` (English, the error's `Display`), `error-project-outside-coverage` and `mcp-instructions` in both catalogs of the CLI, and the two tables of `app.js`. All five get the list from `names` of the registry in the active language.

Coordination: check who has open work on these CLI files (Global Constraints) and agree the change first. The tests of `select_page.rs` hold the page to its rules (same keys in both languages, every `data-text` key has a text, every id a script uses exists); they must pass unchanged.

**Interfaces:** `trait ExtraLayer { fn id(&self) -> &str; fn kinds(&self) -> &[freemapper_contract::MapKind]; fn fetch(...); }`; `NAS_SERVICES` and `TREE_DATASETS` become registry queries `nas_services_for(bbox)` and `tree_datasets_for(bbox)` with the same results; `/api/adapters` returns `[{id, names: {en, de}, preview: {wms, layer, format, params, attribution}}]` and the page picks the name in its own language (ADR 0092). The preview is not the fetch service: it has its own URL and layer for most states (Brandenburg's map proxy, `he_dop_rgb` in Hesse, the ADV mosaic in Mecklenburg-Vorpommern), a `format` of `image/png` or `image/jpeg`, and Hamburg adds `time = "2026"`, so `params` is a table. The English sentence in `project.rs` joins `names.en` in registry order, so its bytes stay as today.

- [ ] **Step 1:** Write `berlin_layers_paths_still_come_from_the_static_table` (ADR 0059: a key outside the table is dropped with the old warning) and `adapters_endpoint_lists_eleven_with_previews` (in `crates/freemapper-cli/tests/select.rs`).
- [ ] **Step 2:** Move the extras; keep `data/berlin`, `data/alkis-nas` and `data/trees` names.
- [ ] **Step 3:** Replace the hard-coded lists in the page; check it in the browser against the 11 states (manual check, note in the commit) and with the content security policy test.
- [ ] **Step 4:** G1 to G4 for BB and BE, `select` tests. Expected: PASS.
- [ ] **Step 5:** Commit in two commits (extras, then page and texts).

### Task C14: Persisted ids and deleting the enums

**Files:** Modify `project.rs` (`ProjectFile.provider: Option<AdapterId>`, record `regions: Vec<AdapterId>`), `region.rs` (delete `Region`, `Provider`, `ALL`, the boundary constants), `adapters/legacy.rs` (delete), all remaining `Region::`/`Provider::` sites.

**Interfaces:** `AdapterId` serialises as a plain lower-case string (the same bytes the enums produced).

- [ ] **Step 1:** Remove the `#[ignore]` of `g5_unknown_provider_is_a_clear_error` (message: `the project names the adapter "czfoo", which is not installed; adapters are searched in: …`).
- [ ] **Step 2:** Add the guard test `no_enum_matches_left`: a test that reads `crates/**/*.rs` outside `tests/` and fails on `Region::` or `Provider::` (the legacy id table lives in the DE adapter crate as string constants). It reads the test modules too, so list first what the earlier tasks left (`rg -c "Region::|Provider::" crates`); the recipe of this stage should have brought the 481 of today near zero.
- [ ] **Step 3:** Delete the enums and the shim; fix the last call sites and the last tests.
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
- Produces: `struct UrlPolicy { hosts: BTreeSet<String>, max_redirects: u8 }`, `fn check_url(&self, url: &str) -> Result<(), PolicyError>`, `fn check_addr(addr: IpAddr) -> Result<(), PolicyError>`, `trait Resolver` (`std::net::ToSocketAddrs` by default, a stub in tests); `HttpFetcher::with_policy(UrlPolicy)`. The built-in adapters get a policy built from their declared hosts; before D6 the policy for built-ins is "address check only". Requests that belong to no adapter use `UrlPolicy::core()`, a fixed list in code: `download.geofabrik.de` (OSM extracts and state files), `nominatim.openstreetmap.org` (place search in `places.rs`) and `tile.openstreetmap.org` (the tile proxy of ADR 0088 in `crates/freemapper-cli/src/tiles.rs`, which has its own `ureq::Agent`; that agent gets the same address check through `check_addr`).
- The self-update (ADR 0094) has a third HTTP path, `crates/freemapper-cli/src/update/source.rs`, with its own host list and its own redirect check (at most 3, allowed hosts only). It becomes `UrlPolicy::update()`, and its redirect check and the one of this task become one function of the api crate. Agree the edit with the self-update session; its tests of the redirect rules must pass unchanged. That session agreed on 2026-11-11 under three conditions, so the update path stays as strict as it is: the host list is `freemapper_update::ALLOWED_HOSTS` of core (`github.com`, `release-assets.githubusercontent.com`, `objects.githubusercontent.com`), taken from there and not copied; `https` only, at most 3 redirects, every hop through core's `check_url`, and the update agent keeps its size caps and `Accept-Encoding: identity`; error texts never show the query or the fragment of a URL (they are the signed parts).
- Tests reach servers on `127.0.0.1` over `http` today (five tests of `HttpFetcher` in `fetch.rs`, the tests of the update source). The address check has one way around it, a constructor behind `#[cfg(test)]` (and the `testing` feature for other crates' tests), never an environment variable or a command-line switch: a switch a user can set is a switch an attacker can set.
- `ureq` offers the resolver hook under `ureq::unversioned::resolver` and `Agent::with_parts`, outside its semver promise. The workspace pins `ureq` exactly (`=3.4.2`); a version bump runs this task's tests first.

- [ ] **Step 1:** Write `core_policy_allows_geofabrik_nominatim_and_osm_tiles_only`, `update_policy_allows_the_two_github_hosts_only`, `declared_host_resolving_to_loopback_is_refused`, `redirect_to_undeclared_host_is_refused`, `redirect_to_ip_literal_is_refused`, `more_than_three_redirects_is_refused`, `answer_over_the_cap_is_too_large`, `private_ranges_are_refused` (10/8, 172.16/12, 192.168/16, 169.254/16, fc00::/7, fe80::/10, ::1, 127/8, 0.0.0.0, 224/4), with a local stub server on 127.0.0.1 and a stub resolver (Review Focus 3).
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

**Files:** Create `crates/freemapper-manifest/src/loader.rs`; modify `crates/freemapper-cli/src/main.rs` (`adapters list`, `--adapters-from-project`), `crates/freemapper-cli/src/adapters.rs` (registration list), `crates/freemapper-cli/src/mcp.rs` (never loads project adapters; agree the edit with whoever has open work on the file), `crates/freemapper-cli/src/select_run.rs` (a run started from the page never loads project adapters either, like MCP), `crates/freemapper-cli/src/help.rs` and `crates/freemapper-cli/i18n/{en,de}/freemapper.ftl` (help and messages of the new command and switch in both languages), `AGENTS.md` (command table), `docs/manual/{de,en}.md`.

**Interfaces:** `Registry::with_dirs(dirs: &[AdapterDir]) -> (Registry, Vec<AdapterWarning>)`, `AdapterDir { path, origin: Builtin | User | Project }`; `freemapper adapters list [--json]`. Command, switch and JSON field names are the same in both languages; the texts come from the catalog (ADR 0092).

- [ ] **Step 1:** Write `folder_adapter_cannot_replace_a_builtin`, `broken_folder_adapter_is_skipped_with_file_and_line`, `shared_provider_shares_the_cache_folder` (Review Focus 4), `project_adapters_need_the_flag`, `mcp_ignores_project_adapters`, `page_run_ignores_project_adapters`, `adapter_dir_env_overrides_the_config_dir`, `config_dir_with_spaces_and_unicode_works`.
- [ ] **Step 2:** Implement; the config folder is `dirs::config_dir()/freemapper/adapters`.
- [ ] **Step 3:** Run, then try `freemapper adapters list` with a sample user folder on all three operating systems in CI.
- [ ] **Step 4:** Commit.

### Task D8: `tile-pattern` raster kind (deferred)

Deferred on 2026-11-11: build it only when a new state or a contributor needs it. The text stays as the design.

**Files:** Modify `crates/freemapper-manifest/src/rasters.rs`, `crates/freemapper-sources/src/tile_pattern.rs`.

**Interfaces:** `[terrain] kind = "tile-pattern"`, keys `url` with `{zone}`, `{e}`, `{n}` placeholders, `grid_km`, `zone`, `format = "geotiff"`, `nodata`; fetched into the shared cache as a tile provider.

- [ ] **Step 1:** Write `pattern_expands_to_the_tile_urls_of_a_box` and `placeholder_values_are_percent_encoded`.
- [ ] **Step 2:** Implement over the existing tile cache and mosaic code.
- [ ] **Step 3:** Run the suite. Expected: PASS; no existing state uses the kind, so goldens are unchanged.
- [ ] **Step 4:** Commit.

---

## Stage E: tooling

### Task E1: Fixtures format and the replay fetcher in the library

**Files:** Create `crates/freemapper-sources/src/replay.rs` (promoted from the test support of A3), `docs/adapters/fixtures.md` (outline only, filled in F1).

**Interfaces:** `ReplayFetcher::from_dir(dir)`; format: the index of A3, `fixtures/index.toml` with `[[response]] url, range, status, content_type, body`; the bodies under `fixtures/` only, names `[0-9a-f]{64}.bin`. The 2 MB cap of an adapter folder fits per-area services (WMS, WCS, WFS). An adapter with a tile backend cannot meet it, as stage A showed; it pins a store archive of its own like `store.lock`. The E5 state settles the details if it has tiles.

- [ ] **Step 1:** Write `replay_rejects_index_entries_that_name_other_paths` and `replay_reports_unrecorded_requests_with_the_url`.
- [ ] **Step 2:** Move the code; A3's tests import it.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E2: `freemapper adapter check`

**Files:** Create `crates/freemapper-cli/src/adapter_check.rs`; modify `main.rs`, `help.rs`, `crates/freemapper-cli/i18n/{en,de}/freemapper.ftl` (help, findings and the per-capability lines in both languages; the finding codes stay the same), `AGENTS.md` (command table), `docs/manual/{de,en}.md`.

**Interfaces:** `freemapper adapter check <dir> [--fixtures] [--live --bbox minLon,minLat,maxLon,maxLat] [--json]`; exit code 0 pass, 1 findings, 2 cannot run. Static checks: schema (D1), policy, boundary validity and CRS, boundary area plausible for the object-id prefix, credit present and licence known, `last_verified` not older than 12 months (warning), every `backend` resolvable, `rules.toml` valid. `--fixtures`: replay a fetch, derive and build of the golden area from `<dir>/fixtures/` and compare with `<dir>/expected.sha256` when present. `--live`: the same against the network on the given box, which must lie inside the boundary, printing one line per capability (`orthophoto ok 0.2 m, 1 piece`, `terrain FAILED: HTTP 404 …`).

- [ ] **Step 1:** Write `check_passes_the_hamburg_folder`, `check_fails_a_boundary_in_the_wrong_crs_with_code_boundary_crs`, `check_fails_a_symlinked_folder`, `check_json_output_lists_codes`, `live_box_outside_the_boundary_is_refused`.
- [ ] **Step 2:** Implement over D1, D3 to D5 and the replay fetcher.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E3: `freemapper adapter record` (deferred)

Deferred on 2026-11-11 until someone outside the project wants to add a state. Until then the recording fetcher of A3 (promoted in E1) records fixtures from a test.

**Files:** Create `crates/freemapper-cli/src/adapter_record.rs`.

**Interfaces:** `freemapper adapter record <dir> --bbox ...` runs a live fetch, derive and build on a 150 m box, writes `<dir>/fixtures/`, `<dir>/expected.sha256` and prints the sizes; refuses over 2 MB; sends no contact data (neutral user agent, as today).

- [ ] **Step 1:** Write `record_refuses_a_fixture_set_over_2_mb` and `recorded_fixtures_replay_to_the_same_hashes` (against the local stub server).
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E4: `adapter new` and `adapter boundary` (deferred)

Deferred on 2026-11-11 like E3. The new state of stage E5 writes its manifest and boundary by hand and notes what a generator would have saved; that note decides the scope of E4.

**Files:** Create `crates/freemapper-cli/src/adapter_new.rs`, `adapter_boundary.rs`.

**Interfaces:** `freemapper adapter new <id> [--kind manifest|backend|crate]` writes a commented manifest or a crate skeleton (`Cargo.toml`, `src/lib.rs` with the `RasterBackend` stub, a test, the line to add to `adapters.rs`); `freemapper adapter boundary <geojson> --out <file> [--crs 4326|25833]` simplifies to 25 m, reprojects to 25833 and validates.

- [ ] **Step 1:** Write `new_manifest_passes_check_after_filling_the_placeholders` (placeholders make `check` fail with `manifest.placeholder`), `new_crate_compiles` (in a temp workspace member), `boundary_output_is_valid_and_within_25_m_of_the_input`.
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit.

### Task E5: First new state as the test of the design

Not a task of this plan but a gate: before F1, one state of stage G (Saarland or Bremen, whichever has the simpler services after the research) gets its own plan and is added as a manifest folder, plus a backend only if its heights need one. Measure against the stage-G rule: if it needs a file outside `crates/adapters/` and the registration line, write the finding back into the spec and fix the design before the guide describes it. The state also brings the house-rule parts (example project, manual row, README row, changelog in German and English).

---

## Stage F: documentation

### Task F1: Contributor guide and manifest reference (bilingual)

**Files:** Create `docs/adapters/{en.md,de.md,manifest-v1.en.md,manifest-v1.de.md}`, `docs/adapters/tutorial.md` pair if length requires.

- [ ] **Step 1:** Write the sections of spec 3.17 in English, then German ("Bundesland" for states, glossary terms), with one runnable tutorial that adds a made-up state from the sample manifests and ends with `adapter check` green. While E3 and E4 are deferred, sections 3, 4 and 8 describe the manual steps the E5 state took instead of `adapter new`, `adapter boundary` and `adapter record`.
- [ ] **Step 2:** Add the pull-request checklist: manifest valid, `check` green with fixtures, credit text matches the licence page, `maintainers` and `last_verified` set, no personal data in fixtures, example project and manual row for a new state (house rule).
- [ ] **Step 3:** Run unslop and humanizer over the prose. Check every command in the guide by running it.
- [ ] **Step 4:** Commit `docs(adapters): contributor guide`.

### Task F2: Manual, README, weekly live workflow, release notes

**Files:** Modify `docs/manual/{de,en}.md` (regions chapter links to the guide), `README.md`, `README.en.md` (one sentence, link; re-check status per the pre-push rule), `CHANGELOG.md`, `CHANGELOG.en.md` (entry under "Unveröffentlicht"/"Unreleased" for each merged stage, written when it merges), `docs/adr/README.md` (status of ADR 0082 to Accepted).

Deferred on 2026-11-11: `.github/workflows/adapters-live.yml` (weekly `adapter check --live` for every built-in adapter, an issue on failure). It pays off once outside adapters exist; until then the maintainer's own runs notice broken services.

- [ ] **Step 1:** Update the files; run the README re-check.
- [ ] **Step 2:** Shrink the net. G3 and G4 exist to prove the migration, and every later change to a request pays for them with a new recording. Keep G1, G2 and G5 (they cost nothing) and G3 and G4 for three states, one of each kind: Brandenburg (tiles in the cache), Saxony-Anhalt (per-area services, the fully declarative manifest) and Berlin (the code adapter). Remove the other eight from the tests, publish the smaller store as the next version, and say so in `tests/golden/adapters/README.md`. Commit with the trailer `Golden-Bless: net shrunk after the migration`.
- [ ] **Step 3:** Commit; after CI is green, release at a sensible point (auto-release rule).

---

## Stage G: new states (separate plans)

Not planned here. Each is a plan of its own because the service research comes first: Baden-Württemberg, Bremen (a hole in the Lower Saxony boundary today; its adapter removes the `reaches_beyond_coverage` special case), Bayern, Saarland. Each follows the house rules for a new state: manifest folder, example project, manual row, README row, changelog entry in German and English. They are the first test of the guide: if a stage-G task needs a file outside `crates/adapters/` or the registration line, the design failed and a finding goes back to the spec.

## Self-review

- Spec coverage: 3.2 crates (B1 to B4, C1), 3.3 trait (C1 to C13), 3.4 schema (D1, D3 to D5, D8), 3.5 samples (D1, D4, D5, D6), 3.6 limits (D5 backend errors, F1), 3.7 registry (C1, D7), 3.8 security (D2, D1, D7, C13 for the page), 3.9 normalisers (C6), 3.10 versions (D1, D7), 3.11 errors (D1, C14), 3.12 i18n (C13 names, D1), 3.13 ids (B3, C14, A5), 3.14 testing (A1 to A5, E1 to E3), 3.15 net and migration (A, C), 3.16 performance (A2, C2), 3.17 docs (F1, F2), 3.18 core (B1, B2). Gaps: none found; the `rules.toml` per adapter (3.8, D11) has a check in E2 but no loader task. Added to D3: load `rules.toml` through the existing rule validator and rank it below OSM.
- Type consistency: `AdapterId` (B3) is used in C1 to C14 and D; `RasterRecords` (B3) in C7; `Registry::backend` (D5) matches `RasterBackend` (C7); `UsedSources` (C4) and `EstimateArea` (C5) are defined where first used.
- Proportion: the plan is longer than the spec's staged table suggests because every task carries its tests; bodies are left to the implementer.
- Revision 2026-11-11, checked against `main` at v0.10.0: the counts of spec section 2.1 (about 190 match places, about 440 enum references) still hold, 440 references in 16 files today; file and line references were updated or replaced by names; the clock (A0), cancel (C7), program language (A3, C1, C13), the ADR 0091 entry points (A4), the push guard (A5) and the hosts outside adapters (D2) were gaps of the first version.
- Revision 3, checked against `main` at v0.11.0 by reading the code and measuring the cache: 481 enum references in 16 files, about 290 of them in tests; `project.rs` has 14 687 lines. Gaps of the second version: the fixture size (A3, A4), the clock calls that name no clock (A0), bytes across systems (A4 pilot), the parameters of the raster fetch (B5), the notes (B3), `Product` and the map kind (B3), the cache folder names (A1, B5, C7), the unit tests (stage C recipe), the third HTTP path (D2), the texts of the new commands (D7, E2) and the lifetime of the net (F2).
- Type consistency after revision 3: `AdapterEnv`, `RasterProject`, `RasterRecords`, `CacheDir` and `AdapterError` are defined in B5 and used from C1 on; `RasterBackend::fetch` (C7) has the signature of B5; `Product` and `Note` are in the api crate from B3 on; `ProviderInfo.cache_dir` (C7) holds the names of G1 (A1).

## Size and order

| Stage | Tasks | Estimated size | Can run in parallel |
|---|---:|---|---|
| A | 6 (A0 to A5) | 250 lines production (clock), 1 500 lines of tests and scripts, under 1 MB in the repository, a store of 120 to 180 MB outside it | A0 first; A1, A2 together; A3 and A4 for two states (pilot), then the other nine |
| B | 3 (B3 to B5); B1, B2 done | 6 500 lines moved, 600 new; B5 touches about 1 100 lines of `project.rs` in place | B3, B4, B5 in this order |
| C | 14 | 4 000 lines moved with their tests, 1 500 new | C3 to C6 independent after C2; C8 to C12 independent after C7 |
| D | 7 (D1 to D7); D8 deferred | 2 300 lines new, 11 manifests | D1 and D2 together; D6 after D3 to D5 |
| E | 2 (E1, E2) and the E5 gate; E3, E4 deferred | 800 lines new | E1 then E2 |
| F | 2 | docs, the smaller net | after E5 |
| Total | 34 active, 3 deferred, 2 done | | |

Recommended execution: subagent-driven. The tasks depend on each other's interfaces (B3's types, B5's parameters, C1's trait), there are 34 of them, and a mistake in a move task changes example output silently, so an independent review per task earns its cost. Stage A is the one place where a human should look at the recorded fixtures before the rest starts, and where the user's go covers a release (the fixture store). Stage A runs on its own branch and merges to `main` before stage B starts, so the goldens guard the other sessions' work too. B5 to C14 run in one stretch (Global Constraints).
