# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = попередження: { $text }

warning-other-zone = територія карти лежить у зоні UTM { $zone } (EPSG:{ $epsg }); freemapper знову завантажує все в цій зоні
warning-other-state = територія карти тепер потрапила в дані іншої федеральної землі; freemapper знову завантажує все
warning-no-orthophoto = для цього регіону немає ортофотоплану; карта не має підкладки з ортофотопланом
warning-terrain-area-uncovered = Для { $percent } % області мапи в { $model } немає плитки; там відсутній рельєф, тому немає горизонталей, відмивки та нахилу
warning-beyond-coverage = частина території карти лежить за межами підтримуваних федеральних земель: за кордоном, у морі або у федеральній землі, яку freemapper ще не підтримує, наприклад у Тюрингії. Там немає даних, тому ортофото може бути білим, а ALKIS не дає ні будівель, ні землекористування
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: масштаб 1:{ $scale } не входить до масштабів для { $kind ->
        [schoolyard] карт шкільного подвір'я
        [forest] лісових карт
       *[sprint] спринтерських карт
    } ({ $allowed }); freemapper-print відхилить карту
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: запис { $key } проігноровано ({ $what ->
        [tree] шари дерев
        [NAS] шари NAS
        [set] додаткові шари
       *[other] шари { $german }
    }): { $reason ->
        [name] це не звичайна назва файлу
       *[unknown] такого шару freemapper не знає
    }

## Disk space

warning-disk-space-unreadable = не вдається прочитати вільне місце на диску для { $path } ({ $error }), тому freemapper його не перевіряє

## Tiles

warning-tile-changed = плитка { $product } { $tile } змінилася в постачальника ({ $old } → { $new })
warning-bdom-tile-unavailable = модель поверхні (bDOM) для плитки { $tile } недоступна ({ $error }); шаблона nDSM там немає
warning-bdom-unavailable = модель поверхні (bDOM) недоступна ({ $error }); шаблона nDSM для { $tiles } немає
progress-ndsm-from-bdom = nDSM плитки { $tile } з моделі поверхні (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] модель рельєфу
        [surface] модель поверхні
       *[orthophoto] ортофото
    } ({ $name })
progress-model-download = завантаження: { $model }
progress-model-read = читання: { $model }
warning-model-unavailable = { $model }: недоступно ({ $error }); шаблона nDSM немає
warning-model-unavailable-kept = { $model }: недоступно ({ $error }); збережено модель поверхні, завантажену { $fetched }
warning-model-tile-unavailable = { $model }, плитка { $tile }: недоступно ({ $error }); шаблона nDSM там немає
warning-archive-passed-over = { $model }: архів не вдалося прочитати ({ $error }); його пропущено, а плитки території карти беруться з інших архівів
warning-archive-directory-kept = { $model }: архів не вдалося прочитати ({ $error }); використано його каталог із попереднього запуску, тому плитку, яку LVGL відтоді замінило, не буде помічено
warning-model-tiles-missing = у { $model } { $state ->
        [the-map-area] на території карти
        [hamburg] у Гамбурзі
        [hesse] у Гессені
        [north-rhine-westphalia] у Північному Рейні-Вестфалії
        [schleswig-holstein] у Шлезвіг-Гольштейні
        [mecklenburg-vorpommern] у Мекленбург-Передній Померанії
        [lower-saxony] у Нижній Саксонії
        [bavaria] у Баварії
       *[other] у { $state }
    } немає { $count ->
        [one] плитки
        [few] плиток
        [many] плиток
       *[other] плиток
    } { $names }; шаблони там залишаються порожніми
warning-no-surface-stored = модель поверхні не збережено; запустіть `freemapper fetch` з мережею, щоб отримати шаблон nDSM
warning-surface-older-than-orthophoto = модель поверхні (bDOM { $surface }) старіша за ортофото ({ $photo }); у nDSM можуть бути відсутні нові будівлі
progress-surface-kept = freemapper зберігає модель поверхні, завантажену { $fetched }, для шаблона nDSM
progress-surface-tiles-kept = freemapper зберігає кешовану модель поверхні для { $count ->
        [one] плитки
        [few] плиток
        [many] плиток
       *[other] плиток
    } { $names } для шаблона nDSM
