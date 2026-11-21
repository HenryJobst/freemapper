# Textos del programa freemapper, español (ADR 0092). El inglés es la referencia: cada otro idioma
# tiene los mismos ids de mensaje y las mismas variables. Los textos de otros programas (errores del
# sistema operativo, mensajes de analizadores) llegan en variables y se quedan como están.

## The --lang switch

cli-lang-unknown = { $value } no es un idioma para el que freemapper tenga textos; disponibles: { $available }

## init

cli-part-needs-osm = --part elige una parte de un área de OSM y necesita --osm; con --place, busca primero el id y usa --osm <id> --part <n>
cli-reading-file = leyendo { $path }
cli-no-area-for-place = no se encontró ningún área para "{ $query }". Si el lugar es un área de varias partes separadas, elige una con --osm <id> --part <n>, o usa --geojson
cli-using-place = Usando { $name } ({ $id })
cli-created-project = Proyecto creado { $path }

## Cache

cli-no-cache-dir = este sistema no tiene carpeta de caché; indica --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Descargada 1 tesela
        [many] Descargadas { $count } de teselas
       *[other] Descargadas { $count } teselas
    }
cli-wrote = Escrito { $path }
cli-kept-map = Se conserva { $path } (puede contener tu trabajo)
cli-map-lacks-templates = El mapa aún no hace referencia a las plantillas de imagen. Añádelas en OOM (Plantillas > Abrir plantilla…) o sustituye el mapa con --force, que conserva el anterior como .omap.bak.
cli-map-lacks-buildings = El mapa aún no tiene edificios de ALKIS. Para añadirlos, sustitúyelo con --force, que conserva el anterior como .omap.bak.
cli-map-lacks-canopies = Al mapa le faltan los tejadillos y cocheras de ALKIS (522), que ahora son objetos del mapa. Para añadirlos, ejecuta con --force, que conserva el mapa anterior como .omap.bak.
cli-map-light = El mapa base: { $joined } líneas unidas, { $left } ocultas o demasiado pequeñas pasan a las capas de candidatos (no se movió nada)
cli-map-light-kept = Las capas de candidatos contienen ahora además { $left } objetos o tramos que esta versión saca del mapa base (con la etiqueta freemapper:generalized). El mapa conservado aún los tiene, así que aparecen dos veces. Sustituye el mapa con --force para limpiarlo (el anterior queda como .omap.bak).
cli-map-old-candidates = El mapa aún nombra templates/candidates.omap; esta ejecución lo llenó con todos los candidatos. Ahora los candidatos vienen también en siete capas, un archivo cada una (templates/candidates-*.omap). El mapa las lista después de que lo sustituyas con --force (el anterior queda como .omap.bak), o añádelas tú mismo en OOM (Plantillas > Abrir plantilla…).

## PDF

cli-pdf-without-credits = PDF sin créditos de fuentes: ODbL y dl-de/by-2-0 piden que nombres las fuentes al compartir el mapa; consulta ATTRIBUTION.txt
cli-wrote-pdf = Escrito { $path } (el mapa sin candidatos)
cli-wrote-pdf-candidates = Escrito { $path } (la impresión de control: el mapa con todos los candidatos, fiel a la posición; compruébalos sobre el terreno)
cli-wrote-training = Escritos { $map } y { $pdf } (el mapa de entrenamiento: { $joined } unidos, { $shifted } desplazados, { $left } omitidos o recortados)
cli-training-free-ends = Extremos de línea dejados libres: { $ways } donde un camino se encontraba con otro camino, { $others } en otras líneas
cli-training-apart-left = el mapa de entrenamiento aún tiene { $length } mm (en papel) donde los objetos guardan muy poca separación; revísalo en OOM
cli-kept-training = Se conserva el mapa de entrenamiento { $path }, para que una ejecución no sobrescriba uno que se imprimió o se entregó; bórralo o indica --force para renovarlo

## OCAD

cli-ocad-kept = Se conserva { $path } (puede contener tu trabajo; --force lo sustituye y conserva el anterior como .ocd.bak)
cli-ocad-old-candidates = El mapa OCAD aún nombra { $path } como mapa de fondo; esta ejecución lo llenó con todos los candidatos. Ahora los candidatos vienen también en siete capas, un archivo cada una (-candidates-*.ocd). El mapa OCAD las lista después de que lo sustituyas con --force (el anterior queda como .ocd.bak), o añádelas tú mismo en OCAD como mapas de fondo.
cli-ocad-wrote = { $count ->
        [one] Escrito { $path } (1 objeto)
        [many] Escrito { $path } ({ $count } de objetos)
       *[other] Escrito { $path } ({ $count } objetos)
    }
cli-ocad-left-out = Omitidos en el mapa OCAD: { $skipped } objetos (sin símbolo para OCAD, sin superficie o fuera del área de dibujo de OCAD de ±2 m) y { $dropped } partes de línea o huecos degenerados.
cli-ocad-load-symbols = En OCAD, elige Símbolo > Cargar colores y símbolos desde… y selecciona el conjunto de símbolos oficial{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , el conjunto escolar de la IOF 2019
       *[other] {" "}ISSprOM 2019
    }; sustituye por número los símbolos provisionales de freemapper.

## select

cli-select-running = Selección de área para { $path }: pulsa "Listo" en la página o Ctrl+C para detenerla.
cli-select-no-browser = No se pudo abrir el navegador ({ $error }); abre la dirección de arriba.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; sin área, no puede ser un área de mapa): { $place }
cli-find-hint = Crea el proyecto con: freemapper init <dir> --osm { $id }
cli-find-no-area = Ninguno de estos lugares tiene un área que pueda ser un área de mapa. Dibuja el área con: freemapper select <dir>
cli-find-nothing = no se encontró ningún lugar para "{ $query }"; prueba con otro nombre o dibuja el área con: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = aviso: el adaptador de { $folder } se deja fuera: { $finding }
cli-adapter-note = aviso: el adaptador de { $folder } se carga, pero: { $finding }
cli-adapter-from-project = aviso: se usa el adaptador { $adapter } de la carpeta del proyecto; puede consultar estos hosts: { $hosts }
cli-adapter-overlaps = aviso: el adaptador { $adapter } cubre también territorio de { $others }; sus objetos van a los mapas de allí
cli-adapter-dir-relative = aviso: FREEMAPPER_ADAPTER_DIR no es una ruta absoluta ({ $path }); se ignora
cli-adapters-no-project = aviso: { $path } no es una carpeta; no se leyó ningún adaptador de un proyecto
cli-adapters-overlaps = comparte territorio con: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revisión { $revision }  comprobado { $checked }  hosts: { $hosts }
cli-adapters-source-built-in = integrado
cli-adapters-source-user = de tu carpeta
cli-adapters-source-project = del proyecto
cli-adapters-never-checked = nunca
cli-adapters-user-dir = Tus adaptadores se leen de { $path }
cli-adapters-no-user-dir = Este sistema no tiene carpeta para tus adaptadores; FREEMAPPER_ADAPTER_DIR indica una

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper puede ahora consultar también { $host } para { $name }. La entrada está en { $file }; `freemapper adapters disallow-host { $adapter } { $host }` la retira. Una página de área o un asistente de IA que esté en marcha lee la entrada cuando se inicia de nuevo
cli-allow-host-already = { $host } ya está permitido para { $name }
cli-allow-host-declared = el adaptador de { $name } ya nombra { $host } por sí mismo; no hay nada que permitir
cli-allow-host-refused = no se puede permitir { $host }: { $reason }
cli-allow-host-unknown-adapter = no existe el adaptador integrado { $adapter }; los adaptadores integrados son: { $adapters }
cli-allow-host-folder-adapter = { $adapter } es un adaptador de tu carpeta de adaptadores; añade el host a `[net] hosts` de su manifiesto
cli-allow-host-no-config = este sistema no tiene carpeta de configuración donde guardar los hosts permitidos
cli-allow-host-write = no se pudo escribir { $file }: { $detail }
cli-allow-host-unreadable = no se puede leer { $file } ({ $detail }); corrige o elimina el archivo primero; no se cambió nada
cli-disallow-host-removed = freemapper ya no consulta { $host } para { $name }
cli-disallow-host-absent = { $host } no estaba permitido para { $name }; no hay nada que retirar
cli-allowed-hosts-unreadable = aviso: no se pudo leer { $file } ({ $detail }); no se permite ningún host adicional
cli-allowed-hosts-ignored = aviso: { $file }: se ignora la entrada { $host } de { $adapter }: { $reason }
cli-allowed-hosts-no-adapter = freemapper no tiene ningún adaptador integrado con ese nombre
cli-allowed-host-redundant = aviso: { $host } es ahora uno de los hosts que nombra el adaptador de { $name }; tu entrada en { $file } se eliminó
cli-adapters-allowed = permitidos por ti: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adaptador { $adapter } ({ $name }), revisión { $revision }, hosts: { $hosts }
cli-check-error = error: { $finding }
cli-check-warning = aviso: { $finding }
cli-check-passed = { $folder } supera la comprobación
cli-check-passed-warnings = { $folder } supera la comprobación, con { $count ->
        [one] un aviso
        [many] { $count } de avisos
       *[other] { $count } avisos
    }
cli-check-failed = { $folder } no supera la comprobación
cli-check-no-folder = no existe la carpeta { $folder }
cli-check-live-ok = { $capability } correcto: { $detail }
cli-check-live-failed = { $capability } FALLÓ: { $detail }
cli-check-live-not-asked = { $capability } no consultado: { $detail }
cli-check-live-box-outside = el recuadro no está dentro del límite del adaptador { $adapter }; una comprobación en vivo consulta los servicios para un lugar que el adaptador cubre
cli-check-live-cannot = la comprobación en vivo no puede ejecutarse: { $detail }
cli-check-detail-not-declared = el adaptador no declara ninguno
cli-check-detail-stopped = falló una parte anterior
cli-check-detail-see-warnings = consulta los avisos
cli-check-detail-outside-dataset = el recuadro queda fuera del conjunto de datos
cli-check-detail-objects = { $count ->
        [one] un objeto
        [many] { $count } de objetos
       *[other] { $count } objetos
    }
cli-check-detail-layers = { $layers } capas, { $count } objetos
cli-check-detail-trees = { $count ->
        [one] un árbol
        [many] { $count } de árboles
       *[other] { $count } árboles
    }
