# 0031. North Rhine-Westphalia: orthophoto from the WMS, because the tiles are JPEG 2000 only

Date: 2026-11-08
Status: Accepted

## Context

Geobasis NRW publishes the orthophoto (DOP10, RGBI) as 1 km tiles of 10000 x 10000 px, 30 to 80 MB each, in JPEG 2000
only. freemapper has no JPEG 2000 decoder. The WMS `wms_nw_dop` serves the same pixels (mean difference 3.0 of 255
against the decoded tile), accepts requests up to 5000 px per side and reprojects, but it has no time dimension and
serves only the current flight.

## Decision

The orthophoto comes from the WMS, layer `nw_dop_rgb`, in EPSG:25832, as JPEG pieces of at most 4000 px on the grid,
through the shared WMS orthophoto path. The flight date comes from GetFeatureInfo on `nw_dop_utm_info`
(`Bildflugdatum`), which gives one date per tile. We do not add a JPEG 2000 decoder.

## Consequences

- No new dependency and no decoder to keep safe against hostile files.
- Only the current flight is available; the demo and the project entry state its date.
- The flights are 2022 and 2024 to 2026, partly leaf-off and partly leaf-on, so the date per tile matters.
- Outside the tile set the WMS answers white; at the Hesse border the rasters come from the state of the map centre.
- Cost if wrong: a slow WMS or one that changes its limits slows every NRW map, and a decoder would still be needed to
  read the tiles directly.

Source: NRW reference §1, §4.2, §4.3; NRW plan `2026-11-08-nordrhein-westfalen.md` (Rulings) and commit fed28f9 on
branch `nordrhein-westfalen`.