progress-orthophoto-unchanged = ортофото без змін
progress-orthophoto-unchanged-dated = ортофото без змін ({ $label } { $date })
progress-orthophoto-download = завантаження ортофото з { $service } ({ $provider })
progress-orthophoto-in-leaf = ортофото знято { $date }, дерева були вкриті листям; їхні крони закривають те, що під ними
progress-orthophoto-bare = ортофото знято { $date }, дерева були без листя
warning-lgln-orthophoto-kept = ортофото (LGLN WMS): { $reason }; збережено старе ортофото та дату його зйомки { $date }
warning-lgln-orthophoto-undated = ортофото (LGLN WMS): немає дати зйомки, { $reason }
word-stac-no-image = пошук STAC не знаходить знімка в центрі території карти
word-stac-no-answer = пошук STAC не дав придатної відповіді ({ $error })
warning-downloadcenter-no-file = у Downloadcenter немає файлу для { $places }
warning-downloadcenter-no-file-empty = у Downloadcenter немає файлу для { $places }; шаблони для { $count ->
        [one] плитки
        [few] плиток
        [many] плиток
       *[other] плиток
    } { $names } залишилися порожніми
progress-k-five-unchanged = шаблон K5 без змін
progress-k-five-download = завантаження карти K5 (1:5000) із WMS Geoportal Berlin
warning-k-five-unavailable = шаблон карти K5 недоступний ({ $detail }); пропущено

## The state border

progress-beyond-border = територія карти виходить за межі кордону федеральної землі; за ним шаблони порожні всюди, де закінчуються ортофото та моделі висот { $provider }. ALKIS і OSM беруться з обох федеральних земель
progress-beyond-border-no-data = територія карти виходить за межі кордону федеральної землі; даних { $provider } для { $tiles } немає, тому шаблони там залишаються порожніми

## OSM

warning-osm-extract-missing-kept = кешованого витягу OSM немає; зберігаються наявні кандидати OSM від { $date }
progress-osm-extract-read = читання витягу OSM { $region } від { $date }
progress-osm-candidates = об'єктів-кандидатів OSM: { $count }

## Permit notes

progress-permits-fetch = завантаження даних { $source } для приміток щодо дозволів
progress-permits-osm = читання витягу OSM { $region } від { $date } для приміток щодо дозволів
progress-permits-stored = читання збережених даних для приміток щодо дозволів
warning-permits-source-unreachable = не вдалося прочитати { $source }, тому примітки щодо дозволів його пропускають: { $error }
warning-permits-osm-no-extract = для цієї території карти немає кешованого витягу OSM, тому примітки щодо дозволів пропускають OSM; `freemapper fetch` його завантажує

## ALKIS

