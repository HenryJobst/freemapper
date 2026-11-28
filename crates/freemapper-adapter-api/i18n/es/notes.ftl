# Líneas de progreso y avisos del motor (ADR 0092, etapa 3). El inglés es la referencia.
# Ids: progress- para líneas de progreso, warning- para avisos, word- para nombres dentro de
# otros mensajes. Un aviso recibe su prefijo de note-warning.

note-warning = aviso: { $text }

warning-other-zone = el área del mapa está en la zona UTM { $zone } (EPSG:{ $epsg }); freemapper vuelve a descargar todo en esa zona
warning-other-state = el área del mapa está ahora en los datos de otro estado federado; freemapper vuelve a descargar todo
warning-beyond-coverage = una parte del área del mapa queda fuera de los estados federados admitidos. Está en el extranjero, en el mar o en un estado que freemapper aún no admite, como Turingia. Los estados no tienen datos allí, así que la ortofoto puede salir en blanco y ALKIS no da edificios ni usos del suelo
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: la escala 1:{ $scale } no es ninguna de las escalas de los mapas de { $kind } ({ $allowed }); freemapper-print rechazará el mapa
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: se ignoró la entrada { $key } de la capa { $german ->
       *[any] { $what }
    }: { $reason ->
        [name] no es un nombre de archivo simple
       *[unknown] no es una capa que freemapper conozca
    }

## Disk space

warning-disk-space-unreadable = no se puede leer el espacio libre en disco de { $path } ({ $error }), así que freemapper no lo comprueba

## Tiles

warning-tile-changed = la tesela { $tile } de { $product } cambió en el origen ({ $old } → { $new })
warning-bdom-tile-unavailable = modelo de superficie (bDOM) de la tesela { $tile } no disponible ({ $error }); allí no hay plantilla nDSM
warning-bdom-unavailable = modelo de superficie (bDOM) no disponible ({ $error }); no hay plantilla nDSM para { $tiles }
progress-ndsm-from-bdom = nDSM de la tesela { $tile } a partir del modelo de superficie (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] modelo del terreno
        [surface] modelo de superficie
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = descargando { $model }
progress-model-read = leyendo { $model }
warning-model-unavailable = { $model } no disponible ({ $error }); no hay plantilla nDSM
warning-model-unavailable-kept = { $model } no disponible ({ $error }); se conserva el modelo de superficie descargado el { $fetched }
warning-model-tile-unavailable = { $model } de la tesela { $tile } no disponible ({ $error }); allí no hay plantilla nDSM
warning-archive-passed-over = { $model }: no se pudo leer un archivo ({ $error }); se omitió y las teselas del área del mapa salen de los demás archivos
warning-archive-directory-kept = { $model }: no se pudo leer un archivo ({ $error }); se usa su directorio guardado de una ejecución anterior, así que no se nota una tesela que el LVGL haya sustituido desde entonces
warning-model-tiles-missing = { $model } no tiene { $count ->
        [one] la tesela
        [many] las teselas
       *[other] las teselas
    } { $names } { $state ->
        [the-map-area] dentro del área del mapa
        [hamburg] dentro de Hamburgo
        [hesse] dentro de Hesse
        [north-rhine-westphalia] dentro de Renania del Norte-Westfalia
        [schleswig-holstein] dentro de Schleswig-Holstein
        [mecklenburg-vorpommern] dentro de Mecklemburgo-Pomerania Occidental
        [lower-saxony] dentro de Baja Sajonia
        [bavaria] dentro de Baviera
       *[other] dentro de { $state }
    }; las plantillas quedan vacías allí
warning-no-surface-stored = no hay ningún modelo de superficie guardado; ejecuta `freemapper fetch` con conexión para la plantilla nDSM
warning-surface-older-than-orthophoto = el modelo de superficie (bDOM { $surface }) es más antiguo que la ortofoto ({ $photo }); puede que falten edificios nuevos en el nDSM
progress-surface-kept = freemapper conserva el modelo de superficie descargado el { $fetched } para la plantilla nDSM
progress-surface-tiles-kept = freemapper conserva el modelo de superficie en caché de { $count ->
        [one] la tesela
        [many] las teselas
       *[other] las teselas
    } { $names } para la plantilla nDSM
