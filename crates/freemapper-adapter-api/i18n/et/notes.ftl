# Käivitusmootori edenemisread ja hoiatused (ADR 0092, 3. etapp). Viitekeel on inglise keel.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = hoiatus: { $text }

warning-other-zone = kaardiala asub UTM-tsoonis { $zone } (EPSG:{ $epsg }); freemapper laadib kõik selles tsoonis uuesti alla
warning-other-state = kaardiala asub nüüd teise liidumaa andmetes; freemapper laadib kõik uuesti alla
warning-beyond-coverage = osa kaardialast jääb toetatud liidumaadest välja. See asub välismaal, merel või liidumaal, mida freemapper veel ei toeta, näiteks Tüüringis. Liidumaadel pole seal andmeid, seetõttu võib ortofoto olla valge ning ALKIS ei anna hooneid ega maakasutust
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: mõõtkava 1:{ $scale } ei kuulu { $kind }kaartide mõõtkavade hulka ({ $allowed }); freemapper-print keeldub kaardist
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: { $german ->
       *[any] { $what }
    } kihi kirje { $key } jäeti tähelepanuta: { $reason ->
        [name] see pole tavaline failinimi
       *[unknown] freemapper ei tunne sellist kihti
    }

## Disk space

warning-disk-space-unreadable = asukoha { $path } vaba kettaruumi ei saa lugeda ({ $error }), seetõttu freemapper seda ei kontrolli

## Tiles

warning-tile-changed = { $product } ruut { $tile } muutus allikas ({ $old } → { $new })
warning-bdom-tile-unavailable = ruudu { $tile } pinnamudel (bDOM) pole saadaval ({ $error }); seal puudub nDSM-i mall
warning-bdom-unavailable = pinnamudel (bDOM) pole saadaval ({ $error }); { $tiles } jaoks puudub nDSM-i mall
progress-ndsm-from-bdom = ruudu { $tile } nDSM pinnamudelist (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] maastikumudel
        [surface] pinnamudel
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = allalaadimine: { $model }
progress-model-read = lugemine: { $model }
warning-model-unavailable = { $model } pole saadaval ({ $error }); nDSM-i mall puudub
warning-model-unavailable-kept = { $model } pole saadaval ({ $error }); kasutatakse kuupäeval { $fetched } alla laaditud pinnamudelit
warning-model-tile-unavailable = { $model } ruut { $tile } pole saadaval ({ $error }); seal puudub nDSM-i mall
warning-archive-passed-over = { $model }: arhiivi ei saanud lugeda ({ $error }); see jäeti vahele ja kaardiala ruudud pärinevad teistest arhiividest
warning-archive-directory-kept = { $model }: arhiivi ei saanud lugeda ({ $error }); kasutatakse varasemast käivitusest säilitatud kataloogi, seega ei märgata ruutu, mille LVGL on vahepeal asendanud
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] kaardialal
        [hamburg] Hamburgis
        [hesse] Hessenis
        [north-rhine-westphalia] Nordrhein-Westfalenis
        [schleswig-holstein] Schleswig-Holsteinis
        [mecklenburg-vorpommern] Mecklenburg-Vorpommernis
        [lower-saxony] Alam-Saksis
        [bavaria] Baieris
       *[other] { $state } liidumaal
    } { $count ->
        [one] puudub ruut
       *[other] puuduvad ruudud
    } { $names }; mallid jäävad seal tühjaks
warning-no-surface-stored = pinnamudelit pole salvestatud; nDSM-i malli jaoks tuleb käivitada `freemapper fetch` võrguühendusega
warning-surface-older-than-orthophoto = pinnamudel (bDOM { $surface }) on ortofotost ({ $photo }) vanem; nDSM-ist võivad uued hooned puududa
progress-surface-kept = freemapper kasutab nDSM-i malli jaoks kuupäeval { $fetched } alla laaditud pinnamudelit
progress-surface-tiles-kept = freemapper kasutab nDSM-i malli jaoks { $count ->
        [one] ruudu
       *[other] ruutude
    } { $names } vahemällu salvestatud pinnamudelit
