# 0082. Region adapters: a trait, TOML manifests and adapter crates

Date: 2026-11-10
Status: Proposed

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
- `Crs`, `BoundingBox` and `TileId` move to `freemapper-georef` in freemapper-core, as ADR 0067 planned.
- The migration keeps the example output byte for byte. Five offline golden layers are recorded from v0.9.0 first,
  and a CI step rejects goldens edited in a `refactor(adapters):` pull request.

## Consequences

- A new state with standard services is a folder, no rebuild. A state with a special source adds one backend.
- Two of eleven current states become pure manifests, nine manifest plus backend, Berlin stays code. The saving is
  for new states, not for the existing ones.
- Persisted ids (`saxonyanhalt`, `lvermgeosh`) and the record types stay as they are, so old projects load.
- The migration is 37 small tasks. The registry stays process-global so the free functions keep their signatures.
- Cost if wrong: a manifest format that is too narrow forces a format revision per new `kind`; contributor adapters
  rot unless the weekly live check is kept up; the SSRF rules block a legitimate service on another port.

Source: user request of 2026-11-09 (memory "Adapter architecture"); spec
`docs/superpowers/specs/2026-11-10-region-adapters-design.md`; plan `docs/superpowers/plans/2026-11-10-region-adapters.md`;
ADRs 0013, 0016, 0059, 0060, 0067; freemapper-core ADRs 0005, 0007.
