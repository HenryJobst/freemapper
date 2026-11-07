# M5: OSM Candidates – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper run` adds OSM features of the school grounds — hydrants, benches, tables, lamps,
poles, playground equipment, trees, hedges, walls, fences, cliffs, sports pitches — as symbolized
candidates to `templates/candidates.omap`, with provenance, for the mapper to check and copy.

**Architecture:** A Geofabrik extract (`brandenburg-latest.osm.pbf`, ≈ 300 MB) is cached once and
refreshed when Geofabrik's state is more than 7 days newer. A PBF reader (`osmpbf`) pulls the tagged
nodes inside the map area's geographic box and the tagged ways touching it (with their nodes). An
embedded TOML rule file maps tag combinations to symbol codes per map kind. `fetch` stores the matched
features of the project as GeoJSON (`data/osm-candidates.geojson`); `export` turns them into objects in
the candidates template map.

**Tech Stack:** Rust 2024; new crate `osmpbf = "0.3.8"` (pure Rust); existing `toml`, `geo`, `geojson`.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1 (OSM source: Geofabrik extract +
`osmpbf`, no Overpass), §4.5 (candidates, rule file), §4.7 (extract timestamp recorded).
Scope cut: out-of-bounds 520 from ALKIS parcels and contours stay for later plans.

## Global Constraints

- Extract URL `https://download.geofabrik.de/europe/germany/brandenburg-latest.osm.pbf`, state
  `https://download.geofabrik.de/europe/germany/brandenburg-updates/state.txt` (line
  `timestamp=2026-11-06T21\:21\:21Z`). Cache file `<cache>/osm/brandenburg_<YYYY-MM-DD>.osm.pbf`
  (date of the state timestamp); the large download streams to a `.part` file, never into memory.
- Freshness: online, reuse a cached extract when it is at most 7 days older than the state date;
  otherwise download and delete older extracts. Offline: the newest cached extract, else an error.
- Candidates only, in the part `candidates` of `templates/candidates.omap`; the map itself is untouched
  (output policy "Sicheres fertig, Rest Vorlage").
- Tags per candidate: `freemapper:source = osm`, `freemapper:source_id = node/<id>` or `way/<id>`,
  `freemapper:osm = <matched tags, "k=v; k=v">`, `freemapper:currency = <extract date>`.
- Rules (first match wins; `*` any value; `a|b` alternatives; all conditions must hold):

  | Tags | Geometry | Schoolyard | Sprint |
  |---|---|---|---|
  | `emergency=fire_hydrant` | point | 321 | – |
  | `amenity=bench` | point → short line 1.5 mm, or line | 543 | – |
  | `leisure=picnic_table` / `amenity=table` | point | 542 | – |
  | `highway=street_lamp` | point | 545 | – |
  | `man_made=flagpole\|mast` | point | 544 | – |
  | `playground=*` | point → short line 1.8 mm, or line | 541 | – |
  | `natural=tree` | point | 418 | 418 |
  | `barrier=hedge` | line | 410.1 | 410.1 |
  | `barrier=wall` | line | 515 | 515 |
  | `barrier=retaining_wall` | line | 513.2 | 513.2 |
  | `barrier=fence` | line | 518 | 518 |
  | `natural=cliff` | line | 201 | 201 |
  | `leisure=pitch\|track` + `surface=tartan\|asphalt\|concrete` | area (closed way) | 501.10 | 501.10 |
  | `leisure=pitch` + `surface=grass` | area | 401 | 401 |

- Selection: within `buffer_m` of the map area (distance 0 inside), as for buildings.
- Attribution: the OSM line now covers the candidates too: `© OpenStreetMap contributors, ODbL`
  (notes, ATTRIBUTION.txt) and `© OpenStreetMap-Mitwirkende, ODbL` (sheet).
- Test fixture `tests/fixtures/strausberg-school.osm.pbf`: a cut of the real extract around the school
  (ODbL; attribution and the cutting script `tests/fixtures/make_osm_fixture.py` next to it).

## Review Focus

1. **Ways crossing the box edge** – resolved with all their nodes, kept when they touch the buffer
   (Task 2: `way_crossing_the_box_is_complete`).
2. **Unclosed ways for area rules** – skipped, not turned into broken areas (Task 3:
   `unclosed_area_way_is_skipped`).
3. **The 300 MB download interrupted or offline** – no partial extract in the cache; offline uses the
   newest cached extract (Task 1).
4. **Map kind without a symbol for a rule** – the feature is skipped for that kind (Task 3:
   `school_only_rules_skip_sprint`).
5. **Projects fetched before M5** – export works without `data/osm-candidates.geojson` (Task 4).

---

### Task 1: Extract download and cache

**Files:** create `crates/freemapper/src/sources/osm.rs`; modify `sources/mod.rs`, `fetch.rs`.

**Interfaces:**
- `Fetcher::download_to(&self, url: &str, path: &Path) -> Result<(), FetchError>` with a default
  implementation (`get` + write); `HttpFetcher` streams with a body timeout of 2 h.
