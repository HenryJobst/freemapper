# The adapter manifest, format 1

Reference for `manifest.toml`, the file that describes a region adapter. The [guide](en.md) explains how to
write one, the [tutorial](tutorial.en.md) builds one step by step. German: [manifest-v1.de.md](manifest-v1.de.md).

Folder adapters are read by freemapper 0.14.0 and later.

## The folder

```
<id>/
  manifest.toml       this file, at most 64 KiB
  boundary.geojson    the boundary the manifest names
```

- The folder has the id of the adapter as its name.
- It holds at most 64 files. The folder and the files freemapper reads must be real ones; a link is not
  followed.
- A `rules.toml` in the folder is not read (warning `adapter.rules-not-read`); the adapter still loads.
- In the adapter folder, beside the folders of the adapters, a plain file and anything whose name starts with
  a dot (`.git`, the folder of a sync tool) is passed over.

## Rules that hold everywhere

- An unknown key is a finding. So is a key that does not belong to the `kind` of its table.
- A text has at least one character, no line break and no other control character. The tables below give the
  longest length.
- A **URL** starts with `https://`, has no user name, no port other than 443, no query and no fragment
  (request parameters belong in `params`), and its host is a DNS name listed in `[net] hosts`. An IP address,
  `localhost` and a name with one label are refused. A manifest holds at most 64 URLs.
- A **CRS** is an EPSG number. A map is in 25832 or 25833, and so is every service table. A boundary file is in
  4326 or 25833.
- A **layer**, a **coverage**: 1 to 100 letters, digits and `_.:-`. A **feature type**: a name, or
  `prefix:name`. A **field**: starts with a letter or `_`, then letters, digits and `_.-`, at most 60.
- The validator goes on after a finding, so one run names all of them, each with the file, the line, the key
  and a code.

## `format`

The first line of the file: `format = 1`. Another number is `manifest.format`.

## `[adapter]`

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `id` | text | yes | A lower-case letter, then 1 to 31 lower-case letters or digits. The name of the folder. The project file stores it. |
| `provider` | text | no | Who serves the rasters; same pattern as `id`, which is also its default. The cache folder of the rasters is named after it. Two adapters share a provider only when both name the same backend. |
| `revision` | whole number | yes | From 1. Raise it when you change the manifest in a way that changes the data. `adapters list` shows it. |
| `min_freemapper` | text | yes | `x.y.z`: the oldest freemapper that reads this manifest. An older program leaves the adapter out and says which version it needs. For a folder adapter this is `0.14.0` or later. |
| `crs` | whole number | yes | 25832 or 25833: the CRS of a map whose centre lies in the region. |
| `object_id_prefix` | text | no | What the object ids of the region's cadastre start with (`DEST`, `DESL`): 2 to 12 capital letters or digits. At a border it tells which region an object belongs to. |
| `names` | table | yes | `en` (required) and `de` (default: `en`), each at most 80 characters. |
| `agency` | text | no | The short name of who serves the data, for messages; at most 40. Default: `names.en`. |
| `maintainers` | list of texts | no | Who looks after the adapter; at most 8 entries of at most 100 characters, for example `github:name`. No e-mail address: the file is public. |
| `last_verified` | text | no | `YYYY-MM-DD`: the day the services were last checked against the manifest. `adapters check` warns when it is missing or older than twelve months. |
| `licence_of_manifest` | text | no | The licence of the manifest itself; at most 100. |

## `[net]`

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `hosts` | list of texts | yes | Every server the adapter may ask, as DNS names, at most 32. No wildcard. A request to any other host is refused, also one a redirect or a link in an answer leads to. |

## `[boundary]`

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `file` | text | yes | A file name in the folder that ends in `.geojson`: letters, digits and `._-`, at most 64. No path. |
| `crs` | whole number | yes | 4326 (longitude, latitude) or 25833. |
| `simplify_m` | number | no | How far the boundary was simplified, in metres: 0, or above 0 up to 1000. Default 25. |

The file is GeoJSON: a `Feature`, or a `FeatureCollection` with exactly one feature, whose geometry is a
`Polygon` or a `MultiPolygon`. At most 2 MiB and a million vertices. The geometry must be valid (no ring that
crosses itself), and its positions must be possible in the stated CRS: degrees for 4326, metres for 25833.

## `[[osm.extract]]`

