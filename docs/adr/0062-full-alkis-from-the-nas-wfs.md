# 0062. Full ALKIS from the NAS WFS, Brandenburg first

Date: 2026-11-09
Status: Accepted

## Context

ALKIS vereinfacht has buildings and land use, but no walls, fences, hedges, embankment edges or bridges. The
full ALKIS (NAS-konformer WFS) has them as surveyed lines and points, and it is open in Brandenburg, Hesse,
Mecklenburg-Vorpommern, North Rhine-Westphalia and Lower Saxony, all with the same AdV schema. The answers are
GML 3.2 only, with curves, arcs, points and surfaces built from curve rings, which the vereinfacht parser
(`alkis_wfs::parse_gml`) cannot read.

## Decision

- New reader `sources/alkis_nas.rs` with its own parser `parse_nas_gml`. Arcs (`Arc`, `ArcString`, `Circle`) are
  densified at 10°, curve members that meet become one line, and members that do not meet a MultiLineString. A
  3D `posList` loses its heights. An unknown element skips the feature and lists it, it is no error.
- A service table `NAS_SERVICES` (plan 1: `LGB_NAS`, `https://isk.geobasis-bb.de/ows/alkis_nas_wfs`,
  EPSG:25833) and a layer table `NAS_LAYERS`: `sonstiges-bauwerk`, `vegetationsmerkmal`,
  `bauwerk-gewaesser` and `untergeordnetes-gewaesser` on both map kinds, `strukturlinie` and `bauwerk-verkehr` on
  sprint maps. A service may serve only a subset (`NasService::layers`). Later states add one row each.
- No paging. All eight services checked have `ImplementsResultPaging=FALSE`, so there is one GetFeature per type
  with `COUNT=max_count` (the service's `CountDefault`, LGB 100 000). If `numberReturned < numberMatched`, or
  `numberMatched` is `unknown` and the cap is reached, or the answer passes 256 MB, the box is split into
  quadrants, at most 4 levels deep. Beyond that: `<service> returned <n> of <m> features; choose a smaller
  area`. Duplicates by `gml:id` are dropped.
- Region gate: only areas that touch a state in the service table make a request. An exception report or a
  failing layer is a warning (`warning: ALKIS NAS <file> unavailable (…); …`), never a failed fetch. Requests
  carry the `User-Agent` of `fetch.rs` and no personal data such as an e-mail address.
- No canopies and no building parts from NAS: `AX_Bauteil` is not fetched and 1610/1611 have no rule. The
  vereinfacht `oid` is the NAS `gml:id` plus `BL` (all 6 Sanssouci canopies), so the same canopy would be drawn
  twice.
- Currency is `lebenszeitintervall/beginnt` of the object version, the date of the cadastre entry and not a
  survey date. The fetch date applies only when it is missing or no date.
- Paths follow ADR 0059: layer files come only from `NAS_LAYERS` (`data/alkis-nas/<file>.geojson`), unknown or
  unsafe keys are dropped on load. Writes and the part files of each answer follow ADR 0060.
- The layers form the optional disk part `Part::StateLayers` with the tree cadastres (2 MB per km² for NAS).

## Consequences

- Brandenburg sprint and schoolyard maps get fences, wall edges, hedges, monuments, fountains and ditches as
  candidates, sprint maps also embankment tops and bridges. Brandenburg NAS has no stairs, retaining walls or
  trees.
- Cost if wrong: an old `beginnt` hides how recent a line is; a state with a lower cap splits more often.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Global Constraints, Task 2);
branch `states-plus-1`, Task 2, its review, re-review and fix rounds.
