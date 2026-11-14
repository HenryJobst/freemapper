# The three content profiles compared

[Deutsch](vergleich.md) · [Examples overview](README.en.md)

What freemapper puts straight onto the map and what stays a candidate in the template `templates/candidates.omap`
is decided by the [content profile](../docs/manual/en.md#what-goes-straight-onto-the-map): `safe`, `balanced` (the
default) or `full`. Candidates are uncertain derived objects that a mapper checks before taking them over. The
manual explains [what to check in the terrain](../docs/manual/en.md#9-what-to-check-in-the-terrain).

This page shows one area, Reichsburg and Cochem old town, in all three profiles. The pictures show the map alone,
exactly the objects of the OOM file (`--pdf`); the link below opens the PDF. All examples are built with
`balanced`; the other two versions were made from the same data with `freemapper export --force --content safe`
and `--content full`.

## Cochem in three profiles

| `safe` | `balanced` (default) | `full` |
|---|---|---|
| ![Cochem, profile safe](rheinland-pfalz-cochem/preview-safe.webp) | ![Cochem, profile balanced](rheinland-pfalz-cochem/preview-balanced.webp) | ![Cochem, profile full](rheinland-pfalz-cochem/preview-full.webp) |
| [rheinland-pfalz-cochem-safe.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-safe.pdf) | [rheinland-pfalz-cochem.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf) | [rheinland-pfalz-cochem-full.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-full.pdf) |

| Profile | Map objects | Candidates |
|---|---:|---:|
| `safe` | 1664 | 1691 |
| `balanced` | 2866 | 506 |
| `full` | 3285 | 25 |

All three versions have the same 1156 buildings (521), the forest (405), the Moselle and the railway. The
difference lies in everything that does not come from the cadastre.

## From `safe` to `balanced`

With `safe` the map shows the town but not the terrain. The castle hills and valley sides are white, because the
contours stay candidates, and so do paths of unknown surface, walls, stairs and trees.

With `balanced` these objects go onto the map, because OpenStreetMap and the terrain model are often their only
source:

| Symbol | `safe`: candidates | `balanced`: map |
|---|---:|---:|
| 101 contour | 627 | 627 |
| 102 index contour | 155 | 155 |
| 418 bush or small tree | 234 | 188 |
| 532.6 stairway | 51 | 51 |
| 505.1 unpaved footpath | 46 | 51 |
| 513.2 passable retaining wall | 29 | 29 |
| 515 impassable wall | 30 | 11 |

506 objects stay candidates, most of them estimates: 203 out-of-bounds areas (520) from the ALKIS land use, 114
earth banks (104) from the terrain model, 78 control sites (531), 46 trees, 24 areas of open land (401), 19 walls
and 10 cliffs (201).

## From `balanced` to `full`

`full` puts everything onto the map, as the training map (`--pdf-candidates`) draws it. In Cochem 203
out-of-bounds areas, 114 earth banks, 71 control sites, the other 19 walls and 10 cliffs come in. The plots between
the houses turn olive green. Some points that were candidates in `balanced` are missing from `full` altogether,
among them 46 trees and 7 control sites: as map objects they are subject to the ISSprOM minimum distance, and
freemapper leaves out those that stand too close. 25 objects stay candidates, 24 of them open land (401).

## Which profile for what

- `safe` fits when you survey the terrain yourself and want only the surveyed base.
- `balanced` is the default. Contours, paths, stairs and walls are on the map already; you still check them. What
  is an estimate, above all the out-of-bounds areas, stays a candidate.
- `full` gives a quick training map without going into the terrain. It does not tell whether a 520 area is really
  forbidden; before a competition you check each one.

The training map of each example draws the map with all candidates, for Cochem
[rheinland-pfalz-cochem-candidates.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-candidates.pdf). It leaves
out points under a 520 area and passages that cannot be reached (`freemapper:hidden`), so its picture does not
match the counts above in every point. The counts come from `rheinland-pfalz-cochem.omap` and
`templates/candidates.omap` of each version.

Made with freemapper – https://github.com/HenryJobst/freemapper
