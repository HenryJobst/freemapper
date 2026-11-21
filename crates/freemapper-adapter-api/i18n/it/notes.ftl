# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = avviso: { $text }

warning-other-zone = l'area della carta si trova nella zona UTM { $zone } (EPSG:{ $epsg }); freemapper scarica di nuovo tutto in quella zona
warning-other-state = l'area della carta ora si trova nei dati di un altro Land; freemapper scarica di nuovo tutto
warning-beyond-coverage = una parte dell'area della carta si trova fuori dai Land supportati. Si trova all'estero, in mare o in un Land che freemapper non supporta ancora, come la Turingia. Lì i Land non hanno dati, quindi l'ortofoto può essere bianca e ALKIS non fornisce edifici né uso del suolo
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: la scala 1:{ $scale } non è tra le scale per le carte di tipo { $kind } ({ $allowed }); freemapper-print rifiuterà la carta
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorata la voce { $key } del livello { $german ->
       *[any] { $what }
    }: { $reason ->
        [name] non è un semplice nome di file
       *[unknown] nessun livello che freemapper conosca
    }

## Disk space

warning-disk-space-unreadable = impossibile leggere lo spazio libero su disco di { $path } ({ $error }), quindi freemapper non lo controlla

## Tiles

warning-tile-changed = il riquadro { $product } { $tile } è cambiato presso il fornitore ({ $old } → { $new })
warning-bdom-tile-unavailable = modello di superficie (bDOM) del riquadro { $tile } non disponibile ({ $error }); lì nessun template nDSM
warning-bdom-unavailable = modello di superficie (bDOM) non disponibile ({ $error }); nessun template nDSM per { $tiles }
progress-ndsm-from-bdom = nDSM del riquadro { $tile } dal modello di superficie (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] modello del terreno
        [surface] modello di superficie
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = scaricamento: { $model }
progress-model-read = lettura: { $model }
warning-model-unavailable = { $model } non disponibile ({ $error }); nessun template nDSM
warning-model-unavailable-kept = { $model } non disponibile ({ $error }); mantenuto il modello di superficie scaricato il { $fetched }
warning-model-tile-unavailable = { $model }, riquadro { $tile }: non disponibile ({ $error }); lì nessun template nDSM
warning-archive-passed-over = { $model }: impossibile leggere un archivio ({ $error }); è stato saltato e i riquadri dell'area della carta vengono dagli altri archivi
warning-archive-directory-kept = { $model }: impossibile leggere un archivio ({ $error }); si usa il suo elenco conservato da un'esecuzione precedente, quindi un riquadro che nel frattempo l'LVGL ha sostituito non viene notato
warning-model-tiles-missing = { $model }: { $count ->
        [one] manca il riquadro
        [many] mancano i riquadri
       *[other] mancano i riquadri
    } { $names } { $state ->
        [the-map-area] nell'area della carta
        [hamburg] ad Amburgo
        [hesse] in Assia
        [north-rhine-westphalia] nella Renania Settentrionale-Vestfalia
        [schleswig-holstein] nello Schleswig-Holstein
        [mecklenburg-vorpommern] nel Meclemburgo-Pomerania Anteriore
        [lower-saxony] nella Bassa Sassonia
        [bavaria] in Baviera
       *[other] in { $state }
    }; lì i template restano vuoti
warning-no-surface-stored = nessun modello di superficie salvato; esegui `freemapper fetch` online per il template nDSM
warning-surface-older-than-orthophoto = il modello di superficie (bDOM { $surface }) è più vecchio dell'ortofoto ({ $photo }); nell'nDSM possono mancare nuovi edifici
progress-surface-kept = freemapper conserva il modello di superficie scaricato il { $fetched } per il template nDSM
progress-surface-tiles-kept = freemapper conserva nella cache il modello di superficie { $count ->
        [one] del riquadro
        [many] dei riquadri
       *[other] dei riquadri
    } { $names } per il template nDSM
