# M5b: Out-of-Bounds Candidates from ALKIS Land Use – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper run` adds private land (residential, commercial, mixed use, railway) around the
school as 520 "area that shall not be entered" candidates to `templates/candidates.omap`, so the mapper
can check and copy them. Private land stays a template (user: "Privatgelände nur als Vorlage").

**Architecture:** The ALKIS source gains a generic paged collection download; `nutzung_flurstueck`
(land use per parcel part, GeoJSON EPSG:25833) is stored as `data/alkis-land-use.geojson` with a fetch
record like the buildings. A rule module selects private land uses, leaves out the school's own land
(schoolyard: parts at least half inside the map area), clips to map area + buffer and drops slivers.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.5 (out-of-bounds 520: residential
parcels minus buildings/traffic areas (sprint); for the schoolyard, the non-school parcels inside map
area + buffer), user rule "Privatgelände nur als Vorlage".

## Global Constraints

- Endpoint `…/alkis-vereinfacht/v1/collections/nutzung_flurstueck/items`, same query parameters and paging
  as `gebaeude_bauwerk`. Properties (live 2026-11-07): `oid`, `aktualit`, `nutzart`, optional `bez`,
  `flstkennz`. Values around the school: `Wohnbaufläche`, `Industrie- und Gewerbefläche`,
  `Straßenverkehr`, `Wald`, `Bahnverkehr`, `Sport-, Freizeit- und Erholungsfläche`,
  `Fläche besonderer funktionaler Prägung` (school: `bez = Bildung und Wissenschaft`).
- Private land uses → 520: `Wohnbaufläche`, `Industrie- und Gewerbefläche`, `Fläche gemischter Nutzung`,
  `Bahnverkehr`. Everything else (traffic, forest, sport, water, agriculture, special function) is not a
  candidate. Buildings on private land are drawn on top by the map's 521 objects.
- Schoolyard: parts with ≥ 50 % of their area inside the map area are the school's own land and left out.
- Clip to the map area's bounding box plus buffer; drop parts below (1 mm)² on paper.
- Tags: `freemapper:source = alkis-nutzung`, `freemapper:source_id = <oid>`,
  `freemapper:nutzart = <nutzart[ / bez]>`, `freemapper:currency = <aktualit date>`.

## Review Focus

1. **A parcel reaching far beyond the area** (e.g. the 16 ha sports park parcel) – clipped, not drawn whole.
2. **The school's own parcel** – never a 520 candidate on the schoolyard map.
3. **Multipolygon parts and holes** – each part its own object, holes kept.
4. **ALKIS answer invalid or offline** – stored land use untouched / used, like the buildings.
5. **Projects fetched before M5b** – export works without the land use file.

---

### Task 1: Land use source
`sources/alkis.rs`: `fetch_collection(fetcher, collection, bbox) -> Result<String, FetchError>` (buildings use
it), `LAND_USE_COLLECTION = "nutzung_flurstueck"`, `LandUse { oid, nutzart, bez: Option<String>, updated,
footprint: MultiPolygon }`, `parse_land_use(&str) -> Result<Vec<LandUse>, AlkisError>` (unusable features
skipped like buildings). Tests: `land_use_url_uses_its_collection`, `parse_land_use_features`.

### Task 2: Out-of-bounds rules
`build/out_of_bounds.rs`: `is_private(&LandUse) -> bool`, `out_of_bounds_objects(land_use, map_area (EPSG:25833),
clip: &BoundingBox, kind, georef, symbol_set) -> Result<Vec<MapObject>, GeorefError>`. Tests:
`private_land_uses`, `school_parcel_is_left_out_on_schoolyard`, `sprint_keeps_parcels_inside_the_area`,
`large_parcel_is_clipped`, `slivers_are_dropped`, `holes_are_kept`, `tags_carry_provenance`.

### Task 3: Project integration
`ProjectFile.land_use: Option<AlkisRecord>` (the buildings record type, renamed from `BuildingsRecord`);
`fetch_project` stores `data/alkis-land-use.geojson` (online only after it parses; offline needs the file
and coverage); export adds the objects to the candidates part. Tests: `fetch_stores_land_use`,
`export_adds_out_of_bounds_candidates`, `export_without_land_use_still_works`,
`failed_land_use_fetch_keeps_stored_file`.

### Task 4: Docs, live run, acceptance
README DE/EN, spec §4.2 (`data/alkis-land-use.geojson`), glossary (`Sperrgebiet` / out-of-bounds area
if missing). Live test (school parcel left out, Am Sportpark 2 commercial parcel a candidate); sample
regenerated; overlay checked.
