# Änderungen

[English version](CHANGELOG.en.md) · [Handbuch](docs/manual/de.md)

Hier stehen alle wichtigen Änderungen an freemapper, die neueste zuerst. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unveröffentlicht]

### Hinzugefügt

- Programmsprache, dritte Stufe (ADR 0092): Fortschrittszeilen und Warnungen kommen in der Sprache des Programms, auf der Kommandozeile, im MCP-Server und im Protokoll der Gebietsseite. Eine Warnung beginnt deutsch mit `Warnung:`. Steckt in einer Warnung ein Fehler, etwa ein fehlgeschlagener Download, ist auch er übersetzt. Noch Englisch sind `--help` und rund 25 Warnungen zu den Höhenmodellen einzelner Bundesländer (Kachellisten, Datumsangaben).

### Geändert

- Bibliothek: Die Rückruffunktion für den Fortschritt bekommt statt einer fertigen Zeile (`&str`) einen Wert `Note` mit Kennung und Angaben; `to_string()` gibt die englische Zeile wie bisher, `Note::text` die Zeile in einer Sprache, `Note::is_warning` sagt, ob es eine Warnung ist. Die Texte liegen in `crates/freemapper/i18n/`.

## [0.10.0] - 2026-11-11

### Hinzugefügt

- Gebietsauswahl: Die Seite erzeugt jetzt die ganze Karte (ADR 0090, Stufe 5; ADR 0009, Nachtrag). Unter den Einstellungen steht der Block „Karte erzeugen“ mit den Optionen OCAD-Startprojekt, Druckfertige PDF, Trainings-PDF, Maßstabsleiste, Quellenangabe (automatisch, vollständig, nur freemapper, mit Hinweis auf die Lizenzen ODbL und dl-de/by-2-0), „Nur Zwischenspeicher“ und, nur bei vorhandener Karte, „Vorhandene Karte ersetzen“ (nie vorgewählt, mit Bestätigungsdialog). Die Seite speichert bei Bedarf zuerst, zeigt „Schritt i von n“, den Protokollauszug und die Warnungen, lässt sich abbrechen, listet am Ende die Dateien mit Größe und öffnet den Projektordner im Dateimanager. Ein Neuladen während des Laufs setzt die Anzeige fort. Während des Laufs sind Gebiet, Einstellungen und „Speichern“ gesperrt. Die Voreinstellungen kommen von `freemapper select --ocad --pdf --pdf-candidates --pdf-scale-bar --pdf-sources --offline`. Alle Texte gibt es auf Deutsch und Englisch nach der Sprache des Browsers; ein Feld `lang` der Seite hätte Vorrang, ist aber noch leer. Der Maßstab einer Kartenart kommt jetzt vom Server (`allowed_scales`), nicht mehr aus einer Kopie in der Seite. Neues Kapitel im Handbuch: „Karte im Browser erzeugen“.
- Gebietsauswahl: Der lokale Server hat Routen für den Lauf (ADR 0090, Stufe 4), die erst Skripte nutzen können, die Seite folgt in Stufe 5. `POST api/run` startet den Lauf mit den sieben Optionen (`force`, `offline`, `ocad`, `pdf`, `pdf_candidates`, `pdf_scale_bar`, `pdf_sources`) und antwortet 202 oder 409, wenn schon ein Lauf oder ein anderer Prozess den Ordner belegt; `GET api/status?since=N` meldet Schritt, neue Zeilen, Warnungen und Dateien, `POST api/cancel` bricht ab, `POST api/open-folder` öffnet den Projektordner im Dateimanager. Art, Maßstab und Titel bleiben wie gespeichert. `freemapper select` und das MCP-Werkzeug `open_area_selector` nehmen `--offline --ocad --pdf --pdf-candidates --pdf-scale-bar --pdf-sources` als Voreinstellung der Seite.
- Ein Projektordner, ein Lauf (ADR 0090, Stufe 3; schließt die Lücke aus ADR 0009). `run`, `fetch`, `derive`, `export`, die MCP-Pipeline und das Speichern des Gebiets auf der Seite legen im Ordner die Datei `.freemapper-run.lock` an. Ein zweiter Prozess im selben Ordner hört mit einer klaren Meldung auf, etwa „Ein anderer freemapper-Lauf verwendet diesen Ordner (seit 14:02, gestartet vom Terminal)“. Der Halter meldet sich alle zehn Sekunden. Nach 60 Sekunden ohne Meldung gilt die Sperre als verwaist, etwa nach einem Absturz, und wird ersetzt; Strg+C räumt sie sofort auf. Die Datei enthält nur Prozessnummer, Startzeit und den Starter. Dazu kann die Engine zwischen Schritten, Kacheln und Anfragen anhalten (`ProjectError::Cancelled`, `run_project_cancellable`). Ein abgebrochener Lauf lässt nur ganze Dateien zurück, und der nächste Lauf setzt dort fort. Die Abbruchtaste auf der Seite folgt in Stufe 4.
- Bahngleise aus OSM (ADR 0093). Auf Sprintkarten stehen `railway=rail`, `light_rail` und `narrow_gauge` jetzt als Kartenobjekte 509.1 (Bahn, mindestens 4 mm lang), Brücken und Nebengleise eingeschlossen. Vorher las freemapper kein Gleis: Auf der Karte vom Biesdorfer Park fehlte die S-Bahn. Tunnel, Durchgänge unter Gebäuden, stillgelegte, abgebaute und geplante Gleise und Überleitungen entfallen. U-Bahn-Gleise ohne Tunnel sind Kandidaten 509.1, auf einer Hochbahnbrücke Kartenobjekte. Straßenbahngleise sind Kandidaten 509.2, denn die Norm zeichnet sie nur, wenn sie der Orientierung dienen. Ein Wald wird an einem Gleis geschnitten. Das Bahngelände aus ALKIS bleibt ein Kandidat 520. Schulhofkarten haben kein Gleis. Bei den Beispielen ändern sich drei: Viktoriapark (3 Gleise), Herrenhausen (2) und Cochem (4); Herrenhausen und Cochem bekommen dabei auch die festen Kieswege der vorigen Korrektur. Die übrigen acht haben kein Gleis im Gebiet; Quedlinburg ließ sich nicht neu bauen (der Luftbild-Dienst von Sachsen-Anhalt antwortete mit einem Fehler). Bestehende Karten bekommen die Gleise mit `run --force`.
- Programmsprache, erste Stufe (ADR 0092): `--lang de` oder `--lang en` wählt die Sprache der Programmtexte, sonst gilt `FREEMAPPER_LANG`, dann die Sprache des Systems (`LC_ALL`, `LC_MESSAGES`, `LANG`), sonst Englisch. Deutsch sind bisher die Zeilen, die das Programm selbst am Ende eines Schritts ausgibt („… geschrieben“, „… behalten“, die Hinweise nach `run`, `export` und `select`). Fortschrittszeilen, Warnungen, Fehlermeldungen und `--help` bleiben vorerst Englisch.
- Programmsprache, zweite Stufe (ADR 0092): Fehlermeldungen kommen in der Sprache des Programms, im Terminal, vom MCP-Server und auf der Gebietsseite. Mit `--lang de` heißt es zum Beispiel „Fehler: Es sind noch keine Kacheln geladen; rufen Sie zuerst `freemapper fetch` auf“. Englisch bleibt, was ein Server, das Betriebssystem oder ein Parser zu einem Fehler sagt, und einige Einzelheiten, die freemapper noch als freien Text führt (etwa bei der Ortssuche). Fortschrittszeilen, Warnungen und `--help` folgen in den nächsten Stufen.
- Gebietsauswahl: Die Seite nimmt die Sprache des Programms (ADR 0092), wenn `--lang`, `FREEMAPPER_LANG` oder das System (`LC_ALL`, `LC_MESSAGES`, `LANG`) eine nennt. `freemapper select --lang en` zeigt die Seite also auch in einem deutschen Browser auf Englisch. Nennt nichts eine Sprache, richtet sich die Seite wie bisher nach dem Browser.

