# 0063. AdV code rules for the states and one duplicate rank for every state

Date: 2026-11-09
Status: Accepted

## Context

The NAS layers (ADR 0062) carry AdV codes, e.g. `bauwerksfunktion` 1740 for a fence. The code lists are the same
in every state; the labels are German text that services spell in different ways. The Berlin duplicate rules
(ADR 0055) lived in `build/berlin.rs` and knew only Berlin sources, but OSM fences and trees now meet NAS lines
and city trees in Brandenburg too.

## Decision

- `assets/rules/state-candidates.toml` (`Rules::states()`) runs on the same engine as the OSM and Berlin rules.
  Its tags are `alkis:<property>`, `alkis:type`, `alkis:geom` (`point`, `line`, `area`) and `tree:<field>`, so no
  OSM or Berlin rule can match them. NAS keeps `AX_` in `alkis:type`, vereinfacht stays `GebaeudeBauwerk`.
- Rules match codes, never labels. `assets/rules/adv-codes.toml` (code → label, from the GDI-DE registry, 121
  entries) only fills `freemapper:alkis` for the mapper, e.g. `Mauerkante, rechts`.
- Symbols follow ADR 0047, and height decides nothing: walls 1700 to 1703 → 515, retaining walls 1720 to 1723 →
  513.2, fences 1740 → 518, monuments 1750, 1760 to 1763, 1770 → 526, fountains 1780 to 1783 → 311, hedges 1100
  to 1103 → 410.1, single trees 1011/1012 → 418, embankment top 1220 → 104 (sprint), bridges 1800 to 1808 and
  1820 → 501.5 (sprint), 2136 → 515, ditches 1010/1013 → 306, brooks 1030 → 305, a seasonal brook (2000/3000)
  → 306. Both edges of a wall stay.
- Every area rule names `alkis:geom=area`; a closed line with a bridge code gives nothing. Water lines need
  `alkis:geom=line`. A NAS area for a point rule gives a point on its surface (`PointOn::Surface`), not the
  centroid.
- `build/dedup.rs` holds the dedup for every state. `Rank` comes from `freemapper:source`: register (the Berlin
  registers and tree layers, `trees/*`) > ALKIS (`alkis`, `alkis-nas/*`, the other `geoportal-berlin/*`) > survey
  (`geoportal-berlin/sb-*`) > OSM. Distances and shares stay those of ADRs 0047, 0051 and 0055.
  - A cadastre or OSM line (515, 513.2, 518, 410.1, 305, 306) with 80 % of its length within 1.5 m of a
    higher-ranked line with the same symbol is dropped and named in `freemapper:also`.
  - A 417/418 point within 2 m of a higher-ranked tree is dropped, unnamed as in ADR 0051.
  - A NAS fountain (311) beats OSM `amenity=fountain` and `man_made=water_well` within 3 m, named.
- Berlin keeps its order and its tags; a characterisation test (`berlin_dedup_unchanged`) holds the old result.

## Consequences

- A new state with NAS needs a service row and maybe rules, no new code; a new code is one TOML line.
- Berlin ALKIS and NAS have the same rank, so an object in both cadastres would stay twice. The two cadastres do
  not overlap.
- Cost if wrong: a code list change by the AdV makes a rule match nothing until the TOML follows.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Task 3); branch `states-plus-1`,
Task 3, its review and fix round.
