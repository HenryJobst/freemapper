# 0118. Baden-Württemberg: the nDOM1 on the terrain, and the ALKIS of the views by code

Date: 2026-11-13
Status: Accepted

The user chose Baden-Württemberg as the next state on 2026-11-13. Research:
[`docs/regions/baden-wuerttemberg-reference.md`](../regions/baden-wuerttemberg-reference.md) (2026-11-08, checked
again on 2026-11-13). It is the second state added on the adapter scheme of
[0082](0082-region-adapters-trait-and-manifests.md), after Saarland ([0111](0111-saarland-the-first-state-on-the-adapter-scheme.md)).

## Context

The LGL publishes everything a map needs under dl-de/by-2-0, and three of its sources do not fit the forms the
manifest knows:

- The ALKIS WFS serves GeoServer views (`nora:v_al_gebaeude`, `nora:v_al_tatsaechliche_nutzung`), not the
  simplified schema: the id is `gml_id`, the geometry `geom`, the class `objektname`, and `SORTBY=oid` is refused.
- The heights are one zip per 2 km block with four 1 km tiles. The terrain model is XYZ text of 29 MB per tile,
  and a tile at the state border holds only the rows inside the state.
- There are two surface products. DOM1 is the laser scan, flown without leaves between 2016 and 2024 (2017 in
  Freiburg). nDOM1 is the height above the terrain of the surface matched from the aerial images of a summer
  flight: in the places looked at, the flight of the orthophoto or the one before it (2024 or 2025 against 2025
  or 2026).

## Decision

- Baden-Württemberg is a built-in state: manifest folder `crates/adapters/de/manifests/badenwuerttemberg/`, id
  `badenwuerttemberg`, provider `lglbw`, raster backend `lgl-bw` (`crates/adapters/de/src/lgl_bw.rs`), EPSG:25832.
- **The ALKIS is code, the manifest declares none.** `LglBwAlkis` pages the views by `gml_id` and turns each feature
  into the simplified schema: `gml_id` → `oid`, `objektname` → `nutzart`, `gebaeudefunktion_name` → `funktion`,
  `lage_oberflaeche_name` → `rellage`, `eigenname` → `name`, the day of `beginn` → `aktualit`, and `gebnutzbez` is
  "Gebäude", because the view holds buildings only. `unterart_name` becomes `bez` only when `unterart_id` is set:
  without a subtype the view repeats the class there. A square without a subtype gets `ohne Unterart
  (Baden-Württemberg)` and stays a candidate, as in the other states. No key was added to the manifest format: a
  mapping with a condition and a constant is not a rename table, and one state does not justify a preset.
- The WFS reader of `freemapper-sources` reads a geometry named `geom`, and `fetch_wfs_by` pages, sorts and
  de-duplicates by a key the caller names. `WfsService` is unchanged, so no golden file changed.
- **The surface model is the nDOM1.** The project stores a surface model and `derive` subtracts the terrain, so the
  backend stores nDOM1 + DGM1 on the grid of the area. The record carries the flight date of the surface
  (`bDOM20_Aktualitaet` of the tile at the centre). A tile the nDOM1 lacks inside the state is reported and stays
  empty; DOM1 is not mixed in.
- The terrain is read from the XYZ member of each tile. Cells without a line stay empty; a point that is no cell
  centre of the tile is an error. The cache keeps each tile as a GeoTIFF named after the tile and the CRC-32 of
  its member, so a tile the LGL replaces is read anew.
- A block without an archive (HTTP 404) holds no tile of the state. Any other failure of an archive stops the
  fetch of the terrain; for the surface it costs a warning, as with every provider.
- The orthophoto comes from the WMS in pieces of at most 4 000 px, dated by the WFS of the flight tiles.
- OSM comes from Geofabrik's four extracts by Regierungsbezirk (128 to 213 MB instead of 653 MB), as in North
  Rhine-Westphalia ([0028](0028-nrw-osm-extracts-per-regierungsbezirk.md)).
- Credit: `Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0 (Daten geändert, <year>)`. The open-data page of the LGL
  prescribes the first part word for word; the capabilities of the services ask for the year of the download, and
  the licence for the note of changes.
- Like Saarland, the state is not in the golden net. It has unit tests with synthetic archives and recorded WFS
  answers, and `adapters check --live`.

## Consequences

- The nDSM template shows the trees in leaf, and buildings as of a flight of the last two or three years. The stored
  surface is a sum of two products of different dates; the record names the date of the nDOM1 only.
- `adapters check --live` on the manifest folder asks the rasters and OSM and says "alkis … not asked: the adapter
  declares none", as for Berlin. The ALKIS is covered by a real run.
- The view has no building parts and no roofed areas, so canopies and passages do not come from the cadastre, and
  it has no class for the green beside a street: `Straßenverkehr` areas include it and are drawn as paved.
- Not built: a check that refuses an all-white orthophoto piece inside the state. The WMS answers white from
  6 144 px on without an error; freemapper asks 4 000 px. If the LGL lowers that limit below 4 000 px, a map gets a
  white template without a warning.
- Not built: a fallback to the kept directory of an archive when the file server is down. A run then fails at the
  terrain, and `--offline` uses what the cache holds.
- A new state still touches about twenty places in the adapter crate and the tests that count the states; this one
  touched the same places as Saarland and added none.
