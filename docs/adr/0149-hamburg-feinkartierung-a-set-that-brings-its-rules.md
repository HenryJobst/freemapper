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

## Addendum 2026-11-15: Berlin's data on its drawing (step 2a of moving Berlin)

On the way to drawing Berlin through its set too (design `docs/superpowers/specs/2026-11-15-berlin-onto-sets-design.md`),
`SetDrawing` gets the prefix of its sources (`source_prefix`; Berlin's objects carry `geoportal-berlin/<layer>`, its
key is `berlin`), the ranks of single layers (`layer_ranks`, a file or a prefix ending in `*`), the layers its credit
line names when they drew objects (`credited_layers`) and `generic`. Berlin's set returns a drawing with
`generic: false`: the engine reads its ranks and registers from it (`dedup::rank`, `drawn_registers`) but still
fetches, records and draws it by its own code; only sets with `generic: true` take the generic path. The output is
unchanged (G3/G4 byte for byte).

## Addendum 2026-11-15: the point dedups of a set (step 2b)

Which point of a set beats which other point at the same place is data of the set now (`SetDrawing::point_dedups`,
`PointDedup`, `DedupDistance`): Berlin's 13 entries (registers before survey before OSM) moved from `build/dedup.rs`
into its adapter. The engine keeps only its own two entries for the full ALKIS (311 fountains, 526 monuments) and
takes the entries of the sets first, in the order of the registry. In Berlin the ALKIS entries never apply (it has no
NAS service), so the order and the `freemapper:also` notes stay as they were: G3/G4 byte for byte.

## Addendum 2026-11-15: control kinds, kept tags, fountains and bridge decks (step 2c)

Four more things the engine read from Berlin's names are data of a set now: the kind of control site a 531 point
of a layer is (`control_kinds`; a feature may still name its own in `<prefix>:control`, as Hamburg's do), the tags a
candidate keeps besides `<prefix>:layer` (`kept_tags`, Berlin's survey codes), the set's fountain points that stand
in the basin of the water (`fountain_tags`), and the paved areas that may win a duplicate (`winning_areas`, Berlin's
ALKIS bridge decks). A candidate keeps the layer tag of every set first, then the engine's tags, then the kept tags,
so Berlin's tags stay in their order (G3/G4 byte for byte). Hamburg's candidates gain `hamburg:layer`; Blankenese
is rebuilt with the same objects.

## Addendum 2026-11-15: Berlin's rules in its adapter (step 3)

Berlin's candidate rules moved from `assets/rules/berlin-candidates.toml` into its adapter
(`crates/adapters/de/rules/berlin.toml`, `SetDrawing::rules`); `Rules::berlin()` reads them from Berlin's set. The
output is unchanged (G3/G4).
