# Lignes de progression et avertissements du moteur (ADR 0092, étape 3). L'anglais est la référence.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = avertissement : { $text }

warning-other-zone = la zone de la carte se trouve dans le fuseau UTM { $zone } (EPSG:{ $epsg }) ; freemapper télécharge tout à nouveau dans ce fuseau
warning-other-state = la zone de la carte se trouve maintenant dans les données d'un autre Land ; freemapper télécharge tout à nouveau
warning-beyond-coverage = une partie de la zone de la carte se trouve hors des Länder pris en charge. Elle se trouve à l'étranger, en mer ou dans un Land que freemapper ne prend pas encore en charge, comme la Thuringe. Les Länder n'ont pas de données là-bas ; l'orthophotographie peut donc être blanche, et ALKIS ne donne ni bâtiments ni occupation du sol
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml : l'échelle 1:{ $scale } ne fait pas partie des échelles des cartes de type { $kind } ({ $allowed }) ; freemapper-print refusera la carte
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml : entrée { $key } des couches { $german ->
       *[any] { $what }
    } ignorée : { $reason ->
        [name] pas un simple nom de fichier
       *[unknown] aucune couche que freemapper connaît
    }

## Disk space

warning-disk-space-unreadable = impossible de lire l'espace disque libre de { $path } ({ $error }), freemapper ne le vérifie donc pas

## Tiles

warning-tile-changed = la tuile { $product } { $tile } a changé chez le fournisseur ({ $old } → { $new })
warning-bdom-tile-unavailable = modèle de surface (bDOM) de la tuile { $tile } indisponible ({ $error }) ; pas de modèle nDSM là-bas
warning-bdom-unavailable = modèle de surface (bDOM) indisponible ({ $error }) ; pas de modèle nDSM pour { $tiles }
progress-ndsm-from-bdom = nDSM de la tuile { $tile } à partir du modèle de surface (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] modèle de terrain
        [surface] modèle de surface
       *[orthophoto] orthophotographie
    } ({ $name })
progress-model-download = téléchargement de { $model }
progress-model-read = lecture de { $model }
warning-model-unavailable = { $model } indisponible ({ $error }) ; pas de modèle nDSM
warning-model-unavailable-kept = { $model } indisponible ({ $error }) ; le modèle de surface téléchargé le { $fetched } est conservé
warning-model-tile-unavailable = { $model } de la tuile { $tile } indisponible ({ $error }) ; pas de modèle nDSM là-bas
warning-archive-passed-over = { $model } : une archive n'a pas pu être lue ({ $error }) ; elle a été ignorée, et les tuiles de la zone de la carte viennent des autres archives
warning-archive-directory-kept = { $model } : une archive n'a pas pu être lue ({ $error }) ; son répertoire conservé d'une exécution précédente est utilisé, de sorte qu'une tuile que le LVGL a remplacée depuis n'est pas remarquée
warning-model-tiles-missing = { $model } n'a pas de { $count ->
        [one] tuile
       *[other] tuiles
    } { $names } dans { $state ->
        [the-map-area] la zone de la carte
        [hamburg] le Land de Hambourg
        [hesse] le Land de Hesse
        [north-rhine-westphalia] le Land de Rhénanie-du-Nord-Westphalie
        [schleswig-holstein] le Land de Schleswig-Holstein
        [mecklenburg-vorpommern] le Land de Mecklembourg-Poméranie-Occidentale
        [lower-saxony] le Land de Basse-Saxe
        [bavaria] le Land de Bavière
       *[other] { $state }
    } ; les modèles y restent vides
warning-no-surface-stored = aucun modèle de surface enregistré ; lancez `freemapper fetch` en ligne pour le modèle nDSM
warning-surface-older-than-orthophoto = le modèle de surface (bDOM { $surface }) est plus ancien que l'orthophotographie ({ $photo }) ; de nouveaux bâtiments peuvent manquer dans le nDSM
progress-surface-kept = freemapper conserve le modèle de surface téléchargé le { $fetched } pour le modèle nDSM
progress-surface-tiles-kept = freemapper conserve le modèle de surface en cache de { $count ->
        [one] la tuile
       *[other] des tuiles
    } { $names } pour le modèle nDSM
