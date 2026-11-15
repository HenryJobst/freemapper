# 0151. Ticks of closed and enclosing fences

Date: 2026-11-15
Status: Accepted

Amends [0140](0140-the-training-map-is-a-product-of-its-own.md) (which side a fence's ticks take).

## Context

ADR 0140 lets a closed fence have its ticks inside, then turns the other fences towards olive green, away from a way,
towards a park, or to the side with room. On the Viktoriapark example the user found closed fences with their ticks
outside. Two causes:

- The map writes a ring with the flags close and hole (18) on its last point. `turnable` allowed only the close flag,
  so a closed fence was never turned: its side was the one the source happened to draw.
- A fence round a square whose ends stop 1.5 m apart, and the pieces round a sports ground (an arc round the track,
  three sides of an annex), are not closed. They fell through to the later rules, and the annex took the side of an
  olive green candidate area outside it.

## Decision

- A line may be turned when only its last point carries flags and these are close and hole at most. Turning keeps
  them on the new last point.
- A fence whose ends are no more than 3 m apart and no more than a tenth of its length counts as closed.
- A fence bent round an area has its ticks on its inner side: the area between the line and the line joining its
  ends is at least 0.04 of its squared length (a right angle of equal legs gives 1/8, a half circle about 1/20; a
  line with a slight kink stays undecided). This vote comes right after the closed one, before olive green.

## Consequences

- At Viktoriapark the two closed fences, the square and the sports ground have their ticks inside, on the map and on
  the training map. The goldens of Berlin and Saxony-Anhalt change.
- A fence bent round a garden with olive green outside it now has its ticks in the garden; before they pointed to the
  olive green area.
- Enclosures of several pieces that are each nearly straight are still decided piece by piece.
