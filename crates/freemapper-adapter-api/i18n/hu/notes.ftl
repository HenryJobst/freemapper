# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = figyelmeztetés: { $text }

warning-other-zone = a térképterület a { $zone } UTM-zónában fekszik (EPSG:{ $epsg }); a freemapper ebben a zónában mindent újra letölt
warning-other-state = a térképterület most egy másik tartomány adataihoz tartozik; a freemapper mindent újra letölt
warning-beyond-coverage = a térképterület egy része a támogatott tartományokon kívül esik. Külföldön, a tengeren vagy olyan tartományban van, amelyet a freemapper még nem támogat, például Türingiában. Ott a tartományoknak nincs adatuk, ezért az ortofotó lehet fehér, az ALKIS pedig nem ad épületeket és földhasználatot
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: az 1:{ $scale } méretarány nem tartozik a { $kind } térképek méretarányai közé ({ $allowed }); a freemapper-print elutasítja a térképet
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: a(z) { $key } bejegyzés figyelmen kívül hagyva a { $what ->
        [tree] fa
        [NAS] NAS
        [set] kiegészítő
       *[other] { $german }
    } rétegeknél: { $reason ->
        [name] nem egyszerű fájlnév
       *[unknown] a freemapper nem ismer ilyen réteget
    }

## Disk space

warning-disk-space-unreadable = nem olvasható a szabad lemezterület itt: { $path } ({ $error }), ezért a freemapper nem ellenőrzi

## Tiles

warning-tile-changed = a(z) { $tile } { $product } csempe megváltozott a forrásnál ({ $old } → { $new })
warning-bdom-tile-unavailable = a(z) { $tile } csempe felszínmodellje (bDOM) nem érhető el ({ $error }); ott nincs nDSM-sablon
warning-bdom-unavailable = a felszínmodell (bDOM) nem érhető el ({ $error }); nincs nDSM-sablon ehhez: { $tiles }
progress-ndsm-from-bdom = a(z) { $tile } csempe nDSM-je a felszínmodellből (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terepmodell
        [surface] felszínmodell
       *[orthophoto] ortofotó
    } ({ $name })
progress-model-download = { $model } letöltése
progress-model-read = { $model } beolvasása
warning-model-unavailable = { $model } nem érhető el ({ $error }); nincs nDSM-sablon
warning-model-unavailable-kept = { $model } nem érhető el ({ $error }); a(z) { $fetched } napon letöltött felszínmodell marad
warning-model-tile-unavailable = { $model }, { $tile } csempe nem érhető el ({ $error }); ott nincs nDSM-sablon
warning-archive-passed-over = { $model }: egy archívum nem olvasható ({ $error }); kihagytuk, és a térképterület csempéi a többi archívumból jönnek
warning-archive-directory-kept = { $model }: egy archívum nem olvasható ({ $error }); egy korábbi futásból megőrzött jegyzékét használjuk, ezért nem vesszük észre azt a csempét, amelyet az LVGL azóta lecserélt
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] a térképterületen
        [hamburg] Hamburg területén
        [hesse] Hessen területén
        [north-rhine-westphalia] Észak-Rajna-Vesztfália területén
        [schleswig-holstein] Schleswig-Holstein területén
        [mecklenburg-vorpommern] Mecklenburg-Elő-Pomeránia területén
        [lower-saxony] Alsó-Szászország területén
        [bavaria] Bajorország területén
       *[other] { $state } területén
    } hiányzó { $count ->
        [one] csempe
       *[other] csempe
    }: { $names }; a sablonok ott üresek maradnak
warning-no-surface-stored = nincs eltárolt felszínmodell; az nDSM-sablonhoz futtassa a `freemapper fetch` parancsot online
warning-surface-older-than-orthophoto = a felszínmodell (bDOM { $surface }) régebbi az ortofotónál ({ $photo }); az nDSM-ből hiányozhatnak az új épületek
progress-surface-kept = a freemapper az nDSM-sablonhoz megtartja a(z) { $fetched } napon letöltött felszínmodellt
progress-surface-tiles-kept = a freemapper az nDSM-sablonhoz megtartja a gyorsítótárazott felszínmodell { $count ->
        [one] csempéjét
       *[other] csempéit
    }: { $names }