progress-orthophoto-unchanged = ortofoto invariata
progress-orthophoto-unchanged-dated = ortofoto invariata ({ $label } { $date })
progress-orthophoto-download = scaricamento dell'ortofoto da { $provider } { $service }
progress-orthophoto-in-leaf = l'ortofoto è stata ripresa il { $date }, con gli alberi in foglia; le loro chiome nascondono ciò che sta sotto
progress-orthophoto-bare = l'ortofoto è stata ripresa il { $date }, con gli alberi spogli
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; conservata l'ortofoto salvata e la sua data di volo { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): nessuna data di volo, { $reason }
word-stac-no-image = lo STAC non elenca alcuna immagine al centro dell'area della carta
word-stac-no-answer = la ricerca STAC non ha dato una risposta utilizzabile ({ $error })
warning-downloadcenter-no-file = il Downloadcenter non ha alcun file per { $places }
warning-downloadcenter-no-file-empty = il Downloadcenter non ha alcun file per { $places }; { $count ->
        [one] il riquadro
        [many] i riquadri
       *[other] i riquadri
    } { $names } { $count ->
        [one] è rimasto vuoto
        [many] sono rimasti vuoti
       *[other] sono rimasti vuoti
    }
progress-k-five-unchanged = template K5 invariato
progress-k-five-download = scaricamento della carta K5 (1:5000) dal WMS del Geoportal Berlin
warning-k-five-unavailable = il template della carta K5 non è disponibile ({ $detail }); saltato

## The state border

progress-beyond-border = l'area della carta oltrepassa il confine del Land; oltre di esso i template sono vuoti dove finiscono l'ortofoto { $provider } e i modelli di quota. ALKIS e OSM vengono da entrambi i Land
progress-beyond-border-no-data = l'area della carta oltrepassa il confine del Land; nessun dato { $provider } per { $tiles }, quindi lì i template restano vuoti

## OSM

warning-osm-extract-missing-kept = nessun estratto OSM nella cache; si tengono i candidati OSM salvati del { $date }
progress-osm-extract-read = lettura dell'estratto OSM { $region } del { $date }
progress-osm-candidates = { $count } oggetti candidati OSM

## Permit notes

progress-permits-fetch = scaricamento dei dati { $source } per le note sui permessi
progress-permits-osm = lettura dell'estratto OSM { $region } del { $date } per le note sui permessi
progress-permits-stored = lettura dei dati salvati delle note sui permessi
warning-permits-source-unreachable = impossibile leggere { $source }, quindi le note sui permessi lo omettono: { $error }
warning-permits-osm-no-extract = nessun estratto OSM nella cache per questa area della carta, quindi le note sui permessi omettono OSM; `freemapper fetch` lo scarica

## ALKIS

warning-alkis-coverage-unknown = copertura degli edifici ALKIS sconosciuta; esegui `freemapper fetch` online per registrarla
progress-alkis-buildings-download = scaricamento degli edifici ALKIS
progress-alkis-buildings = { $count } edifici ALKIS
warning-alkis-no-buildings = nessun edificio ALKIS trovato nell'area della carta
progress-alkis-skipped = saltati { $count } oggetti ALKIS: { $list }
warning-alkis-buildings-changed = gli edifici ALKIS sono cambiati presso il fornitore ({ $changes })
progress-alkis-no-land-use-stored = nessun uso del suolo ALKIS salvato; esegui `freemapper fetch` online per i candidati delle aree vietate
progress-alkis-land-use-download = scaricamento dell'uso del suolo ALKIS
progress-alkis-land-use-parts = { $count } parti di uso del suolo ALKIS
warning-alkis-land-use-kept = uso del suolo ALKIS non disponibile ({ $error }); si tiene quello salvato
warning-alkis-land-use-removed = uso del suolo ALKIS non disponibile ({ $error }); quello salvato non copre più l'area della carta, quindi freemapper lo ha rimosso
warning-alkis-land-use-none = uso del suolo ALKIS non disponibile ({ $error }); nessun candidato di aree vietate

