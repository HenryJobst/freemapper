# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = warning: { $text }

warning-other-zone = the map area lies in UTM zone { $zone } (EPSG:{ $epsg }); freemapper fetches everything again in that zone
warning-other-state = the map area now lies in another state's data; freemapper fetches everything again
warning-beyond-coverage = part of the map area lies outside the supported states. It lies abroad, at sea or in a state freemapper does not support yet, such as Bremen or Thuringia. The states have no data there, so the orthophoto can be white and ALKIS gives no buildings or land use
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: scale 1:{ $scale } is none of the scales for { $kind } maps ({ $allowed }); freemapper-print will refuse the map
warning-layer-entry-ignored = freemapper.toml: ignored the { $what } layer entry { $key }: { $reason ->
        [name] not a plain file name
       *[unknown] no layer freemapper knows
    }

## Disk space

warning-disk-space-unreadable = cannot read the free disk space of { $path } ({ $error }), so freemapper does not check it

## Tiles

warning-tile-changed = { $product } tile { $tile } changed upstream ({ $old } → { $new })
warning-bdom-tile-unavailable = surface model (bDOM) of tile { $tile } unavailable ({ $error }); no nDSM template there
warning-bdom-unavailable = surface model (bDOM) unavailable ({ $error }); no nDSM template for { $tiles }
progress-ndsm-from-bdom = nDSM of tile { $tile } from the surface model (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terrain model
        [surface] surface model
       *[orthophoto] orthophoto
    } ({ $name })
progress-model-download = downloading the { $model }
progress-model-read = reading the { $model }
warning-model-unavailable = { $model } unavailable ({ $error }); no nDSM template
warning-model-unavailable-kept = { $model } unavailable ({ $error }); kept the surface model fetched on { $fetched }
warning-model-tile-unavailable = { $model } of tile { $tile } unavailable ({ $error }); no nDSM template there
warning-model-tiles-missing = the { $model } has no { $count ->
        [one] tile
       *[other] tiles
    } { $names } inside { $state ->
        [the-map-area] the map area
        [hamburg] Hamburg
        [hesse] Hesse
        [north-rhine-westphalia] North Rhine-Westphalia
        [schleswig-holstein] Schleswig-Holstein
        [mecklenburg-vorpommern] Mecklenburg-Vorpommern
        [lower-saxony] Lower Saxony
       *[other] { $state }
    }; the templates stay empty there
warning-no-surface-stored = no surface model stored; run `freemapper fetch` online for the nDSM template
warning-surface-older-than-orthophoto = the surface model (bDOM { $surface }) is older than the orthophoto ({ $photo }); new buildings may be missing in the nDSM
progress-surface-kept = freemapper keeps the surface model fetched on { $fetched } for the nDSM template
progress-surface-tiles-kept = freemapper keeps the cached surface model of { $count ->
        [one] tile
       *[other] tiles
    } { $names } for the nDSM template
progress-orthophoto-unchanged = orthophoto unchanged
progress-orthophoto-unchanged-dated = orthophoto unchanged ({ $label } { $date })
progress-orthophoto-download = downloading the orthophoto from the { $provider } { $service }
progress-orthophoto-in-leaf = the orthophoto was flown on { $date }, with the trees in leaf; their crowns hide what lies below them
progress-orthophoto-bare = the orthophoto was flown on { $date }, with the trees bare
warning-lgln-orthophoto-kept = orthophoto (LGLN WMS): { $reason }; kept the stored orthophoto and its flight date { $date }
warning-lgln-orthophoto-undated = orthophoto (LGLN WMS): no flight date, { $reason }
word-stac-no-image = the STAC lists no image at the centre of the map area
word-stac-no-answer = the STAC search gave no usable answer ({ $error })
warning-downloadcenter-no-file = the Downloadcenter has no file for { $places }
warning-downloadcenter-no-file-empty = the Downloadcenter has no file for { $places }; { $count ->
        [one] tile
       *[other] tiles
    } { $names } stayed empty
progress-k-five-unchanged = K5 template unchanged
progress-k-five-download = downloading the K5 map (1:5000) from the Geoportal Berlin WMS
warning-k-five-unavailable = the K5 map template is unavailable ({ $detail }); skipped

