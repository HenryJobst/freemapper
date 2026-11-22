# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = varning: { $text }

warning-other-zone = kartområdet ligger i UTM-zon { $zone } (EPSG:{ $epsg }); freemapper hämtar allt på nytt i den zonen
warning-other-state = kartområdet ligger nu i en annan delstats data; freemapper hämtar allt på nytt
warning-beyond-coverage = en del av kartområdet ligger utanför de delstater som stöds. Den ligger utomlands, till havs eller i en delstat som freemapper ännu inte stöder, till exempel Thüringen. Delstaterna har inga data där, så ortofotot kan vara vitt och ALKIS ger varken byggnader eller markanvändning
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: skala 1:{ $scale } är ingen av skalorna för { $kind ->
        [schoolyard] skolgårdskartor
        [forest] skogskartor
       *[sprint] sprintkartor
    } ({ $allowed }); freemapper-print kommer att vägra kartan
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorerade posten { $key } bland { $what ->
        [tree] trädlagren
        [NAS] NAS-lagren
        [set] extralagren
       *[other] { $german }-lagren
    }: { $reason ->
        [name] inget vanligt filnamn
       *[unknown] inget lager som freemapper känner till
    }

## Disk space

warning-disk-space-unreadable = det går inte att läsa det lediga diskutrymmet för { $path } ({ $error }), så freemapper kontrollerar det inte

## Tiles

warning-tile-changed = { $product }-rutan { $tile } har ändrats hos källan ({ $old } → { $new })
warning-bdom-tile-unavailable = ytmodellen (bDOM) för rutan { $tile } är inte tillgänglig ({ $error }); ingen nDSM-mall där
warning-bdom-unavailable = ytmodellen (bDOM) är inte tillgänglig ({ $error }); ingen nDSM-mall för { $tiles }
progress-ndsm-from-bdom = nDSM för rutan { $tile } från ytmodellen (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terrängmodell
        [surface] ytmodell
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = laddar ner { $model }
progress-model-read = läser { $model }
warning-model-unavailable = { $model } är inte tillgänglig ({ $error }); ingen nDSM-mall
warning-model-unavailable-kept = { $model } är inte tillgänglig ({ $error }); behåller ytmodellen som hämtades { $fetched }
warning-model-tile-unavailable = { $model } för rutan { $tile } är inte tillgänglig ({ $error }); ingen nDSM-mall där
warning-archive-passed-over = { $model }: ett arkiv gick inte att läsa ({ $error }); det hoppades över, och rutorna i kartområdet kommer från de andra arkiven
warning-archive-directory-kept = { $model }: ett arkiv gick inte att läsa ({ $error }); dess katalog från en tidigare körning används, så en ruta som LVGL har ersatt sedan dess upptäcks inte
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] I kartområdet
        [hamburg] I Hamburg
        [hesse] I Hessen
        [north-rhine-westphalia] I Nordrhein-Westfalen
        [schleswig-holstein] I Schleswig-Holstein
        [mecklenburg-vorpommern] I Mecklenburg-Vorpommern
        [lower-saxony] I Niedersachsen
        [bavaria] I Bayern
       *[other] I { $state }
    } { $count ->
        [one] saknas rutan
       *[other] saknas rutorna
    } { $names }; mallarna förblir tomma där
warning-no-surface-stored = ingen ytmodell sparad; kör `freemapper fetch` online för nDSM-mallen
warning-surface-older-than-orthophoto = ytmodellen (bDOM { $surface }) är äldre än ortofotot ({ $photo }); nya byggnader kan saknas i nDSM
progress-surface-kept = freemapper behåller ytmodellen som hämtades { $fetched } för nDSM-mallen
progress-surface-tiles-kept = freemapper behåller den cachade ytmodellen för { $count ->
        [one] rutan
       *[other] rutorna
    } { $names } för nDSM-mallen
progress-orthophoto-unchanged = ortofotot är oförändrat
progress-orthophoto-unchanged-dated = ortofotot är oförändrat ({ $label } { $date })
progress-orthophoto-download = laddar ner ortofotot från { $provider } { $service }
progress-orthophoto-in-leaf = ortofotot flögs den { $date }, med lövade träd; trädkronorna döljer det som ligger under dem
progress-orthophoto-bare = ortofotot flögs den { $date }, med kala träd
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; behöll det sparade ortofotot och dess flygdatum { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): inget flygdatum, { $reason }
word-stac-no-image = STAC listar ingen bild i kartområdets mitt
word-stac-no-answer = STAC-sökningen gav inget användbart svar ({ $error })
warning-downloadcenter-no-file = Downloadcenter har ingen fil för { $places }
warning-downloadcenter-no-file-empty = Downloadcenter har ingen fil för { $places }; { $count ->
        [one] rutan
       *[other] rutorna
    } { $names } förblev tomma
