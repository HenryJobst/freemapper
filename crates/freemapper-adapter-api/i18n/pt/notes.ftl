# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = aviso: { $text }

warning-other-zone = a área do mapa fica na zona UTM { $zone } (EPSG:{ $epsg }); o freemapper volta a obter tudo nessa zona
warning-other-state = a área do mapa fica agora nos dados de outro estado federado; o freemapper volta a obter tudo
warning-beyond-coverage = parte da área do mapa fica fora dos estados federados suportados. Fica no estrangeiro, no mar ou num estado federado que o freemapper ainda não suporta, como a Turíngia. Os estados federados não têm dados aí, por isso a ortofoto pode ficar branca e o ALKIS não dá edifícios nem uso do solo
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: a escala 1:{ $scale } não é nenhuma das escalas dos mapas de { $kind } ({ $allowed }); o freemapper-print vai recusar o mapa
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorada a entrada { $key } da camada { $german ->
       *[any] { $what }
    }: { $reason ->
        [name] não é um nome de ficheiro simples
       *[unknown] não é uma camada que o freemapper conheça
    }

## Disk space

warning-disk-space-unreadable = não foi possível ler o espaço livre em disco de { $path } ({ $error }), por isso o freemapper não o verifica

## Tiles

warning-tile-changed = o mosaico { $tile } de { $product } mudou na origem ({ $old } → { $new })
warning-bdom-tile-unavailable = modelo de superfície (bDOM) do mosaico { $tile } indisponível ({ $error }); sem modelo nDSM aí
warning-bdom-unavailable = modelo de superfície (bDOM) indisponível ({ $error }); sem modelo nDSM para { $tiles }
progress-ndsm-from-bdom = nDSM do mosaico { $tile } a partir do modelo de superfície (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] modelo de terreno
        [surface] modelo de superfície
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = a descarregar o { $model }
progress-model-read = a ler o { $model }
warning-model-unavailable = { $model } indisponível ({ $error }); sem modelo nDSM
warning-model-unavailable-kept = { $model } indisponível ({ $error }); mantido o modelo de superfície obtido em { $fetched }
warning-model-tile-unavailable = { $model } do mosaico { $tile } indisponível ({ $error }); sem modelo nDSM aí
warning-archive-passed-over = { $model }: não foi possível ler um arquivo ({ $error }); foi ignorado, e os mosaicos da área do mapa vêm dos outros arquivos
warning-archive-directory-kept = { $model }: não foi possível ler um arquivo ({ $error }); usa-se o seu diretório de uma execução anterior, por isso não se nota um mosaico que o LVGL tenha entretanto substituído
warning-model-tiles-missing = o { $model } não tem { $count ->
        [one] o mosaico
        [many] os mosaicos
       *[other] os mosaicos
    } { $names } { $state ->
        [the-map-area] na área do mapa
        [hamburg] em Hamburgo
        [hesse] em Hesse
        [north-rhine-westphalia] na Renânia do Norte-Vestefália
        [schleswig-holstein] em Schleswig-Holstein
        [mecklenburg-vorpommern] no Mecklemburgo-Pomerânia Ocidental
        [lower-saxony] na Baixa Saxónia
        [bavaria] na Baviera
       *[other] em { $state }
    }; os modelos ficam aí vazios
warning-no-surface-stored = nenhum modelo de superfície guardado; executa `freemapper fetch` com rede para obter o modelo nDSM
warning-surface-older-than-orthophoto = o modelo de superfície (bDOM { $surface }) é mais antigo do que a ortofoto ({ $photo }); podem faltar edifícios novos no nDSM
progress-surface-kept = o freemapper mantém o modelo de superfície obtido em { $fetched } para o modelo nDSM
progress-surface-tiles-kept = o freemapper mantém na cache o modelo de superfície { $count ->
        [one] do mosaico
        [many] dos mosaicos
       *[other] dos mosaicos
    } { $names } para o modelo nDSM
