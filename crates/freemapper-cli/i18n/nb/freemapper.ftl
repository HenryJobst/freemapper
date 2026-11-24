# Programtekster for freemapper, norsk bokmål (ADR 0092). Engelsk er referansen: alle andre språk
# har de samme meldings-ID-ene og de samme variablene. Tekster fra andre programmer (feil fra
# operativsystemet, meldinger fra parsere) kommer i variabler og står uendret.

## The --lang switch

cli-lang-unknown = { $value } er ikke et språk freemapper har tekster for; tilgjengelige: { $available }

## init

cli-part-needs-osm = --part velger en del av et OSM-område og krever --osm; med --place slår du først opp ID-en og bruker --osm <id> --part <n>
cli-reading-file = leser { $path }
cli-no-area-for-place = fant ikke noe område for "{ $query }". Hvis stedet er et område av flere atskilte deler, velg én med --osm <id> --part <n>, eller bruk --geojson
cli-using-place = Bruker { $name } ({ $id })
cli-created-project = Opprettet prosjektet { $path }

## Cache

cli-no-cache-dir = ingen hurtigbuffermappe på dette systemet; oppgi --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Hentet 1 flis
       *[other] Hentet { $count } fliser
    }
cli-wrote = Skrev { $path }
cli-kept-map = Beholdt { $path } (den kan inneholde arbeidet ditt)
cli-map-lacks-templates = Kartet refererer ennå ikke til bildemalene. Legg dem til i OOM (Templates > Open template…) eller bytt ut kartet med --force, som beholder det gamle som .omap.bak.
cli-map-lacks-buildings = Kartet har ingen ALKIS-bygninger ennå. For å legge dem til, bytt ut kartet med --force, som beholder det gamle som .omap.bak.
cli-map-lacks-canopies = Kartet mangler ALKIS-takene og carportene (522) som nå er kartobjekter. For å legge dem til, kjør med --force, som beholder det gamle kartet som .omap.bak.
cli-map-light = Grunnkartet: { $joined } linjer slått sammen, { $left } skjulte eller for små igjen for kandidatlagene (ingenting flyttet)
cli-map-light-kept = Kandidatlagene inneholder nå også { $left } objekter eller strekninger som denne versjonen tar av grunnkartet (merket freemapper:generalized). Det beholdte kartet har dem fortsatt, så de vises to ganger. Bytt ut kartet med --force for å rydde det (det gamle blir liggende som .omap.bak).
cli-map-old-candidates = Kartet navngir fortsatt templates/candidates.omap; denne kjøringen fylte den med alle kandidater. Kandidatene kommer nå også i sju lag, én fil hver (templates/candidates-*.omap). Kartet lister dem etter at du har byttet det ut med --force (det gamle blir liggende som .omap.bak), eller du legger dem til selv i OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF uten kildehenvisninger: ODbL og dl-de/by-2-0 krever at du oppgir kildene når du gir kartet videre; se ATTRIBUTION.txt
cli-wrote-pdf = Skrev { $path } (kartet uten kandidater)
cli-wrote-pdf-candidates = Skrev { $path } (kontrollutskriften: kartet med alle kandidater, posisjonstro; sjekk dem i terrenget)
cli-wrote-training = Skrev { $map } og { $pdf } (treningskartet: { $joined } slått sammen, { $shifted } flyttet, { $left } utelatt eller kuttet)
cli-training-free-ends = Linjeender latt stå frie: { $ways } der en vei møtte en annen vei, { $others } ved andre linjer
cli-training-apart-left = treningskartet har fortsatt { $length } mm (på papiret) der objekter har for liten avstand; sjekk det i OOM
cli-kept-training = Beholdt treningskartet { $path }, slik at en kjøring ikke overskriver et kart som er skrevet ut eller gitt videre; slett det eller bruk --force for å lage det på nytt

## OCAD

cli-ocad-kept = Beholdt { $path } (den kan inneholde arbeidet ditt; --force erstatter den og beholder den gamle som .ocd.bak)
cli-ocad-old-candidates = OCAD-kartet navngir fortsatt { $path } som bakgrunnskart; denne kjøringen fylte den med alle kandidater. Kandidatene kommer nå også i sju lag, én fil hver (-candidates-*.ocd). OCAD-kartet lister dem etter at du har byttet det ut med --force (det gamle blir liggende som .ocd.bak), eller du legger dem til selv i OCAD som bakgrunnskart.
cli-ocad-wrote = { $count ->
        [one] Skrev { $path } (1 objekt)
       *[other] Skrev { $path } ({ $count } objekter)
    }
cli-ocad-left-out = Utelatt fra OCAD-kartet: { $skipped } objekter (ingen symbol for OCAD, ikke noe område, eller utenfor OCADs tegneområde på ±2 m) og { $dropped } degenererte linjedeler eller hull.
cli-ocad-load-symbols = I OCAD velger du Symbol > Load Colors and Symbols From… og peker på det offisielle symbolsettet{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOFs skolesett 2019
       *[other] {" "}ISSprOM 2019
    }; det erstatter freemappers plassholdersymboler etter nummer.

## select

cli-select-running = Områdevalg for { $path }: trykk "Ferdig" på siden eller Ctrl+C for å stoppe.
cli-select-no-browser = Kunne ikke åpne nettleseren ({ $error }); åpne adressen ovenfor.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; ikke noe område, kan ikke bli kartområde): { $place }
cli-find-hint = Opprett prosjektet med: freemapper init <dir> --osm { $id }
cli-find-no-area = Ingen av disse stedene har et område som kan bli kartområde. Tegn området med: freemapper select <dir>
cli-find-nothing = fant ikke noe sted for "{ $query }"; prøv et annet navn, eller tegn området med: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = advarsel: adapteren i { $folder } er utelatt: { $finding }
cli-adapter-note = advarsel: adapteren i { $folder } lastes, men: { $finding }
cli-adapter-from-project = advarsel: adapteren { $adapter } fra prosjektmappen er i bruk; den kan spørre disse vertene: { $hosts }
cli-adapter-overlaps = advarsel: adapteren { $adapter } dekker også områder som hører til { $others }; objektene fra den havner på kart der
cli-adapter-dir-relative = advarsel: FREEMAPPER_ADAPTER_DIR er ikke en absolutt sti ({ $path }); den ignoreres
cli-adapters-no-project = advarsel: { $path } er ikke en mappe; ingen prosjektadaptere ble lest
cli-adapters-overlaps = deler område med: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revisjon { $revision }  sjekket { $checked }  verter: { $hosts }
cli-adapters-source-built-in = innebygd
cli-adapters-source-user = fra mappen din
cli-adapters-source-project = fra prosjektet
cli-adapters-never-checked = aldri
cli-adapters-user-dir = Adapterne dine leses fra { $path }
cli-adapters-no-user-dir = Dette systemet har ingen mappe for adapterne dine; FREEMAPPER_ADAPTER_DIR kan angi en

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper kan nå også spørre { $host } for { $name }. Oppføringen står i { $file }; `freemapper adapters disallow-host { $adapter } { $host }` tar den tilbake. En områdeside eller en AI-assistent som kjører, leser oppføringen når den startes på nytt
cli-allow-host-already = { $host } er allerede tillatt for { $name }
cli-allow-host-declared = adapteren for { $name } navngir { $host } selv; ingenting å tillate
cli-allow-host-refused = { $host } kan ikke tillates: { $reason }
cli-allow-host-unknown-adapter = det finnes ingen innebygd adapter { $adapter }; de innebygde adapterne er: { $adapters }
cli-allow-host-folder-adapter = { $adapter } er en adapter i adaptermappen din; legg verten til i `[net] hosts` i manifestet i stedet
cli-allow-host-no-config = dette systemet har ingen konfigurasjonsmappe å lagre tillatte verter i
cli-allow-host-write = { $file } kunne ikke skrives: { $detail }
cli-allow-host-unreadable = { $file } kan ikke leses ({ $detail }); rett opp eller fjern filen først; ingenting ble endret
cli-disallow-host-removed = freemapper spør ikke lenger { $host } for { $name }
cli-disallow-host-absent = { $host } var ikke tillatt for { $name }; ingenting å ta tilbake
cli-allowed-hosts-unreadable = advarsel: { $file } kunne ikke leses ({ $detail }); ingen ekstra verter er tillatt
cli-allowed-hosts-ignored = advarsel: { $file }: oppføringen { $host } for { $adapter } ignoreres: { $reason }
cli-allowed-hosts-no-adapter = freemapper har ingen innebygd adapter med det navnet
cli-allowed-host-redundant = advarsel: { $host } er nå en av vertene adapteren for { $name } navngir; oppføringen din i { $file } ble fjernet
cli-adapters-allowed = tillatt av deg: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revisjon { $revision }, verter: { $hosts }
cli-check-error = feil: { $finding }
cli-check-warning = advarsel: { $finding }
cli-check-passed = { $folder } består sjekken
cli-check-passed-warnings = { $folder } består sjekken, med { $count ->
        [one] én advarsel
       *[other] { $count } advarsler
    }
cli-check-failed = { $folder } består ikke sjekken
cli-check-no-folder = det finnes ingen mappe { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } FEILET: { $detail }
cli-check-live-not-asked = { $capability } ikke spurt: { $detail }
cli-check-live-box-outside = boksen ligger ikke innenfor grensen til adapteren { $adapter }; en live-sjekk spør tjenestene om et sted adapteren dekker
cli-check-live-cannot = live-sjekken kan ikke kjøres: { $detail }
cli-check-detail-not-declared = adapteren erklærer ingen
cli-check-detail-stopped = en tidligere del feilet
cli-check-detail-see-warnings = se advarslene
cli-check-detail-outside-dataset = boksen ligger utenfor datasettet
cli-check-detail-objects = { $count ->
        [one] ett objekt
       *[other] { $count } objekter
    }
cli-check-detail-layers = { $layers } lag, { $count } objekter
cli-check-detail-trees = { $count ->
        [one] ett tre
       *[other] { $count } trær
    }
