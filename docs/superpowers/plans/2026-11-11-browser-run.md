# Create the map from the browser page: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper select` creates the whole map project from the browser: options, run, progress, cancel, result.

**Architecture:** First move the select server onto the core crate `freemapper-web`. Then extract one `pipeline::run` and a shared `JobHub` from `main.rs` and `mcp.rs`, add cooperative cancel and a folder lock, add four routes, and build the options, progress and result panels in a new `web/run.js`.

**Tech Stack:** Rust 2024, axum 0.8, `freemapper-web` v0.11.0, `freemapper-fs` v0.5.1, rust-embed, plain JavaScript, `ureq` in tests.

**Spec:** `docs/superpowers/specs/2026-11-11-browser-run-design.md`. ADR: `docs/adr/0090-create-the-map-from-the-browser-page.md`.

## Global Constraints

- Source code, identifiers and comments in English. Page and program texts in German and English; German says "Bundesland" for states, "Land" only for countries.
- No em or en dashes in new prose or program text.
- No `unsafe` (workspace lint). No path, URL or command from a request. The run uses only the folder the selector started with.
- Every file write goes through `freemapper-fs` (`write_inside`, `create_new_file`).
- No request, header or log line contains an e-mail address or other personal data.
- `Selector::start(dir, defaults, cache, port, ...)` stays callable by the MCP server; MCP tool output text stays byte for byte.
- One PR per task unless noted; CI (3 operating systems) green before the next stage. `cargo clean` after merges (disk space).
- Changelog entry (German and English) and manual update (`docs/manual/de.md`, `en.md`) land with the behaviour change in stage 5; ADR 0090 turns Accepted in the last task.
- Allowed scales per kind come from `MapKind::allowed_scales` (ADR 0085); never copy the table into new code.

## Review Focus

1. A POST that omits `Origin`, uses `text/plain`, or comes with `Sec-Fetch-Site: cross-site` must get 403 on every new route (task 1, 7).
2. A second run while one runs, from the page, from MCP and from a terminal, must be refused with a message that says who holds it (tasks 3, 5, 6, 7).
3. Cancel during fetch and during derive must leave no `.part` file and a project that the next `run` completes (task 4).
4. A stale lock file, a lock that is a symlink, and a lock left by a panic must not block or escape the folder (task 5).
5. `force` with an existing map: the old map is kept as `.omap.bak`, and the page never sends `force: true` without the dialog (tasks 7, 9).

---

## Stage 1: move onto the shared web crate

### Task 1: `select` on `freemapper-web`

**Files:**
- Modify: `Cargo.toml` (workspace dependency `freemapper-web`, git tag `v0.11.0`), `crates/freemapper-cli/Cargo.toml`, `crates/freemapper-cli/src/select.rs`
- Test: `crates/freemapper-cli/src/select.rs` (module `tests`)

**Interfaces:**
- Consumes: `freemapper_web::{serve, ServeOptions, LocalServer, embedded, open_browser}`.
- Produces: `Selector::start(dir, defaults, cache, port, busy)` unchanged; `Selector.url` unchanged. `AppState` loses `allowed_hosts`, `allowed_origins`, `done`; keeps `dir`, `defaults`, `cache`, `busy`, `saving`, `tiles`. New const `MAX_BODY_SMALL: usize = 4096` for later tasks.

- [ ] **Step 1: Write the failing tests** `post_without_origin_is_refused`, `post_with_text_plain_is_refused`, `cross_site_fetch_metadata_is_refused`, `responses_carry_nosniff`. Each POSTs `api/done`-like routes with `ureq`; assert 403 (and `x-content-type-options: nosniff` on a GET of `api/project`).
- [ ] **Step 2: Run** `cargo test -p freemapper-cli select::` and see the four fail.
- [ ] **Step 3: Implement** `Selector::start_with_tiles` through `freemapper_web::serve(|prefix, done| router(prefix, state, done), ServeOptions { port, csp: CONTENT_SECURITY_POLICY, strict_post: true, max_body: 1 << 20 })`. `router` drops the middleware; `index` and `asset` use `freemapper_web::embedded::<Assets>`. `done` calls `notify_one()` on the crate's `Notify`. `Selector::finished` awaits `LocalServer::finished`. `open_browser` re-exported from the crate.
- [ ] **Step 4: Update existing tests** (`save_is_refused_while_a_pipeline_runs`, `get`) to send `Origin: http://127.0.0.1:<port>` on POST. Run `cargo test -p freemapper-cli` and `cargo tree -d | rtk proxy grep freemapper` (record duplicates in the PR).
- [ ] **Step 5: Commit** `refactor(select): serve the page with freemapper-web (strict POST)`.

