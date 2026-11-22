# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } není jazyk, pro který má freemapper texty; k dispozici: { $available }

## init

cli-part-needs-osm = --part vybírá část oblasti OSM a vyžaduje --osm; u --place nejprve vyhledejte ID a pak použijte --osm <id> --part <n>
cli-reading-file = čtu { $path }
cli-no-area-for-place = pro „{ $query }“ nebyla nalezena žádná oblast. Je-li místo oblastí o několika samostatných částech, vyberte jednu pomocí --osm <id> --part <n> nebo použijte --geojson
cli-using-place = Používám { $name } ({ $id })
cli-created-project = Vytvořen projekt { $path }

## Cache

cli-no-cache-dir = tento systém nemá složku mezipaměti; zadejte --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Stažena 1 dlaždice
        [few] Staženy { $count } dlaždice
        [many] Staženo { $count } dlaždic
       *[other] Staženo { $count } dlaždic
    }
cli-wrote = Zapsáno { $path }
cli-kept-map = Ponechán { $path } (může obsahovat vaši práci)
cli-map-lacks-templates = Mapa zatím neodkazuje na obrazové podklady. Přidejte je v OOM (Templates > Open template…) nebo mapu nahraďte pomocí --force, které starou mapu ponechá jako .omap.bak.
cli-map-lacks-buildings = Mapa zatím nemá budovy ALKIS. Chcete-li je přidat, nahraďte ji pomocí --force, které starou mapu ponechá jako .omap.bak.
cli-map-lacks-canopies = Mapě chybí přístřešky a přístřešky pro auta ALKIS (522), které jsou nyní objekty mapy. Chcete-li je přidat, spusťte běh s --force, které starou mapu ponechá jako .omap.bak.
cli-map-light = Základní mapa: spojeno linií: { $joined }, skryto nebo příliš malé, ponecháno pro vrstvy kandidátů: { $left } (nic se neposunulo)
cli-map-light-kept = Vrstvy kandidátů nově obsahují také objekty nebo úseky ({ $left }), které tato verze ze základní mapy odebírá (označeno freemapper:generalized). Ponechaná mapa je stále má, takže se zobrazí dvakrát. Mapu vyčistíte tak, že ji nahradíte pomocí --force (stará zůstane jako .omap.bak).
cli-map-old-candidates = Mapa stále uvádí templates/candidates.omap; tento běh ho naplnil všemi kandidáty. Kandidáti nově vycházejí také v sedmi vrstvách, každá v jednom souboru (templates/candidates-*.omap). Mapa je uvede, jakmile ji nahradíte pomocí --force (stará zůstane jako .omap.bak), nebo je přidejte sami v OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF bez uvedení zdrojů: ODbL a dl-de/by-2-0 vyžadují, abyste při předání mapy zdroje uvedli; viz ATTRIBUTION.txt
cli-wrote-pdf = Zapsáno { $path } (mapa bez kandidátů)
cli-wrote-pdf-candidates = Zapsáno { $path } (kontrolní tisk: mapa se všemi kandidáty, polohově věrně; ověřte je v terénu)
cli-wrote-training = Zapsáno { $map } a { $pdf } (tréninková mapa: spojeno { $joined }, posunuto { $shifted }, vynecháno nebo oříznuto { $left })
cli-training-free-ends = Volné konce linií: { $ways } tam, kde cesta narazila na jinou cestu, { $others } u jiných linií
cli-training-apart-left = tréninková mapa má stále { $length } mm (na papíře), kde mají objekty příliš malý odstup; zkontrolujte to v OOM
cli-kept-training = Ponechána tréninková mapa { $path }, aby běh nepřepsal mapu, která byla vytištěna nebo předána dál; chcete-li ji obnovit, smažte ji nebo zadejte --force

## OCAD

cli-ocad-kept = Ponechán { $path } (může obsahovat vaši práci; --force ho nahradí a starý ponechá jako .ocd.bak)
cli-ocad-old-candidates = Mapa OCAD stále uvádí { $path } jako mapu na pozadí; tento běh ji naplnil všemi kandidáty. Kandidáti nově vycházejí také v sedmi vrstvách, každá v jednom souboru (-candidates-*.ocd). Mapa OCAD je uvede, jakmile ji nahradíte pomocí --force (stará zůstane jako .ocd.bak), nebo je přidejte sami v OCAD jako mapy na pozadí.
cli-ocad-wrote = { $count ->
        [one] Zapsáno { $path } (1 objekt)
        [few] Zapsáno { $path } ({ $count } objekty)
        [many] Zapsáno { $path } ({ $count } objektů)
       *[other] Zapsáno { $path } ({ $count } objektů)
    }
cli-ocad-left-out = Z mapy OCAD vynecháno: { $skipped } objektů (žádný symbol pro OCAD, žádná plocha nebo mimo kreslicí plochu OCAD ±2 m) a { $dropped } degenerovaných částí linií nebo děr.
cli-ocad-load-symbols = V OCAD zvolte Symbol > Load Colors and Symbols From… a vyberte oficiální sadu symbolů{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , školní sadu IOF 2019
       *[other] {" "}ISSprOM 2019
    }; nahradí zástupné symboly freemapperu podle čísel.

## select

cli-select-running = Výběr území pro { $path }: stiskněte na stránce „Hotovo“ nebo ukončete pomocí Ctrl+C.
cli-select-no-browser = Prohlížeč se nepodařilo otevřít ({ $error }); otevřete adresu uvedenou výše.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; bez oblasti, nemůže být územím mapy): { $place }
cli-find-hint = Projekt vytvoříte příkazem: freemapper init <dir> --osm { $id }
cli-find-no-area = Žádné z těchto míst nemá oblast, která by mohla být územím mapy. Nakreslete oblast příkazem: freemapper select <dir>
cli-find-nothing = pro „{ $query }“ nebylo nalezeno žádné místo; zkuste jiný název nebo nakreslete oblast příkazem: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = upozornění: adaptér ve složce { $folder } je vynechán: { $finding }
cli-adapter-note = upozornění: adaptér ve složce { $folder } se načte, ale: { $finding }
cli-adapter-from-project = upozornění: používá se adaptér { $adapter } ze složky projektu; může se dotazovat těchto hostitelů: { $hosts }
cli-adapter-overlaps = upozornění: adaptér { $adapter } pokrývá také území adaptérů { $others }; jeho objekty se tam dostanou na mapy
cli-adapter-dir-relative = upozornění: FREEMAPPER_ADAPTER_DIR není absolutní cesta ({ $path }); ignoruje se
cli-adapters-no-project = upozornění: { $path } není složka; nebyly načteny žádné adaptéry projektu
cli-adapters-overlaps = sdílí území s: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revize { $revision }  zkontrolováno { $checked }  hostitelé: { $hosts }
cli-adapters-source-built-in = vestavěný
cli-adapters-source-user = z vaší složky
cli-adapters-source-project = z projektu
cli-adapters-never-checked = nikdy
cli-adapters-user-dir = Vaše adaptéry se čtou z { $path }
cli-adapters-no-user-dir = Tento systém nemá složku pro vaše adaptéry; jednu určuje FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper se nyní smí dotazovat také { $host } pro { $name }. Záznam je v { $file }; `freemapper adapters disallow-host { $adapter } { $host }` ho odvolá. Spuštěná stránka výběru území nebo asistent AI načte záznam až po dalším spuštění
cli-allow-host-already = { $host } je pro { $name } už povolen
cli-allow-host-declared = adaptér { $name } uvádí { $host } sám; není co povolovat
cli-allow-host-refused = { $host } nelze povolit: { $reason }
cli-allow-host-unknown-adapter = vestavěný adaptér { $adapter } neexistuje; vestavěné adaptéry jsou: { $adapters }
cli-allow-host-folder-adapter = { $adapter } je adaptér z vaší složky adaptérů; přidejte hostitele raději do `[net] hosts` jeho manifestu
cli-allow-host-no-config = tento systém nemá konfigurační složku, do které by se povolení hostitelé ukládali
cli-allow-host-write = { $file } se nepodařilo zapsat: { $detail }
cli-allow-host-unreadable = { $file } nelze přečíst ({ $detail }); soubor nejdřív opravte nebo odstraňte; nic nebylo změněno
cli-disallow-host-removed = freemapper se už nedotazuje { $host } pro { $name }
cli-disallow-host-absent = { $host } nebyl pro { $name } povolen; není co odvolávat
cli-allowed-hosts-unreadable = upozornění: { $file } nelze přečíst ({ $detail }); nejsou povoleni žádní další hostitelé
cli-allowed-hosts-ignored = upozornění: { $file }: záznam { $host } pro { $adapter } se ignoruje: { $reason }
cli-allowed-hosts-no-adapter = freemapper nemá vestavěný adaptér tohoto jména
cli-allowed-host-redundant = upozornění: { $host } je nyní jedním z hostitelů, které adaptér { $name } uvádí; váš záznam v { $file } byl odstraněn
cli-adapters-allowed = povoleno vámi: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adaptér { $adapter } ({ $name }), revize { $revision }, hostitelé: { $hosts }
cli-check-error = chyba: { $finding }
cli-check-warning = upozornění: { $finding }
cli-check-passed = kontrola { $folder } prošla
cli-check-passed-warnings = kontrola { $folder } prošla, { $count ->
        [one] s jedním upozorněním
       *[other] s { $count } upozorněními
    }
cli-check-failed = kontrola { $folder } selhala
cli-check-no-folder = složka { $folder } neexistuje
cli-check-live-ok = { $capability } v pořádku: { $detail }
cli-check-live-failed = { $capability } SELHALO: { $detail }
cli-check-live-not-asked = { $capability } bez dotazu: { $detail }
cli-check-live-box-outside = výřez neleží uvnitř hranice adaptéru { $adapter }; živá kontrola se služeb dotazuje na místo, které adaptér pokrývá
cli-check-live-cannot = živou kontrolu nelze spustit: { $detail }
cli-check-detail-not-declared = adaptér žádnou nedeklaruje
cli-check-detail-stopped = dřívější část selhala
cli-check-detail-see-warnings = viz upozornění
cli-check-detail-outside-dataset = výřez leží mimo datovou sadu
cli-check-detail-objects = { $count ->
        [one] jeden objekt
        [few] { $count } objekty
        [many] { $count } objektů
       *[other] { $count } objektů
    }
cli-check-detail-layers = vrstev: { $layers }, objektů: { $count }
cli-check-detail-trees = { $count ->
        [one] jeden strom
        [few] { $count } stromy
        [many] { $count } stromů
       *[other] { $count } stromů
    }
