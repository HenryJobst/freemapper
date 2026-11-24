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
- **BDOT10k.** See "Addendum 22 November 2026: BDOT10k" below; the client is `bdot10k.rs`
  (ADR 0178).

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
- **Administrative units** (ADR 0181), same service: `A01_Granice_wojewodztw`, `A02_Granice_powiatow`,
  `A03_Granice_gmin` (also `A04` towns, `A05` registration units, `A06` registration precincts). Same
  GML 3.2, same `JPT_KOD_JE` (TERYT code) and `JPT_NAZWA_`. A voivodeship has 2 digits, a powiat 4, a
  gmina 7; the last digit of a gmina is its kind (1 urban, 2 rural, 3 urban-rural, 4 and 5 the town and
  the rural part of an urban-rural gmina, 8 a district, 9 a delegation). Names: `lubuskie` (no word
  "województwo"), `powiat słubicki`, `powiat Poznań`, `Słubice`. Słubice box: one gmina `0805053`, powiat
  `0805`, voivodeship `08`. Kościan box: gminas `3011032` and `3011011`, both named `Kościan`. Polygons:
  voivodeship 466 KB, powiat 79 KB, gmina 21 KB (Słubice) to 87 KB (two gminas at Kościan); the notes
  ask the polygons of the gminas only. `PROPERTYNAME` works on these layers too. Credit as for the
  forest districts.
- **Coverage.** The PRG layer covers the State Forests only, not private forest.

### Who to ask, by statute (ADR 0183, read 22 November 2026)

Consolidated texts from `api.sejm.gov.pl/eli/acts/DU/<year>/<pos>/text.pdf`; the full table with the
article numbers and what could not be verified is in
[ADR 0183](../adr/0183-poland-gmina-as-a-contact-in-the-permit-notes.md). Short form:

- Natural monuments, ecological sites, landscape complexes, documentation sites: the council of the
  gmina establishes them and the resolution names the supervisor and the bans (art. 44 ust. 1 and 2 of
  the Nature Conservation Act, Dz. U. 2026 poz. 1315). The Act names no permit for third parties.
- Gmina roads: wójt, burmistrz or prezydent miasta is the manager (art. 19 ust. 2 pkt 4, Public Roads
  Act, Dz. U. 2025 poz. 889). The permit for an event that hinders traffic (art. 65 and 65b of the Road
  Traffic Act, Dz. U. 2024 poz. 1251) comes from the traffic management body: starosta for powiat and
  gmina roads, prezydent miasta in cities with powiat rights (art. 10 ust. 5 and 6).
- Land of the gmina: the wójt manages municipal property and represents the gmina (art. 30 ust. 2 pkt 3
  and art. 31, Act on Gmina Self-Government, Dz. U. 2026 poz. 662).
- Several gminas (ADR 0193, probed 24 November 2026 on `mapy.geoportal.gov.pl` only): the layer
  `A03_Granice_gmin` returns every gmina that meets the box: two at Kościan (`3011032`, `3011011`),
  six west of Kraków (Kraków, Wieliczka, Zielonki, Liszki, Zabierzów, Wielka Wieś). The three hints name
  all of them, no cap, by share descending then by name, each with its share, and say that the head
  of each gmina manages its roads and land; a gmina below 0.5 % is left out when another is more than
  an edge. The plural texts exist in all 21 catalogs.
- Not verified: who the supervisor of a natural monument is in a given resolution; which land is
  municipal; the articles for Natura 2000 and the parks; the Prawo o zgromadzeniach.

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

**Parcel classes (`KLASOUZYTKI_EGIB`).** The ordinance (§ 9 ust. 3) gives: `B` residential land, `Ba`
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
Wohnbaufläche, `Ba` Industrie- und Gewerbefläche, `Bz` Sport-, Freizeit- und Erholungsfläche with `bez`
`Erholungsfläche` (a 401 lawn candidate, never a map object; see the addendum below). Left out: `Bi`, `Bp`, `Ti`, `Tp`, `K`, `Tr`, `Wm`,
`N`, a bare `W` and `Br`. A parcel of several classes has a use only when they all give the same one;
ditches (`W`) and waste land (`N`) among them are not counted. That last rule is a decision made on
the samples (a meadow with a strip of waste land is still a meadow) and has no source in the
ordinance.

