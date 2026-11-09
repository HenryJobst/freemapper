# 0052. The Berlin street survey gives candidates and green cut-outs, never map objects

Date: 2026-11-09
Status: Accepted

## Context

The Straßenbefahrung recorded Berlin's public street space from 2014 to 2016 (WFS `strassenbefahrung`,
dl-de/zero-2-0). It has carriageways, sidewalks, green strips, curbs, walls, railings, stairs and street
furniture, and no newer edition. In the live samples 10.1 % (Viktoriapark) to 26.7 % (around Alexanderplatz) of
the ALKIS street area is not surveyed: verges and places rebuilt since 2014. The attributes are codes without a
published code list. freemapper draws the ALKIS street parcels as 501.5 map objects, green strips included
(spec §4.5).

## Decision

- The survey never replaces ALKIS. The ALKIS street parcels stay the 501.5 map objects.
- The only change to map objects: the surveyed green strips (`ce_gruenflaeche`) are cut out of them. Tree pits
  (`bn_baumscheibe`, 2 to 5 m²) are not cut; they are below the ISSprOM minimum. The cut object keeps its ALKIS
  provenance and names the strips in `freemapper:cut`, `freemapper:cut_ids` and `freemapper:cut_currency`.
- The green strips are also 401 candidates (plan table), built in `build/paved.rs` with the survey tags.
- Surveyed paving becomes 501.5 candidates where it lies outside every parcel that `paved.rs` draws (map objects
  and candidates that survive the sliver filter) and outside the cut-out strips. The plan said "minus
  `Straßenverkehr`"; `Platz` objects and `Weg` candidates are 501.5 too, so subtracting only `Straßenverkehr`
  would stack candidates on them. Each survey feature stays its own candidate, with its id, date and codes.
- Everything else from the survey is a candidate. No rule depends on an undocumented code; the codes are copied
  as `berlin:<code>`.
- The survey date comes from `gis_id` (four spellings). An unreadable id gives 2014-01-01 and
  `freemapper:currency_guessed=yes`; `cut_currency` prefers a real date and marks a guessed one with
  `freemapper:cut_currency_guessed=yes`.

## Consequences

- Berlin sprint maps lose the green strips inside street areas. A wrong cut leaves a white spot that the mapper
  fills from the orthophoto, not a wrong object.
- Park paths, forecourts and squares that ALKIS lacks show as candidates.
- An existing map gets the cut only with `run --force`.

Source: `docs/regions/berlin-plus-reference.md` §6; plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`
(Rulings); branch `berlin-plus-2`, Task 2 and its review.
