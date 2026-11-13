# The extra layers of an adapter: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan step by
> step. Each step is its own pull request.

**Goal:** The engine names no region: an adapter says which extra layers and raster templates it has, and the
engine fetches, stores, lists and removes them in one place.

**Architecture:** First the three hand copies in `project.rs` (Berlin, full ALKIS, trees) become one generic
mechanism in `project/extras.rs`, with no change to the adapter trait. Then the trait grows `extras()` and
`templates()`, the adapters of the states fill them, and the engine reads the registry. Last the questions "is
this Berlin?" go and two general helpers leave the Berlin modules.

**Tech stack:** Rust (workspace of this repository), no new dependency.

**Spec:** `docs/superpowers/specs/2026-11-13-extra-layers-of-an-adapter-design.md`

## Global constraints

- Every output stays byte for byte: `scripts/golden.sh test` passes without a blessed file in every step.
- The files of a project keep their names: `data/berlin/`, `data/alkis-nas/`, `data/trees/`, `templates/k5.png`.
- The project file keeps `berlin_layers`, `nas_layers`, `tree_layers`, `k5`.
- The note ids and their texts stay; the three families keep three sets of ids.
- The draw rules in `build/berlin.rs` stay; `dedup::rank` keeps its signature.
- Each step is announced to the sessions "freemapper-d3" (`build/`) and "freemapper-29" (`project.rs`) and
  reviewed and merged by "freemapper".

## Review focus

1. A layer whose fetch fails while its stored file covers the area: the file and its record stay.
2. A key in the project file that is no layer of the table: no path is built from it, nothing is removed.
3. Offline, in reach, nothing stored: one line says so and nothing is asked.
4. The folder of a family is a link that leads out of the project folder: nothing is written or removed there.
5. A cancelled fetch between two layers: what is on disk and in the project file is what the old code left.

Tests for 1 to 4 are in `project/extras.rs` (4 for a family with a part file and for one without, on
systems with symbolic links); 5 is held by the interrupted fetches of G3.

After the review of step 1 (session "freemapper"): `stored`, `is_stored`, `remove` and the storing of a fetch ask
the table before they touch a file (ADR 0059 is no longer a matter of the callers); each table has one lookup; a
family with a part file cannot be described without its message for a folder that leads out (`Part`).

---

### Step 1: one family in the engine (this pull request)

**Files:** create `crates/freemapper/src/project/extras.rs`; modify `crates/freemapper/src/project.rs`.

**Produces:**

- `extras::Family { dir, known: fn(&str) -> bool, key_arg, part: Option<&'static str>, dated_by: Option<&'static str>, notes: Notes }`
  and the three values `extras::BERLIN`, `extras::NAS`, `extras::TREES`.
- `extras::Fetched { key: String, features: Result<Features, FetchError> }`,
  `extras::Features { features, skipped, regions }`, `extras::Stored { features, fetched }`.
- `extras::path(dir, &Family, key) -> PathBuf`, `extras::remove(dir, &Family, key)`,
  `extras::fetch_into(&Family, dir, near, mode, skip, bbox, previous, progress, fetch) -> BTreeMap<String, AlkisRecord>`,
  `extras::stored(&Family, dir, records, bbox, key) -> Result<Option<Stored>, ProjectError>`.

`fetch_berlin_layers_into`, `fetch_nas_layers_into`, `fetch_tree_layers_into`, `stored_berlin_layers`,
`stored_nas_layers`, `stored_tree_layers` and the three `*_layer_path` keep their signatures and call the generic
functions; the three `remove_*_layer` go. `export.rs` and `build/` do not change.

- [x] Tests of the generic functions first (six, in `extras.rs`), watched failing on stubs.
- [x] The generic functions.
- [x] The three families of `project.rs` as calls; the 301 tests of the module pass unchanged.
- [x] Workspace tests (1918 passed), stub tests, `scripts/golden.sh test` (22 passed): no golden file changes.
- [ ] Pull request, review by "freemapper".

### Step 2: the adapters say what they have

