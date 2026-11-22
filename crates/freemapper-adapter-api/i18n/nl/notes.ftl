# Voortgangsregels en waarschuwingen van de engine (ADR 0092, stap 3). Engels is de referentie.
# Ids: progress- voor voortgangsregels, warning- voor waarschuwingen, word- voor namen binnen
# andere berichten. Een waarschuwing krijgt zijn voorvoegsel van note-warning.

note-warning = waarschuwing: { $text }

warning-other-zone = het kaartgebied ligt in UTM-zone { $zone } (EPSG:{ $epsg }); freemapper haalt alles opnieuw op in die zone
warning-other-state = het kaartgebied ligt nu in de gegevens van een andere deelstaat; freemapper haalt alles opnieuw op
warning-beyond-coverage = een deel van het kaartgebied ligt buiten de ondersteunde deelstaten. Het ligt in het buitenland, op zee of in een deelstaat die freemapper nog niet ondersteunt, zoals Thüringen. De deelstaten hebben daar geen gegevens, dus de orthofoto kan wit zijn en ALKIS levert geen gebouwen of grondgebruik
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: schaal 1:{ $scale } hoort niet bij de schalen voor { $kind }kaarten ({ $allowed }); freemapper-print weigert de kaart
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: laagitem { $key } van { $german ->
       *[any] { $what }
    } genegeerd: { $reason ->
        [name] geen gewone bestandsnaam
       *[unknown] geen laag die freemapper kent
    }

## Disk space

warning-disk-space-unreadable = de vrije schijfruimte van { $path } kan niet worden gelezen ({ $error }), dus freemapper controleert die niet

## Tiles

warning-tile-changed = { $product }-tegel { $tile } is bij de bron gewijzigd ({ $old } → { $new })
warning-bdom-tile-unavailable = oppervlaktemodel (bDOM) van tegel { $tile } niet beschikbaar ({ $error }); daar geen nDSM-template
warning-bdom-unavailable = oppervlaktemodel (bDOM) niet beschikbaar ({ $error }); geen nDSM-template voor { $tiles }
progress-ndsm-from-bdom = nDSM van tegel { $tile } uit het oppervlaktemodel (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terreinmodel
        [surface] oppervlaktemodel
       *[orthophoto] orthofoto
    } ({ $name })
progress-model-download = { $model } wordt gedownload
progress-model-read = { $model } wordt gelezen
warning-model-unavailable = { $model } niet beschikbaar ({ $error }); geen nDSM-template
warning-model-unavailable-kept = { $model } niet beschikbaar ({ $error }); het oppervlaktemodel van { $fetched } is behouden
warning-model-tile-unavailable = { $model } van tegel { $tile } niet beschikbaar ({ $error }); daar geen nDSM-template
warning-archive-passed-over = { $model }: een archief kon niet worden gelezen ({ $error }); het is overgeslagen en de tegels van het kaartgebied komen uit de andere archieven
warning-archive-directory-kept = { $model }: een archief kon niet worden gelezen ({ $error }); de inhoudsopgave uit een eerdere run wordt gebruikt, dus een tegel die de LVGL sindsdien heeft vervangen wordt niet opgemerkt
warning-model-tiles-missing = { $model } heeft geen { $count ->
        [one] tegel
       *[other] tegels
    } { $names } binnen { $state ->
        [the-map-area] het kaartgebied
        [hamburg] Hamburg
        [hesse] Hessen
        [north-rhine-westphalia] Noordrijn-Westfalen
        [schleswig-holstein] Sleeswijk-Holstein
        [mecklenburg-vorpommern] Mecklenburg-Voor-Pommeren
        [lower-saxony] Nedersaksen
        [bavaria] Beieren
       *[other] { $state }
    }; de templates blijven daar leeg
warning-no-surface-stored = geen oppervlaktemodel opgeslagen; voer `freemapper fetch` online uit voor de nDSM-template
warning-surface-older-than-orthophoto = het oppervlaktemodel (bDOM { $surface }) is ouder dan de orthofoto ({ $photo }); nieuwe gebouwen kunnen in de nDSM ontbreken
progress-surface-kept = freemapper behoudt het oppervlaktemodel van { $fetched } voor de nDSM-template
progress-surface-tiles-kept = freemapper behoudt het gecachete oppervlaktemodel van { $count ->
        [one] tegel
       *[other] tegels
    } { $names } voor de nDSM-template