progress-orthophoto-unchanged = ortofoto inalterada
progress-orthophoto-unchanged-dated = ortofoto inalterada ({ $label } { $date })
progress-orthophoto-download = a descarregar a ortofoto de { $provider } { $service }
progress-orthophoto-in-leaf = a ortofoto foi captada em { $date }, com as árvores com folhas; as copas escondem o que está por baixo
progress-orthophoto-bare = a ortofoto foi captada em { $date }, com as árvores sem folhas
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; mantida a ortofoto guardada e a sua data de voo { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): sem data de voo, { $reason }
word-stac-no-image = o STAC não lista nenhuma imagem no centro da área do mapa
word-stac-no-answer = a pesquisa STAC não deu uma resposta utilizável ({ $error })
warning-downloadcenter-no-file = o Downloadcenter não tem ficheiro para { $places }
warning-downloadcenter-no-file-empty = o Downloadcenter não tem ficheiro para { $places }; { $count ->
        [one] o mosaico
        [many] os mosaicos
       *[other] os mosaicos
    } { $names } ficaram vazios
progress-k-five-unchanged = modelo K5 inalterado
progress-k-five-download = a descarregar o mapa K5 (1:5000) do WMS do Geoportal Berlin
warning-k-five-unavailable = o modelo de mapa K5 está indisponível ({ $detail }); ignorado

## The state border

progress-beyond-border = a área do mapa ultrapassa a fronteira do estado federado; para lá dela os modelos ficam vazios onde terminam a ortofoto e os modelos de altura de { $provider }. O ALKIS e o OSM vêm dos dois estados federados
progress-beyond-border-no-data = a área do mapa ultrapassa a fronteira do estado federado; não há dados de { $provider } para { $tiles }, por isso os modelos ficam aí vazios

## OSM

warning-osm-extract-missing-kept = nenhum extrato OSM na cache; mantêm-se os candidatos OSM guardados de { $date }
progress-osm-extract-read = a ler o extrato OSM { $region } de { $date }
progress-osm-candidates = { $count } objetos OSM candidatos

## Permit notes

progress-permits-fetch = a obter dados de { $source } para as notas de licenciamento
progress-permits-osm = a ler o extrato OSM { $region } de { $date } para as notas de licenciamento
progress-permits-stored = a ler os dados guardados das notas de licenciamento
warning-permits-source-unreachable = não foi possível ler { $source }, por isso as notas de licenciamento omitem-na: { $error }
warning-permits-osm-no-extract = nenhum extrato OSM está na cache para esta área do mapa, por isso as notas de licenciamento omitem o OSM; `freemapper fetch` descarrega-o

## ALKIS

