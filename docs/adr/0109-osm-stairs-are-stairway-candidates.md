# 0109. OSM stairs are stairway candidates

Date: 2026-11-12
Status: Accepted

## Context

ISSprOM 2019-2 §4.5 knows the stairway (532.6 to 532.9, a line that runs uphill, the footprint by width). Until now
a stair came from the Berlin registers and the full ALKIS (`build/stairs.rs`, ADR 0048). Hamburg has only the
simplified ALKIS, so the Blankenese example ("Treppenviertel") showed no stair at all. OSM maps them as
`highway=steps`, and the rules had a rule for them only when they are closed (520) or covered (522).

## Decision

- Four rules in `assets/rules/osm-candidates.toml` (sprint only) draw `highway=steps` as a line: 532.9 from a
  `width` of 3.5 m, 532.8 from 2.5 m, 532.7 from 1.8 m, else 532.6. The rules give no target, so they are candidates; the `balanced` profile sends them to the map (user decision 2026-11-12).
- Closed, covered, underground and indoor stairs match the rules before and after and draw no stairway.
- A way tagged `incline=down` is turned round, so the line runs uphill (`build/candidates.rs`).
- The line dedup (`build/dedup.rs`, ADR 0051) knows 532.6 to 532.9: an OSM stair with at least 80 % of its length
  within 1.5 m of an official stair of the same symbol is dropped, the official one names it in `freemapper:also`.
  A stair the sources draw with different widths shows twice until a width-blind comparison exists.

## Consequences

- Stairs appear in states without an official stair source; the `stairs` group of the content profile
  (ADR 0100) decides about the map: `balanced` takes OSM stairs too, `safe` leaves them candidates.
- A way without `incline` has no known direction; the candidate follows the way's order.
