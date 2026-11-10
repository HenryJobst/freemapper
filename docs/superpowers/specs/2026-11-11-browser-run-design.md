# Create the map from the browser page: design

Date: 2026-11-11
Status: Proposed (decided without the user; see the rulings)
ADR: [0090](../../adr/0090-create-the-map-from-the-browser-page.md)

## 1. Intent

The user asked: "Why can I not create the project from the web UI instead of only picking the area (with all
options)?" and noted that part of the options already exist (map kind, scale).

Today `freemapper select` is a dead end. The user draws or picks an area, presses Save, presses Done, goes back to
a terminal and types `freemapper run <dir> --pdf --ocad ...`. A teacher who never opens a terminal cannot finish.
The page should take the whole path: area, options, run, result.

### Goals

- Everything `freemapper run` and the MCP tool `start_pipeline` offer is reachable on the page, with the same
  validation and the same defaults.
- A "Karte erzeugen" / "Create map" button runs the same pipeline as MCP `start_pipeline`, shows progress in plain
  language, can be cancelled, and ends in a result panel (files, folder, warnings).
- Pre-flight in the page: free disk space, area problems, existing map.
- One run at a time, also across processes (closes the gap noted in ADR 0009).
- The page never names a path. The security rules of the page do not weaken.
- First move `select` onto the shared crate `freemapper-web` (core v0.11.0), then build the feature.

### Non-goals

- Adapters, `--cache-dir`, `--port`, `--part`, GeoJSON files, the separate `fetch`, `derive`, `export` steps.
- Byte-exact or percent download progress. The engine reports text lines; changing that is a separate project.
- A second concurrent run, a queue, or a job history across restarts.
- A desktop GUI (still a non-goal, ADR 0009).
- Changing what the pipeline produces.

### Anti-goals (a failure even if it works)

- A page that lets a request choose a folder, a file name, a URL or a command.
- A second implementation of the pipeline steps that drifts from the CLI and MCP.
- A run that survives silently after the user closed the page and then collides with a terminal run.
- A force rebuild that replaces a map the user edited without a clear confirmation.

## 2. What exists today

| Piece | Where | State |
|---|---|---|
| Server, guards (token, Host, Origin, CSP, headers) | `select.rs` `router`, own copy | works, duplicates core `freemapper-web` |
| Defaults for a new project: kind, scale, title | `select::Defaults`, `api/project` | works; title default is the folder name, resolved by `init_project` |
| Kind and scale controls | `index.html` section `#settings`, `SCALES` in `app.js` (ADR 0085) | works; disabled for an existing project |
| Area preview, tiles, download estimate, disk plan | `api/preview`, `estimate_json` (ADR 0045) | works |
| Out of coverage, too large | `AreaPreview.outside_coverage`, `too_large` | works, shown as warnings |
| Save (creates or changes area) | `api/save` | works; refused with 409 while a pipeline runs (`Busy`) |
| Done | `api/done` | stops the server |
| Language | `navigator.language` in `app.js`, `TEXTS.de` / `TEXTS.en` | works; no `--lang` yet |
| Pipeline | `run_project` + `export_ocad` + `export_pdf` + `export_candidates_pdf` | duplicated in `main.rs` (`Run`) and `mcp.rs` (`start_pipeline`) |
| Jobs, progress lines, warnings | `mcp.rs` `Jobs` (200 lines, `warning:` prefix) | MCP only |

New: the options panel, the run endpoints, the progress and result panels, the job hub shared by CLI, MCP and
page, cancellation, the folder lock, and the move onto `freemapper-web`.

Engine facts that shape the design: `run_project(dir, fetcher, cache, mode, force, progress)` reports only text
lines through `&mut dyn FnMut(&str)`. It has no cancellation. Writes are atomic (partial file, rename), so a
stopped run leaves a consistent folder. `fetch_project` stops before any request when a required part does not
fit the disk (`NotEnoughSpace`).

## 3. Design

### 3.1 Stage 1: `select` on `freemapper-web`

`Selector::start` keeps its signature. Inside, it calls `freemapper_web::serve(build, ServeOptions { port, csp,
strict_post: true, max_body })`. `build(prefix, done)` returns the router with the routes of today; `done`
replaces the private `Notify`. `embedded::<Assets>` replaces the local `embedded`, `open_browser` comes from the
crate (it refuses anything but the local URL). The hand-written token, Host and Origin middleware go away.

`strict_post` is safe for the existing page: every POST in `app.js` goes through `api()`, which sends
`Content-Type: application/json`, and `done` sends `{}`. Browsers send `Origin` on same-origin POSTs. The existing
tests that POST must send `Origin` and the JSON type; a test for each refusal is new.

