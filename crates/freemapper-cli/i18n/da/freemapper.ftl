# Program texts of freemapper, Danish (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } er ikke et sprog, freemapper har tekster til; tilgængelige: { $available }

## init

cli-part-needs-osm = --part vælger en del af et OSM-område og kræver --osm; ved --place skal du først slå id'et op og bruge --osm <id> --part <n>
cli-reading-file = læser { $path }
cli-no-area-for-place = intet område fundet for "{ $query }". Hvis stedet er et område med flere adskilte dele, vælg en med --osm <id> --part <n>, eller brug --geojson
cli-using-place = Bruger { $name } ({ $id })
cli-created-project = Projekt oprettet: { $path }

## Cache

cli-no-cache-dir = ingen cachemappe på dette system; angiv --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Hentede 1 flise
       *[other] Hentede { $count } fliser
    }
cli-wrote = Skrev { $path }
cli-kept-map = Beholdt { $path } (den kan indeholde dit arbejde)
cli-map-lacks-templates = Kortet henviser endnu ikke til billedskabelonerne. Tilføj dem i OOM (Templates > Open template…) eller erstat kortet med --force, som beholder det gamle som .omap.bak.
cli-map-lacks-buildings = Kortet har endnu ingen ALKIS-bygninger. For at tilføje dem skal du erstatte det med --force, som beholder det gamle som .omap.bak.
cli-map-lacks-canopies = Kortet mangler ALKIS-overdækninger og carporte (522), som nu er kortobjekter. For at tilføje dem skal du køre med --force, som beholder det gamle kort som .omap.bak.
cli-map-light = Basiskortet: { $joined } linjer sat sammen, { $left } skjulte eller for små efterladt til kandidatlagene (intet flyttet)
cli-map-light-kept = Kandidatlagene indeholder nu også { $left } objekter eller strækninger, som denne version tager af basiskortet (mærket freemapper:generalized). Det beholdte kort har dem stadig, så de vises dobbelt. Erstat kortet med --force for at rydde op (det gamle bliver som .omap.bak).
cli-map-old-candidates = Kortet nævner stadig templates/candidates.omap; denne kørsel fyldte den med alle kandidater. Kandidaterne findes nu også i syv lag, en fil hver (templates/candidates-*.omap). Kortet viser dem, når du erstatter det med --force (det gamle bliver som .omap.bak), eller du tilføjer dem selv i OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF uden kildeangivelser: ODbL og dl-de/by-2-0 kræver, at du nævner kilderne, når du giver kortet videre; se ATTRIBUTION.txt
cli-wrote-pdf = Skrev { $path } (kortet uden kandidater)
cli-wrote-pdf-candidates = Skrev { $path } (kontroludskriften: kortet med alle kandidater, positionstro; kontrollér dem i terrænet)
cli-wrote-training = Skrev { $map } og { $pdf } (træningskortet: { $joined } sat sammen, { $shifted } flyttet, { $left } udeladt eller skåret)
cli-training-free-ends = Linjeender efterladt frie: { $ways } hvor en vej mødte en anden vej, { $others } ved andre linjer
cli-training-apart-left = træningskortet har stadig { $length } mm (på papiret), hvor objekter holder for lidt afstand; kontrollér det i OOM
cli-kept-training = Beholdt træningskortet { $path }, så en kørsel ikke overskriver et, der er printet eller givet videre; slet det eller angiv --force for at fornye det

## OCAD

cli-ocad-kept = Beholdt { $path } (den kan indeholde dit arbejde; --force erstatter den og beholder den gamle som .ocd.bak)
cli-ocad-old-candidates = OCAD-kortet nævner stadig { $path } som baggrundskort; denne kørsel fyldte den med alle kandidater. Kandidaterne findes nu også i syv lag, en fil hver (-candidates-*.ocd). OCAD-kortet viser dem, når du erstatter det med --force (det gamle bliver som .ocd.bak), eller du tilføjer dem selv i OCAD som baggrundskort.
cli-ocad-wrote = { $count ->
        [one] Skrev { $path } (1 objekt)
       *[other] Skrev { $path } ({ $count } objekter)
    }
cli-ocad-left-out = Udeladt fra OCAD-kortet: { $skipped } objekter (intet symbol til OCAD, intet areal eller uden for OCAD's tegneområde på ±2 m) og { $dropped } degenererede linjedele eller huller.
cli-ocad-load-symbols = Vælg i OCAD Symbol > Load Colors and Symbols From… og vælg det officielle symbolsæt{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF-skolesættet 2019
       *[other] {" "}ISSprOM 2019
    }; det erstatter freemappers pladsholdersymboler efter nummer.

## select

cli-select-running = Områdevalg for { $path }: tryk "Færdig" på siden eller Ctrl+C for at stoppe.
cli-select-no-browser = Kunne ikke åbne browseren ({ $error }); åbn adressen ovenfor.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; intet område, kan ikke være et kortområde): { $place }
cli-find-hint = Opret projektet med: freemapper init <dir> --osm { $id }
cli-find-no-area = Ingen af disse steder har et område, der kan være et kortområde. Tegn området med: freemapper select <dir>
cli-find-nothing = intet sted fundet for "{ $query }"; prøv et andet navn, eller tegn området med: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = advarsel: adapteren i { $folder } er udeladt: { $finding }
cli-adapter-note = advarsel: adapteren i { $folder } indlæses, men: { $finding }
cli-adapter-from-project = advarsel: adapteren { $adapter } fra projektmappen er i brug; den kan spørge disse værter: { $hosts }
cli-adapter-overlaps = advarsel: adapteren { $adapter } dækker også område fra { $others }; dens objekter kommer på kort dér
cli-adapter-dir-relative = advarsel: FREEMAPPER_ADAPTER_DIR er ikke en absolut sti ({ $path }); den ignoreres
cli-adapters-no-project = advarsel: { $path } er ikke en mappe; ingen projektadaptere blev læst
cli-adapters-export-unknown = adapteren { $adapter } er ikke beskrevet af et manifest i dette program, så der er intet at eksportere; `freemapper adapters list` viser id'erne
cli-adapters-export-taken = { $path } findes og er ikke tom; eksportér til en mappe, der ikke findes endnu
cli-adapters-export-done = Skrev de { $count } filer for adapteren { $adapter } til { $path }. Der står kun det, manifestet beskriver; adresser i programmets kode er ikke med.
cli-adapters-export-next = Ret adresserne i manifest.toml, tjek mappen med `freemapper adapters check`, og flyt den derefter til { $path }: Fra næste start erstatter den den indbyggede adapter, med en advarsel. Sletter du mappen, er den indbyggede tilbage.
cli-adapters-overlaps = deler område med: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revision { $revision }  kontrolleret { $checked }  værter: { $hosts }
cli-adapters-source-built-in = indbygget
cli-adapters-source-user = fra din mappe
cli-adapters-source-project = fra projektet
cli-adapters-never-checked = aldrig
cli-adapters-user-dir = Dine adaptere læses fra { $path }
cli-adapters-no-user-dir = Dette system har ingen mappe til dine adaptere; FREEMAPPER_ADAPTER_DIR angiver en

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper må nu også spørge { $host } for { $name }. Indgangen står i { $file }; `freemapper adapters disallow-host { $adapter } { $host }` tager den tilbage. En områdeside eller en AI-assistent, der kører, læser indgangen, når den startes igen
cli-allow-host-already = { $host } er allerede tilladt for { $name }
cli-allow-host-declared = adapteren for { $name } nævner selv { $host }; intet at tillade
cli-allow-host-refused = { $host } kan ikke tillades: { $reason }
cli-allow-host-unknown-adapter = der er ingen indbygget adapter { $adapter }; de indbyggede adaptere er: { $adapters }
cli-allow-host-folder-adapter = { $adapter } er en adapter i din adaptermappe; tilføj i stedet værten til `[net] hosts` i dens manifest
cli-allow-host-no-config = dette system har ingen konfigurationsmappe til at gemme tilladte værter i
cli-allow-host-write = { $file } kunne ikke skrives: { $detail }
cli-allow-host-unreadable = { $file } kan ikke læses ({ $detail }); ret eller fjern filen først; intet blev ændret
cli-disallow-host-removed = freemapper spørger ikke længere { $host } for { $name }
cli-disallow-host-absent = { $host } var ikke tilladt for { $name }; intet at tage tilbage
cli-allowed-hosts-unreadable = advarsel: { $file } kunne ikke læses ({ $detail }); ingen ekstra værter er tilladt
cli-allowed-hosts-ignored = advarsel: { $file }: indgangen { $host } for { $adapter } ignoreres: { $reason }
cli-allowed-hosts-no-adapter = freemapper har ingen indbygget adapter med det navn
cli-allowed-host-redundant = advarsel: { $host } er nu en af de værter, adapteren for { $name } nævner; din indgang i { $file } blev fjernet
cli-adapters-allowed = tilladt af dig: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revision { $revision }, værter: { $hosts }
cli-check-error = fejl: { $finding }
cli-check-warning = advarsel: { $finding }
cli-check-passed = { $folder } består kontrollen
cli-check-passed-warnings = { $folder } består kontrollen, med { $count ->
        [one] én advarsel
       *[other] { $count } advarsler
    }
cli-check-failed = { $folder } består ikke kontrollen
cli-check-no-folder = der er ingen mappe { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } FEJLEDE: { $detail }
cli-check-live-not-asked = { $capability } ikke spurgt: { $detail }
cli-check-live-box-outside = boksen ligger ikke inden for adapterens { $adapter } grænse; en live-kontrol spørger tjenesterne om et sted, adapteren dækker
cli-check-live-cannot = live-kontrollen kan ikke køre: { $detail }
cli-check-detail-not-declared = adapteren angiver ingen
cli-check-detail-stopped = en tidligere del fejlede
cli-check-detail-see-warnings = se advarslerne
cli-check-detail-outside-dataset = boksen ligger uden for datasættet
cli-check-detail-objects = { $count ->
        [one] ét objekt
       *[other] { $count } objekter
    }
cli-check-detail-layers = { $layers } lag, { $count } objekter
cli-check-detail-trees = { $count ->
        [one] ét træ
       *[other] { $count } træer
    }
