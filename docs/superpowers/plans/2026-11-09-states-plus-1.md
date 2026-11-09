# States Plus 1 (ALKIS passages and canopies everywhere, full ALKIS for Brandenburg, Potsdam trees) – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- Every state outside Berlin gets the ALKIS canopies `Überdachung` and `Carport` as 522 map objects. Its building
  passages become 522 candidates, on both map kinds. All of this comes from the ALKIS vereinfacht buildings
  freemapper already stores, with no new download.
- Brandenburg also gets candidates from the full ALKIS (NAS-konform WFS): fences, wall edges, hedges, embankment
  tops, bridges, monuments, ditches.
- The Potsdam city map trees become 417/418 candidates.
- One NAS reader, one AdV code → ISSprOM rule file and one tree-point reader are built so that the later states
  only add a service row (roadmap `2026-11-09-states-plus-roadmap.md`).

**Plan 1 of the states-plus series.** Berlin output stays unchanged.

**Architecture:**
- **Vereinfacht parts and structures (all states):** the stored `alkis-buildings.geojson` already holds every
  `GebaeudeBauwerk` feature, and `buildings.rs` keeps only `Gebäude` from it.
  - `build/canopy.rs` gets a second input, `CanopyInput::Buildings`: the stored features with `gebnutzbez` =
    `Sonstiges Bauwerk oder sonstige Einrichtung` and `funktion` = `Überdachung` | `Carport`. They run through
    the same cut at the 521 footprints (ADR 0046).
  - The Berlin input (`bauwerke-flaechen`, `bezbwf`) stays as it is.
  - Passages run through the new state rules (below), with `alkis:type=GebaeudeBauwerk`.
  - Features with a `bauart` property are Berlin's (set only by `sources/berlin.rs`). They stay on the Berlin
    rules, so a Berlin building part never reaches the generic path.
- **NAS reader (`sources/alkis_nas.rs`, new):**
  - a service table `NasService` (name, url, region, crs, `max_count`). Plan 1 has the row `LGB_NAS`, and the
    later states add rows.
  - a layer table `NasLayer` (AdV type, file, map kinds);
  - its own GML parser, `parse_nas_gml`. The vereinfacht parser (`alkis_wfs::parse_gml`) reads only
    `geometrie` polygons, and NAS has `position` with curves, arcs, points and surfaces built from curve rings.
  - The fetch is gated by region and stores `data/alkis-nas/<file>.geojson` in the map CRS. Each feature keeps
    its `oid` (= `gml:id`), `nas_type`, its code properties as strings, `beginnt` (date) and `gehoert_zu` (the
    oid behind an `xlink:href`).
  - Project records go in `nas_layers: BTreeMap<String, AlkisRecord>`, like `berlin_layers`.
- **State rules (`assets/rules/state-candidates.toml`, `Rules::states()`):**
  - the same syntax and engine as `osm-candidates.toml` and `berlin-candidates.toml`;
  - tags are `alkis:<property>` (vereinfacht and NAS) and `tree:<field>`, plus `alkis:type`, `alkis:geom` =
    `point` | `line` | `area`, and `tree:dataset`. An OSM rule or a Berlin rule can never match them.
  - Rules match AdV **codes**, not labels. `assets/rules/adv-codes.toml` (code → German label, from the GDI-DE
    registry) gives the label for `freemapper:alkis`.
  - The geometry handling of `build/berlin.rs::tagged_features` moves to `build/tagged.rs`
    (`tagged_features(prefix, layer, features, crs)`), and Berlin calls it with `("berlin", …, Utm33)`.
- **Stairs:** `build/stairs.rs` gets NAS stair areas (1620/1621) and their bottom edges (1630, linked by
  `gehoert_zu`). Brandenburg has none, so this serves Mecklenburg-Vorpommern and Hesse later.
- **Trees (`sources/trees.rs`, new):** a `TreeDataset` table (id, region, envelope, format, field names, skipped
  kinds, licence line). Plan 1 has the row `POTSDAM` and the format `OdsGeoJsonExport`. The WFS, OGC API and CSV
  formats come with the first state that needs them. Trees are stored as `data/trees/<id>.geojson`, with
  `tree_layers` records.
- **Duplicates (`build/dedup.rs`, new):** the Berlin drop functions (`drop_lines_near`,
  `drop_osm_trees_near_cadastre`, `drop_points_near_registers`) move here. They take a rank from
  `freemapper:source`: register > ALKIS > survey > OSM (ADR 0055).
  - Register: the Berlin registers and tree layers, `trees/*`.
  - ALKIS: `alkis`, `alkis-nas/*`, the other `geoportal-berlin/*` sources.
  - Survey: `geoportal-berlin/sb-*`.
  - OSM: `osm`.
  The Berlin call sites keep their behaviour.

