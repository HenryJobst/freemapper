# Programmtexte von freemapper, Deutsch (ADR 0092). Englisch ist die Referenz: Jede Sprache hat
# dieselben Kennungen und dieselben Variablen. Texte aus fremden Programmen (Fehler des
# Betriebssystems, Parser-Meldungen) kommen in Variablen an und bleiben, wie sie sind.

## Der Schalter --lang

cli-lang-unknown = Für { $value } hat freemapper keine Texte; vorhanden: { $available }

## init

cli-part-needs-osm = --part wählt einen Teil eines OSM-Gebiets und braucht --osm; mit --place suchen Sie zuerst die Kennung und nehmen dann --osm <id> --part <n>
cli-reading-file = beim Lesen von { $path }
cli-no-area-for-place = Zu „{ $query }“ gibt es kein Gebiet. Besteht der Ort aus mehreren getrennten Teilen, wählen Sie einen mit --osm <id> --part <n>, oder nehmen Sie --geojson
cli-using-place = Gewählt: { $name } ({ $id })
cli-created-project = Projekt { $path } angelegt

## Zwischenspeicher

cli-no-cache-dir = Dieses System hat keinen Ordner für den Zwischenspeicher; geben Sie einen mit --cache-dir an

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] 1 Kachel geladen
       *[other] { $count } Kacheln geladen
    }
cli-wrote = { $path } geschrieben
cli-kept-map = { $path } behalten (die Datei kann Ihre Arbeit enthalten)
cli-map-lacks-templates = Die Karte verweist noch nicht auf die Bildvorlagen. Fügen Sie sie in OOM hinzu (Vorlagen > Vorlage öffnen …) oder ersetzen Sie die Karte mit --force; die alte bleibt als .omap.bak erhalten.
cli-map-lacks-buildings = Die Karte hat noch keine ALKIS-Gebäude. Um sie zu ergänzen, ersetzen Sie die Karte mit --force; die alte bleibt als .omap.bak erhalten.
cli-map-lacks-canopies = Der Karte fehlen die ALKIS-Überdachungen und Carports (522), die inzwischen Kartenobjekte sind. Um sie zu ergänzen, starten Sie mit --force; die alte Karte bleibt als .omap.bak erhalten.

## PDF

cli-pdf-without-credits = PDF ohne Quellenangaben: ODbL und dl-de/by-2-0 verlangen, dass Sie die Quellen nennen, wenn Sie die Karte weitergeben; siehe ATTRIBUTION.txt
cli-wrote-pdf = { $path } geschrieben (die Karte ohne Kandidaten)
cli-wrote-pdf-candidates = { $path } geschrieben (Trainingskarte, in der jeder Kandidat als Kartenobjekt gezeichnet ist; prüfen Sie sie im Gelände)

## OCAD

cli-ocad-kept = { $path } behalten (die Datei kann Ihre Arbeit enthalten; --force ersetzt sie und behält die alte als .ocd.bak)
cli-ocad-wrote = { $count ->
        [one] { $path } geschrieben (1 Objekt)
       *[other] { $path } geschrieben ({ $count } Objekte)
    }
cli-ocad-left-out = Nicht in der OCAD-Karte: { $skipped } Objekte (kein Symbol für OCAD, keine Fläche oder außerhalb der Zeichenfläche von OCAD von ±2 m) und { $dropped } entartete Linienteile oder Löcher.
cli-ocad-load-symbols = Wählen Sie in OCAD Symbol > Farben und Symbole laden von … und nehmen Sie den offiziellen Symbolsatz ISSprOM 2019 oder den Schulsymbolsatz; er ersetzt die Platzhaltersymbole von freemapper anhand der Nummern.

## select

cli-select-running = Gebietsauswahl für { $path }: Drücken Sie auf der Seite „Fertig“ oder beenden Sie mit Strg+C.
cli-select-no-browser = Der Browser ließ sich nicht öffnen ({ $error }); öffnen Sie die Adresse oben.

## Ordnersperre

cli-lock-held = Ein anderer freemapper-Lauf verwendet diesen Ordner (seit { $time }, gestartet { $who }). Warten Sie, bis er fertig ist, oder beenden Sie ihn zuerst.
cli-lock-who-cli = vom Terminal
cli-lock-who-mcp = von einem KI-Agenten (MCP)
cli-lock-who-page = von der Gebietsauswahl-Seite
cli-lock-who-unknown = von einem anderen Programm
cli-lock-failed = Der Projektordner ließ sich für diesen Lauf nicht sperren: { $error }
cli-lock-stuck = Im Weg liegt etwas, das keine Sperrdatei ist und sich nicht entfernen ließ

## Lauf-Routen der Gebietsauswahl-Seite

