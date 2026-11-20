# 0165. Mutation tests run on a random part of the diff

Date: 2026-11-20

Status: Accepted, amends ADR 0161

## Context

ADR 0161 made `cargo-mutants` over the changed lines a gate of every pull request. On a pull
request with a large diff (the Poland adapter, 137 mutants) the baseline alone takes about ten
minutes (5.5 min build, 4.5 min tests) and each mutant adds about five more. Two jobs in
parallel need hours; the job hit its 90 minute limit twice and failed without a verdict. The
rest of CI must finish in ten minutes; the mutation tests may take ten more on top of their
baseline.

## Decision

- The job `mutants` stays on every pull request, but tests a random part of the mutants: the
  mutants of the changed lines are split over twelve runners (`--shard`, round-robin), each runs
  its share in random order (`--shuffle`) and stops ten minutes after its baseline
  (`MUTANTS_BUDGET_SECONDS`). Every run covers another part. A part that ends with the budget
  is no failure; a mutant that survived before it is.
- The gate and its rule do not change: a surviving mutant is fixed with a test. The share of the
  diff that a run covers is `MUTANTS_SHARDS` runners times what two jobs finish in ten minutes,
  about a quarter of a large diff and all of a small one. More runners cover more.
- A label that starts the run on request was rejected: it leaves the gate to whoever remembers
  it.

## Consequences

- A pull request takes the baseline plus ten minutes for this job, in parallel with the others.
- A mutant in a large diff can pass one run and be found by a later one; a rerun of the job draws
  another part.
