# M4a: ALKIS Buildings as Map Objects – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper run` fetches the ALKIS buildings around the map area and writes them into the map
as 521 building objects with provenance tags, so the starter project already contains the buildings.

**Architecture:** An ALKIS source module downloads `gebaeude_bauwerk` features by bbox from the LGB OGC
API (GeoJSON in EPSG:25833, following `next` links) through the existing `Fetcher`, and stores the
merged FeatureCollection in the project (`data/alkis-buildings.geojson`). A build module classifies the
features (object / dropped), filters them to map area + buffer, drops tiny ones, simplifies outlines and
turns them into OOM area objects. `export_starter_project` puts them into the map's default part.

**Tech Stack:** Rust 2024, existing crates (`geo`, `geojson`, `serde_json`, `ureq` via `Fetcher`).

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` (§4.1 ALKIS, §4.5 Buildings,
generalization). Scope cut: M4 is split; peripheral information (title, scale, attribution text, north
lines, bar scale) follows in plan M4b.

## Global Constraints

- Source code English; docs bilingual; glossary terms (`Building`, `provenance`, `currency`, `candidate`).
- ALKIS endpoint: `https://ogc-api.geobasis-bb.de/alkis-vereinfacht/v1/collections/gebaeude_bauwerk/items`
  with `f=json`, `bbox=<min_e>,<min_n>,<max_e>,<max_n>`, `bbox-crs` and `crs` both
  `http://www.opengis.net/def/crs/EPSG/0/25833`, `limit=10000`. Responses are GeoJSON
  FeatureCollections in EPSG:25833 with `links`; a link with `"rel": "next"` continues the result.
- Feature properties (verified live 2026-11-07 for tile 33421-5821): `oid` (e.g. `DEBBAL010007KdLbBL`),
  `aktualit` (`2023-05-04T10:27:03Z`), `gebnutzbez` (`Gebäude`, `Bauteil`,
  `Bauwerk oder Anlage für Industrie und Gewerbe`), `funktion` (e.g. `Wohnhaus`, `Garage`, `Schuppen`,
  `Allgemein bildende Schule`, `Sport-, Turnhalle`, `Tiefgarage`, `Schornstein`), optional `rellage`
  (`Unter der Erdoberfläche`). Geometry: `MultiPolygon` (sometimes `Polygon`).
- Rule (spec §4.5, verified values): **object 521** if `gebnutzbez == "Gebäude"` and `rellage` is not
  `Unter der Erdoberfläche` (sheds and garages count); **dropped** otherwise (underground parts,
  `Bauteil`, `Bauwerk …` such as chimneys). Canopies (522) are not present in the live data; no rule yet.
- Generalization: drop buildings whose footprint is below (0.5 mm on paper)² — at 1:1000 0.25 m², at
  1:4000 4 m²; simplify outlines with Ramer–Douglas–Peucker at 0.1 mm on paper (0.1 m at 1:1000).
- Selection: buildings within `buffer_m` of the map area polygon (distance 0 when they intersect it).
- Tags per object: `freemapper:source = alkis`, `freemapper:source_id = <oid>`,
  `freemapper:currency = <date part of aktualit>`.
- Existing behaviour stays: an existing map is never overwritten without `--force`.

## Review Focus

1. **More than one result page** – all pages are merged, none fetched twice (Task 1:
   `fetch_follows_next_links`).
2. **Buildings with holes and multipolygons** – every ring becomes part of the object, every part its
   own object (Task 2: `courtyard_building_keeps_its_hole`, `multipolygon_parts_become_objects`).
3. **ALKIS unavailable or a non-GeoJSON answer** – clear error, previously stored buildings untouched
   (Task 1: `invalid_alkis_answer_is_an_error`; Task 3: `failed_alkis_fetch_keeps_stored_buildings`).
4. **Offline run** – uses the stored buildings; without them a clear message (Task 3:
   `offline_fetch_uses_stored_buildings`).
5. **Projects fetched before M4a** – export still works without `data/alkis-buildings.geojson`
   (Task 3: `export_without_buildings_has_empty_map`).