### Geändert

- Ein Fehler im Terminal steht jetzt in einer Zeile, `error: …` oder `Fehler: …`, die Ursache folgt nach einem Doppelpunkt. Vorher stand dort `Error: …` und die Ursache darunter unter „Caused by:“. Der Exit-Code bleibt 1.
- `export` und `run` bauen Karte und Kandidatenvorlage nur noch einmal je Befehl statt einmal je Ausgabe (ADR 0091). Mit `--ocad --pdf --pdf-candidates` waren es bisher vier Durchläufe über alle gespeicherten Ebenen, jetzt ist es bei `export` einer und bei `run` zwei. Die geschriebenen Dateien sind Byte für Byte dieselben.
- Gebietsauswahl: Der lokale Server läuft jetzt auf der gemeinsamen Bibliothek `freemapper-web` aus freemapper-core v0.11.0 (ADR 0090, Stufe 1; ADR 0009 und 0067, Nachtrag). Die Seite sieht gleich aus. Die Prüfungen sind strenger: Ein POST braucht den `Origin` der Seite und `Content-Type: application/json`, sonst antwortet der Server mit 403. Wer die Routen mit einem eigenen Skript aufruft, muss beides senden. Anfragen mit `Sec-Fetch-Site: cross-site` werden abgelehnt, jede Antwort trägt `X-Content-Type-Options: nosniff`, und ein Anfragekörper darf höchstens 1 MiB groß sein (ein Umriss mit 5000 Punkten braucht etwa 150 KB). Alle Kernbibliotheken sind jetzt auf dasselbe Tag v0.11.0 festgelegt.
- Die Vorschaubilder der Beispiele sind jetzt WebP statt JPEG (`preview.webp`, `preview-map.webp`). Sie entstehen direkt aus den PDFs und sind kleiner als die JPEGs. Das Titelbild der README (Viktoriapark) ist mit 1600 Pixeln Breite schärfer.
- Tests: Jeder Testlauf prüft jetzt, ob Karte und `freemapper.toml` den Vertrag der Suite einhalten (ADR 0089, `freemapper-contract` aus freemapper-core v0.8.0). Geprüft werden eine Schulhofkarte 1:1000 und eine Sprintkarte 1:4000 gegen zwölf Regeln und gegen das Vokabular, auf das sich freemapper-print und freemapper-courseplan verlassen: Namen der Kartenteile, Tags der Randangaben und der belaufbaren Fläche, Vertragsschlüssel der Projektdatei. Das Programm selbst ändert sich nicht.

### Behoben

- Feste Kieswege (501.6 bis 501.9) bleiben ganz, wo nur ein ALKIS-Flurstück `Weg` liegt (ADR 0079, Nachtrag). Dieses Flurstück ist nur ein Kandidat 501.5, kein Kartenobjekt, und schnitt trotzdem die OSM-Linie weg. Jetzt schneiden nur 501.5-Kartenobjekte (Straßenverkehr, Platz, Befahrung). Toleranz 0,25 mm und Mindeststück 1 mm bleiben. Bestehende Karten bekommen das mit `run --force`.

## [0.9.2] - 2026-11-10

### Behoben

