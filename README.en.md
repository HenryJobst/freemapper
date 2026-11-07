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

Under development (milestones M0–M2, parts of M3 and M4). Design: [`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Build

```sh
cargo build --release
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

`run` performs three steps that are also available on their own:

- `freemapper fetch <dir>` – downloads the LGB orthophoto (DOP20) and terrain model (DGM1) tiles for
  map area plus buffer into the cache, the ALKIS buildings and Geofabrik's OSM extract for Brandenburg
  (about 300 MB the first time, afterwards only when it is more than 7 days old). A tile is downloaded again only when
  LGB republished it; `--offline` uses data fetched before.
- `freemapper derive <dir>` – computes orthophoto, hillshade and slope templates from them.
- `freemapper export <dir>` – writes the OOM map with the buildings as finished map objects (symbol 521,
  with provenance: ALKIS object id and data currency). Buildings up to the buffer around the map area
  are included; underground parts, structures such as chimneys and very small buildings are left out,
  and outlines are slightly simplified. `--force` replaces an existing map and keeps the old one as
  `.omap.bak`; `run` leaves an existing map untouched without `--force`.

Result in the project folder:

- `freemapper.toml` – map area, map kind, scale, declination date, tiles used with their data currency
- `clara-zetkin-schule.omap` – georeferenced OOM map (EPSG:25833, 1:1000, IOF school symbol set 2019)
- `templates/orthophoto.jpg`, `hillshade.png`, `slope.png` – templates behind the map, each with a
  world file (`.jgw`/`.pgw`); hillshade and slope can be switched on in OOM
- `templates/candidates.omap` – template map with the outline of the map area and candidates from
  OpenStreetMap to check and copy: hydrants, benches, tables, lamps, poles, playground equipment,
  trees, hedges, walls, fences, cliffs, sports pitches (rules in `assets/rules/osm-candidates.toml`)
- peripheral information in the map (map part "peripheral information"): magnetic north lines over the
  map content, below it the title (`--title`, else the folder name), scale and contour interval, a bar
  scale, the attribution of the data sources and the freemapper credit line
- `data/osm-candidates.geojson` – the OSM features of the map area the candidates come from
- `data/alkis-buildings.geojson` – the fetched ALKIS buildings (for `--offline` and later reconciliation)
- `ATTRIBUTION.txt` – attribution of the data sources

The cache lives in the system cache folder (macOS `~/Library/Caches/freemapper`, Linux
`~/.cache/freemapper`, Windows `%LOCALAPPDATA%\freemapper`) and is shared by all projects;
`--cache-dir` picks another folder.

Map kinds: `schoolyard` (schoolyard map, 1:1000) and `sprint` (sprint map, ISSprOM 2019, 1:4000).

## Licence and attribution

GPL-3.0-or-later, see [LICENSE](LICENSE). Generated maps carry the required attribution of the data
sources, e.g. `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` and
`© OpenStreetMap contributors, ODbL`.
