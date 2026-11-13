# Permit notes for a map area: design

Status: design, 2026-11-13. Step 1 of the topic "permits and contacts" (README paragraph "Geplant:
Genehmigungen und Ansprechpartner"). Research: `docs/research/wald-und-sprint-behoerden/waldschutz-forstbehoerden-recherche.md`.
Live check of the two services on 2026-11-13: see "Service facts" below.

## Intent

Anyone who organises an orienteering event or training on a map has to ask someone first: the
municipality for streets and squares, the lower nature conservation authority when a protected area
is involved, the operator of a school, park or palace garden. Finding out who is in charge takes
hours with portals and maps. freemapper already knows the map area exactly. It can therefore say,
for that area, which administrative units, protected areas and known operators it touches, and whom
that usually means asking.

The user chose this as the next project after v0.16.0, and chose to begin with a base that works the
same in all twelve supported Bundesländer: protected areas from the BfN, Gemeinde/Kreis/Land from
the BKG (VG250), and operators/owners from OpenStreetMap. Forest administrations and the Berlin sprint
package (Bezirke, Grünanlagen, Liegenschaftsplan, Denkmale) come in later steps.

## Goals

1. For a project, write `permits.md` (readable, in the language of `--lang`) and `permits.json`
   (machine-readable, same content) into the project folder.
2. Content, for the map area:
   - administrative units: Land, Kreis (or kreisfreie Stadt), Verwaltungsgemeinschaft/Amt where it
     exists, Gemeinde, each with its share of the map area in percent;
   - protected areas of all nine BfN types, each with type, name, share of the map area, legal
     date and title, link to the legal text where the BfN gives one, data year;
   - from OSM: protected areas the BfN does not list (`boundary=protected_area`,
     `leisure=nature_reserve`, `boundary=national_park`) and the `operator`/`owner` of public-kind
     areas (list under "OSM selection" below), grouped by name with the kinds of objects;
   - for each finding, a short generic hint whom this usually concerns (for example "NSG → untere
     Naturschutzbehörde des Kreises Märkisch-Oderland; im NSG gilt meist ein Wegegebot"), worded as
     a hint and never as legal advice;
   - per source: data stand, fetch date, licence and the credit line the licence asks for.
3. Commands: `freemapper permits <dir>` fetches what it needs and writes the notes;
   `freemapper run <dir> --permits` does the same after the map. There is no `export --permits`:
   export stays a step without network. MCP: a tool `permits_for_area` (`dir`, optional `offline`)
   that writes the files and returns the Markdown text, and a parameter `permits` on
   `start_pipeline` and `open_area_selector`. The area page gets a checkbox "Hinweise zu
   Genehmigungen" next to the other outputs (PDF, OCAD), off by default.
4. Works offline from what was fetched before (`--offline`), with the same text: the OSM findings are
   stored too (`data/permits/osm.geojson`), because the Geofabrik extract leaves the cache after 7
   days.
5. A failing service never fails a map run. The notes then say which source was not reachable;
   `freemapper permits` alone exits with 1 only when no source at all could be read.

## Non-goals (this step)

- Forest administrations (Forstamt, Revier, Staatswald), Waldfunktionen, water protection areas,
  monuments, Berlin/Hamburg Bezirke, Grünanlagen, Liegenschaftsplan: later steps.
- A declarative `[[info_layer]]` section in the adapter manifest (research C3). It comes with the
  first state layers, when there are several of them to describe; step 1 has two national sources
  in code. YAGNI.
- Anything drawn on the map or in the candidates. The notes are a document next to the map; the
  `.omap`, the candidates, the PDFs and the preview do not change.
- A page in the PDF (research C2.4).
- Owners of land. ALKIS owner data is not open. The notes say so with the honest text of research B2.

## Anti-goals (a failure even if it works)

- Anything that reads as legal advice or as a permit. Every file begins with the disclaimer.
- Names of natural persons, e-mail addresses or phone numbers of anyone. OSM `operator`/`owner` is
  taken only from the public-kind objects listed below, never from residential land, houses or
  private gardens, and no `contact:*`, `email`, `phone` tags are read.
- Silent gaps. A source that could not be read, a Bundesland whose data the BfN has not merged yet,
  or the generalisation of VG250 at the edge of the area must be said in the notes.
- Requests to hosts outside a fixed list, or any request carrying personal data. The fixed
  User-Agent of freemapper is the only identification.
- New hosts asked by default. The notes are opt-in: nothing asks the BfN or the BKG unless the
  person asked for permit notes.

## Constraints

- Source code English, user texts German and English (Fluent files of the CLI crate), German says
  "Bundesland". Terms go into `docs/glossary.md` (Genehmigungshinweise / permit notes, Schutzgebiet /
  protected area, Verwaltungseinheit / administrative unit).
- `freemapper.toml` does not change: no new section, no saved switch. The record of what was fetched
  lives in `permits.json` (and the stored source files). This keeps the shared contract of the
  suite (freemapper-core `freemapper-contract`) untouched.
- Host rules: the two hosts go through `PolicyFetcher` with a policy of their own ("national
  sources": `geodienste.bfn.de`, `sgx.geodatenzentrum.de`), not through a bare `HttpFetcher`.
  `allow-host` does not apply to it (it is for built-in state adapters).
- No new heavy dependencies. GeoJSON parsing and polygon intersection with what the workspace already
  uses (`serde_json`, `geo`).
- One ADR (next free number, 0117). Changelog DE+EN. Manual DE+EN (new section), README DE+EN (the
  "Geplant" paragraph becomes a description of what exists and what is still planned), AGENTS.md
  (command, MCP tool, files in the project folder).
- Work goes in as a GitHub pull request; CI on Linux, label `all-systems` only if the change touches
  paths or processes that differ per system.

## Service facts (checked live 2026-11-13, about 45 requests)

BfN `https://geodienste.bfn.de/ogc/wfs/schutzgebiet`, WFS 2.0.0:
- Types (`schutzgebiet:` prefix): `Naturschutzgebiete`, `Landschaftsschutzgebiete`,
  `Fauna_Flora_Habitat_Gebiete`, `Vogelschutzgebiete`, `Nationalparke`, `Naturparke`,
  `Biosphaerenreservate`, `Biosphaerenreservate_Zonierung`, `Nationale_Naturmonumente`.
- Working request: `outputFormat=GEOJSON` (not `application/json`), `srsName=urn:ogc:def:crs:EPSG::4326`,
  `bbox=<minLat>,<minLon>,<maxLat>,<maxLon>` (latitude first), `count`, `startIndex`, `resultType=hits`.
- Wrong axis order returns HTTP 200 with an empty collection. Only a test against a known area finds
  this: Strausberg (13.83,52.53,13.86,52.545 lon/lat) gives 2 NSG, 1 FFH, 2 LSG; Sanssouci
  (13.015,52.395,13.045,52.410) gives 1 LSG "Potsdamer Havelseengebiet".
- Attributes: `NAME`, `BFN_ID`, `BL`, `ID`, `LEG_DATE` (string like `8/1/1966`), `LEG_LINK`,
  `LEG_TITEL`, `IUCN_KAT`, `JAHR`, `FLAECHE` (ha), `STATUS`, plus a type code (`natda_id`/`CDDA_CODE`,
  `SITECODE`/`LINK` for FFH and SPA). No field for the responsible authority. Empty values are
  sometimes a single space.
- Geometries are whole polygons, 25–200 KB each, a nationwide Naturmonument 706 KB. `propertyName`
  without geometry returns broken JSON; do not use it.
- Licence: Fees "Nutzungsbestimmungen für die Bereitstellung von Geodaten des Bundes" (GeoNutzV);
  AccessConstraints "Nicht für Planungszwecke geeignet." Both go into the notes verbatim.

BKG VG250 `https://sgx.geodatenzentrum.de/wfs_vg250`, WFS 2.0.0 (GeoServer):
- Types `vg250:vg250_lan`, `vg250_krs`, `vg250_vwg`, `vg250_gem` (also `rbz`, `sta`, `li`, `pk`).
- Working request: `outputFormat=application/json`, `srsName=urn:ogc:def:crs:EPSG::4326`,
  `bbox=<minLat>,<minLon>,<maxLat>,<maxLon>,urn:ogc:def:crs:EPSG::4326` (latitude first, CRS suffix
  required); a 25833 bbox with matching suffix also works. Default count is 250000: always send `count`.
- Attributes: `gen` (name), `bez` (kind, e.g. "Stadt", "Landkreis", "Amt"), `ags`, `ars`, `nuts`,
  `beginn`, among others.
- Generalised to 1:250 000: a bbox around Sanssouci returns three Kreise. Shares are computed on the
  map area polygon, and units below a small share are listed as "touches the edge, may be an effect
  of the generalisation".
- Credit line, verbatim from the capabilities: "© BKG (Jahr des letzten Datenbezugs) dl-de/by-2-0,
  Datenquellen: https://sg.geodatenzentrum.de/web_public/gdz/datenquellen/datenquellen_vg_nuts.pdf".

## Decided how (Claude's calls, the user handed these over)

- **Opt-in, not part of the content profiles.** The notes are not map objects, so they are not a
  content group. They are written when asked for: `permits` command, `--permits`, MCP.
- **Map area, not bbox.** Shares are the area of (map area ∩ unit) / area of the map area, computed
  in the project's UTM zone. Requests use the bbox of the map area in EPSG:4326, lat/lon order.
- **Storage.** The raw responses go into `data/permits/` of the project (`bfn-<type>.geojson`,
  `vg250-<type>.geojson`), small and per project, not into the shared cache. `permits.json` records
  fetch date, count and HTTP outcome per source. A new fetch replaces them; `--offline` reads them.
- **OSM from the extract already cached.** A second pass over the state's Geofabrik extract with a
  permits filter, no new download. Without a cached extract (map never fetched), the OSM part says
  "not read" and the rest still works. `freemapper permits` does not download an OSM extract.
- **OSM selection** (areas only: closed ways and multipolygons that intersect the map area):
  `boundary=protected_area`, `leisure=nature_reserve`, `boundary=national_park` (with `name`,
  `protect_class`, `protection_title`, `operator`); and `operator`/`owner` of `leisure=park|garden|
  sports_centre|pitch|stadium`, `amenity=school|university|college|kindergarten|place_of_worship|
  hospital`, `landuse=forest|railway|cemetery|religious|military|education`, `natural=wood`,
  `tourism=attraction|zoo|theme_park`, `historic=*`. Nothing else.
- **Hints are text templates**, chosen by finding and map kind, in the Fluent files. They name the
  kind of office and the unit, never a person or an address. They are deliberately cautious where
  the competence differs between Bundesländer: "die untere Naturschutzbehörde (meist der Landkreis
  Märkisch-Oderland)". Exact only where VG250 settles it: a kreisfreie Stadt (`bez` "Kreisfreie
  Stadt", or Land = Gemeinde as in Berlin and Hamburg) is itself named as the authority; in Berlin
  and Hamburg the hint adds that the Bezirksamt or a Landesbehörde is in charge and that the Bezirk
  is not determined yet. Where an Amt or a Verwaltungsgemeinschaft exists (`vg250_vwg`), the
  Ordnungsamt hint names it ("Amt Märkische Schweiz, für die Gemeinde Buckow"), also when the Amt
  carries the name of the Gemeinde ("Amt Brück"). A `vg250_vwg` entry is left out only when its
  `bez` contains "frei" (Amtsfreie or Verbandsfreie Gemeinde; fix round 1). No per-Bundesland
  competence table in this step.
- **The yearly merge of the BfN** is said once per notes file, not guessed per Bundesland: "Das BfN
  führt die Daten der Länder einmal im Jahr zusammen und weist sie als nicht für Planungszwecke
  geeignet aus; maßgeblich sind die Landesbehörden." Each protected area shows its `JAHR`.
- **OSM protected areas against BfN ones.** An OSM protected area is left out only when one BfN
  area and the OSM area cover each other mostly (area of the overlap / area of the larger of the
  two ≥ 0.5; fix round 1). A small OSM area inside a large LSG or Naturpark stays, and so does a
  large OSM area around a small BfN one. Those that remain go into an own section "Weitere Schutzgebiete aus OpenStreetMap" after the BfN types,
  with `protection_title` or `protect_class` as their kind.
- **Credit.** The notes carry the credit lines of the sources they used. `ATTRIBUTION.txt` does not
  change: it lists the sources of the map, and nothing of the BfN or the BKG goes onto the map.
  Lines: BKG verbatim as above; BfN "© Bundesamt für Naturschutz (BfN) (Jahr des Abrufs),
  Schutzgebiete in Deutschland; Nutzungsbestimmungen für die Bereitstellung von Geodaten des Bundes
  (GeoNutzV). Nicht für Planungszwecke geeignet." (Claude's wording, the BfN gives none of its own;
  the ADR says so); OSM "© OpenStreetMap contributors, ODbL" as on the map.
- **Lock and duration.** `permits` takes the project folder lock like `run`. `permits_for_area`
  answers synchronously. The builder measures it on a Brandenburg project with a cached extract; if
  it takes longer than 20 seconds, the tool becomes a job like `start_pipeline` (job id, then
  `project_status`), and the second call returns the stored text.
- **Shares and rounding.** Percent with no decimals, "< 1 %" below one. A unit or protected area that
  touches the area with less than 0.5 % is listed in a separate line "am Rand berührt".
- **Order.** Administrative units top-down (Land, Kreis, Amt, Gemeinde). Protected areas by type in
  the order Nationalpark, Nationales Naturmonument, NSG, FFH, Vogelschutzgebiet, Biosphärenreservat
  (with zone), LSG, Naturpark; within a type by share, descending.

## Left to the builder

- Module layout (suggested: `freemapper-sources` gets a small GeoJSON WFS reader for these two
  services; the `freemapper` crate gets `permits/` for selection, shares and the document model; the
  CLI crate gets the command, the Markdown writer with Fluent texts, and the MCP tool).
- The exact JSON shape (versioned, `"schema": 1`), as long as it holds everything in Goal 2.
- Timeouts and a response size cap per request (a sensible cap such as 50 MB, since whole polygons
  come back).
- Test fixtures: small recorded responses for unit tests; one ignored live test per service that
  checks the axis order against Strausberg.

## Open for later steps

- Forest administrations of BB, BE, NRW, then MV/SN/ST (WMS, needs GetFeatureInfo).
- Berlin sprint package; Bezirke of Berlin and Hamburg (VG250 knows only the whole city).
- Declarative `[[info_layer]]` in the manifest once several state layers exist.
- Water protection areas, monuments.
