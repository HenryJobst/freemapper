# 0027. Mecklenburg-Vorpommern: orthophoto from the WCS, models masked to the tile set

Date: 2026-11-08
Status: Accepted

## Context

The LAiV M-V orthophoto WMS carries a watermark. The DGM1 and DOM1 from the WCS carry no NoData. A ready nDOM exists but is older. MV was taken before Thuringia because the
Thuringia height-data host was in maintenance.

## Decision

- The orthophoto comes from the DOP WCS as JPEG pieces of at most 4000 px on the grid. The 256-colour palette is
  accepted for a template. The flight date comes from `overview.php`.
- nDSM is DOM1 minus DGM1, both from the WCS. The `WcsService` is shared with Saxony-Anhalt.
- Cells outside the set of 2 km tiles from `overview.php` become NaN. Sea and Bodden at 0.0 m inside the set stay.
- South of the MV tile edge the templates stay empty; there is no fill from LGB.

## Consequences

- The orthophoto WCS is a new source type to maintain next to the WMS.
- Cost if wrong: a cliff at the state edge if the tile list is wrong, and an empty strip in rare MV/Brandenburg
  border maps.

Source: spec §4.1 (Mecklenburg-Vorpommern); plan `2026-11-08-mecklenburg-vorpommern.md` (Rulings); MV rulings;
commits ffbb9a6, cb9fadb.
