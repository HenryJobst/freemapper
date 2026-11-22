# Poland: reference of the GUGiK raster services

Probed live on 2026-11-21 (Task 2.0 of `docs/superpowers/plans/2026-11-20-poland.md`). The
sheet-index WFS services of GUGiK answer again after an outage; the host
`opendata.geoportal.gov.pl` serves the files. No key, no login.

## The index services

Base: `https://mapy.geoportal.gov.pl/wss/service/PZGIK`. WFS 2.0.0, one feature layer per flight
year, GML 3.2.

| Product | Service | Layers |
|---|---|---|
| Terrain model (NMT) | `NumerycznyModelTerenuEVRF2007/WFS/Skorowidze` | `gugik:SkorowidzNMT2018` … `2026` |
| Surface model (NMPT) | `NumerycznyModelPokryciaTerenuEVRF2007/WFS/Skorowidze` | `gugik:SkorowidzNMPT2019` … `2026` |
| Orthophoto | `ORTO/WFS/Skorowidze` | `gugik:SkorowidzOrtofomapy1957` … `2026` |

A request that asks for the sheets at a place:

```
…/Skorowidze?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=gugik:SkorowidzNMT2022
  &BBOX=507500,197650,507700,197850,urn:ogc:def:crs:EPSG::2180
```

The box is **north, east** (EPSG:2180 in the URN form has the northing first). Słubice lies at
N 507596, E 197747.

## What a feature carries

Both height products: `godlo` (sheet number, e.g. `N-33-126-C-c-3-4`), `akt_rok` (year),
`char_przestrz` (pixel size as text: `1.00 m`, `0.50 m`, `5.00 m`, `25.0 m`), `zrodlo_danych`
(source), `format` (`ARC/INFO ASCII GRID`), `url_do_pobrania`, `uklad_xy` (`PL-1992`),
`uklad_h` (`PL-EVRF2007-NH`), `blad_sr_wys` (mean height error in m), `msGeometry` (the tilted
sheet outline in PL-1992) and `gml:boundedBy`.

Orthophoto: the same without `char_przestrz`, but with `piksel` (`0.25`, `0.05`), `kolor`
(`RGB` or `CIR`) and `wlk_pliku_mb` (file size in MB).

`zrodlo_danych` takes `Skaning laserowy` (LiDAR), `Zdj. lotnicze` (aerial photographs) and
`Zdj. cyfrowe` (digital photographs); the last two are photogrammetry.

## Findings per place

Sheets whose 100 m box touches the point; years and pixel sizes as the index lists them.

| Place | NMT | NMPT | Orthophoto |
|---|---|---|---|
| Słubice | 2020 to 2025: 1 m (LiDAR 2020 and 2024, photographs 2020 to 2023), 5 m 2022 and 2025 | 0.5 m LiDAR 2020, 2024 | 2014 to 2025, 17 sheets |
| Warszawa | 2019 to 2025: 0.5 m LiDAR 2022 (four sheets), 1 m LiDAR 2023 and 2025, photographs 2019 to 2021 | 0.5 m LiDAR 2022, 2023, 2025 | 2013 to 2025, 27 sheets |
| Kraków | 2018 to 2025: 1 m LiDAR 2021, 2023, 2025, photographs 2019, 2022, 2024, 5 m and 25 m older | 0.5 m LiDAR 2021, 2023, 2025 | 2013 to 2025, 20 sheets |
| Bory Tucholskie (forest) | 2019 to 2026: 1 m LiDAR 2022, 2024, 2026, 1 m photographs 2019, 5 m 2020, 2021, 2023, 2025 | 0.5 m LiDAR 2022, 2024, 2026 | 2013 to 2026, 18 sheets |

What this means for the choice of a sheet:

- A place has several sheets per sheet number, from different years and sources. LiDAR over
  photographs, a finer pixel over a coarser one and a newer flight over an older one is the rule
  `best_per_cell` applies. Warszawa shows that the finest NMT can be 0.5 m, not only 1 m.
