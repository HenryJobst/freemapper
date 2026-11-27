# Programmateksten van freemapper, Nederlands (ADR 0092). Engels is de referentie: elke andere taal
# heeft dezelfde bericht-ids en dezelfde variabelen. Teksten van andere programma's (fouten van het
# besturingssysteem, parserberichten) komen in variabelen binnen en blijven zoals ze zijn.

## The --lang switch

cli-lang-unknown = { $value } is geen taal waarvoor freemapper teksten heeft; beschikbaar: { $available }

## init

cli-part-needs-osm = --part kiest een deel van een OSM-gebied en heeft --osm nodig; zoek bij --place eerst de id op en gebruik --osm <id> --part <n>
cli-reading-file = { $path } wordt gelezen
cli-no-area-for-place = geen gebied gevonden voor "{ $query }". Als de plaats een gebied van meerdere afzonderlijke delen is, kies er een met --osm <id> --part <n>, of gebruik --geojson
cli-using-place = { $name } ({ $id }) wordt gebruikt
cli-created-project = Project { $path } aangemaakt

## Cache

cli-no-cache-dir = deze computer heeft geen cachemap; geef --cache-dir mee

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] 1 tegel opgehaald
       *[other] { $count } tegels opgehaald
    }
cli-wrote = { $path } geschreven
cli-kept-map = { $path } behouden (het kan jouw werk bevatten)
cli-map-lacks-templates = De kaart verwijst nog niet naar de beeldtemplates. Voeg ze toe in OOM (Templates > Open template…) of vervang de kaart met --force; de oude blijft dan bewaard als .omap.bak.
cli-map-lacks-buildings = De kaart heeft nog geen ALKIS-gebouwen. Vervang de kaart met --force om ze toe te voegen; de oude blijft dan bewaard als .omap.bak.
cli-map-lacks-canopies = De kaart mist de ALKIS-overkappingen en carports (522), die nu kaartobjecten zijn. Voer run uit met --force om ze toe te voegen; de oude kaart blijft dan bewaard als .omap.bak.
cli-map-light = De basiskaart: { $joined } lijnen samengevoegd, { $left } verborgen of te klein voor de kandidaatlagen overgelaten (niets verschoven)
cli-map-light-kept = De kandidaatlagen bevatten nu ook { $left } objecten of stukken die deze versie van de basiskaart haalt (met de tag freemapper:generalized). De behouden kaart heeft ze nog, dus ze staan er dubbel op. Vervang de kaart met --force om op te schonen (de oude blijft bewaard als .omap.bak).
cli-map-old-candidates = De kaart noemt nog templates/candidates.omap; deze run heeft die met alle kandidaten gevuld. De kandidaten komen nu ook in zeven lagen, elk in een eigen bestand (templates/candidates-*.omap). De kaart toont ze nadat je haar met --force hebt vervangen (de oude blijft bewaard als .omap.bak), of voeg ze zelf toe in OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF zonder bronvermelding: ODbL en dl-de/by-2-0 vragen je de bronnen te noemen als je de kaart doorgeeft; zie ATTRIBUTION.txt
cli-wrote-pdf = { $path } geschreven (de kaart zonder kandidaten)
cli-wrote-pdf-candidates = { $path } geschreven (de controleprint: de kaart met elke kandidaat, op de juiste plaats; controleer ze in het terrein)
cli-wrote-training = { $map } en { $pdf } geschreven (de trainingskaart: { $joined } samengevoegd, { $shifted } verschoven, { $left } weggelaten of afgesneden)
cli-training-free-ends = Lijneinden vrijgelaten: { $ways } waar een weg een andere weg raakte, { $others } bij andere lijnen
cli-training-apart-left = de trainingskaart heeft nog { $length } mm (op papier) waar objecten te weinig ruimte houden; controleer het in OOM
cli-kept-training = Trainingskaart { $path } behouden, zodat een run er geen overschrijft die is afgedrukt of doorgegeven; verwijder haar of geef --force mee om haar te vernieuwen

## OCAD

cli-ocad-kept = { $path } behouden (het kan jouw werk bevatten; --force vervangt het en bewaart het oude als .ocd.bak)
cli-ocad-old-candidates = De OCAD-kaart noemt { $path } nog als achtergrondkaart; deze run heeft die met alle kandidaten gevuld. De kandidaten komen nu ook in zeven lagen, elk in een eigen bestand (-candidates-*.ocd). De OCAD-kaart toont ze nadat je haar met --force hebt vervangen (de oude blijft bewaard als .ocd.bak), of voeg ze zelf toe in OCAD als achtergrondkaarten.
cli-ocad-wrote = { $count ->
        [one] { $path } geschreven (1 object)
       *[other] { $path } geschreven ({ $count } objecten)
    }
cli-ocad-left-out = Weggelaten uit de OCAD-kaart: { $skipped } objecten (geen symbool voor OCAD, geen vlak, of buiten het tekengebied van OCAD van ±2 m) en { $dropped } ontaarde lijndelen of gaten.
cli-ocad-load-symbols = Kies in OCAD Symbol > Load Colors and Symbols From… en kies de officiële symboolset{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , de IOF-schoolset 2019
       *[other] {" "}ISSprOM 2019
    }; die vervangt de plaatsvervangende symbolen van freemapper op nummer.

## select

cli-select-running = Gebiedskeuze voor { $path }: druk op "Klaar" op de pagina of op Ctrl+C om te stoppen.
cli-select-no-browser = De browser kon niet worden geopend ({ $error }); open het adres hierboven.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; geen gebied, kan geen kaartgebied zijn): { $place }
cli-find-hint = Maak het project aan met: freemapper init <dir> --osm { $id }
cli-find-no-area = Geen van deze plaatsen heeft een gebied dat een kaartgebied kan zijn. Teken het gebied met: freemapper select <dir>
cli-find-nothing = geen plaats gevonden voor "{ $query }"; probeer een andere naam, of teken het gebied met: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = waarschuwing: de adapter in { $folder } is weggelaten: { $finding }
cli-adapter-note = waarschuwing: de adapter in { $folder } laadt, maar: { $finding }
cli-adapter-from-project = waarschuwing: de adapter { $adapter } uit de projectmap wordt gebruikt; hij kan deze hosts aanspreken: { $hosts }
cli-adapter-overlaps = waarschuwing: de adapter { $adapter } dekt ook het gebied van { $others }; zijn objecten komen daar op kaarten
cli-adapter-dir-relative = waarschuwing: FREEMAPPER_ADAPTER_DIR is geen absoluut pad ({ $path }); het wordt genegeerd
cli-adapters-no-project = waarschuwing: { $path } is geen map; er zijn geen adapters van een project gelezen
cli-adapters-export-unknown = de adapter { $adapter } wordt door geen manifest van dit programma beschreven, dus er valt niets te exporteren; `freemapper adapters list` toont de id's
cli-adapters-export-taken = { $path } bestaat al en is niet leeg; exporteer naar een map die nog niet bestaat
cli-adapters-export-done = De { $count } bestanden van de adapter { $adapter } staan in { $path }. Er staat alleen in wat het manifest beschrijft; adressen die in de code van het programma staan, ontbreken.
cli-adapters-export-next = Pas de adressen aan in manifest.toml, controleer de map met `freemapper adapters check` en verplaats haar dan naar { $path }: vanaf de volgende start vervangt ze de ingebouwde adapter, met een waarschuwing. Verwijder je de map, dan is de ingebouwde weer terug.
cli-adapters-overlaps = deelt gebied met: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revisie { $revision }  gecontroleerd { $checked }  hosts: { $hosts }
cli-adapters-source-built-in = ingebouwd
cli-adapters-source-user = uit jouw map
cli-adapters-source-project = uit het project
cli-adapters-never-checked = nooit
cli-adapters-user-dir = Jouw adapters worden gelezen uit { $path }
cli-adapters-no-user-dir = Deze computer heeft geen map voor jouw adapters; FREEMAPPER_ADAPTER_DIR geeft er een op

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper mag nu ook { $host } aanspreken voor { $name }. De vermelding staat in { $file }; `freemapper adapters disallow-host { $adapter } { $host }` maakt dat ongedaan. Een lopende gebiedspagina of AI-assistent leest de vermelding pas bij de volgende start
cli-allow-host-already = { $host } is al toegestaan voor { $name }
cli-allow-host-declared = de adapter van { $name } noemt { $host } zelf; niets toe te staan
cli-allow-host-refused = { $host } kan niet worden toegestaan: { $reason }
cli-allow-host-unknown-adapter = er is geen ingebouwde adapter { $adapter }; de ingebouwde adapters zijn: { $adapters }
cli-allow-host-folder-adapter = { $adapter } is een adapter uit jouw adaptermap; voeg de host in plaats daarvan toe aan `[net] hosts` van zijn manifest
cli-allow-host-no-config = deze computer heeft geen configuratiemap om toegestane hosts in te bewaren
cli-allow-host-write = { $file } kon niet worden geschreven: { $detail }
cli-allow-host-unreadable = { $file } kan niet worden gelezen ({ $detail }); corrigeer of verwijder het bestand eerst; er is niets gewijzigd
cli-disallow-host-removed = freemapper spreekt { $host } niet meer aan voor { $name }
cli-disallow-host-absent = { $host } was niet toegestaan voor { $name }; niets om ongedaan te maken
cli-allowed-hosts-unreadable = waarschuwing: { $file } kon niet worden gelezen ({ $detail }); er zijn geen extra hosts toegestaan
cli-allowed-hosts-ignored = waarschuwing: { $file }: de vermelding { $host } voor { $adapter } wordt genegeerd: { $reason }
cli-allowed-hosts-no-adapter = freemapper heeft geen ingebouwde adapter met die naam
cli-allowed-host-redundant = waarschuwing: { $host } is nu een van de hosts die de adapter van { $name } noemt; jouw vermelding in { $file } is verwijderd
cli-adapters-allowed = door jou toegestaan: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revisie { $revision }, hosts: { $hosts }
cli-check-error = fout: { $finding }
cli-check-warning = waarschuwing: { $finding }
cli-check-passed = { $folder } doorstaat de controle
cli-check-passed-warnings = { $folder } doorstaat de controle, met { $count ->
        [one] één waarschuwing
       *[other] { $count } waarschuwingen
    }
cli-check-failed = { $folder } doorstaat de controle niet
cli-check-no-folder = er is geen map { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } MISLUKT: { $detail }
cli-check-live-not-asked = { $capability } niet opgevraagd: { $detail }
cli-check-live-box-outside = het kader ligt niet binnen de grens van de adapter { $adapter }; een live controle vraagt de services om een plaats die de adapter dekt
cli-check-live-cannot = de live controle kan niet draaien: { $detail }
cli-check-detail-not-declared = de adapter declareert er geen
cli-check-detail-stopped = een eerder onderdeel is mislukt
cli-check-detail-see-warnings = zie de waarschuwingen
cli-check-detail-outside-dataset = het kader ligt buiten de dataset
cli-check-detail-objects = { $count ->
        [one] één object
       *[other] { $count } objecten
    }
cli-check-detail-layers = { $layers } lagen, { $count } objecten
cli-check-detail-trees = { $count ->
        [one] één boom
       *[other] { $count } bomen
    }
cli-check-detail-extract = { $label }, gegevens van { $date }
cli-check-raster-orthophoto = orthofoto
cli-check-raster-terrain = terreinmodel
cli-check-raster-surface = oppervlaktemodel
cli-check-raster-tiles = { $product }: { $count ->
        [one] één tegel
       *[other] { $count } tegels
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } en { $last }

## Folder lock

cli-lock-held = een andere freemapper-run gebruikt deze map (sinds { $time }, gestart door { $who }). Wacht erop of stop haar eerst.
cli-lock-who-cli = de terminal
cli-lock-who-mcp = een AI-agent (MCP)
cli-lock-who-page = de selectiepagina
cli-lock-who-unknown = een ander programma
cli-lock-failed = kon de projectmap niet vergrendelen voor deze run: { $error }
cli-lock-stuck = iets anders dan een vergrendelingsbestand zit in de weg en kon niet worden verwijderd

## Run routes of the selection page

