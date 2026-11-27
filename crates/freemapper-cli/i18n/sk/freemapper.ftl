# Program texts of freemapper, Slovak (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } nie je jazyk, v ktorom má freemapper texty; dostupné: { $available }

## init

cli-part-needs-osm = --part vyberie časť oblasti OSM a vyžaduje --osm; pri --place najprv vyhľadaj id a použi --osm <id> --part <n>
cli-reading-file = načítava sa { $path }
cli-no-area-for-place = pre "{ $query }" sa nenašla žiadna oblasť. Ak miesto tvorí oblasť z viacerých oddelených častí, vyber jednu cez --osm <id> --part <n> alebo použi --geojson
cli-using-place = Používa sa { $name } ({ $id })
cli-created-project = Projekt { $path } bol vytvorený

## Cache

cli-no-cache-dir = tento systém nemá priečinok vyrovnávacej pamäte; zadaj --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Stiahnutá 1 dlaždica
        [few] Stiahnuté { $count } dlaždice
       *[other] Stiahnutých { $count } dlaždíc
    }
cli-wrote = Zapísaný súbor { $path }
cli-kept-map = Súbor { $path } zostal zachovaný (môže obsahovať tvoju prácu)
cli-map-lacks-templates = Mapa zatiaľ neodkazuje na rastrové podklady. Pridaj ich v OOM (Templates > Open template…) alebo mapu nahraď cez --force, ktoré starú ponechá ako .omap.bak.
cli-map-lacks-buildings = Mapa zatiaľ nemá budovy z ALKIS. Ak ich chceš pridať, nahraď ju cez --force, ktoré starú ponechá ako .omap.bak.
cli-map-lacks-canopies = Mape chýbajú prístrešky a prístrešky pre autá z ALKIS (522), ktoré sú teraz objektmi mapy. Ak ich chceš pridať, spusti s --force, ktoré starú mapu ponechá ako .omap.bak.
cli-map-light = Základná mapa: { $joined } čiar spojených, { $left } skrytých alebo príliš malých objektov ostalo vo vrstvách kandidátov (nič sa neposunulo)
cli-map-light-kept = Vrstvy kandidátov teraz obsahujú aj { $left } objektov alebo úsekov, ktoré táto verzia odoberá zo základnej mapy (označené freemapper:generalized). Zachovaná mapa ich ešte má, takže sa zobrazia dvakrát. Nahraď mapu cez --force, aby sa vyčistila (stará zostane ako .omap.bak).
cli-map-old-candidates = Mapa stále uvádza templates/candidates.omap; tento beh ho naplnil všetkými kandidátmi. Kandidáti teraz prichádzajú aj v siedmich vrstvách, každá v jednom súbore (templates/candidates-*.omap). Mapa ich uvedie po nahradení cez --force (stará zostane ako .omap.bak) alebo ich pridaj sám v OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF bez uvedenia zdrojov: ODbL a dl-de/by-2-0 vyžadujú uviesť zdroje pri odovzdaní mapy; pozri ATTRIBUTION.txt
cli-wrote-pdf = Zapísaný súbor { $path } (mapa bez kandidátov)
cli-wrote-pdf-candidates = Zapísaný súbor { $path } (kontrolná tlač: mapa so všetkými kandidátmi, polohovo presne; skontroluj ich v teréne)
cli-wrote-training = Zapísané { $map } a { $pdf } (tréningová mapa: { $joined } spojených, { $shifted } posunutých, { $left } vynechaných alebo orezaných)
cli-training-free-ends = Konce čiar ponechané voľné: { $ways } tam, kde cesta narazila na inú cestu, { $others } pri iných čiarach
cli-training-apart-left = tréningová mapa má stále { $length } mm (na papieri), kde majú objekty príliš malú medzeru; skontroluj to v OOM
cli-kept-training = Tréningová mapa { $path } zostala zachovaná, aby ju beh neprepísal, ak bola vytlačená alebo odovzdaná; na obnovenie ju zmaž alebo zadaj --force

## OCAD

cli-ocad-kept = Súbor { $path } zostal zachovaný (môže obsahovať tvoju prácu; --force ho nahradí a starý ponechá ako .ocd.bak)
cli-ocad-old-candidates = Mapa OCAD stále uvádza { $path } ako podkladovú mapu; tento beh ju naplnil všetkými kandidátmi. Kandidáti teraz prichádzajú aj v siedmich vrstvách, každá v jednom súbore (-candidates-*.ocd). Mapa OCAD ich uvedie po nahradení cez --force (stará zostane ako .ocd.bak) alebo ich pridaj sám v OCAD ako podkladové mapy.
cli-ocad-wrote = { $count ->
        [one] Zapísaný súbor { $path } (1 objekt)
        [few] Zapísaný súbor { $path } ({ $count } objekty)
       *[other] Zapísaný súbor { $path } ({ $count } objektov)
    }
cli-ocad-left-out = Z mapy OCAD vynechané: { $skipped } objektov (pre OCAD bez symbolu, bez plochy alebo mimo kresliacej oblasti OCAD ±2 m) a { $dropped } degenerovaných častí čiar alebo otvorov.
cli-ocad-load-symbols = V OCAD zvoľ Symbol > Load Colors and Symbols From… a vyber oficiálnu sadu symbolov{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , školskú sadu IOF 2019
       *[other] {" "}ISSprOM 2019
    }; nahradí zástupné symboly freemapperu podľa čísla.

## select

cli-select-running = Výber oblasti pre { $path }: na stránke stlač "Hotovo" alebo zastav cez Ctrl+C.
cli-select-no-browser = Prehliadač sa nepodarilo otvoriť ({ $error }); otvor adresu uvedenú vyššie.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; bez plochy, nemôže byť oblasťou mapy): { $place }
cli-find-hint = Projekt vytvoríš príkazom: freemapper init <dir> --osm { $id }
cli-find-no-area = Žiadne z týchto miest nemá plochu, ktorá by mohla byť oblasťou mapy. Oblasť nakresli príkazom: freemapper select <dir>
cli-find-nothing = pre "{ $query }" sa nenašlo žiadne miesto; skús iný názov alebo nakresli oblasť príkazom: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = warning: adaptér v { $folder } je vynechaný: { $finding }
cli-adapter-note = warning: adaptér v { $folder } sa načíta, ale: { $finding }
cli-adapter-from-project = warning: používa sa adaptér { $adapter } z priečinka projektu; môže sa pýtať týchto hostiteľov: { $hosts }
cli-adapter-overlaps = warning: adaptér { $adapter } pokrýva aj územie { $others }; jeho objekty sa tam dostanú na mapy
cli-adapter-dir-relative = warning: FREEMAPPER_ADAPTER_DIR nie je absolútna cesta ({ $path }); ignoruje sa
cli-adapters-no-project = warning: { $path } nie je priečinok; neboli načítané žiadne adaptéry projektu
cli-adapters-export-unknown = adaptér { $adapter } nie je opísaný manifestom tohto programu, takže niet čo exportovať; `freemapper adapters list` ukáže identifikátory
cli-adapters-export-taken = { $path } existuje a nie je prázdny; exportujte do priečinka, ktorý ešte neexistuje
cli-adapters-export-done = Do { $path } sa zapísalo { $count } súborov adaptéra { $adapter }. Je v nich len to, čo opisuje jeho manifest; adresy zapísané v kóde programu v nich nie sú.
cli-adapters-export-next = Zmeňte adresy v manifest.toml, skontrolujte priečinok príkazom `freemapper adapters check` a presuňte ho do { $path }: od ďalšieho spustenia nahradí vstavaný adaptér, s varovaním. Zmazaním priečinka sa vstavaný adaptér vráti.
cli-adapters-overlaps = zdieľa územie s: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revízia { $revision }  skontrolované { $checked }  hostitelia: { $hosts }
cli-adapters-source-built-in = vstavaný
cli-adapters-source-user = z tvojho priečinka
cli-adapters-source-project = z projektu
cli-adapters-never-checked = nikdy
cli-adapters-user-dir = Tvoje adaptéry sa načítavajú z { $path }
cli-adapters-no-user-dir = Tento systém nemá priečinok pre tvoje adaptéry; jeden určuje FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper sa teraz môže pre { $name } pýtať aj hostiteľa { $host }. Záznam je v { $file }; `freemapper adapters disallow-host { $adapter } { $host }` ho zruší. Spustená stránka výberu oblasti alebo AI asistent načíta záznam po opätovnom spustení
cli-allow-host-already = { $host } je pre { $name } už povolený
cli-allow-host-declared = adaptér pre { $name } uvádza { $host } sám; nie je čo povoľovať
cli-allow-host-refused = { $host } sa nedá povoliť: { $reason }
cli-allow-host-unknown-adapter = vstavaný adaptér { $adapter } neexistuje; vstavané adaptéry sú: { $adapters }
cli-allow-host-folder-adapter = { $adapter } je adaptér z tvojho priečinka adaptérov; pridaj hostiteľa radšej do `[net] hosts` jeho manifestu
cli-allow-host-no-config = tento systém nemá konfiguračný priečinok na uloženie povolených hostiteľov
cli-allow-host-write = súbor { $file } sa nepodarilo zapísať: { $detail }
cli-allow-host-unreadable = súbor { $file } sa nedá prečítať ({ $detail }); najprv ho oprav alebo odstráň; nič sa nezmenilo
cli-disallow-host-removed = freemapper sa už pre { $name } nepýta hostiteľa { $host }
cli-disallow-host-absent = { $host } nebol pre { $name } povolený; nie je čo odvolať
cli-allowed-hosts-unreadable = warning: súbor { $file } sa nepodarilo prečítať ({ $detail }); žiadni ďalší hostitelia nie sú povolení
cli-allowed-hosts-ignored = warning: { $file }: záznam { $host } pre { $adapter } sa ignoruje: { $reason }
cli-allowed-hosts-no-adapter = freemapper nemá vstavaný adaptér s týmto názvom
cli-allowed-host-redundant = warning: { $host } je teraz jedným z hostiteľov, ktorých adaptér pre { $name } uvádza; tvoj záznam v { $file } bol odstránený
cli-adapters-allowed = povolil si: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adaptér { $adapter } ({ $name }), revízia { $revision }, hostitelia: { $hosts }
cli-check-error = error: { $finding }
cli-check-warning = warning: { $finding }
cli-check-passed = { $folder } kontrolou prešiel
cli-check-passed-warnings = { $folder } kontrolou prešiel, { $count ->
        [one] s jedným upozornením
        [few] so { $count } upozorneniami
       *[other] s { $count } upozorneniami
    }
cli-check-failed = { $folder } kontrolou neprešiel
cli-check-no-folder = priečinok { $folder } neexistuje
cli-check-live-ok = { $capability } v poriadku: { $detail }
cli-check-live-failed = { $capability } ZLYHALO: { $detail }
cli-check-live-not-asked = { $capability } sa nepýtalo: { $detail }
cli-check-live-box-outside = obdĺžnik neleží v hraniciach adaptéra { $adapter }; živá kontrola sa pýta služieb na miesto, ktoré adaptér pokrýva
cli-check-live-cannot = živá kontrola sa nedá spustiť: { $detail }
cli-check-detail-not-declared = adaptér žiadnu neuvádza
cli-check-detail-stopped = zlyhala skoršia časť
cli-check-detail-see-warnings = pozri upozornenia
cli-check-detail-outside-dataset = obdĺžnik leží mimo súboru dát
cli-check-detail-objects = { $count ->
        [one] jeden objekt
        [few] { $count } objekty
       *[other] { $count } objektov
    }
cli-check-detail-layers = { $layers } vrstiev, { $count } objektov
cli-check-detail-trees = { $count ->
        [one] jeden strom
        [few] { $count } stromy
       *[other] { $count } stromov
    }
