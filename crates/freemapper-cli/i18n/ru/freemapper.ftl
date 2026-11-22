# Program texts of freemapper, Russian (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } не входит в языки, на которых у freemapper есть тексты; доступны: { $available }

## init

cli-part-needs-osm = --part выбирает часть области OSM и требует --osm; с --place сначала найди идентификатор и используй --osm <id> --part <n>
cli-reading-file = чтение { $path }
cli-no-area-for-place = для «{ $query }» область не найдена. Если место состоит из нескольких отдельных частей, выбери одну через --osm <id> --part <n> или используй --geojson
cli-using-place = Используется { $name } ({ $id })
cli-created-project = Проект создан: { $path }

## Cache

cli-no-cache-dir = в этой системе нет каталога кэша; укажи --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Загружена { $count } плитка
        [few] Загружено { $count } плитки
        [many] Загружено { $count } плиток
       *[other] Загружено { $count } плитки
    }
cli-wrote = Записан файл { $path }
cli-kept-map = Файл { $path } оставлен без изменений (в нём может быть твоя работа)
cli-map-lacks-templates = В карте пока нет ссылок на растровые подложки. Добавь их в OOM (Подложки > Открыть подложку…) или замени карту через --force, старая останется как .omap.bak.
cli-map-lacks-buildings = В карте пока нет зданий ALKIS. Чтобы добавить их, замени карту через --force, старая останется как .omap.bak.
cli-map-lacks-canopies = В карте нет навесов и открытых парковочных навесов ALKIS (522), которые теперь стали объектами карты. Чтобы добавить их, запусти с --force, старая карта останется как .omap.bak.
cli-map-light = Базовая карта: объединено линий: { $joined }, оставлено для слоёв кандидатов скрытых или слишком мелких: { $left } (ничего не сдвигалось)
cli-map-light-kept = Слои кандидатов теперь содержат ещё объекты или участки, которые эта версия убирает с базовой карты: { $left } (с тегом freemapper:generalized). В сохранённой карте они всё ещё есть, поэтому показаны дважды. Замени карту через --force, чтобы очистить её (старая останется как .omap.bak).
cli-map-old-candidates = Карта всё ещё ссылается на templates/candidates.omap; этот запуск заполнил его всеми кандидатами. Теперь кандидаты есть и в семи слоях, по одному файлу на слой (templates/candidates-*.omap). Карта перечислит их после замены через --force (старая останется как .omap.bak), либо добавь их сам в OOM (Подложки > Открыть подложку…).

## PDF

cli-pdf-without-credits = PDF без указания источников: ODbL и dl-de/by-2-0 требуют называть источники, когда ты передаёшь карту дальше; см. ATTRIBUTION.txt
cli-wrote-pdf = Записан файл { $path } (карта без кандидатов)
cli-wrote-pdf-candidates = Записан файл { $path } (проверочная печать: карта со всеми кандидатами, точно по положению; проверь их на местности)
cli-wrote-training = Записаны { $map } и { $pdf } (тренировочная карта: объединено { $joined }, сдвинуто { $shifted }, пропущено или обрезано { $left })
cli-training-free-ends = Свободных концов линий: { $ways } там, где путь встречал другой путь, { $others } у других линий
cli-training-apart-left = в тренировочной карте всё ещё { $length } мм (на бумаге), где между объектами слишком малый зазор; проверь это в OOM
cli-kept-training = Тренировочная карта { $path } оставлена, чтобы запуск не перезаписал уже напечатанную или переданную карту; удали её или передай --force, чтобы создать заново

## OCAD

cli-ocad-kept = Файл { $path } оставлен (в нём может быть твоя работа; --force заменяет его, старый остаётся как .ocd.bak)
cli-ocad-old-candidates = Карта OCAD всё ещё ссылается на { $path } как на фоновую карту; этот запуск заполнил её всеми кандидатами. Теперь кандидаты есть и в семи слоях, по одному файлу на слой (-candidates-*.ocd). Карта OCAD перечислит их после замены через --force (старая останется как .ocd.bak), либо добавь их сам в OCAD как фоновые карты.
cli-ocad-wrote = { $count ->
        [one] Записан файл { $path } ({ $count } объект)
        [few] Записан файл { $path } ({ $count } объекта)
        [many] Записан файл { $path } ({ $count } объектов)
       *[other] Записан файл { $path } ({ $count } объекта)
    }
cli-ocad-left-out = Не попало в карту OCAD: объектов { $skipped } (нет символа для OCAD, нет площади или за пределами области рисования OCAD ±2 м) и { $dropped } вырожденных частей линий или отверстий.
cli-ocad-load-symbols = В OCAD выбери Символ > Загрузить цвета и символы из… и укажи официальный набор символов{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , школьный набор IOF 2019
       *[other] {" "}ISSprOM 2019
    }; он заменит символы-заглушки freemapper по номеру.

## select

cli-select-running = Выбор области для { $path }: нажми «Готово» на странице или Ctrl+C, чтобы остановить.
cli-select-no-browser = Не удалось открыть браузер ({ $error }); открой адрес выше.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; без области, не может быть областью карты): { $place }
cli-find-hint = Создай проект так: freemapper init <dir> --osm { $id }
cli-find-no-area = Ни у одного из этих мест нет области, которая могла бы стать областью карты. Нарисуй область так: freemapper select <dir>
cli-find-nothing = для «{ $query }» место не найдено; попробуй другое название или нарисуй область так: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = предупреждение: адаптер в { $folder } пропущен: { $finding }
cli-adapter-note = предупреждение: адаптер в { $folder } загружается, но: { $finding }
cli-adapter-from-project = предупреждение: используется адаптер { $adapter } из папки проекта; он может обращаться к этим хостам: { $hosts }
cli-adapter-overlaps = предупреждение: адаптер { $adapter } покрывает также территорию { $others }; его объекты попадают на тамошние карты
cli-adapter-dir-relative = предупреждение: FREEMAPPER_ADAPTER_DIR не является абсолютным путём ({ $path }); значение игнорируется
cli-adapters-no-project = предупреждение: { $path } не является папкой; адаптеры проекта не читались
cli-adapters-overlaps = пересекается с: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  ревизия { $revision }  проверен { $checked }  хосты: { $hosts }
cli-adapters-source-built-in = встроенный
cli-adapters-source-user = из твоей папки
cli-adapters-source-project = из проекта
cli-adapters-never-checked = никогда
cli-adapters-user-dir = Твои адаптеры читаются из { $path }
cli-adapters-no-user-dir = В этой системе нет папки для твоих адаптеров; её можно задать через FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper теперь может обращаться также к { $host } для { $name }. Запись находится в { $file }; `freemapper adapters disallow-host { $adapter } { $host }` отменяет её. Запущенная страница области или ИИ-ассистент читает запись при следующем запуске
cli-allow-host-already = { $host } уже разрешён для { $name }
cli-allow-host-declared = адаптер для { $name } сам называет { $host }; разрешать нечего
cli-allow-host-refused = { $host } нельзя разрешить: { $reason }
cli-allow-host-unknown-adapter = встроенного адаптера { $adapter } нет; встроенные адаптеры: { $adapters }
cli-allow-host-folder-adapter = { $adapter } является адаптером из твоей папки адаптеров; добавь хост вместо этого в `[net] hosts` его манифеста
cli-allow-host-no-config = в этой системе нет папки конфигурации, где хранить разрешённые хосты
cli-allow-host-write = не удалось записать { $file }: { $detail }
cli-allow-host-unreadable = { $file } не читается ({ $detail }); сначала исправь или удали файл; ничего не изменено
cli-disallow-host-removed = freemapper больше не обращается к { $host } для { $name }
cli-disallow-host-absent = { $host } не был разрешён для { $name }; отменять нечего
cli-allowed-hosts-unreadable = предупреждение: { $file } не читается ({ $detail }); дополнительные хосты не разрешены
cli-allowed-hosts-ignored = предупреждение: { $file }: запись { $host } для { $adapter } игнорируется: { $reason }
cli-allowed-hosts-no-adapter = у freemapper нет встроенного адаптера с таким именем
cli-allowed-host-redundant = предупреждение: { $host } теперь один из хостов, которые называет адаптер для { $name }; твоя запись в { $file } удалена
cli-adapters-allowed = разрешено тобой: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = адаптер { $adapter } ({ $name }), ревизия { $revision }, хосты: { $hosts }
cli-check-error = ошибка: { $finding }
cli-check-warning = предупреждение: { $finding }
cli-check-passed = { $folder } проходит проверку
cli-check-passed-warnings = { $folder } проходит проверку, { $count ->
        [one] с { $count } предупреждением
        [few] с { $count } предупреждениями
        [many] с { $count } предупреждениями
       *[other] с { $count } предупреждениями
    }
cli-check-failed = { $folder } не проходит проверку
cli-check-no-folder = папки { $folder } нет
cli-check-live-ok = { $capability } в порядке: { $detail }
cli-check-live-failed = { $capability } СБОЙ: { $detail }
cli-check-live-not-asked = { $capability } не запрашивалось: { $detail }
cli-check-live-box-outside = рамка не лежит внутри границы адаптера { $adapter }; живая проверка обращается к сервисам для места, которое покрывает адаптер
cli-check-live-cannot = живую проверку не удаётся выполнить: { $detail }
cli-check-detail-not-declared = адаптер ничего не заявляет
cli-check-detail-stopped = предыдущая часть не удалась
cli-check-detail-see-warnings = см. предупреждения
cli-check-detail-outside-dataset = рамка лежит за пределами набора данных
cli-check-detail-objects = { $count ->
        [one] { $count } объект
        [few] { $count } объекта
        [many] { $count } объектов
       *[other] { $count } объекта
    }
cli-check-detail-layers = слоёв: { $layers }, объектов: { $count }
cli-check-detail-trees = { $count ->
        [one] { $count } дерево
        [few] { $count } дерева
        [many] { $count } деревьев
       *[other] { $count } дерева
    }
cli-check-detail-extract = { $label }, данные от { $date }
cli-check-raster-orthophoto = ортофотоплан
cli-check-raster-terrain = модель рельефа
cli-check-raster-surface = модель поверхности
cli-check-raster-tiles = { $product }: { $count ->
        [one] { $count } плитка
        [few] { $count } плитки
        [many] { $count } плиток
       *[other] { $count } плитки
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } и { $last }

## Folder lock

cli-lock-held = другой запуск freemapper использует эту папку (с { $time }, запущен { $who }). Дождись его или сначала останови.
cli-lock-who-cli = терминалом
cli-lock-who-mcp = ИИ-агентом (MCP)
cli-lock-who-page = страницей выбора
cli-lock-who-unknown = другой программой
cli-lock-failed = не удалось заблокировать папку проекта для этого запуска: { $error }
cli-lock-stuck = на пути стоит не файл блокировки, и его не удалось убрать

## Run routes of the selection page