progress-orthophoto-unchanged = orthofoto ongewijzigd
progress-orthophoto-unchanged-dated = orthofoto ongewijzigd ({ $label } { $date })
progress-orthophoto-download = orthofoto wordt gedownload van { $provider } { $service }
progress-orthophoto-in-leaf = de orthofoto is gemaakt op { $date }, met bomen in blad; hun kruinen verbergen wat eronder ligt
progress-orthophoto-bare = de orthofoto is gemaakt op { $date }, met kale bomen
warning-lgln-orthophoto-kept = orthofoto (LGLN WMS): { $reason }; de opgeslagen orthofoto en de vluchtdatum { $date } zijn behouden
warning-lgln-orthophoto-undated = orthofoto (LGLN WMS): geen vluchtdatum, { $reason }
word-stac-no-image = de STAC vermeldt geen beeld in het midden van het kaartgebied
word-stac-no-answer = de STAC-zoekopdracht gaf geen bruikbaar antwoord ({ $error })
warning-downloadcenter-no-file = het Downloadcenter heeft geen bestand voor { $places }
warning-downloadcenter-no-file-empty = het Downloadcenter heeft geen bestand voor { $places }; { $count ->
        [one] tegel
       *[other] tegels
    } { $names } bleef leeg
progress-k-five-unchanged = K5-template ongewijzigd
progress-k-five-download = K5-kaart (1:5000) wordt gedownload van de Geoportal Berlin WMS
warning-k-five-unavailable = de K5-kaarttemplate is niet beschikbaar ({ $detail }); overgeslagen

## The state border

progress-beyond-border = het kaartgebied reikt voorbij de deelstaatgrens; daarachter zijn de templates leeg waar de orthofoto en hoogtemodellen van { $provider } ophouden. ALKIS en OSM komen uit beide deelstaten
progress-beyond-border-no-data = het kaartgebied reikt voorbij de deelstaatgrens; geen gegevens van { $provider } voor { $tiles }, dus de templates blijven daar leeg

## OSM

warning-osm-extract-missing-kept = geen OSM-extract in de cache; de opgeslagen OSM-kandidaten van { $date } blijven behouden
progress-osm-extract-read = OSM-extract { $region } van { $date } wordt gelezen
progress-osm-candidates = { $count } OSM-kandidaatobjecten

## Permit notes

progress-permits-fetch = gegevens van { $source } worden opgehaald voor de vergunningsnotities
progress-permits-osm = OSM-extract { $region } van { $date } wordt gelezen voor de vergunningsnotities
progress-permits-stored = de opgeslagen gegevens van de vergunningsnotities worden gelezen
warning-permits-source-unreachable = { $source } kon niet worden gelezen, dus de vergunningsnotities laten die weg: { $error }
warning-permits-osm-no-extract = voor dit kaartgebied staat geen OSM-extract in de cache, dus de vergunningsnotities laten OSM weg; `freemapper fetch` downloadt het

## ALKIS

