# ADR 0195: Building kinds that the Polish register sends in words

## Status

Accepted, 24 November 2026. Extends [ADR 0171](0171-poland-cadastre-egib.md).

## Context

The open item "PKOB letters" (`docs/regions/poland-reference.md`) meant the finer letters of the newer building scheme
(`mj`, `md`, `mt`, `mz`, `a`, `u`, `d`, `o`, `e`, `f`, `r`, `x`), which `building_function` already maps but which no answer
of the service ever held. Measured again on 24 November 2026 on `mapy.geoportal.gov.pl` only (`ms:budynki`, one page of
at most 1000 buildings per box, no personal data in the requests):

| Area | Buildings | `RODZAJ` |
|---|---|---|
| Słubice, 500 m square | 421 | `m` 133, `t` 110, `i` 81, `h` 80, `k` 8, `b` 6, `z` 2, `p` 1 |
| Kraków, 500 m square | 161 | `m` 79, `i` 45, `t` 22, `h` 6, `b` 4, `k` 3, `z` 2 |
| Gdańsk, 500 m square | 107 | `m` 65, `t` 17, `h` 10, `i` 9, `b` 4, `s` 2 |
| Wrocław, 500 m square | 135 | `m` 59, `g` 57, `i` 8, `p` 6, `t` 3, `b` 1, `s` 1 |
| Łódź, 800 m square | 66 | `g` 51, `m` 15 |
| Poznań (powiat 3064), four boxes | about 1450 | words, see below |

No PKOB letter occurred in any of them: the finer letters stay unverified, and the table keeps them as before (nothing
is guessed beyond the two official lists). What did occur is another gap: **Poznań sends the class names of the KŚT in
words, cut off at 20 characters** (`budynki handlowo-usł`), in all four boxes (in one of them beside plain letters). `building_function` gave the text itself, so a building there had
`funktion` `budynki mieszkalne` and no German name.

## Decision

- The ten cut names map to the letter of their KŚT group, the same ten the documented `kodKst` 101 to 110 give:
  `budynki mieszkalne` m, `budynki biurowe` b, `budynki handlowo-usł` h, `budynki oświaty, nau` k, `budynki
  produkcyjne,` g, `budynki przemysłowe` p, `budynki szpitali i i` z, `budynki transportu i` t, `pozostałe budynki ni`
  i, `zbiorniki, silosy i ` s. The match is a prefix, so a service that sends the whole name reads the same.
- Anything else keeps its text, as before.
- `funktion` is an attribute only: no map object or rule reads it, so no map, golden or example changes.
- Adapter revision 13.

## Consequences

- Poznań buildings carry the same German `funktion` as every other town.
- The PKOB letters remain unverified; the item can be closed as "not occurring in the service".
