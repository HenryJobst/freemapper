# Park vegetation implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sprint and schoolyard maps get vegetation candidates (401, 405 to 410) from classified point clouds where a state publishes them; the forest map kind reuses the same module.

**Architecture:** `derive/vegetation.rs` (done, ADR 0105) turns returns into class areas. What is missing is the data: a point-cloud capability of the region adapters that fetches only the tiles (or byte ranges) under the map area, a derive step that writes `data/vegetation.geojson`, and an export step that puts the areas into the candidates map.

**Tech Stack:** Rust, `las` with LAZ, the adapter API (`freemapper-adapter-api`), `freemapper-sources`.

**Spec:** ADR 0105, ADR 0082 (region adapters), memory notes "forest maps goal" and "Mapant" (point-cloud availability per state).

## Global Constraints

- Stage C of the region adapters is on main (ed319e5); stage D (manifests) is in progress on the branch `region-adapters`. Agreed with that work on 2026-11-12: the point-cloud capability is ours and stays additive (`pointcloud.rs`, a default method on `RegionAdapter`, one line in `lib.rs`); `crates/freemapper-manifest` is not touched until D6; until `Registry::backend(name)` exists (D5) the backend hangs directly on the adapter; `[pointcloud] backend = "<name>"` follows after D5. The hosts are those the adapters already declare (`data.geobasis-bb.de`, `www.opengeodata.nrw.de`, `geobasis-rlp.de`).
- Only per-area downloads; never a statewide bulk file (Berlin's 8 regional ZIPs of about 200 GB need range reads or stay out).
- Verify checksums where the service gives them (Metalink SHA-256 in Rhineland-Palatinate).
- The disk check (ADR 0045) counts the point-cloud tiles in the estimate.
- Everything is a candidate; no vegetation enters the map itself.

## Review Focus

1. A park on a tile border: both tiles are read, no seam in the classes.
2. A scan from winter (leaves off): runnable forest must not turn open; check `OPEN_SHARE` on real data.
3. Water and buildings: no vegetation candidates on them (class filter plus the ALKIS footprints in the export).
4. A state without point clouds: no step runs, no warning beyond one progress line.
5. Very dense tiles (Berlin 10 points/m²): memory stays bounded (read in chunks, count per cell, never keep all points).

---

### Task 1: Point-cloud capability in the adapter API

- [x] `PointCloudBackend` in `freemapper-adapter-api/src/pointcloud.rs` (`info`, `fetch`, `cached`, `estimate`), `PointCloudRecord` for the project file, `RegionAdapter::point_cloud` (default `None`), `Part::PointCloud` with its texts.
- [x] Brandenburg: `crates/adapters/de/src/lgb_als.rs` (zip per km tile streamed into the cache, only the LAZ file kept and checked to its last point, currency from `bb_laserscandaten_aktualitaet.csv`, older publications removed, a failing tile falls back to the cache or is left out with a warning).
- [x] North Rhine-Westphalia (index.json, LAZ, size checked against the index; `PointCloudInfo::epsg`, the engine uses a backend only for maps in its CRS).
- [ ] Rhineland-Palatinate (Metalink with SHA-256), Saxony, Thuringia, Bavaria, Saarland; Berlin only with range reads.
- [ ] `[pointcloud]` in the manifests once stage D has named backends (D5).
- [x] The engine counts the estimate items in `estimate_download` and the disk check (Task 2, beside the fetch step).

### Task 2: Derive step

- [x] Change `read_points` to count per cell while reading (no `Vec` of all points), then `classify` and `vegetation_areas` with the project's `scale_denominator`.
- [x] Write `data/vegetation.geojson` (class code and polygon per feature); no file where no point cloud is recorded.
- [x] Test with a synthetic LAZ under a fixture project; golden re-blessed (G2/G3 for Brandenburg: the estimate item and the unanswered currency list; G4 unchanged, the store holds no point cloud).
- [x] The fetch step: after the K5 map, records `point_clouds` in the project file, keeps the cached records on a full disk or a failing service.

### Task 3: Export step

- [x] `stored_vegetation_objects` in `project/export.rs`: area objects with the class symbol, tags `freemapper:source` (the point-cloud product) and `freemapper:currency`; cut at building footprints and drawn water.
- [x] Where an ALKIS park (401) or wood (405) is a map object, the candidate shows only where the class differs.
- [x] Manual DE/EN, changelog, glossary rows (Grünabstufung / green shade, Unterwuchs / undergrowth).

- [x] Opt-in (user 2026-11-12, decided by me after "Auf was wartest du?"): content group `vegetation`, off in every profile; the choice gates fetch, estimate, derive and export.

### Task 4: Check on real parks

- [x] Sanssouci (2026-11-12): building mask, single-echo rule for roofs, cut at paved areas; the other thresholds stay. Recorded in ADR 0105.
- [x] Volksgarten Cologne (NRW): plausible with the Sanssouci values.
- [ ] Viktoriapark once Berlin has a point-cloud backend (range reads).