- Gebietsauswahl: Die Hintergrundkarte zeigte bei manchen Browsern „access blocked“ (403) von OpenStreetMap (Issue 11). Die Seite schickt kein `Referer`, und die Kachelrichtlinie von OpenStreetMap verlangt ein gültiges `Referer` oder einen erkennbaren `User-Agent`. Jetzt holt der lokale Server die Kacheln selbst (`/tiles/osm/{z}/{x}/{y}.png`) mit `User-Agent: freemapper/<Version> (+https://github.com/HenryJobst/freemapper)`, ohne weitere Daten. Er prüft Zoom (0 bis 19), Spalte und Zeile, folgt keiner Weiterleitung, fragt höchstens zweimal gleichzeitig an und behält die Kacheln im Speicher so lange, wie OpenStreetMap erlaubt (höchstens 1024 Kacheln, 32 MB). Die Seite lädt nichts mehr direkt von `tile.openstreetmap.org`. Die Luftbilder der Bundesländer bleiben unverändert (ADR 0088).
- Breite Treppen wie die Tribünen am Stadion (ADR 0048, Nachtrag): Die parallelen Linien 532.9 stehen jetzt eine Symbolbreite plus 0,15 mm auseinander, dem Mindestabstand der Norm zwischen zwei Linien einer Farbe. Vorher stießen die Querstriche benachbarter Linien aneinander, und die Tribüne sah aus wie eine Leiter. Die Breite bleibt bis zum Rand bedeckt. Im Viktoriapark sinkt die Zahl der Linien 532.9 von 94 auf 86. Bestehende Karten bekommen das mit `run --force`.
- Berlin: Brunnen, Brunnenpumpen und Trinkbrunnen kommen nicht mehr doppelt vor (ADR 0087). Gemessen am Viktoriapark: Vermessene Pumpen und OSM-Brunnen derselben Stelle liegen 0,05 bis 0,5 m auseinander, der nächste Punkt einer anderen Quelle 9,8 m. Die Grenze ist 0,75 mm auf dem Papier (3 m bei 1:4000), im Gelände nie unter 3 m. Die Rangfolge ist Register, ALKIS, Straßenbefahrung, OSM. Die Straßenbefahrung schlägt jetzt OSM-Brunnen (`man_made=water_well`, `amenity=fountain`) und OSM-Trinkwasser (`amenity=drinking_water`), das Zierbrunnen-Register auch den OSM-Brunnen. Der Gewinner nennt den weggelassenen Punkt in `freemapper:also`. Ein Register-, ALKIS- oder Straßenbefahrungs-Brunnen im OSM-Brunnenbecken (`amenity=fountain` als Fläche) oder bis 1 m daneben fällt weg, und das Becken nennt ihn. Zwei Brunnen unterhalb der Grenze ohne höhere Quelle bleiben beide. Am Viktoriapark bleiben 9 Kandidaten 311 (3 Register, 4 Straßenbefahrung, 2 OSM) und 0 Kandidaten 313, nur der Vermerk wechselt von `freemapper:generalized` zu `freemapper:also`. Bestehende Karten bekommen das mit `run --force`.

## [0.9.1] - 2026-11-10

### Geändert

- Koordinatensysteme und Kacheln kommen jetzt aus `freemapper-georef` (freemapper-core v0.5.1). `crs` und `tiles` in freemapper geben die Typen unter den alten Pfaden weiter, das Verhalten ändert sich nicht. Die Projektion rechnet mit der gemeinsamen Transversal-Mercator-Implementierung. `freemapper-fs` steht jetzt ebenfalls auf v0.5.1.

### Hinzugefügt

- Freileitungen, Rohrleitungen, Türme und Masten aus OSM, Sprintkarten, nur als Kandidaten. Oberirdische `power=line|minor_line` und `power=cable` mit `location=overhead` sind 510, Leitungen ab 110 kV sind 511. Rohrleitungen mit `location=overground|surface|overhead` sind 528 (niedrig) oder 529 (hoch, nicht überquerbar: überirdisch auf Stützen oder `height` ab 1,5 m). `power=tower`, `man_made=communications_tower|chimney|water_tower|lighthouse`, Türme und Masten sind Punkte 524 oder 525, je nach Höhe. Unterirdische Leitungen und Rohrleitungen, `power=pole`, Masten unter 10 m und Tanks fehlen. Die Linien halten die Mindestlänge der Norm ein (3,75, 2,2 und 3 mm auf dem Papier). Jeder Turmpunkt trägt `freemapper:control=` und wird wie das Straßenmobiliar ausgedünnt. ALKIS: Die geladenen Ebenen enthalten keine Türme oder Leitungen, darum gibt es keine ALKIS-Regel (Folgearbeit im ADR). Kein Beispielgebiet hat eine Leitung oder Rohrleitung. Wismar, Monschau und Quedlinburg bekommen je einen Turm (Kandidaten 1756 auf 1757, 850 auf 851, 1110 auf 1111) (ADR 0083).

### Behoben

- Die Maßstabsleiste trägt wieder `freemapper:source=sheet` wie alle Blattobjekte (seit 0.9.0 stand dort `scale-bar`). freemapper-print und freemapper-courseplan erkennen das Blatt nur an diesem Wert; sonst druckten sie die Leiste als Karteninhalt und verschoben Seitenmitte und Ausdehnung. Zusätzlich trägt jedes Leistenobjekt `freemapper:sheet=scale-bar`; daran lässt das PDF die Leiste ohne `--pdf-scale-bar` weg. OOM- und OCAD-Karte behalten die Leiste. Bereits erzeugte Karten mit dem alten Wert bleiben lesbar (ADR 0080, Nachtrag).
- Doppelte Flächen nach Rang (ADR 0078): Eine Fläche, die selbst wegen einer höheren weggefallen ist, lässt keine weitere mehr wegfallen. Bei der Kette A > B > C fiel C sonst wegen des schon entfernten B weg, und der Hinweis `freemapper:also` landete auf B. Jetzt trägt ihn die Fläche, die auf der Karte bleibt, und C bleibt, wenn A allein den Anteil nicht erreicht. Bestehende Karten bekommen das mit `run --force`.
- Das Kartengebiet wird auf jedem Weg geprüft, nicht nur auf der Auswahlseite (ADR 0084). `init --geojson`, MCP `create_project` mit `geojson`, `init_project` und das Gebiet in `freemapper.toml` lehnen ein Polygon ohne Ecken, mit sich kreuzenden Kanten oder mit mehr als 5000 Ecken jetzt mit „not a usable map area“ ab. Ein leeres Polygon (`"coordinates": []`) brachte vorher den Server zum Absturz.
- `init`, MCP `create_project` und die Gebietsauswahl nehmen nur noch Maßstäbe an, die freemapper-print druckt (ADR 0085): Schulhofkarte 1:1000, Sprintkarte 1:4000 oder 1:5000. Vorher lief `--kind schoolyard --scale 2000` durch, und freemapper-print brach danach mit `MapScaleNotAllowed` ab. Die Fehlermeldung nennt die erlaubten Maßstäbe; bei Sprint 1:3000 rät sie, bei 1:4000 zu zeichnen, weil freemapper-print beim Drucken auf 1:3000 vergrößert. 1:5000 steht nicht in der Norm; `init` weist darauf hin, die Symbole bleiben bei 100 %. Ein Projekt mit anderem Maßstab in der `freemapper.toml` lässt sich weiter laden und gibt eine Warnung aus. Die Gebietsauswahl bietet die erlaubten Maßstäbe als Liste an.
- `init` schreibt immer einen `title` in die `freemapper.toml`, auch ohne `--title`: der Ordnername mit großen Anfangsbuchstaben, wie ihn das PDF schon bildete (`Clara-Zetkin-Schule`). freemapper-print und freemapper-courseplan lesen damit denselben Titel (ADR 0086). Ältere Projekte ohne `title` bleiben unverändert.

