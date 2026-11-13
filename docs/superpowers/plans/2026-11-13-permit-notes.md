# Permit Notes (step 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** For a project, write `permits.md` and `permits.json` with the administrative units, protected areas and OSM operators the map area touches, from the BfN, the BKG (VG250) and the cached OSM extract.

**Architecture:** `freemapper-sources` gets a small GeoJSON WFS reader plus the BfN and VG250 sources; the `freemapper` crate gets `permits/` (OSM pass, shares, document model, storage, fetch orchestration, `permits.json`); the CLI crate renders `permits.md` from Fluent texts and adds the `permits` command, `run --permits`, the MCP tool and the area-page checkbox.

**Tech Stack:** Rust, `serde_json`, `geo` (BooleanOps, Area), `osmpbf` via `read_extract`, Fluent (`i18n/{de,en}/freemapper.ftl`), rmcp.

**Spec:** `docs/superpowers/specs/2026-11-13-permit-notes-design.md`

## Global Constraints

- Every request sends only the fixed freemapper User-Agent; no personal data anywhere.
- BfN and BKG requests go through `PolicyFetcher` with the policy `UrlPolicy::national()` (hosts `geodienste.bfn.de`, `sgx.geodatenzentrum.de`).
- Nothing asks the BfN or the BKG unless permit notes were asked for.
- `freemapper.toml`, `ATTRIBUTION.txt`, the `.omap`, candidates, PDFs and preview do not change.
- Never read or output `contact:*`, `email`, `phone`, `website` of OSM objects; operator/owner only from the OSM selection of the spec.
- Every `permits.md` starts with the disclaimer; the BfN line "Nicht für Planungszwecke geeignet." appears verbatim.
- German text says "Bundesland". Source code English; user texts DE+EN in Fluent.
- A failing source never fails `run`; `permits` alone exits 1 only when no source could be read.
- Work in worktree `.worktrees/permit-notes`, branch `permit-notes`; finish as a GitHub PR.

## Review Focus

1. An area across two Kreise or Bundesländer: shares of each level add up to 100 % ± 1 and every unit is named (Task 4 test `shares_of_an_area_across_two_kreise`).
2. An area with no protected area: the notes say so in words, not with an empty section (Task 6 test `no_protected_area_is_said`).
3. Wrong axis order gives HTTP 200 and nothing: the ignored live tests check Strausberg finds 2 NSG and Gemeinde Strausberg (Task 2).
4. Offline without stored data: `permits --offline` says what is missing and exits 1; `run --permits --offline` still writes the map (Task 6 tests).
5. BfN values that are a single space and `LEG_DATE` like `8/1/1966`: shown empty / as `1966-08-01` (Task 2 test `bfn_blank_values_and_dates`).

---

### Task 1: National host policy and GeoJSON WFS reader

**Files:**
- Modify: `crates/freemapper-adapter-api/src/policy.rs` (next to `CORE_HOSTS`)
- Create: `crates/freemapper-sources/src/geojson_wfs.rs`; Modify: `crates/freemapper-sources/src/lib.rs`

**Interfaces:**
- Produces: `pub const NATIONAL_HOSTS: [&str; 2] = ["geodienste.bfn.de", "sgx.geodatenzentrum.de"]`; `UrlPolicy::national() -> UrlPolicy`.
- Produces: `pub struct LatLonBox { pub min_lat: f64, pub min_lon: f64, pub max_lat: f64, pub max_lon: f64 }`;
  `pub struct GeoJsonWfs { pub base: &'static str, pub output_format: &'static str, pub bbox_crs_suffix: bool }`;
  `pub fn get_features(fetcher: &dyn Fetcher, service: &GeoJsonWfs, type_name: &str, bbox: &LatLonBox) -> Result<Vec<serde_json::Value>, FetchError>`.
- Request form (verbatim from the spec): `service=WFS&version=2.0.0&request=GetFeature&outputFormat=<fmt>&srsName=urn:ogc:def:crs:EPSG::4326&typeNames=<type>&count=500&startIndex=<n>&bbox=<minLat>,<minLon>,<maxLat>,<maxLon>[,urn:ogc:def:crs:EPSG::4326]`. Pages until a page has fewer than 500 features; refuse a response over 50 MB (`FetchError` with the URL).