Body limit: keep the crate default (1 MiB) for `api/preview` and `api/save` (large outlines), set 4 KiB on
`api/run`, `api/cancel` and `api/open-folder`.

This stage changes no behaviour a user sees except that a POST without `Origin` or without the JSON type is now
403 and every response carries `X-Content-Type-Options: nosniff`.

### 3.2 One pipeline function

New module `crates/freemapper-cli/src/pipeline.rs` is the only place that sequences the steps.

```rust
pub struct RunOptions { force: bool, offline: bool, ocad: bool, pdf: bool, pdf_candidates: bool,
                        pdf_scale_bar: bool, pdf_sources: PdfSources }
pub fn parse(request: &RunRequest) -> Result<RunOptions, String>   // JSON/MCP/clap shapes all end here
pub fn run(dir: &Path, options: &RunOptions, cache: &Cache, fetcher: &dyn Fetcher,
           control: &RunControl, progress: &mut dyn FnMut(Event)) -> Result<RunReport, String>
```

`Event` is `Phase { step: Step, index: u8, total: u8 }` or `Line(String)`. `Step` is `Fetch`, `Derive`, `Map`,
`Ocad`, `Pdf`, `PdfCandidates`. `total` counts only the steps the options select. `RunReport` holds `summary:
String` (the text MCP returns today), `warnings: Vec<String>`, `files: Vec<ResultFile { kind, relative_path,
bytes }>` and `kept_map: bool`. `files` lists only the fixed names the engine writes (`map_path`, `ocad/`, `pdf_path`
for both PDFs, `ATTRIBUTION.txt`), never a directory scan.

The CLI `Run` command, MCP `start_pipeline` and the page call `pipeline::run`. The text of today's MCP output
stays byte for byte (a test pins it).

### 3.3 The job hub

`JobHub` replaces `mcp::Jobs` and the `Busy` closure of `Selector::start`:

```rust
pub struct JobHub { /* Mutex<Inner> */ }
impl JobHub {
    pub fn start(&self, dir: &Path, options: RunOptions) -> Result<JobId, StartError>; // refuses while any job runs
    pub fn status(&self, id: &JobId, since: usize) -> Option<JobStatus>;
    pub fn current(&self) -> Option<JobStatus>;
    pub fn cancel(&self, id: &JobId) -> bool;
    pub fn running_for(&self, dir: &Path) -> bool;
}
```

