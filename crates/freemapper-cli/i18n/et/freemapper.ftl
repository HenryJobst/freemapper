# Program texts of freemapper, Estonian (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } ei ole keel, milles freemapperil on tekstid; saadaval: { $available }

## init

cli-part-needs-osm = --part valib OSM-i ala osa ja nõuab lülitit --osm; lülitiga --place tuleb id enne üles otsida ja seejärel kasutada --osm <id> --part <n>
cli-reading-file = { $path } lugemine
cli-no-area-for-place = kohale „{ $query }“ ei leitud ala. Kui koht on mitmest eraldi osast koosnev ala, tuleb osa valida lülitiga --osm <id> --part <n> või kasutada --geojson
cli-using-place = Kasutatakse kohta { $name } ({ $id })
cli-created-project = Projekt { $path } loodud

## Cache

cli-no-cache-dir = sellel süsteemil pole vahemälukataloogi; tuleb anda --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Alla laaditud 1 ruut
       *[other] Alla laaditud { $count } ruutu
    }
cli-wrote = Kirjutatud { $path }
cli-kept-map = { $path } jäi alles (see võib sisaldada kasutaja tööd)
cli-map-lacks-templates = Kaart ei viita veel pildimallidele. Need tuleb lisada OOM-is (Mallid > Ava mall …) või kaart asendada lülitiga --force, mis jätab vana alles failina .omap.bak.
cli-map-lacks-buildings = Kaardil pole veel ALKIS-i hooneid. Nende lisamiseks tuleb kaart asendada lülitiga --force, mis jätab vana alles failina .omap.bak.
cli-map-lacks-canopies = Kaardil puuduvad ALKIS-i katused ja autokatused (522), mis on nüüd kaardiobjektid. Nende lisamiseks tuleb käivitada lülitiga --force, mis jätab vana kaardi alles failina .omap.bak.
cli-map-light = Aluskaart: { $joined } joont liidetud, { $left } peidetud või liiga väikest jäeti kandidaatkihtidesse (midagi ei nihutatud)
cli-map-light-kept = Kandidaatkihid sisaldavad nüüd ka { $left } objekti või lõiku, mille see versioon aluskaardilt eemaldab (märgitud freemapper:generalized). Säilitatud kaardil on need alles, seega paistavad need kaks korda. Kaardi puhastamiseks tuleb see asendada lülitiga --force (vana jääb failina .omap.bak).
cli-map-old-candidates = Kaart nimetab endiselt faili templates/candidates.omap; see käivitus täitis selle kõigi kandidaatidega. Kandidaadid tulevad nüüd ka seitsmes kihis, igaüks eraldi failis (templates/candidates-*.omap). Kaart loetleb need pärast asendamist lülitiga --force (vana jääb failina .omap.bak) või need saab ise OOM-is lisada (Mallid > Ava mall …).

## PDF

cli-pdf-without-credits = PDF ilma allikaviideteta: ODbL ja dl-de/by-2-0 nõuavad allikate nimetamist kaardi edastamisel; vt ATTRIBUTION.txt
cli-wrote-pdf = Kirjutatud { $path } (kaart ilma kandidaatideta)
cli-wrote-pdf-candidates = Kirjutatud { $path } (kontrollprint: kaart kõigi kandidaatidega, asukohatruult; neid tuleb maastikul kontrollida)
cli-wrote-training = Kirjutatud { $map } ja { $pdf } (treeningkaart: { $joined } liidetud, { $shifted } nihutatud, { $left } välja jäetud või lõigatud)
cli-training-free-ends = Vabaks jäetud joonte otsad: { $ways } kohas, kus tee kohtus teise teega, { $others } muude joonte juures
cli-training-apart-left = treeningkaardil on veel { $length } mm (paberil), kus objektide vahe on liiga väike; see tuleb OOM-is üle kontrollida
cli-kept-training = Treeningkaart { $path } jäi alles, et käivitus ei kirjutaks üle välja prinditud või edasi antud kaarti; selle uuendamiseks tuleb see kustutada või anda --force

## OCAD

cli-ocad-kept = { $path } jäi alles (see võib sisaldada kasutaja tööd; --force asendab selle ja jätab vana alles failina .ocd.bak)
cli-ocad-old-candidates = OCAD-i kaart nimetab endiselt faili { $path } taustakaardina; see käivitus täitis selle kõigi kandidaatidega. Kandidaadid tulevad nüüd ka seitsmes kihis, igaüks eraldi failis (-candidates-*.ocd). OCAD-i kaart loetleb need pärast asendamist lülitiga --force (vana jääb failina .ocd.bak) või need saab ise OCAD-is taustakaartidena lisada.
cli-ocad-wrote = { $count ->
        [one] Kirjutatud { $path } (1 objekt)
       *[other] Kirjutatud { $path } ({ $count } objekti)
    }
cli-ocad-left-out = OCAD-i kaardilt jäid välja: { $skipped } objekti (OCAD-i sümbolit pole, ala puudub või asub väljaspool OCAD-i joonistusala ±2 m) ja { $dropped } degenereerunud joonosa või auku.
cli-ocad-load-symbols = OCAD-is tuleb valida Symbol > Load Colors and Symbols From… ja valida ametlik sümbolikomplekt{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF-i koolikomplekt 2019
       *[other] {" "}ISSprOM 2019
    }; see asendab freemapperi kohatäitesümbolid numbri järgi.

## select

cli-select-running = Ala valik kaustale { $path }: lehel tuleb vajutada „Valmis“ või peatamiseks Ctrl+C.
cli-select-no-browser = Brauserit ei saanud avada ({ $error }); ülaltoodud aadress tuleb avada käsitsi.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; ala puudub, ei saa olla kaardiala): { $place }
cli-find-hint = Projekti loomiseks: freemapper init <dir> --osm { $id }
cli-find-no-area = Ühelgi neist kohtadest pole ala, mis sobiks kaardialaks. Ala saab joonistada käsuga: freemapper select <dir>
cli-find-nothing = päringule „{ $query }“ ei leitud kohta; tuleb proovida teist nime või joonistada ala käsuga: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = hoiatus: kaustas { $folder } olev adapter jäeti välja: { $finding }
cli-adapter-note = hoiatus: kaustas { $folder } olev adapter laaditakse, kuid: { $finding }
cli-adapter-from-project = hoiatus: projektikaustast pärit adapterit { $adapter } kasutatakse; see võib pöörduda nende hostide poole: { $hosts }
cli-adapter-overlaps = hoiatus: adapter { $adapter } katab ka alasid, mida katab { $others }; selle objektid satuvad seal kaartidele
cli-adapter-dir-relative = hoiatus: FREEMAPPER_ADAPTER_DIR ei ole absoluutne tee ({ $path }); seda ignoreeritakse
cli-adapters-no-project = hoiatus: { $path } ei ole kaust; projekti adaptereid ei loetud
cli-adapters-overlaps = jagab ala adapteritega: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  versioon { $revision }  kontrollitud { $checked }  hostid: { $hosts }
cli-adapters-source-built-in = sisseehitatud
cli-adapters-source-user = kasutaja kaustast
cli-adapters-source-project = projektist
cli-adapters-never-checked = mitte kunagi
cli-adapters-user-dir = Adaptereid loetakse kaustast { $path }
cli-adapters-no-user-dir = Sellel süsteemil pole adapterite kausta; selle nimetab FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper võib nüüd küsida ka hosti { $host } piirkonna { $name } jaoks. Kirje on failis { $file }; `freemapper adapters disallow-host { $adapter } { $host }` võtab selle tagasi. Töötav alavaliku leht või tehisintellekti assistent loeb kirje uuesti, kui see taaskäivitatakse
cli-allow-host-already = { $host } on piirkonnale { $name } juba lubatud
cli-allow-host-declared = piirkonna { $name } adapter nimetab hosti { $host } ise; lubada pole midagi
cli-allow-host-refused = hosti { $host } ei saa lubada: { $reason }
cli-allow-host-unknown-adapter = sisseehitatud adapterit { $adapter } pole; sisseehitatud adapterid on: { $adapters }
cli-allow-host-folder-adapter = { $adapter } on adapterite kausta adapter; host tuleb hoopis lisada selle manifesti jaotisesse `[net] hosts`
cli-allow-host-no-config = sellel süsteemil pole seadistuskausta lubatud hostide hoidmiseks
cli-allow-host-write = faili { $file } ei saanud kirjutada: { $detail }
cli-allow-host-unreadable = faili { $file } ei saa lugeda ({ $detail }); fail tuleb esmalt parandada või eemaldada; midagi ei muudetud
cli-disallow-host-removed = freemapper ei küsi enam hosti { $host } piirkonna { $name } jaoks
cli-disallow-host-absent = { $host } ei olnud piirkonnale { $name } lubatud; tagasi võtta pole midagi
cli-allowed-hosts-unreadable = hoiatus: faili { $file } ei saanud lugeda ({ $detail }); lisahoste ei lubata
cli-allowed-hosts-ignored = hoiatus: { $file }: kirjet { $host } adapterile { $adapter } ignoreeritakse: { $reason }
cli-allowed-hosts-no-adapter = freemapperil pole sellise nimega sisseehitatud adapterit
cli-allowed-host-redundant = hoiatus: { $host } on nüüd üks hostidest, mille piirkonna { $name } adapter ise nimetab; kasutaja kirje failis { $file } eemaldati
cli-adapters-allowed = kasutaja lubatud: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), versioon { $revision }, hostid: { $hosts }
cli-check-error = viga: { $finding }
cli-check-warning = hoiatus: { $finding }
cli-check-passed = { $folder } läbib kontrolli
cli-check-passed-warnings = { $folder } läbib kontrolli, { $count ->
        [one] ühe hoiatusega
       *[other] { $count } hoiatusega
    }
cli-check-failed = { $folder } ei läbi kontrolli
cli-check-no-folder = kausta { $folder } pole
cli-check-live-ok = { $capability } korras: { $detail }
cli-check-live-failed = { $capability } EBAÕNNESTUS: { $detail }
cli-check-live-not-asked = { $capability } jäeti küsimata: { $detail }
cli-check-live-box-outside = kast ei asu adapteri { $adapter } piiri sees; reaalajakontroll küsib teenustelt kohta, mida adapter katab
cli-check-live-cannot = reaalajakontrolli ei saa käivitada: { $detail }
cli-check-detail-not-declared = adapter ei deklareeri ühtegi
cli-check-detail-stopped = varasem osa ebaõnnestus
cli-check-detail-see-warnings = vt hoiatusi
cli-check-detail-outside-dataset = kast asub andmestikust väljas
cli-check-detail-objects = { $count ->
        [one] üks objekt
       *[other] { $count } objekti
    }
cli-check-detail-layers = { $layers } kihti, { $count } objekti
cli-check-detail-trees = { $count ->
        [one] üks puu
       *[other] { $count } puud
    }
