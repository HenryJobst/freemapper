# 0117. Permit notes from national sources

Date: 2026-11-13
Status: Accepted

Design: [`2026-11-13-permit-notes-design.md`](../superpowers/specs/2026-11-13-permit-notes-design.md); plan beside
it. Research: [`waldschutz-forstbehoerden-recherche.md`](../research/wald-und-sprint-behoerden/waldschutz-forstbehoerden-recherche.md).
Builds on [0106](0106-requests-go-to-public-addresses-and-declared-hosts.md) (host rules).

## Context

Whoever holds an orienteering event or a training on a map asks someone first: the municipality for streets and
squares, the lower nature conservation authority when a protected area is involved, the operator of a school, a
park or a palace garden. Finding out who that is takes hours in portals and maps. freemapper knows the map area
exactly, so it can name the administrative units, protected areas and known operators the area touches, and whom
that usually means asking.

The README listed this as planned ("Genehmigungen und Ansprechpartner"). After v0.16.0 the user chose it as the
next project, and chose to begin with a base that works the same in all twelve supported states: protected areas
from the BfN, Land, Kreis and Gemeinde from the BKG (VG250), and operators and owners from OpenStreetMap. Forest
administrations and the Berlin sprint package come in later steps. The user handed the detailed calls to Claude;
the design records them.

A live check of the two services on 2026-11-13 (about 45 requests) found what the code must get right: both want
the box latitude first, and a wrong axis order gives HTTP 200 with an empty collection; the BfN answers only with
`outputFormat=GEOJSON`; VG250 needs the CRS at the end of the box and sends 250 000 features unless `count` is
set; both send whole polygons of 25 to 700 KB.

## Decision

- **A document next to the map.** `permits.md` (readable, in the program language) and `permits.json` (the same
  content, `"schema": 1`) go into the project folder. The `.omap`, the candidates, the PDFs and the preview do not
  change; the notes are not map objects, so they are no content group either.
- **Opt-in.** Nothing asks the BfN or the BKG unless the person asks for permit notes: `freemapper permits <dir>`,
  `freemapper run <dir> --permits`, the MCP tool `permits_for_area`, the parameter `permits` of `start_pipeline`
  and `open_area_selector`, and the box "Hinweise zu Genehmigungen" on the area page (off by default). `export`
  gets no switch: it stays a step without network.
- **`freemapper.toml` does not change.** No section, no saved switch. What was fetched is recorded in
  `permits.json` and in `data/permits/sources.json`. The shared contract of the suite stays untouched.
- **`ATTRIBUTION.txt` does not change.** It lists the sources of the map, and nothing of the BfN or the BKG goes
  onto the map. The notes carry the credit lines of the sources they used, in a table at their end.
- **Host policy of the national sources.** The two services go through `PolicyFetcher` with a fixed list of their
  own, `UrlPolicy::national()`: `geodienste.bfn.de` and `sgx.geodatenzentrum.de`. `adapters allow-host` does not
  apply to it; it is for built-in state adapters. The fixed User-Agent is the only identification.
- **One GeoJSON WFS reader** in `freemapper-sources` for both services: latitude first, pages of 500, a cap of
  50 MB per answer while it streams in. The BfN is asked for its nine types (Nationalpark, Nationales
  Naturmonument, NSG, FFH, Vogelschutzgebiet, Biosphärenreservat and its zones, LSG, Naturpark), VG250 for Land,
  Kreis, Verwaltungsgemeinschaft and Gemeinde: 13 requests for an area. The first type that fails stops its
  source, and the source counts as unreachable as a whole, since half a list of protected areas misleads.
- **Shares on the map polygon.** The share of a unit or a protected area is the area of (map area ∩ geometry)
  divided by the area of the map area, computed in the project's UTM zone. The request uses the box of the map
  area. Percent without decimals, "< 1 %" below one; an entry under 0.5 % stands in its own list "touched at the
  edge", and for VG250 a sentence says this can come from the generalisation.
- **Order.** Administrative units top down (Land, Kreis, Amt or Verwaltungsgemeinschaft, Gemeinde). Protected
  areas by type in the order Nationalpark, Nationales Naturmonument, NSG, FFH, Vogelschutzgebiet,
  Biosphärenreservat (each zone after its reserve), LSG, Naturpark; within a type by share. A `vg250_vwg` entry
  that is an amtsfreie or verbandsfreie Gemeinde (its kind contains "frei") is left out: it only repeats the
  Gemeinde. The name does not decide: an Amt, a Verbandsgemeinde or a Samtgemeinde often carries the name of one
  of its Gemeinden (Amt Brück, Stadt Brück), and it stays.
- **OSM selection.** Areas only (closed ways and multipolygons that meet the map area), read from the state's
  Geofabrik extract already cached; `permits` downloads no extract. Protected areas: `boundary=protected_area`,
  `leisure=nature_reserve`, `boundary=national_park`. `operator` and `owner` only of `leisure=park|garden|
  sports_centre|pitch|stadium`, `amenity=school|university|college|kindergarten|place_of_worship|hospital`,
  `landuse=forest|railway|cemetery|religious|military|education`, `natural=wood`,
  `tourism=attraction|zoo|theme_park` and `historic=*`. Nothing else, so no residential land, houses or private
  gardens, and no `contact:*`, `email` or `phone`. A `leisure=garden` with `garden:type=residential|private` or
  `access=private|no` is left out, and so is a `historic=*` object with
  `building=house|residential|detached|semidetached_house|farm|apartments`: their owner is often a person. An OSM protected area is left out only when one BfN area and it
  cover each other mostly (overlap / larger area ≥ 0.5). A small OSM area inside a large LSG or Naturpark, such
  as a Geschützter Landschaftsbestandteil, stays. The findings are stored in `data/permits/osm.geojson`,
  because the extract leaves the cache after 7 days.