The MCP server owns one hub for its lifetime and gives the same `Arc<JobHub>` to every selector page it opens,
so a run started on the page shows in `project_status` and the reverse. `freemapper select` from the CLI creates its own
hub. The one-run-at-a-time rule stays process wide, because projects share the download cache (today's MCP rule).

`JobStatus`: `job_id`, `state` (`running`, `finished`, `failed`, `cancelled`), `phase` (`step`, `index`, `total`),
`lines` (those after the cursor `since`, with the new cursor `next`), `warnings`, `report` (on finish), `error`.
Lines are capped at 200 per job (as today) and 500 characters each.

### 3.4 Cancellation

Add to the engine `ProjectError::Cancelled` and `run_project_cancellable(..., cancelled: &dyn Fn() -> bool)`;
`run_project` calls it with `|| false`. The flag is checked before each phase, at each tile boundary in fetch and
derive, and by `CancellingFetcher`, a `Fetcher` wrapper in the CLI that returns an error before each request. A
running HTTP download finishes its current request or hits the existing timeout. Cancel does not delete the cache
(a later run reuses it) and leaves the project consistent because every write is atomic. Cancel after the OOM map
is written leaves that map; the result panel says what exists.

### 3.5 The folder lock

`<project>/.freemapper-run.lock` holds JSON `{ "pid": n, "started": "...", "who": "cli|mcp|page" }`. It is
created with `freemapper_fs::create_new_file` (fails on any existing entry, links included, ADR 0060). A guard
rewrites it every 10 s (heartbeat through `write_inside`) and removes it on drop. A lock whose file time is older
than 60 s is stale and is taken over; the workspace forbids `unsafe`, so there is no pid probe. A live lock gives:
"another freemapper run is using this folder (started 14:02 by the terminal). Wait for it or stop it first."

`run`, `fetch`, `derive`, `export` and `save` (area change) take the lock. This closes the CLI against page gap of ADR
0009 for runs and saves. Plain reading stays free. Both the hub and the lock apply: the hub for the process, the lock for
the folder across processes.

### 3.6 HTTP interface

All routes under `/{token}/`, JSON in and out, errors as `{ "error": "..." }`.

| Route | Purpose |
|---|---|
| `GET api/project` | adds `run_defaults` (the options as the page starts them), `allowed_scales` per kind, `lang` (null today), `running` (current job or null) |
| `POST api/run` | body = options only (see below); needs a saved project; 202 `{job_id}` |
| `GET api/status?job=ID&since=N` | `JobStatus`; without `job` the current one |
| `POST api/cancel` | `{job_id}`; 200 `{cancelling: true}` |
| `POST api/open-folder` | opens the project folder in the file manager; no body fields |

`api/run` body: `{ force, offline, ocad, pdf, pdf_candidates, pdf_scale_bar, pdf_sources }`. Unknown fields are
refused (`deny_unknown_fields`). There is no `dir`, no `kind`, no `scale`, no `title`: kind, scale and title are
fixed by the saved project (ADR 0085, 0086) and changed only through the existing save of a new project.

Errors: 400 invalid option (same texts as MCP: `pdf_sources must be auto, full or credit-only, not "x"`), 409 a
job is running or the folder is locked, 404 no saved project ("save the area first"), 422 pre-flight stop
(`disk.stops`).

### 3.7 Polling, not Server-Sent Events

Polling `GET api/status` every second with a line cursor. Reasons: the run is slow and lines come at most a few
per second; `EventSource` cannot send headers, cannot show a clean error, and a held connection complicates
graceful shutdown (the server waits for open connections); polling survives a page reload (the state is on the
server), is trivial to test with a stub, and reuses the guard unchanged. Cost: up to one second of latency, and a
request per second to localhost. The poll stops when the state is not `running`; the interval backs off to 3 s
after 5 minutes. Cost if wrong: swapping in SSE later touches one JS function and one route.

### 3.8 The page

New section "Karte erzeugen" below the settings, split into a script `web/run.js` (options, run, progress,
result) next to `app.js` (map, area). Text tables `RUN_TEXTS.de` / `.en` live in `run.js`; the language is chosen
as today (`navigator.language`), and `project.lang`, when the server sends one (future `--lang`), wins. No
framework.

Options (labels DE / EN, defaults as the CLI):

| Option | DE label | EN label | Default | Notes |
|---|---|---|---|---|
| ocad | OCAD-Startprojekt schreiben | Write an OCAD starter project | off | help: "für alle, die in OCAD weiterarbeiten" |
| pdf | Druckfertige PDF | Printable PDF | off | map objects only |
| pdf_candidates | Trainings-PDF mit allen Kandidaten | Training PDF with every candidate | off | |
| pdf_scale_bar | Maßstabsleiste in die PDFs | Scale bar on the PDFs | off | disabled unless a PDF is on; help: German maps usually have none |
| pdf_sources | Quellenangabe in den PDFs | Source lines on the PDFs | auto | select: auto / full / credit-only; disabled unless a PDF is on; credit-only shows the ODbL / dl-de/by-2-0 warning |
| offline | Nur Zwischenspeicher, kein Netz | Cache only, no network | off | |
| force | Vorhandene Karte ersetzen | Replace the existing map | off | shown only when `has_map`; the old file becomes `.omap.bak` |

Kind, scale and title stay where they are; for an existing project they show their stored values, read-only,
with the existing note. Validation in the page mirrors the server (scale list per kind from `allowed_scales`,
scale bar and sources only with a PDF); the server decides.

Flow: the button is disabled until a saved or savable area exists and the preview shows no `too_large`,
`outside_coverage` or `disk.stops`. Pressing it saves first when the area changed (existing `api/save`), then
posts `api/run`. While a job runs the options and the area controls are disabled, "Speichern" too.

Progress panel (`role="status"`, `aria-live="polite"`): a line "Schritt 2 von 5: Daten berechnen" from the phase
event, an indeterminate `<progress>` (no percent is claimed), the last 8 lines in a log, warnings collected in a
list that stays after the end. Plain-language step names: Fetch "Daten laden", Derive "Vorlagen berechnen", Map
"Karte schreiben", Ocad "OCAD-Projekt schreiben", Pdf "PDF schreiben", PdfCandidates "Trainings-PDF schreiben".
Lines from the engine are shown as they are (English today; they are program messages, not page text).

Cancel button next to the bar: "Abbrechen". Pressed once it says "Wird abgebrochen …" and stays disabled until the state
changes.

Result panel (`role="region"`, focus moved to its heading on finish): the state in one sentence, the file list
(name, what it is, size), the folder with a button "Ordner öffnen" and, when that fails, the path as text,
the warnings, and "Karte erzeugen" re-enabled as "Nochmal erzeugen". On failure: the error text from the engine
(`describe`), what stays on disk, and the retry button. On cancel: "Abgebrochen. Was schon geladen ist, bleibt im
Zwischenspeicher."

Pre-flight panel (reuses the summary section): disk volumes and `messages` as today; when `disk.stops` the run
button stays disabled with the message; `outside_coverage` text as today ("Das Gebiet liegt außerhalb der
unterstützten Bundesländer ..." replaces "Länder"; wording pass per the Bundesland rule); states the data comes from
is not shown before the run, border warnings of the engine (`warning:` lines about a second Bundesland) appear in
the warning list after fetch starts, highlighted. Existing map: with `has_map` and `force` ticked, a `<dialog>`:
"Die vorhandene Karte wird ersetzt. Deine Änderungen darin gehen nicht verloren, die alte Datei bleibt als
<name>.omap.bak. Fortfahren?" with Cancel as the default focus.

Done: "Fertig" stays. After a finished run the closing text names the files. For an agent-opened page
(`from_agent`) the closing text still says "go back to the agent"; the agent sees the same job through the shared hub.

### 3.9 Defaults for the options

`freemapper select` gets the run flags as page defaults: `--ocad --pdf --pdf-candidates --pdf-scale-bar
--pdf-sources --offline`. MCP `open_area_selector` gets the same optional fields. The page starts with them. No
browser storage: the origin changes with every start (random port), so it would never be read back.

### 3.10 Security

- Token, Host, Origin, `Sec-Fetch-Site` and `strict_post` come from `freemapper-web`; nothing is re-implemented.
- No path, URL, command or file name in any request. The folder is the one the selector was started with
  (ADR 0059). `open-folder` takes no body fields and runs the OS opener with that fixed absolute folder as its
  only argument, no shell, `Command::arg`, never a value starting with `-` (the folder is made absolute first).
- The run uses `state.dir`, the cache of the selector, and nothing from the request but the seven options.
- Body limits 4 KiB on the new POST routes. At most 1 status request per 200 ms is served; a faster client gets
  429 (a tab in a loop must not starve the pipeline thread).
- The lock file is created and removed through `freemapper-fs`, never through a link (ADR 0060).
- Log lines and errors are text in `textContent`, never HTML.
- `open_browser` is the crate's, restricted to the local URL.

### 3.11 What stays CLI and MCP only

`init` from GeoJSON, OSM id with `--part`, `--place`, `--cache-dir`, `--port`, `fetch`, `derive`, `export` as single
steps (an expert re-running only the PDF), adapters (ADR plan 2026-11-10). MCP keeps `find_place`,
`create_project`, `list_layers`, `preview_layer`. The page does not show layer previews in this project.

## 4. Testing

- Stage 1: HTTP tests with `ureq` on the real server: the old suite, plus POST without `Origin` = 403, POST with
  `text/plain` = 403, a cross-site `Sec-Fetch-Site` = 403, `nosniff` present.
- `pipeline::run` with a stub `Fetcher` and the fixture project: step order, `total` per option set, the
  `RunReport.files` list, the MCP summary pinned.
- `JobHub` unit tests: second start refused, cancel flag, cursor `since`, a panic becomes `failed`.
- HTTP tests against a stub runner (`trait PipelineRunner`, the real one in production): `api/run` rejects unknown
  fields, bad `pdf_sources`, no project (404), pre-flight stop (422), second run (409); `api/status` cursor;
  `api/cancel`; the response never contains the token.
- Lock tests: create twice fails, stale lock (file time 61 s old, via `filetime`-free `set_modified`) is taken over,
  a link at the lock path is not followed, the guard removes the file on drop and on a panic.
- Engine: `run_project_cancellable` stops before the next phase and returns `Cancelled`; no partial file remains.
- Page logic that can run without a browser (option validation, state table, step names) sits in `run.js` as
  pure functions; one Rust test checks that every `RUN_TEXTS` key exists in both languages (parsing the file as
  the existing `app.js` test does for tiles).
- Optional smoke, not in CI: a `playwright-cli` script that opens the page against the stub runner, ticks PDF,
  runs, cancels, runs again.

## 5. Accessibility and UX

- Every control has a visible `<label>`; help text is linked with `aria-describedby`.
- Tab order follows the visual order: area, settings, options, run. The run button is a `<button>`, the dialog is
  `<dialog>` with focus trap and `Esc`.
- Progress and errors in `aria-live="polite"` / `role="alert"` regions; focus moves to the result heading when a
  run ends. No colour-only states: each state has text.
- Disabled controls say why in an adjacent hint, not only a tooltip.
- Errors show the engine text and one next step ("Schaffe Platz und starte erneut"). Long runs say they can take
  minutes and that closing the tab does not stop the run (the program does).
- Closing the tab during a run: the run continues; reopening the URL shows the live state (`running` in
  `api/project`). Pressing Ctrl+C in the terminal asks the hub to cancel, waits up to 5 s, then exits.

## 6. Rulings

Each: Ruling. Why. Cost if wrong.

1. Ruling: poll `GET api/status` with a cursor, no SSE. Why: slow-changing text lines, reload survival, simpler
   shutdown and tests. Cost if wrong: one second of latency; swapping to SSE touches one route and one JS function.
2. Ruling: stage 1 moves `select` to `freemapper-web` v0.11.0 with `strict_post: true` before any feature. Why: the
   new POST routes start a long job and need the strongest guard; the guard code should not exist twice. Cost if wrong:
   a pinned git tag in a second core crate version; a rollback is one revert because the public `Selector` API stays.
3. Ruling: one `pipeline::run` in the CLI crate for CLI, MCP and page. Why: the sequence is duplicated in two places
   today and the page would be the third. Cost if wrong: the module moves into the engine crate later; callers do not change.
4. Ruling: a process wide `JobHub` shared by MCP and its pages replaces `Jobs` and `Busy`. Why: page and agent must see
   the same single run. Cost if wrong: a refactor of `mcp.rs` that is larger than the feature; it is its own task.
5. Ruling: cancel is cooperative (flag checked at phases, tile boundaries and before each request). Why: the engine
   has no abort and killing a thread is unsafe. Cost if wrong: cancel reacts after the current request or tile,
   which can be tens of seconds on a slow download.
6. Ruling: no percent bar; show "step i of n" and an indeterminate bar. Why: the engine reports only text lines;
   an invented percent would lie. Cost if wrong: users want a number; a later engine change adds byte counts.
7. Ruling: the lock is a heartbeat file with a 60 s stale limit, taken by `run`, `fetch`, `derive`, `export` and
   area saves. Why: no `unsafe` means no pid probe; `create_new_file` gives atomic exclusive creation without links.
   Cost if wrong: after a crash the folder is blocked for up to 60 s; a clock jump can free a live lock (the hub
   still protects within the process).
8. Ruling: the run request carries seven option fields and nothing else; kind, scale and title are not changeable
   from `api/run`. Why: ADR 0085 and 0086 fix them at project creation; a request that renames a project is a new
   feature. Cost if wrong: the user changes kind only by starting over with a new folder.
9. Ruling: `force` is a visible checkbox only when a map exists, default off, with a `<dialog>` confirmation; no
   silent force. Why: the existing map may hold the user's work. Cost if wrong: one extra click for experts.
10. Ruling: run defaults come from new `select` flags and MCP fields, not from browser storage. Why: the origin
    changes per start. Cost if wrong: the user ticks the same boxes each time; shell aliases cover it.
11. Ruling: page language stays `navigator.language`; a `lang` field in `api/project` is reserved for the program
    language work (`--lang`, `FREEMAPPER_LANG`), which does not exist in code yet. Why: avoids blocking on it and
    avoids a second language switch later. Cost if wrong: one mismatch (terminal in German, page in English) until that work lands.
12. Ruling: `open-folder` is a server action with no input. Why: browsers cannot open `file:` links from a
    `127.0.0.1` page and the user otherwise copies a path by hand. Cost if wrong: it fails on headless machines;
    the page then shows the path.

## 7. Risks

- `mcp.rs` refactor (hub) touches a 36 KB file with its own tests; do it in one task with the old tests first.
- Cancel reaction time; the UI says "wird abgebrochen" and never claims instant stop.
- A stale or foreign lock file blocks the folder; the message names the file and how long to wait, and a test
  covers the takeover.
- Two core crate versions in the dependency tree (`freemapper-fs` v0.5.1 and `freemapper-web` v0.11.0 come from
  one git repo at different tags). Check `cargo tree -d` in stage 1; both are separate packages, so it builds,
  but the lock file grows.
- A user closes the terminal during a run started from the page: the run dies with the process. The page cannot
  detect this; the next start finds a stale lock and says so.
- Windows: `open-folder` uses `explorer`, and lock file replacement by rename on an open file; both need the CI
  Windows job, as does the heartbeat.
- Scope creep into the layer preview and expert flags; the non-goals list is the answer.
