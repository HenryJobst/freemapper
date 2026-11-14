# 0126. Cliffs on the forest map are generalized

Date: 2026-11-14
Status: Accepted

## Context

OpenStreetMap maps rock country cliff by cliff. At the Bastei in Saxon Switzerland the forest example drew 2144
`natural=cliff` ways as impassable cliffs (201), up to 127 on one square centimetre of paper, often several along
one face and one face in several ways; the rocks became a black mass. The other forest examples have at most 39
cliffs. A spike on the Bastei data (scratch, Python) showed that the rules of the norm make the map readable; the
user chose to build it. After a first version two reviews (Opus 5.5 in the session "freemapper", Fable 5.2 in
freemapper-d3, who measured on the Bastei data) found the selection dropping the tallest faces, the contours losing
71 % of their length, the height of a way judged as a whole, and terrain knolls on every rock top; the user asked
for all of it to be addressed in this change.

ISOM 2017-2 gives for 201: a gap of more than 0.25 mm to any other impassable feature, a minimum length of 0.6 mm,
and tags that may be omitted where space is short (the set has the top line 201.3); for 206 (gigantic boulder,
rock pillar or massive cliff in plan shape): a gap of 0.15 mm, a minimum width of 0.25 mm and a minimum area of
0.3 mm². The bundled 201 draws a line of 0.35 mm with tags reaching 0.575 mm from its centre. What the terrain
model says (measured by freemapper-d3): the OSM lines lie a median 1.0 m from the nearest cell of 60°, OSM misses
only 33 of 443 faces, and on two thirds of the length top and foot of a face are 1 to 2 m apart, so two close
lines are mostly two walls (a fin or a cleft), not one face drawn twice.

## Decision

- A step `generalize_cliffs` (`build/cliffs.rs`) runs in `Built::place` after `route`, on the 201 objects the
  content settings put on the map of a forest map. Sprint and schoolyard maps keep their cliffs.
- Height and side from a cross profile every 2 m: heights every metre to 8 m on each side; the largest drop over
  5 m, less what the slope beside it (2 to 7 m out) drops over 5 m. That is how much the rock stands above its
  slope, and the side with the larger value is the lower one (undecided below 1 m difference). A line drawn up to
  about 2 m off the face still finds it; a steep slope without rock gives nothing. Ruling: below 3 m a cliff is no
  impassable cliff at forest scale.
- A way is cut into stretches where its class changes (tall or not, and the lower side), with runs shorter than
  0.6 mm taking the class of the longer neighbour, so that a way that runs from a face onto the slope above, or
  round a tower and across the ridge behind, is judged in parts. Each stretch is turned so that its tags point to
  the lower side; such a cliff carries `freemapper:ticks=profile`, and `orient_ticks` leaves it as it is.
- Stretches of one class that meet end to end (1 m, turning less than 60°) are joined: OSM splits faces into ways.
- A closed tall cliff with the lower side outside is a pillar; up to 10 mm² (ruling: larger rings are plateaus or
  massifs) it becomes 206, enlarged to 0.3 mm² and 0.25 mm wide where smaller. With the lower side inside it is a
  hollow and stays a cliff.
- Walls and pillars are ranked together by height² × length (ruling: height counts twice, so a high face goes
  before a long low terrace; without terrain the longest go first). A pillar too close to one kept before goes. A
  wall is tested every metre against what was kept, by the sides that face each other (line 0.175 mm or tags
  0.575 mm, gap 0.25 mm: back to back 0.6 mm, staggered 1.0 mm, facing 1.4 mm). Where it has room for line and
  tags it stays a 201; where only for its line it is a top line 201.3, kept from 2.0 mm (ruling: shorter top lines
  read as dashes); back to back closer than the gap, the strip between the two is a rock fin 206; elsewhere it is
  cut. Pieces with tags stay from 1.0 mm (ruling: at 0.6 mm a piece carries one or two tags and prints as a stub).
  The minimum length 0.6 mm of a whole wall is measured on the line as OSM drew it.
- The walls are smoothed: OSM traces every niche, and a bend shorter than about two tags (0.5 mm) reads as zigzag.
  Ruling: a Gaussian along the line, resampled every metre, sigma 0.3 mm (4.5 m at 1:10 000, chosen from 0, 3, 5
  and 8 m on the Bastei); an open line keeps its ends; then simplified by 0.1 mm. Pillars keep their unsmoothed
  outline, because smoothing shrinks a ring.
- What goes is moved to the candidates with `freemapper:hidden=generalized` (the training PDF leaves it out) and
  the reason in `freemapper:generalized`; a cut wall goes there whole.
- The contours stop below every cliff drawn, from its line to the end of its tags, and at pillars and fins;
  pieces shorter than a contour may be (2 mm) go. Below a wall that was not drawn they run on: the relief stays.
- The knolls and depressions of the terrain model within the reach of the tags of a tall cliff or pillar, drawn
  or not, are the tops and clefts of the rock: they are hidden (`freemapper:hidden=rock`).
- `derive` keeps the terrain mosaic of a tile provider as `data/terrain.tif` for every forest map, and the export
  of a forest map reads it when there is no `data/dgm1.tif`. Other maps keep reading only `data/dgm1.tif`.
- Legibility is a test: on the Bastei example no square centimetre of paper may be more than 40 % black with rock
  (201 with tags, 201.3, 206; `tests/rock_ink.rs`). Before: 230 %.

- Tried and dropped: a height threshold that rises with the relief around the cliff, as Karttapullautin does
  (`cliffsteepfactor`; the user asked for it). At Karttapullautin's 0.76 the Bastei lost 34 of 39 pillars and its
  contours piled up again; at 0.38 it removed 15 cliffs at the Bastei, hardly visible, and in the Siebengebirge the
  long rock edge of the Drachenfels; in Dahn nothing. Karttapullautin needs the rule because it has no gap rule.
- Tried and changed: top lines from 1.0 mm gave the Bastei 122 of them, mostly short strokes; from 2.0 mm, 23.

## Consequences

- The Bastei keeps BASTEI_WALLS walls with tags, BASTEI_TOP top lines and BASTEI_206 pillars and fins; its
  contours keep about half their length (35 of 78 km) instead of 22 km.
- A forest project of a tile provider stores a few MB more (`data/terrain.tif`, about 6 MB for the Bastei).
- Follow-ups (not in this change): large rock bodies from the terrain model as areas (freemapper-d3's main idea:
  rock = slope over 45° off the paths, opened and closed at the norm's radius; 97 pillar-size bodies and 11
  massifs at the Bastei), passages kept open and clefts closed, and 202 for walls between 2 and 3 m.