warning-alkis-coverage-unknown = dekking van { $cadastre }-gebouwen onbekend; voer `freemapper fetch` online uit om die vast te leggen
progress-alkis-buildings-download = { $cadastre }-gebouwen worden gedownload
progress-alkis-buildings = { $count } { $cadastre }-gebouwen
warning-alkis-no-buildings = geen { $cadastre }-gebouwen gevonden in het kaartgebied
progress-alkis-skipped = { $count } { $cadastre }-objecten overgeslagen: { $list }
warning-alkis-buildings-changed = { $cadastre }-gebouwen zijn bij de bron gewijzigd ({ $changes })
progress-alkis-no-land-use-stored = geen { $cadastre }-grondgebruik opgeslagen; voer `freemapper fetch` online uit voor kandidaten buiten het kaartgebied
progress-alkis-land-use-download = { $cadastre }-grondgebruik wordt gedownload
progress-alkis-land-use-parts = { $count } { $cadastre }-grondgebruiksdelen
warning-alkis-land-use-kept = { $cadastre }-grondgebruik niet beschikbaar ({ $error }); het opgeslagen grondgebruik blijft behouden
warning-alkis-land-use-removed = { $cadastre }-grondgebruik niet beschikbaar ({ $error }); het opgeslagen grondgebruik dekt het kaartgebied niet meer, dus freemapper heeft het verwijderd
warning-alkis-land-use-none = { $cadastre }-grondgebruik niet beschikbaar ({ $error }); geen kandidaten voor verboden gebied
progress-bdot-filled = { $count } BDOT10k-objecten ({ $what }) aangevuld waar EGiB niets geeft ({ $powiat })
warning-bdot-failed = BDOT10k kon de leemte in EGiB niet vullen ({ $what }): { $error }
warning-bdot-instead = EGiB is niet beschikbaar ({ $error }); in plaats daarvan BDOT10k gebruikt ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = geen Berlijnse ALKIS-structuren opgeslagen; voer `freemapper fetch` online uit voor trappen, muren, bomen en kronen
progress-berlin-removing = de opgeslagen Berlijnse lagen worden verwijderd: het kaartgebied ligt niet meer bij Berlijn
progress-berlin-download = Berlijnse ALKIS-structuren, boomkadaster, straatopname en puntlagen worden gedownload
progress-berlin-features = { $count } Berlijnse { $file }-objecten
warning-berlin-layer-kept = Berlijnse { $file } niet beschikbaar ({ $detail }); de opgeslagen laag blijft behouden
warning-berlin-layer-skipped = Berlijnse { $file } niet beschikbaar ({ $detail }); laag overgeslagen
progress-feinkartierung-none-stored = geen Hamburgse Feinkartierung opgeslagen; voer `freemapper fetch` online uit voor leuningen, muren, trappen en straatmeubilair
progress-feinkartierung-removing = de opgeslagen Hamburgse Feinkartierung wordt verwijderd: het kaartgebied ligt niet meer in Hamburg
progress-feinkartierung-download = de Hamburgse Feinkartierung (straatruimte en openbaar groen) wordt gedownload
progress-feinkartierung-features = { $count } Feinkartierung-{ $file }-objecten
warning-feinkartierung-layer-kept = Feinkartierung { $file } niet beschikbaar ({ $detail }); de opgeslagen laag blijft behouden
warning-feinkartierung-layer-skipped = Feinkartierung { $file } niet beschikbaar ({ $detail }); laag overgeslagen
error-extra-layer-unknown = geen laag van { $folder }
warning-berlin-buildings-missing = geen ALKIS-gebouwen opgeslagen voor dit gebied in Berlijn; voer `freemapper fetch` uit om ze van de Geoportal Berlin te halen

## The full ALKIS layers (NAS)

progress-nas-none-stored = geen volledige ALKIS-structuren opgeslagen; voer `freemapper fetch` online uit voor muren, hekken, heggen en bruggen
progress-nas-removing = de opgeslagen volledige ALKIS-lagen worden verwijderd: het kaartgebied ligt niet meer in een deelstaat met een NAS-service
warning-nas-folder-outside = volledige ALKIS-lagen overgeslagen ({ $folder } leidt buiten de projectmap)
progress-nas-download = de volledige ALKIS-structuren (NAS) worden gedownload
progress-nas-features = { $count } ALKIS NAS-{ $file }-objecten
progress-nas-skipped = { $count } ALKIS NAS-{ $file }-objecten overgeslagen: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } niet beschikbaar ({ $detail }); de opgeslagen laag blijft behouden
warning-nas-layer-skipped = ALKIS NAS { $file } niet beschikbaar ({ $detail }); laag overgeslagen

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = geen objecten van het ATKIS Basis-DLM opgeslagen; voer `freemapper fetch` online uit voor bronnen, grotten, torens, monumenten en fonteinen
progress-dlm-removing = de opgeslagen lagen van het ATKIS Basis-DLM worden verwijderd: het kaartgebied ligt niet meer in een deelstaat die het aanbiedt
warning-dlm-folder-outside = lagen van het ATKIS Basis-DLM overgeslagen ({ $folder } leidt buiten de projectmap)
progress-dlm-download = de objecten van het ATKIS Basis-DLM worden gedownload
progress-dlm-features = { $count } ATKIS Basis-DLM-{ $file }-objecten
progress-dlm-skipped = { $count } ATKIS Basis-DLM-{ $file }-objecten overgeslagen: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } niet beschikbaar ({ $detail }); de opgeslagen laag blijft behouden
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } niet beschikbaar ({ $detail }); laag overgeslagen

## Tree cadastres

