# 0126. Cliffs on the forest map are generalized

Date: 2026-11-14
Status: Accepted

## Context

OpenStreetMap maps rock country cliff by cliff. At the Bastei in Saxon Switzerland the forest example drew 2144
`natural=cliff` ways as impassable cliffs (201), up to 127 on one square centimetre of paper, often several along
one face; the rocks became a black mass. The other forest examples have at most 39 cliffs. A spike on the Bastei
data (scratch, Python) showed that the rules of the norm make the map readable; the user chose to build it.

ISOM 2017-2 gives for 201: a gap of more than 0.25 mm to another impassable cliff or impassable feature, a minimum
length of 0.6 mm; for 206 (gigantic boulder, rock pillar or massive cliff in plan shape): a gap of 0.15 mm, a
minimum width of 0.25 mm and a minimum area of 0.3 mm². The bundled 201 draws a line of 0.35 mm with tags reaching
0.575 mm from its centre.

## Decision

- A new step `generalize_cliffs` (`build/cliffs.rs`) runs in `Built::place` after `route`, on the 201 objects the
  content settings put on the map of a forest map. Sprint and schoolyard maps keep their cliffs.
- Height: the median of the height difference across the cliff, 2.5 m on each side, every 2 m, from the terrain
  model of the project. Ruling: below 3 m a cliff is no impassable cliff at forest scale. Without a terrain model
  every cliff counts as tall.
- A closed tall cliff whose ring encloses 0.3 to 10 mm² (ruling for the upper bound: larger rings are plateaus or
  massifs) and is at least 0.25 mm wide becomes a 206 area; the cliff goes.
- The other tall cliffs are smoothed and then simplified by 0.1 mm like the contours: OSM traces every niche of
  a rock face, and a bend shorter than about the distance of two tags (0.5 mm) reads as zigzag (the user asked for
  it after the first build). Ruling: a Gaussian along the line, resampled every metre, with a sigma of 0.3 mm (4.5 m
  at 1:10 000), chosen from 0, 3, 5 and 8 m on the Bastei; an open line keeps its ends. A pillar keeps its unsmoothed
  outline, because smoothing shrinks a ring (at 5 m, 23 of 39 pillars fell below the minimum area).
- They are kept by weight, height times length. A
  cliff is not kept when it is shorter than 0.6 mm, when its line lies closer than 1.0 mm to a kept cliff (the gap
  of 0.25 mm between the line of one, 0.175 mm, and the tags of the other, 0.575 mm) or closer than 0.725 mm to a
  pillar (gap and tags). All measures are symbol measures, enlarged like the symbols (ADR 0118).
- A cliff that is not kept goes to the candidates with `freemapper:hidden=generalized` (the training PDF leaves it
  out, as for other hidden candidates) and `freemapper:generalized` naming the reason.
- The contours, map and candidates, stop around every tall cliff (as far as line and tags reach) and around the
  pillars, also where the cliff itself was not kept: there is still a rock face there.
- `derive` keeps the terrain mosaic of a tile provider as `data/terrain.tif` for every forest map (before only
  for Berlin stairs), and the export of a forest map reads it when there is no `data/dgm1.tif`. Other maps keep
  reading only `data/dgm1.tif`, so the ticks of their banks stay as they were.

- Tried and dropped: a height threshold that rises with the relief around the cliff, as Karttapullautin does
  (`cliffsteepfactor`; the user asked for it). Relief is highest minus lowest within 3 m of the line; the threshold
  grew by the factor for every metre above 5.5 m. At Karttapullautin's 0.76 the Bastei lost 34 of 39 pillars and,
  with fewer cliffs to stop them, its contours piled up again. At 0.38, pillars exempt and the contours still
  stopped at every cliff of 3 m: 15 fewer cliffs at the Bastei, hardly visible, because the gap rule had taken those
  places already; in the Siebengebirge it removed the long rock edge of the Drachenfels, a prominent face in steep
  ground; in Dahn nothing changed. Karttapullautin needs the rule because it has no gap rule; here the gap rule does
  the job.

## Consequences

- The Bastei keeps 197 cliffs and gets 39 pillars; 1908 cliffs go to the candidates (917 too close, 397 too short,
  320 lower than 3 m, 274 beside a pillar).
- A forest project of a tile provider stores a few MB more (`data/terrain.tif`, about 6 MB for the Bastei).
- Large rock massifs are thinned, not drawn as areas; that would need their outline, which OSM does not give.