cli-check-detail-extract = { $label }, data fra { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = terrænmodel
cli-check-raster-surface = overflademodel
cli-check-raster-tiles = { $product }: { $count ->
        [one] én flise
       *[other] { $count } fliser
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } og { $last }

## Folder lock

cli-lock-held = en anden freemapper-kørsel bruger denne mappe (siden { $time }, startet af { $who }). Vent på den, eller stop den først.
cli-lock-who-cli = terminalen
cli-lock-who-mcp = en AI-agent (MCP)
cli-lock-who-page = områdevalgssiden
cli-lock-who-unknown = et andet program
cli-lock-failed = kunne ikke låse projektmappen til denne kørsel: { $error }
cli-lock-stuck = noget andet end en låsefil er i vejen og kunne ikke fjernes

## Run routes of the selection page

cli-run-bad-request = Kørselsvalgene er ikke gyldige: { $error }
cli-run-no-project = Der er endnu intet gemt projekt i denne mappe. Gem området først.
cli-run-already-running = En kørsel ({ $job }) er allerede i gang for dette projekt. Vent, til den er færdig, eller annullér den først.
cli-run-disk-stops = Ikke nok ledig diskplads til hentningen: { $messages }
cli-run-busy-elsewhere = Et andet projekt i dette program kører en pipeline lige nu. Projekterne deler downloadcachen, så der kører kun én ad gangen. Prøv igen, når den er færdig.
cli-run-nothing-to-cancel = Ingen kørsel er i gang, så der er intet at annullere.
cli-run-unknown-job = Der er ingen kørsel { $job } for dette projekt.
cli-status-too-fast = Statusforespørgsler kommer for ofte. Spørg højst fem gange i sekundet.
cli-open-folder-failed = Mappen kunne ikke åbnes ({ $error }).
cli-open-folder-no-opener = Denne computer har intet program til at åbne mapper ({ $program }). Åbn mappen manuelt.
cli-open-folder-missing = Projektmappen findes endnu ikke. Gem området først.
cli-open-folder-refused = Mappenavnet er ikke sikkert at give videre til filhåndteringen.
cli-save-running = En kørsel er i gang for dette projekt. Gem igen, når den er færdig.
cli-save-unknown-kind = Ukendt korttype "{ $kind }". Brug schoolyard, sprint eller forest.
cli-save-scale-too-large = Målestokken { $scale } er for stor.
cli-save-bad-content = Indholdsindstillingerne er ikke gyldige. Intet blev gemt.
cli-map-image-none = Opret først kortet.
cli-map-image-gone = Denne forhåndsvisning er ikke længere tilgængelig.
cli-map-image-running = En kørsel er i gang for dette projekt. Forhåndsvisningen kan beregnes, når den er færdig.
cli-map-image-stale = Det gemte projekt svarer ikke længere til det genererede kort, for eksempel fordi området, korttypen eller målestokken er ændret. Start først en kørsel, der erstatter kortet ("Erstat det eksisterende kort" på siden, eller freemapper run --force).
cli-map-image-superseded = Der blev i mellemtiden bedt om en nyere forhåndsvisning, så denne blev ikke beregnet.
cli-map-image-bad-content = Indholdsindstillingerne er ikke gyldige.
cli-map-image-bad-request = Forespørgslen om en forhåndsvisning er ikke gyldig. Send et JSON-objekt, eventuelt med indholdsindstillingerne.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – områdevalg
page-heading = Vælg kortområdet
page-update-available = freemapper { $version } er tilgængelig. Opdatér med
page-update-notes = Udgivelsesnoter
page-folder = Projektmappe
page-pick = Søg efter et sted
page-query-placeholder = fx Clara-Zetkin-Schule Strausberg
page-search-button = Søg
page-searching = Søger …
page-no-results = Intet fundet.
page-point-only = kun et punkt, ikke et område
page-draw = Tegn selv
page-rectangle = Rektangel
page-polygon = Polygon
page-finish-polygon = Luk polygon
page-clear = Ryd
page-hint-rectangle = Klik på to modstående hjørner på kortet.
page-hint-polygon = Klik på hjørnerne efter hinanden. For at lukke den, klik på det første hjørne, dobbeltklik eller klik på "Luk polygon". Esc afbryder, Backspace fjerner det sidste hjørne.
page-hint-idle = Vælg et sted fra søgningen, eller tegn området selv.
page-summary = Område
page-source-osm = Fra OpenStreetMap: { $name }
page-source-drawn = Tegnet i hånden
page-size = Størrelse med buffer: { $width } × { $height } m
page-tiles-one = { $n } flise: { $names }
page-tiles-other = { $n } fliser: { $names }
page-download = Download: cirka { $mb } MB
page-download-none = Download: alt er allerede i cachen
page-download-details = Hvad der hentes
page-cached = i cachen
page-disk = Diskplads: { $volumes }
page-disk-volume = { $free } ledig, cirka { $needed } nødvendig
page-cache-volume = cache
page-project-volume = projektmappe
page-no-room = passer ikke på disken
page-no-room-skip = Ikke alt passer på disken (freemapper holder { $reserve } fri). Hentningen udelader de markerede dele. Frigør først plads, hvis du har brug for dem:
page-no-room-stop = Ikke nok diskplads (freemapper holder { $reserve } fri). Uden de markerede dele bliver der intet kort, så hentningen stopper, før den downloader noget. Frigør først plads:
page-disk-unknown = freemapper kan ikke læse den ledige diskplads ({ $error }). Den henter uden kontrollen.
page-too-large = Området er for stort: højst { $max } m pr. side (med buffer). Vælg et mindre.
page-outside = Området ligger uden for de understøttede delstater ({ $states }); freemapper har endnu ingen datakilder til andre regioner.
page-outside-unknown = Området ligger uden for de regioner, freemapper understøtter; der er endnu ingen datakilder dér.
page-adapters-failed = Luftfotos kan ikke vises i listen ({ $detail }). Kortet viser kun OpenStreetMap; alt andet virker.
page-settings = Kort
page-kind = Korttype
page-schoolyard = Skolegårdskort (1:1000)
page-sprint = Sprintkort (1:4000)
page-forest = Skovkort (1:15000)
page-scale = Målestok 1:
page-map-title = Titel
page-contour-interval = Ækvidistance
page-contour-auto = automatisk
page-steep-slope = Stejlhedsgrænse
page-steep-slope-auto = automatisk (27°)
page-steep-slope-hint = Fra denne hældningsvinkel regnes terrænet som stejlt, når der ledes efter skrænter og klipper. 27° (automatisk) passer til bakker, parker og byer. I klippelabyrinter, kløfter og brud er næsten alle skråninger stejle, så skrænter og klipper flyder sammen til én plet, og ingen bliver fundet; 55° til 65° adskiller dem. En for høj værdi beholder kun de stejleste vægge.
page-face-slope = Vægtærskel
page-face-slope-auto = automatisk (70°)
page-face-slope-hint = Fra denne hældningsvinkel tegnes terrænet som en sort klippevægsflade. 70° (automatisk) passer til klippelabyrinter som Adršpach (5,2 % af arealet). 65° giver flere og større sorte flader, også på stejle skråninger (8,4 %). 75° bevarer kun rigtige vægge, med flossede kanter (2,5 %). En for høj værdi levner næsten ingen flader.
page-settings-fixed = Projektet findes allerede; korttype, målestok og titel står i freemapper.toml. Ækvidistancen kan stadig ændres.
page-save = Gem
page-saving = Gemmer …
page-saved = Gemt i freemapper.toml.
page-zone-changed = Gemt. Området ligger nu i en anden UTM-zone; freemapper henter dets data igen ved næste kørsel.
page-provider-changed = Gemt. En anden dataleverandør leverer højde- og luftbilleddata for det nye område; freemapper henter dets data igen ved næste kørsel.
page-save-stale = freemapper.toml er ændret i et andet vindue eller af et andet program, siden denne side læste den. Intet blev gemt; siden kender nu den gemte tilstand. Gem igen for at erstatte den med det område, der vises her.
page-has-map = Det eksisterende kort (.omap) forbliver, som det er; freemapper laver nye skabeloner og kandidater til det nye område. Valgmuligheden "Erstat det eksisterende kort" nedenfor (eller "freemapper run --force") laver også kortet om og beholder det gamle som .omap.bak.
page-done = Færdig
page-source = Kildekode (AGPL-3.0-or-later)
page-closed = Færdig. Du kan lukke dette vindue.
page-next = Næste: freemapper run "{ $dir }"
page-made-files = Oprettet: { $files }. Mappe: { $dir }
page-back-to-agent = Sig til din AI-assistent, at området er gemt.
page-nothing-saved = Intet blev gemt.
page-failed = Fejl: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Luftfoto { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Hvad kommer på kortet
page-content-help = Profilen afgør, hvad der kommer direkte på kortet, og hvad der forbliver en kandidat til kontrol. Et valg for en gruppe vejer tungere end profilen. Det gemmes sammen med området.
page-content-profile-legend = Profil
page-content-profile-safe = Sikker
page-content-profile-safe-hint = Hvert objekt bliver, hvor opbygningen placerer det: bygninger og matrikelarealer kommer på kortet; mure, hegn, træer og meget fra OpenStreetMap (damme, stier uden kendt belægning) forbliver kandidater.
page-content-profile-balanced = Afbalanceret
page-content-profile-balanced-hint = Objekter fra matriklen, registrene og gadeopmålingen kommer på kortet, sammen med højdekurverne og de områder, der ikke må betrædes langs jernbanespor. Mure, hegn, hække, skrænter, trapper, vand, træer, monumenter, passager, stier og jernbaner kommer også fra OpenStreetMap. Gadeinventar, de øvrige forbudte områder og formerne fra terrænmodellen forbliver kandidater.
page-content-profile-full = Fuld
page-content-profile-full-hint = Alt kommer på kortet, som i kontroludskriften; kun skjulte kandidater bliver udeladt.
page-content-groups-legend = Efter gruppe
page-content-groups-help = "Som i profilen" følger profilen. Hvad det betyder for en gruppe, afhænger af, hvor hvert objekt kommer fra.
page-content-place-profile = Som i profilen
page-content-place-map = Kort
page-content-place-candidates = Kandidat
page-content-place-off = Fra
page-content-group-walls = Mure
page-content-group-fences = Hegn
page-content-group-hedges = Hække
page-content-group-slopes = Skrænter og klipper
page-content-group-stairs = Trapper
page-content-group-trees = Træer og trægrupper
page-content-group-water-features = Vand, springvand og kilder
page-content-group-monuments = Tårne, monumenter og elledninger
page-content-group-furniture = Gadeinventar og lygter
page-content-group-passages = Passager
page-content-group-contours = Højdekurver
page-content-group-landforms = Høje, fordybninger, skrænter og klipper fra terrænmodellen
page-content-group-vegetation = Vegetation fra laserpunktskyer (Brandenburg, Berlin, Nordrhein-Westfalen, Rheinland-Pfalz, Saarland og Bayern; fra, indtil du vælger den; downloader cirka 40 til 600 MB pr. km²)
page-content-group-out-of-bounds = Områder, der ikke må betrædes
page-content-group-paths = Stier og jernbaner
page-content-group-streets = Gader og pladser
page-run-heading = Opret kortet
page-preview-heading = Forhåndsvisning af kortet
page-preview-stored = Som sidst genereret.
page-preview-live = Med indstillingerne på denne side, endnu ikke gemt.
page-preview-live-saved = Med de gemte indstillinger, endnu ikke genereret i en kørsel.
page-preview-computing = Beregner …
page-preview-none = Intet kort genereret endnu.
page-preview-later = Dette kort har endnu ingen forhåndsvisning. Den kommer med den næste kørsel, der skriver kortet.
page-preview-stale = Området, korttypen eller målestokken er ændret, siden kortet blev genereret. Billedet viser stadig kortet, som det sidst blev genereret; en ny forhåndsvisning kræver en kørsel med "Erstat det eksisterende kort".
page-preview-area-unsaved = Området på denne side er endnu ikke gemt. Billedet viser kortet, som det sidst blev genereret; det nye område kræver en kørsel med "Erstat det eksisterende kort".
page-preview-run = Som sidst genereret. Siden opdaterer billedet, når kørslen slutter.
page-preview-open = Åbn i fuld størrelse
page-preview-alt = Forhåndsvisning af kortobjekterne
page-opt-ocad = Skriv et OCAD-startprojekt
page-help-ocad = Til alle, der arbejder videre i OCAD.
page-opt-pdf = PDF til udskrivning
page-help-pdf = Kun kortobjekter, ingen kandidater.
page-opt-pdf-candidates = Kontroludskrift med alle kandidater
page-opt-training = Træningskort
page-help-training = Kortet og alle kandidater, ryddet op, så det kan printes, som det er; med OCAD også som OCAD-fil.
page-views-heading = Visninger af træningskortet
page-view-no-paths = Uden stier (Ultimate-kort)
page-view-brown-only = Kun brunt (højdekurve-orientering)
page-view-no-north-lines = Uden nordlinjer
page-view-custom = En anden visning
page-help-views = Hver afkrydset visning er en fil mere ved siden af træningskortet, `<name>-training-<view>.omap` og dens PDF, med OCAD også en OCAD-fil. En anden visning er `no:505,506` (udelad symboler) eller `only:101,102` (behold kun disse); `+` kombinerer visninger. Titel, målestok og kilder forbliver altid.
page-help-views-forest = Uden stier og kun brunt gælder kun for skovkort.
page-opt-scale-bar = Skalalinje på PDF'erne
page-help-scale-bar = Tyske kort har som regel ingen.
page-opt-sources = Kildelinjer på PDF'erne
page-sources-auto = automatisk
page-sources-full = fuld
page-sources-credit-only = kun freemapper
page-warn-credit-only = PDF'erne nævner så kun freemapper. Datalicenserne (ODbL, dl-de/by-2-0) kræver kildeangivelse, når du giver kortet videre; så nævner du selv kilderne. ATTRIBUTION.txt viser dem alle.
page-need-pdf = Skalalinjen og kildelinjerne gælder kun, når der laves en PDF.
page-opt-offline = Kun cache, intet netværk
page-opt-force = Erstat de eksisterende kort
page-help-force = Kortet og, hvis "Træningskort" er afkrydset, træningskortet. De gamle filer bliver som .omap.bak.
page-opt-permits = Tilladelsesnoter
page-help-permits = Administrative enheder, beskyttede områder og hvem du skal spørge, i permits.md. Et fingerpeg, ikke juridisk rådgivning.
page-run-start = Opret kort
page-run-again = Opret igen
page-run-cancel = Annullér
page-run-cancelling = Annullerer …
page-warnings-heading = Advarsler
page-where-files = Filerne ligger i projektmappen:
page-open-folder = Åbn mappe
page-log-label = Kørselslog
page-force-title = Erstat de eksisterende kort?
page-force-text = Det eksisterende kort bliver erstattet, og hvis "Træningskort" er afkrydset, også træningskortet. Dine rettelser går ikke tabt: de gamle filer bliver i projektmappen som .omap.bak. Fortsæt?
page-force-cancel = Annullér
page-force-confirm = Erstat
page-step-fetch = Henter data
page-step-derive = Beregner skabeloner
page-step-map = Skriver kortet
page-step-ocad = Skriver OCAD-projektet
page-step-pdf = Skriver PDF'en
page-step-pdf-candidates = Skriver kontroludskriften
page-step-training = Skriver træningskortet
page-step-permits = Skriver tilladelsesnoterne
page-step-line = Trin { $index } af { $total }: { $label }
page-starting = Kørslen starter …
page-file-kind-map = kort
page-file-kind-template = skabelon
page-file-kind-ocad = OCAD-kort
page-file-kind-ocad-candidates = OCAD-kandidater
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontroludskrift
page-file-kind-training-map = træningskort
page-file-kind-training-pdf = træningskort-PDF
page-file-kind-training-ocad = OCAD-træningskort
page-file-kind-training-view-map = visning af træningskort
page-file-kind-training-view-pdf = PDF af visning af træningskort
page-file-kind-training-view-ocad = OCAD-visning af træningskort
page-file-kind-preview = kortforhåndsvisning
page-file-kind-permits = tilladelsesnoter
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Færdig. Filerne er skrevet.
page-result-failed = Kørslen fejlede: { $error }
page-result-failed-kept = Det, der er skrevet, bliver i projektmappen; en ny kørsel fortsætter derfra.
page-result-cancelled = Annulleret. Det, der allerede er downloadet, bliver i cachen; en ny kørsel fortsætter derfra.
page-result-title-done = Kort oprettet
page-result-title-failed = Kørsel fejlede
page-result-title-cancelled = Kørsel annulleret
page-kept-map = Det eksisterende kort blev ladt, som det er. "Erstat det eksisterende kort" laver det om og beholder det gamle som .omap.bak.
page-no-files = Der blev ikke lavet nogen filer.
page-opened-folder = Mappen er åben.
page-open-failed = { $message } Mappe: { $dir }
page-why-no-area = Vælg først et område.
page-why-checking = Forhåndsvisningen beregnes …
page-why-too-large = Området er for stort.
page-why-outside = Området ligger uden for de understøttede delstater.
page-why-disk = Ikke nok ledig diskplads (se ovenfor).
page-why-running = En kørsel er i gang.
page-err-scale-bar-no-pdf = Skalalinjen kræver en PDF.
page-err-sources-no-pdf = Kildelinjerne kræver en PDF.
page-save-failed = Området er ikke gemt; ingen kørsel blev startet.
page-poll-lost = Intet svar fra serveren. freemapper bliver ved med at prøve.
page-run-unknown = Serveren kender ikke længere denne kørsel.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Afvis

## update (spec 3.4 to 3.6)

update-source-build = Denne freemapper er bygget fra kildekode. Opdatér den med git pull og cargo build --release --locked.
update-current = freemapper { $version } er opdateret.
update-available = freemapper { $new } er tilgængelig (du har { $old }), udgivet { $released }.
update-details =
    Udgivelsesnoter: { $notes }
    Download: { $size } MB
update-decimal-separator = ,
update-ask = Installér den nu? [y/N]
update-not-installed = Intet blev installeret.
update-needs-yes = inputtet er ikke en terminal, så freemapper kan ikke spørge før installationen; kør freemapper update --yes for at installere uden at spørge
update-installing = downloader og kontrollerer freemapper { $version }
update-done = freemapper opdateret fra { $old } til { $new }. Udgivelsesnoter: { $notes }
update-mcp-restart = En kørende MCP-server beholder den gamle version, indtil AI-assistenten genstarter den.
update-rolled-back = Gik tilbage til freemapper { $version }. Versionen, du forlod, er gemt som { $path }.
update-not-writable = kan ikke skrive til programmappen { $path }, og freemapper beder ikke om administratorrettigheder. Pak den nye udgivelse ud dér manuelt, eller flyt freemapper til en mappe, du kan skrive til.
update-too-old = freemapper { $new } kan ikke installeres af selvopdateringen i { $old }; installér { $needed } eller nyere manuelt, se { $notes }
update-busy = en anden opdatering eller tilbagerulning kører i programmappen (låsefil { $path }); vent, til den er færdig, og prøv igen
update-bad-signature = udgivelsen kunne ikke verificeres ({ $error }); intet blev installeret
update-check-bad-signature = den nyeste freemapper-udgivelse kunne ikke verificeres ({ $error }); intet blev installeret
update-no-manifest = den nyeste udgivelse kan ikke installere sig selv (den har intet opdateringsmanifest); download den fra { $page }
update-no-manifest-version = udgivelse { $version } findes ikke eller kan ikke installere sig selv (den har intet opdateringsmanifest); se { $page }
update-no-asset = udgivelsen har intet program til dette system ({ $platform }); intet blev installeret
update-no-previous = der er ingen tidligere version at gå tilbage til ({ $path } mangler)
update-failed = opdateringen mislykkedes: { $error }
update-mode-set = Opdateringstilstand sat til { $mode } i { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } tilsidesætter denne indstilling, så længe den er sat.
update-mode-failed = kunne ikke skrive { $path }: { $error }
update-notice = freemapper { $new } er tilgængelig (du har { $old }). Opdatér med: freemapper update
update-applies-next-start = Den nye version gælder fra næste start af freemapper.
update-install-failed = freemapper { $new } blev ikke installeret: { $error }
update-no-config-dir = ingen konfigurationsmappe på dette system; sæt FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Note til personen: ved sin start fandt freemapper-serveren dette om adaptermapperne. Giv det videre; personen retter mappen eller fjerner den og genstarter derefter serveren.
    { $lines }
update-mcp-note = Note til personen: freemapper { $new } er tilgængelig (dette er { $old }). Spørg dem, før du kalder install_update.
update-mcp-needs-confirm = intet blev installeret; spørg først personen, og kald install_update med confirm: true, når de er enige
update-mcp-note-manual = Note til personen: freemapper { $new } er tilgængelig (dette er { $old }), men install_update kan ikke installere den; installér { $needed } eller nyere manuelt, se { $notes }
update-mcp-note-bad-signature = Note til personen: den nyeste freemapper-udgivelse kunne ikke verificeres ({ $error }); intet blev installeret. Fortæl dem det, og installér den ikke på anden vis.
update-mcp-restart-now = Genstart freemapper MCP-serveren nu; den kører stadig den gamle version.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = fejl
error-cancelled = annulleret
cli-cancelled-kept = annulleret; beholdt: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disken er fuld, så freemapper kunne ikke skrive { $path }. Frigør noget plads, og prøv igen

## Errors: project

error-project-already-exists = projektet findes allerede: { $path }
error-project-map-exists = kortet findes allerede (det kan indeholde dit arbejde); brug --force for at erstatte det: { $path }
error-project-invalid-scale = ugyldig målestok 1:{ $scale }
error-project-invalid-contour-interval = ugyldig ækvidistance { $interval } m i freemapper.toml; den skal være større end 0
error-project-invalid-steep-slope = ugyldig stejlhedsgrænse { $slope }° i freemapper.toml; den skal ligge mellem 15° og 80°
error-project-invalid-face-slope = ugyldig vægtærskel { $slope }° i freemapper.toml; den skal ligge mellem 55° og 85°
error-project-scale-not-allowed = målestok 1:{ $scale } er ikke tilladt for kort af typen { $kind }; brug { $allowed }
error-project-scale-not-allowed-enlarge = målestok 1:{ $scale } er ikke tilladt for kort af typen { $kind }; brug { $allowed }. Tegn i 1:4000, freemapper-print forstørrer til 1:3000 ved udskrivning
word-kind-schoolyard = skolegård
word-kind-sprint = sprint
word-kind-forest = skov
error-project-toml = ugyldig projektfil: { $message }
error-project-unknown-adapter = projektfilen nævner adapteren "{ $adapter }", som denne freemapper ikke har; en nyere freemapper kan have den (`freemapper update`), eller den kommer fra din adaptermappe (`freemapper adapters list` viser den) eller, med --adapters-from-project på kommandolinjen, fra mappen `adapters` i projektet (kun for et projekt, du stoler på)
error-project-not-fetched = ingen fliser hentet endnu; kør først `freemapper fetch`
error-project-ocd = OCAD-eksport: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: billede og verdensfil passer ikke til kortområdet; kør `freemapper derive` igen
error-project-tiles-outdated = kortområdet har brug for fliserne { $tiles }, som ikke blev hentet; kør `freemapper fetch`
error-project-buildings-not-fetched = ingen ALKIS-bygninger gemt for dette kortområde; kør først `freemapper fetch` online
error-project-orthophoto-not-fetched = intet ortofoto gemt for dette kortområde; kør først `freemapper fetch` online
error-project-terrain-not-fetched = ingen terrænmodel gemt for dette kortområde; kør først `freemapper fetch` online
error-project-candidates-outdated = de gemte OSM-kandidater dækker ikke kortområdet; kør `freemapper fetch`
error-project-stored = { $path }: { $message }; kør `freemapper fetch` igen
error-project-area-too-large = kortområde plus buffer er { $width } × { $height } m, mere end { $limit } m pr. side; gør kortområdet eller buffer_m i freemapper.toml mindre
error-project-area-crosses-country = kortområdet ligger på begge sider af en landegrænse; kort på tværs af en grænse understøttes endnu ikke, så tegn området inden for ét land
error-project-permits-not-for-region = tilladelsesnoterne er endnu ikke tilgængelige for denne region: de spørger tyske tjenester og nævner tyske myndigheder
error-project-outside-coverage = kortområdet ligger uden for de understøttede delstater ({ $states }); freemapper har indtil videre kun datakilder til dem

## Errors: disk space

error-space-required = ikke nok diskplads til { $part }. Det kræver { $needed }, men { $path } har kun { $left } ledig ud over reserven på { $reserve }. Frigør noget plads, og kør `freemapper fetch` igen
error-space-skipped = freemapper springer { $part } over på grund af manglende diskplads. Det kræver { $needed }, men { $path } har kun { $left } ledig ud over reserven på { $reserve }. Frigør noget plads, og kør `freemapper fetch` igen
word-part-map-data = ALKIS- og OSM-kortdata
word-part-terrain = terrænmodellen
word-part-orthophoto = ortofotoet
word-part-surface = overflademodellen til nDSM-skabelonen
word-part-osm-update = opdateringen af det cachede OSM-udtræk
word-part-k-five = Berlins K5-kortskabelon
word-part-berlin-layers = Berlins ALKIS-strukturer, trækatastre, gadeopmåling og punktlag
word-part-state-layers = de fulde ALKIS-strukturer, landskabsmodellens objekter og trækatastrene
word-part-point-cloud = laserpunktskyerne til vegetationskandidaterne

## Errors: downloads

error-fetch-http = download af { $url } mislykkedes: { $message }
error-fetch-http-temporary = download af { $url } mislykkedes: { $message }. Tjenesten er sandsynligvis overbelastet eller under vedligeholdelse; freemapper har spurgt op til { $attempts } gange. Start kørslen igen om nogle minutter; det, der allerede er hentet, bevares.
error-fetch-corrupt-archive = download af { $url } er ikke et komplet flisearkiv: { $message }
error-fetch-corrupt-extract = download af { $url } er ikke et komplet OSM-udtræk: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] flisen { $tiles } er ikke dækket af de udgivne fliser; gør kortområdet eller buffer_m i freemapper.toml mindre
       *[other] fliserne { $tiles } er ikke dækket af de udgivne fliser; gør kortområdet eller buffer_m i freemapper.toml mindre
    }
