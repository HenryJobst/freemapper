# Berlin Plus 2 (Straßenbefahrung, street furniture, building currency) – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Berlin maps use the 2014–2016 survey of the public street space:
- surveyed green strips are cut out of the 501.5 paved areas;
- candidates for curbs, surveyed paved areas outside the ALKIS street parcels, stairs, walls and railings;
- schoolyard furniture, lamps and fountains;
- every ALKIS building carries a footprint date.

**Plan 2 of 2.** Run it after `2026-11-09-berlin-plus.md`. It reuses that plan's extra-layer fetch, the Berlin
rules file and `build/stairs.rs`.

**Architecture:**
- **New layers:** the plan 1 layer table gets the Straßenbefahrung, lighting, fountain and building-height
  layers. They are fetched like the others.
- **Rules:** point and line layers run through `assets/rules/berlin-candidates.toml`.
- **Green cut-out:** `build/paved.rs` gets `subtract_green(objects, green)`. It cuts the surveyed
  `Grünfläche` polygons out of the ALKIS 501.5 objects and keeps the rest unchanged.
- **Paved candidates:** `build/paved.rs` `surveyed_paved_candidates` takes the union of the surveyed surface
  layers minus the ALKIS `Straßenverkehr` parcels. The remainder becomes 501.5 candidates on sprint maps.
- **Stairs:** the surveyed `Treppe` polygons go through `build/stairs.rs`. Their `trs` (probably the number of
  steps; there is no code list) is kept as a tag.
- **Survey date:** each Straßenbefahrung feature's date is parsed from `gis_id` (four spellings) and
  becomes `freemapper:currency`.
- **Building currency:** `ua_gebaeudehoehen:gebaeudehoehen` is fetched with the buildings.
  `grundrissaktualitaet` joins by `gml_id` = `uuid` into the normalised building as `aktualit`, so `updated`
  is no longer `None` in Berlin.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1, §4.5. Verified sources (2026-11-08):
`docs/regions/berlin-plus-reference.md` (§6–§8). Live samples (untrusted, fixtures only):
`<Arbeitsverzeichnis>`.

## Rulings (from §6, §13, §14)

- **The survey never replaces ALKIS.** ALKIS street parcels stay the 501.5 map objects.
  - 10.1 % (Viktoriapark) to 26.7 % (Alexanderplatz km) of the ALKIS street area is not surveyed. That is
    verges and places rebuilt since 2014.
  - The only object change is the cut-out of surveyed green (`ce_gruenflaeche`). The error it can cause is
    an omission (a white spot the mapper fills from the orthophoto), not a wrong object.
- **Tree pits (`bn_baumscheibe`, 2–5 m²) are not cut.** They are below the ISSprOM minimum at 1:4000 and too
  fiddly at 1:1000.
- **No rule depends on an undocumented code:** `material`, `hms`, `pot`, `bor`, `gws`, `sre`, `hdt`.
  - The codes are copied into tags for the mapper.
  - Code lists are an open question (§14.5).
- **Curbs (`bd_bordstein`) are 501.1 candidates on sprint maps only.** ALKIS `Fahrbahnbegrenzungslinie` stays
  unused (plan 1).
- **Bollards, masts and flag poles are 544 candidates only on the schoolyard.** On sprint maps they would
  flood the candidates (2 392 bollards in one km²).
- **Lamps come from the lighting cadastre** (`beleuchtung:beleuchtung`), not the survey's `as_mast`. On the
  schoolyard the lighting cadastre wins:
  - an OSM `highway=street_lamp` within 2 m is dropped;
  - a survey `Mast` within 2 m of a lamp is dropped.
- **Railings and traffic barriers are 518 (impassable) candidates,** like OSM fences. The mapper downgrades
  them to 516.
- **Footprint dates from `ua_gebaeudehoehen` (2022 data) are better than none.** They are tagged as
  `Grundrissaktualität`, not as a survey date. A building without a match keeps `None`.

## Global Constraints

