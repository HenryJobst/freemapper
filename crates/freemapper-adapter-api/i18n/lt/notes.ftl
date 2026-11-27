# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = įspėjimas: { $text }

warning-other-zone = žemėlapio plotas yra UTM zonoje { $zone } (EPSG:{ $epsg }); freemapper iš naujo atsisiunčia viską šioje zonoje
warning-other-state = žemėlapio plotas dabar yra kitos federalinės žemės duomenų ribose; freemapper iš naujo atsisiunčia viską
warning-no-orthophoto = šiam regionui ortofotografijos nėra; žemėlapis neturi ortofotografijos šablono
warning-beyond-coverage = dalis žemėlapio ploto yra už palaikomų federalinių žemių ribų: užsienyje, jūroje arba federalinėje žemėje, kurios freemapper dar nepalaiko, pavyzdžiui, Tiuringijoje. Ten federalinės žemės duomenų neturi, todėl ortofoto gali būti baltas, o ALKIS nepateikia nei pastatų, nei žemės naudojimo
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: mastelis 1:{ $scale } nėra tarp { $kind ->
        [schoolyard] mokyklos kiemo
        [forest] miško
       *[sprint] sprinto
    } žemėlapių masteliu ({ $allowed }); freemapper-print žemėlapį atmes
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: { $what ->
        [tree] medžių
        [NAS] NAS
        [set] papildomų
       *[other] { $german }
    } sluoksnių įrašas { $key } nepaisomas: { $reason ->
        [name] tai ne paprastas failo vardas
       *[unknown] freemapper tokio sluoksnio nepažįsta
    }

## Disk space

warning-disk-space-unreadable = nepavyksta nuskaityti laisvos disko vietos { $path } ({ $error }), todėl freemapper jos netikrina

## Tiles

warning-tile-changed = { $product } plytelė { $tile } tiekėjo pusėje pasikeitė ({ $old } → { $new })
warning-bdom-tile-unavailable = plytelės { $tile } paviršiaus modelis (bDOM) nepasiekiamas ({ $error }); ten nDSM šablono nėra
warning-bdom-unavailable = paviršiaus modelis (bDOM) nepasiekiamas ({ $error }); { $tiles } nDSM šablono nėra
progress-ndsm-from-bdom = plytelės { $tile } nDSM iš paviršiaus modelio (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] reljefo modelis
        [surface] paviršiaus modelis
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = atsisiunčiama: { $model }
progress-model-read = skaitoma: { $model }
warning-model-unavailable = nepasiekiama: { $model } ({ $error }); nDSM šablono nėra
warning-model-unavailable-kept = nepasiekiama: { $model } ({ $error }); paliekamas { $fetched } atsisiųstas paviršiaus modelis
warning-model-tile-unavailable = nepasiekiama: plytelės { $tile } { $model } ({ $error }); ten nDSM šablono nėra
warning-archive-passed-over = { $model }: archyvo nepavyko perskaityti ({ $error }); jis praleistas, o žemėlapio ploto plytelės imamos iš kitų archyvų
warning-archive-directory-kept = { $model }: archyvo nepavyko perskaityti ({ $error }); naudojamas ankstesnio paleidimo išsaugotas jo katalogas, todėl plytelės, kurią LVGL nuo tada pakeitė, nepastebėsime
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] žemėlapio plote
        [hamburg] Hamburge
        [hesse] Hese
        [north-rhine-westphalia] Šiaurės Reine-Vestfalijoje
        [schleswig-holstein] Šlėzvige-Holšteine
        [mecklenburg-vorpommern] Meklenburge-Priekinėje Pomeranijoje
        [lower-saxony] Žemutinėje Saksonijoje
        [bavaria] Bavarijoje
       *[other] { $state }
    } trūksta { $count ->
        [one] plytelės
       *[other] plytelių
    } { $names }; šablonai ten lieka tušti
warning-no-surface-stored = paviršiaus modelis neišsaugotas; nDSM šablonui paleiskite `freemapper fetch` su tinklo ryšiu
warning-surface-older-than-orthophoto = paviršiaus modelis (bDOM { $surface }) senesnis už ortofoto ({ $photo }); nDSM gali trūkti naujų pastatų
progress-surface-kept = freemapper nDSM šablonui palieka { $fetched } atsisiųstą paviršiaus modelį
progress-surface-tiles-kept = freemapper nDSM šablonui palieka talpykloje esantį paviršiaus modelį { $count ->
        [one] plytelės
       *[other] plytelių
    } { $names }
