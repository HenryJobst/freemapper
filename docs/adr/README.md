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
| [0002](0002-oom-as-primary-output-ocad-and-pdf-as-extras.md) | OOM as the primary output, OCAD and PDF as extra outputs | Accepted, amended by ADR 0080 | 2026-11-06 |
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
| [0029](0029-lower-saxony-rasters-via-wms-and-wcs.md) | Lower Saxony: rasters via WMS and WCS instead of COG windows | Accepted | 2026-11-08 |
| [0030](0030-nrw-tile-names-and-dates-from-the-tile-list.md) | North Rhine-Westphalia: tile names and dates only from the tile list | Accepted | 2026-11-08 |
| [0031](0031-nrw-orthophoto-from-the-wms.md) | North Rhine-Westphalia: orthophoto from the WMS, because the tiles are JPEG 2000 only | Accepted | 2026-11-08 |
| [0032](0032-lower-saxony-ndsm-from-dom1-with-an-age-warning.md) | Lower Saxony: nDSM from DOM1 minus DGM1, with an age warning | Accepted | 2026-11-08 |
| [0033](0033-lower-saxony-dates-from-stac.md) | Lower Saxony: dates from STAC, the newest item at the centre | Accepted | 2026-11-08 |
| [0034](0034-warning-for-areas-beyond-every-supported-state.md) | A warning for areas that reach beyond every supported state | Accepted | 2026-11-08 |
| [0035](0035-bremen-outside-until-its-own-region.md) | Bremen stays outside until it gets its own region | Accepted | 2026-11-08 |
| [0036](0036-water-from-alkis-as-fill-and-bank-lines.md) | Water from ALKIS as fill and bank lines, water from OSM as candidates | Accepted, amended ADR 0070 | 2026-11-08 |
| [0037](0037-ai-features-are-optional.md) | AI features inside the program are optional | Accepted | 2026-11-08 |
| [0038](0038-rhineland-palatinate-orthophoto-from-the-wms-dated-by-its-creation-date.md) | Rhineland-Palatinate: orthophoto from the WMS, dated by its creation date | Accepted | 2026-11-08 |
| [0039](0039-rhineland-palatinate-tile-names-from-the-laserbefliegung-index.md) | Rhineland-Palatinate: tile names from the Laserbefliegung index, the Metalink for unnamed tiles | Accepted | 2026-11-08 |
| [0040](0040-rhineland-palatinate-missing-tiles-checked-again-after-30-days.md) | Rhineland-Palatinate: missing tiles are reported and checked again after 30 days | Accepted | 2026-11-08 |
| [0041](0041-rhineland-palatinate-alkis-from-ave-nutzung-on-the-origin-host.md) | Rhineland-Palatinate: ALKIS from `ave:Nutzung` on the origin host, dates from `gml:timePosition` | Accepted | 2026-11-08 |
| [0042](0042-rhineland-palatinate-provider-name-and-attribution.md) | Rhineland-Palatinate: provider `LvermGeoRp` and the attribution line | Accepted | 2026-11-08 |
| [0043](0043-magnetic-north-lines-are-map-content.md) | Magnetic north lines are map content | Accepted | 2026-11-08 |
| [0044](0044-iof-colours-rev-4.md) | Colours after IOF Printing and Colour Definitions Rev. 4, not OOM's bundled values | Accepted | 2026-11-08 |
| [0045](0045-disk-space-check-before-fetching-and-disk-full-handling.md) | Disk space: checked before fetching, optional parts left out, a full disk reported plainly | Accepted | 2026-11-08 |
| [0046](0046-alkis-canopies-are-map-objects.md) | ALKIS canopies in Berlin are map objects, not candidates | Accepted | 2026-11-09 |
| [0047](0047-berlin-extra-layers-through-the-candidate-rule-engine.md) | Berlin extra layers become candidates through the candidate rule engine | Accepted | 2026-11-09 |
| [0048](0048-stair-direction-from-dgm1-and-symbol-by-paper-width.md) | Stair direction from the DGM1, stair symbol by paper width | Accepted | 2026-11-09 |
| [0049](0049-allotment-gardens-out-of-bounds-in-every-state.md) | Allotment gardens (Kleingarten) are out of bounds in every state | Accepted | 2026-11-09 |
| [0050](0050-k5-template-at-0-25-m.md) | The Berlin K5 map is a WMS template at 0.25 m per pixel | Accepted | 2026-11-09 |
| [0051](0051-tree-cadastre-417-418-and-osm-tree-dedup.md) | Berlin trees from the tree cadastre: 417 or 418 by crown, OSM trees nearby dropped | Accepted | 2026-11-09 |
| [0052](0052-berlin-street-survey-only-for-candidates-and-green-cut-outs.md) | The Berlin street survey gives candidates and green cut-outs, never map objects | Accepted | 2026-11-09 |
| [0053](0053-survey-pieces-down-to-2-m-on-the-ground.md) | Survey candidates and cut pieces are kept down to 2 m on the ground | Accepted | 2026-11-09 |
| [0054](0054-street-furniture-on-the-schoolyard-only-curbs-on-sprint-maps.md) | Street furniture on schoolyard maps only, curbs on sprint maps only | Accepted | 2026-11-09 |
| [0055](0055-berlin-duplicates-registers-alkis-survey-osm.md) | Berlin duplicates: registers win, then ALKIS, then the survey, then OSM | Accepted | 2026-11-09 |
| [0056](0056-short-line-segments-joined-before-the-1-mm-limit.md) | Short line segments are joined before the 1 mm limit | Accepted | 2026-11-09 |
| [0057](0057-berlin-building-dates-from-the-2022-building-heights.md) | Berlin building dates come from the 2022 building heights layer | Accepted | 2026-11-09 |
| [0058](0058-berlin-layer-tag-and-the-freemapper-also-form.md) | Every Berlin candidate carries `berlin:layer`; `freemapper:also` names `<source>/<id>` | Accepted | 2026-11-09 |
| [0059](0059-project-files-never-name-paths.md) | Project files never name paths | Accepted | 2026-11-09 |
| [0060](0060-writes-never-follow-links.md) | Writes never follow links | Accepted | 2026-11-09 |
| [0061](0061-alkis-canopies-and-passages-in-every-state.md) | ALKIS canopies and passages in every state | Accepted | 2026-11-09 |
| [0062](0062-full-alkis-from-the-nas-wfs.md) | Full ALKIS from the NAS WFS, Brandenburg first | Accepted | 2026-11-09 |
| [0063](0063-adv-code-rules-and-a-shared-duplicate-rank.md) | AdV code rules for the states and one duplicate rank for every state | Accepted | 2026-11-09 |
| [0064](0064-nas-stairs-from-the-bottom-edge.md) | NAS stairs run up from their bottom edge | Accepted | 2026-11-09 |
| [0065](0065-city-tree-cadastres-through-one-reader-potsdam-first.md) | City tree cadastres through one reader, Potsdam first | Accepted | 2026-11-09 |
| [0066](0066-point-symbol-generalization.md) | Point symbols keep the norm's gap; dense tree groups become areas | Accepted, amended 2026-11-10, ADR 0069 | 2026-11-09 |
| [0067](0067-shared-crates-from-freemapper-core.md) | freemapper uses shared crates from freemapper-core | Accepted | 2026-11-09 |
| [0068](0068-overlapping-walls-fences-and-hedges.md) | Overlapping walls, fences and hedges keep the stronger line | Accepted | 2026-11-10 |
| [0069](0069-parks-as-open-land-and-the-walkable-area.md) | Parks are open land, and the walkable area lies under the sprint map | Accepted | 2026-11-09 |
| [0070](0070-osm-multipolygon-relations.md) | OSM multipolygon relations are read | Accepted | 2026-11-10 |
| [0071](0071-kerb-lines-without-tree-pit-notches.md) | Kerb lines of the street survey lose their tree pit notches | Accepted | 2026-11-10 |
| [0072](0072-building-passages-from-osm-are-522-candidates.md) | Building passages from OSM are 522 candidates | Accepted | 2026-11-10 |
| [0073](0073-access-restrictions-and-firm-gravel-from-osm.md) | Access restrictions and firm gravel from OSM | Accepted, amended ADR 0079 | 2026-11-10 |
| [0074](0074-control-sites-from-street-furniture.md) | Control sites from street furniture | Accepted | 2026-11-10 |
| [0075](0075-obvious-memorials-without-stolpersteine.md) | Obvious memorials from OSM, without Stolpersteine and plaques | Accepted, amended ADR 0076 | 2026-11-10 |
| [0076](0076-reachable-passages-and-points-in-buildings.md) | Passages must be reachable, and points inside buildings go | Accepted | 2026-11-10 |
| [0077](0077-berlin-survey-and-registers-on-sprint-maps.md) | Berlin survey and registers on sprint maps | Accepted | 2026-11-10 |
| [0078](0078-duplicate-areas-and-more-duplicate-kinds-by-rank.md) | Duplicate areas and more duplicate kinds, by rank | Accepted | 2026-11-10 |
| [0079](0079-firm-gravel-paths-are-map-objects-again.md) | Firm gravel paths are map objects again | Accepted, amends ADR 0073 | 2026-11-10 |
| [0080](0080-map-pdf-without-candidates-by-default.md) | The PDF is the map without candidates, the scale bar is optional | Accepted, amends ADR 0002 | 2026-11-10 |
| [0081](0081-pdf-source-block-shortened-when-needed.md) | The source block of the PDF is shortened when needed and can be switched to the credit line | Accepted, extends ADR 0080 | 2026-11-10 |
| [0082](0082-region-adapters-trait-and-manifests.md) | Region adapters: a trait, TOML manifests and adapter crates | Accepted | 2026-11-10 |
| [0083](0083-power-lines-pipelines-and-towers-from-osm.md) | Power lines, pipelines and towers from OSM | Accepted | 2026-11-10 |
| [0084](0084-map-area-checked-on-every-path.md) | The map area is checked on every path | Accepted | 2026-11-10 |
| [0085](0085-allowed-map-scales-per-kind.md) | Allowed map scales per kind | Accepted | 2026-11-10 |
| [0086](0086-title-always-in-the-project-file.md) | The project file always holds a title | Accepted | 2026-11-10 |
| [0087](0087-berlin-fountains-from-register-survey-and-osm-once.md) | Berlin fountains from register, survey and OSM once | Accepted | 2026-11-10 |
| [0088](0088-osm-base-tiles-through-the-local-server.md) | OSM base tiles through the local server | Accepted | 2026-11-10 |
| [0089](0089-output-tested-against-the-suite-contract.md) | The output is tested against the suite contract | Accepted | 2026-11-10 |
| [0090](0090-create-the-map-from-the-browser-page.md) | Create the map from the browser page | Accepted | 2026-11-11 |
| [0091](0091-one-document-build-per-command.md) | One document build per command | Accepted | 2026-11-11 |
| [0092](0092-program-language.md) | Program language | Accepted | 2026-11-11 |
| [0093](0093-railways-from-osm.md) | Railways from OSM | Accepted | 2026-11-11 |
| [0094](0094-self-update-from-signed-releases.md) | Self-update from signed releases | Accepted | 2026-11-11 |
| [0095](0095-kinds-and-reserved-words-from-the-contract-crate.md) | Map kinds and reserved words come from the contract crate | Accepted | 2026-11-11 |
| [0096](0096-the-area-page-on-the-suite-frame.md) | The area page on the suite frame | Accepted | 2026-11-11 |
| [0097](0097-ticks-turn-and-lines-stop-at-ways.md) | Ticks turn the right way, lines stop at ways, points keep their gap to fences | Accepted | 2026-11-11 |
| [0098](0098-paved-osm-ways-as-paved-lines.md) | Paved OSM ways are paved lines | Accepted | 2026-11-11 |
| [0099](0099-paved-areas-get-their-border.md) | Paved areas get their border | Accepted | 2026-11-11 |
| [0100](0100-content-profiles.md) | Content profiles decide what goes straight onto the map | Accepted | 2026-11-12 |
| [0101](0101-content-profiles-on-the-area-page.md) | Content profiles on the area page | Accepted | 2026-11-12 |
| [0102](0102-region-adapters-crates-trait-and-registry-as-built.md) | Region adapters, stages B and C: crates, trait and registry as built | Accepted | 2026-11-12 |
| [0103](0103-the-area-page-takes-the-regions-from-the-registry.md) | The area page and the texts take the regions from the registry | Accepted | 2026-11-12 |
| [0104](0104-terrain-candidates-from-the-dgm1.md) | Terrain candidates from the DGM1 | Accepted | 2026-11-12 |
| [0105](0105-vegetation-from-point-clouds.md) | Vegetation from point clouds | Accepted (analysis, the sources of Brandenburg, Berlin, North Rhine-Westphalia, Rhineland-Palatinate and Saarland, fetch, derive and export) | 2026-11-12 |
| [0106](0106-requests-go-to-public-addresses-and-declared-hosts.md) | Requests go to public addresses, and with a policy to declared hosts only | Accepted | 2026-11-12 |
| [0107](0107-adapter-manifests-format-1-as-built.md) | Adapter manifests, format 1, as built | Accepted | 2026-11-12 |
| [0108](0108-map-preview.md) | A preview of the map: file, live image and routes | Accepted | 2026-11-12 |
| [0109](0109-osm-stairs-are-stairway-candidates.md) | OSM stairs are stairway candidates | Accepted | 2026-11-12 |
| [0110](0110-a-person-can-allow-a-state-one-more-host.md) | A person can allow a built-in state one more host | Accepted | 2026-11-12 |
| [0111](0111-saarland-the-first-state-on-the-adapter-scheme.md) | Saarland, the first state on the adapter scheme, and what it needed | Accepted | 2026-11-12 |
| [0112](0112-the-adapter-check-reads-a-folder-and-asks-its-services.md) | The adapter check reads a folder and, when asked, its services | Accepted | 2026-11-12 |
| [0113](0113-the-mouth-of-the-elbe-is-split-at-the-line-of-equal-distance.md) | The mouth of the Elbe is split at the line of equal distance | Accepted | 2026-11-13 |
| [0114](0114-the-extra-layers-and-raster-templates-of-an-adapter.md) | The extra layers and raster templates of an adapter, as built | Accepted | 2026-11-13 |
| [0115](0115-the-map-assembly-reads-no-file.md) | The map assembly reads no file | Accepted | 2026-11-13 |
| [0116](0116-forest-map-kind.md) | Forest map kind | Accepted | 2026-11-13 |
| [0117](0117-permit-notes-from-national-sources.md) | Permit notes from national sources | Accepted | 2026-11-13 |
| [0118](0118-paper-and-symbol-measures-are-types.md) | Paper and symbol measures are types | Accepted | 2026-11-13 |
| [0119](0119-baden-wuerttemberg-ndom-on-the-terrain-and-alkis-by-code.md) | Baden-Württemberg: the nDOM1 on the terrain, and the ALKIS of the views by code | Accepted | 2026-11-13 |
| [0120](0120-permit-sources-of-region-adapters.md) | Permit sources of region adapters | Accepted | 2026-11-13 |
| [0121](0121-bremen-the-full-alkis-as-the-simplified-schema.md) | Bremen: the full ALKIS as the simplified schema, and grid points taken as cells | Accepted | 2026-11-13 |
| [0122](0122-a-sliver-is-measured-by-its-short-side.md) | A sliver is measured by its short side | Accepted | 2026-11-13 |
| [0123](0123-bavaria-the-cadastre-from-files.md) | Bavaria: the cadastre from files | Accepted | 2026-11-14 |
| [0124](0124-forest-administrations-in-the-permit-notes.md) | Forest administrations in the permit notes | Accepted | 2026-11-14 |
| [0125](0125-steep-forest-gets-10-m-contours.md) | A steep forest map gets contours every 10 m | Accepted | 2026-11-14 |
| [0126](0126-cliffs-on-the-forest-map-are-generalized.md) | Cliffs on the forest map are generalized | Accepted | 2026-11-14 |
| [0127](0127-permit-notes-follow-the-map-area.md) | Permit notes follow the map area | Accepted | 2026-11-14 |
| [0128](0128-bavaria-vegetation-from-the-point-clouds.md) | Bavaria: vegetation from the point clouds | Accepted | 2026-11-14 |
| [0129](0129-permit-notes-for-hamburg-water-protection-and-forest-functions.md) | Permit notes for Hamburg, water protection and forest functions | Accepted | 2026-11-14 |
