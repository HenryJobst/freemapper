# Symbol sets

| File | Source | Licence |
|---|---|---|
| `ISSprOM 2019_4000.omap` | OpenOrienteering Mapper, `symbol sets/4000/ISSprOM 2019_4000.omap`, commit `eae21b05851495923ac245fdf4b89cf575df38ce` (2025-05-18), https://github.com/OpenOrienteering/mapper; colour table changed by freemapper (see below) | GPL-3.0-or-later (OpenOrienteering Mapper) |
| `school-2019-additions.xml` | freemapper, drawn after the IOF "Symbol Set for School Orienteering Maps 2019" | GPL-3.0-or-later |

The International Orienteering Federation (IOF) publishes the symbol specifications ISSprOM 2019-2 and the school
symbol set 2019 as well as the "Printing and Colour Definitions".

## Changes to `ISSprOM 2019_4000.omap`

freemapper changed only the CMYK and RGB values of the colour table (ADR 0044). The values follow the IOF
"Printing and Colour Definitions", Revision 4 (September 2024, valid from 1 December 2024), chapter 6:

- The six spot colours: Brown 100% is 25/75/100/0 (was 0/56/100/18), Green 100% is 80/0/100/0 (was 76/0/91/0).
  Purple 35/85/0/0, Black 0/0/0/100, Blue 100/0/0/0 and Yellow 0/27/79/0 stay as they were.
- Five tints whose Rev. 4 row is not the spot colour times the factor get these CMYK values with
  `<cmyk method="custom"/>`: Purple 50% 18/43/0/0, Lower brown 50% 10/35/50/0, Lower brown 30% 6/23/33/0,
  Green 50%/Yellow 100% (olive green) 38/27/100/0, Yellow 50% 0/14/40/0.
- All other tints keep `<cmyk method="spotcolor"/>`. Their stored values are what OOM computes from the new spot
  colours, for example Green 60% 48/0/60/0.
- RGB is computed from CMYK as OOM does it (`<rgb method="cmyk"/>`: r = (1 − c)(1 − k)), or from the spot
  colours where the colour keeps `<rgb method="spotcolor"/>`.
- Names, priorities, `<spotcolors>` entries and the symbols are unchanged. Black 30% for bare rock stays 30 %,
  as ISSprOM 2019-2 symbol 214 says, although Rev. 4 lists a black 35% row.

SHA-256 of the OOM original: `c278c36545e22114b7dd1146d311687ea9677eda63b8c10f8604d754679ea225`

SHA-256 of the changed file: `e78e517f4dd3537fc63287046ba6a1e92881212eee18bfad2689a865b95efd5e`
