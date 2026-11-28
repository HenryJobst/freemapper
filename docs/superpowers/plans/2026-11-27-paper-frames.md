# Paper frames Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The area page shows a draggable paper frame (A6 to A2) at the project scale; kept frames are stored and written as the hidden template map `templates/paper-frame.omap`, which freemapper-print can read.

**Architecture:** A paper model in the `freemapper` crate computes ground sizes once. `ProjectFile` stores `paper_frames` (centre in EPSG:4326). `export` turns them into closed, tagged line objects in map coordinates and writes the template; a newly written map names it, a kept map is never edited. The area page (`crates/freemapper-cli/web/`) draws and saves frames through the existing save call.

**Tech Stack:** Rust workspace (`freemapper`, `freemapper-cli`), vendored Leaflet, plain JS page. Cross-repo: freemapper-core (reserved tag words), freemapper-print (reader, own plan).

**Spec:** `docs/superpowers/specs/2026-11-27-paper-frames-design.md`

## Global Constraints

- Formats and sizes (ISO 216, mm): A2 420 × 594, A3 297 × 420, A4 210 × 297, A5 148 × 210, A6 105 × 148; landscape swaps the sides. Ground size = paper mm × scale denominator. Checks: A4 portrait 1:15000 = 3150 × 4455 m; A6 landscape 1:4000 = 592 × 420 m.
- The frame is the full sheet, no margin, axis-aligned in the map CRS, scale = project scale.
- `paper_frames` in `freemapper.toml`: `format`, `orientation` (`landscape`|`portrait`), `lon`, `lat` (EPSG:4326). Absent = none. Must not enter `project_fingerprint`.
- Tags on each frame object, plain non-localised strings: `freemapper:source=paper-frame`, `freemapper:paper=A4`, `freemapper:orientation=portrait`. Object text is the localised name only.
- Template: `templates/paper-frame.omap`, listed after the candidate layers, hidden, opacity 0.7 (`CANDIDATE_LAYER_OPACITY`). Rewritten on every export; removed (file) when no frames are stored.
- A kept map is never edited. A frame change needs `export` only (no `fetch`, no `derive`).
- All ten format × orientation combinations for every map kind. No new MCP tool, no new CLI switch.
- Source and technical docs English; user texts German and English; ADR (next free number at the time), `CHANGELOG.md` and `CHANGELOG.en.md` entries, glossary entry; user text through unslop and humanizer.

## Review Focus

- Frame partly or wholly outside the map area: valid, drawn and exported as is.
- Project in UTM 32 (west) versus 33: the rectangle is computed in the project's `crs`, not a fixed zone.
- Two identical frames kept: both stay (no silent merge); deleting one removes one.
- Older `freemapper.toml` without `paper_frames`, and a project that had frames, then none: loads, exports, file removed.
- Kept map plus new frames: the `.omap` is byte-identical afterwards; the run prints the open-in-OOM line.
- `list_layers` shows `paper-frame` only when the file exists.
- Page: scale of a new project changes while a frame is shown: frame follows; no format chosen: nothing drawn.

---

### Task 1: Reserved words in freemapper-core (other repo)

**Files:** `/Users/henryprivat/Documents/gitrepos/freemapper-core`, crate `freemapper-contract` (`tags` module, fixtures).

**Interfaces:**
- Produces: constants in `freemapper_contract::tags` for the values `paper-frame` (a `freemapper:source` value), the keys `freemapper:paper` and `freemapper:orientation`, in the style of the existing constants; a fixture project with two frames (`templates/paper-frame.omap`); a core release tag (v0.30.0 or the next free one).

- [ ] **Step 1:** In core, add the three constants and a fixture `paper-frames` whose template holds an A4 portrait and an A5 landscape frame with the tags of the Global Constraints. Extend the fixture list the gate reads.
- [ ] **Step 2:** Run core's tests and the contract gate (`contract-gate.yml` runs print's and courseplan's `contract_*` tests); both must be green. If a reader fails on the new fixture, fix the reader first (core rule: never release over a red gate).
- [ ] **Step 3:** Release core, note the tag. Message `freemapper-print-5e` with the tag, the tag names and the plan in `docs/superpowers/specs/2026-11-27-paper-frames-design.md` ("freemapper-print uses the frames").
- [ ] **Step 4:** Commit and PR in the core repo as that repo's rules say.

