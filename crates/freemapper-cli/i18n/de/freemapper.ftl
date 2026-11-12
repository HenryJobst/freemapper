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

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; keine Fläche, kann kein Kartengebiet sein): { $place }
cli-find-hint = Projekt anlegen mit: freemapper init <ordner> --osm { $id }
cli-find-no-area = Keiner dieser Orte hat eine Fläche, die ein Kartengebiet sein kann. Zeichnen Sie das Gebiet mit: freemapper select <ordner>
cli-find-nothing = Zu „{ $query }“ wurde kein Ort gefunden. Versuchen Sie einen anderen Namen oder zeichnen Sie das Gebiet mit: freemapper select <ordner>

## Aufzählungen im Satz

# Eine Aufzählung im Satz: das letzte Glied nach den anderen, die Kommas verbinden.
cli-list-and = { $first } und { $last }

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
cli-save-running = Für dieses Projekt läuft gerade ein Lauf. Speichern Sie noch einmal, wenn er fertig ist.
cli-save-unknown-kind = Unbekannte Kartenart „{ $kind }“. Möglich sind schoolyard und sprint.
cli-save-scale-too-large = Der Maßstab { $scale } ist zu groß.
cli-save-bad-content = Die Inhaltseinstellungen sind ungültig. Es wurde nichts gespeichert.
cli-map-image-none = Erzeugen Sie zuerst die Karte.
cli-map-image-gone = Diese Vorschau gibt es nicht mehr.
cli-map-image-running = Für dieses Projekt läuft gerade ein Lauf. Die Vorschau lässt sich berechnen, sobald er fertig ist.
cli-map-image-stale = Das gespeicherte Projekt passt nicht mehr zur erzeugten Karte, etwa weil sich Gebiet, Kartenart oder Maßstab geändert haben. Starten Sie zuerst einen neuen Lauf.
cli-map-image-bad-content = Die Inhaltseinstellungen sind ungültig.
cli-map-image-bad-request = Die Anfrage für die Vorschau ist ungültig. Erwartet wird ein JSON-Objekt, gegebenenfalls mit den Inhaltseinstellungen.

## Die Gebietsauswahl-Seite (`web/`, ADR 0096). Die Seite holt jede `page-*`- und `suite-*`-Meldung
## über `api/texts`; ein `{ $name }` bleibt im Text stehen, die Seite setzt den Wert ein. Keine
## Selektoren: Eine Anzahl hat zwei Meldungen, `<id>-one` und `<id>-other`. Die Seite duzt.
## AGENTS.md, das Handbuch und scripts/browser-smoke.md zitieren diese Wörter.

