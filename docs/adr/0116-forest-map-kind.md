# 0116. Forest map kind

Date: 2026-11-13
Status: Accepted

## Context

The user trains in small forests a few kilometres from home (in Berlin: Mittelheide and Dammheide, Uhlenhorst,
Wuhlheide and the woods of Biesdorf, 0.5 to 5 km² each) and wants a forest map for a quick training: make the project,
run it, print it on A4 or A5 at 1:15 000, 1:10 000 or 1:7500. Later the same project may be the base for mapping in
the terrain. The path network must be right; the vegetation may be rough, and invented green is worse than none
(spec `docs/superpowers/specs/2026-11-13-forest-map-design.md`).

A run in Berlin with the sprint kind as a stand-in (2026-11-12) showed:

- OSM has dense path data in all four forests (Dammheide 26 km `path`, Mittelheide 40 km `path` and 24 km `track`),
  and over the hillshade the paths follow what the terrain model shows. ALKIS and the Berlin registers have almost
  no paths in the forest. OSM is the path source.
- The forests are flat: contours at 5 m say little.
- The Berlin laser scan of 2021 was flown without leaves. The vegetation of ADR 0105 found only runnable forest
  (405), and the nDSM shows deciduous stands as low. Neither is a source of green in Berlin.

The suite contract had the kind `forest` without drawing scales (freemapper-core 0.22.0).

## Decision

- **Kind and scales.** `MapKind::Forest`, name `forest`, at 1:15 000 (default), 1:10 000 and 1:7500, from
  freemapper-core 0.23.0 (`MapKind::Forest.drawing_scales()`, written by freemapper). `symbol_enlargement` and
  `symbol_set` take the scale: the ISOM symbols are drawn at 100 %, 150 % and 200 %, as ISOM 2017-2 says for
  enlarged maps, so the paper distances between symbols (point gaps, wall pairs, north lines, stair widths, the
  reach of walls and paths) follow the scale. The minimum sizes on paper (buildings, areas, contour rings,
  slivers) scale with the scale denominator only, as for the school set.
  Default buffer 100 m, contour smoothing 3 m, relief smoothing 8 m, landforms from 1 m.
- **Symbol set.** `ISOM 2017-2_15000.omap` from OpenOrienteering Mapper (same commit as the ISSprOM set) with the
  colour table of IOF Rev. 4 by the rules of ADR 0044; 1:10 000 and 1:7500 enlarge it (`SymbolSet::isom_2017_2`),
  not OOM's separate 10 000 file. No walkable area (990): the forest is the white paper.
- **Aliases instead of a second code path.** The build modules keep their ISSprOM codes.
  `assets/symbol-sets/isom-aliases.toml` maps each code to the ISOM code that draws the same thing with the same
  symbol type (505.1 → 505, 505.2 → 504, 532.6–532.9 → 532, 509.1/509.2 → 509, 513.1/513.2 → 513,
  301.6 → 301.4, 501.x areas → 501, paved paths 501.6/501.7 → 505 and 501.8/501.9 → 503, 601 → 601.1) or leaves the
  object out (321, 410.1, 501.1, 541–545, 990). `id_for_code` asks the aliases first, because some codes mean
  different things in the two norms (501.1 is a step line in ISSprOM and the paved-area fill in ISOM).
- **Paths from OSM, on the map.** A `forest` column in `assets/rules/osm-candidates.toml`, before the sprint rules:
  502 for main roads and wide residential roads, 503 for the other public roads, 504 for tracks of grade 1 and 2 or
  at least 3 m wide and for unpaved roads, 505 for the other tracks, cycleways and firm or wide paths, 506 for the
  other paths, 507 for paths with bad visibility or `informal=yes`, 532 for stairs (ISOM 2017-2 has a stairway,
  unlike what the spec assumed). They go onto the map (`target = "map"`), private ones included; nothing is merged
  or moved. Left out: sidewalks, crossings, driveways and parking aisles (`forest = "-"`). A rule without a forest
  code gives the forest map its sprint code.
