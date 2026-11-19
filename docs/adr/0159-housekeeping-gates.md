# 0159. The repository checks its own housekeeping

Date: 2026-11-19
Status: Accepted

## Context

CI ran the compiler, the golden net, `cargo deny` and the release manifests. Everything around the
code was kept in order by hand: the ADR table, the changelog pair, links between documents, the
dependency list. freemapper-core, freemapper-courseplan and freemapper-print had the same gap, and
the gates are cut the same way in all four.

The first runs showed how much had drifted. ADR 0158 had no row in `docs/adr/README.md` and linked
`0150-training-map-second-half.md`, while the file is `0150-the-training-map-second-half.md`.
`cargo-machete` found `crc32fast` declared in the `freemapper` crate and used only in
`freemapper-sources`, and `schemars` in the CLI. rustdoc, never held to `-D warnings`, linked 78
times to private items and met `<name>` and `<year>` as HTML tags. And nothing measured coverage or
tested the `msrv` claim.

## Decision

- `scripts/check.sh` is the single entry point for the gates and has no dependencies beyond git and
  the tools named below. CI calls the same commands as a person does.
- The gates are `spdx`, `typos`, `links`, `secrets`, `machete`, `actions`, `adr`, `changelog`,
  `doc`, `coverage` and `msrv`. The first eight need no build and run on every commit
  (`lefthook.yml`); `secrets` runs before a push because it walks a history of about 200 MB; `doc`,
  `coverage` and `msrv` build the workspace and each have their own CI job.
- Every Rust file starts with `// SPDX-License-Identifier: AGPL-3.0-or-later`. Documents and the
  example maps do not repeat the line.
- `typos` checks the English documents only — `*.en.md`, `AGENTS.md`, `CLAUDE.md`, `docs/adr/`,
  `docs/superpowers/` and anything named `en.md`. German documents, the examples whose English
  README quotes German sheet names, the maps under `assets/` and the vendor tree stay out. The
  German field names that the English documents quote (`funktion`, `Oder`, `Reviere`, `Forst`,
  `adresse`, `beginn`, `TITEL`) and the identifiers the dictionary splits (`lod`, `ba`, `ue`,
  `ND`) are listed in `_typos.toml`, each with the place that needed it.
- `lychee` checks links and anchors offline. The plans and specs under `docs/superpowers/` are
  excluded: they record how the code came to be and name files as they stood then. Everything
  living — READMEs, manual, glossary, ADRs, example READMEs — is checked.
- `gitleaks` walks the whole history and reports redacted findings. Two paths are allowed:
  `examples/*/permits.json`, whose `key` field holds a source name and not a credential, and the
  removed research note `docs/regions/sweden-reference.md`, whose only `curl -u` carried the
  placeholder `abc:def` while asking whether the Swedish API needs authentication.
- `cargo-machete` removed three dependencies from the `freemapper` crate: `crc32fast` and `weezl`
  belong to `freemapper-sources`, and `freemapper-text` is used by the CLI and the adapter API, not
  by the engine. `schemars` stays in the CLI with a `[package.metadata.cargo-machete]` entry,
  because the `JsonSchema` derive expands to paths in it.
- `cargo doc` runs with `RUSTDOCFLAGS=-D warnings`. Doc comments that linked private items name
  them in backticks now, `[[masks]]`-style TOML and the attribution's `<year>` stand in code spans,
  and the missing ADR row is written. The parameter descriptions of `StartPipeline` keep plain
  `<name>`: an MCP test compares them word for word with the text catalog, so the struct carries
  `#[allow(rustdoc::invalid_html_tags)]` instead of a code span that would change the wording.
- Coverage has a floor of 94.0 % in `scripts/check.sh` and the last measurement in
  `docs/coverage.json`. A run below either fails; a run above the snapshot rewrites it, and the
  commit belongs with the change that raised it. The first measurement is 95.9 %.
- The MSRV stays 1.88 and a job checks it with 1.88.0. freemapper inherits
  `freemapper-contract` with `default-features = false` on the workspace entry itself, which cargo
  before 1.99 reads; only a member that overrides an inherited entry needs 1.99, as
  freemapper-print and freemapper-courseplan do.
- CI gives the gates their own jobs (`hygiene`, `docs`, `msrv`, `coverage`) beside `deny`, `check`,
  `golden`, `golden-guard` and the release pipeline. The tools are pinned by version and checksum.

## Consequences

- A pull request now fails on a broken link or anchor, a missing ADR row, a changelog that misses a
  version, a declared but unused dependency, an action that hangs on a tag, a secret in the
  history, a rustdoc warning and a coverage drop.
- Raising coverage later means editing the floor in `scripts/check.sh` in the same commit as the
  better number.
- Historical plans may keep dangling references, but a link in a living document is a defect now.
- `typos` will meet another German quote in English text and fail. The answer is an entry in
  `_typos.toml` with its reason, not switching the gate off.
- freemapper-core v0.29.1 carries the workspace-dependency fix. This repository does not need it,
  because its own entry is already written the way older cargo reads it.
