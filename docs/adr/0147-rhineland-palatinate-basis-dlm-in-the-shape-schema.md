# 0147. Rhineland-Palatinate: the landscape model in the AdV Shape schema

Date: 2026-11-15
Status: Accepted

Research: [`docs/research/states-plus/STATES-PLUS-SURVEY.md`](../research/states-plus/STATES-PLUS-SURVEY.md), row
Rhineland-Palatinate. It builds on [0135](0135-bavaria-objects-from-the-basis-dlm.md) (the landscape model from a
NAS WFS) and [0062](0062-full-alkis-from-the-nas-wfs.md) (the NAS reader).

## Context

Rhineland-Palatinate has no full ALKIS service, only the simplified one. Its landscape model, the ATKIS
Basis-DLM, is open (dl-de/by-2-0), but not as NAS: the LVermGeo RP serves it as a WFS in the AdV Shape schema
(`basisdlm_shape_rp.fcgi`). That schema has one type per theme and geometry (`Relief_REL01_P`,
`Verkehr_VER02_L`, …), each holding several AdV types; the AdV type is the property `objart_txt`, the object id
`objid`, and the codes have short names (`bwf` for `bauwerksfunktion`, `fkt` for `funktion`, `bef` for
`befestigung`, `bws` for `bewuchs`, `nam` for `name`). The geometry is the property `msGeometry`, in GML 3.2. The
other ways to the same data are a Shape zip of 336 MB and a GeoPackage of 1 GB for the whole state.

The live check of 2026-11-15 over the two examples found the types the landscape layers of 0135 use: rocks,
dams, water axes, springs, towers, structures, vegetation features, tracks and paths. Around Dahn (3.4 by 3.1 km)
it holds 240 track and 127 path axes, 34 water axes, 6 rocks and 4 springs.

## Decision

- **A schema per NAS service.** `NasService` gets `schema`: `Nas` (the default) or `Shape`; the manifest key is
  `[[extra.nas]] schema`, and `shape` is allowed only with `model = "basis-dlm"`, because the table of Shape
  types exists only for the landscape layers.
- **The layers stay.** Each `dlm-…` layer names the Shape types its objects can be in (`NasLayer::shape_types`).
  A Shape service asks each of them for the layer and keeps the features of the layer's AdV type. Some Shape
  types are asked for two layers (`Relief_REL01_P` for rocks and cave entrances, `Verkehr_VER02_L` for tracks and
  paths): about 18 requests for a sprint map, 20 for a forest map, each small. A cache of answers by URL would save
  them, but would hold whole answers in memory; not done.
- **One reader.** `parse_shape_gml` is the NAS reader with the geometry in `msGeometry`, followed by a mapping to
  the NAS shape of a feature: `nas_type` from `objart_txt`, `id` and `oid` from `objid`, `beginnt` from `beginn`,
  and the five short names the rules read under their NAS names. Everything after the reader (rules, duplicate
  step, notes, stored files) is unchanged.
- The Shape type names go to the server without a prefix; it accepts them so.

## Consequences

- Rhineland-Palatinate gets the landscape model like Bavaria: candidates of the lowest rank. In Cochem 19
  monuments and wayside crosses, 2 water axes and a well; around Dahn a rock, 3 water axes and 6 tracks and paths
  on the forest map. The paths cut the contours there as elsewhere, so the contour pieces grow (284 to 317).
- Another state that serves the Shape schema needs only a manifest block. A state with a different Shape
  spelling of a code name would need a row in `SHAPE_NAMES`.
- Rhineland-Palatinate is not recorded by the golden net; G2 (the estimate) has the new item, and G1 prints the
  new field of the Brandenburg service.
