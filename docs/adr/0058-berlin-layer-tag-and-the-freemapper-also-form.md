# 0058. Every Berlin candidate carries `berlin:layer`; `freemapper:also` names `<source>/<id>`

Date: 2026-11-09
Status: Accepted

## Context

Several survey types share one stored file (`sb-moebel` holds bollards, masts, flag poles, benches, hydrants
and play equipment), and `freemapper:source` names only the file. The register dedup must tell a survey mast
(`as_mast`) from a bollard (`av_poller`). Until now `freemapper:also` only named OSM ways (`osm/way/…`); now
ALKIS objects and registers also name survey and OSM nodes.

## Decision

- `berlin:layer` is a tag on every Berlin rule candidate. For the survey and the registers it is the type name
  without namespace (`bf_gelaender`, `beleuchtung`); for the plus 1 files it stays the file name
  (`bauwerke-linien`).
- `freemapper:also` holds `<source>/<source_id>` per dropped object, joined by `; `: `osm/way/123`,
  `osm/node/1`, `geoportal-berlin/sb-gelaender/<gis_id>`. The OSM form is the same as before, because the OSM
  source is `osm` and its source id `way/123`.
- Survey candidates use the `gis_id` as `freemapper:source_id`.

## Consequences

- The mapper sees in OOM and OCAD which survey type a candidate came from.
- One more tag on every Berlin candidate, also the plus 1 ones.
- No code reads `freemapper:also`, so the wider form breaks nothing.

Source: plan `docs/superpowers/plans/2026-11-09-berlin-plus-2.md`; branch `berlin-plus-2`, Task 1, Task 3 review
and Task 4.
