# Sweden adapter, first stage (terrain and forest) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A person with a Geotorget account can make a freemapper map in Sweden from the Lantmäteriet terrain model, laser data and land cover plus OpenStreetMap.

**Architecture:** A new built-in crate `crates/adapters/se` holds `manifests/sweden/manifest.toml` (every address and host, ADR 0189) and the code no manifest can say: a STAC reader, a per-area terrain backend (`tiff_range` blocks of the 10 km COG), a COPC point-cloud backend, a Marktäcke GeoPackage source and the class table. It is wired like the German states (`ManifestAdapter` plus backends in the registry). Three readers already exist on this branch: Deflate in `tiff_range`, `HttpFetcher::with_basic_auth`, `freemapper_sources::gpkg`.

**Tech Stack:** Rust workspace, `freemapper-manifest` (TOML manifests), `freemapper-sources` (`tiff_range`, `gpkg`, `las` for COPC), ureq-based `HttpFetcher`.

**Spec:** `docs/superpowers/specs/2026-11-27-sweden-adapter-design.md` (approved by the owner, 2026-11-27, with the manifest rule added). Research and measurements: `docs/regions/sweden-reference.md` on branch `sweden-reference` (worktree `.worktrees/sweden`; read §3 and §8).

## Global Constraints

- **No address in Rust production code** (issue #273, ADR 0189): every URL and host of the adapter stands in its `manifest.toml`. Tests may hold `https://` literals. `grep -n 'https\?://' crates/adapters/se/src` shows only tests and comments at the end of every task.
- **No Lantmäteriet data in the repository, in tests, in examples or in the docs.** Fixtures are synthetic (built in test code or by `make_fixtures.py`). The owner's application states it.
- **Credentials only from `FREEMAPPER_LM_USER` and `FREEMAPPER_LM_PASSWORD`**, never in `freemapper.toml`, a cache file name, a progress line, an error text, `ATTRIBUTION.txt` or a log. The password type has no `Debug` and no `Display`. The header goes to the one host the manifest names, never along a redirect to another host.
- Hosts the adapter asks: `api.lantmateriet.se`, `dl1.lantmateriet.se`, the OSM extract host. Fixed User-Agent only; no e-mail or personal data in any request (AGENTS.md).
- The map CRS is EPSG:25833; SWEREF99 TM (EPSG:3006) has the same coordinates, so the adapter states `crs = 25833` and nothing is transformed.
- Download limits are unpublished (a cap per period, HTTP 429, a cap on concurrent downloads): fetch tile by tile, never in parallel, reuse the existing retry policy for 429, and say in the error that a cap may be the reason.
- Credit line: `© Lantmäteriet (CC BY 4.0)`. No Byggnad, no orthophoto, no Ytmodell. Buildings from OSM only. No Swedish UI or manual translation.
- Process: TDD (no production code before a failing test, watched failing), `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, commit per task, German commit subjects like `git log` shows, trailer lines from the session's attribution reminder. Source and technical docs English; user docs German and English; Terms follow `docs/glossary.md`. The long `cargo test --workspace --locked` runs in the background.
- Rebase on `github/main` before every task: another session moves addresses into manifests (#273) and touches `crates/freemapper-manifest`, `docs/adapters/manifest-v1.*.md` and the ADR numbers. Take the next free ADR number when you commit, not before.

## Review Focus

- The password in an error, a `Debug` print or a progress line (a failed login, a 401 body, a bad URL): never visible. Task 3 pins it.
- A folder adapter that names `[auth]` with a host it controls would leak the Geotorget password: `[auth]` is refused outside built-in adapters. Task 1 pins it.
- Credentials missing: the error names both variables and the three products to order, and no request is sent. Task 3.
- HTTP 401 or 403 from `dl1` (account without the order) and 429 (cap): each has its own sentence, none looks like "no data here". Tasks 5 to 7.
- A map area that reaches over a DTM tile border, outside Sweden, or on the sea: no crash, tiles missing from the STAC answer are left out with a progress line. Task 5.
- A Marktäcke `objekttyp` the table does not know: left out and named once, the others stay. Task 7.
- A municipality GeoPackage that is large (hundreds of MB): read from the file on disk with the size limit of the downloader, not from a second copy in memory. Task 7.

## File Structure

- `crates/freemapper-manifest/src/{model,validate,rasters,adapter}.rs`, `docs/adapters/manifest-v1.{de,en}.md`: terrain without orthophoto, `lantmateriet` endpoints, `[auth]`, `[point_cloud] kind = "stac-copc"`, `[alkis] kind = "stac-gpkg"`.
- `crates/freemapper-adapter-api/src/{rasters,adapter,info}.rs`: `OrthophotoInput::Absent`, `AuthSpec`, `RegionAdapter::auth`.
- `crates/freemapper/src/{project,fetch,adapters/mod}.rs`: derive without orthophoto, credentials from the environment, registration.
- `crates/freemapper-sources/src/{stac,raster_fetch}.rs`: STAC reader; terrain-only fetch helper.
- `crates/adapters/se/` (new): `manifests/sweden/{manifest.toml,boundary.geojson}`, `src/{lib,sweden,stac_terrain,copc,marktacke,classes}.rs`, `rules/marktacke.toml`.
- Docs: three ADRs, `docs/manual/{de,en}.md`, README rows, `docs/glossary.md`, `CHANGELOG.md`, `CHANGELOG.en.md`.

---

### Task 1: Manifest tables for Sweden

**Files:**
- Modify: `crates/freemapper-manifest/src/{model.rs,validate.rs,rasters.rs,adapter.rs,schema.rs}`, `docs/adapters/manifest-v1.de.md`, `docs/adapters/manifest-v1.en.md`
- Test: the existing test modules of `validate.rs` (follow their helper for a manifest text)

**Interfaces:**
- Produces (model): `Manifest.auth: Option<Auth>` with `pub struct Auth { pub host: String, pub user_env: String, pub password_env: String, pub products: Vec<String> }`; `PointCloudKind::StacCopc` (value `stac-copc`); `Alkis::StacGpkg { stac: String, collection: String, rules: String }`; `Manifest` accepts `[terrain]` with no `[orthophoto]`; `[terrain] backend = "lantmateriet"` takes `endpoints` with the names `stac-hojd`, `stac-vektor`.
- Produces (findings): `manifest.auth-builtin-only` (a folder adapter with `[auth]`), `manifest.auth-host` (host not in `[net] hosts`), `manifest.value` for an env name not matching `FREEMAPPER_[A-Z0-9_]{1,40}`.

- [ ] **Step 1: Write the failing tests** in `validate.rs`: `terrain_alone_is_a_valid_raster_set` (a manifest with `[terrain] backend = "lantmateriet"` and no `[orthophoto]` validates; today the finding says "an adapter with rasters has `[orthophoto]` and `[terrain]`"), `a_surface_without_terrain_is_still_refused`, `lantmateriet_endpoints_are_https_on_a_declared_host`, `auth_names_a_declared_host_and_freemapper_variables` (valid case plus `user_env = "HOME"` and a host outside `[net] hosts`), `a_folder_adapter_may_not_declare_auth` (loader for folder adapters returns `manifest.auth-builtin-only`; find where the loader distinguishes built-in from folder, in `loader.rs`), `stac_copc_point_cloud_takes_a_url_and_no_index`, `stac_gpkg_alkis_names_stac_collection_and_rules_file`.
- [ ] **Step 2: Run** `cargo test -p freemapper-manifest` and see each fail for the missing feature.
- [ ] **Step 3: Implement** the model fields, parsing and validation. `validate.rs:776` (`rasters_fit`): allow terrain alone; `[surface]` still needs `[terrain]`. `ManifestAdapter::new` (`adapter.rs`) builds the raster backend from `[terrain]` when `[orthophoto]` is absent, with the same provider check. `[auth]` keys: `host`, `user_env`, `password_env`, `products` (1 to 6 texts of at most 80 characters, shown when credentials are missing). Update `schema.rs` (JSON schema) the way a neighbouring table is.
- [ ] **Step 4: Run** `cargo test -p freemapper-manifest` and `cargo test -p freemapper --lib adapters` (the German manifests still load): PASS.
- [ ] **Step 5: Docs** in both manifest pages: the three tables, the new findings, the `lantmateriet` row in the endpoints table.
- [ ] **Step 6: Commit** `feat(manifest): Gelände ohne Orthofoto, [auth], stac-copc und stac-gpkg (#273)`.

### Task 2: A map without an orthophoto

**Files:**
- Modify: `crates/freemapper-adapter-api/src/rasters.rs` (`OrthophotoInput`), `crates/freemapper/src/project.rs` (around `match inputs.orthophoto?` and the estimate/export parts that name the orthophoto), `docs/adr/` (new ADR), message catalog of the program (find the file holding `warning:` texts, DE and EN)
- Test: `crates/freemapper/src/project.rs` test module (see how the neighbouring derive tests build a project with a stub backend)

**Interfaces:**
- Produces: `OrthophotoInput::Absent`; a backend returns it from `derive_inputs` when its region has no orthophoto. `derive` then writes hillshade, slope, relief, contours and the rest, no `templates/orthophoto.jpg`, and one warning.

- [ ] **Step 1: Write the failing test** `derive_continues_without_an_orthophoto_and_warns`: a stub backend whose `derive_inputs` gives `orthophoto: Ok(OrthophotoInput::Absent)` and a synthetic terrain; assert derive returns Ok, `templates/hillshade.png` exists, `templates/orthophoto.jpg` does not, and the progress lines contain the new warning (text in English: `warning: the region has no orthophoto; the map has no orthophoto template`, German in the catalog). Second test `a_failing_orthophoto_still_fails_derive`: `orthophoto: Err(..)` keeps today's error (German regions keep failing as before).
- [ ] **Step 2: Run** `cargo test -p freemapper --lib project:: -- derive_continues`: FAIL (variant missing).
- [ ] **Step 3: Implement** the variant and the branch in `project.rs`. The parts that read `orthophoto.jpg` afterwards (rock seen from the orthophoto, ADR 0126, `rock_seen`; the preview; the PDF template list; `templates` entries of the `.omap`) skip it when absent: find them with `grep -n 'ORTHOPHOTO_RELPATH\|orthophoto' crates/freemapper/src` and make each of them conditional, one test per place you change.
- [ ] **Step 4: Run** `cargo test -p freemapper --lib`: PASS.
- [ ] **Step 5: ADR** `docs/adr/<next>-map-without-orthophoto.md` (context: Sweden, decision: `Absent` plus warning, consequence: the check print and the training map have no photo; German regions unchanged) and the index row in `docs/adr/README.md`.
- [ ] **Step 6: Commit** `feat(derive): Karte ohne Orthofoto mit Warnung (ADR <n>)`.

### Task 3: Credentials from the environment

**Files:**
- Create: `crates/freemapper-adapter-api/src/auth.rs` (`pub mod auth` in `lib.rs`)
- Modify: `crates/freemapper-adapter-api/src/adapter.rs` (`RegionAdapter::auth`), `crates/freemapper-manifest/src/adapter.rs` (the manifest adapter answers it from `[auth]`), `crates/freemapper/src/fetch.rs`, `crates/freemapper-cli/src/main.rs` and `mcp.rs` (every place that passes `HttpFetcher::new()` to `fetch`, `run`, `estimate`: find with `grep -n 'HttpFetcher::new()' crates/freemapper-cli/src`)
- Test: `crates/freemapper-adapter-api/src/auth.rs` and `crates/freemapper/src/fetch.rs` test modules

**Interfaces:**
- Produces: `pub struct AuthSpec { pub host: String, pub user_env: String, pub password_env: String, pub products: Vec<String>, pub provider: String }`; `RegionAdapter::auth(&self) -> Option<&AuthSpec>` (default `None`); `pub struct Password(String)` with `fn new(String) -> Self`, `fn expose(&self) -> &str`, no `Debug`, no `Display`, no `Clone`; `pub fn credentials(spec: &AuthSpec, env: &dyn Fn(&str) -> Option<String>) -> Result<(String, Password), AdapterError>`; in `freemapper::fetch`: `pub fn fetcher_for(registry: &Registry, env: &dyn Fn(&str) -> Option<String>, used: &[AdapterId]) -> Result<HttpFetcher, AdapterError>` which adds `with_basic_auth(spec.host, user, password)` for each adapter in `used` that has an `AuthSpec`.

- [ ] **Step 1: Write the failing tests**: `missing_credentials_name_both_variables_and_the_products` (error text contains `FREEMAPPER_LM_USER`, `FREEMAPPER_LM_PASSWORD` and each entry of `products`), `an_empty_variable_counts_as_missing`, `the_password_is_in_no_message_and_no_debug` (build an error, a `format!("{:?}", spec)` and the progress line from a fetch with password `s3cret-πß`, assert none contains it; `Password` has no `Debug`, so a `static_assertions`-style check is a `compile_fail` doc test on `Password`), `fetcher_for_adds_credentials_only_for_adapters_in_use` (a registry with one adapter with `AuthSpec` and one without; a map in Germany needs no variable and works with the environment empty).
- [ ] **Step 2: Run** `cargo test -p freemapper-adapter-api auth`: FAIL.
- [ ] **Step 3: Implement.** The error is an `AdapterError` whose `Remedy` names the Geotorget orders (follow how `Remedy::AllowHost` is declared; add `Remedy::SetCredentials { variables, products }` and its German and English text in the catalog). Reading the process environment happens in exactly one place, the CLI and MCP call sites pass `&|name| std::env::var(name).ok()`.
- [ ] **Step 4: Run** `cargo test -p freemapper-adapter-api && cargo test -p freemapper --lib fetch`: PASS.
- [ ] **Step 5: Commit** `feat(fetch): Zugangsdaten aus FREEMAPPER_LM_USER und FREEMAPPER_LM_PASSWORD`.

### Task 4: STAC reader

**Files:**
- Create: `crates/freemapper-sources/src/stac.rs` (register `pub mod stac;` in `lib.rs`)
- Test: same file, synthetic JSON built in the tests

**Interfaces:**
- Produces: `pub struct StacItem { pub id: String, pub collection: String, pub bbox_proj: Option<[f64; 4]>, pub datetime: Option<String>, pub assets: BTreeMap<String, StacAsset> }`, `pub struct StacAsset { pub href: String, pub size: Option<u64>, pub media_type: Option<String> }`, `pub fn search(fetcher: &dyn Fetcher, base: &str, collection: &str, bbox_wgs84: [f64; 4], limit: u32) -> Result<Vec<StacItem>, FetchError>`. `base` comes from the manifest endpoint. It follows `links` with `rel = "next"` (the answer has no `numberMatched`), at most 20 pages, and asks `GET {base}/search?collections=…&bbox=…&limit=…`.

- [ ] **Step 1: Write the failing tests**: `search_reads_items_with_assets_and_proj_bbox` (one page, fields as in §3.4 of the reference: `proj:bbox`, `assets.data.href`, `assets.data.size`), `search_follows_next_links_until_none`, `a_loop_of_next_links_stops_after_20_pages` (error, not a hang), `an_item_without_the_data_asset_is_kept_with_empty_assets`, `a_404_collection_is_an_error_naming_it`.
- [ ] **Step 2: Run** `cargo test -p freemapper-sources stac`: FAIL.
- [ ] **Step 3: Implement** with `serde_json::Value` (no new dependency). Hrefs are taken as given; the caller checks them against the policy.
- [ ] **Step 4: Run**: PASS. **Step 5: Commit** `feat(sources): STAC-Leser für die Kataloge der Lantmäteriet`.

### Task 5: Terrain backend `lantmateriet`

**Files:**
- Create: `crates/adapters/se/` (`Cargo.toml` from `crates/adapters/cz/Cargo.toml`, `src/lib.rs`, `src/stac_terrain.rs`); add the crate to the workspace `Cargo.toml`
- Modify: `crates/freemapper-sources/src/raster_fetch.rs` (terrain-only helper)
- Test: `crates/adapters/se/src/stac_terrain.rs`, using `freemapper_sources::testing::tiled_float_tiff_with(grid, values, block, predictor, metadata, 8)` and `freemapper_adapter_api::testing::MapFetcher`

**Interfaces:**
- Consumes: `stac::search` (Task 4), `tiff_range::{read_layout, read_blocks}`, manifest endpoint `stac-hojd` through `RasterConfig::require("lantmateriet", "stac-hojd")` (`RasterConfig.endpoints`).
- Produces: `pub struct LantmaterietBackend { .. }` with `pub fn from_config(config: &RasterConfig) -> Result<Self, String>`, implementing `RasterBackend` with `ProviderInfo { id: AdapterId::built_in("lantmateriet"), cache_dir: Some("lantmateriet"), terrain_source: "lantmateriet-mhm".into(), kind: RasterKind::PerArea }`; `fetch` fills `terrain.tif` of the project and the `WcsRecord`; `derive_inputs` answers `orthophoto: Ok(OrthophotoInput::Absent)`, `dtm: Ok(HeightInput::Stored)`, `dsm: None`; `pub fn fetch_area_terrain(...)` in `raster_fetch.rs` (the terrain half of `fetch_area_heights`, no surface, no surface warning). The surface model and the orthophoto are `None`.

- [ ] **Step 1: Write the failing tests**: `reads_only_the_blocks_under_the_area_from_a_deflate_tile` (a synthetic 2 × 2 block COG, one STAC item answering `dtm-cog`; `MapFetcher` counts the bytes asked; assert the written `terrain.tif` holds the heights of the area and fewer bytes than the file were fetched), `joins_two_tiles_over_a_border`, `nodata_minus_9999_stays_nodata`, `a_401_from_the_download_host_says_the_order_is_missing` (text names `Markhöjdmodell Nedladdning`), `a_429_is_retried_by_the_policy_and_then_names_the_cap`, `a_tile_missing_in_stac_is_left_out_with_a_progress_line`, `an_area_outside_every_item_is_an_error`, `the_credentials_header_never_appears_in_the_progress_lines`.
- [ ] **Step 2: Run** `cargo test -p freemapper-adapters-se`: FAIL (crate empty).
- [ ] **Step 3: Implement.** Tile URLs come only from the STAC `assets.data.href`; check each against the adapter's policy through `PolicyFetcher`. Block reading and mosaicking follow `ldbv.rs` `grow_dom20`/`fetch_model` (read them first). The estimate lists the blocks to read per tile, in MB, part `Part::Terrain`.
- [ ] **Step 4: Run** `cargo test -p freemapper-adapters-se -p freemapper-sources`: PASS. `grep -n 'https\?://' crates/adapters/se/src` shows tests only.
- [ ] **Step 5: Commit** `feat(se): Höhenmodell der Lantmäteriet per Bereichsabruf`.

### Task 6: Point-cloud backend

**Files:**
- Create: `crates/adapters/se/src/copc.rs`
- Test: same file with a synthetic COPC-like LAZ built with `las` (`PointDataBuilder` + `fill_points`, as the existing point-cloud tests do; see `crates/adapters/de/src/ldbv_las.rs` tests)

**Interfaces:**
- Consumes: `stac::search` for `dsm-skoglig-copc`; endpoint name `stac-hojd`; `[point_cloud] kind = "stac-copc"` (the `url` there is the catalogue's folder prefix the hrefs must start with).
- Produces: `pub struct CopcBackend` implementing `PointCloudBackend` with `PointCloudInfo { id: lantmateriet, cache_dir: "lantmateriet-copc", source: "lantmateriet-skog", epsg: 25833, grid_km: 3, left_out: vec![] }`. **Decision (the open question of the spec):** the backend names its tiles itself from the STAC items, `grid_km` is read by no engine code (only by the German backends that set it), so it stays a whole number: 3, the 2.5 km grid rounded up, with a comment saying so. No change to `freemapper-adapter-api` and none to the German adapters.

- [ ] **Step 1: Write the failing tests**: `fetch_downloads_the_tiles_under_the_area_into_the_cache`, `a_cached_tile_is_not_downloaded_again` (offline mode uses the newest cached file), `estimate_lists_each_tile_with_its_size_from_stac` (80 MB class sizes from the `size` of the asset), `cached_finds_a_recorded_tile`, `a_download_over_the_size_limit_is_refused` (limit 200 MB per tile), `the_vegetation_classes_of_a_synthetic_tile_are_read` (classes 1, 2, 7, 9, 17, 18 as in the real tile; the existing vegetation reader accepts the file).
- [ ] **Step 2: Run**: FAIL. **Step 3: Implement** with `download_to` into `CacheDir` (name from the item id, never from the href's query). **Step 4: Run**: PASS. **Step 5: Commit** `feat(se): Laserdata Skog als Punktwolke (COPC)`.

### Task 7: Land cover from Marktäcke

**Files:**
- Create: `crates/adapters/se/src/marktacke.rs`, `crates/adapters/se/rules/marktacke.toml`
- Test: same file; a synthetic GeoPackage from `crates/freemapper-sources/tests/fixtures/gpkg/make_fixtures.py` pattern with the real column names (`objektidentitet`, `objekttypnr`, `objekttyp`, `geom`, table `mark`)

**Interfaces:**
- Consumes: `stac::search` on `marktacke` in `stac-vektor`, `gpkg::GeoPackage::{open, rows}`, `gpkg::geometry`.
- Produces: `pub struct MarktackeSource` implementing `AlkisSource`: `buildings` returns an empty `AlkisPart` in `Crs::Utm33`; `land_use` returns GeoJSON features (the shape `WfsAlkis` produces, see its tests in `alkis_source.rs`) with `properties.nutzart` in the LGB spelling, geometry in EPSG:25833. `rules/marktacke.toml` maps `objekttyp` to `nutzart`:

  | objekttyp | nutzart |
  |---|---|
  | Barr- och blandskog | Wald |
  | Låg bebyggelse | Wohnbaufläche |
  | Hög bebyggelse | Fläche gemischter Nutzung |
  | Industri- och handelsbebyggelse | Industrie- und Gewerbefläche |
  | Sjö | Stehendes Gewässer |
  | Vattendragsyta | Fließgewässer |
  | Torg | Platz |

  `Öppen mark` and every class not in the file give no feature (builder's call, marked in the ADR); each unknown name is reported once per fetch as a progress line. The file is checked by a test against `LAND_USE_CLASSES` so a typo in a `nutzart` fails.

- [ ] **Step 1: Write the failing tests**: `maps_the_known_classes_to_the_lgb_names`, `an_unknown_class_is_left_out_and_named_once`, `features_are_cut_to_the_area` (polygons outside the bbox are dropped, ones across the border kept whole), `reads_the_zip_from_disk_not_from_memory` (the GeoPackage is read from the extracted file; a 5 MB synthetic file keeps peak allocation under the file size ×2: assert through the reader API taking a `&Path`, not through a measurement), `two_municipalities_are_joined_without_duplicates_by_objektidentitet`, `a_401_names_the_marktacke_order`, `a_zip_without_a_gpkg_is_an_error_naming_the_file`.
- [ ] **Step 2: Run**: FAIL. **Step 3: Implement.** The zip is downloaded into the cache folder `lantmateriet-marktacke` by item id, unpacked with the `zip` code the other adapters use (`freemapper_sources::zip_range` or the zip crate already in the workspace; check `Cargo.toml`), the GeoPackage is opened by path. If `GeoPackage::open` takes bytes today, add `open_path` first, test first (`open_path_reads_the_same_rows_as_open`) in `gpkg.rs`.
- [ ] **Step 4: Run**: `cargo test -p freemapper-adapters-se -p freemapper-sources` PASS. **Step 5: Commit** `feat(se): Bodenbedeckung aus Marktäcke`.

### Task 8: The adapter itself

**Files:**
- Create: `crates/adapters/se/manifests/sweden/manifest.toml`, `crates/adapters/se/manifests/sweden/boundary.geojson`, `crates/adapters/se/src/sweden.rs`
- Modify: `crates/freemapper/src/adapters/mod.rs` (`freemapper_adapters_se::register_with_hosts(&mut registry, extra)`), `crates/freemapper/Cargo.toml`, workspace `Cargo.toml`, credit code (see how `cesko.rs` `attribution` builds `CreditLine`)
- Test: `crates/adapters/se/src/sweden.rs`, plus the registry tests in `crates/freemapper/src/adapters/`

**Interfaces:**
- Produces: `pub fn register(registry: &mut Registry) -> Result<(), DuplicateAdapter>` and `register_with_hosts(registry, extra: &ExtraHosts)` as in `cesko.rs`; adapter id `sweden`, provider `lantmateriet`, backend name `lantmateriet`. Manifest content: `[adapter] id = "sweden"`, `crs = 25833`, `object_id_prefix = "SE"`, names `{ en = "Sweden", de = "Schweden" }`; `[net] hosts = ["api.lantmateriet.se", "dl1.lantmateriet.se"]` plus the OSM extract host the Czech and Polish adapters list; `[[osm.extract]] geofabrik = "europe/sweden"` with the size from the Geofabrik page at commit time; `[terrain] backend = "lantmateriet"` with `[terrain.endpoints] stac-hojd = "https://api.lantmateriet.se/stac-hojd/v1"` and `stac-vektor = "https://api.lantmateriet.se/stac-vektor/v1"`; `[point_cloud] kind = "stac-copc"`; `[alkis] kind = "stac-gpkg"`; `[auth] host = "dl1.lantmateriet.se"`, `user_env = "FREEMAPPER_LM_USER"`, `password_env = "FREEMAPPER_LM_PASSWORD"`, `products = ["Markhöjdmodell Nedladdning", "Laserdata Skog", "Marktäcke"]`; `[attribution] licence = "CC-BY-4.0"`, `text = "© Lantmäteriet (CC BY 4.0)"`.

- [ ] **Step 1: Write the failing tests**: `sweden_is_registered_and_names_its_hosts`, `the_manifest_passes_adapters_check` (run the validator on the embedded manifest: no findings), `the_boundary_contains_stockholm_and_not_oslo` (EPSG:25833 points), `a_map_in_sweden_resolves_to_the_adapter_and_one_in_germany_does_not`, `no_buildings_warning_stays_unset`, `attribution_names_lantmateriet_once_when_terrain_and_land_cover_are_used`, `the_permit_notes_say_that_no_protected_areas_were_checked`.
- [ ] **Step 2: Run**: FAIL. **Step 3: Implement.** Build `boundary.geojson` once from the open product Kommun, län och rike (`kommun-lan-rike`, reading it with `gpkg`, dissolving the national outline, simplifying to about 100 m, writing EPSG:25833 GeoJSON, a few hundred KB at most). The script is a throwaway in the scratchpad; only its result and a line in the ADR about its origin are committed. The data stays derived geometry of an open product, no attribute rows.
- [ ] **Step 4: Run** `cargo test -p freemapper-adapters-se -p freemapper` and the CLI check `cargo run -p freemapper-cli -- adapters list` shows `sweden`. **Step 5: Commit** `feat(se): Schweden als Region-Adapter registriert`.

### Task 9: End-to-end with synthetic data

**Files:**
- Test: `crates/freemapper/tests/sweden_run.rs` (follow the nearest existing end-to-end test of an adapter with a `MapFetcher`; find with `ls crates/freemapper/tests`)

- [ ] **Step 1: Write the failing test** `a_swedish_map_is_made_from_synthetic_lantmateriet_data`: stub STAC answers for the three collections, a synthetic Deflate DTM tile, a synthetic COPC tile, a synthetic Marktäcke zip, a small OSM extract; `fetch`, `derive`, `export` succeed; the `.omap` has contours and a forest area from Marktäcke; no orthophoto template; the warning of Task 2 is printed; `ATTRIBUTION.txt` holds `© Lantmäteriet (CC BY 4.0)`; `freemapper.toml` and all output hold none of the test password.
- [ ] **Step 2: Run** (fails where the pieces do not join), fix at the joint, run again until PASS. **Step 3: Commit** `test(se): synthetische Karte von Schweden vom Abruf bis zum Export`.

### Task 10: Documentation and the gate before the pull request

**Files:**
- Modify: `docs/manual/de.md`, `docs/manual/en.md` (a section for Sweden: account, three orders, the two variables, what is not included, download sizes), `README.md`, `README.en.md` (one table row, as for every state), `docs/glossary.md` (Marktäcke, Geotorget, Laserdata Skog, STAC), `AGENTS.md` (supported states paragraph and the credentials note, without any real value), `docs/adapters/en.md` if it lists adapters, `CHANGELOG.md`, `CHANGELOG.en.md`, three ADRs (the GeoPackage reader, the Sweden adapter and its credentials, plus Task 2's), the index of `docs/adr/README.md`

- [ ] **Step 1:** Write the docs; run the `unslop` and `humanizer` skills on README, manual and changelog texts (the owner's rule), humanizer wins where they disagree.
- [ ] **Step 2:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`; `cargo test --workspace --locked` in the background, read its result before the next step.
- [ ] **Step 3:** `grep -rn 'lantmateriet' crates/*/src crates/adapters/se/src | grep 'https\?://'` shows tests and comments only; `git grep -n "$FREEMAPPER_LM_PASSWORD"` is not possible without printing it, so run the check as a test that scans the tracked files for the value read from `~/.config/freemapper/lm.env` and prints only the verdict.
- [ ] **Step 4:** Rebase on `github/main`, resolve conflicts in the manifest docs and ADR numbers, rerun the tests touched by the conflict.
- [ ] **Step 5:** Push `sweden-dtm-auth` (remote `github`), open one pull request with the label `all-systems` (fetcher and file code differ per system), watch `ci.yml`, merge once green. Then update `docs/regions/sweden-reference.md` on `sweden-reference` with the `[live]` facts (COPC readable with `las`, GeoPackage reader verified, Deflate reader added) and push it to PR #201.
- [ ] **Step 6:** Commit the docs `docs: Schweden im Handbuch, README, Glossar und Changelog`.

---

## Self-review

- **Spec coverage:** credentials (Task 3, 1), STAC (4), terrain (5), point cloud with the `grid_km` decision (6), land cover and the class table (7), buildings from OSM only (8: no ALKIS buildings, warning unset), map without orthophoto and ADR (2), boundary, registration, credit, notes (8), docs (10), manifest-first rule (Global Constraints, Task 1, 8). Not covered on purpose: Byggnad, orthophoto, Ytmodell.
- **Types:** `AuthSpec` is produced in Task 3 from the `Auth` of Task 1; `LantmaterietBackend::from_config` takes the `RasterConfig` that carries `stac-hojd`; `OrthophotoInput::Absent` of Task 2 is returned in Task 5.
- **Risk left to the builder:** whether `[point_cloud]`/`[alkis]` kinds need a factory in `adapters/de`-style registration (read how `point_cloud_backend` in `states/mod.rs` is wired and mirror it in `sweden.rs`); the exact message catalog file for the new texts.
