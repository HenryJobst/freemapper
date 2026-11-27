# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } är inget språk som freemapper har texter för; tillgängliga: { $available }

## init

cli-part-needs-osm = --part väljer en del av ett OSM-område och kräver --osm; med --place, slå först upp id:t och använd --osm <id> --part <n>
cli-reading-file = läser { $path }
cli-no-area-for-place = inget område hittades för "{ $query }". Om platsen är ett område med flera separata delar, välj en med --osm <id> --part <n>, eller använd --geojson
cli-using-place = Använder { $name } ({ $id })
cli-created-project = Skapade projektet { $path }

## Cache

cli-no-cache-dir = ingen cachemapp finns på det här systemet; ange --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Hämtade 1 ruta
       *[other] Hämtade { $count } rutor
    }
cli-wrote = Skrev { $path }
cli-kept-map = Behöll { $path } (den kan innehålla ditt arbete)
cli-map-lacks-templates = Kartan refererar ännu inte till bildmallarna. Lägg till dem i OOM (Mallar > Öppna mall…) eller ersätt kartan med --force, som behåller den gamla som .omap.bak.
cli-map-lacks-buildings = Kartan har ännu inga ALKIS-byggnader. Ersätt den med --force för att lägga till dem; den gamla behålls som .omap.bak.
cli-map-lacks-canopies = Kartan saknar ALKIS-skärmtaken och carportarna (522) som nu är kartobjekt. Kör med --force för att lägga till dem; den gamla kartan behålls som .omap.bak.
cli-map-light = Baskartan: { $joined } linjer sammanfogade, { $left } dolda eller för små lämnade till kandidatlagren (inget flyttades)
cli-map-light-kept = Kandidatlagren innehåller nu också { $left } objekt eller sträckor som den här versionen tar bort från baskartan (taggade freemapper:generalized). Den behållna kartan har dem fortfarande, så de syns dubbelt. Ersätt kartan med --force för att rensa den (den gamla behålls som .omap.bak).
cli-map-old-candidates = Kartan namnger fortfarande templates/candidates.omap; den här körningen fyllde den med alla kandidater. Kandidaterna finns nu också i sju lager, en fil vardera (templates/candidates-*.omap). Kartan listar dem när du ersätter den med --force (den gamla behålls som .omap.bak), eller lägg till dem själv i OOM (Mallar > Öppna mall…).
cli-paper-frame-kept-map = Skrev pappersramarna som mallen templates/paper-frame.omap. Den befintliga kartan ändras inte och refererar därför inte till den: öppna den i OpenOrienteering Mapper (Mallar > Öppna mall…) eller ersätt kartan med --force (den gamla blir kvar som .omap.bak).

## PDF

cli-pdf-without-credits = PDF utan källhänvisningar: ODbL och dl-de/by-2-0 kräver att du anger källorna när du lämnar kartan vidare; se ATTRIBUTION.txt
cli-wrote-pdf = Skrev { $path } (kartan utan kandidater)
cli-wrote-pdf-candidates = Skrev { $path } (kontrollutskriften: kartan med alla kandidater, lägesriktigt; kontrollera dem i terrängen)
cli-wrote-training = Skrev { $map } och { $pdf } (träningskartan: { $joined } sammanfogade, { $shifted } förskjutna, { $left } utelämnade eller avskurna)
cli-training-free-ends = Linjeändar som lämnats fria: { $ways } där en väg mötte en annan väg, { $others } vid andra linjer
cli-training-apart-left = träningskartan har fortfarande { $length } mm (på papper) där objekt håller för litet avstånd; kontrollera det i OOM
cli-kept-training = Behöll träningskartan { $path }, så att en körning inte skriver över en som har skrivits ut eller lämnats vidare; ta bort den eller ange --force för att förnya den

## OCAD

cli-ocad-kept = Behöll { $path } (den kan innehålla ditt arbete; --force ersätter den och behåller den gamla som .ocd.bak)
cli-ocad-old-candidates = OCAD-kartan namnger fortfarande { $path } som bakgrundskarta; den här körningen fyllde den med alla kandidater. Kandidaterna finns nu också i sju lager, en fil vardera (-candidates-*.ocd). OCAD-kartan listar dem när du ersätter den med --force (den gamla behålls som .ocd.bak), eller lägg till dem själv i OCAD som bakgrundskartor.
cli-ocad-wrote = { $count ->
        [one] Skrev { $path } (1 objekt)
       *[other] Skrev { $path } ({ $count } objekt)
    }
cli-ocad-left-out = Utelämnat i OCAD-kartan: { $skipped } objekt (ingen symbol för OCAD, ingen yta, eller utanför OCAD:s ritområde på ±2 m) och { $dropped } degenererade linjedelar eller hål.
cli-ocad-load-symbols = Välj i OCAD Symbol > Load Colors and Symbols From… och ta den officiella symbolsatsen{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF:s skolsats 2019
       *[other] {" "}ISSprOM 2019
    }; den ersätter freemappers platshållarsymboler efter nummer.

## select

cli-select-running = Områdesval för { $path }: tryck på "Klar" på sidan eller Ctrl+C för att avbryta.
cli-select-no-browser = Kunde inte öppna webbläsaren ({ $error }); öppna adressen ovan.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; inget område, kan inte vara kartområde): { $place }
cli-find-hint = Skapa projektet med: freemapper init <dir> --osm { $id }
cli-find-no-area = Ingen av platserna har ett område som kan vara kartområde. Rita området med: freemapper select <dir>
cli-find-nothing = ingen plats hittades för "{ $query }"; prova ett annat namn, eller rita området med: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = varning: adaptern i { $folder } utelämnas: { $finding }
cli-adapter-note = varning: adaptern i { $folder } läses in, men: { $finding }
cli-adapter-from-project = varning: adaptern { $adapter } från projektmappen används; den kan fråga dessa värdar: { $hosts }
cli-adapter-overlaps = varning: adaptern { $adapter } täcker också mark som tillhör { $others }; dess objekt hamnar på kartor där
cli-adapter-dir-relative = varning: FREEMAPPER_ADAPTER_DIR är ingen absolut sökväg ({ $path }); den ignoreras
cli-adapters-no-project = varning: { $path } är ingen mapp; inga adaptrar från ett projekt lästes
cli-adapters-export-unknown = adaptern { $adapter } beskrivs inte av något manifest i det här programmet, så det finns inget att exportera; `freemapper adapters list` visar id:na
cli-adapters-export-taken = { $path } finns och är inte tom; exportera till en mapp som inte finns än
cli-adapters-export-done = Skrev de { $count } filerna för adaptern { $adapter } till { $path }. I dem står bara det manifestet beskriver; adresser som finns i programmets kod saknas.
cli-adapters-export-next = Ändra adresserna i manifest.toml, kontrollera mappen med `freemapper adapters check` och flytta den sedan till { $path }: Från nästa start ersätter den den inbyggda adaptern, med en varning. Tar du bort mappen är den inbyggda tillbaka.
cli-adapters-overlaps = delar mark med: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revision { $revision }  kontrollerad { $checked }  värdar: { $hosts }
cli-adapters-source-built-in = inbyggd
cli-adapters-source-user = från din mapp
cli-adapters-source-project = från projektet
cli-adapters-never-checked = aldrig
cli-adapters-user-dir = Dina adaptrar läses från { $path }
cli-adapters-no-user-dir = Det här systemet har ingen mapp för dina adaptrar; FREEMAPPER_ADAPTER_DIR anger en

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper får nu också fråga { $host } för { $name }. Posten finns i { $file }; `freemapper adapters disallow-host { $adapter } { $host }` tar tillbaka den. En områdessida eller en AI-assistent som körs läser posten när den startas på nytt
cli-allow-host-already = { $host } är redan tillåten för { $name }
cli-allow-host-declared = adaptern för { $name } anger { $host } själv; inget att tillåta
cli-allow-host-refused = { $host } kan inte tillåtas: { $reason }
cli-allow-host-unknown-adapter = det finns ingen inbyggd adapter { $adapter }; de inbyggda adaptrarna är: { $adapters }
cli-allow-host-folder-adapter = { $adapter } är en adapter i din adaptermapp; lägg i stället till värden i `[net] hosts` i dess manifest
cli-allow-host-no-config = det här systemet har ingen konfigurationsmapp att spara tillåtna värdar i
cli-allow-host-write = { $file } kunde inte skrivas: { $detail }
cli-allow-host-unreadable = { $file } går inte att läsa ({ $detail }); rätta eller ta bort filen först; inget ändrades
cli-disallow-host-removed = freemapper frågar inte längre { $host } för { $name }
cli-disallow-host-absent = { $host } var inte tillåten för { $name }; inget att ta tillbaka
cli-allowed-hosts-unreadable = varning: { $file } kunde inte läsas ({ $detail }); inga extra värdar tillåts
cli-allowed-hosts-ignored = varning: { $file }: posten { $host } för { $adapter } ignoreras: { $reason }
cli-allowed-hosts-no-adapter = freemapper har ingen inbyggd adapter med det namnet
cli-allowed-host-redundant = varning: { $host } är nu en av de värdar som adaptern för { $name } anger; din post i { $file } togs bort
cli-adapters-allowed = tillåtna av dig: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revision { $revision }, värdar: { $hosts }
cli-check-error = fel: { $finding }
cli-check-warning = varning: { $finding }
cli-check-passed = { $folder } klarar kontrollen
cli-check-passed-warnings = { $folder } klarar kontrollen, med { $count ->
        [one] en varning
       *[other] { $count } varningar
    }
cli-check-failed = { $folder } klarar inte kontrollen
cli-check-no-folder = det finns ingen mapp { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } MISSLYCKADES: { $detail }
cli-check-live-not-asked = { $capability } inte tillfrågad: { $detail }
cli-check-live-box-outside = rutan ligger inte inom gränsen för adaptern { $adapter }; en live-kontroll frågar tjänsterna om en plats som adaptern täcker
cli-check-live-cannot = live-kontrollen kan inte köras: { $detail }
cli-check-detail-not-declared = adaptern deklarerar ingen
cli-check-detail-stopped = en tidigare del misslyckades
cli-check-detail-see-warnings = se varningarna
cli-check-detail-outside-dataset = rutan ligger utanför datamängden
cli-check-detail-objects = { $count ->
        [one] ett objekt
       *[other] { $count } objekt
    }
cli-check-detail-layers = { $layers } lager, { $count } objekt
cli-check-detail-trees = { $count ->
        [one] ett träd
       *[other] { $count } träd
    }
