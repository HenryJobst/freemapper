# 0156. The candidates in seven layers

Date: 2026-11-16
Status: Accepted, amends ADR 0003, 0010 and 0140

## Context

Up to 0.23.0 every candidate lay in one template map, `templates/candidates.omap` (ADR 0003, ADR 0010), which the
map listed as visible at 70 % opacity. On the Bremen Bürgerpark forest map and the Viktoriapark sprint map that file
holds thousands of objects of every kind: switched on, it covers the map, and switched off, it shows nothing. A mapper
checks one kind of object at a time (the fences of a park, then its paths), and OpenOrienteering Mapper switches
templates on and off and sets their opacity one by one.

The user decided on 2026-11-14 (spec `docs/superpowers/specs/2026-11-14-training-map-design.md`, decisions 2 and 4)
that candidates of one kind go to separate layers, as separate template files, and that the layers start hidden:
"Layer sollen beim Öffnen ausgeblendet starten, sonst ist die Karte überladen. Initial nur die sicheren Objekte
(Grundlayer)."

## Decision

- Seven layers, by the colour families a mapper thinks in, each a template map of its own
  (`build/candidate_layers.rs`):

  | Layer | File | Groups | Symbol codes of no group |
  |---|---|---|---|
  | terrain | `templates/candidates-terrain.omap` | slopes, landforms, contours | 1xx, 2xx |
  | water | `templates/candidates-water.omap` | water-features | 3xx |
  | vegetation | `templates/candidates-vegetation.omap` | vegetation, trees, hedges | 4xx |
  | paths | `templates/candidates-paths.omap` | paths, streets, stairs, passages | 501 to 509, 512, 532 |
  | barriers | `templates/candidates-barriers.omap` | walls, fences | 513 to 519 |
  | features | `templates/candidates-features.omap` | monuments, furniture | the other 5xx, 6xx to 9xx |
  | out-of-bounds | `templates/candidates-out-of-bounds.omap` | out-of-bounds | 520 |

  The layers follow the colour families first: every object of a vegetation code (4xx) goes to vegetation
  whatever its group (ruling of 2026-11-16 in the review of PR #180). So the green strips of the Berlin street
  survey, open land 401 of the group streets, are vegetation, not paths. Every other object goes by its group, the
  one the content settings route it by (ADR 0100), read whatever its tags `freemapper:hidden` and
  `freemapper:candidate-only` say. A canopy (522 with the ALKIS kind `Überdachung` or
  `Carport`, or a shelter of a set that gives way to them) goes to features before its group is asked: the code
  puts every 522 in the group passages, and a roof is no way. A test checks that every code of the three symbol sets
  at every scale has a layer.
- The build keeps the candidates as one list (`StarterDocuments.candidates`). Only the writers cut it, so every
  candidate is in exactly one layer, in its order, and the layers together are that list. The check print, the
  training map and the hidden-candidate rules read the list as before, and their output does not change. The
  outline of the map area goes first into every layer, so each one shows the area on its own. `split` finds it by
  its symbol (501.1, on the forest map 994) and its source OpenStreetMap, not by its place in the list.
- All seven are written every time, an empty one too: the template list of a map never depends on what a run found.
  The map lists them after the raster templates in the order of the table, each hidden and with an opacity of 0.7
  for when it is switched on. The raster templates keep their settings. The training map lists none (ADR 0140).
- `templates/candidates.omap` is no longer written for a map that is written. A written map deletes one left by an
  older version; the backup `.omap.bak` of the replaced map then names a file that is gone. A kept map is never
  edited: when it names `templates/candidates.omap`, the run fills that file with every candidate, writes the layers
  beside it and prints that the map lists the layers after `--force`, or when they are added in OpenOrienteering
  Mapper (`MapOutcome::Kept { old_candidates }`, `cli-map-old-candidates`).
- OCAD gets `ocad/<name>-candidates-<layer>.ocd`, one per layer, in place of `<name>-candidates.ocd`, listed in the
  OCAD map as hidden background maps. A replaced OCAD map deletes `<name>-candidates.ocd`. A kept one that names it
  among its background maps (the type 8 strings, read by `ocd::file::background_names`) gets it filled with every
  candidate and the note `cli-ocad-old-candidates` (`OcadOutcome::old_candidates`), as the OOM side decides by the
  text of the map. A backup restored after `--force` therefore gets its candidates file back on the next run. OCAD layers inside one file were not looked into: the OCAD output has not
  been checked in OCAD yet.
- The MCP tool `list_layers` names the layers `candidates-terrain` to `candidates-out-of-bounds` in place of
  `candidates`; `preview_layer` has no preview of a candidate layer, as it had none of the candidates.

## Consequences

- A mapper sees the map's sure objects on opening and switches on one kind at a time. Someone used to the visible
  candidates of 0.23.0 sees none until they switch a layer on; the manual says where.
- The map file has six templates more, and the project folder six files more. The outline is written seven times.
- The golden net (G4) changes in the map, the OCAD map and the file list of its four states. The check print, the
  training map and its PDF keep their hashes.
- freemapper-print and freemapper-courseplan read no template, and the contract test of freemapper-core does not
  look at the template list; their fixtures and tests name the old file as maps of 0.23.0 do, which stay valid.
- The examples still have `templates/candidates.omap` until they are rebuilt (spec, part 4); their overview names
  that file as long as they do.

Source: spec `docs/superpowers/specs/2026-11-14-training-map-design.md`, "Part 1: candidate layers"; plan
`docs/superpowers/plans/2026-11-16-candidate-layers.md`.
