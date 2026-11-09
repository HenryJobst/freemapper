# 0053. Survey candidates and cut pieces are kept down to 2 m on the ground

Date: 2026-11-09
Status: Accepted

## Context

The paved areas drop slivers with a mean width (2·area/perimeter) under 1 mm on paper, which is 4 m at
1:4000. Park paths and sidewalks are often 3 to 4 m wide. With the paper limit alone, Viktoriapark kept 11 of 30
surveyed paving candidates and lost 3 200 m² of paths. A continuous green verge also cuts a parcel into a
carriageway and a 3.5 m sidewalk, which the paper limit would drop as a sliver.

## Decision

- Survey candidates (501.5 surveyed paving, 401 green strips) are slivers only below min(1 mm on paper,
  2 m on the ground) (`SURVEY_SLIVER_CAP_M = 2.0`), the cap of the wide-water rule (ADR 0036).
- The same limit decides whether a green overlap is cut at all. A 401 candidate is the strip minus the 501.5
  objects and candidates that stay, so none lies on uncut paving: not on an overlap too narrow to cut, and not
  on a `Weg` candidate parcel, which is never cut.
- Remnants of a cut parcel that touch a strip use the same limit. Pieces that touch no strip keep the paper
  limit, as before.
- The plan constants `GREEN_MIN_SIDE_MM` and `SURVEYED_PAVED_MIN_SIDE_MM` stay at 1.0 mm; the cap only lowers
  them on the ground.

## Consequences

- Park paths of 2 to 4 m show as candidates on sprint maps; sidewalks beyond a verge stay map objects.
- Some candidates are narrower than the ISSprOM minimum; the mapper widens or deletes them.
- Cost if wrong: a few narrow pieces to delete.

Source: plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`; branch `berlin-plus-2`, Task 2, review
(Important 1, Minor 3) and fix round 1.
