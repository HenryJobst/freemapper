# 0123. Bavaria: the cadastre from files

Date: 2026-11-14
Status: Accepted

The user asked for Bavaria on 2026-11-13. Research:
[`docs/regions/bayern-reference.md`](../regions/bayern-reference.md) (services checked live on 2026-11-13, a
live run on 2026-11-14). Design:
[`docs/superpowers/specs/2026-11-13-bayern-design.md`](../superpowers/specs/2026-11-13-bayern-design.md). It is
the fourth state added on the adapter scheme of [0082](0082-region-adapters-trait-and-manifests.md), after
Saarland, Baden-Württemberg and Bremen ([0121](0121-bremen-the-full-alkis-as-the-simplified-schema.md)).

## Context

The Bayerische Vermessungsverwaltung publishes everything a map needs under CC BY 4.0, in EPSG:25832 for the
whole state. Three things do not fit the forms freemapper had:

- **No service for the cadastre.** Buildings and land use are files only. Buildings: the Hausumringe as one
  Shapefile per Regierungsbezirk (66 to 174 MB, no attribute but the municipality), or the LoD2 buildings as
  CityGML, one file per 2 km (3 MB in the country, 160 MB in the centre of Munich). Land use: one Shapefile per
  Kreis (2 to 13 MB), whose fields are already the simplified schema.
- **No 1 m surface model.** There is the DOM20, 20 cm, computed from the aerial pictures: 32 to 56 MB per km².
- **No list of tiles with their dates.** The tiles have fixed addresses that follow from their position; a
  Metalink per municipality names sizes and hashes, but an area would first have to find its municipalities.

## Decision

- Bavaria is a built-in state: manifest folder `crates/adapters/de/manifests/bayern/`, id `bayern`, provider and
  backend `ldbv`, hosts `geoservices.bayern.de` and `download1.bayernwolke.de`, seven OSM extracts by
  Regierungsbezirk. The credit is the agency's own source note for changed data, without a year.
- **Buildings come from the LoD2 tiles**, not from the Hausumringe. A tile is the smaller download for most
  maps, and LoD2 says what an object is: a building, a roofed structure (`51009_1610`, `51009_1611`, drawn as a
  canopy) or something that is no building at all (bridges, walls, locks, left out). Each object carries its
  ALKIS id. At Regensburg LoD2 holds 99.70 % of the Hausumringe of at least 10 m². A tile is streamed to a
  file, read as a stream by the new reader `freemapper_sources::citygml`, and cached as the few megabytes of
  features it holds; the CityGML file is deleted.
- **Land use comes from the Shapefile of each Kreis** within 500 m of the map area, read by the new reader
  `freemapper_sources::shapefile`. Which Kreis lies where comes from `assets/kreise/bayern.geojson`, the 96
  outlines simplified to 100 m. A square without a recorded use gets `ohne Unterart (Bayern)` and stays a
  candidate, as in the other states.
- **A cached file carries the day the server last changed it** (`<stem>-<YYYYMMDD>`). Online one `HEAD` per
  file decides whether the cache is current; a check that fails goes on with the cached file and a warning;
  offline nothing is asked. A 404 is believed for 30 days. This holds for the four kinds of files alike.
- **The surface model is cached reduced to 1 m**, each cell the highest of its 25 values; the 20 cm tile is not
  kept. Its date is the flight date in the metadata of the tile.
- Both readers are own code. No dependency was added.

## Consequences

- A first run in a Bavarian city centre downloads more than anywhere else: 60 to 160 MB per 2 km of buildings
  and about 45 MB per km² of surface model. The estimate lists both; the surface model is left out when the disk
  is short, as everywhere.
- The data has no position to the surface. LoD2 holds what stands above the ground (the five underground
  garages in the Regensburg box are 1.7 to 3.9 m high), so those objects are drawn. The function of a building
  is carried as a tag; no rule reads it.
- Bavaria has no surveyed lines from the cadastre (stairs, walls, embankments): no `[[extra.nas]]`.
- A LoD2 tile the server does not have is a warning inside Bavaria, because a tile without buildings exists as
  an empty file; beyond the border it is silent.
- Thuringia is now the one state without data sources. The coverage warning names it alone.

## Left out

Vegetation from the point clouds (vegetation is class 20 there, not the ASPRS classes), the ATKIS WFS (rocks,
embankments, cuttings, paths), and reading only the part of a DOM20 tile the area needs. Each is a plan of its
own; the reference lists what was found about them.