cli-check-detail-extract = { $label }, datos del { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = modelo del terreno
cli-check-raster-surface = modelo de superficie
cli-check-raster-tiles = { $product }: { $count ->
        [one] una tesela
        [many] { $count } de teselas
       *[other] { $count } teselas
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } y { $last }

## Folder lock

cli-lock-held = otra ejecución de freemapper está usando esta carpeta (desde { $time }, iniciada por { $who }). Espera a que termine o detenla primero.
cli-lock-who-cli = el terminal
cli-lock-who-mcp = un agente de IA (MCP)
cli-lock-who-page = la página de selección
cli-lock-who-unknown = otro programa
cli-lock-failed = no se pudo bloquear la carpeta del proyecto para esta ejecución: { $error }
cli-lock-stuck = algo que no es un archivo de bloqueo estorba y no se pudo eliminar

## Run routes of the selection page

cli-run-bad-request = Las opciones de la ejecución no son válidas: { $error }
cli-run-no-project = Aún no hay ningún proyecto guardado en esta carpeta. Guarda primero el área.
cli-run-already-running = Ya hay una ejecución ({ $job }) en marcha para este proyecto. Espera a que termine o cancélala primero.
cli-run-disk-stops = No hay suficiente espacio libre en disco para la descarga: { $messages }
cli-run-busy-elsewhere = Otro proyecto de este programa está ejecutando ahora un proceso. Los proyectos comparten la caché de descargas, así que solo se ejecuta uno a la vez. Inténtalo de nuevo cuando termine.
cli-run-nothing-to-cancel = No hay ninguna ejecución en marcha, así que no hay nada que cancelar.
cli-run-unknown-job = No existe la ejecución { $job } para este proyecto.
cli-status-too-fast = Las consultas de estado llegan demasiado seguido. Pregunta como máximo cinco veces por segundo.
cli-open-folder-failed = No se pudo abrir la carpeta ({ $error }).
cli-open-folder-no-opener = Este ordenador no tiene ningún programa para abrir carpetas ({ $program }). Abre la carpeta a mano.
cli-open-folder-missing = La carpeta del proyecto aún no existe. Guarda primero el área.
cli-open-folder-refused = El nombre de la carpeta no es seguro para pasarlo al gestor de archivos.
cli-save-running = Hay una ejecución en marcha para este proyecto. Guarda de nuevo cuando termine.
cli-save-unknown-kind = Tipo de mapa desconocido "{ $kind }". Usa schoolyard, sprint o forest.
cli-save-scale-too-large = La escala { $scale } es demasiado grande.
cli-save-bad-content = Los ajustes de contenido no son válidos. No se guardó nada.
cli-map-image-none = Crea primero el mapa.
cli-map-image-gone = Esta vista previa ya no está disponible.
cli-map-image-running = Hay una ejecución en marcha para este proyecto. La vista previa se puede calcular cuando termine.
cli-map-image-stale = El proyecto guardado ya no coincide con el mapa generado, por ejemplo porque cambió el área, el tipo de mapa o la escala. Inicia primero una ejecución que sustituya el mapa ("Sustituir el mapa existente" en la página, o freemapper run --force).
cli-map-image-superseded = Mientras tanto se pidió una vista previa más reciente, así que esta no se calculó.
cli-map-image-bad-content = Los ajustes de contenido no son válidos.
cli-map-image-bad-request = La solicitud de vista previa no es válida. Envía un objeto JSON, con los ajustes de contenido si los hay.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – selección del área
page-heading = Elige el área del mapa
page-update-available = Está disponible freemapper { $version }. Actualiza con
page-update-notes = Notas de la versión
page-folder = Carpeta del proyecto
page-pick = Buscar un lugar
page-query-placeholder = p. ej. Clara-Zetkin-Schule Strausberg
page-search-button = Buscar
page-searching = Buscando …
page-no-results = No se encontró nada.
page-point-only = solo un punto, no un área
page-draw = Dibújala tú
page-rectangle = Rectángulo
page-polygon = Polígono
page-finish-polygon = Cerrar polígono
page-clear = Borrar
page-hint-rectangle = Haz clic en dos esquinas opuestas del mapa.
page-hint-polygon = Haz clic en las esquinas una tras otra. Para cerrarlo, haz clic en la primera esquina, haz doble clic o pulsa "Cerrar polígono". Esc cancela, Retroceso quita la última esquina.
page-hint-idle = Elige un lugar de la búsqueda o dibuja tú el área.
page-summary = Área
page-source-osm = De OpenStreetMap: { $name }
page-source-drawn = Dibujada a mano
page-size = Tamaño con margen: { $width } × { $height } m
page-tiles-one = { $n } tesela: { $names }
page-tiles-other = { $n } teselas: { $names }
page-download = Descarga: unos { $mb } MB
page-download-none = Descarga: todo está ya en caché
page-download-details = Qué se descarga
page-cached = en caché
page-disk = Espacio en disco: { $volumes }
page-disk-volume = { $free } libres, hacen falta unos { $needed }
page-cache-volume = caché
page-project-volume = carpeta del proyecto
page-no-room = no cabe en el disco
page-no-room-skip = No todo cabe en el disco (freemapper mantiene { $reserve } libres). La descarga omite las partes marcadas. Libera primero algo de espacio si las necesitas:
page-no-room-stop = No hay suficiente espacio en disco (freemapper mantiene { $reserve } libres). Sin las partes marcadas no hay mapa, así que la descarga se detiene antes de bajar nada. Libera primero algo de espacio:
page-disk-unknown = freemapper no puede leer el espacio libre en disco ({ $error }). Descarga los datos sin esa comprobación.
page-too-large = El área es demasiado grande: como máximo { $max } m por lado (con margen). Elige una más pequeña.
page-outside = El área queda fuera de los estados federados admitidos ({ $states }); freemapper aún no tiene fuentes de datos para otras regiones.
page-outside-unknown = El área queda fuera de las regiones que freemapper admite; allí aún no tiene fuentes de datos.
page-adapters-failed = No se pueden listar las fotos aéreas ({ $detail }). El mapa muestra solo OpenStreetMap; todo lo demás funciona.
page-settings = Mapa
page-kind = Tipo de mapa
page-schoolyard = Mapa de patio escolar (1:1000)
page-sprint = Mapa de sprint (1:4000)
page-forest = Mapa de bosque (1:15000)
page-scale = Escala 1:
page-map-title = Título
page-contour-interval = Equidistancia de las curvas de nivel
page-contour-auto = automática
page-settings-fixed = El proyecto ya existe; el tipo de mapa, la escala y el título están en freemapper.toml. La equidistancia de las curvas de nivel aún se puede cambiar.
page-save = Guardar
page-saving = Guardando …
page-saved = Guardado en freemapper.toml.
page-zone-changed = Guardado. El área está ahora en otra zona UTM; freemapper vuelve a descargar sus datos en la próxima ejecución.
page-provider-changed = Guardado. Otro proveedor de datos sirve la elevación y las imágenes aéreas de la nueva área; freemapper vuelve a descargar sus datos en la próxima ejecución.
page-save-stale = freemapper.toml se cambió en otra ventana o por otro programa desde que esta página lo leyó. No se guardó nada; la página conoce ahora el estado guardado. Guarda de nuevo para sustituirlo por el área que se muestra aquí.
page-has-map = El mapa existente (.omap) se queda como está; freemapper crea nuevas plantillas y candidatos para la nueva área. La opción "Sustituir el mapa existente" de abajo (o "freemapper run --force") crea también el mapa de nuevo y conserva el anterior como .omap.bak.
page-done = Listo
page-source = Código fuente (AGPL-3.0-or-later)
page-closed = Listo. Puedes cerrar esta ventana.
page-next = Siguiente: freemapper run "{ $dir }"
page-made-files = Creados: { $files }. Carpeta: { $dir }
page-back-to-agent = Dile a tu asistente de IA que el área está guardada.
page-nothing-saved = No se guardó nada.
page-failed = Error: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Foto aérea { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Qué va al mapa
page-content-help = El perfil decide qué va directamente al mapa y qué queda como candidato por revisar. Lo que elijas para un grupo prevalece sobre el perfil. Se guarda con el área.
page-content-profile-legend = Perfil
page-content-profile-safe = Seguro
page-content-profile-safe-hint = Cada objeto se queda donde lo pone la generación: los edificios y las superficies del catastro van al mapa; muros, vallas, árboles y gran parte de OpenStreetMap (estanques, caminos sin superficie conocida) quedan como candidatos.
page-content-profile-balanced = Equilibrado
page-content-profile-balanced-hint = Los objetos del catastro, de los registros y del levantamiento de calles van al mapa, con las curvas de nivel y las zonas prohibidas a lo largo de las vías del tren. Muros, vallas, setos, taludes, escaleras, agua, árboles, monumentos, pasos, caminos y vías férreas vienen además de OpenStreetMap. El mobiliario urbano, las demás zonas prohibidas y las formas del modelo del terreno quedan como candidatos.
page-content-profile-full = Completo
page-content-profile-full-hint = Todo va al mapa, como en la impresión de control; solo quedan fuera los candidatos ocultos.
page-content-groups-legend = Por grupo
page-content-groups-help = "Como en el perfil" sigue el perfil. Lo que eso significa para un grupo depende de dónde viene cada objeto.
page-content-place-profile = Como en el perfil
page-content-place-map = Mapa
page-content-place-candidates = Candidato
page-content-place-off = Desactivado
page-content-group-walls = Muros
page-content-group-fences = Vallas
page-content-group-hedges = Setos
page-content-group-slopes = Taludes y cortados
page-content-group-stairs = Escaleras
page-content-group-trees = Árboles y grupos de árboles
page-content-group-water-features = Agua, fuentes y manantiales
page-content-group-monuments = Torres, monumentos y tendidos eléctricos
page-content-group-furniture = Mobiliario urbano y farolas
page-content-group-passages = Pasos
page-content-group-contours = Curvas de nivel
page-content-group-landforms = Montículos, depresiones, taludes y cortados del modelo del terreno
page-content-group-vegetation = Vegetación de nubes de puntos láser (Brandeburgo, Berlín, Renania del Norte-Westfalia, Renania-Palatinado, Sarre y Baviera; desactivada hasta que la elijas; descarga unos 40 a 600 MB por km²)
page-content-group-out-of-bounds = Zonas prohibidas
page-content-group-paths = Caminos y vías férreas
page-content-group-streets = Calles y plazas
page-run-heading = Crear el mapa
page-preview-heading = Vista previa del mapa
page-preview-stored = Tal como se generó la última vez.
page-preview-live = Con los ajustes de esta página, aún sin guardar.
page-preview-live-saved = Con los ajustes guardados, aún sin generar en una ejecución.
page-preview-computing = Calculando …
page-preview-none = Aún no se ha generado ningún mapa.
page-preview-later = Este mapa aún no tiene vista previa. Llega con la próxima ejecución que escriba el mapa.
page-preview-stale = El área, el tipo de mapa o la escala cambiaron desde que se generó el mapa. La imagen sigue mostrando el mapa tal como se generó la última vez; una vista previa nueva necesita una ejecución con "Sustituir el mapa existente".
page-preview-area-unsaved = El área de esta página aún no está guardada. La imagen muestra el mapa tal como se generó la última vez; la nueva área necesita una ejecución con "Sustituir el mapa existente".
page-preview-run = Tal como se generó la última vez. La página actualiza la imagen cuando termina la ejecución.
page-preview-open = Abrir a tamaño completo
page-preview-alt = Vista previa de los objetos del mapa
page-opt-ocad = Escribir un proyecto inicial para OCAD
page-help-ocad = Para quien sigue trabajando en OCAD.
page-opt-pdf = PDF imprimible
page-help-pdf = Solo los objetos del mapa, sin candidatos.
page-opt-pdf-candidates = Impresión de control con todos los candidatos
page-opt-training = Mapa de entrenamiento
page-help-training = El mapa y todos los candidatos, limpiados para imprimirlos tal cual; con OCAD, también como archivo OCAD.
page-views-heading = Vistas del mapa de entrenamiento
page-view-no-paths = Sin caminos (mapa Ultimate)
page-view-brown-only = Solo marrón (orientación por curvas de nivel)
page-view-no-north-lines = Sin líneas norte
page-view-custom = Otra vista
page-help-views = Cada vista marcada es un archivo más junto al mapa de entrenamiento, `<name>-training-<view>.omap` y su PDF, con OCAD también un archivo OCAD. Otra vista es `no:505,506` (omitir símbolos) o `only:101,102` (conservar solo estos); `+` combina vistas. El título, la escala y las fuentes se mantienen siempre.
page-help-views-forest = Sin caminos y solo marrón son únicamente para mapas de bosque.
page-opt-scale-bar = Barra de escala en los PDF
page-help-scale-bar = Los mapas alemanes no suelen llevarla.
page-opt-sources = Líneas de fuentes en los PDF
page-sources-auto = automáticas
page-sources-full = completas
page-sources-credit-only = solo freemapper
page-warn-credit-only = Entonces los PDF nombran solo a freemapper. Las licencias de los datos (ODbL, dl-de/by-2-0) exigen atribución cuando compartes el mapa; entonces nombras tú las fuentes. ATTRIBUTION.txt las lista todas.
page-need-pdf = La barra de escala y las líneas de fuentes solo se aplican cuando se genera un PDF.
page-opt-offline = Solo caché, sin red
page-opt-force = Sustituir los mapas existentes
page-help-force = El mapa y, con "Mapa de entrenamiento" marcado, el mapa de entrenamiento. Los archivos anteriores quedan como .omap.bak.
page-opt-permits = Notas de permisos
page-help-permits = Unidades administrativas, áreas protegidas y a quién preguntar, en permits.md. Una orientación, no asesoramiento jurídico.
page-run-start = Crear mapa
page-run-again = Crear de nuevo
page-run-cancel = Cancelar
page-run-cancelling = Cancelando …
page-warnings-heading = Avisos
page-where-files = Los archivos están en la carpeta del proyecto:
page-open-folder = Abrir carpeta
page-log-label = Registro de la ejecución
page-force-title = ¿Sustituir los mapas existentes?
page-force-text = Se sustituye el mapa existente y, con "Mapa de entrenamiento" marcado, también el mapa de entrenamiento. Tus ediciones no se pierden: los archivos anteriores quedan en la carpeta del proyecto como .omap.bak. ¿Continuar?
page-force-cancel = Cancelar
page-force-confirm = Sustituir

page-step-fetch = Descargando datos
page-step-derive = Calculando plantillas
page-step-map = Escribiendo el mapa
page-step-ocad = Escribiendo el proyecto OCAD
page-step-pdf = Escribiendo el PDF
page-step-pdf-candidates = Escribiendo la impresión de control
page-step-training = Escribiendo el mapa de entrenamiento
page-step-permits = Escribiendo las notas de permisos
page-step-line = Paso { $index } de { $total }: { $label }
page-starting = La ejecución está empezando …
page-file-kind-map = mapa
page-file-kind-template = plantilla
page-file-kind-ocad = mapa OCAD
page-file-kind-ocad-candidates = candidatos OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = impresión de control
page-file-kind-training-map = mapa de entrenamiento
page-file-kind-training-pdf = PDF del mapa de entrenamiento
page-file-kind-training-ocad = mapa de entrenamiento OCAD
page-file-kind-training-view-map = vista del mapa de entrenamiento
page-file-kind-training-view-pdf = PDF de la vista del mapa de entrenamiento
page-file-kind-training-view-ocad = vista del mapa de entrenamiento OCAD
page-file-kind-preview = vista previa del mapa
page-file-kind-permits = notas de permisos
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Listo. Los archivos están escritos.
page-result-failed = La ejecución falló: { $error }
page-result-failed-kept = Lo escrito se queda en la carpeta del proyecto; una nueva ejecución continúa desde ahí.
page-result-cancelled = Cancelado. Lo ya descargado se queda en la caché; una nueva ejecución continúa desde ahí.
page-result-title-done = Mapa creado
page-result-title-failed = La ejecución falló
page-result-title-cancelled = Ejecución cancelada
page-kept-map = El mapa existente se dejó como estaba. "Sustituir el mapa existente" lo crea de nuevo y conserva el anterior como .omap.bak.
page-no-files = No se creó ningún archivo.
page-opened-folder = La carpeta está abierta.
page-open-failed = { $message } Carpeta: { $dir }
page-why-no-area = Elige primero un área.
page-why-checking = Se está calculando la vista previa …
page-why-too-large = El área es demasiado grande.
page-why-outside = El área queda fuera de los estados federados admitidos.
page-why-disk = No hay suficiente espacio libre en disco (mira arriba).
page-why-running = Hay una ejecución en marcha.
page-err-scale-bar-no-pdf = La barra de escala necesita un PDF.
page-err-sources-no-pdf = Las líneas de fuentes necesitan un PDF.
page-save-failed = El área no está guardada; no se inició ninguna ejecución.
page-poll-lost = No hay respuesta del servidor. freemapper sigue intentándolo.
page-run-unknown = El servidor ya no conoce esta ejecución.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Descartar

## update (spec 3.4 to 3.6)

update-source-build = Este freemapper se compiló desde el código fuente. Actualízalo con git pull y cargo build --release --locked.
update-current = freemapper { $version } está actualizado.
update-available = freemapper { $new } está disponible (tienes { $old }), publicado el { $released }.
update-details =
    Notas de la versión: { $notes }
    Descarga: { $size } MB
update-decimal-separator = ,
update-ask = ¿Instalarlo ahora? [s/N]
update-not-installed = No se instaló nada.
update-needs-yes = la entrada no es un terminal, así que freemapper no puede preguntar antes de instalar; ejecuta freemapper update --yes para instalar sin preguntar
update-installing = descargando y comprobando freemapper { $version }
update-done = freemapper actualizado de { $old } a { $new }. Notas de la versión: { $notes }
update-mcp-restart = Un servidor MCP en marcha conserva la versión anterior hasta que el asistente de IA lo reinicie.
update-rolled-back = Se volvió a freemapper { $version }. La versión que dejaste se conserva como { $path }.
update-not-writable = no se puede escribir en la carpeta del programa { $path }, y freemapper no pide derechos de administrador. Descomprime allí la nueva versión a mano, o mueve freemapper a una carpeta en la que puedas escribir.
update-too-old = freemapper { $new } no se puede instalar con la autoactualización de { $old }; instala a mano { $needed } o una más reciente, consulta { $notes }
update-busy = otra actualización o reversión está en marcha en la carpeta del programa (archivo de bloqueo { $path }); espera a que termine e inténtalo de nuevo
update-bad-signature = no se pudo verificar la versión ({ $error }); no se instaló nada
update-check-bad-signature = no se pudo verificar la versión más reciente de freemapper ({ $error }); no se instaló nada
update-no-manifest = la versión más reciente no puede instalarse por sí sola (no tiene manifiesto de actualización); descárgala de { $page }
update-no-manifest-version = la versión { $version } no existe o no puede instalarse por sí sola (no tiene manifiesto de actualización); consulta { $page }
update-no-asset = la versión no tiene programa para este sistema ({ $platform }); no se instaló nada
update-no-previous = no hay una versión anterior a la que volver (falta { $path })
update-failed = la actualización falló: { $error }
update-mode-set = Modo de actualización establecido en { $mode } en { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } anula este ajuste mientras esté definido.
update-mode-failed = no se pudo escribir { $path }: { $error }
update-notice = freemapper { $new } está disponible (tienes { $old }). Actualiza con: freemapper update
update-applies-next-start = La nueva versión se aplica desde el próximo inicio de freemapper.
update-install-failed = freemapper { $new } no se instaló: { $error }
update-no-config-dir = este sistema no tiene carpeta de configuración; define FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Nota para la persona: al arrancar, el servidor de freemapper encontró esto en las carpetas de adaptadores. Transmítelo; la persona corrige la carpeta o la elimina y luego reinicia el servidor.
    { $lines }
update-mcp-note = Nota para la persona: freemapper { $new } está disponible (esta versión es { $old }). Pregúntale antes de llamar a install_update.
update-mcp-needs-confirm = no se instaló nada; pregunta primero a la persona y llama a install_update con confirm: true cuando acepte
update-mcp-note-manual = Nota para la persona: freemapper { $new } está disponible (esta versión es { $old }), pero install_update no puede instalarla; instala a mano { $needed } o una más reciente, consulta { $notes }
update-mcp-note-bad-signature = Nota para la persona: no se pudo verificar la versión más reciente de freemapper ({ $error }); no se instaló nada. Díselo y no la instales de otra forma.
update-mcp-restart-now = Reinicia ahora el servidor MCP de freemapper; todavía ejecuta la versión anterior.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = error
error-cancelled = cancelado
cli-cancelled-kept = cancelado; conservado: { $files }
error-io = { $path }: { $message }
error-io-disk-full = el disco está lleno, así que freemapper no pudo escribir { $path }. Libera espacio e inténtalo de nuevo

## Errors: project

error-project-already-exists = el proyecto ya existe: { $path }
error-project-map-exists = el mapa ya existe (puede contener tu trabajo); usa --force para sustituirlo: { $path }
error-project-invalid-scale = escala no válida 1:{ $scale }
error-project-invalid-contour-interval = equidistancia de curvas de nivel no válida { $interval } m en freemapper.toml; debe ser mayor que 0
error-project-scale-not-allowed = la escala 1:{ $scale } no está permitida para mapas de { $kind }; usa { $allowed }
error-project-scale-not-allowed-enlarge = la escala 1:{ $scale } no está permitida para mapas de { $kind }; usa { $allowed }. Dibuja a 1:4000, freemapper-print amplía a 1:3000 al imprimir
word-kind-schoolyard = patio escolar
word-kind-sprint = sprint
word-kind-forest = bosque
error-project-toml = archivo de proyecto no válido: { $message }
error-project-unknown-adapter = el archivo de proyecto nombra el adaptador "{ $adapter }", que este freemapper no tiene; quizá lo traiga un freemapper más nuevo (`freemapper update`), o viene de tu carpeta de adaptadores (`freemapper adapters list` lo muestra) o, con --adapters-from-project en la línea de comandos, de la carpeta `adapters` del proyecto (solo para un proyecto de confianza)
error-project-not-fetched = aún no se descargó ninguna tesela; ejecuta primero `freemapper fetch`
error-project-ocd = exportación a OCAD: { $message }
error-project-template-outdated = { $path }: la imagen y el world file no coinciden con el área del mapa; ejecuta `freemapper derive` otra vez
error-project-tiles-outdated = el área del mapa necesita las teselas { $tiles }, que no se descargaron; ejecuta `freemapper fetch`
error-project-buildings-not-fetched = no hay edificios de ALKIS guardados para esta área del mapa; ejecuta primero `freemapper fetch` con conexión
error-project-orthophoto-not-fetched = no hay ortofoto guardada para esta área del mapa; ejecuta primero `freemapper fetch` con conexión
error-project-terrain-not-fetched = no hay modelo del terreno guardado para esta área del mapa; ejecuta primero `freemapper fetch` con conexión
error-project-candidates-outdated = los candidatos de OSM guardados no cubren el área del mapa; ejecuta `freemapper fetch`
error-project-stored = { $path }: { $message }; ejecuta `freemapper fetch` otra vez
error-project-area-too-large = el área del mapa más el margen mide { $width } × { $height } m, más de { $limit } m por lado; reduce el área del mapa o buffer_m en freemapper.toml
error-project-area-crosses-country = el área del mapa está a ambos lados de una frontera nacional; los mapas que cruzan una frontera aún no se admiten, así que dibuja el área dentro de un solo país
error-project-permits-not-for-region = las notas de permisos aún no están disponibles para esta región: consultan servicios alemanes y nombran autoridades alemanas
error-project-outside-coverage = el área del mapa queda fuera de los estados federados admitidos ({ $states }); por ahora freemapper solo tiene fuentes de datos para ellos

## Errors: disk space

error-space-required = no hay suficiente espacio en disco para { $part }. Necesita { $needed }, pero en { $path } solo hay { $left } libres además de la reserva de { $reserve }. Libera espacio y ejecuta `freemapper fetch` otra vez
error-space-skipped = freemapper omite { $part } por falta de espacio en disco. Necesita { $needed }, pero en { $path } solo hay { $left } libres además de la reserva de { $reserve }. Libera espacio y ejecuta `freemapper fetch` otra vez
word-part-map-data = los datos de mapa de ALKIS y OSM
word-part-terrain = el modelo del terreno
word-part-orthophoto = la ortofoto
word-part-surface = el modelo de superficie para la plantilla nDSM
word-part-osm-update = la actualización del extracto de OSM en caché
word-part-k-five = la plantilla del mapa K5 de Berlín
word-part-berlin-layers = las estructuras ALKIS de Berlín, el catastro de árboles, el levantamiento de calles y las capas de puntos
word-part-state-layers = las estructuras ALKIS completas, los objetos del modelo del paisaje y los catastros de árboles
word-part-point-cloud = las nubes de puntos láser para los candidatos de vegetación

## Errors: downloads

error-fetch-http = falló la descarga de { $url }: { $message }
error-fetch-corrupt-archive = la descarga de { $url } no es un archivo de teselas completo: { $message }
error-fetch-corrupt-extract = la descarga de { $url } no es un extracto de OSM completo: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] la tesela { $tiles } no está cubierta por las teselas publicadas; reduce el área del mapa o buffer_m en freemapper.toml
        [many] las teselas { $tiles } no están cubiertas por las teselas publicadas; reduce el área del mapa o buffer_m en freemapper.toml
       *[other] las teselas { $tiles } no están cubiertas por las teselas publicadas; reduce el área del mapa o buffer_m en freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] sin conexión y sin caché: tesela { $tiles } de { $product }; ejecuta `freemapper fetch` con conexión
        [many] sin conexión y sin caché: teselas { $tiles } de { $product }; ejecuta `freemapper fetch` con conexión
       *[other] sin conexión y sin caché: teselas { $tiles } de { $product }; ejecuta `freemapper fetch` con conexión
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = modelo del terreno (DGM1)
word-product-image-dsm = modelo de superficie (bDOM)
word-product-saxony-dtm = modelo del terreno (GeoSN DGM1)
word-product-saxony-dsm = modelo de superficie (GeoSN DOM1)
error-fetch-osm-extract-missing = no hay ningún extracto de OSM en caché; ejecuta `freemapper fetch` con conexión
error-fetch-not-found = { $name } no se encontró en el servidor
error-fetch-refused = { $url }: el servidor rechazó la solicitud (HTTP { $status })
error-fetch-refused-detail = { $url }: el servidor rechazó la solicitud (HTTP { $status }): { $detail }
error-fetch-too-large = la descarga de { $url } es mayor de { $limit } MB; detenida
error-fetch-not-stored = no hay datos guardados para esta área del mapa ({ $label }): falta { $path }, no se puede leer o no la cubre; ejecuta primero `freemapper fetch` con conexión
error-fetch-range-ignored = { $url }: el servidor ignoró el rango de bytes { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper no envía solicitudes a esta dirección: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper no envía solicitudes a esta dirección: el adaptador de { $name } no nombra el host { $host }{ $from }. Si el servicio se ha movido allí, permite el host con `freemapper adapters allow-host { $adapter } { $host }` e infórmalo en https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper no envía solicitudes a esta dirección: el adaptador { $adapter } no nombra el host { $host }{ $from }. Si el servicio se ha movido allí, añade el host a `[net] hosts` del manifiesto del adaptador
error-fetch-host-redirected-from = {" "}({ $url } redirige allí)
error-reason-not-a-host-name = no es un nombre de host
error-reason-not-ascii = un nombre fuera de ASCII debe escribirse en su forma xn--
error-reason-not-a-dns-name = no es un nombre DNS
error-reason-no-domain = un nombre de este ordenador o sin dominio
error-reason-local-name = un nombre de la red local
error-reason-ip-address = una dirección IP, no un nombre DNS
error-reason-unexpected-character = carácter inesperado
error-reason-not-https = no es https
error-reason-user-info = información de usuario
error-reason-port = un puerto distinto de 443
error-reason-host-not-declared = el host no es uno de los hosts declarados
error-reason-local-address = una dirección de este ordenador o de la red local, no de un servidor de internet
error-reason-too-many-redirects = más de { $count } redirecciones
error-reason-redirect-nowhere = una redirección (HTTP { $status }) a ninguna parte
error-fetch-currency-csv = CSV de fechas de los datos, línea { $line }: { $message }
error-alkis = datos de ALKIS no válidos: { $message }
error-osm = datos de OSM: { $message }

## Errors: templates and rasters

error-derive-zip = archivo de teselas: { $message }
error-derive-image = imagen: { $message }
error-derive-no-tiles = no hay teselas de las que derivar
error-derive-area-too-large = área del mapa demasiado grande para una plantilla: { $width } × { $height } píxeles (como máximo { $limit } por lado); reduce el área del mapa o buffer_m en freemapper.toml
error-raster-world-file = world file no válido: { $message }
error-raster-misaligned = las tramas no están alineadas en la misma cuadrícula de píxeles: { $message }
error-raster-pixel-size = las tramas tienen distinto tamaño de píxel: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = GeoJSON no válido: { $message }
error-area-no-polygon = el GeoJSON no contiene ningún polígono
error-area-multi-polygon = MultiPolygon con { $parts } partes; divide el área del mapa en un solo polígono
error-area-out-of-range = coordenada fuera de rango: lat { $lat }, lon { $lon }
error-area-not-an-area = no es un área de mapa utilizable: { $message }
error-georef-date = la fecha { $date } queda fuera de la validez del World Magnetic Model
error-georef-map-range = el punto E { $easting } N { $northing } queda más allá del rango de coordenadas del mapa
error-omap-xml = XML de OOM no válido: { $message }
error-omap-missing-element = no se encontró el elemento <{ $name }>
error-ocd-duplicate-symbol = el número de símbolo { $symbol } se usa dos veces
error-ocd-unknown-symbol = el objeto se refiere al símbolo { $symbol }, que no está en el archivo
error-ocd-wrong-type = un objeto del símbolo { $symbol } tiene un tipo incorrecto para su símbolo

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Uso:
cli-heading-commands = Comandos:
cli-heading-options = Opciones:
cli-heading-arguments = Argumentos:
cli-help-possible-values = valores posibles
cli-help-default = predeterminado
cli-arg-help = Muestra la ayuda
cli-arg-version = Muestra la versión
cli-help-help = Muestra la ayuda de un comando
cli-arg-help-command = El comando del que mostrar la ayuda, por ejemplo run
cli-parse-hint = Para más información, prueba '--help'.
cli-double-click-start = freemapper abre en el navegador la página donde eliges la zona del mapa. La carpeta del proyecto es { $path }. Cierra esta ventana para terminar el programa.
cli-parse-unknown-argument = argumento inesperado { $arg }
cli-parse-missing-required = falta un argumento obligatorio: { $args }
cli-parse-invalid-value = valor no válido { $value } para { $arg }
cli-parse-invalid-value-choices = valor no válido { $value } para { $arg }; valores posibles: { $values }
cli-parse-invalid-subcommand = comando desconocido { $value }
cli-parse-missing-subcommand = hace falta un comando
cli-parse-too-many-values = valor inesperado { $value } para { $arg }
cli-parse-argument-conflict = { $arg } no se puede usar con { $other }

help-freemapper = Crea proyectos iniciales de OOM a partir de geodatos
help-freemapper-lang = Idioma de los textos del programa, `de` o `en` (predeterminado: FREEMAPPER_LANG, luego el idioma del sistema, luego inglés)
help-init = Crea una carpeta de proyecto para un área de mapa
help-init-dir = Carpeta de proyecto que se creará
help-init-geojson = Archivo GeoJSON con el polígono del área del mapa (EPSG:4326)
help-init-osm = Objeto de OSM del área del mapa, p. ej. way/992423119
help-init-place = Nombre del lugar, p. ej. "Clara-Zetkin-Schule Strausberg"; freemapper usa la mejor coincidencia que tenga área
help-init-part = Qué parte usar de un área de OSM de varias partes separadas (1 es la mayor)
help-init-kind = Tipo de mapa
help-arg-scale = Denominador de la escala de un proyecto nuevo: schoolyard 1000 (predeterminado); sprint 4000 (predeterminado) o 5000; forest 15000 (predeterminado), 10000 o 7500. Otros valores se rechazan, porque freemapper-print los rechaza
help-init-title = Título del mapa para la información periférica (predeterminado: el nombre de la carpeta)
help-export = Escribe el proyecto inicial (mapa OOM, plantillas, atribución) en una carpeta de proyecto
help-export-dir = Carpeta del proyecto
help-arg-force = Sustituye un archivo de mapa existente y conserva el anterior como `<name>.omap.bak`
help-export-ocad = Escribe también un punto de partida para OCAD en `ocad/` (formato OCAD 12); un mapa OOM existente se conserva a menos que indiques --force
help-export-pdf = Escribe también `<name>.pdf`, el mapa imprimible (solo los objetos del mapa, sin candidatos); un mapa OOM existente se conserva a menos que indiques --force
help-export-pdf-candidates = Escribe también `<name>-candidates.pdf`, la impresión de control: el mapa con todos los candidatos, fiel a la posición; un mapa OOM existente se conserva a menos que indiques --force
help-export-training = Escribe también el mapa de entrenamiento `<name>-training.omap` y su PDF `<name>-training.pdf`: el mapa y todos los candidatos, limpiados para imprimirlos tal cual; con --ocad también `ocad/<name>-training.ocd`. Un mapa OOM existente se conserva a menos que indiques --force, y --force sustituye también un mapa de entrenamiento existente
help-export-view = Con --training: escribe también el mapa de entrenamiento sin lo que omite una vista, como `<name>-training-<view>.omap` y su PDF (con --ocad también un archivo OCAD). Vistas: `no-paths` (sin calles, caminos ni escaleras), `brown-only` (solo símbolos marrones y el marco), `no-north-lines`, o una lista `no:<codes>` / `only:<codes>`; combínalas con `+`. Repite la opción para varios archivos
help-export-pdf-scale-bar = Dibuja la barra de escala en los PDF (desactivada por defecto, los mapas alemanes no suelen llevarla; el número de escala se mantiene en el cajetín)
help-export-pdf-sources = Líneas de fuentes en los PDF: `auto` imprime la lista completa y solo la acorta si ocuparía más de 5 líneas, `full` no la acorta nunca, `credit-only` imprime solo la línea de créditos de freemapper (ODbL y dl-de/by-2-0 piden entonces que nombres las fuentes al compartir el mapa). ATTRIBUTION.txt lista siempre todas las fuentes
help-fetch = Descarga los datos del área del mapa (ortofoto, modelo del terreno, nDSM, edificios de ALKIS, OSM)
help-fetch-dir = Carpeta del proyecto
help-fetch-offline = Usa solo la caché, sin acceso a la red
help-arg-cache-dir = Carpeta de caché de las teselas descargadas (predeterminada: la carpeta de caché del sistema + `freemapper`)
help-derive = Calcula las plantillas (ortofoto, sombreado, pendiente, relieve local, nDSM), las curvas de nivel y las formas del terreno a partir de los datos descargados
help-derive-dir = Carpeta del proyecto
help-run = Descarga, deriva y exporta de una vez
help-run-dir = Carpeta del proyecto
help-run-offline = Usa solo la caché, sin acceso a la red
help-run-ocad = Escribe también un punto de partida para OCAD en `ocad/` (formato OCAD 12)
help-run-pdf = Escribe también `<name>.pdf`, el mapa imprimible (solo los objetos del mapa, sin candidatos)
help-run-pdf-candidates = Escribe también `<name>-candidates.pdf`, la impresión de control: el mapa con todos los candidatos, fiel a la posición
help-run-training = Escribe también el mapa de entrenamiento `<name>-training.omap` y su PDF `<name>-training.pdf`: el mapa y todos los candidatos, limpiados para imprimirlos tal cual; con --ocad también `ocad/<name>-training.ocd`. Un mapa de entrenamiento existente se conserva a menos que indiques --force
help-run-view = Con --training: escribe también el mapa de entrenamiento sin lo que omite una vista, como `<name>-training-<view>.omap` y su PDF (con --ocad también un archivo OCAD). Vistas: `no-paths` (sin calles, caminos ni escaleras), `brown-only` (solo símbolos marrones y el marco), `no-north-lines`, o una lista `no:<codes>` / `only:<codes>`; combínalas con `+`. Un archivo de vista existente se conserva a menos que indiques --force
help-run-pdf-scale-bar = Dibuja la barra de escala en los PDF (desactivada por defecto; el número de escala se mantiene en el cajetín)
help-run-pdf-sources = Líneas de fuentes en los PDF: `auto` imprime la lista completa y solo la acorta si ocuparía más de 5 líneas, `full` no la acorta nunca, `credit-only` imprime solo la línea de créditos de freemapper (ODbL y dl-de/by-2-0 piden entonces que nombres las fuentes al compartir el mapa). ATTRIBUTION.txt lista siempre todas las fuentes
help-select = Elige el área del mapa en el navegador dibujándola o seleccionando un lugar. Al guardar se escribe freemapper.toml y, si hace falta, se crea la carpeta del proyecto
help-select-dir = Carpeta del proyecto (se crea al guardar si no existe)
help-select-kind = Tipo de mapa de un proyecto nuevo
help-select-title = Título del mapa de un proyecto nuevo
help-select-no-browser = Solo imprime la dirección de la página; no abre el navegador
help-select-port = Puerto en 127.0.0.1 (predeterminado: uno libre)
help-select-offline = La página empieza con "solo caché, sin red" marcado
help-select-ocad = La página empieza con el proyecto inicial para OCAD marcado
help-select-pdf = La página empieza con el PDF imprimible marcado
help-select-pdf-candidates = La página empieza con la impresión de control (todos los candidatos) marcada
help-select-training = La página empieza con el mapa de entrenamiento marcado
help-select-pdf-scale-bar = La página empieza con la barra de escala en los PDF marcada
help-select-pdf-sources = La página empieza con esta elección de líneas de fuentes en los PDF, como en `run`
help-find = Lista los lugares de OpenStreetMap para un nombre, primero las áreas, con el id para init --osm
help-find-query = Nombre del lugar, p. ej. "Clara-Zetkin-Schule Strausberg"
help-find-json = Imprime las coincidencias como JSON, con los mismos campos que la herramienta MCP find_place
help-mcp = Ofrece las herramientas a un agente de IA por MCP (stdin/stdout)
help-update = Instala un freemapper más nuevo de las versiones de GitHub, tras preguntar. Una compilación desde el código fuente indica en cambio cómo actualizarla
help-update-check = Solo comprueba: código de salida 10 si existe una versión más nueva, 0 si esta está actualizada
help-update-yes = Instala sin preguntar; hace falta cuando la entrada no es un terminal
help-update-version = Instala esta versión (x.y.z) en lugar de la más reciente, también una más antigua
help-update-rollback = Vuelve a la versión que sustituyó la última actualización
help-update-mode = Guarda lo que hace la comprobación diaria cuando encuentra una versión más nueva (predeterminado: notify). Funciona en cualquier compilación, porque solo escribe el ajuste
help-adapters = Muestra los adaptadores de región: los estados federados y países para los que freemapper tiene fuentes de datos
help-adapters-list = Lista los adaptadores con su origen (integrado, tu carpeta, un proyecto), su revisión, el día de la última comprobación y los hosts que consulta
help-adapters-list-json = Imprime la lista como JSON
help-adapters-list-project = Lee también los adaptadores de la carpeta `adapters` de esta carpeta de proyecto, como haría --adapters-from-project
help-adapters-check = Comprueba la carpeta de un adaptador antes de usarlo o de pasarlo a otros: si se cargaría, qué debería saber su autor y, con --live, si sus servicios responden. Código de salida 0: lo supera; 1: tiene hallazgos; 2: la comprobación no pudo ejecutarse
help-adapters-check-dir = La carpeta del adaptador; su nombre es el id del adaptador
help-adapters-check-live = Consulta también los servicios del adaptador para el recuadro que indica --bbox y dice qué entrega cada uno. Las solicitudes se quedan en los hosts que declara el manifiesto
help-adapters-check-bbox = El recuadro de la comprobación en vivo en grados, como minLon,minLat,maxLon,maxLat. Debe estar dentro del límite del adaptador; bastan 150 m por 150 m
help-adapters-check-json = Imprime el resultado como JSON
help-adapters-allow-host = Permite a un adaptador integrado un host más: para un estado federado cuyo servicio se movió a un servidor que su adaptador aún no nombra. El mensaje sobre la dirección rechazada dice qué host. El host se comprueba como el de un manifiesto, y freemapper solo se conecta a servidores de internet
help-adapters-allow-host-adapter = El id del adaptador integrado, como lo muestra `adapters list`
help-adapters-allow-host-host = El host que se permite: un nombre DNS, sin dirección ni puerto
help-adapters-disallow-host = Retira un host que allow-host permitió
help-adapters-disallow-host-adapter = El id del adaptador integrado
help-adapters-disallow-host-host = El host que se retira
help-arg-adapters-from-project = Carga también los adaptadores de la carpeta `adapters` de la carpeta del proyecto. freemapper nombra cada uno con los hosts que consulta. Úsalo en un proyecto de confianza: un adaptador hace que freemapper consulte los servidores que nombra

## The summary of a run, for the MCP server and the area page

cli-internal-error = error interno: { $message }
cli-pdf-sources-invalid = pdf_sources debe ser auto, full o credit-only, no { $value }
cli-view-unknown = una vista es no-paths, brown-only, no-north-lines, no:<codes> u only:<codes>, no { $value }
cli-view-empty = la lista { $value } no nombra ningún código de símbolo
cli-view-bad-code = { $value } no es un código de símbolo como 505 o 501.6
cli-summary-training-view = Escritos { $map } y { $pdf } (una vista del mapa de entrenamiento)
cli-summary-kept-training-view = Se conservan los archivos de la vista { $view } del mapa de entrenamiento; indica --force para renovarlos
cli-summary-training-view-failed = Falló la vista del mapa de entrenamiento: { $error }
cli-estimate-disk-unreadable = no se puede leer el espacio libre en disco de { $path } ({ $message })
cli-summary-wrote-map = Escrito el mapa { $path }
cli-summary-kept-map = Se conserva el mapa existente { $path } (puede contener el trabajo de quien lo dibuja)
cli-summary-map-lacks-templates = El mapa aún no hace referencia a las plantillas de imagen. Inicia el proceso con force para sustituirlo y conservar el anterior como .omap.bak.
cli-summary-map-lacks-buildings = El mapa aún no tiene edificios de ALKIS. Inicia el proceso con force para añadirlos; sustituye el mapa y conserva el anterior como .omap.bak.
cli-summary-map-lacks-canopies = Al mapa le faltan los tejadillos y cocheras de ALKIS (522), que ahora son objetos del mapa. Inicia el proceso con force para añadirlos; sustituye el mapa y conserva el anterior como .omap.bak.
cli-summary-map-light = El mapa base: { $joined } líneas unidas, { $left } ocultas o demasiado pequeñas pasan a las capas de candidatos (no se movió nada)
cli-summary-map-light-kept = Las capas de candidatos contienen ahora además { $left } objetos o tramos que esta versión saca del mapa base (con la etiqueta freemapper:generalized). El mapa conservado aún los tiene, así que aparecen dos veces. Inicia el proceso con force para limpiar el mapa; conserva el anterior como .omap.bak.
cli-summary-map-old-candidates = El mapa aún nombra templates/candidates.omap; esta ejecución lo llenó con todos los candidatos. Ahora los candidatos vienen también en siete capas, un archivo cada una (templates/candidates-*.omap). El mapa las lista después de una ejecución con force (el mapa anterior queda como .omap.bak), o quien dibuja el mapa las añade en OOM (Plantillas > Abrir plantilla…).
cli-summary-wrote-ocad = Escrito el mapa OCAD { $path } ({ $count } objetos)
cli-summary-kept-ocad = Se conserva el mapa OCAD existente { $path } (puede contener el trabajo de quien lo dibuja; force lo sustituye)
cli-summary-ocad-old-candidates = El mapa OCAD aún nombra { $path } como mapa de fondo; esta ejecución lo llenó con todos los candidatos. Ahora los candidatos vienen también en siete capas, un archivo cada una (-candidates-*.ocd). El mapa OCAD las lista después de una ejecución con force (el mapa anterior queda como .ocd.bak), o quien dibuja el mapa las añade en OCAD como mapas de fondo.
cli-summary-ocad-left-out = Omitidos en el mapa OCAD: { $skipped } objetos y { $dropped } partes degeneradas
cli-summary-ocad-symbols = En OCAD, elige Símbolo > Cargar colores y símbolos desde… y selecciona el conjunto de símbolos oficial{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , el conjunto escolar de la IOF 2019
       *[other] {" "}ISSprOM 2019
    }; sustituye por número los símbolos provisionales.
cli-summary-ocad-failed = falló la exportación a OCAD: { $error }
cli-summary-pdf-without-credits = PDF sin créditos de fuentes: ODbL y dl-de/by-2-0 piden que nombres las fuentes al compartir el mapa; consulta ATTRIBUTION.txt
cli-summary-wrote-pdf = Escrito el mapa { $path } (sin candidatos)
cli-summary-pdf-failed = falló la exportación a PDF: { $error }
cli-summary-wrote-pdf-candidates = Escrita la impresión de control { $path } (el mapa con todos los candidatos, fiel a la posición)
cli-summary-pdf-candidates-failed = falló la exportación de la impresión de control: { $error }
cli-summary-training-failed = falló la exportación del mapa de entrenamiento: { $error }

cli-summary-kept-training = Se conserva el mapa de entrenamiento existente { $path }, para que una ejecución no sobrescriba uno que se imprimió o se entregó; bórralo o indica force para renovarlo

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = este sistema no tiene carpeta personal; usa una ruta absoluta
mcp-error-relative-path = { $path } es una ruta relativa; usa una ruta absoluta o una que empiece por ~
mcp-error-home-folder = { $path } es la propia carpeta personal; usa una carpeta dentro de ella, p. ej. ~/maps/school
mcp-error-no-project = no hay ningún proyecto de freemapper en { $path }; usa primero create_project
mcp-error-unknown-kind = tipo desconocido { $kind }; usa "schoolyard", "sprint" o "forest"
mcp-error-reading = leyendo { $path }: { $error }
mcp-error-one-area-source = indica exactamente uno de osm_id (de find_place) o geojson
mcp-error-page-open = la página de selección de área de este proyecto sigue abierta; pide a la persona que guarde el área y pulse Listo en la página, y luego inicia el proceso
mcp-error-no-job = no hay ningún trabajo { $job }; start_pipeline y permits_for_area devuelven los ids de trabajo de esta sesión
mcp-error-job-running = { $job } sigue en marcha para { $path }; solo se ejecuta un proceso a la vez porque los proyectos comparten la caché de descargas. Inicia este cuando project_status indique finished o failed
mcp-error-page-start = no se puede iniciar la página: { $error }
mcp-created-project = Proyecto { $path } creado para { $area } ({ $kind }, 1:{ $scale }). Siguiente: estimate_download y luego start_pipeline.
mcp-created-project-given-area = Proyecto { $path } creado para el área indicada ({ $kind }, 1:{ $scale }). Siguiente: estimate_download y luego start_pipeline.
mcp-page-already-open = La página de esta carpeta ya está abierta; conserva los ajustes con los que se abrió.
mcp-page-next = Pide a la persona que elija el área en la página, pulse Guardar y luego Listo.
mcp-preview-of = Vista previa de { $layer }
mcp-permits-next = Se están haciendo las notas de permisos. Consulta project_status con este job_id; cuando termine, permits_md contiene el texto de permits.md. Transmítelo como orientación, no como asesoramiento jurídico.
mcp-tool-check-update = Si existe una versión más nueva de freemapper: su versión, fecha de publicación, notas de la versión y tamaño de descarga, o que esta está actualizada. No instala nada.
mcp-tool-create-project = Crea una carpeta de proyecto para un área de mapa. Indica osm_id (de find_place) o un archivo GeoJSON. kind es schoolyard (1:1000, predeterminado), sprint (1:4000) o forest (1:15000).
mcp-param-create-project-dir = Carpeta de proyecto nueva (ruta absoluta o que empiece por ~).
mcp-param-create-project-geojson = En lugar de osm_id: un archivo GeoJSON con el polígono del área del mapa (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (predeterminado, 1:1000), "sprint" (1:4000) o "forest" (1:15000).
mcp-param-create-project-osm-id = Objeto de OSM del área del mapa de find_place, p. ej. "way/992423119".
mcp-param-create-project-part = Con osm_id: qué parte usar de un área de varias partes separadas (1 es la mayor).
mcp-param-create-project-scale = Denominador de la escala: schoolyard 1000 (predeterminado); sprint 4000 (predeterminado) o 5000; forest 15000 (predeterminado), 10000 o 7500. Otros valores se rechazan; para 1:3000 dibuja a 4000, freemapper-print amplía al imprimir.
mcp-param-create-project-title = Título del mapa para la información periférica (predeterminado: el nombre de la carpeta).
mcp-tool-estimate-download = Qué descargará start_pipeline para un proyecto: las teselas, cada descarga con su tamaño en MB, si está en caché y si cabe en el disco, los MB que aún faltan por descargar y, en disk, el espacio libre de los volúmenes de la caché y del proyecto con mensajes sobre lo que no cabe (díselo a la persona).
mcp-param-estimate-download-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~).
mcp-tool-find-place = Busca una escuela, un parque u otro lugar por nombre (OpenStreetMap Nominatim). Devuelve hasta 5 coincidencias, primero las áreas; solo las coincidencias con has_area pueden ser un área de mapa. Una lista vacía significa que no se encontró nada.
mcp-param-find-place-query = Nombre de la escuela o del lugar, p. ej. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Instala la versión más reciente de freemapper sobre este programa. Llámala solo después de que la persona haya aceptado, con confirm true. La nueva versión se aplica cuando el asistente reinicia el servidor de freemapper.
mcp-param-install-update-confirm = true cuando la persona aceptó instalar la versión más nueva; sin ello no se instala nada.
mcp-tool-list-layers = Las capas de un proyecto: el mapa, las siete capas de candidatos y las plantillas ráster, cada una con su archivo, si ya está creada y qué paso la crea.
mcp-param-list-layers-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~).
mcp-tool-open-area-selector = Abre en el navegador una página donde la persona dibuja el área del mapa (rectángulo o polígono) o elige un lugar, ve las teselas y el tamaño de la descarga, y la guarda; al guardar se crea la carpeta del proyecto o se cambia su área. Devuelve la URL que hay que dar a la persona. La página sigue activa hasta que la persona pulsa Listo; llamar de nuevo para la misma carpeta devuelve la misma URL.
mcp-param-open-area-selector-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~); se crea cuando la persona guarda.
mcp-param-open-area-selector-kind = Tipo de mapa de un proyecto nuevo: "schoolyard" (predeterminado, 1:1000), "sprint" (1:4000) o "forest" (1:15000).
mcp-param-open-area-selector-ocad = El panel de ejecución empieza con el proyecto inicial para OCAD marcado. Por defecto false.
mcp-param-open-area-selector-offline = El panel de ejecución empieza con "solo caché, sin red" marcado. Por defecto false.
mcp-param-open-area-selector-open-browser = Abre también la página en el navegador predeterminado de este ordenador. Por defecto true.
mcp-param-open-area-selector-pdf = El panel de ejecución empieza con el PDF imprimible marcado. Por defecto false.
mcp-param-open-area-selector-pdf-candidates = El panel de ejecución empieza con la impresión de control (todos los candidatos) marcada. Por defecto false.
mcp-param-open-area-selector-pdf-scale-bar = El panel de ejecución empieza con la barra de escala en los PDF marcada. Por defecto false.
mcp-param-open-area-selector-pdf-sources = Líneas de fuentes en los PDF con las que empieza el panel: "auto" (predeterminado), "full" o "credit-only", como en start_pipeline.
mcp-param-open-area-selector-permits = El panel de ejecución empieza con las notas de permisos marcadas. Por defecto false.
mcp-param-open-area-selector-scale = Denominador de la escala de un proyecto nuevo: schoolyard 1000 (predeterminado); sprint 4000 (predeterminado) o 5000; forest 15000 (predeterminado), 10000 o 7500. Otros valores se rechazan.
mcp-param-open-area-selector-title = Título del mapa de un proyecto nuevo.
mcp-param-open-area-selector-training = El panel de ejecución empieza con el mapa de entrenamiento marcado. Por defecto false.
mcp-tool-preview-layer = Una vista previa PNG (de 800 px como máximo) de una capa: map (la imagen de los objetos del mapa que escribió la última ejecución, sin los candidatos), o una plantilla ráster: orthophoto, hillshade, slope, relief, ndsm, o k5 en Berlín.
mcp-param-preview-layer-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" o "k5" (solo Berlín).
mcp-tool-project-status = Estado de un trabajo de start_pipeline o permits_for_area: running, finished o failed, las últimas líneas de progreso, los avisos, los archivos escritos y el resultado o el error. Un trabajo permits_for_area terminado lleva además el texto de permits.md como permits_md.
mcp-param-project-status-job-id = Id de trabajo de start_pipeline o permits_for_area, p. ej. "job-1".
mcp-tool-permits-for-area = Crea las notas de permisos de un proyecto: las unidades administrativas del área del mapa, las áreas protegidas que contiene y quién gestiona allí el terreno, con las autoridades a las que preguntar; una orientación, no asesoramiento jurídico. Escribe permits.md y permits.json en la carpeta. Se ejecuta en segundo plano y devuelve un job_id; consulta project_status, cuya respuesta lleva el texto de permits.md como permits_md cuando el trabajo termina. Solo se ejecuta un trabajo a la vez.
mcp-param-permits-for-area-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~).
mcp-param-permits-for-area-offline = Lee solo los datos que una ejecución anterior guardó en el proyecto, sin acceso a la red. Por defecto false.
mcp-tool-start-pipeline = Descarga los datos, calcula las plantillas y escribe el proyecto inicial (fetch, derive, export). Se ejecuta en segundo plano y devuelve un job_id; consulta project_status. Solo se ejecuta un proceso a la vez. Conserva un mapa existente a menos que force sea true.
mcp-param-start-pipeline-dir = Carpeta del proyecto (ruta absoluta o que empiece por ~).
mcp-param-start-pipeline-force = Sustituye un archivo de mapa existente y conserva el anterior como .omap.bak. Por defecto false.
mcp-param-start-pipeline-ocad = Escribe también un punto de partida para OCAD en la carpeta ocad/ (formato OCAD 12, para quienes editan en OCAD). Por defecto false.
mcp-param-start-pipeline-offline = Usa solo la caché de descargas, sin acceso a la red. Por defecto false.
mcp-param-start-pipeline-pdf = Escribe también <name>.pdf, el mapa listo para imprimir (solo los objetos del mapa, sin candidatos). Por defecto false.
mcp-param-start-pipeline-pdf-candidates = Escribe también <name>-candidates.pdf, la impresión de control: el mapa con todos los candidatos, fiel a la posición (compruébalos sobre el terreno). Por defecto false.
mcp-param-start-pipeline-pdf-scale-bar = Dibuja la barra de escala en los PDF. Por defecto false (los mapas alemanes no suelen llevarla; el número de escala se mantiene en el cajetín).
mcp-param-start-pipeline-pdf-sources = Líneas de fuentes en los PDF: "auto" (predeterminado; la lista completa, acortada solo si ocupa más de 5 líneas), "full" (nunca acortada) o "credit-only" (solo la línea de créditos de freemapper; ODbL y dl-de/by-2-0 exigen entonces nombrar las fuentes al compartir el mapa). ATTRIBUTION.txt lista siempre todas las fuentes.
mcp-param-start-pipeline-permits = Escribe también las notas de permisos (permits.md y permits.json) después del mapa: unidades administrativas, áreas protegidas y quién gestiona el terreno. Un fallo ahí es un aviso, no un trabajo fallido. Por defecto false.
mcp-param-start-pipeline-training = Escribe también el mapa de entrenamiento <name>-training.omap y su PDF <name>-training.pdf: el mapa y todos los candidatos, limpiados para imprimirlos tal cual; con ocad también ocad/<name>-training.ocd. Un mapa de entrenamiento existente se conserva a menos que force sea true. Por defecto false.
mcp-param-start-pipeline-training-views = Con training: escribe también el mapa de entrenamiento sin lo que omite una vista, un juego de archivos por entrada: "no-paths", "brown-only", "no-north-lines", o una lista como "no:505,506" o "only:101,102", combinadas con +. Los archivos son <name>-training-<view>.omap y su PDF.
mcp-instructions = freemapper crea proyectos iniciales de OpenOrienteering Mapper para mapas de orientación (patio escolar 1:1000, sprint 1:4000, bosque 1:15000) a partir de geodatos oficiales de { $states } y de OSM. Flujo de trabajo: find_place (busca la escuela o el área por nombre; elige un resultado con has_area) → create_project (una carpeta nueva, ruta absoluta, con el osm_id) → estimate_download (di a la persona cuánto se descargará) → start_pipeline (devuelve un id de trabajo; la descarga puede tardar minutos) → project_status (consulta cada pocos segundos hasta que el estado sea finished o failed) → list_layers y preview_layer (muestra las plantillas a la persona). Después la persona abre el archivo del mapa en OpenOrienteering Mapper. Cuando ningún lugar encaja o la persona quiere dibujar el área, open_area_selector abre en el navegador una página donde la persona dibuja o elige el área y la guarda (esto crea el proyecto); continúa con estimate_download cuando la persona diga que está guardada. Las rutas deben ser absolutas o empezar por ~. Solo se ejecuta un proceso a la vez. permits_for_area crea las notas de permisos de un proyecto (unidades administrativas, áreas protegidas, quién gestiona el terreno; una orientación, no asesoramiento jurídico) como un trabajo igual que start_pipeline. check_update dice si existe un freemapper más nuevo; install_update lo instala, solo con confirm: true después de que la persona haya aceptado, y luego el asistente reinicia el servidor de freemapper.

