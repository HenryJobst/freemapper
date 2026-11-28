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
- In the adapter folder, beside the folders of the adapters, plain files and anything whose name starts with
  a dot (`.git`, the folder of a sync tool) are passed over.

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

`stac-gpkg` (land cover only, no buildings): `kind`, `stac` (the STAC API, https on a host of `[net] hosts`, no
query, no trailing slash) and `collection` (the collection with one GeoPackage per municipality: 1 to 64 letters,
digits, `_` and `-`), and no other key. The code of the adapter crate reads the files.

`[alkis.fallback]`, with `ogc-api-features` only: a WFS that is asked when the API fails, in the CRS of the API.
Keys: `url`, `buildings_type`, `land_use_type` (required), `name`, `square_bez`, and `kind`, which can only be
`wfs`.

The reader takes the attribute names of the schema in lower case (`oid`, `gebnutzbez`, `funktion`, `nutzart`,
`bez`) or all in upper case, and the geometry from `geometrie` or `SHAPE`.

## `[orthophoto]`, `[terrain]`, `[surface]`

The rasters. Either none of the three, or `[orthophoto]` and `[terrain]`, with or without `[surface]`, or
`[terrain]` alone (a region with a terrain model and no orthophoto: the map then has no orthophoto template and
`derive` warns). The tables are all services, or they all name the same backend. A `[terrain]` without
`[orthophoto]` that names a backend states the backend's `endpoints` itself, as `[orthophoto]` does; beside an
`[orthophoto]` it takes none.

A table that names a backend has one key:

| Key | Type | Meaning and limits |
|---|---|---|
| `backend` | text | The name of a raster backend built into freemapper: 2 to 32 lower-case letters, digits and hyphens. A name the program does not have is `manifest.backend-missing`. `kind = "backend"` may stand beside it; any other key may not. |

