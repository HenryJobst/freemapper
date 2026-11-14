# 0124. A steep forest map gets contours every 10 m

Date: 2026-11-14
Status: Accepted

## Context

A forest map traces its contours every 5 m and every 2.5 m in flat land (ADR 0116), as the ISOM 2017-2 asks.
In steep land 5 m contours run together. On the example Bodetal bei Thale the slopes of the Bode valley became
one brown area, at the Bastei as well. The user rebuilt both with `--contour-interval 10` (PR #98) and asked for
the same choice to be made automatically, and for it to be suppressible.

## Decision

- With `auto`, `derive` gives a forest map 10 m when contours of the kind's interval (5 m) would lie closer than
  0.35 mm, centre to centre, on more than a tenth of the map area. 0.35 mm is a symbol measure (ADR 0118): at
  1:10 000 it is 5.25 m on the ground, so 5 m contours crowd from a slope of 43.6°. Two contours of 0.14 mm that
  close leave less than their own width of paper between them.
- The slope is taken on the smoothed DTM the contours are traced from, at the cells already sampled for the
  flatness test (every 5 m), from the two neighbours on each axis.
- The flatness test comes first; a map area is either flat or steep.
- Measured on the forest examples, the share is 26 % in the Bode valley and 18 % at the Bastei; Cloef and
  Siebengebirge have 5 %, the other examples at most 1 %. A tenth lies between them.
- When it picks 10 m, `derive` says so in one line (`progress-contours-steep`) and names
  `--contour-interval 5`. A set interval always wins over `auto`, so that keeps 5 m; no further switch is needed.
- The 10 m is stored in `freemapper.toml` as `[contours] interval_m` like every interval that is not the kind's,
  and the margin names it.

## Consequences

- The ISOM 2017-2 knows 5 m and 2.5 m only. A steep forest map leaves the norm for legibility; the manual says so.
- Projects that kept `auto` and lie in steep land get 10 m at their next `derive`. Their map changes only with
  `export --force`.
- Sprint and schoolyard maps keep their interval on any slope.
