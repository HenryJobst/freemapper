# 0207. Ways and streets on schoolyard maps

Date: 2026-11-28
Status: Accepted, amends ADR 0073, ADR 0098 and ADR 0101

## Context

A schoolyard map had no rule for OSM ways and no paved areas from the cadastre: `highway=*` had only sprint and
forest codes, and `paved()` of the assembly returned nothing for any kind but sprint. The design of 2026-11-07 meant
the map for the school grounds alone, where everything off the grounds is a 520 candidate. A user in Auerbach
(Saxony) made a schoolyard map of 800 × 600 m for a holiday camp and found no streets and no paths, not even among the
candidates (issue #334). The same area as a sprint map was what they wanted.

The IOF school set 2019 has every ISSprOM symbol at 125 %, so it has the line symbols 501.6 to 501.9, 505, 506,
532.6 to 532.9, 522 and the paved area 501.5.

## Decision

- Schoolyard maps take the OSM way rules of the sprint map: the same target (a way with a known surface is a map
  object, one without is a candidate) and the same codes. These are 505.1, 505.2, 506, the 520 corridors for
  private ways, the 522 passages, the firm and paved ways 501.6 to 501.9 and the stairs 532.6 to 532.9.
- The line symbols 501.6 to 501.9 and 532.6 to 532.9 have a footprint, and the `width` limits of the rules choose
  the symbol by it. The footprint at 1:1000 and 125 % is 0.3125 times that at 1:4000, so the schoolyard copies of
  these rules take the limits times 0.3125 (4 m becomes 1.25 m, 3 m 0.95 m, 2 m 0.65 m, 3.5 m 1.1 m, 2.5 m 0.8 m,
  1.8 m 0.6 m). Limits that say what a way is, not how it is drawn (a path of 1.5 m or more, a track of 3 m), stay.
- `paved()` of the assembly draws the 501.5 paved areas of the traffic land use on a schoolyard map too, but
  without the parts that are 520 candidates (`is_out_of_bounds_part`). A map of the school grounds keeps the
  streets off its land as 520; a map of a larger area, where the parcels count as its own land, gets its streets.

## Consequences

- Schoolyard projects get the OSM ways and, in a large area, the street areas. The examples are not rebuilt; they
  change the next time they are built online.
- Where OSM and the cadastre draw a path twice, the cuts of sprint maps (`firm_paths`, stairs) apply as they do
  there.
- Cost if wrong: a schoolyard map of the school grounds gets ways at the edge that lie on 520 land, or a wide path
  drawn thinner than it is, because the school set has no wider footprint than 1.25 m on the ground.

Source: issue #334, user decision 2026-11-28; IOF school symbol set 2019; ADR 0073, 0098, 0101.