error-fetch-missing-offline = { $count ->
        [one] offline og ikke i cachen: { $product }-flisen { $tiles }; kør `freemapper fetch` online
       *[other] offline og ikke i cachen: { $product }-fliserne { $tiles }; kør `freemapper fetch` online
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = terrænmodel (DGM1)
word-product-image-dsm = overflademodel (bDOM)
word-product-saxony-dtm = terrænmodel (GeoSN DGM1)
word-product-saxony-dsm = overflademodel (GeoSN DOM1)
error-fetch-osm-extract-missing = intet OSM-udtræk i cachen; kør `freemapper fetch` online
error-fetch-not-found = { $name } findes ikke på serveren
error-fetch-refused = { $url }: serveren afviste forespørgslen (HTTP { $status })
error-fetch-refused-detail = { $url }: serveren afviste forespørgslen (HTTP { $status }): { $detail }
error-fetch-too-large = download af { $url } er større end { $limit } MB; stoppet
error-fetch-not-stored = ingen { $label } gemt for dette kortområde: { $path } mangler, kan ikke læses eller dækker det ikke; kør først `freemapper fetch` online
error-fetch-range-ignored = { $url }: serveren ignorerede byteintervallet { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper sender ikke forespørgsler til denne adresse: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper sender ikke forespørgsler til denne adresse: adapteren for { $name } nævner ikke værten { $host }{ $from }. Hvis tjenesten er flyttet dertil, tillad værten med `freemapper adapters allow-host { $adapter } { $host }` og meld det på https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper sender ikke forespørgsler til denne adresse: adapteren { $adapter } nævner ikke værten { $host }{ $from }. Hvis tjenesten er flyttet dertil, tilføj værten til `[net] hosts` i adapterens manifest
error-fetch-host-redirected-from = {" "}({ $url } omdirigerer dertil)
error-reason-not-a-host-name = ikke et værtsnavn
error-reason-not-ascii = et navn uden for ASCII skal skrives i sin xn--form
error-reason-not-a-dns-name = ikke et DNS-navn
error-reason-no-domain = et navn på denne computer eller uden domæne
error-reason-local-name = et navn på det lokale netværk
error-reason-ip-address = en IP-adresse, ikke et DNS-navn
error-reason-unexpected-character = uventet tegn
error-reason-not-https = ikke https
error-reason-user-info = brugeroplysninger
error-reason-port = en anden port end 443
error-reason-host-not-declared = værten er ikke en af de angivne værter
error-reason-local-address = en adresse på denne computer eller det lokale netværk, ikke på en server på internettet
error-reason-too-many-redirects = mere end { $count } omdirigeringer
error-reason-redirect-nowhere = en omdirigering (HTTP { $status }) til ingenting
error-fetch-currency-csv = valuta-CSV linje { $line }: { $message }
error-alkis = ugyldige ALKIS-data: { $message }
error-osm = OSM-data: { $message }

## Errors: templates and rasters

error-derive-zip = flisearkiv: { $message }
error-derive-image = billede: { $message }
error-derive-no-tiles = ingen fliser at udlede fra
error-derive-area-too-large = kortområdet er for stort til én skabelon: { $width } × { $height } pixel (højst { $limit } pr. side); gør kortområdet eller buffer_m i freemapper.toml mindre
error-raster-world-file = ugyldig verdensfil: { $message }
error-raster-misaligned = rastrene er ikke justeret til samme pixelgitter: { $message }
error-raster-pixel-size = rastrene har forskellige pixelstørrelser: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = ugyldig GeoJSON: { $message }
error-area-no-polygon = GeoJSON indeholder ingen polygon
error-area-multi-polygon = MultiPolygon med { $parts } dele; opdel kortområdet i én polygon
error-area-out-of-range = koordinat uden for intervallet: lat { $lat }, lon { $lon }
error-area-not-an-area = ikke et brugbart kortområde: { $message }
error-georef-date = datoen { $date } ligger uden for gyldighedsperioden for World Magnetic Model
error-georef-map-range = punktet E { $easting } N { $northing } ligger uden for kortets koordinatområde
error-omap-xml = ugyldig OOM-XML: { $message }
error-omap-missing-element = elementet <{ $name }> blev ikke fundet
error-ocd-duplicate-symbol = symbolnummer { $symbol } bruges to gange
error-ocd-unknown-symbol = objektet henviser til symbol { $symbol }, som ikke findes i filen
error-ocd-wrong-type = et objekt med symbol { $symbol } har den forkerte type til sit symbol

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Brug:
cli-heading-commands = Kommandoer:
cli-heading-options = Valgmuligheder:
cli-heading-arguments = Argumenter:
cli-help-possible-values = mulige værdier
cli-help-default = standard
cli-arg-help = Vis hjælp
cli-arg-version = Vis version
cli-help-help = Vis hjælpen til en kommando
cli-arg-help-command = Kommandoen, hjælpen skal vises for, for eksempel run
cli-parse-hint = Prøv '--help' for mere information.
cli-double-click-start = freemapper åbner siden, hvor du vælger kortområdet, i din browser. Projektmappen er { $path }. Luk dette vindue for at afslutte programmet.
cli-parse-unknown-argument = uventet argument { $arg }
cli-parse-missing-required = manglende påkrævet argument: { $args }
cli-parse-invalid-value = ugyldig værdi { $value } for { $arg }
cli-parse-invalid-value-choices = ugyldig værdi { $value } for { $arg }; mulige værdier: { $values }
cli-parse-invalid-subcommand = ukendt kommando { $value }
cli-parse-missing-subcommand = en kommando er påkrævet
cli-parse-too-many-values = uventet værdi { $value } for { $arg }
cli-parse-argument-conflict = { $arg } kan ikke bruges sammen med { $other }

help-freemapper = Bygger OOM-startprojekter ud fra geodata
help-freemapper-lang = Sprog for programtekster, `de` eller `en` (standard: FREEMAPPER_LANG, derefter systemsproget, derefter engelsk)
help-init = Opret en projektmappe til et kortområde
help-init-dir = Projektmappe, der skal oprettes
help-init-geojson = GeoJSON-fil med kortområdets polygon (EPSG:4326)
help-init-osm = OSM-objekt for kortområdet, fx way/992423119
help-init-place = Stedets navn, fx "Clara-Zetkin-Schule Strausberg"; freemapper bruger det bedste match med et område
help-init-part = Hvilken del af et OSM-område med flere adskilte dele der skal bruges (1 er den største)
help-init-kind = Korttype
help-arg-scale = Målestoksnævner for et nyt projekt: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skov 15000 (standard), 10000 eller 7500. Andre værdier afvises, fordi freemapper-print afviser dem
help-init-title = Korttitel til randoplysningerne (standard: mappens navn)
help-export = Skriv startprojektet (OOM-kort, skabeloner, kildeangivelse) ind i en projektmappe
help-export-dir = Projektmappe
help-arg-force = Erstat en eksisterende kortfil og behold den gamle som `<name>.omap.bak`
help-export-ocad = Skriv også et OCAD-udgangspunkt ind i `ocad/` (OCAD 12-format); et eksisterende OOM-kort bliver så, medmindre du angiver --force
help-export-pdf = Skriv også `<name>.pdf`, kortet til udskrivning (kun kortobjekterne, ingen kandidater); et eksisterende OOM-kort bliver så, medmindre du angiver --force
help-export-pdf-candidates = Skriv også `<name>-candidates.pdf`, kontroludskriften: kortet med alle kandidater, positionstro; et eksisterende OOM-kort bliver så, medmindre du angiver --force
help-export-training = Skriv også træningskortet `<name>-training.omap` og dets PDF `<name>-training.pdf`: kortet og alle kandidater, ryddet op, så det kan printes, som det er; med --ocad også `ocad/<name>-training.ocd`. Et eksisterende OOM-kort bliver så, medmindre du angiver --force, og --force erstatter også et eksisterende træningskort
help-export-view = Med --training: skriv også træningskortet uden det, en visning udelader, som `<name>-training-<view>.omap` og dets PDF (med --ocad også en OCAD-fil). Visninger: `no-paths` (ingen veje, stier, trapper), `brown-only` (kun brune symboler og rammen), `no-north-lines`, eller en liste `no:<codes>` / `only:<codes>`; kombinér med `+`. Gentag valgmuligheden for flere filer
help-export-pdf-scale-bar = Tegn skalalinjen på PDF'erne (fra som standard, tyske kort har som regel ingen; målestokstallet bliver i titelfeltet)
help-export-pdf-sources = Kildelinjer på PDF'erne: `auto` udskriver den fulde liste og forkorter den kun, hvis den ville fylde mere end 5 linjer, `full` forkorter aldrig, `credit-only` udskriver kun freemappers kildeangivelse (ODbL og dl-de/by-2-0 kræver så, at du nævner kilderne, når du giver kortet videre). ATTRIBUTION.txt viser altid alle kilder
help-fetch = Download data til kortområdet (ortofoto, terrænmodel, nDSM, ALKIS-bygninger, OSM)
help-fetch-dir = Projektmappe
help-fetch-offline = Brug kun cachen, uden netværksadgang
help-arg-cache-dir = Cachemappe til downloadede fliser (standard: systemets cachemappe + `freemapper`)
help-derive = Beregn skabelonerne (ortofoto, skyggerelief, hældning, lokalt relief, nDSM), højdekurver og terrænformer ud fra de hentede data
help-derive-dir = Projektmappe
help-run = Hent, udled og eksportér i ét hug
help-run-dir = Projektmappe
help-run-offline = Brug kun cachen, uden netværksadgang
help-run-ocad = Skriv også et OCAD-udgangspunkt ind i `ocad/` (OCAD 12-format)
help-run-pdf = Skriv også `<name>.pdf`, kortet til udskrivning (kun kortobjekterne, ingen kandidater)
help-run-pdf-candidates = Skriv også `<name>-candidates.pdf`, kontroludskriften: kortet med alle kandidater, positionstro
help-run-training = Skriv også træningskortet `<name>-training.omap` og dets PDF `<name>-training.pdf`: kortet og alle kandidater, ryddet op, så det kan printes, som det er; med --ocad også `ocad/<name>-training.ocd`. Et eksisterende træningskort bliver, medmindre du angiver --force
help-run-view = Med --training: skriv også træningskortet uden det, en visning udelader, som `<name>-training-<view>.omap` og dets PDF (med --ocad også en OCAD-fil). Visninger: `no-paths` (ingen veje, stier, trapper), `brown-only` (kun brune symboler og rammen), `no-north-lines`, eller en liste `no:<codes>` / `only:<codes>`; kombinér med `+`. En eksisterende visningsfil bliver, medmindre du angiver --force
help-run-pdf-scale-bar = Tegn skalalinjen på PDF'erne (fra som standard; målestokstallet bliver i titelfeltet)
help-run-pdf-sources = Kildelinjer på PDF'erne: `auto` udskriver den fulde liste og forkorter den kun, hvis den ville fylde mere end 5 linjer, `full` forkorter aldrig, `credit-only` udskriver kun freemappers kildeangivelse (ODbL og dl-de/by-2-0 kræver så, at du nævner kilderne, når du giver kortet videre). ATTRIBUTION.txt viser altid alle kilder
help-select = Vælg kortområdet i browseren ved at tegne det eller vælge et sted. Ved gem skrives freemapper.toml, og projektmappen oprettes, hvis det er nødvendigt
help-select-dir = Projektmappe (oprettes ved gem, hvis den ikke findes)
help-select-kind = Korttype for et nyt projekt
help-select-title = Korttitel for et nyt projekt
help-select-no-browser = Udskriv kun sidens adresse; åbn ikke browseren
help-select-port = Port på 127.0.0.1 (standard: en ledig)
help-select-offline = Siden starter med "kun cache, intet netværk" afkrydset
help-select-ocad = Siden starter med OCAD-startprojektet afkrydset
help-select-pdf = Siden starter med PDF til udskrivning afkrydset
help-select-pdf-candidates = Siden starter med kontroludskriften (alle kandidater) afkrydset
help-select-training = Siden starter med træningskortet afkrydset
help-select-pdf-scale-bar = Siden starter med skalalinjen på PDF'erne afkrydset
help-select-pdf-sources = Siden starter med dette valg af kildelinjer på PDF'erne, som for `run`
help-find = List OpenStreetMap-steder for et navn, områder først, med id'et til init --osm
help-find-query = Stedets navn, fx "Clara-Zetkin-Schule Strausberg"
help-find-json = Udskriv resultaterne som JSON, samme felter som MCP-værktøjet find_place
help-mcp = Stil værktøjerne til rådighed for en AI-agent over MCP (stdin/stdout)
help-update = Installér en nyere freemapper fra GitHub-udgivelserne, efter at have spurgt. En build fra kildekode siger i stedet, hvordan den opdateres
help-update-check = Kun kontrol: afslutningskode 10, når en nyere version findes, 0, når denne er aktuel
help-update-yes = Installér uden at spørge; nødvendigt, når input ikke er en terminal
help-update-version = Installér denne version (x.y.z) i stedet for den nyeste, også en ældre
help-update-rollback = Gå tilbage til den version, den sidste opdatering erstattede
help-update-mode = Gem, hvad den daglige kontrol gør, når den finder en nyere version (standard: notify). Virker i alle builds, fordi det kun skriver indstillingen
help-adapters = Vis regionsadapterne: de delstater og lande, freemapper har datakilder til
help-adapters-list = List adapterne med hvor hver kommer fra (indbygget, din mappe, et projekt), dens revision, dagen den sidst blev kontrolleret og de værter, den spørger
help-adapters-list-json = Udskriv listen som JSON
help-adapters-list-project = Læs også adapterne i mappen `adapters` i denne projektmappe, som --adapters-from-project ville gøre
help-adapters-check = Kontrollér en adapters mappe, før den bruges eller gives videre: om den kan indlæses, hvad dens forfatter bør vide, og med --live om dens tjenester svarer. Afslutningskode 0: den består; 1: den har fund; 2: kontrollen kunne ikke køre
help-adapters-check-dir = Adapterens mappe; dens navn er adapterens id
help-adapters-check-live = Spørg også adapterens tjenester om den boks, --bbox angiver, og sig, hvad hver leverer. Forespørgslerne bliver på de værter, manifestet angiver
help-adapters-check-bbox = Live-kontrollens boks i grader, som minLon,minLat,maxLon,maxLat. Den ligger inden for adapterens grænse; 150 m gange 150 m er nok
help-adapters-check-json = Udskriv resultatet som JSON
help-adapters-probe = Spørg hver tjeneste, som de indbyggede adaptere nævner, én gang og sig, om den svarer: ok, slow, moved eller error. Afslutningskoden er 0, også når en tjeneste fejler; resultatet er svaret
help-adapters-probe-json = Udskriv resultatet som JSON
help-adapters-probe-status = Læs gårsdagens status fra denne fil (den må mangle) og udskriv den nye status som JSON: En tjeneste er nede efter to dårlige nætter i træk. Ændringerne går til standardfejl
help-adapters-allow-host = Tillad en indbygget adapter en vært mere: for en delstat, hvis tjeneste er flyttet til en server, dens adapter endnu ikke nævner. Meddelelsen om den afviste adresse siger, hvilken vært. Værten kontrolleres som en vært i et manifest, og freemapper forbinder stadig kun til servere på internettet
help-adapters-allow-host-adapter = Den indbyggede adapters id, som `adapters list` viser det
help-adapters-allow-host-host = Værten, der skal tillades: et DNS-navn, ingen adresse, ingen port
help-adapters-disallow-host = Tag en vært tilbage, som allow-host tillod
help-adapters-disallow-host-adapter = Den indbyggede adapters id
help-adapters-disallow-host-host = Værten, der skal tages tilbage
help-adapters-export = Skriv en delstats indbyggede adapter som en adaptermappe, du kan ændre: dens manifest og de filer, manifestet nævner, i `<dir>/<id>`. Mappen må ikke findes endnu eller skal være tom. Ret adresserne i dens `manifest.toml`, tjek den med `adapters check`, og læg den i din adaptermappe: Så erstatter den den indbyggede adapter, med en advarsel. Fjerner du mappen, er den indbyggede tilbage
help-adapters-export-adapter = Id'et på den indbyggede adapter, som `adapters list` viser det
help-adapters-export-dir = Mappen, hvor mappen `<id>` oprettes
help-sources = Læs og ret adresserne på datakilderne uden en ny udgave. En rettelse kommer i din egen adaptermappe og erstatter den indbyggede adresse
help-sources-list = Vis adresserne i manifesterne med deres stier og hvor hvert manifest kommer fra. Med et adapter-id kun den adapters
help-sources-get = Udskriv én adresse og intet andet. Afslutningskode 1, når adapteren eller stien er ukendt
help-sources-set = Ret én adresse i din egen adaptermappe, når hele manifestet har bestået sin kontrol. En adresse på en vært, som manifestet ikke erklærer, afvises, medmindre du angiver --add-host
help-sources-diff = Vis, hvad din adaptermappe ændrer i forhold til det indbyggede manifest
help-sources-check = Kontroller mappen til din adapter med det id som `adapters check`. Afslutningskode 0: bestået; 1: fund; 2: kontrollen kunne ikke køre
help-sources-schema = Udskriv de nøgler i et manifest, der rummer en adresse, med --json som JSON-skema
help-arg-adapter = Adapterens id, som `adapters list` viser det
help-arg-path = Adressens sti, som `sources list` viser den
help-sources-set-url = Den nye adresse: https, uden forespørgsel
help-sources-set-add-host = Tilføj også den nye adresses vært til de værter, manifestet erklærer
cli-sources-no-user-dir = dette system har ingen mappe til dine adaptere; sæt FREEMAPPER_ADAPTER_DIR til en absolut sti
cli-sources-bad-id = { $adapter } er intet adapter-id
cli-sources-unknown-adapter = freemapper kender ingen adapter { $adapter }; `freemapper adapters list` viser id'erne
cli-sources-no-manifest = intet manifest i dette program beskriver adapteren { $adapter }; dens adresser står i koden
cli-sources-unknown-path-of = adapteren { $adapter } har ingen adresse under { $path }; `freemapper sources list { $adapter }` viser stierne
cli-sources-bad-value = den nye adresse skal være en https-adresse på en navngivet vært, uden anførselstegn, omvendt skråstreg, kontroltegn, bruger eller port
cli-sources-host-not-declared = den nye adresse fører til værten { $host }, som manifestet ikke erklærer; --add-host erklærer den
cli-sources-invalid = manifestet ville ikke bestå med denne ændring, så der blev intet skrevet: { $findings }
cli-sources-set-created = Mappen { $folder } til adapteren { $adapter } er lavet ud fra dens indbyggede manifest; den erstatter den indbyggede adapter fra næste start.
cli-sources-set-host-added = Værten { $host } er nu erklæret i [net] hosts; freemapper spørger nu den server for denne adapter.
cli-sources-set-next = Kontroller den med `freemapper adapters check { $folder }`. Kildeangivelse og licens forbliver, som de var: hold dem rigtige, hvis tjenesten skiftede ejer. Genstart MCP-serveren eller områdesiden.
cli-sources-diff-no-folder = du har ingen mappe til adapteren { $adapter } ({ $folder }); der er intet at sammenligne
cli-sources-diff-none = din mappe rummer de samme adresser som det indbyggede manifest
help-sources-list-json = Udskriv listen som JSON
help-sources-check-live = Spørg også adapterens tjenester om den boks, --bbox angiver, og sig, hvad hver leverer. Forespørgslerne bliver på de værter, manifestet angiver
help-sources-check-bbox = Live-kontrollens boks i grader, som minLon,minLat,maxLon,maxLat. Den ligger inden for adapterens grænse; 150 m gange 150 m er nok
help-sources-check-json = Udskriv resultatet som JSON
help-sources-schema-json = Udskriv resultatet som JSON
help-arg-adapters-from-project = Indlæs også adapterne i mappen `adapters` i projektmappen. freemapper nævner hver af dem med de værter, den spørger. Brug det til et projekt, du stoler på: en adapter får freemapper til at spørge de servere, den nævner

## The summary of a run, for the MCP server and the area page

cli-internal-error = intern fejl: { $message }
cli-pdf-sources-invalid = pdf_sources skal være auto, full eller credit-only, ikke { $value }
cli-view-unknown = en visning er no-paths, brown-only, no-north-lines, no:<codes> eller only:<codes>, ikke { $value }
cli-view-empty = listen { $value } nævner ingen symbolkode
cli-view-bad-code = { $value } er ingen symbolkode som 505 eller 501.6
cli-summary-training-view = Skrev { $map } og { $pdf } (en visning af træningskortet)
cli-summary-kept-training-view = Beholdt filerne for visningen { $view } af træningskortet; angiv --force for at fornye dem
cli-summary-training-view-failed = Visningen af træningskortet mislykkedes: { $error }
cli-estimate-disk-unreadable = kan ikke læse den ledige diskplads for { $path } ({ $message })
cli-summary-wrote-map = Skrev kortet { $path }
cli-summary-kept-map = Beholdt det eksisterende kort { $path } (det kan indeholde kortlæggerens arbejde)
cli-summary-map-lacks-templates = Kortet henviser endnu ikke til billedskabelonerne. Start pipelinen med force for at erstatte det og beholde det gamle som .omap.bak.
cli-summary-map-lacks-buildings = Kortet har endnu ingen ALKIS-bygninger. Start pipelinen med force for at tilføje dem; den erstatter kortet og beholder det gamle som .omap.bak.
cli-summary-map-lacks-canopies = Kortet mangler ALKIS-overdækninger og carporte (522), som nu er kortobjekter. Start pipelinen med force for at tilføje dem; den erstatter kortet og beholder det gamle som .omap.bak.
cli-summary-map-light = Basiskortet: { $joined } linjer sat sammen, { $left } skjulte eller for små efterladt til kandidatlagene (intet flyttet)
cli-summary-map-light-kept = Kandidatlagene indeholder nu også { $left } objekter eller strækninger, som denne version tager af basiskortet (mærket freemapper:generalized). Det beholdte kort har dem stadig, så de vises dobbelt. Start pipelinen med force for at rydde kortet op; den beholder det gamle som .omap.bak.
cli-summary-map-old-candidates = Kortet nævner stadig templates/candidates.omap; denne kørsel fyldte den med alle kandidater. Kandidaterne findes nu også i syv lag, en fil hver (templates/candidates-*.omap). Kortet viser dem efter en kørsel med force (det gamle kort bliver som .omap.bak), eller kortlæggeren tilføjer dem i OOM (Templates > Open template…).
cli-summary-wrote-ocad = Skrev OCAD-kortet { $path } ({ $count } objekter)
cli-summary-kept-ocad = Beholdt det eksisterende OCAD-kort { $path } (det kan indeholde kortlæggerens arbejde; force erstatter det)
cli-summary-ocad-old-candidates = OCAD-kortet nævner stadig { $path } som baggrundskort; denne kørsel fyldte den med alle kandidater. Kandidaterne findes nu også i syv lag, en fil hver (-candidates-*.ocd). OCAD-kortet viser dem efter en kørsel med force (det gamle kort bliver som .ocd.bak), eller kortlæggeren tilføjer dem i OCAD som baggrundskort.
cli-summary-ocad-left-out = Udeladt fra OCAD-kortet: { $skipped } objekter og { $dropped } degenererede dele
cli-summary-ocad-symbols = Vælg i OCAD Symbol > Load Colors and Symbols From… og vælg det officielle symbolsæt{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF-skolesættet 2019
       *[other] {" "}ISSprOM 2019
    }; det erstatter pladsholdersymbolerne efter nummer.
cli-summary-ocad-failed = OCAD-eksport mislykkedes: { $error }
cli-summary-pdf-without-credits = PDF uden kildeangivelser: ODbL og dl-de/by-2-0 kræver, at du nævner kilderne, når du giver kortet videre; se ATTRIBUTION.txt
cli-summary-wrote-pdf = Skrev kortet { $path } (uden kandidater)
cli-summary-pdf-failed = PDF-eksport mislykkedes: { $error }
cli-summary-wrote-pdf-candidates = Skrev kontroludskriften { $path } (kortet med alle kandidater, positionstro)
cli-summary-pdf-candidates-failed = Eksport af kontroludskrift mislykkedes: { $error }
cli-summary-training-failed = Eksport af træningskort mislykkedes: { $error }
cli-summary-kept-training = Beholdt det eksisterende træningskort { $path }, så en kørsel ikke overskriver et, der er printet eller givet videre; slet det eller sæt force for at fornye det

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = ingen hjemmemappe på dette system; brug en absolut sti
mcp-error-relative-path = { $path } er en relativ sti; brug en absolut sti eller en, der begynder med ~
mcp-error-home-folder = { $path } er selve hjemmemappen; brug en mappe inde i den, fx ~/maps/school
mcp-error-no-project = intet freemapper-projekt i { $path }; brug først create_project
mcp-error-unknown-kind = ukendt kind { $kind }; brug "schoolyard", "sprint" eller "forest"
mcp-error-reading = læser { $path }: { $error }
mcp-error-one-area-source = angiv præcis en af osm_id (fra find_place) eller geojson
mcp-error-page-open = områdevalgssiden for dette projekt er stadig åben; bed brugeren om at gemme området og trykke på Færdig på siden, og start derefter pipelinen
mcp-error-no-job = intet job { $job }; start_pipeline og permits_for_area returnerer jobbenes id'er for denne session
mcp-error-job-running = { $job } kører stadig for { $path }; én pipeline kører ad gangen, fordi projekterne deler downloadcachen. Start denne, når project_status melder finished eller failed
mcp-error-page-start = kan ikke starte siden: { $error }
mcp-created-project = Projekt oprettet: { $path } for { $area } ({ $kind }, 1:{ $scale }). Næste: estimate_download, derefter start_pipeline.
mcp-created-project-given-area = Projekt oprettet: { $path } for det angivne område ({ $kind }, 1:{ $scale }). Næste: estimate_download, derefter start_pipeline.
mcp-page-already-open = Siden for denne mappe er allerede åben; den beholder de indstillinger, den blev åbnet med.
mcp-page-next = Bed brugeren om at vælge området på siden, trykke på Gem og derefter på Færdig.
mcp-preview-of = Forhåndsvisning af { $layer }
mcp-permits-next = Tilladelsesnoterne bliver lavet. Forespørg project_status med dette job_id; når det er finished, indeholder permits_md teksten i permits.md. Giv den videre som et fingerpeg, ikke som juridisk rådgivning.
mcp-tool-check-update = Om der findes en nyere freemapper-udgivelse: dens version, udgivelsesdato, udgivelsesnoter og downloadstørrelse, eller at denne er aktuel. Installerer intet.
mcp-tool-create-project = Opret en projektmappe til et kortområde. Angiv osm_id (fra find_place) eller en GeoJSON-fil. kind er schoolyard (1:1000, standard), sprint (1:4000) eller forest (1:15000).
mcp-param-create-project-dir = Ny projektmappe (absolut sti eller begyndende med ~).
mcp-param-create-project-geojson = I stedet for osm_id: en GeoJSON-fil med kortområdets polygon (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-create-project-osm-id = OSM-objekt for kortområdet fra find_place, fx "way/992423119".
mcp-param-create-project-part = Med osm_id: hvilken del af et område med flere adskilte dele der skal bruges (1 er den største).
mcp-param-create-project-scale = Målestoksnævner: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skov 15000 (standard), 10000 eller 7500. Andre værdier afvises; til 1:3000 tegnes i 4000, freemapper-print forstørrer ved udskrivning.
mcp-param-create-project-title = Korttitel til randoplysningerne (standard: mappens navn).
mcp-tool-estimate-download = Hvad start_pipeline vil downloade for et projekt: fliserne, hver download med sin størrelse i MB, om den er i cachen, og om den passer på disken, de MB, der stadig skal downloades, og under disk den ledige plads på cache- og projektvolumenerne med meddelelser om det, der ikke passer (fortæl det til brugeren).
mcp-param-estimate-download-dir = Projektmappe (absolut sti eller begyndende med ~).
mcp-tool-find-place = Søg efter en skole, en park eller et andet sted via navn (OpenStreetMap Nominatim). Returnerer op til 5 match, områder først; kun match med has_area kan være et kortområde. En tom liste betyder, at intet blev fundet.
mcp-param-find-place-query = Navnet på skolen eller stedet, fx "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installér den nyeste freemapper-udgivelse over dette program. Kald den kun, efter at personen har sagt ja, med confirm true. Den nye version gælder, når assistenten genstarter freemapper-serveren.
mcp-param-install-update-confirm = true, når personen har sagt ja til at installere den nyere version; uden den installeres intet.
mcp-tool-list-layers = Et projekts lag: kortet, de syv kandidatlag og rasterskabelonerne, hver med sin fil, om det er lavet endnu, og hvilket trin der laver det.
mcp-param-list-layers-dir = Projektmappe (absolut sti eller begyndende med ~).
mcp-tool-open-area-selector = Åbn en side i browseren, hvor brugeren tegner kortområdet (rektangel eller polygon) eller vælger et sted, ser fliserne og downloadstørrelsen og gemmer det; ved gem oprettes projektmappen, eller dens område ændres. Returnerer den URL, brugeren skal have. Siden kører, til brugeren trykker på Færdig; et nyt kald for samme mappe returnerer den samme URL.
mcp-param-open-area-selector-dir = Projektmappe (absolut sti eller begyndende med ~); oprettes, når brugeren gemmer.
mcp-param-open-area-selector-kind = Korttype for et nyt projekt: "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-open-area-selector-ocad = Kørselspanelet starter med OCAD-startprojektet afkrydset. Standard false.
mcp-param-open-area-selector-offline = Kørselspanelet starter med "kun cache, intet netværk" afkrydset. Standard false.
mcp-param-open-area-selector-open-browser = Åbn også siden i denne computers standardbrowser. Standard true.
mcp-param-open-area-selector-pdf = Kørselspanelet starter med PDF til udskrivning afkrydset. Standard false.
mcp-param-open-area-selector-pdf-candidates = Kørselspanelet starter med kontroludskriften (alle kandidater) afkrydset. Standard false.
mcp-param-open-area-selector-pdf-scale-bar = Kørselspanelet starter med skalalinjen på PDF'erne afkrydset. Standard false.
mcp-param-open-area-selector-pdf-sources = Kildelinjer på PDF'erne, som panelet starter med: "auto" (standard), "full" eller "credit-only", som for start_pipeline.
mcp-param-open-area-selector-permits = Kørselspanelet starter med tilladelsesnoterne afkrydset. Standard false.
mcp-param-open-area-selector-scale = Målestoksnævner for et nyt projekt: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skov 15000 (standard), 10000 eller 7500. Andre værdier afvises.
mcp-param-open-area-selector-title = Korttitel for et nyt projekt.
mcp-param-open-area-selector-training = Kørselspanelet starter med træningskortet afkrydset. Standard false.
mcp-tool-preview-layer = En PNG-forhåndsvisning (højst 800 px) af et lag: map (billedet af kortobjekterne, som den sidste kørsel skrev, uden kandidaterne) eller en rasterskabelon: orthophoto, hillshade, slope, relief, ndsm eller k5 i Berlin.
mcp-param-preview-layer-dir = Projektmappe (absolut sti eller begyndende med ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" eller "k5" (kun Berlin).
mcp-tool-project-status = Tilstanden for et job fra start_pipeline eller permits_for_area: running, finished eller failed, de sidste statuslinjer, advarsler, de skrevne filer og resultatet eller fejlen. Et færdigt permits_for_area-job indeholder også teksten i permits.md som permits_md.
mcp-param-project-status-job-id = Job-id fra start_pipeline eller permits_for_area, fx "job-1".
mcp-tool-permits-for-area = Lav et projekts tilladelsesnoter: kortområdets administrative enheder, de beskyttede områder i det og hvem der driver jorden dér, med de myndigheder, du skal spørge; et fingerpeg, ikke juridisk rådgivning. Skriver permits.md og permits.json ind i mappen. Kører i baggrunden og returnerer et job_id; forespørg project_status, hvis svar indeholder teksten i permits.md som permits_md, når jobbet er færdigt. Ét job kører ad gangen.
mcp-param-permits-for-area-dir = Projektmappe (absolut sti eller begyndende med ~).
mcp-param-permits-for-area-offline = Læs kun de data, en tidligere kørsel gemte i projektet, uden netværksadgang. Standard false.
mcp-tool-start-pipeline = Download data, beregn skabelonerne og skriv startprojektet (fetch, derive, export). Kører i baggrunden og returnerer et job_id; forespørg project_status. Én pipeline kører ad gangen. Den beholder et eksisterende kort, medmindre force er true.
mcp-param-start-pipeline-dir = Projektmappe (absolut sti eller begyndende med ~).
mcp-param-start-pipeline-force = Erstat en eksisterende kortfil og behold den gamle som .omap.bak. Standard false.
mcp-param-start-pipeline-ocad = Skriv også et OCAD-udgangspunkt ind i mappen ocad/ (OCAD 12-format, til brugere, der redigerer i OCAD). Standard false.
mcp-param-start-pipeline-offline = Brug kun downloadcachen, uden netværksadgang. Standard false.
mcp-param-start-pipeline-pdf = Skriv også <name>.pdf, kortet klar til udskrivning (kun kortobjekterne, ingen kandidater). Standard false.
mcp-param-start-pipeline-pdf-candidates = Skriv også <name>-candidates.pdf, kontroludskriften: kortet med alle kandidater, positionstro (kontrollér dem i terrænet). Standard false.
mcp-param-start-pipeline-pdf-scale-bar = Tegn skalalinjen på PDF'erne. Standard false (tyske kort har som regel ingen; målestokstallet bliver i titelfeltet).
mcp-param-start-pipeline-pdf-sources = Kildelinjer på PDF'erne: "auto" (standard; den fulde liste, kun forkortet, hvis den fylder mere end 5 linjer), "full" (aldrig forkortet) eller "credit-only" (kun freemappers kildeangivelse; ODbL og dl-de/by-2-0 kræver så, at kilderne nævnes, når kortet gives videre). ATTRIBUTION.txt viser altid alle kilder.
mcp-param-start-pipeline-permits = Skriv også tilladelsesnoterne (permits.md og permits.json) efter kortet: administrative enheder, beskyttede områder og hvem der driver jorden. En fejl dér er en advarsel, ikke et mislykket job. Standard false.
mcp-param-start-pipeline-training = Skriv også træningskortet <name>-training.omap og dets PDF <name>-training.pdf: kortet og alle kandidater, ryddet op, så det kan printes, som det er; med ocad også ocad/<name>-training.ocd. Et eksisterende træningskort beholdes, medmindre force er true. Standard false.
mcp-param-start-pipeline-training-views = Med training: skriv også træningskortet uden det, en visning udelader, ét filsæt pr. indgang: "no-paths", "brown-only", "no-north-lines" eller en liste som "no:505,506" eller "only:101,102", kombineret med +. Filerne er <name>-training-<view>.omap og dens PDF.
mcp-instructions = freemapper laver OpenOrienteering Mapper-startprojekter til orienteringskort (skolegård 1:1000, sprint 1:4000, skov 1:15000) ud fra officielle geodata fra { $states } og fra OSM. Arbejdsgang: find_place (søg efter skolen eller området via navn; vælg et resultat med has_area) → create_project (en ny mappe, absolut sti, med osm_id) → estimate_download (fortæl brugeren, hvor meget der downloades) → start_pipeline (returnerer et job-id; download kan tage minutter) → project_status (forespørg hvert par sekunder, til tilstanden er finished eller failed) → list_layers og preview_layer (vis brugeren skabelonerne). Brugeren åbner derefter kortfilen i OpenOrienteering Mapper. Når intet sted passer, eller brugeren vil tegne området, åbner open_area_selector en side i browseren, hvor brugeren tegner eller vælger området og gemmer det (det opretter projektet); fortsæt med estimate_download, når brugeren siger, at det er gemt. Stier skal være absolutte eller begynde med ~. Én pipeline kører ad gangen. permits_for_area laver et projekts tilladelsesnoter (administrative enheder, beskyttede områder, hvem der driver jorden; et fingerpeg, ikke juridisk rådgivning) som et job ligesom start_pipeline. check_update fortæller, om der findes en nyere freemapper; install_update installerer den, kun med confirm: true, efter at personen har sagt ja, og assistenten genstarter derefter freemapper-serveren.

cli-contour-interval-saved = Ækvidistance gemt i freemapper.toml: { $interval } m; derive tegner højdekurverne med den
cli-contour-interval-auto = Ækvidistance i freemapper.toml: automatisk (korttypens; et skovkort får 2,5 m i fladt land og 10 m i stejlt land)
cli-steep-slope-saved = Stejlhedsgrænse gemt i freemapper.toml: { $slope }°; derive leder efter skrænter og klipper med den
cli-steep-slope-auto = Stejlhedsgrænse i freemapper.toml: standard (ca. 27°)
cli-face-slope-saved = Vægtærskel gemt i freemapper.toml: { $slope }°; derive leder efter klippevægsflader med den
cli-face-slope-auto = Vægtærskel i freemapper.toml: standard (70°)
help-arg-contour-interval = Ækvidistance i meter, fx `2.5`, eller `auto` for korttypens (et skovkort: 5 m, 2,5 m hvor det er fladt, 10 m hvor det er stejlt). Gemmes i freemapper.toml; derive tegner højdekurverne med den
help-arg-steep-slope = Hældningsvinkel i grader, fra hvilken terrænet regnes som stejlt, når der ledes efter skrænter og klipper, fx `55`, eller `auto` for standarden på 27°. Hæv den (55 til 65) i klippelabyrinter og kløfter, hvor næsten alle skråninger er stejle, og skrænter og klipper flyder sammen til én plet. Gemmes i freemapper.toml; derive leder efter skrænter og klipper med den
help-arg-face-slope = Hældningsvinkel i grader, hvorfra terrænet er en klippevæg (sort flade, symbol 206), f.eks. `75`, eller `auto` for standarden på 70°. 65 giver flere og større flader, også på stejle skråninger; 75 bevarer kun rigtige vægge. Gemmes i freemapper.toml; derive leder efter klippevægge med den
cli-content-saved = Indholdsindstilling gemt i freemapper.toml: { $summary }
cli-content-invalid = Kan ikke læse indholdsindstillingen: { $detail }
help-arg-content = Hvad kommer direkte på kortet: `safe` lader hvert objekt blive, hvor opbygningen placerer det, `balanced` (standard) lægger matriklens og registrenes objekter på kortet, `full` lægger alt på kortet. Valget gemmes i freemapper.toml
help-arg-content-set = Send en gruppe objekter til `map`, `candidates` eller `off`, som `GROUP=PLACE`, fx `walls=map`; kan gentages. Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Gemmes i freemapper.toml
mcp-param-create-project-content = Hvad kommer direkte på kortet: "safe", "balanced" (standard) eller "full". Gemmes i projektet. balanced lægger matriklens og registrenes objekter på kortet, full lægger alt på det.
mcp-param-create-project-content-set = Grupper sendt et andet sted hen, end profilen siger, som "group=place" med place map, candidates eller off, fx "walls=map". Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Ækvidistance i meter, fx 2.5; 0 går tilbage til det automatiske valg (korttypens; et skovkort får 2,5 m i fladt land og 10 m i stejlt land). Gemmes i projektet; næste derive tegner højdekurverne med den.
mcp-param-create-project-steep-slope = Hældningsvinkel i grader, fra hvilken terrænet regnes som stejlt, når der ledes efter skrænter og klipper, fx 55; 0 går tilbage til standarden på 27°. Hæv den (55 til 65) i klippelabyrinter og kløfter, hvor næsten alle skråninger er stejle, og skrænter og klipper flyder sammen til én plet. Gemmes i projektet; næste derive leder efter skrænter og klipper med den.
mcp-param-create-project-face-slope = Hældningsvinkel i grader, hvorfra terrænet er en klippevæg (sort flade, symbol 206), f.eks. 75; 0 vender tilbage til standarden på 70°. 65 giver flere og større flader, også på stejle skråninger; 75 bevarer kun rigtige vægge. Gemmes i projektet; næste derive leder efter klippevægge med den.
mcp-param-start-pipeline-content = Ændr, hvad der kommer direkte på kortet, før kørslen: "safe", "balanced" eller "full". Gemmes i projektet. Standard: projektets indstilling, ellers balanced.
mcp-param-start-pipeline-content-set = Grupper sendt et andet sted hen, end profilen siger, som "group=place" med place map, candidates eller off, fx "fences=candidates". Gemmes i projektet.
mcp-param-start-pipeline-contour-interval = Ækvidistance i meter, fx 2.5; 0 går tilbage til det automatiske valg (korttypens; et skovkort får 2,5 m i fladt land og 10 m i stejlt land). Gemmes i projektet; næste derive tegner højdekurverne med den.
mcp-param-start-pipeline-steep-slope = Hældningsvinkel i grader, fra hvilken terrænet regnes som stejlt, når der ledes efter skrænter og klipper, fx 55; 0 går tilbage til standarden på 27°. Hæv den (55 til 65) i klippelabyrinter og kløfter, hvor næsten alle skråninger er stejle, og skrænter og klipper flyder sammen til én plet. Gemmes i projektet; næste derive leder efter skrænter og klipper med den.
mcp-param-start-pipeline-face-slope = Hældningsvinkel i grader, hvorfra terrænet er en klippevæg (sort flade, symbol 206), f.eks. 75; 0 vender tilbage til standarden på 70°. 65 giver flere og større flader, også på stejle skråninger; 75 bevarer kun rigtige vægge. Gemmes i projektet; næste derive leder efter klippevægge med den.
mcp-error-content = Kan ikke læse indholdsindstillingen: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Skriv et projekts tilladelsesnoter (permits.md og permits.json): de administrative enheder, beskyttede områder og driftsherrer, kortområdet berører, og hvem det som regel betyder, at man skal spørge. Den spørger BfN og BKG og, hvor kortområdets delstat har egne åbne tjenester, dem via værterne i dens adapter (liste i manualen, kapitel 12); i Nordrhein-Westfalen downloader den cirka 32 MB til det første gang, i Brandenburg cirka 4 MB, senere kun nyere filer. Den læser det allerede cachede OSM-udtræk; --offline bruger det, der blev hentet før
help-permits-dir = Projektmappe
help-permits-offline = Brug kun de data, der blev hentet før, uden netværksadgang
help-run-permits = Skriv også tilladelsesnoterne (permits.md og permits.json) efter kortet; spørger BfN, BKG og delstatens åbne tjenester (manualen, kapitel 12). En fejl dér er en advarsel, og kortet bliver
cli-wrote-permits = Skrev tilladelsesnoterne { $path }
cli-summary-wrote-permits = Skrev tilladelsesnoterne { $path }
cli-permits-failed = tilladelsesnoterne kunne ikke skrives: { $error }
cli-permits-source-left-out = tilladelsesnoterne udelader { $source }: { $reason }
cli-permits-not-read = den blev ikke læst
error-permits-stored = { $path }: { $message }; hent tilladelsesnoterne igen online
error-permits-nothing-read = ingen kilde til tilladelsesnoterne kunne læses ({ $missing }); hent dem først online
error-permits-missing-not-read = { $source }: ikke læst

permits-title = Tilladelsesnoter
permits-disclaimer = Et fingerpeg, ikke juridisk rådgivning. De kompetente myndigheder afgør. Se tabellen under "Kilder og datadato" for datoen og nøjagtigheden af dataene; områder kan mangle eller være forældede (BfN: samles én gang om året, "Nicht für Planungszwecke geeignet.", det vil sige ikke egnet til planlægning).
permits-disclaimer-regional = Et fingerpeg, ikke juridisk rådgivning. De kompetente myndigheder afgør. Se tabellen under "Kilder og datadato" for datoen og nøjagtigheden af dataene; områder kan mangle eller være forældede.
permits-bfn-merge = BfN samler delstaternes data én gang om året og markerer dem som ikke egnede til planlægning. Delstatens myndigheder afgør.
permits-created = Lavet den { $date } til kortområdet for et { $kind ->
        [schoolyard] skolegårdskort
        [forest] skovkort
       *[other] sprintkort
    } ({ $area } ha).
permits-section-admin = Administrative enheder
permits-section-protected = Beskyttede områder (BfN)
permits-section-osm-protected = Flere beskyttede områder fra OpenStreetMap
permits-section-operators = Driftsherrer og ejere ifølge OpenStreetMap
permits-section-hints = Fingerpeg
permits-section-sources = Kilder og datadato
permits-none-admin = VG250 nævner ingen administrativ enhed for kortområdet.
permits-none-protected = Ifølge BfN ligger intet beskyttet område i kortområdet.
permits-none-osm-protected = OpenStreetMap nævner intet beskyttet område i kortområdet, som BfN ikke allerede opfører.
permits-none-operators = OpenStreetMap nævner ingen driftsherre og ingen ejer for kortområdet.
permits-none-hints = Ingen fingerpeg.
permits-section-not-read = Dataene fra { $source } blev ikke læst, så dette afsnit er tomt, selv om der kan være indgange.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Administrative områder 1:250 000 (VG250) fra BKG
        [bfn] Beskyttede områder i Tyskland fra BfN
       *[osm] OpenStreetMap, fra det udtræk, freemapper har i sin cache
    }
permits-level = { $level ->
        [land] Delstat (Land)
        [kreis] Kreds (Kreis)
        [verwaltungsgemeinschaft] Amt eller Verwaltungsgemeinschaft
       *[gemeinde] Kommune (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Berørt i kanten (mindre end 0,5% af kortområdet):
permits-at-edge-generalised = VG250 er generaliseret til målestok 1:250 000. En enhed, der kun rører kanten, kan være en følge af det.
permits-kind = { $kind ->
        [nationalpark] Nationalpark
        [nationales-naturmonument] Nationalt naturmonument
        [naturschutzgebiet] Naturreservat (Naturschutzgebiet)
        [ffh] FFH-område (Natura 2000)
        [vogelschutzgebiet] Fuglebeskyttelsesområde (Natura 2000)
        [biosphaerenreservat] Biosfærereservat
        [biosphaerenreservat-zone] Zone i et biosfærereservat
        [landschaftsschutzgebiet] Landskabsbeskyttelsesområde (Landschaftsschutzgebiet)
       *[naturpark] Naturpark
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Udpeget den { $date }
permits-protected-title = Retsgrundlag: { $title }
permits-protected-link = Lovtekst: { $url }
permits-protected-site-link = Områdeside (Natura 2000): { $url }
permits-protected-year = Dataår hos BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = uden navn
permits-osm-operator = Driftsherre ifølge OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap-områder, der omslutter hele kortområdet uden et hjørne inde i det, kan mangle her, og det samme gælder beskyttede områder, der er kortlagt i OSM som en relation med `type=boundary`. Delene fra BfN og VG250 er ikke berørt.
permits-osm-enclosing-gap-short = OpenStreetMap-områder, der omslutter hele kortområdet uden et hjørne inde i det, kan mangle her.
permits-operator-line = { $name } ({ $roles ->
        [operator] driftsherre
        [owner] ejer
       *[both] driftsherre og ejer
    }): { $objects }
permits-column-source = Kilde
permits-column-state = Tilstand
permits-column-credit = Kildeangivelse og licens
permits-state-read = læst, hentet den { $date }
permits-state-read-osm = læst, OSM-udtræk fra { $date }
permits-state-unreachable = ikke nået ({ $reason })
permits-state-not-read = ikke læst
permits-no-credit = ikke brugt
permits-hint-city-state = Kortområdet ligger i Berlin eller Hamburg. Dér er som regel Bezirksamt (bydelskontoret) eller en myndighed i delstaten ansvarlig. Disse noter fastlægger endnu ikke Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Beskyttede områder vedrører den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde). Her er det selve byen { $kreis }.
        [no] Beskyttede områder vedrører den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde), som regel { $kreis }.
       *[unknown] Beskyttede områder vedrører den kompetente lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde), som regel Landkreis eller den kreisfreie Stadt.
    }
permits-hint-national-park = En nationalpark har sine egne regler. Spørg nationalparkens administration i god tid.
permits-hint-nsg-paths = I et naturreservat skal man som regel holde sig til stierne. Spørg naturbeskyttelsesmyndigheden, hvis et løb skal gå uden for stierne.
permits-hint-natura2000 = Et FFH-område eller et fuglebeskyttelsesområde (Natura 2000) berører kortområdet. Til en begivenhed kan naturbeskyttelsesmyndigheden kræve en vurdering af virkningerne.
permits-hint-protected-areas-pl = I kortområdet ligger beskyttede områder eller beskyttede naturobjekter. Ved en nationalpark, en landskabspark eller et naturreservat er parkens eller reservatets administration det sædvanlige kontaktpunkt; for de øvrige områder og objekter kan den regionale direktion for miljøbeskyttelse (RDOŚ) i voivodskabet som regel sige, hvem der er ansvarlig. Spørg i god tid, hvis et arrangement er planlagt dér.
permits-hint-order-office = { $amt ->
        [none] For gader, pladser og andet offentligt areal er Ordnungsamt (ordensmyndigheden) i kommunen { $gemeinde } som regel ansvarlig.
       *[other] For gader, pladser og andet offentligt areal er Ordnungsamt (ordensmyndigheden) som regel ansvarlig: { $amt }, for kommunen { $gemeinde }.
    }
permits-hint-park-operator = Driftsherre for en park eller have i kortområdet ifølge OpenStreetMap: { $operator }. Spørg dem, før du bruger den.
permits-hint-school-operator = Driftsherre for skolen eller institutionen ifølge OpenStreetMap: { $operator }. Spørg skolelederen eller dette organ om lov til at bruge arealet.
permits-section-districts = Bezirke (bydele, { $source })
permits-section-green-spaces = Grønne områder ({ $source })
permits-section-owners = Arealer tilhørende delstaten og Bund ({ $source })
permits-section-monuments = Monumenter ({ $source })
permits-section-protected-objects = Naturmonumenter og andre beskyttede objekter ({ $source })
permits-none-districts = Ifølge { $source } ligger kortområdet i ingen Bezirk.
permits-none-green-spaces = Ifølge { $source } ligger intet offentligt grønt område i kortområdet.
permits-none-owners = Ifølge { $source } ligger ingen parcel tilhørende delstaten eller Bund i kortområdet.
permits-none-monuments = Ifølge { $source } ligger intet monument i kortområdet.
permits-none-protected-objects = Ifølge { $source } ligger ingen af de beskyttede objekter, den indeholder, i kortområdet.
permits-none-sampled = Ved de punkter, der blev spurgt, nævner { $source } intet her.
permits-gaps = Dataene fra { $source } er ikke fuldstændige: { $gaps }.
permits-sampled = { $source } svarer kun for enkelte punkter. freemapper spurgte i et net af punkter over kortet: andelene er skøn, og det, der ligger mellem punkterne, kan mangle.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = formål: { $dedication }
permits-playgrounds-none = Ingen offentlig legeplads ligger i kortområdet.
permits-playgrounds-one = En offentlig legeplads ligger i kortområdet. Den opføres ikke særskilt.
permits-playgrounds-many = { $count } offentlige legepladser ligger i kortområdet. De opføres ikke enkeltvis.
permits-owner-other = Andre offentlige ejere
permits-owners-rest = For resten af området nævner dataene ingen ejer.
permits-monuments-total = Monumenter helt eller delvist i kortområdet: { $count }.{ $links ->
        [yes] {" "}Linkene fører til deres indgange i monumentlisten.
       *[no] {""}
    }
permits-monuments-links-left = { $count } flere indgange tælles kun her, uden link.
permits-object-kind = { $kind ->
        [natural-monument] Naturmonument (Naturdenkmal)
        [protected-landscape-part] Beskyttet landskabselement (geschützter Landschaftsbestandteil)
        [biotope] Lovmæssigt beskyttet biotop (geschütztes Biotop)
        [national-park] Nationalpark (park narodowy)
        [park-buffer-zone] Bufferzone om en park (otulina)
        [landscape-park] Landskabspark (park krajobrazowy)
        [nature-reserve] Naturreservat (rezerwat przyrody)
        [protected-landscape-area] Beskyttet landskabsområde (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000-område for naturtyper (SOO)
        [natura2000-birds] Natura 2000-område for fugle (OSO)
        [ecological-site] Økologisk område (użytek ekologiczny)
        [landscape-complex] Natur- og landskabskompleks (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentationssted (stanowisko dokumentacyjne)
        [natural-monument-pl] Naturmindesmærke (pomnik przyrody)
        [national-park-cz] Nationalpark (národní park, NP)
        [protected-landscape-area-cz] Beskyttet landskabsområde (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Nationalt naturreservat (národní přírodní rezervace, NPR)
        [national-nature-monument] Nationalt naturmonument (národní přírodní památka, NPP)
        [nature-reserve-cz] Naturreservat (přírodní rezervace, PR)
        [nature-monument-cz] Naturmonument (přírodní památka, PP)
        [protective-zone-cz] Beskyttelseszone (ochranné pásmo)
        [nature-zone-cz] Zone for naturbeskyttelse (zóna ochrany přírody)
        [natura2000-habitats-cz] Natura 2000-område for naturtyper (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Natura 2000-område for fugle (ptačí oblast, PO)
        [memorial-tree] Mindetræ (památný strom)
       *[special-rules] Område med særlige regler
    }
permits-object-count = { $count } objekter
permits-object-citations = Retsgrundlag:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] For gader, pladser og grønne områder i Bezirk { $district } er Bezirksamt { $district } som regel ansvarlig, dér Fachamt Management des öffentlichen Raumes (kontoret for det offentlige rum).
       *[berlin] For gader, pladser og grønne områder i Bezirk { $district } er Ordnungsamt (ordensmyndigheden) og Straßen- und Grünflächenamt (vej- og parkkontoret) i Bezirksamt { $district } som regel ansvarlige.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] For monumenterne i kortområdet er Denkmalschutzamt Hamburg (monumentkontoret) ansvarlig.
       *[berlin] For monumenterne i Bezirk { $district } er den lokale monumentmyndighed (untere Denkmalschutzbehörde) i Bezirksamt { $district } ansvarlig.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Om der ligger fredede monumenter i kortområdet, kan dataene her ikke sige med sikkerhed.
       *[yes] Der er fredede monumenter i kortområdet.
    } Om et løb berører dem, for eksempel med en post ved et monument eller en delstrækning gennem et beskyttet område, afgør den lokale monumentmyndighed (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Her er det selve byen { $kreis }.
        [no] , som regel { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; i nogle delstater er det kommunen { $gemeinde }.
        }
       *[unknown] {" "}(som regel Landkreis eller den kreisfreie Stadt, i nogle delstater kommunen).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Naturmonumenter, beskyttede landskabselementer og beskyttede biotoper på kortet hører under den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde). Her er det selve byen { $kreis }.
        [no] Naturmonumenter, beskyttede landskabselementer og beskyttede biotoper på kortet hører under den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde), som regel { $kreis }.
       *[unknown] Naturmonumenter, beskyttede landskabselementer og beskyttede biotoper på kortet hører under den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde), som regel Landkreis eller den kreisfreie Stadt.
    }
