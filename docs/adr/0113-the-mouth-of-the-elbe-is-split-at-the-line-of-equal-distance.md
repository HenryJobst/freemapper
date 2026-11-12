# 0113. The mouth of the Elbe is split at the line of equal distance

Date: 2026-11-13
Status: Accepted

## Context

The boundaries of the built-in states come from the cadastres of the states (ALKIS, `VerwaltungsEinheit` of kind
`Bundesland`), simplified to 25 m. Off Cuxhaven the cadastres of Lower Saxony and Schleswig-Holstein both hold the
same 89 km² of the outer Elbe. The overlap is in the official data at full resolution (88.986 km²), not a product
of the simplification ([`niedersachsen-reference.md`](../regions/niedersachsen-reference.md), section 10). It is a
lens of open water: its northern edge is the line Lower Saxony claims, its southern edge the line
Schleswig-Holstein claims, and the two lines cross at both ends.

Nothing went wrong because of it. A map area there would have fetched the data of both states, as at any border,
and the measure of shared ground ([0107](0107-adapter-manifests-format-1-as-built.md)) stays below its hundredth:
the lens is 0.55 % of Schleswig-Holstein. It was still the one place where two built-in regions hold the same
ground on purpose of their sources, and the test of the built-in boundaries had to allow for it.

The user decided on 2026-11-13 to give the area to the states, and chose the line of equal distance over giving
all of it to one of them.

## Decision

- Each of the two boundary files loses the part of the lens that lies nearer to the other state's claim line, so
  each state keeps the water nearer to the ground only it holds. The two files share the new line vertex by
  vertex. freemapper takes no side in the question which state the water belongs to.
- The line is built from points every 20 m on the two claim lines (the cells of their Voronoi diagram, joined per
  line), simplified to 10 m and rounded to whole metres: 22 vertices over 23.6 km. Lower Saxony goes from
  51 401.39 to 51 364.58 km², Schleswig-Holstein from 16 059.17 to 16 007.02 km².
- Only the one ring of each file that ran along the lens changes; every other vertex keeps its value and its
  place. The feature gets a property `changed` that says what was done, as the licence of the data (CC BY 4.0)
  asks for changes to be marked.
- The script is [`scripts/elbe-median.py`](../../scripts/elbe-median.py). It checks that both results are valid,
  that the two states together hold what they held before, and that only strips remain in common.

## Consequences

- The two boundaries have 0.05 km² in common, the strips that the simplification left along the Elbe further
  upstream; the largest common area of two built-in states is now far below a thousandth of the smaller one, and
  the test says so.
- At the eastern end the claim lines cross between vertices. The crossing is rounded to a whole metre and the
  last 72 m of the line are straight; against the old union the new one has 202 m² more and 47 m² less, all of
  it water.
- `g1-regions.json` records the SHA-256 of each boundary file; the two values change. No other golden file does:
  none of the 211 boxes of G2 lies in the lens.
- Nobody maps open water. The change shows in `adapters list` and `adapters check` only in that they have nothing
  to say about the two states, as before.
- A later download of the two boundaries from the cadastres brings the overlap back; the script has to run again
  then.
