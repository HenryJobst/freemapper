# States Plus – Roadmap after plan 1

Sources are in `STATES-PLUS-SURVEY.md` (2026-11-08), corrected by the probes of 2026-11-09 (plan 1, Rulings).
Plan 1 (`2026-11-09-states-plus-1.md`) builds these pieces:
- vereinfacht passages and canopies in every state;
- the NAS reader with the AdV rule table;
- NAS stairs;
- the tree-point reader (ODS export format);
- the duplicate rank.

After plan 1, a state is a service row, a few rules and one reader format at most. The order is the order in which
the states were added.

## Shared pieces and who builds them

| # | Piece | Built in | Used by |
|---|---|---|---|
| A | Vereinfacht passages/canopies | plan 1 | all (no data in SN, ST) |
| B | NAS WFS reader + `state-candidates.toml` + `adv-codes.toml` | plan 1 | ALKIS: BB, HE, MV, NW, NI. Basis-DLM: ST, HE, SH, MV |
| C | Basis-DLM rule section (`alkis:model=dlm`, rank below OSM) | plan 3 (ST) | ST, HE, SH, MV, and C+D states |
| D | basemap.de Basis-DLM GPKG reader | plan 2 (SN) | SN, HH, NW, NI, RP |
| E | Tree formats: ODS export (plan 1), WFS GML/GeoJSON (plan 2), OGC API (plan 4), CSV (plan 5) | as listed | 12 cities |
| F | OGC API Features GeoJSON reader (exists in `alkis.rs`) for non-ALKIS layers | plan 4 (HH) | HH, later NRW `inspire-lc-le` |

The NAS `max_count` per service comes from `CountDefault` (2026-11-09). No service pages.

| Service | `max_count` |
|---|---|
| BB, HE, HE DLM, ST DLM | 100 000 |
| MV, MV DLM | 30 000 |
| NW | 500 000 |
| NI, SH DLM | 10 000 |

## Order

1. **Saxony – plan 2 (`states-plus-2`), effort medium-high.**
   - Needs D and E (WFS).
   - Gives:
     - Basis-DLM water axes with width and `nicht ständig Wasser führend` (305/306);
     - hedges, tree rows, dams;
     - Leipzig WFS trees (`OpenData:Baeume`, GeoJSON output, crown Ø sometimes) and Dresden WFS trees
       (`cls:L1261`, `numberMatched=unknown`).
   - Add-on: Frankfurt (Oder) for Brandenburg (`ffo:Baeume`, crown Ø `KR_DURCHM` → 417/418), one row.
   - **Task 1 is a spike.** D can read the GPKG by HTTP Range (SQLite pages + R-tree, 879 MB file) or download and
     cache it per state with the disk plan. The rest of the plan follows the spike's result.
   - The full ALKIS of Saxony holds no topography, so it is not used.
   - **Decide in plan 2:** how a tree row (line) becomes candidates (418 per vertex, or a line with
     `freemapper:size=row`).
2. **Saxony-Anhalt – plan 3, effort low-medium.**
   - Needs B (Basis-DLM NAS WFS `ST_LVermGeo_ATKIS_WFS_BDLM_OpenData`, EPSG:25832) and builds C.
   - Gives:
     - footpaths `AX_WegPfadSteig` 1103 as candidates (505.x);
     - monuments;
     - water axes;
     - bridges, passages under roads (`BauwerkImVerkehrsbereich` 1900).
   - No trees (Halle and Magdeburg not found), no passages (rule-only data empty).
   - **Decide in plan 3:** the Basis-DLM rank. Recommended: below OSM in towns, but water axes win over OSM
     streams because they carry width and intermittency.
3. **Hamburg – plan 4, effort medium.**
   - Needs F and E (OGC API); D for water (59 MB GPKG).
   - Gives Feinkartierung Straße:
     - curbs 501.1 on sprint maps;
     - railings and fences 518;
     - bollards 544 on the schoolyard;
     - surfaces → `paved.rs` candidates, with `wassergebunden` as unpaved.
   - Feinkartierung Grün: park walls 515, stairs, fences.
   - Straßenbaumkataster with `kronendurchmesser` (417/418).
   - Reuse the Berlin survey rulings (ADRs 0052–0054: candidates only, furniture on the schoolyard only).
4. **Hesse – plan 5, effort low-medium.**
   - Needs B: ALKIS NAS `…/alkis/nas/wfs`, Freitreppe 1621 through plan 1 stairs. C: Basis-DLM NAS on the same
     host.
   - Builds E (CSV) for Frankfurt am Main (`Hochwert`/`Rechtswert`, crown Ø, height).
   - Licence dl-de/zero-2-0.
   - Also regenerate the Marburg example; its data folder is not stored today.
5. **Schleswig-Holstein – plan 6, effort low.**
   - Needs C: Basis-DLM NAS WFS `WFS_SH_ATKIS_BDLM_SF_OpenGBD`. The GPKG is a year old, so it is not used.
   - Needs E: Kiel WFS `Baeume` with crown Ø; Lübeck CSV (CC0).
   - Attribution CC BY 4.0 `© GeoBasis-DE/LVermGeo SH/CC BY 4.0`.
   - Full ALKIS comes per Flur only (bulk GML): not planned.
6. **Mecklenburg-Vorpommern – plan 7, effort low.**
   - Needs B: `alkis_wfs_nas`, EPSG:25833, the richest data. Fences, retaining walls 1721/1722, stairs with 1630
     edges, hedges, wall edges.
   - Needs C (`atkis_bdlm_wfs_nas`) and E: Rostock CSV/JSON.
   - **Check first:** `geo.sv.rostock.de` was unreachable on 2026-11-08. Drop the trees if it still is.
   - Responses are large (1 MB per 800 m box). Check the estimate.
7. **North Rhine-Westphalia – plan 8, effort low-medium.**
   - Needs B: `wfs_nw_alkis_aaa-modell-basiert`. Single trees 1012 → 418 at ALKIS rank, fences, walls,
     Strukturlinie3D 1220.
   - Needs E: Köln WFS, Dortmund CSV, Münster. City trees beat NAS trees (plan 1 rank).
   - Needs D for water: the GPKG is 2.1 GB, so it needs the Range path from the plan 2 spike.
   - Licence dl-de/zero-2-0.
8. **Lower Saxony – plan 9, effort low.**
   - Needs B: `alkis_wfs_nas`, bridges only. The canopies and passages already come from A, and `max_count` is
     10 000.
   - Needs D: GPKG 1.4 GB for water, tree rows, hedges.
   - No trees (Hannover not found).
   - Could be merged with plan 10 if both stay this small.
9. **Rhineland-Palatinate – plan 10, effort low.**
   - Needs D (GPKG 1.0 GB).
   - The Shape-schema WFS has other attribute names: not used.
   - A already gives the high yield here (1 775 canopies, 1 225 passages per the reference doc). Verify that with
     a live count at the start, because RP joins several names with `;`.

## Next step

Write plan 2 (Saxony) after plan 1 is merged. Its first task is the GPKG spike (Range-read vs cached download,
measured on the 59 MB Hamburg file, then on Saxony). Every later plan follows the plan 1 format, with 3–4 tasks:
service row and fixtures, rules, docs and example.

## Open questions carried forward

- Tree rows from Basis-DLM: point per vertex or row line (plan 2).
- Basis-DLM rank against OSM (plan 3).
- Towers (vereinfacht `Turm`, NAS `AX_Turm` with `objekthoehe`) → 524/525 candidates. Not yet planned. It is low
  effort with B, as an add-on to any NAS plan.