## The Berlin layers

progress-berlin-none-stored = nessuna struttura ALKIS di Berlino salvata; esegui `freemapper fetch` online per scale, muri, alberi e tettoie
progress-berlin-removing = rimozione dei livelli di Berlino salvati: l'area della carta non è più vicina a Berlino
progress-berlin-download = scaricamento delle strutture ALKIS di Berlino, del catasto degli alberi, del rilievo stradale e dei livelli di punti
progress-berlin-features = { $count } oggetti Berlino { $file }
warning-berlin-layer-kept = Berlino { $file } non disponibile ({ $detail }); si tiene il livello salvato
warning-berlin-layer-skipped = Berlino { $file } non disponibile ({ $detail }); livello saltato
progress-feinkartierung-none-stored = nessuna Feinkartierung di Amburgo salvata; esegui `freemapper fetch` online per ringhiere, muri, scale e arredo urbano
progress-feinkartierung-removing = rimozione della Feinkartierung di Amburgo salvata: l'area della carta non è più ad Amburgo
progress-feinkartierung-download = scaricamento della Feinkartierung di Amburgo (spazio stradale e verde pubblico)
progress-feinkartierung-features = { $count } oggetti Feinkartierung { $file }
warning-feinkartierung-layer-kept = Feinkartierung { $file } non disponibile ({ $detail }); si tiene il livello salvato
warning-feinkartierung-layer-skipped = Feinkartierung { $file } non disponibile ({ $detail }); livello saltato
error-extra-layer-unknown = non è un livello di { $folder }
warning-berlin-buildings-missing = nessun edificio ALKIS salvato per quest'area a Berlino; esegui `freemapper fetch` per ottenerli dal Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = nessuna struttura ALKIS completa salvata; esegui `freemapper fetch` online per muri, recinzioni, siepi e ponti
progress-nas-removing = rimozione dei livelli ALKIS completi salvati: l'area della carta non è più in un Land con un servizio NAS
warning-nas-folder-outside = livelli ALKIS completi saltati ({ $folder } esce dalla cartella del progetto)
progress-nas-download = scaricamento delle strutture ALKIS complete (NAS)
progress-nas-features = { $count } oggetti ALKIS NAS { $file }
progress-nas-skipped = saltati { $count } oggetti ALKIS NAS { $file }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } non disponibile ({ $detail }); si tiene il livello salvato
warning-nas-layer-skipped = ALKIS NAS { $file } non disponibile ({ $detail }); livello saltato

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nessun oggetto dell'ATKIS Basis-DLM salvato; esegui `freemapper fetch` online per sorgenti, grotte, torri, monumenti e fontane
progress-dlm-removing = rimozione dei livelli dell'ATKIS Basis-DLM salvati: l'area della carta non è più in un Land che lo fornisce
warning-dlm-folder-outside = livelli dell'ATKIS Basis-DLM saltati ({ $folder } esce dalla cartella del progetto)
progress-dlm-download = scaricamento degli oggetti dell'ATKIS Basis-DLM
progress-dlm-features = { $count } oggetti ATKIS Basis-DLM { $file }
progress-dlm-skipped = saltati { $count } oggetti ATKIS Basis-DLM { $file }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } non disponibile ({ $detail }); si tiene il livello salvato
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } non disponibile ({ $detail }); livello saltato

## Tree cadastres

progress-trees-none-stored = nessun catasto degli alberi cittadino salvato; esegui `freemapper fetch` online per i singoli alberi
progress-trees-removing = rimozione dei catasti degli alberi salvati: l'area della carta non è più in una città che ne abbia uno
warning-trees-folder-outside = catasti degli alberi cittadini saltati ({ $folder } esce dalla cartella del progetto)
progress-trees-download = scaricamento dei catasti degli alberi cittadini
progress-trees-count = { $count } alberi { $id }
progress-trees-skipped = saltati { $count } alberi { $id }: { $list }
warning-trees-kept = catasto degli alberi { $id } non disponibile ({ $detail }); si tiene quello salvato
warning-trees-skipped = catasto degli alberi { $id } non disponibile ({ $detail }); alberi saltati