- The surface model exists only from LiDAR (0.5 m). Where a place has none, a map has no nDSM.
- **The orthophoto index mixes RGB and CIR.** Słubice 2022 lists, for one sheet number, a
  0.05 m RGB sheet (243 MB), a 0.25 m CIR sheet (11 MB) and a 0.25 m RGB sheet (11 MB). A CIR
  sheet is false-colour infrared and must never serve as the orthophoto, so the index client drops
  it. The 0.25 m rule (nothing finer, ADR 0167) keeps the 0.05 m sheets out.

## Sizes

- One NMT sheet at 1 m: about 33 MB as ASC (`Content-Length` 33 319 513 for
  `N-33-126-C-c-3-4`, 2022), covering about 5.5 km².
- One orthophoto sheet at 0.25 m RGB: 10 to 11 MB; the same sheet at 0.05 m: 200 to 243 MB.

## Download address

`url_do_pobrania` points to `https://opendata.geoportal.gov.pl/…` (for example
`NumDaneWys/NMT/76153/76153_1192530_N-33-126-C-c-3-4.asc`, `ortofotomapa/76966/…tif`). The client
accepts only `https` and this host.

## Stage 3: the cadastre EGiB (measured 21 November 2026)

- **Service.** `https://mapy.geoportal.gov.pl/wss/service/PZGIK/EGIB/WFS/UslugaZbiorcza`, WFS
  2.0.0, layers `ms:budynki` and `ms:dzialki`. `GetCapabilities` takes about 2 s, a page of 500
  features under 1 s. `CountDefault` is 1000; an answer with `numberMatched="unknown"` and a
  `next` attribute on the collection (its ampersands as `&amp;`) when the page is full.
- **Format.** GML 3.2, `gml:Polygon` with `gml:exterior` and `gml:interior`, `posList` of `N E`
  pairs in EPSG:2180 (`srsName="urn:ogc:def:crs:EPSG::2180"`). The box of a request is north first.
  A request without `SRSNAME` answers in the same CRS.
- **Buildings (`ms:budynki`).** `ID_BUDYNKU` (`080505_4.0001.2682_BUD`), `RODZAJ` (a letter: `m`,
  `b`, ...), `KONDYGNACJE_NADZIEMNE`, `KONDYGNACJE_PODZIEMNE` (floors above and below ground).
- **Parcels (`ms:dzialki`).** `ID_DZIALKI`, `NUMER_DZIALKI`, `NUMER_OBREBU`, `NAZWA_OBREBU`,
  `NAZWA_GMINY`, `POLE_EWIDENYJNE`, `KLASOUZYTKI_EGIB` (the register classes of the parcel,
  comma-separated: `B`, `Bi`, `Bp`, `dr`, `B,Bp`), `GRUPA_REJESTROWA`, `DATA`.
- **Słubice, 500 m square.** 484 parcels in one page of 766 KB; at the Oder bridge 148 buildings
  and 203 parcels of the classes `B`, `Bi`, `Bp`, `Bz` and `dr`.
- **BDOT10k.** The index `PobieranieBDOT10k` (`ms:BDOT10k_powiaty`, 380 powiats) lists per powiat
  `TERYT`, `NAZWA_POWIATU`, `Data_aktualizacji` and `URL_GML`
  (`https://opendata.geoportal.gov.pl/bdot10k/schemat2021/04/0401_GML.zip`). Without
  `PROPERTYNAME=TERYT,NAZWA_POWIATU,URL_GML,Data_aktualizacji` the answer is 63 MB in 68 s. One
  GML archive of a powiat is about 54 MB. The client for it is not written.

## Permit notes (measured 22 November 2026)

- **GDOŚ WFS** `https://sdi.gdos.gov.pl/wfs` (GeoServer, WFS 2.0, `outputFormat=application/json`): 11
  layers answer (national parks, park buffer zones, landscape parks, nature reserves, protected
  landscape areas, Natura 2000 habitats and birds, ecological sites, landscape complexes,
  documentation sites, natural monuments). Park layers also hold the buffer zones, named
  "... - otulina".
- **Axis order.** `srsName=EPSG:4326` with `bbox=minLon,minLat,maxLon,maxLat,EPSG:4326` gives
  longitude-first GeoJSON. The URN `urn:ogc:def:crs:EPSG::4326` gives latitude-first, `EPSG:2180`
  northing-first.