progress-orthophoto-unchanged = az ortofotó változatlan
progress-orthophoto-unchanged-dated = az ortofotó változatlan ({ $label } { $date })
progress-orthophoto-download = ortofotó letöltése innen: { $provider } { $service }
progress-orthophoto-in-leaf = az ortofotó felvétele { $date } napon készült, lombos fákkal; a koronák eltakarják, ami alattuk van
progress-orthophoto-bare = az ortofotó felvétele { $date } napon készült, lombtalan fákkal
warning-lgln-orthophoto-kept = ortofotó (LGLN WMS): { $reason }; a tárolt ortofotót és repülési dátumát ({ $date }) megtartottuk
warning-lgln-orthophoto-undated = ortofotó (LGLN WMS): nincs repülési dátum, { $reason }
word-stac-no-image = a STAC nem sorol fel képet a térképterület közepén
word-stac-no-answer = a STAC-keresés nem adott használható választ ({ $error })
warning-downloadcenter-no-file = a Downloadcenterben nincs fájl ehhez: { $places }
warning-downloadcenter-no-file-empty = a Downloadcenterben nincs fájl ehhez: { $places }; { $count ->
        [one] csempe
       *[other] csempe
    } üres maradt: { $names }
progress-k-five-unchanged = a K5-sablon változatlan
progress-k-five-download = a K5 térkép (1:5000) letöltése a Geoportal Berlin WMS-ről
warning-k-five-unavailable = a K5-térképsablon nem érhető el ({ $detail }); kihagytuk

## The state border

progress-beyond-border = a térképterület túlnyúlik a tartományhatáron; azon túl a sablonok üresek, ahol a { $provider } ortofotója és magassági modelljei véget érnek. Az ALKIS és az OSM mindkét tartományból jön
progress-beyond-border-no-data = a térképterület túlnyúlik a tartományhatáron; nincs { $provider } adat ehhez: { $tiles }, ezért ott üresek maradnak a sablonok

## OSM

warning-osm-extract-missing-kept = nincs gyorsítótárazott OSM-kivonat; megtartjuk a(z) { $date } napi tárolt OSM-jelölteket
progress-osm-extract-read = { $region } OSM-kivonat beolvasása, kelte: { $date }
progress-osm-candidates = { $count } OSM-jelölt objektum

## Permit notes

progress-permits-fetch = { $source } adatainak letöltése az engedélyezési tudnivalókhoz
progress-permits-osm = { $region } OSM-kivonat beolvasása (kelte: { $date }) az engedélyezési tudnivalókhoz
progress-permits-stored = az engedélyezési tudnivalók tárolt adatainak beolvasása
warning-permits-source-unreachable = { $source } nem olvasható, ezért az engedélyezési tudnivalók kihagyják: { $error }
warning-permits-osm-no-extract = ehhez a térképterülethez nincs gyorsítótárazott OSM-kivonat, ezért az engedélyezési tudnivalók kihagyják az OSM-et; a `freemapper fetch` letölti

## ALKIS

