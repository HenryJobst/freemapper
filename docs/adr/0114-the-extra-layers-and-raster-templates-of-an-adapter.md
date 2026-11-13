# 0114. The extra layers and raster templates of an adapter, as built

Date: 2026-11-13
Status: Accepted

Design: [`2026-11-13-extra-layers-of-an-adapter-design.md`](../superpowers/specs/2026-11-13-extra-layers-of-an-adapter-design.md),
approved by the user on 2026-11-13; plan beside it. Built in pull requests 62 to 66.

## Context

The adapter work ([0082](0082-region-adapters-trait-and-manifests.md)) replaced the enums of regions by a trait, a
registry and manifests, and its design named a method `extras()` for "Berlin layers, NAS, tree cadastres". The
migration narrowed that on purpose (task C13 of the adapter plan, "no `ExtraLayer` trait and no `extras()`";
[0102](0102-region-adapters-crates-trait-and-registry-as-built.md) records stages B and C as built):
every output had to stay byte for byte, and Berlin's layers were the largest single piece. So one state stayed in
the engine. The engine asked "is this near Berlin?" in three ways, fetched Berlin's registers and its K5 map by
calling the Berlin module, held three hand copies of "fetch layers into a folder, keep, remove, read back" for
Berlin, the full ALKIS and the tree cadastres, and `freemapper-sources`, which should know no state, held the
WMS of the K5.

The user asked on 2026-11-12 to take Berlin out of the engine.

## Decision

- **One mechanism for stored layers.** `project/extras.rs` fetches the layers of a family into its folder, keeps
  the file of a layer that could not be fetched while it covers the area, removes what is no longer used and
  reads a layer back. A family is described by its folder, its table of keys, its message ids, whether its
  answers stream through a part file and which property dates a feature. A file is read, written or removed only
  under a key of the table ([0059](0059-project-files-never-name-paths.md) holds by the functions, not by
  their callers).
- **A set of extra layers is something an adapter fetches with code of its own.**
  `RegionAdapter::extras() -> &[Box<dyn ExtraSet>]`. A set says its key (the folder `data/<key>`), the CRS of its
  stored features, its message ids, its item of the download estimate, the layers of a map kind, and it fetches
  them. Berlin's registers and surveys are the one set there is.
- **The full ALKIS and the tree cadastres are no sets.** They stay families of the engine fed by the data the
  adapters name (`nas()`, `tree_datasets()`). They named no region before, and a layer of theirs is one file
  merged from the services of every state the area touches, with a feature that two services return kept once. A
  fetch per adapter cannot give that.
- **A raster template is something an adapter fetches with code of its own.**
  `RegionAdapter::templates() -> &[Box<dyn RasterTemplate>]`: the key (the file `templates/<key>.png`), the CRS it
  is served in, the date of its dataset, its message ids, its item of the estimate, and its fetch. Berlin's K5
  map is the one there is. `freemapper-sources` keeps a general fetch of a map as PNG pieces from any WMS; the
  service of the K5 is a constant of the Berlin module.
- **In reach means one thing.** A set or a template is fetched for an area that touches the region of its
  adapter (the rule by which an area needs a region), through the hosts the adapter is held to.
  `adapters::asking(adapter, …)` is the one way to ask for an adapter; it takes the adapter, so no id can miss
  the registry and be asked without host rules.
- **A run without the adapter leaves its things alone.** When the registry of a run has no adapter with the set
  or the template (the check of one adapter on its own), nothing of it is asked, read or removed.
- **The project file does not change.** The records of the set stay in `berlin_layers`, the record of the
  template in `k5`; the engine maps a field to its set or template by key. The files keep their places
  (`data/berlin/`, `templates/k5.png`).
- **A region can warn of an area without buildings** (`RegionAdapter::no_buildings_warning`), which replaces the
  one question about Berlin that `derive` asked.
- **A test holds the result.** It reads the engine's code that is no test, outside the draw rules and the two
  modules of names the tests go by, and fails on the id of a built-in adapter, on a constant of a state, and on
  any path into the adapter crate. What still reaches there is a list in the test, line by line with its
  reason; a new line fails.

## Consequences

- No output changed: the golden net passed every step without a blessed file. Maps, messages, estimates and
  project files are what they were.
- One behaviour changed, and it is a fix. Before, the engine took Berlin's boundary and Berlin's fetch whatever
  the registry held: `adapters check --live` of another adapter near Berlin fetched Berlin's layers and the K5
  without the host rules of Berlin's adapter, and the next command dropped the records. Now such a run asks
  nothing.
- A second city with registers of its own is an `ExtraSet` in its adapter and a field in the project file. The
  field is a change of the contract with freemapper-print and freemapper-courseplan, so it needs their reading.
- Only code adapters can have sets and templates: keys and message ids are static texts, and a folder brings no
  code. The manifest format stays at 1.
- The keys of a set and of its layers become a folder and a file. The engine takes neither on trust: plain names
  only.

## Not done, and why

- **The draw rules of Berlin's layers stay in the engine** (`build/berlin.rs`), keyed by the name of a layer.
  They can leave when an adapter can bring rules; `rules.toml` is not read yet
  ([0107](0107-adapter-manifests-format-1-as-built.md)).
- **Which layers are registers** is still said twice: a table of five in `build/dedup.rs` for the rank of an
  object, and a list of three in the Berlin module for the credit, which `drawn_registers` in `project.rs` reads
  together with the source prefix `geoportal-berlin/`. The rank goes by the `freemapper:source` of an object. How a
  rank comes to an object is an open question of the architecture review (the rank as a datum of the layer); it
  removes the table instead of moving it.
- **Names.** The functions of the engine that feed the Berlin rules keep "berlin" in their names. They read and
  fill the field `berlin_layers` and the folder `data/berlin`.
- **What the engine still takes from the adapter crate by name**, as the list of the test has it: the table of
  Berlin's layers for the inputs of the draw rules; the tile download and the tile names of the tile providers
  (`tile_fetch`, `lgb::ProductTiles`), which `fetch.rs` re-exports for its tests and `derive` reads; the endpoints
  of the cadastre of two states in `sources/alkis.rs`. None of them decides by a region; each is a move of its
  own.
- **The tree cadastres look their adapter up by id** (`sources/trees.rs`) and would ask without host rules if the
  registry did not have it. The pattern that `adapters::asking` removed for sets, templates and point clouds is
  still there for them.

Update 2026-11-13: the tree cadastres no longer look their adapter up by id. `sources/trees.rs` walks the
registry and keeps each dataset with the adapter that brings it; whether an area needs the dataset goes by the
ground of that adapter, and the request goes through `adapters::asking` with it. The region a row names only
fills the record of what was fetched. The point above names the wrong risk: with an id the registry did not hold,
the dataset was left out and nothing was asked. A row that named another adapter of the registry would have been
asked for that adapter's ground and under its hosts. No adapter of today was affected: a manifest writes its own
id into the row.
