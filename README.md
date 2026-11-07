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

In Entwicklung (Meilensteine M0–M7; Gebietsauswahl im Browser, OCAD und PDF folgen). Design: [`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Bauen

```sh
cargo build --release --locked
./target/release/freemapper --version
```

Entwicklung: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
Git-Hooks: `lefthook install`.

## Benutzung

```sh
freemapper init clara-zetkin-schule --geojson tests/fixtures/clara-zetkin-schule.geojson --kind schoolyard \
  --title "Clara-Zetkin-Schule Strausberg"
freemapper run clara-zetkin-schule
```

Statt einer GeoJSON-Datei geht auch das OSM-Objekt (`--osm way/992423119`) oder der Name
(`--place "Clara-Zetkin-Schule Strausberg"`; genommen wird der beste Treffer mit Fläche, gesucht über
Nominatim von OpenStreetMap).

`run` führt drei Schritte nacheinander aus, die es auch einzeln gibt:

- `freemapper fetch <ordner>` – lädt Luftbild- (DOP20) und Geländemodell-Kacheln (DGM1) des LGB für
  Kartengebiet plus Puffer in den Zwischenspeicher, dazu das nDOM (etwa 15 MB je Kachel; erneuert,
  wenn der Server ein neueres Änderungsdatum meldet), die ALKIS-Gebäude und den OSM-Auszug für
  Brandenburg von Geofabrik (beim ersten Mal etwa 300 MB, danach nur, wenn er mehr als 7 Tage alt ist). Kacheln werden nur neu
  geladen, wenn das LGB sie neu veröffentlicht hat; `--offline` nutzt nur bereits geladene Daten.
- `freemapper derive <ordner>` – berechnet daraus Luftbild, Schummerung, Hangneigung und das nDOM
  (Höhe über Grund: Hecken, Mauern, Bäume, Spielgeräte) als Vorlagen sowie Höhenlinien
  (Schulhof 1 m, Sprint 2 m; jede fünfte als Zählkurve) als Kandidaten.
- `freemapper export <ordner>` – schreibt die OOM-Karte mit den Gebäuden als fertigen Kartenobjekten
  (Symbol 521, mit Herkunft: ALKIS-Objekt-ID und Datenstand). Übernommen werden Gebäude bis zum Puffer
  um das Kartengebiet; unterirdische Bauteile und Bauwerke wie Schornsteine entfallen, sehr kleine
  Gebäude ebenso, Umrisse werden leicht vereinfacht. `--force` ersetzt eine vorhandene Karte und legt
  die alte als `.omap.bak` ab; `run` lässt eine vorhandene Karte ohne `--force` unverändert.

Ergebnis im Projektordner:

- `freemapper.toml` – Kartengebiet, Kartenart, Maßstab, Datum der Missweisung, verwendete Kacheln mit
  Datenstand
- `clara-zetkin-schule.omap` – georeferenzierte OOM-Karte (EPSG:25833, 1:1000, IOF-Schulsymbolsatz 2019)
- `templates/orthophoto.jpg`, `hillshade.png`, `slope.png`, `ndsm.png` – Vorlagen hinter der Karte,
  jeweils mit Weltdatei (`.jgw`/`.pgw`); Schummerung, Hangneigung und nDOM lassen sich in OOM
  einblenden. nDOM-Farben: gelb 0,3–1 m, orange 1–2,5 m, hellgrün 2,5–8 m, dunkelgrün über 8 m
- `templates/candidates.omap` – Vorlagenkarte mit dem Umriss des Kartengebiets und Kandidaten aus
  OpenStreetMap zum Prüfen und Übernehmen: Hydranten, Bänke, Tische, Laternen, Masten, Spielgeräte,
  Bäume, Hecken, Mauern, Zäune, Felskanten, Sportplätze (Regeln in `assets/rules/osm-candidates.toml`),
  Höhenlinien und verbotenes Gebiet (520) aus der ALKIS-Nutzung: auf der Schulhofkarte alles außer
  dem Schulgrundstück, auf der Sprintkarte Privatgelände (Wohn-, Gewerbe-, Mischnutzungs- und
  Bahnflächen)
- Randangaben in der Karte (Kartenteil „peripheral information“): Nordlinien über dem Kartenbild,
  darunter Titel (`--title`, sonst der Ordnername), Maßstab und Äquidistanz, Maßstabsleiste,
  Namensnennung der Datenquellen und der Hinweis auf freemapper
- `data/osm-candidates.geojson` – die OSM-Objekte des Kartengebiets, aus denen die Kandidaten entstehen
- `data/contours.geojson` – die Höhenlinien aus `derive`
- `data/alkis-land-use.geojson` – die geladene ALKIS-Nutzung der Flurstücke
- `data/alkis-buildings.geojson` – die geladenen ALKIS-Gebäude (für `--offline` und spätere Abgleiche)
- `ATTRIBUTION.txt` – Namensnennung der Datenquellen

Der Zwischenspeicher liegt im Cache-Ordner des Systems (macOS `~/Library/Caches/freemapper`, Linux
`~/.cache/freemapper`, Windows `%LOCALAPPDATA%\freemapper`) und wird von allen Projekten geteilt;
`--cache-dir` wählt einen anderen Ordner.

Kartenarten: `schoolyard` (Schulhofkarte, 1:1000) und `sprint` (Sprintkarte, ISSprOM 2019, 1:4000).

Die Sprintkarte enthält zusätzlich, als fertige Kartenobjekte:

- befestigte Flächen (501.5) aus der ALKIS-Nutzung: Fahrbahnen mit Gehwegen und Parkstreifen
  (`Straßenverkehr`) sowie Plätze, Parkplätze, Marktplätze und Fußgängerzonen (`Platz`). Begleitflächen
  (Grünstreifen, Baumreihen) werden nicht gezeichnet. Nur als Kandidat kommen: ALKIS-`Weg` (in der Stadt
  Gehwege und Gassen, im Wald unbefestigte Wege), andere Plätze (z. B. Festplätze) und Straßenflurstücke,
  auf denen OSM eine unbefestigte Straße kennt (Sandstraßen). Straßenflächen unter Baumkronen am
  Luftbild prüfen.
- unbefestigte Wege aus OSM mit Angabe zur Oberfläche (`surface=dirt`, `gravel`, `compacted` …):
  Wirtschaftswege 505.2, Fußwege und Pfade 505.1, schmale (`width` unter 1,5 m) oder schwer
  erkennbare Pfade 506. Kandidaten (505.1) sind Wege ohne `surface` oder mit ungewöhnlicher Oberfläche
  sowie Durchgänge durch Gebäude, überdachte Wege und Fußgängerflächen; unbefestigte Straßen sind
  Kandidaten 505.2. Befestigte Wege und Gehwege an Straßen übernimmt freemapper nicht, die zeigen
  ALKIS-Flächen und Luftbild. Alle Linien enden am Puffer um das Kartengebiet.

Beispiel Strausberg Vorstadt:

```sh
freemapper init vorstadt --geojson tests/fixtures/strausberg-vorstadt-sprint.geojson --kind sprint \
  --title "Strausberg Vorstadt"
freemapper run vorstadt
```

## Mit einem KI-Agenten (MCP)

`freemapper mcp` ist ein MCP-Server auf stdin/stdout. Ein KI-Agent kann damit die Karte für jemanden
erstellen, der keine Kommandozeile benutzen möchte: Man nennt dem Agenten die Schule, er erledigt den Rest.

In Claude Code anmelden:

```sh
claude mcp add freemapper -- freemapper mcp
```

Andere MCP-Clients (allgemeine Konfiguration; `--cache-dir <ordner>` ist optional):

```json
{
  "mcpServers": {
    "freemapper": { "command": "freemapper", "args": ["mcp"] }
  }
}
```

Ablauf, wie der Agent ihn geht:

1. `find_place` – sucht die Schule per Name und zeigt die Treffer; ein Treffer mit Fläche wird das
   Kartengebiet.
2. `create_project` – legt den Projektordner an (absoluter Pfad oder `~/…`), Kartenart und Maßstab
   wählbar.
3. `estimate_download` – sagt, wie viel geladen werden muss (beim ersten Mal etwa 300 MB OSM-Auszug).
4. `start_pipeline` – startet Laden, Berechnen und Schreiben im Hintergrund und gibt eine Job-ID zurück.
5. `project_status` – fragt den Fortschritt ab, bis der Job fertig oder fehlgeschlagen ist.
6. `list_layers` und `preview_layer` – zeigen die Ebenen und Vorschaubilder (PNG) der Vorlagen.

Danach öffnet man die `.omap`-Datei im Projektordner mit OpenOrienteering Mapper.

## Lizenz und Namensnennung

GPL-3.0-or-later, siehe [LICENSE](LICENSE). Erzeugte Karten enthalten die Pflichtvermerke der
Datenquellen, z. B. `© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0, Daten geändert` und
`© OpenStreetMap contributors, ODbL`.
