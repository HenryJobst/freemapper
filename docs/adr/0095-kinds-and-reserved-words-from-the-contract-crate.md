# 0095. Map kinds and reserved words come from the contract crate

Date: 2026-11-11
Status: Accepted

## Context

ADR 0089 tests freemapper's output against the suite contract, but freemapper still wrote the contract's words itself:
the names `schoolyard` and `sprint`, the scales of a kind, the tag `freemapper:source` with its reserved values, the
role `scale-bar`, the part names and the file name `freemapper.toml`. freemapper-print and freemapper-courseplan had
their own copies. A new kind, such as the gym map, would have to be added in three repositories, and a typo in one of
them showed only in a test after the fact. freemapper-core v0.18.0 holds these words once, in `freemapper-contract`
(core ADR 0027).

## Decision

- `freemapper-contract` becomes a normal dependency of the library, without default features: the program links the
  shared types only and no map reader. The tests add the feature `fixtures`, as before.
- `MapKind` in `project.rs` stays an enum of the kinds freemapper writes, because the code branches on exactly those.
  Its name, default scale and allowed scales come from `freemapper_contract::MapKind` (`contract()`, `name`,
  `from_name`, `ALL`). The MCP server and the area page parse and print kinds through it.
- The writer takes the reserved words from `freemapper_contract::tags`: the source key, the sources `sheet`, `base` and
  `north-lines`, the role key and `scale-bar`, the parts `peripheral information` and `base`, and the sidecar's file
  name.
- Two tests join the contract tests: `contract_kinds_are_the_kinds_of_the_suite` (freemapper writes exactly the kinds
  the contract says it writes, under the same names and scales) and
  `contract_sidecar_reader_reads_a_written_project_file` (the suite's sidecar reader takes kind, title, scale and CRS
  from a project file freemapper wrote).
- All core crates move to the tag v0.18.0 (ADR 0067). The jump from v0.11.0 needed no code change.

## Consequences

- The output is unchanged, byte for byte: the constants have the values the literals had.
- A kind added to the contract with drawing scales fails `contract_kinds_are_the_kinds_of_the_suite` until freemapper
  can build it. A kind added here only fails to compile, because `contract()` has no variant to map it to.
- The `--kind` option of the command line is still a clap enum with the same two names; it converts into `MapKind`.
- Tags of map content (`freemapper:source=osm` and the like) stay literals here. They are freemapper's own and no
  reader depends on them.
- Cost if wrong: return the five constants and three functions to literals and make the crate a dev-dependency again.

Source: freemapper-core `docs/superpowers/specs/2026-11-10-contract-test-design.md` (stage 4), core ADR 0027, the
user's go of 2026-11-11.