- [ ] Step 1: tests `national_policy_allows_only_the_two_hosts` (bfn and bkg URLs pass `check_url`, `https://example.org` fails), `query_puts_latitude_first` (URL for box 52.53/13.83/52.545/13.86 contains `bbox=52.53,13.83,52.545,13.86`), `vg250_query_ends_the_bbox_with_its_crs`, `pages_until_a_short_page` (MapFetcher with 500 + 3 features → 503, two requests), `a_huge_response_is_refused`.
- [ ] Step 2: run `cargo test -p freemapper-sources geojson_wfs` and `-p freemapper-adapter-api national`; expect FAIL (not defined).
- [ ] Step 3: implement.
- [ ] Step 4: same commands; expect PASS.
- [ ] Step 5: commit `feat(sources): a GeoJSON WFS reader and the host rules of the national sources`.

### Task 2: BfN and VG250 sources

**Files:**
- Create: `crates/freemapper-sources/src/bfn.rs`, `crates/freemapper-sources/src/vg250.rs`, fixtures under `crates/freemapper-sources/tests/fixtures/permits/` (recorded small responses: one LSG, one NSG, one Gemeinde, one Kreis; trimmed to a few vertices)

**Interfaces:**
- Consumes: Task 1.
- Produces:
  - `pub enum ProtectedKind { Nationalpark, NationalesNaturmonument, Naturschutzgebiet, Ffh, Vogelschutzgebiet, Biosphaerenreservat, BiosphaerenreservatZone, Landschaftsschutzgebiet, Naturpark }` with `type_name() -> &'static str` (the nine `schutzgebiet:` names of the spec) and `ALL` in the spec's order.
  - `pub struct ProtectedArea { pub kind: ProtectedKind, pub name: String, pub legal_date: Option<NaiveDate>, pub legal_title: Option<String>, pub legal_link: Option<String>, pub year: Option<i32>, pub geometry: MultiPolygon<f64> /* EPSG:4326, x=lon */ }`; `pub fn parse_bfn(kind, feature: &Value) -> Option<ProtectedArea>`; `pub fn fetch_bfn(fetcher, bbox) -> Vec<(ProtectedKind, Result<Vec<Value>, FetchError>)>` (one entry per kind, raw features kept for storage).
  - `pub enum AdminLevel { Land, Kreis, Verwaltungsgemeinschaft, Gemeinde }` with type names `vg250:vg250_lan|krs|vwg|gem`; `pub struct AdminUnit { pub level: AdminLevel, pub name: String /* gen */, pub kind: String /* bez */, pub ags: String, pub geometry: MultiPolygon<f64> }`; `parse_vg250`, `fetch_vg250` like BfN.
  - `pub const BFN_CREDIT: &str` and `pub const BKG_CREDIT: &str` with `{year}` placeholder, texts verbatim from the spec; `pub const BFN_NOT_FOR_PLANNING: &str = "Nicht für Planungszwecke geeignet."`.
- Note: GeoJSON coordinates with `srsName=…EPSG::4326` come in lat/lon order from GeoServer (VG250) and possibly lon/lat from the BfN; the fixtures decide, and the parsers normalise to x=lon, y=lat. Pin it with a test per service.

- [ ] Step 1: tests `bfn_lsg_fixture_parses` (name "Potsdamer Havelseengebiet", kind Landschaftsschutzgebiet, geometry centroid within 13.0–13.1 / 52.3–52.5), `bfn_blank_values_and_dates` (`" "` → `None`, `8/1/1966` → 1966-08-01), `vg250_gemeinde_fixture_parses` (gen "Potsdam", ags "12054000", centroid lon 13.0–13.2), and ignored live tests `live_bfn_finds_the_nsg_of_strausberg` (≥ 2 Naturschutzgebiete) and `live_vg250_finds_strausberg` (a Gemeinde with gen "Strausberg"), box lon 13.83–13.86, lat 52.53–52.545.
- [ ] Step 2: `cargo test -p freemapper-sources bfn vg250`; expect FAIL.
- [ ] Step 3: implement; record fixtures with the fixed User-Agent only.
- [ ] Step 4: PASS; run the ignored live tests once (`-- --ignored live_`), note the result in the commit body.
- [ ] Step 5: commit `feat(sources): protected areas of the BfN and administrative units of VG250`.

