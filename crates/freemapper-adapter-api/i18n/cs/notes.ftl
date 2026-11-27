# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = upozornění: { $text }

warning-other-zone = území mapy leží v zóně UTM { $zone } (EPSG:{ $epsg }); freemapper v této zóně stáhne vše znovu
warning-other-state = území mapy nyní leží v datech jiné spolkové země; freemapper stáhne vše znovu
warning-beyond-coverage = část území mapy leží mimo podporované spolkové země. Leží v zahraničí, na moři nebo ve spolkové zemi, kterou freemapper zatím nepodporuje, například v Durynsku. Spolkové země tam nemají data, takže ortofoto může být bílé a ALKIS nedodá budovy ani využití pozemků
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: měřítko 1:{ $scale } není mezi měřítky pro mapy typu { $kind } ({ $allowed }); freemapper-print mapu odmítne
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorován záznam { $key } vrstvy { $german ->
       *[any] { $what }
    }: { $reason ->
        [name] není to prostý název souboru
       *[unknown] freemapper takovou vrstvu nezná
    }

## Disk space

warning-disk-space-unreadable = nelze zjistit volné místo na disku pro { $path } ({ $error }), proto ho freemapper nekontroluje

## Tiles

warning-tile-changed = dlaždice { $tile } produktu { $product } se u zdroje změnila ({ $old } → { $new })
warning-bdom-tile-unavailable = model povrchu (bDOM) pro dlaždici { $tile } není k dispozici ({ $error }); tam žádný podklad nDSM
warning-bdom-unavailable = model povrchu (bDOM) není k dispozici ({ $error }); žádný podklad nDSM pro { $tiles }
progress-ndsm-from-bdom = nDSM dlaždice { $tile } z modelu povrchu (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] model terénu
        [surface] model povrchu
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = stahuji: { $model }
progress-model-read = čtu: { $model }
warning-model-unavailable = { $model } není k dispozici ({ $error }); žádný podklad nDSM
warning-model-unavailable-kept = { $model } není k dispozici ({ $error }); ponechán model povrchu stažený { $fetched }
warning-model-tile-unavailable = { $model }, dlaždice { $tile }: není k dispozici ({ $error }); tam žádný podklad nDSM
warning-archive-passed-over = { $model }: archiv nelze přečíst ({ $error }); byl přeskočen a dlaždice území mapy pocházejí z ostatních archivů
warning-archive-directory-kept = { $model }: archiv nelze přečíst ({ $error }); použije se jeho adresář uchovaný z dřívějšího běhu, takže dlaždice, kterou LVGL mezitím nahradil, nebude zaznamenána
warning-model-tiles-missing = { $model }: v rámci { $state ->
        [the-map-area] území mapy
        [hamburg] Hamburku
        [hesse] Hesenska
        [north-rhine-westphalia] Severního Porýní-Vestfálska
        [schleswig-holstein] Šlesvicka-Holštýnska
        [mecklenburg-vorpommern] Meklenburska-Předního Pomořanska
        [lower-saxony] Dolního Saska
        [bavaria] Bavorska
       *[other] { $state }
    } { $count ->
        [one] chybí dlaždice
       *[other] chybějí dlaždice
    } { $names }; podklady tam zůstanou prázdné
warning-no-surface-stored = není uložen žádný model povrchu; pro podklad nDSM spusťte online `freemapper fetch`
warning-surface-older-than-orthophoto = model povrchu (bDOM { $surface }) je starší než ortofoto ({ $photo }); v nDSM mohou chybět nové budovy
progress-surface-kept = freemapper pro podklad nDSM ponechává model povrchu stažený { $fetched }
progress-surface-tiles-kept = freemapper pro podklad nDSM ponechává model povrchu z mezipaměti pro { $count ->
        [one] dlaždici
        [few] dlaždice
        [many] dlaždic
       *[other] dlaždic
    } { $names }
