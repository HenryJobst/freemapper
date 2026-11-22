# Program texts of freemapper, Latvian (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } nav valoda, kurai freemapper ir teksti; pieejamās: { $available }

## init

cli-part-needs-osm = --part izvēlas OSM apgabala daļu un prasa --osm; ar --place vispirms jāuzmeklē id un jālieto --osm <id> --part <n>
cli-reading-file = tiek lasīts { $path }
cli-no-area-for-place = vietai „{ $query }“ apgabals nav atrasts. Ja vieta ir apgabals no vairākām atsevišķām daļām, izvēlieties vienu ar --osm <id> --part <n> vai lietojiet --geojson
cli-using-place = Tiek izmantots { $name } ({ $id })
cli-created-project = Izveidots projekts { $path }

## Cache

cli-no-cache-dir = šajā sistēmā nav kešatmiņas mapes; norādiet --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [zero] Lejupielādēts { $count } flīžu
        [one] Lejupielādēta { $count } flīze
       *[other] Lejupielādētas { $count } flīzes
    }
cli-wrote = Ierakstīts { $path }
cli-kept-map = Atstāts nemainīts { $path } (tajā var būt jūsu darbs)
cli-map-lacks-templates = Karte vēl neatsaucas uz attēlu veidnēm. Pievienojiet tās OOM (Templates > Open template…) vai aizstājiet karti ar --force; vecā karte paliek kā .omap.bak.
cli-map-lacks-buildings = Kartē vēl nav ALKIS ēku. Lai tās pievienotu, aizstājiet karti ar --force; vecā karte paliek kā .omap.bak.
cli-map-lacks-canopies = Kartei trūkst ALKIS nojumju un automašīnu nojumju (522), kas tagad ir kartes objekti. Lai tās pievienotu, palaidiet ar --force; vecā karte paliek kā .omap.bak.
cli-map-light = Pamatkarte: savienotas līnijas: { $joined }, kandidātu slāņiem atstāti paslēpti vai pārāk mazi objekti: { $left } (nekas netika pārbīdīts)
cli-map-light-kept = Kandidātu slāņos tagad ir arī objekti vai posmi: { $left }, ko šī versija noņem no pamatkartes (atzīmēti ar freemapper:generalized). Atstātajā kartē tie joprojām ir, tāpēc redzami divreiz. Lai karti sakārtotu, aizstājiet to ar --force (vecā karte paliek kā .omap.bak).
cli-map-old-candidates = Karte joprojām norāda uz templates/candidates.omap; šajā palaišanā tas tika aizpildīts ar visiem kandidātiem. Kandidāti tagad ir arī septiņos slāņos, katrs savā failā (templates/candidates-*.omap). Karte tos uzskaita pēc aizstāšanas ar --force (vecā karte paliek kā .omap.bak), vai arī pievienojiet tos paši OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF bez avotu norādēm: ODbL un dl-de/by-2-0 prasa nosaukt avotus, kad karte tiek nodota tālāk; skatiet ATTRIBUTION.txt
cli-wrote-pdf = Ierakstīts { $path } (karte bez kandidātiem)
cli-wrote-pdf-candidates = Ierakstīts { $path } (pārbaudes izdruka: karte ar visiem kandidātiem, precīzi pēc atrašanās vietas; pārbaudiet tos apvidū)
cli-wrote-training = Ierakstīti { $map } un { $pdf } (treniņu karte: savienoti { $joined }, pārbīdīti { $shifted }, izlaisti vai apgriezti { $left })
cli-training-free-ends = Brīvi atstāti līniju gali: kur ceļš saskārās ar citu ceļu { $ways }, pie citām līnijām { $others }
cli-training-apart-left = treniņu kartē joprojām ir { $length } mm (uz papīra), kur objektu atstarpe ir pārāk maza; pārbaudiet to OOM
cli-kept-training = Atstāta treniņu karte { $path }, lai palaišana nepārrakstītu izdrukāto vai nodoto karti; dzēsiet to vai norādiet --force, lai to atjaunotu

## OCAD

cli-ocad-kept = Atstāts nemainīts { $path } (tajā var būt jūsu darbs; --force to aizstāj, vecais paliek kā .ocd.bak)
cli-ocad-old-candidates = OCAD karte joprojām norāda { $path } kā fona karti; šajā palaišanā tā tika aizpildīta ar visiem kandidātiem. Kandidāti tagad ir arī septiņos slāņos, katrs savā failā (-candidates-*.ocd). OCAD karte tos uzskaita pēc aizstāšanas ar --force (vecā paliek kā .ocd.bak), vai arī pievienojiet tos paši OCAD kā fona kartes.
cli-ocad-wrote = { $count ->
        [zero] Ierakstīts { $path } ({ $count } objektu)
        [one] Ierakstīts { $path } ({ $count } objekts)
       *[other] Ierakstīts { $path } ({ $count } objekti)
    }
cli-ocad-left-out = Izlaists no OCAD kartes: objekti { $skipped } (OCAD nav simbola, nav laukuma vai ārpus OCAD zīmēšanas apgabala ±2 m) un deģenerētas līniju daļas vai caurumi { $dropped }.
cli-ocad-load-symbols = OCAD izvēlieties Symbol > Load Colors and Symbols From… un norādiet oficiālo simbolu kopu{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF skolu kopu 2019
       *[other] {" "}ISSprOM 2019
    }; tā aizstāj freemapper vietturu simbolus pēc numura.

## select

cli-select-running = Apgabala izvēle projektam { $path }: nospiediet „Gatavs“ lapā vai Ctrl+C, lai apturētu.
cli-select-no-browser = Pārlūku neizdevās atvērt ({ $error }); atveriet iepriekš norādīto adresi.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; nav apgabala, nevar būt kartes apgabals): { $place }
cli-find-hint = Izveidojiet projektu ar: freemapper init <dir> --osm { $id }
cli-find-no-area = Nevienai no šīm vietām nav apgabala, kas varētu būt kartes apgabals. Uzzīmējiet apgabalu ar: freemapper select <dir>
cli-find-nothing = vieta „{ $query }“ nav atrasta; mēģiniet citu nosaukumu vai uzzīmējiet apgabalu ar: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = brīdinājums: adapteris mapē { $folder } ir izlaists: { $finding }
cli-adapter-note = brīdinājums: adapteris mapē { $folder } tiek ielādēts, bet: { $finding }
cli-adapter-from-project = brīdinājums: tiek izmantots projekta mapes adapteris { $adapter }; tas var vērsties pie šiem serveriem: { $hosts }
cli-adapter-overlaps = brīdinājums: adapteris { $adapter } aptver arī šo teritoriju: { $others }; tā objekti nonāk tur izveidotajās kartēs
cli-adapter-dir-relative = brīdinājums: FREEMAPPER_ADAPTER_DIR nav absolūts ceļš ({ $path }); tas tiek ignorēts
cli-adapters-no-project = brīdinājums: { $path } nav mape; projekta adapteri netika lasīti
cli-adapters-overlaps = pārklājas ar: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revīzija { $revision }  pārbaudīts { $checked }  serveri: { $hosts }
cli-adapters-source-built-in = iebūvēts
cli-adapters-source-user = no jūsu mapes
cli-adapters-source-project = no projekta
cli-adapters-never-checked = nekad
cli-adapters-user-dir = Jūsu adapteri tiek lasīti no { $path }
cli-adapters-no-user-dir = Šajā sistēmā nav mapes jūsu adapteriem; FREEMAPPER_ADAPTER_DIR norāda kādu

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper tagad drīkst vaicāt arī serverim { $host } par { $name }. Ieraksts ir failā { $file }; `freemapper adapters disallow-host { $adapter } { $host }` to atsauc. Darbojoša apgabala lapa vai AI asistents ierakstu nolasa, kad tiek palaists atkārtoti
cli-allow-host-already = { $host } jau ir atļauts par { $name }
cli-allow-host-declared = adapteris { $name } pats nosauc { $host }; nav ko atļaut
cli-allow-host-refused = { $host } nevar atļaut: { $reason }
cli-allow-host-unknown-adapter = nav iebūvēta adaptera { $adapter }; iebūvētie adapteri ir: { $adapters }
cli-allow-host-folder-adapter = { $adapter } ir adapteris no jūsu adapteru mapes; tā vietā pievienojiet serveri tā manifesta sadaļai `[net] hosts`
cli-allow-host-no-config = šajā sistēmā nav konfigurācijas mapes, kur glabāt atļautos serverus
cli-allow-host-write = { $file } nevarēja ierakstīt: { $detail }
cli-allow-host-unreadable = { $file } nevar nolasīt ({ $detail }); vispirms izlabojiet vai noņemiet failu; nekas netika mainīts
cli-disallow-host-removed = freemapper vairs nevaicā serverim { $host } par { $name }
cli-disallow-host-absent = { $host } nebija atļauts par { $name }; nav ko atsaukt
cli-allowed-hosts-unreadable = brīdinājums: { $file } nevarēja nolasīt ({ $detail }); papildu serveri netiek atļauti
cli-allowed-hosts-ignored = brīdinājums: { $file }: ieraksts { $host } adapterim { $adapter } tiek ignorēts: { $reason }
cli-allowed-hosts-no-adapter = freemapper nav iebūvēta adaptera ar šādu nosaukumu
cli-allowed-host-redundant = brīdinājums: { $host } tagad ir viens no serveriem, ko nosauc adapteris { $name }; jūsu ieraksts failā { $file } tika noņemts
cli-adapters-allowed = jūsu atļautie: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapteris { $adapter } ({ $name }), revīzija { $revision }, serveri: { $hosts }
cli-check-error = kļūda: { $finding }
cli-check-warning = brīdinājums: { $finding }
cli-check-passed = { $folder } iztur pārbaudi
cli-check-passed-warnings = { $folder } iztur pārbaudi, { $count ->
        [zero] ar { $count } brīdinājumiem
        [one] ar { $count } brīdinājumu
       *[other] ar { $count } brīdinājumiem
    }
cli-check-failed = { $folder } neiztur pārbaudi
cli-check-no-folder = mapes { $folder } nav
cli-check-live-ok = { $capability } kārtībā: { $detail }
cli-check-live-failed = { $capability } NEIZDEVĀS: { $detail }
cli-check-live-not-asked = { $capability } nav vaicāts: { $detail }
cli-check-live-box-outside = taisnstūris neatrodas adaptera { $adapter } robežās; dzīvā pārbaude vaicā pakalpojumiem vietu, ko adapteris aptver
cli-check-live-cannot = dzīvo pārbaudi nevar veikt: { $detail }
cli-check-detail-not-declared = adapteris to nedeklarē
cli-check-detail-stopped = iepriekšējā daļa neizdevās
cli-check-detail-see-warnings = skatiet brīdinājumus
cli-check-detail-outside-dataset = taisnstūris atrodas ārpus datu kopas
cli-check-detail-objects = { $count ->
        [zero] { $count } objektu
        [one] { $count } objekts
       *[other] { $count } objekti
    }
cli-check-detail-layers = slāņi: { $layers }, objekti: { $count }
cli-check-detail-trees = { $count ->
        [zero] { $count } koku
        [one] { $count } koks
       *[other] { $count } koki
    }
