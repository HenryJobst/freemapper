# 0090. Create the map from the browser page

Date: 2026-11-11
Status: Proposed

Amends [0009](0009-area-selection-in-a-local-browser-page.md).

## Context

`freemapper select` ends at "Save" and "Done". The user must then run `freemapper run` in a terminal with the right
options, or ask an agent to call `start_pipeline`. The page already holds map kind, scale and title (ADRs 0085,
0086) and the download estimate with the disk check (ADR 0045). The user asked why the project cannot be created
there, with all options.

Facts that limit the design:

- The pipeline sequence (`run_project`, OCAD, two PDFs) is written twice, in `main.rs` and in `mcp.rs`. A page would be the third copy.
- The engine reports text lines only and has no cancellation. Its writes are atomic.
- ADR 0009 says there is no lock between a CLI `run` and a page in another terminal.
- The page's server guards are copied code. Core `freemapper-web` v0.11.0 offers the same guards plus
  `strict_post` and a body limit.
- A route that starts a download of minutes is a bigger target than a route that saves a polygon.

## Decision

- **Stage 1: move first.** `select` runs on `freemapper-web::serve` with `strict_post: true`. `Selector::start` keeps
  its signature. No feature work starts before this is merged.
- **One pipeline function** `pipeline::run` in the CLI crate serves the CLI, MCP and the page. A `JobHub`, shared
  by the MCP server and its pages, holds the single running job (process wide, because projects share the cache).
- **Routes:** `POST api/run` (seven option fields, nothing else), `GET api/status?since=N` (polling, no
  Server-Sent Events), `POST api/cancel`, `POST api/open-folder`. Kind, scale and title stay as saved. No request
  names a path; the run uses the folder the selector started with (ADRs 0059, 0060).
- **Cancel is cooperative:** a flag checked at phases, tile boundaries and before each request. Progress is "step
  i of n" with text lines; no percent is claimed.
- **Lock across processes:** `<project>/.freemapper-run.lock`, created with `freemapper_fs::create_new_file`,
  refreshed every 10 s, stale after 60 s. `run`, `fetch`, `derive`, `export` and area saves take it. This closes
  the gap named in ADR 0009.
- **Options** are those of `run` and `start_pipeline` with the same defaults and validation: OCAD, PDF, training PDF,
  scale bar, sources (`auto`, `full`, `credit-only`), offline, and force. Force appears only for an existing map,
  off by default, behind a confirmation. Defaults come from new `select` flags and MCP fields.
- **Pre-flight** reuses the preview: coverage, size, disk estimate with the stop case. The run button stays off when
  the fetch would stop.
- **Stays CLI and MCP only:** adapters, `--cache-dir`, `--port`, `--part`, GeoJSON input, the single steps `fetch`,
  `derive`, `export`.
- Texts German and English in `web/run.js`, chosen as today by the browser language, with a reserved `lang` field
  for the later program language setting.

## Consequences

- A user without a terminal can go from the page to the finished map and PDFs.
- The page and the agent see the same job; a run started in one shows in the other.
- The rule "ADR 0009: no lock" no longer holds for runs and saves. Other writers (a text editor, OOM) are not locked.
- A POST without `Origin` or without `Content-Type: application/json` now gets 403.
- The mcp.rs job code moves into the hub; MCP output text stays the same.
- Cost if wrong: a cancel that reacts slowly, a stale lock that blocks a folder for up to a minute after a crash, a
  larger `mcp.rs` change than the feature; each is fixed locally. The move onto `freemapper-web` is one revert.

Source: user request 2026-11-11; spec `docs/superpowers/specs/2026-11-11-browser-run-design.md`; plan
`docs/superpowers/plans/2026-11-11-browser-run.md`; ADRs 0009, 0045, 0059, 0060, 0085, 0086, 0088; core
`freemapper-web` v0.11.0.

Update 2026-11-11, stage 1 done: `select` runs on `freemapper-web` v0.11.0 with `strict_post` on and a 1 MiB body limit. `Selector::start` keeps its signature. The duplicated token, host and origin middleware, security headers and `open_browser` are gone from `select.rs`. All core crates moved to the tag v0.11.0 (ADR 0067). `Selector::is_running` is true until the user presses "done", because the shared crate has no query for a finished server.

Update 2026-11-11, stage 2 done: `pipeline::run` in `crates/freemapper-cli/src/pipeline.rs` is the one sequence behind `freemapper run` and the MCP tool `start_pipeline`; the MCP summary texts moved there unchanged. The command line gets each result as an `Event::Written` as soon as the step is done, so it prints the same lines as before even when a later output fails, and `RunOptions.fail_fast` keeps its stop at the first failing output (MCP and the page keep going and write the failure into the summary). `run_project_steps` in the library tells which step starts next; `run_project` stays as it was. `JobHub` in `jobs.rs` replaces the MCP job table and the `Busy` closure; `Selector::start` takes an `Option<Arc<JobHub>>`, and saving still gets 409 while a run holds the folder. The hub keeps 200 lines per job and the MCP status still shows the last 50; `status` adds the keys `phase` and `next`, and the state `cancelled` appears with stage 3.

Update 2026-11-11, stage 3 done: cancel and folder lock. `run_project_cancellable`, `run_project_steps_cancellable`, `fetch_project_cancellable` and `derive_project_cancellable` take a `&dyn Fn() -> bool`; the old functions call them with `|| false`. The flag is read before each step, after each part of the fetch (before its result is recorded in the project file), before each template of the derive and before each request: the fetch wraps the `Fetcher` in a `CancelCheck` that answers `FetchError::Cancelled`, which no retry loop repeats. Every file still goes through the atomic write (partial file, then rename), so a stop leaves whole files and no `.part`; the stale-part cleanup of ADR 0059 and 0060 is unchanged. `pipeline::run` reads `RunControl` the same way, also before each output, and ends with the error `cancelled` plus `; kept: <files>` when files exist, which the hub turns into the state `cancelled`. There is no `CancellingFetcher` in the CLI: the library check covers every caller. The command line does not cancel itself; Ctrl+C ends it. `lock.rs` holds the folder lock: the claim is `OpenOptions::create_new` and not `freemapper_fs::create_new_file`, because that one removes an existing entry first and would take a live lock; `check_inside` and `remove_inside` of `freemapper-fs` guard the folder and the removal of a stale or linked lock file, and the heartbeat touches the open file handle, never the path. A takeover is not atomic: two processes that find the same stale lock at the same moment can both remove it once; the second look right before the removal makes that window small, and the one who loses the race gets "held". `run`, `fetch`, `derive`, `export` (command line, Ctrl+C releases the lock), the MCP `start_pipeline` (held by the job) and the area save of the page take it; a folder that does not exist yet is not locked. The page answers a held folder with 409.
