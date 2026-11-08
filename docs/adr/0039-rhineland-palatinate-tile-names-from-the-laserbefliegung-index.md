# 0039. Rhineland-Palatinate: tile names from the Laserbefliegung index, the Metalink for unnamed tiles

Date: 2026-11-08
Status: Accepted

## Context

The DGM1 and DOM1 are 1 km GeoTIFF tiles whose names end in the year of the laser scan (`dgm1_32_400_5579_1_rp_2024.tif`),
2022 to 2025 by tile. The directory listing is 9 MB of HTML and takes 52 s; a wrong year is a 404 that can take seconds on
this server. The Blattschnitt WFS has a layer `Laserbefliegung` with one feature per tile: `kachel`, `flugjahr` and the
flight days `flugtage`. A bbox query takes 0.2 s. Its year matches the file name for 21,147 of 21,160 tiles; 13 tiles
north of Trier have neither year nor days. The file server also keeps a Metalink per Land (11 MB per product), Kreis and
Gemeinde with the exact file names.

## Decision

freemapper asks the `Laserbefliegung` layer for the map area, on the origin host, and builds each name from the year of
the flight days, or from `flugjahr` when there are no days. Days and a year that disagree, or days across two years, give
no name. Every name must pass a strict pattern before freemapper builds a URL from it. The date of a tile is its last
flight day. freemapper never tries years. For listed tiles without a year it reads the Land Metalink of the model, only
when the area contains such a tile, and reuses the kept file for 30 days. A tile still without a name is missing, with a
warning that says why. The list is kept per area for offline runs and as a fallback, and reads of the list and the
Metalink stop at 32 MB. The mosaic of named tiles is shared with Geobasis NRW (`height_model::mosaic_tiles`).

## Consequences

- Two list requests of about 0.2 s per run (one per model), then the tiles; DGM1 and DOM1 share one flight.
- Only the 13 tiles north of Trier need the 11 MB Metalink. A Kreis Metalink is smaller but needs a municipality key,
  which freemapper cannot look up. The size estimate counts the Metalink when a kept list needs it.
- The shared fetcher retries a 404 of the list or the Metalink three times, about 6 s when a URL is gone.
- Cost if wrong: if the index lags behind the files, a newer scan is not used; without the Metalink the 13 tiles stay
  empty.

Source: Rhineland-Palatinate reference §2.1, §2.2, §5, §13 item 1; plan `docs/superpowers/plans/2026-11-08-rheinland-pfalz.md`
(Rulings); rulings ledger, Task 3 (unnamed tiles, Metalink fallback, one list per model, 404 retries); commits efdcb39,
cafd6a4, 5dacc98, 8072ac0 and c8efe94 on branch `rheinland-pfalz`.
