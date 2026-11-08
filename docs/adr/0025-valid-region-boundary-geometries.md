# 0025. Valid region boundary geometries, checked by a test

Date: 2026-11-08
Status: Accepted

## Context

Region detection tests points against the boundary assets (ADR 0013). While preparing Thuringia, five of six
assets turned out to cross themselves or to hold overlapping parts, which can misclassify points near a crossing.

## Decision

Every boundary asset is made valid (`make_valid`), snapped to a 1 m grid, and cleaned of slivers. The test
`tests/region_assets.rs` checks every asset. The MV and NRW plans require the new boundary to pass it.

## Consequences

- The fix changed areas by tens of m².
- Cost if wrong: a point near a bad crossing lands in the wrong region and gets the wrong ALKIS source or raster
  provider.

Source: commits 0fc277c, faba270; plans `2026-11-08-mecklenburg-vorpommern.md` and the NRW plan (Global
Constraints, Region).