### Task 3: OSM permit features

**Files:**
- Create: `crates/freemapper/src/permits/mod.rs`, `crates/freemapper/src/permits/osm.rs`; Modify: `crates/freemapper/src/lib.rs`

**Interfaces:**
- Consumes: `read_extract(path, &GeoBox, TagFilter)` and `OsmFeature` (`sources/osm.rs`).
- Produces: `pub fn wants(tags: &[(String, String)]) -> bool` (the OSM selection of the spec, verbatim; areas only);
  `pub enum OsmFindingKind { ProtectedArea, Operated }`;
  `pub struct OsmFinding { pub kind: OsmFindingKind, pub osm_id: String, pub object: String /* e.g. "leisure=park" */, pub name: Option<String>, pub operator: Option<String>, pub owner: Option<String>, pub protect_class: Option<String>, pub protection_title: Option<String>, pub geometry: MultiPolygon<f64> }`;
  `pub fn findings(features: Vec<OsmFeature>) -> Vec<OsmFinding>` (copies only the keys named here).

- [ ] Step 1: tests `a_park_with_an_operator_is_wanted`, `residential_land_with_an_operator_is_not` (`landuse=residential` + `operator`), `a_house_with_an_owner_is_not` (`building=house` + `owner`), `a_protected_area_without_operator_is_wanted`, `contact_tags_never_reach_a_finding` (feature with `email`, `phone`, `contact:email` → finding has none of them; assert on the serialized finding), `a_node_is_not_an_area`.
- [ ] Step 2–4: `cargo test -p freemapper permits::osm`; FAIL, implement, PASS.
- [ ] Step 5: commit `feat(permits): the OSM objects that tell who runs an area`.

### Task 4: Document model, shares and hints

**Files:**
- Create: `crates/freemapper/src/permits/notes.rs`

**Interfaces:**
- Consumes: Tasks 2 and 3 types; `MapArea::projected_polygon_in(crs)`, `ProjectFile::crs()`, `MapKind`.
- Produces:
  - `pub struct Inputs { pub admin: SourceInput<AdminUnit>, pub protected: SourceInput<ProtectedArea>, pub osm: SourceInput<OsmFinding> }`; `pub enum SourceInput<T> { Read { items: Vec<T>, fetched: NaiveDate }, Unreachable { reason: String }, NotRead }`.
  - `pub struct PermitNotes { pub schema: u32 /* 1 */, pub created: NaiveDate, pub map_kind: MapKind, pub area_m2: f64, pub admin: Vec<Share<AdminEntry>>, pub protected: Vec<Share<ProtectedEntry>>, pub osm_protected: Vec<Share<OsmEntry>>, pub operators: Vec<OperatorEntry>, pub hints: Vec<Hint>, pub sources: Vec<SourceStatus> }` (all `Serialize`/`Deserialize`); `pub struct Share<T> { pub item: T, pub percent: f64, pub at_edge: bool }`; `pub struct Hint { pub key: String, pub args: BTreeMap<String, String> }`; `pub struct SourceStatus { pub id: String /* "bfn" | "vg250" | "osm" */, pub state: String /* "read" | "unreachable" | "not-read" */, pub fetched: Option<NaiveDate>, pub reason: Option<String>, pub credit: Option<String> }`.
  - `pub fn assemble(area: &MapArea, crs: Crs, kind: MapKind, inputs: &Inputs, today: NaiveDate) -> PermitNotes`.
- Rules (verbatim from the spec): share = area(map ∩ unit) / area(map), in the project's UTM zone; `at_edge` when share < 0.5 %; admin order Land, Kreis, Verwaltungsgemeinschaft, Gemeinde; protected order Nationalpark, Nationales Naturmonument, NSG, FFH, Vogelschutzgebiet, Biosphärenreservat (zone after its reserve), LSG, Naturpark, within a kind by share descending; an OSM protected area is dropped when a BfN area covers ≥ 0.5 of the smaller of the two; operators grouped by operator/owner name with the sorted, de-duplicated object kinds.
- Hint keys (Fluent ids, rendered in Task 6): `permits-hint-nature-authority` (args `kreis`, `kreisfrei`=yes/no), `permits-hint-national-park`, `permits-hint-nsg-paths`, `permits-hint-natura2000`, `permits-hint-order-office` (args `gemeinde`, `amt` optional), `permits-hint-city-state` (Berlin, Hamburg: Land = Gemeinde), `permits-hint-park-operator` (arg `operator`), `permits-hint-school-operator` (schoolyard maps, arg `operator`), `permits-hint-owners-unknown` (always). A Kreis whose `bez` is "Kreisfreie Stadt" sets `kreisfrei=yes`.