## Deriving the templates

progress-writing = scrittura di { $path }
progress-contours-steep = curve di livello ogni { $interval } m: il terreno è così ripido che curve di livello ogni { $default } m si sovrapporrebbero; un'equidistanza di { $default } m nel progetto le mantiene comunque (`--contour-interval { $default }`)
warning-osm-only-no-terrain = ancora nessun dato del terreno per questa regione: la carta è fatta solo con OpenStreetMap, senza curve di livello, template ed edifici
progress-no-ndsm-recorded = nessun nDSM registrato; esegui `freemapper fetch` per il template nDSM
warning-ndsm-tile-invalid = riquadro nDSM { $tile } non valido in freemapper.toml; saltato
warning-ndsm-not-cached = nDSM { $tile } non nella cache; esegui `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } illeggibile ({ $error }); elimina { $path } ed esegui `freemapper fetch`
warning-surface-unusable = modello di superficie inutilizzabile ({ $error }); esegui `freemapper fetch`

## Downloads and the cache

progress-downloading = scaricamento di { $url }
progress-downloaded = scaricati { $megabytes } MB
progress-downloaded-of = scaricati { $megabytes } MB di { $total } MB
progress-tile-index-download = scaricamento dell'indice dei riquadri { $product }
progress-tile-cached = nella cache: { $tile }
progress-surface-blocks = lettura di { $count } dei { $total } blocchi del riquadro del modello di superficie { $tile } ({ $megabytes } MB)
progress-surface-whole = il riquadro del modello di superficie { $tile } viene scaricato per intero: { $reason }
warning-tile-cached-older = si usa il riquadro { $product } { $tile } nella cache del { $published } (il più recente del { $newer } non è stato scaricato)
warning-ndsm-no-cached = nessun nDSM nella cache per il riquadro { $tile }
warning-ndsm-tile-unavailable-cached = riquadro nDSM { $tile } non disponibile ({ $reason }); si usa il file nella cache
warning-ndsm-tile-unavailable-skipped = riquadro nDSM { $tile } non disponibile ({ $reason }); saltato
warning-file-check-failed = impossibile verificare se { $file } è ancora aggiornato ({ $reason }); si usa il file nella cache, e gli altri file nella cache di questo server senza chiedere
warning-ldbv-no-buildings-file = nessun file degli edifici per il riquadro { $tile } sul server; alla carta possono mancare edifici lì
progress-ndsm-not-published = nessuna nuvola di punti nDSM pubblicata per il riquadro { $tile }
progress-ndsm-cached = nDSM { $tile } nella cache
progress-point-cloud-not-published = nessuna nuvola di punti laser pubblicata per il riquadro { $tile }
progress-point-cloud-cached = nuvola di punti { $tile } nella cache
warning-point-cloud-no-cached = nessuna nuvola di punti nella cache per il riquadro { $tile }; lì nessun candidato di vegetazione
warning-point-cloud-unavailable-cached = nuvola di punti { $tile } non disponibile ({ $reason }); si usa il file nella cache
warning-point-cloud-unavailable-skipped = nuvola di punti { $tile } non disponibile ({ $reason }); lì nessun candidato di vegetazione
warning-point-clouds-unavailable = le nuvole di punti laser non sono disponibili ({ $reason }); i candidati di vegetazione usano i riquadri nella cache, se ce ne sono
progress-no-point-clouds = ancora nessuna nuvola di punti laser per la vegetazione qui: il Land non ne pubblica che freemapper sappia leggere, oppure non nel sistema di coordinate della carta
warning-point-clouds-not-fetched = la vegetazione è stata scelta, ma non sono caricate nuvole di punti; esegui `freemapper fetch` per caricarle
warning-point-cloud-unreadable = nuvola di punti { $tile } illeggibile ({ $error }); elimina { $path } ed esegui `freemapper fetch`
warning-osm-nrw-nearest = nessun estratto del circondario della Renania Settentrionale-Vestfalia entro 100 m dall'area; si legge il più vicino, { $district }
warning-osm-disk-full-cached = il disco è pieno, quindi freemapper usa l'estratto OSM nella cache del { $date }. Libera dello spazio ed esegui di nuovo `freemapper fetch` per averne uno più recente
warning-osm-geofabrik-unavailable = Geofabrik non disponibile ({ $error }); si usa l'estratto OSM nella cache del { $date }
progress-osm-extract-cached = estratto OSM { $region } del { $date } nella cache
progress-osm-extract-download = scaricamento di { $url } (circa { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = scaricamento del riquadro bDOM { $tile } ({ $megabytes } MB, { $index } di { $total })
progress-sh-dgm-not-in-atom = il riquadro DGM1 { $tile } non è nella directory INSPIRE ATOM; scaricamento del suo XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = il riquadro DGM1 { $tile } della directory INSPIRE ATOM non si legge ({ $error }); scaricamento del suo XYZ ({ $megabytes } MB)
progress-sh-tile = riquadro { $tile }: { $text }
progress-sh-download-again = riquadro { $tile }: il download non si legge ({ $message }); viene scaricato di nuovo
warning-sh-portal-no-date = il Downloadportal non dà una data per il riquadro { $model } { $tile }
warning-sh-dgm-xyz-used = il riquadro DGM1 { $tile } non è nella directory INSPIRE ATOM; si è usato il suo XYZ dal Downloadportal
warning-sh-dgm-xyz-used-unreadable = il riquadro DGM1 { $tile } della directory INSPIRE ATOM non si legge ({ $error }); si è usato il suo XYZ dal Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, riquadro { $tile }: { $count } celle dell'area della carta non hanno quota
warning-centre-tile-missing = il riquadro { $model } { $tile } al centro dell'area della carta manca, quindi non c'è una data
warning-centre-tile-unlisted = l'elenco dei riquadri non nomina alcun riquadro { $model } { $tile } al centro dell'area della carta, quindi non c'è una data
warning-list-unreadable-kept = impossibile leggere l'elenco dei riquadri del { $model } ({ $error }); si è usato quello conservato da un'esecuzione precedente
warning-portal-list-unreadable-kept = impossibile leggere l'elenco dei riquadri del { $model } del Downloadportal ({ $error }); si è usato quello conservato da un'esecuzione precedente
warning-surface-much-older = il { $model } è del { $surface }, l'ortofoto del { $flight }. Nell'nDSM possono mancare edifici e alberi più recenti.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } oggetti dell'elenco dei riquadri non hanno una chiave di riquadro utilizzabile; saltati
warning-rp-list-unreadable-kept = impossibile leggere l'elenco dei riquadri di LVermGeo RP ({ $error }); si è usato quello conservato da un'esecuzione precedente
warning-rp-metalink-not-kept = il Metalink di LVermGeo RP non è stato conservato: { $error }
warning-rp-metalink-unreadable-kept = impossibile leggere il Metalink di LVermGeo RP del { $model } ({ $detail }); si è usato quello conservato da un'esecuzione precedente
warning-rp-metalink-unreadable = impossibile leggere il Metalink di LVermGeo RP del { $model } ({ $detail }), quindi i riquadri per cui il suo elenco non dà un anno di volo restano senza quote
warning-rp-tile-without-name = { $model }, riquadro { $tile }: questo quadrato di 1 km non ha quote nella carta. L'elenco dei riquadri di LVermGeo RP (Laserbefliegung) { $note } per esso, quindi il nome del file non è noto{ $either ->
        [metalink] , e nemmeno il suo Metalink nomina un file per esso
       *[none] {""}
    }
warning-rp-centre-no-date = il riquadro { $model } { $tile } non ha una data: l'elenco dei riquadri di LVermGeo RP (Laserbefliegung) { $note } per esso
word-rp-list-neither = non dà né un anno di volo né giorni di volo
word-rp-list-no-days = non dà giorni di volo
word-rp-list-no-year = non dà un anno di volo
word-rp-list-days-unreadable = dà giorni di volo { $days } che non si possono leggere
word-rp-list-disagree = dà giorni di volo { $days } e un anno di volo { $year } che non concordano su un solo anno

## Hesse

warning-hvbg-undated = l'elenco dei metadati non ha un Bildflugdatum leggibile per il riquadro { $tiles }; i loro record di progetto non hanno una data di volo
warning-hvbg-undated-more = l'elenco dei metadati non ha un Bildflugdatum leggibile per il riquadro { $tiles } e per altri { $more }; i loro record di progetto non hanno una data di volo
warning-hvbg-list-unreadable-older = impossibile leggere il nuovo elenco dei metadati HVBG { $name } ({ $detail }); si usa l'elenco più vecchio { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: nessuna data, lo STAC non elenca alcun riquadro al centro dell'area della carta
warning-lgln-no-date = { $model }: nessuna data ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; il modello salvato mantiene la data registrata
warning-mv-cells-empty = { $model }: { $count } celle all'interno dei riquadri del Meclemburgo-Pomerania Anteriore non hanno quota
warning-mv-centre-no-date = { $model }: nessuna data per il riquadro centrale { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } righe dell'elenco dei riquadri del { $model } non hanno un nome di riquadro utilizzabile; saltate
warning-nrw-list-rows-undated = { $count } righe dell'elenco dei riquadri del { $model } non hanno una data leggibile
warning-nrw-list-older = l'elenco dei riquadri del { $model } sul server è del { $server }, più vecchio di quello conservato del { $kept }; si è usato quello conservato
warning-nrw-list-no-date = l'elenco dei riquadri non dà una data per il riquadro { $model } { $tile }

## After `init`

note-scale-outside-norm = nota: 1:5000 non è nella norma sprint; freemapper-print lo segnala e i simboli restano al 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } non è un id OSM come way/992423119
error-place-invalid-answer = risposta non valida dalla ricerca del luogo: { $detail }
error-place-not-found = { $id } non trovato. Nominatim elenca solo oggetti OSM con tag, quindi un way esterno senza tag di un multipoligono non c'è. Usa invece la relazione multipoligono (--osm relation/<id>), oppure salva il contorno come GeoJSON e usa --geojson
error-place-is-a-point = { $id } è un punto, non un'area; scegli il suo terreno (un way o una relazione)
error-place-no-such-part = { $id } ha { $count ->
        [one] 1 parte
        [many] { $count } di parti
       *[other] { $count } parti
    }; non esiste la parte { $part }
error-place-several-parts = { $id } è un'area di { $count } parti separate ({ $sizes }). Un'area della carta è un solo poligono: scegli una parte con --part <n> (1 è la più grande), oppure salva il contorno che vuoi come GeoJSON e usa --geojson
error-layer-not-a-world-file = { $path }: non è un world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = nessuna anteprima per { $layer }; livelli con anteprima: { $names }
error-layer-no-ndsm-published = per i riquadri di quest'area della carta non è pubblicato alcun nDSM, quindi non c'è un template ndsm
error-layer-not-made = { $layer } non è ancora stato creato; esegui { $step ->
        [fetch] freemapper fetch o run (MCP: start_pipeline); solo Berlino
        [export] freemapper export o run (MCP: start_pipeline)
       *[derive] freemapper derive o run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; la carta OCAD non ottiene da qui alcuno sfondo
warning-map-preview-failed = la carta è scritta, ma manca la sua anteprima: { $error }
warning-training-no-tree-cover = la carta di allenamento tiene intere le aree aperte dei parchi: manca la copertura arborea (data/tree-cover.tif); esegui di nuovo `freemapper derive`
