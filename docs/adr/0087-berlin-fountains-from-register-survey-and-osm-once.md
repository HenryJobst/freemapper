# 0087. Berlin fountains from register, survey and OSM once

Date: 2026-11-10
Status: Accepted

## Context

Fountains, wells and drinking fountains (311, 313) come in Berlin from four sources: the fountain registers
(`zierbrunnen`, `trinkwasserbrunnen`), the street survey (`af_wasserpumpen_brunnen`, `bv_springbrunnen_zierbrunnen`,
`ac_trinkwasserbrunnen_wasserspender`), the full ALKIS and OSM (`amenity=fountain`, `man_made=water_well`,
`amenity=drinking_water`). ADR 0055 let a register point drop the survey and OSM points near it, and the full ALKIS
drop OSM. The survey had no rule against OSM, and a register point beside an OSM fountain basin was not compared with
the basin. The earlier review left this open as "survey fountains vs OSM".

## Measurement (Viktoriapark, 2026-11-10)

The sources give 4 survey pumps, 3 register points (all Zierbrunnen, the drinking-water register is empty here) and
8 OSM points (6 wells, 2 springs). OSM has no `amenity=fountain` and no `drinking_water` node there.

| Pair | Distance |
| --- | --- |
| survey pump `Pk_25.09.2014_3` and OSM well `node/1313558844` | 0.05 m |
| survey pump `Pk_29.04.2015_4` and OSM well `node/6005240121` | 0.44 m |
| survey pump `Pk_29.04.2015_6` and OSM well `node/1154543493` | 0.49 m |
| survey pump `Pk_29.04.2015_7` and OSM well `node/966049521` | 9.77 m |

No register point lies within 100 m of a survey or OSM point, and no two points of one source are closer than 8 m.
Within 5 m lie the three close pairs, within 10 m the fourth joins them, within 20 m nothing more. The gap between
0.5 m and 9.8 m is wide. The crops show the three close pairs as one spot each; the fourth pair stands along the same
street, 9.8 m apart, and could be one pump or two.

Before this change the three close pairs already disappeared from the candidates, but through the point
generalisation (the 1.05 mm gap between two 311 symbols, `freemapper:generalized`), not through the dedup, and only
because the symbols touched. The fourth pair vanished under a large tree (417 beats 311 in the generalisation); that is
a separate question and not changed here.

## Decision

- **Limit.** `FOUNTAIN_DEDUP` is 0.75 mm on paper (3 m at 1:4000, 3.75 m at 1:5000), never under 3 m on the ground
  (1:1000). It lies in the gap of the measurement and under the 1.05 mm at which the symbols touch. The ground floor
  stays because source positions differ by metres at any scale. Other point rules keep their ground metres.
- **Order of winners.** Register, then full ALKIS, then survey, then OSM. New entries: the survey pump
  (`af_wasserpumpen_brunnen`), the survey jet (`bv_springbrunnen_zierbrunnen`) and the survey drinking fountain
  (`ac_trinkwasserbrunnen_wasserspender`) beat OSM `man_made=water_well`, `amenity=fountain` and
  `amenity=drinking_water`. The Zierbrunnen register beats an OSM well as well as an OSM fountain. A point dropped by
  an earlier entry wins nothing later. The winner names the dropped one in `freemapper:also` (`osm/node/1313558844`).
- **Fountain basin.** `drop_fountains_in_osm_basins` drops a cadastre, register or survey fountain (not a pump) that
  lies in, or within 0.25 mm on paper (1 m at 1:4000, `AREA_DEDUP_TOLERANCE_MM`) of, an OSM 301 area from
  `amenity=fountain`. The basin names the dropped point in `freemapper:also`. Before, only ALKIS water and OSM points
  were compared with such a basin, so a register jet stayed beside the polygon that already shows the fountain.
  Other OSM water does not count: the stream channel of the Viktoriapark waterfall (`natural=water`, `water=stream`)
  lies 1.0 m from the register point `WA-FK-5`, which is the fountain and stays. ALKIS water kept its rule
  (`drop_points_in_water`, ADR 0036 and 0055).
- **Kept on purpose.** Two fountains closer than the limit but without a higher source stay (two OSM wells 3 m apart).
  A well 3.2 m from a pump stays at 1:4000 (0.8 mm).

## Consequences

- Viktoriapark candidates: 311 stays at 9 (3 register, 4 survey, 2 OSM), 313 at 0. The result looks the same; the three
  survey pumps now name the OSM wells in `freemapper:also` instead of `freemapper:generalized`, and the dedup no
  longer depends on the symbol size.
- No pair of 311 or 313 points of two sources is left within the limit. The basin rule found no case in the park (no
  OSM fountain polygon); it is covered by tests with a synthetic basin.
- The measured coordinates are test fixtures (`survey_pump_beats_the_osm_well_at_the_measured_distances`).
- Existing maps get this with `run --force`.