**Tech Stack:** no new crates (quick-xml, geo, serde_json, toml as today).

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1 (data sources) and §4.5 (symbol
mapping). Verified sources: `STATES-PLUS-SURVEY.md` (2026-11-08), to be copied to
`docs/regions/states-plus-reference.md` with the corrections below. Live samples from 2026-11-09 (untrusted,
fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/states-plus/probe-plan/`.
These are BB NAS for Sanssouci, MV stair samples, the Potsdam export, the LGB vereinfacht buildings, the AdV code
lists `cl-*.json` and the Capabilities `caps-*.xml`.

## Rulings

- **Vereinfacht `Überdachung` and `Carport` are 522 map objects in every state** on both map kinds. ADR 0046 is
  extended from Berlin to all states. They come from the same cadastre as the 521 buildings, and the same cut and
  inside-share rules apply.
  - Tags: `freemapper:source=alkis`, `freemapper:source_id=<oid>`, `freemapper:currency=<aktualit>` (the
    feature date, better than the fetch date) and `freemapper:alkis`.
  - Underground ones (`rellage` = `Unter der Erdoberfläche`) are skipped.
- **Passages** (`Durchfahrt im Gebäude`, `Durchfahrt an überbautem Verkehrsweg`, `Arkade`) are 522 candidates,
  and the 521 stays whole (ADR 0047). `Auskragender Geschossteil` is dropped.
- **Spellings are compared by `lgv::spelling_key`** (case, umlauts, no-break space, `-`/`,`/`/`), made
  `pub(crate)`.
  - Hamburg stores `Sonstiges Bauwerk Oder Sonstige Einrichtung` (example data). `lgv::BUILDING_KINDS` gets the
    third kind.
  - The build compares by key, so projects stored before this change need no new fetch.
- **Saxony and Saxony-Anhalt gain nothing from the rule-only part.** This was verified 2026-11-09 with WFS
  `RESULTTYPE=hits` over the whole state:
  - Saxony vereinfacht: 0 `Bauteil`, 0 `Sonstiges Bauwerk…`;
  - Saxony-Anhalt: 402 `Bauteil`, 0 passages, 0 canopies.
  This corrects survey finding 1. No error, no warning: the data has none.
- **No canopies or building parts from NAS.** `AX_Bauteil` is not fetched, and the codes 1610/1611 have no rule.
  - The vereinfacht `oid` is the NAS `gml:id` plus `BL`. This was verified on all 6 Sanssouci canopies, e.g.
    `DEBBAL540001vmqB` / `DEBBAL540001vmqBBL`. The same objects would otherwise be drawn twice.
- **Rules match codes, never labels.** The AdV code lists are nationwide. Labels are only for the mapper
  (`freemapper:alkis=Mauerkante, rechts`).
- **Symbols follow the Berlin and OSM convention** (ADR 0047), and height decides nothing:
  - walls → 515;
  - retaining walls → 513.2;
  - fences → 518;
  - hedges → 410.1;
  - monuments → 526 points (`freemapper:size=large` above 3 m);
  - bridges → 501.5 on sprint maps;
  - embankment top (`Strukturlinie3D` 1220) → 104 on sprint maps.
  Both edges of a wall stay (ADR 0047, the mapper deletes extras).
- **NAS currency is `lebenszeitintervall/beginnt`** of the object version (first 10 characters). It is the date
  of the cadastre entry, not a survey date. The fetch date is used only when it is missing.
- **No NAS service pages.** `ImplementsResultPaging` is `FALSE` in all eight services checked, so there are no
  `STARTINDEX` requests. One GetFeature per type with `COUNT=max_count`.
  - The bbox is split into 4 quadrants when `numberReturned < numberMatched`, or when `numberMatched` is
    `unknown` and `numberReturned >= max_count`. Duplicates by `gml:id` are dropped.
- **Stairs from NAS:**
  - the bottom edge (1630) gives the direction: the stair runs uphill away from it. It is not `guessed`.
  - Without an edge, the DGM1 decides as in ADR 0048.
  - The symbol is chosen by paper width (ADR 0048).
  - Brandenburg has no stairs in NAS (1620/1621/1630: 0 statewide, verified), so this is fixture-tested on
    Mecklenburg-Vorpommern-shaped data.
- **Potsdam trees are all 418.** The data has no crown diameter (`ggartbez` Laubbaum, Nadelbaum, Obstbaum,
  `Einzelgebüsch, Strauch`), and `Baumstumpf` is skipped.
  - A crown rule (`tree:crown_m>=10` → 417, ADR 0051) is in the file for later datasets.
  - Currency is the per-tree `datum` (1993–2026), not the fetch date.
- **Duplicates:** register > ALKIS > survey > OSM, for the new sources too. Distances and shares are as in ADR
  0047/0051/0055.
  - An OSM 515/513.2/518/410.1/305/306 line with ≥ 80 % of its length within 1.5 m of a NAS line with the same
    symbol is dropped and named in `freemapper:also`.
  - An OSM `natural=tree` within 2 m of a Potsdam tree is dropped.
  - A NAS tree (none in BB) within 2 m of a register tree is dropped.
  - 305/306 join `LINE_DEDUP_CODES`. No Berlin rule emits them, so Berlin is unchanged.
- **Out of scope here:** Basis-DLM, tree rows (`Baumreihe` 1210–1230: 0 in BB), `AX_WegPfadSteig`, towers,
  Frankfurt (Oder) trees. They are in the roadmap.

## Global Constraints

- **Berlin unchanged:**
  - `berlin-candidates.toml`, the Berlin layers and the Berlin canopy input are untouched;
  - the Viktoriapark example re-exports byte-identical (OOM and `candidates.omap`, apart from the timestamp, if
    any).
- **Region gate:** NAS layers are fetched only for regions in the NAS service table (plan 1: `Region::Brandenburg`
  via `regions_for(region_bbox)`). Potsdam trees are fetched only when the region bbox intersects the envelope
  E 359 000–375 000, N 5 801 000–5 820 000 (EPSG:25833, data extent rounded outward). A Berlin-only area makes
  no new request.
- **LGB NAS service (verified 2026-11-09):**
  - `https://isk.geobasis-bb.de/ows/alkis_nas_wfs`, WFS 2.0.0, GML 3.2 only (no GeoJSON), namespace
    `http://www.adv-online.de/namespaces/adv/gid/7.1`;
  - `SRSNAME`/`BBOX` `urn:ogc:def:crs:EPSG::25833`;
  - `max_count = 100_000` (`CountDefault`), at most 3 attempts per request (`get_with_retries`);
  - an `ows:ExceptionReport` is an error for that layer, and the layer is skipped with a warning.
- **NAS layers (BB):**
  | file | type | kinds | Sanssouci 0.54 km² |
  |---|---|---|---|
  | `sonstiges-bauwerk` | `adv:AX_SonstigesBauwerkOderSonstigeEinrichtung` | both | 295 (1740 ×136, 1702 ×84, 1701 ×69, 1610 ×6), 329 KB |
  | `vegetationsmerkmal` | `adv:AX_Vegetationsmerkmal` | both | 96 (1100 Hecke), 103 KB |
  | `strukturlinie` | `adv:AX_Strukturlinie3D` | sprint | 97 (1220 ×47, 1230 ×47, 1240 ×3), 321 KB |
  | `bauwerk-verkehr` | `adv:AX_BauwerkImVerkehrsbereich` | sprint | 4 (1800 Brücke) |
  | `bauwerk-gewaesser` | `adv:AX_BauwerkImGewaesserbereich` | both | 1 (2050 Wehr) |
  | `untergeordnetes-gewaesser` | `adv:AX_UntergeordnetesGewaesser` | both | 0 |

  `AX_BoeschungKliff` has no geometry property (exception), so it is not fetched.
- **GML geometry (`position`):**
  - `gml:Point/pos` → Point;
  - `Curve`, `CompositeCurve`, `LineString`, `OrientableCurve` with `LineStringSegment`, `Arc`, `ArcString`
    and `Circle` segments → one LineString. Segments are concatenated, and an arc through 3 points is
    densified at `ARC_STEP_DEG = 10.0`.
  - `Surface`/`PolygonPatch`/`Ring`/`curveMember` and `Polygon`/`LinearRing`/`MultiSurface` → Polygon or
    MultiPolygon;
  - `srsDimension="3"` → every third value is dropped (BB serves 2D, verified);
  - an unknown element → the feature is skipped and listed, not an error.
- **Split:** at most `NAS_SPLIT_MAX_DEPTH = 4` levels (1/256 of the area). Beyond that the error
  `<service> returned <n> of <m> features; choose a smaller area` is raised.
- **Codes → symbols** (`alkis:type` without `AX_`; draw; kinds):
  | type | property | codes | symbol | draw | kinds |
  |---|---|---|---|---|---|
  | SonstigesBauwerkOderSonstigeEinrichtung | bauwerksfunktion | 1700, 1701, 1702, 1703 | 515 | line | both |
  | 〃 | 〃 | 1720, 1721, 1722, 1723 | 513.2 | line | both |
  | 〃 | 〃 | 1740 | 518 | line | both |
  | 〃 | 〃 | 1750, 1760–1763, 1770 | 526 | point | both |
  | 〃 | 〃 | 1780–1783 | 311 | point | both |
  | 〃 | 〃 | 1620, 1621 (+ 1630 edges) | 532.6–532.9 | `build/stairs.rs` | both |
  | Vegetationsmerkmal | bewuchs | 1100–1103 | 410.1 | line | both |
  | 〃 | 〃 | 1011, 1012 | 418 | point | both |
  | Strukturlinie3D | art | 1220 | 104 | line | sprint |
  | BauwerkImVerkehrsbereich | bauwerksfunktion | 1800–1808, 1820 (`alkis:geom=area`) | 501.5 | area | sprint |
  | BauwerkImGewaesserbereich | bauwerksfunktion | 2136 | 515 | line | both |
  | UntergeordnetesGewaesser | funktion | 1010, 1013 (`hydrologischesMerkmal` 2000/3000 too) | 306 | line | both |
  | 〃 | 〃 | 1030 (not 2000/3000) | 305 | line | both |
  | GebaeudeBauwerk (vereinfacht) | gebnutzbez=Bauteil, funktion | the three passage values | 522 | area | both |
  | tree points | `tree:kind!=Baumstumpf`, `tree:crown_m>=10` | — | 417 | point | both |
  | 〃 | `tree:kind!=Baumstumpf` | — | 418 | point | both |

  Everything else is skipped: 1610/1611, 1622, 1640–1670, 1790, 1791, 9999, and Strukturlinie 1230/1240.
- **Thresholds:** the canopy constants of `canopy.rs` (`CANOPY_MIN_SIDE_MM = 0.5`, `SIMPLIFY_MM = 0.1`,
  `INSIDE_BUILDING_SHARE = 0.9`), the line join and 1 mm drop of `candidates.rs` (`JOIN_TOUCH_M = 0.1`,
  `JOIN_MIN_COS = 0.866`, `MIN_LINE_MM = 1.0`, ADR 0056), and the dedup values `LINE_DEDUP_SHARE = 0.8`,
  `LINE_DEDUP_M = 1.5`, `TREE_DEDUP_M = 2.0`, 3 m for fountains. No new area cut-outs, so the 2 m cap of ADR
  0053 is not used.
- **Provenance tags:**
  - NAS: `freemapper:source=alkis-nas/<file>`, `freemapper:source_id=<gml:id>`,
    `freemapper:currency=<beginnt>`, `freemapper:alkis=<label>`, `alkis:type=AX_…`;
  - passages: `freemapper:source=alkis`, `alkis:type=GebaeudeBauwerk`;
  - trees: `freemapper:source=trees/potsdam`, `freemapper:source_id=<pktnr>`, `freemapper:currency=<datum>`,
    `freemapper:baum=<kind or species>[, Krone <n> m]`.
  `osm_objects` copies `alkis:type` and `tree:dataset` like `berlin:layer` (ADR 0058).
- **Potsdam:** three datasets, `baeume-1-stadtkarte-potsdam`, `-2-` and `-3-` (100 000 + 100 000 + 83 772
  points).
  - URL `https://opendata.potsdam.de/api/explore/v2.1/catalog/datasets/<id>/exports/geojson?where=in_bbox(geo_point_2d,<lat_min>,<lon_min>,<lat_max>,<lon_max>)`.
    The WGS84 bbox comes from the region bbox plus 1 m. There is no paging (export endpoint, verified).
  - Fields: `ggartbez` (kind), `pktnr` (id), `datum` (date), `pktx`/`pkty` (EPSG:25833; the geometry is
    WGS84).
  - Licence dl-de/zero-2-0 (`dl-zero-de/2.0` in the catalogue).
- **Attribution:**
  - the LGB line already covers the NAS data (`© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert`);
  - `Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0` only when a Potsdam tree is in the
    candidates (as `drawn_registers`).
- **Disk and estimate:**
  - new optional `Part::StateLayers` ("the full ALKIS structures and the tree cadastres"), skipped with a
    warning when there is no room. The stored layers stay.
  - `layers.rs`: `NAS_MB_PER_KM2 = 2.0` (Sanssouci measured 761 KB GML for 0.54 km² = 1.4) when a NAS region
    is touched;
  - `TREES_MB_PER_KM2 = 1.0` (Potsdam ≈ 390 B per tree × ≈ 1 460 trees/km² = 0.57) when a dataset envelope is
    touched.
- **Requests:** `User-Agent` as in `fetch.rs`. Offline keeps every stored file. A failing layer is a warning and
  never fails `fetch`.

## Review Focus

1. **Berlin is unchanged:**
   - Berlin parts (`bauart`) never reach the state rules;
   - the vereinfacht canopy input yields nothing for Berlin features;
   - the dedup move keeps every Berlin test green and the Viktoriapark export identical;
   - border areas (Berlin/Brandenburg) get both paths without doubles.
2. **Vereinfacht everywhere:**
   - the spelling-key comparison (Hamburg title case, no-break space, umlauts);
   - underground skipped;
   - the canopy cut at buildings;
   - both map kinds;
   - OCAD gets the 522 objects;
   - `run` reports a kept map that lacks canopies;
   - Saxony and Saxony-Anhalt produce nothing without warnings.
3. **NAS GML reader:**
   - arcs densified the right way round (3-point circle);
   - curve rings closed;
   - 3D handled;
   - the split logic with `numberMatched="unknown"`;
   - `gml:id` dedup across quadrants and regions;
   - skipped features listed;
   - exception reports.
4. **The rule table:**
   - codes, not labels;
   - no NAS 1610/1611 or `AX_Bauteil`;
   - area rules require `alkis:geom=area`;
   - sprint-only types are not fetched for schoolyards;
   - provenance and `freemapper:alkis` labels;
   - short fence segments joined before the 1 mm drop.
5. **Trees and gates:**
   - no request outside Brandenburg or the Potsdam envelope;
   - the bbox order in `in_bbox` is lat/lon;
   - stumps skipped;
   - OSM trees within 2 m dropped, unnamed as in ADR 0051 (ADR 0063);
   - attribution only when drawn;
   - the disk part and the estimate.

---

### Task 1: Vereinfacht passages and canopies in every state
- [ ] `lgv::spelling_key` `pub(crate)`. `lgv::BUILDING_KINDS` += `Sonstiges Bauwerk oder sonstige Einrichtung`.
- [ ] `build/canopy.rs`: `CanopyInput::{Berlin(&[Value]), Buildings(&[Value])}`; `canopy_objects` loops over the
      inputs. `stored_berlin_structures` and `map_lacks_canopies` use both.
- [ ] `build/tagged.rs` (moved from `build/berlin.rs`) and `build/state.rs`: `state_objects` for the stored buildings
      without `bauart`, through `Rules::states()`; `state-candidates.toml` with the passage rule.
- [ ] Wire into `starter_documents`: canopies after the buildings, passages into the candidates.

Tests:
- `vereinfacht_canopies_are_map_objects` (MV fixture: `Überdachung`, `Carport` → 522, currency `aktualit`)
- `hamburg_structure_spelling_is_normalised`
- `passages_are_canopy_candidates_in_every_state` (HE, SH, NW, MV fixtures; `Auskragender Geschossteil` dropped)
- `underground_parts_are_neither_canopy_nor_passage`
- `berlin_parts_stay_on_the_berlin_rules`
- `kept_map_without_state_canopies_is_reported`
- `ocad_export_has_state_canopies`

### Task 2: NAS GML reader and the Brandenburg service
- [ ] `sources/alkis_nas.rs`: `NasService`, `NasLayer`, `parse_nas_gml`, `fetch_nas_layers` (gate, split, oid dedup,
      reprojection into the map CRS).
- [ ] `project.rs`: the fetch step after the ALKIS buildings, `nas_layers` records, `data/alkis-nas/`, a warning per
      failed layer, offline keeps.
- [ ] `space.rs` `Part::StateLayers`; `layers.rs` NAS item.
- [ ] Fixtures from the Sanssouci samples (trimmed): `brandenburg-nas-sonstiges-bauwerk.gml` (line, Arc, Surface),
      `brandenburg-nas-vegetation.gml`, `brandenburg-nas-strukturlinie.gml`.

Tests:
- `nas_curve_with_arc_becomes_line`
- `nas_surface_of_curve_rings_becomes_polygon`
- `nas_3d_poslist_drops_height`
- `nas_feature_keeps_codes_date_and_relation`
- `nas_answer_over_the_cap_is_split_into_quadrants` (also with `numberMatched="unknown"`)
- `nas_layers_only_in_nas_states` (Berlin-only: 0 requests; Strausberg: 6 layers; schoolyard: 4)
- `nas_exception_report_skips_the_layer`
- `nas_estimate_and_disk_part`

### Task 3: State rules, AdV codes, NAS candidates, duplicate rank
- [ ] `assets/rules/adv-codes.toml` (the code lists in the table, labels from `cl-*.json`) and the NAS rules in
      `state-candidates.toml`. Tagging adds `alkis:geom`.
- [ ] `build/dedup.rs`: the rank from `freemapper:source`, the moved Berlin functions, 305/306 in the line codes.
- [ ] `starter_documents`: state candidates join the dedup chain (ALKIS, survey, OSM) and `drop_points_in_water`.

Tests:
- `adv_codes_map_to_issprom` (one case per table row)
- `nas_canopies_and_building_parts_are_not_read`
- `area_rules_need_a_polygon`
- `nas_provenance_and_label`
- `nas_fence_segments_are_joined`
- `osm_lines_near_nas_lines_are_dropped`
- `dedup_rank_from_provenance`
- `berlin_dedup_unchanged`

### Task 4: NAS stairs
- [ ] `build/stairs.rs`: `StairInput::nas` (areas 1620/1621 and edges 1630 by `gehoert_zu`). The direction comes from
      the edge, else the DGM1 (ADR 0048). `data/terrain.tif` is also written when a stored NAS stair has no edge.
- [ ] Fixture `mecklenburg-vorpommern-nas-stairs.gml`: one Rostock 1620 area plus the 1630 edge that names it (one
      small filtered request; take matched pairs).

Tests:
- `nas_stair_runs_up_from_its_bottom_edge`
- `nas_stair_without_edge_uses_the_dtm`
- `edge_of_a_stair_outside_the_area_is_ignored`
- `terrain_written_for_nas_stairs_without_edge`

### Task 5: Tree-point reader and Potsdam
- [ ] `sources/trees.rs`: `TreeDataset`, `TreeFormat::OdsGeoJsonExport`, normalisation to `tree_*` properties, the
      envelope gate, `tree_layers` records, `data/trees/potsdam.geojson`.
- [ ] Tree rules in `state-candidates.toml`. `dedup.rs` treats `trees/*` as a register for trees. Attribution
      line; `layers.rs` trees item.
- [ ] Fixture `potsdam-trees.geojson` (from `pdm-export.geojson`, plus one stump).

Tests:
- `potsdam_url_has_wgs84_bbox`
- `trees_only_inside_the_dataset_envelope`
- `tree_point_keeps_kind_id_and_date`
- `stump_is_no_candidate`
- `osm_tree_near_city_tree_is_dropped`
- `register_tree_beats_nas_tree`
- `potsdam_attribution_only_when_drawn`
- `trees_estimate`

### Task 5b: Point symbol generalisation

User decision 2026-11-09, extended the same day: the point symbols freemapper adds (candidates and map objects)
keep the norm's gap. A dense group becomes the norm's group symbol where one exists, else the smaller symbol
goes. Points inside 520 are hidden. This applies to every point source: the Berlin cadastre trees 417/418 by crown
≥ 10 m (ADR 0051), OSM, ALKIS/NAS, survey, the Berlin registers, Potsdam via the Task 5 reader, and later cities.
The training PDF draws the candidates as map objects, so this matters most there.
- **Exception to "Berlin unchanged":** this task changes the Berlin candidates (Viktoriapark `candidates.omap` and
  the training PDF) on purpose. The Berlin map objects stay byte-identical.

**Norm values (ISSprOM 2019-2, January 2022):**
- §3.1: "When a map is enlarged, all lines, symbols and screens shall be enlarged proportionally." So every paper
  value below is multiplied by `MapKind::symbol_enlargement()` (schoolyard 1.25, sprint 1.0), except the native
  school symbols (321, 542, 544, 545), whose sizes in `school-2019-additions.xml` are already the school sizes.
- §3.3: "Gap between two line symbols of the same colour: 0.15 mm" and "Gap between line symbols and area symbols of
  the same colour: 0.15 mm". The norm names no gap between point symbols.
  - **Ruling:** `POINT_GAP_MM = 0.15` between the outer edges of any two point symbols, whatever their colours.
  - Schoolyard: 0.1875 mm.
- §4.4 417 Prominent large tree: green circle 1.2 (OM), line 0.25, "White mask … (OM 1.4 mm)", footprint 4.8 m
  (OM 5.6 m). This is the errata of 02.01.2022: "1.0 (1.2) → 1.2 (1.4) mm".
  - The bundled OOM set still draws the old 1.0/1.2. Its description says "Footprint: 4 m … (OM 4.8 m)".
  - The gap uses the norm value 1.4. Updating the symbol set is out of scope here.
- §4.4 418 Prominent bush or small tree: ø 0.8, footprint 3.2 m. The bundled set has the same: inner radius 100 µm
  white, ring 300 µm green.
- §4.4 402 Open land with scattered trees:
  - definition: "An area of meadows with scattered trees or bushes, with grass or similar ground cover offering very
    good runnability. … Symbols … (417) and … (418) may be added.";
  - dots ø 0.6 at 0.75 mm (CC), 45°, which leaves π·0.3²/0.75² ≈ 50 % of the area white;
  - "Minimum width: 2.2 mm", "Minimum area: 6.25 mm² (footprint 100 m²)", "Smaller areas must be left out,
    exaggerated or shown using the symbol Open land (401)";
  - OOM `min_area="6300"`.
- §4.4 405 Forest:
  - "Typical open forest for the particular type of terrain.";
  - "Minimum width (for openings): 0.3 mm";
  - "Minimum area: 1 mm² (footprint 16 m²) for openings in screens of other colours, except for … (408) and Open land
    (401), where the minimum area is 0.5 mm² (footprint 8 m²)";
  - OOM `min_area="500"`.
- §4.5 501.3 Paved area with scattered trees: "Area with scattered trees or bushes in a paved area." "Minimum width:
  2.2 mm", "Minimum area: 6.25 mm² (footprint 100 m²)", "Smaller areas must be left out, exaggerated or shown using …
  Paved area (501)".
- §4.5 520 Area that shall not be entered: "No feature shall be represented in this area, except very prominent
  features such as railways, large buildings, or very large trees." Yellow 100 % / green 50 % (olive, colour 22).
- §4.2 207 Boulder cluster and 208 Boulder field are the norm's only other group forms for point features (of 204
  Boulder). §4.4 419 has no group form. Water features (311–313) and man-made points (526, 530, 531) have none
  either.

