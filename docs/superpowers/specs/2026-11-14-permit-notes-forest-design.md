# Permit notes, step 3: forest administrations (design)

Status: design, 2026-11-14. Builds on step 1 (ADR 0117) and step 2 (ADR 0120, the `PermitSource` hook). The
user asked for step 3 on 2026-11-14 and left the choice between the forest administrations and Hamburg's Bezirke
to Claude; Claude took the forest administrations, because since v0.19.0 every state has a forest example and
the notes say nothing yet about who runs the forest. Live check on 2026-11-14 (about 25 requests, fixed program
User-Agent): see "Service facts".

## Intent

For a forest map the first people to ask are the forest owner and the forest administration. Step 1 names the
Gemeinde, the Kreis and the protected areas, but no forest office. Three states publish their forest offices with
their areas under an open licence; step 3 brings them into the notes, adds the share of state forest where the data
show it, and gives every forest map a hint about the forest owner and the forest office, also where no data exist.

## Goals

1. When the map area touches Brandenburg, Berlin or North Rhine-Westphalia, the notes get a section "forest
   administration" with one line per office and unit and its share of the map area:
   - Brandenburg: Landesbetrieb Forst Brandenburg, Oberförsterei and Revier (`IFUEK:AM.ForestManagementArea`).
   - Berlin: Berliner Forsten, Forstamt and Revier (`forstverwalt:b_forstverwalt`, Distrikte summed per Revier).
   - North Rhine-Westphalia: Landesbetrieb Wald und Holz NRW, Regionalforstamt and Forstbetriebsbezirk (the
     shapefile `forstbetriebsbezirke_nrw`).
2. North Rhine-Westphalia also gives the share of Staatswald (`landeseigener_forstbetrieb_staatswald`) as an
   owner line "Bundesland Nordrhein-Westfalen" in the existing owners section, with the owner hint of step 2.
3. Hints, for forest maps only (`MapKind::Forest`):
   - with forest offices on the map (more than an edge): the office, its administration and the units, and that
     an event in the forest needs the forest owner's consent, which the office can help with;
   - without (every other state, or a service that is down): a general hint on the forest owner's consent and
     that several Bundesländer want organised events in the forest reported to or allowed by the forest
     authority, and that these notes do not name the office there.
4. Each source gets its line in the source table with the publisher's credit and licence.
5. Offline, stored data give the same text, as in steps 1 and 2.

## Non-goals

- Saxony, Saxony-Anhalt and Mecklenburg-Vorpommern: their forest offices exist only as WMS (an image, no
  areas); a GetFeatureInfo reader is a later step. MV's licence is unclear.
- Rhineland-Palatinate: `gis.wald.rlp.de` answers 403 to the program; not worked around.
- Hesse, Lower Saxony, Schleswig-Holstein, Hamburg, Bavaria, Baden-Württemberg, Saarland, Bremen: no open layer
  of forest offices was found (step 1 research); they get the general hint.
- Forest functions (Erholungswald, Naturwald, Totalreservat), Wasserschutzgebiete, hunting: later steps.
- Private or corporate forest owners: not open anywhere.
- Hamburg's Bezirke: a later step.

## Anti-goals

- Names, e-mail addresses or phone numbers of people. The NRW Forstbetriebsbezirke carry the forester's first
  and last name, e-mail, phone and mobile number in every row. The source keeps an allow-list of fields
  (`rfa`, `fbb`) and drops everything else before the features are stored, so `data/permits/` never holds them.
  The same allow-list rule holds for Brandenburg (`foa_name`, `rev_name`, and their numbers) and the NRW
  Staatswald (`forstamt`; not the parcel label).
- Requests to the forest services that do not go through the host rules of the state's adapter.
- A forest hint on schoolyard and sprint maps: a Revier in Brandenburg covers towns as well, and a hint there
  would be noise. The section with the facts still appears on every map kind.

## Constraints

As steps 1 and 2: no change to `freemapper.toml`, `ATTRIBUTION.txt`, the map, PDFs or golden files; German says
"Bundesland"; DE and EN Fluent texts; one ADR (next free number); changelog DE and EN under Unveröffentlicht /
Unreleased; manual DE and EN, README one sentence at most, AGENTS.md where the contacted hosts and the download
are listed.

## Service facts (checked live 2026-11-14)