### Task 2: Paper model

**Files:** Create `crates/freemapper/src/paper.rs` (declare in `lib.rs`); tests in the same file.

**Interfaces:**
- Produces: `pub enum PaperFormat { A6, A5, A4, A3, A2 }` with `ALL`, `name() -> &'static str` ("A4"), `from_name(&str) -> Option<Self>`; `pub enum Orientation { Landscape, Portrait }` with `name()`/`from_name`; `pub fn size_mm(format, orientation) -> (f64, f64)` (width, height); `pub fn ground_size_m(format, orientation, scale_denominator: u32) -> (f64, f64)`. Both enums derive `Serialize, Deserialize` with the names `A6`…`A2` and `landscape`/`portrait`.

- [ ] **Step 1: Write the tests** `a4_portrait_at_15000_is_3150_by_4455_m`, `a6_landscape_at_4000_is_592_by_420_m` (assert `ground_size_m(..) == (592.0, 420.0)`), `landscape_swaps_the_sides` for every format, `names_round_trip`, `unknown_name_is_none`.
- [ ] **Step 2:** Run `cargo test -p freemapper paper::` — expect compile failure.
- [ ] **Step 3:** Implement the signatures above.
- [ ] **Step 4:** Run the tests — expect PASS.
- [ ] **Step 5:** Commit `feat(paper): paper formats and ground sizes (#343)`.

### Task 3: Frames in the project file

**Files:** Modify `crates/freemapper/src/project.rs` (`ProjectFile` at ~330, setters near `set_contour_interval` ~947), `crates/freemapper/src/project/export.rs` (`project_fingerprint` ~1106, destructuring without `..`). Tests in `project.rs`'s test module.

**Interfaces:**
- Consumes: `PaperFormat`, `Orientation` (Task 2).
- Produces: `pub struct PaperFrame { pub format: PaperFormat, pub orientation: Orientation, pub lon: f64, pub lat: f64 }` (Serialize/Deserialize/Clone/PartialEq); `ProjectFile.paper_frames: Vec<PaperFrame>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`; `pub fn set_paper_frames(dir: &Path, frames: &[PaperFrame]) -> Result<ProjectFile, ProjectError>` (same shape as `set_contour_interval`: load, `check_folders(dir, &WRITTEN_FOLDERS)`, save only on change); new `ProjectError` variant for an invalid centre (non-finite, lat outside ±90, lon outside ±180) with an English message and the German/English text entry the other variants have.

- [ ] **Step 1: Write the tests** `paper_frames_round_trip_in_the_toml`, `an_older_file_without_paper_frames_loads_empty`, `set_paper_frames_rejects_a_non_finite_centre`, `set_paper_frames_with_an_empty_list_removes_the_key`, and in the fingerprint test: `project_fingerprint_ignores_paper_frames` (equal fingerprint with and without frames).
- [ ] **Step 2:** Run `cargo test -p freemapper paper_frames` — expect FAIL (compile).
- [ ] **Step 3:** Add the field, the setter and the error variant; add `paper_frames: _` to the fingerprint destructuring (it does not compile until sorted there; that is the allow-list design).
- [ ] **Step 4:** Run the tests and `cargo clippy -p freemapper --all-targets -- -D warnings` — expect PASS.
- [ ] **Step 5:** Commit `feat(project): paper_frames in freemapper.toml (#343)`.

### Task 4: The frame layer document

**Files:** Create `crates/freemapper/src/build/paper_frames.rs` (declare in `build/mod.rs`); tests in the same file. Symbol: one helper line symbol, see below.

