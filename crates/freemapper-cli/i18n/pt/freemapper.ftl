# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } não é um idioma para o qual o freemapper tenha textos; disponíveis: { $available }

## init

cli-part-needs-osm = --part escolhe uma parte de uma área OSM e precisa de --osm; com --place, procura primeiro o id e usa --osm <id> --part <n>
cli-reading-file = a ler { $path }
cli-no-area-for-place = nenhuma área encontrada para "{ $query }". Se o local é uma área de várias partes separadas, escolhe uma com --osm <id> --part <n>, ou usa --geojson
cli-using-place = A usar { $name } ({ $id })
cli-created-project = Projeto criado { $path }

## Cache

cli-no-cache-dir = não há pasta de cache neste sistema; indica --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Obtido { $count } mosaico
        [many] Obtidos { $count } de mosaicos
       *[other] Obtidos { $count } mosaicos
    }
cli-wrote = Escrito { $path }
cli-kept-map = Mantido { $path } (pode conter o teu trabalho)
cli-map-lacks-templates = O mapa ainda não referencia os modelos de imagem. Adiciona-os no OOM (Templates > Open template…) ou substitui o mapa com --force, que guarda o antigo como .omap.bak.
cli-map-lacks-buildings = O mapa ainda não tem edifícios ALKIS. Para os adicionar, substitui-o com --force, que guarda o antigo como .omap.bak.
cli-map-lacks-canopies = Ao mapa faltam as coberturas e abrigos de automóveis do ALKIS (522) que agora são objetos do mapa. Para os adicionar, executa com --force, que guarda o mapa antigo como .omap.bak.
cli-map-light = O mapa base: { $joined } linhas unidas, { $left } escondidos ou demasiado pequenos deixados para as camadas de candidatos (nada foi movido)
cli-map-light-kept = As camadas de candidatos passam a conter também { $left } objetos ou troços que esta versão tira do mapa base (marcados freemapper:generalized). O mapa mantido ainda os tem, por isso aparecem duas vezes. Substitui o mapa com --force para o limpar (o antigo fica como .omap.bak).
cli-map-old-candidates = O mapa ainda refere templates/candidates.omap; esta execução preencheu-o com todos os candidatos. Os candidatos vêm agora também em sete camadas, cada uma num ficheiro (templates/candidates-*.omap). O mapa lista-as depois de o substituíres com --force (o antigo fica como .omap.bak), ou adiciona-as tu no OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF sem créditos das fontes: a ODbL e a dl-de/by-2-0 pedem que nomeies as fontes ao passares o mapa a outros; ver ATTRIBUTION.txt
cli-wrote-pdf = Escrito { $path } (o mapa sem candidatos)
cli-wrote-pdf-candidates = Escrito { $path } (a impressão de verificação: o mapa com todos os candidatos, fiel à posição; verifica-os no terreno)
cli-wrote-training = Escritos { $map } e { $pdf } (o mapa de treino: { $joined } unidos, { $shifted } deslocados, { $left } omitidos ou cortados)
cli-training-free-ends = Extremidades de linha deixadas livres: { $ways } onde um caminho encontrava outro caminho, { $others } noutras linhas
cli-training-apart-left = o mapa de treino ainda tem { $length } mm (no papel) onde os objetos ficam com pouca folga; verifica-o no OOM
cli-kept-training = Mantido o mapa de treino { $path }, para que uma execução não substitua um que foi impresso ou entregue; apaga-o ou indica --force para o renovar

## OCAD

cli-ocad-kept = Mantido { $path } (pode conter o teu trabalho; --force substitui-o e guarda o antigo como .ocd.bak)
cli-ocad-old-candidates = O mapa OCAD ainda refere { $path } como mapa de fundo; esta execução preencheu-o com todos os candidatos. Os candidatos vêm agora também em sete camadas, cada uma num ficheiro (-candidates-*.ocd). O mapa OCAD lista-as depois de o substituíres com --force (o antigo fica como .ocd.bak), ou adiciona-as tu no OCAD como mapas de fundo.
cli-ocad-wrote = { $count ->
        [one] Escrito { $path } ({ $count } objeto)
        [many] Escrito { $path } ({ $count } de objetos)
       *[other] Escrito { $path } ({ $count } objetos)
    }
cli-ocad-left-out = Deixados fora do mapa OCAD: { $skipped } objetos (sem símbolo para o OCAD, sem área, ou fora da área de desenho do OCAD de ±2 m) e { $dropped } partes de linha ou buracos degenerados.
cli-ocad-load-symbols = No OCAD, escolhe Symbol > Load Colors and Symbols From… e seleciona o conjunto de símbolos oficial{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , o conjunto escolar da IOF 2019
       *[other] {" "}ISSprOM 2019
    }; este substitui os símbolos provisórios do freemapper pelos de igual número.

## select

cli-select-running = Seleção da área para { $path }: prime "Concluído" na página ou Ctrl+C para parar.
cli-select-no-browser = Não foi possível abrir o navegador ({ $error }); abre o endereço acima.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; sem área, não pode ser uma área de mapa): { $place }
cli-find-hint = Cria o projeto com: freemapper init <dir> --osm { $id }
cli-find-no-area = Nenhum destes locais tem uma área que possa ser uma área de mapa. Desenha a área com: freemapper select <dir>
cli-find-nothing = nenhum local encontrado para "{ $query }"; tenta outro nome, ou desenha a área com: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = aviso: o adaptador em { $folder } foi deixado de fora: { $finding }
cli-adapter-note = aviso: o adaptador em { $folder } carrega, mas: { $finding }
cli-adapter-from-project = aviso: o adaptador { $adapter } da pasta do projeto está em uso; pode contactar estes anfitriões: { $hosts }
cli-adapter-overlaps = aviso: o adaptador { $adapter } cobre também território de { $others }; os seus objetos entram nos mapas aí
cli-adapter-dir-relative = aviso: FREEMAPPER_ADAPTER_DIR não é um caminho absoluto ({ $path }); é ignorado
cli-adapters-no-project = aviso: { $path } não é uma pasta; não foram lidos adaptadores de um projeto
cli-adapters-overlaps = partilha território com: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revisão { $revision }  verificado { $checked }  anfitriões: { $hosts }
cli-adapters-source-built-in = integrado
cli-adapters-source-user = da tua pasta
cli-adapters-source-project = do projeto
cli-adapters-never-checked = nunca
cli-adapters-user-dir = Os teus adaptadores são lidos de { $path }
cli-adapters-no-user-dir = Este sistema não tem pasta para os teus adaptadores; FREEMAPPER_ADAPTER_DIR indica uma

## adapters allow-host (ADR 0110)

cli-allow-host-added = o freemapper pode agora consultar também { $host } para { $name }. A entrada está em { $file }; `freemapper adapters disallow-host { $adapter } { $host }` retira-a. Uma página de área ou um assistente de IA em execução lê a entrada quando for iniciado de novo
cli-allow-host-already = { $host } já está permitido para { $name }
cli-allow-host-declared = o adaptador de { $name } já nomeia { $host }; nada a permitir
cli-allow-host-refused = { $host } não pode ser permitido: { $reason }
cli-allow-host-unknown-adapter = não existe o adaptador integrado { $adapter }; os adaptadores integrados são: { $adapters }
cli-allow-host-folder-adapter = { $adapter } é um adaptador da tua pasta de adaptadores; acrescenta antes o anfitrião a `[net] hosts` do seu manifesto
cli-allow-host-no-config = este sistema não tem pasta de configuração para guardar anfitriões permitidos
cli-allow-host-write = não foi possível escrever { $file }: { $detail }
cli-allow-host-unreadable = não foi possível ler { $file } ({ $detail }); corrige ou remove primeiro o ficheiro; nada foi alterado
cli-disallow-host-removed = o freemapper já não consulta { $host } para { $name }
cli-disallow-host-absent = { $host } não estava permitido para { $name }; nada a retirar
cli-allowed-hosts-unreadable = aviso: não foi possível ler { $file } ({ $detail }); nenhum anfitrião extra é permitido
cli-allowed-hosts-ignored = aviso: { $file }: a entrada { $host } para { $adapter } é ignorada: { $reason }
cli-allowed-hosts-no-adapter = o freemapper não tem nenhum adaptador integrado com esse nome
cli-allowed-host-redundant = aviso: { $host } é agora um dos anfitriões que o adaptador de { $name } nomeia; a tua entrada em { $file } foi removida
cli-adapters-allowed = permitido por ti: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adaptador { $adapter } ({ $name }), revisão { $revision }, anfitriões: { $hosts }
cli-check-error = erro: { $finding }
cli-check-warning = aviso: { $finding }
cli-check-passed = { $folder } passa na verificação
cli-check-passed-warnings = { $folder } passa na verificação, com { $count ->
        [one] { $count } aviso
        [many] { $count } de avisos
       *[other] { $count } avisos
    }
cli-check-failed = { $folder } falha na verificação
cli-check-no-folder = não existe a pasta { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } FALHOU: { $detail }
cli-check-live-not-asked = { $capability } não consultado: { $detail }
cli-check-live-box-outside = a caixa não fica dentro do limite do adaptador { $adapter }; uma verificação em direto consulta os serviços para um local que o adaptador cobre
cli-check-live-cannot = a verificação em direto não pode ser executada: { $detail }
cli-check-detail-not-declared = o adaptador não declara nenhum
cli-check-detail-stopped = uma parte anterior falhou
cli-check-detail-see-warnings = ver os avisos
cli-check-detail-outside-dataset = a caixa fica fora do conjunto de dados
cli-check-detail-objects = { $count ->
        [one] { $count } objeto
        [many] { $count } de objetos
       *[other] { $count } objetos
    }
cli-check-detail-layers = { $layers } camadas, { $count } objetos
cli-check-detail-trees = { $count ->
        [one] { $count } árvore
        [many] { $count } de árvores
       *[other] { $count } árvores
    }
