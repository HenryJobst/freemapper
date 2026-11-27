# Program texts of freemapper, Slovenian (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } ni jezik, v katerem ima freemapper besedila; na voljo so: { $available }

## init

cli-part-needs-osm = --part izbere del območja OSM in zahteva --osm; pri --place najprej poišči id in uporabi --osm <id> --part <n>
cli-reading-file = berem { $path }
cli-no-area-for-place = za „{ $query }“ ni najdenega območja. Če je kraj območje iz več ločenih delov, izberi enega z --osm <id> --part <n> ali uporabi --geojson
cli-using-place = Uporabljam { $name } ({ $id })
cli-created-project = Projekt ustvarjen: { $path }

## Cache

cli-no-cache-dir = v tem sistemu ni mape predpomnilnika; podaj --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Prenesena 1 ploščica
        [two] Preneseni { $count } ploščici
        [few] Prenesene { $count } ploščice
       *[other] Prenesenih { $count } ploščic
    }
cli-wrote = Zapisano: { $path }
cli-kept-map = Ohranjeno: { $path } (morda vsebuje tvoje delo)
cli-map-lacks-templates = Karta še ne vsebuje sklicev na slikovne predloge. Dodaj jih v OOM (Templates > Open template…) ali zamenjaj karto z --force, ki staro ohrani kot .omap.bak.
cli-map-lacks-buildings = Karta še nima stavb iz ALKIS. Če jih želiš dodati, jo zamenjaj z --force, ki staro ohrani kot .omap.bak.
cli-map-lacks-canopies = Karti manjkajo nadstreški in nadstreški za avtomobile iz ALKIS (522), ki so zdaj objekti karte. Če jih želiš dodati, zaženi z --force, ki staro karto ohrani kot .omap.bak.
cli-map-light = Osnovna karta: združenih črt: { $joined }; skritih ali premajhnih objektov, prepuščenih slojem kandidatov: { $left } (nič ni premaknjeno)
cli-map-light-kept = Sloji kandidatov zdaj vsebujejo tudi objekte ali odseke, ki jih ta različica odstrani z osnovne karte (označeni s freemapper:generalized): { $left }. Ohranjena karta jih še ima, zato se prikažejo dvakrat. Zamenjaj karto z --force, da jo očistiš (stara ostane kot .omap.bak).
cli-map-old-candidates = Karta še vedno navaja templates/candidates.omap; ta zagon ga je napolnil z vsemi kandidati. Kandidati so zdaj tudi v sedmih slojih, po ena datoteka na sloj (templates/candidates-*.omap). Karta jih navede, ko jo zamenjaš z --force (stara ostane kot .omap.bak), ali pa jih ročno dodaj v OOM (Templates > Open template…).
cli-paper-frame-kept-map = Papirni okvirji so zapisani kot predloga templates/paper-frame.omap. Obstoječa karta se ne spremeni, zato se nanjo ne sklicuje: odpri jo v OpenOrienteering Mapper (Templates > Open template…) ali zamenjaj karto z --force (stara ostane kot .omap.bak).

## PDF

cli-pdf-without-credits = PDF brez navedbe virov: ODbL in dl-de/by-2-0 zahtevata, da ob predaji karte navedeš vire; glej ATTRIBUTION.txt
cli-wrote-pdf = Zapisano: { $path } (karta brez kandidatov)
cli-wrote-pdf-candidates = Zapisano: { $path } (kontrolni tisk: karta z vsemi kandidati, natančno na položaju; preveri jih na terenu)
cli-wrote-training = Zapisano: { $map } in { $pdf } (vadbena karta: združenih { $joined }, premaknjenih { $shifted }, izpuščenih ali odrezanih { $left })
cli-training-free-ends = Prosti konci črt: { $ways } pri stiku poti z drugo potjo, { $others } pri drugih črtah
cli-training-apart-left = vadbena karta ima še { $length } mm (na papirju), kjer imajo objekti premajhen razmik; preveri v OOM
cli-kept-training = Ohranjena vadbena karta { $path }, da zagon ne prepiše tiste, ki je bila natisnjena ali predana naprej; izbriši jo ali podaj --force, da jo obnoviš

## OCAD

cli-ocad-kept = Ohranjeno: { $path } (morda vsebuje tvoje delo; --force ga zamenja in staro ohrani kot .ocd.bak)
cli-ocad-old-candidates = Karta OCAD še vedno navaja { $path } kot karto v ozadju; ta zagon jo je napolnil z vsemi kandidati. Kandidati so zdaj tudi v sedmih slojih, po ena datoteka na sloj (-candidates-*.ocd). Karta OCAD jih navede, ko jo zamenjaš z --force (stara ostane kot .ocd.bak), ali pa jih v OCAD ročno dodaj kot karte v ozadju.
cli-ocad-wrote = { $count ->
        [one] Zapisano: { $path } (1 objekt)
        [two] Zapisano: { $path } ({ $count } objekta)
        [few] Zapisano: { $path } ({ $count } objekti)
       *[other] Zapisano: { $path } ({ $count } objektov)
    }
cli-ocad-left-out = Izpuščeno iz karte OCAD: objektov { $skipped } (brez simbola za OCAD, brez območja ali zunaj risalnega območja OCAD ±2 m) in izrojenih delov črt ali lukenj { $dropped }.
cli-ocad-load-symbols = V OCAD odpri Symbol > Load Colors and Symbols From… in izberi uradni nabor simbolov{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , šolski nabor IOF 2019
       *[other] {" "}ISSprOM 2019
    }; ta po številki nadomesti začasne simbole freemapperja.

## select

cli-select-running = Izbira območja za { $path }: na strani pritisni „Končano“ ali Ctrl+C za prekinitev.
cli-select-no-browser = Brskalnika ni bilo mogoče odpreti ({ $error }); odpri zgornji naslov.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; brez območja, ne more biti območje karte): { $place }
cli-find-hint = Projekt ustvari z: freemapper init <dir> --osm { $id }
cli-find-no-area = Noben od teh krajev nima območja, ki bi lahko bilo območje karte. Območje nariši z: freemapper select <dir>
cli-find-nothing = za „{ $query }“ ni najden noben kraj; poskusi z drugim imenom ali nariši območje z: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = opozorilo: adapter v { $folder } je izpuščen: { $finding }
cli-adapter-note = opozorilo: adapter v { $folder } se naloži, vendar: { $finding }
cli-adapter-from-project = opozorilo: uporabljen je adapter { $adapter } iz mape projekta; lahko vpraša te gostitelje: { $hosts }
cli-adapter-overlaps = opozorilo: adapter { $adapter } pokriva tudi območje, ki ga pokrivajo: { $others }; njegovi objekti pridejo na tamkajšnje karte
cli-adapter-dir-relative = opozorilo: FREEMAPPER_ADAPTER_DIR ni absolutna pot ({ $path }); prezrta
cli-adapters-no-project = opozorilo: { $path } ni mapa; adapterji projekta niso bili prebrani
cli-adapters-export-unknown = adapterja { $adapter } ne opisuje noben manifest tega programa, zato ni kaj izvoziti; `freemapper adapters list` prikaže identifikatorje
cli-adapters-export-taken = { $path } obstaja in ni prazna; izvozite v mapo, ki še ne obstaja
cli-adapters-export-done = { $count } datotek adapterja { $adapter } je zapisanih v { $path }. V njih je samo to, kar opisuje manifest; naslovov, ki so v kodi programa, ni.
cli-adapters-export-next = Spremenite naslove v manifest.toml, preverite mapo z `freemapper adapters check` in jo prestavite v { $path }: od naslednjega zagona nadomesti vgrajeni adapter, z opozorilom. Če mapo izbrišete, je vgrajeni adapter spet tam.
cli-adapters-overlaps = si deli območje z: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revizija { $revision }  preverjeno { $checked }  gostitelji: { $hosts }
cli-adapters-source-built-in = vgrajen
cli-adapters-source-user = iz tvoje mape
cli-adapters-source-project = iz projekta
cli-adapters-never-checked = nikoli
cli-adapters-user-dir = Tvoji adapterji se berejo iz { $path }
cli-adapters-no-user-dir = Ta sistem nima mape za tvoje adapterje; eno poimenuje FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper zdaj za { $name } sme vprašati tudi { $host }. Vnos je v { $file }; `freemapper adapters disallow-host { $adapter } { $host }` ga prekliče. Odprta stran z območjem ali AI-pomočnik, ki teče, prebere vnos, ko se znova zažene
cli-allow-host-already = { $host } je za { $name } že dovoljen
cli-allow-host-declared = adapter za { $name } sam navaja { $host }; ni kaj dovoliti
cli-allow-host-refused = { $host } ni mogoče dovoliti: { $reason }
cli-allow-host-unknown-adapter = vgrajenega adapterja { $adapter } ni; vgrajeni adapterji so: { $adapters }
cli-allow-host-folder-adapter = { $adapter } je adapter iz tvoje mape z adapterji; gostitelja raje dodaj v `[net] hosts` njegovega manifesta
cli-allow-host-no-config = ta sistem nima konfiguracijske mape, v kateri bi hranil dovoljene gostitelje
cli-allow-host-write = { $file } ni bilo mogoče zapisati: { $detail }
cli-allow-host-unreadable = { $file } ni mogoče prebrati ({ $detail }); najprej popravi ali odstrani datoteko; nič ni bilo spremenjeno
cli-disallow-host-removed = freemapper za { $name } ne sprašuje več { $host }
cli-disallow-host-absent = { $host } za { $name } ni bil dovoljen; ni česa preklicati
cli-allowed-hosts-unreadable = opozorilo: { $file } ni bilo mogoče prebrati ({ $detail }); dodatni gostitelji niso dovoljeni
cli-allowed-hosts-ignored = opozorilo: { $file }: vnos { $host } za { $adapter } je prezrt: { $reason }
cli-allowed-hosts-no-adapter = freemapper nima vgrajenega adapterja s tem imenom
cli-allowed-host-redundant = opozorilo: { $host } je zdaj eden od gostiteljev, ki jih navaja adapter za { $name }; tvoj vnos v { $file } je bil odstranjen
cli-adapters-allowed = dovoljeno po tvoji odločitvi: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revizija { $revision }, gostitelji: { $hosts }
cli-check-error = napaka: { $finding }
cli-check-warning = opozorilo: { $finding }
cli-check-passed = { $folder } je prestal preverjanje
cli-check-passed-warnings = { $folder } je prestal preverjanje, { $count ->
        [one] z enim opozorilom
        [two] z { $count } opozoriloma
        [few] s { $count } opozorili
       *[other] s { $count } opozorili
    }
cli-check-failed = { $folder } ni prestal preverjanja
cli-check-no-folder = mape { $folder } ni
cli-check-live-ok = { $capability } v redu: { $detail }
cli-check-live-failed = { $capability } NEUSPEŠNO: { $detail }
cli-check-live-not-asked = { $capability } ni bil vprašan: { $detail }
cli-check-live-box-outside = okvir ne leži znotraj meje adapterja { $adapter }; živo preverjanje sprašuje storitve za kraj, ki ga adapter pokriva
cli-check-live-cannot = živega preverjanja ni mogoče izvesti: { $detail }
cli-check-detail-not-declared = adapter ga ne navaja
cli-check-detail-stopped = prejšnji del ni uspel
cli-check-detail-see-warnings = glej opozorila
cli-check-detail-outside-dataset = okvir leži zunaj nabora podatkov
cli-check-detail-objects = { $count ->
        [one] en objekt
        [two] { $count } objekta
        [few] { $count } objekti
       *[other] { $count } objektov
    }
cli-check-detail-layers = slojev: { $layers }, objektov: { $count }
cli-check-detail-trees = { $count ->
        [one] eno drevo
        [two] { $count } drevesi
        [few] { $count } drevesa
       *[other] { $count } dreves
    }