## [0.9.0] - 2026-11-10

### Geändert

- Alle elf Beispiele sind mit dem aktuellen Code neu erzeugt. Das PDF jedes Beispiels ist die Trainingskarte (`<name>-candidates.pdf`), und `preview.jpg` ist daraus gerendert, weil die Karte allein sehr schlicht ist. Kartenobjekte, Kandidaten und Symbole in der Beispielübersicht sind neu gezählt, und die Kieswege 501.6 bis 501.9 stehen in allen Beispielen als Kartenobjekte. Nur Cochem und Blankenese haben zusätzlich die Karte ohne Kandidaten (`<name>.pdf`, `preview-map.jpg`). Die Namensnennung des Viktoriaparks nennt wieder die Öffentliche Beleuchtung.
- Der Standard von `--pdf` ändert sich. Es schreibt jetzt `<name>.pdf` mit genau den Objekten der OOM-Karte, ohne Kandidaten und ohne Maßstabsleiste. Die Trainingskarte mit allen Kandidaten steht in `<name>-candidates.pdf` und kommt mit `--pdf-candidates`; beide Schalter zusammen schreiben beide Dateien. Die Maßstabsleiste auf den PDFs gibt es nur mit `--pdf-scale-bar` (deutsche OL-Karten haben meist keine); die Maßstabszahl bleibt im Titelblock. Das alte Verhalten ist `--pdf-candidates --pdf-scale-bar`. Gilt auch für `run` und die MCP-Parameter `pdf`, `pdf_candidates`, `pdf_scale_bar`. OOM- und OCAD-Karte behalten die Leiste. Ändert ADR 0002 (ADR 0080).
- Die Trainingskarte des Viktoriaparks (`berlin-viktoriapark-candidates.pdf`, `preview.jpg`) ist mit der gekürzten Quellenangabe neu erzeugt (vier statt neun Zeilen). Die übrigen Beispiele haben höchstens fünf Zeilen und bleiben unverändert; die Karteninhalte sind identisch.

### Hinzugefügt

- Die Quellenangabe auf der PDF-Karte wird bei Bedarf gekürzt. `--pdf-sources auto` (Standard, auch MCP `pdf_sources`) druckt bis zu fünf Zeilen unverändert; ist der Block länger, fasst freemapper die Quellen mit `dl-de/zero-2-0` je Anbieter zusammen, kürzt lange Namen, verbindet Zeilen mit „ · “ und verweist zuletzt auf `ATTRIBUTION.txt`. OpenStreetMap, alle anderen Lizenzen und die freemapper-Zeile bleiben wörtlich. `full` kürzt nie; `credit-only` druckt nur die freemapper-Zeile und weist in der Ausgabe darauf hin, dass ODbL und `dl-de/by-2-0` die Nennung beim Weitergeben verlangen. Die freemapper-Zeile lässt sich in keinem Modus abschalten. `ATTRIBUTION.txt`, OOM- und OCAD-Karte führen immer alle Quellen. Der Viktoriapark-Block schrumpft von neun auf vier Zeilen (ADR 0081).
- Eine Gegenüberstellung von Karte und Trainingskarte: [`examples/vergleich.md`](examples/vergleich.md) zeigt Cochem und Blankenese nebeneinander, mit den häufigsten Kandidatensymbolen und der Messung an allen elf Beispielen, aus der die Auswahl folgt. Die Beispielübersicht und die README verlinken sie.
- Berliner Kontrollstandorte auf Sprintkarten. Die Straßenbefahrung (Poller, Masten, Fahnenmasten, Bänke) und das Beleuchtungskataster (Laternen in Betrieb) liefern jetzt 531-Kandidaten mit `freemapper:control` (`bollard`, `mast`, `flagpole`, `bench`, `street_lamp`, dazu `advertising_column` für die Werbesäulen der Befahrung), wie die aus OSM. Die Ausdünnung gilt für alle Quellen gemeinsam: Ein Standort, der ein gleichartiges innerhalb von 25 m hat, fehlt, gleich aus welcher Quelle, nachdem Doppel und Standorte in Gebäuden entfallen sind. Ein vermessener Poller ersetzt einen OSM-Poller bis 2 m. Hydranten und Spielgeräte bleiben Schulhof-Sache. Ein Berliner Sprintprojekt lädt sieben WFS-Anfragen mehr (33 statt 26); die Platzschätzung für die Berliner Ebenen steigt von 10,5 auf 12 MB je km². Bestehende Karten bekommen das mit `run --force` (ADR 0077).
- Trinkbrunnen sind auf Sprintkarten 313, aus dem Berliner Register, der Befahrung und aus OSM `amenity=drinking_water`.