---

### Task 1: ALKIS source

**Files:** Create `crates/freemapper/src/sources/alkis.rs`; modify `sources/mod.rs`,
`crates/freemapper/src/fetch.rs` (`test_support`).

**Interfaces:**
- Produces `sources::alkis::Building { oid: String, kind: String, function: String, underground: bool,
  updated: Option<NaiveDate>, footprint: geo_types::MultiPolygon<f64> }` (EPSG:25833).
- Produces `sources::alkis::buildings_url(bbox: &BoundingBox) -> String`.
- Produces `sources::alkis::fetch_buildings(fetcher: &dyn Fetcher, bbox: &BoundingBox) ->
  Result<String, FetchError>`: GeoJSON text of one FeatureCollection with the features of all pages.
- Produces `sources::alkis::parse_buildings(geojson: &str) -> Result<Vec<Building>, AlkisError>`
  (`AlkisError { message: String }`, Display `"invalid ALKIS buildings: {message}"`).
- `FetchError` gains `Alkis(#[from] AlkisError)`.
- `test_support::MapFetcher` answers a URL by exact match, else by the longest registered prefix
  (`insert_prefix(prefix, bytes)`), so tests can serve bbox URLs without recomputing them.

- [ ] **Step 1: Tests** (`sources/alkis.rs`):
  - `buildings_url_has_bbox_and_crs`: bbox `{421490, 5821580, 421736, 5821896}` → URL starts with the
    endpoint and contains `bbox=421490,5821580,421736,5821896`, `crs=http://www.opengis.net/def/crs/EPSG/0/25833`
    and `limit=10000`.
  - `parse_live_like_features`: a FeatureCollection with the school (`Gebäude`, `Allgemein bildende
    Schule`, MultiPolygon with one square part) and a `Bauteil` `Tiefgarage` with
    `rellage = Unter der Erdoberfläche` → two buildings; the second has `underground == true`; `updated`
    of the first is 2023-05-04.
  - `fetch_follows_next_links`: page 1 has one feature and a `next` link to page 2, page 2 one feature
    and no `next` → `fetch_buildings` returns a FeatureCollection with 2 features; 2 requests.
  - `invalid_alkis_answer_is_an_error`: an HTML body → `Err(FetchError::Alkis(_))`.
- [ ] **Step 2: Run** `cargo test -p freemapper alkis` → fail.
- [ ] **Step 3: Implement.** Parse with `serde_json` (`links[].rel/href`, `features`), geometry via
  `geojson::Geometry` → `geo_types::Geometry` (Polygon → one-part MultiPolygon). Guard against a `next`
  link that repeats an earlier URL.
- [ ] **Step 4: Run** `cargo test --workspace` → pass; commit `feat: ALKIS buildings source`.

### Task 2: Building rules and map objects

**Files:** Create `crates/freemapper/src/build/mod.rs`, `crates/freemapper/src/build/buildings.rs`;
modify `lib.rs`.

**Interfaces:**
- Consumes `Building` (Task 1), `Georeferencing::projected_to_map`, `SymbolSet::id_for_code`,
  `MapObject`, `closed_ring`.
- Produces `build::buildings::Rule { Object, Dropped(&'static str) }` and
  `fn classify(b: &Building) -> Rule` (reasons `"underground"`, `"not a building"`).
- Produces `fn building_objects(buildings: &[Building], map_area: &geo_types::Polygon<f64> (EPSG:25833),
  buffer_m: f64, georef: &Georeferencing, symbol_set: &SymbolSet) -> Result<Vec<MapObject>, GeorefError>`.
- Produces `MapArea::projected_polygon(&self) -> geo_types::Polygon<f64>` (in `area.rs`).