Still unverified: the finer PKOB letters (none occurred), the classes of the cities with empty values, and whether `Tk` railway land
should be out-of-bounds private ground as in Germany (it is, through `Bahnverkehr`).

## Addendum 24 November 2026: `Bz` and `Bi` read from the annex (ADR 0192)

The classes are in § 9 ust. 3 of the ordinance (Dz. U. 2021 poz. 1390; the first version of this file said § 68, which
the ordinance does not have); what belongs to each is in Załącznik nr 1, lp. 13 to 17 (PDF
`https://api.sejm.gov.pl/eli/acts/DU/2021/1390/text.pdf`, read as text).

- `Bz` "tereny rekreacyjno-wypoczynkowe", lp. 17, land not under buildings: recreation centres, playgrounds, beaches,
  kept parks, squares and green strips (not in street lanes), historic sites, sports grounds, fairgrounds, zoological
  and botanical gardens, unkept green that is no forest or tree land, family allotment gardens on land unfit for
  farming. Mixed, so the map takes it as a 401 lawn candidate (`bez` `Erholungsfläche`), not as a park and not as
  private ground.
- `Bi` "inne tereny zabudowane", lp. 15: land under buildings other than homes, industrial and storage buildings with
  their yards, the land between and next to them, active cemeteries (and inactive ones not counted as trees), animal
  burial grounds. No single land use of the map fits (school yard, shop, office, cemetery), so it stays unmapped.
- `Bp` is "zurbanizowane tereny niezabudowane lub w trakcie zabudowy": urban land not built on or being built on
  (lp. 16), land where building began or where houses were pulled down for new building.
- Live service, Słubice, 24 November 2026 (box of the fixture, first 1000 parcels): `Bi` 472, `B` 352, `Bp` 88, `dr` 47,
  `Bz` 4, `Bz,Lz` 2, `Bi,Bz` 1 and a few farm classes. Kraków (Rynek area) sends `brak_uprawnień` for all 255 parcels
  asked, as noted below.

## Addendum 22 November 2026: BDOT10k (measured, ADR 0178)

**Service.** The WFS `https://mapy.geoportal.gov.pl/wss/service/PZGIK/BDOT/WFS/PobieranieBDOT10k`
(`Fees` and `AccessConstraints` both `NONE`) has one layer, `ms:BDOT10k_powiaty`, 380 powiats. It
lists packages and holds no objects: no BDOT10k WFS cuts out an area (other paths below
`PZGIK/BDOT/WFS` answer 401; `integracja.gugik.gov.pl` has none). The earlier note that the index
"ignores the box" was the axis order: `BBOX=<north>,<east>,<north>,<east>,urn:ogc:def:crs:EPSG::2180`
works and returns the powiat the box touches (`TERYT` 0805 for Słubice), 1.5 KB with
`PROPERTYNAME=TERYT,NAZWA_POWIATU,URL_GML,Data_aktualizacji`. `Data_aktualizacji` of the index
(2024-04-04) is not the date of the file: 0805 has `Last-Modified` 22 January 2026 and
objects of version 2024-10-21.

**Archives.** `https://opendata.geoportal.gov.pl/bdot10k/schemat2021/<WW>/<TTTT>_GML.zip`, `200
application/octet-stream`, `Accept-Ranges: bytes`. Sizes: Tarnów 7.6 MB, Płock 9.1, Białystok 15.6,
miechowski 17.4, Gdynia 29.4, suski 32.1, Kraków 35.2, Wrocław 37.0, Poznań 36.5, nowotarski 56.8,
słubicki 53.9, Warszawa 89.0 MB. The 0805 archive unpacks to 264 MB in 75 files (73 classes and
two XSD files); the largest are `OT_RTLW_L` 136 MB, `OT_BUBD_A` 32.5 MB (21 940 buildings),
`OT_SKJZ_L` and `OT_SKDR_L` about 20 MB. Download 17 s at home bandwidth. A file starts with a byte
order mark; a class without an object is a 454-byte file.