progress-orthophoto-unchanged = ortofoto sin cambios
progress-orthophoto-unchanged-dated = ortofoto sin cambios ({ $label } { $date })
progress-orthophoto-download = descargando la ortofoto de { $provider } { $service }
progress-orthophoto-in-leaf = la ortofoto se tomó el { $date }, con los árboles con hojas; sus copas ocultan lo que hay debajo
progress-orthophoto-bare = la ortofoto se tomó el { $date }, con los árboles sin hojas
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; se conservan la ortofoto guardada y su fecha de vuelo { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): sin fecha de vuelo, { $reason }
word-stac-no-image = el STAC no lista ninguna imagen en el centro del área del mapa
word-stac-no-answer = la búsqueda STAC no dio una respuesta utilizable ({ $error })
warning-downloadcenter-no-file = el Downloadcenter no tiene ningún archivo para { $places }
warning-downloadcenter-no-file-empty = el Downloadcenter no tiene ningún archivo para { $places }; { $count ->
        [one] la tesela { $names } quedó vacía
        [many] las teselas { $names } quedaron vacías
       *[other] las teselas { $names } quedaron vacías
    }
progress-k-five-unchanged = plantilla K5 sin cambios
progress-k-five-download = descargando el mapa K5 (1:5000) del WMS del Geoportal Berlin
warning-k-five-unavailable = la plantilla del mapa K5 no está disponible ({ $detail }); omitida

## The state border

progress-beyond-border = el área del mapa llega más allá de la frontera del estado federado; al otro lado las plantillas quedan vacías allí donde terminan la ortofoto y los modelos de altura de { $provider }. ALKIS y OSM vienen de ambos estados
progress-beyond-border-no-data = el área del mapa llega más allá de la frontera del estado federado; no hay datos de { $provider } para { $tiles }, así que las plantillas quedan vacías allí

## OSM

warning-osm-extract-missing-kept = no hay ningún extracto de OSM en caché; se conservan los candidatos de OSM guardados del { $date }
progress-osm-extract-read = leyendo el extracto de OSM { $region } del { $date }
progress-osm-candidates = { $count } elementos candidatos de OSM

## Permit notes

progress-permits-fetch = obteniendo datos de { $source } para las notas de permisos
progress-permits-osm = leyendo el extracto de OSM { $region } del { $date } para las notas de permisos
progress-permits-stored = leyendo los datos guardados de las notas de permisos
warning-permits-source-unreachable = no se pudo leer { $source }, así que las notas de permisos lo omiten: { $error }
warning-permits-osm-no-extract = no hay ningún extracto de OSM en caché para esta área del mapa, así que las notas de permisos omiten OSM; `freemapper fetch` lo descarga

## ALKIS