**Rulings:**
- **Size on paper** = the larger of the norm's outer measure (OM) and the drawn extent of the bundled symbol.
  `POINT_SIZES_MM` in `build/points.rs`:
  | code | symbol | size mm (1:4000) | schoolyard mm | note |
  |---|---|---|---|---|
  | 311 | small fountain or well | 0.90 | 1.125 | OM 0.9 |
  | 312 | spring | 1.20 | 1.50 | OM 1.2 × 0.6, longer side |
  | 313 | prominent water feature | 1.45 | 1.8125 | OM 1.2, drawn 1.45 |
  | 321 | hydrant (school) | — | 1.30 | native school size |
  | 417 | prominent large tree | 1.40 | 1.75 | norm mask, drawn 1.2 |
  | 418 | prominent bush or small tree | 0.80 | 1.00 | |
  | 526 | cairn, memorial, small monument | 1.00 | 1.25 | OM 1.0 |
  | 531 | prominent man-made feature – x | 1.26 | 1.575 | OM 1.2, drawn 1.26 |
  | 542 | table (school) | — | 1.72 | native |
  | 544 | pole, small pylon (school) | — | 1.10 | native |
  | 545 | lamppost (school) | — | 1.40 | native |

  These are all the point codes of `berlin-candidates.toml`, `osm-candidates.toml` and `state-candidates.toml` today.
  `build/` emits no other point symbols. Short-line points (`point_as_short_line_mm`, e.g. 543 benches) are lines
  and stay out.
