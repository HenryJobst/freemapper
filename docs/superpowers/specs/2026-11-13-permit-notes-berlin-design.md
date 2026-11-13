# Permit notes, step 2: the Berlin sprint package (design)

Status: design, 2026-11-13. Builds on step 1 (ADR 0117, `2026-11-13-permit-notes-design.md`). The user chose
this step on 2026-11-13. Live check of the five Berlin services on 2026-11-13 (24 requests): see "Service facts".

## Intent

For a map area in Berlin, step 1 can say only "Land Berlin" and that the Bezirk is not determined yet. For a sprint
map that is the gap that matters: the Bezirksamt is in charge of streets, parks and monuments, and many parks
belong to the Land. Berlin publishes all of this openly (Geoportal Berlin, dl-de/zero-2-0). Step 2 brings it into
the permit notes when the map area lies in Berlin.

## Goals

1. When the map area touches Berlin, the permit notes add:
   - the Bezirk(e) with their share of the map area (`alkis_bezirke`);
   - the Grünanlagen (public green spaces) with name, Bezirk, dedication (`widmung`) and share; playgrounds are
     counted, not listed;
   - the share of the map area on land owned by the Land Berlin and by the Bund (`liegenschaftsplan`, owner from
     `sch`), as one line per owner, never per parcel;
   - the monuments (`denkmale`): the number per kind (Baudenkmal, Gesamtanlage, Ensemble, Gartendenkmal) with
     the links into the Denkmaldatenbank Berlin, at most 20 links, the rest counted;
   - Berlin's protected objects that the BfN does not have: Naturdenkmale as points (`a_nd`, with the German name
     of the tree or object where given) and as areas (`e_fnd`), geschützte Landschaftsbestandteile (`g_glb`), and
     `f_fspr` if it has features, each with its legal citation (`zitat`) where given.
2. Hints that use the Bezirk: Ordnungsamt and Straßen- und Grünflächenamt of the Bezirksamt X; untere
   Denkmalschutzbehörde of the Bezirksamt X when a monument is on the map; untere Naturschutzbehörde of the
   Bezirksamt X when a Naturdenkmal or geschützter Landschaftsbestandteil is on the map. They replace the step-1
   hint "the Bezirk is not determined yet" for Berlin. Hamburg keeps that hint.
3. The owner hint says what the open data shows ("x % of the map area is land of the Land Berlin, y % of the
   Bund") and keeps the honest text for the rest.
4. The source table gets one line for the Geoportal Berlin with its licence text verbatim and the note from the
   capabilities that only the analogue official maps are legally binding.
5. Offline, stored data gives the same text, as in step 1.

## Non-goals

- Hamburg's Bezirke, forest administrations, other Länder: later steps.
- Berlin's own NSG/LSG/FFH/SPA/Naturpark layers. The BfN keeps those for all Bundesländer the same way
  (Claude's call: one source per kind, no duplicates to reconcile; the Berlin layers had no hit at Viktoriapark).
- Parcel numbers, parcel lists, the `art` code of the Liegenschaftsplan (no codelist found).
- Spielplätze as a list; the Denkmaldatenbank texts behind the links.
- A declarative `[[info_layer]]` in manifests. Berlin is a code adapter; this step adds a code hook.

## Anti-goals

- Requests to `gdi.berlin.de` that do not go through the Berlin adapter's own host rules.
- Asking Berlin's services for a map area that does not touch Berlin, or without permit notes being asked for.
- Names of natural persons. None of the layers carries any; the owner line names only "Land Berlin" and
  "Bundesrepublik Deutschland" as the data writes them.

## Constraints

As step 1 (Global Constraints of the step-1 plan apply): no change to `freemapper.toml`, `ATTRIBUTION.txt`, the
map, PDFs or golden files; German says "Bundesland"; DE+EN Fluent texts; one ADR (next free number); changelog
DE+EN under Unveröffentlicht; manual, README (no per-state detail beyond one sentence and a link), AGENTS.md if
the tool contract changes (it should not).

## Service facts (checked live 2026-11-13)

- All five are GeoServer WFS 2.0.0 at `https://gdi.berlin.de/services/wfs/<service>`; GeoJSON as
  `outputFormat=application/json`; CRS 25833 default, 4326 offered. A 4326 bbox is lat,lon first with the CRS
  suffix, which `geojson_wfs::query_url` already writes with `bbox_crs_suffix: true`; GeoJSON coordinates come
  lon,lat. A 25833 bbox `minE,minN,maxE,maxN,urn:ogc:def:crs:EPSG::25833` works too.
- Types: `alkis_bezirke:bezirksgrenzen` (`namgem` = Bezirk), `gruenanlagen:gruenanlagen` (`namenr`,
  `namezusatz`, `bezirkname`, `widmung`, `ortstlname`; blanks are a single space), `gruenanlagen:spielplaetze`,
  `liegenschaftsplan:liegenschaftsplan` (`sch` = "Land Berlin" / "Bundesrepublik Deutschland", `art` numeric,
  `bem` null), `denkmale:denkmale` (`id`, `typ`, `link` = `https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr=<id>`,
  no name), `schutzgebiete:a_nd` (MultiPoint; `nd_id`, `baum_deutsch`, `schutzgrund`, `zitat`, …),
  `schutzgebiete:e_fnd`, `g_glb`, `f_fspr` (polygons; `gebietsname`, `schutzstatus`, `zitat`, `datum_stammvo`, …).
- Viktoriapark box (25833 389613,5816190,390529,5816909): 2 Bezirke (Friedrichshain-Kreuzberg main,
  Tempelhof-Schöneberg edge), 7 Grünanlagen, 4 Spielplätze, 60 parcels (58 Land Berlin, 2 Bund), 43 monuments
  (30 Baudenkmal, 8 Gesamtanlage, 4 Ensemble, 1 Gartendenkmal; one answer 548 KB), 8 Naturdenkmal points, no
  polygon protected layer.
- Licence, verbatim: "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - Version 2.0 anzuwenden.
  Die Lizenz ist über https://www.govdata.de/dl-de/zero-2-0 abrufbar." The abstracts say only the analogue
  official maps are legally binding.
- `gdi.berlin.de` is already among the hosts of Berlin's manifest.

## Decided how (Claude's calls)

- **Hook:** a new method on the region adapter, `permit_sources()`, empty by default; Berlin returns its five
  services. `permits::fetch::fetch_inputs` asks every adapter of the registry whose ground touches the map area
  and runs its permit fetch inside `adapters::asking(adapter, …)`, so the adapter's host rules apply. The generic
  reader is step 1's `geojson_wfs::get_features` (streaming cap, paging limit).
- **Model:** the adapter returns typed findings (Bezirk, green space, owner share, monument, protected object)
  that step 1's `PermitNotes` holds in a new optional `regional` part, keyed by adapter id. `assemble` computes
  shares on the map polygon as in step 1; points count as "on the map" when inside the polygon.
- **Storage:** `data/permits/<adapter id>/` with one GeoJSON per layer, and the adapter's source state in
  `sources.json` as for the national sources.
- **Failure:** a failing Berlin layer makes the Berlin source "unreachable" in the table and drops its section;
  the rest of the notes stay; the Bezirk hints fall back to step 1's city-state hint.
- **Size:** the monument answer can be large; the step-1 cap (50 MB per answer, 20 pages) holds.

## Left to the builder

Type and module names, the JSON shape of the `regional` part (versioned with the notes, schema stays 1 if the
part is optional and older files still read), exact Fluent wording within the rules above, test fixtures recorded
from the live answers (trimmed).
