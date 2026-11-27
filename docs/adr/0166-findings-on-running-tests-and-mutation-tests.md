# 0166. Findings on running tests and mutation tests

Date: 2026-11-20
Status: Accepted, supplements ADR 0161

## Context

ADR 0161 adopted mutation tests on the diff, a property test at `parse_directory`, nextest, the
SBOM, the complexity limit and `cargo hack --each-feature`, and ADR 0165 narrowed the mutation job
to a random part of the diff. Working with these gates, and closing the test gaps of #216, taught
a few things that 0161 does not say. This ADR records them and what follows for this repository.
It repeats nothing of 0161: the diff-only gate, the timeout counting as caught, the missing `.git`
in the copied tree, the survivors closed with tests, the nextest and `cargo test` figures, the
proptest, the SBOM, the threshold of 57 and `hack` stay as written there.

## Findings

- **Exit codes of the mutation job.** Exit code 2 (a mutant survived) and exit code 4 (the baseline
  fails) fail the job. Exit code 3 is the timeout of a mutant and counts as caught (ADR 0161).
- **A changed arithmetic line needs a test that fixes its value.** The lesson comes from
  freemapper-core PR #22, where a test of the value of `water()` showed that `NaN` got through.
  Where a line computes, a test states what it computes; a test that only runs the line is not
  enough.
- **nextest against `cargo test`, as a ratio.** On the whole workspace `cargo test` took 1224 s and
  `cargo nextest run` 470 s, a ratio of about 2.6. Both were measured while a foreign `cargo-mutants`
  process was running on the same machine. The ratio holds up, because both runs had the same load;
  the absolute seconds do not. `cargo test --doc` stays a separate step, since nextest does not
  run doc tests.
- **Gap closing found no bug.** PR #216 added 10 tests (2887 in all at that time) for the replay,
  the manifest validator, the TIFF reader, the MCP outputs and the adapter check. None of them
  failed on the existing code. The tests prove what the code already did; they are not a hunt for
  defects, and a gap closed without a finding is not a wasted change.
- **A defect found here may lie in core.** The app pins `freemapper-*` to one release tag of
  freemapper-core (ADR 0067). A defect that a test here finds in core code is fixed in core and
  reaches the app through a new tag, not through a patch or a workaround in the app. Example: the
  `NaN` in `water()` is fixed in freemapper-core PR #22, released as v0.31.2.

## Decision

- **nextest stays** for the `check` job and for the mutants; `cargo test --doc` and the stub-server
  test binaries keep their `cargo test` step (ADR 0161, 0165).
- **The mutation gate stays on the diff**, and a timeout counts as caught. No whole-workspace run
  becomes a gate.
- **A timed measurement does not run beside a foreign mutation run** on the same machine. Whoever
  measures says in the ADR or the PR how loaded the machine was; a ratio of two runs under the same
  load may be quoted, absolute seconds under unknown load may not.
- **Core fixes arrive by tag.** The pin of the core crates moves to v0.31.2 in a change of its own,
  with the app's tests, not in a documentation change and not as a side effect of another change.

## Consequences

- No gate, no script and no workflow changes.
- The `NaN` fix of core is not in the app until the pin moves; the app does not work around it.
- A future gap-closing change may report "no finding" as a result.

Source: measured in this repository; freemapper-core PR #22 and release v0.31.2.