page-title = freemapper – Gebietsauswahl
page-heading = Kartengebiet wählen
page-update-available = freemapper { $version } ist verfügbar. Aktualisieren mit
page-update-notes = Versionshinweise
page-folder = Projektordner
page-pick = Ort suchen
page-query-placeholder = z. B. Clara-Zetkin-Schule Strausberg
page-search-button = Suchen
page-searching = Suche …
page-no-results = Nichts gefunden.
page-point-only = nur ein Punkt, keine Fläche
page-draw = Selbst zeichnen
page-rectangle = Rechteck
page-polygon = Polygon
page-finish-polygon = Polygon schließen
page-clear = Löschen
page-hint-rectangle = Klicke zwei gegenüberliegende Ecken in die Karte.
page-hint-polygon = Klicke die Ecken nacheinander. Zum Schließen die erste Ecke anklicken, doppelklicken oder „Polygon schließen“ wählen. Esc bricht ab, Rücktaste nimmt die letzte Ecke zurück.
page-hint-idle = Wähle einen Ort aus der Suche oder zeichne das Gebiet selbst.
page-summary = Gebiet
page-source-osm = Aus OpenStreetMap: { $name }
page-source-drawn = Selbst gezeichnet
page-size = Größe mit Puffer: { $width } × { $height } m
page-tiles-one = { $n } Kachel: { $names }
page-tiles-other = { $n } Kacheln: { $names }
page-download = Download: etwa { $mb } MB
page-download-none = Download: alles schon im Zwischenspeicher
page-download-details = Was geladen wird
page-cached = schon geladen
page-disk = Speicherplatz: { $volumes }
page-disk-volume = { $free } frei, gebraucht etwa { $needed }
page-cache-volume = Zwischenspeicher
page-project-volume = Projektordner
page-no-room = passt nicht auf den Datenträger
page-no-room-skip = Nicht alles passt auf den Datenträger (freemapper lässt { $reserve } frei). Das Laden lässt die markierten Teile aus. Schaffe vorher Platz, wenn du sie brauchst:
page-no-room-stop = Zu wenig Speicherplatz (freemapper lässt { $reserve } frei). Ohne die markierten Teile entsteht keine Karte, deshalb bricht das Laden ab, bevor es etwas lädt. Schaffe vorher Platz:
page-disk-unknown = Den freien Speicherplatz kann freemapper nicht lesen ({ $error }). Es lädt dann ohne Prüfung.
page-too-large = Das Gebiet ist zu groß: höchstens { $max } m Seitenlänge (mit Puffer). Bitte kleiner wählen.
page-outside = Das Gebiet liegt außerhalb der unterstützten Bundesländer ({ $states }); für andere Regionen hat freemapper noch keine Datenquellen.
page-outside-unknown = Das Gebiet liegt außerhalb der Regionen, die freemapper unterstützt; dort hat es noch keine Datenquellen.
page-adapters-failed = Die Luftbilder lassen sich nicht auflisten ({ $detail }). Die Karte zeigt nur OpenStreetMap; alles andere funktioniert.
page-settings = Karte
page-kind = Kartenart
page-schoolyard = Schulhofkarte (1:1000)
page-sprint = Sprintkarte (1:4000)
page-scale = Maßstab 1:
page-map-title = Titel
page-settings-fixed = Das Projekt gibt es schon; Kartenart, Maßstab und Titel stehen in freemapper.toml.
page-save = Speichern
page-saving = Speichere …
page-saved = Gespeichert in freemapper.toml.
page-zone-changed = Gespeichert. Das Gebiet liegt jetzt in einer anderen UTM-Zone; freemapper ruft seine Daten beim nächsten Lauf neu ab.
page-provider-changed = Gespeichert. Für das neue Gebiet liefert ein anderer Datenanbieter die Höhen- und Luftbilddaten; freemapper ruft seine Daten beim nächsten Lauf neu ab.
page-save-stale = freemapper.toml wurde in einem anderen Fenster oder von einem anderen Programm geändert, seit diese Seite sie gelesen hat. Es wurde nichts gespeichert; die Seite kennt jetzt den gespeicherten Stand. Speichere noch einmal, um ihn durch das hier gezeigte Gebiet zu ersetzen.
page-has-map = Die vorhandene Karte (.omap) bleibt, wie sie ist; freemapper erzeugt neue Vorlagen und Kandidaten für das neue Gebiet. Mit der Option „Vorhandene Karte ersetzen“ unten (oder „freemapper run --force“) entsteht auch die Karte neu, die alte bleibt als .omap.bak.
page-done = Fertig
page-closed = Fertig. Du kannst dieses Fenster schließen.
page-next = Weiter: freemapper run "{ $dir }"
page-made-files = Erzeugt: { $files }. Ordner: { $dir }
page-back-to-agent = Sag deinem KI-Assistenten, dass das Gebiet gespeichert ist.
page-nothing-saved = Es wurde nichts gespeichert.
page-failed = Fehler: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Luftbild { $state } ({ $agency })

## Der Block „Karte erzeugen“ der Gebietsauswahl-Seite (ADR 0090)

