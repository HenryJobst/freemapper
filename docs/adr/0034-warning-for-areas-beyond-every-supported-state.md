# 0034. A warning for areas that reach beyond every supported state

Date: 2026-11-08
Status: Accepted

## Context

The existing border message covers an area whose cells lie in a second supported state. An area in Lower Saxony that
reaches into Bremen got no message at all, although the WMS answers white there and ALKIS has no buildings. The same
gap exists abroad and at sea: the Lower Saxony boundary includes the Wadden Sea, but the Schleswig-Holstein and
Mecklenburg-Vorpommern boundaries end at the Baltic shore.

## Decision

`reaches_beyond_coverage` clips every region boundary near the area to the box plus 50 m and tests points on a 25 m
grid. It is true as soon as one point lies in no region and more than 50 m (`UNCOVERED_M`) from all of them. The
boundaries are simplified separately by up to 25 m each, so a shared border can leave a gap of up to 50 m. A fetch
calls it once for every provider and prints `BEYOND_COVERAGE_WARNING` once, naming abroad, Bremen and sea. The check
runs on the fetch path only, not in the size estimate or the preview.

## Consequences

- Maps at a national border, in Bremen or on a coast now say why the state data are missing there.
- Baltic beach maps get the warning, which is true but new for coastal users.
- A strip of up to 50 m beyond the border is not reported.
- The cost is a few milliseconds per fetch.
- Cost if wrong: users learn of the gap only at fetch, not before; a wider tolerance would hide more.

Source: plan `docs/superpowers/plans/2026-11-08-niedersachsen.md` (Review Focus 1); Task 4 review and ruling; commits
a6a3f18 and b93c503 on branch `niedersachsen`.