cli-contour-interval-saved = Equidistancia de las curvas de nivel guardada en freemapper.toml: { $interval } m; derive traza las curvas de nivel con ella
cli-contour-interval-auto = Equidistancia de las curvas de nivel en freemapper.toml: automática (la del tipo de mapa; un mapa de bosque recibe 2,5 m en terreno llano y 10 m en terreno empinado)
help-arg-contour-interval = Equidistancia de las curvas de nivel en metros, p. ej. `2.5`, o `auto` para la del tipo de mapa (un mapa de bosque: 5 m, 2,5 m donde es llano, 10 m donde es empinado). Se guarda en freemapper.toml; derive traza las curvas de nivel con ella
cli-content-saved = Ajuste de contenido guardado en freemapper.toml: { $summary }
cli-content-invalid = No se puede leer el ajuste de contenido: { $detail }
help-arg-content = Qué va directamente al mapa: `safe` deja cada objeto donde lo pone la generación, `balanced` (predeterminado) pone en el mapa los objetos del catastro y de los registros, `full` pone todo en el mapa. La elección se guarda en freemapper.toml
help-arg-content-set = Envía un grupo de objetos a `map`, `candidates` u `off`, como `GROUP=PLACE`, p. ej. `walls=map`; se puede repetir. Grupos: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Se guarda en freemapper.toml
mcp-param-create-project-content = Qué va directamente al mapa: "safe", "balanced" (predeterminado) o "full". Se guarda en el proyecto. balanced pone en el mapa los objetos del catastro y de los registros, full pone todo en él.
mcp-param-create-project-content-set = Grupos enviados a un sitio distinto del que indica el perfil, como "group=place" con place map, candidates u off, p. ej. "walls=map". Grupos: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Equidistancia de las curvas de nivel en metros, p. ej. 2.5; 0 vuelve a la elección automática (la del tipo de mapa; un mapa de bosque recibe 2,5 m en terreno llano y 10 m en terreno empinado). Se guarda en el proyecto; el próximo derive traza las curvas de nivel con ella.
mcp-param-start-pipeline-content = Cambia qué va directamente al mapa antes de la ejecución: "safe", "balanced" o "full". Se guarda en el proyecto. Por defecto: el ajuste del proyecto y, si no, balanced.
mcp-param-start-pipeline-content-set = Grupos enviados a un sitio distinto del que indica el perfil, como "group=place" con place map, candidates u off, p. ej. "fences=candidates". Se guarda en el proyecto.
mcp-param-start-pipeline-contour-interval = Equidistancia de las curvas de nivel en metros, p. ej. 2.5; 0 vuelve a la elección automática (la del tipo de mapa; un mapa de bosque recibe 2,5 m en terreno llano y 10 m en terreno empinado). Se guarda en el proyecto; el próximo derive traza las curvas de nivel con ella.
mcp-error-content = No se puede leer el ajuste de contenido: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Escribe las notas de permisos de un proyecto (permits.md y permits.json): las unidades administrativas, áreas protegidas y gestores que toca el área del mapa, y a quién suele haber que preguntar. Consulta al BfN y al BKG y, donde el estado federado del área del mapa tiene servicios abiertos propios, a estos mediante los hosts de su adaptador (lista en el manual, capítulo 12); en Renania del Norte-Westfalia descarga la primera vez unos 32 MB para esto, en Brandeburgo unos 4 MB, después solo archivos más nuevos. Lee el extracto de OSM que ya está en caché; --offline usa lo descargado antes
help-permits-dir = Carpeta del proyecto
help-permits-offline = Usa solo los datos descargados antes, sin acceso a la red
help-run-permits = Escribe también las notas de permisos (permits.md y permits.json) después del mapa; consulta al BfN, al BKG y a los servicios abiertos del estado federado (manual, capítulo 12). Un fallo ahí es un aviso y el mapa se conserva
cli-wrote-permits = Escritas las notas de permisos { $path }
cli-summary-wrote-permits = Escritas las notas de permisos { $path }
cli-permits-failed = no se pudieron escribir las notas de permisos: { $error }
cli-permits-source-left-out = las notas de permisos omiten { $source }: { $reason }
cli-permits-not-read = no se leyó
error-permits-stored = { $path }: { $message }; vuelve a descargar las notas de permisos con conexión
error-permits-nothing-read = no se pudo leer ninguna fuente de las notas de permisos ({ $missing }); descárgalas primero con conexión
error-permits-missing-not-read = { $source }: no se leyó

