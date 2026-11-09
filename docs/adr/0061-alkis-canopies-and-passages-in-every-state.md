# 0061. ALKIS canopies and passages in every state

Date: 2026-11-09
Status: Accepted

## Context

ADR 0046 made the Berlin ALKIS canopies 522 map objects, and ADR 0047 made the Berlin building passages 522
candidates. Both came from Geoportal Berlin layers. The ALKIS vereinfacht buildings that freemapper already
stores for every other state (`data/alkis-buildings.geojson`) hold the same objects: `Sonstiges Bauwerk oder
sonstige Einrichtung` with `funktion` `Überdachung` or `Carport`, and `Bauteil` with `Durchfahrt im Gebäude`,
`Durchfahrt an überbautem Verkehrsweg`, `Arkade` or `Auskragender Geschossteil`. `build/buildings.rs` kept only
`Gebäude` and dropped them.

## Decision

- ADR 0046 now applies to every state. `Überdachung` and `Carport` from the stored buildings are 522 map objects
  on both map kinds (`CanopyInput::Buildings` in `build/canopy.rs`), with the same cut at the 521 footprints, the
  90 % inside share, the `(0.5 mm × scale)²` minimum and the 0.1 mm simplification. Underground ones
  (`rellage` = `Unter der Erdoberfläche`) are skipped.
- Tags: `freemapper:source=alkis`, `freemapper:source_id=<oid>`, `freemapper:currency=<aktualit>` (the feature
  date; the fetch date only without one) and `freemapper:alkis=Überdachung|Carport`.
- The three passage kinds are 522 candidates through the state rules (`state-candidates.toml`,
  `alkis:type=GebaeudeBauwerk`, `alkis:geom=area`). The 521 stays whole, as in ADR 0047. `Auskragender
  Geschossteil` is dropped.
- Values are compared by `lgv::spelling_key` (case, umlauts, no-break space, `-`, `,`, `/`). Hamburg stores
  `Sonstiges Bauwerk Oder Sonstige Einrichtung`, and projects stored before this change need no new fetch.
- Features with `bauart` come from `sources/berlin.rs`. They stay on the Berlin rules and the Berlin canopy
  input, so the Berlin output is unchanged.
- Saxony and Saxony-Anhalt get nothing. A statewide `RESULTTYPE=hits` count on 2026-11-09 found 0 passages and
  0 canopies in both (Saxony-Anhalt has 402 other `Bauteil`). No warning, because the data has none.

## Consequences

- Every state except Saxony and Saxony-Anhalt starts with canopies drawn and passages as candidates, with no
  new download. An existing map gets the canopies only with `run --force`, and `run` says so (its message still
  names "the Berlin canopies").
- Cost if wrong: as in ADR 0046, a demolished canopy is a map object the mapper has to delete.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Task 1);
`docs/regions/states-plus-reference.md` (finding 1, corrected); branch `states-plus-1`, Task 1 and its review.
