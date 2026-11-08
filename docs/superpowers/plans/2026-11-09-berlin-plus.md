# Berlin Plus 1 (ALKIS structures, K5, trees) – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Berlin sprint and schoolyard maps get canopies as map objects, plus candidates for stairs, walls,
retaining walls, fences, hedges, building passages, embankments, monuments and trees, the Kleingarten
out-of-bounds areas, and the K5 1:5000 map as a template. All of it comes from Geoportal Berlin open data
(dl-de/zero-2-0).

**Plan 1 of 2.** Do this one first. Plan 2 (`2026-11-09-berlin-plus-2.md`) adds the Straßenbefahrung 2014
street survey, the small point layers and building currency.

**Architecture:**
- **Fetch:** `sources/berlin.rs` gets a table of extra Berlin layers. Each entry has a service, a feature type,
  a file name and the map kinds it serves. The existing `fetch_wfs` (GeoJSON, `links[rel=next]` paging,
  `get_with_retries`) loads each layer into `data/berlin/<file>.geojson` for areas that touch the Berlin region.
  De-duplication changes from `properties.uuid` to the top-level GeoServer `id`.
- **Rules:** the new embedded file `assets/rules/berlin-candidates.toml` uses the syntax of
  `osm-candidates.toml` and runs through the same `Rules` engine. A Berlin feature becomes a tagged feature:
  - tags: `berlin:layer=<file>` plus its string and number properties;
  - geometry: a point stays a point, each part of a MultiLineString becomes a line, and each polygon
    exterior becomes a closed line (holes are dropped; candidates only);
  - id: `berlin/<file>/<GeoServer id>`.
  `osm_objects` gets the source name and the currency as parameters, so provenance says `geoportal-berlin`
  and not `osm`.
- **Builders that need geometry work:**
  - `build/canopy.rs`: 522 map objects from ALKIS `Überdachung`/`Carport`.
  - `build/stairs.rs`: a stair polygon becomes a 532.x line along the walking direction, which comes
    from DGM1.
- **Trees:** the tree cadastre runs through the rules. OSM `natural=tree` candidates within 2 m of a
  cadastre tree are dropped.
- **K5 template:** `templates/k5.png` plus a world file, through the existing WMS template code (`sources/wms.rs`).
- **Kleingarten:** `out_of_bounds.rs` treats `Sport-, Freizeit- und Erholungsfläche / Kleingarten` as private.