permits-title = Notas de permisos
permits-disclaimer = Una orientación, no asesoramiento jurídico. Deciden las autoridades competentes. Para la fecha y la exactitud de los datos consulta la tabla de "Fuentes y fecha de los datos"; pueden faltar áreas o estar desactualizadas (BfN: fusionado una vez al año, "Nicht für Planungszwecke geeignet.", es decir, no apto para planificación).
permits-disclaimer-regional = Una orientación, no asesoramiento jurídico. Deciden las autoridades competentes. Para la fecha y la exactitud de los datos consulta la tabla de "Fuentes y fecha de los datos"; pueden faltar áreas o estar desactualizadas.
permits-bfn-merge = El BfN fusiona una vez al año los datos de los estados federados y los marca como no aptos para planificación. Deciden las autoridades del estado federado.
permits-created = Creadas el { $date } para el área del mapa de un { $kind ->
        [schoolyard] mapa de patio escolar
        [forest] mapa de bosque
       *[other] mapa de sprint
    } ({ $area } ha).
permits-section-admin = Unidades administrativas
permits-section-protected = Áreas protegidas (BfN)
permits-section-osm-protected = Más áreas protegidas de OpenStreetMap
permits-section-operators = Gestores y propietarios según OpenStreetMap
permits-section-hints = Indicaciones
permits-section-sources = Fuentes y fecha de los datos
permits-none-admin = VG250 no nombra ninguna unidad administrativa para el área del mapa.
permits-none-protected = Según el BfN, ningún área protegida está en el área del mapa.
permits-none-osm-protected = OpenStreetMap no nombra ningún área protegida en el área del mapa que el BfN no liste ya.
permits-none-operators = OpenStreetMap no nombra ningún gestor ni propietario para el área del mapa.
permits-none-hints = Sin indicaciones.
permits-section-not-read = No se leyeron los datos de { $source }, así que esta sección está vacía, aunque puede haber entradas.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Áreas administrativas 1:250 000 (VG250) del BKG
        [bfn] Áreas protegidas de Alemania del BfN
       *[osm] OpenStreetMap, del extracto que freemapper guarda en su caché
    }