progress-orthophoto-unchanged = ortofoto nepasikeitė
progress-orthophoto-unchanged-dated = ortofoto nepasikeitė ({ $label } { $date })
progress-orthophoto-download = ortofoto atsisiunčiama iš { $provider } { $service }
progress-orthophoto-in-leaf = ortofoto skraidinta { $date }, medžiai buvo su lapais; jų lajos dengia tai, kas po jomis
progress-orthophoto-bare = ortofoto skraidinta { $date }, medžiai buvo be lapų
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; paliekamas išsaugotas ortofoto ir jo skrydžio data { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): skrydžio datos nėra, { $reason }
word-stac-no-image = STAC žemėlapio ploto centre nenurodo jokio vaizdo
word-stac-no-answer = STAC paieška nedavė tinkamo atsakymo ({ $error })
warning-downloadcenter-no-file = Downloadcenter neturi failo { $places }
warning-downloadcenter-no-file-empty = Downloadcenter neturi failo { $places }; { $count ->
        [one] plytelė
        [few] plytelės
       *[other] plytelių
    } { $names } liko tuščios
progress-k-five-unchanged = K5 šablonas nepasikeitė
progress-k-five-download = K5 žemėlapis (1:5000) atsisiunčiamas iš Geoportal Berlin WMS
warning-k-five-unavailable = K5 žemėlapio šablonas nepasiekiamas ({ $detail }); praleista

## The state border

progress-beyond-border = žemėlapio plotas siekia už federalinės žemės ribos; už jos šablonai tušti ten, kur baigiasi { $provider } ortofoto ir aukščių modeliai. ALKIS ir OSM imami iš abiejų federalinių žemių
progress-beyond-border-no-data = žemėlapio plotas siekia už federalinės žemės ribos; { $tiles } { $provider } duomenų nėra, todėl šablonai ten lieka tušti

## OSM

warning-osm-extract-missing-kept = talpykloje nėra OSM ištraukos; paliekami išsaugoti OSM kandidatai iš { $date }
progress-osm-extract-read = skaitoma OSM ištrauka { $region } iš { $date }
progress-osm-candidates = { $count ->
        [one] { $count } OSM objektas kandidatas
        [few] { $count } OSM objektai kandidatai
       *[other] { $count } OSM objektų kandidatų
    }

## Permit notes

progress-permits-fetch = leidimų pastaboms atsisiunčiami { $source } duomenys
progress-permits-osm = leidimų pastaboms skaitoma OSM ištrauka { $region } iš { $date }
progress-permits-stored = skaitomi išsaugoti leidimų pastabų duomenys
warning-permits-source-unreachable = { $source } nepavyko perskaityti, todėl leidimų pastabose jo nėra: { $error }
warning-permits-osm-no-extract = šiam žemėlapio plotui talpykloje nėra OSM ištraukos, todėl leidimų pastabose OSM nėra; `freemapper fetch` ją atsisiunčia

## ALKIS

warning-alkis-coverage-unknown = { $cadastre } pastatų aprėptis nežinoma; paleiskite `freemapper fetch` su tinklo ryšiu, kad ji būtų užfiksuota
progress-alkis-buildings-download = atsisiunčiami { $cadastre } pastatai
progress-alkis-buildings = { $count ->
        [one] { $count } { $cadastre } pastatas
        [few] { $count } { $cadastre } pastatai
       *[other] { $count } { $cadastre } pastatų
    }
warning-alkis-no-buildings = žemėlapio plote { $cadastre } pastatų nerasta
progress-alkis-skipped = praleista { $count ->
        [one] { $count } { $cadastre } objektas
        [few] { $count } { $cadastre } objektai
       *[other] { $count } { $cadastre } objektų
    }: { $list }