warning-alkis-coverage-unknown = cobertura dos edifícios { $cadastre } desconhecida; executa `freemapper fetch` com rede para a registar
progress-alkis-buildings-download = a descarregar edifícios { $cadastre }
progress-alkis-buildings = { $count } edifícios { $cadastre }
warning-alkis-no-buildings = nenhum edifício { $cadastre } encontrado na área do mapa
progress-alkis-skipped = ignorados { $count } objetos { $cadastre }: { $list }
warning-alkis-buildings-changed = os edifícios { $cadastre } mudaram na origem ({ $changes })
progress-alkis-no-land-use-stored = nenhum uso do solo { $cadastre } guardado; executa `freemapper fetch` com rede para obter candidatos de zonas proibidas
progress-alkis-land-use-download = a descarregar o uso do solo { $cadastre }
progress-alkis-land-use-parts = { $count } partes de uso do solo { $cadastre }
warning-alkis-land-use-kept = uso do solo { $cadastre } indisponível ({ $error }); mantém-se o uso do solo guardado
warning-alkis-land-use-removed = uso do solo { $cadastre } indisponível ({ $error }); o uso do solo guardado já não cobre a área do mapa, por isso o freemapper removeu-o
warning-alkis-land-use-none = uso do solo { $cadastre } indisponível ({ $error }); sem candidatos de zonas proibidas
progress-bdot-filled = { $count } objetos do BDOT10k ({ $what }) acrescentados onde o EGiB não dá nada ({ $powiat })
warning-bdot-failed = O BDOT10k não conseguiu preencher a lacuna do EGiB ({ $what }): { $error }
warning-bdot-instead = O EGiB não está disponível ({ $error }); usa-se o BDOT10k em seu lugar ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = nenhuma estrutura ALKIS de Berlim guardada; executa `freemapper fetch` com rede para escadas, muros, árvores e coberturas
progress-berlin-removing = a remover as camadas de Berlim guardadas: a área do mapa já não fica perto de Berlim
progress-berlin-download = a descarregar estruturas ALKIS de Berlim, cadastro de árvores, levantamento de ruas e camadas de pontos
progress-berlin-features = { $count } objetos { $file } de Berlim
warning-berlin-layer-kept = { $file } de Berlim indisponível ({ $detail }); mantém-se a camada guardada
warning-berlin-layer-skipped = { $file } de Berlim indisponível ({ $detail }); camada ignorada
progress-feinkartierung-none-stored = nenhuma Feinkartierung de Hamburgo guardada; executa `freemapper fetch` com rede para grades, muros, escadas e mobiliário urbano
progress-feinkartierung-removing = a remover a Feinkartierung de Hamburgo guardada: a área do mapa já não fica em Hamburgo
progress-feinkartierung-download = a descarregar a Feinkartierung de Hamburgo (espaço das ruas e espaços verdes públicos)
progress-feinkartierung-features = { $count } objetos { $file } da Feinkartierung
warning-feinkartierung-layer-kept = Feinkartierung { $file } indisponível ({ $detail }); mantém-se a camada guardada
warning-feinkartierung-layer-skipped = Feinkartierung { $file } indisponível ({ $detail }); camada ignorada
error-extra-layer-unknown = não é uma camada de { $folder }
warning-berlin-buildings-missing = nenhum edifício ALKIS guardado para esta área em Berlim; executa `freemapper fetch` para os obter do Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = nenhuma estrutura ALKIS completa guardada; executa `freemapper fetch` com rede para muros, vedações, sebes e pontes
progress-nas-removing = a remover as camadas ALKIS completas guardadas: a área do mapa já não fica num estado federado com serviço NAS
warning-nas-folder-outside = camadas ALKIS completas ignoradas ({ $folder } sai da pasta do projeto)
progress-nas-download = a descarregar as estruturas ALKIS completas (NAS)
progress-nas-features = { $count } objetos ALKIS NAS { $file }
progress-nas-skipped = ignorados { $count } objetos ALKIS NAS { $file }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } indisponível ({ $detail }); mantém-se a camada guardada
warning-nas-layer-skipped = ALKIS NAS { $file } indisponível ({ $detail }); camada ignorada

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nenhum objeto do ATKIS Basis-DLM guardado; executa `freemapper fetch` com rede para nascentes, grutas, torres, monumentos e fontes
progress-dlm-removing = a remover as camadas guardadas do ATKIS Basis-DLM: a área do mapa já não fica num estado federado que o disponibilize
warning-dlm-folder-outside = camadas do ATKIS Basis-DLM ignoradas ({ $folder } sai da pasta do projeto)
progress-dlm-download = a descarregar os objetos do ATKIS Basis-DLM
progress-dlm-features = { $count } objetos ATKIS Basis-DLM { $file }
progress-dlm-skipped = ignorados { $count } objetos ATKIS Basis-DLM { $file }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } indisponível ({ $detail }); mantém-se a camada guardada
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } indisponível ({ $detail }); camada ignorada

## Tree cadastres

