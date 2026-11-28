# Textes du programme freemapper, français (ADR 0092). L'anglais est la référence : toutes les autres
# langues ont les mêmes identifiants de message et les mêmes variables. Les textes d'autres programmes
# (erreurs du système d'exploitation, messages d'analyse) arrivent dans des variables et restent tels quels.

## L'option --lang

cli-lang-unknown = { $value } n'est pas une langue dont freemapper a les textes ; disponibles : { $available }

## init

cli-part-needs-osm = --part choisit une partie d'une zone OSM et exige --osm ; avec --place, cherchez d'abord l'identifiant, puis utilisez --osm <id> --part <n>
cli-reading-file = lecture de { $path }
cli-no-area-for-place = aucune zone trouvée pour « { $query } ». Si le lieu est une zone de plusieurs parties séparées, choisissez-en une avec --osm <id> --part <n>, ou utilisez --geojson
cli-using-place = Utilisation de { $name } ({ $id })
cli-created-project = Projet créé : { $path }

## Cache

cli-no-cache-dir = aucun dossier de cache sur ce système ; indiquez --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] { $count } tuile téléchargée
        [many] { $count } de tuiles téléchargées
       *[other] { $count } tuiles téléchargées
    }
cli-wrote = { $path } écrit
cli-kept-map = { $path } conservé (il peut contenir votre travail)
cli-map-lacks-templates = La carte ne référence pas encore les modèles d'image. Ajoutez-les dans OOM (Modèles > Ouvrir un modèle…) ou remplacez la carte avec --force, qui conserve l'ancienne sous le nom .omap.bak.
cli-map-lacks-buildings = La carte ne contient pas encore les bâtiments ALKIS. Pour les ajouter, remplacez-la avec --force, qui conserve l'ancienne sous le nom .omap.bak.
cli-map-lacks-canopies = La carte n'a pas les auvents et abris à voitures ALKIS (522) qui sont désormais des objets de la carte. Pour les ajouter, lancez avec --force, qui conserve l'ancienne carte sous le nom .omap.bak.
cli-map-light = La carte de base : { $joined } lignes réunies, { $left } objets cachés ou trop petits laissés aux couches de candidats (rien déplacé)
cli-map-light-kept = Les couches de candidats contiennent maintenant aussi { $left } objets ou tronçons que cette version retire de la carte de base (marqués freemapper:generalized). La carte conservée les a encore, ils apparaissent donc deux fois. Remplacez la carte avec --force pour la nettoyer (l'ancienne reste sous le nom .omap.bak).
cli-map-old-candidates = La carte nomme encore templates/candidates.omap ; cette exécution l'a rempli de tous les candidats. Les candidats existent désormais aussi en sept couches, un fichier chacune (templates/candidates-*.omap). La carte les liste une fois remplacée avec --force (l'ancienne reste sous le nom .omap.bak), ou ajoutez-les vous-même dans OOM (Modèles > Ouvrir un modèle…).
cli-paper-frame-kept-map = Les cadres papier ont été écrits dans le modèle templates/paper-frame.omap. La carte existante n'est pas modifiée et ne le référence donc pas : ouvrez-le dans OpenOrienteering Mapper (Modèles > Ouvrir un modèle…) ou remplacez la carte avec --force (l'ancienne reste sous le nom .omap.bak).

## PDF

