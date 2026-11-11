# 0098. Paved OSM ways are paved lines

Date: 2026-11-11
Status: Accepted, amends ADR 0079

## Context

OSM footways, paths and tracks with a paved surface (`asphalt`, `paving_stones`, `concrete`, `sett` and the like)
had no rule: the ALKIS traffic areas and the orthophoto were meant to show them. In the Biesdorfer Park the
Berlin and ALKIS areas do not hold the park paths, so a path was missing: a footway ended at 52.5140 N and the
next began at 52.5133 N, 81 m apart, because the way between them is paved. ISSprOM 2019-2 §4.5 calls asphalt,
tiles and concrete firm surfaces of a paved area, which "should be bordered" with 501.1. The line symbols 501.6 to
501.9 carry that border (ADR 0079).

## Decision

- The rules of ADR 0079 for firm gravel (footways, paths, bridleways and tracks, no sidewalks, crossings, tunnels,
  covered, indoor or area ways; by `width` 501.6 to 501.9, a track without width 501.7) also take the paved
  surfaces `asphalt`, `paved`, `paving_stones`, `concrete`, `sett`, `cobblestone`, `bricks`, `chipseal`,
  `concrete:plates` and `concrete:lanes`. They are map objects.
- `build/firm_paths.rs` cuts them where ALKIS or survey paving lies, so a paved area outranks a way: the access to
  a castle that is a paved area is the area, the paths beside it are lines.
- Streets (`service`, `residential` and so on) and sidewalks keep no rule.

## Consequences

- Park and estate paths of paved surface appear with the black border, also on the walkable base. The Biesdorfer
  Park gets 134 more OSM features. Real projects change wherever OSM knows paved paths. The golden areas (150 m) hold none, so G4 stays as it is.
- A paved area without the 501.1 border (the castle forecourt) stays as it is. Bordering paved areas is another
  decision.
- Cost if wrong: a paved way that the traffic areas already draw as an area is drawn twice where the cut misses
  it by more than the footprint.

Source: user decision 2026-11-11 after the Biesdorfer Park review; ISSprOM 2019-2 §4.5.