- **Region gate:** the same as plan 1 (Berlin region only).
- **Layers (§6, §7, §8):**
  | file | service | type | kinds | symbol |
  |---|---|---|---|---|
  | `sb-gruenflaeche` | `https://gdi.berlin.de/services/wfs/strassenbefahrung` | `strassenbefahrung:ce_gruenflaeche` | sprint | cut + 401 cand. |
  | `sb-flaechen` | same | `cm_fahrbahn`, `cl_gehweg`, `ch_radweg`, `ck_parkflaeche`, `ci_oeffentlicher_platz`, `cj_fussgaengerzone`, `by_gehwegueberfahrt` | sprint | 501.5 cand. outside ALKIS |
  | `sb-bordstein` | same | `bd_bordstein` | sprint | 501.1 |
  | `sb-treppe` | same | `cc_treppe` | both | 532.6–.9 |
  | `sb-mauer` | same | `bh_mauer` | both | 515 |
  | `sb-gelaender` | same | `bf_gelaender`, `bb_verkehrsschutzgitter`, `bg_leitplanke` | both | 518 |
  | `sb-moebel` | same | `bj_sitzbank` (543), `aq_hydrant` (321), `av_poller`/`as_mast`/`am_fahnenmast` (544), `aw_spielgeraet` (541) | schoolyard | as listed |
  | `sb-objekte` | same | `ag_werbesaeule` (531, sprint), `br_fahrgastunterstand` (522, both), `af_wasserpumpen_brunnen` + `bv_springbrunnen_zierbrunnen` (311, both), `ac_trinkwasserbrunnen_wasserspender` (313, schoolyard) | as listed | as listed |
  | `beleuchtung` | `https://gdi.berlin.de/services/wfs/beleuchtung` | `beleuchtung:beleuchtung` (`status` starts with `In Betrieb`) | schoolyard | 545 |
  | `zierbrunnen` | `https://gdi.berlin.de/services/wfs/zierbrunnen` | `zierbrunnen:bez_zierbrunnen` | both | 311 |
  | `trinkwasserbrunnen` | `https://gdi.berlin.de/services/wfs/trinkwasserbrunnen` | `trinkwasserbrunnen:trinkwasserbrunnen` | schoolyard | 313 |
  | `gebaeudehoehen` | `https://gdi.berlin.de/services/wfs/ua_gebaeudehoehen` | `ua_gebaeudehoehen:gebaeudehoehen` | both | `aktualit` join |

  Several types under one file are fetched one by one and stored together with `berlin:layer` = the type
  name.
- **Survey date from `gis_id`:** the forms `20150413`, `13.04.2015`, `2015-04-13` and `16072014` (DDMMYYYY
  when the first four digits are not a plausible year 2010–2020) all give `freemapper:currency`. An
  unparsable id gives the dataset date 2014-01-01 and the tag `freemapper:currency_guessed=yes`.
- **Thresholds:**
  - `GREEN_MIN_SIDE_MM = 1.0` (as `out_of_bounds.rs`);
  - `SURVEYED_PAVED_MIN_SIDE_MM = 1.0`;
  - `FURNITURE_DEDUP_M = 2.0`;
  - curb lines shorter than `MIN_LINE_MM` (1 mm on paper) are dropped, as in `candidates.rs`.
- **MultiPoint** (`beleuchtung`) becomes one point per member.
- **Attribution:**
  - `Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert`;
  - `Geoportal Berlin / Öffentliche Beleuchtung, Zierbrunnen, Trinkwasserbrunnen, dl-de/zero-2-0`;
  - `Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0`.
  Each line appears only when its layer contributed.
- **Download estimate:** +4 MB per km² (survey) and +1.3 MB per km² (building heights) in the centre.

## Review Focus

1. **The green cut-out:**
   - only `ce_gruenflaeche` is cut;
   - slivers are removed;
   - the 501.5 objects keep their provenance;
   - areas outside Berlin are unchanged.
2. **Surveyed paved candidates:**
   - the difference with ALKIS is computed in map CRS;
   - park paths in the Viktoriapark appear as candidates;
   - no candidate overlaps a 501.5 object.
3. **Currency:** all four `gis_id` spellings parse, and the guessed date is tagged.
4. **Candidate volume:** sprint maps carry no bollards or masts, and the schoolyard dedup against OSM works.
5. **Building `aktualit`:**
   - the join is by uuid;
   - an unmatched building keeps `None`;
   - Brandenburg buildings are untouched;
   - the "changed upstream" logic ignores the new field on old caches.

---

### Task 1: Survey and point layers fetch
Tests:
- `survey_layers_only_inside_berlin`
- `several_types_share_one_file`
- `gis_id_dates_parse`
- `multipoint_lamps_split`

### Task 2: Green cut-out and surveyed paved candidates
Tests:
- `surveyed_green_is_cut_from_paved_area`
- `tree_pits_are_not_cut`
- `surveyed_paving_outside_alkis_is_candidate`
- `no_candidate_overlaps_paved_object`

### Task 3: Curbs, stairs, walls, railings, street objects
Tests:
- `curbs_are_edge_candidates_on_sprint_only`
- `surveyed_stairs_keep_step_count`
- `railings_are_impassable_fence_candidates`
- `bollards_only_on_schoolyard`
- `advertising_column_is_531`

### Task 4: Lamps, fountains, building currency
Tests:
- `lamp_cadastre_replaces_osm_lamp_nearby`
- `fountains_from_both_sources_dedup`
- `building_gets_footprint_date`
- `unmatched_building_keeps_no_date`
- `berlin_plus_2_attribution_lines`

### Task 5: Docs, ADRs, changelog, example, live run, review
- `docs/regions/berlin-plus-reference.md` §6–§8 as built.
- Spec §4.1 and §4.5.
- Manual chapters 12 and 14 in German and English (unslop + humanizer): the Berlin paved-area note gets the
  green cut-out and the survey age.
- ADRs:
  - the survey is used only for candidates and green cut-outs;
  - furniture on the schoolyard only;
  - footprint dates from the Umweltatlas.
- A changelog entry (DE + EN).
- Live runs:
  - the Helene-Haeusler-Schule schoolyard;
  - the Viktoriapark sprint map;
  - a Berlin/Brandenburg border area (Lichterfelde/Teltow box from `berlin-reference.md` §12).
- Regenerate `examples/berlin-viktoriapark` (OOM, OCAD, PDF, preview) and update its README.