## The state border

progress-beyond-border = the map area reaches beyond the state border; beyond it the templates are empty wherever the { $provider } orthophoto and height models end. ALKIS and OSM come from both states
progress-beyond-border-no-data = the map area reaches beyond the state border; no { $provider } data for { $tiles }, so the templates stay empty there

## OSM

warning-osm-extract-missing-kept = no OSM extract cached; keeping the stored OSM candidates from { $date }
progress-osm-extract-read = reading OSM extract { $region } from { $date }
progress-osm-candidates = { $count } OSM candidate features

## ALKIS

warning-alkis-coverage-unknown = ALKIS buildings coverage unknown; run `freemapper fetch` online to record it
progress-alkis-buildings-download = downloading ALKIS buildings
progress-alkis-buildings = { $count } ALKIS buildings
warning-alkis-no-buildings = no ALKIS buildings found in the map area
progress-alkis-skipped = skipped { $count } ALKIS features: { $list }
warning-alkis-buildings-changed = ALKIS buildings changed upstream ({ $changes })
progress-alkis-no-land-use-stored = no ALKIS land use stored; run `freemapper fetch` online for out-of-bounds candidates
progress-alkis-land-use-download = downloading ALKIS land use
progress-alkis-land-use-parts = { $count } ALKIS land use parts
warning-alkis-land-use-kept = ALKIS land use unavailable ({ $error }); keeping the stored land use
warning-alkis-land-use-removed = ALKIS land use unavailable ({ $error }); the stored land use no longer covers the map area, so freemapper removed it
warning-alkis-land-use-none = ALKIS land use unavailable ({ $error }); no out-of-bounds candidates

## The Berlin layers

progress-berlin-none-stored = no Berlin ALKIS structures stored; run `freemapper fetch` online for stairs, walls, trees and canopies
progress-berlin-removing = removing the stored Berlin layers: the map area is no longer near Berlin
progress-berlin-download = downloading Berlin ALKIS structures, tree cadastre, street survey and point layers
progress-berlin-features = { $count } Berlin { $file } features
warning-berlin-layer-kept = Berlin { $file } unavailable ({ $detail }); keeping the stored layer
warning-berlin-layer-skipped = Berlin { $file } unavailable ({ $detail }); layer skipped
warning-berlin-buildings-missing = no ALKIS buildings stored for this area in Berlin; run `freemapper fetch` to get them from the Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = no full ALKIS structures stored; run `freemapper fetch` online for walls, fences, hedges and bridges
progress-nas-removing = removing the stored full ALKIS layers: the map area is no longer in a state with a NAS service
warning-nas-folder-outside = full ALKIS layers skipped ({ $folder } leads out of the project folder)
progress-nas-download = downloading the full ALKIS structures (NAS)
progress-nas-features = { $count } ALKIS NAS { $file } features
progress-nas-skipped = skipped { $count } ALKIS NAS { $file } features: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } unavailable ({ $detail }); keeping the stored layer
warning-nas-layer-skipped = ALKIS NAS { $file } unavailable ({ $detail }); layer skipped

## Tree cadastres

progress-trees-none-stored = no city tree cadastre stored; run `freemapper fetch` online for the single trees
progress-trees-removing = removing the stored tree cadastres: the map area is no longer in a city with one
warning-trees-folder-outside = city tree cadastres skipped ({ $folder } leads out of the project folder)
progress-trees-download = downloading the city tree cadastres
progress-trees-count = { $count } { $id } trees
progress-trees-skipped = skipped { $count } { $id } trees: { $list }
warning-trees-kept = { $id } tree cadastre unavailable ({ $detail }); keeping the stored one
warning-trees-skipped = { $id } tree cadastre unavailable ({ $detail }); trees skipped

## Deriving the templates

progress-writing = writing { $path }
progress-no-ndsm-recorded = no nDSM recorded; run `freemapper fetch` for the nDSM template
warning-ndsm-tile-invalid = invalid nDSM tile { $tile } in freemapper.toml; skipped
warning-ndsm-not-cached = nDSM { $tile } not cached; run `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } unreadable ({ $error }); delete { $path } and run `freemapper fetch`
warning-surface-unusable = surface model unusable ({ $error }); run `freemapper fetch`