page-content-heading = Was auf die Karte kommt
page-content-help = Das Profil legt fest, was direkt auf die Karte kommt und was als Kandidat zur Prüfung bleibt. Eine Wahl für eine Gruppe geht dem Profil vor. Sie wird mit dem Gebiet gespeichert.
page-content-profile-legend = Profil
page-content-profile-safe = Sicher
page-content-profile-safe-hint = Jedes Objekt bleibt dort, wo die Erzeugung es hinlegt: Gebäude und Flächen des Katasters kommen auf die Karte; Mauern, Zäune, Bäume und vieles aus OpenStreetMap (Teiche, Wege ohne bekannte Oberfläche) bleiben Kandidaten.
page-content-profile-balanced = Ausgewogen
page-content-profile-balanced-hint = Objekte des Katasters, der Register und der Straßenbefahrung kommen direkt auf die Karte, außer Ausstattung, Höhenlinien und Sperrgebiete; bei Mauern, Zäunen, Hecken, Gewässern, Bäumen, Wegen und Gleisen auch die von OpenStreetMap.
page-content-profile-full = Voll
page-content-profile-full-hint = Alles kommt auf die Karte, wie in der Trainingskarte; nur verborgene Kandidaten bleiben draußen.
page-content-groups-legend = Je Gruppe
page-content-groups-help = „Wie im Profil“ folgt dem Profil. Was das für eine Gruppe bedeutet, hängt davon ab, woher das einzelne Objekt stammt.
page-content-place-profile = Wie im Profil
page-content-place-map = Karte
page-content-place-candidates = Kandidat
page-content-place-off = Aus
page-content-group-walls = Mauern
page-content-group-fences = Zäune
page-content-group-hedges = Hecken
page-content-group-slopes = Böschungen und Felswände
page-content-group-stairs = Treppen
page-content-group-trees = Bäume und Baumgruppen
page-content-group-water-features = Gewässer, Brunnen und Quellen
page-content-group-monuments = Türme, Denkmäler und Leitungen
page-content-group-furniture = Ausstattung und Laternen
page-content-group-passages = Durchgänge
page-content-group-contours = Höhenlinien
page-content-group-out-of-bounds = Sperrgebiete
page-content-group-paths = Wege und Gleise
page-content-group-streets = Straßen und Plätze
page-run-heading = Karte erzeugen
page-preview-heading = Kartenvorschau
page-preview-stored = Stand der zuletzt erzeugten Karte.
page-preview-live = Mit den Einstellungen auf dieser Seite, noch nicht gespeichert.
page-preview-live-saved = Mit den gespeicherten Einstellungen, noch in keinem Lauf erzeugt.
page-preview-computing = Wird berechnet …
page-preview-none = Noch keine Karte erzeugt.
page-preview-later = Zu dieser Karte gibt es noch keine Vorschau. Sie entsteht mit dem nächsten Lauf, der die Karte schreibt.
page-preview-stale = Gebiet, Kartenart oder Maßstab haben sich seit dem letzten Lauf geändert. Das Bild zeigt noch die zuletzt erzeugte Karte; für eine neue Vorschau braucht es einen neuen Lauf.
page-preview-open = In voller Größe öffnen
page-preview-alt = Vorschau der Kartenobjekte
page-opt-ocad = OCAD-Startprojekt schreiben
page-help-ocad = Für alle, die in OCAD weiterarbeiten.
page-opt-pdf = Druckfertige PDF
page-help-pdf = Nur die Kartenobjekte, ohne Kandidaten.
page-opt-pdf-candidates = Trainings-PDF mit allen Kandidaten
page-opt-scale-bar = Maßstabsleiste in die PDFs
page-help-scale-bar = Deutsche Karten haben meist keine.
page-opt-sources = Quellenangabe in den PDFs
page-sources-auto = automatisch
page-sources-full = vollständig
page-sources-credit-only = nur freemapper
page-warn-credit-only = Die PDFs nennen dann nur freemapper. Die Lizenzen der Daten (ODbL, dl-de/by-2-0) verlangen die Quellenangabe, wenn du die Karte weitergibst; dann nennst du die Quellen selbst. ATTRIBUTION.txt listet sie alle.
page-need-pdf = Maßstabsleiste und Quellenangabe gelten nur, wenn eine PDF entsteht.
page-opt-offline = Nur Zwischenspeicher, kein Netz
page-opt-force = Vorhandene Karte ersetzen
page-help-force = Die alte Datei bleibt als .omap.bak.
page-run-start = Karte erzeugen
page-run-again = Nochmal erzeugen
page-run-cancel = Abbrechen
page-run-cancelling = Wird abgebrochen …
page-warnings-heading = Warnungen
page-where-files = Die Dateien liegen im Projektordner:
page-open-folder = Ordner öffnen
page-log-label = Protokoll des Laufs
page-force-title = Vorhandene Karte ersetzen?
page-force-text = Die vorhandene Karte wird ersetzt. Deine Änderungen darin gehen nicht verloren: Die alte Datei bleibt als .omap.bak im Projektordner. Fortfahren?
page-force-cancel = Abbrechen
page-force-confirm = Ersetzen
page-step-fetch = Daten laden
page-step-derive = Vorlagen berechnen
page-step-map = Karte schreiben
page-step-ocad = OCAD-Projekt schreiben
page-step-pdf = PDF schreiben
page-step-pdf-candidates = Trainings-PDF schreiben
page-step-line = Schritt { $index } von { $total }: { $label }
page-starting = Der Lauf startet …
page-file-kind-map = Karte
page-file-kind-template = Vorlage
page-file-kind-ocad = OCAD-Karte
page-file-kind-ocad-candidates = OCAD-Kandidaten
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = Trainings-PDF
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Fertig. Die Dateien sind geschrieben.
page-result-failed = Der Lauf ist fehlgeschlagen: { $error }
page-result-failed-kept = Was schon geschrieben ist, bleibt im Projektordner; ein neuer Lauf macht dort weiter.
page-result-cancelled = Abgebrochen. Was schon geladen ist, bleibt im Zwischenspeicher; ein neuer Lauf macht dort weiter.
page-result-title-done = Karte erzeugt
page-result-title-failed = Lauf fehlgeschlagen
page-result-title-cancelled = Lauf abgebrochen
page-kept-map = Die vorhandene Karte blieb unverändert. Mit „Vorhandene Karte ersetzen“ entsteht sie neu, die alte bleibt als .omap.bak.
page-no-files = Es sind keine Dateien entstanden.
page-opened-folder = Der Ordner ist geöffnet.
page-open-failed = { $message } Ordner: { $dir }
page-why-no-area = Wähle zuerst ein Gebiet.
page-why-checking = Die Vorschau wird berechnet …
page-why-too-large = Das Gebiet ist zu groß.
page-why-outside = Das Gebiet liegt außerhalb der unterstützten Bundesländer.
page-why-disk = Es ist zu wenig Speicherplatz frei (siehe oben).
page-why-running = Ein Lauf ist im Gang.
page-err-scale-bar-no-pdf = Die Maßstabsleiste braucht eine PDF.
page-err-sources-no-pdf = Die Quellenangabe braucht eine PDF.
page-save-failed = Das Gebiet ist nicht gespeichert; es gibt noch keinen Lauf.
page-poll-lost = Keine Antwort vom Server. freemapper versucht es weiter.
page-run-unknown = Der Lauf ist dem Server nicht mehr bekannt.