**GML.** GML 3.2, namespace `urn:gugik:specyfikacje:gmlas:bazaDanychObiektowTopograficznych10k:2.0`,
`ot:OT_<class>` in `gml:featureMember`, one `gml:Polygon` in `ot:geometria`, `srsName="EPSG:2180"`,
`posList` of **E N** pairs (EGiB: N E).

**Classes, values (powiat 0805) and mapping.**

| Class | Attribute (values) | Mapped to |
|---|---|---|
| `OT_BUBD_A` buildings (21 940) | `kodKst` 101 to 110, `kategoriaIstnienia` (eksploatowany 21 491, nieczynny 169, w budowie 180, zniszczony 100), `identyfikatorEGiB`, `zrodloDanychGeometrycznych` (EGiB 20 657, ortofotomapa 994, mapa zasadnicza 264, pomiar terenowy 23) | letter of the register by `kodKst` (counts equal those of `funkcjaOgolnaBudynku`: 110 m 9454, 108 g 7882, 102 t 2903, 103 h 515, 104 s 276, 101 p 271, 109 i 268, 105 b 179, 107 k 157, 106 z 34), then `building_function` |
| `OT_PTLZ_A` forest | `rodzaj`: zagajnik 1570, las 1143, zadrzewienie 33 | Wald, Wald, Gehölz |
| `OT_PTTR_A` | roślinność trawiasta 1606, uprawa na gruntach ornych 349 | Landwirtschaft / Grünland, Landwirtschaft |
| `OT_PTUT_A` permanent crops | sad, szkółka roślin, plantacja, ogródki działkowe | Landwirtschaft; allotments Kleingarten |
| `OT_PTWP_A` water | woda stojąca 971, woda płynąca 28 | Stehendes Gewässer, Fließgewässer |
| `OT_PTZB_A` built-up | jednorodzinna 1274, pozostała zabudowa 616, wielorodzinna 247, przemysłowo-składowa 198, handlowo-usługowa 130 | Wohnbaufläche, none, Wohnbaufläche, Industrie- und Gewerbefläche, none |
| `OT_PTNZ_A` | teren pod urządzeniami technicznymi lub budowlami 99, teren przemysłowo-składowy 51 | none, Industrie- und Gewerbefläche |
| `OT_PTPL_A` squares | `materialNawierzchni`: kostka prefabrykowana 94, grunt naturalny 48, masa bitumiczna 32, beton 27, ... | Platz, except grunt naturalny, żwir, tłuczeń |
| `OT_KUKO_A` | stacja paliw 45, parking 13, ... | parking: Platz / Parkplatz |
| `OT_KUSC_A` | cmentarz komunalny 23, wyznaniowy 12, wojenny 3, zespół sakralny 3 | Friedhof (not the church grounds) |
| `OT_KUSK_A` | kompleks sportowo-rekreacyjny 8, park lub skwer 3 | Sport-, ... / Grünanlage for the park |

Read since ADR 0187 (forest maps only, see the addendum below): `OT_SKDR_L` and `OT_SKRP_L`. Not
read: `OT_SKJZ_L` (carriageway lines, repeat the axes), `OT_SKTR_L` (railway track, not tracks of
the forest), `OT_PTRK_A` bushes, `OT_PTWZ_A` pits, `OT_PTGN_A`, `OT_OIMK_A`
wetlands (532 objects overlapping forest and grass), `OT_BUZT_A`, `OT_KUPG_A` and the rest.

**Licence.** `geoportal.gov.pl/en/data/topographic-objects-database-bdot10k`: the database "is
provided free of charge and can be used for any purpose"; no attribution wording. The legal basis
is the ordinance of 27 July 2021. `Fees`/`AccessConstraints` of the WFS: `NONE`.

