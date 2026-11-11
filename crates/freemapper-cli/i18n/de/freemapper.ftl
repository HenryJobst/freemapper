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
