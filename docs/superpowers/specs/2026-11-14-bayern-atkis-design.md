# Bavaria: objects from the ATKIS Basis-DLM (design)

Date: 2026-11-14. Research: [`docs/regions/bayern-reference.md`](../../regions/bayern-reference.md), section
"ATKIS Basis-DLM". Builds on [ADR 0062](../../adr/0062-full-alkis-from-the-nas-wfs.md) (the full ALKIS from a NAS
WFS), [0063](../../adr/0063-adv-code-rules-and-a-shared-duplicate-rank.md) (AdV code rules, the rank of a source)
and [0123](../../adr/0123-bavaria-the-cadastre-from-files.md) (Bavaria).

## Intent

Bavaria has no service for its cadastre, so a map there has no fountains, monuments, fences or ditches from an
official source. The agency does serve its landscape model, the ATKIS Basis-DLM, as a WFS under CC BY 4.0. The
research of 2026-11-14 showed that it holds less than hoped: no cuttings at all, rocks only as named landmark
points (76 in 300 km² of the Fränkische Schweiz), dams only in river plains, and paths that OpenStreetMap already
has (at the Schmausenbuck 96 % of the ATKIS path length lies within 10 m of an OSM path).

I proposed to leave the source out. The user decided on 2026-11-14 to build it in all the same: "Vielleicht kommen
da zukünftig noch mehr Daten. Auch ein paar Daten sind besser als keine Daten."

## Goals

- A map in Bavaria gets the point and line objects of the Basis-DLM that a mapper wants and that the other
  sources lack: caves, springs, named rocks, towers, monuments and wayside crosses, fountains, fences, dams and
  walls, small watercourses, named trees.
- The source is built so that a further object type is one row in a table and one rule, and so that the
  Basis-DLM of another state can use it through its manifest.
- A DLM object never doubles an object another source has.

## Non-goals

- No paths and no power lines from the DLM: OpenStreetMap has them, and the path classes of the forest map depend
  on the OSM tags.
- No areas (forest, heath, quarries, water): the land use files of Bavaria have them.
- No change to the cliff generalisation (`build/cliffs.rs`, ADR 0126).

## Anti-goals

- A DLM object must not go onto the map by the `balanced` profile and must not beat an OSM or a cadastre object:
  the model is made for 1:25 000 and is accurate to about 3 m.
- No map outside Bavaria may change; no golden file of another state may change.
- A failing DLM request must not fail a fetch (as for the NAS layers today).

## What is fixed by the data

| | |
|---|---|
| Service | `https://geoservices.bayern.de/wfs/v1/ogc_atkis_basisdlm.cgi`, WFS 2.0.0, CC BY 4.0, host already in the manifest |
| Answer | the AAA schema (NAS) in GML 3.2 with a default namespace, `wfs:member`, `numberMatched="unknown"`, no paging, default count 500 000 |
| Request | one type per request; a `BBOX` needs `NAMESPACES=xmlns(adv,http://www.adv-online.de/namespaces/adv/gid/7.1)`; the plain URL freemapper builds for a NAS service is accepted as it is |
| Geometry | points and line strings in `position`, EPSG:25832 |
| Date | `lebenszeitintervall/beginnt` of each object |

## Design

### One reader, two models

The answers are the same schema the NAS reader of ADR 0062 reads. So the Basis-DLM is a second **model** of an
AAA service, not a second reader:

- A NAS layer (`freemapper_sources::nas::NasLayer`) names its model, `Alkis` or `BasisDlm`. A service names its
  model too and is asked only for the layers of that model. The manifest key is `model = "basis-dlm"` in
  `[[extra.nas]]`; without it a service is ALKIS, as today.
- The DLM layers are stored beside the others, `data/alkis-nas/dlm-<name>.geojson`, and recorded in `nas_layers`
  of the project file: no new table in `freemapper.toml`, so the suite contract stays as it is. The folder name
  is historical; the manual says so.
- Their objects carry `freemapper:source=atkis-dlm/<name>`, the object id, `beginnt` as the currency and the
  label of the AdV code in `freemapper:alkis`, as NAS objects do.
- Progress and warning lines name the model ("ATKIS Basis-DLM" instead of "ALKIS NAS").

### The layers and their symbols

| Layer | Type | What | Sprint | Forest |
|---|---|---|---|---|
| `dlm-bauwerk` | `AX_SonstigesBauwerkOderSonstigeEinrichtung` | walls, fences, monuments and crosses, fountains | the NAS rules: 515, 513.2, 518, 526, 311 | the same through the aliases |
| `dlm-vegetation` | `AX_Vegetationsmerkmal` | named trees (1011, 1012), hedges | the NAS rules: 418, 410.1 | 418 |
| `dlm-felsen` | `AX_FelsenFelsblockFelsnadel` | rock, boulder, rock needle (a point) | 205 | 205 |
| `dlm-hoehle` | `AX_Hoehleneingang` | cave entrance | 203 | 203.2 |
| `dlm-gewaessermerkmal` | `AX_Gewaessermerkmal`, `art` 1610 | spring | 312 | 312 |
| `dlm-damm` | `AX_DammWallDeich` | dam, wall, dike (a line) | 105 | 105 |
| `dlm-turm` | `AX_Turm` | 1003 observation tower: 525; 1001, 1004 to 1008, 1010, 1011: 524 | 525, 524 | 525, 524 |
| `dlm-gewaesserachse` | `AX_Gewaesserachse` | small watercourse; `funktion` 8300 is a canal by the code list and is left out | 305 | 305 |

Church towers, town gates and castle towers (1002, 1009, 1012) are parts of buildings and are left out. A rock is
a point in the data; the peer session that owns the cliffs agreed that DLM rocks stay candidates and that
`build/cliffs.rs` is not touched.

### Rank and duplicates

A new lowest rank, `Landscape`, for `atkis-dlm/…`. The `balanced` profile puts objects on the map from the rank
of OpenStreetMap up, so DLM objects stay candidates; `full` puts them on the map like everything else.

A new step after the existing duplicate rules: a DLM point is dropped when an object of a higher rank with the
same symbol lies within 15 m; a DLM line when 80 % of its length lies within 10 m of such a line. The object
that stays names the DLM object in `freemapper:also`. The distances are wider than the cadastre's 1.5 m because
both sides are only a few metres accurate.

### Registration and texts

- Manifest of Bavaria: one `[[extra.nas]]` block with `model = "basis-dlm"`, the `namespaces` value, revision 3.
- ADR 0133. Manual (German and English), README, the adapters guide and the manifest reference (the key `model`),
  the glossary (ATKIS, Basis-DLM), changelog, reference. User texts go through `unslop`.
- Both Bavarian examples are rebuilt.

## My calls (not the user's)

- Which eight layers, and that paths, power lines and areas stay out.
- The rank below OpenStreetMap and the distances 15 m and 10 m.
- Storing the layers in the NAS folder and records instead of a new table in the project file.
- 205 for a rock point, 105 for a dam.

## Left to the builder

Names of private helpers, the layout of tests, and whether the model lives in `freemapper-sources` or
`freemapper-adapter-api`.
