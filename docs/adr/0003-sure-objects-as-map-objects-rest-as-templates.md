# 0003. Sure objects as map objects, the rest as templates

Date: 2026-11-06
Status: Accepted

## Context

The spec names "wrong data presented as fact" as an anti-goal: an uncertain feature that ends up as a map object,
which the mapper then has to find and delete.

## Decision

The user's output policy is "reliable as objects, everything else as template":

- Map objects are ALKIS buildings and, on sprint maps, ALKIS traffic areas and OSM paths with an explicit
  unpaved `surface`.
- Everything else is a template: out-of-bounds candidates, impassable features, school furniture, contours,
  hillshade, slope, nDSM and the orthophoto. Vector candidates go into `templates/candidates.omap`.
- The mapping rules are a TOML data file embedded in the binary. OOM's `OSM-ISSprOM 2019.crt` is the source of
  OSM tag knowledge, filtered by this policy.

The PDF training map is the only exception (ADR 0002).

## Consequences

- The starter map contains few objects. Most of the work stays with the mapper, guided by the templates.
- Cost if wrong in one direction: the mapper redraws features freemapper could have placed. In the other: wrong
  objects in the map, which this policy exists to prevent.

Source: spec §2 (Goals, anti-goals), §4.5; plan-mode notes "Ergebnisanspruch (Nutzer)"; M7 rulings (Task 3,
Final); PDF rulings (Task 5).
