# Architecture Decision Records

An ADR records one decision that shapes freemapper: what was decided, why, and what it costs if it turns out
wrong. The ADRs here were written on 2026-11-08, after the fact, from the design spec
(`docs/superpowers/specs/2026-11-07-freemapper-design.md`), the plans in `docs/superpowers/plans/`, the region
references in `docs/regions/`, the commit messages and the rulings recorded while the plans were carried out.
Each ADR ends with a `Source:` line that names where the decision is written down.

Two sources are not in the repository: the plan-mode notes from the first brainstorming on 2026-11-06 (German),
and the rulings ledger kept during plan execution (one line per ruling: decision, reason, cost if wrong). An ADR
that cites them names the plan and the task.

## Numbering

ADRs are numbered from 0001 in the order the decisions were made. A number is never reused. The file name is
`NNNN-short-kebab-title.md`. A decision that replaces an older one gets a new ADR, and the old one changes its
status to `Superseded by NNNN`. Its text stays as it was.

## Template

```markdown
# NNNN. Title

Date: YYYY-MM-DD
Status: Accepted

## Context

What forced a decision: the facts, limits and data that were known.

## Decision

What was decided, in the active voice.

## Consequences

What follows from it, good and bad, and what it costs if the decision is wrong.

Source: spec §…, plan …, commit …
```

Keep an ADR between 10 and 30 lines. Write it in English, like all technical docs, and use the terms from
`docs/glossary.md`.

## Index

| Number | Title | Status | Date |
|---|---|---|---|
| [0001](0001-rust-with-a-pure-rust-core.md) | Rust with a pure-Rust core | Accepted | 2026-11-06 |
| [0002](0002-oom-as-primary-output-ocad-and-pdf-as-extras.md) | OOM as the primary output, OCAD and PDF as extra outputs | Accepted | 2026-11-06 |
| [0003](0003-sure-objects-as-map-objects-rest-as-templates.md) | Sure objects as map objects, the rest as templates | Accepted | 2026-11-06 |
| [0004](0004-provenance-tags-on-generated-objects.md) | Provenance tags on generated objects | Accepted | 2026-11-06 |
| [0005](0005-mcp-over-stdio-as-the-lay-interface.md) | MCP over stdio as the interface for non-technical users | Accepted | 2026-11-06 |
| [0006](0006-gpl-3.0-or-later.md) | GPL-3.0-or-later | Accepted | 2026-11-06 |
| [0007](0007-glossary-first-english-code-bilingual-docs.md) | Glossary first, English code, bilingual user docs | Accepted | 2026-11-06 |
| [0008](0008-osm-from-geofabrik-extracts.md) | OSM from Geofabrik extracts, not Overpass | Accepted | 2026-11-06 |
| [0009](0009-area-selection-in-a-local-browser-page.md) | Area selection in a local browser page | Accepted | 2026-11-06 |
| [0010](0010-templates-as-image-world-file-and-template-map.md) | Raster templates as image plus world file, vector templates as a template map | Accepted | 2026-11-07 |
| [0011](0011-re-runnable-projects-and-a-currency-keyed-cache.md) | Re-runnable projects and a cache keyed by data currency | Accepted | 2026-11-07 |
| [0012](0012-dependencies-at-least-two-weeks-old.md) | Dependencies at least two weeks old | Accepted | 2026-11-07 |
| [0013](0013-regions-as-data-and-a-raster-region-by-the-area-centre.md) | Regions as data, and a raster region by the area centre | Accepted | 2026-11-07 |
| [0014](0014-two-utm-zones-map-crs-from-the-raster-region.md) | Two UTM zones, the map CRS from the raster region | Accepted | 2026-11-07 |
| [0015](0015-alkis-merged-reprojected-and-de-duplicated-by-oid.md) | ALKIS from each touched state, merged, reprojected and de-duplicated by oid | Accepted | 2026-11-07 |
| [0016](0016-per-area-raster-providers-and-tile-providers.md) | Per-area raster providers and tile providers | Accepted | 2026-11-07 |
| [0017](0017-http-byte-range-reads-of-large-zips.md) | HTTP byte-range reads of large zips, with CRC checks and per-release caches | Accepted | 2026-11-07 |
| [0018](0018-rust-toolchain-pinned.md) | The Rust toolchain is pinned in rust-toolchain.toml | Accepted | 2026-11-08 |
| [0019](0019-hesse-downloadcenter-via-the-metadata-xlsx.md) | Hesse DGM1 and DOM1 from the Downloadcenter via the metadata xlsx | Accepted | 2026-11-08 |
| [0020](0020-one-example-project-per-state.md) | One example project per state | Accepted | 2026-11-08 |
| [0021](0021-hamburg-alkis-wfs-fallback.md) | Hamburg ALKIS falls back to the classic WFS | Accepted | 2026-11-08 |
| [0022](0022-unresolvable-places-are-skipped.md) | Unresolvable places are skipped and reported | Accepted | 2026-11-08 |
| [0023](0023-schleswig-holstein-pinned-d-trust-anchors.md) | Schleswig-Holstein: pinned D-TRUST anchors for one host | Accepted | 2026-11-08 |
| [0024](0024-schleswig-holstein-bdom-reduced-to-1-m.md) | Schleswig-Holstein: nDSM from the bDOM reduced to 1 m | Accepted | 2026-11-08 |
| [0025](0025-valid-region-boundary-geometries.md) | Valid region boundary geometries, checked by a test | Accepted | 2026-11-08 |
| [0026](0026-lenient-lzw-fallback.md) | Lenient LZW fallback for GeoTIFF chunks | Accepted | 2026-11-08 |
| [0027](0027-mv-orthophoto-from-the-wcs-and-masking-to-the-tile-set.md) | Mecklenburg-Vorpommern: orthophoto from the WCS, models masked to the tile set | Accepted | 2026-11-08 |
| [0028](0028-nrw-osm-extracts-per-regierungsbezirk.md) | North Rhine-Westphalia: OSM extracts per Regierungsbezirk | Accepted | 2026-11-08 |
| [0029](0029-lower-saxony-rasters-via-wms-and-wcs.md) | Lower Saxony: rasters via WMS and WCS instead of COG windows | Accepted (planned, not yet implemented) | 2026-11-08 |