**The gaps in EGiB** (buildings, `ms:budynki`, 600 m boxes, three tries each):
Chochołów (E 561324, N 172781), Czarny Dunajec centre (E 561964, N 172655) and Lipnica Wielka
(E 532323, N 184714) answer `numberReturned="0"` every time, while Nowy Targ, Jabłonka and the
rest of powiat 1211 answer. Of 45 random points inside the border, one answered nothing. The
service also answers empty sporadically for a 500 m cell (a cell with zero in one run held 735
buildings in the next, 600 m box), so an empty answer is not proof of a gap; the client asks an empty first
answer twice more, and the fallback treats a third empty one as a gap. Land use classes (`KLASOUZYTKI_EGIB`, 600 m boxes, share of the box covered by parcels
with a usable class): Słubice 0.60, Poznań 0.63, a village of powiat 2002 0.99; Warszawa, Białystok,
Gdańsk, Lublin, Zakopane 0.0 (empty class) and Kraków and Wrocław 0.0 (the class is the text
`brak_uprawnień` / `brak uprawnień`, "no authorisation"); five of seven rural points 0.0 (empty
class). The threshold of the fallback is 0.25. A building kind in words
(`RODZAJ` = `budynekNiemieszkalny`) came back at 52.851 N, 14.824 E (West Pomerania).

## Open

- Whether the index lists all years in one request with several `TYPENAMES`, and whether the
  service limits the number of features, is settled in the client and noted in its tests.
- Stability over days is not known; this is one session's measurement.

## Register of monuments of the NID (ADR 0184)

- **Service.** `https://usluga.zabytek.gov.pl/INSPIRE_IMD/service.svc/get`, WMS 1.3.0, layer
  `Immovable_Monuments`, only `GetFeatureInfo` (no feature download). With `CRS=EPSG:4326` the box is
  latitude first. The whole map box as one pixel (`WIDTH=1&HEIGHT=1&I=0&J=0`, `INFO_FORMAT=text/xml`,
  `FEATURE_COUNT=5000`) returns every monument that meets the box.
- **Answer.** GML `FeatureCollection` with `Layer` elements and `Attribute Name` children
  (`INSPIREID`, `SITENAME`, `NATIONALMONUMENTSRECORD`, ...), no geometry. Entries repeat: Kraków
  2108 entries with 39 repeated INSPIREIDs; merged by INSPIREID. An empty box gives a self-closing
  `FeatureCollection`; an error gives HTTP 200 with a `ServiceExceptionReport`.
- **Gaps.** Archaeological sites and Pomniki historii are not in the layer (checked 23 November 2026: the
  Kopiec Kościuszki of Kraków, a Pomnik historii, appears only through its register entries, no `NID_N_18_PH` id).
- **Licence.** Credit "© Narodowy Instytut Dziedzictwa (CC BY 4.0)"; the wording is not prescribed
  and was not confirmed with the NID.
- **Law.** Ustawa o ochronie zabytków, art. 36 ust. 1 pkt 1 (permit: conservation, restoration and building work at
  a registered monument) and, since 3 June 2026 (Dz. U. 2025 poz. 1673), art. 36aa (notification instead of permit
  for building work in the surroundings of a monument, a change of use, devices and signs on a monument and other
  actions that could change substance or appearance; the conservator may object within 60 days, art. 36aa ust. 3,
  and then require a permit, ust. 10); pkt 2, 7, 9, 10 and 11 of art. 36 ust. 1 were repealed by that act. Checked
  on the Sejm ELI API (`api.sejm.gov.pl/eli`) on 23 November 2026. Ustawa o lasach, art. 5 ust. 1 pkt 2
  (Dz. U. 2025 poz. 567), checked 22 November 2026.

## Register of archaeological monuments of the NID (ADR 0190)

Measured 23 November 2026, anonymous, `User-Agent` of the program only.

- **Service.** `https://usluga.zabytek.gov.pl/INSPIRE_AMD/service.svc/get`, WMS 1.3.0, layer
  `Archaeological_Monuments`, the same software and the same `GetFeatureInfo` request as the register of
  immovable monuments (whole map box as one pixel, latitude first). The capabilities list this one layer.
  Dataset 210 of dane.gov.pl ("Rejestr zabytków archeologicznych", CC BY 4.0, quarterly) names this WMS, a REST API
  and a CSV.