warning-alkis-coverage-unknown = az { $cadastre }-épületek lefedettsége ismeretlen; rögzítéséhez futtassa a `freemapper fetch` parancsot online
progress-alkis-buildings-download = { $cadastre }-épületek letöltése
progress-alkis-buildings = { $count } { $cadastre }-épület
warning-alkis-no-buildings = a térképterületen nincs { $cadastre }-épület
progress-alkis-skipped = { $count } { $cadastre }-elem kihagyva: { $list }
warning-alkis-buildings-changed = az { $cadastre }-épületek megváltoztak a forrásnál ({ $changes })
progress-alkis-no-land-use-stored = nincs eltárolt { $cadastre }-földhasználat; a tiltott területek jelöltjeihez futtassa a `freemapper fetch` parancsot online
progress-alkis-land-use-download = { $cadastre }-földhasználat letöltése
progress-alkis-land-use-parts = { $count } { $cadastre }-földhasználati rész
warning-alkis-land-use-kept = az { $cadastre }-földhasználat nem érhető el ({ $error }); a tárolt földhasználatot megtartjuk
warning-alkis-land-use-removed = az { $cadastre }-földhasználat nem érhető el ({ $error }); a tárolt földhasználat már nem fedi le a térképterületet, ezért a freemapper eltávolította
warning-alkis-land-use-none = az { $cadastre }-földhasználat nem érhető el ({ $error }); nincsenek tiltott terület jelöltek
progress-bdot-filled = { $count } BDOT10k-objektum ({ $what }) hozzáadva ott, ahol az EGiB nem ad semmit ({ $powiat })
warning-bdot-failed = A BDOT10k nem tudta kitölteni az EGiB hiányát ({ $what }): { $error }
warning-bdot-instead = Az EGiB nem érhető el ({ $error }); helyette a BDOT10k szolgál ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = nincsenek eltárolt berlini ALKIS-szerkezetek; lépcsőkhöz, falakhoz, fákhoz és lombkoronákhoz futtassa a `freemapper fetch` parancsot online
progress-berlin-removing = a tárolt berlini rétegek eltávolítása: a térképterület már nincs Berlin közelében
progress-berlin-download = berlini ALKIS-szerkezetek, fakataszter, utcafelmérés és ponttérképrétegek letöltése
progress-berlin-features = { $count } berlini { $file } objektum
warning-berlin-layer-kept = a berlini { $file } nem érhető el ({ $detail }); a tárolt réteget megtartjuk
warning-berlin-layer-skipped = a berlini { $file } nem érhető el ({ $detail }); a réteget kihagytuk
progress-feinkartierung-none-stored = nincs eltárolt hamburgi Feinkartierung; korlátokhoz, falakhoz, lépcsőkhöz és utcabútorokhoz futtassa a `freemapper fetch` parancsot online
progress-feinkartierung-removing = a tárolt hamburgi Feinkartierung eltávolítása: a térképterület már nincs Hamburgban
progress-feinkartierung-download = a hamburgi Feinkartierung letöltése (utcatér és közzöld)
progress-feinkartierung-features = { $count } Feinkartierung { $file } objektum
warning-feinkartierung-layer-kept = a Feinkartierung { $file } nem érhető el ({ $detail }); a tárolt réteget megtartjuk
warning-feinkartierung-layer-skipped = a Feinkartierung { $file } nem érhető el ({ $detail }); a réteget kihagytuk
progress-bdot-roads-none-stored = nincs mentett BDOT10k út és ösvény; a nyilvántartás útjaihoz és ösvényeihez futtasd online a `freemapper fetch` parancsot
progress-bdot-roads-removing = a mentett BDOT10k utak és ösvények eltávolítása: a térképterület már nem Lengyelországban van
progress-bdot-roads-download = a BDOT10k utak és ösvények letöltése
progress-bdot-roads-features = { $count } BDOT10k { $file } objektum
warning-bdot-roads-layer-kept = a BDOT10k { $file } nem érhető el ({ $detail }); a mentett réteg megmarad
warning-bdot-roads-layer-skipped = a BDOT10k { $file } nem érhető el ({ $detail }); a réteg kimarad
error-extra-layer-unknown = nem a(z) { $folder } rétege
warning-berlin-buildings-missing = Berlinben ehhez a területhez nincs eltárolt ALKIS-épület; a `freemapper fetch` letölti őket a Geoportal Berlinről

## The full ALKIS layers (NAS)

