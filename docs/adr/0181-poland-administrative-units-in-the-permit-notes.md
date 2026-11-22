# ADR 0181: Administrative units in the Polish permit notes

## Status

Accepted, 22 November 2026. Completes [ADR 0175](0175-poland-permit-notes.md), which left the
administrative units as "not decided here".

## Context

The German notes open with the administrative units of the map area (state, Kreis, Gemeinde from
the BKG VG250). The Polish notes had none, so a person who plans an event could not read from them
which municipality (gmina) to ask. The national boundary register (PRG) of GUGiK, which already
gives the forest districts, also holds the boundaries of the voivodeships (A01), powiats (A02) and
gminas (A03). Measured on 22 November 2026 at Słubice, Kościan and Poznań, User-Agent
`freemapper-research`.

## Decision

- **A third source `prg-granice`** in `crates/adapters/pl/src/permits.rs`, same service, host and
  credit as `prg` (`© GUGiK, geoportal.gov.pl`). No new host; the adapter revision is 5.
- **Three requests, polygons once.** `A03_Granice_gmin` with its polygons (a gmina is 20 to 100 KB),
  then `A02_Granice_powiatow` and `A01_Granice_wojewodztw` with `PROPERTYNAME=JPT_KOD_JE,JPT_NAZWA_`
  and no polygons (the voivodeship polygon alone is 466 KB). The TERYT code of a gmina has seven
  digits; its first four are the powiat and its first two the voivodeship (checked on Słubice
  `0805053`, `0805`, `08` and Kościan `3011011`, `3011032`, `3011`, `30`). Each gmina so gives three
  findings with its own polygon, and the notes sum the shares per unit, so a unit that straddles
  borders has the sum of its gminas and nothing is asked twice. A box without a gmina costs one
  request.
- **Names as the register writes them,** with the kind in front where the register leaves it out:
  `lubuskie` becomes `województwo lubuskie`, `powiat słubicki` stays, and a gmina gets its kind from
  the last digit of the code (`gmina miejska`, `gmina wiejska`, `gmina miejsko-wiejska`, `miasto`,
  `obszar wiejski`, `dzielnica`, `delegatura`). The two Kościan gminas have the same name and differ
  only in this digit, which is why it is shown. A level whose name the register did not answer is
  left out.
- **Engine.** `PermitSection::AdminUnits`, `FindingKind::AdminUnit { level, name, code }` with
  `AdminUnitLevel { Region, County, Municipality }`, `RegionalNotes.admin_units`
  (`Vec<Share<AdminUnitEntry>>`, level by level top down, within a level by share, skipped in the
  JSON when `None`, so every older notes file stays valid). The German `admin` field and the VG250
  types are not touched; the German notes do not change.
- **Texts.** `permits-section-admin-units`, `permits-none-admin-units` and
  `permits-admin-unit-code` (`{ $name } (TERYT { $code })`) in the catalogs of `freemapper-cli`:
  English, German and Polish by hand, the other 18 by machine from the English (unreviewed, as in
  ADR 0175). The section comes first in the regional part.
- **Failure.** The source is read or not. When one of its three requests fails, the notes leave the
  section out, name the source as unreachable (a warning), and keep the other sources. `--offline`
  reads what `data/permits/` holds.
- **Fixtures** are the real answers; the polygons are replaced by the bounding rectangle of the
  feature, as in ADR 0175. The Kościan rectangles overlap, which is why the sums over a straddling
  area are tested with synthetic polygons in `notes.rs`.

## Not decided here

- **Hints that name the gmina.** The Polish hints still say nothing about which office of the
  gmina decides small objects (natural monuments, ecological sites). The gmina is now in the notes,
  the competence is still not checked. Read and decided in [ADR 0183](0183-poland-gmina-as-a-contact-in-the-permit-notes.md).
- **Cities with powiat rights** show a powiat and a gmina of the same name (`powiat Poznań`,
  `gmina miejska Poznań`). That is how the register has them.
- The sub-gmina units (`miasto`, `obszar wiejski`, `dzielnica`) were read from the TERYT convention
  and checked on Kościan (`1`, `2`) and Słubice (`3`) only.

Source: this ADR; `docs/regions/poland-reference.md`.