cli-run-bad-request = Die Optionen für den Lauf sind nicht gültig: { $error }
cli-run-no-project = In diesem Ordner gibt es noch kein gespeichertes Projekt. Speichern Sie zuerst das Gebiet.
cli-run-already-running = Für dieses Projekt läuft schon ein Lauf ({ $job }). Warten Sie, bis er fertig ist, oder brechen Sie ihn ab.
cli-run-disk-stops = Nicht genug freier Speicherplatz für den Download: { $messages }
cli-run-busy-elsewhere = Ein anderes Projekt dieses Programms führt gerade einen Lauf aus. Projekte teilen sich den Zwischenspeicher, deshalb läuft immer nur einer. Versuchen Sie es erneut, wenn er fertig ist.
cli-run-nothing-to-cancel = Es läuft nichts, also gibt es nichts abzubrechen.
cli-run-unknown-job = Für dieses Projekt gibt es keinen Lauf { $job }.
cli-status-too-fast = Statusabfragen kommen zu oft. Fragen Sie höchstens fünfmal pro Sekunde.
cli-open-folder-failed = Der Ordner ließ sich nicht öffnen ({ $error }).
cli-open-folder-no-opener = Dieser Rechner hat kein Programm zum Öffnen von Ordnern ({ $program }). Öffnen Sie den Ordner von Hand.
cli-open-folder-missing = Der Projektordner existiert noch nicht. Speichern Sie zuerst das Gebiet.
cli-open-folder-refused = Der Ordnername ist nicht sicher genug, um ihn an den Dateimanager weiterzugeben.

## update (Spezifikation 3.4 bis 3.6)

update-source-build = Dieses freemapper wurde aus dem Quellcode gebaut. Aktualisieren Sie es mit git pull und cargo build --release --locked.
update-current = freemapper { $version } ist aktuell.
update-available = freemapper { $new } ist verfügbar (Sie haben { $old }), erschienen am { $released }.
update-details =
    Versionshinweise: { $notes }
    Download: { $size } MB
update-decimal-separator = ,
update-ask = Jetzt installieren? [j/N]
update-not-installed = Es wurde nichts installiert.
update-needs-yes = Die Eingabe ist kein Terminal, deshalb kann freemapper vor der Installation nicht fragen. Mit freemapper update --yes wird ohne Rückfrage installiert.
update-installing = freemapper { $version } wird geladen und geprüft
update-done = freemapper wurde von { $old } auf { $new } aktualisiert. Versionshinweise: { $notes }
update-mcp-restart = Ein laufender MCP-Server behält die alte Version, bis der KI-Assistent ihn neu startet.
update-rolled-back = Zurück zu freemapper { $version }. Die verlassene Version liegt als { $path } bereit.
update-not-writable = In den Programmordner { $path } kann nicht geschrieben werden, und freemapper fragt nicht nach Administratorrechten. Entpacken Sie die neue Version dort von Hand oder verschieben Sie freemapper in einen Ordner, in den Sie schreiben dürfen.
update-too-old = freemapper { $new } lässt sich nicht per Update von { $old } aus installieren. Installieren Sie { $needed } oder neuer von Hand, siehe { $notes }
update-busy = Im Programmordner läuft schon ein Update oder ein Zurücksetzen (Sperrdatei { $path }). Warten Sie, bis es fertig ist, und versuchen Sie es dann erneut.
update-no-previous = Es gibt keine vorherige Version, zu der zurückgekehrt werden kann ({ $path } fehlt).
update-failed = Das Update ist fehlgeschlagen: { $error }
update-mode-set = Update-Modus { $mode } in { $path } gespeichert.
update-mode-env = FREEMAPPER_UPDATE={ $value } hat Vorrang vor dieser Einstellung, solange die Variable gesetzt ist.
update-mode-failed = { $path } ließ sich nicht schreiben: { $error }
update-no-config-dir = Dieses System hat keinen Ordner für Einstellungen; setzen Sie FREEMAPPER_CONFIG_DIR.

## Fehler. Was andere Programme formulieren (Betriebssystem, Parser, Server), kommt in Variablen
## an und bleibt, wie es ist; das gilt auch für Einzelheiten, die freemapper bisher nur auf
## Englisch kennt.

cli-error-prefix = Fehler
error-cancelled = abgebrochen
cli-cancelled-kept = abgebrochen; behalten: { $files }
error-io = { $path }: { $message }
error-io-disk-full = Der Datenträger ist voll, deshalb konnte freemapper { $path } nicht schreiben. Geben Sie Platz frei und versuchen Sie es noch einmal

## Fehler: Projekt

