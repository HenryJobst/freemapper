# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = предупреждение: { $text }

warning-other-zone = область карты лежит в зоне UTM { $zone } (EPSG:{ $epsg }); freemapper загружает всё заново в этой зоне
warning-other-state = область карты теперь лежит в данных другой федеральной земли; freemapper загружает всё заново
warning-beyond-coverage = часть области карты лежит за пределами поддерживаемых федеральных земель. Она находится за границей, в море или в федеральной земле, которую freemapper пока не поддерживает, например в Тюрингии. Данных там у земель нет, поэтому ортофотоплан может быть белым, а ALKIS не даёт ни зданий, ни землепользования
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: масштаб 1:{ $scale } не входит в масштабы для карт вида { $kind } ({ $allowed }); freemapper-print отклонит карту
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: запись { $key } слоя { $german ->
       *[any] { $what }
    } проигнорирована: { $reason ->
        [name] это не простое имя файла
       *[unknown] такого слоя freemapper не знает
    }

## Disk space

warning-disk-space-unreadable = не удаётся прочитать свободное место на диске для { $path } ({ $error }), поэтому freemapper его не проверяет

## Tiles

warning-tile-changed = плитка { $tile } ({ $product }) изменилась у поставщика ({ $old } → { $new })
warning-bdom-tile-unavailable = модель поверхности (bDOM) плитки { $tile } недоступна ({ $error }); подложки nDSM там нет
warning-bdom-unavailable = модель поверхности (bDOM) недоступна ({ $error }); подложки nDSM для { $tiles } нет
progress-ndsm-from-bdom = nDSM плитки { $tile } из модели поверхности (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] модель рельефа
        [surface] модель поверхности
       *[orthophoto] ортофотоплан
    } ({ $name })
progress-model-download = загрузка: { $model }
progress-model-read = чтение: { $model }
warning-model-unavailable = { $model }: недоступно ({ $error }); подложки nDSM нет
warning-model-unavailable-kept = { $model }: недоступно ({ $error }); оставлена модель поверхности, загруженная { $fetched }
warning-model-tile-unavailable = { $model }, плитка { $tile }: недоступно ({ $error }); подложки nDSM там нет
warning-archive-passed-over = { $model }: архив не удалось прочитать ({ $error }); он пропущен, плитки области карты берутся из других архивов
warning-archive-directory-kept = { $model }: архив не удалось прочитать ({ $error }); используется его оглавление, сохранённое с прошлого запуска, поэтому плитка, которую LVGL с тех пор заменил, не будет замечена
warning-model-tiles-missing = { $model }: нет { $count ->
        [one] плитки
        [few] плиток
        [many] плиток
       *[other] плиток
    } { $names } в пределах { $state ->
        [the-map-area] области карты
        [hamburg] Гамбурга
        [hesse] Гессена
        [north-rhine-westphalia] Северного Рейна-Вестфалии
        [schleswig-holstein] Шлезвиг-Гольштейна
        [mecklenburg-vorpommern] Мекленбурга-Передней Померании
        [lower-saxony] Нижней Саксонии
        [bavaria] Баварии
       *[other] { $state }
    }; подложки там остаются пустыми
warning-no-surface-stored = модель поверхности не сохранена; для подложки nDSM выполни `freemapper fetch` онлайн
warning-surface-older-than-orthophoto = модель поверхности (bDOM { $surface }) старше ортофотоплана ({ $photo }); в nDSM могут отсутствовать новые здания
progress-surface-kept = freemapper оставляет для подложки nDSM модель поверхности, загруженную { $fetched }
progress-surface-tiles-kept = freemapper оставляет для подложки nDSM кэшированную модель поверхности { $count ->
        [one] плитки
        [few] плиток
        [many] плиток
       *[other] плиток
    } { $names }
progress-orthophoto-unchanged = ортофотоплан не изменился
progress-orthophoto-unchanged-dated = ортофотоплан не изменился ({ $label } { $date })
progress-orthophoto-download = загрузка ортофотоплана с { $provider } { $service }
progress-orthophoto-in-leaf = ортофотоплан снят { $date }, деревья в листве; их кроны скрывают то, что под ними
progress-orthophoto-bare = ортофотоплан снят { $date }, деревья без листвы
warning-lgln-orthophoto-kept = ортофотоплан (LGLN WMS): { $reason }; оставлен сохранённый ортофотоплан и дата его съёмки { $date }
warning-lgln-orthophoto-undated = ортофотоплан (LGLN WMS): нет даты съёмки, { $reason }
word-stac-no-image = STAC не перечисляет ни одного снимка в центре области карты
word-stac-no-answer = поиск STAC не дал пригодного ответа ({ $error })
warning-downloadcenter-no-file = в Downloadcenter нет файла для { $places }
warning-downloadcenter-no-file-empty = в Downloadcenter нет файла для { $places }; { $count ->
        [one] плитка
        [few] плитки
        [many] плиток
       *[other] плитки
    } { $names } { $count ->
        [one] осталась пустой
       *[other] остались пустыми
    }