cli-pdf-without-credits = PDF sans mention des sources : l'ODbL et dl-de/by-2-0 demandent de nommer les sources quand la carte est transmise ; voir ATTRIBUTION.txt
cli-wrote-pdf = { $path } écrit (la carte sans candidats)
cli-wrote-pdf-candidates = { $path } écrit (l'impression de contrôle : la carte avec tous les candidats, à leur position exacte ; à vérifier sur le terrain)
cli-wrote-training = { $map } et { $pdf } écrits (la carte d'entraînement : { $joined } réunis, { $shifted } décalés, { $left } omis ou coupés)
cli-training-free-ends = Extrémités de lignes laissées libres : { $ways } où un chemin en rencontrait un autre, { $others } sur d'autres lignes
cli-training-apart-left = la carte d'entraînement a encore { $length } mm (sur le papier) où les objets gardent trop peu d'écart ; vérifiez-la dans OOM
cli-kept-training = Carte d'entraînement { $path } conservée, pour qu'une exécution n'écrase pas une carte imprimée ou transmise ; supprimez-la ou passez --force pour la renouveler

## OCAD

cli-ocad-kept = { $path } conservé (il peut contenir votre travail ; --force le remplace et conserve l'ancien sous le nom .ocd.bak)
cli-ocad-old-candidates = La carte OCAD nomme encore { $path } comme carte de fond ; cette exécution l'a remplie de tous les candidats. Les candidats existent désormais aussi en sept couches, un fichier chacune (-candidates-*.ocd). La carte OCAD les liste une fois remplacée avec --force (l'ancienne reste sous le nom .ocd.bak), ou ajoutez-les vous-même dans OCAD comme cartes de fond.
cli-ocad-wrote = { $count ->
        [one] { $path } écrit (1 objet)
        [many] { $path } écrit ({ $count } d'objets)
       *[other] { $path } écrit ({ $count } objets)
    }
cli-ocad-left-out = Omis de la carte OCAD : { $skipped } objets (pas de symbole pour OCAD, pas de surface, ou hors de la zone de dessin d'OCAD de ±2 m) et { $dropped } parties de lignes ou trous dégénérés.
cli-ocad-load-symbols = Dans OCAD, choisissez Symbole > Charger les couleurs et symboles de… et sélectionnez le jeu de symboles officiel{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , le jeu scolaire IOF 2019
       *[other] {" "}ISSprOM 2019
    } ; il remplace par numéro les symboles provisoires de freemapper.

## select

cli-select-running = Sélection de la zone pour { $path } : cliquez sur « Terminé » dans la page ou Ctrl+C pour arrêter.
cli-select-no-browser = Impossible d'ouvrir le navigateur ({ $error }) ; ouvrez l'adresse ci-dessus.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }) : { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind } ; pas de zone, ne peut pas être une zone de carte) : { $place }
cli-find-hint = Créez le projet avec : freemapper init <dir> --osm { $id }
cli-find-no-area = Aucun de ces lieux n'a de zone pouvant servir de zone de carte. Dessinez la zone avec : freemapper select <dir>
cli-find-nothing = aucun lieu trouvé pour « { $query } » ; essayez un autre nom, ou dessinez la zone avec : freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = avertissement : l'adaptateur dans { $folder } est ignoré : { $finding }
cli-adapter-note = avertissement : l'adaptateur dans { $folder } se charge, mais : { $finding }
cli-adapter-from-project = avertissement : l'adaptateur { $adapter } du dossier du projet est utilisé ; il peut interroger ces hôtes : { $hosts }
cli-adapter-overlaps = avertissement : l'adaptateur { $adapter } couvre aussi le territoire de { $others } ; ses objets vont sur les cartes de ces lieux
cli-adapter-dir-relative = avertissement : FREEMAPPER_ADAPTER_DIR n'est pas un chemin absolu ({ $path }) ; il est ignoré
cli-adapters-no-project = avertissement : { $path } n'est pas un dossier ; aucun adaptateur de projet n'a été lu
cli-adapters-export-unknown = l'adaptateur { $adapter } n'est décrit par aucun manifeste de ce programme, il n'y a donc rien à exporter ; `freemapper adapters list` montre les identifiants
cli-adapters-export-taken = { $path } existe et n'est pas vide ; exportez dans un dossier qui n'existe pas encore
cli-adapters-export-done = Les { $count } fichiers de l'adaptateur { $adapter } ont été écrits dans { $path }. Ils ne contiennent que ce que décrit son manifeste ; les adresses inscrites dans le code du programme n'y sont pas.
cli-adapters-export-next = Modifiez les adresses dans manifest.toml, vérifiez le dossier avec `freemapper adapters check`, puis déplacez-le dans { $path } : dès le prochain démarrage, il remplace l'adaptateur intégré, avec un avertissement. Supprimer le dossier rétablit l'adaptateur intégré.
cli-adapters-overlaps = partage le territoire avec : { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  révision { $revision }  vérifié { $checked }  hôtes : { $hosts }
cli-adapters-source-built-in = intégré
cli-adapters-source-user = de votre dossier
cli-adapters-source-project = du projet
cli-adapters-never-checked = jamais
cli-adapters-user-dir = Vos adaptateurs sont lus dans { $path }
cli-adapters-no-user-dir = Ce système n'a pas de dossier pour vos adaptateurs ; FREEMAPPER_ADAPTER_DIR en indique un

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper peut maintenant aussi interroger { $host } pour { $name }. L'entrée se trouve dans { $file } ; `freemapper adapters disallow-host { $adapter } { $host }` la retire. Une page de sélection ou un assistant IA en cours d'exécution lit l'entrée au prochain démarrage
cli-allow-host-already = { $host } est déjà autorisé pour { $name }
cli-allow-host-declared = l'adaptateur de { $name } nomme déjà { $host } lui-même ; rien à autoriser
cli-allow-host-refused = { $host } ne peut pas être autorisé : { $reason }
cli-allow-host-unknown-adapter = il n'y a pas d'adaptateur intégré { $adapter } ; les adaptateurs intégrés sont : { $adapters }
cli-allow-host-folder-adapter = { $adapter } est un adaptateur de votre dossier d'adaptateurs ; ajoutez plutôt l'hôte à `[net] hosts` de son manifeste
cli-allow-host-no-config = ce système n'a pas de dossier de configuration pour conserver les hôtes autorisés
cli-allow-host-write = { $file } n'a pas pu être écrit : { $detail }
cli-allow-host-unreadable = { $file } est illisible ({ $detail }) ; corrigez ou supprimez d'abord le fichier ; rien n'a été modifié
cli-disallow-host-removed = freemapper n'interroge plus { $host } pour { $name }
cli-disallow-host-absent = { $host } n'était pas autorisé pour { $name } ; rien à retirer
cli-allowed-hosts-unreadable = avertissement : { $file } n'a pas pu être lu ({ $detail }) ; aucun hôte supplémentaire n'est autorisé
cli-allowed-hosts-ignored = avertissement : { $file } : l'entrée { $host } pour { $adapter } est ignorée : { $reason }
cli-allowed-hosts-no-adapter = freemapper n'a pas d'adaptateur intégré de ce nom
cli-allowed-host-redundant = avertissement : { $host } fait désormais partie des hôtes que nomme l'adaptateur de { $name } ; votre entrée dans { $file } a été supprimée
cli-adapters-allowed = autorisé par vous : { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adaptateur { $adapter } ({ $name }), révision { $revision }, hôtes : { $hosts }
cli-check-error = erreur : { $finding }
cli-check-warning = avertissement : { $finding }
cli-check-passed = { $folder } réussit la vérification
cli-check-passed-warnings = { $folder } réussit la vérification, avec { $count ->
        [one] un avertissement
        [many] { $count } d'avertissements
       *[other] { $count } avertissements
    }
cli-check-failed = { $folder } échoue à la vérification
cli-check-no-folder = il n'y a pas de dossier { $folder }
cli-check-live-ok = { $capability } ok : { $detail }
cli-check-live-failed = { $capability } ÉCHEC : { $detail }
cli-check-live-not-asked = { $capability } non interrogé : { $detail }
cli-check-live-box-outside = le cadre ne se trouve pas dans la limite de l'adaptateur { $adapter } ; une vérification en direct interroge les services pour un lieu que l'adaptateur couvre
cli-check-live-cannot = la vérification en direct ne peut pas s'exécuter : { $detail }
cli-check-detail-not-declared = l'adaptateur n'en déclare aucun
cli-check-detail-stopped = une partie précédente a échoué
cli-check-detail-see-warnings = voir les avertissements
cli-check-detail-outside-dataset = le cadre se trouve hors du jeu de données
cli-check-detail-objects = { $count ->
        [one] un objet
        [many] { $count } d'objets
       *[other] { $count } objets
    }
cli-check-detail-layers = { $layers } couches, { $count } objets
cli-check-detail-trees = { $count ->
        [one] un arbre
        [many] { $count } d'arbres
       *[other] { $count } arbres
    }
cli-check-detail-extract = { $label }, données du { $date }
cli-check-raster-orthophoto = orthophotographie
cli-check-raster-terrain = modèle numérique de terrain
cli-check-raster-surface = modèle numérique de surface
cli-check-raster-tiles = { $product } : { $count ->
        [one] une tuile
        [many] { $count } de tuiles
       *[other] { $count } tuiles
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } et { $last }

## Folder lock

cli-lock-held = une autre exécution de freemapper utilise ce dossier (depuis { $time }, lancée par { $who }). Attendez qu'elle se termine ou arrêtez-la d'abord.
cli-lock-who-cli = le terminal
cli-lock-who-mcp = un agent IA (MCP)
cli-lock-who-page = la page de sélection
cli-lock-who-unknown = un autre programme
cli-lock-failed = impossible de verrouiller le dossier du projet pour cette exécution : { $error }
cli-lock-stuck = autre chose qu'un fichier de verrou bloque le passage et n'a pas pu être supprimé

## Run routes of the selection page

cli-run-bad-request = Les options d'exécution ne sont pas valides : { $error }
cli-run-no-project = Aucun projet enregistré dans ce dossier pour l'instant. Enregistrez d'abord la zone.
cli-run-already-running = Une exécution ({ $job }) est déjà en cours pour ce projet. Attendez qu'elle se termine ou annulez-la d'abord.
cli-run-disk-stops = Pas assez d'espace disque libre pour le téléchargement : { $messages }
cli-run-busy-elsewhere = Un autre projet de ce programme exécute un pipeline en ce moment. Les projets partagent le cache de téléchargement, donc une seule exécution à la fois. Réessayez quand elle sera terminée.
cli-run-nothing-to-cancel = Aucune exécution en cours, rien à annuler.
cli-run-unknown-job = Il n'y a pas d'exécution { $job } pour ce projet.
cli-status-too-fast = Les demandes d'état sont trop fréquentes. Demandez au plus cinq fois par seconde.
cli-open-folder-failed = Impossible d'ouvrir le dossier ({ $error }).
cli-open-folder-no-opener = Cet ordinateur n'a pas de programme pour ouvrir des dossiers ({ $program }). Ouvrez le dossier à la main.
cli-open-folder-missing = Le dossier du projet n'existe pas encore. Enregistrez d'abord la zone.
cli-open-folder-refused = Le nom du dossier n'est pas sûr à transmettre au gestionnaire de fichiers.
cli-save-running = Une exécution est en cours pour ce projet. Enregistrez à nouveau quand elle sera terminée.
cli-save-unknown-kind = Type de carte inconnu « { $kind } ». Utilisez schoolyard, sprint ou forest.
cli-save-scale-too-large = L'échelle { $scale } est trop grande.
cli-save-bad-content = Les réglages du contenu ne sont pas valides. Rien n'a été enregistré.
cli-save-bad-paper-frames = Les cadres papier ne sont pas valides. Rien n'a été enregistré.
cli-save-paper-frames-no-project = Ce dossier n'a pas encore de projet. Enregistre d'abord une zone de carte, puis les cadres papier.
cli-map-image-none = Créez d'abord la carte.
cli-map-image-gone = Cet aperçu n'est plus disponible.
cli-map-image-running = Une exécution est en cours pour ce projet. L'aperçu pourra être calculé une fois qu'elle sera terminée.
cli-map-image-stale = Le projet enregistré ne correspond plus à la carte générée, par exemple parce que la zone, le type de carte ou l'échelle ont changé. Lancez d'abord une exécution qui remplace la carte (« Remplacer la carte existante » dans la page, ou freemapper run --force).
cli-map-image-superseded = Un aperçu plus récent a été demandé entre-temps, celui-ci n'a donc pas été calculé.
cli-map-image-bad-content = Les réglages du contenu ne sont pas valides.
cli-map-image-bad-request = La demande d'aperçu n'est pas valide. Envoyez un objet JSON, avec les réglages du contenu le cas échéant.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – sélection de la zone
page-heading = Choisis la zone de la carte
page-update-available = freemapper { $version } est disponible. Mets à jour avec
page-update-notes = Notes de version
page-folder = Dossier du projet
page-pick = Chercher un lieu
page-query-placeholder = p. ex. Clara-Zetkin-Schule Strausberg
page-search-button = Chercher
page-searching = Recherche …
page-no-results = Rien trouvé.
page-point-only = un point seulement, pas une zone
page-draw = Dessine-la toi-même
page-rectangle = Rectangle
page-polygon = Polygone
page-finish-polygon = Fermer le polygone
page-clear = Effacer
page-hint-rectangle = Clique sur deux coins opposés de la carte.
page-hint-polygon = Clique sur les coins l'un après l'autre. Pour fermer, clique sur le premier coin, double-clique ou clique sur « Fermer le polygone ». Échap annule, Retour arrière supprime le dernier coin.
page-hint-idle = Choisis un lieu dans la recherche ou dessine la zone toi-même.
page-summary = Zone
page-source-osm = D'après OpenStreetMap : { $name }
page-source-drawn = Dessinée à la main
page-size = Taille avec marge : { $width } × { $height } m
page-tiles-one = { $n } tuile : { $names }
page-tiles-other = { $n } tuiles : { $names }
page-download = Téléchargement : environ { $mb } Mo
page-download-none = Téléchargement : tout est déjà en cache
page-download-details = Ce qui est téléchargé
page-cached = en cache
page-disk = Espace disque : { $volumes }
page-disk-volume = { $free } libres, environ { $needed } nécessaires
page-cache-volume = cache
page-project-volume = dossier du projet
page-no-room = ne tient pas sur le disque
page-no-room-skip = Tout ne tient pas sur le disque (freemapper garde { $reserve } libres). Le téléchargement omet les parties marquées. Libère d'abord de l'espace si tu en as besoin :
page-no-room-stop = Pas assez d'espace disque (freemapper garde { $reserve } libres). Sans les parties marquées, il n'y a pas de carte ; le téléchargement s'arrête donc avant de rien télécharger. Libère d'abord de l'espace :
page-disk-unknown = freemapper ne peut pas lire l'espace disque libre ({ $error }). Il télécharge sans vérification.
page-too-large = La zone est trop grande : { $max } m au plus par côté (marge comprise). Choisis-en une plus petite.
page-outside = La zone se trouve hors des Länder pris en charge ({ $states }) ; freemapper n'a pas encore de sources de données pour d'autres régions.
page-outside-unknown = La zone se trouve hors des régions prises en charge par freemapper ; il n'y a pas encore de sources de données là-bas.
page-adapters-failed = Impossible de lister les photos aériennes ({ $detail }). La carte n'affiche qu'OpenStreetMap ; tout le reste fonctionne.
page-settings = Carte
page-kind = Type de carte
page-schoolyard = Carte de cour d'école (1:1000)
page-sprint = Carte de sprint (1:4000)
page-forest = Carte de forêt (1:15000)
page-scale = Échelle 1:
page-paper-format = Cadre papier
page-paper-none = aucun
page-paper-orientation = Orientation
page-paper-landscape = paysage
page-paper-portrait = portrait
page-paper-name-landscape = { $format } paysage
page-paper-name-portrait = { $format } portrait
page-paper-keep = Garder le cadre
page-paper-delete = Supprimer
page-paper-help = Le cadre t'aide seulement à choisir l'échelle et la zone ; fais-le glisser avec la souris. La prochaine exécution écrit les cadres gardés dans le modèle templates/paper-frame.omap. Une nouvelle carte le référence ; une carte existante n'est pas modifiée, ouvre alors le modèle dans OpenOrienteering Mapper (Modèles > Ouvrir un modèle…).
page-paper-saved = Cadres papier enregistrés. La prochaine exécution les écrit dans le modèle templates/paper-frame.omap. Une nouvelle carte le référence ; pour une carte existante, ouvre-le dans OpenOrienteering Mapper (Modèles > Ouvrir un modèle…).
page-map-title = Titre
page-contour-interval = Équidistance
page-contour-auto = automatique
page-steep-slope = Seuil de pente
page-steep-slope-auto = automatique (27°)
page-steep-slope-hint = À partir de cet angle de pente, le terrain est considéré comme raide lors de la recherche des talus et des falaises. 27° (automatique) convient aux collines, aux parcs et aux villes. Dans les villes de rochers, les gorges et les carrières, presque chaque pente est raide : talus et falaises se fondent alors en une seule tache et aucun n'est trouvé ; de 55° à 65°, ils sont séparés. Une valeur trop élevée ne garde que les parois les plus raides.
page-face-slope = Seuil de paroi
page-face-slope-auto = automatique (70°)
page-face-slope-hint = À partir de cet angle de pente, le terrain est dessiné en surface noire de paroi rocheuse. 70° (automatique) convient aux villes de rochers comme Adršpach (5,2 % de la surface). 65° donne plus de surfaces noires et plus grandes, pentes raides comprises (8,4 %). 75° ne garde que les vraies parois, aux bords effilochés (2,5 %). Une valeur trop haute ne laisse presque aucune surface.
page-settings-fixed = Le projet existe déjà ; le type de carte, l'échelle et le titre se trouvent dans freemapper.toml. L'équidistance peut encore être modifiée.
page-save = Enregistrer
page-saving = Enregistrement …
page-saved = Enregistré dans freemapper.toml.
page-zone-changed = Enregistré. La zone se trouve maintenant dans un autre fuseau UTM ; freemapper télécharge à nouveau ses données à la prochaine exécution.
page-provider-changed = Enregistré. Un autre fournisseur de données sert l'altitude et les images aériennes de la nouvelle zone ; freemapper télécharge à nouveau ses données à la prochaine exécution.
page-save-stale = freemapper.toml a été modifié dans une autre fenêtre ou par un autre programme depuis que cette page l'a lu. Rien n'a été enregistré ; la page connaît maintenant l'état enregistré. Enregistre à nouveau pour le remplacer par la zone affichée ici.
page-has-map = La carte existante (.omap) reste telle quelle ; freemapper crée de nouveaux modèles et candidats pour la nouvelle zone. L'option « Remplacer la carte existante » ci-dessous (ou « freemapper run --force ») refait aussi la carte et conserve l'ancienne sous le nom .omap.bak.
page-done = Terminé
page-source = Code source (AGPL-3.0-or-later)
page-closed = Terminé. Tu peux fermer cette fenêtre.
page-next = Ensuite : freemapper run "{ $dir }"
page-made-files = Créés : { $files }. Dossier : { $dir }
page-back-to-agent = Dis à ton assistant IA que la zone est enregistrée.
page-nothing-saved = Rien n'a été enregistré.
page-failed = Erreur : { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Photo aérienne { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Ce qui va sur la carte
page-content-help = Le profil décide de ce qui va directement sur la carte et de ce qui reste un candidat à vérifier. Un choix pour un groupe l'emporte sur le profil. Il est enregistré avec la zone.
page-content-profile-legend = Profil
page-content-profile-safe = Prudent
page-content-profile-safe-hint = Chaque objet reste là où l'assemblage de la carte le place : les bâtiments et les surfaces du cadastre vont sur la carte ; les murs, clôtures, arbres et beaucoup d'objets d'OpenStreetMap (étangs, chemins sans revêtement connu) restent des candidats.
page-content-profile-balanced = Équilibré
page-content-profile-balanced-hint = Les objets du cadastre, des registres et du relevé des rues vont sur la carte, avec les courbes de niveau et les zones interdites le long des voies ferrées. Murs, clôtures, haies, talus, escaliers, eaux, arbres, monuments, passages, chemins et voies ferrées viennent aussi d'OpenStreetMap. Le mobilier urbain, les autres zones interdites et les formes issues du modèle de terrain restent des candidats.
page-content-profile-full = Complet
page-content-profile-full-hint = Tout va sur la carte, comme dans l'impression de contrôle ; seuls les candidats cachés restent exclus.
page-content-groups-legend = Par groupe
page-content-groups-help = « Comme dans le profil » suit le profil. Ce que cela signifie pour un groupe dépend de l'origine de chaque objet.
page-content-place-profile = Comme dans le profil
page-content-place-map = Carte
page-content-place-candidates = Candidat
page-content-place-off = Désactivé
page-content-group-walls = Murs
page-content-group-fences = Clôtures
page-content-group-hedges = Haies
page-content-group-slopes = Talus et falaises
page-content-group-stairs = Escaliers
page-content-group-trees = Arbres et groupes d'arbres
page-content-group-water-features = Eaux, fontaines et sources
page-content-group-monuments = Tours, monuments et lignes électriques
page-content-group-furniture = Mobilier urbain et lampadaires
page-content-group-passages = Passages
page-content-group-contours = Courbes de niveau
page-content-group-landforms = Buttes, dépressions, talus et falaises issus du modèle de terrain
page-content-group-vegetation = Végétation issue de nuages de points laser (Brandebourg, Berlin, Rhénanie-du-Nord-Westphalie, Rhénanie-Palatinat, Sarre et Bavière ; désactivée tant que tu ne la choisis pas ; télécharge environ 40 à 600 Mo par km²)
page-content-group-out-of-bounds = Zones interdites
page-content-group-paths = Chemins et voies ferrées
page-content-group-streets = Rues et places
page-run-heading = Créer la carte
page-preview-heading = Aperçu de la carte
page-preview-stored = Telle que générée en dernier.
page-preview-live = Avec les réglages de cette page, pas encore enregistrés.
page-preview-live-saved = Avec les réglages enregistrés, pas encore générée lors d'une exécution.
page-preview-computing = Calcul …
page-preview-none = Aucune carte générée pour l'instant.
page-preview-later = Cette carte n'a pas encore d'aperçu. Il viendra avec la prochaine exécution qui écrit la carte.
page-preview-stale = La zone, le type de carte ou l'échelle ont changé depuis la génération de la carte. L'image montre encore la carte telle que générée en dernier ; un nouvel aperçu demande une exécution avec « Remplacer la carte existante ».
page-preview-area-unsaved = La zone de cette page n'est pas encore enregistrée. L'image montre la carte telle que générée en dernier ; la nouvelle zone demande une exécution avec « Remplacer la carte existante ».
page-preview-run = Telle que générée en dernier. La page met l'image à jour à la fin de l'exécution.
page-preview-open = Ouvrir en taille réelle
page-preview-alt = Aperçu des objets de la carte
page-opt-ocad = Écrire un projet de départ OCAD
page-help-ocad = Pour tous ceux qui continuent dans OCAD.
page-opt-pdf = PDF imprimable
page-help-pdf = Objets de la carte seulement, sans candidats.
page-opt-pdf-candidates = Impression de contrôle avec tous les candidats
page-opt-training = Carte d'entraînement
page-help-training = La carte et tous les candidats, nettoyés pour être imprimés tels quels ; avec OCAD aussi en fichier OCAD.
page-views-heading = Vues de la carte d'entraînement
page-view-no-paths = Sans chemins (carte Ultimate)
page-view-brown-only = Brun seulement (course d'orientation sur courbes de niveau)
page-view-no-north-lines = Sans lignes nord
page-view-custom = Autre vue
page-help-views = Chaque vue cochée est un fichier de plus à côté de la carte d'entraînement, `<name>-training-<view>.omap` et son PDF, avec OCAD aussi un fichier OCAD. Une autre vue s'écrit `no:505,506` (omettre des symboles) ou `only:101,102` (garder seulement ceux-ci) ; `+` combine des vues. Le titre, l'échelle et les sources restent toujours.
page-help-views-forest = Sans chemins et brun seulement ne valent que pour les cartes de forêt.
page-opt-scale-bar = Barre d'échelle sur les PDF
page-help-scale-bar = Les cartes allemandes n'en ont généralement pas.
page-opt-sources = Lignes de sources sur les PDF
page-sources-auto = automatique
page-sources-full = complètes
page-sources-credit-only = freemapper seulement
page-warn-credit-only = Les PDF ne nomment alors que freemapper. Les licences des données (ODbL, dl-de/by-2-0) exigent une mention quand tu transmets la carte ; tu nommes alors toi-même les sources. ATTRIBUTION.txt les liste toutes.
page-need-pdf = La barre d'échelle et les lignes de sources ne valent que si un PDF est créé.
page-opt-offline = Cache seulement, pas de réseau
page-opt-force = Remplacer les cartes existantes
page-help-force = La carte et, si « Carte d'entraînement » est cochée, la carte d'entraînement. Les anciens fichiers restent sous le nom .omap.bak.
page-opt-permits = Notes d'autorisation
page-help-permits = Unités administratives, zones protégées et à qui s'adresser, dans permits.md. Une indication, pas un avis juridique.
page-run-start = Créer la carte
page-run-again = Créer à nouveau
page-run-cancel = Annuler
page-run-cancelling = Annulation …
page-warnings-heading = Avertissements
page-where-files = Les fichiers se trouvent dans le dossier du projet :
page-open-folder = Ouvrir le dossier
page-log-label = Journal de l'exécution
page-force-title = Remplacer les cartes existantes ?
page-force-text = La carte existante est remplacée, et la carte d'entraînement aussi si « Carte d'entraînement » est cochée. Tes modifications ne sont pas perdues : les anciens fichiers restent dans le dossier du projet sous le nom .omap.bak. Continuer ?
page-force-cancel = Annuler
page-force-confirm = Remplacer
page-step-fetch = Téléchargement des données
page-step-derive = Calcul des modèles
page-step-map = Écriture de la carte
page-step-ocad = Écriture du projet OCAD
page-step-pdf = Écriture du PDF
page-step-pdf-candidates = Écriture de l'impression de contrôle
page-step-training = Écriture de la carte d'entraînement
page-step-permits = Écriture des notes d'autorisation
page-step-line = Étape { $index } sur { $total } : { $label }
page-starting = L'exécution démarre …
page-file-kind-map = carte
page-file-kind-template = modèle
page-file-kind-ocad = carte OCAD
page-file-kind-ocad-candidates = candidats OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = impression de contrôle
page-file-kind-training-map = carte d'entraînement
page-file-kind-training-pdf = PDF de la carte d'entraînement
page-file-kind-training-ocad = carte d'entraînement OCAD
page-file-kind-training-view-map = vue de la carte d'entraînement
page-file-kind-training-view-pdf = PDF d'une vue de la carte d'entraînement
page-file-kind-training-view-ocad = vue OCAD de la carte d'entraînement
page-file-kind-preview = aperçu de la carte
page-file-kind-permits = notes d'autorisation
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Terminé. Les fichiers sont écrits.
page-result-failed = L'exécution a échoué : { $error }
page-result-failed-kept = Ce qui est écrit reste dans le dossier du projet ; une nouvelle exécution reprend à partir de là.
page-result-cancelled = Annulé. Ce qui est déjà téléchargé reste dans le cache ; une nouvelle exécution reprend à partir de là.
page-result-title-done = Carte créée
page-result-title-failed = Exécution échouée
page-result-title-cancelled = Exécution annulée
page-kept-map = La carte existante a été laissée telle quelle. « Remplacer la carte existante » la refait et conserve l'ancienne sous le nom .omap.bak.
page-no-files = Aucun fichier n'a été créé.
page-opened-folder = Le dossier est ouvert.
page-open-failed = { $message } Dossier : { $dir }
page-why-no-area = Choisis d'abord une zone.
page-why-checking = L'aperçu est en cours de calcul …
page-why-too-large = La zone est trop grande.
page-why-outside = La zone se trouve hors des Länder pris en charge.
page-why-disk = Pas assez d'espace disque libre (voir ci-dessus).
page-why-running = Une exécution est en cours.
page-err-scale-bar-no-pdf = La barre d'échelle demande un PDF.
page-err-sources-no-pdf = Les lignes de sources demandent un PDF.
page-save-failed = La zone n'est pas enregistrée ; aucune exécution n'a été lancée.
page-poll-lost = Pas de réponse du serveur. freemapper continue d'essayer.
page-run-unknown = Le serveur ne connaît plus cette exécution.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Fermer

## update (spec 3.4 to 3.6)

update-source-build = Ce freemapper a été compilé à partir des sources. Mettez-le à jour avec git pull et cargo build --release --locked.
update-current = freemapper { $version } est à jour.
update-available = freemapper { $new } est disponible (vous avez { $old }), publié le { $released }.
update-details =
    Notes de version : { $notes }
    Téléchargement : { $size } Mo
update-decimal-separator = ,
update-ask = L'installer maintenant ? [o/N]
update-not-installed = Rien n'a été installé.
update-needs-yes = l'entrée n'est pas un terminal, freemapper ne peut donc pas demander avant d'installer ; lancez freemapper update --yes pour installer sans demander
update-installing = téléchargement et vérification de freemapper { $version }
update-done = freemapper est passé de { $old } à { $new }. Notes de version : { $notes }
update-mcp-restart = Un serveur MCP en cours d'exécution garde l'ancienne version jusqu'à ce que l'assistant IA le redémarre.
update-rolled-back = Retour à freemapper { $version }. La version quittée est conservée sous { $path }.
update-not-writable = impossible d'écrire dans le dossier du programme { $path }, et freemapper ne demande pas de droits d'administrateur. Décompressez la nouvelle version à la main à cet endroit, ou déplacez freemapper dans un dossier où vous pouvez écrire.
update-too-old = freemapper { $new } ne peut pas être installé par la mise à jour automatique de { $old } ; installez { $needed } ou plus récent à la main, voir { $notes }
update-busy = une autre mise à jour ou un retour en arrière est en cours dans le dossier du programme (fichier de verrou { $path }) ; attendez la fin et réessayez
update-bad-signature = la version n'a pas pu être vérifiée ({ $error }) ; rien n'a été installé
update-check-bad-signature = la version la plus récente de freemapper n'a pas pu être vérifiée ({ $error }) ; rien n'a été installé
update-no-manifest = la version la plus récente ne peut pas s'installer elle-même (elle n'a pas de manifeste de mise à jour) ; téléchargez-la sur { $page }
update-no-manifest-version = la version { $version } n'existe pas ou ne peut pas s'installer elle-même (elle n'a pas de manifeste de mise à jour) ; voir { $page }
update-no-asset = la version n'a pas de programme pour ce système ({ $platform }) ; rien n'a été installé
update-no-previous = il n'y a pas de version précédente à restaurer ({ $path } est absent)
update-failed = la mise à jour a échoué : { $error }
update-mode-set = Mode de mise à jour réglé sur { $mode } dans { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } remplace ce réglage tant que cette variable est définie.
update-mode-failed = impossible d'écrire { $path } : { $error }
update-notice = freemapper { $new } est disponible (vous avez { $old }). Mise à jour avec : freemapper update
update-applies-next-start = La nouvelle version s'applique dès le prochain démarrage de freemapper.
update-install-failed = freemapper { $new } n'a pas été installé : { $error }
update-no-config-dir = aucun dossier de configuration sur ce système ; définissez FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Note pour la personne : au démarrage, le serveur freemapper a trouvé ceci dans les dossiers d'adaptateurs. Transmettez-le ; la personne corrige le dossier ou le supprime, puis redémarre le serveur.
    { $lines }
update-mcp-note = Note pour la personne : freemapper { $new } est disponible (la version actuelle est { $old }). Demandez-lui avant d'appeler install_update.
update-mcp-needs-confirm = rien n'a été installé ; demandez d'abord à la personne et appelez install_update avec confirm: true une fois qu'elle a accepté
update-mcp-note-manual = Note pour la personne : freemapper { $new } est disponible (la version actuelle est { $old }), mais install_update ne peut pas l'installer ; installez { $needed } ou plus récent à la main, voir { $notes }
update-mcp-note-bad-signature = Note pour la personne : la version la plus récente de freemapper n'a pas pu être vérifiée ({ $error }) ; rien n'a été installé. Dites-le-lui, et ne l'installez pas d'une autre manière.
update-mcp-restart-now = Redémarrez maintenant le serveur MCP freemapper ; il exécute encore l'ancienne version.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = erreur
error-cancelled = annulé
cli-cancelled-kept = annulé ; conservé : { $files }
error-io = { $path } : { $message }
error-io-disk-full = le disque est plein, freemapper n'a donc pas pu écrire { $path }. Libérez de l'espace et réessayez

## Errors: project

error-project-already-exists = le projet existe déjà : { $path }
error-project-map-exists = la carte existe déjà (elle peut contenir votre travail) ; utilisez --force pour la remplacer : { $path }
error-project-invalid-scale = échelle 1:{ $scale } non valide
error-project-invalid-contour-interval = équidistance { $interval } m non valide dans freemapper.toml ; elle doit être supérieure à 0
error-project-invalid-steep-slope = seuil de pente { $slope }° non valide dans freemapper.toml ; il doit être compris entre 15° et 80°
error-project-invalid-face-slope = seuil de paroi { $slope }° non valide dans freemapper.toml ; il doit être compris entre 55° et 85°
error-project-invalid-paper-frame-centre = centre de cadre papier non valide, lon { $lon }, lat { $lat } ; ce doit être un lieu sur Terre
error-project-scale-not-allowed = l'échelle 1:{ $scale } n'est pas autorisée pour les cartes de type { $kind } ; utilisez { $allowed }
error-project-scale-not-allowed-enlarge = l'échelle 1:{ $scale } n'est pas autorisée pour les cartes de type { $kind } ; utilisez { $allowed }. Dessinez à 1:4000, freemapper-print agrandit à 1:3000 à l'impression
word-kind-schoolyard = cour d'école
word-kind-sprint = sprint
word-kind-forest = forêt
error-project-toml = fichier de projet non valide : { $message }
error-project-unknown-adapter = le fichier de projet nomme l'adaptateur « { $adapter } », que ce freemapper n'a pas ; un freemapper plus récent peut l'apporter (`freemapper update`), ou il vient de votre dossier d'adaptateurs (`freemapper adapters list` le montre) ou, avec --adapters-from-project en ligne de commande, du dossier `adapters` du projet (seulement pour un projet de confiance)
error-project-not-fetched = aucune tuile téléchargée pour l'instant ; lancez d'abord `freemapper fetch`
error-project-ocd = export OCAD : { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path } : l'image et le fichier de géoréférencement ne correspondent pas à la zone de la carte ; relancez `freemapper derive`
error-project-tiles-outdated = la zone de la carte a besoin des tuiles { $tiles } qui n'ont pas été téléchargées ; lancez `freemapper fetch`
error-project-buildings-not-fetched = aucun bâtiment ALKIS enregistré pour cette zone de carte ; lancez d'abord `freemapper fetch` en ligne
error-project-orthophoto-not-fetched = aucune orthophotographie enregistrée pour cette zone de carte ; lancez d'abord `freemapper fetch` en ligne
error-project-terrain-not-fetched = aucun modèle de terrain enregistré pour cette zone de carte ; lancez d'abord `freemapper fetch` en ligne
error-project-credentials-missing = les données de { $name } demandent une connexion : définissez les variables d'environnement { $user_env } (le nom d'utilisateur) et { $password_env } (le mot de passe) ; le compte doit avoir commandé : { $products }
error-project-candidates-outdated = les candidats OSM enregistrés ne couvrent pas la zone de la carte ; lancez `freemapper fetch`
error-project-stored = { $path } : { $message } ; relancez `freemapper fetch`
error-project-area-too-large = la zone de la carte plus la marge fait { $width } × { $height } m, plus de { $limit } m par côté ; réduisez la zone de la carte ou buffer_m dans freemapper.toml
error-project-area-crosses-country = la zone de la carte s'étend des deux côtés d'une frontière ; les cartes à cheval sur une frontière ne sont pas encore prises en charge, dessinez donc la zone à l'intérieur d'un seul pays
error-project-permits-not-for-region = les notes d'autorisation ne sont pas encore disponibles pour cette région : elles interrogent des services allemands et nomment des autorités allemandes
error-project-outside-coverage = la zone de la carte se trouve hors des Länder pris en charge ({ $states }) ; freemapper n'a de sources de données que pour ceux-ci pour l'instant

## Errors: disk space

error-space-required = pas assez d'espace disque pour { $part }. Il faut { $needed }, mais { $path } n'a que { $left } de libre au-delà de la réserve de { $reserve }. Libérez de l'espace et relancez `freemapper fetch`
error-space-skipped = freemapper omet { $part } faute d'espace disque. Il faut { $needed }, mais { $path } n'a que { $left } de libre au-delà de la réserve de { $reserve }. Libérez de l'espace et relancez `freemapper fetch`
word-part-map-data = les données cartographiques ALKIS et OSM
word-part-terrain = le modèle de terrain
word-part-orthophoto = l'orthophotographie
word-part-surface = le modèle de surface pour le modèle nDSM
word-part-osm-update = la mise à jour de l'extrait OSM en cache
word-part-k-five = le modèle de carte K5 de Berlin
word-part-berlin-layers = les structures ALKIS de Berlin, le cadastre des arbres, le relevé des rues et les couches de points
word-part-state-layers = les structures ALKIS complètes, les objets du modèle de paysage et les cadastres des arbres
word-part-point-cloud = les nuages de points laser pour les candidats de végétation

## Errors: downloads

error-fetch-http = le téléchargement de { $url } a échoué : { $message }
error-fetch-http-temporary = le téléchargement de { $url } a échoué : { $message }. Le service est probablement surchargé ou en maintenance ; freemapper a essayé jusqu'à { $attempts } fois. Relancez l'exécution dans quelques minutes ; ce qui est déjà téléchargé est conservé.
error-fetch-corrupt-archive = le téléchargement de { $url } n'est pas une archive de tuiles complète : { $message }
error-fetch-corrupt-extract = le téléchargement de { $url } n'est pas un extrait OSM complet : { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] la tuile { $tiles } n'est pas couverte par les tuiles publiées ; réduisez la zone de la carte ou buffer_m dans freemapper.toml
       *[other] les tuiles { $tiles } ne sont pas couvertes par les tuiles publiées ; réduisez la zone de la carte ou buffer_m dans freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] hors ligne et pas en cache : tuile { $product } { $tiles } ; lancez `freemapper fetch` en ligne
       *[other] hors ligne et pas en cache : tuiles { $product } { $tiles } ; lancez `freemapper fetch` en ligne
    }
word-product-orthophoto = orthophotographie (DOP20)
word-product-dtm = modèle de terrain (DGM1)
word-product-image-dsm = modèle de surface (bDOM)
word-product-saxony-dtm = modèle de terrain (GeoSN DGM1)
word-product-saxony-dsm = modèle de surface (GeoSN DOM1)
error-fetch-osm-extract-missing = aucun extrait OSM en cache ; lancez `freemapper fetch` en ligne
error-fetch-not-found = { $name } introuvable sur le serveur
error-fetch-refused = { $url } : le serveur a refusé la requête (HTTP { $status })
error-fetch-refused-detail = { $url } : le serveur a refusé la requête (HTTP { $status }) : { $detail }
error-fetch-too-large = le téléchargement de { $url } dépasse { $limit } Mo ; arrêté
error-fetch-not-stored = aucun { $label } enregistré pour cette zone de carte : { $path } est absent, illisible ou ne la couvre pas ; lancez d'abord `freemapper fetch` en ligne
error-fetch-range-ignored = { $url } : le serveur a ignoré la plage d'octets { $range } (HTTP { $status })
error-fetch-not-allowed = { $url } : freemapper n'envoie pas de requêtes à cette adresse : { $reason }
error-fetch-host-not-named-allow = { $target } : freemapper n'envoie pas de requêtes à cette adresse : l'adaptateur de { $name } ne nomme pas l'hôte { $host }{ $from }. Si le service a déménagé là, autorisez l'hôte avec `freemapper adapters allow-host { $adapter } { $host }` et signalez-le sur https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target } : freemapper n'envoie pas de requêtes à cette adresse : l'adaptateur { $adapter } ne nomme pas l'hôte { $host }{ $from }. Si le service a déménagé là, ajoutez l'hôte à `[net] hosts` du manifeste de l'adaptateur
error-fetch-host-redirected-from = {" "}({ $url } redirige vers cet hôte)
error-reason-not-a-host-name = pas un nom d'hôte
error-reason-not-ascii = un nom hors ASCII doit être écrit sous sa forme xn--
error-reason-not-a-dns-name = pas un nom DNS
error-reason-no-domain = un nom de cet ordinateur ou sans domaine
error-reason-local-name = un nom du réseau local
error-reason-ip-address = une adresse IP, pas un nom DNS
error-reason-unexpected-character = caractère inattendu
error-reason-not-https = pas https
error-reason-user-info = informations d'utilisateur
error-reason-port = un port autre que 443
error-reason-host-not-declared = l'hôte ne fait pas partie des hôtes déclarés
error-reason-local-address = une adresse de cet ordinateur ou du réseau local, pas d'un serveur sur Internet
error-reason-too-many-redirects = plus de { $count } redirections
error-reason-redirect-nowhere = une redirection (HTTP { $status }) vers nulle part
error-fetch-currency-csv = CSV d'actualité, ligne { $line } : { $message }
error-alkis = données ALKIS non valides : { $message }
error-osm = données OSM : { $message }

## Errors: templates and rasters

error-derive-zip = archive de tuiles : { $message }
error-derive-image = image : { $message }
error-derive-no-tiles = aucune tuile pour le calcul
error-derive-area-too-large = zone de carte trop grande pour un seul modèle : { $width } × { $height } pixels ({ $limit } au plus par côté) ; réduisez la zone de la carte ou buffer_m dans freemapper.toml
error-raster-world-file = fichier de géoréférencement non valide : { $message }
error-raster-misaligned = les rasters ne sont pas alignés sur la même grille de pixels : { $message }
error-raster-pixel-size = les rasters ont des tailles de pixel différentes : { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = GeoJSON non valide : { $message }
error-area-no-polygon = le GeoJSON ne contient aucun polygone
error-area-multi-polygon = MultiPolygon de { $parts } parties ; découpez la zone de la carte en un seul polygone
error-area-out-of-range = coordonnée hors limites : lat { $lat }, lon { $lon }
error-area-not-an-area = pas une zone de carte utilisable : { $message }
error-georef-date = la date { $date } est hors de la plage de validité du World Magnetic Model
error-georef-map-range = le point E { $easting } N { $northing } se trouve au-delà de la plage de coordonnées de la carte
error-omap-xml = XML OOM non valide : { $message }
error-omap-missing-element = élément <{ $name }> introuvable
error-ocd-duplicate-symbol = le symbole numéro { $symbol } est utilisé deux fois
error-ocd-unknown-symbol = l'objet renvoie au symbole { $symbol }, qui n'est pas dans le fichier
error-ocd-wrong-type = un objet du symbole { $symbol } n'a pas le bon type pour son symbole

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Utilisation :
cli-heading-commands = Commandes :
cli-heading-options = Options :
cli-heading-arguments = Arguments :
cli-help-possible-values = valeurs possibles
cli-help-default = par défaut
cli-arg-help = Afficher l'aide
cli-arg-version = Afficher la version
cli-help-help = Afficher l'aide d'une commande
cli-arg-help-command = La commande dont afficher l'aide, par exemple run
cli-parse-hint = Pour plus d'informations, essayez '--help'.
cli-double-click-start = freemapper ouvre dans le navigateur la page où vous choisissez la zone de la carte. Le dossier du projet est { $path }. Fermez cette fenêtre pour quitter le programme.
cli-parse-unknown-argument = argument inattendu { $arg }
cli-parse-missing-required = argument obligatoire manquant : { $args }
cli-parse-invalid-value = valeur { $value } non valide pour { $arg }
cli-parse-invalid-value-choices = valeur { $value } non valide pour { $arg } ; valeurs possibles : { $values }
cli-parse-invalid-subcommand = commande inconnue { $value }
cli-parse-missing-subcommand = une commande est requise
cli-parse-too-many-values = valeur inattendue { $value } pour { $arg }
cli-parse-argument-conflict = { $arg } ne peut pas être utilisé avec { $other }

help-freemapper = Crée des projets de départ OOM à partir de géodonnées
help-freemapper-lang = Langue des textes du programme, `de` ou `en` (par défaut : FREEMAPPER_LANG, puis la langue du système, puis l'anglais)
help-init = Créer un dossier de projet pour une zone de carte
help-init-dir = Dossier de projet à créer
help-init-geojson = Fichier GeoJSON avec le polygone de la zone de carte (EPSG:4326)
help-init-osm = Objet OSM de la zone de carte, p. ex. way/992423119
help-init-place = Nom du lieu, p. ex. "Clara-Zetkin-Schule Strausberg" ; freemapper utilise la meilleure correspondance qui a une zone
help-init-part = Quelle partie d'une zone OSM de plusieurs parties séparées utiliser (1 est la plus grande)
help-init-kind = Type de carte
help-arg-scale = Dénominateur de l'échelle d'un nouveau projet : cour d'école 1000 (par défaut) ; sprint 4000 (par défaut) ou 5000 ; forêt 15000 (par défaut), 10000 ou 7500. Les autres valeurs sont refusées, car freemapper-print les refuse
help-init-title = Titre de la carte pour les informations périphériques (par défaut : le nom du dossier)
help-export = Écrire le projet de départ (carte OOM, modèles, mentions de sources) dans un dossier de projet
help-export-dir = Dossier de projet
help-arg-force = Remplacer un fichier de carte existant et conserver l'ancien sous le nom `<name>.omap.bak`
help-export-ocad = Écrire aussi un point de départ OCAD dans `ocad/` (format OCAD 12) ; une carte OOM existante reste alors en place sauf avec --force
help-export-pdf = Écrire aussi `<name>.pdf`, la carte imprimable (les objets de la carte seulement, sans candidats) ; une carte OOM existante reste alors en place sauf avec --force
help-export-pdf-candidates = Écrire aussi `<name>-candidates.pdf`, l'impression de contrôle : la carte avec tous les candidats, à leur position exacte ; une carte OOM existante reste alors en place sauf avec --force
help-export-training = Écrire aussi la carte d'entraînement `<name>-training.omap` et son PDF `<name>-training.pdf` : la carte et tous les candidats, nettoyés pour être imprimés tels quels ; avec --ocad aussi `ocad/<name>-training.ocd`. Une carte OOM existante reste alors en place sauf avec --force, et --force remplace aussi une carte d'entraînement existante
help-export-view = Avec --training : écrire aussi la carte d'entraînement sans ce qu'une vue omet, sous la forme `<name>-training-<view>.omap` et son PDF (avec --ocad aussi un fichier OCAD). Vues : `no-paths` (ni routes, ni chemins, ni escaliers), `brown-only` (seulement les symboles bruns et le cadre), `no-north-lines`, ou une liste `no:<codes>` / `only:<codes>` ; à combiner avec `+`. Répétez l'option pour plusieurs fichiers
help-export-pdf-scale-bar = Dessiner la barre d'échelle sur les PDF (désactivée par défaut, les cartes allemandes n'en ont généralement pas ; le nombre d'échelle reste dans le cartouche)
help-export-pdf-sources = Lignes de sources sur les PDF : `auto` imprime la liste complète et ne la raccourcit que si elle dépasse 5 lignes, `full` ne la raccourcit jamais, `credit-only` n'imprime que la mention de crédit de freemapper (l'ODbL et dl-de/by-2-0 demandent alors de nommer les sources quand la carte est transmise). ATTRIBUTION.txt liste toujours toutes les sources
help-fetch = Télécharger les données de la zone de carte (orthophotographie, modèle de terrain, nDSM, bâtiments ALKIS, OSM)
help-fetch-dir = Dossier de projet
help-fetch-offline = Utiliser seulement le cache, sans accès au réseau
help-arg-cache-dir = Dossier de cache pour les tuiles téléchargées (par défaut : le dossier de cache du système + `freemapper`)
help-derive = Calculer les modèles (orthophotographie, ombrage, pente, relief local, nDSM), les courbes de niveau et les formes du terrain à partir des données téléchargées
help-derive-dir = Dossier de projet
help-run = Télécharger, calculer et exporter en une seule fois
help-run-dir = Dossier de projet
help-run-offline = Utiliser seulement le cache, sans accès au réseau
help-run-ocad = Écrire aussi un point de départ OCAD dans `ocad/` (format OCAD 12)
help-run-pdf = Écrire aussi `<name>.pdf`, la carte imprimable (les objets de la carte seulement, sans candidats)
help-run-pdf-candidates = Écrire aussi `<name>-candidates.pdf`, l'impression de contrôle : la carte avec tous les candidats, à leur position exacte
help-run-training = Écrire aussi la carte d'entraînement `<name>-training.omap` et son PDF `<name>-training.pdf` : la carte et tous les candidats, nettoyés pour être imprimés tels quels ; avec --ocad aussi `ocad/<name>-training.ocd`. Une carte d'entraînement existante reste en place sauf avec --force
help-run-view = Avec --training : écrire aussi la carte d'entraînement sans ce qu'une vue omet, sous la forme `<name>-training-<view>.omap` et son PDF (avec --ocad aussi un fichier OCAD). Vues : `no-paths` (ni routes, ni chemins, ni escaliers), `brown-only` (seulement les symboles bruns et le cadre), `no-north-lines`, ou une liste `no:<codes>` / `only:<codes>` ; à combiner avec `+`. Un fichier de vue existant reste en place sauf avec --force
help-run-pdf-scale-bar = Dessiner la barre d'échelle sur les PDF (désactivée par défaut ; le nombre d'échelle reste dans le cartouche)
help-run-pdf-sources = Lignes de sources sur les PDF : `auto` imprime la liste complète et ne la raccourcit que si elle dépasse 5 lignes, `full` ne la raccourcit jamais, `credit-only` n'imprime que la mention de crédit de freemapper (l'ODbL et dl-de/by-2-0 demandent alors de nommer les sources quand la carte est transmise). ATTRIBUTION.txt liste toujours toutes les sources
help-select = Choisir la zone de la carte dans le navigateur, en la dessinant ou en désignant un lieu. L'enregistrement écrit freemapper.toml et crée le dossier de projet si nécessaire
help-select-dir = Dossier de projet (créé à l'enregistrement s'il n'existe pas)
help-select-kind = Type de carte d'un nouveau projet
help-select-title = Titre de la carte d'un nouveau projet
help-select-no-browser = Afficher seulement l'adresse de la page ; ne pas ouvrir le navigateur
help-select-port = Port sur 127.0.0.1 (par défaut : un port libre)
help-select-offline = La page démarre avec « Cache seulement, pas de réseau » coché
help-select-ocad = La page démarre avec le projet de départ OCAD coché
help-select-pdf = La page démarre avec le PDF imprimable coché
help-select-pdf-candidates = La page démarre avec l'impression de contrôle (tous les candidats) cochée
help-select-training = La page démarre avec la carte d'entraînement cochée
help-select-pdf-scale-bar = La page démarre avec la barre d'échelle sur les PDF cochée
help-select-pdf-sources = La page démarre avec ce choix de lignes de sources sur les PDF, comme pour `run`
help-find = Lister les lieux OpenStreetMap pour un nom, les zones d'abord, avec l'identifiant pour init --osm
help-find-query = Nom du lieu, p. ex. "Clara-Zetkin-Schule Strausberg"
help-find-json = Afficher les correspondances en JSON, avec les mêmes champs que l'outil MCP find_place
help-mcp = Fournir les outils à un agent IA via MCP (stdin/stdout)
help-update = Installer un freemapper plus récent depuis les versions GitHub, après confirmation. Une version compilée à partir des sources indique plutôt comment la mettre à jour
help-update-check = Vérifier seulement : code de sortie 10 s'il existe une version plus récente, 0 si celle-ci est à jour
help-update-yes = Installer sans demander ; nécessaire quand l'entrée n'est pas un terminal
help-update-version = Installer cette version (x.y.z) au lieu de la plus récente, y compris une plus ancienne
help-update-rollback = Revenir à la version que la dernière mise à jour a remplacée
help-update-mode = Enregistrer ce que fait la vérification quotidienne quand elle trouve une version plus récente (par défaut : notify). Fonctionne dans toutes les versions, car l'option ne fait qu'écrire le réglage
help-adapters = Afficher les adaptateurs régionaux : les Länder et pays pour lesquels freemapper a des sources de données
help-adapters-list = Lister les adaptateurs avec leur origine (intégré, votre dossier, un projet), leur révision, la date de leur dernière vérification et les hôtes qu'ils interrogent
help-adapters-list-json = Afficher la liste en JSON
help-adapters-list-project = Lire aussi les adaptateurs du dossier `adapters` de ce dossier de projet, comme le ferait --adapters-from-project
help-adapters-check = Vérifier le dossier d'un adaptateur avant son utilisation ou sa transmission : s'il se chargerait, ce que son auteur devrait savoir et, avec --live, si ses services répondent. Code de sortie 0 : il réussit ; 1 : il a des constats ; 2 : la vérification n'a pas pu s'exécuter
help-adapters-check-dir = Le dossier de l'adaptateur ; son nom est l'identifiant de l'adaptateur
help-adapters-check-live = Interroger aussi les services de l'adaptateur pour le cadre que --bbox désigne et dire ce que chacun fournit. Les requêtes restent sur les hôtes que déclare le manifeste
help-adapters-check-bbox = Le cadre de la vérification en direct en degrés, sous la forme minLon,minLat,maxLon,maxLat. Il se trouve dans la limite de l'adaptateur ; 150 m sur 150 m suffisent
help-adapters-check-json = Afficher le résultat en JSON
help-adapters-probe = Interroger une fois chaque service que nomment les adaptateurs intégrés et dire s'il répond : ok, slow, moved ou error. Le code de sortie est 0 même quand un service échoue ; le résultat est la réponse
help-adapters-probe-json = Afficher le résultat en JSON
help-adapters-probe-status = Lire l'état de la nuit précédente dans ce fichier (il peut manquer) et afficher le nouvel état en JSON : un service est hors service après deux mauvaises nuits de suite. Les changements vont sur la sortie d'erreur standard
help-adapters-allow-host = Autoriser un hôte de plus pour un adaptateur intégré : pour un Land dont le service a déménagé vers un serveur que son adaptateur ne nomme pas encore. Le message sur l'adresse refusée indique quel hôte. L'hôte est vérifié comme un hôte de manifeste, et freemapper ne se connecte toujours qu'à des serveurs sur Internet
help-adapters-allow-host-adapter = L'identifiant de l'adaptateur intégré, tel que `adapters list` le montre
help-adapters-allow-host-host = L'hôte à autoriser : un nom DNS, pas d'adresse, pas de port
help-adapters-disallow-host = Retirer un hôte que allow-host a autorisé
help-adapters-disallow-host-adapter = L'identifiant de l'adaptateur intégré
help-adapters-disallow-host-host = L'hôte à retirer
help-adapters-export = Écrit l'adaptateur intégré d'un Land comme dossier d'adaptateur que vous pouvez modifier : son manifeste et les fichiers que le manifeste nomme, dans `<dir>/<id>`. Le dossier ne doit pas encore exister ou doit être vide. Modifiez les adresses dans son `manifest.toml`, vérifiez-le avec `adapters check` et placez-le dans votre dossier d'adaptateurs : il remplace alors l'adaptateur intégré, avec un avertissement. Supprimer le dossier rétablit l'adaptateur intégré
help-adapters-export-adapter = L'identifiant de l'adaptateur intégré, tel que le montre `adapters list`
help-adapters-export-dir = Le dossier dans lequel le dossier `<id>` est créé
help-sources = Lire et corriger les adresses des sources de données sans nouvelle version. Une correction va dans votre propre dossier d'adaptateurs et remplace l'adresse intégrée
help-sources-list = Lister les adresses des manifestes avec leurs chemins et l'origine de chaque manifeste. Avec un identifiant d'adaptateur, seulement les siennes
help-sources-get = Afficher une adresse et rien d'autre. Code de sortie 1 si l'adaptateur ou le chemin est inconnu
help-sources-set = Corriger une adresse dans votre propre dossier d'adaptateurs une fois que tout le manifeste a passé sa vérification. Une adresse sur un hôte que le manifeste ne déclare pas est refusée, sauf avec --add-host
help-sources-diff = Montrer ce que votre dossier d'adaptateur change par rapport au manifeste intégré
help-sources-check = Vérifier le dossier de votre adaptateur de cet identifiant comme `adapters check`. Code de sortie 0 : réussi ; 1 : constats ; 2 : la vérification n'a pas pu s'exécuter
help-sources-schema = Afficher les clés d'un manifeste qui contiennent une adresse, avec --json comme schéma JSON
help-arg-adapter = L'identifiant de l'adaptateur, tel que `adapters list` le montre
help-arg-path = Le chemin de l'adresse, tel que `sources list` le montre
help-sources-set-url = La nouvelle adresse : https, sans requête
help-sources-set-add-host = Ajouter aussi l'hôte de la nouvelle adresse aux hôtes que déclare le manifeste
cli-sources-no-user-dir = ce système n'a pas de dossier pour vos adaptateurs ; définissez FREEMAPPER_ADAPTER_DIR sur un chemin absolu
cli-sources-bad-id = { $adapter } n'est pas un identifiant d'adaptateur
cli-sources-unknown-adapter = freemapper ne connaît pas d'adaptateur { $adapter } ; `freemapper adapters list` montre les identifiants
cli-sources-no-manifest = aucun manifeste de ce programme ne décrit l'adaptateur { $adapter } ; ses adresses sont dans le code
cli-sources-unknown-path-of = l'adaptateur { $adapter } n'a pas d'adresse sous { $path } ; `freemapper sources list { $adapter }` montre les chemins
cli-sources-bad-value = la nouvelle adresse doit être une adresse https sur un hôte nommé, sans guillemet, barre oblique inverse, caractère de contrôle, utilisateur ni port
cli-sources-host-not-declared = la nouvelle adresse mène à l'hôte { $host }, que le manifeste ne déclare pas ; --add-host le déclare
cli-sources-invalid = le manifeste ne passerait pas avec ce changement, donc rien n'a été écrit : { $findings }
cli-sources-set-created = Le dossier { $folder } de l'adaptateur { $adapter } a été créé à partir de son manifeste intégré ; il remplace l'adaptateur intégré dès le prochain démarrage.
cli-sources-set-host-added = L'hôte { $host } est maintenant déclaré dans [net] hosts ; freemapper interroge désormais ce serveur pour cet adaptateur.
cli-sources-set-next = Vérifiez-le avec `freemapper adapters check { $folder }`. La mention de la source et la licence restent comme elles étaient : gardez-les exactes si le service a changé de mains. Redémarrez le serveur MCP ou la page de zone.
cli-sources-diff-no-folder = vous n'avez pas de dossier de l'adaptateur { $adapter } ({ $folder }) ; il n'y a rien à comparer
cli-sources-diff-none = votre dossier contient les mêmes adresses que le manifeste intégré
cli-sources-update-stored = Liste de corrections du { $issued } enregistrée ; { $applied } de ses corrections s'appliquent à ce programme.
cli-sources-update-current = La liste de corrections du { $issued } est déjà enregistrée.
cli-sources-update-older = La liste de GitHub date du { $offered }, plus ancienne que celle enregistrée du { $stored } ; celle enregistrée reste.
cli-sources-update-source-build = Ce freemapper a été compilé depuis les sources et ne demande pas à GitHub de corrections d'adresses ; un programme publié le fait.
cli-sources-update-failed = impossible d'obtenir la liste de corrections : { $reason }
cli-sources-update-mode-set = La vérification quotidienne de la liste de corrections est { $mode } (réglage dans { $path }).
cli-sources-update-removed = La liste de corrections enregistrée est supprimée. Dès le prochain démarrage, les adresses du programme font foi.
cli-sources-update-none = Aucune liste de corrections n'est enregistrée ; `freemapper sources update` en obtient une.
cli-sources-corrections-head = Liste de corrections du { $issued } (signature vérifiée), entrées : { $count }
cli-corrections-invalid = la liste de corrections enregistrée n'est pas utilisée : { $reason }
cli-corrections-refused = la correction de { $adapter } à { $path } n'est pas utilisée : { $reason }
cli-corrections-daily-stored = Une liste de corrections d'adresses plus récente a été enregistrée (du { $issued }) ; { $applied } de ses corrections s'appliquent à ce programme.
help-sources-list-json = Afficher la liste en JSON
help-sources-check-live = Interroger aussi les services de l'adaptateur pour le cadre que --bbox désigne et dire ce que chacun fournit. Les requêtes restent sur les hôtes que déclare le manifeste
help-sources-check-bbox = Le cadre de la vérification en direct en degrés, sous la forme minLon,minLat,maxLon,maxLat. Il se trouve dans la limite de l'adaptateur ; 150 m sur 150 m suffisent
help-sources-check-json = Afficher le résultat en JSON
help-sources-schema-json = Afficher le résultat en JSON
help-sources-update = Obtient de GitHub la liste signée de corrections d'adresses et l'enregistre, pour qu'un service déménagé refonctionne sans nouvelle version. Seulement sur demande : rien d'autre ne la demande à GitHub, sauf si vous activez la vérification quotidienne avec --mode on
help-sources-update-mode = Active ou désactive la vérification quotidienne de la liste (par défaut : désactivée) et ne fait rien d'autre
help-sources-update-remove = Supprime la liste enregistrée ; dès le prochain démarrage, les adresses du programme font foi
help-sources-corrections = Affiche la liste de corrections enregistrée et ce que chaque correction fait à ce programme
help-sources-corrections-json = Afficher le résultat en JSON
help-arg-adapters-from-project = Charger aussi les adaptateurs du dossier `adapters` du dossier de projet. freemapper nomme chacun avec les hôtes qu'il interroge. À utiliser pour un projet de confiance : un adaptateur fait interroger à freemapper les serveurs qu'il nomme

## The summary of a run, for the MCP server and the area page

cli-internal-error = erreur interne : { $message }
cli-pdf-sources-invalid = pdf_sources doit être auto, full ou credit-only, pas { $value }
cli-view-unknown = une vue est no-paths, brown-only, no-north-lines, no:<codes> ou only:<codes>, pas { $value }
cli-view-empty = la liste { $value } ne nomme aucun code de symbole
cli-view-bad-code = { $value } n'est pas un code de symbole comme 505 ou 501.6
cli-summary-training-view = { $map } et { $pdf } écrits (une vue de la carte d'entraînement)
cli-summary-kept-training-view = Fichiers de la vue { $view } de la carte d'entraînement conservés ; passez --force pour les renouveler
cli-summary-training-view-failed = La vue de la carte d'entraînement a échoué : { $error }
cli-estimate-disk-unreadable = impossible de lire l'espace disque libre de { $path } ({ $message })
cli-summary-wrote-map = Carte { $path } écrite
cli-summary-kept-map = Carte existante { $path } conservée (elle peut contenir le travail du cartographe)
cli-summary-map-lacks-templates = La carte ne référence pas encore les modèles d'image. Lancez le pipeline avec force pour la remplacer et conserver l'ancienne sous le nom .omap.bak.
cli-summary-map-lacks-buildings = La carte ne contient pas encore les bâtiments ALKIS. Lancez le pipeline avec force pour les ajouter ; il remplace la carte et conserve l'ancienne sous le nom .omap.bak.
cli-summary-map-lacks-canopies = La carte n'a pas les auvents et abris à voitures ALKIS (522) qui sont désormais des objets de la carte. Lancez le pipeline avec force pour les ajouter ; il remplace la carte et conserve l'ancienne sous le nom .omap.bak.
cli-summary-map-light = La carte de base : { $joined } lignes réunies, { $left } objets cachés ou trop petits laissés aux couches de candidats (rien déplacé)
cli-summary-map-light-kept = Les couches de candidats contiennent maintenant aussi { $left } objets ou tronçons que cette version retire de la carte de base (marqués freemapper:generalized). La carte conservée les a encore, ils apparaissent donc deux fois. Lancez le pipeline avec force pour nettoyer la carte ; il conserve l'ancienne sous le nom .omap.bak.
cli-summary-map-old-candidates = La carte nomme encore templates/candidates.omap ; cette exécution l'a rempli de tous les candidats. Les candidats existent désormais aussi en sept couches, un fichier chacune (templates/candidates-*.omap). La carte les liste après une exécution avec force (l'ancienne carte reste sous le nom .omap.bak), ou le cartographe les ajoute dans OOM (Modèles > Ouvrir un modèle…).
cli-summary-paper-frame-kept-map = Les cadres papier ont été écrits dans le modèle templates/paper-frame.omap. La carte existante n'est pas modifiée et ne le référence donc pas : le cartographe l'ouvre dans OpenOrienteering Mapper (Modèles > Ouvrir un modèle…), ou la carte le référence après une exécution avec force (l'ancienne carte reste sous le nom .omap.bak).
cli-summary-wrote-ocad = Carte OCAD { $path } écrite ({ $count } objets)
cli-summary-kept-ocad = Carte OCAD existante { $path } conservée (elle peut contenir le travail du cartographe ; force la remplace)
cli-summary-ocad-old-candidates = La carte OCAD nomme encore { $path } comme carte de fond ; cette exécution l'a remplie de tous les candidats. Les candidats existent désormais aussi en sept couches, un fichier chacune (-candidates-*.ocd). La carte OCAD les liste après une exécution avec force (l'ancienne carte reste sous le nom .ocd.bak), ou le cartographe les ajoute dans OCAD comme cartes de fond.
cli-summary-ocad-left-out = Omis de la carte OCAD : { $skipped } objets et { $dropped } parties dégénérées
cli-summary-ocad-symbols = Dans OCAD, choisissez Symbole > Charger les couleurs et symboles de… et sélectionnez le jeu de symboles officiel{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , le jeu scolaire IOF 2019
       *[other] {" "}ISSprOM 2019
    } ; il remplace par numéro les symboles provisoires.
cli-summary-ocad-failed = l'export OCAD a échoué : { $error }
cli-summary-pdf-without-credits = PDF sans mention des sources : l'ODbL et dl-de/by-2-0 demandent de nommer les sources quand la carte est transmise ; voir ATTRIBUTION.txt
cli-summary-wrote-pdf = Carte { $path } écrite (sans candidats)
cli-summary-pdf-failed = l'export PDF a échoué : { $error }
cli-summary-wrote-pdf-candidates = Impression de contrôle { $path } écrite (la carte avec tous les candidats, à leur position exacte)
cli-summary-pdf-candidates-failed = l'export de l'impression de contrôle a échoué : { $error }
cli-summary-training-failed = l'export de la carte d'entraînement a échoué : { $error }
cli-summary-kept-training = Carte d'entraînement existante { $path } conservée, pour qu'une exécution n'écrase pas une carte imprimée ou transmise ; supprimez-la ou activez force pour la renouveler

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = aucun dossier personnel sur ce système ; utilisez un chemin absolu
mcp-error-relative-path = { $path } est un chemin relatif ; utilisez un chemin absolu ou un chemin commençant par ~
mcp-error-home-folder = { $path } est le dossier personnel lui-même ; utilisez un dossier à l'intérieur, p. ex. ~/maps/school
mcp-error-no-project = aucun projet freemapper dans { $path } ; utilisez d'abord create_project
mcp-error-unknown-kind = type inconnu { $kind } ; utilisez "schoolyard", "sprint" ou "forest"
mcp-error-reading = lecture de { $path } : { $error }
mcp-error-one-area-source = indiquez exactement l'un de osm_id (de find_place) ou geojson
mcp-error-page-open = la page de sélection de la zone pour ce projet est encore ouverte ; demandez à la personne d'enregistrer la zone et de cliquer sur Terminé dans la page, puis lancez le pipeline
mcp-error-no-job = aucune tâche { $job } ; start_pipeline et permits_for_area renvoient les identifiants de tâche de cette session
mcp-error-job-running = { $job } est encore en cours pour { $path } ; un seul pipeline s'exécute à la fois, car les projets partagent le cache de téléchargement. Lancez celui-ci quand project_status indique finished ou failed
mcp-error-page-start = impossible de démarrer la page : { $error }
mcp-created-project = Projet { $path } créé pour { $area } ({ $kind }, 1:{ $scale }). Ensuite : estimate_download, puis start_pipeline.
mcp-created-project-given-area = Projet { $path } créé pour la zone indiquée ({ $kind }, 1:{ $scale }). Ensuite : estimate_download, puis start_pipeline.
mcp-page-already-open = La page de ce dossier est déjà ouverte ; elle garde les réglages avec lesquels elle a été ouverte.
mcp-page-next = Demandez à la personne de choisir la zone dans la page, de cliquer sur Enregistrer, puis sur Terminé.
mcp-preview-of = Aperçu de { $layer }
mcp-permits-next = Les notes d'autorisation sont en cours de création. Interrogez project_status avec ce job_id ; une fois terminé, permits_md contient le texte de permits.md. Transmettez-le comme indication, pas comme avis juridique.
mcp-tool-check-update = Indique s'il existe une version plus récente de freemapper : sa version, sa date de publication, ses notes de version et la taille du téléchargement, ou que celle-ci est à jour. N'installe rien.
mcp-tool-create-project = Crée un dossier de projet pour une zone de carte. Indiquez osm_id (de find_place) ou un fichier GeoJSON. kind vaut schoolyard (1:1000, par défaut), sprint (1:4000) ou forest (1:15000).
mcp-param-create-project-dir = Nouveau dossier de projet (chemin absolu ou commençant par ~).
mcp-param-create-project-geojson = À la place de osm_id : un fichier GeoJSON avec le polygone de la zone de carte (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (par défaut, 1:1000), "sprint" (1:4000) ou "forest" (1:15000).
mcp-param-create-project-osm-id = Objet OSM de la zone de carte issu de find_place, p. ex. "way/992423119".
mcp-param-create-project-part = Avec osm_id : quelle partie d'une zone de plusieurs parties séparées utiliser (1 est la plus grande).
mcp-param-create-project-scale = Dénominateur de l'échelle : schoolyard 1000 (par défaut) ; sprint 4000 (par défaut) ou 5000 ; forest 15000 (par défaut), 10000 ou 7500. Les autres valeurs sont refusées ; pour 1:3000, dessinez à 4000, freemapper-print agrandit à l'impression.
mcp-param-create-project-title = Titre de la carte pour les informations périphériques (par défaut : le nom du dossier).
mcp-tool-estimate-download = Ce que start_pipeline va télécharger pour un projet : les tuiles, chaque téléchargement avec sa taille en Mo, s'il est en cache et s'il tient sur le disque, les Mo restant à télécharger, et sous disk l'espace libre des volumes du cache et du projet, avec des messages pour ce qui ne tient pas (à communiquer à l'utilisateur).
mcp-param-estimate-download-dir = Dossier de projet (chemin absolu ou commençant par ~).
mcp-tool-find-place = Recherche une école, un parc ou un autre lieu par son nom (OpenStreetMap Nominatim). Renvoie jusqu'à 5 correspondances, les zones d'abord ; seules les correspondances avec has_area peuvent servir de zone de carte. Une liste vide signifie que rien n'a été trouvé.
mcp-param-find-place-query = Nom de l'école ou du lieu, p. ex. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Installe la version la plus récente de freemapper par-dessus ce programme. À appeler seulement après l'accord de la personne, avec confirm true. La nouvelle version s'applique une fois que l'assistant a redémarré le serveur freemapper.
mcp-param-install-update-confirm = true une fois que la personne a accepté d'installer la version plus récente ; sans cela, rien n'est installé.
mcp-tool-list-layers = Les couches d'un projet : la carte, les sept couches de candidats, les cadres papier (paper-frame, seulement si des cadres ont été gardés sur la page de la zone) et les modèles raster, chacune avec son fichier, si elle est déjà créée et quelle étape la crée.
mcp-param-list-layers-dir = Dossier de projet (chemin absolu ou commençant par ~).
mcp-tool-open-area-selector = Ouvre dans le navigateur une page où l'utilisateur dessine la zone de la carte (rectangle ou polygone) ou désigne un lieu, voit les tuiles et la taille du téléchargement, puis l'enregistre ; l'enregistrement crée le dossier de projet ou modifie sa zone. Renvoie l'URL à donner à l'utilisateur. La page s'exécute jusqu'à ce que l'utilisateur clique sur Terminé ; un nouvel appel pour le même dossier renvoie la même URL.
mcp-param-open-area-selector-dir = Dossier de projet (chemin absolu ou commençant par ~) ; créé quand l'utilisateur enregistre.
mcp-param-open-area-selector-kind = Type de carte d'un nouveau projet : "schoolyard" (par défaut, 1:1000), "sprint" (1:4000) ou "forest" (1:15000).
mcp-param-open-area-selector-ocad = Le panneau d'exécution démarre avec le projet de départ OCAD coché. Par défaut false.
mcp-param-open-area-selector-offline = Le panneau d'exécution démarre avec « Cache seulement, pas de réseau » coché. Par défaut false.
mcp-param-open-area-selector-open-browser = Ouvre aussi la page dans le navigateur par défaut de cet ordinateur. Par défaut true.
mcp-param-open-area-selector-pdf = Le panneau d'exécution démarre avec le PDF imprimable coché. Par défaut false.
mcp-param-open-area-selector-pdf-candidates = Le panneau d'exécution démarre avec l'impression de contrôle (tous les candidats) cochée. Par défaut false.
mcp-param-open-area-selector-pdf-scale-bar = Le panneau d'exécution démarre avec la barre d'échelle sur les PDF cochée. Par défaut false.
mcp-param-open-area-selector-pdf-sources = Lignes de sources sur les PDF avec lesquelles le panneau démarre : "auto" (par défaut), "full" ou "credit-only", comme pour start_pipeline.
mcp-param-open-area-selector-permits = Le panneau d'exécution démarre avec les notes d'autorisation cochées. Par défaut false.
mcp-param-open-area-selector-scale = Dénominateur de l'échelle d'un nouveau projet : schoolyard 1000 (par défaut) ; sprint 4000 (par défaut) ou 5000 ; forest 15000 (par défaut), 10000 ou 7500. Les autres valeurs sont refusées.
mcp-param-open-area-selector-title = Titre de la carte d'un nouveau projet.
mcp-param-open-area-selector-training = Le panneau d'exécution démarre avec la carte d'entraînement cochée. Par défaut false.
mcp-tool-preview-layer = Un aperçu PNG (800 px au plus) d'une couche : map (l'image des objets de la carte écrite par la dernière exécution, sans les candidats), ou un modèle raster : orthophoto, hillshade, slope, relief, ndsm, ou k5 à Berlin.
mcp-param-preview-layer-dir = Dossier de projet (chemin absolu ou commençant par ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" ou "k5" (Berlin seulement).
mcp-tool-project-status = État d'une tâche de start_pipeline ou permits_for_area : running, finished ou failed, les dernières lignes de progression, les avertissements, les fichiers écrits, et le résultat ou l'erreur. Une tâche permits_for_area terminée porte aussi le texte de permits.md sous permits_md.
mcp-param-project-status-job-id = Identifiant de tâche de start_pipeline ou permits_for_area, p. ex. "job-1".
mcp-tool-permits-for-area = Crée les notes d'autorisation d'un projet : les unités administratives de la zone de la carte, les zones protégées qui s'y trouvent et qui gère le terrain, avec les autorités à interroger ; une indication, pas un avis juridique. Écrit permits.md et permits.json dans le dossier. S'exécute en arrière-plan et renvoie un job_id ; interrogez project_status, dont la réponse porte le texte de permits.md sous permits_md quand la tâche est terminée. Une seule tâche s'exécute à la fois.
mcp-param-permits-for-area-dir = Dossier de projet (chemin absolu ou commençant par ~).
mcp-param-permits-for-area-offline = Lire seulement les données enregistrées dans le projet par une exécution précédente, sans accès au réseau. Par défaut false.
mcp-tool-start-pipeline = Télécharge les données, calcule les modèles et écrit le projet de départ (fetch, derive, export). S'exécute en arrière-plan et renvoie un job_id ; interrogez project_status. Un seul pipeline s'exécute à la fois. Conserve une carte existante sauf si force vaut true.
mcp-param-start-pipeline-dir = Dossier de projet (chemin absolu ou commençant par ~).
mcp-param-start-pipeline-force = Remplace un fichier de carte existant et conserve l'ancien sous le nom .omap.bak. Par défaut false.
mcp-param-start-pipeline-ocad = Écrit aussi un point de départ OCAD dans le dossier ocad/ (format OCAD 12, pour les utilisateurs qui éditent dans OCAD). Par défaut false.
mcp-param-start-pipeline-offline = Utilise seulement le cache de téléchargement, sans accès au réseau. Par défaut false.
mcp-param-start-pipeline-pdf = Écrit aussi <name>.pdf, la carte prête à imprimer (les objets de la carte seulement, sans candidats). Par défaut false.
mcp-param-start-pipeline-pdf-candidates = Écrit aussi <name>-candidates.pdf, l'impression de contrôle : la carte avec tous les candidats, à leur position exacte (à vérifier sur le terrain). Par défaut false.
mcp-param-start-pipeline-pdf-scale-bar = Dessine la barre d'échelle sur les PDF. Par défaut false (les cartes allemandes n'en ont généralement pas ; le nombre d'échelle reste dans le cartouche).
mcp-param-start-pipeline-pdf-sources = Lignes de sources sur les PDF : "auto" (par défaut ; la liste complète, raccourcie seulement si elle dépasse 5 lignes), "full" (jamais raccourcie) ou "credit-only" (seulement la mention de crédit de freemapper ; l'ODbL et dl-de/by-2-0 exigent alors de nommer les sources quand la carte est transmise). ATTRIBUTION.txt liste toujours toutes les sources.
mcp-param-start-pipeline-permits = Écrit aussi les notes d'autorisation (permits.md et permits.json) après la carte : unités administratives, zones protégées et qui gère le terrain. Un échec à ce stade est un avertissement, pas une tâche échouée. Par défaut false.
mcp-param-start-pipeline-training = Écrit aussi la carte d'entraînement <name>-training.omap et son PDF <name>-training.pdf : la carte et tous les candidats, nettoyés pour être imprimés tels quels ; avec ocad aussi ocad/<name>-training.ocd. Une carte d'entraînement existante est conservée sauf si force vaut true. Par défaut false.
mcp-param-start-pipeline-training-views = Avec training : écrit aussi la carte d'entraînement sans ce qu'une vue omet, un jeu de fichiers par entrée : "no-paths", "brown-only", "no-north-lines", ou une liste comme "no:505,506" ou "only:101,102", combinée avec +. Les fichiers sont <name>-training-<view>.omap et son PDF.
mcp-instructions = freemapper crée des projets de départ OpenOrienteering Mapper pour des cartes de course d'orientation (cour d'école 1:1000, sprint 1:4000, forêt 1:15000) à partir des géodonnées officielles de { $states } et d'OSM. Marche à suivre : find_place (chercher l'école ou la zone par son nom ; choisir un résultat avec has_area) → create_project (un nouveau dossier, chemin absolu, avec l'osm_id) → estimate_download (dire à l'utilisateur combien sera téléchargé) → start_pipeline (renvoie un identifiant de tâche ; le téléchargement peut prendre des minutes) → project_status (interroger toutes les quelques secondes jusqu'à ce que l'état soit finished ou failed) → list_layers et preview_layer (montrer les modèles à l'utilisateur). L'utilisateur ouvre ensuite le fichier de carte dans OpenOrienteering Mapper. Quand aucun lieu ne convient ou que l'utilisateur veut dessiner la zone, open_area_selector ouvre dans le navigateur une page où l'utilisateur dessine ou choisit la zone et l'enregistre (cela crée le projet) ; continuez avec estimate_download dès que l'utilisateur dit que c'est enregistré. Les chemins doivent être absolus ou commencer par ~. Un seul pipeline s'exécute à la fois. permits_for_area crée les notes d'autorisation d'un projet (unités administratives, zones protégées, qui gère le terrain ; une indication, pas un avis juridique) comme une tâche, à la manière de start_pipeline. check_update indique s'il existe un freemapper plus récent ; install_update l'installe, seulement avec confirm: true après l'accord de la personne, et l'assistant redémarre ensuite le serveur freemapper.

cli-contour-interval-saved = Équidistance enregistrée dans freemapper.toml : { $interval } m ; derive trace les courbes de niveau avec elle
cli-contour-interval-auto = Équidistance dans freemapper.toml : automatique (celle du type de carte ; une carte de forêt a 2,5 m en terrain plat et 10 m en terrain raide)
cli-steep-slope-saved = Seuil de pente enregistré dans freemapper.toml : { $slope }° ; derive cherche avec lui les talus et les falaises
cli-steep-slope-auto = Seuil de pente dans freemapper.toml : par défaut (environ 27°)
cli-face-slope-saved = Seuil de paroi enregistré dans freemapper.toml : { $slope }° ; derive cherche avec lui les surfaces de paroi rocheuse
cli-face-slope-auto = Seuil de paroi dans freemapper.toml : par défaut (70°)
help-arg-contour-interval = Équidistance en mètres, p. ex. `2.5`, ou `auto` pour celle du type de carte (carte de forêt : 5 m, 2,5 m en terrain plat, 10 m en terrain raide). Enregistrée dans freemapper.toml ; derive trace les courbes de niveau avec elle
help-arg-steep-slope = Angle de pente en degrés à partir duquel le terrain est considéré comme raide lors de la recherche des talus et des falaises, p. ex. `55`, ou `auto` pour la valeur par défaut de 27°. Relève-le (55 à 65) dans les villes de rochers et les gorges, où presque chaque pente est raide et où talus et falaises se fondent en une seule tache. Enregistré dans freemapper.toml ; derive cherche avec lui les talus et les falaises
help-arg-face-slope = Angle de pente en degrés à partir duquel le terrain est une paroi rocheuse (surface noire, symbole 206), par ex. `75`, ou `auto` pour la valeur par défaut de 70°. 65 donne plus de surfaces et de plus grandes, aussi sur les pentes raides ; 75 ne garde que les vraies parois. Enregistré dans freemapper.toml ; derive cherche avec lui les parois rocheuses
cli-content-saved = Réglage du contenu enregistré dans freemapper.toml : { $summary }
cli-content-invalid = Impossible de lire le réglage du contenu : { $detail }
help-arg-content = Ce qui va directement sur la carte : `safe` laisse chaque objet là où l'assemblage de la carte le place, `balanced` (par défaut) met sur la carte les objets du cadastre et des registres, `full` met tout sur la carte. Le choix est enregistré dans freemapper.toml
help-arg-content-set = Envoyer un groupe d'objets vers `map`, `candidates` ou `off`, sous la forme `GROUPE=LIEU`, p. ex. `walls=map` ; peut être répété. Groupes : walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Enregistré dans freemapper.toml
mcp-param-create-project-content = Ce qui va directement sur la carte : "safe", "balanced" (par défaut) ou "full". Enregistré dans le projet. balanced met sur la carte les objets du cadastre et des registres, full met tout dessus.
mcp-param-create-project-content-set = Groupes envoyés ailleurs que selon le profil, sous la forme "groupe=lieu" avec le lieu map, candidates ou off, p. ex. "walls=map". Groupes : walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Équidistance en mètres, p. ex. 2.5 ; 0 revient au choix automatique (celui du type de carte ; une carte de forêt a 2,5 m en terrain plat et 10 m en terrain raide). Enregistrée dans le projet ; le prochain derive trace les courbes de niveau avec elle.
mcp-param-create-project-steep-slope = Angle de pente en degrés à partir duquel le terrain est considéré comme raide lors de la recherche des talus et des falaises, p. ex. 55 ; 0 revient à la valeur par défaut de 27°. Relève-le (55 à 65) dans les villes de rochers et les gorges, où presque chaque pente est raide et où talus et falaises se fondent en une seule tache. Enregistré dans le projet ; le prochain derive cherche avec lui les talus et les falaises.
mcp-param-create-project-face-slope = Angle de pente en degrés à partir duquel le terrain est une paroi rocheuse (surface noire, symbole 206), par ex. 75 ; 0 revient à la valeur par défaut de 70°. 65 donne plus de surfaces et de plus grandes, aussi sur les pentes raides ; 75 ne garde que les vraies parois. Enregistré dans le projet ; le prochain derive cherche avec lui les parois rocheuses.
mcp-param-start-pipeline-content = Modifie avant l'exécution ce qui va directement sur la carte : "safe", "balanced" ou "full". Enregistré dans le projet. Par défaut : le réglage du projet, sinon balanced.
mcp-param-start-pipeline-content-set = Groupes envoyés ailleurs que selon le profil, sous la forme "groupe=lieu" avec le lieu map, candidates ou off, p. ex. "fences=candidates". Enregistré dans le projet.
mcp-param-start-pipeline-contour-interval = Équidistance en mètres, p. ex. 2.5 ; 0 revient au choix automatique (celui du type de carte ; une carte de forêt a 2,5 m en terrain plat et 10 m en terrain raide). Enregistrée dans le projet ; le prochain derive trace les courbes de niveau avec elle.
mcp-param-start-pipeline-steep-slope = Angle de pente en degrés à partir duquel le terrain est considéré comme raide lors de la recherche des talus et des falaises, p. ex. 55 ; 0 revient à la valeur par défaut de 27°. Relève-le (55 à 65) dans les villes de rochers et les gorges, où presque chaque pente est raide et où talus et falaises se fondent en une seule tache. Enregistré dans le projet ; le prochain derive cherche avec lui les talus et les falaises.
mcp-param-start-pipeline-face-slope = Angle de pente en degrés à partir duquel le terrain est une paroi rocheuse (surface noire, symbole 206), par ex. 75 ; 0 revient à la valeur par défaut de 70°. 65 donne plus de surfaces et de plus grandes, aussi sur les pentes raides ; 75 ne garde que les vraies parois. Enregistré dans le projet ; le prochain derive cherche avec lui les parois rocheuses.
mcp-error-content = Impossible de lire le réglage du contenu : { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Écrire les notes d'autorisation d'un projet (permits.md et permits.json) : les unités administratives, zones protégées et gestionnaires que touche la zone de la carte, et à qui il faut généralement s'adresser. La commande interroge le BfN et le BKG et, là où le Land de la zone de la carte a ses propres services ouverts, ceux-ci via les hôtes de son adaptateur (liste dans le manuel, chapitre 12) ; en Rhénanie-du-Nord-Westphalie elle télécharge pour cela environ 32 Mo la première fois, en Brandebourg environ 4 Mo, ensuite seulement des fichiers plus récents. Elle lit l'extrait OSM déjà en cache ; --offline utilise ce qui a été téléchargé avant
help-permits-dir = Dossier de projet
help-permits-offline = Utiliser seulement les données téléchargées avant, sans accès au réseau
help-run-permits = Écrire aussi les notes d'autorisation (permits.md et permits.json) après la carte ; interroge le BfN, le BKG et les services ouverts du Land (manuel, chapitre 12). Un échec à ce stade est un avertissement et la carte reste
cli-wrote-permits = Notes d'autorisation { $path } écrites
cli-summary-wrote-permits = Notes d'autorisation { $path } écrites
cli-permits-failed = les notes d'autorisation n'ont pas pu être écrites : { $error }
cli-permits-source-left-out = les notes d'autorisation omettent { $source } : { $reason }
cli-permits-not-read = elle n'a pas été lue
error-permits-stored = { $path } : { $message } ; téléchargez à nouveau les notes d'autorisation en ligne
error-permits-nothing-read = aucune source des notes d'autorisation n'a pu être lue ({ $missing }) ; téléchargez-les d'abord en ligne
error-permits-missing-not-read = { $source } : non lue

permits-title = Notes d'autorisation
permits-disclaimer = Une indication, pas un avis juridique. Les autorités compétentes décident. Pour la date et la précision des données, voir le tableau sous « Sources et date des données » ; des zones peuvent manquer ou être périmées (BfN : fusionné une fois par an, « Nicht für Planungszwecke geeignet. », c'est-à-dire non adapté à la planification).
permits-disclaimer-regional = Une indication, pas un avis juridique. Les autorités compétentes décident. Pour la date et la précision des données, voir le tableau sous « Sources et date des données » ; des zones peuvent manquer ou être périmées.
permits-bfn-merge = Le BfN fusionne une fois par an les données des Länder et les marque comme non adaptées à la planification. Les autorités du Land décident.
permits-created = Créées le { $date } pour la zone de la carte d'une { $kind ->
        [schoolyard] carte de cour d'école
        [forest] carte de forêt
       *[other] carte de sprint
    } ({ $area } ha).
permits-section-admin = Unités administratives
permits-section-protected = Zones protégées (BfN)
permits-section-osm-protected = Autres zones protégées d'après OpenStreetMap
permits-section-operators = Exploitants et propriétaires d'après OpenStreetMap
permits-section-hints = Indications
permits-section-sources = Sources et date des données
permits-none-admin = VG250 ne nomme aucune unité administrative pour la zone de la carte.
permits-none-protected = Selon le BfN, aucune zone protégée ne se trouve dans la zone de la carte.
permits-none-osm-protected = OpenStreetMap ne nomme dans la zone de la carte aucune zone protégée que le BfN ne liste pas déjà.
permits-none-operators = OpenStreetMap ne nomme ni exploitant ni propriétaire pour la zone de la carte.
permits-none-hints = Aucune indication.
permits-section-not-read = Les données de { $source } n'ont pas été lues, cette section est donc vide, bien qu'il puisse y avoir des entrées.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Limites administratives au 1:250 000 (VG250) du BKG
        [bfn] Zones protégées d'Allemagne du BfN
       *[osm] OpenStreetMap, d'après l'extrait que freemapper conserve dans son cache
    }
permits-level = { $level ->
        [land] Land
        [kreis] Arrondissement (Kreis)
        [verwaltungsgemeinschaft] Amt ou Verwaltungsgemeinschaft
       *[gemeinde] Commune (Gemeinde)
    }
permits-admin-line = { $level } : { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Touchées en bordure (moins de 0,5 % de la zone de la carte) :
permits-at-edge-generalised = VG250 est généralisé à l'échelle 1:250 000. Une unité qui touche seulement la bordure peut en être un effet.
permits-kind = { $kind ->
        [nationalpark] Parc national
        [nationales-naturmonument] Monument naturel national
        [naturschutzgebiet] Réserve naturelle (Naturschutzgebiet)
        [ffh] Zone FFH (Natura 2000)
        [vogelschutzgebiet] Zone de protection des oiseaux (Natura 2000)
        [biosphaerenreservat] Réserve de biosphère
        [biosphaerenreservat-zone] Zone d'une réserve de biosphère
        [landschaftsschutzgebiet] Zone de protection du paysage (Landschaftsschutzgebiet)
       *[naturpark] Parc naturel
    }
permits-protected-line = { $kind } « { $name } », { $share }
permits-protected-date = Classée le { $date }
permits-protected-title = Base juridique : { $title }
permits-protected-link = Texte juridique : { $url }
permits-protected-site-link = Page du site (Natura 2000) : { $url }
permits-protected-year = Année des données au BfN : { $year }
permits-osm-line = { $kind } : { $name }, { $share } (OSM { $id })
permits-osm-protect-class = classe de protection { $class }
permits-quoted = « { $text } »
permits-osm-unnamed = sans nom
permits-osm-operator = Exploitant d'après OSM : { $operator }
permits-osm-enclosing-gap = Les zones OpenStreetMap qui entourent toute la zone de la carte sans qu'un de leurs coins s'y trouve peuvent manquer ici, de même que les zones protégées cartographiées dans OSM comme une relation `type=boundary`. Les parties issues du BfN et de VG250 ne sont pas concernées.
permits-osm-enclosing-gap-short = Les surfaces d'OpenStreetMap qui entourent toute la zone de la carte sans qu'un de leurs coins s'y trouve peuvent manquer ici.
permits-operator-line = { $name } ({ $roles ->
        [operator] exploitant
        [owner] propriétaire
       *[both] exploitant et propriétaire
    }) : { $objects }
permits-column-source = Source
permits-column-state = État
permits-column-credit = Crédit et licence
permits-state-read = lue, téléchargée le { $date }
permits-state-read-osm = lue, extrait OSM du { $date }
permits-state-unreachable = inaccessible ({ $reason })
permits-state-not-read = non lue
permits-no-credit = non utilisé
permits-hint-city-state = La zone de la carte se trouve à Berlin ou à Hambourg. Là, le Bezirksamt (office d'arrondissement) ou une autorité du Land est généralement compétent. Ces notes ne déterminent pas encore le Bezirk.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Les zones protégées relèvent de l'autorité inférieure de protection de la nature (untere Naturschutzbehörde). Ici, c'est la ville de { $kreis } elle-même.
        [no] Les zones protégées relèvent de l'autorité inférieure de protection de la nature (untere Naturschutzbehörde), généralement le { $kreis }.
       *[unknown] Les zones protégées relèvent de l'autorité inférieure de protection de la nature compétente (untere Naturschutzbehörde), généralement le Landkreis ou la kreisfreie Stadt.
    }
permits-hint-national-park = Un parc national a ses propres règles. Adressez-vous tôt à l'administration du parc national.
permits-hint-nsg-paths = Dans une réserve naturelle, il faut généralement rester sur les chemins. Pour une course hors des chemins, adressez-vous à l'autorité de protection de la nature.
permits-hint-natura2000 = Une zone FFH ou une zone de protection des oiseaux (Natura 2000) touche la zone de la carte. Pour un événement, l'autorité de protection de la nature peut demander une évaluation de ses effets.
permits-hint-protected-areas-pl = Des zones ou des objets protégés au titre de la protection de la nature se trouvent dans la zone de la carte. Pour un parc national, un parc paysager ou une réserve naturelle, l'interlocuteur habituel est l'administration du parc ou de la réserve ; pour les autres zones et objets, la direction régionale de la protection de l'environnement (RDOŚ) de la voïvodie peut généralement dire qui est compétent. Demandez suffisamment tôt si vous prévoyez un événement à cet endroit.
permits-hint-order-office = { $amt ->
        [none] Pour les rues, places et autres terrains publics, l'Ordnungsamt (office de l'ordre public) de la Gemeinde { $gemeinde } est généralement compétent.
       *[other] Pour les rues, places et autres terrains publics, l'Ordnungsamt (office de l'ordre public) est généralement compétent : { $amt }, pour la Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Exploitant d'un parc ou d'un jardin dans la zone de la carte d'après OpenStreetMap : { $operator }. Demandez-lui avant de l'utiliser.
permits-hint-school-operator = Exploitant de l'école ou de l'établissement d'après OpenStreetMap : { $operator }. Pour son terrain, adressez-vous à la direction de l'école ou à cet organisme.
permits-section-districts = Bezirke (arrondissements, { $source })
permits-section-green-spaces = Espaces verts ({ $source })
permits-section-owners = Terrains du Land et du Bund ({ $source })
permits-section-monuments = Monuments ({ $source })
permits-section-protected-objects = Monuments naturels et autres objets protégés ({ $source })
permits-none-districts = Selon { $source }, la zone de la carte ne se trouve dans aucun Bezirk.
permits-none-green-spaces = Selon { $source }, aucun espace vert public ne se trouve dans la zone de la carte.
permits-none-owners = Selon { $source }, aucune parcelle du Land ou du Bund ne se trouve dans la zone de la carte.
permits-none-monuments = Selon { $source }, aucun monument ne se trouve dans la zone de la carte.
permits-none-protected-objects = Selon { $source }, aucun des objets protégés qu'il contient ne se trouve dans la zone de la carte.
permits-none-sampled = Aux points interrogés, { $source } ne nomme rien ici.
permits-gaps = Les données de { $source } ne sont pas complètes : { $gaps }.
permits-sampled = { $source } ne répond que pour des points isolés. freemapper a interrogé une grille de points sur la carte : les parts sont des estimations, et ce qui se trouve entre les points peut manquer.
permits-share-line = { $name } : { $share }
permits-count-line = { $name } : { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = affectation : { $dedication }
permits-playgrounds-none = Aucune aire de jeux publique ne se trouve dans la zone de la carte.
permits-playgrounds-one = Une aire de jeux publique se trouve dans la zone de la carte. Elle n'est pas listée séparément.
permits-playgrounds-many = { $count } aires de jeux publiques se trouvent dans la zone de la carte. Elles ne sont pas listées une par une.
permits-owner-other = Autres propriétaires publics
permits-owners-rest = Pour le reste de la zone, les données ne nomment aucun propriétaire.
permits-monuments-total = Monuments entièrement ou partiellement dans la zone de la carte : { $count }.{ $links ->
        [yes] {" "}Les liens mènent à leurs entrées dans la liste des monuments.
       *[no] {""}
    }
permits-monuments-links-left = { $count } entrées de plus sont seulement comptées ici, sans lien.
permits-object-kind = { $kind ->
        [natural-monument] Monument naturel (Naturdenkmal)
        [protected-landscape-part] Élément de paysage protégé (geschützter Landschaftsbestandteil)
        [biotope] Biotope légalement protégé (geschütztes Biotop)
        [national-park] Parc national (park narodowy)
        [park-buffer-zone] Zone tampon d'un parc (otulina)
        [landscape-park] Parc paysager (park krajobrazowy)
        [nature-reserve] Réserve naturelle (rezerwat przyrody)
        [protected-landscape-area] Zone de paysage protégé (obszar chronionego krajobrazu)
        [natura2000-habitats] Site Natura 2000 pour les habitats (SOO)
        [natura2000-birds] Site Natura 2000 pour les oiseaux (OSO)
        [ecological-site] Site écologique (użytek ekologiczny)
        [landscape-complex] Ensemble naturel et paysager (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Site de documentation (stanowisko dokumentacyjne)
        [natural-monument-pl] Monument naturel (pomnik przyrody)
        [national-park-cz] Parc national (národní park, NP)
        [protected-landscape-area-cz] Zone de paysage protégé (chráněná krajinná oblast, CHKO)
        [national-nature-reserve] Réserve naturelle nationale (národní přírodní rezervace, NPR)
        [national-nature-monument] Monument naturel national (národní přírodní památka, NPP)
        [nature-reserve-cz] Réserve naturelle (přírodní rezervace, PR)
        [nature-monument-cz] Monument naturel (přírodní památka, PP)
        [protective-zone-cz] Zone de protection (ochranné pásmo)
        [nature-zone-cz] Zone de protection de la nature (zóna ochrany přírody)
        [natura2000-habitats-cz] Site Natura 2000 pour les habitats (evropsky významná lokalita, EVL)
        [natura2000-birds-cz] Site Natura 2000 pour les oiseaux (ptačí oblast, PO)
        [memorial-tree] Arbre remarquable (památný strom)
       *[special-rules] Zone à règles particulières
    }
permits-object-count = { $count } objets
permits-object-citations = Bases juridiques :
permits-source-note = { $source } : « { $note } »
permits-hint-district-office = { $offices ->
        [hamburg] Pour les rues, places et espaces verts du Bezirk { $district }, le Bezirksamt { $district } est généralement compétent, plus précisément le Fachamt Management des öffentlichen Raumes (office de gestion de l'espace public).
       *[berlin] Pour les rues, places et espaces verts du Bezirk { $district }, l'Ordnungsamt (office de l'ordre public) et le Straßen- und Grünflächenamt (office des routes et espaces verts) du Bezirksamt { $district } sont généralement compétents.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Pour les monuments de la zone de la carte, le Denkmalschutzamt Hamburg (office des monuments) est compétent.
       *[berlin] Pour les monuments du Bezirk { $district }, l'autorité inférieure des monuments (untere Denkmalschutzbehörde) du Bezirksamt { $district } est compétente.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Les données disponibles ici ne permettent pas de dire avec certitude si des monuments classés se trouvent dans la zone de la carte.
       *[yes] Il y a des monuments classés dans la zone de la carte.
    } Pour savoir si une course les touche, par exemple avec un poste à un monument ou un tronçon à travers un site protégé, c'est l'autorité inférieure des monuments (untere Denkmalschutzbehörde) qui décide{ $kreisfrei ->
        [yes] . Ici, c'est la ville de { $kreis } elle-même.
        [no] , généralement le { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; dans certains Länder, c'est la Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(généralement le Landkreis ou la kreisfreie Stadt, dans certains Länder la Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Les monuments naturels, éléments de paysage protégés et biotopes protégés de la carte relèvent de l'autorité inférieure de protection de la nature (untere Naturschutzbehörde). Ici, c'est la ville de { $kreis } elle-même.
        [no] Les monuments naturels, éléments de paysage protégés et biotopes protégés de la carte relèvent de l'autorité inférieure de protection de la nature (untere Naturschutzbehörde), généralement le { $kreis }.
       *[unknown] Les monuments naturels, éléments de paysage protégés et biotopes protégés de la carte relèvent de l'autorité inférieure de protection de la nature (untere Naturschutzbehörde), généralement le Landkreis ou la kreisfreie Stadt.
    }
permits-hint-biotope = Des biotopes légalement protégés se trouvent dans la zone de la carte{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Tout ce qui peut les détruire ou leur nuire considérablement est interdit (§ 30 BNatSchG). Placez les postes et les tronçons en dehors, ou adressez-vous à l'autorité inférieure de protection de la nature (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Pour les monuments naturels et éléments de paysage protégés du Bezirk { $district }, le Bezirksamt { $district } est généralement compétent.
       *[berlin] Pour les monuments naturels et éléments de paysage protégés du Bezirk { $district }, l'autorité inférieure de protection de la nature (untere Naturschutzbehörde) du Bezirksamt { $district } est compétente.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Selon les données ouvertes du Land, { $share } de la zone de la carte appartient à d'autres propriétaires publics.
       *[no] Selon les données ouvertes du Land, { $share } de la zone de la carte appartient au propriétaire « { $owner } ».
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Les propriétaires du reste de la zone ne peuvent pas être retrouvés à partir de données ouvertes. Avec un intérêt légitime (un organisateur, par exemple), vous pouvez demander un extrait au registre foncier (Grundbuchamt) ; pour les terrains publics, l'Ordnungsamt ou le service des parcs compétent vous aide.
       *[no] Les propriétaires de ces zones ne peuvent pas être retrouvés à partir de données ouvertes. Avec un intérêt légitime (un organisateur, par exemple), vous pouvez demander un extrait au registre foncier (Grundbuchamt) ; pour les terrains publics, l'Ordnungsamt ou le service des parcs compétent vous aide.
    }
permits-section-forest-units = Administration forestière ({ $source })
permits-section-state-forest = Forêt domaniale du Land ({ $source })
permits-none-forest-units = Selon { $source }, la zone de la carte ne se trouve dans le ressort d'aucun office forestier.
permits-section-admin-units = Unités administratives ({ $source })
permits-none-admin-units = Selon { $source }, la zone de la carte ne se trouve dans aucune unité administrative du registre.
permits-section-register-monuments = Monuments du registre national ({ $source })
permits-none-register-monuments = Selon { $source }, aucun monument inscrit ne se trouve dans le rectangle autour de la zone de la carte.
permits-register-monuments-total = Monuments inscrits dans le rectangle autour de la zone de la carte : { $count }. Le registre ne donne pas de position, ces notes ne peuvent donc pas dire où ils se trouvent ; il faut le vérifier sur la carte.
permits-register-monuments-left = { $count } autres sont seulement comptés ici, sans leur nom.
permits-section-register-archaeology = Monuments archéologiques du registre national ({ $source })
permits-none-register-archaeology = Selon { $source }, aucun monument archéologique inscrit ne se trouve dans le rectangle autour de la zone de la carte.
permits-register-archaeology-total = Monuments archéologiques inscrits dans le rectangle autour de la zone de la carte : { $count }. Le registre ne donne pas de position, ces notes ne peuvent donc pas dire où ils se trouvent ; il faut le vérifier sur la carte.
permits-register-archaeology-left = { $count } autres sont seulement comptés ici, sans leur nom.
permits-section-register-history = Monuments de l'histoire ({ $source })
permits-none-register-history = Selon { $source }, aucun monument de l'histoire ne se trouve dans le rectangle autour de la zone de la carte.
permits-register-history-in-area = Monuments de l'histoire dans la zone de la carte : { $count }.
permits-register-history-nearby = Autres monuments de l'histoire dans le rectangle autour de la zone de la carte, mais pas dans la zone de la carte : { $count }.
permits-register-history-left = { $count } autres sont seulement comptés ici, sans leur nom.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-admin-unit-code-ruian = { $name } (RÚIAN { $code })
permits-none-state-forest = Selon { $source }, aucune forêt domaniale du Land ne se trouve dans la zone de la carte.
permits-forest-units-reach = Un office forestier est compétent pour la forêt de son ressort. Son ressort peut aussi englober des villes et des champs.
permits-state-forest-office = compétent : { $office }
permits-hint-forest-office = { $units ->
        [none] Office forestier compétent pour la forêt de la zone de la carte : { $office } ({ $administration }). Contactez-le tôt si vous prévoyez un événement en forêt.
       *[other] Office forestier compétent pour la forêt de la zone de la carte : { $office } ({ $administration }), { $units }. Contactez-le tôt si vous prévoyez un événement en forêt.
    }
permits-hint-forest-owner = { $office ->
        [known] Un événement en forêt exige l'accord des propriétaires forestiers. L'office forestier peut généralement dire à qui appartient la forêt. Dans plusieurs Länder, un événement organisé en forêt doit aussi être déclaré à l'autorité forestière ou autorisé par elle.
       *[unknown] Un événement en forêt exige l'accord des propriétaires forestiers. Dans plusieurs Länder, un événement organisé en forêt doit aussi être déclaré à l'autorité forestière ou autorisé par elle. Ces notes ne nomment pas l'office forestier compétent ici ; l'administration forestière du Land ou l'autorité forestière inférieure peut vous le dire.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Un événement en forêt nécessite généralement l'accord du propriétaire de la forêt. Le district forestier (Nadleśnictwo) nommé ci-dessus peut généralement dire qui s'occupe de la forêt. Une forêt qui n'appartient pas aux Forêts d'État, une forêt privée par exemple, ne figure pas dans les données de ces notes. La forêt qui n'appartient pas à l'État, par exemple une forêt privée ou communale, est placée sous la surveillance du starosta (starosta) pour la gestion forestière (loi sur les forêts, art. 5 ust. 1 pkt 2) ; renseignez-vous alors au starostwo powiatowe.
       *[unknown] Un événement en forêt nécessite généralement l'accord du propriétaire de la forêt. Ces notes ne nomment aucun district forestier des Forêts d'État pour la zone de la carte ; les données ne couvrent que les Forêts d'État, la forêt peut donc appartenir à quelqu'un d'autre, à un propriétaire privé par exemple. Le propriétaire doit être déterminé séparément. La forêt qui n'appartient pas à l'État, par exemple une forêt privée ou communale, est placée sous la surveillance du starosta (starosta) pour la gestion forestière (loi sur les forêts, art. 5 ust. 1 pkt 2) ; renseignez-vous alors au starostwo powiatowe.
    }
permits-hint-monument-register-pl = Des monuments du registre national se trouvent dans le rectangle autour de la zone de la carte. Leurs positions ne figurent pas dans les données ; vérifiez sur la carte où ils se trouvent. Selon l'art. 36 ust. 1 de la loi sur la protection des monuments (ustawa o ochronie zabytków), les travaux de conservation, de restauration et de construction sur un monument inscrit exigent l'autorisation du conservateur des monuments de la voïvodie (Wojewódzki Konserwator Zabytków). Depuis le 3 juin 2026, les travaux de construction aux abords d'un monument, le changement de son usage, les dispositifs ou panneaux posés dessus et d'autres actions pouvant modifier sa substance ou son aspect exigent seulement une déclaration au conservateur (art. 36aa) ; celui-ci peut faire opposition dans les 60 jours et exiger alors une autorisation. La loi ne nomme pas une course ni un poste de contrôle en tant que tels ; demandez au bureau du conservateur si un événement touche un monument ou ses abords.
permits-hint-archaeology-register-pl = Des monuments archéologiques du registre national se trouvent dans le rectangle autour de la zone de la carte. Leurs positions ne figurent pas dans les données, vérifiez donc sur la carte où ils sont. Selon l'art. 36 ust. 1 de la loi sur la protection des monuments (ustawa o ochronie zabytków), les recherches archéologiques (pkt 5), la recherche de monuments mobiles cachés ou abandonnés avec des appareils électroniques ou techniques tels que les détecteurs de métaux (pkt 12) et les travaux de conservation, de restauration ou de construction sur un monument inscrit (pkt 1) exigent l'autorisation du conservateur des monuments de la voïvodie (Wojewódzki Konserwator Zabytków). L'autorisation pour les recherches tombe le 1er janvier 2027 : l'art. 36b permettra alors à une personne majeure de chercher avec l'accord du propriétaire et du possesseur du bien, après une déclaration dans le registre des recherches par l'application du ministère de la culture, et l'art. 36c interdit de telles recherches notamment sur les monuments inscrits et à moins de 10 m de ceux-ci. La loi ne nomme ni une promenade, ni une course, ni un poste de contrôle comme tels ; demandez au bureau du conservateur si une manifestation ou un travail de terrain touche un site inscrit.
permits-hint-history-register-pl = Un monument de l'histoire (Pomnik historii) se trouve dans la zone de la carte ou près d'elle ; la section sur les monuments de l'histoire dit lequel. Le président de la République de Pologne déclare par décret Pomnik historii un monument immobilier inscrit au registre, ou un parc culturel, de valeur particulière (art. 15 ust. 1 de la loi sur la protection des monuments, ustawa o ochronie zabytków). Il est donc déjà sous la protection du registre ou du parc culturel. Pour des travaux sur un monument inscrit au registre, le conservateur des monuments de la voïvodie (Wojewódzki Konserwator Zabytków) doit accorder une autorisation (art. 36 ust. 1) ; dans un parc culturel, la délibération du conseil de la gmina fixe les interdictions et les limites (art. 16 ust. 2, art. 17 ust. 1). La loi ne nomme ni une promenade, ni une course, ni un poste en tant que tels ; demandez au bureau du conservateur si un événement ou un travail de terrain touche le monument.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] La zone de la carte comprend un monument naturel (pomnik przyrody), un site écologique (użytek ekologiczny), un ensemble naturel et paysager (zespół przyrodniczo-krajobrazowy) ou un site de documentation (stanowisko dokumentacyjne). Le conseil de la gmina les établit par délibération, qui indique qui en assure la surveillance et quelles interdictions s'appliquent (art. 44 de la loi sur la protection de la nature). La loi elle-même ne prévoit ici aucune autorisation pour une manifestation. Demandez la délibération et le responsable de la surveillance au bureau de la gmina sur le territoire de laquelle se trouve l'objet.
       *[other] La zone de la carte comprend un monument naturel (pomnik przyrody), un site écologique (użytek ekologiczny), un ensemble naturel et paysager (zespół przyrodniczo-krajobrazowy) ou un site de documentation (stanowisko dokumentacyjne). Le conseil de la gmina les établit par délibération, qui indique qui en assure la surveillance et quelles interdictions s'appliquent (art. 44 de la loi sur la protection de la nature). La loi elle-même ne prévoit ici aucune autorisation pour une manifestation. Demandez la délibération et le responsable de la surveillance { $count ->
        [one] au bureau de la gmina ({ $gmina }) sur le territoire de laquelle se trouve l'objet
       *[other] aux bureaux des gminas ({ $gmina }), chacun pour les objets de son territoire
    }.
    }
permits-hint-gmina-roads-pl = Une manifestation qui gêne la circulation ou utilise une rue de manière particulière nécessite une autorisation au titre de l'art. 65 du code de la route, demandée au moins 30 jours à l'avance. Elle est délivrée par l'autorité qui gère la circulation sur la route : le starosta pour les routes du powiat et de la gmina, le maire dans une ville à statut de powiat, le maréchal de la voïvodie ou la GDDKiA pour les autres routes (art. 10 et 65b). Le gestionnaire d'une route de la gmina est le wójt, burmistrz ou prezydent miasta { $count ->
        [one] de la gmina ({ $gmina }) ;
       *[other] de chacune des gminas ({ $gmina }) ;
    } il décide de l'occupation de l'emprise routière (art. 19 et 40 de la loi sur les routes publiques). Ces notes n'indiquent pas la catégorie des routes.
permits-hint-gmina-land-pl = Si un terrain de la zone de la carte appartient à la gmina, par exemple un parc municipal, une place, un terrain d'école ou une forêt, il est administré par le wójt, burmistrz ou prezydent miasta, qui représente la gmina (art. 30 et 31 de la loi sur l'autonomie de la gmina). Demandez l'accord { $count ->
        [one] du bureau de la gmina ({ $gmina }) avant
       *[other] des bureaux des gminas ({ $gmina }) avant
    } de l'utiliser. Ces notes n'indiquent pas à qui appartient le terrain.
permits-hint-protected-areas-cz = Des zones protégées ou des objets de protection de la nature se trouvent dans la zone de la carte. Selon la loi sur la protection de la nature et du paysage (loi n° 114/1992 Sb.), les manifestations sportives, touristiques et autres manifestations publiques organisées dans un parc national sont interdites en dehors des lieux que l'autorité de protection de la nature a désignés (§ 16 odst. 1 písm. q), et dans la zone de protection d'un parc national, ces manifestations de masse en dehors de ces lieux et en dehors des zones bâties exigent le consentement de l'autorité (§ 37 odst. 3 písm. d). Dans une réserve naturelle nationale, il est interdit d'entrer ou de circuler en dehors des chemins balisés avec le consentement de l'autorité (§ 29 písm. d). Les autorités de protection de la nature sont, entre autres, l'administration d'un parc national, l'Agence de protection de la nature et du paysage (AOPK ČR), le bureau régional (krajský úřad) et le bureau d'une commune à compétences élargies (§ 75 odst. 1). Les autres zones et objets protégés ont leurs propres conditions de protection. Interrogez tôt l'autorité si vous prévoyez une manifestation à cet endroit.
permits-hint-monuments-cz = Des monuments culturels ou des zones monumentales de la liste de l'État se trouvent dans la zone de la carte. Selon la loi sur la protection du patrimoine de l'État (loi n° 20/1987 Sb., § 14 odst. 1 et 2), le propriétaire d'un monument doit demander au préalable une décision ou un avis contraignant avant l'entretien, la réparation, la reconstruction ou une autre modification du monument ou de ses abords. Il en va de même pour le propriétaire d'un bien situé dans une réserve monumentale, une zone monumentale ou la zone de protection d'un monument pour une construction, une modification d'une construction, des travaux de terrain, un dispositif ou un panneau, la démolition d'un ouvrage, le traitement d'arbres ou des travaux d'entretien. Le bureau de la commune à compétences élargies décide{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, pour un monument culturel national le bureau régional. La loi ne nomme ni promenade, ni course, ni poste de contrôle en tant que tels ; demandez à ce bureau si une manifestation touche un monument ou y place des dispositifs ou des panneaux.
permits-hint-forest-owner-cz = Une manifestation sociale de masse, par exemple sportive ou culturelle, sur des terrains destinés aux fonctions forestières ne peut avoir lieu qu'avec le consentement écrit du propriétaire de la forêt où elle se déroule (loi sur les forêts, loi n° 289/1995 Sb., § 20 odst. 5). L'organisateur doit en outre la déclarer par écrit au bureau de la commune à compétences élargies{ $orp ->
        [none] {""}
       *[other] {" "}({ $orp})
    }, ou au bureau régional si la manifestation dépasse le ressort d'une telle commune, au plus tard 30 jours avant la manifestation. La déclaration indique la date et le lieu, la façon dont la manifestation est encadrée, le nombre prévu de participants et les consentements écrits des propriétaires (même paragraphe). Ces notes n'indiquent pas à qui appartient la forêt ; interrogez tôt les propriétaires forestiers.
permits-hint-roads-cz = Une manifestation qui utilise une route autrement que d'habitude, par exemple une manifestation sportive, culturelle ou similaire qui pourrait compromettre la sécurité ou la fluidité de la circulation, nécessite l'autorisation du bureau compétent de l'administration routière. Elle est délivrée avec le consentement préalable du propriétaire de la route et, là où la sécurité ou la fluidité de la circulation peut être affectée, de la Police de la République tchèque (loi sur les routes, loi n° 13/1997 Sb., § 25 odst. 1 et odst. 6 písm. d). Le bureau est la mairie pour les routes locales, ou le bureau de la commune à compétences élargies là où il décide, et le bureau de la commune à compétences élargies pour les routes de deuxième et de troisième classe (§ 40 odst. 4 et 5). Communes dans la zone de la carte : { $gmina }. Ces notes n'indiquent pas la classe des routes.
permits-section-water-protection = Zones de protection des eaux ({ $source })
permits-section-forest-functions = Fonctions de la forêt et forêt protégée ({ $source })
permits-none-water-protection = Selon { $source }, aucune zone de protection des eaux ne se trouve dans la zone de la carte.
permits-none-forest-functions = Selon { $source }, aucune forêt de loisirs, aucune forêt protégée, aucune forêt naturelle et aucune zone forestière fermée ne se trouve dans la zone de la carte.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Forêt de loisirs (Erholungswald)
        [protected] Forêt protégée (Bannwald ou Schutzwald)
        [natural] Forêt naturelle ou réserve intégrale (Naturwald, Totalreservat)
       *[closed] Zone forestière fermée
    }
permits-hint-water-protection = { $inner ->
        [yes] La { $zone } de la zone de protection des eaux « { $name } » se trouve dans la zone de la carte. Dans les zones I et II, les événements sont généralement exclus. Adressez-vous à l'autorité inférieure de l'eau (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] La { $zone } de la zone de protection des eaux « { $name } » se trouve dans la zone de la carte. Les événements y sont généralement possibles ; l'autorité inférieure de l'eau (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) peut dire ce que l'arrêté autorise.
       *[unknown] La zone de protection des eaux « { $name } » se trouve dans la zone de la carte. Les données n'indiquent pas de quelle zone il s'agit. Dans les zones I et II, les événements sont généralement exclus ; adressez-vous à l'autorité inférieure de l'eau (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Une forêt de loisirs (Erholungswald) se trouve dans la zone de la carte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pour des événements à cet endroit, adressez-vous à l'autorité forestière.
        [protected] Une forêt protégée (Bannwald ou Schutzwald) se trouve dans la zone de la carte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Adressez-vous à l'autorité forestière avant de prévoir un événement à cet endroit.
        [natural] Une forêt naturelle ou une réserve intégrale se trouve dans la zone de la carte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Quitter les chemins y est souvent interdit ; n'y placez aucun poste sans demander à l'autorité forestière.
       *[closed] Une zone forestière interdite d'accès se trouve dans la zone de la carte{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Tenez-la hors du parcours.
    }
permits-section-roads = Routes classées d'après OpenStreetMap
permits-none-roads = Selon OpenStreetMap, aucune Autobahn ni aucune Bundes-, Landes-, Staats- ou Kreisstraße ne traverse la zone de la carte.
permits-road-line = { $class ->
        [autobahn] Autobahn (autoroute)
        [bundesstrasse] Bundesstraße (route fédérale)
        [landesstrasse] Landesstraße ou Staatsstraße (route du Land)
       *[kreisstrasse] Kreisstraße (route d'arrondissement)
    } : { $refs }
permits-hint-road-class = { $class ->
        [autobahn] L'Autobahn { $refs } traverse la zone de la carte. Un parcours ne peut la franchir que par des ponts ou des passages souterrains.
        [bundesstrasse] La Bundesstraße { $refs } traverse la zone de la carte. Une fermeture ou une traversée avec commissaires est ordonnée par l'autorité de la circulation routière (Straßenverkehrsbehörde) ; la route est généralement entretenue par le Bund via l'administration routière du Land, dans les grandes villes souvent par la ville.
        [landesstrasse] La Landes- ou Staatsstraße { $refs } traverse la zone de la carte. Une fermeture ou une traversée avec commissaires est ordonnée par l'autorité de la circulation routière (Straßenverkehrsbehörde) ; la route est généralement entretenue par le Land, dans les grandes villes souvent par la ville.
       *[kreisstrasse] La Kreisstraße { $refs } traverse la zone de la carte. Une fermeture ou une traversée avec commissaires est ordonnée par l'autorité de la circulation routière (Straßenverkehrsbehörde) ; la route est généralement entretenue par le Kreis, dans les grandes villes souvent par la ville.
    }
permits-hint-state-forest = { $office ->
        [none] Selon les données ouvertes du Land, { $share } de la zone de la carte est une forêt domaniale du Land appartenant à « { $owner } ».
       *[other] Selon les données ouvertes du Land, { $share } de la zone de la carte est une forêt domaniale du Land appartenant à « { $owner } ». Compétent : { $office }.
    }