cli-check-detail-extract = { $label }, andmed kuupäevast { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = maastikumudel
cli-check-raster-surface = pinnamudel
cli-check-raster-tiles = { $product }: { $count ->
        [one] üks ruut
       *[other] { $count } ruutu
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } ja { $last }

## Folder lock

cli-lock-held = see kaust on teise freemapperi käivituse kasutuses (alates { $time }, käivitaja: { $who }). Tuleb oodata või see esmalt peatada.
cli-lock-who-cli = terminal
cli-lock-who-mcp = tehisintellekti agent (MCP)
cli-lock-who-page = valikuleht
cli-lock-who-unknown = teine programm
cli-lock-failed = projektikausta ei saanud selle käivituse jaoks lukustada: { $error }
cli-lock-stuck = teel on midagi muud kui lukufail ja seda ei saanud eemaldada

## Run routes of the selection page

cli-run-bad-request = Käivitusvalikud ei kehti: { $error }
cli-run-no-project = Selles kaustas pole veel salvestatud projekti. Esmalt tuleb ala salvestada.
cli-run-already-running = Selle projekti käivitus ({ $job }) juba käib. Tuleb oodata selle lõppu või see esmalt tühistada.
cli-run-disk-stops = Allalaadimiseks pole piisavalt vaba kettaruumi: { $messages }
cli-run-busy-elsewhere = Selle programmi teine projekt käivitab praegu torujuhet. Projektid jagavad allalaadimise vahemälu, seetõttu käib korraga ainult üks käivitus. Tuleb proovida uuesti, kui see on lõppenud.
cli-run-nothing-to-cancel = Ükski käivitus ei käi, seega pole midagi tühistada.
cli-run-unknown-job = Sellel projektil pole käivitust { $job }.
cli-status-too-fast = Olekupäringuid tuleb liiga sageli. Küsida võib kuni viis korda sekundis.
cli-open-folder-failed = Kausta ei saanud avada ({ $error }).
cli-open-folder-no-opener = Sellel arvutil pole programmi kaustade avamiseks ({ $program }). Kaust tuleb avada käsitsi.
cli-open-folder-missing = Projektikausta pole veel olemas. Esmalt tuleb ala salvestada.
cli-open-folder-refused = Kausta nime ei ole ohutu failihaldurile edasi anda.
cli-save-running = Selle projekti käivitus käib. Salvestada saab uuesti, kui see on lõppenud.
cli-save-unknown-kind = Tundmatu kaarditüüp „{ $kind }“. Kasutada tuleb schoolyard, sprint või forest.
cli-save-scale-too-large = Mõõtkava { $scale } on liiga suur.
cli-save-bad-content = Sisuseaded ei kehti. Midagi ei salvestatud.
cli-map-image-none = Esmalt tuleb kaart luua.
cli-map-image-gone = See eelvaade pole enam saadaval.
cli-map-image-running = Selle projekti käivitus käib. Eelvaate saab arvutada, kui see on lõppenud.
cli-map-image-stale = Salvestatud projekt ei vasta enam loodud kaardile, näiteks seetõttu, et ala, kaarditüüp või mõõtkava muutus. Esmalt tuleb käivitada käivitus, mis kaardi asendab („Asenda olemasolevad kaardid“ lehel või freemapper run --force).
cli-map-image-superseded = Vahepeal küsiti uuemat eelvaadet, seetõttu seda ei arvutatud.
cli-map-image-bad-content = Sisuseaded ei kehti.
cli-map-image-bad-request = Eelvaate päring ei kehti. Tuleb saata JSON-objekt, vajaduse korral sisuseadetega.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – ala valik
page-heading = Vali kaardiala
page-update-available = freemapper { $version } on saadaval. Värskenda käsuga
page-update-notes = Väljalaskemärkmed
page-folder = Projektikaust
page-pick = Otsi kohta
page-query-placeholder = nt Clara-Zetkin-Schule Strausberg
page-search-button = Otsi
page-searching = Otsin …
page-no-results = Midagi ei leitud.
page-point-only = ainult punkt, mitte ala
page-draw = Joonista ise
page-rectangle = Ristkülik
page-polygon = Hulknurk
page-finish-polygon = Sulge hulknurk
page-clear = Tühjenda
page-hint-rectangle = Klõpsa kaardil kahte vastasnurka.
page-hint-polygon = Klõpsa nurki üksteise järel. Sulgemiseks klõpsa esimest nurka, tee topeltklõps või klõpsa „Sulge hulknurk“. Esc tühistab, Backspace eemaldab viimase nurga.
page-hint-idle = Vali koht otsingust või joonista ala ise.
page-summary = Ala
page-source-osm = OpenStreetMapist: { $name }
page-source-drawn = Käsitsi joonistatud
page-size = Suurus koos puhvriga: { $width } × { $height } m
page-tiles-one = { $n } ruut: { $names }
page-tiles-other = { $n } ruutu: { $names }
page-download = Allalaadimine: umbes { $mb } MB
page-download-none = Allalaadimine: kõik on juba vahemälus
page-download-details = Mida alla laaditakse
page-cached = vahemälus
page-disk = Kettaruum: { $volumes }
page-disk-volume = { $free } vaba, vaja umbes { $needed }
page-cache-volume = vahemälu
page-project-volume = projektikaust
page-no-room = ei mahu kettale
page-no-room-skip = Kõik ei mahu kettale (freemapper hoiab { $reserve } vaba). Allalaadimine jätab märgitud osad välja. Vabasta esmalt ruumi, kui neid vajad:
page-no-room-stop = Kettaruumi ei ole piisavalt (freemapper hoiab { $reserve } vaba). Märgitud osadeta kaarti ei saa, seetõttu allalaadimine peatub enne midagi alla laadimist. Vabasta esmalt ruumi:
page-disk-unknown = freemapper ei saa vaba kettaruumi lugeda ({ $error }). Allalaadimine toimub ilma kontrollita.
page-too-large = Ala on liiga suur: kuni { $max } m küljepikkus (koos puhvriga). Vali väiksem.
page-outside = Ala asub toetatud liidumaadest ({ $states }) väljas; freemapperil pole muude piirkondade jaoks veel andmeallikaid.
page-outside-unknown = Ala asub piirkondadest, mida freemapper toetab, väljas; seal pole veel andmeallikaid.
page-adapters-failed = Aerofotosid ei saa loetleda ({ $detail }). Kaart näitab ainult OpenStreetMapi; kõik muu töötab.
page-settings = Kaart
page-kind = Kaarditüüp
page-schoolyard = Koolihoovikaart (1:1000)
page-sprint = Sprindikaart (1:4000)
page-forest = Metsakaart (1:15000)
page-scale = Mõõtkava 1:
page-map-title = Pealkiri
page-contour-interval = Kõrgusjoonte vahe
page-contour-auto = automaatne
page-settings-fixed = Projekt on juba olemas; kaarditüüp, mõõtkava ja pealkiri on failis freemapper.toml. Kõrgusjoonte vahet saab siiski muuta.
page-save = Salvesta
page-saving = Salvestan …
page-saved = Salvestatud faili freemapper.toml.
page-zone-changed = Salvestatud. Ala asub nüüd teises UTM-tsoonis; freemapper laadib selle andmed järgmisel käivitusel uuesti alla.
page-provider-changed = Salvestatud. Uue ala kõrgus- ja aerofotoandmeid pakub teine andmepakkuja; freemapper laadib selle andmed järgmisel käivitusel uuesti alla.
page-save-stale = freemapper.toml muudeti teises aknas või teise programmi poolt pärast seda, kui see leht selle luges. Midagi ei salvestatud; leht teab nüüd salvestatud olekut. Salvesta uuesti, et asendada see siin näidatud alaga.
page-has-map = Olemasolev kaart (.omap) jääb samaks; freemapper teeb uuele alale uued mallid ja kandidaadid. Allolev valik „Asenda olemasolevad kaardid“ (või „freemapper run --force“) teeb ka kaardi uuesti ja jätab vana alles failina .omap.bak.
page-done = Valmis
page-source = Lähtekood (AGPL-3.0-or-later)
page-closed = Valmis. Võid selle akna sulgeda.
page-next = Järgmiseks: freemapper run "{ $dir }"
page-made-files = Loodud: { $files }. Kaust: { $dir }
page-back-to-agent = Anna oma tehisintellekti assistendile teada, et ala on salvestatud.
page-nothing-saved = Midagi ei salvestatud.
page-failed = Viga: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Aerofoto { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Mis läheb kaardile
page-content-help = Profiil otsustab, mis läheb otse kaardile ja mis jääb kontrollimist ootavaks kandidaadiks. Rühma valik ületab profiili. See salvestatakse koos alaga.
page-content-profile-legend = Profiil
page-content-profile-safe = Turvaline
page-content-profile-safe-hint = Iga objekt jääb sinna, kuhu koostamine selle paneb: hooned ja katastri alad lähevad kaardile; müürid, piirded, puud ja palju OpenStreetMapist (tiigid, teadmata pinnasega teed) jäävad kandidaatideks.
page-content-profile-balanced = Tasakaalustatud
page-content-profile-balanced-hint = Katastri, registrite ja tänavamõõdistuse objektid lähevad kaardile, koos kõrgusjoonte ja raudteede äärsete keelualadega. Müürid, piirded, hekid, nõlvad, trepid, vesi, puud, mälestusmärgid, läbipääsud, teed ja raudteed tulevad ka OpenStreetMapist. Tänavamööbel, muud keelualad ja maastikumudelist tuletatud vormid jäävad kandidaatideks.
page-content-profile-full = Täielik
page-content-profile-full-hint = Kõik läheb kaardile, nagu kontrollprindis; ainult peidetud kandidaadid jäävad välja.
page-content-groups-legend = Rühmade kaupa
page-content-groups-help = „Nagu profiilis“ järgib profiili. Mida see rühma jaoks tähendab, sõltub sellest, kust iga objekt pärineb.
page-content-place-profile = Nagu profiilis
page-content-place-map = Kaart
page-content-place-candidates = Kandidaat
page-content-place-off = Väljas
page-content-group-walls = Müürid
page-content-group-fences = Piirded
page-content-group-hedges = Hekid
page-content-group-slopes = Nõlvad ja kaljud
page-content-group-stairs = Trepid
page-content-group-trees = Puud ja puugrupid
page-content-group-water-features = Vesi, purskkaevud ja allikad
page-content-group-monuments = Tornid, mälestusmärgid ja elektriliinid
page-content-group-furniture = Tänavamööbel ja lambid
page-content-group-passages = Läbipääsud
page-content-group-contours = Kõrgusjooned
page-content-group-landforms = Künkad, lohud, nõlvad ja kaljud maastikumudelist
page-content-group-vegetation = Taimestik laserpunktipilvedest (Brandenburg, Berliin, Nordrhein-Westfalen, Rheinland-Pfalz, Saarimaa ja Baieri; väljas, kuni valid selle; laadib alla umbes 40 kuni 600 MB km² kohta)
page-content-group-out-of-bounds = Keelualad
page-content-group-paths = Teed ja raudteed
page-content-group-streets = Tänavad ja väljakud
page-run-heading = Loo kaart
page-preview-heading = Kaardi eelvaade
page-preview-stored = Nagu viimati loodud.
page-preview-live = Selle lehe seadetega, veel salvestamata.
page-preview-live-saved = Salvestatud seadetega, veel käivituses loomata.
page-preview-computing = Arvutan …
page-preview-none = Kaarti pole veel loodud.
page-preview-later = Sellel kaardil pole veel eelvaadet. See tuleb järgmise käivitusega, mis kaardi kirjutab.
page-preview-stale = Ala, kaarditüüp või mõõtkava on pärast kaardi loomist muutunud. Pilt näitab endiselt viimati loodud kaarti; uus eelvaade nõuab käivitust valikuga „Asenda olemasolevad kaardid“.
page-preview-area-unsaved = Selle lehe ala pole veel salvestatud. Pilt näitab viimati loodud kaarti; uus ala nõuab käivitust valikuga „Asenda olemasolevad kaardid“.
page-preview-run = Nagu viimati loodud. Leht uuendab pilti, kui käivitus lõpeb.
page-preview-open = Ava täissuuruses
page-preview-alt = Kaardiobjektide eelvaade
page-opt-ocad = Kirjuta OCAD-i algprojekt
page-help-ocad = Neile, kes jätkavad OCAD-is.
page-opt-pdf = Prinditav PDF
page-help-pdf = Ainult kaardiobjektid, kandidaadita.
page-opt-pdf-candidates = Kontrollprint kõigi kandidaatidega
page-opt-training = Treeningkaart
page-help-training = Kaart ja kõik kandidaadid, puhastatud nii, et seda saab printida just sellisena; OCAD-iga ka OCAD-i failina.
page-views-heading = Treeningkaardi vaated
page-view-no-paths = Ilma teedeta (Ultimate map)
page-view-brown-only = Ainult pruun (kõrgusjoonte orienteerumine)
page-view-no-north-lines = Ilma põhjajoonteta
page-view-custom = Muu vaade
page-help-views = Iga märgitud vaade on üks fail treeningkaardi kõrval, `<name>-training-<view>.omap` ja selle PDF, OCAD-iga ka OCAD-i fail. Muu vaade on `no:505,506` (sümbolid jäetakse välja) või `only:101,102` (jäävad ainult need); `+` ühendab vaated. Pealkiri, mõõtkava ja allikad jäävad alati.
page-help-views-forest = Ilma teedeta ja ainult pruun on mõeldud ainult metsakaartidele.
page-opt-scale-bar = Mõõtkavariba PDF-idel
page-help-scale-bar = Saksa kaartidel see tavaliselt puudub.
page-opt-sources = Allikaread PDF-idel
page-sources-auto = automaatne
page-sources-full = täielik
page-sources-credit-only = ainult freemapper
page-warn-credit-only = Siis nimetavad PDF-id ainult freemapperit. Andmelitsentsid (ODbL, dl-de/by-2-0) nõuavad kaardi edastamisel allikaviidet; siis nimeta allikad ise. ATTRIBUTION.txt loetleb need kõik.
page-need-pdf = Mõõtkavariba ja allikaread kehtivad ainult siis, kui tehakse PDF.
page-opt-offline = Ainult vahemälu, ilma võrguta
page-opt-force = Asenda olemasolevad kaardid
page-help-force = Kaart ja, kui „Treeningkaart“ on märgitud, ka treeningkaart. Vanad failid jäävad failidena .omap.bak.
page-opt-permits = Loamärkmed
page-help-permits = Haldusüksused, kaitsealad ja kellelt küsida, failis permits.md. Vihje, mitte õigusnõu.
page-run-start = Loo kaart
page-run-again = Loo uuesti
page-run-cancel = Tühista
page-run-cancelling = Tühistan …
page-warnings-heading = Hoiatused
page-where-files = Failid asuvad projektikaustas:
page-open-folder = Ava kaust
page-log-label = Käivituse logi
page-force-title = Asendada olemasolevad kaardid?
page-force-text = Olemasolev kaart asendatakse ja kui „Treeningkaart“ on märgitud, ka treeningkaart. Sinu muudatused ei lähe kaduma: vanad failid jäävad projektikaustas failidena .omap.bak. Jätkata?
page-force-cancel = Tühista
page-force-confirm = Asenda
page-step-fetch = Andmete allalaadimine
page-step-derive = Mallide arvutamine
page-step-map = Kaardi kirjutamine
page-step-ocad = OCAD-i projekti kirjutamine
page-step-pdf = PDF-i kirjutamine
page-step-pdf-candidates = Kontrollprindi kirjutamine
page-step-training = Treeningkaardi kirjutamine
page-step-permits = Loamärkmete kirjutamine
page-step-line = Samm { $index } / { $total }: { $label }
page-starting = Käivitus algab …
page-file-kind-map = kaart
page-file-kind-template = mall
page-file-kind-ocad = OCAD-i kaart
page-file-kind-ocad-candidates = OCAD-i kandidaadid
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrollprint
page-file-kind-training-map = treeningkaart
page-file-kind-training-pdf = treeningkaardi PDF
page-file-kind-training-ocad = OCAD-i treeningkaart
page-file-kind-training-view-map = treeningkaardi vaade
page-file-kind-training-view-pdf = treeningkaardi vaate PDF
page-file-kind-training-view-ocad = OCAD-i treeningkaardi vaade
page-file-kind-preview = kaardi eelvaade
page-file-kind-permits = loamärkmed
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Valmis. Failid on kirjutatud.
page-result-failed = Käivitus ebaõnnestus: { $error }
page-result-failed-kept = Kirjutatu jääb projektikausta; uus käivitus jätkab sealt.
page-result-cancelled = Tühistatud. Juba alla laaditu jääb vahemällu; uus käivitus jätkab sealt.
page-result-title-done = Kaart loodud
page-result-title-failed = Käivitus ebaõnnestus
page-result-title-cancelled = Käivitus tühistati
page-kept-map = Olemasolev kaart jäeti samaks. „Asenda olemasolevad kaardid“ teeb selle uuesti ja jätab vana alles failina .omap.bak.
page-no-files = Ühtegi faili ei tehtud.
page-opened-folder = Kaust on avatud.
page-open-failed = { $message } Kaust: { $dir }
page-why-no-area = Vali esmalt ala.
page-why-checking = Eelvaadet arvutatakse …
page-why-too-large = Ala on liiga suur.
page-why-outside = Ala asub toetatud liidumaadest väljas.
page-why-disk = Vaba kettaruumi ei ole piisavalt (vt ülal).
page-why-running = Käivitus käib.
page-err-scale-bar-no-pdf = Mõõtkavariba vajab PDF-i.
page-err-sources-no-pdf = Allikaread vajavad PDF-i.
page-save-failed = Ala pole salvestatud; käivitust ei alustatud.
page-poll-lost = Server ei vasta. freemapper proovib edasi.
page-run-unknown = Server ei tea seda käivitust enam.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Sulge

## update (spec 3.4 to 3.6)

update-source-build = See freemapper on ehitatud lähtekoodist. Värskendamiseks tuleb kasutada git pull ja cargo build --release --locked.
update-current = freemapper { $version } on ajakohane.
update-available = freemapper { $new } on saadaval (kasutusel { $old }), välja lastud { $released }.
update-details =
    Väljalaskemärkmed: { $notes }
    Allalaadimine: { $size } MB
update-decimal-separator = ,
update-ask = Kas installida nüüd? [j/E]
update-not-installed = Midagi ei installitud.
update-needs-yes = sisend ei ole terminal, seetõttu freemapper ei saa enne installimist küsida; küsimata installimiseks tuleb käivitada freemapper update --yes
update-installing = freemapper { $version } allalaadimine ja kontrollimine
update-done = freemapper uuendati versioonilt { $old } versioonile { $new }. Väljalaskemärkmed: { $notes }
update-mcp-restart = Töötav MCP-server jätkab vana versiooniga, kuni tehisintellekti assistent selle taaskäivitab.
update-rolled-back = Tagasi versioonil freemapper { $version }. Versioon, millest lahkuti, on alles failina { $path }.
update-not-writable = programmikausta { $path } ei saa kirjutada ja freemapper ei küsi administraatoriõigusi. Uus väljalase tuleb sinna käsitsi lahti pakkida või freemapper tuleb viia kausta, kuhu saab kirjutada.
update-too-old = freemapper { $new } ei saa installida versiooni { $old } isevärskendusega; { $needed } või uuem tuleb installida käsitsi, vt { $notes }
update-busy = programmikaustas töötab teine värskendus või tagasivõtmine (lukufail { $path }); tuleb oodata selle lõppu ja proovida uuesti
update-bad-signature = väljalaset ei saanud kontrollida ({ $error }); midagi ei installitud
update-check-bad-signature = uusimat freemapperi väljalaset ei saanud kontrollida ({ $error }); midagi ei installitud
update-no-manifest = uusim väljalase ei saa end ise installida (sellel pole värskendusmanifesti); tuleb see alla laadida aadressilt { $page }
update-no-manifest-version = väljalaset { $version } pole olemas või see ei saa end ise installida (sellel pole värskendusmanifesti); vt { $page }
update-no-asset = väljalases pole programmi sellele süsteemile ({ $platform }); midagi ei installitud
update-no-previous = eelmist versiooni, millele tagasi minna, pole ({ $path } puudub)
update-failed = värskendus ebaõnnestus: { $error }
update-mode-set = Värskendusrežiim seati väärtusele { $mode } failis { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } alistab selle seade, kuni see on määratud.
update-mode-failed = faili { $path } ei saanud kirjutada: { $error }
update-notice = freemapper { $new } on saadaval (kasutusel { $old }). Värskendamiseks: freemapper update
update-applies-next-start = Uus versioon rakendub freemapperi järgmisest käivitusest.
update-install-failed = freemapper { $new } ei installitud: { $error }
update-no-config-dir = sellel süsteemil pole seadistuskausta; tuleb määrata FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Märkus inimesele: oma käivitusel leidis freemapperi server adapterikaustade kohta järgmist. Anna see edasi; inimene parandab kausta või eemaldab selle ja käivitab seejärel serveri uuesti.
    { $lines }
update-mcp-note = Märkus inimesele: freemapper { $new } on saadaval (praegune on { $old }). Enne install_update kutsumist tuleb temalt luba küsida.
update-mcp-needs-confirm = midagi ei installitud; esmalt tuleb inimeselt küsida ja kutsuda install_update väärtusega confirm: true, kui ta nõustub
update-mcp-note-manual = Märkus inimesele: freemapper { $new } on saadaval (praegune on { $old }), kuid install_update ei saa seda installida; { $needed } või uuem tuleb installida käsitsi, vt { $notes }
update-mcp-note-bad-signature = Märkus inimesele: uusimat freemapperi väljalaset ei saanud kontrollida ({ $error }); midagi ei installitud. Tuleb temale teatada ja seda mitte muul viisil installida.
update-mcp-restart-now = Freemapperi MCP-server tuleb nüüd taaskäivitada; see töötab veel vana versiooniga.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = viga
error-cancelled = tühistatud
cli-cancelled-kept = tühistatud; alles jäid: { $files }
error-io = { $path }: { $message }
error-io-disk-full = ketas on täis, seetõttu freemapper ei saanud kirjutada { $path }. Tuleb ruumi vabastada ja proovida uuesti

## Errors: project

error-project-already-exists = projekt on juba olemas: { $path }
error-project-map-exists = kaart on juba olemas (see võib sisaldada kasutaja tööd); asendamiseks tuleb kasutada --force: { $path }
error-project-invalid-scale = kehtetu mõõtkava 1:{ $scale }
error-project-invalid-contour-interval = kehtetu kõrgusjoonte vahe { $interval } m failis freemapper.toml; see peab olema suurem kui 0
error-project-scale-not-allowed = mõõtkava 1:{ $scale } pole { $kind }kaartidele lubatud; kasutada tuleb { $allowed }
error-project-scale-not-allowed-enlarge = mõõtkava 1:{ $scale } pole { $kind }kaartidele lubatud; kasutada tuleb { $allowed }. Joonistada tuleb mõõtkavas 1:4000, freemapper-print suurendab printimisel mõõtkavasse 1:3000
word-kind-schoolyard = koolihoovi
word-kind-sprint = sprindi
word-kind-forest = metsa
error-project-toml = kehtetu projektifail: { $message }
error-project-unknown-adapter = projektifail nimetab adapterit „{ $adapter }“, mida see freemapper ei tunne; seda võib tuua uuem freemapper (`freemapper update`), see võib pärineda adapterite kaustast (`freemapper adapters list` näitab seda) või käsurea lülitiga --adapters-from-project projekti kaustast `adapters` (ainult usaldatud projekti puhul)
error-project-not-fetched = ruute pole veel alla laaditud; esmalt tuleb käivitada `freemapper fetch`
error-project-ocd = OCAD-i eksport: { $message }
error-project-template-outdated = { $path }: pilt ja world-fail ei vasta kaardialale; tuleb käivitada `freemapper derive` uuesti
error-project-tiles-outdated = kaardiala vajab ruute { $tiles }, mida ei laaditud alla; tuleb käivitada `freemapper fetch`
error-project-buildings-not-fetched = sellele kaardialale pole ALKIS-i hooneid salvestatud; esmalt tuleb käivitada `freemapper fetch` võrguühendusega
error-project-orthophoto-not-fetched = sellele kaardialale pole ortofotot salvestatud; esmalt tuleb käivitada `freemapper fetch` võrguühendusega
error-project-terrain-not-fetched = sellele kaardialale pole maastikumudelit salvestatud; esmalt tuleb käivitada `freemapper fetch` võrguühendusega
error-project-candidates-outdated = salvestatud OSM-i kandidaadid ei kata kaardiala; tuleb käivitada `freemapper fetch`
error-project-stored = { $path }: { $message }; tuleb käivitada `freemapper fetch` uuesti
error-project-area-too-large = kaardiala pluss puhver on { $width } × { $height } m, rohkem kui { $limit } m külje kohta; kaardiala või buffer_m failis freemapper.toml tuleb vähendada
error-project-area-crosses-country = kaardiala asub riigipiiri mõlemal küljel; piiriülesed kaardid pole veel toetatud, seetõttu tuleb ala joonistada ühe riigi sisse
error-project-permits-not-for-region = loamärkmed pole selle piirkonna jaoks veel saadaval: need küsivad Saksa teenustelt ja nimetavad Saksa ametiasutusi
error-project-outside-coverage = kaardiala asub toetatud liidumaadest ({ $states }) väljas; freemapperil on seni andmeallikad ainult nende jaoks

## Errors: disk space

error-space-required = kettaruumi pole piisavalt; puudutab: { $part }. Vaja on { $needed }, kuid { $path } juures on { $reserve } reservi järel vaba ainult { $left }. Tuleb ruumi vabastada ja käivitada `freemapper fetch` uuesti
error-space-skipped = freemapper jätab kettaruumi puudumise tõttu vahele: { $part }. Vaja on { $needed }, kuid { $path } juures on { $reserve } reservi järel vaba ainult { $left }. Tuleb ruumi vabastada ja käivitada `freemapper fetch` uuesti
word-part-map-data = ALKIS-i ja OSM-i kaardiandmed
word-part-terrain = maastikumudel
word-part-orthophoto = ortofoto
word-part-surface = nDSM-i malli pinnamudel
word-part-osm-update = vahemällu salvestatud OSM-i väljavõtte uuendus
word-part-k-five = Berliini K5 kaardi mall
word-part-berlin-layers = Berliini ALKIS-i rajatised, puukataster, tänavamõõdistus ja punktkihid
word-part-state-layers = täielikud ALKIS-i rajatised, maastikumudeli objektid ja puukatastrid
word-part-point-cloud = taimestiku kandidaatide laserpunktipilved

## Errors: downloads

error-fetch-http = { $url } allalaadimine ebaõnnestus: { $message }
error-fetch-corrupt-archive = { $url } allalaadimine ei ole täielik ruudu-arhiiv: { $message }
error-fetch-corrupt-extract = { $url } allalaadimine ei ole täielik OSM-i väljavõte: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] ruut { $tiles } ei ole avaldatud ruutudega kaetud; kaardiala või buffer_m failis freemapper.toml tuleb vähendada
       *[other] ruudud { $tiles } ei ole avaldatud ruutudega kaetud; kaardiala või buffer_m failis freemapper.toml tuleb vähendada
    }