`[orthophoto]` of ten backends (`geosn`, `hvbg`, `lvermgeo-rp`, `lgl-bw`, `lvermgeo-sh`, `lvgl`, `geobasis-nrw`,
`ldbv`, `lgln`, `lgv`) states where the backend asks for the orthophoto, with the same keys as a WMS (issue #273). Each key
is optional; one left out is the one of the built-in manifest of the state, so a folder that replaces a state
(ADR 0188) states only what moved. The code keeps the CRS, the name of the info layer and the way the answers are read.
`[terrain]` and `[surface]` take none of these keys, and a backend not on this list takes none in any table.

| Key | Type | Meaning and limits |
|---|---|---|
| `url` | text | The WMS address, https on a host of `[net] hosts`, without a query. |
| `layer` | text | The layer of the orthophoto: 1 to 100 letters, digits and `_.:-`. |
| `max_px` | integer | The largest side of one request in pixels, 256 to 10000. |

Six backends state more addresses in `endpoints`, a table of names and addresses (`[orthophoto.endpoints]` or an
inline table), each a https address on a host of `[net] hosts`, without a query (issue #273). A name the backend does
not ask is `manifest.unknown-key`; a name left out is the one of the built-in manifest. The code keeps what it does
with the files: the tile lists, the coverages and extents of the services, and the names of the layers and years.

| Backend | Names | Address of |
|---|---|---|
| `lgv` (Hamburg) | `metadata`, `dgm1`, `bdom` | the flight dates of the orthophoto, the DGM1 archive, the bDOM archive |
| `geobasis-nrw` | `hm` | the folder of the height model files |
| `lgln` (Lower Saxony) | `dop-stac`, `dgm-stac`, `dom-stac`, `dgm-wcs`, `dom-wcs` | the STAC APIs and the WCS of the height models |
| `laiv-mv` | `overview`, `dop-wcs`, `dgm-wcs`, `dom-wcs` | the tile index of the Downloadportal and the three WCS |
| `geobremen` | `wms`, `files` | the WMS of the orthophotos (its layers come from its capabilities) and the file server |
| `lgb` (Brandenburg, Berlin) | `data`, `currency` | the folder of the tile archives (orthophoto, DGM, bDOM) and of the nDOM point clouds, and the folder of the currency lists (CSV) |
| `lantmateriet` (Sweden, in `[terrain]`) | `stac-hojd`, `stac-vektor` | the STAC APIs of the height products and of the vector products |

The cache folder of Hamburg's archives follows the file name of the address, so an archive at a new address gets its
own folder; the download estimate still looks in the one of the built-in address.

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

At most one per model: a WFS with the full ALKIS (NAS), from which freemapper reads structures, vegetation
features and small waters, and one with the landscape model (`model = "basis-dlm"`). Hesse names both.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `name` | text | yes | At most 80. |
| `url` | URL | yes | The WFS. |
| `crs` | whole number | no | Default: `[adapter] crs`. |
| `max_count` | whole number | yes | Features per request, 1 to 1 000 000. |
| `max_bytes` | whole number | yes | The largest answer read, 1 to 268 435 456 (256 MiB). |
| `layers` | list of texts | no | The layers the service serves, of `sonstiges-bauwerk`, `vegetationsmerkmal`, `strukturlinie`, `bauwerk-verkehr`, `bauwerk-gewaesser`, `untergeordnetes-gewaesser`; for the model `basis-dlm` of `dlm-bauwerk`, `dlm-vegetation`, `dlm-felsen`, `dlm-hoehle`, `dlm-gewaessermerkmal`, `dlm-damm`, `dlm-turm`, `dlm-gewaesserachse`, `dlm-weg`, `dlm-fahrweg`. Default: all of the model. |
| `namespaces` | text | no | The `NAMESPACES` parameter of the requests; at most 300. |
| `model` | text | no | `alkis` (the default): the cadastre. `basis-dlm`: the landscape model ATKIS Basis-DLM, whose layers start with `dlm-`. A service is asked only for the layers of its model. Objects of the landscape model stay candidates and go where another source has the same object. |
| `schema` | text | no | `nas` (the default): the answers are NAS, the AdV types as elements. `forest-roads`: the answers are GeoJSON (`OUTPUTFORMAT=GEOJSON`) of the forest road network of the Saarland, whose road class (`WKLASSE`) gives the AdV type and codes of the layers `dlm-fahrweg` and `dlm-weg`; machine roads and skidding lanes are left out; only with `model = "basis-dlm"` and `type_name` (ADR 0155). `shape`: the answers are in the AdV Shape schema (one type per theme and geometry such as `Relief_REL01_P`, the AdV type in `objart_txt`, the codes under short names such as `bwf`), as Rhineland-Palatinate serves its landscape model; only with `model = "basis-dlm"`. |
| `type_name` | text | for `forest-roads` | The one feature type of the road network, such as `Forstwirtschaft:Forstwegenetz`. Only with `schema = "forest-roads"`. |

## `[[extra.trees]]`

At most 32: the tree cadastres of cities, read from an Opendatasoft GeoJSON export.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `id` | text | yes | 2 to 32 lower-case letters, digits and hyphens; once per manifest. |
| `name` | text | yes | At most 80. |
| `crs` | whole number | no | Default: `[adapter] crs`. |
| `envelope` | list of four numbers | yes | The extent of the data in `crs`. |
| `format` | text | yes | `ods-geojson-export` (an OpenDataSoft export, asked by a position field) or `ogc-api` (the items of an OGC API Features collection, asked with a box in CRS84, 10 000 per page, further pages by their `next` links; a Point or a MultiPoint of one point) or `csv` (one file for the whole city, read whole and cut to the area: a header row, no quoting, decimal commas or points) or `wfs-geojson` (a WFS asked for `type_name` in the box of the area with `OUTPUTFORMAT=<output_format>` and up to 100 000 features; GeoJSON in the CRS its `crs` member names, lon/lat without one; a cut answer is refused). |
| `geo_field` | field | for `ods-geojson-export` | The field with the position. Not with the other formats. |
| `x_field`, `y_field` | field | for `csv` | The columns of the easting and the northing, in `crs`. Not with the other formats. |
| `separator` | text | for `csv` | The one character between the values, such as `;`. Not with the other formats. |
| `type_name` | text | for `wfs-geojson` | The feature type, such as `OpenData:Baeume`. Not with the other formats. |
| `output_format` | text | for `wfs-geojson` | The output format the server names for GeoJSON, such as `application/json` or `GEOJSON`. Not with the other formats. |
| `endpoints` | list of URLs | yes | 1 to 8. |
| `fields` | table | yes | `kind`, `id` (both required), `date`, `species`, `crown`, `gone`: the names of the fields. Without `date`, the fetch date is the currency of the trees. A tree with a value in `gone` (such as the date it was felled) is left out. |
| `skipped_kinds` | list of texts | no | Kinds that are no trees; at most 16. |
| `attribution` | text | yes | The credit of the cadastre, at most 400, with `{year}` at most once. |
| `max_bytes` | whole number | yes | 1 to 268 435 456. |

## `[[permits.wfs]]`

A source of the permit notes: a WFS service that answers GeoJSON (or GML 3.2), read by a table of its
layers. Each layer says which attributes give the name, the zone or the office of a finding; the program keeps
those attributes and nothing else. At most 12 sources with 1 to 8 layers each. A manifest that declares one
needs freemapper 0.28.0 or later (`min_freemapper`). The tables of the built-in states move here
one by one (issue #273); `adapters probe` asks every `url`.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `key` | text | yes | The name of the source: lower-case letters, digits and hyphens, at most 40; one per source. The program stores its layers as `<key>-<layer>.geojson`. |
| `label` | `{ en, de }` | yes | Who publishes it, the same words for all sources of one portal; at most 200 characters each. |
| `what` | `{ en, de }` | yes | What the source holds, short; at most 300 characters each. |
| `credit` | text | yes | The credit line or licence text, verbatim as the publisher gives it; at most 400. The notes print it. |
| `gaps` | `{ en, de }` | no | Where the data are not complete although they cover the region; up to 400 characters per language. |
| `sections` | list of texts | yes | The sections of the notes it fills: `districts`, `green-spaces`, `owners`, `monuments`, `protected-objects`, `forest-units`, `state-forest`, `water-protection`, `forest-functions`, `admin-units`, … (the names of the program). |
| `url` | URL | yes | The address of the service, on a host of `[net] hosts`. Unlike every other URL it may carry a fixed query of plain `name=value` pairs that selects the service on the server (`?mapfile=glb`); the requests append their own parameters. |
| `output_format` | text | yes | The `outputFormat` the service names for GeoJSON, URL-encoded as it needs it (`application/geo%2Bjson`). With `gml = true` the one for GML 3.2. |
| `gml` | yes/no | no | The service answers GML 3.2 only. |
| `note` | text | no | A note of the publisher on what the data are worth, verbatim; at most 600 characters. The notes print it with the source. |
| `offices` | text | no | `berlin` or `hamburg`: the source holds the districts of that city state and the notes name the offices of its districts (a source with the reading `district`). |
| `native_crs` | whole number | no | 25832 or 25833: the service finds nothing by a box in WGS 84 or answers only in its own CRS, so it is asked in this one. |
| `layers` | list of tables | yes | The layers, below. |

A layer has `key` (a name as above, unique in its source), `type_name` (the WFS feature type; letters such as `ä` are allowed and go into the request percent-encoded) and `reading`,
which says how its features become findings and which further keys it takes:

| `reading` | Keys | Meaning |
|---|---|---|
| `water` | `name`, `zone` (both required), `citation`, `planned_attribute` with `planned_value` and `planned_label`, `authority`, `name_form` | A water protection area or zone; where the layer has planned zones, the attribute and the value that mark one (the zone then says `planned_label` in brackets, `geplant` by default). `authority` is the attribute of the authority in charge. `name_form = "register"` reads the name as a register writes it: without quotes and the number at its end, in the case of a name. |
| `function` | `function` (required: `recreation`, `protected`, `natural`, `closed`), `name` | Forest with a function or a protection. |
| `unit` | `administration`, `prefix`, `office` (all required), `administration_attribute`, `unit_attribute` with `unit_prefix` | A unit of a forest administration; the office is `prefix` and the attribute `office`. With `administration_attribute` the administration is `administration` followed by the text of that attribute (`Bezirksamt ` and the Bezirk). With `unit_attribute` and `unit_prefix` the finding also names a smaller unit within the office (`Revier ` and its name). `office_unless` and `office_exceptions` (1 to 4 pairs of `name` and `prefix`) say how an office name becomes the name of the notes: a name that holds `office_unless` (in any case) stays as it is, a name of the exceptions gets that prefix instead of `prefix`. `office_renames` (1 or 2 pairs of `starts_with` and `name`) gives an office whose name starts with the text the whole name (where a service cuts a long name). |
| `district` | `name` (required) | A district of a city state, named by the attribute. |
| `green-space` | `name` (required), `fallback`, `district_attribute` with `districts` (1 to 16 names), `dedication`, `name_form` | A public green space: its name, or where that is blank the attribute `fallback`. `district_attribute` holds the number of its district (from 1) in the list `districts`, or, without the list, the name of the district. `dedication` is the attribute that says whether it is dedicated for public use. `name_form = "park"` takes the part of the name before a comma (the park a part belongs to). |
| `playground` | none | A public playground; the notes count them. |
| `owner` | `owner_attribute` (required), `owners` (required, 1 to 8 names) | A parcel of a public owner: the attribute names the owner, and only an owner of the list comes with its name; the notes leave out the name of any other. |
| `state-forest` | `owner`, `prefix`, `office` (all required), `office_unless`, `office_exceptions` | Forest of the state and the office that looks after it; the office name follows the same rules as for a `unit`. |
| `object` | `object` (required: the kind of protected object, such as `natural-monument`, `protected-landscape-part`, `biotope`), `name`, `citation`, `only_attribute` with `only_values` (1 to 8), `point_names` (1 to 4 attributes) | A protected object of nature conservation law; with `only`, a feature counts only when the attribute holds one of the values. A point (a tree, a boulder) takes its name from the first of `point_names` that has a value; an area takes `name`. |
| `monument` | `monument` (required: the kind), `kind_from`, `link_attribute` with `link_prefix` | A listed monument; the link counts when the attribute starts with `link_prefix`. |
| `function-code` | `code_attribute`, `codes` (1 to 8 tables of `code`, `function` and `label`) (all required), `name`, and together `water_prefix`, `water_length`, `zone`, `zone_after` | The function of a forest named by a code. A feature whose code is in `codes` is forest of that `function` (`recreation`, `protected`, `natural`, `closed`), named by the attribute `name` or by `label`. A code of `water_length` characters that starts with `water_prefix` is a zone of a drinking water protection area: the attribute `zone` ends in the zone, after the text `zone_after`. Another code is left out. |

A key of another reading is `manifest.unknown-key`; a key a reading needs and the layer lacks is
`manifest.missing-key`. A program without the reader of these tables answers `manifest.backend-missing`.

## `[[permits.shapefile]]`

A source of the permit notes that is one shapefile in a zip for the whole state. It has the keys of a
`[[permits.wfs]]` source, with these differences: `url` is the address of the zip (https, a host of
`[net] hosts`, no query), `crs` is the CRS of its positions (25832 or 25833), and it has no `output_format`,
`gml`, `native_crs` or `offices`. `layers` holds exactly one layer, with `key` and `reading` as above but without
`type_name`; the program reads the `.shp` and `.dbf` of the zip, keeps the records whose box meets the map area and
of every record only the attributes the reading names. A zip is kept in the download cache and downloaded again
only when the server has a newer one (at most 50 MB). Both tables together hold at most 12 sources; the
sources of `[[permits.wfs]]` come first in the notes, then those of `[[permits.shapefile]]`, each in the order of the
manifest. `adapters probe` asks the zip with a HEAD.

## `[[permits.geojson]]`

A source of the permit notes that is one GeoJSON file for the whole state. It has the keys of a `[[permits.shapefile]]`
source: `url` is the address of the file (https, a host of `[net] hosts`, no query), `crs` the CRS of its positions
(25832 or 25833), `layers` holds exactly one layer without `type_name`; there is no `output_format`, `gml`,
`native_crs` or `offices`. The program reads the file as a `FeatureCollection`, keeps the features whose box meets the
map area and of every one only the attributes the reading names. The file is kept in the download cache and downloaded
again only when the server has a newer one (at most 150 MB); an answer that is no `FeatureCollection` does not replace
it. The notes list the sources of `[[permits.wfs]]` first, then those of `[[permits.shapefile]]` and
`[[permits.geojson]]`.

## `[[permits.arcgis]]`

A source of the permit notes that is an ArcGIS REST map service. It has the keys of a `[[permits.wfs]]` source, but
`url` is the address of the map service (up to `MapServer`, https, a host of `[net] hosts`, no query), and there is no
`output_format`, `gml`, `crs`, `native_crs` or `offices`. The `type_name` of a layer is the number of the layer in the
map service (one to four digits). The program asks the `query` of each layer by the box of the map area in WGS 84,
page by page while the service says more follow, and keeps of every feature only the attributes the reading names.
A service has 1 to 8 layers. The notes list the sources of `[[permits.wfs]]` first, then those of
`[[permits.shapefile]]`, `[[permits.geojson]]` and `[[permits.arcgis]]`.

## `[[permits.getfeatureinfo]]`

A source of the permit notes that is a WMS asked at points: its GetFeatureInfo names what lies at a point, without
its geometry. It has `key`, `label`, `what`, `credit`, `gaps`, `note` and `sections` as a `[[permits.wfs]]` source,
but no `url`, `output_format`, `gml`, `crs`, `native_crs` or `offices`: every layer names its service. A layer has
`key` and `reading` as above, `url` (https, a host of `[net] hosts`, no query), `wms_layers` (up to 24 layer names
of the WMS, separated by commas, asked at once) and `one` (default `false`; `true` where the layers do not overlap,
such as the Reviere of a state: a point then keeps its first feature only, because near a border a server answers for
both sides); it has no `type_name`. The program lays a grid of at most 25 cells over the map area, no side shorter
than 50 m, asks at the centre of each cell and stores each answer with its cell as its area. The notes call the
shares estimates (ADR 0139). The tables of all five kinds hold at most 12 sources together; the notes list the
sources of `[[permits.wfs]]` first, then those of `[[permits.shapefile]]`, `[[permits.geojson]]`, `[[permits.arcgis]]` and `[[permits.getfeatureinfo]]`, each
in the order of the manifest.

## `[point_cloud]`

Where the LAZ tiles of the vegetation candidates come from. Optional; only the built-in adapters use it. The backend of
the state keeps the names of the tiles and how it reads the answers, the manifest holds where (ADR 0189, addendum). A
folder adapter that replaces a built-in state keeps no vegetation from point clouds, because its backend is code.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `kind` | text | yes | `laz-tiles` (tiles named by grid position in one folder), `index-json` (the folder lists its tiles in a JSON index), `meta4` (a Metalink lists the tiles), `currency-csv` (a CSV lists the survey dates) `zip-archives` (archives that each hold many tiles) or `stac-copc` (COPC files that a STAC API lists; `url` is the API, no index). |
| `url` | URL | yes | The folder of the tiles or archives, without a trailing slash. https, a host of `[net] hosts`, no query. |
| `index_url` | URL | for `index-json`, `meta4`, `currency-csv` | The index file; the other kinds refuse it. |

## `[auth]`

HTTP Basic credentials for one host, read from two environment variables. Optional; **only the adapters built into
freemapper may declare it**: an adapter from a folder with this table is left out (`manifest.auth-builtin-only`),
because the table would send a password to a host the folder names.

| Key | Type | Required | Meaning and limits |
|---|---|---|---|
| `host` | text | yes | The one host that gets the `Authorization` header, one of `[net] hosts` (`manifest.auth-host` otherwise). The header goes to no other host, also not along a redirect. |
| `user_env` | text | yes | The variable with the user name: `FREEMAPPER_` and 1 to 40 capitals, digits or `_`. |
| `password_env` | text | yes | The variable with the password, as `user_env`. |
| `products` | list of texts | yes | 1 to 6 names of what a person orders to get an account that works, at most 80 characters each; the message names them when a variable is missing. |

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
| `manifest.auth-host` | The host of `[auth]` is not in `[net] hosts`. |
| `manifest.auth-builtin-only` | An adapter from a folder declares `[auth]`. |
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
| `adapter.duplicate` | The id is taken. A folder replaces a built-in adapter, once, and no other. |
| `adapter.replaces-built-in` | A note, the adapter loads: it takes the place of the built-in adapter with its id. |
| `adapter.replacement-outdated` | A note, the adapter loads: the built-in adapter it replaces has a higher `revision`. |
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
| `check.built-in-id` | An adapter with this id is built into the program; from an adapter folder this one replaces it. |
| `check.built-in-newer` | The built-in adapter with this id has a higher `revision` than the folder; `adapters export` writes it again. |
| `check.prefix-shared` | A built-in adapter has the same `object_id_prefix`. |
| `check.boundary-area` | The boundary encloses under 1 km² or over 2 000 000 km²: probably the wrong unit. |
| `check.shares-ground` | The adapter lies on the ground of another one: the two have more than a hundredth of the smaller region in common. A map there gets the objects of both. |

## Complete examples

The manifests of the built-in states are in
[`crates/adapters/de/manifests/`](https://github.com/HenryJobst/freemapper/tree/main/crates/adapters/de/manifests). Saxony-Anhalt declares every source as a
service. Saarland declares the cadastre and names a backend for the rasters. Hamburg uses an OGC API with a WFS
fallback. Brandenburg has the full ALKIS (`[[extra.nas]]`) and a tree cadastre. Berlin has no `[alkis]`: its
cadastre is code.