- **Answer.** `Layer` elements with `Attribute Name` children in camel case (`inspireID`, `siteName`,
  `nationalMonumentsRecord`, `siteProtectionClassification` = `archaeological`), no geometry. The reader lowers
  the names, so both registers use one parser. Entries repeat (Biskupin: 9 entries, 8 ids). Biskupin box
  (52.79 to 52.80, 17.74 to 17.752): 5 KB in 0.4 s; Kraków Stare Miasto box (50.055 to 50.073, 19.93 to 19.96): 67
  entries, 64 ids, 38 KB in 0.5 s; Słubice: empty `FeatureCollection`. The whole of Poland as one box: 2.8 MB in
  2.8 s, so the cap of 5000 entries holds.
- **Names.** Mostly the place and a site number (`Biskupin, st. 10`, `Kraków-Stare Miasto, st. 2`; `st.` is
  stanowisko, site), so grouped names repeat little.
- **CSV.** `https://api.dane.gov.pl/media/resources/20261002/V_OTWARTE_DANE_ZESTWIENIE_ZRA.csv` (2.6 MB, 843 rows,
  encoding not UTF-8) has gmina and village but no coordinates; not used.
- **AZP.** `https://usluga.zabytek.gov.pl/WFS_AZP/service.svc/get` gives only the sheet grid of the Archaeological
  Record of Poland ("Sekcje Archeologicznego Zdjęcia Polski"), not the sites. The sites of the AZP that are not in
  the register are not open data; not covered.
- **Law.** Ustawa o ochronie zabytków, art. 36 ust. 1, checked on the Sejm ELI API on 23 November 2026: pkt 5
  archaeological research and pkt 12 searching for hidden or abandoned movable monuments with electronic or
  technical devices (and diving equipment) need the permit of the voivodeship conservator; pkt 1 conservation,
  restoration and building work at a registered monument likewise. The amendment Dz. U. 2023 poz. 1904 repeals
  pkt 12 and adds art. 36b (an adult may search with the consent of the owner and the holder of the property after
  a notification in the search register through the app of the ministry), art. 36c (searches forbidden among others
  on registered monuments and within 10 m of them, on cemeteries and in cultural parks) and art. 36d. Its entry into
  force was moved twice: to 1 May 2025 (Dz. U. 2024 poz. 647) and then to 1 January 2027 (Dz. U. 2025 poz. 537, in
  force since 1 May 2025). So on 23 November 2026 pkt 12 still applies and art. 36b is not yet in force; the hint
  says so and names the date. Dz. U. 2025 poz. 1673, art. 4, adapts that amendment to 1 January 2027. Check the date
  again before each release; the legislator moved it before.

## Pomniki historii (ADR 0191)

Measured again 23 November 2026, anonymous, `User-Agent` of the program only.

- **Service.** `https://usluga.zabytek.gov.pl/INSPIRE_WFS_MHD/service.svc/get`, WFS 2.0.0, feature type
  `gmgml:Monuments_of_History`, `outputFormat=application/vnd.geo%2Bjson` (the `+` must be written `%2B`;
  `application/json` is refused), EPSG:2180 only. `GetFeature` needs `typeNames`. The whole country: 387 features
  (184 INSPIREIDs, 134 names), 2.0 MB, 2.5 s, with polygons, lines or points. A monument has several parts
  (Tarnowskie Góry: 55, Przemyśl: 48). Properties: `INSPIREID` (`..._PH.<n>`), `SITENAME`,
  `NATIONALMONUMENTSRECORD`, `SITEPROTECTIONCLASSIFICATION` (`cultural`, `landscape`, `archaeological`),
  `LEGALFOUNDATIONDATE`, `LEGALFOUNDATIONDOCUMENT` (a `zabytek.pl` link), `PERCENTAGEUNDERDESIGNATION`.
- **Coordinates.** GeoJSON with `"crs": "EPSG:2180"`, easting first (`[563837.95, 243217.53]` is the Kopiec
  Kościuszki area), converted by the adapter with `pl1992_inverse`.
- **BBOX.** `bbox=<northing min>,<easting min>,<northing max>,<easting max>,EPSG:2180` works (northing first).
  Without the CRS suffix the box is read differently: Kraków (242905,563698,245146,565102) gave one of the two parts,
  Słubice nothing (an earlier note that this returns the whole country could not be reproduced).
  `urn:ogc:def:crs:EPSG::2180` and `EPSG:4326` give an `ExceptionReport` with status 200, as does the wrong output
  format. The Kraków box of the fixture (242905,563671,245146,565130): 2 parts of one monument, 11 KB, 0.7 s.
