# 0138. Vegetation: the green shades follow the density below 3 m

Date: 2026-11-14
Status: Accepted

Measurement: [`docs/research/vegetation-leaf-off/2026-11-14-thickets.md`](../research/vegetation-leaf-off/2026-11-14-thickets.md).
It changes one rule of [0105](0105-vegetation-from-point-clouds.md) and leaves
[0136](0136-vegetation-outlines-traced-on-a-blurred-raster.md) (the outlines) as it is.

## Context

Thickets came out as runnable forest (405). The Bavarian flights are winter flights, and the first idea was to
lower the thresholds for them. The user asked for a measurement first. It ran over ten example areas in the six
states with point clouds and found three things.

- Every flight was flown without leaves, between 20 December and 10 March. This is no matter of Bavaria, Berlin
  and Saarland; there is no flight in leaf to tune against.
- The green index of 0105 counted a return above 3 m with 0.1. In tall forest 58 % of the returns lie above 8 m,
  which alone gives an index of about 0.2 with nothing growing below the trees. A leafless thicket reaches about
  0.3. A lower threshold catches both: at 0.25 instead of 0.40, 30 % of the tall forest on the Schmausenbuck turn
  green (11 % before).
- The layer a runner moves through tells them apart. In a thicket 35 % of the returns lie between 0.8 and 3 m, in
  tall forest 3 %.

## Decision

- The green shades follow the **density** of that layer: the returns between 0.8 and 3 m among all returns below
  3 m (ground, undergrowth and the layer itself). Returns above 3 m do not count. From 0.20 a cell is slow
  running (406), from 0.35 walk (408), from 0.50 impassable (410).
- The rest of 0105 stays: open land at 90 % of the returns below 0.9 m, undergrowth (407, 409) by the share of
  the returns between 0.2 and 0.8 m where the density stays below 0.20, roofs by their single echoes, no class
  below four returns, the majority filter, the minimum area.
- One rule for every state. No flight is in leaf, so a rule by season has nothing to switch on.
- The user chose 0.20 over 0.25. There was no forest known from the terrain to check either against.

## Consequences

- On the Schmausenbuck 71 % of the thicket cells of the reference are green (29 % before) and 7 % of the tall
  forest (11 % before). The pine forest of the Rauener Berge stays white (0.2 % green, as before).
- A map with spruce or scrub gets more green: Monschau 55 % of the cells with vegetation (43 % before), Cochem
  57 % (18 % before), the Dammheide 16 % (4 % before). These shares are counted on the cells; the smallest area of
  a map removes part of them.
- The nine examples with vegetation are rebuilt; their map objects stay, the candidates change. The sprint
  examples get more green areas (406, 408, 410): Sanssouci 205 (54 before), Monschau 638 (218), Cochem 404 (150),
  Saarlouis 125 (69), the Viktoriapark 107 (38). The forest examples change little, because their smallest area
  is larger: the Schmausenbuck has 8 green areas (16 before), the Siebengebirge 29 (7), the Cloef 14 (6), the
  Rauener Berge 1 (none).
- The reference is a summer canopy height from aerial pictures, 711 cells on the Schmausenbuck. It is no survey
  in the terrain.

## Left out

- Pole stands, with crowns at 3 to 8 m and a clear stem space, stay runnable forest. They need another value,
  the number of stems for example.
- A smallest number of returns below 3 m. Below a closed canopy few returns reach the layer, and a density from
  three returns is noise; the majority filter removes single cells, a threshold could remove more.
- The Saarland cloud, thinned to one point per 0.5 m, gives scattered patches under every rule tried.
- The undergrowth classes 407 and 409 were not measured.