**Interfaces:**
- Consumes: `PaperFrame`, `ground_size_m` (Tasks 2, 3); `Georeferencing` (`crates/freemapper/src/georef.rs`) and the project CRS to project the centre; `MapObject`/`MapDocument` (`omap/model.rs`).
- Produces: `pub fn paper_frame_document(frames: &[PaperFrame], scale_denominator: u32, georef: &Georeferencing, template_base: &MapDocument) -> MapDocument` — one part named `paper frames`, one closed `Path` object per frame (4 corners, `FLAG_CLOSE`), tags per Global Constraints, a text name ("A4 portrait" in the program language; builder's call whether as object text or as a tag `name`). `template_base` supplies the symbol set and georeferencing exactly as `candidate_layers()` does (`..self.candidates.clone()`).

- [ ] **Step 1: Write the tests** `a_frame_is_a_closed_path_of_the_paper_size_on_the_map` (A4 portrait at the map's scale: bounding box 210 × 297 mm in map coordinates, tolerance 0.01 mm), `the_tags_are_plain_strings` (`freemapper:source=paper-frame`, `freemapper:paper=A4`, `freemapper:orientation=portrait`), `a_frame_beyond_the_area_is_kept`, `two_equal_frames_give_two_objects`, `utm32_and_utm33_projects_place_the_same_centre_at_the_same_lat_lon` (centre of the object's bounding box maps back to lon/lat within 1 m).
- [ ] **Step 2:** Run `cargo test -p freemapper build::paper_frames` — expect FAIL.
- [ ] **Step 3:** Implement. Helper line symbol: add one entry to the symbol set used by candidate documents if it has none suitable (colour and width are the builder's call: it must read clearly over the orthophoto and not look like map content); the symbol id must not collide with ISSprOM/ISOM numbers.
- [ ] **Step 4:** Run the tests — expect PASS.
- [ ] **Step 5:** Commit `feat(build): paper frame layer document (#343)`.

### Task 5: Export, template reference and layer list

**Files:** Modify `crates/freemapper/src/project/export.rs` (`write_starter_project` ~356, template list ~1440, `StarterDocuments` ~1282), `crates/freemapper/src/layers.rs` (~371–395), `crates/freemapper-cli/src/pipeline.rs` (run output line). Texts: the program's text tables (German and English).

**Interfaces:**
- Consumes: `paper_frame_document` (Task 4), `ProjectFile.paper_frames`.
- Produces: `PAPER_FRAME_RELPATH: &str = "templates/paper-frame.omap"`; `StarterDocuments.paper_frames: Option<MapDocument>` (None when no frames); `Layer` entry `paper-frame` in `list_layers` only when the file exists; a progress/warning line key `paper-frame-kept-map` (en: the template was written, open it in OOM via Templates, because the existing map is not changed).

- [ ] **Step 1: Write the tests** (in `project.rs` tests, using `school_project`): `export_writes_the_paper_frame_template_and_the_new_map_names_it`, `export_without_frames_writes_no_template_and_the_map_names_none`, `export_removes_a_stale_paper_frame_template`, `a_kept_map_stays_byte_identical_when_frames_are_added_and_the_file_is_written`, `list_layers_names_paper_frame_only_when_the_file_exists`, `a_frame_change_does_not_change_the_data_or_templates_other_than_its_own` (derived data files untouched).
- [ ] **Step 2:** Run `cargo test -p freemapper paper_frame` — expect FAIL.
- [ ] **Step 3:** In `write_starter_project`: write the document to `PAPER_FRAME_RELPATH` when `Some`, else `freemapper_fs::remove_inside` it. In the template list, push the reference (`TemplateKind::Map`, hidden, `CANDIDATE_LAYER_OPACITY`, `behind_map: false`) after the candidates only when frames exist. For `MapWrite::Keep` with frames: emit the `paper-frame-kept-map` line once per run. Add the layer in `layers.rs`.
- [ ] **Step 4:** Run the tests and the workspace `cargo test --workspace --locked` — expect PASS (existing golden tests unchanged, since no frames means no change).
- [ ] **Step 5:** OCAD: leave the template out of `export_ocad`; add a one-line note to the OCAD outcome `warnings` only if the project has frames. Test `ocad_export_leaves_the_paper_frame_out_and_says_so`.
- [ ] **Step 6:** Commit `feat(export): paper-frame template layer (#343)`.

### Task 6: Area page server side

**Files:** Modify `crates/freemapper-cli/src/select.rs` (project JSON ~446–490, save ~640–740, body parsing helpers), `crates/freemapper-cli/tests/select.rs`.

**Interfaces:**
- Consumes: `set_paper_frames`, `PaperFormat`, `size_mm` (Tasks 2, 3).
- Produces: project JSON gets `paper_formats: [{ "name": "A4", "width_mm": 210, "height_mm": 297 }, …]` (portrait sizes) and `paper_frames: [{ format, orientation, lon, lat }]` (empty when none, also for a new project). The save body accepts `paper_frames` (absent keeps, `[]` clears, invalid entry refused with a message via the existing `SaveError::Refused` path). A body with `paper_frames` and without `area` on an existing project saves only the frames (no `set_map_area_with_content`); the stale-revision and lock checks stay.

- [ ] **Step 1: Write the tests** in `tests/select.rs`: `the_project_answer_lists_the_paper_formats`, `saving_frames_alone_on_an_existing_project_keeps_area_kind_scale_and_title`, `an_unknown_format_is_refused`, `an_empty_list_clears_the_frames`, `a_save_without_paper_frames_keeps_them`, `frames_alone_on_a_folder_without_a_project_are_refused`.
- [ ] **Step 2:** Run `cargo test -p freemapper-cli --test select paper` — expect FAIL.
- [ ] **Step 3:** Implement per the interfaces. Parse `paper_frames` into `Vec<PaperFrame>` with serde; map errors to the page's message style.
- [ ] **Step 4:** Run the tests — expect PASS.
- [ ] **Step 5:** Commit `feat(select): paper frames in the project answer and the save (#343)`.

### Task 7: Area page client

**Files:** Modify `crates/freemapper-cli/web/index.html` (controls beside `#scale`, ~63), `web/app.js` (map ~37, scale handling ~559–600, save ~624–650, project load ~727), `web/app.css`; page texts in the German and English tables (names like the existing `page-scale`).

**Interfaces:**
- Consumes: project JSON fields of Task 6.
- Produces: controls `#paper-format` (none, A6…A2), `#paper-orientation` (landscape, portrait), button `#paper-keep`, list `#paper-frames` with a delete button per entry; a draggable frame; kept frames drawn thinner. Pure helper `frameCorners(lat, lon, widthM, heightM) -> [[lat, lon] ×4]` in `app.js`, dividing the ground size by `cos(lat)` for the web-mercator view (the stored frame stays exact).

- [ ] **Step 1:** Add markup, texts (DE/EN) and styles. No format chosen: nothing is drawn.
- [ ] **Step 2:** Implement the frame as an `L.polygon` that follows the scale selector and orientation at once; drag by mouse on the polygon (mousedown, mousemove, mouseup on `map`, with `map.dragging` disabled while dragging); start centre = map centre.
- [ ] **Step 3:** "Keep" adds `{format, orientation, lon, lat}` to a list; delete removes one; the list goes into the save body as `paper_frames`; on an existing project, changing the list saves frames alone (no `area`) and shows the revision handling the page already has.
- [ ] **Step 4: Verify by hand** (no browser tests exist): `cargo run -p freemapper-cli -- select <tmp dir with a project> --no-browser`, open the printed address. Check: A4 portrait at 1:15000 on a forest project measures about 3.15 × 4.46 km against the scale bar; scale change resizes the frame; keep, reload, frames come back; delete works. Report what was seen; if no browser is available say so instead of claiming it works.
- [ ] **Step 5:** Commit `feat(web): paper frame on the area page (#343)`.

### Task 8: Contract pin, docs, ADR, changelog

**Files:** `Cargo.toml` (core pins → tag of Task 1), `docs/manual/de.md`, `docs/manual/en.md`, `docs/glossary.md`, `docs/adr/<next free>-paper-frames.md`, `CHANGELOG.md`, `CHANGELOG.en.md`, README only if it lists the page's controls.

- [ ] **Step 1:** Move all core pins to the release of Task 1 together (they are pinned as one set); `cargo build --workspace --locked`; run the contract tests; expect PASS.
- [ ] **Step 2:** Use the tags from `freemapper_contract::tags` in Task 4's code instead of local literals; tests still pass.
- [ ] **Step 3:** Write the manual section (formats, the A4 example numbers, kept frames, the template in OOM, the kept-map hint, OCAD left out), the glossary entry (Papierrahmen / paper frame), the ADR (decision, rejected alternatives from the spec, cost if wrong), both changelog entries under Unveröffentlicht/Unreleased. Run the unslop and humanizer skills over the user texts (humanizer wins where they disagree).
- [ ] **Step 4:** Re-check README.md and README.en.md for status and tables before pushing.
- [ ] **Step 5:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`; commit `docs: paper frames (#343)`.
- [ ] **Step 6:** Open one PR for Tasks 2–8 (GitHub, bigger-PR rule), watch CI (Linux; label `all-systems` is not needed, no per-OS code), merge on green, watch `ci.yml` on main; close #343 with a comment that points to the manual section; report to `freemapper-print-5e` that the freemapper side is merged.
