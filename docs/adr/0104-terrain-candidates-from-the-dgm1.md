# 0104. Terrain candidates from the DGM1

Date: 2026-11-12
Status: Accepted, extends spec §4.4 and §4.5

## Context

The contours of a sprint map (2 m) or a schoolyard map (1 m) miss the small forms of a park: a play mound, a
hollow, the bank of a terrace, a retaining wall the ALKIS does not have. Two open-source tools find such forms in
LiDAR data for forest maps: karttapullautin (GPL-3.0) and omapmaker (a noncommercial licence that is not
compatible with the GPL). The user asked which of their ideas improve sprint maps with park parts, and chose the
three that work on the DGM1 every state publishes.

## Decision

- `derive` writes a new template `templates/relief.png`: the DGM1 minus its Gaussian mean (σ 5 m sprint, 3 m
  schoolyard), grey with the mean at 128 and ±1 m and beyond white or black (`derive/relief.rs`). The idea is
  omapmaker's local relief layer. It is hidden in the map and listed as layer `relief`.
- `derive` writes `data/landforms.geojson` (`derive/landforms.rs`), read by the export into the candidates map:
  - small knolls (109), small elongated knolls (110) and small depressions (111). The terrain is smoothed with
    σ 1 m and traced at 0.25 m steps around each top. The outline is the lowest closed ring that lies at least
    1 m (sprint) or 0.5 m (schoolyard) below the top, is between 2 m and 5 mm on paper (at least 10 m) across, and
    rises at least 0.15 m per metre of its radius. Of two tops in one ring the one that stands out more (the
    greater height above its outline) stays. An outline with axes
    of 1.5 : 1 or more is elongated and gets its direction. Depressions are found the same way on the negated
    terrain.
  - earth banks (104) and impassable cliffs (201, from 3 m). A cell is steep from a gradient of 0.5, because DGM1
    blurs a 45° bank of 1.5 m to about 30°. A patch of steep cells is a bank when its median drop over ±3 m along
    the slope reaches the minimum height and holds at least 70 % of the drop over ±6 m (a plain slope holds half),
    and when it is at least twice as long as wide and at least 2 mm on paper (4 m on the ground) long. The line
    follows the long axis with the low side on the right, so the ticks point down. A patch that falls to both
    sides of its axis (a dyke, a ditch) is split by side, and one with cells more than 2 m off its line (a
    corner, a curve) is split across the axis at the cell farthest off, up to six times; a piece still that far
    off is left out.
- The export leaves out a knoll or depression within 2 m of a building and a bank or cliff that runs mostly
  within 2 m of one (DGM1 is interpolated there), a knoll or depression
  in the drawn water, and a bank or cliff that runs mostly within 2 m of the water or of a drawn earth bank, cliff,
  wall or fence. Everything is a candidate with `freemapper:source` (the terrain model), `freemapper:height`
  (0.1 m), `freemapper:currency` and `freemapper:landform` (the kind). The landforms are added before the
  content routing (ADR 0100). All of them belong to the group `landforms`, by that tag and not by their symbol:
  ALKIS and OpenStreetMap draw banks with the same 104, and a person who sets `slopes=map` for those must not
  get every bank and impassable cliff of the terrain model on the map too. `full` and `landforms=map` put them
  on the map; `balanced` leaves them candidates, because the terrain model has no source rank and a wrong
  impassable cliff forbids a way in a sprint race.
- Measured on 2026-11-12 in Viktoriapark, Sanssouci and Marburg (examples), against the orthophoto, the
  relief and the hillshade: 7 knolls, all in Viktoriapark, none clearly a knoll of the terrain (4 heaps of
  bulk material in a builders' yard, 2 heaps in allotments, 1 unclear); 12 depressions, about 1 real (the
  others in courtyards, light wells, ramps and a traffic island); 14 cliffs, real steps but mostly retaining
  walls and road edges; 155 banks, mostly plausible (path edges, terraces, sports grounds), a few at
  buildings. So `balanced` keeps all of them candidates.
- Filters after that measurement, in the export: no knoll or depression on residential, mixed, industrial
  and commercial land, traffic areas or allotments of the ALKIS land use (heaps, garden mounds, courtyards,
  ramps; land of `Fläche besonderer funktionaler Prägung` stays open, because schoolyard maps lie there); no
  depression within 8 m of a building (light wells, basement stairs); a cliff that runs mostly within 3 m of a
  traffic area or 5 m of a building is offered as an impassable wall, 515 (retaining walls; first an earth bank,
  changed after the review because a bank shows something impassable as passable). In `derive` the
  centre line of a bank or cliff is averaged over ±2 m before it is simplified, so it no longer zigzags.
  Measured again in the three areas: no knoll is left (all seven were heaps), 1 depression (the real hollow in
  Viktoriapark), 9 cliffs (3 in Marburg became banks), 162 banks. The question whether `balanced` takes the
  knolls comes back with areas that have real ones.
- The algorithms are our own. The pairing of drop and steepness follows karttapullautin's cliff test, the drop
  across the patch and the tick side follow omapmaker's cliff classification. No code is copied from omapmaker.
  freemapper does not depend on karttapullautin: its library split (`pullauta-core`) is not upstream, and most of
  what it needs are ground points we do not have in every state.

## Consequences

- Every project derives one more template and one more data file; the golden hashes (G4) change for every state.
- The candidates show forms the contours miss, and the mapper confirms them against the orthophoto and in the
  terrain. On DGM1 a wall narrower than a metre can be missed, and a road embankment shows up as a bank.
- The parameters scale with `scale_denominator` and the map kind, so a later forest map kind can reuse the module
  with its own values.