Size: M. Outcome: same page, guards from core, no feature change.

---

## Stage 2: one pipeline, one job table

### Task 2: `pipeline::run` and `RunOptions`

**Files:**
- Create: `crates/freemapper-cli/src/pipeline.rs`
- Modify: `crates/freemapper-cli/src/main.rs` (`mod pipeline;`, `Commands::Run` uses it), `crates/freemapper-cli/src/mcp.rs` (`summary`, `ocad_summary` move into `pipeline.rs`)

**Interfaces:**
- Produces:
  - `pub struct RunOptions { pub force: bool, pub offline: bool, pub ocad: bool, pub pdf: bool, pub pdf_candidates: bool, pub pdf_scale_bar: bool, pub pdf_sources: PdfSources }`
  - `pub struct RunRequest` (serde, `deny_unknown_fields`, all `Option<bool>` and `pdf_sources: Option<String>`)
  - `pub fn parse(request: RunRequest) -> Result<RunOptions, String>` (text for a bad `pdf_sources`: `pdf_sources must be auto, full or credit-only, not "x"`)
  - `pub enum Step { Fetch, Derive, Map, Ocad, Pdf, PdfCandidates }` with `fn label_key(self) -> &'static str`
  - `pub enum Event<'a> { Phase { step: Step, index: u8, total: u8 }, Line(&'a str) }`
  - `pub struct ResultFile { pub kind: &'static str, pub relative_path: String, pub bytes: u64 }`
  - `pub struct RunReport { pub summary: String, pub warnings: Vec<String>, pub files: Vec<ResultFile>, pub kept_map: bool }`
  - `pub fn run(dir: &Path, options: &RunOptions, cache: &Cache, fetcher: &dyn Fetcher, control: &RunControl, progress: &mut dyn FnMut(Event)) -> Result<RunReport, String>`
  - `pub struct RunControl { cancelled: Arc<AtomicBool> }` with `cancel()`, `is_cancelled()` (used from task 4; here it is never set)
- Consumes: `run_project`, `export_ocad`, `export_pdf`, `export_candidates_pdf`, `map_path`, `pdf_path`, `OCAD_DIR`.