cli-run-bad-request = Параметры запуска недопустимы: { $error }
cli-run-no-project = В этой папке пока нет сохранённого проекта. Сначала сохрани область.
cli-run-already-running = Для этого проекта уже идёт запуск ({ $job }). Дождись его окончания или сначала отмени.
cli-run-disk-stops = Недостаточно свободного места на диске для загрузки: { $messages }
cli-run-busy-elsewhere = Другой проект этой программы сейчас выполняет конвейер. Проекты делят кэш загрузок, поэтому одновременно идёт только один запуск. Повтори, когда он закончится.
cli-run-nothing-to-cancel = Запуска нет, отменять нечего.
cli-run-unknown-job = Для этого проекта нет запуска { $job }.
cli-status-too-fast = Запросы состояния приходят слишком часто. Спрашивай не чаще пяти раз в секунду.
cli-open-folder-failed = Не удалось открыть папку ({ $error }).
cli-open-folder-no-opener = На этом компьютере нет программы для открытия папок ({ $program }). Открой папку вручную.
cli-open-folder-missing = Папки проекта пока нет. Сначала сохрани область.
cli-open-folder-refused = Имя папки небезопасно передавать файловому менеджеру.
cli-save-running = Для этого проекта идёт запуск. Сохрани снова, когда он закончится.
cli-save-unknown-kind = Неизвестный вид карты «{ $kind }». Используй schoolyard, sprint или forest.
cli-save-scale-too-large = Масштаб { $scale } слишком крупный.
cli-save-bad-content = Настройки содержимого недопустимы. Ничего не сохранено.
cli-map-image-none = Сначала создай карту.
cli-map-image-gone = Этот предпросмотр больше недоступен.
cli-map-image-running = Для этого проекта идёт запуск. Предпросмотр можно вычислить после его окончания.
cli-map-image-stale = Сохранённый проект больше не соответствует созданной карте, например потому, что изменились область, вид карты или масштаб. Сначала запусти создание, которое заменяет карту («Заменить существующую карту» на странице или freemapper run --force).
cli-map-image-superseded = Тем временем запрошен более новый предпросмотр, поэтому этот не вычислялся.
cli-map-image-bad-content = Настройки содержимого недопустимы.
cli-map-image-bad-request = Запрос предпросмотра недопустим. Отправь объект JSON, при необходимости с настройками содержимого.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – выбор области
page-heading = Выбери область карты
page-update-available = Доступен freemapper { $version }. Обновить командой
page-update-notes = Примечания к выпуску
page-folder = Папка проекта
page-pick = Найти место
page-query-placeholder = например, Clara-Zetkin-Schule Strausberg
page-search-button = Найти
page-searching = Поиск …
page-no-results = Ничего не найдено.
page-point-only = только точка, не область
page-draw = Нарисуй сам
page-rectangle = Прямоугольник
page-polygon = Многоугольник
page-finish-polygon = Замкнуть многоугольник
page-clear = Очистить
page-hint-rectangle = Щёлкни два противоположных угла на карте.
page-hint-polygon = Щёлкай углы один за другим. Чтобы замкнуть, щёлкни первый угол, сделай двойной щелчок или нажми «Замкнуть многоугольник». Esc отменяет, Backspace убирает последний угол.
page-hint-idle = Выбери место из поиска или нарисуй область сам.
page-summary = Область
page-source-osm = Из OpenStreetMap: { $name }
page-source-drawn = Нарисована вручную
page-size = Размер с буфером: { $width } × { $height } м
page-tiles-one = { $n } плитка: { $names }
page-tiles-other = Плиток: { $n }: { $names }
page-download = Загрузка: около { $mb } МБ
page-download-none = Загрузка: всё уже в кэше
page-download-details = Что загружается
page-cached = в кэше
page-disk = Место на диске: { $volumes }
page-disk-volume = свободно { $free }, нужно около { $needed }
page-cache-volume = кэш
page-project-volume = папка проекта
page-no-room = не помещается на диск
page-no-room-skip = На диске помещается не всё (freemapper оставляет свободными { $reserve }). Загрузка пропускает отмеченные части. Если они нужны, сначала освободи место:
page-no-room-stop = Недостаточно места на диске (freemapper оставляет свободными { $reserve }). Без отмеченных частей карты не будет, поэтому загрузка останавливается до скачивания. Сначала освободи место:
page-disk-unknown = freemapper не может прочитать свободное место на диске ({ $error }). Загрузка идёт без проверки.
page-too-large = Область слишком велика: не более { $max } м по стороне (с буфером). Выбери меньшую.
page-outside = Область лежит за пределами поддерживаемых федеральных земель ({ $states }); для других регионов у freemapper пока нет источников данных.
page-outside-unknown = Область лежит за пределами регионов, которые поддерживает freemapper; источников данных там пока нет.
page-adapters-failed = Аэрофотоснимки не удаётся перечислить ({ $detail }). Карта показывает только OpenStreetMap; всё остальное работает.
page-settings = Карта
page-kind = Вид карты
page-schoolyard = Карта школьного двора (1:1000)
page-sprint = Спринт-карта (1:4000)
page-forest = Лесная карта (1:15000)
page-scale = Масштаб 1:
page-map-title = Название
page-contour-interval = Сечение горизонталей
page-contour-auto = автоматически
page-settings-fixed = Проект уже существует; вид карты, масштаб и название лежат в freemapper.toml. Сечение горизонталей всё ещё можно изменить.
page-save = Сохранить
page-saving = Сохранение …
page-saved = Сохранено в freemapper.toml.
page-zone-changed = Сохранено. Область теперь лежит в другой зоне UTM; при следующем запуске freemapper загрузит её данные заново.
page-provider-changed = Сохранено. Высоты и аэрофотоснимки новой области предоставляет другой поставщик данных; при следующем запуске freemapper загрузит её данные заново.
page-save-stale = freemapper.toml изменили в другом окне или другой программой с тех пор, как эта страница его прочитала. Ничего не сохранено; страница теперь знает сохранённое состояние. Сохрани снова, чтобы заменить его показанной здесь областью.
page-has-map = Существующая карта (.omap) остаётся как есть; freemapper делает новые подложки и кандидаты для новой области. Опция «Заменить существующую карту» ниже (или «freemapper run --force») создаёт заново и саму карту, а старую сохраняет как .omap.bak.
page-done = Готово
page-source = Исходный код (AGPL-3.0-or-later)
page-closed = Готово. Можешь закрыть это окно.
page-next = Дальше: freemapper run "{ $dir }"
page-made-files = Создано: { $files }. Папка: { $dir }
page-back-to-agent = Скажи своему ИИ-ассистенту, что область сохранена.
page-nothing-saved = Ничего не сохранено.
page-failed = Ошибка: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Аэрофотоснимок { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Что попадает на карту
page-content-help = Профиль решает, что сразу попадает на карту, а что остаётся кандидатом для проверки. Выбор для группы сильнее профиля. Он сохраняется вместе с областью.
page-content-profile-legend = Профиль
page-content-profile-safe = Осторожный
page-content-profile-safe-hint = Каждый объект остаётся там, куда его кладёт сборка: здания и площади кадастра попадают на карту; стены, заборы, деревья и многое из OpenStreetMap (пруды, тропы с неизвестным покрытием) остаются кандидатами.
page-content-profile-balanced = Сбалансированный
page-content-profile-balanced-hint = Объекты кадастра, реестров и съёмки улиц попадают на карту, вместе с горизонталями и площадями, куда нельзя заходить, вдоль железнодорожных путей. Стены, заборы, живые изгороди, откосы, лестницы, вода, деревья, памятники, проходы, тропы и железные дороги берутся также из OpenStreetMap. Уличное оборудование, остальные площади, куда нельзя заходить, и формы рельефа из модели рельефа остаются кандидатами.
page-content-profile-full = Полный
page-content-profile-full-hint = Всё попадает на карту, как в проверочной печати; вне карты остаются только скрытые кандидаты.
page-content-groups-legend = По группам
page-content-groups-help = «Как в профиле» следует профилю. Что это значит для группы, зависит от источника каждого объекта.
page-content-place-profile = Как в профиле
page-content-place-map = Карта
page-content-place-candidates = Кандидат
page-content-place-off = Выкл.
page-content-group-walls = Стены
page-content-group-fences = Заборы
page-content-group-hedges = Живые изгороди
page-content-group-slopes = Откосы и обрывы
page-content-group-stairs = Лестницы
page-content-group-trees = Деревья и группы деревьев
page-content-group-water-features = Вода, фонтаны и родники
page-content-group-monuments = Башни, памятники и линии электропередачи
page-content-group-furniture = Уличное оборудование и фонари
page-content-group-passages = Проходы
page-content-group-contours = Горизонтали
page-content-group-landforms = Холмики, ямы, откосы и обрывы из модели рельефа
page-content-group-vegetation = Растительность по лазерным облакам точек (Бранденбург, Берлин, Северный Рейн-Вестфалия, Рейнланд-Пфальц, Саар и Бавария; выключено, пока не выберешь; загружает около 40–600 МБ на км²)
page-content-group-out-of-bounds = Площади, куда нельзя заходить
page-content-group-paths = Тропы и железные дороги
page-content-group-streets = Улицы и площади
page-run-heading = Создать карту
page-preview-heading = Предпросмотр карты
page-preview-stored = Как создано в последний раз.
page-preview-live = С настройками на этой странице, ещё не сохранёнными.
page-preview-live-saved = С сохранёнными настройками, ещё не создано запуском.
page-preview-computing = Вычисление …
page-preview-none = Карта ещё не создана.
page-preview-later = Предпросмотра этой карты пока нет. Он появится при следующем запуске, записывающем карту.
page-preview-stale = С момента создания карты изменились область, вид карты или масштаб. Изображение по-прежнему показывает карту, как она создана в последний раз; для нового предпросмотра нужен запуск с «Заменить существующую карту».
page-preview-area-unsaved = Область на этой странице ещё не сохранена. Изображение показывает карту, как она создана в последний раз; для новой области нужен запуск с «Заменить существующую карту».
page-preview-run = Как создано в последний раз. Страница обновит изображение, когда запуск закончится.
page-preview-open = Открыть в полном размере
page-preview-alt = Предпросмотр объектов карты
page-opt-ocad = Записать стартовый проект OCAD
page-help-ocad = Для тех, кто продолжает работу в OCAD.
page-opt-pdf = PDF для печати
page-help-pdf = Только объекты карты, без кандидатов.
page-opt-pdf-candidates = Проверочная печать со всеми кандидатами
page-opt-training = Тренировочная карта
page-help-training = Карта и все кандидаты, приведённые в порядок для печати как есть; с OCAD также файлом OCAD.
page-views-heading = Виды тренировочной карты
page-view-no-paths = Без троп (карта Ultimate)
page-view-brown-only = Только коричневое (ориентирование по горизонталям)
page-view-no-north-lines = Без линий севера
page-view-custom = Другой вид
page-help-views = Каждый отмеченный вид даёт ещё один файл рядом с тренировочной картой, `<name>-training-<view>.omap` и его PDF, с OCAD также файл OCAD. Другой вид задаётся как `no:505,506` (убрать символы) или `only:101,102` (оставить только эти); `+` объединяет виды. Название, масштаб и источники остаются всегда.
page-help-views-forest = «Без троп» и «только коричневое» есть только для лесных карт.
page-opt-scale-bar = Линейный масштаб на PDF
page-help-scale-bar = На немецких картах его обычно нет.
page-opt-sources = Строки источников на PDF
page-sources-auto = автоматически
page-sources-full = полностью
page-sources-credit-only = только freemapper
page-warn-credit-only = Тогда в PDF указан только freemapper. Лицензии на данные (ODbL, dl-de/by-2-0) требуют указания источников, когда ты передаёшь карту дальше; тогда источники называешь ты сам. ATTRIBUTION.txt перечисляет их все.
page-need-pdf = Линейный масштаб и строки источников действуют, только когда создаётся PDF.
page-opt-offline = Только кэш, без сети
page-opt-force = Заменить существующие карты
page-help-force = Карту и, если отмечена «Тренировочная карта», тренировочную карту. Старые файлы остаются как .omap.bak.
page-opt-permits = Заметки о разрешениях
page-help-permits = Административные единицы, охраняемые территории и к кому обращаться, в permits.md. Подсказка, не юридическая консультация.
page-run-start = Создать карту
page-run-again = Создать снова
page-run-cancel = Отмена
page-run-cancelling = Отмена …
page-warnings-heading = Предупреждения
page-where-files = Файлы лежат в папке проекта:
page-open-folder = Открыть папку
page-log-label = Журнал запуска
page-force-title = Заменить существующие карты?
page-force-text = Существующая карта будет заменена, а если отмечена «Тренировочная карта», то и тренировочная. Твои правки не потеряются: старые файлы останутся в папке проекта как .omap.bak. Продолжить?
page-force-cancel = Отмена
page-force-confirm = Заменить
page-step-fetch = Загрузка данных
page-step-derive = Вычисление подложек
page-step-map = Запись карты
page-step-ocad = Запись проекта OCAD
page-step-pdf = Запись PDF
page-step-pdf-candidates = Запись проверочной печати
page-step-training = Запись тренировочной карты
page-step-permits = Запись заметок о разрешениях
page-step-line = Шаг { $index } из { $total }: { $label }
page-starting = Запуск начинается …
page-file-kind-map = карта
page-file-kind-template = подложка
page-file-kind-ocad = карта OCAD
page-file-kind-ocad-candidates = кандидаты OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = проверочная печать
page-file-kind-training-map = тренировочная карта
page-file-kind-training-pdf = PDF тренировочной карты
page-file-kind-training-ocad = тренировочная карта OCAD
page-file-kind-training-view-map = вид тренировочной карты
page-file-kind-training-view-pdf = PDF вида тренировочной карты
page-file-kind-training-view-ocad = вид тренировочной карты OCAD
page-file-kind-preview = предпросмотр карты
page-file-kind-permits = заметки о разрешениях
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Готово. Файлы записаны.
page-result-failed = Запуск не удался: { $error }
page-result-failed-kept = Записанное остаётся в папке проекта; новый запуск продолжит оттуда.
page-result-cancelled = Отменено. Уже загруженное остаётся в кэше; новый запуск продолжит оттуда.
page-result-title-done = Карта создана
page-result-title-failed = Запуск не удался
page-result-title-cancelled = Запуск отменён
page-kept-map = Существующая карта оставлена как есть. «Заменить существующую карту» создаёт её заново, а старую сохраняет как .omap.bak.
page-no-files = Файлы не созданы.
page-opened-folder = Папка открыта.
page-open-failed = { $message } Папка: { $dir }
page-why-no-area = Сначала выбери область.
page-why-checking = Предпросмотр вычисляется …
page-why-too-large = Область слишком велика.
page-why-outside = Область лежит за пределами поддерживаемых федеральных земель.
page-why-disk = Недостаточно свободного места на диске (см. выше).
page-why-running = Идёт запуск.
page-err-scale-bar-no-pdf = Линейному масштабу нужен PDF.
page-err-sources-no-pdf = Строкам источников нужен PDF.
page-save-failed = Область не сохранена; запуск не начат.
page-poll-lost = Нет ответа от сервера. freemapper продолжает попытки.
page-run-unknown = Сервер больше не знает этот запуск.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Закрыть

## update (spec 3.4 to 3.6)

update-source-build = Этот freemapper собран из исходников. Обнови его через git pull и cargo build --release --locked.
update-current = freemapper { $version } актуален.
update-available = Доступен freemapper { $new } (у тебя { $old }), выпущен { $released }.
update-details =
    Примечания к выпуску: { $notes }
    Загрузка: { $size } МБ
update-decimal-separator = ,
update-ask = Установить сейчас? [y/N]
update-not-installed = Ничего не установлено.
update-needs-yes = ввод не является терминалом, поэтому freemapper не может спросить перед установкой; запусти freemapper update --yes, чтобы установить без вопроса
update-installing = загрузка и проверка freemapper { $version }
update-done = freemapper обновлён с { $old } до { $new }. Примечания к выпуску: { $notes }
update-mcp-restart = Работающий сервер MCP остаётся на старой версии, пока ИИ-ассистент не перезапустит его.
update-rolled-back = Возврат к freemapper { $version }. Версия, которую ты оставил, сохранена как { $path }.
update-not-writable = не удаётся записать в папку программы { $path }, а freemapper не запрашивает права администратора. Распакуй новый выпуск туда вручную или перенеси freemapper в папку, в которую ты можешь писать.
update-too-old = freemapper { $new } нельзя установить самообновлением версии { $old }; установи { $needed } или новее вручную, см. { $notes }
update-busy = в папке программы уже идёт другое обновление или откат (файл блокировки { $path }); дождись его окончания и повтори
update-bad-signature = выпуск не удалось проверить ({ $error }); ничего не установлено
update-check-bad-signature = новейший выпуск freemapper не удалось проверить ({ $error }); ничего не установлено
update-no-manifest = новейший выпуск не может установить себя сам (у него нет манифеста обновления); скачай его на { $page }
update-no-manifest-version = выпуска { $version } нет, или он не может установить себя сам (у него нет манифеста обновления); см. { $page }
update-no-asset = в выпуске нет программы для этой системы ({ $platform }); ничего не установлено
update-no-previous = нет предыдущей версии для возврата ({ $path } отсутствует)
update-failed = обновление не удалось: { $error }
update-mode-set = Режим обновления установлен: { $mode }, запись в { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } переопределяет эту настройку, пока задана.
update-mode-failed = не удалось записать { $path }: { $error }
update-notice = Доступен freemapper { $new } (у тебя { $old }). Обновить: freemapper update
update-applies-next-start = Новая версия применится при следующем запуске freemapper.
update-install-failed = freemapper { $new } не установлен: { $error }
update-no-config-dir = в этой системе нет папки конфигурации; задай FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Заметка для человека: при запуске сервер freemapper обнаружил следующее в папках адаптеров. Передай это дальше; человек исправляет папку или удаляет её, затем перезапускает сервер.
    { $lines }
update-mcp-note = Заметка для человека: доступен freemapper { $new } (это { $old }). Спроси его перед вызовом install_update.
update-mcp-needs-confirm = ничего не установлено; сначала спроси человека и вызови install_update с confirm: true, когда он согласится
update-mcp-note-manual = Заметка для человека: доступен freemapper { $new } (это { $old }), но install_update не может его установить; установи { $needed } или новее вручную, см. { $notes }
update-mcp-note-bad-signature = Заметка для человека: новейший выпуск freemapper не удалось проверить ({ $error }); ничего не установлено. Скажи ему об этом и не устанавливай выпуск другим способом.
update-mcp-restart-now = Перезапусти сервер freemapper MCP сейчас; он всё ещё работает на старой версии.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = ошибка
error-cancelled = отменено
cli-cancelled-kept = отменено; сохранено: { $files }
error-io = { $path }: { $message }
error-io-disk-full = диск заполнен, поэтому freemapper не смог записать { $path }. Освободи место и повтори

## Errors: project

error-project-already-exists = проект уже существует: { $path }
error-project-map-exists = карта уже существует (в ней может быть твоя работа); используй --force, чтобы заменить её: { $path }
error-project-invalid-scale = недопустимый масштаб 1:{ $scale }
error-project-invalid-contour-interval = недопустимое сечение горизонталей { $interval } м в freemapper.toml; оно должно быть больше 0
error-project-scale-not-allowed = масштаб 1:{ $scale } недопустим для карт вида { $kind }; используй { $allowed }
error-project-scale-not-allowed-enlarge = масштаб 1:{ $scale } недопустим для карт вида { $kind }; используй { $allowed }. Рисуй в 1:4000, freemapper-print увеличивает до 1:3000 при печати
word-kind-schoolyard = школьный двор
word-kind-sprint = спринт
word-kind-forest = лес
error-project-toml = недопустимый файл проекта: { $message }
error-project-unknown-adapter = в файле проекта назван адаптер «{ $adapter }», которого нет в этом freemapper; его может принести более новый freemapper (`freemapper update`), либо он из твоей папки адаптеров (его покажет `freemapper adapters list`), либо, с --adapters-from-project в командной строке, из папки `adapters` проекта (только для проекта, которому ты доверяешь)
error-project-not-fetched = плитки ещё не загружены; сначала выполни `freemapper fetch`
error-project-ocd = экспорт OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: изображение и world-файл не соответствуют области карты; выполни `freemapper derive` снова
error-project-tiles-outdated = для области карты нужны плитки { $tiles }, которые не были загружены; выполни `freemapper fetch`
error-project-buildings-not-fetched = для этой области карты не сохранены здания ALKIS; сначала выполни `freemapper fetch` онлайн
error-project-orthophoto-not-fetched = для этой области карты не сохранён ортофотоплан; сначала выполни `freemapper fetch` онлайн
error-project-terrain-not-fetched = для этой области карты не сохранена модель рельефа; сначала выполни `freemapper fetch` онлайн
error-project-candidates-outdated = сохранённые кандидаты OSM не покрывают область карты; выполни `freemapper fetch`
error-project-stored = { $path }: { $message }; выполни `freemapper fetch` снова
error-project-area-too-large = область карты с буфером составляет { $width } × { $height } м, больше { $limit } м по стороне; уменьши область карты или buffer_m в freemapper.toml
error-project-area-crosses-country = область карты лежит по обе стороны государственной границы; карты через границу пока не поддерживаются, поэтому нарисуй область внутри одной страны
error-project-permits-not-for-region = заметки о разрешениях для этого региона пока недоступны: они обращаются к немецким сервисам и называют немецкие ведомства
error-project-outside-coverage = область карты лежит за пределами поддерживаемых федеральных земель ({ $states }); пока у freemapper есть источники данных только для них

## Errors: disk space

error-space-required = недостаточно места на диске для: { $part }. Нужно { $needed }, но на { $path } свободно только { $left } сверх резерва { $reserve }. Освободи место и выполни `freemapper fetch` снова
error-space-skipped = freemapper пропускает: { $part }, из-за нехватки места на диске. Нужно { $needed }, но на { $path } свободно только { $left } сверх резерва { $reserve }. Освободи место и выполни `freemapper fetch` снова
word-part-map-data = данные карты ALKIS и OSM
word-part-terrain = модель рельефа
word-part-orthophoto = ортофотоплан
word-part-surface = модель поверхности для подложки nDSM
word-part-osm-update = обновление кэшированной выгрузки OSM
word-part-k-five = берлинская подложка карты K5
word-part-berlin-layers = берлинские структуры ALKIS, кадастр деревьев, съёмка улиц и точечные слои
word-part-state-layers = полные структуры ALKIS, объекты ландшафтной модели и кадастры деревьев
word-part-point-cloud = лазерные облака точек для кандидатов растительности

## Errors: downloads

error-fetch-http = загрузка { $url } не удалась: { $message }
error-fetch-http-temporary = загрузка { $url } не удалась: { $message }. Сервис, вероятно, перегружен или на обслуживании; freemapper пытался до { $attempts } раз. Запустите выполнение снова через несколько минут; уже загруженное сохраняется.
error-fetch-corrupt-archive = загрузка { $url } не является полным архивом плиток: { $message }
error-fetch-corrupt-extract = загрузка { $url } не является полной выгрузкой OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] плитка { $tiles } не входит в опубликованные плитки; уменьши область карты или buffer_m в freemapper.toml
        [few] плитки { $tiles } не входят в опубликованные плитки; уменьши область карты или buffer_m в freemapper.toml
        [many] плитки { $tiles } не входят в опубликованные плитки; уменьши область карты или buffer_m в freemapper.toml
       *[other] плитки { $tiles } не входят в опубликованные плитки; уменьши область карты или buffer_m в freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] офлайн и нет в кэше: { $product }, плитка { $tiles }; выполни `freemapper fetch` онлайн
        [few] офлайн и нет в кэше: { $product }, плитки { $tiles }; выполни `freemapper fetch` онлайн
        [many] офлайн и нет в кэше: { $product }, плитки { $tiles }; выполни `freemapper fetch` онлайн
       *[other] офлайн и нет в кэше: { $product }, плитки { $tiles }; выполни `freemapper fetch` онлайн
    }
