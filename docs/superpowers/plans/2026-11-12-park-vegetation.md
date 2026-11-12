# Park vegetation implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sprint and schoolyard maps get vegetation candidates (401, 405 to 410) from classified point clouds where a state publishes them; the forest map kind reuses the same module.

**Architecture:** `derive/vegetation.rs` (done, ADR 0105) turns returns into class areas. What is missing is the data: a point-cloud capability of the region adapters that fetches only the tiles (or byte ranges) under the map area, a derive step that writes `data/vegetation.geojson`, and an export step that puts the areas into the candidates map.

**Tech Stack:** Rust, `las` with LAZ, the adapter API (`freemapper-adapter-api`), `freemapper-sources`.

**Spec:** ADR 0105, ADR 0082 (region adapters), memory notes "forest maps goal" and "Mapant" (point-cloud availability per state).

## Global Constraints

- Stage C of the region adapters is on main (ed319e5). Stage D (manifests) is in progress on the branch `region-adapters`: the point-cloud capability touches its schema (D1) and named backends (D5), so Task 1 starts only after agreement with that session on who builds it and when.
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

- [ ] `PointCloudBackend` beside the raster backends: tiles under a box, their URLs, sizes and checksums; a `pointcloud` kind for manifests (stage D).
- [ ] Brandenburg first (per-tile ZIP, LAS 1.2), then North Rhine-Westphalia (index.json, LAZ), Rhineland-Palatinate (Metalink with SHA-256), Saxony, Thuringia, Bavaria, Saarland; Berlin only with range reads.
- [ ] Estimate and disk check include the tiles; cache like the DTM tiles.

### Task 2: Derive step

- [ ] Change `read_points` to count per cell while reading (no `Vec` of all points), then `classify` and `vegetation_areas` with the project's `scale_denominator`.
- [ ] Write `data/vegetation.geojson` (class code and polygon per feature); no file where no point cloud is recorded.
- [ ] Test with a synthetic LAZ under a fixture project; golden G4 re-blessed.

### Task 3: Export step

- [ ] `stored_vegetation_objects` in `project/export.rs`: area objects with the class symbol, tags `freemapper:source` (the point-cloud product) and `freemapper:currency`; cut at building footprints and drawn water.
- [ ] Where an ALKIS park (401) or wood (405) is a map object, the candidate shows only where the class differs.
- [ ] Manual DE/EN, changelog, glossary rows (Grünabstufung / green shade, Unterwuchs / undergrowth).

### Task 4: Check on real parks

- [ ] One park per state with a source (Sanssouci, Viktoriapark, a park in NRW); compare with the orthophoto and the nDSM; tune `OPEN_SHARE`, the index thresholds and the undergrowth shares; record the values in ADR 0105.