- **Brandenburg**: GeoServer WFS 2.0.0 `https://www.brandenburg-forst.de/geoserver/IFUEK/ows`, one type
  `IFUEK:AM.ForestManagementArea`. No JSON output (`application/json` is refused); `outputFormat=gml32` works,
  the default output format does not. A WGS 84 box written latitude first with the CRS suffix works;
  geometries come as `gml:Polygon` in EPSG:25833. Properties `foa_nr`, `foa_name` (Oberförsterei, e.g.
  "Barnim"), `rev_nr`, `rev_name` (Revier, e.g. "Werneuchen"), `adresse` (a number). Paging with `next`.
  Strausberg box: 4 Reviere, 138 KB for two. Capabilities: Fees and AccessConstraints "NONE"; GovData lists
  the Reviere under dl-de/by-2-0, publisher Landesbetrieb Forst Brandenburg (LFB).
  The host is not yet among Brandenburg's manifest hosts.
- **Berlin**: GeoServer WFS `https://gdi.berlin.de/services/wfs/forstverwalt`, type
  `forstverwalt:b_forstverwalt`, GeoJSON like the step-2 services. Features are Distrikte (MultiPolygon) with
  `foa_name` (Forstamt, e.g. "Grunewald"), `rev_name` (Revier, e.g. "Dachsberg"), `land`, `distrikt`,
  `best_dist`; 606 in a 5 by 7 km box of the Grunewald. Licence as the other Berlin services (dl-de/zero-2-0,
  verbatim text in `berlin_permits::LICENCE`).
- **North Rhine-Westphalia**: downloads on `https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/`
  (a host of NRW's manifest): `forstbetriebsbezirke_nrw_EPSG25832_Shape.zip` (10.7 MB, 299 rows, fields `rfa`,
  `fbb` plus the personal fields above), `landeseigener_forstbetrieb_staatswald_EPSG25832_Shape.zip` (5.9 MB,
  22 999 parcels, fields `forstamt`, `gemeinde`, `gemarkung1`, `label`). UTF-8 (`.cpg`), EPSG:25832. `rfa`
  values are the names of 14 Regionalforstämter plus "Nationalparkforstamt Eifel" and "Arnsberger Wald" (the
  Lehr- und Versuchsforstamt). GovData: dl-de/by-2-0, publisher Landesbetrieb Wald und Holz Nordrhein-Westfalen.

## Decided how (Claude's calls)

- **API**: `PermitSection::ForestUnits`; `FindingKind::ForestUnit { administration, office, unit }`, all text
  as the notes print them ("Landesbetrieb Forst Brandenburg", "Oberförsterei Barnim", "Revier Werneuchen"). The
  source composes the office word: Brandenburg "Oberförsterei X", Berlin "Forstamt X", NRW "Regionalforstamt X"
  unless the name already says "forstamt", and "Arnsberger Wald" becomes "Lehr- und Versuchsforstamt Arnsberger
  Wald". Proper names stay German in the English notes.
- **Engine**: `regional.rs` sums the shares per (administration, office, unit), as for Bezirke, and the hints take
  the offices that are more than an edge, as for the Bezirk hints.
- **Brandenburg**: a GML variant of the step-1 reader (`geojson_wfs`), sharing its paging, its 50 MB cap and its
  page limit, parsing pages with `wfs::parse_gml`; the source turns EPSG:25833 into WGS 84 and keeps only the
  allowed properties. `www.brandenburg-forst.de` joins Brandenburg's manifest hosts.
- **Berlin**: a sixth source of `berlin_permits`.
- **NRW**: two sources (`forstbetriebsbezirke`, `staatswald`) that download the zip (50 MB cap), read `.shp` and
  `.dbf` with `shapefile::polygons_in` cut to the map box in EPSG:25832, turn the rings into WGS 84 and return
  GeoJSON features with the allowed fields only. About 17 MB per online run; the stored features are small.
- **Failure**: as step 2: a source that fails is "unreachable" in the table and its section is left out; the
  forest hint then falls back to the general one.

## Left to the builder

Module names, the JSON field names of the new section (schema stays 1: the part is optional), Fluent wording within
the rules above, trimmed fixtures recorded from the live answers with the personal fields replaced by made-up
values in the one fixture that proves they are dropped.
