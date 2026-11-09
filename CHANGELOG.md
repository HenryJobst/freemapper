# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Geändert

- Dichte Baumgruppen werden jetzt auch im Park zur Fläche, die man auf der Karte sieht. Drei oder mehr Bäume, die auf dem Papier näher als 2,5 mm (10 m bei 1:4000) beieinander stehen oder deren Kronen sich fast berühren, werden Wald (405), wenn ihre Kronen mindestens die Hälfte der Fläche decken, sonst offenes Gelände mit verstreuten Bäumen (402). Auf weißem Grund bekommt der Wald eine punktierte Vegetationsgrenze (416), damit er sichtbar ist. Auf Pflaster bleibt es bei 501.3, ab 9 Bäumen. Baumreihen an Straßen und Alleen bleiben Punkte. Im Park Sanssouci sinkt die Zahl der Baumpunkte von 1 654 auf 880 (51 Flächen 402), im Viktoriapark von 1 660 auf 949 (ADR 0066, Nachtrag).
- Die Trainingskarte (PDF) zeichnet die Punktmuster von Flächen, etwa die weißen Punkte von 402. Bisher sah 402 dort aus wie 401.

- freemapper schreibt, kopiert und löscht im Projektordner jetzt mit der gemeinsamen Bibliothek `freemapper-fs` aus freemapper-core 0.2.0 und pflegt keine eigene Kopie mehr. Für Nutzer bleibt alles gleich, auch der Wortlaut der Meldungen (ADR 0067).

## [0.7.0] - 2026-11-09

### Hinzugefügt

- Überdachungen und Durchfahrten in weiteren Ländern. In jedem Land außer Sachsen und Sachsen-Anhalt werden Überdachungen und Carports aus ALKIS Kartenobjekte (522) auf beiden Kartenarten, an den Gebäuden abgeschnitten wie in Berlin. Durchfahrten und Arkaden werden Kandidaten 522, das Gebäude bleibt ganz. freemapper liest sie aus den Gebäuden, die es ohnehin lädt, ein zusätzlicher Download entfällt. Datenstand ist das Datum des ALKIS-Objekts. Eine vorhandene Karte bekommt die Überdachungen erst mit `run --force` (ADR 0061).
- Brandenburg: das vollständige ALKIS aus dem NAS-Dienst der LGB. Mauern und Mauerkanten, Zäune, Hecken, Denkmäler, Brunnen, Gräben und Bäche werden Kandidaten, auf Sprintkarten auch Böschungsoberkanten und Brücken. Das Symbol wählt der Code im Kataster, `freemapper:alkis` nennt seine Bezeichnung, z. B. `Mauerkante, rechts`. Datenstand ist das Datum, seit dem die Objektversion im Kataster steht. Liegen die zwei Kanten einer Mauer einander zugewandt und auf dem Papier näher als 0,55 mm (2,2 m bei 1:4000), zeichnet freemapper eine Linie 515 in der Mitte mit `freemapper:alkis=Mauer (Mauerkanten gepaart)`, und `freemapper:also` nennt die andere Kante. Im Park Sanssouci sind es 264 statt 271 Mauerlinien. Die Ebenen liegen in `data/alkis-nas/`, etwa 1 MB je km² (ADRs 0062, 0063).
- Potsdam: Die Bäume der Stadtkarte werden Kandidaten 418, jeder mit seinem eigenen Aufnahmedatum. Die Daten kennen keine Krone, Baumstümpfe fallen weg. OSM-Bäume bis 2 m neben einem Potsdamer Baum fallen weg. Eine Antwort über 256 MB bricht freemapper beim Lesen ab. Die neue Namensnennung `Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0` erscheint, wenn Potsdamer Bäume unter den Kandidaten sind (ADR 0065).
- Punktgeneralisierung. Punktsymbole halten den Abstand von 0,15 mm ein, den die ISSprOM für Linien setzt. Drei oder mehr Bäume, die einander zu nahe stehen, werden eine Fläche, wenn mindestens die Hälfte ihrer Kronen auf Pflaster oder auf offenem Gelände (401 bis 404) liegt: auf Pflaster befestigte Fläche mit verstreuten Bäumen (501.3), auf offenem Gelände Wald (405), wenn die Kronen mindestens die Hälfte der Gruppe decken, sonst offenes Gelände mit verstreuten Bäumen (402). Anderswo bleiben die Bäume Punkte und werden ausgedünnt, denn ein weißer Wald auf weißem Grund würde sie verstecken. Baumreihen bleiben einzelne Baumsymbole. Die Fläche trägt `freemapper:source=generalization` und nennt ihre Bäume in `freemapper:source_id` (ADR 0066).