progress-k-five-unchanged = подложка K5 не изменилась
progress-k-five-download = загрузка карты K5 (1:5000) с WMS Geoportal Berlin
warning-k-five-unavailable = подложка карты K5 недоступна ({ $detail }); пропущена

## The state border

progress-beyond-border = область карты выходит за границу федеральной земли; за ней подложки пусты там, где заканчиваются ортофотоплан и модели высот { $provider }. ALKIS и OSM берутся из обеих земель
progress-beyond-border-no-data = область карты выходит за границу федеральной земли; данных { $provider } для { $tiles } нет, поэтому подложки там остаются пустыми

## OSM

warning-osm-extract-missing-kept = выгрузки OSM в кэше нет; остаются сохранённые кандидаты OSM от { $date }
progress-osm-extract-read = чтение выгрузки OSM { $region } от { $date }
progress-osm-candidates = объектов-кандидатов OSM: { $count }

## Permit notes

progress-permits-fetch = загрузка данных { $source } для заметок о разрешениях
progress-permits-osm = чтение выгрузки OSM { $region } от { $date } для заметок о разрешениях
progress-permits-stored = чтение сохранённых данных заметок о разрешениях
warning-permits-source-unreachable = { $source } не удалось прочитать, поэтому в заметках о разрешениях его нет: { $error }
warning-permits-osm-no-extract = для этой области карты в кэше нет выгрузки OSM, поэтому в заметках о разрешениях нет OSM; `freemapper fetch` загружает её

## ALKIS

warning-alkis-coverage-unknown = покрытие зданиями { $cadastre } неизвестно; выполни `freemapper fetch` онлайн, чтобы его записать
progress-alkis-buildings-download = загрузка зданий { $cadastre }
progress-alkis-buildings = зданий { $cadastre }: { $count }
warning-alkis-no-buildings = в области карты не найдено зданий { $cadastre }
progress-alkis-skipped = пропущено объектов { $cadastre }: { $count }: { $list }
warning-alkis-buildings-changed = здания { $cadastre } изменились у поставщика ({ $changes })
progress-alkis-no-land-use-stored = землепользование { $cadastre } не сохранено; для кандидатов закрытых зон выполни `freemapper fetch` онлайн
progress-alkis-land-use-download = загрузка землепользования { $cadastre }
progress-alkis-land-use-parts = частей землепользования { $cadastre }: { $count }
warning-alkis-land-use-kept = землепользование { $cadastre } недоступно ({ $error }); остаётся сохранённое землепользование
warning-alkis-land-use-removed = землепользование { $cadastre } недоступно ({ $error }); сохранённое землепользование больше не покрывает область карты, поэтому freemapper его удалил
warning-alkis-land-use-none = землепользование { $cadastre } недоступно ({ $error }); кандидатов закрытых зон нет
progress-bdot-filled = добавлено { $count } объектов BDOT10k ({ $what }) там, где EGiB ничего не даёт ({ $powiat })
warning-bdot-failed = BDOT10k не смог заполнить пробел в EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB недоступен ({ $error }); вместо него используется BDOT10k ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = берлинские структуры ALKIS не сохранены; для лестниц, стен, деревьев и навесов выполни `freemapper fetch` онлайн
progress-berlin-removing = удаление сохранённых берлинских слоёв: область карты больше не рядом с Берлином
progress-berlin-download = загрузка берлинских структур ALKIS, кадастра деревьев, съёмки улиц и точечных слоёв
progress-berlin-features = объектов Берлина { $file }: { $count }
warning-berlin-layer-kept = Берлин { $file } недоступен ({ $detail }); остаётся сохранённый слой
warning-berlin-layer-skipped = Берлин { $file } недоступен ({ $detail }); слой пропущен
progress-feinkartierung-none-stored = детальная съёмка Гамбурга (Feinkartierung) не сохранена; для ограждений, стен, лестниц и уличного оборудования выполни `freemapper fetch` онлайн
progress-feinkartierung-removing = удаление сохранённой детальной съёмки Гамбурга (Feinkartierung): область карты больше не в Гамбурге
progress-feinkartierung-download = загрузка детальной съёмки Гамбурга (Feinkartierung; уличное пространство и общественная зелень)
progress-feinkartierung-features = объектов Feinkartierung { $file }: { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file } недоступна ({ $detail }); остаётся сохранённый слой
warning-feinkartierung-layer-skipped = Feinkartierung { $file } недоступна ({ $detail }); слой пропущен
error-extra-layer-unknown = не слой { $folder }
warning-berlin-buildings-missing = для этой области в Берлине не сохранены здания ALKIS; выполни `freemapper fetch`, чтобы получить их из Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = полные структуры ALKIS не сохранены; для стен, заборов, живых изгородей и мостов выполни `freemapper fetch` онлайн
progress-nas-removing = удаление сохранённых полных слоёв ALKIS: область карты больше не в федеральной земле с сервисом NAS
warning-nas-folder-outside = полные слои ALKIS пропущены ({ $folder } выводит за пределы папки проекта)
progress-nas-download = загрузка полных структур ALKIS (NAS)
progress-nas-features = объектов ALKIS NAS { $file }: { $count }
progress-nas-skipped = пропущено объектов ALKIS NAS { $file }: { $count }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } недоступен ({ $detail }); остаётся сохранённый слой
warning-nas-layer-skipped = ALKIS NAS { $file } недоступен ({ $detail }); слой пропущен

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = объекты ATKIS Basis-DLM не сохранены; для родников, пещер, башен, памятников и фонтанов выполни `freemapper fetch` онлайн
progress-dlm-removing = удаление сохранённых слоёв ATKIS Basis-DLM: область карты больше не в федеральной земле, которая его предоставляет
warning-dlm-folder-outside = слои ATKIS Basis-DLM пропущены ({ $folder } выводит за пределы папки проекта)
progress-dlm-download = загрузка объектов ATKIS Basis-DLM
progress-dlm-features = объектов ATKIS Basis-DLM { $file }: { $count }
progress-dlm-skipped = пропущено объектов ATKIS Basis-DLM { $file }: { $count }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } недоступен ({ $detail }); остаётся сохранённый слой
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } недоступен ({ $detail }); слой пропущен