## Die gemeinsamen Module der Seiten der Suite (core ADR 0030)

suite-banner-dismiss = Ausblenden

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
update-too-old = freemapper { $new } lässt sich nicht durch die Selbstaktualisierung von { $old } installieren. Installieren Sie { $needed } oder neuer von Hand, siehe { $notes }
update-busy = Im Programmordner läuft schon eine Aktualisierung oder ein Zurücksetzen (Sperrdatei { $path }). Warten Sie, bis der Vorgang beendet ist, und versuchen Sie es dann erneut.
update-bad-signature = Die neue Version ließ sich nicht verifizieren ({ $error }). Es wurde nichts installiert.
update-check-bad-signature = Die neueste freemapper-Version ließ sich nicht verifizieren ({ $error }). Es wurde nichts installiert.
update-no-manifest = Die neueste Version kann sich nicht selbst installieren (ihr fehlt das Aktualisierungsmanifest). Laden Sie sie von { $page } herunter.
update-no-manifest-version = Die Version { $version } gibt es nicht, oder sie kann sich nicht selbst installieren (ihr fehlt das Aktualisierungsmanifest); siehe { $page }
update-no-asset = Die neue Version enthält kein Programm für dieses System ({ $platform }). Es wurde nichts installiert.
update-no-previous = Es gibt keine vorherige Version, zu der zurückgekehrt werden kann ({ $path } fehlt).
update-failed = Die Aktualisierung ist fehlgeschlagen: { $error }
update-mode-set = Aktualisierungsmodus { $mode } in { $path } gespeichert.
update-mode-env = FREEMAPPER_UPDATE={ $value } hat Vorrang vor dieser Einstellung, solange die Variable gesetzt ist.
update-mode-failed = { $path } ließ sich nicht schreiben: { $error }
update-notice = freemapper { $new } ist verfügbar (Sie haben { $old }). Aktualisieren mit: freemapper update
update-applies-next-start = Die neue Version gilt ab dem nächsten Start von freemapper.
update-install-failed = freemapper { $new } wurde nicht installiert: { $error }
update-no-config-dir = Dieses System hat keinen Ordner für Einstellungen; setzen Sie FREEMAPPER_CONFIG_DIR.
update-mcp-note = Hinweis für die Nutzerin oder den Nutzer: freemapper { $new } ist verfügbar (dies ist { $old }). Fragen Sie nach, bevor Sie install_update aufrufen.
update-mcp-needs-confirm = Es wurde nichts installiert; fragen Sie zuerst die Nutzerin oder den Nutzer und rufen Sie install_update nach der Zustimmung mit confirm: true auf
update-mcp-note-manual = Hinweis für die Nutzerin oder den Nutzer: freemapper { $new } ist verfügbar (dies ist { $old }), lässt sich aber nicht mit install_update installieren. { $needed } oder neuer muss von Hand installiert werden, siehe { $notes }
update-mcp-note-bad-signature = Hinweis für die Nutzerin oder den Nutzer: Die neueste freemapper-Version ließ sich nicht verifizieren ({ $error }). Es wurde nichts installiert. Sagen Sie es ihr oder ihm und installieren Sie sie nicht auf anderem Weg.
update-mcp-restart-now = Starten Sie den freemapper-MCP-Server jetzt neu, denn er läuft noch mit der alten Version.

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
error-project-unknown-adapter = Die Projektdatei nennt den Adapter „{ $adapter }“, den dieses freemapper nicht hat; eine neuere Version kann ihn haben
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
error-project-outside-coverage = Das Kartengebiet liegt außerhalb der unterstützten Bundesländer ({ $states }); bisher hat freemapper nur für diese Datenquellen

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
help-find = Die OpenStreetMap-Orte zu einem Namen auflisten, Flächen zuerst, mit der Kennung für init --osm
help-find-query = Name des Ortes, z. B. "Clara-Zetkin-Schule Strausberg"
help-find-json = Die Treffer als JSON ausgeben, mit denselben Feldern wie das MCP-Werkzeug find_place
help-mcp = Die Werkzeuge einem KI-Agenten über MCP anbieten (stdin/stdout)
help-update = Eine neuere freemapper-Version aus den GitHub-Releases installieren, nach Rückfrage. Ein aus dem Quellcode gebautes freemapper nennt stattdessen den Weg zum Aktualisieren
help-update-check = Nur prüfen: Exit-Code 10, wenn es eine neuere Version gibt, 0, wenn diese aktuell ist
help-update-yes = Ohne Rückfrage installieren; nötig, wenn die Eingabe kein Terminal ist
help-update-version = Diese Version (x.y.z) statt der neuesten installieren, auch eine ältere
help-update-rollback = Zur Version zurückkehren, die die letzte Aktualisierung ersetzt hat
help-update-mode = Festlegen, was die tägliche Prüfung tut, wenn sie eine neuere Version findet (Standard: notify). Funktioniert in jedem Build, weil nur die Einstellung geschrieben wird