word-product-orthophoto = ортофотоплан (DOP20)
word-product-dtm = модель рельефа (DGM1)
word-product-image-dsm = модель поверхности (bDOM)
word-product-saxony-dtm = модель рельефа (GeoSN DGM1)
word-product-saxony-dsm = модель поверхности (GeoSN DOM1)
error-fetch-osm-extract-missing = выгрузки OSM в кэше нет; выполни `freemapper fetch` онлайн
error-fetch-not-found = { $name } не найден на сервере
error-fetch-refused = { $url }: сервер отклонил запрос (HTTP { $status })
error-fetch-refused-detail = { $url }: сервер отклонил запрос (HTTP { $status }): { $detail }
error-fetch-too-large = загрузка { $url } больше { $limit } МБ; остановлена
error-fetch-not-stored = для этой области карты не сохранено: { $label }; { $path } отсутствует, не читается или не покрывает её; сначала выполни `freemapper fetch` онлайн
error-fetch-range-ignored = { $url }: сервер проигнорировал диапазон байтов { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper не отправляет запросы по этому адресу: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper не отправляет запросы по этому адресу: адаптер для { $name } не называет хост { $host }{ $from }. Если сервис переехал туда, разреши хост командой `freemapper adapters allow-host { $adapter } { $host }` и сообщи об этом на https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper не отправляет запросы по этому адресу: адаптер { $adapter } не называет хост { $host }{ $from }. Если сервис переехал туда, добавь хост в `[net] hosts` манифеста адаптера
error-fetch-host-redirected-from = {" "}(туда перенаправляет { $url })
error-reason-not-a-host-name = не имя хоста
error-reason-not-ascii = имя вне ASCII нужно записывать в форме xn--
error-reason-not-a-dns-name = не имя DNS
error-reason-no-domain = имя этого компьютера или без домена
error-reason-local-name = имя локальной сети
error-reason-ip-address = IP-адрес, а не имя DNS
error-reason-unexpected-character = неожиданный символ
error-reason-not-https = не https
error-reason-user-info = данные пользователя
error-reason-port = порт, отличный от 443
error-reason-host-not-declared = хост не входит в объявленные хосты
error-reason-local-address = адрес этого компьютера или локальной сети, а не сервера в интернете
error-reason-too-many-redirects = больше { $count } перенаправлений
error-reason-redirect-nowhere = перенаправление (HTTP { $status }) в никуда
error-fetch-currency-csv = CSV актуальности, строка { $line }: { $message }
error-alkis = недопустимые данные ALKIS: { $message }
error-osm = данные OSM: { $message }

## Errors: templates and rasters

error-derive-zip = архив плиток: { $message }
error-derive-image = изображение: { $message }
error-derive-no-tiles = нет плиток для вычисления
error-derive-area-too-large = область карты слишком велика для одной подложки: { $width } × { $height } пикселей (не более { $limit } по стороне); уменьши область карты или buffer_m в freemapper.toml
error-raster-world-file = недопустимый world-файл: { $message }
error-raster-misaligned = растры не выровнены по одной пиксельной сетке: { $message }
error-raster-pixel-size = у растров разный размер пикселя: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = недопустимый GeoJSON: { $message }
error-area-no-polygon = в GeoJSON нет многоугольника
error-area-multi-polygon = MultiPolygon из { $parts } частей; раздели область карты на отдельные многоугольники
error-area-out-of-range = координата вне диапазона: lat { $lat }, lon { $lon }
error-area-not-an-area = непригодная область карты: { $message }
error-georef-date = дата { $date } вне срока действия World Magnetic Model
error-georef-map-range = точка E { $easting } N { $northing } лежит за пределами диапазона координат карты
error-omap-xml = недопустимый XML OOM: { $message }
error-omap-missing-element = элемент <{ $name }> не найден
error-ocd-duplicate-symbol = номер символа { $symbol } использован дважды
error-ocd-unknown-symbol = объект ссылается на символ { $symbol }, которого нет в файле
error-ocd-wrong-type = у объекта символа { $symbol } неподходящий для его символа тип

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Использование:
cli-heading-commands = Команды:
cli-heading-options = Параметры:
cli-heading-arguments = Аргументы:
cli-help-possible-values = допустимые значения
cli-help-default = по умолчанию
cli-arg-help = Показать справку
cli-arg-version = Показать версию
cli-help-help = Показать справку по команде
cli-arg-help-command = Команда, справку по которой показать, например run
cli-parse-hint = Подробнее: '--help'.
cli-double-click-start = freemapper открывает в браузере страницу, на которой вы выбираете область карты. Папка проекта: { $path }. Закройте это окно, чтобы завершить программу.
cli-parse-unknown-argument = неожиданный аргумент { $arg }
cli-parse-missing-required = не хватает обязательного аргумента: { $args }
cli-parse-invalid-value = недопустимое значение { $value } для { $arg }
cli-parse-invalid-value-choices = недопустимое значение { $value } для { $arg }; допустимые значения: { $values }
cli-parse-invalid-subcommand = неизвестная команда { $value }
cli-parse-missing-subcommand = нужна команда
cli-parse-too-many-values = неожиданное значение { $value } для { $arg }
cli-parse-argument-conflict = { $arg } нельзя использовать вместе с { $other }

help-freemapper = Создаёт стартовые проекты OOM из геоданных
help-freemapper-lang = Язык текстов программы, `de` или `en` (по умолчанию: FREEMAPPER_LANG, затем язык системы, затем английский)
help-init = Создать папку проекта для области карты
help-init-dir = Создаваемая папка проекта
help-init-geojson = Файл GeoJSON с многоугольником области карты (EPSG:4326)
help-init-osm = Объект OSM области карты, например way/992423119
help-init-place = Название места, например "Clara-Zetkin-Schule Strausberg"; freemapper берёт лучшее совпадение с областью
help-init-part = Какую часть области OSM из нескольких отдельных частей взять (1 самая большая)
help-init-kind = Вид карты
help-arg-scale = Знаменатель масштаба нового проекта: школьный двор 1000 (по умолчанию); спринт 4000 (по умолчанию) или 5000; лес 15000 (по умолчанию), 10000 или 7500. Другие значения отклоняются, потому что их отклоняет freemapper-print
help-init-title = Название карты для периферийной информации (по умолчанию: имя папки)
help-export = Записать стартовый проект (карта OOM, подложки, указание источников) в папку проекта
help-export-dir = Папка проекта
help-arg-force = Заменить существующий файл карты, старый сохранить как `<name>.omap.bak`
help-export-ocad = Дополнительно записать стартовый проект OCAD в `ocad/` (формат OCAD 12); существующая карта OOM тогда остаётся, если не передать --force
help-export-pdf = Дополнительно записать `<name>.pdf`, карту для печати (только объекты карты, без кандидатов); существующая карта OOM тогда остаётся, если не передать --force
help-export-pdf-candidates = Дополнительно записать `<name>-candidates.pdf`, проверочную печать: карту со всеми кандидатами, точно по положению; существующая карта OOM тогда остаётся, если не передать --force
help-export-training = Дополнительно записать тренировочную карту `<name>-training.omap` и её PDF `<name>-training.pdf`: карту и всех кандидатов, приведённых в порядок для печати как есть; с --ocad также `ocad/<name>-training.ocd`. Существующая карта OOM тогда остаётся, если не передать --force, а --force заменяет и существующую тренировочную карту
help-export-view = С --training: дополнительно записать тренировочную карту без того, что исключает вид, как `<name>-training-<view>.omap` и её PDF (с --ocad также файл OCAD). Виды: `no-paths` (без дорог, троп, лестниц), `brown-only` (только коричневые символы и рамка), `no-north-lines`, либо список `no:<codes>` / `only:<codes>`; объединяй через `+`. Повтори параметр для нескольких файлов
help-export-pdf-scale-bar = Рисовать линейный масштаб на PDF (по умолчанию выключено, на немецких картах его обычно нет; число масштаба остаётся в основной надписи)
help-export-pdf-sources = Строки источников на PDF: `auto` печатает полный список и сокращает его, только если он занял бы больше 5 строк, `full` никогда не сокращает, `credit-only` печатает только строку авторства freemapper (ODbL и dl-de/by-2-0 тогда требуют называть источники, когда ты передаёшь карту дальше). ATTRIBUTION.txt всегда перечисляет все источники
help-fetch = Загрузить данные для области карты (ортофотоплан, модель рельефа, nDSM, здания ALKIS, OSM)
help-fetch-dir = Папка проекта
help-fetch-offline = Использовать только кэш, без доступа к сети
help-arg-cache-dir = Папка кэша для загруженных плиток (по умолчанию: системная папка кэша + `freemapper`)
help-derive = Вычислить подложки (ортофотоплан, отмывка рельефа, уклоны, локальный рельеф, nDSM), горизонтали и формы рельефа из загруженных данных
help-derive-dir = Папка проекта
help-run = Загрузить, вычислить и экспортировать за один раз
help-run-dir = Папка проекта
help-run-offline = Использовать только кэш, без доступа к сети
help-run-ocad = Дополнительно записать стартовый проект OCAD в `ocad/` (формат OCAD 12)
help-run-pdf = Дополнительно записать `<name>.pdf`, карту для печати (только объекты карты, без кандидатов)
help-run-pdf-candidates = Дополнительно записать `<name>-candidates.pdf`, проверочную печать: карту со всеми кандидатами, точно по положению
help-run-training = Дополнительно записать тренировочную карту `<name>-training.omap` и её PDF `<name>-training.pdf`: карту и всех кандидатов, приведённых в порядок для печати как есть; с --ocad также `ocad/<name>-training.ocd`. Существующая тренировочная карта остаётся, если не передать --force
help-run-view = С --training: дополнительно записать тренировочную карту без того, что исключает вид, как `<name>-training-<view>.omap` и её PDF (с --ocad также файл OCAD). Виды: `no-paths` (без дорог, троп, лестниц), `brown-only` (только коричневые символы и рамка), `no-north-lines`, либо список `no:<codes>` / `only:<codes>`; объединяй через `+`. Существующий файл вида остаётся, если не передать --force
help-run-pdf-scale-bar = Рисовать линейный масштаб на PDF (по умолчанию выключено; число масштаба остаётся в основной надписи)
help-run-pdf-sources = Строки источников на PDF: `auto` печатает полный список и сокращает его, только если он занял бы больше 5 строк, `full` никогда не сокращает, `credit-only` печатает только строку авторства freemapper (ODbL и dl-de/by-2-0 тогда требуют называть источники, когда ты передаёшь карту дальше). ATTRIBUTION.txt всегда перечисляет все источники
help-select = Выбрать область карты в браузере, нарисовав её или указав место. Сохранение записывает freemapper.toml и при необходимости создаёт папку проекта
help-select-dir = Папка проекта (создаётся при сохранении, если её нет)
help-select-kind = Вид карты нового проекта
help-select-title = Название карты нового проекта
help-select-no-browser = Только вывести адрес страницы; не открывать браузер
help-select-port = Порт на 127.0.0.1 (по умолчанию: свободный)
help-select-offline = Страница открывается с отмеченным «Только кэш, без сети»
help-select-ocad = Страница открывается с отмеченным стартовым проектом OCAD
help-select-pdf = Страница открывается с отмеченным PDF для печати
help-select-pdf-candidates = Страница открывается с отмеченной проверочной печатью (все кандидаты)
help-select-training = Страница открывается с отмеченной тренировочной картой
help-select-pdf-scale-bar = Страница открывается с отмеченным линейным масштабом на PDF
help-select-pdf-sources = Страница открывается с этим выбором строк источников на PDF, как для `run`
help-find = Перечислить места OpenStreetMap по названию, сначала области, с идентификатором для init --osm
help-find-query = Название места, например "Clara-Zetkin-Schule Strausberg"
help-find-json = Вывести совпадения как JSON, те же поля, что у инструмента MCP find_place
help-mcp = Предоставить инструменты ИИ-агенту через MCP (stdin/stdout)
help-update = Установить более новый freemapper из выпусков GitHub, после вопроса. Сборка из исходников вместо этого сообщает, как её обновить
help-update-check = Только проверить: код выхода 10, если есть более новая версия, 0, если эта актуальна
help-update-yes = Установить без вопроса; нужно, когда ввод не является терминалом
help-update-version = Установить эту версию (x.y.z) вместо новейшей, в том числе более старую
help-update-rollback = Вернуться к версии, которую заменило последнее обновление
help-update-mode = Сохранить, что делает ежедневная проверка, когда находит более новую версию (по умолчанию: notify). Работает в любой сборке, потому что только записывает настройку
help-adapters = Показать региональные адаптеры: федеральные земли и страны, для которых у freemapper есть источники данных
help-adapters-list = Перечислить адаптеры с указанием происхождения (встроенный, твоя папка, проект), ревизии, дня последней проверки и хостов, к которым они обращаются
help-adapters-list-json = Вывести список как JSON
help-adapters-list-project = Дополнительно прочитать адаптеры из папки `adapters` этой папки проекта, как это сделал бы --adapters-from-project
help-adapters-check = Проверить папку адаптера перед использованием или передачей: загрузится ли он, что стоит знать его автору, и с --live отвечают ли его сервисы. Код выхода 0: проходит; 1: есть замечания; 2: проверку не удалось выполнить
help-adapters-check-dir = Папка адаптера; её имя является идентификатором адаптера
help-adapters-check-live = Дополнительно опросить сервисы адаптера для рамки, названной в --bbox, и сказать, что выдаёт каждый. Запросы идут только на хосты, объявленные в манифесте
help-adapters-check-bbox = Рамка живой проверки в градусах, как minLon,minLat,maxLon,maxLat. Она лежит внутри границы адаптера; достаточно 150 м на 150 м
help-adapters-check-json = Вывести результат как JSON
help-adapters-probe = Один раз опросить каждую службу, которую называют встроенные адаптеры, и сообщить, отвечает ли она: ok, slow, moved или error. Код выхода равен 0, даже если служба не отвечает; результат и есть ответ
help-adapters-probe-json = Вывести результат как JSON
help-adapters-allow-host = Разрешить встроенному адаптеру ещё один хост: для федеральной земли, сервис которой переехал на сервер, которого её адаптер пока не называет. Какой это хост, говорит сообщение об отклонённом адресе. Хост проверяется как хост манифеста, и freemapper по-прежнему подключается только к серверам в интернете
help-adapters-allow-host-adapter = Идентификатор встроенного адаптера, как его показывает `adapters list`
help-adapters-allow-host-host = Разрешаемый хост: имя DNS, не адрес, без порта
help-adapters-disallow-host = Отменить хост, разрешённый через allow-host
help-adapters-disallow-host-adapter = Идентификатор встроенного адаптера
help-adapters-disallow-host-host = Хост, который нужно отменить
help-arg-adapters-from-project = Дополнительно загрузить адаптеры из папки `adapters` папки проекта. freemapper называет каждый из них вместе с хостами, к которым он обращается. Используй это для проекта, которому доверяешь: адаптер заставляет freemapper обращаться к названным им серверам

## The summary of a run, for the MCP server and the area page

cli-internal-error = внутренняя ошибка: { $message }
cli-pdf-sources-invalid = pdf_sources должно быть auto, full или credit-only, а не { $value }
cli-view-unknown = вид - это no-paths, brown-only, no-north-lines, no:<codes> или only:<codes>, а не { $value }
cli-view-empty = список { $value } не называет ни одного кода символа
cli-view-bad-code = { $value } не код символа вроде 505 или 501.6
cli-summary-training-view = Записаны { $map } и { $pdf } (вид тренировочной карты)
cli-summary-kept-training-view = Файлы вида { $view } тренировочной карты оставлены; передай --force, чтобы создать их заново
cli-summary-training-view-failed = Вид тренировочной карты не удался: { $error }
cli-estimate-disk-unreadable = не удаётся прочитать свободное место на диске для { $path } ({ $message })
cli-summary-wrote-map = Записана карта { $path }
cli-summary-kept-map = Существующая карта { $path } оставлена (в ней может быть работа картографа)
cli-summary-map-lacks-templates = В карте пока нет ссылок на растровые подложки. Запусти конвейер с force, чтобы заменить её, старая останется как .omap.bak.
cli-summary-map-lacks-buildings = В карте пока нет зданий ALKIS. Запусти конвейер с force, чтобы добавить их; он заменяет карту, старая остаётся как .omap.bak.
cli-summary-map-lacks-canopies = В карте нет навесов и открытых парковочных навесов ALKIS (522), которые теперь стали объектами карты. Запусти конвейер с force, чтобы добавить их; он заменяет карту, старая остаётся как .omap.bak.
cli-summary-map-light = Базовая карта: объединено линий: { $joined }, оставлено для слоёв кандидатов скрытых или слишком мелких: { $left } (ничего не сдвигалось)
cli-summary-map-light-kept = Слои кандидатов теперь содержат ещё объекты или участки, которые эта версия убирает с базовой карты: { $left } (с тегом freemapper:generalized). В сохранённой карте они всё ещё есть, поэтому показаны дважды. Запусти конвейер с force, чтобы очистить карту; старая остаётся как .omap.bak.
cli-summary-map-old-candidates = Карта всё ещё ссылается на templates/candidates.omap; этот запуск заполнил его всеми кандидатами. Теперь кандидаты есть и в семи слоях, по одному файлу на слой (templates/candidates-*.omap). Карта перечислит их после запуска с force (старая карта останется как .omap.bak), либо картограф добавит их в OOM (Подложки > Открыть подложку…).
cli-summary-wrote-ocad = Записана карта OCAD { $path } (объектов: { $count })
cli-summary-kept-ocad = Существующая карта OCAD { $path } оставлена (в ней может быть работа картографа; force заменяет её)
cli-summary-ocad-old-candidates = Карта OCAD всё ещё ссылается на { $path } как на фоновую карту; этот запуск заполнил её всеми кандидатами. Теперь кандидаты есть и в семи слоях, по одному файлу на слой (-candidates-*.ocd). Карта OCAD перечислит их после запуска с force (старая карта останется как .ocd.bak), либо картограф добавит их в OCAD как фоновые карты.
cli-summary-ocad-left-out = Не попало в карту OCAD: объектов { $skipped } и вырожденных частей { $dropped }
cli-summary-ocad-symbols = В OCAD выбери Символ > Загрузить цвета и символы из… и укажи официальный набор символов{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , школьный набор IOF 2019
       *[other] {" "}ISSprOM 2019
    }; он заменит символы-заглушки по номеру.
cli-summary-ocad-failed = экспорт OCAD не удался: { $error }
cli-summary-pdf-without-credits = PDF без указания источников: ODbL и dl-de/by-2-0 требуют называть источники, когда ты передаёшь карту дальше; см. ATTRIBUTION.txt
cli-summary-wrote-pdf = Записана карта { $path } (без кандидатов)
cli-summary-pdf-failed = Экспорт PDF не удался: { $error }
cli-summary-wrote-pdf-candidates = Записана проверочная печать { $path } (карта со всеми кандидатами, точно по положению)
cli-summary-pdf-candidates-failed = Экспорт проверочной печати не удался: { $error }
cli-summary-training-failed = Экспорт тренировочной карты не удался: { $error }
cli-summary-kept-training = Существующая тренировочная карта { $path } оставлена, чтобы запуск не перезаписал уже напечатанную или переданную карту; удали её или задай force, чтобы создать заново

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = в этой системе нет домашней папки; используй абсолютный путь
mcp-error-relative-path = { $path } - относительный путь; используй абсолютный путь или путь, начинающийся с ~
mcp-error-home-folder = { $path } - сама домашняя папка; используй папку внутри неё, например ~/maps/school
mcp-error-no-project = в { $path } нет проекта freemapper; сначала используй create_project
mcp-error-unknown-kind = неизвестный вид { $kind }; используй "schoolyard", "sprint" или "forest"
mcp-error-reading = чтение { $path }: { $error }
mcp-error-one-area-source = укажи ровно одно из osm_id (из find_place) или geojson
mcp-error-page-open = страница выбора области для этого проекта всё ещё открыта; попроси пользователя сохранить область и нажать «Готово» на странице, затем запусти конвейер
mcp-error-no-job = задачи { $job } нет; start_pipeline и permits_for_area возвращают идентификаторы задач этого сеанса
mcp-error-job-running = { $job } всё ещё выполняется для { $path }; конвейер идёт один за раз, потому что проекты делят кэш загрузок. Запусти этот, когда project_status сообщит finished или failed
mcp-error-page-start = не удаётся запустить страницу: { $error }
mcp-created-project = Проект { $path } создан для { $area } ({ $kind }, 1:{ $scale }). Дальше: estimate_download, затем start_pipeline.
mcp-created-project-given-area = Проект { $path } создан для заданной области ({ $kind }, 1:{ $scale }). Дальше: estimate_download, затем start_pipeline.
mcp-page-already-open = Страница для этой папки уже открыта; она сохраняет настройки, с которыми была открыта.
mcp-page-next = Попроси пользователя выбрать область на странице, нажать «Сохранить», затем «Готово».
mcp-preview-of = Предпросмотр: { $layer }
mcp-permits-next = Заметки о разрешениях создаются. Опрашивай project_status с этим job_id; когда задача завершится, permits_md содержит текст permits.md. Передай его как подсказку, не как юридическую консультацию.
mcp-tool-check-update = Есть ли более новый выпуск freemapper: его версия, дата выпуска, примечания к выпуску и размер загрузки, либо что эта версия актуальна. Ничего не устанавливает.
mcp-tool-create-project = Создать папку проекта для области карты. Укажи osm_id (из find_place) или файл GeoJSON. kind: schoolyard (1:1000, по умолчанию), sprint (1:4000) или forest (1:15000).
mcp-param-create-project-dir = Новая папка проекта (абсолютный путь или начинающийся с ~).
mcp-param-create-project-geojson = Вместо osm_id: файл GeoJSON с многоугольником области карты (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (по умолчанию, 1:1000), "sprint" (1:4000) или "forest" (1:15000).
mcp-param-create-project-osm-id = Объект OSM области карты из find_place, например "way/992423119".
mcp-param-create-project-part = С osm_id: какую часть области из нескольких отдельных частей взять (1 самая большая).
mcp-param-create-project-scale = Знаменатель масштаба: школьный двор 1000 (по умолчанию); спринт 4000 (по умолчанию) или 5000; лес 15000 (по умолчанию), 10000 или 7500. Другие значения отклоняются; для 1:3000 рисуй в 4000, freemapper-print увеличивает при печати.
mcp-param-create-project-title = Название карты для периферийной информации (по умолчанию: имя папки).
mcp-tool-estimate-download = Что start_pipeline загрузит для проекта: плитки, каждую загрузку с её размером в МБ, лежит ли она в кэше и помещается ли на диск, сколько МБ ещё предстоит загрузить, а в disk свободное место томов кэша и проекта с сообщениями о том, что не помещается (скажи об этом пользователю).
mcp-param-estimate-download-dir = Папка проекта (абсолютный путь или начинающийся с ~).
mcp-tool-find-place = Найти школу, парк или другое место по названию (OpenStreetMap Nominatim). Возвращает до 5 совпадений, сначала области; областью карты могут быть только совпадения с has_area. Пустой список значит, что ничего не найдено.
mcp-param-find-place-query = Название школы или места, например "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Установить новейший выпуск freemapper поверх этой программы. Вызывай только после согласия человека, с confirm true. Новая версия применяется, когда ассистент перезапустит сервер freemapper.
mcp-param-install-update-confirm = true, когда человек согласился установить более новую версию; без этого ничего не устанавливается.
mcp-tool-list-layers = Слои проекта: карта, семь слоёв кандидатов и растровые подложки, каждый с его файлом, создан ли он уже и каким шагом создаётся.
mcp-param-list-layers-dir = Папка проекта (абсолютный путь или начинающийся с ~).
mcp-tool-open-area-selector = Открыть в браузере страницу, где пользователь рисует область карты (прямоугольник или многоугольник) или выбирает место, видит плитки и размер загрузки и сохраняет её; сохранение создаёт папку проекта или меняет её область. Возвращает URL, который нужно передать пользователю. Страница работает, пока пользователь не нажмёт «Готово»; повторный вызов для той же папки возвращает тот же URL.
mcp-param-open-area-selector-dir = Папка проекта (абсолютный путь или начинающийся с ~); создаётся, когда пользователь сохраняет.
mcp-param-open-area-selector-kind = Вид карты нового проекта: "schoolyard" (по умолчанию, 1:1000), "sprint" (1:4000) или "forest" (1:15000).
mcp-param-open-area-selector-ocad = Панель запуска открывается с отмеченным стартовым проектом OCAD. По умолчанию false.
mcp-param-open-area-selector-offline = Панель запуска открывается с отмеченным «Только кэш, без сети». По умолчанию false.
mcp-param-open-area-selector-open-browser = Дополнительно открыть страницу в браузере по умолчанию на этом компьютере. По умолчанию true.
mcp-param-open-area-selector-pdf = Панель запуска открывается с отмеченным PDF для печати. По умолчанию false.
mcp-param-open-area-selector-pdf-candidates = Панель запуска открывается с отмеченной проверочной печатью (все кандидаты). По умолчанию false.
mcp-param-open-area-selector-pdf-scale-bar = Панель запуска открывается с отмеченным линейным масштабом на PDF. По умолчанию false.
mcp-param-open-area-selector-pdf-sources = Строки источников на PDF, с которыми открывается панель: "auto" (по умолчанию), "full" или "credit-only", как для start_pipeline.
mcp-param-open-area-selector-permits = Панель запуска открывается с отмеченными заметками о разрешениях. По умолчанию false.
mcp-param-open-area-selector-scale = Знаменатель масштаба нового проекта: школьный двор 1000 (по умолчанию); спринт 4000 (по умолчанию) или 5000; лес 15000 (по умолчанию), 10000 или 7500. Другие значения отклоняются.
mcp-param-open-area-selector-title = Название карты нового проекта.
mcp-param-open-area-selector-training = Панель запуска открывается с отмеченной тренировочной картой. По умолчанию false.
mcp-tool-preview-layer = Предпросмотр слоя в PNG (не более 800 px): map (изображение объектов карты, записанное последним запуском, без кандидатов) или растровая подложка: orthophoto, hillshade, slope, relief, ndsm, либо k5 в Берлине.
mcp-param-preview-layer-dir = Папка проекта (абсолютный путь или начинающийся с ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" или "k5" (только Берлин).
mcp-tool-project-status = Состояние задачи start_pipeline или permits_for_area: running, finished или failed, последние строки хода, предупреждения, записанные файлы, а также результат или ошибка. Завершённая задача permits_for_area содержит также текст permits.md как permits_md.
mcp-param-project-status-job-id = Идентификатор задачи из start_pipeline или permits_for_area, например "job-1".
mcp-tool-permits-for-area = Создать заметки о разрешениях для проекта: административные единицы области карты, охраняемые территории в ней и кто управляет землёй, с ведомствами, к которым стоит обратиться; подсказка, не юридическая консультация. Записывает permits.md и permits.json в папку. Работает в фоне и возвращает job_id; опрашивай project_status, ответ которого по завершении задачи содержит текст permits.md как permits_md. Одновременно выполняется одна задача.
mcp-param-permits-for-area-dir = Папка проекта (абсолютный путь или начинающийся с ~).
mcp-param-permits-for-area-offline = Читать только данные, сохранённые в проекте предыдущим запуском, без доступа к сети. По умолчанию false.
mcp-tool-start-pipeline = Загрузить данные, вычислить подложки и записать стартовый проект (fetch, derive, export). Работает в фоне и возвращает job_id; опрашивай project_status. Одновременно идёт один конвейер. Существующую карту оставляет, если force не true.
mcp-param-start-pipeline-dir = Папка проекта (абсолютный путь или начинающийся с ~).
mcp-param-start-pipeline-force = Заменить существующий файл карты, старый сохранить как .omap.bak. По умолчанию false.
mcp-param-start-pipeline-ocad = Дополнительно записать стартовый проект OCAD в папку ocad/ (формат OCAD 12, для тех, кто работает в OCAD). По умолчанию false.
mcp-param-start-pipeline-offline = Использовать только кэш загрузок, без доступа к сети. По умолчанию false.
mcp-param-start-pipeline-pdf = Дополнительно записать <name>.pdf, карту, готовую к печати (только объекты карты, без кандидатов). По умолчанию false.
mcp-param-start-pipeline-pdf-candidates = Дополнительно записать <name>-candidates.pdf, проверочную печать: карту со всеми кандидатами, точно по положению (проверь их на местности). По умолчанию false.
mcp-param-start-pipeline-pdf-scale-bar = Рисовать линейный масштаб на PDF. По умолчанию false (на немецких картах его обычно нет; число масштаба остаётся в основной надписи).
mcp-param-start-pipeline-pdf-sources = Строки источников на PDF: "auto" (по умолчанию; полный список, сокращается, только если занимает больше 5 строк), "full" (никогда не сокращается) или "credit-only" (только строка авторства freemapper; ODbL и dl-de/by-2-0 тогда требуют называть источники при передаче карты дальше). ATTRIBUTION.txt всегда перечисляет все источники.
mcp-param-start-pipeline-permits = Дополнительно записать заметки о разрешениях (permits.md и permits.json) после карты: административные единицы, охраняемые территории и кто управляет землёй. Сбой там даёт предупреждение, а не неудачную задачу. По умолчанию false.
mcp-param-start-pipeline-training = Дополнительно записать тренировочную карту <name>-training.omap и её PDF <name>-training.pdf: карту и всех кандидатов, приведённых в порядок для печати как есть; с ocad также ocad/<name>-training.ocd. Существующая тренировочная карта остаётся, если force не true. По умолчанию false.
mcp-param-start-pipeline-training-views = С training: дополнительно записать тренировочную карту без того, что исключает вид, по набору файлов на запись: "no-paths", "brown-only", "no-north-lines" либо список вроде "no:505,506" или "only:101,102", объединённые через +. Файлы: <name>-training-<view>.omap и её PDF.
mcp-instructions = freemapper создаёт стартовые проекты OpenOrienteering Mapper для карт спортивного ориентирования (школьный двор 1:1000, спринт 1:4000, лес 1:15000) из официальных геоданных { $states } и из OSM. Порядок работы: find_place (найти школу или область по названию; выбрать результат с has_area) → create_project (новая папка, абсолютный путь, с osm_id) → estimate_download (скажи пользователю, сколько будет загружено) → start_pipeline (возвращает идентификатор задачи; загрузка может занять минуты) → project_status (опрашивай каждые несколько секунд, пока состояние не станет finished или failed) → list_layers и preview_layer (покажи пользователю подложки). Затем пользователь открывает файл карты в OpenOrienteering Mapper. Когда ни одно место не подходит или пользователь хочет нарисовать область, open_area_selector открывает в браузере страницу, где пользователь рисует или выбирает область и сохраняет её (это создаёт проект); продолжай с estimate_download, когда пользователь скажет, что сохранено. Пути должны быть абсолютными или начинаться с ~. Одновременно идёт один конвейер. permits_for_area создаёт заметки о разрешениях для проекта (административные единицы, охраняемые территории, кто управляет землёй; подсказка, не юридическая консультация) как задачу вроде start_pipeline. check_update сообщает, есть ли более новый freemapper; install_update устанавливает его, только с confirm: true после согласия человека, и затем ассистент перезапускает сервер freemapper.

cli-contour-interval-saved = Сечение горизонталей сохранено в freemapper.toml: { $interval } м; derive трассирует горизонтали с ним
cli-contour-interval-auto = Сечение горизонталей в freemapper.toml: автоматически (по виду карты; лесная карта получает 2,5 м на равнине и 10 м в крутой местности)
help-arg-contour-interval = Сечение горизонталей в метрах, например `2.5`, или `auto` по виду карты (лесная карта: 5 м, 2,5 м где ровно, 10 м где круто). Сохраняется в freemapper.toml; derive трассирует горизонтали с ним
cli-content-saved = Настройка содержимого сохранена в freemapper.toml: { $summary }
cli-content-invalid = Не удаётся прочитать настройку содержимого: { $detail }
help-arg-content = Что попадает сразу на карту: `safe` оставляет каждый объект там, куда его кладёт сборка, `balanced` (по умолчанию) кладёт на карту объекты кадастра и реестров, `full` кладёт на карту всё. Выбор сохраняется в freemapper.toml
help-arg-content-set = Отправить группу объектов в `map`, `candidates` или `off`, как `GROUP=PLACE`, например `walls=map`; можно повторять. Группы: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Сохраняется в freemapper.toml
mcp-param-create-project-content = Что попадает сразу на карту: "safe", "balanced" (по умолчанию) или "full". Сохраняется в проекте. balanced кладёт на карту объекты кадастра и реестров, full кладёт на неё всё.
mcp-param-create-project-content-set = Группы, отправленные не туда, куда говорит профиль, как "group=place", где place это map, candidates или off, например "walls=map". Группы: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Сечение горизонталей в метрах, например 2.5; 0 возвращает автоматический выбор (по виду карты; лесная карта получает 2,5 м на равнине и 10 м в крутой местности). Сохраняется в проекте; следующий derive трассирует горизонтали с ним.
mcp-param-start-pipeline-content = Изменить перед запуском, что попадает сразу на карту: "safe", "balanced" или "full". Сохраняется в проекте. По умолчанию: настройка проекта, иначе balanced.
mcp-param-start-pipeline-content-set = Группы, отправленные не туда, куда говорит профиль, как "group=place", где place это map, candidates или off, например "fences=candidates". Сохраняется в проекте.
mcp-param-start-pipeline-contour-interval = Сечение горизонталей в метрах, например 2.5; 0 возвращает автоматический выбор (по виду карты; лесная карта получает 2,5 м на равнине и 10 м в крутой местности). Сохраняется в проекте; следующий derive трассирует горизонтали с ним.
mcp-error-content = Не удаётся прочитать настройку содержимого: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Записать заметки о разрешениях для проекта (permits.md и permits.json): административные единицы, охраняемые территории и владельцы, которых касается область карты, и к кому обычно приходится обращаться. Команда обращается к BfN и BKG и, если у федеральной земли области карты есть собственные открытые сервисы, к ним через хосты её адаптера (список в руководстве, глава 12); в Северном Рейне-Вестфалии в первый раз загружает для этого около 32 МБ, в Бранденбурге около 4 МБ, позже только более новые файлы. Она читает уже кэшированную выгрузку OSM; --offline использует то, что загружено раньше
help-permits-dir = Папка проекта
help-permits-offline = Использовать только ранее загруженные данные, без доступа к сети
help-run-permits = Дополнительно записать заметки о разрешениях (permits.md и permits.json) после карты; обращается к BfN, BKG и открытым сервисам федеральной земли (руководство, глава 12). Сбой там даёт предупреждение, а карта остаётся
cli-wrote-permits = Записаны заметки о разрешениях { $path }
cli-summary-wrote-permits = Записаны заметки о разрешениях { $path }
cli-permits-failed = заметки о разрешениях не удалось записать: { $error }
cli-permits-source-left-out = в заметках о разрешениях нет: { $source }: { $reason }
cli-permits-not-read = не читалось
error-permits-stored = { $path }: { $message }; загрузи заметки о разрешениях заново онлайн
error-permits-nothing-read = ни один источник заметок о разрешениях не удалось прочитать ({ $missing }); сначала загрузи их онлайн
error-permits-missing-not-read = { $source }: не читалось

permits-title = Заметки о разрешениях
permits-disclaimer = Подсказка, не юридическая консультация. Решают компетентные ведомства. Дату и точность данных см. в таблице под «Источники и дата данных»; области могут отсутствовать или устареть (BfN: сводится раз в год, "Nicht für Planungszwecke geeignet.", то есть не пригодно для планирования).
permits-disclaimer-regional = Подсказка, не юридическая консультация. Решают компетентные ведомства. Дату и точность данных см. в таблице под «Источники и дата данных»; области могут отсутствовать или устареть.
permits-bfn-merge = BfN раз в год сводит данные федеральных земель и помечает их как непригодные для планирования. Решают ведомства федеральной земли.
permits-created = Составлено { $date } для области карты ({ $kind ->
        [schoolyard] карта школьного двора
        [forest] лесная карта
       *[other] спринт-карта
    }, { $area } га).
permits-section-admin = Административные единицы
permits-section-protected = Охраняемые территории (BfN)
permits-section-osm-protected = Другие охраняемые территории из OpenStreetMap
permits-section-operators = Операторы и владельцы по OpenStreetMap
permits-section-hints = Подсказки
permits-section-sources = Источники и дата данных
permits-none-admin = VG250 не называет для области карты ни одной административной единицы.
permits-none-protected = По данным BfN, в области карты нет охраняемых территорий.
permits-none-osm-protected = OpenStreetMap не называет в области карты охраняемых территорий, которых BfN ещё не перечисляет.
permits-none-operators = OpenStreetMap не называет для области карты ни оператора, ни владельца.
permits-none-hints = Подсказок нет.
permits-section-not-read = Данные { $source } не читались, поэтому этот раздел пуст, хотя записи могут быть.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Административные границы 1:250 000 (VG250) от BKG
        [bfn] Охраняемые территории Германии от BfN
       *[osm] OpenStreetMap, из выгрузки, которую freemapper держит в кэше
    }
permits-level = { $level ->
        [land] Федеральная земля
        [kreis] Район (Kreis)
        [verwaltungsgemeinschaft] Amt или Verwaltungsgemeinschaft
       *[gemeinde] Муниципалитет (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Затронуто по краю (менее 0,5 % области карты):
permits-at-edge-generalised = VG250 обобщён до масштаба 1:250 000. Единица, которая лишь касается края, может быть следствием этого.
permits-kind = { $kind ->
        [nationalpark] Национальный парк
        [nationales-naturmonument] Национальный памятник природы
        [naturschutzgebiet] Природный заповедник (Naturschutzgebiet)
        [ffh] Территория FFH (Natura 2000)
        [vogelschutzgebiet] Территория охраны птиц (Natura 2000)
        [biosphaerenreservat] Биосферный заповедник
        [biosphaerenreservat-zone] Зона биосферного заповедника
        [landschaftsschutzgebiet] Охраняемый ландшафт (Landschaftsschutzgebiet)
       *[naturpark] Природный парк
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Объявлено { $date }
permits-protected-title = Правовая основа: { $title }
permits-protected-link = Текст закона: { $url }
permits-protected-site-link = Страница территории (Natura 2000): { $url }
permits-protected-year = Год данных в BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = класс охраны { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = без названия
permits-osm-operator = Оператор по OSM: { $operator }
permits-osm-enclosing-gap = Области OpenStreetMap, которые охватывают всю область карты без угла внутри неё, здесь могут отсутствовать, как и охраняемые территории, нанесённые в OSM как отношение с `type=boundary`. Части из BfN и VG250 это не затрагивает.
permits-osm-enclosing-gap-short = Здесь могут отсутствовать области OpenStreetMap, которые охватывают всю область карты, не имея ни одного угла внутри неё.
permits-operator-line = { $name } ({ $roles ->
        [operator] оператор
        [owner] владелец
       *[both] оператор и владелец
    }): { $objects }
permits-column-source = Источник
permits-column-state = Состояние
permits-column-credit = Авторство и лицензия
permits-state-read = прочитано, загружено { $date }
permits-state-read-osm = прочитано, выгрузка OSM от { $date }
permits-state-unreachable = недоступно ({ $reason })
permits-state-not-read = не читалось
permits-no-credit = не использовалось
permits-hint-city-state = Область карты лежит в Берлине или Гамбурге. Там обычно отвечает Bezirksamt (окружная управа) или ведомство федеральной земли. Эти заметки пока не определяют Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Охраняемые территории относятся к нижестоящему ведомству охраны природы (untere Naturschutzbehörde). Здесь это сам город { $kreis }.
        [no] Охраняемые территории относятся к нижестоящему ведомству охраны природы (untere Naturschutzbehörde), обычно { $kreis }.
       *[unknown] Охраняемые территории относятся к компетентному нижестоящему ведомству охраны природы (untere Naturschutzbehörde), обычно это Landkreis или kreisfreie Stadt.
    }
permits-hint-national-park = У национального парка свои правила. Обратись в администрацию национального парка заранее.
permits-hint-nsg-paths = В природном заповеднике обычно нужно держаться троп. Для бега вне троп обратись в ведомство охраны природы.
permits-hint-natura2000 = Территория FFH или территория охраны птиц (Natura 2000) затрагивает область карты. Для мероприятия ведомство охраны природы может потребовать оценку его воздействия.
permits-hint-protected-areas-pl = В области карты находятся охраняемые территории или объекты охраны природы. Для национального парка, ландшафтного парка или заповедника обычно обращаются в администрацию парка или заповедника; по остальным территориям и объектам, кто ответственен, обычно может сказать Региональная дирекция охраны окружающей среды (RDOŚ) воеводства. Спрашивай заранее, если там планируется мероприятие.
permits-hint-order-office = { $amt ->
        [none] За улицы, площади и другие общественные земли обычно отвечает Ordnungsamt (управление общественного порядка) Gemeinde { $gemeinde }.
       *[other] За улицы, площади и другие общественные земли обычно отвечает Ordnungsamt (управление общественного порядка): { $amt }, для Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Оператор парка или сада в области карты по OpenStreetMap: { $operator }. Спроси его, прежде чем использовать территорию.
permits-hint-school-operator = Оператор школы или учреждения по OpenStreetMap: { $operator }. По поводу территории обратись к директору школы или в этот орган.
permits-section-districts = Bezirke (округа, { $source })
permits-section-green-spaces = Зелёные зоны ({ $source })
permits-section-owners = Земли федеральной земли и Bund ({ $source })
permits-section-monuments = Памятники ({ $source })
permits-section-protected-objects = Памятники природы и другие охраняемые объекты ({ $source })
permits-none-districts = По данным { $source }, область карты не лежит ни в одном Bezirk.
permits-none-green-spaces = По данным { $source }, в области карты нет общественных зелёных зон.
permits-none-owners = По данным { $source }, в области карты нет участков федеральной земли или Bund.
permits-none-monuments = По данным { $source }, в области карты нет памятников.
permits-none-protected-objects = По данным { $source }, ни один из охраняемых объектов, которые он содержит, не лежит в области карты.
permits-none-sampled = В опрошенных точках { $source } здесь ничего не называет.
permits-gaps = Данные { $source } неполны: { $gaps }.
permits-sampled = { $source } отвечает только для отдельных точек. freemapper опросил сетку точек над картой: доли являются оценками, а то, что лежит между точками, может отсутствовать.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = назначение: { $dedication }
permits-playgrounds-none = В области карты нет общественных игровых площадок.
permits-playgrounds-one = В области карты лежит одна общественная игровая площадка. Отдельно она не перечисляется.
permits-playgrounds-many = { $count ->
        [one] В области карты лежит { $count } общественная игровая площадка. Они не перечисляются по одной.
        [few] В области карты лежат { $count } общественные игровые площадки. Они не перечисляются по одной.
        [many] В области карты лежат { $count } общественных игровых площадок. Они не перечисляются по одной.
       *[other] В области карты лежат { $count } общественные игровые площадки. Они не перечисляются по одной.
    }
permits-owner-other = Другие общественные владельцы
permits-owners-rest = Для остальной части области данные не называют владельца.
permits-monuments-total = Памятников целиком или частично в области карты: { $count }.{ $links ->
        [yes] {" "}Ссылки ведут на их записи в списке памятников.
       *[no] {""}
    }
permits-monuments-links-left = Ещё записей, которые здесь только подсчитаны, без ссылки: { $count }.
permits-object-kind = { $kind ->
        [natural-monument] Памятник природы (Naturdenkmal)
        [protected-landscape-part] Охраняемый элемент ландшафта (geschützter Landschaftsbestandteil)
        [biotope] Охраняемый законом биотоп (geschütztes Biotop)
        [national-park] Национальный парк (park narodowy)
        [park-buffer-zone] Буферная зона парка (otulina)
        [landscape-park] Ландшафтный парк (park krajobrazowy)
        [nature-reserve] Природный заповедник (rezerwat przyrody)
        [protected-landscape-area] Охраняемый ландшафт (obszar chronionego krajobrazu)
        [natura2000-habitats] Территория Natura 2000 для местообитаний (SOO)
        [natura2000-birds] Территория Natura 2000 для птиц (OSO)
        [ecological-site] Экологический участок (użytek ekologiczny)
        [landscape-complex] Природно-ландшафтный комплекс (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Документационный объект (stanowisko dokumentacyjne)
        [natural-monument-pl] Памятник природы (pomnik przyrody)
       *[special-rules] Территория с особыми правилами
    }
permits-object-count = Объектов: { $count }
permits-object-citations = Правовые основы:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] За улицы, площади и зелёные зоны в Bezirk { $district } обычно отвечает Bezirksamt { $district }, там Fachamt Management des öffentlichen Raumes (отдел управления общественным пространством).
       *[berlin] За улицы, площади и зелёные зоны в Bezirk { $district } обычно отвечают Ordnungsamt (управление общественного порядка) и Straßen- und Grünflächenamt (управление дорог и зелёных зон) Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] За памятники в области карты отвечает Denkmalschutzamt Hamburg (управление охраны памятников).
       *[berlin] За памятники в Bezirk { $district } отвечает нижестоящее ведомство охраны памятников (untere Denkmalschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Лежат ли в области карты охраняемые памятники, имеющиеся здесь данные точно сказать не могут.
       *[yes] В области карты есть охраняемые памятники.
    } Затрагивает ли бег их, например пункт у памятника или участок через охраняемое место, решает нижестоящее ведомство охраны памятников (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Здесь это сам город { $kreis }.
        [no] , обычно { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; в некоторых федеральных землях это Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(обычно Landkreis или kreisfreie Stadt, в некоторых федеральных землях Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Памятники природы, охраняемые элементы ландшафта и охраняемые биотопы на карте относятся к нижестоящему ведомству охраны природы (untere Naturschutzbehörde). Здесь это сам город { $kreis }.
        [no] Памятники природы, охраняемые элементы ландшафта и охраняемые биотопы на карте относятся к нижестоящему ведомству охраны природы (untere Naturschutzbehörde), обычно { $kreis }.
       *[unknown] Памятники природы, охраняемые элементы ландшафта и охраняемые биотопы на карте относятся к нижестоящему ведомству охраны природы (untere Naturschutzbehörde), обычно Landkreis или kreisfreie Stadt.
    }
permits-hint-biotope = В области карты лежат охраняемые законом биотопы{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Всё, что может их уничтожить или существенно повредить, запрещено (§ 30 BNatSchG). Ставь пункты и участки дистанции вне них или спроси нижестоящее ведомство охраны природы (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] За памятники природы и охраняемые элементы ландшафта в Bezirk { $district } обычно отвечает Bezirksamt { $district }.
       *[berlin] За памятники природы и охраняемые элементы ландшафта в Bezirk { $district } отвечает нижестоящее ведомство охраны природы (untere Naturschutzbehörde) Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] По открытым данным федеральной земли, { $share } области карты принадлежит другим общественным владельцам.
       *[no] По открытым данным федеральной земли, { $share } области карты принадлежит владельцу "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Владельцев остальной части области по открытым данным узнать нельзя. При законном интересе (например, у организатора) можно запросить выписку в земельной книге (Grundbuchamt); для общественных земель поможет компетентное Ordnungsamt или управление парков.
       *[no] Владельцев этих участков по открытым данным узнать нельзя. При законном интересе (например, у организатора) можно запросить выписку в земельной книге (Grundbuchamt); для общественных земель поможет компетентное Ordnungsamt или управление парков.
    }
permits-section-forest-units = Лесное управление ({ $source })
permits-section-state-forest = Лес федеральной земли ({ $source })
permits-none-forest-units = По данным { $source }, область карты не лежит в районе ни одного лесничества.
permits-section-admin-units = Административные единицы ({ $source })
permits-none-admin-units = Согласно { $source } территория карты не входит ни в одну административную единицу реестра.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = По данным { $source }, в области карты нет леса федеральной земли.
permits-forest-units-reach = Лесничество отвечает за лес в своём районе. Его район может охватывать также города и поля.
permits-state-forest-office = отвечает: { $office }
permits-hint-forest-office = { $units ->
        [none] Лесничество, отвечающее за лес в области карты: { $office } ({ $administration }). Обратись туда заранее, если планируешь мероприятие в лесу.
       *[other] Лесничество, отвечающее за лес в области карты: { $office } ({ $administration }), { $units }. Обратись туда заранее, если планируешь мероприятие в лесу.
    }
permits-hint-forest-owner = { $office ->
        [known] Для мероприятия в лесу нужно согласие владельцев леса. Лесничество обычно может сказать, кому принадлежит лес. В нескольких федеральных землях организованное мероприятие в лесу нужно также сообщить лесному ведомству или получить его разрешение.
       *[unknown] Для мероприятия в лесу нужно согласие владельцев леса. В нескольких федеральных землях организованное мероприятие в лесу нужно также сообщить лесному ведомству или получить его разрешение. Эти заметки не называют лесничество, отвечающее здесь; подскажет лесное управление федеральной земли или нижестоящее лесное ведомство.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Для мероприятия в лесу обычно нужно согласие владельца леса. Названное выше лесничество (Nadleśnictwo) обычно может сказать, кто заботится о лесе. Лес, не принадлежащий Государственным лесам, например частный лес, в данных этих заметок отсутствует.
       *[unknown] Для мероприятия в лесу обычно нужно согласие владельца леса. Эти заметки не называют для области карты ни одного лесничества Государственных лесов; данные охватывают только Государственные леса, поэтому лес может принадлежать кому-то другому, например частному владельцу. Кто владелец, нужно выяснять отдельно.
    }
permits-hint-gmina-nature-pl = { $gmina ->
        [none] В области карты есть памятник природы (pomnik przyrody), экологический участок (użytek ekologiczny), природно-ландшафтный комплекс (zespół przyrodniczo-krajobrazowy) или документационный объект (stanowisko dokumentacyjne). Их учреждает совет гмины решением, в котором указано, кто осуществляет надзор за объектом и какие запреты действуют (ст. 44 Закона об охране природы). Сам закон не предусматривает здесь никакого разрешения на мероприятие. Запросите решение и сведения о лице, осуществляющем надзор, в управе гмины, на территории которой находится объект.
       *[other] В области карты есть памятник природы (pomnik przyrody), экологический участок (użytek ekologiczny), природно-ландшафтный комплекс (zespół przyrodniczo-krajobrazowy) или документационный объект (stanowisko dokumentacyjne). Их учреждает совет гмины решением, в котором указано, кто осуществляет надзор за объектом и какие запреты действуют (ст. 44 Закона об охране природы). Сам закон не предусматривает здесь никакого разрешения на мероприятие. Запросите решение и сведения о лице, осуществляющем надзор, в управе гмины ({ $gmina }), на территории которой находится объект.
    }
permits-hint-gmina-roads-pl = Мероприятие, которое мешает движению или особым образом использует улицу, требует разрешения по ст. 65 Закона о дорожном движении; заявку подают не позднее чем за 30 дней. Разрешение выдаёт орган, управляющий движением на дороге: староста для дорог повята и гмины, мэр в городе с правами повята, маршал воеводства или GDDKiA для остальных дорог (ст. 10 и 65b). Управляет дорогой гмины wójt, burmistrz или prezydent miasta гмины ({ $gmina }); он решает вопрос о занятии полосы отвода дороги (ст. 19 и 40 Закона об общественных дорогах). Эти заметки не показывают категорию дорог.
permits-hint-gmina-land-pl = Если земля в области карты принадлежит гмине, например городской парк, площадь, школьная территория или лес, ею управляет wójt, burmistrz или prezydent miasta, который представляет гмину (ст. 30 и 31 Закона о самоуправлении гмины). Перед использованием запросите согласие в управе гмины ({ $gmina }). Эти заметки не показывают, кому принадлежит земля.
permits-section-water-protection = Зоны охраны вод ({ $source })
permits-section-forest-functions = Функции леса и охраняемый лес ({ $source })
permits-none-water-protection = По данным { $source }, в области карты нет зон охраны вод.
permits-none-forest-functions = По данным { $source }, в области карты нет рекреационного леса, охраняемого леса, природного леса и закрытой лесной территории.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Рекреационный лес (Erholungswald)
        [protected] Охраняемый лес (Bannwald или Schutzwald)
        [natural] Природный лес или строгий заповедник (Naturwald, Totalreservat)
       *[closed] Закрытая лесная территория
    }
permits-hint-water-protection = { $inner ->
        [yes] { $zone } зоны охраны вод "{ $name }" лежит в области карты. В зонах I и II мероприятия обычно исключены. Обратись в нижестоящее водное ведомство (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] { $zone } зоны охраны вод "{ $name }" лежит в области карты. Мероприятия там обычно возможны; нижестоящее водное ведомство (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) скажет, что разрешает постановление.
       *[unknown] Зона охраны вод "{ $name }" лежит в области карты. Данные не говорят, какая это зона. В зонах I и II мероприятия обычно исключены; обратись в нижестоящее водное ведомство (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] В области карты лежит рекреационный лес (Erholungswald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. По поводу мероприятий там обратись в лесное ведомство.
        [protected] В области карты лежит охраняемый лес (Bannwald или Schutzwald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Спроси лесное ведомство, прежде чем планировать там мероприятие.
        [natural] В области карты лежит природный лес или строгий заповедник{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Сходить с троп там часто запрещено; не ставь в нём пункты, не спросив лесное ведомство.
       *[closed] В области карты лежит лесная территория, куда нельзя заходить{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Не включай её в дистанцию.
    }
permits-section-roads = Дороги с классом по OpenStreetMap
permits-none-roads = По данным OpenStreetMap, через область карты не проходят ни Autobahn, ни Bundes-, Landes-, Staats- или Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (автомагистраль)
        [bundesstrasse] Bundesstraße (федеральная дорога)
        [landesstrasse] Landesstraße или Staatsstraße (дорога федеральной земли)
       *[kreisstrasse] Kreisstraße (районная дорога)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Через область карты проходит Autobahn { $refs }. Дистанция может пересекать её только по мостам или через подземные переходы.
        [bundesstrasse] Через область карты проходит Bundesstraße { $refs }. Перекрытие или пересечение с маршалами назначает ведомство дорожного движения (Straßenverkehrsbehörde); дорогу обычно содержит Bund через дорожное управление федеральной земли, в крупных городах часто город.
        [landesstrasse] Через область карты проходит Landes- или Staatsstraße { $refs }. Перекрытие или пересечение с маршалами назначает ведомство дорожного движения (Straßenverkehrsbehörde); дорогу обычно содержит федеральная земля, в крупных городах часто город.
       *[kreisstrasse] Через область карты проходит Kreisstraße { $refs }. Перекрытие или пересечение с маршалами назначает ведомство дорожного движения (Straßenverkehrsbehörde); дорогу обычно содержит Kreis, в крупных городах часто город.
    }
permits-hint-state-forest = { $office ->
        [none] По открытым данным федеральной земли, { $share } области карты занимает лес федеральной земли владельца "{ $owner }".
       *[other] По открытым данным федеральной земли, { $share } области карты занимает лес федеральной земли владельца "{ $owner }". Отвечает: { $office }.
    }