progress-orthophoto-unchanged = ortofoto beze změny
progress-orthophoto-unchanged-dated = ortofoto beze změny ({ $label } { $date })
progress-orthophoto-download = stahuji ortofoto z { $provider } { $service }
progress-orthophoto-in-leaf = ortofoto bylo pořízeno { $date }, když stromy měly listí; jejich koruny zakrývají, co leží pod nimi
progress-orthophoto-bare = ortofoto bylo pořízeno { $date }, když byly stromy bez listí
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; ponecháno uložené ortofoto a datum jeho snímkování { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): bez data snímkování, { $reason }
word-stac-no-image = STAC neuvádí žádný snímek ve středu území mapy
word-stac-no-answer = vyhledávání ve STAC nedalo použitelnou odpověď ({ $error })
warning-downloadcenter-no-file = Downloadcenter nemá soubor pro { $places }
warning-downloadcenter-no-file-empty = Downloadcenter nemá soubor pro { $places }; { $count ->
        [one] dlaždice { $names } zůstala prázdná
       *[other] dlaždice { $names } zůstaly prázdné
    }
progress-k-five-unchanged = podklad K5 beze změny
progress-k-five-download = stahuji mapu K5 (1:5000) z WMS Geoportal Berlin
warning-k-five-unavailable = podklad mapy K5 není k dispozici ({ $detail }); přeskočeno

## The state border

progress-beyond-border = území mapy zasahuje za hranici spolkové země; za ní jsou podklady prázdné všude tam, kde končí ortofoto a výškové modely poskytovatele { $provider }. ALKIS a OSM pocházejí z obou spolkových zemí
progress-beyond-border-no-data = území mapy zasahuje za hranici spolkové země; pro { $tiles } nejsou data poskytovatele { $provider }, takže tam podklady zůstanou prázdné

## OSM

warning-osm-extract-missing-kept = v mezipaměti není žádný výřez OSM; ponechávám uložené kandidáty OSM z { $date }
progress-osm-extract-read = čtu výřez OSM { $region } z { $date }
progress-osm-candidates = kandidátních objektů OSM: { $count }

## Permit notes

progress-permits-fetch = stahuji data { $source } pro poznámky k povolením
progress-permits-osm = čtu výřez OSM { $region } z { $date } pro poznámky k povolením
progress-permits-stored = čtu uložená data poznámek k povolením
warning-permits-source-unreachable = { $source } nelze přečíst, proto ho poznámky k povolením vynechají: { $error }
warning-permits-osm-no-extract = pro toto území mapy není v mezipaměti žádný výřez OSM, proto poznámky k povolením OSM vynechají; `freemapper fetch` ho stáhne

## ALKIS

