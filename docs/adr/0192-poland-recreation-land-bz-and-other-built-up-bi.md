# ADR 0192: Recreation land (`Bz`) and other built-up land (`Bi`) in the Polish land use

## Status

Accepted, 24 November 2026. Extends [ADR 0171](0171-poland-cadastre-egib.md), which took the classes from the
ordinance without reading the annex.

## Context

ADR 0171 left `Bz` and `Bi` under "not checked". `Bz` gave the land use `Sport-, Freizeit- und Erholungsfläche` without
a `bez`, which the map ignores (only `Grünanlage`, `Park` and `Siedlungsgrünfläche` are lawn), so recreation parcels
were never drawn. `Bi` gave nothing. The sources, read on 24 November 2026 (Dz. U. 2021 poz. 1390, text and the
PDF of the annex at `https://api.sejm.gov.pl/eli/acts/DU/2021/1390/text.pdf`):

- § 9 ust. 3 names the classes (not § 68, which the ordinance does not have): `B` tereny mieszkaniowe, `Ba` tereny
  przemysłowe, `Bi` inne tereny zabudowane, `Bp` zurbanizowane tereny niezabudowane lub w trakcie zabudowy, `Bz` tereny
  rekreacyjno-wypoczynkowe. The annex (Załącznik nr 1, lp. 15 and 17) says what belongs to them.
- **`Bi`, lp. 15:** land under buildings that are not homes, industrial or storage buildings, with the structures and the
  yard that go with them, the land between and next to them that has no other use, and active cemeteries and
  inactive ones not counted as trees and shrubs, and animal burial grounds. Schools, offices, shops, churches, farm
  yards of non-farm buildings and cemeteries are all `Bi`.
- **`Bz`, lp. 17:** land not under buildings of: recreation centres, playgrounds, beaches, kept parks, squares and
  green strips (not those inside street lanes); historic sites (castle ruins, mounds, natural monuments); sports
  grounds (stadiums, pitches, ski jumps, shooting ranges, baths, golf courses); fairgrounds; zoological and botanical
  gardens; unkept green not counted as forest or trees and shrubs; family allotment gardens on land unfit for farming.
- Counts of the live service, Słubice, 24 November 2026 (first 1000 parcels in the box of the fixture): `Bi` 472,
  `B` 352, `Bp` 88, `dr` 47, `Bz` 4, `Bz,Lz` 2, `Bi,Bz` 1.

## Decision

- **`Bz` becomes `Sport-, Freizeit- und Erholungsfläche` with `bez` `Erholungsfläche`.** The engine takes that `bez`
  (`UNSPECIFIED_RECREATION_BEZ` in `open_land.rs`) as open land 401 **only as a candidate**, so it is in the candidate
  layer of the vegetation (balanced and every other profile alike), never on the map. The annex lumps parks and
  sports grounds, zoos and allotments in one class, so the class alone does not tell which of them lies there; the
  mapper checks it. It stays no out-of-bounds ground: allotments are only one of seven kinds, so `Kleingarten` is not
  guessed. No German land use is touched, the new `bez` exists only in Polish data.
- **`Bi` stays unmapped.** The annex has no single land use for it: a school yard (open to the map's user, the
  case of the schoolyard map), an office, a shop (private) and an active cemetery (candidate lawn in Germany) share the
  class, so any one name would be wrong for most of its parcels. Mapping it as private ground (520) would mark the
  schools of the schoolyard map out of bounds. What lies on a `Bi` parcel is drawn from the buildings and OSM as before.
  A parcel that mixes `Bi` with another class (`Bi,Bz`, `B,Bi`) has no use, by the rule of ADR 0171.
- Adapter revision 10.

## Consequences

- Recreation parcels of EGiB appear as 401 candidates; where OSM or the ALKIS-like land use already has the park as a
  map object, the existing rule drops the candidate below it (`drop_open_land_under_alkis`), and the training map
  reduces it to the open ground like any park.
- `Bi` stays the largest unmapped class in a Polish town (472 of the first 1000 parcels of Słubice).
- Open: a source that splits `Bi` (the BDOT10k `PTZB_A` layer and `KUSC_A` cemeteries already do for the fallback).
  Measured on 24 November 2026 in [ADR 0194](0194-poland-cemeteries-on-bi-parcels-from-bdot10k.md): `PTZB_A` does not split
  `Bi` (it follows the orthophoto, not the register); `KUSC_A` does for cemeteries, which become `Friedhof` candidates;
  schools and hospitals (`KUOS_A`, `KUOZ_A`) are found but have no land use that fits, shops and offices have no class.