cli-check-detail-extract = { $label }, data fra { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = terrengmodell
cli-check-raster-surface = overflatemodell
cli-check-raster-tiles = { $product }: { $count ->
        [one] én flis
       *[other] { $count } fliser
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } og { $last }

## Folder lock

cli-lock-held = en annen freemapper-kjøring bruker denne mappen (siden { $time }, startet av { $who }). Vent på den eller stopp den først.
cli-lock-who-cli = terminalen
cli-lock-who-mcp = en AI-agent (MCP)
cli-lock-who-page = valgsiden
cli-lock-who-unknown = et annet program
cli-lock-failed = kunne ikke låse prosjektmappen for denne kjøringen: { $error }
cli-lock-stuck = noe annet enn en låsefil ligger i veien og kunne ikke fjernes

## Run routes of the selection page

cli-run-bad-request = Kjørealternativene er ugyldige: { $error }
cli-run-no-project = Det finnes ikke noe lagret prosjekt i denne mappen ennå. Lagre området først.
cli-run-already-running = En kjøring ({ $job }) pågår allerede for dette prosjektet. Vent til den er ferdig eller avbryt den først.
cli-run-disk-stops = Ikke nok ledig diskplass til nedlastingen: { $messages }
cli-run-busy-elsewhere = Et annet prosjekt i dette programmet kjører en pipeline akkurat nå. Prosjektene deler nedlastingsbufferet, så bare én kjøring går om gangen. Prøv igjen når den er ferdig.
cli-run-nothing-to-cancel = Ingen kjøring pågår, så det er ingenting å avbryte.
cli-run-unknown-job = Det finnes ingen kjøring { $job } for dette prosjektet.
cli-status-too-fast = Statusforespørslene kommer for ofte. Spør høyst fem ganger i sekundet.
cli-open-folder-failed = Mappen kunne ikke åpnes ({ $error }).
cli-open-folder-no-opener = Denne datamaskinen har ikke noe program for å åpne mapper ({ $program }). Åpne mappen manuelt.
cli-open-folder-missing = Prosjektmappen finnes ikke ennå. Lagre området først.
cli-open-folder-refused = Mappenavnet er ikke trygt å gi videre til filbehandleren.
cli-save-running = En kjøring pågår for dette prosjektet. Lagre igjen når den er ferdig.
cli-save-unknown-kind = Ukjent karttype "{ $kind }". Bruk schoolyard, sprint eller forest.
cli-save-scale-too-large = Målestokken { $scale } er for stor.
cli-save-bad-content = Innholdsinnstillingene er ugyldige. Ingenting ble lagret.
cli-map-image-none = Lag kartet først.
cli-map-image-gone = Denne forhåndsvisningen er ikke lenger tilgjengelig.
cli-map-image-running = En kjøring pågår for dette prosjektet. Forhåndsvisningen kan beregnes når den er ferdig.
cli-map-image-stale = Det lagrede prosjektet stemmer ikke lenger med det genererte kartet, for eksempel fordi området, karttypen eller målestokken er endret. Start først en kjøring som erstatter kartet ("Erstatt det eksisterende kartet" på siden, eller freemapper run --force).
cli-map-image-superseded = Det ble bedt om en nyere forhåndsvisning i mellomtiden, så denne ble ikke beregnet.
cli-map-image-bad-content = Innholdsinnstillingene er ugyldige.
cli-map-image-bad-request = Forespørselen om forhåndsvisning er ugyldig. Send et JSON-objekt, eventuelt med innholdsinnstillingene.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – valg av kartområde
page-heading = Velg kartområdet
page-update-available = freemapper { $version } er tilgjengelig. Oppdater med
page-update-notes = Utgivelsesnotater
page-folder = Prosjektmappe
page-pick = Søk etter et sted
page-query-placeholder = f.eks. Clara-Zetkin-Schule Strausberg
page-search-button = Søk
page-searching = Søker …
page-no-results = Fant ingenting.
page-point-only = bare et punkt, ikke et område
page-draw = Tegn selv
page-rectangle = Rektangel
page-polygon = Polygon
page-finish-polygon = Lukk polygon
page-clear = Tøm
page-hint-rectangle = Klikk på to motstående hjørner på kartet.
page-hint-polygon = Klikk på hjørnene ett etter ett. For å lukke polygonet klikker du på første hjørne, dobbeltklikker eller klikker "Lukk polygon". Esc avbryter, Backspace fjerner siste hjørne.
page-hint-idle = Velg et sted fra søket eller tegn området selv.
page-summary = Område
page-source-osm = Fra OpenStreetMap: { $name }
page-source-drawn = Tegnet for hånd
page-size = Størrelse med buffer: { $width } × { $height } m
page-tiles-one = { $n } flis: { $names }
page-tiles-other = { $n } fliser: { $names }
page-download = Nedlasting: ca. { $mb } MB
page-download-none = Nedlasting: alt ligger allerede i hurtigbufferet
page-download-details = Hva som lastes ned
page-cached = i hurtigbufferet
page-disk = Diskplass: { $volumes }
page-disk-volume = { $free } ledig, ca. { $needed } trengs
page-cache-volume = hurtigbuffer
page-project-volume = prosjektmappe
page-no-room = får ikke plass på disken
page-no-room-skip = Ikke alt får plass på disken (freemapper holder { $reserve } ledig). Hentingen utelater de merkede delene. Frigjør plass først hvis du trenger dem:
page-no-room-stop = Ikke nok diskplass (freemapper holder { $reserve } ledig). Det finnes ikke noe kart uten de merkede delene, så hentingen stopper før den laster ned noe. Frigjør plass først:
page-disk-unknown = freemapper kan ikke lese ledig diskplass ({ $error }). Den henter uten sjekk.
page-too-large = Området er for stort: høyst { $max } m per side (med buffer). Velg et mindre.
page-outside = Området ligger utenfor de støttede delstatene ({ $states }); freemapper har ennå ingen datakilder for andre regioner.
page-outside-unknown = Området ligger utenfor regionene freemapper støtter; det har ennå ingen datakilder der.
page-adapters-failed = Flybildene kan ikke listes ({ $detail }). Kartet viser bare OpenStreetMap; alt annet fungerer.
page-settings = Kart
page-kind = Karttype
page-schoolyard = Skolegårdskart (1:1000)
page-sprint = Sprintkart (1:4000)
page-forest = Skogskart (1:15000)
page-scale = Målestokk 1:
page-map-title = Tittel
page-contour-interval = Ekvidistanse
page-contour-auto = automatisk
page-settings-fixed = Prosjektet finnes allerede; karttype, målestokk og tittel står i freemapper.toml. Ekvidistansen kan fortsatt endres.
page-save = Lagre
page-saving = Lagrer …
page-saved = Lagret i freemapper.toml.
page-zone-changed = Lagret. Området ligger nå i en annen UTM-sone; freemapper henter dataene på nytt ved neste kjøring.
page-provider-changed = Lagret. En annen dataleverandør dekker høyde og flybilder for det nye området; freemapper henter dataene på nytt ved neste kjøring.
page-save-stale = freemapper.toml er endret i et annet vindu eller av et annet program siden denne siden leste den. Ingenting ble lagret; siden kjenner nå den lagrede tilstanden. Lagre igjen for å erstatte den med området som vises her.
page-has-map = Det eksisterende kartet (.omap) blir som det er; freemapper lager nye maler og kandidater for det nye området. Valget "Erstatt det eksisterende kartet" nedenfor (eller "freemapper run --force") lager også kartet på nytt og beholder det gamle som .omap.bak.
page-done = Ferdig
page-source = Kildekode (AGPL-3.0-or-later)
page-closed = Ferdig. Du kan lukke dette vinduet.
page-next = Neste: freemapper run "{ $dir }"
page-made-files = Opprettet: { $files }. Mappe: { $dir }
page-back-to-agent = Si til AI-assistenten din at området er lagret.
page-nothing-saved = Ingenting ble lagret.
page-failed = Feil: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Flyfoto { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Hva som havner på kartet
page-content-help = Profilen avgjør hva som havner direkte på kartet og hva som forblir en kandidat som må sjekkes. Et valg for en gruppe går foran profilen. Det lagres sammen med området.
page-content-profile-legend = Profil
page-content-profile-safe = Trygg
page-content-profile-safe-hint = Hvert objekt blir der byggingen plasserer det: bygninger og matrikkelflater havner på kartet; murer, gjerder, trær og mye fra OpenStreetMap (dammer, stier uten kjent underlag) forblir kandidater.
page-content-profile-balanced = Balansert
page-content-profile-balanced-hint = Objekter fra matrikkelen, registrene og gatekartleggingen havner på kartet, sammen med høydekurvene og forbudsområdene langs jernbanespor. Murer, gjerder, hekker, skråninger, trapper, vann, trær, minnesmerker, passasjer, stier og jernbaner kommer også fra OpenStreetMap. Gatemøbler, de andre forbudsområdene og formene fra terrengmodellen forblir kandidater.
page-content-profile-full = Full
page-content-profile-full-hint = Alt havner på kartet, som i kontrollutskriften; bare skjulte kandidater blir utenfor.
page-content-groups-legend = Etter gruppe
page-content-groups-help = "Som i profilen" følger profilen. Hva det betyr for en gruppe, avhenger av hvor hvert objekt kommer fra.
page-content-place-profile = Som i profilen
page-content-place-map = Kart
page-content-place-candidates = Kandidat
page-content-place-off = Av
page-content-group-walls = Murer
page-content-group-fences = Gjerder
page-content-group-hedges = Hekker
page-content-group-slopes = Skråninger og stup
page-content-group-stairs = Trapper
page-content-group-trees = Trær og tregrupper
page-content-group-water-features = Vann, fontener og kilder
page-content-group-monuments = Tårn, minnesmerker og kraftlinjer
page-content-group-furniture = Gatemøbler og lamper
page-content-group-passages = Passasjer
page-content-group-contours = Høydekurver
page-content-group-landforms = Knauser, forsenkninger, skråninger og stup fra terrengmodellen
page-content-group-vegetation = Vegetasjon fra laserpunktskyer (Brandenburg, Berlin, Nordrhein-Westfalen, Rheinland-Pfalz, Saarland og Bayern; av til du velger det; laster ned ca. 40 til 600 MB per km²)
page-content-group-out-of-bounds = Forbudsområder
page-content-group-paths = Stier og jernbaner
page-content-group-streets = Gater og plasser
page-run-heading = Lag kartet
page-preview-heading = Forhåndsvisning av kartet
page-preview-stored = Som sist generert.
page-preview-live = Med innstillingene på denne siden, ikke lagret ennå.
page-preview-live-saved = Med de lagrede innstillingene, ikke generert i en kjøring ennå.
page-preview-computing = Beregner …
page-preview-none = Ingen kart generert ennå.
page-preview-later = Dette kartet har ingen forhåndsvisning ennå. Den kommer med neste kjøring som skriver kartet.
page-preview-stale = Området, karttypen eller målestokken er endret siden kartet ble generert. Bildet viser fortsatt kartet som sist generert; en ny forhåndsvisning krever en kjøring med "Erstatt det eksisterende kartet".
page-preview-area-unsaved = Området på denne siden er ikke lagret ennå. Bildet viser kartet som sist generert; det nye området krever en kjøring med "Erstatt det eksisterende kartet".
page-preview-run = Som sist generert. Siden oppdaterer bildet når kjøringen er ferdig.
page-preview-open = Åpne i full størrelse
page-preview-alt = Forhåndsvisning av kartobjektene
page-opt-ocad = Skriv et OCAD-startprosjekt
page-help-ocad = For alle som jobber videre i OCAD.
page-opt-pdf = PDF til utskrift
page-help-pdf = Bare kartobjekter, ingen kandidater.
page-opt-pdf-candidates = Kontrollutskrift med alle kandidater
page-opt-training = Treningskart
page-help-training = Kartet og alle kandidater, ryddet slik at det kan skrives ut som det er; med OCAD også som OCAD-fil.
page-views-heading = Visninger av treningskartet
page-view-no-paths = Uten stier (Ultimate-kart)
page-view-brown-only = Bare brunt (høydekurve-orientering)
page-view-no-north-lines = Uten nordlinjer
page-view-custom = En annen visning
page-help-views = Hver avmerkede visning blir én fil til ved siden av treningskartet, `<name>-training-<view>.omap` og dens PDF, med OCAD også en OCAD-fil. En annen visning er `no:505,506` (utelat symboler) eller `only:101,102` (behold bare disse); `+` kombinerer visninger. Tittel, målestokk og kilder blir alltid stående.
page-help-views-forest = Uten stier og bare brunt gjelder bare skogskart.
page-opt-scale-bar = Målestokklinjal på PDF-ene
page-help-scale-bar = Tyske kart har vanligvis ingen.
page-opt-sources = Kildelinjer på PDF-ene
page-sources-auto = automatisk
page-sources-full = full
page-sources-credit-only = bare freemapper
page-warn-credit-only = PDF-ene navngir da bare freemapper. Datalisensene (ODbL, dl-de/by-2-0) krever kildehenvisning når du gir kartet videre; da navngir du kildene selv. ATTRIBUTION.txt lister dem alle.
page-need-pdf = Målestokklinjalen og kildelinjene gjelder bare når det lages en PDF.
page-opt-offline = Bare hurtigbuffer, ingen nettverk
page-opt-force = Erstatt de eksisterende kartene
page-help-force = Kartet og, med "Treningskart" avmerket, treningskartet. De gamle filene blir liggende som .omap.bak.
page-opt-permits = Tillatelsesnotater
page-help-permits = Administrative enheter, verneområder og hvem du bør spørre, i permits.md. Et hint, ikke juridisk rådgivning.
page-run-start = Lag kart
page-run-again = Lag på nytt
page-run-cancel = Avbryt
page-run-cancelling = Avbryter …
page-warnings-heading = Advarsler
page-where-files = Filene ligger i prosjektmappen:
page-open-folder = Åpne mappen
page-log-label = Logg fra kjøringen
page-force-title = Erstatte de eksisterende kartene?
page-force-text = Det eksisterende kartet erstattes, og med "Treningskart" avmerket også treningskartet. Redigeringene dine går ikke tapt: de gamle filene blir liggende i prosjektmappen som .omap.bak. Fortsette?
page-force-cancel = Avbryt
page-force-confirm = Erstatt
page-step-fetch = Henter data
page-step-derive = Beregner maler
page-step-map = Skriver kartet
page-step-ocad = Skriver OCAD-prosjektet
page-step-pdf = Skriver PDF-en
page-step-pdf-candidates = Skriver kontrollutskriften
page-step-training = Skriver treningskartet
page-step-permits = Skriver tillatelsesnotatene
page-step-line = Trinn { $index } av { $total }: { $label }
page-starting = Kjøringen starter …
page-file-kind-map = kart
page-file-kind-template = mal
page-file-kind-ocad = OCAD-kart
page-file-kind-ocad-candidates = OCAD-kandidater
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrollutskrift
page-file-kind-training-map = treningskart
page-file-kind-training-pdf = PDF av treningskartet
page-file-kind-training-ocad = OCAD-treningskart
page-file-kind-training-view-map = visning av treningskartet
page-file-kind-training-view-pdf = PDF av visning av treningskartet
page-file-kind-training-view-ocad = OCAD-visning av treningskartet
page-file-kind-preview = forhåndsvisning av kartet
page-file-kind-permits = tillatelsesnotater
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Ferdig. Filene er skrevet.
page-result-failed = Kjøringen feilet: { $error }
page-result-failed-kept = Det som er skrevet, blir liggende i prosjektmappen; en ny kjøring fortsetter derfra.
page-result-cancelled = Avbrutt. Det som allerede er lastet ned, blir liggende i hurtigbufferet; en ny kjøring fortsetter derfra.
page-result-title-done = Kart laget
page-result-title-failed = Kjøringen feilet
page-result-title-cancelled = Kjøringen avbrutt
page-kept-map = Det eksisterende kartet ble latt stå som det er. "Erstatt det eksisterende kartet" lager det på nytt og beholder det gamle som .omap.bak.
page-no-files = Ingen filer ble laget.
page-opened-folder = Mappen er åpen.
page-open-failed = { $message } Mappe: { $dir }
page-why-no-area = Velg et område først.
page-why-checking = Forhåndsvisningen beregnes …
page-why-too-large = Området er for stort.
page-why-outside = Området ligger utenfor de støttede delstatene.
page-why-disk = Ikke nok ledig diskplass (se ovenfor).
page-why-running = En kjøring pågår.
page-err-scale-bar-no-pdf = Målestokklinjalen krever en PDF.
page-err-sources-no-pdf = Kildelinjene krever en PDF.
page-save-failed = Området er ikke lagret; ingen kjøring ble startet.
page-poll-lost = Ingen svar fra tjeneren. freemapper prøver videre.
page-run-unknown = Tjeneren kjenner ikke lenger denne kjøringen.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Lukk

## update (spec 3.4 to 3.6)

update-source-build = Denne freemapper er bygget fra kildekode. Oppdater den med git pull og cargo build --release --locked.
update-current = freemapper { $version } er oppdatert.
update-available = freemapper { $new } er tilgjengelig (du har { $old }), utgitt { $released }.
update-details =
    Utgivelsesnotater: { $notes }
    Nedlasting: { $size } MB
update-decimal-separator = ,
update-ask = Installere nå? [y/N]
update-not-installed = Ingenting ble installert.
update-needs-yes = inndata er ikke en terminal, så freemapper kan ikke spørre før installasjonen; kjør freemapper update --yes for å installere uten å spørre
update-installing = laster ned og kontrollerer freemapper { $version }
update-done = freemapper er oppdatert fra { $old } til { $new }. Utgivelsesnotater: { $notes }
update-mcp-restart = En MCP-tjener som kjører, beholder den gamle versjonen til AI-assistenten starter den på nytt.
update-rolled-back = Gikk tilbake til freemapper { $version }. Versjonen du forlot, er beholdt som { $path }.
update-not-writable = kan ikke skrive til programmappen { $path }, og freemapper ber ikke om administratorrettigheter. Pakk ut den nye utgivelsen der manuelt, eller flytt freemapper til en mappe du kan skrive til.
update-too-old = freemapper { $new } kan ikke installeres av selvoppdateringen i { $old }; installer { $needed } eller nyere manuelt, se { $notes }
update-busy = en annen oppdatering eller tilbakerulling pågår i programmappen (låsefil { $path }); vent til den er ferdig og prøv igjen
update-bad-signature = utgivelsen kunne ikke verifiseres ({ $error }); ingenting ble installert
update-check-bad-signature = den nyeste freemapper-utgivelsen kunne ikke verifiseres ({ $error }); ingenting ble installert
update-no-manifest = den nyeste utgivelsen kan ikke installere seg selv (den har ikke noe oppdateringsmanifest); last den ned fra { $page }
update-no-manifest-version = utgivelse { $version } finnes ikke eller kan ikke installere seg selv (den har ikke noe oppdateringsmanifest); se { $page }
update-no-asset = utgivelsen har ikke noe program for dette systemet ({ $platform }); ingenting ble installert
update-no-previous = det finnes ingen tidligere versjon å gå tilbake til ({ $path } mangler)
update-failed = oppdateringen feilet: { $error }
update-mode-set = Oppdateringsmodus satt til { $mode } i { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } overstyrer denne innstillingen så lenge den er satt.
update-mode-failed = kunne ikke skrive { $path }: { $error }
update-notice = freemapper { $new } er tilgjengelig (du har { $old }). Oppdater med: freemapper update
update-applies-next-start = Den nye versjonen gjelder fra neste oppstart av freemapper.
update-install-failed = freemapper { $new } ble ikke installert: { $error }
update-no-config-dir = ingen konfigurasjonsmappe på dette systemet; sett FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Merknad til personen: ved oppstart fant freemapper-tjeneren dette om adaptermappene. Gi det videre; personen retter opp mappen eller fjerner den og starter tjeneren på nytt.
    { $lines }
update-mcp-note = Merknad til personen: freemapper { $new } er tilgjengelig (dette er { $old }). Spør dem før du kaller install_update.
update-mcp-needs-confirm = ingenting ble installert; spør personen først og kall install_update med confirm: true når de samtykker
update-mcp-note-manual = Merknad til personen: freemapper { $new } er tilgjengelig (dette er { $old }), men install_update kan ikke installere den; installer { $needed } eller nyere manuelt, se { $notes }
update-mcp-note-bad-signature = Merknad til personen: den nyeste freemapper-utgivelsen kunne ikke verifiseres ({ $error }); ingenting ble installert. Si fra til dem, og ikke installer den på noen annen måte.
update-mcp-restart-now = Start freemapper MCP-tjeneren på nytt nå; den kjører fortsatt den gamle versjonen.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = feil
error-cancelled = avbrutt
cli-cancelled-kept = avbrutt; beholdt: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disken er full, så freemapper kunne ikke skrive { $path }. Frigjør plass og prøv igjen

## Errors: project

error-project-already-exists = prosjektet finnes allerede: { $path }
error-project-map-exists = kartet finnes allerede (det kan inneholde arbeidet ditt); bruk --force for å erstatte det: { $path }
error-project-invalid-scale = ugyldig målestokk 1:{ $scale }
error-project-invalid-contour-interval = ugyldig ekvidistanse { $interval } m i freemapper.toml; den må være over 0
error-project-scale-not-allowed = målestokk 1:{ $scale } er ikke tillatt for { $kind }; bruk { $allowed }
error-project-scale-not-allowed-enlarge = målestokk 1:{ $scale } er ikke tillatt for { $kind }; bruk { $allowed }. Tegn i 1:4000, freemapper-print forstørrer til 1:3000 ved utskrift
word-kind-schoolyard = skolegårdskart
word-kind-sprint = sprintkart
word-kind-forest = skogskart
error-project-toml = ugyldig prosjektfil: { $message }
error-project-unknown-adapter = prosjektfilen navngir adapteren "{ $adapter }", som denne freemapper ikke har; en nyere freemapper kan ha den (`freemapper update`), eller den kommer fra adaptermappen din (`freemapper adapters list` viser den) eller, med --adapters-from-project på kommandolinjen, fra mappen `adapters` i prosjektet (bare for et prosjekt du stoler på)
error-project-not-fetched = ingen fliser hentet ennå; kjør `freemapper fetch` først
error-project-ocd = OCAD-eksport: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: bilde og verdensfil passer ikke til kartområdet; kjør `freemapper derive` på nytt
error-project-tiles-outdated = kartområdet trenger fliser { $tiles } som ikke ble hentet; kjør `freemapper fetch`
error-project-buildings-not-fetched = ingen ALKIS-bygninger lagret for dette kartområdet; kjør `freemapper fetch` tilkoblet først
error-project-orthophoto-not-fetched = ikke noe ortofoto lagret for dette kartområdet; kjør `freemapper fetch` tilkoblet først
error-project-terrain-not-fetched = ingen terrengmodell lagret for dette kartområdet; kjør `freemapper fetch` tilkoblet først
error-project-candidates-outdated = de lagrede OSM-kandidatene dekker ikke kartområdet; kjør `freemapper fetch`
error-project-stored = { $path }: { $message }; kjør `freemapper fetch` på nytt
error-project-area-too-large = kartområdet pluss buffer er { $width } × { $height } m, mer enn { $limit } m per side; reduser kartområdet eller buffer_m i freemapper.toml
error-project-area-crosses-country = kartområdet ligger på begge sider av en landegrense; kart på tvers av en grense støttes ikke ennå, så tegn området innenfor ett land
error-project-permits-not-for-region = tillatelsesnotatene er ikke tilgjengelige for denne regionen ennå: de spør tyske tjenester og navngir tyske myndigheter
error-project-outside-coverage = kartområdet ligger utenfor de støttede delstatene ({ $states }); freemapper har så langt bare datakilder for dem

## Errors: disk space

error-space-required = ikke nok diskplass til { $part }. Det trengs { $needed }, men { $path } har bare { $left } ledig utover reserven på { $reserve }. Frigjør plass og kjør `freemapper fetch` på nytt
error-space-skipped = freemapper hopper over { $part } på grunn av manglende diskplass. Det trengs { $needed }, men { $path } har bare { $left } ledig utover reserven på { $reserve }. Frigjør plass og kjør `freemapper fetch` på nytt
word-part-map-data = ALKIS- og OSM-kartdataene
word-part-terrain = terrengmodellen
word-part-orthophoto = ortofotoet
word-part-surface = overflatemodellen for nDSM-malen
word-part-osm-update = oppdateringen av det bufrede OSM-utdraget
word-part-k-five = K5-kartmalen for Berlin
word-part-berlin-layers = Berlins ALKIS-strukturer, trematrikkel, gatekartlegging og punktlag
word-part-state-layers = de fulle ALKIS-strukturene, objektene i landskapsmodellen og trematrikklene
word-part-point-cloud = laserpunktskyene for vegetasjonskandidatene

## Errors: downloads

error-fetch-http = nedlastingen av { $url } feilet: { $message }
error-fetch-http-temporary = nedlastingen av { $url } feilet: { $message }. Tjenesten er sannsynligvis overbelastet eller under vedlikehold; freemapper har spurt opptil { $attempts } ganger. Start kjøringen på nytt om noen minutter; det som allerede er lastet ned, beholdes.
error-fetch-corrupt-archive = nedlastingen av { $url } er ikke et fullstendig flisarkiv: { $message }
error-fetch-corrupt-extract = nedlastingen av { $url } er ikke et fullstendig OSM-utdrag: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] flis { $tiles } dekkes ikke av de publiserte flisene; reduser kartområdet eller buffer_m i freemapper.toml
       *[other] fliser { $tiles } dekkes ikke av de publiserte flisene; reduser kartområdet eller buffer_m i freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] frakoblet og ikke i hurtigbufferet: { $product }-flis { $tiles }; kjør `freemapper fetch` tilkoblet
       *[other] frakoblet og ikke i hurtigbufferet: { $product }-fliser { $tiles }; kjør `freemapper fetch` tilkoblet
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = terrengmodell (DGM1)
word-product-image-dsm = overflatemodell (bDOM)
word-product-saxony-dtm = terrengmodell (GeoSN DGM1)
word-product-saxony-dsm = overflatemodell (GeoSN DOM1)
error-fetch-osm-extract-missing = ikke noe OSM-utdrag i hurtigbufferet; kjør `freemapper fetch` tilkoblet
error-fetch-not-found = { $name } ble ikke funnet på tjeneren
error-fetch-refused = { $url }: tjeneren avslo forespørselen (HTTP { $status })
error-fetch-refused-detail = { $url }: tjeneren avslo forespørselen (HTTP { $status }): { $detail }
error-fetch-too-large = nedlastingen av { $url } er større enn { $limit } MB; stoppet
error-fetch-not-stored = ingen { $label } lagret for dette kartområdet: { $path } mangler, kan ikke leses eller dekker det ikke; kjør `freemapper fetch` tilkoblet først
error-fetch-range-ignored = { $url }: tjeneren ignorerte byteområdet { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper sender ikke forespørsler til denne adressen: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper sender ikke forespørsler til denne adressen: adapteren for { $name } navngir ikke verten { $host }{ $from }. Hvis tjenesten har flyttet dit, tillat verten med `freemapper adapters allow-host { $adapter } { $host }` og meld fra på https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper sender ikke forespørsler til denne adressen: adapteren { $adapter } navngir ikke verten { $host }{ $from }. Hvis tjenesten har flyttet dit, legg verten til i `[net] hosts` i adapterens manifest
error-fetch-host-redirected-from = {" "}({ $url } omdirigerer dit)
error-reason-not-a-host-name = ikke et vertsnavn
error-reason-not-ascii = et navn utenfor ASCII må skrives i xn--form
error-reason-not-a-dns-name = ikke et DNS-navn
error-reason-no-domain = et navn på denne datamaskinen eller uten domene
error-reason-local-name = et navn i det lokale nettverket
error-reason-ip-address = en IP-adresse, ikke et DNS-navn
error-reason-unexpected-character = uventet tegn
error-reason-not-https = ikke https
error-reason-user-info = brukerinfo
error-reason-port = en annen port enn 443
error-reason-host-not-declared = verten er ikke en av de deklarerte vertene
error-reason-local-address = en adresse på denne datamaskinen eller i det lokale nettverket, ikke på en tjener på internett
error-reason-too-many-redirects = mer enn { $count } omdirigeringer
error-reason-redirect-nowhere = en omdirigering (HTTP { $status }) til ingensteds
error-fetch-currency-csv = valuta-CSV linje { $line }: { $message }
error-alkis = ugyldige ALKIS-data: { $message }
error-osm = OSM-data: { $message }

## Errors: templates and rasters

error-derive-zip = flisarkiv: { $message }
error-derive-image = bilde: { $message }
error-derive-no-tiles = ingen fliser å beregne fra
error-derive-area-too-large = kartområdet er for stort for én mal: { $width } × { $height } piksler (høyst { $limit } per side); reduser kartområdet eller buffer_m i freemapper.toml
error-raster-world-file = ugyldig verdensfil: { $message }
error-raster-misaligned = rasterne er ikke justert til samme pikselrutenett: { $message }
error-raster-pixel-size = rasterne har ulik pikselstørrelse: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = ugyldig GeoJSON: { $message }
error-area-no-polygon = GeoJSON inneholder ingen polygon
error-area-multi-polygon = MultiPolygon med { $parts } deler; del kartområdet opp i ett polygon
error-area-out-of-range = koordinat utenfor gyldig område: lat { $lat }, lon { $lon }
error-area-not-an-area = ikke et brukbart kartområde: { $message }
error-georef-date = datoen { $date } ligger utenfor gyldighetsperioden til World Magnetic Model
error-georef-map-range = punktet E { $easting } N { $northing } ligger utenfor kartets koordinatområde
error-omap-xml = ugyldig OOM-XML: { $message }
error-omap-missing-element = elementet <{ $name }> ble ikke funnet
error-ocd-duplicate-symbol = symbolnummer { $symbol } er brukt to ganger
error-ocd-unknown-symbol = et objekt viser til symbolet { $symbol }, som ikke finnes i filen
error-ocd-wrong-type = et objekt med symbolet { $symbol } har feil type for symbolet sitt

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Bruk:
cli-heading-commands = Kommandoer:
cli-heading-options = Valg:
cli-heading-arguments = Argumenter:
cli-help-possible-values = mulige verdier
cli-help-default = standard
cli-arg-help = Vis hjelp
cli-arg-version = Vis versjon
cli-help-help = Vis hjelpen til en kommando
cli-arg-help-command = Kommandoen du vil se hjelpen til, for eksempel run
cli-parse-hint = Prøv '--help' for mer informasjon.
cli-double-click-start = freemapper åpner siden der du velger kartområdet, i nettleseren. Prosjektmappen er { $path }. Lukk dette vinduet for å avslutte programmet.
cli-parse-unknown-argument = uventet argument { $arg }
cli-parse-missing-required = mangler påkrevd argument: { $args }
cli-parse-invalid-value = ugyldig verdi { $value } for { $arg }
cli-parse-invalid-value-choices = ugyldig verdi { $value } for { $arg }; mulige verdier: { $values }
cli-parse-invalid-subcommand = ukjent kommando { $value }
cli-parse-missing-subcommand = en kommando er påkrevd
cli-parse-too-many-values = uventet verdi { $value } for { $arg }
cli-parse-argument-conflict = { $arg } kan ikke brukes sammen med { $other }

help-freemapper = Bygger OOM-startprosjekter fra geodata
help-freemapper-lang = Språk for programtekstene, `de` eller `en` (standard: FREEMAPPER_LANG, deretter systemspråket, deretter engelsk)
help-init = Opprett en prosjektmappe for et kartområde
help-init-dir = Prosjektmappe som skal opprettes
help-init-geojson = GeoJSON-fil med kartområdets polygon (EPSG:4326)
help-init-osm = OSM-objektet for kartområdet, f.eks. way/992423119
help-init-place = Navnet på stedet, f.eks. "Clara-Zetkin-Schule Strausberg"; freemapper bruker det beste treffet som har et område
help-init-part = Hvilken del av et OSM-område med flere atskilte deler som skal brukes (1 er den største)
help-init-kind = Karttype
help-arg-scale = Målestokkens nevner for et nytt prosjekt: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andre verdier avvises, fordi freemapper-print avviser dem
help-init-title = Karttittel for randinformasjonen (standard: mappenavnet)
help-export = Skriv startprosjektet (OOM-kart, maler, kildeangivelse) til en prosjektmappe
help-export-dir = Prosjektmappe
help-arg-force = Erstatt en eksisterende kartfil og behold den gamle som `<name>.omap.bak`
help-export-ocad = Skriv også et OCAD-startpunkt til `ocad/` (OCAD 12-format); et eksisterende OOM-kart blir da stående med mindre du bruker --force
help-export-pdf = Skriv også `<name>.pdf`, kartet til utskrift (bare kartobjektene, ingen kandidater); et eksisterende OOM-kart blir da stående med mindre du bruker --force
help-export-pdf-candidates = Skriv også `<name>-candidates.pdf`, kontrollutskriften: kartet med alle kandidater, posisjonstro; et eksisterende OOM-kart blir da stående med mindre du bruker --force
help-export-training = Skriv også treningskartet `<name>-training.omap` og dets PDF `<name>-training.pdf`: kartet og alle kandidater, ryddet slik at det kan skrives ut som det er; med --ocad også `ocad/<name>-training.ocd`. Et eksisterende OOM-kart blir da stående med mindre du bruker --force, og --force erstatter også et eksisterende treningskart
help-export-view = Med --training: skriv også treningskartet uten det en visning utelater, som `<name>-training-<view>.omap` og dets PDF (med --ocad også en OCAD-fil). Visninger: `no-paths` (ingen veier, stier, trapper), `brown-only` (bare brune symboler og rammen), `no-north-lines`, eller en liste `no:<codes>` / `only:<codes>`; kombiner med `+`. Gjenta valget for flere filer
help-export-pdf-scale-bar = Tegn målestokklinjalen på PDF-ene (av som standard, tyske kart har vanligvis ingen; målestokktallet blir stående i tittelfeltet)
help-export-pdf-sources = Kildelinjer på PDF-ene: `auto` skriver ut hele listen og forkorter den bare når den ville tatt mer enn 5 linjer, `full` forkorter aldri, `credit-only` skriver bare ut freemapper-krediteringen (ODbL og dl-de/by-2-0 krever da at du oppgir kildene når du gir kartet videre). ATTRIBUTION.txt lister alltid alle kilder
help-fetch = Last ned dataene for kartområdet (ortofoto, terrengmodell, nDSM, ALKIS-bygninger, OSM)
help-fetch-dir = Prosjektmappe
help-fetch-offline = Bruk bare hurtigbufferet, uten nettverkstilgang
help-arg-cache-dir = Hurtigbuffermappe for nedlastede fliser (standard: systemets hurtigbuffermappe + `freemapper`)
help-derive = Beregn malene (ortofoto, skyggerelieff, helning, lokalt relieff, nDSM), høydekurver og landformer fra de hentede dataene
help-derive-dir = Prosjektmappe
help-run = Hent, beregn og eksporter i ett
help-run-dir = Prosjektmappe
help-run-offline = Bruk bare hurtigbufferet, uten nettverkstilgang
help-run-ocad = Skriv også et OCAD-startpunkt til `ocad/` (OCAD 12-format)
help-run-pdf = Skriv også `<name>.pdf`, kartet til utskrift (bare kartobjektene, ingen kandidater)
help-run-pdf-candidates = Skriv også `<name>-candidates.pdf`, kontrollutskriften: kartet med alle kandidater, posisjonstro
help-run-training = Skriv også treningskartet `<name>-training.omap` og dets PDF `<name>-training.pdf`: kartet og alle kandidater, ryddet slik at det kan skrives ut som det er; med --ocad også `ocad/<name>-training.ocd`. Et eksisterende treningskart blir stående med mindre du bruker --force
help-run-view = Med --training: skriv også treningskartet uten det en visning utelater, som `<name>-training-<view>.omap` og dets PDF (med --ocad også en OCAD-fil). Visninger: `no-paths` (ingen veier, stier, trapper), `brown-only` (bare brune symboler og rammen), `no-north-lines`, eller en liste `no:<codes>` / `only:<codes>`; kombiner med `+`. En eksisterende visningsfil blir stående med mindre du bruker --force
help-run-pdf-scale-bar = Tegn målestokklinjalen på PDF-ene (av som standard; målestokktallet blir stående i tittelfeltet)
help-run-pdf-sources = Kildelinjer på PDF-ene: `auto` skriver ut hele listen og forkorter den bare når den ville tatt mer enn 5 linjer, `full` forkorter aldri, `credit-only` skriver bare ut freemapper-krediteringen (ODbL og dl-de/by-2-0 krever da at du oppgir kildene når du gir kartet videre). ATTRIBUTION.txt lister alltid alle kilder
help-select = Velg kartområdet i nettleseren ved å tegne det eller velge et sted. Lagring skriver freemapper.toml og oppretter prosjektmappen om nødvendig
help-select-dir = Prosjektmappe (opprettes ved lagring hvis den ikke finnes)
help-select-kind = Karttype for et nytt prosjekt
help-select-title = Karttittel for et nytt prosjekt
help-select-no-browser = Skriv bare ut adressen til siden; ikke åpne nettleseren
help-select-port = Port på 127.0.0.1 (standard: en ledig)
help-select-offline = Siden starter med "Bare hurtigbuffer, ingen nettverk" avmerket
help-select-ocad = Siden starter med OCAD-startprosjektet avmerket
help-select-pdf = Siden starter med PDF til utskrift avmerket
help-select-pdf-candidates = Siden starter med kontrollutskriften (alle kandidater) avmerket
help-select-training = Siden starter med treningskartet avmerket
help-select-pdf-scale-bar = Siden starter med målestokklinjalen på PDF-ene avmerket
help-select-pdf-sources = Siden starter med dette valget av kildelinjer på PDF-ene, som for `run`
help-find = List OpenStreetMap-steder for et navn, områder først, med ID-en for init --osm
help-find-query = Navnet på stedet, f.eks. "Clara-Zetkin-Schule Strausberg"
help-find-json = Skriv ut treffene som JSON, med de samme feltene som MCP-verktøyet find_place
help-mcp = Tilby verktøyene til en AI-agent via MCP (stdin/stdout)
help-update = Installer en nyere freemapper fra GitHub-utgivelsene, etter å ha spurt. En bygging fra kildekode sier i stedet hvordan den oppdateres
help-update-check = Bare sjekk: avslutningskode 10 når det finnes en nyere versjon, 0 når denne er oppdatert
help-update-yes = Installer uten å spørre; nødvendig når inndata ikke er en terminal
help-update-version = Installer denne versjonen (x.y.z) i stedet for den nyeste, også en eldre
help-update-rollback = Gå tilbake til versjonen den siste oppdateringen erstattet
help-update-mode = Lagre hva den daglige sjekken gjør når den finner en nyere versjon (standard: notify). Fungerer i alle bygg, fordi den bare skriver innstillingen
help-adapters = Vis regionsadapterne: delstatene og landene freemapper har datakilder for
help-adapters-list = List adapterne med hvor hver kommer fra (innebygd, mappen din, et prosjekt), revisjonen, dagen den sist ble sjekket og vertene den spør
help-adapters-list-json = Skriv ut listen som JSON
help-adapters-list-project = Les også adapterne i mappen `adapters` i denne prosjektmappen, som --adapters-from-project ville gjort
help-adapters-check = Sjekk mappen til en adapter før den brukes eller gis videre: om den ville lastes, hva forfatteren bør vite, og med --live om tjenestene svarer. Avslutningskode 0: den består; 1: den har funn; 2: sjekken kunne ikke kjøres
help-adapters-check-dir = Mappen til adapteren; navnet er adapterens ID
help-adapters-check-live = Spør også adapterens tjenester om boksen --bbox navngir, og si hva hver leverer. Forespørslene holder seg til vertene manifestet deklarerer
help-adapters-check-bbox = Boksen til live-sjekken i grader, som minLon,minLat,maxLon,maxLat. Den ligger innenfor adapterens grense; 150 m ganger 150 m er nok
help-adapters-check-json = Skriv ut resultatet som JSON
help-adapters-probe = Spør hver tjeneste de innebygde adapterne nevner én gang, og si om den svarer: ok, slow, moved eller error. Avslutningskoden er 0 også når en tjeneste feiler; resultatet er svaret
help-adapters-probe-json = Skriv ut resultatet som JSON
help-adapters-probe-status = Les statusen fra forrige natt fra denne filen (den kan mangle) og skriv ut ny status som JSON: En tjeneste er nede etter to dårlige netter på rad. Endringene går til standard feil
help-adapters-allow-host = Tillat en innebygd adapter én vert til: for en delstat hvis tjeneste har flyttet til en tjener adapteren ennå ikke navngir. Meldingen om den avviste adressen sier hvilken vert. Verten kontrolleres som en vert i et manifest, og freemapper kobler fortsatt bare til tjenere på internett
help-adapters-allow-host-adapter = ID-en til den innebygde adapteren, slik `adapters list` viser den
help-adapters-allow-host-host = Verten som skal tillates: et DNS-navn, ingen adresse, ingen port
help-adapters-disallow-host = Ta tilbake en vert som allow-host tillot
help-adapters-disallow-host-adapter = ID-en til den innebygde adapteren
help-adapters-disallow-host-host = Verten som skal tas tilbake
help-arg-adapters-from-project = Last også adapterne i mappen `adapters` i prosjektmappen. freemapper navngir hver av dem sammen med vertene den spør. Bruk det for et prosjekt du stoler på: en adapter får freemapper til å spørre tjenerne den navngir

## The summary of a run, for the MCP server and the area page

cli-internal-error = intern feil: { $message }
cli-pdf-sources-invalid = pdf_sources må være auto, full eller credit-only, ikke { $value }
cli-view-unknown = en visning er no-paths, brown-only, no-north-lines, no:<codes> eller only:<codes>, ikke { $value }
cli-view-empty = listen { $value } navngir ingen symbolkode
cli-view-bad-code = { $value } er ingen symbolkode som 505 eller 501.6
cli-summary-training-view = Skrev { $map } og { $pdf } (en visning av treningskartet)
cli-summary-kept-training-view = Beholdt filene til visningen { $view } av treningskartet; bruk --force for å lage dem på nytt
cli-summary-training-view-failed = Visningen av treningskartet feilet: { $error }
cli-estimate-disk-unreadable = kan ikke lese ledig diskplass for { $path } ({ $message })
cli-summary-wrote-map = Skrev kartet { $path }
cli-summary-kept-map = Beholdt det eksisterende kartet { $path } (det kan inneholde kartleggerens arbeid)
cli-summary-map-lacks-templates = Kartet refererer ennå ikke til bildemalene. Start pipelinen med force for å erstatte det og beholde det gamle som .omap.bak.
cli-summary-map-lacks-buildings = Kartet har ingen ALKIS-bygninger ennå. Start pipelinen med force for å legge dem til; den erstatter kartet og beholder det gamle som .omap.bak.
cli-summary-map-lacks-canopies = Kartet mangler ALKIS-takene og carportene (522) som nå er kartobjekter. Start pipelinen med force for å legge dem til; den erstatter kartet og beholder det gamle som .omap.bak.
cli-summary-map-light = Grunnkartet: { $joined } linjer slått sammen, { $left } skjulte eller for små igjen for kandidatlagene (ingenting flyttet)
cli-summary-map-light-kept = Kandidatlagene inneholder nå også { $left } objekter eller strekninger som denne versjonen tar av grunnkartet (merket freemapper:generalized). Det beholdte kartet har dem fortsatt, så de vises to ganger. Start pipelinen med force for å rydde kartet; den beholder det gamle som .omap.bak.
cli-summary-map-old-candidates = Kartet navngir fortsatt templates/candidates.omap; denne kjøringen fylte den med alle kandidater. Kandidatene kommer nå også i sju lag, én fil hver (templates/candidates-*.omap). Kartet lister dem etter en kjøring med force (det gamle kartet blir liggende som .omap.bak), eller kartleggeren legger dem til i OOM (Templates > Open template…).
cli-summary-wrote-ocad = Skrev OCAD-kartet { $path } ({ $count } objekter)
cli-summary-kept-ocad = Beholdt det eksisterende OCAD-kartet { $path } (det kan inneholde kartleggerens arbeid; force erstatter det)
cli-summary-ocad-old-candidates = OCAD-kartet navngir fortsatt { $path } som bakgrunnskart; denne kjøringen fylte den med alle kandidater. Kandidatene kommer nå også i sju lag, én fil hver (-candidates-*.ocd). OCAD-kartet lister dem etter en kjøring med force (det gamle kartet blir liggende som .ocd.bak), eller kartleggeren legger dem til i OCAD som bakgrunnskart.
cli-summary-ocad-left-out = Utelatt fra OCAD-kartet: { $skipped } objekter og { $dropped } degenererte deler
cli-summary-ocad-symbols = I OCAD velger du Symbol > Load Colors and Symbols From… og peker på det offisielle symbolsettet{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOFs skolesett 2019
       *[other] {" "}ISSprOM 2019
    }; det erstatter plassholdersymbolene etter nummer.
cli-summary-ocad-failed = OCAD-eksporten feilet: { $error }
cli-summary-pdf-without-credits = PDF uten kildehenvisninger: ODbL og dl-de/by-2-0 krever at du oppgir kildene når du gir kartet videre; se ATTRIBUTION.txt
cli-summary-wrote-pdf = Skrev kartet { $path } (uten kandidater)
cli-summary-pdf-failed = PDF-eksporten feilet: { $error }
cli-summary-wrote-pdf-candidates = Skrev kontrollutskriften { $path } (kartet med alle kandidater, posisjonstro)
cli-summary-pdf-candidates-failed = Eksporten av kontrollutskriften feilet: { $error }
cli-summary-training-failed = Eksporten av treningskartet feilet: { $error }
cli-summary-kept-training = Beholdt det eksisterende treningskartet { $path }, slik at en kjøring ikke overskriver et kart som er skrevet ut eller gitt videre; slett det eller sett force for å lage det på nytt

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = ingen hjemmemappe på dette systemet; bruk en absolutt sti
mcp-error-relative-path = { $path } er en relativ sti; bruk en absolutt sti eller en som starter med ~
mcp-error-home-folder = { $path } er selve hjemmemappen; bruk en mappe inni den, f.eks. ~/maps/school
mcp-error-no-project = ikke noe freemapper-prosjekt i { $path }; bruk create_project først
mcp-error-unknown-kind = ukjent kind { $kind }; bruk "schoolyard", "sprint" eller "forest"
mcp-error-reading = leser { $path }: { $error }
mcp-error-one-area-source = oppgi nøyaktig én av osm_id (fra find_place) eller geojson
mcp-error-page-open = områdevalgsiden for dette prosjektet er fortsatt åpen; be brukeren lagre området og trykke Ferdig på siden, og start så pipelinen
mcp-error-no-job = ingen jobb { $job }; start_pipeline og permits_for_area returnerer jobb-ID-ene i denne økten
mcp-error-job-running = { $job } kjører fortsatt for { $path }; én pipeline kjører om gangen fordi prosjektene deler nedlastingsbufferet. Start denne når project_status melder finished eller failed
mcp-error-page-start = kan ikke starte siden: { $error }
mcp-created-project = Opprettet prosjektet { $path } for { $area } ({ $kind }, 1:{ $scale }). Neste: estimate_download, deretter start_pipeline.
mcp-created-project-given-area = Opprettet prosjektet { $path } for det oppgitte området ({ $kind }, 1:{ $scale }). Neste: estimate_download, deretter start_pipeline.
mcp-page-already-open = Siden for denne mappen er allerede åpen; den beholder innstillingene den ble åpnet med.
mcp-page-next = Be brukeren velge området på siden, trykke Lagre og deretter Ferdig.
mcp-preview-of = Forhåndsvisning av { $layer }
mcp-permits-next = Tillatelsesnotatene lages. Spør project_status med denne job_id; når jobben er ferdig, inneholder permits_md teksten i permits.md. Gi den videre som et hint, ikke juridisk rådgivning.
mcp-tool-check-update = Om det finnes en nyere freemapper-utgivelse: versjon, utgivelsesdato, utgivelsesnotater og nedlastingsstørrelse, eller at denne er oppdatert. Installerer ingenting.
mcp-tool-create-project = Opprett en prosjektmappe for et kartområde. Oppgi osm_id (fra find_place) eller en GeoJSON-fil. kind er schoolyard (1:1000, standard), sprint (1:4000) eller forest (1:15000).
mcp-param-create-project-dir = Ny prosjektmappe (absolutt sti eller starter med ~).
mcp-param-create-project-geojson = I stedet for osm_id: en GeoJSON-fil med kartområdets polygon (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-create-project-osm-id = OSM-objektet for kartområdet fra find_place, f.eks. "way/992423119".
mcp-param-create-project-part = Med osm_id: hvilken del av et område med flere atskilte deler som skal brukes (1 er den største).
mcp-param-create-project-scale = Målestokkens nevner: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andre verdier avvises; for 1:3000 tegn i 4000, freemapper-print forstørrer ved utskrift.
mcp-param-create-project-title = Karttittel for randinformasjonen (standard: mappenavnet).
mcp-tool-estimate-download = Hva start_pipeline vil laste ned for et prosjekt: flisene, hver nedlasting med størrelse i MB, om den ligger i hurtigbufferet og om den får plass på disken, MB som gjenstår å laste ned, og under disk den ledige plassen på hurtigbuffer- og prosjektvolumene med meldinger om det som ikke får plass (si det til brukeren).
mcp-param-estimate-download-dir = Prosjektmappe (absolutt sti eller starter med ~).
mcp-tool-find-place = Søk etter en skole, en park eller et annet sted ved navn (OpenStreetMap Nominatim). Returnerer opptil 5 treff, områder først; bare treff med has_area kan bli kartområde. En tom liste betyr at ingenting ble funnet.
mcp-param-find-place-query = Navnet på skolen eller stedet, f.eks. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installer den nyeste freemapper-utgivelsen over dette programmet. Kall det først etter at personen har samtykket, med confirm true. Den nye versjonen gjelder når assistenten starter freemapper-tjeneren på nytt.
mcp-param-install-update-confirm = true når personen har samtykket i å installere den nyere versjonen; uten det installeres ingenting.
mcp-tool-list-layers = Lagene i et prosjekt: kartet, de sju kandidatlagene og rastermalene, hver med fil, om det er laget ennå og hvilket trinn som lager det.
mcp-param-list-layers-dir = Prosjektmappe (absolutt sti eller starter med ~).
mcp-tool-open-area-selector = Åpne en side i nettleseren der brukeren tegner kartområdet (rektangel eller polygon) eller velger et sted, ser flisene og nedlastingsstørrelsen, og lagrer det; lagring oppretter prosjektmappen eller endrer området. Returnerer URL-en som skal gis til brukeren. Siden kjører til brukeren trykker Ferdig; et nytt kall for samme mappe returnerer samme URL.
mcp-param-open-area-selector-dir = Prosjektmappe (absolutt sti eller starter med ~); opprettes når brukeren lagrer.
mcp-param-open-area-selector-kind = Karttype for et nytt prosjekt: "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-open-area-selector-ocad = Kjørepanelet starter med OCAD-startprosjektet avmerket. Standard false.
mcp-param-open-area-selector-offline = Kjørepanelet starter med "Bare hurtigbuffer, ingen nettverk" avmerket. Standard false.
mcp-param-open-area-selector-open-browser = Åpne også siden i standardnettleseren på denne datamaskinen. Standard true.
mcp-param-open-area-selector-pdf = Kjørepanelet starter med PDF til utskrift avmerket. Standard false.
mcp-param-open-area-selector-pdf-candidates = Kjørepanelet starter med kontrollutskriften (alle kandidater) avmerket. Standard false.
mcp-param-open-area-selector-pdf-scale-bar = Kjørepanelet starter med målestokklinjalen på PDF-ene avmerket. Standard false.
mcp-param-open-area-selector-pdf-sources = Kildelinjer på PDF-ene som panelet starter med: "auto" (standard), "full" eller "credit-only", som for start_pipeline.
mcp-param-open-area-selector-permits = Kjørepanelet starter med tillatelsesnotatene avmerket. Standard false.
mcp-param-open-area-selector-scale = Målestokkens nevner for et nytt prosjekt: skolegård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andre verdier avvises.
mcp-param-open-area-selector-title = Karttittel for et nytt prosjekt.
mcp-param-open-area-selector-training = Kjørepanelet starter med treningskartet avmerket. Standard false.
mcp-tool-preview-layer = En PNG-forhåndsvisning (høyst 800 px) av et lag: map (bildet av kartobjektene den siste kjøringen skrev, uten kandidatene), eller en rastermal: orthophoto, hillshade, slope, relief, ndsm, eller k5 i Berlin.
mcp-param-preview-layer-dir = Prosjektmappe (absolutt sti eller starter med ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" eller "k5" (bare Berlin).
mcp-tool-project-status = Tilstanden til en jobb fra start_pipeline eller permits_for_area: running, finished eller failed, de siste fremdriftslinjene, advarsler, filene som er skrevet, og resultatet eller feilen. En ferdig permits_for_area-jobb har også teksten i permits.md som permits_md.
mcp-param-project-status-job-id = Jobb-ID fra start_pipeline eller permits_for_area, f.eks. "job-1".
mcp-tool-permits-for-area = Lag tillatelsesnotatene for et prosjekt: de administrative enhetene i kartområdet, verneområdene i det og hvem som driver landet der, med myndighetene du bør spørre; et hint, ikke juridisk rådgivning. Skriver permits.md og permits.json til mappen. Kjører i bakgrunnen og returnerer en job_id; spør project_status, hvis svar har teksten i permits.md som permits_md når jobben er ferdig. Én jobb kjører om gangen.
mcp-param-permits-for-area-dir = Prosjektmappe (absolutt sti eller starter med ~).
mcp-param-permits-for-area-offline = Les bare dataene som en tidligere kjøring har lagret i prosjektet, uten nettverkstilgang. Standard false.
mcp-tool-start-pipeline = Last ned dataene, beregn malene og skriv startprosjektet (fetch, derive, export). Kjører i bakgrunnen og returnerer en job_id; spør project_status. Én pipeline kjører om gangen. Den beholder et eksisterende kart med mindre force er true.
mcp-param-start-pipeline-dir = Prosjektmappe (absolutt sti eller starter med ~).
mcp-param-start-pipeline-force = Erstatt en eksisterende kartfil og behold den gamle som .omap.bak. Standard false.
mcp-param-start-pipeline-ocad = Skriv også et OCAD-startpunkt til mappen ocad/ (OCAD 12-format, for brukere som redigerer i OCAD). Standard false.
mcp-param-start-pipeline-offline = Bruk bare nedlastingsbufferet, uten nettverkstilgang. Standard false.
mcp-param-start-pipeline-pdf = Skriv også <name>.pdf, kartet klart til utskrift (bare kartobjektene, ingen kandidater). Standard false.
mcp-param-start-pipeline-pdf-candidates = Skriv også <name>-candidates.pdf, kontrollutskriften: kartet med alle kandidater, posisjonstro (sjekk dem i terrenget). Standard false.
mcp-param-start-pipeline-pdf-scale-bar = Tegn målestokklinjalen på PDF-ene. Standard false (tyske kart har vanligvis ingen; målestokktallet blir stående i tittelfeltet).
mcp-param-start-pipeline-pdf-sources = Kildelinjer på PDF-ene: "auto" (standard; hele listen, forkortet bare når den tar mer enn 5 linjer), "full" (aldri forkortet) eller "credit-only" (bare freemapper-krediteringen; ODbL og dl-de/by-2-0 krever da at kildene oppgis når kartet gis videre). ATTRIBUTION.txt lister alltid alle kilder.
mcp-param-start-pipeline-permits = Skriv også tillatelsesnotatene (permits.md og permits.json) etter kartet: administrative enheter, verneområder og hvem som driver landet. En feil der er en advarsel, ikke en mislykket jobb. Standard false.
mcp-param-start-pipeline-training = Skriv også treningskartet <name>-training.omap og dets PDF <name>-training.pdf: kartet og alle kandidater, ryddet slik at det kan skrives ut som det er; med ocad også ocad/<name>-training.ocd. Et eksisterende treningskart beholdes med mindre force er true. Standard false.
mcp-param-start-pipeline-training-views = Med training: skriv også treningskartet uten det en visning utelater, ett filsett per oppføring: "no-paths", "brown-only", "no-north-lines", eller en liste som "no:505,506" eller "only:101,102", kombinert med +. Filene er <name>-training-<view>.omap og dets PDF.
mcp-instructions = freemapper lager OpenOrienteering Mapper-startprosjekter for orienteringskart (skolegård 1:1000, sprint 1:4000, skog 1:15000) fra offisielle geodata fra { $states }, og fra OSM. Arbeidsgang: find_place (søk etter skolen eller området ved navn; velg et treff med has_area) → create_project (en ny mappe, absolutt sti, med osm_id) → estimate_download (si til brukeren hvor mye som lastes ned) → start_pipeline (returnerer en jobb-ID; nedlasting kan ta minutter) → project_status (spør hvert par sekunder til tilstanden er finished eller failed) → list_layers og preview_layer (vis brukeren malene). Brukeren åpner deretter kartfilen i OpenOrienteering Mapper. Når ingen steder passer eller brukeren vil tegne området, åpner open_area_selector en side i nettleseren der brukeren tegner eller velger området og lagrer det (dette oppretter prosjektet); fortsett med estimate_download når brukeren sier at det er lagret. Stier må være absolutte eller starte med ~. Én pipeline kjører om gangen. permits_for_area lager tillatelsesnotatene for et prosjekt (administrative enheter, verneområder, hvem som driver landet; et hint, ikke juridisk rådgivning) som en jobb som start_pipeline. check_update forteller om en nyere freemapper finnes; install_update installerer den, bare med confirm: true etter at personen har samtykket, og assistenten starter deretter freemapper-tjeneren på nytt.

cli-contour-interval-saved = Ekvidistanse lagret i freemapper.toml: { $interval } m; derive trekker høydekurvene med den
cli-contour-interval-auto = Ekvidistanse i freemapper.toml: automatisk (karttypens; et skogskart får 2,5 m i flatt land og 10 m i bratt land)
help-arg-contour-interval = Ekvidistanse i meter, f.eks. `2.5`, eller `auto` for karttypens (et skogskart: 5 m, 2,5 m der det er flatt, 10 m der det er bratt). Lagres i freemapper.toml; derive trekker høydekurvene med den
cli-content-saved = Innholdsinnstilling lagret i freemapper.toml: { $summary }
cli-content-invalid = Kan ikke lese innholdsinnstillingen: { $detail }
help-arg-content = Hva som havner direkte på kartet: `safe` lar hvert objekt bli der byggingen plasserer det, `balanced` (standard) legger objektene fra matrikkelen og registrene på kartet, `full` legger alt på kartet. Valget lagres i freemapper.toml
help-arg-content-set = Send en gruppe objekter til `map`, `candidates` eller `off`, som `GROUP=PLACE`, f.eks. `walls=map`; kan gjentas. Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Lagres i freemapper.toml
mcp-param-create-project-content = Hva som havner direkte på kartet: "safe", "balanced" (standard) eller "full". Lagres i prosjektet. balanced legger objektene fra matrikkelen og registrene på kartet, full legger alt på det.
mcp-param-create-project-content-set = Grupper som sendes et annet sted enn profilen sier, som "group=place" med place map, candidates eller off, f.eks. "walls=map". Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Ekvidistanse i meter, f.eks. 2.5; 0 går tilbake til det automatiske valget (karttypens; et skogskart får 2,5 m i flatt land og 10 m i bratt land). Lagres i prosjektet; neste derive trekker høydekurvene med den.
mcp-param-start-pipeline-content = Endre hva som havner direkte på kartet før kjøringen: "safe", "balanced" eller "full". Lagres i prosjektet. Standard: prosjektets innstilling, ellers balanced.
mcp-param-start-pipeline-content-set = Grupper som sendes et annet sted enn profilen sier, som "group=place" med place map, candidates eller off, f.eks. "fences=candidates". Lagres i prosjektet.
mcp-param-start-pipeline-contour-interval = Ekvidistanse i meter, f.eks. 2.5; 0 går tilbake til det automatiske valget (karttypens; et skogskart får 2,5 m i flatt land og 10 m i bratt land). Lagres i prosjektet; neste derive trekker høydekurvene med den.
mcp-error-content = Kan ikke lese innholdsinnstillingen: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Skriv tillatelsesnotatene for et prosjekt (permits.md og permits.json): de administrative enhetene, verneområdene og driverne kartområdet berører, og hvem det vanligvis betyr at du bør spørre. Den spør BfN og BKG og, der delstaten til kartområdet har egne åpne tjenester, dem gjennom vertene til adapteren (liste i håndboken, kapittel 12); i Nordrhein-Westfalen laster den ned ca. 32 MB til dette første gang, i Brandenburg ca. 4 MB, senere bare nyere filer. Den leser OSM-utdraget som allerede ligger i hurtigbufferet; --offline bruker det som ble hentet før
help-permits-dir = Prosjektmappe
help-permits-offline = Bruk bare dataene som ble hentet før, uten nettverkstilgang
help-run-permits = Skriv også tillatelsesnotatene (permits.md og permits.json) etter kartet; spør BfN, BKG og delstatens åpne tjenester (håndboken, kapittel 12). En feil der er en advarsel, og kartet blir stående
cli-wrote-permits = Skrev tillatelsesnotatene { $path }
cli-summary-wrote-permits = Skrev tillatelsesnotatene { $path }
cli-permits-failed = tillatelsesnotatene kunne ikke skrives: { $error }
cli-permits-source-left-out = tillatelsesnotatene utelater { $source }: { $reason }
cli-permits-not-read = den ble ikke lest
error-permits-stored = { $path }: { $message }; hent tillatelsesnotatene på nytt tilkoblet
error-permits-nothing-read = ingen kilde til tillatelsesnotatene kunne leses ({ $missing }); hent dem tilkoblet først
error-permits-missing-not-read = { $source }: ikke lest

permits-title = Tillatelsesnotater
permits-disclaimer = Et hint, ikke juridisk rådgivning. De kompetente myndighetene avgjør. For dataenes dato og nøyaktighet, se tabellen under "Kilder og datadato"; områder kan mangle eller være utdaterte (BfN: slått sammen én gang i året, "Nicht für Planungszwecke geeignet.", altså ikke egnet for planlegging).
permits-disclaimer-regional = Et hint, ikke juridisk rådgivning. De kompetente myndighetene avgjør. For dataenes dato og nøyaktighet, se tabellen under "Kilder og datadato"; områder kan mangle eller være utdaterte.
permits-bfn-merge = BfN slår sammen delstatenes data én gang i året og merker dem som ikke egnet for planlegging. Delstatens myndigheter avgjør.
permits-created = Laget { $date } for kartområdet til et { $kind ->
        [schoolyard] skolegårdskart
        [forest] skogskart
       *[other] sprintkart
    } ({ $area } ha).
permits-section-admin = Administrative enheter
permits-section-protected = Verneområder (BfN)
permits-section-osm-protected = Flere verneområder fra OpenStreetMap
permits-section-operators = Drivere og eiere ifølge OpenStreetMap
permits-section-hints = Hint
permits-section-sources = Kilder og datadato
permits-none-admin = VG250 navngir ingen administrativ enhet for kartområdet.
permits-none-protected = Ifølge BfN ligger det ikke noe verneområde i kartområdet.
permits-none-osm-protected = OpenStreetMap navngir ikke noe verneområde i kartområdet som BfN ikke allerede lister.
permits-none-operators = OpenStreetMap navngir ingen driver og ingen eier for kartområdet.
permits-none-hints = Ingen hint.
permits-section-not-read = Dataene fra { $source } ble ikke lest, så denne delen er tom, selv om det kan finnes oppføringer.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Administrative områder 1:250 000 (VG250) fra BKG
        [bfn] Verneområder i Tyskland fra BfN
       *[osm] OpenStreetMap, fra utdraget freemapper holder i hurtigbufferet sitt
    }
permits-level = { $level ->
        [land] Delstat
        [kreis] Distrikt (Kreis)
        [verwaltungsgemeinschaft] Amt eller Verwaltungsgemeinschaft
       *[gemeinde] Kommune (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Berørt i kanten (mindre enn 0,5 % av kartområdet):
permits-at-edge-generalised = VG250 er generalisert til målestokk 1:250 000. En enhet som bare berører kanten, kan være en følge av det.
permits-kind = { $kind ->
        [nationalpark] Nasjonalpark
        [nationales-naturmonument] Nasjonalt naturmonument
        [naturschutzgebiet] Naturreservat (Naturschutzgebiet)
        [ffh] FFH-område (Natura 2000)
        [vogelschutzgebiet] Fuglevernområde (Natura 2000)
        [biosphaerenreservat] Biosfærereservat
        [biosphaerenreservat-zone] Sone i et biosfærereservat
        [landschaftsschutzgebiet] Landskapsvernområde (Landschaftsschutzgebiet)
       *[naturpark] Naturpark
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Vernet { $date }
permits-protected-title = Rettsgrunnlag: { $title }
permits-protected-link = Lovtekst: { $url }
permits-protected-site-link = Områdeside (Natura 2000): { $url }
permits-protected-year = Dataår hos BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = uten navn
permits-osm-operator = Driver ifølge OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap-områder som omslutter hele kartområdet uten et hjørne inni det, kan mangle her, og det samme gjelder verneområder kartlagt i OSM som en relasjon med `type=boundary`. Delene fra BfN og VG250 berøres ikke.
permits-osm-enclosing-gap-short = OpenStreetMap-områder som omslutter hele kartområdet uten å ha et hjørne inni det, kan mangle her.
permits-operator-line = { $name } ({ $roles ->
        [operator] driver
        [owner] eier
       *[both] driver og eier
    }): { $objects }
permits-column-source = Kilde
permits-column-state = Status
permits-column-credit = Kreditering og lisens
permits-state-read = lest, hentet { $date }
permits-state-read-osm = lest, OSM-utdrag fra { $date }
permits-state-unreachable = ikke tilgjengelig ({ $reason })
permits-state-not-read = ikke lest
permits-no-credit = ikke brukt
permits-hint-city-state = Kartområdet ligger i Berlin eller Hamburg. Der er vanligvis Bezirksamt (bydelskontoret) eller en myndighet i delstaten ansvarlig. Disse notatene fastslår ennå ikke hvilket Bezirk det gjelder.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Verneområder angår den lavere naturvernmyndigheten (untere Naturschutzbehörde). Her er det byen { $kreis } selv.
        [no] Verneområder angår den lavere naturvernmyndigheten (untere Naturschutzbehörde), vanligvis { $kreis }.
       *[unknown] Verneområder angår den kompetente lavere naturvernmyndigheten (untere Naturschutzbehörde), vanligvis Landkreis eller kreisfreie Stadt.
    }
permits-hint-national-park = En nasjonalpark har egne regler. Spør nasjonalparkforvaltningen tidlig.
permits-hint-nsg-paths = I et naturreservat må man vanligvis holde seg til stiene. For et løp utenfor stiene, spør naturvernmyndigheten.
permits-hint-natura2000 = Et FFH-område eller et fuglevernområde (Natura 2000) berører kartområdet. For et arrangement kan naturvernmyndigheten be om en vurdering av virkningene.
permits-hint-protected-areas-pl = I kartområdet ligger verneområder eller verneobjekter. For en nasjonalpark, en landskapspark eller et naturreservat er parkens eller reservatets forvaltning vanligvis kontaktpunktet; for de andre områdene og objektene kan det regionale direktoratet for miljøvern (RDOŚ) i voivodskapet som regel si hvem som er ansvarlig. Spør i god tid hvis det planlegges et arrangement der.
permits-hint-order-office = { $amt ->
        [none] For gater, plasser og annen offentlig grunn er vanligvis Ordnungsamt (ordensetaten) i kommunen (Gemeinde) { $gemeinde } ansvarlig.
       *[other] For gater, plasser og annen offentlig grunn er vanligvis Ordnungsamt (ordensetaten) ansvarlig: { $amt }, for kommunen (Gemeinde) { $gemeinde }.
    }
permits-hint-park-operator = Driver av en park eller hage i kartområdet ifølge OpenStreetMap: { $operator }. Spør dem før du bruker den.
permits-hint-school-operator = Driver av skolen eller institusjonen ifølge OpenStreetMap: { $operator }. For området deres, spør rektor eller dette organet.
permits-section-districts = Bezirke (bydeler, { $source })
permits-section-green-spaces = Grøntområder ({ $source })
permits-section-owners = Arealer som tilhører delstaten og Bund ({ $source })
permits-section-monuments = Kulturminner ({ $source })
permits-section-protected-objects = Naturminner og andre vernede objekter ({ $source })
permits-none-districts = Ifølge { $source } ligger kartområdet ikke i noe Bezirk.
permits-none-green-spaces = Ifølge { $source } ligger det ikke noe offentlig grøntområde i kartområdet.
permits-none-owners = Ifølge { $source } ligger det ingen parsell som tilhører delstaten eller Bund i kartområdet.
permits-none-monuments = Ifølge { $source } ligger det ikke noe kulturminne i kartområdet.
permits-none-protected-objects = Ifølge { $source } ligger ingen av de vernede objektene den har, i kartområdet.
permits-none-sampled = I punktene det ble spurt om, navngir { $source } ingenting her.
permits-gaps = Dataene fra { $source } er ikke fullstendige: { $gaps }.
permits-sampled = { $source } svarer bare for enkeltpunkter. freemapper spurte i et rutenett av punkter over kartet: andelene er anslag, og det som ligger mellom punktene, kan mangle.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = formål: { $dedication }
permits-playgrounds-none = Ingen offentlig lekeplass ligger i kartområdet.
permits-playgrounds-one = Én offentlig lekeplass ligger i kartområdet. Den er ikke oppført for seg.
permits-playgrounds-many = { $count } offentlige lekeplasser ligger i kartområdet. De er ikke oppført enkeltvis.
permits-owner-other = Andre offentlige eiere
permits-owners-rest = For resten av området navngir dataene ingen eier.
permits-monuments-total = Kulturminner helt eller delvis i kartområdet: { $count }.{ $links ->
        [yes] {" "}Lenkene fører til oppføringene deres i kulturminnelisten.
       *[no] {""}
    }
permits-monuments-links-left = { $count } oppføringer til er bare talt her, uten lenke.
permits-object-kind = { $kind ->
        [natural-monument] Naturminne (Naturdenkmal)
        [protected-landscape-part] Vernet landskapselement (geschützter Landschaftsbestandteil)
        [biotope] Lovfestet vernet biotop (geschütztes Biotop)
        [national-park] Nasjonalpark (park narodowy)
        [park-buffer-zone] Buffersone rundt en park (otulina)
        [landscape-park] Landskapspark (park krajobrazowy)
        [nature-reserve] Naturreservat (rezerwat przyrody)
        [protected-landscape-area] Verna landskapsområde (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000-område for naturtyper (SOO)
        [natura2000-birds] Natura 2000-område for fugler (OSO)
        [ecological-site] Økologisk område (użytek ekologiczny)
        [landscape-complex] Natur- og landskapskompleks (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentasjonssted (stanowisko dokumentacyjne)
        [natural-monument-pl] Naturminne (pomnik przyrody)
       *[special-rules] Område med spesielle regler
    }
permits-object-count = { $count } objekter
permits-object-citations = Rettsgrunnlag:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] For gater, plasser og grøntområder i Bezirk { $district } er vanligvis Bezirksamt { $district } ansvarlig, der Fachamt Management des öffentlichen Raumes (kontoret for offentlig rom).
       *[berlin] For gater, plasser og grøntområder i Bezirk { $district } er vanligvis Ordnungsamt (ordensetaten) og Straßen- und Grünflächenamt (vei- og parketaten) ved Bezirksamt { $district } ansvarlige.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] For kulturminnene i kartområdet er Denkmalschutzamt Hamburg (kulturminneetaten) ansvarlig.
       *[berlin] For kulturminnene i Bezirk { $district } er den lavere kulturminnemyndigheten (untere Denkmalschutzbehörde) ved Bezirksamt { $district } ansvarlig.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Om det ligger fredede kulturminner i kartområdet, kan ikke dataene her si sikkert.
       *[yes] Det finnes fredede kulturminner i kartområdet.
    } Om et løp berører dem, for eksempel med en post ved et kulturminne eller en etappe gjennom et vernet område, avgjør den lavere kulturminnemyndigheten (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Her er det byen { $kreis } selv.
        [no] , vanligvis { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; i noen delstater er det kommunen (Gemeinde) { $gemeinde }.
        }
       *[unknown] {" "}(vanligvis Landkreis eller kreisfreie Stadt, i noen delstater kommunen).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Naturminner, vernede landskapselementer og vernede biotoper på kartet er en sak for den lavere naturvernmyndigheten (untere Naturschutzbehörde). Her er det byen { $kreis } selv.
        [no] Naturminner, vernede landskapselementer og vernede biotoper på kartet er en sak for den lavere naturvernmyndigheten (untere Naturschutzbehörde), vanligvis { $kreis }.
       *[unknown] Naturminner, vernede landskapselementer og vernede biotoper på kartet er en sak for den lavere naturvernmyndigheten (untere Naturschutzbehörde), vanligvis Landkreis eller kreisfreie Stadt.
    }
permits-hint-biotope = Lovfestet vernede biotoper ligger i kartområdet{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Alt som kan ødelegge dem eller skade dem betydelig, er forbudt (§ 30 BNatSchG). Plasser poster og etapper utenfor dem, eller spør den lavere naturvernmyndigheten (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] For naturminner og vernede landskapselementer i Bezirk { $district } er vanligvis Bezirksamt { $district } ansvarlig.
       *[berlin] For naturminner og vernede landskapselementer i Bezirk { $district } er den lavere naturvernmyndigheten (untere Naturschutzbehörde) ved Bezirksamt { $district } ansvarlig.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Ifølge delstatens åpne data tilhører { $share } av kartområdet andre offentlige eiere.
       *[no] Ifølge delstatens åpne data tilhører { $share } av kartområdet eieren "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Eierne av resten av området kan ikke finnes ut fra åpne data. Med en berettiget interesse (for eksempel som arrangør) kan du be tinglysingen (Grundbuchamt) om et utdrag; for offentlig grunn hjelper den kompetente Ordnungsamt eller parketaten.
       *[no] Eierne av disse områdene kan ikke finnes ut fra åpne data. Med en berettiget interesse (for eksempel som arrangør) kan du be tinglysingen (Grundbuchamt) om et utdrag; for offentlig grunn hjelper den kompetente Ordnungsamt eller parketaten.
    }
permits-section-forest-units = Skogforvaltning ({ $source })
permits-section-state-forest = Delstatsskog ({ $source })
permits-none-forest-units = Ifølge { $source } ligger kartområdet i området til ikke noe skogkontor.
permits-section-admin-units = Administrative enheter ({ $source })
permits-none-admin-units = Ifølge { $source } ligger kartområdet ikke i noen administrativ enhet i registeret.
permits-section-register-monuments = Monumenter i det nasjonale registeret ({ $source })
permits-none-register-monuments = Ifølge { $source } ligger det ingen registrert monument i rektangelet rundt kartområdet.
permits-register-monuments-total = Registrerte monumenter i rektangelet rundt kartområdet: { $count }. Registeret oppgir ingen posisjon, så disse merknadene kan ikke si hvor de ligger; det må sjekkes på kartet.
permits-register-monuments-left = { $count } flere er bare talt med her, uten navn.
permits-section-register-archaeology = Arkeologiske monumenter i det nasjonale registeret ({ $source })
permits-none-register-archaeology = Ifølge { $source } ligger det ingen registrert arkeologisk monument i rektangelet rundt kartområdet.
permits-register-archaeology-total = Registrerte arkeologiske monumenter i rektangelet rundt kartområdet: { $count }. Registeret oppgir ingen posisjon, så disse merknadene kan ikke si hvor de ligger; det må sjekkes på kartet.
permits-register-archaeology-left = { $count } flere er bare talt med her, uten navn.
permits-section-register-history = Historiens minnesmerker ({ $source })
permits-none-register-history = Ifølge { $source } ligger det ikke noe historiens minnesmerke i rektangelet rundt kartområdet.
permits-register-history-in-area = Historiens minnesmerker i kartområdet: { $count }.
permits-register-history-nearby = Flere historiens minnesmerker i rektangelet rundt kartområdet, men ikke i kartområdet: { $count }.
permits-register-history-left = { $count } til er bare talt med her, uten navn.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Ifølge { $source } ligger det ingen delstatsskog i kartområdet.
permits-forest-units-reach = Et skogkontor er ansvarlig for skogen i sitt område. Området kan også omfatte byer og jorder.
permits-state-forest-office = ansvarlig: { $office }
permits-hint-forest-office = { $units ->
        [none] Skogkontor som er ansvarlig for skogen i kartområdet: { $office } ({ $administration }). Spør der tidlig når du planlegger et arrangement i skogen.
       *[other] Skogkontor som er ansvarlig for skogen i kartområdet: { $office } ({ $administration }), { $units }. Spør der tidlig når du planlegger et arrangement i skogen.
    }
permits-hint-forest-owner = { $office ->
        [known] Et arrangement i skogen krever samtykke fra skogeierne. Skogkontoret kan vanligvis si hvem som eier skogen. I flere delstater må et organisert arrangement i skogen dessuten meldes til skogmyndigheten eller tillates av den.
       *[unknown] Et arrangement i skogen krever samtykke fra skogeierne. I flere delstater må et organisert arrangement i skogen dessuten meldes til skogmyndigheten eller tillates av den. Disse notatene navngir ikke skogkontoret som er ansvarlig her; skogforvaltningen i delstaten eller den lavere skogmyndigheten kan si det.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Et arrangement i skogen krever vanligvis samtykke fra skogeieren. Skogdistriktet (Nadleśnictwo) som er nevnt ovenfor, kan som regel si hvem som forvalter skogen. Skog som ikke hører til Statsskogene, for eksempel privat skog, finnes ikke i dataene til disse merknadene. Skog som ikke tilhører staten, for eksempel privat eller kommunal skog, er under tilsyn av starosten (starosta) når det gjelder skogforvaltningen (skogloven, art. 5 ust. 1 pkt 2); spør der hos starostwo powiatowe.
       *[unknown] Et arrangement i skogen krever vanligvis samtykke fra skogeieren. Disse merknadene nevner ikke noe skogdistrikt i Statsskogene for kartområdet; dataene dekker bare Statsskogene, så skogen kan tilhøre noen andre, for eksempel en privat eier. Hvem eieren er, må finnes ut separat. Skog som ikke tilhører staten, for eksempel privat eller kommunal skog, er under tilsyn av starosten (starosta) når det gjelder skogforvaltningen (skogloven, art. 5 ust. 1 pkt 2); spør der hos starostwo powiatowe.
    }
permits-hint-monument-register-pl = Monumenter i det nasjonale registeret ligger i rektangelet rundt kartområdet. Posisjonene deres står ikke i dataene, så sjekk på kartet hvor de er. Etter art. 36 ust. 1 i loven om vern av monumenter (ustawa o ochronie zabytków) krever konserverings-, restaurerings- og byggearbeid på et registrert monument tillatelse fra voivodskapets monumentkonservator (Wojewódzki Konserwator Zabytków). Siden 3. juni 2026 krever byggearbeid i omgivelsene til et monument, endret bruk, innretninger eller skilt som settes opp på det og andre handlinger som kan endre substansen eller utseendet, bare en melding til konservatoren (art. 36aa); konservatoren kan komme med innsigelse innen 60 dager og da kreve tillatelse. Loven nevner ikke et løp eller en post som sådan; spør konservatorens kontor hvis et arrangement berører et monument eller omgivelsene.
permits-hint-archaeology-register-pl = Arkeologiske monumenter i det nasjonale registeret ligger i rektangelet rundt kartområdet. Posisjonene deres står ikke i dataene, så sjekk på kartet hvor de er. Etter art. 36 ust. 1 i loven om vern av monumenter (ustawa o ochronie zabytków) krever arkeologiske undersøkelser (pkt 5), leting etter skjulte eller forlatte løse monumenter med elektroniske eller tekniske apparater som metalldetektorer (pkt 12) og konserverings-, restaurerings- eller byggearbeid på et registrert monument (pkt 1) tillatelse fra voivodskapets monumentkonservator (Wojewódzki Konserwator Zabytków). Tillatelsen til leting faller bort 1. januar 2027: art. 36b lar da en voksen person lete med samtykke fra eieren og besitteren av eiendommen etter en melding i søkeregisteret via kulturdepartementets app, og art. 36c forbyr slik leting blant annet på registrerte monumenter og nærmere enn 10 m fra dem. Loven nevner ikke en tur, et løp eller en post som sådan; spør forvalterens kontor hvis et arrangement eller feltarbeid berører et registrert sted.
permits-hint-history-register-pl = Et historiens minnesmerke (Pomnik historii) ligger i eller nær kartområdet; avsnittet om historiens minnesmerker sier hvilket. Presidenten i Republikken Polen erklærer ved forskrift et uflyttbart minnesmerke som er oppført i registeret, eller en kulturpark av særlig verdi, til Pomnik historii (art. 15 ust. 1 i loven om vern av minnesmerker, ustawa o ochronie zabytków). Det står altså allerede under vern av registeret eller kulturparken. For arbeid på et minnesmerke som er oppført i registeret må vojvodskapets minnesmerkekonservator (Wojewódzki Konserwator Zabytków) gi tillatelse (art. 36 ust. 1); i en kulturpark fastsetter kommunestyrets vedtak forbud og begrensninger (art. 16 ust. 2, art. 17 ust. 1). Loven nevner ikke en tur, et løp eller en post som sådan; spør konservatorens kontor hvis et arrangement eller feltarbeid berører minnesmerket.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Kartområdet rommer et naturminne (pomnik przyrody), et økologisk område (użytek ekologiczny), et natur- og landskapskompleks (zespół przyrodniczo-krajobrazowy) eller et dokumentasjonssted (stanowisko dokumentacyjne). Kommunestyret (rada gminy) oppretter dem ved et vedtak, og vedtaket sier hvem som fører tilsyn med objektet og hvilke forbud som gjelder (art. 44 i naturvernloven). Loven selv nevner ingen tillatelse til et arrangement her. Be kontoret i den gminaen der objektet ligger, om vedtaket og tilsynet.
       *[other] Kartområdet rommer et naturminne (pomnik przyrody), et økologisk område (użytek ekologiczny), et natur- og landskapskompleks (zespół przyrodniczo-krajobrazowy) eller et dokumentasjonssted (stanowisko dokumentacyjne). Kommunestyret (rada gminy) oppretter dem ved et vedtak, og vedtaket sier hvem som fører tilsyn med objektet og hvilke forbud som gjelder (art. 44 i naturvernloven). Loven selv nevner ingen tillatelse til et arrangement her. Be { $count ->
        [one] kontoret i gminaen ({ $gmina }), der objektet ligger
       *[other] kontorene i gminaene ({ $gmina }), hvert for objektene på sitt område
    }, om vedtaket og tilsynet.
    }
permits-hint-gmina-roads-pl = Et arrangement som hindrer trafikken eller bruker en gate på en særskilt måte, trenger tillatelse etter art. 65 i vegtrafikkloven, søkt minst 30 dager i forveien. Den gis av myndigheten som styrer trafikken på veien: starosten for powiat- og gminaveier, ordføreren i en by med powiat-status, marskalken i voivodskapet eller GDDKiA for øvrige veier (art. 10 og 65b). Forvalter av en gminavei er wójt, burmistrz eller prezydent miasta { $count ->
        [one] i gminaen ({ $gmina });
       *[other] i hver av gminaene ({ $gmina });
    } forvalteren avgjør om veiarealet kan tas i bruk (art. 19 og 40 i loven om offentlige veier). Disse notatene viser ikke veienes kategori.
permits-hint-gmina-land-pl = Hvis grunn i kartområdet tilhører gminaen, for eksempel en kommunal park, en plass, en skoletomt eller en skog, forvaltes den av wójt, burmistrz eller prezydent miasta, som representerer gminaen (art. 30 og 31 i loven om gminaens selvstyre). Be { $count ->
        [one] kontoret i gminaen ({ $gmina }) om samtykke
       *[other] kontorene i gminaene ({ $gmina }) om samtykke
    } før du bruker den. Disse notatene viser ikke hvem som eier grunnen.
permits-section-water-protection = Vannvernområder ({ $source })
permits-section-forest-functions = Skogfunksjoner og vernskog ({ $source })
permits-none-water-protection = Ifølge { $source } ligger det ikke noe vannvernområde i kartområdet.
permits-none-forest-functions = Ifølge { $source } ligger det ingen rekreasjonsskog, ingen vernskog, ingen naturskog og ikke noe stengt skogområde i kartområdet.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreasjonsskog (Erholungswald)
        [protected] Vernskog (Bannwald eller Schutzwald)
        [natural] Naturskog eller strengt reservat (Naturwald, Totalreservat)
       *[closed] Stengt skogområde
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } i vannvernområdet "{ $name }" ligger i kartområdet. I sone I og II er arrangementer vanligvis utelukket. Spør den lavere vannmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } i vannvernområdet "{ $name }" ligger i kartområdet. Arrangementer der er vanligvis mulige; den lavere vannmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) kan si hva forskriften tillater.
       *[unknown] Vannvernområdet "{ $name }" ligger i kartområdet. Dataene sier ikke hvilken sone det er. I sone I og II er arrangementer vanligvis utelukket; spør den lavere vannmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Rekreasjonsskog (Erholungswald) ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. For arrangementer der, spør skogmyndigheten.
        [protected] Vernskog (Bannwald eller Schutzwald) ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Spør skogmyndigheten før du planlegger et arrangement der.
        [natural] Naturskog eller et strengt reservat ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Å forlate stiene er ofte forbudt der; plasser ingen poster i den uten å spørre skogmyndigheten.
       *[closed] Et skogområde det er forbudt å gå inn i, ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Hold det utenfor løypa.
    }