warning-alkis-buildings-changed = { $cadastre } pastatai tiekėjo pusėje pasikeitė ({ $changes })
progress-alkis-no-land-use-stored = { $cadastre } žemės naudojimas neišsaugotas; kandidatams už ribų paleiskite `freemapper fetch` su tinklo ryšiu
progress-alkis-land-use-download = atsisiunčiamas { $cadastre } žemės naudojimas
progress-alkis-land-use-parts = { $count ->
        [one] { $count } { $cadastre } žemės naudojimo dalis
        [few] { $count } { $cadastre } žemės naudojimo dalys
       *[other] { $count } { $cadastre } žemės naudojimo dalių
    }
warning-alkis-land-use-kept = { $cadastre } žemės naudojimas nepasiekiamas ({ $error }); paliekamas išsaugotas žemės naudojimas
warning-alkis-land-use-removed = { $cadastre } žemės naudojimas nepasiekiamas ({ $error }); išsaugotas žemės naudojimas nebeapima žemėlapio ploto, todėl freemapper jį pašalino
warning-alkis-land-use-none = { $cadastre } žemės naudojimas nepasiekiamas ({ $error }); kandidatų už ribų nėra
progress-bdot-filled = pridėta { $count } BDOT10k objektų ({ $what }), kur EGiB nieko neduoda ({ $powiat })
warning-bdot-failed = BDOT10k negalėjo užpildyti EGiB spragos ({ $what }): { $error }
warning-bdot-instead = EGiB nepasiekiamas ({ $error }); vietoj jo naudojamas BDOT10k ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = Berlyno ALKIS statinių neišsaugota; laiptams, sienoms, medžiams ir stogeliams paleiskite `freemapper fetch` su tinklo ryšiu
progress-berlin-removing = šalinami išsaugoti Berlyno sluoksniai: žemėlapio plotas nebėra netoli Berlyno
progress-berlin-download = atsisiunčiami Berlyno ALKIS statiniai, medžių kadastras, gatvių apžvalga ir taškų sluoksniai
progress-berlin-features = { $count ->
        [one] { $count } Berlyno { $file } objektas
        [few] { $count } Berlyno { $file } objektai
       *[other] { $count } Berlyno { $file } objektų
    }
warning-berlin-layer-kept = Berlyno { $file } nepasiekiamas ({ $detail }); paliekamas išsaugotas sluoksnis
warning-berlin-layer-skipped = Berlyno { $file } nepasiekiamas ({ $detail }); sluoksnis praleistas
progress-feinkartierung-none-stored = Hamburgo Feinkartierung neišsaugotas; turėklams, sienoms, laiptams ir gatvės įrangai paleiskite `freemapper fetch` su tinklo ryšiu
progress-feinkartierung-removing = šalinamas išsaugotas Hamburgo Feinkartierung: žemėlapio plotas nebėra Hamburge
progress-feinkartierung-download = atsisiunčiamas Hamburgo Feinkartierung (gatvių erdvė ir viešoji žaluma)
progress-feinkartierung-features = { $count ->
        [one] { $count } Feinkartierung { $file } objektas
        [few] { $count } Feinkartierung { $file } objektai
       *[other] { $count } Feinkartierung { $file } objektų
    }
warning-feinkartierung-layer-kept = Feinkartierung { $file } nepasiekiamas ({ $detail }); paliekamas išsaugotas sluoksnis
warning-feinkartierung-layer-skipped = Feinkartierung { $file } nepasiekiamas ({ $detail }); sluoksnis praleistas
progress-bdot-roads-none-stored = išsaugotų BDOT10k kelių ir takų nėra; paleiskite `freemapper fetch` prisijungę prie tinklo, kad gautumėte registro kelius ir takus
progress-bdot-roads-removing = šalinami išsaugoti BDOT10k keliai ir takai: žemėlapio sritis nebėra Lenkijoje
progress-bdot-roads-download = atsisiunčiami BDOT10k keliai ir takai
progress-bdot-roads-features = BDOT10k { $file } objektų: { $count }
warning-bdot-roads-layer-kept = BDOT10k { $file } nepasiekiamas ({ $detail }); paliekamas išsaugotas sluoksnis
warning-bdot-roads-layer-skipped = BDOT10k { $file } nepasiekiamas ({ $detail }); sluoksnis praleistas
error-extra-layer-unknown = tai ne { $folder } sluoksnis
warning-berlin-buildings-missing = šiam Berlyno plotui ALKIS pastatų neišsaugota; paleiskite `freemapper fetch`, kad gautumėte juos iš Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = pilnų ALKIS statinių neišsaugota; sienoms, tvoroms, gyvatvorėms ir tiltams paleiskite `freemapper fetch` su tinklo ryšiu
progress-nas-removing = šalinami išsaugoti pilni ALKIS sluoksniai: žemėlapio plotas nebėra federalinėje žemėje su NAS paslauga
warning-nas-folder-outside = pilni ALKIS sluoksniai praleisti ({ $folder } išeina už projekto aplanko ribų)
progress-nas-download = atsisiunčiami pilni ALKIS statiniai (NAS)
progress-nas-features = { $count ->
        [one] { $count } ALKIS NAS { $file } objektas
        [few] { $count } ALKIS NAS { $file } objektai
       *[other] { $count } ALKIS NAS { $file } objektų
    }