warning-alkis-coverage-unknown = покриття будівлями { $cadastre } невідоме; запустіть `freemapper fetch` з мережею, щоб його записати
progress-alkis-buildings-download = завантаження будівель { $cadastre }
progress-alkis-buildings = будівель { $cadastre }: { $count }
warning-alkis-no-buildings = на території карти не знайдено будівель { $cadastre }
progress-alkis-skipped = пропущено об'єктів { $cadastre }: { $count }: { $list }
warning-alkis-buildings-changed = будівлі { $cadastre } змінилися в постачальника ({ $changes })
progress-alkis-no-land-use-stored = землекористування { $cadastre } не збережено; запустіть `freemapper fetch` з мережею, щоб отримати кандидатів заборонених зон
progress-alkis-land-use-download = завантаження землекористування { $cadastre }
progress-alkis-land-use-parts = ділянок землекористування { $cadastre }: { $count }
warning-alkis-land-use-kept = землекористування { $cadastre } недоступне ({ $error }); зберігається наявне землекористування
warning-alkis-land-use-removed = землекористування { $cadastre } недоступне ({ $error }); збережене землекористування більше не покриває територію карти, тому freemapper його видалив
warning-alkis-land-use-none = землекористування { $cadastre } недоступне ({ $error }); кандидатів заборонених зон немає
progress-bdot-filled = додано { $count } об'єктів BDOT10k ({ $what }) там, де EGiB нічого не дає ({ $powiat })
warning-bdot-failed = BDOT10k не зміг заповнити прогалину в EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB недоступний ({ $error }); замість нього використано BDOT10k ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = споруд ALKIS Берліна не збережено; запустіть `freemapper fetch` з мережею, щоб отримати сходи, стіни, дерева та навіси
progress-berlin-removing = видалення збережених шарів Берліна: територія карти вже не поблизу Берліна
progress-berlin-download = завантаження споруд ALKIS Берліна, кадастру дерев, зйомки вулиць і точкових шарів
progress-berlin-features = об'єктів Берліна { $file }: { $count }
warning-berlin-layer-kept = Берлін, { $file }: недоступно ({ $detail }); зберігається наявний шар
warning-berlin-layer-skipped = Берлін, { $file }: недоступно ({ $detail }); шар пропущено
progress-feinkartierung-none-stored = Feinkartierung Гамбурга не збережено; запустіть `freemapper fetch` з мережею, щоб отримати огорожі, стіни, сходи та вуличні меблі
progress-feinkartierung-removing = видалення збереженої Feinkartierung Гамбурга: територія карти вже не в Гамбурзі
progress-feinkartierung-download = завантаження Feinkartierung Гамбурга (вуличний простір і громадська зелень)
progress-feinkartierung-features = об'єктів Feinkartierung { $file }: { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file }: недоступно ({ $detail }); зберігається наявний шар
warning-feinkartierung-layer-skipped = Feinkartierung { $file }: недоступно ({ $detail }); шар пропущено
progress-bdot-roads-none-stored = збережених доріг і стежок BDOT10k немає; запустіть `freemapper fetch` онлайн, щоб отримати дороги й стежки з реєстру
progress-bdot-roads-removing = видаляються збережені дороги й стежки BDOT10k: область карти більше не лежить у Польщі
progress-bdot-roads-download = завантаження доріг і стежок BDOT10k
progress-bdot-roads-features = об'єктів BDOT10k { $file }: { $count }
warning-bdot-roads-layer-kept = BDOT10k { $file } недоступний ({ $detail }); збережений шар лишається
warning-bdot-roads-layer-skipped = BDOT10k { $file } недоступний ({ $detail }); шар пропущено
error-extra-layer-unknown = це не шар { $folder }
warning-berlin-buildings-missing = для цієї території в Берліні не збережено будівель ALKIS; запустіть `freemapper fetch`, щоб отримати їх із Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = повних споруд ALKIS не збережено; запустіть `freemapper fetch` з мережею, щоб отримати стіни, огорожі, живоплоти та мости
progress-nas-removing = видалення збережених повних шарів ALKIS: територія карти вже не у федеральній землі зі службою NAS
warning-nas-folder-outside = повні шари ALKIS пропущено ({ $folder } виходить за межі папки проєкту)
progress-nas-download = завантаження повних споруд ALKIS (NAS)
progress-nas-features = об'єктів ALKIS NAS { $file }: { $count }
progress-nas-skipped = пропущено об'єктів ALKIS NAS { $file }: { $count }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file }: недоступно ({ $detail }); зберігається наявний шар
warning-nas-layer-skipped = ALKIS NAS { $file }: недоступно ({ $detail }); шар пропущено

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = об'єктів ATKIS Basis-DLM не збережено; запустіть `freemapper fetch` з мережею, щоб отримати джерела, печери, вежі, пам'ятки та фонтани
progress-dlm-removing = видалення збережених шарів ATKIS Basis-DLM: територія карти вже не у федеральній землі, яка їх надає
warning-dlm-folder-outside = шари ATKIS Basis-DLM пропущено ({ $folder } виходить за межі папки проєкту)
progress-dlm-download = завантаження об'єктів ATKIS Basis-DLM
progress-dlm-features = об'єктів ATKIS Basis-DLM { $file }: { $count }
progress-dlm-skipped = пропущено об'єктів ATKIS Basis-DLM { $file }: { $count }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file }: недоступно ({ $detail }); зберігається наявний шар
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file }: недоступно ({ $detail }); шар пропущено

