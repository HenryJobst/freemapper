# 0044. Colours after IOF Printing and Colour Definitions Rev. 4, not OOM's bundled values

Date: 2026-11-08
Status: Accepted

## Context

The bundled `ISSprOM 2019_4000.omap` comes from OOM (commit `eae21b0`, 2025-05-18) and carries the old IOF colour
values, for example Brown 100% 0/56/100/18 and Green 100% 76/0/91/0. The IOF "Printing and Colour Definitions",
Revision 4 (September 2024, valid from 1 December 2024), changed brown to 25/75/100/0, Brown 50% to 10/35/50/0 and
Brown 30% to 6/23/33/0. Its chapter 6 lists green as 80/0/100/0. Rev. 4 also dropped spot colour (PMS) printing and
recommends CMYK. Sprint maps are laser printed anyway. The PDF renderer and the OCAD writer both read the CMYK of the
set's colour table, so the old values ended up in every PDF and OCAD file.

## Decision

- freemapper changes the CMYK values of the bundled colour table to Rev. 4, chapter 6. The six spot colours take
  their Rev. 4 rows. A tint whose Rev. 4 row is not the spot colour times the factor (Purple 50%, Lower brown 50%,
  Lower brown 30%, the olive green Green 50%/Yellow 100%, Yellow 50%) gets the row's CMYK with
  `<cmyk method="custom"/>`. All other tints stay derived from the spot colours, and their stored values are what
  OOM computes on load.
- RGB is derived from CMYK the way OOM does it (`rgb method="cmyk"`), and from the spot colours where a tint keeps
  `rgb method="spotcolor"`.
- Names, priorities and symbols stay as OOM ships them. The `<spotcolors>` entries also stay. OOM uses them for
  knockout, for its overprint simulation and for spot colour separations. Rev. 4 still describes CMYK+B, offset
  printing with brown as a spot colour. The CMYK values are now what counts for printing, and the spot colour names
  only label the separations.
- Black 30% for bare rock keeps 30 %, because ISSprOM 2019-2 symbol 214 says so. Rev. 4 lists a black 35% row
  for ISSprOM.
- `school-2019-additions.xml` and `sheet-additions.xml` define no colours; the school set uses the same table.
- `assets/symbol-sets/README.md` records the change and both SHA-256 sums. Tests pin some Rev. 4 values and check
  that every stored value is what OOM computes from the table.

## Consequences

- PDF training maps and OCAD files print brown and green in the current IOF CMYK. In OOM the map shows the new
  values too.
- Replacing the symbol set in OOM with OOM's bundled ISSprOM, or loading an older set into the OCAD file, brings the
  old colours back. OCAD's own sets bring their colours anyway.
- An existing map keeps its colour table until `run --force` writes it again.
- An update of the OOM set must keep this colour table, or the tests fail.
- Cost if wrong: the browns look a little redder than on older maps and on maps from OOM's own set.

Source: IOF Map Specifications, Printing and Colour Definitions, Revision 4 (September 2024), chapters 1, 2 and 6;
ISSprOM 2019-2 symbol 214; OOM `src/fileformats/xml_file_format.cpp` and `src/core/map_color.cpp` (how OOM derives
tints on load); colour values passed on by the freemapper-print session on 2026-11-08; branch `sheet-colours`.