cli-check-detail-extract = { $label }, dati no { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = reljefa modelis
cli-check-raster-surface = virsmas modelis
cli-check-raster-tiles = { $product }: { $count ->
        [zero] { $count } flīžu
        [one] { $count } flīze
       *[other] { $count } flīzes
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } un { $last }

## Folder lock

cli-lock-held = šo mapi jau izmanto cita freemapper palaišana (kopš { $time }, to sāka: { $who }). Pagaidiet, līdz tā beidzas, vai vispirms to apturiet.
cli-lock-who-cli = terminālis
cli-lock-who-mcp = AI aģents (MCP)
cli-lock-who-page = izvēles lapa
cli-lock-who-unknown = cita programma
cli-lock-failed = neizdevās bloķēt projekta mapi šai palaišanai: { $error }
cli-lock-stuck = ceļā ir kaut kas cits, nevis bloķēšanas fails, un to nevarēja noņemt

## Run routes of the selection page

cli-run-bad-request = Palaišanas opcijas nav derīgas: { $error }
cli-run-no-project = Šajā mapē vēl nav saglabāta projekta. Vispirms saglabājiet apgabalu.
cli-run-already-running = Šim projektam jau notiek palaišana ({ $job }). Pagaidiet, līdz tā beidzas, vai vispirms to atceliet.
cli-run-disk-stops = Lejupielādei nepietiek brīvas vietas diskā: { $messages }
cli-run-busy-elsewhere = Cits šīs programmas projekts šobrīd veic palaišanu. Projekti dala lejupielāžu kešatmiņu, tāpēc vienlaikus darbojas tikai viena palaišana. Mēģiniet vēlreiz, kad tā būs beigusies.
cli-run-nothing-to-cancel = Neviena palaišana nenotiek, tāpēc nav ko atcelt.
cli-run-unknown-job = Šim projektam nav palaišanas { $job }.
cli-status-too-fast = Statusa pieprasījumi nāk pārāk bieži. Jautājiet ne biežāk kā piecas reizes sekundē.
cli-open-folder-failed = Mapi neizdevās atvērt ({ $error }).
cli-open-folder-no-opener = Šajā datorā nav programmas mapju atvēršanai ({ $program }). Atveriet mapi manuāli.
cli-open-folder-missing = Projekta mape vēl nepastāv. Vispirms saglabājiet apgabalu.
cli-open-folder-refused = Mapes nosaukums nav pietiekami drošs, lai to nodotu failu pārvaldniekam.
cli-save-running = Šim projektam notiek palaišana. Saglabājiet vēlreiz, kad tā būs beigusies.
cli-save-unknown-kind = Nezināms kartes veids „{ $kind }“. Lietojiet schoolyard, sprint vai forest.
cli-save-scale-too-large = Mērogs { $scale } ir pārāk liels.
cli-save-bad-content = Satura iestatījumi nav derīgi. Nekas netika saglabāts.
cli-map-image-none = Vispirms izveidojiet karti.
cli-map-image-gone = Šis priekšskatījums vairs nav pieejams.
cli-map-image-running = Šim projektam notiek palaišana. Priekšskatījumu var aprēķināt, kad tā būs beigusies.
cli-map-image-stale = Saglabātais projekts vairs neatbilst ģenerētajai kartei, piemēram, tāpēc, ka mainījies apgabals, kartes veids vai mērogs. Vispirms sāciet palaišanu, kas aizstāj karti („Aizstāt esošo karti“ lapā vai freemapper run --force).
cli-map-image-superseded = Starplaikā pieprasīts jaunāks priekšskatījums, tāpēc šis netika aprēķināts.
cli-map-image-bad-content = Satura iestatījumi nav derīgi.
cli-map-image-bad-request = Priekšskatījuma pieprasījums nav derīgs. Nosūtiet JSON objektu, ja vajadzīgs, ar satura iestatījumiem.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – apgabala izvēle
page-heading = Izvēlieties kartes apgabalu
page-update-available = Ir pieejams freemapper { $version }. Atjauniniet ar
page-update-notes = Laidiena piezīmes
page-folder = Projekta mape
page-pick = Meklēt vietu
page-query-placeholder = piem., Clara-Zetkin-Schule Strausberg
page-search-button = Meklēt
page-searching = Meklē …
page-no-results = Nekas nav atrasts.
page-point-only = tikai punkts, nevis apgabals
page-draw = Zīmējiet paši
page-rectangle = Taisnstūris
page-polygon = Daudzstūris
page-finish-polygon = Aizvērt daudzstūri
page-clear = Notīrīt
page-hint-rectangle = Noklikšķiniet kartē uz diviem pretējiem stūriem.
page-hint-polygon = Noklikšķiniet uz stūriem pēc kārtas. Lai daudzstūri aizvērtu, noklikšķiniet uz pirmā stūra, veiciet dubultklikšķi vai nospiediet „Aizvērt daudzstūri“. Esc atceļ, Backspace noņem pēdējo stūri.
page-hint-idle = Izvēlieties vietu no meklēšanas rezultātiem vai uzzīmējiet apgabalu paši.
page-summary = Apgabals
page-source-osm = No OpenStreetMap: { $name }
page-source-drawn = Uzzīmēts ar roku
page-size = Izmērs ar buferi: { $width } × { $height } m
page-tiles-one = Flīzes ({ $n }): { $names }
page-tiles-other = Flīzes ({ $n }): { $names }
page-download = Lejupielāde: apmēram { $mb } MB
page-download-none = Lejupielāde: viss jau ir kešatmiņā
page-download-details = Kas tiek lejupielādēts
page-cached = kešatmiņā
page-disk = Vieta diskā: { $volumes }
page-disk-volume = { $free } brīvas, vajadzīgas apmēram { $needed }
page-cache-volume = kešatmiņa
page-project-volume = projekta mape
page-no-room = neietilpst diskā
page-no-room-skip = Ne viss ietilpst diskā (freemapper patur { $reserve } brīvu). Lejupielāde izlaiž atzīmētās daļas. Ja tās ir vajadzīgas, vispirms atbrīvojiet vietu:
page-no-room-stop = Diskā nepietiek vietas (freemapper patur { $reserve } brīvu). Bez atzīmētajām daļām kartes nav, tāpēc lejupielāde apstājas, pirms kaut ko lejupielādē. Vispirms atbrīvojiet vietu:
page-disk-unknown = freemapper nevar nolasīt brīvo vietu diskā ({ $error }). Tas lejupielādē bez pārbaudes.
page-too-large = Apgabals ir pārāk liels: ne vairāk kā { $max } m katrā malā (ar buferi). Lūdzu, izvēlieties mazāku.
page-outside = Apgabals atrodas ārpus atbalstītajām federālajām zemēm ({ $states }); freemapper citiem reģioniem datu avotu vēl nav.
page-outside-unknown = Apgabals atrodas ārpus reģioniem, ko freemapper atbalsta; tur tam vēl nav datu avotu.
page-adapters-failed = Aerofotogrāfijas nevar uzskaitīt ({ $detail }). Karte rāda tikai OpenStreetMap; viss pārējais darbojas.
page-settings = Karte
page-kind = Kartes veids
page-schoolyard = Skolas pagalma karte (1:1000)
page-sprint = Sprinta karte (1:4000)
page-forest = Meža karte (1:15000)
page-scale = Mērogs 1:
page-map-title = Nosaukums
page-contour-interval = Horizontāļu intervāls
page-contour-auto = automātiski
page-settings-fixed = Projekts jau pastāv; kartes veids, mērogs un nosaukums ir failā freemapper.toml. Horizontāļu intervālu joprojām var mainīt.
page-save = Saglabāt
page-saving = Saglabā …
page-saved = Saglabāts failā freemapper.toml.
page-zone-changed = Saglabāts. Apgabals tagad atrodas citā UTM zonā; freemapper nākamajā palaišanā datus lejupielādēs no jauna.
page-provider-changed = Saglabāts. Jaunā apgabala augstuma un aerofotogrāfiju datus sniedz cits datu nodrošinātājs; freemapper nākamajā palaišanā datus lejupielādēs no jauna.
page-save-stale = freemapper.toml kopš šīs lapas nolasīšanas ir mainīts citā logā vai ar citu programmu. Nekas netika saglabāts; lapa tagad zina saglabāto stāvokli. Saglabājiet vēlreiz, lai to aizstātu ar šeit redzamo apgabalu.
page-has-map = Esošā karte (.omap) paliek, kāda bija; freemapper izveido jaunas veidnes un kandidātus jaunajam apgabalam. Opcija „Aizstāt esošo karti“ zemāk (vai „freemapper run --force“) izveido arī karti no jauna un vecā paliek kā .omap.bak.
page-done = Gatavs
page-source = Pirmkods (AGPL-3.0-or-later)
page-closed = Gatavs. Varat aizvērt šo logu.
page-next = Tālāk: freemapper run "{ $dir }"
page-made-files = Izveidots: { $files }. Mape: { $dir }
page-back-to-agent = Pasakiet savam AI asistentam, ka apgabals ir saglabāts.
page-nothing-saved = Nekas netika saglabāts.
page-failed = Kļūda: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Aerofotogrāfija { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Kas nonāk kartē
page-content-help = Profils nosaka, kas nonāk tieši kartē un kas paliek kā pārbaudāms kandidāts. Izvēle grupai ir svarīgāka par profilu. Tā tiek saglabāta kopā ar apgabalu.
page-content-profile-legend = Profils
page-content-profile-safe = Drošs
page-content-profile-safe-hint = Katrs objekts paliek tur, kur to ieliek kartes veidošana: ēkas un kadastra laukumi nonāk kartē; mūri, žogi, koki un daudz kas no OpenStreetMap (dīķi, ceļi bez zināma seguma) paliek kandidāti.
page-content-profile-balanced = Sabalansēts
page-content-profile-balanced-hint = Kadastra, reģistru un ielu uzmērījuma objekti nonāk kartē, kopā ar horizontālēm un aizliegtajām zonām gar dzelzceļa sliedēm. Mūri, žogi, dzīvžogi, nogāzes, kāpnes, ūdens, koki, pieminekļi, caurejas, takas un dzelzceļi nāk arī no OpenStreetMap. Ielu aprīkojums, pārējās aizliegtās zonas un reljefa modeļa formas paliek kandidāti.
page-content-profile-full = Pilns
page-content-profile-full-hint = Viss nonāk kartē, tāpat kā pārbaudes izdrukā; ārpus paliek tikai paslēptie kandidāti.
page-content-groups-legend = Pa grupām
page-content-groups-help = „Kā profilā“ seko profilam. Ko tas nozīmē grupai, atkarīgs no tā, no kurienes nāk katrs objekts.
page-content-place-profile = Kā profilā
page-content-place-map = Karte
page-content-place-candidates = Kandidāts
page-content-place-off = Izslēgts
page-content-group-walls = Mūri
page-content-group-fences = Žogi
page-content-group-hedges = Dzīvžogi
page-content-group-slopes = Nogāzes un klintis
page-content-group-stairs = Kāpnes
page-content-group-trees = Koki un koku grupas
page-content-group-water-features = Ūdens, strūklakas un avoti
page-content-group-monuments = Torņi, pieminekļi un elektrolīnijas
page-content-group-furniture = Ielu aprīkojums un laternas
page-content-group-passages = Caurejas
page-content-group-contours = Horizontāles
page-content-group-landforms = Pauguri, ieplakas, nogāzes un klintis no reljefa modeļa
page-content-group-vegetation = Veģetācija no lāzera punktu mākoņiem (Brandenburga, Berlīne, Ziemeļreina-Vestfālene, Reinzeme-Pfalce, Zāra un Bavārija; izslēgta, kamēr to neizvēlaties; lejupielādē apmēram 40 līdz 600 MB uz km²)
page-content-group-out-of-bounds = Aizliegtās zonas
page-content-group-paths = Takas un dzelzceļi
page-content-group-streets = Ielas un laukumi
page-run-heading = Izveidot karti
page-preview-heading = Kartes priekšskatījums
page-preview-stored = Kā ģenerēts pēdējo reizi.
page-preview-live = Ar šīs lapas iestatījumiem, vēl nesaglabāts.
page-preview-live-saved = Ar saglabātajiem iestatījumiem, vēl nav ģenerēts palaišanā.
page-preview-computing = Aprēķina …
page-preview-none = Karte vēl nav ģenerēta.
page-preview-later = Šai kartei vēl nav priekšskatījuma. Tas parādīsies ar nākamo palaišanu, kas ieraksta karti.
page-preview-stale = Apgabals, kartes veids vai mērogs ir mainījies kopš kartes ģenerēšanas. Attēls joprojām rāda karti, kāda tā ģenerēta pēdējo reizi; jaunam priekšskatījumam vajadzīga palaišana ar „Aizstāt esošo karti“.
page-preview-area-unsaved = Apgabals šajā lapā vēl nav saglabāts. Attēls rāda karti, kāda tā ģenerēta pēdējo reizi; jaunajam apgabalam vajadzīga palaišana ar „Aizstāt esošo karti“.
page-preview-run = Kā ģenerēts pēdējo reizi. Lapa atjaunina attēlu, kad palaišana beidzas.
page-preview-open = Atvērt pilnā izmērā
page-preview-alt = Kartes objektu priekšskatījums
page-opt-ocad = Ierakstīt OCAD sākuma projektu
page-help-ocad = Visiem, kas turpina darbu OCAD.
page-opt-pdf = Drukājams PDF
page-help-pdf = Tikai kartes objekti, bez kandidātiem.
page-opt-pdf-candidates = Pārbaudes izdruka ar visiem kandidātiem
page-opt-training = Treniņu karte
page-help-training = Karte un visi kandidāti, sakopti, lai tos drukātu tādus, kādi tie ir; ar OCAD arī kā OCAD fails.
page-views-heading = Treniņu kartes skati
page-view-no-paths = Bez takām (Ultimate karte)
page-view-brown-only = Tikai brūns (horizontāļu orientēšanās)
page-view-no-north-lines = Bez ziemeļu līnijām
page-view-custom = Cits skats
page-help-views = Katrs atzīmētais skats ir viens papildu fails blakus treniņu kartei, `<name>-training-<view>.omap` un tā PDF, ar OCAD arī OCAD fails. Cits skats ir `no:505,506` (izlaist simbolus) vai `only:101,102` (atstāt tikai šos); `+` apvieno skatus. Nosaukums, mērogs un avoti paliek vienmēr.
page-help-views-forest = Bez takām un tikai brūns ir tikai meža kartēm.
page-opt-scale-bar = Mēroga līnija PDF failos
page-help-scale-bar = Vācijas kartēm parasti tādas nav.
page-opt-sources = Avotu rindas PDF failos
page-sources-auto = automātiski
page-sources-full = pilnas
page-sources-credit-only = tikai freemapper
page-warn-credit-only = Tad PDF failos minēts tikai freemapper. Datu licences (ODbL, dl-de/by-2-0) prasa norādīt avotus, kad karte tiek nodota tālāk; tad avotus norādāt paši. ATTRIBUTION.txt tos uzskaita visus.
page-need-pdf = Mēroga līnija un avotu rindas attiecas tikai tad, kad tiek veidots PDF.
page-opt-offline = Tikai kešatmiņa, bez tīkla
page-opt-force = Aizstāt esošās kartes
page-help-force = Karte un, ja atzīmēts „Treniņu karte“, arī treniņu karte. Vecie faili paliek kā .omap.bak.
page-opt-permits = Atļauju piezīmes
page-help-permits = Administratīvās vienības, aizsargājamās teritorijas un kam jautāt, failā permits.md. Norāde, nevis juridisks padoms.
page-run-start = Izveidot karti
page-run-again = Izveidot vēlreiz
page-run-cancel = Atcelt
page-run-cancelling = Atceļ …
page-warnings-heading = Brīdinājumi
page-where-files = Faili atrodas projekta mapē:
page-open-folder = Atvērt mapi
page-log-label = Palaišanas žurnāls
page-force-title = Aizstāt esošās kartes?
page-force-text = Esošā karte tiek aizstāta, un, ja atzīmēts „Treniņu karte“, arī treniņu karte. Jūsu labojumi neaiziet zudumā: vecie faili paliek projekta mapē kā .omap.bak. Turpināt?
page-force-cancel = Atcelt
page-force-confirm = Aizstāt

page-step-fetch = Lejupielādē datus
page-step-derive = Aprēķina veidnes
page-step-map = Raksta karti
page-step-ocad = Raksta OCAD projektu
page-step-pdf = Raksta PDF
page-step-pdf-candidates = Raksta pārbaudes izdruku
page-step-training = Raksta treniņu karti
page-step-permits = Raksta atļauju piezīmes
page-step-line = Solis { $index } no { $total }: { $label }
page-starting = Palaišana sākas …
page-file-kind-map = karte
page-file-kind-template = veidne
page-file-kind-ocad = OCAD karte
page-file-kind-ocad-candidates = OCAD kandidāti
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = pārbaudes izdruka
page-file-kind-training-map = treniņu karte
page-file-kind-training-pdf = treniņu kartes PDF
page-file-kind-training-ocad = OCAD treniņu karte
page-file-kind-training-view-map = treniņu kartes skats
page-file-kind-training-view-pdf = treniņu kartes skata PDF
page-file-kind-training-view-ocad = OCAD treniņu kartes skats
page-file-kind-preview = kartes priekšskatījums
page-file-kind-permits = atļauju piezīmes
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Gatavs. Faili ir ierakstīti.
page-result-failed = Palaišana neizdevās: { $error }
page-result-failed-kept = Kas ierakstīts, paliek projekta mapē; jauna palaišana turpina no turienes.
page-result-cancelled = Atcelts. Jau lejupielādētais paliek kešatmiņā; jauna palaišana turpina no turienes.
page-result-title-done = Karte izveidota
page-result-title-failed = Palaišana neizdevās
page-result-title-cancelled = Palaišana atcelta
page-kept-map = Esošā karte tika atstāta nemainīta. „Aizstāt esošo karti“ izveido to no jauna un vecā paliek kā .omap.bak.
page-no-files = Faili netika izveidoti.
page-opened-folder = Mape ir atvērta.
page-open-failed = { $message } Mape: { $dir }
page-why-no-area = Vispirms izvēlieties apgabalu.
page-why-checking = Priekšskatījums tiek aprēķināts …
page-why-too-large = Apgabals ir pārāk liels.
page-why-outside = Apgabals atrodas ārpus atbalstītajām federālajām zemēm.
page-why-disk = Nepietiek brīvas vietas diskā (skat. iepriekš).
page-why-running = Notiek palaišana.
page-err-scale-bar-no-pdf = Mēroga līnijai vajadzīgs PDF.
page-err-sources-no-pdf = Avotu rindām vajadzīgs PDF.
page-save-failed = Apgabals nav saglabāts; palaišana netika sākta.
page-poll-lost = Serveris neatbild. freemapper turpina mēģināt.
page-run-unknown = Serveris šo palaišanu vairs nepazīst.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Aizvērt

## update (spec 3.4 to 3.6)

update-source-build = Šis freemapper ir būvēts no pirmkoda. Atjauniniet to ar git pull un cargo build --release --locked.
update-current = freemapper { $version } ir aktuāls.
update-available = Ir pieejams freemapper { $new } (jums ir { $old }), izdots { $released }.
update-details =
    Laidiena piezīmes: { $notes }
    Lejupielāde: { $size } MB
update-decimal-separator = ,
update-ask = Instalēt tagad? [j/N]
update-not-installed = Nekas netika instalēts.
update-needs-yes = ievade nav terminālis, tāpēc freemapper nevar pajautāt pirms instalēšanas; palaidiet freemapper update --yes, lai instalētu bez jautāšanas
update-installing = lejupielādē un pārbauda freemapper { $version }
update-done = freemapper atjaunināts no { $old } uz { $new }. Laidiena piezīmes: { $notes }
update-mcp-restart = Darbojošs MCP serveris paliek pie vecās versijas, kamēr AI asistents to atkārtoti nepalaiž.
update-rolled-back = Atgriezts pie freemapper { $version }. Versija, ko atstājāt, ir saglabāta kā { $path }.
update-not-writable = nevar rakstīt programmas mapē { $path }, un freemapper nepieprasa administratora tiesības. Izsaiņojiet jauno laidienu tur manuāli vai pārvietojiet freemapper uz mapi, kurā varat rakstīt.
update-too-old = freemapper { $new } nevar instalēt ar versijas { $old } pašatjaunināšanu; instalējiet { $needed } vai jaunāku manuāli, skatiet { $notes }
update-busy = programmas mapē darbojas cits atjauninājums vai atgriešana (bloķēšanas fails { $path }); pagaidiet, līdz tas beidzas, un mēģiniet vēlreiz
update-bad-signature = laidienu neizdevās pārbaudīt ({ $error }); nekas netika instalēts
update-check-bad-signature = jaunāko freemapper laidienu neizdevās pārbaudīt ({ $error }); nekas netika instalēts
update-no-manifest = jaunākais laidiens pats sevi nevar instalēt (tam nav atjaunināšanas manifesta); lejupielādējiet to no { $page }
update-no-manifest-version = laidiena { $version } nav vai tas pats sevi nevar instalēt (tam nav atjaunināšanas manifesta); skatiet { $page }
update-no-asset = laidienam nav programmas šai sistēmai ({ $platform }); nekas netika instalēts
update-no-previous = nav iepriekšējās versijas, uz kuru atgriezties (trūkst { $path })
update-failed = atjaunināšana neizdevās: { $error }
update-mode-set = Atjaunināšanas režīms iestatīts uz { $mode } failā { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } pārraksta šo iestatījumu, kamēr tas ir iestatīts.
update-mode-failed = nevarēja ierakstīt { $path }: { $error }
update-notice = Ir pieejams freemapper { $new } (jums ir { $old }). Atjauniniet ar: freemapper update
update-applies-next-start = Jaunā versija stājas spēkā no freemapper nākamās palaišanas.
update-install-failed = freemapper { $new } netika instalēts: { $error }
update-no-config-dir = šajā sistēmā nav konfigurācijas mapes; iestatiet FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Piezīme personai: startējot freemapper serveris atrada šo par adapteru mapēm. Nododiet to tālāk; persona izlabo mapi vai to noņem, tad atkārtoti palaiž serveri.
    { $lines }
update-mcp-note = Piezīme personai: ir pieejams freemapper { $new } (šis ir { $old }). Pajautājiet personai pirms install_update izsaukšanas.
update-mcp-needs-confirm = nekas netika instalēts; vispirms pajautājiet personai un izsauciet install_update ar confirm: true, kad tā piekrīt
update-mcp-note-manual = Piezīme personai: ir pieejams freemapper { $new } (šis ir { $old }), bet install_update to nevar instalēt; instalējiet { $needed } vai jaunāku manuāli, skatiet { $notes }
update-mcp-note-bad-signature = Piezīme personai: jaunāko freemapper laidienu neizdevās pārbaudīt ({ $error }); nekas netika instalēts. Pastāstiet tai un neinstalējiet to citā veidā.
update-mcp-restart-now = Tagad atkārtoti palaidiet freemapper MCP serveri; tas joprojām darbojas ar veco versiju.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = kļūda
error-cancelled = atcelts
cli-cancelled-kept = atcelts; atstāts: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disks ir pilns, tāpēc freemapper nevarēja ierakstīt { $path }. Atbrīvojiet vietu un mēģiniet vēlreiz

## Errors: project

error-project-already-exists = projekts jau pastāv: { $path }
error-project-map-exists = karte jau pastāv (tajā var būt jūsu darbs); lai to aizstātu, lietojiet --force: { $path }
error-project-invalid-scale = nederīgs mērogs 1:{ $scale }
error-project-invalid-contour-interval = nederīgs horizontāļu intervāls { $interval } m failā freemapper.toml; tam jābūt lielākam par 0
error-project-scale-not-allowed = mērogs 1:{ $scale } nav atļauts { $kind } kartēm; lietojiet { $allowed }
error-project-scale-not-allowed-enlarge = mērogs 1:{ $scale } nav atļauts { $kind } kartēm; lietojiet { $allowed }. Zīmējiet pie 1:4000, freemapper-print drukājot palielina līdz 1:3000
word-kind-schoolyard = skolas pagalma
word-kind-sprint = sprinta
word-kind-forest = meža
error-project-toml = nederīgs projekta fails: { $message }
error-project-unknown-adapter = projekta fails nosauc adapteri „{ $adapter }“, kura šim freemapper nav; jaunāks freemapper to var nodrošināt (`freemapper update`), vai arī tas nāk no jūsu adapteru mapes (`freemapper adapters list` to parāda) vai, ar --adapters-from-project komandrindā, no projekta mapes `adapters` (tikai projektam, kuram uzticaties)
error-project-not-fetched = flīzes vēl nav lejupielādētas; vispirms palaidiet `freemapper fetch`
error-project-ocd = OCAD eksports: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: attēls un world fails neatbilst kartes apgabalam; vēlreiz palaidiet `freemapper derive`
error-project-tiles-outdated = kartes apgabalam vajadzīgas flīzes { $tiles }, kas netika lejupielādētas; palaidiet `freemapper fetch`
error-project-buildings-not-fetched = šim kartes apgabalam nav saglabātu ALKIS ēku; vispirms palaidiet `freemapper fetch` tiešsaistē
error-project-orthophoto-not-fetched = šim kartes apgabalam nav saglabāta ortofoto; vispirms palaidiet `freemapper fetch` tiešsaistē
error-project-terrain-not-fetched = šim kartes apgabalam nav saglabāta reljefa modeļa; vispirms palaidiet `freemapper fetch` tiešsaistē
error-project-candidates-outdated = saglabātie OSM kandidāti neaptver kartes apgabalu; palaidiet `freemapper fetch`
error-project-stored = { $path }: { $message }; vēlreiz palaidiet `freemapper fetch`
error-project-area-too-large = kartes apgabals plus buferis ir { $width } × { $height } m, vairāk nekā { $limit } m katrā malā; samaziniet kartes apgabalu vai buffer_m failā freemapper.toml
error-project-area-crosses-country = kartes apgabals atrodas abpus valsts robežai; kartes pāri robežai vēl netiek atbalstītas, tāpēc uzzīmējiet apgabalu vienas valsts iekšienē
error-project-permits-not-for-region = atļauju piezīmes šim reģionam vēl nav pieejamas: tās vaicā Vācijas pakalpojumiem un nosauc Vācijas iestādes
error-project-outside-coverage = kartes apgabals atrodas ārpus atbalstītajām federālajām zemēm ({ $states }); freemapper pagaidām ir datu avoti tikai tiem

## Errors: disk space

error-space-required = nepietiek vietas diskā: { $part }. Vajadzīgs { $needed }, bet { $path } ir brīvs tikai { $left } virs { $reserve } rezerves. Atbrīvojiet vietu un vēlreiz palaidiet `freemapper fetch`
error-space-skipped = freemapper izlaiž šo: { $part }, jo nepietiek vietas diskā. Vajadzīgs { $needed }, bet { $path } ir brīvs tikai { $left } virs { $reserve } rezerves. Atbrīvojiet vietu un vēlreiz palaidiet `freemapper fetch`
word-part-map-data = ALKIS un OSM kartes dati
word-part-terrain = reljefa modelis
word-part-orthophoto = ortofoto
word-part-surface = virsmas modelis nDSM veidnei
word-part-osm-update = kešotā OSM izvilkuma atjauninājums
word-part-k-five = Berlīnes K5 kartes veidne
word-part-berlin-layers = Berlīnes ALKIS struktūras, koku kadastrs, ielu uzmērījums un punktu slāņi
word-part-state-layers = pilnās ALKIS struktūras, ainavas modeļa objekti un koku kadastri
word-part-point-cloud = lāzera punktu mākoņi veģetācijas kandidātiem

## Errors: downloads

error-fetch-http = { $url } lejupielāde neizdevās: { $message }
error-fetch-http-temporary = { $url } lejupielāde neizdevās: { $message }. Pakalpojums, iespējams, ir pārslogots vai tiek apkopts; freemapper mēģināja līdz { $attempts } reizēm. Palaidiet izpildi vēlreiz pēc dažām minūtēm; jau lejupielādētais tiek saglabāts.
error-fetch-corrupt-archive = { $url } lejupielāde nav pilns flīžu arhīvs: { $message }
error-fetch-corrupt-extract = { $url } lejupielāde nav pilns OSM izvilkums: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] flīze { $tiles } nav iekļauta publicētajās flīzēs; samaziniet kartes apgabalu vai buffer_m failā freemapper.toml
       *[other] flīzes { $tiles } nav iekļautas publicētajās flīzēs; samaziniet kartes apgabalu vai buffer_m failā freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] bezsaistē un kešatmiņā nav: { $product } flīze { $tiles }; palaidiet `freemapper fetch` tiešsaistē
       *[other] bezsaistē un kešatmiņā nav: { $product } flīzes { $tiles }; palaidiet `freemapper fetch` tiešsaistē
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = reljefa modelis (DGM1)
word-product-image-dsm = virsmas modelis (bDOM)
word-product-saxony-dtm = reljefa modelis (GeoSN DGM1)
word-product-saxony-dsm = virsmas modelis (GeoSN DOM1)
error-fetch-osm-extract-missing = kešatmiņā nav OSM izvilkuma; palaidiet `freemapper fetch` tiešsaistē
error-fetch-not-found = { $name } serverī nav atrasts
error-fetch-refused = { $url }: serveris noraidīja pieprasījumu (HTTP { $status })
error-fetch-refused-detail = { $url }: serveris noraidīja pieprasījumu (HTTP { $status }): { $detail }
error-fetch-too-large = { $url } lejupielāde ir lielāka par { $limit } MB; apturēta
error-fetch-not-stored = šim kartes apgabalam nav saglabāts ({ $label }): { $path } trūkst, nav nolasāms vai neaptver to; vispirms palaidiet `freemapper fetch` tiešsaistē
error-fetch-range-ignored = { $url }: serveris ignorēja baitu diapazonu { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper nesūta pieprasījumus uz šo adresi: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper nesūta pieprasījumus uz šo adresi: adapteris { $name } nenosauc serveri { $host }{ $from }. Ja pakalpojums ir pārcelts uz turieni, atļaujiet serveri ar `freemapper adapters allow-host { $adapter } { $host }` un ziņojiet par to https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper nesūta pieprasījumus uz šo adresi: adapteris { $adapter } nenosauc serveri { $host }{ $from }. Ja pakalpojums ir pārcelts uz turieni, pievienojiet serveri adaptera manifesta sadaļai `[net] hosts`
error-fetch-host-redirected-from = {" "}({ $url } novirza uz turieni)
error-reason-not-a-host-name = nav servera nosaukums
error-reason-not-ascii = nosaukums ārpus ASCII jāraksta tā xn-- formā
error-reason-not-a-dns-name = nav DNS nosaukums
error-reason-no-domain = šī datora nosaukums vai nosaukums bez domēna
error-reason-local-name = vietējā tīkla nosaukums
error-reason-ip-address = IP adrese, nevis DNS nosaukums
error-reason-unexpected-character = neparedzēta rakstzīme
error-reason-not-https = nav https
error-reason-user-info = lietotāja informācija
error-reason-port = ports, kas nav 443
error-reason-host-not-declared = serveris nav viens no deklarētajiem serveriem
error-reason-local-address = šī datora vai vietējā tīkla adrese, nevis interneta servera adrese
error-reason-too-many-redirects = vairāk nekā { $count } novirzījumi
error-reason-redirect-nowhere = novirze (HTTP { $status }) uz nekurieni
error-fetch-currency-csv = aktualitātes CSV rinda { $line }: { $message }
error-alkis = nederīgi ALKIS dati: { $message }
error-osm = OSM dati: { $message }

## Errors: templates and rasters

error-derive-zip = flīžu arhīvs: { $message }
error-derive-image = attēls: { $message }
error-derive-no-tiles = nav flīžu, no kurām aprēķināt
error-derive-area-too-large = kartes apgabals ir pārāk liels vienai veidnei: { $width } × { $height } pikseļi (ne vairāk kā { $limit } katrā malā); samaziniet kartes apgabalu vai buffer_m failā freemapper.toml
error-raster-world-file = nederīgs world fails: { $message }
error-raster-misaligned = rastri nav izlīdzināti pēc vienāda pikseļu režģa: { $message }
error-raster-pixel-size = rastriem ir atšķirīgi pikseļu izmēri: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = nederīgs GeoJSON: { $message }
error-area-no-polygon = GeoJSON nesatur daudzstūri
error-area-multi-polygon = MultiPolygon, daļu skaits: { $parts }; sadaliet kartes apgabalu vienā daudzstūrī
error-area-out-of-range = koordināta ārpus diapazona: lat { $lat }, lon { $lon }
error-area-not-an-area = nav derīgs kartes apgabals: { $message }
error-georef-date = datums { $date } ir ārpus World Magnetic Model derīguma
error-georef-map-range = punkts E { $easting } N { $northing } atrodas ārpus kartes koordinātu diapazona
error-omap-xml = nederīgs OOM XML: { $message }
error-omap-missing-element = elements <{ $name }> nav atrasts
error-ocd-duplicate-symbol = simbola numurs { $symbol } ir lietots divreiz
error-ocd-unknown-symbol = objekts atsaucas uz simbolu { $symbol }, kura failā nav
error-ocd-wrong-type = objektam ar simbolu { $symbol } ir nepareizs tips šim simbolam

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Lietošana:
cli-heading-commands = Komandas:
cli-heading-options = Opcijas:
cli-heading-arguments = Argumenti:
cli-help-possible-values = iespējamās vērtības
cli-help-default = noklusējums
cli-arg-help = Parādīt palīdzību
cli-arg-version = Parādīt versiju
cli-help-help = Parādīt komandas palīdzību
cli-arg-help-command = Komanda, kuras palīdzību rādīt, piemēram, run
cli-parse-hint = Plašāku informāciju skatiet ar '--help'.
cli-double-click-start = freemapper pārlūkprogrammā atver lapu, kurā izvēlaties kartes apgabalu. Projekta mape ir { $path }. Aizveriet šo logu, lai beigtu programmu.
cli-parse-unknown-argument = neparedzēts arguments { $arg }
cli-parse-missing-required = trūkst obligātā argumenta: { $args }
cli-parse-invalid-value = nederīga vērtība { $value } argumentam { $arg }
cli-parse-invalid-value-choices = nederīga vērtība { $value } argumentam { $arg }; iespējamās vērtības: { $values }
cli-parse-invalid-subcommand = nezināma komanda { $value }
cli-parse-missing-subcommand = nepieciešama komanda
cli-parse-too-many-values = neparedzēta vērtība { $value } argumentam { $arg }
cli-parse-argument-conflict = { $arg } nevar lietot kopā ar { $other }

help-freemapper = Veido OOM sākuma projektus no ģeodatiem
help-freemapper-lang = Programmas tekstu valoda, `de` vai `en` (noklusējums: FREEMAPPER_LANG, tad sistēmas valoda, tad angļu)
help-init = Izveidot projekta mapi kartes apgabalam
help-init-dir = Izveidojamā projekta mape
help-init-geojson = GeoJSON fails ar kartes apgabala daudzstūri (EPSG:4326)
help-init-osm = Kartes apgabala OSM objekts, piem., way/992423119
help-init-place = Vietas nosaukums, piem., „Clara-Zetkin-Schule Strausberg“; freemapper izmanto labāko atbilstību ar apgabalu
help-init-part = Kuru daļu izmantot OSM apgabalam no vairākām atsevišķām daļām (1 ir lielākā)
help-init-kind = Kartes veids
help-arg-scale = Jauna projekta mēroga saucējs: skolas pagalms 1000 (noklusējums); sprints 4000 (noklusējums) vai 5000; mežs 15000 (noklusējums), 10000 vai 7500. Citas vērtības tiek noraidītas, jo freemapper-print tās noraida
help-init-title = Kartes nosaukums malas informācijai (noklusējums: mapes nosaukums)
help-export = Ierakstīt sākuma projektu (OOM karte, veidnes, avotu norādes) projekta mapē
help-export-dir = Projekta mape
help-arg-force = Aizstāt esošo kartes failu un vecā saglabāt kā `<name>.omap.bak`
help-export-ocad = Ierakstīt arī OCAD sākumpunktu mapē `ocad/` (OCAD 12 formāts); esošā OOM karte tad paliek, ja nenorādāt --force
help-export-pdf = Ierakstīt arī `<name>.pdf`, drukājamo karti (tikai kartes objekti, bez kandidātiem); esošā OOM karte tad paliek, ja nenorādāt --force
help-export-pdf-candidates = Ierakstīt arī `<name>-candidates.pdf`, pārbaudes izdruku: karte ar visiem kandidātiem, precīzi pēc atrašanās vietas; esošā OOM karte tad paliek, ja nenorādāt --force
help-export-training = Ierakstīt arī treniņu karti `<name>-training.omap` un tās PDF `<name>-training.pdf`: karte un visi kandidāti, sakopti, lai tos drukātu tādus, kādi tie ir; ar --ocad arī `ocad/<name>-training.ocd`. Esošā OOM karte tad paliek, ja nenorādāt --force, un --force aizstāj arī esošo treniņu karti
help-export-view = Ar --training: ierakstīt arī treniņu karti bez tā, ko skats izlaiž, kā `<name>-training-<view>.omap` un tās PDF (ar --ocad arī OCAD failu). Skati: `no-paths` (bez ielām, takām, kāpnēm), `brown-only` (tikai brūni simboli un rāmis), `no-north-lines`, vai saraksts `no:<codes>` / `only:<codes>`; apvienojiet ar `+`. Atkārtojiet opciju vairākiem failiem
help-export-pdf-scale-bar = Zīmēt mēroga līniju PDF failos (pēc noklusējuma izslēgts, Vācijas kartēm parasti tādas nav; mēroga skaitlis paliek nosaukuma blokā)
help-export-pdf-sources = Avotu rindas PDF failos: `auto` drukā pilnu sarakstu un saīsina to tikai tad, ja tas aizņemtu vairāk nekā 5 rindas, `full` nekad nesaīsina, `credit-only` drukā tikai freemapper norādes rindu (ODbL un dl-de/by-2-0 tad prasa nosaukt avotus, kad karte tiek nodota tālāk). ATTRIBUTION.txt vienmēr uzskaita visus avotus
help-fetch = Lejupielādēt kartes apgabala datus (ortofoto, reljefa modelis, nDSM, ALKIS ēkas, OSM)
help-fetch-dir = Projekta mape
help-fetch-offline = Izmantot tikai kešatmiņu, bez piekļuves tīklam
help-arg-cache-dir = Lejupielādēto flīžu kešatmiņas mape (noklusējums: sistēmas kešatmiņas mape + `freemapper`)
help-derive = Aprēķināt veidnes (ortofoto, ēnotais reljefs, nogāzes, lokālais reljefs, nDSM), horizontāles un reljefa formas no lejupielādētajiem datiem
help-derive-dir = Projekta mape
help-run = Lejupielādēt, aprēķināt un eksportēt vienā piegājienā
help-run-dir = Projekta mape
help-run-offline = Izmantot tikai kešatmiņu, bez piekļuves tīklam
help-run-ocad = Ierakstīt arī OCAD sākumpunktu mapē `ocad/` (OCAD 12 formāts)
help-run-pdf = Ierakstīt arī `<name>.pdf`, drukājamo karti (tikai kartes objekti, bez kandidātiem)
help-run-pdf-candidates = Ierakstīt arī `<name>-candidates.pdf`, pārbaudes izdruku: karte ar visiem kandidātiem, precīzi pēc atrašanās vietas
help-run-training = Ierakstīt arī treniņu karti `<name>-training.omap` un tās PDF `<name>-training.pdf`: karte un visi kandidāti, sakopti, lai tos drukātu tādus, kādi tie ir; ar --ocad arī `ocad/<name>-training.ocd`. Esošā treniņu karte paliek, ja nenorādāt --force
help-run-view = Ar --training: ierakstīt arī treniņu karti bez tā, ko skats izlaiž, kā `<name>-training-<view>.omap` un tās PDF (ar --ocad arī OCAD failu). Skati: `no-paths` (bez ielām, takām, kāpnēm), `brown-only` (tikai brūni simboli un rāmis), `no-north-lines`, vai saraksts `no:<codes>` / `only:<codes>`; apvienojiet ar `+`. Esošs skata fails paliek, ja nenorādāt --force
help-run-pdf-scale-bar = Zīmēt mēroga līniju PDF failos (pēc noklusējuma izslēgts; mēroga skaitlis paliek nosaukuma blokā)
help-run-pdf-sources = Avotu rindas PDF failos: `auto` drukā pilnu sarakstu un saīsina to tikai tad, ja tas aizņemtu vairāk nekā 5 rindas, `full` nekad nesaīsina, `credit-only` drukā tikai freemapper norādes rindu (ODbL un dl-de/by-2-0 tad prasa nosaukt avotus, kad karte tiek nodota tālāk). ATTRIBUTION.txt vienmēr uzskaita visus avotus
help-select = Izvēlēties kartes apgabalu pārlūkā, to zīmējot vai izvēloties vietu. Saglabājot tiek ierakstīts freemapper.toml un vajadzības gadījumā izveidota projekta mape
help-select-dir = Projekta mape (saglabājot tiek izveidota, ja nepastāv)
help-select-kind = Jauna projekta kartes veids
help-select-title = Jauna projekta kartes nosaukums
help-select-no-browser = Tikai izdrukāt lapas adresi; neatvērt pārlūku
help-select-port = Ports uz 127.0.0.1 (noklusējums: brīvs)
help-select-offline = Lapa sākas ar atzīmētu „Tikai kešatmiņa, bez tīkla“
help-select-ocad = Lapa sākas ar atzīmētu OCAD sākuma projektu
help-select-pdf = Lapa sākas ar atzīmētu drukājamo PDF
help-select-pdf-candidates = Lapa sākas ar atzīmētu pārbaudes izdruku (visi kandidāti)
help-select-training = Lapa sākas ar atzīmētu treniņu karti
help-select-pdf-scale-bar = Lapa sākas ar atzīmētu mēroga līniju PDF failos
help-select-pdf-sources = Lapa sākas ar šādu avotu rindu izvēli PDF failos, kā `run`
help-find = Uzskaitīt OpenStreetMap vietas pēc nosaukuma, vispirms apgabalus, ar id komandai init --osm
help-find-query = Vietas nosaukums, piem., „Clara-Zetkin-Schule Strausberg“
help-find-json = Izdrukāt atbilstības kā JSON, tie paši lauki kā MCP rīkam find_place
help-mcp = Piedāvāt rīkus AI aģentam, izmantojot MCP (stdin/stdout)
help-update = Instalēt jaunāku freemapper no GitHub laidieniem pēc jautāšanas. Būvējums no pirmkoda tā vietā pasaka, kā to atjaunināt
help-update-check = Tikai pārbaudīt: izejas kods 10, ja pastāv jaunāka versija, 0, ja šī ir aktuāla
help-update-yes = Instalēt bez jautāšanas; vajadzīgs, kad ievade nav terminālis
help-update-version = Instalēt šo versiju (x.y.z) jaunākās vietā, arī vecāku
help-update-rollback = Atgriezties pie versijas, ko aizstāja pēdējais atjauninājums
help-update-mode = Saglabāt, ko dara ikdienas pārbaude, kad atrod jaunāku versiju (noklusējums: notify). Darbojas katrā būvējumā, jo tikai ieraksta iestatījumu
help-adapters = Parādīt reģionu adapterus: federālās zemes un valstis, kurām freemapper ir datu avoti
help-adapters-list = Uzskaitīt adapterus ar to, no kurienes katrs nāk (iebūvēts, jūsu mape, projekts), tā revīziju, pēdējās pārbaudes dienu un serverus, kuriem tas vaicā
help-adapters-list-json = Izdrukāt sarakstu kā JSON
help-adapters-list-project = Lasīt arī adapterus šīs projekta mapes mapē `adapters`, kā to darītu --adapters-from-project
help-adapters-check = Pārbaudīt adaptera mapi, pirms to lieto vai nodod tālāk: vai tā tiktu ielādēta, kas jāzina tā autoram, un ar --live, vai tā pakalpojumi atbild. Izejas kods 0: tā iztur; 1: ir atradumi; 2: pārbaudi nevarēja veikt
help-adapters-check-dir = Adaptera mape; tās nosaukums ir adaptera id
help-adapters-check-live = Vaicāt arī adaptera pakalpojumiem par taisnstūri, ko nosauc --bbox, un pateikt, ko katrs sniedz. Pieprasījumi paliek pie serveriem, ko manifests deklarē
help-adapters-check-bbox = Dzīvās pārbaudes taisnstūris grādos kā minLon,minLat,maxLon,maxLat. Tam jāatrodas adaptera robežās; pietiek ar 150 m reiz 150 m
help-adapters-check-json = Izdrukāt rezultātu kā JSON
help-adapters-allow-host = Atļaut iebūvētam adapterim vēl vienu serveri: federālajai zemei, kuras pakalpojums pārcēlies uz serveri, ko tās adapteris vēl nenosauc. Ziņojums par noraidīto adresi pasaka, kuru serveri. Serveris tiek pārbaudīts kā manifesta serveris, un freemapper joprojām savienojas tikai ar interneta serveriem
help-adapters-allow-host-adapter = Iebūvētā adaptera id, kā to rāda `adapters list`
help-adapters-allow-host-host = Atļaujamais serveris: DNS nosaukums, nevis adrese, bez porta
help-adapters-disallow-host = Atsaukt serveri, ko allow-host atļāva
help-adapters-disallow-host-adapter = Iebūvētā adaptera id
help-adapters-disallow-host-host = Atsaucamais serveris
help-arg-adapters-from-project = Ielādēt arī adapterus projekta mapes mapē `adapters`. freemapper nosauc katru no tiem kopā ar serveriem, kuriem tas vaicā. Lietojiet to projektam, kuram uzticaties: adapteris liek freemapper vaicāt serveriem, ko tas nosauc

## The summary of a run, for the MCP server and the area page

cli-internal-error = iekšēja kļūda: { $message }
cli-pdf-sources-invalid = pdf_sources jābūt auto, full vai credit-only, nevis { $value }
cli-view-unknown = skats ir no-paths, brown-only, no-north-lines, no:<codes> vai only:<codes>, nevis { $value }
cli-view-empty = sarakstā { $value } nav nosaukts neviens simbola kods
cli-view-bad-code = { $value } nav simbola kods, piemēram, 505 vai 501.6
cli-summary-training-view = Ierakstīti { $map } un { $pdf } (treniņu kartes skats)
cli-summary-kept-training-view = Atstāti treniņu kartes skata { $view } faili; norādiet --force, lai tos atjaunotu
cli-summary-training-view-failed = Treniņu kartes skats neizdevās: { $error }
cli-estimate-disk-unreadable = nevar nolasīt brīvo vietu { $path } diskā ({ $message })
cli-summary-wrote-map = Ierakstīta karte { $path }
cli-summary-kept-map = Atstāta esošā karte { $path } (tajā var būt kartētāja darbs)
cli-summary-map-lacks-templates = Karte vēl neatsaucas uz attēlu veidnēm. Sāciet konveijeru ar force, lai to aizstātu; vecā karte paliek kā .omap.bak.
cli-summary-map-lacks-buildings = Kartē vēl nav ALKIS ēku. Sāciet konveijeru ar force, lai tās pievienotu; tas aizstāj karti, un vecā paliek kā .omap.bak.
cli-summary-map-lacks-canopies = Kartei trūkst ALKIS nojumju un automašīnu nojumju (522), kas tagad ir kartes objekti. Sāciet konveijeru ar force, lai tās pievienotu; tas aizstāj karti, un vecā paliek kā .omap.bak.
cli-summary-map-light = Pamatkarte: savienotas līnijas: { $joined }, kandidātu slāņiem atstāti paslēpti vai pārāk mazi objekti: { $left } (nekas netika pārbīdīts)
cli-summary-map-light-kept = Kandidātu slāņos tagad ir arī objekti vai posmi: { $left }, ko šī versija noņem no pamatkartes (atzīmēti ar freemapper:generalized). Atstātajā kartē tie joprojām ir, tāpēc redzami divreiz. Sāciet konveijeru ar force, lai karti sakārtotu; vecā paliek kā .omap.bak.
cli-summary-map-old-candidates = Karte joprojām norāda uz templates/candidates.omap; šajā palaišanā tas tika aizpildīts ar visiem kandidātiem. Kandidāti tagad ir arī septiņos slāņos, katrs savā failā (templates/candidates-*.omap). Karte tos uzskaita pēc palaišanas ar force (vecā karte paliek kā .omap.bak), vai arī kartētājs pievieno tos OOM (Templates > Open template…).
cli-summary-wrote-ocad = Ierakstīta OCAD karte { $path } (objekti: { $count })
cli-summary-kept-ocad = Atstāta esošā OCAD karte { $path } (tajā var būt kartētāja darbs; force to aizstāj)
cli-summary-ocad-old-candidates = OCAD karte joprojām norāda { $path } kā fona karti; šajā palaišanā tā tika aizpildīta ar visiem kandidātiem. Kandidāti tagad ir arī septiņos slāņos, katrs savā failā (-candidates-*.ocd). OCAD karte tos uzskaita pēc palaišanas ar force (vecā karte paliek kā .ocd.bak), vai arī kartētājs pievieno tos OCAD kā fona kartes.
cli-summary-ocad-left-out = Izlaists no OCAD kartes: objekti { $skipped } un deģenerētas daļas { $dropped }
cli-summary-ocad-symbols = OCAD izvēlieties Symbol > Load Colors and Symbols From… un norādiet oficiālo simbolu kopu{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF skolu kopu 2019
       *[other] {" "}ISSprOM 2019
    }; tā aizstāj vietturu simbolus pēc numura.
cli-summary-ocad-failed = OCAD eksports neizdevās: { $error }
cli-summary-pdf-without-credits = PDF bez avotu norādēm: ODbL un dl-de/by-2-0 prasa nosaukt avotus, kad karte tiek nodota tālāk; skatiet ATTRIBUTION.txt
cli-summary-wrote-pdf = Ierakstīta karte { $path } (bez kandidātiem)
cli-summary-pdf-failed = PDF eksports neizdevās: { $error }
cli-summary-wrote-pdf-candidates = Ierakstīta pārbaudes izdruka { $path } (karte ar visiem kandidātiem, precīzi pēc atrašanās vietas)
cli-summary-pdf-candidates-failed = Pārbaudes izdrukas eksports neizdevās: { $error }
cli-summary-training-failed = Treniņu kartes eksports neizdevās: { $error }
cli-summary-kept-training = Atstāta esošā treniņu karte { $path }, lai palaišana nepārrakstītu izdrukāto vai nodoto karti; dzēsiet to vai iestatiet force, lai to atjaunotu

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = šajā sistēmā nav mājas mapes; lietojiet absolūtu ceļu
mcp-error-relative-path = { $path } ir relatīvs ceļš; lietojiet absolūtu ceļu vai tādu, kas sākas ar ~
mcp-error-home-folder = { $path } ir pati mājas mape; lietojiet tajā esošu mapi, piem., ~/maps/school
mcp-error-no-project = mapē { $path } nav freemapper projekta; vispirms lietojiet create_project
mcp-error-unknown-kind = nezināms veids { $kind }; lietojiet "schoolyard", "sprint" vai "forest"
mcp-error-reading = lasīšana { $path }: { $error }
mcp-error-one-area-source = norādiet tieši vienu no osm_id (no find_place) vai geojson
mcp-error-page-open = šī projekta apgabala izvēles lapa joprojām ir atvērta; palūdziet lietotājam saglabāt apgabalu un lapā nospiest „Gatavs“, tad sāciet konveijeru
mcp-error-no-job = nav darba { $job }; start_pipeline un permits_for_area atgriež šīs sesijas darbu id
mcp-error-job-running = { $job } vēl darbojas mapē { $path }; vienlaikus darbojas viens konveijers, jo projekti dala lejupielāžu kešatmiņu. Sāciet šo, kad project_status ziņo finished vai failed
mcp-error-page-start = nevar startēt lapu: { $error }
mcp-created-project = Izveidots projekts { $path } apgabalam { $area } ({ $kind }, 1:{ $scale }). Tālāk: estimate_download, pēc tam start_pipeline.
mcp-created-project-given-area = Izveidots projekts { $path } dotajam apgabalam ({ $kind }, 1:{ $scale }). Tālāk: estimate_download, pēc tam start_pipeline.
mcp-page-already-open = Šīs mapes lapa jau ir atvērta; tā saglabā iestatījumus, ar kādiem tā tika atvērta.
mcp-page-next = Palūdziet lietotājam lapā izvēlēties apgabalu, nospiest „Saglabāt“ un tad „Gatavs“.
mcp-preview-of = { $layer } priekšskatījums
mcp-permits-next = Atļauju piezīmes tiek veidotas. Aptaujājiet project_status ar šo job_id; kad darbs ir finished, permits_md satur permits.md tekstu. Nododiet to tālāk kā norādi, nevis juridisku padomu.
mcp-tool-check-update = Vai pastāv jaunāks freemapper laidiens: tā versija, izdošanas datums, laidiena piezīmes un lejupielādes izmērs, vai ka šis ir aktuāls. Neko neinstalē.
mcp-tool-create-project = Izveido projekta mapi kartes apgabalam. Norādiet osm_id (no find_place) vai GeoJSON failu. kind ir schoolyard (1:1000, noklusējums), sprint (1:4000) vai forest (1:15000).
mcp-param-create-project-dir = Jaunā projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-param-create-project-geojson = osm_id vietā: GeoJSON fails ar kartes apgabala daudzstūri (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (noklusējums, 1:1000), "sprint" (1:4000) vai "forest" (1:15000).
mcp-param-create-project-osm-id = Kartes apgabala OSM objekts no find_place, piem., "way/992423119".
mcp-param-create-project-part = Ar osm_id: kuru daļu izmantot apgabalam no vairākām atsevišķām daļām (1 ir lielākā).
mcp-param-create-project-scale = Mēroga saucējs: skolas pagalms 1000 (noklusējums); sprints 4000 (noklusējums) vai 5000; mežs 15000 (noklusējums), 10000 vai 7500. Citas vērtības tiek noraidītas; 1:3000 zīmējiet pie 4000, freemapper-print drukājot palielina.
mcp-param-create-project-title = Kartes nosaukums malas informācijai (noklusējums: mapes nosaukums).
mcp-tool-estimate-download = Ko start_pipeline lejupielādēs projektam: flīzes, katra lejupielāde ar izmēru MB, vai tā ir kešatmiņā un vai ietilpst diskā, vēl lejupielādējamie MB, un zem disk brīvā vieta kešatmiņas un projekta diskos ar ziņojumiem par to, kas neietilpst (pastāstiet to lietotājam).
mcp-param-estimate-download-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-tool-find-place = Meklē skolu, parku vai citu vietu pēc nosaukuma (OpenStreetMap Nominatim). Atgriež līdz 5 atbilstībām, vispirms apgabalus; tikai atbilstības ar has_area var būt kartes apgabals. Tukšs saraksts nozīmē, ka nekas nav atrasts.
mcp-param-find-place-query = Skolas vai vietas nosaukums, piem., "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Instalē jaunāko freemapper laidienu pāri šai programmai. Izsauciet to tikai pēc personas piekrišanas, ar confirm true. Jaunā versija stājas spēkā, kad asistents atkārtoti palaiž freemapper serveri.
mcp-param-install-update-confirm = true, kad persona ir piekritusi jaunākās versijas instalēšanai; bez tā nekas netiek instalēts.
mcp-tool-list-layers = Projekta slāņi: karte, septiņi kandidātu slāņi un rastra veidnes, katrs ar savu failu, vai tas jau ir izveidots un kurš solis to veido.
mcp-param-list-layers-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-tool-open-area-selector = Atver pārlūkā lapu, kurā lietotājs uzzīmē kartes apgabalu (taisnstūri vai daudzstūri) vai izvēlas vietu, redz flīzes un lejupielādes izmēru un to saglabā; saglabājot tiek izveidota projekta mape vai mainīts tās apgabals. Atgriež URL, ko dot lietotājam. Lapa darbojas, līdz lietotājs nospiež „Gatavs“; atkārtots izsaukums tai pašai mapei atgriež to pašu URL.
mcp-param-open-area-selector-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~); tiek izveidota, kad lietotājs saglabā.
mcp-param-open-area-selector-kind = Jauna projekta kartes veids: "schoolyard" (noklusējums, 1:1000), "sprint" (1:4000) vai "forest" (1:15000).
mcp-param-open-area-selector-ocad = Palaišanas panelis sākas ar atzīmētu OCAD sākuma projektu. Noklusējums false.
mcp-param-open-area-selector-offline = Palaišanas panelis sākas ar atzīmētu „Tikai kešatmiņa, bez tīkla“. Noklusējums false.
mcp-param-open-area-selector-open-browser = Atvērt lapu arī šī datora noklusējuma pārlūkā. Noklusējums true.
mcp-param-open-area-selector-pdf = Palaišanas panelis sākas ar atzīmētu drukājamo PDF. Noklusējums false.
mcp-param-open-area-selector-pdf-candidates = Palaišanas panelis sākas ar atzīmētu pārbaudes izdruku (visi kandidāti). Noklusējums false.
mcp-param-open-area-selector-pdf-scale-bar = Palaišanas panelis sākas ar atzīmētu mēroga līniju PDF failos. Noklusējums false.
mcp-param-open-area-selector-pdf-sources = Avotu rindas PDF failos, ar kurām panelis sākas: "auto" (noklusējums), "full" vai "credit-only", kā start_pipeline.
mcp-param-open-area-selector-permits = Palaišanas panelis sākas ar atzīmētām atļauju piezīmēm. Noklusējums false.
mcp-param-open-area-selector-scale = Jauna projekta mēroga saucējs: skolas pagalms 1000 (noklusējums); sprints 4000 (noklusējums) vai 5000; mežs 15000 (noklusējums), 10000 vai 7500. Citas vērtības tiek noraidītas.
mcp-param-open-area-selector-title = Jauna projekta kartes nosaukums.
mcp-param-open-area-selector-training = Palaišanas panelis sākas ar atzīmētu treniņu karti. Noklusējums false.
mcp-tool-preview-layer = PNG priekšskatījums (ne vairāk kā 800 px) slānim: map (kartes objektu attēls, ko ierakstīja pēdējā palaišana, bez kandidātiem) vai rastra veidne: orthophoto, hillshade, slope, relief, ndsm vai k5 Berlīnē.
mcp-param-preview-layer-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" vai "k5" (tikai Berlīne).
mcp-tool-project-status = start_pipeline vai permits_for_area darba stāvoklis: running, finished vai failed, pēdējās norises rindas, brīdinājumi, ierakstītie faili un rezultāts vai kļūda. Pabeigts permits_for_area darbs satur arī permits.md tekstu kā permits_md.
mcp-param-project-status-job-id = Darba id no start_pipeline vai permits_for_area, piem., "job-1".
mcp-tool-permits-for-area = Izveido projekta atļauju piezīmes: kartes apgabala administratīvās vienības, tajā esošās aizsargājamās teritorijas un kas tur apsaimnieko zemi, ar iestādēm, kam vaicāt; norāde, nevis juridisks padoms. Ieraksta permits.md un permits.json mapē. Darbojas fonā un atgriež job_id; aptaujājiet project_status, kura atbilde satur permits.md tekstu kā permits_md, kad darbs ir pabeigts. Vienlaikus darbojas viens darbs.
mcp-param-permits-for-area-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-param-permits-for-area-offline = Lasīt tikai datus, ko projektā saglabāja agrāka palaišana, bez piekļuves tīklam. Noklusējums false.
mcp-tool-start-pipeline = Lejupielādē datus, aprēķina veidnes un ieraksta sākuma projektu (fetch, derive, export). Darbojas fonā un atgriež job_id; aptaujājiet project_status. Vienlaikus darbojas viens konveijers. Tas atstāj esošu karti, ja force nav true.
mcp-param-start-pipeline-dir = Projekta mape (absolūts ceļš vai tāds, kas sākas ar ~).
mcp-param-start-pipeline-force = Aizstāt esošo kartes failu un vecā saglabāt kā .omap.bak. Noklusējums false.
mcp-param-start-pipeline-ocad = Ierakstīt arī OCAD sākumpunktu mapē ocad/ (OCAD 12 formāts, lietotājiem, kas rediģē OCAD). Noklusējums false.
mcp-param-start-pipeline-offline = Izmantot tikai lejupielāžu kešatmiņu, bez piekļuves tīklam. Noklusējums false.
mcp-param-start-pipeline-pdf = Ierakstīt arī <name>.pdf, drukai gatavo karti (tikai kartes objekti, bez kandidātiem). Noklusējums false.
mcp-param-start-pipeline-pdf-candidates = Ierakstīt arī <name>-candidates.pdf, pārbaudes izdruku: karte ar visiem kandidātiem, precīzi pēc atrašanās vietas (pārbaudiet tos apvidū). Noklusējums false.
mcp-param-start-pipeline-pdf-scale-bar = Zīmēt mēroga līniju PDF failos. Noklusējums false (Vācijas kartēm parasti tādas nav; mēroga skaitlis paliek nosaukuma blokā).
mcp-param-start-pipeline-pdf-sources = Avotu rindas PDF failos: "auto" (noklusējums; pilns saraksts, saīsināts tikai tad, ja aizņem vairāk nekā 5 rindas), "full" (nekad nesaīsina) vai "credit-only" (tikai freemapper norādes rinda; ODbL un dl-de/by-2-0 tad prasa nosaukt avotus, kad karte tiek nodota tālāk). ATTRIBUTION.txt vienmēr uzskaita visus avotus.
mcp-param-start-pipeline-permits = Ierakstīt arī atļauju piezīmes (permits.md un permits.json) pēc kartes: administratīvās vienības, aizsargājamās teritorijas un kas apsaimnieko zemi. Kļūme tur ir brīdinājums, nevis neizdevies darbs. Noklusējums false.
mcp-param-start-pipeline-training = Ierakstīt arī treniņu karti <name>-training.omap un tās PDF <name>-training.pdf: karte un visi kandidāti, sakopti, lai tos drukātu tādus, kādi tie ir; ar ocad arī ocad/<name>-training.ocd. Esoša treniņu karte tiek atstāta, ja force nav true. Noklusējums false.
mcp-param-start-pipeline-training-views = Ar training: ierakstīt arī treniņu karti bez tā, ko skats izlaiž, viens failu kopums katram ierakstam: "no-paths", "brown-only", "no-north-lines" vai saraksts, piemēram, "no:505,506" vai "only:101,102", apvienots ar +. Faili ir <name>-training-<view>.omap un tā PDF.
mcp-instructions = freemapper veido OpenOrienteering Mapper sākuma projektus orientēšanās kartēm (skolas pagalms 1:1000, sprints 1:4000, mežs 1:15000) no oficiālajiem ģeodatiem: { $states }, un no OSM. Darbplūsma: find_place (meklēt skolu vai apgabalu pēc nosaukuma; izvēlēties rezultātu ar has_area) → create_project (jauna mape, absolūts ceļš, ar osm_id) → estimate_download (pastāstīt lietotājam, cik daudz tiks lejupielādēts) → start_pipeline (atgriež darba id; lejupielāde var ilgt minūtes) → project_status (aptaujāt ik pēc dažām sekundēm, līdz stāvoklis ir finished vai failed) → list_layers un preview_layer (parādīt lietotājam veidnes). Tad lietotājs atver kartes failu programmā OpenOrienteering Mapper. Ja neviena vieta nederēja vai lietotājs vēlas apgabalu uzzīmēt, open_area_selector atver pārlūkā lapu, kurā lietotājs uzzīmē vai izvēlas apgabalu un to saglabā (tas izveido projektu); turpiniet ar estimate_download, kad lietotājs pasaka, ka saglabāts. Ceļiem jābūt absolūtiem vai jāsākas ar ~. Vienlaikus darbojas viens konveijers. permits_for_area izveido projekta atļauju piezīmes (administratīvās vienības, aizsargājamās teritorijas, kas apsaimnieko zemi; norāde, nevis juridisks padoms) kā darbu, tāpat kā start_pipeline. check_update pasaka, vai pastāv jaunāks freemapper; install_update to instalē, tikai ar confirm: true pēc personas piekrišanas, un asistents tad atkārtoti palaiž freemapper serveri.

cli-contour-interval-saved = Horizontāļu intervāls saglabāts failā freemapper.toml: { $interval } m; derive velk horizontāles ar to
cli-contour-interval-auto = Horizontāļu intervāls failā freemapper.toml: automātisks (kartes veida noklusējums; meža karte iegūst 2,5 m līdzenumā un 10 m stāvā apvidū)
help-arg-contour-interval = Horizontāļu intervāls metros, piem., `2.5`, vai `auto` kartes veida noklusējumam (meža karte: 5 m, 2,5 m, kur līdzens, 10 m, kur stāvs). Saglabāts failā freemapper.toml; derive velk horizontāles ar to
cli-content-saved = Satura iestatījums saglabāts failā freemapper.toml: { $summary }
cli-content-invalid = Nevar nolasīt satura iestatījumu: { $detail }
help-arg-content = Kas nonāk tieši kartē: `safe` atstāj katru objektu tur, kur to ieliek kartes veidošana, `balanced` (noklusējums) ieliek kartē kadastra un reģistru objektus, `full` ieliek kartē visu. Izvēle tiek saglabāta failā freemapper.toml
help-arg-content-set = Nosūtīt objektu grupu uz `map`, `candidates` vai `off`, kā `GROUP=PLACE`, piem., `walls=map`; var atkārtot. Grupas: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Saglabāts failā freemapper.toml
mcp-param-create-project-content = Kas nonāk tieši kartē: "safe", "balanced" (noklusējums) vai "full". Saglabāts projektā. balanced ieliek kartē kadastra un reģistru objektus, full ieliek kartē visu.
mcp-param-create-project-content-set = Grupas, kas nosūtītas citur, nekā norāda profils, kā "group=place", kur place ir map, candidates vai off, piem., "walls=map". Grupas: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Horizontāļu intervāls metros, piem., 2.5; 0 atgriež automātisko izvēli (kartes veida noklusējums; meža karte iegūst 2,5 m līdzenumā un 10 m stāvā apvidū). Saglabāts projektā; nākamais derive velk horizontāles ar to.
mcp-param-start-pipeline-content = Mainīt pirms palaišanas, kas nonāk tieši kartē: "safe", "balanced" vai "full". Saglabāts projektā. Noklusējums: projekta iestatījums, citādi balanced.
mcp-param-start-pipeline-content-set = Grupas, kas nosūtītas citur, nekā norāda profils, kā "group=place", kur place ir map, candidates vai off, piem., "fences=candidates". Saglabāts projektā.
mcp-param-start-pipeline-contour-interval = Horizontāļu intervāls metros, piem., 2.5; 0 atgriež automātisko izvēli (kartes veida noklusējums; meža karte iegūst 2,5 m līdzenumā un 10 m stāvā apvidū). Saglabāts projektā; nākamais derive velk horizontāles ar to.
mcp-error-content = Nevar nolasīt satura iestatījumu: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Ierakstīt projekta atļauju piezīmes (permits.md un permits.json): administratīvās vienības, aizsargājamās teritorijas un apsaimniekotāji, ko skar kartes apgabals, un kam tas parasti nozīmē jāvaicā. Tas vaicā BfN un BKG un, ja kartes apgabala federālajai zemei ir savi atvērti pakalpojumi, arī tiem caur sava adaptera serveriem (saraksts rokasgrāmatas 12. nodaļā); Ziemeļreinā-Vestfālenē tas pirmo reizi lejupielādē apmēram 32 MB, Brandenburgā apmēram 4 MB, vēlāk tikai jaunākus failus. Tas lasa jau kešoto OSM izvilkumu; --offline izmanto agrāk lejupielādēto
help-permits-dir = Projekta mape
help-permits-offline = Izmantot tikai agrāk lejupielādētos datus, bez piekļuves tīklam
help-run-permits = Ierakstīt arī atļauju piezīmes (permits.md un permits.json) pēc kartes; vaicā BfN, BKG un federālās zemes atvērtajiem pakalpojumiem (rokasgrāmata, 12. nodaļa). Kļūme tur ir brīdinājums, un karte paliek
cli-wrote-permits = Ierakstītas atļauju piezīmes { $path }
cli-summary-wrote-permits = Ierakstītas atļauju piezīmes { $path }
cli-permits-failed = atļauju piezīmes nevarēja ierakstīt: { $error }
cli-permits-source-left-out = atļauju piezīmes izlaiž šo: { $source }: { $reason }
cli-permits-not-read = tas netika nolasīts
error-permits-stored = { $path }: { $message }; vēlreiz lejupielādējiet atļauju piezīmes tiešsaistē
error-permits-nothing-read = nevienu atļauju piezīmju avotu nevarēja nolasīt ({ $missing }); vispirms lejupielādējiet tos tiešsaistē
error-permits-missing-not-read = { $source }: nenolasīts

permits-title = Atļauju piezīmes
permits-disclaimer = Norāde, nevis juridisks padoms. Lēmumu pieņem kompetentās iestādes. Datu datumu un precizitāti skatiet tabulā sadaļā „Avoti un datu datums“; teritorijas var trūkt vai būt novecojušas (BfN: apvienots reizi gadā, „Nicht für Planungszwecke geeignet.“, tas ir, nav piemērots plānošanai).
permits-disclaimer-regional = Norāde, nevis juridisks padoms. Lēmumu pieņem kompetentās iestādes. Datu datumu un precizitāti skatiet tabulā sadaļā „Avoti un datu datums“; teritorijas var trūkt vai būt novecojušas.
permits-bfn-merge = BfN reizi gadā apvieno federālo zemju datus un atzīmē tos kā nepiemērotus plānošanai. Lēmumu pieņem federālās zemes iestādes.
permits-created = Sastādīts { $date } kartes apgabalam: { $kind ->
        [schoolyard] skolas pagalma karte
        [forest] meža karte
       *[other] sprinta karte
    } ({ $area } ha).
permits-section-admin = Administratīvās vienības
permits-section-protected = Aizsargājamās teritorijas (BfN)
permits-section-osm-protected = Citas aizsargājamās teritorijas no OpenStreetMap
permits-section-operators = Apsaimniekotāji un īpašnieki pēc OpenStreetMap
permits-section-hints = Norādes
permits-section-sources = Avoti un datu datums
permits-none-admin = VG250 nenosauc nevienu administratīvo vienību kartes apgabalam.
permits-none-protected = Pēc BfN datiem kartes apgabalā nav aizsargājamu teritoriju.
permits-none-osm-protected = OpenStreetMap nenosauc kartes apgabalā nevienu aizsargājamu teritoriju, ko BfN jau neuzskaita.
permits-none-operators = OpenStreetMap nenosauc ne apsaimniekotāju, ne īpašnieku kartes apgabalam.
permits-none-hints = Norāžu nav.
permits-section-not-read = Dati no { $source } netika nolasīti, tāpēc šī sadaļa ir tukša, kaut gan ieraksti var pastāvēt.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] BKG administratīvās teritorijas 1:250 000 (VG250)
        [bfn] BfN aizsargājamās teritorijas Vācijā
       *[osm] OpenStreetMap no izvilkuma, ko freemapper glabā savā kešatmiņā
    }
