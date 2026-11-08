# 0033. Lower Saxony: dates from STAC, the newest item at the centre

Date: 2026-11-08
Status: Accepted

## Context

The WMS and the WCS give no dates. LGLN runs one STAC API per product (`dop`, `dgm`, `dom.stac.lgln.niedersachsen.de`)
that lists every tile with its date, and it keeps all historic vintages: a search returns several items per cell, up to
nine for one 2 km DOP block, in two tile schemes. A search takes 0.2 to 0.3 s and pages by a `next` link.

## Decision

freemapper searches each STAC with a box reaching 10 m from the area's centre, converted to lon/lat, and takes the
item with the newest `datetime`. The DOP item gives the flight date, and its month gives the leaf state (May to October in
leaf). The DGM and DOM items give the dates of the height models. One DOP search runs per fetch. Paging follows `next`
links on the same host only, stops at an empty page or a page already read, and fails after 10 pages. The answers are
not cached; the dates live only in the project record. An empty or failed search gives no date and a warning, not an
error. Offline, no search runs and the record keeps its dates.

## Consequences

- No tile list to download, store or keep in step with the server.
- Online fetches always see the newest vintage at the centre.
- Near a tile corner the search can match neighbouring tiles; the newest of them wins.
- Cost if wrong: an unreachable STAC leaves new projects undated, and a map on a vintage seam shows one date only.

Source: Lower Saxony reference §2.2, §4.1, §7; plan `docs/superpowers/plans/2026-11-08-niedersachsen.md` (Rulings);
commits 2f1fd54, 4a89d08 and e2e08fc on branch `niedersachsen`.