**Tech Stack:** no new crates.

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` §4.1 (Berlin) and §4.5 (symbol mapping).
Verified sources (2026-11-08): `BERLIN-PLUS-REFERENCE.md` in the scratchpad (§n below), to be copied to
`docs/regions/berlin-plus-reference.md`. Live samples (untrusted, fixtures only):
`/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/berlin-plus/dl/`.

## Rulings (from §3, §5, §13, §14)

- **ALKIS `Überdachung` and `Carport` become 522 map objects** on both map kinds. They are cadastral survey
  data, as trustworthy as the 521 buildings. This amends spec §4.5 ("candidate 522") and needs an ADR.
  - Every other structure is a candidate: no height and no passability is recorded.
- **Building passages are 522 candidates,** and the 521 object stays whole. These are the Bauteile
  `Durchfahrt im Gebäude`, `Durchfahrt an überbautem Verkehrsweg` (with a no-break space) and `Arkade`.
  - Reason: the data does not say whether a passage is gated.
  - `Auskragender Geschossteil / zurückspringendes Geschoss` is dropped, because it mixes overhangs with
    set-back storeys.
- **Walls get one symbol each by the ALKIS name; height decides nothing:**
  - `Mauer`, `Mauerkante*`, `Mauermitte` → 515;
  - `Stützmauer*` → 513.2;
  - `Zaun` → 518;
  - `Heckenkante*` → 410.1.
  These are the same symbols as the OSM rules, so the mapper sees one convention.
- **Stair direction comes from DGM1.** The oriented minimum rectangle of a stair polygon gives two axes. The
  axis with the larger height difference between its ends is the walking direction.
  - If neither axis rises 0.3 m, the long axis is used and the object is tagged
    `freemapper:direction=guessed`.
  - The width across the walking direction picks the footprint symbol.
- **Embankments:** `relief` `Oberkante` becomes a 104 candidate on sprint maps. `Unterkante` and
  `Sonstige Begrenzungskante` are skipped.
- **Kerb lines are not taken from ALKIS** (`Fahrbahnbegrenzungslinie`, 526 342 citywide). Plan 2 brings the
  surveyed curbs, and two sources would duplicate them.
- **Trees:** the cadastre only (`strassenbaeume`, `anlagenbaeume`).
  - 417 when `kronedurch` ≥ 10 m, otherwise 418.
  - Grün Berlin's tree layer is left out because of its licence conflict (§5).
- **Kleingarten is out of bounds (520 candidate) in every state** with that `bez`, not only in Berlin.
- **The K5 template is a WMS image** (PNG, 0.25 m/px on sprint, 0.1 m/px on the schoolyard), never the
  regional TIFF ZIPs.

## Global Constraints

- **Region gate:** extra layers are fetched only when the widened bbox touches the Berlin region
  (`sources/region.rs`, `assets/regions/berlin.geojson`, the same test as the Berlin ALKIS). Outside Berlin
  every layer returns 0 features.
- **Layers (§3, §5):**
  | file | service | type | kinds |
  |---|---|---|---|
  | `bauwerke-flaechen` | `https://gdi.berlin.de/services/wfs/alkis` | `alkis:bauwerkeflaechen` | both |
  | `bauwerke-linien` | same | `alkis:bauwerkelinien` | both |
  | `relief` | same | `alkis:relief` | sprint |
  | `vegetation-linien` | same | `alkis:gewaesservegetationlinien` | both |
  | `strassenbaeume` | `https://gdi.berlin.de/services/wfs/baumbestand` | `baumbestand:strassenbaeume` | both |
  | `anlagenbaeume` | same | `baumbestand:anlagenbaeume` | both |

  Building parts come from the stored `alkis_gebaeude:gebaeude` response. `normalise_building` keeps
  `bezbat` as `bauart`.
- **WFS:** `outputFormat=application/json`, `count=5000` (`PAGE_COUNT`), `bbox=…,urn:ogc:def:crs:EPSG::25833`,
  at most 3 attempts per page, de-duplication by the top-level `id`.
  - `bauwerkelinien` with `propertyName` filtering is not used. The whole feature is needed: 1.5 MB per km²
    in the centre.
- **Thresholds:**
  - `CANOPY_MIN_SIDE_MM = 0.5` and `SIMPLIFY_MM = 0.1`, as in `buildings.rs`;
  - `STAIR_MIN_RISE_M = 0.3`;
  - footprints `< 1.8 m → 532.6`, `< 2.5 m → 532.7`, `< 3.5 m → 532.8`, otherwise `532.9`;
  - `LARGE_TREE_CROWN_M = 10.0`;
  - `TREE_DEDUP_M = 2.0`;
  - every ALKIS `Denkmal, Denkstein, Standbild` area becomes a 526 point at its centroid. Areas wider than
    3 m are also tagged `freemapper:size=large`, so the mapper can draw the outline instead.
- **Candidate symbols (§3.6):** 522, 532.6–532.9, 515, 513.2, 518, 410.1, 104 (sprint), 526, 501.5 for ALKIS
  `Brücke` (sprint), 417, 418, 520 (Kleingarten).
- **Provenance tags:** `freemapper:source=geoportal-berlin/<file>`, `freemapper:source_id=<uuid or gisid>`,
  `freemapper:currency` = the fetch date (no feature dates in these layers), plus `bezbwf` / `bezart` / `bezbws`
  copied as `freemapper:alkis`.