permits-hint-biotope = Lovmæssigt beskyttede biotoper ligger i kortområdet{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Alt, der kan ødelægge dem eller skade dem væsentligt, er forbudt (§ 30 BNatSchG). Placér poster og delstrækninger uden for dem, eller spørg den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] For naturmonumenter og beskyttede landskabselementer i Bezirk { $district } er Bezirksamt { $district } som regel ansvarlig.
       *[berlin] For naturmonumenter og beskyttede landskabselementer i Bezirk { $district } er den lokale naturbeskyttelsesmyndighed (untere Naturschutzbehörde) i Bezirksamt { $district } ansvarlig.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Ifølge delstatens åbne data tilhører { $share } af kortområdet andre offentlige ejere.
       *[no] Ifølge delstatens åbne data tilhører { $share } af kortområdet ejeren "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Ejerne af resten af området kan ikke findes i åbne data. Med en legitim interesse (for eksempel som arrangør) kan du bede tinglysningen (Grundbuchamt) om et udtræk; for offentligt areal hjælper det kompetente Ordnungsamt eller parkforvaltningen.
       *[no] Ejerne af disse områder kan ikke findes i åbne data. Med en legitim interesse (for eksempel som arrangør) kan du bede tinglysningen (Grundbuchamt) om et udtræk; for offentligt areal hjælper det kompetente Ordnungsamt eller parkforvaltningen.
    }
