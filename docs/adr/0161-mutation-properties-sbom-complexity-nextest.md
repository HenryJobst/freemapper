# 0161. Mutation tests on the diff, property tests, nextest, SBOM and a complexity limit

Date: 2026-11-20
Status: Accepted

## Context

ADR 0159 added the housekeeping gates. The next candidates measure what the tests prove, how
fast they run and what ships. Each was measured here first (macOS, Apple silicon, warm `target`)
and is adopted or dropped by that number. The verdicts follow freemapper-print ADR 0072,
freemapper-core ADR 0040 and freemapper-courseplan ADR 0099, and differ where the numbers differ.

## Decision

- **Mutation tests on the diff: adopted.** `scripts/check.sh mutants` runs `cargo-mutants 27.1.0`
  with `--in-diff` over the Rust lines the branch changed against the base branch, in its own CI
  job on pull requests. A whole-workspace run is not a gate; the diff is. `cargo-mutants` copies
  the tree without `.git`: a test that reads the repository's history, or a path above the
  workspace, fails the baseline there (exit code 4) and is a bug in the test. A mutant that
  survives is fixed with a test, not with an exclusion; `mutants.out/missed.txt` names it. A
  mutant that makes a loop endless ends in a timeout (exit code 3), which counts as caught.
  Measured on `parse_directory` and `parse_entries` (60 mutants, 7 min with two jobs): 55
  caught, 2 unviable, 1 timeout, 2 missed. Both misses were real gaps (an entry of exactly the
  fixed 46 bytes, and the step to the next entry over name, extra field and comment) and are
  closed in the same change.
- **Property tests: adopted at one place.** `proptest` checks `zip_range::parse_directory`, the
  reader of a zip central directory that a server sends: whatever bytes arrive, it ends in
  members or in a message, never in a panic, and never lists more members than the end record
  announced. The reader indexes into untrusted bytes, which is where a unit test with three
  hand-made archives does not look. The `.omap` and TOML round trips belong to freemapper-core
  and freemapper-print, where the formats live.
- **cargo-nextest: adopted.** `cargo test` took 1224 s, `cargo nextest run` 470 s on the whole
  workspace, because it runs the test binaries side by side. The `check` job runs
  `cargo nextest run` and `cargo test --doc`; the stub-server test binaries keep their
  `cargo test` step. Pinned to 0.9.152. freemapper-print measured the opposite (11:33 min
  against 8:42 min) and keeps `cargo test`.
- **cargo-hack: adopted as a compile check.** `check --each-feature --no-dev-deps` over the
  features (`golden`, `testing`, `test-stub`) in its own job, 257 s cold. The features exist
  for tests; the powerset would only repeat the combinations.
- **Complexity: adopted as a clippy threshold.** `clippy::cognitive_complexity` with the
  baseline `cognitive-complexity-threshold = 57` in `clippy.toml`; today's worst function
  scores 57 (eleven are above clippy's default of 25). `scripts/check.sh complexity` fails on
  anything above. Lowering the number is welcome, raising it needs an ADR.
- **SBOM: adopted.** `scripts/check.sh sbom` renders a CycloneDX 1.5 list of the CLI with
  `cargo-cyclonedx` 0.5.10 and compares it with `docs/sbom.cdx.json`; `sbom --fix` rewrites it.
  The tool writes a random serial number, a time stamp, the build host and absolute paths; they
  are removed, so the same dependencies give the same bytes. `--target all` lists the
  dependencies of every platform: without it macOS and Linux disagree. The gate needs only
  `cargo metadata` and runs with the cheap ones.

## Consequences

- Three new CI jobs (`complexity`, `hack`, `mutants`), one new cheap gate (`sbom`), and the
  badge for the number of checks counts them.
- A change of `Cargo.lock` changes `docs/sbom.cdx.json` in the same commit.
- Tool versions are pinned in `ci.yml`: cargo-mutants 27.1.0, cargo-hack 0.6.46, cargo-nextest
  0.9.152, cargo-cyclonedx 0.5.10, installed with `cargo install --locked --version`.

Source: measured in this repository; the sibling ADRs for the same tools there.

## Addendum (2026-11-20, ADR 0165)

The mutation job tests a random part of the mutants on twelve runners, each ten minutes after
its baseline, not all of them. See [ADR 0165](0165-mutation-tests-on-a-random-part.md).
