# Forest map kind: design

Date: 2026-11-13
Status: draft, written after a conversation with the user; waits for their approval

## Intent

The user trains in small forests 4 to 5 km from home (the first ones: Mittelheide and Dammheide, Uhlenhorst,
Wuhlheide and the woods of Biesdorf, all in Berlin, 0.5 to 5 km² each) and wants a forest map for a training without
much work: make the project, run it, print it. Later the same project may be the base the user maps in the terrain.
Printed on A4 or A5, at 1:15 000, 1:10 000 or 1:7500.

What must be right: **the path network**. What may be rough: vegetation. The user does not know the vegetation of
the first forests either; a map that shows runnable forest where it is not runnable is acceptable for this training,
a map with invented thickets is worse.

## What a run in Berlin showed (2026-11-12, sprint kind at 1:5000 as a stand-in)

- OpenStreetMap has dense path data in all four forests (Dammheide 26 km `path`, 4 km `track`; Mittelheide 40 km
  `path`, 24 km `track`, mostly with `tracktype`). Over the hillshade the OSM paths follow the lines the terrain
  model shows; no path was obviously missing or invented. ALKIS and the Berlin registers have almost no paths in
  the forest. **OSM is the path source.**
- The forests are flat; contours say little, the small forms of the terrain model (banks, ditches, hollows,
  mounds) say more.
- The Berlin laser scan (2021) and with it the Berlin nDSM were flown without leaves. The vegetation module of
  ADR 0105 classified the whole Dammheide as runnable forest (405) and found no green shades; the nDSM shows
  deciduous stands as low. Neither is a reliable source for green in Berlin.
- A run with the release build took 3 to 11 minutes per forest (without point clouds); with point clouds 22 minutes
  for the Dammheide, about 2 GB of download.

## Goals

1. A new map kind `forest` (ISOM 2017-2) next to `schoolyard` and `sprint`, chosen as today with `--kind forest`,
   in MCP and on the area page.
2. The path network from OSM in the ISOM path classes, on the map.
3. Contours and the small landforms of ADR 0104, with forest parameters.
4. White forest by default; yellow open land from ALKIS and OSM; green only where OSM explicitly says scrub or
   thicket. The laser vegetation of ADR 0105 stays a separate, opt-in group.
5. The quick training map: `--pdf-candidates` gives a printable forest map at the chosen scale with everything drawn,
   the uncertain objects included. `--pdf` gives the map without the candidates.
6. The `.omap` with map objects and the candidates template stays the base for mapping in the terrain, as for the
   other kinds.

## Non-goals

- Tuning the laser vegetation for forests, form lines, rock and boulder detection from the point cloud, a forest
  course planner. Each can come later.
- A5 or other sheet layouts in freemapper itself: freemapper-print lays out sheets; freemapper's own PDF stays as
  it is (A4/A3, chosen by size).
- New data sources. Everything comes from the region adapters that exist.

## Anti-goals

- Green that the data does not support (from the leaf-off nDSM, from a guess). White is the honest default.
- Paths that look certain on the map but come from a weak guess. OSM paths are the source the user wants; they
  go on the map, but nothing is invented between them.
- Breaking the schoolyard and sprint kinds: their output and the golden net stay byte for byte as they are.

## Decisions

### Kind and scales

- `MapKind::Forest`, name `forest` from `freemapper_contract::MapKind::Forest` (it exists in freemapper-core, default
  scale 15 000, but `drawing_scales()` is empty and `is_written_by_freemapper()` is false).
- Scales: **15 000** (default, symbols 100 %), **10 000** (symbols 150 %) and **7500** (symbols 200 %), as ISOM
  2017-2 and `freemapper-printrules/rules/iof-print.toml` say. The scale is the project's `scale_denominator`; the
  enlargement goes into `symbol_enlargement` as for the schoolyard (1.25). So a 1:10 000 project draws its PDF
  at 1:10 000 with 150 % symbols, which is what the user prints on A4.
- This needs freemapper-core first: `Forest.drawing_scales()` = `[15000, 10000, 7500]` and freemapper in the list of
  writers of `Forest`, then a new core tag and the pin in freemapper (ADR 0067, ADR 0095). The design asks the
  session that owns freemapper-core (or the user) for it; until then the work sits on a branch.

### Symbol set

- `ISOM 2017-2_15000.omap` from OpenOrienteering Mapper (GPL-3.0-or-later, same source and handling as
  `ISSprOM 2019_4000.omap`): the file goes to `assets/symbol-sets/` with its commit and SHA-256 in the README, and
  the colour table is changed to IOF Rev. 4 exactly as for ISSprOM (ADR 0044).
- The sheet symbols 991 to 994 are added as for the other kinds. No walkable-area base fill (990): a forest map has
  white forest as its base.
- 10 000 and 7500 enlarge the 15 000 set by 150 % / 200 % (as `school_2019()` enlarges ISSprOM by 125 %), not
  OOM's separate 10 000 file, so there is one source file.

### Paths (OSM to ISOM)

ISOM 2017-2 codes: 502 wide road, 503 road, 504 vehicle track, 505 footpath, 506 small path, 507 less distinct
small path, 508 narrow ride or linear trace. The forest kind gets its own column in `assets/rules/osm-candidates.toml`
(the rules already have one field per kind):