cli-check-detail-extract = { $label }, dados de { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = modelo de terreno
cli-check-raster-surface = modelo de superfície
cli-check-raster-tiles = { $product }: { $count ->
        [one] { $count } mosaico
        [many] { $count } de mosaicos
       *[other] { $count } mosaicos
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } e { $last }

## Folder lock

cli-lock-held = outra execução do freemapper está a usar esta pasta (desde { $time }, iniciada por { $who }). Espera por ela ou para-a primeiro.
cli-lock-who-cli = o terminal
cli-lock-who-mcp = um agente de IA (MCP)
cli-lock-who-page = a página de seleção
cli-lock-who-unknown = outro programa
cli-lock-failed = não foi possível bloquear a pasta do projeto para esta execução: { $error }
cli-lock-stuck = há algo que não é um ficheiro de bloqueio no caminho e não pôde ser removido

## Run routes of the selection page

cli-run-bad-request = As opções da execução não são válidas: { $error }
cli-run-no-project = Ainda não há nenhum projeto guardado nesta pasta. Guarda primeiro a área.
cli-run-already-running = Já está em curso uma execução ({ $job }) para este projeto. Espera que termine ou cancela-a primeiro.
cli-run-disk-stops = Não há espaço livre suficiente em disco para a descarga: { $messages }
cli-run-busy-elsewhere = Outro projeto deste programa está agora a executar um pipeline. Os projetos partilham a cache de descargas, por isso só corre uma execução de cada vez. Tenta de novo quando terminar.
cli-run-nothing-to-cancel = Nenhuma execução em curso, por isso não há nada a cancelar.
cli-run-unknown-job = Não existe a execução { $job } para este projeto.
cli-status-too-fast = Os pedidos de estado são demasiado frequentes. Pergunta no máximo cinco vezes por segundo.
cli-open-folder-failed = Não foi possível abrir a pasta ({ $error }).
cli-open-folder-no-opener = Este computador não tem programa para abrir pastas ({ $program }). Abre a pasta manualmente.
cli-open-folder-missing = A pasta do projeto ainda não existe. Guarda primeiro a área.
cli-open-folder-refused = O nome da pasta não é seguro para passar ao gestor de ficheiros.
cli-save-running = Há uma execução em curso para este projeto. Guarda de novo quando terminar.
cli-save-unknown-kind = Tipo de mapa desconhecido "{ $kind }". Usa schoolyard, sprint ou forest.
cli-save-scale-too-large = A escala { $scale } é demasiado grande.
cli-save-bad-content = As definições de conteúdo não são válidas. Nada foi guardado.
cli-map-image-none = Cria primeiro o mapa.
cli-map-image-gone = Esta pré-visualização já não está disponível.
cli-map-image-running = Há uma execução em curso para este projeto. A pré-visualização pode ser calculada quando terminar.
cli-map-image-stale = O projeto guardado já não corresponde ao mapa gerado, por exemplo porque a área, o tipo de mapa ou a escala mudaram. Inicia primeiro uma execução que substitua o mapa ("Substituir o mapa existente" na página, ou freemapper run --force).
cli-map-image-superseded = Entretanto foi pedida uma pré-visualização mais recente, por isso esta não foi calculada.
cli-map-image-bad-content = As definições de conteúdo não são válidas.
cli-map-image-bad-request = O pedido de pré-visualização não é válido. Envia um objeto JSON, com as definições de conteúdo, se as houver.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – seleção da área
page-heading = Escolhe a área do mapa
page-update-available = O freemapper { $version } está disponível. Atualiza com
page-update-notes = Notas de versão
page-folder = Pasta do projeto
page-pick = Procurar um local
page-query-placeholder = p. ex. Clara-Zetkin-Schule Strausberg
page-search-button = Procurar
page-searching = A procurar …
page-no-results = Nada encontrado.
page-point-only = apenas um ponto, não uma área
page-draw = Desenha tu
page-rectangle = Retângulo
page-polygon = Polígono
page-finish-polygon = Fechar polígono
page-clear = Limpar
page-hint-rectangle = Clica em dois cantos opostos no mapa.
page-hint-polygon = Clica nos cantos um a seguir ao outro. Para fechar, clica no primeiro canto, faz duplo clique ou clica em "Fechar polígono". Esc cancela, Backspace remove o último canto.
page-hint-idle = Escolhe um local na pesquisa ou desenha tu a área.
page-summary = Área
page-source-osm = Do OpenStreetMap: { $name }
page-source-drawn = Desenhada à mão
page-size = Tamanho com margem: { $width } × { $height } m
page-tiles-one = { $n } mosaico: { $names }
page-tiles-other = { $n } mosaicos: { $names }
page-download = Descarga: cerca de { $mb } MB
page-download-none = Descarga: tudo já está na cache
page-download-details = O que é descarregado
page-cached = na cache
page-disk = Espaço em disco: { $volumes }
page-disk-volume = { $free } livres, cerca de { $needed } necessários
page-cache-volume = cache
page-project-volume = pasta do projeto
page-no-room = não cabe no disco
page-no-room-skip = Nem tudo cabe no disco (o freemapper mantém { $reserve } livres). A obtenção deixa de fora as partes assinaladas. Liberta primeiro espaço, se precisares delas:
page-no-room-stop = Espaço em disco insuficiente (o freemapper mantém { $reserve } livres). Sem as partes assinaladas não há mapa, por isso a obtenção para antes de descarregar seja o que for. Liberta primeiro espaço:
page-disk-unknown = o freemapper não consegue ler o espaço livre em disco ({ $error }). Obtém os dados sem essa verificação.
page-too-large = A área é demasiado grande: no máximo { $max } m por lado (com margem). Escolhe uma mais pequena.
page-outside = A área fica fora dos estados federados suportados ({ $states }); o freemapper ainda não tem fontes de dados para outras regiões.
page-outside-unknown = A área fica fora das regiões que o freemapper suporta; ainda não tem fontes de dados aí.
page-adapters-failed = Não é possível listar as fotografias aéreas ({ $detail }). O mapa mostra só o OpenStreetMap; tudo o resto funciona.
page-settings = Mapa
page-kind = Tipo de mapa
page-schoolyard = Mapa de recreio escolar (1:1000)
page-sprint = Mapa de sprint (1:4000)
page-forest = Mapa de floresta (1:15000)
page-scale = Escala 1:
page-map-title = Título
page-contour-interval = Equidistância das curvas de nível
page-contour-auto = automática
page-settings-fixed = O projeto já existe; o tipo de mapa, a escala e o título estão em freemapper.toml. A equidistância das curvas de nível ainda pode ser alterada.
page-save = Guardar
page-saving = A guardar …
page-saved = Guardado em freemapper.toml.
page-zone-changed = Guardado. A área fica agora noutra zona UTM; o freemapper volta a obter os seus dados na próxima execução.
page-provider-changed = Guardado. Outro fornecedor de dados serve a altimetria e as imagens aéreas da nova área; o freemapper volta a obter os seus dados na próxima execução.
page-save-stale = freemapper.toml foi alterado noutra janela ou por outro programa desde que esta página o leu. Nada foi guardado; a página conhece agora o estado guardado. Guarda de novo para o substituir pela área mostrada aqui.
page-has-map = O mapa existente (.omap) fica como está; o freemapper cria novos modelos e candidatos para a nova área. A opção "Substituir o mapa existente" abaixo (ou "freemapper run --force") refaz também o mapa e guarda o antigo como .omap.bak.
page-done = Concluído
page-source = Código-fonte (AGPL-3.0-or-later)
page-closed = Concluído. Podes fechar esta janela.
page-next = A seguir: freemapper run "{ $dir }"
page-made-files = Criados: { $files }. Pasta: { $dir }
page-back-to-agent = Diz ao teu assistente de IA que a área está guardada.
page-nothing-saved = Nada foi guardado.
page-failed = Erro: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Fotografia aérea { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = O que vai para o mapa
page-content-help = O perfil decide o que vai diretamente para o mapa e o que fica como candidato a verificar. A escolha para um grupo sobrepõe-se ao perfil. É guardada com a área.
page-content-profile-legend = Perfil
page-content-profile-safe = Seguro
page-content-profile-safe-hint = Cada objeto fica onde a construção o põe: os edifícios e as áreas do cadastro vão para o mapa; muros, vedações, árvores e muito do OpenStreetMap (lagos, caminhos sem superfície conhecida) ficam como candidatos.
page-content-profile-balanced = Equilibrado
page-content-profile-balanced-hint = Os objetos do cadastro, dos registos e do levantamento de ruas vão para o mapa, com as curvas de nível e as áreas proibidas ao longo das linhas ferroviárias. Muros, vedações, sebes, taludes, escadas, água, árvores, monumentos, passagens, caminhos e linhas ferroviárias vêm também do OpenStreetMap. O mobiliário urbano, as outras áreas proibidas e as formas do modelo de terreno ficam como candidatos.
page-content-profile-full = Completo
page-content-profile-full-hint = Tudo vai para o mapa, como na impressão de verificação; só ficam de fora os candidatos escondidos.
page-content-groups-legend = Por grupo
page-content-groups-help = "Como no perfil" segue o perfil. O que isso significa para um grupo depende da origem de cada objeto.
page-content-place-profile = Como no perfil
page-content-place-map = Mapa
page-content-place-candidates = Candidato
page-content-place-off = Desligado
page-content-group-walls = Muros
page-content-group-fences = Vedações
page-content-group-hedges = Sebes
page-content-group-slopes = Taludes e rochedos
page-content-group-stairs = Escadas
page-content-group-trees = Árvores e grupos de árvores
page-content-group-water-features = Água, fontes e nascentes
page-content-group-monuments = Torres, monumentos e linhas elétricas
page-content-group-furniture = Mobiliário urbano e candeeiros
page-content-group-passages = Passagens
page-content-group-contours = Curvas de nível
page-content-group-landforms = Colinas, depressões, taludes e rochedos do modelo de terreno
page-content-group-vegetation = Vegetação a partir de nuvens de pontos laser (Brandemburgo, Berlim, Renânia do Norte-Vestefália, Renânia-Palatinado, Sarre e Baviera; desligada até a escolheres; descarrega cerca de 40 a 600 MB por km²)
page-content-group-out-of-bounds = Áreas proibidas
page-content-group-paths = Caminhos e linhas ferroviárias
page-content-group-streets = Ruas e praças
page-run-heading = Criar o mapa
page-preview-heading = Pré-visualização do mapa
page-preview-stored = Como foi gerado da última vez.
page-preview-live = Com as definições desta página, ainda não guardadas.
page-preview-live-saved = Com as definições guardadas, ainda não geradas numa execução.
page-preview-computing = A calcular …
page-preview-none = Ainda não foi gerado nenhum mapa.
page-preview-later = Este mapa ainda não tem pré-visualização. Vem com a próxima execução que escrever o mapa.
page-preview-stale = A área, o tipo de mapa ou a escala mudaram desde que o mapa foi gerado. A imagem ainda mostra o mapa como foi gerado da última vez; uma nova pré-visualização precisa de uma execução com "Substituir o mapa existente".
page-preview-area-unsaved = A área desta página ainda não está guardada. A imagem mostra o mapa como foi gerado da última vez; a nova área precisa de uma execução com "Substituir o mapa existente".
page-preview-run = Como foi gerado da última vez. A página atualiza a imagem quando a execução terminar.
page-preview-open = Abrir em tamanho real
page-preview-alt = Pré-visualização dos objetos do mapa
page-opt-ocad = Escrever um projeto inicial para OCAD
page-help-ocad = Para quem continua a trabalhar no OCAD.
page-opt-pdf = PDF para impressão
page-help-pdf = Só os objetos do mapa, sem candidatos.
page-opt-pdf-candidates = Impressão de verificação com todos os candidatos
page-opt-training = Mapa de treino
page-help-training = O mapa e todos os candidatos, limpos para serem impressos tal como estão; com OCAD também como ficheiro OCAD.
page-views-heading = Vistas do mapa de treino
page-view-no-paths = Sem caminhos (mapa Ultimate)
page-view-brown-only = Só castanho (orientação por curvas de nível)
page-view-no-north-lines = Sem linhas norte
page-view-custom = Outra vista
page-help-views = Cada vista marcada é mais um ficheiro junto ao mapa de treino, `<name>-training-<view>.omap` e o seu PDF, com OCAD também um ficheiro OCAD. Outra vista é `no:505,506` (omitir símbolos) ou `only:101,102` (manter só estes); `+` combina vistas. O título, a escala e as fontes ficam sempre.
page-help-views-forest = Sem caminhos e só castanho servem apenas para mapas de floresta.
page-opt-scale-bar = Barra de escala nos PDF
page-help-scale-bar = Os mapas alemães normalmente não a têm.
page-opt-sources = Linhas de fontes nos PDF
page-sources-auto = automáticas
page-sources-full = completas
page-sources-credit-only = só freemapper
page-warn-credit-only = Os PDF passam então a nomear só o freemapper. As licenças dos dados (ODbL, dl-de/by-2-0) exigem atribuição quando passas o mapa a outros; então nomeias tu as fontes. ATTRIBUTION.txt lista-as todas.
page-need-pdf = A barra de escala e as linhas de fontes só se aplicam quando se cria um PDF.
page-opt-offline = Só cache, sem rede
page-opt-force = Substituir os mapas existentes
page-help-force = O mapa e, com "Mapa de treino" marcado, o mapa de treino. Os ficheiros antigos ficam como .omap.bak.
page-opt-permits = Notas de licenciamento
page-help-permits = Unidades administrativas, áreas protegidas e a quem perguntar, em permits.md. Uma indicação, não aconselhamento jurídico.
page-run-start = Criar mapa
page-run-again = Criar de novo
page-run-cancel = Cancelar
page-run-cancelling = A cancelar …
page-warnings-heading = Avisos
page-where-files = Os ficheiros estão na pasta do projeto:
page-open-folder = Abrir pasta
page-log-label = Registo da execução
page-force-title = Substituir os mapas existentes?
page-force-text = O mapa existente é substituído e, com "Mapa de treino" marcado, o mapa de treino também. As tuas edições não se perdem: os ficheiros antigos ficam na pasta do projeto como .omap.bak. Continuar?
page-force-cancel = Cancelar
page-force-confirm = Substituir
page-step-fetch = A obter dados
page-step-derive = A calcular os modelos
page-step-map = A escrever o mapa
page-step-ocad = A escrever o projeto OCAD
page-step-pdf = A escrever o PDF
page-step-pdf-candidates = A escrever a impressão de verificação
page-step-training = A escrever o mapa de treino
page-step-permits = A escrever as notas de licenciamento
page-step-line = Passo { $index } de { $total }: { $label }
page-starting = A execução está a começar …
page-file-kind-map = mapa
page-file-kind-template = modelo
page-file-kind-ocad = mapa OCAD
page-file-kind-ocad-candidates = candidatos OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = impressão de verificação
page-file-kind-training-map = mapa de treino
page-file-kind-training-pdf = PDF do mapa de treino
page-file-kind-training-ocad = mapa de treino OCAD
page-file-kind-training-view-map = vista do mapa de treino
page-file-kind-training-view-pdf = PDF da vista do mapa de treino
page-file-kind-training-view-ocad = vista do mapa de treino OCAD
page-file-kind-preview = pré-visualização do mapa
page-file-kind-permits = notas de licenciamento
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Concluído. Os ficheiros foram escritos.
page-result-failed = A execução falhou: { $error }
page-result-failed-kept = O que foi escrito fica na pasta do projeto; uma nova execução continua a partir daí.
page-result-cancelled = Cancelado. O que já foi descarregado fica na cache; uma nova execução continua a partir daí.
page-result-title-done = Mapa criado
page-result-title-failed = A execução falhou
page-result-title-cancelled = Execução cancelada
page-kept-map = O mapa existente ficou como estava. "Substituir o mapa existente" refá-lo e guarda o antigo como .omap.bak.
page-no-files = Não foram criados ficheiros.
page-opened-folder = A pasta está aberta.
page-open-failed = { $message } Pasta: { $dir }
page-why-no-area = Escolhe primeiro uma área.
page-why-checking = A pré-visualização está a ser calculada …
page-why-too-large = A área é demasiado grande.
page-why-outside = A área fica fora dos estados federados suportados.
page-why-disk = Espaço livre em disco insuficiente (ver acima).
page-why-running = Há uma execução em curso.
page-err-scale-bar-no-pdf = A barra de escala precisa de um PDF.
page-err-sources-no-pdf = As linhas de fontes precisam de um PDF.
page-save-failed = A área não está guardada; nenhuma execução foi iniciada.
page-poll-lost = Sem resposta do servidor. O freemapper continua a tentar.
page-run-unknown = O servidor já não conhece esta execução.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Dispensar

## update (spec 3.4 to 3.6)

update-source-build = Este freemapper foi compilado a partir do código-fonte. Atualiza-o com git pull e cargo build --release --locked.
update-current = O freemapper { $version } está atualizado.
update-available = O freemapper { $new } está disponível (tens o { $old }), publicado em { $released }.
update-details =
    Notas de versão: { $notes }
    Descarga: { $size } MB
update-decimal-separator = ,
update-ask = Instalar agora? [s/N]
update-not-installed = Nada foi instalado.
update-needs-yes = a entrada não é um terminal, por isso o freemapper não pode perguntar antes de instalar; executa freemapper update --yes para instalar sem perguntar
update-installing = a descarregar e verificar o freemapper { $version }
update-done = freemapper atualizado de { $old } para { $new }. Notas de versão: { $notes }
update-mcp-restart = Um servidor MCP em execução mantém a versão antiga até o assistente de IA o reiniciar.
update-rolled-back = Voltou-se ao freemapper { $version }. A versão que deixaste fica guardada como { $path }.
update-not-writable = não é possível escrever na pasta do programa { $path }, e o freemapper não pede direitos de administrador. Descompacta aí a nova versão manualmente, ou move o freemapper para uma pasta onde possas escrever.
update-too-old = o freemapper { $new } não pode ser instalado pela autoatualização de { $old }; instala manualmente { $needed } ou mais recente, ver { $notes }
update-busy = outra atualização ou reversão está a decorrer na pasta do programa (ficheiro de bloqueio { $path }); espera que termine e tenta de novo
update-bad-signature = não foi possível verificar a versão ({ $error }); nada foi instalado
update-check-bad-signature = não foi possível verificar a versão mais recente do freemapper ({ $error }); nada foi instalado
update-no-manifest = a versão mais recente não se consegue instalar sozinha (não tem manifesto de atualização); descarrega-a de { $page }
update-no-manifest-version = a versão { $version } não existe ou não se consegue instalar sozinha (não tem manifesto de atualização); ver { $page }
update-no-asset = a versão não tem programa para este sistema ({ $platform }); nada foi instalado
update-no-previous = não há versão anterior para onde voltar ({ $path } não existe)
update-failed = a atualização falhou: { $error }
update-mode-set = Modo de atualização definido como { $mode } em { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } sobrepõe-se a esta definição enquanto estiver definido.
update-mode-failed = não foi possível escrever { $path }: { $error }
update-notice = O freemapper { $new } está disponível (tens o { $old }). Atualiza com: freemapper update
update-applies-next-start = A nova versão aplica-se a partir do próximo arranque do freemapper.
update-install-failed = o freemapper { $new } não foi instalado: { $error }
update-no-config-dir = nenhuma pasta de configuração neste sistema; define FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Nota para a pessoa: ao arrancar, o servidor freemapper encontrou isto nas pastas de adaptadores. Passa-o à pessoa; ela corrige a pasta ou remove-a e depois reinicia o servidor.
    { $lines }
update-mcp-note = Nota para a pessoa: o freemapper { $new } está disponível (este é o { $old }). Pergunta-lhe antes de chamar install_update.
update-mcp-needs-confirm = nada foi instalado; pergunta primeiro à pessoa e chama install_update com confirm: true quando ela concordar
update-mcp-note-manual = Nota para a pessoa: o freemapper { $new } está disponível (este é o { $old }), mas install_update não o consegue instalar; instala manualmente { $needed } ou mais recente, ver { $notes }
update-mcp-note-bad-signature = Nota para a pessoa: não foi possível verificar a versão mais recente do freemapper ({ $error }); nada foi instalado. Diz-lho e não a instales de outra forma.
update-mcp-restart-now = Reinicia agora o servidor MCP do freemapper; ainda executa a versão antiga.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = erro
error-cancelled = cancelado
cli-cancelled-kept = cancelado; mantidos: { $files }
error-io = { $path }: { $message }
error-io-disk-full = o disco está cheio, por isso o freemapper não pôde escrever { $path }. Liberta espaço e tenta de novo

## Errors: project

error-project-already-exists = o projeto já existe: { $path }
error-project-map-exists = o mapa já existe (pode conter o teu trabalho); usa --force para o substituir: { $path }
error-project-invalid-scale = escala 1:{ $scale } inválida
error-project-invalid-contour-interval = equidistância das curvas de nível { $interval } m inválida em freemapper.toml; tem de ser maior do que 0
error-project-scale-not-allowed = a escala 1:{ $scale } não é permitida para mapas de { $kind }; usa { $allowed }
error-project-scale-not-allowed-enlarge = a escala 1:{ $scale } não é permitida para mapas de { $kind }; usa { $allowed }. Desenha a 1:4000, o freemapper-print amplia para 1:3000 ao imprimir
word-kind-schoolyard = recreio escolar
word-kind-sprint = sprint
word-kind-forest = floresta
error-project-toml = ficheiro de projeto inválido: { $message }
error-project-unknown-adapter = o ficheiro de projeto nomeia o adaptador "{ $adapter }", que este freemapper não tem; um freemapper mais recente pode trazê-lo (`freemapper update`), ou vem da tua pasta de adaptadores (`freemapper adapters list` mostra-o) ou, com --adapters-from-project na linha de comandos, da pasta `adapters` do projeto (só para um projeto em que confies)
error-project-not-fetched = ainda não foram obtidos mosaicos; executa primeiro `freemapper fetch`
error-project-ocd = exportação OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: a imagem e o world file não correspondem à área do mapa; executa `freemapper derive` de novo
error-project-tiles-outdated = a área do mapa precisa dos mosaicos { $tiles }, que não foram obtidos; executa `freemapper fetch`
error-project-buildings-not-fetched = nenhum edifício ALKIS guardado para esta área do mapa; executa primeiro `freemapper fetch` com rede
error-project-orthophoto-not-fetched = nenhuma ortofoto guardada para esta área do mapa; executa primeiro `freemapper fetch` com rede
error-project-terrain-not-fetched = nenhum modelo de terreno guardado para esta área do mapa; executa primeiro `freemapper fetch` com rede
error-project-candidates-outdated = os candidatos OSM guardados não cobrem a área do mapa; executa `freemapper fetch`
error-project-stored = { $path }: { $message }; executa `freemapper fetch` de novo
error-project-area-too-large = a área do mapa mais a margem é { $width } × { $height } m, mais do que { $limit } m por lado; reduz a área do mapa ou buffer_m em freemapper.toml
error-project-area-crosses-country = a área do mapa fica dos dois lados de uma fronteira entre países; os mapas que atravessam uma fronteira ainda não são suportados, por isso desenha a área dentro de um só país
error-project-permits-not-for-region = as notas de licenciamento ainda não estão disponíveis para esta região: consultam serviços alemães e nomeiam autoridades alemãs
error-project-outside-coverage = a área do mapa fica fora dos estados federados suportados ({ $states }); por enquanto o freemapper só tem fontes de dados para esses

## Errors: disk space

error-space-required = espaço em disco insuficiente para { $part }. Precisa de { $needed }, mas { $path } só tem { $left } livres além da reserva de { $reserve }. Liberta espaço e executa `freemapper fetch` de novo
error-space-skipped = o freemapper ignora { $part } por falta de espaço em disco. Precisa de { $needed }, mas { $path } só tem { $left } livres além da reserva de { $reserve }. Liberta espaço e executa `freemapper fetch` de novo
word-part-map-data = os dados de mapa ALKIS e OSM
word-part-terrain = o modelo de terreno
word-part-orthophoto = a ortofoto
word-part-surface = o modelo de superfície para o modelo nDSM
word-part-osm-update = a atualização do extrato OSM da cache
word-part-k-five = o modelo de mapa K5 de Berlim
word-part-berlin-layers = as estruturas ALKIS de Berlim, o cadastro de árvores, o levantamento de ruas e as camadas de pontos
word-part-state-layers = as estruturas ALKIS completas, os objetos do modelo de paisagem e os cadastros de árvores
word-part-point-cloud = as nuvens de pontos laser para os candidatos de vegetação

## Errors: downloads

error-fetch-http = a descarga de { $url } falhou: { $message }
error-fetch-http-temporary = a descarga de { $url } falhou: { $message }. O serviço provavelmente está sobrecarregado ou em manutenção; o freemapper tentou até { $attempts } vezes. Execute novamente dentro de alguns minutos; o que já foi descarregado é mantido.
error-fetch-corrupt-archive = a descarga de { $url } não é um arquivo de mosaicos completo: { $message }
error-fetch-corrupt-extract = a descarga de { $url } não é um extrato OSM completo: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] o mosaico { $tiles } não é coberto pelos mosaicos publicados; reduz a área do mapa ou buffer_m em freemapper.toml
        [many] os mosaicos { $tiles } não são cobertos pelos mosaicos publicados; reduz a área do mapa ou buffer_m em freemapper.toml
       *[other] os mosaicos { $tiles } não são cobertos pelos mosaicos publicados; reduz a área do mapa ou buffer_m em freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline e não está na cache: mosaico { $tiles } de { $product }; executa `freemapper fetch` com rede
        [many] offline e não estão na cache: mosaicos { $tiles } de { $product }; executa `freemapper fetch` com rede
       *[other] offline e não estão na cache: mosaicos { $tiles } de { $product }; executa `freemapper fetch` com rede
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = modelo de terreno (DGM1)
word-product-image-dsm = modelo de superfície (bDOM)
word-product-saxony-dtm = modelo de terreno (GeoSN DGM1)
word-product-saxony-dsm = modelo de superfície (GeoSN DOM1)
error-fetch-osm-extract-missing = nenhum extrato OSM na cache; executa `freemapper fetch` com rede
error-fetch-not-found = { $name } não encontrado no servidor
error-fetch-refused = { $url }: o servidor recusou o pedido (HTTP { $status })
error-fetch-refused-detail = { $url }: o servidor recusou o pedido (HTTP { $status }): { $detail }
error-fetch-too-large = a descarga de { $url } é maior do que { $limit } MB; interrompida
error-fetch-not-stored = nenhum dado guardado para esta área do mapa ({ $label }): { $path } não existe, não pode ser lido ou não a cobre; executa primeiro `freemapper fetch` com rede
error-fetch-range-ignored = { $url }: o servidor ignorou o intervalo de bytes { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: o freemapper não envia pedidos para este endereço: { $reason }
error-fetch-host-not-named-allow = { $target }: o freemapper não envia pedidos para este endereço: o adaptador de { $name } não nomeia o anfitrião { $host }{ $from }. Se o serviço se mudou para aí, permite o anfitrião com `freemapper adapters allow-host { $adapter } { $host }` e comunica-o em https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: o freemapper não envia pedidos para este endereço: o adaptador { $adapter } não nomeia o anfitrião { $host }{ $from }. Se o serviço se mudou para aí, acrescenta o anfitrião a `[net] hosts` do manifesto do adaptador
error-fetch-host-redirected-from = {" "}({ $url } redireciona para lá)
error-reason-not-a-host-name = não é um nome de anfitrião
error-reason-not-ascii = um nome fora do ASCII tem de ser escrito na sua forma xn--
error-reason-not-a-dns-name = não é um nome DNS
error-reason-no-domain = um nome deste computador ou sem domínio
error-reason-local-name = um nome da rede local
error-reason-ip-address = um endereço IP, não um nome DNS
error-reason-unexpected-character = carácter inesperado
error-reason-not-https = não é https
error-reason-user-info = informação de utilizador
error-reason-port = uma porta diferente de 443
error-reason-host-not-declared = o anfitrião não é um dos anfitriões declarados
error-reason-local-address = um endereço deste computador ou da rede local, não de um servidor na internet
error-reason-too-many-redirects = mais de { $count } redirecionamentos
error-reason-redirect-nowhere = um redirecionamento (HTTP { $status }) para lado nenhum
error-fetch-currency-csv = linha { $line } do CSV de atualidade: { $message }
error-alkis = dados ALKIS inválidos: { $message }
error-osm = dados OSM: { $message }

## Errors: templates and rasters

error-derive-zip = arquivo de mosaicos: { $message }
error-derive-image = imagem: { $message }
error-derive-no-tiles = nenhum mosaico de onde derivar
error-derive-area-too-large = área do mapa demasiado grande para um só modelo: { $width } × { $height } píxeis (no máximo { $limit } por lado); reduz a área do mapa ou buffer_m em freemapper.toml
error-raster-world-file = world file inválido: { $message }
error-raster-misaligned = os rasters não estão alinhados na mesma grelha de píxeis: { $message }
error-raster-pixel-size = os rasters têm tamanhos de píxel diferentes: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = GeoJSON inválido: { $message }
error-area-no-polygon = o GeoJSON não contém nenhum polígono
error-area-multi-polygon = MultiPolygon com { $parts } partes; divide a área do mapa em polígonos individuais
error-area-out-of-range = coordenada fora do intervalo: lat { $lat }, lon { $lon }
error-area-not-an-area = não é uma área de mapa utilizável: { $message }
error-georef-date = a data { $date } está fora da validade do World Magnetic Model
error-georef-map-range = o ponto E { $easting } N { $northing } fica fora do intervalo de coordenadas do mapa
error-omap-xml = XML do OOM inválido: { $message }
error-omap-missing-element = elemento <{ $name }> não encontrado
error-ocd-duplicate-symbol = o número de símbolo { $symbol } é usado duas vezes
error-ocd-unknown-symbol = um objeto refere o símbolo { $symbol }, que não está no ficheiro
error-ocd-wrong-type = um objeto do símbolo { $symbol } tem o tipo errado para o seu símbolo

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Utilização:
cli-heading-commands = Comandos:
cli-heading-options = Opções:
cli-heading-arguments = Argumentos:
cli-help-possible-values = valores possíveis
cli-help-default = predefinição
cli-arg-help = Mostrar a ajuda
cli-arg-version = Mostrar a versão
cli-help-help = Mostrar a ajuda de um comando
cli-arg-help-command = O comando cuja ajuda se mostra, por exemplo run
cli-parse-hint = Para mais informação, experimenta '--help'.
cli-double-click-start = O freemapper abre no navegador a página onde você escolhe a área do mapa. A pasta do projeto é { $path }. Feche esta janela para encerrar o programa.
cli-parse-unknown-argument = argumento inesperado { $arg }
cli-parse-missing-required = falta um argumento obrigatório: { $args }
cli-parse-invalid-value = valor { $value } inválido para { $arg }
cli-parse-invalid-value-choices = valor { $value } inválido para { $arg }; valores possíveis: { $values }
cli-parse-invalid-subcommand = comando desconhecido { $value }
cli-parse-missing-subcommand = é necessário um comando
cli-parse-too-many-values = valor inesperado { $value } para { $arg }
cli-parse-argument-conflict = { $arg } não pode ser usado com { $other }

help-freemapper = Cria projetos iniciais para o OOM a partir de geodados
help-freemapper-lang = Idioma dos textos do programa, `de` ou `en` (predefinição: FREEMAPPER_LANG, depois o idioma do sistema, depois inglês)
help-init = Criar uma pasta de projeto para uma área de mapa
help-init-dir = Pasta de projeto a criar
help-init-geojson = Ficheiro GeoJSON com o polígono da área do mapa (EPSG:4326)
help-init-osm = Objeto OSM da área do mapa, p. ex. way/992423119
help-init-place = Nome do local, p. ex. "Clara-Zetkin-Schule Strausberg"; o freemapper usa a melhor correspondência com uma área
help-init-part = Que parte de uma área OSM de várias partes separadas usar (1 é a maior)
help-init-kind = Tipo de mapa
help-arg-scale = Denominador da escala de um novo projeto: recreio escolar 1000 (predefinição); sprint 4000 (predefinição) ou 5000; floresta 15000 (predefinição), 10000 ou 7500. Outros valores são recusados, porque o freemapper-print os recusa
help-init-title = Título do mapa para a informação marginal (predefinição: o nome da pasta)
help-export = Escrever o projeto inicial (mapa OOM, modelos, atribuição) numa pasta de projeto
help-export-dir = Pasta do projeto
help-arg-force = Substituir um ficheiro de mapa existente e guardar o antigo como `<name>.omap.bak`
help-export-ocad = Escrever também um ponto de partida OCAD em `ocad/` (formato OCAD 12); um mapa OOM existente fica então, a menos que indiques --force
help-export-pdf = Escrever também `<name>.pdf`, o mapa para impressão (só os objetos do mapa, sem candidatos); um mapa OOM existente fica então, a menos que indiques --force
help-export-pdf-candidates = Escrever também `<name>-candidates.pdf`, a impressão de verificação: o mapa com todos os candidatos, fiel à posição; um mapa OOM existente fica então, a menos que indiques --force
help-export-training = Escrever também o mapa de treino `<name>-training.omap` e o seu PDF `<name>-training.pdf`: o mapa e todos os candidatos, limpos para serem impressos tal como estão; com --ocad também `ocad/<name>-training.ocd`. Um mapa OOM existente fica então, a menos que indiques --force, e --force substitui também um mapa de treino existente
help-export-view = Com --training: escrever também o mapa de treino sem o que uma vista omite, como `<name>-training-<view>.omap` e o seu PDF (com --ocad também um ficheiro OCAD). Vistas: `no-paths` (sem estradas, caminhos, escadas), `brown-only` (só símbolos castanhos e a moldura), `no-north-lines`, ou uma lista `no:<codes>` / `only:<codes>`; combina com `+`. Repete a opção para vários ficheiros
help-export-pdf-scale-bar = Desenhar a barra de escala nos PDF (desligada por predefinição, os mapas alemães normalmente não a têm; o número da escala fica no bloco de título)
help-export-pdf-sources = Linhas de fontes nos PDF: `auto` imprime a lista completa e só a encurta quando ocuparia mais de 5 linhas, `full` nunca encurta, `credit-only` imprime só a linha de créditos do freemapper (a ODbL e a dl-de/by-2-0 pedem então que nomeies as fontes ao passares o mapa a outros). ATTRIBUTION.txt lista sempre todas as fontes
help-fetch = Descarregar os dados da área do mapa (ortofoto, modelo de terreno, nDSM, edifícios ALKIS, OSM)
help-fetch-dir = Pasta do projeto
help-fetch-offline = Usar só a cache, sem acesso à rede
help-arg-cache-dir = Pasta de cache para os mosaicos descarregados (predefinição: a pasta de cache do sistema + `freemapper`)
help-derive = Calcular os modelos (ortofoto, relevo sombreado, declive, relevo local, nDSM), as curvas de nível e as formas de terreno a partir dos dados obtidos
help-derive-dir = Pasta do projeto
help-run = Obter, derivar e exportar de uma só vez
help-run-dir = Pasta do projeto
help-run-offline = Usar só a cache, sem acesso à rede
help-run-ocad = Escrever também um ponto de partida OCAD em `ocad/` (formato OCAD 12)
help-run-pdf = Escrever também `<name>.pdf`, o mapa para impressão (só os objetos do mapa, sem candidatos)
help-run-pdf-candidates = Escrever também `<name>-candidates.pdf`, a impressão de verificação: o mapa com todos os candidatos, fiel à posição
help-run-training = Escrever também o mapa de treino `<name>-training.omap` e o seu PDF `<name>-training.pdf`: o mapa e todos os candidatos, limpos para serem impressos tal como estão; com --ocad também `ocad/<name>-training.ocd`. Um mapa de treino existente fica, a menos que indiques --force
help-run-view = Com --training: escrever também o mapa de treino sem o que uma vista omite, como `<name>-training-<view>.omap` e o seu PDF (com --ocad também um ficheiro OCAD). Vistas: `no-paths` (sem estradas, caminhos, escadas), `brown-only` (só símbolos castanhos e a moldura), `no-north-lines`, ou uma lista `no:<codes>` / `only:<codes>`; combina com `+`. Um ficheiro de vista existente fica, a menos que indiques --force
help-run-pdf-scale-bar = Desenhar a barra de escala nos PDF (desligada por predefinição; o número da escala fica no bloco de título)
help-run-pdf-sources = Linhas de fontes nos PDF: `auto` imprime a lista completa e só a encurta quando ocuparia mais de 5 linhas, `full` nunca encurta, `credit-only` imprime só a linha de créditos do freemapper (a ODbL e a dl-de/by-2-0 pedem então que nomeies as fontes ao passares o mapa a outros). ATTRIBUTION.txt lista sempre todas as fontes
help-select = Escolher a área do mapa no navegador, desenhando-a ou escolhendo um local. Ao guardar escreve freemapper.toml e cria a pasta do projeto, se for preciso
help-select-dir = Pasta do projeto (criada ao guardar, se não existir)
help-select-kind = Tipo de mapa de um novo projeto
help-select-title = Título do mapa de um novo projeto
help-select-no-browser = Só imprimir o endereço da página; não abrir o navegador
help-select-port = Porta em 127.0.0.1 (predefinição: uma livre)
help-select-offline = A página começa com "só cache, sem rede" marcado
help-select-ocad = A página começa com o projeto inicial OCAD marcado
help-select-pdf = A página começa com o PDF para impressão marcado
help-select-pdf-candidates = A página começa com a impressão de verificação (todos os candidatos) marcada
help-select-training = A página começa com o mapa de treino marcado
help-select-pdf-scale-bar = A página começa com a barra de escala nos PDF marcada
help-select-pdf-sources = A página começa com esta escolha de linhas de fontes nos PDF, como em `run`
help-find = Listar os locais do OpenStreetMap para um nome, áreas primeiro, com o id para init --osm
help-find-query = Nome do local, p. ex. "Clara-Zetkin-Schule Strausberg"
help-find-json = Imprimir as correspondências como JSON, com os mesmos campos da ferramenta MCP find_place
help-mcp = Disponibilizar as ferramentas a um agente de IA por MCP (stdin/stdout)
help-update = Instalar um freemapper mais recente das versões do GitHub, depois de perguntar. Uma compilação a partir do código-fonte diz antes como atualizá-la
help-update-check = Só verificar: código de saída 10 quando existe uma versão mais recente, 0 quando esta está atualizada
help-update-yes = Instalar sem perguntar; necessário quando a entrada não é um terminal
help-update-version = Instalar esta versão (x.y.z) em vez da mais recente, também uma mais antiga
help-update-rollback = Voltar à versão que a última atualização substituiu
help-update-mode = Guardar o que a verificação diária faz quando encontra uma versão mais recente (predefinição: notify). Funciona em todas as compilações, porque só escreve a definição
help-adapters = Mostrar os adaptadores de região: os estados federados e países para os quais o freemapper tem fontes de dados
help-adapters-list = Listar os adaptadores com a origem de cada um (integrado, a tua pasta, um projeto), a sua revisão, o dia da última verificação e os anfitriões que consulta
help-adapters-list-json = Imprimir a lista como JSON
help-adapters-list-project = Ler também os adaptadores da pasta `adapters` desta pasta de projeto, como faria --adapters-from-project
help-adapters-check = Verificar a pasta de um adaptador antes de ser usado ou passado a outros: se carregaria, o que o seu autor deve saber e, com --live, se os seus serviços respondem. Código de saída 0: passa; 1: tem observações; 2: a verificação não pôde ser executada
help-adapters-check-dir = A pasta do adaptador; o seu nome é o id do adaptador
help-adapters-check-live = Consultar também os serviços do adaptador para a caixa que --bbox indica e dizer o que cada um entrega. Os pedidos ficam nos anfitriões que o manifesto declara
help-adapters-check-bbox = A caixa da verificação em direto em graus, como minLon,minLat,maxLon,maxLat. Fica dentro do limite do adaptador; 150 m por 150 m chegam
help-adapters-check-json = Imprimir o resultado como JSON
help-adapters-probe = Consultar uma vez cada serviço que os adaptadores integrados nomeiam e dizer se responde: ok, slow, moved ou error. O código de saída é 0 mesmo quando um serviço falha; o resultado é a resposta
help-adapters-probe-json = Imprimir o resultado como JSON
help-adapters-allow-host = Permitir a um adaptador integrado mais um anfitrião: para um estado federado cujo serviço se mudou para um servidor que o seu adaptador ainda não nomeia. A mensagem sobre o endereço recusado diz qual é o anfitrião. O anfitrião é verificado como um anfitrião de um manifesto, e o freemapper continua a ligar-se apenas a servidores na internet
help-adapters-allow-host-adapter = O id do adaptador integrado, como `adapters list` o mostra
help-adapters-allow-host-host = O anfitrião a permitir: um nome DNS, sem endereço, sem porta
help-adapters-disallow-host = Retirar um anfitrião que allow-host permitiu
help-adapters-disallow-host-adapter = O id do adaptador integrado
help-adapters-disallow-host-host = O anfitrião a retirar
help-arg-adapters-from-project = Carregar também os adaptadores da pasta `adapters` da pasta do projeto. O freemapper nomeia cada um com os anfitriões que consulta. Usa-o para um projeto em que confies: um adaptador faz o freemapper consultar os servidores que ele nomeia

## The summary of a run, for the MCP server and the area page

cli-internal-error = erro interno: { $message }
cli-pdf-sources-invalid = pdf_sources tem de ser auto, full ou credit-only, não { $value }
cli-view-unknown = uma vista é no-paths, brown-only, no-north-lines, no:<codes> ou only:<codes>, não { $value }
cli-view-empty = a lista { $value } não nomeia nenhum código de símbolo
cli-view-bad-code = { $value } não é um código de símbolo como 505 ou 501.6
cli-summary-training-view = Escritos { $map } e { $pdf } (uma vista do mapa de treino)
cli-summary-kept-training-view = Mantidos os ficheiros da vista { $view } do mapa de treino; indica --force para os renovar
cli-summary-training-view-failed = A vista do mapa de treino falhou: { $error }
cli-estimate-disk-unreadable = não é possível ler o espaço livre em disco de { $path } ({ $message })
cli-summary-wrote-map = Escrito o mapa { $path }
cli-summary-kept-map = Mantido o mapa existente { $path } (pode conter o trabalho do cartógrafo)
cli-summary-map-lacks-templates = O mapa ainda não referencia os modelos de imagem. Inicia o pipeline com force para o substituir e guardar o antigo como .omap.bak.
cli-summary-map-lacks-buildings = O mapa ainda não tem edifícios ALKIS. Inicia o pipeline com force para os adicionar; substitui o mapa e guarda o antigo como .omap.bak.
cli-summary-map-lacks-canopies = Ao mapa faltam as coberturas e abrigos de automóveis do ALKIS (522) que agora são objetos do mapa. Inicia o pipeline com force para os adicionar; substitui o mapa e guarda o antigo como .omap.bak.
cli-summary-map-light = O mapa base: { $joined } linhas unidas, { $left } escondidos ou demasiado pequenos deixados para as camadas de candidatos (nada foi movido)
cli-summary-map-light-kept = As camadas de candidatos passam a conter também { $left } objetos ou troços que esta versão tira do mapa base (marcados freemapper:generalized). O mapa mantido ainda os tem, por isso aparecem duas vezes. Inicia o pipeline com force para limpar o mapa; guarda o antigo como .omap.bak.
cli-summary-map-old-candidates = O mapa ainda refere templates/candidates.omap; esta execução preencheu-o com todos os candidatos. Os candidatos vêm agora também em sete camadas, cada uma num ficheiro (templates/candidates-*.omap). O mapa lista-as depois de uma execução com force (o mapa antigo fica como .omap.bak), ou o cartógrafo adiciona-as no OOM (Templates > Open template…).
cli-summary-wrote-ocad = Escrito o mapa OCAD { $path } ({ $count } objetos)
cli-summary-kept-ocad = Mantido o mapa OCAD existente { $path } (pode conter o trabalho do cartógrafo; force substitui-o)
cli-summary-ocad-old-candidates = O mapa OCAD ainda refere { $path } como mapa de fundo; esta execução preencheu-o com todos os candidatos. Os candidatos vêm agora também em sete camadas, cada uma num ficheiro (-candidates-*.ocd). O mapa OCAD lista-as depois de uma execução com force (o mapa antigo fica como .ocd.bak), ou o cartógrafo adiciona-as no OCAD como mapas de fundo.
cli-summary-ocad-left-out = Deixados fora do mapa OCAD: { $skipped } objetos e { $dropped } partes degeneradas
cli-summary-ocad-symbols = No OCAD, escolhe Symbol > Load Colors and Symbols From… e seleciona o conjunto de símbolos oficial{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , o conjunto escolar da IOF 2019
       *[other] {" "}ISSprOM 2019
    }; este substitui os símbolos provisórios pelos de igual número.
cli-summary-ocad-failed = A exportação OCAD falhou: { $error }
cli-summary-pdf-without-credits = PDF sem créditos das fontes: a ODbL e a dl-de/by-2-0 pedem que nomeies as fontes ao passares o mapa a outros; ver ATTRIBUTION.txt
cli-summary-wrote-pdf = Escrito o mapa { $path } (sem candidatos)
cli-summary-pdf-failed = A exportação do PDF falhou: { $error }
cli-summary-wrote-pdf-candidates = Escrita a impressão de verificação { $path } (o mapa com todos os candidatos, fiel à posição)
cli-summary-pdf-candidates-failed = A exportação da impressão de verificação falhou: { $error }
cli-summary-training-failed = A exportação do mapa de treino falhou: { $error }
cli-summary-kept-training = Mantido o mapa de treino existente { $path }, para que uma execução não substitua um que foi impresso ou entregue; apaga-o ou define force para o renovar

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = nenhuma pasta pessoal neste sistema; usa um caminho absoluto
mcp-error-relative-path = { $path } é um caminho relativo; usa um caminho absoluto ou um que comece por ~
mcp-error-home-folder = { $path } é a própria pasta pessoal; usa uma pasta dentro dela, p. ex. ~/maps/school
mcp-error-no-project = nenhum projeto freemapper em { $path }; usa primeiro create_project
mcp-error-unknown-kind = tipo desconhecido { $kind }; usa "schoolyard", "sprint" ou "forest"
mcp-error-reading = a ler { $path }: { $error }
mcp-error-one-area-source = indica exatamente um de osm_id (de find_place) ou geojson
mcp-error-page-open = a página de seleção da área deste projeto ainda está aberta; pede ao utilizador que guarde a área e prima Concluído na página, e depois inicia o pipeline
mcp-error-no-job = nenhum trabalho { $job }; start_pipeline e permits_for_area devolvem os ids de trabalho desta sessão
mcp-error-job-running = { $job } ainda está em execução para { $path }; só corre um pipeline de cada vez porque os projetos partilham a cache de descargas. Inicia este quando project_status indicar finished ou failed
mcp-error-page-start = não é possível iniciar a página: { $error }
mcp-created-project = Projeto { $path } criado para { $area } ({ $kind }, 1:{ $scale }). A seguir: estimate_download e depois start_pipeline.
mcp-created-project-given-area = Projeto { $path } criado para a área indicada ({ $kind }, 1:{ $scale }). A seguir: estimate_download e depois start_pipeline.
mcp-page-already-open = A página desta pasta já está aberta; mantém as definições com que foi aberta.
mcp-page-next = Pede ao utilizador que escolha a área na página, prima Guardar e depois Concluído.
mcp-preview-of = Pré-visualização de { $layer }
mcp-permits-next = As notas de licenciamento estão a ser criadas. Consulta project_status com este job_id; quando estiver finished, permits_md contém o texto de permits.md. Passa-o como indicação, não como aconselhamento jurídico.
mcp-tool-check-update = Se existe uma versão mais recente do freemapper: a sua versão, data de lançamento, notas de versão e tamanho da descarga, ou que esta está atualizada. Não instala nada.
mcp-tool-create-project = Criar uma pasta de projeto para uma área de mapa. Indica osm_id (de find_place) ou um ficheiro GeoJSON. kind é schoolyard (1:1000, predefinição), sprint (1:4000) ou forest (1:15000).
mcp-param-create-project-dir = Nova pasta de projeto (caminho absoluto ou começado por ~).
mcp-param-create-project-geojson = Em vez de osm_id: um ficheiro GeoJSON com o polígono da área do mapa (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (predefinição, 1:1000), "sprint" (1:4000) ou "forest" (1:15000).
mcp-param-create-project-osm-id = Objeto OSM da área do mapa, de find_place, p. ex. "way/992423119".
mcp-param-create-project-part = Com osm_id: que parte de uma área de várias partes separadas usar (1 é a maior).
mcp-param-create-project-scale = Denominador da escala: recreio escolar 1000 (predefinição); sprint 4000 (predefinição) ou 5000; floresta 15000 (predefinição), 10000 ou 7500. Outros valores são recusados; para 1:3000 desenha a 4000, o freemapper-print amplia ao imprimir.
mcp-param-create-project-title = Título do mapa para a informação marginal (predefinição: o nome da pasta).
mcp-tool-estimate-download = O que start_pipeline vai descarregar para um projeto: os mosaicos, cada descarga com o seu tamanho em MB, se está na cache e se cabe no disco, os MB que faltam descarregar e, em disk, o espaço livre dos volumes da cache e do projeto, com mensagens sobre o que não cabe (diz ao utilizador).
mcp-param-estimate-download-dir = Pasta do projeto (caminho absoluto ou começado por ~).
mcp-tool-find-place = Procurar uma escola, parque ou outro local por nome (OpenStreetMap Nominatim). Devolve até 5 correspondências, áreas primeiro; só as correspondências com has_area podem ser uma área de mapa. Uma lista vazia significa que nada foi encontrado.
mcp-param-find-place-query = Nome da escola ou do local, p. ex. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Instalar a versão mais recente do freemapper por cima deste programa. Chama-a só depois de a pessoa concordar, com confirm true. A nova versão aplica-se quando o assistente reiniciar o servidor freemapper.
mcp-param-install-update-confirm = true quando a pessoa concordou em instalar a versão mais recente; sem isso nada é instalado.
mcp-tool-list-layers = As camadas de um projeto: o mapa, as sete camadas de candidatos e os modelos raster, cada um com o seu ficheiro, se já foi criado e que passo o cria.
mcp-param-list-layers-dir = Pasta do projeto (caminho absoluto ou começado por ~).
mcp-tool-open-area-selector = Abrir no navegador uma página onde o utilizador desenha a área do mapa (retângulo ou polígono) ou escolhe um local, vê os mosaicos e o tamanho da descarga, e a guarda; ao guardar cria a pasta do projeto ou altera a sua área. Devolve o URL a dar ao utilizador. A página corre até o utilizador premir Concluído; chamar de novo para a mesma pasta devolve o mesmo URL.
mcp-param-open-area-selector-dir = Pasta do projeto (caminho absoluto ou começado por ~); criada quando o utilizador guarda.
mcp-param-open-area-selector-kind = Tipo de mapa de um novo projeto: "schoolyard" (predefinição, 1:1000), "sprint" (1:4000) ou "forest" (1:15000).
mcp-param-open-area-selector-ocad = O painel de execução começa com o projeto inicial OCAD marcado. Predefinição false.
mcp-param-open-area-selector-offline = O painel de execução começa com "só cache, sem rede" marcado. Predefinição false.
mcp-param-open-area-selector-open-browser = Abrir também a página no navegador predefinido deste computador. Predefinição true.
mcp-param-open-area-selector-pdf = O painel de execução começa com o PDF para impressão marcado. Predefinição false.
mcp-param-open-area-selector-pdf-candidates = O painel de execução começa com a impressão de verificação (todos os candidatos) marcada. Predefinição false.
mcp-param-open-area-selector-pdf-scale-bar = O painel de execução começa com a barra de escala nos PDF marcada. Predefinição false.
mcp-param-open-area-selector-pdf-sources = Linhas de fontes nos PDF com que o painel começa: "auto" (predefinição), "full" ou "credit-only", como em start_pipeline.
mcp-param-open-area-selector-permits = O painel de execução começa com as notas de licenciamento marcadas. Predefinição false.
mcp-param-open-area-selector-scale = Denominador da escala de um novo projeto: recreio escolar 1000 (predefinição); sprint 4000 (predefinição) ou 5000; floresta 15000 (predefinição), 10000 ou 7500. Outros valores são recusados.
mcp-param-open-area-selector-title = Título do mapa de um novo projeto.
mcp-param-open-area-selector-training = O painel de execução começa com o mapa de treino marcado. Predefinição false.
mcp-tool-preview-layer = Uma pré-visualização PNG (no máximo 800 px) de uma camada: map (a imagem dos objetos do mapa que a última execução escreveu, sem os candidatos), ou um modelo raster: orthophoto, hillshade, slope, relief, ndsm, ou k5 em Berlim.
mcp-param-preview-layer-dir = Pasta do projeto (caminho absoluto ou começado por ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" ou "k5" (só Berlim).
mcp-tool-project-status = Estado de um trabalho de start_pipeline ou permits_for_area: running, finished ou failed, as últimas linhas de progresso, avisos, os ficheiros escritos e o resultado ou o erro. Um trabalho permits_for_area terminado traz também o texto de permits.md como permits_md.
mcp-param-project-status-job-id = Id do trabalho de start_pipeline ou permits_for_area, p. ex. "job-1".
mcp-tool-permits-for-area = Criar as notas de licenciamento de um projeto: as unidades administrativas da área do mapa, as áreas protegidas nela e quem gere o terreno aí, com as autoridades a quem perguntar; uma indicação, não aconselhamento jurídico. Escreve permits.md e permits.json na pasta. Corre em segundo plano e devolve um job_id; consulta project_status, cuja resposta traz o texto de permits.md como permits_md quando o trabalho termina. Só corre um trabalho de cada vez.
mcp-param-permits-for-area-dir = Pasta do projeto (caminho absoluto ou começado por ~).
mcp-param-permits-for-area-offline = Ler só os dados guardados no projeto por uma execução anterior, sem acesso à rede. Predefinição false.
mcp-tool-start-pipeline = Descarregar os dados, calcular os modelos e escrever o projeto inicial (fetch, derive, export). Corre em segundo plano e devolve um job_id; consulta project_status. Só corre um pipeline de cada vez. Mantém um mapa existente, a menos que force seja true.
mcp-param-start-pipeline-dir = Pasta do projeto (caminho absoluto ou começado por ~).
mcp-param-start-pipeline-force = Substituir um ficheiro de mapa existente e guardar o antigo como .omap.bak. Predefinição false.
mcp-param-start-pipeline-ocad = Escrever também um ponto de partida OCAD na pasta ocad/ (formato OCAD 12, para quem edita no OCAD). Predefinição false.
mcp-param-start-pipeline-offline = Usar só a cache de descargas, sem acesso à rede. Predefinição false.
mcp-param-start-pipeline-pdf = Escrever também <name>.pdf, o mapa pronto a imprimir (só os objetos do mapa, sem candidatos). Predefinição false.
mcp-param-start-pipeline-pdf-candidates = Escrever também <name>-candidates.pdf, a impressão de verificação: o mapa com todos os candidatos, fiel à posição (verifica-os no terreno). Predefinição false.
mcp-param-start-pipeline-pdf-scale-bar = Desenhar a barra de escala nos PDF. Predefinição false (os mapas alemães normalmente não a têm; o número da escala fica no bloco de título).
mcp-param-start-pipeline-pdf-sources = Linhas de fontes nos PDF: "auto" (predefinição; a lista completa, encurtada só quando ocupa mais de 5 linhas), "full" (nunca encurtada) ou "credit-only" (só a linha de créditos do freemapper; a ODbL e a dl-de/by-2-0 exigem então que as fontes sejam nomeadas quando o mapa é passado a outros). ATTRIBUTION.txt lista sempre todas as fontes.
mcp-param-start-pipeline-permits = Escrever também as notas de licenciamento (permits.md e permits.json) depois do mapa: unidades administrativas, áreas protegidas e quem gere o terreno. Uma falha aí é um aviso, não um trabalho falhado. Predefinição false.
mcp-param-start-pipeline-training = Escrever também o mapa de treino <name>-training.omap e o seu PDF <name>-training.pdf: o mapa e todos os candidatos, limpos para serem impressos tal como estão; com ocad também ocad/<name>-training.ocd. Um mapa de treino existente é mantido, a menos que force seja true. Predefinição false.
mcp-param-start-pipeline-training-views = Com training: escrever também o mapa de treino sem o que uma vista omite, um conjunto de ficheiros por entrada: "no-paths", "brown-only", "no-north-lines", ou uma lista como "no:505,506" ou "only:101,102", combinada com +. Os ficheiros são <name>-training-<view>.omap e o seu PDF.
mcp-instructions = O freemapper cria projetos iniciais de mapas de orientação para o OpenOrienteering Mapper (recreio escolar 1:1000, sprint 1:4000, floresta 1:15000) a partir de geodados oficiais de { $states } e do OSM. Fluxo: find_place (procurar a escola ou a área por nome; escolher um resultado com has_area) → create_project (uma nova pasta, caminho absoluto, com o osm_id) → estimate_download (dizer ao utilizador quanto será descarregado) → start_pipeline (devolve um id de trabalho; a descarga pode demorar minutos) → project_status (consultar a cada poucos segundos até o estado ser finished ou failed) → list_layers e preview_layer (mostrar ao utilizador os modelos). O utilizador abre depois o ficheiro do mapa no OpenOrienteering Mapper. Quando nenhum local serve ou o utilizador quer desenhar a área, open_area_selector abre no navegador uma página onde o utilizador desenha ou escolhe a área e a guarda (isto cria o projeto); continua com estimate_download quando o utilizador disser que está guardada. Os caminhos têm de ser absolutos ou começar por ~. Só corre um pipeline de cada vez. permits_for_area cria as notas de licenciamento de um projeto (unidades administrativas, áreas protegidas, quem gere o terreno; uma indicação, não aconselhamento jurídico) como um trabalho igual a start_pipeline. check_update diz se existe um freemapper mais recente; install_update instala-o, só com confirm: true depois de a pessoa concordar, e o assistente reinicia então o servidor freemapper.

cli-contour-interval-saved = Equidistância das curvas de nível guardada em freemapper.toml: { $interval } m; derive traça as curvas de nível com ela
cli-contour-interval-auto = Equidistância das curvas de nível em freemapper.toml: automática (a do tipo de mapa; um mapa de floresta tem 2,5 m em terreno plano e 10 m em terreno íngreme)
help-arg-contour-interval = Equidistância das curvas de nível em metros, p. ex. `2.5`, ou `auto` para a do tipo de mapa (um mapa de floresta: 5 m, 2,5 m onde é plano, 10 m onde é íngreme). Guardada em freemapper.toml; derive traça as curvas de nível com ela
cli-content-saved = Definição de conteúdo guardada em freemapper.toml: { $summary }
cli-content-invalid = Não é possível ler a definição de conteúdo: { $detail }
help-arg-content = O que vai diretamente para o mapa: `safe` deixa cada objeto onde a construção o põe, `balanced` (predefinição) põe no mapa os objetos do cadastro e dos registos, `full` põe tudo no mapa. A escolha é guardada em freemapper.toml
help-arg-content-set = Enviar um grupo de objetos para `map`, `candidates` ou `off`, como `GROUP=PLACE`, p. ex. `walls=map`; pode ser repetido. Grupos: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Guardado em freemapper.toml
mcp-param-create-project-content = O que vai diretamente para o mapa: "safe", "balanced" (predefinição) ou "full". Guardado no projeto. balanced põe no mapa os objetos do cadastro e dos registos, full põe tudo.
mcp-param-create-project-content-set = Grupos enviados para outro sítio que não o do perfil, como "group=place" com place map, candidates ou off, p. ex. "walls=map". Grupos: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Equidistância das curvas de nível em metros, p. ex. 2.5; 0 volta à escolha automática (a do tipo de mapa; um mapa de floresta tem 2,5 m em terreno plano e 10 m em terreno íngreme). Guardada no projeto; o próximo derive traça as curvas de nível com ela.
mcp-param-start-pipeline-content = Alterar o que vai diretamente para o mapa antes da execução: "safe", "balanced" ou "full". Guardado no projeto. Predefinição: a definição do projeto, senão balanced.
mcp-param-start-pipeline-content-set = Grupos enviados para outro sítio que não o do perfil, como "group=place" com place map, candidates ou off, p. ex. "fences=candidates". Guardado no projeto.
mcp-param-start-pipeline-contour-interval = Equidistância das curvas de nível em metros, p. ex. 2.5; 0 volta à escolha automática (a do tipo de mapa; um mapa de floresta tem 2,5 m em terreno plano e 10 m em terreno íngreme). Guardada no projeto; o próximo derive traça as curvas de nível com ela.
mcp-error-content = Não é possível ler a definição de conteúdo: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Escrever as notas de licenciamento de um projeto (permits.md e permits.json): as unidades administrativas, áreas protegidas e gestores que a área do mapa toca, e a quem, por norma, é preciso perguntar. Consulta o BfN e o BKG e, onde o estado federado da área do mapa tem serviços abertos próprios, esses através dos anfitriões do seu adaptador (lista no manual, capítulo 12); na Renânia do Norte-Vestefália descarrega para isso cerca de 32 MB da primeira vez, em Brandemburgo cerca de 4 MB, depois só ficheiros mais recentes. Lê o extrato OSM já guardado na cache; --offline usa o que foi obtido antes
help-permits-dir = Pasta do projeto
help-permits-offline = Usar só os dados obtidos antes, sem acesso à rede
help-run-permits = Escrever também as notas de licenciamento (permits.md e permits.json) depois do mapa; consulta o BfN, o BKG e os serviços abertos do estado federado (manual, capítulo 12). Uma falha aí é um aviso e o mapa fica
cli-wrote-permits = Escritas as notas de licenciamento { $path }
cli-summary-wrote-permits = Escritas as notas de licenciamento { $path }
cli-permits-failed = não foi possível escrever as notas de licenciamento: { $error }
cli-permits-source-left-out = as notas de licenciamento omitem { $source }: { $reason }
cli-permits-not-read = não foi lida
error-permits-stored = { $path }: { $message }; obtém de novo as notas de licenciamento com rede
error-permits-nothing-read = não foi possível ler nenhuma fonte das notas de licenciamento ({ $missing }); obtém-nas primeiro com rede
error-permits-missing-not-read = { $source }: não lida

permits-title = Notas de licenciamento
permits-disclaimer = Uma indicação, não aconselhamento jurídico. As autoridades competentes decidem. Para a data e a exatidão dos dados, vê a tabela em "Fontes e data dos dados"; podem faltar áreas ou estar desatualizadas (BfN: reunidas uma vez por ano, "Nicht für Planungszwecke geeignet.", isto é, não adequadas ao planeamento).
permits-disclaimer-regional = Uma indicação, não aconselhamento jurídico. As autoridades competentes decidem. Para a data e a exatidão dos dados, vê a tabela em "Fontes e data dos dados"; podem faltar áreas ou estar desatualizadas.
permits-bfn-merge = O BfN reúne os dados dos estados federados uma vez por ano e marca-os como não adequados ao planeamento. As autoridades do estado federado decidem.
permits-created = Feitas em { $date } para a área do mapa de um { $kind ->
        [schoolyard] mapa de recreio escolar
        [forest] mapa de floresta
       *[other] mapa de sprint
    } ({ $area } ha).
permits-section-admin = Unidades administrativas
permits-section-protected = Áreas protegidas (BfN)
permits-section-osm-protected = Mais áreas protegidas do OpenStreetMap
permits-section-operators = Gestores e proprietários segundo o OpenStreetMap
permits-section-hints = Indicações
permits-section-sources = Fontes e data dos dados
permits-none-admin = O VG250 não nomeia nenhuma unidade administrativa para a área do mapa.
permits-none-protected = Segundo o BfN, nenhuma área protegida fica na área do mapa.
permits-none-osm-protected = O OpenStreetMap não nomeia nenhuma área protegida na área do mapa que o BfN não liste já.
permits-none-operators = O OpenStreetMap não nomeia nenhum gestor nem proprietário para a área do mapa.
permits-none-hints = Sem indicações.
permits-section-not-read = Os dados de { $source } não foram lidos, por isso esta secção está vazia, embora possa haver entradas.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Áreas administrativas 1:250 000 (VG250) do BKG
        [bfn] Áreas protegidas na Alemanha do BfN
       *[osm] OpenStreetMap, a partir do extrato que o freemapper guarda na sua cache
    }
permits-level = { $level ->
        [land] Estado federado (Land)
        [kreis] Distrito (Kreis)
        [verwaltungsgemeinschaft] Amt ou Verwaltungsgemeinschaft (associação de municípios)
       *[gemeinde] Município (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Tocadas na orla (menos de 0,5% da área do mapa):
permits-at-edge-generalised = O VG250 está generalizado para a escala 1:250 000. Uma unidade que só toca a orla pode ser um efeito disso.
permits-kind = { $kind ->
        [nationalpark] Parque nacional
        [nationales-naturmonument] Monumento natural nacional
        [naturschutzgebiet] Reserva natural (Naturschutzgebiet)
        [ffh] Área FFH (Natura 2000)
        [vogelschutzgebiet] Área de proteção de aves (Natura 2000)
        [biosphaerenreservat] Reserva da biosfera
        [biosphaerenreservat-zone] Zona de uma reserva da biosfera
        [landschaftsschutzgebiet] Área de paisagem protegida (Landschaftsschutzgebiet)
       *[naturpark] Parque natural
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Designada em { $date }
permits-protected-title = Base legal: { $title }
permits-protected-link = Texto legal: { $url }
permits-protected-site-link = Página da área (Natura 2000): { $url }
permits-protected-year = Ano dos dados no BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = classe de proteção { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = sem nome
permits-osm-operator = Gestor segundo o OSM: { $operator }
permits-osm-enclosing-gap = Podem faltar aqui áreas do OpenStreetMap que envolvem toda a área do mapa sem um canto dentro dela, e também áreas protegidas mapeadas no OSM como relação com `type=boundary`. As partes do BfN e do VG250 não são afetadas.
permits-osm-enclosing-gap-short = Aqui podem faltar áreas do OpenStreetMap que envolvem toda a área do mapa sem ter um canto dentro dela.
permits-operator-line = { $name } ({ $roles ->
        [operator] gestor
        [owner] proprietário
       *[both] gestor e proprietário
    }): { $objects }
permits-column-source = Fonte
permits-column-state = Estado
permits-column-credit = Créditos e licença
permits-state-read = lida, obtida em { $date }
permits-state-read-osm = lida, extrato OSM de { $date }
permits-state-unreachable = inacessível ({ $reason })
permits-state-not-read = não lida
permits-no-credit = não usada
permits-hint-city-state = A área do mapa fica em Berlim ou Hamburgo. Aí costuma ser competente o Bezirksamt (câmara do distrito urbano) ou uma autoridade do estado federado. Estas notas ainda não determinam o Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] As áreas protegidas dizem respeito à autoridade inferior de conservação da natureza (untere Naturschutzbehörde). Aqui é a própria cidade de { $kreis }.
        [no] As áreas protegidas dizem respeito à autoridade inferior de conservação da natureza (untere Naturschutzbehörde), normalmente o { $kreis }.
       *[unknown] As áreas protegidas dizem respeito à autoridade inferior de conservação da natureza (untere Naturschutzbehörde) competente, normalmente o Landkreis ou a kreisfreie Stadt.
    }
permits-hint-national-park = Um parque nacional tem regras próprias. Pergunta cedo à administração do parque nacional.
permits-hint-nsg-paths = Numa reserva natural costuma ser obrigatório manter-se nos caminhos. Para uma corrida fora dos caminhos, pergunta à autoridade de conservação da natureza.
permits-hint-natura2000 = Uma área FFH ou uma área de proteção de aves (Natura 2000) toca a área do mapa. Para um evento, a autoridade de conservação da natureza pode exigir uma avaliação dos seus efeitos.
permits-hint-protected-areas-pl = Na área do mapa há áreas ou objetos protegidos de conservação da natureza. Para um parque nacional, um parque paisagístico ou uma reserva natural, o contacto habitual é a administração do parque ou da reserva; para as outras áreas e objetos, a Direção Regional de Proteção do Ambiente (RDOŚ) do voivodato costuma poder dizer quem é o responsável. Pergunta cedo se planeias um evento aí.
permits-hint-order-office = { $amt ->
        [none] Para ruas, praças e outros terrenos públicos costuma ser competente o Ordnungsamt (serviço de ordem pública) do Gemeinde { $gemeinde }.
       *[other] Para ruas, praças e outros terrenos públicos costuma ser competente o Ordnungsamt (serviço de ordem pública): { $amt }, para o Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Gestor de um parque ou jardim na área do mapa segundo o OpenStreetMap: { $operator }. Pergunta-lhe antes de o usares.
permits-hint-school-operator = Gestor da escola ou instituição segundo o OpenStreetMap: { $operator }. Para os seus terrenos, pergunta à direção da escola ou a este organismo.
permits-section-districts = Bezirke (distritos urbanos, { $source })
permits-section-green-spaces = Espaços verdes ({ $source })
permits-section-owners = Terrenos do estado federado e do Bund ({ $source })
permits-section-monuments = Monumentos ({ $source })
permits-section-protected-objects = Monumentos naturais e outros objetos protegidos ({ $source })
permits-none-districts = Segundo { $source }, a área do mapa não fica em nenhum Bezirk.
permits-none-green-spaces = Segundo { $source }, nenhum espaço verde público fica na área do mapa.
permits-none-owners = Segundo { $source }, nenhuma parcela do estado federado ou do Bund fica na área do mapa.
permits-none-monuments = Segundo { $source }, nenhum monumento fica na área do mapa.
permits-none-protected-objects = Segundo { $source }, nenhum dos objetos protegidos que contém fica na área do mapa.
permits-none-sampled = Nos pontos consultados, { $source } não nomeia nada aqui.
permits-gaps = Os dados de { $source } não estão completos: { $gaps }.
permits-sampled = { $source } só responde para pontos isolados. O freemapper consultou uma grelha de pontos sobre o mapa: as percentagens são estimativas, e o que fica entre os pontos pode faltar.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = afetação: { $dedication }
permits-playgrounds-none = Nenhum parque infantil público fica na área do mapa.
permits-playgrounds-one = Um parque infantil público fica na área do mapa. Não é listado individualmente.
permits-playgrounds-many = { $count } parques infantis públicos ficam na área do mapa. Não são listados um a um.
permits-owner-other = Outros proprietários públicos
permits-owners-rest = Para o resto da área os dados não nomeiam proprietário.
permits-monuments-total = Monumentos total ou parcialmente na área do mapa: { $count }.{ $links ->
        [yes] {" "}As ligações levam às suas entradas na lista de monumentos.
       *[no] {""}
    }
permits-monuments-links-left = Mais { $count } entradas só são contadas aqui, sem ligação.
permits-object-kind = { $kind ->
        [natural-monument] Monumento natural (Naturdenkmal)
        [protected-landscape-part] Elemento de paisagem protegido (geschützter Landschaftsbestandteil)
        [biotope] Biótopo legalmente protegido (geschütztes Biotop)
        [national-park] Parque nacional (park narodowy)
        [park-buffer-zone] Zona tampão de um parque (otulina)
        [landscape-park] Parque paisagístico (park krajobrazowy)
        [nature-reserve] Reserva natural (rezerwat przyrody)
        [protected-landscape-area] Área de paisagem protegida (obszar chronionego krajobrazu)
        [natura2000-habitats] Sítio Natura 2000 para habitats (SOO)
        [natura2000-birds] Sítio Natura 2000 para aves (OSO)
        [ecological-site] Área ecológica (użytek ekologiczny)
        [landscape-complex] Complexo natural e paisagístico (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Sítio de documentação (stanowisko dokumentacyjne)
        [natural-monument-pl] Monumento natural (pomnik przyrody)
       *[special-rules] Área com regras especiais
    }
permits-object-count = { $count } objetos
permits-object-citations = Bases legais:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Para ruas, praças e espaços verdes no Bezirk { $district }, costuma ser competente o Bezirksamt { $district }, aí o Fachamt Management des öffentlichen Raumes (serviço de gestão do espaço público).
       *[berlin] Para ruas, praças e espaços verdes no Bezirk { $district }, costumam ser competentes o Ordnungsamt (serviço de ordem pública) e o Straßen- und Grünflächenamt (serviço de estradas e parques) do Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Para os monumentos na área do mapa, é competente o Denkmalschutzamt Hamburg (serviço de monumentos).
       *[berlin] Para os monumentos no Bezirk { $district }, é competente a autoridade inferior de monumentos (untere Denkmalschutzbehörde) do Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Se há monumentos classificados na área do mapa, os dados aqui não o permitem dizer com certeza.
       *[yes] Há monumentos classificados na área do mapa.
    } Se uma corrida os toca, por exemplo com um controlo num monumento ou um percurso através de um local protegido, decide a autoridade inferior de monumentos (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Aqui é a própria cidade de { $kreis }.
        [no] , normalmente o { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; nalguns estados federados é o Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(normalmente o Landkreis ou a kreisfreie Stadt, nalguns estados federados o Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Os monumentos naturais, elementos de paisagem protegidos e biótopos protegidos no mapa são assunto da autoridade inferior de conservação da natureza (untere Naturschutzbehörde). Aqui é a própria cidade de { $kreis }.
        [no] Os monumentos naturais, elementos de paisagem protegidos e biótopos protegidos no mapa são assunto da autoridade inferior de conservação da natureza (untere Naturschutzbehörde), normalmente o { $kreis }.
       *[unknown] Os monumentos naturais, elementos de paisagem protegidos e biótopos protegidos no mapa são assunto da autoridade inferior de conservação da natureza (untere Naturschutzbehörde), normalmente o Landkreis ou a kreisfreie Stadt.
    }
permits-hint-biotope = Há biótopos legalmente protegidos na área do mapa{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. É proibido tudo o que os possa destruir ou prejudicar consideravelmente (§ 30 BNatSchG). Coloca controlos e percursos fora deles, ou pergunta à autoridade inferior de conservação da natureza (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Para os monumentos naturais e elementos de paisagem protegidos no Bezirk { $district }, costuma ser competente o Bezirksamt { $district }.
       *[berlin] Para os monumentos naturais e elementos de paisagem protegidos no Bezirk { $district }, é competente a autoridade inferior de conservação da natureza (untere Naturschutzbehörde) do Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Segundo os dados abertos do estado federado, { $share } da área do mapa pertence a outros proprietários públicos.
       *[no] Segundo os dados abertos do estado federado, { $share } da área do mapa pertence ao proprietário "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Os proprietários do resto da área não se encontram nos dados abertos. Com um interesse legítimo (um organizador, por exemplo) podes pedir um extrato ao registo predial (Grundbuchamt); para terrenos públicos ajuda o Ordnungsamt ou o serviço de parques competente.
       *[no] Os proprietários destas áreas não se encontram nos dados abertos. Com um interesse legítimo (um organizador, por exemplo) podes pedir um extrato ao registo predial (Grundbuchamt); para terrenos públicos ajuda o Ordnungsamt ou o serviço de parques competente.
    }
permits-section-forest-units = Administração florestal ({ $source })
permits-section-state-forest = Floresta do estado federado ({ $source })
permits-none-forest-units = Segundo { $source }, a área do mapa não fica na área de nenhum serviço florestal.
permits-section-admin-units = Unidades administrativas ({ $source })
permits-none-admin-units = Segundo { $source }, a área do mapa não se situa em nenhuma unidade administrativa do registo.
permits-section-register-monuments = Monumentos do registo nacional ({ $source })
permits-none-register-monuments = Segundo { $source }, nenhum monumento registado se encontra no retângulo à volta da área do mapa.
permits-register-monuments-total = Monumentos registados no retângulo à volta da área do mapa: { $count }. O registo não indica a posição, por isso estas notas não podem dizer onde estão; é preciso verificá-lo no mapa.
permits-register-monuments-left = Mais { $count } são apenas contados aqui, sem nome.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Segundo { $source }, nenhuma floresta do estado federado fica na área do mapa.
permits-forest-units-reach = Um serviço florestal é competente para a floresta da sua área. A sua área pode abranger também cidades e campos.
permits-state-forest-office = competente: { $office }
permits-hint-forest-office = { $units ->
        [none] Serviço florestal competente para a floresta na área do mapa: { $office } ({ $administration }). Pergunta cedo aí quando planeares um evento na floresta.
       *[other] Serviço florestal competente para a floresta na área do mapa: { $office } ({ $administration }), { $units }. Pergunta cedo aí quando planeares um evento na floresta.
    }
permits-hint-forest-owner = { $office ->
        [known] Um evento na floresta precisa do consentimento dos proprietários florestais. O serviço florestal costuma poder dizer quem é o proprietário da floresta. Em vários estados federados, um evento organizado na floresta tem também de ser comunicado à autoridade florestal ou autorizado por ela.
       *[unknown] Um evento na floresta precisa do consentimento dos proprietários florestais. Em vários estados federados, um evento organizado na floresta tem também de ser comunicado à autoridade florestal ou autorizado por ela. Estas notas não nomeiam o serviço florestal competente aqui; a administração florestal do estado federado ou a autoridade florestal inferior podem dizer-to.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Um evento na floresta costuma precisar do consentimento do proprietário da floresta. O distrito florestal (Nadleśnictwo) indicado acima costuma poder dizer quem cuida da floresta. A floresta que não pertence às Florestas do Estado, por exemplo uma floresta privada, não consta dos dados destas notas. A floresta que não pertence ao Estado, por exemplo uma floresta privada ou municipal, está sob a supervisão do starosta (starosta) quanto à gestão florestal (lei das florestas, art. 5 ust. 1 pkt 2); pergunte aí no starostwo powiatowe.
       *[unknown] Um evento na floresta costuma precisar do consentimento do proprietário da floresta. Estas notas não indicam nenhum distrito florestal das Florestas do Estado para a área do mapa; os dados abrangem apenas as Florestas do Estado, por isso a floresta pode pertencer a outra pessoa, por exemplo a um proprietário privado. Quem é o proprietário tem de ser apurado à parte. A floresta que não pertence ao Estado, por exemplo uma floresta privada ou municipal, está sob a supervisão do starosta (starosta) quanto à gestão florestal (lei das florestas, art. 5 ust. 1 pkt 2); pergunte aí no starostwo powiatowe.
    }
permits-hint-monument-register-pl = Há monumentos do registo nacional no retângulo à volta da área do mapa. As suas posições não constam dos dados; verifique no mapa onde estão. Segundo o art. 36 ust. 1 da lei de proteção dos monumentos (ustawa o ochronie zabytków), os trabalhos num monumento registado, as obras na envolvente de um monumento e outras ações que possam alterar a sua substância ou aspeto precisam da licença do conservador de monumentos da voivodia (Wojewódzki Konserwator Zabytków). A lei não nomeia uma corrida nem um controlo como tais; pergunte ao gabinete do conservador se um evento toca um monumento ou a sua envolvente.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] A área do mapa inclui um monumento natural (pomnik przyrody), uma área ecológica (użytek ekologiczny), um complexo natural e paisagístico (zespół przyrodniczo-krajobrazowy) ou um sítio de documentação (stanowisko dokumentacyjne). O conselho da gmina estabelece-os por deliberação, que indica quem fiscaliza o objeto e que proibições se aplicam (art. 44 da Lei de Conservação da Natureza). A própria lei não prevê aqui nenhuma licença para um evento. Peça a deliberação e o responsável pela fiscalização ao serviço da gmina em cujo território o objeto se encontra.
       *[other] A área do mapa inclui um monumento natural (pomnik przyrody), uma área ecológica (użytek ekologiczny), um complexo natural e paisagístico (zespół przyrodniczo-krajobrazowy) ou um sítio de documentação (stanowisko dokumentacyjne). O conselho da gmina estabelece-os por deliberação, que indica quem fiscaliza o objeto e que proibições se aplicam (art. 44 da Lei de Conservação da Natureza). A própria lei não prevê aqui nenhuma licença para um evento. Peça a deliberação e o responsável pela fiscalização ao serviço da gmina ({ $gmina }) em cujo território o objeto se encontra.
    }
permits-hint-gmina-roads-pl = Um evento que perturba o trânsito ou usa uma rua de forma especial precisa de licença nos termos do art. 65 do Código da Estrada, pedida com pelo menos 30 dias de antecedência. Quem a emite é o órgão que gere o trânsito na via: o starosta para as vias do powiat e da gmina, o presidente da câmara numa cidade com direitos de powiat, o marechal do voivodato ou a GDDKiA para as restantes vias (art. 10 e 65b). O gestor de uma via da gmina é o wójt, burmistrz ou prezydent miasta da gmina ({ $gmina }); o gestor decide sobre a ocupação da faixa da via (art. 19 e 40 da Lei das Estradas Públicas). Estas notas não mostram a categoria das vias.
permits-hint-gmina-land-pl = Se um terreno da área do mapa pertence à gmina, por exemplo um parque municipal, uma praça, um recinto escolar ou uma floresta, é administrado pelo wójt, burmistrz ou prezydent miasta, que representa a gmina (art. 30 e 31 da Lei da Autonomia da Gmina). Peça o consentimento ao serviço da gmina ({ $gmina }) antes de o usar. Estas notas não mostram a quem pertence o terreno.
permits-section-water-protection = Áreas de proteção da água ({ $source })
permits-section-forest-functions = Funções da floresta e floresta protegida ({ $source })
permits-none-water-protection = Segundo { $source }, nenhuma área de proteção da água fica na área do mapa.
permits-none-forest-functions = Segundo { $source }, nenhuma floresta de recreio, floresta protegida, floresta natural ou área florestal fechada fica na área do mapa.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Floresta de recreio (Erholungswald)
        [protected] Floresta protegida (Bannwald ou Schutzwald)
        [natural] Floresta natural ou reserva integral (Naturwald, Totalreservat)
       *[closed] Área florestal fechada
    }
permits-hint-water-protection = { $inner ->
        [yes] A { $zone } da área de proteção da água "{ $name }" fica na área do mapa. Nas zonas I e II os eventos costumam estar excluídos. Pergunta à autoridade inferior das águas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] A { $zone } da área de proteção da água "{ $name }" fica na área do mapa. Os eventos aí costumam ser possíveis; a autoridade inferior das águas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) pode dizer o que o regulamento permite.
       *[unknown] A área de proteção da água "{ $name }" fica na área do mapa. Os dados não dizem de que zona se trata. Nas zonas I e II os eventos costumam estar excluídos; pergunta à autoridade inferior das águas (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Há floresta de recreio (Erholungswald) na área do mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Para eventos aí, pergunta à autoridade florestal.
        [protected] Há floresta protegida (Bannwald ou Schutzwald) na área do mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pergunta à autoridade florestal antes de planeares aí um evento.
        [natural] Há floresta natural ou uma reserva integral na área do mapa{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Sair dos caminhos é aí muitas vezes proibido; não coloques aí controlos sem perguntar à autoridade florestal.
       *[closed] Há na área do mapa uma área florestal onde não se pode entrar{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Deixa-a fora do percurso.
    }
permits-section-roads = Estradas classificadas segundo o OpenStreetMap
permits-none-roads = Segundo o OpenStreetMap, nenhuma Autobahn nem Bundes-, Landes-, Staats- ou Kreisstraße atravessa a área do mapa.
permits-road-line = { $class ->
        [autobahn] Autobahn (autoestrada)
        [bundesstrasse] Bundesstraße (estrada federal)
        [landesstrasse] Landesstraße ou Staatsstraße (estrada do estado federado)
       *[kreisstrasse] Kreisstraße (estrada distrital)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] A Autobahn { $refs } atravessa a área do mapa. Um percurso só a pode cruzar por pontes ou passagens inferiores.
        [bundesstrasse] A Bundesstraße { $refs } atravessa a área do mapa. Um corte ou uma travessia com fiscais é ordenado pela autoridade de trânsito (Straßenverkehrsbehörde); a estrada costuma ser mantida pelo Bund através da administração de estradas do estado federado, nas cidades maiores muitas vezes pela cidade.
        [landesstrasse] A Landes- ou Staatsstraße { $refs } atravessa a área do mapa. Um corte ou uma travessia com fiscais é ordenado pela autoridade de trânsito (Straßenverkehrsbehörde); a estrada costuma ser mantida pelo estado federado, nas cidades maiores muitas vezes pela cidade.
       *[kreisstrasse] A Kreisstraße { $refs } atravessa a área do mapa. Um corte ou uma travessia com fiscais é ordenado pela autoridade de trânsito (Straßenverkehrsbehörde); a estrada costuma ser mantida pelo Kreis, nas cidades maiores muitas vezes pela cidade.
    }
permits-hint-state-forest = { $office ->
        [none] Segundo os dados abertos do estado federado, { $share } da área do mapa é floresta do estado federado do proprietário "{ $owner }".
       *[other] Segundo os dados abertos do estado federado, { $share } da área do mapa é floresta do estado federado do proprietário "{ $owner }". Competente: { $office }.
    }
