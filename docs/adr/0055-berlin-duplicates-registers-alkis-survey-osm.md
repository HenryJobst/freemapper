# 0055. Berlin duplicates: registers win, then ALKIS, then the survey, then OSM

Date: 2026-11-09
Status: Accepted

## Context

Berlin now has up to four sources for one object: a register (lighting cadastre, Zierbrunnen,
Trinkwasserbrunnen), ALKIS, the street survey of 2014 to 2016 and OSM. ADR 0047 already drops OSM lines near
ALKIS lines, and ADR 0051 OSM trees near cadastre trees.

## Decision

- Order: register > ALKIS > survey > OSM. A register is maintained for its one object type; ALKIS is the
  surveyed cadastre; the survey is older than both; OSM is least reliable.
- Lines (515, 513.2, 518, 410.1): a survey line with at least 80 % of its length within 1.5 m of an ALKIS line
  with the same symbol is dropped. Then OSM lines are compared with ALKIS and the remaining survey lines.
- Stairs: a survey stair covering at least 50 % of the smaller of it and an ALKIS stair area or step-edge flight
  is dropped. There are no OSM stair rules.
- Points: within 2 m of a lamp of the lighting cadastre, OSM `highway=street_lamp` and survey `as_mast` are
  dropped. Within 3 m of a register fountain or drinking fountain, OSM `amenity=fountain` or
  `amenity=drinking_water` and the matching survey points are dropped. 3 m, because a fountain basin is larger
  than a lamp post.
- The winner names each dropped object in `freemapper:also` (ADR 0058).
- A survey wall (515) and an ALKIS retaining wall (513.2) at the same place both stay: different symbols, and
  the mapper decides which applies.
- Register fountains inside drawn water are dropped, like the OSM fountains (ADR 0036).

## Consequences

- Each object shows once, from its best source, and the others stay visible as tags.
- Survey points without a register point nearby are not compared with OSM (furniture is schoolyard-only).
- Cost if wrong: two distinct lamps or fountains within the distance lose the weaker one.

Source: plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md` (Rulings); branch `berlin-plus-2`, Tasks 3
and 4 and their reviews.
