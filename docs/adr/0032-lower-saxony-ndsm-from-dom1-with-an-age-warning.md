# 0032. Lower Saxony: nDSM from DOM1 minus DGM1, with an age warning

Date: 2026-11-08
Status: Accepted

## Context

LGLN offers two surface models. The DOM1 comes from the laser scan, has the same lot, date, grid and format as the
DGM1 and costs about 4 MB per km². It is usually 5 to 9 years older than the orthophoto: 79 % of the tiles differ by 5
years or more, and the Hannover school has a 2016 scan under a 2025 orthophoto. The image-based bDOM20 has the date of
the orthophoto on 96.7 % of the tiles, but it is 146 MB per km² uncompressed, and the spring flights are leaf-off, so
image matching loses the tree crowns (school: 36 % of cells above 2 m against 64 % with the DOM1).

## Decision

The nDSM is DOM1 minus DGM1. bDOM20 is not used. When the DOM1 is more than `SURFACE_AGE_WARNING_MONTHS` (24) older
than the orthophoto, freemapper warns with both dates. It compares full dates, because STAC gives days. Exactly 24
months does not warn. The flight date in the project record is compared, so an orthophoto kept after a failed search
still drives the warning. Offline there is no new date and no warning.

## Consequences

- The nDSM keeps the crowns and costs one small WCS download per model.
- Most Lower Saxony maps get the warning, and new buildings can be missing from the nDSM.
- At the coast, where the 2025 scan meets a 2025 or 2026 orthophoto, the warning stays quiet.
- Cost if wrong: an nDSM that misses recent building work; bDOM20 reads would add a range reader and large downloads.

Source: Lower Saxony reference §6, §13 item 3; plan `docs/superpowers/plans/2026-11-08-niedersachsen.md` (Rulings);
commit e2e08fc on branch `niedersachsen`.
