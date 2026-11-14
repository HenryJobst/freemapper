# Bavaria as the fifteenth state (design)

Status: design, 2026-11-13. The user asked for a plan to integrate Bavaria on 2026-11-13. The services were
checked live the same day: `docs/regions/bayern-reference.md` holds every URL, format and size this design
names. Calls marked "Claude's call" were made without asking and are open to the user's veto.

## Intent

freemapper makes maps in fourteen states. Bavaria is the largest one missing, and the one with the most
orienteering clubs still without a base map from freemapper. It was left for last because its cadastre has no
service: buildings and land use are published as files only. This design adds Bavaria as a built-in state on the
adapter scheme (ADR 0082), with the same outputs as every other state, and adds to the readers what files need.

## Goals

1. A map area in Bavaria gives the same project as in any other state: orthophoto, terrain model with hillshade,
   slope, relief and contours, surface model with the nDSM, buildings and land use from the cadastre, the OSM
   objects, the credit line, the area page preview, the estimate, the permit notes of step 1.
2. All three map kinds work (`schoolyard`, `sprint`, `forest`), in EPSG:25832 for the whole state, also east of
   12° E.
3. A run downloads only what the map area needs, in the units the agency publishes, and says the sizes in the
   estimate before it starts.
4. Offline, a run with the cache of an earlier run gives the same map.
5. Two examples: a sprint map `examples/bayern-regensburg` (old town with the Steinerne Brücke and Stadtamhof;
   it lies east of 12° E, so it also shows zone 32 holding there) and a forest map (Claude's call for both
   places; the forest one is the Schmausenbuck in the Nürnberger Reichswald, in the folder name the forest
   examples of the other states use by then).

## Non-goals

- Vegetation from the point clouds (LAZ, class 20). A later plan of its own; the backend of the other five states
  is the pattern. Until then a forest map of Bavaria has what a forest map of Hesse has.
- The ATKIS WFS (rocks, embankments, cuttings, paths as candidates). A later plan on the extra layers of an
  adapter (`2026-11-13-extra-layers-of-an-adapter-design.md`).
- The single trees (Einzelbäume) and the Hausumringe.
- Reading only the part of a DOM20 tile the area needs (range requests on its 256 px tiles). Worth doing when
  the 45 MB per km² hurt in practice.
- A general "bulk vector" source in the manifest. Bavaria is a code adapter; a folder adapter cannot name
  Shape or CityGML files yet, and nobody asked for that.
- Thuringia. It stays the one state without data sources.

## Anti-goals

- Downloading a file of a whole Regierungsbezirk or of the state (Hausumringe 66–174 MB, land use GeoPackage
  5.4 GB, the state Metalinks) to cut a schoolyard out of it.
- Holding a 160 MB CityGML file or a 5000 × 5000 px tile longer in the cache than its reduced form.
- A request that carries anything personal, or goes to a host the manifest does not name.
- A new dependency for a format the house reads itself (the TIFF, zip and XLSX readers are own code).

## Constraints

- House rules for a new state: example project, manual row, README row in both languages, AGENTS.md state list,
  glossary, changelog in both languages under Unveröffentlicht/Unreleased, ADR (next free number, 0123 on
  2026-11-13), `docs/adapters/{de,en}.md` where they list the states, the survey row.
- Source and technical docs English, user docs German and English, German text says "Bundesland".
- Work goes in as a GitHub pull request with the label `all-systems` (file system code), merged after the user
  says yes.
- Golden net: files that list every adapter change; such a commit carries `Golden-Bless: Bavaria added` and is
  no `refactor(` commit. No other golden file changes.
- The tests and texts that use Bavaria as the example of an unsupported area move to Thuringia.

## Identity

| What | Value |
|---|---|
| Adapter id, manifest folder | `bayern` |
| Names | `Bavaria` (en), `Bayern` (de) |
| Provider id, backend name, cache folder | `ldbv` |
| Agency label | `LDBV` |
| CRS | EPSG:25832 |
| Object-id prefix | `DEBY` |
| Hosts | `geoservices.bayern.de`, `download1.bayernwolke.de` |
| Licence | `cc-by-4.0` |
| Credit | `Geobasisdaten: Bayerische Vermessungsverwaltung – www.geodaten.bayern.de (Daten verändert), Lizenz: CC BY 4.0` |
| Short credit | `Bayerische Vermessungsverwaltung` |

