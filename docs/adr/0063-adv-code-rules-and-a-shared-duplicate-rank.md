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
  → 306. Both edges of a wall stayed; the amendment below pairs them.
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

## Amendment of 2026-11-09: wall edges become one centre line

The live run at Park Sanssouci gave 271 lines 515 for about 150 walls. Two 515 lines closer than the line width
plus the gap (0.4 mm + 0.15 mm) print as one heavy line, and the mapper had to delete one of each pair.

- `build/walls.rs` (`pair_wall_edges`) pairs the edges of the NAS walls: 1701 and 1702 lines, and the rings of a
  1700 area. It runs in `nas_objects` on the stored features, before `tagged_features` and the join of short
  segments, because only the stored feature says on which side the wall lies. The join may turn a piece round.
- Side: 1701 (Mauerkante, rechts) has the wall on the left of its direction, 1702 on the right. Measured on the
  Sanssouci data: for 1701 388 m of the near partners lie on the left and 48 m on the right, for 1702 268 m on
  the right and 14 m on the left. A 1700 ring has the wall inside.
- Two edges pair where they lie closer than 0.55 mm on paper (`WALL_PAIR_MM` × the symbol enlargement; 2.2 m at
  1:4000, 0.69 m on the schoolyard at 1:1000), run at most 20° apart, lie each on the wall side of the other,
  and pair with each other mutually. Two walls back to back across a narrow path face away from each other and
  stay apart, so the gap stays passable on the map.
- The centre line runs only where both edges overlap: each point of an edge is averaged with the nearest point
  of its partner, and a point whose foot falls beyond the partner's free end has none. Remnants stay as edges,
  so a gate cut in one edge is not bridged; the 1 mm limit then drops remnants shorter than 4 m at 1:4000.
  Corner ends of two centre lines are snapped to their crossing (each moves at most half the pair width).
  Concentric rings give one closed centre ring, a thin 1700 area its centre line. A ring that would pair with
  itself but whose bounding box stays below the 1 mm limit (the 2 m pedestals on the Sanssouci terraces) stays
  as stored: its centre would be a point.
- A stored centre line (1700 line, 1703 Mauermitte) stays, and the edges within half the pair distance on its
  wall side are dropped and named in its `freemapper:also`.
- Tags: a 1700 with `freemapper:alkis=Mauer (Mauerkanten gepaart)`, `freemapper:source_id` of the edge that
  draws it, the other edges in `freemapper:also`, the older `beginnt` as currency. The OSM dedup (1.5 m) then
  runs on the centre line.
- NAS only. The Berlin ALKIS wall edges keep ADR 0047, so the Berlin output stays the same. Retaining walls
  (513.2, ticks on one side) are never paired. Hedge edges (1101, 1102) have the same shape and can reuse the
  code once a state serves them.
- The join of short segments (`join_touching_lines`) no longer turns a line with a side round and never chains
  two different sides: NAS edges 1701/1702, 1721/1722, 1101/1102, and the symbols 104, 201 and 513.2 join only
  as same-way continuations of the same code. Lines without a side join as before. This leaves two more 104
  lines in the Viktoriapark candidates and one more in Sanssouci, where pieces ran opposite ways.

Consequences: Sanssouci has 264 lines 515 instead of 271, 14 of them centre lines; the length of 515 lines with
another nearly parallel one within 2.2 m fell from 876 m to 283 m. What stays close are mostly edges that face
the same way (a terrace edge beside a wall), which the data does not pair. Cost if wrong: a wall drawn once
where two walls stand, or a gate the centre line hides; the edges are named in `freemapper:also`.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Task 3); branch `states-plus-1`,
Task 3, its review and fix round; final review of the branch (ruling on wall edges) and its fix pass.