- **Minimum centre distance** of two points a, b on paper: `(size_a + size_b) / 2 + POINT_GAP_MM × enlargement`.
  - The code works in map coordinates (1/1000 mm on paper), so the map scale only enters through the crowns
    (ground m × 10⁶ / scale).
  - Sprint 1:4000: 418–418 0.95 mm = 3.8 m, 417–418 1.25 mm = 5.0 m, 417–417 1.55 mm = 6.2 m.
  - Schoolyard 1:1000: 418–418 1.19 m, 417–418 1.56 m, 417–417 1.94 m, 544–544 1.29 m, 545–544 1.44 m.
- **"Smaller"**, compared in this order:
  1. paper size;
  2. for two trees, the crown diameter (`freemapper:crown_m`, else the default below);
  3. the rank of `dedup::rank` (register > ALKIS > survey > OSM). A map object beats every candidate.
  4. `freemapper:source` + `/` + `freemapper:source_id`, lexicographic, then the map x and y.
- **Group symbols:** only trees have one in what freemapper emits (417/418 → 402, 405 or 501.3). Every other code
  is thinned.
  - A later boulder source (204) maps to 207 for a cluster and 208 for a field; that is not built here.
- **Crown diameter:** `freemapper:crown_m` (new tag) is set by:
  - Berlin, from `kronedurch` > 0;
  - the tree reader, from `tree:crown_m`;
  - OSM, from `diameter_crown` (metres) when it is present.
  Without one, the default is `DEFAULT_CROWN_M`: 417 → 12 m (the Berlin rule is ≥ 10 m), 418 → 5 m.