### Behoben

- Wege aus festem Kies sind wieder Kartenobjekte. `surface=compacted|fine_gravel` und `tracktype=grade2` (befestigt nach ISSprOM 501, „hard gravel“) zeichnet freemapper als Linie 501.6 bis 501.9 nach `width` in der Karte, nicht mehr nur als Kandidat; die Parkkarten haben ihre Wege zurück. Wo ALKIS oder die Befahrung schon Pflaster zeichnen (501.5), entfällt der Teil der Linie im Pflaster (aufgeweitet um 0,25 mm auf dem Papier), und Reststücke unter 1 mm auf dem Papier entfallen. Wege ohne `surface`, unbekannte Oberflächen und `tracktype=grade3` bis `grade5` bleiben Kandidaten, Baumgruppen werden weiter an den Linien geschnitten. Viktoriapark: 26 Kartenobjekte 501.6 statt 49 Kandidaten, Sanssouci: 192 statt 218. Ändert ADR 0073 (ADR 0079).
- Doppelte Flächen aus mehreren Quellen bleiben nur einmal. Ein OSM-Durchgang (`tunnel=building_passage`, `covered`) über einem Berliner Gebäudeteil, einem ALKIS-Durchgang oder einer Überdachung, ein privater OSM-Streifen in einer ALKIS-Fläche, die nicht betreten werden darf, und eine OSM-Brücke über einer Berliner Brückenfläche entfallen, wenn die Hälfte (520: 80 %) ihrer Fläche unter der höherrangigen Fläche liegt, aufgeweitet um 0,25 mm auf dem Papier. Das bleibende Objekt nennt das andere in `freemapper:also`. Auch ein OSM-Denkmal bis 5 m neben einem Katasterdenkmal sowie OSM-Bordsteine und -Böschungen entlang von Katasterlinien entfallen. Viktoriapark: 520 von 117 auf 94, 522 von 211 auf 192. Lübeck: 520 von 1 125 auf 1 112, 522 von 280 auf 221 (ADR 0078).
- Kontrollstandorte in Gebäuden dünnen ihre Nachbarn nicht mehr aus. Die 25-m-Regel läuft jetzt, nachdem die Standorte in Gebäuden entfallen sind.

## [0.8.0] - 2026-11-09

### Hinzugefügt

- Nur erreichbare Durchgänge werden gezeichnet. Ein 522-Durchgang (aus OSM, ALKIS oder den Berliner Bauteilen), der nur an Gebäude, 520-Flächen oder Wasser grenzt oder zwei umbaute Höfe ohne Ausgang verbindet, bekommt `freemapper:hidden=unreachable`: Er bleibt unter den Kandidaten und fehlt in der Trainingskarte. Als erreichbar gilt ein Durchgang, wenn er (oder eine Kette von Durchgängen, etwa die Lübecker Höfe und Gänge) auf belaufbaren Grund trifft, der mit dem Straßennetz zusammenhängt, mit 0,5 mm (2 m) Spielraum. Lübeck: 16 von 280 Durchgängen verborgen, Viktoriapark: 52 von 211 (ADR 0076).
- Denkmäler aus OSM. Denkmäler, Wegkreuze und Grenzsteine (`historic=memorial|monument|wayside_cross|boundary_stone`) sind auf Sprintkarten 526-Kandidaten. Stolpersteine, Gedenktafeln und Gedenkstücke, die eine Bank, ein Baum oder ein Wandbild sind, bleiben draußen, denn 526 gilt nur für Auffälliges (ADR 0075).
- Kontrollstandorte für Bahnleger. Poller, Straßenlaternen, Papierkörbe, Fahrradständer und Bänke aus OSM sind auf Sprintkarten 531-Kandidaten mit dem Tag `freemapper:control` (Art des Objekts), nie Kartenobjekte. Ein Objekt mit einem gleichartigen innerhalb von 25 m (Pollerreihe, Bankpaar, Laternenfolge) wird weggelassen, und ein Kontrollstandort weicht jedem anderen Punktsymbol, damit er keine Bäume oder Brunnen verdrängt. Das Tag ist die Stelle, an der ein Planungswerkzeug die Standorte findet (ADR 0074).
- Zugangsbeschränkungen aus OSM. Private und gesperrte Wege (`access=private|no`, `foot=private|no`) werden auf Sprintkarten 520-Kandidaten, ein Streifen von 2 m (Straßen 3 m) um den Weg, und die Wegelinie entfällt. `foot=yes|permissive|designated` lässt den Weg, wie er war. Flächen mit `access=private|no` (Landnutzung, Garten, Platz, Parkplatz, Schulgelände) sind 520 in ihrer Form (ADR 0073).
- Durchgänge durch Gebäude aus OSM. Wege mit `tunnel=building_passage` und überdachte Wege (`covered=yes`, Arkaden) sind auf Sprintkarten jetzt 522-Kandidaten, als Streifen von 3 m (oder `width`) entlang des Weges, das Gebäude bleibt ganz. Bisher war ein gepflasterter Durchgang gar kein Kandidat und ein ungepflasterter nur eine Linie 505.1. Die Regeln kennen dafür das neue Feld `corridor_m` (ADR 0072). Bestehende Karten bekommen sie mit `run --force`.
- OSM-Multipolygon-Relationen werden gelesen. Parks, Gewässer, Wälder und Gebäude, die nur als Relation vorliegen (Tags an der Relation, Ringe aus Wegen ohne Tags), erreichen jetzt die Regeln. freemapper verkettet die Wege in beliebiger Richtung zu geschlossenen Ringen und behält Löcher, der Innenhof liegt also nicht im Gebäude und der Teich nicht im Park. Ein äußerer Weg mit den Tags der Relation wird einmal gezeichnet. Im Beispiel Viktoriapark steigen die gespeicherten OSM-Objekte von 752 auf 755 und die Kandidaten von 2.093 auf 2.094: Ein Teich (301) ist neu, zwei Wiesen mit Löchern liegen unter dem ALKIS-Park. Im Park Sanssouci kommt eine Wiese mit 13 Löchern hinzu (1.047 auf 1.048 Objekte), sobald sein Beispiel neu gebaut wird. Bestehende Karten bekommen Relationen mit `run --force` (ADR 0070).
- Parks sind gelb. Grünanlagen, Parks, Siedlungsgrün und Grünland aus ALKIS werden offenes Gelände (401) in der Karte, Wald und Gehölz Wald (405), Heide 403. Friedhöfe, anderes Landwirtschaftsland, Moor und Sumpf sind Kandidaten, ebenso Parks und Rasen aus OSM, wo kein ALKIS-Park liegt (ADR 0069).
- Die belaufbare Fläche. Auf der Sprintkarte liegt unter allem eine Fläche im unteren Braun von 501.5 über das ganze Blatt, Symbol 990 im eigenen Kartenteil `base`, Tag `freemapper:source=base`. Sie füllt jede Lücke, die nichts anderes zeigt; eine Sprintkarte ohne sie ist unvollständig. Ihre Farbe steht hinter allen anderen, in OOM, OCAD (Farbe 32, Symbol 990.000) und im PDF. Drucken Sie Sprintkarten farbig, denn in Graustufen liegt das Braun nahe am Papierweiß (ADR 0069).