permits-level = { $level ->
        [land] Estado federado (Land)
        [kreis] Distrito (Kreis)
        [verwaltungsgemeinschaft] Amt o Verwaltungsgemeinschaft (mancomunidad)
       *[gemeinde] Municipio (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Tocadas en el borde (menos del 0,5% del área del mapa):
permits-at-edge-generalised = VG250 está generalizado a la escala 1:250 000. Una unidad que solo toca el borde puede ser un efecto de ello.
permits-kind = { $kind ->
        [nationalpark] Parque nacional
        [nationales-naturmonument] Monumento natural nacional
        [naturschutzgebiet] Reserva natural (Naturschutzgebiet)
        [ffh] Zona FFH (Natura 2000)
        [vogelschutzgebiet] Zona de protección de aves (Natura 2000)
        [biosphaerenreservat] Reserva de la biosfera
        [biosphaerenreservat-zone] Zona de una reserva de la biosfera
        [landschaftsschutzgebiet] Área de protección del paisaje (Landschaftsschutzgebiet)
       *[naturpark] Parque natural
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Declarado el { $date }
permits-protected-title = Base legal: { $title }
permits-protected-link = Texto legal: { $url }
permits-protected-site-link = Página del sitio (Natura 2000): { $url }
permits-protected-year = Año de los datos en el BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = clase de protección { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = sin nombre
permits-osm-operator = Gestor según OSM: { $operator }
permits-osm-enclosing-gap = Aquí pueden faltar áreas de OpenStreetMap que encierran toda el área del mapa sin tener una esquina dentro, y también áreas protegidas mapeadas en OSM como relación con `type=boundary`. Las partes del BfN y de VG250 no se ven afectadas.
permits-osm-enclosing-gap-short = Aquí pueden faltar las superficies de OpenStreetMap que rodean toda el área del mapa sin tener una esquina dentro de ella.
permits-operator-line = { $name } ({ $roles ->
        [operator] gestor
        [owner] propietario
       *[both] gestor y propietario
    }): { $objects }
permits-column-source = Fuente
permits-column-state = Estado
permits-column-credit = Crédito y licencia
permits-state-read = leído, descargado el { $date }
permits-state-read-osm = leído, extracto de OSM del { $date }
permits-state-unreachable = no accesible ({ $reason })
permits-state-not-read = no leído
permits-no-credit = no usado
permits-hint-city-state = El área del mapa está en Berlín o Hamburgo. Allí suele ser competente el Bezirksamt (oficina del distrito) o una autoridad del estado. Estas notas aún no determinan el Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Las áreas protegidas competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde). Aquí es la propia ciudad de { $kreis }.
        [no] Las áreas protegidas competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde), normalmente el { $kreis }.
       *[unknown] Las áreas protegidas competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde) competente, normalmente el Landkreis o la kreisfreie Stadt (ciudad con rango de distrito).
    }
