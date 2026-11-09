# 0071. Kerb lines of the street survey lose their tree pit notches

Date: 2026-11-10
Status: Accepted, amended 2026-11-10

## Context

User feedback on the Viktoriapark training map (2026-11-09): the edge of the street along the park looks odd
around the trees. A small hook sits at every tree of the row, and the tree point lies at its tip.

The cause is in the kerb lines (`bd_bordstein`, 501.1 candidates, ADR 0054). Berlin's street survey draws the
parking strip and the sidewalk beside the carriageway. Each street tree stands in a slot cut across that strip,
about 2 m wide and 4.5 m long, and the surveyed kerb runs around the slot. The kerb line leaves the street edge,
follows the slot and returns. The training map draws every candidate, so each slot shows as a hook. The map
file itself is clean: the street is one 501.5 area from ALKIS (ADRs 0052 and 0053), and tree pits are not cut out of it
(the type list in `build/paved.rs` leaves them alone). The hooks are only in
the candidates and in the PDF.

On the Viktoriapark sheet the survey kerbs hold 6,522 vertices in 81 lines. A scan for detours of this shape
finds 34, of which 27 enclose less than the limit below, mostly the pits in question.

What the norm says: ISSprOM 2019-2 §3.3 gives 1.0 mm² as the smallest area of a brown dot screen and 0.4 mm as
the smallest width of a colour screen (1.6 m at 1:4000). The slot is 0.5 mm wide, so the width rule does not
remove it, and the norm has no notch rule. A slot with its lead-in still encloses 0.7 to 1.1 mm² at 1:4000, the
size of the smallest screen the norm allows to stand alone.

## Decision

`build/berlin.rs` (`drop_kerb_notches`) cleans each kerb line of the survey layer `sb-bordstein` after the rules
have made it a candidate:

1. A detour of the line is replaced by its chord when it returns to the line within 10 mm on paper, runs at
   least twice as far as its chord and encloses less than 1.5 mm² on paper (24 m² at 1:4000; 6 m² at 1:2000).
   The limit is the 1.0 mm² minimum of ISSprOM plus half a square millimetre, because the surveyed pits with
   their lead-in reach 1.1 mm². The ratio keeps curves: a quarter circle runs 1.1 times its chord.
2. A closed kerb ring that encloses less than 1.5 mm², the kerb around a lone pit, is dropped.
3. A line with curve points or other coordinate flags stays as it is.

The limits are in millimetres on paper, so the rule follows the scale. Larger bays (a 12 m by 6 m parking
pocket is 4.5 mm² at 1:4000) stay, and so do kerb lines that merely bend.

## Consequences

- On the Viktoriapark sprint map the survey kerbs lose 459 of 6,522 vertices (81 lines remain), and the hooks
  along the street edge are gone. Trees stay where they are as points.
- Only the candidate layer changes. The map has no kerb lines, and the street polygon and its green cuts are the
  same. Existing projects get the cleaner lines with `run --force`.
- Real kerb detours below 1.5 mm² on paper vanish at every scale, also pits that are no trees. At 1:4000 that is
  24 m², about a parking space. A mapper who wants one in the map draws it by hand.
- Survey polygons of parking strips still carry the slots, but they have no outline and lie on the same 501.5
  fill, so they are not visible.

## Amendment 2026-11-10: wide bumps and ring flags

The first rule missed two things on the Viktoriapark example, and the user still saw bumps at some trees.

- Wider pits. Some trees stand in a slot that the survey draws as a trapezoid: the kerb leaves the edge on a
  slant, rounds the tree and returns on a slant (Li_13.04.2015_719). It has a chord of 1.7 mm, a way of 3.2 mm
  and encloses 1.3 mm² at 1:4000. The way is 1.8 times the chord, below the factor 2, so it stayed. A factor
  cannot separate such a pit from a curve well, because a semicircle runs 1.57 times its chord. The depth does:
  a pit reaches 0.4 or more of its chord, a quarter-circle corner 0.2.
- Rings. The builder marks the last coordinate of a closed ring with the close flag. Rule 3 then took every ring
  for a line with curves, so no ring was ever dropped. The pill-shaped kerb rings around lone pits (1.8 mm by
  0.8 mm) stayed on the map.

Rule 1 now reads: a detour is replaced by its chord when it returns within 10 mm of way and 2.5 mm of chord
(10 m at 1:4000), reaches out at least 0.3 of its chord from the chord and encloses less than 1.5 mm². The chord
limit keeps a long gentle deviation, which a depth share alone would flatten. Rule 3 looks for curve start flags
only. In the example the survey kerbs go from 6,522 to 5,105 vertices and from 81 to 76 lines.

## Amendment 2026-11-10 (2): a notch returns to the line

The depth rule of the first amendment let a detour end anywhere. In the Viktoriapark it cut from the last vertex
on the street edge to the apex of a slot (Li_15.04.2015_251) and drew a spike at that tree. Next to the X mark
it cut across a step of the kerb, where the kerb moves back by 5.7 m, and drew a diagonal instead of the
survey's corner. The user saw both on the street along the park.

A detour now also has to lie on a straight run: the kerb 0.5 mm before its start and 0.5 mm after its end
points within 20 degrees of its chord. A side that has no point that far away (the start or end of a line, where
the survey splits the kerb at a pit) is not tested. A spike chord is 31 degrees off the edge and a step chord
54 degrees, a pit chord under 15. Tests carry the real geometry of both places. In the example the survey kerbs
have 5,787 vertices in 76 lines.
