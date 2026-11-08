# 0040. Rhineland-Palatinate: missing tiles are reported and checked again after 30 days

Date: 2026-11-08
Status: Accepted

## Context

The `Laserbefliegung` index can name a tile that the file server lacks; the server then answers 404. For NRW a new
release of the tile list clears the notes of such tiles (ADR 0030). The RP index has no release date, so nothing tells
freemapper when a missing tile may have arrived.

## Decision

A listed tile that answers 404 is reported missing, its part of the templates stays empty, and freemapper writes the
note `<name>.missing` into the cache. Offline the note always holds. Online it holds for 30 days, by the file's
modification time; after that the next run asks again. A successful download removes the note. A tile the index does not
name is reported missing too if its centre lies in Rhineland-Palatinate; unlisted tiles beyond the border are skipped
without a message. A new scan year gives a new name and so a new cache file. The size estimate counts a tile with a fresh
note as cached.

## Consequences

- A gap costs one 404 per tile and month instead of one per run.
- A tile the server adds later is used at most 30 days late.
- The warning names the missing tiles by their short names (`32_400_5579`).
- Cost if wrong: a new tile stays empty for up to a month. Deleting the note in the cache forces a new check.

Source: Rhineland-Palatinate reference §2.2, §12; plan `docs/superpowers/plans/2026-11-08-rheinland-pfalz.md` (Rulings);
Task 3 report, concern 2 (the 30 days are the implementer's choice); commits efdcb39 and 8072ac0 on branch
`rheinland-pfalz`.