progress-orthophoto-unchanged = ortofoto on muutumatu
progress-orthophoto-unchanged-dated = ortofoto on muutumatu ({ $label } { $date })
progress-orthophoto-download = ortofoto allalaadimine teenusest { $provider } { $service }
progress-orthophoto-in-leaf = ortofoto on lennatud { $date }, puud olid lehes; nende võrad varjavad, mis jääb nende alla
progress-orthophoto-bare = ortofoto on lennatud { $date }, puud olid lehtedeta
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; salvestatud ortofoto ja selle lennukuupäev { $date } jäid alles
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): lennukuupäev puudub, { $reason }
word-stac-no-image = STAC ei loetle kaardiala keskel ühtegi pilti
word-stac-no-answer = STAC-otsing ei andnud kasutatavat vastust ({ $error })
warning-downloadcenter-no-file = Downloadcenteris pole faili kohale { $places }
warning-downloadcenter-no-file-empty = Downloadcenteris pole faili kohale { $places }; { $count ->
        [one] ruut
       *[other] ruudud
    } { $names } jäid tühjaks
progress-k-five-unchanged = K5 mall on muutumatu
progress-k-five-download = K5 kaardi (1:5000) allalaadimine Geoportal Berlini WMS-ist
warning-k-five-unavailable = K5 kaardi mall pole saadaval ({ $detail }); jäeti vahele

## The state border

progress-beyond-border = kaardiala ulatub üle liidumaa piiri; selle taga on mallid tühjad seal, kus { $provider } ortofoto ja kõrgusmudelid lõpevad. ALKIS ja OSM pärinevad mõlemast liidumaast
progress-beyond-border-no-data = kaardiala ulatub üle liidumaa piiri; { $tiles } jaoks pole { $provider } andmeid, seetõttu jäävad mallid seal tühjaks

## OSM

warning-osm-extract-missing-kept = OSM-i väljavõtet pole vahemälus; kasutatakse salvestatud OSM-i kandidaate kuupäevast { $date }
progress-osm-extract-read = OSM-i väljavõtte { $region } lugemine (kuupäev { $date })
progress-osm-candidates = { $count } OSM-i kandidaatobjekti

## Permit notes

progress-permits-fetch = loamärkmete jaoks laaditakse alla { $source } andmed
progress-permits-osm = OSM-i väljavõtte { $region } lugemine (kuupäev { $date }) loamärkmete jaoks
progress-permits-stored = loamärkmete salvestatud andmete lugemine
warning-permits-source-unreachable = { $source } ei saanud lugeda, seetõttu loamärkmed jätavad selle välja: { $error }
warning-permits-osm-no-extract = selle kaardiala OSM-i väljavõtet pole vahemälus, seetõttu loamärkmed jätavad OSM-i välja; `freemapper fetch` laadib selle alla

## ALKIS

warning-alkis-coverage-unknown = ALKIS-i hoonete katvus on teadmata; selle salvestamiseks tuleb käivitada `freemapper fetch` võrguühendusega
progress-alkis-buildings-download = ALKIS-i hoonete allalaadimine
progress-alkis-buildings = { $count } ALKIS-i hoonet
warning-alkis-no-buildings = kaardialal ei leitud ALKIS-i hooneid
progress-alkis-skipped = jäeti vahele { $count } ALKIS-i objekti: { $list }
warning-alkis-buildings-changed = ALKIS-i hooned muutusid allikas ({ $changes })
progress-alkis-no-land-use-stored = ALKIS-i maakasutust pole salvestatud; keelualade kandidaatide jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-alkis-land-use-download = ALKIS-i maakasutuse allalaadimine
progress-alkis-land-use-parts = { $count } ALKIS-i maakasutuse osa
warning-alkis-land-use-kept = ALKIS-i maakasutus pole saadaval ({ $error }); salvestatud maakasutus jääb alles
warning-alkis-land-use-removed = ALKIS-i maakasutus pole saadaval ({ $error }); salvestatud maakasutus ei kata enam kaardiala, seetõttu freemapper eemaldas selle
warning-alkis-land-use-none = ALKIS-i maakasutus pole saadaval ({ $error }); keelualade kandidaate pole

## The Berlin layers

