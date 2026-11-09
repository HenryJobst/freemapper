# 0057. Berlin building dates come from the 2022 building heights layer

Date: 2026-11-09
Status: Accepted

## Context

The Berlin ALKIS buildings carry no date, so their 521 objects had no `freemapper:currency`. The Umweltatlas
layer `ua_gebaeudehoehen:gebaeudehoehen` ("Gebäudehöhen 2022") has a footprint date
(`grundrissaktualitaet`) per ALKIS building, keyed by `gml_id` = the ALKIS uuid. In Viktoriapark 418 of 430
buildings match. The LoD2 tiles have newer dates but need 3.3 MB per km² and a CityGML reader.

## Decision

- freemapper fetches the heights layer with the other Berlin layers (`gebaeudehoehen`, both map kinds) and joins
  it when the stored buildings are read (`join_footprint_dates`): `aktualit` = `grundrissaktualitaet` of the
  feature whose `gml_id` equals the building `oid`. The 521 object gets it as `freemapper:currency`.
- An old date beats none. The footprint date is the state of the ALKIS outline that the 2022 data used, not a
  survey date.
- A building without a match, or a project without the layer (also outside Berlin), keeps no date.
- The building record's `latest_update` stays `None` in Berlin. It is the ALKIS change detection; a date from a
  separately fetched layer would give false "changed upstream" warnings.
- Attribution `Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0` when the layer has features.

## Consequences

- Berlin buildings show a currency per object, often years before the fetch.
- The project record still has no building currency for Berlin.
- An existing map gets the dates only with `run --force`.

Source: `docs/regions/berlin-plus-reference.md` §8; plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`;
branch `berlin-plus-2`, Task 4 and its review.
