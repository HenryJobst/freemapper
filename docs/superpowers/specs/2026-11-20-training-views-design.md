# Training views: leave out parts of a finished map — design

Date: 2026-11-20. Request of the user (2026-11-19): training forms such as Ultimate maps (forest, no paths),
contour orienteering (only brown: contours, depressions, pits) and maps without north lines. Agreed in chat the
same day: one shared selection in freemapper-core, the files in freemapper, the option in freemapper-print.
ADR 0160.

## Goal

A trainer has a finished training map and wants the same map with a class of symbols left out. The map is not
generated again: the variant is a selection from the objects of the finished map.

## The views

A view is a named rule on a symbol (its code and its colours). Three presets and a free list:

| Name | Keeps | Leaves out |
|---|---|---|
| `no-paths` | everything else | roads, paths and stairs: 502 to 507, 501.6 to 501.8, 532 |
| `brown-only` | symbols drawn in brown alone (contours, form lines, banks, knolls, depressions, pits), and the frame: the information block, title, scale, credit lines | everything else, the north lines too |
| `no-north-lines` | everything else | the north lines (601) |

- `--view <name>` takes a preset or a list `no:<code>,<code>…` (leave out) or `only:<code>,<code>…` (keep only,
  plus the frame). Several views combine: an object stays when every view keeps it.
- "Brown alone" is read from the symbol set, not from a list of codes: a symbol belongs to `brown-only` when all
  its colours are brown colours of the set. That keeps it right for ISOM and ISSprOM.
- The frame is the objects of the map part "peripheral information" and the north lines where the view does not
  name them. A view never removes the credit lines (ATTRIBUTION rule of the suite).
- Templates are not objects of the map: a view leaves them as they are. A view of the training map has none
  (the training map lists the raster templates only).

## Where it lives

1. **freemapper-core**, `freemapper-omap`: the module `view` with `View` (the rule), the presets, the parser of the
   `--view` value, and `keeps(view, code, colours) -> bool`. It works on codes and colours, so both object models
   (core's and freemapper's own) use it. Nothing there writes a file.
2. **freemapper**: `export --training --view <name>` (repeatable) writes `<name>-training-<view>.omap` and `.pdf`
   beside the full training map; with `--ocad` also `ocad/<name>-training-<view>.ocd`. The MCP tool `start_pipeline`
   gets `training_views` (list of names). The variant is cut from the objects of the training map before they are
   written; nothing is derived again. `run` keeps an existing variant like the training map itself.
3. **freemapper-print**: `--view <name>` on any `.omap`: the view applies to the objects in memory before the sheet
   is rendered. It also works on maps the user edited and on maps not made by freemapper.

## Anti-goals

- No second training pipeline: a view never changes how the map was cleaned (a tree that moved off a path stays
  where it is when the paths go).
- No change of the full training map or the base map.
- No views that need data (no "only where the nDSM shows trees").

## Tests

- core: each preset on a small set of objects with known symbols (ISOM and ISSprOM); the parser (names, lists,
  wrong code); combination; the frame and credit lines always stay.
- freemapper: `export --training --view brown-only` on a fixture writes the variant with only brown symbols and
  the frame; the full training map is unchanged; the files of a second run are kept.
- print: the PDF of a view has the same sheet and frame as the full map.

## Left to the builder

The exact code list of `no-paths` per symbol set (read it from the sets, check that 501.x paved areas that are
areas, not lines, stay), the file name rule when several views combine (`<name>-training-no-paths+no-north-lines`),
and the wording of the program texts (DE/EN).