- **Hints are cautious text templates.** Chosen by finding and map kind in the Fluent files, they name the kind
  of office and the unit, never a person or an address, and say "usually": "die untere Naturschutzbehörde (meist
  der Landkreis Märkisch-Oderland)". They are exact only where VG250 settles it: a kreisfreie Stadt is itself the
  authority; Berlin and Hamburg get a hint that a Bezirksamt or a Landesbehörde is in charge and that the Bezirk
  is not determined yet; an Amt or a Verwaltungsgemeinschaft is named in the hint on the Ordnungsamt. The school
  hint comes on schoolyard maps only. A last hint says that owners cannot be found from open data and how one
  asks the Grundbuchamt. There is no table of competences per state in this step.
- **Disclaimer first.** Every `permits.md` starts with "Hinweis, keine Rechtsauskunft" and says once that the BfN
  merges the data of the states once a year and marks them as not suitable for planning. The access constraint
  of the BfN, "Nicht für Planungszwecke geeignet.", is quoted verbatim in both languages.
- **Credit lines.** BKG verbatim from the capabilities: "© BKG (<year>) dl-de/by-2-0, Datenquellen:
  https://sg.geodatenzentrum.de/web_public/gdz/datenquellen/datenquellen_vg_nuts.pdf". BfN: "© Bundesamt für
  Naturschutz (BfN) (<year>), Schutzgebiete in Deutschland; Nutzungsbestimmungen für die Bereitstellung von
  Geodaten des Bundes (GeoNutzV). Nicht für Planungszwecke geeignet." The BfN gives no credit line of its own;
  this wording is freemapper's, built from the name of the service, its fees text (GeoNutzV) and its access
  constraint. OSM: "© OpenStreetMap contributors, ODbL", the same constant as on the map. `<year>` is the year
  of the fetch.
- **Storage and offline.** The raw answers go into `data/permits/` of the project (`vg250-<level>.geojson`,
  `bfn-<kind>.geojson`, `osm.geojson`, `sources.json` with the box, state, date and count per source), not into
  the shared cache. A new fetch replaces them. `--offline` (`offline` in MCP) reads them and gives the same text;
  stored data of another map area is not used. An online run reads its own files back, so online and offline
  cannot differ.
- **Failures.** A source that cannot be read is named in the notes and in a warning; its section says the data
  were not read instead of "none". `freemapper permits` exits with 1 only when no source at all could be read.
  In `run --permits` and `start_pipeline` any failure of the notes is a warning, and the map run keeps its exit
  code and state.
- **Lock and duration.** `permits` takes the project folder lock like `run`. On a Brandenburg project with a
  cached extract (283 MB) the notes took about 26 s in a release build: 7 s for the 13 requests, 19 s for the
  pass over the extract. The design said a tool over 20 s becomes a job, so `permits_for_area` returns a job id;
  `project_status` of the finished job carries the text of `permits.md` as `permits_md`. `project_status` lists
  the written files (`files`) for every job.
- **Daily update check.** `permits` runs it like `fetch`; with `--offline` it does not.

## Consequences

- One more document per project when asked for, plus the stored answers in `data/permits/`. Maps, golden outputs
  and `freemapper.toml` are unchanged.
- Known gaps, each said in the notes:
  - The extract reader keeps a way only when one of its nodes lies in the box. An OSM area that encloses the
    whole map area without a corner inside it is missed. Both OSM sections say so; the BfN and VG250 parts are not
    affected. A test of the box against all nodes of a way would close it.
  - VG250 is generalised to 1:250 000. A unit may touch the edge of a small area only because of that; such units
    stand in the edge list with a sentence that says it.
  - The BfN merges the data of the states once a year. A protected area set up or changed since then is missing
    or out of date. Each protected area shows its data year.
  - Berlin and Hamburg are one unit each in VG250. The Bezirk is not determined yet; the hint says so.
- `run --permits` reads the extract a second time, about 20 s on a large state. Merging the permit filter into
  the candidate pass would save it.
- The notes depend on two services the project cannot control. A changed axis order would give empty answers
  without an error; one ignored live test per service checks Strausberg against known areas.
- The amtsfrei rule for `vg250_vwg` reads only the kind (`bez` contains "frei"). It was written from the `bez`
  values seen (Sanssouci, Brück) and not checked against every state.
- An OSM protected area and a BfN area count as the same only when their footprints mostly match. The same area
  mapped very differently in OSM can therefore stand in both sections.

## Later steps

- Forest administrations (Forstamt, Revier, Staatswald) of Brandenburg, Berlin and North Rhine-Westphalia, then
  Mecklenburg-Vorpommern, Saxony and Saxony-Anhalt (WMS, needs GetFeatureInfo).
- The Berlin sprint package: Bezirke, Grünanlagen, Liegenschaftsplan, Denkmale; Bezirke of Hamburg.
- A declarative `[[info_layer]]` section in the adapter manifest, once several state layers exist.
- Water protection areas and monuments.
