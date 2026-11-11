# Fortschrittszeilen und Warnungen der Engine (ADR 0092, Stufe 3). Englisch ist die Referenz.

note-warning = Warnung: { $text }

warning-other-zone = Das Kartengebiet liegt in UTM-Zone { $zone } (EPSG:{ $epsg }); freemapper lädt alles in dieser Zone neu
warning-other-state = Das Kartengebiet liegt jetzt in den Daten eines anderen Bundeslands; freemapper lädt alles neu
warning-beyond-coverage = Ein Teil des Kartengebiets liegt außerhalb der unterstützten Bundesländer: im Ausland, auf See oder in einem Bundesland, das freemapper noch nicht unterstützt, etwa Bremen oder Thüringen. Die Bundesländer haben dort keine Daten; das Orthophoto kann dort weiß sein, und ALKIS liefert weder Gebäude noch Nutzung
# Eine Warnung, deren Text ein Fehler ist.
warning-error = { $error }

## Die Projektdatei

warning-scale-not-allowed = freemapper.toml: Der Maßstab 1:{ $scale } gehört nicht zu den Maßstäben für { $kind ->
        [schoolyard] Schulhofkarten
       *[sprint] Sprintkarten
    } ({ $allowed }); freemapper-print wird die Karte ablehnen
warning-layer-entry-ignored = freemapper.toml: Eintrag { $key } der { $what ->
        [tree] Baumebenen
        [NAS] NAS-Ebenen
       *[Berlin] Berlin-Ebenen
    } ignoriert: { $reason ->
        [name] kein einfacher Dateiname
       *[unknown] keine Ebene, die freemapper kennt
    }

## Plattenplatz

warning-disk-space-unreadable = Der freie Plattenplatz von { $path } lässt sich nicht lesen ({ $error }); freemapper prüft ihn deshalb nicht

## Kacheln

warning-tile-changed = { $product }-Kachel { $tile } hat sich beim Anbieter geändert ({ $old } → { $new })
warning-bdom-tile-unavailable = Oberflächenmodell (bDOM) der Kachel { $tile } nicht verfügbar ({ $error }); dort keine nDSM-Vorlage
warning-bdom-unavailable = Oberflächenmodell (bDOM) nicht verfügbar ({ $error }); keine nDSM-Vorlage für { $tiles }
progress-ndsm-from-bdom = nDSM der Kachel { $tile } aus dem Oberflächenmodell (bDOM)

## Höhenmodelle und Orthophotos

word-model = { $kind ->
        [terrain] Geländemodell
        [surface] Oberflächenmodell
       *[orthophoto] Orthophoto
    } ({ $name })
progress-model-download = { $model } wird geladen
progress-model-read = { $model } wird gelesen
warning-model-unavailable = { $model } nicht verfügbar ({ $error }); keine nDSM-Vorlage
warning-model-unavailable-kept = { $model } nicht verfügbar ({ $error }); das am { $fetched } geladene Oberflächenmodell bleibt
warning-model-tile-unavailable = { $model } der Kachel { $tile } nicht verfügbar ({ $error }); dort keine nDSM-Vorlage
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] Im Kartengebiet
        [hamburg] In Hamburg
        [hesse] In Hessen
        [north-rhine-westphalia] In Nordrhein-Westfalen
        [schleswig-holstein] In Schleswig-Holstein
        [mecklenburg-vorpommern] In Mecklenburg-Vorpommern
        [lower-saxony] In Niedersachsen
       *[other] In { $state }
    } { $count ->
        [one] fehlt die Kachel
       *[other] fehlen die Kacheln
    } { $names }; die Vorlagen bleiben dort leer
warning-no-surface-stored = Kein Oberflächenmodell gespeichert; rufen Sie `freemapper fetch` mit Netzverbindung auf, um die nDSM-Vorlage zu erhalten
warning-surface-older-than-orthophoto = Das Oberflächenmodell (bDOM { $surface }) ist älter als das Orthophoto ({ $photo }); neue Gebäude können im nDSM fehlen
progress-surface-kept = freemapper behält das am { $fetched } geladene Oberflächenmodell für die nDSM-Vorlage
progress-surface-tiles-kept = freemapper behält das zwischengespeicherte Oberflächenmodell der { $count ->
        [one] Kachel
       *[other] Kacheln
    } { $names } für die nDSM-Vorlage
