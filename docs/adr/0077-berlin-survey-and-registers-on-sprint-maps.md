# 0077. Berlin survey and registers on sprint maps

Date: 2026-11-10
Status: Accepted

## Context

ADR 0074 left the Berlin street survey and the lighting cadastre as schoolyard layers, because switching them on changes
which layers a Berlin sprint project fetches. The Viktoriapark example then had only OSM control sites: 114 of its 119
531 candidates. The survey has 1 077 bollards, 35 masts, 10 benches and the lighting cadastre 268 lamps in the same
box, so Berlin sprint maps ignored the better data.

The norm (ISSprOM 2019-2, 2.1) says that waste baskets, fire hydrants, parking meters and individual street lights
should not be mapped. They stay candidates, as in ADR 0074. 4.5 gives 531 "Prominent man-made feature, x" and 4.3 gives
313 "Prominent water feature" for small water features that are significant or prominent.

## Inventory of the Berlin layers

S is the sprint map, H the schoolyard map. "Both" was already so.

| Layer / rule | Symbol | Maps | Verdict for sprint |
| --- | --- | --- | --- |
| walls, retaining walls, fences, hedges (`bauwerke-*`, `vegetation-linien`) | 515, 513.2, 518, 410.1 | both | right |
| monuments (`bauwerke-flaechen`) | 526 | both | right |
| bridges, embankment top (`bauwerke-flaechen`, `relief`) | 501.5, 104 | S | right |
| building parts (passages, arcades) | 522 | both | right |
| street and park trees | 417, 418 | both | right |
| curbs `bd_bordstein` | 501.1 | S | right |
| walls `bh_mauer`, railings, barriers, guard rails | 515, 518 | both | right |
| bus shelters `br_fahrgastunterstand` | 522 | both | right |
| pumps, ornamental fountains, fountain register | 311 | both | right |
| advertising columns `ag_werbesaeule` | 531 | S | right, now also a control site |
| benches `bj_sitzbank` | 543 / 531 | H, now S | changed |
| bollards `av_poller` | 544 / 531 | H, now S | changed |
| masts `as_mast`, flag poles `am_fahnenmast` | 544 / 531 | H, now S | changed |
| lamps, lighting cadastre `beleuchtung` | 545 / 531 | H, now S | changed |
| drinking fountains `ac_trinkwasserbrunnen_wasserspender`, register `trinkwasserbrunnen` | 313 | H, now both | changed |
| hydrants `aq_hydrant` | 321 | H | stays: the norm names hydrants as not mapped, and the type code `hdt` is undocumented (66 of 68 are "2", probably flush covers) |
| play equipment `aw_spielgeraet` | 541 | H | stays: no sprint meaning |
| paved areas, green strips, stairs | | S or both | not touched |

## Decision

- **Control sites from Berlin.** On sprint maps `av_poller`, `as_mast`, `am_fahnenmast`, `bj_sitzbank`, `beleuchtung`
  and `ag_werbesaeule` are 531 candidates with `freemapper:control=bollard|mast|flagpole|bench|street_lamp|advertising_column`
  and the usual `berlin:layer`. `mast`, `flagpole` and `advertising_column` are new kinds. The kind comes from the layer
  in `build/candidates.rs`. Candidates only, never map objects.
- **Benches are lines in the survey.** For the sprint rule the build adds a point at the middle of each bench line
  (`berlin:point=middle`). The schoolyard rule skips that point and draws the line.
- **One thinning for all sources.** Thinning runs in `generalize_points`, after the duplicates of the sources are gone and
  after the points in buildings are dropped. Before, it ran on the OSM points alone. Now a survey bollard and an OSM
  bollard 3 m apart are one object (dedup), and a row of survey bollards is thinned like a row of OSM ones: a site of the
  same kind within 25 m is dropped (ADR 0074). A control site still gives way to every other point symbol and is not
  kept inside a building (ADR 0076).
- **Duplicates.** A surveyed bollard beats an OSM bollard within 2 m. The lighting register beats OSM lamps (5 m) and
  survey masts (2 m), and the survey bench, mast and flag pole beat their OSM twins (2 m), as before.
- **Drinking fountains are 313 on sprint maps**, from the register, the survey and OSM `amenity=drinking_water`. They are
  rare and need no thinning.
- **Fetch.** A sprint project in Berlin now also fetches the survey types `bj_sitzbank`, `av_poller`, `as_mast`,
  `am_fahnenmast` and `ac_trinkwasserbrunnen_wasserspender`, and the layers `beleuchtung` and `trinkwasserbrunnen`
  (the layer list, 26 to 33 requests). The disk estimate for the Berlin layers goes from 10.5 to 12 MB per km². At
  Viktoriapark the new files are 0.54 MB for 0.66 km². Existing projects get the layers with `run --force`. The project
  file records them under `[berlin_layers.*]` as before. Offline, the stored files stay as they are.

## Consequences

- Viktoriapark, from the cache plus one online run for the new layers: the candidates go from 2 325 to 2 354. 531 goes
  from 119 to 146. Of 1 077 surveyed bollards 3 are left, of 35 masts 9, of 268 lamps 91; none of the 10 surveyed
  benches is left (they stand beside trees, and a control site gives way to those). The OSM lamps drop from 74 to 8, because
  the register replaces them.
- Lübeck, which has no Berlin layers, changes through the shared thinning: 531 goes from 180 to 192, because a site in a
  building no longer takes its neighbour out, and one OSM drinking fountain (313) appears.
- A course setter in Berlin gets the lamps of the register, not only the ones OSM happens to have. The survey is from
  2014 to 2016, so every site carries its survey date in `freemapper:currency`, and the check on site (manual) matters
  more than for OSM.
- Not done: OSM `man_made=mast|flagpole` stay schoolyard-only. Hydrants and play equipment stay schoolyard-only.
- Existing maps get this with `run --force`.
