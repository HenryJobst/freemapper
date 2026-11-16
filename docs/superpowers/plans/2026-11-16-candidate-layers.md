# Candidate layers (plan)

Part 1 of `docs/superpowers/specs/2026-11-14-training-map-design.md`. ADR 0156 records it.

## Shape

`StarterDocuments.candidates` stays the one document of every candidate (the union). The check print, the
training input, the hidden-candidate rules and the unit tests keep reading it. Only the writers split it: the
OOM and the OCAD export write one file per layer, cut from the union at write time. So the union cannot drift
from the layers.

## Files and signatures

- `crates/freemapper/src/build/candidate_layers.rs` (new)
  - `pub enum CandidateLayer { Terrain, Water, Vegetation, Paths, Barriers, Features, OutOfBounds }`
  - `CandidateLayer::ALL: [CandidateLayer; 7]` in the order of the spec table
  - `fn name(self) -> &'static str` (`terrain`, …, `out-of-bounds`), `fn relpath(self) -> &'static str`
    (`templates/candidates-<name>.omap`)
  - `fn of_group(Group) -> CandidateLayer` (exhaustive match)
  - `fn of_code(&str) -> Option<CandidateLayer>` (fallback by symbol code, by number range)
  - `pub fn split(candidates: &[MapObject], symbol_set: &SymbolSet) -> Vec<(CandidateLayer, Vec<MapObject>)>`:
    the first candidate (map area outline) goes first into every layer, every other object into exactly one
  - `fn layer_of(&MapObject, &Classifier, &SymbolSet) -> CandidateLayer`: group (hidden and candidate-only
    tags ignored), else a canopy 522 to features, else `of_code`, else features
- `crates/freemapper/src/build/content.rs`: `pub(crate) struct Classifier` with `new(&SymbolSet)`,
  `group(&MapObject) -> Option<Group>` and `is_tree_area(id)`, taken out of `route`; `route` keeps its rule that
  hidden and candidate-only objects have no group.
- `crates/freemapper/src/build/canopy.rs`: `pub(crate) fn is_canopy(&MapObject) -> bool` (ALKIS `Überdachung`,
  `Carport`, or a set's canopy layer).
- `crates/freemapper/src/project/export.rs`
  - `StarterDocuments::candidate_layers(&self) -> Vec<(CandidateLayer, MapDocument)>`
  - `build_project`: seven `TemplateRef`s after the rasters, `visible: false`, `opacity: 0.7`
  - `OLD_CANDIDATES_RELPATH` (was `CANDIDATES_RELPATH`)
  - `write_starter_project(dir, documents, MapWrite)` with `enum MapWrite { Write { force }, Keep { old_candidates } }`:
    writes the seven layers; a written map removes `templates/candidates.omap`; a kept map that names it gets it
    refreshed as the union
  - `export_ocad_from`: `ocad/<name>-candidates-<layer>.ocd` each, hidden backgrounds; a replaced OCAD map removes
    `<name>-candidates.ocd`, a kept one gets it refreshed as the union if it exists. `OcadOutcome.candidates:
    Vec<PathBuf>`
- `crates/freemapper/src/project.rs`: `MapOutcome::Kept { old_candidates }`
- `crates/freemapper/src/layers.rs`: `list_layers` lists `candidates-<layer>` seven times in place of
  `candidates`
- CLI: `main.rs`, `pipeline.rs` print the seven OCAD files and the note for a kept map that names the old file
  (`cli-map-old-candidates`, `cli-summary-map-old-candidates`, de and en); MCP tool description of `list_layers`.

## Tests

- `candidate_layers.rs`: every code of every symbol set has a layer; one object per layer lands there (104, 305,
  410.1, 505.1, 518, 531, 520); fallback 401, 501, 522 canopy, 521, 301; a passage 522 goes to paths; the outline
  goes first into every layer; the union of the layers is the candidates, each once.
- `project.rs`: the map lists the seven after the rasters, in order, hidden, opacity 0.7; all seven written for a
  project without candidates; the layer files together hold every candidate of `documents.candidates`;
  `--force` removes `templates/candidates.omap`; a kept map naming it gets it refreshed as the union and
  `old_candidates`; OCAD writes seven files and lists them hidden, `--force` removes `<name>-candidates.ocd`;
  the training map lists no candidate layer.
- `layers.rs`, `tests/mcp.rs`: `list_layers` names the seven layers.
- Existing tests that read `templates/candidates.omap` read the union of the layer files (test helper).
- G4 goldens: map, OCAD and the file list change; the check print, the training map and the PDF keep their
  hashes.
