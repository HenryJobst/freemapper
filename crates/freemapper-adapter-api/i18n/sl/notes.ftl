# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = opozorilo: { $text }

warning-other-zone = območje karte leži v coni UTM { $zone } (EPSG:{ $epsg }); freemapper znova prenese vse v tej coni
warning-other-state = območje karte zdaj leži v podatkih druge zvezne dežele; freemapper znova prenese vse
warning-beyond-coverage = del območja karte leži zunaj podprtih zveznih dežel. Leži v tujini, na morju ali v zvezni deželi, ki je freemapper še ne podpira, na primer v Turingiji. Dežele tam nimajo podatkov, zato je ortofoto lahko bel, ALKIS pa ne da stavb ali rabe tal
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: merilo 1:{ $scale } ni med merili za karte vrste { $kind } ({ $allowed }); freemapper-print bo karto zavrnil
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: prezrt vnos sloja { $key } ({ $german ->
       *[any] { $what }
    }): { $reason ->
        [name] ni navadno ime datoteke
       *[unknown] ni sloj, ki ga freemapper pozna
    }

## Disk space

warning-disk-space-unreadable = prostega prostora na disku { $path } ni mogoče prebrati ({ $error }), zato ga freemapper ne preverja

## Tiles

warning-tile-changed = { $product }: ploščica { $tile } se je pri ponudniku spremenila ({ $old } → { $new })
warning-bdom-tile-unavailable = model površja (bDOM) za ploščico { $tile } ni na voljo ({ $error }); tam ni predloge nDSM
warning-bdom-unavailable = model površja (bDOM) ni na voljo ({ $error }); ni predloge nDSM za { $tiles }
progress-ndsm-from-bdom = nDSM ploščice { $tile } iz modela površja (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] model reliefa
        [surface] model površja
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = prenašam { $model }
progress-model-read = berem { $model }
warning-model-unavailable = { $model } ni na voljo ({ $error }); ni predloge nDSM
warning-model-unavailable-kept = { $model } ni na voljo ({ $error }); ohranjen je model površja, prenesen { $fetched }
warning-model-tile-unavailable = { $model }: ploščica { $tile } ni na voljo ({ $error }); tam ni predloge nDSM
warning-archive-passed-over = { $model }: arhiva ni bilo mogoče prebrati ({ $error }); preskočen je, ploščice območja karte pa prihajajo iz drugih arhivov
warning-archive-directory-kept = { $model }: arhiva ni bilo mogoče prebrati ({ $error }); uporabljen je njegov imenik iz prejšnjega zagona, zato se ploščica, ki jo je LVGL med tem zamenjal, ne opazi
warning-model-tiles-missing = { $model } nima { $count ->
        [one] ploščice
       *[other] ploščic
    } { $names } { $state ->
        [the-map-area] na območju karte
        [hamburg] v Hamburgu
        [hesse] v Hessnu
        [north-rhine-westphalia] v Severnem Porenju-Vestfaliji
        [schleswig-holstein] v Schleswig-Holsteinu
        [mecklenburg-vorpommern] v Mecklenburg-Vorpommernu
        [lower-saxony] v Spodnji Saški
        [bavaria] na Bavarskem
       *[other] v { $state }
    }; predloge tam ostanejo prazne
warning-no-surface-stored = model površja ni shranjen; za predlogo nDSM zaženi `freemapper fetch` z omrežjem
warning-surface-older-than-orthophoto = model površja (bDOM { $surface }) je starejši od ortofota ({ $photo }); novih stavb v nDSM morda ni
progress-surface-kept = freemapper za predlogo nDSM ohrani model površja, prenesen { $fetched }
progress-surface-tiles-kept = freemapper za predlogo nDSM ohrani predpomnjeni model površja { $count ->
        [one] ploščice
       *[other] ploščic
    } { $names }