progress-orthophoto-unchanged = orthophotographie inchangée
progress-orthophoto-unchanged-dated = orthophotographie inchangée ({ $label } { $date })
progress-orthophoto-download = téléchargement de l'orthophotographie depuis { $provider } { $service }
progress-orthophoto-in-leaf = l'orthophotographie a été prise le { $date }, avec les arbres en feuilles ; leurs cimes cachent ce qui se trouve dessous
progress-orthophoto-bare = l'orthophotographie a été prise le { $date }, avec les arbres sans feuilles
warning-lgln-orthophoto-kept = orthophotographie (LGLN WMS) : { $reason } ; l'orthophotographie enregistrée et sa date de vol { $date } sont conservées
warning-lgln-orthophoto-undated = orthophotographie (LGLN WMS) : pas de date de vol, { $reason }
word-stac-no-image = le STAC ne liste aucune image au centre de la zone de la carte
word-stac-no-answer = la recherche STAC n'a donné aucune réponse utilisable ({ $error })
warning-downloadcenter-no-file = le Downloadcenter n'a pas de fichier pour { $places }
warning-downloadcenter-no-file-empty = le Downloadcenter n'a pas de fichier pour { $places } ; { $count ->
        [one] la tuile
       *[other] les tuiles
    } { $names } sont restées vides
progress-k-five-unchanged = modèle K5 inchangé
progress-k-five-download = téléchargement de la carte K5 (1:5000) depuis le WMS du Geoportal Berlin
warning-k-five-unavailable = le modèle de carte K5 est indisponible ({ $detail }) ; ignoré

## The state border

progress-beyond-border = la zone de la carte dépasse la frontière du Land ; au-delà, les modèles sont vides partout où s'arrêtent l'orthophotographie et les modèles d'altitude de { $provider }. ALKIS et OSM viennent des deux Länder
progress-beyond-border-no-data = la zone de la carte dépasse la frontière du Land ; pas de données { $provider } pour { $tiles }, les modèles y restent donc vides

## OSM

warning-osm-extract-missing-kept = aucun extrait OSM en cache ; les candidats OSM enregistrés du { $date } sont conservés
progress-osm-extract-read = lecture de l'extrait OSM { $region } du { $date }
progress-osm-candidates = { $count } objets candidats OSM

## Permit notes

progress-permits-fetch = téléchargement des données { $source } pour les notes d'autorisation
progress-permits-osm = lecture de l'extrait OSM { $region } du { $date } pour les notes d'autorisation
progress-permits-stored = lecture des données enregistrées des notes d'autorisation
warning-permits-source-unreachable = { $source } n'a pas pu être lue, les notes d'autorisation l'omettent donc : { $error }
warning-permits-osm-no-extract = aucun extrait OSM n'est en cache pour cette zone de carte, les notes d'autorisation omettent donc OSM ; `freemapper fetch` le télécharge

## ALKIS

