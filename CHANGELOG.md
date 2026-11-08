# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- Speicherplatz: Vor jedem Laden prüft freemapper den freien Platz im Zwischenspeicher und im Projektordner, auf zwei Laufwerken jedes für sich, und lässt auf jedem 1 GB frei. Passt nicht alles, lässt es das Oberflächenmodell und die Aktualisierung eines älteren OSM-Auszugs aus und warnt mit dem nötigen und dem freien Platz. Ohne Platz für Kartendaten, Geländemodell oder Luftbild bricht es ab, bevor es etwas lädt. Läuft der Datenträger beim Schreiben voll, meldet freemapper das klar, löscht die halbe Datei und versucht es nicht erneut. Die Download-Schätzung (MCP-Werkzeug `estimate_download`, Gebietsauswahl) zeigt den freien Platz und markiert, was nicht passt.
- Berlin: weitere offene Daten des Geoportals Berlin. Überdachungen und Carports aus ALKIS werden Kartenobjekte (522), auf beiden Kartenarten. freemapper schneidet sie an Gebäuden ab und lässt sie weg, wenn sie fast ganz in einem Gebäude liegen. Als Kandidaten kommen Treppen (532.6 bis 532.9) mit der Laufrichtung aus dem Geländemodell, Mauern, Stützmauern, Zäune, Hecken, Durchfahrten, Denkmäler, auf Sprintkarten auch Brücken und Böschungskanten. Straßen- und Anlagenbäume aus dem Baumkataster werden je nach Krone 417 oder 418, OSM-Bäume in 2 m Umkreis entfallen. Die Karte von Berlin 1:5000 (K5) liegt als Vorlage `templates/k5.png` bei. Eine vorhandene Karte bekommt die Überdachungen erst mit `run --force`.

### Geändert

- Kleingärten (ALKIS-Nutzung „Sport-, Freizeit- und Erholungsfläche“ mit `bez` „Kleingarten“) sind in jedem Land Kandidaten für verbotenes Gebiet (520). Parks, Sportanlagen und Spielplätze bleiben offen.

## [0.4.0] - 2026-11-08

### Hinzugefügt

- Niedersachsen (LGLN): Orthophoto aus dem WMS, Gelände- und Oberflächenmodell (DGM1/DOM1) aus dem WCS in Stücken von 2 km, Flugdatum und Datenstand aus den STAC-Suchen, Gebäude und Nutzung aus ALKIS. Das nDOM ist DOM1 − DGM1; ist das DOM1 mehr als zwei Jahre älter als das Luftbild, warnt freemapper mit beiden Daten. Eine Meldung sagt, ob die Bäume beim Bildflug belaubt waren. Die Karten nutzen UTM-Zone 32. Bremen gehört noch nicht dazu.
- Rheinland-Pfalz (LVermGeo RP): Orthophoto aus dem WMS, als Flugdatum gilt das Erstellungsdatum des Blocks. Gelände- und Oberflächenmodell (DGM1/DOM1) kommen aus Laserkacheln von 1 km, deren Namen und Daten die Befliegungsübersicht `Laserbefliegung` nennt; für Kacheln ohne Flugjahr liest freemapper die Metalink-Datei des Landes. Hat der Server eine Kachel nicht, meldet freemapper sie als fehlend und fragt nach 30 Tagen wieder. Gebäude und Nutzung kommen aus ALKIS. Die Karten nutzen UTM-Zone 32. An der Grenze zu Hessen (Mainz, Wiesbaden) liest freemapper ALKIS und OSM beider Länder.
- Gewässer auf allen Karten: Flüsse, Kanäle, Seen, Teiche, Hafenbecken und Meer aus ALKIS werden unpassierbares Gewässer (301.1) mit Uferlinie (301.6), ohne Linien zwischen den Katasterflächen und am Kartenrand. Gewässer, die im Mittel schmaler als 2 m sind, kommen als Kandidaten. Aus OSM kommen Teiche, Schwimmbecken, Brunnenbecken, Bäche, Gräben, Brunnen, Quellen, auf Schulhofkarten Trinkbrunnen und auf Sprintkarten Brücken als Kandidaten. Höhenlinien enden am Ufer, und Brunnenpunkte im Wasser (meist Fontänen in einem Becken) fallen weg. In OCAD tragen Fläche und Uferlinie die Nummern 301.001 und 301.002. Eine vorhandene Karte bekommt die Gewässer erst mit `run --force`.

### Geändert

- Die Nordlinien liegen im Kartenteil „default part“ bei den übrigen Kartenobjekten, mit dem Tag `freemapper:source=north-lines`. In OOM lassen sie sich an kleinen Objekten unterbrechen, und in der Trainingskarte liegen Mauern und Zäune darüber. Titel, Maßstab und Namensnennung bleiben im Teil „peripheral information“. Eine vorhandene Karte ändert sich erst mit `run --force`.
- Die Farben folgen den IOF-Druck- und Farbdefinitionen Rev. 4 (September 2024, gültig ab 1.12.2024) statt der älteren Werte aus dem OOM-Symbolsatz: Braun ist 25/75/100/0 statt 0/56/100/18, Grün 80/0/100/0 statt 76/0/91/0, Braun 50 % 10/35/50/0. Gebäude sind wie in ISSprOM 2019-2 mit 50 % Schwarz gefüllt statt 60 %. Das gilt für OOM-Karte, OCAD-Datei und Trainings-PDF. Namen und Reihenfolge der Farben bleiben gleich.
- Längere Meldungen sind klarer formuliert.
- Reicht ein Kartengebiet über alle unterstützten Länder hinaus (ins Ausland, aufs Meer oder in ein Land, das freemapper noch nicht unterstützt, etwa Bremen oder Thüringen), warnt freemapper beim Laden, dass dort Luftbild und ALKIS fehlen können. Das gilt für alle Länder.

### Behoben

- Wo die Gewässerflächen zweier Länder an der Grenze mitten im Fluss nicht ganz aneinanderstoßen (Mainz, Rhein), lief eine Uferlinie durch den Fluss. freemapper schließt jetzt Lücken unter 0,5 m zwischen Gewässerflächen, bevor es die Uferlinien zeichnet. Das gilt auch für Flurstücke desselben Landes. Brücken und Straßen (ab 0,8 m) bleiben Land, Inseln behalten ihr Ufer.

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
