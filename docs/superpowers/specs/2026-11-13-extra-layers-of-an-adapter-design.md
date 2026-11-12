# The extra layers of an adapter: Berlin leaves the engine

Date: 2026-11-13
Status: Draft, for the user's approval. Reviewed by: session "freemapper-d3" (pending).

The user asked on 2026-11-12 for three restructurings left over from the adapter work (ADR 0082). Two are done
(PR 57, the dead code; PR 59, the mouth of the Elbe). This is the third: "Berlin aus dem Kern lösen".

## Why

The adapter work replaced the `Region` and `Provider` enums by a trait, a registry and manifests. One state did
not follow: the engine still names Berlin. Task C13 of the adapter plan was narrowed on purpose ("no `ExtraLayer`
trait and no `extras()`"), because the migration had to keep every output byte for byte and Berlin's layers were
the largest single piece. What that left behind:

- The engine asks "is this near Berlin?" in three ways: `berlin::near_berlin` (fetch of the K5 template and of the
  layers), `touches_berlin` in `sources/region.rs` (a warning in `derive`), and comparisons with
  `boundaries::BERLIN` in `layers.rs` and `project.rs`.
- Three families of extra layers (Berlin's registers, the full ALKIS of five states, the tree cadastres of
  cities) are three hand copies in `project.rs`: `known_*`, `*_path`, `remove_*`, `fetch_*_into`, `Stored*Layer`,
  `stored_*_layers`, `stored_*_objects`, each wired at about fourteen places.
- `berlin::parse_layer` and `layer_to_geojson` serve as the general GeoJSON reader for the full ALKIS, the trees
  and the buildings. `object_polygon` lives in `build/berlin.rs` and is imported by eight build modules.
  `BERLIN_REGISTERS` exists twice with different content (`build/dedup.rs`, `adapters/de/berlin.rs`).

A second city with registers of its own, or a neighbouring country with a layer the simplified cadastre does not
have, would today mean a fourth hand copy and a fourth way to ask "is this near …?". The adapter scheme promises
that a region is a folder and, where needed, a crate.

## Goals

1. The engine names no region. `grep -ri berlin crates/freemapper/src` finds draw rules and tests, and nothing in
   fetch, estimate, derive or the project file code that decides by a region's id.
2. One mechanism for extra layers: an adapter says which layer sets it has, the engine fetches, stores, lists and
   removes them in one place.
3. Every output stays byte for byte: the golden net (G1 to G5) passes unchanged through every step, and the files
   of a project keep their names (`data/berlin/`, `data/alkis-nas/`, `data/trees/`, `templates/k5.png`).

## Non-goals

- The draw rules of Berlin's layers (`build/berlin.rs`, about 2 000 lines, owned by session "freemapper-d3") stay
  rules of the engine, keyed by the file name of a layer as today. They can only leave when an adapter can ship
  rules (`rules.toml` is not read yet, ADR 0107). This design does not start that.
- The project file keeps its fields `berlin_layers`, `nas_layers`, `tree_layers` and `k5`. They are part of the
  contract with freemapper-print and freemapper-courseplan (`freemapper-contract`, fixtures v1).
- No new data, no new layer, no change of a credit line or of an estimate.
- Folder adapters do not get extra layers in this step. The manifest format stays at 1.

## Anti-goals

- A trait so general that Berlin's eighteen layers, the full ALKIS and the trees are squeezed into it by flags.
  If the three do not share a step, that step stays three functions.
- A golden file that has to be blessed. A step that changes one is wrong or belongs to another change.
- A long-lived branch. The area is shared with other sessions (`project.rs`, `build/`).

## What is decided

- The trait method is `RegionAdapter::extras(&self) -> &[ExtraSet]`, with the default empty, as section 3.3 of the
  adapter design names it. An `ExtraSet` is data plus one function: its key (`berlin`, `alkis-nas`, `trees`), the
  folder under `data/`, the layers for a map kind (key, file name), and `fetch(env, area, kind) -> Vec<Fetched>`.
  It answers the design's open question "one trait or two" with one: the three families differ in what they
  fetch, not in how the engine stores, lists and removes them.
- `nas()` and `tree_datasets()` of the trait stay as the data they are; the adapters of the states build their
  `ExtraSet` from them. A caller that needs the service of the full ALKIS still gets it.
- "Near a region" has one meaning: the area, with the margin of the fetch, overlaps the boundary of the adapter
  that has the set. The three ways of asking for Berlin become that one question to the registry.
- The K5 template is a raster template of an adapter: `RegionAdapter::templates(&self) -> &[RasterTemplate]`
  (default empty), with the WMS, the file name and the date of the dataset. The engine fetches the templates of
  the adapters that cover the area; the record stays in the field `k5`.
- The warning about Berlin's missing buildings in `derive` is asked of the adapter: a note id in its `RegionInfo`
  for "this region had no buildings", `None` for all others.
- The GeoJSON reader moves to `freemapper-sources` under a neutral name; `object_polygon` moves to a build module
  that is not named after a state; `BERLIN_REGISTERS` exists once, in the adapter crate.

## How it gets built

Four steps, each its own pull request, each with the golden net unchanged:

1. **One family in the engine.** The three hand copies in `project.rs` become one generic set of functions over a
   small description (folder, records of the project file, note ids). Berlin, full ALKIS and trees are three
   values of it. No trait changes yet. This is the largest step by lines and the one that needs the most care;
   Berlin and Brandenburg are both recorded in G3 and G4.
2. **The adapters say what they have.** `extras()` and `templates()` in the api crate; the adapters of the states
   return their sets; the engine reads them from the registry. The three ways of asking for Berlin collapse.
3. **The engine names no region.** `touches_berlin`, the constant `BERLIN` in `project.rs`, the comparisons with
   `boundaries::BERLIN`, the Berlin items of the estimate in `layers.rs` and the note texts that say "Berlin"
   where they mean "the extra layers of a region". The readers and `object_polygon` move. `sources::berlin` goes.
4. **Records.** ADR for the change as built, the adapter guide (`docs/adapters/`) gets a paragraph on what a code
   adapter can add, the plan section of ADR 0082 is closed.

Step 1 touches `project.rs` only. Steps 2 and 3 touch `build/` by `use` paths and by the place of
`object_polygon`; session "freemapper-d3" sees each pull request before it merges.

## Risks

- `project.rs` is where other sessions work (a forest map kind is in planning with session "freemapper-29"). Each
  step is announced, kept small in time and merged before the next starts.
- The order of fetching, of notes and of saves is behaviour (a cancelled fetch goes on from what is on disk). The
  generic functions keep the order of today; G3 records it for Berlin, and `g3-interrupted` for three states.
- The notes of the three families have different texts. They stay three sets of ids; a generic text would change
  what people read.

## Left to the builder

The names of the generic functions and of the description type; whether `ExtraSet::fetch` returns features or
writes through a sink; where exactly `object_polygon` lands; the order of the sub-steps inside step 1.

## Questions for the user

1. Is the limit right: the draw rules of Berlin stay in the engine until adapters can ship rules?
2. Four pull requests over the shared files, each announced to the other sessions: go ahead in this order?
