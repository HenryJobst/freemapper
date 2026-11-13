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

## Open: measures that do not grow with the symbols today

These are `PaperMm` because the code did not enlarge them. Sprint maps are not affected (100 %); on a school
map and a forest map they are smaller, relative to the symbols, than on a sprint map. Which of them the norm
means as symbol measures is the user's decision; each change alters school and forest maps.

| Measure | Value | Where |
|---|---|---|
| Smallest building and canopy | 0.5 mm × 0.5 mm | `build/buildings.rs`, `build/canopy.rs` |
| Shortest line of a line symbol by ISSprOM 4.5 (railway, power line, 528, 529) | 2.2 to 4 mm | `build/candidates.rs` |
| Shortest line piece after a cut or a clip | 1 mm | `build/candidates.rs`, `build/line_cuts.rs`, `build/paved_borders.rs` |
| Smallest piece of out-of-bounds land, paving, green and open land | 1 mm × 1 mm | `build/out_of_bounds.rs`, `build/paved.rs`, `build/open_land.rs` |
| Tolerance of the area duplicates ("one thin line width") | 0.25 mm | `build/dedup.rs`, `build/firm_paths.rs`, `build/water.rs` |
| Fountains of two sources are one | 0.75 mm, at least 3 m | `build/dedup.rs` |
| Reach of a passage | 0.5 mm | `build/passages.rs` |
| Contours: shortest line, smallest ring | 2 mm, 1.5 mm² | `derive/contours.rs` |
| Landforms: largest knoll, shortest bank | 5 mm, 2 mm | `derive/landforms.rs` |
| Smallest vegetation area | 1.5 mm × 1.5 mm | `derive/vegetation.rs` |
| Simplification of outlines | 0.1 mm | buildings, canopies, tree groups, contours |
| Longest bar scale | 60 mm | `build/peripheral.rs` (a measure of the sheet) |