cli-check-detail-extract = { $label }, data ze dne { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = model terénu
cli-check-raster-surface = model povrchu
cli-check-raster-tiles = { $product }: { $count ->
        [one] jedna dlaždice
        [few] { $count } dlaždice
        [many] { $count } dlaždic
       *[other] { $count } dlaždic
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } a { $last }

## Folder lock

cli-lock-held = jiný běh freemapperu používá tuto složku (od { $time }, spustil: { $who }). Počkejte na něj nebo ho nejdřív zastavte.
cli-lock-who-cli = terminál
cli-lock-who-mcp = asistent AI (MCP)
cli-lock-who-page = stránka výběru
cli-lock-who-unknown = jiný program
cli-lock-failed = složku projektu se pro tento běh nepodařilo zamknout: { $error }
cli-lock-stuck = v cestě je něco jiného než soubor zámku a nepodařilo se to odstranit

## Run routes of the selection page

cli-run-bad-request = Volby běhu nejsou platné: { $error }
cli-run-no-project = V této složce zatím není uložen žádný projekt. Nejdřív uložte území.
cli-run-already-running = Pro tento projekt už probíhá běh ({ $job }). Počkejte, až skončí, nebo ho nejdřív zrušte.
cli-run-disk-stops = Pro stažení není dost volného místa na disku: { $messages }
cli-run-busy-elsewhere = Jiný projekt tohoto programu právě spouští pipeline. Projekty sdílejí mezipaměť stahování, proto běží vždy jen jeden běh. Zkuste to znovu, až skončí.
cli-run-nothing-to-cancel = Žádný běh neprobíhá, takže není co rušit.
cli-run-unknown-job = Pro tento projekt neexistuje běh { $job }.
cli-status-too-fast = Dotazy na stav chodí příliš často. Ptejte se nejvýše pětkrát za sekundu.
cli-open-folder-failed = Složku se nepodařilo otevřít ({ $error }).
cli-open-folder-no-opener = Tento počítač nemá program pro otevírání složek ({ $program }). Otevřete složku ručně.
cli-open-folder-missing = Složka projektu zatím neexistuje. Nejdřív uložte území.
cli-open-folder-refused = Název složky není bezpečné předat správci souborů.
cli-save-running = Pro tento projekt probíhá běh. Až skončí, uložte znovu.
cli-save-unknown-kind = Neznámý typ mapy „{ $kind }“. Použijte schoolyard, sprint nebo forest.
cli-save-scale-too-large = Měřítko { $scale } je příliš velké.
cli-save-bad-content = Nastavení obsahu není platné. Nic nebylo uloženo.
cli-map-image-none = Nejdřív vytvořte mapu.
cli-map-image-gone = Tento náhled už není k dispozici.
cli-map-image-running = Pro tento projekt probíhá běh. Náhled lze spočítat, až skončí.
cli-map-image-stale = Uložený projekt už neodpovídá vygenerované mapě, například protože se změnilo území, typ mapy nebo měřítko. Nejdřív spusťte běh, který mapu nahradí („Nahradit existující mapu“ na stránce, nebo freemapper run --force).
cli-map-image-superseded = Mezitím byl vyžádán novější náhled, proto tento nebyl vypočten.
cli-map-image-bad-content = Nastavení obsahu není platné.
cli-map-image-bad-request = Požadavek na náhled není platný. Pošlete objekt JSON, případně s nastavením obsahu.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – výběr území
page-heading = Vyberte území mapy
page-update-available = freemapper { $version } je k dispozici. Aktualizace příkazem
page-update-notes = Poznámky k vydání
page-folder = Složka projektu
page-pick = Vyhledat místo
page-query-placeholder = např. Clara-Zetkin-Schule Strausberg
page-search-button = Hledat
page-searching = Hledám …
page-no-results = Nic nenalezeno.
page-point-only = pouze bod, ne plocha
page-draw = Nakreslit samostatně
page-rectangle = Obdélník
page-polygon = Polygon
page-finish-polygon = Uzavřít polygon
page-clear = Smazat
page-hint-rectangle = Klikněte na mapě na dva protilehlé rohy.
page-hint-polygon = Klikejte na rohy jeden po druhém. Uzavřete ho kliknutím na první roh, dvojklikem nebo tlačítkem „Uzavřít polygon“. Esc ruší, Backspace odebere poslední roh.
page-hint-idle = Vyberte místo z vyhledávání nebo nakreslete území sami.
page-summary = Území
page-source-osm = Z OpenStreetMap: { $name }
page-source-drawn = Nakresleno ručně
page-size = Velikost s okrajem: { $width } × { $height } m
page-tiles-one = { $n } dlaždice: { $names }
page-tiles-other = Dlaždice ({ $n }): { $names }
page-download = Stažení: asi { $mb } MB
page-download-none = Stažení: vše je už v mezipaměti
page-download-details = Co se stahuje
page-cached = v mezipaměti
page-disk = Místo na disku: { $volumes }
page-disk-volume = { $free } volných, potřeba asi { $needed }
page-cache-volume = mezipaměť
page-project-volume = složka projektu
page-no-room = nevejde se na disk
page-no-room-skip = Ne všechno se vejde na disk (freemapper ponechává { $reserve } volných). Stahování označené části vynechá. Pokud je potřebujete, nejdřív uvolněte místo:
page-no-room-stop = Není dost místa na disku (freemapper ponechává { $reserve } volných). Bez označených částí mapa nevznikne, proto se stahování zastaví dřív, než cokoli stáhne. Nejdřív uvolněte místo:
page-disk-unknown = freemapper nemůže zjistit volné místo na disku ({ $error }). Stahuje bez kontroly.
page-too-large = Území je příliš velké: nejvýše { $max } m na stranu (s okrajem). Vyberte menší.
page-outside = Území leží mimo podporované spolkové země ({ $states }); freemapper pro jiné oblasti zatím nemá zdroje dat.
page-outside-unknown = Území leží mimo oblasti, které freemapper podporuje; zatím tam nemá zdroje dat.
page-adapters-failed = Letecké snímky nelze vypsat ({ $detail }). Mapa ukazuje jen OpenStreetMap; vše ostatní funguje.
page-settings = Mapa
page-kind = Typ mapy
page-schoolyard = Mapa školního dvora (1:1000)
page-sprint = Sprintová mapa (1:4000)
page-forest = Lesní mapa (1:15000)
page-scale = Měřítko 1:
page-map-title = Název
page-contour-interval = Interval vrstevnic
page-contour-auto = automaticky
page-settings-fixed = Projekt už existuje; typ mapy, měřítko a název jsou ve freemapper.toml. Interval vrstevnic lze stále změnit.
page-save = Uložit
page-saving = Ukládám …
page-saved = Uloženo do freemapper.toml.
page-zone-changed = Uloženo. Území nyní leží v jiné zóně UTM; freemapper stáhne jeho data při dalším běhu znovu.
page-provider-changed = Uloženo. Výšky a letecké snímky nového území poskytuje jiný poskytovatel dat; freemapper stáhne jeho data při dalším běhu znovu.
page-save-stale = freemapper.toml se od načtení této stránky změnil v jiném okně nebo jiným programem. Nic nebylo uloženo; stránka teď zná uložený stav. Uložte znovu, chcete-li ho nahradit územím zobrazeným zde.
page-has-map = Existující mapa (.omap) zůstává, jak je; freemapper vytvoří nové podklady a kandidáty pro nové území. Volba „Nahradit existující mapu“ níže (nebo „freemapper run --force“) vytvoří znovu i mapu a starou ponechá jako .omap.bak.
page-done = Hotovo
page-source = Zdrojový kód (AGPL-3.0-or-later)
page-closed = Hotovo. Toto okno můžete zavřít.
page-next = Dále: freemapper run "{ $dir }"
page-made-files = Vytvořeno: { $files }. Složka: { $dir }
page-back-to-agent = Řekněte svému asistentovi AI, že území je uloženo.
page-nothing-saved = Nic nebylo uloženo.
page-failed = Chyba: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Letecký snímek { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Co patří na mapu
page-content-help = Profil rozhoduje, co se dostane přímo na mapu a co zůstane kandidátem ke kontrole. Volba pro skupinu má přednost před profilem. Ukládá se spolu s územím.
page-content-profile-legend = Profil
page-content-profile-safe = Bezpečný
page-content-profile-safe-hint = Každý objekt zůstane tam, kam ho sestavení umístí: budovy a plochy katastru jdou na mapu; zdi, ploty, stromy a mnoho z OpenStreetMap (rybníky, cesty bez známého povrchu) zůstávají kandidáty.
page-content-profile-balanced = Vyvážený
page-content-profile-balanced-hint = Objekty katastru, rejstříků a mapování ulic jdou na mapu, spolu s vrstevnicemi a zakázanými prostory podél železničních tratí. Zdi, ploty, živé ploty, zemní stupně, schody, voda, stromy, památníky, průchody, cesty a železnice se berou také z OpenStreetMap. Uliční mobiliář, ostatní zakázané prostory a tvary z modelu terénu zůstávají kandidáty.
page-content-profile-full = Úplný
page-content-profile-full-hint = Všechno jde na mapu jako v kontrolním tisku; ven zůstávají jen skrytí kandidáti.
page-content-groups-legend = Podle skupin
page-content-groups-help = „Jako v profilu“ se řídí profilem. Co to pro skupinu znamená, závisí na tom, odkud každý objekt pochází.
page-content-place-profile = Jako v profilu
page-content-place-map = Mapa
page-content-place-candidates = Kandidát
page-content-place-off = Vypnuto
page-content-group-walls = Zdi
page-content-group-fences = Ploty
page-content-group-hedges = Živé ploty
page-content-group-slopes = Zemní stupně a skalní stěny
page-content-group-stairs = Schody
page-content-group-trees = Stromy a skupiny stromů
page-content-group-water-features = Voda, fontány a prameny
page-content-group-monuments = Věže, památníky a elektrická vedení
page-content-group-furniture = Uliční mobiliář a lampy
page-content-group-passages = Průchody
page-content-group-contours = Vrstevnice
page-content-group-landforms = Kupky, prohlubně, zemní stupně a skalní stěny z modelu terénu
page-content-group-vegetation = Vegetace z laserových mračen bodů (Braniborsko, Berlín, Severní Porýní-Vestfálsko, Porýní-Falc, Sársko a Bavorsko; vypnuto, dokud ji nezvolíte; stahuje asi 40 až 600 MB na km²)
page-content-group-out-of-bounds = Zakázané prostory
page-content-group-paths = Cesty a železnice
page-content-group-streets = Ulice a náměstí
page-run-heading = Vytvořit mapu
page-preview-heading = Náhled mapy
page-preview-stored = Jak byla naposledy vygenerována.
page-preview-live = S nastavením na této stránce, zatím neuloženým.
page-preview-live-saved = S uloženým nastavením, zatím nevygenerováno během běhu.
page-preview-computing = Počítám …
page-preview-none = Zatím nebyla vygenerována žádná mapa.
page-preview-later = Tato mapa zatím nemá náhled. Objeví se s dalším během, který mapu zapíše.
page-preview-stale = Území, typ mapy nebo měřítko se od vygenerování mapy změnily. Obrázek stále ukazuje mapu, jak byla naposledy vygenerována; nový náhled vyžaduje běh s volbou „Nahradit existující mapu“.
page-preview-area-unsaved = Území na této stránce zatím není uloženo. Obrázek ukazuje mapu, jak byla naposledy vygenerována; nové území vyžaduje běh s volbou „Nahradit existující mapu“.
page-preview-run = Jak byla naposledy vygenerována. Stránka obrázek obnoví, až běh skončí.
page-preview-open = Otevřít v plné velikosti
page-preview-alt = Náhled objektů mapy
page-opt-ocad = Zapsat startovní projekt OCAD
page-help-ocad = Pro všechny, kdo pokračují v OCAD.
page-opt-pdf = Tisknutelné PDF
page-help-pdf = Jen objekty mapy, bez kandidátů.
page-opt-pdf-candidates = Kontrolní tisk se všemi kandidáty
page-opt-training = Tréninková mapa
page-help-training = Mapa a každý kandidát, upravené tak, aby se daly tisknout tak, jak jsou; s OCAD také jako soubor OCAD.
page-views-heading = Pohledy na tréninkovou mapu
page-view-no-paths = Bez cest (Ultimate map)
page-view-brown-only = Jen hnědá (orientace podle vrstevnic)
page-view-no-north-lines = Bez severních čar
page-view-custom = Jiný pohled
page-help-views = Každý zaškrtnutý pohled je jeden další soubor vedle tréninkové mapy, `<name>-training-<view>.omap` a jeho PDF, s OCAD také soubor OCAD. Jiný pohled je `no:505,506` (vynechat symboly) nebo `only:101,102` (ponechat jen tyto); `+` pohledy kombinuje. Název, měřítko a zdroje zůstávají vždy.
page-help-views-forest = Bez cest a jen hnědá jsou pouze pro lesní mapy.
page-opt-scale-bar = Grafické měřítko v PDF
page-help-scale-bar = Německé mapy ho obvykle nemají.
page-opt-sources = Řádky zdrojů v PDF
page-sources-auto = automaticky
page-sources-full = úplné
page-sources-credit-only = jen freemapper
page-warn-credit-only = PDF pak uvádějí jen freemapper. Licence dat (ODbL, dl-de/by-2-0) vyžadují při předání mapy uvedení zdrojů; ty pak uvedete sami. ATTRIBUTION.txt je uvádí všechny.
page-need-pdf = Grafické měřítko a řádky zdrojů se uplatní jen při tvorbě PDF.
page-opt-offline = Jen mezipaměť, bez sítě
page-opt-force = Nahradit existující mapy
page-help-force = Mapu a, je-li zaškrtnuta „Tréninková mapa“, i tréninkovou mapu. Staré soubory zůstanou jako .omap.bak.
page-opt-permits = Poznámky k povolením
page-help-permits = Správní celky, chráněná území a koho se ptát, v permits.md. Orientační poznámka, ne právní poradenství.
page-run-start = Vytvořit mapu
page-run-again = Vytvořit znovu
page-run-cancel = Zrušit
page-run-cancelling = Ruším …
page-warnings-heading = Upozornění
page-where-files = Soubory jsou ve složce projektu:
page-open-folder = Otevřít složku
page-log-label = Záznam běhu
page-force-title = Nahradit existující mapy?
page-force-text = Existující mapa se nahradí, a je-li zaškrtnuta „Tréninková mapa“, i tréninková mapa. Vaše úpravy nezmizí: staré soubory zůstanou ve složce projektu jako .omap.bak. Pokračovat?
page-force-cancel = Zrušit
page-force-confirm = Nahradit

page-step-fetch = Stahuji data
page-step-derive = Počítám podklady
page-step-map = Zapisuji mapu
page-step-ocad = Zapisuji projekt OCAD
page-step-pdf = Zapisuji PDF
page-step-pdf-candidates = Zapisuji kontrolní tisk
page-step-training = Zapisuji tréninkovou mapu
page-step-permits = Zapisuji poznámky k povolením
page-step-line = Krok { $index } z { $total }: { $label }
page-starting = Běh se spouští …
page-file-kind-map = mapa
page-file-kind-template = podklad
page-file-kind-ocad = mapa OCAD
page-file-kind-ocad-candidates = kandidáti OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = kontrolní tisk
page-file-kind-training-map = tréninková mapa
page-file-kind-training-pdf = PDF tréninkové mapy
page-file-kind-training-ocad = tréninková mapa OCAD
page-file-kind-training-view-map = pohled na tréninkovou mapu
page-file-kind-training-view-pdf = PDF pohledu na tréninkovou mapu
page-file-kind-training-view-ocad = pohled na tréninkovou mapu OCAD
page-file-kind-preview = náhled mapy
page-file-kind-permits = poznámky k povolením
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Hotovo. Soubory jsou zapsány.
page-result-failed = Běh selhal: { $error }
page-result-failed-kept = Co je zapsáno, zůstane ve složce projektu; nový běh naváže odtud.
page-result-cancelled = Zrušeno. Co už je staženo, zůstane v mezipaměti; nový běh naváže odtud.
page-result-title-done = Mapa vytvořena
page-result-title-failed = Běh selhal
page-result-title-cancelled = Běh zrušen
page-kept-map = Existující mapa zůstala beze změny. „Nahradit existující mapu“ ji vytvoří znovu a starou ponechá jako .omap.bak.
page-no-files = Nebyly vytvořeny žádné soubory.
page-opened-folder = Složka je otevřená.
page-open-failed = { $message } Složka: { $dir }
page-why-no-area = Nejdřív vyberte území.
page-why-checking = Náhled se počítá …
page-why-too-large = Území je příliš velké.
page-why-outside = Území leží mimo podporované spolkové země.
page-why-disk = Není dost volného místa na disku (viz výše).
page-why-running = Probíhá běh.
page-err-scale-bar-no-pdf = Grafické měřítko vyžaduje PDF.
page-err-sources-no-pdf = Řádky zdrojů vyžadují PDF.
page-save-failed = Území není uloženo; žádný běh nebyl spuštěn.
page-poll-lost = Server neodpovídá. freemapper to zkouší dál.
page-run-unknown = Server tento běh už nezná.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Zavřít

## update (spec 3.4 to 3.6)

update-source-build = Tento freemapper byl sestaven ze zdrojového kódu. Aktualizujte ho pomocí git pull a cargo build --release --locked.
update-current = freemapper { $version } je aktuální.
update-available = freemapper { $new } je k dispozici (máte { $old }), vydán { $released }.
update-details =
    Poznámky k vydání: { $notes }
    Stažení: { $size } MB
update-decimal-separator = ,
update-ask = Nainstalovat nyní? [y/N]
update-not-installed = Nic nebylo nainstalováno.
update-needs-yes = vstup není terminál, takže se freemapper před instalací nemůže zeptat; spusťte freemapper update --yes pro instalaci bez dotazu
update-installing = stahuji a kontroluji freemapper { $version }
update-done = freemapper byl aktualizován z { $old } na { $new }. Poznámky k vydání: { $notes }
update-mcp-restart = Běžící server MCP si ponechá starou verzi, dokud ho asistent AI nespustí znovu.
update-rolled-back = Návrat na freemapper { $version }. Verze, kterou jste opustili, je uložena jako { $path }.
update-not-writable = nelze zapisovat do složky programu { $path } a freemapper nežádá o práva správce. Rozbalte tam novou verzi ručně, nebo přesuňte freemapper do složky, do které smíte zapisovat.
update-too-old = freemapper { $new } nelze nainstalovat samoaktualizací verze { $old }; nainstalujte { $needed } nebo novější ručně, viz { $notes }
update-busy = ve složce programu probíhá jiná aktualizace nebo návrat na předchozí verzi (soubor zámku { $path }); počkejte, až skončí, a zkuste to znovu
update-bad-signature = verzi se nepodařilo ověřit ({ $error }); nic nebylo nainstalováno
update-check-bad-signature = nejnovější verzi freemapperu se nepodařilo ověřit ({ $error }); nic nebylo nainstalováno
update-no-manifest = nejnovější verze se nemůže nainstalovat sama (nemá manifest aktualizace); stáhněte ji z { $page }
update-no-manifest-version = verze { $version } neexistuje nebo se nemůže nainstalovat sama (nemá manifest aktualizace); viz { $page }
update-no-asset = verze nemá program pro tento systém ({ $platform }); nic nebylo nainstalováno
update-no-previous = není předchozí verze, na kterou by se dalo vrátit ({ $path } chybí)
update-failed = aktualizace selhala: { $error }
update-mode-set = Režim aktualizace nastaven na { $mode } v { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } toto nastavení přepisuje, dokud je nastaveno.
update-mode-failed = nepodařilo se zapsat { $path }: { $error }
update-notice = freemapper { $new } je k dispozici (máte { $old }). Aktualizace příkazem: freemapper update
update-applies-next-start = Nová verze se uplatní od příštího spuštění freemapperu.
update-install-failed = freemapper { $new } nebyl nainstalován: { $error }
update-no-config-dir = tento systém nemá konfigurační složku; nastavte FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Poznámka pro uživatele: při spuštění našel server freemapper o složkách adaptérů toto. Předejte to dál; uživatel složku opraví nebo odstraní a pak server spustí znovu.
    { $lines }
update-mcp-note = Poznámka pro uživatele: freemapper { $new } je k dispozici (toto je { $old }). Před voláním install_update se ho zeptejte.
update-mcp-needs-confirm = nic nebylo nainstalováno; nejdřív se zeptejte uživatele a install_update zavolejte s confirm: true, jakmile souhlasí
update-mcp-note-manual = Poznámka pro uživatele: freemapper { $new } je k dispozici (toto je { $old }), ale install_update ho nemůže nainstalovat; nainstalujte { $needed } nebo novější ručně, viz { $notes }
update-mcp-note-bad-signature = Poznámka pro uživatele: nejnovější verzi freemapperu se nepodařilo ověřit ({ $error }); nic nebylo nainstalováno. Řekněte mu to a neinstalujte ji jinou cestou.
update-mcp-restart-now = Restartujte nyní server MCP freemapperu; stále běží ve staré verzi.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = chyba
error-cancelled = zrušeno
cli-cancelled-kept = zrušeno; ponecháno: { $files }
error-io = { $path }: { $message }
error-io-disk-full = disk je plný, proto freemapper nemohl zapsat { $path }. Uvolněte místo a zkuste to znovu

## Errors: project

error-project-already-exists = projekt už existuje: { $path }
error-project-map-exists = mapa už existuje (může obsahovat vaši práci); k nahrazení použijte --force: { $path }
error-project-invalid-scale = neplatné měřítko 1:{ $scale }
error-project-invalid-contour-interval = neplatný interval vrstevnic { $interval } m ve freemapper.toml; musí být větší než 0
error-project-scale-not-allowed = měřítko 1:{ $scale } není povoleno pro mapy typu { $kind }; použijte { $allowed }
error-project-scale-not-allowed-enlarge = měřítko 1:{ $scale } není povoleno pro mapy typu { $kind }; použijte { $allowed }. Kreslete v 1:4000, freemapper-print při tisku zvětší na 1:3000
word-kind-schoolyard = školní dvůr
word-kind-sprint = sprint
word-kind-forest = les
error-project-toml = neplatný soubor projektu: { $message }
error-project-unknown-adapter = soubor projektu uvádí adaptér „{ $adapter }“, který tento freemapper nemá; může ho přinést novější freemapper (`freemapper update`), nebo pochází z vaší složky adaptérů (ukáže ho `freemapper adapters list`) či, s --adapters-from-project na příkazové řádce, ze složky `adapters` projektu (jen u projektu, kterému důvěřujete)
error-project-not-fetched = zatím nebyly staženy žádné dlaždice; nejdřív spusťte `freemapper fetch`
error-project-ocd = export OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: obrázek a world file neodpovídají území mapy; spusťte `freemapper derive` znovu
error-project-tiles-outdated = území mapy potřebuje dlaždice { $tiles }, které nebyly staženy; spusťte `freemapper fetch`
error-project-buildings-not-fetched = pro toto území mapy nejsou uloženy žádné budovy ALKIS; nejdřív spusťte online `freemapper fetch`
error-project-orthophoto-not-fetched = pro toto území mapy není uloženo žádné ortofoto; nejdřív spusťte online `freemapper fetch`
error-project-terrain-not-fetched = pro toto území mapy není uložen žádný model terénu; nejdřív spusťte online `freemapper fetch`
error-project-candidates-outdated = uložení kandidáti OSM nepokrývají území mapy; spusťte `freemapper fetch`
error-project-stored = { $path }: { $message }; spusťte `freemapper fetch` znovu
error-project-area-too-large = území mapy s okrajem je { $width } × { $height } m, tedy více než { $limit } m na stranu; zmenšte území mapy nebo buffer_m ve freemapper.toml
error-project-area-crosses-country = území mapy leží na obou stranách státní hranice; mapy přes hranici zatím nejsou podporovány, proto nakreslete území uvnitř jednoho státu
error-project-permits-not-for-region = poznámky k povolením pro tuto oblast zatím nejsou k dispozici: dotazují se německých služeb a uvádějí německé úřady
error-project-outside-coverage = území mapy leží mimo podporované spolkové země ({ $states }); freemapper má zatím zdroje dat jen pro ně

## Errors: disk space

error-space-required = není dost místa na disku pro { $part }. Je potřeba { $needed }, ale { $path } má mimo rezervu { $reserve } volných jen { $left }. Uvolněte místo a spusťte `freemapper fetch` znovu
error-space-skipped = freemapper pro nedostatek místa na disku vynechává { $part }. Je potřeba { $needed }, ale { $path } má mimo rezervu { $reserve } volných jen { $left }. Uvolněte místo a spusťte `freemapper fetch` znovu
word-part-map-data = mapová data ALKIS a OSM
word-part-terrain = model terénu
word-part-orthophoto = ortofoto
word-part-surface = model povrchu pro podklad nDSM
word-part-osm-update = aktualizaci výřezu OSM v mezipaměti
word-part-k-five = podklad mapy K5 Berlína
word-part-berlin-layers = struktury ALKIS Berlína, katastr stromů, mapování ulic a bodové vrstvy
word-part-state-layers = plné struktury ALKIS, objekty krajinného modelu a katastry stromů
word-part-point-cloud = laserová mračna bodů pro kandidáty vegetace

## Errors: downloads

error-fetch-http = stažení { $url } selhalo: { $message }
error-fetch-http-temporary = stažení { $url } selhalo: { $message }. Služba je pravděpodobně přetížená nebo v údržbě; freemapper se ptal až { $attempts }krát. Spusťte běh za několik minut znovu; co už je stažené, zůstává zachováno.
error-fetch-corrupt-archive = stažení { $url } není úplný archiv dlaždic: { $message }
error-fetch-corrupt-extract = stažení { $url } není úplný výřez OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] dlaždice { $tiles } není pokryta zveřejněnými dlaždicemi; zmenšete území mapy nebo buffer_m ve freemapper.toml
       *[other] dlaždice { $tiles } nejsou pokryty zveřejněnými dlaždicemi; zmenšete území mapy nebo buffer_m ve freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline a v mezipaměti chybí: dlaždice { $tiles } produktu { $product }; spusťte online `freemapper fetch`
       *[other] offline a v mezipaměti chybí: dlaždice { $tiles } produktu { $product }; spusťte online `freemapper fetch`
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = model terénu (DGM1)
word-product-image-dsm = model povrchu (bDOM)
word-product-saxony-dtm = model terénu (GeoSN DGM1)
word-product-saxony-dsm = model povrchu (GeoSN DOM1)
error-fetch-osm-extract-missing = v mezipaměti není žádný výřez OSM; spusťte online `freemapper fetch`
error-fetch-not-found = { $name } nebylo na serveru nalezeno
error-fetch-refused = { $url }: server požadavek odmítl (HTTP { $status })
error-fetch-refused-detail = { $url }: server požadavek odmítl (HTTP { $status }): { $detail }
error-fetch-too-large = stažení { $url } je větší než { $limit } MB; zastaveno
error-fetch-not-stored = pro toto území mapy nejsou uložena data: { $label }. Soubor { $path } chybí, nelze ho přečíst nebo území nepokrývá; nejdřív spusťte online `freemapper fetch`
error-fetch-range-ignored = { $url }: server ignoroval rozsah bajtů { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper na tuto adresu žádné požadavky neposílá: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper na tuto adresu žádné požadavky neposílá: adaptér { $name } neuvádí hostitele { $host }{ $from }. Pokud se služba přestěhovala tam, povolte hostitele příkazem `freemapper adapters allow-host { $adapter } { $host }` a nahlaste to na https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper na tuto adresu žádné požadavky neposílá: adaptér { $adapter } neuvádí hostitele { $host }{ $from }. Pokud se služba přestěhovala tam, přidejte hostitele do `[net] hosts` manifestu adaptéru
error-fetch-host-redirected-from = {" "}({ $url } sem přesměrovává)
error-reason-not-a-host-name = není název hostitele
error-reason-not-ascii = název mimo ASCII se musí zapsat ve tvaru xn--
error-reason-not-a-dns-name = není název DNS
error-reason-no-domain = název tohoto počítače nebo název bez domény
error-reason-local-name = název z místní sítě
error-reason-ip-address = adresa IP, ne název DNS
error-reason-unexpected-character = neočekávaný znak
error-reason-not-https = ne https
error-reason-user-info = uživatelské údaje
error-reason-port = jiný port než 443
error-reason-host-not-declared = hostitel není mezi deklarovanými hostiteli
error-reason-local-address = adresa tohoto počítače nebo místní sítě, ne serveru na internetu
error-reason-too-many-redirects = více než { $count } přesměrování
error-reason-redirect-nowhere = přesměrování (HTTP { $status }) nikam
error-fetch-currency-csv = CSV s aktuálností, řádek { $line }: { $message }
error-alkis = neplatná data ALKIS: { $message }
error-osm = data OSM: { $message }

## Errors: templates and rasters

error-derive-zip = archiv dlaždic: { $message }
error-derive-image = obrázek: { $message }
error-derive-no-tiles = nejsou dlaždice, ze kterých by se dalo odvodit
error-derive-area-too-large = území mapy je příliš velké pro jeden podklad: { $width } × { $height } pixelů (nejvýše { $limit } na stranu); zmenšete území mapy nebo buffer_m ve freemapper.toml
error-raster-world-file = neplatný world file: { $message }
error-raster-misaligned = rastry nejsou zarovnány na stejnou pixelovou mřížku: { $message }
error-raster-pixel-size = rastry mají různou velikost pixelu: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = neplatný GeoJSON: { $message }
error-area-no-polygon = GeoJSON neobsahuje žádný polygon
error-area-multi-polygon = MultiPolygon o { $parts } částech; rozdělte území mapy na jednotlivé polygony
error-area-out-of-range = souřadnice mimo rozsah: lat { $lat }, lon { $lon }
error-area-not-an-area = není použitelné území mapy: { $message }
error-georef-date = datum { $date } leží mimo platnost World Magnetic Model
error-georef-map-range = bod E { $easting } N { $northing } leží za rozsahem souřadnic mapy
error-omap-xml = neplatný XML OOM: { $message }
error-omap-missing-element = prvek <{ $name }> nenalezen
error-ocd-duplicate-symbol = číslo symbolu { $symbol } je použito dvakrát
error-ocd-unknown-symbol = objekt odkazuje na symbol { $symbol }, který v souboru není
error-ocd-wrong-type = objekt se symbolem { $symbol } má pro svůj symbol nesprávný typ

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Použití:
cli-heading-commands = Příkazy:
cli-heading-options = Volby:
cli-heading-arguments = Argumenty:
cli-help-possible-values = možné hodnoty
cli-help-default = výchozí
cli-arg-help = Zobrazit nápovědu
cli-arg-version = Zobrazit verzi
cli-help-help = Zobrazí nápovědu k příkazu
cli-arg-help-command = Příkaz, k němuž se má zobrazit nápověda, například run
cli-parse-hint = Další informace získáte pomocí '--help'.
cli-double-click-start = freemapper otevře v prohlížeči stránku, na které vyberete oblast mapy. Složka projektu je { $path }. Zavřením tohoto okna program ukončíte.
cli-parse-unknown-argument = neočekávaný argument { $arg }
cli-parse-missing-required = chybí povinný argument: { $args }
cli-parse-invalid-value = neplatná hodnota { $value } pro { $arg }
cli-parse-invalid-value-choices = neplatná hodnota { $value } pro { $arg }; možné hodnoty: { $values }
cli-parse-invalid-subcommand = neznámý příkaz { $value }
cli-parse-missing-subcommand = je vyžadován příkaz
cli-parse-too-many-values = neočekávaná hodnota { $value } pro { $arg }
cli-parse-argument-conflict = { $arg } nelze použít spolu s { $other }

help-freemapper = Sestavuje startovní projekty OOM z geodat
help-freemapper-lang = Jazyk textů programu, `de` nebo `en` (výchozí: FREEMAPPER_LANG, pak jazyk systému, pak angličtina)
help-init = Vytvoří složku projektu pro území mapy
help-init-dir = Složka projektu, která se vytvoří
help-init-geojson = Soubor GeoJSON s polygonem území mapy (EPSG:4326)
help-init-osm = Objekt OSM území mapy, např. way/992423119
help-init-place = Název místa, např. "Clara-Zetkin-Schule Strausberg"; freemapper použije nejlepší shodu s oblastí
help-init-part = Kterou část oblasti OSM o několika samostatných částech použít (1 je největší)
help-init-kind = Typ mapy
help-arg-scale = Jmenovatel měřítka nového projektu: schoolyard 1000 (výchozí); sprint 4000 (výchozí) nebo 5000; forest 15000 (výchozí), 10000 nebo 7500. Jiné hodnoty jsou odmítnuty, protože je odmítá freemapper-print
help-init-title = Název mapy pro okrajové údaje (výchozí: název složky)
help-export = Zapíše startovní projekt (mapa OOM, podklady, uvedení původu dat) do složky projektu
help-export-dir = Složka projektu
help-arg-force = Nahradí existující soubor mapy a starý ponechá jako `<name>.omap.bak`
help-export-ocad = Zapíše také startovní bod OCAD do `ocad/` (formát OCAD 12); existující mapa OOM pak zůstane, pokud nezadáte --force
help-export-pdf = Zapíše také `<name>.pdf`, tisknutelnou mapu (jen objekty mapy, bez kandidátů); existující mapa OOM pak zůstane, pokud nezadáte --force
help-export-pdf-candidates = Zapíše také `<name>-candidates.pdf`, kontrolní tisk: mapu se všemi kandidáty, polohově věrně; existující mapa OOM pak zůstane, pokud nezadáte --force
help-export-training = Zapíše také tréninkovou mapu `<name>-training.omap` a její PDF `<name>-training.pdf`: mapu a každého kandidáta, upravené tak, aby se daly tisknout tak, jak jsou; s --ocad také `ocad/<name>-training.ocd`. Existující mapa OOM pak zůstane, pokud nezadáte --force, a --force nahradí i existující tréninkovou mapu
help-export-view = S --training: zapíše také tréninkovou mapu bez toho, co pohled vynechává, jako `<name>-training-<view>.omap` a její PDF (s --ocad také soubor OCAD). Pohledy: `no-paths` (bez silnic, cest, schodů), `brown-only` (jen hnědé symboly a rámeček), `no-north-lines`, nebo seznam `no:<codes>` / `only:<codes>`; kombinujte pomocí `+`. Volbu zopakujte pro více souborů
help-export-pdf-scale-bar = Nakreslí na PDF grafické měřítko (ve výchozím stavu vypnuto, německé mapy ho obvykle nemají; číslo měřítka zůstane v záhlaví)
help-export-pdf-sources = Řádky zdrojů v PDF: `auto` vytiskne úplný seznam a zkrátí ho, jen když by zabral více než 5 řádků, `full` nikdy nezkracuje, `credit-only` vytiskne jen řádek s uvedením freemapperu (ODbL a dl-de/by-2-0 pak vyžadují, abyste při předání mapy zdroje uvedli). ATTRIBUTION.txt uvádí vždy všechny zdroje
help-fetch = Stáhne data pro území mapy (ortofoto, model terénu, nDSM, budovy ALKIS, OSM)
help-fetch-dir = Složka projektu
help-fetch-offline = Použije jen mezipaměť, bez přístupu k síti
help-arg-cache-dir = Složka mezipaměti pro stažené dlaždice (výchozí: systémová složka mezipaměti + `freemapper`)
help-derive = Vypočítá podklady (ortofoto, stínovaný reliéf, sklon, místní reliéf, nDSM), vrstevnice a tvary terénu ze stažených dat
help-derive-dir = Složka projektu
help-run = Stáhne, odvodí a exportuje najednou
help-run-dir = Složka projektu
help-run-offline = Použije jen mezipaměť, bez přístupu k síti
help-run-ocad = Zapíše také startovní bod OCAD do `ocad/` (formát OCAD 12)
help-run-pdf = Zapíše také `<name>.pdf`, tisknutelnou mapu (jen objekty mapy, bez kandidátů)
help-run-pdf-candidates = Zapíše také `<name>-candidates.pdf`, kontrolní tisk: mapu se všemi kandidáty, polohově věrně
help-run-training = Zapíše také tréninkovou mapu `<name>-training.omap` a její PDF `<name>-training.pdf`: mapu a každého kandidáta, upravené tak, aby se daly tisknout tak, jak jsou; s --ocad také `ocad/<name>-training.ocd`. Existující tréninková mapa zůstane, pokud nezadáte --force
help-run-view = S --training: zapíše také tréninkovou mapu bez toho, co pohled vynechává, jako `<name>-training-<view>.omap` a její PDF (s --ocad také soubor OCAD). Pohledy: `no-paths` (bez silnic, cest, schodů), `brown-only` (jen hnědé symboly a rámeček), `no-north-lines`, nebo seznam `no:<codes>` / `only:<codes>`; kombinujte pomocí `+`. Existující soubor pohledu zůstane, pokud nezadáte --force
help-run-pdf-scale-bar = Nakreslí na PDF grafické měřítko (ve výchozím stavu vypnuto; číslo měřítka zůstane v záhlaví)
help-run-pdf-sources = Řádky zdrojů v PDF: `auto` vytiskne úplný seznam a zkrátí ho, jen když by zabral více než 5 řádků, `full` nikdy nezkracuje, `credit-only` vytiskne jen řádek s uvedením freemapperu (ODbL a dl-de/by-2-0 pak vyžadují, abyste při předání mapy zdroje uvedli). ATTRIBUTION.txt uvádí vždy všechny zdroje
help-select = Vyberte území mapy v prohlížeči: nakreslete ho nebo zvolte místo. Uložení zapíše freemapper.toml a podle potřeby vytvoří složku projektu
help-select-dir = Složka projektu (při uložení se vytvoří, pokud neexistuje)
help-select-kind = Typ mapy nového projektu
help-select-title = Název mapy nového projektu
help-select-no-browser = Jen vypíše adresu stránky; neotevírá prohlížeč
help-select-port = Port na 127.0.0.1 (výchozí: volný)
help-select-offline = Stránka se spustí se zaškrtnutou volbou „Jen mezipaměť, bez sítě“
help-select-ocad = Stránka se spustí se zaškrtnutým startovním projektem OCAD
help-select-pdf = Stránka se spustí se zaškrtnutým tisknutelným PDF
help-select-pdf-candidates = Stránka se spustí se zaškrtnutým kontrolním tiskem (každý kandidát)
help-select-training = Stránka se spustí se zaškrtnutou tréninkovou mapou
help-select-pdf-scale-bar = Stránka se spustí se zaškrtnutým grafickým měřítkem v PDF
help-select-pdf-sources = Stránka se spustí s touto volbou řádků zdrojů v PDF, jako u `run`
help-find = Vypíše místa OpenStreetMap pro název, oblasti první, s ID pro init --osm
help-find-query = Název místa, např. "Clara-Zetkin-Schule Strausberg"
help-find-json = Vypíše shody jako JSON, stejná pole jako nástroj MCP find_place
help-mcp = Zpřístupní nástroje agentovi AI přes MCP (stdin/stdout)
help-update = Nainstaluje novější freemapper z vydání na GitHubu, po dotazu. Build ze zdrojového kódu místo toho řekne, jak ho aktualizovat
help-update-check = Jen zkontroluje: návratový kód 10, existuje-li novější verze, 0, je-li tato aktuální
help-update-yes = Nainstaluje bez dotazu; je potřeba, když vstup není terminál
help-update-version = Nainstaluje tuto verzi (x.y.z) místo nejnovější, i starší
help-update-rollback = Vrátí se k verzi, kterou nahradila poslední aktualizace
help-update-mode = Uloží, co dělá denní kontrola, když najde novější verzi (výchozí: notify). Funguje v každém buildu, protože jen zapisuje nastavení
help-adapters = Zobrazí regionální adaptéry: spolkové země a státy, pro které má freemapper zdroje dat
help-adapters-list = Vypíše adaptéry s tím, odkud každý pochází (vestavěný, vaše složka, projekt), jeho revizí, dnem poslední kontroly a hostiteli, na které se dotazuje
help-adapters-list-json = Vypíše seznam jako JSON
help-adapters-list-project = Přečte také adaptéry ve složce `adapters` této složky projektu, jako by to udělalo --adapters-from-project
help-adapters-check = Zkontroluje složku adaptéru dřív, než se použije nebo předá dál: zda by se načetl, co by měl autor vědět, a s --live zda jeho služby odpovídají. Návratový kód 0: prošel; 1: má nálezy; 2: kontrolu nešlo spustit
help-adapters-check-dir = Složka adaptéru; její název je ID adaptéru
help-adapters-check-live = Zeptá se také služeb adaptéru na výřez daný --bbox a řekne, co každá dodá. Požadavky zůstávají u hostitelů, které manifest deklaruje
help-adapters-check-bbox = Výřez živé kontroly ve stupních, jako minLon,minLat,maxLon,maxLat. Leží uvnitř hranice adaptéru; stačí 150 m na 150 m
help-adapters-check-json = Vypíše výsledek jako JSON
help-adapters-probe = Každou službu, kterou vestavěné adaptéry uvádějí, jednou dotázat a říct, zda odpovídá: ok, slow, moved nebo error. Návratový kód je 0 i při selhání služby; výsledek je odpověď
help-adapters-probe-json = Vypíše výsledek jako JSON
help-adapters-probe-status = Přečíst stav z minulé noci z tohoto souboru (smí chybět) a vypsat nový stav jako JSON: služba je vyřazená po dvou špatných nocích za sebou. Změny jdou na standardní chybový výstup
help-adapters-allow-host = Povolí vestavěnému adaptéru jednoho dalšího hostitele: pro spolkovou zemi, jejíž služba se přestěhovala na server, který její adaptér ještě neuvádí. Zpráva o odmítnuté adrese říká, o kterého hostitele jde. Hostitel se kontroluje jako hostitel manifestu a freemapper se stále připojuje jen k serverům na internetu
help-adapters-allow-host-adapter = ID vestavěného adaptéru, jak ho ukazuje `adapters list`
help-adapters-allow-host-host = Hostitel k povolení: název DNS, žádná adresa, žádný port
help-adapters-disallow-host = Odvolá hostitele, kterého povolil allow-host
help-adapters-disallow-host-adapter = ID vestavěného adaptéru
help-adapters-disallow-host-host = Hostitel k odvolání
help-arg-adapters-from-project = Načte také adaptéry ve složce `adapters` složky projektu. freemapper každý z nich jmenuje spolu s hostiteli, na které se dotazuje. Použijte u projektu, kterému důvěřujete: adaptér přiměje freemapper dotazovat se serverů, které uvádí

## The summary of a run, for the MCP server and the area page

cli-internal-error = interní chyba: { $message }
cli-pdf-sources-invalid = pdf_sources musí být auto, full nebo credit-only, ne { $value }
cli-view-unknown = pohled je no-paths, brown-only, no-north-lines, no:<codes> nebo only:<codes>, ne { $value }
cli-view-empty = seznam { $value } neuvádí žádný kód symbolu
cli-view-bad-code = { $value } není kód symbolu jako 505 nebo 501.6
cli-summary-training-view = Zapsáno { $map } a { $pdf } (pohled na tréninkovou mapu)
cli-summary-kept-training-view = Ponechány soubory pohledu { $view } na tréninkovou mapu; k jejich obnovení zadejte --force
cli-summary-training-view-failed = Pohled na tréninkovou mapu selhal: { $error }
cli-estimate-disk-unreadable = nelze zjistit volné místo na disku pro { $path } ({ $message })
cli-summary-wrote-map = Zapsána mapa { $path }
cli-summary-kept-map = Ponechána existující mapa { $path } (může obsahovat práci mapéra)
cli-summary-map-lacks-templates = Mapa zatím neodkazuje na obrazové podklady. Spusťte pipeline s force; nahradí ji a starou ponechá jako .omap.bak.
cli-summary-map-lacks-buildings = Mapa zatím nemá budovy ALKIS. Spusťte pipeline s force, abyste je přidali; mapu nahradí a starou ponechá jako .omap.bak.
cli-summary-map-lacks-canopies = Mapě chybí přístřešky a přístřešky pro auta ALKIS (522), které jsou nyní objekty mapy. Spusťte pipeline s force, abyste je přidali; mapu nahradí a starou ponechá jako .omap.bak.
cli-summary-map-light = Základní mapa: spojeno linií: { $joined }, skryto nebo příliš malé, ponecháno pro vrstvy kandidátů: { $left } (nic se neposunulo)
cli-summary-map-light-kept = Vrstvy kandidátů nově obsahují také objekty nebo úseky ({ $left }), které tato verze ze základní mapy odebírá (označeno freemapper:generalized). Ponechaná mapa je stále má, takže se zobrazí dvakrát. Mapu vyčistíte spuštěním pipeline s force; starou ponechá jako .omap.bak.
cli-summary-map-old-candidates = Mapa stále uvádí templates/candidates.omap; tento běh ho naplnil všemi kandidáty. Kandidáti nově vycházejí také v sedmi vrstvách, každá v jednom souboru (templates/candidates-*.omap). Mapa je uvede po běhu s force (stará mapa zůstane jako .omap.bak), nebo je mapér přidá sám v OOM (Templates > Open template…).
cli-summary-wrote-ocad = Zapsána mapa OCAD { $path } (objektů: { $count })
cli-summary-kept-ocad = Ponechána existující mapa OCAD { $path } (může obsahovat práci mapéra; force ji nahradí)
cli-summary-ocad-old-candidates = Mapa OCAD stále uvádí { $path } jako mapu na pozadí; tento běh ji naplnil všemi kandidáty. Kandidáti nově vycházejí také v sedmi vrstvách, každá v jednom souboru (-candidates-*.ocd). Mapa OCAD je uvede po běhu s force (stará mapa zůstane jako .ocd.bak), nebo je mapér přidá sám v OCAD jako mapy na pozadí.
cli-summary-ocad-left-out = Z mapy OCAD vynecháno: { $skipped } objektů a { $dropped } degenerovaných částí
cli-summary-ocad-symbols = V OCAD zvolte Symbol > Load Colors and Symbols From… a vyberte oficiální sadu symbolů{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , školní sadu IOF 2019
       *[other] {" "}ISSprOM 2019
    }; nahradí zástupné symboly podle čísel.
cli-summary-ocad-failed = Export OCAD selhal: { $error }
cli-summary-pdf-without-credits = PDF bez uvedení zdrojů: ODbL a dl-de/by-2-0 vyžadují, abyste při předání mapy zdroje uvedli; viz ATTRIBUTION.txt
cli-summary-wrote-pdf = Zapsána mapa { $path } (bez kandidátů)
cli-summary-pdf-failed = Export PDF selhal: { $error }
cli-summary-wrote-pdf-candidates = Zapsán kontrolní tisk { $path } (mapa se všemi kandidáty, polohově věrně)
cli-summary-pdf-candidates-failed = Export kontrolního tisku selhal: { $error }
cli-summary-training-failed = Export tréninkové mapy selhal: { $error }
cli-summary-kept-training = Ponechána existující tréninková mapa { $path }, aby běh nepřepsal mapu, která byla vytištěna nebo předána dál; k jejímu obnovení nastavte force

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = tento systém nemá domovskou složku; použijte absolutní cestu
mcp-error-relative-path = { $path } je relativní cesta; použijte absolutní cestu nebo cestu začínající ~
mcp-error-home-folder = { $path } je samotná domovská složka; použijte složku v ní, např. ~/maps/school
mcp-error-no-project = v { $path } není žádný projekt freemapperu; nejdřív použijte create_project
mcp-error-unknown-kind = neznámý typ { $kind }; použijte "schoolyard", "sprint" nebo "forest"
mcp-error-reading = čtu { $path }: { $error }
mcp-error-one-area-source = zadejte přesně jedno z osm_id (z find_place) nebo geojson
mcp-error-page-open = stránka výběru území pro tento projekt je stále otevřená; požádejte uživatele, aby území uložil a na stránce stiskl Hotovo, a teprve potom spusťte pipeline
mcp-error-no-job = žádná úloha { $job }; start_pipeline a permits_for_area vracejí ID úloh této relace
mcp-error-job-running = { $job } stále běží pro { $path }; pipeline běží vždy jen jedna, protože projekty sdílejí mezipaměť stahování. Tuto spusťte, až project_status ohlásí finished nebo failed
mcp-error-page-start = stránku nelze spustit: { $error }
mcp-created-project = Vytvořen projekt { $path } pro { $area } ({ $kind }, 1:{ $scale }). Dále: estimate_download, potom start_pipeline.
mcp-created-project-given-area = Vytvořen projekt { $path } pro zadané území ({ $kind }, 1:{ $scale }). Dále: estimate_download, potom start_pipeline.
mcp-page-already-open = Stránka pro tuto složku je už otevřená; ponechává nastavení, se kterým byla otevřena.
mcp-page-next = Požádejte uživatele, aby na stránce vybral území, stiskl Uložit a potom Hotovo.
mcp-preview-of = Náhled vrstvy { $layer }
mcp-permits-next = Poznámky k povolením se vytvářejí. Dotazujte se project_status s tímto job_id; až bude úloha dokončena (finished), permits_md obsahuje text permits.md. Předejte ho jako orientační poznámku, ne právní poradenství.
mcp-tool-check-update = Zda existuje novější verze freemapperu: její verze, datum vydání, poznámky k vydání a velikost stažení, nebo že tato verze je aktuální. Nic neinstaluje.
mcp-tool-create-project = Vytvoří složku projektu pro území mapy. Zadejte osm_id (z find_place) nebo soubor GeoJSON. kind je schoolyard (1:1000, výchozí), sprint (1:4000) nebo forest (1:15000).
mcp-param-create-project-dir = Nová složka projektu (absolutní cesta nebo začínající ~).
mcp-param-create-project-geojson = Místo osm_id: soubor GeoJSON s polygonem území mapy (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (výchozí, 1:1000), "sprint" (1:4000) nebo "forest" (1:15000).
mcp-param-create-project-osm-id = Objekt OSM území mapy z find_place, např. "way/992423119".
mcp-param-create-project-part = S osm_id: kterou část oblasti o několika samostatných částech použít (1 je největší).
mcp-param-create-project-scale = Jmenovatel měřítka: schoolyard 1000 (výchozí); sprint 4000 (výchozí) nebo 5000; forest 15000 (výchozí), 10000 nebo 7500. Jiné hodnoty jsou odmítnuty; pro 1:3000 kreslete v 4000, freemapper-print při tisku zvětší.
mcp-param-create-project-title = Název mapy pro okrajové údaje (výchozí: název složky).
mcp-tool-estimate-download = Co start_pipeline pro projekt stáhne: dlaždice, každé stažení s velikostí v MB, zda je v mezipaměti a zda se vejde na disk, ještě stahované MB a v poli disk volné místo svazků mezipaměti a projektu se zprávami o tom, co se nevejde (sdělte to uživateli).
mcp-param-estimate-download-dir = Složka projektu (absolutní cesta nebo začínající ~).
mcp-tool-find-place = Vyhledá školu, park nebo jiné místo podle názvu (OpenStreetMap Nominatim). Vrací nejvýše 5 shod, oblasti první; územím mapy mohou být jen shody s has_area. Prázdný seznam znamená, že nic nebylo nalezeno.
mcp-param-find-place-query = Název školy nebo místa, např. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Nainstaluje nejnovější verzi freemapperu přes tento program. Volejte ji, teprve když osoba souhlasila, s confirm true. Nová verze se uplatní, až asistent server freemapperu restartuje.
mcp-param-install-update-confirm = true, jakmile osoba souhlasila s instalací novější verze; bez toho se nic neinstaluje.
mcp-tool-list-layers = Vrstvy projektu: mapa, sedm vrstev kandidátů a rastrové podklady, každá se svým souborem, zda už vznikla a který krok ji vytváří.
mcp-param-list-layers-dir = Složka projektu (absolutní cesta nebo začínající ~).
mcp-tool-open-area-selector = Otevře v prohlížeči stránku, kde uživatel nakreslí území mapy (obdélník nebo polygon) nebo zvolí místo, uvidí dlaždice a velikost stažení a území uloží; uložení vytvoří složku projektu nebo změní její území. Vrací URL, které se předá uživateli. Stránka běží, dokud uživatel nestiskne Hotovo; opakované volání pro stejnou složku vrátí stejné URL.
mcp-param-open-area-selector-dir = Složka projektu (absolutní cesta nebo začínající ~); vytvoří se, když uživatel uloží.
mcp-param-open-area-selector-kind = Typ mapy nového projektu: "schoolyard" (výchozí, 1:1000), "sprint" (1:4000) nebo "forest" (1:15000).
mcp-param-open-area-selector-ocad = Panel běhu se spustí se zaškrtnutým startovním projektem OCAD. Výchozí false.
mcp-param-open-area-selector-offline = Panel běhu se spustí se zaškrtnutou volbou „Jen mezipaměť, bez sítě“. Výchozí false.
mcp-param-open-area-selector-open-browser = Otevře stránku také ve výchozím prohlížeči tohoto počítače. Výchozí true.
mcp-param-open-area-selector-pdf = Panel běhu se spustí se zaškrtnutým tisknutelným PDF. Výchozí false.
mcp-param-open-area-selector-pdf-candidates = Panel běhu se spustí se zaškrtnutým kontrolním tiskem (každý kandidát). Výchozí false.
mcp-param-open-area-selector-pdf-scale-bar = Panel běhu se spustí se zaškrtnutým grafickým měřítkem v PDF. Výchozí false.
mcp-param-open-area-selector-pdf-sources = Řádky zdrojů v PDF, s nimiž se panel spustí: "auto" (výchozí), "full" nebo "credit-only", jako u start_pipeline.
mcp-param-open-area-selector-permits = Panel běhu se spustí se zaškrtnutými poznámkami k povolením. Výchozí false.
mcp-param-open-area-selector-scale = Jmenovatel měřítka nového projektu: schoolyard 1000 (výchozí); sprint 4000 (výchozí) nebo 5000; forest 15000 (výchozí), 10000 nebo 7500. Jiné hodnoty jsou odmítnuty.
mcp-param-open-area-selector-title = Název mapy nového projektu.
mcp-param-open-area-selector-training = Panel běhu se spustí se zaškrtnutou tréninkovou mapou. Výchozí false.
mcp-tool-preview-layer = Náhled PNG (nejvýše 800 px) vrstvy: map (obrázek objektů mapy, který zapsal poslední běh, bez kandidátů), nebo rastrový podklad: orthophoto, hillshade, slope, relief, ndsm, nebo k5 v Berlíně.
mcp-param-preview-layer-dir = Složka projektu (absolutní cesta nebo začínající ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" nebo "k5" (jen Berlín).
mcp-tool-project-status = Stav úlohy start_pipeline nebo permits_for_area: running, finished nebo failed, poslední řádky průběhu, upozornění, zapsané soubory a výsledek nebo chyba. Dokončená úloha permits_for_area nese také text permits.md jako permits_md.
mcp-param-project-status-job-id = ID úlohy z start_pipeline nebo permits_for_area, např. "job-1".
mcp-tool-permits-for-area = Vytvoří poznámky k povolením projektu: správní celky území mapy, chráněná území v něm a kdo tam spravuje pozemky, s úřady, na které se obrátit; orientační poznámka, ne právní poradenství. Zapíše permits.md a permits.json do složky. Běží na pozadí a vrací job_id; dotazujte se project_status, jehož odpověď po dokončení úlohy nese text permits.md jako permits_md. Vždy běží jen jedna úloha.
mcp-param-permits-for-area-dir = Složka projektu (absolutní cesta nebo začínající ~).
mcp-param-permits-for-area-offline = Čte jen data uložená v projektu dřívějším během, bez přístupu k síti. Výchozí false.
mcp-tool-start-pipeline = Stáhne data, vypočítá podklady a zapíše startovní projekt (fetch, derive, export). Běží na pozadí a vrací job_id; dotazujte se project_status. Vždy běží jen jedna pipeline. Existující mapu ponechá, pokud force není true.
mcp-param-start-pipeline-dir = Složka projektu (absolutní cesta nebo začínající ~).
mcp-param-start-pipeline-force = Nahradí existující soubor mapy a starý ponechá jako .omap.bak. Výchozí false.
mcp-param-start-pipeline-ocad = Zapíše také startovní bod OCAD do složky ocad/ (formát OCAD 12, pro uživatele, kteří upravují v OCAD). Výchozí false.
mcp-param-start-pipeline-offline = Použije jen mezipaměť stahování, bez přístupu k síti. Výchozí false.
mcp-param-start-pipeline-pdf = Zapíše také <name>.pdf, mapu připravenou k tisku (jen objekty mapy, bez kandidátů). Výchozí false.
mcp-param-start-pipeline-pdf-candidates = Zapíše také <name>-candidates.pdf, kontrolní tisk: mapu se všemi kandidáty, polohově věrně (ověřte je v terénu). Výchozí false.
mcp-param-start-pipeline-pdf-scale-bar = Nakreslí na PDF grafické měřítko. Výchozí false (německé mapy ho obvykle nemají; číslo měřítka zůstane v záhlaví).
mcp-param-start-pipeline-pdf-sources = Řádky zdrojů v PDF: "auto" (výchozí; úplný seznam, zkrácený, jen když zabere více než 5 řádků), "full" (nikdy nezkracovat) nebo "credit-only" (jen řádek s uvedením freemapperu; ODbL a dl-de/by-2-0 pak vyžadují uvést zdroje při předání mapy). ATTRIBUTION.txt uvádí vždy všechny zdroje.
mcp-param-start-pipeline-permits = Zapíše také poznámky k povolením (permits.md a permits.json) po mapě: správní celky, chráněná území a kdo spravuje pozemky. Selhání tam je upozornění, ne selhání úlohy. Výchozí false.
mcp-param-start-pipeline-training = Zapíše také tréninkovou mapu <name>-training.omap a její PDF <name>-training.pdf: mapu a každého kandidáta, upravené tak, aby se daly tisknout tak, jak jsou; s ocad také ocad/<name>-training.ocd. Existující tréninková mapa se ponechá, pokud force není true. Výchozí false.
mcp-param-start-pipeline-training-views = S training: zapíše také tréninkovou mapu bez toho, co pohled vynechává, jedna sada souborů na položku: "no-paths", "brown-only", "no-north-lines", nebo seznam jako "no:505,506" či "only:101,102", kombinovaný pomocí +. Soubory jsou <name>-training-<view>.omap a jeho PDF.
mcp-instructions = freemapper vytváří startovní projekty OpenOrienteering Mapper pro orientační mapy (školní dvůr 1:1000, sprint 1:4000, les 1:15000) z oficiálních geodat { $states } a z OSM. Postup: find_place (vyhledá školu nebo oblast podle názvu; vyberte shodu s has_area) → create_project (nová složka, absolutní cesta, s osm_id) → estimate_download (sdělte uživateli, kolik se bude stahovat) → start_pipeline (vrací ID úlohy; stahování může trvat minuty) → project_status (dotazujte se každých několik sekund, dokud stav není finished nebo failed) → list_layers a preview_layer (ukažte uživateli podklady). Uživatel pak otevře soubor mapy v OpenOrienteering Mapper. Když žádné místo nesedí nebo chce uživatel území nakreslit, open_area_selector otevře v prohlížeči stránku, kde uživatel území nakreslí nebo zvolí a uloží (tím se vytvoří projekt); pokračujte estimate_download, až uživatel řekne, že je uloženo. Cesty musí být absolutní nebo začínat ~. Vždy běží jen jedna pipeline. permits_for_area vytvoří poznámky k povolením projektu (správní celky, chráněná území, kdo spravuje pozemky; orientační poznámka, ne právní poradenství) jako úlohu stejně jako start_pipeline. check_update zjistí, zda existuje novější freemapper; install_update ho nainstaluje, jen s confirm: true poté, co osoba souhlasila, a asistent pak server freemapperu restartuje.

cli-contour-interval-saved = Interval vrstevnic uložen ve freemapper.toml: { $interval } m; derive vede vrstevnice podle něj
cli-contour-interval-auto = Interval vrstevnic ve freemapper.toml: automaticky (podle typu mapy; lesní mapa má 2,5 m v rovině a 10 m ve strmém terénu)
help-arg-contour-interval = Interval vrstevnic v metrech, např. `2.5`, nebo `auto` pro výchozí podle typu mapy (lesní mapa: 5 m, 2,5 m v rovině, 10 m ve strmém terénu). Uloženo ve freemapper.toml; derive vede vrstevnice podle něj
cli-content-saved = Nastavení obsahu uloženo ve freemapper.toml: { $summary }
cli-content-invalid = Nastavení obsahu nelze přečíst: { $detail }
help-arg-content = Co jde přímo na mapu: `safe` ponechá každý objekt tam, kam ho sestavení umístí, `balanced` (výchozí) dá na mapu objekty katastru a rejstříků, `full` dá na mapu všechno. Volba se ukládá do freemapper.toml
help-arg-content-set = Pošle skupinu objektů do `map`, `candidates` nebo `off`, jako `GROUP=PLACE`, např. `walls=map`; lze opakovat. Skupiny: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Uloženo ve freemapper.toml
mcp-param-create-project-content = Co jde přímo na mapu: "safe", "balanced" (výchozí) nebo "full". Uloženo v projektu. balanced dá na mapu objekty katastru a rejstříků, full dá na mapu všechno.
mcp-param-create-project-content-set = Skupiny poslané jinam, než říká profil, jako "group=place" s place map, candidates nebo off, např. "walls=map". Skupiny: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Interval vrstevnic v metrech, např. 2.5; 0 se vrátí k automatické volbě (podle typu mapy; lesní mapa má 2,5 m v rovině a 10 m ve strmém terénu). Uloženo v projektu; příští derive vede vrstevnice podle něj.
mcp-param-start-pipeline-content = Před během změní, co jde přímo na mapu: "safe", "balanced" nebo "full". Uloženo v projektu. Výchozí: nastavení projektu, jinak balanced.
mcp-param-start-pipeline-content-set = Skupiny poslané jinam, než říká profil, jako "group=place" s place map, candidates nebo off, např. "fences=candidates". Uloženo v projektu.
mcp-param-start-pipeline-contour-interval = Interval vrstevnic v metrech, např. 2.5; 0 se vrátí k automatické volbě (podle typu mapy; lesní mapa má 2,5 m v rovině a 10 m ve strmém terénu). Uloženo v projektu; příští derive vede vrstevnice podle něj.
mcp-error-content = Nastavení obsahu nelze přečíst: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Zapíše poznámky k povolením projektu (permits.md a permits.json): správní celky, chráněná území a provozovatele, kterých se území mapy dotýká, a koho je obvykle třeba požádat. Dotazuje se BfN a BKG a, má-li spolková země území mapy vlastní otevřené služby, i těch přes hostitele jejího adaptéru (seznam v manuálu, kapitola 12); v Severním Porýní-Vestfálsku při prvním použití stahuje kolem 32 MB, v Braniborsku kolem 4 MB, později jen novější soubory. Čte výřez OSM, který už je v mezipaměti; --offline použije to, co bylo staženo dříve
help-permits-dir = Složka projektu
help-permits-offline = Použije jen dříve stažená data, bez přístupu k síti
help-run-permits = Zapíše po mapě také poznámky k povolením (permits.md a permits.json); dotazuje se BfN, BKG a otevřených služeb spolkové země (manuál, kapitola 12). Selhání tam je upozornění a mapa zůstane
cli-wrote-permits = Zapsány poznámky k povolením { $path }
cli-summary-wrote-permits = Zapsány poznámky k povolením { $path }
cli-permits-failed = poznámky k povolením se nepodařilo zapsat: { $error }
cli-permits-source-left-out = poznámky k povolením vynechávají { $source }: { $reason }
cli-permits-not-read = nebylo přečteno
error-permits-stored = { $path }: { $message }; stáhněte poznámky k povolením znovu online
error-permits-nothing-read = nepodařilo se přečíst žádný zdroj poznámek k povolením ({ $missing }); nejdřív je stáhněte online
error-permits-missing-not-read = { $source }: nepřečteno

permits-title = Poznámky k povolením
permits-disclaimer = Orientační poznámka, ne právní poradenství. Rozhodují příslušné úřady. Datum a přesnost dat viz tabulka pod „Zdroje a datum dat“; plochy mohou chybět nebo být zastaralé (BfN: sloučeno jednou ročně, „Nicht für Planungszwecke geeignet.“, tedy nevhodné pro plánování).
permits-disclaimer-regional = Orientační poznámka, ne právní poradenství. Rozhodují příslušné úřady. Datum a přesnost dat viz tabulka pod „Zdroje a datum dat“; plochy mohou chybět nebo být zastaralé.
permits-bfn-merge = BfN slučuje data spolkových zemí jednou ročně a označuje je jako nevhodná pro plánování. Rozhoduje úřad spolkové země.
permits-created = Vytvořeno { $date } pro území mapy { $kind ->
        [schoolyard] školního dvora
        [forest] lesní mapy
       *[other] sprintové mapy
    } ({ $area } ha).
permits-section-admin = Správní celky
permits-section-protected = Chráněná území (BfN)
permits-section-osm-protected = Další chráněná území z OpenStreetMap
permits-section-operators = Provozovatelé a vlastníci podle OpenStreetMap
permits-section-hints = Pokyny
permits-section-sources = Zdroje a datum dat
permits-none-admin = VG250 neuvádí pro území mapy žádný správní celek.
permits-none-protected = Podle BfN v území mapy neleží žádné chráněné území.
permits-none-osm-protected = OpenStreetMap neuvádí v území mapy žádné chráněné území, které BfN ještě nevede.
permits-none-operators = OpenStreetMap neuvádí pro území mapy žádného provozovatele ani vlastníka.
permits-none-hints = Žádné pokyny.
permits-section-not-read = Data { $source } nebyla přečtena, proto je tato část prázdná, i když v ní mohou být záznamy.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Správní oblasti 1:250 000 (VG250) od BKG
        [bfn] Chráněná území v Německu od BfN
       *[osm] OpenStreetMap, z výřezu, který freemapper drží v mezipaměti
    }
permits-level = { $level ->
        [land] Spolková země
        [kreis] Okres (Kreis)
        [verwaltungsgemeinschaft] Amt nebo Verwaltungsgemeinschaft
       *[gemeinde] Obec (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Dotčeno na okraji (méně než 0,5 % území mapy):
permits-at-edge-generalised = VG250 je generalizováno na měřítko 1:250 000. Celek, který se jen dotýká okraje, může být jeho důsledkem.
permits-kind = { $kind ->
        [nationalpark] Národní park
        [nationales-naturmonument] Národní přírodní monument (Nationales Naturmonument)
        [naturschutzgebiet] Přírodní rezervace (Naturschutzgebiet)
        [ffh] Oblast FFH (Natura 2000)
        [vogelschutzgebiet] Ptačí oblast (Natura 2000)
        [biosphaerenreservat] Biosférická rezervace
        [biosphaerenreservat-zone] Zóna biosférické rezervace
        [landschaftsschutzgebiet] Chráněná krajinná oblast (Landschaftsschutzgebiet)
       *[naturpark] Přírodní park
    }
permits-protected-line = { $kind } „{ $name }“, { $share }
permits-protected-date = Vyhlášeno { $date }
permits-protected-title = Právní základ: { $title }
permits-protected-link = Právní text: { $url }
permits-protected-site-link = Stránka lokality (Natura 2000): { $url }
permits-protected-year = Rok dat u BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = třída ochrany { $class }
permits-quoted = „{ $text }“
permits-osm-unnamed = bez názvu
permits-osm-operator = Provozovatel podle OSM: { $operator }
permits-osm-enclosing-gap = Plochy OpenStreetMap, které celé území mapy obklopují, aniž by v něm ležel některý jejich roh, tu mohou chybět, stejně jako chráněná území zanesená v OSM jako relace s `type=boundary`. Části z BfN a VG250 se to netýká.
permits-osm-enclosing-gap-short = Plochy OpenStreetMap, které celé území mapy obklopují, aniž by v něm ležel některý jejich roh, tu mohou chybět.
permits-operator-line = { $name } ({ $roles ->
        [operator] provozovatel
        [owner] vlastník
       *[both] provozovatel a vlastník
    }): { $objects }
permits-column-source = Zdroj
permits-column-state = Stav
permits-column-credit = Uvedení původu a licence
permits-state-read = přečteno, staženo { $date }
permits-state-read-osm = přečteno, výřez OSM ze dne { $date }
permits-state-unreachable = nedostupné ({ $reason })
permits-state-not-read = nepřečteno
permits-no-credit = nepoužito
permits-hint-city-state = Území mapy leží v Berlíně nebo Hamburku. Tam je obvykle příslušný Bezirksamt (úřad městského obvodu) nebo úřad spolkové země. Tyto poznámky zatím Bezirk neurčují.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Chráněných území se týká nižší orgán ochrany přírody (untere Naturschutzbehörde). Tím je zde samo město { $kreis }.
        [no] Chráněných území se týká nižší orgán ochrany přírody (untere Naturschutzbehörde), obvykle { $kreis }.
       *[unknown] Chráněných území se týká příslušný nižší orgán ochrany přírody (untere Naturschutzbehörde), obvykle Landkreis nebo kreisfreie Stadt.
    }
permits-hint-national-park = Národní park má vlastní pravidla. Obraťte se na správu národního parku včas.
permits-hint-nsg-paths = V přírodní rezervaci se obvykle musíte držet cest. Pro běh mimo cesty se zeptejte orgánu ochrany přírody.
permits-hint-natura2000 = Oblast FFH nebo ptačí oblast (Natura 2000) zasahuje do území mapy. U akce může orgán ochrany přírody požadovat posouzení jejích dopadů.
permits-hint-protected-areas-pl = V území mapy leží chráněná území nebo chráněné objekty ochrany přírody. U národního parku, krajinného parku nebo přírodní rezervace je obvyklým kontaktem správa parku nebo rezervace; u ostatních území a objektů obvykle řekne, kdo je příslušný, Regionální ředitelství ochrany životního prostředí (RDOŚ) vojvodství. Zeptejte se včas, když tam plánujete akci.
permits-hint-order-office = { $amt ->
        [none] Za ulice, náměstí a další veřejné pozemky obvykle odpovídá Ordnungsamt (úřad veřejného pořádku) obce { $gemeinde }.
       *[other] Za ulice, náměstí a další veřejné pozemky obvykle odpovídá Ordnungsamt (úřad veřejného pořádku): { $amt }, pro obec { $gemeinde }.
    }
permits-hint-park-operator = Provozovatel parku nebo zahrady v území mapy podle OpenStreetMap: { $operator }. Zeptejte se ho, než park využijete.
permits-hint-school-operator = Provozovatel školy nebo zařízení podle OpenStreetMap: { $operator }. Pro jeho pozemky se zeptejte ředitele školy nebo tohoto subjektu.
permits-section-districts = Bezirke (městské obvody, { $source })
permits-section-green-spaces = Zelené plochy ({ $source })
permits-section-owners = Pozemky spolkové země a Bundu ({ $source })
permits-section-monuments = Památky ({ $source })
permits-section-protected-objects = Přírodní památky a jiné chráněné objekty ({ $source })
permits-none-districts = Podle { $source } území mapy neleží v žádném Bezirku.
permits-none-green-spaces = Podle { $source } v území mapy neleží žádná veřejná zelená plocha.
permits-none-owners = Podle { $source } v území mapy neleží žádná parcela spolkové země ani Bundu.
permits-none-monuments = Podle { $source } v území mapy neleží žádná památka.
permits-none-protected-objects = Podle { $source } v území mapy neleží žádný z chráněných objektů, které vede.
permits-none-sampled = V dotazovaných bodech { $source } zde nic neuvádí.
permits-gaps = Data { $source } nejsou úplná: { $gaps }.
permits-sampled = { $source } odpovídá jen pro jednotlivé body. freemapper se ptal v mřížce bodů přes mapu: podíly jsou odhady a to, co leží mezi body, může chybět.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = určení: { $dedication }
permits-playgrounds-none = V území mapy neleží žádné veřejné dětské hřiště.
permits-playgrounds-one = V území mapy leží jedno veřejné dětské hřiště. Samostatně se neuvádí.
permits-playgrounds-many = V území mapy leží veřejná dětská hřiště: { $count }. Jednotlivě se neuvádějí.
permits-owner-other = Ostatní veřejní vlastníci
permits-owners-rest = Pro zbytek území data neuvádějí žádného vlastníka.
permits-monuments-total = Památky zcela nebo zčásti v území mapy: { $count }.{ $links ->
        [yes] {" "}Odkazy vedou na jejich záznamy v seznamu památek.
       *[no] {""}
    }
permits-monuments-links-left = Dalších záznamů je jen započítáno, bez odkazu: { $count }.
permits-object-kind = { $kind ->
        [natural-monument] Přírodní památka (Naturdenkmal)
        [protected-landscape-part] Chráněný krajinný prvek (geschützter Landschaftsbestandteil)
        [biotope] Zákonem chráněný biotop (geschütztes Biotop)
        [national-park] Národní park (park narodowy)
        [park-buffer-zone] Ochranné pásmo parku (otulina)
        [landscape-park] Krajinný park (park krajobrazowy)
        [nature-reserve] Přírodní rezervace (rezerwat przyrody)
        [protected-landscape-area] Chráněná krajinná oblast (obszar chronionego krajobrazu)
        [natura2000-habitats] Lokalita Natura 2000 pro stanoviště (SOO)
        [natura2000-birds] Lokalita Natura 2000 pro ptáky (OSO)
        [ecological-site] Ekologická plocha (użytek ekologiczny)
        [landscape-complex] Přírodní a krajinný celek (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentační lokalita (stanowisko dokumentacyjne)
        [natural-monument-pl] Přírodní památka (pomnik przyrody)
       *[special-rules] Oblast se zvláštními pravidly
    }
permits-object-count = objektů: { $count }
permits-object-citations = Právní základy:
permits-source-note = { $source }: „{ $note }“
permits-hint-district-office = { $offices ->
        [hamburg] Za ulice, náměstí a zelené plochy v Bezirku { $district } je obvykle příslušný Bezirksamt { $district }, tam Fachamt Management des öffentlichen Raumes (odbor veřejného prostoru).
       *[berlin] Za ulice, náměstí a zelené plochy v Bezirku { $district } jsou obvykle příslušné Ordnungsamt (úřad veřejného pořádku) a Straßen- und Grünflächenamt (úřad silnic a zeleně) úřadu Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Za památky v území mapy odpovídá Denkmalschutzamt Hamburg (památkový úřad).
       *[berlin] Za památky v Bezirku { $district } odpovídá nižší památkový orgán (untere Denkmalschutzbehörde) úřadu Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Zda v území mapy leží památky ze seznamu, zde uvedená data nedokážou říct jistě.
       *[yes] V území mapy jsou památky ze seznamu.
    } O tom, zda se jich běh dotkne, například stanovištěm u památky nebo úsekem přes chráněné místo, rozhoduje nižší památkový orgán (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Tím je zde samo město { $kreis }.
        [no] , obvykle { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; v některých spolkových zemích je to obec { $gemeinde }.
        }
       *[unknown] {" "}(obvykle Landkreis nebo kreisfreie Stadt, v některých spolkových zemích obec).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Přírodní památky, chráněné krajinné prvky a chráněné biotopy na mapě spadají pod nižší orgán ochrany přírody (untere Naturschutzbehörde). Tím je zde samo město { $kreis }.
        [no] Přírodní památky, chráněné krajinné prvky a chráněné biotopy na mapě spadají pod nižší orgán ochrany přírody (untere Naturschutzbehörde), obvykle { $kreis }.
       *[unknown] Přírodní památky, chráněné krajinné prvky a chráněné biotopy na mapě spadají pod nižší orgán ochrany přírody (untere Naturschutzbehörde), obvykle Landkreis nebo kreisfreie Stadt.
    }
permits-hint-biotope = V území mapy leží zákonem chráněné biotopy{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Vše, co je může zničit nebo značně poškodit, je zakázáno (§ 30 BNatSchG). Stanoviště a úseky umístěte mimo ně, nebo se zeptejte nižšího orgánu ochrany přírody (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Za přírodní památky a chráněné krajinné prvky v Bezirku { $district } je obvykle příslušný Bezirksamt { $district }.
       *[berlin] Za přírodní památky a chráněné krajinné prvky v Bezirku { $district } odpovídá nižší orgán ochrany přírody (untere Naturschutzbehörde) úřadu Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Podle otevřených dat spolkové země patří { $share } území mapy jiným veřejným vlastníkům.
       *[no] Podle otevřených dat spolkové země patří { $share } území mapy vlastníkovi „{ $owner }“.
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Vlastníky zbytku území nelze z otevřených dat zjistit. Při oprávněném zájmu (například jako pořadatel) můžete u katastrálního úřadu (Grundbuchamt) požádat o výpis; u veřejných pozemků pomůže příslušný Ordnungsamt nebo správa parků.
       *[no] Vlastníky těchto ploch nelze z otevřených dat zjistit. Při oprávněném zájmu (například jako pořadatel) můžete u katastrálního úřadu (Grundbuchamt) požádat o výpis; u veřejných pozemků pomůže příslušný Ordnungsamt nebo správa parků.
    }
permits-section-forest-units = Lesní správa ({ $source })
permits-section-state-forest = Les spolkové země ({ $source })
permits-none-forest-units = Podle { $source } území mapy neleží v působnosti žádného lesního úřadu.
permits-section-admin-units = Správní jednotky ({ $source })
permits-none-admin-units = Podle { $source } oblast mapy neleží v žádné správní jednotce registru.
permits-section-register-monuments = Památky národního rejstříku ({ $source })
permits-none-register-monuments = Podle { $source } v obdélníku kolem oblasti mapy neleží žádná zapsaná památka.
permits-register-monuments-total = Zapsané památky v obdélníku kolem oblasti mapy: { $count }. Rejstřík neuvádí polohu, tyto pokyny proto neřeknou, kde leží; je třeba to ověřit na mapě.
permits-register-monuments-left = Dalších { $count } je zde jen započteno, bez názvů.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Podle { $source } v území mapy neleží žádný les spolkové země.
permits-forest-units-reach = Lesní úřad odpovídá za les ve své působnosti. Jeho působnost může zahrnovat i města a pole.
permits-state-forest-office = příslušný úřad: { $office }
permits-hint-forest-office = { $units ->
        [none] Lesní úřad příslušný pro les v území mapy: { $office } ({ $administration }). Při plánování akce v lese se tam zeptejte včas.
       *[other] Lesní úřad příslušný pro les v území mapy: { $office } ({ $administration }), { $units }. Při plánování akce v lese se tam zeptejte včas.
    }
permits-hint-forest-owner = { $office ->
        [known] Akce v lese potřebuje souhlas vlastníků lesa. Lesní úřad obvykle ví, komu les patří. V několika spolkových zemích se musí organizovaná akce v lese také hlásit lesnímu orgánu nebo jím být povolena.
       *[unknown] Akce v lese potřebuje souhlas vlastníků lesa. V několika spolkových zemích se musí organizovaná akce v lese také hlásit lesnímu orgánu nebo jím být povolena. Tyto poznámky zde příslušný lesní úřad neuvádějí; sdělí ho lesní správa spolkové země nebo nižší lesní orgán.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Akce v lese obvykle potřebuje souhlas vlastníka lesa. Výše uvedená lesní správa (Nadleśnictwo) obvykle ví, kdo se o les stará. Les, který nepatří Státním lesům, například soukromý les, v datech těchto poznámek není. Les, který nepatří státu, například soukromý nebo obecní les, podléhá v lesním hospodářství dozoru starosty (starosta) (zákon o lesích, čl. 5 odst. 1 bod 2); zeptejte se tam na úřadu (starostwo powiatowe).
       *[unknown] Akce v lese obvykle potřebuje souhlas vlastníka lesa. Tyto poznámky pro území mapy neuvádějí žádnou lesní správu Státních lesů; data zahrnují jen Státní lesy, takže les může patřit někomu jinému, například soukromému vlastníkovi. Kdo je vlastníkem, je třeba zjistit zvlášť. Les, který nepatří státu, například soukromý nebo obecní les, podléhá v lesním hospodářství dozoru starosty (starosta) (zákon o lesích, čl. 5 odst. 1 bod 2); zeptejte se tam na úřadu (starostwo powiatowe).
    }
permits-hint-monument-register-pl = V obdélníku kolem oblasti mapy leží památky národního rejstříku. Jejich poloha v datech není, ověřte na mapě, kde jsou. Podle čl. 36 odst. 1 zákona o ochraně památek (ustawa o ochronie zabytków) vyžadují práce na zapsané památce, stavební práce v okolí památky a jiné činnosti, které by mohly změnit její podstatu nebo vzhled, povolení vojvodského památkáře (Wojewódzki Konserwator Zabytków). Zákon neuvádí běh ani kontrolu jako takové; zeptejte se úřadu památkáře, pokud se akce dotýká památky nebo jejího okolí.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] V oblasti mapy leží přírodní památka (pomnik przyrody), ekologická plocha (użytek ekologiczny), přírodní a krajinný celek (zespół przyrodniczo-krajobrazowy) nebo dokumentační lokalita (stanowisko dokumentacyjne). Zřizuje je zastupitelstvo gminy usnesením, které určuje, kdo nad objektem dohlíží a jaké zákazy platí (čl. 44 zákona o ochraně přírody). Zákon sám pro akci zde žádné povolení nestanovuje. Požádejte úřad gminy, na jejímž území objekt leží, o usnesení a o určení dohlížející osoby.
       *[other] V oblasti mapy leží přírodní památka (pomnik przyrody), ekologická plocha (użytek ekologiczny), přírodní a krajinný celek (zespół przyrodniczo-krajobrazowy) nebo dokumentační lokalita (stanowisko dokumentacyjne). Zřizuje je zastupitelstvo gminy usnesením, které určuje, kdo nad objektem dohlíží a jaké zákazy platí (čl. 44 zákona o ochraně přírody). Zákon sám pro akci zde žádné povolení nestanovuje. Požádejte úřad gminy ({ $gmina }), na jejímž území objekt leží, o usnesení a o určení dohlížející osoby.
    }
permits-hint-gmina-roads-pl = Akce, která brání provozu nebo využívá ulici zvláštním způsobem, potřebuje povolení podle čl. 65 zákona o silničním provozu; žádost se podává nejméně 30 dní předem. Povolení vydává orgán řídící provoz na dané silnici: starosta pro silnice powiatu a gminy, primátor ve městě s právy powiatu, maršálek vojvodství nebo GDDKiA pro ostatní silnice (čl. 10 a 65b). Správcem silnice gminy je wójt, burmistrz nebo prezydent miasta gminy ({ $gmina }); správce rozhoduje o záboru silničního pásma (čl. 19 a 40 zákona o veřejných komunikacích). Tyto poznámky neukazují kategorii silnic.
permits-hint-gmina-land-pl = Pokud pozemek v oblasti mapy patří gmině, například městský park, náměstí, školní areál nebo les, spravuje jej wójt, burmistrz nebo prezydent miasta, který gminu zastupuje (čl. 30 a 31 zákona o samosprávě gmin). Před použitím požádejte o souhlas úřad gminy ({ $gmina }). Tyto poznámky neukazují, komu pozemek patří.
permits-section-water-protection = Ochranná pásma vodních zdrojů ({ $source })
permits-section-forest-functions = Funkce lesa a ochranný les ({ $source })
permits-none-water-protection = Podle { $source } v území mapy neleží žádné ochranné pásmo vodních zdrojů.
permits-none-forest-functions = Podle { $source } v území mapy neleží žádný rekreační les, ochranný les, přírodní les ani uzavřená lesní plocha.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Rekreační les (Erholungswald)
        [protected] Ochranný les (Bannwald nebo Schutzwald)
        [natural] Přírodní les nebo přísná rezervace (Naturwald, Totalreservat)
       *[closed] Uzavřená lesní plocha
    }
permits-hint-water-protection = { $inner ->
        [yes] V území mapy leží { $zone } ochranného pásma vodních zdrojů „{ $name }“. V pásmech I a II jsou akce obvykle vyloučeny. Zeptejte se nižšího vodoprávního orgánu (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] V území mapy leží { $zone } ochranného pásma vodních zdrojů „{ $name }“. Akce tam jsou obvykle možné; nižší vodoprávní orgán (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) řekne, co nařízení dovoluje.
       *[unknown] V území mapy leží ochranné pásmo vodních zdrojů „{ $name }“. Data neříkají, o které pásmo jde. V pásmech I a II jsou akce obvykle vyloučeny; zeptejte se nižšího vodoprávního orgánu (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] V území mapy leží rekreační les (Erholungswald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pro akce tam se zeptejte lesního orgánu.
        [protected] V území mapy leží ochranný les (Bannwald nebo Schutzwald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Než tam budete plánovat akci, zeptejte se lesního orgánu.
        [natural] V území mapy leží přírodní les nebo přísná rezervace{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Opouštět tam cesty je často zakázáno; bez dotazu u lesního orgánu tam nestavte žádná stanoviště.
       *[closed] V území mapy leží lesní plocha, do které je vstup zakázán{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Do trati ji nezahrnujte.
    }
permits-section-roads = Klasifikované silnice podle OpenStreetMap
permits-none-roads = Podle OpenStreetMap územím mapy neprochází žádná dálnice (Autobahn) ani silnice Bundes-, Landes-, Staats- či Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Dálnice (Autobahn)
        [bundesstrasse] Spolková silnice (Bundesstraße)
        [landesstrasse] Zemská silnice (Landesstraße nebo Staatsstraße)
       *[kreisstrasse] Okresní silnice (Kreisstraße)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Územím mapy prochází dálnice (Autobahn) { $refs }. Trať ji smí křížit jen po mostech nebo podchodech.
        [bundesstrasse] Územím mapy prochází spolková silnice (Bundesstraße) { $refs }. Uzavírku nebo přechod s pořadateli nařizuje silniční správní úřad (Straßenverkehrsbehörde); silnici obvykle spravuje Bund prostřednictvím silniční správy spolkové země, ve větších městech často město.
        [landesstrasse] Územím mapy prochází zemská silnice (Landes- nebo Staatsstraße) { $refs }. Uzavírku nebo přechod s pořadateli nařizuje silniční správní úřad (Straßenverkehrsbehörde); silnici obvykle spravuje spolková země, ve větších městech často město.
       *[kreisstrasse] Územím mapy prochází okresní silnice (Kreisstraße) { $refs }. Uzavírku nebo přechod s pořadateli nařizuje silniční správní úřad (Straßenverkehrsbehörde); silnici obvykle spravuje okres (Kreis), ve větších městech často město.
    }
permits-hint-state-forest = { $office ->
        [none] Podle otevřených dat spolkové země je { $share } území mapy les spolkové země vlastníka „{ $owner }“.
       *[other] Podle otevřených dat spolkové země je { $share } území mapy les spolkové země vlastníka „{ $owner }“. Příslušný úřad: { $office }.
    }
