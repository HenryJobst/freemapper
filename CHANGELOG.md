# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

## [0.3.0] - 2026-11-08

### Hinzugefügt

- Hamburg (LGV): Orthophoto aus dem WMS (Stand 2026), Gelände- und Oberflächenmodell aus den Archiven des Landes, Gebäude und Nutzung aus ALKIS. Die Karten nutzen UTM-Zone 32.
- Sachsen-Anhalt (LVermGeo): Orthophoto, Gelände- und Oberflächenmodell (DGM1/DOM1), nDOM, Gebäude und Nutzung. Die Karten nutzen UTM-Zone 32.
- Hessen (HVBG): Orthophoto, Gelände- und Oberflächenmodell aus den Gemeinde-Downloads, Gebäude und Nutzung aus ALKIS.
- Schleswig-Holstein (LVermGeo SH): Orthophoto, Geländemodell aus dem INSPIRE-ATOM-Dienst (mit XYZ-Ersatz), Oberflächenmodell, Gebäude und Nutzung aus ALKIS.
- Mecklenburg-Vorpommern (LAiV M-V): Orthophoto aus dem WCS, Gelände- und Oberflächenmodell, Gebäude und Nutzung aus ALKIS.
- Nordrhein-Westfalen (Geobasis NRW): Orthophoto aus dem WMS, Gelände- und Oberflächenmodell (DGM1/DOM1) aus Laserkacheln, Gebäude und Nutzung aus ALKIS, OSM aus den Auszügen der fünf Regierungsbezirke statt des Landesauszugs. Die Karten nutzen UTM-Zone 32.
- Karten können UTM-Zone 32 oder 33 nutzen. Ein Projekt übernimmt die Zone seines Bundeslands. Wechselt ein bestehendes Projekt die Zone, meldet freemapper das und räumt die alten Dateien auf.
- Das Repository enthält pro unterstütztem Bundesland ein Beispielprojekt mit OpenOrienteering-Mapper-Datei, OCAD-Datei, Trainings-PDF und Vorschaubild.
- Die Release-Archive enthalten das Handbuch mit Bildern, am Pfad, auf den die READMEs verlinken.
- Das README erklärt Handbuch, Beispielprojekte und die Gebietsauswahl im Browser.
- Dieses Änderungsprotokoll und Architekturentscheidungen (ADRs) in `docs/adr/`.

### Geändert

- Jedes Bundesland hat eine Zeile in der Regionentabelle des README. Die Einzelheiten stehen im Handbuch.
- Die Größenschätzung vor dem Download zählt bei Ländern mit Kacheln ganze Kacheln und markiert die, die schon im Cache liegen.
- Ältere Projekte verwerfen ihre gespeicherten Objekte, wenn sich der Rasterdaten-Anbieter ändert, und speichern diese Änderung.
- freemapper legt die Rust-Version in `rust-toolchain.toml` fest, damit lokale Builds und CI dieselbe Version nutzen.

### Behoben

- Hamburg: ALKIS fällt auf den klassischen WFS zurück, wenn die OGC-API ausfällt.
- Die Landesgrenzen sind jetzt gültige Geometrien. Die Prüfung, ob ein Gebiet im Bundesland liegt, stolpert nicht mehr über Splitter oder Selbstüberschneidungen.
- Fehlerberichte der Server erkennt freemapper als solche und wiederholt die Anfrage nicht, auch bei XML-Fehlern von WMS- und WCS-Diensten.
- Downloads haben eine Größengrenze, folgen beim Downloadportal von Schleswig-Holstein keinen Weiterleitungen und hinterlassen nach einem abgebrochenen Lauf keine halben Dateien.
- Beschädigte oder ungewöhnliche LZW-komprimierte TIFF-Dateien liest freemapper so wie libtiff.
- Schlägt der Download des Oberflächenmodells fehl, bleibt ein gespeichertes Modell erhalten, das das Gebiet abdeckt. Ein unlesbares Befliegungsdatum bricht den Lauf nicht mehr ab.
- Fehlende Kacheln und Orte, die sich nicht auflösen lassen, erscheinen als Warnung mit Produkt und Kachel statt den Lauf zu beenden.
- Ein Orthophoto ohne Befliegungsdatum wird nur 30 Tage wiederverwendet und danach neu geladen. Scheitert die Abfrage des Datums, behält freemapper ein passendes gespeichertes Orthophoto samt Datum.

## [0.2.0] - 2026-11-07

### Hinzugefügt

