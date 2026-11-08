# 0043. Magnetic north lines are map content

Date: 2026-11-08
Status: Accepted

## Context

The north lines (ISSprOM 601) sat in the map part "peripheral information" with title, scale bar and attribution,
tagged `freemapper:source=sheet`. The user's rule, passed on by the freemapper-print session, is that they belong
to the map: the mapper breaks them at small features in OOM (ISSprOM 2019-2 §4.6), and the colour priority puts
them under walls and fences. The training PDF painted the sheet after the map, so a north line lay on a black wall.

## Decision

- The north lines are the first objects of the map part "default part", tagged `freemapper:source=north-lines`,
  symbol 601. The info block (991 to 994) stays in "peripheral information" with `freemapper:source=sheet`.
- Spacing: 30 mm on sprint maps (§4.6), 30 mm × 1.25 = 37.5 mm (37.5 m) on schoolyard maps, because §3.1
  enlarges all lines and symbols of an enlarged map in proportion. No source at hand (ISSprOM 2019-2,
  `school-2019-additions.xml`, the design spec) gives a school value, so 37.5 mm stays until one does.

## Consequences

- The mapper cuts north lines in OOM with the usual tools; hiding "peripheral information" no longer hides them.
- The training PDF clips them to the map content and paints them first within the black, under walls and fences.
- The OCAD file has no parts and is unchanged. An existing map changes only with `run --force`.
- Cost if wrong: a sheet layout that wants them outside the map content moves them back and changes the tag again.

Source: user rule via the freemapper-print session, 2026-11-08; ISSprOM 2019-2 §3.1, §4.6; branch `sheet-colours`.
