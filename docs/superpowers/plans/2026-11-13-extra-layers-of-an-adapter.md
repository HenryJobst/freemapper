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

Tests for 1 to 3 are in `project/extras.rs`; 4 and 5 are held by the existing tests of `project.rs`
(`*_folder_outside*`, the interrupted fetches of G3).

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
- [ ] Workspace tests, stub tests, `scripts/golden.sh test`: no golden file changes.
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

### Step 4: records

ADR "The extra layers of an adapter, as built"; a paragraph in `docs/adapters/{en,de}.md` on what a code adapter
can add; the open point of ADR 0082 and of the adapter plan closed; changelog in both languages.