progress-nas-none-stored = nincsenek eltárolt teljes ALKIS-szerkezetek; falakhoz, kerítésekhez, sövényekhez és hidakhoz futtassa a `freemapper fetch` parancsot online
progress-nas-removing = a tárolt teljes ALKIS-rétegek eltávolítása: a térképterület már nem olyan tartományban van, amelyben van NAS-szolgáltatás
warning-nas-folder-outside = a teljes ALKIS-rétegeket kihagytuk ({ $folder } kivezet a projektmappából)
progress-nas-download = a teljes ALKIS-szerkezetek (NAS) letöltése
progress-nas-features = { $count } ALKIS NAS { $file } objektum
progress-nas-skipped = { $count } ALKIS NAS { $file } objektum kihagyva: { $list }
warning-nas-layer-kept = az ALKIS NAS { $file } nem érhető el ({ $detail }); a tárolt réteget megtartjuk
warning-nas-layer-skipped = az ALKIS NAS { $file } nem érhető el ({ $detail }); a réteget kihagytuk

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nincsenek eltárolt ATKIS Basis-DLM objektumok; forrásokhoz, barlangokhoz, tornyokhoz, emlékművekhez és szökőkutakhoz futtassa a `freemapper fetch` parancsot online
progress-dlm-removing = az ATKIS Basis-DLM tárolt rétegeinek eltávolítása: a térképterület már nem olyan tartományban van, amely szolgáltatja
warning-dlm-folder-outside = az ATKIS Basis-DLM rétegeit kihagytuk ({ $folder } kivezet a projektmappából)
progress-dlm-download = az ATKIS Basis-DLM objektumainak letöltése
progress-dlm-features = { $count } ATKIS Basis-DLM { $file } objektum
progress-dlm-skipped = { $count } ATKIS Basis-DLM { $file } objektum kihagyva: { $list }
warning-dlm-layer-kept = az ATKIS Basis-DLM { $file } nem érhető el ({ $detail }); a tárolt réteget megtartjuk
warning-dlm-layer-skipped = az ATKIS Basis-DLM { $file } nem érhető el ({ $detail }); a réteget kihagytuk

## Tree cadastres

progress-trees-none-stored = nincs eltárolt városi fakataszter; az egyes fákhoz futtassa a `freemapper fetch` parancsot online
progress-trees-removing = a tárolt fakataszterek eltávolítása: a térképterület már nincs olyan városban, amelynek van
warning-trees-folder-outside = a városi fakatasztereket kihagytuk ({ $folder } kivezet a projektmappából)
progress-trees-download = a városi fakataszterek letöltése
progress-trees-count = { $count } { $id } fa
progress-trees-skipped = { $count } { $id } fa kihagyva: { $list }
warning-trees-kept = a(z) { $id } fakataszter nem érhető el ({ $detail }); a tárolt kataszter marad
warning-trees-skipped = a(z) { $id } fakataszter nem érhető el ({ $detail }); a fákat kihagytuk

## Deriving the templates

progress-writing = { $path } írása
progress-contours-steep = szintvonalak { $interval } m-enként: a terep olyan meredek, hogy a { $default } m-enkénti szintvonalak összefolynának; a projektben megadott { $default } m-es szintvonalköz megtartja őket (`--contour-interval { $default }`)
warning-osm-only-no-terrain = ehhez a régióhoz még nincs terepadat: a térkép csak az OpenStreetMapből készül, szintvonalak, sablonok és épületek nélkül
progress-no-ndsm-recorded = nincs rögzített nDSM; az nDSM-sablonhoz futtassa a `freemapper fetch` parancsot
warning-ndsm-tile-invalid = érvénytelen nDSM-csempe a freemapper.toml fájlban: { $tile }; kihagytuk
warning-ndsm-not-cached = a(z) { $tile } nDSM nincs a gyorsítótárban; futtassa a `freemapper fetch` parancsot
warning-ndsm-unreadable = a(z) { $tile } nDSM nem olvasható ({ $error }); törölje ezt: { $path }, és futtassa a `freemapper fetch` parancsot
warning-surface-unusable = a felszínmodell használhatatlan ({ $error }); futtassa a `freemapper fetch` parancsot