- **Empty answer.** `{"type": "FeatureCollection", ..., "features": [  ], "bbox": [1.79769313486232e308,
  1.79769313486232e308, -1.79769313486232e308, -1.79769313486232e308] }`: the numbers exceed an f64, so a plain JSON
  parse fails; the adapter reads the members raw.
- **Archaeological Pomniki historii.** Biskupin (reserve), Ostrów Lednicki and Krzemionki have
  `SITEPROTECTIONCLASSIFICATION` `archaeological` and are also in the archaeological register.
- **Kopiec Kościuszki.** It is in this layer as `NID_N_12_PH.15423` (two polygon parts); the register of immovable
  monuments knows it only through its entries, with no `PH` id (see above).
- **Licence.** Dataset 168 of dane.gov.pl, CC BY 4.0; credit as for the other NID sources.
- **Law, read on the Sejm API (`api.sejm.gov.pl/eli`) on 23 November 2026.** Art. 15 ust. 1: the President, by
  regulation, on the minister's motion, may declare a registered immovable monument or a cultural park of special
  value a Pomnik historii. Art. 36 ust. 1 (consolidated text Dz. U. 2024 poz. 1292): pkt 1 work at a registered
  monument, pkt 5 archaeological research, pkt 12 searching with devices. The amendment Dz. U. 2023 poz. 1904 repeals pkt 12 and adds art. 36b (an adult searches with the consent of the owner and the possessor after notifying the search in the ministry's app), art. 36c (the finder's duties) and art. 109e (a fine); its entry into force was moved to 1 January 2027 (Dz. U. 2025 poz. 537), so pkt 12 still applies on 23 November 2026.
  Dz. U. 2025 poz. 1673 (in force 3 June 2026) repeals art. 36 ust. 1 pkt 2, 7 and 9 to 11 and adds art. 36aa: a
  notification, not a permit, for building work in the surroundings, a change of use, signs and other actions that could
  change the look. The hints on the archaeological register (ADR 0190) and the register of immovable monuments (ADR 0184) follow this law since the update of 23 November 2026; the permit of pkt 12 still applies until 1 January 2027, so check the date before every release.

## Addendum 22 November 2026: BDOT10k roads and paths (measured, ADR 0187)

Powiat 0805 (słubicki), archive of 22 January 2026, GML of 21 October 2024 for the roads.

**Classes.** `OT_SKDR_L` road axes (13 822, 19.7 MB), `OT_SKJZ_L` carriageway lines (14 147, 20.6 MB,
repeat the axes), `OT_SKRP_L` footways and cycle ways (540, 0.7 MB), `OT_SKTR_L` **railway** track
(273, 0.4 MB; `rodzajPojazduSzynowego` kolej, `funkcjaToru`, `rodzajTorow`). A geometry is one
`gml:Curve` of one `gml:LineStringSegment` with `posList` of **E N** pairs in EPSG:2180 (all 13 822
roads have a single segment; the reader joins several, should one come).

**Attributes (SKDR_L).** `kategoriaIstnienia` eksploatowany 13 821, w budowie 1; `klasaDrogi` droga
wewnętrzna 9 726, lokalna 2 437, zbiorcza 909, główna ruchu przyśpieszonego 314, główna 226,
dojazdowa 179, autostrada 31; `materialNawierzchni` grunt naturalny 9 357, masa bitumiczna 2 711,
kostka prefabrykowana 501, bruk 373, płyty betonowe 350, beton 232, tłuczeń 165, żwir 77, kostka
kamienna 56; `szerokoscNawierzchni` (m) on 4 247 (none of the 9 357 roads on bare ground; nearly every paved road); `liczbaJezdniDrogi` 1 or 2; `polozenie` na powierzchni gruntu 13 618, ponad
powierzchnią gruntu poziom 1 88, pod powierzchnią gruntu 3 (a missing `polozenie` is the rest);
`zrodloDanychGeometrycznych` ortofotomapa 12 548, mapa topograficzna 10k 1 033, pomiar terenowy 228,
mapa zasadnicza 13. `numerDrogi` and `nazwa` on the numbered roads.