At least one, at most 16. The OpenStreetMap extract a map in the region reads.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `geofabrik` | text | yes | The path of the extract on `download.geofabrik.de` without `-latest.osm.pbf`, as in `europe/germany/hessen`. |
| `megabytes` | number | yes | The size of the extract, for the download estimate; above 0. |
| `label` | text | yes | What the extract is called in messages; at most 80. |
| `boundary` | text | with several extracts | The boundary file of this extract, named like `[boundary] file`. An adapter with several extracts gives each one. |
| `cache_prefix` | text | no | What the name of the cached file starts with: lower-case letters, digits and hyphens, at most 40. Default: the last part of `geofabrik`. Two extracts cannot share it. |

## `[attribution]`

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `licence` | text | yes | One of `dl-de/zero-2-0`, `dl-de/by-2-0`, `cc-by-4.0`, `odbl`, `cc0-1.0`, `other`. |
| `licence_text` | text | with `other` | The text of the licence, at most 2000. Only with `licence = "other"`. |
| `text` | text | yes | The credit line, at most 400. `{year}` may appear once and stands for the year of the newest fetch. No other placeholder. |
| `year` | text | no | Only `newest-fetch`, which is also what happens without the key. |
| `short` | text | no | A short credit for where the full one has no room; at most 60. |
| `note` | text | no | A line about the licence for `ATTRIBUTION.txt`, for example the address of the licence text; at most 400. |

## `[alkis]`

The cadastre: buildings and land use in the simplified ALKIS schema of the AdV ("ALKIS vereinfacht"). Optional,
but an adapter has `[alkis]`, `[orthophoto]` or both.

| Key | Type | `wfs` | `ogc-api-features` | Meaning and limits |
|---|---|---|---|---|
| `kind` | text | yes | yes | `wfs` or `ogc-api-features`. |
| `crs` | whole number | no | no | The CRS the service is asked in. Default: `[adapter] crs`. |
| `name` | text | no | – | The name of the service in messages; at most 80. Default: `<names.en> ALKIS`. |
| `url` | URL | yes | – | The WFS, without a query. |
| `buildings_type` | feature type | yes | – | The type with the buildings, as in `ave:GebaeudeBauwerk`. |
| `land_use_type` | feature type | yes | – | The type with the land use, as in `ave:Nutzung` or `ave:NutzungFlurstueck`. |
| `square_bez` | text | no | – | The `bez` given to a square (`Platz`) that has none; at most 100. Such a square stays a candidate. |
| `sorted` | true or false | no | – | `false` for a WFS that answers `SORTBY` with an exception; freemapper then pages without it and sorts the features itself. Default `true`. |
| `buildings` | URL | – | yes | The items endpoint of the buildings collection. |
| `land_use` | URL | – | yes | The items endpoint of the land use collection. |
| `normalise` | text | – | no | `lgb` (default): the service writes the class names as the LGB does. `hamburg-ascii`: it writes them in ASCII (`Wohnbauflaeche`). |

`[alkis.fallback]`, with `ogc-api-features` only: a WFS that is asked when the API fails, in the CRS of the API.
Keys: `url`, `buildings_type`, `land_use_type` (required), `name`, `square_bez`, and `kind`, which can only be
`wfs`.

The reader takes the attribute names of the schema in lower case (`oid`, `gebnutzbez`, `funktion`, `nutzart`,
`bez`) or all in upper case, and the geometry from `geometrie` or `SHAPE`.

## `[orthophoto]`, `[terrain]`, `[surface]`

The rasters. Either none of the three, or `[orthophoto]` and `[terrain]`, with or without `[surface]`. The
tables are all services, or they all name the same backend.

A table that names a backend has one key:

| Key | Type | Meaning and limits |
|---|---|---|
| `backend` | text | The name of a raster backend built into freemapper: 2 to 32 lower-case letters, digits and hyphens. A name the program does not have is `manifest.backend-missing`. `kind = "backend"` may stand beside it; any other key may not. |

`[orthophoto]` as a service:

| Key | Type | `wms` | `wcs` | Meaning and limits |
|---|---|---|---|---|
| `kind` | text | yes | yes | `wms` or `wcs`. |
| `url` | URL | yes | yes | The service, without a query. |
| `layer` | layer | yes | – | The layer of the WMS. |
| `coverage` | coverage | – | yes | The coverage of the WCS. |
| `max_px` | whole number | yes | yes | The longest side of one request in pixels, 256 to 10000. A larger area is asked in parts. |
| `crs` | whole number | no | no | Default: `[adapter] crs`. |
| `label` | text | no | no | What the raster is called in the estimate and in messages; at most 80. Default: `orthophoto (<names.en> WMS)` or `… WCS)`. |
| `params` | table | no | – | More GetMap parameters, see below. |
| `date` | table | no | no | Where the flight date is asked, see below. |

