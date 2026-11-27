# Program texts of freemapper, Italian (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } non è una lingua per cui freemapper ha dei testi; disponibili: { $available }

## init

cli-part-needs-osm = --part sceglie una parte di un'area OSM e richiede --osm; con --place cerca prima l'id e usa --osm <id> --part <n>
cli-reading-file = lettura di { $path }
cli-no-area-for-place = nessuna area trovata per "{ $query }". Se il luogo è un'area composta da più parti separate, scegline una con --osm <id> --part <n>, oppure usa --geojson
cli-using-place = Uso { $name } ({ $id })
cli-created-project = Progetto creato: { $path }

## Cache

cli-no-cache-dir = nessuna cartella cache su questo sistema; passa --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Scaricato 1 riquadro
        [many] Scaricati { $count } di riquadri
       *[other] Scaricati { $count } riquadri
    }
cli-wrote = Scritto { $path }
cli-kept-map = Conservato { $path } (può contenere il tuo lavoro)
cli-map-lacks-templates = La carta non fa ancora riferimento ai template immagine. Aggiungili in OOM (Templates > Open template…) oppure sostituisci la carta con --force, che conserva quella vecchia come .omap.bak.
cli-map-lacks-buildings = La carta non ha ancora gli edifici ALKIS. Per aggiungerli, sostituiscila con --force, che conserva quella vecchia come .omap.bak.
cli-map-lacks-canopies = Alla carta mancano le tettoie e i carport ALKIS (522), che ora sono oggetti della carta. Per aggiungerli, esegui con --force, che conserva la vecchia carta come .omap.bak.
cli-map-light = La carta base: { $joined } linee unite, { $left } nascoste o troppo piccole lasciate ai livelli candidati (nulla è stato spostato)
cli-map-light-kept = I livelli candidati ora contengono anche { $left } oggetti o tratti che questa versione toglie dalla carta base (con il tag freemapper:generalized). La carta conservata li ha ancora, quindi compaiono due volte. Sostituisci la carta con --force per ripulirla (quella vecchia resta come .omap.bak).
cli-map-old-candidates = La carta nomina ancora templates/candidates.omap; questa esecuzione lo ha riempito con tutti i candidati. I candidati ora sono anche in sette livelli, un file ciascuno (templates/candidates-*.omap). La carta li elenca dopo che l'hai sostituita con --force (la vecchia resta come .omap.bak), oppure aggiungili tu in OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF senza crediti delle fonti: ODbL e dl-de/by-2-0 chiedono di citare le fonti quando passi la carta ad altri; vedi ATTRIBUTION.txt
cli-wrote-pdf = Scritto { $path } (la carta senza candidati)
cli-wrote-pdf-candidates = Scritto { $path } (la stampa di controllo: la carta con ogni candidato, fedele alla posizione; controllali sul terreno)
cli-wrote-training = Scritti { $map } e { $pdf } (la carta di allenamento: { $joined } unite, { $shifted } spostate, { $left } omesse o tagliate)
cli-training-free-ends = Estremità di linea lasciate libere: { $ways } dove un sentiero incontrava un altro sentiero, { $others } presso altre linee
cli-training-apart-left = la carta di allenamento ha ancora { $length } mm (su carta) dove gli oggetti hanno troppo poco spazio tra loro; controllala in OOM
cli-kept-training = Conservata la carta di allenamento { $path }, così che un'esecuzione non sovrascriva una carta già stampata o consegnata; eliminala o passa --force per rinnovarla

## OCAD

cli-ocad-kept = Conservato { $path } (può contenere il tuo lavoro; --force lo sostituisce e conserva quello vecchio come .ocd.bak)
cli-ocad-old-candidates = La carta OCAD nomina ancora { $path } come carta di sfondo; questa esecuzione l'ha riempita con tutti i candidati. I candidati ora sono anche in sette livelli, un file ciascuno (-candidates-*.ocd). La carta OCAD li elenca dopo che l'hai sostituita con --force (la vecchia resta come .ocd.bak), oppure aggiungili tu in OCAD come carte di sfondo.
cli-ocad-wrote = { $count ->
        [one] Scritto { $path } (1 oggetto)
        [many] Scritto { $path } ({ $count } di oggetti)
       *[other] Scritto { $path } ({ $count } oggetti)
    }
cli-ocad-left-out = Esclusi dalla carta OCAD: { $skipped } oggetti (nessun simbolo per OCAD, nessuna superficie, o fuori dall'area di disegno di OCAD di ±2 m) e { $dropped } parti di linea o buchi degeneri.
cli-ocad-load-symbols = In OCAD scegli Symbol > Load Colors and Symbols From… e seleziona il set di simboli ufficiale{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , il set scuola IOF 2019
       *[other] {" "}ISSprOM 2019
    }; sostituisce i simboli segnaposto di freemapper in base al numero.

## select

cli-select-running = Selezione dell'area per { $path }: premi "Fatto" sulla pagina oppure Ctrl+C per interrompere.
cli-select-no-browser = Impossibile aprire il browser ({ $error }); apri l'indirizzo qui sopra.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; nessuna area, non può essere un'area della carta): { $place }
cli-find-hint = Crea il progetto con: freemapper init <dir> --osm { $id }
cli-find-no-area = Nessuno di questi luoghi ha un'area che possa essere un'area della carta. Disegna l'area con: freemapper select <dir>
cli-find-nothing = nessun luogo trovato per "{ $query }"; prova un altro nome oppure disegna l'area con: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = avviso: l'adattatore in { $folder } è escluso: { $finding }
cli-adapter-note = avviso: l'adattatore in { $folder } viene caricato, ma: { $finding }
cli-adapter-from-project = avviso: viene usato l'adattatore { $adapter } della cartella del progetto; può interrogare questi host: { $hosts }
cli-adapter-overlaps = avviso: l'adattatore { $adapter } copre anche il territorio di { $others }; i suoi oggetti finiscono sulle carte di quella zona
cli-adapter-dir-relative = avviso: FREEMAPPER_ADAPTER_DIR non è un percorso assoluto ({ $path }); viene ignorata
cli-adapters-no-project = avviso: { $path } non è una cartella; non sono stati letti adattatori di un progetto
cli-adapters-export-unknown = l'adattatore { $adapter } non è descritto da nessun manifesto di questo programma, quindi non c'è nulla da esportare; `freemapper adapters list` mostra gli id
cli-adapters-export-taken = { $path } esiste e non è vuota; esporta in una cartella che non esiste ancora
cli-adapters-export-done = I { $count } file dell'adattatore { $adapter } sono stati scritti in { $path }. Contengono solo ciò che descrive il suo manifesto; gli indirizzi scritti nel codice del programma non ci sono.
cli-adapters-export-next = Modifica gli indirizzi in manifest.toml, controlla la cartella con `freemapper adapters check` e spostala in { $path }: dal prossimo avvio sostituisce l'adattatore integrato, con un avviso. Eliminando la cartella torna quello integrato.
cli-adapters-overlaps = condivide il territorio con: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revisione { $revision }  verificato { $checked }  host: { $hosts }
cli-adapters-source-built-in = integrato
cli-adapters-source-user = dalla tua cartella
cli-adapters-source-project = dal progetto
cli-adapters-never-checked = mai
cli-adapters-user-dir = I tuoi adattatori vengono letti da { $path }
cli-adapters-no-user-dir = Questo sistema non ha una cartella per i tuoi adattatori; FREEMAPPER_ADAPTER_DIR ne indica una

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper può ora interrogare anche { $host } per { $name }. La voce è in { $file }; `freemapper adapters disallow-host { $adapter } { $host }` la revoca. Una pagina dell'area o un assistente IA in esecuzione legge la voce al riavvio
cli-allow-host-already = { $host } è già consentito per { $name }
cli-allow-host-declared = l'adattatore di { $name } nomina già { $host }; non c'è nulla da consentire
cli-allow-host-refused = { $host } non può essere consentito: { $reason }
cli-allow-host-unknown-adapter = non esiste un adattatore integrato { $adapter }; gli adattatori integrati sono: { $adapters }
cli-allow-host-folder-adapter = { $adapter } è un adattatore della tua cartella degli adattatori; aggiungi invece l'host a `[net] hosts` del suo manifest
cli-allow-host-no-config = questo sistema non ha una cartella di configurazione in cui conservare gli host consentiti
cli-allow-host-write = impossibile scrivere { $file }: { $detail }
cli-allow-host-unreadable = impossibile leggere { $file } ({ $detail }); correggi o rimuovi prima il file; non è stato modificato nulla
cli-disallow-host-removed = freemapper non interroga più { $host } per { $name }
cli-disallow-host-absent = { $host } non era consentito per { $name }; non c'è nulla da revocare
cli-allowed-hosts-unreadable = avviso: impossibile leggere { $file } ({ $detail }); nessun host aggiuntivo è consentito
cli-allowed-hosts-ignored = avviso: { $file }: la voce { $host } per { $adapter } viene ignorata: { $reason }
cli-allowed-hosts-no-adapter = freemapper non ha un adattatore integrato con quel nome
cli-allowed-host-redundant = avviso: { $host } è ora uno degli host nominati dall'adattatore di { $name }; la tua voce in { $file } è stata rimossa
cli-adapters-allowed = consentito da te: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adattatore { $adapter } ({ $name }), revisione { $revision }, host: { $hosts }
cli-check-error = errore: { $finding }
cli-check-warning = avviso: { $finding }
cli-check-passed = { $folder } supera il controllo
cli-check-passed-warnings = { $folder } supera il controllo, con { $count ->
        [one] un avviso
        [many] { $count } di avvisi
       *[other] { $count } avvisi
    }
cli-check-failed = { $folder } non supera il controllo
cli-check-no-folder = la cartella { $folder } non esiste
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } FALLITO: { $detail }
cli-check-live-not-asked = { $capability } non interrogato: { $detail }
cli-check-live-box-outside = il riquadro non si trova entro il confine dell'adattatore { $adapter }; un controllo live interroga i servizi per un luogo coperto dall'adattatore
cli-check-live-cannot = il controllo live non può essere eseguito: { $detail }
cli-check-detail-not-declared = l'adattatore non ne dichiara
cli-check-detail-stopped = una parte precedente è fallita
cli-check-detail-see-warnings = vedi gli avvisi
cli-check-detail-outside-dataset = il riquadro si trova fuori dal set di dati
cli-check-detail-objects = { $count ->
        [one] un oggetto
        [many] { $count } di oggetti
       *[other] { $count } oggetti
    }
cli-check-detail-layers = { $layers } livelli, { $count } oggetti
cli-check-detail-trees = { $count ->
        [one] un albero
        [many] { $count } di alberi
       *[other] { $count } alberi
    }