progress-orthophoto-unchanged = ortofoto nespremenjen
progress-orthophoto-unchanged-dated = ortofoto nespremenjen ({ $label } { $date })
progress-orthophoto-download = prenašam ortofoto z { $service } ponudnika { $provider }
progress-orthophoto-in-leaf = ortofoto je bil posnet { $date }, ko so imela drevesa listje; njihove krošnje prekrivajo, kar leži pod njimi
progress-orthophoto-bare = ortofoto je bil posnet { $date }, ko so bila drevesa brez listja
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; ohranjen je shranjeni ortofoto in njegov datum preleta { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): brez datuma preleta, { $reason }
word-stac-no-image = STAC v središču območja karte ne navaja nobene slike
word-stac-no-answer = iskanje STAC ni dalo uporabnega odgovora ({ $error })
warning-downloadcenter-no-file = Downloadcenter nima datoteke za { $places }
warning-downloadcenter-no-file-empty = Downloadcenter nima datoteke za { $places }; { $count ->
        [one] ploščica { $names } je ostala prazna
        [two] ploščici { $names } sta ostali prazni
       *[other] ploščice { $names } so ostale prazne
    }
progress-k-five-unchanged = predloga K5 nespremenjena
progress-k-five-download = prenašam karto K5 (1:5000) z WMS Geoportala Berlin
warning-k-five-unavailable = predloga karte K5 ni na voljo ({ $detail }); preskočeno

## The state border

progress-beyond-border = območje karte sega prek meje zvezne dežele; onkraj nje so predloge prazne povsod, kjer se ortofoto in višinski modeli ponudnika { $provider } končajo. ALKIS in OSM prihajata iz obeh zveznih dežel
progress-beyond-border-no-data = območje karte sega prek meje zvezne dežele; za { $tiles } ni podatkov ponudnika { $provider }, zato predloge tam ostanejo prazne

## OSM

warning-osm-extract-missing-kept = v predpomnilniku ni izvlečka OSM; ohranjeni so shranjeni kandidati OSM z dne { $date }
progress-osm-extract-read = berem izvleček OSM { $region } z dne { $date }
progress-osm-candidates = kandidatnih objektov OSM: { $count }

## Permit notes

progress-permits-fetch = prenašam podatke { $source } za napotke za dovoljenja
progress-permits-osm = berem izvleček OSM { $region } z dne { $date } za napotke za dovoljenja
progress-permits-stored = berem shranjene podatke napotkov za dovoljenja
warning-permits-source-unreachable = { $source } ni bilo mogoče prebrati, zato ga napotki za dovoljenja izpustijo: { $error }
warning-permits-osm-no-extract = za to območje karte v predpomnilniku ni izvlečka OSM, zato napotki za dovoljenja OSM izpustijo; `freemapper fetch` ga prenese

## ALKIS

warning-alkis-coverage-unknown = pokritost stavb ALKIS ni znana; za zapis zaženi `freemapper fetch` z omrežjem
progress-alkis-buildings-download = prenašam stavbe ALKIS
progress-alkis-buildings = stavb ALKIS: { $count }
warning-alkis-no-buildings = na območju karte ni najdenih stavb ALKIS
progress-alkis-skipped = preskočeni objekti ALKIS ({ $count }): { $list }
warning-alkis-buildings-changed = stavbe ALKIS so se pri ponudniku spremenile ({ $changes })
progress-alkis-no-land-use-stored = raba tal ALKIS ni shranjena; za kandidate prepovedanih območij zaženi `freemapper fetch` z omrežjem
progress-alkis-land-use-download = prenašam rabo tal ALKIS
progress-alkis-land-use-parts = delov rabe tal ALKIS: { $count }
warning-alkis-land-use-kept = raba tal ALKIS ni na voljo ({ $error }); ohranjena je shranjena raba tal
warning-alkis-land-use-removed = raba tal ALKIS ni na voljo ({ $error }); shranjena raba tal ne pokriva več območja karte, zato jo je freemapper odstranil
warning-alkis-land-use-none = raba tal ALKIS ni na voljo ({ $error }); ni kandidatov prepovedanih območij

## The Berlin layers