cli-check-detail-extract = { $label }, podatki z dne { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = model reliefa
cli-check-raster-surface = model površja
cli-check-raster-tiles = { $product }: { $count ->
        [one] ena ploščica
        [two] { $count } ploščici
        [few] { $count } ploščice
       *[other] { $count } ploščic
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } in { $last }

## Folder lock

cli-lock-held = to mapo že uporablja drug zagon freemapperja (od { $time }, zagnal: { $who }). Počakaj nanj ali ga najprej ustavi.
cli-lock-who-cli = terminal
cli-lock-who-mcp = AI-agent (MCP)
cli-lock-who-page = izbirna stran
cli-lock-who-unknown = drug program
cli-lock-failed = mape projekta za ta zagon ni bilo mogoče zakleniti: { $error }
cli-lock-stuck = nekaj drugega kot datoteka zaklepa je na poti in ga ni bilo mogoče odstraniti

## Run routes of the selection page

cli-run-bad-request = Možnosti zagona niso veljavne: { $error }
cli-run-no-project = V tej mapi še ni shranjenega projekta. Najprej shrani območje.
cli-run-already-running = Za ta projekt že poteka zagon ({ $job }). Počakaj, da se konča, ali ga najprej prekliči.
cli-run-disk-stops = Za prenos ni dovolj prostega prostora na disku: { $messages }
cli-run-busy-elsewhere = Drug projekt tega programa trenutno izvaja cevovod. Projekti si delijo predpomnilnik prenosov, zato teče le en zagon naenkrat. Poskusi znova, ko bo končan.
cli-run-nothing-to-cancel = Noben zagon ne poteka, zato ni kaj preklicati.
cli-run-unknown-job = Za ta projekt zagona { $job } ni.
cli-status-too-fast = Zahteve za stanje prihajajo prepogosto. Vprašaj največ petkrat na sekundo.
cli-open-folder-failed = Mape ni bilo mogoče odpreti ({ $error }).
cli-open-folder-no-opener = Ta računalnik nima programa za odpiranje map ({ $program }). Mapo odpri ročno.
cli-open-folder-missing = Mape projekta še ni. Najprej shrani območje.
cli-open-folder-refused = Ime mape ni varno za predajo upravljalniku datotek.
cli-save-running = Za ta projekt poteka zagon. Shrani znova, ko bo končan.
cli-save-unknown-kind = Neznana vrsta karte „{ $kind }“. Uporabi schoolyard, sprint ali forest.
cli-save-scale-too-large = Merilo { $scale } je preveliko.
cli-save-bad-content = Nastavitve vsebine niso veljavne. Nič ni bilo shranjeno.
cli-save-bad-paper-frames = Papirni okvirji niso veljavni. Nič ni bilo shranjeno.
cli-save-paper-frames-no-project = V tej mapi še ni projekta. Najprej shrani območje karte, nato papirne okvirje.
cli-map-image-none = Najprej ustvari karto.
cli-map-image-gone = Ta predogled ni več na voljo.
cli-map-image-running = Za ta projekt poteka zagon. Predogled je mogoče izračunati, ko bo končan.
cli-map-image-stale = Shranjeni projekt se ne ujema več z ustvarjeno karto, na primer ker so se spremenili območje, vrsta karte ali merilo. Najprej zaženi zagon, ki karto zamenja („Zamenjaj obstoječo karto“ na strani ali freemapper run --force).
cli-map-image-superseded = Medtem je bil zahtevan novejši predogled, zato ta ni bil izračunan.
cli-map-image-bad-content = Nastavitve vsebine niso veljavne.
cli-map-image-bad-request = Zahteva za predogled ni veljavna. Pošlji objekt JSON, po potrebi z nastavitvami vsebine.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – izbira območja
page-heading = Izberi območje karte
page-update-available = Na voljo je freemapper { $version }. Posodobi z
page-update-notes = Opombe ob izdaji
page-folder = Mapa projekta
page-pick = Poišči kraj
page-query-placeholder = npr. Clara-Zetkin-Schule Strausberg
page-search-button = Išči
page-searching = Iščem …
page-no-results = Ni zadetkov.
page-point-only = samo točka, ne območje
page-draw = Nariši ročno
page-rectangle = Pravokotnik
page-polygon = Mnogokotnik
page-finish-polygon = Zapri mnogokotnik
page-clear = Počisti
page-hint-rectangle = Na karti klikni dva nasprotna kota.
page-hint-polygon = Klikaj kote enega za drugim. Za sklenitev klikni prvi kot, dvakrat klikni ali klikni „Zapri mnogokotnik“. Esc prekliče, Backspace odstrani zadnji kot.
page-hint-idle = Izberi kraj iz iskanja ali nariši območje ročno.
page-summary = Območje
page-source-osm = Iz OpenStreetMap: { $name }
page-source-drawn = Narisano ročno
page-size = Velikost z robom: { $width } × { $height } m
page-tiles-one = { $n } ploščica: { $names }
page-tiles-other = Ploščice ({ $n }): { $names }
page-download = Prenos: približno { $mb } MB
page-download-none = Prenos: vse je že v predpomnilniku
page-download-details = Kaj se prenese
page-cached = v predpomnilniku
page-disk = Prostor na disku: { $volumes }
page-disk-volume = prosto { $free }, potrebno približno { $needed }
page-cache-volume = predpomnilnik
page-project-volume = mapa projekta
page-no-room = ne gre na disk
page-no-room-skip = Ne gre vse na disk (freemapper ohrani { $reserve } prostega). Prenos izpusti označene dele. Če jih potrebuješ, najprej sprosti nekaj prostora:
page-no-room-stop = Ni dovolj prostora na disku (freemapper ohrani { $reserve } prostega). Brez označenih delov karte ni, zato se prenos ustavi, še preden karkoli prenese. Najprej sprosti nekaj prostora:
page-disk-unknown = freemapper ne more prebrati prostega prostora na disku ({ $error }). Prenaša brez preverjanja.
page-too-large = Območje je preveliko: največ { $max } m na stran (z robom). Izberi manjše.
page-outside = Območje leži zunaj podprtih zveznih dežel ({ $states }); freemapper za druge regije še nima virov podatkov.
page-outside-unknown = Območje leži zunaj regij, ki jih freemapper podpira; tam še nima virov podatkov.
page-adapters-failed = Letalskih posnetkov ni mogoče navesti ({ $detail }). Karta prikazuje samo OpenStreetMap; vse drugo deluje.
page-settings = Karta
page-kind = Vrsta karte
page-schoolyard = Karta šolskega dvorišča (1:1000)
page-sprint = Sprint karta (1:4000)
page-forest = Gozdna karta (1:15000)
page-scale = Merilo 1:
page-paper-format = Papirni okvir
page-paper-none = brez
page-paper-orientation = Usmerjenost
page-paper-landscape = ležeče
page-paper-portrait = pokončno
page-paper-name-landscape = { $format } ležeče
page-paper-name-portrait = { $format } pokončno
page-paper-keep = Obdrži okvir
page-paper-delete = Izbriši
page-paper-help = Okvir ti le pomaga izbrati merilo in območje; povleci ga z miško. Naslednji zagon zapiše obdržane okvirje v predlogo templates/paper-frame.omap. Nova karta se nanjo sklicuje; obstoječa karta se ne spremeni, tam predlogo odpri v OpenOrienteering Mapper (Templates > Open template…).
page-paper-saved = Papirni okvirji shranjeni. Naslednji zagon jih zapiše v predlogo templates/paper-frame.omap. Nova karta se nanjo sklicuje; pri obstoječi karti jo odpri v OpenOrienteering Mapper (Templates > Open template…).
page-map-title = Naslov
page-contour-interval = Ekvidistanca plastnic
page-contour-auto = samodejno
page-steep-slope = Prag strmosti
page-steep-slope-auto = samodejno (27°)
page-steep-slope-hint = Od tega naklona pobočja naprej se teren pri iskanju brežin in prepadov šteje za strmega. 27° (samodejno) ustreza gričevju, parkom in mestom. V skalnih mestih, soteskah in kamnolomih je strmo skoraj vsako pobočje, zato se brežine in prepadi zlijejo v eno liso in nobena ni najdena; 55° do 65° jih loči. Previsoka vrednost ohrani le najstrmejše stene.
page-face-slope = Prag sten
page-face-slope-auto = samodejno (70°)
page-face-slope-hint = Od tega naklona pobočja naprej se teren nariše kot črna ploskev skalne stene. 70° (samodejno) ustreza skalnim mestom, kot je Adršpach (5,2 % površine). 65° da več in večjih črnih ploskev, tudi na strmih pobočjih (8,4 %). 75° pusti samo prave stene z razmršenimi robovi (2,5 %). Previsoka vrednost skoraj ne pusti ploskev.
page-settings-fixed = Projekt že obstaja; vrsta karte, merilo in naslov so v freemapper.toml. Ekvidistanco plastnic je še vedno mogoče spremeniti.
page-save = Shrani
page-saving = Shranjujem …
page-saved = Shranjeno v freemapper.toml.
page-zone-changed = Shranjeno. Območje zdaj leži v drugi coni UTM; freemapper ob naslednjem zagonu znova prenese njegove podatke.
page-provider-changed = Shranjeno. Višine in letalske posnetke novega območja zagotavlja drug ponudnik podatkov; freemapper ob naslednjem zagonu znova prenese njegove podatke.
page-save-stale = freemapper.toml je bil po tem, ko ga je ta stran prebrala, spremenjen v drugem oknu ali z drugim programom. Nič ni bilo shranjeno; stran zdaj pozna shranjeno stanje. Shrani znova, da ga zamenjaš z območjem, prikazanim tukaj.
page-has-map = Obstoječa karta (.omap) ostane, kot je; freemapper naredi nove predloge in kandidate za novo območje. Možnost „Zamenjaj obstoječo karto“ spodaj (ali „freemapper run --force“) naredi znova tudi karto in staro ohrani kot .omap.bak.
page-done = Končano
page-source = Izvorna koda (AGPL-3.0-or-later)
page-closed = Končano. To okno lahko zapreš.
page-next = Naprej: freemapper run "{ $dir }"
page-made-files = Ustvarjeno: { $files }. Mapa: { $dir }
page-back-to-agent = Sporoči svojemu AI-pomočniku, da je območje shranjeno.
page-nothing-saved = Nič ni bilo shranjeno.
page-failed = Napaka: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Letalski posnetek { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Kaj pride na karto
page-content-help = Profil določa, kaj pride neposredno na karto in kaj ostane kandidat za preverjanje. Izbira za skupino prevlada nad profilom. Shrani se skupaj z območjem.
page-content-profile-legend = Profil
page-content-profile-safe = Varno
page-content-profile-safe-hint = Vsak objekt ostane tam, kamor ga postavi izgradnja: stavbe in območja katastra pridejo na karto; zidovi, ograje, drevesa in veliko podatkov iz OpenStreetMap (ribniki, poti brez znane podlage) ostanejo kandidati.
page-content-profile-balanced = Uravnoteženo
page-content-profile-balanced-hint = Objekti katastra, registrov in cestnega popisa pridejo na karto, skupaj s plastnicami in prepovedanimi območji ob železniških tirih. Zidovi, ograje, žive meje, brežine, stopnice, voda, drevesa, spomeniki, prehodi, poti in železnice prihajajo tudi iz OpenStreetMap. Ulično pohištvo, druga prepovedana območja in oblike iz modela reliefa ostanejo kandidati.
page-content-profile-full = Polno
page-content-profile-full-hint = Vse pride na karto, kot pri kontrolnem tisku; samo skriti kandidati ostanejo zunaj.
page-content-groups-legend = Po skupinah
page-content-groups-help = „Kot v profilu“ sledi profilu. Kaj to pomeni za skupino, je odvisno od tega, od kod prihaja posamezni objekt.
page-content-place-profile = Kot v profilu
page-content-place-map = Karta
page-content-place-candidates = Kandidat
page-content-place-off = Izklopljeno
page-content-group-walls = Zidovi
page-content-group-fences = Ograje
page-content-group-hedges = Žive meje
page-content-group-slopes = Brežine in prepadi
page-content-group-stairs = Stopnice
page-content-group-trees = Drevesa in skupine dreves
page-content-group-water-features = Voda, vodnjaki in izviri
page-content-group-monuments = Stolpi, spomeniki in daljnovodi
page-content-group-furniture = Ulično pohištvo in svetilke
page-content-group-passages = Prehodi
page-content-group-contours = Plastnice
page-content-group-landforms = Vzpetine, vdolbine, brežine in prepadi iz modela reliefa
page-content-group-vegetation = Vegetacija iz laserskih oblakov točk (Brandenburg, Berlin, Severno Porenje-Vestfalija, Porenje-Pfalška, Posarje in Bavarska; izklopljeno, dokler je ne izbereš; prenese približno 40 do 600 MB na km²)
page-content-group-out-of-bounds = Prepovedana območja
page-content-group-paths = Poti in železnice
page-content-group-streets = Ulice in trgi
page-run-heading = Ustvari karto
page-preview-heading = Predogled karte
page-preview-stored = Kot nazadnje ustvarjeno.
page-preview-live = Z nastavitvami na tej strani, še ne shranjeno.
page-preview-live-saved = S shranjenimi nastavitvami, še ni ustvarjeno v zagonu.
page-preview-computing = Računam …
page-preview-none = Karta še ni ustvarjena.
page-preview-later = Ta karta še nima predogleda. Dobi ga ob naslednjem zagonu, ki zapiše karto.
page-preview-stale = Območje, vrsta karte ali merilo so se spremenili, odkar je bila karta ustvarjena. Slika še vedno prikazuje karto, kot je bila nazadnje ustvarjena; za nov predogled je potreben zagon z „Zamenjaj obstoječo karto“.
page-preview-area-unsaved = Območje na tej strani še ni shranjeno. Slika prikazuje karto, kot je bila nazadnje ustvarjena; za novo območje je potreben zagon z „Zamenjaj obstoječo karto“.
page-preview-run = Kot nazadnje ustvarjeno. Stran posodobi sliko, ko se zagon konča.
page-preview-open = Odpri v polni velikosti
page-preview-alt = Predogled objektov karte
page-opt-ocad = Zapiši začetni projekt za OCAD
page-help-ocad = Za vse, ki nadaljujejo v OCAD.
page-opt-pdf = PDF za tisk
page-help-pdf = Samo objekti karte, brez kandidatov.
page-opt-pdf-candidates = Kontrolni tisk z vsemi kandidati
page-opt-training = Vadbena karta
page-help-training = Karta in vsi kandidati, očiščeni za tisk takšni, kot so; z OCAD tudi kot datoteka OCAD.
page-views-heading = Pogledi vadbene karte
page-view-no-paths = Brez poti (karta za Ultimate)
page-view-brown-only = Samo rjavo (orientacija po plastnicah)
page-view-no-north-lines = Brez severnic
page-view-custom = Drug pogled
page-help-views = Vsak označen pogled je ena datoteka več ob vadbeni karti, `<name>-training-<view>.omap` in njen PDF, z OCAD tudi datoteka OCAD. Drug pogled je `no:505,506` (izpusti simbole) ali `only:101,102` (ohrani samo te); `+` združuje poglede. Naslov, merilo in viri vedno ostanejo.
page-help-views-forest = Brez poti in samo rjavo sta samo za gozdne karte.
page-opt-scale-bar = Grafično merilo na PDF-jih
page-help-scale-bar = Nemške karte ga običajno nimajo.
page-opt-sources = Vrstice z viri na PDF-jih
page-sources-auto = samodejno
page-sources-full = polno
page-sources-credit-only = samo freemapper
page-warn-credit-only = PDF-ji bodo nato navajali samo freemapper. Licence podatkov (ODbL, dl-de/by-2-0) zahtevajo navedbo avtorstva, ko karto predaš naprej; vire moraš nato navesti drugje. ATTRIBUTION.txt jih našteje vse.
page-need-pdf = Grafično merilo in vrstice z viri veljajo samo, če nastane PDF.
page-opt-offline = Samo predpomnilnik, brez omrežja
page-opt-force = Zamenjaj obstoječe karte
page-help-force = Karto in, če je označena „Vadbena karta“, tudi vadbeno karto. Stare datoteke ostanejo kot .omap.bak.
page-opt-permits = Napotki za dovoljenja
page-help-permits = Upravne enote, zavarovana območja in koga vprašati, v permits.md. Namig, ne pravni nasvet.
page-run-start = Ustvari karto
page-run-again = Ustvari znova
page-run-cancel = Prekliči
page-run-cancelling = Preklicujem …
page-warnings-heading = Opozorila
page-where-files = Datoteke so v mapi projekta:
page-open-folder = Odpri mapo
page-log-label = Dnevnik zagona
page-force-title = Zamenjam obstoječe karte?
page-force-text = Obstoječa karta bo zamenjana, z označeno „Vadbena karta“ tudi vadbena karta. Tvoji popravki ne bodo izgubljeni: stare datoteke ostanejo v mapi projekta kot .omap.bak. Nadaljujem?
page-force-cancel = Prekliči
page-force-confirm = Zamenjaj
page-step-fetch = Prenos podatkov
page-step-derive = Računanje predlog
page-step-map = Pisanje karte
page-step-ocad = Pisanje projekta OCAD
page-step-pdf = Pisanje PDF
page-step-pdf-candidates = Pisanje kontrolnega tiska
page-step-training = Pisanje vadbene karte
page-step-permits = Pisanje napotkov za dovoljenja
page-step-line = Korak { $index } od { $total }: { $label }
page-starting = Zagon se začenja …
page-file-kind-map = karta
page-file-kind-template = predloga
page-file-kind-ocad = karta OCAD
page-file-kind-ocad-candidates = kandidati OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrolni tisk
page-file-kind-training-map = vadbena karta
page-file-kind-training-pdf = PDF vadbene karte
page-file-kind-training-ocad = vadbena karta OCAD
page-file-kind-training-view-map = pogled vadbene karte
page-file-kind-training-view-pdf = PDF pogleda vadbene karte
page-file-kind-training-view-ocad = pogled vadbene karte OCAD
page-file-kind-preview = predogled karte
page-file-kind-permits = napotki za dovoljenja
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Končano. Datoteke so zapisane.
page-result-failed = Zagon ni uspel: { $error }
page-result-failed-kept = Kar je zapisano, ostane v mapi projekta; nov zagon nadaljuje od tam.
page-result-cancelled = Preklicano. Kar je že preneseno, ostane v predpomnilniku; nov zagon nadaljuje od tam.
page-result-title-done = Karta ustvarjena
page-result-title-failed = Zagon ni uspel
page-result-title-cancelled = Zagon preklican
page-kept-map = Obstoječa karta je ostala, kot je. „Zamenjaj obstoječo karto“ jo naredi znova in staro ohrani kot .omap.bak.
page-no-files = Datotek ni bilo ustvarjenih.
page-opened-folder = Mapa je odprta.
page-open-failed = { $message } Mapa: { $dir }
page-why-no-area = Najprej izberi območje.
page-why-checking = Predogled se računa …
page-why-too-large = Območje je preveliko.
page-why-outside = Območje leži zunaj podprtih zveznih dežel.
page-why-disk = Ni dovolj prostega prostora na disku (glej zgoraj).
page-why-running = Zagon poteka.
page-err-scale-bar-no-pdf = Grafično merilo potrebuje PDF.
page-err-sources-no-pdf = Vrstice z viri potrebujejo PDF.
page-save-failed = Območje ni shranjeno; zagon ni bil začet.
page-poll-lost = Ni odgovora strežnika. freemapper poskuša znova.
page-run-unknown = Strežnik tega zagona ne pozna več.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Skrij

## update (spec 3.4 to 3.6)

update-source-build = Ta freemapper je bil zgrajen iz izvorne kode. Posodobi ga z git pull in cargo build --release --locked.
update-current = freemapper { $version } je posodobljen.
update-available = Na voljo je freemapper { $new } (imaš { $old }), izdan { $released }.
update-details =
    Opombe ob izdaji: { $notes }
    Prenos: { $size } MB
update-decimal-separator = ,
update-ask = Ga namestim zdaj? [d/N]
update-not-installed = Nič ni bilo nameščeno.
update-needs-yes = vnos ni terminal, zato freemapper ne more vprašati pred namestitvijo; zaženi freemapper update --yes za namestitev brez vprašanja
update-installing = prenašam in preverjam freemapper { $version }
update-done = freemapper posodobljen z { $old } na { $new }. Opombe ob izdaji: { $notes }
update-mcp-restart = Zagnan strežnik MCP obdrži staro različico, dokler ga AI-pomočnik ne zažene znova.
update-rolled-back = Vrnjeno na freemapper { $version }. Zapuščena različica je shranjena kot { $path }.
update-not-writable = v mapo programa { $path } ni mogoče pisati, freemapper pa ne zahteva skrbniških pravic. Novo izdajo tam razpakiraj ročno ali premakni freemapper v mapo, v katero lahko pišeš.
update-too-old = freemapper { $new } ni mogoče namestiti s samodejno posodobitvijo različice { $old }; namesti { $needed } ali novejšo ročno, glej { $notes }
update-busy = v mapi programa teče druga posodobitev ali vrnitev (datoteka zaklepa { $path }); počakaj, da se konča, in poskusi znova
update-bad-signature = izdaje ni bilo mogoče preveriti ({ $error }); nič ni bilo nameščeno
update-check-bad-signature = najnovejše izdaje freemapperja ni bilo mogoče preveriti ({ $error }); nič ni bilo nameščeno
update-no-manifest = najnovejša izdaja se ne more namestiti sama (nima manifesta posodobitve); prenesi jo z { $page }
update-no-manifest-version = izdaje { $version } ni ali se ne more namestiti sama (nima manifesta posodobitve); glej { $page }
update-no-asset = izdaja nima programa za ta sistem ({ $platform }); nič ni bilo nameščeno
update-no-previous = prejšnje različice, na katero bi se bilo mogoče vrniti, ni ({ $path } manjka)
update-failed = posodobitev ni uspela: { $error }
update-mode-set = Način posodabljanja je nastavljen na { $mode } v { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } preglasi to nastavitev, dokler je nastavljen.
update-mode-failed = { $path } ni bilo mogoče zapisati: { $error }
update-notice = Na voljo je freemapper { $new } (imaš { $old }). Posodobi z: freemapper update
update-applies-next-start = Nova različica velja od naslednjega zagona freemapperja.
update-install-failed = freemapper { $new } ni bil nameščen: { $error }
update-no-config-dir = v tem sistemu ni konfiguracijske mape; nastavi FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Opomba za osebo: ob zagonu je strežnik freemapper ugotovil naslednje o mapah z adapterji. Posreduj jo; oseba popravi mapo ali jo odstrani, nato znova zažene strežnik.
    { $lines }
update-mcp-note = Opomba za osebo: na voljo je freemapper { $new } (to je { $old }). Pred klicem install_update vprašaj osebo.
update-mcp-needs-confirm = nič ni bilo nameščeno; najprej vprašaj osebo in pokliči install_update s confirm: true, ko se strinja
update-mcp-note-manual = Opomba za osebo: na voljo je freemapper { $new } (to je { $old }), vendar ga install_update ne more namestiti; namesti { $needed } ali novejšo ročno, glej { $notes }
update-mcp-note-bad-signature = Opomba za osebo: najnovejše izdaje freemapperja ni bilo mogoče preveriti ({ $error }); nič ni bilo nameščeno. Povej ji in je ne nameščaj na noben drug način.
update-mcp-restart-now = Zdaj znova zaženi strežnik MCP freemapperja; še vedno teče stara različica.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = napaka
error-cancelled = preklicano
cli-cancelled-kept = preklicano; ohranjeno: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disk je poln, zato freemapper ni mogel zapisati { $path }. Sprosti nekaj prostora in poskusi znova

## Errors: project

error-project-already-exists = projekt že obstaja: { $path }
error-project-map-exists = karta že obstaja (morda vsebuje tvoje delo); za zamenjavo uporabi --force: { $path }
error-project-invalid-scale = neveljavno merilo 1:{ $scale }
error-project-invalid-contour-interval = neveljavna ekvidistanca plastnic { $interval } m v freemapper.toml; biti mora več kot 0
error-project-invalid-steep-slope = neveljaven prag strmosti { $slope }° v freemapper.toml; biti mora med 15° in 80°
error-project-invalid-face-slope = neveljaven prag sten { $slope }° v freemapper.toml; biti mora med 55° in 85°
error-project-invalid-paper-frame-centre = neveljavno središče papirnega okvirja, dolžina { $lon }, širina { $lat }; biti mora kraj na Zemlji
error-project-scale-not-allowed = merilo 1:{ $scale } ni dovoljeno za karte vrste { $kind }; uporabi { $allowed }
error-project-scale-not-allowed-enlarge = merilo 1:{ $scale } ni dovoljeno za karte vrste { $kind }; uporabi { $allowed }. Riši v 1:4000, freemapper-print pri tisku poveča na 1:3000
word-kind-schoolyard = šolsko dvorišče
word-kind-sprint = sprint
word-kind-forest = gozd
error-project-toml = neveljavna datoteka projekta: { $message }
error-project-unknown-adapter = datoteka projekta navaja adapter „{ $adapter }“, ki ga ta freemapper nima; novejši freemapper ga morda prinese (`freemapper update`), ali pa prihaja iz tvoje mape z adapterji (prikaže ga `freemapper adapters list`) ali, z --adapters-from-project v ukazni vrstici, iz mape `adapters` projekta (samo za projekt, ki mu zaupaš)
error-project-not-fetched = ploščice še niso prenesene; najprej zaženi `freemapper fetch`
error-project-ocd = izvoz OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: slika in world datoteka se ne ujemata z območjem karte; znova zaženi `freemapper derive`
error-project-tiles-outdated = območje karte potrebuje ploščice { $tiles }, ki niso bile prenesene; zaženi `freemapper fetch`
error-project-buildings-not-fetched = za to območje karte ni shranjenih stavb iz ALKIS; najprej zaženi `freemapper fetch` z omrežjem
error-project-orthophoto-not-fetched = za to območje karte ni shranjenega ortofota; najprej zaženi `freemapper fetch` z omrežjem
error-project-terrain-not-fetched = za to območje karte ni shranjenega modela reliefa; najprej zaženi `freemapper fetch` z omrežjem
error-project-credentials-missing = podatki za { $name } zahtevajo prijavo: nastavite okoljski spremenljivki { $user_env } (uporabniško ime) in { $password_env } (geslo); račun mora imeti naročeno: { $products }
error-project-candidates-outdated = shranjeni kandidati OSM ne pokrivajo območja karte; zaženi `freemapper fetch`
error-project-stored = { $path }: { $message }; znova zaženi `freemapper fetch`
error-project-area-too-large = območje karte z robom meri { $width } × { $height } m, kar je več kot { $limit } m na stran; zmanjšaj območje karte ali buffer_m v freemapper.toml
error-project-area-crosses-country = območje karte leži na obeh straneh državne meje; karte čez mejo še niso podprte, zato nariši območje znotraj ene države
error-project-permits-not-for-region = napotki za dovoljenja za to regijo še niso na voljo: sprašujejo nemške storitve in navajajo nemške organe
error-project-outside-coverage = območje karte leži zunaj podprtih zveznih dežel ({ $states }); freemapper ima za zdaj vire podatkov samo zanje

## Errors: disk space

error-space-required = za { $part } ni dovolj prostora na disku. Potrebno je { $needed }, vendar je v { $path } prosto le { $left } nad rezervo { $reserve }. Sprosti nekaj prostora in znova zaženi `freemapper fetch`
error-space-skipped = freemapper zaradi pomanjkanja prostora na disku preskoči { $part }. Potrebno je { $needed }, vendar je v { $path } prosto le { $left } nad rezervo { $reserve }. Sprosti nekaj prostora in znova zaženi `freemapper fetch`
word-part-map-data = podatke karte ALKIS in OSM
word-part-terrain = model reliefa
word-part-orthophoto = ortofoto
word-part-surface = model površja za predlogo nDSM
word-part-osm-update = posodobitev predpomnjenega izvlečka OSM
word-part-k-five = berlinsko predlogo karte K5
word-part-berlin-layers = berlinske strukture ALKIS, kataster dreves, cestni popis in točkovne sloje
word-part-state-layers = polne strukture ALKIS, objekte krajinskega modela in katastre dreves
word-part-point-cloud = laserske oblake točk za kandidate vegetacije

## Errors: downloads

error-fetch-http = prenos { $url } ni uspel: { $message }
error-fetch-http-temporary = prenos { $url } ni uspel: { $message }. Storitev je verjetno preobremenjena ali v vzdrževanju; freemapper je poskusil do { $attempts }-krat. Zaženite zagon čez nekaj minut znova; že prenesano se ohrani.
error-fetch-corrupt-archive = prenos { $url } ni popoln arhiv ploščic: { $message }
error-fetch-corrupt-extract = prenos { $url } ni popoln izvleček OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] ploščica { $tiles } ni pokrita z objavljenimi ploščicami; zmanjšaj območje karte ali buffer_m v freemapper.toml
        [two] ploščici { $tiles } nista pokriti z objavljenimi ploščicami; zmanjšaj območje karte ali buffer_m v freemapper.toml
       *[other] ploščice { $tiles } niso pokrite z objavljenimi ploščicami; zmanjšaj območje karte ali buffer_m v freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] brez omrežja in ni v predpomnilniku: { $product }, ploščica { $tiles }; zaženi `freemapper fetch` z omrežjem
        [two] brez omrežja in ni v predpomnilniku: { $product }, ploščici { $tiles }; zaženi `freemapper fetch` z omrežjem
       *[other] brez omrežja in ni v predpomnilniku: { $product }, ploščice { $tiles }; zaženi `freemapper fetch` z omrežjem
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = model reliefa (DGM1)
word-product-image-dsm = model površja (bDOM)
word-product-saxony-dtm = model reliefa (GeoSN DGM1)
word-product-saxony-dsm = model površja (GeoSN DOM1)
error-fetch-osm-extract-missing = izvlečka OSM ni v predpomnilniku; zaženi `freemapper fetch` z omrežjem
error-fetch-not-found = { $name } ni najden na strežniku
error-fetch-refused = { $url }: strežnik je zavrnil zahtevo (HTTP { $status })
error-fetch-refused-detail = { $url }: strežnik je zavrnil zahtevo (HTTP { $status }): { $detail }
error-fetch-too-large = prenos { $url } je večji od { $limit } MB; ustavljeno
error-fetch-not-stored = za to območje karte ni shranjeno: { $label }; { $path } manjka, ga ni mogoče prebrati ali ne pokriva območja; najprej zaženi `freemapper fetch` z omrežjem
error-fetch-range-ignored = { $url }: strežnik je prezrl razpon bajtov { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper ne pošilja zahtev na ta naslov: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper ne pošilja zahtev na ta naslov: adapter za { $name } ne navaja gostitelja { $host }{ $from }. Če se je storitev preselila tja, dovoli gostitelja z `freemapper adapters allow-host { $adapter } { $host }` in to sporoči na https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper ne pošilja zahtev na ta naslov: adapter { $adapter } ne navaja gostitelja { $host }{ $from }. Če se je storitev preselila tja, dodaj gostitelja v `[net] hosts` manifesta adapterja
error-fetch-host-redirected-from = {" "}({ $url } preusmeri tja)
error-reason-not-a-host-name = ni ime gostitelja
error-reason-not-ascii = ime zunaj ASCII mora biti zapisano v obliki xn--
error-reason-not-a-dns-name = ni ime DNS
error-reason-no-domain = ime tega računalnika ali brez domene
error-reason-local-name = ime lokalnega omrežja
error-reason-ip-address = naslov IP, ne ime DNS
error-reason-unexpected-character = nepričakovan znak
error-reason-not-https = ni https
error-reason-user-info = uporabniški podatki
error-reason-port = druga vrata kot 443
error-reason-host-not-declared = gostitelj ni med navedenimi gostitelji
error-reason-local-address = naslov tega računalnika ali lokalnega omrežja, ne strežnika na internetu
error-reason-too-many-redirects = več kot { $count } preusmeritev
error-reason-redirect-nowhere = preusmeritev (HTTP { $status }) nikamor
error-fetch-currency-csv = CSV aktualnosti, vrstica { $line }: { $message }
error-alkis = neveljavni podatki ALKIS: { $message }
error-osm = podatki OSM: { $message }

## Errors: templates and rasters

error-derive-zip = arhiv ploščic: { $message }
error-derive-image = slika: { $message }
error-derive-no-tiles = ni ploščic za izpeljavo
error-derive-area-too-large = območje karte je preveliko za eno predlogo: { $width } × { $height } slikovnih pik (največ { $limit } na stran); zmanjšaj območje karte ali buffer_m v freemapper.toml
error-raster-world-file = neveljavna world datoteka: { $message }
error-raster-misaligned = rastri niso poravnani na isto mrežo slikovnih pik: { $message }
error-raster-pixel-size = rastri imajo različne velikosti slikovnih pik: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = neveljaven GeoJSON: { $message }
error-area-no-polygon = GeoJSON ne vsebuje mnogokotnika
error-area-multi-polygon = MultiPolygon z { $parts } deli; razdeli območje karte na en mnogokotnik
error-area-out-of-range = koordinata izven obsega: lat { $lat }, lon { $lon }
error-area-not-an-area = ni uporabno območje karte: { $message }
error-georef-date = datum { $date } je zunaj veljavnosti World Magnetic Model
error-georef-map-range = točka E { $easting } N { $northing } leži zunaj obsega koordinat karte
error-omap-xml = neveljaven OOM XML: { $message }
error-omap-missing-element = elementa <{ $name }> ni
error-ocd-duplicate-symbol = številka simbola { $symbol } je uporabljena dvakrat
error-ocd-unknown-symbol = objekt se sklicuje na simbol { $symbol }, ki ga ni v datoteki
error-ocd-wrong-type = objekt simbola { $symbol } ima napačno vrsto za svoj simbol

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Uporaba:
cli-heading-commands = Ukazi:
cli-heading-options = Možnosti:
cli-heading-arguments = Argumenti:
cli-help-possible-values = možne vrednosti
cli-help-default = privzeto
cli-arg-help = Izpiše pomoč
cli-arg-version = Izpiše različico
cli-help-help = Pokaže pomoč za ukaz
cli-arg-help-command = Ukaz, za katerega se pokaže pomoč, na primer run
cli-parse-hint = Za več informacij poskusi '--help'.
cli-double-click-start = freemapper v brskalniku odpre stran, na kateri izberete območje zemljevida. Mapa projekta je { $path }. Zaprite to okno, da končate program.
cli-parse-unknown-argument = nepričakovan argument { $arg }
cli-parse-missing-required = manjka obvezen argument: { $args }
cli-parse-invalid-value = neveljavna vrednost { $value } za { $arg }
cli-parse-invalid-value-choices = neveljavna vrednost { $value } za { $arg }; možne vrednosti: { $values }
cli-parse-invalid-subcommand = neznan ukaz { $value }
cli-parse-missing-subcommand = zahtevan je ukaz
cli-parse-too-many-values = nepričakovana vrednost { $value } za { $arg }
cli-parse-argument-conflict = { $arg } ni mogoče uporabiti skupaj z { $other }

help-freemapper = Gradi začetne projekte OOM iz geopodatkov
help-freemapper-lang = Jezik besedil programa, `de` ali `en` (privzeto: FREEMAPPER_LANG, nato jezik sistema, nato angleščina)
help-init = Ustvari mapo projekta za območje karte
help-init-dir = Mapa projekta, ki se ustvari
help-init-geojson = Datoteka GeoJSON z mnogokotnikom območja karte (EPSG:4326)
help-init-osm = Objekt OSM območja karte, npr. way/992423119
help-init-place = Ime kraja, npr. "Clara-Zetkin-Schule Strausberg"; freemapper uporabi najboljši zadetek z območjem
help-init-part = Kateri del območja OSM iz več ločenih delov se uporabi (1 je največji)
help-init-kind = Vrsta karte
help-arg-scale = Imenovalec merila novega projekta: schoolyard 1000 (privzeto); sprint 4000 (privzeto) ali 5000; forest 15000 (privzeto), 10000 ali 7500. Druge vrednosti so zavrnjene, ker jih zavrne tudi freemapper-print
help-init-title = Naslov karte za obrobne podatke (privzeto: ime mape)
help-export = Zapiše začetni projekt (karta OOM, predloge, navedba virov) v mapo projekta
help-export-dir = Mapa projekta
help-arg-force = Zamenja obstoječo datoteko karte in staro ohrani kot `<name>.omap.bak`
help-export-ocad = Zapiše tudi izhodišče za OCAD v `ocad/` (oblika OCAD 12); obstoječa karta OOM nato ostane, razen če podaš --force
help-export-pdf = Zapiše tudi `<name>.pdf`, karto za tisk (samo objekti karte, brez kandidatov); obstoječa karta OOM nato ostane, razen če podaš --force
help-export-pdf-candidates = Zapiše tudi `<name>-candidates.pdf`, kontrolni tisk: karto z vsemi kandidati, natančno na položaju; obstoječa karta OOM nato ostane, razen če podaš --force
help-export-training = Zapiše tudi vadbeno karto `<name>-training.omap` in njen PDF `<name>-training.pdf`: karto in vse kandidate, očiščene za tisk takšne, kot so; z --ocad tudi `ocad/<name>-training.ocd`. Obstoječa karta OOM nato ostane, razen če podaš --force, ki zamenja tudi obstoječo vadbeno karto
help-export-view = Z --training: zapiše tudi vadbeno karto brez tistega, kar pogled izpusti, kot `<name>-training-<view>.omap` in njen PDF (z --ocad tudi datoteko OCAD). Pogledi: `no-paths` (brez cest, poti, stopnic), `brown-only` (samo rjavi simboli in okvir), `no-north-lines`, ali seznam `no:<codes>` / `only:<codes>`; združuj s `+`. Možnost ponovi za več datotek
help-export-pdf-scale-bar = Nariše grafično merilo na PDF-je (privzeto izklopljeno, nemške karte ga običajno nimajo; številka merila ostane v naslovnem bloku)
help-export-pdf-sources = Vrstice z viri na PDF-jih: `auto` natisne celoten seznam in ga skrajša samo, če bi zavzel več kot 5 vrstic, `full` ga nikoli ne skrajša, `credit-only` natisne samo vrstico z zaslugami freemapperja (ODbL in dl-de/by-2-0 nato zahtevata, da ob predaji karte navedeš vire). ATTRIBUTION.txt vedno navaja vse vire
help-fetch = Prenese podatke za območje karte (ortofoto, model reliefa, nDSM, stavbe ALKIS, OSM)
help-fetch-dir = Mapa projekta
help-fetch-offline = Uporabi samo predpomnilnik, brez dostopa do omrežja
help-arg-cache-dir = Mapa predpomnilnika za prenesene ploščice (privzeto: sistemska mapa predpomnilnika + `freemapper`)
help-derive = Izračuna predloge (ortofoto, senčenje reliefa, nagib, lokalni relief, nDSM), plastnice in oblike reliefa iz prenesenih podatkov
help-derive-dir = Mapa projekta
help-run = Prenese, izpelje in izvozi v enem koraku
help-run-dir = Mapa projekta
help-run-offline = Uporabi samo predpomnilnik, brez dostopa do omrežja
help-run-ocad = Zapiše tudi izhodišče za OCAD v `ocad/` (oblika OCAD 12)
help-run-pdf = Zapiše tudi `<name>.pdf`, karto za tisk (samo objekti karte, brez kandidatov)
help-run-pdf-candidates = Zapiše tudi `<name>-candidates.pdf`, kontrolni tisk: karto z vsemi kandidati, natančno na položaju
help-run-training = Zapiše tudi vadbeno karto `<name>-training.omap` in njen PDF `<name>-training.pdf`: karto in vse kandidate, očiščene za tisk takšne, kot so; z --ocad tudi `ocad/<name>-training.ocd`. Obstoječa vadbena karta ostane, razen če podaš --force
help-run-view = Z --training: zapiše tudi vadbeno karto brez tistega, kar pogled izpusti, kot `<name>-training-<view>.omap` in njen PDF (z --ocad tudi datoteko OCAD). Pogledi: `no-paths` (brez cest, poti, stopnic), `brown-only` (samo rjavi simboli in okvir), `no-north-lines`, ali seznam `no:<codes>` / `only:<codes>`; združuj s `+`. Obstoječa datoteka pogleda ostane, razen če podaš --force
help-run-pdf-scale-bar = Nariše grafično merilo na PDF-je (privzeto izklopljeno; številka merila ostane v naslovnem bloku)
help-run-pdf-sources = Vrstice z viri na PDF-jih: `auto` natisne celoten seznam in ga skrajša samo, če bi zavzel več kot 5 vrstic, `full` ga nikoli ne skrajša, `credit-only` natisne samo vrstico z zaslugami freemapperja (ODbL in dl-de/by-2-0 nato zahtevata, da ob predaji karte navedeš vire). ATTRIBUTION.txt vedno navaja vse vire
help-select = Območje karte izberi v brskalniku, tako da ga narišeš ali izbereš kraj. Shranjevanje zapiše freemapper.toml in po potrebi ustvari mapo projekta
help-select-dir = Mapa projekta (ustvari se ob shranjevanju, če ne obstaja)
help-select-kind = Vrsta karte novega projekta
help-select-title = Naslov karte novega projekta
help-select-no-browser = Samo izpiše naslov strani; ne odpre brskalnika
help-select-port = Vrata na 127.0.0.1 (privzeto: prosta)
help-select-offline = Stran se zažene z označeno možnostjo „Samo predpomnilnik, brez omrežja“
help-select-ocad = Stran se zažene z označenim začetnim projektom za OCAD
help-select-pdf = Stran se zažene z označenim PDF za tisk
help-select-pdf-candidates = Stran se zažene z označenim kontrolnim tiskom (vsi kandidati)
help-select-training = Stran se zažene z označeno vadbeno karto
help-select-pdf-scale-bar = Stran se zažene z označenim grafičnim merilom na PDF-jih
help-select-pdf-sources = Stran se zažene s to izbiro vrstic z viri na PDF-jih, kot pri `run`
help-find = Navede kraje OpenStreetMap za ime, najprej območja, z id za init --osm
help-find-query = Ime kraja, npr. "Clara-Zetkin-Schule Strausberg"
help-find-json = Zadetke izpiše kot JSON, ista polja kot orodje MCP find_place
help-mcp = Orodja streže AI-agentu prek MCP (stdin/stdout)
help-update = Po vprašanju namesti novejši freemapper iz izdaj na GitHubu. Gradnja iz izvorne kode namesto tega pove, kako se posodobi
help-update-check = Samo preveri: izhodna koda 10, če obstaja novejša različica, 0, če je ta trenutna
help-update-yes = Namesti brez vprašanja; potrebno, kadar vnos ni terminal
help-update-version = Namesti to različico (x.y.z) namesto najnovejše, tudi starejšo
help-update-rollback = Vrne se na različico, ki jo je zamenjala zadnja posodobitev
help-update-mode = Shrani, kaj stori dnevno preverjanje, ko najde novejšo različico (privzeto: notify). Deluje v vsaki gradnji, ker samo zapiše nastavitev
help-adapters = Pokaže regijske adapterje: zvezne dežele in države, za katere ima freemapper vire podatkov
help-adapters-list = Navede adapterje s poreklom (vgrajen, tvoja mapa, projekt), revizijo, dnevom zadnjega preverjanja in gostitelji, ki jih sprašuje
help-adapters-list-json = Seznam izpiše kot JSON
help-adapters-list-project = Prebere tudi adapterje v mapi `adapters` te mape projekta, kot bi jih --adapters-from-project
help-adapters-check = Preveri mapo adapterja, preden se uporabi ali preda naprej: ali bi se naložil, kaj naj avtor ve, in z --live, ali njegove storitve odgovarjajo. Izhodna koda 0: prestane; 1: ima ugotovitve; 2: preverjanja ni bilo mogoče izvesti
help-adapters-check-dir = Mapa adapterja; njeno ime je id adapterja
help-adapters-check-live = Storitve adapterja vpraša tudi za okvir, ki ga imenuje --bbox, in pove, kaj vsaka vrne. Zahteve ostanejo pri gostiteljih, ki jih navaja manifest
help-adapters-check-bbox = Okvir živega preverjanja v stopinjah, kot minLon,minLat,maxLon,maxLat. Leži znotraj meje adapterja; zadošča 150 m krat 150 m
help-adapters-check-json = Rezultat izpiše kot JSON
help-adapters-probe = Enkrat povprašati vsako storitev, ki jo navajajo vgrajeni adapterji, in povedati, ali odgovarja: ok, slow, moved ali error. Izhodna koda je 0 tudi, če storitev odpove; rezultat je odgovor
help-adapters-probe-json = Rezultat izpiše kot JSON
help-adapters-probe-status = Prebrati stanje prejšnje noči iz te datoteke (lahko manjka) in izpisati novo stanje kot JSON: storitev je nedosegljiva po dveh slabih nočeh zapored. Spremembe gredo na standardni izhod za napake
help-adapters-allow-host = Vgrajenemu adapterju dovoli še enega gostitelja: za zvezno deželo, katere storitev se je preselila na strežnik, ki ga njen adapter še ne navaja. Sporočilo o zavrnjenem naslovu pove, kateri gostitelj. Gostitelj se preveri kot gostitelj manifesta, freemapper pa se še vedno povezuje samo s strežniki na internetu
help-adapters-allow-host-adapter = Id vgrajenega adapterja, kot ga prikaže `adapters list`
help-adapters-allow-host-host = Gostitelj, ki se dovoli: ime DNS, ne naslov, ne vrata
help-adapters-disallow-host = Prekliče gostitelja, ki ga je dovolil allow-host
help-adapters-disallow-host-adapter = Id vgrajenega adapterja
help-adapters-disallow-host-host = Gostitelj, ki se prekliče
help-adapters-export = Zapiše vgrajeni adapter zvezne dežele kot mapo adapterja, ki jo lahko spremenite: njegov manifest in datoteke, ki jih manifest navaja, v `<dir>/<id>`. Mapa še ne sme obstajati ali mora biti prazna. Spremenite naslove v njenem `manifest.toml`, preverite jo z `adapters check` in jo dajte v svojo mapo adapterjev: potem nadomesti vgrajeni adapter, z opozorilom. Če mapo odstranite, je vgrajeni adapter spet tam
help-adapters-export-adapter = Identifikator vgrajenega adapterja, kot ga prikaže `adapters list`
help-adapters-export-dir = Mapa, v kateri nastane mapa `<id>`
help-sources = Brati in popravljati naslove virov podatkov brez nove izdaje. Popravek gre v vašo lastno mapo adapterjev in nadomesti vgrajeni naslov
help-sources-list = Naštej naslove v manifestih z njihovimi potmi in izvorom vsakega manifesta. Z ID-jem adapterja samo naslove tega adapterja
help-sources-get = Izpiši en naslov in nič drugega. Izhodna koda 1, če je adapter ali pot neznana
help-sources-set = Popravi en naslov v svoji mapi adapterjev, ko je celoten manifest prestal preverjanje. Naslov na gostitelju, ki ga manifest ne deklarira, je zavrnjen, razen če podate --add-host
help-sources-diff = Pokaži, kaj vaša mapa adapterja spremeni glede na vgrajeni manifest
help-sources-check = Preveri mapo vašega adapterja s tem ID-jem kot `adapters check`. Izhodna koda 0: prestal; 1: ugotovitve; 2: preverjanje se ni moglo izvesti
help-sources-schema = Izpiši ključe manifesta, ki vsebujejo naslov, z --json kot shemo JSON
help-arg-adapter = ID adapterja, kot ga prikaže `adapters list`
help-arg-path = Pot naslova, kot jo prikaže `sources list`
help-sources-set-url = Novi naslov: https, brez poizvedbe
help-sources-set-add-host = Dodaj tudi gostitelja novega naslova med gostitelje, ki jih manifest deklarira
cli-sources-no-user-dir = ta sistem nima mape za vaše adapterje; nastavite FREEMAPPER_ADAPTER_DIR na absolutno pot
cli-sources-bad-id = { $adapter } ni ID adapterja
cli-sources-unknown-adapter = freemapper ne pozna adapterja { $adapter }; `freemapper adapters list` prikaže ID-je
cli-sources-no-manifest = noben manifest tega programa ne opisuje adapterja { $adapter }; njegovi naslovi so v kodi
cli-sources-unknown-path-of = adapter { $adapter } nima naslova pod { $path }; `freemapper sources list { $adapter }` prikaže poti
cli-sources-bad-value = novi naslov mora biti naslov https na poimenovanem gostitelju, brez navednice, poševnice nazaj, krmilnega znaka, uporabnika in vrat
cli-sources-host-not-declared = novi naslov vodi do gostitelja { $host }, ki ga manifest ne deklarira; --add-host ga deklarira
cli-sources-invalid = manifest s to spremembo preverjanja ne bi prestal, zato ni bilo nič zapisano: { $findings }
cli-sources-set-created = Mapa { $folder } adapterja { $adapter } je bila ustvarjena iz njegovega vgrajenega manifesta; od naslednjega zagona nadomesti vgrajeni adapter.
cli-sources-set-host-added = Gostitelj { $host } je zdaj deklariran v [net] hosts; freemapper za ta adapter zdaj sprašuje ta strežnik.
cli-sources-set-next = Preverite jo z `freemapper adapters check { $folder }`. Navedba vira in licenca ostaneta, kakršni sta bili: ohranite ju pravilni, če je storitev zamenjala lastnika. Znova zaženite strežnik MCP ali stran območja.
cli-sources-diff-no-folder = nimate mape adapterja { $adapter } ({ $folder }); ni kaj primerjati
cli-sources-diff-none = vaša mapa vsebuje enake naslove kot vgrajeni manifest
help-sources-list-json = Seznam izpiše kot JSON
help-sources-check-live = Storitve adapterja vpraša tudi za okvir, ki ga imenuje --bbox, in pove, kaj vsaka vrne. Zahteve ostanejo pri gostiteljih, ki jih navaja manifest
help-sources-check-bbox = Okvir živega preverjanja v stopinjah, kot minLon,minLat,maxLon,maxLat. Leži znotraj meje adapterja; zadošča 150 m krat 150 m
help-sources-check-json = Rezultat izpiše kot JSON
help-sources-schema-json = Rezultat izpiše kot JSON
help-arg-adapters-from-project = Naloži tudi adapterje v mapi `adapters` mape projekta. freemapper vsakega poimenuje z gostitelji, ki jih sprašuje. Uporabi za projekt, ki mu zaupaš: adapter prisili freemapper, da sprašuje strežnike, ki jih navaja

## The summary of a run, for the MCP server and the area page

cli-internal-error = notranja napaka: { $message }
cli-pdf-sources-invalid = pdf_sources mora biti auto, full ali credit-only, ne { $value }
cli-view-unknown = pogled je no-paths, brown-only, no-north-lines, no:<codes> ali only:<codes>, ne { $value }
cli-view-empty = seznam { $value } ne navaja nobene kode simbola
cli-view-bad-code = { $value } ni koda simbola, kot sta 505 ali 501.6
cli-summary-training-view = Zapisano: { $map } in { $pdf } (pogled vadbene karte)
cli-summary-kept-training-view = Ohranjene datoteke pogleda { $view } vadbene karte; podaj --force, da jih obnoviš
cli-summary-training-view-failed = Pogled vadbene karte ni uspel: { $error }
cli-estimate-disk-unreadable = prostega prostora na disku { $path } ni mogoče prebrati ({ $message })
cli-summary-wrote-map = Zapisana karta { $path }
cli-summary-kept-map = Ohranjena obstoječa karta { $path } (morda vsebuje delo kartografa)
cli-summary-map-lacks-templates = Karta še ne vsebuje sklicev na slikovne predloge. Zaženi cevovod s force, da jo zamenjaš in staro ohraniš kot .omap.bak.
cli-summary-map-lacks-buildings = Karta še nima stavb iz ALKIS. Zaženi cevovod s force, da jih dodaš; zamenja karto in staro ohrani kot .omap.bak.
cli-summary-map-lacks-canopies = Karti manjkajo nadstreški in nadstreški za avtomobile iz ALKIS (522), ki so zdaj objekti karte. Zaženi cevovod s force, da jih dodaš; zamenja karto in staro ohrani kot .omap.bak.
cli-summary-map-light = Osnovna karta: združenih črt: { $joined }; skritih ali premajhnih objektov, prepuščenih slojem kandidatov: { $left } (nič ni premaknjeno)
cli-summary-map-light-kept = Sloji kandidatov zdaj vsebujejo tudi objekte ali odseke, ki jih ta različica odstrani z osnovne karte (označeni s freemapper:generalized): { $left }. Ohranjena karta jih še ima, zato se prikažejo dvakrat. Zaženi cevovod s force, da karto očistiš; staro ohrani kot .omap.bak.
cli-summary-map-old-candidates = Karta še vedno navaja templates/candidates.omap; ta zagon ga je napolnil z vsemi kandidati. Kandidati so zdaj tudi v sedmih slojih, po ena datoteka na sloj (templates/candidates-*.omap). Karta jih navede po zagonu s force (stara karta ostane kot .omap.bak), ali pa jih kartograf doda v OOM (Templates > Open template…).
cli-summary-paper-frame-kept-map = Papirni okvirji so zapisani kot predloga templates/paper-frame.omap. Obstoječa karta se ne spremeni, zato se nanjo ne sklicuje: kartograf jo odpre v OpenOrienteering Mapper (Templates > Open template…), ali pa se karta nanjo sklicuje po zagonu s force (stara karta ostane kot .omap.bak).
cli-summary-wrote-ocad = Zapisana karta OCAD { $path } (objektov: { $count })
cli-summary-kept-ocad = Ohranjena obstoječa karta OCAD { $path } (morda vsebuje delo kartografa; force jo zamenja)
cli-summary-ocad-old-candidates = Karta OCAD še vedno navaja { $path } kot karto v ozadju; ta zagon jo je napolnil z vsemi kandidati. Kandidati so zdaj tudi v sedmih slojih, po ena datoteka na sloj (-candidates-*.ocd). Karta OCAD jih navede po zagonu s force (stara karta ostane kot .ocd.bak), ali pa jih kartograf doda v OCAD kot karte v ozadju.
cli-summary-ocad-left-out = Izpuščeno iz karte OCAD: objektov { $skipped } in izrojenih delov { $dropped }
cli-summary-ocad-symbols = V OCAD odpri Symbol > Load Colors and Symbols From… in izberi uradni nabor simbolov{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , šolski nabor IOF 2019
       *[other] {" "}ISSprOM 2019
    }; ta po številki nadomesti začasne simbole.
cli-summary-ocad-failed = izvoz OCAD ni uspel: { $error }
cli-summary-pdf-without-credits = PDF brez navedbe virov: ODbL in dl-de/by-2-0 zahtevata, da ob predaji karte navedeš vire; glej ATTRIBUTION.txt
cli-summary-wrote-pdf = Zapisana karta { $path } (brez kandidatov)
cli-summary-pdf-failed = izvoz PDF ni uspel: { $error }
cli-summary-wrote-pdf-candidates = Zapisan kontrolni tisk { $path } (karta z vsemi kandidati, natančno na položaju)
cli-summary-pdf-candidates-failed = izvoz kontrolnega tiska ni uspel: { $error }
cli-summary-training-failed = izvoz vadbene karte ni uspel: { $error }
cli-summary-kept-training = Ohranjena obstoječa vadbena karta { $path }, da zagon ne prepiše tiste, ki je bila natisnjena ali predana naprej; izbriši jo ali nastavi force, da jo obnoviš

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = v tem sistemu ni domače mape; uporabi absolutno pot
mcp-error-relative-path = { $path } je relativna pot; uporabi absolutno pot ali pot, ki se začne z ~
mcp-error-home-folder = { $path } je sama domača mapa; uporabi mapo v njej, npr. ~/maps/school
mcp-error-no-project = v { $path } ni projekta freemapper; najprej uporabi create_project
mcp-error-unknown-kind = neznana vrsta { $kind }; uporabi "schoolyard", "sprint" ali "forest"
mcp-error-reading = berem { $path }: { $error }
mcp-error-one-area-source = podaj natanko enega od osm_id (iz find_place) ali geojson
mcp-error-page-open = stran za izbiro območja tega projekta je še odprta; prosi uporabnika, naj shrani območje in na strani pritisne Končano, nato zaženi cevovod
mcp-error-no-job = opravila { $job } ni; start_pipeline in permits_for_area vrneta id-je opravil te seje
mcp-error-job-running = { $job } še teče za { $path }; naenkrat teče en cevovod, ker si projekti delijo predpomnilnik prenosov. Ta zaženi, ko project_status sporoči finished ali failed
mcp-error-page-start = strani ni mogoče zagnati: { $error }
mcp-created-project = Projekt { $path } ustvarjen za { $area } ({ $kind }, 1:{ $scale }). Naprej: estimate_download, nato start_pipeline.
mcp-created-project-given-area = Projekt { $path } ustvarjen za podano območje ({ $kind }, 1:{ $scale }). Naprej: estimate_download, nato start_pipeline.
mcp-page-already-open = Stran za to mapo je že odprta; obdrži nastavitve, s katerimi je bila odprta.
mcp-page-next = Prosi uporabnika, naj na strani izbere območje, pritisne Shrani in nato Končano.
mcp-preview-of = Predogled sloja { $layer }
mcp-permits-next = Napotki za dovoljenja se izdelujejo. Poizveduj po project_status z id-jem tega opravila (job_id); ko je finished, permits_md vsebuje besedilo permits.md. Posreduj ga kot namig, ne kot pravni nasvet.
mcp-tool-check-update = Ali obstaja novejša izdaja freemapperja: njena različica, datum izdaje, opombe ob izdaji in velikost prenosa, ali da je ta trenutna. Ničesar ne namesti.
mcp-tool-create-project = Ustvari mapo projekta za območje karte. Podaj osm_id (iz find_place) ali datoteko GeoJSON. kind je schoolyard (1:1000, privzeto), sprint (1:4000) ali forest (1:15000).
mcp-param-create-project-dir = Nova mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-param-create-project-geojson = Namesto osm_id: datoteka GeoJSON z mnogokotnikom območja karte (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (privzeto, 1:1000), "sprint" (1:4000) ali "forest" (1:15000).
mcp-param-create-project-osm-id = Objekt OSM območja karte iz find_place, npr. "way/992423119".
mcp-param-create-project-part = Z osm_id: kateri del območja iz več ločenih delov se uporabi (1 je največji).
mcp-param-create-project-scale = Imenovalec merila: schoolyard 1000 (privzeto); sprint 4000 (privzeto) ali 5000; forest 15000 (privzeto), 10000 ali 7500. Druge vrednosti so zavrnjene; za 1:3000 riši pri 4000, freemapper-print pri tisku poveča.
mcp-param-create-project-title = Naslov karte za obrobne podatke (privzeto: ime mape).
mcp-tool-estimate-download = Kaj bo start_pipeline prenesel za projekt: ploščice, vsak prenos z velikostjo v MB, ali je v predpomnilniku in ali gre na disk, še neprenesene MB, ter pod disk prosti prostor na nosilcih predpomnilnika in projekta, s sporočili za to, kar ne gre (povej uporabniku).
mcp-param-estimate-download-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-tool-find-place = Poišči šolo, park ali drug kraj po imenu (OpenStreetMap Nominatim). Vrne do 5 zadetkov, najprej območja; samo zadetki z has_area so lahko območje karte. Prazen seznam pomeni, da ni bilo najdeno nič.
mcp-param-find-place-query = Ime šole ali kraja, npr. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Namesti najnovejšo izdajo freemapperja čez ta program. Pokliči jo šele, ko se je oseba strinjala, s confirm true. Nova različica velja, ko pomočnik znova zažene strežnik freemapper.
mcp-param-install-update-confirm = true, ko se je oseba strinjala z namestitvijo novejše različice; brez tega se ničesar ne namesti.
mcp-tool-list-layers = Sloji projekta: karta, sedem slojev kandidatov, papirni okvirji (paper-frame, samo če so bili na strani območja obdržani okvirji) in rastrske predloge, vsak z datoteko, ali je že izdelan in kateri korak ga izdela.
mcp-param-list-layers-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-tool-open-area-selector = Odpre stran v brskalniku, kjer uporabnik nariše območje karte (pravokotnik ali mnogokotnik) ali izbere kraj, vidi ploščice in velikost prenosa ter ga shrani; shranjevanje ustvari mapo projekta ali spremeni njeno območje. Vrne URL, ki ga je treba dati uporabniku. Stran teče, dokler uporabnik ne pritisne Končano; ponoven klic za isto mapo vrne isti URL.
mcp-param-open-area-selector-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~); ustvari se, ko uporabnik shrani.
mcp-param-open-area-selector-kind = Vrsta karte novega projekta: "schoolyard" (privzeto, 1:1000), "sprint" (1:4000) ali "forest" (1:15000).
mcp-param-open-area-selector-ocad = Plošča zagona se začne z označenim začetnim projektom za OCAD. Privzeto false.
mcp-param-open-area-selector-offline = Plošča zagona se začne z označeno možnostjo „Samo predpomnilnik, brez omrežja“. Privzeto false.
mcp-param-open-area-selector-open-browser = Stran odpre tudi v privzetem brskalniku tega računalnika. Privzeto true.
mcp-param-open-area-selector-pdf = Plošča zagona se začne z označenim PDF za tisk. Privzeto false.
mcp-param-open-area-selector-pdf-candidates = Plošča zagona se začne z označenim kontrolnim tiskom (vsi kandidati). Privzeto false.
mcp-param-open-area-selector-pdf-scale-bar = Plošča zagona se začne z označenim grafičnim merilom na PDF-jih. Privzeto false.
mcp-param-open-area-selector-pdf-sources = Vrstice z viri na PDF-jih, s katerimi se plošča začne: "auto" (privzeto), "full" ali "credit-only", kot pri start_pipeline.
mcp-param-open-area-selector-permits = Plošča zagona se začne z označenimi napotki za dovoljenja. Privzeto false.
mcp-param-open-area-selector-scale = Imenovalec merila novega projekta: schoolyard 1000 (privzeto); sprint 4000 (privzeto) ali 5000; forest 15000 (privzeto), 10000 ali 7500. Druge vrednosti so zavrnjene.
mcp-param-open-area-selector-title = Naslov karte novega projekta.
mcp-param-open-area-selector-training = Plošča zagona se začne z označeno vadbeno karto. Privzeto false.
mcp-tool-preview-layer = Predogled PNG (največ 800 px) sloja: map (slika objektov karte, ki jo je zapisal zadnji zagon, brez kandidatov) ali rastrska predloga: orthophoto, hillshade, slope, relief, ndsm, ali k5 v Berlinu.
mcp-param-preview-layer-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" ali "k5" (samo Berlin).
mcp-tool-project-status = Stanje opravila start_pipeline ali permits_for_area: running, finished ali failed, zadnje vrstice napredka, opozorila, zapisane datoteke ter rezultat ali napaka. Dokončano opravilo permits_for_area nosi tudi besedilo permits.md kot permits_md.
mcp-param-project-status-job-id = Id opravila iz start_pipeline ali permits_for_area, npr. "job-1".
mcp-tool-permits-for-area = Izdela napotke za dovoljenja projekta: upravne enote območja karte, zavarovana območja v njem in kdo tam upravlja zemljišče, z organi, ki jih je treba vprašati; namig, ne pravni nasvet. V mapo zapiše permits.md in permits.json. Teče v ozadju in vrne job_id; poizveduj po project_status, katerega odgovor ob dokončanem opravilu nosi besedilo permits.md kot permits_md. Naenkrat teče eno opravilo.
mcp-param-permits-for-area-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-param-permits-for-area-offline = Prebere samo podatke, ki jih je v projektu shranil prejšnji zagon, brez dostopa do omrežja. Privzeto false.
mcp-tool-start-pipeline = Prenese podatke, izračuna predloge in zapiše začetni projekt (fetch, derive, export). Teče v ozadju in vrne job_id; poizveduj po project_status. Naenkrat teče en cevovod. Obstoječo karto ohrani, razen če je force true.
mcp-param-start-pipeline-dir = Mapa projekta (absolutna pot ali pot, ki se začne z ~).
mcp-param-start-pipeline-force = Zamenja obstoječo datoteko karte in staro ohrani kot .omap.bak. Privzeto false.
mcp-param-start-pipeline-ocad = Zapiše tudi izhodišče za OCAD v mapo ocad/ (oblika OCAD 12, za uporabnike, ki urejajo v OCAD). Privzeto false.
mcp-param-start-pipeline-offline = Uporabi samo predpomnilnik prenosov, brez dostopa do omrežja. Privzeto false.
mcp-param-start-pipeline-pdf = Zapiše tudi <name>.pdf, karto, pripravljeno za tisk (samo objekti karte, brez kandidatov). Privzeto false.
mcp-param-start-pipeline-pdf-candidates = Zapiše tudi <name>-candidates.pdf, kontrolni tisk: karto z vsemi kandidati, natančno na položaju (preveri jih na terenu). Privzeto false.
mcp-param-start-pipeline-pdf-scale-bar = Nariše grafično merilo na PDF-je. Privzeto false (nemške karte ga običajno nimajo; številka merila ostane v naslovnem bloku).
mcp-param-start-pipeline-pdf-sources = Vrstice z viri na PDF-jih: "auto" (privzeto; celoten seznam, skrajšan samo, če zavzame več kot 5 vrstic), "full" (nikoli skrajšan) ali "credit-only" (samo vrstica z zaslugami freemapperja; ODbL in dl-de/by-2-0 nato zahtevata navedbo virov ob predaji karte). ATTRIBUTION.txt vedno navaja vse vire.
mcp-param-start-pipeline-permits = Po karti zapiše tudi napotke za dovoljenja (permits.md in permits.json): upravne enote, zavarovana območja in kdo upravlja zemljišče. Napaka pri tem je opozorilo, ne neuspešno opravilo. Privzeto false.
mcp-param-start-pipeline-training = Zapiše tudi vadbeno karto <name>-training.omap in njen PDF <name>-training.pdf: karto in vse kandidate, očiščene za tisk takšne, kot so; z ocad tudi ocad/<name>-training.ocd. Obstoječa vadbena karta se ohrani, razen če je force true. Privzeto false.
mcp-param-start-pipeline-training-views = S training: zapiše tudi vadbeno karto brez tistega, kar pogled izpusti, en komplet datotek na vnos: "no-paths", "brown-only", "no-north-lines" ali seznam, kot je "no:505,506" ali "only:101,102", združen s +. Datoteke so <name>-training-<view>.omap in njen PDF.
mcp-instructions = freemapper izdeluje začetne projekte OpenOrienteering Mapper za orientacijske karte (šolsko dvorišče 1:1000, sprint 1:4000, gozd 1:15000) iz uradnih geopodatkov { $states } in iz OSM. Potek: find_place (poišči šolo ali območje po imenu; izberi zadetek z has_area) → create_project (nova mapa, absolutna pot, z osm_id) → estimate_download (povej uporabniku, koliko se bo preneslo) → start_pipeline (vrne id opravila; prenos lahko traja minute) → project_status (poizveduj vsakih nekaj sekund, dokler stanje ni finished ali failed) → list_layers in preview_layer (pokaži uporabniku predloge). Uporabnik nato odpre datoteko karte v OpenOrienteering Mapper. Kadar noben kraj ne ustreza ali uporabnik želi narisati območje, open_area_selector odpre stran v brskalniku, kjer uporabnik nariše ali izbere območje in ga shrani (to ustvari projekt); nadaljuj z estimate_download, ko uporabnik pove, da je shranjeno. Poti morajo biti absolutne ali se začeti z ~. Naenkrat teče en cevovod. permits_for_area izdela napotke za dovoljenja projekta (upravne enote, zavarovana območja, kdo upravlja zemljišče; namig, ne pravni nasvet) kot opravilo, podobno start_pipeline. check_update pove, ali obstaja novejši freemapper; install_update ga namesti, samo s confirm: true, ko se je oseba strinjala, pomočnik pa nato znova zažene strežnik freemapper.

cli-contour-interval-saved = Ekvidistanca plastnic shranjena v freemapper.toml: { $interval } m; derive izriše plastnice z njo
cli-contour-interval-auto = Ekvidistanca plastnic v freemapper.toml: samodejno (po vrsti karte; gozdna karta dobi 2,5 m v ravnini in 10 m v strmem svetu)
cli-steep-slope-saved = Prag strmosti shranjen v freemapper.toml: { $slope }°; derive z njim išče brežine in prepade
cli-steep-slope-auto = Prag strmosti v freemapper.toml: privzeto (približno 27°)
cli-face-slope-saved = Prag sten shranjen v freemapper.toml: { $slope }°; derive z njim išče ploskve skalnih sten
cli-face-slope-auto = Prag sten v freemapper.toml: privzeto (70°)
help-arg-contour-interval = Ekvidistanca plastnic v metrih, npr. `2.5`, ali `auto` za tisto, ki velja za vrsto karte (gozdna karta: 5 m, 2,5 m kjer je ravno, 10 m kjer je strmo). Shranjeno v freemapper.toml; derive izriše plastnice z njo
help-arg-steep-slope = Naklon pobočja v stopinjah, od katerega se teren pri iskanju brežin in prepadov šteje za strmega, npr. `55`, ali `auto` za privzetih 27°. Zvišaj ga (55 do 65) v skalnih mestih in soteskah, kjer je strmo skoraj vsako pobočje, brežine in prepadi pa se zlijejo v eno liso. Shrani se v freemapper.toml; derive z njim išče brežine in prepade
help-arg-face-slope = Naklon pobočja v stopinjah, od katerega je teren skalna stena (črna ploskev, znak 206), npr. `75`, ali `auto` za privzetih 70°. 65 da več in večjih ploskev, tudi na strmih pobočjih; 75 pusti samo prave stene. Shrani se v freemapper.toml; derive z njim išče skalne stene
cli-content-saved = Nastavitev vsebine shranjena v freemapper.toml: { $summary }
cli-content-invalid = Nastavitve vsebine ni mogoče prebrati: { $detail }
help-arg-content = Kaj pride neposredno na karto: `safe` pusti vsak objekt tam, kamor ga postavi izgradnja, `balanced` (privzeto) da na karto objekte katastra in registrov, `full` da na karto vse. Izbira se shrani v freemapper.toml
help-arg-content-set = Pošlje skupino objektov na `map`, `candidates` ali `off`, kot `GROUP=PLACE`, npr. `walls=map`; lahko se ponovi. Skupine: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Shranjeno v freemapper.toml
mcp-param-create-project-content = Kaj pride neposredno na karto: "safe", "balanced" (privzeto) ali "full". Shranjeno v projektu. balanced da na karto objekte katastra in registrov, full da na karto vse.
mcp-param-create-project-content-set = Skupine, poslane drugam, kot določa profil, kot "group=place" s place map, candidates ali off, npr. "walls=map". Skupine: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Ekvidistanca plastnic v metrih, npr. 2.5; 0 se vrne k samodejni izbiri (po vrsti karte; gozdna karta dobi 2,5 m v ravnini in 10 m v strmem svetu). Shranjeno v projektu; naslednji derive izriše plastnice z njo.
mcp-param-create-project-steep-slope = Naklon pobočja v stopinjah, od katerega se teren pri iskanju brežin in prepadov šteje za strmega, npr. 55; 0 vrne privzetih 27°. Zvišaj ga (55 do 65) v skalnih mestih in soteskah, kjer je strmo skoraj vsako pobočje, brežine in prepadi pa se zlijejo v eno liso. Shrani se v projekt; naslednji derive z njim išče brežine in prepade.
mcp-param-create-project-face-slope = Naklon pobočja v stopinjah, od katerega je teren skalna stena (črna ploskev, znak 206), npr. 75; 0 vrne privzetih 70°. 65 da več in večjih ploskev, tudi na strmih pobočjih; 75 pusti samo prave stene. Shrani se v projekt; naslednji derive z njim išče skalne stene.
mcp-param-start-pipeline-content = Pred zagonom spremeni, kaj pride neposredno na karto: "safe", "balanced" ali "full". Shranjeno v projektu. Privzeto: nastavitev projekta, sicer balanced.
mcp-param-start-pipeline-content-set = Skupine, poslane drugam, kot določa profil, kot "group=place" s place map, candidates ali off, npr. "fences=candidates". Shranjeno v projektu.
mcp-param-start-pipeline-contour-interval = Ekvidistanca plastnic v metrih, npr. 2.5; 0 se vrne k samodejni izbiri (po vrsti karte; gozdna karta dobi 2,5 m v ravnini in 10 m v strmem svetu). Shranjeno v projektu; naslednji derive izriše plastnice z njo.
mcp-param-start-pipeline-steep-slope = Naklon pobočja v stopinjah, od katerega se teren pri iskanju brežin in prepadov šteje za strmega, npr. 55; 0 vrne privzetih 27°. Zvišaj ga (55 do 65) v skalnih mestih in soteskah, kjer je strmo skoraj vsako pobočje, brežine in prepadi pa se zlijejo v eno liso. Shrani se v projekt; naslednji derive z njim išče brežine in prepade.
mcp-param-start-pipeline-face-slope = Naklon pobočja v stopinjah, od katerega je teren skalna stena (črna ploskev, znak 206), npr. 75; 0 vrne privzetih 70°. 65 da več in večjih ploskev, tudi na strmih pobočjih; 75 pusti samo prave stene. Shrani se v projekt; naslednji derive z njim išče skalne stene.
mcp-error-content = Nastavitve vsebine ni mogoče prebrati: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Zapiše napotke za dovoljenja projekta (permits.md in permits.json): upravne enote, zavarovana območja in upravljavce, ki jih območje karte zadeva, in koga je običajno treba vprašati. Vpraša BfN in BKG ter, kjer ima zvezna dežela območja karte lastne odprte storitve, te prek gostiteljev njenega adapterja (seznam v priročniku, poglavje 12); v Severnem Porenju-Vestfaliji pri prvem zagonu za to prenese približno 32 MB, v Brandenburgu približno 4 MB, pozneje samo novejše datoteke. Prebere že predpomnjeni izvleček OSM; --offline uporabi, kar je bilo preneseno prej
help-permits-dir = Mapa projekta
help-permits-offline = Uporabi samo prej prenesene podatke, brez dostopa do omrežja
help-run-permits = Po karti zapiše tudi napotke za dovoljenja (permits.md in permits.json); vpraša BfN, BKG in odprte storitve zvezne dežele (priročnik, poglavje 12). Napaka pri tem je opozorilo, karta ostane
cli-wrote-permits = Zapisani napotki za dovoljenja { $path }
cli-summary-wrote-permits = Zapisani napotki za dovoljenja { $path }
cli-permits-failed = napotkov za dovoljenja ni bilo mogoče zapisati: { $error }
cli-permits-source-left-out = napotki za dovoljenja izpustijo { $source }: { $reason }
cli-permits-not-read = ni bilo prebrano
error-permits-stored = { $path }: { $message }; znova prenesi napotke za dovoljenja z omrežjem
error-permits-nothing-read = noben vir napotkov za dovoljenja ni bil prebran ({ $missing }); najprej jih prenesi z omrežjem
error-permits-missing-not-read = { $source }: ni prebrano

permits-title = Napotki za dovoljenja
permits-disclaimer = Namig, ne pravni nasvet. Odločajo pristojni organi. Za datum in natančnost podatkov glej tabelo pod „Viri in datum podatkov“; območja lahko manjkajo ali so zastarela (BfN: združeno enkrat na leto, „Nicht für Planungszwecke geeignet.“, torej ni primerno za načrtovanje).
permits-disclaimer-regional = Namig, ne pravni nasvet. Odločajo pristojni organi. Za datum in natančnost podatkov glej tabelo pod „Viri in datum podatkov“; območja lahko manjkajo ali so zastarela.
permits-bfn-merge = BfN enkrat na leto združi podatke zveznih dežel in jih označi kot neprimerne za načrtovanje. Odločajo organi zvezne dežele.
permits-created = Izdelano { $date } za območje { $kind ->
        [schoolyard] karte šolskega dvorišča
        [forest] gozdne karte
       *[other] sprint karte
    } ({ $area } ha).
permits-section-admin = Upravne enote
permits-section-protected = Zavarovana območja (BfN)
permits-section-osm-protected = Dodatna zavarovana območja iz OpenStreetMap
permits-section-operators = Upravljavci in lastniki po OpenStreetMap
permits-section-hints = Namigi
permits-section-sources = Viri in datum podatkov
permits-none-admin = VG250 za območje karte ne navaja nobene upravne enote.
permits-none-protected = Po BfN na območju karte ni zavarovanega območja.
permits-none-osm-protected = OpenStreetMap ne navaja zavarovanega območja na območju karte, ki ga BfN ne bi že navedel.
permits-none-operators = OpenStreetMap za območje karte ne navaja upravljavca ne lastnika.
permits-none-hints = Ni namigov.
permits-section-not-read = Podatki { $source } niso bili prebrani, zato je ta razdelek prazen, čeprav vnosi morda obstajajo.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Upravna območja 1:250 000 (VG250) urada BKG
        [bfn] Zavarovana območja v Nemčiji urada BfN
       *[osm] OpenStreetMap, iz izvlečka, ki ga freemapper hrani v predpomnilniku
    }