cli-run-bad-request = De run-opties zijn niet geldig: { $error }
cli-run-no-project = In deze map staat nog geen opgeslagen project. Sla eerst het gebied op.
cli-run-already-running = Voor dit project loopt al een run ({ $job }). Wacht tot hij klaar is of annuleer hem eerst.
cli-run-disk-stops = Niet genoeg vrije schijfruimte voor de download: { $messages }
cli-run-busy-elsewhere = Een ander project van dit programma draait nu een pipeline. Projecten delen de downloadcache, dus er loopt maar één run tegelijk. Probeer het opnieuw als hij klaar is.
cli-run-nothing-to-cancel = Er loopt geen run, dus er is niets te annuleren.
cli-run-unknown-job = Er is geen run { $job } voor dit project.
cli-status-too-fast = Statusverzoeken komen te vaak. Vraag hooguit vijf keer per seconde.
cli-open-folder-failed = De map kon niet worden geopend ({ $error }).
cli-open-folder-no-opener = Deze computer heeft geen programma om mappen te openen ({ $program }). Open de map zelf.
cli-open-folder-missing = De projectmap bestaat nog niet. Sla eerst het gebied op.
cli-open-folder-refused = De mapnaam is niet veilig om aan de bestandsbeheerder door te geven.
cli-save-running = Voor dit project loopt een run. Sla opnieuw op als hij klaar is.
cli-save-unknown-kind = Onbekend kaarttype "{ $kind }". Gebruik schoolyard, sprint of forest.
cli-save-scale-too-large = De schaal { $scale } is te groot.
cli-save-bad-content = De inhoudsinstellingen zijn niet geldig. Er is niets opgeslagen.
cli-map-image-none = Maak eerst de kaart.
cli-map-image-gone = Deze voorvertoning is niet meer beschikbaar.
cli-map-image-running = Voor dit project loopt een run. De voorvertoning kan worden berekend als hij klaar is.
cli-map-image-stale = Het opgeslagen project komt niet meer overeen met de gemaakte kaart, bijvoorbeeld doordat het gebied, het kaarttype of de schaal is gewijzigd. Start eerst een run die de kaart vervangt ("Bestaande kaart vervangen" op de pagina, of freemapper run --force).
cli-map-image-superseded = Intussen is een nieuwere voorvertoning gevraagd, dus deze is niet berekend.
cli-map-image-bad-content = De inhoudsinstellingen zijn niet geldig.
cli-map-image-bad-request = Het verzoek om een voorvertoning is niet geldig. Stuur een JSON-object, eventueel met de inhoudsinstellingen.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – gebiedskeuze
page-heading = Kies het kaartgebied
page-update-available = freemapper { $version } is beschikbaar. Update met
page-update-notes = Release-notes
page-folder = Projectmap
page-pick = Zoek een plaats
page-query-placeholder = bijv. Clara-Zetkin-Schule Strausberg
page-search-button = Zoeken
page-searching = Zoeken …
page-no-results = Niets gevonden.
page-point-only = alleen een punt, geen gebied
page-draw = Teken het zelf
page-rectangle = Rechthoek
page-polygon = Polygoon
page-finish-polygon = Polygoon sluiten
page-clear = Wissen
page-hint-rectangle = Klik op de kaart op twee tegenover elkaar liggende hoeken.
page-hint-polygon = Klik de hoeken na elkaar aan. Sluit af door op de eerste hoek te klikken, te dubbelklikken of op "Polygoon sluiten" te klikken. Esc annuleert, Backspace verwijdert de laatste hoek.
page-hint-idle = Kies een plaats uit de zoekresultaten of teken het gebied zelf.
page-summary = Gebied
page-source-osm = Van OpenStreetMap: { $name }
page-source-drawn = Met de hand getekend
page-size = Grootte met buffer: { $width } × { $height } m
page-tiles-one = { $n } tegel: { $names }
page-tiles-other = { $n } tegels: { $names }
page-download = Download: ongeveer { $mb } MB
page-download-none = Download: alles staat al in de cache
page-download-details = Wat wordt gedownload
page-cached = in de cache
page-disk = Schijfruimte: { $volumes }
page-disk-volume = { $free } vrij, ongeveer { $needed } nodig
page-cache-volume = cache
page-project-volume = projectmap
page-no-room = past niet op de schijf
page-no-room-skip = Niet alles past op de schijf (freemapper houdt { $reserve } vrij). Het ophalen laat de gemarkeerde delen weg. Maak eerst ruimte vrij als je ze nodig hebt:
page-no-room-stop = Niet genoeg schijfruimte (freemapper houdt { $reserve } vrij). Zonder de gemarkeerde delen is er geen kaart, dus het ophalen stopt voordat er iets wordt gedownload. Maak eerst ruimte vrij:
page-disk-unknown = freemapper kan de vrije schijfruimte niet lezen ({ $error }). Het haalt op zonder controle.
page-too-large = Het gebied is te groot: hooguit { $max } m per zijde (met buffer). Kies een kleiner gebied.
page-outside = Het gebied ligt buiten de ondersteunde deelstaten ({ $states }); freemapper heeft nog geen gegevensbronnen voor andere regio's.
page-outside-unknown = Het gebied ligt buiten de regio's die freemapper ondersteunt; er zijn daar nog geen gegevensbronnen.
page-adapters-failed = De luchtfoto's kunnen niet worden getoond ({ $detail }). De kaart toont alleen OpenStreetMap; al het andere werkt.
page-settings = Kaart
page-kind = Kaarttype
page-schoolyard = Schoolpleinkaart (1:1000)
page-sprint = Sprintkaart (1:4000)
page-forest = Boskaart (1:15000)
page-scale = Schaal 1:
page-map-title = Titel
page-contour-interval = Hoogtelijninterval
page-contour-auto = automatisch
page-steep-slope = Steilheidsdrempel
page-steep-slope-auto = automatisch (27°)
page-steep-slope-hint = Vanaf deze hellingshoek geldt het terrein als steil bij het zoeken naar taluds en rotswanden. 27° (automatisch) past bij heuvels, parken en steden. In rotssteden, kloven en steengroeven is bijna elke helling steil, zodat taluds en rotswanden samenvloeien tot één vlek en er geen wordt gevonden; 55° tot 65° scheidt ze. Een te hoge waarde laat alleen de steilste wanden over.
page-face-slope = Wanddrempel
page-face-slope-auto = automatisch (70°)
page-face-slope-hint = Vanaf deze hellingshoek wordt het terrein getekend als een zwart rotswandvlak. 70° (automatisch) past bij rotssteden zoals Adršpach (5,2 % van het oppervlak). 65° geeft meer en grotere zwarte vlakken, ook op steile hellingen (8,4 %). 75° houdt alleen echte wanden over, met rafelige randen (2,5 %). Een te hoge waarde laat bijna geen vlakken over.
page-settings-fixed = Het project bestaat al; kaarttype, schaal en titel staan in freemapper.toml. Het hoogtelijninterval kun je nog wel wijzigen.
page-save = Opslaan
page-saving = Opslaan …
page-saved = Opgeslagen in freemapper.toml.
page-zone-changed = Opgeslagen. Het gebied ligt nu in een andere UTM-zone; freemapper haalt de gegevens bij de volgende run opnieuw op.
page-provider-changed = Opgeslagen. Een andere gegevensaanbieder levert de hoogte- en luchtbeeldgegevens van het nieuwe gebied; freemapper haalt de gegevens bij de volgende run opnieuw op.
page-save-stale = freemapper.toml is sinds deze pagina het las in een ander venster of door een ander programma gewijzigd. Er is niets opgeslagen; de pagina kent nu de opgeslagen toestand. Sla opnieuw op om die te vervangen door het gebied dat hier staat.
page-has-map = De bestaande kaart (.omap) blijft zoals ze is; freemapper maakt nieuwe templates en kandidaten voor het nieuwe gebied. De optie "Bestaande kaart vervangen" hieronder (of "freemapper run --force") maakt ook de kaart opnieuw en bewaart de oude als .omap.bak.
page-done = Klaar
page-source = Broncode (AGPL-3.0-or-later)
page-closed = Klaar. Je kunt dit venster sluiten.
page-next = Daarna: freemapper run "{ $dir }"
page-made-files = Aangemaakt: { $files }. Map: { $dir }
page-back-to-agent = Zeg tegen je AI-assistent dat het gebied is opgeslagen.
page-nothing-saved = Er is niets opgeslagen.
page-failed = Fout: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Luchtfoto { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Wat op de kaart komt
page-content-help = Het profiel bepaalt wat direct op de kaart komt en wat een kandidaat blijft om te controleren. Een keuze voor een groep gaat boven het profiel. Ze wordt met het gebied opgeslagen.
page-content-profile-legend = Profiel
page-content-profile-safe = Veilig
page-content-profile-safe-hint = Elk object blijft waar de build het zet: gebouwen en de vlakken van het kadaster komen op de kaart; muren, hekken, bomen en veel uit OpenStreetMap (vijvers, paden met onbekend oppervlak) blijven kandidaten.
page-content-profile-balanced = Evenwichtig
page-content-profile-balanced-hint = Objecten van het kadaster, de registers en de straatopname komen op de kaart, met de hoogtelijnen en de verboden gebieden langs spoorrails. Muren, hekken, heggen, taluds, trappen, water, bomen, monumenten, doorgangen, paden en spoorwegen komen ook uit OpenStreetMap. Straatmeubilair, de andere verboden gebieden en de vormen uit het terreinmodel blijven kandidaten.
page-content-profile-full = Volledig
page-content-profile-full-hint = Alles komt op de kaart, zoals in de controleprint; alleen verborgen kandidaten blijven eruit.
page-content-groups-legend = Per groep
page-content-groups-help = "Zoals in het profiel" volgt het profiel. Wat dat voor een groep betekent, hangt af van waar elk object vandaan komt.
page-content-place-profile = Zoals in het profiel
page-content-place-map = Kaart
page-content-place-candidates = Kandidaat
page-content-place-off = Uit
page-content-group-walls = Muren
page-content-group-fences = Hekken
page-content-group-hedges = Heggen
page-content-group-slopes = Taluds en rotswanden
page-content-group-stairs = Trappen
page-content-group-trees = Bomen en boomgroepen
page-content-group-water-features = Water, fonteinen en bronnen
page-content-group-monuments = Torens, monumenten en hoogspanningslijnen
page-content-group-furniture = Straatmeubilair en lantaarns
page-content-group-passages = Doorgangen
page-content-group-contours = Hoogtelijnen
page-content-group-landforms = Heuveltjes, kuilen, taluds en rotswanden uit het terreinmodel
page-content-group-vegetation = Vegetatie uit laserpuntenwolken (Brandenburg, Berlijn, Noordrijn-Westfalen, Rijnland-Palts, Saarland en Beieren; uit tot je het kiest; downloadt ongeveer 40 tot 600 MB per km²)
page-content-group-out-of-bounds = Verboden gebieden
page-content-group-paths = Paden en spoorwegen
page-content-group-streets = Straten en pleinen
page-run-heading = Kaart maken
page-preview-heading = Kaartvoorvertoning
page-preview-stored = Zoals laatst gemaakt.
page-preview-live = Met de instellingen op deze pagina, nog niet opgeslagen.
page-preview-live-saved = Met de opgeslagen instellingen, nog niet in een run gemaakt.
page-preview-computing = Berekenen …
page-preview-none = Nog geen kaart gemaakt.
page-preview-later = Deze kaart heeft nog geen voorvertoning. Die komt met de volgende run die de kaart schrijft.
page-preview-stale = Het gebied, het kaarttype of de schaal is gewijzigd sinds de kaart is gemaakt. De afbeelding toont nog de kaart zoals laatst gemaakt; voor een nieuwe voorvertoning is een run met "Bestaande kaart vervangen" nodig.
page-preview-area-unsaved = Het gebied op deze pagina is nog niet opgeslagen. De afbeelding toont de kaart zoals laatst gemaakt; voor het nieuwe gebied is een run met "Bestaande kaart vervangen" nodig.
page-preview-run = Zoals laatst gemaakt. De pagina werkt de afbeelding bij als de run klaar is.
page-preview-open = Op volledige grootte openen
page-preview-alt = Voorvertoning van de kaartobjecten
page-opt-ocad = OCAD-startproject schrijven
page-help-ocad = Voor iedereen die in OCAD verdergaat.
page-opt-pdf = Afdrukbare PDF
page-help-pdf = Alleen kaartobjecten, geen kandidaten.
page-opt-pdf-candidates = Controleprint met elke kandidaat
page-opt-training = Trainingskaart
page-help-training = De kaart en elke kandidaat, opgeschoond om ze zo af te drukken; met OCAD ook als OCAD-bestand.
page-views-heading = Weergaven van de trainingskaart
page-view-no-paths = Zonder paden (Ultimate-kaart)
page-view-brown-only = Alleen bruin (hoogtelijnoriëntatie)
page-view-no-north-lines = Zonder noordlijnen
page-view-custom = Andere weergave
page-help-views = Elke aangevinkte weergave is een extra bestand naast de trainingskaart, `<name>-training-<view>.omap` en de bijbehorende PDF, met OCAD ook een OCAD-bestand. Een andere weergave is `no:505,506` (symbolen weglaten) of `only:101,102` (alleen deze houden); met `+` combineer je weergaven. Titel, schaal en bronnen blijven altijd.
page-help-views-forest = Zonder paden en alleen bruin zijn alleen voor boskaarten.
page-opt-scale-bar = Schaalbalk op de PDF's
page-help-scale-bar = Duitse kaarten hebben er meestal geen.
page-opt-sources = Bronregels op de PDF's
page-sources-auto = automatisch
page-sources-full = volledig
page-sources-credit-only = alleen freemapper
page-warn-credit-only = De PDF's noemen dan alleen freemapper. De gegevenslicenties (ODbL, dl-de/by-2-0) vereisen bronvermelding als je de kaart doorgeeft; dan noem je de bronnen zelf. ATTRIBUTION.txt vermeldt ze allemaal.
page-need-pdf = De schaalbalk en de bronregels gelden alleen als er een PDF wordt gemaakt.
page-opt-offline = Alleen cache, geen netwerk
page-opt-force = Bestaande kaarten vervangen
page-help-force = De kaart en, als "Trainingskaart" is aangevinkt, de trainingskaart. De oude bestanden blijven bewaard als .omap.bak.
page-opt-permits = Vergunningsnotities
page-help-permits = Bestuurlijke eenheden, beschermde gebieden en wie je moet vragen, in permits.md. Een hint, geen juridisch advies.
page-run-start = Kaart maken
page-run-again = Opnieuw maken
page-run-cancel = Annuleren
page-run-cancelling = Annuleren …
page-warnings-heading = Waarschuwingen
page-where-files = De bestanden staan in de projectmap:
page-open-folder = Map openen
page-log-label = Logboek van de run
page-force-title = Bestaande kaarten vervangen?
page-force-text = De bestaande kaart wordt vervangen, en als "Trainingskaart" is aangevinkt ook de trainingskaart. Je wijzigingen gaan niet verloren: de oude bestanden blijven als .omap.bak in de projectmap staan. Doorgaan?
page-force-cancel = Annuleren
page-force-confirm = Vervangen
page-step-fetch = Gegevens ophalen
page-step-derive = Templates berekenen
page-step-map = Kaart schrijven
page-step-ocad = OCAD-project schrijven
page-step-pdf = PDF schrijven
page-step-pdf-candidates = Controleprint schrijven
page-step-training = Trainingskaart schrijven
page-step-permits = Vergunningsnotities schrijven
page-step-line = Stap { $index } van { $total }: { $label }
page-starting = De run start …
page-file-kind-map = kaart
page-file-kind-template = template
page-file-kind-ocad = OCAD-kaart
page-file-kind-ocad-candidates = OCAD-kandidaten
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = controleprint
page-file-kind-training-map = trainingskaart
page-file-kind-training-pdf = PDF van de trainingskaart
page-file-kind-training-ocad = OCAD-trainingskaart
page-file-kind-training-view-map = weergave van de trainingskaart
page-file-kind-training-view-pdf = PDF van een weergave van de trainingskaart
page-file-kind-training-view-ocad = OCAD-weergave van de trainingskaart
page-file-kind-preview = kaartvoorvertoning
page-file-kind-permits = vergunningsnotities
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Klaar. De bestanden zijn geschreven.
page-result-failed = De run is mislukt: { $error }
page-result-failed-kept = Wat is geschreven blijft in de projectmap staan; een nieuwe run gaat daar verder.
page-result-cancelled = Geannuleerd. Wat al is gedownload blijft in de cache; een nieuwe run gaat daar verder.
page-result-title-done = Kaart gemaakt
page-result-title-failed = Run mislukt
page-result-title-cancelled = Run geannuleerd
page-kept-map = De bestaande kaart is ongemoeid gelaten. "Bestaande kaart vervangen" maakt haar opnieuw en bewaart de oude als .omap.bak.
page-no-files = Er zijn geen bestanden gemaakt.
page-opened-folder = De map is geopend.
page-open-failed = { $message } Map: { $dir }
page-why-no-area = Kies eerst een gebied.
page-why-checking = De voorvertoning wordt berekend …
page-why-too-large = Het gebied is te groot.
page-why-outside = Het gebied ligt buiten de ondersteunde deelstaten.
page-why-disk = Niet genoeg vrije schijfruimte (zie boven).
page-why-running = Er loopt een run.
page-err-scale-bar-no-pdf = De schaalbalk heeft een PDF nodig.
page-err-sources-no-pdf = De bronregels hebben een PDF nodig.
page-save-failed = Het gebied is niet opgeslagen; er is geen run gestart.
page-poll-lost = Geen antwoord van de server. freemapper blijft het proberen.
page-run-unknown = De server kent deze run niet meer.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Sluiten

## update (spec 3.4 to 3.6)

update-source-build = Deze freemapper is uit de broncode gebouwd. Werk hem bij met git pull en cargo build --release --locked.
update-current = freemapper { $version } is up-to-date.
update-available = freemapper { $new } is beschikbaar (je hebt { $old }), uitgebracht op { $released }.
update-details =
    Release-notes: { $notes }
    Download: { $size } MB
update-decimal-separator = ,
update-ask = Nu installeren? [j/N]
update-not-installed = Er is niets geïnstalleerd.
update-needs-yes = de invoer is geen terminal, dus freemapper kan niet vragen voordat het installeert; voer freemapper update --yes uit om zonder vragen te installeren
update-installing = freemapper { $version } wordt gedownload en gecontroleerd
update-done = freemapper bijgewerkt van { $old } naar { $new }. Release-notes: { $notes }
update-mcp-restart = Een draaiende MCP-server houdt de oude versie totdat de AI-assistent hem opnieuw start.
update-rolled-back = Teruggegaan naar freemapper { $version }. De versie die je hebt verlaten is bewaard als { $path }.
update-not-writable = kan niet schrijven naar de programmamap { $path }, en freemapper vraagt niet om beheerdersrechten. Pak de nieuwe release daar zelf uit, of verplaats freemapper naar een map waarin je kunt schrijven.
update-too-old = freemapper { $new } kan niet worden geïnstalleerd met de zelfupdate van { $old }; installeer { $needed } of nieuwer zelf, zie { $notes }
update-busy = een andere update of terugdraaiing loopt in de programmamap (vergrendelingsbestand { $path }); wacht tot die klaar is en probeer het opnieuw
update-bad-signature = de release kon niet worden geverifieerd ({ $error }); er is niets geïnstalleerd
update-check-bad-signature = de nieuwste freemapper-release kon niet worden geverifieerd ({ $error }); er is niets geïnstalleerd
update-no-manifest = de nieuwste release kan zichzelf niet installeren (hij heeft geen updatemanifest); download hem van { $page }
update-no-manifest-version = release { $version } bestaat niet of kan zichzelf niet installeren (hij heeft geen updatemanifest); zie { $page }
update-no-asset = de release heeft geen programma voor dit systeem ({ $platform }); er is niets geïnstalleerd
update-no-previous = er is geen eerdere versie om naar terug te gaan ({ $path } ontbreekt)
update-failed = de update is mislukt: { $error }
update-mode-set = Updatemodus ingesteld op { $mode } in { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } overschrijft deze instelling zolang het is gezet.
update-mode-failed = kon { $path } niet schrijven: { $error }
update-notice = freemapper { $new } is beschikbaar (je hebt { $old }). Update met: freemapper update
update-applies-next-start = De nieuwe versie geldt vanaf de volgende start van freemapper.
update-install-failed = freemapper { $new } is niet geïnstalleerd: { $error }
update-no-config-dir = geen configuratiemap op dit systeem; stel FREEMAPPER_CONFIG_DIR in
mcp-adapter-note = Opmerking voor de persoon: bij de start vond de freemapper-server dit over de adaptermappen. Geef het door; de persoon herstelt de map of verwijdert haar en start daarna de server opnieuw.
    { $lines }
update-mcp-note = Opmerking voor de persoon: freemapper { $new } is beschikbaar (dit is { $old }). Vraag het hen voordat je install_update aanroept.
update-mcp-needs-confirm = er is niets geïnstalleerd; vraag de persoon eerst en roep install_update aan met confirm: true zodra die akkoord gaat
update-mcp-note-manual = Opmerking voor de persoon: freemapper { $new } is beschikbaar (dit is { $old }), maar install_update kan hem niet installeren; installeer { $needed } of nieuwer zelf, zie { $notes }
update-mcp-note-bad-signature = Opmerking voor de persoon: de nieuwste freemapper-release kon niet worden geverifieerd ({ $error }); er is niets geïnstalleerd. Vertel het hen en installeer hem niet op een andere manier.
update-mcp-restart-now = Start de freemapper MCP-server nu opnieuw; hij draait nog de oude versie.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = fout
error-cancelled = geannuleerd
cli-cancelled-kept = geannuleerd; behouden: { $files }
error-io = { $path }: { $message }
error-io-disk-full = de schijf is vol, dus freemapper kon { $path } niet schrijven. Maak ruimte vrij en probeer het opnieuw

## Errors: project

error-project-already-exists = project bestaat al: { $path }
error-project-map-exists = kaart bestaat al (ze kan jouw werk bevatten); gebruik --force om haar te vervangen: { $path }
error-project-invalid-scale = ongeldige schaal 1:{ $scale }
error-project-invalid-contour-interval = ongeldig hoogtelijninterval { $interval } m in freemapper.toml; het moet groter zijn dan 0
error-project-invalid-steep-slope = ongeldige steilheidsdrempel { $slope }° in freemapper.toml; die moet tussen 15° en 80° liggen
error-project-invalid-face-slope = ongeldige wanddrempel { $slope }° in freemapper.toml; die moet tussen 55° en 85° liggen
error-project-scale-not-allowed = schaal 1:{ $scale } is niet toegestaan voor { $kind }kaarten; gebruik { $allowed }
error-project-scale-not-allowed-enlarge = schaal 1:{ $scale } is niet toegestaan voor { $kind }kaarten; gebruik { $allowed }. Teken op 1:4000, freemapper-print vergroot bij het afdrukken naar 1:3000
word-kind-schoolyard = schoolplein
word-kind-sprint = sprint
word-kind-forest = bos
error-project-toml = ongeldig projectbestand: { $message }
error-project-unknown-adapter = het projectbestand noemt de adapter "{ $adapter }", die deze freemapper niet heeft; een nieuwere freemapper kan hem meebrengen (`freemapper update`), of hij komt uit jouw adaptermap (`freemapper adapters list` toont hem) of, met --adapters-from-project op de opdrachtregel, uit de map `adapters` van het project (alleen voor een project dat je vertrouwt)
error-project-not-fetched = nog geen tegels opgehaald; voer eerst `freemapper fetch` uit
error-project-ocd = OCAD-export: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: afbeelding en worldfile komen niet overeen met het kaartgebied; voer `freemapper derive` opnieuw uit
error-project-tiles-outdated = het kaartgebied heeft tegels { $tiles } nodig die niet zijn opgehaald; voer `freemapper fetch` uit
error-project-buildings-not-fetched = geen ALKIS-gebouwen opgeslagen voor dit kaartgebied; voer eerst `freemapper fetch` online uit
error-project-orthophoto-not-fetched = geen orthofoto opgeslagen voor dit kaartgebied; voer eerst `freemapper fetch` online uit
error-project-terrain-not-fetched = geen terreinmodel opgeslagen voor dit kaartgebied; voer eerst `freemapper fetch` online uit
error-project-candidates-outdated = de opgeslagen OSM-kandidaten dekken het kaartgebied niet; voer `freemapper fetch` uit
error-project-stored = { $path }: { $message }; voer `freemapper fetch` opnieuw uit
error-project-area-too-large = kaartgebied plus buffer is { $width } × { $height } m, meer dan { $limit } m per zijde; verklein het kaartgebied of buffer_m in freemapper.toml
error-project-area-crosses-country = het kaartgebied ligt aan beide kanten van een landsgrens; kaarten over een grens heen worden nog niet ondersteund, dus teken het gebied binnen één land
error-project-permits-not-for-region = de vergunningsnotities zijn voor deze regio nog niet beschikbaar: ze vragen Duitse services op en noemen Duitse instanties
error-project-outside-coverage = het kaartgebied ligt buiten de ondersteunde deelstaten ({ $states }); freemapper heeft tot nu toe alleen daarvoor gegevensbronnen

## Errors: disk space

error-space-required = niet genoeg schijfruimte voor { $part }. Er is { $needed } nodig, maar { $path } heeft naast de reserve van { $reserve } nog maar { $left } vrij. Maak ruimte vrij en voer `freemapper fetch` opnieuw uit
error-space-skipped = freemapper slaat { $part } over wegens gebrek aan schijfruimte. Er is { $needed } nodig, maar { $path } heeft naast de reserve van { $reserve } nog maar { $left } vrij. Maak ruimte vrij en voer `freemapper fetch` opnieuw uit
word-part-map-data = de ALKIS- en OSM-kaartgegevens
word-part-terrain = het terreinmodel
word-part-orthophoto = de orthofoto
word-part-surface = het oppervlaktemodel voor de nDSM-template
word-part-osm-update = de update van het gecachete OSM-extract
word-part-k-five = de Berlijnse K5-kaarttemplate
word-part-berlin-layers = de Berlijnse ALKIS-structuren, het boomkadaster, de straatopname en de puntlagen
word-part-state-layers = de volledige ALKIS-structuren, de objecten van het landschapsmodel en de boomkadasters
word-part-point-cloud = de laserpuntenwolken voor de vegetatiekandidaten

## Errors: downloads

error-fetch-http = download van { $url } mislukt: { $message }
error-fetch-http-temporary = download van { $url } mislukt: { $message }. De dienst is waarschijnlijk overbelast of in onderhoud; freemapper heeft maximaal { $attempts } keer geprobeerd. Start de run over enkele minuten opnieuw; wat al is gedownload, blijft bewaard.
error-fetch-corrupt-archive = download van { $url } is geen compleet tegelarchief: { $message }
error-fetch-corrupt-extract = download van { $url } is geen compleet OSM-extract: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] tegel { $tiles } wordt niet gedekt door de gepubliceerde tegels; verklein het kaartgebied of buffer_m in freemapper.toml
       *[other] tegels { $tiles } worden niet gedekt door de gepubliceerde tegels; verklein het kaartgebied of buffer_m in freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline en niet in de cache: { $product }-tegel { $tiles }; voer `freemapper fetch` online uit
       *[other] offline en niet in de cache: { $product }-tegels { $tiles }; voer `freemapper fetch` online uit
    }
