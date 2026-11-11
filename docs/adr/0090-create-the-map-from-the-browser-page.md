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
