# ADR 0187: BDOT10k roads and paths on forest maps

## Status

Accepted, 22 November 2026. Builds on [ADR 0178](0178-poland-bdot10k-fallback.md), which left the
roads out ("lines without a width; OSM has them"), on [ADR 0135](0135-bavaria-objects-from-the-basis-dlm.md)
(a landscape model below OpenStreetMap) and on [ADR 0149](0149-hamburg-feinkartierung-a-set-that-brings-its-rules.md)
(a set that brings its rules).

## Context

ADR 0178 read BDOT10k for buildings and land use only. Two of the classes it skipped are lines
that OpenStreetMap often lacks in the open land of Poland, above all the dirt roads of fields and
forests. Measured at powiat 0805 (słubicki, archive of 22 January 2026, objects of 21 October
2024):

- `OT_SKDR_L` (road axes), 13 822 lines, 19.7 MB as GML. `klasaDrogi`: droga wewnętrzna 9 726,
  lokalna 2 437, zbiorcza 909, główna ruchu przyśpieszonego 314, główna 226, dojazdowa 179,
  autostrada 31. `materialNawierzchni`: grunt naturalny (bare ground) 9 357, masa bitumiczna 2 711,
  kostka prefabrykowana 501, bruk 373, płyty betonowe 350, beton 232, tłuczeń 165, żwir 77, kostka
  kamienna 56. `szerokoscNawierzchni` (m) on 4 247 of them, none on bare ground. The earlier note
  "no width" held for the bare-ground roads only. `polozenie`: on the ground 13 618, above it 88,
  under it 3.
- `OT_SKRP_L` (footways, cycle ways), 540 lines: `rodzaj` aleja lub pasaż 374, droga dla
  rowerów 128, ścieżka 38; `szerokosc` 1.5 to 7 m.
- `OT_SKJZ_L` (carriageway lines) repeats the axes (14 147 lines) and is not read.
- `OT_SKTR_L` is **railway** track (273 lines, `rodzajPojazduSzynowego` kolej), not footways or
  tracks of the forest. OpenStreetMap has the railways; the class is not read.

The architecture has a way for line sources: `RegionAdapter::extras()` returns `ExtraSet`s, and a
set that brings a `SetDrawing` (ADR 0149) gets its layers stored under `data/<key>/` and drawn
through its own rule file, with a rank in the duplicate step. Hamburg's Feinkartierung is the
precedent. What did not exist is a rank for a source that must rank **below** OpenStreetMap: the
rank `Landscape` was tied to the source prefix `atkis-dlm/`.

## Decision

- **A set `bdot10k`** (`bdot_roads.rs`), two layers: `bdot-drogi` from `OT_SKDR_L` and
  `bdot-sciezki` from `OT_SKRP_L`, for **forest maps only**. A sprint or schoolyard map has the
  paved areas of the cadastre and the paths of the orthophoto; ADR 0135 left the path layers on
  sprint maps out for the same reason.
- **One new rank, `SetRank::Landscape`**, which the engine maps to `Rank::Landscape`. A set may
  now rank below OpenStreetMap. Nothing else of the engine changes: the objects of the set lose
  against OpenStreetMap exactly as the DLM paths do (a line with 80 % of its length within 10 m of
  a line of a higher rank, all path classes one kind; they stay candidates under `balanced`).
- **Two member reads instead of the archive.** The set reads the two class files as single members
  of the powiat archive by HTTP byte range (ADR 0017: central directory, then the member, checked
  against its CRC). Measured at 0805: 1.94 MB plus 0.07 MB packed against the 53.9 MB archive. The
  set therefore needs no cache directory (an `ExtraSet` has none), and what it fetched is stored
  in the project like every set. A member above 64 MB unpacked is refused by the shared reader and
  fails the layer with a warning; the largest class file measured is 19.7 MB.
- **Rules** (`rules/bdot-roads.toml`, forest only), by the measured values:

  | BDOT10k | ISOM |
  |---|---|
  | autostrada, główna ruchu przyśpieszonego, główna, zbiorcza | 502 |
  | lokalna, dojazdowa, paved, `szerokosc` at least 5 m | 502 |
  | any other paved road (bitumen, concrete, cobbles, setts, paving stones, concrete plates) | 503 |
  | gravel (żwir), crushed stone (tłuczeń) | 504 |
  | lokalna, dojazdowa, bare ground | 504 |
  | wewnętrzna, bare ground | 505 |
  | footway ścieżka | 506 |
  | footway droga dla rowerów, aleja lub pasaż | 505 |

  The table follows what the OSM rules of the forest map do for the same things (a residential
  road 502 from 5 m and else 503, an unpaved road 504, a track without a tracktype 505, a cycle way
  505, a path 506). The assignment of the bare-ground internal roads to 505 and of the local ones
  to 504 is a judgement, not a measurement. A class or material the rules do not name draws
  nothing.
- **Left out:** objects that are not in use, stretches under the ground, `OT_SKJZ_L`,
  `OT_SKTR_L`, the width of the road as a corridor, the `kategoriaZarzadzania` and the road number.
- **Licence and credit.** As ADR 0178: free of charge for any purpose, no wording prescribed. The
  credit of GUGiK (`© GUGiK, geoportal.gov.pl`) is given when the project holds a layer of the set,
  also when its rasters are not from GUGiK.
- **Hosts.** The index and the archives are on the two hosts the adapter asks already; the adapter
  manifest revision is 7 and the host list is unchanged.

## Not measured

- How much of the BDOT10k road length OpenStreetMap lacks. ADR 0135 measured 4 % for Bavaria at one
  forest; for Poland nothing is measured, and the run needs the OSM extract of a voivodeship. The
  rule of ADR 0135 (candidates, never doubles) is taken over for that reason.
- Other powiats than 0805: the values of `klasaDrogi` and `materialNawierzchni` of the rest of
  Poland. Unknown values draw nothing.
- The estimate counts 2 MB per km² of the map as a stand-in for the 2 MB per powiat of the member
  reads, which is too high for a large map and, where a map spans powiats, too low.

## Consequences

- A forest map in Poland has candidate paths from the register where OpenStreetMap has none within
  10 m; the download is about 2 MB per powiat, once for the project.
- A forest map in a municipality with a thin OSM has the dirt roads of the fields as 505 candidates
  to check against the orthophoto; `full` puts them on the map like the DLM paths.
- Another country's landscape model can use `SetRank::Landscape` with a rule file and a reader, no
  engine code.