progress-berlin-none-stored = Berliini ALKIS-i rajatisi pole salvestatud; treppide, müüride, puude ja katuste jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-berlin-removing = salvestatud Berliini kihtide eemaldamine: kaardiala ei asu enam Berliini lähedal
progress-berlin-download = Berliini ALKIS-i rajatiste, puukatastri, tänavamõõdistuse ja punktkihtide allalaadimine
progress-berlin-features = { $count } Berliini { $file } objekti
warning-berlin-layer-kept = Berliini { $file } pole saadaval ({ $detail }); salvestatud kiht jääb alles
warning-berlin-layer-skipped = Berliini { $file } pole saadaval ({ $detail }); kiht jäeti vahele
progress-feinkartierung-none-stored = Hamburgi Feinkartierungit pole salvestatud; piirete, müüride, treppide ja tänavamööbli jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-feinkartierung-removing = salvestatud Hamburgi Feinkartierungi eemaldamine: kaardiala ei asu enam Hamburgis
progress-feinkartierung-download = Hamburgi Feinkartierungi (tänavaruum ja avalik haljastus) allalaadimine
progress-feinkartierung-features = { $count } Feinkartierungi { $file } objekti
warning-feinkartierung-layer-kept = Feinkartierung { $file } pole saadaval ({ $detail }); salvestatud kiht jääb alles
warning-feinkartierung-layer-skipped = Feinkartierung { $file } pole saadaval ({ $detail }); kiht jäeti vahele
error-extra-layer-unknown = ei ole kaustas { $folder } olev kiht
warning-berlin-buildings-missing = selle ala ALKIS-i hooneid Berliinis pole salvestatud; Geoportal Berlinist saamiseks tuleb käivitada `freemapper fetch`

## The full ALKIS layers (NAS)

progress-nas-none-stored = täielikke ALKIS-i rajatisi pole salvestatud; müüride, piirete, hekkide ja sildade jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-nas-removing = salvestatud täielike ALKIS-i kihtide eemaldamine: kaardiala ei asu enam liidumaal, kus on NAS-teenus
warning-nas-folder-outside = täielikud ALKIS-i kihid jäeti vahele ({ $folder } viib projektikaustast välja)
progress-nas-download = täielike ALKIS-i rajatiste (NAS) allalaadimine
progress-nas-features = { $count } ALKIS NAS-i { $file } objekti
progress-nas-skipped = jäeti vahele { $count } ALKIS NAS-i { $file } objekti: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } pole saadaval ({ $detail }); salvestatud kiht jääb alles
warning-nas-layer-skipped = ALKIS NAS { $file } pole saadaval ({ $detail }); kiht jäeti vahele

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = ATKIS Basis-DLM-i objekte pole salvestatud; allikate, koobaste, tornide, mälestusmärkide ja purskkaevude jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-dlm-removing = salvestatud ATKIS Basis-DLM-i kihtide eemaldamine: kaardiala ei asu enam liidumaal, mis seda pakub
warning-dlm-folder-outside = ATKIS Basis-DLM-i kihid jäeti vahele ({ $folder } viib projektikaustast välja)
progress-dlm-download = ATKIS Basis-DLM-i objektide allalaadimine
progress-dlm-features = { $count } ATKIS Basis-DLM-i { $file } objekti
progress-dlm-skipped = jäeti vahele { $count } ATKIS Basis-DLM-i { $file } objekti: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } pole saadaval ({ $detail }); salvestatud kiht jääb alles
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } pole saadaval ({ $detail }); kiht jäeti vahele

## Tree cadastres

progress-trees-none-stored = linna puukatastrit pole salvestatud; üksikpuude jaoks tuleb käivitada `freemapper fetch` võrguühendusega
progress-trees-removing = salvestatud puukatastrite eemaldamine: kaardiala ei asu enam linnas, kus see on
warning-trees-folder-outside = linnade puukatastrid jäeti vahele ({ $folder } viib projektikaustast välja)
progress-trees-download = linnade puukatastrite allalaadimine
progress-trees-count = { $count } puud ({ $id })
progress-trees-skipped = jäeti vahele { $count } puud ({ $id }): { $list }
warning-trees-kept = puukataster { $id } pole saadaval ({ $detail }); salvestatud kataster jääb alles
warning-trees-skipped = puukataster { $id } pole saadaval ({ $detail }); puud jäeti vahele

