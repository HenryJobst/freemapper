# The three content profiles compared

[Deutsch](vergleich.md) · [Examples overview](README.en.md)

What freemapper puts straight onto the map and what stays a candidate in the candidate layers
`templates/candidates-<layer>.omap` is decided by the
[content profile](../docs/manual/en.md#what-goes-straight-onto-the-map): `safe`, `balanced` (the default) or
`full`. Candidates are uncertain derived objects that a mapper checks before taking them over. The manual explains
[what to check in the terrain](../docs/manual/en.md#9-what-to-check-in-the-terrain).

This page shows one area, Reichsburg and Cochem old town, in all three profiles. The pictures show the map alone,
exactly the objects of the OOM file (`--pdf`); the link below opens the PDF. The candidates lie in seven layers by
kind (ADR 0156), which OOM shows hidden when the map opens. In every profile the light level takes hidden lines and
objects below the minimum size off the map and puts them with the candidates (ADR 0157). All examples are built
with `balanced`; the other two versions were made from the same data with `freemapper export --force --content
safe` and `--content full`.

## Cochem in three profiles

| `safe` | `balanced` (default) | `full` |
|---|---|---|
| [![Cochem, profile safe](rheinland-pfalz-cochem/preview-safe-thumb.webp)](rheinland-pfalz-cochem/preview-safe.webp) | [![Cochem, profile balanced](rheinland-pfalz-cochem/preview-balanced-thumb.webp)](rheinland-pfalz-cochem/preview-balanced.webp) | [![Cochem, profile full](rheinland-pfalz-cochem/preview-full-thumb.webp)](rheinland-pfalz-cochem/preview-full.webp) |
| [rheinland-pfalz-cochem-safe.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-safe.pdf) | [rheinland-pfalz-cochem.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf) | [rheinland-pfalz-cochem-full.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-full.pdf) |

| Profile | Map objects | Candidates |
|---|---:|---:|
| `safe` | 1649 | 1278 |
| `balanced` | 2355 | 545 |
| `full` | 2817 | 117 |

All three versions have the same 1156 buildings (521), the forest (405), the Moselle and the railway. The
difference lies in everything that does not come from the cadastre.

## From `safe` to `balanced`

With `safe` the map shows the town but not the terrain. The castle hills and valley sides are white, because the
contours stay candidates, and so do paths of unknown surface, walls, stairs and trees.

With `balanced` these objects go onto the map, because OpenStreetMap and the terrain model are often their only
source:

| Symbol | `safe`: candidates | `balanced`: map |
|---|---:|---:|
| 101 contour | 282 | 248 |
| 102 index contour | 74 | 65 |
| 418 bush or small tree | 233 | 187 |
| 532.6 stairway | 51 | 38 |
| 505.1 unpaved footpath | 46 | 46 |
| 513.2 passable retaining wall | 29 | 26 |
| 515 impassable wall | 30 | 13 |

The map has fewer contours and stairs than `safe` had as candidates: the light level joins contours that meet end to
end, and it puts stairs at a building or below the minimum size with the candidates. The walls from the terrain
model stay candidates with `balanced` too.

545 objects stay candidates, most of them estimates: 199 out-of-bounds areas (520) from the ALKIS land use, 113
earth banks (104) from the terrain model, 77 control sites (531), 46 trees, 24 areas of open land (401), 19 walls
from the terrain model, 19 monuments (526) and 10 cliffs (201). The light level took 22 more off the map: 15
stairways (9 at a building, 6 below the minimum size), 3 walls, and 2 hedges and 2 retaining walls.

## From `balanced` to `full`

`full` puts everything onto the map, as the check print (`--pdf-candidates`) draws it. In Cochem 199
out-of-bounds areas, the earth banks, 70 control sites, the walls from the terrain model, 16 monuments and the 10
cliffs come in; of the cliffs, 5 stretches and a top line (201.3) are drawn, and 12 stretches stay candidates
because they are too low, too short or too close to another cliff. The plots between the houses turn olive green.
Some points that were candidates in `balanced` are missing from `full` altogether, among them 46 trees, 7 control
sites and 3 monuments: as map objects they are subject to the ISSprOM minimum distance, and freemapper leaves out
those that stand too close.

117 objects stay candidates. The 24 areas of open land (401), the 12 cliff stretches, a passage that cannot be
reached (522) and the outline of the map area stay off the map with `full` as well. The light level takes the other
79 off the map, because more lines lie on each other in `full`: 37 stretches of contours on an earth bank, 13
stretches of earth banks on a path, a building or another bank, the 15 stairways, 8 walls, and 2 each of hedges,
retaining walls and small watercourses (305).

## Which profile for what

- `safe` fits when you survey the terrain yourself and want only the surveyed base.
- `balanced` is the default. Contours, paths, stairs and walls are on the map already; you still check them. What
  is an estimate, above all the out-of-bounds areas, stays a candidate.
- `full` gives a quick map without going into the terrain. It does not tell whether a 520 area is really
  forbidden; before a competition you check each one.

## Base map, check print and training map

One run with `balanced` makes three maps. The base map shows the sure objects true to position; the candidates lie
hidden in the layers. The check print draws the base map with every candidate, also true to position, for checking
in the terrain. The training map takes the map and the candidates and cleans them up for printing and running: it
joins, shifts and leaves out what would otherwise cover something else.

| Base map | Check print | Training map |
|---|---|---|
| [![Cochem, base map](rheinland-pfalz-cochem/preview-balanced-thumb.webp)](rheinland-pfalz-cochem/preview-balanced.webp) | [![Cochem, check print](rheinland-pfalz-cochem/preview-candidates-thumb.webp)](rheinland-pfalz-cochem/preview-candidates.webp) | [![Cochem, training map](rheinland-pfalz-cochem/preview-thumb.webp)](rheinland-pfalz-cochem/preview.webp) |
| [rheinland-pfalz-cochem.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf) | [rheinland-pfalz-cochem-candidates.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-candidates.pdf) | [rheinland-pfalz-cochem-training.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-training.pdf) |

The check print of this example also draws the vegetation from the laser points, which the counts above leave out.
It leaves out points under a 520 area and passages that cannot be reached (`freemapper:hidden`), so its picture does
not match the counts in every point. The counts come from `rheinland-pfalz-cochem.omap` and the seven candidate
layers of each version; they count the outline, which every layer has, once.

Made with freemapper – https://github.com/HenryJobst/freemapper