**Files:** `crates/freemapper-adapter-api/src/{adapter.rs,extras.rs}`, `crates/adapters/de/src/{berlin.rs,states/mod.rs}`,
`crates/freemapper/src/project.rs`, `project/extras.rs`, `layers.rs`.

**Produces:** `RegionAdapter::extras(&self) -> &[ExtraSet]` and `RegionAdapter::templates(&self) -> &[RasterTemplate]`
(both default empty). An `ExtraSet` carries what `extras::Family` carries plus the layers for a map kind (key, stored
CRS, `register`), the item of the estimate (text, megabytes per km²) and `fetch`. The engine asks the registry for
the sets of the adapters whose boundary the area with its margin overlaps; `berlin::near_berlin`, the comparisons
with `boundaries::BERLIN` and `nas_services_for(..).is_empty()` as "in reach" become that one question.

Tests first: a registry with a made-up adapter that has one set and one template; the engine fetches, stores,
estimates and removes it without naming it. G2 and G3 unchanged.

#### Step 2 as planned in detail (2026-11-13, after step 1 and its review were merged as PR 62 and 63)

Ruling: only a set that an adapter fetches with code of its own becomes an `ExtraSet` (Berlin). The full ALKIS
and the tree cadastres stay families of the engine fed by `nas()` and `tree_datasets()`: they name no region
today, and a layer of theirs is one file merged from the services of every state the area touches (features
that two services return are kept once), which a fetch per adapter cannot give. Cost if wrong: a second code
adapter that wants a merged family needs the merge in the engine then.

Ruling: the records of the set `berlin` stay in the field `berlin_layers` of the project file (contract with
print and courseplan). The engine maps a set to its records by the set's key; a second set gets a field then.

Sub-steps, one commit each, golden unchanged after each:

- **2a, API and Berlin's adapter.** `freemapper-adapter-api/src/extras.rs`: `trait ExtraSet { info() -> &ExtraSetInfo;
  layers(kind) -> Vec<ExtraLayerInfo>; has(key) -> bool; fetch(fetcher, region_bbox, kind) -> Vec<FetchedExtra> }`,
  `ExtraSetInfo { key, notes: ExtraNotes, estimate: ExtraEstimate { what, megabytes_per_km2 } }`,
  `ExtraLayerInfo { key, crs, register }`, `FetchedExtra { key, features: Result<Vec<Value>, FetchError> }`;
  `RegionAdapter::extras(&self) -> &[Box<dyn ExtraSet>]` (default empty). The api crate gets `freemapper-contract`
  for `MapKind`. `adapters/de`: `berlin::Registers` implements it over `EXTRA_LAYERS`, `extra_layers`,
  `fetch_extra_layers`; `StateAdapter` returns it for Berlin. Tests: Berlin has one set `berlin` whose layers per
  kind are those of `extra_layers(kind)`, `register` is true for the five layers `build/dedup.rs` ranks as
  registers; every other state has none.
- **2b, the engine fetches the sets of the registry.** `project/extras.rs`: `Family` is built from a set
  (`known` becomes a call of `ExtraSet::has`); `fetch_berlin_layers_into` becomes `fetch_extra_sets_into`: for the
  adapters with sets whose boundary the area touches (`touches_region_in`, the rule `near_berlin` is), through
  `asking_for(adapter id, …)`. `stored_berlin_layers` reads through the set. The message for a key that is no
  layer gets a catalog id (review of PR 63). Test: a registry with a made-up adapter that has one set; the engine
  fetches, stores and removes its layers without naming it.
- **2c, the estimate.** `layers.rs` builds the item of a set from `ExtraSetInfo::estimate` in the place and order
  of today's Berlin item (G2 holds it).
2a to 2c are one pull request (done: the trait and Berlin's set; the engine fetches the set of the registry; the
estimate asks the set). `ExtraLayerInfo` carries the key only: whether a layer is a register is a question of
`dedup::rank` about the `freemapper:source` of an object, which names a type inside a file for some layers, so it
is settled in step 3 with the rank. 2d is the next pull request.

- **2d, raster templates.** `RegionAdapter::templates(&self) -> &[RasterTemplate]` for the K5: the WMS, the file,
  the pixel size, the date of the dataset, its item of the estimate; `fetch_k5_into` becomes generic; the record
  stays in `k5`.

2d as built: `trait RasterTemplate { info() -> &RasterTemplateInfo; fetch(fetcher, bbox, root, out) }` with
`RasterTemplateInfo { key, crs, dataset_date, dataset_label, notes: TemplateNotes, estimate }` and
`RegionAdapter::templates()`; `berlin::K5` over `freemapper_sources::wms::fetch_k5`; the engine has
`extras::{template_in, template_path, fetch_template_key_into}`, and `fetch_k5_into` calls it with the key of the
template recorded in `k5` (`K5_TEMPLATE`). A run without the adapter of the template leaves it alone, as for a
set. The constant `BERLIN` of `project.rs` has no user left and is gone. `K5_WMS` and `fetch_k5` still live in
`freemapper-sources`; they move to the adapter crate in step 3.

### Step 3: the engine names no region

**Files:** `crates/freemapper/src/{project.rs,project/export.rs,layers.rs,sources/region.rs,sources/berlin.rs}`,
`crates/freemapper/src/build/{geometry.rs,dedup.rs}` and the `use` lines of the modules that call
`object_polygon`, `crates/freemapper-sources/src/` (the GeoJSON reader).

`touches_berlin` and the constant `BERLIN` of `project.rs` go; the warning about missing buildings is a note id of
the adapter's `RegionInfo`. `parse_layer` and `layer_to_geojson` move to `freemapper-sources`. `object_polygon`
moves to `build/geometry.rs`, unchanged and `pub(crate)`. `dedup::rank` asks the set whether a layer is a
register; its signature stays. `sources::berlin` goes.

Test: `grep -ri berlin crates/freemapper/src` outside `build/`, `#[cfg(test)]` and the note ids finds nothing; a
test in the engine fails when a non-test line names a state id.