permits-section-forest-units = Skovforvaltning ({ $source })
permits-section-state-forest = Delstatsskov ({ $source })
permits-none-forest-units = Ifølge { $source } ligger kortområdet i intet skovdistrikt.
permits-section-admin-units = Administrative enheder ({ $source })
permits-none-admin-units = Ifølge { $source } ligger kortområdet ikke i nogen administrativ enhed i registret.
permits-section-register-monuments = Monumenter i det nationale register ({ $source })
permits-none-register-monuments = Ifølge { $source } ligger der intet registreret monument i rektanglet omkring kortområdet.
permits-register-monuments-total = Registrerede monumenter i rektanglet omkring kortområdet: { $count }. Registeret angiver ingen position, så disse noter kan ikke sige, hvor de ligger; det må tjekkes på kortet.
permits-register-monuments-left = { $count } flere er kun talt med her, uden navne.
permits-section-register-archaeology = Arkæologiske monumenter i det nationale register ({ $source })
permits-none-register-archaeology = Ifølge { $source } ligger der intet registreret arkæologisk monument i rektanglet omkring kortområdet.
permits-register-archaeology-total = Registrerede arkæologiske monumenter i rektanglet omkring kortområdet: { $count }. Registeret angiver ingen position, så disse noter kan ikke sige, hvor de ligger; det må tjekkes på kortet.
permits-register-archaeology-left = { $count } flere er kun talt med her, uden navne.
permits-section-register-history = Historiens mindesmærker ({ $source })
permits-none-register-history = Ifølge { $source } ligger der intet historiens mindesmærke i rektanglet omkring kortområdet.
permits-register-history-in-area = Historiens mindesmærker i kortområdet: { $count }.
permits-register-history-nearby = Yderligere historiens mindesmærker i rektanglet omkring kortområdet, men ikke i kortområdet: { $count }.
permits-register-history-left = { $count } flere er kun talt med her, uden navne.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Ifølge { $source } ligger ingen delstatsskov i kortområdet.
permits-forest-units-reach = Et skovdistrikt er ansvarligt for skoven i sit område. Dets område kan også omfatte byer og marker.
permits-state-forest-office = ansvarlig: { $office }
permits-hint-forest-office = { $units ->
        [none] Skovdistrikt, der er ansvarligt for skoven i kortområdet: { $office } ({ $administration }). Spørg dér i god tid, når du planlægger en begivenhed i skoven.
       *[other] Skovdistrikt, der er ansvarligt for skoven i kortområdet: { $office } ({ $administration }), { $units }. Spørg dér i god tid, når du planlægger en begivenhed i skoven.
    }