progress-k-five-unchanged = K5-mallen är oförändrad
progress-k-five-download = laddar ner K5-kartan (1:5000) från Geoportal Berlin WMS
warning-k-five-unavailable = K5-kartmallen är inte tillgänglig ({ $detail }); hoppades över

## The state border

progress-beyond-border = kartområdet sträcker sig över delstatsgränsen; bortom den är mallarna tomma där { $provider }-ortofotot och höjdmodellerna tar slut. ALKIS och OSM kommer från båda delstaterna
progress-beyond-border-no-data = kartområdet sträcker sig över delstatsgränsen; inga { $provider }-data för { $tiles }, så mallarna förblir tomma där

## OSM

warning-osm-extract-missing-kept = inget OSM-extrakt i cachen; behåller de sparade OSM-kandidaterna från { $date }
progress-osm-extract-read = läser OSM-extrakt { $region } från { $date }
progress-osm-candidates = { $count } OSM-kandidatobjekt

## Permit notes

progress-permits-fetch = hämtar { $source }-data för tillståndsnoteringarna
progress-permits-osm = läser OSM-extrakt { $region } från { $date } för tillståndsnoteringarna
progress-permits-stored = läser de sparade uppgifterna för tillståndsnoteringarna
warning-permits-source-unreachable = { $source } gick inte att läsa, så tillståndsnoteringarna utelämnar den: { $error }
warning-permits-osm-no-extract = inget OSM-extrakt finns i cachen för det här kartområdet, så tillståndsnoteringarna utelämnar OSM; `freemapper fetch` laddar ner det

## ALKIS

warning-alkis-coverage-unknown = täckningen för { $cadastre }-byggnader är okänd; kör `freemapper fetch` online för att spara den
progress-alkis-buildings-download = laddar ner { $cadastre }-byggnader
progress-alkis-buildings = { $count } { $cadastre }-byggnader
warning-alkis-no-buildings = inga { $cadastre }-byggnader hittades i kartområdet
progress-alkis-skipped = hoppade över { $count } { $cadastre }-objekt: { $list }
warning-alkis-buildings-changed = { $cadastre }-byggnaderna har ändrats hos källan ({ $changes })
progress-alkis-no-land-use-stored = ingen { $cadastre }-markanvändning sparad; kör `freemapper fetch` online för kandidater till förbjudna områden
progress-alkis-land-use-download = laddar ner { $cadastre }-markanvändning
progress-alkis-land-use-parts = { $count } { $cadastre }-markanvändningsdelar
warning-alkis-land-use-kept = { $cadastre }-markanvändning är inte tillgänglig ({ $error }); behåller den sparade markanvändningen
warning-alkis-land-use-removed = { $cadastre }-markanvändning är inte tillgänglig ({ $error }); den sparade markanvändningen täcker inte längre kartområdet, så freemapper tog bort den
warning-alkis-land-use-none = { $cadastre }-markanvändning är inte tillgänglig ({ $error }); inga kandidater till förbjudna områden
progress-bdot-filled = { $count } BDOT10k-objekt ({ $what }) tillagda där EGiB inte ger något ({ $powiat })
warning-bdot-failed = BDOT10k kunde inte fylla luckan i EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB är inte tillgänglig ({ $error }); BDOT10k används i stället ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = inga Berlin-ALKIS-strukturer sparade; kör `freemapper fetch` online för trappor, murar, träd och skärmtak
progress-berlin-removing = tar bort de sparade Berlin-lagren: kartområdet ligger inte längre nära Berlin
progress-berlin-download = laddar ner Berlin-ALKIS-strukturer, trädkataster, gatuinventering och punktlager
progress-berlin-features = { $count } Berlin-objekt i { $file }
warning-berlin-layer-kept = Berlin { $file } är inte tillgänglig ({ $detail }); behåller det sparade lagret
warning-berlin-layer-skipped = Berlin { $file } är inte tillgänglig ({ $detail }); lagret hoppades över
progress-feinkartierung-none-stored = ingen Hamburg-Feinkartierung sparad; kör `freemapper fetch` online för räcken, murar, trappor och gatuinredning
progress-feinkartierung-removing = tar bort den sparade Hamburg-Feinkartierung: kartområdet ligger inte längre i Hamburg
progress-feinkartierung-download = laddar ner Hamburg-Feinkartierung (gatuutrymme och offentligt grönt)
progress-feinkartierung-features = { $count } Feinkartierung-objekt i { $file }
warning-feinkartierung-layer-kept = Feinkartierung { $file } är inte tillgänglig ({ $detail }); behåller det sparade lagret
warning-feinkartierung-layer-skipped = Feinkartierung { $file } är inte tillgänglig ({ $detail }); lagret hoppades över
error-extra-layer-unknown = inte ett lager i { $folder }
warning-berlin-buildings-missing = inga ALKIS-byggnader sparade för det här området i Berlin; kör `freemapper fetch` för att hämta dem från Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = inga fullständiga ALKIS-strukturer sparade; kör `freemapper fetch` online för murar, staket, häckar och broar
progress-nas-removing = tar bort de sparade fullständiga ALKIS-lagren: kartområdet ligger inte längre i en delstat med NAS-tjänst
warning-nas-folder-outside = fullständiga ALKIS-lager hoppades över ({ $folder } leder ut ur projektmappen)
progress-nas-download = laddar ner de fullständiga ALKIS-strukturerna (NAS)
progress-nas-features = { $count } ALKIS NAS-objekt i { $file }
progress-nas-skipped = hoppade över { $count } ALKIS NAS-objekt i { $file }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } är inte tillgänglig ({ $detail }); behåller det sparade lagret
warning-nas-layer-skipped = ALKIS NAS { $file } är inte tillgänglig ({ $detail }); lagret hoppades över

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = inga objekt ur ATKIS Basis-DLM sparade; kör `freemapper fetch` online för källor, grottor, torn, minnesmärken och fontäner
progress-dlm-removing = tar bort de sparade lagren ur ATKIS Basis-DLM: kartområdet ligger inte längre i en delstat som tillhandahåller det
warning-dlm-folder-outside = lager ur ATKIS Basis-DLM hoppades över ({ $folder } leder ut ur projektmappen)
progress-dlm-download = laddar ner objekten ur ATKIS Basis-DLM
progress-dlm-features = { $count } ATKIS Basis-DLM-objekt i { $file }
progress-dlm-skipped = hoppade över { $count } ATKIS Basis-DLM-objekt i { $file }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } är inte tillgänglig ({ $detail }); behåller det sparade lagret
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } är inte tillgänglig ({ $detail }); lagret hoppades över