progress-orthophoto-unchanged = Orthophoto unverändert
progress-orthophoto-unchanged-dated = Orthophoto unverändert ({ $label } { $date })
progress-orthophoto-download = Orthophoto wird vom { $service } von { $provider } geladen
progress-orthophoto-in-leaf = Das Orthophoto wurde am { $date } geflogen, die Bäume waren belaubt; ihre Kronen verdecken, was darunter liegt
progress-orthophoto-bare = Das Orthophoto wurde am { $date } geflogen, die Bäume waren unbelaubt
warning-lgln-orthophoto-kept = Orthophoto (LGLN WMS): { $reason }; das gespeicherte Orthophoto und sein Flugdatum { $date } bleiben
warning-lgln-orthophoto-undated = Orthophoto (LGLN WMS): kein Flugdatum, { $reason }
word-stac-no-image = die STAC-Suche nennt in der Mitte des Kartengebiets kein Bild
word-stac-no-answer = die STAC-Suche gab keine brauchbare Antwort ({ $error })
warning-downloadcenter-no-file = Das Downloadcenter hat keine Datei für { $places }
warning-downloadcenter-no-file-empty = Das Downloadcenter hat keine Datei für { $places }; { $count ->
        [one] die Kachel { $names } blieb
       *[other] die Kacheln { $names } blieben
    } leer
progress-k-five-unchanged = K5-Vorlage unverändert
progress-k-five-download = Karte K5 (1:5000) wird vom WMS des Geoportals Berlin geladen
warning-k-five-unavailable = Die Kartenvorlage K5 ist nicht verfügbar ({ $detail }); übersprungen

## Die Grenze des Bundeslands

progress-beyond-border = Das Kartengebiet reicht über die Grenze des Bundeslands hinaus; jenseits davon sind die Vorlagen dort leer, wo das Orthophoto und die Höhenmodelle von { $provider } enden. ALKIS und OSM kommen aus beiden Bundesländern
progress-beyond-border-no-data = Das Kartengebiet reicht über die Grenze des Bundeslands hinaus; für { $tiles } gibt es keine Daten von { $provider }, die Vorlagen bleiben dort leer

## OSM

warning-osm-extract-missing-kept = Kein OSM-Auszug im Zwischenspeicher; die gespeicherten OSM-Kandidaten vom { $date } bleiben
progress-osm-extract-read = OSM-Auszug { $region } vom { $date } wird gelesen
progress-osm-candidates = { $count ->
        [one] 1 OSM-Objekt als Kandidat
       *[other] { $count } OSM-Objekte als Kandidaten
    }

## ALKIS

warning-alkis-coverage-unknown = Unbekannt, welches Gebiet die ALKIS-Gebäude abdecken; rufen Sie `freemapper fetch` mit Netzverbindung auf, damit es vermerkt wird
progress-alkis-buildings-download = ALKIS-Gebäude werden geladen
progress-alkis-buildings = { $count ->
        [one] 1 ALKIS-Gebäude
       *[other] { $count } ALKIS-Gebäude
    }
warning-alkis-no-buildings = Im Kartengebiet wurden keine ALKIS-Gebäude gefunden
progress-alkis-skipped = { $count ->
        [one] 1 ALKIS-Objekt
       *[other] { $count } ALKIS-Objekte
    } übersprungen: { $list }
warning-alkis-buildings-changed = Die ALKIS-Gebäude haben sich beim Anbieter geändert ({ $changes })
progress-alkis-no-land-use-stored = Keine ALKIS-Nutzung gespeichert; rufen Sie `freemapper fetch` mit Netzverbindung auf, um Kandidaten für verbotene Gebiete zu erhalten
progress-alkis-land-use-download = ALKIS-Nutzung wird geladen
progress-alkis-land-use-parts = { $count ->
        [one] 1 ALKIS-Nutzungsfläche
       *[other] { $count } ALKIS-Nutzungsflächen
    }
warning-alkis-land-use-kept = ALKIS-Nutzung nicht verfügbar ({ $error }); die gespeicherte Nutzung bleibt
warning-alkis-land-use-removed = ALKIS-Nutzung nicht verfügbar ({ $error }); die gespeicherte Nutzung deckt das Kartengebiet nicht mehr ab, freemapper hat sie deshalb entfernt
warning-alkis-land-use-none = ALKIS-Nutzung nicht verfügbar ({ $error }); keine Kandidaten für verbotene Gebiete