permits-level = { $level ->
        [land] Zvezna dežela (Land)
        [kreis] Okraj (Kreis)
        [verwaltungsgemeinschaft] Upravna skupnost (Amt ali Verwaltungsgemeinschaft)
       *[gemeinde] Občina (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Dotakne se roba (manj kot 0,5 % območja karte):
permits-at-edge-generalised = VG250 je posplošen na merilo 1:250 000. Enota, ki se le dotakne roba, je lahko posledica tega.
permits-kind = { $kind ->
        [nationalpark] Narodni park
        [nationales-naturmonument] Nacionalni naravni spomenik
        [naturschutzgebiet] Naravni rezervat (Naturschutzgebiet)
        [ffh] Območje FFH (Natura 2000)
        [vogelschutzgebiet] Območje zaščite ptic (Natura 2000)
        [biosphaerenreservat] Biosferni rezervat
        [biosphaerenreservat-zone] Cona biosfernega rezervata
        [landschaftsschutzgebiet] Krajinsko zaščiteno območje (Landschaftsschutzgebiet)
       *[naturpark] Naravni park
    }
permits-protected-line = { $kind } „{ $name }“, { $share }
permits-protected-date = Razglašeno { $date }
permits-protected-title = Pravna podlaga: { $title }
permits-protected-link = Pravno besedilo: { $url }
permits-protected-site-link = Stran območja (Natura 2000): { $url }
permits-protected-year = Leto podatkov pri BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = razred zaščite { $class }
permits-quoted = „{ $text }“
permits-osm-unnamed = brez imena
permits-osm-operator = Upravljavec po OSM: { $operator }
permits-osm-enclosing-gap = Območja OpenStreetMap, ki obkrožajo celotno območje karte brez vogala znotraj njega, tu lahko manjkajo, prav tako zavarovana območja, ki so v OSM vrisana kot relacija s `type=boundary`. Deli iz BfN in VG250 tega ne zadeva.
permits-osm-enclosing-gap-short = Tu lahko manjkajo območja OpenStreetMap, ki obkrožajo celotno območje karte, a nobeno njihovo oglišče ni v njem.
permits-operator-line = { $name } ({ $roles ->
        [operator] upravljavec
        [owner] lastnik
       *[both] upravljavec in lastnik
    }): { $objects }
permits-column-source = Vir
permits-column-state = Stanje
permits-column-credit = Zasluge in licenca
permits-state-read = prebrano, preneseno { $date }
permits-state-read-osm = prebrano, izvleček OSM z dne { $date }
permits-state-unreachable = ni dosegljivo ({ $reason })
permits-state-not-read = ni prebrano
permits-no-credit = ni uporabljeno
permits-hint-city-state = Območje karte leži v Berlinu ali Hamburgu. Tam je običajno pristojen Bezirksamt (okrajni urad) ali organ zvezne dežele. Ti napotki Bezirka še ne določijo.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Zavarovana območja sodijo v pristojnost spodnjega organa za varstvo narave (untere Naturschutzbehörde). Tu je to samo mesto { $kreis }.
        [no] Zavarovana območja sodijo v pristojnost spodnjega organa za varstvo narave (untere Naturschutzbehörde), običajno { $kreis }.
       *[unknown] Zavarovana območja sodijo v pristojnost pristojnega spodnjega organa za varstvo narave (untere Naturschutzbehörde), običajno Landkreis ali kreisfreie Stadt.
    }
permits-hint-national-park = Narodni park ima lastna pravila. Zgodaj vprašaj upravo narodnega parka.
permits-hint-nsg-paths = V naravnem rezervatu se je običajno treba držati poti. Za tek izven poti vprašaj organ za varstvo narave.
permits-hint-natura2000 = Območje FFH ali območje zaščite ptic (Natura 2000) posega na območje karte. Za prireditev lahko organ za varstvo narave zahteva oceno vplivov.
permits-hint-protected-areas-pl = Na območju karte ležijo zavarovana območja ali objekti varstva narave. Pri narodnem parku, krajinskem parku ali naravnem rezervatu je običajni stik uprava parka ali rezervata; za druga območja in objekte običajno pove, kdo je pristojen, Regionalna direkcija za varstvo okolja (RDOŚ) vojvodine. Vprašajte pravočasno, če tam načrtujete prireditev.
permits-hint-order-office = { $amt ->
        [none] Za ulice, trge in drugo javno zemljišče je običajno pristojen Ordnungsamt (urad za javni red) občine { $gemeinde }.
       *[other] Za ulice, trge in drugo javno zemljišče je običajno pristojen Ordnungsamt (urad za javni red): { $amt }, za občino { $gemeinde }.
    }
permits-hint-park-operator = Upravljavec parka ali vrta na območju karte po OpenStreetMap: { $operator }. Vprašaj ga, preden območje uporabiš.
permits-hint-school-operator = Upravljavec šole ali ustanove po OpenStreetMap: { $operator }. Za njeno zemljišče vprašaj vodstvo šole ali ta organ.
permits-section-districts = Bezirke (mestni okraji, { $source })
permits-section-green-spaces = Zelene površine ({ $source })
permits-section-owners = Zemljišča zvezne dežele in Bunda ({ $source })
permits-section-monuments = Spomeniki ({ $source })
permits-section-protected-objects = Naravni spomeniki in drugi zavarovani objekti ({ $source })
permits-none-districts = Po { $source } območje karte ne leži v nobenem Bezirku.
permits-none-green-spaces = Po { $source } na območju karte ni javne zelene površine.
permits-none-owners = Po { $source } na območju karte ni parcele zvezne dežele ali Bunda.
permits-none-monuments = Po { $source } na območju karte ni spomenika.
permits-none-protected-objects = Po { $source } na območju karte ni nobenega od zavarovanih objektov, ki jih vsebuje.
permits-none-sampled = V vprašanih točkah { $source } tu ne navaja ničesar.
permits-gaps = Podatki { $source } niso popolni: { $gaps }.
permits-sampled = { $source } odgovarja samo za posamezne točke. freemapper je vprašal na mreži točk čez karto: deleži so ocene, kar leži med točkami, lahko manjka.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = namen: { $dedication }
permits-playgrounds-none = Na območju karte ni javnega igrišča.
permits-playgrounds-one = Na območju karte je eno javno igrišče. Posebej ni navedeno.
permits-playgrounds-many = Javnih igrišč na območju karte: { $count }. Posamično niso navedena.
permits-owner-other = Drugi javni lastniki
permits-owners-rest = Za preostali del območja podatki ne navajajo lastnika.
permits-monuments-total = Spomeniki v celoti ali delno na območju karte: { $count }.{ $links ->
        [yes] {" "}Povezave vodijo do njihovih vnosov na seznamu spomenikov.
       *[no] {""}
    }
permits-monuments-links-left = Preostali vnosi ({ $count }) so tu samo prešteti, brez povezave.
permits-object-kind = { $kind ->
        [natural-monument] Naravni spomenik (Naturdenkmal)
        [protected-landscape-part] Zavarovan krajinski element (geschützter Landschaftsbestandteil)
        [biotope] Zakonsko zavarovan biotop (geschütztes Biotop)
        [national-park] Narodni park (park narodowy)
        [park-buffer-zone] Varovalni pas parka (otulina)
        [landscape-park] Krajinski park (park krajobrazowy)
        [nature-reserve] Naravni rezervat (rezerwat przyrody)
        [protected-landscape-area] Zavarovano krajinsko območje (obszar chronionego krajobrazu)
        [natura2000-habitats] Območje Natura 2000 za habitate (SOO)
        [natura2000-birds] Območje Natura 2000 za ptice (OSO)
        [ecological-site] Ekološko območje (użytek ekologiczny)
        [landscape-complex] Naravni in krajinski kompleks (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentacijsko najdišče (stanowisko dokumentacyjne)
        [natural-monument-pl] Naravni spomenik (pomnik przyrody)
        [national-park-cz] Narodni park (národní park, NP)
        [protected-landscape-area-cz] Zavarovano krajinsko območje (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Državni naravni rezervat (národní přírodní rezervace, NPR)
        [national-nature-monument] Državni naravni spomenik (národní přírodní památka, NPP)
        [nature-reserve-cz] Naravni rezervat (přírodní rezervace, PR)
        [nature-monument-cz] Naravni spomenik (přírodní památka, PP)
        [protective-zone-cz] Varstveni pas (ochranné pásmo)
        [nature-zone-cz] Cona varstva narave (zóna ochrany přírody)
        [natura2000-habitats-cz] Območje Natura 2000 za habitate (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Območje Natura 2000 za ptice (ptačí oblast, PO)
        [memorial-tree] Spomeniško drevo (památný strom)
       *[special-rules] Območje s posebnimi pravili
    }
permits-object-count = objektov: { $count }
permits-object-citations = Pravne podlage:
permits-source-note = { $source }: „{ $note }“
permits-hint-district-office = { $offices ->
        [hamburg] Za ulice, trge in zelene površine v Bezirku { $district } je običajno pristojen Bezirksamt { $district }, tam Fachamt Management des öffentlichen Raumes (urad za upravljanje javnega prostora).
       *[berlin] Za ulice, trge in zelene površine v Bezirku { $district } sta običajno pristojna Ordnungsamt (urad za javni red) ter Straßen- und Grünflächenamt (urad za ceste in zelenice) urada Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Za spomenike na območju karte je pristojen Denkmalschutzamt Hamburg (urad za varstvo spomenikov).
       *[berlin] Za spomenike v Bezirku { $district } je pristojen spodnji organ za varstvo spomenikov (untere Denkmalschutzbehörde) urada Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Podatki tu ne morejo zanesljivo povedati, ali na območju karte ležijo spomeniki s seznama.
       *[yes] Na območju karte so spomeniki s seznama.
    } Ali jih tek zadene, na primer s kontrolo pri spomeniku ali etapo skozi zavarovan objekt, odloča spodnji organ za varstvo spomenikov (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Tu je to samo mesto { $kreis }.
        [no] , običajno { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; v nekaterih zveznih deželah je to občina { $gemeinde }.
        }
       *[unknown] {" "}(običajno Landkreis ali kreisfreie Stadt, v nekaterih zveznih deželah občina).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Naravni spomeniki, zavarovani krajinski elementi in zavarovani biotopi na karti sodijo v pristojnost spodnjega organa za varstvo narave (untere Naturschutzbehörde). Tu je to samo mesto { $kreis }.
        [no] Naravni spomeniki, zavarovani krajinski elementi in zavarovani biotopi na karti sodijo v pristojnost spodnjega organa za varstvo narave (untere Naturschutzbehörde), običajno { $kreis }.
       *[unknown] Naravni spomeniki, zavarovani krajinski elementi in zavarovani biotopi na karti sodijo v pristojnost spodnjega organa za varstvo narave (untere Naturschutzbehörde), običajno Landkreis ali kreisfreie Stadt.
    }
permits-hint-biotope = Zakonsko zavarovani biotopi ležijo na območju karte{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Vse, kar jih lahko uniči ali bistveno poškoduje, je prepovedano (§ 30 BNatSchG). Kontrole in etape postavi zunaj njih ali vprašaj spodnji organ za varstvo narave (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Za naravne spomenike in zavarovane krajinske elemente v Bezirku { $district } je običajno pristojen Bezirksamt { $district }.
       *[berlin] Za naravne spomenike in zavarovane krajinske elemente v Bezirku { $district } je pristojen spodnji organ za varstvo narave (untere Naturschutzbehörde) urada Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Po odprtih podatkih zvezne dežele { $share } območja karte pripada drugim javnim lastnikom.
       *[no] Po odprtih podatkih zvezne dežele { $share } območja karte pripada lastniku „{ $owner }“.
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Lastnikov preostalega dela območja ni mogoče ugotoviti iz odprtih podatkov. Z upravičenim interesom (na primer kot organizator) lahko pri zemljiškoknjižnem uradu (Grundbuchamt) zaprosiš za izpisek; za javno zemljišče pomaga pristojni Ordnungsamt ali urad za parke.
       *[no] Lastnikov teh območij ni mogoče ugotoviti iz odprtih podatkov. Z upravičenim interesom (na primer kot organizator) lahko pri zemljiškoknjižnem uradu (Grundbuchamt) zaprosiš za izpisek; za javno zemljišče pomaga pristojni Ordnungsamt ali urad za parke.
    }
permits-section-forest-units = Gozdarska uprava ({ $source })
permits-section-state-forest = Državni gozd ({ $source })
permits-none-forest-units = Po { $source } območje karte ne leži na območju nobenega gozdarskega urada.
permits-section-admin-units = Upravne enote ({ $source })
permits-none-admin-units = Po podatkih { $source } območje zemljevida ne leži v nobeni upravni enoti registra.
permits-section-register-monuments = Spomeniki nacionalnega registra ({ $source })
permits-none-register-monuments = Po podatkih { $source } v pravokotniku okoli območja zemljevida ni nobenega vpisanega spomenika.
permits-register-monuments-total = Vpisani spomeniki v pravokotniku okoli območja zemljevida: { $count }. Register ne navaja lege, zato ti zapiski ne morejo povedati, kje so; to je treba preveriti na zemljevidu.
permits-register-monuments-left = Še { $count } jih je tu samo preštetih, brez imen.
permits-section-register-archaeology = Arheološki spomeniki nacionalnega registra ({ $source })
permits-none-register-archaeology = Po podatkih { $source } v pravokotniku okoli območja zemljevida ni nobenega vpisanega arheološkega spomenika.
permits-register-archaeology-total = Vpisani arheološki spomeniki v pravokotniku okoli območja zemljevida: { $count }. Register ne navaja lege, zato ti zapiski ne morejo povedati, kje so; to je treba preveriti na zemljevidu.
permits-register-archaeology-left = Še { $count } jih je tu samo preštetih, brez imen.
permits-section-register-history = Spomeniki zgodovine ({ $source })
permits-none-register-history = Po podatkih { $source } v pravokotniku okoli območja zemljevida ni nobenega spomenika zgodovine.
permits-register-history-in-area = Spomeniki zgodovine na območju zemljevida: { $count }.
permits-register-history-nearby = Nadaljnji spomeniki zgodovine v pravokotniku okoli območja zemljevida, vendar ne na območju zemljevida: { $count }.
permits-register-history-left = Še { $count } jih je tu samo preštetih, brez imen.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Po { $source } na območju karte ni državnega gozda.
permits-forest-units-reach = Gozdarski urad je pristojen za gozd na svojem območju. Njegovo območje lahko zajema tudi mesta in polja.
permits-state-forest-office = pristojen: { $office }
permits-hint-forest-office = { $units ->
        [none] Gozdarski urad, pristojen za gozd na območju karte: { $office } ({ $administration }). Če načrtuješ prireditev v gozdu, vprašaj tam zgodaj.
       *[other] Gozdarski urad, pristojen za gozd na območju karte: { $office } ({ $administration }), { $units }. Če načrtuješ prireditev v gozdu, vprašaj tam zgodaj.
    }
permits-hint-forest-owner = { $office ->
        [known] Za prireditev v gozdu je potrebno soglasje lastnikov gozda. Gozdarski urad običajno lahko pove, kdo je lastnik gozda. V več zveznih deželah je treba organizirano prireditev v gozdu tudi prijaviti gozdarskemu organu ali pridobiti njegovo dovoljenje.
       *[unknown] Za prireditev v gozdu je potrebno soglasje lastnikov gozda. V več zveznih deželah je treba organizirano prireditev v gozdu tudi prijaviti gozdarskemu organu ali pridobiti njegovo dovoljenje. Ti napotki ne navajajo pristojnega gozdarskega urada; povesta ti lahko gozdarska uprava zvezne dežele ali spodnji gozdarski organ.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Prireditev v gozdu običajno potrebuje soglasje lastnika gozda. Zgoraj navedena gozdna uprava (Nadleśnictwo) običajno lahko pove, kdo gozd upravlja. Gozda, ki ne pripada državnim gozdovom, na primer zasebnega gozda, v podatkih teh opomb ni. Gozd, ki ni v lasti države, na primer zasebni ali občinski gozd, je glede gospodarjenja z gozdom pod nadzorom starosta (starosta) (zakon o gozdovih, čl. 5 ust. 1 pkt 2); vprašajte tam na uradu (starostwo powiatowe).
       *[unknown] Prireditev v gozdu običajno potrebuje soglasje lastnika gozda. Te opombe za območje karte ne navajajo nobene gozdne uprave državnih gozdov; podatki zajemajo samo državne gozdove, zato gozd lahko pripada komu drugemu, na primer zasebnemu lastniku. Kdo je lastnik, je treba ugotoviti posebej. Gozd, ki ni v lasti države, na primer zasebni ali občinski gozd, je glede gospodarjenja z gozdom pod nadzorom starosta (starosta) (zakon o gozdovih, čl. 5 ust. 1 pkt 2); vprašajte tam na uradu (starostwo powiatowe).
    }
permits-hint-monument-register-pl = V pravokotniku okoli območja zemljevida so spomeniki nacionalnega registra. Njihove lege ni v podatkih, zato na zemljevidu preverite, kje so. Po čl. 36 odst. 1 zakona o varstvu spomenikov (ustawa o ochronie zabytków) konservatorska, restavratorska in gradbena dela na vpisanem spomeniku zahtevajo dovoljenje vojvodinskega konservatorja spomenikov (Wojewódzki Konserwator Zabytków). Od 3. junija 2026 gradbena dela v okolici spomenika, sprememba njegove rabe, nanj pritrjene naprave ali table in druga dejanja, ki bi lahko spremenila njegovo substanco ali videz, zahtevajo le prijavo konservatorju (čl. 36aa); ta lahko v 60 dneh ugovarja in nato zahteva dovoljenje. Zakon teka ali kontrolne točke kot takih ne omenja; vprašajte pisarno konservatorja, če se prireditev dotika spomenika ali njegove okolice.
permits-hint-archaeology-register-pl = Arheološki spomeniki nacionalnega registra ležijo v pravokotniku okoli območja zemljevida. Njihove lege ni v podatkih, zato na zemljevidu preverite, kje so. Po čl. 36 odst. 1 zakona o varstvu spomenikov (ustawa o ochronie zabytków) arheološke raziskave (pkt 5), iskanje skritih ali opuščenih premičnih spomenikov z elektronskimi ali tehničnimi napravami, kot so detektorji kovin (pkt 12), ter konservatorska, restavratorska ali gradbena dela na vpisanem spomeniku (pkt 1) zahtevajo dovoljenje vojvodinskega konservatorja spomenikov (Wojewódzki Konserwator Zabytków). Dovoljenje za iskanje preneha 1. januarja 2027: čl. 36b takrat dovoljuje polnoletni osebi iskanje s soglasjem lastnika in posestnika nepremičnine po prijavi v register iskanj prek aplikacije ministrstva za kulturo, čl. 36c pa takšna iskanja med drugim prepoveduje na vpisanih spomenikih in manj kot 10 m od njih. Zakon ne našteva sprehoda, teka ali kontrolne točke kot take; vprašajte urad konservatorja, če prireditev ali terensko delo zadeva vpisano najdišče.
permits-hint-history-register-pl = Spomenik zgodovine (Pomnik historii) leži na območju zemljevida ali blizu njega; razdelek o spomenikih zgodovine pove, kateri. Predsednik Republike Poljske z uredbo lahko razglasi za Pomnik historii nepremični spomenik, vpisan v register, ali kulturni park posebne vrednosti (čl. 15 odst. 1 zakona o varstvu spomenikov, ustawa o ochronie zabytków). Zato je že pod varstvom registra ali kulturnega parka. Za dela na spomeniku, vpisanem v register, mora vojvodinski konservator spomenikov (Wojewódzki Konserwator Zabytków) izdati dovoljenje (čl. 36 odst. 1); v kulturnem parku prepovedi in omejitve določa sklep sveta gmine (čl. 16 odst. 2, čl. 17 odst. 1). Zakon ne navaja sprehoda, teka ali kontrolne točke kot takih; vprašajte urad konservatorja, če prireditev ali terensko delo zadeva spomenik.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Na območju zemljevida leži naravni spomenik (pomnik przyrody), ekološko območje (użytek ekologiczny), naravni in krajinski kompleks (zespół przyrodniczo-krajobrazowy) ali dokumentacijsko najdišče (stanowisko dokumentacyjne). Ustanovi jih svet gmine s sklepom, ki določa, kdo nadzoruje objekt in kateri prepovedi veljajo (člen 44 zakona o varstvu narave). Zakon sam tu za prireditev ne predvideva nobenega dovoljenja. Zaprosite za sklep in določitev nadzornika pri uradu gmine, na katere območju objekt leži.
       *[other] Na območju zemljevida leži naravni spomenik (pomnik przyrody), ekološko območje (użytek ekologiczny), naravni in krajinski kompleks (zespół przyrodniczo-krajobrazowy) ali dokumentacijsko najdišče (stanowisko dokumentacyjne). Ustanovi jih svet gmine s sklepom, ki določa, kdo nadzoruje objekt in kateri prepovedi veljajo (člen 44 zakona o varstvu narave). Zakon sam tu za prireditev ne predvideva nobenega dovoljenja. Zaprosite za sklep in določitev nadzornika { $count ->
        [one] pri uradu gmine ({ $gmina }), na katere območju objekt leži
       *[other] pri uradih gmin ({ $gmina }), pri vsakem za objekte na njegovem območju
    }.
    }
permits-hint-gmina-roads-pl = Prireditev, ki ovira promet ali na poseben način uporablja ulico, potrebuje dovoljenje po členu 65 zakona o cestnem prometu; prošnja se vloži vsaj 30 dni vnaprej. Dovoljenje izda organ, ki upravlja promet na cesti: starosta za ceste powiata in gmine, župan v mestu s pravicami powiata, maršal vojvodine ali GDDKiA za druge ceste (člena 10 in 65b). Upravljavec ceste gmine je wójt, burmistrz ali { $count ->
        [one] prezydent miasta gmine ({ $gmina })
       *[other] prezydent miasta vsake od gmin ({ $gmina })
    }; upravljavec odloča o zasedbi cestnega pasu (člena 19 in 40 zakona o javnih cestah). Ti zapiski ne kažejo kategorije cest.
permits-hint-gmina-land-pl = Če zemljišče na območju zemljevida pripada gmini, na primer mestni park, trg, šolsko zemljišče ali gozd, ga upravlja wójt, burmistrz ali prezydent miasta, ki gmino zastopa (člena 30 in 31 zakona o samoupravi gmine). Pred uporabo zaprosite za soglasje { $count ->
        [one] urad gmine ({ $gmina }).
       *[other] urade gmin ({ $gmina }).
    } Ti zapiski ne kažejo, čigavo je zemljišče.
permits-hint-protected-areas-cz = Na območju zemljevida so zavarovana območja ali objekti varstva narave. Po zakonu o varstvu narave in krajine (zakon št. 114/1992 Sb.) so v narodnem parku prepovedane organizirane športne, turistične in druge javne prireditve zunaj krajev, ki jih je določil organ za varstvo narave (§ 16 odst. 1 písm. q), v varstvenem pasu narodnega parka pa takšne množične prireditve zunaj teh krajev in zunaj pozidanih območij zahtevajo soglasje organa (§ 37 odst. 3 písm. d). V državnem naravnem rezervatu je prepovedano vstopati ali voziti zunaj poti, označenih s soglasjem organa (§ 29 písm. d). Organi za varstvo narave so med drugim uprava narodnega parka, Agencija za varstvo narave (AOPK ČR), regionalni urad (krajský úřad) in urad občine z razširjenimi pristojnostmi (§ 75 odst. 1). Druga zavarovana območja in objekti imajo lastne varstvene pogoje. Če tam načrtujete prireditev, se pravočasno obrnite na organ.
permits-hint-monuments-cz = Na območju zemljevida so kulturni spomeniki ali spomeniška območja z državnega seznama. Po zakonu o državnem varstvu spomenikov (zakon št. 20/1987 Sb., § 14 odst. 1 in 2) mora lastnik spomenika pred vzdrževanjem, popravilom, rekonstrukcijo ali drugim posegom v spomenik ali njegovo okolico vnaprej zaprositi za odločbo ali zavezujoče mnenje. Enako velja za lastnika nepremičnine v spomeniškem rezervatu, spomeniški coni ali varstvenem pasu spomenika pri gradnji, spremembi stavbe, zemeljskih delih, napravi ali tabli, odstranitvi objekta, oskrbi dreves ali vzdrževalnih delih. Odloča urad občine z razširjenimi pristojnostmi{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, pri državnem kulturnem spomeniku regionalni urad. Zakon sprehoda, teka ali kontrolne točke kot takih ne omenja; vprašajte ta urad, če se prireditev dotika spomenika ali če na njem postavlja naprave ali table.
permits-hint-forest-owner-cz = Množično športno, kulturno ali drugo družbeno prireditev na zemljišču, namenjenem funkcijam gozda, je dovoljeno izvesti le s pisnim soglasjem lastnika gozda, v katerem poteka (zakon o gozdovih, zakon št. 289/1995 Sb., § 20 odst. 5). Organizator mora prireditev tudi pisno prijaviti uradu občine z razširjenimi pristojnostmi{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, ali regionalnemu uradu, če prireditev sega čez območje take občine, najpozneje 30 dni pred prireditvijo. Prijava navaja datum in kraj, kako je prireditev zagotovljena, pričakovano število udeležencev in pisna soglasja lastnikov (isti odstavek). Ti zapiski ne kažejo, kdo je lastnik gozda; lastnike gozda vprašajte pravočasno.
permits-hint-roads-cz = Prireditev, ki cesto uporablja drugače kot običajno, na primer športna, kulturna ali podobna prireditev, ki bi lahko ogrozila varnost ali pretočnost cestnega prometa, potrebuje dovoljenje pristojnega urada za ceste. Izda se z vnaprejšnjim soglasjem lastnika ceste in, kjer bi bila lahko prizadeta varnost ali pretočnost prometa, Policije Češke republike (zakon o cestah, zakon št. 13/1997 Sb., § 25 odst. 1 in odst. 6 písm. d). Urad je občinski urad za lokalne ceste ali urad občine z razširjenimi pristojnostmi, kjer ta odloča, za ceste druge in tretje kategorije pa urad občine z razširjenimi pristojnostmi (§ 40 odst. 4 in 5). Občine na območju zemljevida: { $gmina }. Ti zapiski ne kažejo kategorije cest.
permits-section-water-protection = Vodovarstvena območja ({ $source })
permits-section-forest-functions = Gozdne funkcije in varovalni gozd ({ $source })
permits-none-water-protection = Po { $source } na območju karte ni vodovarstvenega območja.
permits-none-forest-functions = Po { $source } na območju karte ni rekreacijskega gozda, varovalnega gozda, naravnega gozda ali zaprtega gozdnega območja.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreacijski gozd (Erholungswald)
        [protected] Varovalni gozd (Bannwald ali Schutzwald)
        [natural] Naravni gozd ali strogi rezervat (Naturwald, Totalreservat)
       *[closed] Zaprto gozdno območje
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } vodovarstvenega območja „{ $name }“ leži na območju karte. V conah I in II so prireditve običajno izključene. Vprašaj spodnji vodni organ (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } vodovarstvenega območja „{ $name }“ leži na območju karte. Prireditve tam so običajno mogoče; spodnji vodni organ (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) lahko pove, kaj dovoljuje uredba.
       *[unknown] Vodovarstveno območje „{ $name }“ leži na območju karte. Podatki ne povedo, katera cona je to. V conah I in II so prireditve običajno izključene; vprašaj spodnji vodni organ (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Rekreacijski gozd (Erholungswald) leži na območju karte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Za prireditve tam vprašaj gozdarski organ.
        [protected] Varovalni gozd (Bannwald ali Schutzwald) leži na območju karte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Preden tam načrtuješ prireditev, vprašaj gozdarski organ.
        [natural] Naravni gozd ali strogi rezervat leži na območju karte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Zapuščanje poti je tam pogosto prepovedano; brez vprašanja gozdarskega organa vanj ne postavljaj kontrol.
       *[closed] Gozdno območje, kamor je vstop prepovedan, leži na območju karte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Izogni se mu pri progi.
    }
permits-section-roads = Uvrščene ceste po OpenStreetMap
permits-none-roads = Po OpenStreetMap skozi območje karte ne poteka nobena Autobahn ali Bundes-, Landes-, Staats- ali Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (avtocesta)
        [bundesstrasse] Bundesstraße (zvezna cesta)
        [landesstrasse] Landesstraße ali Staatsstraße (deželna cesta)
       *[kreisstrasse] Kreisstraße (okrajna cesta)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } poteka skozi območje karte. Proga jo sme prečkati samo po mostovih ali podhodih.
        [bundesstrasse] Bundesstraße { $refs } poteka skozi območje karte. Zaporo ali prečkanje z reditelji odredi organ za cestni promet (Straßenverkehrsbehörde); cesto običajno vzdržuje Bund prek cestne uprave zvezne dežele, v večjih mestih pogosto mesto.
        [landesstrasse] Landes- ali Staatsstraße { $refs } poteka skozi območje karte. Zaporo ali prečkanje z reditelji odredi organ za cestni promet (Straßenverkehrsbehörde); cesto običajno vzdržuje zvezna dežela, v večjih mestih pogosto mesto.
       *[kreisstrasse] Kreisstraße { $refs } poteka skozi območje karte. Zaporo ali prečkanje z reditelji odredi organ za cestni promet (Straßenverkehrsbehörde); cesto običajno vzdržuje Kreis, v večjih mestih pogosto mesto.
    }
permits-hint-state-forest = { $office ->
        [none] Po odprtih podatkih zvezne dežele je { $share } območja karte državni gozd lastnika „{ $owner }“.
       *[other] Po odprtih podatkih zvezne dežele je { $share } območja karte državni gozd lastnika „{ $owner }“. Pristojen: { $office }.
    }