permits-hint-forest-owner = { $office ->
        [known] En begivenhed i skoven kræver skovejernes samtykke. Skovdistriktet kan som regel sige, hvem der ejer skoven. I flere delstater skal en organiseret begivenhed i skoven også anmeldes til skovmyndigheden eller tillades af den.
       *[unknown] En begivenhed i skoven kræver skovejernes samtykke. I flere delstater skal en organiseret begivenhed i skoven også anmeldes til skovmyndigheden eller tillades af den. Disse noter nævner ikke det ansvarlige skovdistrikt her; delstatens skovforvaltning eller den lokale skovmyndighed kan fortælle det.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Et arrangement i skoven kræver normalt skovejerens samtykke. Skovdistriktet (Nadleśnictwo), der er nævnt ovenfor, kan som regel sige, hvem der forvalter skoven. Skov, der ikke tilhører Statsskovene, for eksempel privat skov, findes ikke i data for disse noter. Skov, der ikke tilhører staten, for eksempel privat eller kommunal skov, er under tilsyn af starosten (starosta) med hensyn til skovdriften (skovloven, art. 5 ust. 1 pkt 2); spørg dér hos starostwo powiatowe.
       *[unknown] Et arrangement i skoven kræver normalt skovejerens samtykke. Disse noter nævner intet skovdistrikt under Statsskovene for kortområdet; data dækker kun Statsskovene, så skoven kan tilhøre en anden, for eksempel en privat ejer. Hvem ejeren er, må undersøges særskilt. Skov, der ikke tilhører staten, for eksempel privat eller kommunal skov, er under tilsyn af starosten (starosta) med hensyn til skovdriften (skovloven, art. 5 ust. 1 pkt 2); spørg dér hos starostwo powiatowe.
    }
