# freemapper

[Deutsche Version](README.md) · [Glossary](docs/glossary.md)

freemapper turns official geodata (terrain model, nDSM, orthophoto, ALKIS cadastre) and OpenStreetMap
into a **starter project for OpenOrienteering Mapper (OOM)**: a georeferenced `.omap` file with the
reliable map objects already symbolized, plus templates for everything the mapper has to judge.
Drawing and field checking stay in OOM.

- A single program for Linux, macOS and Windows, no GIS installation needed.
- Use it from the command line or through AI agents (MCP server).
- First: Brandenburg (LGB open data), schoolyard maps 1:1000 and sprint maps 1:4000.

## Status

Under development (milestones M0–M8; OCAD and PDF follow). Design: [`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Build

```sh
cargo build --release --locked
./target/release/freemapper --version
```

Development: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
Git hooks: `lefthook install`.

## Usage

```sh
freemapper init clara-zetkin-schule --geojson tests/fixtures/clara-zetkin-schule.geojson --kind schoolyard \
  --title "Clara-Zetkin-Schule Strausberg"
freemapper run clara-zetkin-schule
```

Instead of a GeoJSON file, give the OSM object (`--osm way/992423119`) or the name
(`--place "Clara-Zetkin-Schule Strausberg"`; the best match with an area is used, searched with
OpenStreetMap's Nominatim).

Or choose the area in the browser:

```sh
freemapper select clara-zetkin-schule --kind schoolyard
```

The page opens in the default browser (`--no-browser` only prints the address). Search a place there, or
draw a rectangle (click two corners) or a polygon (click the corners, double-click closes). The
background is OpenStreetMap or the LGB aerial photo. The page shows the LGB 1 km tiles, the size with
buffer and the estimated download; "Save" creates the project or changes the map area of an existing
one (an existing map stays as it is), "Done" ends the page. The page runs on this computer only
(127.0.0.1, random address).

`run` performs three steps that are also available on their own:

- `freemapper fetch <dir>` – downloads the LGB orthophoto (DOP20) and terrain model (DGM1) tiles for
  map area plus buffer into the cache, the nDSM (about 15 MB per tile; refreshed when the server reports
  a newer modification date), the ALKIS buildings and Geofabrik's OSM extract for Brandenburg
  (about 300 MB the first time, afterwards only when it is more than 7 days old). A tile is downloaded again only when
  LGB republished it; `--offline` uses data fetched before.
- `freemapper derive <dir>` – computes orthophoto, hillshade, slope and nDSM (height above ground:
  hedges, walls, trees, playground equipment) templates from them, and contour lines (schoolyard
  1 m, sprint 2 m; every fifth an index contour) as candidates.
- `freemapper export <dir>` – writes the OOM map with the buildings as finished map objects (symbol 521,
  with provenance: ALKIS object id and data currency). Buildings up to the buffer around the map area
  are included; underground parts, structures such as chimneys and very small buildings are left out,
  and outlines are slightly simplified. `--force` replaces an existing map and keeps the old one as
  `.omap.bak`; `run` leaves an existing map untouched without `--force`.

Result in the project folder:

- `freemapper.toml` – map area, map kind, scale, declination date, tiles used with their data currency
- `clara-zetkin-schule.omap` – georeferenced OOM map (EPSG:25833, 1:1000, IOF school symbol set 2019)
- `templates/orthophoto.jpg`, `hillshade.png`, `slope.png`, `ndsm.png` – templates behind the map,
  each with a world file (`.jgw`/`.pgw`); hillshade, slope and nDSM can be switched on in OOM. nDSM
  colours: yellow 0.3–1 m, orange 1–2.5 m, light green 2.5–8 m, dark green above 8 m
- `templates/candidates.omap` – template map with the outline of the map area and candidates from
  OpenStreetMap to check and copy: hydrants, benches, tables, lamps, poles, playground equipment,
  trees, hedges, walls, fences, cliffs, sports pitches (rules in `assets/rules/osm-candidates.toml`),
  contours, and out-of-bounds areas (520) from the ALKIS land use: on the schoolyard map everything but
  the school's own parcel, on the sprint map private land (residential, commercial, mixed-use and
  railway land)
- peripheral information in the map (map part "peripheral information"): magnetic north lines over the
  map content, below it the title (`--title`, else the folder name), scale and contour interval, a bar
  scale, the attribution of the data sources and the freemapper credit line
- `data/osm-candidates.geojson` – the OSM features of the map area the candidates come from
- `data/contours.geojson` – the contour lines from `derive`
- `data/alkis-land-use.geojson` – the fetched ALKIS land use per parcel part
- `data/alkis-buildings.geojson` – the fetched ALKIS buildings (for `--offline` and later reconciliation)
- `ATTRIBUTION.txt` – attribution of the data sources

The cache lives in the system cache folder (macOS `~/Library/Caches/freemapper`, Linux
`~/.cache/freemapper`, Windows `%LOCALAPPDATA%\freemapper`) and is shared by all projects;
`--cache-dir` picks another folder.

Map kinds: `schoolyard` (schoolyard map, 1:1000) and `sprint` (sprint map, ISSprOM 2019, 1:4000).

The sprint map also contains, as finished map objects:

- paved areas (501.5) from the ALKIS land use: carriageways with sidewalks and parking lanes
  (`Straßenverkehr`) and squares, car parks, market squares and pedestrian zones (`Platz`). Verges
  (green strips, tree rows) are not drawn. Candidates only: ALKIS `Weg` (sidewalks and lanes in town,
  unpaved tracks in the woods), other squares (e.g. fairgrounds) and street parcels on which OSM knows
  an unpaved street (sand roads). Check street areas under tree canopies against the orthophoto.
- unpaved ways from OSM that state their surface (`surface=dirt`, `gravel`, `compacted` …): tracks
  505.2, footpaths and paths 505.1, narrow (`width` below 1.5 m) or faint paths 506. Candidates (505.1)
  are ways without `surface` or with an unusual one, passages through buildings, covered ways and
  pedestrian areas; unpaved streets are candidates 505.2. freemapper leaves out paved ways and
  sidewalks, which the ALKIS areas and the orthophoto show. All lines end at the buffer around the map
  area.

Example Strausberg Vorstadt:

```sh
freemapper init vorstadt --geojson tests/fixtures/strausberg-vorstadt-sprint.geojson --kind sprint \
  --title "Strausberg Vorstadt"
freemapper run vorstadt
```

## Quick training map (PDF)

```sh
freemapper export clara-zetkin-schule --pdf      # or: freemapper run clara-zetkin-schule --pdf
```

writes `<name>.pdf`: a fully generated map ready to print (A4 or A3, CMYK). Unlike the starter
project, **every candidate becomes a map object** here: furniture, barriers, paths without a surface
tag, contours, out-of-bounds areas and ALKIS ways. It is drawn with the project's symbol set the way
OOM draws it (dashes, symbols along lines, colour order), with the peripheral information, the data
credits and the freemapper credit line. The orthophoto is not part of it. Because uncertain data is
drawn too, check the map in the terrain before training.

## Starting point for OCAD

```sh
freemapper export clara-zetkin-schule --ocad      # or: freemapper run clara-zetkin-schule --ocad
```

also writes the folder `ocad/`:

- `<name>.ocd`: the map in OCAD 12 format (expected to open in OCAD 12 and newer), with georeferencing
  (ETRS89/UTM 33), colours, the map objects and the templates as background maps.
- `<name>-candidates.ocd`: the candidates as a separate OCAD map.
- Orthophoto, hillshade, slope and nDSM with world files, right next to the `.ocd`.

The symbols in the `.ocd` are placeholders: right number, name and main colour, but simple graphics.
In OCAD, load the official set (*Symbol › Load Colors and Symbols From…*: ISSprOM 2019 for sprint, the
IOF school symbol set 2019 for schoolyards); it replaces the placeholders by number.
- The numbers follow OCAD's official sets, e.g. 501.001 for paved area and 505.000/505.001 for paths.
- An existing `.ocd` is only replaced with `--force` (the old one is kept as `.ocd.bak`); with `--ocad`
  an existing OOM map stays as it is unless `--force` is given.

The files have not been checked in OCAD yet. What to check is listed in
[`docs/ocad/test-checklist.md`](docs/ocad/test-checklist.md), e.g. whether "Replace" when loading the
symbol set keeps the sheet symbols (991–994) and whether OCAD shows the candidates map aligned (else
open it as a background map by hand). Folder names with umlauts are written in ASCII for the OCAD
files ("Straußberg" → "Straussberg").

## With an AI agent (MCP)

`freemapper mcp` is an MCP server on stdin/stdout. An AI agent can use it to make the map for someone
who would rather not use a command line: tell the agent the school, it does the rest.

Register it in Claude Code:

```sh
claude mcp add freemapper -- freemapper mcp
```

Other MCP clients (generic configuration; `--cache-dir <folder>` is optional):

```json
{
  "mcpServers": {
    "freemapper": { "command": "freemapper", "args": ["mcp"] }
  }
}
```

The workflow the agent follows:

1. `find_place` – searches the school by name and lists the matches; a match with an area becomes the
   map area.
2. `create_project` – creates the project folder (absolute path or `~/…`), map kind and scale optional.
3. `estimate_download` – tells how much will be downloaded (about 300 MB OSM extract the first time).
4. `start_pipeline` – starts downloading, computing and writing in the background and returns a job id
   (with `ocad: true` also the OCAD starting point, with `pdf: true` the training map).
5. `project_status` – polls the progress until the job has finished or failed.
6. `list_layers` and `preview_layer` – show the layers and preview images (PNG) of the templates.

When no search result fits or the user wants to draw the area, `open_area_selector` opens the area
selection in the browser and returns its address; after saving, continue with step 3.

Then open the `.omap` file in the project folder with OpenOrienteering Mapper.

## Licence and attribution

GPL-3.0-or-later, see [LICENSE](LICENSE). Generated maps carry the required attribution of the data
sources, e.g. `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` and
`© OpenStreetMap contributors, ODbL`.