### Behoben

- Punkte in Gebäuden fallen weg. Ein Kandidatenpunkt auf Sprintkarten (Denkmal, Laterne, Papierkorb, Brunnen, Quelle), dessen Mitte in einem Gebäudegrundriss liegt, ist weder sichtbar noch nutzbar und wird entfernt; die dunklen Punkte auf der Kirche im Lübeck-Beispiel waren Denkmäler. Bäume bleiben. Lübeck: 526 von 10 auf 4, 531 von 195 auf 180; Viktoriapark: 531 von 121 auf 119 (ADR 0076).
- Der Straßenrand macht keinen Haken mehr um jeden Straßenbaum. Die Bordsteinlinien der Befahrung (Kandidaten 501.1) liefen um jeden Baumschlitz im Parkstreifen, deshalb zeigte die Trainingskarte an jedem Baum einen Haken. freemapper ersetzt jetzt einen Bordstein-Umweg durch seine Sehne, wenn er auf dem Papier weniger als 1,5 mm² umschließt (24 m² bei 1:4000), und lässt einen geschlossenen Bordsteinring dieser Größe weg. Die Regel beurteilt einen Umweg danach, wie weit er aus seiner Sehne ragt (mindestens 0,3 davon, höchstens 10 m breit); so verschwinden auch die breiteren Trapezschlitze, und geschlossene Bordsteinringe um einzelne Baumscheiben entfallen. Ein Umweg muss außerdem auf einer geraden Strecke liegen; so entstehen weder eine Spitze an einem Baum noch eine Diagonale über einen Versatz des Bordsteins. Kurven, größere Buchten und die Karte selbst bleiben unverändert. Im Viktoriapark verlieren die Bordsteine der Befahrung 735 von 6.522 Stützpunkten und 5 von 81 Linien. Bestehende Projekte bekommen die saubereren Linien mit `run --force` (ADR 0071).

### Geändert

- Die Beispiele Viktoriapark und Lübeck sind mit den neuen OSM-Regeln neu gebaut. Viktoriapark: Kartenobjekte 789 auf 740 (505.1: 58 auf 9, weil verdichteter Kies zum Kandidaten wird), Kandidaten 2.089 auf 2.327 (531: 5 auf 121, davon 116 Kontrollstandorte; 520: 73 auf 117; 522: 179 auf 211; 501.6: 45 neu, 501.9: 4 neu; 526: 1 auf 3). Lübeck: Kartenobjekte 2.294 auf 2.281, Kandidaten 1.528 auf 1.859 (522: 185 auf 280; 531: 0 auf 195; 520: 1.090 auf 1.125; 526: 0 auf 10; 501.6: 13 neu; 505.1: 35 auf 16). Die gespeicherten OSM-Objekte steigen von 755 auf 1.371 (Viktoriapark) und von 234 auf 927 (Lübeck).
- Verdichteter Kies und Feinkies (`surface=compacted|fine_gravel`) ist nach ISSprOM 501 fest und deshalb kein unbefestigter Weg mehr in der Karte, sondern ein 501.6-bis-501.9-Kandidat nach `width`. Ab 3 m Breite sind unbefestigte Wege 505.2. Ein Wirtschaftsweg ohne `surface` folgt `tracktype` (ADR 0073).
- Baumgruppenflächen (402, 405) verdecken keine Wege mehr. freemapper schneidet sie auch an befestigten Kandidaten und an Wegen (505, 506, 507) aus. Die punktierte Vegetationsgrenze (416) gibt es nur noch auf der Schulhofkarte, auf der Sprintkarte zeigt sich der weiße Wald auf Gelb oder Braun. Im Viktoriapark bleiben dadurch mehr Bäume Punkte (1 128 statt 835), die 49 Grenzlinien entfallen (ADR 0069).
- Alle elf Beispielprojekte sind neu erzeugt, mit gelben Parks und der belaufbaren Fläche.
- Dichte Baumgruppen werden jetzt auch im Park zur Fläche, die man auf der Karte sieht. Drei oder mehr Bäume, die auf dem Papier näher als 2,5 mm (10 m bei 1:4000) beieinander stehen oder deren Kronen sich fast berühren, werden Wald (405), wenn ihre Kronen mindestens die Hälfte der Fläche decken, sonst offenes Gelände mit verstreuten Bäumen (402). Auf weißem Grund bekommt der Wald eine punktierte Vegetationsgrenze (416), damit er sichtbar ist. Auf Pflaster bleibt es bei 501.3, ab 9 Bäumen. Baumreihen an Straßen und Alleen bleiben Punkte. Im Park Sanssouci sinkt die Zahl der Baumpunkte von 1 654 auf 782 (48 Flächen 402), im Viktoriapark von 1 660 auf 942 (43 Wälder, 9 Flächen 402; ADR 0066, Nachtrag).
- Die Trainingskarte (PDF) zeichnet die Punktmuster von Flächen, etwa die weißen Punkte von 402. Bisher sah 402 dort aus wie 401.
- Überlappende Mauern, Zäune und Hecken. Laufen zwei solche Linien parallel und näher, als ihre Symbole auf dem Papier dürfen, verliert die schwächere das Stück: die schlechtere Quelle, bei gleicher Quelle das schwächere Symbol (Mauer vor Zaun vor Hecke vor Stützmauer). Mauerstücke bis 0,5 m neben einem Gebäudeumriss fallen weg, ebenso Bäume bis 1 m neben einer dieser Linien. Im Park Sanssouci sinken die parallelen Linienpaare unter 1 m Abstand von 162 auf 4 (ADR 0068).
- Alle elf Beispielprojekte sind mit diesen Regeln neu erzeugt, ihre Kartenobjekte bleiben gleich.
- freemapper schreibt, kopiert und löscht im Projektordner jetzt mit der gemeinsamen Bibliothek `freemapper-fs` aus freemapper-core 0.2.0 und pflegt keine eigene Kopie mehr. Für Nutzer bleibt alles gleich, auch der Wortlaut der Meldungen (ADR 0067).