progress-trees-none-stored = geen stedelijk boomkadaster opgeslagen; voer `freemapper fetch` online uit voor de losse bomen
progress-trees-removing = de opgeslagen boomkadasters worden verwijderd: het kaartgebied ligt niet meer in een stad met een boomkadaster
warning-trees-folder-outside = stedelijke boomkadasters overgeslagen ({ $folder } leidt buiten de projectmap)
progress-trees-download = de stedelijke boomkadasters worden gedownload
progress-trees-count = { $count } { $id }-bomen
progress-trees-skipped = { $count } { $id }-bomen overgeslagen: { $list }
warning-trees-kept = boomkadaster { $id } niet beschikbaar ({ $detail }); het opgeslagen kadaster blijft behouden
warning-trees-skipped = boomkadaster { $id } niet beschikbaar ({ $detail }); bomen overgeslagen

## Deriving the templates

progress-writing = { $path } wordt geschreven
progress-contours-steep = hoogtelijnen om de { $interval } m: het terrein is zo steil dat hoogtelijnen om de { $default } m in elkaar zouden lopen; met een hoogtelijninterval van { $default } m in het project blijven ze gescheiden (`--contour-interval { $default }`)
warning-osm-only-no-terrain = nog geen terreingegevens voor deze regio: de kaart wordt alleen uit OpenStreetMap gemaakt, zonder hoogtelijnen, templates en gebouwen
progress-no-ndsm-recorded = geen nDSM vastgelegd; voer `freemapper fetch` uit voor de nDSM-template
warning-ndsm-tile-invalid = ongeldige nDSM-tegel { $tile } in freemapper.toml; overgeslagen
warning-ndsm-not-cached = nDSM { $tile } niet in de cache; voer `freemapper fetch` uit
warning-ndsm-unreadable = nDSM { $tile } onleesbaar ({ $error }); verwijder { $path } en voer `freemapper fetch` uit
warning-surface-unusable = oppervlaktemodel onbruikbaar ({ $error }); voer `freemapper fetch` uit

## Downloads and the cache

