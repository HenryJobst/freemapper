# ADR 0173: Faster mutation runs

Date: 2026-11-21

Status: Accepted, amends ADR 0165

## Context

A run of the job `mutants` takes the baseline plus ten minutes. In the last pull requests it was 5
to 21 minutes on the slowest of twelve runners, 0 for a change without Rust code. Two things made
it longer than the work asks for:

- `cargo mutants` builds in a copy of the tree, in another directory, so the cache of the runner
  (`rust-cache`) never held that build. The baseline compiled everything each time.
- A mutant that makes a loop endless was stopped after five times the baseline test time. With a
  workspace baseline of several minutes, one such mutant cost 19 minutes on a local run
  (`Event::Eof` deleted in the reader of the Polish cadastre).

Some mutants cannot be caught by any test: the arithmetic of a numeric iteration whose result does
not change when a slope or step is altered.

## Decision

- The job builds the tests once in the tree of the runner (`cargo nextest run --workspace
  --no-run`), which `rust-cache` keeps, and `cargo mutants` copies that target directory into its
  build directories (`--copy-target true`).
- The timeout of a mutant is twice the baseline test time (was five times).
- The mutants of the arithmetic of `pl1992_inverse` are left out (`.cargo/mutants.toml`,
  pattern ` in pl1992_inverse$`). The function itself stays mutated as a whole (returning `None`
  is caught); the rule "fix with a test, not with an exclusion" holds for everything else.

## Consequences

- The baseline of a run should shrink to the compile time of the changed crates plus the tests.
  The effect is measured on the next pull requests with Rust changes and noted here.
- A mutant that needs more than twice the test time because of a slow runner would count as
  caught; ADR 0161 already treats a timeout as caught.
- Not decided here: letting a failed mutation run no longer block a merge, and a nightly full run
  on main.