| OSM | ISOM |
|---|---|
| `highway` motorway … tertiary, or residential/unclassified with `width` ≥ 5 | 502 |
| residential, unclassified, service with a paved or compacted surface | 503 |
| `track` grade1–grade2, or `track`/`service` unpaved with `width` ≥ 3 | 504 |
| `track` grade3–grade5 or without grade; `path`/`footway`/`bridleway`/`cycleway` with `width` ≥ 1.5 or a firm surface | 505 |
| `path`/`footway`/`bridleway` otherwise | 506 |
| `path` with `trail_visibility` bad/horrible/no, or `informal=yes` | 507 |
| `highway=steps` | 506 (ISOM has no steps symbol; the landform shows the slope) |

- All of them go **on the map** (target `map`), because the user chose OSM as the path source and the training
  depends on them. Private or forbidden ways (`access=private|no`) stay paths; an area that must not be entered is
  520 (as in sprint, content group `out-of-bounds`), never 709, which is a course symbol of freemapper-courseplan.
- *Builder's call, made in the design:* no merging or moving of OSM ways. Left out are only ways ISOM does not draw
  at 1:15 000: sidewalks and cycleways that OSM ties to a road (`footway=sidewalk`, `cycleway=*` along a road,
  `highway=footway` with `footway=crossing`) and service ways inside settlement areas. Everything else is drawn as
  OSM has it; dense spots are left to the mapper.
- OOM's `OSM-ISOM 2017-2.crt` is a cross-reference to compare with (GPL); the table above is ours.

### Terrain

- Contour interval **5 m**, or **2.5 m** where the map area is flat: 2.5 m when the range between the 5th and 95th
  percentile of the smoothed DTM inside the map area is below 15 m. Index contour every fifth (as now). The
  interval can be set in `freemapper.toml` (`contour_interval_m`), and the PDF names it as today.
- Landforms of ADR 0104 with forest values: knolls and depressions at least 1 m high (ISOM 109–111 minimum),
  banks and cliffs as now with the paper minimums at 1:15 000 (they scale with `scale_denominator` already);
  candidates as now.
- Smoothing of contours and relief: `contour_sigma_m` 3, `relief_sigma_m` 8 (builder may tune on the four Berlin
  forests).

### Vegetation and land cover

- Forest from ALKIS (`Wald`, `Gehölz`) and OSM (`landuse=forest`, `natural=wood`): white, no object (the paper is
  white). *Builder's call, made in the design:* forest wins. Where OSM or ALKIS says forest, no open land is drawn,
  even if ALKIS calls the land a park or a recreation area (Wuhlheide, Biesdorf); ALKIS parks become 401 only
  where no forest lies over them.
- Open land: ALKIS `Landwirtschaft`/`Grünland`, OSM `landuse=meadow|grass`, `natural=grassland`: 401 open land;
  `natural=heath`, OSM clearings (`landuse=clearing`, `natural=grassland` inside forest): 403 rough open land;
  as candidates, and on the map with `full` like the other OSM land cover.
- Green only from OSM `natural=scrub` (406) and `landcover=scrub` with `density=dense` (408), as candidates.
- The laser vegetation (group `vegetation`, ADR 0105) works for the forest kind too, opt-in as now, with its
  minimum areas from the scale (`min_area_m2` already scales).
- Settlement at the forest edge: ISOM 520 (area that shall not be entered) for private gardens as in sprint, 521
  buildings, 501 paved areas (ISOM: paved area), roads as above.

### Other objects

*Builder's call, made in the design:* every object freemapper draws today that has an ISOM 2017-2 counterpart is
drawn in the forest kind too, with the ISOM code: water (301–313), walls and fences (513–519), power lines
(510–512), railways (509), buildings (521), paved areas (501), towers and other prominent man-made features (526,
530, 531, 532 where ISOM has them), and single trees only where OSM tags `natural=tree` with `denotation=natural_monument`
or similar as ISOM 417. Left out: street furniture, the Berlin street-tree register, lamps and other point objects
that ISOM has no symbol for or that are too small at 1:15 000. Point minimum distances follow the scale as now.

### Output

- `.omap` with the ISOM set, candidates template, ATTRIBUTION, preview: as for the other kinds.
- `--pdf` and `--pdf-candidates` at the project scale. The candidates PDF is the quick training map.
- OCAD export: maps the ISOM codes like the others (ISOM numbers are OCAD's numbers).

## Left to the builder

- Where exactly each `MapKind` branch goes (about 25 files, see the recon of 2026-11-12) and whether a parameter
  table per kind replaces the matches.
- The details of the path table where OSM data is ambiguous, checked on the four Berlin forests.
- Default buffer around the map area (sprint 50 m; forest probably 100 m), and the warning for very large areas.
- Tests: unit tests per rule; one golden state gets a forest project only if the golden store already holds what
  it needs (no new recording just for this).

## Coordination

- freemapper-core: contract change for `Forest` (scales, writer). Without it `contract_kinds_are_the_kinds_of_the_suite`
  fails. Until a core tag holds it, the branch pins core by `[patch]` to a local checkout or a core branch; the PR is
  merged only once a core tag holds it.
- freemapper-print reads `kind = "forest"` already (its print rules know ISOM); it is told when the kind exists.
- Sessions working on `build/` (freemapper-d3: vegetation cut at 520) and on the adapters (FableWorker) are told
  before the branch touches their files.
- The work goes in as a pull request; merged after the user's yes.

## Test areas

Dammheide (`relation/4513841`), Mittelheide (`relation/11032199`), Uhlenhorst (`way/347014858`), Wuhlheide
(`way/1488438700`), all Berlin; one forest in Brandenburg with a leaf-on scan for the laser vegetation.
