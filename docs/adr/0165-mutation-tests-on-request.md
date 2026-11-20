# 0165. Mutation tests run on request

Date: 2026-11-20

Status: Accepted, amends ADR 0161

## Context

ADR 0161 made `cargo-mutants` over the changed lines a gate of every pull request. On a pull
request with a large diff (the Poland adapter, 137 mutants) the baseline alone takes about ten
minutes (5.5 min build, 4.5 min tests) and each mutant adds about five more. Two jobs in
parallel need hours; the job hit its 90 minute limit twice and failed without a verdict. The
rest of CI must finish in ten minutes: a gate that takes hours is not one.

## Decision

- The job `mutants` runs only on a pull request that carries the label `mutants`. Without the
  label it is skipped, and a skipped job passes. Its limit is 360 minutes.
- The gate itself, `scripts/check.sh mutants`, and its rule do not change: a surviving mutant is
  fixed with a test. The label is for a change that touches logic with branches (a border
  check, a parser, a rule), before it merges. A pull request of data, text or wiring does not
  need it.
- A random part of the mutants on every run, with more jobs, was rejected: the baseline alone is
  as long as the whole budget, so no part of the run fits.

## Consequences

- Pull requests finish in the time of the other jobs.
- A mutant can reach `main` unnoticed when nobody sets the label; the author of a logic change
  sets it, and a reviewer asks for it.
