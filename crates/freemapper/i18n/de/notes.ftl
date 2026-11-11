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
