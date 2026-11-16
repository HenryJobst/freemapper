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

## Addendum 2026-11-15: points at the middle of lines (step 4a)

The first of Berlin's own processing steps became a generic one steered by data: a set names in `line_middles` the
lines that give a point at the middle of their extent too, tagged `<prefix>:point=middle` (Berlin's surveyed benches,
lines that a sprint map takes as control sites). `build/sets.rs::with_line_middles` does it for every set; Berlin's
code calls it. Output unchanged (G3/G4).

## Addendum 2026-11-15: notch-free lines (step 4b)

The cleanup of Berlin's surveyed kerbs (a kerb run around a tree pit leaves a hook along the street edge) is
geometry of any surveyed line: it moved from `build/berlin.rs` with its tests into `build/notches.rs`, and a set
names the layers whose lines lose their notches (`notch_free_layers`; Berlin: `sb-bordstein`). Output unchanged
(G3/G4).

## Addendum 2026-11-15: roofs under canopies (step 4c)

The cadastre wins over a set for roofs: a roof (522) of a set that lies at least half under the canopies of the
cadastre is dropped, and the canopy with the largest share names it in `freemapper:also`. That step moved from
`build/berlin.rs` into `build/canopy.rs` (`drop_roofs_under_canopies`), and a set names the roofs that yield
(`canopy_yielding`; Berlin: the surveyed bus shelters, `br_fahrgastunterstand`). Output unchanged (G3/G4).

## Addendum 2026-11-15: a set's own feature tags (step 4d)

How Berlin tags its features is its own: the type a WFS file names per feature, `NULL` as "no value", the id from
one of four fields, the ALKIS names as `freemapper:alkis`, the tree label and crown diameter, and the survey date
stamped at download. No other region needs that combination, so it is an adapter hook rather than a generic step:
`SetDrawing::feature_tags` names the set's function, and Berlin's moved with `survey_currency` from
`build/berlin.rs` into its adapter. Without one, a feature gets `<prefix>:layer` and `<prefix>:<property>` as
before (Hamburg). `build/sets.rs::tags_of` picks between the two. A function pointer cannot be compared, so
`SetDrawing` no longer derives `PartialEq`, which nothing used. Output unchanged (G3/G4).

## Addendum 2026-11-16: canopies of a set (step 4e)

A set names the areas of its layers that are canopies (`canopy_areas`, each a layer, the property that names the
kind and the kinds; Berlin: `bezbwf` `Überdachung` or `Carport` of `bauwerke-flaechen`). `CanopyInput::Set` carries
the set's drawing, the layer, its features and their CRS in place of the Berlin variant. The provenance tags of an
object built from a set's feature directly come from `build/sets.rs::provenance_tags`, which reads the set's own
tags and its source prefix. The map's check for canopies that an older map lacks reads the canopy sources of every
set. Output unchanged (G3/G4).

## Addendum 2026-11-16: stairs of a set's cadastre layers (step 4f)

A set names where its stairs are: the stair areas that rank with the cadastre (`cadastre_stairs`), the step edges
whose parallel neighbours make a flight (`step_edges`), the areas besides the buildings that a flat stair faces
(`facades`) and its surveyed stairs (`survey_stairs`), each a list of `LayerKinds`. Berlin names `Treppe` and
`Freitreppe` of `bauwerke-flaechen`, `Freitreppe` of `bauwerke-linien`, the monuments of `bauwerke-flaechen` and the
`cc_treppe` of `sb-treppe`. `StairInput` takes a `CadastreStairs` (the set's drawing, the CRS and its stored layers)
in place of three Berlin fields. A surveyed stair is tagged by `build/sets.rs::dated_tags`, the generic form of
Berlin's survey tags: the provenance tags with the feature's own date, its `<prefix>:layer` and the kept tags. A
feature without a date of its own gets the fetch date there; Berlin stamps every survey feature at download, so
none lacks one. Output unchanged (G3/G4).

## Addendum 2026-11-16: a set's surveyed paving (step 4g)

A set names the surveyed areas that are cut out of the paved areas of the cadastre (`paving_cuts`; Berlin: the
`ce_gruenflaeche` green strips of `sb-gruenflaeche`), its surveyed paved areas that become 501.5 candidates outside
the traffic parcels (`surveyed_paving`; Berlin: carriageway, sidewalk, cycle track, parking, public square,
pedestrian zone and driveway of `sb-flaechen`), and the tags of the paving that is carriageway (`carriageway_tags`,
for the hidden 501.11 candidates). `StreetSurvey::of_set` reads them from the set's stored layers, tags each area
with `dated_tags` (step 4f), and a cut names the source of the strip's layer. With that, Berlin's survey helpers
left `build/berlin.rs`. Output unchanged (G3/G4).

## Addendum 2026-11-16: building parts and footprint dates of a set (step 4h)

Two ties between a set and the stored buildings are data of the set now. `building_parts` names the layer under
which the set's rules get the building parts of the buildings (`Bauteil` with a `bauart`: passages, arcades;
Berlin: `building-parts`); `build/sets.rs::building_part_features` picks them. `footprint_dates` names the set's
layer that dates the buildings and the adapter's function that joins the dates (Berlin: the 2022 heights,
`gebaeudehoehen`, with `join_footprint_dates`, an adapter hook because the join by `gml_id` is Berlin's). The engine
no longer names the join, so `STILL_NAMED` lost its line. Output unchanged (G3/G4).

## Addendum 2026-11-16: `Part::ExtraSets` (step 5b)

The part of a fetch that holds the sets of extra layers has stood for every set since Hamburg's Feinkartierung, so
`Part::BerlinLayers` is `Part::ExtraSets` now. The label in the estimate and in the messages stays as it was, so the
output does not change. The step that was to move the credits onto every set (5a) goes with the storage of Berlin's
layers in `extra_sets` (5c): only then does a credit read another set's layers. Output unchanged (G3/G4).
