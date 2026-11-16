# 0155. Saarland: the forest road network as candidates, and class 14 of the point cloud left out

Date: 2026-11-16
Status: Accepted

Research: [`docs/research/new-sources-2026-11-15.md`](../research/new-sources-2026-11-15.md), ranking 1 and 11. It builds
on [0147](0147-rhineland-palatinate-basis-dlm-in-the-shape-schema.md) (a second schema for the landscape-model
layers) and [0105](0105-vegetation-from-point-clouds.md) (the point clouds).

## Context

The Saarland serves no landscape model (ATKIS Basis-DLM) as a WFS, so a forest map there had no candidate paths
besides OpenStreetMap. The forest administration's geoportal serves the road network of the forest
(`Forstwirtschaft:Forstwegenetz`, 36 633 lines statewide) as a WFS with GeoJSON output, each line with a road class
(`WKLASSE`, `WEGTYP` 1 to 7): main road for trucks, side road for trucks (also "restricted"), forest road for cars, footway,
machine road, skidding lane. The live check of 2026-11-15 around Saarlouis found 2 358 main roads, 641 side roads, 543
forest roads, 114 restricted side roads, 24 footways, 415 machine roads and 1 499 skidding lanes in 20 by 20 km. A
research pass found a third of the skidding lanes and 60 % of the machine roads in OpenStreetMap, in one or two cutouts
only; the check was not repeated.

The licence is unclear in one point: the catalogue says CC BY 4.0 with the credit "© GDI-SL", the capabilities say "Es
gelten keine Bedingungen". We credit the stricter one.

The Saarland's point clouds carry a class 14 that no document names. On tile 32322-5485, 399 points lie on a line of about
200 m whose height falls evenly from 251 m to 190 m, all of them first to fourth echoes, never last echoes: the pattern of
a wire. The reader counted them as tall vegetation.

## Decision

- **A third schema for a NAS service.** `NasSchema::ForestRoads` (`[[extra.nas]] schema = "forest-roads"`, only with the
  model `basis-dlm`, with `type_name` naming the one feature type) asks the service for GeoJSON
  (`OUTPUTFORMAT=GEOJSON`) and maps each road to the AdV type and codes the rules of the landscape model already read
  ([`nas.rs`](../../crates/freemapper-sources/src/nas.rs), `FOREST_ROAD_CLASSES`):

  | `WKLASSE` | AdV type | codes | forest map |
  |---|---|---|---|
  | Hauptweg LKW | `AX_Fahrwegachse` | `funktion` 5211 | 504 |
  | Nebenweg LKW | `AX_Fahrwegachse` | `funktion` 5212, `befestigung` 1000 | 504 |
  | Nebenweg LKW (eingeschränkt nutzbar), Waldweg PKW | `AX_Fahrwegachse` | `funktion` 5212 | 505 |
  | Fußweg | `AX_WegPfadSteig` | `art` 1103 | 506 |

  A side road for trucks is read as one with a firm surface, because a truck needs one; the data say nothing about it.
  Machine roads and skidding lanes are no paths: they are left out and not drawn, not even as candidates, because no
  symbol of the ISOM names them and a path symbol on a lane would send runners onto it.
- **Both path layers ask one type.** `dlm-fahrweg` and `dlm-weg` each ask the one type and keep their own AdV type, as
  the Shape schema does. A road whose class is not in the table counts as returned (the cap of the service is on the
  answer) but is not kept.
- **Candidates, as for every landscape model.** The rules and the dedup step are unchanged: the roads are candidates of
  the lowest rank where OpenStreetMap has no path within 10 m, on the forest map only.
- **Credit.** The credit line of the Saarland names the forest road network with its licence (`[attribution] text`, and the
  same text in `states/credit.rs`, which a test holds equal), and the note names CC BY 4.0. The line stands on every map of
  the Saarland, a sprint map too: the credit code does not know which layers a project used, and a credit too much is
  harmless.
- **Class 14 is left out for the Saarland** (`PointCloudInfo::left_out = [14]`), so a wire across a forest does not
  count as tall vegetation. The meaning is a reading of the pattern, not of a document; the comment on the reader says so.
  North Rhine-Westphalia's index lists a class 14 too, but none of the tiles measured held it; it stays as it is.

## Not done

- **Geotopes and spoil heaps of the Saarland** (110 points, `TYP` as free text such as "Aufschluß", "Felsklippe",
  "ehem. Steinbruch"; 35 of them rock or cave): a table of free text would be brittle, and no symbol takes an outcrop or a
  spoil heap safely. Not read.
- **Skidding lanes to tell paths from lanes.** The data could tell OpenStreetMap paths that are only lanes from real ones; that is
  another step.
- **Other states' forest networks.** None was found open; another state with the same shape needs one row in
  `FOREST_ROAD_CLASSES` and a manifest block.

## Consequences

- Probe of 2026-11-16, a forest map of 1.1 by 0.8 km near Neunkirchen: 25 roads fetched, nearly all of them already in
  OpenStreetMap (6 map objects name a forest road as a second source, one candidate is new). A sprint map in Saarbrücken
  asks no road, because the layers are for forest maps only.
- A forest map in the Saarland gets tracks and footpaths from the forest administration where OpenStreetMap has none. The
  area page's estimate lists the service.
- The manifest of the Saarland has revision 2. Format 1 stays; a manifest without `type_name` or with `forest-roads` outside
  the landscape model is refused.
- Class 14 of the Saarland's tiles no longer counts for the vegetation density; a few hundred points per tile at most.
