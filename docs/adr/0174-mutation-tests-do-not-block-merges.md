# ADR 0174: Mutation tests do not block merges; a nightly run reports

Date: 2026-11-21

Status: Accepted, amends ADR 0161 and ADR 0165

## Context

ADR 0161 made the mutation tests a gate of every pull request, and ADR 0165 limited them to a
random part of the diff. A run still takes the baseline plus ten minutes, 13 to 22 minutes on the
last pull requests (ADR 0173), and every other check is done long before. The wait slows the
development more than a surviving mutant costs: a mutant that no test catches is a gap in the
tests, not a bug in the program.

## Decision

- The job `mutants` of a pull request stays and shows a surviving mutant (the step fails and lists
  it), but the job does not stop the merge (`continue-on-error`).
- A nightly run (`mutants-nightly.yml`) takes the lines that main changed in the last 24 hours,
  gives each of twelve runners forty minutes after its baseline and opens an issue when a mutant
  survived. The issue names the run; the rule "fix with a test, not with an exclusion" stays.
- The author of a pull request still sees the result before merging and fixes what is cheap.

## Consequences

- A merge no longer waits 13 to 22 minutes for the mutation tests; a gap in the tests is found
  within a day instead of before the merge.
- A gap can reach main. The issue is the reminder; nothing ties a release to it. A release stays
  on hold while such an issue is open for the code it ships (the person who releases decides).
- The nightly run costs runner minutes on days main changed Rust code only: with no change it ends
  at once (`no Rust change`).

## Addendum 2026-11-28: later start, no job without Rust

The twelve shards of a pull request queued behind the other checks and held the runners while the
person waited for them. Two changes make them cheaper, and they stay non-blocking:

- The job `mutants` has `needs: check`. A pull request whose tests do not pass starts no shard; the
  fix pushes a new run anyway (ADR 0161 stays: twelve shards, ten minutes after the baseline).
- Each shard first looks for `*.rs` files in the diff against the base. With none (documentation,
  changelog, manifests, goldens) it ends after the checkout, without the toolchain, the cache, the
  tools and the build of the tests. `scripts/check.sh mutants` keeps its own check for the same case.
