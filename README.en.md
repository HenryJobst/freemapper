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

Under development (milestone M0/M1). Design: [`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Build

```sh
cargo build --release
./target/release/freemapper --version
```

Development: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
Git hooks: `lefthook install`.

## Licence and attribution

GPL-3.0-or-later, see [LICENSE](LICENSE). Generated maps carry the required attribution of the data
sources, e.g. `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` and
`© OpenStreetMap contributors, ODbL`.
