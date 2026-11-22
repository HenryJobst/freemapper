# ADR 0176: Rounded counts in the badges

Date: 2026-11-22

Status: Accepted, amends ADR 0070 and ADR 0163

## Context

The README badges name the number of tests and of ADRs, and `docs/tests.json` records the number
of tests. Every pull request that adds a test or an ADR changed both files. Two open pull
requests then collided in `README.md`, `README.en.md` and `docs/tests.json`, and a merge into
main made every other open pull request fail the gates `badges` and `tests` until it was rebased
and its numbers were written again. In one day that cost about one CI run per pull request.

## Decision

- The badge shows the number rounded down: tests to the hundred ("3000+"), ADRs to the ten
  ("170+").
- `docs/tests.json` records the same rounded number. The gate `tests` holds while the declared
  count lies in `[recorded, recorded + 100)`; `scripts/check.sh tests --fix` rewrites the file
  only when it does not.
- The coverage, the test budget, the crates, the MSRV, the licence and the number of checks stay
  as they are: they change rarely.

## Consequences

- A pull request touches the README badges only when it crosses a hundred tests or ten ADRs, so
  most pull requests no longer collide there.
- The badge says less: "3000+" does not tell 3001 from 3099. The exact count is one command away
  (`scripts/check.sh tests` names the declared count when it differs).
