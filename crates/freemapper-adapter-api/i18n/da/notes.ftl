# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = advarsel: { $text }

warning-other-zone = kortområdet ligger i UTM-zone { $zone } (EPSG:{ $epsg }); freemapper henter alt igen i den zone
warning-other-state = kortområdet ligger nu i en anden delstats data; freemapper henter alt igen
warning-no-orthophoto = der findes intet ortofoto for denne region; kortet har ingen ortofotoskabelon
warning-beyond-coverage = en del af kortområdet ligger uden for de understøttede delstater. Den ligger i udlandet, på havet eller i en delstat, freemapper endnu ikke understøtter, for eksempel Thüringen. Delstaterne har ingen data dér, så ortofotoet kan være hvidt, og ALKIS giver ingen bygninger eller arealanvendelse
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: målestok 1:{ $scale } er ikke en af målestokkene for kort af typen { $kind } ({ $allowed }); freemapper-print vil afvise kortet
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorerede { $german ->
       *[any] { $what }
    }-lagindgangen { $key }: { $reason ->
        [name] ikke et almindeligt filnavn
       *[unknown] intet lag, freemapper kender
    }

## Disk space

warning-disk-space-unreadable = kan ikke læse den ledige diskplads for { $path } ({ $error }), så freemapper kontrollerer den ikke

## Tiles

