# 0029. Lower Saxony: rasters via WMS and WCS instead of COG windows

Date: 2026-11-08
Status: Accepted (planned, not yet implemented)

## Context

LGLN publishes its DOP, DGM1 and DOM1 as Cloud Optimized GeoTIFFs found through STAC. Window reads of the DOP COGs
would need a range-backed reader, YCbCr to RGB conversion and mask handling. The WCS values are byte-identical to
the COGs (Lower Saxony reference §5), and the WMS pixels match the tiles (§4).

## Decision

- Rasters are fetched per area: the orthophoto from the WMS in pieces of at most 4000 px, DGM1 and DOM1 from the
  WCS through the shared `WcsService`.
- Dates come from STAC: the newest item at the area's centre.
- nDSM is DOM1 minus DGM1. bDOM20 is not used.
- COG window reads stay a later option.

## Consequences

- No COG range reader and no YCbCr conversion are needed.
- The DOM is often 5 to 9 years older than the orthophoto; freemapper warns with both dates.
- Cost if wrong: orthophotos about three times slower than range reads.

Source: Lower Saxony reference §4, §5, §13, §14; draft plan `2026-11-08-niedersachsen.md` (Rulings), not yet in
the repository.