cli-check-detail-extract = { $label }, dati del { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = modello del terreno
cli-check-raster-surface = modello di superficie
cli-check-raster-tiles = { $product }: { $count ->
        [one] un riquadro
        [many] { $count } di riquadri
       *[other] { $count } riquadri
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } e { $last }

## Folder lock

cli-lock-held = un'altra esecuzione di freemapper sta usando questa cartella (dalle { $time }, avviata da { $who }). Aspetta che finisca o fermala prima.
cli-lock-who-cli = il terminale
cli-lock-who-mcp = un agente IA (MCP)
cli-lock-who-page = la pagina di selezione
cli-lock-who-unknown = un altro programma
cli-lock-failed = impossibile bloccare la cartella del progetto per questa esecuzione: { $error }
cli-lock-stuck = qualcosa che non è un file di blocco è d'intralcio e non ha potuto essere rimosso

## Run routes of the selection page

cli-run-bad-request = Le opzioni di esecuzione non sono valide: { $error }
cli-run-no-project = In questa cartella non c'è ancora un progetto salvato. Salva prima l'area.
cli-run-already-running = Per questo progetto è già in corso un'esecuzione ({ $job }). Aspetta che finisca o annullala prima.
cli-run-disk-stops = Spazio libero su disco insufficiente per il download: { $messages }
cli-run-busy-elsewhere = Un altro progetto di questo programma sta eseguendo una pipeline in questo momento. I progetti condividono la cache dei download, quindi si esegue un solo lavoro alla volta. Riprova quando ha finito.
cli-run-nothing-to-cancel = Nessuna esecuzione in corso, quindi non c'è nulla da annullare.
cli-run-unknown-job = Per questo progetto non esiste l'esecuzione { $job }.
cli-status-too-fast = Le richieste di stato sono troppo frequenti. Chiedi al massimo cinque volte al secondo.
cli-open-folder-failed = Impossibile aprire la cartella ({ $error }).
cli-open-folder-no-opener = Questo computer non ha un programma per aprire le cartelle ({ $program }). Apri la cartella a mano.
cli-open-folder-missing = La cartella del progetto non esiste ancora. Salva prima l'area.
cli-open-folder-refused = Il nome della cartella non è sicuro da passare al file manager.
cli-save-running = Per questo progetto è in corso un'esecuzione. Salva di nuovo quando ha finito.
cli-save-unknown-kind = Tipo di carta sconosciuto "{ $kind }". Usa schoolyard, sprint o forest.
cli-save-scale-too-large = La scala { $scale } è troppo grande.
cli-save-bad-content = Le impostazioni dei contenuti non sono valide. Non è stato salvato nulla.
cli-map-image-none = Crea prima la carta.
cli-map-image-gone = Questa anteprima non è più disponibile.
cli-map-image-running = Per questo progetto è in corso un'esecuzione. L'anteprima si può calcolare quando ha finito.
cli-map-image-stale = Il progetto salvato non corrisponde più alla carta generata, ad esempio perché sono cambiati l'area, il tipo di carta o la scala. Avvia prima un'esecuzione che sostituisca la carta ("Sostituisci la carta esistente" sulla pagina, oppure freemapper run --force).
cli-map-image-superseded = Nel frattempo è stata richiesta un'anteprima più recente, quindi questa non è stata calcolata.
cli-map-image-bad-content = Le impostazioni dei contenuti non sono valide.
cli-map-image-bad-request = La richiesta di anteprima non è valida. Invia un oggetto JSON, con le impostazioni dei contenuti se presenti.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – selezione dell'area
page-heading = Scegli l'area della carta
page-update-available = freemapper { $version } è disponibile. Aggiorna con
page-update-notes = Note di rilascio
page-folder = Cartella del progetto
page-pick = Cerca un luogo
page-query-placeholder = es. Clara-Zetkin-Schule Strausberg
page-search-button = Cerca
page-searching = Ricerca in corso …
page-no-results = Nessun risultato.
page-point-only = solo un punto, non un'area
page-draw = Disegnala tu
page-rectangle = Rettangolo
page-polygon = Poligono
page-finish-polygon = Chiudi poligono
page-clear = Cancella
page-hint-rectangle = Clicca due angoli opposti sulla mappa.
page-hint-polygon = Clicca gli angoli uno dopo l'altro. Per chiuderlo, clicca il primo angolo, fai doppio clic o clicca "Chiudi poligono". Esc annulla, Backspace toglie l'ultimo angolo.
page-hint-idle = Scegli un luogo dalla ricerca o disegna tu l'area.
page-summary = Area
page-source-osm = Da OpenStreetMap: { $name }
page-source-drawn = Disegnata a mano
page-size = Dimensione con margine: { $width } × { $height } m
page-tiles-one = { $n } riquadro: { $names }
page-tiles-other = { $n } riquadri: { $names }
page-download = Download: circa { $mb } MB
page-download-none = Download: è già tutto nella cache
page-download-details = Cosa viene scaricato
page-cached = nella cache
page-disk = Spazio su disco: { $volumes }
page-disk-volume = { $free } liberi, ne servono circa { $needed }
page-cache-volume = cache
page-project-volume = cartella del progetto
page-no-room = non c'è spazio sul disco
page-no-room-skip = Non c'è spazio sul disco per tutto (freemapper tiene { $reserve } liberi). Il download omette le parti contrassegnate. Libera prima dello spazio se ti servono:
page-no-room-stop = Spazio su disco insufficiente (freemapper tiene { $reserve } liberi). Senza le parti contrassegnate non c'è carta, quindi il download si ferma prima di scaricare qualsiasi cosa. Libera prima dello spazio:
page-disk-unknown = freemapper non riesce a leggere lo spazio libero su disco ({ $error }). Il download procede senza il controllo.
page-too-large = L'area è troppo grande: al massimo { $max } m per lato (con margine). Scegline una più piccola.
page-outside = L'area si trova fuori dai Land supportati ({ $states }); freemapper non ha ancora fonti di dati per altre regioni.
page-outside-unknown = L'area si trova fuori dalle regioni supportate da freemapper; lì non ha ancora fonti di dati.
page-adapters-failed = Non è possibile elencare le foto aeree ({ $detail }). La mappa mostra solo OpenStreetMap; tutto il resto funziona.
page-settings = Carta
page-kind = Tipo di carta
page-schoolyard = Carta da cortile scolastico (1:1000)
page-sprint = Carta sprint (1:4000)
page-forest = Carta da bosco (1:15000)
page-scale = Scala 1:
page-map-title = Titolo
page-contour-interval = Equidistanza delle curve di livello
page-contour-auto = automatica
page-steep-slope = Soglia di pendenza
page-steep-slope-auto = automatica (27°)
page-steep-slope-hint = Da questo angolo di pendenza in poi il terreno è considerato ripido nella ricerca di scarpate e pareti rocciose. 27° (automatica) va bene per colline, parchi e città. Nelle città di roccia, nelle gole e nelle cave quasi ogni pendio è ripido, quindi scarpate e pareti si fondono in un'unica macchia e nessuna viene trovata; da 55° a 65° le separa. Un valore troppo alto conserva solo le pareti più ripide.
page-face-slope = Soglia di parete
page-face-slope-auto = automatica (70°)
page-face-slope-hint = Da questo angolo di pendenza in poi il terreno è disegnato come superficie nera di parete rocciosa. 70° (automatica) va bene per le città di roccia come Adršpach (5,2 % della superficie). 65° dà più superfici nere e più grandi, anche sui pendii ripidi (8,4 %). 75° lascia solo vere pareti, con bordi frastagliati (2,5 %). Un valore troppo alto lascia quasi nessuna superficie.
page-settings-fixed = Il progetto esiste già; tipo di carta, scala e titolo sono in freemapper.toml. L'equidistanza delle curve di livello si può ancora cambiare.
page-save = Salva
page-saving = Salvataggio …
page-saved = Salvato in freemapper.toml.
page-zone-changed = Salvato. L'area ora si trova in un'altra zona UTM; freemapper scarica di nuovo i suoi dati alla prossima esecuzione.
page-provider-changed = Salvato. Un altro fornitore di dati serve la quota e le immagini aeree della nuova area; freemapper scarica di nuovo i suoi dati alla prossima esecuzione.
page-save-stale = freemapper.toml è stato modificato in un'altra finestra o da un altro programma da quando questa pagina lo ha letto. Non è stato salvato nulla; la pagina ora conosce lo stato salvato. Salva di nuovo per sostituirlo con l'area mostrata qui.
page-has-map = La carta esistente (.omap) resta com'è; freemapper crea nuovi template e candidati per la nuova area. L'opzione "Sostituisci la carta esistente" qui sotto (o "freemapper run --force") rifà anche la carta e conserva quella vecchia come .omap.bak.
page-done = Fatto
page-source = Codice sorgente (AGPL-3.0-or-later)
page-closed = Fatto. Puoi chiudere questa finestra.
page-next = Poi: freemapper run "{ $dir }"
page-made-files = Creati: { $files }. Cartella: { $dir }
page-back-to-agent = Di' al tuo assistente IA che l'area è salvata.
page-nothing-saved = Non è stato salvato nulla.
page-failed = Errore: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Foto aerea { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Cosa finisce sulla carta
page-content-help = Il profilo decide cosa finisce direttamente sulla carta e cosa resta un candidato da controllare. La scelta per un gruppo prevale sul profilo. Viene salvata con l'area.
page-content-profile-legend = Profilo
page-content-profile-safe = Sicuro
page-content-profile-safe-hint = Ogni oggetto resta dove lo mette la generazione: gli edifici e le superfici del catasto vanno sulla carta; muri, recinzioni, alberi e molto di OpenStreetMap (stagni, sentieri senza fondo noto) restano candidati.
page-content-profile-balanced = Equilibrato
page-content-profile-balanced-hint = Gli oggetti del catasto, dei registri e del rilievo stradale vanno sulla carta, con le curve di livello e le aree vietate lungo i binari. Muri, recinzioni, siepi, scarpate, scale, acqua, alberi, monumenti, passaggi, sentieri e ferrovie vengono anche da OpenStreetMap. L'arredo urbano, le altre aree vietate e le forme del modello del terreno restano candidati.
page-content-profile-full = Completo
page-content-profile-full-hint = Tutto va sulla carta, come nella stampa di controllo; restano fuori solo i candidati nascosti.
page-content-groups-legend = Per gruppo
page-content-groups-help = "Come nel profilo" segue il profilo. Cosa significhi per un gruppo dipende da dove viene ogni oggetto.
page-content-place-profile = Come nel profilo
page-content-place-map = Carta
page-content-place-candidates = Candidato
page-content-place-off = Disattivato
page-content-group-walls = Muri
page-content-group-fences = Recinzioni
page-content-group-hedges = Siepi
page-content-group-slopes = Scarpate e rocce
page-content-group-stairs = Scale
page-content-group-trees = Alberi e gruppi di alberi
page-content-group-water-features = Acqua, fontane e sorgenti
page-content-group-monuments = Torri, monumenti e linee elettriche
page-content-group-furniture = Arredo urbano e lampioni
page-content-group-passages = Passaggi
page-content-group-contours = Curve di livello
page-content-group-landforms = Collinette, depressioni, scarpate e rocce dal modello del terreno
page-content-group-vegetation = Vegetazione dalle nuvole di punti laser (Brandeburgo, Berlino, Renania Settentrionale-Vestfalia, Renania-Palatinato, Saarland e Baviera; disattivata finché non la scegli; scarica circa 40-600 MB per km²)
page-content-group-out-of-bounds = Aree vietate
page-content-group-paths = Sentieri e ferrovie
page-content-group-streets = Strade e piazze
page-run-heading = Crea la carta
page-preview-heading = Anteprima della carta
page-preview-stored = Come generata l'ultima volta.
page-preview-live = Con le impostazioni di questa pagina, non ancora salvate.
page-preview-live-saved = Con le impostazioni salvate, non ancora generata in un'esecuzione.
page-preview-computing = Calcolo in corso …
page-preview-none = Nessuna carta generata finora.
page-preview-later = Questa carta non ha ancora un'anteprima. Arriva con la prossima esecuzione che scrive la carta.
page-preview-stale = L'area, il tipo di carta o la scala sono cambiati da quando la carta è stata generata. L'immagine mostra ancora la carta come generata l'ultima volta; per una nuova anteprima serve un'esecuzione con "Sostituisci la carta esistente".
page-preview-area-unsaved = L'area su questa pagina non è ancora salvata. L'immagine mostra la carta come generata l'ultima volta; per la nuova area serve un'esecuzione con "Sostituisci la carta esistente".
page-preview-run = Come generata l'ultima volta. La pagina aggiorna l'immagine quando l'esecuzione finisce.
page-preview-open = Apri a grandezza naturale
page-preview-alt = Anteprima degli oggetti della carta
page-opt-ocad = Scrivi un progetto di partenza OCAD
page-help-ocad = Per chi prosegue in OCAD.
page-opt-pdf = PDF stampabile
page-help-pdf = Solo gli oggetti della carta, senza candidati.
page-opt-pdf-candidates = Stampa di controllo con ogni candidato
page-opt-training = Carta di allenamento
page-help-training = La carta e ogni candidato, ripuliti per essere stampati così come sono; con OCAD anche come file OCAD.
page-views-heading = Viste della carta di allenamento
page-view-no-paths = Senza sentieri (carta Ultimate)
page-view-brown-only = Solo marrone (orienteering a curve di livello)
page-view-no-north-lines = Senza linee del nord
page-view-custom = Un'altra vista
page-help-views = Ogni vista selezionata è un file in più accanto alla carta di allenamento, `<name>-training-<view>.omap` e il suo PDF, con OCAD anche un file OCAD. Un'altra vista è `no:505,506` (omette simboli) oppure `only:101,102` (tiene solo questi); `+` combina le viste. Titolo, scala e fonti restano sempre.
page-help-views-forest = Senza sentieri e solo marrone valgono solo per le carte da bosco.
page-opt-scale-bar = Barra della scala sui PDF
page-help-scale-bar = Le carte tedesche di solito non ce l'hanno.
page-opt-sources = Righe delle fonti sui PDF
page-sources-auto = automatiche
page-sources-full = complete
page-sources-credit-only = solo freemapper
page-warn-credit-only = In questo caso i PDF citano solo freemapper. Le licenze dei dati (ODbL, dl-de/by-2-0) richiedono l'attribuzione quando passi la carta ad altri; le fonti devi quindi citarle tu. ATTRIBUTION.txt le elenca tutte.
page-need-pdf = La barra della scala e le righe delle fonti valgono solo se si crea un PDF.
page-opt-offline = Solo cache, nessuna rete
page-opt-force = Sostituisci le carte esistenti
page-help-force = La carta e, con "Carta di allenamento" selezionata, la carta di allenamento. I vecchi file restano come .omap.bak.
page-opt-permits = Note sui permessi
page-help-permits = Unità amministrative, aree protette e a chi chiedere, in permits.md. Un'indicazione, non una consulenza legale.
page-run-start = Crea la carta
page-run-again = Crea di nuovo
page-run-cancel = Annulla
page-run-cancelling = Annullamento …
page-warnings-heading = Avvisi
page-where-files = I file si trovano nella cartella del progetto:
page-open-folder = Apri la cartella
page-log-label = Registro dell'esecuzione
page-force-title = Sostituire le carte esistenti?
page-force-text = La carta esistente viene sostituita e, con "Carta di allenamento" selezionata, anche la carta di allenamento. Le tue modifiche non vanno perse: i vecchi file restano nella cartella del progetto come .omap.bak. Continuare?
page-force-cancel = Annulla
page-force-confirm = Sostituisci
page-step-fetch = Scaricamento dei dati
page-step-derive = Calcolo dei template
page-step-map = Scrittura della carta
page-step-ocad = Scrittura del progetto OCAD
page-step-pdf = Scrittura del PDF
page-step-pdf-candidates = Scrittura della stampa di controllo
page-step-training = Scrittura della carta di allenamento
page-step-permits = Scrittura delle note sui permessi
page-step-line = Passo { $index } di { $total }: { $label }
page-starting = L'esecuzione sta partendo …
page-file-kind-map = carta
page-file-kind-template = template
page-file-kind-ocad = carta OCAD
page-file-kind-ocad-candidates = candidati OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = stampa di controllo
page-file-kind-training-map = carta di allenamento
page-file-kind-training-pdf = PDF della carta di allenamento
page-file-kind-training-ocad = carta di allenamento OCAD
page-file-kind-training-view-map = vista della carta di allenamento
page-file-kind-training-view-pdf = PDF della vista della carta di allenamento
page-file-kind-training-view-ocad = vista OCAD della carta di allenamento
page-file-kind-preview = anteprima della carta
page-file-kind-permits = note sui permessi
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Fatto. I file sono stati scritti.
page-result-failed = L'esecuzione è fallita: { $error }
page-result-failed-kept = Ciò che è stato scritto resta nella cartella del progetto; una nuova esecuzione riprende da lì.
page-result-cancelled = Annullata. Ciò che è già stato scaricato resta nella cache; una nuova esecuzione riprende da lì.
page-result-title-done = Carta creata
page-result-title-failed = Esecuzione fallita
page-result-title-cancelled = Esecuzione annullata
page-kept-map = La carta esistente è stata lasciata com'è. "Sostituisci la carta esistente" la rifà e conserva quella vecchia come .omap.bak.
page-no-files = Non è stato creato alcun file.
page-opened-folder = La cartella è aperta.
page-open-failed = { $message } Cartella: { $dir }
page-why-no-area = Scegli prima un'area.
page-why-checking = L'anteprima è in calcolo …
page-why-too-large = L'area è troppo grande.
page-why-outside = L'area si trova fuori dai Land supportati.
page-why-disk = Spazio libero su disco insufficiente (vedi sopra).
page-why-running = È in corso un'esecuzione.
page-err-scale-bar-no-pdf = La barra della scala richiede un PDF.
page-err-sources-no-pdf = Le righe delle fonti richiedono un PDF.
page-save-failed = L'area non è salvata; non è stata avviata alcuna esecuzione.
page-poll-lost = Nessuna risposta dal server. freemapper continua a riprovare.
page-run-unknown = Il server non conosce più questa esecuzione.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Chiudi

## update (spec 3.4 to 3.6)

update-source-build = Questo freemapper è stato compilato dai sorgenti. Aggiornalo con git pull e cargo build --release --locked.
update-current = freemapper { $version } è aggiornato.
update-available = freemapper { $new } è disponibile (hai { $old }), rilasciato il { $released }.
update-details =
    Note di rilascio: { $notes }
    Download: { $size } MB
update-decimal-separator = ,
update-ask = Installarlo ora? [s/N]
update-not-installed = Non è stato installato nulla.
update-needs-yes = l'input non è un terminale, quindi freemapper non può chiedere prima di installare; esegui freemapper update --yes per installare senza chiedere
update-installing = scaricamento e verifica di freemapper { $version }
update-done = freemapper aggiornato da { $old } a { $new }. Note di rilascio: { $notes }
update-mcp-restart = Un server MCP in esecuzione mantiene la vecchia versione finché l'assistente IA non lo riavvia.
update-rolled-back = Tornato a freemapper { $version }. La versione che hai lasciato è conservata come { $path }.
update-not-writable = impossibile scrivere nella cartella del programma { $path }, e freemapper non chiede i diritti di amministratore. Decomprimi lì a mano la nuova versione, oppure sposta freemapper in una cartella in cui puoi scrivere.
update-too-old = freemapper { $new } non può essere installato dall'aggiornamento automatico della versione { $old }; installa a mano la versione { $needed } o una più recente, vedi { $notes }
update-busy = un altro aggiornamento o ripristino è in corso nella cartella del programma (file di blocco { $path }); aspetta che finisca e riprova
update-bad-signature = impossibile verificare la versione ({ $error }); non è stato installato nulla
update-check-bad-signature = impossibile verificare la versione più recente di freemapper ({ $error }); non è stato installato nulla
update-no-manifest = la versione più recente non può installarsi da sola (non ha un manifest di aggiornamento); scaricala da { $page }
update-no-manifest-version = la versione { $version } non esiste o non può installarsi da sola (non ha un manifest di aggiornamento); vedi { $page }
update-no-asset = la versione non ha un programma per questo sistema ({ $platform }); non è stato installato nulla
update-no-previous = non c'è una versione precedente a cui tornare ({ $path } manca)
update-failed = aggiornamento fallito: { $error }
update-mode-set = Modalità di aggiornamento impostata su { $mode } in { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } prevale su questa impostazione finché è impostata.
update-mode-failed = impossibile scrivere { $path }: { $error }
update-notice = freemapper { $new } è disponibile (hai { $old }). Aggiorna con: freemapper update
update-applies-next-start = La nuova versione si applica dal prossimo avvio di freemapper.
update-install-failed = freemapper { $new } non è stato installato: { $error }
update-no-config-dir = nessuna cartella di configurazione su questo sistema; imposta FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Nota per la persona: all'avvio il server freemapper ha trovato questo nelle cartelle degli adattatori. Riferiscilo; la persona corregge la cartella o la rimuove, poi riavvia il server.
    { $lines }
update-mcp-note = Nota per la persona: freemapper { $new } è disponibile (questa è { $old }). Chiedi alla persona prima di chiamare install_update.
update-mcp-needs-confirm = non è stato installato nulla; chiedi prima alla persona e chiama install_update con confirm: true quando è d'accordo
update-mcp-note-manual = Nota per la persona: freemapper { $new } è disponibile (questa è { $old }), ma install_update non può installarla; installa a mano la versione { $needed } o una più recente, vedi { $notes }
update-mcp-note-bad-signature = Nota per la persona: impossibile verificare la versione più recente di freemapper ({ $error }); non è stato installato nulla. Diglielo e non installarla in altro modo.
update-mcp-restart-now = Riavvia ora il server MCP di freemapper; esegue ancora la vecchia versione.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = errore
error-cancelled = annullato
cli-cancelled-kept = annullato; conservati: { $files }
error-io = { $path }: { $message }
error-io-disk-full = il disco è pieno, quindi freemapper non ha potuto scrivere { $path }. Libera dello spazio e riprova

## Errors: project

error-project-already-exists = il progetto esiste già: { $path }
error-project-map-exists = la carta esiste già (può contenere il tuo lavoro); usa --force per sostituirla: { $path }
error-project-invalid-scale = scala 1:{ $scale } non valida
error-project-invalid-contour-interval = equidistanza delle curve di livello { $interval } m non valida in freemapper.toml; deve essere maggiore di 0
error-project-invalid-steep-slope = soglia di pendenza { $slope }° non valida in freemapper.toml; deve essere compresa tra 15° e 80°
error-project-invalid-face-slope = soglia di parete { $slope }° non valida in freemapper.toml; deve essere compresa tra 55° e 85°
error-project-scale-not-allowed = la scala 1:{ $scale } non è consentita per le carte di tipo { $kind }; usa { $allowed }
error-project-scale-not-allowed-enlarge = la scala 1:{ $scale } non è consentita per le carte di tipo { $kind }; usa { $allowed }. Disegna a 1:4000, freemapper-print ingrandisce a 1:3000 in stampa
word-kind-schoolyard = cortile scolastico
word-kind-sprint = sprint
word-kind-forest = bosco
error-project-toml = file di progetto non valido: { $message }
error-project-unknown-adapter = il file di progetto nomina l'adattatore "{ $adapter }", che questo freemapper non ha; una versione più recente di freemapper potrebbe averlo (`freemapper update`), oppure viene dalla tua cartella degli adattatori (`freemapper adapters list` lo mostra) o, con --adapters-from-project sulla riga di comando, dalla cartella `adapters` del progetto (solo per un progetto di cui ti fidi)
error-project-not-fetched = nessun riquadro ancora scaricato; esegui prima `freemapper fetch`
error-project-ocd = esportazione OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: l'immagine e il world file non corrispondono all'area della carta; esegui di nuovo `freemapper derive`
error-project-tiles-outdated = l'area della carta richiede i riquadri { $tiles } che non sono stati scaricati; esegui `freemapper fetch`
error-project-buildings-not-fetched = nessun edificio ALKIS salvato per questa area della carta; esegui prima `freemapper fetch` online
error-project-orthophoto-not-fetched = nessuna ortofoto salvata per questa area della carta; esegui prima `freemapper fetch` online
error-project-terrain-not-fetched = nessun modello del terreno salvato per questa area della carta; esegui prima `freemapper fetch` online
error-project-candidates-outdated = i candidati OSM salvati non coprono l'area della carta; esegui `freemapper fetch`
error-project-stored = { $path }: { $message }; esegui di nuovo `freemapper fetch`
error-project-area-too-large = l'area della carta più il margine è { $width } × { $height } m, più di { $limit } m per lato; riduci l'area della carta o buffer_m in freemapper.toml
error-project-area-crosses-country = l'area della carta si trova su entrambi i lati di un confine di stato; le carte a cavallo di un confine non sono ancora supportate, quindi disegna l'area all'interno di un solo paese
error-project-permits-not-for-region = le note sui permessi non sono ancora disponibili per questa regione: interrogano servizi tedeschi e nominano autorità tedesche
error-project-outside-coverage = l'area della carta si trova fuori dai Land supportati ({ $states }); finora freemapper ha fonti di dati solo per quelli

## Errors: disk space

error-space-required = spazio su disco insufficiente per { $part }. Servono { $needed }, ma { $path } ha solo { $left } liberi oltre alla riserva di { $reserve }. Libera dello spazio ed esegui di nuovo `freemapper fetch`
error-space-skipped = freemapper salta { $part } per mancanza di spazio su disco. Servono { $needed }, ma { $path } ha solo { $left } liberi oltre alla riserva di { $reserve }. Libera dello spazio ed esegui di nuovo `freemapper fetch`
word-part-map-data = i dati cartografici ALKIS e OSM
word-part-terrain = il modello del terreno
word-part-orthophoto = l'ortofoto
word-part-surface = il modello di superficie per il template nDSM
word-part-osm-update = l'aggiornamento dell'estratto OSM nella cache
word-part-k-five = il template della carta K5 di Berlino
word-part-berlin-layers = le strutture ALKIS di Berlino, il catasto degli alberi, il rilievo stradale e i livelli di punti
word-part-state-layers = le strutture ALKIS complete, gli oggetti del modello del paesaggio e i catasti degli alberi
word-part-point-cloud = le nuvole di punti laser per i candidati di vegetazione

## Errors: downloads

error-fetch-http = download di { $url } fallito: { $message }
error-fetch-http-temporary = download di { $url } fallito: { $message }. Il servizio è probabilmente sovraccarico o in manutenzione; freemapper ha provato fino a { $attempts } volte. Riavvii l'esecuzione tra qualche minuto; ciò che è già stato scaricato viene conservato.
error-fetch-corrupt-archive = il download di { $url } non è un archivio di riquadri completo: { $message }
error-fetch-corrupt-extract = il download di { $url } non è un estratto OSM completo: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] il riquadro { $tiles } non è coperto dai riquadri pubblicati; riduci l'area della carta o buffer_m in freemapper.toml
        [many] i riquadri { $tiles } non sono coperti dai riquadri pubblicati; riduci l'area della carta o buffer_m in freemapper.toml
       *[other] i riquadri { $tiles } non sono coperti dai riquadri pubblicati; riduci l'area della carta o buffer_m in freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline e non nella cache: { $product }, riquadro { $tiles }; esegui `freemapper fetch` online
        [many] offline e non nella cache: { $product }, riquadri { $tiles }; esegui `freemapper fetch` online
       *[other] offline e non nella cache: { $product }, riquadri { $tiles }; esegui `freemapper fetch` online
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = modello del terreno (DGM1)
word-product-image-dsm = modello di superficie (bDOM)
word-product-saxony-dtm = modello del terreno (GeoSN DGM1)
word-product-saxony-dsm = modello di superficie (GeoSN DOM1)
error-fetch-osm-extract-missing = nessun estratto OSM nella cache; esegui `freemapper fetch` online
error-fetch-not-found = { $name } non trovato sul server
error-fetch-refused = { $url }: il server ha rifiutato la richiesta (HTTP { $status })
error-fetch-refused-detail = { $url }: il server ha rifiutato la richiesta (HTTP { $status }): { $detail }
error-fetch-too-large = il download di { $url } supera { $limit } MB; interrotto
error-fetch-not-stored = nessun dato salvato per questa area della carta ({ $label }): { $path } manca, non si può leggere o non la copre; esegui prima `freemapper fetch` online
error-fetch-range-ignored = { $url }: il server ha ignorato l'intervallo di byte { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper non invia richieste a questo indirizzo: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper non invia richieste a questo indirizzo: l'adattatore di { $name } non nomina l'host { $host }{ $from }. Se il servizio si è spostato lì, consenti l'host con `freemapper adapters allow-host { $adapter } { $host }` e segnalalo su https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper non invia richieste a questo indirizzo: l'adattatore { $adapter } non nomina l'host { $host }{ $from }. Se il servizio si è spostato lì, aggiungi l'host a `[net] hosts` del manifest dell'adattatore
error-fetch-host-redirected-from = {" "}({ $url } reindirizza lì)
error-reason-not-a-host-name = non è un nome di host
error-reason-not-ascii = un nome fuori da ASCII va scritto nella sua forma xn--
error-reason-not-a-dns-name = non è un nome DNS
error-reason-no-domain = un nome di questo computer o senza dominio
error-reason-local-name = un nome della rete locale
error-reason-ip-address = un indirizzo IP, non un nome DNS
error-reason-unexpected-character = carattere inatteso
error-reason-not-https = non https
error-reason-user-info = informazioni utente
error-reason-port = una porta diversa da 443
error-reason-host-not-declared = l'host non è tra quelli dichiarati
error-reason-local-address = un indirizzo di questo computer o della rete locale, non di un server su internet
error-reason-too-many-redirects = più di { $count } reindirizzamenti
error-reason-redirect-nowhere = un reindirizzamento (HTTP { $status }) verso il nulla
error-fetch-currency-csv = CSV di aggiornamento, riga { $line }: { $message }
error-alkis = dati ALKIS non validi: { $message }
error-osm = dati OSM: { $message }

## Errors: templates and rasters

error-derive-zip = archivio di riquadri: { $message }
error-derive-image = immagine: { $message }
error-derive-no-tiles = nessun riquadro da cui derivare
error-derive-area-too-large = area della carta troppo grande per un solo template: { $width } × { $height } pixel (al massimo { $limit } per lato); riduci l'area della carta o buffer_m in freemapper.toml
error-raster-world-file = world file non valido: { $message }
error-raster-misaligned = i raster non sono allineati alla stessa griglia di pixel: { $message }
error-raster-pixel-size = i raster hanno dimensioni di pixel diverse: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = GeoJSON non valido: { $message }
error-area-no-polygon = il GeoJSON non contiene alcun poligono
error-area-multi-polygon = MultiPolygon con { $parts } parti; dividi l'area della carta in un solo poligono
error-area-out-of-range = coordinata fuori intervallo: lat { $lat }, lon { $lon }
error-area-not-an-area = non è un'area della carta utilizzabile: { $message }
error-georef-date = la data { $date } è fuori dalla validità del World Magnetic Model
error-georef-map-range = il punto E { $easting } N { $northing } si trova oltre l'intervallo di coordinate della carta
error-omap-xml = XML di OOM non valido: { $message }
error-omap-missing-element = elemento <{ $name }> non trovato
error-ocd-duplicate-symbol = il numero di simbolo { $symbol } è usato due volte
error-ocd-unknown-symbol = un oggetto rimanda al simbolo { $symbol }, che non è nel file
error-ocd-wrong-type = un oggetto del simbolo { $symbol } ha il tipo sbagliato per il suo simbolo

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Uso:
cli-heading-commands = Comandi:
cli-heading-options = Opzioni:
cli-heading-arguments = Argomenti:
cli-help-possible-values = valori possibili
cli-help-default = predefinito
cli-arg-help = Mostra l'aiuto
cli-arg-version = Mostra la versione
cli-help-help = Mostra l'aiuto di un comando
cli-arg-help-command = Il comando di cui mostrare l'aiuto, per esempio run
cli-parse-hint = Per maggiori informazioni, prova '--help'.
cli-double-click-start = freemapper apre nel browser la pagina in cui scegli l'area della mappa. La cartella del progetto è { $path }. Chiudi questa finestra per terminare il programma.
cli-parse-unknown-argument = argomento inatteso { $arg }
cli-parse-missing-required = argomento obbligatorio mancante: { $args }
cli-parse-invalid-value = valore { $value } non valido per { $arg }
cli-parse-invalid-value-choices = valore { $value } non valido per { $arg }; valori possibili: { $values }
cli-parse-invalid-subcommand = comando sconosciuto { $value }
cli-parse-missing-subcommand = serve un comando
cli-parse-too-many-values = valore inatteso { $value } per { $arg }
cli-parse-argument-conflict = { $arg } non si può usare con { $other }

help-freemapper = Crea progetti di partenza OOM da dati geografici
help-freemapper-lang = Lingua dei testi del programma, `de` o `en` (predefinita: FREEMAPPER_LANG, poi la lingua del sistema, poi l'inglese)
help-init = Crea una cartella di progetto per un'area della carta
help-init-dir = Cartella del progetto da creare
help-init-geojson = File GeoJSON con il poligono dell'area della carta (EPSG:4326)
help-init-osm = Oggetto OSM dell'area della carta, es. way/992423119
help-init-place = Nome del luogo, es. "Clara-Zetkin-Schule Strausberg"; freemapper usa la corrispondenza migliore con un'area
help-init-part = Quale parte di un'area OSM composta da più parti separate usare (1 è la più grande)
help-init-kind = Tipo di carta
help-arg-scale = Denominatore della scala di un nuovo progetto: cortile scolastico 1000 (predefinito); sprint 4000 (predefinito) o 5000; bosco 15000 (predefinito), 10000 o 7500. Gli altri valori vengono rifiutati, perché freemapper-print li rifiuta
help-init-title = Titolo della carta per le informazioni marginali (predefinito: il nome della cartella)
help-export = Scrive il progetto di partenza (carta OOM, template, attribuzioni) in una cartella di progetto
help-export-dir = Cartella del progetto
help-arg-force = Sostituisce un file di carta esistente e conserva quello vecchio come `<name>.omap.bak`
help-export-ocad = Scrive anche un punto di partenza OCAD in `ocad/` (formato OCAD 12); una carta OOM esistente resta allora com'è, a meno che tu passi --force
help-export-pdf = Scrive anche `<name>.pdf`, la carta stampabile (solo gli oggetti della carta, senza candidati); una carta OOM esistente resta allora com'è, a meno che tu passi --force
help-export-pdf-candidates = Scrive anche `<name>-candidates.pdf`, la stampa di controllo: la carta con ogni candidato, fedele alla posizione; una carta OOM esistente resta allora com'è, a meno che tu passi --force
help-export-training = Scrive anche la carta di allenamento `<name>-training.omap` e il suo PDF `<name>-training.pdf`: la carta e ogni candidato, ripuliti per essere stampati così come sono; con --ocad anche `ocad/<name>-training.ocd`. Una carta OOM esistente resta allora com'è, a meno che tu passi --force, e --force sostituisce anche una carta di allenamento esistente
help-export-view = Con --training: scrive anche la carta di allenamento senza ciò che una vista esclude, come `<name>-training-<view>.omap` e il suo PDF (con --ocad anche un file OCAD). Viste: `no-paths` (senza strade, sentieri, scale), `brown-only` (solo simboli marroni e la cornice), `no-north-lines`, oppure un elenco `no:<codes>` / `only:<codes>`; combina con `+`. Ripeti l'opzione per più file
help-export-pdf-scale-bar = Disegna la barra della scala sui PDF (disattivata per impostazione predefinita, le carte tedesche di solito non ce l'hanno; il numero della scala resta nel riquadro del titolo)
help-export-pdf-sources = Righe delle fonti sui PDF: `auto` stampa l'elenco completo e lo accorcia solo se occuperebbe più di 5 righe, `full` non accorcia mai, `credit-only` stampa solo la riga dei crediti di freemapper (ODbL e dl-de/by-2-0 chiedono allora di citare le fonti quando passi la carta ad altri). ATTRIBUTION.txt elenca sempre ogni fonte
help-fetch = Scarica i dati per l'area della carta (ortofoto, modello del terreno, nDSM, edifici ALKIS, OSM)
help-fetch-dir = Cartella del progetto
help-fetch-offline = Usa solo la cache, senza accesso alla rete
help-arg-cache-dir = Cartella cache per i riquadri scaricati (predefinita: la cartella cache del sistema + `freemapper`)
help-derive = Calcola i template (ortofoto, ombreggiatura, pendenza, rilievo locale, nDSM), le curve di livello e le forme del terreno dai dati scaricati
help-derive-dir = Cartella del progetto
help-run = Scarica, deriva ed esporta in un colpo solo
help-run-dir = Cartella del progetto
help-run-offline = Usa solo la cache, senza accesso alla rete
help-run-ocad = Scrive anche un punto di partenza OCAD in `ocad/` (formato OCAD 12)
help-run-pdf = Scrive anche `<name>.pdf`, la carta stampabile (solo gli oggetti della carta, senza candidati)
help-run-pdf-candidates = Scrive anche `<name>-candidates.pdf`, la stampa di controllo: la carta con ogni candidato, fedele alla posizione
help-run-training = Scrive anche la carta di allenamento `<name>-training.omap` e il suo PDF `<name>-training.pdf`: la carta e ogni candidato, ripuliti per essere stampati così come sono; con --ocad anche `ocad/<name>-training.ocd`. Una carta di allenamento esistente resta, a meno che tu passi --force
help-run-view = Con --training: scrive anche la carta di allenamento senza ciò che una vista esclude, come `<name>-training-<view>.omap` e il suo PDF (con --ocad anche un file OCAD). Viste: `no-paths` (senza strade, sentieri, scale), `brown-only` (solo simboli marroni e la cornice), `no-north-lines`, oppure un elenco `no:<codes>` / `only:<codes>`; combina con `+`. Un file di vista esistente resta, a meno che tu passi --force
help-run-pdf-scale-bar = Disegna la barra della scala sui PDF (disattivata per impostazione predefinita; il numero della scala resta nel riquadro del titolo)
help-run-pdf-sources = Righe delle fonti sui PDF: `auto` stampa l'elenco completo e lo accorcia solo se occuperebbe più di 5 righe, `full` non accorcia mai, `credit-only` stampa solo la riga dei crediti di freemapper (ODbL e dl-de/by-2-0 chiedono allora di citare le fonti quando passi la carta ad altri). ATTRIBUTION.txt elenca sempre ogni fonte
help-select = Sceglie l'area della carta nel browser, disegnandola o scegliendo un luogo. Il salvataggio scrive freemapper.toml e crea la cartella del progetto se serve
help-select-dir = Cartella del progetto (creata al salvataggio se non esiste)
help-select-kind = Tipo di carta di un nuovo progetto
help-select-title = Titolo della carta di un nuovo progetto
help-select-no-browser = Stampa solo l'indirizzo della pagina; non apre il browser
help-select-port = Porta su 127.0.0.1 (predefinita: una libera)
help-select-offline = La pagina parte con "Solo cache, nessuna rete" selezionato
help-select-ocad = La pagina parte con il progetto di partenza OCAD selezionato
help-select-pdf = La pagina parte con il PDF stampabile selezionato
help-select-pdf-candidates = La pagina parte con la stampa di controllo (ogni candidato) selezionata
help-select-training = La pagina parte con la carta di allenamento selezionata
help-select-pdf-scale-bar = La pagina parte con la barra della scala sui PDF selezionata
help-select-pdf-sources = La pagina parte con questa scelta di righe delle fonti sui PDF, come per `run`
help-find = Elenca i luoghi OpenStreetMap per un nome, prima le aree, con l'id per init --osm
help-find-query = Nome del luogo, es. "Clara-Zetkin-Schule Strausberg"
help-find-json = Stampa le corrispondenze come JSON, con gli stessi campi dello strumento MCP find_place
help-mcp = Fornisce gli strumenti a un agente IA via MCP (stdin/stdout)
help-update = Installa un freemapper più recente dalle release di GitHub, dopo aver chiesto. Una build dai sorgenti dice invece come aggiornarla
help-update-check = Solo controllo: codice di uscita 10 se esiste una versione più recente, 0 se questa è aggiornata
help-update-yes = Installa senza chiedere; serve quando l'input non è un terminale
help-update-version = Installa questa versione (x.y.z) invece della più recente, anche una più vecchia
help-update-rollback = Torna alla versione che l'ultimo aggiornamento ha sostituito
help-update-mode = Memorizza cosa fa il controllo giornaliero quando trova una versione più recente (predefinito: notify). Funziona in ogni build, perché scrive solo l'impostazione
help-adapters = Mostra gli adattatori regionali: i Land e i paesi per cui freemapper ha fonti di dati
help-adapters-list = Elenca gli adattatori con la loro origine (integrato, la tua cartella, un progetto), la revisione, il giorno dell'ultimo controllo e gli host che interrogano
help-adapters-list-json = Stampa l'elenco come JSON
help-adapters-list-project = Legge anche gli adattatori nella cartella `adapters` di questa cartella di progetto, come farebbe --adapters-from-project
help-adapters-check = Controlla la cartella di un adattatore prima che venga usato o passato ad altri: se verrebbe caricato, cosa dovrebbe sapere il suo autore e, con --live, se i suoi servizi rispondono. Codice di uscita 0: lo supera; 1: ha rilievi; 2: il controllo non ha potuto essere eseguito
help-adapters-check-dir = La cartella dell'adattatore; il suo nome è l'id dell'adattatore
help-adapters-check-live = Interroga anche i servizi dell'adattatore per il riquadro indicato da --bbox e dice cosa fornisce ciascuno. Le richieste restano sugli host dichiarati dal manifest
help-adapters-check-bbox = Il riquadro del controllo live in gradi, come minLon,minLat,maxLon,maxLat. Si trova entro il confine dell'adattatore; bastano 150 m per 150 m
help-adapters-check-json = Stampa il risultato come JSON
help-adapters-probe = Interrogare una volta ogni servizio nominato dagli adattatori integrati e dire se risponde: ok, slow, moved o error. Il codice di uscita è 0 anche quando un servizio fallisce; il risultato è la risposta
help-adapters-probe-json = Stampa il risultato come JSON
help-adapters-probe-status = Leggere lo stato della notte precedente da questo file (può mancare) e stampare il nuovo stato come JSON: un servizio è fuori uso dopo due notti negative di fila. Le variazioni vanno sullo standard error
help-adapters-allow-host = Consente a un adattatore integrato un host in più: per un Land il cui servizio si è spostato su un server che il suo adattatore non nomina ancora. Il messaggio sull'indirizzo rifiutato dice quale host. L'host viene verificato come un host di un manifest, e freemapper si collega comunque solo a server su internet
help-adapters-allow-host-adapter = L'id dell'adattatore integrato, come lo mostra `adapters list`
help-adapters-allow-host-host = L'host da consentire: un nome DNS, nessun indirizzo, nessuna porta
help-adapters-disallow-host = Revoca un host che allow-host aveva consentito
help-adapters-disallow-host-adapter = L'id dell'adattatore integrato
help-adapters-disallow-host-host = L'host da revocare
help-adapters-export = Scrive l'adattatore integrato di un Land come cartella di adattatore che puoi modificare: il suo manifesto e i file che il manifesto nomina, in `<dir>/<id>`. La cartella non deve ancora esistere o deve essere vuota. Modifica gli indirizzi nel suo `manifest.toml`, controllala con `adapters check` e mettila nella tua cartella degli adattatori: sostituisce allora l'adattatore integrato, con un avviso. Eliminando la cartella torna quello integrato
help-adapters-export-adapter = L'id dell'adattatore integrato, come lo mostra `adapters list`
help-adapters-export-dir = La cartella in cui viene creata la cartella `<id>`
help-sources = Leggere e correggere gli indirizzi delle fonti di dati senza una nuova versione. Una correzione va nella tua cartella degli adattatori e sostituisce l'indirizzo integrato
help-sources-list = Elencare gli indirizzi dei manifesti con i loro percorsi e l'origine di ogni manifesto. Con un id di adattatore, solo i suoi
help-sources-get = Stampare un indirizzo e nient'altro. Codice di uscita 1 se l'adattatore o il percorso è sconosciuto
help-sources-set = Correggere un indirizzo nella tua cartella degli adattatori dopo che l'intero manifesto ha superato il controllo. Un indirizzo su un host che il manifesto non dichiara viene rifiutato, salvo --add-host
help-sources-diff = Mostrare che cosa la tua cartella dell'adattatore cambia rispetto al manifesto integrato
help-sources-check = Controllare la cartella del tuo adattatore con quell'id come `adapters check`. Codice di uscita 0: superato; 1: rilievi; 2: il controllo non ha potuto essere eseguito
help-sources-schema = Stampare le chiavi di un manifesto che contengono un indirizzo, con --json come schema JSON
help-arg-adapter = L'id dell'adattatore, come lo mostra `adapters list`
help-arg-path = Il percorso dell'indirizzo, come lo mostra `sources list`
help-sources-set-url = Il nuovo indirizzo: https, senza query
help-sources-set-add-host = Aggiungere anche l'host del nuovo indirizzo agli host che il manifesto dichiara
cli-sources-no-user-dir = questo sistema non ha una cartella per i tuoi adattatori; imposta FREEMAPPER_ADAPTER_DIR su un percorso assoluto
cli-sources-bad-id = { $adapter } non è un id di adattatore
cli-sources-unknown-adapter = freemapper non conosce nessun adattatore { $adapter }; `freemapper adapters list` mostra gli id
cli-sources-no-manifest = nessun manifesto di questo programma descrive l'adattatore { $adapter }; i suoi indirizzi sono nel codice
cli-sources-unknown-path-of = l'adattatore { $adapter } non ha nessun indirizzo sotto { $path }; `freemapper sources list { $adapter }` mostra i percorsi
cli-sources-bad-value = il nuovo indirizzo deve essere un indirizzo https su un host con nome, senza virgolette, barra rovesciata, carattere di controllo, utente o porta
cli-sources-host-not-declared = il nuovo indirizzo porta all'host { $host }, che il manifesto non dichiara; --add-host lo dichiara
cli-sources-invalid = il manifesto non supererebbe il controllo con questa modifica, quindi non è stato scritto nulla: { $findings }
cli-sources-set-created = La cartella { $folder } dell'adattatore { $adapter } è stata creata dal suo manifesto integrato; sostituisce l'adattatore integrato dal prossimo avvio.
cli-sources-set-host-added = L'host { $host } è ora dichiarato in [net] hosts; freemapper interroga ora quel server per questo adattatore.
cli-sources-set-next = Controllala con `freemapper adapters check { $folder }`. L'indicazione della fonte e la licenza restano come erano: mantienile corrette se il servizio ha cambiato gestore. Riavvia il server MCP o la pagina dell'area.
cli-sources-diff-no-folder = non hai una cartella dell'adattatore { $adapter } ({ $folder }); non c'è nulla da confrontare
cli-sources-diff-none = la tua cartella contiene gli stessi indirizzi del manifesto integrato
help-sources-list-json = Stampa l'elenco come JSON
help-sources-check-live = Interroga anche i servizi dell'adattatore per il riquadro indicato da --bbox e dice cosa fornisce ciascuno. Le richieste restano sugli host dichiarati dal manifest
help-sources-check-bbox = Il riquadro del controllo live in gradi, come minLon,minLat,maxLon,maxLat. Si trova entro il confine dell'adattatore; bastano 150 m per 150 m
help-sources-check-json = Stampa il risultato come JSON
help-sources-schema-json = Stampa il risultato come JSON
help-arg-adapters-from-project = Carica anche gli adattatori nella cartella `adapters` della cartella del progetto. freemapper nomina ciascuno con gli host che interroga. Usalo per un progetto di cui ti fidi: un adattatore fa interrogare a freemapper i server che nomina

## The summary of a run, for the MCP server and the area page

cli-internal-error = errore interno: { $message }
cli-pdf-sources-invalid = pdf_sources deve essere auto, full o credit-only, non { $value }
cli-view-unknown = una vista è no-paths, brown-only, no-north-lines, no:<codes> o only:<codes>, non { $value }
cli-view-empty = l'elenco { $value } non nomina alcun codice di simbolo
cli-view-bad-code = { $value } non è un codice di simbolo come 505 o 501.6
cli-summary-training-view = Scritti { $map } e { $pdf } (una vista della carta di allenamento)
cli-summary-kept-training-view = Conservati i file della vista { $view } della carta di allenamento; passa --force per rinnovarli
cli-summary-training-view-failed = La vista della carta di allenamento è fallita: { $error }
cli-estimate-disk-unreadable = impossibile leggere lo spazio libero su disco di { $path } ({ $message })
cli-summary-wrote-map = Scritta la carta { $path }
cli-summary-kept-map = Conservata la carta esistente { $path } (può contenere il lavoro del cartografo)
cli-summary-map-lacks-templates = La carta non fa ancora riferimento ai template immagine. Avvia la pipeline con force per sostituirla e conservare quella vecchia come .omap.bak.
cli-summary-map-lacks-buildings = La carta non ha ancora gli edifici ALKIS. Avvia la pipeline con force per aggiungerli; sostituisce la carta e conserva quella vecchia come .omap.bak.
cli-summary-map-lacks-canopies = Alla carta mancano le tettoie e i carport ALKIS (522), che ora sono oggetti della carta. Avvia la pipeline con force per aggiungerli; sostituisce la carta e conserva quella vecchia come .omap.bak.
cli-summary-map-light = La carta base: { $joined } linee unite, { $left } nascoste o troppo piccole lasciate ai livelli candidati (nulla è stato spostato)
cli-summary-map-light-kept = I livelli candidati ora contengono anche { $left } oggetti o tratti che questa versione toglie dalla carta base (con il tag freemapper:generalized). La carta conservata li ha ancora, quindi compaiono due volte. Avvia la pipeline con force per ripulire la carta; conserva quella vecchia come .omap.bak.
cli-summary-map-old-candidates = La carta nomina ancora templates/candidates.omap; questa esecuzione lo ha riempito con tutti i candidati. I candidati ora sono anche in sette livelli, un file ciascuno (templates/candidates-*.omap). La carta li elenca dopo un'esecuzione con force (la vecchia carta resta come .omap.bak), oppure il cartografo li aggiunge in OOM (Templates > Open template…).
cli-summary-wrote-ocad = Scritta la carta OCAD { $path } ({ $count } oggetti)
cli-summary-kept-ocad = Conservata la carta OCAD esistente { $path } (può contenere il lavoro del cartografo; force la sostituisce)
cli-summary-ocad-old-candidates = La carta OCAD nomina ancora { $path } come carta di sfondo; questa esecuzione l'ha riempita con tutti i candidati. I candidati ora sono anche in sette livelli, un file ciascuno (-candidates-*.ocd). La carta OCAD li elenca dopo un'esecuzione con force (la vecchia carta resta come .ocd.bak), oppure il cartografo li aggiunge in OCAD come carte di sfondo.
cli-summary-ocad-left-out = Esclusi dalla carta OCAD: { $skipped } oggetti e { $dropped } parti degeneri
cli-summary-ocad-symbols = In OCAD scegli Symbol > Load Colors and Symbols From… e seleziona il set di simboli ufficiale{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , il set scuola IOF 2019
       *[other] {" "}ISSprOM 2019
    }; sostituisce i simboli segnaposto in base al numero.
cli-summary-ocad-failed = Esportazione OCAD fallita: { $error }
cli-summary-pdf-without-credits = PDF senza crediti delle fonti: ODbL e dl-de/by-2-0 chiedono di citare le fonti quando passi la carta ad altri; vedi ATTRIBUTION.txt
cli-summary-wrote-pdf = Scritta la carta { $path } (senza candidati)
cli-summary-pdf-failed = Esportazione PDF fallita: { $error }
cli-summary-wrote-pdf-candidates = Scritta la stampa di controllo { $path } (la carta con ogni candidato, fedele alla posizione)
cli-summary-pdf-candidates-failed = Esportazione della stampa di controllo fallita: { $error }
cli-summary-training-failed = Esportazione della carta di allenamento fallita: { $error }
cli-summary-kept-training = Conservata la carta di allenamento esistente { $path }, così che un'esecuzione non sovrascriva una carta già stampata o consegnata; eliminala o imposta force per rinnovarla

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = nessuna cartella home su questo sistema; usa un percorso assoluto
mcp-error-relative-path = { $path } è un percorso relativo; usa un percorso assoluto o uno che inizi con ~
mcp-error-home-folder = { $path } è la cartella home stessa; usa una cartella al suo interno, es. ~/maps/school
mcp-error-no-project = nessun progetto freemapper in { $path }; usa prima create_project
mcp-error-unknown-kind = tipo sconosciuto { $kind }; usa "schoolyard", "sprint" o "forest"
mcp-error-reading = lettura di { $path }: { $error }
mcp-error-one-area-source = indica esattamente uno tra osm_id (da find_place) e geojson
mcp-error-page-open = la pagina di selezione dell'area per questo progetto è ancora aperta; chiedi all'utente di salvare l'area e di premere Fatto sulla pagina, poi avvia la pipeline
mcp-error-no-job = nessun job { $job }; start_pipeline e permits_for_area restituiscono gli id dei job di questa sessione
mcp-error-job-running = { $job } è ancora in esecuzione per { $path }; si esegue una sola pipeline alla volta perché i progetti condividono la cache dei download. Avvia questa quando project_status riporta finished o failed
mcp-error-page-start = impossibile avviare la pagina: { $error }
mcp-created-project = Progetto { $path } creato per { $area } ({ $kind }, 1:{ $scale }). Poi: estimate_download, quindi start_pipeline.
mcp-created-project-given-area = Progetto { $path } creato per l'area indicata ({ $kind }, 1:{ $scale }). Poi: estimate_download, quindi start_pipeline.
mcp-page-already-open = La pagina per questa cartella è già aperta; mantiene le impostazioni con cui è stata aperta.
mcp-page-next = Chiedi all'utente di scegliere l'area sulla pagina, premere Salva e poi Fatto.
mcp-preview-of = Anteprima di { $layer }
mcp-permits-next = Le note sui permessi sono in preparazione. Interroga project_status con questo job_id; quando è finished, permits_md contiene il testo di permits.md. Passalo come indicazione, non come consulenza legale.
mcp-tool-check-update = Se esiste una versione più recente di freemapper: la sua versione, la data di rilascio, le note di rilascio e la dimensione del download, oppure che questa è aggiornata. Non installa nulla.
mcp-tool-create-project = Crea una cartella di progetto per un'area della carta. Indica osm_id (da find_place) o un file GeoJSON. kind è schoolyard (1:1000, predefinito), sprint (1:4000) o forest (1:15000).
mcp-param-create-project-dir = Nuova cartella del progetto (percorso assoluto o che inizia con ~).
mcp-param-create-project-geojson = Al posto di osm_id: un file GeoJSON con il poligono dell'area della carta (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (predefinito, 1:1000), "sprint" (1:4000) o "forest" (1:15000).
mcp-param-create-project-osm-id = Oggetto OSM dell'area della carta da find_place, es. "way/992423119".
mcp-param-create-project-part = Con osm_id: quale parte di un'area composta da più parti separate usare (1 è la più grande).
mcp-param-create-project-scale = Denominatore della scala: schoolyard 1000 (predefinito); sprint 4000 (predefinito) o 5000; forest 15000 (predefinito), 10000 o 7500. Gli altri valori vengono rifiutati; per 1:3000 disegna a 4000, freemapper-print ingrandisce in stampa.
mcp-param-create-project-title = Titolo della carta per le informazioni marginali (predefinito: il nome della cartella).
mcp-tool-estimate-download = Cosa scaricherà start_pipeline per un progetto: i riquadri, ogni download con la sua dimensione in MB, se è nella cache e se entra nel disco, gli MB ancora da scaricare e, sotto disk, lo spazio libero dei volumi della cache e del progetto con i messaggi per ciò che non entra (riferiscili all'utente).
mcp-param-estimate-download-dir = Cartella del progetto (percorso assoluto o che inizia con ~).
mcp-tool-find-place = Cerca una scuola, un parco o un altro luogo per nome (OpenStreetMap Nominatim). Restituisce fino a 5 corrispondenze, prima le aree; solo le corrispondenze con has_area possono essere un'area della carta. Un elenco vuoto significa che non è stato trovato nulla.
mcp-param-find-place-query = Nome della scuola o del luogo, es. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installa la versione più recente di freemapper sopra questo programma. Chiamalo solo dopo che la persona ha acconsentito, con confirm true. La nuova versione si applica quando l'assistente riavvia il server freemapper.
mcp-param-install-update-confirm = true quando la persona ha acconsentito a installare la versione più recente; senza di esso non viene installato nulla.
mcp-tool-list-layers = I livelli di un progetto: la carta, i sette livelli candidati e i template raster, ciascuno con il suo file, se è già stato creato e quale passo lo crea.
mcp-param-list-layers-dir = Cartella del progetto (percorso assoluto o che inizia con ~).
mcp-tool-open-area-selector = Apre nel browser una pagina in cui l'utente disegna l'area della carta (rettangolo o poligono) o sceglie un luogo, vede i riquadri e la dimensione del download, e la salva; il salvataggio crea la cartella del progetto o ne cambia l'area. Restituisce l'URL da dare all'utente. La pagina resta attiva finché l'utente non preme Fatto; chiamandolo di nuovo per la stessa cartella si ottiene lo stesso URL.
mcp-param-open-area-selector-dir = Cartella del progetto (percorso assoluto o che inizia con ~); creata quando l'utente salva.
mcp-param-open-area-selector-kind = Tipo di carta di un nuovo progetto: "schoolyard" (predefinito, 1:1000), "sprint" (1:4000) o "forest" (1:15000).
mcp-param-open-area-selector-ocad = Il pannello di esecuzione parte con il progetto di partenza OCAD selezionato. Predefinito false.
mcp-param-open-area-selector-offline = Il pannello di esecuzione parte con "Solo cache, nessuna rete" selezionato. Predefinito false.
mcp-param-open-area-selector-open-browser = Apre la pagina anche nel browser predefinito di questo computer. Predefinito true.
mcp-param-open-area-selector-pdf = Il pannello di esecuzione parte con il PDF stampabile selezionato. Predefinito false.
mcp-param-open-area-selector-pdf-candidates = Il pannello di esecuzione parte con la stampa di controllo (ogni candidato) selezionata. Predefinito false.
mcp-param-open-area-selector-pdf-scale-bar = Il pannello di esecuzione parte con la barra della scala sui PDF selezionata. Predefinito false.
mcp-param-open-area-selector-pdf-sources = Righe delle fonti sui PDF con cui parte il pannello: "auto" (predefinito), "full" o "credit-only", come per start_pipeline.
mcp-param-open-area-selector-permits = Il pannello di esecuzione parte con le note sui permessi selezionate. Predefinito false.
mcp-param-open-area-selector-scale = Denominatore della scala di un nuovo progetto: schoolyard 1000 (predefinito); sprint 4000 (predefinito) o 5000; forest 15000 (predefinito), 10000 o 7500. Gli altri valori vengono rifiutati.
mcp-param-open-area-selector-title = Titolo della carta di un nuovo progetto.
mcp-param-open-area-selector-training = Il pannello di esecuzione parte con la carta di allenamento selezionata. Predefinito false.
mcp-tool-preview-layer = Un'anteprima PNG (al massimo 800 px) di un livello: map (l'immagine degli oggetti della carta scritta dall'ultima esecuzione, senza i candidati) oppure un template raster: orthophoto, hillshade, slope, relief, ndsm o k5 a Berlino.
mcp-param-preview-layer-dir = Cartella del progetto (percorso assoluto o che inizia con ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" o "k5" (solo Berlino).
mcp-tool-project-status = Stato di un job di start_pipeline o permits_for_area: running, finished o failed, le ultime righe di avanzamento, gli avvisi, i file scritti e il risultato o l'errore. Un job permits_for_area finished porta anche il testo di permits.md come permits_md.
mcp-param-project-status-job-id = Id del job di start_pipeline o permits_for_area, es. "job-1".
mcp-tool-permits-for-area = Crea le note sui permessi di un progetto: le unità amministrative dell'area della carta, le aree protette al suo interno e chi gestisce il terreno, con le autorità a cui rivolgersi; un'indicazione, non una consulenza legale. Scrive permits.md e permits.json nella cartella. Gira in background e restituisce un job_id; interroga project_status, la cui risposta porta il testo di permits.md come permits_md quando il job è finito. Si esegue un solo job alla volta.
mcp-param-permits-for-area-dir = Cartella del progetto (percorso assoluto o che inizia con ~).
mcp-param-permits-for-area-offline = Legge solo i dati salvati nel progetto da un'esecuzione precedente, senza accesso alla rete. Predefinito false.
mcp-tool-start-pipeline = Scarica i dati, calcola i template e scrive il progetto di partenza (fetch, derive, export). Gira in background e restituisce un job_id; interroga project_status. Si esegue una sola pipeline alla volta. Conserva una carta esistente a meno che force sia true.
mcp-param-start-pipeline-dir = Cartella del progetto (percorso assoluto o che inizia con ~).
mcp-param-start-pipeline-force = Sostituisce un file di carta esistente e conserva quello vecchio come .omap.bak. Predefinito false.
mcp-param-start-pipeline-ocad = Scrive anche un punto di partenza OCAD nella cartella ocad/ (formato OCAD 12, per chi lavora in OCAD). Predefinito false.
mcp-param-start-pipeline-offline = Usa solo la cache dei download, senza accesso alla rete. Predefinito false.
mcp-param-start-pipeline-pdf = Scrive anche <name>.pdf, la carta pronta da stampare (solo gli oggetti della carta, senza candidati). Predefinito false.
mcp-param-start-pipeline-pdf-candidates = Scrive anche <name>-candidates.pdf, la stampa di controllo: la carta con ogni candidato, fedele alla posizione (controllali sul terreno). Predefinito false.
mcp-param-start-pipeline-pdf-scale-bar = Disegna la barra della scala sui PDF. Predefinito false (le carte tedesche di solito non ce l'hanno; il numero della scala resta nel riquadro del titolo).
mcp-param-start-pipeline-pdf-sources = Righe delle fonti sui PDF: "auto" (predefinito; l'elenco completo, accorciato solo se occupa più di 5 righe), "full" (mai accorciato) o "credit-only" (solo la riga dei crediti di freemapper; ODbL e dl-de/by-2-0 richiedono allora di citare le fonti quando la carta viene passata ad altri). ATTRIBUTION.txt elenca sempre ogni fonte.
mcp-param-start-pipeline-permits = Scrive anche le note sui permessi (permits.md e permits.json) dopo la carta: unità amministrative, aree protette e chi gestisce il terreno. Un errore in questa fase è un avviso, non un job fallito. Predefinito false.
mcp-param-start-pipeline-training = Scrive anche la carta di allenamento <name>-training.omap e il suo PDF <name>-training.pdf: la carta e ogni candidato, ripuliti per essere stampati così come sono; con ocad anche ocad/<name>-training.ocd. Una carta di allenamento esistente viene conservata a meno che force sia true. Predefinito false.
mcp-param-start-pipeline-training-views = Con training: scrive anche la carta di allenamento senza ciò che una vista esclude, un set di file per voce: "no-paths", "brown-only", "no-north-lines", oppure un elenco come "no:505,506" o "only:101,102", combinato con +. I file sono <name>-training-<view>.omap e il suo PDF.
mcp-instructions = freemapper crea progetti di partenza OpenOrienteering Mapper per carte da orienteering (cortile scolastico 1:1000, sprint 1:4000, bosco 1:15000) a partire da dati geografici ufficiali di { $states } e da OSM. Flusso di lavoro: find_place (cerca la scuola o l'area per nome; scegli un risultato con has_area) → create_project (una nuova cartella, percorso assoluto, con l'osm_id) → estimate_download (di' all'utente quanto verrà scaricato) → start_pipeline (restituisce un id di job; il download può richiedere minuti) → project_status (interroga ogni pochi secondi finché lo stato è finished o failed) → list_layers e preview_layer (mostra all'utente i template). L'utente poi apre il file della carta in OpenOrienteering Mapper. Se nessun luogo va bene o l'utente vuole disegnare l'area, open_area_selector apre nel browser una pagina in cui l'utente disegna o sceglie l'area e la salva (questo crea il progetto); prosegui con estimate_download quando l'utente dice che è salvata. I percorsi devono essere assoluti o iniziare con ~. Si esegue una sola pipeline alla volta. permits_for_area crea le note sui permessi di un progetto (unità amministrative, aree protette, chi gestisce il terreno; un'indicazione, non una consulenza legale) come un job simile a start_pipeline. check_update dice se esiste un freemapper più recente; install_update lo installa, solo con confirm: true dopo che la persona ha acconsentito, e poi l'assistente riavvia il server freemapper.

cli-contour-interval-saved = Equidistanza delle curve di livello salvata in freemapper.toml: { $interval } m; derive traccia le curve di livello con essa
cli-contour-interval-auto = Equidistanza delle curve di livello in freemapper.toml: automatica (quella del tipo di carta; una carta da bosco ha 2,5 m in pianura e 10 m in terreno ripido)
cli-steep-slope-saved = Soglia di pendenza salvata in freemapper.toml: { $slope }°; derive cerca con essa scarpate e pareti rocciose
cli-steep-slope-auto = Soglia di pendenza in freemapper.toml: predefinita (circa 27°)
cli-face-slope-saved = Soglia di parete salvata in freemapper.toml: { $slope }°; derive cerca con essa le superfici di parete rocciosa
cli-face-slope-auto = Soglia di parete in freemapper.toml: predefinita (70°)
help-arg-contour-interval = Equidistanza delle curve di livello in metri, es. `2.5`, oppure `auto` per quella del tipo di carta (una carta da bosco: 5 m, 2,5 m dove è pianeggiante, 10 m dove è ripido). Salvata in freemapper.toml; derive traccia le curve di livello con essa
help-arg-steep-slope = Angolo di pendenza in gradi a partire dal quale il terreno è considerato ripido nella ricerca di scarpate e pareti rocciose, es. `55`, oppure `auto` per il valore predefinito di 27°. Alzala (da 55 a 65) nelle città di roccia e nelle gole, dove quasi ogni pendio è ripido e scarpate e pareti si fondono in un'unica macchia. Salvata in freemapper.toml; derive cerca con essa scarpate e pareti rocciose
help-arg-face-slope = Angolo di pendenza in gradi da cui il terreno è una parete rocciosa (superficie nera, simbolo 206), ad es. `75`, oppure `auto` per il valore predefinito di 70°. 65 dà più superfici e più grandi, anche sui pendii ripidi; 75 lascia solo vere pareti. Salvato in freemapper.toml; derive cerca con essa le pareti rocciose
cli-content-saved = Impostazione dei contenuti salvata in freemapper.toml: { $summary }
cli-content-invalid = Impossibile leggere l'impostazione dei contenuti: { $detail }
help-arg-content = Cosa va direttamente sulla carta: `safe` lascia ogni oggetto dove lo mette la generazione, `balanced` (predefinito) mette sulla carta gli oggetti del catasto e dei registri, `full` mette tutto sulla carta. La scelta è salvata in freemapper.toml
help-arg-content-set = Manda un gruppo di oggetti a `map`, `candidates` oppure `off`, come `GROUP=PLACE`, es. `walls=map`; si può ripetere. Gruppi: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Salvato in freemapper.toml
mcp-param-create-project-content = Cosa va direttamente sulla carta: "safe", "balanced" (predefinito) o "full". Salvato nel progetto. balanced mette sulla carta gli oggetti del catasto e dei registri, full mette tutto.
mcp-param-create-project-content-set = Gruppi mandati altrove rispetto a quanto dice il profilo, come "group=place" con place map, candidates oppure off, es. "walls=map". Gruppi: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Equidistanza delle curve di livello in metri, es. 2.5; 0 torna alla scelta automatica (quella del tipo di carta; una carta da bosco ha 2,5 m in pianura e 10 m in terreno ripido). Salvata nel progetto; il prossimo derive traccia le curve di livello con essa.
mcp-param-create-project-steep-slope = Angolo di pendenza in gradi a partire dal quale il terreno è considerato ripido nella ricerca di scarpate e pareti rocciose, es. 55; 0 torna al valore predefinito di 27°. Alzala (da 55 a 65) nelle città di roccia e nelle gole, dove quasi ogni pendio è ripido e scarpate e pareti si fondono in un'unica macchia. Salvata nel progetto; il prossimo derive cerca con essa scarpate e pareti rocciose.
mcp-param-create-project-face-slope = Angolo di pendenza in gradi da cui il terreno è una parete rocciosa (superficie nera, simbolo 206), ad es. 75; 0 torna al valore predefinito di 70°. 65 dà più superfici e più grandi, anche sui pendii ripidi; 75 lascia solo vere pareti. Salvato nel progetto; il prossimo derive cerca con essa le pareti rocciose.
mcp-param-start-pipeline-content = Cambia prima dell'esecuzione cosa va direttamente sulla carta: "safe", "balanced" o "full". Salvato nel progetto. Predefinito: l'impostazione del progetto, altrimenti balanced.
mcp-param-start-pipeline-content-set = Gruppi mandati altrove rispetto a quanto dice il profilo, come "group=place" con place map, candidates oppure off, es. "fences=candidates". Salvato nel progetto.
mcp-param-start-pipeline-contour-interval = Equidistanza delle curve di livello in metri, es. 2.5; 0 torna alla scelta automatica (quella del tipo di carta; una carta da bosco ha 2,5 m in pianura e 10 m in terreno ripido). Salvata nel progetto; il prossimo derive traccia le curve di livello con essa.
mcp-param-start-pipeline-steep-slope = Angolo di pendenza in gradi a partire dal quale il terreno è considerato ripido nella ricerca di scarpate e pareti rocciose, es. 55; 0 torna al valore predefinito di 27°. Alzala (da 55 a 65) nelle città di roccia e nelle gole, dove quasi ogni pendio è ripido e scarpate e pareti si fondono in un'unica macchia. Salvata nel progetto; il prossimo derive cerca con essa scarpate e pareti rocciose.
mcp-param-start-pipeline-face-slope = Angolo di pendenza in gradi da cui il terreno è una parete rocciosa (superficie nera, simbolo 206), ad es. 75; 0 torna al valore predefinito di 70°. 65 dà più superfici e più grandi, anche sui pendii ripidi; 75 lascia solo vere pareti. Salvato nel progetto; il prossimo derive cerca con essa le pareti rocciose.
mcp-error-content = Impossibile leggere l'impostazione dei contenuti: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Scrive le note sui permessi di un progetto (permits.md e permits.json): le unità amministrative, le aree protette e i gestori che l'area della carta tocca, e a chi di solito ci si deve rivolgere. Interroga il BfN e il BKG e, dove il Land dell'area della carta ha servizi aperti propri, quelli attraverso gli host del suo adattatore (elenco nel manuale, capitolo 12); nella Renania Settentrionale-Vestfalia scarica per questo circa 32 MB la prima volta, nel Brandeburgo circa 4 MB, poi solo file più recenti. Legge l'estratto OSM già nella cache; --offline usa ciò che è stato scaricato prima
help-permits-dir = Cartella del progetto
help-permits-offline = Usa solo i dati scaricati prima, senza accesso alla rete
help-run-permits = Scrive anche le note sui permessi (permits.md e permits.json) dopo la carta; interroga il BfN, il BKG e i servizi aperti del Land (manuale, capitolo 12). Un errore in questa fase è un avviso e la carta resta
cli-wrote-permits = Scritte le note sui permessi { $path }
cli-summary-wrote-permits = Scritte le note sui permessi { $path }
cli-permits-failed = impossibile scrivere le note sui permessi: { $error }
cli-permits-source-left-out = le note sui permessi omettono { $source }: { $reason }
cli-permits-not-read = non è stato letto
error-permits-stored = { $path }: { $message }; scarica di nuovo online le note sui permessi
error-permits-nothing-read = non è stato possibile leggere alcuna fonte delle note sui permessi ({ $missing }); scaricale prima online
error-permits-missing-not-read = { $source }: non letto

permits-title = Note sui permessi
permits-disclaimer = Un'indicazione, non una consulenza legale. Decidono le autorità competenti. Per la data e la precisione dei dati vedi la tabella sotto "Fonti e data dei dati"; le aree possono mancare o essere superate (BfN: unite una volta all'anno, "Nicht für Planungszwecke geeignet.", cioè non adatto alla pianificazione).
permits-disclaimer-regional = Un'indicazione, non una consulenza legale. Decidono le autorità competenti. Per la data e la precisione dei dati vedi la tabella sotto "Fonti e data dei dati"; le aree possono mancare o essere superate.
permits-bfn-merge = Il BfN unisce i dati dei Land una volta all'anno e li contrassegna come non adatti alla pianificazione. Decidono le autorità del Land.
permits-created = Creato il { $date } per l'area di una { $kind ->
        [schoolyard] carta da cortile scolastico
        [forest] carta da bosco
       *[other] carta sprint
    } ({ $area } ha).
permits-section-admin = Unità amministrative
permits-section-protected = Aree protette (BfN)
permits-section-osm-protected = Altre aree protette da OpenStreetMap
permits-section-operators = Gestori e proprietari secondo OpenStreetMap
permits-section-hints = Indicazioni
permits-section-sources = Fonti e data dei dati
permits-none-admin = VG250 non nomina alcuna unità amministrativa per l'area della carta.
permits-none-protected = Secondo il BfN, nell'area della carta non si trova alcuna area protetta.
permits-none-osm-protected = OpenStreetMap non nomina nell'area della carta alcuna area protetta che il BfN non elenchi già.
permits-none-operators = OpenStreetMap non nomina alcun gestore né alcun proprietario per l'area della carta.
permits-none-hints = Nessuna indicazione.
permits-section-not-read = I dati di { $source } non sono stati letti, quindi questa sezione è vuota, anche se potrebbero esserci voci.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Aree amministrative 1:250 000 (VG250) del BKG
        [bfn] Aree protette in Germania del BfN
       *[osm] OpenStreetMap, dall'estratto che freemapper tiene nella sua cache
    }
permits-level = { $level ->
        [land] Land (stato federato)
        [kreis] Circondario (Kreis)
        [verwaltungsgemeinschaft] Amt o Verwaltungsgemeinschaft
       *[gemeinde] Comune (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Toccato al margine (meno dello 0,5% dell'area della carta):
permits-at-edge-generalised = VG250 è generalizzato alla scala 1:250 000. Un'unità che tocca solo il margine può essere un effetto di questo.
permits-kind = { $kind ->
        [nationalpark] Parco nazionale
        [nationales-naturmonument] Monumento naturale nazionale
        [naturschutzgebiet] Riserva naturale (Naturschutzgebiet)
        [ffh] Area FFH (Natura 2000)
        [vogelschutzgebiet] Area di protezione degli uccelli (Natura 2000)
        [biosphaerenreservat] Riserva della biosfera
        [biosphaerenreservat-zone] Zona di una riserva della biosfera
        [landschaftsschutzgebiet] Area di protezione del paesaggio (Landschaftsschutzgebiet)
       *[naturpark] Parco naturale
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Istituita il { $date }
permits-protected-title = Base giuridica: { $title }
permits-protected-link = Testo di legge: { $url }
permits-protected-site-link = Pagina del sito (Natura 2000): { $url }
permits-protected-year = Anno dei dati presso il BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = classe di protezione { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = senza nome
permits-osm-operator = Gestore secondo OSM: { $operator }
permits-osm-enclosing-gap = Qui possono mancare le aree di OpenStreetMap che racchiudono l'intera area della carta senza un angolo al suo interno, e così le aree protette mappate in OSM come relazione con `type=boundary`. Le parti del BfN e di VG250 non ne sono toccate.
permits-osm-enclosing-gap-short = Qui possono mancare le aree di OpenStreetMap che circondano l'intera area della mappa senza avere un angolo al suo interno.
permits-operator-line = { $name } ({ $roles ->
        [operator] gestore
        [owner] proprietario
       *[both] gestore e proprietario
    }): { $objects }
permits-column-source = Fonte
permits-column-state = Stato
permits-column-credit = Crediti e licenza
permits-state-read = letto, scaricato il { $date }
permits-state-read-osm = letto, estratto OSM del { $date }
permits-state-unreachable = non raggiungibile ({ $reason })
permits-state-not-read = non letto
permits-no-credit = non usato
permits-hint-city-state = L'area della carta si trova a Berlino o ad Amburgo. Lì di solito è competente il Bezirksamt (ufficio di distretto) o un'autorità del Land. Queste note non stabiliscono ancora il Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Le aree protette riguardano l'autorità inferiore per la tutela della natura (untere Naturschutzbehörde). Qui è la città di { $kreis } stessa.
        [no] Le aree protette riguardano l'autorità inferiore per la tutela della natura (untere Naturschutzbehörde), di solito il { $kreis }.
       *[unknown] Le aree protette riguardano l'autorità inferiore competente per la tutela della natura (untere Naturschutzbehörde), di solito il Landkreis o la kreisfreie Stadt.
    }
permits-hint-national-park = Un parco nazionale ha regole proprie. Rivolgiti per tempo all'amministrazione del parco nazionale.
permits-hint-nsg-paths = In una riserva naturale di solito bisogna restare sui sentieri. Per una corsa fuori dai sentieri, rivolgiti all'autorità per la tutela della natura.
permits-hint-natura2000 = Un'area FFH o un'area di protezione degli uccelli (Natura 2000) tocca l'area della carta. Per una manifestazione l'autorità per la tutela della natura può chiedere una valutazione dei suoi effetti.
permits-hint-protected-areas-pl = Nell'area della mappa si trovano aree o oggetti protetti per la conservazione della natura. Per un parco nazionale, un parco paesaggistico o una riserva naturale il contatto abituale è l'amministrazione del parco o della riserva; per le altre aree e oggetti la Direzione regionale per la protezione dell'ambiente (RDOŚ) del voivodato di solito può dire chi è competente. Chiedi per tempo se pianifichi un evento lì.
permits-hint-order-office = { $amt ->
        [none] Per strade, piazze e altri terreni pubblici di solito è competente l'Ordnungsamt (ufficio per l'ordine pubblico) del comune { $gemeinde }.
       *[other] Per strade, piazze e altri terreni pubblici di solito è competente l'Ordnungsamt (ufficio per l'ordine pubblico): { $amt }, per il comune { $gemeinde }.
    }
permits-hint-park-operator = Gestore di un parco o giardino nell'area della carta secondo OpenStreetMap: { $operator }. Chiedi prima di usarlo.
permits-hint-school-operator = Gestore della scuola o dell'istituzione secondo OpenStreetMap: { $operator }. Per il suo terreno, rivolgiti al dirigente scolastico o a questo ente.
permits-section-districts = Bezirke (distretti, { $source })
permits-section-green-spaces = Spazi verdi ({ $source })
permits-section-owners = Terreni del Land e del Bund ({ $source })
permits-section-monuments = Monumenti ({ $source })
permits-section-protected-objects = Monumenti naturali e altri oggetti protetti ({ $source })
permits-none-districts = Secondo { $source }, l'area della carta non si trova in alcun Bezirk.
permits-none-green-spaces = Secondo { $source }, nell'area della carta non si trova alcuno spazio verde pubblico.
permits-none-owners = Secondo { $source }, nell'area della carta non si trova alcuna particella del Land o del Bund.
permits-none-monuments = Secondo { $source }, nell'area della carta non si trova alcun monumento.
permits-none-protected-objects = Secondo { $source }, nell'area della carta non si trova nessuno degli oggetti protetti che contiene.
permits-none-sampled = Nei punti interrogati, { $source } qui non nomina nulla.
permits-gaps = I dati di { $source } non sono completi: { $gaps }.
permits-sampled = { $source } risponde solo per singoli punti. freemapper ha interrogato una griglia di punti sulla carta: le percentuali sono stime, e ciò che si trova tra i punti può mancare.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = destinazione: { $dedication }
permits-playgrounds-none = Nell'area della carta non si trova alcun parco giochi pubblico.
permits-playgrounds-one = Nell'area della carta si trova un parco giochi pubblico. Non è elencato a parte.
permits-playgrounds-many = Nell'area della carta si trovano { $count } parchi giochi pubblici. Non sono elencati uno per uno.
permits-owner-other = Altri proprietari pubblici
permits-owners-rest = Per il resto dell'area i dati non nominano alcun proprietario.
permits-monuments-total = Monumenti in tutto o in parte nell'area della carta: { $count }.{ $links ->
        [yes] {" "}I link portano alle loro voci nell'elenco dei monumenti.
       *[no] {""}
    }
permits-monuments-links-left = Altre { $count } voci sono solo conteggiate qui, senza link.
permits-object-kind = { $kind ->
        [natural-monument] Monumento naturale (Naturdenkmal)
        [protected-landscape-part] Elemento paesaggistico protetto (geschützter Landschaftsbestandteil)
        [biotope] Biotopo protetto per legge (geschütztes Biotop)
        [national-park] Parco nazionale (park narodowy)
        [park-buffer-zone] Zona cuscinetto di un parco (otulina)
        [landscape-park] Parco paesaggistico (park krajobrazowy)
        [nature-reserve] Riserva naturale (rezerwat przyrody)
        [protected-landscape-area] Area di paesaggio protetto (obszar chronionego krajobrazu)
        [natura2000-habitats] Sito Natura 2000 per gli habitat (SOO)
        [natura2000-birds] Sito Natura 2000 per gli uccelli (OSO)
        [ecological-site] Area ecologica (użytek ekologiczny)
        [landscape-complex] Complesso naturale e paesaggistico (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Sito di documentazione (stanowisko dokumentacyjne)
        [natural-monument-pl] Monumento naturale (pomnik przyrody)
        [national-park-cz] Parco nazionale (národní park, NP)
        [protected-landscape-area-cz] Area paesaggistica protetta (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Riserva naturale nazionale (národní přírodní rezervace, NPR)
        [national-nature-monument] Monumento naturale nazionale (národní přírodní památka, NPP)
        [nature-reserve-cz] Riserva naturale (přírodní rezervace, PR)
        [nature-monument-cz] Monumento naturale (přírodní památka, PP)
        [protective-zone-cz] Zona di protezione (ochranné pásmo)
        [nature-zone-cz] Zona di tutela della natura (zóna ochrany přírody)
        [natura2000-habitats-cz] Sito Natura 2000 per gli habitat (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Sito Natura 2000 per gli uccelli (ptačí oblast, PO)
        [memorial-tree] Albero monumentale (památný strom)
       *[special-rules] Area con regole particolari
    }
permits-object-count = { $count } oggetti
permits-object-citations = Basi giuridiche:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Per strade, piazze e spazi verdi nel Bezirk { $district } di solito è competente il Bezirksamt { $district }, lì il Fachamt Management des öffentlichen Raumes (ufficio per lo spazio pubblico).
       *[berlin] Per strade, piazze e spazi verdi nel Bezirk { $district } di solito sono competenti l'Ordnungsamt (ufficio per l'ordine pubblico) e lo Straßen- und Grünflächenamt (ufficio strade e parchi) del Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Per i monumenti nell'area della carta è competente il Denkmalschutzamt Hamburg (ufficio per i monumenti).
       *[berlin] Per i monumenti nel Bezirk { $district } è competente l'autorità inferiore per i monumenti (untere Denkmalschutzbehörde) del Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Se nell'area della carta si trovino monumenti tutelati, i dati qui non possono dirlo con certezza.
       *[yes] Nell'area della carta ci sono monumenti tutelati.
    } Se una corsa li tocchi, per esempio con un punto di controllo presso un monumento o un tratto attraverso un sito protetto, lo decide l'autorità inferiore per i monumenti (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Qui è la città di { $kreis } stessa.
        [no] , di solito il { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; in alcuni Land è il comune { $gemeinde }.
        }
       *[unknown] {" "}(di solito il Landkreis o la kreisfreie Stadt, in alcuni Land il comune).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] I monumenti naturali, gli elementi paesaggistici protetti e i biotopi protetti sulla carta sono di competenza dell'autorità inferiore per la tutela della natura (untere Naturschutzbehörde). Qui è la città di { $kreis } stessa.
        [no] I monumenti naturali, gli elementi paesaggistici protetti e i biotopi protetti sulla carta sono di competenza dell'autorità inferiore per la tutela della natura (untere Naturschutzbehörde), di solito il { $kreis }.
       *[unknown] I monumenti naturali, gli elementi paesaggistici protetti e i biotopi protetti sulla carta sono di competenza dell'autorità inferiore per la tutela della natura (untere Naturschutzbehörde), di solito il Landkreis o la kreisfreie Stadt.
    }
permits-hint-biotope = Nell'area della carta si trovano biotopi protetti per legge{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. È vietato tutto ciò che può distruggerli o danneggiarli in modo considerevole (§ 30 BNatSchG). Colloca punti di controllo e tratti fuori da essi, oppure rivolgiti all'autorità inferiore per la tutela della natura (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Per i monumenti naturali e gli elementi paesaggistici protetti nel Bezirk { $district } di solito è competente il Bezirksamt { $district }.
       *[berlin] Per i monumenti naturali e gli elementi paesaggistici protetti nel Bezirk { $district } è competente l'autorità inferiore per la tutela della natura (untere Naturschutzbehörde) del Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Secondo i dati aperti del Land, { $share } dell'area della carta appartiene ad altri proprietari pubblici.
       *[no] Secondo i dati aperti del Land, { $share } dell'area della carta appartiene al proprietario "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] I proprietari del resto dell'area non si possono ricavare dai dati aperti. Con un interesse legittimo (un organizzatore, per esempio) puoi chiedere un estratto all'ufficio del libro fondiario (Grundbuchamt); per i terreni pubblici aiuta l'Ordnungsamt o l'ufficio parchi competente.
       *[no] I proprietari di queste aree non si possono ricavare dai dati aperti. Con un interesse legittimo (un organizzatore, per esempio) puoi chiedere un estratto all'ufficio del libro fondiario (Grundbuchamt); per i terreni pubblici aiuta l'Ordnungsamt o l'ufficio parchi competente.
    }
permits-section-forest-units = Amministrazione forestale ({ $source })
permits-section-state-forest = Foresta demaniale del Land ({ $source })
permits-none-forest-units = Secondo { $source }, l'area della carta non si trova nel territorio di alcun ufficio forestale.
permits-section-admin-units = Unità amministrative ({ $source })
permits-none-admin-units = Secondo { $source }, l'area della mappa non si trova in nessuna unità amministrativa del registro.
permits-section-register-monuments = Monumenti del registro nazionale ({ $source })
permits-none-register-monuments = Secondo { $source }, nessun monumento iscritto si trova nel rettangolo attorno all'area della mappa.
permits-register-monuments-total = Monumenti iscritti nel rettangolo attorno all'area della mappa: { $count }. Il registro non indica la posizione, quindi queste note non possono dire dove si trovano; va verificato sulla mappa.
permits-register-monuments-left = Altri { $count } sono solo contati qui, senza nome.
permits-section-register-archaeology = Monumenti archeologici del registro nazionale ({ $source })
permits-none-register-archaeology = Secondo { $source }, nessun monumento archeologico iscritto si trova nel rettangolo attorno all'area della mappa.
permits-register-archaeology-total = Monumenti archeologici iscritti nel rettangolo attorno all'area della mappa: { $count }. Il registro non indica la posizione, quindi queste note non possono dire dove si trovano; va verificato sulla mappa.
permits-register-archaeology-left = Altri { $count } sono solo contati qui, senza nome.
permits-section-register-history = Monumenti della storia ({ $source })
permits-none-register-history = Secondo { $source }, nessun monumento della storia si trova nel rettangolo intorno all'area della mappa.
permits-register-history-in-area = Monumenti della storia nell'area della mappa: { $count }.
permits-register-history-nearby = Altri monumenti della storia nel rettangolo intorno all'area della mappa, ma non nell'area della mappa: { $count }.
permits-register-history-left = Altri { $count } sono solo contati qui, senza i nomi.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Secondo { $source }, nell'area della carta non si trova alcuna foresta demaniale del Land.
permits-forest-units-reach = Un ufficio forestale è competente per il bosco nel suo territorio. Il suo territorio può comprendere anche città e campi.
permits-state-forest-office = competente: { $office }
permits-hint-forest-office = { $units ->
        [none] Ufficio forestale competente per il bosco nell'area della carta: { $office } ({ $administration }). Rivolgiti lì per tempo se prevedi una manifestazione nel bosco.
       *[other] Ufficio forestale competente per il bosco nell'area della carta: { $office } ({ $administration }), { $units }. Rivolgiti lì per tempo se prevedi una manifestazione nel bosco.
    }
permits-hint-forest-owner = { $office ->
        [known] Una manifestazione nel bosco richiede il consenso dei proprietari del bosco. L'ufficio forestale di solito sa dire chi possiede il bosco. In diversi Land una manifestazione organizzata nel bosco va anche notificata all'autorità forestale o da essa autorizzata.
       *[unknown] Una manifestazione nel bosco richiede il consenso dei proprietari del bosco. In diversi Land una manifestazione organizzata nel bosco va anche notificata all'autorità forestale o da essa autorizzata. Queste note non nominano l'ufficio forestale competente qui; te lo può dire l'amministrazione forestale del Land o l'autorità forestale inferiore.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Un evento nel bosco di solito richiede il consenso del proprietario del bosco. Il distretto forestale (Nadleśnictwo) indicato sopra di solito sa dire chi si occupa del bosco. Il bosco che non appartiene alle Foreste di Stato, ad esempio un bosco privato, non è nei dati di queste note. Il bosco che non appartiene allo Stato, per esempio un bosco privato o comunale, è sotto la vigilanza dello starosta (starosta) per la gestione forestale (legge sulle foreste, art. 5 ust. 1 pkt 2); chieda lì presso lo starostwo powiatowe.
       *[unknown] Un evento nel bosco di solito richiede il consenso del proprietario del bosco. Queste note non indicano alcun distretto forestale delle Foreste di Stato per l'area della mappa; i dati comprendono solo le Foreste di Stato, quindi il bosco può appartenere a qualcun altro, ad esempio a un proprietario privato. Chi sia il proprietario va accertato separatamente. Il bosco che non appartiene allo Stato, per esempio un bosco privato o comunale, è sotto la vigilanza dello starosta (starosta) per la gestione forestale (legge sulle foreste, art. 5 ust. 1 pkt 2); chieda lì presso lo starostwo powiatowe.
    }
permits-hint-monument-register-pl = Nel rettangolo attorno all'area della mappa si trovano monumenti del registro nazionale. Le loro posizioni non sono nei dati: verifichi sulla mappa dove sono. Secondo l'art. 36 ust. 1 della legge sulla tutela dei monumenti (ustawa o ochronie zabytków), i lavori di conservazione, restauro ed edili su un monumento iscritto richiedono il permesso del conservatore dei monumenti del voivodato (Wojewódzki Konserwator Zabytków). Dal 3 giugno 2026 i lavori edili nei dintorni di un monumento, il cambio della sua destinazione, i dispositivi o i cartelli applicati su di esso e altre azioni che potrebbero modificarne la sostanza o l'aspetto richiedono solo una comunicazione al conservatore (art. 36aa); il conservatore può opporsi entro 60 giorni e allora richiedere un permesso. La legge non nomina una corsa o un controllo in quanto tali; chieda all'ufficio del conservatore se un evento tocca un monumento o i suoi dintorni.
permits-hint-archaeology-register-pl = I monumenti archeologici del registro nazionale si trovano nel rettangolo attorno all'area della mappa. Le loro posizioni non sono nei dati, verificate sulla mappa dove sono. Secondo l'art. 36 ust. 1 della legge sulla tutela dei monumenti (ustawa o ochronie zabytków), le ricerche archeologiche (pkt 5), la ricerca di monumenti mobili nascosti o abbandonati con dispositivi elettronici o tecnici come i metal detector (pkt 12) e i lavori di conservazione, restauro o edili su un monumento iscritto (pkt 1) richiedono il permesso del conservatore dei monumenti del voivodato (Wojewódzki Konserwator Zabytków). Il permesso per le ricerche decade il 1 gennaio 2027: l'art. 36b consente allora a una persona maggiorenne di cercare con il consenso del proprietario e del possessore dell'immobile dopo una comunicazione nel registro delle ricerche tramite l'app del ministero della cultura, e l'art. 36c vieta tali ricerche, tra l'altro, sui monumenti iscritti e a meno di 10 m da essi. La legge non nomina una camminata, una corsa o un controllo come tali; chiedete all'ufficio del conservatore se una manifestazione o un lavoro sul campo tocca un sito iscritto.
permits-hint-history-register-pl = Un monumento della storia (Pomnik historii) si trova nell'area della mappa o vicino ad essa; la sezione sui monumenti della storia dice quale. Il presidente della Repubblica di Polonia dichiara con regolamento Pomnik historii un monumento immobile iscritto nel registro, o un parco culturale, di particolare valore (art. 15 ust. 1 della legge sulla tutela dei monumenti, ustawa o ochronie zabytków). È quindi già sotto la tutela del registro o del parco culturale. Per i lavori su un monumento iscritto nel registro il conservatore dei monumenti del voivodato (Wojewódzki Konserwator Zabytków) deve rilasciare un permesso (art. 36 ust. 1); in un parco culturale la delibera del consiglio della gmina stabilisce i divieti e i limiti (art. 16 ust. 2, art. 17 ust. 1). La legge non nomina una passeggiata, una corsa o un controllo in quanto tali; chieda all'ufficio del conservatore se un evento o un lavoro sul terreno riguarda il monumento.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] L'area della mappa comprende un monumento naturale (pomnik przyrody), un'area ecologica (użytek ekologiczny), un complesso naturale e paesaggistico (zespół przyrodniczo-krajobrazowy) o un sito di documentazione (stanowisko dokumentacyjne). Il consiglio della gmina li istituisce con una delibera, che indica chi vigila sull'oggetto e quali divieti si applicano (art. 44 della legge sulla tutela della natura). La legge stessa non prevede qui alcun permesso per un evento. Chieda la delibera e il responsabile della vigilanza all'ufficio della gmina nel cui territorio si trova l'oggetto.
       *[other] L'area della mappa comprende un monumento naturale (pomnik przyrody), un'area ecologica (użytek ekologiczny), un complesso naturale e paesaggistico (zespół przyrodniczo-krajobrazowy) o un sito di documentazione (stanowisko dokumentacyjne). Il consiglio della gmina li istituisce con una delibera, che indica chi vigila sull'oggetto e quali divieti si applicano (art. 44 della legge sulla tutela della natura). La legge stessa non prevede qui alcun permesso per un evento. Chieda la delibera e il responsabile della vigilanza { $count ->
        [one] all'ufficio della gmina ({ $gmina }) nel cui territorio si trova l'oggetto
       *[other] agli uffici delle gmine ({ $gmina }), ciascuno per gli oggetti del proprio territorio
    }.
    }
permits-hint-gmina-roads-pl = Un evento che ostacola il traffico o usa una strada in modo particolare richiede un permesso ai sensi dell'art. 65 del codice della strada, da richiedere almeno 30 giorni prima. Lo rilascia l'organo che gestisce il traffico sulla strada: lo starosta per le strade del powiat e della gmina, il sindaco in una città con diritti di powiat, il maresciallo del voivodato o la GDDKiA per le altre strade (art. 10 e 65b). Il gestore di una strada della gmina è il wójt, burmistrz o prezydent miasta { $count ->
        [one] della gmina ({ $gmina });
       *[other] di ciascuna delle gmine ({ $gmina });
    } il gestore decide sull'occupazione della sede stradale (art. 19 e 40 della legge sulle strade pubbliche). Queste note non mostrano la categoria delle strade.
permits-hint-gmina-land-pl = Se un terreno dell'area della mappa appartiene alla gmina, per esempio un parco comunale, una piazza, un'area scolastica o un bosco, lo amministra il wójt, burmistrz o prezydent miasta, che rappresenta la gmina (art. 30 e 31 della legge sull'autogoverno della gmina). Chieda il consenso { $count ->
        [one] all'ufficio della gmina ({ $gmina }) prima
       *[other] agli uffici delle gmine ({ $gmina }) prima
    } di usarlo. Queste note non mostrano di chi sia il terreno.
permits-hint-protected-areas-cz = Nell'area della mappa si trovano aree o oggetti protetti per la tutela della natura. Secondo la legge sulla tutela della natura e del paesaggio (legge n. 114/1992 Sb.), in un parco nazionale sono vietati gli eventi sportivi, turistici e altri eventi pubblici organizzati al di fuori dei luoghi che l'autorità per la tutela della natura ha destinato a questo scopo (§ 16 odst. 1 písm. q); nella zona di protezione di un parco nazionale gli eventi di massa di questo tipo al di fuori di quei luoghi e al di fuori delle aree edificate richiedono il consenso dell'autorità (§ 37 odst. 3 písm. d). In una riserva naturale nazionale è vietato entrare o transitare con veicoli fuori dai sentieri segnalati con il consenso dell'autorità (§ 29 písm. d). Le autorità per la tutela della natura sono, tra le altre, l'amministrazione di un parco nazionale, l'Agenzia per la tutela della natura (AOPK ČR), l'ufficio regionale (krajský úřad) e l'ufficio del comune con competenze estese (§ 75 odst. 1). Le altre aree e gli altri oggetti protetti hanno condizioni di tutela proprie. Si rivolga per tempo all'autorità se prevede un evento in quel luogo.
permits-hint-monuments-cz = Nell'area della mappa si trovano monumenti culturali o aree monumentali dell'elenco statale. Secondo la legge sulla tutela del patrimonio statale (legge n. 20/1987 Sb., § 14 odst. 1 e 2), il proprietario di un monumento deve chiedere in anticipo una decisione o un parere vincolante prima della manutenzione, della riparazione, della ricostruzione o di un'altra modifica del monumento o del suo ambiente circostante. Lo stesso vale per il proprietario di un immobile in una riserva monumentale, in una zona monumentale o nella zona di protezione di un monumento per una costruzione, la modifica di una costruzione, lavori sul terreno, un impianto o un'insegna, la rimozione di una struttura, la cura degli alberi o lavori di manutenzione. Decide l'ufficio del comune con competenze estese{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, per un monumento culturale nazionale l'ufficio regionale. La legge non nomina alcuna camminata, corsa o punto di controllo in quanto tali; chieda a quell'ufficio quando un evento tocca un monumento o vi colloca impianti o insegne.
permits-hint-forest-owner-cz = Un evento di massa sportivo, culturale o di altro tipo sociale su terreni destinati alle funzioni forestali può svolgersi solo con il consenso scritto del proprietario del bosco in cui ha luogo (legge sulle foreste, legge n. 289/1995 Sb., § 20 odst. 5). L'organizzatore deve inoltre comunicare per iscritto l'evento all'ufficio del comune con competenze estese{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, oppure all'ufficio regionale se l'evento si estende oltre il distretto di tale comune, al più tardi 30 giorni prima dell'evento. La comunicazione indica data e luogo, come è organizzato il servizio, il numero previsto di partecipanti e i consensi scritti dei proprietari (stesso paragrafo). Queste note non mostrano chi possiede il bosco; chieda per tempo ai proprietari del bosco.
permits-hint-roads-cz = Un evento che usa una strada in modo diverso dal solito, per esempio un evento sportivo, culturale o simile che potrebbe mettere in pericolo la sicurezza o la fluidità del traffico stradale, richiede il permesso dell'ufficio competente per le strade. Viene rilasciato con il previo consenso del proprietario della strada e, dove la sicurezza o la fluidità del traffico possono essere compromesse, della Polizia della Repubblica Ceca (legge sulle strade, legge n. 13/1997 Sb., § 25 odst. 1 e odst. 6 písm. d). L'ufficio è l'ufficio comunale per le strade locali, oppure l'ufficio del comune con competenze estese dove questo decide, e l'ufficio del comune con competenze estese per le strade di seconda e terza classe (§ 40 odst. 4 e 5). Comuni nell'area della mappa: { $gmina }. Queste note non mostrano la classe delle strade.
permits-section-water-protection = Aree di protezione delle acque ({ $source })
permits-section-forest-functions = Funzioni del bosco e bosco protetto ({ $source })
permits-none-water-protection = Secondo { $source }, nell'area della carta non si trova alcuna area di protezione delle acque.
permits-none-forest-functions = Secondo { $source }, nell'area della carta non si trova alcun bosco ricreativo, alcun bosco protetto, alcun bosco naturale e alcuna zona boschiva chiusa.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Bosco ricreativo (Erholungswald)
        [protected] Bosco protetto (Bannwald o Schutzwald)
        [natural] Bosco naturale o riserva integrale (Naturwald, Totalreservat)
       *[closed] Zona boschiva chiusa
    }