## Tree cadastres

progress-trees-none-stored = городской кадастр деревьев не сохранён; для отдельных деревьев выполни `freemapper fetch` онлайн
progress-trees-removing = удаление сохранённых кадастров деревьев: область карты больше не в городе, у которого он есть
warning-trees-folder-outside = городские кадастры деревьев пропущены ({ $folder } выводит за пределы папки проекта)
progress-trees-download = загрузка городских кадастров деревьев
progress-trees-count = деревьев { $id }: { $count }
progress-trees-skipped = пропущено деревьев { $id }: { $count }: { $list }
warning-trees-kept = кадастр деревьев { $id } недоступен ({ $detail }); остаётся сохранённый
warning-trees-skipped = кадастр деревьев { $id } недоступен ({ $detail }); деревья пропущены

## Deriving the templates

progress-writing = запись { $path }
progress-contours-steep = горизонтали через { $interval } м: рельеф настолько крутой, что горизонтали через { $default } м слились бы; сечение { $default } м в проекте сохраняет их (`--contour-interval { $default }`)
warning-osm-only-no-terrain = для этого региона пока нет данных рельефа: карта делается только из OpenStreetMap, без горизонталей, подложек и зданий
progress-no-ndsm-recorded = nDSM не записан; для подложки nDSM выполни `freemapper fetch`
warning-ndsm-tile-invalid = недопустимая плитка nDSM { $tile } в freemapper.toml; пропущена
warning-ndsm-not-cached = nDSM { $tile } нет в кэше; выполни `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } не читается ({ $error }); удали { $path } и выполни `freemapper fetch`
warning-surface-unusable = модель поверхности непригодна ({ $error }); выполни `freemapper fetch`

## Downloads and the cache

progress-downloading = загрузка { $url }
progress-retry-wait = сервис занят; следующая попытка через { $seconds } с
progress-downloaded = загружено { $megabytes } МБ
progress-downloaded-of = загружено { $megabytes } МБ из { $total } МБ
progress-tile-index-download = загрузка индекса плиток { $product }
progress-tile-cached = в кэше { $tile }
progress-surface-blocks = чтение { $count } из { $total } блоков плитки модели поверхности { $tile } ({ $megabytes } МБ)
progress-surface-whole = плитка модели поверхности { $tile } загружается целиком: { $reason }
warning-tile-cached-older = используется кэшированная плитка { $tile } ({ $product }) от { $published } (более новая от { $newer } не загружена)
warning-ndsm-no-cached = в кэше нет nDSM для плитки { $tile }
warning-ndsm-tile-unavailable-cached = плитка nDSM { $tile } недоступна ({ $reason }); используется файл из кэша
warning-ndsm-tile-unavailable-skipped = плитка nDSM { $tile } недоступна ({ $reason }); пропущена
warning-file-check-failed = не удалось проверить, актуален ли ещё { $file } ({ $reason }); используется файл из кэша и другие файлы из кэша этого сервера без вопросов
warning-ldbv-no-buildings-file = на сервере нет файла зданий для плитки { $tile }; на карте там могут отсутствовать здания
progress-ndsm-not-published = для плитки { $tile } не опубликовано облако точек nDSM
progress-ndsm-cached = nDSM { $tile } в кэше
progress-point-cloud-not-published = для плитки { $tile } не опубликовано лазерное облако точек
progress-point-cloud-cached = облако точек { $tile } в кэше
warning-point-cloud-no-cached = в кэше нет облака точек для плитки { $tile }; кандидатов растительности там нет
warning-point-cloud-unavailable-cached = облако точек { $tile } недоступно ({ $reason }); используется файл из кэша
warning-point-cloud-unavailable-skipped = облако точек { $tile } недоступно ({ $reason }); кандидатов растительности там нет
warning-point-clouds-unavailable = лазерные облака точек недоступны ({ $reason }); кандидаты растительности используют кэшированные плитки, если они есть
progress-no-point-clouds = лазерных облаков точек для растительности здесь пока нет: федеральная земля не публикует таких, которые freemapper мог бы прочитать, или не в системе координат карты
warning-point-clouds-not-fetched = растительность выбрана, но облака точек не загружены; выполни `freemapper fetch`, чтобы загрузить их
warning-point-cloud-unreadable = облако точек { $tile } не читается ({ $error }); удали { $path } и выполни `freemapper fetch`
warning-osm-nrw-nearest = в пределах 100 м от области нет выгрузки округа Северного Рейна-Вестфалии; читается ближайшая, { $district }
warning-osm-disk-full-cached = диск заполнен, поэтому freemapper использует кэшированную выгрузку OSM от { $date }. Освободи место и выполни `freemapper fetch` снова, чтобы получить более новую
warning-osm-geofabrik-unavailable = Geofabrik недоступен ({ $error }); используется кэшированная выгрузка OSM от { $date }
progress-osm-extract-cached = выгрузка OSM { $region } от { $date } в кэше
progress-osm-extract-download = загрузка { $url } (около { $megabytes } МБ)

## Schleswig-Holstein

progress-sh-bdom-download = загрузка плитки bDOM { $tile } ({ $megabytes } МБ, { $index } из { $total })
progress-sh-dgm-not-in-atom = плитки DGM1 { $tile } нет в каталоге INSPIRE ATOM; загрузка её XYZ ({ $megabytes } МБ)
progress-sh-dgm-atom-unreadable = плитка DGM1 { $tile } из каталога INSPIRE ATOM не читается ({ $error }); загрузка её XYZ ({ $megabytes } МБ)
progress-sh-tile = плитка { $tile }: { $text }
progress-sh-download-again = плитка { $tile }: загрузка не читается ({ $message }); загружается ещё раз
warning-sh-portal-no-date = Downloadportal не даёт даты для плитки { $tile } ({ $model })
warning-sh-dgm-xyz-used = плитки DGM1 { $tile } нет в каталоге INSPIRE ATOM; использован её XYZ из Downloadportal
warning-sh-dgm-xyz-used-unreadable = плитка DGM1 { $tile } из каталога INSPIRE ATOM не читается ({ $error }); использован её XYZ из Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, плитка { $tile }: у { $count } ячеек области карты нет высоты
warning-centre-tile-missing = { $model }: плитка { $tile } в центре области карты отсутствует, поэтому даты нет
warning-centre-tile-unlisted = список плиток не называет плитку { $tile } ({ $model }) в центре области карты, поэтому даты нет
warning-list-unreadable-kept = список плиток ({ $model }) не удалось прочитать ({ $error }); использован список, сохранённый с прошлого запуска
warning-portal-list-unreadable-kept = список плиток Downloadportal ({ $model }) не удалось прочитать ({ $error }); использован список, сохранённый с прошлого запуска
warning-surface-much-older = { $model } от { $surface }, ортофотоплан от { $flight }. В nDSM могут отсутствовать более новые здания и деревья.

## Rhineland-Palatinate

warning-rp-list-features-unusable = у { $count } объектов списка плиток нет пригодного ключа плитки; пропущены
warning-rp-list-unreadable-kept = список плиток LVermGeo RP не удалось прочитать ({ $error }); использован список, сохранённый с прошлого запуска
warning-rp-metalink-not-kept = Metalink LVermGeo RP не сохранён: { $error }
warning-rp-metalink-unreadable-kept = Metalink LVermGeo RP ({ $model }) не удалось прочитать ({ $detail }); использован Metalink, сохранённый с прошлого запуска
warning-rp-metalink-unreadable = Metalink LVermGeo RP ({ $model }) не удалось прочитать ({ $detail }), поэтому плитки, для которых список плиток не даёт года съёмки, остаются без высот
warning-rp-tile-without-name = { $model }, плитка { $tile }: у этого квадрата 1 км на карте нет высот. Список плиток LVermGeo RP (Laserbefliegung) { $note }, поэтому имя файла неизвестно{ $either ->
        [metalink] , и Metalink тоже не называет для неё файла
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }: у плитки { $tile } нет даты: список плиток LVermGeo RP (Laserbefliegung) { $note }
word-rp-list-neither = не называет для неё ни года съёмки, ни дней съёмки
word-rp-list-no-days = не называет для неё дней съёмки
word-rp-list-no-year = не называет для неё года съёмки
word-rp-list-days-unreadable = называет для неё дни съёмки { $days }, которые не читаются
word-rp-list-disagree = называет для неё дни съёмки { $days } и год съёмки { $year }, которые не сходятся на одном году

## Hesse

warning-hvbg-undated = в списке метаданных нет читаемой даты залёта (Bildflugdatum) для плитки { $tiles }; записи проекта для неё не содержат даты съёмки
warning-hvbg-undated-more = в списке метаданных нет читаемой даты залёта (Bildflugdatum) для плитки { $tiles } и ещё { $more }; записи проекта для них не содержат даты съёмки
warning-hvbg-list-unreadable-older = новый список метаданных HVBG { $name } не удалось прочитать ({ $detail }); используется более старый список { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: нет даты, STAC не перечисляет ни одной плитки в центре области карты
warning-lgln-no-date = { $model }: нет даты ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; у сохранённой модели остаётся её записанная дата
warning-mv-cells-empty = { $model }: у { $count } ячеек внутри плиток Мекленбурга-Передней Померании нет высоты
warning-mv-centre-no-date = { $model }: нет даты для центральной плитки { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = у { $count } строк списка плиток ({ $model }) нет пригодного имени плитки; пропущены
warning-nrw-list-rows-undated = у { $count } строк списка плиток ({ $model }) нет читаемой даты
warning-nrw-list-older = список плиток ({ $model }) на сервере датирован { $server }, старше сохранённого от { $kept }; использован сохранённый
warning-nrw-list-no-date = список плиток не даёт даты для плитки { $tile } ({ $model })

## After `init`

note-scale-outside-norm = примечание: 1:5000 нет в норме спринта; freemapper-print предупреждает об этом, а символы остаются в 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } не является идентификатором OSM вида way/992423119
error-place-invalid-answer = недопустимый ответ поиска места: { $detail }
error-place-not-found = { $id } не найден. Nominatim перечисляет только объекты OSM с тегами, поэтому внешней линии мультиполигона без тегов там нет. Используй вместо неё отношение мультиполигона (--osm relation/<id>) или сохрани контур как GeoJSON и используй --geojson
error-place-is-a-point = { $id } - это точка, а не область; выбери её территорию (way или relation)
error-place-no-such-part = { $id } { $count ->
        [one] имеет { $count } часть
        [few] имеет { $count } части
        [many] имеет { $count } частей
       *[other] имеет { $count } части
    }; части { $part } нет
error-place-several-parts = { $id } - область из { $count } отдельных частей ({ $sizes }). Область карты - один многоугольник: выбери часть через --part <n> (1 самая большая) или сохрани нужный контур как GeoJSON и используй --geojson
error-layer-not-a-world-file = { $path }: не world-файл
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = для { $layer } нет предпросмотра; слои с предпросмотром: { $names }
error-layer-no-ndsm-published = для плиток этой области карты nDSM не опубликован, поэтому подложки ndsm нет
error-layer-not-made = { $layer } ещё не создан; выполни { $step ->
        [fetch] freemapper fetch или run (MCP: start_pipeline); только Берлин
        [export] freemapper export или run (MCP: start_pipeline)
       *[derive] freemapper derive или run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; карта OCAD не получает из этого фона
warning-map-preview-failed = карта записана, но её предпросмотра нет: { $error }
warning-training-no-tree-cover = тренировочная карта оставляет открытые участки парков целыми: отсутствует покрытие деревьями (data/tree-cover.tif); выполни `freemapper derive` снова
