# Poland: map generation for all of Poland

Design, 2026-11-20. Status: draft, waits for approval.

## Intent and why

freemapper makes starter orienteering maps from official geodata and OpenStreetMap. Today it covers 15 German
states. The user wants the same for **all of Poland**, and the program has to **work for Polish users**: a Polish
orienteering club or teacher types a place in Poland and gets a map, in their own language, without German
data or German law in the way.

Poland was always the first neighbour country on the agenda (user, 2026-11-07, "after the German states"). The
research is `docs/regions/crossborder-reference.md` (probed live 2026-11-07, border strip only). No account is
needed for any Polish source, unlike Sweden.

## Goals

- A project anywhere in Poland runs `init`, `run` and `export` with the kinds `schoolyard`, `sprint` and
  `forest`, with orthophoto, hillshade, slope, relief, nDSM, contours, buildings and land use, and the candidate
  layers, as in a German state.
- One national adapter `polska`. The GUGiK services are national; only the OSM extract differs by voivodeship.
- Polish as a third program language (web UI, console, manual, README), chosen like DE/EN
  (`--lang`, `FREEMAPPER_LANG`, system language).
- Polish permit notes (who is in charge: forest district, Natura 2000, park, owner), as an own last stage.
- Every stage releasable on its own.

## Non-goals

- No voivodeship adapters (16 of them): the sources are national.
- No other neighbour country in this work. Czechia has its own research and its own plan.
- No change of the map output, the rule engine or the candidate rules for the sake of Poland.
- No licensed or contract-bound Polish data (parked idea, see `licensed-sources-idea`).

## Anti-goals (a failure even if it runs)

- A map in Poland that is quietly worse than OSM alone because a bad tile (old year, 5 m grid) won over a good one.
- A German-only error message or German legal hint shown to a Polish user.
- A raster warp that shifts objects against the vectors by more than a pixel.
- A fixed-size download that fills the disk (ortho 0.05 m is 1.4 GB per sheet).

## Constraints

- CRS: PL-1992 (EPSG:2180, transverse Mercator, lon0 19°, k 0.9993, FE 500 000, FN -5 300 000, GRS80). It is rotated
  about 3° against the UTM grid at the German border and more elsewhere. freemapper knows UTM 32 and 33 only, and the
  adapter design (`2026-11-10-region-adapters-design.md`, non-goals) says another zone or a national grid needs its
  **own ADR**. Poland spans UTM 33 and 34, and 2180 is the grid of every Polish product.
- Axis order: WFS 2.0 BBOX in EPSG:2180 takes N,E; the data comes back E,N.
- Heights PL-EVRF2007-NH; the older KRON86 is not used. On the Oder overlap the difference to DHHN2016 was a median
  of 3 cm, so no correction.
- Quality differs by tile: year, resolution (1 m, 5 m), source (LiDAR or photogrammetry). Picking is per tile.
- Licence: GUGiK data free for any use. Credit wording is our suggestion until a source says otherwise.
- The crate layout is `crates/adapters/de`; a non-German adapter needs a place and a country concept.
- The research is two weeks old and covers the Słubice border strip only. It is re-checked before building, and
  the EGiB WFS (announced 2026-07-16) has undocumented limits.

## Approaches considered

- **A. By product in stages, one national adapter (chosen by the user, 2026-11-20).** Foundation and OSM first,
  then terrain and orthophoto, then buildings and land use, then Polish, then permit notes. Each stage ships.
  Trade-off: until stage 3 the vector content is OSM only.
- B. OSM and heights only, wait. Fast, but without an orthophoto nothing on the map can be checked, which defeats the
  candidate layers.
- C. One pilot region first (for example Poznań). Little gain, because the services are national and the effort per
  region does not drop. The pilot is served by the example maps of stages 2 and 3.

## Design (stages)

**Stage 1: foundation and OSM.**
- A new ADR: a country level above the region, and national grids. Add EPSG:2180 to `Crs`, forward and inverse, with
  tests against known points (Słubice, Warsaw, Gdańsk).
- UTM 34 as a project grid (see the calls below); the raster warp itself is stage 2.
- Adapter `polska` with the 16 voivodeship OSM extracts (Geofabrik `europe/poland/<voivodeship>`), the choice by
  the map area, merged when the area crosses a voivodeship border. An area that reaches into Germany or another
  country is refused with a message (see the calls below).
- Coverage check per country: an area outside Poland and outside the supported states fails with the usual message.
- Result: sprint and forest base maps from OSM for all of Poland, no templates beyond OSM.

