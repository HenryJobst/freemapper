# 0198. The steepness threshold of banks and cliffs is a setting

Date: 2026-11-26
Status: Accepted, extends ADR 0134

## Context

`derive` finds earth banks (104) and impassable cliffs (201) in the terrain model as steep, long and narrow patches
(`derive::landforms::height_jumps`). A cell is steep from a gradient of 0.5, about 27°. In a rock town every
slope is steep: on the example Adršpach (Czechia, ČÚZK DMR 5G, 2 m) 47 % of the area is above 27°, and 92 % of those cells hang
together in one patch of 687 000 cells. A patch that is not long and narrow is a hillside, so `derive` found two
cliffs of 59 m in all (one of 30 m with the direction check below), although the model holds drops of up to 38 m on 2 m. The terrain model is no coarser than it
needs to be: the cause is the threshold. A user saw that the map showed the grey rock plateaus of OpenStreetMap and
hardly any black cliff.

## Decision

- The threshold is a setting of the project, `steep_slope_deg` in `freemapper.toml` (15° to 80°; none: the default of
  about 27°), set by `--steep-slope <degrees>|auto` on `init`, `derive` and `run`, by `steep_slope` (0: default) in the
  MCP tools `create_project` and `start_pipeline`, and by the field "Steepness threshold" on the area page. It follows
  the contour interval (ADR 0116) in every place. It counts for the fingerprint of the project like the interval.
- No automatic choice. One measurement on one area is too little to pick a value for every terrain, and a default that
  changed with the relief would change the golden maps of every state. The manual and the area page say which values suit
  which terrain; the table of the manual rests on the Adršpach measurement and marks what is an estimate.
- A line that runs down the slope instead of along the contours is dropped at every threshold
  (`runs_down_the_slope`): sampled every 2 m, the mean cosine to the fall line (the mean direction of the steep cells
  in 5 × 5 cells around the point) must be at most 0.57 (55°). Without it a third to a half of the length of the cliffs
  at 55° to 60° lay across the contours (measured: 17 of 34 lines under 60° to the fall line, 52 % of the length); the
  centre line of a wide steep patch follows its longest axis, not the wall.
- The texts of the page, the help, the MCP tools and the error are in all 21 languages of the program; the ones except
  English and German are machine translated (the user's rule for the program languages).

## Consequences

- The default threshold is unchanged. The direction check holds for every terrain, also at the default: it removes
  lines that ran across the contours and so were no walls. The golden maps of Berlin, Brandenburg, Hesse and
  Saxony-Anhalt changed in `data/landforms.geojson` and in what depends on it (the terrain candidates, the check
  print, the OCAD file of that layer, in Hesse also the training map); they were blessed (`Golden-Bless:`). The old test cases
  (steps, an L and a U) pass unchanged. An example of a state rebuilt later may lose such a line as well.
- Measured on Adršpach after the direction check: default 1 cliff (30 m), 50° 6 (230 m), 55° 11 (610 m), 60° 11 (470 m),
  65° 11 (430 m), 70° 4 (170 m). The check took the candidates at 55° to 60° from about 30 to 11. A single-point
  measurement against the raw gradient still calls half of the remaining length "across the contours"; it is noisy in
  broken rock and the check judges a 5 × 5 mean, so it is no proof either way.
- The hillshade of Adršpach shows hundreds of single rock towers, not a network of walls. The search for long narrow
  strips cannot draw towers; it finds a handful of walls where a mapper would draw a cliff ring round every tower
  or use rock pillars (203, 204). The candidates are hints to check. Drawing towers is a separate decision.
- The threshold holds for banks and cliffs alike. On the Nikolsdorfer Wände (Saxony, DGM1 of 1 m) the default finds 4 cliffs
  (159 m) and 16 banks (850 m), 55° finds 9 cliffs (377 m) and no bank. A person who raises it for the cliffs of a rock
  town loses the banks of the ordinary slopes there. Most cliffs of that map come from OpenStreetMap (540 objects), not
  from the terrain; on Adršpach OSM has 3. How good a rock map looks depends on the OSM data first.
- Contours still run together in such land. The ISOM 2017-2 asks for 0.15 mm between lines of one colour and for contours
  that interplay with cliff symbols; freemapper only chooses a wider interval (ADR 0125). Thinning the contours is a
  separate decision. The norms and what was read of them: `docs/research/orienteering-specifications.md`.