progress-nas-skipped = praleista { $count ->
        [one] { $count } ALKIS NAS { $file } objektas
        [few] { $count } ALKIS NAS { $file } objektai
       *[other] { $count } ALKIS NAS { $file } objektų
    }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } nepasiekiamas ({ $detail }); paliekamas išsaugotas sluoksnis
warning-nas-layer-skipped = ALKIS NAS { $file } nepasiekiamas ({ $detail }); sluoksnis praleistas

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = ATKIS Basis-DLM objektų neišsaugota; šaltiniams, urvams, bokštams, paminklams ir fontanams paleiskite `freemapper fetch` su tinklo ryšiu
progress-dlm-removing = šalinami išsaugoti ATKIS Basis-DLM sluoksniai: žemėlapio plotas nebėra federalinėje žemėje, kuri juos teikia
warning-dlm-folder-outside = ATKIS Basis-DLM sluoksniai praleisti ({ $folder } išeina už projekto aplanko ribų)
progress-dlm-download = atsisiunčiami ATKIS Basis-DLM objektai
progress-dlm-features = { $count ->
        [one] { $count } ATKIS Basis-DLM { $file } objektas
        [few] { $count } ATKIS Basis-DLM { $file } objektai
       *[other] { $count } ATKIS Basis-DLM { $file } objektų
    }
progress-dlm-skipped = praleista { $count ->
        [one] { $count } ATKIS Basis-DLM { $file } objektas
        [few] { $count } ATKIS Basis-DLM { $file } objektai
       *[other] { $count } ATKIS Basis-DLM { $file } objektų
    }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } nepasiekiamas ({ $detail }); paliekamas išsaugotas sluoksnis
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } nepasiekiamas ({ $detail }); sluoksnis praleistas

## Tree cadastres

progress-trees-none-stored = miesto medžių kadastras neišsaugotas; pavieniams medžiams paleiskite `freemapper fetch` su tinklo ryšiu
progress-trees-removing = šalinami išsaugoti medžių kadastrai: žemėlapio plotas nebėra mieste, kuris jį turi
warning-trees-folder-outside = miestų medžių kadastrai praleisti ({ $folder } išeina už projekto aplanko ribų)
progress-trees-download = atsisiunčiami miestų medžių kadastrai
progress-trees-count = { $count ->
        [one] { $count } medis
        [few] { $count } medžiai
       *[other] { $count } medžių
    } iš { $id }
progress-trees-skipped = praleista { $count ->
        [one] { $count } medis
        [few] { $count } medžiai
       *[other] { $count } medžių
    } iš { $id }: { $list }
warning-trees-kept = { $id } medžių kadastras nepasiekiamas ({ $detail }); paliekamas išsaugotas
warning-trees-skipped = { $id } medžių kadastras nepasiekiamas ({ $detail }); medžiai praleisti

## Deriving the templates

