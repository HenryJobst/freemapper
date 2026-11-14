# Bavaria: objects from the ATKIS Basis-DLM — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A map in Bavaria gets point and line objects of the agency's landscape model (ATKIS Basis-DLM) as candidates.

**Architecture:** The Basis-DLM WFS answers in the AAA schema the NAS reader already reads. A NAS layer and a NAS service get a model (`Alkis`, `BasisDlm`); the DLM layers are eight new rows of the layer table, drawn by rules in `state-candidates.toml`. Their source is `atkis-dlm/<name>` with a new lowest rank, and a new duplicate step drops a DLM object that another source has.

**Tech Stack:** Rust workspace; crates `freemapper-sources`, `freemapper-adapter-api`, `freemapper-manifest`, `freemapper` (engine), `freemapper-adapters-de`.

**Spec:** `docs/superpowers/specs/2026-11-14-bayern-atkis-design.md`

## Global Constraints

- Service, exactly: `https://geoservices.bayern.de/wfs/v1/ogc_atkis_basisdlm.cgi`; `namespaces = "xmlns(adv,http://www.adv-online.de/namespaces/adv/gid/7.1)"`; `max_count = 100000`; `max_bytes = 268435456`; `model = "basis-dlm"`. Manifest revision 3. No new host.
- Layer files, exactly: `dlm-bauwerk`, `dlm-vegetation`, `dlm-felsen`, `dlm-hoehle`, `dlm-gewaessermerkmal`, `dlm-damm`, `dlm-turm`, `dlm-gewaesserachse`. Stored in `data/alkis-nas/`, recorded in `nas_layers`.
- Source of a DLM object: `atkis-dlm/<file without "dlm-">`. Rank `Landscape`, below `Osm`.
- Duplicate distances: a point within 15 m, a line with 80 % of its length within 10 m, of an object of a higher rank with the same symbol.
- Symbols: rock 205; cave 203 (sprint) and 203.2 (forest); spring 312; dam 105; tower 1003 → 525, towers 1001, 1004, 1005, 1006, 1007, 1008, 1010, 1011 → 524; watercourse 305; `funktion` 8300 is a canal by the code list (read during the build) and is left out. Towers 1002, 1009, 1012 are left out.
- No map outside Bavaria changes; a commit that changes a golden file carries `Golden-Bless: <reason>`.
- `build/cliffs.rs` is not touched.
- Source code and technical docs English; user docs German and English; German says "Bundesland".
- Disk is short: `CARGO_INCREMENTAL=0`, `cargo clean` after a build.

## Review Focus

1. A DLM object beside an OSM object of the same symbol at 12 m (a point) and at 20 m: the first goes, the second stays.
2. A DLM line that follows an OSM line for half its length stays whole; one that follows it for 90 % goes.
3. A service of model `basis-dlm` is never asked for an ALKIS layer, and Brandenburg and Bremen never for a DLM layer.
4. Under `balanced` a DLM fountain stays a candidate while an OSM fountain goes onto the map; under `full` both go.
5. An answer with `numberMatched="unknown"` and fewer objects than `max_count` is complete (no split).

---

### Task 1: The rank `Landscape` and the duplicate step

**Files:** Modify `crates/freemapper/src/build/dedup.rs`, `crates/freemapper/src/build/assembly.rs`.

**Interfaces:**
- Produces: `Rank::Landscape` (first, lowest variant); `rank()` gives it for a source that starts with `atkis-dlm/`; `pub fn drop_landscape_near_others(landscape: Vec<MapObject>, others: &mut [MapObject], symbol_set: &SymbolSet, scale_denominator: u32) -> Vec<MapObject>` returns the DLM objects that stay; an object in `others` that made one go names it in `freemapper:also` as `<source>/<source_id>`.

- [ ] **Step 1: Failing tests** in `dedup.rs`: `a_landscape_source_ranks_below_osm`; `a_landscape_point_goes_when_another_source_has_it` (an OSM 311 at 12 m: gone and named in `freemapper:also`; at 20 m: stays; an OSM 526 at 5 m, another symbol: stays); `a_landscape_line_goes_when_it_follows_another_line` (518: 90 % within 10 m of an OSM 518 goes, 50 % stays); `landscape_objects_never_win` (an OSM line beside a DLM line of the same symbol survives `drop_osm_lines_near_cadastre`).
- [ ] **Step 2: Run** `cargo test -p freemapper --lib dedup` — Expected: does not compile.
- [ ] **Step 3: Implement**; in `assembly.rs` split the DLM objects off the cadastre candidates before the existing rules and run the new step after them, against the cadastre and OSM candidates that are left.
- [ ] **Step 4: Run** `cargo test -p freemapper --lib` — Expected: PASS.
- [ ] **Step 5: Commit** `feat(dedup): a landscape model ranks below OpenStreetMap and never doubles it`.

### Task 2: A NAS layer and a NAS service name their model

**Files:** Modify `crates/freemapper-sources/src/nas.rs`, `crates/freemapper-adapter-api/src/extras.rs`, `crates/freemapper-manifest/src/{schema,model,validate,extras}.rs`, `crates/freemapper/src/build/state.rs` and `build/stairs.rs` (the source of a layer), the notes that say "ALKIS NAS", `docs/adapters/manifest-v1.en.md` and its German twin if there is one.