permits-hint-water-protection = { $inner ->
        [yes] La { $zone } dell'area di protezione delle acque "{ $name }" si trova nell'area della carta. Nelle zone I e II le manifestazioni di solito sono escluse. Rivolgiti all'autorità inferiore per le acque (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] La { $zone } dell'area di protezione delle acque "{ $name }" si trova nell'area della carta. Lì le manifestazioni di solito sono possibili; l'autorità inferiore per le acque (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) può dire cosa consente il regolamento.
       *[unknown] L'area di protezione delle acque "{ $name }" si trova nell'area della carta. I dati non dicono di quale zona si tratti. Nelle zone I e II le manifestazioni di solito sono escluse; rivolgiti all'autorità inferiore per le acque (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Nell'area della carta si trova bosco ricreativo (Erholungswald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Per le manifestazioni lì, rivolgiti all'autorità forestale.
        [protected] Nell'area della carta si trova bosco protetto (Bannwald o Schutzwald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Rivolgiti all'autorità forestale prima di prevedere una manifestazione lì.
        [natural] Nell'area della carta si trova un bosco naturale o una riserva integrale{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Lì spesso è vietato lasciare i sentieri; non collocarvi punti di controllo senza chiedere all'autorità forestale.
       *[closed] Nell'area della carta si trova una zona boschiva in cui non si deve entrare{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Tienila fuori dal percorso.
    }
permits-section-roads = Strade classificate secondo OpenStreetMap
permits-none-roads = Secondo OpenStreetMap, nessuna Autobahn e nessuna Bundes-, Landes-, Staats- o Kreisstraße attraversa l'area della carta.
permits-road-line = { $class ->
        [autobahn] Autobahn (autostrada)
        [bundesstrasse] Bundesstraße (strada federale)
        [landesstrasse] Landesstraße o Staatsstraße (strada del Land)
       *[kreisstrasse] Kreisstraße (strada circondariale)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] L'Autobahn { $refs } attraversa l'area della carta. Un percorso può attraversarla solo su ponti o in sottopassi.
        [bundesstrasse] La Bundesstraße { $refs } attraversa l'area della carta. Una chiusura o un attraversamento con addetti è disposto dall'autorità per la circolazione stradale (Straßenverkehrsbehörde); la strada di solito è mantenuta dal Bund tramite l'amministrazione stradale del Land, nelle città più grandi spesso dalla città.
        [landesstrasse] La Landes- o Staatsstraße { $refs } attraversa l'area della carta. Una chiusura o un attraversamento con addetti è disposto dall'autorità per la circolazione stradale (Straßenverkehrsbehörde); la strada di solito è mantenuta dal Land, nelle città più grandi spesso dalla città.
       *[kreisstrasse] La Kreisstraße { $refs } attraversa l'area della carta. Una chiusura o un attraversamento con addetti è disposto dall'autorità per la circolazione stradale (Straßenverkehrsbehörde); la strada di solito è mantenuta dal Kreis, nelle città più grandi spesso dalla città.
    }
permits-hint-state-forest = { $office ->
        [none] Secondo i dati aperti del Land, { $share } dell'area della carta è foresta demaniale del proprietario "{ $owner }".
       *[other] Secondo i dati aperti del Land, { $share } dell'area della carta è foresta demaniale del proprietario "{ $owner }". Competente: { $office }.
    }