- **402, 405 or 501.3** (the norm gives no canopy share, so this is a ruling):
  - Group region G = the union of the crown discs U, closed by `GROUP_CLOSE_MM = 0.5` × enlargement on paper (buffer
    +d then −d). Cover = area(U) / area(G).
  - At least half of G on a drawn 501.x paved map object (`paved.map`) → **501.3**.
  - Otherwise, cover ≥ `FOREST_COVER = 0.5` → **405**. The 402 screen itself is half white, so a canopy denser than
    the screen shows is forest.
  - Otherwise → **402**.
- **Minimum sizes** (paper; on the schoolyard, × 1.25 for widths and × 1.25² for areas):
  - 402 and 501.3: 6.25 mm², mean width (2 · area / perimeter, `out_of_bounds::mean_width`) 2.2 mm;
  - 405: 1.0 mm² (the stricter value: the build does not know the surrounding screen), width 0.3 mm.
  - An area below its minimum is dropped (the norm's "left out"), and its trees go back to the thinning.
- **A group needs `GROUP_MIN_TREES = 3` trees.** Two close trees are always thinned.
- **What the group's trees become:**
  - inside 405 and 501.3, all of them become the area;
  - inside 402, the 417 trees stay points (the norm says they "may be added") and go through the thinning, and the
    418 trees become the area.
- **520:** every point the build adds whose centre lies inside a 520 area (map object or candidate) is hidden, for
  every code. The norm allows no feature there.
  - Its exception "very large trees" needs a judgement the data cannot give, so the mapper adds such a tree back.
  - On the schoolyard map this hides every point off the school grounds.
- **Out of scope:** gaps between points and lines or areas (walls, buildings), and moving symbols apart (the norm's
  "displaced"). The mapper does both.

**Algorithm (`build/points.rs`, new: `generalise_points(candidates, map, out_of_bounds, paved, footprints,
water_union, clip, kind, georef, symbol_set)`):**
1. Hide: drop every point candidate and point map object whose centre is inside a 520 polygon (from the map and the
   candidates). Use a bbox prefilter, then `Contains`.
2. Cluster the trees (417/418 point objects):
   - single linkage: two trees closer than their minimum centre distance are in one cluster;
   - a grid of cells of the largest pair distance, as in `drop_osm_trees_near_cadastre`;
   - chains are transitive (A–B close, B–C close, A–C far → one cluster).
3. For each cluster with ≥ 3 trees:
   - U = the crown discs (`CROWN_SEGMENTS = 32`); G = the closing of U; cover; choose the symbol;
   - clip G to the clip window (`LandClip`) and subtract the 520 areas, the 521 footprints and the water union;
   - simplify at `SIMPLIFY_MM = 0.1` as in `canopy.rs`;
   - each part that meets its minimum becomes an area candidate, and the trees whose centres lie in the kept parts
     are removed (402: only the 418);
   - a cluster without a kept part stays as points.
4. Thin every remaining point (all codes together, trees too):
   - sort by "smaller", largest first;
   - keep a point when it keeps the minimum centre distance to every point kept so far, else drop it;
   - this greedy pass settles ties and chains: with A–B–C and B the largest, B stays and A and C go; with A the
     largest, A and C stay;
   - map points come first and are never dropped by a candidate. Two map points are thinned among themselves.
   - The kept point lists the points it dropped in `freemapper:generalised=<source>/<source_id>; …`, sorted.
5. Return the kept candidates, the new area candidates and the kept map objects.

**Tags of a group area:**
- `freemapper:source=generalisation`;
- `freemapper:source_id=<source>/<source_id>; …` for every merged tree, sorted (the form of `freemapper:also`,
  ADR 0058);
- `freemapper:currency=<oldest currency of the merged trees>`;
- `freemapper:generalised=<n> trees, cover <p> %`.

`drawn_registers` and the Potsdam attribution count a source as drawn when a kept point has it, or when it is a
prefix of an entry in an area's `freemapper:source_id`.

**Where it runs:** in `starter_documents`:
- after every tree and point source and the whole dedup chain (Task 3/5: OSM, Berlin and state candidates,
  `drop_points_in_water`, `drop_shelters_under_canopies`);
- before `drawn_registers`, the attribution and the assembly of the candidates part.
`stored_out_of_bounds_objects` is computed once, before the call. The map points are the `Target::Map` OSM objects
(no point rule targets the map today, so the call is ready for later rules). Both map kinds, OOM, OCAD and the
training PDF get the same result.

- [ ] `build/points.rs`: `POINT_SIZES_MM`, `POINT_GAP_MM`, `DEFAULT_CROWN_M`, `GROUP_CLOSE_MM`, `FOREST_COVER`,
      `GROUP_MIN_TREES`, `generalise_points`. `freemapper:crown_m` in the Berlin, tree-reader and OSM tagging.
- [ ] Wire into `starter_documents`, and move `drawn_registers` and the attribution after the call.

Tests:
- `trees_on_out_of_bounds_are_hidden` (a 418 centre 1 m inside a 520 candidate is gone, the same 0.5 m outside
  stays, and a 520 map object hides too)
- `hydrant_on_out_of_bounds_is_hidden` (schoolyard: a 321 off the school grounds inside the 520 is gone; one on
  the grounds stays)
- `two_close_trees_keep_the_larger` (sprint, 3 m apart: 417 crown 12 m and 418 crown 6 m → the 417 stays and its
  `freemapper:generalised` names the 418)
- `equal_trees_keep_by_rank_then_id` (two 418, 3 m apart: register beats OSM; two OSM: the lower id stays)
- `chain_of_trees_keeps_the_ends` (three 418, 3.5 m apart, crown 1 m: no area, B with the smallest crown goes, A and
  C at 7 m stay)
- `dense_group_becomes_forest_area` (3 × 3 grid of 418 at 3.5 m, crown 6 m → one 405, cover ≥ 50 %, no tree
  point left, `source_id` lists the 9 sorted, currency the oldest)
- `sparse_group_becomes_scattered_trees_area` (5 × 4 grid of 418 at 3.5 m, crown 2 m → one 402 ≥ 100 m², cover
  < 50 %)
- `group_on_paved_area_becomes_paved_with_trees` (the same grid on a 501.5 map object → 501.3)
- `large_tree_stays_on_scattered_trees_area` (a 417 inside the 402 group stays a point)
- `group_area_below_minimum_is_dropped` (three 418, crown 1 m, 3 m apart → no area; one tree stays by the thinning)
- `group_area_is_cut_at_buildings_and_out_of_bounds` (no part of the 405 inside a 521 or 520; the trees under
  the cut stay points or go by the thinning)
- `trees_far_apart_are_unchanged` (418s at 10 m, 417s at 7 m → all kept, objects equal to the input)
- `schoolyard_scale_uses_enlarged_symbols` (two 418 2 m apart: sprint → one; schoolyard → both; schoolyard
  1.1 m → one)
- `close_bollards_keep_one` (schoolyard: two 544 1.0 m apart → one by rank and id; 1.4 m apart → both)
- `different_point_symbols_too_close_keep_the_larger` (schoolyard: 545 and 544 1.0 m apart → the 545; sprint:
  531 and 418 3 m apart → the 531)
- `map_point_beats_candidates`
- `point_sizes_cover_the_drawn_symbols` (every table entry × enlargement ≥ the drawn extent in the loaded set, on
  both kinds; every `draw = "point"` code of the three rule files is in the table)
- `generalised_area_provenance_and_attribution` (tags as above; a Potsdam tree only inside an area still gives the
  Potsdam line)

### Task 6: Docs, ADRs, changelog, example, live run, review
- Copy `STATES-PLUS-SURVEY.md` to `docs/regions/states-plus-reference.md`, with the corrections of 2026-11-09:
  - Saxony and Saxony-Anhalt have no vereinfacht passages or canopies;
  - BB NAS has no stairs, retaining walls or trees;
  - no NAS service pages, with `CountDefault` per service;
  - vereinfacht `oid` = NAS `gml:id` + `BL`;
  - Potsdam has no crown diameter.
- Spec §4.1 (Brandenburg NAS, Potsdam) and §4.5 (522 in every state, the NAS table, trees).
- Glossary (DE/EN first):
  - full ALKIS / NAS (`Vollständiges ALKIS (NAS)`);
  - building function code (`Bauwerksfunktion`);
  - AdV code list;
  - stair bottom edge (`Treppenunterkante`);
  - city map tree points (`Bäume der Stadtkarte`).
- Manual chapters 12 and 14 in German and English (unslop + humanizer). The README keeps one row per state, with
  no detail.
- ADRs:
  - 0059 canopies and passages from ALKIS vereinfacht in every state (extends 0046/0047);
  - 0060 the full ALKIS NAS reader and the AdV code rule table (codes not labels, no NAS canopies, `beginnt`
    currency, no paging);
  - 0061 NAS stairs from the bottom edge;
  - 0062 the tree-point reader and Potsdam trees as a register;
  - 0063 duplicate rank by provenance for every state.
  - one more for the point symbol generalisation (Task 5b: gap, sizes, 402/405/501.3 rulings, 520 hiding). Regenerate
    `examples/berlin-viktoriapark`: its candidates and training PDF change on purpose, its map objects stay
    identical.
- A changelog entry (DE + EN) under `[Unveröffentlicht]`.
- Before the live runs, run `df -h` (SH/BB downloads, `target/`). After the merge, run `cargo clean`.
- Live runs:
  - Park Sanssouci sprint map (`examples/brandenburg-sanssouci`);
  - a Strausberg schoolyard;
  - a Berlin/Brandenburg border area (Lichterfelde/Teltow).
  Compare the candidate counts with the table above.
- Regenerate `examples/brandenburg-sanssouci` (OOM, OCAD, PDF, preview).
  - Re-export `examples/hamburg-blankenese` from its stored data (6 canopies, 4 passages, 3 arcades), with no new
    download.
  - Check that `examples/berlin-viktoriapark` re-exports unchanged.
  - Update `examples/README*.md` (counts, one sentence on the new candidates).
- Final review (superpowers:requesting-code-review) over the five Review Focus items.