## Die Zusammenfassung eines Laufs, für den MCP-Server und die Gebietsseite

cli-internal-error = Interner Fehler: { $message }
cli-pdf-sources-invalid = pdf_sources muss auto, full oder credit-only sein, nicht { $value }
cli-estimate-disk-unreadable = Der freie Speicherplatz von { $path } lässt sich nicht lesen ({ $message })
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
mcp-instructions = freemapper erzeugt Startprojekte für OpenOrienteering Mapper für Orientierungslaufkarten (Schulhof 1:1000, Sprint 1:4000) aus amtlichen Geodaten von { $states } sowie aus OSM. Ablauf: find_place (die Schule oder das Gebiet nach Namen suchen; ein Ergebnis mit has_area wählen) → create_project (ein neuer Ordner, absoluter Pfad, mit der osm_id) → estimate_download (der Nutzerin oder dem Nutzer sagen, wie viel geladen wird) → start_pipeline (gibt eine Job-ID zurück; das Laden kann Minuten dauern) → project_status (alle paar Sekunden abfragen, bis der Zustand finished oder failed ist) → list_layers und preview_layer (die Kartenvorlagen zeigen). Danach öffnet die Nutzerin oder der Nutzer die Kartendatei in OpenOrienteering Mapper. Passt kein Ort oder soll das Gebiet gezeichnet werden, öffnet open_area_selector eine Seite im Browser, auf der das Gebiet gezeichnet oder gewählt und gespeichert wird (das legt das Projekt an); weiter mit estimate_download, sobald es gespeichert ist. Pfade müssen absolut sein oder mit ~ beginnen. Es läuft immer nur eine Pipeline. check_update sagt, ob es eine neuere Version von freemapper gibt; install_update installiert sie, nur mit confirm: true nach Zustimmung der Nutzerin oder des Nutzers, und danach startet der Assistent den freemapper-Server neu.
mcp-tool-check-update = Ob es eine neuere Version von freemapper gibt: ihre Versionsnummer, ihr Erscheinungsdatum, die Versionshinweise und die Downloadgröße, oder dass diese Version aktuell ist. Installiert nichts.
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
mcp-tool-install-update = Installiert die neueste Version von freemapper über dieses Programm. Rufen Sie es erst auf, wenn die Nutzerin oder der Nutzer zugestimmt hat, und mit confirm true. Die neue Version gilt, sobald der Assistent den freemapper-Server neu startet.
mcp-param-install-update-confirm = true, sobald die Nutzerin oder der Nutzer der Installation der neueren Version zugestimmt hat; ohne das wird nichts installiert.
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