`[terrain]` and `[surface]` as a service are a WCS 2.0 that answers a GeoTIFF with heights:

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `kind` | text | yes | `wcs`. |
| `url` | URL | yes | The service, without a query. |
| `coverage` | coverage | yes | The coverage id. |
| `envelope` | list of four numbers | yes | The extent of the coverage in the CRS of the adapter: `[min_e, min_n, max_e, max_n]`. |
| `max_m` | number | yes | The longest side of one request in metres, above 99 up to 20000. |
| `multipart` | true or false | no | `true` when the TIFF comes inside a `multipart/related` answer. Default `false`. |
| `label` | text | no | Default: `terrain model (<names.en> WCS)` or `surface model (…)`. |
| `params` | table | no | More GetCoverage parameters, see below. |
| `date` | table | no | Where the date of the model is asked. |
| `source_label` | text | no | `[terrain]` only: the name of the model in the `freemapper:source` tag of the contours: 2 to 40 lower-case letters, digits and hyphens. |

### `params`

A table of at most 16 request parameters that freemapper adds to each request of the table. A name starts with
a letter and has letters, digits and `_.:-`, at most 40; a value has at most 200 characters. freemapper sets
these itself, so a manifest may not: `service`, `version`, `request`, `layers`, `styles`, `crs`, `srs`, `bbox`,
`width`, `height`, `format`, `coverageid`, `subset`, `query_layers` (in any case). A TOML table cannot hold a
name twice, so a service that needs one parameter twice cannot be declared.

### `date`

`[orthophoto.date]`, `[terrain.date]`, `[surface.date]`. Without the table the raster has no date.

| `kind` | Keys | What is asked |
|---|---|---|
| `none` | none | Nothing. |
| `wfs-geojson` | `url`, `layer` (a feature type), `field`, `format` (`DD.MM.YYYY` or `YYYY-MM-DD`) | A WFS that answers GeoJSON: the `field` of the feature of `layer` at the centre of the area. |
| `stac-newest` | `url` | A STAC API: the newest item at the centre of the area. |
| `wms-featureinfo` | `layer`, `field` | GetFeatureInfo on a layer of the orthophoto's WMS. Only in `[orthophoto.date]` of kind `wms`. |

## `[[extra.nas]]`

At most one: a WFS with the full ALKIS (NAS), from which freemapper reads structures, vegetation features and
small waters.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `name` | text | yes | At most 80. |
| `url` | URL | yes | The WFS. |
| `crs` | whole number | no | Default: `[adapter] crs`. |
| `max_count` | whole number | yes | Features per request, 1 to 1 000 000. |
| `max_bytes` | whole number | yes | The largest answer read, 1 to 268 435 456 (256 MiB). |
| `layers` | list of texts | no | The layers the service serves, of `sonstiges-bauwerk`, `vegetationsmerkmal`, `strukturlinie`, `bauwerk-verkehr`, `bauwerk-gewaesser`, `untergeordnetes-gewaesser`. Default: all. |
| `namespaces` | text | no | The `NAMESPACES` parameter of the requests; at most 300. |

## `[[extra.trees]]`

At most 32: the tree cadastres of cities, read from an Opendatasoft GeoJSON export.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `id` | text | yes | 2 to 32 lower-case letters, digits and hyphens; once per manifest. |
| `name` | text | yes | At most 80. |
| `crs` | whole number | no | Default: `[adapter] crs`. |
| `envelope` | list of four numbers | yes | The extent of the data in `crs`. |
| `format` | text | yes | `ods-geojson-export`. |
| `geo_field` | field | yes | The field with the position. |
| `endpoints` | list of URLs | yes | 1 to 8. |
| `fields` | table | yes | `kind`, `id`, `date` (required), `species`, `crown`: the names of the fields. |
| `skipped_kinds` | list of texts | no | Kinds that are no trees; at most 16. |
| `attribution` | text | yes | The credit of the cadastre, at most 400, with `{year}` at most once. |
| `max_bytes` | whole number | yes | 1 to 268 435 456. |

## `[preview]`

The aerial photo on the map of the area page. Optional.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `wms` | URL | yes | A WMS that serves EPSG:3857. |
| `layer` | layer | yes | |
| `format` | text | yes | `image/png` or `image/jpeg`. |
| `params` | table | no | As above. |
| `attribution` | text | yes | The credit on the map, at most 200, with `{year}` (the current year) at most once. |
| `agency` | text | yes | The short name of the agency, for the name of the layer; at most 40. |

