# ADR 0171: Poland takes buildings and land use from the cadastre EGiB

## Status

Accepted, 21 November 2026. Builds on [ADR 0164](0164-poland-national-adapter-osm-first.md) and
[ADR 0170](0170-poland-rasters-from-gugik.md).

## Context

With the rasters of GUGiK, a map in Poland still took its buildings and land use from OpenStreetMap.
Poland has a cadastre, EGiB (`ewidencja gruntów i budynków`), and GUGiK serves it for the whole
country through one WFS (`docs/regions/poland-reference.md`, Stage 3). The footprints are surveyed,
and the register class of a parcel says what lies on it.

## Decision

- **EGiB through the national WFS.** `egib.rs` reads `ms:budynki` and `ms:dzialki` from
  `UslugaZbiorcza` in EPSG:2180, box north first. A page holds at most 1000 features; a full page
  names the next in the attribute `next`, and the client follows it, stops at a short page, at a
  page it has seen, and after 200 pages. Both hosts are those the policy of the adapter names
  already.
- **The LGB schema again.** The engine reads one schema for buildings and land use. EGiB is
  turned into it: a building is a `Gebäude` whose `funktion` follows the register's kind
  (`RODZAJ`: residential, farm, industrial, office and so on); a parcel gets the `nutzart` its
  register class (`KLASOUZYTKI_EGIB`) stands for: `Ls` forest, `Lz` trees, `Ł` and `Ps` grassland,
  `R` and `S` farmland, `Wp` and `Ws` water, `dr` road. Built-up classes, waste land and ditches
  give none.
- **A parcel of two uses has no land use.** The class list of a parcel (`R,Ls`) says what lies on
  it, not where. Painting the whole parcel with one of them would colour a field with the wood
  beside it, so it gets none.
- **The grid is inverted in the adapter.** The core of the suite projects to PL-1992 and not back.
  The adapter finds the position of a point by Newton's method on the forward projection, exact
  to under a millimetre, instead of waiting for a core release.
- **Coverage `Full`.** Poland has rasters and a cadastre now, so the engine treats it as a state
  with both. What was true only of the German states stays true of them: the permit notes (German
  sources and law) are refused for a map outside Germany, and an area that reaches over a border is
  refused when it is centred outside Germany. A map centred in Germany with a strip of Poland
  keeps its warning, because the rasters of its own region end at the border.
- **The estimate names EGiB** for a Polish map and not ALKIS.

## Not decided here

- **BDOT10k as a fallback.** The plan has it for buildings when EGiB fails, and for land use that
  is finer than parcels. It is a GML archive of about 54 MB per powiat. Until it exists, a failing
  EGiB service stops the fetch with its message instead of leaving the map without buildings.
- **The progress lines still say "ALKIS"** for a Polish map ("299 ALKIS buildings"). The text is
  in every language catalog and gets its own change.
- **The letters of `RODZAJ` and the classes `Bz` and `Tp`** were taken from the ordinance on the
  register and not checked with the service; a class the table does not know gives no land use,
  and a kind it does not know keeps its letter.

## Consequences

- A map in Słubice shows 299 buildings with surveyed outlines where OSM gave a fraction of them
  as areas of the right shape.
- Land use from parcels is coarse in towns: most parcels are built-up and give none, so a town
  map has roads, and fields and woods show where parcels are large.
- Poland stays a preview until the manual, the examples and the permit notes catch up.
