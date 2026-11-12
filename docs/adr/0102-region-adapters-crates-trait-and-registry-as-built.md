# 0102. Region adapters, stages B and C: crates, trait and registry as built

Date: 2026-11-12
Status: Accepted

Carries out stages B and C of [0082](0082-region-adapters-trait-and-manifests.md). Stage A (the golden net) is on
`main` since v0.13.0; the manifests of stage D are still to come.

## Context

ADR 0082 decided that a `RegionAdapter` trait and a `Registry` replace the enums `Region` and `Provider`, and that
the code lives in workspace crates. The plan (`docs/superpowers/plans/2026-11-10-region-adapters.md`) left the
shape of several parts to the tasks that move the code. This ADR records what was built and where it differs from
the first design.

## Decision

- **Four crates.** `freemapper-adapter-api` holds what an adapter and the engine share: the trait, the registry,
  the records of the project file an adapter fills, the errors of a fetch, the notes, the clock and the geometry
  questions. `freemapper-sources` holds the generic readers (WMS, WCS, WFS, OGC API Features, NAS, tree cadastres,
  zip ranges, float GeoTIFFs). `freemapper-adapters-de` (`crates/adapters/de`) holds the eleven German states. It
  depends on the two crates above and not on the engine; `freemapper` depends on it and fills its registry from
  `states::adapters()`. The workspace lists its members as `crates/freemapper*` and `crates/adapters/*`, because
  `crates/adapters` is no crate itself.
- **The trait.** `RegionAdapter` gives `info` (id, provider id, names, CRS, object id prefix, OSM extracts, aerial
  preview), `boundary` and `bounds`, `extract_boundary` (for a region with several OSM extracts), `alkis`,
  `rasters`, `nas`, `tree_datasets`, `attribution` and `estimate`. The engine names no region: the project file
  stores regions and providers as `AdapterId`, written as the same lower-case text the enums wrote. A file that
  names an adapter the program does not have fails with an error that names it.
- **Rasters.** Every provider has a `RasterBackend`: who it is (`ProviderInfo` with the cache folder, the terrain
  source label and whether it serves tiles or fetches per area), the fetch, and `derive_inputs`. The fetch
  functions moved out of `project.rs` without a change to their bodies. `derive_inputs` returns one result per
  input (orthophoto, terrain model, surface model, nDSM point clouds) instead of one result for all, so `derive`
  reports a missing tile at the step it reported it before.
- **What a backend may ask.** `AdapterEnv` carries the fetcher, the registry, the cache folder, the mode, the day
  and the progress callback. The geometry questions (`regions_for_in`, `provider_tiles_in`, `tile_touches_in` and
  the others) are in the api crate and take an `Outline`, so a state module can ask them of its own boundary. The
  boundaries are embedded by the adapter crate.
- **Trust anchors.** A backend names a host that must be verified against bundled certificates
  (`RasterBackend::trust`). The HTTP client pins the hosts the backends of the registry name; the rules of the pin
  are unchanged (exactly that host, no redirects). Today that is the Downloadportal of LVermGeo SH.
- **The clock** moved to the api crate with its fixed variant for tests, because the Rhineland-Palatinate module
  asks it for the age of two kept files.
- **Credit lines.** One loop over the registry gives the lines of `ATTRIBUTION.txt` and of the map sheet. A line
  names the licence text it refers to; the notes give each text once, after the last line that refers to it. The
  old order of the notes differed from the registry order only for states that are no neighbours.
- **Extras.** A region names its NAS service and its tree cadastres as data. The Berlin module moved to the
  adapter crate, but there is no trait for extra layers: the build rules of the engine know the Berlin layers one
  by one, so a trait would only wrap the fetch loop. It comes with the first second region that has code layers.
- **Tests.** A test fails when a source file outside the `tests` folders names the old enums. The tests of the
  tile download and a few that need the test data of the engine stay in `freemapper`; they run against the moved
  code. A switch for test data inside a moved module hangs on the feature `testing`, because `cfg(test)` does not
  reach the tests of another crate.

## Consequences

- A new state is a module in an adapter crate and one entry in its list; nothing in the engine changes. Stage D
  turns most of such a module into a manifest.
- `project.rs` lost the ten fetch functions, the attribution functions and the provider branches of `derive`
  (about 1 200 lines against `main`). `sources/` of the engine keeps `region.rs` (wrappers over the registry), `osm.rs`,
  `alkis.rs` (the merge by object id) and thin modules that re-export the moved ones.
- Old paths such as `freemapper::sources::lgb` re-export the moved modules. They are for the tests of the engine
  and can go when those tests move. (Gone since 2026-11-12: the engine and its tests name
  `freemapper-adapters-de` and `freemapper-sources`; the coded raster backend of Saxony-Anhalt, kept to compare the
  manifest with, went with them.)
- The golden layers G1 to G5 pass unchanged through every step; three were added (`g1-credits.json`,
  `g2-estimates.json`, `g3-estimates.json`).
- The city tree lines keep their old place in the credit list through the Berlin adapter. A second city in another
  state would move them to the adapters that own the cadastres, with a new order to decide.
- `total_megabytes` of an estimate with everything cached is `-0.0` (a sum of no numbers), as in v0.12.0; the
  golden holds it. A fix is a separate change.
