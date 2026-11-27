# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } не є мовою, для якої в freemapper є тексти; доступні: { $available }

## init

cli-part-needs-osm = --part вибирає частину площі OSM і потребує --osm; з --place спершу знайдіть id і скористайтеся --osm <id> --part <n>
cli-reading-file = читання { $path }
cli-no-area-for-place = для «{ $query }» не знайдено площі. Якщо місце є площею з кількох окремих частин, виберіть одну через --osm <id> --part <n> або скористайтеся --geojson
cli-using-place = Використовується { $name } ({ $id })
cli-created-project = Створено проєкт { $path }

## Cache

cli-no-cache-dir = у цій системі немає папки кешу; вкажіть --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Завантажено { $count } плитку
        [few] Завантажено { $count } плитки
        [many] Завантажено { $count } плиток
       *[other] Завантажено { $count } плитки
    }
cli-wrote = Записано { $path }
cli-kept-map = Залишено { $path } (там може бути ваша робота)
cli-map-lacks-templates = Карта ще не посилається на шаблони із зображеннями. Додайте їх в OOM (Шаблони > Відкрити шаблон…) або замініть карту через --force; стару буде збережено як .omap.bak.
cli-map-lacks-buildings = На карті ще немає будівель ALKIS. Щоб їх додати, замініть карту через --force; стару буде збережено як .omap.bak.
cli-map-lacks-canopies = На карті немає навісів і критих стоянок ALKIS (522), які тепер є об'єктами карти. Щоб їх додати, запустіть із --force; стару карту буде збережено як .omap.bak.
cli-map-light = Базова карта: об'єднано ліній: { $joined }; прихованих або надто малих об'єктів залишено для шарів кандидатів: { $left } (нічого не зсунуто)
cli-map-light-kept = Шари кандидатів тепер також містять { $left } об'єктів або відрізків, які ця версія прибирає з базової карти (позначені freemapper:generalized). Збережена карта їх ще має, тому вони показані двічі. Замініть карту через --force, щоб її очистити (стара залишиться як .omap.bak).
cli-map-old-candidates = Карта ще називає templates/candidates.omap; цей запуск заповнив його всіма кандидатами. Кандидати тепер також є в семи шарах, по одному файлу на кожен (templates/candidates-*.omap). Карта перелічує їх після заміни через --force (стара залишиться як .omap.bak), або додайте їх самі в OOM (Шаблони > Відкрити шаблон…).

## PDF

