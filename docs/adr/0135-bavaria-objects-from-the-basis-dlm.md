# 0135. Bavaria: objects from the ATKIS Basis-DLM

Date: 2026-11-14
Status: Accepted

Research: [`docs/regions/bayern-reference.md`](../regions/bayern-reference.md), section "ATKIS Basis-DLM".
Design: [`docs/superpowers/specs/2026-11-14-bayern-atkis-design.md`](../superpowers/specs/2026-11-14-bayern-atkis-design.md).
It builds on [0062](0062-full-alkis-from-the-nas-wfs.md) (the full ALKIS from a NAS WFS),
[0063](0063-adv-code-rules-and-a-shared-duplicate-rank.md) (AdV code rules, the rank of a source) and
[0123](0123-bavaria-the-cadastre-from-files.md) (Bavaria).

## Context

Bavaria has no service for its cadastre, so a map there had no fountains, monuments, fences or small
watercourses from an official source. The agency serves its landscape model, the ATKIS Basis-DLM, as a WFS under
CC BY 4.0. The research of 2026-11-14 found less in it than hoped: no cuttings at all, rocks only as named points
(76 in 300 km² of the Fränkische Schweiz), dams only in river plains, and paths that OpenStreetMap mostly has (at
the Schmausenbuck 4 % of the DLM path length has no OSM path within 10 m).

I proposed to leave the source out. The user decided to build it in: more data may come, and a few objects are
better than none. Asked again about the paths, the user chose to take them as candidates where OpenStreetMap has
none.

## Decision

- **One reader, two models.** The service answers in the AAA schema the NAS reader of 0062 reads. A NAS service
  names its model, `alkis` (the default) or `basis-dlm` (`[[extra.nas]] model`), and is asked only for the layers
  of that model. The layers of the landscape model are rows of the same table; their files start with `dlm-`.
  They are stored and recorded beside the cadastre's layers (`data/alkis-nas/`, `nas_layers`), so the project
  file keeps its shape.
- **Ten layers.** Structures (walls, fences, monuments and wayside crosses, fountains), vegetation features
  (named trees), rocks (205), cave entrances (203), springs (312), dams, walls and dikes (105), free-standing
  towers (525 for an observation tower, 524 for the others), the axes of small watercourses (305), and on forest
  maps only the paths and tracks (504 to 507 by function and surface). Church towers, town gates and castle
  towers are parts of buildings and stay out, as do canals, power lines and every area.
- **The lowest rank.** The objects carry `atkis-dlm/<name>` as their source and rank below OpenStreetMap
  (`Rank::Landscape`). The `balanced` profile takes objects from the rank of OpenStreetMap up, so they stay
  candidates; `full` puts them on the map like everything else. The earth walls, cave entrances and rocks
  (105, 203, 205) belong to the content group `slopes`: no other source draws these symbols, and without a
  group `full` would have left them behind.
- **Never a double.** After the other duplicate rules, an object of the model goes where a source of a higher
  rank has the same object: a point within 15 m of a point with the same symbol, a line with 80 % of its length
  within 10 m of a line of the same kind. All path classes are one kind, because two sources seldom give a path
  the same class; the two tree symbols are one kind for the same reason, and so are the two tower symbols
  (OpenStreetMap picks them by height, the model by function). The cadastre, the OSM candidates and
  the OSM objects already on the map count as one list, so a path that follows a candidate for one half and
  an object on the map for the other goes too. The object that stays names the dropped one in
  `freemapper:also`.
- **Its own line in the estimate.** The download estimate and the disk plan name the layers "objects of the
  ATKIS Basis-DLM" and count 0.5 MB per km²; the Schmausenbuck stored 0.08 MB for 1.6 km².
- Bavaria's manifest names the service; the host was already in it. Revision 3.

## Consequences

- At the Schmausenbuck (forest) the service answers 152 objects for the fetch box, 144 of them pieces of
  paths. Joined where pieces of one class meet, they are 61 path lines, 3 fences, a fountain and the
  observation tower. 55 path lines and a fence go because OpenStreetMap has them. The fountain goes in the
  point generalization beside an OSM drinking-water point, and the tower's point lies in the footprint of the
  tower building, where candidate points go (ADR 0076). 14 candidates stay: 8 pieces of fences and 6 paths
  OpenStreetMap lacks. In the old town of Regensburg (sprint) 35 objects give 8 candidates: 3 fountains, a monument, 2 named
  trees and 2 watercourse axes (2 more fountains stand in OSM fountain basins, as of 2026-11-15); the 15 church towers and the town gate are left out by rule.
- A path line of the model (its pieces joined) is kept or dropped as a whole. One that follows an OSM path for less than 80 % of its
  length stays, also on the stretch both have.
- The folder `data/alkis-nas/` and the record `nas_layers` now hold layers that are not ALKIS. The names stay
  for the sake of the project file; the manual says what the `dlm-` layers are.
- Another state can name the Basis-DLM service of its agency in its manifest; nothing else is needed for the
  types of the table.
- A new object type of the model costs a row in the layer table and a rule.

## Left out

Cutting a path of the model where OpenStreetMap covers part of it, the path layers on sprint maps, and using the
surface and function of the model's paths to check the path classes OpenStreetMap gives.
