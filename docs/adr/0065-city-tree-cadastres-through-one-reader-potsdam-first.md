# 0065. City tree cadastres through one reader, Potsdam first

Date: 2026-11-09
Status: Accepted

## Context

Twelve German cities publish their trees as open data (survey of 2026-11-08), through ODS exports, WFS, OGC API
or CSV, with different field names. Potsdam has the Bäume der Stadtkarte: three ODS datasets
(`baeume-1-stadtkarte-potsdam`, `-2-`, `-3-`, 283 772 points in all) with kind (`ggartbez`: Laubbaum, Nadelbaum,
Obstbaum, `Einzelgebüsch, Strauch`, Baumstumpf), point number `pktnr` and survey date `datum` (1993 to 2026), but no
crown diameter. Licence dl-de/zero-2-0.

## Decision

- One reader `sources/trees.rs` with a table `TREE_DATASETS`: id, name, region, envelope, format, endpoints,
  field names, skipped kinds and attribution line. A new city is one row; the WFS, OGC API and CSV formats come
  with the first city that needs them.
- Gate: a dataset is asked only when the area touches its region and its envelope (Potsdam E 359 000 to
  375 000, N 5 801 000 to 5 820 000, EPSG:25833). A Berlin area inside the envelope makes no request.
- Potsdam: `exports/geojson?where=in_bbox(geo_point_2d,<lat_min>,<lon_min>,<lat_max>,<lon_max>)`, the area plus
  1 m in WGS84, latitude first. The export has no pages. The three datasets are merged and deduplicated by id
  and position; one failing endpoint fails the dataset with a warning.
- Stumps (`Baumstumpf`) are dropped at the reader and never stored. Trees go to `data/trees/<id>.geojson` in the
  map CRS, recorded in `tree_layers`. Paths come only from the table (ADR 0059), writes follow ADR 0060.
- Rules in `state-candidates.toml`: `tree:crown_m>=10` → 417 (ADR 0051), every other tree → 418. Potsdam has no
  crown, so all its trees are 418 candidates.
- Tags: `freemapper:source=trees/potsdam`, `freemapper:source_id=<pktnr>`, `freemapper:currency=<datum>` (each
  tree's own date, not the fetch date), `freemapper:baum=<species or kind>[, Krone <n> m]`.
- A city cadastre is a register (ADR 0063): an OSM or NAS tree within 2 m of a Potsdam tree is dropped.
- Attribution `Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0`, only when a Potsdam tree is
  among the candidates or inside a generalized tree group (`drawn_registers`, ADR 0066).
- Disk: part of `Part::StateLayers`, 1 MB per km² in the estimate (Potsdam measures 0.57).

## Consequences

- Potsdam maps get street and park trees with their own date. The controller's probe box at Sanssouci gave
  1 467 trees with unique point numbers and 9 real stumps; the example's fetch box (map area plus 50 m) gave
  1 964 trees, 1 646 of them inside the clip, of which 1 267 stay points after the thinning (ADR 0066).
- Each endpoint answer streams into a part file beside `data/trees/` and stops at `max_bytes` (256 MB, as for
  the NAS answers; the whole Potsdam envelope is about 110 MB), so a misbehaving server fails the dataset with
  a warning instead of filling the memory.
- Without a crown, a large Potsdam tree is a 418. The mapper upgrades it to 417 from the nDSM or the terrain.
- Cost if wrong: a dataset that changes its field names gives no trees until its row is updated.

Source: plan `docs/superpowers/plans/2026-11-09-states-plus-1.md` (Rulings, Task 5);
`docs/regions/states-plus-reference.md` (Brandenburg, trees); branch `states-plus-1`, Task 5, its review and the
live check.
