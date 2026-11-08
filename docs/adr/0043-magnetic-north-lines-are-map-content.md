# 0043. Magnetic north lines are map content

Date: 2026-11-08
Status: Accepted

## Context

freemapper put the magnetic north lines (ISSprOM 601) into the map part "peripheral information", next to title,
scale bar and attribution, all tagged `freemapper:source=sheet`. The user then set a rule, passed on by the
freemapper-print session: north lines belong to the map. The mapper breaks them in OOM where they would hide small
features (ISSprOM 2019-2 §4.6 names boulders, knolls, cliffs, stream junctions and path ends), and the colour
priority puts them under walls and fences. The training PDF painted the sheet part after the map within each colour,
so a north line lay on top of a black wall.

## Decision

- The north lines go into the map part "default part", before every other object, tagged
  `freemapper:source=north-lines`. The symbol stays 601.
- Title, scale and contour interval, bar scale and attribution (sheet symbols 991 to 994) stay in
  "peripheral information" with `freemapper:source=sheet`.
- The spacing stays 30 mm on sprint maps (§4.6: 30 mm, 120 m on the ground at 1:4000) and 30 mm × 1.25 = 37.5 mm on
  schoolyard maps, which is 37.5 m on the ground at 1:1000. §3.1 says that an enlarged map enlarges all lines and
  symbols in proportion, and freemapper enlarges the school set by 125 %. No source at hand gives a north line
  spacing for the school set: not the ISSprOM 2019-2 text, not freemapper's school additions
  (`assets/symbol-sets/school-2019-additions.xml`) and not the design spec. The 37.5 mm stay until a source says
  otherwise.

## Consequences

- In OOM the north lines sit in the same part as the buildings, so the mapper cuts them with the usual tools. Hiding
  the part "peripheral information" no longer hides them.
- The training PDF clips them to the map content like any map object. Within the black they come first, so walls
  and fences lie on top.
- The OCAD file has no parts and stays as it was.
- An existing map keeps its old parts until `run --force` writes it again.
- Cost if wrong: a later sheet layout that wants the north lines outside the map content would have to move them back
  and change the tag a second time.

Source: user rule, passed on by the freemapper-print session on 2026-11-08; ISSprOM 2019-2 §3.1 and §4.6; branch
`sheet-colours`.
