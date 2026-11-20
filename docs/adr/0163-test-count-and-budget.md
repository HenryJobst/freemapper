# 0163. The test count and the test budget are data

Date: 2026-11-20
Status: Accepted

## Context

The README carries badges for coverage, MSRV, ADRs, the hygiene gates and the licence, each read
from a file in the repository. Two numbers about the tests were missing: how many there are and
how long the suite may take. The sibling project MergeWarden shows both and fails when the suite
grows slower than its budget.

## Decision

- **The test count is declared, not run.** `scripts/check.sh tests` counts the `#[test]` and
  `#[tokio::test]` attributes in the tracked Rust files and compares the number with `tests` in
  `docs/tests.json`; `tests --fix` rewrites it. Measured here: 2877 declared, 2846 listed by
  `cargo test -- --list`. A run would count differently per system (tests for one platform only),
  needs a full build and could not be checked in the cheap `hygiene` job. Doc tests are not
  counted. A test that is removed or added is a visible edit of the file in the same commit.
- **The budget is a number set by hand.** `budget_seconds` in `docs/tests.json` is 900 s. CI times
  its main test step on Linux, build included, and `scripts/check.sh budget` fails when it takes
  longer. The step took 254 s on the Linux runner, so the budget leaves room for a cold
  cache and a slow runner without hiding a slowdown. It is raised only with a reason in the commit,
  as the coverage floor is.
- **Two badges** (`Tests`, `Test budget`) join the block that `scripts/badges.sh` writes; both read
  `docs/tests.json`. The hygiene badge counts the two new gates.
- **Rejected: counting from a run.** It would make the number depend on the system that wrote it
  and tie the README to a build.

## Consequences

- Adding or removing a test changes `docs/tests.json`; `scripts/check.sh tests --fix` does it.
- A suite that runs longer than the budget fails the Linux `check` job.
- A changed budget is an edit of `budget_seconds` with the reason in the commit.
