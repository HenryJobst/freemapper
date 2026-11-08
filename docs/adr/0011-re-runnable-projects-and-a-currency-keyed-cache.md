# 0011. Re-runnable projects and a cache keyed by data currency

Date: 2026-11-07
Status: Accepted

## Context

State data is republished over time, and the OSM extract changes daily. A mapper must be able to re-run a
project and know which data went into it. Byte-identical output across months is not possible with live sources.

## Decision

Projects are re-runnable and traceable, not byte-identical across time:

- `freemapper.toml` records the URL and data currency of every tile, the OSM extract timestamp and the
  declination date, which is fixed at `init`.
- The shared cache is keyed by tile id and data currency and keeps the latest version per tile. When the
  upstream currency changed, `fetch` warns, uses the new data and updates the record. `--offline` uses the cache
  only.
- Byte-identical output is tested only for fixed inputs.
- Added 2026-11-08: an orthophoto without any flight date is reused only when its record covers the area and is
  less than 30 days old.

## Consequences

- A product with no published currency needs a stand-in key, such as the LGB nDOM's `Last-Modified` date.
- Cost if wrong: stale data stays in use, up to 30 days for providers without flight dates.

Source: spec §4.4 (nDSM), §4.7; MV rulings (Final); commits 011c99b, 939f330.