## Downloads and the cache

progress-downloading = { $url } letöltése
progress-retry-wait = a szolgáltatás foglalt; új próbálkozás { $seconds } mp múlva
progress-downloaded = letöltve: { $megabytes } MB
progress-downloaded-of = letöltve: { $megabytes } MB / { $total } MB
progress-tile-index-download = a(z) { $product } csempeindex letöltése
progress-tile-cached = gyorsítótárba mentve: { $tile }
progress-surface-blocks = a(z) { $tile } felszínmodell-csempe { $total } blokkjából { $count } beolvasása ({ $megabytes } MB)
progress-surface-whole = a(z) { $tile } felszínmodell-csempét egészben töltjük le: { $reason }
warning-tile-cached-older = a gyorsítótárazott { $product } csempét használjuk: { $tile }, kelte { $published } (az újabbat, { $newer }, nem töltöttük le)
warning-ndsm-no-cached = nincs gyorsítótárazott nDSM a(z) { $tile } csempéhez
warning-ndsm-tile-unavailable-cached = a(z) { $tile } nDSM-csempe nem érhető el ({ $reason }); a gyorsítótárazott fájlt használjuk
warning-ndsm-tile-unavailable-skipped = a(z) { $tile } nDSM-csempe nem érhető el ({ $reason }); kihagytuk
warning-file-check-failed = nem sikerült ellenőrizni, hogy a(z) { $file } még naprakész-e ({ $reason }); a gyorsítótárazott fájlt használjuk, és ennek a szervernek a többi gyorsítótárazott fájlját is, kérdezés nélkül
warning-ldbv-no-buildings-file = a szerveren nincs épületfájl a(z) { $tile } csempéhez; a térképről ott hiányozhatnak az épületek
progress-ndsm-not-published = a(z) { $tile } csempéhez nincs közzétett nDSM-pontfelhő
progress-ndsm-cached = gyorsítótárazott nDSM: { $tile }
progress-point-cloud-not-published = a(z) { $tile } csempéhez nincs közzétett lézerpontfelhő
progress-point-cloud-cached = gyorsítótárazott pontfelhő: { $tile }
warning-point-cloud-no-cached = a(z) { $tile } csempéhez nincs gyorsítótárazott pontfelhő; ott nincsenek növényzetjelöltek
warning-point-cloud-unavailable-cached = a(z) { $tile } pontfelhő nem érhető el ({ $reason }); a gyorsítótárazott fájlt használjuk
warning-point-cloud-unavailable-skipped = a(z) { $tile } pontfelhő nem érhető el ({ $reason }); ott nincsenek növényzetjelöltek
warning-point-clouds-unavailable = a lézerpontfelhők nem érhetők el ({ $reason }); a növényzetjelöltek a gyorsítótárazott csempéket használják, ha van ilyen
progress-no-point-clouds = itt a növényzethez még nincs lézerpontfelhő: a tartomány nem tesz közzé olyat, amelyet a freemapper el tud olvasni, vagy nem a térkép koordináta-rendszerében
warning-point-clouds-not-fetched = a növényzet ki van választva, de nincs pontfelhő betöltve; a betöltéshez futtassa a `freemapper fetch` parancsot
warning-point-cloud-unreadable = a(z) { $tile } pontfelhő nem olvasható ({ $error }); törölje ezt: { $path }, és futtassa a `freemapper fetch` parancsot
warning-osm-nrw-nearest = a területtől 100 m-en belül nincs észak-rajna-vesztfáliai járási kivonat; a legközelebbit olvassuk: { $district }
warning-osm-disk-full-cached = megtelt a lemez, ezért a freemapper a(z) { $date } napi gyorsítótárazott OSM-kivonatot használja. Szabadítson fel helyet, és futtassa újra a `freemapper fetch` parancsot egy újabbért
warning-osm-geofabrik-unavailable = a Geofabrik nem érhető el ({ $error }); a(z) { $date } napi gyorsítótárazott OSM-kivonatot használjuk
progress-osm-extract-cached = gyorsítótárazott OSM-kivonat: { $region }, kelte { $date }
progress-osm-extract-download = { $url } letöltése (kb. { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = a(z) { $tile } bDOM-csempe letöltése ({ $megabytes } MB, { $index } / { $total })
progress-sh-dgm-not-in-atom = a(z) { $tile } DGM1-csempe nincs az INSPIRE ATOM jegyzékben; az XYZ-fájlját töltjük le ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = a(z) { $tile } DGM1-csempe az INSPIRE ATOM jegyzékből nem olvasható ({ $error }); az XYZ-fájlját töltjük le ({ $megabytes } MB)
progress-sh-tile = { $tile } csempe: { $text }
progress-sh-download-again = { $tile } csempe: a letöltés nem olvasható ({ $message }); még egyszer letöltjük
warning-sh-portal-no-date = a Downloadportal nem ad dátumot a(z) { $tile } { $model } csempéhez
warning-sh-dgm-xyz-used = a(z) { $tile } DGM1-csempe nincs az INSPIRE ATOM jegyzékben; a Downloadportal XYZ-fájlját használtuk
warning-sh-dgm-xyz-used-unreadable = a(z) { $tile } DGM1-csempe az INSPIRE ATOM jegyzékből nem olvasható ({ $error }); a Downloadportal XYZ-fájlját használtuk

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, { $tile } csempe: a térképterület { $count } cellájához nincs magasság
warning-centre-tile-missing = a térképterület közepén lévő { $tile } csempe hiányzik a(z) { $model } modellből, ezért nincs dátum
warning-centre-tile-unlisted = a csempelista nem említi a térképterület közepén lévő { $tile } csempét a(z) { $model } modellből, ezért nincs dátum
warning-list-unreadable-kept = a(z) { $model } csempelistája nem olvasható ({ $error }); egy korábbi futásból megőrzöttet használtuk
warning-portal-list-unreadable-kept = a Downloadportal csempelistája a(z) { $model } modellhez nem olvasható ({ $error }); egy korábbi futásból megőrzöttet használtuk
warning-surface-much-older = a(z) { $model } kelte { $surface }, az ortofotóé { $flight }. Az nDSM-ből hiányozhatnak az újabb épületek és fák.

## Rhineland-Palatinate

warning-rp-list-features-unusable = a csempelista { $count } objektumának nincs használható csempekulcsa; kihagytuk
warning-rp-list-unreadable-kept = az LVermGeo RP csempelistája nem olvasható ({ $error }); egy korábbi futásból megőrzöttet használtuk
warning-rp-metalink-not-kept = az LVermGeo RP Metalinkjét nem őriztük meg: { $error }
warning-rp-metalink-unreadable-kept = az LVermGeo RP Metalinkje a(z) { $model } modellhez nem olvasható ({ $detail }); egy korábbi futásból megőrzöttet használtuk
warning-rp-metalink-unreadable = az LVermGeo RP Metalinkje a(z) { $model } modellhez nem olvasható ({ $detail }), ezért azok a csempék, amelyekhez a csempelistája nem ad repülési évet, magasságok nélkül maradnak
warning-rp-tile-without-name = { $model }, { $tile } csempe: ennek az 1 km-es négyzetnek nincsenek magasságai a térképen. Az LVermGeo RP csempelistája (Laserbefliegung) { $note }, ezért a fájlnév nem ismert{ $either ->
        [metalink] , és a Metalinkje sem nevez meg hozzá fájlt
       *[none] {""}
    }
warning-rp-centre-no-date = a(z) { $tile } { $model } csempének nincs dátuma: az LVermGeo RP csempelistája (Laserbefliegung) { $note }
word-rp-list-neither = sem repülési évet, sem repülési napokat nem ad meg ehhez
word-rp-list-no-days = repülési napokat nem ad meg ehhez
word-rp-list-no-year = repülési évet nem ad meg ehhez
word-rp-list-days-unreadable = olvashatatlan repülési napokat ad meg ehhez: { $days }
word-rp-list-disagree = repülési napokat ({ $days }) és repülési évet ({ $year }) ad meg ehhez, amelyek nem egyeznek egy évben

## Hesse

warning-hvbg-undated = a metaadatlistában nincs olvasható Bildflugdatum a(z) { $tiles } csempéhez; a projektrekordjaiknak nincs repülési dátuma
warning-hvbg-undated-more = a metaadatlistában nincs olvasható Bildflugdatum a(z) { $tiles } csempéhez és { $more } további csempéhez; a projektrekordjaiknak nincs repülési dátuma
warning-hvbg-list-unreadable-older = az új HVBG-metaadatlista ({ $name }) nem olvasható ({ $detail }); a régebbi listát használjuk: { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: nincs dátum, a STAC nem sorol fel csempét a térképterület közepén
warning-lgln-no-date = { $model }: nincs dátum ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; a tárolt modell megtartja a rögzített dátumát
warning-mv-cells-empty = { $model }: a mecklenburg-elő-pomerániai csempéken belül { $count } cellához nincs magasság
warning-mv-centre-no-date = { $model }: nincs dátum a középső csempéhez ({ $tile })

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = a(z) { $model } csempelistájának { $count } sorában nincs használható csempenév; kihagytuk
warning-nrw-list-rows-undated = a(z) { $model } csempelistájának { $count } sorában nincs olvasható dátum
warning-nrw-list-older = a szerver csempelistája a(z) { $model } modellhez { $server } keltű, régebbi, mint a megőrzött ({ $kept }); a megőrzöttet használtuk
warning-nrw-list-no-date = a csempelista nem ad dátumot a(z) { $tile } { $model } csempéhez

## After `init`

note-scale-outside-norm = megjegyzés: az 1:5000 nem szerepel a sprintnormában; a freemapper-print figyelmeztet erre, a jelkulcsok pedig 100 %-on maradnak

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } nem OSM-azonosító, mint például way/992423119
error-place-invalid-answer = érvénytelen válasz a helykeresőtől: { $detail }
error-place-not-found = { $id } nem található. A Nominatim csak címkézett OSM-objektumokat listáz, ezért egy multipoligon címke nélküli külső vonala nincs köztük. Használja inkább a multipoligon relációt (--osm relation/<id>), vagy mentse el a körvonalat GeoJSON-ként, és használja a --geojson kapcsolót
error-place-is-a-point = { $id } egy pont, nem terület; válassza a telkét (egy vonalat vagy relációt)
error-place-no-such-part = { $id } { $count ->
        [one] 1 részből áll
       *[other] { $count } részből áll
    }; nincs { $part }. rész
error-place-several-parts = { $id } { $count } különálló részből álló terület ({ $sizes }). A térképterület egyetlen sokszög: válasszon részt a --part <n> kapcsolóval (az 1. a legnagyobb), vagy mentse el a kívánt körvonalat GeoJSON-ként, és használja a --geojson kapcsolót
error-layer-not-a-world-file = { $path }: nem világfájl
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = nincs előnézet ehhez: { $layer }; előnézettel rendelkező rétegek: { $names }
error-layer-no-ndsm-published = ennek a térképterületnek a csempéihez nincs közzétett nDSM, ezért nincs ndsm-sablon
error-layer-not-made = { $layer } még nem készült el; futtassa ezt: { $step ->
        [fetch] freemapper fetch vagy run (MCP: start_pipeline); csak Berlin
        [export] freemapper export vagy run (MCP: start_pipeline)
       *[derive] freemapper derive vagy run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; az OCAD-térkép ebből nem kap háttért
warning-map-preview-failed = a térkép elkészült, de az előnézete hiányzik: { $error }
warning-training-no-tree-cover = az edzéstérkép egyben tartja a parkok nyílt területeit: hiányzik a fedettség (data/tree-cover.tif); futtassa újra a `freemapper derive` parancsot