warning-alkis-coverage-unknown = couverture des bâtiments { $cadastre } inconnue ; lancez `freemapper fetch` en ligne pour l'enregistrer
progress-alkis-buildings-download = téléchargement des bâtiments { $cadastre }
progress-alkis-buildings = { $count } bâtiments { $cadastre }
warning-alkis-no-buildings = aucun bâtiment { $cadastre } trouvé dans la zone de la carte
progress-alkis-skipped = { $count } objets { $cadastre } ignorés : { $list }
warning-alkis-buildings-changed = les bâtiments { $cadastre } ont changé chez le fournisseur ({ $changes })
progress-alkis-no-land-use-stored = aucune occupation du sol { $cadastre } enregistrée ; lancez `freemapper fetch` en ligne pour les candidats de zones interdites
progress-alkis-land-use-download = téléchargement de l'occupation du sol { $cadastre }
progress-alkis-land-use-parts = { $count } parties d'occupation du sol { $cadastre }
warning-alkis-land-use-kept = occupation du sol { $cadastre } indisponible ({ $error }) ; l'occupation du sol enregistrée est conservée
warning-alkis-land-use-removed = occupation du sol { $cadastre } indisponible ({ $error }) ; l'occupation du sol enregistrée ne couvre plus la zone de la carte, freemapper l'a donc supprimée
warning-alkis-land-use-none = occupation du sol { $cadastre } indisponible ({ $error }) ; pas de candidats de zones interdites
progress-bdot-filled = { $count } objets BDOT10k ({ $what }) ajoutés là où EGiB ne donne rien ({ $powiat })
warning-bdot-failed = BDOT10k n'a pas pu combler le vide d'EGiB ({ $what }) : { $error }
warning-bdot-instead = EGiB est indisponible ({ $error }) ; BDOT10k est utilisé à la place ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = aucune structure ALKIS de Berlin enregistrée ; lancez `freemapper fetch` en ligne pour les escaliers, murs, arbres et auvents
progress-berlin-removing = suppression des couches de Berlin enregistrées : la zone de la carte n'est plus près de Berlin
progress-berlin-download = téléchargement des structures ALKIS de Berlin, du cadastre des arbres, du relevé des rues et des couches de points
progress-berlin-features = { $count } objets { $file } de Berlin
warning-berlin-layer-kept = { $file } de Berlin indisponible ({ $detail }) ; la couche enregistrée est conservée
warning-berlin-layer-skipped = { $file } de Berlin indisponible ({ $detail }) ; couche ignorée
progress-feinkartierung-none-stored = aucune Feinkartierung de Hambourg enregistrée ; lancez `freemapper fetch` en ligne pour les garde-corps, murs, escaliers et mobilier urbain
progress-feinkartierung-removing = suppression de la Feinkartierung de Hambourg enregistrée : la zone de la carte n'est plus à Hambourg
progress-feinkartierung-download = téléchargement de la Feinkartierung de Hambourg (espace des rues et espaces verts publics)
progress-feinkartierung-features = { $count } objets { $file } de la Feinkartierung
warning-feinkartierung-layer-kept = { $file } de la Feinkartierung indisponible ({ $detail }) ; la couche enregistrée est conservée
warning-feinkartierung-layer-skipped = { $file } de la Feinkartierung indisponible ({ $detail }) ; couche ignorée
error-extra-layer-unknown = pas une couche de { $folder }
warning-berlin-buildings-missing = aucun bâtiment ALKIS enregistré pour cette zone à Berlin ; lancez `freemapper fetch` pour les obtenir du Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = aucune structure ALKIS complète enregistrée ; lancez `freemapper fetch` en ligne pour les murs, clôtures, haies et ponts
progress-nas-removing = suppression des couches ALKIS complètes enregistrées : la zone de la carte n'est plus dans un Land avec un service NAS
warning-nas-folder-outside = couches ALKIS complètes ignorées ({ $folder } sort du dossier du projet)
progress-nas-download = téléchargement des structures ALKIS complètes (NAS)
progress-nas-features = { $count } objets ALKIS NAS { $file }
progress-nas-skipped = { $count } objets ALKIS NAS { $file } ignorés : { $list }
warning-nas-layer-kept = ALKIS NAS { $file } indisponible ({ $detail }) ; la couche enregistrée est conservée
warning-nas-layer-skipped = ALKIS NAS { $file } indisponible ({ $detail }) ; couche ignorée

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = aucun objet de l'ATKIS Basis-DLM enregistré ; lancez `freemapper fetch` en ligne pour les sources, grottes, tours, monuments et fontaines
progress-dlm-removing = suppression des couches enregistrées de l'ATKIS Basis-DLM : la zone de la carte n'est plus dans un Land qui le fournit
warning-dlm-folder-outside = couches de l'ATKIS Basis-DLM ignorées ({ $folder } sort du dossier du projet)
progress-dlm-download = téléchargement des objets de l'ATKIS Basis-DLM
progress-dlm-features = { $count } objets ATKIS Basis-DLM { $file }
progress-dlm-skipped = { $count } objets ATKIS Basis-DLM { $file } ignorés : { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } indisponible ({ $detail }) ; la couche enregistrée est conservée
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } indisponible ({ $detail }) ; couche ignorée

## Tree cadastres

progress-trees-none-stored = aucun cadastre des arbres d'une ville enregistré ; lancez `freemapper fetch` en ligne pour les arbres isolés
progress-trees-removing = suppression des cadastres d'arbres enregistrés : la zone de la carte n'est plus dans une ville qui en a un
warning-trees-folder-outside = cadastres d'arbres de villes ignorés ({ $folder } sort du dossier du projet)
progress-trees-download = téléchargement des cadastres d'arbres des villes
progress-trees-count = { $count } arbres { $id }
progress-trees-skipped = { $count } arbres { $id } ignorés : { $list }
warning-trees-kept = cadastre des arbres { $id } indisponible ({ $detail }) ; le cadastre enregistré est conservé
warning-trees-skipped = cadastre des arbres { $id } indisponible ({ $detail }) ; arbres ignorés