## [0.7.0] - 2026-11-09

### Hinzugefügt

- Überdachungen und Durchfahrten in weiteren Bundesländern. In jedem Bundesland außer Sachsen und Sachsen-Anhalt werden Überdachungen und Carports aus ALKIS Kartenobjekte (522) auf beiden Kartenarten, an den Gebäuden abgeschnitten wie in Berlin. Durchfahrten und Arkaden werden Kandidaten 522, das Gebäude bleibt ganz. freemapper liest sie aus den Gebäuden, die es ohnehin lädt, ein zusätzlicher Download entfällt. Datenstand ist das Datum des ALKIS-Objekts. Eine vorhandene Karte bekommt die Überdachungen erst mit `run --force` (ADR 0061).
- Brandenburg: das vollständige ALKIS aus dem NAS-Dienst der LGB. Mauern und Mauerkanten, Zäune, Hecken, Denkmäler, Brunnen, Gräben und Bäche werden Kandidaten, auf Sprintkarten auch Böschungsoberkanten und Brücken. Das Symbol wählt der Code im Kataster, `freemapper:alkis` nennt seine Bezeichnung, z. B. `Mauerkante, rechts`. Datenstand ist das Datum, seit dem die Objektversion im Kataster steht. Liegen die zwei Kanten einer Mauer einander zugewandt und auf dem Papier näher als 0,55 mm (2,2 m bei 1:4000), zeichnet freemapper eine Linie 515 in der Mitte mit `freemapper:alkis=Mauer (Mauerkanten gepaart)`, und `freemapper:also` nennt die andere Kante. Im Park Sanssouci sind es 264 statt 271 Mauerlinien. Die Ebenen liegen in `data/alkis-nas/`, etwa 1 MB je km² (ADRs 0062, 0063).
- Potsdam: Die Bäume der Stadtkarte werden Kandidaten 418, jeder mit seinem eigenen Aufnahmedatum. Die Daten kennen keine Krone, Baumstümpfe fallen weg. OSM-Bäume bis 2 m neben einem Potsdamer Baum fallen weg. Eine Antwort über 256 MB bricht freemapper beim Lesen ab. Die neue Namensnennung `Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0` erscheint, wenn Potsdamer Bäume unter den Kandidaten sind (ADR 0065).
- Punktgeneralisierung. Punktsymbole halten den Abstand von 0,15 mm ein, den die ISSprOM für Linien setzt. Drei oder mehr Bäume, die einander zu nahe stehen, werden eine Fläche, wenn mindestens die Hälfte ihrer Kronen auf Pflaster oder auf offenem Gelände (401 bis 404) liegt: auf Pflaster befestigte Fläche mit verstreuten Bäumen (501.3), auf offenem Gelände Wald (405), wenn die Kronen mindestens die Hälfte der Gruppe decken, sonst offenes Gelände mit verstreuten Bäumen (402). Anderswo bleiben die Bäume Punkte und werden ausgedünnt, denn ein weißer Wald auf weißem Grund würde sie verstecken. Baumreihen bleiben einzelne Baumsymbole. Die Fläche trägt `freemapper:source=generalization` und nennt ihre Bäume in `freemapper:source_id` (ADR 0066).

### Geändert

- Zu dicht stehende Punkte werden ausgedünnt. Kämen sich zwei Punktsymbole auf dem Papier näher, als die Norm erlaubt, behält freemapper das größere, dann das aus der besseren Quelle, dann das mit der größeren Krone. Der bleibende Punkt nennt die anderen in `freemapper:generalized`. Das ändert Kandidaten und Trainingskarte in jedem Bundesland, auch in Berlin. Die Kartenobjekte bleiben gleich (ADR 0066).
- Bäume und andere Punkte in Sperrgebiet-Kandidaten (520) bleiben in den Kandidaten mit `freemapper:hidden=520`, die Trainingskarte lässt sie aber weg. Auf Schulhofkarten zeigt das PDF deshalb keinen Punkt außerhalb des Schulgeländes (ADR 0066).
- Doppel: Register vor ALKIS vor Straßenbefahrung vor OSM gilt jetzt in jedem Bundesland und auch für Bäche und Gräben (305, 306). Eine OSM-Linie, die zu mindestens 80 % ihrer Länge höchstens 1,5 m neben einer Linie des vollständigen ALKIS mit demselben Symbol liegt, fällt weg, und `freemapper:also` nennt sie (ADR 0063).
- Größenschätzung und Platzprüfung rechnen das vollständige ALKIS und die Stadtbaumkataster als optionalen Teil (2 und 1 MB je km²). Fehlt der Platz, lässt freemapper sie weg und behält die gespeicherten.
- `run` meldet eine behaltene Karte ohne die ALKIS-Überdachungen jetzt in jedem Bundesland mit Überdachungen. Die Meldung lautet nun `The map lacks the ALKIS canopies and carports (522) …`.
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

