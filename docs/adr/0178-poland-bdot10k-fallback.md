# ADR 0178: BDOT10k fills where the Polish cadastre gives nothing

## Status

Accepted, 22 November 2026. Builds on [ADR 0171](0171-poland-cadastre-egib.md), which left
BDOT10k as "not decided here".

## Context

The national EGiB service has two gaps (`docs/regions/poland-reference.md`, addenda of 22
November 2026):

- **No cadastre service in a municipality.** The WFS answers with zero features. Measured at
  Lipnica Wielka and Chochołów (powiat nowotarski, Orawa), and at one of 45 random points
  inside the border (the others answered).
- **No land use class on most parcels** of about half the big cities (Warszawa, Białystok,
  Gdańsk, Lublin, Olsztyn, Rzeszów, Toruń, Gorzów, Zamość, Zakopane). The buildings are there.

BDOT10k, the topographic object database of GUGiK, covers the whole country with buildings
(`OT_BUBD_A`, 95 % of them with their geometry from EGiB, the rest from the orthophoto and the
base map) and land cover polygons (forest, grass, water, built-up land, squares, cemeteries and
more).

## Decision

- **Archive per powiat, no WFS for areas.** The WFS `PobieranieBDOT10k` lists only the packages.
  Its `BBOX` works with the north-first URN box (the earlier note "ignores the box" was the axis
  order), so one request of 1.5 KB names the powiats a map area touches. The client downloads the
  GML archive of each (7 to 89 MB measured, 54 MB at Słubice, cap 150 MB) into the shared cache
  (`gugik/bdot10k/<teryt>_<date>.zip`; the date is the `Last-Modified` of the archive, so a package
  published again is fetched again and the older file is removed) and reads from it, streaming,
  only the class files it maps. No new host: the index is on `mapy.geoportal.gov.pl` and the
  archives on `opendata.geoportal.gov.pl`, both in the adapter's list already.
- **EGiB first.** Buildings: BDOT10k only when EGiB gives none, or fails (then with a warning, and
  the error of EGiB stands when BDOT10k fails too). Land use: only when the parcels with a land
  use cover less than 25 % of the map area (clipped to it; Słubice and the rural samples are
  well above, the cities with empty classes at a few per cent). Then BDOT10k land use is added
  beside EGiB's, and an object whose interior point lies in a parcel that gave a use is left out.
  The share is a threshold set on the samples, not a rule of the data.
- **A failing BDOT10k is a warning**, never a failed run: the map is what it was before.
- **Mapping** (`bdot10k.rs`, one test per rule). Buildings by `kodKst` to the letters of the
  register, then the existing `building_function`; under construction and destroyed buildings are
  left out. Land use: forest and grove `Wald`, tree stand `Gehölz`, grass `Landwirtschaft /
  Grünland`, field, orchard, nursery `Landwirtschaft`, allotments `Sport-, Freizeit- und
  Erholungsfläche / Kleingarten`, still and running water, single and multi-family housing
  `Wohnbaufläche`, industrial land `Industrie- und Gewerbefläche`, paved squares `Platz` (bare
  ground, gravel and crushed stone are not paving), car parks `Platz / Parkplatz`, cemeteries
  `Friedhof`, parks `Sport-, ... / Grünanlage`.
- **Left out:** roads and tracks (`SKDR_L`, `SKJZ_L`, `SKTR_L` are lines without a width; OSM has
  them), bushes, pits, wetlands (they overlap forest and grass), commercial and other built-up
  land (as `Bi` in EGiB).
- **Offline** nothing is asked. The project keeps what an online run stored, as for every ALKIS
  source.

## Licence

The page of the topographic database says the data "is provided free of charge and can be used
for any purpose"; the index service states `Fees: NONE` and `AccessConstraints: NONE`. No
attribution rule is named. The credit line of GUGiK (`© GUGiK, geoportal.gov.pl`) is already
given for every project that has cadastre data, and now also for one filled from BDOT10k.

## Not decided here

- **A box that straddles a municipality without a cadastre service** (some buildings from EGiB,
  none beyond the border) gets no BDOT10k buildings: the trigger is the area, not the parcel.
- **The download estimate** does not name BDOT10k: whether the gap exists is known only after the
  cadastre has answered.
- **`RODZAJ` in words** (`budynekNiemieszkalny`) came back from a municipality in West Pomerania;
  the table of EGiB knows only the letters, so such a building keeps its text.

## Consequences

- A map in a municipality without EGiB has buildings (surveyed or from the orthophoto) instead of
  the OSM fraction, and land use for woods, meadows, water and built-up land.
- The first map of a powiat downloads up to 89 MB more; later maps there read the cache.
- BDOT10k geometry from the orthophoto is less exact than the survey; where its source is not
  EGiB the outlines are digitised.
