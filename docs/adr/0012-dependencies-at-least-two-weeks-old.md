# 0012. Dependencies at least two weeks old

Date: 2026-11-07
Status: Accepted

## Context

Each plan picks crate versions. The plans do not record why the age limit exists.

## Decision

Every dependency version must be at least two weeks old. Where a newer release is younger, the older version is
pinned exactly, in `Cargo.toml` or in `Cargo.lock` for transitive crates. Examples: `rmcp = "=3.6.0"` because
3.7 and later were younger, `rmcp-macros` 3.6.0 instead of 3.9.0, and `mime` 0.3.17 instead of 0.3.18.

## Consequences

- `cargo update` can pull in a too-young version, so a lock file change needs a check against this rule.
- Cost if wrong: fixes from the newest releases arrive up to two weeks later.

Source: plan `2026-11-07-m2-m3a-orthophoto-hillshade.md` (Global Constraints); plans
`2026-11-07-m3b-contours-ndsm.md`, `2026-11-07-m6-mcp-server.md`, `2026-11-07-m8-area-selector.md` (Tech Stack);
M6 rulings (Task 4); M8 rulings (Task 2).