progress-downloading = { $url } wordt gedownload
progress-retry-wait = de dienst is druk; nieuwe poging over { $seconds } s
progress-downloaded = { $megabytes } MB gedownload
progress-downloaded-of = { $megabytes } MB van { $total } MB gedownload
progress-tile-index-download = de tegelindex van { $product } wordt gedownload
progress-tile-cached = { $tile } in de cache
progress-surface-blocks = { $count } van de { $total } blokken van oppervlaktemodeltegel { $tile } worden gelezen ({ $megabytes } MB)
progress-surface-whole = oppervlaktemodeltegel { $tile } wordt in zijn geheel gedownload: { $reason }
warning-tile-cached-older = gecachete { $product }-tegel { $tile } van { $published } wordt gebruikt (nieuwere van { $newer } niet gedownload)
warning-ndsm-no-cached = geen nDSM in de cache voor tegel { $tile }
warning-ndsm-tile-unavailable-cached = nDSM-tegel { $tile } niet beschikbaar ({ $reason }); het gecachete bestand wordt gebruikt
warning-ndsm-tile-unavailable-skipped = nDSM-tegel { $tile } niet beschikbaar ({ $reason }); overgeslagen
warning-file-check-failed = kon niet controleren of { $file } nog actueel is ({ $reason }); het gecachete bestand wordt gebruikt, en de andere gecachete bestanden van deze server zonder te vragen
warning-ldbv-no-buildings-file = geen gebouwenbestand voor tegel { $tile } op de server; de kaart kan daar gebouwen missen
progress-ndsm-not-published = geen nDSM-puntenwolk gepubliceerd voor tegel { $tile }
progress-ndsm-cached = nDSM { $tile } in de cache
progress-point-cloud-not-published = geen laserpuntenwolk gepubliceerd voor tegel { $tile }
progress-point-cloud-cached = puntenwolk { $tile } in de cache
warning-point-cloud-no-cached = geen puntenwolk in de cache voor tegel { $tile }; daar geen vegetatiekandidaten
warning-point-cloud-unavailable-cached = puntenwolk { $tile } niet beschikbaar ({ $reason }); het gecachete bestand wordt gebruikt
warning-point-cloud-unavailable-skipped = puntenwolk { $tile } niet beschikbaar ({ $reason }); daar geen vegetatiekandidaten
warning-point-clouds-unavailable = de laserpuntenwolken zijn niet beschikbaar ({ $reason }); de vegetatiekandidaten gebruiken de gecachete tegels, als die er zijn
progress-no-point-clouds = hier nog geen laserpuntenwolken voor de vegetatie: de deelstaat publiceert er geen die freemapper kan lezen, of niet in het coördinatenstelsel van de kaart
warning-point-clouds-not-fetched = de vegetatie is gekozen, maar er zijn geen puntenwolken geladen; voer `freemapper fetch` uit om ze te laden
warning-point-cloud-unreadable = puntenwolk { $tile } onleesbaar ({ $error }); verwijder { $path } en voer `freemapper fetch` uit
warning-osm-nrw-nearest = geen districtsextract van Noordrijn-Westfalen binnen 100 m van het gebied; het dichtstbijzijnde wordt gelezen, { $district }
warning-osm-disk-full-cached = de schijf is vol, dus freemapper gebruikt het gecachete OSM-extract van { $date }. Maak ruimte vrij en voer `freemapper fetch` opnieuw uit voor een nieuwer extract
warning-osm-geofabrik-unavailable = Geofabrik niet beschikbaar ({ $error }); het gecachete OSM-extract van { $date } wordt gebruikt
progress-osm-extract-cached = OSM-extract { $region } van { $date } in de cache
progress-osm-extract-download = { $url } wordt gedownload (ongeveer { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = bDOM-tegel { $tile } wordt gedownload ({ $megabytes } MB, { $index } van { $total })
progress-sh-dgm-not-in-atom = DGM1-tegel { $tile } staat niet in de INSPIRE ATOM-map; de XYZ wordt gedownload ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1-tegel { $tile } uit de INSPIRE ATOM-map is niet leesbaar ({ $error }); de XYZ wordt gedownload ({ $megabytes } MB)
progress-sh-tile = tegel { $tile }: { $text }
progress-sh-download-again = tegel { $tile }: de download is niet leesbaar ({ $message }); hij wordt opnieuw gedownload
warning-sh-portal-no-date = het Downloadportal geeft geen datum voor { $model }-tegel { $tile }
warning-sh-dgm-xyz-used = DGM1-tegel { $tile } staat niet in de INSPIRE ATOM-map; de XYZ van het Downloadportal is gebruikt
warning-sh-dgm-xyz-used-unreadable = DGM1-tegel { $tile } uit de INSPIRE ATOM-map is niet leesbaar ({ $error }); de XYZ van het Downloadportal is gebruikt

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }-tegel { $tile }: { $count } cellen van het kaartgebied hebben geen hoogte
warning-centre-tile-missing = { $model }-tegel { $tile } in het midden van het kaartgebied ontbreekt, dus er is geen datum
warning-centre-tile-unlisted = de tegellijst noemt geen { $model }-tegel { $tile } in het midden van het kaartgebied, dus er is geen datum
warning-list-unreadable-kept = de tegellijst van { $model } kon niet worden gelezen ({ $error }); de lijst uit een eerdere run is gebruikt
warning-portal-list-unreadable-kept = de tegellijst van het Downloadportal voor { $model } kon niet worden gelezen ({ $error }); de lijst uit een eerdere run is gebruikt
warning-surface-much-older = { $model } is van { $surface }, de orthofoto van { $flight }. De nDSM kan nieuwere gebouwen en bomen missen.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objecten van de tegellijst hebben geen bruikbare tegelsleutel; overgeslagen
warning-rp-list-unreadable-kept = de LVermGeo RP-tegellijst kon niet worden gelezen ({ $error }); de lijst uit een eerdere run is gebruikt
warning-rp-metalink-not-kept = de LVermGeo RP-Metalink is niet bewaard: { $error }
warning-rp-metalink-unreadable-kept = de LVermGeo RP-Metalink van { $model } kon niet worden gelezen ({ $detail }); de Metalink uit een eerdere run is gebruikt
warning-rp-metalink-unreadable = de LVermGeo RP-Metalink van { $model } kon niet worden gelezen ({ $detail }), dus de tegels waarvoor de tegellijst geen vluchtjaar geeft blijven zonder hoogtes
warning-rp-tile-without-name = { $model }-tegel { $tile }: dit vierkant van 1 km heeft geen hoogtes in de kaart. De tegellijst van LVermGeo RP (Laserbefliegung) { $note } ervoor, dus de bestandsnaam is niet bekend{ $either ->
        [metalink] , en ook de Metalink noemt er geen bestand voor
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }-tegel { $tile } heeft geen datum: de tegellijst van LVermGeo RP (Laserbefliegung) { $note } ervoor
word-rp-list-neither = geeft geen vluchtjaar en geen vluchtdagen
word-rp-list-no-days = geeft geen vluchtdagen
word-rp-list-no-year = geeft geen vluchtjaar
word-rp-list-days-unreadable = geeft vluchtdagen { $days } die niet te lezen zijn
word-rp-list-disagree = geeft vluchtdagen { $days } en een vluchtjaar { $year } die het niet eens zijn over één jaar

## Hesse

warning-hvbg-undated = de metadatalijst heeft geen leesbare Bildflugdatum voor tegel { $tiles }; hun projectgegevens hebben geen vluchtdatum
warning-hvbg-undated-more = de metadatalijst heeft geen leesbare Bildflugdatum voor tegel { $tiles } en { $more } meer; hun projectgegevens hebben geen vluchtdatum
warning-hvbg-list-unreadable-older = de nieuwe HVBG-metadatalijst { $name } kon niet worden gelezen ({ $detail }); de oudere lijst { $older } wordt gebruikt

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: geen datum, de STAC vermeldt geen tegel in het midden van het kaartgebied
warning-lgln-no-date = { $model }: geen datum ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; het opgeslagen model behoudt zijn vastgelegde datum
warning-mv-cells-empty = { $model }: { $count } cellen binnen de tegels van Mecklenburg-Voor-Pommeren hebben geen hoogte
warning-mv-centre-no-date = { $model }: geen datum voor de middelste tegel { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } rijen van de tegellijst van { $model } hebben geen bruikbare tegelnaam; overgeslagen
warning-nrw-list-rows-undated = { $count } rijen van de tegellijst van { $model } hebben geen leesbare datum
warning-nrw-list-older = de tegellijst van de server voor { $model } is van { $server }, ouder dan de bewaarde van { $kept }; de bewaarde is gebruikt
warning-nrw-list-no-date = de tegellijst geeft geen datum voor { $model }-tegel { $tile }

## After `init`

note-scale-outside-norm = let op: 1:5000 hoort niet bij de sprintnorm; freemapper-print waarschuwt daarvoor en de symbolen blijven op 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } is geen OSM-id zoals way/992423119
error-place-invalid-answer = ongeldig antwoord van de plaatszoekopdracht: { $detail }
error-place-not-found = { $id } niet gevonden. Nominatim vermeldt alleen OSM-objecten met tags, dus een buitenste way zonder tags van een multipolygoon staat er niet in. Gebruik in plaats daarvan de multipolygoonrelatie (--osm relation/<id>), of sla de omtrek op als GeoJSON en gebruik --geojson
error-place-is-a-point = { $id } is een punt, geen gebied; kies het terrein ervan (een way of relation)
error-place-no-such-part = { $id } heeft { $count ->
        [one] 1 deel
       *[other] { $count } delen
    }; er is geen deel { $part }
error-place-several-parts = { $id } is een gebied van { $count } afzonderlijke delen ({ $sizes }). Een kaartgebied is één polygoon: kies een deel met --part <n> (1 is het grootste), of sla de gewenste omtrek op als GeoJSON en gebruik --geojson
error-layer-not-a-world-file = { $path }: geen worldfile
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = geen voorvertoning voor { $layer }; lagen met voorvertoning: { $names }
error-layer-no-ndsm-published = voor de tegels van dit kaartgebied is geen nDSM gepubliceerd, dus er is geen ndsm-template
error-layer-not-made = { $layer } is nog niet gemaakt; voer { $step ->
        [fetch] freemapper fetch of run uit (MCP: start_pipeline); alleen Berlijn
        [export] freemapper export of run uit (MCP: start_pipeline)
       *[derive] freemapper derive of run uit (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; de OCAD-kaart krijgt daardoor geen achtergrond
warning-map-preview-failed = de kaart is geschreven, maar de voorvertoning ontbreekt: { $error }
warning-training-no-tree-cover = de trainingskaart houdt het open terrein van parken heel: de boombedekking (data/tree-cover.tif) ontbreekt; voer `freemapper derive` opnieuw uit
