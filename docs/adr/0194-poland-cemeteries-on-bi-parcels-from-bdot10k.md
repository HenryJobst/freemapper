# ADR 0194: Cemeteries on `Bi` parcels from BDOT10k

## Status

Accepted, 24 November 2026. Extends [ADR 0192](0192-poland-recreation-land-bz-and-other-built-up-bi.md), which left
"a source that splits `Bi`" open, and uses the range reads of [ADR 0187](0187-poland-bdot10k-roads-and-paths-on-forest-maps.md).

## Context

The EGiB class `Bi` (other built-up land) holds schools, offices, shops, churches and active cemeteries (ADR 0192),
so the map draws nothing on it. BDOT10k has classes of its own for several of these uses. Measured on 24 November
2026 on the archives of powiat 0805 (słubicki, Słubice, 53.9 MB) and 3064 (Poznań, 36.5 MB), against the live EGiB
parcels: 1919 parcels in a 3 km box at the Słubice cemeteries, 23 191 parcels in a 5 km box at Poznań. Each BDOT10k
polygon was sampled on a grid and the EGiB class under every sample point counted.

| BDOT10k class (`rodzaj`) | Słubice / Poznań objects | EGiB class under the samples |
|---|---|---|
| `OT_PTZB_A` built-up land: `pozostała zabudowa`, `handlowo-usługowa` | 616 and 130 / 1850 and 638 | does not follow `Bi`: `handlowo-usługowa` lies on `Bi` 687, `B` 449, `dr` 243, `Bp` 222 samples, `pozostała zabudowa` on `Bi` 1188, `B` 481, `dr` 371, `Bp` 154, `jednorodzinna` on `B` 1344 but also on `Bi` 233. Drawn from the orthophoto; it gives no school, office or shop (Słubice, 3.2 km box) |
| `OT_KUOS_A` education: `szkoła lub zespół szkół`, `przedszkole`, `szkoła wyższa`, `ośrodek naukowo-badawczy` | 28 / 279 | `Bi` in most samples: Słubice 157 of 223; Poznań `szkoła` 1623 of 1987, `przedszkole` 424 of 609, `szkoła wyższa` 477 of 643 (the rest `dr`, `B`, `Bi,Bz`, no parcel) |
| `OT_KUOZ_A` health: `zespół szpitalny lub sanatoryjny`, `zakład opieki socjalnej`, `żłobek` | 4 / 32 | `Bi` (Poznań `zespół szpitalny` 376 of 402) |
| `OT_KUHU_A` trade and services | 3 / 8 | no `rodzaj` at all; `B` 27 of 29 samples at Słubice, `Bi` 14 of 19 in the second box |
| `OT_KUSC_A`: `cmentarz komunalny`, `cmentarz wyznaniowy`, `cmentarz wojenny` | 38 / 24 | Poznań `cmentarz komunalny`: `Bi` 47, `Bz` 13, `dr` 2 samples. Słubice: `Bi` 17, `Bz` 25; `cmentarz wyznaniowy`: `Lz` 18 (an inactive cemetery counted as trees, ADR 0192), `B` 12 |
| `OT_KUSC_A` `zespół sakralny lub klasztorny` | 3 / 58 | `Bi` (church grounds, not a cemetery) |

So BDOT10k does separate the uses, but only one of them has a map object that fits: the cemetery. Schools,
kindergartens, hospitals and social homes are found reliably (KUOS, KUOZ), but ADR 0192 names no land use for them
that is right for most of them, and a 520 on a school would close the schoolyard map's own ground. Shops and offices
have no class (`KUHU_A` is a handful of objects without a kind; `PTZB_A` does not follow the register).

The cemetery file is small. `OT_KUSC_A` is 7.6 KB packed at Słubice (52 KB as GML, 41 objects) and 23.2 KB at Poznań
(127 KB, 82 objects); the archive directory is 7.8 KB for 75 members.

## Decision

- **A cemetery of BDOT10k becomes `Friedhof`** (land use of the map, a lawn candidate 401 like the German cemeteries)
  where **no parcel of EGiB with a land use of its own lies under its representative point**. That is `Bi` (also `Bp`,
  `Ti` and mixed classes such as `Bi,Bz`), exactly the parcels ADR 0192 left unmapped. A cemetery on `Bz`, `Lz`, `B`
  or any class with a land use stays with EGiB; the rule is that of the fallback (ADR 0178: EGiB comes first).
- **Which objects.** `cmentarz komunalny`, `cmentarz wyznaniowy` and `cmentarz wojenny` (the mapping of ADR 0178,
  `land_use_of`). `zespół sakralny lub klasztorny` (church and monastery grounds) is not a cemetery and stays out.
- **Schools, health, trade, offices and the rest of `Bi` stay unmapped.** The classes are measured above and not used:
  nothing in the data says what is right for the map (open to the person, private, or a lawn).
- **Read by byte ranges, not the archive.** `bdot10k::cemeteries` asks the index and reads the member
  `OT_KUSC_A.xml` of each powiat archive as ADR 0187 does (central directory, then the member). It needs no cache
  directory. Per map: the index (1.5 KB), the directory (7.8 KB) and the member (7.6 KB at Słubice, 23.2 KB at
  Poznań), so about 17 and 32 KB, against 53.9 and 36.5 MB for the archive that the fallback (ADR 0178) downloads
  where EGiB has no land use. The map already asks the same two hosts.
- **When it runs.** Where EGiB gives a land use (not thin, `land_use_is_thin`). Where EGiB is thin the fallback has
  read the whole archive, cemeteries included, and nothing more is asked. Offline nothing is asked.
- **Failure is a warning.** An unreachable index or a broken member gives the warning of the fallback (`BDOT10k could
  not fill the gap in EGiB (cemeteries)`) and the land use stays that of EGiB. An archive without the member, or a
  box in no powiat, gives no object and no note.
- Adapter revision 12. The adapter asks no new host.

## Consequences

- A cemetery on a `Bi` parcel is a 401 candidate in the vegetation candidate layer, like the German ones (the map
  never draws it, the person decides). Where the cemetery lies on `Bz` or `Lz` nothing changes.
- A run in Poland asks the index and two ranges more per map, a few ten KB.
- Not measured: other powiats; the share of Polish cemeteries on `Bi` beyond the two measured (Poznań komunalny 76 %
  of the samples, Słubice komunalny 40 %).
- The measurement is in [poland-reference.md](../regions/poland-reference.md).
