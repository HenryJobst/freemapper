# freemapper

[English version](README.en.md) · [Glossar](docs/glossary.md)

freemapper erzeugt aus amtlichen Geodaten (Geländemodell, nDOM, Luftbild, ALKIS) und OpenStreetMap
ein **Startprojekt für OpenOrienteering Mapper (OOM)**: eine georeferenzierte `.omap`-Datei, in der
verlässliche Kartenobjekte schon symbolisiert sind, plus Kartenvorlagen für alles, was der Kartierer
selbst beurteilen muss. Gezeichnet und im Gelände geprüft wird weiterhin in OOM.

- Läuft als ein einzelnes Programm unter Linux, macOS und Windows, ohne GIS-Installation.
- Bedienung per Kommandozeile oder über KI-Agenten (MCP-Server).
- Zuerst: Brandenburg (LGB-Open-Data), Schulhofkarten 1:1000 und Sprintkarten 1:4000.

## Status

In Entwicklung (Meilenstein M0/M1). Design: [`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Bauen

```sh
cargo build --release
./target/release/freemapper --version
```

Entwicklung: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
Git-Hooks: `lefthook install`.

## Benutzung (Stand M1)

```sh
freemapper init clara-zetkin-schule --geojson tests/fixtures/clara-zetkin-schule.geojson --kind schoolyard
freemapper export clara-zetkin-schule
```

Ergebnis im Projektordner:

- `freemapper.toml` – Kartengebiet, Kartenart, Maßstab, Datum der Missweisung
- `clara-zetkin-schule.omap` – georeferenzierte OOM-Karte (EPSG:25833, 1:1000, IOF-Schulsymbolsatz 2019)
- `templates/candidates.omap` – Vorlagenkarte mit dem Umriss des Kartengebiets
- `ATTRIBUTION.txt` – Namensnennung der Datenquellen

Kartenarten: `schoolyard` (Schulhofkarte, 1:1000) und `sprint` (Sprintkarte, ISSprOM 2019, 1:4000).

## Lizenz und Namensnennung

GPL-3.0-or-later, siehe [LICENSE](LICENSE). Erzeugte Karten enthalten die Pflichtvermerke der
Datenquellen, z. B. `© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0, Daten geändert` und
`© OpenStreetMap contributors, ODbL`.