permits-hint-monument-register-pl = Monumenter i det nationale register ligger i rektanglet omkring kortområdet. Deres positioner er ikke i dataene, så tjek på kortet, hvor de er. Efter art. 36 ust. 1 i loven om beskyttelse af monumenter (ustawa o ochronie zabytków) kræver konserverings-, restaurerings- og byggearbejde på et registreret monument tilladelse fra voivodskabets monumentkonservator (Wojewódzki Konserwator Zabytków). Siden 3. juni 2026 kræver byggearbejde i et monuments omgivelser, ændret brug, anordninger eller skilte sat op på det og andre handlinger, der kan ændre dets substans eller udseende, kun en underretning til konservatoren (art. 36aa); konservatoren kan gøre indsigelse inden for 60 dage og så kræve en tilladelse. Loven nævner ikke et løb eller en post som sådan; spørg konservatorens kontor, hvis et arrangement berører et monument eller dets omgivelser.
permits-hint-archaeology-register-pl = Arkæologiske monumenter i det nationale register ligger i rektanglet omkring kortområdet. Deres positioner er ikke i dataene, så tjek på kortet, hvor de er. Efter art. 36 ust. 1 i loven om beskyttelse af monumenter (ustawa o ochronie zabytków) kræver arkæologiske undersøgelser (pkt 5), eftersøgning af skjulte eller forladte løsøremonumenter med elektroniske eller tekniske apparater som metaldetektorer (pkt 12) og konserverings-, restaurerings- eller byggearbejde på et registreret monument (pkt 1) tilladelse fra voivodskabets monumentkonservator (Wojewódzki Konserwator Zabytków). Tilladelsen til eftersøgning bortfalder den 1. januar 2027: art. 36b tillader da en voksen at søge med samtykke fra ejendommens ejer og besidder efter en anmeldelse i søgeregisteret via kulturministeriets app, og art. 36c forbyder sådan søgning blandt andet på registrerede monumenter og tættere end 10 m på dem. Loven nævner ikke en gåtur, et løb eller en post som sådan; spørg bevarerens kontor, hvis en begivenhed eller feltarbejde berører et registreret sted.
permits-hint-history-register-pl = Et historiens mindesmærke (Pomnik historii) ligger i eller nær kortområdet; afsnittet om historiens mindesmærker siger hvilket. Den polske republiks præsident erklærer ved forordning et urørligt monument, der er optaget i registret, eller en kulturpark af særlig værdi for et Pomnik historii (art. 15 ust. 1 i loven om beskyttelse af monumenter, ustawa o ochronie zabytków). Det står altså allerede under beskyttelse af registret eller kulturparken. Arbejder på et monument, der er optaget i registret, kræver tilladelse fra vojvodskabets monumentkonservator (Wojewódzki Konserwator Zabytków) (art. 36 ust. 1); i en kulturpark fastsætter kommunalbestyrelsens beslutning forbud og begrænsninger (art. 16 ust. 2, art. 17 ust. 1). Loven nævner hverken gåtur, løb eller post som sådan; spørg konservatorens kontor, hvis et arrangement eller feltarbejde berører monumentet.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Kortområdet rummer et naturmindesmærke (pomnik przyrody), et økologisk område (użytek ekologiczny), et natur- og landskabskompleks (zespół przyrodniczo-krajobrazowy) eller et dokumentationssted (stanowisko dokumentacyjne). Kommunerådet (rada gminy) opretter dem ved en beslutning, og beslutningen angiver, hvem der fører tilsyn med objektet, og hvilke forbud der gælder (art. 44 i naturbeskyttelsesloven). Loven selv nævner ingen tilladelse til en begivenhed her. Spørg kommunekontoret i den gmina, hvor objektet ligger, om beslutningen og tilsynet.
       *[other] Kortområdet rummer et naturmindesmærke (pomnik przyrody), et økologisk område (użytek ekologiczny), et natur- og landskabskompleks (zespół przyrodniczo-krajobrazowy) eller et dokumentationssted (stanowisko dokumentacyjne). Kommunerådet (rada gminy) opretter dem ved en beslutning, og beslutningen angiver, hvem der fører tilsyn med objektet, og hvilke forbud der gælder (art. 44 i naturbeskyttelsesloven). Loven selv nævner ingen tilladelse til en begivenhed her. Spørg { $count ->
        [one] kommunekontoret i gminaen ({ $gmina }), hvor objektet ligger
       *[other] kommunekontorerne i gminaerne ({ $gmina }), hver for objekterne på sit område
    }, om beslutningen og tilsynet.
    }
