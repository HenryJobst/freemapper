# 0047. Berlin extra layers become candidates through the candidate rule engine

Date: 2026-11-09
Status: Accepted

## Context

Geoportal Berlin publishes ALKIS structures (walls, fences, stairs, bridges, monuments), relief edges and hedge
lines as WFS layers under dl-de/zero-2-0. Their features carry German attribute values (`bezbwf`, `bezart`,
`bezbws`) and no height or passability. The OSM candidates already run through a TOML rule engine
(`osm-candidates.toml`, `Rules`).

## Decision

- Fetch: a table of six layers in `sources/berlin.rs` (`EXTRA_LAYERS`), loaded with `fetch_wfs` only when the
  widened bbox touches the Berlin region; stored in `data/berlin/<file>.geojson`. A failing layer is a warning.
  De-duplication uses the top-level GeoServer `id`.
- Rules: a separate embedded file `assets/rules/berlin-candidates.toml` (`Rules::berlin`), same syntax and engine.
  Tags are `berlin:layer=<file>` plus `berlin:<property>`, so an OSM rule cannot match a Berlin feature and the
  other way round. A value ending in `*` is a prefix (`Mauerkante*`).
- Geometry: a point stays a point, each MultiLineString part becomes a line, a polygon becomes its exterior as a
  closed line plus a centroid point. Holes are dropped knowingly: these are candidates.
- Symbols by the ALKIS name, height decides nothing: `Mauer`, `Mauerkante*`, `Mauermitte` → 515; `Stützmauer*` →
  513.2; `Zaun` → 518; `Heckenkante*` → 410.1. These are the symbols of the OSM rules, so the mapper sees one
  convention.
- Ruling (Task 2 review): a wall that ALKIS records with edges and a centre line keeps all its 515 lines. The
  mapper deletes the extras. Picking one line per wall would need geometry matching that the data does not
  support reliably.
- Passages (`Durchfahrt im Gebäude`, `Durchfahrt an überbautem Verkehrsweg`, `Arkade`) are 522 candidates and the
  521 building stays whole, because the data does not say whether a passage is gated. `Auskragender Geschossteil
  / zurückspringendes Geschoss` is dropped.
- Monuments → 526 point, `freemapper:size=large` above 3 m; `Brücke` → 501.5 and `relief` `Oberkante` → 104 on
  sprint maps only. ALKIS kerb lines are not taken (plan 2 brings the surveyed curbs).
- Provenance says `geoportal-berlin/<file>`, not `osm`: `osm_objects` takes the source name and the currency.
  The ALKIS name is copied as `freemapper:alkis`.

## Consequences

- New Berlin candidate kinds need a rule line, not code. Stairs and canopies need geometry work and have builders
  of their own (ADRs 0046, 0048).
- Walls with edges and centre line show two or three parallel 515 lines in the candidates.
- Cost if wrong: a wrong ALKIS value spelling silently matches nothing; the fixtures from live data guard the
  spellings in use.

Source: `docs/regions/berlin-plus-reference.md` §3 to §5; plan `docs/superpowers/plans/2026-11-09-berlin-plus.md`;
branch `berlin-plus`, Tasks 1 and 2.
