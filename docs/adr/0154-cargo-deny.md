# 0154. cargo-deny checks the dependencies

Date: 2026-11-15
Status: Accepted

Builds on [0152](0152-agpl-3.0-or-later.md) (the licence) and [0067](0067-shared-crates-from-freemapper-core.md) (the core pinned by tag).

## Context

freemapper ships one program built from about four hundred crates. Nothing checked whether a new crate comes with a
licence that does not fit the AGPL, whether a crate in the tree has a known vulnerability or lost its maintainer, or
whether a dependency comes from a source other than crates.io and freemapper-core. freemapper-print added cargo-deny
for the same reason (its ADR 0068) and asked the suite to follow.

## Decision

- `deny.toml` takes freemapper-print's: the licences that fit AGPL-3.0-or-later, yanked crates denied, duplicates only
  reported, crates.io and freemapper-core as the only sources, and `wildcards = "allow"` because the workspace and
  core crates are path or git dependencies without a version.
- The CI job `deny` runs `cargo deny check` on every push and pull request, with the action pinned by commit (v2.1.1).
- The first run found four advisories; each is ignored with a reason and when to look again:
  - RUSTSEC-2026-0457 and RUSTSEC-2026-0462 (rmcp): the OAuth and the Streamable HTTP client. freemapper builds rmcp
    with `server`, `transport-io` and `macros` only, so neither is in the program. rmcp 3.9.0 fixes both; it replaces
    the pin of 3.6.0 once it is two weeks old, and both ignores go.
  - RUSTSEC-2026-0192 (ttf-parser, unmaintained): freemapper-render reads fonts with it; a replacement is a decision of
    freemapper-core.
  - RUSTSEC-2026-0374 (basic-toml, unmaintained): askama reads its templates with it while world_magnetic_model is
    compiled; nothing of it is in the program.

## Consequences

- A new dependency with a licence outside the list, from another source, or with a known vulnerability fails CI and
  needs a decision here.
- An advisory published later fails the next run on main even without a change; it is decided the same way.

## Addendum (2026-11-15): ttf-parser is gone

freemapper-core v0.29.0 (core ADR 0037) reads fonts with skrifa instead of ttf-parser. freemapper pins it, and the
ignore of RUSTSEC-2026-0192 is gone from `deny.toml`.
