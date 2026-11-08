# 0029. Lower Saxony: rasters via WMS and WCS instead of COG windows

Date: 2026-11-08
Status: Accepted

## Context

LGLN publishes its DOP, DGM1 and DOM1 as Cloud Optimized GeoTIFFs found through STAC. Window reads of the DOP COGs
would need a range-backed reader, YCbCr to RGB conversion and mask handling. The WCS values are byte-identical to
the COGs (Lower Saxony reference §5), and the WMS pixels match the tiles (§4). The WCS sends its headers only once the
whole image is ready: a 4 km piece took 48.6 s live, close to freemapper's 60 s response timeout.

## Decision

- Rasters are fetched per area: the orthophoto from the WMS `ni_dop20` in pieces of at most 4000 px, DGM1 and DOM1
  from the WCS (`ni_dgm1`, `ni_dom1`) through the shared `WcsService` in pieces of at most 2 km.
- An HTTP 200 answer that is not a TIFF, such as the HTML page "MapServer Message", is an error. Like every page that is
  not an OGC report, it is retried three times first (shared with Mecklenburg-Vorpommern).
- Dates come from STAC (ADR 0033). The nDSM is DOM1 minus DGM1; bDOM20 is not used (ADR 0032).
- COG window reads stay a later option.

## Consequences

- No COG range reader and no YCbCr conversion are needed.
- A 4 km area makes four GetCoverages per model and about 25 GetMaps, a few minutes in all.
- A failing WCS costs a few seconds of retries before the error.
- Cost if wrong: orthophotos about three times slower than range reads, and more requests per area than 4 km pieces.

Source: Lower Saxony reference §4, §5, §13, §14; plan `docs/superpowers/plans/2026-11-08-niedersachsen.md` (Rulings),
Task 3 ruling on the 2 km pieces, commits 2f1fd54, 4a89d08 and e2e08fc on branch `niedersachsen`.