error-project-already-exists = Das Projekt gibt es schon: { $path }
error-project-map-exists = Die Karte gibt es schon (sie kann Ihre Arbeit enthalten); --force ersetzt sie: { $path }
error-project-invalid-scale = ungültiger Maßstab 1:{ $scale }
error-project-scale-not-allowed = Der Maßstab 1:{ $scale } ist für { $kind } nicht erlaubt; nehmen Sie { $allowed }
error-project-scale-not-allowed-enlarge = Der Maßstab 1:{ $scale } ist für { $kind } nicht erlaubt; nehmen Sie { $allowed }. Zeichnen Sie in 1:4000, freemapper-print vergrößert beim Drucken auf 1:3000
word-kind-schoolyard = Schulhofkarten
word-kind-sprint = Sprintkarten
error-project-toml = ungültige Projektdatei: { $message }
error-project-not-fetched = Es sind noch keine Kacheln geladen; rufen Sie zuerst `freemapper fetch` auf
error-project-ocd = OCAD-Export: { $message }
error-project-template-outdated = { $path }: Bild und Weltdatei passen nicht zum Kartengebiet; rufen Sie `freemapper derive` noch einmal auf
error-project-tiles-outdated = Das Kartengebiet braucht die Kacheln { $tiles }, die nicht geladen sind; rufen Sie `freemapper fetch` auf
error-project-buildings-not-fetched = Für dieses Kartengebiet sind keine ALKIS-Gebäude gespeichert; rufen Sie zuerst `freemapper fetch` mit Netzverbindung auf
error-project-orthophoto-not-fetched = Für dieses Kartengebiet ist kein Orthophoto gespeichert; rufen Sie zuerst `freemapper fetch` mit Netzverbindung auf
error-project-terrain-not-fetched = Für dieses Kartengebiet ist kein Geländemodell gespeichert; rufen Sie zuerst `freemapper fetch` mit Netzverbindung auf
error-project-candidates-outdated = Die gespeicherten OSM-Kandidaten decken das Kartengebiet nicht ab; rufen Sie `freemapper fetch` auf
error-project-stored = { $path }: { $message }; rufen Sie `freemapper fetch` noch einmal auf
error-project-area-too-large = Das Kartengebiet mit Puffer misst { $width } × { $height } m, mehr als { $limit } m je Seite; verkleinern Sie das Kartengebiet oder buffer_m in freemapper.toml
error-project-outside-coverage = Das Kartengebiet liegt außerhalb der unterstützten Bundesländer (Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen, Rheinland-Pfalz); bisher hat freemapper nur für diese Datenquellen

## Fehler: Speicherplatz

error-space-required = Der Speicherplatz reicht nicht für { $part }. Gebraucht werden { $needed }, aber { $path } hat über die Reserve von { $reserve } hinaus nur { $left } frei. Geben Sie Platz frei und rufen Sie `freemapper fetch` noch einmal auf
error-space-skipped = Aus Platzmangel lässt freemapper { $part } weg. Gebraucht werden { $needed }, aber { $path } hat über die Reserve von { $reserve } hinaus nur { $left } frei. Geben Sie Platz frei und rufen Sie `freemapper fetch` noch einmal auf
word-part-map-data = die Kartendaten aus ALKIS und OSM
word-part-terrain = das Geländemodell
word-part-orthophoto = das Orthophoto
word-part-surface = das Oberflächenmodell für die nDSM-Vorlage
word-part-osm-update = die Aktualisierung des gespeicherten OSM-Auszugs
word-part-k-five = die Berliner Kartenvorlage K5
word-part-berlin-layers = die Berliner ALKIS-Bauwerke, das Baumkataster, die Straßenbefahrung und die Punktebenen
word-part-state-layers = die vollständigen ALKIS-Bauwerke und die Baumkataster

## Fehler: Herunterladen

error-fetch-http = Das Herunterladen von { $url } ist gescheitert: { $message }
error-fetch-corrupt-archive = Von { $url } kam kein vollständiges Kachelarchiv: { $message }
error-fetch-corrupt-extract = Von { $url } kam kein vollständiger OSM-Auszug: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] Die Kachel { $tiles } gehört nicht zu den veröffentlichten Kacheln; verkleinern Sie das Kartengebiet oder buffer_m in freemapper.toml
       *[other] Die Kacheln { $tiles } gehören nicht zu den veröffentlichten Kacheln; verkleinern Sie das Kartengebiet oder buffer_m in freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] Ohne Netzverbindung und nicht im Zwischenspeicher: { $product }, Kachel { $tiles }; rufen Sie `freemapper fetch` mit Netzverbindung auf
       *[other] Ohne Netzverbindung und nicht im Zwischenspeicher: { $product }, Kacheln { $tiles }; rufen Sie `freemapper fetch` mit Netzverbindung auf
    }
word-product-orthophoto = Orthophoto (DOP20)
word-product-dtm = Geländemodell (DGM1)
word-product-image-dsm = Oberflächenmodell (bDOM)
word-product-saxony-dtm = Geländemodell (GeoSN DGM1)
word-product-saxony-dsm = Oberflächenmodell (GeoSN DOM1)
error-fetch-osm-extract-missing = Im Zwischenspeicher liegt kein OSM-Auszug; rufen Sie `freemapper fetch` mit Netzverbindung auf
error-fetch-not-found = { $name } gibt es auf dem Server nicht
error-fetch-refused = { $url }: Der Server hat die Anfrage abgelehnt (HTTP { $status })
error-fetch-refused-detail = { $url }: Der Server hat die Anfrage abgelehnt (HTTP { $status }): { $detail }
error-fetch-too-large = Die Datei von { $url } ist größer als { $limit } MB; abgebrochen
error-fetch-not-stored = Für dieses Kartengebiet ist nichts gespeichert ({ $label }): { $path } fehlt, lässt sich nicht lesen oder deckt das Gebiet nicht ab; rufen Sie zuerst `freemapper fetch` mit Netzverbindung auf
error-fetch-range-ignored = { $url }: Der Server hat den Bytebereich { $range } nicht beachtet (HTTP { $status })
error-fetch-currency-csv = Aktualitäts-CSV, Zeile { $line }: { $message }
error-alkis = ungültige ALKIS-Daten: { $message }
error-osm = OSM-Daten: { $message }