**Interfaces:**
- Produces: `pub enum Model { Alkis, BasisDlm }` (with `label()`: `"ALKIS NAS"`, `"ATKIS Basis-DLM"`); `NasLayer::model`; `NasService::model`; the eight DLM rows in the layer table; `pub fn layer_source(file: &str) -> String` (`alkis-nas/<file>` or `atkis-dlm/<name>`); manifest key `model = "alkis" | "basis-dlm"` (default `alkis`).

- [ ] **Step 1: Failing tests**: in `nas.rs` `a_service_is_asked_only_for_the_layers_of_its_model` (a DLM service gets eight requests, none for `AX_Strukturlinie3D`; an ALKIS service none for `AX_Turm`) and `the_source_of_a_layer_names_its_model`; in the manifest crate `the_model_of_a_nas_service_is_read_and_checked` (`basis-dlm` accepted, `dlm` refused with a finding that names the key, default `alkis`); in `nas.rs` `an_answer_of_the_basis_dlm_is_read` with the fixture `tests/fixtures/bayern-dlm-turm.gml` (the real answer for the Schmausenbuck: one point, `bauwerksfunktion` 1003, the name, `beginnt`).
- [ ] **Step 2: Run** the three crates' tests — Expected: do not compile.
- [ ] **Step 3: Implement.** The progress and warning notes of the NAS family take the model's label as an argument.
- [ ] **Step 4: Run** `cargo test -p freemapper-sources -p freemapper-manifest -p freemapper-adapter-api` and `cargo test -p freemapper --lib` — Expected: PASS.
- [ ] **Step 5: Commit** `feat(nas): a layer and a service name their model, ALKIS or the Basis-DLM`.

### Task 3: The rules and labels of the DLM types

**Files:** Modify `assets/rules/state-candidates.toml`, `assets/rules/adv-codes.toml`; tests in `crates/freemapper/src/build/state.rs`; fixtures `tests/fixtures/bayern-dlm-{hoehle,felsen,gewaessermerkmal,damm,gewaesserachse,bauwerk}.gml` (real answers, cut to a few objects) and a row each in `tests/fixtures/README.md`.

- [ ] **Step 1: Failing test** `the_basis_dlm_objects_get_their_symbols`: the fixtures through `parse_nas_gml` and `nas_objects` give, for a sprint and for a forest map: tower 1003 → 525 and 1002 → nothing; cave → 203 / 203.2; rock → 205; spring → 312; dam → 105; watercourse → 305; fence 1740 → 518; fountain 1780 → 311; each with `freemapper:source=atkis-dlm/<name>`, `freemapper:source_id`, `freemapper:currency` and a `freemapper:alkis` label.
- [ ] **Step 2: Run** — Expected: FAIL (no rule, no symbol).
- [ ] **Step 3: Write** the rules and the labels.
- [ ] **Step 4: Run** `cargo test -p freemapper --lib` — Expected: PASS.
- [ ] **Step 5: Commit** `feat(rules): the objects of the Basis-DLM and their symbols`.

### Task 4: Bavaria asks the service

**Files:** Modify `crates/adapters/de/manifests/bayern/manifest.toml`; tests that list the states with a NAS service (`crates/freemapper/src/adapters/mod.rs`, `crates/adapters/de/tests/manifests.rs`, CLI tests of `adapters list`), the estimate tests.

- [ ] **Step 1: Failing test**: the test that says only Brandenburg and Bremen have a NAS service expects Bavaria too, with model `BasisDlm`; a test in `crates/adapters/de` that the Bavarian service builds the request URL the research used (type `adv:AX_Turm`, the `NAMESPACES` value, the box).
- [ ] **Step 2: Run** — Expected: FAIL.
- [ ] **Step 3: Add** the `[[extra.nas]]` block, revision 3.
- [ ] **Step 4: Run** `cargo test --workspace --locked --no-fail-fast`, clippy, fmt, `scripts/golden.sh test` — Expected: PASS; bless golden files only where they hold Bavaria's manifest data.
- [ ] **Step 5: Commit** `feat(bayern): objects from the ATKIS Basis-DLM`.

### Task 5: Live run and the examples

- [ ] **Step 1:** Build the release program; run both Bavarian examples online (`--force --pdf --pdf-candidates --ocad --lang en`). Expected: no warning; `nas_layers` in both project files with eight `dlm-…` records; at the Schmausenbuck a tower 525 and a fountain 311 from `atkis-dlm`, in Regensburg fountains and monuments.
- [ ] **Step 2:** Count the DLM objects by symbol and how many the duplicate step dropped; look at the candidates PDF of both. A second run `--offline --force` writes the same files.
- [ ] **Step 3:** Finish the example files (previews, no `.bak`, no map-only PDF), `cargo test -p freemapper --test adapters_golden`.
- [ ] **Step 4: Commit** `docs(examples): the Bavarian examples with the objects of the Basis-DLM`.

### Task 6: Texts

**Files:** Create `docs/adr/0133-bavaria-objects-from-the-basis-dlm.md`; modify `docs/adr/README.md`, `docs/manual/{de,en}.md`, `README.md`, `README.en.md`, `docs/adapters/{en,de}.md`, `docs/glossary.md`, `examples/README.md`, `examples/README.en.md`, `CHANGELOG.md`, `CHANGELOG.en.md`, `docs/regions/bayern-reference.md` (the live run).

- [ ] **Step 1:** Write the texts with the numbers of the live run. Run `unslop` over the user texts.
- [ ] **Step 2:** All checks again.
- [ ] **Step 3: Commit** `docs: Bavaria's Basis-DLM objects in the manual, the READMEs and the changelog (ADR 0133)`.
