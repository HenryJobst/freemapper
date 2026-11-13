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
warning-archive-passed-over = { $model }: an archive could not be read ({ $error }); it was passed over, and the tiles of the map area come from the other archives
warning-archive-directory-kept = { $model }: an archive could not be read ({ $error }); its directory kept from an earlier run is used, so a tile the LVGL replaced since is not noticed
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

## Permit notes

progress-permits-fetch = fetching { $source } data for the permit notes
progress-permits-osm = reading OSM extract { $region } from { $date } for the permit notes
progress-permits-stored = reading the stored data of the permit notes
warning-permits-source-unreachable = { $source } could not be read, so the permit notes leave it out: { $error }
warning-permits-osm-no-extract = no OSM extract is cached for this map area, so the permit notes leave OSM out; `freemapper fetch` downloads it

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
error-extra-layer-unknown = not a layer of { $folder }
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

## Downloads and the cache

progress-downloading = downloading { $url }
progress-downloaded = downloaded { $megabytes } MB
progress-downloaded-of = downloaded { $megabytes } MB of { $total } MB
progress-tile-index-download = downloading the { $product } tile index
progress-tile-cached = cached { $tile }
warning-tile-cached-older = using cached { $product } tile { $tile } from { $published } (newer { $newer } not downloaded)
warning-ndsm-no-cached = no cached nDSM for tile { $tile }
warning-ndsm-tile-unavailable-cached = nDSM tile { $tile } unavailable ({ $reason }); using the cached file
warning-ndsm-tile-unavailable-skipped = nDSM tile { $tile } unavailable ({ $reason }); skipped
progress-ndsm-not-published = no nDSM point cloud published for tile { $tile }
progress-ndsm-cached = cached nDSM { $tile }
progress-point-cloud-not-published = no laser point cloud published for tile { $tile }
progress-point-cloud-cached = cached point cloud { $tile }
warning-point-cloud-no-cached = no cached point cloud for tile { $tile }; no vegetation candidates there
warning-point-cloud-unavailable-cached = point cloud { $tile } unavailable ({ $reason }); using the cached file
warning-point-cloud-unavailable-skipped = point cloud { $tile } unavailable ({ $reason }); no vegetation candidates there
warning-point-clouds-unavailable = the laser point clouds are unavailable ({ $reason }); the vegetation candidates use the cached tiles, if any
progress-no-point-clouds = no laser point clouds for the vegetation here yet: the state publishes none freemapper can read, or not in the map's coordinate system
warning-point-clouds-not-fetched = the vegetation is chosen, but no point clouds are loaded; run `freemapper fetch` to load them
warning-point-cloud-unreadable = point cloud { $tile } unreadable ({ $error }); delete { $path } and run `freemapper fetch`
warning-osm-nrw-nearest = no North Rhine-Westphalia district extract within 100 m of the area; reading the nearest, { $district }
warning-osm-disk-full-cached = the disk is full, so freemapper uses the cached OSM extract from { $date }. Free some space and run `freemapper fetch` again for a newer one
warning-osm-geofabrik-unavailable = Geofabrik unavailable ({ $error }); using the cached OSM extract from { $date }
progress-osm-extract-cached = cached OSM extract { $region } from { $date }
progress-osm-extract-download = downloading { $url } (about { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = downloading bDOM tile { $tile } ({ $megabytes } MB, { $index } of { $total })
progress-sh-dgm-not-in-atom = DGM1 tile { $tile } is not in the INSPIRE ATOM directory; downloading its XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1 tile { $tile } from the INSPIRE ATOM directory does not read ({ $error }); downloading its XYZ ({ $megabytes } MB)
progress-sh-tile = tile { $tile }: { $text }
progress-sh-download-again = tile { $tile }: the download does not read ({ $message }); downloading it once more
warning-sh-portal-no-date = the Downloadportal gives no date for the { $model } tile { $tile }
warning-sh-dgm-xyz-used = DGM1 tile { $tile } is not in the INSPIRE ATOM directory; used its XYZ from the Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1 tile { $tile } from the INSPIRE ATOM directory does not read ({ $error }); used its XYZ from the Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model } tile { $tile }: { $count } cells of the map area have no height
warning-centre-tile-missing = the { $model } tile { $tile } at the centre of the map area is missing, so there is no date
warning-centre-tile-unlisted = the tile list names no { $model } tile { $tile } at the centre of the map area, so there is no date
warning-list-unreadable-kept = the tile list of the { $model } could not be read ({ $error }); used the one kept from an earlier run
warning-portal-list-unreadable-kept = the Downloadportal's tile list of the { $model } could not be read ({ $error }); used the one kept from an earlier run
warning-surface-much-older = the { $model } is from { $surface }, the orthophoto from { $flight }. The nDSM may lack newer buildings and trees.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } features of the tile list have no usable tile key; skipped
warning-rp-list-unreadable-kept = the LVermGeo RP tile list could not be read ({ $error }); used the one kept from an earlier run
warning-rp-metalink-not-kept = the LVermGeo RP Metalink was not kept: { $error }
warning-rp-metalink-unreadable-kept = the LVermGeo RP Metalink of the { $model } could not be read ({ $detail }); used the one kept from an earlier run
warning-rp-metalink-unreadable = the LVermGeo RP Metalink of the { $model } could not be read ({ $detail }), so the tiles its tile list gives no flight year for stay without heights
warning-rp-tile-without-name = { $model } tile { $tile }: this 1 km square has no heights in the map. LVermGeo RP's tile list (Laserbefliegung) { $note } for it, so the file name is not known{ $either ->
        [metalink] , and its Metalink names no file for it either
       *[none] {""}
    }