- `sources::osm::parse_state_date(text: &str) -> Option<NaiveDate>`.
- `sources::osm::fetch_extract(fetcher: &dyn Fetcher, cache: &Cache, mode: FetchMode, progress: &mut dyn
  FnMut(&str)) -> Result<Extract, FetchError>`; `Extract { path: PathBuf, date: NaiveDate }`;
  `FetchError::OsmExtractMissing` ("no OSM extract cached; run `freemapper fetch` online").
- [ ] Tests: `state_date_parses`, `extract_downloaded_then_reused`, `extract_within_seven_days_is_reused`,
  `older_extract_is_replaced`, `offline_uses_newest_extract`, `offline_without_extract_fails`,
  `failed_download_leaves_no_extract`.
- [ ] Commit `feat(osm): Geofabrik extract download and cache`.

### Task 2: PBF reader

**Files:** `crates/freemapper/src/sources/osm.rs`, `crates/freemapper/src/area.rs`, fixture files.

**Interfaces:**
- `area::GeoBox { min_lat, min_lon, max_lat, max_lon }`, `MapArea::geo_box(buffer_m: f64) -> GeoBox`
  (polygon bounds grown by the buffer in degrees).
- `osm::OsmFeature { id: String /* node/1 | way/2 */, tags: Vec<(String, String)>, geometry: OsmGeometry }`,
  `OsmGeometry { Point(LatLon), Line(Vec<LatLon>) }` (a closed way is a line whose last point repeats the first).
- `osm::read_extract(path: &Path, area: &GeoBox, wanted: &dyn Fn(&[(String, String)]) -> bool) -> Result<Vec<OsmFeature>, OsmError>`:
  tagged nodes inside the box (parallel pass), tagged ways via `IndexedReader::read_ways_and_deps` kept
  when a node lies inside the box.
- `osm::features_to_geojson(&[OsmFeature]) -> String`, `osm::features_from_geojson(&str) -> Result<Vec<OsmFeature>, OsmError>`.
- [ ] Tests on the fixture: `fixture_has_school_hydrant`, `fixture_benches_as_nodes_and_ways`,
  `way_crossing_the_box_is_complete`, `untagged_or_unwanted_elements_are_left_out`,
  `geojson_roundtrip`.
- [ ] Commit `feat(osm): PBF reader for the map area`.

### Task 3: Candidate rules and objects

**Files:** create `assets/rules/osm-candidates.toml`, `crates/freemapper/src/build/candidates.rs`.

**Interfaces:**
- `candidates::Rules::embedded() -> Rules`, `Rules::from_toml(&str) -> Result<Rules, RulesError>`,
  `Rules::wants(&self, tags) -> bool`, `Rules::symbol_for(&self, tags, geometry_is_closed: bool, kind: MapKind) -> Option<Match { code, draw: Draw }>`
  (`Draw { Point, ShortLine(mm), Line, Area }`).
- `candidates::candidate_objects(features: &[OsmFeature], rules: &Rules, kind: MapKind, map_area: &Polygon (EPSG:25833),
  buffer_m: f64, georef: &Georeferencing, symbol_set: &SymbolSet, currency: NaiveDate) -> Result<Vec<MapObject>, GeorefError>`.
- [ ] Tests: `rules_parse_and_first_match_wins`, `value_alternatives_and_wildcard`,
  `school_only_rules_skip_sprint`, `hydrant_becomes_321_point`, `bench_node_becomes_short_line`,
  `fence_becomes_518_line`, `tartan_pitch_becomes_501_10_area`, `unclosed_area_way_is_skipped`,
  `features_outside_buffer_are_skipped`, `candidate_tags_carry_provenance`.
- [ ] Commit `feat: OSM candidate rules and objects`.

### Task 4: Project integration

**Files:** `crates/freemapper/src/project.rs`.

**Interfaces:** `OSM_CANDIDATES_RELPATH = "data/osm-candidates.geojson"`; `ProjectFile.osm:
Option<OsmRecord { extract: NaiveDate, count: usize }>`; `fetch_project` (online and offline) reads the
extract for `MapArea::geo_box(buffer_m)` and writes the file; `export` adds the candidate objects to the
`candidates` part.
- [ ] Tests: `fetch_stores_osm_candidates` (fixture extract served as the download),
  `export_adds_candidates`, `export_without_osm_candidates_still_works`, `offline_fetch_reads_cached_extract`.
- [ ] Commit `feat: OSM candidates in fetch and export`.

### Task 5: Docs, live run, acceptance

- [ ] README DE/EN (download size and freshness, what becomes a candidate), spec §4.2 (`data/` file),
  glossary (`Kandidat` exists; add `OSM-Auszug | OSM extract | … | Extract`).
- [ ] `#[ignore] live_osm_extract`: real download, the school's hydrant is found. Regenerate the
  sample and hand it to the user. Commit `docs: M5 OSM candidates`.