cli-check-detail-extract = { $label }, dáta z { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = model terénu
cli-check-raster-surface = model povrchu
cli-check-raster-tiles = { $product }: { $count ->
        [one] jedna dlaždica
        [few] { $count } dlaždice
       *[other] { $count } dlaždíc
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } a { $last }

## Folder lock

cli-lock-held = tento priečinok používa iný beh freemapperu (od { $time }, spustil { $who }). Počkaj naň alebo ho najprv zastav.
cli-lock-who-cli = terminál
cli-lock-who-mcp = AI agent (MCP)
cli-lock-who-page = stránka výberu
cli-lock-who-unknown = iný program
cli-lock-failed = priečinok projektu sa pre tento beh nepodarilo zamknúť: { $error }
cli-lock-stuck = v ceste je niečo iné ako súbor zámku a nepodarilo sa to odstrániť

## Run routes of the selection page

cli-run-bad-request = Voľby behu nie sú platné: { $error }
cli-run-no-project = V tomto priečinku zatiaľ nie je uložený projekt. Najprv ulož oblasť.
cli-run-already-running = Pre tento projekt už prebieha beh ({ $job }). Počkaj, kým skončí, alebo ho najprv zruš.
cli-run-disk-stops = Na stiahnutie nie je dosť voľného miesta na disku: { $messages }
cli-run-busy-elsewhere = Iný projekt tohto programu práve spúšťa pipeline. Projekty zdieľajú vyrovnávaciu pamäť sťahovania, preto beží vždy len jeden beh. Skús znova, keď skončí.
cli-run-nothing-to-cancel = Žiadny beh neprebieha, nie je čo zrušiť.
cli-run-unknown-job = Pre tento projekt neexistuje beh { $job }.
cli-status-too-fast = Požiadavky na stav prichádzajú príliš často. Pýtaj sa najviac päťkrát za sekundu.
cli-open-folder-failed = Priečinok sa nepodarilo otvoriť ({ $error }).
cli-open-folder-no-opener = Tento počítač nemá program na otváranie priečinkov ({ $program }). Otvor priečinok ručne.
cli-open-folder-missing = Priečinok projektu zatiaľ neexistuje. Najprv ulož oblasť.
cli-open-folder-refused = Názov priečinka nie je bezpečné odovzdať správcovi súborov.
cli-save-running = Pre tento projekt prebieha beh. Ulož znova, keď skončí.
cli-save-unknown-kind = Neznámy druh mapy "{ $kind }". Použi schoolyard, sprint alebo forest.
cli-save-scale-too-large = Mierka { $scale } je príliš veľká.
cli-save-bad-content = Nastavenia obsahu nie sú platné. Nič sa neuložilo.
cli-map-image-none = Najprv vytvor mapu.
cli-map-image-gone = Tento náhľad už nie je dostupný.
cli-map-image-running = Pre tento projekt prebieha beh. Náhľad sa dá vypočítať, keď skončí.
cli-map-image-stale = Uložený projekt už nezodpovedá vygenerovanej mape, napríklad preto, že sa zmenila oblasť, druh mapy alebo mierka. Najprv spusti beh, ktorý mapu nahradí ("Nahradiť existujúcu mapu" na stránke alebo freemapper run --force).
cli-map-image-superseded = Medzitým sa žiadal novší náhľad, preto sa tento nevypočítal.
cli-map-image-bad-content = Nastavenia obsahu nie sú platné.
cli-map-image-bad-request = Požiadavka na náhľad nie je platná. Pošli objekt JSON, prípadne s nastaveniami obsahu.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – výber oblasti
page-heading = Vyber oblasť mapy
page-update-available = Je dostupný freemapper { $version }. Aktualizuj príkazom
page-update-notes = Poznámky k vydaniu
page-folder = Priečinok projektu
page-pick = Vyhľadaj miesto
page-query-placeholder = napr. Clara-Zetkin-Schule Strausberg
page-search-button = Hľadať
page-searching = Hľadá sa …
page-no-results = Nič sa nenašlo.
page-point-only = iba bod, nie plocha
page-draw = Nakresli sám
page-rectangle = Obdĺžnik
page-polygon = Mnohouholník
page-finish-polygon = Uzavrieť mnohouholník
page-clear = Vymazať
page-hint-rectangle = Klikni na mape na dva protiľahlé rohy.
page-hint-polygon = Klikaj na rohy jeden po druhom. Uzavrieš ho kliknutím na prvý roh, dvojitým kliknutím alebo tlačidlom "Uzavrieť mnohouholník". Esc ruší, Backspace odstráni posledný roh.
page-hint-idle = Vyber miesto z vyhľadávania alebo nakresli oblasť sám.
page-summary = Oblasť
page-source-osm = Z OpenStreetMap: { $name }
page-source-drawn = Nakreslené ručne
page-size = Veľkosť s okrajom: { $width } × { $height } m
page-tiles-one = { $n } dlaždica: { $names }
page-tiles-other = { $n } dlaždíc: { $names }
page-download = Sťahovanie: približne { $mb } MB
page-download-none = Sťahovanie: všetko je už vo vyrovnávacej pamäti
page-download-details = Čo sa sťahuje
page-cached = vo vyrovnávacej pamäti
page-disk = Miesto na disku: { $volumes }
page-disk-volume = { $free } voľných, približne { $needed } potrebných
page-cache-volume = vyrovnávacia pamäť
page-project-volume = priečinok projektu
page-no-room = nezmestí sa na disk
page-no-room-skip = Nezmestí sa všetko na disk (freemapper ponecháva { $reserve } voľných). Sťahovanie vynechá označené časti. Ak ich potrebuješ, najprv uvoľni miesto:
page-no-room-stop = Nie je dosť miesta na disku (freemapper ponecháva { $reserve } voľných). Bez označených častí mapa nevznikne, preto sťahovanie skončí skôr, než niečo stiahne. Najprv uvoľni miesto:
page-disk-unknown = freemapper nevie zistiť voľné miesto na disku ({ $error }). Sťahuje bez kontroly.
page-too-large = Oblasť je príliš veľká: najviac { $max } m na stranu (s okrajom). Vyber menšiu.
page-outside = Oblasť leží mimo podporovaných spolkových krajín ({ $states }); freemapper zatiaľ nemá zdroje dát pre iné regióny.
page-outside-unknown = Oblasť leží mimo regiónov, ktoré freemapper podporuje; zatiaľ tam nemá zdroje dát.
page-adapters-failed = Letecké snímky sa nedajú vypísať ({ $detail }). Mapa zobrazuje iba OpenStreetMap; všetko ostatné funguje.
page-settings = Mapa
page-kind = Druh mapy
page-schoolyard = Mapa školského dvora (1:1000)
page-sprint = Šprintová mapa (1:4000)
page-forest = Lesná mapa (1:15000)
page-scale = Mierka 1:
page-map-title = Názov
page-contour-interval = Interval vrstevníc
page-contour-auto = automaticky
page-steep-slope = Prah strmosti
page-steep-slope-auto = automaticky (27°)
page-steep-slope-hint = Od tohto uhla svahu sa terén pri hľadaní zemných stupňov a skalných stien považuje za strmý. 27° (automaticky) sa hodí pre pahorkatiny, parky a mestá. V skalných mestách, roklinách a lomoch je strmý takmer každý svah, takže stupne a steny splynú do jednej škvrny a nenájde sa žiadna; 55° až 65° ich oddelí. Príliš vysoká hodnota nechá iba najstrmšie steny.
page-face-slope = Prah stien
page-face-slope-auto = automaticky (70°)
page-face-slope-hint = Od tohto uhla svahu sa terén kreslí ako čierna plocha skalnej steny. 70° (automaticky) sa hodí pre skalné mestá ako Adršpach (5,2 % plochy). 65° dáva viac a väčších čiernych plôch, vrátane strmých svahov (8,4 %). 75° nechá len skutočné steny s rozstrapkanými okrajmi (2,5 %). Príliš vysoká hodnota nenechá takmer žiadne plochy.
page-settings-fixed = Projekt už existuje; druh mapy, mierka a názov sú vo freemapper.toml. Interval vrstevníc sa dá ešte zmeniť.
page-save = Uložiť
page-saving = Ukladá sa …
page-saved = Uložené do freemapper.toml.
page-zone-changed = Uložené. Oblasť teraz leží v inom pásme UTM; freemapper pri ďalšom behu stiahne jej dáta znova.
page-provider-changed = Uložené. Výšky a letecké snímky novej oblasti poskytuje iný poskytovateľ dát; freemapper pri ďalšom behu stiahne jej dáta znova.
page-save-stale = freemapper.toml sa od načítania tejto stránky zmenil v inom okne alebo iným programom. Nič sa neuložilo; stránka teraz pozná uložený stav. Ulož znova, aby ho nahradila oblasť zobrazená tu.
page-has-map = Existujúca mapa (.omap) zostane ako je; freemapper vytvorí nové podklady a kandidátov pre novú oblasť. Voľba "Nahradiť existujúcu mapu" nižšie (alebo "freemapper run --force") vytvorí znova aj mapu a starú ponechá ako .omap.bak.
page-done = Hotovo
page-source = Zdrojový kód (AGPL-3.0-or-later)
page-closed = Hotovo. Toto okno môžeš zavrieť.
page-next = Ďalej: freemapper run "{ $dir }"
page-made-files = Vytvorené: { $files }. Priečinok: { $dir }
page-back-to-agent = Povedz svojmu AI asistentovi, že oblasť je uložená.
page-nothing-saved = Nič sa neuložilo.
page-failed = Chyba: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Letecká snímka { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Čo sa dostane na mapu
page-content-help = Profil rozhoduje, čo ide rovno na mapu a čo zostane kandidátom na kontrolu. Voľba pre skupinu má prednosť pred profilom. Ukladá sa spolu s oblasťou.
page-content-profile-legend = Profil
page-content-profile-safe = Bezpečný
page-content-profile-safe-hint = Každý objekt zostane tam, kam ho zaradí zostavenie: budovy a plochy z katastra idú na mapu; múry, ploty, stromy a veľa z OpenStreetMap (rybníky, cesty bez známeho povrchu) zostanú kandidátmi.
page-content-profile-balanced = Vyvážený
page-content-profile-balanced-hint = Objekty z katastra, registrov a mapovania ulíc idú na mapu, spolu s vrstevnicami a plochami, do ktorých sa nesmie vstupovať, pozdĺž železničných tratí. Múry, ploty, živé ploty, svahy, schody, voda, stromy, pamätihodnosti, priechody, cesty a železnice pochádzajú tiež z OpenStreetMap. Ulicový mobiliár, ostatné plochy, do ktorých sa nesmie vstupovať, a tvary z modelu terénu zostanú kandidátmi.
page-content-profile-full = Úplný
page-content-profile-full-hint = Všetko ide na mapu ako v kontrolnej tlači; mimo zostanú len skryté kandidáty.
page-content-groups-legend = Podľa skupín
page-content-groups-help = "Ako v profile" sa riadi profilom. Čo to pre skupinu znamená, závisí od toho, odkiaľ každý objekt pochádza.
page-content-place-profile = Ako v profile
page-content-place-map = Mapa
page-content-place-candidates = Kandidát
page-content-place-off = Vypnuté
page-content-group-walls = Múry
page-content-group-fences = Ploty
page-content-group-hedges = Živé ploty
page-content-group-slopes = Svahy a skaly
page-content-group-stairs = Schody
page-content-group-trees = Stromy a skupiny stromov
page-content-group-water-features = Voda, fontány a pramene
page-content-group-monuments = Veže, pamätníky a elektrické vedenia
page-content-group-furniture = Ulicový mobiliár a lampy
page-content-group-passages = Priechody
page-content-group-contours = Vrstevnice
page-content-group-landforms = Kopčeky, priehlbiny, svahy a skaly z modelu terénu
page-content-group-vegetation = Vegetácia z laserových mračien bodov (Brandenbursko, Berlín, Severné Porýnie-Vestfálsko, Porýnie-Falcko, Sársko a Bavorsko; vypnuté, kým ju nezvolíš; sťahuje približne 40 až 600 MB na km²)
page-content-group-out-of-bounds = Plochy, do ktorých sa nesmie vstupovať
page-content-group-paths = Cesty a železnice
page-content-group-streets = Ulice a námestia
page-run-heading = Vytvoriť mapu
page-preview-heading = Náhľad mapy
page-preview-stored = Ako naposledy vygenerované.
page-preview-live = S nastaveniami na tejto stránke, zatiaľ neuloženými.
page-preview-live-saved = S uloženými nastaveniami, zatiaľ nevygenerované v behu.
page-preview-computing = Počíta sa …
page-preview-none = Zatiaľ nebola vygenerovaná žiadna mapa.
page-preview-later = Táto mapa zatiaľ nemá náhľad. Príde s ďalším behom, ktorý mapu zapíše.
page-preview-stale = Oblasť, druh mapy alebo mierka sa od vygenerovania mapy zmenili. Obrázok stále ukazuje mapu tak, ako bola vygenerovaná naposledy; nový náhľad vyžaduje beh s "Nahradiť existujúcu mapu".
page-preview-area-unsaved = Oblasť na tejto stránke zatiaľ nie je uložená. Obrázok ukazuje mapu tak, ako bola vygenerovaná naposledy; nová oblasť vyžaduje beh s "Nahradiť existujúcu mapu".
page-preview-run = Ako naposledy vygenerované. Stránka obrázok aktualizuje, keď beh skončí.
page-preview-open = Otvoriť v plnej veľkosti
page-preview-alt = Náhľad objektov mapy
page-opt-ocad = Zapísať štartovací projekt OCAD
page-help-ocad = Pre všetkých, ktorí pokračujú v OCAD.
page-opt-pdf = Tlačiteľné PDF
page-help-pdf = Iba objekty mapy, bez kandidátov.
page-opt-pdf-candidates = Kontrolná tlač so všetkými kandidátmi
page-opt-training = Tréningová mapa
page-help-training = Mapa a všetci kandidáti, upravené tak, aby sa dali tlačiť ako sú; s OCAD aj ako súbor OCAD.
page-views-heading = Pohľady na tréningovú mapu
page-view-no-paths = Bez ciest (mapa Ultimate)
page-view-brown-only = Iba hnedá (orientačný beh podľa vrstevníc)
page-view-no-north-lines = Bez severných čiar
page-view-custom = Iný pohľad
page-help-views = Každý zaškrtnutý pohľad je jeden ďalší súbor vedľa tréningovej mapy, `<name>-training-<view>.omap` a jeho PDF, s OCAD aj súbor OCAD. Iný pohľad je `no:505,506` (vynechať symboly) alebo `only:101,102` (ponechať iba tieto); `+` spája pohľady. Názov, mierka a zdroje zostávajú vždy.
page-help-views-forest = Bez ciest a iba hnedá sú len pre lesné mapy.
page-opt-scale-bar = Grafická mierka na PDF
page-help-scale-bar = Nemecké mapy ju zvyčajne nemajú.
page-opt-sources = Riadky so zdrojmi na PDF
page-sources-auto = automaticky
page-sources-full = úplne
page-sources-credit-only = iba freemapper
page-warn-credit-only = PDF potom uvádzajú iba freemapper. Licencie dát (ODbL, dl-de/by-2-0) vyžadujú uvedenie zdrojov pri odovzdaní mapy; zdroje potom uvedieš sám. ATTRIBUTION.txt ich vypisuje všetky.
page-need-pdf = Grafická mierka a riadky so zdrojmi platia len pri vytváraní PDF.
page-opt-offline = Iba vyrovnávacia pamäť, bez siete
page-opt-force = Nahradiť existujúce mapy
page-help-force = Mapa a, ak je zaškrtnutá "Tréningová mapa", aj tréningová mapa. Staré súbory zostanú ako .omap.bak.
page-opt-permits = Poznámky k povoleniam
page-help-permits = Správne celky, chránené územia a koho sa pýtať, v permits.md. Orientačná pomôcka, nie právna rada.
page-run-start = Vytvoriť mapu
page-run-again = Vytvoriť znova
page-run-cancel = Zrušiť
page-run-cancelling = Ruší sa …
page-warnings-heading = Upozornenia
page-where-files = Súbory sú v priečinku projektu:
page-open-folder = Otvoriť priečinok
page-log-label = Záznam behu
page-force-title = Nahradiť existujúce mapy?
page-force-text = Existujúca mapa sa nahradí a ak je zaškrtnutá "Tréningová mapa", aj tréningová mapa. Tvoje úpravy sa nestratia: staré súbory zostanú v priečinku projektu ako .omap.bak. Pokračovať?
page-force-cancel = Zrušiť
page-force-confirm = Nahradiť
page-step-fetch = Sťahovanie dát
page-step-derive = Výpočet podkladov
page-step-map = Zápis mapy
page-step-ocad = Zápis projektu OCAD
page-step-pdf = Zápis PDF
page-step-pdf-candidates = Zápis kontrolnej tlače
page-step-training = Zápis tréningovej mapy
page-step-permits = Zápis poznámok k povoleniam
page-step-line = Krok { $index } z { $total }: { $label }
page-starting = Beh sa spúšťa …
page-file-kind-map = mapa
page-file-kind-template = podklad
page-file-kind-ocad = mapa OCAD
page-file-kind-ocad-candidates = kandidáti OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrolná tlač
page-file-kind-training-map = tréningová mapa
page-file-kind-training-pdf = PDF tréningovej mapy
page-file-kind-training-ocad = tréningová mapa OCAD
page-file-kind-training-view-map = pohľad na tréningovú mapu
page-file-kind-training-view-pdf = PDF pohľadu na tréningovú mapu
page-file-kind-training-view-ocad = pohľad na tréningovú mapu OCAD
page-file-kind-preview = náhľad mapy
page-file-kind-permits = poznámky k povoleniam
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Hotovo. Súbory sú zapísané.
page-result-failed = Beh zlyhal: { $error }
page-result-failed-kept = Zapísané zostáva v priečinku projektu; nový beh pokračuje odtiaľ.
page-result-cancelled = Zrušené. Už stiahnuté zostáva vo vyrovnávacej pamäti; nový beh pokračuje odtiaľ.
page-result-title-done = Mapa vytvorená
page-result-title-failed = Beh zlyhal
page-result-title-cancelled = Beh zrušený
page-kept-map = Existujúca mapa zostala nezmenená. "Nahradiť existujúcu mapu" ju vytvorí znova a starú ponechá ako .omap.bak.
page-no-files = Nevznikli žiadne súbory.
page-opened-folder = Priečinok je otvorený.
page-open-failed = { $message } Priečinok: { $dir }
page-why-no-area = Najprv vyber oblasť.
page-why-checking = Náhľad sa počíta …
page-why-too-large = Oblasť je príliš veľká.
page-why-outside = Oblasť leží mimo podporovaných spolkových krajín.
page-why-disk = Nie je dosť voľného miesta na disku (pozri vyššie).
page-why-running = Beh prebieha.
page-err-scale-bar-no-pdf = Grafická mierka vyžaduje PDF.
page-err-sources-no-pdf = Riadky so zdrojmi vyžadujú PDF.
page-save-failed = Oblasť nie je uložená; žiadny beh sa nespustil.
page-poll-lost = Server neodpovedá. freemapper to skúša ďalej.
page-run-unknown = Server tento beh už nepozná.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Zavrieť

## update (spec 3.4 to 3.6)

update-source-build = Tento freemapper bol zostavený zo zdrojového kódu. Aktualizuj ho cez git pull a cargo build --release --locked.
update-current = freemapper { $version } je aktuálny.
update-available = Je dostupný freemapper { $new } (máš { $old }), vydaný { $released }.
update-details =
    Poznámky k vydaniu: { $notes }
    Stiahnutie: { $size } MB
update-decimal-separator = ,
update-ask = Nainštalovať teraz? [y/N]
update-not-installed = Nič sa nenainštalovalo.
update-needs-yes = vstup nie je terminál, takže sa freemapper nemôže pred inštaláciou opýtať; spusti freemapper update --yes na inštaláciu bez pýtania
update-installing = sťahuje a kontroluje sa freemapper { $version }
update-done = freemapper bol aktualizovaný z { $old } na { $new }. Poznámky k vydaniu: { $notes }
update-mcp-restart = Spustený server MCP zostane na starej verzii, kým ho AI asistent nespustí znova.
update-rolled-back = Návrat na freemapper { $version }. Verzia, ktorú si opustil, zostala uložená ako { $path }.
update-not-writable = do priečinka programu { $path } sa nedá zapisovať a freemapper nežiada práva správcu. Rozbaľ tam nové vydanie ručne alebo presuň freemapper do priečinka, do ktorého môžeš zapisovať.
update-too-old = freemapper { $new } sa nedá nainštalovať samoaktualizáciou verzie { $old }; nainštaluj { $needed } alebo novšiu ručne, pozri { $notes }
update-busy = v priečinku programu už beží iná aktualizácia alebo návrat (súbor zámku { $path }); počkaj, kým skončí, a skús znova
update-bad-signature = vydanie sa nepodarilo overiť ({ $error }); nič sa nenainštalovalo
update-check-bad-signature = najnovšie vydanie freemapperu sa nepodarilo overiť ({ $error }); nič sa nenainštalovalo
update-no-manifest = najnovšie vydanie sa nevie samo nainštalovať (nemá manifest aktualizácie); stiahni ho z { $page }
update-no-manifest-version = vydanie { $version } neexistuje alebo sa nevie samo nainštalovať (nemá manifest aktualizácie); pozri { $page }
update-no-asset = vydanie nemá program pre tento systém ({ $platform }); nič sa nenainštalovalo
update-no-previous = neexistuje predchádzajúca verzia, na ktorú sa dá vrátiť ({ $path } chýba)
update-failed = aktualizácia zlyhala: { $error }
update-mode-set = Režim aktualizácie nastavený na { $mode } v { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } toto nastavenie prebíja, kým je nastavené.
update-mode-failed = súbor { $path } sa nepodarilo zapísať: { $error }
update-notice = Je dostupný freemapper { $new } (máš { $old }). Aktualizuj príkazom: freemapper update
update-applies-next-start = Nová verzia sa použije od ďalšieho spustenia freemapperu.
update-install-failed = freemapper { $new } sa nenainštaloval: { $error }
update-no-config-dir = tento systém nemá konfiguračný priečinok; nastav FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Poznámka pre osobu: pri štarte server freemapper zistil toto o priečinkoch adaptérov. Odovzdaj to ďalej; osoba priečinok opraví alebo odstráni a potom server spustí znova.
    { $lines }
update-mcp-note = Poznámka pre osobu: je dostupný freemapper { $new } (táto je { $old }). Pred volaním install_update sa jej opýtaj.
update-mcp-needs-confirm = nič sa nenainštalovalo; najprv sa opýtaj osoby a install_update zavolaj s confirm: true, až keď súhlasí
update-mcp-note-manual = Poznámka pre osobu: je dostupný freemapper { $new } (táto je { $old }), ale install_update ho nevie nainštalovať; nainštaluj { $needed } alebo novšiu ručne, pozri { $notes }
update-mcp-note-bad-signature = Poznámka pre osobu: najnovšie vydanie freemapperu sa nepodarilo overiť ({ $error }); nič sa nenainštalovalo. Povedz jej to a neinštaluj ho iným spôsobom.
update-mcp-restart-now = Spusti server MCP freemapperu teraz znova; stále beží na starej verzii.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = chyba
error-cancelled = zrušené
cli-cancelled-kept = zrušené; zachované: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disk je plný, preto freemapper nemohol zapísať { $path }. Uvoľni miesto a skús znova

## Errors: project

error-project-already-exists = projekt už existuje: { $path }
error-project-map-exists = mapa už existuje (môže obsahovať tvoju prácu); nahradíš ju cez --force: { $path }
error-project-invalid-scale = neplatná mierka 1:{ $scale }
error-project-invalid-contour-interval = neplatný interval vrstevníc { $interval } m vo freemapper.toml; musí byť väčší ako 0
error-project-invalid-steep-slope = neplatný prah strmosti { $slope }° vo freemapper.toml; musí ležať medzi 15° a 80°
error-project-invalid-face-slope = neplatný prah stien { $slope }° vo freemapper.toml; musí ležať medzi 55° a 85°
error-project-scale-not-allowed = mierka 1:{ $scale } nie je povolená pre { $kind } mapy; použi { $allowed }
error-project-scale-not-allowed-enlarge = mierka 1:{ $scale } nie je povolená pre { $kind } mapy; použi { $allowed }. Kresli v 1:4000, freemapper-print pri tlači zväčší na 1:3000
word-kind-schoolyard = školské
word-kind-sprint = šprintové
word-kind-forest = lesné
error-project-toml = neplatný súbor projektu: { $message }
error-project-unknown-adapter = súbor projektu uvádza adaptér "{ $adapter }", ktorý tento freemapper nemá; môže ho priniesť novší freemapper (`freemapper update`), alebo pochádza z tvojho priečinka adaptérov (zobrazí ho `freemapper adapters list`) či, s --adapters-from-project na príkazovom riadku, z priečinka `adapters` projektu (iba pre projekt, ktorému dôveruješ)
error-project-not-fetched = zatiaľ nie sú stiahnuté žiadne dlaždice; najprv spusti `freemapper fetch`
error-project-ocd = export OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: obrázok a world file nezodpovedajú oblasti mapy; spusti `freemapper derive` znova
error-project-tiles-outdated = oblasť mapy potrebuje dlaždice { $tiles }, ktoré neboli stiahnuté; spusti `freemapper fetch`
error-project-buildings-not-fetched = pre túto oblasť mapy nie sú uložené budovy z ALKIS; najprv spusti `freemapper fetch` online
error-project-orthophoto-not-fetched = pre túto oblasť mapy nie je uložené ortofoto; najprv spusti `freemapper fetch` online
error-project-terrain-not-fetched = pre túto oblasť mapy nie je uložený model terénu; najprv spusti `freemapper fetch` online
error-project-candidates-outdated = uložení kandidáti z OSM nepokrývajú oblasť mapy; spusti `freemapper fetch`
error-project-stored = { $path }: { $message }; spusti `freemapper fetch` znova
error-project-area-too-large = oblasť mapy spolu s okrajom má { $width } × { $height } m, viac než { $limit } m na stranu; zmenši oblasť mapy alebo buffer_m vo freemapper.toml
error-project-area-crosses-country = oblasť mapy leží na oboch stranách štátnej hranice; mapy cez hranicu zatiaľ nie sú podporované, nakresli oblasť v rámci jedného štátu
error-project-permits-not-for-region = poznámky k povoleniam zatiaľ nie sú pre tento región dostupné: pýtajú sa nemeckých služieb a uvádzajú nemecké úrady
error-project-outside-coverage = oblasť mapy leží mimo podporovaných spolkových krajín ({ $states }); freemapper má zatiaľ zdroje dát iba pre ne

## Errors: disk space

error-space-required = nie je dosť miesta na disku pre { $part }. Potrebuje { $needed }, ale { $path } má voľných iba { $left } nad rezervu { $reserve }. Uvoľni miesto a spusti `freemapper fetch` znova
error-space-skipped = freemapper vynecháva { $part } pre nedostatok miesta na disku. Potrebuje { $needed }, ale { $path } má voľných iba { $left } nad rezervu { $reserve }. Uvoľni miesto a spusti `freemapper fetch` znova
word-part-map-data = mapové dáta ALKIS a OSM
word-part-terrain = model terénu
word-part-orthophoto = ortofoto
word-part-surface = model povrchu pre podklad nDSM
word-part-osm-update = aktualizáciu uloženého výrezu OSM
word-part-k-five = berlínsky mapový podklad K5
word-part-berlin-layers = berlínske štruktúry ALKIS, kataster stromov, mapovanie ulíc a bodové vrstvy
word-part-state-layers = úplné štruktúry ALKIS, objekty krajinného modelu a katastre stromov
word-part-point-cloud = laserové mračná bodov pre kandidátov vegetácie

## Errors: downloads

error-fetch-http = sťahovanie { $url } zlyhalo: { $message }
error-fetch-http-temporary = sťahovanie { $url } zlyhalo: { $message }. Služba je pravdepodobne preťažená alebo v údržbe; freemapper sa pýtal až { $attempts }-krát. Spustite beh o niekoľko minút znova; to, čo je už stiahnuté, zostane zachované.
error-fetch-corrupt-archive = stiahnuté { $url } nie je úplný archív dlaždíc: { $message }
error-fetch-corrupt-extract = stiahnuté { $url } nie je úplný výrez OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] dlaždica { $tiles } nie je pokrytá zverejnenými dlaždicami; zmenši oblasť mapy alebo buffer_m vo freemapper.toml
        [few] dlaždice { $tiles } nie sú pokryté zverejnenými dlaždicami; zmenši oblasť mapy alebo buffer_m vo freemapper.toml
       *[other] dlaždice { $tiles } nie sú pokryté zverejnenými dlaždicami; zmenši oblasť mapy alebo buffer_m vo freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline a nie je vo vyrovnávacej pamäti: { $product } dlaždica { $tiles }; spusti `freemapper fetch` online
        [few] offline a nie sú vo vyrovnávacej pamäti: { $product } dlaždice { $tiles }; spusti `freemapper fetch` online
       *[other] offline a nie sú vo vyrovnávacej pamäti: { $product } dlaždice { $tiles }; spusti `freemapper fetch` online
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = model terénu (DGM1)
word-product-image-dsm = model povrchu (bDOM)
word-product-saxony-dtm = model terénu (GeoSN DGM1)
word-product-saxony-dsm = model povrchu (GeoSN DOM1)
error-fetch-osm-extract-missing = vo vyrovnávacej pamäti nie je výrez OSM; spusti `freemapper fetch` online
error-fetch-not-found = { $name } sa na serveri nenašlo
error-fetch-refused = { $url }: server požiadavku odmietol (HTTP { $status })
error-fetch-refused-detail = { $url }: server požiadavku odmietol (HTTP { $status }): { $detail }
error-fetch-too-large = stiahnuté { $url } je väčšie než { $limit } MB; zastavené
error-fetch-not-stored = pre túto oblasť mapy nie je uložené { $label }: { $path } chýba, nedá sa prečítať alebo ju nepokrýva; najprv spusti `freemapper fetch` online
error-fetch-range-ignored = { $url }: server ignoroval rozsah bajtov { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper na túto adresu požiadavky neposiela: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper na túto adresu požiadavky neposiela: adaptér pre { $name } neuvádza hostiteľa { $host }{ $from }. Ak sa služba presunula tam, povoľ hostiteľa cez `freemapper adapters allow-host { $adapter } { $host }` a nahlás to na https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper na túto adresu požiadavky neposiela: adaptér { $adapter } neuvádza hostiteľa { $host }{ $from }. Ak sa služba presunula tam, pridaj hostiteľa do `[net] hosts` manifestu adaptéra
error-fetch-host-redirected-from = {" "}({ $url } presmeruje tam)
error-reason-not-a-host-name = nie je názov hostiteľa
error-reason-not-ascii = názov mimo ASCII sa musí zapísať vo forme xn--
error-reason-not-a-dns-name = nie je názov DNS
error-reason-no-domain = názov tohto počítača alebo bez domény
error-reason-local-name = názov lokálnej siete
error-reason-ip-address = adresa IP, nie názov DNS
error-reason-unexpected-character = neočakávaný znak
error-reason-not-https = nie je https
error-reason-user-info = údaje o používateľovi
error-reason-port = iný port než 443
error-reason-host-not-declared = hostiteľ nie je medzi uvedenými hostiteľmi
error-reason-local-address = adresa tohto počítača alebo lokálnej siete, nie servera na internete
error-reason-too-many-redirects = viac než { $count } presmerovaní
error-reason-redirect-nowhere = presmerovanie (HTTP { $status }) nikam
error-fetch-currency-csv = CSV aktuálnosti, riadok { $line }: { $message }
error-alkis = neplatné dáta ALKIS: { $message }
error-osm = dáta OSM: { $message }

## Errors: templates and rasters

error-derive-zip = archív dlaždíc: { $message }
error-derive-image = obrázok: { $message }
error-derive-no-tiles = nie sú dlaždice, z ktorých by sa dalo odvodzovať
error-derive-area-too-large = oblasť mapy je príliš veľká pre jeden podklad: { $width } × { $height } pixelov (najviac { $limit } na stranu); zmenši oblasť mapy alebo buffer_m vo freemapper.toml
error-raster-world-file = neplatný world file: { $message }
error-raster-misaligned = rastre nie sú zarovnané na rovnakú mriežku pixelov: { $message }
error-raster-pixel-size = rastre majú rôzne veľkosti pixelov: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = neplatný GeoJSON: { $message }
error-area-no-polygon = GeoJSON neobsahuje žiadny mnohouholník
error-area-multi-polygon = MultiPolygon s { $parts } časťami; rozdeľ oblasť mapy na jeden mnohouholník
error-area-out-of-range = súradnica mimo rozsahu: lat { $lat }, lon { $lon }
error-area-not-an-area = nepoužiteľná oblasť mapy: { $message }
error-georef-date = dátum { $date } je mimo platnosti World Magnetic Model
error-georef-map-range = bod E { $easting } N { $northing } leží mimo rozsahu súradníc mapy
error-omap-xml = neplatný XML OOM: { $message }
error-omap-missing-element = prvok <{ $name }> sa nenašiel
error-ocd-duplicate-symbol = číslo symbolu { $symbol } sa používa dvakrát
error-ocd-unknown-symbol = objekt odkazuje na symbol { $symbol }, ktorý v súbore nie je
error-ocd-wrong-type = objekt so symbolom { $symbol } má pre svoj symbol nesprávny typ

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Použitie:
cli-heading-commands = Príkazy:
cli-heading-options = Voľby:
cli-heading-arguments = Argumenty:
cli-help-possible-values = možné hodnoty
cli-help-default = predvolené
cli-arg-help = Vypísať pomocníka
cli-arg-version = Vypísať verziu
cli-help-help = Zobraziť pomocníka k príkazu
cli-arg-help-command = Príkaz, ku ktorému sa má zobraziť pomocník, napríklad run
cli-parse-hint = Ďalšie informácie získaš cez '--help'.
cli-double-click-start = freemapper otvorí v prehliadači stránku, na ktorej vyberiete oblasť mapy. Priečinok projektu je { $path }. Zatvorením tohto okna program ukončíte.
cli-parse-unknown-argument = neočakávaný argument { $arg }
cli-parse-missing-required = chýba povinný argument: { $args }
cli-parse-invalid-value = neplatná hodnota { $value } pre { $arg }
cli-parse-invalid-value-choices = neplatná hodnota { $value } pre { $arg }; možné hodnoty: { $values }
cli-parse-invalid-subcommand = neznámy príkaz { $value }
cli-parse-missing-subcommand = príkaz je povinný
cli-parse-too-many-values = neočakávaná hodnota { $value } pre { $arg }
cli-parse-argument-conflict = { $arg } sa nedá použiť spolu s { $other }

help-freemapper = Zostavuje štartovacie projekty OOM z geodát
help-freemapper-lang = Jazyk textov programu, `de` alebo `en` (predvolene: FREEMAPPER_LANG, potom jazyk systému, potom angličtina)
help-init = Vytvoriť priečinok projektu pre oblasť mapy
help-init-dir = Priečinok projektu, ktorý sa vytvorí
help-init-geojson = Súbor GeoJSON s mnohouholníkom oblasti mapy (EPSG:4326)
help-init-osm = Objekt OSM oblasti mapy, napr. way/992423119
help-init-place = Názov miesta, napr. "Clara-Zetkin-Schule Strausberg"; freemapper použije najlepšiu zhodu s plochou
help-init-part = Ktorú časť oblasti OSM z viacerých oddelených častí použiť (1 je najväčšia)
help-init-kind = Druh mapy
help-arg-scale = Menovateľ mierky nového projektu: schoolyard 1000 (predvolene); sprint 4000 (predvolene) alebo 5000; forest 15000 (predvolene), 10000 alebo 7500. Iné hodnoty sa odmietnu, pretože ich odmieta aj freemapper-print
help-init-title = Názov mapy pre okrajové údaje (predvolene: názov priečinka)
help-export = Zapísať štartovací projekt (mapa OOM, podklady, uvedenie zdrojov) do priečinka projektu
help-export-dir = Priečinok projektu
help-arg-force = Nahradiť existujúci súbor mapy a starý ponechať ako `<name>.omap.bak`
help-export-ocad = Zapísať aj štartovací projekt OCAD do `ocad/` (formát OCAD 12); existujúca mapa OOM potom zostane, ak nezadáš --force
help-export-pdf = Zapísať aj `<name>.pdf`, tlačiteľnú mapu (iba objekty mapy, bez kandidátov); existujúca mapa OOM potom zostane, ak nezadáš --force
help-export-pdf-candidates = Zapísať aj `<name>-candidates.pdf`, kontrolnú tlač: mapu so všetkými kandidátmi, polohovo presne; existujúca mapa OOM potom zostane, ak nezadáš --force
help-export-training = Zapísať aj tréningovú mapu `<name>-training.omap` a jej PDF `<name>-training.pdf`: mapu a všetkých kandidátov, upravené tak, aby sa dali tlačiť ako sú; s --ocad aj `ocad/<name>-training.ocd`. Existujúca mapa OOM potom zostane, ak nezadáš --force, a --force nahradí aj existujúcu tréningovú mapu
help-export-view = S --training: zapísať aj tréningovú mapu bez toho, čo pohľad vynecháva, ako `<name>-training-<view>.omap` a jej PDF (s --ocad aj súbor OCAD). Pohľady: `no-paths` (bez ciest, chodníkov, schodov), `brown-only` (iba hnedé symboly a rám), `no-north-lines`, alebo zoznam `no:<codes>` / `only:<codes>`; kombinuj cez `+`. Voľbu opakuj pre viac súborov
help-export-pdf-scale-bar = Nakresliť grafickú mierku na PDF (predvolene vypnuté, nemecké mapy ju zvyčajne nemajú; číslo mierky zostáva v titulnom bloku)
help-export-pdf-sources = Riadky so zdrojmi na PDF: `auto` vypíše úplný zoznam a skráti ho, iba ak by zabral viac než 5 riadkov, `full` nikdy neskracuje, `credit-only` vypíše iba riadok freemapperu (ODbL a dl-de/by-2-0 potom vyžadujú uviesť zdroje pri odovzdaní mapy). ATTRIBUTION.txt vždy uvádza všetky zdroje
help-fetch = Stiahnuť dáta pre oblasť mapy (ortofoto, model terénu, nDSM, budovy z ALKIS, OSM)
help-fetch-dir = Priečinok projektu
help-fetch-offline = Použiť iba vyrovnávaciu pamäť, bez prístupu k sieti
help-arg-cache-dir = Priečinok vyrovnávacej pamäte pre stiahnuté dlaždice (predvolene: systémový priečinok vyrovnávacej pamäte + `freemapper`)
help-derive = Vypočítať podklady (ortofoto, tieňovaný reliéf, sklon, lokálny reliéf, nDSM), vrstevnice a tvary terénu zo stiahnutých dát
help-derive-dir = Priečinok projektu
help-run = Stiahnuť, odvodiť a exportovať naraz
help-run-dir = Priečinok projektu
help-run-offline = Použiť iba vyrovnávaciu pamäť, bez prístupu k sieti
help-run-ocad = Zapísať aj štartovací projekt OCAD do `ocad/` (formát OCAD 12)
help-run-pdf = Zapísať aj `<name>.pdf`, tlačiteľnú mapu (iba objekty mapy, bez kandidátov)
help-run-pdf-candidates = Zapísať aj `<name>-candidates.pdf`, kontrolnú tlač: mapu so všetkými kandidátmi, polohovo presne
help-run-training = Zapísať aj tréningovú mapu `<name>-training.omap` a jej PDF `<name>-training.pdf`: mapu a všetkých kandidátov, upravené tak, aby sa dali tlačiť ako sú; s --ocad aj `ocad/<name>-training.ocd`. Existujúca tréningová mapa zostane, ak nezadáš --force
help-run-view = S --training: zapísať aj tréningovú mapu bez toho, čo pohľad vynecháva, ako `<name>-training-<view>.omap` a jej PDF (s --ocad aj súbor OCAD). Pohľady: `no-paths` (bez ciest, chodníkov, schodov), `brown-only` (iba hnedé symboly a rám), `no-north-lines`, alebo zoznam `no:<codes>` / `only:<codes>`; kombinuj cez `+`. Existujúci súbor pohľadu zostane, ak nezadáš --force
help-run-pdf-scale-bar = Nakresliť grafickú mierku na PDF (predvolene vypnuté; číslo mierky zostáva v titulnom bloku)
help-run-pdf-sources = Riadky so zdrojmi na PDF: `auto` vypíše úplný zoznam a skráti ho, iba ak by zabral viac než 5 riadkov, `full` nikdy neskracuje, `credit-only` vypíše iba riadok freemapperu (ODbL a dl-de/by-2-0 potom vyžadujú uviesť zdroje pri odovzdaní mapy). ATTRIBUTION.txt vždy uvádza všetky zdroje
help-select = Vybrať oblasť mapy v prehliadači nakreslením alebo výberom miesta. Uloženie zapíše freemapper.toml a v prípade potreby vytvorí priečinok projektu
help-select-dir = Priečinok projektu (pri ukladaní sa vytvorí, ak neexistuje)
help-select-kind = Druh mapy nového projektu
help-select-title = Názov mapy nového projektu
help-select-no-browser = Iba vypísať adresu stránky; neotvárať prehliadač
help-select-port = Port na 127.0.0.1 (predvolene: voľný)
help-select-offline = Stránka sa spustí so zaškrtnutým "Iba vyrovnávacia pamäť, bez siete"
help-select-ocad = Stránka sa spustí so zaškrtnutým štartovacím projektom OCAD
help-select-pdf = Stránka sa spustí so zaškrtnutým tlačiteľným PDF
help-select-pdf-candidates = Stránka sa spustí so zaškrtnutou kontrolnou tlačou (každý kandidát)
help-select-training = Stránka sa spustí so zaškrtnutou tréningovou mapou
help-select-pdf-scale-bar = Stránka sa spustí so zaškrtnutou grafickou mierkou na PDF
help-select-pdf-sources = Stránka sa spustí s touto voľbou riadkov so zdrojmi na PDF, ako pri `run`
help-find = Vypísať miesta OpenStreetMap pre názov, plochy ako prvé, s id pre init --osm
help-find-query = Názov miesta, napr. "Clara-Zetkin-Schule Strausberg"
help-find-json = Vypísať zhody ako JSON, tie isté polia ako nástroj MCP find_place
help-mcp = Sprístupniť nástroje AI agentovi cez MCP (stdin/stdout)
help-update = Nainštalovať novší freemapper z vydaní na GitHube, po opýtaní. Zostavenie zo zdrojového kódu povie, ako ho aktualizovať
help-update-check = Iba skontrolovať: kód ukončenia 10, ak existuje novšia verzia, 0, ak je táto aktuálna
help-update-yes = Nainštalovať bez pýtania; potrebné, keď vstup nie je terminál
help-update-version = Nainštalovať túto verziu (x.y.z) namiesto najnovšej, aj staršiu
help-update-rollback = Vrátiť sa na verziu, ktorú nahradila posledná aktualizácia
help-update-mode = Uložiť, čo robí denná kontrola, keď nájde novšiu verziu (predvolene: notify). Funguje v každom zostavení, lebo iba zapisuje nastavenie
help-adapters = Zobraziť regionálne adaptéry: spolkové krajiny a štáty, pre ktoré má freemapper zdroje dát
help-adapters-list = Vypísať adaptéry s tým, odkiaľ každý pochádza (vstavaný, tvoj priečinok, projekt), jeho revíziou, dňom poslednej kontroly a hostiteľmi, ktorých sa pýta
help-adapters-list-json = Vypísať zoznam ako JSON
help-adapters-list-project = Načítať aj adaptéry z priečinka `adapters` tohto priečinka projektu, ako by to urobilo --adapters-from-project
help-adapters-check = Skontrolovať priečinok adaptéra pred jeho použitím alebo odovzdaním: či by sa načítal, čo by mal vedieť jeho autor, a s --live, či jeho služby odpovedajú. Kód ukončenia 0: prešiel; 1: má zistenia; 2: kontrolu sa nepodarilo spustiť
help-adapters-check-dir = Priečinok adaptéra; jeho názov je id adaptéra
help-adapters-check-live = Opýtať sa aj služieb adaptéra na obdĺžnik, ktorý určuje --bbox, a povedať, čo každá dodáva. Požiadavky zostávajú na hostiteľoch, ktorých manifest uvádza
help-adapters-check-bbox = Obdĺžnik živej kontroly v stupňoch, ako minLon,minLat,maxLon,maxLat. Leží v hraniciach adaptéra; stačí 150 m na 150 m
help-adapters-check-json = Vypísať výsledok ako JSON
help-adapters-probe = Raz sa opýtať každej služby, ktorú uvádzajú vstavané adaptéry, a povedať, či odpovedá: ok, slow, moved alebo error. Návratový kód je 0 aj pri zlyhaní služby; výsledok je odpoveď
help-adapters-probe-json = Vypísať výsledok ako JSON
help-adapters-probe-status = Prečítať stav z minulej noci z tohto súboru (smie chýbať) a vypísať nový stav ako JSON: služba je vypadnutá po dvoch zlých nociach po sebe. Zmeny idú na štandardný chybový výstup
help-adapters-allow-host = Povoliť vstavanému adaptéru jedného ďalšieho hostiteľa: pre krajinu, ktorej služba sa presunula na server, ktorý jej adaptér ešte neuvádza. Hlásenie o odmietnutej adrese povie, o ktorého hostiteľa ide. Hostiteľ sa kontroluje ako hostiteľ z manifestu a freemapper sa naďalej pripája iba k serverom na internete
help-adapters-allow-host-adapter = Id vstavaného adaptéra, ako ho zobrazuje `adapters list`
help-adapters-allow-host-host = Hostiteľ na povolenie: názov DNS, nie adresa, nie port
help-adapters-disallow-host = Odvolať hostiteľa, ktorého povolil allow-host
help-adapters-disallow-host-adapter = Id vstavaného adaptéra
help-adapters-disallow-host-host = Hostiteľ na odvolanie
help-adapters-export = Zapíše vstavaný adaptér spolkovej krajiny ako priečinok adaptéra, ktorý môžete meniť: jeho manifest a súbory, ktoré manifest uvádza, do `<dir>/<id>`. Priečinok ešte nesmie existovať alebo musí byť prázdny. Zmeňte adresy v jeho `manifest.toml`, skontrolujte ho príkazom `adapters check` a vložte do svojho priečinka adaptérov: nahradí potom vstavaný adaptér, s varovaním. Odstránením priečinka sa vstavaný adaptér vráti
help-adapters-export-adapter = Identifikátor vstavaného adaptéra, ako ho ukazuje `adapters list`
help-adapters-export-dir = Priečinok, v ktorom vznikne priečinok `<id>`
help-sources = Čítať a opravovať adresy zdrojov údajov bez nového vydania. Oprava ide do vášho priečinka adaptérov a nahrádza vstavanú adresu
help-sources-list = Vypísať adresy z manifestov s ich cestami a pôvodom každého manifestu. S ID adaptéra len jeho adresy
help-sources-get = Vypísať jednu adresu a nič iné. Návratový kód 1, keď je adaptér alebo cesta neznáma
help-sources-set = Opraviť jednu adresu vo vašom priečinku adaptérov po tom, čo celý manifest prešiel kontrolou. Adresa na hostiteľovi, ktorého manifest nedeklaruje, sa odmietne, ak nezadáte --add-host
help-sources-diff = Ukázať, čo váš priečinok adaptéra mení oproti vstavanému manifestu
help-sources-check = Skontrolovať priečinok vášho adaptéra s týmto ID ako `adapters check`. Návratový kód 0: prešiel; 1: má nálezy; 2: kontrola sa nemohla spustiť
help-sources-schema = Vypísať kľúče manifestu, ktoré obsahujú adresu, s --json ako schému JSON
help-arg-adapter = ID adaptéra, ako ho ukazuje `adapters list`
help-arg-path = Cesta adresy, ako ju ukazuje `sources list`
help-sources-set-url = Nová adresa: https, bez dotazu
help-sources-set-add-host = Pridať aj hostiteľa novej adresy k hostiteľom, ktorých manifest deklaruje
cli-sources-no-user-dir = tento systém nemá priečinok pre vaše adaptéry; nastavte FREEMAPPER_ADAPTER_DIR na absolútnu cestu
cli-sources-bad-id = { $adapter } nie je ID adaptéra
cli-sources-unknown-adapter = freemapper nepozná adaptér { $adapter }; `freemapper adapters list` ukazuje ID
cli-sources-no-manifest = žiadny manifest tohto programu adaptér { $adapter } nepopisuje; jeho adresy sú v kóde
cli-sources-unknown-path-of = adaptér { $adapter } nemá adresu pod { $path }; `freemapper sources list { $adapter }` ukazuje cesty
cli-sources-bad-value = nová adresa musí byť adresa https na pomenovanom hostiteľovi, bez úvodzovky, spätnej lomky, riadiaceho znaku, používateľa a portu
cli-sources-host-not-declared = nová adresa vedie na hostiteľa { $host }, ktorého manifest nedeklaruje; --add-host ho deklaruje
cli-sources-invalid = manifest by s touto zmenou neprešiel kontrolou, preto sa nič nezapísalo: { $findings }
cli-sources-set-created = Priečinok { $folder } adaptéra { $adapter } bol vytvorený z jeho vstavaného manifestu; od ďalšieho spustenia nahrádza vstavaný adaptér.
cli-sources-set-host-added = Hostiteľ { $host } je teraz v [net] hosts; freemapper sa na tento server pre tento adaptér pýta.
cli-sources-set-next = Skontrolujte ho príkazom `freemapper adapters check { $folder }`. Uvedenie zdroja a licencia zostávajú, aké boli: udržujte ich správne, ak služba zmenila prevádzkovateľa. Reštartujte server MCP alebo stránku oblasti.
cli-sources-diff-no-folder = nemáte priečinok adaptéra { $adapter } ({ $folder }); nie je čo porovnávať
cli-sources-diff-none = váš priečinok obsahuje rovnaké adresy ako vstavaný manifest
help-sources-list-json = Vypísať zoznam ako JSON
help-sources-check-live = Opýtať sa aj služieb adaptéra na obdĺžnik, ktorý určuje --bbox, a povedať, čo každá dodáva. Požiadavky zostávajú na hostiteľoch, ktorých manifest uvádza
help-sources-check-bbox = Obdĺžnik živej kontroly v stupňoch, ako minLon,minLat,maxLon,maxLat. Leží v hraniciach adaptéra; stačí 150 m na 150 m
help-sources-check-json = Vypísať výsledok ako JSON
help-sources-schema-json = Vypísať výsledok ako JSON
help-arg-adapters-from-project = Načítať aj adaptéry z priečinka `adapters` priečinka projektu. freemapper každý z nich uvedie spolu s hostiteľmi, ktorých sa pýta. Použi to pre projekt, ktorému dôveruješ: adaptér núti freemapper pýtať sa serverov, ktoré uvádza

## The summary of a run, for the MCP server and the area page

cli-internal-error = interná chyba: { $message }
cli-pdf-sources-invalid = pdf_sources musí byť auto, full alebo credit-only, nie { $value }
cli-view-unknown = pohľad je no-paths, brown-only, no-north-lines, no:<codes> alebo only:<codes>, nie { $value }
cli-view-empty = zoznam { $value } neuvádza žiadny kód symbolu
cli-view-bad-code = { $value } nie je kód symbolu ako 505 alebo 501.6
cli-summary-training-view = Zapísané { $map } a { $pdf } (pohľad na tréningovú mapu)
cli-summary-kept-training-view = Súbory pohľadu { $view } na tréningovú mapu zostali zachované; na obnovenie zadaj --force
cli-summary-training-view-failed = Pohľad na tréningovú mapu zlyhal: { $error }
cli-estimate-disk-unreadable = voľné miesto na disku { $path } sa nedá prečítať ({ $message })
cli-summary-wrote-map = Zapísaná mapa { $path }
cli-summary-kept-map = Existujúca mapa { $path } zostala zachovaná (môže obsahovať prácu mapéra)
cli-summary-map-lacks-templates = Mapa zatiaľ neodkazuje na rastrové podklady. Spusti pipeline s force, aby ju nahradil, a starú ponechaj ako .omap.bak.
cli-summary-map-lacks-buildings = Mapa zatiaľ nemá budovy z ALKIS. Spusti pipeline s force, aby sa pridali; nahradí mapu a starú ponechá ako .omap.bak.
cli-summary-map-lacks-canopies = Mape chýbajú prístrešky a prístrešky pre autá z ALKIS (522), ktoré sú teraz objektmi mapy. Spusti pipeline s force, aby sa pridali; nahradí mapu a starú ponechá ako .omap.bak.
cli-summary-map-light = Základná mapa: { $joined } čiar spojených, { $left } skrytých alebo príliš malých objektov ostalo vo vrstvách kandidátov (nič sa neposunulo)
cli-summary-map-light-kept = Vrstvy kandidátov teraz obsahujú aj { $left } objektov alebo úsekov, ktoré táto verzia odoberá zo základnej mapy (označené freemapper:generalized). Zachovaná mapa ich ešte má, takže sa zobrazia dvakrát. Spusti pipeline s force, aby sa mapa vyčistila; starú ponechá ako .omap.bak.
cli-summary-map-old-candidates = Mapa stále uvádza templates/candidates.omap; tento beh ho naplnil všetkými kandidátmi. Kandidáti teraz prichádzajú aj v siedmich vrstvách, každá v jednom súbore (templates/candidates-*.omap). Mapa ich uvedie po behu s force (stará mapa zostane ako .omap.bak) alebo ich mapér pridá v OOM (Templates > Open template…).
cli-summary-wrote-ocad = Zapísaná mapa OCAD { $path } ({ $count } objektov)
cli-summary-kept-ocad = Existujúca mapa OCAD { $path } zostala zachovaná (môže obsahovať prácu mapéra; force ju nahradí)
cli-summary-ocad-old-candidates = Mapa OCAD stále uvádza { $path } ako podkladovú mapu; tento beh ju naplnil všetkými kandidátmi. Kandidáti teraz prichádzajú aj v siedmich vrstvách, každá v jednom súbore (-candidates-*.ocd). Mapa OCAD ich uvedie po behu s force (stará mapa zostane ako .ocd.bak) alebo ich mapér pridá v OCAD ako podkladové mapy.
cli-summary-ocad-left-out = Z mapy OCAD vynechané: { $skipped } objektov a { $dropped } degenerovaných častí
cli-summary-ocad-symbols = V OCAD zvoľ Symbol > Load Colors and Symbols From… a vyber oficiálnu sadu symbolov{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , školskú sadu IOF 2019
       *[other] {" "}ISSprOM 2019
    }; nahradí zástupné symboly podľa čísla.
cli-summary-ocad-failed = Export OCAD zlyhal: { $error }
cli-summary-pdf-without-credits = PDF bez uvedenia zdrojov: ODbL a dl-de/by-2-0 vyžadujú uviesť zdroje pri odovzdaní mapy; pozri ATTRIBUTION.txt
cli-summary-wrote-pdf = Zapísaná mapa { $path } (bez kandidátov)
cli-summary-pdf-failed = Export PDF zlyhal: { $error }
cli-summary-wrote-pdf-candidates = Zapísaná kontrolná tlač { $path } (mapa so všetkými kandidátmi, polohovo presne)
cli-summary-pdf-candidates-failed = Export kontrolnej tlače zlyhal: { $error }
cli-summary-training-failed = Export tréningovej mapy zlyhal: { $error }
cli-summary-kept-training = Existujúca tréningová mapa { $path } zostala zachovaná, aby ju beh neprepísal, ak bola vytlačená alebo odovzdaná; na obnovenie ju zmaž alebo nastav force

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = tento systém nemá domovský priečinok; použi absolútnu cestu
mcp-error-relative-path = { $path } je relatívna cesta; použi absolútnu cestu alebo cestu začínajúcu ~
mcp-error-home-folder = { $path } je samotný domovský priečinok; použi priečinok v ňom, napr. ~/maps/school
mcp-error-no-project = v { $path } nie je projekt freemapperu; najprv použi create_project
mcp-error-unknown-kind = neznámy druh { $kind }; použi "schoolyard", "sprint" alebo "forest"
mcp-error-reading = načítava sa { $path }: { $error }
mcp-error-one-area-source = zadaj práve jedno z osm_id (z find_place) alebo geojson
mcp-error-page-open = stránka výberu oblasti pre tento projekt je ešte otvorená; popros používateľa, aby oblasť uložil a na stránke stlačil Hotovo, potom spusti pipeline
mcp-error-no-job = úloha { $job } neexistuje; start_pipeline a permits_for_area vracajú id úloh tejto relácie
mcp-error-job-running = { $job } ešte beží pre { $path }; beží vždy len jedna pipeline, pretože projekty zdieľajú vyrovnávaciu pamäť sťahovania. Spusti túto, keď project_status ohlási finished alebo failed
mcp-error-page-start = stránku sa nepodarilo spustiť: { $error }
mcp-created-project = Projekt { $path } vytvorený pre { $area } ({ $kind }, 1:{ $scale }). Ďalej: estimate_download, potom start_pipeline.
mcp-created-project-given-area = Projekt { $path } vytvorený pre zadanú oblasť ({ $kind }, 1:{ $scale }). Ďalej: estimate_download, potom start_pipeline.
mcp-page-already-open = Stránka pre tento priečinok je už otvorená; ponecháva nastavenia, s ktorými bola otvorená.
mcp-page-next = Popros používateľa, aby na stránke vybral oblasť, stlačil Uložiť a potom Hotovo.
mcp-preview-of = Náhľad vrstvy { $layer }
mcp-permits-next = Poznámky k povoleniam sa pripravujú. Pýtaj sa project_status s týmto job_id; keď skončí, permits_md obsahuje text permits.md. Odovzdaj ho ďalej ako orientačnú pomôcku, nie právnu radu.
mcp-tool-check-update = Či existuje novšie vydanie freemapperu: jeho verzia, dátum vydania, poznámky k vydaniu a veľkosť sťahovania, alebo že táto verzia je aktuálna. Nič neinštaluje.
mcp-tool-create-project = Vytvoriť priečinok projektu pre oblasť mapy. Zadaj osm_id (z find_place) alebo súbor GeoJSON. kind je schoolyard (1:1000, predvolené), sprint (1:4000) alebo forest (1:15000).
mcp-param-create-project-dir = Nový priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-param-create-project-geojson = Namiesto osm_id: súbor GeoJSON s mnohouholníkom oblasti mapy (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (predvolené, 1:1000), "sprint" (1:4000) alebo "forest" (1:15000).
mcp-param-create-project-osm-id = Objekt OSM oblasti mapy z find_place, napr. "way/992423119".
mcp-param-create-project-part = S osm_id: ktorú časť oblasti z viacerých oddelených častí použiť (1 je najväčšia).
mcp-param-create-project-scale = Menovateľ mierky: schoolyard 1000 (predvolené); sprint 4000 (predvolené) alebo 5000; forest 15000 (predvolené), 10000 alebo 7500. Iné hodnoty sa odmietnu; pre 1:3000 kresli v 4000, freemapper-print pri tlači zväčší.
mcp-param-create-project-title = Názov mapy pre okrajové údaje (predvolene: názov priečinka).
mcp-tool-estimate-download = Čo start_pipeline stiahne pre projekt: dlaždice, každé sťahovanie s veľkosťou v MB, či je vo vyrovnávacej pamäti a či sa zmestí na disk, ešte nestiahnuté MB a pod disk voľné miesto zväzkov vyrovnávacej pamäte a projektu so správami o tom, čo sa nezmestí (povedz to používateľovi).
mcp-param-estimate-download-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-tool-find-place = Vyhľadať školu, park alebo iné miesto podľa názvu (OpenStreetMap Nominatim). Vráti najviac 5 zhôd, plochy ako prvé; oblasťou mapy môžu byť len zhody s has_area. Prázdny zoznam znamená, že sa nič nenašlo.
mcp-param-find-place-query = Názov školy alebo miesta, napr. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Nainštalovať najnovšie vydanie freemapperu cez tento program. Zavolaj až po súhlase osoby, s confirm true. Nová verzia sa použije, keď asistent znova spustí server freemapperu.
mcp-param-install-update-confirm = true, keď osoba súhlasila s inštaláciou novšej verzie; bez toho sa nič neinštaluje.
mcp-tool-list-layers = Vrstvy projektu: mapa, sedem vrstiev kandidátov a rastrové podklady, každá so svojím súborom, či už vznikla a ktorý krok ju vytvára.
mcp-param-list-layers-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-tool-open-area-selector = Otvoriť v prehliadači stránku, kde používateľ nakreslí oblasť mapy (obdĺžnik alebo mnohouholník) alebo vyberie miesto, uvidí dlaždice a veľkosť sťahovania a uloží ju; uloženie vytvorí priečinok projektu alebo zmení jeho oblasť. Vráti URL, ktorú dáš používateľovi. Stránka beží, kým používateľ nestlačí Hotovo; opakované volanie pre ten istý priečinok vráti tú istú URL.
mcp-param-open-area-selector-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~); vytvorí sa, keď používateľ uloží.
mcp-param-open-area-selector-kind = Druh mapy nového projektu: "schoolyard" (predvolené, 1:1000), "sprint" (1:4000) alebo "forest" (1:15000).
mcp-param-open-area-selector-ocad = Panel behu sa spustí so zaškrtnutým štartovacím projektom OCAD. Predvolene false.
mcp-param-open-area-selector-offline = Panel behu sa spustí so zaškrtnutým "Iba vyrovnávacia pamäť, bez siete". Predvolene false.
mcp-param-open-area-selector-open-browser = Otvoriť stránku aj v predvolenom prehliadači tohto počítača. Predvolene true.
mcp-param-open-area-selector-pdf = Panel behu sa spustí so zaškrtnutým tlačiteľným PDF. Predvolene false.
mcp-param-open-area-selector-pdf-candidates = Panel behu sa spustí so zaškrtnutou kontrolnou tlačou (každý kandidát). Predvolene false.
mcp-param-open-area-selector-pdf-scale-bar = Panel behu sa spustí so zaškrtnutou grafickou mierkou na PDF. Predvolene false.
mcp-param-open-area-selector-pdf-sources = Riadky so zdrojmi na PDF, s ktorými sa panel spustí: "auto" (predvolené), "full" alebo "credit-only", ako pri start_pipeline.
mcp-param-open-area-selector-permits = Panel behu sa spustí so zaškrtnutými poznámkami k povoleniam. Predvolene false.
mcp-param-open-area-selector-scale = Menovateľ mierky nového projektu: schoolyard 1000 (predvolené); sprint 4000 (predvolené) alebo 5000; forest 15000 (predvolené), 10000 alebo 7500. Iné hodnoty sa odmietnu.
mcp-param-open-area-selector-title = Názov mapy nového projektu.
mcp-param-open-area-selector-training = Panel behu sa spustí so zaškrtnutou tréningovou mapou. Predvolene false.
mcp-tool-preview-layer = Náhľad PNG (najviac 800 px) vrstvy: map (obrázok objektov mapy, ktorý zapísal posledný beh, bez kandidátov) alebo rastrový podklad: orthophoto, hillshade, slope, relief, ndsm, alebo k5 v Berlíne.
mcp-param-preview-layer-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" alebo "k5" (iba Berlín).
mcp-tool-project-status = Stav úlohy start_pipeline alebo permits_for_area: running, finished alebo failed, posledné riadky priebehu, upozornenia, zapísané súbory a výsledok alebo chyba. Dokončená úloha permits_for_area nesie aj text permits.md ako permits_md.
mcp-param-project-status-job-id = Id úlohy zo start_pipeline alebo permits_for_area, napr. "job-1".
mcp-tool-permits-for-area = Pripraviť poznámky k povoleniam projektu: správne celky oblasti mapy, chránené územia v nej a kto tam spravuje pozemky, s úradmi, na ktoré sa obrátiť; orientačná pomôcka, nie právna rada. Zapíše permits.md a permits.json do priečinka. Beží na pozadí a vráti job_id; pýtaj sa project_status, ktorého odpoveď nesie text permits.md ako permits_md, keď je úloha dokončená. Beží vždy len jedna úloha.
mcp-param-permits-for-area-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-param-permits-for-area-offline = Čítať iba dáta uložené v projekte skorším behom, bez prístupu k sieti. Predvolene false.
mcp-tool-start-pipeline = Stiahnuť dáta, vypočítať podklady a zapísať štartovací projekt (fetch, derive, export). Beží na pozadí a vráti job_id; pýtaj sa project_status. Beží vždy len jedna pipeline. Existujúcu mapu ponechá, ak force nie je true.
mcp-param-start-pipeline-dir = Priečinok projektu (absolútna cesta alebo začínajúca ~).
mcp-param-start-pipeline-force = Nahradiť existujúci súbor mapy a starý ponechať ako .omap.bak. Predvolene false.
mcp-param-start-pipeline-ocad = Zapísať aj štartovací projekt OCAD do priečinka ocad/ (formát OCAD 12, pre používateľov, ktorí upravujú v OCAD). Predvolene false.
mcp-param-start-pipeline-offline = Použiť iba vyrovnávaciu pamäť sťahovania, bez prístupu k sieti. Predvolene false.
mcp-param-start-pipeline-pdf = Zapísať aj <name>.pdf, mapu pripravenú na tlač (iba objekty mapy, bez kandidátov). Predvolene false.
mcp-param-start-pipeline-pdf-candidates = Zapísať aj <name>-candidates.pdf, kontrolnú tlač: mapu so všetkými kandidátmi, polohovo presne (skontroluj ich v teréne). Predvolene false.
mcp-param-start-pipeline-pdf-scale-bar = Nakresliť grafickú mierku na PDF. Predvolene false (nemecké mapy ju zvyčajne nemajú; číslo mierky zostáva v titulnom bloku).
mcp-param-start-pipeline-pdf-sources = Riadky so zdrojmi na PDF: "auto" (predvolené; úplný zoznam, skrátený, iba keď zaberie viac než 5 riadkov), "full" (nikdy neskrátený) alebo "credit-only" (iba riadok freemapperu; ODbL a dl-de/by-2-0 potom vyžadujú uviesť zdroje pri odovzdaní mapy). ATTRIBUTION.txt vždy uvádza všetky zdroje.
mcp-param-start-pipeline-permits = Zapísať aj poznámky k povoleniam (permits.md a permits.json) po mape: správne celky, chránené územia a kto spravuje pozemky. Zlyhanie tam je upozornenie, nie zlyhaná úloha. Predvolene false.
mcp-param-start-pipeline-training = Zapísať aj tréningovú mapu <name>-training.omap a jej PDF <name>-training.pdf: mapu a všetkých kandidátov, upravené tak, aby sa dali tlačiť ako sú; s ocad aj ocad/<name>-training.ocd. Existujúca tréningová mapa zostane, ak force nie je true. Predvolene false.
mcp-param-start-pipeline-training-views = S training: zapísať aj tréningovú mapu bez toho, čo pohľad vynecháva, jednu sadu súborov na záznam: "no-paths", "brown-only", "no-north-lines" alebo zoznam ako "no:505,506" či "only:101,102", spájaný cez +. Súbory sú <name>-training-<view>.omap a jeho PDF.
mcp-instructions = freemapper vytvára štartovacie projekty OpenOrienteering Mapper pre orientačné mapy (školský dvor 1:1000, šprint 1:4000, les 1:15000) z oficiálnych geodát { $states } a z OSM. Postup: find_place (vyhľadaj školu alebo oblasť podľa názvu; vyber výsledok s has_area) → create_project (nový priečinok, absolútna cesta, s osm_id) → estimate_download (povedz používateľovi, koľko sa stiahne) → start_pipeline (vráti id úlohy; sťahovanie môže trvať minúty) → project_status (pýtaj sa každých pár sekúnd, kým stav nie je finished alebo failed) → list_layers a preview_layer (ukáž používateľovi podklady). Používateľ potom otvorí súbor mapy v OpenOrienteering Mapper. Keď nevyhovuje žiadne miesto alebo chce používateľ oblasť nakresliť, open_area_selector otvorí v prehliadači stránku, kde používateľ oblasť nakreslí alebo vyberie a uloží (tým sa vytvorí projekt); pokračuj s estimate_download, keď používateľ povie, že je uložená. Cesty musia byť absolútne alebo začínať ~. Beží vždy len jedna pipeline. permits_for_area pripraví poznámky k povoleniam projektu (správne celky, chránené územia, kto spravuje pozemky; orientačná pomôcka, nie právna rada) ako úlohu ako start_pipeline. check_update povie, či existuje novší freemapper; install_update ho nainštaluje, len s confirm: true po súhlase osoby, a asistent potom znova spustí server freemapperu.

cli-contour-interval-saved = Interval vrstevníc uložený vo freemapper.toml: { $interval } m; derive vedie vrstevnice podľa neho
cli-contour-interval-auto = Interval vrstevníc vo freemapper.toml: automaticky (podľa druhu mapy; lesná mapa má 2,5 m v rovine a 10 m v strmom teréne)
cli-steep-slope-saved = Prah strmosti uložený vo freemapper.toml: { $slope }°; derive podľa neho hľadá zemné stupne a skalné steny
cli-steep-slope-auto = Prah strmosti vo freemapper.toml: predvolený (asi 27°)
cli-face-slope-saved = Prah stien uložený vo freemapper.toml: { $slope }°; derive podľa neho hľadá plochy skalných stien
cli-face-slope-auto = Prah stien vo freemapper.toml: predvolený (70°)
help-arg-contour-interval = Interval vrstevníc v metroch, napr. `2.5`, alebo `auto` podľa druhu mapy (lesná mapa: 5 m, 2,5 m v rovine, 10 m v strmom teréne). Uložené vo freemapper.toml; derive vedie vrstevnice podľa neho
help-arg-steep-slope = Uhol svahu v stupňoch, od ktorého sa terén pri hľadaní zemných stupňov a skalných stien považuje za strmý, napr. `55`, alebo `auto` pre predvolených 27°. Zvýš ho (55 až 65) v skalných mestách a roklinách, kde je strmý takmer každý svah a stupne so stenami splývajú do jednej škvrny. Uložené vo freemapper.toml; derive podľa neho hľadá stupne a steny
help-arg-face-slope = Uhol svahu v stupňoch, od ktorého je terén skalnou stenou (čierna plocha, značka 206), napr. `75`, alebo `auto` pre predvolených 70°. 65 dáva viac a väčších plôch, aj na strmých svahoch; 75 nechá len skutočné steny. Uloží sa do freemapper.toml; derive podľa neho hľadá skalné steny
cli-content-saved = Nastavenie obsahu uložené vo freemapper.toml: { $summary }
cli-content-invalid = Nastavenie obsahu sa nedá prečítať: { $detail }
help-arg-content = Čo ide rovno na mapu: `safe` ponechá každý objekt tam, kam ho zaradí zostavenie, `balanced` (predvolené) dá na mapu objekty z katastra a registrov, `full` dá na mapu všetko. Voľba sa uloží vo freemapper.toml
help-arg-content-set = Poslať skupinu objektov na `map`, `candidates` alebo `off`, ako `GROUP=PLACE`, napr. `walls=map`; možno opakovať. Skupiny: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Uložené vo freemapper.toml
mcp-param-create-project-content = Čo ide rovno na mapu: "safe", "balanced" (predvolené) alebo "full". Uložené v projekte. balanced dá na mapu objekty z katastra a registrov, full dá na mapu všetko.
mcp-param-create-project-content-set = Skupiny poslané inam, než hovorí profil, ako "group=place" s place map, candidates alebo off, napr. "walls=map". Skupiny: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Interval vrstevníc v metroch, napr. 2.5; 0 sa vráti k automatickej voľbe (podľa druhu mapy; lesná mapa má 2,5 m v rovine a 10 m v strmom teréne). Uložené v projekte; ďalší derive vedie vrstevnice podľa neho.
mcp-param-create-project-steep-slope = Uhol svahu v stupňoch, od ktorého sa terén pri hľadaní zemných stupňov a skalných stien považuje za strmý, napr. 55; 0 vráti predvolených 27°. Zvýš ho (55 až 65) v skalných mestách a roklinách, kde je strmý takmer každý svah a stupne so stenami splývajú do jednej škvrny. Uložené v projekte; ďalší derive podľa neho hľadá stupne a steny.
mcp-param-create-project-face-slope = Uhol svahu v stupňoch, od ktorého je terén skalnou stenou (čierna plocha, značka 206), napr. 75; 0 vráti predvolených 70°. 65 dáva viac a väčších plôch, aj na strmých svahoch; 75 nechá len skutočné steny. Uloží sa do projektu; nasledujúci derive podľa neho hľadá skalné steny.
mcp-param-start-pipeline-content = Zmeniť pred behom, čo ide rovno na mapu: "safe", "balanced" alebo "full". Uložené v projekte. Predvolene: nastavenie projektu, inak balanced.
mcp-param-start-pipeline-content-set = Skupiny poslané inam, než hovorí profil, ako "group=place" s place map, candidates alebo off, napr. "fences=candidates". Uložené v projekte.
mcp-param-start-pipeline-contour-interval = Interval vrstevníc v metroch, napr. 2.5; 0 sa vráti k automatickej voľbe (podľa druhu mapy; lesná mapa má 2,5 m v rovine a 10 m v strmom teréne). Uložené v projekte; ďalší derive vedie vrstevnice podľa neho.
mcp-param-start-pipeline-steep-slope = Uhol svahu v stupňoch, od ktorého sa terén pri hľadaní zemných stupňov a skalných stien považuje za strmý, napr. 55; 0 vráti predvolených 27°. Zvýš ho (55 až 65) v skalných mestách a roklinách, kde je strmý takmer každý svah a stupne so stenami splývajú do jednej škvrny. Uložené v projekte; ďalší derive podľa neho hľadá stupne a steny.
mcp-param-start-pipeline-face-slope = Uhol svahu v stupňoch, od ktorého je terén skalnou stenou (čierna plocha, značka 206), napr. 75; 0 vráti predvolených 70°. 65 dáva viac a väčších plôch, aj na strmých svahoch; 75 nechá len skutočné steny. Uloží sa do projektu; nasledujúci derive podľa neho hľadá skalné steny.
mcp-error-content = Nastavenie obsahu sa nedá prečítať: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Zapísať poznámky k povoleniam projektu (permits.md a permits.json): správne celky, chránené územia a prevádzkovateľov, ktorých sa oblasť mapy dotýka, a koho to zvyčajne znamená požiadať. Pýta sa BfN a BKG a, kde má spolková krajina oblasti mapy vlastné otvorené služby, aj tých cez hostiteľov jej adaptéra (zoznam v príručke, kapitola 12); v Severnom Porýní-Vestfálsku sa pritom prvýkrát stiahne asi 32 MB, v Brandenbursku asi 4 MB, neskôr už len novšie súbory. Číta už uložený výrez OSM; --offline použije, čo sa stiahlo predtým
help-permits-dir = Priečinok projektu
help-permits-offline = Použiť iba predtým stiahnuté dáta, bez prístupu k sieti
help-run-permits = Zapísať aj poznámky k povoleniam (permits.md a permits.json) po mape; pýta sa BfN, BKG a otvorených služieb spolkovej krajiny (príručka, kapitola 12). Zlyhanie tam je upozornenie a mapa zostane
cli-wrote-permits = Zapísané poznámky k povoleniam { $path }
cli-summary-wrote-permits = Zapísané poznámky k povoleniam { $path }
cli-permits-failed = poznámky k povoleniam sa nepodarilo zapísať: { $error }
cli-permits-source-left-out = poznámky k povoleniam vynechávajú { $source }: { $reason }
cli-permits-not-read = nebolo prečítané
error-permits-stored = { $path }: { $message }; stiahni poznámky k povoleniam znova online
error-permits-nothing-read = nepodarilo sa prečítať žiaden zdroj poznámok k povoleniam ({ $missing }); najprv ich stiahni online
error-permits-missing-not-read = { $source }: neprečítané

permits-title = Poznámky k povoleniam
permits-disclaimer = Orientačná pomôcka, nie právna rada. Rozhodujú príslušné úrady. Dátum a presnosť dát nájdeš v tabuľke pod "Zdroje a dátum dát"; plochy môžu chýbať alebo byť zastarané (BfN: zlučované raz ročne, "Nicht für Planungszwecke geeignet.", teda nevhodné na plánovanie).
permits-disclaimer-regional = Orientačná pomôcka, nie právna rada. Rozhodujú príslušné úrady. Dátum a presnosť dát nájdeš v tabuľke pod "Zdroje a dátum dát"; plochy môžu chýbať alebo byť zastarané.
permits-bfn-merge = BfN zlučuje dáta spolkových krajín raz ročne a označuje ich ako nevhodné na plánovanie. Rozhodujú úrady spolkovej krajiny.
permits-created = Vytvorené { $date } pre oblasť { $kind ->
        [schoolyard] mapy školského dvora
        [forest] lesnej mapy
       *[other] šprintovej mapy
    } ({ $area } ha).
permits-section-admin = Správne celky
permits-section-protected = Chránené územia (BfN)
permits-section-osm-protected = Ďalšie chránené územia z OpenStreetMap
permits-section-operators = Prevádzkovatelia a vlastníci podľa OpenStreetMap
permits-section-hints = Upozornenia
permits-section-sources = Zdroje a dátum dát
permits-none-admin = VG250 pre oblasť mapy neuvádza žiadny správny celok.
permits-none-protected = Podľa BfN v oblasti mapy neleží žiadne chránené územie.
permits-none-osm-protected = OpenStreetMap v oblasti mapy neuvádza žiadne chránené územie, ktoré BfN ešte nevedie.
permits-none-operators = OpenStreetMap pre oblasť mapy neuvádza žiadneho prevádzkovateľa ani vlastníka.
permits-none-hints = Žiadne upozornenia.
permits-section-not-read = Dáta zdroja { $source } sa neprečítali, preto je táto časť prázdna, hoci záznamy môžu existovať.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Správne oblasti 1:250 000 (VG250) z BKG
        [bfn] Chránené územia v Nemecku z BfN
       *[osm] OpenStreetMap, z výrezu, ktorý freemapper drží vo vyrovnávacej pamäti
    }
permits-level = { $level ->
        [land] Spolková krajina (Land)
        [kreis] Okres (Kreis)
        [verwaltungsgemeinschaft] Amt alebo Verwaltungsgemeinschaft
       *[gemeinde] Obec (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Dotknuté na okraji (menej než 0,5 % oblasti mapy):
permits-at-edge-generalised = VG250 je generalizované na mierku 1:250 000. Celok, ktorý sa len dotýka okraja, môže byť jej dôsledkom.
permits-kind = { $kind ->
        [nationalpark] Národný park
        [nationales-naturmonument] Národná prírodná pamiatka
        [naturschutzgebiet] Prírodná rezervácia (Naturschutzgebiet)
        [ffh] Územie FFH (Natura 2000)
        [vogelschutzgebiet] Chránené vtáčie územie (Natura 2000)
        [biosphaerenreservat] Biosférická rezervácia
        [biosphaerenreservat-zone] Zóna biosférickej rezervácie
        [landschaftsschutzgebiet] Chránená krajinná oblasť (Landschaftsschutzgebiet)
       *[naturpark] Prírodný park
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Vyhlásené { $date }
permits-protected-title = Právny základ: { $title }
permits-protected-link = Právny text: { $url }
permits-protected-site-link = Stránka lokality (Natura 2000): { $url }
permits-protected-year = Rok dát v BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = trieda ochrany { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = bez názvu
permits-osm-operator = Prevádzkovateľ podľa OSM: { $operator }
permits-osm-enclosing-gap = Plochy OpenStreetMap, ktoré obklopujú celú oblasť mapy bez rohu v nej, tu môžu chýbať, rovnako ako chránené územia mapované v OSM ako relácia s `type=boundary`. Časti z BfN a VG250 sa to netýka.
permits-osm-enclosing-gap-short = Plochy OpenStreetMap, ktoré obklopujú celé územie mapy bez toho, aby v ňom ležal niektorý ich roh, tu môžu chýbať.
permits-operator-line = { $name } ({ $roles ->
        [operator] prevádzkovateľ
        [owner] vlastník
       *[both] prevádzkovateľ a vlastník
    }): { $objects }
permits-column-source = Zdroj
permits-column-state = Stav
permits-column-credit = Uvedenie zdroja a licencia
permits-state-read = prečítané, stiahnuté { $date }
permits-state-read-osm = prečítané, výrez OSM z { $date }
permits-state-unreachable = nedostupné ({ $reason })
permits-state-not-read = neprečítané
permits-no-credit = nepoužité
permits-hint-city-state = Oblasť mapy leží v Berlíne alebo Hamburgu. Tam je zvyčajne príslušný Bezirksamt (okresný úrad) alebo úrad spolkovej krajiny. Tieto poznámky Bezirk zatiaľ neurčujú.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Chránené územia sa týkajú nižšieho orgánu ochrany prírody (untere Naturschutzbehörde). Tu je to samotné mesto { $kreis }.
        [no] Chránené územia sa týkajú nižšieho orgánu ochrany prírody (untere Naturschutzbehörde), zvyčajne { $kreis }.
       *[unknown] Chránené územia sa týkajú príslušného nižšieho orgánu ochrany prírody (untere Naturschutzbehörde), zvyčajne Landkreis alebo kreisfreie Stadt.
    }
permits-hint-national-park = Národný park má vlastné pravidlá. Obráť sa na správu národného parku včas.
permits-hint-nsg-paths = V prírodnej rezervácii sa zvyčajne treba držať ciest. Pre beh mimo ciest sa obráť na orgán ochrany prírody.
permits-hint-natura2000 = Územie FFH alebo chránené vtáčie územie (Natura 2000) sa dotýka oblasti mapy. Pri podujatí môže orgán ochrany prírody požadovať posúdenie jeho vplyvu.
permits-hint-protected-areas-pl = V území mapy ležia chránené územia alebo chránené objekty ochrany prírody. Pri národnom parku, krajinnom parku alebo prírodnej rezervácii je zvyčajným kontaktom správa parku alebo rezervácie; pri ostatných územiach a objektoch vie zvyčajne povedať, kto je príslušný, Regionálne riaditeľstvo ochrany životného prostredia (RDOŚ) vojvodstva. Opýtajte sa včas, ak tam plánujete podujatie.
permits-hint-order-office = { $amt ->
        [none] Za ulice, námestia a iné verejné pozemky je zvyčajne príslušný Ordnungsamt (úrad verejného poriadku) obce (Gemeinde) { $gemeinde }.
       *[other] Za ulice, námestia a iné verejné pozemky je zvyčajne príslušný Ordnungsamt (úrad verejného poriadku): { $amt }, pre obec (Gemeinde) { $gemeinde }.
    }
permits-hint-park-operator = Prevádzkovateľ parku alebo záhrady v oblasti mapy podľa OpenStreetMap: { $operator }. Pred použitím sa ho opýtaj.
permits-hint-school-operator = Prevádzkovateľ školy alebo zariadenia podľa OpenStreetMap: { $operator }. Pre jej areál sa obráť na vedenie školy alebo tento orgán.
permits-section-districts = Bezirke (obvody, { $source })
permits-section-green-spaces = Zelené plochy ({ $source })
permits-section-owners = Pozemky spolkovej krajiny a Bundu ({ $source })
permits-section-monuments = Pamiatky ({ $source })
permits-section-protected-objects = Prírodné pamiatky a iné chránené objekty ({ $source })
permits-none-districts = Podľa { $source } oblasť mapy neleží v žiadnom Bezirk.
permits-none-green-spaces = Podľa { $source } v oblasti mapy neleží žiadna verejná zelená plocha.
permits-none-owners = Podľa { $source } v oblasti mapy neleží žiadna parcela spolkovej krajiny ani Bundu.
permits-none-monuments = Podľa { $source } v oblasti mapy neleží žiadna pamiatka.
permits-none-protected-objects = Podľa { $source } v oblasti mapy neleží žiadny z chránených objektov, ktoré vedie.
permits-none-sampled = V opýtaných bodoch { $source } tu nič neuvádza.
permits-gaps = Dáta zdroja { $source } nie sú úplné: { $gaps }.
permits-sampled = { $source } odpovedá len pre jednotlivé body. freemapper sa pýtal v mriežke bodov nad mapou: podiely sú odhady a to, čo leží medzi bodmi, môže chýbať.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = určenie: { $dedication }
permits-playgrounds-none = V oblasti mapy neleží žiadne verejné ihrisko.
permits-playgrounds-one = V oblasti mapy leží jedno verejné ihrisko. Samostatne sa neuvádza.
permits-playgrounds-many = V oblasti mapy leží { $count } verejných ihrísk. Neuvádzajú sa po jednom.
permits-owner-other = Iní verejní vlastníci
permits-owners-rest = Pre zvyšok oblasti dáta žiadneho vlastníka neuvádzajú.
permits-monuments-total = Pamiatky celé alebo čiastočne v oblasti mapy: { $count }.{ $links ->
        [yes] {" "}Odkazy vedú na ich záznamy v zozname pamiatok.
       *[no] {""}
    }
permits-monuments-links-left = Ďalších { $count } záznamov sa tu iba počíta, bez odkazu.
permits-object-kind = { $kind ->
        [natural-monument] Prírodná pamiatka (Naturdenkmal)
        [protected-landscape-part] Chránený krajinný prvok (geschützter Landschaftsbestandteil)
        [biotope] Zákonom chránený biotop (geschütztes Biotop)
        [national-park] Národný park (park narodowy)
        [park-buffer-zone] Ochranné pásmo parku (otulina)
        [landscape-park] Krajinný park (park krajobrazowy)
        [nature-reserve] Prírodná rezervácia (rezerwat przyrody)
        [protected-landscape-area] Chránená krajinná oblasť (obszar chronionego krajobrazu)
        [natura2000-habitats] Lokalita Natura 2000 pre biotopy (SOO)
        [natura2000-birds] Lokalita Natura 2000 pre vtáky (OSO)
        [ecological-site] Ekologická plocha (użytek ekologiczny)
        [landscape-complex] Prírodný a krajinný celok (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentačná lokalita (stanowisko dokumentacyjne)
        [natural-monument-pl] Prírodná pamiatka (pomnik przyrody)
        [national-park-cz] Národný park (národní park, NP)
        [protected-landscape-area-cz] Chránená krajinná oblasť (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Národná prírodná rezervácia (národní přírodní rezervace, NPR)
        [national-nature-monument] Národná prírodná pamiatka (národní přírodní památka, NPP)
        [nature-reserve-cz] Prírodná rezervácia (přírodní rezervace, PR)
        [nature-monument-cz] Prírodná pamiatka (přírodní památka, PP)
        [protective-zone-cz] Ochranné pásmo (ochranné pásmo)
        [nature-zone-cz] Zóna ochrany prírody (zóna ochrany přírody)
        [natura2000-habitats-cz] Lokalita Natura 2000 pre biotopy (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Lokalita Natura 2000 pre vtáky (ptačí oblast, PO)
        [memorial-tree] Pamätný strom (památný strom)
       *[special-rules] Oblasť so zvláštnymi pravidlami
    }
permits-object-count = { $count } objektov
permits-object-citations = Právne základy:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Za ulice, námestia a zelené plochy v Bezirk { $district } je zvyčajne príslušný Bezirksamt { $district }, tam Fachamt Management des öffentlichen Raumes (odbor správy verejného priestoru).
       *[berlin] Za ulice, námestia a zelené plochy v Bezirk { $district } sú zvyčajne príslušné Ordnungsamt (úrad verejného poriadku) a Straßen- und Grünflächenamt (úrad ciest a zelene) Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Za pamiatky v oblasti mapy je príslušný Denkmalschutzamt Hamburg (pamiatkový úrad).
       *[berlin] Za pamiatky v Bezirk { $district } je príslušný nižší pamiatkový orgán (untere Denkmalschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Či v oblasti mapy ležia zapísané pamiatky, tu dáta s istotou povedať nevedia.
       *[yes] V oblasti mapy sú zapísané pamiatky.
    } Či sa ich beh dotkne, napríklad kontrolou pri pamiatke alebo úsekom cez chránené miesto, rozhoduje nižší pamiatkový orgán (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Tu je to samotné mesto { $kreis }.
        [no] , zvyčajne { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; v niektorých spolkových krajinách je to obec (Gemeinde) { $gemeinde }.
        }
       *[unknown] {" "}(zvyčajne Landkreis alebo kreisfreie Stadt, v niektorých spolkových krajinách obec).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Prírodné pamiatky, chránené krajinné prvky a chránené biotopy na mape spadajú pod nižší orgán ochrany prírody (untere Naturschutzbehörde). Tu je to samotné mesto { $kreis }.
        [no] Prírodné pamiatky, chránené krajinné prvky a chránené biotopy na mape spadajú pod nižší orgán ochrany prírody (untere Naturschutzbehörde), zvyčajne { $kreis }.
       *[unknown] Prírodné pamiatky, chránené krajinné prvky a chránené biotopy na mape spadajú pod nižší orgán ochrany prírody (untere Naturschutzbehörde), zvyčajne Landkreis alebo kreisfreie Stadt.
    }
permits-hint-biotope = V oblasti mapy ležia zákonom chránené biotopy{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Zakázané je všetko, čo ich môže zničiť alebo značne poškodiť (§ 30 BNatSchG). Kontroly a úseky umiestni mimo nich alebo sa opýtaj nižšieho orgánu ochrany prírody (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Za prírodné pamiatky a chránené krajinné prvky v Bezirk { $district } je zvyčajne príslušný Bezirksamt { $district }.
       *[berlin] Za prírodné pamiatky a chránené krajinné prvky v Bezirk { $district } je príslušný nižší orgán ochrany prírody (untere Naturschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Podľa otvorených dát spolkovej krajiny patrí { $share } oblasti mapy iným verejným vlastníkom.
       *[no] Podľa otvorených dát spolkovej krajiny patrí { $share } oblasti mapy vlastníkovi "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Vlastníkov zvyšku oblasti sa z otvorených dát zistiť nedá. S oprávneným záujmom (napríklad ako organizátor) môžeš požiadať katastrálny úrad (Grundbuchamt) o výpis; pri verejných pozemkoch pomôže príslušný Ordnungsamt alebo odbor parkov.
       *[no] Vlastníkov týchto plôch sa z otvorených dát zistiť nedá. S oprávneným záujmom (napríklad ako organizátor) môžeš požiadať katastrálny úrad (Grundbuchamt) o výpis; pri verejných pozemkoch pomôže príslušný Ordnungsamt alebo odbor parkov.
    }
permits-section-forest-units = Lesná správa ({ $source })
permits-section-state-forest = Štátny les ({ $source })
permits-none-forest-units = Podľa { $source } oblasť mapy neleží v pôsobnosti žiadneho lesného úradu.
permits-section-admin-units = Správne jednotky ({ $source })
permits-none-admin-units = Podľa { $source } oblasť mapy neleží v žiadnej správnej jednotke registra.
permits-section-register-monuments = Pamiatky národného registra ({ $source })
permits-none-register-monuments = Podľa { $source } v obdĺžniku okolo oblasti mapy neleží žiadna zapísaná pamiatka.
permits-register-monuments-total = Zapísané pamiatky v obdĺžniku okolo oblasti mapy: { $count }. Register neuvádza polohu, tieto upozornenia preto nevedia povedať, kde ležia; treba to overiť na mape.
permits-register-monuments-left = Ďalších { $count } je tu len započítaných, bez názvov.
permits-section-register-archaeology = Archeologické pamiatky národného registra ({ $source })
permits-none-register-archaeology = Podľa { $source } v obdĺžniku okolo oblasti mapy neleží žiadna zapísaná archeologická pamiatka.
permits-register-archaeology-total = Zapísané archeologické pamiatky v obdĺžniku okolo oblasti mapy: { $count }. Register neuvádza polohu, tieto upozornenia preto nevedia povedať, kde ležia; treba to overiť na mape.
permits-register-archaeology-left = Ďalších { $count } je tu len započítaných, bez názvov.
permits-section-register-history = Pamätníky dejín ({ $source })
permits-none-register-history = Podľa { $source } v obdĺžniku okolo oblasti mapy neleží žiadny pamätník dejín.
permits-register-history-in-area = Pamätníky dejín v oblasti mapy: { $count }.
permits-register-history-nearby = Ďalšie pamätníky dejín v obdĺžniku okolo oblasti mapy, ale nie v oblasti mapy: { $count }.
permits-register-history-left = Ďalších { $count } je tu len započítaných, bez názvov.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Podľa { $source } v oblasti mapy neleží žiadny štátny les.
permits-forest-units-reach = Lesný úrad je príslušný za les vo svojej oblasti. Jeho oblasť môže zahŕňať aj mestá a polia.
permits-state-forest-office = príslušný: { $office }
permits-hint-forest-office = { $units ->
        [none] Lesný úrad príslušný za les v oblasti mapy: { $office } ({ $administration }). Ak plánuješ podujatie v lese, obráť sa naň včas.
       *[other] Lesný úrad príslušný za les v oblasti mapy: { $office } ({ $administration }), { $units }. Ak plánuješ podujatie v lese, obráť sa naň včas.
    }
permits-hint-forest-owner = { $office ->
        [known] Podujatie v lese vyžaduje súhlas vlastníkov lesa. Kto les vlastní, zvyčajne povie lesný úrad. V niektorých spolkových krajinách sa musí organizované podujatie v lese navyše ohlásiť lesnému orgánu alebo ho tento orgán musí povoliť.
       *[unknown] Podujatie v lese vyžaduje súhlas vlastníkov lesa. V niektorých spolkových krajinách sa musí organizované podujatie v lese navyše ohlásiť lesnému orgánu alebo ho tento orgán musí povoliť. Tieto poznámky tu príslušný lesný úrad neuvádzajú; poradí lesná správa spolkovej krajiny alebo nižší lesný orgán.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Podujatie v lese zvyčajne potrebuje súhlas vlastníka lesa. Hore uvedená lesná správa (Nadleśnictwo) vie zvyčajne povedať, kto sa o les stará. Les, ktorý nepatrí Štátnym lesom, napríklad súkromný les, nie je v údajoch týchto poznámok. Les, ktorý nepatrí štátu, napríklad súkromný alebo obecný les, podlieha v lesnom hospodárstve dozoru starostu (starosta) (zákon o lesoch, čl. 5 ods. 1 bod 2); opýtajte sa tam na úrade (starostwo powiatowe).
       *[unknown] Podujatie v lese zvyčajne potrebuje súhlas vlastníka lesa. Tieto poznámky pre územie mapy neuvádzajú žiadnu lesnú správu Štátnych lesov; údaje zahŕňajú len Štátne lesy, takže les môže patriť niekomu inému, napríklad súkromnému vlastníkovi. Kto je vlastníkom, treba zistiť osobitne. Les, ktorý nepatrí štátu, napríklad súkromný alebo obecný les, podlieha v lesnom hospodárstve dozoru starostu (starosta) (zákon o lesoch, čl. 5 ods. 1 bod 2); opýtajte sa tam na úrade (starostwo powiatowe).
    }
permits-hint-monument-register-pl = V obdĺžniku okolo oblasti mapy ležia pamiatky národného registra. Ich poloha v dátach nie je, overte na mape, kde sú. Podľa čl. 36 ods. 1 zákona o ochrane pamiatok (ustawa o ochronie zabytków) vyžadujú konzervátorské, reštaurátorské a stavebné práce na zapísanej pamiatke povolenie vojvodského pamiatkara (Wojewódzki Konserwator Zabytków). Od 3. júna 2026 vyžadujú stavebné práce v okolí pamiatky, zmena jej využitia, zariadenia alebo tabule umiestnené na nej a iné činnosti, ktoré by mohli zmeniť jej podstatu alebo vzhľad, len oznámenie pamiatkarovi (čl. 36aa); ten môže do 60 dní namietať a potom vyžadovať povolenie. Zákon neuvádza beh ani kontrolu ako takú; opýtajte sa úradu pamiatkara, ak sa podujatie dotýka pamiatky alebo jej okolia.
permits-hint-archaeology-register-pl = Archeologické pamiatky národného registra ležia v obdĺžniku okolo oblasti mapy. Ich poloha v dátach nie je, overte na mape, kde sú. Podľa čl. 36 ods. 1 zákona o ochrane pamiatok (ustawa o ochronie zabytków) vyžadujú archeologický výskum (pkt 5), hľadanie ukrytých alebo opustených hnuteľných pamiatok elektronickými alebo technickými zariadeniami, napríklad detektormi kovov (pkt 12), a konzervátorské, reštaurátorské alebo stavebné práce na zapísanej pamiatke (pkt 1) povolenie vojvodského konzervátora pamiatok (Wojewódzki Konserwator Zabytków). Povolenie na hľadanie zaniká 1. januára 2027: čl. 36b vtedy dovoľuje plnoletej osobe hľadať so súhlasom vlastníka a držiteľa nehnuteľnosti po oznámení do registra hľadaní cez aplikáciu ministerstva kultúry a čl. 36c také hľadanie okrem iného zakazuje na zapísaných pamiatkach a bližšie ako 10 m od nich. Zákon neuvádza prechádzku, beh ani kontrolu ako takú; opýtajte sa úradu konzervátora, ak podujatie alebo terénna práca zasahuje zapísanú lokalitu.
permits-hint-history-register-pl = Pamätník dejín (Pomnik historii) leží v oblasti mapy alebo v jej blízkosti; časť o pamätníkoch dejín hovorí, ktorý. Prezident Poľskej republiky môže nariadením vyhlásiť za Pomnik historii nehnuteľnú pamiatku zapísanú v registri alebo kultúrny park osobitnej hodnoty (čl. 15 ods. 1 zákona o ochrane pamiatok, ustawa o ochronie zabytków). Už teda stojí pod ochranou registra alebo kultúrneho parku. Na práce na pamiatke zapísanej v registri musí vojvodský pamiatkar (Wojewódzki Konserwator Zabytków) vydať povolenie (čl. 36 ods. 1); v kultúrnom parku zákazy a obmedzenia ustanovuje uznesenie rady gminy (čl. 16 ods. 2, čl. 17 ods. 1). Zákon nepozná prechádzku, beh ani kontrolu ako také; opýtajte sa úradu pamiatkara, ak sa podujatia alebo prác v teréne pamiatka týka.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] V oblasti mapy leží prírodná pamiatka (pomnik przyrody), ekologická plocha (użytek ekologiczny), prírodný a krajinný celok (zespół przyrodniczo-krajobrazowy) alebo dokumentačná lokalita (stanowisko dokumentacyjne). Zriaďuje ich zastupiteľstvo gminy uznesením, ktoré určuje, kto nad objektom dohliada a aké zákazy platia (čl. 44 zákona o ochrane prírody). Zákon sám pre podujatie tu žiadne povolenie nestanovuje. Požiadajte úrad gminy, na ktorej území objekt leží, o uznesenie a o určenie dohliadajúcej osoby.
       *[other] V oblasti mapy leží prírodná pamiatka (pomnik przyrody), ekologická plocha (użytek ekologiczny), prírodný a krajinný celok (zespół przyrodniczo-krajobrazowy) alebo dokumentačná lokalita (stanowisko dokumentacyjne). Zriaďuje ich zastupiteľstvo gminy uznesením, ktoré určuje, kto nad objektom dohliada a aké zákazy platia (čl. 44 zákona o ochrane prírody). Zákon sám pre podujatie tu žiadne povolenie nestanovuje. Požiadajte { $count ->
        [one] úrad gminy ({ $gmina }), na ktorej území objekt leží
       *[other] úrady gmín ({ $gmina }), každý pre objekty na jeho území
    }, o uznesenie a o určenie dohliadajúcej osoby.
    }
permits-hint-gmina-roads-pl = Podujatie, ktoré bráni premávke alebo využíva ulicu zvláštnym spôsobom, potrebuje povolenie podľa čl. 65 zákona o cestnej premávke; žiadosť sa podáva najmenej 30 dní vopred. Povolenie vydáva orgán riadiaci premávku na ceste: starosta pre cesty powiatu a gminy, primátor v meste s právami powiatu, maršal vojvodstva alebo GDDKiA pre ostatné cesty (čl. 10 a 65b). Správcom cesty gminy je wójt, burmistrz alebo { $count ->
        [one] prezydent miasta gminy ({ $gmina })
       *[other] prezydent miasta každej z gmín ({ $gmina })
    }; správca rozhoduje o zábere cestného pásma (čl. 19 a 40 zákona o verejných cestách). Tieto poznámky neukazujú kategóriu ciest.
permits-hint-gmina-land-pl = Ak pozemok v oblasti mapy patrí gmine, napríklad mestský park, námestie, školský areál alebo les, spravuje ho wójt, burmistrz alebo prezydent miasta, ktorý gminu zastupuje (čl. 30 a 31 zákona o samospráve gminy). Pred použitím požiadajte o súhlas { $count ->
        [one] úrad gminy ({ $gmina }).
       *[other] úrady gmín ({ $gmina }).
    } Tieto poznámky neukazujú, komu pozemok patrí.
permits-hint-protected-areas-cz = V oblasti mapy ležia chránené územia alebo objekty ochrany prírody. Podľa zákona o ochrane prírody a krajiny (zákon č. 114/1992 Sb.) sú v národnom parku zakázané organizované športové, turistické a iné verejné podujatia mimo miest, ktoré vyhradil orgán ochrany prírody (§ 16 odst. 1 písm. q), a v ochrannom pásme národného parku si hromadné podujatia tohto druhu mimo týchto miest a mimo zastavaných území vyžadujú súhlas orgánu (§ 37 odst. 3 písm. d). V národnej prírodnej rezervácii je zakázané vstupovať a vychádzať mimo ciest vyznačených so súhlasom orgánu (§ 29 písm. d). Orgánmi ochrany prírody sú okrem iných správa národného parku, Agentúra ochrany prírody (AOPK ČR), krajský úrad (krajský úřad) a úrad obce s rozšírenou pôsobnosťou (§ 75 odst. 1). Iné chránené územia a objekty majú vlastné ochranné podmienky. Ak tam plánujete podujatie, požiadajte orgán včas.
permits-hint-monuments-cz = V oblasti mapy ležia kultúrne pamiatky alebo pamiatkové územia štátneho zoznamu. Podľa zákona o štátnej pamiatkovej starostlivosti (zákon č. 20/1987 Sb., § 14 odst. 1 a 2) musí vlastník pamiatky vopred požiadať o rozhodnutie alebo záväzné stanovisko pred údržbou, opravou, rekonštrukciou alebo inou úpravou pamiatky alebo jej okolia. To isté platí pre vlastníka nehnuteľnosti v pamiatkovej rezervácii, pamiatkovej zóne alebo v ochrannom pásme pamiatky pri stavbe, zmene stavby, terénnych prácach, zariadení alebo tabuli, odstránení stavby, ošetrení stromov alebo udržiavacích prácach. Rozhoduje úrad obce s rozšírenou pôsobnosťou{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, pri národnej kultúrnej pamiatke krajský úrad. Zákon neuvádza prechádzku, beh ani kontrolu ako takú; opýtajte sa tohto úradu, ak sa podujatie dotýka pamiatky alebo na nej umiestňuje zariadenia alebo tabule.
permits-hint-forest-owner-cz = Hromadné športové, kultúrne alebo iné spoločenské podujatie na pozemku určenom na plnenie funkcií lesa sa smie konať len s písomným súhlasom vlastníka lesa, v ktorom sa koná (zákon o lesoch, zákon č. 289/1995 Sb., § 20 odst. 5). Organizátor musí podujatie tiež písomne oznámiť úradu obce s rozšírenou pôsobnosťou{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, alebo krajskému úradu, ak podujatie presahuje obvod takej obce, najneskôr 30 dní pred podujatím. Oznámenie uvádza dátum a miesto, ako je podujatie zabezpečené, očakávaný počet účastníkov a písomné súhlasy vlastníkov (ten istý paragraf). Tieto poznámky neukazujú, komu les patrí; opýtajte sa vlastníkov lesa včas.
permits-hint-roads-cz = Podujatie, ktoré využíva cestu inak ako obvykle, napríklad športové, kultúrne alebo podobné podujatie, ktoré by mohlo ohroziť bezpečnosť alebo plynulosť cestnej premávky, potrebuje povolenie príslušného cestného správneho úradu. Vydáva sa s predchádzajúcim súhlasom vlastníka cesty a, ak môže byť dotknutá bezpečnosť alebo plynulosť premávky, Polície Českej republiky (zákon o cestách, zákon č. 13/1997 Sb., § 25 odst. 1 a odst. 6 písm. d). Úradom je obecný úrad pre miestne cesty alebo úrad obce s rozšírenou pôsobnosťou tam, kde rozhoduje, a úrad obce s rozšírenou pôsobnosťou pre cesty druhej a tretej triedy (§ 40 odst. 4 a 5). Obce v oblasti mapy: { $gmina }. Tieto poznámky neukazujú triedu ciest.
permits-section-water-protection = Ochranné pásma vôd ({ $source })
permits-section-forest-functions = Funkcie lesa a ochranný les ({ $source })
permits-none-water-protection = Podľa { $source } v oblasti mapy neleží žiadne ochranné pásmo vôd.
permits-none-forest-functions = Podľa { $source } v oblasti mapy neleží žiadny rekreačný les, ochranný les, prírodný les ani uzavretá lesná plocha.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreačný les (Erholungswald)
        [protected] Ochranný les (Bannwald alebo Schutzwald)
        [natural] Prírodný les alebo prísna rezervácia (Naturwald, Totalreservat)
       *[closed] Uzavretá lesná plocha
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } ochranného pásma vôd "{ $name }" leží v oblasti mapy. V zónach I a II sú podujatia zvyčajne vylúčené. Opýtaj sa nižšieho vodohospodárskeho orgánu (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } ochranného pásma vôd "{ $name }" leží v oblasti mapy. Podujatia tam sú zvyčajne možné; nižší vodohospodársky orgán (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) povie, čo nariadenie povoľuje.
       *[unknown] Ochranné pásmo vôd "{ $name }" leží v oblasti mapy. Dáta neuvádzajú, o ktorú zónu ide. V zónach I a II sú podujatia zvyčajne vylúčené; opýtaj sa nižšieho vodohospodárskeho orgánu (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] V oblasti mapy leží rekreačný les (Erholungswald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pri podujatiach tam sa opýtaj lesného orgánu.
        [protected] V oblasti mapy leží ochranný les (Bannwald alebo Schutzwald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pred plánovaním podujatia tam sa opýtaj lesného orgánu.
        [natural] V oblasti mapy leží prírodný les alebo prísna rezervácia{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Opustenie ciest je tam často zakázané; bez opýtania sa lesného orgánu tam neumiestňuj kontroly.
       *[closed] V oblasti mapy leží lesná plocha, do ktorej sa nesmie vstupovať{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Nechaj ju mimo trate.
    }
permits-section-roads = Klasifikované cesty podľa OpenStreetMap
permits-none-roads = Podľa OpenStreetMap oblasťou mapy nevedie žiadna Autobahn ani Bundes-, Landes-, Staats- alebo Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (diaľnica)
        [bundesstrasse] Bundesstraße (spolková cesta)
        [landesstrasse] Landesstraße alebo Staatsstraße (cesta spolkovej krajiny)
       *[kreisstrasse] Kreisstraße (okresná cesta)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Oblasťou mapy vedie Autobahn { $refs }. Trať ju smie križovať len po mostoch alebo podchodmi.
        [bundesstrasse] Oblasťou mapy vedie Bundesstraße { $refs }. Uzávierku alebo križovanie s usmerňovateľmi nariaďuje cestný dopravný orgán (Straßenverkehrsbehörde); cestu zvyčajne spravuje Bund prostredníctvom cestnej správy spolkovej krajiny, vo väčších mestách často mesto.
        [landesstrasse] Oblasťou mapy vedie Landes- alebo Staatsstraße { $refs }. Uzávierku alebo križovanie s usmerňovateľmi nariaďuje cestný dopravný orgán (Straßenverkehrsbehörde); cestu zvyčajne spravuje spolková krajina, vo väčších mestách často mesto.
       *[kreisstrasse] Oblasťou mapy vedie Kreisstraße { $refs }. Uzávierku alebo križovanie s usmerňovateľmi nariaďuje cestný dopravný orgán (Straßenverkehrsbehörde); cestu zvyčajne spravuje Kreis, vo väčších mestách často mesto.
    }
permits-hint-state-forest = { $office ->
        [none] Podľa otvorených dát spolkovej krajiny je { $share } oblasti mapy štátny les vlastníka "{ $owner }".
       *[other] Podľa otvorených dát spolkovej krajiny je { $share } oblasti mapy štátny les vlastníka "{ $owner }". Príslušný: { $office }.
    }