The credit is the FAQ's own form for changed data, without a year (the agency names none). The mirror
`download2.bayernwolke.de` is not asked (Claude's call: one host less to trust; a failed download is retried on
the same host as everywhere else).

## The parts

### Rasters: backend `ldbv`

After the pattern of North Rhine-Westphalia (`geobasis_nrw.rs`): the orthophoto from the WMS, both height models
from 1 km tiles, the surface model only a template whose failure costs a warning.

- **Orthophoto**: WMS `…/od/wms/dop/v1/dop20`, layer `by_dop20c`, JPEG, EPSG:25832. Flight date from
  `GetFeatureInfo` on `by_dop20_info`, field `ua` (`DD.MM.YYYY`), with the reader NRW uses.
- **Terrain**: DGM1 tiles, names computed from the map area (`<E>_<N>`), mosaicked with
  `height_model::mosaic_tiles`. No date (the files carry none).
- **Surface**: DOM20 tiles (20 cm). Each tile is downloaded, reduced to 1 m and cached reduced; the 20 cm file is
  not kept. A 1 m cell takes the highest of its 25 values (Claude's call: a surface model shows what stands
  highest; a mean would lower thin crowns and roof edges). The date is `BILDFLUG_DATUM` of the centre tile.

A tile the server does not have (404) is silent when its centre lies outside Bavaria and a warning inside, as in
the other tiled states.

### Buildings: the LoD2 tiles

Claude's call, against the Hausumringe:

| | LoD2 (chosen) | Hausumringe |
|---|---|---|
| Unit | 2 km tile, 3 to 160 MB | Regierungsbezirk, 66 to 174 MB |
| Attributes | function, ALKIS id, date of the outline | none (only the municipality key) |
| Reader | new: CityGML, streamed | new: Shape (needed for land use anyway) |

Why LoD2:

- **The unit.** A schoolyard or a village downloads 3 to 20 MB, not the 66 to 174 MB of its Regierungsbezirk.
- **Identity.** The ALKIS id is the `oid` the engine keeps an object by (an object two sources deliver is kept
  once; the map object carries it as a tag), and the outline has a date. A Hausumring has neither.
- **The kind.** LoD2 says what an object is. A Hausumring of a canopy looks like one of a house.

What the gate found (2026-11-13, Regensburg; numbers in the reference): LoD2 holds 99.70 % of the Hausumringe;
it holds the roofed structures (`51009_1610`, 482 in the box), so Bavaria has canopies; it also holds bridges,
walls and other structures that are no buildings. The engine reads the function of a building only as a tag;
what it needs from this source is the kind (building or roofed structure) and the id.

The price is a CityGML reader and large tiles in city centres.

A new reader `freemapper_sources::citygml` reads a file as a stream (never the whole file in memory) and gives,
per `bldg:Building`: the ground surface polygons without their heights, the function code, the ALKIS object id
and the date of the outline. The adapter turns them into the simplified schema the engine knows:

- `31001_<code>`: `gebnutzbez` "Gebäude", `funktion` the label of the AdV code list the Bremen adapter already
  uses (`assets/rules/adv-codes.toml`, `AX_Gebaeude.gebaeudefunktion`); a code the list does not know leaves
  the `funktion` out. No function at all: a building too.
- `51009_1610` and `51009_1611` (Überdachung, Carport): a structure, as Bremen writes them; the engine draws
  them as canopies.
- any other object type (walls `51009_1700`, bridges `53001_…`, `51007_…`, `52003_…`): left out, as the Bremen
  adapter leaves out the structures that are no roofs. (Changed after the gate: the first draft made them
  buildings, which would have drawn bridges as houses.)
- `oid`: the ALKIS id with the suffix `BL`, as the simplified services write it; without an ALKIS id the `gml:id`.
- `aktualit`: `Grundrissaktualitaet`.
- No `rellage`. (Changed after the gate: the first draft marked a Tiefgarage, `31001_2465`, as underground. The
  five in the box have measured heights of 1.7 to 3.9 m; what LoD2 holds of an underground garage stands above
  the ground, so it is drawn.)

The features of a tile are cached as GeoJSON; the CityGML file is deleted after reading. A building two tiles
hold is kept once.