error-fetch-missing-offline = { $count ->
        [one] võrguühenduseta ja pole vahemälus: { $product } ruut { $tiles }; tuleb käivitada `freemapper fetch` võrguühendusega
       *[other] võrguühenduseta ja pole vahemälus: { $product } ruudud { $tiles }; tuleb käivitada `freemapper fetch` võrguühendusega
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = maastikumudel (DGM1)
word-product-image-dsm = pinnamudel (bDOM)
word-product-saxony-dtm = maastikumudel (GeoSN DGM1)
word-product-saxony-dsm = pinnamudel (GeoSN DOM1)
error-fetch-osm-extract-missing = OSM-i väljavõtet pole vahemälus; tuleb käivitada `freemapper fetch` võrguühendusega
error-fetch-not-found = { $name } ei leitud serverist
error-fetch-refused = { $url }: server keeldus päringust (HTTP { $status })
error-fetch-refused-detail = { $url }: server keeldus päringust (HTTP { $status }): { $detail }
error-fetch-too-large = { $url } allalaadimine on suurem kui { $limit } MB; peatati
error-fetch-not-stored = sellele kaardialale puudub salvestatud { $label }: { $path } puudub, seda ei saa lugeda või see ei kata kaardiala; esmalt tuleb käivitada `freemapper fetch` võrguühendusega
error-fetch-range-ignored = { $url }: server ignoreeris baidivahemikku { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper ei saada sellele aadressile päringuid: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper ei saada sellele aadressile päringuid: piirkonna { $name } adapter ei nimeta hosti { $host }{ $from }. Kui teenus on sinna kolinud, tuleb host lubada käsuga `freemapper adapters allow-host { $adapter } { $host }` ja sellest teatada aadressil https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper ei saada sellele aadressile päringuid: adapter { $adapter } ei nimeta hosti { $host }{ $from }. Kui teenus on sinna kolinud, tuleb host lisada adapteri manifesti jaotisesse `[net] hosts`
error-fetch-host-redirected-from = {" "}({ $url } suunab sinna)
error-reason-not-a-host-name = pole hostinimi
error-reason-not-ascii = väljaspool ASCII-d olev nimi tuleb kirjutada xn-- kujul
error-reason-not-a-dns-name = pole DNS-nimi
error-reason-no-domain = selle arvuti nimi või domeenita nimi
error-reason-local-name = kohtvõrgu nimi
error-reason-ip-address = IP-aadress, mitte DNS-nimi
error-reason-unexpected-character = ootamatu märk
error-reason-not-https = ei ole https
error-reason-user-info = kasutajateave
error-reason-port = muu port kui 443
error-reason-host-not-declared = host ei ole deklareeritud hostide hulgas
error-reason-local-address = selle arvuti või kohtvõrgu aadress, mitte internetis oleva serveri oma
error-reason-too-many-redirects = rohkem kui { $count } ümbersuunamist
error-reason-redirect-nowhere = ümbersuunamine (HTTP { $status }) eikuhugi
error-fetch-currency-csv = valuuta-CSV rida { $line }: { $message }
error-alkis = kehtetud ALKIS-i andmed: { $message }
error-osm = OSM-i andmed: { $message }

## Errors: templates and rasters

error-derive-zip = ruudu-arhiiv: { $message }
error-derive-image = pilt: { $message }
error-derive-no-tiles = pole ruute, millest tuletada
error-derive-area-too-large = kaardiala on ühe malli jaoks liiga suur: { $width } × { $height } pikslit (kuni { $limit } külje kohta); kaardiala või buffer_m failis freemapper.toml tuleb vähendada
error-raster-world-file = kehtetu world-fail: { $message }
error-raster-misaligned = rasterid ei ole joondatud samale pikslivõrgule: { $message }
error-raster-pixel-size = rasteritel on erinev piksli suurus: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = kehtetu GeoJSON: { $message }
error-area-no-polygon = GeoJSON ei sisalda polügooni
error-area-multi-polygon = MultiPolygon { $parts } osaga; kaardiala tuleb jagada üheks polügooniks
error-area-out-of-range = koordinaat vahemikust väljas: lat { $lat }, lon { $lon }
error-area-not-an-area = ei ole kasutatav kaardiala: { $message }
error-georef-date = kuupäev { $date } jääb World Magnetic Model kehtivusest välja
error-georef-map-range = punkt E { $easting } N { $northing } jääb kaardi koordinaatide vahemikust välja
error-omap-xml = kehtetu OOM XML: { $message }
error-omap-missing-element = elementi <{ $name }> ei leitud
error-ocd-duplicate-symbol = sümboli number { $symbol } on kasutusel kaks korda
error-ocd-unknown-symbol = objekt viitab sümbolile { $symbol }, mida failis pole
error-ocd-wrong-type = sümboli { $symbol } objektil on selle sümboli jaoks vale tüüp

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Kasutus:
cli-heading-commands = Käsud:
cli-heading-options = Valikud:
cli-heading-arguments = Argumendid:
cli-help-possible-values = võimalikud väärtused
cli-help-default = vaikimisi
cli-arg-help = Näita abi
cli-arg-version = Näita versiooni
cli-help-help = Näita käsu abi
cli-arg-help-command = Käsk, mille abi näidata, näiteks run
cli-parse-hint = Lisateabe saamiseks vt '--help'.
cli-parse-unknown-argument = ootamatu argument { $arg }
cli-parse-missing-required = puudub nõutav argument: { $args }
cli-parse-invalid-value = kehtetu väärtus { $value } argumendile { $arg }
cli-parse-invalid-value-choices = kehtetu väärtus { $value } argumendile { $arg }; võimalikud väärtused: { $values }
cli-parse-invalid-subcommand = tundmatu käsk { $value }
cli-parse-missing-subcommand = käsk on nõutav
cli-parse-too-many-values = ootamatu väärtus { $value } argumendile { $arg }
cli-parse-argument-conflict = { $arg } ei saa kasutada koos argumendiga { $other }

help-freemapper = Ehitab geoandmetest OOM-i algprojekte
help-freemapper-lang = Programmi tekstide keel, `de` või `en` (vaikimisi: FREEMAPPER_LANG, siis süsteemikeel, siis inglise keel)
help-init = Loo projektikaust kaardialale
help-init-dir = Loodav projektikaust
help-init-geojson = GeoJSON-fail kaardiala polügooniga (EPSG:4326)
help-init-osm = Kaardiala OSM-i objekt, nt way/992423119
help-init-place = Koha nimi, nt "Clara-Zetkin-Schule Strausberg"; freemapper kasutab parimat vastet, millel on ala
help-init-part = Millist osa mitmest eraldi osast koosnevast OSM-i alast kasutada (1 on suurim)
help-init-kind = Kaarditüüp
help-arg-scale = Uue projekti mõõtkava nimetaja: schoolyard 1000 (vaikimisi); sprint 4000 (vaikimisi) või 5000; forest 15000 (vaikimisi), 10000 või 7500. Muud väärtused lükatakse tagasi, sest freemapper-print lükkab need tagasi
help-init-title = Kaardi pealkiri äärteabe jaoks (vaikimisi: kausta nimi)
help-export = Kirjuta algprojekt (OOM-i kaart, mallid, allikaviited) projektikausta
help-export-dir = Projektikaust
help-arg-force = Asenda olemasolev kaardifail ja jäta vana alles failina `<name>.omap.bak`
help-export-ocad = Kirjuta ka OCAD-i lähtepunkt kausta `ocad/` (OCAD 12 vorming); olemasolev OOM-i kaart jääb siis alles, kui ei anta --force
help-export-pdf = Kirjuta ka `<name>.pdf`, prinditav kaart (ainult kaardiobjektid, kandidaadita); olemasolev OOM-i kaart jääb siis alles, kui ei anta --force
help-export-pdf-candidates = Kirjuta ka `<name>-candidates.pdf`, kontrollprint: kaart kõigi kandidaatidega, asukohatruult; olemasolev OOM-i kaart jääb siis alles, kui ei anta --force
help-export-training = Kirjuta ka treeningkaart `<name>-training.omap` ja selle PDF `<name>-training.pdf`: kaart ja kõik kandidaadid, puhastatud nii, et seda saab printida just sellisena; koos --ocad ka `ocad/<name>-training.ocd`. Olemasolev OOM-i kaart jääb siis alles, kui ei anta --force, ja --force asendab ka olemasoleva treeningkaardi
help-export-view = Koos --training: kirjuta ka treeningkaart ilma sellega, mida vaade välja jätab, kujul `<name>-training-<view>.omap` ja selle PDF (koos --ocad ka OCAD-i fail). Vaated: `no-paths` (ei teid, radu ega treppe), `brown-only` (ainult pruunid sümbolid ja raam), `no-north-lines`, või loend `no:<codes>` / `only:<codes>`; ühenda märgiga `+`. Mitme faili jaoks korda valikut
help-export-pdf-scale-bar = Joonista mõõtkavariba PDF-idele (vaikimisi välja lülitatud, Saksa kaartidel seda tavaliselt pole; mõõtkava number jääb pealkirjaplokki)
help-export-pdf-sources = Allikaread PDF-idel: `auto` prindib täieliku loendi ja lühendab seda ainult siis, kui see võtaks üle 5 rea, `full` ei lühenda kunagi, `credit-only` prindib ainult freemapperi allikaviite (ODbL ja dl-de/by-2-0 nõuavad siis allikate nimetamist kaardi edastamisel). ATTRIBUTION.txt loetleb alati kõik allikad
help-fetch = Laadi kaardiala andmed alla (ortofoto, maastikumudel, nDSM, ALKIS-i hooned, OSM)
help-fetch-dir = Projektikaust
help-fetch-offline = Kasuta ainult vahemälu, ilma võrguühenduseta
help-arg-cache-dir = Allalaaditud ruutude vahemälukaust (vaikimisi: süsteemi vahemälukaust + `freemapper`)
help-derive = Arvuta mallid (ortofoto, reljeefivarjutus, kallak, kohalik reljeef, nDSM), kõrgusjooned ja pinnavormid alla laaditud andmetest
help-derive-dir = Projektikaust
help-run = Laadi alla, tuleta ja ekspordi ühe korraga
help-run-dir = Projektikaust
help-run-offline = Kasuta ainult vahemälu, ilma võrguühenduseta
help-run-ocad = Kirjuta ka OCAD-i lähtepunkt kausta `ocad/` (OCAD 12 vorming)
help-run-pdf = Kirjuta ka `<name>.pdf`, prinditav kaart (ainult kaardiobjektid, kandidaadita)
help-run-pdf-candidates = Kirjuta ka `<name>-candidates.pdf`, kontrollprint: kaart kõigi kandidaatidega, asukohatruult
help-run-training = Kirjuta ka treeningkaart `<name>-training.omap` ja selle PDF `<name>-training.pdf`: kaart ja kõik kandidaadid, puhastatud nii, et seda saab printida just sellisena; koos --ocad ka `ocad/<name>-training.ocd`. Olemasolev treeningkaart jääb alles, kui ei anta --force
help-run-view = Koos --training: kirjuta ka treeningkaart ilma sellega, mida vaade välja jätab, kujul `<name>-training-<view>.omap` ja selle PDF (koos --ocad ka OCAD-i fail). Vaated: `no-paths` (ei teid, radu ega treppe), `brown-only` (ainult pruunid sümbolid ja raam), `no-north-lines`, või loend `no:<codes>` / `only:<codes>`; ühenda märgiga `+`. Olemasolev vaatefail jääb alles, kui ei anta --force
help-run-pdf-scale-bar = Joonista mõõtkavariba PDF-idele (vaikimisi välja lülitatud; mõõtkava number jääb pealkirjaplokki)
help-run-pdf-sources = Allikaread PDF-idel: `auto` prindib täieliku loendi ja lühendab seda ainult siis, kui see võtaks üle 5 rea, `full` ei lühenda kunagi, `credit-only` prindib ainult freemapperi allikaviite (ODbL ja dl-de/by-2-0 nõuavad siis allikate nimetamist kaardi edastamisel). ATTRIBUTION.txt loetleb alati kõik allikad
help-select = Vali kaardiala brauseris, joonistades selle või valides koha. Salvestamine kirjutab freemapper.toml ja loob vajaduse korral projektikausta
help-select-dir = Projektikaust (luuakse salvestamisel, kui seda pole)
help-select-kind = Uue projekti kaarditüüp
help-select-title = Uue projekti kaardi pealkiri
help-select-no-browser = Trüki ainult lehe aadress; ära ava brauserit
help-select-port = Port aadressil 127.0.0.1 (vaikimisi: vaba port)
help-select-offline = Leht algab märgitud valikuga „Ainult vahemälu, ilma võrguta“
help-select-ocad = Leht algab märgitud OCAD-i algprojektiga
help-select-pdf = Leht algab märgitud prinditava PDF-iga
help-select-pdf-candidates = Leht algab märgitud kontrollprindiga (iga kandidaat)
help-select-training = Leht algab märgitud treeningkaardiga
help-select-pdf-scale-bar = Leht algab märgitud mõõtkavaribaga PDF-idel
help-select-pdf-sources = Leht algab selle allikaridade valikuga PDF-idel, nagu käsul `run`
help-find = Loetle nime järgi OpenStreetMapi kohad, alad eespool, koos id-ga käsule init --osm
help-find-query = Koha nimi, nt "Clara-Zetkin-Schule Strausberg"
help-find-json = Trüki vasted JSON-ina, samad väljad kui MCP-tööriistal find_place
help-mcp = Paku tööriistu tehisintellekti agendile MCP kaudu (stdin/stdout)
help-update = Installi uuem freemapper GitHubi väljalasetest, pärast küsimist. Lähtekoodist ehitatud versioon ütleb selle asemel, kuidas seda värskendada
help-update-check = Ainult kontrolli: väljumiskood 10, kui uuem versioon on olemas, 0, kui see on ajakohane
help-update-yes = Installi küsimata; vajalik, kui sisend ei ole terminal
help-update-version = Installi see versioon (x.y.z) uusima asemel, ka vanem
help-update-rollback = Mine tagasi versioonile, mille viimane värskendus asendas
help-update-mode = Salvesta, mida igapäevane kontroll teeb, kui leiab uuema versiooni (vaikimisi: notify). Töötab igas ehituses, sest kirjutab ainult seade
help-adapters = Näita piirkonnaadaptereid: liidumaad ja riigid, mille jaoks freemapperil on andmeallikad
help-adapters-list = Loetle adapterid koos sellega, kust igaüks pärineb (sisseehitatud, kasutaja kaust, projekt), selle versiooni, viimase kontrolli päeva ja hostidega, mille poole see pöördub
help-adapters-list-json = Trüki loend JSON-ina
help-adapters-list-project = Loe ka selle projektikausta kaustas `adapters` olevaid adaptereid, nagu teeks --adapters-from-project
help-adapters-check = Kontrolli adapteri kausta enne selle kasutamist või edasiandmist: kas see laaditaks, mida selle autor peaks teadma, ja koos --live, kas selle teenused vastavad. Väljumiskood 0: läbib; 1: on tähelepanekuid; 2: kontrolli ei saanud käivitada
help-adapters-check-dir = Adapteri kaust; selle nimi on adapteri id
help-adapters-check-live = Küsi adapteri teenustelt ka kasti, mille --bbox nimetab, ja ütle, mida igaüks annab. Päringud jäävad hostidele, mille manifest deklareerib
help-adapters-check-bbox = Reaalajakontrolli kast kraadides kujul minLon,minLat,maxLon,maxLat. See asub adapteri piiri sees; 150 m korda 150 m on piisav
help-adapters-check-json = Trüki tulemus JSON-ina
help-adapters-allow-host = Luba sisseehitatud adapterile üks host juurde: liidumaa jaoks, mille teenus on kolinud serverisse, mida selle adapter veel ei nimeta. Teade keelatud aadressi kohta ütleb, milline host. Hosti kontrollitakse nagu manifesti hosti ja freemapper ühendub endiselt ainult internetis olevate serveritega
help-adapters-allow-host-adapter = Sisseehitatud adapteri id, nagu `adapters list` seda näitab
help-adapters-allow-host-host = Lubatav host: DNS-nimi, mitte aadress, mitte port
help-adapters-disallow-host = Võta tagasi host, mille allow-host lubas
help-adapters-disallow-host-adapter = Sisseehitatud adapteri id
help-adapters-disallow-host-host = Tagasi võetav host
help-arg-adapters-from-project = Laadi ka projektikausta kaustas `adapters` olevad adapterid. freemapper nimetab igaühe koos hostidega, mille poole see pöördub. Kasuta seda usaldatud projekti jaoks: adapter paneb freemapperi küsima servereid, mida see nimetab

## The summary of a run, for the MCP server and the area page

cli-internal-error = sisemine viga: { $message }
cli-pdf-sources-invalid = pdf_sources peab olema auto, full või credit-only, mitte { $value }
cli-view-unknown = vaade on no-paths, brown-only, no-north-lines, no:<codes> või only:<codes>, mitte { $value }
cli-view-empty = loend { $value } ei nimeta ühtegi sümbolikoodi
cli-view-bad-code = { $value } ei ole sümbolikood, nagu 505 või 501.6
cli-summary-training-view = Kirjutatud { $map } ja { $pdf } (treeningkaardi vaade)
cli-summary-kept-training-view = Treeningkaardi vaate { $view } failid jäid alles; uuendamiseks tuleb anda --force
cli-summary-training-view-failed = Treeningkaardi vaade ebaõnnestus: { $error }
cli-estimate-disk-unreadable = asukoha { $path } vaba kettaruumi ei saa lugeda ({ $message })
cli-summary-wrote-map = Kirjutatud kaart { $path }
cli-summary-kept-map = Olemasolev kaart { $path } jäi alles (see võib sisaldada kaardistaja tööd)
cli-summary-map-lacks-templates = Kaart ei viita veel pildimallidele. Selle asendamiseks tuleb käivitada töövoog valikuga force; vana jääb alles failina .omap.bak.
cli-summary-map-lacks-buildings = Kaardil pole veel ALKIS-i hooneid. Nende lisamiseks tuleb käivitada töövoog valikuga force; see asendab kaardi ja jätab vana alles failina .omap.bak.
cli-summary-map-lacks-canopies = Kaardil puuduvad ALKIS-i katused ja autokatused (522), mis on nüüd kaardiobjektid. Nende lisamiseks tuleb käivitada töövoog valikuga force; see asendab kaardi ja jätab vana alles failina .omap.bak.
cli-summary-map-light = Aluskaart: { $joined } joont liidetud, { $left } peidetud või liiga väikest jäeti kandidaatkihtidesse (midagi ei nihutatud)
cli-summary-map-light-kept = Kandidaatkihid sisaldavad nüüd ka { $left } objekti või lõiku, mille see versioon aluskaardilt eemaldab (märgitud freemapper:generalized). Säilitatud kaardil on need alles, seega paistavad need kaks korda. Kaardi puhastamiseks tuleb käivitada töövoog valikuga force; see jätab vana alles failina .omap.bak.
cli-summary-map-old-candidates = Kaart nimetab endiselt faili templates/candidates.omap; see käivitus täitis selle kõigi kandidaatidega. Kandidaadid tulevad nüüd ka seitsmes kihis, igaüks eraldi failis (templates/candidates-*.omap). Kaart loetleb need pärast käivitust valikuga force (vana kaart jääb failina .omap.bak) või kaardistaja lisab need OOM-is (Mallid > Ava mall …).
cli-summary-wrote-ocad = Kirjutatud OCAD-i kaart { $path } ({ $count } objekti)
cli-summary-kept-ocad = Olemasolev OCAD-i kaart { $path } jäi alles (see võib sisaldada kaardistaja tööd; force asendab selle)
cli-summary-ocad-old-candidates = OCAD-i kaart nimetab endiselt faili { $path } taustakaardina; see käivitus täitis selle kõigi kandidaatidega. Kandidaadid tulevad nüüd ka seitsmes kihis, igaüks eraldi failis (-candidates-*.ocd). OCAD-i kaart loetleb need pärast käivitust valikuga force (vana kaart jääb failina .ocd.bak) või kaardistaja lisab need OCAD-is taustakaartidena.
cli-summary-ocad-left-out = OCAD-i kaardilt jäid välja: { $skipped } objekti ja { $dropped } degenereerunud osa
cli-summary-ocad-symbols = OCAD-is tuleb valida Symbol > Load Colors and Symbols From… ja valida ametlik sümbolikomplekt{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF-i koolikomplekt 2019
       *[other] {" "}ISSprOM 2019
    }; see asendab kohatäitesümbolid numbri järgi.
cli-summary-ocad-failed = OCAD-i eksport ebaõnnestus: { $error }
cli-summary-pdf-without-credits = PDF ilma allikaviideteta: ODbL ja dl-de/by-2-0 nõuavad allikate nimetamist kaardi edastamisel; vt ATTRIBUTION.txt
cli-summary-wrote-pdf = Kirjutatud kaart { $path } (ilma kandidaatideta)
cli-summary-pdf-failed = PDF-i eksport ebaõnnestus: { $error }
cli-summary-wrote-pdf-candidates = Kirjutatud kontrollprint { $path } (kaart kõigi kandidaatidega, asukohatruult)
cli-summary-pdf-candidates-failed = Kontrollprindi eksport ebaõnnestus: { $error }
cli-summary-training-failed = Treeningkaardi eksport ebaõnnestus: { $error }
cli-summary-kept-training = Olemasolev treeningkaart { $path } jäi alles, et käivitus ei kirjutaks üle välja prinditud või edasi antud kaarti; selle uuendamiseks tuleb see kustutada või määrata force

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = sellel süsteemil pole kodukausta; tuleb kasutada absoluutset teed
mcp-error-relative-path = { $path } on suhteline tee; tuleb kasutada absoluutset teed või ~-ga algavat teed
mcp-error-home-folder = { $path } on kodukaust ise; tuleb kasutada selle sees olevat kausta, nt ~/maps/school
mcp-error-no-project = kaustas { $path } pole freemapperi projekti; esmalt tuleb kasutada create_project
mcp-error-unknown-kind = tundmatu kaarditüüp { $kind }; tuleb kasutada "schoolyard", "sprint" või "forest"
mcp-error-reading = { $path } lugemine: { $error }
mcp-error-one-area-source = tuleb anda täpselt üks: osm_id (käsust find_place) või geojson
mcp-error-page-open = selle projekti alavaliku leht on veel avatud; tuleb paluda kasutajal ala salvestada ja lehel vajutada „Valmis“, seejärel käivitada töövoog
mcp-error-no-job = tööd { $job } pole; start_pipeline ja permits_for_area tagastavad selle seansi töö-id-d
mcp-error-job-running = { $job } töötab veel kaustale { $path }; korraga töötab üks töövoog, sest projektid jagavad allalaadimise vahemälu. Selle saab käivitada, kui project_status teatab finished või failed
mcp-error-page-start = lehte ei saa käivitada: { $error }
mcp-created-project = Projekt { $path } loodud alale { $area } ({ $kind }, 1:{ $scale }). Järgmiseks: estimate_download, seejärel start_pipeline.
mcp-created-project-given-area = Projekt { $path } loodud antud alale ({ $kind }, 1:{ $scale }). Järgmiseks: estimate_download, seejärel start_pipeline.
mcp-page-already-open = Selle kausta leht on juba avatud; see säilitab seaded, millega see avati.
mcp-page-next = Tuleb paluda kasutajal valida lehel ala, vajutada Salvesta ja seejärel Valmis.
mcp-preview-of = Kihi { $layer } eelvaade
mcp-permits-next = Loamärkmeid tehakse. Tuleb küsida project_status selle job_id-ga; kui töö on lõppenud, sisaldab permits_md faili permits.md teksti. See tuleb edastada vihjena, mitte õigusnõuna.
mcp-tool-check-update = Kas uuem freemapperi väljalase on olemas: selle versioon, väljalaskekuupäev, väljalaskemärkmed ja allalaadimise suurus või teade, et see on ajakohane. Ei installi midagi.
mcp-tool-create-project = Loo projektikaust kaardialale. Anna osm_id (käsust find_place) või GeoJSON-fail. kind on schoolyard (1:1000, vaikimisi), sprint (1:4000) või forest (1:15000).
mcp-param-create-project-dir = Uus projektikaust (absoluutne tee või ~-ga algav).
mcp-param-create-project-geojson = osm_id asemel: GeoJSON-fail kaardiala polügooniga (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (vaikimisi, 1:1000), "sprint" (1:4000) või "forest" (1:15000).
mcp-param-create-project-osm-id = Kaardiala OSM-i objekt käsust find_place, nt "way/992423119".
mcp-param-create-project-part = Koos osm_id-ga: millist osa mitmest eraldi osast koosnevast alast kasutada (1 on suurim).
mcp-param-create-project-scale = Mõõtkava nimetaja: schoolyard 1000 (vaikimisi); sprint 4000 (vaikimisi) või 5000; forest 15000 (vaikimisi), 10000 või 7500. Muud väärtused lükatakse tagasi; 1:3000 jaoks tuleb joonistada mõõtkavas 4000, freemapper-print suurendab printimisel.
mcp-param-create-project-title = Kaardi pealkiri äärteabe jaoks (vaikimisi: kausta nimi).
mcp-tool-estimate-download = Mida start_pipeline projekti jaoks alla laadib: ruudud, iga allalaadimine oma suurusega MB-des, kas see on vahemälus ja kas see mahub kettale, veel allalaaditavad MB-d ning väljal disk vahemälu ja projekti köidete vaba ruum koos teadetega selle kohta, mis ei mahu (tuleb kasutajale öelda).
mcp-param-estimate-download-dir = Projektikaust (absoluutne tee või ~-ga algav).
mcp-tool-find-place = Otsi kooli, parki või muud kohta nime järgi (OpenStreetMap Nominatim). Tagastab kuni 5 vastet, alad eespool; kaardialaks sobivad ainult vasted, millel has_area. Tühi loend tähendab, et midagi ei leitud.
mcp-param-find-place-query = Kooli või koha nimi, nt "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installi uusim freemapperi väljalase selle programmi peale. Kutsu seda ainult pärast inimese nõusolekut, väärtusega confirm true. Uus versioon rakendub, kui assistent freemapperi serveri taaskäivitab.
mcp-param-install-update-confirm = true, kui inimene nõustus uuema versiooni installimisega; ilma selleta ei installita midagi.
mcp-tool-list-layers = Projekti kihid: kaart, seitse kandidaatkihti ja rastermallid, igaüks oma faili, olekuga (kas see on juba tehtud) ja sammuga, mis selle teeb.
mcp-param-list-layers-dir = Projektikaust (absoluutne tee või ~-ga algav).
mcp-tool-open-area-selector = Ava brauseris leht, kus kasutaja joonistab kaardiala (ristkülik või hulknurk) või valib koha, näeb ruute ja allalaadimise suurust ning salvestab selle; salvestamine loob projektikausta või muudab selle ala. Tagastab URL-i, mis kasutajale anda. Leht töötab, kuni kasutaja vajutab Valmis; sama kausta jaoks uuesti kutsumine tagastab sama URL-i.
mcp-param-open-area-selector-dir = Projektikaust (absoluutne tee või ~-ga algav); luuakse, kui kasutaja salvestab.
mcp-param-open-area-selector-kind = Uue projekti kaarditüüp: "schoolyard" (vaikimisi, 1:1000), "sprint" (1:4000) või "forest" (1:15000).
mcp-param-open-area-selector-ocad = Käivituspaneel algab märgitud OCAD-i algprojektiga. Vaikimisi false.
mcp-param-open-area-selector-offline = Käivituspaneel algab märgitud valikuga „ainult vahemälu, ilma võrguta“. Vaikimisi false.
mcp-param-open-area-selector-open-browser = Ava leht ka selle arvuti vaikebrauseris. Vaikimisi true.
mcp-param-open-area-selector-pdf = Käivituspaneel algab märgitud prinditava PDF-iga. Vaikimisi false.
mcp-param-open-area-selector-pdf-candidates = Käivituspaneel algab märgitud kontrollprindiga (iga kandidaat). Vaikimisi false.
mcp-param-open-area-selector-pdf-scale-bar = Käivituspaneel algab märgitud mõõtkavaribaga PDF-idel. Vaikimisi false.
mcp-param-open-area-selector-pdf-sources = Allikaread PDF-idel, millega paneel algab: "auto" (vaikimisi), "full" või "credit-only", nagu start_pipeline puhul.
mcp-param-open-area-selector-permits = Käivituspaneel algab märgitud loamärkmetega. Vaikimisi false.
mcp-param-open-area-selector-scale = Uue projekti mõõtkava nimetaja: schoolyard 1000 (vaikimisi); sprint 4000 (vaikimisi) või 5000; forest 15000 (vaikimisi), 10000 või 7500. Muud väärtused lükatakse tagasi.
mcp-param-open-area-selector-title = Uue projekti kaardi pealkiri.
mcp-param-open-area-selector-training = Käivituspaneel algab märgitud treeningkaardiga. Vaikimisi false.
mcp-tool-preview-layer = Kihi PNG-eelvaade (kuni 800 px): map (viimase käivituse kirjutatud kaardiobjektide pilt, ilma kandidaatideta) või rastermall: orthophoto, hillshade, slope, relief, ndsm või k5 Berliinis.
mcp-param-preview-layer-dir = Projektikaust (absoluutne tee või ~-ga algav).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" või "k5" (ainult Berliin).
mcp-tool-project-status = Tööoleku start_pipeline või permits_for_area töö kohta: running, finished või failed, viimased edenemisread, hoiatused, kirjutatud failid ning tulemus või viga. Lõppenud permits_for_area töö kannab ka faili permits.md teksti väljal permits_md.
mcp-param-project-status-job-id = Töö-id käsust start_pipeline või permits_for_area, nt "job-1".
mcp-tool-permits-for-area = Tee projekti loamärkmed: kaardiala haldusüksused, selles asuvad kaitsealad ja kes seal maad haldab, koos ametiasutustega, kellelt küsida; vihje, mitte õigusnõu. Kirjutab kausta permits.md ja permits.json. Töötab taustal ja tagastab job_id; tuleb küsida project_status, mille vastus kannab töö lõppedes faili permits.md teksti väljal permits_md. Korraga töötab üks töö.
mcp-param-permits-for-area-dir = Projektikaust (absoluutne tee või ~-ga algav).
mcp-param-permits-for-area-offline = Loe ainult andmeid, mille varasem käivitus projekti salvestas, ilma võrguühenduseta. Vaikimisi false.
mcp-tool-start-pipeline = Laadi andmed alla, arvuta mallid ja kirjuta algprojekt (fetch, derive, export). Töötab taustal ja tagastab job_id; tuleb küsida project_status. Korraga töötab üks töövoog. Olemasolev kaart jääb alles, kui force pole true.
mcp-param-start-pipeline-dir = Projektikaust (absoluutne tee või ~-ga algav).
mcp-param-start-pipeline-force = Asenda olemasolev kaardifail ja jäta vana alles failina .omap.bak. Vaikimisi false.
mcp-param-start-pipeline-ocad = Kirjuta ka OCAD-i lähtepunkt kausta ocad/ (OCAD 12 vorming, OCAD-is töötavatele kasutajatele). Vaikimisi false.
mcp-param-start-pipeline-offline = Kasuta ainult allalaadimise vahemälu, ilma võrguühenduseta. Vaikimisi false.
mcp-param-start-pipeline-pdf = Kirjuta ka <name>.pdf, printimiseks valmis kaart (ainult kaardiobjektid, kandidaadita). Vaikimisi false.
mcp-param-start-pipeline-pdf-candidates = Kirjuta ka <name>-candidates.pdf, kontrollprint: kaart kõigi kandidaatidega, asukohatruult (neid tuleb maastikul kontrollida). Vaikimisi false.
mcp-param-start-pipeline-pdf-scale-bar = Joonista mõõtkavariba PDF-idele. Vaikimisi false (Saksa kaartidel seda tavaliselt pole; mõõtkava number jääb pealkirjaplokki).
mcp-param-start-pipeline-pdf-sources = Allikaread PDF-idel: "auto" (vaikimisi; täielik loend, lühendatud ainult siis, kui see võtab üle 5 rea), "full" (ei lühendata kunagi) või "credit-only" (ainult freemapperi allikaviide; ODbL ja dl-de/by-2-0 nõuavad siis allikate nimetamist kaardi edastamisel). ATTRIBUTION.txt loetleb alati kõik allikad.
mcp-param-start-pipeline-permits = Kirjuta pärast kaarti ka loamärkmed (permits.md ja permits.json): haldusüksused, kaitsealad ja kes maad haldab. Rike seal on hoiatus, mitte ebaõnnestunud töö. Vaikimisi false.
mcp-param-start-pipeline-training = Kirjuta ka treeningkaart <name>-training.omap ja selle PDF <name>-training.pdf: kaart ja kõik kandidaadid, puhastatud nii, et seda saab printida just sellisena; koos ocad ka ocad/<name>-training.ocd. Olemasolev treeningkaart jääb alles, kui force pole true. Vaikimisi false.
mcp-param-start-pipeline-training-views = Koos training: kirjuta ka treeningkaart ilma sellega, mida vaade välja jätab, üks failikomplekt kirje kohta: "no-paths", "brown-only", "no-north-lines" või loend nagu "no:505,506" või "only:101,102", ühendatud märgiga +. Failid on <name>-training-<view>.omap ja selle PDF.
mcp-instructions = freemapper teeb OpenOrienteering Mapperi algprojekte orienteerumiskaartidele (koolihoov 1:1000, sprint 1:4000, mets 1:15000) piirkondade { $states } ametlikest geoandmetest ja OSM-ist. Töövoog: find_place (otsi kooli või ala nime järgi; vali tulemus, millel has_area) → create_project (uus kaust, absoluutne tee, osm_id-ga) → estimate_download (tuleb kasutajale öelda, kui palju alla laaditakse) → start_pipeline (tagastab töö-id; allalaadimine võib võtta minuteid) → project_status (küsi iga mõne sekundi järel, kuni olek on finished või failed) → list_layers ja preview_layer (näita kasutajale malle). Seejärel avab kasutaja kaardifaili OpenOrienteering Mapperis. Kui ükski koht ei sobi või kasutaja soovib ala joonistada, avab open_area_selector brauseris lehe, kus kasutaja joonistab või valib ala ja salvestab selle (see loob projekti); pärast kasutaja teadet, et see on salvestatud, tuleb jätkata käsuga estimate_download. Teed peavad olema absoluutsed või algama ~-ga. Korraga töötab üks töövoog. permits_for_area teeb projekti loamärkmed (haldusüksused, kaitsealad, kes maad haldab; vihje, mitte õigusnõu) tööna nagu start_pipeline. check_update ütleb, kas uuem freemapper on olemas; install_update installib selle, ainult väärtusega confirm: true pärast inimese nõusolekut, ja assistent taaskäivitab seejärel freemapperi serveri.

cli-contour-interval-saved = Kõrgusjoonte vahe salvestatud faili freemapper.toml: { $interval } m; derive joonistab kõrgusjooned selle järgi
cli-contour-interval-auto = Kõrgusjoonte vahe failis freemapper.toml: automaatne (kaarditüübi oma; metsakaardil 2,5 m tasasel maal ja 10 m järsul maal)
help-arg-contour-interval = Kõrgusjoonte vahe meetrites, nt `2.5`, või `auto` kaarditüübi vaikimisi jaoks (metsakaart: 5 m, 2,5 m tasasel maal, 10 m järsul maal). Salvestatud faili freemapper.toml; derive joonistab kõrgusjooned selle järgi
cli-content-saved = Sisuseade salvestatud faili freemapper.toml: { $summary }
cli-content-invalid = Sisuseadet ei saa lugeda: { $detail }
help-arg-content = Mis läheb otse kaardile: `safe` jätab iga objekti sinna, kuhu koostamine selle paneb, `balanced` (vaikimisi) paneb katastri ja registrite objektid kaardile, `full` paneb kõik kaardile. Valik salvestatakse faili freemapper.toml
help-arg-content-set = Saada objektirühm kohta `map`, `candidates` või `off`, kujul `GROUP=PLACE`, nt `walls=map`; võib korrata. Rühmad: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Salvestatud faili freemapper.toml
mcp-param-create-project-content = Mis läheb otse kaardile: "safe", "balanced" (vaikimisi) või "full". Salvestatud projekti. balanced paneb katastri ja registrite objektid kaardile, full paneb kõik kaardile.
mcp-param-create-project-content-set = Rühmad, mis saadetakse mujale, kui profiil ütleb, kujul "group=place", kus place on map, candidates või off, nt "walls=map". Rühmad: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Kõrgusjoonte vahe meetrites, nt 2.5; 0 läheb tagasi automaatse valiku juurde (kaarditüübi oma; metsakaardil 2,5 m tasasel maal ja 10 m järsul maal). Salvestatud projekti; järgmine derive joonistab kõrgusjooned selle järgi.
mcp-param-start-pipeline-content = Muuda enne käivitust, mis läheb otse kaardile: "safe", "balanced" või "full". Salvestatud projekti. Vaikimisi: projekti seade, muidu balanced.
mcp-param-start-pipeline-content-set = Rühmad, mis saadetakse mujale, kui profiil ütleb, kujul "group=place", kus place on map, candidates või off, nt "fences=candidates". Salvestatud projekti.
mcp-param-start-pipeline-contour-interval = Kõrgusjoonte vahe meetrites, nt 2.5; 0 läheb tagasi automaatse valiku juurde (kaarditüübi oma; metsakaardil 2,5 m tasasel maal ja 10 m järsul maal). Salvestatud projekti; järgmine derive joonistab kõrgusjooned selle järgi.
mcp-error-content = Sisuseadet ei saa lugeda: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Kirjuta projekti loamärkmed (permits.md ja permits.json): haldusüksused, kaitsealad ja haldajad, mida kaardiala puudutab, ning see, kellelt see tavaliselt luba küsida tähendab. See küsib BfN-ilt ja BKG-lt ning kui kaardiala liidumaal on oma avatud teenused, siis neid selle adapteri hostide kaudu (loend käsiraamatu 12. peatükis); Nordrhein-Westfalenis laaditakse selleks esimesel korral alla umbes 32 MB, Brandenburgis umbes 4 MB, hiljem ainult uuemad failid. See loeb juba vahemällu salvestatud OSM-i väljavõtet; --offline kasutab varem alla laaditut
help-permits-dir = Projektikaust
help-permits-offline = Kasuta ainult varem alla laaditud andmeid, ilma võrguühenduseta
help-run-permits = Kirjuta pärast kaarti ka loamärkmed (permits.md ja permits.json); küsib BfN-ilt, BKG-lt ja liidumaa avatud teenustelt (käsiraamat, 12. peatükk). Rike seal on hoiatus ja kaart jääb alles
cli-wrote-permits = Kirjutatud loamärkmed { $path }
cli-summary-wrote-permits = Kirjutatud loamärkmed { $path }
cli-permits-failed = loamärkmeid ei saanud kirjutada: { $error }
cli-permits-source-left-out = loamärkmed jätavad välja allika { $source }: { $reason }
cli-permits-not-read = seda ei loetud
error-permits-stored = { $path }: { $message }; loamärkmed tuleb uuesti võrgust alla laadida
error-permits-nothing-read = ühtegi loamärkmete allikat ei saanud lugeda ({ $missing }); need tuleb esmalt võrgust alla laadida
error-permits-missing-not-read = { $source }: ei loetud

permits-title = Loamärkmed
permits-disclaimer = Vihje, mitte õigusnõu. Otsustavad pädevad ametiasutused. Andmete kuupäeva ja täpsuse kohta vt tabelit jaotise „Allikad ja andmete kuupäev“ all; alasid võib puududa või need võivad olla aegunud (BfN: ühendatud kord aastas, „Nicht für Planungszwecke geeignet.“ ehk planeerimiseks sobimatu).
permits-bfn-merge = BfN ühendab liidumaade andmed kord aastas ja märgib need planeerimiseks sobimatuks. Otsustavad liidumaa ametiasutused.
permits-created = Tehtud { $date } { $kind ->
        [schoolyard] koolihoovikaardi
        [forest] metsakaardi
       *[other] sprindikaardi
    } kaardiala jaoks ({ $area } ha).
permits-section-admin = Haldusüksused
permits-section-protected = Kaitsealad (BfN)
permits-section-osm-protected = Veel kaitsealasid OpenStreetMapist
permits-section-operators = Haldajad ja omanikud OpenStreetMapi järgi
permits-section-hints = Vihjed
permits-section-sources = Allikad ja andmete kuupäev
permits-none-admin = VG250 ei nimeta kaardialale ühtegi haldusüksust.
permits-none-protected = BfN-i järgi ei asu kaardialal ühtegi kaitseala.
permits-none-osm-protected = OpenStreetMap ei nimeta kaardialal ühtegi kaitseala, mida BfN juba ei loetle.
permits-none-operators = OpenStreetMap ei nimeta kaardialale ühtegi haldajat ega omanikku.
permits-none-hints = Vihjeid pole.
permits-section-not-read = Allika { $source } andmeid ei loetud, seetõttu on see jaotis tühi, kuigi kirjeid võib olla.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] BKG haldusalad 1:250 000 (VG250)
        [bfn] BfN-i kaitsealad Saksamaal
       *[osm] OpenStreetMap, väljavõttest, mida freemapper hoiab oma vahemälus
    }
permits-level = { $level ->
        [land] Liidumaa
        [kreis] Ringkond (Kreis)
        [verwaltungsgemeinschaft] Amt või Verwaltungsgemeinschaft
       *[gemeinde] Omavalitsus (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Puudutatud servas (alla 0,5 % kaardialast):
permits-at-edge-generalised = VG250 on üldistatud mõõtkavasse 1:250 000. Üksus, mis puudutab ainult serva, võib olla selle tagajärg.
permits-kind = { $kind ->
        [nationalpark] Rahvuspark
        [nationales-naturmonument] Riiklik loodusmälestis
        [naturschutzgebiet] Looduskaitseala (Naturschutzgebiet)
        [ffh] FFH-ala (Natura 2000)
        [vogelschutzgebiet] Linnukaitseala (Natura 2000)
        [biosphaerenreservat] Biosfäärikaitseala
        [biosphaerenreservat-zone] Biosfäärikaitseala tsoon
        [landschaftsschutzgebiet] Maastikukaitseala (Landschaftsschutzgebiet)
       *[naturpark] Looduspark
    }
permits-protected-line = { $kind } „{ $name }“, { $share }
permits-protected-date = Kuulutatud { $date }
permits-protected-title = Õiguslik alus: { $title }
permits-protected-link = Õigustekst: { $url }
permits-protected-site-link = Ala leht (Natura 2000): { $url }
permits-protected-year = Andmeaasta BfN-is: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = kaitseklass { $class }
permits-quoted = „{ $text }“
permits-osm-unnamed = nimeta
permits-osm-operator = Haldaja OSM-i järgi: { $operator }
permits-osm-enclosing-gap = OpenStreetMapi alad, mis ümbritsevad kogu kaardiala ilma nurgata selle sees, võivad siit puududa, samuti OSM-is relatsioonina `type=boundary` kaardistatud kaitsealad. BfN-i ja VG250 osi see ei puuduta.
permits-operator-line = { $name } ({ $roles ->
        [operator] haldaja
        [owner] omanik
       *[both] haldaja ja omanik
    }): { $objects }
permits-column-source = Allikas
permits-column-state = Olek
permits-column-credit = Allikaviide ja litsents
permits-state-read = loetud, alla laaditud { $date }
permits-state-read-osm = loetud, OSM-i väljavõte kuupäevast { $date }
permits-state-unreachable = kättesaamatu ({ $reason })
permits-state-not-read = ei loetud
permits-no-credit = ei kasutatud
permits-hint-city-state = Kaardiala asub Berliinis või Hamburgis. Seal vastutab tavaliselt Bezirksamt (linnaosa amet) või liidumaa ametiasutus. Need märkmed ei määra veel linnaosa (Bezirk).
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Kaitsealade asjus on pädev alumine looduskaitseasutus (untere Naturschutzbehörde). Siin on selleks { $kreis } linn ise.
        [no] Kaitsealade asjus on pädev alumine looduskaitseasutus (untere Naturschutzbehörde), tavaliselt { $kreis }.
       *[unknown] Kaitsealade asjus on pädev alumine looduskaitseasutus (untere Naturschutzbehörde), tavaliselt Landkreis või kreisfreie Stadt.
    }
permits-hint-national-park = Rahvuspargil on oma reeglid. Rahvuspargi haldusega tuleb ühendust võtta varakult.
permits-hint-nsg-paths = Looduskaitsealal tuleb tavaliselt hoida radadest kinni. Jooksuks väljaspool radu tuleb küsida looduskaitseasutuselt.
permits-hint-natura2000 = FFH-ala või linnukaitseala (Natura 2000) puudutab kaardiala. Ürituse jaoks võib looduskaitseasutus nõuda mõjude hindamist.
permits-hint-order-office = { $amt ->
        [none] Tänavate, väljakute ja muu avaliku maa eest vastutab tavaliselt omavalitsuse { $gemeinde } Ordnungsamt (avaliku korra amet).
       *[other] Tänavate, väljakute ja muu avaliku maa eest vastutab tavaliselt Ordnungsamt (avaliku korra amet): { $amt }, omavalitsuse { $gemeinde } jaoks.
    }
permits-hint-park-operator = Pargi või aia haldaja kaardialal OpenStreetMapi järgi: { $operator }. Enne kasutamist tuleb temalt küsida.
permits-hint-school-operator = Kooli või asutuse haldaja OpenStreetMapi järgi: { $operator }. Selle maa-ala kasutamiseks tuleb küsida kooli juhatajalt või sellelt asutuselt.
permits-section-districts = Bezirke (linnaosad, { $source })
permits-section-green-spaces = Haljasalad ({ $source })
permits-section-owners = Liidumaa ja Bundi maa ({ $source })
permits-section-monuments = Mälestised ({ $source })
permits-section-protected-objects = Loodusmälestised ja muud kaitstud objektid ({ $source })
permits-none-districts = { $source } järgi ei asu kaardiala üheski Bezirkis.
permits-none-green-spaces = { $source } järgi ei asu kaardialal avalikku haljasala.
permits-none-owners = { $source } järgi ei asu kaardialal ühtegi liidumaa ega Bundi kinnistut.
permits-none-monuments = { $source } järgi ei asu kaardialal ühtegi mälestist.
permits-none-protected-objects = { $source } järgi ei asu kaardialal ühtegi selle sisaldatud kaitstud objekti.
permits-none-sampled = Küsitud punktides ei nimeta { $source } siin midagi.
permits-gaps = Allika { $source } andmed ei ole täielikud: { $gaps }.
permits-sampled = { $source } vastab ainult üksikute punktide kohta. freemapper küsis kaardi kohal punktivõrgult: osakaalud on hinnangud ja punktide vahele jääv võib puududa.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = otstarve: { $dedication }
permits-playgrounds-none = Kaardialal pole avalikku mänguväljakut.
permits-playgrounds-one = Kaardialal on üks avalik mänguväljak. Seda ei loetleta eraldi.
permits-playgrounds-many = Kaardialal on { $count } avalikku mänguväljakut. Neid ei loetleta ükshaaval.
permits-owner-other = Muud avalikud omanikud
permits-owners-rest = Ülejäänud ala kohta andmed omanikku ei nimeta.
permits-monuments-total = Kaardialal täielikult või osaliselt asuvad mälestised: { $count }.{ $links ->
        [yes] {" "}Lingid viivad nende kirjeteni mälestiste nimekirjas.
       *[no] {""}
    }
permits-monuments-links-left = Veel { $count } kirjet on siin ainult loetud, ilma lingita.
permits-object-kind = { $kind ->
        [natural-monument] Loodusmälestis (Naturdenkmal)
        [protected-landscape-part] Kaitstud maastikuelement (geschützter Landschaftsbestandteil)
        [biotope] Seadusega kaitstud biotoop (geschütztes Biotop)
       *[special-rules] Erireeglitega ala
    }
permits-object-count = { $count } objekti
permits-object-citations = Õiguslikud alused:
permits-source-note = { $source }: „{ $note }“
permits-hint-district-office = { $offices ->
        [hamburg] Bezirk { $district } tänavate, väljakute ja haljasalade eest vastutab tavaliselt Bezirksamt { $district }, seal Fachamt Management des öffentlichen Raumes (avaliku ruumi haldamise osakond).
       *[berlin] Bezirk { $district } tänavate, väljakute ja haljasalade eest vastutavad tavaliselt Bezirksamt { $district } Ordnungsamt (avaliku korra amet) ja Straßen- und Grünflächenamt (teede ja haljastuse amet).
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Kaardialal asuvate mälestiste eest vastutab Denkmalschutzamt Hamburg (mälestiseamet).
       *[berlin] Bezirk { $district } mälestiste eest vastutab Bezirksamt { $district } alumine mälestisekaitseasutus (untere Denkmalschutzbehörde).
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Kas kaardialal asub kaitse all olevaid mälestisi, seda ei saa siinsetest andmetest kindlalt öelda.
       *[yes] Kaardialal on kaitse all olevaid mälestisi.
    } Kas jooks neid puudutab, näiteks kontrollpunkt mälestise juures või lõik läbi kaitstud paiga, otsustab alumine mälestisekaitseasutus (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Siin on selleks { $kreis } linn ise.
        [no] , tavaliselt { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; mõnes liidumaal on selleks omavalitsus { $gemeinde }.
        }
       *[unknown] {" "}(tavaliselt Landkreis või kreisfreie Stadt, mõnes liidumaal omavalitsus).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Loodusmälestised, kaitstud maastikuelemendid ja kaitstud biotoobid kaardil on alumise looduskaitseasutuse (untere Naturschutzbehörde) asi. Siin on selleks { $kreis } linn ise.
        [no] Loodusmälestised, kaitstud maastikuelemendid ja kaitstud biotoobid kaardil on alumise looduskaitseasutuse (untere Naturschutzbehörde) asi, tavaliselt { $kreis }.
       *[unknown] Loodusmälestised, kaitstud maastikuelemendid ja kaitstud biotoobid kaardil on alumise looduskaitseasutuse (untere Naturschutzbehörde) asi, tavaliselt Landkreis või kreisfreie Stadt.
    }
permits-hint-biotope = Seadusega kaitstud biotoobid asuvad kaardialal{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Kõik, mis võib neid hävitada või oluliselt kahjustada, on keelatud (§ 30 BNatSchG). Kontrollpunktid ja lõigud tuleb paigutada nende väljapoole või küsida alumiselt looduskaitseasutuselt (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Bezirk { $district } loodusmälestiste ja kaitstud maastikuelementide eest vastutab tavaliselt Bezirksamt { $district }.
       *[berlin] Bezirk { $district } loodusmälestiste ja kaitstud maastikuelementide eest vastutab Bezirksamt { $district } alumine looduskaitseasutus (untere Naturschutzbehörde).
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Liidumaa avaandmete järgi kuulub { $share } kaardialast teistele avalikele omanikele.
       *[no] Liidumaa avaandmete järgi kuulub { $share } kaardialast omanikule „{ $owner }“.
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Ülejäänud ala omanikke avaandmetest ei leia. Õigustatud huvi korral (näiteks korraldajana) võib kinnistusraamatu ametilt (Grundbuchamt) küsida väljavõtet; avaliku maa puhul aitab pädev Ordnungsamt või haljastusamet.
       *[no] Nende alade omanikke avaandmetest ei leia. Õigustatud huvi korral (näiteks korraldajana) võib kinnistusraamatu ametilt (Grundbuchamt) küsida väljavõtet; avaliku maa puhul aitab pädev Ordnungsamt või haljastusamet.
    }
permits-section-forest-units = Metsahaldus ({ $source })
permits-section-state-forest = Riigimets ({ $source })
permits-none-forest-units = { $source } järgi ei asu kaardiala ühegi metsaameti tööpiirkonnas.
permits-none-state-forest = { $source } järgi ei asu kaardialal riigimetsa.
permits-forest-units-reach = Metsaamet vastutab oma piirkonna metsa eest. Selle piirkond võib hõlmata ka linnu ja põlde.
permits-state-forest-office = vastutaja: { $office }
permits-hint-forest-office = { $units ->
        [none] Kaardialal asuva metsa eest vastutav metsaamet: { $office } ({ $administration }). Metsas ürituse kavandamisel tuleb sealt varakult küsida.
       *[other] Kaardialal asuva metsa eest vastutav metsaamet: { $office } ({ $administration }), { $units }. Metsas ürituse kavandamisel tuleb sealt varakult küsida.
    }
permits-hint-forest-owner = { $office ->
        [known] Üritus metsas vajab metsaomanike nõusolekut. Metsaamet oskab tavaliselt öelda, kes metsa omab. Mitmes liidumaal tuleb organiseeritud üritusest metsas ka metsaasutust teavitada või saada selle luba.
       *[unknown] Üritus metsas vajab metsaomanike nõusolekut. Mitmes liidumaal tuleb organiseeritud üritusest metsas ka metsaasutust teavitada või saada selle luba. Need märkmed ei nimeta siin vastutavat metsaametit; seda oskab öelda liidumaa metsahaldus või alumine metsaasutus.
    }
permits-section-water-protection = Veekaitsealad ({ $source })
permits-section-forest-functions = Metsafunktsioonid ja kaitsemets ({ $source })
permits-none-water-protection = { $source } järgi ei asu kaardialal veekaitseala.
permits-none-forest-functions = { $source } järgi ei asu kaardialal puhkemetsa, kaitsemetsa, looduslikku metsa ega suletud metsaala.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Puhkemets (Erholungswald)
        [protected] Kaitsemets (Bannwald või Schutzwald)
        [natural] Looduslik mets või range kaitseala (Naturwald, Totalreservat)
       *[closed] Suletud metsaala
    }
permits-hint-water-protection = { $inner ->
        [yes] Veekaitseala „{ $name }“ tsoon ({ $zone }) asub kaardialal. Tsoonides I ja II on üritused tavaliselt välistatud. Tuleb küsida alumiselt veeasutuselt (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] Veekaitseala „{ $name }“ tsoon ({ $zone }) asub kaardialal. Üritused seal on tavaliselt võimalikud; alumine veeasutus (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) oskab öelda, mida määrus lubab.
       *[unknown] Veekaitseala „{ $name }“ asub kaardialal. Andmed ei ütle, milline tsoon see on. Tsoonides I ja II on üritused tavaliselt välistatud; tuleb küsida alumiselt veeasutuselt (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Puhkemets (Erholungswald) asub kaardialal{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Seal ürituste korraldamiseks tuleb küsida metsaasutuselt.
        [protected] Kaitsemets (Bannwald või Schutzwald) asub kaardialal{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Enne ürituse kavandamist seal tuleb küsida metsaasutuselt.
        [natural] Looduslik mets või range kaitseala asub kaardialal{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Radadelt lahkumine on seal sageli keelatud; kontrollpunkte ei tohi sinna panna ilma metsaasutuselt küsimata.
       *[closed] Metsaala, kuhu ei tohi siseneda, asub kaardialal{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. See tuleb rajast välja jätta.
    }
permits-section-roads = Klassifitseeritud teed OpenStreetMapi järgi
permits-none-roads = OpenStreetMapi järgi ei läbi kaardiala ükski Autobahn ega Bundes-, Landes-, Staats- või Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (kiirtee)
        [bundesstrasse] Bundesstraße (liidutee)
        [landesstrasse] Landesstraße või Staatsstraße (liidumaa tee)
       *[kreisstrasse] Kreisstraße (ringkonna tee)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } läbib kaardiala. Rada võib seda ületada ainult sildadel või tunnelites.
        [bundesstrasse] Bundesstraße { $refs } läbib kaardiala. Sulgemise või korraldajatega ületuse korraldab liikluskorraldusasutus (Straßenverkehrsbehörde); teed haldab tavaliselt Bund liidumaa teedeameti kaudu, suuremates linnades sageli linn.
        [landesstrasse] Landes- või Staatsstraße { $refs } läbib kaardiala. Sulgemise või korraldajatega ületuse korraldab liikluskorraldusasutus (Straßenverkehrsbehörde); teed haldab tavaliselt liidumaa, suuremates linnades sageli linn.
       *[kreisstrasse] Kreisstraße { $refs } läbib kaardiala. Sulgemise või korraldajatega ületuse korraldab liikluskorraldusasutus (Straßenverkehrsbehörde); teed haldab tavaliselt Kreis, suuremates linnades sageli linn.
    }
permits-hint-state-forest = { $office ->
        [none] Liidumaa avaandmete järgi on { $share } kaardialast omaniku „{ $owner }“ riigimets.
       *[other] Liidumaa avaandmete järgi on { $share } kaardialast omaniku „{ $owner }“ riigimets. Vastutaja: { $office }.
    }