permits-hint-national-park = Un parque nacional tiene normas propias. Pregunta pronto a la administración del parque nacional.
permits-hint-nsg-paths = En una reserva natural normalmente hay que ceñirse a los caminos. Para una carrera fuera de los caminos, pregunta a la autoridad de conservación de la naturaleza.
permits-hint-natura2000 = Una zona FFH o una zona de protección de aves (Natura 2000) toca el área del mapa. Para un evento, la autoridad de conservación de la naturaleza puede pedir una evaluación de sus efectos.
permits-hint-protected-areas-pl = En el área del mapa hay zonas u objetos protegidos de conservación de la naturaleza. Para un parque nacional, un parque paisajístico o una reserva natural, el contacto habitual es la administración del parque o de la reserva; para las demás zonas y objetos, la Dirección Regional de Protección del Medio Ambiente (RDOŚ) del voivodato suele poder decir quién es competente. Consulta con antelación si planeas un evento allí.
permits-hint-order-office = { $amt ->
        [none] Para calles, plazas y otros terrenos públicos suele ser competente el Ordnungsamt (oficina de orden público) del municipio (Gemeinde) { $gemeinde }.
       *[other] Para calles, plazas y otros terrenos públicos suele ser competente el Ordnungsamt (oficina de orden público): { $amt }, para el municipio (Gemeinde) { $gemeinde }.
    }
