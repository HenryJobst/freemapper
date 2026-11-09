# 0071. Kerb lines of the street survey lose their tree pit notches

Date: 2026-11-10
Status: Accepted

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