progress-berlin-none-stored = berlinske strukture ALKIS niso shranjene; za stopnice, zidove, drevesa in krošnje zaženi `freemapper fetch` z omrežjem
progress-berlin-removing = odstranjujem shranjene berlinske sloje: območje karte ni več blizu Berlina
progress-berlin-download = prenašam berlinske strukture ALKIS, kataster dreves, cestni popis in točkovne sloje
progress-berlin-features = berlinski sloj { $file }: objektov { $count }
warning-berlin-layer-kept = berlinski sloj { $file } ni na voljo ({ $detail }); ohranjen je shranjeni sloj
warning-berlin-layer-skipped = berlinski sloj { $file } ni na voljo ({ $detail }); sloj preskočen
progress-feinkartierung-none-stored = hamburška Feinkartierung ni shranjena; za ograje, zidove, stopnice in ulično pohištvo zaženi `freemapper fetch` z omrežjem
progress-feinkartierung-removing = odstranjujem shranjeno hamburško Feinkartierung: območje karte ni več v Hamburgu
progress-feinkartierung-download = prenašam hamburško Feinkartierung (cestni prostor in javno zelenje)
progress-feinkartierung-features = Feinkartierung { $file }: objektov { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file } ni na voljo ({ $detail }); ohranjen je shranjeni sloj
warning-feinkartierung-layer-skipped = Feinkartierung { $file } ni na voljo ({ $detail }); sloj preskočen
error-extra-layer-unknown = ni sloj mape { $folder }
warning-berlin-buildings-missing = za to območje v Berlinu ni shranjenih stavb ALKIS; za prenos iz Geoportala Berlin zaženi `freemapper fetch`

## The full ALKIS layers (NAS)

progress-nas-none-stored = polne strukture ALKIS niso shranjene; za zidove, ograje, žive meje in mostove zaženi `freemapper fetch` z omrežjem
progress-nas-removing = odstranjujem shranjene polne sloje ALKIS: območje karte ni več v zvezni deželi s storitvijo NAS
warning-nas-folder-outside = polni sloji ALKIS preskočeni ({ $folder } vodi iz mape projekta)
progress-nas-download = prenašam polne strukture ALKIS (NAS)
progress-nas-features = ALKIS NAS { $file }: objektov { $count }
progress-nas-skipped = preskočeni objekti ALKIS NAS { $file } ({ $count }): { $list }
warning-nas-layer-kept = ALKIS NAS { $file } ni na voljo ({ $detail }); ohranjen je shranjeni sloj
warning-nas-layer-skipped = ALKIS NAS { $file } ni na voljo ({ $detail }); sloj preskočen

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = objekti ATKIS Basis-DLM niso shranjeni; za izvire, jame, stolpe, spomenike in vodnjake zaženi `freemapper fetch` z omrežjem
progress-dlm-removing = odstranjujem shranjene sloje ATKIS Basis-DLM: območje karte ni več v zvezni deželi, ki jih streže
warning-dlm-folder-outside = sloji ATKIS Basis-DLM preskočeni ({ $folder } vodi iz mape projekta)
progress-dlm-download = prenašam objekte ATKIS Basis-DLM
progress-dlm-features = ATKIS Basis-DLM { $file }: objektov { $count }
progress-dlm-skipped = preskočeni objekti ATKIS Basis-DLM { $file } ({ $count }): { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } ni na voljo ({ $detail }); ohranjen je shranjeni sloj
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } ni na voljo ({ $detail }); sloj preskočen

## Tree cadastres

progress-trees-none-stored = mestni kataster dreves ni shranjen; za posamezna drevesa zaženi `freemapper fetch` z omrežjem
progress-trees-removing = odstranjujem shranjene katastre dreves: območje karte ni več v mestu z njim
warning-trees-folder-outside = mestni katastri dreves preskočeni ({ $folder } vodi iz mape projekta)
progress-trees-download = prenašam mestne katastre dreves
progress-trees-count = { $id }: dreves { $count }
progress-trees-skipped = preskočena drevesa { $id } ({ $count }): { $list }
warning-trees-kept = kataster dreves { $id } ni na voljo ({ $detail }); ohranjen je shranjeni
warning-trees-skipped = kataster dreves { $id } ni na voljo ({ $detail }); drevesa preskočena