word-product-orthophoto = orthofoto (DOP20)
word-product-dtm = terreinmodel (DGM1)
word-product-image-dsm = oppervlaktemodel (bDOM)
word-product-saxony-dtm = terreinmodel (GeoSN DGM1)
word-product-saxony-dsm = oppervlaktemodel (GeoSN DOM1)
error-fetch-osm-extract-missing = geen OSM-extract in de cache; voer `freemapper fetch` online uit
error-fetch-not-found = { $name } niet gevonden op de server
error-fetch-refused = { $url }: de server weigerde het verzoek (HTTP { $status })
error-fetch-refused-detail = { $url }: de server weigerde het verzoek (HTTP { $status }): { $detail }
error-fetch-too-large = download van { $url } is groter dan { $limit } MB; gestopt
error-fetch-not-stored = geen { $label } opgeslagen voor dit kaartgebied: { $path } ontbreekt, kan niet worden gelezen of dekt het niet; voer eerst `freemapper fetch` online uit
error-fetch-range-ignored = { $url }: de server negeerde het bytebereik { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper stuurt geen verzoeken naar dit adres: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper stuurt geen verzoeken naar dit adres: de adapter van { $name } noemt de host { $host } niet{ $from }. Als de service daarheen is verhuisd, sta de host toe met `freemapper adapters allow-host { $adapter } { $host }` en meld het op https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper stuurt geen verzoeken naar dit adres: de adapter { $adapter } noemt de host { $host } niet{ $from }. Als de service daarheen is verhuisd, voeg de host toe aan `[net] hosts` van het manifest van de adapter
error-fetch-host-redirected-from = {" "}({ $url } verwijst daarheen door)
error-reason-not-a-host-name = geen hostnaam
error-reason-not-ascii = een naam buiten ASCII moet in zijn xn--vorm worden geschreven
error-reason-not-a-dns-name = geen DNS-naam
error-reason-no-domain = een naam van deze computer of zonder domein
error-reason-local-name = een naam van het lokale netwerk
error-reason-ip-address = een IP-adres, geen DNS-naam
error-reason-unexpected-character = onverwacht teken
error-reason-not-https = geen https
error-reason-user-info = gebruikersinformatie
error-reason-port = een andere poort dan 443
error-reason-host-not-declared = de host staat niet bij de gedeclareerde hosts
error-reason-local-address = een adres van deze computer of van het lokale netwerk, niet van een server op internet
error-reason-too-many-redirects = meer dan { $count } doorverwijzingen
error-reason-redirect-nowhere = een doorverwijzing (HTTP { $status }) naar nergens
error-fetch-currency-csv = actualiteits-CSV regel { $line }: { $message }
error-alkis = ongeldige ALKIS-gegevens: { $message }
error-osm = OSM-gegevens: { $message }

## Errors: templates and rasters

error-derive-zip = tegelarchief: { $message }
error-derive-image = afbeelding: { $message }
error-derive-no-tiles = geen tegels om uit af te leiden
error-derive-area-too-large = kaartgebied te groot voor één template: { $width } × { $height } pixels (hooguit { $limit } per zijde); verklein het kaartgebied of buffer_m in freemapper.toml
error-raster-world-file = ongeldig worldfile: { $message }
error-raster-misaligned = rasters liggen niet op hetzelfde pixelraster: { $message }
error-raster-pixel-size = rasters hebben verschillende pixelgroottes: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = ongeldige GeoJSON: { $message }
error-area-no-polygon = GeoJSON bevat geen polygoon
error-area-multi-polygon = MultiPolygon met { $parts } delen; splits het kaartgebied in één polygoon
error-area-out-of-range = coördinaat buiten bereik: lat { $lat }, lon { $lon }
error-area-not-an-area = geen bruikbaar kaartgebied: { $message }
error-georef-date = datum { $date } valt buiten de geldigheid van het World Magnetic Model
error-georef-map-range = punt E { $easting } N { $northing } ligt buiten het coördinatenbereik van de kaart
error-omap-xml = ongeldige OOM-XML: { $message }
error-omap-missing-element = element <{ $name }> niet gevonden
error-ocd-duplicate-symbol = symboolnummer { $symbol } wordt twee keer gebruikt
error-ocd-unknown-symbol = object verwijst naar symbool { $symbol }, dat niet in het bestand staat
error-ocd-wrong-type = een object van symbool { $symbol } heeft het verkeerde type voor zijn symbool

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Gebruik:
cli-heading-commands = Opdrachten:
cli-heading-options = Opties:
cli-heading-arguments = Argumenten:
cli-help-possible-values = mogelijke waarden
cli-help-default = standaard
cli-arg-help = Help tonen
cli-arg-version = Versie tonen
cli-help-help = De help van een opdracht tonen
cli-arg-help-command = De opdracht waarvan de help wordt getoond, bijvoorbeeld run
cli-parse-hint = Probeer '--help' voor meer informatie.
cli-double-click-start = freemapper opent in de browser de pagina waar u het kaartgebied kiest. De projectmap is { $path }. Sluit dit venster om het programma te beëindigen.
cli-parse-unknown-argument = onverwacht argument { $arg }
cli-parse-missing-required = vereist argument ontbreekt: { $args }
cli-parse-invalid-value = ongeldige waarde { $value } voor { $arg }
cli-parse-invalid-value-choices = ongeldige waarde { $value } voor { $arg }; mogelijke waarden: { $values }
cli-parse-invalid-subcommand = onbekende opdracht { $value }
cli-parse-missing-subcommand = een opdracht is vereist
cli-parse-too-many-values = onverwachte waarde { $value } voor { $arg }
cli-parse-argument-conflict = { $arg } kan niet samen met { $other } worden gebruikt

help-freemapper = Bouwt OOM-startprojecten uit geodata
help-freemapper-lang = Taal van de programmateksten, `de` of `en` (standaard: FREEMAPPER_LANG, dan de systeemtaal, dan Engels)
help-init = Een projectmap voor een kaartgebied aanmaken
help-init-dir = Aan te maken projectmap
help-init-geojson = GeoJSON-bestand met de polygoon van het kaartgebied (EPSG:4326)
help-init-osm = OSM-object van het kaartgebied, bijv. way/992423119
help-init-place = Naam van de plaats, bijv. "Clara-Zetkin-Schule Strausberg"; freemapper gebruikt de beste treffer met een gebied
help-init-part = Welk deel van een OSM-gebied van meerdere afzonderlijke delen te gebruiken (1 is het grootste)
help-init-kind = Kaarttype
help-arg-scale = Schaalnoemer van een nieuw project: schoolyard 1000 (standaard); sprint 4000 (standaard) of 5000; forest 15000 (standaard), 10000 of 7500. Andere waarden worden geweigerd, omdat freemapper-print ze weigert
help-init-title = Kaarttitel voor de randinformatie (standaard: de mapnaam)
help-export = Het startproject (OOM-kaart, templates, bronvermelding) in een projectmap schrijven
help-export-dir = Projectmap
help-arg-force = Een bestaand kaartbestand vervangen en het oude bewaren als `<name>.omap.bak`
help-export-ocad = Schrijf ook een OCAD-startpunt in `ocad/` (OCAD 12-formaat); een bestaande OOM-kaart blijft dan staan tenzij je --force meegeeft
help-export-pdf = Schrijf ook `<name>.pdf`, de afdrukbare kaart (alleen de kaartobjecten, geen kandidaten); een bestaande OOM-kaart blijft dan staan tenzij je --force meegeeft
help-export-pdf-candidates = Schrijf ook `<name>-candidates.pdf`, de controleprint: de kaart met elke kandidaat, op de juiste plaats; een bestaande OOM-kaart blijft dan staan tenzij je --force meegeeft
help-export-training = Schrijf ook de trainingskaart `<name>-training.omap` en haar PDF `<name>-training.pdf`: de kaart en elke kandidaat, opgeschoond om ze zo af te drukken; met --ocad ook `ocad/<name>-training.ocd`. Een bestaande OOM-kaart blijft dan staan tenzij je --force meegeeft, en --force vervangt ook een bestaande trainingskaart
help-export-view = Met --training: schrijf ook de trainingskaart zonder wat een weergave weglaat, als `<name>-training-<view>.omap` en haar PDF (met --ocad ook een OCAD-bestand). Weergaven: `no-paths` (geen wegen, paden, trappen), `brown-only` (alleen bruine symbolen en het kader), `no-north-lines`, of een lijst `no:<codes>` / `only:<codes>`; combineer met `+`. Herhaal de optie voor meerdere bestanden
help-export-pdf-scale-bar = De schaalbalk op de PDF's tekenen (standaard uit, Duitse kaarten hebben er meestal geen; het schaalgetal blijft in het titelblok)
help-export-pdf-sources = Bronregels op de PDF's: `auto` drukt de volledige lijst af en kort haar alleen in als ze meer dan 5 regels zou innemen, `full` kort nooit in, `credit-only` drukt alleen de bronvermelding van freemapper af (ODbL en dl-de/by-2-0 vragen je dan de bronnen te noemen als je de kaart doorgeeft). ATTRIBUTION.txt vermeldt altijd elke bron
help-fetch = De gegevens voor het kaartgebied downloaden (orthofoto, terreinmodel, nDSM, ALKIS-gebouwen, OSM)
help-fetch-dir = Projectmap
help-fetch-offline = Alleen de cache gebruiken, zonder netwerktoegang
help-arg-cache-dir = Cachemap voor gedownloade tegels (standaard: de cachemap van het systeem + `freemapper`)
help-derive = De templates (orthofoto, hillshade, helling, lokaal reliëf, nDSM), hoogtelijnen en landvormen berekenen uit de opgehaalde gegevens
help-derive-dir = Projectmap
help-run = Ophalen, afleiden en exporteren in één keer
help-run-dir = Projectmap
help-run-offline = Alleen de cache gebruiken, zonder netwerktoegang
help-run-ocad = Schrijf ook een OCAD-startpunt in `ocad/` (OCAD 12-formaat)
help-run-pdf = Schrijf ook `<name>.pdf`, de afdrukbare kaart (alleen de kaartobjecten, geen kandidaten)
help-run-pdf-candidates = Schrijf ook `<name>-candidates.pdf`, de controleprint: de kaart met elke kandidaat, op de juiste plaats
help-run-training = Schrijf ook de trainingskaart `<name>-training.omap` en haar PDF `<name>-training.pdf`: de kaart en elke kandidaat, opgeschoond om ze zo af te drukken; met --ocad ook `ocad/<name>-training.ocd`. Een bestaande trainingskaart blijft staan tenzij je --force meegeeft
help-run-view = Met --training: schrijf ook de trainingskaart zonder wat een weergave weglaat, als `<name>-training-<view>.omap` en haar PDF (met --ocad ook een OCAD-bestand). Weergaven: `no-paths` (geen wegen, paden, trappen), `brown-only` (alleen bruine symbolen en het kader), `no-north-lines`, of een lijst `no:<codes>` / `only:<codes>`; combineer met `+`. Een bestaand weergavebestand blijft staan tenzij je --force meegeeft
help-run-pdf-scale-bar = De schaalbalk op de PDF's tekenen (standaard uit; het schaalgetal blijft in het titelblok)
help-run-pdf-sources = Bronregels op de PDF's: `auto` drukt de volledige lijst af en kort haar alleen in als ze meer dan 5 regels zou innemen, `full` kort nooit in, `credit-only` drukt alleen de bronvermelding van freemapper af (ODbL en dl-de/by-2-0 vragen je dan de bronnen te noemen als je de kaart doorgeeft). ATTRIBUTION.txt vermeldt altijd elke bron
help-select = Het kaartgebied in de browser kiezen door het te tekenen of een plaats te kiezen. Opslaan schrijft freemapper.toml en maakt zo nodig de projectmap aan
help-select-dir = Projectmap (wordt bij het opslaan aangemaakt als ze niet bestaat)
help-select-kind = Kaarttype van een nieuw project
help-select-title = Kaarttitel van een nieuw project
help-select-no-browser = Alleen het adres van de pagina tonen; de browser niet openen
help-select-port = Poort op 127.0.0.1 (standaard: een vrije)
help-select-offline = De pagina start met "alleen cache, geen netwerk" aangevinkt
help-select-ocad = De pagina start met het OCAD-startproject aangevinkt
help-select-pdf = De pagina start met de afdrukbare PDF aangevinkt
help-select-pdf-candidates = De pagina start met de controleprint (elke kandidaat) aangevinkt
help-select-training = De pagina start met de trainingskaart aangevinkt
help-select-pdf-scale-bar = De pagina start met de schaalbalk op de PDF's aangevinkt
help-select-pdf-sources = De pagina start met deze keuze van bronregels op de PDF's, zoals bij `run`
help-find = De OpenStreetMap-plaatsen voor een naam tonen, gebieden eerst, met de id voor init --osm
help-find-query = Naam van de plaats, bijv. "Clara-Zetkin-Schule Strausberg"
help-find-json = De treffers als JSON tonen, met dezelfde velden als de MCP-tool find_place
help-mcp = De tools via MCP (stdin/stdout) aan een AI-agent aanbieden
help-update = Een nieuwere freemapper uit de GitHub-releases installeren, na te hebben gevraagd. Een build uit de broncode zegt in plaats daarvan hoe je hem bijwerkt
help-update-check = Alleen controleren: afsluitcode 10 als er een nieuwere versie is, 0 als deze actueel is
help-update-yes = Installeren zonder te vragen; nodig als de invoer geen terminal is
help-update-version = Deze versie (x.y.z) installeren in plaats van de nieuwste, ook een oudere
help-update-rollback = Teruggaan naar de versie die de laatste update heeft vervangen
help-update-mode = Bewaar wat de dagelijkse controle doet als ze een nieuwere versie vindt (standaard: notify). Werkt in elke build, omdat het alleen de instelling schrijft
help-adapters = De regio-adapters tonen: de deelstaten en landen waarvoor freemapper gegevensbronnen heeft
help-adapters-list = De adapters tonen met waar elk vandaan komt (ingebouwd, jouw map, een project), zijn revisie, de dag van de laatste controle en de hosts die hij aanspreekt
help-adapters-list-json = De lijst als JSON tonen
help-adapters-list-project = Lees ook de adapters in de map `adapters` van deze projectmap, zoals --adapters-from-project zou doen
help-adapters-check = De map van een adapter controleren voordat hij wordt gebruikt of doorgegeven: of hij zou laden, wat zijn auteur moet weten, en met --live of zijn services antwoorden. Afsluitcode 0: doorstaan; 1: er zijn bevindingen; 2: de controle kon niet draaien
help-adapters-check-dir = De map van de adapter; haar naam is de id van de adapter
help-adapters-check-live = Vraag ook de services van de adapter om het kader dat --bbox noemt en meld wat elk levert. De verzoeken blijven bij de hosts die het manifest declareert
help-adapters-check-bbox = Het kader van de live controle in graden, als minLon,minLat,maxLon,maxLat. Het ligt binnen de grens van de adapter; 150 m bij 150 m is genoeg
help-adapters-check-json = Het resultaat als JSON tonen
help-adapters-probe = Elke dienst die de ingebouwde adapters noemen één keer bevragen en zeggen of hij antwoordt: ok, slow, moved of error. De afsluitcode is 0, ook als een dienst faalt; het resultaat is het antwoord
help-adapters-probe-json = Het resultaat als JSON tonen
help-adapters-probe-status = Lees de status van afgelopen nacht uit dit bestand (het mag ontbreken) en druk de nieuwe status af als JSON: een dienst ligt eruit na twee slechte nachten achter elkaar. De wijzigingen gaan naar standaardfout
help-adapters-allow-host = Een ingebouwde adapter nog een host toestaan: voor een deelstaat waarvan de service is verhuisd naar een server die zijn adapter nog niet noemt. De melding over het geweigerde adres zegt welke host. De host wordt gecontroleerd als een host van een manifest, en freemapper maakt nog steeds alleen verbinding met servers op internet
help-adapters-allow-host-adapter = De id van de ingebouwde adapter, zoals `adapters list` hem toont
help-adapters-allow-host-host = De toe te staan host: een DNS-naam, geen adres, geen poort
help-adapters-disallow-host = Een host intrekken die allow-host heeft toegestaan
help-adapters-disallow-host-adapter = De id van de ingebouwde adapter
help-adapters-disallow-host-host = De in te trekken host
help-adapters-export = Schrijf de ingebouwde adapter van een deelstaat als adaptermap die je kunt aanpassen: het manifest en de bestanden die het manifest noemt, in `<dir>/<id>`. De map mag nog niet bestaan of moet leeg zijn. Pas de adressen in `manifest.toml` aan, controleer de map met `adapters check` en zet haar in je adaptermap: ze vervangt dan de ingebouwde adapter, met een waarschuwing. Verwijder je de map, dan is de ingebouwde weer terug
help-adapters-export-adapter = De id van de ingebouwde adapter, zoals `adapters list` hem toont
help-adapters-export-dir = De map waarin de map `<id>` wordt aangemaakt
help-arg-adapters-from-project = Laad ook de adapters in de map `adapters` van de projectmap. freemapper noemt elk ervan met de hosts die hij aanspreekt. Gebruik het voor een project dat je vertrouwt: een adapter laat freemapper de servers aanspreken die hij noemt

## The summary of a run, for the MCP server and the area page

cli-internal-error = interne fout: { $message }
cli-pdf-sources-invalid = pdf_sources moet auto, full of credit-only zijn, niet { $value }
cli-view-unknown = een weergave is no-paths, brown-only, no-north-lines, no:<codes> of only:<codes>, niet { $value }
cli-view-empty = de lijst { $value } noemt geen symboolcode
cli-view-bad-code = { $value } is geen symboolcode zoals 505 of 501.6
cli-summary-training-view = { $map } en { $pdf } geschreven (een weergave van de trainingskaart)
cli-summary-kept-training-view = De bestanden van de weergave { $view } van de trainingskaart behouden; geef --force mee om ze te vernieuwen
cli-summary-training-view-failed = De weergave van de trainingskaart is mislukt: { $error }
cli-estimate-disk-unreadable = kan de vrije schijfruimte van { $path } niet lezen ({ $message })
cli-summary-wrote-map = De kaart { $path } geschreven
cli-summary-kept-map = De bestaande kaart { $path } behouden (ze kan het werk van de kaartmaker bevatten)
cli-summary-map-lacks-templates = De kaart verwijst nog niet naar de beeldtemplates. Start de pipeline met force om haar te vervangen en de oude te bewaren als .omap.bak.
cli-summary-map-lacks-buildings = De kaart heeft nog geen ALKIS-gebouwen. Start de pipeline met force om ze toe te voegen; ze vervangt de kaart en bewaart de oude als .omap.bak.
cli-summary-map-lacks-canopies = De kaart mist de ALKIS-overkappingen en carports (522), die nu kaartobjecten zijn. Start de pipeline met force om ze toe te voegen; ze vervangt de kaart en bewaart de oude als .omap.bak.
cli-summary-map-light = De basiskaart: { $joined } lijnen samengevoegd, { $left } verborgen of te klein voor de kandidaatlagen overgelaten (niets verschoven)
cli-summary-map-light-kept = De kandidaatlagen bevatten nu ook { $left } objecten of stukken die deze versie van de basiskaart haalt (met de tag freemapper:generalized). De behouden kaart heeft ze nog, dus ze staan er dubbel op. Start de pipeline met force om de kaart op te schonen; ze bewaart de oude als .omap.bak.
cli-summary-map-old-candidates = De kaart noemt nog templates/candidates.omap; deze run heeft die met alle kandidaten gevuld. De kandidaten komen nu ook in zeven lagen, elk in een eigen bestand (templates/candidates-*.omap). De kaart toont ze na een run met force (de oude kaart blijft als .omap.bak), of de kaartmaker voegt ze toe in OOM (Templates > Open template…).
cli-summary-wrote-ocad = De OCAD-kaart { $path } geschreven ({ $count } objecten)
cli-summary-kept-ocad = De bestaande OCAD-kaart { $path } behouden (ze kan het werk van de kaartmaker bevatten; force vervangt haar)
cli-summary-ocad-old-candidates = De OCAD-kaart noemt { $path } nog als achtergrondkaart; deze run heeft die met alle kandidaten gevuld. De kandidaten komen nu ook in zeven lagen, elk in een eigen bestand (-candidates-*.ocd). De OCAD-kaart toont ze na een run met force (de oude kaart blijft als .ocd.bak), of de kaartmaker voegt ze in OCAD toe als achtergrondkaarten.
cli-summary-ocad-left-out = Weggelaten uit de OCAD-kaart: { $skipped } objecten en { $dropped } ontaarde delen
cli-summary-ocad-symbols = Kies in OCAD Symbol > Load Colors and Symbols From… en kies de officiële symboolset{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , de IOF-schoolset 2019
       *[other] {" "}ISSprOM 2019
    }; die vervangt de plaatsvervangende symbolen op nummer.
cli-summary-ocad-failed = OCAD-export mislukt: { $error }
cli-summary-pdf-without-credits = PDF zonder bronvermelding: ODbL en dl-de/by-2-0 vragen je de bronnen te noemen als je de kaart doorgeeft; zie ATTRIBUTION.txt
cli-summary-wrote-pdf = De kaart { $path } geschreven (zonder kandidaten)
cli-summary-pdf-failed = PDF-export mislukt: { $error }
cli-summary-wrote-pdf-candidates = De controleprint { $path } geschreven (de kaart met elke kandidaat, op de juiste plaats)
cli-summary-pdf-candidates-failed = Export van de controleprint mislukt: { $error }
cli-summary-training-failed = Export van de trainingskaart mislukt: { $error }
cli-summary-kept-training = De bestaande trainingskaart { $path } behouden, zodat een run er geen overschrijft die is afgedrukt of doorgegeven; verwijder haar of zet force om haar te vernieuwen

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = geen thuismap op dit systeem; gebruik een absoluut pad
mcp-error-relative-path = { $path } is een relatief pad; gebruik een absoluut pad of een pad dat met ~ begint
mcp-error-home-folder = { $path } is de thuismap zelf; gebruik een map daarbinnen, bijv. ~/maps/school
mcp-error-no-project = geen freemapper-project in { $path }; gebruik eerst create_project
mcp-error-unknown-kind = onbekend kaarttype { $kind }; gebruik "schoolyard", "sprint" of "forest"
mcp-error-reading = { $path } lezen: { $error }
mcp-error-one-area-source = geef precies een van osm_id (uit find_place) of geojson
mcp-error-page-open = de gebiedskeuzepagina van dit project is nog open; vraag de gebruiker het gebied op te slaan en op Klaar te drukken, start daarna de pipeline
mcp-error-no-job = geen job { $job }; start_pipeline en permits_for_area geven de job-id's van deze sessie terug
mcp-error-job-running = { $job } loopt nog voor { $path }; er loopt één pipeline tegelijk omdat projecten de downloadcache delen. Start deze als project_status finished of failed meldt
mcp-error-page-start = kan de pagina niet starten: { $error }
mcp-created-project = Project { $path } aangemaakt voor { $area } ({ $kind }, 1:{ $scale }). Daarna: estimate_download, dan start_pipeline.
mcp-created-project-given-area = Project { $path } aangemaakt voor het opgegeven gebied ({ $kind }, 1:{ $scale }). Daarna: estimate_download, dan start_pipeline.
mcp-page-already-open = De pagina voor deze map is al open; ze houdt de instellingen waarmee ze is geopend.
mcp-page-next = Vraag de gebruiker het gebied op de pagina te kiezen, op Opslaan te drukken en dan op Klaar.
mcp-preview-of = Voorvertoning van { $layer }
mcp-permits-next = De vergunningsnotities worden gemaakt. Poll project_status met deze job_id; als hij finished is, bevat permits_md de tekst van permits.md. Geef die door als hint, niet als juridisch advies.
mcp-tool-check-update = Of er een nieuwere freemapper-release bestaat: zijn versie, releasedatum, release-notes en downloadgrootte, of dat deze actueel is. Installeert niets.
mcp-tool-create-project = Een projectmap voor een kaartgebied aanmaken. Geef osm_id (uit find_place) of een GeoJSON-bestand. kind is schoolyard (1:1000, standaard), sprint (1:4000) of forest (1:15000).
mcp-param-create-project-dir = Nieuwe projectmap (absoluut pad of beginnend met ~).
mcp-param-create-project-geojson = In plaats van osm_id: een GeoJSON-bestand met de polygoon van het kaartgebied (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (standaard, 1:1000), "sprint" (1:4000) of "forest" (1:15000).
mcp-param-create-project-osm-id = OSM-object van het kaartgebied uit find_place, bijv. "way/992423119".
mcp-param-create-project-part = Met osm_id: welk deel van een gebied van meerdere afzonderlijke delen te gebruiken (1 is het grootste).
mcp-param-create-project-scale = Schaalnoemer: schoolyard 1000 (standaard); sprint 4000 (standaard) of 5000; forest 15000 (standaard), 10000 of 7500. Andere waarden worden geweigerd; teken voor 1:3000 op 4000, freemapper-print vergroot bij het afdrukken.
mcp-param-create-project-title = Kaarttitel voor de randinformatie (standaard: de mapnaam).
mcp-tool-estimate-download = Wat start_pipeline voor een project zal downloaden: de tegels, elke download met zijn grootte in MB, of hij in de cache staat en of hij op de schijf past, de nog te downloaden MB, en onder disk de vrije ruimte van de cache- en projectvolumes met meldingen over wat niet past (vertel dat de gebruiker).
mcp-param-estimate-download-dir = Projectmap (absoluut pad of beginnend met ~).
mcp-tool-find-place = Een school, park of andere plaats op naam zoeken (OpenStreetMap Nominatim). Geeft tot 5 treffers terug, gebieden eerst; alleen treffers met has_area kunnen een kaartgebied zijn. Een lege lijst betekent dat er niets is gevonden.
mcp-param-find-place-query = Naam van de school of plaats, bijv. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = De nieuwste freemapper-release over dit programma heen installeren. Roep dit pas aan nadat de persoon akkoord is gegaan, met confirm true. De nieuwe versie geldt zodra de assistent de freemapper-server opnieuw start.
mcp-param-install-update-confirm = true zodra de persoon akkoord is gegaan met het installeren van de nieuwere versie; zonder wordt niets geïnstalleerd.
mcp-tool-list-layers = De lagen van een project: de kaart, de zeven kandidaatlagen en de rastertemplates, elk met zijn bestand, of het al is gemaakt en welke stap het maakt.
mcp-param-list-layers-dir = Projectmap (absoluut pad of beginnend met ~).
mcp-tool-open-area-selector = Een pagina in de browser openen waar de gebruiker het kaartgebied tekent (rechthoek of polygoon) of een plaats kiest, de tegels en de downloadgrootte ziet en het opslaat; opslaan maakt de projectmap aan of wijzigt haar gebied. Geeft de URL terug die je de gebruiker geeft. De pagina draait tot de gebruiker op Klaar drukt; opnieuw aanroepen voor dezelfde map geeft dezelfde URL terug.
mcp-param-open-area-selector-dir = Projectmap (absoluut pad of beginnend met ~); wordt aangemaakt als de gebruiker opslaat.
mcp-param-open-area-selector-kind = Kaarttype van een nieuw project: "schoolyard" (standaard, 1:1000), "sprint" (1:4000) of "forest" (1:15000).
mcp-param-open-area-selector-ocad = Het runpaneel start met het OCAD-startproject aangevinkt. Standaard false.
mcp-param-open-area-selector-offline = Het runpaneel start met "alleen cache, geen netwerk" aangevinkt. Standaard false.
mcp-param-open-area-selector-open-browser = Open de pagina ook in de standaardbrowser van deze computer. Standaard true.
mcp-param-open-area-selector-pdf = Het runpaneel start met de afdrukbare PDF aangevinkt. Standaard false.
mcp-param-open-area-selector-pdf-candidates = Het runpaneel start met de controleprint (elke kandidaat) aangevinkt. Standaard false.
mcp-param-open-area-selector-pdf-scale-bar = Het runpaneel start met de schaalbalk op de PDF's aangevinkt. Standaard false.
mcp-param-open-area-selector-pdf-sources = Bronregels op de PDF's waarmee het paneel start: "auto" (standaard), "full" of "credit-only", zoals bij start_pipeline.
mcp-param-open-area-selector-permits = Het runpaneel start met de vergunningsnotities aangevinkt. Standaard false.
mcp-param-open-area-selector-scale = Schaalnoemer van een nieuw project: schoolyard 1000 (standaard); sprint 4000 (standaard) of 5000; forest 15000 (standaard), 10000 of 7500. Andere waarden worden geweigerd.
mcp-param-open-area-selector-title = Kaarttitel van een nieuw project.
mcp-param-open-area-selector-training = Het runpaneel start met de trainingskaart aangevinkt. Standaard false.
mcp-tool-preview-layer = Een PNG-voorvertoning (hooguit 800 px) van een laag: map (de afbeelding van de kaartobjecten die de laatste run schreef, zonder de kandidaten), of een rastertemplate: orthophoto, hillshade, slope, relief, ndsm, of k5 in Berlijn.
mcp-param-preview-layer-dir = Projectmap (absoluut pad of beginnend met ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" of "k5" (alleen Berlijn).
mcp-tool-project-status = Toestand van een job van start_pipeline of permits_for_area: running, finished of failed, de laatste voortgangsregels, waarschuwingen, de geschreven bestanden, en het resultaat of de fout. Een afgeronde permits_for_area-job bevat ook de tekst van permits.md als permits_md.
mcp-param-project-status-job-id = Job-id van start_pipeline of permits_for_area, bijv. "job-1".
mcp-tool-permits-for-area = De vergunningsnotities van een project maken: de bestuurlijke eenheden van het kaartgebied, de beschermde gebieden erin en wie het land daar beheert, met de instanties die je kunt vragen; een hint, geen juridisch advies. Schrijft permits.md en permits.json in de map. Draait op de achtergrond en geeft een job_id terug; poll project_status, waarvan het antwoord de tekst van permits.md als permits_md bevat als de job klaar is. Er loopt één job tegelijk.
mcp-param-permits-for-area-dir = Projectmap (absoluut pad of beginnend met ~).
mcp-param-permits-for-area-offline = Alleen de gegevens lezen die een eerdere run in het project heeft opgeslagen, zonder netwerktoegang. Standaard false.
mcp-tool-start-pipeline = De gegevens downloaden, de templates berekenen en het startproject schrijven (fetch, derive, export). Draait op de achtergrond en geeft een job_id terug; poll project_status. Er loopt één pipeline tegelijk. Ze behoudt een bestaande kaart tenzij force true is.
mcp-param-start-pipeline-dir = Projectmap (absoluut pad of beginnend met ~).
mcp-param-start-pipeline-force = Een bestaand kaartbestand vervangen en het oude bewaren als .omap.bak. Standaard false.
mcp-param-start-pipeline-ocad = Schrijf ook een OCAD-startpunt in de map ocad/ (OCAD 12-formaat, voor gebruikers die in OCAD bewerken). Standaard false.
mcp-param-start-pipeline-offline = Alleen de downloadcache gebruiken, zonder netwerktoegang. Standaard false.
mcp-param-start-pipeline-pdf = Schrijf ook <name>.pdf, de kaart klaar om af te drukken (alleen de kaartobjecten, geen kandidaten). Standaard false.
mcp-param-start-pipeline-pdf-candidates = Schrijf ook <name>-candidates.pdf, de controleprint: de kaart met elke kandidaat, op de juiste plaats (controleer ze in het terrein). Standaard false.
mcp-param-start-pipeline-pdf-scale-bar = De schaalbalk op de PDF's tekenen. Standaard false (Duitse kaarten hebben er meestal geen; het schaalgetal blijft in het titelblok).
mcp-param-start-pipeline-pdf-sources = Bronregels op de PDF's: "auto" (standaard; de volledige lijst, alleen ingekort als ze meer dan 5 regels inneemt), "full" (nooit ingekort) of "credit-only" (alleen de bronvermelding van freemapper; ODbL en dl-de/by-2-0 vereisen dan dat de bronnen worden genoemd als de kaart wordt doorgegeven). ATTRIBUTION.txt vermeldt altijd elke bron.
mcp-param-start-pipeline-permits = Schrijf na de kaart ook de vergunningsnotities (permits.md en permits.json): bestuurlijke eenheden, beschermde gebieden en wie het land beheert. Een mislukking daarbij is een waarschuwing, geen mislukte job. Standaard false.
mcp-param-start-pipeline-training = Schrijf ook de trainingskaart <name>-training.omap en haar PDF <name>-training.pdf: de kaart en elke kandidaat, opgeschoond om ze zo af te drukken; met ocad ook ocad/<name>-training.ocd. Een bestaande trainingskaart blijft staan tenzij force true is. Standaard false.
mcp-param-start-pipeline-training-views = Met training: schrijf ook de trainingskaart zonder wat een weergave weglaat, een bestandenset per item: "no-paths", "brown-only", "no-north-lines", of een lijst zoals "no:505,506" of "only:101,102", gecombineerd met +. De bestanden zijn <name>-training-<view>.omap en haar PDF.
mcp-instructions = freemapper maakt OpenOrienteering Mapper-startprojecten voor orienteringskaarten (schoolplein 1:1000, sprint 1:4000, bos 1:15000) uit officiële geodata van { $states }, en uit OSM. Werkwijze: find_place (zoek de school of het gebied op naam; kies een treffer met has_area) → create_project (een nieuwe map, absoluut pad, met de osm_id) → estimate_download (vertel de gebruiker hoeveel er wordt gedownload) → start_pipeline (geeft een job-id terug; downloaden kan minuten duren) → project_status (poll om de paar seconden tot de toestand finished of failed is) → list_layers en preview_layer (toon de gebruiker de templates). De gebruiker opent daarna het kaartbestand in OpenOrienteering Mapper. Als geen plaats past of de gebruiker het gebied wil tekenen, opent open_area_selector een pagina in de browser waar de gebruiker het gebied tekent of kiest en opslaat (dat maakt het project aan); ga verder met estimate_download zodra de gebruiker zegt dat het is opgeslagen. Paden moeten absoluut zijn of met ~ beginnen. Er loopt één pipeline tegelijk. permits_for_area maakt de vergunningsnotities van een project (bestuurlijke eenheden, beschermde gebieden, wie het land beheert; een hint, geen juridisch advies) als job zoals start_pipeline. check_update meldt of er een nieuwere freemapper is; install_update installeert hem, alleen met confirm: true nadat de persoon akkoord is gegaan, en de assistent start daarna de freemapper-server opnieuw.

cli-contour-interval-saved = Hoogtelijninterval opgeslagen in freemapper.toml: { $interval } m; derive trekt de hoogtelijnen daarmee
cli-contour-interval-auto = Hoogtelijninterval in freemapper.toml: automatisch (dat van het kaarttype; een boskaart krijgt 2,5 m in vlak land en 10 m in steil land)
cli-steep-slope-saved = Steilheidsdrempel opgeslagen in freemapper.toml: { $slope }°; derive zoekt er taluds en rotswanden mee
cli-steep-slope-auto = Steilheidsdrempel in freemapper.toml: standaard (ongeveer 27°)
cli-face-slope-saved = Wanddrempel opgeslagen in freemapper.toml: { $slope }°; derive zoekt er rotswandvlakken mee
cli-face-slope-auto = Wanddrempel in freemapper.toml: standaard (70°)
help-arg-contour-interval = Hoogtelijninterval in meters, bijv. `2.5`, of `auto` voor dat van het kaarttype (een boskaart: 5 m, 2,5 m waar het vlak is, 10 m waar het steil is). Opgeslagen in freemapper.toml; derive trekt de hoogtelijnen daarmee
help-arg-steep-slope = Hellingshoek in graden vanaf welke het terrein als steil geldt bij het zoeken naar taluds en rotswanden, bijv. `55`, of `auto` voor de standaard van 27°. Verhoog hem (55 tot 65) in rotssteden en kloven, waar bijna elke helling steil is en taluds en rotswanden samenvloeien tot één vlek. Opgeslagen in freemapper.toml; derive zoekt er taluds en rotswanden mee
help-arg-face-slope = Hellingshoek in graden vanaf waar het terrein een rotswand is (zwart vlak, symbool 206), bijv. `75`, of `auto` voor de standaard van 70°. 65 geeft meer en grotere vlakken, ook op steile hellingen; 75 houdt alleen echte wanden over. Opgeslagen in freemapper.toml; derive zoekt er rotswanden mee
cli-content-saved = Inhoudsinstelling opgeslagen in freemapper.toml: { $summary }
cli-content-invalid = Kan de inhoudsinstelling niet lezen: { $detail }
help-arg-content = Wat direct op de kaart komt: `safe` laat elk object waar de build het zet, `balanced` (standaard) zet de objecten van het kadaster en de registers op de kaart, `full` zet alles op de kaart. De keuze wordt opgeslagen in freemapper.toml
help-arg-content-set = Een groep objecten naar `map`, `candidates` of `off` sturen, als `GROUP=PLACE`, bijv. `walls=map`; mag worden herhaald. Groepen: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Opgeslagen in freemapper.toml
mcp-param-create-project-content = Wat direct op de kaart komt: "safe", "balanced" (standaard) of "full". Opgeslagen in het project. balanced zet de objecten van het kadaster en de registers op de kaart, full zet alles erop.
mcp-param-create-project-content-set = Groepen die ergens anders heen gaan dan het profiel zegt, als "group=place" met place map, candidates of off, bijv. "walls=map". Groepen: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Hoogtelijninterval in meters, bijv. 2.5; 0 gaat terug naar de automatische keuze (die van het kaarttype; een boskaart krijgt 2,5 m in vlak land en 10 m in steil land). Opgeslagen in het project; de volgende derive trekt de hoogtelijnen daarmee.
mcp-param-create-project-steep-slope = Hellingshoek in graden vanaf welke het terrein als steil geldt bij het zoeken naar taluds en rotswanden, bijv. 55; 0 gaat terug naar de standaard van 27°. Verhoog hem (55 tot 65) in rotssteden en kloven, waar bijna elke helling steil is en taluds en rotswanden samenvloeien tot één vlek. Opgeslagen in het project; de volgende derive zoekt er taluds en rotswanden mee.
mcp-param-create-project-face-slope = Hellingshoek in graden vanaf waar het terrein een rotswand is (zwart vlak, symbool 206), bijv. 75; 0 gaat terug naar de standaard van 70°. 65 geeft meer en grotere vlakken, ook op steile hellingen; 75 houdt alleen echte wanden over. Opgeslagen in het project; de volgende derive zoekt er rotswanden mee.
mcp-param-start-pipeline-content = Wijzig voor de run wat direct op de kaart komt: "safe", "balanced" of "full". Opgeslagen in het project. Standaard: de instelling van het project, anders balanced.
mcp-param-start-pipeline-content-set = Groepen die ergens anders heen gaan dan het profiel zegt, als "group=place" met place map, candidates of off, bijv. "fences=candidates". Opgeslagen in het project.
mcp-param-start-pipeline-contour-interval = Hoogtelijninterval in meters, bijv. 2.5; 0 gaat terug naar de automatische keuze (die van het kaarttype; een boskaart krijgt 2,5 m in vlak land en 10 m in steil land). Opgeslagen in het project; de volgende derive trekt de hoogtelijnen daarmee.
mcp-param-start-pipeline-steep-slope = Hellingshoek in graden vanaf welke het terrein als steil geldt bij het zoeken naar taluds en rotswanden, bijv. 55; 0 gaat terug naar de standaard van 27°. Verhoog hem (55 tot 65) in rotssteden en kloven, waar bijna elke helling steil is en taluds en rotswanden samenvloeien tot één vlek. Opgeslagen in het project; de volgende derive zoekt er taluds en rotswanden mee.
mcp-param-start-pipeline-face-slope = Hellingshoek in graden vanaf waar het terrein een rotswand is (zwart vlak, symbool 206), bijv. 75; 0 gaat terug naar de standaard van 70°. 65 geeft meer en grotere vlakken, ook op steile hellingen; 75 houdt alleen echte wanden over. Opgeslagen in het project; de volgende derive zoekt er rotswanden mee.
mcp-error-content = Kan de inhoudsinstelling niet lezen: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = De vergunningsnotities van een project schrijven (permits.md en permits.json): de bestuurlijke eenheden, beschermde gebieden en beheerders die het kaartgebied raakt, en wie je daarvoor meestal moet vragen. Het vraagt de BfN en de BKG en, waar de deelstaat van het kaartgebied eigen open services heeft, die via de hosts van zijn adapter (lijst in de handleiding, hoofdstuk 12); in Noordrijn-Westfalen downloadt het hiervoor de eerste keer ongeveer 32 MB, in Brandenburg ongeveer 4 MB, later alleen nieuwere bestanden. Het leest het al gecachete OSM-extract; --offline gebruikt wat eerder is opgehaald
help-permits-dir = Projectmap
help-permits-offline = Alleen de eerder opgehaalde gegevens gebruiken, zonder netwerktoegang
help-run-permits = Schrijf na de kaart ook de vergunningsnotities (permits.md en permits.json); vraagt de BfN, de BKG en de open services van de deelstaat (handleiding, hoofdstuk 12). Een mislukking daarbij is een waarschuwing en de kaart blijft
cli-wrote-permits = De vergunningsnotities { $path } geschreven
cli-summary-wrote-permits = De vergunningsnotities { $path } geschreven
cli-permits-failed = de vergunningsnotities konden niet worden geschreven: { $error }
cli-permits-source-left-out = de vergunningsnotities laten { $source } weg: { $reason }
cli-permits-not-read = het is niet gelezen
error-permits-stored = { $path }: { $message }; haal de vergunningsnotities opnieuw online op
error-permits-nothing-read = geen bron van de vergunningsnotities kon worden gelezen ({ $missing }); haal ze eerst online op
error-permits-missing-not-read = { $source }: niet gelezen

permits-title = Vergunningsnotities
permits-disclaimer = Een hint, geen juridisch advies. De bevoegde instanties beslissen. Zie voor de datum en nauwkeurigheid van de gegevens de tabel onder "Bronnen en gegevensdatum"; gebieden kunnen ontbreken of verouderd zijn (BfN: eens per jaar samengevoegd, "Nicht für Planungszwecke geeignet.", dat wil zeggen niet geschikt voor planning).
permits-disclaimer-regional = Een hint, geen juridisch advies. De bevoegde instanties beslissen. Zie voor de datum en nauwkeurigheid van de gegevens de tabel onder "Bronnen en gegevensdatum"; gebieden kunnen ontbreken of verouderd zijn.
permits-bfn-merge = De BfN voegt de gegevens van de deelstaten eens per jaar samen en markeert ze als niet geschikt voor planning. De instanties van de deelstaat beslissen.
permits-created = Gemaakt op { $date } voor het kaartgebied van een { $kind ->
        [schoolyard] schoolpleinkaart
        [forest] boskaart
       *[other] sprintkaart
    } ({ $area } ha).
permits-section-admin = Bestuurlijke eenheden
permits-section-protected = Beschermde gebieden (BfN)
permits-section-osm-protected = Meer beschermde gebieden uit OpenStreetMap
permits-section-operators = Beheerders en eigenaren volgens OpenStreetMap
permits-section-hints = Hints
permits-section-sources = Bronnen en gegevensdatum
permits-none-admin = VG250 noemt geen bestuurlijke eenheid voor het kaartgebied.
permits-none-protected = Volgens de BfN ligt er geen beschermd gebied in het kaartgebied.
permits-none-osm-protected = OpenStreetMap noemt in het kaartgebied geen beschermd gebied dat de BfN nog niet vermeldt.
permits-none-operators = OpenStreetMap noemt geen beheerder en geen eigenaar voor het kaartgebied.
permits-none-hints = Geen hints.
permits-section-not-read = De gegevens van { $source } zijn niet gelezen, dus deze sectie is leeg, hoewel er vermeldingen kunnen zijn.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Bestuurlijke gebieden 1:250 000 (VG250) van de BKG
        [bfn] Beschermde gebieden in Duitsland van de BfN
       *[osm] OpenStreetMap, uit het extract dat freemapper in zijn cache bewaart
    }
permits-level = { $level ->
        [land] Deelstaat
        [kreis] District (Kreis)
        [verwaltungsgemeinschaft] Amt of Verwaltungsgemeinschaft
       *[gemeinde] Gemeente (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Aan de rand geraakt (minder dan 0,5% van het kaartgebied):
permits-at-edge-generalised = VG250 is gegeneraliseerd naar schaal 1:250 000. Een eenheid die alleen de rand raakt kan daar een gevolg van zijn.
permits-kind = { $kind ->
        [nationalpark] Nationaal park
        [nationales-naturmonument] Nationaal natuurmonument
        [naturschutzgebiet] Natuurreservaat (Naturschutzgebiet)
        [ffh] FFH-gebied (Natura 2000)
        [vogelschutzgebiet] Vogelbeschermingsgebied (Natura 2000)
        [biosphaerenreservat] Biosfeerreservaat
        [biosphaerenreservat-zone] Zone van een biosfeerreservaat
        [landschaftsschutzgebiet] Landschapsbeschermingsgebied (Landschaftsschutzgebiet)
       *[naturpark] Natuurpark
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Aangewezen op { $date }
permits-protected-title = Rechtsgrondslag: { $title }
permits-protected-link = Wettekst: { $url }
permits-protected-site-link = Gebiedspagina (Natura 2000): { $url }
permits-protected-year = Gegevensjaar bij de BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = zonder naam
permits-osm-operator = Beheerder volgens OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap-gebieden die het hele kaartgebied omsluiten zonder een hoek erin kunnen hier ontbreken, en ook beschermde gebieden die in OSM als relatie met `type=boundary` zijn getekend. De delen van de BfN en VG250 worden hierdoor niet getroffen.
permits-osm-enclosing-gap-short = OpenStreetMap-vlakken die het hele kaartgebied omsluiten zonder een hoek erin te hebben, kunnen hier ontbreken.
permits-operator-line = { $name } ({ $roles ->
        [operator] beheerder
        [owner] eigenaar
       *[both] beheerder en eigenaar
    }): { $objects }
permits-column-source = Bron
permits-column-state = Status
permits-column-credit = Bronvermelding en licentie
permits-state-read = gelezen, opgehaald op { $date }
permits-state-read-osm = gelezen, OSM-extract van { $date }
permits-state-unreachable = niet bereikbaar ({ $reason })
permits-state-not-read = niet gelezen
permits-no-credit = niet gebruikt
permits-hint-city-state = Het kaartgebied ligt in Berlijn of Hamburg. Daar is meestal het Bezirksamt (districtsbestuur) of een instantie van de deelstaat verantwoordelijk. Deze notities bepalen het Bezirk nog niet.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Beschermde gebieden vallen onder de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde). Dat is hier de stad { $kreis } zelf.
        [no] Beschermde gebieden vallen onder de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde), meestal de { $kreis }.
       *[unknown] Beschermde gebieden vallen onder de bevoegde lagere natuurbeschermingsinstantie (untere Naturschutzbehörde), meestal de Landkreis of de kreisfreie Stadt.
    }
permits-hint-national-park = Een nationaal park heeft eigen regels. Vraag de parkbeheerder vroeg.
permits-hint-nsg-paths = In een natuurreservaat moet je meestal op de paden blijven. Vraag voor een loop buiten de paden de natuurbeschermingsinstantie.
permits-hint-natura2000 = Een FFH-gebied of een vogelbeschermingsgebied (Natura 2000) raakt het kaartgebied. Voor een evenement kan de natuurbeschermingsinstantie een beoordeling van de effecten vragen.
permits-hint-protected-areas-pl = In het kaartgebied liggen beschermde gebieden of beschermde natuurobjecten. Bij een nationaal park, een landschapspark of een natuurreservaat is de beheerder van het park of reservaat meestal het aanspreekpunt; voor de overige gebieden en objecten kan de Regionale Directie voor Milieubescherming (RDOŚ) van het woiwodschap meestal zeggen wie bevoegd is. Vraag het op tijd als daar een evenement gepland wordt.
permits-hint-order-office = { $amt ->
        [none] Voor straten, pleinen en andere openbare grond is meestal het Ordnungsamt (dienst openbare orde) van de Gemeinde { $gemeinde } verantwoordelijk.
       *[other] Voor straten, pleinen en andere openbare grond is meestal het Ordnungsamt (dienst openbare orde) verantwoordelijk: { $amt }, voor de Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Beheerder van een park of tuin in het kaartgebied volgens OpenStreetMap: { $operator }. Vraag hen voordat je het gebruikt.
permits-hint-school-operator = Beheerder van de school of instelling volgens OpenStreetMap: { $operator }. Vraag voor het terrein de schoolleiding of deze instantie.
permits-section-districts = Bezirke (districten, { $source })
permits-section-green-spaces = Groenvoorzieningen ({ $source })
permits-section-owners = Grond van de deelstaat en van de Bund ({ $source })
permits-section-monuments = Monumenten ({ $source })
permits-section-protected-objects = Natuurmonumenten en andere beschermde objecten ({ $source })
permits-none-districts = Volgens { $source } ligt het kaartgebied in geen Bezirk.
permits-none-green-spaces = Volgens { $source } ligt er geen openbare groenvoorziening in het kaartgebied.
permits-none-owners = Volgens { $source } ligt er geen perceel van de deelstaat of van de Bund in het kaartgebied.
permits-none-monuments = Volgens { $source } ligt er geen monument in het kaartgebied.
permits-none-protected-objects = Volgens { $source } ligt geen van de beschermde objecten die het bevat in het kaartgebied.
permits-none-sampled = Op de gevraagde punten noemt { $source } hier niets.
permits-gaps = De gegevens van { $source } zijn niet compleet: { $gaps }.
permits-sampled = { $source } antwoordt alleen voor afzonderlijke punten. freemapper heeft op een raster van punten over de kaart gevraagd: de aandelen zijn schattingen en wat tussen de punten ligt kan ontbreken.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = bestemming: { $dedication }
permits-playgrounds-none = Er ligt geen openbare speeltuin in het kaartgebied.
permits-playgrounds-one = Er ligt één openbare speeltuin in het kaartgebied. Hij staat niet apart vermeld.
permits-playgrounds-many = Er liggen { $count } openbare speeltuinen in het kaartgebied. Ze staan niet afzonderlijk vermeld.
permits-owner-other = Andere openbare eigenaren
permits-owners-rest = Voor de rest van het gebied noemen de gegevens geen eigenaar.
permits-monuments-total = Monumenten geheel of gedeeltelijk in het kaartgebied: { $count }.{ $links ->
        [yes] {" "}De links leiden naar hun vermelding in de monumentenlijst.
       *[no] {""}
    }
permits-monuments-links-left = { $count } vermeldingen meer worden hier alleen geteld, zonder link.
permits-object-kind = { $kind ->
        [natural-monument] Natuurmonument (Naturdenkmal)
        [protected-landscape-part] Beschermd landschapselement (geschützter Landschaftsbestandteil)
        [biotope] Wettelijk beschermd biotoop (geschütztes Biotop)
        [national-park] Nationaal park (park narodowy)
        [park-buffer-zone] Bufferzone van een park (otulina)
        [landscape-park] Landschapspark (park krajobrazowy)
        [nature-reserve] Natuurreservaat (rezerwat przyrody)
        [protected-landscape-area] Beschermd landschapsgebied (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000-gebied voor habitats (SOO)
        [natura2000-birds] Natura 2000-gebied voor vogels (OSO)
        [ecological-site] Ecologisch gebied (użytek ekologiczny)
        [landscape-complex] Natuur- en landschapscomplex (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Documentatiesite (stanowisko dokumentacyjne)
        [natural-monument-pl] Natuurmonument (pomnik przyrody)
        [national-park-cz] Nationaal park (národní park, NP)
        [protected-landscape-area-cz] Beschermd landschapsgebied (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Nationaal natuurreservaat (národní přírodní rezervace, NPR)
        [national-nature-monument] Nationaal natuurmonument (národní přírodní památka, NPP)
        [nature-reserve-cz] Natuurreservaat (přírodní rezervace, PR)
        [nature-monument-cz] Natuurmonument (přírodní památka, PP)
        [protective-zone-cz] Beschermingszone (ochranné pásmo)
        [nature-zone-cz] Zone voor natuurbescherming (zóna ochrany přírody)
        [natura2000-habitats-cz] Natura 2000-gebied voor habitats (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Natura 2000-gebied voor vogels (ptačí oblast, PO)
        [memorial-tree] Gedenkboom (památný strom)
       *[special-rules] Gebied met bijzondere regels
    }
permits-object-count = { $count } objecten
permits-object-citations = Rechtsgrondslagen:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Voor straten, pleinen en groenvoorzieningen in het Bezirk { $district } is meestal het Bezirksamt { $district } verantwoordelijk, daar het Fachamt Management des öffentlichen Raumes (dienst voor de openbare ruimte).
       *[berlin] Voor straten, pleinen en groenvoorzieningen in het Bezirk { $district } zijn meestal het Ordnungsamt (dienst openbare orde) en het Straßen- und Grünflächenamt (dienst wegen en groen) van het Bezirksamt { $district } verantwoordelijk.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Voor de monumenten in het kaartgebied is het Denkmalschutzamt Hamburg (monumentendienst) verantwoordelijk.
       *[berlin] Voor de monumenten in het Bezirk { $district } is de lagere monumentenautoriteit (untere Denkmalschutzbehörde) van het Bezirksamt { $district } verantwoordelijk.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Of er beschermde monumenten in het kaartgebied liggen, kunnen de gegevens hier niet met zekerheid zeggen.
       *[yes] Er zijn beschermde monumenten in het kaartgebied.
    } Of een loop ze raakt, bijvoorbeeld met een controlepost bij een monument of een traject door een beschermd terrein, beslist de lagere monumentenautoriteit (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Dat is hier de stad { $kreis } zelf.
        [no] , meestal de { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; in sommige deelstaten is dat de Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(meestal de Landkreis of de kreisfreie Stadt, in sommige deelstaten de Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Natuurmonumenten, beschermde landschapselementen en beschermde biotopen op de kaart zijn een zaak van de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde). Dat is hier de stad { $kreis } zelf.
        [no] Natuurmonumenten, beschermde landschapselementen en beschermde biotopen op de kaart zijn een zaak van de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde), meestal de { $kreis }.
       *[unknown] Natuurmonumenten, beschermde landschapselementen en beschermde biotopen op de kaart zijn een zaak van de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde), meestal de Landkreis of de kreisfreie Stadt.
    }
permits-hint-biotope = Wettelijk beschermde biotopen liggen in het kaartgebied{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Alles wat ze kan vernietigen of aanzienlijk kan schaden is verboden (§ 30 BNatSchG). Plaats controleposten en trajecten erbuiten, of vraag de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Voor natuurmonumenten en beschermde landschapselementen in het Bezirk { $district } is meestal het Bezirksamt { $district } verantwoordelijk.
       *[berlin] Voor natuurmonumenten en beschermde landschapselementen in het Bezirk { $district } is de lagere natuurbeschermingsinstantie (untere Naturschutzbehörde) van het Bezirksamt { $district } verantwoordelijk.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Volgens de open gegevens van de deelstaat is { $share } van het kaartgebied eigendom van andere openbare eigenaren.
       *[no] Volgens de open gegevens van de deelstaat is { $share } van het kaartgebied eigendom van de eigenaar "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] De eigenaren van de rest van het gebied zijn niet uit open gegevens te achterhalen. Met een gerechtvaardigd belang (bijvoorbeeld als organisator) kun je het kadaster (Grundbuchamt) om een uittreksel vragen; voor openbare grond helpt het bevoegde Ordnungsamt of de parkendienst.
       *[no] De eigenaren van deze gebieden zijn niet uit open gegevens te achterhalen. Met een gerechtvaardigd belang (bijvoorbeeld als organisator) kun je het kadaster (Grundbuchamt) om een uittreksel vragen; voor openbare grond helpt het bevoegde Ordnungsamt of de parkendienst.
    }
permits-section-forest-units = Bosbeheer ({ $source })
permits-section-state-forest = Staatsbos ({ $source })
permits-none-forest-units = Volgens { $source } ligt het kaartgebied in het gebied van geen bosbeheerkantoor.
permits-section-admin-units = Bestuurlijke eenheden ({ $source })
permits-none-admin-units = Volgens { $source } ligt het kaartgebied in geen enkele bestuurlijke eenheid van het register.
permits-section-register-monuments = Monumenten van het nationale register ({ $source })
permits-none-register-monuments = Volgens { $source } ligt er geen ingeschreven monument in de rechthoek rond het kaartgebied.
permits-register-monuments-total = Ingeschreven monumenten in de rechthoek rond het kaartgebied: { $count }. Het register geeft geen positie, dus deze notities kunnen niet zeggen waar ze liggen; dat moet op de kaart worden gecontroleerd.
permits-register-monuments-left = Nog { $count } worden hier alleen geteld, zonder naam.
permits-section-register-archaeology = Archeologische monumenten van het nationale register ({ $source })
permits-none-register-archaeology = Volgens { $source } ligt er geen ingeschreven archeologisch monument in de rechthoek rond het kaartgebied.
permits-register-archaeology-total = Ingeschreven archeologische monumenten in de rechthoek rond het kaartgebied: { $count }. Het register geeft geen positie, dus deze notities kunnen niet zeggen waar ze liggen; dat moet op de kaart worden gecontroleerd.
permits-register-archaeology-left = Nog { $count } worden hier alleen geteld, zonder naam.
permits-section-register-history = Monumenten van de geschiedenis ({ $source })
permits-none-register-history = Volgens { $source } ligt er geen monument van de geschiedenis in de rechthoek rond het kaartgebied.
permits-register-history-in-area = Monumenten van de geschiedenis in het kaartgebied: { $count }.
permits-register-history-nearby = Verdere monumenten van de geschiedenis in de rechthoek rond het kaartgebied, maar niet in het kaartgebied: { $count }.
permits-register-history-left = Nog { $count } worden hier alleen geteld, zonder namen.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Volgens { $source } ligt er geen staatsbos in het kaartgebied.
permits-forest-units-reach = Een bosbeheerkantoor is verantwoordelijk voor het bos in zijn gebied. Zijn gebied kan ook steden en akkers omvatten.
permits-state-forest-office = verantwoordelijk: { $office }
permits-hint-forest-office = { $units ->
        [none] Bosbeheerkantoor dat verantwoordelijk is voor het bos in het kaartgebied: { $office } ({ $administration }). Vraag daar vroeg als je een evenement in het bos plant.
       *[other] Bosbeheerkantoor dat verantwoordelijk is voor het bos in het kaartgebied: { $office } ({ $administration }), { $units }. Vraag daar vroeg als je een evenement in het bos plant.
    }
permits-hint-forest-owner = { $office ->
        [known] Een evenement in het bos heeft de toestemming van de boseigenaren nodig. Het bosbeheerkantoor kan meestal zeggen wie de eigenaar van het bos is. In meerdere deelstaten moet een georganiseerd evenement in het bos ook worden gemeld bij de bosinstantie of door haar worden toegestaan.
       *[unknown] Een evenement in het bos heeft de toestemming van de boseigenaren nodig. In meerdere deelstaten moet een georganiseerd evenement in het bos ook worden gemeld bij de bosinstantie of door haar worden toegestaan. Deze notities noemen het verantwoordelijke bosbeheerkantoor hier niet; het bosbeheer van de deelstaat of de lagere bosinstantie kan het je zeggen.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Een evenement in het bos heeft meestal de toestemming van de boseigenaar nodig. Het hierboven genoemde boswachterijdistrict (Nadleśnictwo) kan meestal zeggen wie het bos beheert. Bos dat niet bij de Staatsbossen hoort, bijvoorbeeld privébos, staat niet in de gegevens van deze notities. Bos dat niet van de staat is, bijvoorbeeld privé- of gemeentebos, staat voor het bosbeheer onder toezicht van de starost (starosta) (boswet, art. 5 ust. 1 pkt 2); vraag daar bij het starostwo powiatowe.
       *[unknown] Een evenement in het bos heeft meestal de toestemming van de boseigenaar nodig. Deze notities noemen voor het kaartgebied geen bosdistrict van de Staatsbossen; de gegevens omvatten alleen de Staatsbossen, dus het bos kan van iemand anders zijn, bijvoorbeeld van een particuliere eigenaar. Wie de eigenaar is, moet apart worden uitgezocht. Bos dat niet van de staat is, bijvoorbeeld privé- of gemeentebos, staat voor het bosbeheer onder toezicht van de starost (starosta) (boswet, art. 5 ust. 1 pkt 2); vraag daar bij het starostwo powiatowe.
    }
permits-hint-monument-register-pl = In de rechthoek rond het kaartgebied liggen monumenten van het nationale register. Hun posities staan niet in de gegevens, controleer dus op de kaart waar ze zijn. Volgens art. 36 ust. 1 van de wet op de bescherming van monumenten (ustawa o ochronie zabytków) hebben conserverings-, restauratie- en bouwwerkzaamheden aan een ingeschreven monument de vergunning nodig van de monumentenconservator van het woiwodschap (Wojewódzki Konserwator Zabytków). Sinds 3 juni 2026 vragen bouwwerkzaamheden in de omgeving van een monument, een wijziging van het gebruik, daarop aangebrachte installaties of borden en andere handelingen die de substantie of het uiterlijk kunnen veranderen alleen een melding aan de conservator (art. 36aa); die kan binnen 60 dagen bezwaar maken en dan een vergunning eisen. De wet noemt een loop of een controlepost als zodanig niet; vraag het bureau van de conservator als een evenement een monument of de omgeving raakt.
permits-hint-archaeology-register-pl = Archeologische monumenten van het nationale register liggen in de rechthoek rond het kaartgebied. Hun posities staan niet in de gegevens, controleer dus op de kaart waar ze liggen. Volgens art. 36 ust. 1 van de wet op de bescherming van monumenten (ustawa o ochronie zabytków) hebben archeologisch onderzoek (pkt 5), het zoeken naar verborgen of verlaten roerende monumenten met elektronische of technische apparaten zoals metaaldetectors (pkt 12) en conserverings-, restauratie- of bouwwerkzaamheden aan een ingeschreven monument (pkt 1) de vergunning nodig van de woiwodschapsconservator (Wojewódzki Konserwator Zabytków). De vergunning voor zoektochten vervalt op 1 januari 2027: art. 36b staat dan een volwassene toe te zoeken met toestemming van de eigenaar en de bezitter van het perceel na een melding in het zoekregister via de app van het ministerie van cultuur, en art. 36c verbiedt zulke zoektochten onder meer op ingeschreven monumenten en op minder dan 10 m ervan. De wet noemt een wandeling, een loop of een controle als zodanig niet; vraag het bureau van de conservator of een evenement of veldwerk een ingeschreven terrein raakt.
permits-hint-history-register-pl = Een monument van de geschiedenis (Pomnik historii) ligt in of bij het kaartgebied; de sectie over de monumenten van de geschiedenis zegt welk. De president van de Republiek Polen verklaart bij verordening een in het register ingeschreven onroerend monument, of een cultuurpark, van bijzondere waarde tot Pomnik historii (art. 15 ust. 1 van de monumentenwet, ustawa o ochronie zabytków). Het staat dus al onder de bescherming van het register of van het cultuurpark. Voor werk aan een in het register ingeschreven monument moet de monumentenconservator van het woiwodschap (Wojewódzki Konserwator Zabytków) een vergunning verlenen (art. 36 ust. 1); in een cultuurpark stelt het raadsbesluit van de gmina de verboden en beperkingen vast (art. 16 ust. 2, art. 17 ust. 1). De wet noemt een wandeling, een loop of een controlepost niet als zodanig; vraag het bureau van de conservator of een evenement of veldwerk het monument raakt.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] In het kaartgebied ligt een natuurmonument (pomnik przyrody), een ecologisch gebied (użytek ekologiczny), een natuur- en landschapscomplex (zespół przyrodniczo-krajobrazowy) of een documentatiesite (stanowisko dokumentacyjne). De gemeenteraad (rada gminy) stelt die vast bij besluit; het besluit zegt wie toezicht houdt op het object en welke verboden gelden (art. 44 van de Wet op de natuurbescherming). De wet zelf noemt hier geen vergunning voor een evenement. Vraag het besluit en de toezichthouder aan bij het kantoor van de gmina op wier grondgebied het object ligt.
       *[other] In het kaartgebied ligt een natuurmonument (pomnik przyrody), een ecologisch gebied (użytek ekologiczny), een natuur- en landschapscomplex (zespół przyrodniczo-krajobrazowy) of een documentatiesite (stanowisko dokumentacyjne). De gemeenteraad (rada gminy) stelt die vast bij besluit; het besluit zegt wie toezicht houdt op het object en welke verboden gelden (art. 44 van de Wet op de natuurbescherming). De wet zelf noemt hier geen vergunning voor een evenement. Vraag het besluit en de toezichthouder aan { $count ->
        [one] bij het kantoor van de gmina ({ $gmina }) op wier grondgebied het object ligt
       *[other] bij de kantoren van de gmina's ({ $gmina }), elk voor de objecten op zijn grondgebied
    }.
    }
permits-hint-gmina-roads-pl = Een evenement dat het verkeer hindert of een straat op bijzondere wijze gebruikt, heeft een vergunning nodig volgens art. 65 van de Wegenverkeerswet, minstens 30 dagen vooraf aangevraagd. Die geeft de instantie die het verkeer op de weg regelt: de starosta voor wegen van het powiat en de gmina, de burgemeester in een stad met powiatrechten, de maarschalk van het woiwodschap of de GDDKiA voor de overige wegen (art. 10 en 65b). Beheerder van een gminaweg is de wójt, burmistrz of prezydent miasta { $count ->
        [one] van de gmina ({ $gmina });
       *[other] van elke gmina ({ $gmina });
    } de beheerder beslist over het innemen van de wegstrook (art. 19 en 40 van de Wet op de openbare wegen). Deze notities tonen de categorie van de wegen niet.
permits-hint-gmina-land-pl = Als grond in het kaartgebied van de gmina is, bijvoorbeeld een gemeentepark, een plein, een schoolterrein of een bos, beheert de wójt, burmistrz of prezydent miasta die en vertegenwoordigt de gmina (art. 30 en 31 van de Wet op het gemeentelijk zelfbestuur). Vraag { $count ->
        [one] het kantoor van de gmina ({ $gmina }) om
       *[other] de kantoren van de gmina's ({ $gmina }) om
    } toestemming voordat u de grond gebruikt. Deze notities tonen niet wie de eigenaar van de grond is.
permits-hint-protected-areas-cz = In het kaartgebied liggen beschermde gebieden of objecten voor natuurbescherming. Volgens de Wet op de natuur- en landschapsbescherming (Wet nr. 114/1992 Sb.) zijn georganiseerde sport-, toeristische en andere openbare evenementen in een nationaal park verboden buiten de plaatsen die de natuurbeschermingsinstantie daarvoor heeft aangewezen (§ 16 odst. 1 písm. q), en in de beschermingszone van een nationaal park hebben massale evenementen van dit soort buiten die plaatsen en buiten bebouwde gebieden de toestemming van de instantie nodig (§ 37 odst. 3 písm. d). In een nationaal natuurreservaat is het verboden om buiten de paden te lopen of te rijden die met toestemming van de instantie zijn gemarkeerd (§ 29 písm. d). Natuurbeschermingsinstanties zijn onder meer het bestuur van een nationaal park, het Agentschap voor natuurbescherming (AOPK ČR), het regionale kantoor (krajský úřad) en het kantoor van de gemeente met uitgebreide bevoegdheden (§ 75 odst. 1). Andere beschermde gebieden en objecten hebben eigen beschermingsvoorwaarden. Vraag de instantie vroeg als u daar een evenement plant.
permits-hint-monuments-cz = In het kaartgebied liggen cultuurmonumenten of monumentengebieden van de staatslijst. Volgens de Wet op de staatszorg voor erfgoed (Wet nr. 20/1987 Sb., § 14 odst. 1 en 2) moet de eigenaar van een monument vooraf om een besluit of een bindend advies vragen voor onderhoud, herstel, verbouwing of andere wijziging van het monument of zijn omgeving. Hetzelfde geldt voor de eigenaar van een pand in een monumentenreservaat, een monumentenzone of de beschermingszone van een monument voor een bouwwerk, een wijziging van een bouwwerk, grondwerk, een voorziening of een bord, het verwijderen van een constructie, boomverzorging of onderhoudswerk. Het kantoor van de gemeente met uitgebreide bevoegdheden beslist{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, bij een nationaal cultuurmonument het regionale kantoor. De wet noemt geen wandeling, loop of controlepost als zodanig; vraag dat kantoor als een evenement een monument raakt of er voorzieningen of borden bij plaatst.
permits-hint-forest-owner-cz = Een massaal sport-, cultureel of ander maatschappelijk evenement op grond die voor bosfuncties is bestemd, mag alleen worden gehouden met schriftelijke toestemming van de eigenaar van het bos waarin het plaatsvindt (Boswet, Wet nr. 289/1995 Sb., § 20 odst. 5). De organisator moet het evenement bovendien schriftelijk melden bij het kantoor van de gemeente met uitgebreide bevoegdheden{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, of bij het regionale kantoor als het evenement buiten het district van zo'n gemeente reikt, uiterlijk 30 dagen voor het evenement. De melding vermeldt datum en plaats, hoe het evenement wordt bemand, het verwachte aantal deelnemers en de schriftelijke toestemmingen van de eigenaren (zelfde paragraaf). Deze notities tonen niet wie het bos bezit; vraag de boseigenaren vroeg.
permits-hint-roads-cz = Een evenement dat een weg anders gebruikt dan gebruikelijk, bijvoorbeeld een sport-, cultureel of vergelijkbaar evenement dat de veiligheid of doorstroming van het wegverkeer kan in gevaar brengen, heeft een vergunning nodig van het bevoegde wegbeheerkantoor. Die wordt verleend met voorafgaande toestemming van de eigenaar van de weg en, waar de veiligheid of doorstroming van het verkeer kan worden geraakt, van de Politie van de Tsjechische Republiek (Wegenwet, Wet nr. 13/1997 Sb., § 25 odst. 1 en odst. 6 písm. d). Het kantoor is het gemeentekantoor voor lokale wegen, of het kantoor van de gemeente met uitgebreide bevoegdheden waar dat beslist, en het kantoor van de gemeente met uitgebreide bevoegdheden voor wegen van de tweede en derde klasse (§ 40 odst. 4 en 5). Gemeenten in het kaartgebied: { $gmina }. Deze notities tonen de klasse van de wegen niet.
permits-section-water-protection = Waterbeschermingsgebieden ({ $source })
permits-section-forest-functions = Bosfuncties en beschermd bos ({ $source })
permits-none-water-protection = Volgens { $source } ligt er geen waterbeschermingsgebied in het kaartgebied.
permits-none-forest-functions = Volgens { $source } ligt er geen recreatiebos, geen beschermd bos, geen natuurbos en geen afgesloten bosgebied in het kaartgebied.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Recreatiebos (Erholungswald)
        [protected] Beschermd bos (Bannwald of Schutzwald)
        [natural] Natuurbos of strikt reservaat (Naturwald, Totalreservat)
       *[closed] Afgesloten bosgebied
    }
permits-hint-water-protection = { $inner ->
        [yes] De { $zone } van het waterbeschermingsgebied "{ $name }" ligt in het kaartgebied. In zones I en II zijn evenementen meestal uitgesloten. Vraag de lagere waterinstantie (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] De { $zone } van het waterbeschermingsgebied "{ $name }" ligt in het kaartgebied. Evenementen daar zijn meestal mogelijk; de lagere waterinstantie (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) kan zeggen wat de verordening toestaat.
       *[unknown] Het waterbeschermingsgebied "{ $name }" ligt in het kaartgebied. De gegevens zeggen niet welke zone het is. In zones I en II zijn evenementen meestal uitgesloten; vraag de lagere waterinstantie (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Recreatiebos (Erholungswald) ligt in het kaartgebied{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Vraag voor evenementen daar de bosinstantie.
        [protected] Beschermd bos (Bannwald of Schutzwald) ligt in het kaartgebied{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Vraag de bosinstantie voordat je daar een evenement plant.
        [natural] Natuurbos of een strikt reservaat ligt in het kaartgebied{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Het verlaten van de paden is daar vaak verboden; plaats er geen controleposten in zonder de bosinstantie te vragen.
       *[closed] Een bosgebied dat niet betreden mag worden ligt in het kaartgebied{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Houd het buiten de baan.
    }
permits-section-roads = Geclassificeerde wegen volgens OpenStreetMap
permits-none-roads = Volgens OpenStreetMap loopt er geen Autobahn en geen Bundes-, Landes-, Staats- of Kreisstraße door het kaartgebied.
permits-road-line = { $class ->
        [autobahn] Autobahn (snelweg)
        [bundesstrasse] Bundesstraße (federale weg)
        [landesstrasse] Landesstraße of Staatsstraße (deelstaatweg)
       *[kreisstrasse] Kreisstraße (districtsweg)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] De Autobahn { $refs } loopt door het kaartgebied. Een baan mag haar alleen oversteken via bruggen of onderdoorgangen.
        [bundesstrasse] De Bundesstraße { $refs } loopt door het kaartgebied. Een afsluiting of een oversteek met verkeersregelaars wordt bevolen door de wegverkeersinstantie (Straßenverkehrsbehörde); de weg wordt meestal door de Bund onderhouden via het wegenbeheer van de deelstaat, in grotere steden vaak door de stad.
        [landesstrasse] De Landes- of Staatsstraße { $refs } loopt door het kaartgebied. Een afsluiting of een oversteek met verkeersregelaars wordt bevolen door de wegverkeersinstantie (Straßenverkehrsbehörde); de weg wordt meestal door de deelstaat onderhouden, in grotere steden vaak door de stad.
       *[kreisstrasse] De Kreisstraße { $refs } loopt door het kaartgebied. Een afsluiting of een oversteek met verkeersregelaars wordt bevolen door de wegverkeersinstantie (Straßenverkehrsbehörde); de weg wordt meestal door het Kreis onderhouden, in grotere steden vaak door de stad.
    }
permits-hint-state-forest = { $office ->
        [none] Volgens de open gegevens van de deelstaat is { $share } van het kaartgebied staatsbos van de eigenaar "{ $owner }".
       *[other] Volgens de open gegevens van de deelstaat is { $share } van het kaartgebied staatsbos van de eigenaar "{ $owner }". Verantwoordelijk: { $office }.
    }
