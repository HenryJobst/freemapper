# 0107. Adapter manifests, format 1, as built

Date: 2026-11-12
Status: Accepted

Part of stage D of [0082](0082-region-adapters-trait-and-manifests.md) (tasks D1 and D3 to D6). The rules for
where requests go are [0106](0106-requests-go-to-public-addresses-and-declared-hosts.md).

## Context

After stages B and C ([0102](0102-region-adapters-crates-trait-and-registry-as-built.md)) every state was an
adapter behind one trait, written in Rust. The design of 0082 wanted more: a state with standard services should
be a folder with a TOML file, so that someone who knows the services of a region can add it without writing Rust.
The design sketched the format; building it against the eleven states changed some of it.

## Decision

- A manifest is `manifest.toml` in a folder that has the id of the adapter as its name, beside the boundary files
  it names. The crate `freemapper-manifest` reads it into a strict schema: an unknown key is an error, every value
  is checked against a closed list, a range or a pattern, and every URL against the rules of 0106 and the hosts of
  `[net] hosts`. The validator reports all findings at once, each with the file, the line, the key and a stable
  code (`manifest.unknown-key`, `boundary.crs`, `policy.host-not-declared`).
- Tables of format 1: `[adapter]` (id, provider, names, agency, CRS, object-id prefix, revision, the oldest
  freemapper that reads it), `[net]`, `[boundary]`, `[[osm.extract]]`, `[attribution]`, `[alkis]`, `[orthophoto]`,
  `[terrain]`, `[surface]`, `[[extra.nas]]`, `[[extra.trees]]`, `[preview]`, `[estimate]`. Limits: 64 KiB per
  manifest, 64 URLs, 2 MiB and a million vertices per boundary file.
- A boundary file is GeoJSON with one polygon feature, in EPSG:25833 or in longitude and latitude. It must be a
  valid geometry, and its positions must be possible in the CRS the manifest states.
- `[alkis]` is a WFS or an OGC API Features service with an optional WFS fallback; `normalise` names the preset
  that turns the service's spellings into those of LGB.
- The three raster tables are all services or all name one backend, and an orthophoto comes with a terrain model.
  As services they are a WMS or WCS for the orthophoto and a WCS for each height model, each with an optional
  `date`: a WFS that answers GeoJSON, a STAC API or the feature info of the WMS. `DeclaredRasters` fetches them
  for the map area. A backend is code of an adapter crate, registered under a name (`Registry::add_backend`); a
  manifest that names a backend the program does not have is a finding.
- The credit is one line with `{year}` for the year of the newest fetch, a licence from a closed list and an
  optional note about the licence.
- The eleven states are manifests in `crates/adapters/de/manifests/<id>/`, embedded in the program. The adapter of
  a state is its manifest plus what no manifest says: the raster code of its agency, the ALKIS and the layers of
  Berlin, the credits of Brandenburg, Berlin and Saxony (they depend on tiles, on the neighbour and on the layers
  that drew objects) and the download estimate.

## What differs from the sketches of the design

- The design let one table be a service and the next a backend (Hamburg: a WMS orthophoto beside code for the
  heights). The backends of the states fetch the three rasters together, in an order the golden net records, so
  format 1 does not mix. A state with one odd source writes a backend for all three.
- A date is asked of a service or not at all; there is no `date = { backend = … }`.
- One state, Saxony-Anhalt, needs no raster code, not two. Lower Saxony has standard services too, but its fetch
  tells the user the leaf state of the flight and how old the surface model is, which no table says.
- New keys: `agency`, `label` of a raster, `cache_prefix` of an extract, `source_label` of the terrain model,
  `name` of an ALKIS service. `page_count` is gone: the readers have no such parameter.
- An adapter's own `rules.toml` is not read. The rule engine is part of the build code, and no state needs rules
  of its own yet.

## Consequences

- The golden net did not change: the eleven states fetch, estimate and credit as before. For Saxony-Anhalt a test
  runs the declared backend and the former code against the same services and compares requests, notes and
  records.
- A new state with standard services is a manifest, a boundary file and one line that registers it; a state with
  special rasters adds a backend. Folders outside the program are loaded by task D7.
- A manifest states the hosts its state asks, and a test compares that list with the recorded requests of the
  golden net. The fetcher does not enforce the list yet: a redirect to a host nobody listed would then fail, and
  no golden sees redirects. That switch follows with live runs of all states.
- Two things are said twice for now: the WFS constants in `freemapper-sources` beside the manifests, and the raster
  code of Saxony-Anhalt beside its tables. Tests hold them equal until the constants go.
