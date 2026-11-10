# 0089. The output is tested against the suite contract

Date: 2026-11-10
Status: Accepted

## Context

freemapper-print and freemapper-courseplan read the map and the `freemapper.toml` freemapper writes. In v0.9.0 the
scale bar got the tag `freemapper:source=scale-bar`, which neither reader knew, and no test in any of the three
repositories noticed. freemapper-core v0.8.0 now holds the contract as a crate, `freemapper-contract` (core ADR 0018):
rules for the output, and two fixtures with the vocabulary freemapper must keep.

## Decision

- freemapper takes `freemapper-contract` as a dev-dependency with the feature `fixtures`, pinned to its own core tag.
  The crate brings a private `freemapper-omap` and touches none of the core crates freemapper ships.
- Two tests in `project.rs` run with every `cargo test`, for a schoolyard map at 1:1000 and a sprint map at 1:4000 built
  from the offline test project:
  - `contract_output_breaks_no_rule`: `freemapper_contract::check` finds no violation.
  - `contract_vocabulary_matches_the_fixture`: part names, the tags in the parts `peripheral information` and `base`,
    the roles within the peripheral information, the form of the CRS text, the symbol set name and the contract keys of the project file
    equal those of the fixture in core.
- The tests compare no coordinates and no tags of map content. A new data source or a moved object fails nothing.
- A change to the vocabulary is a change to the contract and goes through core first: run the ignored test
  `contract_write_fixtures` with `FREEMAPPER_CONTRACT_OUT` set to `crates/freemapper-contract/fixtures/v1` in a
  freemapper-core checkout, fill in `expected.toml` from the `census.toml` it writes, release core, then move the pin
  here. During that work a `[patch]` to the local core checkout resolves the crate (ADR 0067).

## Consequences

- Tagging the scale bar the v0.9.0 way again fails both tests. That was tried before the merge.
- The test build compiles `freemapper-omap` and embeds about 540 kB of fixture text. The shipped program is unchanged.
- A new part name, a new reserved tag or a new contract key in the project file needs a core release before freemapper's
  tests pass. That delay is intended: the readers learn of the change from the same release.
- Cost if wrong: a rule in core that is too strict blocks a freemapper change until core is released again.

Source: core ADR 0018; the review of freemapper `0f25841`; ADR 0085 and 0086.