## Deriving the templates

progress-writing = écriture de { $path }
progress-contours-steep = courbes de niveau tous les { $interval } m : le terrain est si raide que des courbes tous les { $default } m se confondraient ; une équidistance de { $default } m dans le projet les conserve (`--contour-interval { $default }`)
warning-osm-only-no-terrain = pas encore de données de terrain pour cette région : la carte est faite à partir d'OpenStreetMap seul, sans courbes de niveau, modèles ni bâtiments
progress-no-ndsm-recorded = aucun nDSM enregistré ; lancez `freemapper fetch` pour le modèle nDSM
warning-ndsm-tile-invalid = tuile nDSM { $tile } non valide dans freemapper.toml ; ignorée
warning-ndsm-not-cached = nDSM { $tile } pas en cache ; lancez `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } illisible ({ $error }) ; supprimez { $path } et lancez `freemapper fetch`
warning-surface-unusable = modèle de surface inutilisable ({ $error }) ; lancez `freemapper fetch`

## Downloads and the cache

progress-downloading = téléchargement de { $url }
progress-retry-wait = le service est occupé ; nouvel essai dans { $seconds } s
progress-downloaded = { $megabytes } Mo téléchargés
progress-downloaded-of = { $megabytes } Mo téléchargés sur { $total } Mo
progress-tile-index-download = téléchargement de l'index des tuiles { $product }
progress-tile-cached = { $tile } mise en cache
progress-surface-blocks = lecture de { $count } des { $total } blocs de la tuile { $tile } du modèle de surface ({ $megabytes } Mo)
progress-surface-whole = la tuile { $tile } du modèle de surface est téléchargée en entier : { $reason }
warning-tile-cached-older = utilisation de la tuile { $product } { $tile } en cache du { $published } (la plus récente du { $newer } n'est pas téléchargée)
warning-ndsm-no-cached = aucun nDSM en cache pour la tuile { $tile }
warning-ndsm-tile-unavailable-cached = tuile nDSM { $tile } indisponible ({ $reason }) ; le fichier en cache est utilisé
warning-ndsm-tile-unavailable-skipped = tuile nDSM { $tile } indisponible ({ $reason }) ; ignorée
warning-file-check-failed = impossible de vérifier si { $file } est toujours à jour ({ $reason }) ; le fichier en cache est utilisé, ainsi que les autres fichiers en cache de ce serveur sans demander
warning-ldbv-no-buildings-file = pas de fichier de bâtiments pour la tuile { $tile } sur le serveur ; la carte peut manquer de bâtiments là-bas
progress-ndsm-not-published = aucun nuage de points nDSM publié pour la tuile { $tile }
progress-ndsm-cached = nDSM { $tile } mis en cache
progress-point-cloud-not-published = aucun nuage de points laser publié pour la tuile { $tile }
progress-point-cloud-cached = nuage de points { $tile } mis en cache
warning-point-cloud-no-cached = aucun nuage de points en cache pour la tuile { $tile } ; pas de candidats de végétation là-bas
warning-point-cloud-unavailable-cached = nuage de points { $tile } indisponible ({ $reason }) ; le fichier en cache est utilisé
warning-point-cloud-unavailable-skipped = nuage de points { $tile } indisponible ({ $reason }) ; pas de candidats de végétation là-bas
warning-point-clouds-unavailable = les nuages de points laser sont indisponibles ({ $reason }) ; les candidats de végétation utilisent les tuiles en cache, s'il y en a
progress-no-point-clouds = pas encore de nuages de points laser pour la végétation ici : le Land n'en publie aucun que freemapper puisse lire, ou pas dans le système de coordonnées de la carte
warning-point-clouds-not-fetched = la végétation est choisie, mais aucun nuage de points n'est chargé ; lancez `freemapper fetch` pour les charger
warning-point-cloud-unreadable = nuage de points { $tile } illisible ({ $error }) ; supprimez { $path } et lancez `freemapper fetch`
warning-osm-nrw-nearest = aucun extrait d'arrondissement de Rhénanie-du-Nord-Westphalie à moins de 100 m de la zone ; lecture du plus proche, { $district }
warning-osm-disk-full-cached = le disque est plein, freemapper utilise donc l'extrait OSM en cache du { $date }. Libérez de l'espace et relancez `freemapper fetch` pour en obtenir un plus récent
warning-osm-geofabrik-unavailable = Geofabrik indisponible ({ $error }) ; utilisation de l'extrait OSM en cache du { $date }
progress-osm-extract-cached = extrait OSM { $region } du { $date } mis en cache
progress-osm-extract-download = téléchargement de { $url } (environ { $megabytes } Mo)

## Schleswig-Holstein

progress-sh-bdom-download = téléchargement de la tuile bDOM { $tile } ({ $megabytes } Mo, { $index } sur { $total })
progress-sh-dgm-not-in-atom = la tuile DGM1 { $tile } n'est pas dans le répertoire INSPIRE ATOM ; téléchargement de son XYZ ({ $megabytes } Mo)
progress-sh-dgm-atom-unreadable = la tuile DGM1 { $tile } du répertoire INSPIRE ATOM ne se lit pas ({ $error }) ; téléchargement de son XYZ ({ $megabytes } Mo)
progress-sh-tile = tuile { $tile } : { $text }
progress-sh-download-again = tuile { $tile } : le téléchargement ne se lit pas ({ $message }) ; nouveau téléchargement
warning-sh-portal-no-date = le Downloadportal ne donne aucune date pour la tuile { $tile } du { $model }
warning-sh-dgm-xyz-used = la tuile DGM1 { $tile } n'est pas dans le répertoire INSPIRE ATOM ; son XYZ du Downloadportal a été utilisé
warning-sh-dgm-xyz-used-unreadable = la tuile DGM1 { $tile } du répertoire INSPIRE ATOM ne se lit pas ({ $error }) ; son XYZ du Downloadportal a été utilisé

## Tile lists and dates of the height models

warning-model-tile-cells-empty = tuile { $tile } du { $model } : { $count } cellules de la zone de la carte n'ont pas d'altitude
warning-centre-tile-missing = la tuile { $tile } du { $model } au centre de la zone de la carte est absente, il n'y a donc pas de date
warning-centre-tile-unlisted = la liste des tuiles ne nomme aucune tuile { $tile } du { $model } au centre de la zone de la carte, il n'y a donc pas de date
warning-list-unreadable-kept = la liste des tuiles du { $model } n'a pas pu être lue ({ $error }) ; celle conservée d'une exécution précédente a été utilisée
warning-portal-list-unreadable-kept = la liste des tuiles du Downloadportal pour le { $model } n'a pas pu être lue ({ $error }) ; celle conservée d'une exécution précédente a été utilisée
warning-surface-much-older = le { $model } date de { $surface }, l'orthophotographie de { $flight }. Le nDSM peut manquer de bâtiments et d'arbres plus récents.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objets de la liste des tuiles n'ont pas de clé de tuile utilisable ; ignorés
warning-rp-list-unreadable-kept = la liste des tuiles du LVermGeo RP n'a pas pu être lue ({ $error }) ; celle conservée d'une exécution précédente a été utilisée
warning-rp-metalink-not-kept = le Metalink du LVermGeo RP n'a pas été conservé : { $error }
warning-rp-metalink-unreadable-kept = le Metalink du LVermGeo RP pour le { $model } n'a pas pu être lu ({ $detail }) ; celui conservé d'une exécution précédente a été utilisé
warning-rp-metalink-unreadable = le Metalink du LVermGeo RP pour le { $model } n'a pas pu être lu ({ $detail }), les tuiles pour lesquelles sa liste des tuiles ne donne pas d'année de vol restent donc sans altitudes
warning-rp-tile-without-name = tuile { $tile } du { $model } : ce carré de 1 km n'a pas d'altitudes dans la carte. La liste des tuiles du LVermGeo RP (Laserbefliegung) { $note } pour lui, le nom du fichier n'est donc pas connu{ $either ->
        [metalink] , et son Metalink ne nomme pas non plus de fichier pour lui
       *[none] {""}
    }
warning-rp-centre-no-date = la tuile { $tile } du { $model } n'a pas de date : la liste des tuiles du LVermGeo RP (Laserbefliegung) { $note } pour elle
word-rp-list-neither = ne donne ni année de vol ni jours de vol
word-rp-list-no-days = ne donne pas de jours de vol
word-rp-list-no-year = ne donne pas d'année de vol
word-rp-list-days-unreadable = donne des jours de vol { $days } illisibles
word-rp-list-disagree = donne des jours de vol { $days } et une année de vol { $year } qui ne s'accordent pas sur une seule année

## Hesse

warning-hvbg-undated = la liste des métadonnées n'a pas de Bildflugdatum lisible pour la tuile { $tiles } ; les enregistrements de leur projet n'ont pas de date de vol
warning-hvbg-undated-more = la liste des métadonnées n'a pas de Bildflugdatum lisible pour la tuile { $tiles } et { $more } de plus ; les enregistrements de leur projet n'ont pas de date de vol
warning-hvbg-list-unreadable-older = la nouvelle liste de métadonnées HVBG { $name } n'a pas pu être lue ({ $detail }) ; utilisation de l'ancienne liste { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model } : pas de date, le STAC ne liste aucune tuile au centre de la zone de la carte
warning-lgln-no-date = { $model } : pas de date ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error } ; le modèle enregistré garde sa date notée
warning-mv-cells-empty = { $model } : { $count } cellules dans les tuiles du Mecklembourg-Poméranie-Occidentale n'ont pas d'altitude
warning-mv-centre-no-date = { $model } : pas de date pour la tuile centrale { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } lignes de la liste des tuiles du { $model } n'ont pas de nom de tuile utilisable ; ignorées
warning-nrw-list-rows-undated = { $count } lignes de la liste des tuiles du { $model } n'ont pas de date lisible
warning-nrw-list-older = la liste des tuiles du serveur pour le { $model } date du { $server }, plus ancienne que celle conservée du { $kept } ; celle conservée a été utilisée
warning-nrw-list-no-date = la liste des tuiles ne donne aucune date pour la tuile { $tile } du { $model }

## After `init`

note-scale-outside-norm = note : 1:5000 ne fait pas partie de la norme sprint ; freemapper-print le signale et les symboles restent à 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } n'est pas un identifiant OSM comme way/992423119
error-place-invalid-answer = réponse non valide de la recherche de lieux : { $detail }
error-place-not-found = { $id } introuvable. Nominatim ne liste que des objets OSM avec étiquettes, un chemin extérieur sans étiquette d'un multipolygone n'y figure donc pas. Utilisez plutôt la relation multipolygone (--osm relation/<id>), ou enregistrez le contour en GeoJSON et utilisez --geojson
error-place-is-a-point = { $id } est un point, pas une zone ; choisissez son terrain (un chemin ou une relation)
error-place-no-such-part = { $id } a { $count ->
        [one] 1 partie
        [many] { $count } de parties
       *[other] { $count } parties
    } ; il n'y a pas de partie { $part }
error-place-several-parts = { $id } est une zone de { $count } parties séparées ({ $sizes }). Une zone de carte est un seul polygone : choisissez une partie avec --part <n> (1 est la plus grande), ou enregistrez le contour voulu en GeoJSON et utilisez --geojson
error-layer-not-a-world-file = { $path } : pas un fichier de géoréférencement
error-layer-file = { $path } : { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = pas d'aperçu pour { $layer } ; couches avec aperçu : { $names }
error-layer-no-ndsm-published = aucun nDSM n'est publié pour les tuiles de cette zone de carte, il n'y a donc pas de modèle ndsm
error-layer-not-made = { $layer } n'est pas encore créée ; lancez { $step ->
        [fetch] freemapper fetch ou run (MCP : start_pipeline) ; Berlin seulement
        [export] freemapper export ou run (MCP : start_pipeline)
       *[derive] freemapper derive ou run (MCP : start_pipeline)
    }
warning-ocad-no-background = { $error } ; la carte OCAD n'en reçoit aucun fond
warning-map-preview-failed = la carte est écrite, mais son aperçu manque : { $error }
warning-training-no-tree-cover = la carte d'entraînement garde entiers les espaces ouverts des parcs : la couverture arborée (data/tree-cover.tif) est absente ; relancez `freemapper derive`