## `[estimate]`

Megabytes per km² for the download estimate, where the adapter knows better than the defaults (8 for a WMS
orthophoto, 4 for a height model from a WCS). Each above 0 up to 10 000: `orthophoto_mb_per_km2`,
`terrain_mb_per_km2`, `surface_mb_per_km2`.

## Findings

`freemapper adapters check` and the loader name each finding with its code. A finding keeps the adapter out; a
warning does not.

| Code | What it means |
|---|---|
| `manifest.too-large` | The manifest is over 64 KiB. |
| `manifest.syntax` | Not TOML. |
| `manifest.format` | `format` is missing or not 1. |
| `manifest.unknown-key` | A key the format does not have, or one of another `kind`. |
| `manifest.missing-key` | A key the table or its `kind` needs. |
| `manifest.type` | A value of the wrong type. |
| `manifest.id` | An id that does not fit the pattern or is not the name of the folder. |
| `manifest.min-freemapper` | Not a version `x.y.z`. |
| `manifest.crs` | A CRS the key does not allow. |
| `manifest.names` | An empty or overlong name. |
| `manifest.value` | A value outside its range or pattern. |
| `manifest.file-name` | A file name that is no plain name in the folder. |
| `manifest.geofabrik` | Not a path of an extract on the Geofabrik server. |
| `manifest.osm` | The extracts do not fit together. |
| `manifest.licence` | A licence outside the list, or `other` without its text. |
| `manifest.attribution` | A credit that is empty, too long or has a line break or a second placeholder. |
| `manifest.kind` | An unknown `kind`, or `kind` and `backend` that contradict. |
| `manifest.date-kind` | An unknown `kind` of a date lookup, or `wms-featureinfo` where there is no WMS. |
| `manifest.normalise` | An unknown `normalise` preset. |
| `manifest.params` | A request parameter the manifest may not set. |
| `manifest.too-many-urls` | More than 64 URLs. |
| `manifest.empty` | Neither `[alkis]` nor `[orthophoto]`. |
| `manifest.rasters` | Raster tables that do not fit together. |
| `manifest.backend-missing` | A raster backend this program does not have. |
| `manifest.file-missing` | A file the manifest names is not there or cannot be read. |
| `adapter.folder` | The folder is a link, has too many files or cannot be read. |
| `adapter.needs-newer` | The manifest asks for a newer freemapper. |
| `adapter.duplicate` | The id is taken. A folder cannot replace another adapter. |
| `adapter.provider-shared` | Services of its own under the provider of another adapter. |
| `boundary.too-large` | A boundary file over 2 MiB or with more than a million vertices. |
| `boundary.format` | Not GeoJSON with one polygon feature. |
| `boundary.crs` | Positions that cannot be of the stated CRS. |
| `boundary.invalid` | No valid geometry. |
| `policy.url` | A URL that is not `https` on port 443 with a DNS name and without a query. |
| `policy.host` | An entry of `[net] hosts` that is no DNS name. |
| `policy.host-not-declared` | A URL whose host is not in `[net] hosts`. |

Warnings:

| Code | What it means |
|---|---|
| `adapter.rules-not-read` | The folder has a `rules.toml`, which this freemapper does not read. |
| `check.never-verified` | The manifest has no `last_verified`. |
| `check.verified-long-ago` | `last_verified` is more than twelve months ago. |
| `check.built-in-id` | An adapter with this id is built into the program; from an adapter folder this one does not load. |
| `check.prefix-shared` | A built-in adapter has the same `object_id_prefix`. |
| `check.boundary-area` | The boundary encloses under 1 km² or over 2 000 000 km²: probably the wrong unit. |
| `check.shares-ground` | The adapter lies on the ground of another one: the two have more than a hundredth of the smaller region in common. A map there gets the objects of both. |

## Complete examples

The manifests of the built-in states are in
[`crates/adapters/de/manifests/`](https://github.com/HenryJobst/freemapper/tree/main/crates/adapters/de/manifests). Saxony-Anhalt declares every source as a
service. Saarland declares the cadastre and names a backend for the rasters. Hamburg uses an OGC API with a WFS
fallback. Brandenburg has the full ALKIS (`[[extra.nas]]`) and a tree cadastre. Berlin has no `[alkis]`: its
cadastre is code.