warning-alkis-coverage-unknown = se desconoce la cobertura de edificios de { $cadastre }; ejecuta `freemapper fetch` con conexión para registrarla
progress-alkis-buildings-download = descargando los edificios de { $cadastre }
progress-alkis-buildings = { $count } edificios de { $cadastre }
warning-alkis-no-buildings = no se encontraron edificios de { $cadastre } en el área del mapa
progress-alkis-skipped = se omitieron { $count } elementos de { $cadastre }: { $list }
warning-alkis-buildings-changed = los edificios de { $cadastre } cambiaron en el origen ({ $changes })
progress-alkis-no-land-use-stored = no hay usos del suelo de { $cadastre } guardados; ejecuta `freemapper fetch` con conexión para los candidatos de zonas prohibidas
progress-alkis-land-use-download = descargando los usos del suelo de { $cadastre }
progress-alkis-land-use-parts = { $count } partes de uso del suelo de { $cadastre }
warning-alkis-land-use-kept = usos del suelo de { $cadastre } no disponibles ({ $error }); se conservan los guardados
warning-alkis-land-use-removed = usos del suelo de { $cadastre } no disponibles ({ $error }); los guardados ya no cubren el área del mapa, así que freemapper los eliminó
warning-alkis-land-use-none = usos del suelo de { $cadastre } no disponibles ({ $error }); no hay candidatos de zonas prohibidas
progress-bdot-filled = { $count } objetos de BDOT10k ({ $what }) añadidos donde EGiB no da nada ({ $powiat })
warning-bdot-failed = BDOT10k no pudo cubrir el hueco de EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB no está disponible ({ $error }); se usa BDOT10k en su lugar ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = no hay estructuras ALKIS de Berlín guardadas; ejecuta `freemapper fetch` con conexión para escaleras, muros, árboles y copas
progress-berlin-removing = eliminando las capas de Berlín guardadas: el área del mapa ya no está cerca de Berlín
progress-berlin-download = descargando estructuras ALKIS de Berlín, catastro de árboles, levantamiento de calles y capas de puntos
progress-berlin-features = { $count } elementos de { $file } de Berlín
warning-berlin-layer-kept = { $file } de Berlín no disponible ({ $detail }); se conserva la capa guardada
warning-berlin-layer-skipped = { $file } de Berlín no disponible ({ $detail }); capa omitida
progress-feinkartierung-none-stored = no hay Feinkartierung de Hamburgo guardada; ejecuta `freemapper fetch` con conexión para barandillas, muros, escaleras y mobiliario urbano
progress-feinkartierung-removing = eliminando la Feinkartierung de Hamburgo guardada: el área del mapa ya no está en Hamburgo
progress-feinkartierung-download = descargando la Feinkartierung de Hamburgo (espacio de calles y zonas verdes públicas)
progress-feinkartierung-features = { $count } elementos de { $file } de la Feinkartierung
warning-feinkartierung-layer-kept = { $file } de la Feinkartierung no disponible ({ $detail }); se conserva la capa guardada
warning-feinkartierung-layer-skipped = { $file } de la Feinkartierung no disponible ({ $detail }); capa omitida
progress-bdot-roads-none-stored = no hay carreteras ni senderos de BDOT10k guardados; ejecuta `freemapper fetch` en línea para obtener las carreteras y los senderos del registro
progress-bdot-roads-removing = se eliminan las carreteras y los senderos guardados de BDOT10k: el área del mapa ya no está en Polonia
progress-bdot-roads-download = descargando las carreteras y los senderos de BDOT10k
progress-bdot-roads-features = { $count } objetos de BDOT10k { $file }
warning-bdot-roads-layer-kept = BDOT10k { $file } no está disponible ({ $detail }); se conserva la capa guardada
warning-bdot-roads-layer-skipped = BDOT10k { $file } no está disponible ({ $detail }); capa omitida
error-extra-layer-unknown = no es una capa de { $folder }
warning-berlin-buildings-missing = no hay edificios de ALKIS guardados para esta área en Berlín; ejecuta `freemapper fetch` para obtenerlos del Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = no hay estructuras ALKIS completas guardadas; ejecuta `freemapper fetch` con conexión para muros, vallas, setos y puentes
progress-nas-removing = eliminando las capas ALKIS completas guardadas: el área del mapa ya no está en un estado federado con servicio NAS
warning-nas-folder-outside = capas ALKIS completas omitidas ({ $folder } sale de la carpeta del proyecto)
progress-nas-download = descargando las estructuras ALKIS completas (NAS)
progress-nas-features = { $count } elementos de { $file } de ALKIS NAS
progress-nas-skipped = se omitieron { $count } elementos de { $file } de ALKIS NAS: { $list }
warning-nas-layer-kept = { $file } de ALKIS NAS no disponible ({ $detail }); se conserva la capa guardada
warning-nas-layer-skipped = { $file } de ALKIS NAS no disponible ({ $detail }); capa omitida

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = no hay objetos del ATKIS Basis-DLM guardados; ejecuta `freemapper fetch` con conexión para manantiales, cuevas, torres, monumentos y fuentes
progress-dlm-removing = eliminando las capas guardadas del ATKIS Basis-DLM: el área del mapa ya no está en un estado federado que lo ofrezca
warning-dlm-folder-outside = capas del ATKIS Basis-DLM omitidas ({ $folder } sale de la carpeta del proyecto)
progress-dlm-download = descargando los objetos del ATKIS Basis-DLM
progress-dlm-features = { $count } elementos de { $file } del ATKIS Basis-DLM
progress-dlm-skipped = se omitieron { $count } elementos de { $file } del ATKIS Basis-DLM: { $list }
warning-dlm-layer-kept = { $file } del ATKIS Basis-DLM no disponible ({ $detail }); se conserva la capa guardada
warning-dlm-layer-skipped = { $file } del ATKIS Basis-DLM no disponible ({ $detail }); capa omitida