- **Forest wins.** Where OSM (`landuse=forest`, `natural=wood`) or ALKIS (`Wald`, `Gehölz`) says forest, no ALKIS open
  land is drawn, and the woods get no object. The OSM forest features are kept for this by rules with
  `forest = "-"`. OSM parks draw nothing on a forest map; OSM meadows, grassland, heath and clearings are 401 and
  403 candidates, scrub 406 and dense scrub 408 candidates. That is all the green: none from the nDSM.
- **What else.** A forest map takes the sprint build otherwise: buildings, water, walls and fences, power lines,
  railways, out-of-bounds land as 520 (never 709), and the Berlin and NAS layers of the sprint map. Street
  furniture and lamps (OSM benches, bollards, waste baskets, bicycle stands, street lamps, the Berlin survey's
  benches, bollards and masts, the lighting cadastre), register trees and plain OSM trees stay off; a tree tagged `denotation=natural_monument|landmark` is a 417
  candidate. ALKIS street parcels are no paved areas: ISOM draws roads as lines. The map area outline of the
  candidates uses the sheet line 994, as the ISOM set has no thin line under that code. OSM paths that the
  aliases give the symbols of the sprint map's firm-gravel ways (505, 503) are not cut at paving or shortened.
- **OCAD.** The OCAD export numbers the ISOM codes by the decimal rule (`301.1` = 301.100); the ISSprOM
  exceptions of `ocd/symbols.rs` do not apply. In OCAD the forest map gets the ISOM 2017-2 set.
- **Area page.** The page selects each kind's default scale (`default_scales`), 1:15 000 for a forest map.
- **Contour interval from the relief.** `derive::contours::contour_interval_for`: a forest map gets 2.5 m when the
  5th to 95th percentile of the smoothed terrain inside the map area spans less than 15 m, else 5 m; every fifth line
  is an index contour. `contour_interval_m` in `freemapper.toml` wins; 0, a negative value or NaN is refused when
  the project loads. `contours.geojson` stores an interval that is not the kind's as `interval_m`, which the export
  reads for the index contours and the margin ("Äquidistanz 2,5 m"). Files of schoolyard and sprint projects stay
  byte for byte as before.

## Addendum (2026-11-13): the interval in the sidecar

freemapper-print printed the Dammheide map and proposed 5 m, because the interval was only in `contours.geojson`.
`derive` now also writes it into `freemapper.toml` as the record `[contours] interval_m`, under the same rule (only
when it is not the kind's), next to the setting `contour_interval_m`, which stays the person's choice. The record is
a contract key from freemapper-core 0.25.0 on (`Sidecar::contour_interval_m`); a project at the kind's interval keeps
its file byte for byte.

## Consequences

- `freemapper init <dir> --kind forest --scale 10000` and `run --pdf-candidates` give the quick training map; the
  `.omap` with its candidates stays the base for mapping, as for the other kinds.
- OSM paths without tags beyond `highway=path` all become 506; the mapper sorts out what is wider or fainter.
- The contract fixtures of freemapper-core have no forest map yet, so `contract_output_breaks_no_rule` checks only
  schoolyard and sprint.
- An ISSprOM code that gets drawn some day without an alias is caught by `every_code_freemapper_draws_has_an_isom_answer`
  for the OSM rules; a code used only in Rust needs its alias added by hand.

## Update 2026-11-13: north line spacing

The north lines (symbol 601) of a forest map are 20 mm apart at symbol size 100 %, not 30 mm. ISOM 2017-2 changed
this against ISOM 2000: "Their spacing on the map shall be 20 mm on the map which represents 300 m on the ground at
the scale of 1:15 000" (OMapWiki, symbol 601). British Orienteering's letter to the federations on ISOM 2017 says:
"ISOM201X 601: The distance between the magnetic north lines has been adjusted so that it will be reasonable when
enlarged to 1:10000". At 1:10 000 the symbols are enlarged by 150 %, so the lines are 30 mm apart on paper; both
scales give 300 m on the ground. Sprint (ISSprOM 2019) and schoolyard maps keep 30 mm.