warning-alkis-coverage-unknown = pokrytí budovami { $cadastre } není známo; zaznamenáte ho spuštěním `freemapper fetch` online
progress-alkis-buildings-download = stahuji budovy { $cadastre }
progress-alkis-buildings = budov { $cadastre }: { $count }
warning-alkis-no-buildings = v území mapy nebyly nalezeny žádné budovy { $cadastre }
progress-alkis-skipped = přeskočeno prvků { $cadastre } ({ $count }): { $list }
warning-alkis-buildings-changed = budovy { $cadastre } se u zdroje změnily ({ $changes })
progress-alkis-no-land-use-stored = není uloženo žádné využití pozemků { $cadastre }; pro kandidáty zakázaného prostoru spusťte online `freemapper fetch`
progress-alkis-land-use-download = stahuji využití pozemků { $cadastre }
progress-alkis-land-use-parts = částí využití pozemků { $cadastre }: { $count }
warning-alkis-land-use-kept = využití pozemků { $cadastre } není k dispozici ({ $error }); ponechávám uložené využití pozemků
warning-alkis-land-use-removed = využití pozemků { $cadastre } není k dispozici ({ $error }); uložené využití pozemků už nepokrývá území mapy, proto ho freemapper odstranil
warning-alkis-land-use-none = využití pozemků { $cadastre } není k dispozici ({ $error }); žádní kandidáti zakázaného prostoru
progress-bdot-filled = doplněno { $count } objektů BDOT10k ({ $what }) tam, kde EGiB nic nedává ({ $powiat })
warning-bdot-failed = BDOT10k nemohl zaplnit mezeru v EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB není dostupný ({ $error }); místo něj se použilo BDOT10k ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = nejsou uloženy žádné struktury ALKIS Berlína; pro schody, zdi, stromy a přístřešky spusťte online `freemapper fetch`
progress-berlin-removing = odstraňuji uložené vrstvy Berlína: území mapy už není poblíž Berlína
progress-berlin-download = stahuji struktury ALKIS Berlína, katastr stromů, mapování ulic a bodové vrstvy
progress-berlin-features = Berlín { $file }, objektů: { $count }
warning-berlin-layer-kept = Berlín { $file } není k dispozici ({ $detail }); ponechávám uloženou vrstvu
warning-berlin-layer-skipped = Berlín { $file } není k dispozici ({ $detail }); vrstva přeskočena
progress-feinkartierung-none-stored = není uloženo žádné Feinkartierung Hamburku; pro zábradlí, zdi, schody a uliční mobiliář spusťte online `freemapper fetch`
progress-feinkartierung-removing = odstraňuji uložené Feinkartierung Hamburku: území mapy už není v Hamburku
progress-feinkartierung-download = stahuji Feinkartierung Hamburku (uliční prostor a veřejná zeleň)
progress-feinkartierung-features = Feinkartierung { $file }, objektů: { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file } není k dispozici ({ $detail }); ponechávám uloženou vrstvu
warning-feinkartierung-layer-skipped = Feinkartierung { $file } není k dispozici ({ $detail }); vrstva přeskočena
progress-bdot-roads-none-stored = nejsou uloženy žádné silnice a cesty BDOT10k; pro silnice a cesty z databáze spusťte online `freemapper fetch`
progress-bdot-roads-removing = odstraňuji uložené silnice a cesty BDOT10k: území mapy už není v Polsku
progress-bdot-roads-download = stahuji silnice a cesty BDOT10k
progress-bdot-roads-features = BDOT10k { $file }, objektů: { $count }
warning-bdot-roads-layer-kept = BDOT10k { $file } není k dispozici ({ $detail }); ponechávám uloženou vrstvu
warning-bdot-roads-layer-skipped = BDOT10k { $file } není k dispozici ({ $detail }); vrstva přeskočena
error-extra-layer-unknown = není vrstvou složky { $folder }
warning-berlin-buildings-missing = pro tuto oblast nejsou v Berlíně uloženy žádné budovy ALKIS; získáte je z Geoportal Berlin příkazem `freemapper fetch`

## The full ALKIS layers (NAS)

progress-nas-none-stored = nejsou uloženy plné struktury ALKIS; pro zdi, ploty, živé ploty a mosty spusťte online `freemapper fetch`
progress-nas-removing = odstraňuji uložené plné vrstvy ALKIS: území mapy už není ve spolkové zemi se službou NAS
warning-nas-folder-outside = plné vrstvy ALKIS přeskočeny ({ $folder } vede ven ze složky projektu)
progress-nas-download = stahuji plné struktury ALKIS (NAS)
progress-nas-features = ALKIS NAS { $file }, objektů: { $count }
progress-nas-skipped = ALKIS NAS { $file }: přeskočeno objektů ({ $count }): { $list }
warning-nas-layer-kept = ALKIS NAS { $file } není k dispozici ({ $detail }); ponechávám uloženou vrstvu
warning-nas-layer-skipped = ALKIS NAS { $file } není k dispozici ({ $detail }); vrstva přeskočena

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nejsou uloženy žádné objekty ATKIS Basis-DLM; pro prameny, jeskyně, věže, památníky a fontány spusťte online `freemapper fetch`
progress-dlm-removing = odstraňuji uložené vrstvy ATKIS Basis-DLM: území mapy už není ve spolkové zemi, která je poskytuje
warning-dlm-folder-outside = vrstvy ATKIS Basis-DLM přeskočeny ({ $folder } vede ven ze složky projektu)
progress-dlm-download = stahuji objekty ATKIS Basis-DLM
progress-dlm-features = ATKIS Basis-DLM { $file }, objektů: { $count }
progress-dlm-skipped = ATKIS Basis-DLM { $file }: přeskočeno objektů ({ $count }): { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } není k dispozici ({ $detail }); ponechávám uloženou vrstvu
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } není k dispozici ({ $detail }); vrstva přeskočena

## Tree cadastres

