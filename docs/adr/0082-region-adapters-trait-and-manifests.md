# 0082. Region adapters: a trait, TOML manifests and adapter crates

Date: 2026-11-10
Status: Accepted (built in stages A to F by 2026-11-12; what differs from this text is recorded in 0102, 0103,
0106, 0107, 0110 and 0111)

## Context

A state is wired in by hand. The enums `Region` (11 variants) and `Provider` (10) are matched in 14 files, about 190
places outside the tests. NRW touched about 15 files. The user wants other developers to add their own country or
state with little work, while the maintainer keeps only the German states and at most the neighbours. Most of what a
state supplies is already data: URLs, layers, CRS, limits, a credit line. Eight of the eleven height-model sources
need code (tile lists, zip ranges, Metalink, a pinned certificate).

## Decision

- A `RegionAdapter` trait and a `Registry` replace the enums and their `match`es. An adapter gives boundary, CRS,
  OSM extracts, credit, estimate items, and optional ALKIS, raster and extra-layer parts.
- Standard OGC services (WMS, WCS, WFS, OGC API Features, a Geofabrik path) are declared in a TOML manifest, format
  version 1, read from a folder at run time. A manifest names a code `RasterBackend` where the source is not
  standard, so a special state needs one small backend, not a whole adapter.
- Code lives in workspace crates behind Cargo features: `freemapper-adapter-api`, `freemapper-sources` (the generic
  readers), `freemapper-manifest`, `crates/adapters/de`. No dynamic or WASM plugins.
- Manifests are untrusted. Every URL is `https`, port 443, on a host the manifest declares; the fetcher refuses
  internal addresses at connect time. Project-local adapters load only with a flag and never through MCP.
- `freemapper adapter check` runs static checks, a fixtures replay and an optional live run.
- `Crs`, `BoundingBox` and `TileId` move to `freemapper-georef` in freemapper-core, as ADR 0067 planned (done in core
  v0.5.0).
- The migration keeps the example output byte for byte. An injectable clock comes first, then five offline golden
  layers are recorded from v0.11.0 under a fixed date. A CI step on every push rejects a commit that changes a
  golden without the trailer `Golden-Bless:`, and any `refactor(` commit that changes one.
- The recorded answers of the services (whole tiles, 120 to 180 MB) live in a fixture store outside the repository:
  one archive, published as a prerelease of this repository and pinned by SHA-256. The repository keeps the index,
  the request traces and the output hashes. Two states go through the three CI systems before the rest is recorded.
- Before a backend moves, the raster fetch functions get their adapter parameters in place, and the notes with
  their catalog move to the api crate. Built-in providers keep their cache folder names.
- The trait passes the cancel of ADR 0090 through unchanged, and the goldens render notes in English (ADR 0092).

## Consequences

- A new state with standard services is a folder, no rebuild. A state with a special source adds one backend.
- Two of eleven current states become pure manifests, nine manifest plus backend, Berlin stays code. The saving is
  for new states, not for the existing ones.
- Persisted ids (`saxonyanhalt`, `lvermgeosh`) and the record types stay as they are, so old projects load.
- The migration is 34 small tasks. The registry stays process-global so the free functions keep their signatures.
- While the full net stands, a change to a request needs a new recording and a new store version. After the
  migration the request traces and the pipeline hashes stay for three states, one of each kind.
  The tile-pattern kind, `adapter record`, `adapter new`, `adapter boundary` and the weekly live check wait for the
  first outside contributor; one new state (Saarland or Bremen) tests the design before the guide is written.
- Cost if wrong: a manifest format that is too narrow forces a format revision per new `kind`; contributor adapters
  rot unless the weekly live check is kept up; the SSRF rules block a legitimate service on another port.

Revised on 2026-11-11 after a review against v0.10.0 (user's go): the first version recorded the net from v0.9.0, had
no fixed clock, checked pull-request titles only and predated cancel, program language and ADR 0091.

Revised again on 2026-11-11 after a review against v0.11.0 that measured the cache and read the fetch functions
(user's go): the second version planned 2 MB of fixtures per state inside the repository, moved the fetch functions
without changing their parameters, put `Product` into a crate that could not hold it and took the cache folder
names for the provider ids. The spec's decisions D28 to D35 record the changes.

Source: user request of 2026-11-09 (memory "Adapter architecture"); spec
`docs/superpowers/specs/2026-11-10-region-adapters-design.md`; plan `docs/superpowers/plans/2026-11-10-region-adapters.md`;
ADRs 0013, 0016, 0059, 0060, 0067; freemapper-core ADRs 0005, 0007.