warning-tile-changed = { $product }-flisen { $tile } er ændret hos leverandøren ({ $old } → { $new })
warning-bdom-tile-unavailable = overflademodel (bDOM) for flisen { $tile } utilgængelig ({ $error }); ingen nDSM-skabelon dér
warning-bdom-unavailable = overflademodel (bDOM) utilgængelig ({ $error }); ingen nDSM-skabelon for { $tiles }
progress-ndsm-from-bdom = nDSM for flisen { $tile } fra overflademodellen (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terrænmodel
        [surface] overflademodel
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = henter { $model }
progress-model-read = læser { $model }
warning-model-unavailable = { $model } utilgængelig ({ $error }); ingen nDSM-skabelon
warning-model-unavailable-kept = { $model } utilgængelig ({ $error }); beholdt overflademodellen hentet den { $fetched }
warning-model-tile-unavailable = { $model } for flisen { $tile } utilgængelig ({ $error }); ingen nDSM-skabelon dér
warning-archive-passed-over = { $model }: et arkiv kunne ikke læses ({ $error }); det blev sprunget over, og kortområdets fliser kommer fra de andre arkiver
warning-archive-directory-kept = { $model }: et arkiv kunne ikke læses ({ $error }); dets fortegnelse fra en tidligere kørsel bruges, så en flise, LVGL har erstattet siden, opdages ikke
warning-model-tiles-missing = { $model } har ingen { $count ->
        [one] flise
       *[other] fliser
    } { $names } inden for { $state ->
        [the-map-area] kortområdet
        [hamburg] Hamburg
        [hesse] Hessen
        [north-rhine-westphalia] Nordrhein-Westfalen
        [schleswig-holstein] Slesvig-Holsten
        [mecklenburg-vorpommern] Mecklenburg-Vorpommern
        [lower-saxony] Niedersachsen
        [bavaria] Bayern
       *[other] { $state }
    }; skabelonerne forbliver tomme dér
warning-no-surface-stored = ingen overflademodel gemt; kør `freemapper fetch` online for nDSM-skabelonen
warning-surface-older-than-orthophoto = overflademodellen (bDOM { $surface }) er ældre end ortofotoet ({ $photo }); nye bygninger kan mangle i nDSM
progress-surface-kept = freemapper beholder overflademodellen hentet den { $fetched } til nDSM-skabelonen
progress-surface-tiles-kept = freemapper beholder den cachede overflademodel for { $count ->
        [one] flisen
       *[other] fliserne
    } { $names } til nDSM-skabelonen
progress-orthophoto-unchanged = ortofoto uændret
progress-orthophoto-unchanged-dated = ortofoto uændret ({ $label } { $date })
progress-orthophoto-download = henter ortofotoet fra { $provider } { $service }
progress-orthophoto-in-leaf = ortofotoet blev fløjet den { $date }, med træerne i løv; deres kroner skjuler det, der ligger under dem
progress-orthophoto-bare = ortofotoet blev fløjet den { $date }, med træerne uden blade
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; beholdt det gemte ortofoto og dets flyvedato { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): ingen flyvedato, { $reason }
word-stac-no-image = STAC viser intet billede i midten af kortområdet
word-stac-no-answer = STAC-søgningen gav intet brugbart svar ({ $error })
warning-downloadcenter-no-file = Downloadcenter har ingen fil til { $places }
warning-downloadcenter-no-file-empty = Downloadcenter har ingen fil til { $places }; { $count ->
        [one] flisen
       *[other] fliserne
    } { $names } forblev tomme
progress-k-five-unchanged = K5-skabelon uændret
progress-k-five-download = henter K5-kortet (1:5000) fra Geoportal Berlin WMS
warning-k-five-unavailable = K5-kortskabelonen er utilgængelig ({ $detail }); sprunget over

## The state border

progress-beyond-border = kortområdet rækker ud over delstatsgrænsen; dér er skabelonerne tomme, hvor { $provider }-ortofotoet og højdemodellerne slutter. ALKIS og OSM kommer fra begge delstater
progress-beyond-border-no-data = kortområdet rækker ud over delstatsgrænsen; ingen { $provider }-data for { $tiles }, så skabelonerne forbliver tomme dér

## OSM

warning-osm-extract-missing-kept = intet OSM-udtræk i cachen; beholder de gemte OSM-kandidater fra { $date }
progress-osm-extract-read = læser OSM-udtrækket { $region } fra { $date }
progress-osm-candidates = { $count } OSM-kandidatobjekter

## Permit notes

progress-permits-fetch = henter { $source }-data til tilladelsesnoterne
progress-permits-osm = læser OSM-udtrækket { $region } fra { $date } til tilladelsesnoterne
progress-permits-stored = læser tilladelsesnoternes gemte data
warning-permits-source-unreachable = { $source } kunne ikke læses, så tilladelsesnoterne udelader den: { $error }
warning-permits-osm-no-extract = intet OSM-udtræk er cachet for dette kortområde, så tilladelsesnoterne udelader OSM; `freemapper fetch` downloader det

## ALKIS

warning-alkis-coverage-unknown = dækningen af { $cadastre }-bygninger er ukendt; kør `freemapper fetch` online for at registrere den
progress-alkis-buildings-download = henter { $cadastre }-bygninger
progress-alkis-buildings = { $count } { $cadastre }-bygninger
warning-alkis-no-buildings = ingen { $cadastre }-bygninger fundet i kortområdet
progress-alkis-skipped = sprang { $count } { $cadastre }-objekter over: { $list }
warning-alkis-buildings-changed = { $cadastre }-bygninger er ændret hos leverandøren ({ $changes })
progress-alkis-no-land-use-stored = ingen { $cadastre }-arealanvendelse gemt; kør `freemapper fetch` online for kandidater til forbudte områder
progress-alkis-land-use-download = henter { $cadastre }-arealanvendelse
progress-alkis-land-use-parts = { $count } dele af { $cadastre }-arealanvendelsen
warning-alkis-land-use-kept = { $cadastre }-arealanvendelse utilgængelig ({ $error }); beholder den gemte arealanvendelse
warning-alkis-land-use-removed = { $cadastre }-arealanvendelse utilgængelig ({ $error }); den gemte arealanvendelse dækker ikke længere kortområdet, så freemapper fjernede den
warning-alkis-land-use-none = { $cadastre }-arealanvendelse utilgængelig ({ $error }); ingen kandidater til forbudte områder
progress-bdot-filled = { $count } BDOT10k-objekter ({ $what }) tilføjet, hvor EGiB intet giver ({ $powiat })
warning-bdot-failed = BDOT10k kunne ikke udfylde hullet i EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB er ikke tilgængelig ({ $error }); BDOT10k bruges i stedet ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = ingen Berlin-ALKIS-strukturer gemt; kør `freemapper fetch` online for trapper, mure, træer og overdækninger
progress-berlin-removing = fjerner de gemte Berlin-lag: kortområdet ligger ikke længere i nærheden af Berlin
progress-berlin-download = henter Berlin-ALKIS-strukturer, trækatastre, gadeopmåling og punktlag
progress-berlin-features = { $count } Berlin-objekter i { $file }
warning-berlin-layer-kept = Berlin { $file } utilgængelig ({ $detail }); beholder det gemte lag
warning-berlin-layer-skipped = Berlin { $file } utilgængelig ({ $detail }); lag sprunget over
progress-feinkartierung-none-stored = ingen Hamburg Feinkartierung gemt; kør `freemapper fetch` online for rækværk, mure, trapper og gadeinventar
progress-feinkartierung-removing = fjerner den gemte Hamburg Feinkartierung: kortområdet ligger ikke længere i Hamburg
progress-feinkartierung-download = henter Hamburg Feinkartierung (gaderum og offentligt grønt)
progress-feinkartierung-features = { $count } Feinkartierung-objekter i { $file }
warning-feinkartierung-layer-kept = Feinkartierung { $file } utilgængelig ({ $detail }); beholder det gemte lag
warning-feinkartierung-layer-skipped = Feinkartierung { $file } utilgængelig ({ $detail }); lag sprunget over
progress-bdot-roads-none-stored = ingen BDOT10k-veje og stier gemt; kør `freemapper fetch` online for at hente vejene og stierne fra registret
progress-bdot-roads-removing = fjerner de gemte BDOT10k-veje og -stier: kortområdet ligger ikke længere i Polen
progress-bdot-roads-download = henter BDOT10k-veje og -stier
progress-bdot-roads-features = { $count } BDOT10k-objekter i { $file }
warning-bdot-roads-layer-kept = BDOT10k { $file } er ikke tilgængelig ({ $detail }); beholder det gemte lag
warning-bdot-roads-layer-skipped = BDOT10k { $file } er ikke tilgængelig ({ $detail }); laget springes over
error-extra-layer-unknown = ikke et lag i { $folder }
warning-berlin-buildings-missing = ingen ALKIS-bygninger gemt for dette område i Berlin; kør `freemapper fetch` for at hente dem fra Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = ingen fulde ALKIS-strukturer gemt; kør `freemapper fetch` online for mure, hegn, hække og broer
progress-nas-removing = fjerner de gemte fulde ALKIS-lag: kortområdet ligger ikke længere i en delstat med en NAS-tjeneste
warning-nas-folder-outside = fulde ALKIS-lag sprunget over ({ $folder } fører ud af projektmappen)
progress-nas-download = henter de fulde ALKIS-strukturer (NAS)
progress-nas-features = { $count } ALKIS NAS-objekter i { $file }
progress-nas-skipped = sprang { $count } ALKIS NAS-objekter i { $file } over: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } utilgængelig ({ $detail }); beholder det gemte lag
warning-nas-layer-skipped = ALKIS NAS { $file } utilgængelig ({ $detail }); lag sprunget over

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = ingen objekter fra ATKIS Basis-DLM gemt; kør `freemapper fetch` online for kilder, huler, tårne, monumenter og springvand
progress-dlm-removing = fjerner de gemte lag fra ATKIS Basis-DLM: kortområdet ligger ikke længere i en delstat, der leverer det
warning-dlm-folder-outside = lag fra ATKIS Basis-DLM sprunget over ({ $folder } fører ud af projektmappen)
progress-dlm-download = henter objekterne fra ATKIS Basis-DLM
progress-dlm-features = { $count } ATKIS Basis-DLM-objekter i { $file }
progress-dlm-skipped = sprang { $count } ATKIS Basis-DLM-objekter i { $file } over: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } utilgængelig ({ $detail }); beholder det gemte lag
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } utilgængelig ({ $detail }); lag sprunget over