- [ ] **Step 1: Write tests** in `pipeline.rs`: `parse_defaults_match_the_cli`, `parse_refuses_unknown_pdf_sources`, `parse_refuses_unknown_fields`, `total_counts_only_selected_steps` (pdf+ocad gives total 5: fetch, derive, map, ocad, pdf), `summary_text_is_unchanged` (golden string copied from today's `mcp.rs` output for a stub run), `files_lists_only_fixed_names`.
- [ ] **Step 2: Run** `cargo test -p freemapper-cli pipeline::` and see failure.
- [ ] **Step 3: Implement** the types and `run`: emit `Phase` before calling each engine step; engine lines become `Event::Line`. Warnings are lines with the prefix `warning:`. OCAD and PDF failures are appended to `summary` and do not fail the run (today's MCP rule). `files` checks existence of each fixed name and reads its size.
- [ ] **Step 4: Switch `main.rs` `Run`** to `pipeline::run` with a stderr progress printer that prints `Line` as today and ignores `Phase`; keep the `println!` result lines by reading `RunReport`. Run the CLI tests and `cargo test -p freemapper-cli`.
- [ ] **Step 5: Commit** `refactor(cli): one pipeline function for run`.

Size: L. Outcome: CLI `run` unchanged for users, one sequence in code.

### Task 3: `JobHub` shared by MCP and pages

**Files:**
- Create: `crates/freemapper-cli/src/jobs.rs`
- Modify: `crates/freemapper-cli/src/mcp.rs` (remove `Jobs`, `run_job`, `JOB_LINES`; use hub), `crates/freemapper-cli/src/select.rs` (`Busy` replaced by `Arc<JobHub>`), `crates/freemapper-cli/src/main.rs`

**Interfaces:**
- Produces:
  - `pub struct JobHub` (`Default`), `pub type JobId = String` (`job-N`)
  - `pub enum JobState { Running, Finished, Failed, Cancelled }`
  - `pub struct JobStatus { job_id, dir, state, phase: Option<(Step, u8, u8)>, lines: Vec<String>, next: usize, warnings: Vec<String>, report: Option<RunReport>, error: Option<String> }` with `fn to_json(&self) -> serde_json::Value` (MCP shape unchanged: `job_id, dir, state, lines, warnings, result, error`; `result` is `report.summary`; new keys `phase`, `next`)
  - `impl JobHub { pub fn start(&self, dir: &Path, runner: Box<dyn FnOnce(&RunControl, &mut dyn FnMut(Event)) -> Result<RunReport, String> + Send>) -> Result<JobId, String>; pub fn status(&self, id: &str, since: usize) -> Option<JobStatus>; pub fn current(&self) -> Option<JobStatus>; pub fn cancel(&self, id: &str) -> bool; pub fn running_for(&self, dir: &Path) -> bool }`
  - `start` spawns a blocking thread via `tokio::task::spawn_blocking`, catches panics (job fails with `internal error: ...`), caps 200 lines of 500 characters, refuses with today's text when any job runs.
- `Selector::start(dir, defaults, cache, port, hub: Option<Arc<JobHub>>)`; `save` calls `hub.running_for(&state.dir)`.

- [ ] **Step 1: Move and adapt the existing tests** from `mcp.rs` (`only_one_pipeline_runs_at_a_time`, `panicking_job_is_reported_as_failed`, `job_progress_and_outcome_are_recorded`, `job_keeps_the_last_lines_and_the_warnings`) into `jobs.rs`; add `status_since_returns_only_new_lines`, `cancel_sets_the_flag_and_state_becomes_cancelled`, `line_longer_than_500_characters_is_cut`.
- [ ] **Step 2: Run** and see them fail to compile.
- [ ] **Step 3: Implement** `jobs.rs`; wire `mcp.rs` (`FreemapperServer.jobs: Arc<JobHub>`, selectors get the same `Arc`) and `main.rs select` (own hub).
- [ ] **Step 4: Run** `cargo test -p freemapper-cli` and the MCP smoke test; confirm MCP tool texts did not change.
- [ ] **Step 5: Commit** `refactor(mcp): job hub shared with the selection page`.

Size: L. Risk: the largest diff in `mcp.rs`; keep the old tests as the safety net.

---

## Stage 3: cancel and lock

### Task 4: cooperative cancel

**Files:**
- Modify: `crates/freemapper/src/project.rs` (`ProjectError::Cancelled`, `run_project_cancellable`), `crates/freemapper/src/fetch.rs` only if the tile loop lives there, `crates/freemapper-cli/src/pipeline.rs` (`CancellingFetcher`)
- Test: `crates/freemapper/src/project.rs` (module `tests`), `crates/freemapper-cli/src/pipeline.rs`

**Interfaces:**
- Produces: `pub fn run_project_cancellable(dir, fetcher, cache, mode, force, progress, cancelled: &dyn Fn() -> bool) -> Result<RunOutcome, ProjectError>`; `run_project` calls it with `&|| false`. `fetch_project` and `derive_project` gain a private `_cancellable` twin used by it. `ProjectError::Cancelled` displays `cancelled`. `pipeline::CancellingFetcher<'a> { inner: &'a dyn Fetcher, control: &'a RunControl }` implements `Fetcher`.
- Consumes: `RunControl` (task 2).

- [ ] **Step 1: Write tests** `cancel_before_fetch_returns_cancelled_and_writes_nothing`, `cancel_between_fetch_and_derive_leaves_no_part_files`, `cancelled_project_runs_to_the_end_the_next_time` (stub fetcher, fixture project), `cancelling_fetcher_refuses_the_next_request`.
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** checks before each phase and at each tile boundary of fetch and derive. A cancelled error is not retried and is not reported as a failure; `pipeline::run` maps it to `Err("cancelled")` and the hub sets `JobState::Cancelled`.
- [ ] **Step 4: Run** `cargo test -p freemapper -p freemapper-cli`.
- [ ] **Step 5: Commit** `feat(engine): cancel a run between steps and tiles`.

Size: M. Outcome: `JobHub::cancel` stops a run at the next boundary.

### Task 5: folder lock

**Files:**
- Create: `crates/freemapper-cli/src/lock.rs`
- Modify: `crates/freemapper-cli/src/main.rs` (take the lock in `Run`, `Fetch`, `Derive`, `Export`, and `select` saves), `crates/freemapper-cli/src/jobs.rs` (lock held while a job runs), `crates/freemapper-cli/src/select.rs` (`save` takes the lock)

**Interfaces:**
- Produces:
  - `pub const LOCK_NAME: &str = ".freemapper-run.lock"`
  - `pub struct FolderLock` (guard; heartbeat thread every 10 s; removes the file on drop)
  - `pub fn acquire(dir: &Path, who: Who) -> Result<FolderLock, LockError>` with `pub enum Who { Cli, Mcp, Page }`
  - `LockError::Held { who: String, since_seconds: u64 }` displays `another freemapper run is using this folder (started 14:02 by the terminal). Wait for it or stop it first.`; `LockError::Io(String)`
  - stale after `STALE_AFTER: Duration = Duration::from_secs(60)` by file modification time
- Consumes: `freemapper_fs::{create_new_file, write_inside, remove_inside}`.

- [ ] **Step 1: Write tests** `second_acquire_is_refused_with_who_and_time`, `stale_lock_is_taken_over` (set modified time 61 s back with `File::set_modified`), `lock_that_is_a_symlink_is_not_followed` (unix only; target file unchanged), `drop_removes_the_file`, `panic_in_the_holder_releases_the_lock`, `heartbeat_keeps_the_lock_fresh` (interval injected as a parameter of an inner fn, 50 ms).
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** `acquire`: `create_new_file`; on `AlreadyExists` read the JSON, check the age, take over by removing and creating again, else `Held`. Content is `{pid, started, who}` and nothing else (no paths, no user name).
- [ ] **Step 4: Wire** the commands and the hub; the CLI prints the `Held` text and exits 1. Add `.freemapper-run.lock` to the ignore list in the project template if one exists.
- [ ] **Step 5: Run** `cargo test -p freemapper-cli`, including on the Windows and macOS CI jobs.
- [ ] **Step 6: Commit** `feat(cli): one run per project folder across processes`.

Size: M. Outcome: ADR 0009's "no lock" gap closed for runs and saves.

---

## Stage 4: server routes

### Task 6: run defaults and `api/project`

**Files:**
- Modify: `crates/freemapper-cli/src/select.rs` (`Defaults.run: RunOptions`, `api/project` adds `run_defaults`, `allowed_scales`, `lang: null`, `running`), `crates/freemapper-cli/src/main.rs` (`Select` gets `--ocad --pdf --pdf-candidates --pdf-scale-bar --pdf-sources --offline`), `crates/freemapper-cli/src/mcp.rs` (`OpenAreaSelector` gets the same optional fields)

**Interfaces:**
- Produces: `Defaults { kind, scale, title, from_agent, run: RunOptions }` (`force` is always false there); `GET api/project` keys `run_defaults` (object with the seven option names), `allowed_scales` (`{"schoolyard":[1000],"sprint":[4000,5000]}` from `MapKind::allowed_scales`), `lang` (`null`), `running` (`JobStatus` JSON or `null`).

- [ ] **Step 1: Write tests** `project_answer_has_run_defaults`, `allowed_scales_come_from_the_engine`, `running_is_null_without_a_job`.
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** the fields and flags; `--pdf-scale-bar` or `--pdf-sources` without a PDF flag is accepted (the page disables them), as in `run`.
- [ ] **Step 4: Run** `cargo test -p freemapper-cli`.
- [ ] **Step 5: Commit** `feat(select): run defaults in the project answer`.

Size: S.

### Task 7: `api/run`, `api/status`, `api/cancel`, `api/open-folder`

**Files:**
- Modify: `crates/freemapper-cli/src/select.rs`
- Create: `crates/freemapper-cli/src/select_run.rs` (handlers; keeps `select.rs` from growing)
- Test: `crates/freemapper-cli/src/select_run.rs`

**Interfaces:**
- Consumes: `pipeline::{RunRequest, parse, run}`, `JobHub`, `lock::acquire`, `estimate_json`, `freemapper_web` guards.
- Produces:
  - `pub trait PipelineRunner: Send + Sync { fn run(&self, dir: &Path, options: &RunOptions, cache: &Cache, control: &RunControl, progress: &mut dyn FnMut(Event)) -> Result<RunReport, String>; }`, real one `RealRunner` (uses `HttpFetcher` wrapped in `CancellingFetcher`), `Selector::start_with_runner` for tests.
  - Routes as in the spec 3.6. `api/run` is 202 `{"job_id": "job-1"}`; 400 bad option; 404 `save the area first`; 409 job or lock held; 422 `{"error", "disk"}` when the pre-flight `stops`.
  - `api/status?job=&since=` returns `JobStatus::to_json()`; more than one request per 200 ms gets 429.
  - `api/open-folder` runs the OS opener with `state.dir` made absolute; `{"opened": bool, "dir": "<path>"}`.
  - Body limit `MAX_BODY_SMALL` on the three POST routes.

- [ ] **Step 1: Write tests** (stub runner that emits two phases and finishes on a channel signal): `run_starts_a_job_and_status_follows_it`, `run_refuses_unknown_fields`, `run_refuses_a_bad_pdf_sources`, `run_without_a_saved_project_is_404`, `second_run_is_409_with_the_job_id`, `run_is_422_when_the_disk_stops_the_fetch` (cache with `FixedSpace`), `status_cursor_returns_only_new_lines`, `status_faster_than_200ms_is_429`, `cancel_ends_in_cancelled`, `run_post_without_origin_is_403`, `run_with_force_keeps_the_old_map_as_bak` (real pipeline on the fixture project), `run_answer_never_contains_the_token`, `lock_held_by_the_cli_is_409`, `open_folder_takes_no_input` (body with `dir` field is ignored and the stub opener sees `state.dir`).
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** the handlers; `api/run` order: parse options, load project (404), take the folder lock, estimate and check `stops` (422), `hub.start(...)` (409), return 202. The lock guard moves into the job closure and drops at its end.
- [ ] **Step 4: Run** `cargo test -p freemapper-cli`.
- [ ] **Step 5: Commit** `feat(select): start, follow and cancel a run over HTTP`.

Size: L. Outcome: the whole flow works with `curl`.

---

## Stage 5: the page

### Task 8: options panel and run button

**Files:**
- Create: `crates/freemapper-cli/web/run.js`
- Modify: `crates/freemapper-cli/web/index.html` (section `#run`, script tag after `app.js`), `crates/freemapper-cli/web/app.css`, `crates/freemapper-cli/web/app.js` (expose `window.freemapperPage = { getProject, ensureSaved, onAreaChange }`)
- Test: `crates/freemapper-cli/src/select.rs` (embedded-asset tests)

**Interfaces:**
- Produces in `run.js` (pure, exported on `window.freemapperRun` for tests): `optionsFromForm(form) -> {force, offline, ocad, pdf, pdf_candidates, pdf_scale_bar, pdf_sources}`, `validateOptions(options) -> string[]` (scale bar and sources only with a PDF), `RUN_TEXTS = { de: {...}, en: {...} }`.
- Page rules per the spec 3.8 table: defaults from `project.run_defaults`, force checkbox only when `project.has_map`, kind and scale read-only for an existing project.

- [ ] **Step 1: Write tests** `run_texts_have_the_same_keys_in_both_languages` (parse the embedded `run.js` text for the key lists), `run_js_names_no_path_field` (no `dir`, `path` or `folder` key in the posted body builder), `index_has_labels_for_every_option` (each `input` and `select` in `#run` has a `<label>`), `german_text_says_bundesland_not_land_for_states`.
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** the panel: seven options, help texts DE and EN from the spec table, disabled states with an adjacent hint, button "Karte erzeugen" / "Create map" disabled until `preview` is valid and `!disk.stops`. Pressing it calls `ensureSaved()` then `POST api/run`; error text goes to a `role="alert"` region.
- [ ] **Step 4: Run** `cargo test -p freemapper-cli` and check the page by hand with `freemapper select` on a temp folder (note in the PR).
- [ ] **Step 5: Commit** `feat(select): options panel and run button`.

Size: M.

### Task 9: progress, cancel, result, force dialog

**Files:**
- Modify: `crates/freemapper-cli/web/run.js`, `index.html`, `app.css`

**Interfaces:**
- Consumes: `GET api/status`, `POST api/cancel`, `POST api/open-folder`; `JobStatus` JSON from task 3.
- Produces: `pollStatus(jobId)` (1 s, 3 s after 5 minutes, stops when not `running`), `renderProgress(status)`, `renderResult(status)`, `confirmForce() -> Promise<boolean>` using `<dialog>`; `STEP_NAMES` DE and EN for the six steps.

- [ ] **Step 1: Write tests** `step_names_exist_for_all_steps_in_both_languages` (against `Step` names listed in a Rust constant shared by test), `no_percent_is_rendered` (the progress element is `<progress>` without a `value` attribute set by script), `dialog_default_focus_is_cancel`.
- [ ] **Step 2: Run** and see failure.
- [ ] **Step 3: Implement** the progress panel (`aria-live="polite"`, last 8 lines, warnings list kept after the end), the cancel button states, the result panel (`role="region"`, focus to its heading, file list with sizes, "Ordner öffnen" with the path as text when it fails, retry button), the force dialog. Reload during a run resumes polling from `project.running`. Use only `textContent`.
- [ ] **Step 4: Run** `cargo test -p freemapper-cli`; manual run against a small fixture area including cancel and a forced rebuild.
- [ ] **Step 5: Commit** `feat(select): follow, cancel and finish a run on the page`.

Size: L.

### Task 10: pre-flight, texts and documentation

**Files:**
- Modify: `crates/freemapper-cli/web/app.js` (wording "Bundesländer" in `outside`, enabling rule for the run button), `docs/manual/de.md`, `docs/manual/en.md`, `README.md`, `README.en.md`, `CHANGELOG.md` (German and English entries), `docs/adr/0090-create-the-map-from-the-browser-page.md` (Status Accepted), `docs/adr/README.md` (Accepted), `docs/adr/0009-area-selection-in-a-local-browser-page.md` (status line "Amended by 0090")

**Interfaces:**
- Consumes: everything above. Produces nothing code-wise.

- [ ] **Step 1: Write the test** `outside_text_uses_bundesland` (embedded `app.js` contains "Bundesländer" and not "unterstützten Ländern").
- [ ] **Step 2: Run** and see failure, then edit the text.
- [ ] **Step 3: Update** the manual sections "Gebietsauswahl" and the flow chapters (page now creates the map; terminal optional), README status, the changelog entry for the release. Run `unslop` and `humanizer` over the new prose; no dashes.
- [ ] **Step 4: Re-check** README.md and README.en.md before push (status, version, tables).
- [ ] **Step 5: Commit** `docs: create the map from the page (manual, changelog, ADR 0090 accepted)`.

Size: M.

### Task 11: optional browser smoke test (not in CI)

**Files:**
- Create: `scripts/smoke-select-run.sh` (calls `playwright-cli`)

**Interfaces:**
- Consumes: `freemapper select --no-browser` printing the URL, a fixture area GeoJSON, the stub runner env `FREEMAPPER_TEST_STUB_RUN=1` read only in `cfg(test)` builds or a feature `test-stub` (the release binary must not read it).

- [ ] **Step 1: Write the script**: open the URL, search no place, load the fixture area through the saved project, tick PDF, press "Karte erzeugen", wait for "Fertig", cancel in a second run, assert the result text.
- [ ] **Step 2: Run it locally** and note the result in the PR. Skip when `playwright-cli` is missing.
- [ ] **Step 3: Commit** `test(select): browser smoke script for the run flow`.

Size: S. Skip if time is short; nothing else depends on it.

---

## Self-review

- Spec coverage: 3.1 task 1; 3.2 task 2; 3.3 task 3; 3.4 task 4; 3.5 task 5; 3.6 tasks 6, 7; 3.7 task 7 and 9; 3.8 tasks 8, 9, 10; 3.9 task 6; 3.10 tasks 1, 5, 7; 3.11 stays out; tests section spread over all tasks; accessibility in tasks 8, 9.
- Types: `RunOptions`, `RunReport`, `RunControl`, `Event`, `Step`, `JobHub`, `JobStatus`, `FolderLock` are defined once (tasks 2, 3, 5) and used under those names later. `Busy` is removed in task 3 and not used afterwards.
- Count: 11 tasks (S: 6 and 11, M: 1, 4, 5, 8, 10, L: 2, 3, 7, 9), about 5 to 7 days of agent work; stage 1 and stage 2 can ship alone as refactors with no user-visible change.
- Order: 1, then 2, 3 (sequential), 4 and 5 in parallel, 6, 7, 8, 9, 10.
