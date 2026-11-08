# 0041. Rhineland-Palatinate: ALKIS from `ave:Nutzung` on the origin host, dates from `gml:timePosition`

Date: 2026-11-08
Status: Accepted

## Context

ALKIS-vereinfacht for Rhineland-Palatinate runs on MapServer at `geo5.service24.rlp.de/wfs/alkis_rp.fcgi`. The open-data
page links the GeoPortal.rlp registry proxy (`registry/wfs/519`) in front of it. The proxy returns the same bytes plus
15 cookies, and for the boundary WFS it answers every request with an HTML page and status 200. The service has no
`ave:NutzungFlurstueck`. Its `ave:Nutzung` holds unsplit objects with both `nutzart` and `bez`. MapServer wraps `aktualit`
in `<gml:timePosition>`, which `parse_gml` did not read. Some names are several names joined with `;`.

## Decision

freemapper uses the origin hosts (`geo4` and `geo5.service24.rlp.de`, `geobasis-rlp.de`) and never the registry proxy.
ALKIS goes through the shared `alkis_wfs` reader with `ave:GebaeudeBauwerk` and, as for Saxony-Anhalt, `ave:Nutzung` as
the land-use type. Squares without `bez` get `square_bez` "ohne Unterart (Rheinland-Pfalz)" and become candidates.
`parse_gml` reads `aktualit` from a `gml:timePosition` at any depth below it; the GML of the other states reads as before.
Names joined with `;` stay as they are. At the Hesse border freemapper reads both ALKIS services and keeps a shared oid
once. The test does not pin which copy wins, because the same oid is the same feature.

## Consequences

- RP buildings and land use carry their `aktualit` date like those of the other states.
- The spelling "Unland, Vegetationslose Fläche" stays unmapped, because no build rule uses it.
- A joined name shows as one long name.
- Cost if wrong: if the origin hosts move, freemapper needs the new address, while the registry URL might still work.
  The border de-duplication costs nothing visible.

Source: Rhineland-Palatinate reference §2, §8, §13 items 2, 3 and 10; plan
`docs/superpowers/plans/2026-11-08-rheinland-pfalz.md` (Rulings); rulings ledger, Task 2 (Mainz border oid); commit
13b1912 on branch `rheinland-pfalz`.