## Deriving the templates

progress-writing = zapisujem { $path }
progress-contours-steep = plastnice na vsakih { $interval } m: svet je tako strm, da bi se plastnice na vsakih { $default } m zlile; ekvidistanca { $default } m v projektu jih ohrani (`--contour-interval { $default }`)
warning-osm-only-no-terrain = za to regijo še ni podatkov o terenu: karta nastane samo iz OpenStreetMap, brez plastnic, predlog in stavb
progress-no-ndsm-recorded = nDSM ni zabeležen; za predlogo nDSM zaženi `freemapper fetch`
warning-ndsm-tile-invalid = neveljavna ploščica nDSM { $tile } v freemapper.toml; preskočena
warning-ndsm-not-cached = nDSM { $tile } ni v predpomnilniku; zaženi `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } ni berljiv ({ $error }); izbriši { $path } in zaženi `freemapper fetch`
warning-surface-unusable = model površja je neuporaben ({ $error }); zaženi `freemapper fetch`

## Downloads and the cache

progress-downloading = prenašam { $url }
progress-downloaded = preneseno { $megabytes } MB
progress-downloaded-of = preneseno { $megabytes } MB od { $total } MB
progress-tile-index-download = prenašam kazalo ploščic { $product }
progress-tile-cached = v predpomnilniku: { $tile }
progress-surface-blocks = berem { $count } od { $total } blokov ploščice modela površja { $tile } ({ $megabytes } MB)
progress-surface-whole = ploščica modela površja { $tile } se prenese v celoti: { $reason }
warning-tile-cached-older = uporabljam predpomnjeno ploščico { $product } { $tile } z dne { $published } (novejša { $newer } ni prenesena)
warning-ndsm-no-cached = za ploščico { $tile } ni predpomnjenega nDSM
warning-ndsm-tile-unavailable-cached = ploščica nDSM { $tile } ni na voljo ({ $reason }); uporabljam predpomnjeno datoteko
warning-ndsm-tile-unavailable-skipped = ploščica nDSM { $tile } ni na voljo ({ $reason }); preskočena
warning-file-check-failed = ni bilo mogoče preveriti, ali je { $file } še aktualna ({ $reason }); uporabljam predpomnjeno datoteko in druge predpomnjene datoteke tega strežnika brez vprašanja
warning-ldbv-no-buildings-file = na strežniku ni datoteke stavb za ploščico { $tile }; karti tam morda manjkajo stavbe
progress-ndsm-not-published = za ploščico { $tile } ni objavljenega oblaka točk nDSM
progress-ndsm-cached = predpomnjen nDSM { $tile }
progress-point-cloud-not-published = za ploščico { $tile } ni objavljenega laserskega oblaka točk
progress-point-cloud-cached = predpomnjen oblak točk { $tile }
warning-point-cloud-no-cached = za ploščico { $tile } ni predpomnjenega oblaka točk; tam ni kandidatov vegetacije
warning-point-cloud-unavailable-cached = oblak točk { $tile } ni na voljo ({ $reason }); uporabljam predpomnjeno datoteko
warning-point-cloud-unavailable-skipped = oblak točk { $tile } ni na voljo ({ $reason }); tam ni kandidatov vegetacije
warning-point-clouds-unavailable = laserski oblaki točk niso na voljo ({ $reason }); kandidati vegetacije uporabijo predpomnjene ploščice, če obstajajo
progress-no-point-clouds = za vegetacijo tu še ni laserskih oblakov točk: zvezna dežela ne objavlja nobenih, ki bi jih freemapper lahko prebral, ali ne v koordinatnem sistemu karte
warning-point-clouds-not-fetched = vegetacija je izbrana, vendar noben oblak točk ni naložen; za nalaganje zaženi `freemapper fetch`
warning-point-cloud-unreadable = oblak točk { $tile } ni berljiv ({ $error }); izbriši { $path } in zaženi `freemapper fetch`
warning-osm-nrw-nearest = v 100 m od območja ni izvlečka okraja Severnega Porenja-Vestfalije; berem najbližjega, { $district }
warning-osm-disk-full-cached = disk je poln, zato freemapper uporablja predpomnjeni izvleček OSM z dne { $date }. Sprosti nekaj prostora in znova zaženi `freemapper fetch` za novejšega
warning-osm-geofabrik-unavailable = Geofabrik ni na voljo ({ $error }); uporabljam predpomnjeni izvleček OSM z dne { $date }
progress-osm-extract-cached = predpomnjen izvleček OSM { $region } z dne { $date }
progress-osm-extract-download = prenašam { $url } (približno { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = prenašam ploščico bDOM { $tile } ({ $megabytes } MB, { $index } od { $total })
progress-sh-dgm-not-in-atom = ploščice DGM1 { $tile } ni v imeniku INSPIRE ATOM; prenašam njen XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = ploščica DGM1 { $tile } iz imenika INSPIRE ATOM ni berljiva ({ $error }); prenašam njen XYZ ({ $megabytes } MB)
progress-sh-tile = ploščica { $tile }: { $text }
progress-sh-download-again = ploščica { $tile }: prenos ni berljiv ({ $message }); prenašam jo še enkrat
warning-sh-portal-no-date = Downloadportal ne navaja datuma za ploščico { $model } { $tile }
warning-sh-dgm-xyz-used = ploščice DGM1 { $tile } ni v imeniku INSPIRE ATOM; uporabljen je njen XYZ iz Downloadportala
warning-sh-dgm-xyz-used-unreadable = ploščica DGM1 { $tile } iz imenika INSPIRE ATOM ni berljiva ({ $error }); uporabljen je njen XYZ iz Downloadportala

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, ploščica { $tile }: celic območja karte brez višine: { $count }
warning-centre-tile-missing = ploščice { $model } { $tile } v središču območja karte ni, zato ni datuma
warning-centre-tile-unlisted = seznam ploščic ne navaja ploščice { $model } { $tile } v središču območja karte, zato ni datuma
warning-list-unreadable-kept = seznama ploščic modela { $model } ni bilo mogoče prebrati ({ $error }); uporabljen je tisti iz prejšnjega zagona
warning-portal-list-unreadable-kept = seznama ploščic modela { $model } iz Downloadportala ni bilo mogoče prebrati ({ $error }); uporabljen je tisti iz prejšnjega zagona
warning-surface-much-older = { $model } je iz { $surface }, ortofoto iz { $flight }. V nDSM morda manjkajo novejše stavbe in drevesa.

## Rhineland-Palatinate

warning-rp-list-features-unusable = objektov seznama ploščic brez uporabnega ključa ploščice: { $count }; preskočeni
warning-rp-list-unreadable-kept = seznama ploščic LVermGeo RP ni bilo mogoče prebrati ({ $error }); uporabljen je tisti iz prejšnjega zagona
warning-rp-metalink-not-kept = Metalink LVermGeo RP ni bil ohranjen: { $error }
warning-rp-metalink-unreadable-kept = Metalinka LVermGeo RP za { $model } ni bilo mogoče prebrati ({ $detail }); uporabljen je tisti iz prejšnjega zagona
warning-rp-metalink-unreadable = Metalinka LVermGeo RP za { $model } ni bilo mogoče prebrati ({ $detail }), zato ploščice, za katere seznam ploščic ne navaja leta preleta, ostanejo brez višin
warning-rp-tile-without-name = { $model }, ploščica { $tile }: ta kvadrat 1 km nima višin na karti. Seznam ploščic LVermGeo RP (Laserbefliegung) { $note } zanj, zato ime datoteke ni znano{ $either ->
        [metalink] , in tudi njegov Metalink zanj ne navaja nobene datoteke
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }, ploščica { $tile } nima datuma: seznam ploščic LVermGeo RP (Laserbefliegung) { $note } zanjo
word-rp-list-neither = ne navaja ne leta preleta ne dni preleta
word-rp-list-no-days = ne navaja dni preleta
word-rp-list-no-year = ne navaja leta preleta
word-rp-list-days-unreadable = navaja dneve preleta { $days }, ki jih ni mogoče prebrati
word-rp-list-disagree = navaja dneve preleta { $days } in leto preleta { $year }, ki se ne ujemata v enem letu

## Hesse

warning-hvbg-undated = seznam metapodatkov nima berljivega Bildflugdatum za ploščico { $tiles }; njihovi zapisi projekta nimajo datuma preleta
warning-hvbg-undated-more = seznam metapodatkov nima berljivega Bildflugdatum za ploščico { $tiles } in še { $more }; njihovi zapisi projekta nimajo datuma preleta
warning-hvbg-list-unreadable-older = novega seznama metapodatkov HVBG { $name } ni bilo mogoče prebrati ({ $detail }); uporabljam starejši seznam { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: brez datuma, STAC v središču območja karte ne navaja nobene ploščice
warning-lgln-no-date = { $model }: brez datuma ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; shranjeni model ohrani svoj zabeleženi datum
warning-mv-cells-empty = { $model }: celic znotraj ploščic Mecklenburg-Vorpommern brez višine: { $count }
warning-mv-centre-no-date = { $model }: brez datuma za osrednjo ploščico { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = vrstic seznama ploščic { $model } brez uporabnega imena ploščice: { $count }; preskočene
warning-nrw-list-rows-undated = vrstic seznama ploščic { $model } brez berljivega datuma: { $count }
warning-nrw-list-older = seznam ploščic modela { $model } na strežniku je z dne { $server }, starejši od ohranjenega z dne { $kept }; uporabljen je ohranjeni
warning-nrw-list-no-date = seznam ploščic ne navaja datuma za ploščico { $model } { $tile }

## After `init`

note-scale-outside-norm = opomba: 1:5000 ni v normi za sprint; freemapper-print na to opozori, simboli pa ostanejo pri 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } ni id OSM, kot je way/992423119
error-place-invalid-answer = neveljaven odgovor iskanja kraja: { $detail }
error-place-not-found = { $id } ni najden. Nominatim navaja samo označene objekte OSM, zato neoznačena zunanja pot multipoligona tam ni. Uporabi raje relacijo multipoligona (--osm relation/<id>) ali shrani obris kot GeoJSON in uporabi --geojson
error-place-is-a-point = { $id } je točka, ne območje; izberi njegovo zemljišče (pot ali relacijo)
error-place-no-such-part = { $id } ima { $count ->
        [one] 1 del
        [two] { $count } dela
        [few] { $count } dele
       *[other] { $count } delov
    }; dela { $part } ni
error-place-several-parts = { $id } je območje iz { $count } ločenih delov ({ $sizes }). Območje karte je en mnogokotnik: izberi del z --part <n> (1 je največji) ali shrani želeni obris kot GeoJSON in uporabi --geojson
error-layer-not-a-world-file = { $path }: ni world datoteka
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = za { $layer } ni predogleda; sloji s predogledi: { $names }
error-layer-no-ndsm-published = za ploščice tega območja karte nDSM ni objavljen, zato predloge ndsm ni
error-layer-not-made = { $layer } še ni izdelan; zaženi { $step ->
        [fetch] freemapper fetch ali run (MCP: start_pipeline); samo Berlin
        [export] freemapper export ali run (MCP: start_pipeline)
       *[derive] freemapper derive ali run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; karta OCAD iz tega ne dobi ozadja
warning-map-preview-failed = karta je zapisana, njen predogled pa manjka: { $error }
warning-training-no-tree-cover = vadbena karta ohrani odprte površine parkov cele: pokritost z drevesi (data/tree-cover.tif) manjka; znova zaženi `freemapper derive`