- [ ] Step 1: tests `shares_of_an_area_across_two_kreise` (synthetic squares: 70/30 → 70 and 30, sum 100 ± 1), `a_unit_touching_the_edge_is_marked`, `protected_areas_follow_the_order_of_the_spec`, `an_osm_area_the_bfn_covers_is_dropped` and `an_osm_area_beside_it_stays`, `a_kreisfreie_stadt_is_its_own_nature_authority`, `berlin_gets_the_city_state_hint`, `an_amt_is_named_in_the_order_office_hint`, `unreachable_sources_are_recorded`, `notes_round_trip_through_json`.
- [ ] Step 2–4: `cargo test -p freemapper permits::notes`; FAIL, implement, PASS.
- [ ] Step 5: commit `feat(permits): shares, order and hints of the permit notes`.

### Task 5: Storage and fetch orchestration

**Files:**
- Create: `crates/freemapper/src/permits/store.rs`, `crates/freemapper/src/permits/fetch.rs`

**Interfaces:**
- Consumes: Tasks 1–4; `cached_extracts`/`extracts_for` and `read_extract` (`sources/osm.rs`); `Cache`; `PolicyFetcher::new(inner, &UrlPolicy::national())`.
- Produces:
  - `pub const PERMITS_DIR: &str = "data/permits"`, `pub const PERMITS_JSON: &str = "permits.json"`, `pub const PERMITS_MD: &str = "permits.md"`.
  - `pub fn store_inputs(dir: &Path, raw: &RawInputs) -> io::Result<()>` writing `bfn-<kind>.geojson`, `vg250-<level>.geojson`, `osm.geojson` plus `sources.json` (state and fetch date per source); `pub fn load_inputs(dir: &Path) -> Result<Inputs, PermitsError>`.
  - `pub fn fetch_inputs(project: &ProjectFile, dir: &Path, fetcher: &dyn Fetcher, cache: &Cache, offline: bool, progress: &mut dyn FnMut(Note)) -> Result<Inputs, PermitsError>`: online fetches BfN and VG250 through the national policy and reads OSM from the newest cached extract (never downloads one); offline loads what is stored; a source that fails becomes `Unreachable` and keeps its earlier stored data out (stale data is not mixed with a new date).
  - `pub fn write_permit_notes(dir: &Path, project: &ProjectFile, inputs: &Inputs, today: NaiveDate) -> Result<PermitNotes, PermitsError>` writing `permits.json`.
  - `PermitsError::NothingRead` when all three sources are not read or unreachable.

- [ ] Step 1: tests (MapFetcher with the Task 2 fixtures, temp dirs): `fetch_then_offline_gives_the_same_notes`, `offline_without_stored_data_is_nothing_read`, `a_down_bfn_leaves_vg250_and_osm`, `no_cached_extract_means_osm_not_read`, `requests_outside_the_national_hosts_are_refused` (policy fetcher wrapping a MapFetcher), `osm_findings_are_stored_for_offline_use`.
- [ ] Step 2–4: `cargo test -p freemapper permits::`; FAIL, implement, PASS. Measure `fetch_inputs` on a real Brandenburg project with a cached extract (`examples/brandenburg-sanssouci` copied to the scratchpad) and write the time into the commit body; if over 20 s, Task 7 makes the MCP tool a job.
- [ ] Step 5: commit `feat(permits): fetch, store and read the permit notes of a project`.

### Task 6: Markdown, `permits` command, `run --permits`

**Files:**
- Create: `crates/freemapper-cli/src/permits.rs`; Modify: `crates/freemapper-cli/src/main.rs` (`Commands::Permits`, `--permits` on `Run`), `crates/freemapper-cli/src/pipeline.rs` (`Written::Permits`, kind `"permits"` in `result_files`), `crates/freemapper-cli/i18n/{de,en}/freemapper.ftl`, `crates/freemapper-cli/tests/cli.rs`