### Geändert

- Zu dicht stehende Punkte werden ausgedünnt. Kämen sich zwei Punktsymbole auf dem Papier näher, als die Norm erlaubt, behält freemapper das größere, dann das aus der besseren Quelle, dann das mit der größeren Krone. Der bleibende Punkt nennt die anderen in `freemapper:generalized`. Das ändert Kandidaten und Trainingskarte in jedem Land, auch in Berlin. Die Kartenobjekte bleiben gleich (ADR 0066).
- Bäume und andere Punkte in Sperrgebiet-Kandidaten (520) bleiben in den Kandidaten mit `freemapper:hidden=520`, die Trainingskarte lässt sie aber weg. Auf Schulhofkarten zeigt das PDF deshalb keinen Punkt außerhalb des Schulgeländes (ADR 0066).
- Doppel: Register vor ALKIS vor Straßenbefahrung vor OSM gilt jetzt in jedem Land und auch für Bäche und Gräben (305, 306). Eine OSM-Linie, die zu mindestens 80 % ihrer Länge höchstens 1,5 m neben einer Linie des vollständigen ALKIS mit demselben Symbol liegt, fällt weg, und `freemapper:also` nennt sie (ADR 0063).
- Größenschätzung und Platzprüfung rechnen das vollständige ALKIS und die Stadtbaumkataster als optionalen Teil (2 und 1 MB je km²). Fehlt der Platz, lässt freemapper sie weg und behält die gespeicherten.
- `run` meldet eine behaltene Karte ohne die ALKIS-Überdachungen jetzt in jedem Land mit Überdachungen. Die Meldung lautet nun `The map lacks the ALKIS canopies and carports (522) …`.
- Teilstücke einer Linie mit Seite (Böschung 104, Felswand 201, Stützmauer 513.2, die rechten und linken Kanten im vollständigen ALKIS) verbindet freemapper nur noch in ihrer eigenen Richtung und nie über zwei Seiten hinweg, damit die Seite der Striche stimmt. Im Viktoriapark bleiben dadurch zwei Böschungslinien mehr getrennt (ADR 0063).

### Sicherheit

- Das Aufräumen alter `.part`-Dateien im Ordner `templates` folgt keinem Link mehr aus dem Projekt hinaus und löscht keine Dateien außerhalb. Ein `.part`-Eintrag, der selbst ein Link ist, wird übersprungen (ADR 0060).

## [0.6.2] - 2026-11-09

### Sicherheit

- Ein Projektordner von jemand anderem konnte freemapper über Links außerhalb des Ordners schreiben lassen. War `ATTRIBUTION.txt` ein Link, überschrieb jeder `run` mit vorhandener Karte die Datei, auf die er zeigte. Mit `--force` kopierten die Sicherungen `.omap.bak` und `.ocd.bak` die Karte des Projekts in das Ziel des Links, etwa `~/.zshrc`. Auch die Bildkopien nach `ocad/` folgten Links, ebenso jede Datei in einem Ordner `data`, `templates` oder `ocad`, der selbst ein Link war. freemapper schreibt jede Datei jetzt unter einem neuen Namen und benennt sie dann um. Dabei ersetzt es einen Link, statt hindurchzuschreiben. Führt ein Ordner `data`, `templates` oder `ocad` aus dem Projekt hinaus, bricht der Befehl ab (`…: a link leads out of the project folder, so freemapper writes nothing there`). Teildateien bekommen jedes Mal einen neuen Namen und übernehmen nie eine vorhandene Datei oder einen Link (ADR 0060).
- Kopien innerhalb eines Projekts konnten durch Links hindurch lesen. Ein Vorlagenbild, das ein Link auf eine Datei an anderer Stelle war, etwa `~/.ssh/id_rsa`, landete als Kopie in `ocad/`. freemapper kopiert jetzt nur noch Dateien des Projekts selbst. Der OCAD-Export lässt eine solche Vorlage weg und warnt (`warning: …: a link or outside the project folder, so freemapper does not copy it; …`). freemapper prüft jeden einzelnen Schreibvorgang, nicht mehr nur einmal je Lauf (ADR 0060).
- Ein Wettlauf bleibt: Wer in einen Projektordner schreiben kann, während freemapper darauf läuft, und im richtigen Moment einen Ordner gegen einen Link tauscht, kann einen Schreibvorgang mit festem Dateinamen noch umlenken. freemapper also nicht auf Projektordnern laufen lassen, in die andere gleichzeitig schreiben können (ADR 0060).

### Geändert