- **Attributes.** `nazwa`, `kodinspire`; Natura 2000 also `kod` (PLB, PLH, PLC); natural monuments
  `gatunek`, `typ`, `podtyp`, `obiekt`, `data_utwor`, `link`; documentation sites `rzedna`. Polygons are
  large (up to 4.6 MB for one landscape protection area).
- **PRG WFS** (MapServer): GML 3.2 only, JSON is refused with 400. Layers `U06_Nadlesnictwo` (forest
  districts) and `U07_Regionalna_dyrekcja_lasow_panstwowych`; `PROPERTYNAME` drops the 700 KB directorate
  polygon. `JPT_KOD_JE`, `JPT_NAZWA_` (district names in capitals). A district code `17-02` belongs to the
  directorate `17` (checked on Warsaw samples only). The box with the URN `EPSG:2180` is northing-first.
- **Not done.** Gmina, powiat and voivodeship: the districts section assumes Bezirke.
- **Coverage.** The PRG layer covers the State Forests only, not private forest.

## Addendum 22 November 2026: EGiB codes checked against the live service

ADR 0171 listed the building letters and parcel classes as not verified. Checked now; the ADR
stays as it was.

**Samples** (`ms:budynki` and `ms:dzialki`, one page each, User-Agent `freemapper-research`):
Słubice (N 507300 to 507900, E 197600 to 198200), Warszawa Śródmieście (N 487000 to 487600, E
636800 to 637400), Białystok (N 590000 to 591500, E 775000 to 776500) and two villages of Podlasie
(powiats 2002 and 2008: N 600000 to 601500, E 800000 to 801500; N 640000 to 641500, E 760000 to
761500).

**Building letters (`RODZAJ`).** The service has no code table (`DescribeFeatureType` says only
`string`), and the text of the ordinance (Dz.U. 2021 poz. 1390) names the kind only as "by the
Classification of Fixed Assets (KŚT)" without the letters. The letters come from two official
documents: the reply of the Ministry of Development and Technology to a petition (5 June 2025,
DAB-III.053.8.2025, `gov.pl/attachment/55e4c6f7-df33-426f-8345-566c497bb1c7`) and the list of the
district office of Brzeziny on the building data (`powiat-brzeziny.pl`, after the ordinance of
29 November 2013). Both say: `m` residential, `g` production, services and farm buildings, `t`
transport and communication, `k` education, science, culture and sport, `z` hospitals and other
health care, `b` office, `h` commercial and services, `p` industrial, `s` tank, silo and storage
building, `i` other non-residential. The first version of the client had `z`, `s` and `k` wrong
(tank, hospital and sport) and `o` unused. Counts per letter:

| Area | b | g | h | i | k | m | p | s | t | z |
|---|---|---|---|---|---|---|---|---|---|---|
| Słubice (583) | 8 | 0 | 110 | 107 | 10 | 167 | 1 | 0 | 177 | 3 |
| Warszawa (127) | 38 | 0 | 11 | 24 | 6 | 48 | 0 | 0 | 0 | 0 |
| Białystok (935) | 11 | 2 | 86 | 131 | 11 | 569 | 10 | 36 | 86 | 0 |
| Village 2002 (8) | 0 | 1 | 0 | 2 | 0 | 5 | 0 | 0 | 0 | 0 |
| Village 2008 (25) | 0 | 18 | 0 | 2 | 0 | 5 | 0 | 0 | 0 | 0 |

No other letter occurred. The Brzeziny list also gives the finer building classes of the newer
scheme (PKOB: `mj`, `md`, `mt`, `mz`, `a`, `u`, `d`, `o`, `e`, `f`, `r`, `x`; `t` there means
garages, `h` hotels). No answer of the service held one; the client maps them to the same names as
the coarse letters, so data in that scheme would still read. The data cannot settle the letters by
itself (`t` is 177 of 583 buildings in Słubice, which fits garages and sheds as well as transport);
the table rests on the two documents.

