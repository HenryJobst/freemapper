# 0128. Bavaria: vegetation from the point clouds

Date: 2026-11-14
Status: Accepted

The user asked on 2026-11-14 to go on with Bavaria and chose the vegetation first. Research:
[`docs/regions/bayern-reference.md`](../regions/bayern-reference.md), sections "Point clouds" and "Live run".
Design: [`docs/superpowers/specs/2026-11-14-bayern-vegetation-design.md`](../superpowers/specs/2026-11-14-bayern-vegetation-design.md).
It adds a sixth source to [0105](0105-vegetation-from-point-clouds.md) and builds on
[0123](0123-bavaria-the-cadastre-from-files.md).

## Context

The Bayerische Vermessungsverwaltung publishes its laser points under CC BY 4.0, one LAZ file per km tile in
EPSG:25832. Four things differ from the five sources freemapper had:

- **No list of tiles.** The address follows from the tile (`https://geodaten.bayern.de/odd_data/laser/<E>_<N>.laz`).
  Nothing names a size or a hash without first finding the municipalities of an area.
- **A third host.** The tiles lie on `geodaten.bayern.de`; the server of the other tiles does not have them.
- **No service names the flight.** `Last-Modified` is the day a file was put on the server, up to years after the
  flight.
- **Other classes.** 22 is a cellar exit (a bridge until 2020), 23 and 24 are synthetic ground. No point carries
  the synthetic flag.

The tiles are large, 34 to 614 MB in fourteen tiles across the state, up to 1 GB by the portal. The lots differ:
8.7 points per m² in a lot of 2011, 52 in one of 2021. The three flights whose day is known were flown between
November and February.

## Decision

- Bavaria gets a point-cloud backend, `ldbv_las`: provider `ldbv`, cache folder `ldbv/las`, source `ldbv-las`.
- **A tile is kept like Bavaria's other files** (0123): named by the day the server last changed it, one `HEAD`
  per tile at a fetch, after one failed check the cached tiles of the fetch are used without asking, and a 404 is
  believed for 30 days only beyond the border. A download has no listed size to match, so it must be a complete
  point cloud (header, chunk table, last point) and at most 2 GiB.
- **A failing tile never fails the fetch**: the older cached copy serves with a warning, or the tile is left out
  with one, as for the other sources.
- **The date of the vegetation is the day of the flight**, read from the tile: the newest day among 16 points
  spread over the file, when its header says adjusted standard GPS time. A time far off is no day (the synthetic
  ground points of a lot of 2017 carry the year 8416). Older lots have GPS week time, which names no week; such a
  tile takes the day its header says it was made, which lies after the flight and before the upload. Only then
  comes the day of the server.
- **A provider names the classes it leaves out** (`PointCloudInfo::left_out`), besides those `derive` leaves out
  everywhere. Bavaria names 22 and 23; the other five name none, so no other map changes.
- The manifest of Bavaria names the third host and is revision 2.
- The estimate counts 250 MB per tile.

## Consequences

- A forest map of 1.2 km × 0.9 km in Bavaria loads four tiles, 1.2 GB at the Schmausenbuck. The group stays off
  until it is chosen, as everywhere.
- Flights without leaves understate thickets, as in Berlin and Saarland. At the Schmausenbuck the swath of a power
  line with young growth of 2 to 5 m has 45 to 55 % of its returns above 0.9 m and still comes out as runnable
  forest (405): the bare twigs give a green index of 0.31, below the 0.4 of the first green class. The classes are
  not tuned for that here.
- A recorded tile is found by its name, not by its day: a project keeps its tile after another project fetched a
  newer copy.

## Left out

Tuning the classes for flights without leaves or for the density of a lot, the single trees the agency derives
from the point clouds (a GeoPackage per flight lot), and a partial download of a tile (a LAZ file is not sorted
by place).