permits-level = { $level ->
        [land] Federālā zeme
        [kreis] Apriņķis (Kreis)
        [verwaltungsgemeinschaft] Amt vai Verwaltungsgemeinschaft (pārvalžu apvienība)
       *[gemeinde] Pašvaldība (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Skarts malā (mazāk nekā 0,5% no kartes apgabala):
permits-at-edge-generalised = VG250 ir vispārināts līdz mērogam 1:250 000. Vienība, kas tikai pieskaras malai, var būt tā sekas.
permits-kind = { $kind ->
        [nationalpark] Nacionālais parks
        [nationales-naturmonument] Nacionālais dabas piemineklis
        [naturschutzgebiet] Dabas liegums (Naturschutzgebiet)
        [ffh] FFH teritorija (Natura 2000)
        [vogelschutzgebiet] Putnu aizsardzības teritorija (Natura 2000)
        [biosphaerenreservat] Biosfēras rezervāts
        [biosphaerenreservat-zone] Biosfēras rezervāta zona
        [landschaftsschutzgebiet] Ainavu aizsardzības teritorija (Landschaftsschutzgebiet)
       *[naturpark] Dabas parks
    }
permits-protected-line = { $kind } „{ $name }“, { $share }
permits-protected-date = Noteikts { $date }
permits-protected-title = Tiesiskais pamats: { $title }
permits-protected-link = Tiesību akta teksts: { $url }
permits-protected-site-link = Teritorijas lapa (Natura 2000): { $url }
permits-protected-year = Datu gads BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = aizsardzības klase { $class }
permits-quoted = „{ $text }“
permits-osm-unnamed = bez nosaukuma
permits-osm-operator = Apsaimniekotājs pēc OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap teritorijas, kas ietver visu kartes apgabalu bez stūra tā iekšienē, šeit var trūkt, tāpat kā aizsargājamās teritorijas, kas OSM kartētas kā relācija ar `type=boundary`. BfN un VG250 daļas tas neskar.
permits-osm-enclosing-gap-short = Šeit var trūkt OpenStreetMap laukumu, kas ieskauj visu kartes apgabalu, bet kuru stūris neatrodas tajā.
permits-operator-line = { $name } ({ $roles ->
        [operator] apsaimniekotājs
        [owner] īpašnieks
       *[both] apsaimniekotājs un īpašnieks
    }): { $objects }
permits-column-source = Avots
permits-column-state = Stāvoklis
permits-column-credit = Norāde un licence
permits-state-read = nolasīts, lejupielādēts { $date }
permits-state-read-osm = nolasīts, OSM izvilkums no { $date }
permits-state-unreachable = nav sasniedzams ({ $reason })
permits-state-not-read = nenolasīts
permits-no-credit = netiek izmantots
permits-hint-city-state = Kartes apgabals atrodas Berlīnē vai Hamburgā. Tur parasti atbildīga ir Bezirksamt (rajona pārvalde) vai federālās zemes iestāde. Šīs piezīmes Bezirk vēl nenosaka.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Aizsargājamās teritorijas skar zemāko dabas aizsardzības iestādi (untere Naturschutzbehörde). Šeit tā ir pati pilsēta { $kreis }.
        [no] Aizsargājamās teritorijas skar zemāko dabas aizsardzības iestādi (untere Naturschutzbehörde), parasti { $kreis }.
       *[unknown] Aizsargājamās teritorijas skar kompetento zemāko dabas aizsardzības iestādi (untere Naturschutzbehörde), parasti Landkreis vai kreisfreie Stadt (apriņķis vai apriņķa pilsēta).
    }
permits-hint-national-park = Nacionālajam parkam ir savi noteikumi. Vērsieties pie nacionālā parka pārvaldes savlaicīgi.
permits-hint-nsg-paths = Dabas liegumā parasti jāturas pie takām. Skrējienam ārpus takām vaicājiet dabas aizsardzības iestādei.
permits-hint-natura2000 = FFH teritorija vai putnu aizsardzības teritorija (Natura 2000) skar kartes apgabalu. Pasākumam dabas aizsardzības iestāde var prasīt ietekmes novērtējumu.
permits-hint-protected-areas-pl = Kartes apgabalā atrodas aizsargājamas teritorijas vai dabas aizsardzības objekti. Nacionālā parka, ainavu parka vai dabas rezervāta gadījumā parasti jāvēršas parka vai rezervāta administrācijā; par pārējām teritorijām un objektiem, kas ir atbildīgais, parasti var pateikt vojevodistes reģionālā vides aizsardzības direkcija (RDOŚ). Jautājiet laikus, ja tur plānojat pasākumu.
permits-hint-order-office = { $amt ->
        [none] Ielām, laukumiem un citai publiskai zemei parasti atbildīgs ir Gemeinde { $gemeinde } Ordnungsamt (sabiedriskās kārtības birojs).
       *[other] Ielām, laukumiem un citai publiskai zemei parasti atbildīgs ir Ordnungsamt (sabiedriskās kārtības birojs): { $amt }, Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Parka vai dārza apsaimniekotājs kartes apgabalā pēc OpenStreetMap: { $operator }. Pirms lietošanas vaicājiet viņam.
permits-hint-school-operator = Skolas vai iestādes apsaimniekotājs pēc OpenStreetMap: { $operator }. Par tās teritoriju vaicājiet skolas vadītājam vai šai struktūrai.
permits-section-districts = Bezirke (rajoni, { $source })
permits-section-green-spaces = Zaļās zonas ({ $source })
permits-section-owners = Federālās zemes un Bund zeme ({ $source })
permits-section-monuments = Pieminekļi ({ $source })
permits-section-protected-objects = Dabas pieminekļi un citi aizsargājamie objekti ({ $source })
permits-none-districts = Pēc { $source } datiem kartes apgabals neatrodas nevienā Bezirk.
permits-none-green-spaces = Pēc { $source } datiem kartes apgabalā nav publisku zaļo zonu.
permits-none-owners = Pēc { $source } datiem kartes apgabalā nav federālās zemes vai Bund zemes gabalu.
permits-none-monuments = Pēc { $source } datiem kartes apgabalā nav pieminekļu.
permits-none-protected-objects = Pēc { $source } datiem kartes apgabalā nav neviena no tajā esošajiem aizsargājamajiem objektiem.
permits-none-sampled = Pārbaudītajos punktos { $source } šeit neko nenosauc.
permits-gaps = { $source } dati nav pilnīgi: { $gaps }.
permits-sampled = { $source } atbild tikai par atsevišķiem punktiem. freemapper vaicāja punktu režģī pāri kartei: daļas ir aplēses, un var trūkt tas, kas atrodas starp punktiem.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = statuss: { $dedication }
permits-playgrounds-none = Kartes apgabalā nav publisku rotaļu laukumu.
permits-playgrounds-one = Kartes apgabalā ir viens publisks rotaļu laukums. Tas atsevišķi nav uzskaitīts.
permits-playgrounds-many = Publisku rotaļu laukumu kartes apgabalā: { $count }. Tie nav uzskaitīti pa vienam.
permits-owner-other = Citi publiskie īpašnieki
permits-owners-rest = Pārējai teritorijai dati īpašnieku nenosauc.
permits-monuments-total = Pieminekļi pilnībā vai daļēji kartes apgabalā: { $count }.{ $links ->
        [yes] {" "}Saites ved uz to ierakstiem pieminekļu sarakstā.
       *[no] {""}
    }
permits-monuments-links-left = Vēl ieraksti, kas šeit tikai saskaitīti, bez saites: { $count }.
permits-object-kind = { $kind ->
        [natural-monument] Dabas piemineklis (Naturdenkmal)
        [protected-landscape-part] Aizsargājams ainavas elements (geschützter Landschaftsbestandteil)
        [biotope] Likumā aizsargāts biotops (geschütztes Biotop)
        [national-park] Nacionālais parks (park narodowy)
        [park-buffer-zone] Parka aizsargjosla (otulina)
        [landscape-park] Ainavu parks (park krajobrazowy)
        [nature-reserve] Dabas rezervāts (rezerwat przyrody)
        [protected-landscape-area] Aizsargājamā ainavu teritorija (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000 teritorija biotopiem (SOO)
        [natura2000-birds] Natura 2000 teritorija putniem (OSO)
        [ecological-site] Ekoloģiskā teritorija (użytek ekologiczny)
        [landscape-complex] Dabas un ainavu komplekss (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentācijas vieta (stanowisko dokumentacyjne)
        [natural-monument-pl] Dabas piemineklis (pomnik przyrody)
       *[special-rules] Teritorija ar īpašiem noteikumiem
    }
permits-object-count = objekti: { $count }
permits-object-citations = Tiesiskie pamati:
permits-source-note = { $source }: „{ $note }“
permits-hint-district-office = { $offices ->
        [hamburg] Ielām, laukumiem un zaļajām zonām Bezirk { $district } parasti atbildīgs ir Bezirksamt { $district }, tur Fachamt Management des öffentlichen Raumes (publiskās telpas pārvaldības nodaļa).
       *[berlin] Ielām, laukumiem un zaļajām zonām Bezirk { $district } parasti atbildīgi ir Bezirksamt { $district } Ordnungsamt (sabiedriskās kārtības birojs) un Straßen- und Grünflächenamt (ielu un zaļo zonu birojs).
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Par pieminekļiem kartes apgabalā atbildīgs ir Denkmalschutzamt Hamburg (pieminekļu aizsardzības birojs).
       *[berlin] Par pieminekļiem Bezirk { $district } atbildīga ir Bezirksamt { $district } zemākā pieminekļu aizsardzības iestāde (untere Denkmalschutzbehörde).
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Vai kartes apgabalā ir aizsargājami pieminekļi, šeit pieejamie dati droši pateikt nevar.
       *[yes] Kartes apgabalā ir aizsargājami pieminekļi.
    } Vai skrējiens tos skar, piemēram, kontrolpunkts pie pieminekļa vai posms cauri aizsargājamai vietai, lemj zemākā pieminekļu aizsardzības iestāde (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Šeit tā ir pati pilsēta { $kreis }.
        [no] , parasti { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; dažās federālajās zemēs tā ir Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(parasti Landkreis vai kreisfreie Stadt, dažās federālajās zemēs Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Dabas pieminekļi, aizsargājami ainavas elementi un aizsargājami biotopi kartē ir zemākās dabas aizsardzības iestādes (untere Naturschutzbehörde) kompetencē. Šeit tā ir pati pilsēta { $kreis }.
        [no] Dabas pieminekļi, aizsargājami ainavas elementi un aizsargājami biotopi kartē ir zemākās dabas aizsardzības iestādes (untere Naturschutzbehörde) kompetencē, parasti { $kreis }.
       *[unknown] Dabas pieminekļi, aizsargājami ainavas elementi un aizsargājami biotopi kartē ir zemākās dabas aizsardzības iestādes (untere Naturschutzbehörde) kompetencē, parasti Landkreis vai kreisfreie Stadt.
    }
permits-hint-biotope = Likumā aizsargāti biotopi atrodas kartes apgabalā{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Aizliegts viss, kas tos var iznīcināt vai būtiski kaitēt (§ 30 BNatSchG). Novietojiet kontrolpunktus un posmus ārpus tiem vai vaicājiet zemākajai dabas aizsardzības iestādei (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Dabas pieminekļiem un aizsargājamiem ainavas elementiem Bezirk { $district } parasti atbildīgs ir Bezirksamt { $district }.
       *[berlin] Dabas pieminekļiem un aizsargājamiem ainavas elementiem Bezirk { $district } atbildīga ir Bezirksamt { $district } zemākā dabas aizsardzības iestāde (untere Naturschutzbehörde).
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Pēc federālās zemes atvērtajiem datiem { $share } kartes apgabala pieder citiem publiskajiem īpašniekiem.
       *[no] Pēc federālās zemes atvērtajiem datiem { $share } kartes apgabala pieder īpašniekam „{ $owner }“.
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Pārējās teritorijas īpašniekus no atvērtajiem datiem noskaidrot nevar. Ar likumīgu interesi (piemēram, organizators) varat vaicāt zemesgrāmatu birojam (Grundbuchamt) pēc izraksta; publiskai zemei palīdz kompetentais Ordnungsamt vai parku pārvalde.
       *[no] Šo teritoriju īpašniekus no atvērtajiem datiem noskaidrot nevar. Ar likumīgu interesi (piemēram, organizators) varat vaicāt zemesgrāmatu birojam (Grundbuchamt) pēc izraksta; publiskai zemei palīdz kompetentais Ordnungsamt vai parku pārvalde.
    }
permits-section-forest-units = Meža pārvalde ({ $source })
permits-section-state-forest = Valsts mežs ({ $source })
permits-none-forest-units = Pēc { $source } datiem kartes apgabals neatrodas nevienas mežniecības (Forstamt) teritorijā.
permits-section-admin-units = Administratīvās vienības ({ $source })
permits-none-admin-units = Saskaņā ar { $source } kartes teritorija neatrodas nevienā reģistra administratīvajā vienībā.
permits-section-register-monuments = Nacionālā reģistra pieminekļi ({ $source })
permits-none-register-monuments = Saskaņā ar { $source } taisnstūrī ap kartes apgabalu nav neviena reģistrēta pieminekļa.
permits-register-monuments-total = Reģistrēti pieminekļi taisnstūrī ap kartes apgabalu: { $count }. Reģistrs nenorāda atrašanās vietu, tāpēc šīs piezīmes nevar pateikt, kur tie atrodas; tas jāpārbauda kartē.
permits-register-monuments-left = Vēl { $count } šeit tikai saskaitīti, bez nosaukumiem.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Pēc { $source } datiem kartes apgabalā nav valsts meža.
permits-forest-units-reach = Mežniecība atbild par mežu savā teritorijā. Tās teritorija var ietvert arī pilsētas un laukus.
permits-state-forest-office = atbildīga: { $office }
permits-hint-forest-office = { $units ->
        [none] Mežniecība, kas atbild par mežu kartes apgabalā: { $office } ({ $administration }). Vērsieties tur savlaicīgi, ja plānojat pasākumu mežā.
       *[other] Mežniecība, kas atbild par mežu kartes apgabalā: { $office } ({ $administration }), { $units }. Vērsieties tur savlaicīgi, ja plānojat pasākumu mežā.
    }
permits-hint-forest-owner = { $office ->
        [known] Pasākumam mežā vajadzīga meža īpašnieku piekrišana. Mežniecība parasti var pateikt, kam pieder mežs. Vairākās federālajās zemēs organizēts pasākums mežā jāpiesaka meža iestādei vai tā jāatļauj.
       *[unknown] Pasākumam mežā vajadzīga meža īpašnieku piekrišana. Vairākās federālajās zemēs organizēts pasākums mežā jāpiesaka meža iestādei vai tā jāatļauj. Šīs piezīmes nenosauc šeit atbildīgo mežniecību; to var pateikt federālās zemes meža pārvalde vai zemākā meža iestāde.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Pasākumam mežā parasti ir vajadzīga meža īpašnieka piekrišana. Iepriekš minētā mežniecība (Nadleśnictwo) parasti var pateikt, kas mežu kopj. Mežs, kas nepieder Valsts mežiem, piemēram, privātais mežs, šo piezīmju datos nav. Mežu, kas nepieder valstij, piemēram, privātu vai pašvaldības mežu, meža apsaimniekošanas jomā uzrauga starosts (starosta) (Mežu likums, 5. panta 1. daļas 2. punkts); jautājiet tur birojā (starostwo powiatowe).
       *[unknown] Pasākumam mežā parasti ir vajadzīga meža īpašnieka piekrišana. Šīs piezīmes kartes apgabalam nenosauc nevienu Valsts mežu mežniecību; dati aptver tikai Valsts mežus, tāpēc mežs var piederēt kādam citam, piemēram, privātam īpašniekam. Īpašnieks jānoskaidro atsevišķi. Mežu, kas nepieder valstij, piemēram, privātu vai pašvaldības mežu, meža apsaimniekošanas jomā uzrauga starosts (starosta) (Mežu likums, 5. panta 1. daļas 2. punkts); jautājiet tur birojā (starostwo powiatowe).
    }
permits-hint-monument-register-pl = Taisnstūrī ap kartes apgabalu atrodas nacionālā reģistra pieminekļi. To atrašanās vietu datos nav, tāpēc kartē pārbaudiet, kur tie ir. Saskaņā ar Pieminekļu aizsardzības likuma (ustawa o ochronie zabytków) 36. panta 1. daļu darbiem reģistrētā piemineklī, būvdarbiem pieminekļa apkārtnē un citām darbībām, kas varētu mainīt tā būtību vai izskatu, vajadzīga vojevodistes pieminekļu konservatora (Wojewódzki Konserwator Zabytków) atļauja. Likums neminē skrējienu vai kontrolpunktu kā tādu; jautājiet konservatora birojā, ja pasākums skar pieminekli vai tā apkārtni.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Kartes teritorijā atrodas dabas piemineklis (pomnik przyrody), ekoloģiskā teritorija (użytek ekologiczny), dabas un ainavu komplekss (zespół przyrodniczo-krajobrazowy) vai dokumentācijas vieta (stanowisko dokumentacyjne). Tos nodibina gminas dome ar lēmumu, kurā norādīts, kas uzrauga objektu un kādi aizliegumi ir spēkā (Dabas aizsardzības likuma 44. pants). Pats likums šeit pasākumam neparedz nekādu atļauju. Lūdziet lēmumu un uzraudzības personu tās gminas iestādē, kuras teritorijā objekts atrodas.
       *[other] Kartes teritorijā atrodas dabas piemineklis (pomnik przyrody), ekoloģiskā teritorija (użytek ekologiczny), dabas un ainavu komplekss (zespół przyrodniczo-krajobrazowy) vai dokumentācijas vieta (stanowisko dokumentacyjne). Tos nodibina gminas dome ar lēmumu, kurā norādīts, kas uzrauga objektu un kādi aizliegumi ir spēkā (Dabas aizsardzības likuma 44. pants). Pats likums šeit pasākumam neparedz nekādu atļauju. Lūdziet lēmumu un uzraudzības personu tās gminas ({ $gmina }) iestādē, kuras teritorijā objekts atrodas.
    }
permits-hint-gmina-roads-pl = Pasākumam, kas traucē satiksmi vai īpašā veidā izmanto ielu, vajadzīga atļauja saskaņā ar Ceļu satiksmes likuma 65. pantu; pieteikumu iesniedz vismaz 30 dienas iepriekš. To izsniedz iestāde, kas ceļā pārvalda satiksmi: starosts powiata un gminas ceļiem, mērs pilsētā ar powiata tiesībām, vojevodistes maršals vai GDDKiA pārējiem ceļiem (10. un 65b. pants). Gminas ceļa pārvaldītājs ir gminas ({ $gmina }) wójt, burmistrz vai prezydent miasta; pārvaldītājs lemj par ceļa joslas aizņemšanu (Publisko ceļu likuma 19. un 40. pants). Šīs piezīmes nerāda ceļu kategoriju.
permits-hint-gmina-land-pl = Ja kartes teritorijas zeme pieder gminai, piemēram, pilsētas parks, laukums, skolas teritorija vai mežs, to pārvalda wójt, burmistrz vai prezydent miasta, kas pārstāv gminu (Gminas pašvaldības likuma 30. un 31. pants). Pirms izmantošanas lūdziet piekrišanu gminas ({ $gmina }) iestādē. Šīs piezīmes nerāda, kam zeme pieder.
permits-section-water-protection = Ūdens aizsardzības zonas ({ $source })
permits-section-forest-functions = Meža funkcijas un aizsargājamais mežs ({ $source })
permits-none-water-protection = Pēc { $source } datiem kartes apgabalā nav ūdens aizsardzības zonu.
permits-none-forest-functions = Pēc { $source } datiem kartes apgabalā nav atpūtas meža, aizsargmeža, dabiskā meža un slēgtas meža teritorijas.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Atpūtas mežs (Erholungswald)
        [protected] Aizsargājamais mežs (Bannwald vai Schutzwald)
        [natural] Dabiskais mežs vai stingrais rezervāts (Naturwald, Totalreservat)
       *[closed] Slēgta meža teritorija
    }
permits-hint-water-protection = { $inner ->
        [yes] Ūdens aizsardzības zona „{ $name }“ atrodas kartes apgabalā: { $zone }. I un II zonā pasākumi parasti ir izslēgti. Vaicājiet zemākajai ūdens iestādei (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] Ūdens aizsardzības zona „{ $name }“ atrodas kartes apgabalā: { $zone }. Pasākumi tur parasti ir iespējami; zemākā ūdens iestāde (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) var pateikt, ko atļauj noteikumi.
       *[unknown] Ūdens aizsardzības zona „{ $name }“ atrodas kartes apgabalā. Dati nepasaka, kura zona tā ir. I un II zonā pasākumi parasti ir izslēgti; vaicājiet zemākajai ūdens iestādei (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Atpūtas mežs (Erholungswald) atrodas kartes apgabalā{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pasākumiem tajā vaicājiet meža iestādei.
        [protected] Aizsargājamais mežs (Bannwald vai Schutzwald) atrodas kartes apgabalā{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pirms plānojat pasākumu tajā, vaicājiet meža iestādei.
        [natural] Dabiskais mežs vai stingrais rezervāts atrodas kartes apgabalā{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Iziet no takām tur bieži ir aizliegts; nenovietojiet tajā kontrolpunktus, nevaicājot meža iestādei.
       *[closed] Meža teritorija, kurā aizliegts ieiet, atrodas kartes apgabalā{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Neiekļaujiet to distancē.
    }
permits-section-roads = Klasificētie ceļi pēc OpenStreetMap
permits-none-roads = Pēc OpenStreetMap kartes apgabalā neiet neviena Autobahn, Bundes-, Landes-, Staats- vai Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (automaģistrāle)
        [bundesstrasse] Bundesstraße (federālais ceļš)
        [landesstrasse] Landesstraße vai Staatsstraße (federālās zemes ceļš)
       *[kreisstrasse] Kreisstraße (apriņķa ceļš)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } iet cauri kartes apgabalam. Distanci drīkst vest pāri tai tikai pa tiltiem vai caur apakšizejām.
        [bundesstrasse] Bundesstraße { $refs } iet cauri kartes apgabalam. Slēgšanu vai šķērsošanu ar uzraugiem pasūta ceļu satiksmes iestāde (Straßenverkehrsbehörde); ceļu parasti uztur Bund ar federālās zemes ceļu pārvaldes starpniecību, lielākās pilsētās bieži pati pilsēta.
        [landesstrasse] Landes- vai Staatsstraße { $refs } iet cauri kartes apgabalam. Slēgšanu vai šķērsošanu ar uzraugiem pasūta ceļu satiksmes iestāde (Straßenverkehrsbehörde); ceļu parasti uztur federālā zeme, lielākās pilsētās bieži pati pilsēta.
       *[kreisstrasse] Kreisstraße { $refs } iet cauri kartes apgabalam. Slēgšanu vai šķērsošanu ar uzraugiem pasūta ceļu satiksmes iestāde (Straßenverkehrsbehörde); ceļu parasti uztur Kreis, lielākās pilsētās bieži pati pilsēta.
    }
permits-hint-state-forest = { $office ->
        [none] Pēc federālās zemes atvērtajiem datiem { $share } kartes apgabala ir valsts mežs ar īpašnieku „{ $owner }“.
       *[other] Pēc federālās zemes atvērtajiem datiem { $share } kartes apgabala ir valsts mežs ar īpašnieku „{ $owner }“. Atbildīga: { $office }.
    }