## Tree cadastres

progress-trees-none-stored = no hay catastro municipal de árboles guardado; ejecuta `freemapper fetch` con conexión para los árboles individuales
progress-trees-removing = eliminando los catastros de árboles guardados: el área del mapa ya no está en una ciudad que tenga uno
warning-trees-folder-outside = catastros municipales de árboles omitidos ({ $folder } sale de la carpeta del proyecto)
progress-trees-download = descargando los catastros municipales de árboles
progress-trees-count = { $count } árboles de { $id }
progress-trees-skipped = se omitieron { $count } árboles de { $id }: { $list }
warning-trees-kept = catastro de árboles de { $id } no disponible ({ $detail }); se conserva el guardado
warning-trees-skipped = catastro de árboles de { $id } no disponible ({ $detail }); árboles omitidos

## Deriving the templates

progress-writing = escribiendo { $path }
progress-contours-steep = curvas de nivel cada { $interval } m: el terreno es tan empinado que las curvas de nivel cada { $default } m se juntarían; una equidistancia de { $default } m fijada en el proyecto las conserva (`--contour-interval { $default }`)
warning-osm-only-no-terrain = aún no hay datos de terreno para esta región: el mapa se hace solo con OpenStreetMap, sin curvas de nivel, plantillas ni edificios
progress-no-ndsm-recorded = no hay nDSM registrado; ejecuta `freemapper fetch` para la plantilla nDSM
warning-ndsm-tile-invalid = tesela nDSM no válida { $tile } en freemapper.toml; omitida
warning-ndsm-not-cached = nDSM { $tile } no está en caché; ejecuta `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } ilegible ({ $error }); borra { $path } y ejecuta `freemapper fetch`
warning-surface-unusable = modelo de superficie inutilizable ({ $error }); ejecuta `freemapper fetch`

## Downloads and the cache

progress-downloading = descargando { $url }
progress-retry-wait = el servicio está ocupado; nuevo intento en { $seconds } s
progress-downloaded = descargados { $megabytes } MB
progress-downloaded-of = descargados { $megabytes } MB de { $total } MB
progress-tile-index-download = descargando el índice de teselas de { $product }
progress-tile-cached = { $tile } en caché
progress-surface-blocks = leyendo { $count } de los { $total } bloques de la tesela { $tile } del modelo de superficie ({ $megabytes } MB)
progress-surface-whole = la tesela { $tile } del modelo de superficie se descarga entera: { $reason }
warning-tile-cached-older = se usa la tesela { $tile } de { $product } en caché, publicada el { $published } (no se descargó la más nueva, del { $newer })
warning-ndsm-no-cached = no hay nDSM en caché para la tesela { $tile }
warning-ndsm-tile-unavailable-cached = tesela nDSM { $tile } no disponible ({ $reason }); se usa el archivo en caché
warning-ndsm-tile-unavailable-skipped = tesela nDSM { $tile } no disponible ({ $reason }); omitida
warning-file-check-failed = no se pudo comprobar si { $file } sigue vigente ({ $reason }); se usa el archivo en caché y, sin preguntar, los demás archivos en caché de este servidor
warning-ldbv-no-buildings-file = no hay archivo de edificios para la tesela { $tile } en el servidor; puede que falten edificios allí
progress-ndsm-not-published = no hay nube de puntos nDSM publicada para la tesela { $tile }
progress-ndsm-cached = nDSM { $tile } en caché
progress-point-cloud-not-published = no hay nube de puntos láser publicada para la tesela { $tile }
progress-point-cloud-cached = nube de puntos { $tile } en caché
warning-point-cloud-no-cached = no hay nube de puntos en caché para la tesela { $tile }; allí no hay candidatos de vegetación
warning-point-cloud-unavailable-cached = nube de puntos { $tile } no disponible ({ $reason }); se usa el archivo en caché
warning-point-cloud-unavailable-skipped = nube de puntos { $tile } no disponible ({ $reason }); allí no hay candidatos de vegetación
warning-point-clouds-unavailable = las nubes de puntos láser no están disponibles ({ $reason }); los candidatos de vegetación usan las teselas en caché, si las hay
progress-no-point-clouds = aún no hay nubes de puntos láser para la vegetación aquí: el estado federado no publica ninguna que freemapper pueda leer, o no en el sistema de coordenadas del mapa
warning-point-clouds-not-fetched = se eligió la vegetación, pero no hay nubes de puntos descargadas; ejecuta `freemapper fetch` para descargarlas
warning-point-cloud-unreadable = nube de puntos { $tile } ilegible ({ $error }); borra { $path } y ejecuta `freemapper fetch`
warning-osm-nrw-nearest = no hay ningún extracto de distrito de Renania del Norte-Westfalia a menos de 100 m del área; se lee el más cercano, { $district }
warning-osm-disk-full-cached = el disco está lleno, así que freemapper usa el extracto de OSM en caché del { $date }. Libera espacio y ejecuta `freemapper fetch` otra vez para obtener uno más reciente
warning-osm-geofabrik-unavailable = Geofabrik no disponible ({ $error }); se usa el extracto de OSM en caché del { $date }
progress-osm-extract-cached = extracto de OSM { $region } del { $date } en caché
progress-osm-extract-download = descargando { $url } (unos { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = descargando la tesela bDOM { $tile } ({ $megabytes } MB, { $index } de { $total })
progress-sh-dgm-not-in-atom = la tesela DGM1 { $tile } no está en el directorio INSPIRE ATOM; descargando su XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = la tesela DGM1 { $tile } del directorio INSPIRE ATOM no se puede leer ({ $error }); descargando su XYZ ({ $megabytes } MB)
progress-sh-tile = tesela { $tile }: { $text }
progress-sh-download-again = tesela { $tile }: la descarga no se puede leer ({ $message }); descargándola otra vez
warning-sh-portal-no-date = el Downloadportal no da fecha para la tesela { $tile } de { $model }
warning-sh-dgm-xyz-used = la tesela DGM1 { $tile } no está en el directorio INSPIRE ATOM; se usó su XYZ del Downloadportal
warning-sh-dgm-xyz-used-unreadable = la tesela DGM1 { $tile } del directorio INSPIRE ATOM no se puede leer ({ $error }); se usó su XYZ del Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = tesela { $tile } de { $model }: { $count } celdas del área del mapa no tienen altura
warning-centre-tile-missing = falta la tesela { $tile } de { $model } en el centro del área del mapa, así que no hay fecha
warning-centre-tile-unlisted = la lista de teselas no nombra ninguna tesela { $tile } de { $model } en el centro del área del mapa, así que no hay fecha
warning-list-unreadable-kept = no se pudo leer la lista de teselas de { $model } ({ $error }); se usó la guardada de una ejecución anterior
warning-portal-list-unreadable-kept = no se pudo leer la lista de teselas de { $model } del Downloadportal ({ $error }); se usó la guardada de una ejecución anterior
warning-surface-much-older = { $model } es de { $surface } y la ortofoto de { $flight }. Puede que al nDSM le falten edificios y árboles más recientes.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } elementos de la lista de teselas no tienen una clave de tesela utilizable; omitidos
warning-rp-list-unreadable-kept = no se pudo leer la lista de teselas del LVermGeo RP ({ $error }); se usó la guardada de una ejecución anterior
warning-rp-metalink-not-kept = no se conservó el Metalink del LVermGeo RP: { $error }
warning-rp-metalink-unreadable-kept = no se pudo leer el Metalink del LVermGeo RP de { $model } ({ $detail }); se usó el guardado de una ejecución anterior
warning-rp-metalink-unreadable = no se pudo leer el Metalink del LVermGeo RP de { $model } ({ $detail }), así que las teselas para las que su lista no da año de vuelo quedan sin alturas
warning-rp-tile-without-name = tesela { $tile } de { $model }: este cuadrado de 1 km no tiene alturas en el mapa. La lista de teselas del LVermGeo RP (Laserbefliegung) { $note } para él, así que no se conoce el nombre del archivo{ $either ->
        [metalink] , y su Metalink tampoco nombra ningún archivo para él
       *[none] {""}
    }
warning-rp-centre-no-date = la tesela { $tile } de { $model } no tiene fecha: la lista de teselas del LVermGeo RP (Laserbefliegung) { $note } para ella
word-rp-list-neither = no da ni año de vuelo ni días de vuelo
word-rp-list-no-days = no da días de vuelo
word-rp-list-no-year = no da año de vuelo
word-rp-list-days-unreadable = da días de vuelo { $days } que no se pueden leer
word-rp-list-disagree = da días de vuelo { $days } y un año de vuelo { $year } que no coinciden en un mismo año

## Hesse

warning-hvbg-undated = la lista de metadatos no tiene un Bildflugdatum legible para la tesela { $tiles }; sus registros en el proyecto no llevan fecha de vuelo
warning-hvbg-undated-more = la lista de metadatos no tiene un Bildflugdatum legible para la tesela { $tiles } y { $more } más; sus registros en el proyecto no llevan fecha de vuelo
warning-hvbg-list-unreadable-older = no se pudo leer la nueva lista de metadatos del HVBG { $name } ({ $detail }); se usa la lista anterior { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: sin fecha, el STAC no lista ninguna tesela en el centro del área del mapa
warning-lgln-no-date = { $model }: sin fecha ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; el modelo guardado conserva su fecha registrada
warning-mv-cells-empty = { $model }: { $count } celdas dentro de las teselas de Mecklemburgo-Pomerania Occidental no tienen altura
warning-mv-centre-no-date = { $model }: sin fecha para la tesela central { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } filas de la lista de teselas de { $model } no tienen un nombre de tesela utilizable; omitidas
warning-nrw-list-rows-undated = { $count } filas de la lista de teselas de { $model } no tienen una fecha legible
warning-nrw-list-older = la lista de teselas de { $model } del servidor es del { $server }, más antigua que la guardada, del { $kept }; se usó la guardada
warning-nrw-list-no-date = la lista de teselas no da fecha para la tesela { $tile } de { $model }

## After `init`

note-scale-outside-norm = nota: 1:5000 no figura en la norma de sprint; freemapper-print avisa de ello y los símbolos se quedan al 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } no es un id de OSM como way/992423119
error-place-invalid-answer = respuesta no válida de la búsqueda de lugares: { $detail }
error-place-not-found = no se encontró { $id }. Nominatim solo lista objetos de OSM con etiquetas, así que una vía exterior sin etiquetar de un multipolígono no aparece. Usa en su lugar la relación del multipolígono (--osm relation/<id>), o guarda el contorno como GeoJSON y usa --geojson
error-place-is-a-point = { $id } es un punto, no un área; elige su recinto (una vía o una relación)
error-place-no-such-part = { $id } tiene { $count ->
        [one] 1 parte
        [many] { $count } de partes
       *[other] { $count } partes
    }; no existe la parte { $part }
error-place-several-parts = { $id } es un área de { $count } partes separadas ({ $sizes }). Un área de mapa es un solo polígono: elige una parte con --part <n> (1 es la mayor), o guarda como GeoJSON el contorno que quieras y usa --geojson
error-layer-not-a-world-file = { $path }: no es un world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = no hay vista previa de { $layer }; capas con vista previa: { $names }
error-layer-no-ndsm-published = no se publica ningún nDSM para las teselas de esta área del mapa, así que no hay plantilla ndsm
error-layer-not-made = { $layer }: aún no se ha creado; ejecuta { $step ->
        [fetch] freemapper fetch o run (MCP: start_pipeline); solo Berlín
        [export] freemapper export o run (MCP: start_pipeline)
       *[derive] freemapper derive o run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; el mapa OCAD no recibe ningún fondo de ello
warning-map-preview-failed = el mapa está escrito, pero falta su vista previa: { $error }
warning-training-no-tree-cover = el mapa de entrenamiento conserva entero el terreno abierto de los parques: falta la cobertura arbórea (data/tree-cover.tif); ejecuta `freemapper derive` otra vez
warning-ocad-no-paper-frames = el punto de partida OCAD aún no tiene marcos de papel; están en templates/paper-frame.omap, una plantilla para OOM
warning-paper-frames-left-out = marcos de papel omitidos en templates/paper-frame.omap: { $count }; están demasiado lejos del mapa para un mapa de OOM; acércalos al área del mapa en la página del área
