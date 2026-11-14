# Bavaria: vegetation from the point clouds (design)

Date: 2026-11-14. Research: [`docs/regions/bayern-reference.md`](../../regions/bayern-reference.md), section
"Point clouds". Decision this builds on: [ADR 0105](../../adr/0105-vegetation-from-point-clouds.md) (vegetation
from classified point clouds) and [ADR 0123](../../adr/0123-bavaria-the-cadastre-from-files.md) (Bavaria).

## Intent

The user asked on 2026-11-14 to go on with Bavaria in this session and chose the vegetation first. Bavaria is the
one state with open point clouds that freemapper supports and does not read. Its forest example, the
Schmausenbuck, is the only one in a state with point clouds that has no vegetation.

With `--content-set vegetation=candidates` (or the box on the area page) a map in Bavaria gets the vegetation
candidates of ADR 0105, as in Brandenburg, Berlin, North Rhine-Westphalia, Rhineland-Palatinate and Saarland.

## Goals

- A sixth `PointCloudBackend`, for Bavaria, that behaves like the other five to the engine and to the user.
- The classes of the agency are read as it describes them.
- The date on the vegetation candidates is the day of the flight, not the day a file was put on a server.
- The Schmausenbuck example shows the vegetation.

## Non-goals

- No change to how cells are classified (ADR 0105). No tuning for the density of 52 points per m² or for flights
  without leaves; the live run reports what comes out.
- No partial download of a tile. A LAZ file is not sorted by place.
- The single trees of the agency (GeoPackage per flight lot), the ATKIS WFS and reading the DOM20 by range stay
  plans of their own.

## Anti-goals

- A map in another state must not change. The class rule for Bavaria applies to Bavarian tiles only; no golden
  file may change because of it.
- A failing or missing tile must not fail a fetch: the vegetation is only a candidate.
- No request to a host the manifest does not name.

## What is fixed by the data

| | |
|---|---|
| Address | `https://geodaten.bayern.de/odd_data/laser/<E>_<N>.laz`, south-west corner in km, 1 km, EPSG:25832 |
| Host | `geodaten.bayern.de`, a third host of the adapter |
| Size | 34 to 614 MB seen, median 230 MB, up to 1 GB by the portal |
| Format | LAS 1.2, point format 1, LAZ, no CRS in the header |
| Classes | 1, 2 ground, 6 building, 7, 9 water, 20 object (vegetation), 22 bridge or cellar exit, 23 and 24 synthetic ground |
| Freshness | `HEAD` gives `Last-Modified`; no list of tiles |
| Flight day | in the GPS time of the points (adjusted standard GPS time); no service names it |

## Design

### The backend

`crates/adapters/de/src/ldbv_las.rs`, `LdbvLasBackend`: provider `ldbv`, cache folder `ldbv/las`, source
`ldbv-las`, EPSG 25832, 1 km grid.

- **Which tiles**: the km tiles under the box, as the other backends.
- **Freshness and cache** use what Bavaria's other files use (`ldbv_files`): a cached tile is
  `<E>_<N>-<YYYYMMDD>.laz`, named by the day the server last changed it. Online one `HEAD` per tile decides.
  After one check that fails, the cached tiles of the run are used without asking. A 404 is believed for 30 days
  only for a tile whose centre lies beyond the border. Offline nothing is asked.
- **Download**: streamed to a partial file, at most 2 GiB, then checked to be a complete point cloud (header,
  chunk table, last point) and moved into place; the older copy goes after that. There is no listed size to
  compare with, so the completeness check is the one check.
- **A tile that fails** (download, check, server) falls back to the older cached copy with the warning the
  other backends give, or is left out with a warning. A tile the server does not have is a progress line, as
  for the other backends. Offline without a cached copy is the warning `warning-point-cloud-no-cached`. Only a
  cancelled fetch is an error.
- **The record** names the tile as `32<E>-<N>` (as the other states of zone 32), its address, and as
  `published` the flight day (below); where the points do not tell it, the day the header says the file was
  made; without that, the day in the name of the cached file; where the server named no day either, nothing.
- **`cached(record)`** is the newest cached copy of the tile. A project that recorded a tile keeps finding it
  after another project fetched a newer copy.
- **Estimate**: 250 MB per tile, cached when a copy exists.

### The flight day

`las_files::flight_day(path) -> Option<NaiveDate>`: when the header says adjusted standard GPS time, the day of
the newest of 16 points spread evenly over the file (the first and the last among them); GPS time + 10⁹ s counted
from 1980-01-06. A day before 1995 or after today is no day. With GPS week time (older lots may have it) there is
no day. Reading it costs 16 seeks in the chunk table and is done at every fetch, so nothing is stored beside the
tile.

Added during the build (2026-11-14): three more tiles showed that a lot of 2011 has GPS week time (tile 610_5480,
processed 2018, on the server since 2023) and that synthetic ground points (class 24) carry a GPS time of the year
8416. So a tile whose points do not tell the day is dated by the day in its header (`las_files::header_day`),
which lies after the flight and closer to it than the day the server got the file; and a time far off is no day.

### The classes

`PointCloudInfo` gets `left_out: Vec<u8>`: classes of this provider that are no returns of what grows there,
besides those every provider leaves out (buildings, low points, water, bridge decks, high noise, class 24, and
every synthetic or withheld point). Bavaria: 22 (bridges until 2020, cellar exits since) and 23 (synthetic
ground). The other five backends leave the list empty. `derive` hands the list of the project's backend to
`read_tile`.

### Registration

- Manifest of Bavaria: `geodaten.bayern.de` in `[net] hosts`, `revision = 2`.
- `State::Bayern` offers the backend; the test that lists the states with point clouds counts six.

### Texts

ADR 0128 (short: what is different about Bavaria's point clouds). Manual (German and English): the state list
of "Vegetation aus Laserpunktwolken", the size per tile, a sentence on the flight without leaves if the live run
shows it matters. README (both), the text of the content group on the area page (both languages), the examples
README (Schmausenbuck with vegetation), changelog (both), the reference. All user texts go through `unslop` and
`humanizer`.

### The example

`examples/bayern-schmausenbuck` is rebuilt with `--content-set vegetation=candidates`: four tiles, 1.2 GB.

## Gate

The live run of the Schmausenbuck decides whether this ships as it is. It passes when the vegetation candidates
show forest (405) under the crowns and open land where the orthophoto shows clearings and the power lines' swaths,
with green classes in between, and no class covers the whole forest. If one class covers nearly everything, or
the power lines come out as forest strips over open ground, the build stops and the finding goes to the user
with the picture.

## My calls (not the user's)

- The flight day from the GPS time instead of `Last-Modified` as the date of the candidates.
- The class list per provider instead of leaving out 22 and 23 for every state.
- Reusing `ldbv_files` (names with the server's day) instead of `las_files` names.
- 250 MB as the estimate per tile.
- Only the forest example gets the vegetation; Regensburg stays as it is.

## Left to the builder

The names of private helpers, the layout of the tests, and whether `download_las` takes an optional size or gets
a sibling.