permits-section-roads = Klassifiserte veier ifølge OpenStreetMap
permits-none-roads = Ifølge OpenStreetMap går det ingen Autobahn og ingen Bundes-, Landes-, Staats- eller Kreisstraße gjennom kartområdet.
permits-road-line = { $class ->
        [autobahn] Autobahn (motorvei)
        [bundesstrasse] Bundesstraße (riksvei)
        [landesstrasse] Landesstraße eller Staatsstraße (delstatsvei)
       *[kreisstrasse] Kreisstraße (distriktsvei)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } går gjennom kartområdet. En løype kan krysse den bare på bruer eller gjennom underganger.
        [bundesstrasse] Bundesstraße { $refs } går gjennom kartområdet. En stenging eller en kryssing med funksjonærer pålegges av vegtrafikkmyndigheten (Straßenverkehrsbehörde); veien vedlikeholdes vanligvis av Bund gjennom delstatens veiforvaltning, i større byer ofte av byen.
        [landesstrasse] Landes- eller Staatsstraße { $refs } går gjennom kartområdet. En stenging eller en kryssing med funksjonærer pålegges av vegtrafikkmyndigheten (Straßenverkehrsbehörde); veien vedlikeholdes vanligvis av delstaten, i større byer ofte av byen.
       *[kreisstrasse] Kreisstraße { $refs } går gjennom kartområdet. En stenging eller en kryssing med funksjonærer pålegges av vegtrafikkmyndigheten (Straßenverkehrsbehörde); veien vedlikeholdes vanligvis av Kreis, i større byer ofte av byen.
    }
permits-hint-state-forest = { $office ->
        [none] Ifølge delstatens åpne data er { $share } av kartområdet delstatsskog som eies av "{ $owner }".
       *[other] Ifølge delstatens åpne data er { $share } av kartområdet delstatsskog som eies av "{ $owner }". Ansvarlig: { $office }.
    }