A tile without buildings exists on the server as an empty `CityModel`; the reader gives no building for it.
A LoD2 tile the server does not have (404) is silent when its centre lies outside Bavaria, and a warning inside:
there it means the file was moved or renamed, and the map would lack buildings without anyone noticing.
(Changed after the gate, which showed that tiles of forest and water do have a file.)

**Gate before the build** (first task of the plan): on the tiles of the Regensburg example, compare LoD2 with
the Hausumringe of the same box (one download of the Oberpfalz zip, 71 MB, into a scratch folder). The measure:
of the Hausumringe with an area of at least 10 m², the share whose inner point lies in a LoD2 ground surface.
Below 98 %, stop and ask the user: the design then needs the Hausumringe as a second source. The gate also
lists the function prefixes with their counts, whether `bldg:BuildingPart` occurs, and what the Hausumringe
without a LoD2 building are (by size).

### Land use: the Kreis files

A new reader `freemapper_sources::shapefile` (own code: polygon shapes and the text fields of the DBF) reads
`Nutzung.shp` and `Nutzung.dbf` out of the zip of a Kreis. The fields are already the simplified schema (`oid`,
`aktualit`, `nutzart`, `bez`, `name`), so the adapter only cuts the map area out and passes them on. Records
are skipped by the box in their record header before their points are read. One rule of the other states
applies: a `Platz` without `bez` gets `ohne Unterart (Bayern)`, so a square whose use the cadastre does not
record stays a candidate instead of becoming a paved map object. The file of a Kreis that answers 404 fails the
fetch (all 96 exist).

Which Kreise a map area touches comes from `assets/kreise/bayern.geojson`, embedded in the program: the 96 outlines with their
five-digit key, made once from the agency's administrative units, simplified 100 m, in the form of the
`osm-*.geojson` files (680 KB; at 40 m it was 1.3 MB). A Kreis is read when its outline comes within 500 m of
the map area; the outlines lie at most 181 m from the source. An object two Kreise hold is kept once.

### Freshness of the files (all four products)

A cached file carries the day of the server's `Last-Modified` in its name. Online, one `HEAD` per file decides:
the same day, and the cache is used; another day, and the file is downloaded and the older one removed. When the
`HEAD` fails for another reason than 404, the newest cached file is used and a note says so, once: the other
cached files of that run are used without asking. Without a cached file the run fails as any download failure
does. Offline, the newest cached file is used without asking. A 404 is remembered for 30 days only for a tile
beyond the border; a file that must exist is asked for again at every run (changed after the final review: one
404 while the agency replaces a file must not lock it out for a month). (Claude's call; the pattern is `tile_fetch.rs` and the
point cloud backends.)

### OpenStreetMap

Seven `[[osm.extract]]` by Regierungsbezirk with a boundary each, as Baden-Württemberg has four. Cache prefixes
`by-<name>`.

### What else a state needs

The registration (`State::Bayern`, `Agency::Ldbv`, the tables and counting tests), the boundary of the state
(made as Baden-Württemberg's: the official outline, simplified 25 m, EPSG:25833, checked for overlaps and gaps
against the three neighbours that have a boundary: Baden-Württemberg, Hesse, Saxony), the estimate (orthophoto per km²; 2.8 MB per DGM1 tile, 45 MB per DOM20
tile, 40 MB per LoD2 tile, 8 MB per Kreis, each only when not cached; the cadastre files under the part
`MapData`), the preview of the area page (the same WMS), the notes in both languages.

## Sizes a person will see

| Map | Downloads besides OSM (first run) |
|---|---|
| Schoolyard, one tile | about 3 + 45 + 20 + 8 MB |
| Sprint, 2 × 2 km in a city | about 11 + 180 + up to 4 LoD2 tiles (12 to 300 MB) + 8 MB |
| Forest 1:10000, 12 km² | about 35 + 540 + 4 to 6 LoD2 tiles (mostly small) + 8 to 16 MB |

The surface model is the heavy part. The run already leaves it out when the disk is short (`skip_surface`).

## Left to the builder

- Names of private functions, the split of `ldbv.rs` into files if it grows past the size of its neighbours.
- The fixtures: cut from fresh answers, a few objects each, public data only.
- The ADR's wording. Its decisions are the two calls above: LoD2 for buildings, files instead of a service.

## Later, each a plan of its own

1. Vegetation from the Bavarian point clouds (class 20).
2. ATKIS extras through the WFS.
3. DOM20 by range.
