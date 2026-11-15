# 0149. Hamburg's Feinkartierung, the first set of extra layers that brings its rules

Date: 2026-11-15
Status: Accepted

Design: [`docs/superpowers/specs/2026-11-15-hamburg-feinkartierung-design.md`](../superpowers/specs/2026-11-15-hamburg-feinkartierung-design.md).
It builds on [0082](0082-region-adapters-trait-and-manifests.md) (region adapters), [0114](0114-the-extra-layers-and-raster-templates-of-an-adapter.md) (the
extra layers of an adapter), [0100](0100-content-profiles.md) (content profiles) and [0074](0074-control-sites-from-street-furniture.md)
(control sites).

## Context

The LGV Hamburg serves two detailed surveys as OGC API collections on `api.hamburg.de`: the Feinkartierung of the
street space (curbs, railings, walls, bollards, benches, lamps, the areas of the street with their use and surface)
and of the public green (fences, handrails, walls, stairs, benches, lamps, play equipment). They are recent (mostly
2021 to 2023) and cover the district and main roads and the public parks. Blankenese alone has 375 street lines,
505 street points, 1 824 street areas and about 600 green objects.

Berlin's comparable layers (Straßenbefahrung, registers) are drawn by code of the engine that names Berlin, the one
exception to "the engine names no region", kept "until adapters bring rules". A second region drawn the same way
would widen the exception.

The user decided (question round of 2026-11-15) that railings, fences, walls and stairs go straight onto the map,
like the other official sources under `balanced`, and that surface materials, bollards and benches become
candidates.

## Decision

- **A set brings its rules.** `ExtraSet::drawing()` (adapter API) returns a `SetDrawing`: the candidate rules as
  TOML text, the prefix of the feature tags, the rank of the set in the duplicate step (`SetRank`) and the layers
  whose areas are stairs. Berlin's set has none and keeps its path.
- **The engine draws it without knowing the region.** The project file records the layers of such sets in
  `extra_sets` (set key → file → record); `fetch` asks every set of the adapters whose region the area touches
  (`extras::drawn_sets`). `build/sets.rs` tags each feature `<prefix>:<property>` and `<prefix>:layer`, draws it
  through the set's rules with the source `<set key>/<file>`, and gives the stair layers to `build/stairs.rs` as
  stair areas in the set's CRS (an ALKIS stair at the same place wins). The duplicate step takes the rank from the
  set; a 531 control site takes its kind from the feature's `<prefix>:control`, which the adapter writes.
- **Hamburg's set** `lgv-feinkartierung` (`lgv_fk.rs`, sprint and schoolyard maps): six layers from six
  collections, each asked once in the region box in EPSG:25832 through its `next` links. Street areas are kept only
  when they are stairs (the surfaces come later), green areas only when they are stairs or walls. Rank `Survey`, as
  Berlin's street survey. Rules: railings, fences, handrails, guard rails → 518; walls → 515; curbs → 501.1 (sprint);
  bollards, benches, lamps → the schoolyard's 544, 543, 545 and a sprint map's 531 control site; play equipment →
  541; monuments and art → 526. Gutters, traffic lights, bicycle stands, manhole covers, boulders and gates are left
  out.
- **Placement needs nothing new.** Under `balanced`, walls, fences and stairs go to the map from rank OSM up, curbs
  (streets) from `Survey` up, furniture never: the user's decision.

## Consequences

- Blankenese (live run of 2026-11-15): 155 railings and fences, 15 walls, 36 curbs and 352 stair lines from 102
  stair areas go onto the map; the OSM stairs at the same places go (110 to 40); 96 control sites become candidates.
- Another state with such a survey needs an adapter set and a rule file, no engine code.
- The street surfaces (water-bound gravel as unpaved candidates) are a second step on the same set.
- The `stand` date of each object is not used; the layers carry their fetch date, as Berlin's registers.

## Addendum 2026-11-15: the green of the street space

The user decided (question round of 2026-11-15) that the street surfaces be candidates on the base map and that the
training map take the sensible ones straight. The street areas whose `inhalt` is green (grass, shrubs up to and from
1 m, hedges, wild growth) form the layer `fk-strasse-gruen`, drawn as 401, 407, 410 and 406; the paving materials,
tree pits and water-bound gravel stay out (the street is a paved area on the map already). Under `balanced` the open
land of a street survey goes to the map (the green strips of Berlin's survey), so `SetDrawing` names
`candidate_layers`: their objects carry `freemapper:candidate-only` and no profile moves them. The training map takes
them as every candidate. Blankenese: 56 such candidates, the map unchanged.