- Berlin (Geoportal Berlin): ALKIS-Gebäude und -Nutzung in Projekten, mit passendem Quellenhinweis. Wo es kein fertiges nDOM gibt, berechnet freemapper es aus bDOM minus DGM1.
- Sachsen (GeoSN): Orthophoto, DGM1/DOM1-Kacheln, nDOM, Gebäude und Nutzung aus ALKIS und ein OSM-Auszug für Sachsen. Jede Region nennt ihren eigenen Datenanbieter im Kartenhinweis.
- Projekte wählen die Datenregion nach der Mitte des Gebiets. Unterstützt sind Brandenburg (ohne Berlin), Berlin und Sachsen.
- Gebietsauswahl und Downloadschätzung kennen die Regionen und zeigen die passenden Ebenen und Größen je Region.
- Ein zweisprachiges Handbuch (Deutsch und Englisch) mit Bildern. Es behandelt Installation, KI-Assistent, Gebietsauswahl, Vorlagen, Geländeprüfung, PDF, OCAD und Regionen und erklärt die Meldungen. Beide READMEs verlinken darauf.
- Das README sagt gleich am Anfang, dass freemapper ein OCAD-Startprojekt und ein Trainings-PDF schreibt.

### Geändert

- Das README führt die Regionen in einer gleichrangigen Tabelle statt in getrennten Abschnitten für Berlin und Sachsen.
- Die OSM-Fortschrittsmeldungen nennen den Auszug, den sie laden (Brandenburg oder Sachsen).

### Behoben

- Leere Antworten des sächsischen WFS, Kacheln am Rand außerhalb der Rasterregion und abgelehnte Anfragen behandelt freemapper korrekt. Eine abgelehnte Anfrage wird nicht wiederholt.
- Die Quellenzeile für Berlin nennt das Geoportal Berlin, und die ALKIS-Zeile erscheint nur bei Berliner Objekten.

## [0.1.0] - 2026-11-07

### Hinzugefügt

- Erste Veröffentlichung. freemapper erstellt aus amtlichen Geodaten Brandenburgs und aus OpenStreetMap ein Startprojekt für OpenOrienteering Mapper, als Kommandozeilenwerkzeug (`init`, `fetch`, `derive`, `export`, `run`) und als MCP-Server für KI-Assistenten.
- Georeferenzierte Karte mit dem IOF-Symbolsatz für Sprintkarten 1:1000, Randinformationen (Nordlinien, Maßstabsleiste, Infoblock) und einem Hinweis, der freemapper und sein Repository nennt.
- Orthophoto, Schummerung und Hangneigung als Vorlagen hinter der Karte sowie Höhenlinien und das nDOM, die freemapper aus den Geländedaten ableitet.
- Gebäude aus ALKIS als Kartenobjekte.
- OSM-Daten als Kandidaten: Zuverlässige OSM-Wege werden Kartenobjekte, der Rest bleibt als Kandidat zur Prüfung. Geofabrik-Auszüge werden zwischengespeichert, und freemapper nutzt den Cache, wenn Geofabrik nicht erreichbar ist.
- Sprintkarten-Objekte: befestigte Flächen aus ALKIS-Verkehrsflächen und unbefestigte Wege aus OSM.
- Verbotene Flächen für Privatgrund, abgeleitet aus der ALKIS-Nutzung.
- OCAD-Export (`--ocad`, OCAD 12) mit Platzhalter-Symbolen.
- Trainingskarte als einseitiges PDF (`export --pdf`, `run --pdf`, MCP-Option `pdf`).
- Gebietsauswahl im Browser (`freemapper select`, MCP-Werkzeug `open_area_selector`) mit Ortssuche, Rechteck, Polygon, Kachelumrissen und Downloadschätzung.
- Ortssuche über Nominatim (`init --place`) und Start aus OSM (`init --osm`). Der MCP-Server listet außerdem die Ebenen und liefert PNG-Vorschauen.
- Release-Archive für Linux (x86_64 und ARM64), macOS (Apple Silicon und Intel) und Windows, jeweils mit SHA-256-Prüfsumme.

### Behoben

- Downloads haben Zeitgrenzen für Verbindung, Antwort und Inhalt, behalten bei ungültigem neuem Download die zwischengespeicherte Kopie und räumen alte Teildateien auf.
- Eine vorhandene Karte bleibt bei einem erneuten Lauf erhalten. freemapper aktualisiert ihre Vorlagen und sagt das.
- freemapper schützt bearbeitete Karten beim Export vor dem Überschreiben.
- Fehlermeldungen nennen Produkt und nächsten Schritt. Maßstab 0 und Koordinaten außerhalb des gültigen Bereichs lehnt freemapper ab.
- Fehler bei nicht abgedeckten Kacheln erscheinen in einem Bericht, und Läufe ohne Netz greifen auf zwischengespeicherte Daten zurück.