cli-content-saved = Inhaltseinstellung in freemapper.toml gespeichert: { $summary }
cli-content-invalid = Die Inhaltseinstellung ist nicht lesbar: { $detail }
help-arg-content = Was direkt auf die Karte kommt: `safe` lässt jedes Objekt dort, wo die Erzeugung es hinlegt, `balanced` (Standard) legt die Objekte des Katasters und der Register auf die Karte, `full` legt alles auf die Karte. Die Wahl wird in freemapper.toml gespeichert
help-arg-content-set = Eine Objektgruppe nach `map`, `candidates` oder `off` schicken, als `GRUPPE=ORT`, z. B. `walls=map`; mehrfach möglich. Gruppen: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, out-of-bounds, paths, streets. Wird in freemapper.toml gespeichert
mcp-param-create-project-content = Was direkt auf die Karte kommt: "safe", "balanced" (Standard) oder "full". Wird im Projekt gespeichert. balanced legt die Objekte des Katasters und der Register auf die Karte, full legt alles darauf.
mcp-param-create-project-content-set = Gruppen, die anders als das Profil behandelt werden, als "gruppe=ort" mit dem Ort map, candidates oder off, z. B. "walls=map". Gruppen: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, out-of-bounds, paths, streets.
mcp-param-start-pipeline-content = Vor dem Lauf ändern, was direkt auf die Karte kommt: "safe", "balanced" oder "full". Wird im Projekt gespeichert. Standard: die Einstellung des Projekts, sonst balanced.
mcp-param-start-pipeline-content-set = Gruppen, die anders als das Profil behandelt werden, als "gruppe=ort" mit dem Ort map, candidates oder off, z. B. "fences=candidates". Wird im Projekt gespeichert.
mcp-error-content = Die Inhaltseinstellung ist nicht lesbar: { $detail }