progress-writing = rašoma { $path }
progress-contours-steep = horizontalės kas { $interval } m: reljefas toks status, kad horizontalės kas { $default } m susiliptų; horizontalių intervalas { $default } m projekte jas išlaiko (`--contour-interval { $default }`)
warning-osm-only-no-terrain = šiam regionui reljefo duomenų dar nėra: žemėlapis kuriamas vien iš OpenStreetMap, be horizontalių, šablonų ir pastatų
progress-no-ndsm-recorded = nDSM neužfiksuotas; nDSM šablonui paleiskite `freemapper fetch`
warning-ndsm-tile-invalid = netinkama nDSM plytelė { $tile } faile freemapper.toml; praleista
warning-ndsm-not-cached = nDSM { $tile } talpykloje nėra; paleiskite `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } neperskaitomas ({ $error }); ištrinkite { $path } ir paleiskite `freemapper fetch`
warning-surface-unusable = paviršiaus modelis netinkamas naudoti ({ $error }); paleiskite `freemapper fetch`

## Downloads and the cache

progress-downloading = atsisiunčiama { $url }
progress-retry-wait = paslauga užimta; kitas bandymas po { $seconds } s
progress-downloaded = atsisiųsta { $megabytes } MB
progress-downloaded-of = atsisiųsta { $megabytes } MB iš { $total } MB
progress-tile-index-download = atsisiunčiamas { $product } plytelių indeksas
progress-tile-cached = talpykloje: { $tile }
progress-surface-blocks = skaitoma { $count } iš { $total } paviršiaus modelio plytelės { $tile } blokų ({ $megabytes } MB)
progress-surface-whole = paviršiaus modelio plytelė { $tile } atsisiunčiama visa: { $reason }
warning-tile-cached-older = naudojama talpykloje esanti { $product } plytelė { $tile } iš { $published } (naujesnė { $newer } neatsisiųsta)
warning-ndsm-no-cached = plytelei { $tile } talpykloje nDSM nėra
warning-ndsm-tile-unavailable-cached = nDSM plytelė { $tile } nepasiekiama ({ $reason }); naudojamas talpykloje esantis failas
warning-ndsm-tile-unavailable-skipped = nDSM plytelė { $tile } nepasiekiama ({ $reason }); praleista
warning-file-check-failed = nepavyko patikrinti, ar { $file } vis dar aktualus ({ $reason }); naudojamas talpykloje esantis failas ir kiti šio serverio talpykloje esantys failai be patikros
warning-ldbv-no-buildings-file = plytelei { $tile } serveryje nėra pastatų failo; žemėlapyje ten gali trūkti pastatų
progress-ndsm-not-published = plytelei { $tile } nDSM taškų debesis neskelbiamas
progress-ndsm-cached = talpykloje nDSM { $tile }
progress-point-cloud-not-published = plytelei { $tile } lazerinių taškų debesis neskelbiamas
progress-point-cloud-cached = talpykloje taškų debesis { $tile }
warning-point-cloud-no-cached = plytelei { $tile } talpykloje taškų debesies nėra; ten augmenijos kandidatų nėra
warning-point-cloud-unavailable-cached = taškų debesis { $tile } nepasiekiamas ({ $reason }); naudojamas talpykloje esantis failas
warning-point-cloud-unavailable-skipped = taškų debesis { $tile } nepasiekiamas ({ $reason }); ten augmenijos kandidatų nėra
warning-point-clouds-unavailable = lazerinių taškų debesys nepasiekiami ({ $reason }); augmenijos kandidatai naudoja talpykloje esančias plyteles, jei jų yra
progress-no-point-clouds = augmenijai čia lazerinių taškų debesų dar nėra: federalinė žemė neskelbia nė vieno, kurį freemapper gali perskaityti, arba jie ne žemėlapio koordinačių sistemoje
warning-point-clouds-not-fetched = augmenija pasirinkta, bet taškų debesys neįkelti; paleiskite `freemapper fetch`, kad juos įkeltumėte
warning-point-cloud-unreadable = taškų debesis { $tile } neperskaitomas ({ $error }); ištrinkite { $path } ir paleiskite `freemapper fetch`
warning-osm-nrw-nearest = per 100 m nuo ploto nėra Šiaurės Reino-Vestfalijos apskrities ištraukos; skaitoma artimiausia, { $district }
warning-osm-disk-full-cached = diskas pilnas, todėl freemapper naudoja talpykloje esančią OSM ištrauką iš { $date }. Atlaisvinkite vietos ir vėl paleiskite `freemapper fetch`, kad gautumėte naujesnę
warning-osm-geofabrik-unavailable = Geofabrik nepasiekiamas ({ $error }); naudojama talpykloje esanti OSM ištrauka iš { $date }
progress-osm-extract-cached = talpykloje OSM ištrauka { $region } iš { $date }
progress-osm-extract-download = atsisiunčiama { $url } (apie { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = atsisiunčiama bDOM plytelė { $tile } ({ $megabytes } MB, { $index } iš { $total })
progress-sh-dgm-not-in-atom = DGM1 plytelės { $tile } INSPIRE ATOM kataloge nėra; atsisiunčiamas jos XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1 plytelė { $tile } iš INSPIRE ATOM katalogo neskaitoma ({ $error }); atsisiunčiamas jos XYZ ({ $megabytes } MB)
progress-sh-tile = plytelė { $tile }: { $text }
progress-sh-download-again = plytelė { $tile }: atsisiųstas failas neskaitomas ({ $message }); atsisiunčiama dar kartą
warning-sh-portal-no-date = Downloadportal nenurodo { $model } plytelės { $tile } datos
warning-sh-dgm-xyz-used = DGM1 plytelės { $tile } INSPIRE ATOM kataloge nėra; naudotas jos XYZ iš Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1 plytelė { $tile } iš INSPIRE ATOM katalogo neskaitoma ({ $error }); naudotas jos XYZ iš Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model } plytelė { $tile }: { $count ->
        [one] { $count } žemėlapio ploto ląstelė neturi
        [few] { $count } žemėlapio ploto ląstelės neturi
       *[other] { $count } žemėlapio ploto ląstelių neturi
    } aukščio
warning-centre-tile-missing = žemėlapio ploto centre trūksta { $model } plytelės { $tile }, todėl datos nėra
warning-centre-tile-unlisted = plytelių sąraše žemėlapio ploto centre nėra { $model } plytelės { $tile }, todėl datos nėra
warning-list-unreadable-kept = { $model } plytelių sąrašo nepavyko perskaityti ({ $error }); naudotas ankstesnio paleidimo išsaugotas sąrašas
warning-portal-list-unreadable-kept = Downloadportal { $model } plytelių sąrašo nepavyko perskaityti ({ $error }); naudotas ankstesnio paleidimo išsaugotas sąrašas
warning-surface-much-older = { $model } yra iš { $surface }, ortofoto iš { $flight }. nDSM gali trūkti naujesnių pastatų ir medžių.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count ->
        [one] { $count } plytelių sąrašo objektas neturi
        [few] { $count } plytelių sąrašo objektai neturi
       *[other] { $count } plytelių sąrašo objektų neturi
    } tinkamo plytelės rakto; praleista
warning-rp-list-unreadable-kept = LVermGeo RP plytelių sąrašo nepavyko perskaityti ({ $error }); naudotas ankstesnio paleidimo išsaugotas sąrašas
warning-rp-metalink-not-kept = LVermGeo RP Metalink neišsaugotas: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP { $model } Metalink nepavyko perskaityti ({ $detail }); naudotas ankstesnio paleidimo išsaugotas
warning-rp-metalink-unreadable = LVermGeo RP { $model } Metalink nepavyko perskaityti ({ $detail }), todėl plytelės, kurioms jos plytelių sąrašas nenurodo skrydžio metų, lieka be aukščių
warning-rp-tile-without-name = { $model } plytelė { $tile }: šis 1 km kvadratas žemėlapyje aukščių neturi. LVermGeo RP plytelių sąraše (Laserbefliegung) jai { $note }, todėl failo vardas nežinomas{ $either ->
        [metalink] , o jos Metalink jai taip pat nenurodo failo
       *[none] {""}
    }
warning-rp-centre-no-date = { $model } plytelė { $tile } datos neturi: LVermGeo RP plytelių sąraše (Laserbefliegung) jai { $note }
word-rp-list-neither = nenurodyti nei skrydžio metai, nei skrydžio dienos
word-rp-list-no-days = nenurodytos skrydžio dienos
word-rp-list-no-year = nenurodyti skrydžio metai
word-rp-list-days-unreadable = nurodytos skrydžio dienos { $days }, kurių neįmanoma perskaityti
word-rp-list-disagree = nurodytos skrydžio dienos { $days } ir skrydžio metai { $year }, kurie nesutampa dėl metų

## Hesse

warning-hvbg-undated = metaduomenų sąraše plytelei { $tiles } nėra perskaitomo Bildflugdatum; jų projekto įrašuose skrydžio datos nėra
warning-hvbg-undated-more = metaduomenų sąraše plytelei { $tiles } ir dar { $more } nėra perskaitomo Bildflugdatum; jų projekto įrašuose skrydžio datos nėra
warning-hvbg-list-unreadable-older = naujo HVBG metaduomenų sąrašo { $name } nepavyko perskaityti ({ $detail }); naudojamas senesnis sąrašas { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: datos nėra, STAC žemėlapio ploto centre nenurodo jokios plytelės
warning-lgln-no-date = { $model }: datos nėra ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; išsaugotas modelis pasilieka užfiksuotą datą
warning-mv-cells-empty = { $model }: { $count ->
        [one] { $count } ląstelė Meklenburgo-Priekinės Pomeranijos plytelėse neturi
        [few] { $count } ląstelės Meklenburgo-Priekinės Pomeranijos plytelėse neturi
       *[other] { $count } ląstelių Meklenburgo-Priekinės Pomeranijos plytelėse neturi
    } aukščio
warning-mv-centre-no-date = { $model }: centrinei plytelei { $tile } datos nėra

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $model } plytelių sąrašo { $count ->
        [one] { $count } eilutė neturi
        [few] { $count } eilutės neturi
       *[other] { $count } eilučių neturi
    } tinkamo plytelės pavadinimo; praleista
warning-nrw-list-rows-undated = { $model } plytelių sąrašo { $count ->
        [one] { $count } eilutė neturi
        [few] { $count } eilutės neturi
       *[other] { $count } eilučių neturi
    } perskaitomos datos
warning-nrw-list-older = serverio { $model } plytelių sąrašas yra iš { $server }, senesnis už išsaugotąjį iš { $kept }; naudotas išsaugotasis
warning-nrw-list-no-date = plytelių sąrašas nenurodo { $model } plytelės { $tile } datos

## After `init`

note-scale-outside-norm = pastaba: 1:5000 nėra sprinto normoje; freemapper-print dėl to perspės, o simboliai lieka 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } nėra OSM id, kaip way/992423119
error-place-invalid-answer = netinkamas vietos paieškos atsakymas: { $detail }
error-place-not-found = { $id } nerasta. Nominatim išvardija tik pažymėtus OSM objektus, todėl nepažymėto multipoligono išorinio kelio ten nėra. Vietoj jo naudokite multipoligono santykį (--osm relation/<id>) arba išsaugokite kontūrą kaip GeoJSON ir naudokite --geojson
error-place-is-a-point = { $id } yra taškas, o ne plotas; pasirinkite jo teritoriją (kelią arba santykį)
error-place-no-such-part = { $id } turi { $count ->
        [one] { $count } dalį
        [few] { $count } dalis
       *[other] { $count } dalių
    }; dalies { $part } nėra
error-place-several-parts = { $id } yra plotas iš { $count } atskirų dalių ({ $sizes }). Žemėlapio plotas yra vienas poligonas: pasirinkite dalį su --part <n> (1 yra didžiausia) arba išsaugokite norimą kontūrą kaip GeoJSON ir naudokite --geojson
error-layer-not-a-world-file = { $path }: tai ne world failas
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = { $layer } peržiūros nėra; sluoksniai su peržiūra: { $names }
error-layer-no-ndsm-published = šio žemėlapio ploto plytelėms nDSM neskelbiamas, todėl ndsm šablono nėra
error-layer-not-made = { $layer } dar nesukurtas; paleiskite { $step ->
        [fetch] freemapper fetch arba run (MCP: start_pipeline); tik Berlynui
        [export] freemapper export arba run (MCP: start_pipeline)
       *[derive] freemapper derive arba run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD žemėlapis iš to fono negauna
warning-map-preview-failed = žemėlapis įrašytas, bet jo peržiūros nėra: { $error }
warning-training-no-tree-cover = treniruočių žemėlapis parkų atviras vietas palieka sveikas: trūksta medžių dangos (data/tree-cover.tif); dar kartą paleiskite `freemapper derive`
warning-ocad-no-paper-frames = OCAD pradinis projektas dar neturi popieriaus rėmelių; jie yra OOM šablone templates/paper-frame.omap
warning-paper-frames-left-out = iš templates/paper-frame.omap praleisti popieriaus rėmeliai: { $count }; OOM žemėlapiui jie yra per toli nuo žemėlapio; ploto puslapyje patraukite juos arčiau žemėlapio ploto