## Fehler: Vorlagen und Raster

error-derive-zip = Kachelarchiv: { $message }
error-derive-image = Bild: { $message }
error-derive-no-tiles = Es gibt keine Kacheln, aus denen sich etwas ableiten lässt
error-derive-area-too-large = Das Kartengebiet ist zu groß für eine Vorlage: { $width } × { $height } Pixel (höchstens { $limit } je Seite); verkleinern Sie das Kartengebiet oder buffer_m in freemapper.toml
error-raster-world-file = ungültige Weltdatei: { $message }
error-raster-misaligned = Die Raster liegen nicht auf demselben Pixelgitter: { $message }
error-raster-pixel-size = Die Raster haben verschiedene Pixelgrößen: { $message }

## Fehler: Kartengebiet, Georeferenzierung, Kartendatei, OCAD-Datei

error-area-geojson = ungültiges GeoJSON: { $message }
error-area-no-polygon = Das GeoJSON enthält kein Polygon
error-area-multi-polygon = MultiPolygon mit { $parts } Teilen; das Kartengebiet muss ein einzelnes Polygon sein
error-area-out-of-range = Koordinate außerhalb des Wertebereichs: Breite { $lat }, Länge { $lon }
error-area-not-an-area = kein brauchbares Kartengebiet: { $message }
error-georef-date = Das Datum { $date } liegt außerhalb der Gültigkeit des World Magnetic Model
error-georef-map-range = Der Punkt E { $easting } N { $northing } liegt außerhalb des Koordinatenbereichs der Karte
error-omap-xml = ungültiges OOM-XML: { $message }
error-omap-missing-element = Das Element <{ $name }> fehlt
error-ocd-duplicate-symbol = Die Symbolnummer { $symbol } kommt zweimal vor
error-ocd-unknown-symbol = Ein Objekt verweist auf das Symbol { $symbol }, das in der Datei fehlt
error-ocd-wrong-type = Ein Objekt des Symbols { $symbol } hat nicht den Typ seines Symbols

## Die Hilfe (`--help`)

cli-heading-usage = Aufruf:
cli-heading-commands = Befehle:
cli-heading-options = Optionen:
cli-heading-arguments = Argumente:
cli-help-possible-values = mögliche Werte
cli-help-default = Standard
cli-arg-help = Hilfe anzeigen
cli-arg-version = Version anzeigen
cli-help-help = Die Hilfe eines Befehls anzeigen
cli-arg-help-command = Der Befehl, dessen Hilfe angezeigt wird, zum Beispiel run
cli-parse-hint = Mehr dazu mit '--help'.
cli-parse-unknown-argument = Unbekanntes Argument { $arg }
cli-parse-missing-required = Pflichtargument fehlt: { $args }
cli-parse-invalid-value = Ungültiger Wert { $value } für { $arg }
cli-parse-invalid-value-choices = Ungültiger Wert { $value } für { $arg }. Mögliche Werte: { $values }
cli-parse-invalid-subcommand = Unbekannter Befehl { $value }
cli-parse-missing-subcommand = Ein Befehl ist nötig
cli-parse-too-many-values = Unerwarteter Wert { $value } für { $arg }
cli-parse-argument-conflict = { $arg } lässt sich nicht zusammen mit { $other } verwenden