## Die Berliner Ebenen

progress-berlin-none-stored = Keine Berliner ALKIS-Bauwerke gespeichert; rufen Sie `freemapper fetch` mit Netzverbindung auf, um Treppen, Mauern, Bäume und Überdachungen zu erhalten
progress-berlin-removing = Die gespeicherten Berliner Ebenen werden entfernt: Das Kartengebiet liegt nicht mehr bei Berlin
progress-berlin-download = Berliner ALKIS-Bauwerke, Baumkataster, Straßenbefahrung und Punktebenen werden geladen
progress-berlin-features = { $count ->
        [one] 1 Objekt
       *[other] { $count } Objekte
    } in der Berliner Ebene { $file }
warning-berlin-layer-kept = Berliner Ebene { $file } nicht verfügbar ({ $detail }); die gespeicherte Ebene bleibt
warning-berlin-layer-skipped = Berliner Ebene { $file } nicht verfügbar ({ $detail }); Ebene übersprungen
warning-berlin-buildings-missing = Für dieses Gebiet in Berlin sind keine ALKIS-Gebäude gespeichert; rufen Sie `freemapper fetch` auf, um sie vom Geoportal Berlin zu laden

## Die vollständigen ALKIS-Ebenen (NAS)

progress-nas-none-stored = Keine vollständigen ALKIS-Bauwerke gespeichert; rufen Sie `freemapper fetch` mit Netzverbindung auf, um Mauern, Zäune, Hecken und Brücken zu erhalten
progress-nas-removing = Die gespeicherten vollständigen ALKIS-Ebenen werden entfernt: Das Kartengebiet liegt nicht mehr in einem Bundesland mit NAS-Dienst
warning-nas-folder-outside = Vollständige ALKIS-Ebenen übersprungen ({ $folder } führt aus dem Projektordner hinaus)
progress-nas-download = Die vollständigen ALKIS-Bauwerke (NAS) werden geladen
progress-nas-features = { $count ->
        [one] 1 Objekt
       *[other] { $count } Objekte
    } in der ALKIS-NAS-Ebene { $file }
progress-nas-skipped = { $count ->
        [one] 1 Objekt
       *[other] { $count } Objekte
    } der ALKIS-NAS-Ebene { $file } übersprungen: { $list }
warning-nas-layer-kept = ALKIS-NAS-Ebene { $file } nicht verfügbar ({ $detail }); die gespeicherte Ebene bleibt
warning-nas-layer-skipped = ALKIS-NAS-Ebene { $file } nicht verfügbar ({ $detail }); Ebene übersprungen

## Baumkataster

progress-trees-none-stored = Kein städtisches Baumkataster gespeichert; rufen Sie `freemapper fetch` mit Netzverbindung auf, um die Einzelbäume zu erhalten
progress-trees-removing = Die gespeicherten Baumkataster werden entfernt: Das Kartengebiet liegt nicht mehr in einer Stadt mit Baumkataster
warning-trees-folder-outside = Städtische Baumkataster übersprungen ({ $folder } führt aus dem Projektordner hinaus)
progress-trees-download = Die städtischen Baumkataster werden geladen
progress-trees-count = { $count ->
        [one] 1 Baum
       *[other] { $count } Bäume
    } aus { $id }
progress-trees-skipped = { $count ->
        [one] 1 Baum
       *[other] { $count } Bäume
    } aus { $id } übersprungen: { $list }
warning-trees-kept = Baumkataster { $id } nicht verfügbar ({ $detail }); das gespeicherte bleibt
warning-trees-skipped = Baumkataster { $id } nicht verfügbar ({ $detail }); Bäume übersprungen

## Vorlagen ableiten

progress-writing = { $path } wird geschrieben
progress-no-ndsm-recorded = Kein nDSM vermerkt; rufen Sie `freemapper fetch` auf, um die nDSM-Vorlage zu erhalten
warning-ndsm-tile-invalid = Ungültige nDSM-Kachel { $tile } in freemapper.toml; übersprungen
warning-ndsm-not-cached = nDSM { $tile } liegt nicht im Zwischenspeicher; rufen Sie `freemapper fetch` auf
warning-ndsm-unreadable = nDSM { $tile } nicht lesbar ({ $error }); löschen Sie { $path } und rufen Sie `freemapper fetch` auf
warning-surface-unusable = Oberflächenmodell nicht verwendbar ({ $error }); rufen Sie `freemapper fetch` auf

