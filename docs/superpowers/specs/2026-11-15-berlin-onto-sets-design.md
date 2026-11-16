# Berlin onto the sets that bring their rules — design

Date: 2026-11-15. Decision of the user (question round of 2026-11-15): move Berlin's street survey and registers
onto the mechanism of ADR 0149, so that the engine names no region. Every step keeps the output of the Berlin
examples (Viktoriapark, Dammheide) and of the golden net (G1 to G5) byte for byte; a step that cannot is split.

## Where Berlin is named today (inventory of 2026-11-15)

22 places in 14 files of the engine, three kinds:

- **Data a set can declare** (A): the rules (`assets/rules/berlin-candidates.toml`), the source prefix
  (`geoportal-berlin/`, which differs from the set key `berlin`), a rank per layer (survey `sb-…`, registers,
  cadastre), the register layers for the credits (`drawn_registers`, `source_block.rs`), control-site kinds per
  type (`berlin_control_kind`), the tags kept on a candidate (`berlin:layer`, the survey codes), the point-dedup
  table of `dedup.rs` (11 entries), the area that may win a dedup (`bauwerke-flaechen` for 501.5), fountain layers
  and types (`water.rs`), stair layers with a kind filter (`cc_treppe`, `bezbwf`).
- **Berlin's own processing** (B), each to move into the adapter behind a hook or to become a generic step: feature
  tags and currency from a stamped survey date; a point at the middle of a bench line; the kerb notch cleanup; canopy
  areas of a layer by kind; stairs from step-edge lines; the green cut out of paved areas and surveyed paving as
  candidates; shelters under canopies; the building parts as a derived layer; the footprint dates joined from the
  building heights.
- **Plumbing** (C): the project field `berlin_layers`, `REGISTERS_SET`, `stored_berlin_layers`, `Stored.berlin`,
  `Part::BerlinLayers` (which stands for every set), the credits reading only `berlin_layers`.

## Steps, each a PR with unchanged output

1. **Pure moves.** The geometry readers of `build/berlin.rs` go to `build/tagged.rs` with an explicit CRS; the NAS
   stairs get their own "no reprojection" path instead of borrowing Berlin's CRS; the unused `has_stairs` goes.
2. **Data on `SetDrawing`.** Source prefix, ranks per layer, register layers, kept tags, control kinds, the
   point-dedup table, fountain, canopy and stair kinds. Berlin fills them in its adapter but keeps drawing through
   the engine's code, which now reads the data instead of constants.
3. **The rules file** moves into the adapter (`SetDrawing.rules`).
4. **Hooks for Berlin's processing**, one PR each (tags and currency, bench middles, kerb cleanup, canopies,
   step-edge flights, paved cut, shelters, building parts, footprint dates). Generic where another region could use
   it, an adapter hook otherwise.
5. **Plumbing.** Berlin's layers live in `extra_sets["berlin"]` and are written as `[berlin_layers.*]` for old
   project files (a set may name a legacy field); the credits read every set; `Part::BerlinLayers` becomes
   `Part::ExtraSets` with the same label in the estimate.
6. **The switch.** Berlin's `drawing()` returns its drawing; `build/berlin.rs`, `Stored.berlin` and the entries of
   `STILL_NAMED` go. `the_engine_names_no_region` passes with no Berlin line.

## Checks

G4 (`tests/golden/adapters/berlin/expected.sha256`) and the two Berlin examples, rebuilt offline from their stored
data where possible, compared object for object with `scratchpad/sprint/q.py`; G1 (credits) and G5 (project file
round trip) unchanged.

## As built (2026-11-16)

All six steps are on main (pull requests #163 to #185, ADR 0149 addenda). Where the build differs from the plan above:

- **Step 4 was eight pieces, 4a to 4h**: bench middles, kerb notches, roofs under canopies, a set's own feature tags
  (an adapter hook, because Berlin's tagging is its own), canopy areas, the stairs of the cadastre, surveyed paving,
  and building parts with footprint dates (the date join is an adapter hook). Most became data on `SetDrawing`
  (`LayerKinds` lists), not code.
- **Step 5 was 5b and 5c.** The credits (5a) needed no step of their own: once Berlin's records are in
  `extra_sets`, the credit asks for the layer names of every set. `Part::BerlinLayers` is `Part::ExtraSets`.
  The goldens had to name a part by its label first, because a rename of a variant changed them and a refactor may
  not change goldens (golden-guard).
- **Step 6 was 6a and 6b.** 6a drew Berlin through the generic path while its fetch stayed; it needed one datum the
  plan lacked, `id_prefix` (a feature's id is the source id of an object whose feature has none, and Berlin's
  stayed `berlin/…`), and the building parts for every set that names a layer for them, stored layers or not.
  6b set `generic: true`.
- **Left in the engine on purpose:** the test helpers of `build/berlin.rs` (Berlin's rules drawn through the generic
  step) and the K5 template, which is a raster template of the adapter, not a set.
- The golden net was never changed except by the label commit; G3/G4 and G5 held at every step.