- Bibliotheks-API (Crate `freemapper`): Die Funktionen, die Vorlagen und gespeicherte Daten schreiben, bekommen den Projektordner als zusätzliches Argument `root` vor dem Ausgabepfad. Das betrifft `write_gray_png`, `write_ndsm_png`, `build_orthophoto`, `build_orthophoto_from_image`, `wms::fetch_orthophoto`, `wms::fetch_orthophoto_from`, `wms::fetch_k5`, `geosn::fetch_orthophoto` und die `fetch_model`-Funktionen der Höhenmodell-Quellen. `OcadOutcome` hat das neue Feld `warnings`. Kommandozeile und MCP bleiben gleich.

### Behoben

- Windows: Die Prüfung des freien Speicherplatzes vor `fetch` erkennt Laufwerke jetzt an ihrer Seriennummer statt am Laufwerksbuchstaben. Bisher übersprang freemapper die Prüfung mit einer Warnung (`warning: cannot read the free disk space of …`), wenn Windows den Pfad nicht auflösen konnte, etwa auf manchen RAM- oder virtuellen Laufwerken. Ein Laufwerk, das in einen Ordner eines anderen Laufwerks eingebunden ist, zählte als dieses andere Laufwerk, sodass sein eigener Platz nicht geprüft wurde.

## [0.6.1] - 2026-11-09

### Sicherheit