## Tree cadastres

progress-trees-none-stored = intet bytrækataster gemt; kør `freemapper fetch` online for de enkelte træer
progress-trees-removing = fjerner de gemte trækatastre: kortområdet ligger ikke længere i en by med et
warning-trees-folder-outside = bytrækatastre sprunget over ({ $folder } fører ud af projektmappen)
progress-trees-download = henter bytrækatastrene
progress-trees-count = { $count } { $id }-træer
progress-trees-skipped = sprang { $count } { $id }-træer over: { $list }
warning-trees-kept = trækatastret { $id } utilgængeligt ({ $detail }); beholder det gemte
warning-trees-skipped = trækatastret { $id } utilgængeligt ({ $detail }); træer sprunget over

## Deriving the templates

progress-writing = skriver { $path }
progress-contours-steep = højdekurver for hver { $interval } m: terrænet er så stejlt, at højdekurver for hver { $default } m ville løbe sammen; en ækvidistance på { $default } m i projektet holder dem adskilt (`--contour-interval { $default }`)
warning-osm-only-no-terrain = endnu ingen terrændata for denne region: kortet laves kun af OpenStreetMap, uden højdekurver, skabeloner og bygninger
progress-no-ndsm-recorded = ingen nDSM registreret; kør `freemapper fetch` for nDSM-skabelonen
warning-ndsm-tile-invalid = ugyldig nDSM-flise { $tile } i freemapper.toml; sprunget over
warning-ndsm-not-cached = nDSM { $tile } ikke i cachen; kør `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } kan ikke læses ({ $error }); slet { $path }, og kør `freemapper fetch`
warning-surface-unusable = overflademodellen kan ikke bruges ({ $error }); kør `freemapper fetch`

## Downloads and the cache

progress-downloading = henter { $url }
progress-retry-wait = tjenesten er optaget; prøver igen om { $seconds } s
progress-downloaded = hentet { $megabytes } MB
progress-downloaded-of = hentet { $megabytes } MB af { $total } MB
progress-tile-index-download = henter { $product }-flisefortegnelsen
progress-tile-cached = cachet { $tile }
progress-surface-blocks = læser { $count } af de { $total } blokke i overflademodellens flise { $tile } ({ $megabytes } MB)
progress-surface-whole = overflademodellens flise { $tile } hentes hel: { $reason }
warning-tile-cached-older = bruger den cachede { $product }-flise { $tile } fra { $published } (nyere { $newer } ikke hentet)
warning-ndsm-no-cached = ingen cachet nDSM for flisen { $tile }
warning-ndsm-tile-unavailable-cached = nDSM-flisen { $tile } utilgængelig ({ $reason }); bruger den cachede fil
warning-ndsm-tile-unavailable-skipped = nDSM-flisen { $tile } utilgængelig ({ $reason }); sprunget over
warning-file-check-failed = kunne ikke kontrollere, om { $file } stadig er aktuel ({ $reason }); bruger den cachede fil og denne servers andre cachede filer uden at spørge
warning-ldbv-no-buildings-file = ingen bygningsfil for flisen { $tile } på serveren; kortet kan mangle bygninger dér
progress-ndsm-not-published = ingen nDSM-punktsky udgivet for flisen { $tile }
progress-ndsm-cached = cachet nDSM { $tile }
progress-point-cloud-not-published = ingen laserpunktsky udgivet for flisen { $tile }
progress-point-cloud-cached = cachet punktsky { $tile }
warning-point-cloud-no-cached = ingen cachet punktsky for flisen { $tile }; ingen vegetationskandidater dér
warning-point-cloud-unavailable-cached = punktsky { $tile } utilgængelig ({ $reason }); bruger den cachede fil
warning-point-cloud-unavailable-skipped = punktsky { $tile } utilgængelig ({ $reason }); ingen vegetationskandidater dér
warning-point-clouds-unavailable = laserpunktskyerne er utilgængelige ({ $reason }); vegetationskandidaterne bruger de cachede fliser, hvis der er nogen
progress-no-point-clouds = endnu ingen laserpunktskyer til vegetationen her: delstaten udgiver ingen, freemapper kan læse, eller ikke i kortets koordinatsystem
warning-point-clouds-not-fetched = vegetationen er valgt, men ingen punktskyer er indlæst; kør `freemapper fetch` for at indlæse dem
warning-point-cloud-unreadable = punktsky { $tile } kan ikke læses ({ $error }); slet { $path }, og kør `freemapper fetch`
warning-osm-nrw-nearest = intet Nordrhein-Westfalen-kredsudtræk inden for 100 m af området; læser det nærmeste, { $district }
warning-osm-disk-full-cached = disken er fuld, så freemapper bruger det cachede OSM-udtræk fra { $date }. Frigør noget plads, og kør `freemapper fetch` igen for et nyere
warning-osm-geofabrik-unavailable = Geofabrik utilgængelig ({ $error }); bruger det cachede OSM-udtræk fra { $date }
progress-osm-extract-cached = cachet OSM-udtræk { $region } fra { $date }
progress-osm-extract-download = henter { $url } (cirka { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = henter bDOM-flisen { $tile } ({ $megabytes } MB, { $index } af { $total })
progress-sh-dgm-not-in-atom = DGM1-flisen { $tile } findes ikke i INSPIRE ATOM-fortegnelsen; henter dens XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1-flisen { $tile } fra INSPIRE ATOM-fortegnelsen kan ikke læses ({ $error }); henter dens XYZ ({ $megabytes } MB)
progress-sh-tile = flise { $tile }: { $text }
progress-sh-download-again = flise { $tile }: downloaden kan ikke læses ({ $message }); henter den en gang til
warning-sh-portal-no-date = Downloadportal giver ingen dato for { $model }-flisen { $tile }
warning-sh-dgm-xyz-used = DGM1-flisen { $tile } findes ikke i INSPIRE ATOM-fortegnelsen; brugte dens XYZ fra Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1-flisen { $tile } fra INSPIRE ATOM-fortegnelsen kan ikke læses ({ $error }); brugte dens XYZ fra Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }-flisen { $tile }: { $count } celler i kortområdet har ingen højde
warning-centre-tile-missing = { $model }-flisen { $tile } i midten af kortområdet mangler, så der er ingen dato
warning-centre-tile-unlisted = flisefortegnelsen nævner ingen { $model }-flise { $tile } i midten af kortområdet, så der er ingen dato
warning-list-unreadable-kept = flisefortegnelsen for { $model } kunne ikke læses ({ $error }); brugte den, der blev beholdt fra en tidligere kørsel
warning-portal-list-unreadable-kept = Downloadportals flisefortegnelse for { $model } kunne ikke læses ({ $error }); brugte den, der blev beholdt fra en tidligere kørsel
warning-surface-much-older = { $model } er fra { $surface }, ortofotoet fra { $flight }. nDSM kan mangle nyere bygninger og træer.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objekter i flisefortegnelsen har ingen brugbar flisenøgle; sprunget over
warning-rp-list-unreadable-kept = LVermGeo RP's flisefortegnelse kunne ikke læses ({ $error }); brugte den, der blev beholdt fra en tidligere kørsel
warning-rp-metalink-not-kept = LVermGeo RP's Metalink blev ikke beholdt: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP's Metalink for { $model } kunne ikke læses ({ $detail }); brugte den, der blev beholdt fra en tidligere kørsel
warning-rp-metalink-unreadable = LVermGeo RP's Metalink for { $model } kunne ikke læses ({ $detail }), så de fliser, hvis flisefortegnelse ikke giver noget flyveår, forbliver uden højder
warning-rp-tile-without-name = { $model }-flisen { $tile }: dette 1 km-kvadrat har ingen højder i kortet. LVermGeo RP's flisefortegnelse (Laserbefliegung) { $note } for det, så filnavnet ikke er kendt{ $either ->
        [metalink] , og dets Metalink nævner heller ingen fil til det
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }-flisen { $tile } har ingen dato: LVermGeo RP's flisefortegnelse (Laserbefliegung) { $note } for det
word-rp-list-neither = angiver hverken flyveår eller flyvedage
word-rp-list-no-days = angiver ingen flyvedage
word-rp-list-no-year = angiver intet flyveår
word-rp-list-days-unreadable = angiver flyvedage { $days }, som ikke kan læses
word-rp-list-disagree = angiver flyvedage { $days } og et flyveår { $year }, som ikke stemmer overens om ét år

## Hesse

warning-hvbg-undated = metadatalisten har ingen læsbar Bildflugdatum for flisen { $tiles }; deres projektposter har ingen flyvedato
warning-hvbg-undated-more = metadatalisten har ingen læsbar Bildflugdatum for flisen { $tiles } og { $more } flere; deres projektposter har ingen flyvedato
warning-hvbg-list-unreadable-older = den nye HVBG-metadataliste { $name } kunne ikke læses ({ $detail }); bruger den ældre liste { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: ingen dato, STAC viser ingen flise i midten af kortområdet
warning-lgln-no-date = { $model }: ingen dato ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; den gemte model beholder sin registrerede dato
warning-mv-cells-empty = { $model }: { $count } celler inden for Mecklenburg-Vorpommerns fliser har ingen højde
warning-mv-centre-no-date = { $model }: ingen dato for midterflisen { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } rækker i flisefortegnelsen for { $model } har intet brugbart flisenavn; sprunget over
warning-nrw-list-rows-undated = { $count } rækker i flisefortegnelsen for { $model } har ingen læsbar dato
warning-nrw-list-older = serverens flisefortegnelse for { $model } er fra { $server }, ældre end den beholdte fra { $kept }; brugte den beholdte
warning-nrw-list-no-date = flisefortegnelsen giver ingen dato for { $model }-flisen { $tile }

## After `init`

note-scale-outside-norm = note: 1:5000 er ikke i sprintnormen; freemapper-print advarer om det, og symbolerne forbliver på 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } er ikke et OSM-id som way/992423119
error-place-invalid-answer = ugyldigt svar fra stedsøgningen: { $detail }
error-place-not-found = { $id } blev ikke fundet. Nominatim viser kun mærkede OSM-objekter, så en umærket ydre linje i en multipolygon findes ikke dér. Brug i stedet multipolygon-relationen (--osm relation/<id>), eller gem omridset som GeoJSON og brug --geojson
error-place-is-a-point = { $id } er et punkt, ikke et område; vælg dets grund (en way eller relation)
error-place-no-such-part = { $id } har { $count ->
        [one] 1 del
       *[other] { $count } dele
    }; der er ingen del { $part }
error-place-several-parts = { $id } er et område med { $count } adskilte dele ({ $sizes }). Et kortområde er én polygon: vælg en del med --part <n> (1 er den største), eller gem det omrids, du vil have, som GeoJSON og brug --geojson
error-layer-not-a-world-file = { $path }: ikke en verdensfil
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = ingen forhåndsvisning af { $layer }; lag med forhåndsvisning: { $names }
error-layer-no-ndsm-published = ingen nDSM er udgivet for dette kortområdes fliser, så der er ingen ndsm-skabelon
error-layer-not-made = { $layer } er ikke lavet endnu; kør { $step ->
        [fetch] freemapper fetch eller run (MCP: start_pipeline); kun Berlin
        [export] freemapper export eller run (MCP: start_pipeline)
       *[derive] freemapper derive eller run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD-kortet får ingen baggrund fra den
warning-map-preview-failed = kortet er skrevet, men dets forhåndsvisning mangler: { $error }
warning-training-no-tree-cover = træningskortet beholder parkernes åbne land helt: trædækket (data/tree-cover.tif) mangler; kør `freemapper derive` igen
warning-ocad-no-paper-frames = OCAD-startpunktet har endnu ingen papirrammer; de ligger i templates/paper-frame.omap, en skabelon til OOM
warning-paper-frames-left-out = papirrammer udeladt af templates/paper-frame.omap: { $count }; de ligger for langt fra kortet til et OOM-kort; flyt dem tættere på kortområdet på områdesiden