## Downloads und der Zwischenspeicher

progress-downloading = { $url } wird geladen
progress-downloaded = { $megabytes } MB geladen
progress-downloaded-of = { $megabytes } MB von { $total } MB geladen
progress-tile-index-download = Kachelübersicht für { $product } wird geladen
progress-tile-cached = { $tile } aus dem Zwischenspeicher
warning-tile-cached-older = Die Kachel { $tile } ({ $product }) kommt aus dem Zwischenspeicher, Stand { $published }; die neuere vom { $newer } wurde nicht geladen
warning-ndsm-no-cached = Kein nDSM für die Kachel { $tile } im Zwischenspeicher
warning-ndsm-tile-unavailable-cached = nDSM-Kachel { $tile } nicht verfügbar ({ $reason }); die Datei aus dem Zwischenspeicher wird verwendet
warning-ndsm-tile-unavailable-skipped = nDSM-Kachel { $tile } nicht verfügbar ({ $reason }); übersprungen
progress-ndsm-not-published = Für die Kachel { $tile } ist keine nDSM-Punktwolke veröffentlicht
progress-ndsm-cached = nDSM { $tile } aus dem Zwischenspeicher
warning-osm-nrw-nearest = Kein Kreis-Auszug für Nordrhein-Westfalen liegt näher als 100 m am Gebiet; der nächste wird gelesen, { $district }
warning-osm-disk-full-cached = Der Datenträger ist voll, deshalb verwendet freemapper den OSM-Auszug vom { $date } aus dem Zwischenspeicher. Schaffen Sie Platz und rufen Sie `freemapper fetch` erneut auf, um einen neueren zu erhalten
warning-osm-geofabrik-unavailable = Geofabrik nicht verfügbar ({ $error }); der OSM-Auszug vom { $date } aus dem Zwischenspeicher wird verwendet
progress-osm-extract-cached = OSM-Auszug { $region } vom { $date } aus dem Zwischenspeicher
progress-osm-extract-download = { $url } wird geladen (etwa { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = bDOM-Kachel { $tile } wird geladen ({ $megabytes } MB, { $index } von { $total })
progress-sh-dgm-not-in-atom = Die DGM1-Kachel { $tile } liegt nicht im INSPIRE-ATOM-Verzeichnis; ihre XYZ-Datei wird geladen ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = Die DGM1-Kachel { $tile } aus dem INSPIRE-ATOM-Verzeichnis lässt sich nicht lesen ({ $error }); ihre XYZ-Datei wird geladen ({ $megabytes } MB)
progress-sh-tile = Kachel { $tile }: { $text }
progress-sh-download-again = Kachel { $tile }: Der Download lässt sich nicht lesen ({ $message }); er wird noch einmal geladen
warning-sh-portal-no-date = Das Downloadportal nennt kein Datum für die Kachel { $tile }, { $model }
warning-sh-dgm-xyz-used = Die DGM1-Kachel { $tile } liegt nicht im INSPIRE-ATOM-Verzeichnis; ihre XYZ-Datei aus dem Downloadportal wurde verwendet
warning-sh-dgm-xyz-used-unreadable = Die DGM1-Kachel { $tile } aus dem INSPIRE-ATOM-Verzeichnis lässt sich nicht lesen ({ $error }); ihre XYZ-Datei aus dem Downloadportal wurde verwendet

## Kachellisten und Datumsangaben der Höhenmodelle

warning-model-tile-cells-empty = { $model }, Kachel { $tile }: { $count ->
        [one] 1 Zelle des Kartengebiets hat
       *[other] { $count } Zellen des Kartengebiets haben
    } keine Höhe
warning-centre-tile-missing = { $model }: Die Kachel { $tile } in der Mitte des Kartengebiets fehlt, deshalb gibt es kein Datum
warning-centre-tile-unlisted = { $model }: Die Kachelliste nennt die Kachel { $tile } in der Mitte des Kartengebiets nicht, deshalb gibt es kein Datum
warning-list-unreadable-kept = { $model }: Die Kachelliste ließ sich nicht lesen ({ $error }); die Liste aus einem früheren Lauf wurde verwendet
warning-portal-list-unreadable-kept = { $model }: Die Kachelliste des Downloadportals ließ sich nicht lesen ({ $error }); die Liste aus einem früheren Lauf wurde verwendet
warning-surface-much-older = Das { $model } ist vom { $surface }, das Orthophoto vom { $flight }. Im nDSM können neuere Gebäude und Bäume fehlen.

## Rheinland-Pfalz

warning-rp-list-features-unusable = { $count ->
        [one] 1 Eintrag der Kachelliste hat
       *[other] { $count } Einträge der Kachelliste haben
    } keinen brauchbaren Kachelschlüssel; übersprungen
warning-rp-list-unreadable-kept = Die Kachelliste von LVermGeo RP ließ sich nicht lesen ({ $error }); die Liste aus einem früheren Lauf wurde verwendet
warning-rp-metalink-not-kept = Der Metalink von LVermGeo RP wurde nicht aufbewahrt: { $error }
warning-rp-metalink-unreadable-kept = { $model }: Der Metalink von LVermGeo RP ließ sich nicht lesen ({ $detail }); der Metalink aus einem früheren Lauf wurde verwendet
warning-rp-metalink-unreadable = { $model }: Der Metalink von LVermGeo RP ließ sich nicht lesen ({ $detail }); die Kacheln, für die die Kachelliste kein Flugjahr nennt, bleiben deshalb ohne Höhen
warning-rp-tile-without-name = { $model }, Kachel { $tile }: Dieses 1-km-Quadrat hat in der Karte keine Höhen. Die Kachelliste von LVermGeo RP (Laserbefliegung) { $note }, der Dateiname ist deshalb nicht bekannt{ $either ->
        [metalink] , und auch der Metalink nennt keine Datei für sie
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }: Die Kachel { $tile } hat kein Datum: Die Kachelliste von LVermGeo RP (Laserbefliegung) { $note }
word-rp-list-neither = nennt für sie weder ein Flugjahr noch Flugtage
word-rp-list-no-days = nennt für sie keine Flugtage
word-rp-list-no-year = nennt für sie kein Flugjahr
word-rp-list-days-unreadable = nennt für sie Flugtage { $days }, die sich nicht lesen lassen
word-rp-list-disagree = nennt für sie Flugtage { $days } und ein Flugjahr { $year }, die nicht auf ein Jahr passen

## Hessen

warning-hvbg-undated = Die Metadatenliste hat kein lesbares Bildflugdatum für die Kachel { $tiles }; die Projekteinträge dazu haben kein Flugdatum
warning-hvbg-undated-more = Die Metadatenliste hat kein lesbares Bildflugdatum für die Kachel { $tiles } und { $more } weitere; die Projekteinträge dazu haben kein Flugdatum
warning-hvbg-list-unreadable-older = Die neue HVBG-Metadatenliste { $name } ließ sich nicht lesen ({ $detail }); die ältere Liste { $older } wird verwendet

## Niedersachsen

warning-lgln-no-date-unlisted = { $model }: kein Datum, die STAC-Suche nennt in der Mitte des Kartengebiets keine Kachel
warning-lgln-no-date = { $model }: kein Datum ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; das gespeicherte Modell behält sein vermerktes Datum
warning-mv-cells-empty = { $model }: { $count ->
        [one] 1 Zelle innerhalb der Kacheln von Mecklenburg-Vorpommern hat
       *[other] { $count } Zellen innerhalb der Kacheln von Mecklenburg-Vorpommern haben
    } keine Höhe
warning-mv-centre-no-date = { $model }: kein Datum für die mittlere Kachel { $tile }

## Nordrhein-Westfalen

warning-nrw-list-rows-unusable = { $model }: { $count ->
        [one] 1 Zeile der Kachelliste hat
       *[other] { $count } Zeilen der Kachelliste haben
    } keinen brauchbaren Kachelnamen; übersprungen
warning-nrw-list-rows-undated = { $model }: { $count ->
        [one] 1 Zeile der Kachelliste hat
       *[other] { $count } Zeilen der Kachelliste haben
    } kein lesbares Datum
warning-nrw-list-older = { $model }: Die Kachelliste des Servers ist vom { $server } und damit älter als die aufbewahrte vom { $kept }; die aufbewahrte wurde verwendet
warning-nrw-list-no-date = Die Kachelliste nennt kein Datum für die Kachel { $tile }, { $model }