help-freemapper = Erzeugt OOM-Startprojekte aus Geodaten
help-freemapper-lang = Sprache der Programmtexte, `de` oder `en` (Standard: FREEMAPPER_LANG, dann die Systemsprache, dann Englisch)
help-init = Einen Projektordner für ein Kartengebiet anlegen
help-init-dir = Projektordner, der angelegt wird
help-init-geojson = GeoJSON-Datei mit dem Polygon des Kartengebiets (EPSG:4326)
help-init-osm = OSM-Objekt des Kartengebiets, z. B. way/992423119
help-init-place = Name des Ortes, z. B. "Clara-Zetkin-Schule Strausberg"; freemapper nimmt den besten Treffer mit einer Fläche
help-init-part = Welcher Teil einer OSM-Fläche aus mehreren getrennten Teilen verwendet wird (1 ist der größte)
help-init-kind = Kartenart
help-arg-scale = Maßstabszahl eines neuen Projekts: Schulhof 1000 (Standard); Sprint 4000 (Standard) oder 5000. Andere Werte werden abgelehnt, weil freemapper-print sie ablehnt
help-init-title = Kartentitel für die Randangaben (Standard: der Ordnername)
help-export = Das Startprojekt (OOM-Karte, Kartenvorlagen, Namensnennung) in einen Projektordner schreiben
help-export-dir = Projektordner
help-arg-force = Eine vorhandene Kartendatei ersetzen und die alte als `<name>.omap.bak` behalten
help-export-ocad = Zusätzlich einen OCAD-Startpunkt nach `ocad/` schreiben (Format OCAD 12); eine vorhandene OOM-Karte bleibt dabei, außer mit --force
help-export-pdf = Zusätzlich `<name>.pdf` schreiben, die druckfertige Karte (nur die Kartenobjekte, keine Kandidaten); eine vorhandene OOM-Karte bleibt dabei, außer mit --force
help-export-pdf-candidates = Zusätzlich `<name>-candidates.pdf` schreiben, die Trainingskarte (die Karte und jeder Kandidat als Kartenobjekt); eine vorhandene OOM-Karte bleibt dabei, außer mit --force
help-export-pdf-scale-bar = Die Maßstabsleiste auf die PDFs zeichnen (standardmäßig aus, deutsche Karten haben meist keine; die Maßstabszahl bleibt im Titelblock)
help-export-pdf-sources = Quellenzeilen auf den PDFs: `auto` druckt die ganze Liste und kürzt sie nur, wenn sie mehr als 5 Zeilen bräuchte, `full` kürzt nie, `credit-only` druckt nur die freemapper-Zeile (ODbL und dl-de/by-2-0 verlangen dann, dass Sie die Quellen nennen, wenn Sie die Karte weitergeben). ATTRIBUTION.txt nennt immer jede Quelle
help-fetch = Die Daten für das Kartengebiet laden (Orthophoto, Geländemodell, nDSM, ALKIS-Gebäude, OSM)
help-fetch-dir = Projektordner
help-fetch-offline = Nur den Zwischenspeicher verwenden, ohne Netzzugriff
help-arg-cache-dir = Ordner des Zwischenspeichers für geladene Kacheln (Standard: der Cache-Ordner des Systems + `freemapper`)
help-derive = Die Kartenvorlagen (Orthophoto, Schummerung, Hangneigung, nDSM) und die Höhenlinien aus den geladenen Daten berechnen
help-derive-dir = Projektordner
help-run = Laden, ableiten und exportieren in einem Schritt
help-run-dir = Projektordner
help-run-offline = Nur den Zwischenspeicher verwenden, ohne Netzzugriff
help-run-ocad = Zusätzlich einen OCAD-Startpunkt nach `ocad/` schreiben (Format OCAD 12)
help-run-pdf = Zusätzlich `<name>.pdf` schreiben, die druckfertige Karte (nur die Kartenobjekte, keine Kandidaten)
help-run-pdf-candidates = Zusätzlich `<name>-candidates.pdf` schreiben, die Trainingskarte (die Karte und jeder Kandidat)
help-run-pdf-scale-bar = Die Maßstabsleiste auf die PDFs zeichnen (standardmäßig aus; die Maßstabszahl bleibt im Titelblock)
help-run-pdf-sources = Quellenzeilen auf den PDFs: `auto` druckt die ganze Liste und kürzt sie nur, wenn sie mehr als 5 Zeilen bräuchte, `full` kürzt nie, `credit-only` druckt nur die freemapper-Zeile (ODbL und dl-de/by-2-0 verlangen dann, dass Sie die Quellen nennen, wenn Sie die Karte weitergeben). ATTRIBUTION.txt nennt immer jede Quelle
help-select = Das Kartengebiet im Browser wählen: zeichnen oder einen Ort suchen. Beim Speichern wird freemapper.toml geschrieben und der Projektordner bei Bedarf angelegt
help-select-dir = Projektordner (wird beim Speichern angelegt, wenn es ihn nicht gibt)
help-select-kind = Kartenart eines neuen Projekts
help-select-title = Kartentitel eines neuen Projekts
help-select-no-browser = Nur die Adresse der Seite ausgeben; den Browser nicht öffnen
help-select-port = Port auf 127.0.0.1 (Standard: ein freier)
help-select-offline = Auf der Seite ist „Nur Zwischenspeicher“ vorgewählt
help-select-ocad = Auf der Seite ist das OCAD-Startprojekt vorgewählt
help-select-pdf = Auf der Seite ist die druckfertige PDF vorgewählt
help-select-pdf-candidates = Auf der Seite ist die Trainings-PDF (jeder Kandidat) vorgewählt
help-select-pdf-scale-bar = Auf der Seite ist die Maßstabsleiste auf den PDFs vorgewählt
help-select-pdf-sources = Auf der Seite ist diese Wahl der Quellenzeilen auf den PDFs vorgewählt, wie bei `run`
help-mcp = Die Werkzeuge einem KI-Agenten über MCP anbieten (stdin/stdout)
help-update = Eine neuere freemapper-Version aus den GitHub-Releases installieren, nach Rückfrage. Ein aus dem Quellcode gebautes freemapper nennt stattdessen den Weg zum Aktualisieren
help-update-check = Nur prüfen: Exit-Code 10, wenn es eine neuere Version gibt, 0, wenn diese aktuell ist
help-update-yes = Ohne Rückfrage installieren; nötig, wenn die Eingabe kein Terminal ist
help-update-version = Diese Version (x.y.z) statt der neuesten installieren, auch eine ältere
help-update-rollback = Zur Version zurückkehren, die das letzte Update ersetzt hat
help-update-mode = Festlegen, was die tägliche Prüfung tut, wenn sie eine neuere Version findet (Standard: notify). Funktioniert in jedem Build, weil nur die Einstellung geschrieben wird

