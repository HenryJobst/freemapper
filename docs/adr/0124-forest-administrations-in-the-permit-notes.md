# 0124. Forest administrations in the permit notes

Date: 2026-11-14
Status: Accepted

Design: [`2026-11-14-permit-notes-forest-design.md`](../superpowers/specs/2026-11-14-permit-notes-forest-design.md).
Builds on [0117](0117-permit-notes-from-national-sources.md) (permit notes) and
[0120](0120-permit-sources-of-region-adapters.md) (permit sources of region adapters).

## Context

For a forest map the people to ask first are the forest owners and the forest administration. The notes of steps 1
and 2 named the Gemeinde, the Kreis, the protected areas and, in Berlin, the Bezirk, but no forest office. A live
check on 2026-11-14 found three states that publish the areas of their forest offices openly:

- Brandenburg: the Reviere with their Oberförsterei in a GeoServer WFS of the Landesbetrieb Forst Brandenburg
  (`www.brandenburg-forst.de`). It answers GML 3.2 only (`gml32`); JSON and even its own default format are
  refused. GovData lists it under dl-de/by-2-0.
- Berlin: the Distrikte of the Berliner Forsten with Forstamt and Revier, in the Geoportal Berlin like the
  services of step 2 (GeoJSON, dl-de/zero-2-0).
- North Rhine-Westphalia: the Forstbetriebsbezirke with their Regionalforstamt and the parcels of Staatswald as
  shapefiles for the whole state on `www.opengeodata.nrw.de` (about 11 and 6 MB, dl-de/by-2-0). Every row of the
  districts also carries the forester's first and last name, e-mail address, phone and mobile number.

Saxony, Saxony-Anhalt and Mecklenburg-Vorpommern offer their forest offices only as WMS images; Mecklenburg-
Vorpommern's licence is unclear. Rhineland-Palatinate's server answers 403. For the other states no open layer was
found.

## Decision

- **Two kinds of finding.** `ForestUnit { administration, office, unit }` fills the new section `ForestUnits`;
  `StateForest { owner, office }` fills `StateForest`. The source composes the texts as the notes print them
  ("Oberförsterei Barnim", "Forstamt Grunewald", "Regionalforstamt Rhein-Sieg-Erft"); proper names stay German in
  the English notes. The engine sums the shares per entry, as for the Bezirke.
- **Hints for forest maps only.** A forest map gets one hint per office that is more than an edge of the map, with
  its units, and a hint that the forest owners must agree and that several Bundesländer want organised events
  reported to or allowed by the forest authority. Without an office (another state, or the service down) that
  hint says the notes do not name the office. A Revier in Brandenburg covers towns as well, so schoolyard and
  sprint maps get the sections but none of these hints. The state forest is an owner hint on every kind of map.
- **Brandenburg** reads GML through a variant of the step-1 reader that shares its paging, its 50 MB cap and its
  page limit. `www.brandenburg-forst.de` joins the hosts of Brandenburg's manifest.
- **Berlin** gets a sixth source.
- **North Rhine-Westphalia** downloads both zips on every online fetch (50 MB cap per zip, 100 MB per file inside),
  keeps the records whose box meets the map box and stores them as GeoJSON in WGS 84.
- **Allow-lists of fields.** Each source keeps only the fields it reads (Brandenburg: the numbers and names of the
  Oberförsterei and the Revier; NRW: `rfa`, `fbb` and `forstamt`). Everything else, the forester's name and
  contact data above all, is dropped before the features are stored, so `data/permits/` never holds it.
- **German wording.** The owner of the Staatswald is "Bundesland Nordrhein-Westfalen"; the data give no name of
  their own, so the notes follow the rule for German text.

## Consequences

- Forest maps in Brandenburg, Berlin and North Rhine-Westphalia name the forest office; every other forest map
  says that the owners must agree and that the office is not named.
- An online run in North Rhine-Westphalia downloads about 17 MB more. The stored features are small, and an
  offline run downloads nothing. A cache of the two zips across projects is left for later.
- A source that fails leaves out its section, as in step 2; the forest hint then falls back to the general one.
- The WMS-only states need a GetFeatureInfo reader; that, forest functions (Erholungswald, Naturwald),
  Wasserschutzgebiete and Hamburg's Bezirke are later steps.