permits-hint-park-operator = Gestor de un parque o jardín en el área del mapa según OpenStreetMap: { $operator }. Pregúntale antes de usarlo.
permits-hint-school-operator = Gestor de la escuela o institución según OpenStreetMap: { $operator }. Para sus terrenos, pregunta a la dirección de la escuela o a este organismo.
permits-section-districts = Bezirke (distritos, { $source })
permits-section-green-spaces = Zonas verdes ({ $source })
permits-section-owners = Terrenos del estado y de la Federación ({ $source })
permits-section-monuments = Monumentos ({ $source })
permits-section-protected-objects = Monumentos naturales y otros objetos protegidos ({ $source })
permits-none-districts = Según { $source }, el área del mapa no está en ningún Bezirk.
permits-none-green-spaces = Según { $source }, ninguna zona verde pública está en el área del mapa.
permits-none-owners = Según { $source }, ninguna parcela del estado ni de la Federación está en el área del mapa.
permits-none-monuments = Según { $source }, ningún monumento está en el área del mapa.
permits-none-protected-objects = Según { $source }, ninguno de los objetos protegidos que contiene está en el área del mapa.
permits-none-sampled = En los puntos consultados, { $source } no nombra nada aquí.
permits-gaps = Los datos de { $source } no están completos: { $gaps }.
permits-sampled = { $source } responde solo para puntos sueltos. freemapper consultó en una cuadrícula de puntos sobre el mapa: los porcentajes son estimaciones y puede faltar lo que queda entre los puntos.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = destino: { $dedication }
permits-playgrounds-none = Ningún parque infantil público está en el área del mapa.
permits-playgrounds-one = Un parque infantil público está en el área del mapa. No se lista por separado.
permits-playgrounds-many = { $count } parques infantiles públicos están en el área del mapa. No se listan uno por uno.
permits-owner-other = Otros propietarios públicos
permits-owners-rest = Para el resto del área los datos no nombran ningún propietario.
permits-monuments-total = Monumentos total o parcialmente en el área del mapa: { $count }.{ $links ->
        [yes] {" "}Los enlaces llevan a sus entradas en la lista de monumentos.
       *[no] {""}
    }