cli-check-detail-extract = { $label }, data från { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = terrängmodell
cli-check-raster-surface = ytmodell
cli-check-raster-tiles = { $product }: { $count ->
        [one] en ruta
       *[other] { $count } rutor
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } och { $last }

## Folder lock

cli-lock-held = en annan freemapper-körning använder den här mappen (sedan { $time }, startad av { $who }). Vänta tills den är klar eller stoppa den först.
cli-lock-who-cli = terminalen
cli-lock-who-mcp = en AI-agent (MCP)
cli-lock-who-page = urvalssidan
cli-lock-who-unknown = ett annat program
cli-lock-failed = kunde inte låsa projektmappen för den här körningen: { $error }
cli-lock-stuck = något annat än en låsfil ligger i vägen och kunde inte tas bort

## Run routes of the selection page

cli-run-bad-request = Körningsalternativen är inte giltiga: { $error }
cli-run-no-project = Det finns inget sparat projekt i den här mappen ännu. Spara området först.
cli-run-already-running = En körning ({ $job }) pågår redan för det här projektet. Vänta tills den är klar eller avbryt den först.
cli-run-disk-stops = Inte tillräckligt med ledigt diskutrymme för nedladdningen: { $messages }
cli-run-busy-elsewhere = Ett annat projekt i det här programmet kör en pipeline just nu. Projekten delar nedladdningscachen, så bara en körning går åt gången. Försök igen när den är klar.
cli-run-nothing-to-cancel = Ingen körning pågår, så det finns inget att avbryta.
cli-run-unknown-job = Det finns ingen körning { $job } för det här projektet.
cli-status-too-fast = Statusförfrågningarna kommer för ofta. Fråga högst fem gånger i sekunden.
cli-open-folder-failed = Mappen kunde inte öppnas ({ $error }).
cli-open-folder-no-opener = Den här datorn har inget program för att öppna mappar ({ $program }). Öppna mappen för hand.
cli-open-folder-missing = Projektmappen finns inte ännu. Spara området först.
cli-open-folder-refused = Mappnamnet är inte säkert att skicka vidare till filhanteraren.
cli-save-running = En körning pågår för det här projektet. Spara igen när den är klar.
cli-save-unknown-kind = Okänd kartsort "{ $kind }". Använd schoolyard, sprint eller forest.
cli-save-scale-too-large = Skalan { $scale } är för stor.
cli-save-bad-content = Innehållsinställningarna är inte giltiga. Inget sparades.
cli-save-bad-paper-frames = Pappersramarna är inte giltiga. Inget sparades.
cli-save-paper-frames-no-project = Det finns inget projekt i den här mappen än. Spara först ett kartområde, sedan pappersramarna.
cli-map-image-none = Skapa kartan först.
cli-map-image-gone = Den här förhandsvisningen är inte längre tillgänglig.
cli-map-image-running = En körning pågår för det här projektet. Förhandsvisningen kan beräknas när den är klar.
cli-map-image-stale = Det sparade projektet stämmer inte längre med den genererade kartan, till exempel för att området, kartsorten eller skalan ändrades. Starta först en körning som ersätter kartan ("Ersätt den befintliga kartan" på sidan, eller freemapper run --force).
cli-map-image-superseded = En nyare förhandsvisning begärdes under tiden, så den här beräknades inte.
cli-map-image-bad-content = Innehållsinställningarna är inte giltiga.
cli-map-image-bad-request = Begäran om en förhandsvisning är inte giltig. Skicka ett JSON-objekt, med innehållsinställningarna om det finns några.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – områdesval
page-heading = Välj kartområde
page-update-available = freemapper { $version } är tillgänglig. Uppdatera med
page-update-notes = Versionsanteckningar
page-folder = Projektmapp
page-pick = Sök en plats
page-query-placeholder = t.ex. Clara-Zetkin-Schule Strausberg
page-search-button = Sök
page-searching = Söker …
page-no-results = Inget hittades.
page-point-only = bara en punkt, inget område
page-draw = Rita själv
page-rectangle = Rektangel
page-polygon = Polygon
page-finish-polygon = Stäng polygonen
page-clear = Rensa
page-hint-rectangle = Klicka på två motstående hörn på kartan.
page-hint-polygon = Klicka på hörnen ett efter ett. För att stänga den, klicka på det första hörnet, dubbelklicka eller klicka på "Stäng polygonen". Esc avbryter, Backspace tar bort det sista hörnet.
page-hint-idle = Välj en plats från sökningen eller rita området själv.
page-summary = Område
page-source-osm = Från OpenStreetMap: { $name }
page-source-drawn = Ritat för hand
page-size = Storlek med buffert: { $width } × { $height } m
page-tiles-one = { $n } ruta: { $names }
page-tiles-other = { $n } rutor: { $names }
page-download = Nedladdning: cirka { $mb } MB
page-download-none = Nedladdning: allt finns redan i cachen
page-download-details = Vad som laddas ner
page-cached = i cachen
page-disk = Diskutrymme: { $volumes }
page-disk-volume = { $free } ledigt, cirka { $needed } behövs
page-cache-volume = cache
page-project-volume = projektmapp
page-no-room = ryms inte på disken
page-no-room-skip = Allt ryms inte på disken (freemapper håller { $reserve } ledigt). Hämtningen utelämnar de markerade delarna. Frigör utrymme först om du behöver dem:
page-no-room-stop = Inte tillräckligt med diskutrymme (freemapper håller { $reserve } ledigt). Det går inte att göra en karta utan de markerade delarna, så hämtningen stoppar innan något laddas ner. Frigör utrymme först:
page-disk-unknown = freemapper kan inte läsa det lediga diskutrymmet ({ $error }). Den hämtar utan kontrollen.
page-too-large = Området är för stort: högst { $max } m per sida (med buffert). Välj ett mindre.
page-outside = Området ligger utanför de delstater som stöds ({ $states }); freemapper har ännu inga datakällor för andra regioner.
page-outside-unknown = Området ligger utanför de regioner som freemapper stöder; det har ännu inga datakällor där.
page-adapters-failed = Flygfotona kan inte listas ({ $detail }). Kartan visar bara OpenStreetMap; allt annat fungerar.
page-settings = Karta
page-kind = Kartsort
page-schoolyard = Skolgårdskarta (1:1000)
page-sprint = Sprintkarta (1:4000)
page-forest = Skogskarta (1:15000)
page-scale = Skala 1:
page-paper-format = Pappersram
page-paper-none = ingen
page-paper-orientation = Riktning
page-paper-landscape = liggande
page-paper-portrait = stående
page-paper-name-landscape = { $format } liggande
page-paper-name-portrait = { $format } stående
page-paper-keep = Behåll ram
page-paper-delete = Ta bort
page-paper-help = Ramen hjälper bara till att välja skala och område; dra den med musen. Behållna ramar syns efter nästa körning i kartan som ett dolt lager.
page-paper-saved = Pappersramar sparade. Nästa körning lägger in dem i kartan.
page-map-title = Titel
page-contour-interval = Ekvidistans
page-contour-auto = automatisk
page-steep-slope = Branthetsgräns
page-steep-slope-auto = automatisk (27°)
page-steep-slope-hint = Från denna lutningsvinkel räknas terrängen som brant när slänter och klippor eftersöks. 27° (automatisk) passar för kullar, parker och städer. I klipplabyrinter, raviner och stenbrott är nästan varje sluttning brant, så slänter och klippor flyter ihop till en fläck och ingen hittas; 55° till 65° skiljer dem åt. Ett för högt värde behåller bara de brantaste väggarna.
page-face-slope = Väggräns
page-face-slope-auto = automatisk (70°)
page-face-slope-hint = Från denna lutningsvinkel ritas terrängen som en svart klippväggsyta. 70° (automatisk) passar för klipplabyrinter som Adršpach (5,2 % av ytan). 65° ger fler och större svarta ytor, även på branta sluttningar (8,4 %). 75° lämnar bara riktiga väggar, med fransiga kanter (2,5 %). Ett för högt värde lämnar knappt några ytor.
page-settings-fixed = Projektet finns redan; kartsort, skala och titel står i freemapper.toml. Ekvidistansen går fortfarande att ändra.
page-save = Spara
page-saving = Sparar …
page-saved = Sparat i freemapper.toml.
page-zone-changed = Sparat. Området ligger nu i en annan UTM-zon; freemapper hämtar dess data på nytt vid nästa körning.
page-provider-changed = Sparat. En annan dataleverantör tillhandahåller höjddata och flygbilder för det nya området; freemapper hämtar dess data på nytt vid nästa körning.
page-save-stale = freemapper.toml ändrades i ett annat fönster eller av ett annat program sedan den här sidan läste den. Inget sparades; sidan känner nu till det sparade läget. Spara igen för att ersätta det med området som visas här.
page-has-map = Den befintliga kartan (.omap) förblir som den är; freemapper gör nya mallar och kandidater för det nya området. Alternativet "Ersätt den befintliga kartan" nedan (eller "freemapper run --force") gör också kartan på nytt och behåller den gamla som .omap.bak.
page-done = Klar
page-source = Källkod (AGPL-3.0-or-later)
page-closed = Klart. Du kan stänga det här fönstret.
page-next = Nästa: freemapper run "{ $dir }"
page-made-files = Skapat: { $files }. Mapp: { $dir }
page-back-to-agent = Säg till din AI-assistent att området är sparat.
page-nothing-saved = Inget sparades.
page-failed = Fel: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Flygfoto { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Vad som hamnar på kartan
page-content-help = Profilen avgör vad som hamnar direkt på kartan och vad som förblir en kandidat att kontrollera. Ett val för en grupp går före profilen. Det sparas med området.
page-content-profile-legend = Profil
page-content-profile-safe = Säker
page-content-profile-safe-hint = Varje objekt stannar där bygget lägger det: byggnader och katasterytor hamnar på kartan; murar, staket, träd och mycket från OpenStreetMap (dammar, stigar utan känt underlag) förblir kandidater.
page-content-profile-balanced = Balanserad
page-content-profile-balanced-hint = Objekt ur katastret, registren och gatuinventeringen hamnar på kartan, med höjdkurvorna och de områden som inte får beträdas längs järnvägsspår. Murar, staket, häckar, slänter, trappor, vatten, träd, minnesmärken, passager, stigar och järnvägar kommer också från OpenStreetMap. Gatuinredning, de övriga områden som inte får beträdas och formerna från terrängmodellen förblir kandidater.
page-content-profile-full = Full
page-content-profile-full-hint = Allt hamnar på kartan, som i kontrollutskriften; bara dolda kandidater utelämnas.
page-content-groups-legend = Per grupp
page-content-groups-help = "Som i profilen" följer profilen. Vad det betyder för en grupp beror på varifrån varje objekt kommer.
page-content-place-profile = Som i profilen
page-content-place-map = Karta
page-content-place-candidates = Kandidat
page-content-place-off = Av
page-content-group-walls = Murar
page-content-group-fences = Staket
page-content-group-hedges = Häckar
page-content-group-slopes = Slänter och klippor
page-content-group-stairs = Trappor
page-content-group-trees = Träd och trädgrupper
page-content-group-water-features = Vatten, fontäner och källor
page-content-group-monuments = Torn, minnesmärken och kraftledningar
page-content-group-furniture = Gatuinredning och lyktor
page-content-group-passages = Passager
page-content-group-contours = Höjdkurvor
page-content-group-landforms = Kullar, försänkningar, slänter och klippor från terrängmodellen
page-content-group-vegetation = Vegetation från laserpunktmoln (Brandenburg, Berlin, Nordrhein-Westfalen, Rheinland-Pfalz, Saarland och Bayern; av tills du väljer den; laddar ner cirka 40 till 600 MB per km²)
page-content-group-out-of-bounds = Områden som inte får beträdas
page-content-group-paths = Stigar och järnvägar
page-content-group-streets = Gator och torg
page-run-heading = Skapa kartan
page-preview-heading = Förhandsvisning av kartan
page-preview-stored = Som senast genererad.
page-preview-live = Med inställningarna på den här sidan, inte sparade ännu.
page-preview-live-saved = Med de sparade inställningarna, ännu inte genererad i en körning.
page-preview-computing = Beräknar …
page-preview-none = Ingen karta genererad ännu.
page-preview-later = Den här kartan har ingen förhandsvisning ännu. Den kommer med nästa körning som skriver kartan.
page-preview-stale = Området, kartsorten eller skalan ändrades sedan kartan genererades. Bilden visar fortfarande kartan som senast genererad; en ny förhandsvisning kräver en körning med "Ersätt den befintliga kartan".
page-preview-area-unsaved = Området på den här sidan är inte sparat ännu. Bilden visar kartan som senast genererad; det nya området kräver en körning med "Ersätt den befintliga kartan".
page-preview-run = Som senast genererad. Sidan uppdaterar bilden när körningen är klar.
page-preview-open = Öppna i full storlek
page-preview-alt = Förhandsvisning av kartobjekten
page-opt-ocad = Skriv ett OCAD-startprojekt
page-help-ocad = För alla som fortsätter i OCAD.
page-opt-pdf = Utskrivbar PDF
page-help-pdf = Bara kartobjekt, inga kandidater.
page-opt-pdf-candidates = Kontrollutskrift med alla kandidater
page-opt-training = Träningskarta
page-help-training = Kartan och alla kandidater, rensade så att de kan skrivas ut som de är; med OCAD också som OCAD-fil.
page-views-heading = Vyer av träningskartan
page-view-no-paths = Utan stigar (Ultimate-karta)
page-view-brown-only = Bara brunt (höjdkurvorientering)
page-view-no-north-lines = Utan nordlinjer
page-view-custom = En annan vy
page-help-views = Varje ibockad vy blir en fil till bredvid träningskartan, `<name>-training-<view>.omap` och dess PDF, med OCAD också en OCAD-fil. En annan vy är `no:505,506` (utelämna symboler) eller `only:101,102` (behåll bara dessa); `+` kombinerar vyer. Titel, skala och källor stannar alltid kvar.
page-help-views-forest = Utan stigar och bara brunt gäller bara skogskartor.
page-opt-scale-bar = Skalstock på PDF:erna
page-help-scale-bar = Tyska kartor har oftast ingen.
page-opt-sources = Källrader på PDF:erna
page-sources-auto = automatisk
page-sources-full = full
page-sources-credit-only = bara freemapper
page-warn-credit-only = PDF:erna anger då bara freemapper. Datalicenserna (ODbL, dl-de/by-2-0) kräver källhänvisning när du lämnar kartan vidare; då anger du källorna själv. ATTRIBUTION.txt listar dem alla.
page-need-pdf = Skalstocken och källraderna gäller bara när en PDF skapas.
page-opt-offline = Bara cache, inget nätverk
page-opt-force = Ersätt de befintliga kartorna
page-help-force = Kartan och, med "Träningskarta" ibockad, träningskartan. De gamla filerna behålls som .omap.bak.
page-opt-permits = Tillståndsnoteringar
page-help-permits = Förvaltningsenheter, skyddade områden och vem du ska fråga, i permits.md. En vägledning, inte juridisk rådgivning.
page-run-start = Skapa karta
page-run-again = Skapa igen
page-run-cancel = Avbryt
page-run-cancelling = Avbryter …
page-warnings-heading = Varningar
page-where-files = Filerna ligger i projektmappen:
page-open-folder = Öppna mappen
page-log-label = Körningens logg
page-force-title = Ersätta de befintliga kartorna?
page-force-text = Den befintliga kartan ersätts, och med "Träningskarta" ibockad också träningskartan. Dina ändringar går inte förlorade: de gamla filerna stannar i projektmappen som .omap.bak. Fortsätta?
page-force-cancel = Avbryt
page-force-confirm = Ersätt
page-step-fetch = Hämtar data
page-step-derive = Beräknar mallar
page-step-map = Skriver kartan
page-step-ocad = Skriver OCAD-projektet
page-step-pdf = Skriver PDF:en
page-step-pdf-candidates = Skriver kontrollutskriften
page-step-training = Skriver träningskartan
page-step-permits = Skriver tillståndsnoteringarna
page-step-line = Steg { $index } av { $total }: { $label }
page-starting = Körningen startar …
page-file-kind-map = karta
page-file-kind-template = mall
page-file-kind-ocad = OCAD-karta
page-file-kind-ocad-candidates = OCAD-kandidater
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrollutskrift
page-file-kind-training-map = träningskarta
page-file-kind-training-pdf = träningskarta som PDF
page-file-kind-training-ocad = OCAD-träningskarta
page-file-kind-training-view-map = vy av träningskartan
page-file-kind-training-view-pdf = vy av träningskartan som PDF
page-file-kind-training-view-ocad = vy av OCAD-träningskartan
page-file-kind-preview = förhandsvisning av kartan
page-file-kind-permits = tillståndsnoteringar
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Klart. Filerna är skrivna.
page-result-failed = Körningen misslyckades: { $error }
page-result-failed-kept = Det som är skrivet stannar i projektmappen; en ny körning fortsätter därifrån.
page-result-cancelled = Avbrutet. Det som redan är nedladdat stannar i cachen; en ny körning fortsätter därifrån.
page-result-title-done = Kartan är skapad
page-result-title-failed = Körningen misslyckades
page-result-title-cancelled = Körningen avbröts
page-kept-map = Den befintliga kartan lämnades som den är. "Ersätt den befintliga kartan" gör den på nytt och behåller den gamla som .omap.bak.
page-no-files = Inga filer skapades.
page-opened-folder = Mappen är öppen.
page-open-failed = { $message } Mapp: { $dir }
page-why-no-area = Välj först ett område.
page-why-checking = Förhandsvisningen beräknas …
page-why-too-large = Området är för stort.
page-why-outside = Området ligger utanför de delstater som stöds.
page-why-disk = Inte tillräckligt med ledigt diskutrymme (se ovan).
page-why-running = En körning pågår.
page-err-scale-bar-no-pdf = Skalstocken kräver en PDF.
page-err-sources-no-pdf = Källraderna kräver en PDF.
page-save-failed = Området är inte sparat; ingen körning startades.
page-poll-lost = Inget svar från servern. freemapper försöker vidare.
page-run-unknown = Servern känner inte längre till den här körningen.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Stäng

## update (spec 3.4 to 3.6)

update-source-build = Den här freemapper byggdes från källkod. Uppdatera den med git pull och cargo build --release --locked.
update-current = freemapper { $version } är aktuell.
update-available = freemapper { $new } är tillgänglig (du har { $old }), utgiven { $released }.
update-details =
    Versionsanteckningar: { $notes }
    Nedladdning: { $size } MB
update-decimal-separator = ,
update-ask = Installera nu? [y/N]
update-not-installed = Inget installerades.
update-needs-yes = indata är inte en terminal, så freemapper kan inte fråga före installationen; kör freemapper update --yes för att installera utan att fråga
update-installing = laddar ner och kontrollerar freemapper { $version }
update-done = freemapper uppdaterades från { $old } till { $new }. Versionsanteckningar: { $notes }
update-mcp-restart = En MCP-server som körs behåller den gamla versionen tills AI-assistenten startar om den.
update-rolled-back = Gick tillbaka till freemapper { $version }. Versionen du lämnade sparas som { $path }.
update-not-writable = kan inte skriva i programmappen { $path }, och freemapper ber inte om administratörsrättigheter. Packa upp den nya versionen där för hand, eller flytta freemapper till en mapp du kan skriva i.
update-too-old = freemapper { $new } kan inte installeras av självuppdateringen i { $old }; installera { $needed } eller nyare för hand, se { $notes }
update-busy = en annan uppdatering eller återgång pågår i programmappen (låsfil { $path }); vänta tills den är klar och försök igen
update-bad-signature = versionen kunde inte verifieras ({ $error }); inget installerades
update-check-bad-signature = den senaste freemapper-versionen kunde inte verifieras ({ $error }); inget installerades
update-no-manifest = den senaste versionen kan inte installera sig själv (den har inget uppdateringsmanifest); ladda ner den från { $page }
update-no-manifest-version = versionen { $version } finns inte eller kan inte installera sig själv (den har inget uppdateringsmanifest); se { $page }
update-no-asset = versionen har inget program för det här systemet ({ $platform }); inget installerades
update-no-previous = det finns ingen tidigare version att gå tillbaka till ({ $path } saknas)
update-failed = uppdateringen misslyckades: { $error }
update-mode-set = Uppdateringsläget är satt till { $mode } i { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } åsidosätter den här inställningen så länge den är satt.
update-mode-failed = kunde inte skriva { $path }: { $error }
update-notice = freemapper { $new } är tillgänglig (du har { $old }). Uppdatera med: freemapper update
update-applies-next-start = Den nya versionen gäller från nästa start av freemapper.
update-install-failed = freemapper { $new } installerades inte: { $error }
update-no-config-dir = ingen konfigurationsmapp på det här systemet; sätt FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Obs till personen: vid starten hittade freemapper-servern det här om adaptermapparna. För det vidare; personen rättar mappen eller tar bort den och startar sedan om servern.
    { $lines }
update-mcp-note = Obs till personen: freemapper { $new } är tillgänglig (det här är { $old }). Fråga personen innan du anropar install_update.
update-mcp-needs-confirm = inget installerades; fråga först personen och anropa install_update med confirm: true när hen samtycker
update-mcp-note-manual = Obs till personen: freemapper { $new } är tillgänglig (det här är { $old }), men install_update kan inte installera den; installera { $needed } eller nyare för hand, se { $notes }
update-mcp-note-bad-signature = Obs till personen: den senaste freemapper-versionen kunde inte verifieras ({ $error }); inget installerades. Berätta det för personen och installera den inte på något annat sätt.
update-mcp-restart-now = Starta om freemapper-MCP-servern nu; den kör fortfarande den gamla versionen.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = fel
error-cancelled = avbrutet
cli-cancelled-kept = avbrutet; behållet: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disken är full, så freemapper kunde inte skriva { $path }. Frigör utrymme och försök igen

## Errors: project

error-project-already-exists = projektet finns redan: { $path }
error-project-map-exists = kartan finns redan (den kan innehålla ditt arbete); använd --force för att ersätta den: { $path }
error-project-invalid-scale = ogiltig skala 1:{ $scale }
error-project-invalid-contour-interval = ogiltig ekvidistans { $interval } m i freemapper.toml; den måste vara större än 0
error-project-invalid-steep-slope = ogiltig branthetsgräns { $slope }° i freemapper.toml; den måste ligga mellan 15° och 80°
error-project-invalid-face-slope = ogiltig väggräns { $slope }° i freemapper.toml; den måste ligga mellan 55° och 85°
error-project-invalid-paper-frame-centre = ogiltig mittpunkt för pappersram, longitud { $lon }, latitud { $lat }; den måste vara en plats på jorden
error-project-scale-not-allowed = skala 1:{ $scale } är inte tillåten för { $kind }; använd { $allowed }
error-project-scale-not-allowed-enlarge = skala 1:{ $scale } är inte tillåten för { $kind }; använd { $allowed }. Rita i 1:4000, freemapper-print förstorar till 1:3000 vid utskrift
word-kind-schoolyard = skolgårdskartor
word-kind-sprint = sprintkartor
word-kind-forest = skogskartor
error-project-toml = ogiltig projektfil: { $message }
error-project-unknown-adapter = projektfilen anger adaptern "{ $adapter }", som den här freemapper inte har; en nyare freemapper kan ha den (`freemapper update`), eller så kommer den från din adaptermapp (`freemapper adapters list` visar den) eller, med --adapters-from-project på kommandoraden, från mappen `adapters` i projektet (bara för ett projekt du litar på)
error-project-not-fetched = inga rutor hämtade ännu; kör först `freemapper fetch`
error-project-ocd = OCAD-export: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: bild och world-fil stämmer inte med kartområdet; kör `freemapper derive` igen
error-project-tiles-outdated = kartområdet behöver rutorna { $tiles } som inte hämtades; kör `freemapper fetch`
error-project-buildings-not-fetched = inga ALKIS-byggnader sparade för det här kartområdet; kör först `freemapper fetch` online
error-project-orthophoto-not-fetched = inget ortofoto sparat för det här kartområdet; kör först `freemapper fetch` online
error-project-terrain-not-fetched = ingen terrängmodell sparad för det här kartområdet; kör först `freemapper fetch` online
error-project-candidates-outdated = de sparade OSM-kandidaterna täcker inte kartområdet; kör `freemapper fetch`
error-project-stored = { $path }: { $message }; kör `freemapper fetch` igen
error-project-area-too-large = kartområdet plus buffert är { $width } × { $height } m, mer än { $limit } m per sida; minska kartområdet eller buffer_m i freemapper.toml
error-project-area-crosses-country = kartområdet ligger på båda sidor om en landsgräns; kartor över en gräns stöds inte ännu, så rita området inom ett land
error-project-permits-not-for-region = tillståndsnoteringarna är inte tillgängliga för den här regionen ännu: de frågar tyska tjänster och nämner tyska myndigheter
error-project-outside-coverage = kartområdet ligger utanför de delstater som stöds ({ $states }); freemapper har hittills datakällor bara för dem

## Errors: disk space

error-space-required = inte tillräckligt med diskutrymme för { $part }. Det behövs { $needed }, men { $path } har bara { $left } ledigt utöver reserven på { $reserve }. Frigör utrymme och kör `freemapper fetch` igen
error-space-skipped = freemapper hoppar över { $part } på grund av brist på diskutrymme. Det behövs { $needed }, men { $path } har bara { $left } ledigt utöver reserven på { $reserve }. Frigör utrymme och kör `freemapper fetch` igen
word-part-map-data = kartdata från ALKIS och OSM
word-part-terrain = terrängmodellen
word-part-orthophoto = ortofotot
word-part-surface = ytmodellen för nDSM-mallen
word-part-osm-update = uppdateringen av det cachade OSM-extraktet
word-part-k-five = Berlins K5-kartmall
word-part-berlin-layers = Berlins ALKIS-strukturer, trädkataster, gatuinventering och punktlager
word-part-state-layers = de fullständiga ALKIS-strukturerna, objekten i landskapsmodellen och trädkatastren
word-part-point-cloud = laserpunktmolnen för vegetationskandidaterna

## Errors: downloads

error-fetch-http = nedladdningen av { $url } misslyckades: { $message }
error-fetch-http-temporary = nedladdningen av { $url } misslyckades: { $message }. Tjänsten är förmodligen överbelastad eller under underhåll; freemapper har frågat upp till { $attempts } gånger. Starta körningen igen om några minuter; det som redan är hämtat bevaras.
error-fetch-corrupt-archive = nedladdningen av { $url } är inte ett komplett rutarkiv: { $message }
error-fetch-corrupt-extract = nedladdningen av { $url } är inte ett komplett OSM-extrakt: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] rutan { $tiles } täcks inte av de publicerade rutorna; minska kartområdet eller buffer_m i freemapper.toml
       *[other] rutorna { $tiles } täcks inte av de publicerade rutorna; minska kartområdet eller buffer_m i freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline och inte i cachen: { $product }-rutan { $tiles }; kör `freemapper fetch` online
       *[other] offline och inte i cachen: { $product }-rutorna { $tiles }; kör `freemapper fetch` online
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = terrängmodell (DGM1)
word-product-image-dsm = ytmodell (bDOM)
word-product-saxony-dtm = terrängmodell (GeoSN DGM1)
word-product-saxony-dsm = ytmodell (GeoSN DOM1)
error-fetch-osm-extract-missing = inget OSM-extrakt i cachen; kör `freemapper fetch` online
error-fetch-not-found = { $name } hittades inte på servern
error-fetch-refused = { $url }: servern vägrade begäran (HTTP { $status })
error-fetch-refused-detail = { $url }: servern vägrade begäran (HTTP { $status }): { $detail }
error-fetch-too-large = nedladdningen av { $url } är större än { $limit } MB; stoppad
error-fetch-not-stored = ingen { $label } sparad för det här kartområdet: { $path } saknas, går inte att läsa eller täcker det inte; kör först `freemapper fetch` online
error-fetch-range-ignored = { $url }: servern ignorerade byteintervallet { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper skickar inga begäranden till den här adressen: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper skickar inga begäranden till den här adressen: adaptern för { $name } anger inte värden { $host }{ $from }. Om tjänsten har flyttat dit, tillåt värden med `freemapper adapters allow-host { $adapter } { $host }` och rapportera det på https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper skickar inga begäranden till den här adressen: adaptern { $adapter } anger inte värden { $host }{ $from }. Om tjänsten har flyttat dit, lägg till värden i `[net] hosts` i adapterns manifest
error-fetch-host-redirected-from = {" "}({ $url } omdirigerar dit)
error-reason-not-a-host-name = inget värdnamn
error-reason-not-ascii = ett namn utanför ASCII måste skrivas i sin xn--form
error-reason-not-a-dns-name = inget DNS-namn
error-reason-no-domain = ett namn på den här datorn eller utan domän
error-reason-local-name = ett namn i det lokala nätverket
error-reason-ip-address = en IP-adress, inget DNS-namn
error-reason-unexpected-character = oväntat tecken
error-reason-not-https = inte https
error-reason-user-info = användarinformation
error-reason-port = en annan port än 443
error-reason-host-not-declared = värden är inte en av de deklarerade värdarna
error-reason-local-address = en adress på den här datorn eller i det lokala nätverket, inte på en server på internet
error-reason-too-many-redirects = mer än { $count } omdirigeringar
error-reason-redirect-nowhere = en omdirigering (HTTP { $status }) till ingenstans
error-fetch-currency-csv = CSV för aktualitet, rad { $line }: { $message }
error-alkis = ogiltiga ALKIS-data: { $message }
error-osm = OSM-data: { $message }

## Errors: templates and rasters

error-derive-zip = rutarkiv: { $message }
error-derive-image = bild: { $message }
error-derive-no-tiles = inga rutor att härleda från
error-derive-area-too-large = kartområdet är för stort för en mall: { $width } × { $height } pixlar (högst { $limit } per sida); minska kartområdet eller buffer_m i freemapper.toml
error-raster-world-file = ogiltig world-fil: { $message }
error-raster-misaligned = rastren ligger inte på samma pixelrutnät: { $message }
error-raster-pixel-size = rastren har olika pixelstorlekar: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = ogiltig GeoJSON: { $message }
error-area-no-polygon = GeoJSON innehåller ingen polygon
error-area-multi-polygon = MultiPolygon med { $parts } delar; dela kartområdet i en polygon
error-area-out-of-range = koordinat utanför intervallet: lat { $lat }, lon { $lon }
error-area-not-an-area = inget användbart kartområde: { $message }
error-georef-date = datumet { $date } ligger utanför giltighetsperioden för World Magnetic Model
error-georef-map-range = punkten E { $easting } N { $northing } ligger utanför kartans koordinatintervall
error-omap-xml = ogiltig OOM-XML: { $message }
error-omap-missing-element = elementet <{ $name }> hittades inte
error-ocd-duplicate-symbol = symbolnummer { $symbol } används två gånger
error-ocd-unknown-symbol = ett objekt hänvisar till symbolen { $symbol }, som inte finns i filen
error-ocd-wrong-type = ett objekt med symbolen { $symbol } har fel typ för sin symbol

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Användning:
cli-heading-commands = Kommandon:
cli-heading-options = Alternativ:
cli-heading-arguments = Argument:
cli-help-possible-values = möjliga värden
cli-help-default = standard
cli-arg-help = Visa hjälp
cli-arg-version = Visa version
cli-help-help = Visa hjälpen för ett kommando
cli-arg-help-command = Kommandot som hjälpen ska visas för, till exempel run
cli-parse-hint = Försök med '--help' för mer information.
cli-double-click-start = freemapper öppnar sidan där du väljer kartområdet i webbläsaren. Projektmappen är { $path }. Stäng det här fönstret för att avsluta programmet.
cli-parse-unknown-argument = oväntat argument { $arg }
cli-parse-missing-required = obligatoriskt argument saknas: { $args }
cli-parse-invalid-value = ogiltigt värde { $value } för { $arg }
cli-parse-invalid-value-choices = ogiltigt värde { $value } för { $arg }; möjliga värden: { $values }
cli-parse-invalid-subcommand = okänt kommando { $value }
cli-parse-missing-subcommand = ett kommando krävs
cli-parse-too-many-values = oväntat värde { $value } för { $arg }
cli-parse-argument-conflict = { $arg } kan inte användas tillsammans med { $other }

help-freemapper = Bygger OOM-startprojekt av geodata
help-freemapper-lang = Språk för programtexterna, `de` eller `en` (standard: FREEMAPPER_LANG, sedan systemspråket, sedan engelska)
help-init = Skapa en projektmapp för ett kartområde
help-init-dir = Projektmapp att skapa
help-init-geojson = GeoJSON-fil med kartområdets polygon (EPSG:4326)
help-init-osm = OSM-objekt för kartområdet, t.ex. way/992423119
help-init-place = Platsens namn, t.ex. "Clara-Zetkin-Schule Strausberg"; freemapper använder den bästa träffen med ett område
help-init-part = Vilken del av ett OSM-område med flera separata delar som ska användas (1 är den största)
help-init-kind = Kartsort
help-arg-scale = Skalnämnare för ett nytt projekt: skolgård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andra värden nekas, eftersom freemapper-print nekar dem
help-init-title = Kartans titel för randuppgifterna (standard: mappnamnet)
help-export = Skriv startprojektet (OOM-karta, mallar, källhänvisning) till en projektmapp
help-export-dir = Projektmapp
help-arg-force = Ersätt en befintlig kartfil och behåll den gamla som `<name>.omap.bak`
help-export-ocad = Skriv också ett OCAD-startprojekt till `ocad/` (OCAD 12-format); en befintlig OOM-karta blir då kvar om du inte anger --force
help-export-pdf = Skriv också `<name>.pdf`, den utskrivbara kartan (bara kartobjekten, inga kandidater); en befintlig OOM-karta blir då kvar om du inte anger --force
help-export-pdf-candidates = Skriv också `<name>-candidates.pdf`, kontrollutskriften: kartan med alla kandidater, lägesriktigt; en befintlig OOM-karta blir då kvar om du inte anger --force
help-export-training = Skriv också träningskartan `<name>-training.omap` och dess PDF `<name>-training.pdf`: kartan och alla kandidater, rensade så att de kan skrivas ut som de är; med --ocad också `ocad/<name>-training.ocd`. En befintlig OOM-karta blir då kvar om du inte anger --force, och --force ersätter också en befintlig träningskarta
help-export-view = Med --training: skriv också träningskartan utan det en vy utelämnar, som `<name>-training-<view>.omap` och dess PDF (med --ocad också en OCAD-fil). Vyer: `no-paths` (inga vägar, stigar, trappor), `brown-only` (bara bruna symboler och ramen), `no-north-lines`, eller en lista `no:<codes>` / `only:<codes>`; kombinera med `+`. Upprepa alternativet för flera filer
help-export-pdf-scale-bar = Rita skalstocken på PDF:erna (av som standard, tyska kartor har oftast ingen; skalnumret stannar i titelblocket)
help-export-pdf-sources = Källrader på PDF:erna: `auto` skriver ut hela listan och kortar den bara när den skulle ta mer än 5 rader, `full` kortar aldrig, `credit-only` skriver bara ut freemappers krediteringsrad (ODbL och dl-de/by-2-0 kräver då att du anger källorna när du lämnar kartan vidare). ATTRIBUTION.txt listar alltid varje källa
help-fetch = Ladda ner data för kartområdet (ortofoto, terrängmodell, nDSM, ALKIS-byggnader, OSM)
help-fetch-dir = Projektmapp
help-fetch-offline = Använd bara cachen, utan nätverksåtkomst
help-arg-cache-dir = Cachemapp för nedladdade rutor (standard: systemets cachemapp + `freemapper`)
help-derive = Beräkna mallarna (ortofoto, skuggad relief, lutning, lokal relief, nDSM), höjdkurvor och landformer av de hämtade data
help-derive-dir = Projektmapp
help-run = Hämta, beräkna och exportera i ett svep
help-run-dir = Projektmapp
help-run-offline = Använd bara cachen, utan nätverksåtkomst
help-run-ocad = Skriv också ett OCAD-startprojekt till `ocad/` (OCAD 12-format)
help-run-pdf = Skriv också `<name>.pdf`, den utskrivbara kartan (bara kartobjekten, inga kandidater)
help-run-pdf-candidates = Skriv också `<name>-candidates.pdf`, kontrollutskriften: kartan med alla kandidater, lägesriktigt
help-run-training = Skriv också träningskartan `<name>-training.omap` och dess PDF `<name>-training.pdf`: kartan och alla kandidater, rensade så att de kan skrivas ut som de är; med --ocad också `ocad/<name>-training.ocd`. En befintlig träningskarta blir kvar om du inte anger --force
help-run-view = Med --training: skriv också träningskartan utan det en vy utelämnar, som `<name>-training-<view>.omap` och dess PDF (med --ocad också en OCAD-fil). Vyer: `no-paths` (inga vägar, stigar, trappor), `brown-only` (bara bruna symboler och ramen), `no-north-lines`, eller en lista `no:<codes>` / `only:<codes>`; kombinera med `+`. En befintlig vyfil blir kvar om du inte anger --force
help-run-pdf-scale-bar = Rita skalstocken på PDF:erna (av som standard; skalnumret stannar i titelblocket)
help-run-pdf-sources = Källrader på PDF:erna: `auto` skriver ut hela listan och kortar den bara när den skulle ta mer än 5 rader, `full` kortar aldrig, `credit-only` skriver bara ut freemappers krediteringsrad (ODbL och dl-de/by-2-0 kräver då att du anger källorna när du lämnar kartan vidare). ATTRIBUTION.txt listar alltid varje källa
help-select = Välj kartområde i webbläsaren genom att rita det eller välja en plats. Sparandet skriver freemapper.toml och skapar projektmappen vid behov
help-select-dir = Projektmapp (skapas vid sparandet om den inte finns)
help-select-kind = Kartsort för ett nytt projekt
help-select-title = Kartans titel för ett nytt projekt
help-select-no-browser = Skriv bara ut sidans adress; öppna inte webbläsaren
help-select-port = Port på 127.0.0.1 (standard: en ledig)
help-select-offline = Sidan startar med "Bara cache, inget nätverk" ibockat
help-select-ocad = Sidan startar med OCAD-startprojektet ibockat
help-select-pdf = Sidan startar med den utskrivbara PDF:en ibockad
help-select-pdf-candidates = Sidan startar med kontrollutskriften (alla kandidater) ibockad
help-select-training = Sidan startar med träningskartan ibockad
help-select-pdf-scale-bar = Sidan startar med skalstocken på PDF:erna ibockad
help-select-pdf-sources = Sidan startar med det här valet av källrader på PDF:erna, som för `run`
help-find = Lista OpenStreetMap-platserna för ett namn, områden först, med id:t för init --osm
help-find-query = Platsens namn, t.ex. "Clara-Zetkin-Schule Strausberg"
help-find-json = Skriv ut träffarna som JSON, samma fält som MCP-verktyget find_place
help-mcp = Tillhandahåll verktygen åt en AI-agent via MCP (stdin/stdout)
help-update = Installera en nyare freemapper från GitHub-versionerna, efter att ha frågat. Ett bygge från källkod säger i stället hur det uppdateras
help-update-check = Kontrollera bara: slutkod 10 när en nyare version finns, 0 när den här är aktuell
help-update-yes = Installera utan att fråga; behövs när indata inte är en terminal
help-update-version = Installera den här versionen (x.y.z) i stället för den senaste, även en äldre
help-update-rollback = Gå tillbaka till versionen som den senaste uppdateringen ersatte
help-update-mode = Spara vad den dagliga kontrollen gör när den hittar en nyare version (standard: notify). Fungerar i varje bygge, eftersom det bara skriver inställningen
help-adapters = Visa regionadaptrarna: de delstater och länder som freemapper har datakällor för
help-adapters-list = Lista adaptrarna med varifrån var och en kommer (inbyggd, din mapp, ett projekt), dess revision, dagen den senast kontrollerades och värdarna den frågar
help-adapters-list-json = Skriv ut listan som JSON
help-adapters-list-project = Läs också adaptrarna i mappen `adapters` i den här projektmappen, som --adapters-from-project skulle göra
help-adapters-check = Kontrollera mappen för en adapter innan den används eller lämnas vidare: om den skulle läsas in, vad dess upphovsperson bör känna till, och med --live om dess tjänster svarar. Slutkod 0: den klarar kontrollen; 1: den har anmärkningar; 2: kontrollen kunde inte köras
help-adapters-check-dir = Adapterns mapp; dess namn är adapterns id
help-adapters-check-live = Fråga också adapterns tjänster om rutan --bbox anger och säg vad var och en levererar. Begäranden stannar på de värdar manifestet deklarerar
help-adapters-check-bbox = Live-kontrollens ruta i grader, som minLon,minLat,maxLon,maxLat. Den ligger inom adapterns gräns; 150 m gånger 150 m räcker
help-adapters-check-json = Skriv ut resultatet som JSON
help-adapters-probe = Fråga varje tjänst som de inbyggda adaptrarna nämner en gång och säg om den svarar: ok, slow, moved eller error. Slutkoden är 0 även när en tjänst misslyckas; resultatet är svaret
help-adapters-probe-json = Skriv ut resultatet som JSON
help-adapters-probe-status = Läs gårdagens status från den här filen (den får saknas) och skriv ut den nya statusen som JSON: En tjänst är nere efter två dåliga nätter i rad. Ändringarna går till standardfel
help-adapters-allow-host = Tillåt en inbyggd adapter ytterligare en värd: för en delstat vars tjänst har flyttat till en server som adaptern ännu inte anger. Meddelandet om den nekade adressen säger vilken värd. Värden kontrolleras som en värd i ett manifest, och freemapper ansluter fortfarande bara till servrar på internet
help-adapters-allow-host-adapter = Den inbyggda adapterns id, som `adapters list` visar det
help-adapters-allow-host-host = Värden som ska tillåtas: ett DNS-namn, ingen adress, ingen port
help-adapters-disallow-host = Ta tillbaka en värd som allow-host tillät
help-adapters-disallow-host-adapter = Den inbyggda adapterns id
help-adapters-disallow-host-host = Värden som ska tas tillbaka
help-adapters-export = Skriv en delstats inbyggda adapter som en adaptermapp du kan ändra: dess manifest och filerna manifestet nämner, i `<dir>/<id>`. Mappen får inte finnas än eller måste vara tom. Ändra adresserna i dess `manifest.toml`, kontrollera den med `adapters check` och lägg den i din adaptermapp: Då ersätter den den inbyggda adaptern, med en varning. Tar du bort mappen är den inbyggda tillbaka
help-adapters-export-adapter = Id:t för den inbyggda adaptern, som `adapters list` visar det
help-adapters-export-dir = Mappen där mappen `<id>` skapas
help-sources = Läs och rätta adresserna till datakällorna utan ny utgåva. En rättelse hamnar i din egen adaptermapp och ersätter den inbyggda adressen
help-sources-list = Lista adresserna i manifesten med deras sökvägar och varifrån varje manifest kommer. Med ett adapter-id bara den adapterns
help-sources-get = Skriv ut en adress och inget annat. Slutkod 1 när adaptern eller sökvägen är okänd
help-sources-set = Rätta en adress i din egen adaptermapp när hela manifestet har klarat sin kontroll. En adress på en värd som manifestet inte deklarerar avvisas om du inte anger --add-host
help-sources-diff = Visa vad din adaptermapp ändrar jämfört med det inbyggda manifestet
help-sources-check = Kontrollera mappen för din adapter med det id:t som `adapters check`. Slutkod 0: godkänd; 1: fynd; 2: kontrollen kunde inte köras
help-sources-schema = Skriv ut de nycklar i ett manifest som innehåller en adress, med --json som JSON-schema
help-arg-adapter = Adapterns id, som `adapters list` visar det
help-arg-path = Adressens sökväg, som `sources list` visar den
help-sources-set-url = Den nya adressen: https, utan fråga
help-sources-set-add-host = Lägg också till den nya adressens värd bland de värdar manifestet deklarerar
cli-sources-no-user-dir = det här systemet har ingen mapp för dina adaptrar; sätt FREEMAPPER_ADAPTER_DIR till en absolut sökväg
cli-sources-bad-id = { $adapter } är inget adapter-id
cli-sources-unknown-adapter = freemapper känner ingen adapter { $adapter }; `freemapper adapters list` visar id:na
cli-sources-no-manifest = inget manifest i det här programmet beskriver adaptern { $adapter }; dess adresser finns i koden
cli-sources-unknown-path-of = adaptern { $adapter } har ingen adress under { $path }; `freemapper sources list { $adapter }` visar sökvägarna
cli-sources-bad-value = den nya adressen måste vara en https-adress på en namngiven värd, utan citattecken, omvänt snedstreck, styrtecken, användare eller port
cli-sources-host-not-declared = den nya adressen leder till värden { $host }, som manifestet inte deklarerar; --add-host deklarerar den
cli-sources-invalid = manifestet skulle inte klara kontrollen med den här ändringen, så ingenting skrevs: { $findings }
cli-sources-set-created = Mappen { $folder } för adaptern { $adapter } har gjorts av dess inbyggda manifest; den ersätter den inbyggda adaptern från nästa start.
cli-sources-set-host-added = Värden { $host } är nu deklarerad i [net] hosts; freemapper frågar nu den servern för den här adaptern.
cli-sources-set-next = Kontrollera den med `freemapper adapters check { $folder }`. Källangivelse och licens förblir som de var: håll dem riktiga om tjänsten bytte ägare. Starta om MCP-servern eller områdessidan.
cli-sources-diff-no-folder = du har ingen mapp för adaptern { $adapter } ({ $folder }); det finns inget att jämföra
cli-sources-diff-none = din mapp innehåller samma adresser som det inbyggda manifestet
help-sources-list-json = Skriv ut listan som JSON
help-sources-check-live = Fråga också adapterns tjänster om rutan --bbox anger och säg vad var och en levererar. Begäranden stannar på de värdar manifestet deklarerar
help-sources-check-bbox = Live-kontrollens ruta i grader, som minLon,minLat,maxLon,maxLat. Den ligger inom adapterns gräns; 150 m gånger 150 m räcker
help-sources-check-json = Skriv ut resultatet som JSON
help-sources-schema-json = Skriv ut resultatet som JSON
help-arg-adapters-from-project = Läs också in adaptrarna i mappen `adapters` i projektmappen. freemapper nämner var och en med de värdar den frågar. Använd det för ett projekt du litar på: en adapter får freemapper att fråga de servrar den anger

## The summary of a run, for the MCP server and the area page

cli-internal-error = internt fel: { $message }
cli-pdf-sources-invalid = pdf_sources måste vara auto, full eller credit-only, inte { $value }
cli-view-unknown = en vy är no-paths, brown-only, no-north-lines, no:<codes> eller only:<codes>, inte { $value }
cli-view-empty = listan { $value } anger ingen symbolkod
cli-view-bad-code = { $value } är ingen symbolkod som 505 eller 501.6
cli-summary-training-view = Skrev { $map } och { $pdf } (en vy av träningskartan)
cli-summary-kept-training-view = Behöll filerna för vyn { $view } av träningskartan; ange --force för att förnya dem
cli-summary-training-view-failed = Vyn av träningskartan misslyckades: { $error }
cli-estimate-disk-unreadable = det går inte att läsa det lediga diskutrymmet för { $path } ({ $message })
cli-summary-wrote-map = Skrev kartan { $path }
cli-summary-kept-map = Behöll den befintliga kartan { $path } (den kan innehålla kartritarens arbete)
cli-summary-map-lacks-templates = Kartan refererar ännu inte till bildmallarna. Starta pipelinen med force för att ersätta den och behålla den gamla som .omap.bak.
cli-summary-map-lacks-buildings = Kartan har ännu inga ALKIS-byggnader. Starta pipelinen med force för att lägga till dem; den ersätter kartan och behåller den gamla som .omap.bak.
cli-summary-map-lacks-canopies = Kartan saknar ALKIS-skärmtaken och carportarna (522) som nu är kartobjekt. Starta pipelinen med force för att lägga till dem; den ersätter kartan och behåller den gamla som .omap.bak.
cli-summary-map-light = Baskartan: { $joined } linjer sammanfogade, { $left } dolda eller för små lämnade till kandidatlagren (inget flyttades)
cli-summary-map-light-kept = Kandidatlagren innehåller nu också { $left } objekt eller sträckor som den här versionen tar bort från baskartan (taggade freemapper:generalized). Den behållna kartan har dem fortfarande, så de syns dubbelt. Starta pipelinen med force för att rensa kartan; den behåller den gamla som .omap.bak.
cli-summary-map-old-candidates = Kartan namnger fortfarande templates/candidates.omap; den här körningen fyllde den med alla kandidater. Kandidaterna finns nu också i sju lager, en fil vardera (templates/candidates-*.omap). Kartan listar dem efter en körning med force (den gamla kartan behålls som .omap.bak), eller så lägger kartritaren till dem i OOM (Mallar > Öppna mall…).
cli-summary-paper-frame-kept-map = Skrev pappersramarna som mallen templates/paper-frame.omap. Den befintliga kartan ändras inte och refererar därför inte till den: kartritaren öppnar den i OpenOrienteering Mapper (Mallar > Öppna mall…), eller så refererar kartan till den efter en körning med force (den gamla kartan blir kvar som .omap.bak).
cli-summary-wrote-ocad = Skrev OCAD-kartan { $path } ({ $count } objekt)
cli-summary-kept-ocad = Behöll den befintliga OCAD-kartan { $path } (den kan innehålla kartritarens arbete; force ersätter den)
cli-summary-ocad-old-candidates = OCAD-kartan namnger fortfarande { $path } som bakgrundskarta; den här körningen fyllde den med alla kandidater. Kandidaterna finns nu också i sju lager, en fil vardera (-candidates-*.ocd). OCAD-kartan listar dem efter en körning med force (den gamla kartan behålls som .ocd.bak), eller så lägger kartritaren till dem i OCAD som bakgrundskartor.
cli-summary-ocad-left-out = Utelämnat i OCAD-kartan: { $skipped } objekt och { $dropped } degenererade delar
cli-summary-ocad-symbols = Välj i OCAD Symbol > Load Colors and Symbols From… och ta den officiella symbolsatsen{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF:s skolsats 2019
       *[other] {" "}ISSprOM 2019
    }; den ersätter platshållarsymbolerna efter nummer.
cli-summary-ocad-failed = OCAD-exporten misslyckades: { $error }
cli-summary-pdf-without-credits = PDF utan källhänvisningar: ODbL och dl-de/by-2-0 kräver att du anger källorna när du lämnar kartan vidare; se ATTRIBUTION.txt
cli-summary-wrote-pdf = Skrev kartan { $path } (utan kandidater)
cli-summary-pdf-failed = PDF-exporten misslyckades: { $error }
cli-summary-wrote-pdf-candidates = Skrev kontrollutskriften { $path } (kartan med alla kandidater, lägesriktigt)
cli-summary-pdf-candidates-failed = Exporten av kontrollutskriften misslyckades: { $error }
cli-summary-training-failed = Exporten av träningskartan misslyckades: { $error }
cli-summary-kept-training = Behöll den befintliga träningskartan { $path }, så att en körning inte skriver över en som har skrivits ut eller lämnats vidare; ta bort den eller sätt force för att förnya den

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = ingen hemmapp på det här systemet; använd en absolut sökväg
mcp-error-relative-path = { $path } är en relativ sökväg; använd en absolut sökväg eller en som börjar med ~
mcp-error-home-folder = { $path } är själva hemmappen; använd en mapp i den, t.ex. ~/maps/school
mcp-error-no-project = inget freemapper-projekt i { $path }; använd först create_project
mcp-error-unknown-kind = okänd sort { $kind }; använd "schoolyard", "sprint" eller "forest"
mcp-error-reading = läser { $path }: { $error }
mcp-error-one-area-source = ange exakt en av osm_id (från find_place) eller geojson
mcp-error-page-open = områdesvalssidan för det här projektet är fortfarande öppen; be användaren spara området och trycka på Klar på sidan, starta sedan pipelinen
mcp-error-no-job = inget jobb { $job }; start_pipeline och permits_for_area returnerar jobb-id:n för den här sessionen
mcp-error-job-running = { $job } körs fortfarande för { $path }; en pipeline körs åt gången eftersom projekten delar nedladdningscachen. Starta den här när project_status rapporterar finished eller failed
mcp-error-page-start = kan inte starta sidan: { $error }
mcp-created-project = Skapade projektet { $path } för { $area } ({ $kind }, 1:{ $scale }). Nästa: estimate_download, sedan start_pipeline.
mcp-created-project-given-area = Skapade projektet { $path } för det angivna området ({ $kind }, 1:{ $scale }). Nästa: estimate_download, sedan start_pipeline.
mcp-page-already-open = Sidan för den här mappen är redan öppen; den behåller inställningarna den öppnades med.
mcp-page-next = Be användaren välja området på sidan, trycka på Spara och sedan på Klar.
mcp-preview-of = Förhandsvisning av { $layer }
mcp-permits-next = Tillståndsnoteringarna görs. Fråga project_status med det här job_id; när det är finished innehåller permits_md texten i permits.md. För den vidare som en vägledning, inte juridisk rådgivning.
mcp-tool-check-update = Om en nyare freemapper-version finns: dess version, utgivningsdatum, versionsanteckningar och nedladdningsstorlek, eller att den här är aktuell. Installerar ingenting.
mcp-tool-create-project = Skapa en projektmapp för ett kartområde. Ange osm_id (från find_place) eller en GeoJSON-fil. kind är schoolyard (1:1000, standard), sprint (1:4000) eller forest (1:15000).
mcp-param-create-project-dir = Ny projektmapp (absolut sökväg eller börjar med ~).
mcp-param-create-project-geojson = I stället för osm_id: en GeoJSON-fil med kartområdets polygon (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-create-project-osm-id = OSM-objekt för kartområdet från find_place, t.ex. "way/992423119".
mcp-param-create-project-part = Med osm_id: vilken del av ett område med flera separata delar som ska användas (1 är den största).
mcp-param-create-project-scale = Skalnämnare: skolgård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andra värden nekas; för 1:3000 rita i 4000, freemapper-print förstorar vid utskrift.
mcp-param-create-project-title = Kartans titel för randuppgifterna (standard: mappnamnet).
mcp-tool-estimate-download = Vad start_pipeline kommer att ladda ner för ett projekt: rutorna, varje nedladdning med sin storlek i MB, om den finns i cachen och om den ryms på disken, de MB som ännu ska laddas ner, och under disk det lediga utrymmet för cache- och projektvolymerna med meddelanden om det som inte ryms (berätta det för användaren).
mcp-param-estimate-download-dir = Projektmapp (absolut sökväg eller börjar med ~).
mcp-tool-find-place = Sök en skola, park eller annan plats på namn (OpenStreetMap Nominatim). Returnerar upp till 5 träffar, områden först; bara träffar med has_area kan vara kartområde. En tom lista betyder att inget hittades.
mcp-param-find-place-query = Namnet på skolan eller platsen, t.ex. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installera den senaste freemapper-versionen över det här programmet. Anropa det först när personen har samtyckt, med confirm true. Den nya versionen gäller när assistenten startar om freemapper-servern.
mcp-param-install-update-confirm = true när personen har samtyckt till att installera den nyare versionen; utan det installeras ingenting.
mcp-tool-list-layers = Projektets lager: kartan, de sju kandidatlagren, pappersramarna (paper-frame, bara när ramar behölls på områdessidan) och rastermallarna, var och en med sin fil, om den är gjord än och vilket steg som gör den.
mcp-param-list-layers-dir = Projektmapp (absolut sökväg eller börjar med ~).
mcp-tool-open-area-selector = Öppna en sida i webbläsaren där användaren ritar kartområdet (rektangel eller polygon) eller väljer en plats, ser rutorna och nedladdningsstorleken och sparar; sparandet skapar projektmappen eller ändrar dess område. Returnerar URL:en att ge användaren. Sidan körs tills användaren trycker på Klar; ett nytt anrop för samma mapp returnerar samma URL.
mcp-param-open-area-selector-dir = Projektmapp (absolut sökväg eller börjar med ~); skapas när användaren sparar.
mcp-param-open-area-selector-kind = Kartsort för ett nytt projekt: "schoolyard" (standard, 1:1000), "sprint" (1:4000) eller "forest" (1:15000).
mcp-param-open-area-selector-ocad = Körpanelen startar med OCAD-startprojektet ibockat. Standard false.
mcp-param-open-area-selector-offline = Körpanelen startar med "Bara cache, inget nätverk" ibockat. Standard false.
mcp-param-open-area-selector-open-browser = Öppna också sidan i den här datorns standardwebbläsare. Standard true.
mcp-param-open-area-selector-pdf = Körpanelen startar med den utskrivbara PDF:en ibockad. Standard false.
mcp-param-open-area-selector-pdf-candidates = Körpanelen startar med kontrollutskriften (alla kandidater) ibockad. Standard false.
mcp-param-open-area-selector-pdf-scale-bar = Körpanelen startar med skalstocken på PDF:erna ibockad. Standard false.
mcp-param-open-area-selector-pdf-sources = Källrader på PDF:erna som panelen startar med: "auto" (standard), "full" eller "credit-only", som för start_pipeline.
mcp-param-open-area-selector-permits = Körpanelen startar med tillståndsnoteringarna ibockade. Standard false.
mcp-param-open-area-selector-scale = Skalnämnare för ett nytt projekt: skolgård 1000 (standard); sprint 4000 (standard) eller 5000; skog 15000 (standard), 10000 eller 7500. Andra värden nekas.
mcp-param-open-area-selector-title = Kartans titel för ett nytt projekt.
mcp-param-open-area-selector-training = Körpanelen startar med träningskartan ibockad. Standard false.
mcp-tool-preview-layer = En PNG-förhandsvisning (högst 800 px) av ett lager: map (bilden av kartobjekten som den senaste körningen skrev, utan kandidaterna), eller en rastermall: orthophoto, hillshade, slope, relief, ndsm, eller k5 i Berlin.
mcp-param-preview-layer-dir = Projektmapp (absolut sökväg eller börjar med ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" eller "k5" (bara Berlin).
mcp-tool-project-status = Läget för ett jobb av start_pipeline eller permits_for_area: running, finished eller failed, de senaste förloppsraderna, varningar, de skrivna filerna och resultatet eller felet. Ett färdigt permits_for_area-jobb innehåller också texten i permits.md som permits_md.
mcp-param-project-status-job-id = Jobb-id från start_pipeline eller permits_for_area, t.ex. "job-1".
mcp-tool-permits-for-area = Gör ett projekts tillståndsnoteringar: kartområdets förvaltningsenheter, de skyddade områdena i det och vem som förvaltar marken där, med myndigheterna att fråga; en vägledning, inte juridisk rådgivning. Skriver permits.md och permits.json i mappen. Körs i bakgrunden och returnerar ett job_id; fråga project_status, vars svar innehåller texten i permits.md som permits_md när jobbet är färdigt. Ett jobb körs åt gången.
mcp-param-permits-for-area-dir = Projektmapp (absolut sökväg eller börjar med ~).
mcp-param-permits-for-area-offline = Läs bara de data som en tidigare körning sparade i projektet, utan nätverksåtkomst. Standard false.
mcp-tool-start-pipeline = Ladda ner data, beräkna mallarna och skriv startprojektet (fetch, derive, export). Körs i bakgrunden och returnerar ett job_id; fråga project_status. En pipeline körs åt gången. Den behåller en befintlig karta om inte force är true.
mcp-param-start-pipeline-dir = Projektmapp (absolut sökväg eller börjar med ~).
mcp-param-start-pipeline-force = Ersätt en befintlig kartfil och behåll den gamla som .omap.bak. Standard false.
mcp-param-start-pipeline-ocad = Skriv också ett OCAD-startprojekt till mappen ocad/ (OCAD 12-format, för användare som redigerar i OCAD). Standard false.
mcp-param-start-pipeline-offline = Använd bara nedladdningscachen, utan nätverksåtkomst. Standard false.
mcp-param-start-pipeline-pdf = Skriv också <name>.pdf, kartan färdig att skriva ut (bara kartobjekten, inga kandidater). Standard false.
mcp-param-start-pipeline-pdf-candidates = Skriv också <name>-candidates.pdf, kontrollutskriften: kartan med alla kandidater, lägesriktigt (kontrollera dem i terrängen). Standard false.
mcp-param-start-pipeline-pdf-scale-bar = Rita skalstocken på PDF:erna. Standard false (tyska kartor har oftast ingen; skalnumret stannar i titelblocket).
mcp-param-start-pipeline-pdf-sources = Källrader på PDF:erna: "auto" (standard; hela listan, kortas bara när den tar mer än 5 rader), "full" (kortas aldrig) eller "credit-only" (bara freemappers krediteringsrad; ODbL och dl-de/by-2-0 kräver då att källorna anges när kartan lämnas vidare). ATTRIBUTION.txt listar alltid varje källa.
mcp-param-start-pipeline-permits = Skriv också tillståndsnoteringarna (permits.md och permits.json) efter kartan: förvaltningsenheter, skyddade områden och vem som förvaltar marken. Ett fel där är en varning, inte ett misslyckat jobb. Standard false.
mcp-param-start-pipeline-training = Skriv också träningskartan <name>-training.omap och dess PDF <name>-training.pdf: kartan och alla kandidater, rensade så att de kan skrivas ut som de är; med ocad också ocad/<name>-training.ocd. En befintlig träningskarta behålls om inte force är true. Standard false.
mcp-param-start-pipeline-training-views = Med training: skriv också träningskartan utan det en vy utelämnar, en filuppsättning per post: "no-paths", "brown-only", "no-north-lines", eller en lista som "no:505,506" eller "only:101,102", kombinerad med +. Filerna är <name>-training-<view>.omap och dess PDF.
mcp-instructions = freemapper gör OpenOrienteering Mapper-startprojekt för orienteringskartor (skolgård 1:1000, sprint 1:4000, skog 1:15000) av officiella geodata från { $states } och från OSM. Arbetsgång: find_place (sök skolan eller området på namn; välj en träff med has_area) → create_project (en ny mapp, absolut sökväg, med osm_id) → estimate_download (berätta för användaren hur mycket som laddas ner) → start_pipeline (returnerar ett jobb-id; nedladdningen kan ta minuter) → project_status (fråga var några sekunder tills läget är finished eller failed) → list_layers och preview_layer (visa användaren mallarna). Användaren öppnar sedan kartfilen i OpenOrienteering Mapper. När ingen plats passar eller användaren vill rita området öppnar open_area_selector en sida i webbläsaren där användaren ritar eller väljer området och sparar det (det skapar projektet); fortsätt med estimate_download när användaren säger att det är sparat. Sökvägar måste vara absoluta eller börja med ~. En pipeline körs åt gången. permits_for_area gör ett projekts tillståndsnoteringar (förvaltningsenheter, skyddade områden, vem som förvaltar marken; en vägledning, inte juridisk rådgivning) som ett jobb som start_pipeline. check_update talar om om en nyare freemapper finns; install_update installerar den, bara med confirm: true efter att personen har samtyckt, och assistenten startar sedan om freemapper-servern.

cli-contour-interval-saved = Ekvidistansen sparad i freemapper.toml: { $interval } m; derive drar höjdkurvorna med den
cli-contour-interval-auto = Ekvidistans i freemapper.toml: automatisk (kartsortens; en skogskarta får 2,5 m i flack mark och 10 m i brant mark)
cli-steep-slope-saved = Branthetsgräns sparad i freemapper.toml: { $slope }°; derive letar efter slänter och klippor med den
cli-steep-slope-auto = Branthetsgräns i freemapper.toml: standard (cirka 27°)
cli-face-slope-saved = Väggräns sparad i freemapper.toml: { $slope }°; derive letar efter klippväggsytor med den
cli-face-slope-auto = Väggräns i freemapper.toml: standard (70°)
help-arg-contour-interval = Ekvidistans i meter, t.ex. `2.5`, eller `auto` för kartsortens (en skogskarta: 5 m, 2,5 m där det är flackt, 10 m där det är brant). Sparas i freemapper.toml; derive drar höjdkurvorna med den
help-arg-steep-slope = Lutningsvinkel i grader från vilken terrängen räknas som brant när slänter och klippor eftersöks, t.ex. `55`, eller `auto` för standarden 27°. Höj den (55 till 65) i klipplabyrinter och raviner, där nästan varje sluttning är brant och slänter och klippor flyter ihop till en fläck. Sparas i freemapper.toml; derive letar efter slänter och klippor med den
help-arg-face-slope = Lutningsvinkel i grader från vilken terrängen är en klippvägg (svart yta, symbol 206), t.ex. `75`, eller `auto` för standarden 70°. 65 ger fler och större ytor, även på branta sluttningar; 75 lämnar bara riktiga väggar. Sparas i freemapper.toml; derive letar efter klippväggar med den
cli-content-saved = Innehållsinställningen sparad i freemapper.toml: { $summary }
cli-content-invalid = Kan inte läsa innehållsinställningen: { $detail }
help-arg-content = Vad som hamnar direkt på kartan: `safe` låter varje objekt stanna där bygget lägger det, `balanced` (standard) lägger objekten ur katastret och registren på kartan, `full` lägger allt på kartan. Valet sparas i freemapper.toml
help-arg-content-set = Skicka en grupp objekt till `map`, `candidates` eller `off`, som `GROUP=PLACE`, t.ex. `walls=map`; kan upprepas. Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Sparas i freemapper.toml
mcp-param-create-project-content = Vad som hamnar direkt på kartan: "safe", "balanced" (standard) eller "full". Sparas i projektet. balanced lägger objekten ur katastret och registren på kartan, full lägger allt på den.
mcp-param-create-project-content-set = Grupper som skickas någon annanstans än profilen säger, som "group=place" med place map, candidates eller off, t.ex. "walls=map". Grupper: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Ekvidistans i meter, t.ex. 2.5; 0 går tillbaka till det automatiska valet (kartsortens; en skogskarta får 2,5 m i flack mark och 10 m i brant mark). Sparas i projektet; nästa derive drar höjdkurvorna med den.
mcp-param-create-project-steep-slope = Lutningsvinkel i grader från vilken terrängen räknas som brant när slänter och klippor eftersöks, t.ex. 55; 0 går tillbaka till standarden 27°. Höj den (55 till 65) i klipplabyrinter och raviner, där nästan varje sluttning är brant och slänter och klippor flyter ihop till en fläck. Sparas i projektet; nästa derive letar efter slänter och klippor med den.
mcp-param-create-project-face-slope = Lutningsvinkel i grader från vilken terrängen är en klippvägg (svart yta, symbol 206), t.ex. 75; 0 går tillbaka till standarden 70°. 65 ger fler och större ytor, även på branta sluttningar; 75 lämnar bara riktiga väggar. Sparas i projektet; nästa derive letar efter klippväggar med den.
mcp-param-start-pipeline-content = Ändra vad som hamnar direkt på kartan före körningen: "safe", "balanced" eller "full". Sparas i projektet. Standard: projektets inställning, annars balanced.
mcp-param-start-pipeline-content-set = Grupper som skickas någon annanstans än profilen säger, som "group=place" med place map, candidates eller off, t.ex. "fences=candidates". Sparas i projektet.
mcp-param-start-pipeline-contour-interval = Ekvidistans i meter, t.ex. 2.5; 0 går tillbaka till det automatiska valet (kartsortens; en skogskarta får 2,5 m i flack mark och 10 m i brant mark). Sparas i projektet; nästa derive drar höjdkurvorna med den.
mcp-param-start-pipeline-steep-slope = Lutningsvinkel i grader från vilken terrängen räknas som brant när slänter och klippor eftersöks, t.ex. 55; 0 går tillbaka till standarden 27°. Höj den (55 till 65) i klipplabyrinter och raviner, där nästan varje sluttning är brant och slänter och klippor flyter ihop till en fläck. Sparas i projektet; nästa derive letar efter slänter och klippor med den.
mcp-param-start-pipeline-face-slope = Lutningsvinkel i grader från vilken terrängen är en klippvägg (svart yta, symbol 206), t.ex. 75; 0 går tillbaka till standarden 70°. 65 ger fler och större ytor, även på branta sluttningar; 75 lämnar bara riktiga väggar. Sparas i projektet; nästa derive letar efter klippväggar med den.
mcp-error-content = Kan inte läsa innehållsinställningen: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Skriv ett projekts tillståndsnoteringar (permits.md och permits.json): de förvaltningsenheter, skyddade områden och huvudmän som kartområdet berör, och vem det oftast innebär att du bör fråga. Den frågar BfN och BKG och, där kartområdets delstat har egna öppna tjänster, dem via värdarna i dess adapter (lista i handboken, kapitel 12); i Nordrhein-Westfalen laddar den ner cirka 32 MB för detta första gången, i Brandenburg cirka 4 MB, senare bara nyare filer. Den läser det OSM-extrakt som redan finns i cachen; --offline använder det som hämtades tidigare
help-permits-dir = Projektmapp
help-permits-offline = Använd bara de data som hämtades tidigare, utan nätverksåtkomst
help-run-permits = Skriv också tillståndsnoteringarna (permits.md och permits.json) efter kartan; frågar BfN, BKG och delstatens öppna tjänster (handboken, kapitel 12). Ett fel där är en varning och kartan blir kvar
cli-wrote-permits = Skrev tillståndsnoteringarna { $path }
cli-summary-wrote-permits = Skrev tillståndsnoteringarna { $path }
cli-permits-failed = tillståndsnoteringarna kunde inte skrivas: { $error }
cli-permits-source-left-out = tillståndsnoteringarna utelämnar { $source }: { $reason }
cli-permits-not-read = den lästes inte
error-permits-stored = { $path }: { $message }; hämta tillståndsnoteringarna på nytt online
error-permits-nothing-read = ingen källa till tillståndsnoteringarna kunde läsas ({ $missing }); hämta dem först online
error-permits-missing-not-read = { $source }: inte läst

permits-title = Tillståndsnoteringar
permits-disclaimer = En vägledning, inte juridisk rådgivning. De behöriga myndigheterna beslutar. För datadatum och noggrannhet, se tabellen under "Källor och datadatum"; områden kan saknas eller vara föråldrade (BfN: sammanförs en gång per år, "Nicht für Planungszwecke geeignet.", det vill säga inte lämpliga för planering).
permits-disclaimer-regional = En vägledning, inte juridisk rådgivning. De behöriga myndigheterna beslutar. För datadatum och noggrannhet, se tabellen under "Källor och datadatum"; områden kan saknas eller vara föråldrade.
permits-bfn-merge = BfN sammanför delstaternas data en gång per år och markerar dem som inte lämpliga för planering. Delstatens myndigheter beslutar.
permits-created = Gjord den { $date } för kartområdet till en { $kind ->
        [schoolyard] skolgårdskarta
        [forest] skogskarta
       *[other] sprintkarta
    } ({ $area } ha).
permits-section-admin = Förvaltningsenheter
permits-section-protected = Skyddade områden (BfN)
permits-section-osm-protected = Fler skyddade områden från OpenStreetMap
permits-section-operators = Huvudmän och ägare enligt OpenStreetMap
permits-section-hints = Råd
permits-section-sources = Källor och datadatum
permits-none-admin = VG250 anger ingen förvaltningsenhet för kartområdet.
permits-none-protected = Enligt BfN ligger inget skyddat område i kartområdet.
permits-none-osm-protected = OpenStreetMap anger inget skyddat område i kartområdet som BfN inte redan listar.
permits-none-operators = OpenStreetMap anger ingen huvudman och ingen ägare för kartområdet.
permits-none-hints = Inga råd.
permits-section-not-read = Uppgifterna från { $source } lästes inte, så det här avsnittet är tomt, även om det kan finnas poster.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Förvaltningsområden 1:250 000 (VG250) från BKG
        [bfn] Skyddade områden i Tyskland från BfN
       *[osm] OpenStreetMap, från det extrakt freemapper har i sin cache
    }
permits-level = { $level ->
        [land] Delstat
        [kreis] Distrikt (Kreis)
        [verwaltungsgemeinschaft] Amt eller Verwaltungsgemeinschaft
       *[gemeinde] Kommun (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Berörs i kanten (mindre än 0,5 % av kartområdet):
permits-at-edge-generalised = VG250 är generaliserad till skalan 1:250 000. En enhet som bara berör kanten kan vara en följd av det.
permits-kind = { $kind ->
        [nationalpark] Nationalpark
        [nationales-naturmonument] Nationellt naturmonument
        [naturschutzgebiet] Naturreservat (Naturschutzgebiet)
        [ffh] FFH-område (Natura 2000)
        [vogelschutzgebiet] Fågelskyddsområde (Natura 2000)
        [biosphaerenreservat] Biosfärreservat
        [biosphaerenreservat-zone] Zon i ett biosfärreservat
        [landschaftsschutzgebiet] Landskapsskyddsområde (Landschaftsschutzgebiet)
       *[naturpark] Naturpark
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Utsett den { $date }
permits-protected-title = Rättslig grund: { $title }
permits-protected-link = Lagtext: { $url }
permits-protected-site-link = Områdessida (Natura 2000): { $url }
permits-protected-year = Dataår hos BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = utan namn
permits-osm-operator = Huvudman enligt OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap-områden som omsluter hela kartområdet utan något hörn inuti det kan saknas här, och det kan skyddade områden som i OSM är kartlagda som en relation med `type=boundary`. Delarna från BfN och VG250 påverkas inte.
permits-osm-enclosing-gap-short = OpenStreetMap-områden som omsluter hela kartområdet utan att ha ett hörn inuti det kan saknas här.
permits-operator-line = { $name } ({ $roles ->
        [operator] huvudman
        [owner] ägare
       *[both] huvudman och ägare
    }): { $objects }
permits-column-source = Källa
permits-column-state = Läge
permits-column-credit = Källhänvisning och licens
permits-state-read = läst, hämtad den { $date }
permits-state-read-osm = läst, OSM-extrakt från { $date }
permits-state-unreachable = inte nåbar ({ $reason })
permits-state-not-read = inte läst
permits-no-credit = används inte
permits-hint-city-state = Kartområdet ligger i Berlin eller Hamburg. Där är oftast Bezirksamt (stadsdelskontoret) eller en delstatsmyndighet ansvarig. De här noteringarna avgör ännu inte vilken Bezirk det gäller.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Skyddade områden rör den lägre naturvårdsmyndigheten (untere Naturschutzbehörde). Här är det staden { $kreis } själv.
        [no] Skyddade områden rör den lägre naturvårdsmyndigheten (untere Naturschutzbehörde), oftast { $kreis }.
       *[unknown] Skyddade områden rör den behöriga lägre naturvårdsmyndigheten (untere Naturschutzbehörde), oftast Landkreis eller den kreisfria staden.
    }
permits-hint-national-park = En nationalpark har egna regler. Fråga nationalparksförvaltningen i god tid.
permits-hint-nsg-paths = I ett naturreservat måste du oftast hålla dig på stigarna. För ett lopp utanför stigarna, fråga naturvårdsmyndigheten.
permits-hint-natura2000 = Ett FFH-område eller ett fågelskyddsområde (Natura 2000) berör kartområdet. För ett arrangemang kan naturvårdsmyndigheten kräva en bedömning av dess effekter.
permits-hint-protected-areas-pl = I kartområdet ligger skyddade områden eller skyddade naturobjekt. För en nationalpark, en landskapspark eller ett naturreservat är parkens eller reservatets förvaltning vanligen kontaktpunkten; för de övriga områdena och objekten kan den regionala direktionen för miljöskydd (RDOŚ) i vojvodskapet oftast säga vem som är ansvarig. Fråga i god tid när ett evenemang planeras där.
permits-hint-order-office = { $amt ->
        [none] För gator, torg och annan offentlig mark är oftast Ordnungsamt (ordningskontoret) i kommunen { $gemeinde } ansvarigt.
       *[other] För gator, torg och annan offentlig mark är oftast Ordnungsamt (ordningskontoret) ansvarigt: { $amt }, för kommunen { $gemeinde }.
    }
permits-hint-park-operator = Huvudman för en park eller trädgård i kartområdet enligt OpenStreetMap: { $operator }. Fråga dem innan du använder den.
permits-hint-school-operator = Huvudman för skolan eller institutionen enligt OpenStreetMap: { $operator }. För dess område, fråga skolans rektor eller den här organisationen.
permits-section-districts = Bezirke (stadsdelar, { $source })
permits-section-green-spaces = Grönområden ({ $source })
permits-section-owners = Mark som ägs av delstaten och av Bund ({ $source })
permits-section-monuments = Kulturminnen ({ $source })
permits-section-protected-objects = Naturminnen och andra skyddade objekt ({ $source })
permits-none-districts = Enligt { $source } ligger kartområdet i ingen Bezirk.
permits-none-green-spaces = Enligt { $source } ligger inget offentligt grönområde i kartområdet.
permits-none-owners = Enligt { $source } ligger ingen fastighet som ägs av delstaten eller av Bund i kartområdet.
permits-none-monuments = Enligt { $source } ligger inget kulturminne i kartområdet.
permits-none-protected-objects = Enligt { $source } ligger inget av de skyddade objekt den innehåller i kartområdet.
permits-none-sampled = I de punkter som frågades anger { $source } inget här.
permits-gaps = Uppgifterna från { $source } är inte kompletta: { $gaps }.
permits-sampled = { $source } svarar bara för enskilda punkter. freemapper frågade i ett rutnät av punkter över kartan: andelarna är uppskattningar, och det som ligger mellan punkterna kan saknas.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = ändamål: { $dedication }
permits-playgrounds-none = Ingen offentlig lekplats ligger i kartområdet.
permits-playgrounds-one = En offentlig lekplats ligger i kartområdet. Den listas inte särskilt.
permits-playgrounds-many = { $count } offentliga lekplatser ligger i kartområdet. De listas inte en och en.
permits-owner-other = Andra offentliga ägare
permits-owners-rest = För resten av området anger uppgifterna ingen ägare.
permits-monuments-total = Kulturminnen helt eller delvis i kartområdet: { $count }.{ $links ->
        [yes] {" "}Länkarna leder till deras poster i kulturminneslistan.
       *[no] {""}
    }
permits-monuments-links-left = { $count } poster till räknas bara här, utan länk.
permits-object-kind = { $kind ->
        [natural-monument] Naturminne (Naturdenkmal)
        [protected-landscape-part] Skyddad landskapsdel (geschützter Landschaftsbestandteil)
        [biotope] Lagskyddad biotop (geschütztes Biotop)
        [national-park] Nationalpark (park narodowy)
        [park-buffer-zone] Buffertzon kring en park (otulina)
        [landscape-park] Landskapspark (park krajobrazowy)
        [nature-reserve] Naturreservat (rezerwat przyrody)
        [protected-landscape-area] Skyddat landskapsområde (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000-område för livsmiljöer (SOO)
        [natura2000-birds] Natura 2000-område för fåglar (OSO)
        [ecological-site] Ekologiskt område (użytek ekologiczny)
        [landscape-complex] Natur- och landskapskomplex (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentationsplats (stanowisko dokumentacyjne)
        [natural-monument-pl] Naturminne (pomnik przyrody)
        [national-park-cz] Nationalpark (národní park, NP)
        [protected-landscape-area-cz] Skyddat landskapsområde (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Nationellt naturreservat (národní přírodní rezervace, NPR)
        [national-nature-monument] Nationellt naturminne (národní přírodní památka, NPP)
        [nature-reserve-cz] Naturreservat (přírodní rezervace, PR)
        [nature-monument-cz] Naturminne (přírodní památka, PP)
        [protective-zone-cz] Skyddszon (ochranné pásmo)
        [nature-zone-cz] Zon för naturskydd (zóna ochrany přírody)
        [natura2000-habitats-cz] Natura 2000-område för livsmiljöer (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Natura 2000-område för fåglar (ptačí oblast, PO)
        [memorial-tree] Minnesträd (památný strom)
       *[special-rules] Område med särskilda regler
    }
permits-object-count = { $count } objekt
permits-object-citations = Rättsliga grunder:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] För gator, torg och grönområden i Bezirk { $district } är oftast Bezirksamt { $district } ansvarigt, där Fachamt Management des öffentlichen Raumes (kontoret för det offentliga rummet).
       *[berlin] För gator, torg och grönområden i Bezirk { $district } är oftast Ordnungsamt (ordningskontoret) och Straßen- und Grünflächenamt (väg- och parkkontoret) vid Bezirksamt { $district } ansvariga.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] För kulturminnena i kartområdet är Denkmalschutzamt Hamburg (kulturminnesmyndigheten) ansvarig.
       *[berlin] För kulturminnena i Bezirk { $district } är den lägre kulturminnesmyndigheten (untere Denkmalschutzbehörde) vid Bezirksamt { $district } ansvarig.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Om listade kulturminnen ligger i kartområdet kan uppgifterna här inte säga säkert.
       *[yes] Det finns listade kulturminnen i kartområdet.
    } Om ett lopp berör dem, till exempel med en kontroll vid ett kulturminne eller en sträcka genom ett skyddat område, avgör den lägre kulturminnesmyndigheten (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Här är det staden { $kreis } själv.
        [no] , oftast { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; i vissa delstater är det kommunen { $gemeinde }.
        }
       *[unknown] {" "}(oftast Landkreis eller den kreisfria staden, i vissa delstater kommunen).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Naturminnen, skyddade landskapsdelar och skyddade biotoper på kartan är en fråga för den lägre naturvårdsmyndigheten (untere Naturschutzbehörde). Här är det staden { $kreis } själv.
        [no] Naturminnen, skyddade landskapsdelar och skyddade biotoper på kartan är en fråga för den lägre naturvårdsmyndigheten (untere Naturschutzbehörde), oftast { $kreis }.
       *[unknown] Naturminnen, skyddade landskapsdelar och skyddade biotoper på kartan är en fråga för den lägre naturvårdsmyndigheten (untere Naturschutzbehörde), oftast Landkreis eller den kreisfria staden.
    }
permits-hint-biotope = Lagskyddade biotoper ligger i kartområdet{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Allt som kan förstöra dem eller skada dem avsevärt är förbjudet (§ 30 BNatSchG). Lägg kontroller och sträckor utanför dem, eller fråga den lägre naturvårdsmyndigheten (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] För naturminnen och skyddade landskapsdelar i Bezirk { $district } är oftast Bezirksamt { $district } ansvarigt.
       *[berlin] För naturminnen och skyddade landskapsdelar i Bezirk { $district } är den lägre naturvårdsmyndigheten (untere Naturschutzbehörde) vid Bezirksamt { $district } ansvarig.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Enligt delstatens öppna data tillhör { $share } av kartområdet andra offentliga ägare.
       *[no] Enligt delstatens öppna data tillhör { $share } av kartområdet ägaren "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Ägarna till resten av området går inte att utläsa ur öppna data. Med ett berättigat intresse (till exempel som arrangör) kan du be fastighetsregistret (Grundbuchamt) om ett utdrag; för offentlig mark hjälper det behöriga Ordnungsamt eller parkförvaltningen.
       *[no] Ägarna till de här områdena går inte att utläsa ur öppna data. Med ett berättigat intresse (till exempel som arrangör) kan du be fastighetsregistret (Grundbuchamt) om ett utdrag; för offentlig mark hjälper det behöriga Ordnungsamt eller parkförvaltningen.
    }
permits-section-forest-units = Skogsförvaltning ({ $source })
permits-section-state-forest = Delstatsskog ({ $source })
permits-none-forest-units = Enligt { $source } ligger kartområdet inom ingen skogsmyndighets område.
permits-section-admin-units = Administrativa enheter ({ $source })
permits-none-admin-units = Enligt { $source } ligger kartområdet inte i någon administrativ enhet i registret.
permits-section-register-monuments = Minnesmärken i det nationella registret ({ $source })
permits-none-register-monuments = Enligt { $source } finns inget registrerat minnesmärke i rektangeln runt kartområdet.
permits-register-monuments-total = Registrerade minnesmärken i rektangeln runt kartområdet: { $count }. Registret anger ingen position, så de här anmärkningarna kan inte säga var de ligger; det måste kontrolleras på kartan.
permits-register-monuments-left = { $count } till är bara räknade här, utan namn.
permits-section-register-archaeology = Arkeologiska minnesmärken i det nationella registret ({ $source })
permits-none-register-archaeology = Enligt { $source } finns inget registrerat arkeologiskt minnesmärke i rektangeln runt kartområdet.
permits-register-archaeology-total = Registrerade arkeologiska minnesmärken i rektangeln runt kartområdet: { $count }. Registret anger ingen position, så de här anmärkningarna kan inte säga var de ligger; det måste kontrolleras på kartan.
permits-register-archaeology-left = { $count } till är bara räknade här, utan namn.
permits-section-register-history = Historiens minnesmärken ({ $source })
permits-none-register-history = Enligt { $source } ligger inget historiens minnesmärke i rektangeln runt kartområdet.
permits-register-history-in-area = Historiens minnesmärken i kartområdet: { $count }.
permits-register-history-nearby = Fler historiens minnesmärken i rektangeln runt kartområdet, men inte i kartområdet: { $count }.
permits-register-history-left = { $count } till räknas här bara, utan namn.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Enligt { $source } ligger ingen delstatsskog i kartområdet.
permits-forest-units-reach = En skogsmyndighet ansvarar för skogen inom sitt område. Dess område kan också omfatta städer och åkermark.
permits-state-forest-office = ansvarig: { $office }
permits-hint-forest-office = { $units ->
        [none] Skogsmyndighet som ansvarar för skogen i kartområdet: { $office } ({ $administration }). Fråga dit i god tid när du planerar ett arrangemang i skogen.
       *[other] Skogsmyndighet som ansvarar för skogen i kartområdet: { $office } ({ $administration }), { $units }. Fråga dit i god tid när du planerar ett arrangemang i skogen.
    }
permits-hint-forest-owner = { $office ->
        [known] Ett arrangemang i skogen kräver skogsägarnas samtycke. Skogsmyndigheten kan oftast säga vem som äger skogen. I flera delstater måste ett organiserat arrangemang i skogen också anmälas till skogsmyndigheten eller tillåtas av den.
       *[unknown] Ett arrangemang i skogen kräver skogsägarnas samtycke. I flera delstater måste ett organiserat arrangemang i skogen också anmälas till skogsmyndigheten eller tillåtas av den. De här noteringarna nämner inte den skogsmyndighet som ansvarar här; delstatens skogsförvaltning eller den lägre skogsmyndigheten kan säga det.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Ett evenemang i skogen kräver vanligen skogsägarens samtycke. Skogsdistriktet (Nadleśnictwo) som nämns ovan kan oftast säga vem som sköter skogen. Skog som inte tillhör Statsskogarna, till exempel privatskog, finns inte i uppgifterna i dessa anteckningar. Skog som inte tillhör staten, till exempel privat eller kommunal skog, står under tillsyn av starosten (starosta) när det gäller skogsbruket (skogslagen, art. 5 ust. 1 pkt 2); fråga där hos starostwo powiatowe.
       *[unknown] Ett evenemang i skogen kräver vanligen skogsägarens samtycke. Dessa anteckningar nämner inget skogsdistrikt i Statsskogarna för kartområdet; uppgifterna omfattar bara Statsskogarna, så skogen kan tillhöra någon annan, till exempel en privat ägare. Vem ägaren är måste utredas separat. Skog som inte tillhör staten, till exempel privat eller kommunal skog, står under tillsyn av starosten (starosta) när det gäller skogsbruket (skogslagen, art. 5 ust. 1 pkt 2); fråga där hos starostwo powiatowe.
    }
permits-hint-monument-register-pl = Minnesmärken i det nationella registret ligger i rektangeln runt kartområdet. Deras positioner finns inte i data, så kontrollera på kartan var de är. Enligt art. 36 ust. 1 i lagen om skydd av minnesmärken (ustawa o ochronie zabytków) kräver konserverings-, restaurerings- och byggarbete på ett registrerat minnesmärke tillstånd av voivodskapets minnesvårdare (Wojewódzki Konserwator Zabytków). Sedan den 3 juni 2026 kräver byggarbete i ett minnesmärkes omgivning, ändrad användning, anordningar eller skyltar som sätts upp på det och andra åtgärder som kan ändra dess substans eller utseende bara en anmälan till vårdaren (art. 36aa); vårdaren kan invända inom 60 dagar och då kräva tillstånd. Lagen nämner inte ett lopp eller en kontroll som sådana; fråga vårdarens kontor om ett arrangemang berör ett minnesmärke eller dess omgivning.
permits-hint-archaeology-register-pl = Arkeologiska minnesmärken i det nationella registret ligger i rektangeln runt kartområdet. Deras positioner finns inte i data, så kontrollera på kartan var de ligger. Enligt art. 36 ust. 1 i lagen om skydd av minnesmärken (ustawa o ochronie zabytków) kräver arkeologiska undersökningar (pkt 5), sökning efter dolda eller övergivna lösa minnesmärken med elektroniska eller tekniska apparater som metalldetektorer (pkt 12) och konserverings-, restaurerings- eller byggarbete på ett registrerat minnesmärke (pkt 1) tillstånd av voivodskapets minnesvårdare (Wojewódzki Konserwator Zabytków). Tillståndet för sökning upphör den 1 januari 2027: art. 36b tillåter då en vuxen person att söka med samtycke från fastighetens ägare och innehavare efter en anmälan i sökregistret via kulturministeriets app, och art. 36c förbjuder sådan sökning bland annat på registrerade minnesmärken och närmare än 10 m från dem. Lagen nämner inte en promenad, ett lopp eller en kontroll som sådan; fråga minnesvårdarens kansli om ett evenemang eller fältarbete berör en registrerad plats.
permits-hint-history-register-pl = Ett historiens minnesmärke (Pomnik historii) ligger i eller nära kartområdet; avsnittet om historiens minnesmärken säger vilket. Republiken Polens president förklarar genom förordning ett fast minnesmärke som är infört i registret, eller en kulturpark av särskilt värde, som Pomnik historii (art. 15 ust. 1 i lagen om skydd av minnesmärken, ustawa o ochronie zabytków). Det står alltså redan under skydd av registret eller kulturparken. För arbeten på ett minnesmärke som är infört i registret måste vojvodskapets minnesvårdare (Wojewódzki Konserwator Zabytków) ge tillstånd (art. 36 ust. 1); i en kulturpark fastställer kommunfullmäktiges beslut förbud och begränsningar (art. 16 ust. 2, art. 17 ust. 1). Lagen nämner inte en promenad, ett lopp eller en kontroll som sådana; fråga minnesvårdarens kontor om ett evenemang eller fältarbete berör minnesmärket.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Kartområdet rymmer ett naturminne (pomnik przyrody), ett ekologiskt område (użytek ekologiczny), ett natur- och landskapskomplex (zespół przyrodniczo-krajobrazowy) eller en dokumentationsplats (stanowisko dokumentacyjne). Kommunfullmäktige (rada gminy) inrättar dem genom ett beslut, och beslutet anger vem som utövar tillsyn över objektet och vilka förbud som gäller (art. 44 i naturvårdslagen). Lagen själv nämner inget tillstånd för ett evenemang här. Be kontoret i den gmina där objektet ligger om beslutet och tillsynen.
       *[other] Kartområdet rymmer ett naturminne (pomnik przyrody), ett ekologiskt område (użytek ekologiczny), ett natur- och landskapskomplex (zespół przyrodniczo-krajobrazowy) eller en dokumentationsplats (stanowisko dokumentacyjne). Kommunfullmäktige (rada gminy) inrättar dem genom ett beslut, och beslutet anger vem som utövar tillsyn över objektet och vilka förbud som gäller (art. 44 i naturvårdslagen). Lagen själv nämner inget tillstånd för ett evenemang här. Be { $count ->
        [one] kontoret i gminan ({ $gmina }), där objektet ligger
       *[other] kontoren i gminorna ({ $gmina }), vart och ett för objekten inom sitt område
    }, om beslutet och tillsynen.
    }
permits-hint-gmina-roads-pl = Ett evenemang som hindrar trafiken eller använder en gata på ett särskilt sätt kräver tillstånd enligt art. 65 i trafiklagen, ansökt minst 30 dagar i förväg. Tillståndet ges av den myndighet som styr trafiken på vägen: starosta för powiat- och gminavägar, borgmästaren i en stad med powiatställning, voivodskapets marskalk eller GDDKiA för övriga vägar (art. 10 och 65b). Väghållare för en gminaväg är wójt, burmistrz eller prezydent miasta { $count ->
        [one] i gminan ({ $gmina });
       *[other] i var och en av gminorna ({ $gmina });
    } väghållaren avgör om vägområdet får tas i anspråk (art. 19 och 40 i lagen om allmänna vägar). Dessa anteckningar visar inte vägarnas kategori.
permits-hint-gmina-land-pl = Om mark i kartområdet tillhör gminan, till exempel en kommunal park, ett torg, en skolgård eller en skog, förvaltas den av wójt, burmistrz eller prezydent miasta, som företräder gminan (art. 30 och 31 i lagen om gminans självstyre). Be { $count ->
        [one] kontoret i gminan ({ $gmina }) om samtycke
       *[other] kontoren i gminorna ({ $gmina }) om samtycke
    } innan du använder den. Dessa anteckningar visar inte vem som äger marken.
permits-hint-protected-areas-cz = Skyddade områden eller föremål för naturvård ligger i kartområdet. Enligt lagen om natur- och landskapsskydd (lag nr 114/1992 Sb.) är organiserade idrotts-, turist- och andra offentliga evenemang i en nationalpark förbjudna utanför de platser som naturvårdsmyndigheten har anvisat (§ 16 odst. 1 písm. q), och i en nationalparks skyddszon kräver massevenemang av detta slag utanför dessa platser och utanför bebyggda områden myndighetens samtycke (§ 37 odst. 3 písm. d). I ett nationellt naturreservat är det förbjudet att gå eller köra utanför de stigar som markerats med myndighetens samtycke (§ 29 písm. d). Naturvårdsmyndigheter är bland andra nationalparkens förvaltning, naturvårdsverket (AOPK ČR), regionkontoret (krajský úřad) och kontoret för en kommun med utökade befogenheter (§ 75 odst. 1). Andra skyddade områden och föremål har egna skyddsvillkor. Fråga myndigheten i god tid om du planerar ett evenemang där.
permits-hint-monuments-cz = Kulturminnen eller kulturminnesområden i den statliga förteckningen ligger i kartområdet. Enligt lagen om statlig kulturminnesvård (lag nr 20/1987 Sb., § 14 odst. 1 och 2) måste ägaren till ett kulturminne i förväg begära ett beslut eller ett bindande yttrande före underhåll, reparation, ombyggnad eller annan ändring av kulturminnet eller dess omgivning. Detsamma gäller ägaren till en fastighet i ett minnesreservat, en minneszon eller ett kulturminnes skyddszon för en byggnad, ändring av en byggnad, markarbeten, en anordning eller en skylt, rivning av en anläggning, skötsel av träd eller underhållsarbeten. Kontoret för kommunen med utökade befogenheter beslutar{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, för ett nationellt kulturminne regionkontoret. Lagen nämner inte en promenad, ett lopp eller en kontroll som sådana; fråga det kontoret om ett evenemang berör ett kulturminne eller sätter upp anordningar eller skyltar på det.
permits-hint-forest-owner-cz = Ett stort idrotts-, kultur- eller annat samhällsevenemang på mark som är avsedd för skogens funktioner får hållas bara med skriftligt samtycke från ägaren till skogen där det äger rum (skogslagen, lag nr 289/1995 Sb., § 20 odst. 5). Arrangören måste dessutom skriftligen anmäla evenemanget till kontoret för kommunen med utökade befogenheter{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, eller till regionkontoret om evenemanget sträcker sig utanför en sådan kommuns distrikt, senast 30 dagar före evenemanget. Anmälan anger datum och plats, hur evenemanget bemannas, det förväntade antalet deltagare och ägarnas skriftliga samtycken (samma paragraf). Dessa anteckningar visar inte vem som äger skogen; fråga skogsägarna i god tid.
permits-hint-roads-cz = Ett evenemang som använder en väg på annat sätt än vanligt, till exempel ett idrotts-, kultur- eller liknande evenemang som kan äventyra trafiksäkerheten eller trafikflödet, kräver tillstånd av den behöriga vägmyndigheten. Det ges med förhandssamtycke från vägens ägare och, där trafiksäkerheten eller trafikflödet kan påverkas, från Tjeckiens polis (vägagen, lag nr 13/1997 Sb., § 25 odst. 1 och odst. 6 písm. d). Myndigheten är kommunkontoret för lokala vägar, eller kontoret för kommunen med utökade befogenheter där det beslutar, och kontoret för kommunen med utökade befogenheter för vägar av andra och tredje klass (§ 40 odst. 4 och 5). Kommuner i kartområdet: { $gmina }. Dessa anteckningar visar inte vägarnas klass.
permits-section-water-protection = Vattenskyddsområden ({ $source })
permits-section-forest-functions = Skogsfunktioner och skyddsskog ({ $source })
permits-none-water-protection = Enligt { $source } ligger inget vattenskyddsområde i kartområdet.
permits-none-forest-functions = Enligt { $source } ligger ingen rekreationsskog, ingen skyddsskog, ingen naturskog och inget stängt skogsområde i kartområdet.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreationsskog (Erholungswald)
        [protected] Skyddsskog (Bannwald eller Schutzwald)
        [natural] Naturskog eller strikt reservat (Naturwald, Totalreservat)
       *[closed] Stängt skogsområde
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } i vattenskyddsområdet "{ $name }" ligger i kartområdet. I zon I och II är arrangemang oftast uteslutna. Fråga den lägre vattenmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } i vattenskyddsområdet "{ $name }" ligger i kartområdet. Arrangemang där är oftast möjliga; den lägre vattenmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) kan säga vad föreskriften tillåter.
       *[unknown] Vattenskyddsområdet "{ $name }" ligger i kartområdet. Uppgifterna säger inte vilken zon det är. I zon I och II är arrangemang oftast uteslutna; fråga den lägre vattenmyndigheten (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Rekreationsskog (Erholungswald) ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. För arrangemang där, fråga skogsmyndigheten.
        [protected] Skyddsskog (Bannwald eller Schutzwald) ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Fråga skogsmyndigheten innan du planerar ett arrangemang där.
        [natural] Naturskog eller ett strikt reservat ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Att lämna stigarna är ofta förbjudet där; lägg inga kontroller i den utan att fråga skogsmyndigheten.
       *[closed] Ett skogsområde som inte får beträdas ligger i kartområdet{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Håll det utanför banan.
    }
permits-section-roads = Klassade vägar enligt OpenStreetMap
permits-none-roads = Enligt OpenStreetMap går ingen Autobahn och ingen Bundes-, Landes-, Staats- eller Kreisstraße genom kartområdet.
permits-road-line = { $class ->
        [autobahn] Autobahn (motorväg)
        [bundesstrasse] Bundesstraße (riksväg)
        [landesstrasse] Landesstraße eller Staatsstraße (delstatsväg)
       *[kreisstrasse] Kreisstraße (distriktsväg)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } går genom kartområdet. En bana får korsa den bara på broar eller genom tunnlar.
        [bundesstrasse] Bundesstraße { $refs } går genom kartområdet. En avstängning eller en korsning med funktionärer beslutas av trafikmyndigheten (Straßenverkehrsbehörde); vägen underhålls oftast av Bund genom delstatens vägförvaltning, i större städer ofta av staden.
        [landesstrasse] Landes- eller Staatsstraße { $refs } går genom kartområdet. En avstängning eller en korsning med funktionärer beslutas av trafikmyndigheten (Straßenverkehrsbehörde); vägen underhålls oftast av delstaten, i större städer ofta av staden.
       *[kreisstrasse] Kreisstraße { $refs } går genom kartområdet. En avstängning eller en korsning med funktionärer beslutas av trafikmyndigheten (Straßenverkehrsbehörde); vägen underhålls oftast av Kreis, i större städer ofta av staden.
    }
permits-hint-state-forest = { $office ->
        [none] Enligt delstatens öppna data är { $share } av kartområdet delstatsskog som ägs av "{ $owner }".
       *[other] Enligt delstatens öppna data är { $share } av kartområdet delstatsskog som ägs av "{ $owner }". Ansvarig: { $office }.
    }