## Deriving the templates

progress-writing = kirjutatakse { $path }
progress-contours-steep = kõrgusjooned { $interval } m järel: maastik on nii järsk, et { $default } m järel kõrgusjooned sulaksid kokku; projektis seatud kõrgusjoonte vahe { $default } m säilitab need (`--contour-interval { $default }`)
warning-osm-only-no-terrain = selle piirkonna kohta pole veel maastikuandmeid: kaart tehakse ainult OpenStreetMapist, ilma kõrgusjoonte, mallide ja hoonete
progress-no-ndsm-recorded = nDSM-i pole salvestatud; nDSM-i malli jaoks tuleb käivitada `freemapper fetch`
warning-ndsm-tile-invalid = freemapper.toml sisaldab kehtetut nDSM-i ruutu { $tile }; jäeti vahele
warning-ndsm-not-cached = nDSM { $tile } pole vahemälus; tuleb käivitada `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } ei saa lugeda ({ $error }); tuleb kustutada { $path } ja käivitada `freemapper fetch`
warning-surface-unusable = pinnamudel on kasutuskõlbmatu ({ $error }); tuleb käivitada `freemapper fetch`

## Downloads and the cache

progress-downloading = allalaadimine: { $url }
progress-downloaded = alla laaditud { $megabytes } MB
progress-downloaded-of = alla laaditud { $megabytes } MB / { $total } MB
progress-tile-index-download = { $product } ruuduindeksi allalaadimine
progress-tile-cached = vahemällu salvestatud { $tile }
progress-surface-blocks = pinnamudeli ruudu { $tile } { $total } plokist loetakse { $count } ({ $megabytes } MB)
progress-surface-whole = pinnamudeli ruut { $tile } laaditakse alla tervikuna: { $reason }
warning-tile-cached-older = kasutatakse vahemälus olevat { $product } ruutu { $tile } kuupäevast { $published } (uuemat { $newer } ei laaditud alla)
warning-ndsm-no-cached = ruudu { $tile } nDSM-i pole vahemälus
warning-ndsm-tile-unavailable-cached = nDSM-i ruut { $tile } pole saadaval ({ $reason }); kasutatakse vahemälus olevat faili
warning-ndsm-tile-unavailable-skipped = nDSM-i ruut { $tile } pole saadaval ({ $reason }); jäeti vahele
warning-file-check-failed = ei õnnestunud kontrollida, kas { $file } on veel ajakohane ({ $reason }); kasutatakse vahemälus olevat faili ja selle serveri teisi vahemälus olevaid faile küsimata
warning-ldbv-no-buildings-file = ruudu { $tile } hoonefaili pole serveris; kaardilt võivad seal hooned puududa
progress-ndsm-not-published = ruudu { $tile } nDSM-i punktipilve pole avaldatud
progress-ndsm-cached = vahemällu salvestatud nDSM { $tile }
progress-point-cloud-not-published = ruudu { $tile } laserpunktipilve pole avaldatud
progress-point-cloud-cached = vahemällu salvestatud punktipilv { $tile }
warning-point-cloud-no-cached = ruudu { $tile } punktipilve pole vahemälus; seal pole taimestiku kandidaate
warning-point-cloud-unavailable-cached = punktipilv { $tile } pole saadaval ({ $reason }); kasutatakse vahemälus olevat faili
warning-point-cloud-unavailable-skipped = punktipilv { $tile } pole saadaval ({ $reason }); seal pole taimestiku kandidaate
warning-point-clouds-unavailable = laserpunktipilved pole saadaval ({ $reason }); taimestiku kandidaadid kasutavad vahemälus olevaid ruute, kui neid on
progress-no-point-clouds = taimestiku jaoks pole siin veel laserpunktipilvi: liidumaa ei avalda ühtki, mida freemapper suudab lugeda, või need pole kaardi koordinaatsüsteemis
warning-point-clouds-not-fetched = taimestik on valitud, kuid punktipilvi pole laaditud; nende laadimiseks tuleb käivitada `freemapper fetch`
warning-point-cloud-unreadable = punktipilve { $tile } ei saa lugeda ({ $error }); tuleb kustutada { $path } ja käivitada `freemapper fetch`
warning-osm-nrw-nearest = 100 m raadiuses pole Nordrhein-Westfaleni ringkonna väljavõtet; loetakse lähimat: { $district }
warning-osm-disk-full-cached = ketas on täis, seetõttu kasutab freemapper vahemälus olevat OSM-i väljavõtet kuupäevast { $date }. Uuema saamiseks tuleb ruumi vabastada ja käivitada `freemapper fetch` uuesti
warning-osm-geofabrik-unavailable = Geofabrik pole saadaval ({ $error }); kasutatakse vahemälus olevat OSM-i väljavõtet kuupäevast { $date }
progress-osm-extract-cached = vahemällu salvestatud OSM-i väljavõte { $region } kuupäevast { $date }
progress-osm-extract-download = allalaadimine: { $url } (umbes { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = bDOM-i ruudu { $tile } allalaadimine ({ $megabytes } MB, { $index } / { $total })
progress-sh-dgm-not-in-atom = DGM1 ruutu { $tile } pole INSPIRE ATOM-kataloogis; selle XYZ laaditakse alla ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = INSPIRE ATOM-kataloogi DGM1 ruutu { $tile } ei saa lugeda ({ $error }); selle XYZ laaditakse alla ({ $megabytes } MB)
progress-sh-tile = ruut { $tile }: { $text }
progress-sh-download-again = ruut { $tile }: allalaaditut ei saa lugeda ({ $message }); laaditakse uuesti alla
warning-sh-portal-no-date = Downloadportal ei anna kuupäeva { $model } ruudule { $tile }
warning-sh-dgm-xyz-used = DGM1 ruutu { $tile } pole INSPIRE ATOM-kataloogis; kasutati selle XYZ-i Downloadportalist
warning-sh-dgm-xyz-used-unreadable = INSPIRE ATOM-kataloogi DGM1 ruutu { $tile } ei saa lugeda ({ $error }); kasutati selle XYZ-i Downloadportalist

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model } ruut { $tile }: kaardiala { $count } lahtril puudub kõrgus
warning-centre-tile-missing = { $model } ruut { $tile } kaardiala keskel puudub, seetõttu pole kuupäeva
warning-centre-tile-unlisted = ruutude loend ei nimeta kaardiala keskel { $model } ruutu { $tile }, seetõttu pole kuupäeva
warning-list-unreadable-kept = { $model } ruutude loendit ei saanud lugeda ({ $error }); kasutati varasemast käivitusest säilitatud loendit
warning-portal-list-unreadable-kept = Downloadportali { $model } ruutude loendit ei saanud lugeda ({ $error }); kasutati varasemast käivitusest säilitatud loendit
warning-surface-much-older = { $model } pärineb ajast { $surface }, ortofoto ajast { $flight }. nDSM-ist võivad uuemad hooned ja puud puududa.

## Rhineland-Palatinate

warning-rp-list-features-unusable = ruutude loendi { $count } objektil pole kasutatavat ruuduvõtit; jäeti vahele
warning-rp-list-unreadable-kept = LVermGeo RP ruutude loendit ei saanud lugeda ({ $error }); kasutati varasemast käivitusest säilitatud loendit
warning-rp-metalink-not-kept = LVermGeo RP Metalinki ei säilitatud: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP { $model } Metalinki ei saanud lugeda ({ $detail }); kasutati varasemast käivitusest säilitatud Metalinki
warning-rp-metalink-unreadable = LVermGeo RP { $model } Metalinki ei saanud lugeda ({ $detail }), seetõttu jäävad ruudud, mille lennuaastat nende ruutude loend ei anna, ilma kõrgusteta
warning-rp-tile-without-name = { $model } ruut { $tile }: sellel 1 km ruudul pole kaardil kõrgusi. LVermGeo RP ruutude loend (Laserbefliegung) { $note } selle kohta, seetõttu pole failinime teada{ $either ->
        [metalink] , ja selle Metalink ei nimeta selle kohta samuti ühtegi faili
       *[none] {""}
    }
warning-rp-centre-no-date = { $model } ruudul { $tile } puudub kuupäev: LVermGeo RP ruutude loend (Laserbefliegung) { $note } selle kohta
word-rp-list-neither = ei anna ei lennuaastat ega lennupäevi
word-rp-list-no-days = ei anna lennupäevi
word-rp-list-no-year = ei anna lennuaastat
word-rp-list-days-unreadable = annab lennupäevad { $days }, mida ei saa lugeda
word-rp-list-disagree = annab lennupäevad { $days } ja lennuaasta { $year }, mis ei lange ühte aastasse

## Hesse

warning-hvbg-undated = metaandmete loendis pole ruudu { $tiles } loetavat Bildflugdatumit; nende projektikirjetel pole lennukuupäeva
warning-hvbg-undated-more = metaandmete loendis pole ruudu { $tiles } ja veel { $more } ruudu loetavat Bildflugdatumit; nende projektikirjetel pole lennukuupäeva
warning-hvbg-list-unreadable-older = uut HVBG metaandmete loendit { $name } ei saanud lugeda ({ $detail }); kasutatakse vanemat loendit { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: kuupäev puudub, STAC ei loetle kaardiala keskel ühtegi ruutu
warning-lgln-no-date = { $model }: kuupäev puudub ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; salvestatud mudel säilitab oma kirjendatud kuupäeva
warning-mv-cells-empty = { $model }: Mecklenburg-Vorpommerni ruutude sees on { $count } lahtrit ilma kõrguseta
warning-mv-centre-no-date = { $model }: keskruudul { $tile } puudub kuupäev

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $model } ruutude loendi { $count } real pole kasutatavat ruudunime; jäeti vahele
warning-nrw-list-rows-undated = { $model } ruutude loendi { $count } real pole loetavat kuupäeva
warning-nrw-list-older = serveri { $model } ruutude loend on kuupäevast { $server }, vanem kui säilitatud loend kuupäevast { $kept }; kasutati säilitatud loendit
warning-nrw-list-no-date = ruutude loend ei anna kuupäeva { $model } ruudule { $tile }

## After `init`

note-scale-outside-norm = märkus: 1:5000 ei kuulu sprindinormi; freemapper-print hoiatab selle eest ja sümbolid jäävad 100 %-ni

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } ei ole OSM-i id, nagu way/992423119
error-place-invalid-answer = kehtetu vastus kohaotsingust: { $detail }
error-place-not-found = { $id } ei leitud. Nominatim loetleb ainult sildistatud OSM-i objekte, seetõttu märgendita multipolügooni välimist joont seal pole. Tuleb kasutada multipolügooni relatsiooni (--osm relation/<id>) või salvestada kontuur GeoJSON-ina ja kasutada --geojson
error-place-is-a-point = { $id } on punkt, mitte ala; tuleb valida selle maa-ala (joon või relatsioon)
error-place-no-such-part = { $id } koosneb { $count ->
        [one] 1 osast
       *[other] { $count } osast
    }; osa { $part } pole olemas
error-place-several-parts = { $id } on ala, mis koosneb { $count } eraldi osast ({ $sizes }). Kaardiala on üks polügoon: osa saab valida lülitiga --part <n> (1 on suurim) või salvestada soovitud kontuuri GeoJSON-ina ja kasutada --geojson
error-layer-not-a-world-file = { $path }: ei ole world-fail
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = kihil { $layer } pole eelvaadet; eelvaatega kihid: { $names }
error-layer-no-ndsm-published = selle kaardiala ruutude nDSM-i pole avaldatud, seetõttu ndsm-i malli pole
error-layer-not-made = { $layer } pole veel tehtud; tuleb käivitada { $step ->
        [fetch] freemapper fetch või run (MCP: start_pipeline); ainult Berliin
        [export] freemapper export või run (MCP: start_pipeline)
       *[derive] freemapper derive või run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD-i kaart ei saa sellest taustakaarti
warning-map-preview-failed = kaart on kirjutatud, kuid selle eelvaade puudub: { $error }
warning-training-no-tree-cover = treeningkaart jätab parkide lage maa tervikuks: puude katvus (data/tree-cover.tif) puudub; tuleb käivitada `freemapper derive` uuesti
