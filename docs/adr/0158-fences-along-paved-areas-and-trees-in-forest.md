# 0158. Fences along paved areas, and single trees in the forest

Date: 2026-11-16
Status: Accepted

Amends [0151](0151-ticks-of-closed-and-enclosing-fences.md) (which side a fence's ticks take) and
[0150](0150-training-map-second-half.md) (what the training map leaves out).

## Context

On the Sanssouci example the user found two faults on the training map:

- A fence along the north edge of a street had its ticks towards the street. The street is a paved area of the
  cadastre (501.5), not a road line. The rule "away from a road or path running along one side" looks at road and path
  lines only, so the fence got no vote and kept the side its source drew.
- In the south west lie many single trees on white ground. The white is a forest area (405) from the point cloud;
  the trees are points of the Potsdam tree cadastre that came too far apart to form a group.

## Decision

- A fence that gets no vote from a closed shape, a bend, olive green or a way is turned away from a paved area
  (501, 501.5, 501.10, 501.11) that lies on one side of it. The side is read one metre from the line, as for olive
  green and parks, because a cadastre fence often lies exactly on the edge of the street.
- The training map drops a tree point (417, 418) whose centre lies in a forest area (405): the white forest is the
  trees, and single ones in it read as open ground. Trees on open land stay. The base map and the candidates keep them.

## Consequences

- Fences along streets drawn as areas change their ticks side on the map and the training map after the next
  `export --force`.
- Training maps have fewer tree points in forest. The summary counts them as left out.