permits-monuments-links-left = { $count } entradas más solo se cuentan aquí, sin enlace.
permits-object-kind = { $kind ->
        [natural-monument] Monumento natural (Naturdenkmal)
        [protected-landscape-part] Elemento protegido del paisaje (geschützter Landschaftsbestandteil)
        [biotope] Biotopo protegido por ley (geschütztes Biotop)
        [national-park] Parque nacional (park narodowy)
        [park-buffer-zone] Zona de amortiguación de un parque (otulina)
        [landscape-park] Parque paisajístico (park krajobrazowy)
        [nature-reserve] Reserva natural (rezerwat przyrody)
        [protected-landscape-area] Área de paisaje protegido (obszar chronionego krajobrazu)
        [natura2000-habitats] Sitio Natura 2000 de hábitats (SOO)
        [natura2000-birds] Sitio Natura 2000 de aves (OSO)
        [ecological-site] Espacio ecológico (użytek ekologiczny)
        [landscape-complex] Complejo natural y paisajístico (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Sitio de documentación (stanowisko dokumentacyjne)
        [natural-monument-pl] Monumento natural (pomnik przyrody)
       *[special-rules] Área con normas especiales
    }
permits-object-count = { $count } objetos
permits-object-citations = Bases legales:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Para calles, plazas y zonas verdes del Bezirk { $district } suele ser competente el Bezirksamt { $district }, allí el Fachamt Management des öffentlichen Raumes (oficina del espacio público).
       *[berlin] Para calles, plazas y zonas verdes del Bezirk { $district } suelen ser competentes el Ordnungsamt (oficina de orden público) y el Straßen- und Grünflächenamt (oficina de calles y parques) del Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Para los monumentos del área del mapa es competente el Denkmalschutzamt Hamburg (oficina de monumentos).
       *[berlin] Para los monumentos del Bezirk { $district } es competente la autoridad inferior de monumentos (untere Denkmalschutzbehörde) del Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Con los datos de aquí no se puede saber con certeza si hay monumentos protegidos en el área del mapa.
       *[yes] Hay monumentos protegidos en el área del mapa.
    } Si una carrera los toca, por ejemplo con un control en un monumento o un tramo por un lugar protegido, lo decide la autoridad inferior de monumentos (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Aquí es la propia ciudad de { $kreis }.
        [no] , normalmente el { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; en algunos estados federados es el municipio { $gemeinde }.
        }
       *[unknown] {" "}(normalmente el Landkreis o la kreisfreie Stadt, en algunos estados federados el municipio).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Los monumentos naturales, los elementos protegidos del paisaje y los biotopos protegidos del mapa competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde). Aquí es la propia ciudad de { $kreis }.
        [no] Los monumentos naturales, los elementos protegidos del paisaje y los biotopos protegidos del mapa competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde), normalmente el { $kreis }.
       *[unknown] Los monumentos naturales, los elementos protegidos del paisaje y los biotopos protegidos del mapa competen a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde), normalmente el Landkreis o la kreisfreie Stadt.
    }
permits-hint-biotope = Hay biotopos protegidos por ley en el área del mapa{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Está prohibido todo lo que pueda destruirlos o dañarlos considerablemente (§ 30 BNatSchG). Coloca controles y tramos fuera de ellos, o pregunta a la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Para los monumentos naturales y los elementos protegidos del paisaje del Bezirk { $district } suele ser competente el Bezirksamt { $district }.
       *[berlin] Para los monumentos naturales y los elementos protegidos del paisaje del Bezirk { $district } es competente la autoridad inferior de conservación de la naturaleza (untere Naturschutzbehörde) del Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Según los datos abiertos del estado, { $share } del área del mapa pertenece a otros propietarios públicos.
       *[no] Según los datos abiertos del estado, { $share } del área del mapa pertenece al propietario "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Los propietarios del resto del área no se pueden averiguar con datos abiertos. Con un interés legítimo (por ejemplo, como organizador) puedes pedir un extracto al registro de la propiedad (Grundbuchamt); para terrenos públicos ayuda el Ordnungsamt competente o el departamento de parques.
       *[no] Los propietarios de estas áreas no se pueden averiguar con datos abiertos. Con un interés legítimo (por ejemplo, como organizador) puedes pedir un extracto al registro de la propiedad (Grundbuchamt); para terrenos públicos ayuda el Ordnungsamt competente o el departamento de parques.
    }
permits-section-forest-units = Administración forestal ({ $source })
permits-section-state-forest = Bosque del estado ({ $source })
permits-none-forest-units = Según { $source }, el área del mapa no está en la demarcación de ninguna oficina forestal.
permits-none-state-forest = Según { $source }, ningún bosque del estado está en el área del mapa.
permits-forest-units-reach = Una oficina forestal es competente para el bosque de su demarcación. Su demarcación puede abarcar también ciudades y campos.
permits-state-forest-office = competente: { $office }
permits-hint-forest-office = { $units ->
        [none] Oficina forestal competente para el bosque del área del mapa: { $office } ({ $administration }). Pregunta allí pronto si planeas un evento en el bosque.
       *[other] Oficina forestal competente para el bosque del área del mapa: { $office } ({ $administration }), { $units }. Pregunta allí pronto si planeas un evento en el bosque.
    }
permits-hint-forest-owner = { $office ->
        [known] Un evento en el bosque necesita el consentimiento de los propietarios del bosque. La oficina forestal suele poder decir quién es el propietario. En varios estados federados, un evento organizado en el bosque debe además comunicarse a la autoridad forestal o ser autorizado por ella.
       *[unknown] Un evento en el bosque necesita el consentimiento de los propietarios del bosque. En varios estados federados, un evento organizado en el bosque debe además comunicarse a la autoridad forestal o ser autorizado por ella. Estas notas no nombran la oficina forestal competente aquí; la administración forestal del estado o la autoridad forestal inferior pueden decírtelo.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Un evento en el bosque suele necesitar el consentimiento del propietario del bosque. El distrito forestal (Nadleśnictwo) citado arriba suele poder decir quién se ocupa del bosque. El bosque que no pertenece a los Bosques del Estado, por ejemplo un bosque privado, no figura en los datos de estas notas.
       *[unknown] Un evento en el bosque suele necesitar el consentimiento del propietario del bosque. Estas notas no nombran ningún distrito forestal de los Bosques del Estado para el área del mapa; los datos solo abarcan los Bosques del Estado, así que el bosque puede pertenecer a otra persona, por ejemplo a un propietario privado. Quién es el propietario hay que averiguarlo aparte.
    }
permits-section-water-protection = Zonas de protección de aguas ({ $source })
permits-section-forest-functions = Funciones del bosque y bosque protegido ({ $source })
permits-none-water-protection = Según { $source }, ninguna zona de protección de aguas está en el área del mapa.
permits-none-forest-functions = Según { $source }, ningún bosque de recreo, bosque protegido, bosque natural ni área forestal cerrada está en el área del mapa.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Bosque de recreo (Erholungswald)
        [protected] Bosque protegido (Bannwald o Schutzwald)
        [natural] Bosque natural o reserva estricta (Naturwald, Totalreservat)
       *[closed] Área forestal cerrada
    }
permits-hint-water-protection = { $inner ->
        [yes] La { $zone } de la zona de protección de aguas "{ $name }" está en el área del mapa. En las zonas I y II normalmente no se permiten eventos. Pregunta a la autoridad inferior de aguas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] La { $zone } de la zona de protección de aguas "{ $name }" está en el área del mapa. Los eventos allí suelen ser posibles; la autoridad inferior de aguas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) puede decir qué permite el reglamento.
       *[unknown] La zona de protección de aguas "{ $name }" está en el área del mapa. Los datos no dicen de qué zona se trata. En las zonas I y II normalmente no se permiten eventos; pregunta a la autoridad inferior de aguas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Hay bosque de recreo (Erholungswald) en el área del mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Para eventos allí, pregunta a la autoridad forestal.
        [protected] Hay bosque protegido (Bannwald o Schutzwald) en el área del mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pregunta a la autoridad forestal antes de planear un evento allí.
        [natural] Hay bosque natural o una reserva estricta en el área del mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Salirse de los caminos suele estar prohibido allí; no coloques controles en él sin preguntar a la autoridad forestal.
       *[closed] Hay un área forestal de acceso prohibido en el área del mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Déjala fuera del recorrido.
    }
permits-section-roads = Carreteras clasificadas según OpenStreetMap
permits-none-roads = Según OpenStreetMap, ninguna Autobahn ni Bundes-, Landes-, Staats- o Kreisstraße atraviesa el área del mapa.
permits-road-line = { $class ->
        [autobahn] Autobahn (autopista)
        [bundesstrasse] Bundesstraße (carretera federal)
        [landesstrasse] Landesstraße o Staatsstraße (carretera estatal)
       *[kreisstrasse] Kreisstraße (carretera de distrito)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] La Autobahn { $refs } atraviesa el área del mapa. Un recorrido solo puede cruzarla por puentes o pasos inferiores.
        [bundesstrasse] La Bundesstraße { $refs } atraviesa el área del mapa. Un corte o un cruce con controladores lo ordena la autoridad de tráfico (Straßenverkehrsbehörde); la carretera suele mantenerla la Federación a través de la administración de carreteras del estado, en ciudades grandes a menudo la ciudad.
        [landesstrasse] La Landes- o Staatsstraße { $refs } atraviesa el área del mapa. Un corte o un cruce con controladores lo ordena la autoridad de tráfico (Straßenverkehrsbehörde); la carretera suele mantenerla el estado, en ciudades grandes a menudo la ciudad.
       *[kreisstrasse] La Kreisstraße { $refs } atraviesa el área del mapa. Un corte o un cruce con controladores lo ordena la autoridad de tráfico (Straßenverkehrsbehörde); la carretera suele mantenerla el Kreis, en ciudades grandes a menudo la ciudad.
    }
permits-hint-state-forest = { $office ->
        [none] Según los datos abiertos del estado, { $share } del área del mapa es bosque del estado del propietario "{ $owner }".
       *[other] Según los datos abiertos del estado, { $share } del área del mapa es bosque del estado del propietario "{ $owner }". Competente: { $office }.
    }
