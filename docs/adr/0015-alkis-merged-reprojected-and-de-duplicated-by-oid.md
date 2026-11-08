# 0015. ALKIS from each touched state, merged, reprojected and de-duplicated by oid

Date: 2026-11-07
Status: Accepted

## Context

Each state runs its own ALKIS service: OGC API Features in Brandenburg, Hamburg and MV, a WFS elsewhere,
answering GML or, in Berlin, GeoJSON.
Field values are spelled differently, for example Hamburg's ASCII class names. A building on a state border can
come back from both services.

## Decision

- Each state's features are normalised to the LGB property schema, so the stored files and the build rules stay
  the same. WFS states share one reader with a `WfsService` per state.
- The merged fetch asks every touched region (ADR 0013) and reprojects each source into the project CRS
  (ADR 0014).
- Features are kept once per `oid`, both across result pages and across services. The oid prefixes of the
  states never collide (`DEBBAL`, `DEBE`, `DESN`, `DEST`, `DEHE`, `DESH`, `DEMV`).

## Consequences

- Rules are written once for all states.
- Each new state needs a check of its spellings, `rellage` and squares without `bez`.
- Cost if wrong: duplicate buildings on border maps, or a lost object when two states share an oid.

Source: spec §4.1 (Berlin, Saxony-Anhalt, Hamburg); plans `2026-11-07-berlin.md`, `2026-11-07-saxony-anhalt.md`,
`2026-11-07-polish-deferred-minors.md`; commits 4a1ecb8, 62747e1, 86bf930, d5462a9; `sources/alkis.rs` `merged`.
