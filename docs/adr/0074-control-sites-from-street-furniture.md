# 0074. Control sites from street furniture

Date: 2026-11-10
Status: Accepted

## Context

Course setters use small man-made objects as control locations: a bollard, a street lamp, a waste basket, a bicycle
rack, a bench. A sprint map does not draw them (ISSprOM maps only what a competitor must see to navigate), and OSM
has plenty. The census counted 68 bollards, 176 street lamps, 115 waste baskets and 34 bicycle racks in the Viktoriapark box,
322 lamps, 77 baskets and 88 racks in the Lübeck Altstadt, and the Berlin street survey 2,392 bollards per km². Drawn
as map objects they would bury the map, and the census advised against them as symbols.

What the norm says (ISSprOM 2019-2, section 4.5):

- 530 Prominent man-made feature, ring, and 531 Prominent man-made feature, x: "A man-made feature which is
  significant or prominent. The definition of the symbol must be given on the map." Footprint 4 m (530) and 4.8 m
  (531). 531 is a point symbol, oriented to north.
- There is no sprint symbol for a lamp, bollard or bin as such. 545 (lamp post) and 543 (bench) belong to the school
  symbol set, not to ISSprOM.
- 526 is for memorials and boundary stones, 525 for a small tower: neither fits.

## Decision

- **Symbol.** All five kinds are **531** candidate points on sprint maps. The definition the norm asks for belongs in
  the legend of the finished map and is the mapper's to write. The kind is in the tag.
- **Candidates only.** None is a map object: a control site is a choice of the course setter, not a fact of the terrain.
  The rules in `osm-candidates.toml` have no `target = "map"`. On schoolyard maps nothing changes (545 and 543 stay
  there; bollards, waste baskets and racks have no schoolyard symbol).
- **Way to find them.** Each carries `freemapper:control=bollard|street_lamp|waste_basket|bicycle_parking|bench`, next to the
  usual `freemapper:source=osm`, `freemapper:source_id=node/…` and `freemapper:osm=…`. In OOM, all of them are
  found by the symbol (Select all objects with the same symbol: 531), the kind by the tag. A planning tool such as
  freemapper-courseplan can read the tag from the candidates file; this ADR only names it, courseplan gets no code.
- **Not flooding.**
  1. *Same kind within 25 m is no site* (`build/control_sites.rs`). A row of bollards (posts 1 to 5 m apart), a
     pair of benches or the lamps along a street cannot be told apart on the ground, so neither is kept. 25 m is
     6 mm at 1:4000; the norm and the IOF rules name no figure, so it is a ruling. Objects of another kind do not count,
     and the user's example (bollards less than 8 m apart) is inside this limit. Bollard ways (`barrier=bollard` on a way)
     are not read, so a drawn barrier line stays a barrier.
  2. *The gap between point symbols* of `build/points.rs` (ADR 0066) holds as before.
  3. *A control site gives way.* It never knocks out another point. Without this, the 531 (1.27 mm) beat tree candidates
     (418, 0.8 mm) and fountains (311, 0.9 mm) in the first test run: 53 trees and 2 fountains went. Now a control site
     is thinned first, in `generalize_points` (`Spot::weak`).
  4. A site in a 520 area is hidden as every candidate point is (`freemapper:hidden=520`).
- **Berlin.** The survey (`av_poller`, `bj_sitzbank`) and the lighting register (`beleuchtung`) are schoolyard layers by
  design (their rules say why). On sprint they would need the same logic: 531, the same filter, the same tag. They are not
  switched on here, because that changes which layers a Berlin sprint project fetches, and this change must work from the
  cache. OSM lamps and bollards cover the Viktoriapark. Follow-up when Berlin sprint maps are rebuilt online.
- A bench is a point on sprint maps and a short line on schoolyard maps: the rules skip a rule without a symbol for
  the map kind (ADR 0073), so both coexist.

## Consequences

- In the Viktoriapark the candidates grow by 116 control sites (531 goes from 5 to 121 points), 18 of them hidden under 520
  areas: 74 lamps, 21 waste baskets, 10 bicycle racks, 6 benches and 5 bollards. With a limit of 8 m instead of 25 m there
  were 267 sites. The changelog lists the counts.
- A course setter finds fewer sites than exist. A lamp in a series is a poor control anyway. The limit is a constant,
  `SAME_KIND_M`.
- Existing maps get the points with `run --force`.
- Amendment (ADR 0076): a control site inside a building footprint is dropped, on sprint maps.
- Not done: ranking sites by prominence (a lamp near a path junction), sites from ways (bollard rows, bicycle parking areas),
  and the Berlin survey layers on sprint.
- Amendment (ADR 0077): the Berlin survey and the lighting cadastre now give control sites on sprint maps, thinned with the OSM ones.
