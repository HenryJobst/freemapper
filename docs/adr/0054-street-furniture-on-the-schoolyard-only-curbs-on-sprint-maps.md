# 0054. Street furniture on schoolyard maps only, curbs on sprint maps only

Date: 2026-11-09
Status: Accepted

## Context

The street survey has 344 666 bollards and 524 409 curb lines citywide. Around the Helene-Haeusler-Schule one
km² holds 2 392 bollards and 37 km of curbs. On a schoolyard map at 1:1000 a bench or a lamp is a control
feature; on a sprint map at 1:4000 most of them are below what the map shows. ALKIS also has curb lines
(`Fahrbahnbegrenzungslinie`), which plan 1 left unused.

## Decision

- Schoolyard maps only: benches (543), hydrants (321), bollards, masts and flag poles (544), play equipment
  (541), drinking fountains of the survey and the register (313) and lamps (545).
- Lamps come from the lighting cadastre (`beleuchtung`, only lamps in operation), not from the survey's
  `as_mast`, because the cadastre is maintained (updated 2026-09-30).
- Sprint maps only: curbs (`bd_bordstein`) as 501.1 candidates (user decision, as in the plan) and advertising
  columns (531). ALKIS curb lines stay unused.
- Both kinds: survey walls (515), railings, traffic barriers and crash barriers (518), bus shelters (522),
  pumps and fountains (311), survey stairs (532.6 to 532.9).
- Railings are 518 like the OSM fences. The mapper downgrades them to 516 where they can be crossed.

## Consequences

- Sprint candidates stay readable; the schoolyard candidates show the furniture a course can use.
- A sprint map has no bollards or lamps from Berlin data; OSM lamps stay schoolyard-only as before.
- Cost if wrong: a prominent bollard on a sprint map has to be drawn by hand.

Source: `docs/regions/berlin-plus-reference.md` §6, §7; plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`
(Rulings); user decision in the ledger; branch `berlin-plus-2`, Tasks 3 and 4.
