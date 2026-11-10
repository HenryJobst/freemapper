# 0079. Firm gravel paths are map objects again

Date: 2026-11-10
Status: Accepted, amends ADR 0073

## Context

ADR 0073 made `surface=compacted|fine_gravel` ways (and `tracktype=grade2` tracks) 501.6 to 501.9 candidates instead of
505.1 map objects, so that the mapper would check them against the orthophoto. In the park examples this removed the
paths from the main map: Sanssouci 237 to 19 map lines, Hessen 43 to 8, Quedlinburg 41 to 5, Viktoriapark 58 to 9.
The paths are real and firm in OSM; a park map without them is nearly empty. The user decided on 2026-11-10 that hard
gravel paths are map objects again, with the right ISSprOM symbol.

What the norm says (ISSprOM 2019-2, section 4.5, text of the 2022 update):

- 501 Paved area: "A paved area is an area with a firm surface such as asphalt, hard gravel, tiles, concrete or the
  like. It should be bordered (or framed) by the symbol Step or edge of paved area (501.1)." So hard gravel is paved,
  not unpaved.
- 505 Unpaved footpath or track: "a way for passing mainly by foot, without a smooth, hard surface. The colour shall
  be the same as for the symbol Paved area (501)." A compacted or fine gravel path has such a surface, so it is no 505.
- 506 Small unpaved footpath or track and 507 Less distinct small path are the unpaved narrow forms; 506 has a
  minimum length (isolated) of two dashes, 3.4 mm.
- Section 2, minimum dimensions: area symbols have minimum widths (colour screens 0.4 mm, footprint 1.6 m at 1:4000);
  features below it are exaggerated or left out.

The norm text has no 501.6 to 501.9. They are the line forms of 501 that the bundled symbol set
(`assets/symbol-sets/ISSprOM 2019_4000.omap`) offers for a paved way of footprint 1.4, 2, 3 and 4 m: a brown 30 %
line of 0.35, 0.5, 0.75 and 1.0 mm with a black 0.1 mm border line on both sides (the colours of the 501 area).
They are the symbols the mapper uses for a narrow paved path, so they are the right ones for hard gravel.

## Decision

- The six rules of ADR 0073 for firm gravel (`surface=compacted|fine_gravel` by `width`: under 2 m or none 501.6, 2 m
  501.7, 3 m 501.8, 4 m 501.9, a track without width 501.7; and `tracktype=grade2` 501.7) get `target = "map"`.
  Their conditions do not change: footways, paths, bridleways and tracks, not sidewalks or crossings, not tunnels,
  covered or indoor ways, not closed rings. The widths stay the ones of ADR 0073.
- A line form, not a corridor area. The line symbols carry the footprint and the border of the norm's paved area, an
  area strip of 1.4 m would be under the minimum width of colour screens, and the other way-based map objects (505, 506)
  are lines too. The `corridor_m` field stays for 520 and 522.
- What stays a candidate: a way without `surface` (505.1), an unusual surface, `tracktype=grade3|grade4|grade5` (505.2),
  and unpaved streets. Their surface is not known to be firm or to be loose.
- What stays out: private and closed ways are 520 candidates before any path rule (ADR 0073), steps and sidewalks have no
  rule, paved surfaces have none (ALKIS and the survey draw them).
- Where ALKIS or the Berlin survey draws paving (501.5 map objects and candidates), the way is the same object. The
  part of the line within that paving, widened by 0.25 mm on paper (the tolerance of ADR 0078), is cut away
  (`build/firm_paths.rs`). A piece shorter than 1 mm on paper (4 m at 1:4000) is a stub and goes, whether it came from
  the cut or was short to begin with. Only OSM lines of 501.6 to 501.9 are touched.
- The tree groups are still cut at these lines (ADR 0069; `PATH_WIDTHS_MM` lists 501.6 to 501.9 among the candidates
  and the map objects). A test covers the four widths as map objects.

## Consequences

Counts of the map part and the candidates, before (ADR 0073 state, `main` at c37999b) and after, rebuilt from the same
stored data:

| | 501.6 map | 501.7 to 501.9 map | 501.6 to 501.9 candidates |
|---|---:|---:|---:|
| Viktoriapark before | 0 | 0 | 49 |
| Viktoriapark after | 26 | 0 | 0 |
| Sanssouci before | 0 | 0 | 218 |
| Sanssouci after | 192 | 0 | 0 |

Viktoriapark loses 23 of its 49 objects, and Sanssouci 26 of 218. They lie in ALKIS or survey paving, are stubs after
the cut, or are shorter than 1 mm. The 501.9 ways of the Viktoriapark (four) lie in paving.

- The park paths are on the map again and the candidates are free of them. The mapper checks a path against the
  orthophoto after the fact, as for every other map object.
- A compacted path that has no ALKIS paving under it but is a lawn in reality is now drawn. That is the cost of the
  decision; the `freemapper:osm` tag names the OSM tags.
- A `Weg` parcel of ALKIS is a 501.5 candidate, and it cuts the line too, so a path inside such a parcel is only a
  candidate (the parcel).
- Existing maps get all of this with `run --force`.

## Addendum 2026-11-10: only paving map objects cut the line

Cause: the first version cut the line at all 501.5 areas, map objects and candidates alike. An ALKIS `Weg` parcel is
a candidate only (sidewalks and lanes in town, unpaved tracks in the woods), so a firm path inside it vanished from
the map and the candidate parcel was all that was left. Decision (user, 2026-11-10): the line is a map object and stays
where only a candidate parcel lies. `cut_firm_paths` now takes `PavedObjects` and cuts at `paved.map` only. The
0.25 mm tolerance, the 1 mm minimum piece and the no-doubling where a 501.5 map object exists are unchanged. In the
candidates PDF the parcel fill and the line now overlap; that is intended, as the parcel is a candidate and the line
a map object. Cost if wrong: where the `Weg` parcel is the same path, the path is drawn on the map although the
mapper may reject the parcel. Measured on the same stored data, the 501.6 to 501.9 map objects rise from 26 to 44 in
the Viktoriapark and from 192 to 199 in Sanssouci. Existing maps get this with `run --force`.