## Die Zusammenfassung eines Laufs, für den MCP-Server und die Gebietsseite

cli-internal-error = Interner Fehler: { $message }
cli-pdf-sources-invalid = pdf_sources muss auto, full oder credit-only sein, nicht { $value }
cli-summary-wrote-map = Karte { $path } geschrieben
cli-summary-kept-map = Die vorhandene Karte { $path } bleibt (sie kann die Arbeit der Kartiererin oder des Kartierers enthalten)
cli-summary-map-lacks-templates = Die Karte verweist noch nicht auf die Bildvorlagen. Starten Sie die Pipeline mit force, um sie zu ersetzen; die alte bleibt als .omap.bak erhalten.
cli-summary-map-lacks-buildings = Die Karte hat noch keine ALKIS-Gebäude. Starten Sie die Pipeline mit force, um sie zu ergänzen; das ersetzt die Karte, die alte bleibt als .omap.bak erhalten.
cli-summary-map-lacks-canopies = Der Karte fehlen die ALKIS-Überdachungen und Carports (522), die jetzt Kartenobjekte sind. Starten Sie die Pipeline mit force, um sie zu ergänzen; das ersetzt die Karte, die alte bleibt als .omap.bak erhalten.
cli-summary-wrote-ocad = OCAD-Karte { $path } geschrieben ({ $count ->
        [one] 1 Objekt
       *[other] { $count } Objekte
    })
cli-summary-kept-ocad = Die vorhandene OCAD-Karte { $path } bleibt (sie kann die Arbeit der Kartiererin oder des Kartierers enthalten; force ersetzt sie)
cli-summary-ocad-left-out = In der OCAD-Karte fehlen: { $skipped ->
        [one] 1 Objekt
       *[other] { $skipped } Objekte
    } und { $dropped ->
        [one] 1 entarteter Teil
       *[other] { $dropped } entartete Teile
    }
cli-summary-ocad-symbols = Wählen Sie in OCAD Symbol > Farben und Symbole laden von … und nehmen Sie den offiziellen Symbolsatz ISSprOM 2019 oder den Schulsymbolsatz; er ersetzt die Platzhaltersymbole nach ihrer Nummer.
cli-summary-ocad-failed = OCAD-Export fehlgeschlagen: { $error }
cli-summary-pdf-without-credits = PDF ohne Quellenangaben: ODbL und dl-de/by-2-0 verlangen, dass Sie die Quellen nennen, wenn Sie die Karte weitergeben; siehe ATTRIBUTION.txt
cli-summary-wrote-pdf = Karte { $path } geschrieben (ohne Kandidaten)
cli-summary-pdf-failed = PDF-Export fehlgeschlagen: { $error }
cli-summary-wrote-pdf-candidates = Trainingskarte { $path } geschrieben (jeder Kandidat als Kartenobjekt gezeichnet)
cli-summary-pdf-candidates-failed = Export der Trainings-PDF fehlgeschlagen: { $error }

## Der MCP-Server. Tool-, Parameter- und Feldnamen bleiben Englisch.

mcp-error-no-home = Dieses System hat keinen Home-Ordner; geben Sie einen absoluten Pfad an
mcp-error-relative-path = { $path } ist ein relativer Pfad; geben Sie einen absoluten Pfad an oder einen, der mit ~ beginnt
mcp-error-home-folder = { $path } ist der Home-Ordner selbst; nehmen Sie einen Ordner darin, z. B. ~/maps/school
mcp-error-no-project = In { $path } liegt kein freemapper-Projekt; rufen Sie zuerst create_project auf
mcp-error-unknown-kind = Unbekannte Kartenart { $kind }; möglich sind "schoolyard" und "sprint"
mcp-error-reading = { $path } lässt sich nicht lesen: { $error }
mcp-error-one-area-source = Geben Sie genau eines an: osm_id (aus find_place) oder geojson
mcp-error-page-open = Die Seite zur Gebietsauswahl für dieses Projekt ist noch offen; bitten Sie die Nutzerin oder den Nutzer, das Gebiet zu speichern und auf der Seite „Fertig“ zu drücken, und starten Sie dann die Pipeline
mcp-error-no-job = Keinen Job { $job } gefunden; start_pipeline gibt die Job-IDs dieser Sitzung zurück
mcp-error-job-running = { $job } läuft noch für { $path }; es läuft immer nur eine Pipeline, weil sich die Projekte den Zwischenspeicher teilen. Starten Sie diese, wenn project_status finished oder failed meldet
mcp-error-page-start = Die Seite lässt sich nicht starten: { $error }
mcp-created-project = Projekt { $path } für { $area } angelegt ({ $kind ->
        [schoolyard] Schulhof
       *[sprint] Sprint
    }, 1:{ $scale }). Als Nächstes: estimate_download, dann start_pipeline.