- Kleingärten (ALKIS-Nutzung „Sport-, Freizeit- und Erholungsfläche“ mit `bez` „Kleingarten“) sind in jedem Bundesland Kandidaten für verbotenes Gebiet (520). Parks, Sportanlagen und Spielplätze bleiben offen.

### Behoben

- Die Höhenlinien-Kandidaten nennen in `freemapper:source` das Geländemodell ihres Bundeslandes, z. B. `laiv-mv-dgm1` oder `hvbg-dgm1`. Bisher stand in jedem Bundesland `lgb-dgm1` da.

## [0.4.0] - 2026-11-08

### Hinzugefügt

- Niedersachsen (LGLN): Orthophoto aus dem WMS, Gelände- und Oberflächenmodell (DGM1/DOM1) aus dem WCS in Stücken von 2 km, Flugdatum und Datenstand aus den STAC-Suchen, Gebäude und Nutzung aus ALKIS. Das nDOM ist DOM1 − DGM1; ist das DOM1 mehr als zwei Jahre älter als das Luftbild, warnt freemapper mit beiden Daten. Eine Meldung sagt, ob die Bäume beim Bildflug belaubt waren. Die Karten nutzen UTM-Zone 32. Bremen gehört noch nicht dazu.
- Rheinland-Pfalz (LVermGeo RP): Orthophoto aus dem WMS, als Flugdatum gilt das Erstellungsdatum des Blocks. Gelände- und Oberflächenmodell (DGM1/DOM1) kommen aus Laserkacheln von 1 km, deren Namen und Daten die Befliegungsübersicht `Laserbefliegung` nennt; für Kacheln ohne Flugjahr liest freemapper die Metalink-Datei des Bundeslandes. Hat der Server eine Kachel nicht, meldet freemapper sie als fehlend und fragt nach 30 Tagen wieder. Gebäude und Nutzung kommen aus ALKIS. Die Karten nutzen UTM-Zone 32. An der Grenze zu Hessen (Mainz, Wiesbaden) liest freemapper ALKIS und OSM beider Länder.
- Gewässer auf allen Karten: Flüsse, Kanäle, Seen, Teiche, Hafenbecken und Meer aus ALKIS werden unpassierbares Gewässer (301.1) mit Uferlinie (301.6), ohne Linien zwischen den Katasterflächen und am Kartenrand. Gewässer, die im Mittel schmaler als 2 m sind, kommen als Kandidaten. Aus OSM kommen Teiche, Schwimmbecken, Brunnenbecken, Bäche, Gräben, Brunnen, Quellen, auf Schulhofkarten Trinkbrunnen und auf Sprintkarten Brücken als Kandidaten. Höhenlinien enden am Ufer, und Brunnenpunkte im Wasser (meist Fontänen in einem Becken) fallen weg. In OCAD tragen Fläche und Uferlinie die Nummern 301.001 und 301.002. Eine vorhandene Karte bekommt die Gewässer erst mit `run --force`.

### Geändert

- Die Nordlinien liegen im Kartenteil „default part“ bei den übrigen Kartenobjekten, mit dem Tag `freemapper:source=north-lines`. In OOM lassen sie sich an kleinen Objekten unterbrechen, und in der Trainingskarte liegen Mauern und Zäune darüber. Titel, Maßstab und Namensnennung bleiben im Teil „peripheral information“. Eine vorhandene Karte ändert sich erst mit `run --force`.
- Die Farben folgen den IOF-Druck- und Farbdefinitionen Rev. 4 (September 2024, gültig ab 1.12.2024) statt der älteren Werte aus dem OOM-Symbolsatz: Braun ist 25/75/100/0 statt 0/56/100/18, Grün 80/0/100/0 statt 76/0/91/0, Braun 50 % 10/35/50/0. Gebäude sind wie in ISSprOM 2019-2 mit 50 % Schwarz gefüllt statt 60 %. Das gilt für OOM-Karte, OCAD-Datei und Trainings-PDF. Namen und Reihenfolge der Farben bleiben gleich.
- Längere Meldungen sind klarer formuliert.
- Reicht ein Kartengebiet über alle unterstützten Bundesländer hinaus (ins Ausland, aufs Meer oder in ein Bundesland, das freemapper noch nicht unterstützt, etwa Bremen oder Thüringen), warnt freemapper beim Laden, dass dort Luftbild und ALKIS fehlen können. Das gilt für alle Länder.

### Behoben

- Wo die Gewässerflächen zweier Bundesländer an der Grenze mitten im Fluss nicht ganz aneinanderstoßen (Mainz, Rhein), lief eine Uferlinie durch den Fluss. freemapper schließt jetzt Lücken unter 0,5 m zwischen Gewässerflächen, bevor es die Uferlinien zeichnet. Das gilt auch für Flurstücke desselben Landes. Brücken und Straßen (ab 0,8 m) bleiben Land, Inseln behalten ihr Ufer.

## [0.3.0] - 2026-11-08

### Hinzugefügt

- Hamburg (LGV): Orthophoto aus dem WMS (Stand 2026), Gelände- und Oberflächenmodell aus den Archiven des Bundeslandes, Gebäude und Nutzung aus ALKIS. Die Karten nutzen UTM-Zone 32.
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
- Die Größenschätzung vor dem Download zählt bei Bundesländern mit Kacheln ganze Kacheln und markiert die, die schon im Cache liegen.
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
