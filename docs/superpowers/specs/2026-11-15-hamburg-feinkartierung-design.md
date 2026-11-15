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

## PR 1: lines, stairs, points

- **Fetch.** An `ExtraSet` of the Hamburg adapter, key `lgv-feinkartierung`, six layers (`fk-strasse-linien`,
  `fk-strasse-punkte`, `fk-strasse-treppen`, `fk-gruen-flaechen`, `fk-gruen-linien`, `fk-gruen-punkte`), each
  the items of one collection in the region box (CRS84) with `crs=EPSG:25832`, `limit=10000` and the `next`
  links. `fk-strasse-treppen` keeps only the street areas with `nutzung=Treppe` (the surfaces come in PR 2).
  Stored in EPSG:25832.
- **Build.** `build/feinkartierung.rs` tags each feature `hamburg:<property>` plus `hamburg:layer=<file>` and
  draws it through `assets/rules/hamburg-candidates.toml` like the Berlin layers, with the source
  `lgv-feinkartierung/<file>` (rank `Survey`, as the Berlin street survey). Rules: Geländer, Zaun, Zäune,
  Handläufe, Schutzgitter → 518; Mauer → 515 (the green wall areas as their outline); Bordstein, Hochbord,
  Tiefbord → 501.1 (sprint); bollards, benches and lamps → the schoolyard's 544, 543, 545 and a sprint map's
  531 with `freemapper:control`, as the Berlin ones; play equipment → 541 (schoolyard); monuments and art → 526.
  Rinne, traffic lights, bicycle stands and manhole covers are left out.
- **Stairs.** The street areas with `nutzung=Treppe` and the green areas Wegenetz/Treppen go to
  `build/stairs.rs` as surveyed stair areas in the map's CRS (a new field of `StairInput`), drawn as 532.6–532.9
  with the walking direction from the terrain; an ALKIS stair at the same place wins.
- **Placement.** Nothing new: under `balanced` the groups walls, fences and stairs go to the map from rank OSM
  up, streets (501.1) from `Survey` up, furniture never. So the lines and stairs land on the map and the points
  stay candidates, as the user decided.

## PR 2: surfaces as candidates

The street areas by `inhalt`: water-bound and grassed-through gravel as unpaved-path candidates, the paved
ones not at all (the land use and OSM have them). Designed after PR 1.

## Left out

The `stand` date per object as the currency (the layers get their fetch date, as Berlin's registers); the
Feinkartierung of other districts is the same service.