**Stage 2: terrain and orthophoto.**
- The raster warp from 2180 to the project grid (bilinear for heights, bilinear or nearest for orthophoto).
- WFS sheet indexes per year (NMT, NMPT, ORTO), BBOX in N,E; per tile choose the best by rule: LiDAR over
  photogrammetry, 1 m over 5 m, newer over older; ties by year. The rule is data, with a test table.
- NMT 1 m ASC reader (text grid, nodata -9999, pixel-centre corners) and NMPT 0.5 m; nDSM as the difference.
  Orthophoto 0.25 m GeoTIFF (JPEG tiles; check the `tiff` crate, else a decoder).
- Never take the 0.05 m orthophoto; cap the size per area and say the download size first (`estimate_download`).
- Vegetation from the LAZ point cloud is optional and later (point-clouds backend exists for German states).
- Result: templates, contours, hillshade, slope, relief, nDSM, candidate layers for all of Poland. Example maps:
  one sprint and one forest in a Polish city, as acceptance.

**Stage 3: buildings and land use.**
- EGiB national WFS (buildings, GeoJSON) and BDOT10k per powiat (GPKG/GML: forest, grassland, water, built-up,
  watercourses) as the official vector sources, with content-profile rules like the German states.
- Where EGiB limits block a fetch, fall back to BDOT10k buildings (source "EGiB" inside it).
- Example maps rebuilt; credits "Źródło: Główny Urząd Geodezji i Kartografii (geoportal.gov.pl)".

**Stage 4: Polish language.**
- Third language in the Fluent texts of web UI and console, `--lang pl`, system language detection. Manual
  (`docs/manual/pl.md`), README (`README.pl.md`), the area page. Run unslop and humanizer on the user texts
  (the humanizer holds where they disagree). Glossary gets a Polish column first.
- The program texts that still say "Bundesland" are German-state specific; for Poland the word is "województwo".

**Stage 5: permit notes.**
- Polish sources of who is in charge: forest districts (Lasy Państwowe, `nadleśnictwo`), protected areas
  (GDOŚ, Natura 2000, parks), owners via EGiB. Same rule as the German notes: only what applies to the area, rather
  too much when unclear; a hint, not legal advice.

Each stage: ADR per significant decision, changelog entries DE and EN (and PL once it exists), manual update, a
region row in both READMEs, `all-systems` label on the PR (paths, files, projection code), watch CI after merge.

## Left to the builder

- Crate layout (a `polska` crate under `crates/adapters/` or a country directory) and the exact shape of the country
  concept in the manifest, as long as the ADR records it.
- Tile-choice rule details and the cache layout; the warp algorithm.
- Whether stage 2 reads the LAZ cloud or only NMPT.
- The order of work inside a stage.

## Calls made after the builder check (mine, not the user's; change them if wrong)

- **Project grid stays UTM.** Stage 1 adds **UTM 34** (Poland east of about 18°E) next to 32 and 33; EPSG:2180 is
  only a *source* grid that rasters are warped from. The zone of a project is the one of the map area's centroid;
  an area that straddles zones 33 and 34 (Wrocław, Kraków, Warsaw region) uses the centroid's zone and the rest is
  warped into it. Why: the `.omap` georeferencing, OpenOrienteering Mapper and the suite tools (freemapper-print,
  freemapper-courseplan, the contract test) all work on UTM today. The ADR of stage 1 records this.
- **Warp moves to stage 2.** Stage 1 is OSM only and reads no raster, so the warp has no user there. Stage 1 keeps the
  vector projection (2180 and UTM 34 forward and inverse, with tests).
- **Texts before stage 4.** Stages 1 to 3 merge to main with the existing DE/EN texts, but Poland is announced as a
  **preview** (README row, manual, release notes) until stage 4 is done. Texts a Polish user reaches (errors,
  area page) must not say "Bundesland" or name German law; they take the country-neutral wording. The anti-goal binds
  from stage 1 for these texts and from stage 4 for the language.
- **Border areas.** In stage 1 an area must lie wholly in Poland. One that reaches into a German state or another
  country fails with a message that says so, instead of a half-merged map. Border areas with a seam are a later
  stage after stage 5, by the priority rule of `crossborder-reference.md` §4.2. German states already serve
  their side of the Polish border as before.

## Open points, my calls (marked for the reader)

- Polish stays a full program language, not only a README (my call from "Polish users").
- Stage 5 may move behind stage 4 or be dropped if the sources are not open (decided when stage 4 is done).