- Eine präparierte `freemapper.toml` konnte beim Laden Dateien außerhalb des Projekts löschen. Ein Eintrag unter `berlin_layers` mit einem Namen wie `../../../x` ließ `fetch` die Datei `x.geojson` neben dem Projektordner löschen. freemapper bildet die Pfade der Berliner Ebenen jetzt nur noch aus seiner eigenen Ebenenliste. Einträge mit unbekanntem Namen oder mit `/`, `\`, `..`, Laufwerksbuchstaben oder Steuerzeichen lässt es beim Laden weg und warnt (`warning: freemapper.toml: ignored the Berlin layer entry …`). Führt der Ordner `data/berlin` über einen Link aus dem Projekt hinaus, schreibt und löscht freemapper dort nichts (ADR 0059).

## [0.6.0] - 2026-11-09

### Hinzugefügt

- Berlin: die Straßenbefahrung des Geoportals Berlin, aufgenommen 2014 bis 2016. Auf Sprintkarten werden Bordsteine (501.1), Werbesäulen (531), Grünstreifen (401) und vermessene Gehwege, Plätze und Parkwege außerhalb der ALKIS-Straßenflächen (501.5) Kandidaten. Auf beiden Kartenarten kommen Treppen, Mauern, Geländer, Verkehrsschutzgitter und Leitplanken (518), Fahrgastunterstände und Brunnen dazu. Jeder vermessene Kandidat trägt sein Aufnahmedatum; ist es nicht lesbar, steht 2014-01-01 mit `freemapper:currency_guessed=yes` da.
- Berlin: Möblierung auf Schulhofkarten. Bänke, Hydranten, Poller, Masten, Fahnenmasten und Spielgeräte kommen aus der Straßenbefahrung, Laternen (nur die in Betrieb) aus dem Beleuchtungskataster, Zier- und Trinkbrunnen aus ihren Registern. Auf Sprintkarten bleibt die Möblierung weg.
- Berlin: Ein Punkt aus einem Register ersetzt OSM- und vermessene Punkte derselben Art: eine Laterne OSM-Laternen bis 5 m und vermessene Masten bis 2 m Abstand, ein Brunnen Brunnen bis 3 m. Auf Schulhofkarten ersetzt ein vermessener Hydrant, eine Bank, ein Mast oder ein Spielgerät das OSM-Gegenstück bis 2 m. Liegt ein Brunnen aus einem Register oder der Straßenbefahrung in einem gezeichneten Gewässer, fällt er weg.
- Berlin: Gebäude tragen als Datenstand ihre Grundrissaktualität aus den Gebäudehöhen 2022 des Umweltatlas. Ein Gebäude ohne Treffer bleibt ohne Datum.
- Neue Zeilen in der Namensnennung für die Straßenbefahrung, die Register und die Gebäudehöhen, jeweils nur, wenn sie Objekte geliefert haben. Die Registerzeile nennt nur die Register, deren Punkte in Karte oder Kandidaten stehen.

### Geändert

- Berlin: Die befestigten Flächen (501.5) enthalten die vermessenen Grünstreifen nicht mehr, freemapper schneidet sie aus. Baumscheiben bleiben darin. Ein Grünstreifen wird nur dort Kandidat 401, wo freemapper ihn ausgeschnitten hat oder wo er neben den befestigten Flächen liegt. Eine beschnittene Fläche nennt die Grünstreifen in `freemapper:cut`, `freemapper:cut_ids` und `freemapper:cut_currency`. Eine vorhandene Karte ändert sich erst mit `run --force`.
- Berlin: Kurze Linienstücke aus ALKIS und der Straßenbefahrung, die Ende an Ende aneinanderstoßen, fügt freemapper zu einer Linie zusammen. Sie nennt die übrigen Stücke in `freemapper:also` und trägt das älteste Aufnahmedatum. Danach fallen Linien unter 1 mm auf dem Papier weg, jetzt auch bei ALKIS.
- Berlin: Gibt es ein Objekt mehrfach, gewinnt das Register vor ALKIS, ALKIS vor der Straßenbefahrung und diese vor OSM. Ein vermessener Fahrgastunterstand, der mindestens zur Hälfte unter einer ALKIS-Überdachung liegt, fällt weg. Das bleibende Objekt nennt die anderen in `freemapper:also`, vermessene als `geoportal-berlin/<Ebene>/<gis_id>`. Jeder Berliner Kandidat trägt den Tag `berlin:layer`.
- Die Download-Schätzung rechnet für die Berliner Ebenen mit 10,5 statt 4 MB je km² (gemessen am Viktoriapark: 9,8).

## [0.5.0] - 2026-11-09

### Hinzugefügt

- Speicherplatz: Vor jedem Laden prüft freemapper den freien Platz im Zwischenspeicher und im Projektordner, auf zwei Laufwerken jedes für sich, und lässt auf jedem 1 GB frei. Passt nicht alles, lässt es das Oberflächenmodell, die Aktualisierung eines älteren OSM-Auszugs und in Berlin die Bauwerke, Bäume und die K5 aus und warnt mit dem nötigen und dem freien Platz. Ohne Platz für Kartendaten, Geländemodell oder Luftbild bricht es ab, bevor es etwas lädt. Läuft der Datenträger beim Schreiben voll, meldet freemapper das klar, löscht die halbe Datei und versucht es nicht erneut. Die Download-Schätzung (MCP-Werkzeug `estimate_download`, Gebietsauswahl) zeigt den freien Platz und markiert, was nicht passt.
- Berlin: weitere offene Daten des Geoportals Berlin. Überdachungen und Carports aus ALKIS werden Kartenobjekte (522), auf beiden Kartenarten. freemapper schneidet sie an Gebäuden ab und lässt sie weg, wenn sie fast ganz in einem Gebäude liegen. Als Kandidaten kommen Treppen (532.6 bis 532.9) mit der Laufrichtung aus dem Geländemodell. Steigt eine Treppe darin nicht, führt sie auf ein angrenzendes Gebäude oder Denkmal zu oder läuft über ihre kurze Seite. Treppen breiter als dreimal 532.9 werden parallele Linien. Die `Freitreppe`-Linien sind Stufenkanten, aus jeder Folge paralleler Kanten wird eine Treppe quer dazu. Weitere Kandidaten sind Mauern, Stützmauern, Zäune, Hecken, Durchfahrten, Denkmäler, auf Sprintkarten auch Brücken und Böschungskanten. Straßen- und Anlagenbäume aus dem Baumkataster werden je nach Krone 417 oder 418, OSM-Bäume in 2 m Umkreis entfallen. Ebenso entfallen OSM-Zäune, -Mauern und -Hecken, die zu 80 % ihrer Länge höchstens 1,5 m neben derselben ALKIS-Linie liegen. Die Karte von Berlin 1:5000 (K5) liegt als Vorlage `templates/k5.png` bei. Eine vorhandene Karte bekommt die Überdachungen erst mit `run --force`, und `run` weist darauf hin.
- `init --osm` nimmt mit `--part <n>` einen Teil einer OSM-Fläche aus mehreren getrennten Teilen (1 ist der größte; im MCP `part`). Ohne `--part` nennt die Meldung die Teile mit ihrer Größe. Findet Nominatim einen Weg nicht, etwa einen äußeren Weg ohne Tags, schlägt die Meldung die Relation oder `--geojson` vor.

### Geändert

- Kleingärten (ALKIS-Nutzung „Sport-, Freizeit- und Erholungsfläche“ mit `bez` „Kleingarten“) sind in jedem Land Kandidaten für verbotenes Gebiet (520). Parks, Sportanlagen und Spielplätze bleiben offen.

### Behoben

- Die Höhenlinien-Kandidaten nennen in `freemapper:source` das Geländemodell ihres Landes, z. B. `laiv-mv-dgm1` oder `hvbg-dgm1`. Bisher stand in jedem Land `lgb-dgm1` da.

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
