# 0064. NAS stairs run up from their bottom edge

Date: 2026-11-09
Status: Accepted

## Context

ADR 0048 takes the walking direction of a Berlin stair from the DGM1 and guesses when the stair is flat in it.
The full ALKIS has stairs as areas (`bauwerksfunktion` 1620 Treppe, 1621 Freitreppe) and, in some states, the
stair bottom edge (1630 Treppenunterkante) as a separate line linked to its stair by `gehoertZu`. That edge says
where the stair starts. Brandenburg has no NAS stairs (1620, 1621, 1630: 0 statewide, checked 2026-11-09);
Mecklenburg-Vorpommern and Hesse have them.

## Decision

- `build/stairs.rs` reads the stored `sonstiges-bauwerk` layer (`StairInput::nas`): areas with 1620/1621 and
  edges with 1630 whose `gehoert_zu` names the stair's oid.
- With an edge, the axis runs across the edge, from the end nearest to it, uphill away from it. The line is not
  tagged `freemapper:direction=guessed`. An edge that lies outside its stair area is ignored, and an edge alone
  draws nothing.
- Without an edge, the DGM1, the facade and the short axis decide as in ADR 0048. `derive` then keeps
  `data/terrain.tif` for the export, as it does for the Berlin stairs.
- The symbol comes from the paper width, the same `StairWriter` and wide-stair rule as ADR 0048. Tags:
  `freemapper:source=alkis-nas/sonstiges-bauwerk` plus the NAS tags (ADR 0062); currency `beginnt`, else the
  fetch date.
- A surveyed stair over a NAS stair is dropped like one over a Berlin ALKIS stair (ADR 0055).
- No OSM stair dedup: no OSM rule emits 532.x, so there is no OSM stair to drop. When one is added, it takes the
  same overlap rule with ALKIS winning.

## Consequences

- Stairs in later NAS states get a reliable direction where the cadastre has the edge, and fewer `guessed` tags.
- The fixture (`mecklenburg-vorpommern-nas-stairs.gml`) has a real stair area and a constructed edge, because the
  probe held no matched pair. The first live run in Mecklenburg-Vorpommern should replace it with a real pair.
- Cost if wrong: an edge recorded at the top would turn the stair round; the mapper sees no `guessed` tag there.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Task 4); ADR 0048 (consequence
line); branch `states-plus-1`, Task 4 and its review.