warning-rp-centre-no-date = the { $model } tile { $tile } has no date: LVermGeo RP's tile list (Laserbefliegung) { $note } for it
word-rp-list-neither = gives neither a flight year nor flight days
word-rp-list-no-days = gives no flight days
word-rp-list-no-year = gives no flight year
word-rp-list-days-unreadable = gives flight days { $days } that cannot be read
word-rp-list-disagree = gives flight days { $days } and a flight year { $year } that do not agree on one year

## Hesse

warning-hvbg-undated = the metadata list has no readable Bildflugdatum for tile { $tiles }; their project records have no flight date
warning-hvbg-undated-more = the metadata list has no readable Bildflugdatum for tile { $tiles } and { $more } more; their project records have no flight date
warning-hvbg-list-unreadable-older = the new HVBG metadata list { $name } could not be read ({ $detail }); using the older list { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: no date, the STAC lists no tile at the centre of the map area
warning-lgln-no-date = { $model }: no date ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; the stored model keeps its recorded date
warning-mv-cells-empty = { $model }: { $count } cells inside the Mecklenburg-Vorpommern tiles have no height
warning-mv-centre-no-date = { $model }: no date for the centre tile { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } rows of the { $model } tile list have no usable tile name; skipped
warning-nrw-list-rows-undated = { $count } rows of the { $model } tile list have no readable date
warning-nrw-list-older = the server's tile list of the { $model } dates from { $server }, older than the kept one from { $kept }; used the kept one
warning-nrw-list-no-date = the tile list gives no date for the { $model } tile { $tile }

## After `init`

note-scale-outside-norm = note: 1:5000 is not in the sprint norm; freemapper-print warns about it and the symbols stay at 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } is not an OSM id like way/992423119
error-place-invalid-answer = invalid answer from the place search: { $detail }
error-place-not-found = { $id } not found. Nominatim lists only tagged OSM objects, so an untagged outer way of a multipolygon is not there. Use the multipolygon relation instead (--osm relation/<id>), or save the outline as GeoJSON and use --geojson
error-place-is-a-point = { $id } is a point, not an area; pick its grounds (a way or relation)
error-place-no-such-part = { $id } has { $count ->
        [one] 1 part
       *[other] { $count } parts
    }; there is no part { $part }
error-place-several-parts = { $id } is an area of { $count } separate parts ({ $sizes }). A map area is one polygon: pick a part with --part <n> (1 is the largest), or save the outline you want as GeoJSON and use --geojson
error-layer-not-a-world-file = { $path }: not a world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = no preview for { $layer }; layers with previews: { $names }
error-layer-no-ndsm-published = no nDSM is published for the tiles of this map area, so there is no ndsm template
error-layer-not-made = { $layer } is not made yet; run { $step ->
        [fetch] freemapper fetch or run (MCP: start_pipeline); Berlin only
        [export] freemapper export or run (MCP: start_pipeline)
       *[derive] freemapper derive or run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; the OCAD map gets no background from it
warning-map-preview-failed = the map is written, but its preview is missing: { $error }
