# freemapper

[English version](README.en.md) · [Handbuch](docs/manual/de.md) · [Glossar](docs/glossary.md) · [Änderungen](CHANGELOG.md)

freemapper erzeugt aus amtlichen Geodaten (Geländemodell, nDOM, Luftbild, ALKIS) und OpenStreetMap
ein **Startprojekt für OpenOrienteering Mapper (OOM) und auf Wunsch auch für OCAD**: eine
georeferenzierte Karte, in der verlässliche Kartenobjekte schon symbolisiert sind, plus Kartenvorlagen
für alles, was der Kartierer selbst beurteilen muss. Gezeichnet und im Gelände geprüft wird weiterhin
in OOM oder OCAD.

- **Ausgabe:**
  - immer eine OOM-Karte (`.omap`);
  - mit `--ocad` zusätzlich eine OCAD-Karte (`.ocd`, OCAD 12 und neuer), siehe
    [Ausgangsstand für OCAD](#ausgangsstand-für-ocad);
  - mit `--pdf` eine druckfertige Trainingskarte, siehe
    [Schnelle Trainingskarte](#schnelle-trainingskarte-pdf).
- Läuft als ein einzelnes Programm unter Linux, macOS und Windows, ohne GIS-Installation.
- Bedienung per Kommandozeile oder über KI-Agenten (MCP-Server).
- Regionen: Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen und Rheinland-Pfalz (Open Data von LGB,
  Geoportal Berlin, GeoSN, LVermGeo, LGV, HVBG, LVermGeo SH, LAiV M-V, Geobasis NRW, LGLN und LVermGeo RP);
  Schulhofkarten 1:1000 und Sprintkarten 1:4000.
- Beispiele: je Land ein fertiges Projekt mit OOM-Karte, OCAD-Ausgangsstand und PDF-Trainingskarte, siehe
  [Beispiele](#beispiele).

## Die freemapper-Suite

freemapper ist der erste Schritt einer Werkzeugkette für Orientierungslauf-Karten. Zwei Folgeprojekte
sind in Entwicklung:

- freemapper liefert den Ausgangsstand der Karte,
- die Karte zeichnet man in OOM,
- [freemapper-courseplan](https://github.com/HenryJobst/freemapper-courseplan) legt die Bahnen in einem
  Editor im Browser und prüft sie gegen die Wettkampfbestimmungen,
- [freemapper-print](https://github.com/HenryJobst/freemapper-print) druckt die Kartenblätter, ein Blatt je
  Bahn, mit Bahnaufdruck und Postenbeschreibung.

## Status

Aktuelle Version ist [v0.3.0](https://github.com/HenryJobst/freemapper/releases/tag/v0.3.0) vom
8. November 2026 (was sich geändert hat, steht in den [Änderungen](CHANGELOG.md)). freemapper erzeugt
Schulhof- und Sprintkarten für elf Länder: Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg,
Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen und
Rheinland-Pfalz. Zu jeder Karte gehören ein Ausgangsstand für OCAD und eine PDF-Trainingskarte.
Niedersachsen, Rheinland-Pfalz und die Gewässer auf den Karten kommen mit der nächsten Version.

Das Programm ist noch in Entwicklung. Als Nächstes kommen die übrigen Länder mit offenen Geodaten:
Baden-Württemberg, Bremen, Bayern und das Saarland. Thüringen folgt, sobald sein Server für die
Höhendaten wieder erreichbar ist. Waldkarten und die Nachbarländer kommen später. Design:
[`docs/superpowers/specs/2026-11-07-freemapper-design.md`](docs/superpowers/specs/2026-11-07-freemapper-design.md).

## Handbuch

Das [Handbuch](docs/manual/de.md) führt Schritt für Schritt vom Gebiet zur fertigen Karte, auch ohne Erfahrung
mit der Kommandozeile. Die meistgebrauchten Kapitel:

- [Installation](docs/manual/de.md#3-installation)
- [Mit einem KI-Assistenten](docs/manual/de.md#4-weg-a-mit-einem-ki-assistenten) oder
  [ohne Assistent](docs/manual/de.md#5-weg-b-ohne-assistent)
- [Das Kartengebiet im Browser wählen](docs/manual/de.md#6-das-kartengebiet-im-browser-wählen)
- [Was im Gelände zu prüfen ist](docs/manual/de.md#9-was-im-gelände-zu-prüfen-ist)
- [Meldungen und was zu tun ist](docs/manual/de.md#14-meldungen-und-was-zu-tun-ist)

## Beispiele

Für jedes unterstützte Land liegt ein fertiges Sprintkarten-Projekt im Repository, mit OOM-Karte,
OCAD-Datei und PDF-Trainingskarte. Ein Klick auf das Bild öffnet das PDF. Wie die Gebiete ausgewählt
wurden und wie viele Objekte jede Karte hat, steht in [`examples/README.md`](examples/README.md).

| Land | Gebiet | Vorschau |
|---|---|---|
| Brandenburg | [Park Sanssouci, Potsdam](examples/brandenburg-sanssouci/) | <a href="examples/brandenburg-sanssouci/brandenburg-sanssouci.pdf"><img src="examples/brandenburg-sanssouci/preview.jpg" width="160" alt=""></a> |
| Berlin | [Viktoriapark, Kreuzberg](examples/berlin-viktoriapark/) | <a href="examples/berlin-viktoriapark/berlin-viktoriapark.pdf"><img src="examples/berlin-viktoriapark/preview.jpg" width="160" alt=""></a> |
| Sachsen | [Altstadt Bautzen](examples/sachsen-bautzen/) | <a href="examples/sachsen-bautzen/sachsen-bautzen.pdf"><img src="examples/sachsen-bautzen/preview.jpg" width="160" alt=""></a> |
| Sachsen-Anhalt | [Schlossberg Quedlinburg](examples/sachsen-anhalt-quedlinburg/) | <a href="examples/sachsen-anhalt-quedlinburg/sachsen-anhalt-quedlinburg.pdf"><img src="examples/sachsen-anhalt-quedlinburg/preview.jpg" width="160" alt=""></a> |
| Hamburg | [Treppenviertel Blankenese](examples/hamburg-blankenese/) | <a href="examples/hamburg-blankenese/hamburg-blankenese.pdf"><img src="examples/hamburg-blankenese/preview.jpg" width="160" alt=""></a> |
| Hessen | [Altstadt und Landgrafenschloss Marburg](examples/hessen-marburg/) | <a href="examples/hessen-marburg/hessen-marburg.pdf"><img src="examples/hessen-marburg/preview.jpg" width="160" alt=""></a> |
| Schleswig-Holstein | [Altstadt Lübeck](examples/schleswig-holstein-luebeck/) | <a href="examples/schleswig-holstein-luebeck/schleswig-holstein-luebeck.pdf"><img src="examples/schleswig-holstein-luebeck/preview.jpg" width="160" alt=""></a> |
| Mecklenburg-Vorpommern | [Altstadt Wismar](examples/mecklenburg-vorpommern-wismar/) | <a href="examples/mecklenburg-vorpommern-wismar/mecklenburg-vorpommern-wismar.pdf"><img src="examples/mecklenburg-vorpommern-wismar/preview.jpg" width="160" alt=""></a> |
| Nordrhein-Westfalen | [Altstadt Monschau](examples/nordrhein-westfalen-monschau/) | <a href="examples/nordrhein-westfalen-monschau/nordrhein-westfalen-monschau.pdf"><img src="examples/nordrhein-westfalen-monschau/preview.jpg" width="160" alt=""></a> |
| Niedersachsen | [Großer Garten Herrenhausen, Hannover](examples/niedersachsen-herrenhausen/) | <a href="examples/niedersachsen-herrenhausen/niedersachsen-herrenhausen.pdf"><img src="examples/niedersachsen-herrenhausen/preview.jpg" width="160" alt=""></a> |
| Rheinland-Pfalz | [Reichsburg und Altstadt Cochem](examples/rheinland-pfalz-cochem/) | <a href="examples/rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf"><img src="examples/rheinland-pfalz-cochem/preview.jpg" width="160" alt=""></a> |

## Gebiet im Browser wählen

Am einfachsten zeichnet man das Kartengebiet im Browser:

```sh
freemapper select mein-schulhof --kind schoolyard
```

![Gebietsauswahl im Browser](docs/manual/images/area-selector.jpg)

Auf der Seite sucht man einen Ort oder zeichnet ein Rechteck oder Polygon. Der Hintergrund ist
OpenStreetMap oder das Luftbild des Landes. Die Seite zeigt die Größe mit Puffer und den
geschätzten Download. „Speichern“ legt das Projekt an, danach erzeugt
`freemapper run mein-schulhof --pdf --ocad` die Karten. Die Seite läuft nur auf diesem Rechner
(127.0.0.1). Mehr im [Handbuch, Kapitel 6](docs/manual/de.md#6-das-kartengebiet-im-browser-wählen).

## Installation

freemapper ist ein einzelnes Programm ohne Installer. Es muss nicht selbst gebaut werden:

1. Auf der [Release-Seite](https://github.com/HenryJobst/freemapper/releases/latest) das Archiv für
   das eigene System herunterladen:
   - Windows: `freemapper-…-windows-x86_64.zip`
   - macOS mit Apple-Chip (M1 und neuer): `freemapper-…-macos-arm64.tar.gz`
   - macOS mit Intel-Prozessor: `freemapper-…-macos-x86_64.tar.gz`
   - Linux: `freemapper-…-linux-x86_64.tar.gz` (ARM-Rechner: `linux-arm64`)
2. Das Archiv entpacken (Doppelklick). Der Ordner enthält `freemapper` bzw. `freemapper.exe`.
3. Beim ersten Start warnt das System, weil das Programm nicht signiert ist:
   - **Windows:** „Der Computer wurde durch Windows geschützt“ → *Weitere Informationen* →
     *Trotzdem ausführen*.
   - **macOS:** „kann nicht geöffnet werden, da der Entwickler nicht verifiziert werden kann“ →
     *Systemeinstellungen › Datenschutz & Sicherheit* → *Trotzdem öffnen*; oder im Terminal im
     entpackten Ordner `xattr -d com.apple.quarantine freemapper`.
   - **Linux:** keine Warnung; ggf. `chmod +x freemapper`.
4. Prüfen im Terminal (Windows: PowerShell) im entpackten Ordner: `./freemapper --version`
   (Windows: `.\freemapper.exe --version`).

Bequemer geht es mit einem KI-Assistenten (siehe [Mit einem KI-Agenten](#mit-einem-ki-agenten-mcp)):
dort wird freemapper einmal mit seinem vollständigen Pfad angemeldet, danach erledigt der Assistent
den Rest.

## Selbst bauen (für Entwickler)

```sh
cargo build --release --locked
./target/release/freemapper --version
```

Ein Tag `v<Version>` (passend zur Version in `Cargo.toml`) erzeugt über
`.github/workflows/release.yml` die Archive für alle Systeme und das GitHub-Release.

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

Oder das Gebiet im Browser wählen, siehe [Gebiet im Browser wählen](#gebiet-im-browser-wählen).

`run` führt drei Schritte nacheinander aus, die es auch einzeln gibt:

- `freemapper fetch <ordner>` – lädt für Kartengebiet plus Puffer Luftbild, Geländemodell und nDOM des
  zuständigen Landes, die ALKIS-Gebäude und -Nutzung sowie den OSM-Auszug des Landes von Geofabrik
  (beim ersten Mal 180–300 MB, danach nur, wenn er mehr als 7 Tage alt ist) in den Zwischenspeicher.
  Neu geladen wird nur, was neu veröffentlicht wurde; `--offline` nutzt nur bereits geladene Daten.
  Woher die Daten je Land kommen, steht unter [Regionen](#regionen).
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

## Regionen

| Land | Daten | nDOM-Vorlage | Hinweise |
|---|---|---|---|
| Brandenburg | LGB | Laser-nDOM, 0,2 m | |
| Berlin | LGB, ALKIS vom Geoportal Berlin | aus dem bildbasierten Oberflächenmodell, 0,2 m | Plätze nur als Kandidaten; Straßenflächen mit Grünstreifen |
| Sachsen | GeoSN | DOM1 − DGM1, 1 m | Plätze nur als Kandidaten; Straßenflächen mit Grünstreifen; keine Lage unter der Erdoberfläche |
| Sachsen-Anhalt | LVermGeo | DOM1 − DGM1, 1 m | Karte in UTM-Zone 32 (EPSG:25832); Plätze nur als Kandidaten; Straßenflächen mit Grünstreifen |
| Hamburg | LGV | bDOM − DGM1, 1 m (Stand 2022) | Karte in UTM-Zone 32 (EPSG:25832); Höhenmodelle älter als das Luftbild (2026) |
| Hessen | HVBG | DOM1 − DGM1, 1 m (Laserscan) | Karte in UTM-Zone 32 (EPSG:25832); Luftbild im Sommer beflogen; Plätze nur als Kandidaten; Straßenflächen mit Grünstreifen |
| Schleswig-Holstein | LVermGeo SH | bDOM − DGM1, 1 m (bildbasiert) | Karte in UTM-Zone 32 (EPSG:25832); Luftbild je nach Gebiet unbelaubt oder belaubt; Oberflächenmodell als ganze Kacheln (105 MB je km²) |
| Mecklenburg-Vorpommern | LAiV M-V | DOM1 − DGM1, 1 m | Luftbild als Farbpalette (256 Farben); Höhenmodelle nur innerhalb der Landeskacheln |
| Nordrhein-Westfalen | Geobasis NRW | DOM1 − DGM1, 1 m (Laserscan) | Karte in UTM-Zone 32 (EPSG:25832); OSM aus den Auszügen der Regierungsbezirke |
| Niedersachsen | LGLN | DOM1 − DGM1, 1 m (Laserscan) | Karte in UTM-Zone 32 (EPSG:25832); Oberflächenmodell oft Jahre älter als das Luftbild; Bremen noch nicht dabei |
| Rheinland-Pfalz | LVermGeo RP | DOM1 − DGM1, 1 m (Laserscan) | Karte in UTM-Zone 32 (EPSG:25832); Luftbild im Sommer beflogen, als Flugdatum gilt das Erstellungsdatum; Luftbild jenseits der Landesgrenze weiß |

An einer Landesgrenze fragt freemapper das Kataster beider Länder ab und liest beide OSM-Auszüge;
Luftbild und Gelände kommen aus dem Land, in dem die Mitte des Kartengebiets liegt. Mehr dazu im
[Handbuch, Kapitel 12](docs/manual/de.md#12-regionen-und-grenzgebiete). Ein Beispielprojekt je Land liegt unter
[`examples/`](examples/).

## Schnelle Trainingskarte (PDF)

```sh
freemapper export clara-zetkin-schule --pdf      # oder: freemapper run clara-zetkin-schule --pdf
```

schreibt `<name>.pdf`: eine voll generierte, druckfertige Karte (A4 oder A3, CMYK). Anders als im
Startprojekt werden hier **alle Kandidaten zu Kartenobjekten**: Möblierung, Barrieren, Wege ohne
Oberflächenangabe, Höhenlinien, Sperrgebiete und ALKIS-Wege. Gezeichnet wird mit dem Symbolsatz des
Projekts so, wie OOM ihn zeichnet (Strichelungen, Zeichen auf Linien, Farbreihenfolge), dazu die
Randangaben mit der Namensnennung und dem Hinweis auf freemapper. Das Luftbild gehört nicht dazu.
Weil auch unsichere Daten eingezeichnet sind, ist die Karte vor dem Training im Gelände zu prüfen.

## Ausgangsstand für OCAD

```sh
freemapper export clara-zetkin-schule --ocad      # oder: freemapper run clara-zetkin-schule --ocad
```

schreibt zusätzlich den Ordner `ocad/`:

- `<name>.ocd`: Karte im OCAD-12-Format (sollte sich in OCAD 12 und neuer öffnen), mit Georeferenz
  (ETRS89/UTM 33), Farben, den Kartenobjekten und den Vorlagen als Hintergrundkarten.
- `<name>-candidates.ocd`: die Kandidaten als eigene OCAD-Karte.
- Luftbild, Schummerung, Hangneigung und nDOM mit Weltdateien, direkt neben der `.ocd`.

Die Symbole in der `.ocd` sind Platzhalter: richtige Nummer, Name und Hauptfarbe, aber einfache
Grafik. In OCAD lädt man den offiziellen Satz (*Symbol › Farben und Symbole laden aus…*, ISSprOM 2019
für Sprint, IOF-Schulsatz 2019 für Schulhof), der die Platzhalter nach Nummer ersetzt.
- Die Nummern folgen OCADs offiziellen Sätzen, z. B. 501.001 für befestigte Fläche und 505.000/505.001
  für Wege.
- Eine vorhandene `.ocd` wird nur mit `--force` ersetzt (die alte bleibt als `.ocd.bak`); eine
  vorhandene OOM-Karte bleibt bei `--ocad` ohne `--force` unverändert.

Die Dateien sind noch nicht in OCAD geprüft. Was zu prüfen ist, steht in
[`docs/ocad/test-checklist.md`](docs/ocad/test-checklist.md), z. B. ob „Ersetzen“ beim Laden des
Symbolsatzes die Randangaben-Symbole (991–994) behält und ob OCAD die Kandidaten-Karte deckungsgleich
einblendet (sonst als Hintergrundkarte von Hand öffnen). Ordnernamen mit Umlauten werden für die
OCAD-Dateien in ASCII geschrieben („Straußberg“ → „Straussberg“).

## Mit einem KI-Agenten (MCP)

`freemapper mcp` ist ein MCP-Server auf stdin/stdout. Ein KI-Agent kann damit die Karte für jemanden
erstellen, der keine Kommandozeile benutzen möchte: Man nennt dem Agenten die Schule, er erledigt den Rest.

In Claude Code anmelden (mit dem Pfad, unter dem freemapper entpackt wurde):

```sh
claude mcp add freemapper -- /Pfad/zu/freemapper mcp
```

Andere MCP-Clients (allgemeine Konfiguration; `--cache-dir <ordner>` ist optional):

```json
{
  "mcpServers": {
    "freemapper": { "command": "/Pfad/zu/freemapper", "args": ["mcp"] }
  }
}
```

Ablauf, wie der Agent ihn geht:

1. `find_place` – sucht die Schule per Name und zeigt die Treffer; ein Treffer mit Fläche wird das
   Kartengebiet.
2. `create_project` – legt den Projektordner an (absoluter Pfad oder `~/…`), Kartenart und Maßstab
   wählbar.
3. `estimate_download` – sagt, wie viel geladen werden muss (beim ersten Mal etwa 300 MB OSM-Auszug).
4. `start_pipeline` – startet Laden, Berechnen und Schreiben im Hintergrund und gibt eine Job-ID zurück
   (mit `ocad: true` auch den OCAD-Ausgangsstand, mit `pdf: true` die Trainingskarte).
5. `project_status` – fragt den Fortschritt ab, bis der Job fertig oder fehlgeschlagen ist.
6. `list_layers` und `preview_layer` – zeigen die Ebenen und Vorschaubilder (PNG) der Vorlagen.

Passt kein Suchtreffer oder will man das Gebiet selbst zeichnen, öffnet `open_area_selector` die
Gebietsauswahl im Browser und gibt die Adresse zurück; nach dem Speichern geht es mit Schritt 3 weiter.

Danach öffnet man die `.omap`-Datei im Projektordner mit OpenOrienteering Mapper.

## Lizenz und Namensnennung

GPL-3.0-or-later, siehe [LICENSE](LICENSE). Erzeugte Karten enthalten die Pflichtvermerke der
Datenquellen, z. B. `© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0, Daten geändert` und
`© OpenStreetMap contributors, ODbL`. In Berlin lautet der LGB-Vermerk, wie vom LGB verlangt,
`© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten geändert`; wurden
Berliner ALKIS-Daten geladen, kommt `© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert` hinzu.
In Sachsen lautet der Vermerk `Quelle: GeoSN, dl-de/by-2-0, Daten geändert`.
