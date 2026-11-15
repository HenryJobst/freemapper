# 0153. Narrow openings in walls and fences on the training map

Date: 2026-11-15
Status: Accepted

Builds on [0068](0068-overlapping-walls-fences-and-hedges.md) (the norm's opening of walls and fences) and
[0140](0140-the-training-map-is-a-product-of-its-own.md) (the training map).

## Context

On the training map of Viktoriapark the user found a fence round a small pitch that looked drawn wrong at one corner.
In OpenStreetMap the fence (way/708914949) leaves an opening of 1.5 m there, the way onto the pitch. At 1:4000 that
is 0.375 mm: the two ends nearly touch and the corner reads as a slip, not as a way in. ISSprOM 2019-2 §3.3 gives the
opening of walls and fences as 1.0 mm (ADR 0068).

## Decision

- A step of the training level, right after the lines are joined, widens such openings. Two ends of impassable walls
  and fences (515, 518) of one source, the two ends of one line among them, that lie closer than 1.0 mm on paper
  (with the symbol enlargement of the map kind) and at least 0.5 m apart on the ground are an opening.
- Ends of two sources stay: at Viktoriapark OSM and the Berlin survey draw the fences round the sports ground a
  little apart, and a first version that ignored the source opened gaps at their corners that are not there. Gaps
  under 0.5 m are no way through for a runner and stay too.
- Both ends give way along their lines by the same length until they lie 1.0 mm apart; round a corner that is more
  than half of what was missing each. The nearest pairs go first, an end takes part once. A line shortened to nothing
  goes.
- Lines with curves or dashes stay as they are. A shortened line carries `freemapper:generalized` "opening widened to
  the norm (ISSprOM 2019-2 §3.3)"; the report counts them.
- The base map and the candidates keep the measured positions.

## Consequences

- The pitch at Viktoriapark has a visible opening on the training map; the training goldens change where such
  openings exist.
- An opening that is a mistake within one source (two fence pieces that should meet) gets wider too; this concerns
  gaps of 0.5 m up to 4 m at 1:4000. At Viktoriapark a handful of openings are widened, among them the pitch.
- Openings between the end of one line and the middle of another are not widened yet.