- **K5:**
  - WMS `https://gdi.berlin.de/services/wms/k5_farbe`, layer `k5_farbe`, `CRS=EPSG:25833`, `FORMAT=image/png`;
  - `max_px` 4000 per request, tiled like the orthophoto WMS;
  - output `templates/k5.png` + `.pgw`;
  - currency: the dataset date 2026-01-02 (catalogue `karte-von-berlin-1-5000-k5-farbausgabe-9f3ecf8d`).
- **Attribution (§12):**
  - `© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert` (existing line);
  - `Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0` when the K5 template was written;
  - `Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0` when a tree candidate was written.
- **Download estimate** (`layers.rs`): +2 MB per km² for the vector layers, +1.5 MB per km² for K5.

## Review Focus

1. **The 522 canopy map objects:**
   - they sit below buildings in paint order (colour priority);
   - canopies inside a building footprint are not doubled;
   - the ADR states the spec change.
2. **Stair direction:** the DGM is sampled in map CRS, the flat-stair fallback is tagged, and footprint
   widths are in metres, not paper mm.
3. **Region routing:**
   - a Brandenburg-only area makes no extra request;
   - a border area keeps the Berlin layers;
   - an empty layer is not an error.
4. **The rule engine reuse:**
   - Berlin tags cannot match OSM rules and OSM tags cannot match Berlin rules (separate rule files);
   - polygon holes are dropped knowingly;
   - provenance says `geoportal-berlin`.
5. **The Kleingarten rule** changes out-of-bounds output in other states. Check the fixtures of all states,
   and the schoolyard path.

---

### Task 1: Extra layer fetch (Berlin)
Tests:
- `extra_layers_only_inside_berlin`
- `extra_layer_url_matches_the_wfs`
- `dedup_by_feature_id_without_uuid`
- `empty_layer_is_stored_not_an_error`
- `building_parts_keep_bauart`

### Task 2: Berlin rules engine and candidates
Tests:
- `berlin_feature_becomes_tagged_feature` (point, multiline, multipolygon with a hole)
- `walls_fences_hedges_map_to_issprom`
- `passage_bauteile_are_canopy_candidates`
- `embankment_top_only_on_sprint`
- `berlin_provenance_is_geoportal`
- `osm_rules_ignore_berlin_tags`

### Task 3: Canopies and stairs
Tests:
- `ueberdachung_and_carport_are_canopy_objects`
- `tiny_canopy_is_dropped`
- `stair_direction_follows_the_dtm`
- `flat_stair_uses_long_axis_and_is_tagged`
- `stair_width_picks_footprint_symbol`

### Task 4: Trees, Kleingarten, K5 template
Tests:
- `crown_decides_417_or_418`
- `osm_tree_near_cadastre_tree_is_dropped`
- `kleingarten_is_out_of_bounds_candidate`
- `k5_template_has_world_file`
- `berlin_plus_attribution_lines`
- `berlin_plus_estimate`

### Task 5: Docs, ADRs, changelog, example, live run, review
- Copy `BERLIN-PLUS-REFERENCE.md` to `docs/regions/berlin-plus-reference.md`.
- Spec §4.1 (Berlin) and §4.5 (522 canopy, stairs, walls, trees, Kleingarten).
- Manual chapters 12 and 14 in German and English (unslop + humanizer). This includes the new candidate
  kinds and the K5 template, and the Berlin row note on paved areas stays.
- ADRs:
  - ALKIS canopies as map objects;
  - Berlin extra layers through the candidate rule engine;
  - stair direction from DGM1;
  - Kleingarten out of bounds.
- A changelog entry (DE + EN).
- Live runs:
  - the Helene-Haeusler-Schule schoolyard (1:1000);
  - the Viktoriapark sprint map.
- Regenerate `examples/berlin-viktoriapark` (OOM, OCAD, PDF, preview) and update its README.