mcp-created-project-given-area = Projekt { $path } für das angegebene Gebiet angelegt ({ $kind ->
        [schoolyard] Schulhof
       *[sprint] Sprint
    }, 1:{ $scale }). Als Nächstes: estimate_download, dann start_pipeline.
mcp-page-already-open = Die Seite für diesen Ordner ist schon offen; sie behält die Einstellungen, mit denen sie geöffnet wurde.
mcp-page-next = Bitten Sie die Nutzerin oder den Nutzer, das Gebiet auf der Seite zu wählen und dann „Speichern“ und „Fertig“ zu drücken.
mcp-preview-of = Vorschau von { $layer }
mcp-instructions = freemapper erzeugt Startprojekte für OpenOrienteering Mapper für Orientierungslaufkarten (Schulhof 1:1000, Sprint 1:4000) aus amtlichen Geodaten von Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen und Rheinland-Pfalz sowie aus OSM. Ablauf: find_place (die Schule oder das Gebiet nach Namen suchen; ein Ergebnis mit has_area wählen) → create_project (ein neuer Ordner, absoluter Pfad, mit der osm_id) → estimate_download (der Nutzerin oder dem Nutzer sagen, wie viel geladen wird) → start_pipeline (gibt eine Job-ID zurück; das Laden kann Minuten dauern) → project_status (alle paar Sekunden abfragen, bis der Zustand finished oder failed ist) → list_layers und preview_layer (die Kartenvorlagen zeigen). Danach öffnet die Nutzerin oder der Nutzer die Kartendatei in OpenOrienteering Mapper. Passt kein Ort oder soll das Gebiet gezeichnet werden, öffnet open_area_selector eine Seite im Browser, auf der das Gebiet gezeichnet oder gewählt und gespeichert wird (das legt das Projekt an); weiter mit estimate_download, sobald es gespeichert ist. Pfade müssen absolut sein oder mit ~ beginnen. Es läuft immer nur eine Pipeline.
mcp-tool-create-project = Legt einen Projektordner für ein Kartengebiet an. Geben Sie osm_id (aus find_place) oder eine GeoJSON-Datei an. kind ist schoolyard (1:1000, Standard) oder sprint (1:4000).
mcp-param-create-project-dir = Neuer Projektordner (absoluter Pfad oder mit ~ beginnend).
mcp-param-create-project-geojson = Statt osm_id: eine GeoJSON-Datei mit dem Polygon des Kartengebiets (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (Standard, 1:1000) oder "sprint" (1:4000).
mcp-param-create-project-osm-id = OSM-Objekt des Kartengebiets aus find_place, z. B. "way/992423119".
mcp-param-create-project-part = Mit osm_id: welcher Teil einer Fläche aus mehreren getrennten Teilen verwendet wird (1 ist der größte).
mcp-param-create-project-scale = Maßstabszahl: Schulhof 1000 (Standard); Sprint 4000 (Standard) oder 5000. Andere Werte werden abgelehnt; für 1:3000 in 4000 zeichnen, freemapper-print vergrößert beim Drucken.
mcp-param-create-project-title = Kartentitel für die Randangaben (Standard: der Ordnername).
mcp-tool-estimate-download = Was start_pipeline für ein Projekt laden wird: die Kacheln, jeder Download mit seiner Größe in MB, ob er im Zwischenspeicher liegt und ob er auf den Datenträger passt, die noch zu ladenden MB und unter disk der freie Platz der Datenträger von Zwischenspeicher und Projekt mit Meldungen zu dem, was nicht passt (sagen Sie es der Nutzerin oder dem Nutzer).
mcp-param-estimate-download-dir = Projektordner (absoluter Pfad oder mit ~ beginnend).
mcp-tool-find-place = Sucht eine Schule, einen Park oder einen anderen Ort nach Namen (OpenStreetMap Nominatim). Liefert bis zu 5 Treffer, Flächen zuerst; nur Treffer mit has_area können ein Kartengebiet sein. Eine leere Liste heißt, dass nichts gefunden wurde.
mcp-param-find-place-query = Name der Schule oder des Ortes, z. B. "Clara-Zetkin-Schule Strausberg".
mcp-tool-list-layers = Die Ebenen eines Projekts: die Karte, die Kandidaten und die Rastervorlagen, jede mit ihrer Datei, ob es sie schon gibt und welcher Schritt sie erzeugt.
mcp-param-list-layers-dir = Projektordner (absoluter Pfad oder mit ~ beginnend).
mcp-tool-open-area-selector = Öffnet eine Seite im Browser, auf der die Nutzerin oder der Nutzer das Kartengebiet zeichnet (Rechteck oder Polygon) oder einen Ort wählt, die Kacheln und die Downloadgröße sieht und speichert; das Speichern legt den Projektordner an oder ändert sein Gebiet. Liefert die URL für die Nutzerin oder den Nutzer. Die Seite läuft, bis „Fertig“ gedrückt wird; ein weiterer Aufruf für denselben Ordner liefert dieselbe URL.
mcp-param-open-area-selector-dir = Projektordner (absoluter Pfad oder mit ~ beginnend); wird beim Speichern angelegt.
mcp-param-open-area-selector-kind = Kartenart eines neuen Projekts: "schoolyard" (Standard, 1:1000) oder "sprint" (1:4000).
mcp-param-open-area-selector-ocad = Im Lauf-Block ist das OCAD-Startprojekt vorgewählt. Standard false.
mcp-param-open-area-selector-offline = Im Lauf-Block ist „Nur Zwischenspeicher“ vorgewählt. Standard false.
mcp-param-open-area-selector-open-browser = Die Seite auch im Standardbrowser dieses Rechners öffnen. Standard true.
mcp-param-open-area-selector-pdf = Im Lauf-Block ist die druckfertige PDF vorgewählt. Standard false.
mcp-param-open-area-selector-pdf-candidates = Im Lauf-Block ist die Trainings-PDF (jeder Kandidat) vorgewählt. Standard false.
mcp-param-open-area-selector-pdf-scale-bar = Im Lauf-Block ist die Maßstabsleiste auf den PDFs vorgewählt. Standard false.
mcp-param-open-area-selector-pdf-sources = Quellenzeilen auf den PDFs, mit denen der Block startet: "auto" (Standard), "full" oder "credit-only", wie bei start_pipeline.
mcp-param-open-area-selector-scale = Maßstabszahl eines neuen Projekts: Schulhof 1000 (Standard); Sprint 4000 (Standard) oder 5000. Andere Werte werden abgelehnt.
mcp-param-open-area-selector-title = Kartentitel eines neuen Projekts.
mcp-tool-preview-layer = Eine PNG-Vorschau (höchstens 800 px) einer Rastervorlage: orthophoto, hillshade, slope oder ndsm.
mcp-param-preview-layer-dir = Projektordner (absoluter Pfad oder mit ~ beginnend).
mcp-param-preview-layer-layer = "orthophoto", "hillshade", "slope" oder "ndsm".
mcp-tool-project-status = Zustand eines Pipeline-Jobs: running, finished oder failed, die letzten Fortschrittszeilen, die Warnungen und das Ergebnis oder der Fehler.
mcp-param-project-status-job-id = Job-ID aus start_pipeline, z. B. "job-1".
mcp-tool-start-pipeline = Lädt die Daten, berechnet die Kartenvorlagen und schreibt das Startprojekt (fetch, derive, export). Läuft im Hintergrund und gibt eine job_id zurück; fragen Sie project_status ab. Es läuft immer nur eine Pipeline. Eine vorhandene Karte bleibt, außer force ist true.
mcp-param-start-pipeline-dir = Projektordner (absoluter Pfad oder mit ~ beginnend).
mcp-param-start-pipeline-force = Eine vorhandene Kartendatei ersetzen und die alte als .omap.bak behalten. Standard false.
mcp-param-start-pipeline-ocad = Zusätzlich einen OCAD-Startpunkt in den Ordner ocad/ schreiben (Format OCAD 12, für alle, die in OCAD arbeiten). Standard false.
mcp-param-start-pipeline-offline = Nur den Zwischenspeicher verwenden, ohne Netzzugriff. Standard false.
mcp-param-start-pipeline-pdf = Zusätzlich <name>.pdf schreiben, die druckfertige Karte (nur die Kartenobjekte, keine Kandidaten). Standard false.
mcp-param-start-pipeline-pdf-candidates = Zusätzlich <name>-candidates.pdf schreiben, die Trainingskarte (die Karte und jeder Kandidat als Kartenobjekt; im Gelände prüfen). Standard false.
mcp-param-start-pipeline-pdf-scale-bar = Die Maßstabsleiste auf die PDFs zeichnen. Standard false (deutsche Karten haben meist keine; die Maßstabszahl bleibt im Titelblock).
mcp-param-start-pipeline-pdf-sources = Quellenzeilen auf den PDFs: "auto" (Standard; die ganze Liste, gekürzt nur, wenn sie mehr als 5 Zeilen braucht), "full" (nie gekürzt) oder "credit-only" (nur die freemapper-Zeile; ODbL und dl-de/by-2-0 verlangen dann, die Quellen zu nennen, wenn die Karte weitergegeben wird). ATTRIBUTION.txt nennt immer jede Quelle.
