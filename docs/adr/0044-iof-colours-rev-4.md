# 0044. Colours after IOF Printing and Colour Definitions Rev. 4, not OOM's bundled values

Date: 2026-11-08
Status: Accepted

## Context

The bundled `ISSprOM 2019_4000.omap` (OOM `eae21b0`, 2025-05-18) has the old IOF colours, e.g. brown 0/56/100/18,
green 76/0/91/0, and fills buildings with black 60 %. IOF "Printing and Colour Definitions" Rev. 4 (September 2024,
valid from 1 December 2024) sets brown 25/75/100/0, brown 50 % 10/35/50/0, brown 30 % 6/23/33/0, green 80/0/100/0,
drops spot colour (PMS) printing and recommends CMYK. ISSprOM 2019-2 (2022) changed 521 Building to black 50 %. The
PDF renderer and the OCAD writer read the set's CMYK, so the old values reached every output.

## Decision

- Precedence: ISSprOM 2019-2 decides which tint a symbol uses, Rev. 4 decides the CMYK of each colour.
- The six spot colours take their Rev. 4 rows. A tint whose Rev. 4 row is not spot colour × factor (Purple 50%,
  Lower brown 50% and 30%, olive green Green 50%/Yellow 100%, Yellow 50%) gets the row's CMYK with
  `cmyk method="custom"` and RGB from CMYK. The other tints stay derived; their stored values are what OOM computes
  on load. Tests pin some values and recompute every derived one.
- The building fills 521.3 and 521.1 use "Black 50%" instead of "Black 60%". Bare rock (214) stays black 30 %, as
  ISSprOM says, although Rev. 4 lists a black 35 % row.
- Names, priorities, the other symbols and the `<spotcolors>` entries stay. OOM uses the spot colours for knockout,
  overprint simulation and separations, and Rev. 4 still describes CMYK+B; CMYK is what counts for printing.
- Rev. 4 lists dark green 100/0/80/30 for ISSprOM 411 (uncrossable vegetation, 2019-2). The OOM set has neither the
  colour nor 411, and freemapper does not emit 411, so neither is added.
- The school and sheet additions define no colours. `assets/symbol-sets/README.md` records the change and both hashes.

## Consequences

- PDF, OCAD and OOM files show the current IOF colours and lighter buildings.
- Replacing the set in OOM with OOM's own, or loading an older set in OCAD, brings the old colours back.
- An existing map keeps its colours until `run --force`. An update of the OOM set must keep these changes.
- Cost if wrong: browns a little redder than on older maps and on maps from OOM's own set.

Source: IOF Printing and Colour Definitions Rev. 4, chapters 1, 2, 6; ISSprOM 2019-2 symbols 214, 411, 521 and its
changelog (02.01.2022); OOM `xml_file_format.cpp`, `map_color.cpp`; freemapper-print session, 2026-11-08.