## Tree cadastres

progress-trees-none-stored = міського кадастру дерев не збережено; запустіть `freemapper fetch` з мережею, щоб отримати окремі дерева
progress-trees-removing = видалення збережених кадастрів дерев: територія карти вже не в місті, яке його має
warning-trees-folder-outside = міські кадастри дерев пропущено ({ $folder } виходить за межі папки проєкту)
progress-trees-download = завантаження міських кадастрів дерев
progress-trees-count = дерев { $id }: { $count }
progress-trees-skipped = пропущено дерев { $id }: { $count }: { $list }
warning-trees-kept = кадастр дерев { $id } недоступний ({ $detail }); зберігається наявний
warning-trees-skipped = кадастр дерев { $id } недоступний ({ $detail }); дерева пропущено

## Deriving the templates

progress-writing = запис { $path }
progress-contours-steep = горизонталі через { $interval } м: рельєф такий крутий, що горизонталі через { $default } м зливалися б; висота перерізу { $default } м у проєкті їх зберігає (`--contour-interval { $default }`)
warning-osm-only-no-terrain = для цього регіону ще немає даних про рельєф: карта створюється лише з OpenStreetMap, без горизонталей, шаблонів і будівель
progress-no-ndsm-recorded = nDSM не записано; запустіть `freemapper fetch`, щоб отримати шаблон nDSM
warning-ndsm-tile-invalid = недійсна плитка nDSM { $tile } у freemapper.toml; пропущено
warning-ndsm-not-cached = nDSM { $tile } немає в кеші; запустіть `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } не читається ({ $error }); видаліть { $path } і запустіть `freemapper fetch`
warning-surface-unusable = модель поверхні непридатна ({ $error }); запустіть `freemapper fetch`

## Downloads and the cache

progress-downloading = завантаження { $url }
progress-retry-wait = сервіс зайнятий; наступна спроба через { $seconds } с
progress-downloaded = завантажено { $megabytes } МБ
progress-downloaded-of = завантажено { $megabytes } МБ із { $total } МБ
progress-tile-index-download = завантаження індексу плиток { $product }
progress-tile-cached = у кеші { $tile }
progress-surface-blocks = читання { $count } із { $total } блоків плитки моделі поверхні { $tile } ({ $megabytes } МБ)
progress-surface-whole = плитку моделі поверхні { $tile } завантажено цілком: { $reason }
warning-tile-cached-older = використано кешовану плитку { $product } { $tile } від { $published } (новішу { $newer } не завантажено)
warning-ndsm-no-cached = для плитки { $tile } немає кешованого nDSM
warning-ndsm-tile-unavailable-cached = плитка nDSM { $tile } недоступна ({ $reason }); використано кешований файл
warning-ndsm-tile-unavailable-skipped = плитка nDSM { $tile } недоступна ({ $reason }); пропущено
warning-file-check-failed = не вдалося перевірити, чи { $file } ще актуальний ({ $reason }); використано кешований файл і решту кешованих файлів цього сервера без запиту
warning-ldbv-no-buildings-file = на сервері немає файлу будівель для плитки { $tile }; на карті там можуть бути відсутні будівлі
progress-ndsm-not-published = для плитки { $tile } не опубліковано хмари точок nDSM
progress-ndsm-cached = nDSM { $tile } у кеші
progress-point-cloud-not-published = для плитки { $tile } не опубліковано лазерної хмари точок
progress-point-cloud-cached = хмара точок { $tile } у кеші
warning-point-cloud-no-cached = для плитки { $tile } немає кешованої хмари точок; кандидатів рослинності там немає
warning-point-cloud-unavailable-cached = хмара точок { $tile } недоступна ({ $reason }); використано кешований файл
warning-point-cloud-unavailable-skipped = хмара точок { $tile } недоступна ({ $reason }); кандидатів рослинності там немає
warning-point-clouds-unavailable = лазерні хмари точок недоступні ({ $reason }); кандидати рослинності використовують кешовані плитки, якщо вони є
progress-no-point-clouds = для рослинності тут поки немає лазерних хмар точок: федеральна земля не публікує таких, які freemapper може прочитати, або вони не в системі координат карти
warning-point-clouds-not-fetched = рослинність вибрано, але хмари точок не завантажено; запустіть `freemapper fetch`, щоб їх завантажити
warning-point-cloud-unreadable = хмара точок { $tile } не читається ({ $error }); видаліть { $path } і запустіть `freemapper fetch`
warning-osm-nrw-nearest = у межах 100 м від території немає витягу округу Північного Рейну-Вестфалії; читається найближчий, { $district }
warning-osm-disk-full-cached = диск заповнений, тому freemapper використовує кешований витяг OSM від { $date }. Звільніть місце та знову запустіть `freemapper fetch`, щоб отримати новіший
warning-osm-geofabrik-unavailable = Geofabrik недоступний ({ $error }); використано кешований витяг OSM від { $date }
progress-osm-extract-cached = витяг OSM { $region } від { $date } у кеші
progress-osm-extract-download = завантаження { $url } (приблизно { $megabytes } МБ)

## Schleswig-Holstein

progress-sh-bdom-download = завантаження плитки bDOM { $tile } ({ $megabytes } МБ, { $index } із { $total })
progress-sh-dgm-not-in-atom = плитки DGM1 { $tile } немає в каталозі INSPIRE ATOM; завантажується її XYZ ({ $megabytes } МБ)
progress-sh-dgm-atom-unreadable = плитка DGM1 { $tile } з каталогу INSPIRE ATOM не читається ({ $error }); завантажується її XYZ ({ $megabytes } МБ)
progress-sh-tile = плитка { $tile }: { $text }
progress-sh-download-again = плитка { $tile }: завантажене не читається ({ $message }); завантажується ще раз
warning-sh-portal-no-date = Downloadportal не дає дати для плитки { $model } { $tile }
warning-sh-dgm-xyz-used = плитки DGM1 { $tile } немає в каталозі INSPIRE ATOM; використано її XYZ із Downloadportal
warning-sh-dgm-xyz-used-unreadable = плитка DGM1 { $tile } з каталогу INSPIRE ATOM не читається ({ $error }); використано її XYZ із Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, плитка { $tile }: клітинок території карти без висоти: { $count }
warning-centre-tile-missing = плитки { $model } { $tile } у центрі території карти немає, тому дати немає
warning-centre-tile-unlisted = у списку плиток немає плитки { $model } { $tile } у центрі території карти, тому дати немає
warning-list-unreadable-kept = список плиток { $model } не вдалося прочитати ({ $error }); використано збережений із попереднього запуску
warning-portal-list-unreadable-kept = список плиток { $model } на Downloadportal не вдалося прочитати ({ $error }); використано збережений із попереднього запуску
warning-surface-much-older = { $model } датується { $surface }, ортофото { $flight }. У nDSM можуть бути відсутні новіші будівлі та дерева.

## Rhineland-Palatinate

warning-rp-list-features-unusable = об'єктів списку плиток без придатного ключа плитки: { $count }; пропущено
warning-rp-list-unreadable-kept = список плиток LVermGeo RP не вдалося прочитати ({ $error }); використано збережений із попереднього запуску
warning-rp-metalink-not-kept = Metalink LVermGeo RP не збережено: { $error }
warning-rp-metalink-unreadable-kept = Metalink LVermGeo RP для { $model } не вдалося прочитати ({ $detail }); використано збережений із попереднього запуску
warning-rp-metalink-unreadable = Metalink LVermGeo RP для { $model } не вдалося прочитати ({ $detail }), тому плитки, для яких список плиток не дає року польоту, залишаються без висот
warning-rp-tile-without-name = { $model }, плитка { $tile }: цей квадрат 1 км на карті без висот. Список плиток LVermGeo RP (Laserbefliegung) для нього { $note }, тому назва файлу невідома{ $either ->
        [metalink] , і його Metalink також не називає для нього файлу
       *[none] {""}
    }
warning-rp-centre-no-date = плитка { $model } { $tile } не має дати: список плиток LVermGeo RP (Laserbefliegung) для неї { $note }
word-rp-list-neither = не дає ні року польоту, ні днів польоту
word-rp-list-no-days = не дає днів польоту
word-rp-list-no-year = не дає року польоту
word-rp-list-days-unreadable = дає дні польоту { $days }, які не читаються
word-rp-list-disagree = дає дні польоту { $days } і рік польоту { $year }, які не збігаються за роком

## Hesse

warning-hvbg-undated = у списку метаданих немає читабельної Bildflugdatum для плитки { $tiles }; їхні записи проєкту не мають дати польоту
warning-hvbg-undated-more = у списку метаданих немає читабельної Bildflugdatum для плитки { $tiles } і ще { $more }; їхні записи проєкту не мають дати польоту
warning-hvbg-list-unreadable-older = новий список метаданих HVBG { $name } не вдалося прочитати ({ $detail }); використано старіший список { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: дати немає, STAC не містить плитки в центрі території карти
warning-lgln-no-date = { $model }: дати немає ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; збережена модель зберігає свою записану дату
warning-mv-cells-empty = { $model }: клітинок у плитках Мекленбург-Передньої Померанії без висоти: { $count }
warning-mv-centre-no-date = { $model }: немає дати для центральної плитки { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = рядків списку плиток { $model } без придатної назви плитки: { $count }; пропущено
warning-nrw-list-rows-undated = рядків списку плиток { $model } без читабельної дати: { $count }
warning-nrw-list-older = список плиток { $model } на сервері датується { $server }, тобто старіший за збережений від { $kept }; використано збережений
warning-nrw-list-no-date = список плиток не дає дати для плитки { $model } { $tile }

## After `init`

note-scale-outside-norm = примітка: 1:5000 немає в нормі для спринту; freemapper-print попереджає про це, а знаки залишаються на 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } не є id OSM на кшталт way/992423119
error-place-invalid-answer = недійсна відповідь пошуку місць: { $detail }
error-place-not-found = { $id } не знайдено. Nominatim показує лише OSM-об'єкти з тегами, тому зовнішнього контуру мультиполігона без тегів там немає. Використайте натомість відношення мультиполігона (--osm relation/<id>) або збережіть контур як GeoJSON і використайте --geojson
error-place-is-a-point = { $id } є точкою, а не площею; виберіть його територію (way або relation)
error-place-no-such-part = { $id } має { $count ->
        [one] { $count } частину
        [few] { $count } частини
        [many] { $count } частин
       *[other] { $count } частини
    }; частини { $part } немає
error-place-several-parts = { $id } є площею з { $count } окремих частин ({ $sizes }). Територія карти це один полігон: виберіть частину через --part <n> (1 найбільша) або збережіть потрібний контур як GeoJSON і використайте --geojson
error-layer-not-a-world-file = { $path }: це не world-файл
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = для { $layer } немає попереднього перегляду; шари з переглядом: { $names }
error-layer-no-ndsm-published = для плиток цієї території карти не опубліковано nDSM, тому шаблона ndsm немає
error-layer-not-made = { $layer } ще не створено; запустіть { $step ->
        [fetch] freemapper fetch або run (MCP: start_pipeline); лише Берлін
        [export] freemapper export або run (MCP: start_pipeline)
       *[derive] freemapper derive або run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; карта OCAD не отримує з цього фону
warning-map-preview-failed = карту записано, але її попереднього перегляду немає: { $error }
warning-training-no-tree-cover = тренувальна карта зберігає відкриті ділянки парків цілими: немає покриву деревами (data/tree-cover.tif); знову запустіть `freemapper derive`
warning-ocad-no-paper-frames = стартовий проєкт OCAD ще не має паперових рамок; вони в templates/paper-frame.omap, шаблоні для OOM
warning-paper-frames-left-out = пропущено паперових рамок у templates/paper-frame.omap: { $count }; для карти OOM вони лежать задалеко від карти; перетягніть їх на сторінці території ближче до області карти
