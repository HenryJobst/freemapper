# Forest Map Kind Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A third map kind `forest` (ISOM 2017-2, 1:15 000 / 1:10 000 / 1:7500) that turns the data freemapper already fetches into a printable forest training map, with the OSM path network as its core.

**Architecture:** `MapKind::Forest` runs the sprint pipeline with forest parameters. The ISOM symbol set answers the ISSprOM codes the build modules use through an alias table (`SymbolSet::id_for_code`), so the modules keep their codes; OSM rules get a `forest` column for paths and land cover in ISOM codes. Symbol enlargement and the symbol set depend on kind *and* scale. Contours get a fractional interval chosen from the relief.

**Tech Stack:** Rust (workspace freemapper), OOM `.omap` XML symbol sets, TOML rules, freemapper-core (`freemapper-contract`).

**Spec:** `docs/superpowers/specs/2026-11-13-forest-map-design.md`

## Global Constraints

- Scales: 15 000 (symbols 100 %), 10 000 (150 %), 7500 (200 %); default 15 000. They come from `freemapper_contract::MapKind::Forest.drawing_scales()` (core PR HenryJobst/freemapper-core#7).
- Schoolyard and sprint output must stay byte for byte as it is: the golden net (`cargo test -p freemapper --test adapters_golden -- --include-ignored` with the store) passes without a bless.
- No green from the nDSM or a guess; green only from OSM `natural=scrub` (406) and dense scrub (408), as candidates; the laser vegetation stays the opt-in group `vegetation`.
- OSM paths of the forest kind go on the map (`target = "map"`); nothing is merged or moved. Left out: sidewalks and crossings (`footway=sidewalk|crossing`), cycleways tagged `cycleway=sidewalk`, and driveways and parking aisles (`service=driveway|parking_aisle`).
- Forest wins over ALKIS park/recreation land: no 401 where OSM or ALKIS says forest.
- Out-of-bounds areas are 520, never 709.
- Source code and technical docs English; user docs German and English; German says "Bundesland". Every change: changelog entry in `CHANGELOG.md` and `CHANGELOG.en.md`; the decision gets an ADR (next free number at commit time, title "Forest map kind").
- Third-party dependencies at least two weeks old; no new dependency is planned.
- Build with `CARGO_TARGET_DIR=/Users/henryprivat/Documents/gitrepos/freemapper/target`; check free disk (`df -h /`) before a release build (> 15 GB).
- Work goes in as a GitHub pull request from branch `forest-map`; merged only after the user's yes and after core is tagged and pinned.

## Review Focus

1. A forest project at 1:7500: every paper distance (point gaps, wall pairs, north-line spacing, stair widths) and the symbol set are enlarged 200 %, not 100 % or 150 %. Test in Task 1 and Task 2.
2. A forest area with no OSM paths and no ALKIS land use at all: the map is written (white forest, contours) without a panic on a code the ISOM set lacks. Test in Task 4.
3. A very flat area (DTM range below 15 m): interval 2.5 m, index contour every 12.5 m, the margin says "2,5 m"/"2.5 m". An explicit `contour_interval_m = 5` in `freemapper.toml` wins over the automatic choice; 0 or a negative value is refused. Tests in Task 5.
4. An existing sprint or schoolyard project file (no `contour_interval_m` key) loads and rebuilds unchanged. Golden net in Task 7, round-trip test in Task 5.
5. `--kind forest --scale 4000` is refused with the allowed scales named. Test in Task 1.

---

### Task 0: Pin freemapper-core with the forest scales

**Files:**
- Modify: `Cargo.toml` (workspace `[patch."https://github.com/HenryJobst/freemapper-core"]`)

- [ ] **Step 1:** Add a `[patch]` for every core crate the workspace uses, pointing at `/Users/henryprivat/Documents/gitrepos/freemapper-core/.worktrees/forest-kind/crates/<crate>` (ADR 0006 of core). Comment: `# Until core tags the forest scales (freemapper-core#7); replaced by the tag before the merge.`
- [ ] **Step 2:** `cargo build -p freemapper --locked` fails on `Cargo.lock`; run `cargo update -w --offline`, then `cargo test -p freemapper --lib contract_kinds_are_the_kinds_of_the_suite`. Expected: FAIL (`forest` written by the contract, not by freemapper). This failure is what Task 1 fixes.
- [ ] **Step 3:** Commit `chore: pin freemapper-core to the forest-kind branch for the forest map work`.

### Task 1: `MapKind::Forest` and its parameters

**Files:**
- Modify: `crates/freemapper/src/project.rs:188-330` (enum, `ALL`, `contract`, parameters), tests near `:5538` and `:14055`
- Modify: every caller of `symbol_enlargement()` and `symbol_set()` (`build/{stairs,barriers,walls,points,barrier_gaps}.rs`, `project/export.rs:1718`, and the pass-through modules `barriers`, `barrier_gaps`, `stairs`, `walls`, `state`, `berlin`)
- Modify: `crates/freemapper-cli/src/main.rs:623-636` (clap `Kind`), `mcp.rs:347,450,673`, `select.rs:133,370-376,481`, `crates/freemapper-cli/i18n/{de,en}/freemapper.ftl` (kind names and help)

**Interfaces:**
- Produces: `MapKind::Forest`; `MapKind::ALL: [MapKind; 3]`; `MapKind::symbol_enlargement(self, scale: u32) -> f64`; `MapKind::symbol_set(self, scale: u32) -> SymbolSet` (Task 2 fills the forest arm); `MapKind::contour_interval_m(self) -> f64` (Task 5 uses it as the default).
- Forest values: `default_buffer_m` 100, `slope_black_deg` 30, `contour_sigma_m` 3, `relief_sigma_m` 8, `landform_min_height_m` 1, `contour_interval_m` 5.0, `symbol_enlargement(15000)` 1.0, `(10000)` 1.5, `(7500)` 2.0. Schoolyard 1.25 and sprint 1.0 whatever the scale.

- [ ] **Step 1: Failing tests** in `project.rs` tests:

```rust
#[test]
fn forest_is_a_kind_with_the_isom_scales() {
    assert_eq!(MapKind::from_name("forest"), Some(MapKind::Forest));
    assert_eq!(MapKind::Forest.default_scale(), 15000);
    assert_eq!(MapKind::Forest.allowed_scales(), [7500, 10000, 15000]);
    assert_eq!(MapKind::Forest.symbol_enlargement(15000), 1.0);
    assert_eq!(MapKind::Forest.symbol_enlargement(10000), 1.5);
    assert_eq!(MapKind::Forest.symbol_enlargement(7500), 2.0);
    assert_eq!(MapKind::Schoolyard.symbol_enlargement(1000), 1.25);
    assert_eq!(MapKind::Sprint.symbol_enlargement(5000), 1.0);
    assert_eq!(MapKind::Forest.contour_interval_m(), 5.0);
    let err = MapKind::Forest.check_scale(4000).unwrap_err().to_string();
    assert!(err.contains("1:7500, 1:10000, 1:15000"), "{err}");
}
```

and a CLI test that `init --kind forest --scale 10000` writes `map_kind = "forest"` and `scale_denominator = 10000` (next to the existing `init` tests in `crates/freemapper-cli/tests/`).
- [ ] **Step 2:** Run `cargo test -p freemapper --lib forest_is_a_kind`; expected: does not compile (`MapKind::Forest`).
- [ ] **Step 3:** Implement. `symbol_enlargement(self, scale)`: forest = `15000.0 / scale` rounded to the table (1.0, 1.5, 2.0). Every existing `match` on `MapKind` gets a `Forest` arm; where this plan says nothing else, Forest behaves like Sprint. Callers pass the project's or georef's `scale_denominator`. `contour_interval_m` returns `f64` (schoolyard 1.0, sprint 2.0); its callers change in Task 5, here only `f64::from` drops.
- [ ] **Step 4:** `cargo test -p freemapper --lib` and `cargo test -p freemapper-cli`; expected PASS, `contract_kinds_are_the_kinds_of_the_suite` included.
- [ ] **Step 5:** Commit `feat(project): the map kind forest with the ISOM scales`.

### Task 2: The ISOM 2017-2 symbol set

**Files:**
- Create: `assets/symbol-sets/ISOM 2017-2_15000.omap` (from OOM, see below), `assets/symbol-sets/isom-aliases.toml`
- Modify: `assets/symbol-sets/README.md`, `crates/freemapper/src/omap/symbol_set.rs`, `crates/freemapper/src/project.rs` (`symbol_set` forest arm)

**Interfaces:**
- Consumes: `MapKind::symbol_set(self, scale)` from Task 1.
- Produces: `SymbolSet::isom_2017_2(enlargement: f64) -> SymbolSet` (with sheet symbols added by the caller as for the others); `SymbolSet::id_for_code(&self, code: &str) -> Option<u32>` now resolves aliases: an ISSprOM code listed in `isom-aliases.toml` resolves to its ISOM code's id, or to `None` when listed as dropped.

- [ ] **Step 1:** Copy `ISOM 2017-2_15000.omap` from OpenOrienteering Mapper (`symbol sets/15000/` at the same commit as ISSprOM, `eae21b05851495923ac245fdf4b89cf575df38ce`; fetch it from GitHub, not from the installed app). Record source, commit and SHA-256 of original and changed file in the README. Change the colour table exactly as the README describes for ISSprOM (Rev. 4 values, same colour names).
- [ ] **Step 2: Failing tests** in `symbol_set.rs`:

```rust
#[test]
fn the_isom_set_has_the_path_classes_and_answers_issprom_codes() {
    let set = SymbolSet::isom_2017_2(1.0);
    for code in ["502", "503", "504", "505", "506", "507", "508", "520", "521"] {
        assert!(set.id_for_code(code).is_some(), "{code}");
    }
    assert_eq!(set.id_for_code("505.1"), set.id_for_code("505"));
    assert_eq!(set.id_for_code("505.2"), set.id_for_code("504"));
    assert_eq!(set.id_for_code("990"), None);
}

#[test]
fn the_isom_set_at_1_to_10000_is_enlarged_150_percent() {
    // The line width of 505 at 1:10000 is 1.5 times that at 1:15000.
}
```

(the second test compares `first_attribute_after(.., r#" code="505""#, "line_width")` of `isom_2017_2(1.0)` and `isom_2017_2(1.5)`).
- [ ] **Step 3:** Implement. `isom_2017_2(enlargement)` uses `enlarge_symbols` as `school_2019` does and sets `<georeferencing scale=…>` to 15000 / enlargement. `isom-aliases.toml`: `[alias] "<issprom>" = "<isom>"` and `drop = [...]`; start with these and extend after grepping every literal code passed to `id_for_code` and every `sprint = "…"` code of `osm-candidates.toml` that has no `forest` value:

| ISSprOM | ISOM | | ISSprOM | ISOM |
|---|---|---|---|---|
| 501.6, 501.7 | 503 | | 532.6–532.9 | 506 |
| 501.8, 501.9 | 505 | | 990, 541–545, 321 | drop |
| 505.1 | 505 | | 501.1, 501.10 | 501 |
| 505.2 | 504 | | 410.1 | 410 |

Codes both sets have with the same meaning (101–113, 2xx, 3xx, 401–418, 501, 509–531) need no entry.
- [ ] **Step 4:** Test: a new test `every_code_freemapper_draws_has_an_isom_answer` collects the `sprint` codes of `osm-candidates.toml` (via `Rules`) and asserts each is in the ISOM set, aliased or dropped. Run `cargo test -p freemapper --lib symbol_set`; PASS.
- [ ] **Step 5:** Commit `feat(omap): the ISOM 2017-2 symbol set with the colours of IOF Rev. 4`.

### Task 3: OSM rules for the forest kind

**Files:**
- Modify: `crates/freemapper/src/build/candidates.rs` (`forest: Option<String>` in the rule entry and `Rule`, `wants`, `symbol_for`), `assets/rules/osm-candidates.toml`

**Interfaces:**
- Consumes: `MapKind::Forest` (Task 1).
- Produces: `Rules::symbol_for(&self, tags, geometry, MapKind::Forest) -> Option<Match>`: a rule takes part when it has a `forest` value; the value `"-"` matches and draws nothing (the feature is skipped, later rules are not asked). A rule without `forest` but with `sprint` takes part with the sprint code (aliases do the rest).

- [ ] **Step 1: Failing tests** in `candidates.rs` tests, one per row of the spec's path table, e.g.:

```rust
#[test]
fn forest_paths_follow_the_isom_classes() {
    let rules = Rules::builtin();
    let code = |tags: &[(&str, &str)]| {
        let tags: Tags = tags.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        rules.symbol_for(&tags, &line(), MapKind::Forest).map(|m| (m.code, m.target))
    };
    assert_eq!(code(&[("highway", "track"), ("tracktype", "grade1")]), Some(("504".into(), Target::Map)));
    assert_eq!(code(&[("highway", "track"), ("tracktype", "grade4")]), Some(("505".into(), Target::Map)));
    assert_eq!(code(&[("highway", "path")]), Some(("506".into(), Target::Map)));
    assert_eq!(code(&[("highway", "path"), ("trail_visibility", "bad")]), Some(("507".into(), Target::Map)));
    assert_eq!(code(&[("highway", "footway"), ("footway", "sidewalk")]), None);
    assert_eq!(code(&[("highway", "tertiary")]), Some(("502".into(), Target::Map)));
    assert_eq!(code(&[("highway", "steps")]), Some(("506".into(), Target::Map)));
    assert_eq!(code(&[("natural", "scrub")]), None); // a line, not an area
}
```

plus `forest_land_cover_is_open_land_heath_and_scrub` (closed ways: `landuse=meadow` → 401 candidates, `natural=heath` → 403, `natural=scrub` → 406, `natural=scrub`+`density=dense` → 408) and `sprint_and_schoolyard_rules_do_not_change` (a handful of existing sprint/schoolyard assertions, unchanged codes).
- [ ] **Step 2:** Run `cargo test -p freemapper --lib build::candidates`; FAIL.
- [ ] **Step 3:** Implement the field and put the forest-only rules **before** the sprint highway rules in `osm-candidates.toml`, under a comment `# Forest maps (ISOM 2017-2), spec 2026-11-13`; rows exactly as the spec's path table, `target = "map"`. Settlement service ways: `highway=service` inside `landuse=residential` is not decidable per way; drop `service=driveway|parking_aisle` with `forest = "-"`.
- [ ] **Step 4:** Run the tests; PASS.
- [ ] **Step 5:** Commit `feat(osm): forest rules for paths and land cover in ISOM codes`.

### Task 4: Build modules for the forest kind

**Files:**
- Modify: `crates/freemapper/src/build/{candidates.rs:577,591, points.rs:933,955, out_of_bounds.rs:71, open_land.rs:200}`, `crates/freemapper/src/ocd/{symbols.rs:33, convert.rs:140,411}`, `crates/freemapper/src/project/export.rs:607`, every `id_for_code(...)` call that `expect`s, `crates/adapters/de/src/berlin.rs:101-103`, `crates/freemapper-sources/src/nas.rs:62-63` (tell FableWorker before touching these two)

**Interfaces:**
- Consumes: aliases of Task 2, rules of Task 3.
- Produces: `build/open_land.rs`: for `MapKind::Forest`, no 401 where an OSM `landuse=forest|natural=wood` area or an ALKIS forest (`Wald`, `Gehölz`) lies; the sprint walkable area (990) is not built for Forest.

- [ ] **Step 1: Failing tests:** `open_land` test `forest_wins_over_an_alkis_park_on_a_forest_map` (ALKIS park polygon overlapped by an OSM forest polygon → no 401 for Forest, 401 for Sprint); an export test `a_forest_map_without_paths_or_land_use_is_written` (project of kind Forest at 1:10000 over the existing export test fixture area, OSM candidates empty, ALKIS land use empty → `.omap` written, its symbols are the ISOM set, no panic).
- [ ] **Step 2:** Run; FAIL.
- [ ] **Step 3:** Implement. Every `match` on the kind: Forest like Sprint unless named here. Every `id_for_code(code).expect(..)` in the build modules becomes a skip of the object when it returns `None` (a dropped alias); keep `expect` only where the code is defined in all three sets (contours 101/102, 521, sheet symbols). Berlin/NAS kind lists: Forest gets every layer Sprint gets.
- [ ] **Step 4:** `cargo test -p freemapper --lib` and `cargo test --workspace --exclude freemapper`; PASS.
- [ ] **Step 5:** Commit `feat(build): forest maps from the sprint pipeline, forest over parks`.

### Task 5: Contour interval from the relief

**Files:**
- Modify: `crates/freemapper/src/project.rs` (`ProjectFile.contour_interval_m: Option<f64>`, serde default, skip if `None`; validation; derive at `:2742-2750`; `project_fingerprint` counts it), `crates/freemapper/src/derive/contours.rs` (`contours_to_geojson`/`contours_from_geojson` carry `interval_m` as a top-level member), `crates/freemapper/src/project/export.rs:162,1767`, `crates/freemapper/src/build/peripheral.rs:68,208,254`

**Interfaces:**
- Produces: `pub fn contour_interval_for(kind: MapKind, heights_in_area: &[f32]) -> f64` in `derive/contours.rs`: Forest → 2.5 when `p95 - p5 < 15.0` m, else 5.0; other kinds → `kind.contour_interval_m()`. `ProjectFile::contour_interval_m` (when set) overrides it. `contours_to_geojson(lines, interval_m: f64)`, `contours_from_geojson(text) -> Result<(Vec<ContourLine>, Option<f64>), ContourError>` (missing member: `None`, the export then uses the kind's default). `PeripheralInfo.contour_interval_m: f64`, printed with one decimal only when it has a fraction, with the locale's decimal separator.

- [ ] **Step 1: Failing tests:** `flat_forest_gets_2_5_m_and_hilly_5_m` (heights 40..52 → 2.5; 40..80 → 5.0; sprint → 2.0 whatever the heights); `the_interval_is_stored_with_the_contours` (round trip with 2.5); `index_contours_every_fifth_at_2_5_m` (heights 12.5 and 25.0 are index, 15.0 is not); `peripheral_says_2_5_m` (de "Äquidistanz 2,5 m", en "contour interval 2.5 m"; 2.0 still prints "2 m"); `an_explicit_interval_wins_and_zero_is_refused` (`contour_interval_m = 5.0` in a flat forest → 5.0; `0.0` → `ProjectError` on load); `a_project_file_without_the_key_round_trips` (existing fixture text unchanged after load and save).
- [ ] **Step 2:** Run; FAIL.
- [ ] **Step 3:** Implement. The heights are the smoothed DTM cells inside the projected map area (no-data cells skipped); with no cell inside, the kind's default.
- [ ] **Step 4:** `cargo test -p freemapper --lib`; PASS.
- [ ] **Step 5:** Commit `feat(derive): contour interval from the relief, 2.5 m in flat forests`.

### Task 6: Documentation

**Files:**
- Create: `docs/adr/<next>-forest-map-kind.md`
- Modify: `docs/adr/README.md`, `docs/manual/{de,en}.md` (map kinds, scales, what a forest map holds, the quick training map), `README.md`, `README.en.md`, `AGENTS.md` (map kinds line, `--kind`), `docs/glossary.md` (Waldkarte / forest map), `CHANGELOG.md`, `CHANGELOG.en.md` (Unreleased, Added)

- [ ] **Step 1:** Write the ADR from the spec's decisions (kind and scales, symbol set and aliases, path table, forest over parks, contour rule, what is left out) and the measured Berlin results.
- [ ] **Step 2:** User texts through the `unslop` and `humanizer` skills; German says "Bundesland".
- [ ] **Step 3:** Commit `docs: the forest map kind`.

### Task 7: Check on the Berlin forests and open the PR

- [ ] **Step 1:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test -p freemapper --lib`, `cargo test --workspace --exclude freemapper`, `cargo test -p freemapper --test '*' -- --include-ignored` with `FREEMAPPER_GOLDEN_STORE=$CARGO_TARGET_DIR/golden-adapters/2`; all PASS, golden without bless.
- [ ] **Step 2:** Release build (disk check first), then for Dammheide (`relation/4513841`) and Mittelheide (`relation/11032199`): `init --kind forest --scale 10000`, `run --pdf-candidates`. Look at the PDF: paths in ISOM classes, white forest, 2.5 m contours, no 990, gardens 520; note what is wrong and fix it in the owning task's code with a test.
- [ ] **Step 3:** When core has tagged #7: replace the `[patch]` by the tag (`cargo update`), rerun Step 1. Until then the PR stays a draft.
- [ ] **Step 4:** Push `forest-map` to `github`, open the PR (body: spec, plan, results, the core dependency; ends with the attribution lines), watch its CI, give the user the URL. Tell FableWorker, freemapper-d3 and the freemapper-print session that the kind exists.