- [ ] **Step 1: Tests** (`build/buildings.rs`; georef at the school, 1:1000; symbol set `school_2019`):
  - `classify_rules`: `Gebäude`/`Garage` → Object; `Gebäude` + underground → Dropped("underground");
    `Bauteil` → Dropped("not a building"); `Bauwerk oder Anlage für Industrie und Gewerbe` →
    Dropped("not a building").
  - `building_inside_area_becomes_521`: a 10 × 20 m square in the area → one object, symbol id of
    `521`, `Path`, 5 coords (4 + closing point flagged 18), tags source/source_id/currency in that order.
  - `buildings_outside_buffer_are_skipped`: one 25 m outside the area (buffer 20) → skipped; one 15 m
    outside → kept.
  - `tiny_building_is_dropped`: 0.4 × 0.4 m at 1:1000 → no object; 0.6 × 0.6 m → object.
  - `outline_is_simplified`: a square with an extra vertex 0.05 m off a side → 5 coords, not 6.
  - `courtyard_building_keeps_its_hole`: square with a square hole → one object with 10 coords, both
    rings ending with flags 18.
  - `multipolygon_parts_become_objects`: two parts → two objects with the same `source_id`.
- [ ] **Step 2: Run** `cargo test -p freemapper buildings` → fail.
- [ ] **Step 3: Implement** with `geo` (`Simplify`, `Area`, Euclidean distance between polygons). The
  tolerances come from `georef.scale_denominator`: `0.5 mm` and `0.1 mm` on paper in metres.
- [ ] **Step 4: Run** `cargo test --workspace` → pass; commit `feat: ALKIS buildings as 521 map objects`.

### Task 3: Project integration

**Files:** Modify `crates/freemapper/src/project.rs`, `crates/freemapper/src/fetch.rs` (`test_support`).

**Interfaces:**
- `BUILDINGS_RELPATH = "data/alkis-buildings.geojson"`.
- `fetch_project` additionally (online) calls `fetch_buildings` for `project.bbox()`, checks it with
  `parse_buildings`, and writes it atomically to `BUILDINGS_RELPATH` only after both succeeded; offline it
  requires the stored file (`FetchError::MissingOffline` is for tiles; use the new
  `ProjectError::BuildingsNotFetched` "no ALKIS buildings stored; run `freemapper fetch` online first").
  Progress message `"downloading ALKIS buildings"`, then `"{n} ALKIS buildings"`.
- `export_starter_project` reads `BUILDINGS_RELPATH` if present and puts `building_objects(…)` into the
  map's default part; progress-free.
- `test_support::serve_synthetic_tile` also serves a one-building ALKIS answer (the school square at the
  fixture centroid) under the ALKIS endpoint prefix.

- [ ] **Step 1: Tests** (`project.rs`):
  - `fetch_stores_buildings`: after fetch, `data/alkis-buildings.geojson` parses to 1 building.
  - `failed_alkis_fetch_keeps_stored_buildings`: second fetch with an HTML ALKIS answer → error; the
    stored file is unchanged.
  - `offline_fetch_uses_stored_buildings`: after an online fetch, an offline fetch succeeds without
    requests; with the file deleted it fails with `BuildingsNotFetched`.
  - `export_includes_buildings`: after fetch, the exported map contains one object with
    `<t k="freemapper:source">alkis</t>` inside `<parts`.
  - `export_without_buildings_has_empty_map`: a fresh project exports without building objects.
- [ ] **Step 2: Run** `cargo test -p freemapper project` → fail.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test --workspace` → pass; commit `feat: buildings in fetch and export`.

### Task 4: Docs, live run, acceptance

**Files:** `README.md`, `README.en.md`, spec §4.2 (`data/alkis-buildings.geojson`), glossary if a term
is missing; `crates/freemapper-cli/tests/cli.rs` (`live_run_school` also asserts building objects).

- [ ] **Step 1:** Extend `live_run_school`: the map contains at least 5 `freemapper:source">alkis`
  objects. Run `cargo test -p freemapper-cli live_run_school -- --ignored` → pass.
- [ ] **Step 2:** Docs DE/EN: what `fetch` downloads now, that buildings arrive as objects, the rule
  (underground parts and structures dropped), `data/` folder.
- [ ] **Step 3:** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` → pass.
- [ ] **Step 4:** Regenerate the sample with `run --force`; check visually that the buildings lie on the
  orthophoto. Commit `docs: M4a buildings`.
