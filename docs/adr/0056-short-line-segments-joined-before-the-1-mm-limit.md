# 0056. Short line segments are joined before the 1 mm limit

Date: 2026-11-09
Status: Accepted

## Context

Candidate lines shorter than 1 mm on paper (`MIN_LINE_MM`, 4 m at 1:4000) say nothing on the map. Plan 2
applies this limit to every non-OSM line, not only to clipped pieces. The cadastre and the survey often cut one
wall or railing into short segments. Each segment alone can be below the limit, so the whole wall would
vanish.

## Decision

- For every non-OSM source (ALKIS layers, street survey, registers), `osm_objects` first joins line features
  that the rules draw as lines with the same symbol and whose ends lie within 0.1 m (`JOIN_TOUCH_M`) with a bend
  of at most about 30° (`JOIN_MIN_COS`). Closed lines are not joined.
- The length limit then holds for the joined line.
- The joined line keeps the tags of its first segment. It names the other segments in `freemapper:also`
  (`<source>/<source_id>`, ADR 0058).
- A joined survey line carries the oldest survey date of its segments as `freemapper:currency`, and
  `freemapper:currency_guessed=yes` when one segment has a guessed date. This is the rule of
  `freemapper:cut_currency` (ADR 0052): the line is only as current as its oldest piece.
- OSM lines are not joined; OSM ways are already whole.

## Consequences

- Walls and railings made of short pieces stay as one candidate line.
- A single ALKIS wall or fence under 4 m is dropped at 1:4000. This changes the Berlin plus 1 output slightly.
- `freemapper:source_id` is the first segment's id; the others are in `freemapper:also`.
- Cost if wrong: two separate walls that meet end to end become one line, which the mapper splits.

Source: plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`; branch `berlin-plus-2`, Task 3 review
(Minor) and fix round, final review (Minor 3).
