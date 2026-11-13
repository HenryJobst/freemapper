# 0118. Paper and symbol measures are types

Date: 2026-11-13
Status: Accepted

Chosen by the user on 2026-11-13 from the architecture review of 2026-11-12 (candidate "symbol measures and
geometry helpers"), with the depth of the cut decided the same day.

## Context

A rule of the build names a length in one of three units: millimetres on the printed sheet, millimetres of a
symbol as the norm draws it at 100 %, or metres on the ground. The two paper units differ by the symbol
enlargement of the map kind (school map 125 %, forest map 150 % at 1:10000 and 200 % at 1:7500,
[0116](0116-forest-map-kind.md)).

Every constant was an `f64` with `_MM` in its name, and the conversion to ground metres or map coordinates was
written by hand at some 45 places in 17 modules of `build/` and three of `derive/`: `scale / 1000`, `1e6 /
scale`, `* 1000`, with or without the enlargement. Whether a measure grows with the symbols was decided where
it was converted, not where it was defined, and a function that did not get the map kind passed could not
enlarge at all. Some measures were not named: "1 mm on paper" was a bare `scale / 1000` in three places. The
width of the wall line (515) stood in three rules, the gap between two lines of one colour (§3.3) in three.

The review also named a width of 513.2 that stood as 0.60 and as 0.21 mm. Both are right: the line is 0.21 mm
wide, 0.60 mm with its half dots.

## Decision

- **A type for each paper unit** (`crate::measure`). `PaperMm` is a length on the printed sheet and converts
  with the scale alone. `SymbolMm` is a length of a symbol at 100 % and converts only with a `SymbolScale`, the
  scale together with the enlargement of the map kind. The type of a constant says whether it grows with the
  symbols, and the compiler asks for the enlargement where a symbol measure is used. `PaperMm2` and `SymbolMm2`
  do the same for areas.
- **Ground metres stay `f64`.** Their conversion to map coordinates is one function
  (`map_units_per_metre`).
- **Every constant gets the type of what it does today.** No measure changes with this record; the golden net
  passes without a blessed file. For the three map kinds and their six scales the old and the new arithmetic
  were compared number by number.
- **The measures of the norm that several rules share are written down once** (`build/symbol_measures.rs`):
  the gap between lines, the widths of wall, fence, hedge and retained wall, and the table of the point sizes.
  A table that belongs to one rule (the order of the barrier symbols, the paths a tree group is cut at, the
  smallest tree groups) stays with its rule and holds typed measures.
- **Not chosen: reading the measures from the symbol set.** The tables hold values that differ from the
  bundled symbols on purpose: the larger of the norm's measure and the drawn extent, and the errata of 2022.
- **Geometry helpers**: the distance from a point to a segment and the pair of numbers of a map coordinate are
  functions of `build/geometry.rs`. The two signed areas (line ticks, wall pairs) work on different point
  types and stay.

## Consequences

- A new rule declares its measure as `PaperMm` or `SymbolMm` and cannot forget the enlargement silently.
- Making a paper measure grow with the symbols is a change of its type; the compiler then names every place
  that needs the map kind.
- The limits of the barrier overlaps (`build/barriers.rs`) multiply by the map coordinates per symbol
  millimetre as one factor, as before. Enlarging first, as `SymbolMm::map_units` does, moves three of them by
  the last bit on a forest map at 1:10000, and such a limit decides between lines whose distance is a whole
  number of map coordinates. Bringing the two in line is a change of the output and not part of this record.

## Addendum 2026-11-13: the norm's minimum sizes grow with the symbols

The first step left every measure as it was and listed those that did not grow with the symbols. The user
decided the same day: **what the norm names as a minimum size is a symbol measure; a tolerance of the
computation is not.** Sprint maps draw their symbols at 100 % and stay as they were (the golden net passes
without a blessed file). School maps (125 %) and forest maps at 1:10000 and 1:7500 (150 %, 200 %) keep fewer
small objects.

| Measure | Value at 100 % | Now | Where |
|---|---|---|---|
| Smallest building and canopy | 0.5 mm × 0.5 mm | grows | `build/buildings.rs`, `build/canopy.rs` |
| Shortest line of a line symbol by ISSprOM 4.5 (railway, power line, 528, 529) | 2.2 to 4 mm | grows | `build/candidates.rs` |
| Smallest piece of out-of-bounds land, paving, green, open land and water | 1 mm × 1 mm | grows | `build/out_of_bounds.rs`, `build/paved.rs`, `build/open_land.rs`, `build/water.rs` |
| Contours: shortest line, smallest ring | 2 mm, 1.5 mm² | grows | `derive/contours.rs` |
| Landforms: largest knoll, shortest bank | 5 mm, 2 mm | grows | `derive/landforms.rs` |
| Smallest vegetation area | 1.5 mm × 1.5 mm | grows | `derive/vegetation.rs` |
| Simplification of outlines | 0.1 mm | stays | buildings, canopies, tree groups, contours |
| Tolerance of the area duplicates | 0.25 mm | stays | `build/dedup.rs`, `build/firm_paths.rs`, `build/water.rs` |
| Fountains of two sources are one | 0.75 mm, at least 3 m | stays | `build/dedup.rs` |
| Reach of a passage | 0.5 mm | stays | `build/passages.rs` |
| Shortest line piece after a cut or a clip | 1 mm | stays, not decided | `build/candidates.rs`, `build/line_cuts.rs`, `build/paved_borders.rs` |
| Longest bar scale | 60 mm | stays (a measure of the sheet) | `build/peripheral.rs` |

- **The symbol set carries its enlargement** (`SymbolSet::enlargement`). Most of these rules get the symbol set
  and not the map kind; `SymbolScale::drawn_with(symbol_set, scale)` gives them the scale without passing the
  kind through some 15 functions. `MapKind::symbol_set` builds the set with the enlargement of the kind, and a
  test holds the two together for every kind and scale.
- The derive steps (contours, landforms, vegetation) take a `SymbolScale` instead of the scale denominator.
  Their files in a project folder are as the derive step of that time wrote them: a school or forest project
  gets the new limits when it is derived again.
- The stubs of a cut line (1 mm) were in neither list of the decision and stay; they are a paper measure until
  someone decides otherwise.

## Addendum 2026-11-13: the line stubs stay, the barrier limits convert like the rest

Two points the record left open are closed.

- **The shortest line piece after a cut or a clip stays 1 mm on the sheet** (`MIN_LINE_MM` in
  `build/candidates.rs`, `MIN_PIECE_MM` in `build/line_cuts.rs`, `MIN_BORDER_MM` in `build/paved_borders.rs`).
  The user decided it: the stub is what is left of a line the map cut itself, a limit of legibility on the
  sheet and no minimum size of a symbol. The row "stays, not decided" of the table above reads "stays".
- **The limits of the barrier overlaps convert with `SymbolMm::map_units`** like every other rule;
  `SymbolScale::map_units_per_symbol_mm` is gone. On a forest map at 1:10000 (symbols at 150 %) two limits move
  up by the last bit: 0.4 mm for a wall or fence beside a hedge (600 map coordinates) and 0.65 mm for a wall
  or fence beside a retained wall (975). A line at exactly that distance now counts as too close. Sprint and
  school maps and the other forest scales compute the same numbers as before; the golden net passes without
  a blessed file, and the Dammheide example (forest, 1:10000) is rebuilt byte for byte.
