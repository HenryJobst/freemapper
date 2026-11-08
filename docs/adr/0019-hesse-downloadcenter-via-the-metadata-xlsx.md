# 0019. Hesse DGM1 and DOM1 from the Downloadcenter via the metadata xlsx

Date: 2026-11-08
Status: Accepted

## Context

HVBG offers DGM1 and DOM1 through a WCS and as municipal zips in its Downloadcenter. The WCS `dom1` is a
different surface from the DOM1 download tiles (Hesse reference §6.1). The tiles are grouped by municipality,
so freemapper needs a tile-to-municipality index.

## Decision

- DOM1 comes from the Downloadcenter tiles, and DGM1 uses the same path. DGM1 tiles are identical to the WCS,
  four times faster and dated per tile.
- The orthophoto stays on the WMS, which gives the tiles' pixels, in pieces of at most 3000 px.
- The tile-to-municipality index is the DGM/DOM metadata xlsx, the only index that also dates the tiles. Its URL
  comes from the REST product description and is never pinned. Members are read by HTTP range (ADR 0017).
- If the metadata list cannot be downloaded, the fetch falls back to the kept list.

## Consequences

- freemapper carries an xlsx reader with row, column and cell caps.
- Cost if wrong: one more 1 MB download per fetch, and an index over an undocumented API that may change.

Source: spec §4.1 (Hesse); plan `2026-11-08-hessen.md` (Rulings); Hesse rulings; commits ce1eae5, 91848dd, 5b14eb7.