progress-trees-none-stored = není uložen žádný městský katastr stromů; pro jednotlivé stromy spusťte online `freemapper fetch`
progress-trees-removing = odstraňuji uložené katastry stromů: území mapy už není ve městě, které ho má
warning-trees-folder-outside = městské katastry stromů přeskočeny ({ $folder } vede ven ze složky projektu)
progress-trees-download = stahuji městské katastry stromů
progress-trees-count = { $id }, stromů: { $count }
progress-trees-skipped = { $id }: přeskočeno stromů ({ $count }): { $list }
warning-trees-kept = katastr stromů { $id } není k dispozici ({ $detail }); ponechávám uložený
warning-trees-skipped = katastr stromů { $id } není k dispozici ({ $detail }); stromy přeskočeny

## Deriving the templates

progress-writing = zapisuji { $path }
progress-contours-steep = vrstevnice po { $interval } m: terén je tak strmý, že by se vrstevnice po { $default } m slévaly; interval vrstevnic { $default } m v projektu je zachová (`--contour-interval { $default }`)
warning-osm-only-no-terrain = pro tuto oblast zatím nejsou data o terénu: mapa vzniká jen z OpenStreetMap, bez vrstevnic, podkladů a budov
progress-no-ndsm-recorded = není zaznamenáno žádné nDSM; pro podklad nDSM spusťte `freemapper fetch`
warning-ndsm-tile-invalid = neplatná dlaždice nDSM { $tile } ve freemapper.toml; přeskočeno
warning-ndsm-not-cached = nDSM { $tile } není v mezipaměti; spusťte `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } nelze přečíst ({ $error }); smažte { $path } a spusťte `freemapper fetch`
warning-surface-unusable = model povrchu je nepoužitelný ({ $error }); spusťte `freemapper fetch`

## Downloads and the cache

progress-downloading = stahuji { $url }
progress-retry-wait = služba je vytížená; další pokus za { $seconds } s
progress-downloaded = staženo { $megabytes } MB
progress-downloaded-of = staženo { $megabytes } z { $total } MB
progress-tile-index-download = stahuji index dlaždic produktu { $product }
progress-tile-cached = v mezipaměti: { $tile }
progress-surface-blocks = čtu { $count } z { $total } bloků dlaždice { $tile } modelu povrchu ({ $megabytes } MB)
progress-surface-whole = dlaždice { $tile } modelu povrchu se stahuje celá: { $reason }
warning-tile-cached-older = používám dlaždici { $tile } produktu { $product } z mezipaměti ze dne { $published } (novější { $newer } nestažena)
warning-ndsm-no-cached = pro dlaždici { $tile } není v mezipaměti žádné nDSM
warning-ndsm-tile-unavailable-cached = dlaždice nDSM { $tile } není k dispozici ({ $reason }); používám soubor z mezipaměti
warning-ndsm-tile-unavailable-skipped = dlaždice nDSM { $tile } není k dispozici ({ $reason }); přeskočeno
warning-file-check-failed = nepodařilo se zjistit, zda je { $file } stále aktuální ({ $reason }); používám soubor z mezipaměti a ostatní soubory tohoto serveru z mezipaměti bez dotazu
warning-ldbv-no-buildings-file = na serveru není soubor s budovami pro dlaždici { $tile }; na mapě tam mohou chybět budovy
progress-ndsm-not-published = pro dlaždici { $tile } není zveřejněno žádné mračno bodů nDSM
progress-ndsm-cached = nDSM { $tile } v mezipaměti
progress-point-cloud-not-published = pro dlaždici { $tile } není zveřejněno žádné laserové mračno bodů
progress-point-cloud-cached = mračno bodů { $tile } v mezipaměti
warning-point-cloud-no-cached = pro dlaždici { $tile } není v mezipaměti žádné mračno bodů; tam žádní kandidáti vegetace
warning-point-cloud-unavailable-cached = mračno bodů { $tile } není k dispozici ({ $reason }); používám soubor z mezipaměti
warning-point-cloud-unavailable-skipped = mračno bodů { $tile } není k dispozici ({ $reason }); tam žádní kandidáti vegetace
warning-point-clouds-unavailable = laserová mračna bodů nejsou k dispozici ({ $reason }); kandidáti vegetace použijí dlaždice z mezipaměti, pokud nějaké jsou
progress-no-point-clouds = pro vegetaci zde zatím nejsou laserová mračna bodů: spolková země žádná nezveřejňuje, kterou by freemapper uměl číst, nebo ne v souřadnicovém systému mapy
warning-point-clouds-not-fetched = vegetace je zvolena, ale nejsou načtena žádná mračna bodů; načtete je příkazem `freemapper fetch`
warning-point-cloud-unreadable = mračno bodů { $tile } nelze přečíst ({ $error }); smažte { $path } a spusťte `freemapper fetch`
warning-osm-nrw-nearest = do 100 m od oblasti není žádný výřez okresu Severního Porýní-Vestfálska; čtu nejbližší, { $district }
warning-osm-disk-full-cached = disk je plný, proto freemapper používá výřez OSM z mezipaměti ze dne { $date }. Uvolněte místo a spusťte `freemapper fetch` znovu, abyste dostali novější
warning-osm-geofabrik-unavailable = Geofabrik není dostupný ({ $error }); používám výřez OSM z mezipaměti ze dne { $date }
progress-osm-extract-cached = výřez OSM { $region } ze dne { $date } v mezipaměti
progress-osm-extract-download = stahuji { $url } (asi { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = stahuji dlaždici bDOM { $tile } ({ $megabytes } MB, { $index } z { $total })
progress-sh-dgm-not-in-atom = dlaždice DGM1 { $tile } není v adresáři INSPIRE ATOM; stahuji její XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = dlaždici DGM1 { $tile } z adresáře INSPIRE ATOM nelze přečíst ({ $error }); stahuji její XYZ ({ $megabytes } MB)
progress-sh-tile = dlaždice { $tile }: { $text }
progress-sh-download-again = dlaždice { $tile }: stažený soubor nelze přečíst ({ $message }); stahuji ji znovu
warning-sh-portal-no-date = Downloadportal neuvádí datum pro dlaždici { $tile } ({ $model })
warning-sh-dgm-xyz-used = dlaždice DGM1 { $tile } není v adresáři INSPIRE ATOM; použit její XYZ z Downloadportalu
warning-sh-dgm-xyz-used-unreadable = dlaždici DGM1 { $tile } z adresáře INSPIRE ATOM nelze přečíst ({ $error }); použit její XYZ z Downloadportalu

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, dlaždice { $tile }: buněk území mapy bez výšky: { $count }
warning-centre-tile-missing = dlaždice { $tile } ({ $model }) ve středu území mapy chybí, proto není datum
warning-centre-tile-unlisted = seznam dlaždic neuvádí dlaždici { $tile } ({ $model }) ve středu území mapy, proto není datum
warning-list-unreadable-kept = seznam dlaždic ({ $model }) nelze přečíst ({ $error }); použit seznam uchovaný z dřívějšího běhu
warning-portal-list-unreadable-kept = seznam dlaždic Downloadportalu ({ $model }) nelze přečíst ({ $error }); použit seznam uchovaný z dřívějšího běhu
warning-surface-much-older = { $model } je z { $surface }, ortofoto z { $flight }. V nDSM mohou chybět novější budovy a stromy.

## Rhineland-Palatinate

warning-rp-list-features-unusable = prvků seznamu dlaždic bez použitelného klíče dlaždice: { $count }; přeskočeno
warning-rp-list-unreadable-kept = seznam dlaždic LVermGeo RP nelze přečíst ({ $error }); použit seznam uchovaný z dřívějšího běhu
warning-rp-metalink-not-kept = Metalink LVermGeo RP nebyl uchován: { $error }
warning-rp-metalink-unreadable-kept = Metalink LVermGeo RP pro { $model } nelze přečíst ({ $detail }); použit Metalink uchovaný z dřívějšího běhu
warning-rp-metalink-unreadable = Metalink LVermGeo RP pro { $model } nelze přečíst ({ $detail }), proto zůstanou bez výšek dlaždice, u nichž jejich seznam dlaždic neuvádí rok snímkování
warning-rp-tile-without-name = { $model }, dlaždice { $tile }: tento čtverec 1 km nemá v mapě žádné výšky. Seznam dlaždic LVermGeo RP (Laserbefliegung) pro něj { $note }, proto není znám název souboru{ $either ->
        [metalink] , a ani jeho Metalink pro něj žádný soubor neuvádí
       *[none] {""}
    }
warning-rp-centre-no-date = dlaždice { $tile } ({ $model }) nemá datum: seznam dlaždic LVermGeo RP (Laserbefliegung) pro ni { $note }
word-rp-list-neither = neuvádí ani rok snímkování, ani dny snímkování
word-rp-list-no-days = neuvádí dny snímkování
word-rp-list-no-year = neuvádí rok snímkování
word-rp-list-days-unreadable = uvádí dny snímkování { $days }, které nelze přečíst
word-rp-list-disagree = uvádí dny snímkování { $days } a rok snímkování { $year }, které se na jednom roce neshodují

## Hesse

warning-hvbg-undated = seznam metadat nemá čitelné Bildflugdatum pro dlaždice { $tiles }; jejich záznamy v projektu nemají datum snímkování
warning-hvbg-undated-more = seznam metadat nemá čitelné Bildflugdatum pro dlaždice { $tiles } a dalších { $more }; jejich záznamy v projektu nemají datum snímkování
warning-hvbg-list-unreadable-older = nový seznam metadat HVBG { $name } nelze přečíst ({ $detail }); používám starší seznam { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: bez data, STAC neuvádí žádnou dlaždici ve středu území mapy
warning-lgln-no-date = { $model }: bez data ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; uložený model si ponechává zaznamenané datum
warning-mv-cells-empty = { $model }: buněk bez výšky uvnitř dlaždic Meklenburska-Předního Pomořanska: { $count }
warning-mv-centre-no-date = { $model }: bez data pro střední dlaždici { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = řádků seznamu dlaždic { $model } bez použitelného názvu dlaždice: { $count }; přeskočeno
warning-nrw-list-rows-undated = řádků seznamu dlaždic { $model } bez čitelného data: { $count }
warning-nrw-list-older = seznam dlaždic { $model } na serveru je z { $server }, tedy starší než uchovaný z { $kept }; použit uchovaný
warning-nrw-list-no-date = seznam dlaždic neuvádí datum pro dlaždici { $tile } ({ $model })

## After `init`

note-scale-outside-norm = poznámka: 1:5000 není v normě pro sprint; freemapper-print na to upozorní a symboly zůstanou na 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } není ID OSM jako way/992423119
error-place-invalid-answer = neplatná odpověď vyhledávání místa: { $detail }
error-place-not-found = { $id } nenalezeno. Nominatim uvádí jen označené objekty OSM, takže neoznačená vnější cesta multipolygonu v něm není. Použijte místo ní relaci multipolygonu (--osm relation/<id>) nebo uložte obrys jako GeoJSON a použijte --geojson
error-place-is-a-point = { $id } je bod, ne plocha; vyberte jeho areál (cestu nebo relaci)
error-place-no-such-part = { $id } má { $count ->
        [one] 1 část
        [few] { $count } části
        [many] { $count } části
       *[other] { $count } částí
    }; část { $part } neexistuje
error-place-several-parts = { $id } je plocha o { $count } samostatných částech ({ $sizes }). Území mapy je jeden polygon: vyberte část pomocí --part <n> (1 je největší) nebo uložte požadovaný obrys jako GeoJSON a použijte --geojson
error-layer-not-a-world-file = { $path }: není world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = pro { $layer } není náhled; vrstvy s náhledem: { $names }
error-layer-no-ndsm-published = pro dlaždice tohoto území mapy není zveřejněno žádné nDSM, takže neexistuje podklad ndsm
error-layer-not-made = { $layer } ještě nevzniklo; spusťte { $step ->
        [fetch] freemapper fetch nebo run (MCP: start_pipeline); jen Berlín
        [export] freemapper export nebo run (MCP: start_pipeline)
       *[derive] freemapper derive nebo run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; mapa OCAD z toho nedostane žádné pozadí
warning-map-preview-failed = mapa je zapsána, ale chybí její náhled: { $error }
warning-training-no-tree-cover = tréninková mapa zachovává otevřenou krajinu parků vcelku: chybí pokryv stromů (data/tree-cover.tif); spusťte `freemapper derive` znovu
warning-ocad-no-paper-frames = výchozí projekt OCAD zatím nemá papírové rámy; jsou v templates/paper-frame.omap, podkladu pro OOM
warning-paper-frames-left-out = papírové rámy vynechané v templates/paper-frame.omap: { $count }; pro mapu OOM leží příliš daleko od mapy; přesuňte je na stránce území blíže k mapovému území
