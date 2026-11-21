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

## Open

- Whether the index lists all years in one request with several `TYPENAMES`, and whether the
  service limits the number of features, is settled in the client and noted in its tests.
- Stability over days is not known; this is one session's measurement.