#### Step 3 as built (2026-11-13)

- `touches_berlin` is gone: an adapter can name a warning for an area of its ground without buildings
  (`RegionAdapter::no_buildings_warning`), and `derive` asks the regions the area lies on.
- `list_layers` lists the K5 where an adapter the area touches has the template; the key has one name per crate
  (`K5_TEMPLATE`, `berlin::K5_KEY`) and a test ties them.
- `parse_layer` and `layer_to_geojson` are `freemapper_sources::layer_file`. The K5 service, its pixel size and its
  fetch are in the Berlin module; `freemapper-sources` has `png_pieces` and `fetch_png_template` over any WMS.
- `sources::berlin` of the engine is gone. `object_polygon` is `build/geometry.rs`, unchanged.
- `asking_for(id, …)` became `asking(adapter, …)`: no lookup by id and no silent fall back to a fetcher without
  host rules; the sets, the templates and the point clouds go through it.
- A test scans the engine's code that is no test and fails on the id of a built-in adapter or a constant of a
  state outside `build/`, `sources::region::ids` and the key of the set recorded in `berlin_layers`.

Left open, on purpose:

- `dedup::rank` keeps its table of five register layers and the adapter crate its three for the credit. The rank
  goes by `freemapper:source`; how a rank comes to an object is the question of a candidate of the architecture
  review of session "freemapper-d3" ("provenance as a module"), where the table falls away instead of moving.
- `drawn_registers` in `project.rs` still reads the register list of the Berlin module and the source prefix
  `geoportal-berlin/` to tell the credit which registers drew objects. It belongs to the same question.
- The functions and types of the engine that feed the Berlin rules keep "berlin" in their names
  (`stored_berlin_layers`, `fetch_berlin_layers_into`, `Part::BerlinLayers`): they read and fill the field
  `berlin_layers` and the folder `data/berlin`.

### Step 4: records

ADR "The extra layers of an adapter, as built"; a paragraph in `docs/adapters/{en,de}.md` on what a code adapter
can add; the open point of ADR 0082 and of the adapter plan closed; changelog in both languages.

Done: ADR 0114; the paragraph at the end of section 2 of the guide in both languages; the note at C13 in the
adapter plan. The changelog entries came with each step.
