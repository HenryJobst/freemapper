# Hamburg: the Feinkartierung — design

Date: 2026-11-15. Decision of the user (question round of 2026-11-15): railings, fences, walls and stairs go
straight onto the map like the other official sources under the profile `balanced`; surface materials, bollards
and benches become candidates, as for Berlin. ADR 0149.

## The data

The LGV Hamburg serves two surveys as OGC API Features collections on `api.hamburg.de` (checked live on
2026-11-15 over Blankenese, `stand` mostly 2021–2023):

- `feinkartierung_strasse` (dl-de/by-2-0, "Freie und Hansestadt Hamburg, Behörde für Verkehr und
  Mobilitätswende"): `linien` (`inhalt`: Bordstein, Hochbord, Tiefbord, Rinne, Geländer, Mauer, Zaun), `punkte`
  (`kategorie`/`inhalt`: Lampe, Poller/Pfosten/Wegesperre, Bügel, Fahrradbügel, Verkehrsampel, Findling, Bank,
  Kunst im öffentlichen Raum, Pflanzkübel), `strassenflaechen` (`nutzung`: Gehweg, Überfahrt, Grünfläche,
  Parkplätze, Treppe, Fahrbahn …; `inhalt`: the surface, e.g. `wassergebunden (Grandfläche)`). District and main
  roads only.
- `feinkartierung_gruen` (dl-de/zero-2-0): `flaechenobjekte` (`kategorie_0`/`_1`: Wegenetz/Treppen,
  Abgrenzungen/Mauern, Wände, Wegenetz/Wege …), `linienobjekte` (Abgrenzungen/Einfriedungen/Zäune,
  Schutzkonstruktionen/Absturzsicherungen/Handläufe, Schutzgitter, Tore), `punktobjekte` (Absperrungen, Möbel,
  Leuchten, Spielgeräte, Denkmale …). Public parks and playgrounds only.

Blankenese (1.0 × 0.7 km): 375 street lines, 505 street points, 1 824 street areas, 313 green areas, 64 green
lines, 225 green points.

## A set that brings its rules

The engine names no region (ADR 0082, 0114); Berlin's layers are the one exception, "until adapters bring
rules". The Feinkartierung is the first set that brings them, so the engine draws it without knowing Hamburg:

- `ExtraSet::drawing()` (adapter API) returns a `SetDrawing`: the rules as TOML text (the syntax of the candidate
  rules), the tag prefix of the features (`hamburg`), the rank of the source in the duplicate step (`Survey`) and
  the layers whose areas are stairs. A set without one is fetched and stored as before and drawn by nobody
  (Berlin keeps its own path).
- The project file records the layers of such sets in one map, `extra_sets` (set key → file → record), beside
  `berlin_layers`. `fetch` asks every set of the adapters whose region the area touches.
- The build tags each feature `<prefix>:<property>` plus `<prefix>:layer=<file>`, draws it through the set's
  rules with the source `<set key>/<file>`, takes the rank from the set, and gives a control site the kind the
  feature names in `<prefix>:control` (the adapter writes it at fetch). The stair layers go to `build/stairs.rs`
  as surveyed stair areas; an ALKIS stair at the same place wins.

## PR 1: lines, stairs, points

- **Fetch.** The set `lgv-feinkartierung` of the Hamburg adapter, five layers: `fk-strasse-linien`,
  `fk-strasse-punkte`, `fk-gruen-linien`, `fk-gruen-punkte`, `fk-gruen-flaechen` (walls) and `fk-treppen` (the
  street areas with `nutzung=Treppe` and the green areas Wegenetz/Treppen). Each collection is asked in the
  region box (CRS84) with `crs=EPSG:25832`, `limit=10000` and its `next` links; stored in EPSG:25832.
- **Rules** (`crates/adapters/de/rules/lgv-feinkartierung.toml`): Geländer, Zaun, Zäune, Handläufe, Schutzgitter
  → 518; Mauer → 515 (the green wall areas as their outline); Bordstein, Hochbord, Tiefbord → 501.1 (sprint);
  bollards, benches and lamps → the schoolyard's 544, 543, 545 and a sprint map's 531 control site; play
  equipment → 541 (schoolyard); monuments and art → 526. Rinne, traffic lights, bicycle stands and manhole covers
  are left out.
- **Placement.** Nothing new: under `balanced` the groups walls, fences and stairs go to the map from rank OSM
  up, streets (501.1) from `Survey` up, furniture never. So the lines and stairs land on the map and the points
  stay candidates, as the user decided.

## PR 2: surfaces as candidates

The street areas by `inhalt`: water-bound and grassed-through gravel as unpaved-path candidates, the paved
ones not at all (the land use and OSM have them). Designed after PR 1.

## Left out

The `stand` date per object as the currency (the layers get their fetch date, as Berlin's registers); the
Feinkartierung of other districts is the same service.
