# 0202. A map without an orthophoto

Date: 2026-11-27
Status: Accepted

## Context

`derive` wrote the orthophoto template first and stopped with an error when the backend of the region gave no
orthophoto input. Every region until now has one. Sweden has none that freemapper can read: no WMS serves the
Lantmäteriet orthophoto, and the download is a 0.6 GB COG per 2.5 km square in 8 bits with four bands
(`docs/regions/sweden-reference.md`, §8.3). The first stage of the Swedish adapter (ADR 0203) works with the terrain
model, the laser data and the land cover, and its owner's application for the orthophoto is still open. Without a
change no Swedish map could be derived.

## Decision

- `OrthophotoInput` has a third variant, `Absent`. A raster backend answers it from `derive_inputs` when its region
  has no orthophoto.
- `derive` then writes no `templates/orthophoto.jpg` and no `rock_seen` raster (it is read from the orthophoto), and
  says so once in the warning `warning-no-orthophoto`. Everything else is written as usual: hillshade, slope, relief,
  contours, vegetation from the point cloud, the nDSM.
- The export lists the templates that exist, so the map has no orthophoto template and the `layers` command shows
  the orthophoto as not ready.
- A backend whose input is an error still fails `derive` as before. The German regions give an orthophoto always,
  so nothing changes for them.
- In the manifest a region with no orthophoto declares `[terrain]` alone (ADR 0189 addendum, format 1).

## Consequences

- A forest map of a region without an orthophoto has no cliffs from rock seen in the photo. The cliffs from the
  terrain (ADR 0198) stay.
- A sprint map without a photo can be drawn from the objects and the terrain, but the person cannot check it against
  the photo. The warning is the only hint; the manual says it for Sweden.
- If the orthophoto is added later, the backend stops answering `Absent`; no stored project needs a change.

Source: issue #273 (manifest rule), the design `docs/superpowers/specs/2026-11-27-sweden-adapter-design.md`.