**Attributes (SKRP_L).** `rodzaj` aleja lub pasaż 374, droga dla rowerów 128, ścieżka 38;
`materialNawierzchni` kostka prefabrykowana 223, grunt naturalny 177, masa bitumiczna 64, kostka
kamienna 17, żwir 9, płyty betonowe 5, beton 5; `szerokosc` 1.5 to 7 m (3.0: 172, 1.5: 91, 2.0: 46).

**Reading single members.** The archive answers byte ranges (`Accept-Ranges: bytes`). The two class
files are 1 943 505 and 65 334 bytes packed (deflate, 90 and 91 %); with the central directory
about 2.1 MB for the whole map instead of the 53.9 MB archive.

**Dense where?** 119 axis pieces in 1 km² at Słubice.

**Not measured:** other powiats; the share of the BDOT10k length that OpenStreetMap lacks.

## Addendum 24 November 2026: what BDOT10k says about `Bi` parcels (measured, ADR 0194)

Archives of 0805 (słubicki, 22 January 2026, 53.9 MB) and 3064 (Poznań, 36.5 MB), unpacked in the scratch folder;
the live EGiB parcels (`ms:dzialki`, 1000 per page) of a 3 km box at the Słubice cemeteries (1919 parcels) and a 5 km
box at Poznań (23 191 parcels, `Bi` 3585, `B` 11 251). Every BDOT10k polygon was sampled on a grid of at most 8 steps
and the EGiB class under each sample counted.

- **`OT_PTZB_A`** (2465 objects at 0805, 5217 at 3064): `rodzaj` jednorodzinna, wielorodzinna, przemysłowo-składowa,
  handlowo-usługowa, pozostała zabudowa; `charakter` gęsta, luźna, zwarta. On the register: `handlowo-usługowa` on `Bi`
  687, `B` 449, `dr` 243, `Bp` 222 samples; `pozostała zabudowa` on `Bi` 1188, `B` 481, `dr` 371, `Bp` 154;
  `jednorodzinna` on `B` 1344 and `Bi` 233. It does not split `Bi`.
- **`OT_KUOS_A`** (28 at 0805, 279 at 3064): `rodzaj` szkoła lub zespół szkół 19 and 152, przedszkole 7 and 76, szkoła
  wyższa 1 and 39, ośrodek naukowo-badawczy 1 and 12. On `Bi`: Słubice 157 of 223 samples, Poznań `szkoła` 1623 of 1987.
- **`OT_KUOZ_A`** (4 and 32): zespół szpitalny lub sanatoryjny, zakład opieki socjalnej lub dom dziecka, żłobek; on
  `Bi` (Poznań hospitals 376 of 402).
- **`OT_KUHU_A`** (3 and 8): trade and services, **no `rodzaj`**; 20 samples at Słubice, mostly `B`/`Bi`.
- **`OT_KUSC_A`** (41 and 82): `cmentarz komunalny` 23 and 4, `cmentarz wyznaniowy` 12 and 19, `cmentarz wojenny` 3 and 1,
  `zespół sakralny lub klasztorny` 3 and 58. Poznań `cmentarz komunalny`: `Bi` 47, `Bz` 13, `dr` 2 samples; Słubice
  `cmentarz komunalny`: `Bi` 17, `Bz` 25; `cmentarz wyznaniowy`: `Lz` 18, `B` 12 (Słubice), `B,Bz` 13 (Poznań).
  Church grounds lie on `Bi` (Poznań 517 of 660 samples) and are no cemetery.
- **Sizes.** Member `OT_KUSC_A.xml`: 52 094 B unpacked and 7 648 B packed at 0805, 127 063 B and 23 230 B at 3064. The
  directory of an archive: 7 753 B for 75 members. Per map (index 1.5 KB, directory, one member): about 17 KB at
  Słubice and 32 KB at Poznań, instead of the whole archive.
- **Use.** The cemeteries (not church grounds) on parcels without a land use of EGiB become `Friedhof`; everything else
  of `Bi` stays unmapped (ADR 0194).
- **Not measured:** other powiats.