## Tree cadastres

progress-trees-none-stored = inget stadsträdkataster sparat; kör `freemapper fetch` online för de enskilda träden
progress-trees-removing = tar bort de sparade trädkatastren: kartområdet ligger inte längre i en stad som har ett
warning-trees-folder-outside = stadsträdkatastren hoppades över ({ $folder } leder ut ur projektmappen)
progress-trees-download = laddar ner stadsträdkatastren
progress-trees-count = { $count } { $id }-träd
progress-trees-skipped = hoppade över { $count } { $id }-träd: { $list }
warning-trees-kept = trädkatastret { $id } är inte tillgängligt ({ $detail }); behåller det sparade
warning-trees-skipped = trädkatastret { $id } är inte tillgängligt ({ $detail }); träden hoppades över

## Deriving the templates

progress-writing = skriver { $path }
progress-contours-steep = höjdkurvor var { $interval } m: terrängen är så brant att höjdkurvor var { $default } m skulle flyta ihop; en ekvidistans på { $default } m i projektet behåller dem (`--contour-interval { $default }`)
warning-osm-only-no-terrain = inga terrängdata för den här regionen ännu: kartan görs enbart av OpenStreetMap, utan höjdkurvor, mallar och byggnader
progress-no-ndsm-recorded = ingen nDSM registrerad; kör `freemapper fetch` för nDSM-mallen
warning-ndsm-tile-invalid = ogiltig nDSM-ruta { $tile } i freemapper.toml; hoppades över
warning-ndsm-not-cached = nDSM { $tile } finns inte i cachen; kör `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } går inte att läsa ({ $error }); ta bort { $path } och kör `freemapper fetch`
warning-surface-unusable = ytmodellen är oanvändbar ({ $error }); kör `freemapper fetch`

## Downloads and the cache

progress-downloading = laddar ner { $url }
progress-downloaded = nedladdat { $megabytes } MB
progress-downloaded-of = nedladdat { $megabytes } MB av { $total } MB
progress-tile-index-download = laddar ner rutindexet för { $product }
progress-tile-cached = cachad { $tile }
progress-surface-blocks = läser { $count } av de { $total } blocken i ytmodellrutan { $tile } ({ $megabytes } MB)
progress-surface-whole = ytmodellrutan { $tile } laddas ner i sin helhet: { $reason }
warning-tile-cached-older = använder den cachade { $product }-rutan { $tile } från { $published } (den nyare { $newer } laddades inte ner)
warning-ndsm-no-cached = ingen cachad nDSM för rutan { $tile }
warning-ndsm-tile-unavailable-cached = nDSM-rutan { $tile } är inte tillgänglig ({ $reason }); använder den cachade filen
warning-ndsm-tile-unavailable-skipped = nDSM-rutan { $tile } är inte tillgänglig ({ $reason }); hoppades över
warning-file-check-failed = kunde inte kontrollera om { $file } fortfarande är aktuell ({ $reason }); använder den cachade filen, och de andra cachade filerna från den här servern utan att fråga
warning-ldbv-no-buildings-file = ingen byggnadsfil för rutan { $tile } på servern; kartan kan sakna byggnader där
progress-ndsm-not-published = inget nDSM-punktmoln publicerat för rutan { $tile }
progress-ndsm-cached = cachad nDSM { $tile }
progress-point-cloud-not-published = inget laserpunktmoln publicerat för rutan { $tile }
progress-point-cloud-cached = cachat punktmoln { $tile }
warning-point-cloud-no-cached = inget cachat punktmoln för rutan { $tile }; inga vegetationskandidater där
warning-point-cloud-unavailable-cached = punktmolnet { $tile } är inte tillgängligt ({ $reason }); använder den cachade filen
warning-point-cloud-unavailable-skipped = punktmolnet { $tile } är inte tillgängligt ({ $reason }); inga vegetationskandidater där
warning-point-clouds-unavailable = laserpunktmolnen är inte tillgängliga ({ $reason }); vegetationskandidaterna använder de cachade rutorna, om det finns några
progress-no-point-clouds = inga laserpunktmoln för vegetationen här ännu: delstaten publicerar inga som freemapper kan läsa, eller inte i kartans koordinatsystem
warning-point-clouds-not-fetched = vegetationen är vald, men inga punktmoln är laddade; kör `freemapper fetch` för att ladda dem
warning-point-cloud-unreadable = punktmolnet { $tile } går inte att läsa ({ $error }); ta bort { $path } och kör `freemapper fetch`
warning-osm-nrw-nearest = inget distriktsextrakt för Nordrhein-Westfalen inom 100 m från området; läser det närmaste, { $district }
warning-osm-disk-full-cached = disken är full, så freemapper använder det cachade OSM-extraktet från { $date }. Frigör utrymme och kör `freemapper fetch` igen för ett nyare
warning-osm-geofabrik-unavailable = Geofabrik är inte tillgängligt ({ $error }); använder det cachade OSM-extraktet från { $date }
progress-osm-extract-cached = cachat OSM-extrakt { $region } från { $date }
progress-osm-extract-download = laddar ner { $url } (cirka { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = laddar ner bDOM-rutan { $tile } ({ $megabytes } MB, { $index } av { $total })
progress-sh-dgm-not-in-atom = DGM1-rutan { $tile } finns inte i INSPIRE ATOM-katalogen; laddar ner dess XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1-rutan { $tile } från INSPIRE ATOM-katalogen går inte att läsa ({ $error }); laddar ner dess XYZ ({ $megabytes } MB)
progress-sh-tile = ruta { $tile }: { $text }
progress-sh-download-again = ruta { $tile }: nedladdningen går inte att läsa ({ $message }); laddar ner den en gång till
warning-sh-portal-no-date = Downloadportal anger inget datum för { $model }-rutan { $tile }
warning-sh-dgm-xyz-used = DGM1-rutan { $tile } finns inte i INSPIRE ATOM-katalogen; använde dess XYZ från Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1-rutan { $tile } från INSPIRE ATOM-katalogen går inte att läsa ({ $error }); använde dess XYZ från Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }-rutan { $tile }: { $count } celler i kartområdet saknar höjd
warning-centre-tile-missing = { $model }-rutan { $tile } i kartområdets mitt saknas, så det finns inget datum
warning-centre-tile-unlisted = rutlistan nämner ingen { $model }-ruta { $tile } i kartområdets mitt, så det finns inget datum
warning-list-unreadable-kept = rutlistan för { $model } gick inte att läsa ({ $error }); använde den som sparades från en tidigare körning
warning-portal-list-unreadable-kept = Downloadportals rutlista för { $model } gick inte att läsa ({ $error }); använde den som sparades från en tidigare körning
warning-surface-much-older = { $model } är från { $surface }, ortofotot från { $flight }. nDSM kan sakna nyare byggnader och träd.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objekt i rutlistan har ingen användbar rutnyckel; hoppades över
warning-rp-list-unreadable-kept = LVermGeo RP:s rutlista gick inte att läsa ({ $error }); använde den som sparades från en tidigare körning
warning-rp-metalink-not-kept = LVermGeo RP:s Metalink sparades inte: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP:s Metalink för { $model } gick inte att läsa ({ $detail }); använde den som sparades från en tidigare körning
warning-rp-metalink-unreadable = LVermGeo RP:s Metalink för { $model } gick inte att läsa ({ $detail }), så de rutor som rutlistan inte anger något flygår för förblir utan höjder
warning-rp-tile-without-name = { $model }-rutan { $tile }: den här kilometerrutan har inga höjder i kartan. LVermGeo RP:s rutlista (Laserbefliegung) { $note } för den, så filnamnet är okänt{ $either ->
        [metalink] , och dess Metalink anger ingen fil för den heller
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }-rutan { $tile } har inget datum: LVermGeo RP:s rutlista (Laserbefliegung) { $note } för den
word-rp-list-neither = anger varken flygår eller flygdagar
word-rp-list-no-days = anger inga flygdagar
word-rp-list-no-year = anger inget flygår
word-rp-list-days-unreadable = anger flygdagar { $days } som inte går att läsa
word-rp-list-disagree = anger flygdagar { $days } och ett flygår { $year } som inte stämmer överens om ett och samma år

## Hesse

warning-hvbg-undated = metadatalistan har inget läsbart Bildflugdatum för rutan { $tiles }; deras projektposter har inget flygdatum
warning-hvbg-undated-more = metadatalistan har inget läsbart Bildflugdatum för rutan { $tiles } och { $more } till; deras projektposter har inget flygdatum
warning-hvbg-list-unreadable-older = den nya HVBG-metadatalistan { $name } gick inte att läsa ({ $detail }); använder den äldre listan { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: inget datum, STAC listar ingen ruta i kartområdets mitt
warning-lgln-no-date = { $model }: inget datum ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; den sparade modellen behåller sitt registrerade datum
warning-mv-cells-empty = { $model }: { $count } celler inom rutorna i Mecklenburg-Vorpommern saknar höjd
warning-mv-centre-no-date = { $model }: inget datum för mittrutan { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } rader i rutlistan för { $model } har inget användbart rutnamn; hoppades över
warning-nrw-list-rows-undated = { $count } rader i rutlistan för { $model } har inget läsbart datum
warning-nrw-list-older = serverns rutlista för { $model } är daterad { $server }, äldre än den sparade från { $kept }; använde den sparade
warning-nrw-list-no-date = rutlistan anger inget datum för { $model }-rutan { $tile }

## After `init`

note-scale-outside-norm = obs: 1:5000 ingår inte i sprintnormen; freemapper-print varnar för det och symbolerna förblir på 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } är inget OSM-id som way/992423119
error-place-invalid-answer = ogiltigt svar från platssökningen: { $detail }
error-place-not-found = { $id } hittades inte. Nominatim listar bara taggade OSM-objekt, så en otaggad yttre linje i en multipolygon finns inte där. Använd i stället multipolygonrelationen (--osm relation/<id>), eller spara konturen som GeoJSON och använd --geojson
error-place-is-a-point = { $id } är en punkt, inte ett område; välj dess område (en linje eller relation)
error-place-no-such-part = { $id } har { $count ->
        [one] 1 del
       *[other] { $count } delar
    }; det finns ingen del { $part }
error-place-several-parts = { $id } är ett område med { $count } separata delar ({ $sizes }). Ett kartområde är en polygon: välj en del med --part <n> (1 är den största), eller spara den kontur du vill ha som GeoJSON och använd --geojson
error-layer-not-a-world-file = { $path }: inte en world-fil
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = ingen förhandsvisning för { $layer }; lager med förhandsvisning: { $names }
error-layer-no-ndsm-published = ingen nDSM är publicerad för rutorna i det här kartområdet, så det finns ingen ndsm-mall
error-layer-not-made = { $layer } är inte gjord än; kör { $step ->
        [fetch] freemapper fetch eller run (MCP: start_pipeline); endast Berlin
        [export] freemapper export eller run (MCP: start_pipeline)
       *[derive] freemapper derive eller run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD-kartan får ingen bakgrund av den
warning-map-preview-failed = kartan är skriven, men dess förhandsvisning saknas: { $error }
warning-training-no-tree-cover = träningskartan behåller parkernas öppna mark hel: trädtäckningen (data/tree-cover.tif) saknas; kör `freemapper derive` igen
