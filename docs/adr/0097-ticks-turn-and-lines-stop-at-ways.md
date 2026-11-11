# 0097. Ticks turn the right way, lines stop at ways, points keep their gap to fences

Date: 2026-11-11
Status: Accepted

## Context

A user reviewing the Biesdorfer Park map found fences whose ticks pointed away from the olive green area they
border, banks whose fall ticks pointed uphill or against the bank beside them, contours across paths and north
lines across paths, and tree and point symbols touching a fence. The ticks of fences (518), earth banks (104, 104.1), impassable cliffs (201) and passable
retained walls (513.2) sit on the right of the line, so the direction of the line is part of the drawing, and the
sources give lines in any direction. Contours and north lines were cut at buildings and water only.

## Decision

- `build/ticks.rs` is the last step on the lines before the documents are assembled. A fence turns its ticks
  towards an olive green area (520) beside it. Else a closed fence has them inside, and a fence along a park (401)
  towards the park. A bank (and cliff, retaining wall) points them down the slope by `data/dgm1.tif`; where the terrain is
  missing or flat it takes the direction of the already decided banks beside it. Lines with curves or dashes are
  left alone.
- `build/line_cuts.rs` cuts contours (101, 102, in the candidates) and north lines (601, in the map) at the paved
  areas and the paths (`way_footprints`, the footprints that already cut tree groups, ADR 0069). Pieces shorter
  than 1 mm on paper go.
- `build/barrier_gaps.rs` moves a point symbol that comes closer to a wall (515), fence (518) or retained wall
  (513.2) than the gap of ADR 0066 (0.15 mm between outer edges, §3.3) away from the line, straight across it to
  that distance. On the tick side of a fence or retained wall the ticks count (0.65 mm, 0.3 mm), so the step runs
  after the ticks are turned. A moved point carries `freemapper:moved`. It also keeps the gap to every other point symbol: the nearest free spot along the line or further away is used; if the same symbol stands there, the moved point is reduced (left out) and the other names it in `freemapper:generalized` (user decision 2026-11-11). ADR 0066 left gaps to
  lines out of scope; this closes it for walls and fences. If no free spot lies within five symbol sizes, the point stays at the gap to the line.
- The norm allows it: ISSprOM 2019-2 §4.6 lets north lines be broken where they obscure small features such as
  path ends (symbol 601). ADR 0043 keeps them map content; only their drawing across ways changes.

## Consequences

- Banks, fences and retaining walls change direction in existing projects after `export --force`. The output of
  the golden net (G4) changes for all eleven states; the commit blesses it (`Golden-Bless:`).
- Contours and north lines have gaps where ways cross them. Trees and other points stand a little off a fence.
- Cost if wrong: the olive-green-first rule for a fence that borders olive green on its inner side; the vote order in `build/ticks.rs` changes in one place.

Source: user review of the Biesdorfer Park, 2026-11-11; ISSprOM 2019-2 §4.6.