**Interfaces:**
- Consumes: Task 5; `lock::acquire(dir, who)`; the Fluent loader the CLI uses for other texts.
- Produces: `pub fn render_markdown(notes: &PermitNotes, lang: Lang) -> String`; `pub fn run_permits(dir: &Path, offline: bool, cache: &Cache, lang: Lang, progress) -> Result<PathBuf, CliError>` (lock, fetch, `permits.json`, `permits.md`).
- Markdown layout: title `permits-title`; disclaimer `permits-disclaimer` (research C5 text) and `permits-bfn-merge` (spec text); sections Verwaltung, Schutzgebiete (BfN), Weitere Schutzgebiete aus OpenStreetMap, Betreiber und Eigentümer laut OpenStreetMap, Hinweise, Quellen und Datenstand; an empty section says `permits-none-<section>`; percent without decimals, "< 1 %" below one, `at_edge` lines under `permits-at-edge`.

- [ ] Step 1: tests `markdown_starts_with_the_disclaimer` (de and en), `no_protected_area_is_said`, `the_bfn_line_is_verbatim`, `a_source_that_was_down_is_named`; CLI tests `permits_offline_without_data_exits_with_1`, `run_with_permits_offline_still_writes_the_map` (warning line, exit 0), `permits_appears_in_the_written_files`; Fluent catalog test passes for every new key in both languages.
- [ ] Step 2–4: `cargo test -p freemapper-cli permits`; FAIL, implement, PASS.
- [ ] Step 5: commit `feat(cli): freemapper permits and run --permits write the permit notes`.

### Task 7: MCP tool and area page

**Files:**
- Modify: `crates/freemapper-cli/src/mcp.rs`, `crates/freemapper-cli/src/select_run*.rs` (request field), `crates/freemapper-cli/web/run.js` and its HTML, ftl files, `crates/freemapper-cli/tests/mcp.rs`

**Interfaces:**
- Consumes: Task 6 `run_permits`, `render_markdown`.
- Produces: MCP tool `permits_for_area { dir: String, offline: Option<bool> }` returning the Markdown text (or a job id if Task 5 measured > 20 s); `permits: Option<bool>` on `start_pipeline` and `open_area_selector`; page checkbox id `out-permits`, label `page-output-permits` "Hinweise zu Genehmigungen" / "Permit notes", off by default, sent as `permits` in the run request.

- [ ] Step 1: tests `permits_for_area_returns_the_notes` (offline fixture project), `permits_for_area_refuses_a_relative_path`, `start_pipeline_with_permits_lists_the_file`, tool and parameter descriptions present in both ftl files (existing description test), page test that the run request carries `permits` only when ticked.
- [ ] Step 2–4: `cargo test -p freemapper-cli mcp select_run`; FAIL, implement, PASS.
- [ ] Step 5: commit `feat(mcp): permits_for_area and the permit notes on the area page`.

### Task 8: Documentation

**Files:**
- Create: `docs/adr/0115-permit-notes-from-national-sources.md`; Modify: `docs/adr/README.md`, `docs/glossary.md`, `docs/manual/{de,en}.md` (new section), `README.md`, `README.en.md` (the "Geplant" paragraph), `AGENTS.md` (command table, MCP tools, project folder files), `CHANGELOG.md`, `CHANGELOG.en.md` (Hinzugefügt/Added, under Unveröffentlicht/Unreleased)

- [ ] Step 1: write the ADR (context, decision with the spec's calls, the BfN wording as Claude's, consequences, later steps); index line.
- [ ] Step 2: manual, README, AGENTS.md, glossary (Genehmigungshinweise / permit notes, Schutzgebiet / protected area, Verwaltungseinheit / administrative unit), changelog; run unslop and humanizer over the user prose.
- [ ] Step 3: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`, `scripts/golden.sh test`; all green, golden unchanged.
- [ ] Step 4: commit `docs: permit notes (ADR 0115), manual, README, AGENTS.md, changelog`; push branch; open PR with the run of the live tests and the timing in the description.