permits-hint-gmina-roads-pl = En begivenhed, der hindrer trafikken eller bruger en vej på en særlig måde, kræver en tilladelse efter art. 65 i færdselsloven, søgt mindst 30 dage i forvejen. Den udstedes af den myndighed, der styrer trafikken på vejen: starosten for powiat- og gminavejene, borgmesteren i en by med powiatstatus, voivodskabets marskal eller GDDKiA for de øvrige veje (art. 10 og 65b). Forvalter af en gminavej er wójt, burmistrz eller prezydent miasta { $count ->
        [one] i gminaen ({ $gmina });
       *[other] i hver af gminaerne ({ $gmina });
    } forvalteren afgør, om vejarealet må optages (art. 19 og 40 i loven om offentlige veje). Disse noter viser ikke vejenes kategori.
permits-hint-gmina-land-pl = Hvis jord i kortområdet tilhører gminaen, for eksempel en kommunal park, en plads, en skolegrund eller en skov, forvaltes den af wójt, burmistrz eller prezydent miasta, som repræsenterer gminaen (art. 30 og 31 i loven om gminaens selvstyre). Bed { $count ->
        [one] kommunekontoret i gminaen ({ $gmina }) om samtykke
       *[other] kommunekontorerne i gminaerne ({ $gmina }) om samtykke
    }, før du bruger den. Disse noter viser ikke, hvem der ejer jorden.
permits-hint-protected-areas-cz = Beskyttede områder eller naturbeskyttelsesobjekter ligger i kortområdet. Efter loven om natur- og landskabsbeskyttelse (lov nr. 114/1992 Sb.) er organiserede sports-, turist- og andre offentlige arrangementer i en nationalpark forbudt uden for de steder, som naturbeskyttelsesmyndigheden har udpeget (§ 16 odst. 1 písm. q), og i en nationalparks beskyttelseszone kræver sådanne massearrangementer uden for disse steder og uden for bebyggede områder myndighedens samtykke (§ 37 odst. 3 písm. d). I et nationalt naturreservat er det forbudt at færdes eller køre uden for de stier, der er afmærket med myndighedens samtykke (§ 29 písm. d). Naturbeskyttelsesmyndigheder er blandt andre en nationalparks forvaltning, Naturbeskyttelsesagenturet (AOPK ČR), regionskontoret (krajský úřad) og kontoret i en kommune med udvidede beføjelser (§ 75 odst. 1). Andre beskyttede områder og objekter har egne beskyttelsesbetingelser. Spørg myndigheden tidligt, når du planlægger et arrangement der.
permits-hint-monuments-cz = Kulturmonumenter eller monumentområder på den statslige liste ligger i kortområdet. Efter loven om statslig monumentpleje (lov nr. 20/1987 Sb., § 14 odst. 1 og 2) skal ejeren af et monument på forhånd søge om en afgørelse eller en bindende udtalelse, før monumentet eller dets omgivelser vedligeholdes, repareres, ombygges eller ændres på anden måde. Det samme gælder ejeren af en ejendom i et monumentreservat, en monumentzone eller et monuments beskyttelseszone ved en bygning, en ændring af en bygning, terrænarbejder, en anordning eller et skilt, fjernelse af et bygværk, behandling af træer eller vedligeholdelsesarbejder. Kontoret i kommunen med udvidede beføjelser afgør{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, for et nationalt kulturmonument regionskontoret. Loven nævner ingen gåtur, intet løb og ingen post som sådan; spørg dette kontor, hvis et arrangement berører et monument eller sætter anordninger eller skilte op ved det.
permits-hint-forest-owner-cz = Et socialt massearrangement, for eksempel et sportsligt eller kulturelt, på arealer med skovfunktion må kun afholdes med skriftligt samtykke fra ejeren af den skov, hvor det finder sted (skovloven, lov nr. 289/1995 Sb., § 20 odst. 5). Arrangøren skal desuden anmelde det skriftligt til kontoret i kommunen med udvidede beføjelser{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, eller til regionskontoret, hvis arrangementet rækker ud over en sådan kommunes område, senest 30 dage før arrangementet. Anmeldelsen angiver dato og sted, hvordan arrangementet bemandes, det forventede antal deltagere og ejernes skriftlige samtykker (samme paragraf). Disse noter viser ikke, hvem der ejer skoven; spørg skovejerne tidligt.
permits-hint-roads-cz = Et arrangement, der bruger en vej på en anden måde end sædvanlig, for eksempel et sportsligt, kulturelt eller lignende arrangement, der kan true færdselssikkerheden eller trafikafviklingen, kræver tilladelse fra den kompetente vejmyndighed. Den udstedes med forudgående samtykke fra vejens ejer og, hvor færdselssikkerheden eller trafikafviklingen kan blive berørt, fra Tjekkiets politi (vejloven, lov nr. 13/1997 Sb., § 25 odst. 1 og odst. 6 písm. d). Myndigheden er kommunekontoret for lokale veje eller kontoret i kommunen med udvidede beføjelser, hvor det afgør, og kontoret i kommunen med udvidede beføjelser for veje af anden og tredje klasse (§ 40 odst. 4 og 5). Kommuner i kortområdet: { $gmina }. Disse noter viser ikke vejenes klasse.
permits-section-water-protection = Vandbeskyttelsesområder ({ $source })
permits-section-forest-functions = Skovfunktioner og beskyttet skov ({ $source })
permits-none-water-protection = Ifølge { $source } ligger intet vandbeskyttelsesområde i kortområdet.
permits-none-forest-functions = Ifølge { $source } ligger ingen rekreativ skov, ingen beskyttet skov, ingen naturskov og intet lukket skovområde i kortområdet.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreativ skov (Erholungswald)
        [protected] Beskyttet skov (Bannwald eller Schutzwald)
        [natural] Naturskov eller strengt reservat (Naturwald, Totalreservat)
       *[closed] Lukket skovområde
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } i vandbeskyttelsesområdet "{ $name }" ligger i kortområdet. I zone I og II er begivenheder som regel udelukket. Spørg den lokale vandmyndighed (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } i vandbeskyttelsesområdet "{ $name }" ligger i kortområdet. Begivenheder dér er som regel mulige; den lokale vandmyndighed (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) kan sige, hvad forordningen tillader.
       *[unknown] Vandbeskyttelsesområdet "{ $name }" ligger i kortområdet. Dataene siger ikke, hvilken zone det er. I zone I og II er begivenheder som regel udelukket; spørg den lokale vandmyndighed (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Rekreativ skov (Erholungswald) ligger i kortområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Spørg skovmyndigheden om begivenheder dér.
        [protected] Beskyttet skov (Bannwald eller Schutzwald) ligger i kortområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Spørg skovmyndigheden, før du planlægger en begivenhed dér.
        [natural] Naturskov eller et strengt reservat ligger i kortområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Det er ofte forbudt at forlade stierne dér; placér ingen poster i den uden at spørge skovmyndigheden.
       *[closed] Et skovområde, der ikke må betrædes, ligger i kortområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Hold det uden for banen.
    }
permits-section-roads = Klassificerede veje ifølge OpenStreetMap
permits-none-roads = Ifølge OpenStreetMap løber ingen Autobahn og ingen Bundes-, Landes-, Staats- eller Kreisstraße gennem kortområdet.
permits-road-line = { $class ->
        [autobahn] Autobahn (motorvej)
        [bundesstrasse] Bundesstraße (forbundsvej)
        [landesstrasse] Landesstraße eller Staatsstraße (delstatsvej)
       *[kreisstrasse] Kreisstraße (kredsvej)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } løber gennem kortområdet. En bane må kun krydse den på broer eller gennem underføringer.
        [bundesstrasse] Bundesstraße { $refs } løber gennem kortområdet. En afspærring eller en krydsning med posteringer beordres af færdselsmyndigheden (Straßenverkehrsbehörde); vejen vedligeholdes som regel af Bund gennem delstatens vejforvaltning, i større byer ofte af byen.
        [landesstrasse] Landes- eller Staatsstraße { $refs } løber gennem kortområdet. En afspærring eller en krydsning med posteringer beordres af færdselsmyndigheden (Straßenverkehrsbehörde); vejen vedligeholdes som regel af delstaten, i større byer ofte af byen.
       *[kreisstrasse] Kreisstraße { $refs } løber gennem kortområdet. En afspærring eller en krydsning med posteringer beordres af færdselsmyndigheden (Straßenverkehrsbehörde); vejen vedligeholdes som regel af Kreis, i større byer ofte af byen.
    }
permits-hint-state-forest = { $office ->
        [none] Ifølge delstatens åbne data er { $share } af kortområdet delstatsskov tilhørende ejeren "{ $owner }".
       *[other] Ifølge delstatens åbne data er { $share } af kortområdet delstatsskov tilhørende ejeren "{ $owner }". Ansvarlig: { $office }.
    }