progress-trees-none-stored = nenhum cadastro municipal de árvores guardado; executa `freemapper fetch` com rede para as árvores individuais
progress-trees-removing = a remover os cadastros de árvores guardados: a área do mapa já não fica numa cidade que tenha um
warning-trees-folder-outside = cadastros municipais de árvores ignorados ({ $folder } sai da pasta do projeto)
progress-trees-download = a descarregar os cadastros municipais de árvores
progress-trees-count = { $count } árvores de { $id }
progress-trees-skipped = ignoradas { $count } árvores de { $id }: { $list }
warning-trees-kept = cadastro de árvores de { $id } indisponível ({ $detail }); mantém-se o guardado
warning-trees-skipped = cadastro de árvores de { $id } indisponível ({ $detail }); árvores ignoradas

## Deriving the templates

progress-writing = a escrever { $path }
progress-contours-steep = curvas de nível de { $interval } em { $interval } m: o terreno é tão íngreme que curvas de nível de { $default } em { $default } m se sobreporiam; uma equidistância de { $default } m no projeto mantém-nas separadas (`--contour-interval { $default }`)
warning-osm-only-no-terrain = ainda não há dados de terreno para esta região: o mapa é feito só a partir do OpenStreetMap, sem curvas de nível, modelos nem edifícios
progress-no-ndsm-recorded = nenhum nDSM registado; executa `freemapper fetch` para obter o modelo nDSM
warning-ndsm-tile-invalid = mosaico nDSM { $tile } inválido em freemapper.toml; ignorado
warning-ndsm-not-cached = nDSM { $tile } não está na cache; executa `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } ilegível ({ $error }); apaga { $path } e executa `freemapper fetch`
warning-surface-unusable = modelo de superfície inutilizável ({ $error }); executa `freemapper fetch`

## Downloads and the cache

progress-downloading = a descarregar { $url }
progress-retry-wait = o serviço está ocupado; nova tentativa em { $seconds } s
progress-downloaded = descarregados { $megabytes } MB
progress-downloaded-of = descarregados { $megabytes } MB de { $total } MB
progress-tile-index-download = a descarregar o índice de mosaicos de { $product }
progress-tile-cached = { $tile } na cache
progress-surface-blocks = a ler { $count } dos { $total } blocos do mosaico { $tile } do modelo de superfície ({ $megabytes } MB)
progress-surface-whole = o mosaico { $tile } do modelo de superfície é descarregado por inteiro: { $reason }
warning-tile-cached-older = a usar o mosaico { $tile } de { $product } da cache, de { $published } (o mais recente, { $newer }, não foi descarregado)
warning-ndsm-no-cached = nenhum nDSM na cache para o mosaico { $tile }
warning-ndsm-tile-unavailable-cached = mosaico nDSM { $tile } indisponível ({ $reason }); a usar o ficheiro da cache
warning-ndsm-tile-unavailable-skipped = mosaico nDSM { $tile } indisponível ({ $reason }); ignorado
warning-file-check-failed = não foi possível verificar se { $file } ainda está atual ({ $reason }); a usar o ficheiro da cache e, sem perguntar, os outros ficheiros da cache deste servidor
warning-ldbv-no-buildings-file = nenhum ficheiro de edifícios para o mosaico { $tile } no servidor; o mapa pode ficar sem edifícios aí
progress-ndsm-not-published = nenhuma nuvem de pontos nDSM publicada para o mosaico { $tile }
progress-ndsm-cached = nDSM { $tile } na cache
progress-point-cloud-not-published = nenhuma nuvem de pontos laser publicada para o mosaico { $tile }
progress-point-cloud-cached = nuvem de pontos { $tile } na cache
warning-point-cloud-no-cached = nenhuma nuvem de pontos na cache para o mosaico { $tile }; sem candidatos de vegetação aí
warning-point-cloud-unavailable-cached = nuvem de pontos { $tile } indisponível ({ $reason }); a usar o ficheiro da cache
warning-point-cloud-unavailable-skipped = nuvem de pontos { $tile } indisponível ({ $reason }); sem candidatos de vegetação aí
warning-point-clouds-unavailable = as nuvens de pontos laser estão indisponíveis ({ $reason }); os candidatos de vegetação usam os mosaicos da cache, se os houver
progress-no-point-clouds = ainda não há nuvens de pontos laser para a vegetação aqui: o estado federado não publica nenhuma que o freemapper consiga ler, ou não no sistema de coordenadas do mapa
warning-point-clouds-not-fetched = a vegetação está escolhida, mas não há nuvens de pontos carregadas; executa `freemapper fetch` para as carregar
warning-point-cloud-unreadable = nuvem de pontos { $tile } ilegível ({ $error }); apaga { $path } e executa `freemapper fetch`
warning-osm-nrw-nearest = nenhum extrato do distrito da Renânia do Norte-Vestefália a menos de 100 m da área; a ler o mais próximo, { $district }
warning-osm-disk-full-cached = o disco está cheio, por isso o freemapper usa o extrato OSM da cache de { $date }. Liberta espaço e executa `freemapper fetch` de novo para obter um mais recente
warning-osm-geofabrik-unavailable = Geofabrik indisponível ({ $error }); a usar o extrato OSM da cache de { $date }
progress-osm-extract-cached = extrato OSM { $region } de { $date } na cache
progress-osm-extract-download = a descarregar { $url } (cerca de { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = a descarregar o mosaico bDOM { $tile } ({ $megabytes } MB, { $index } de { $total })
progress-sh-dgm-not-in-atom = o mosaico DGM1 { $tile } não está no diretório INSPIRE ATOM; a descarregar o seu XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = o mosaico DGM1 { $tile } do diretório INSPIRE ATOM não pode ser lido ({ $error }); a descarregar o seu XYZ ({ $megabytes } MB)
progress-sh-tile = mosaico { $tile }: { $text }
progress-sh-download-again = mosaico { $tile }: a descarga não pode ser lida ({ $message }); a descarregar de novo
warning-sh-portal-no-date = o Downloadportal não dá data para o mosaico { $tile } de { $model }
warning-sh-dgm-xyz-used = o mosaico DGM1 { $tile } não está no diretório INSPIRE ATOM; usado o seu XYZ do Downloadportal
warning-sh-dgm-xyz-used-unreadable = o mosaico DGM1 { $tile } do diretório INSPIRE ATOM não pode ser lido ({ $error }); usado o seu XYZ do Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = mosaico { $tile } de { $model }: { $count } células da área do mapa não têm altura
warning-centre-tile-missing = falta o mosaico { $tile } de { $model } no centro da área do mapa, por isso não há data
warning-centre-tile-unlisted = a lista de mosaicos não nomeia nenhum mosaico { $tile } de { $model } no centro da área do mapa, por isso não há data
warning-list-unreadable-kept = não foi possível ler a lista de mosaicos do { $model } ({ $error }); usada a guardada de uma execução anterior
warning-portal-list-unreadable-kept = não foi possível ler a lista de mosaicos do { $model } do Downloadportal ({ $error }); usada a guardada de uma execução anterior
warning-surface-much-older = o { $model } é de { $surface }, a ortofoto de { $flight }. O nDSM pode não ter edifícios e árvores mais recentes.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objetos da lista de mosaicos não têm chave de mosaico utilizável; ignorados
warning-rp-list-unreadable-kept = não foi possível ler a lista de mosaicos do LVermGeo RP ({ $error }); usada a guardada de uma execução anterior
warning-rp-metalink-not-kept = o Metalink do LVermGeo RP não foi guardado: { $error }
warning-rp-metalink-unreadable-kept = não foi possível ler o Metalink do LVermGeo RP de { $model } ({ $detail }); usado o guardado de uma execução anterior
warning-rp-metalink-unreadable = não foi possível ler o Metalink do LVermGeo RP de { $model } ({ $detail }), por isso os mosaicos para os quais a sua lista de mosaicos não dá ano de voo ficam sem alturas
warning-rp-tile-without-name = mosaico { $tile } de { $model }: este quadrado de 1 km não tem alturas no mapa. A lista de mosaicos do LVermGeo RP (Laserbefliegung) { $note } para ele, por isso o nome do ficheiro não é conhecido{ $either ->
        [metalink] , e o seu Metalink também não nomeia nenhum ficheiro para ele
       *[none] {""}
    }
warning-rp-centre-no-date = o mosaico { $tile } de { $model } não tem data: a lista de mosaicos do LVermGeo RP (Laserbefliegung) { $note } para ele
word-rp-list-neither = não dá nem ano de voo nem dias de voo
word-rp-list-no-days = não dá dias de voo
word-rp-list-no-year = não dá ano de voo
word-rp-list-days-unreadable = dá dias de voo { $days } que não podem ser lidos
word-rp-list-disagree = dá dias de voo { $days } e um ano de voo { $year } que não coincidem num só ano

## Hesse

warning-hvbg-undated = a lista de metadados não tem um Bildflugdatum legível para { $tiles }; os seus registos de projeto ficam sem data de voo
warning-hvbg-undated-more = a lista de metadados não tem um Bildflugdatum legível para { $tiles } e mais { $more }; os seus registos de projeto ficam sem data de voo
warning-hvbg-list-unreadable-older = não foi possível ler a nova lista de metadados do HVBG { $name } ({ $detail }); a usar a lista mais antiga { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: sem data, o STAC não lista nenhum mosaico no centro da área do mapa
warning-lgln-no-date = { $model }: sem data ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; o modelo guardado mantém a sua data registada
warning-mv-cells-empty = { $model }: { $count } células dentro dos mosaicos do Mecklemburgo-Pomerânia Ocidental não têm altura
warning-mv-centre-no-date = { $model }: sem data para o mosaico central { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } linhas da lista de mosaicos de { $model } não têm nome de mosaico utilizável; ignoradas
warning-nrw-list-rows-undated = { $count } linhas da lista de mosaicos de { $model } não têm data legível
warning-nrw-list-older = a lista de mosaicos do servidor para { $model } é de { $server }, mais antiga do que a guardada, de { $kept }; usada a guardada
warning-nrw-list-no-date = a lista de mosaicos não dá data para o mosaico { $tile } de { $model }

## After `init`

note-scale-outside-norm = nota: 1:5000 não consta da norma de sprint; o freemapper-print avisa disso e os símbolos ficam a 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } não é um id OSM como way/992423119
error-place-invalid-answer = resposta inválida da pesquisa de locais: { $detail }
error-place-not-found = { $id } não encontrado. O Nominatim lista apenas objetos OSM com etiquetas, por isso a linha exterior sem etiquetas de um multipolígono não aparece. Usa antes a relação do multipolígono (--osm relation/<id>), ou guarda o contorno como GeoJSON e usa --geojson
error-place-is-a-point = { $id } é um ponto, não uma área; escolhe o seu terreno (uma way ou relation)
error-place-no-such-part = { $id } tem { $count ->
        [one] { $count } parte
        [many] { $count } de partes
       *[other] { $count } partes
    }; não existe a parte { $part }
error-place-several-parts = { $id } é uma área de { $count } partes separadas ({ $sizes }). Uma área de mapa é um só polígono: escolhe uma parte com --part <n> (1 é a maior), ou guarda o contorno que queres como GeoJSON e usa --geojson
error-layer-not-a-world-file = { $path }: não é um world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = sem pré-visualização para { $layer }; camadas com pré-visualização: { $names }
error-layer-no-ndsm-published = não há nDSM publicado para os mosaicos desta área do mapa, por isso não há modelo ndsm
error-layer-not-made = { $layer } ainda não foi criado; executa { $step ->
        [fetch] freemapper fetch ou run (MCP: start_pipeline); só Berlim
        [export] freemapper export ou run (MCP: start_pipeline)
       *[derive] freemapper derive ou run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; o mapa OCAD fica sem fundo a partir dele
warning-map-preview-failed = o mapa foi escrito, mas falta a sua pré-visualização: { $error }
warning-training-no-tree-cover = o mapa de treino mantém inteiras as zonas abertas dos parques: falta a cobertura arbórea (data/tree-cover.tif); executa `freemapper derive` de novo