cli-pdf-without-credits = PDF без вказання джерел: ODbL і dl-de/by-2-0 вимагають називати джерела, коли ви передаєте карту далі; див. ATTRIBUTION.txt
cli-wrote-pdf = Записано { $path } (карта без кандидатів)
cli-wrote-pdf-candidates = Записано { $path } (контрольний друк: карта з усіма кандидатами, точно за положенням; перевірте їх на місцевості)
cli-wrote-training = Записано { $map } і { $pdf } (тренувальна карта: об'єднано { $joined }, зсунуто { $shifted }, пропущено або обрізано { $left })
cli-training-free-ends = Кінці ліній залишено вільними: { $ways } там, де шлях зустрічав інший шлях, { $others } біля інших ліній
cli-training-apart-left = на тренувальній карті ще є { $length } мм (на папері), де об'єкти мають замалий проміжок; перевірте її в OOM
cli-kept-training = Залишено тренувальну карту { $path }, щоб запуск не перезаписав карту, яку вже надрукували або передали; видаліть її або передайте --force, щоб оновити

## OCAD

cli-ocad-kept = Залишено { $path } (там може бути ваша робота; --force замінює його й зберігає старий як .ocd.bak)
cli-ocad-old-candidates = Карта OCAD ще називає { $path } фоновою картою; цей запуск заповнив її всіма кандидатами. Кандидати тепер також є в семи шарах, по одному файлу на кожен (-candidates-*.ocd). Карта OCAD перелічує їх після заміни через --force (стара залишиться як .ocd.bak), або додайте їх самі в OCAD як фонові карти.
cli-ocad-wrote = { $count ->
        [one] Записано { $path } (1 об'єкт)
        [few] Записано { $path } ({ $count } об'єкти)
        [many] Записано { $path } ({ $count } об'єктів)
       *[other] Записано { $path } ({ $count } об'єкта)
    }
cli-ocad-left-out = Пропущено на карті OCAD: об'єктів { $skipped } (немає знака для OCAD, немає площі або поза областю малювання OCAD ±2 м) і виродкових частин ліній чи отворів { $dropped }.
cli-ocad-load-symbols = В OCAD виберіть Symbol > Load Colors and Symbols From… і вкажіть офіційний набір знаків{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , шкільний набір IOF 2019
       *[other] {" "}ISSprOM 2019
    }; він замінює умовні знаки freemapper за номером.

## select

cli-select-running = Вибір території для { $path }: натисніть «Готово» на сторінці або Ctrl+C, щоб зупинити.
cli-select-no-browser = Не вдалося відкрити браузер ({ $error }); відкрийте адресу вище.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; без площі, не може бути територією карти): { $place }
cli-find-hint = Створіть проєкт командою: freemapper init <dir> --osm { $id }
cli-find-no-area = Жодне з цих місць не має площі, яка могла б бути територією карти. Накресліть територію командою: freemapper select <dir>
cli-find-nothing = для «{ $query }» нічого не знайдено; спробуйте іншу назву або накресліть територію командою: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = попередження: адаптер у { $folder } пропущено: { $finding }
cli-adapter-note = попередження: адаптер у { $folder } завантажується, але: { $finding }
cli-adapter-from-project = попередження: використовується адаптер { $adapter } з папки проєкту; він може звертатися до цих хостів: { $hosts }
cli-adapter-overlaps = попередження: адаптер { $adapter } також охоплює територію { $others }; його об'єкти потрапляють на тамтешні карти
cli-adapter-dir-relative = попередження: FREEMAPPER_ADAPTER_DIR не є абсолютним шляхом ({ $path }); його ігноровано
cli-adapters-no-project = попередження: { $path } не є папкою; адаптерів проєкту не прочитано
cli-adapters-export-unknown = адаптер { $adapter } не описаний жодним маніфестом цієї програми, тож експортувати нічого; `freemapper adapters list` показує ідентифікатори
cli-adapters-export-taken = { $path } існує і не порожня; експортуйте в папку, якої ще немає
cli-adapters-export-done = { $count } файлів адаптера { $adapter } записано в { $path }. У них лише те, що описує маніфест; адрес із коду програми там немає.
cli-adapters-export-next = Змініть адреси в manifest.toml, перевірте папку командою `freemapper adapters check` і перенесіть її в { $path }: з наступного запуску вона замінює вбудований адаптер, із попередженням. Якщо видалити папку, вбудований адаптер повернеться.
cli-adapters-overlaps = перетинається з: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  ревізія { $revision }  перевірено { $checked }  хости: { $hosts }
cli-adapters-source-built-in = вбудований
cli-adapters-source-user = з вашої папки
cli-adapters-source-project = з проєкту
cli-adapters-never-checked = ніколи
cli-adapters-user-dir = Ваші адаптери читаються з { $path }
cli-adapters-no-user-dir = У цій системі немає папки для ваших адаптерів; одну можна вказати через FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper тепер може також звертатися до { $host } для { $name }. Запис у { $file }; `freemapper adapters disallow-host { $adapter } { $host }` його скасовує. Запущена сторінка вибору території або ШІ-асистент читає запис після повторного запуску
cli-allow-host-already = { $host } уже дозволено для { $name }
cli-allow-host-declared = адаптер { $name } сам називає { $host }; дозволяти нічого
cli-allow-host-refused = { $host } не можна дозволити: { $reason }
cli-allow-host-unknown-adapter = вбудованого адаптера { $adapter } немає; вбудовані адаптери: { $adapters }
cli-allow-host-folder-adapter = { $adapter } є адаптером із вашої папки адаптерів; натомість додайте хост у `[net] hosts` його маніфесту
cli-allow-host-no-config = у цій системі немає папки конфігурації для збереження дозволених хостів
cli-allow-host-write = не вдалося записати { $file }: { $detail }
cli-allow-host-unreadable = { $file } не читається ({ $detail }); спершу виправте або видаліть файл; нічого не змінено
cli-disallow-host-removed = freemapper більше не звертається до { $host } для { $name }
cli-disallow-host-absent = { $host } не був дозволений для { $name }; скасовувати нічого
cli-allowed-hosts-unreadable = попередження: { $file } не вдалося прочитати ({ $detail }); додаткових хостів не дозволено
cli-allowed-hosts-ignored = попередження: { $file }: запис { $host } для { $adapter } ігноровано: { $reason }
cli-allowed-hosts-no-adapter = у freemapper немає вбудованого адаптера з такою назвою
cli-allowed-host-redundant = попередження: { $host } тепер є одним із хостів, які називає адаптер { $name }; ваш запис у { $file } видалено
cli-adapters-allowed = дозволено вами: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = адаптер { $adapter } ({ $name }), ревізія { $revision }, хости: { $hosts }
cli-check-error = помилка: { $finding }
cli-check-warning = попередження: { $finding }
cli-check-passed = { $folder } проходить перевірку
cli-check-passed-warnings = { $folder } проходить перевірку, { $count ->
        [one] з одним попередженням
        [few] з { $count } попередженнями
        [many] з { $count } попередженнями
       *[other] з { $count } попередженнями
    }
cli-check-failed = { $folder } не проходить перевірку
cli-check-no-folder = папки { $folder } немає
cli-check-live-ok = { $capability } у порядку: { $detail }
cli-check-live-failed = { $capability } НЕВДАЛО: { $detail }
cli-check-live-not-asked = { $capability } не запитувалось: { $detail }
cli-check-live-box-outside = рамка не лежить усередині межі адаптера { $adapter }; жива перевірка запитує сервіси про місце, яке адаптер охоплює
cli-check-live-cannot = жива перевірка не може виконатися: { $detail }
cli-check-detail-not-declared = адаптер нічого такого не оголошує
cli-check-detail-stopped = попередня частина не вдалася
cli-check-detail-see-warnings = див. попередження
cli-check-detail-outside-dataset = рамка лежить поза набором даних
cli-check-detail-objects = { $count ->
        [one] один об'єкт
        [few] { $count } об'єкти
        [many] { $count } об'єктів
       *[other] { $count } об'єкта
    }
cli-check-detail-layers = шарів: { $layers }, об'єктів: { $count }
cli-check-detail-trees = { $count ->
        [one] одне дерево
        [few] { $count } дерева
        [many] { $count } дерев
       *[other] { $count } дерева
    }
cli-check-detail-extract = { $label }, дані від { $date }
cli-check-raster-orthophoto = ортофото
cli-check-raster-terrain = модель рельєфу
cli-check-raster-surface = модель поверхні
cli-check-raster-tiles = { $product }: { $count ->
        [one] одна плитка
        [few] { $count } плитки
        [many] { $count } плиток
       *[other] { $count } плитки
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } і { $last }

## Folder lock

cli-lock-held = інший запуск freemapper використовує цю папку (з { $time }, запущено: { $who }). Зачекайте, поки він завершиться, або спершу зупиніть його.
cli-lock-who-cli = термінал
cli-lock-who-mcp = ШІ-агент (MCP)
cli-lock-who-page = сторінка вибору
cli-lock-who-unknown = інша програма
cli-lock-failed = не вдалося заблокувати папку проєкту для цього запуску: { $error }
cli-lock-stuck = на заваді стоїть щось, крім файла блокування, і його не вдалося видалити

## Run routes of the selection page

cli-run-bad-request = Параметри запуску недійсні: { $error }
cli-run-no-project = У цій папці ще немає збереженого проєкту. Спершу збережіть територію.
cli-run-already-running = Для цього проєкту вже триває запуск ({ $job }). Дочекайтеся його завершення або спершу скасуйте його.
cli-run-disk-stops = Недостатньо вільного місця на диску для завантаження: { $messages }
cli-run-busy-elsewhere = Інший проєкт цієї програми зараз виконує конвеєр. Проєкти спільно використовують кеш завантажень, тому одночасно триває лише один запуск. Спробуйте, коли він завершиться.
cli-run-nothing-to-cancel = Жоден запуск не триває, тож скасовувати нічого.
cli-run-unknown-job = Для цього проєкту немає запуску { $job }.
cli-status-too-fast = Запити стану надходять надто часто. Запитуйте не частіше п'яти разів на секунду.
cli-open-folder-failed = Не вдалося відкрити папку ({ $error }).
cli-open-folder-no-opener = На цьому комп'ютері немає програми для відкриття папок ({ $program }). Відкрийте папку вручну.
cli-open-folder-missing = Папки проєкту ще не існує. Спершу збережіть територію.
cli-open-folder-refused = Назву папки небезпечно передавати файловому менеджеру.
cli-save-running = Для цього проєкту триває запуск. Збережіть знову, коли він завершиться.
cli-save-unknown-kind = Невідомий вид карти «{ $kind }». Використайте schoolyard, sprint або forest.
cli-save-scale-too-large = Масштаб { $scale } завеликий.
cli-save-bad-content = Налаштування вмісту недійсні. Нічого не збережено.
cli-map-image-none = Спершу створіть карту.
cli-map-image-gone = Цей попередній перегляд більше недоступний.
cli-map-image-running = Для цього проєкту триває запуск. Попередній перегляд можна обчислити після його завершення.
cli-map-image-stale = Збережений проєкт більше не відповідає створеній карті, наприклад через зміну території, виду карти або масштабу. Спершу запустіть створення, яке замінює карту («Замінити наявну карту» на сторінці або freemapper run --force).
cli-map-image-superseded = Тим часом запитано новіший попередній перегляд, тому цей не обчислювався.
cli-map-image-bad-content = Налаштування вмісту недійсні.
cli-map-image-bad-request = Запит попереднього перегляду недійсний. Надішліть об'єкт JSON, за потреби з налаштуваннями вмісту.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – вибір території
page-heading = Вибери територію карти
page-update-available = Доступний freemapper { $version }. Онови командою
page-update-notes = Примітки до випуску
page-folder = Папка проєкту
page-pick = Знайти місце
page-query-placeholder = напр. Clara-Zetkin-Schule Strausberg
page-search-button = Шукати
page-searching = Пошук …
page-no-results = Нічого не знайдено.
page-point-only = лише точка, не площа
page-draw = Накресли сам
page-rectangle = Прямокутник
page-polygon = Полігон
page-finish-polygon = Замкнути полігон
page-clear = Очистити
page-hint-rectangle = Клацни на карті два протилежні кути.
page-hint-polygon = Клацай кути один за одним. Щоб замкнути, клацни перший кут, двічі клацни або вибери «Замкнути полігон». Esc скасовує, Backspace прибирає останній кут.
page-hint-idle = Вибери місце з результатів пошуку або накресли територію сам.
page-summary = Територія
page-source-osm = З OpenStreetMap: { $name }
page-source-drawn = Накреслено вручну
page-size = Розмір із буфером: { $width } × { $height } м
page-tiles-one = Плитки ({ $n }): { $names }
page-tiles-other = Плитки ({ $n }): { $names }
page-download = Завантаження: приблизно { $mb } МБ
page-download-none = Завантаження: усе вже в кеші
page-download-details = Що завантажується
page-cached = у кеші
page-disk = Місце на диску: { $volumes }
page-disk-volume = вільно { $free }, потрібно приблизно { $needed }
page-cache-volume = кеш
page-project-volume = папка проєкту
page-no-room = не вміщається на диск
page-no-room-skip = Не все вміщається на диск (freemapper залишає { $reserve } вільними). Завантаження пропускає позначені частини. Якщо вони тобі потрібні, спершу звільни місце:
page-no-room-stop = Недостатньо місця на диску (freemapper залишає { $reserve } вільними). Без позначених частин карти не буде, тому завантаження зупиняється, нічого не завантаживши. Спершу звільни місце:
page-disk-unknown = freemapper не може прочитати вільне місце на диску ({ $error }). Він завантажує без перевірки.
page-too-large = Територія завелика: щонайбільше { $max } м на сторону (з буфером). Вибери меншу.
page-outside = Територія лежить за межами підтримуваних федеральних земель ({ $states }); для інших регіонів у freemapper поки немає джерел даних.
page-outside-unknown = Територія лежить за межами регіонів, які підтримує freemapper; джерел даних там поки немає.
page-adapters-failed = Список аерофотознімків недоступний ({ $detail }). Карта показує лише OpenStreetMap; усе інше працює.
page-settings = Карта
page-kind = Вид карти
page-schoolyard = Карта шкільного подвір'я (1:1000)
page-sprint = Спринтерська карта (1:4000)
page-forest = Лісова карта (1:15000)
page-scale = Масштаб 1:
page-map-title = Назва
page-contour-interval = Висота перерізу
page-contour-auto = автоматично
page-steep-slope = Поріг крутизни
page-steep-slope-auto = автоматично (27°)
page-steep-slope-hint = Починаючи з цього кута схилу місцевість вважається крутою під час пошуку укосів та обривів. 27° (автоматично) підходить для пагорбів, парків і міст. У скельних містах, ущелинах і кар'єрах крутим є майже кожен схил, тому укоси й обриви зливаються в одну пляму й жодного не знаходять; 55° до 65° їх розділяє. Занадто високе значення залишає лише найкрутіші стіни.
page-face-slope = Поріг стін
page-face-slope-auto = автоматично (70°)
page-face-slope-hint = Починаючи з цього кута схилу місцевість малюється як чорна площа скельної стіни. 70° (автоматично) підходить для скельних міст на кшталт Адршпаха (5,2 % площі). 65° дає більше й більших чорних площ, зокрема на крутих схилах (8,4 %). 75° залишає лише справжні стіни з рваними краями (2,5 %). Надто високе значення майже не залишає площ.
page-settings-fixed = Проєкт уже існує; вид карти, масштаб і назва містяться у freemapper.toml. Висоту перерізу ще можна змінити.
page-save = Зберегти
page-saving = Збереження …
page-saved = Збережено у freemapper.toml.
page-zone-changed = Збережено. Територія тепер лежить в іншій зоні UTM; freemapper знову завантажить її дані під час наступного запуску.
page-provider-changed = Збережено. Висоти та аерофотознімки нової території надає інший постачальник даних; freemapper знову завантажить її дані під час наступного запуску.
page-save-stale = freemapper.toml змінили в іншому вікні або іншою програмою, відколи ця сторінка його прочитала. Нічого не збережено; сторінка тепер знає збережений стан. Збережи ще раз, щоб замінити його територією, показаною тут.
page-has-map = Наявна карта (.omap) залишається як є; freemapper створює нові шаблони й кандидатів для нової території. Опція «Замінити наявну карту» нижче (або «freemapper run --force») створює й карту заново та зберігає стару як .omap.bak.
page-done = Готово
page-source = Вихідний код (AGPL-3.0-or-later)
page-closed = Готово. Можеш закрити це вікно.
page-next = Далі: freemapper run "{ $dir }"
page-made-files = Створено: { $files }. Папка: { $dir }
page-back-to-agent = Скажи своєму ШІ-асистентові, що територію збережено.
page-nothing-saved = Нічого не збережено.
page-failed = Помилка: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Аерофотознімок { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Що потрапляє на карту
page-content-help = Профіль визначає, що потрапляє прямо на карту, а що залишається кандидатом для перевірки. Вибір для групи сильніший за профіль. Його зберігають разом із територією.
page-content-profile-legend = Профіль
page-content-profile-safe = Обережний
page-content-profile-safe-hint = Кожен об'єкт лишається там, куди його ставить побудова: будівлі й ділянки кадастру потрапляють на карту; стіни, огорожі, дерева та багато чого з OpenStreetMap (ставки, доріжки з невідомим покриттям) залишаються кандидатами.
page-content-profile-balanced = Збалансований
page-content-profile-balanced-hint = Об'єкти кадастру, реєстрів і вуличної зйомки потрапляють на карту разом із горизонталями та зонами, куди заборонено заходити, вздовж залізничних колій. Стіни, огорожі, живоплоти, укоси, сходи, воду, дерева, пам'ятки, проходи, доріжки та залізниці також беруть з OpenStreetMap. Вуличні меблі, інші заборонені зони та форми з моделі рельєфу залишаються кандидатами.
page-content-profile-full = Повний
page-content-profile-full-hint = Усе потрапляє на карту, як у контрольному друку; поза нею залишаються лише приховані кандидати.
page-content-groups-legend = За групами
page-content-groups-help = «Як у профілі» слідує профілю. Що це означає для групи, залежить від того, звідки взято кожен об'єкт.
page-content-place-profile = Як у профілі
page-content-place-map = Карта
page-content-place-candidates = Кандидат
page-content-place-off = Вимкнено
page-content-group-walls = Стіни
page-content-group-fences = Огорожі
page-content-group-hedges = Живоплоти
page-content-group-slopes = Укоси та уступи
page-content-group-stairs = Сходи
page-content-group-trees = Дерева та групи дерев
page-content-group-water-features = Вода, фонтани та джерела
page-content-group-monuments = Вежі, пам'ятки та лінії електропередач
page-content-group-furniture = Вуличні меблі та ліхтарі
page-content-group-passages = Проходи
page-content-group-contours = Горизонталі
page-content-group-landforms = Горби, западини, укоси та уступи з моделі рельєфу
page-content-group-vegetation = Рослинність із лазерних хмар точок (Бранденбург, Берлін, Північний Рейн-Вестфалія, Рейнланд-Пфальц, Саар і Баварія; вимкнено, доки ти її не вибереш; завантажує приблизно від 40 до 600 МБ на км²)
page-content-group-out-of-bounds = Зони, куди заборонено заходити
page-content-group-paths = Доріжки та залізниці
page-content-group-streets = Вулиці та площі
page-run-heading = Створити карту
page-preview-heading = Попередній перегляд карти
page-preview-stored = Як створено востаннє.
page-preview-live = З налаштуваннями на цій сторінці, ще не збереженими.
page-preview-live-saved = Зі збереженими налаштуваннями, ще не створено запуском.
page-preview-computing = Обчислення …
page-preview-none = Карту ще не створено.
page-preview-later = Для цієї карти ще немає попереднього перегляду. Він з'явиться після наступного запуску, що записує карту.
page-preview-stale = Територія, вид карти або масштаб змінилися відтоді, як карту створено. Зображення все ще показує карту, створену востаннє; для нового перегляду потрібен запуск із «Замінити наявну карту».
page-preview-area-unsaved = Територію на цій сторінці ще не збережено. Зображення показує карту, створену востаннє; для нової території потрібен запуск із «Замінити наявну карту».
page-preview-run = Як створено востаннє. Сторінка оновить зображення, коли запуск завершиться.
page-preview-open = Відкрити в повному розмірі
page-preview-alt = Попередній перегляд об'єктів карти
page-opt-ocad = Записати стартовий проєкт OCAD
page-help-ocad = Для всіх, хто продовжує працювати в OCAD.
page-opt-pdf = PDF для друку
page-help-pdf = Лише об'єкти карти, без кандидатів.
page-opt-pdf-candidates = Контрольний друк з усіма кандидатами
page-opt-training = Тренувальна карта
page-help-training = Карта й усі кандидати, приведені до вигляду, придатного для друку як є; з OCAD також як файл OCAD.
page-views-heading = Вигляди тренувальної карти
page-view-no-paths = Без доріжок (Ultimate map)
page-view-brown-only = Лише коричневий (орієнтування за горизонталями)
page-view-no-north-lines = Без ліній півночі
page-view-custom = Інший вигляд
page-help-views = Кожен позначений вигляд це ще один файл поруч із тренувальною картою, `<name>-training-<view>.omap` і його PDF, з OCAD також файл OCAD. Інший вигляд задається як `no:505,506` (пропустити знаки) або `only:101,102` (залишити лише ці); `+` поєднує вигляди. Назва, масштаб і джерела залишаються завжди.
page-help-views-forest = Без доріжок і лише коричневий підходять тільки для лісових карт.
page-opt-scale-bar = Лінійка масштабу на PDF
page-help-scale-bar = Німецькі карти зазвичай її не мають.
page-opt-sources = Рядки джерел на PDF
page-sources-auto = автоматично
page-sources-full = повні
page-sources-credit-only = лише freemapper
page-warn-credit-only = Тоді PDF називають лише freemapper. Ліцензії на дані (ODbL, dl-de/by-2-0) вимагають вказувати авторство, коли ти передаєш карту далі; тоді джерела називаєш сам. ATTRIBUTION.txt перелічує їх усі.
page-need-pdf = Лінійка масштабу та рядки джерел діють лише тоді, коли створюється PDF.
page-opt-offline = Лише кеш, без мережі
page-opt-force = Замінити наявні карти
page-help-force = Карту, а з позначеною «Тренувальною картою» також тренувальну карту. Старі файли залишаються як .omap.bak.
page-opt-permits = Примітки щодо дозволів
page-help-permits = Адміністративні одиниці, природоохоронні території та до кого звертатися, у permits.md. Підказка, а не юридична консультація.
page-run-start = Створити карту
page-run-again = Створити знову
page-run-cancel = Скасувати
page-run-cancelling = Скасування …
page-warnings-heading = Попередження
page-where-files = Файли лежать у папці проєкту:
page-open-folder = Відкрити папку
page-log-label = Журнал запуску
page-force-title = Замінити наявні карти?
page-force-text = Наявну карту буде замінено, а з позначеною «Тренувальною картою» і тренувальну. Твої правки не пропадуть: старі файли залишаються в папці проєкту як .omap.bak. Продовжити?
page-force-cancel = Скасувати
page-force-confirm = Замінити
page-step-fetch = Завантаження даних
page-step-derive = Обчислення шаблонів
page-step-map = Запис карти
page-step-ocad = Запис проєкту OCAD
page-step-pdf = Запис PDF
page-step-pdf-candidates = Запис контрольного друку
page-step-training = Запис тренувальної карти
page-step-permits = Запис приміток щодо дозволів
page-step-line = Крок { $index } із { $total }: { $label }
page-starting = Запуск починається …
page-file-kind-map = карта
page-file-kind-template = шаблон
page-file-kind-ocad = карта OCAD
page-file-kind-ocad-candidates = кандидати OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = контрольний друк
page-file-kind-training-map = тренувальна карта
page-file-kind-training-pdf = PDF тренувальної карти
page-file-kind-training-ocad = тренувальна карта OCAD
page-file-kind-training-view-map = вигляд тренувальної карти
page-file-kind-training-view-pdf = PDF вигляду тренувальної карти
page-file-kind-training-view-ocad = вигляд тренувальної карти OCAD
page-file-kind-preview = попередній перегляд карти
page-file-kind-permits = примітки щодо дозволів
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Готово. Файли записано.
page-result-failed = Запуск не вдався: { $error }
page-result-failed-kept = Те, що записано, залишається в папці проєкту; новий запуск продовжить звідти.
page-result-cancelled = Скасовано. Те, що вже завантажено, залишається в кеші; новий запуск продовжить звідти.
page-result-title-done = Карту створено
page-result-title-failed = Запуск не вдався
page-result-title-cancelled = Запуск скасовано
page-kept-map = Наявну карту залишено як є. «Замінити наявну карту» створює її заново й зберігає стару як .omap.bak.
page-no-files = Файлів не створено.
page-opened-folder = Папку відкрито.
page-open-failed = { $message } Папка: { $dir }
page-why-no-area = Спершу вибери територію.
page-why-checking = Попередній перегляд обчислюється …
page-why-too-large = Територія завелика.
page-why-outside = Територія лежить за межами підтримуваних федеральних земель.
page-why-disk = Недостатньо вільного місця на диску (див. вище).
page-why-running = Триває запуск.
page-err-scale-bar-no-pdf = Лінійці масштабу потрібен PDF.
page-err-sources-no-pdf = Рядкам джерел потрібен PDF.
page-save-failed = Територію не збережено; запуск не розпочато.
page-poll-lost = Сервер не відповідає. freemapper продовжує спроби.
page-run-unknown = Сервер більше не знає цього запуску.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Закрити

## update (spec 3.4 to 3.6)

update-source-build = Цей freemapper зібрано з вихідного коду. Оновіть його командами git pull і cargo build --release --locked.
update-current = freemapper { $version } актуальний.
update-available = Доступний freemapper { $new } (у вас { $old }), випущений { $released }.
update-details =
    Примітки до випуску: { $notes }
    Завантаження: { $size } МБ
update-decimal-separator = ,
update-ask = Встановити зараз? [y/N]
update-not-installed = Нічого не встановлено.
update-needs-yes = введення не є терміналом, тому freemapper не може запитати перед встановленням; запустіть freemapper update --yes, щоб встановити без запиту
update-installing = завантаження й перевірка freemapper { $version }
update-done = freemapper оновлено з { $old } до { $new }. Примітки до випуску: { $notes }
update-mcp-restart = Запущений сервер MCP зберігає стару версію, доки ШІ-асистент його не перезапустить.
update-rolled-back = Повернуто freemapper { $version }. Версію, яку ви залишили, збережено як { $path }.
update-not-writable = не вдається записати в папку програми { $path }, а freemapper не просить прав адміністратора. Розпакуйте там нову версію вручну або перенесіть freemapper у папку, до якої ви маєте доступ на запис.
update-too-old = freemapper { $new } не можна встановити самооновленням версії { $old }; встановіть { $needed } або новішу вручну, див. { $notes }
update-busy = в папці програми вже виконується інше оновлення або відкат (файл блокування { $path }); дочекайтеся його завершення й спробуйте знову
update-bad-signature = випуск не вдалося перевірити ({ $error }); нічого не встановлено
update-check-bad-signature = найновіший випуск freemapper не вдалося перевірити ({ $error }); нічого не встановлено
update-no-manifest = найновіший випуск не може встановити себе сам (у ньому немає маніфесту оновлення); завантажте його зі сторінки { $page }
update-no-manifest-version = випуску { $version } не існує або він не може встановити себе сам (у ньому немає маніфесту оновлення); див. { $page }
update-no-asset = у випуску немає програми для цієї системи ({ $platform }); нічого не встановлено
update-no-previous = немає попередньої версії для повернення ({ $path } відсутній)
update-failed = оновлення не вдалося: { $error }
update-mode-set = Режим оновлення встановлено на { $mode } у { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } перевизначає це налаштування, доки його задано.
update-mode-failed = не вдалося записати { $path }: { $error }
update-notice = Доступний freemapper { $new } (у вас { $old }). Оновіть командою: freemapper update
update-applies-next-start = Нова версія застосується під час наступного запуску freemapper.
update-install-failed = freemapper { $new } не встановлено: { $error }
update-no-config-dir = у цій системі немає папки конфігурації; задайте FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Примітка для людини: під час запуску сервер freemapper виявив таке щодо папок адаптерів. Передайте це далі; людина виправляє папку або видаляє її, а потім перезапускає сервер.
    { $lines }
update-mcp-note = Примітка для людини: доступний freemapper { $new } (зараз { $old }). Запитайте її, перш ніж викликати install_update.
update-mcp-needs-confirm = нічого не встановлено; спершу запитайте людину й викличте install_update з confirm: true, коли вона погодиться
update-mcp-note-manual = Примітка для людини: доступний freemapper { $new } (зараз { $old }), але install_update не може його встановити; встановіть { $needed } або новішу вручну, див. { $notes }
update-mcp-note-bad-signature = Примітка для людини: найновіший випуск freemapper не вдалося перевірити ({ $error }); нічого не встановлено. Повідомте їй про це й не встановлюйте його іншим способом.
update-mcp-restart-now = Перезапустіть сервер MCP freemapper зараз; він ще працює на старій версії.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = помилка
error-cancelled = скасовано
cli-cancelled-kept = скасовано; залишено: { $files }
error-io = { $path }: { $message }
error-io-disk-full = диск заповнений, тому freemapper не зміг записати { $path }. Звільніть місце й спробуйте знову

## Errors: project

error-project-already-exists = проєкт уже існує: { $path }
error-project-map-exists = карта вже існує (там може бути ваша робота); щоб замінити її, використайте --force: { $path }
error-project-invalid-scale = недійсний масштаб 1:{ $scale }
error-project-invalid-contour-interval = недійсна висота перерізу { $interval } м у freemapper.toml; вона має бути більшою за 0
error-project-invalid-steep-slope = недійсний поріг крутизни { $slope }° у freemapper.toml; він має лежати між 15° і 80°
error-project-invalid-face-slope = недійсний поріг стін { $slope }° у freemapper.toml; він має лежати між 55° і 85°
error-project-scale-not-allowed = масштаб 1:{ $scale } не дозволено для карт виду { $kind }; використайте { $allowed }
error-project-scale-not-allowed-enlarge = масштаб 1:{ $scale } не дозволено для карт виду { $kind }; використайте { $allowed }. Малюйте в масштабі 1:4000, freemapper-print збільшує до 1:3000 під час друку
word-kind-schoolyard = шкільне подвір'я
word-kind-sprint = спринт
word-kind-forest = ліс
error-project-toml = недійсний файл проєкту: { $message }
error-project-unknown-adapter = файл проєкту називає адаптер «{ $adapter }», якого цей freemapper не має; його може додати новіший freemapper (`freemapper update`), або він походить із вашої папки адаптерів (`freemapper adapters list` його показує) чи, з --adapters-from-project у командному рядку, з папки `adapters` проєкту (лише для проєкту, якому ви довіряєте)
error-project-not-fetched = плитки ще не завантажено; спершу запустіть `freemapper fetch`
error-project-ocd = експорт OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: зображення та world-файл не відповідають території карти; знову запустіть `freemapper derive`
error-project-tiles-outdated = території карти потрібні плитки { $tiles }, які не були завантажені; запустіть `freemapper fetch`
error-project-buildings-not-fetched = для цієї території карти не збережено будівель ALKIS; спершу запустіть `freemapper fetch` з мережею
error-project-orthophoto-not-fetched = для цієї території карти не збережено ортофото; спершу запустіть `freemapper fetch` з мережею
error-project-terrain-not-fetched = для цієї території карти не збережено моделі рельєфу; спершу запустіть `freemapper fetch` з мережею
error-project-candidates-outdated = збережені кандидати OSM не покривають територію карти; запустіть `freemapper fetch`
error-project-stored = { $path }: { $message }; знову запустіть `freemapper fetch`
error-project-area-too-large = територія карти з буфером становить { $width } × { $height } м, що більше за { $limit } м на сторону; зменште територію карти або buffer_m у freemapper.toml
error-project-area-crosses-country = територія карти лежить по обидва боки державного кордону; карти через кордон ще не підтримуються, тому накресліть територію всередині однієї країни
error-project-permits-not-for-region = примітки щодо дозволів для цього регіону поки недоступні: вони звертаються до німецьких сервісів і називають німецькі органи влади
error-project-outside-coverage = територія карти лежить за межами підтримуваних федеральних земель ({ $states }); джерела даних у freemapper поки є лише для них

## Errors: disk space

error-space-required = недостатньо місця на диску для { $part }. Потрібно { $needed }, але на { $path } вільно лише { $left } понад резерв { $reserve }. Звільніть місце й знову запустіть `freemapper fetch`
error-space-skipped = freemapper пропускає { $part } через брак місця на диску. Потрібно { $needed }, але на { $path } вільно лише { $left } понад резерв { $reserve }. Звільніть місце й знову запустіть `freemapper fetch`
word-part-map-data = дані карти ALKIS і OSM
word-part-terrain = модель рельєфу
word-part-orthophoto = ортофото
word-part-surface = модель поверхні для шаблона nDSM
word-part-osm-update = оновлення кешованого витягу OSM
word-part-k-five = шаблон карти K5 Берліна
word-part-berlin-layers = споруди ALKIS Берліна, кадастр дерев, зйомку вулиць і точкові шари
word-part-state-layers = повні споруди ALKIS, об'єкти ландшафтної моделі та кадастри дерев
word-part-point-cloud = лазерні хмари точок для кандидатів рослинності

## Errors: downloads

error-fetch-http = завантаження { $url } не вдалося: { $message }
error-fetch-http-temporary = завантаження { $url } не вдалося: { $message }. Сервіс, імовірно, перевантажений або на обслуговуванні; freemapper намагався до { $attempts } разів. Запустіть виконання знову через кілька хвилин; уже завантажене зберігається.
error-fetch-corrupt-archive = завантаження { $url } не є повним архівом плиток: { $message }
error-fetch-corrupt-extract = завантаження { $url } не є повним витягом OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] плитка { $tiles } не покрита опублікованими плитками; зменште територію карти або buffer_m у freemapper.toml
        [few] плитки { $tiles } не покриті опублікованими плитками; зменште територію карти або buffer_m у freemapper.toml
        [many] плитки { $tiles } не покриті опублікованими плитками; зменште територію карти або buffer_m у freemapper.toml
       *[other] плитки { $tiles } не покриті опублікованими плитками; зменште територію карти або buffer_m у freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] офлайн і немає в кеші: плитка { $product } { $tiles }; запустіть `freemapper fetch` з мережею
        [few] офлайн і немає в кеші: плитки { $product } { $tiles }; запустіть `freemapper fetch` з мережею
        [many] офлайн і немає в кеші: плитки { $product } { $tiles }; запустіть `freemapper fetch` з мережею
       *[other] офлайн і немає в кеші: плитки { $product } { $tiles }; запустіть `freemapper fetch` з мережею
    }
word-product-orthophoto = ортофото (DOP20)
word-product-dtm = модель рельєфу (DGM1)
word-product-image-dsm = модель поверхні (bDOM)
word-product-saxony-dtm = модель рельєфу (GeoSN DGM1)
word-product-saxony-dsm = модель поверхні (GeoSN DOM1)
error-fetch-osm-extract-missing = кешованого витягу OSM немає; запустіть `freemapper fetch` з мережею
error-fetch-not-found = { $name } не знайдено на сервері
error-fetch-refused = { $url }: сервер відхилив запит (HTTP { $status })
error-fetch-refused-detail = { $url }: сервер відхилив запит (HTTP { $status }): { $detail }
error-fetch-too-large = завантаження { $url } більше за { $limit } МБ; зупинено
error-fetch-not-stored = для цієї території карти не збережено: { $label }: { $path } відсутній, не читається або не покриває її; спершу запустіть `freemapper fetch` з мережею
error-fetch-range-ignored = { $url }: сервер проігнорував діапазон байтів { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper не надсилає запитів на цю адресу: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper не надсилає запитів на цю адресу: адаптер { $name } не називає хоста { $host }{ $from }. Якщо сервіс переїхав туди, дозвольте хост командою `freemapper adapters allow-host { $adapter } { $host }` і повідомте про це на https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper не надсилає запитів на цю адресу: адаптер { $adapter } не називає хоста { $host }{ $from }. Якщо сервіс переїхав туди, додайте хост у `[net] hosts` маніфесту адаптера
error-fetch-host-redirected-from = {" "}({ $url } перенаправляє туди)
error-reason-not-a-host-name = не є назвою хоста
error-reason-not-ascii = назву поза ASCII треба писати у формі xn--
error-reason-not-a-dns-name = не є назвою DNS
error-reason-no-domain = назва цього комп'ютера або без домену
error-reason-local-name = назва локальної мережі
error-reason-ip-address = IP-адреса, а не назва DNS
error-reason-unexpected-character = неочікуваний символ
error-reason-not-https = не https
error-reason-user-info = дані користувача
error-reason-port = порт, відмінний від 443
error-reason-host-not-declared = хост не входить до оголошених хостів
error-reason-local-address = адреса цього комп'ютера або локальної мережі, а не сервера в інтернеті
error-reason-too-many-redirects = понад { $count } перенаправлень
error-reason-redirect-nowhere = перенаправлення (HTTP { $status }) в нікуди
error-fetch-currency-csv = CSV актуальності, рядок { $line }: { $message }
error-alkis = недійсні дані ALKIS: { $message }
error-osm = дані OSM: { $message }

## Errors: templates and rasters

error-derive-zip = архів плиток: { $message }
error-derive-image = зображення: { $message }
error-derive-no-tiles = немає плиток, з яких можна обчислювати
error-derive-area-too-large = територія карти завелика для одного шаблона: { $width } × { $height } пікселів (щонайбільше { $limit } на сторону); зменште територію карти або buffer_m у freemapper.toml
error-raster-world-file = недійсний world-файл: { $message }
error-raster-misaligned = растри не вирівняно за однією сіткою пікселів: { $message }
error-raster-pixel-size = растри мають різні розміри пікселя: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = недійсний GeoJSON: { $message }
error-area-no-polygon = GeoJSON не містить полігона
error-area-multi-polygon = MultiPolygon із { $parts } частинами; розділіть територію карти на окремі полігони
error-area-out-of-range = координата поза діапазоном: lat { $lat }, lon { $lon }
error-area-not-an-area = непридатна територія карти: { $message }
error-georef-date = дата { $date } поза межами чинності World Magnetic Model
error-georef-map-range = точка E { $easting } N { $northing } лежить за межами діапазону координат карти
error-omap-xml = недійсний XML OOM: { $message }
error-omap-missing-element = елемент <{ $name }> не знайдено
error-ocd-duplicate-symbol = номер знака { $symbol } використано двічі
error-ocd-unknown-symbol = об'єкт посилається на знак { $symbol }, якого немає у файлі
error-ocd-wrong-type = об'єкт знака { $symbol } має хибний для цього знака тип

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Використання:
cli-heading-commands = Команди:
cli-heading-options = Опції:
cli-heading-arguments = Аргументи:
cli-help-possible-values = можливі значення
cli-help-default = типово
cli-arg-help = Показати довідку
cli-arg-version = Показати версію
cli-help-help = Показати довідку команди
cli-arg-help-command = Команда, довідку якої показати, наприклад run
cli-parse-hint = Докладніше: '--help'.
cli-double-click-start = freemapper відкриває в браузері сторінку, на якій ви обираєте область карти. Папка проєкту: { $path }. Закрийте це вікно, щоб завершити програму.
cli-parse-unknown-argument = неочікуваний аргумент { $arg }
cli-parse-missing-required = бракує обов'язкового аргументу: { $args }
cli-parse-invalid-value = недійсне значення { $value } для { $arg }
cli-parse-invalid-value-choices = недійсне значення { $value } для { $arg }; можливі значення: { $values }
cli-parse-invalid-subcommand = невідома команда { $value }
cli-parse-missing-subcommand = потрібна команда
cli-parse-too-many-values = неочікуване значення { $value } для { $arg }
cli-parse-argument-conflict = { $arg } не можна використовувати разом із { $other }

help-freemapper = Створює стартові проєкти OOM із геоданих
help-freemapper-lang = Мова текстів програми, `de` або `en` (типово: FREEMAPPER_LANG, потім мова системи, потім англійська)
help-init = Створити папку проєкту для території карти
help-init-dir = Папка проєкту, яку слід створити
help-init-geojson = Файл GeoJSON із полігоном території карти (EPSG:4326)
help-init-osm = Об'єкт OSM території карти, напр. way/992423119
help-init-place = Назва місця, напр. "Clara-Zetkin-Schule Strausberg"; freemapper бере найкращий збіг із площею
help-init-part = Яку частину площі OSM з кількох окремих частин використати (1 найбільша)
help-init-kind = Вид карти
help-arg-scale = Знаменник масштабу нового проєкту: шкільне подвір'я 1000 (типово); спринт 4000 (типово) або 5000; ліс 15000 (типово), 10000 або 7500. Інші значення відхиляються, бо їх відхиляє freemapper-print
help-init-title = Назва карти для периферійної інформації (типово: назва папки)
help-export = Записати стартовий проєкт (карту OOM, шаблони, атрибуцію) у папку проєкту
help-export-dir = Папка проєкту
help-arg-force = Замінити наявний файл карти й зберегти старий як `<name>.omap.bak`
help-export-ocad = Також записати стартовий проєкт OCAD у `ocad/` (формат OCAD 12); наявна карта OOM тоді залишається, якщо не передати --force
help-export-pdf = Також записати `<name>.pdf`, карту для друку (лише об'єкти карти, без кандидатів); наявна карта OOM тоді залишається, якщо не передати --force
help-export-pdf-candidates = Також записати `<name>-candidates.pdf`, контрольний друк: карту з усіма кандидатами, точно за положенням; наявна карта OOM тоді залишається, якщо не передати --force
help-export-training = Також записати тренувальну карту `<name>-training.omap` і її PDF `<name>-training.pdf`: карту й усіх кандидатів, приведених до вигляду, придатного для друку як є; з --ocad також `ocad/<name>-training.ocd`. Наявна карта OOM тоді залишається, якщо не передати --force, а --force замінює й наявну тренувальну карту
help-export-view = З --training: також записати тренувальну карту без того, що вигляд пропускає, як `<name>-training-<view>.omap` і її PDF (з --ocad також файл OCAD). Вигляди: `no-paths` (без доріг, доріжок, сходів), `brown-only` (лише коричневі знаки та рамка), `no-north-lines`, або список `no:<codes>` / `only:<codes>`; поєднуються через `+`. Повторіть опцію, щоб отримати кілька файлів
help-export-pdf-scale-bar = Малювати лінійку масштабу на PDF (типово вимкнено, німецькі карти зазвичай її не мають; число масштабу залишається в основному написі)
help-export-pdf-sources = Рядки джерел на PDF: `auto` друкує повний список і скорочує його, лише коли він займає понад 5 рядків, `full` ніколи не скорочує, `credit-only` друкує лише рядок авторства freemapper (ODbL і dl-de/by-2-0 тоді вимагають називати джерела, коли ви передаєте карту далі). ATTRIBUTION.txt завжди перелічує кожне джерело
help-fetch = Завантажити дані для території карти (ортофото, модель рельєфу, nDSM, будівлі ALKIS, OSM)
help-fetch-dir = Папка проєкту
help-fetch-offline = Використовувати лише кеш, без доступу до мережі
help-arg-cache-dir = Папка кешу для завантажених плиток (типово: системна папка кешу + `freemapper`)
help-derive = Обчислити шаблони (ортофото, відмивку рельєфу, ухили, локальний рельєф, nDSM), горизонталі та форми рельєфу із завантажених даних
help-derive-dir = Папка проєкту
help-run = Завантажити, обчислити й експортувати за один раз
help-run-dir = Папка проєкту
help-run-offline = Використовувати лише кеш, без доступу до мережі
help-run-ocad = Також записати стартовий проєкт OCAD у `ocad/` (формат OCAD 12)
help-run-pdf = Також записати `<name>.pdf`, карту для друку (лише об'єкти карти, без кандидатів)
help-run-pdf-candidates = Також записати `<name>-candidates.pdf`, контрольний друк: карту з усіма кандидатами, точно за положенням
help-run-training = Також записати тренувальну карту `<name>-training.omap` і її PDF `<name>-training.pdf`: карту й усіх кандидатів, приведених до вигляду, придатного для друку як є; з --ocad також `ocad/<name>-training.ocd`. Наявна тренувальна карта залишається, якщо не передати --force
help-run-view = З --training: також записати тренувальну карту без того, що вигляд пропускає, як `<name>-training-<view>.omap` і її PDF (з --ocad також файл OCAD). Вигляди: `no-paths` (без доріг, доріжок, сходів), `brown-only` (лише коричневі знаки та рамка), `no-north-lines`, або список `no:<codes>` / `only:<codes>`; поєднуються через `+`. Наявний файл вигляду залишається, якщо не передати --force
help-run-pdf-scale-bar = Малювати лінійку масштабу на PDF (типово вимкнено; число масштабу залишається в основному написі)
help-run-pdf-sources = Рядки джерел на PDF: `auto` друкує повний список і скорочує його, лише коли він займає понад 5 рядків, `full` ніколи не скорочує, `credit-only` друкує лише рядок авторства freemapper (ODbL і dl-de/by-2-0 тоді вимагають називати джерела, коли ви передаєте карту далі). ATTRIBUTION.txt завжди перелічує кожне джерело
help-select = Вибрати територію карти в браузері: накреслити її або вибрати місце. Збереження записує freemapper.toml і за потреби створює папку проєкту
help-select-dir = Папка проєкту (створюється під час збереження, якщо її немає)
help-select-kind = Вид карти нового проєкту
help-select-title = Назва карти нового проєкту
help-select-no-browser = Лише надрукувати адресу сторінки; не відкривати браузер
help-select-port = Порт на 127.0.0.1 (типово: вільний)
help-select-offline = Сторінка починається з позначеного «Лише кеш, без мережі»
help-select-ocad = Сторінка починається з позначеного стартового проєкту OCAD
help-select-pdf = Сторінка починається з позначеного PDF для друку
help-select-pdf-candidates = Сторінка починається з позначеного контрольного друку (усі кандидати)
help-select-training = Сторінка починається з позначеної тренувальної карти
help-select-pdf-scale-bar = Сторінка починається з позначеної лінійки масштабу на PDF
help-select-pdf-sources = Сторінка починається з цього вибору рядків джерел на PDF, як для `run`
help-find = Перелічити місця OpenStreetMap за назвою, спершу площі, з id для init --osm
help-find-query = Назва місця, напр. "Clara-Zetkin-Schule Strausberg"
help-find-json = Вивести збіги як JSON, з тими самими полями, що й інструмент MCP find_place
help-mcp = Надавати інструменти ШІ-агентові через MCP (stdin/stdout)
help-update = Встановити новішу версію freemapper з випусків GitHub після запиту. Збірка з вихідного коду натомість каже, як її оновити
help-update-check = Лише перевірити: код виходу 10, якщо є новіша версія, 0, якщо ця актуальна
help-update-yes = Встановити без запиту; потрібно, коли введення не є терміналом
help-update-version = Встановити цю версію (x.y.z) замість найновішої, також старішу
help-update-rollback = Повернутися до версії, яку замінило останнє оновлення
help-update-mode = Зберегти, що робить щоденна перевірка, коли знаходить новішу версію (типово: notify). Працює в кожній збірці, бо лише записує налаштування
help-adapters = Показати регіональні адаптери: федеральні землі та країни, для яких у freemapper є джерела даних
help-adapters-list = Перелічити адаптери з джерелом кожного (вбудований, ваша папка, проєкт), ревізією, днем останньої перевірки та хостами, до яких він звертається
help-adapters-list-json = Вивести список як JSON
help-adapters-list-project = Також прочитати адаптери з папки `adapters` цієї папки проєкту, як це робить --adapters-from-project
help-adapters-check = Перевірити папку адаптера перед використанням або передачею: чи завантажиться він, що слід знати його авторові, а з --live чи відповідають його сервіси. Код виходу 0: проходить; 1: є зауваження; 2: перевірку не вдалося виконати
help-adapters-check-dir = Папка адаптера; її назва є id адаптера
help-adapters-check-live = Також запитати сервіси адаптера для рамки з --bbox і сказати, що кожен із них повертає. Запити лишаються на хостах, оголошених у маніфесті
help-adapters-check-bbox = Рамка живої перевірки в градусах, як minLon,minLat,maxLon,maxLat. Вона лежить усередині межі адаптера; достатньо 150 м на 150 м
help-adapters-check-json = Вивести результат як JSON
help-adapters-probe = Один раз опитати кожну службу, яку називають вбудовані адаптери, і сказати, чи вона відповідає: ok, slow, moved або error. Код виходу дорівнює 0, навіть якщо служба не відповідає; результат і є відповіддю
help-adapters-probe-json = Вивести результат як JSON
help-adapters-probe-status = Прочитати стан минулої ночі з цього файлу (його може не бути) і вивести новий стан як JSON: служба вважається недоступною після двох поганих ночей поспіль. Зміни йдуть у стандартний вивід помилок
help-adapters-allow-host = Дозволити вбудованому адаптеру ще один хост: для федеральної землі, сервіс якої переїхав на сервер, якого її адаптер ще не називає. Повідомлення про відхилену адресу каже, який це хост. Хост перевіряється як хост маніфесту, а freemapper і далі підключається лише до серверів в інтернеті
help-adapters-allow-host-adapter = Id вбудованого адаптера, як його показує `adapters list`
help-adapters-allow-host-host = Хост, який слід дозволити: назва DNS, не адреса, не порт
help-adapters-disallow-host = Скасувати хост, який дозволила allow-host
help-adapters-disallow-host-adapter = Id вбудованого адаптера
help-adapters-disallow-host-host = Хост, який слід скасувати
help-adapters-export = Записує вбудований адаптер федеральної землі як папку адаптера, яку можна змінювати: його маніфест і файли, названі в маніфесті, у `<dir>/<id>`. Папки ще не повинно бути, або вона має бути порожньою. Змініть адреси в її `manifest.toml`, перевірте її командою `adapters check` і покладіть у свою папку адаптерів: тоді вона замінить вбудований адаптер, із попередженням. Якщо прибрати папку, вбудований адаптер повернеться
help-adapters-export-adapter = Ідентифікатор вбудованого адаптера, як його показує `adapters list`
help-adapters-export-dir = Папка, в якій створюється папка `<id>`
help-arg-adapters-from-project = Також завантажити адаптери з папки `adapters` папки проєкту. freemapper називає кожен із них разом із хостами, до яких він звертається. Користуйтеся цим для проєкту, якому довіряєте: адаптер змушує freemapper звертатися до серверів, які він називає

## The summary of a run, for the MCP server and the area page

cli-internal-error = внутрішня помилка: { $message }
cli-pdf-sources-invalid = pdf_sources має бути auto, full або credit-only, а не { $value }
cli-view-unknown = вигляд це no-paths, brown-only, no-north-lines, no:<codes> або only:<codes>, а не { $value }
cli-view-empty = список { $value } не називає жодного коду знака
cli-view-bad-code = { $value } не є кодом знака на кшталт 505 або 501.6
cli-summary-training-view = Записано { $map } і { $pdf } (вигляд тренувальної карти)
cli-summary-kept-training-view = Залишено файли вигляду { $view } тренувальної карти; передайте --force, щоб їх оновити
cli-summary-training-view-failed = Вигляд тренувальної карти не вдався: { $error }
cli-estimate-disk-unreadable = не вдається прочитати вільне місце на диску для { $path } ({ $message })
cli-summary-wrote-map = Записано карту { $path }
cli-summary-kept-map = Залишено наявну карту { $path } (там може бути робота мапера)
cli-summary-map-lacks-templates = Карта ще не посилається на шаблони із зображеннями. Запустіть конвеєр з force, щоб замінити її; стару буде збережено як .omap.bak.
cli-summary-map-lacks-buildings = На карті ще немає будівель ALKIS. Запустіть конвеєр з force, щоб їх додати; він замінює карту й зберігає стару як .omap.bak.
cli-summary-map-lacks-canopies = На карті немає навісів і критих стоянок ALKIS (522), які тепер є об'єктами карти. Запустіть конвеєр з force, щоб їх додати; він замінює карту й зберігає стару як .omap.bak.
cli-summary-map-light = Базова карта: об'єднано ліній: { $joined }; прихованих або надто малих об'єктів залишено для шарів кандидатів: { $left } (нічого не зсунуто)
cli-summary-map-light-kept = Шари кандидатів тепер також містять { $left } об'єктів або відрізків, які ця версія прибирає з базової карти (позначені freemapper:generalized). Збережена карта їх ще має, тому вони показані двічі. Запустіть конвеєр з force, щоб очистити карту; він зберігає стару як .omap.bak.
cli-summary-map-old-candidates = Карта ще називає templates/candidates.omap; цей запуск заповнив його всіма кандидатами. Кандидати тепер також є в семи шарах, по одному файлу на кожен (templates/candidates-*.omap). Карта перелічує їх після запуску з force (стара карта залишиться як .omap.bak), або мапер додає їх сам в OOM (Шаблони > Відкрити шаблон…).
cli-summary-wrote-ocad = Записано карту OCAD { $path } (об'єктів: { $count })
cli-summary-kept-ocad = Залишено наявну карту OCAD { $path } (там може бути робота мапера; force замінює її)
cli-summary-ocad-old-candidates = Карта OCAD ще називає { $path } фоновою картою; цей запуск заповнив її всіма кандидатами. Кандидати тепер також є в семи шарах, по одному файлу на кожен (-candidates-*.ocd). Карта OCAD перелічує їх після запуску з force (стара карта залишиться як .ocd.bak), або мапер додає їх сам в OCAD як фонові карти.
cli-summary-ocad-left-out = Пропущено на карті OCAD: об'єктів { $skipped } і виродкових частин { $dropped }
cli-summary-ocad-symbols = В OCAD виберіть Symbol > Load Colors and Symbols From… і вкажіть офіційний набір знаків{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , шкільний набір IOF 2019
       *[other] {" "}ISSprOM 2019
    }; він замінює умовні знаки за номером.
cli-summary-ocad-failed = Експорт OCAD не вдався: { $error }
cli-summary-pdf-without-credits = PDF без вказання джерел: ODbL і dl-de/by-2-0 вимагають називати джерела, коли ви передаєте карту далі; див. ATTRIBUTION.txt
cli-summary-wrote-pdf = Записано карту { $path } (без кандидатів)
cli-summary-pdf-failed = Експорт PDF не вдався: { $error }
cli-summary-wrote-pdf-candidates = Записано контрольний друк { $path } (карта з усіма кандидатами, точно за положенням)
cli-summary-pdf-candidates-failed = Експорт контрольного друку не вдався: { $error }
cli-summary-training-failed = Експорт тренувальної карти не вдався: { $error }
cli-summary-kept-training = Залишено наявну тренувальну карту { $path }, щоб запуск не перезаписав карту, яку вже надрукували або передали; видаліть її або задайте force, щоб оновити

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = у цій системі немає домашньої папки; використайте абсолютний шлях
mcp-error-relative-path = { $path } є відносним шляхом; використайте абсолютний шлях або шлях, що починається з ~
mcp-error-home-folder = { $path } є самою домашньою папкою; використайте папку всередині неї, напр. ~/maps/school
mcp-error-no-project = у { $path } немає проєкту freemapper; спершу використайте create_project
mcp-error-unknown-kind = невідомий вид { $kind }; використайте "schoolyard", "sprint" або "forest"
mcp-error-reading = читання { $path }: { $error }
mcp-error-one-area-source = вкажіть рівно одне з osm_id (з find_place) або geojson
mcp-error-page-open = сторінка вибору території для цього проєкту ще відкрита; попросіть користувача зберегти територію й натиснути «Готово» на сторінці, а потім запустіть конвеєр
mcp-error-no-job = завдання { $job } немає; start_pipeline і permits_for_area повертають id завдань цього сеансу
mcp-error-job-running = { $job } ще виконується для { $path }; одночасно працює один конвеєр, бо проєкти спільно використовують кеш завантажень. Запустіть цей, коли project_status повідомить finished або failed
mcp-error-page-start = не вдається запустити сторінку: { $error }
mcp-created-project = Створено проєкт { $path } для { $area } ({ $kind }, 1:{ $scale }). Далі: estimate_download, потім start_pipeline.
mcp-created-project-given-area = Створено проєкт { $path } для заданої території ({ $kind }, 1:{ $scale }). Далі: estimate_download, потім start_pipeline.
mcp-page-already-open = Сторінка для цієї папки вже відкрита; вона зберігає налаштування, з якими її відкрито.
mcp-page-next = Попросіть користувача вибрати територію на сторінці, натиснути «Зберегти», а потім «Готово».
mcp-preview-of = Попередній перегляд { $layer }
mcp-permits-next = Примітки щодо дозволів створюються. Опитуйте project_status із цим job_id; коли завдання завершиться, permits_md містить текст permits.md. Передайте його як підказку, а не юридичну консультацію.
mcp-tool-check-update = Чи є новіший випуск freemapper: його версія, дата випуску, примітки до випуску та розмір завантаження, або що ця версія актуальна. Нічого не встановлює.
mcp-tool-create-project = Створити папку проєкту для території карти. Вкажіть osm_id (з find_place) або файл GeoJSON. kind: schoolyard (1:1000, типово), sprint (1:4000) або forest (1:15000).
mcp-param-create-project-dir = Нова папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-param-create-project-geojson = Замість osm_id: файл GeoJSON із полігоном території карти (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (типово, 1:1000), "sprint" (1:4000) або "forest" (1:15000).
mcp-param-create-project-osm-id = Об'єкт OSM території карти з find_place, напр. "way/992423119".
mcp-param-create-project-part = З osm_id: яку частину площі з кількох окремих частин використати (1 найбільша).
mcp-param-create-project-scale = Знаменник масштабу: шкільне подвір'я 1000 (типово); спринт 4000 (типово) або 5000; ліс 15000 (типово), 10000 або 7500. Інші значення відхиляються; для 1:3000 малюйте в 4000, freemapper-print збільшує під час друку.
mcp-param-create-project-title = Назва карти для периферійної інформації (типово: назва папки).
mcp-tool-estimate-download = Що start_pipeline завантажить для проєкту: плитки, кожне завантаження з розміром у МБ, чи воно в кеші та чи вміщається на диск, скільки МБ ще треба завантажити, а в disk вільне місце томів кешу й проєкту з повідомленнями про те, що не вміщається (повідомте користувача).
mcp-param-estimate-download-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-tool-find-place = Шукати школу, парк чи інше місце за назвою (OpenStreetMap Nominatim). Повертає до 5 збігів, спершу площі; територією карти можуть бути лише збіги з has_area. Порожній список означає, що нічого не знайдено.
mcp-param-find-place-query = Назва школи чи місця, напр. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Встановити найновіший випуск freemapper поверх цієї програми. Викликайте лише після згоди людини, з confirm true. Нова версія застосовується, коли асистент перезапустить сервер freemapper.
mcp-param-install-update-confirm = true, коли людина погодилася встановити новішу версію; без цього нічого не встановлюється.
mcp-tool-list-layers = Шари проєкту: карта, сім шарів кандидатів і растрові шаблони, кожен зі своїм файлом, чи його вже створено та який крок його створює.
mcp-param-list-layers-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-tool-open-area-selector = Відкрити в браузері сторінку, де користувач креслить територію карти (прямокутник або полігон) або вибирає місце, бачить плитки й розмір завантаження та зберігає її; збереження створює папку проєкту або змінює його територію. Повертає URL для користувача. Сторінка працює, доки користувач не натисне «Готово»; повторний виклик для тієї самої папки повертає той самий URL.
mcp-param-open-area-selector-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~); створюється, коли користувач зберігає.
mcp-param-open-area-selector-kind = Вид карти нового проєкту: "schoolyard" (типово, 1:1000), "sprint" (1:4000) або "forest" (1:15000).
mcp-param-open-area-selector-ocad = Панель запуску починається з позначеного стартового проєкту OCAD. Типово false.
mcp-param-open-area-selector-offline = Панель запуску починається з позначеного «Лише кеш, без мережі». Типово false.
mcp-param-open-area-selector-open-browser = Також відкрити сторінку в типовому браузері цього комп'ютера. Типово true.
mcp-param-open-area-selector-pdf = Панель запуску починається з позначеного PDF для друку. Типово false.
mcp-param-open-area-selector-pdf-candidates = Панель запуску починається з позначеного контрольного друку (усі кандидати). Типово false.
mcp-param-open-area-selector-pdf-scale-bar = Панель запуску починається з позначеної лінійки масштабу на PDF. Типово false.
mcp-param-open-area-selector-pdf-sources = Рядки джерел на PDF, з якими починається панель: "auto" (типово), "full" або "credit-only", як для start_pipeline.
mcp-param-open-area-selector-permits = Панель запуску починається з позначених приміток щодо дозволів. Типово false.
mcp-param-open-area-selector-scale = Знаменник масштабу нового проєкту: шкільне подвір'я 1000 (типово); спринт 4000 (типово) або 5000; ліс 15000 (типово), 10000 або 7500. Інші значення відхиляються.
mcp-param-open-area-selector-title = Назва карти нового проєкту.
mcp-param-open-area-selector-training = Панель запуску починається з позначеної тренувальної карти. Типово false.
mcp-tool-preview-layer = Попередній перегляд шару у форматі PNG (щонайбільше 800 пк): map (зображення об'єктів карти, яке записав останній запуск, без кандидатів) або растровий шаблон: orthophoto, hillshade, slope, relief, ndsm чи k5 у Берліні.
mcp-param-preview-layer-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" або "k5" (лише Берлін).
mcp-tool-project-status = Стан завдання start_pipeline або permits_for_area: running, finished або failed, останні рядки прогресу, попередження, записані файли та результат або помилка. Завершене завдання permits_for_area також несе текст permits.md як permits_md.
mcp-param-project-status-job-id = Id завдання з start_pipeline або permits_for_area, напр. "job-1".
mcp-tool-permits-for-area = Створити примітки щодо дозволів для проєкту: адміністративні одиниці території карти, природоохоронні території на ній і хто керує землею там, з органами, до яких звертатися; підказка, а не юридична консультація. Записує permits.md і permits.json у папку. Працює у фоні й повертає job_id; опитуйте project_status, відповідь якого після завершення завдання несе текст permits.md як permits_md. Одночасно виконується одне завдання.
mcp-param-permits-for-area-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-param-permits-for-area-offline = Читати лише дані, збережені в проєкті попереднім запуском, без доступу до мережі. Типово false.
mcp-tool-start-pipeline = Завантажити дані, обчислити шаблони й записати стартовий проєкт (fetch, derive, export). Працює у фоні й повертає job_id; опитуйте project_status. Одночасно працює один конвеєр. Наявну карту залишає, якщо force не true.
mcp-param-start-pipeline-dir = Папка проєкту (абсолютний шлях або шлях, що починається з ~).
mcp-param-start-pipeline-force = Замінити наявний файл карти й зберегти старий як .omap.bak. Типово false.
mcp-param-start-pipeline-ocad = Також записати стартовий проєкт OCAD у папку ocad/ (формат OCAD 12, для тих, хто редагує в OCAD). Типово false.
mcp-param-start-pipeline-offline = Використовувати лише кеш завантажень, без доступу до мережі. Типово false.
mcp-param-start-pipeline-pdf = Також записати <name>.pdf, карту, готову до друку (лише об'єкти карти, без кандидатів). Типово false.
mcp-param-start-pipeline-pdf-candidates = Також записати <name>-candidates.pdf, контрольний друк: карту з усіма кандидатами, точно за положенням (перевірте їх на місцевості). Типово false.
mcp-param-start-pipeline-pdf-scale-bar = Малювати лінійку масштабу на PDF. Типово false (німецькі карти зазвичай її не мають; число масштабу залишається в основному написі).
mcp-param-start-pipeline-pdf-sources = Рядки джерел на PDF: "auto" (типово; повний список, скорочений лише тоді, коли займає понад 5 рядків), "full" (ніколи не скорочується) або "credit-only" (лише рядок авторства freemapper; ODbL і dl-de/by-2-0 тоді вимагають називати джерела, коли карту передають далі). ATTRIBUTION.txt завжди перелічує кожне джерело.
mcp-param-start-pipeline-permits = Також записати примітки щодо дозволів (permits.md і permits.json) після карти: адміністративні одиниці, природоохоронні території та хто керує землею. Збій там є попередженням, а не невдалим завданням. Типово false.
mcp-param-start-pipeline-training = Також записати тренувальну карту <name>-training.omap і її PDF <name>-training.pdf: карту й усіх кандидатів, приведених до вигляду, придатного для друку як є; з ocad також ocad/<name>-training.ocd. Наявна тренувальна карта залишається, якщо force не true. Типово false.
mcp-param-start-pipeline-training-views = З training: також записати тренувальну карту без того, що вигляд пропускає, по одному набору файлів на запис: "no-paths", "brown-only", "no-north-lines" або список на кшталт "no:505,506" чи "only:101,102", поєднаний через +. Файли: <name>-training-<view>.omap і його PDF.
mcp-instructions = freemapper створює стартові проєкти OpenOrienteering Mapper для карт зі спортивного орієнтування (шкільне подвір'я 1:1000, спринт 1:4000, ліс 1:15000) з офіційних геоданих { $states } і з OSM. Робочий процес: find_place (шукати школу чи територію за назвою; вибрати результат із has_area) → create_project (нова папка, абсолютний шлях, з osm_id) → estimate_download (повідомити користувача, скільки буде завантажено) → start_pipeline (повертає id завдання; завантаження може тривати хвилини) → project_status (опитувати кожні кілька секунд, доки стан не стане finished або failed) → list_layers і preview_layer (показати користувачеві шаблони). Потім користувач відкриває файл карти в OpenOrienteering Mapper. Коли жодне місце не підходить або користувач хоче накреслити територію, open_area_selector відкриває в браузері сторінку, де користувач креслить або вибирає територію й зберігає її (це створює проєкт); продовжуйте з estimate_download, коли користувач скаже, що збережено. Шляхи мають бути абсолютними або починатися з ~. Одночасно працює один конвеєр. permits_for_area створює примітки щодо дозволів для проєкту (адміністративні одиниці, природоохоронні території, хто керує землею; підказка, а не юридична консультація) як завдання на кшталт start_pipeline. check_update каже, чи є новіший freemapper; install_update його встановлює, лише з confirm: true після згоди людини, а потім асистент перезапускає сервер freemapper.

cli-contour-interval-saved = Висоту перерізу збережено у freemapper.toml: { $interval } м; derive веде горизонталі з нею
cli-contour-interval-auto = Висота перерізу у freemapper.toml: автоматично (за видом карти; лісова карта отримує 2,5 м на рівнині та 10 м на крутому рельєфі)
cli-steep-slope-saved = Поріг крутизни збережено у freemapper.toml: { $slope }°; derive шукає за ним укоси та обриви
cli-steep-slope-auto = Поріг крутизни у freemapper.toml: типовий (близько 27°)
cli-face-slope-saved = Поріг стін збережено у freemapper.toml: { $slope }°; derive шукає за ним площі скельних стін
cli-face-slope-auto = Поріг стін у freemapper.toml: типовий (70°)
help-arg-contour-interval = Висота перерізу в метрах, напр. `2.5`, або `auto` для вибору за видом карти (лісова карта: 5 м, 2,5 м на рівнині, 10 м на крутому рельєфі). Зберігається у freemapper.toml; derive веде горизонталі з нею
help-arg-steep-slope = Кут схилу в градусах, починаючи з якого місцевість вважається крутою під час пошуку укосів та обривів, напр. `55`, або `auto` для типових 27°. Підвищ його (55 до 65) у скельних містах і ущелинах, де майже кожен схил крутий, а укоси й обриви зливаються в одну пляму. Зберігається у freemapper.toml; derive шукає за ним укоси та обриви
help-arg-face-slope = Кут схилу в градусах, починаючи з якого місцевість є скельною стіною (чорна площа, знак 206), наприклад `75`, або `auto` для типового значення 70°. 65 дає більше й більших площ, зокрема на крутих схилах; 75 залишає лише справжні стіни. Зберігається у freemapper.toml; derive шукає за ним скельні стіни
cli-content-saved = Налаштування вмісту збережено у freemapper.toml: { $summary }
cli-content-invalid = Не вдається прочитати налаштування вмісту: { $detail }
help-arg-content = Що потрапляє прямо на карту: `safe` лишає кожен об'єкт там, куди його ставить побудова, `balanced` (типово) ставить на карту об'єкти кадастру та реєстрів, `full` ставить на карту все. Вибір зберігається у freemapper.toml
help-arg-content-set = Спрямувати групу об'єктів на `map`, `candidates` або `off`, як `GROUP=PLACE`, напр. `walls=map`; можна повторювати. Групи: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Зберігається у freemapper.toml
mcp-param-create-project-content = Що потрапляє прямо на карту: "safe", "balanced" (типово) або "full". Зберігається в проєкті. balanced ставить на карту об'єкти кадастру та реєстрів, full ставить на неї все.
mcp-param-create-project-content-set = Групи, спрямовані не туди, куди каже профіль, як "group=place", де place це map, candidates або off, напр. "walls=map". Групи: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Висота перерізу в метрах, напр. 2.5; 0 повертає до автоматичного вибору (за видом карти; лісова карта отримує 2,5 м на рівнині та 10 м на крутому рельєфі). Зберігається в проєкті; наступний derive веде горизонталі з нею.
mcp-param-create-project-steep-slope = Кут схилу в градусах, починаючи з якого місцевість вважається крутою під час пошуку укосів та обривів, напр. 55; 0 повертає типові 27°. Підвищ його (55 до 65) у скельних містах і ущелинах, де майже кожен схил крутий, а укоси й обриви зливаються в одну пляму. Зберігається в проєкті; наступний derive шукає за ним укоси та обриви.
mcp-param-create-project-face-slope = Кут схилу в градусах, починаючи з якого місцевість є скельною стіною (чорна площа, знак 206), наприклад 75; 0 повертає типове значення 70°. 65 дає більше й більших площ, зокрема на крутих схилах; 75 залишає лише справжні стіни. Зберігається в проєкті; наступний derive шукає за ним скельні стіни.
mcp-param-start-pipeline-content = Змінити перед запуском, що потрапляє прямо на карту: "safe", "balanced" або "full". Зберігається в проєкті. Типово: налаштування проєкту, інакше balanced.
mcp-param-start-pipeline-content-set = Групи, спрямовані не туди, куди каже профіль, як "group=place", де place це map, candidates або off, напр. "fences=candidates". Зберігається в проєкті.
mcp-param-start-pipeline-contour-interval = Висота перерізу в метрах, напр. 2.5; 0 повертає до автоматичного вибору (за видом карти; лісова карта отримує 2,5 м на рівнині та 10 м на крутому рельєфі). Зберігається в проєкті; наступний derive веде горизонталі з нею.
mcp-param-start-pipeline-steep-slope = Кут схилу в градусах, починаючи з якого місцевість вважається крутою під час пошуку укосів та обривів, напр. 55; 0 повертає типові 27°. Підвищ його (55 до 65) у скельних містах і ущелинах, де майже кожен схил крутий, а укоси й обриви зливаються в одну пляму. Зберігається в проєкті; наступний derive шукає за ним укоси та обриви.
mcp-param-start-pipeline-face-slope = Кут схилу в градусах, починаючи з якого місцевість є скельною стіною (чорна площа, знак 206), наприклад 75; 0 повертає типове значення 70°. 65 дає більше й більших площ, зокрема на крутих схилах; 75 залишає лише справжні стіни. Зберігається в проєкті; наступний derive шукає за ним скельні стіни.
mcp-error-content = Не вдається прочитати налаштування вмісту: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Записати примітки щодо дозволів для проєкту (permits.md і permits.json): адміністративні одиниці, природоохоронні території та оператори, яких торкається територія карти, і до кого зазвичай слід звертатися. Звертається до BfN і BKG, а де федеральна земля території карти має власні відкриті сервіси, то й до них через хости її адаптера (список у посібнику, розділ 12); у Північному Рейні-Вестфалії вперше завантажує для цього близько 32 МБ, у Бранденбурзі близько 4 МБ, пізніше лише новіші файли. Читає вже кешований витяг OSM; --offline використовує те, що завантажено раніше
help-permits-dir = Папка проєкту
help-permits-offline = Використовувати лише раніше завантажені дані, без доступу до мережі
help-run-permits = Також записати примітки щодо дозволів (permits.md і permits.json) після карти; звертається до BfN, BKG і відкритих сервісів федеральної землі (посібник, розділ 12). Збій там є попередженням, а карта залишається
cli-wrote-permits = Записано примітки щодо дозволів { $path }
cli-summary-wrote-permits = Записано примітки щодо дозволів { $path }
cli-permits-failed = не вдалося записати примітки щодо дозволів: { $error }
cli-permits-source-left-out = примітки щодо дозволів пропускають { $source }: { $reason }
cli-permits-not-read = його не прочитано
error-permits-stored = { $path }: { $message }; знову завантажте примітки щодо дозволів з мережею
error-permits-nothing-read = не вдалося прочитати жодного джерела приміток щодо дозволів ({ $missing }); спершу завантажте їх з мережею
error-permits-missing-not-read = { $source }: не прочитано

permits-title = Примітки щодо дозволів
permits-disclaimer = Підказка, а не юридична консультація. Рішення ухвалюють компетентні органи. Дату даних і точність дивіться в таблиці під «Джерела та дата даних»; території можуть бути відсутні або застарілі (BfN: об'єднує дані раз на рік, «Nicht für Planungszwecke geeignet.», тобто непридатні для планування).
permits-disclaimer-regional = Підказка, а не юридична консультація. Рішення ухвалюють компетентні органи. Дату даних і точність дивіться в таблиці під «Джерела та дата даних»; території можуть бути відсутні або застарілі.
permits-bfn-merge = BfN об'єднує дані федеральних земель раз на рік і позначає їх як непридатні для планування. Рішення ухвалюють органи федеральної землі.
permits-created = Створено { $date } для території { $kind ->
        [schoolyard] карти шкільного подвір'я
        [forest] лісової карти
       *[other] спринтерської карти
    } ({ $area } га).
permits-section-admin = Адміністративні одиниці
permits-section-protected = Природоохоронні території (BfN)
permits-section-osm-protected = Інші природоохоронні території з OpenStreetMap
permits-section-operators = Оператори та власники за даними OpenStreetMap
permits-section-hints = Підказки
permits-section-sources = Джерела та дата даних
permits-none-admin = VG250 не називає для території карти жодної адміністративної одиниці.
permits-none-protected = За даними BfN, на території карти немає природоохоронних територій.
permits-none-osm-protected = OpenStreetMap не називає на території карти природоохоронних територій, яких BfN ще не перелічив.
permits-none-operators = OpenStreetMap не називає для території карти ні оператора, ні власника.
permits-none-hints = Підказок немає.
permits-section-not-read = Дані { $source } не прочитано, тому цей розділ порожній, хоч записи й можуть існувати.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Адміністративні території 1:250 000 (VG250) від BKG
        [bfn] Природоохоронні території Німеччини від BfN
       *[osm] OpenStreetMap, із витягу, який freemapper тримає у своєму кеші
    }
permits-level = { $level ->
        [land] Федеральна земля
        [kreis] Район (Kreis)
        [verwaltungsgemeinschaft] Amt або Verwaltungsgemeinschaft
       *[gemeinde] Громада (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Зачеплено на краю (менше ніж 0,5 % території карти):
permits-at-edge-generalised = VG250 генералізовано до масштабу 1:250 000. Одиниця, що лише торкається краю, може бути наслідком цього.
permits-kind = { $kind ->
        [nationalpark] Національний парк
        [nationales-naturmonument] Національна пам'ятка природи
        [naturschutzgebiet] Природний заповідник (Naturschutzgebiet)
        [ffh] Територія FFH (Natura 2000)
        [vogelschutzgebiet] Територія охорони птахів (Natura 2000)
        [biosphaerenreservat] Біосферний заповідник
        [biosphaerenreservat-zone] Зона біосферного заповідника
        [landschaftsschutzgebiet] Територія охорони ландшафту (Landschaftsschutzgebiet)
       *[naturpark] Природний парк
    }
permits-protected-line = { $kind } «{ $name }», { $share }
permits-protected-date = Оголошено { $date }
permits-protected-title = Правова підстава: { $title }
permits-protected-link = Правовий текст: { $url }
permits-protected-site-link = Сторінка території (Natura 2000): { $url }
permits-protected-year = Рік даних у BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = клас охорони { $class }
permits-quoted = «{ $text }»
permits-osm-unnamed = без назви
permits-osm-operator = Оператор за даними OSM: { $operator }
permits-osm-enclosing-gap = Території OpenStreetMap, які охоплюють усю територію карти без жодного кута всередині неї, тут можуть бути відсутні, так само як і природоохоронні території, нанесені в OSM як відношення з `type=boundary`. Частин від BfN і VG250 це не стосується.
permits-osm-enclosing-gap-short = Тут можуть бракувати ділянки OpenStreetMap, які охоплюють усю область карти, не маючи жодного кута всередині неї.
permits-operator-line = { $name } ({ $roles ->
        [operator] оператор
        [owner] власник
       *[both] оператор і власник
    }): { $objects }
permits-column-source = Джерело
permits-column-state = Стан
permits-column-credit = Авторство та ліцензія
permits-state-read = прочитано, завантажено { $date }
permits-state-read-osm = прочитано, витяг OSM від { $date }
permits-state-unreachable = недоступно ({ $reason })
permits-state-not-read = не прочитано
permits-no-credit = не використовується
permits-hint-city-state = Територія карти лежить у Берліні або Гамбурзі. Там зазвичай відповідальний Bezirksamt (районна адміністрація) або орган федеральної землі. Ці примітки поки не визначають район (Bezirk).
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Природоохоронні території стосуються нижчого органу охорони природи (untere Naturschutzbehörde). Тут це саме місто { $kreis }.
        [no] Природоохоронні території стосуються нижчого органу охорони природи (untere Naturschutzbehörde), зазвичай { $kreis }.
       *[unknown] Природоохоронні території стосуються компетентного нижчого органу охорони природи (untere Naturschutzbehörde), зазвичай Landkreis або kreisfreie Stadt.
    }
permits-hint-national-park = Національний парк має власні правила. Зверніться до адміністрації національного парку заздалегідь.
permits-hint-nsg-paths = У природному заповіднику зазвичай треба триматися доріжок. Для бігу поза доріжками зверніться до органу охорони природи.
permits-hint-natura2000 = Територія FFH або територія охорони птахів (Natura 2000) торкається території карти. Для заходу орган охорони природи може вимагати оцінки його впливу.
permits-hint-protected-areas-pl = В області карти розташовані охоронювані території або об'єкти охорони природи. Для національного парку, ландшафтного парку чи заповідника зазвичай звертаються до адміністрації парку або заповідника; щодо решти територій та об'єктів, хто відповідальний, зазвичай може сказати Регіональна дирекція охорони довкілля (RDOŚ) воєводства. Питай заздалегідь, якщо там планується захід.
permits-hint-order-office = { $amt ->
        [none] За вулиці, площі та інші громадські землі зазвичай відповідає Ordnungsamt (відомство громадського порядку) громади { $gemeinde }.
       *[other] За вулиці, площі та інші громадські землі зазвичай відповідає Ordnungsamt (відомство громадського порядку): { $amt }, для громади { $gemeinde }.
    }
permits-hint-park-operator = Оператор парку чи саду на території карти за даними OpenStreetMap: { $operator }. Запитайте в нього, перш ніж ним користуватися.
permits-hint-school-operator = Оператор школи чи закладу за даними OpenStreetMap: { $operator }. Щодо його території зверніться до директора школи або до цього органу.
permits-section-districts = Bezirke (райони міста, { $source })
permits-section-green-spaces = Зелені зони ({ $source })
permits-section-owners = Землі федеральної землі та Bund ({ $source })
permits-section-monuments = Пам'ятки ({ $source })
permits-section-protected-objects = Пам'ятки природи та інші охоронювані об'єкти ({ $source })
permits-none-districts = За даними { $source }, територія карти не лежить в жодному Bezirk.
permits-none-green-spaces = За даними { $source }, на території карти немає громадських зелених зон.
permits-none-owners = За даними { $source }, на території карти немає ділянок федеральної землі чи Bund.
permits-none-monuments = За даними { $source }, на території карти немає пам'яток.
permits-none-protected-objects = За даними { $source }, жоден із охоронюваних об'єктів, які воно містить, не лежить на території карти.
permits-none-sampled = У запитаних точках { $source } тут нічого не називає.
permits-gaps = Дані { $source } неповні: { $gaps }.
permits-sampled = { $source } відповідає лише для окремих точок. freemapper запитав у сітці точок над картою: частки є оцінками, а те, що лежить між точками, може бути відсутнє.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = призначення: { $dedication }
permits-playgrounds-none = На території карти немає громадських дитячих майданчиків.
permits-playgrounds-one = На території карти є один громадський дитячий майданчик. Окремо його не перелічено.
permits-playgrounds-many = На території карти громадських дитячих майданчиків: { $count }. Їх не перелічено поодинці.
permits-owner-other = Інші публічні власники
permits-owners-rest = Для решти території дані не називають власника.
permits-monuments-total = Пам'яток, цілком або частково на території карти: { $count }.{ $links ->
        [yes] {" "}Посилання ведуть до їхніх записів у списку пам'яток.
       *[no] {""}
    }
permits-monuments-links-left = Ще записів, лише порахованих тут, без посилання: { $count }.
permits-object-kind = { $kind ->
        [natural-monument] Пам'ятка природи (Naturdenkmal)
        [protected-landscape-part] Охоронюваний елемент ландшафту (geschützter Landschaftsbestandteil)
        [biotope] Законом охоронюваний біотоп (geschütztes Biotop)
        [national-park] Національний парк (park narodowy)
        [park-buffer-zone] Буферна зона парку (otulina)
        [landscape-park] Ландшафтний парк (park krajobrazowy)
        [nature-reserve] Природний заповідник (rezerwat przyrody)
        [protected-landscape-area] Охоронюваний ландшафт (obszar chronionego krajobrazu)
        [natura2000-habitats] Територія Natura 2000 для оселищ (SOO)
        [natura2000-birds] Територія Natura 2000 для птахів (OSO)
        [ecological-site] Екологічна ділянка (użytek ekologiczny)
        [landscape-complex] Природно-ландшафтний комплекс (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Документаційний об'єкт (stanowisko dokumentacyjne)
        [natural-monument-pl] Пам'ятка природи (pomnik przyrody)
        [national-park-cz] Національний парк (národní park, NP)
        [protected-landscape-area-cz] Охоронюваний ландшафт (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Національний природний заповідник (národní přírodní rezervace, NPR)
        [national-nature-monument] Національна пам'ятка природи (národní přírodní památka, NPP)
        [nature-reserve-cz] Природний заповідник (přírodní rezervace, PR)
        [nature-monument-cz] Пам'ятка природи (přírodní památka, PP)
        [protective-zone-cz] Охоронна зона (ochranné pásmo)
        [nature-zone-cz] Зона охорони природи (zóna ochrany přírody)
        [natura2000-habitats-cz] Територія Natura 2000 для оселищ (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Територія Natura 2000 для птахів (ptačí oblast, PO)
        [memorial-tree] Пам'ятне дерево (památný strom)
       *[special-rules] Територія зі спеціальними правилами
    }
permits-object-count = об'єктів: { $count }
permits-object-citations = Правові підстави:
permits-source-note = { $source }: «{ $note }»
permits-hint-district-office = { $offices ->
        [hamburg] За вулиці, площі та зелені зони в Bezirk { $district } зазвичай відповідає Bezirksamt { $district }, там Fachamt Management des öffentlichen Raumes (відділ управління громадським простором).
       *[berlin] За вулиці, площі та зелені зони в Bezirk { $district } зазвичай відповідають Ordnungsamt (відомство громадського порядку) і Straßen- und Grünflächenamt (відомство доріг і парків) Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] За пам'ятки на території карти відповідає Denkmalschutzamt Hamburg (відомство охорони пам'яток).
       *[berlin] За пам'ятки в Bezirk { $district } відповідає нижчий орган охорони пам'яток (untere Denkmalschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Чи лежать на території карти охоронювані пам'ятки, ці дані напевно сказати не можуть.
       *[yes] На території карти є охоронювані пам'ятки.
    } Чи зачепить їх забіг, наприклад КП біля пам'ятки чи перегін через охоронювану ділянку, вирішує нижчий орган охорони пам'яток (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Тут це саме місто { $kreis }.
        [no] , зазвичай { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; у деяких федеральних землях це громада { $gemeinde }.
        }
       *[unknown] {" "}(зазвичай Landkreis або kreisfreie Stadt, у деяких федеральних землях громада).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Пам'ятки природи, охоронювані елементи ландшафту та охоронювані біотопи на карті належать до компетенції нижчого органу охорони природи (untere Naturschutzbehörde). Тут це саме місто { $kreis }.
        [no] Пам'ятки природи, охоронювані елементи ландшафту та охоронювані біотопи на карті належать до компетенції нижчого органу охорони природи (untere Naturschutzbehörde), зазвичай { $kreis }.
       *[unknown] Пам'ятки природи, охоронювані елементи ландшафту та охоронювані біотопи на карті належать до компетенції нижчого органу охорони природи (untere Naturschutzbehörde), зазвичай Landkreis або kreisfreie Stadt.
    }
permits-hint-biotope = На території карти лежать законом охоронювані біотопи{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Усе, що може їх знищити або значно пошкодити, заборонено (§ 30 BNatSchG). Розміщуйте КП і перегони поза ними або запитайте нижчий орган охорони природи (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] За пам'ятки природи та охоронювані елементи ландшафту в Bezirk { $district } зазвичай відповідає Bezirksamt { $district }.
       *[berlin] За пам'ятки природи та охоронювані елементи ландшафту в Bezirk { $district } відповідає нижчий орган охорони природи (untere Naturschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] За відкритими даними федеральної землі, { $share } території карти належить іншим публічним власникам.
       *[no] За відкритими даними федеральної землі, { $share } території карти належить власнику «{ $owner }».
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Власників решти території за відкритими даними встановити не можна. За наявності законного інтересу (наприклад, в організатора) можна запросити витяг із земельного реєстру (Grundbuchamt); щодо публічних земель допоможе компетентний Ordnungsamt або відділ парків.
       *[no] Власників цих територій за відкритими даними встановити не можна. За наявності законного інтересу (наприклад, в організатора) можна запросити витяг із земельного реєстру (Grundbuchamt); щодо публічних земель допоможе компетентний Ordnungsamt або відділ парків.
    }
permits-section-forest-units = Лісове управління ({ $source })
permits-section-state-forest = Ліс федеральної землі ({ $source })
permits-none-forest-units = За даними { $source }, територія карти не лежить у зоні жодного лісництва (Forstamt).
permits-section-admin-units = Адміністративні одиниці ({ $source })
permits-none-admin-units = За даними { $source } територія карти не належить до жодної адміністративної одиниці реєстру.
permits-section-register-monuments = Пам'ятки національного реєстру ({ $source })
permits-none-register-monuments = За даними { $source } у прямокутнику навколо області карти немає жодної зареєстрованої пам'ятки.
permits-register-monuments-total = Зареєстровані пам'ятки в прямокутнику навколо області карти: { $count }. Реєстр не вказує положення, тому ці примітки не можуть сказати, де вони розташовані; це треба перевірити на карті.
permits-register-monuments-left = Ще { $count } тут лише пораховано, без назв.
permits-section-register-archaeology = Археологічні пам'ятки національного реєстру ({ $source })
permits-none-register-archaeology = За даними { $source } у прямокутнику навколо області карти немає жодної зареєстрованої археологічної пам'ятки.
permits-register-archaeology-total = Зареєстровані археологічні пам'ятки в прямокутнику навколо області карти: { $count }. Реєстр не вказує положення, тому ці примітки не можуть сказати, де вони розташовані; це треба перевірити на карті.
permits-register-archaeology-left = Ще { $count } тут лише пораховано, без назв.
permits-section-register-history = Пам'ятки історії ({ $source })
permits-none-register-history = За даними { $source } у прямокутнику навколо області карти немає жодної пам'ятки історії.
permits-register-history-in-area = Пам'ятки історії в області карти: { $count }.
permits-register-history-nearby = Ще пам'ятки історії в прямокутнику навколо області карти, але не в області карти: { $count }.
permits-register-history-left = Ще { $count } тут лише пораховано, без назв.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = За даними { $source }, на території карти немає лісу федеральної землі.
permits-forest-units-reach = Лісництво відповідає за ліс у своїй зоні. Його зона може охоплювати також міста й поля.
permits-state-forest-office = відповідальний: { $office }
permits-hint-forest-office = { $units ->
        [none] Лісництво, що відповідає за ліс на території карти: { $office } ({ $administration }). Зверніться туди заздалегідь, коли плануєте захід у лісі.
       *[other] Лісництво, що відповідає за ліс на території карти: { $office } ({ $administration }), { $units }. Зверніться туди заздалегідь, коли плануєте захід у лісі.
    }
permits-hint-forest-owner = { $office ->
        [known] Для заходу в лісі потрібна згода власників лісу. Лісництво зазвичай може сказати, кому належить ліс. У кількох федеральних землях організований захід у лісі треба також повідомити лісовому органу або отримати його дозвіл.
       *[unknown] Для заходу в лісі потрібна згода власників лісу. У кількох федеральних землях організований захід у лісі треба також повідомити лісовому органу або отримати його дозвіл. Ці примітки не називають лісництва, відповідального тут; про це можуть сказати лісове управління федеральної землі або нижчий лісовий орган.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Для заходу в лісі зазвичай потрібна згода власника лісу. Згадане вище лісництво (Nadleśnictwo) зазвичай може сказати, хто дбає про ліс. Ліс, що не належить Державним лісам, наприклад приватний ліс, у даних цих приміток відсутній. Ліс, що не належить державі, наприклад приватний або громадський ліс, перебуває під наглядом старости (starosta) щодо лісового господарства (закон про ліси, ст. 5 ust. 1 pkt 2); запитайте там в управлінні (starostwo powiatowe).
       *[unknown] Для заходу в лісі зазвичай потрібна згода власника лісу. Ці примітки не називають для області карти жодного лісництва Державних лісів; дані охоплюють лише Державні ліси, тому ліс може належати комусь іншому, наприклад приватному власнику. Хто власник, потрібно з'ясувати окремо. Ліс, що не належить державі, наприклад приватний або громадський ліс, перебуває під наглядом старости (starosta) щодо лісового господарства (закон про ліси, ст. 5 ust. 1 pkt 2); запитайте там в управлінні (starostwo powiatowe).
    }
permits-hint-monument-register-pl = У прямокутнику навколо області карти розташовані пам'ятки національного реєстру. Їхнього положення в даних немає, тому перевірте на карті, де вони. Згідно зі ст. 36 ч. 1 закону про охорону пам'яток (ustawa o ochronie zabytków) консерваційні, реставраційні та будівельні роботи на зареєстрованій пам'ятці потребують дозволу воєводського охоронця пам'яток (Wojewódzki Konserwator Zabytków). З 3 червня 2026 року будівельні роботи в оточенні пам'ятки, зміна її призначення, розміщення на ній пристроїв чи вивісок та інші дії, що можуть змінити її суть або вигляд, потребують лише повідомлення охоронцю (ст. 36aa); він може протягом 60 днів заперечити і тоді вимагати дозволу. Закон не називає біг чи контрольний пункт як такі; запитайте в бюро охоронця, якщо захід торкається пам'ятки чи її оточення.
permits-hint-archaeology-register-pl = Археологічні пам'ятки національного реєстру розташовані в прямокутнику навколо області карти. Їхнього положення в даних немає, тому перевірте на карті, де вони. Згідно зі ст. 36 ч. 1 закону про охорону пам'яток (ustawa o ochronie zabytków) археологічні дослідження (pkt 5), пошук прихованих або покинутих рухомих пам'яток за допомогою електронних чи технічних пристроїв, наприклад металошукачів (pkt 12), та консерваційні, реставраційні чи будівельні роботи на зареєстрованій пам'ятці (pkt 1) потребують дозволу воєводського консерватора пам'яток (Wojewódzki Konserwator Zabytków). Дозвіл на пошук втрачає чинність 1 січня 2027 року: ст. 36b тоді дозволить повнолітній особі шукати за згодою власника і користувача нерухомості після повідомлення в реєстрі пошуків через застосунок міністерства культури, а ст. 36c забороняє такі пошуки, зокрема, на зареєстрованих пам'ятках і ближче ніж 10 м від них. Закон не називає прогулянку, біг чи контрольний пункт як такі; запитайте відомство консерватора, якщо захід або польові роботи зачіпають зареєстрований об'єкт.
permits-hint-history-register-pl = Пам'ятка історії (Pomnik historii) розташована в області карти або поблизу неї; розділ про пам'ятки історії каже, яка саме. Президент Республіки Польща розпорядженням може визнати Pomnik historii нерухому пам'ятку, внесену до реєстру, або культурний парк особливої цінності (ст. 15 ч. 1 закону про охорону пам'яток, ustawa o ochronie zabytków). Отже, вона вже перебуває під охороною реєстру або культурного парку. Для робіт на пам'ятці, внесеній до реєстру, воєводський охоронець пам'яток (Wojewódzki Konserwator Zabytków) має видати дозвіл (ст. 36 ч. 1); у культурному парку заборони й обмеження встановлює рішення ради гміни (ст. 16 ч. 2, ст. 17 ч. 1). Закон не називає прогулянку, біг чи контрольний пункт як такі; запитайте відомство охоронця, якщо захід або польові роботи стосуються пам'ятки.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] В області карти є пам'ятка природи (pomnik przyrody), екологічна ділянка (użytek ekologiczny), природно-ландшафтний комплекс (zespół przyrodniczo-krajobrazowy) або документаційний об'єкт (stanowisko dokumentacyjne). Їх засновує рада гміни рішенням, у якому зазначено, хто здійснює нагляд за об'єктом і які заборони діють (ст. 44 Закону про охорону природи). Сам закон тут не передбачає жодного дозволу на захід. Запитайте рішення та відомості про особу, що здійснює нагляд, в управі гміни, на території якої розташований об'єкт.
       *[other] В області карти є пам'ятка природи (pomnik przyrody), екологічна ділянка (użytek ekologiczny), природно-ландшафтний комплекс (zespół przyrodniczo-krajobrazowy) або документаційний об'єкт (stanowisko dokumentacyjne). Їх засновує рада гміни рішенням, у якому зазначено, хто здійснює нагляд за об'єктом і які заборони діють (ст. 44 Закону про охорону природи). Сам закон тут не передбачає жодного дозволу на захід. Запитайте рішення та відомості про особу, що здійснює нагляд, { $count ->
        [one] в управі гміни ({ $gmina }), на території якої розташований об'єкт
       *[other] в управах гмін ({ $gmina }), у кожній щодо об'єктів на її території
    }.
    }
permits-hint-gmina-roads-pl = Захід, що заважає руху або особливим способом використовує вулицю, потребує дозволу за ст. 65 Закону про дорожній рух; заявку подають щонайменше за 30 днів. Дозвіл видає орган, що керує рухом на дорозі: староста для доріг повіту та гміни, мер у місті з правами повіту, маршалок воєводства або GDDKiA для решти доріг (ст. 10 і 65b). Дорогою гміни керує wójt, burmistrz або { $count ->
        [one] prezydent miasta гміни ({ $gmina })
       *[other] prezydent miasta кожної з гмін ({ $gmina })
    }; він вирішує питання про зайняття смуги відведення дороги (ст. 19 і 40 Закону про публічні дороги). Ці нотатки не показують категорію доріг.
permits-hint-gmina-land-pl = Якщо земля в області карти належить гміні, наприклад міський парк, площа, шкільна територія чи ліс, нею керує wójt, burmistrz або prezydent miasta, який представляє гміну (ст. 30 і 31 Закону про самоврядування гміни). Перед використанням запитайте згоду { $count ->
        [one] в управі гміни ({ $gmina }).
       *[other] в управах гмін ({ $gmina }).
    } Ці нотатки не показують, кому належить земля.
permits-hint-protected-areas-cz = В області карти розташовані охоронювані території або об'єкти охорони природи. Згідно із законом про охорону природи та ландшафту (закон № 114/1992 Sb.) у національному парку заборонені організовані спортивні, туристичні та інші публічні заходи поза місцями, які виділив орган охорони природи (§ 16 odst. 1 písm. q), а в охоронній зоні національного парку масові заходи такого роду поза цими місцями та поза забудованими територіями потребують згоди органу (§ 37 odst. 3 písm. d). У національному природному заповіднику заборонено заходити й виїжджати за межі доріжок, позначених за згодою органу (§ 29 písm. d). Органами охорони природи є, серед інших, адміністрація національного парку, Агентство охорони природи (AOPK ČR), крайове управління (krajský úřad) та управління громади з розширеними повноваженнями (§ 75 odst. 1). Для інших охоронюваних територій та об'єктів діють власні умови охорони. Якщо ви плануєте там захід, зверніться до органу заздалегідь.
permits-hint-monuments-cz = В області карти розташовані культурні пам'ятки або зони пам'яток державного переліку. Згідно із законом про державну охорону пам'яток (закон № 20/1987 Sb., § 14 odst. 1 і 2) власник пам'ятки має заздалегідь запросити рішення або обов'язковий висновок перед доглядом, ремонтом, реконструкцією чи іншою зміною пам'ятки або її оточення. Те саме стосується власника нерухомості в заповіднику пам'яток, зоні пам'яток або охоронній зоні пам'ятки у разі будівництва, зміни будівлі, земляних робіт, пристрою чи вивіски, знесення споруди, догляду за деревами або робіт з утримання. Вирішує управління громади з розширеними повноваженнями{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, для національної культурної пам'ятки крайове управління. Закон не називає прогулянку, біг чи контрольний пункт як такі; запитайте в цьому управлінні, якщо захід торкається пам'ятки або передбачає розміщення на ній пристроїв чи вивісок.
permits-hint-forest-owner-cz = Масовий спортивний, культурний або інший громадський захід на землі, призначеній для функцій лісу, можна проводити лише з письмової згоди власника лісу, в якому він відбувається (закон про ліси, закон № 289/1995 Sb., § 20 odst. 5). Організатор також має письмово повідомити про захід управління громади з розширеними повноваженнями{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, а якщо захід виходить за межі округу такої громади, крайове управління, не пізніше ніж за 30 днів до заходу. У повідомленні вказують дату і місце, як забезпечується захід, очікувану кількість учасників та письмові згоди власників (той самий параграф). Ці примітки не показують, кому належить ліс; заздалегідь запитайте власників лісу.
permits-hint-roads-cz = Захід, під час якого дорога використовується інакше, ніж звичайно, наприклад спортивний, культурний чи подібний захід, що може загрожувати безпеці або плавності дорожнього руху, потребує дозволу компетентного органу управління дорогами. Його видають за попередньої згоди власника дороги та, якщо можуть постраждати безпека або плавність руху, Поліції Чеської Республіки (закон про дороги, закон № 13/1997 Sb., § 25 odst. 1 і odst. 6 písm. d). Таким органом є муніципальне управління для місцевих доріг або управління громади з розширеними повноваженнями, де воно вирішує, а для доріг другого й третього класу управління громади з розширеними повноваженнями (§ 40 odst. 4 і 5). Громади в області карти: { $gmina }. Ці примітки не показують клас доріг.
permits-section-water-protection = Зони охорони вод ({ $source })
permits-section-forest-functions = Функції лісу та охоронюваний ліс ({ $source })
permits-none-water-protection = За даними { $source }, на території карти немає зони охорони вод.
permits-none-forest-functions = За даними { $source }, на території карти немає рекреаційного лісу, охоронюваного лісу, природного лісу та закритої лісової ділянки.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Рекреаційний ліс (Erholungswald)
        [protected] Охоронюваний ліс (Bannwald або Schutzwald)
        [natural] Природний ліс або суворий заповідник (Naturwald, Totalreservat)
       *[closed] Закрита лісова ділянка
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } зони охорони вод «{ $name }» лежить на території карти. У зонах I і II заходи зазвичай виключені. Зверніться до нижчого водного органу (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } зони охорони вод «{ $name }» лежить на території карти. Заходи там зазвичай можливі; нижчий водний орган (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) може сказати, що дозволяє розпорядження.
       *[unknown] Зона охорони вод «{ $name }» лежить на території карти. Дані не кажуть, яка це зона. У зонах I і II заходи зазвичай виключені; зверніться до нижчого водного органу (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Рекреаційний ліс (Erholungswald) лежить на території карти{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Щодо заходів там зверніться до лісового органу.
        [protected] Охоронюваний ліс (Bannwald або Schutzwald) лежить на території карти{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Зверніться до лісового органу, перш ніж планувати там захід.
        [natural] Природний ліс або суворий заповідник лежить на території карти{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Сходити зі стежок там часто заборонено; не ставте в ньому КП, не запитавши лісовий орган.
       *[closed] Лісова ділянка, куди заходити не можна, лежить на території карти{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Тримайте її поза дистанцією.
    }
permits-section-roads = Дороги з класифікацією за даними OpenStreetMap
permits-none-roads = За даними OpenStreetMap, територією карти не проходить жодна Autobahn і жодна Bundes-, Landes-, Staats- чи Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (автомагістраль)
        [bundesstrasse] Bundesstraße (федеральна дорога)
        [landesstrasse] Landesstraße або Staatsstraße (дорога федеральної землі)
       *[kreisstrasse] Kreisstraße (районна дорога)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Територією карти проходить Autobahn { $refs }. Дистанція може перетинати її лише мостами або підземними переходами.
        [bundesstrasse] Територією карти проходить Bundesstraße { $refs }. Перекриття або перетин із суддями на трасі призначає орган дорожнього руху (Straßenverkehrsbehörde); дорогу зазвичай утримує Bund через дорожню адміністрацію федеральної землі, у більших містах часто саме місто.
        [landesstrasse] Територією карти проходить Landes- або Staatsstraße { $refs }. Перекриття або перетин із суддями на трасі призначає орган дорожнього руху (Straßenverkehrsbehörde); дорогу зазвичай утримує федеральна земля, у більших містах часто саме місто.
       *[kreisstrasse] Територією карти проходить Kreisstraße { $refs }. Перекриття або перетин із суддями на трасі призначає орган дорожнього руху (Straßenverkehrsbehörde); дорогу зазвичай утримує Kreis, у більших містах часто саме місто.
    }
permits-hint-state-forest = { $office ->
        [none] За відкритими даними федеральної землі, { $share } території карти є лісом федеральної землі власника «{ $owner }».
       *[other] За відкритими даними федеральної землі, { $share } території карти є лісом федеральної землі власника «{ $owner }». Відповідальний: { $office }.
    }