**Parcel classes (`KLASOUZYTKI_EGIB`).** The ordinance (§ 8, § 68) gives: `B` residential land, `Ba`
industrial, `Bi` other built-up, `Bp` urban land not built on, `Bz` recreation and leisure, `K`
mining land, `dr` roads, `Tk` railway, `Ti` other transport land, `Tp` land set aside for roads and
railways to be built, `Wm`, `Wp`, `Ws` sea, running and standing water, `Tr` miscellaneous land,
`N` waste land, and the farm and forest classes `R` arable, `Ł` meadow, `Ps` pasture, `S` orchard,
`Br` farmland with buildings, `Lzr` trees and shrubs on farmland, `Wsr` ponds, `W` ditches, `Ls`
forest, `Lz` trees and shrubs. `Bz` is recreation land (the first version had it as unknown), `Tp`
is not "industrial" but land for planned roads.

What the service really sends differs from the plain letters: the classes of farm and forest land
carry their soil class (`RIVa`, `RIIIb`, `ŁV`, `PsVI`, `LsIV`, `LzV`), and a class can have a prefix
and a slash for a second kind of land on that soil (`Br/RV`, `W/ŁIV`, `Lzr/PsVI`, `Wsr/PsIV`). The
first version knew only the bare letters, so every rural parcel had no land use. Counts of parcels
by class in the five samples:

| Area | Parcels | Classes |
|---|---|---|
| Słubice | 694 | `B` 348, `Bi` 206, `Bp` 72, `dr` 55, `Bz` 6, `Ba` 2, `B,Bp` 2, `Wp` 1, `Tr` 1, `B,Bi` 1 |
| Warszawa | 267 | no class at all (the attribute is empty) |
| Białystok | 972 | 14 `dr`, 8 `B`, 5 `RVI`, 1 `Ls`; 944 empty |
| Village 2002 | 193 | `R`, `Ps`, `Ł`, `Ls` with all soil classes and the prefixes `Br/`, `W/`, `Lzr/`, `Wsr/`; `dr` 18, `B` 12, `Wp` 1, `N` beside others in several |
| Village 2008 | 350 | `Ł`, `Ps`, `R`, `Lz` with soil classes; `N` and `W` beside them in many; `dr` 19, `Wp` 1, `K` 1 |

So in the big cities (Warszawa, Białystok) the national service gives no classes for most parcels:
the land use of a map there is nearly empty. That is a gap in the data, not in the client.

**Mapping now** (`land_use_of_class` in `crates/adapters/pl/src/egib.rs`): the soil class is cut
off, a prefix `Lzr` gives `Gehölz`, `Wsr` `Stehendes Gewässer`, `Br` and `W` the soil underneath.
`Ls` Wald, `Lz` Gehölz, `Ł` and `Ps` Landwirtschaft (Grünland), `R` and `S` Landwirtschaft, `Wp`
Fließgewässer, `Ws` and `Wsr` Stehendes Gewässer, `dr` Straßenverkehr, `Tk` Bahnverkehr, `B`
Wohnbaufläche, `Ba` Industrie- und Gewerbefläche, `Bz` Sport-, Freizeit- und Erholungsfläche (without
`bez`, so the map draws nothing for it yet). Left out: `Bi`, `Bp`, `Ti`, `Tp`, `K`, `Tr`, `Wm`,
`N`, a bare `W` and `Br`. A parcel of several classes has a use only when they all give the same one;
ditches (`W`) and waste land (`N`) among them are not counted. That last rule is a decision made on
the samples (a meadow with a strip of waste land is still a meadow) and has no source in the
ordinance.

Still unverified: the finer PKOB letters (none occurred), whether `Bi` should be a mixed-use area
(schools, offices and shops are in it), whether `Bz` should draw as a park (it holds sports grounds
and allotments as well), the classes of the cities with empty values, and whether `Tk` railway land
should be out-of-bounds private ground as in Germany (it is, through `Bahnverkehr`).

## Open

- Whether the index lists all years in one request with several `TYPENAMES`, and whether the
  service limits the number of features, is settled in the client and noted in its tests.
- Stability over days is not known; this is one session's measurement.
