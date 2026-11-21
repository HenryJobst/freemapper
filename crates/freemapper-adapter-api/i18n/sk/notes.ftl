# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = warning: { $text }

warning-other-zone = oblasť mapy leží v pásme UTM { $zone } (EPSG:{ $epsg }); freemapper v ňom stiahne všetko znova
warning-other-state = oblasť mapy teraz leží v dátach inej spolkovej krajiny; freemapper stiahne všetko znova
warning-beyond-coverage = časť oblasti mapy leží mimo podporovaných spolkových krajín. Leží v zahraničí, na mori alebo v spolkovej krajine, ktorú freemapper zatiaľ nepodporuje, napríklad v Durínsku. Spolkové krajiny tam nemajú dáta, takže ortofoto môže byť biele a ALKIS nedá žiadne budovy ani využitie pôdy
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: mierka 1:{ $scale } nie je medzi mierkami pre { $kind } mapy ({ $allowed }); freemapper-print mapu odmietne
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: záznam { $key } { $what ->
        [tree] vrstiev stromov
        [NAS] vrstiev NAS
        [set] doplnkových vrstiev
       *[other] vrstiev { $german }
    } sa ignoruje: { $reason ->
        [name] nie je obyčajný názov súboru
       *[unknown] nie je vrstva, ktorú freemapper pozná
    }

## Disk space

warning-disk-space-unreadable = voľné miesto na disku { $path } sa nedá prečítať ({ $error }), preto ho freemapper nekontroluje

## Tiles

warning-tile-changed = dlaždica { $tile } produktu { $product } sa u zdroja zmenila ({ $old } → { $new })
warning-bdom-tile-unavailable = model povrchu (bDOM) dlaždice { $tile } nie je k dispozícii ({ $error }); tam nie je podklad nDSM
warning-bdom-unavailable = model povrchu (bDOM) nie je k dispozícii ({ $error }); bez podkladu nDSM pre { $tiles }
progress-ndsm-from-bdom = nDSM dlaždice { $tile } z modelu povrchu (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] model terénu
        [surface] model povrchu
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = sťahuje sa { $model }
progress-model-read = číta sa { $model }
warning-model-unavailable = { $model } nie je k dispozícii ({ $error }); bez podkladu nDSM
warning-model-unavailable-kept = { $model } nie je k dispozícii ({ $error }); zachovaný zostáva model povrchu stiahnutý { $fetched }
warning-model-tile-unavailable = { $model } dlaždice { $tile } nie je k dispozícii ({ $error }); tam bez podkladu nDSM
warning-archive-passed-over = { $model }: archív sa nepodarilo prečítať ({ $error }); bol preskočený a dlaždice oblasti mapy pochádzajú z ostatných archívov
warning-archive-directory-kept = { $model }: archív sa nepodarilo prečítať ({ $error }); použije sa jeho adresár z predchádzajúceho behu, takže dlaždica, ktorú LVGL odvtedy nahradil, sa nezaznamená
warning-model-tiles-missing = { $model }: { $state ->
        [the-map-area] V oblasti mapy
        [hamburg] V Hamburgu
        [hesse] V Hesensku
        [north-rhine-westphalia] V Severnom Porýní-Vestfálsku
        [schleswig-holstein] V Šlezvicku-Holštajnsku
        [mecklenburg-vorpommern] V Meklenbursku-Predpomoransku
        [lower-saxony] V Dolnom Sasku
        [bavaria] V Bavorsku
       *[other] V { $state }
    } { $count ->
        [one] chýba dlaždica
        [few] chýbajú dlaždice
       *[other] chýbajú dlaždice
    } { $names }; podklady tam zostanú prázdne
warning-no-surface-stored = nie je uložený model povrchu; pre podklad nDSM spusti `freemapper fetch` online
warning-surface-older-than-orthophoto = model povrchu (bDOM { $surface }) je starší než ortofoto ({ $photo }); v nDSM môžu chýbať nové budovy
progress-surface-kept = freemapper ponecháva model povrchu stiahnutý { $fetched } pre podklad nDSM
progress-surface-tiles-kept = freemapper ponecháva uložený model povrchu pre { $count ->
        [one] dlaždicu
        [few] dlaždice
       *[other] dlaždíc
    } { $names } pre podklad nDSM
progress-orthophoto-unchanged = ortofoto nezmenené
progress-orthophoto-unchanged-dated = ortofoto nezmenené ({ $label } { $date })
progress-orthophoto-download = sťahuje sa ortofoto zo služby { $provider } { $service }
progress-orthophoto-in-leaf = ortofoto bolo nasnímané { $date }, keď mali stromy lístie; ich koruny zakrývajú, čo leží pod nimi
progress-orthophoto-bare = ortofoto bolo nasnímané { $date }, keď boli stromy bez lístia
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; uložené ortofoto a dátum jeho zhotovenia { $date } zostali zachované
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): bez dátumu zhotovenia, { $reason }
word-stac-no-image = STAC nevedie v strede oblasti mapy žiadny snímok
word-stac-no-answer = vyhľadávanie v STAC nedalo použiteľnú odpoveď ({ $error })
warning-downloadcenter-no-file = Downloadcenter nemá súbor pre { $places }
warning-downloadcenter-no-file-empty = Downloadcenter nemá súbor pre { $places }; { $count ->
        [one] dlaždica { $names } zostala prázdna
        [few] dlaždice { $names } zostali prázdne
       *[other] dlaždíc { $names } zostalo prázdnych
    }
progress-k-five-unchanged = podklad K5 nezmenený
progress-k-five-download = sťahuje sa mapa K5 (1:5000) zo služby WMS Geoportal Berlin
warning-k-five-unavailable = mapový podklad K5 nie je k dispozícii ({ $detail }); preskočené

## The state border

progress-beyond-border = oblasť mapy zasahuje za hranicu spolkovej krajiny; za ňou sú podklady prázdne všade, kde ortofoto a výškové modely { $provider } končia. ALKIS a OSM pochádzajú z oboch spolkových krajín
progress-beyond-border-no-data = oblasť mapy zasahuje za hranicu spolkovej krajiny; pre { $tiles } nie sú dáta { $provider }, takže tam podklady zostanú prázdne

## OSM

warning-osm-extract-missing-kept = vo vyrovnávacej pamäti nie je výrez OSM; ponechávajú sa uložení kandidáti OSM z { $date }
progress-osm-extract-read = číta sa výrez OSM { $region } z { $date }
progress-osm-candidates = { $count } kandidátskych objektov OSM

## Permit notes

progress-permits-fetch = sťahujú sa dáta { $source } pre poznámky k povoleniam
progress-permits-osm = číta sa výrez OSM { $region } z { $date } pre poznámky k povoleniam
progress-permits-stored = čítajú sa uložené dáta poznámok k povoleniam
warning-permits-source-unreachable = { $source } sa nepodarilo prečítať, preto ho poznámky k povoleniam vynechávajú: { $error }
warning-permits-osm-no-extract = pre túto oblasť mapy nie je vo vyrovnávacej pamäti výrez OSM, preto poznámky k povoleniam OSM vynechávajú; `freemapper fetch` ho stiahne

## ALKIS

warning-alkis-coverage-unknown = pokrytie budov ALKIS je neznáme; na jeho zaznamenanie spusti `freemapper fetch` online
progress-alkis-buildings-download = sťahujú sa budovy ALKIS
progress-alkis-buildings = { $count } budov ALKIS
warning-alkis-no-buildings = v oblasti mapy sa nenašli žiadne budovy ALKIS
progress-alkis-skipped = preskočených { $count } objektov ALKIS: { $list }
warning-alkis-buildings-changed = budovy ALKIS sa u zdroja zmenili ({ $changes })
progress-alkis-no-land-use-stored = nie je uložené využitie pôdy ALKIS; pre kandidátov na plochy, do ktorých sa nesmie vstupovať, spusti `freemapper fetch` online
progress-alkis-land-use-download = sťahuje sa využitie pôdy ALKIS
progress-alkis-land-use-parts = { $count } častí využitia pôdy ALKIS
warning-alkis-land-use-kept = využitie pôdy ALKIS nie je k dispozícii ({ $error }); ponecháva sa uložené využitie pôdy
warning-alkis-land-use-removed = využitie pôdy ALKIS nie je k dispozícii ({ $error }); uložené využitie pôdy už nepokrýva oblasť mapy, preto ho freemapper odstránil
warning-alkis-land-use-none = využitie pôdy ALKIS nie je k dispozícii ({ $error }); bez kandidátov na plochy, do ktorých sa nesmie vstupovať

## The Berlin layers

progress-berlin-none-stored = nie sú uložené berlínske štruktúry ALKIS; pre schody, múry, stromy a prístrešky spusti `freemapper fetch` online
progress-berlin-removing = odstraňujú sa uložené berlínske vrstvy: oblasť mapy už nie je pri Berlíne
progress-berlin-download = sťahujú sa berlínske štruktúry ALKIS, kataster stromov, mapovanie ulíc a bodové vrstvy
progress-berlin-features = { $count } objektov Berlín { $file }
warning-berlin-layer-kept = Berlín { $file } nie je k dispozícii ({ $detail }); ponecháva sa uložená vrstva
warning-berlin-layer-skipped = Berlín { $file } nie je k dispozícii ({ $detail }); vrstva preskočená
progress-feinkartierung-none-stored = nie je uložené hamburské Feinkartierung; pre zábradlia, múry, schody a ulicový mobiliár spusti `freemapper fetch` online
progress-feinkartierung-removing = odstraňuje sa uložené hamburské Feinkartierung: oblasť mapy už nie je v Hamburgu
progress-feinkartierung-download = sťahuje sa hamburské Feinkartierung (uličný priestor a verejná zeleň)
progress-feinkartierung-features = { $count } objektov Feinkartierung { $file }
warning-feinkartierung-layer-kept = Feinkartierung { $file } nie je k dispozícii ({ $detail }); ponecháva sa uložená vrstva
warning-feinkartierung-layer-skipped = Feinkartierung { $file } nie je k dispozícii ({ $detail }); vrstva preskočená
error-extra-layer-unknown = nie je vrstva z { $folder }
warning-berlin-buildings-missing = pre túto oblasť v Berlíne nie sú uložené budovy ALKIS; `freemapper fetch` ich stiahne z Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = nie sú uložené úplné štruktúry ALKIS; pre múry, ploty, živé ploty a mosty spusti `freemapper fetch` online
progress-nas-removing = odstraňujú sa uložené úplné vrstvy ALKIS: oblasť mapy už nie je v spolkovej krajine so službou NAS
warning-nas-folder-outside = úplné vrstvy ALKIS preskočené ({ $folder } vedie mimo priečinka projektu)
progress-nas-download = sťahujú sa úplné štruktúry ALKIS (NAS)
progress-nas-features = { $count } objektov ALKIS NAS { $file }
progress-nas-skipped = preskočených { $count } objektov ALKIS NAS { $file }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } nie je k dispozícii ({ $detail }); ponecháva sa uložená vrstva
warning-nas-layer-skipped = ALKIS NAS { $file } nie je k dispozícii ({ $detail }); vrstva preskočená

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nie sú uložené objekty ATKIS Basis-DLM; pre pramene, jaskyne, veže, pamätníky a fontány spusti `freemapper fetch` online
progress-dlm-removing = odstraňujú sa uložené vrstvy ATKIS Basis-DLM: oblasť mapy už nie je v spolkovej krajine, ktorá ich poskytuje
warning-dlm-folder-outside = vrstvy ATKIS Basis-DLM preskočené ({ $folder } vedie mimo priečinka projektu)
progress-dlm-download = sťahujú sa objekty ATKIS Basis-DLM
progress-dlm-features = { $count } objektov ATKIS Basis-DLM { $file }
progress-dlm-skipped = preskočených { $count } objektov ATKIS Basis-DLM { $file }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } nie je k dispozícii ({ $detail }); ponecháva sa uložená vrstva
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } nie je k dispozícii ({ $detail }); vrstva preskočená

## Tree cadastres

progress-trees-none-stored = nie je uložený mestský kataster stromov; pre jednotlivé stromy spusti `freemapper fetch` online
progress-trees-removing = odstraňujú sa uložené katastre stromov: oblasť mapy už nie je v meste, ktoré ho má
warning-trees-folder-outside = mestské katastre stromov preskočené ({ $folder } vedie mimo priečinka projektu)
progress-trees-download = sťahujú sa mestské katastre stromov
progress-trees-count = { $count } stromov { $id }
progress-trees-skipped = preskočených { $count } stromov { $id }: { $list }
warning-trees-kept = kataster stromov { $id } nie je k dispozícii ({ $detail }); ponecháva sa uložený
warning-trees-skipped = kataster stromov { $id } nie je k dispozícii ({ $detail }); stromy preskočené

## Deriving the templates

progress-writing = zapisuje sa { $path }
progress-contours-steep = vrstevnice po { $interval } m: terén je taký strmý, že by vrstevnice po { $default } m splývali; interval vrstevníc { $default } m v projekte ich zachová (`--contour-interval { $default }`)
warning-osm-only-no-terrain = pre tento región zatiaľ nie sú dáta terénu: mapa vzniká iba z OpenStreetMap, bez vrstevníc, podkladov a budov
progress-no-ndsm-recorded = nie je zaznamenaný nDSM; pre podklad nDSM spusti `freemapper fetch`
warning-ndsm-tile-invalid = neplatná dlaždica nDSM { $tile } vo freemapper.toml; preskočená
warning-ndsm-not-cached = nDSM { $tile } nie je vo vyrovnávacej pamäti; spusti `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } sa nedá prečítať ({ $error }); zmaž { $path } a spusti `freemapper fetch`
warning-surface-unusable = model povrchu je nepoužiteľný ({ $error }); spusti `freemapper fetch`

## Downloads and the cache

progress-downloading = sťahuje sa { $url }
progress-downloaded = stiahnuté { $megabytes } MB
progress-downloaded-of = stiahnuté { $megabytes } MB z { $total } MB
progress-tile-index-download = sťahuje sa index dlaždíc { $product }
progress-tile-cached = vo vyrovnávacej pamäti { $tile }
progress-surface-blocks = číta sa { $count } z { $total } blokov dlaždice modelu povrchu { $tile } ({ $megabytes } MB)
progress-surface-whole = dlaždica modelu povrchu { $tile } sa sťahuje celá: { $reason }
warning-tile-cached-older = používa sa dlaždica { $tile } produktu { $product } z vyrovnávacej pamäte z { $published } (novšia { $newer } sa nestiahla)
warning-ndsm-no-cached = vo vyrovnávacej pamäti nie je nDSM pre dlaždicu { $tile }
warning-ndsm-tile-unavailable-cached = dlaždica nDSM { $tile } nie je k dispozícii ({ $reason }); používa sa súbor z vyrovnávacej pamäte
warning-ndsm-tile-unavailable-skipped = dlaždica nDSM { $tile } nie je k dispozícii ({ $reason }); preskočená
warning-file-check-failed = nepodarilo sa overiť, či je { $file } ešte aktuálny ({ $reason }); používa sa súbor z vyrovnávacej pamäte a ostatné súbory tohto servera z vyrovnávacej pamäte bez pýtania
warning-ldbv-no-buildings-file = na serveri nie je súbor budov pre dlaždicu { $tile }; na mape tam môžu chýbať budovy
progress-ndsm-not-published = pre dlaždicu { $tile } nie je zverejnené mračno bodov nDSM
progress-ndsm-cached = nDSM { $tile } vo vyrovnávacej pamäti
progress-point-cloud-not-published = pre dlaždicu { $tile } nie je zverejnené laserové mračno bodov
progress-point-cloud-cached = mračno bodov { $tile } vo vyrovnávacej pamäti
warning-point-cloud-no-cached = vo vyrovnávacej pamäti nie je mračno bodov pre dlaždicu { $tile }; tam bez kandidátov vegetácie
warning-point-cloud-unavailable-cached = mračno bodov { $tile } nie je k dispozícii ({ $reason }); používa sa súbor z vyrovnávacej pamäte
warning-point-cloud-unavailable-skipped = mračno bodov { $tile } nie je k dispozícii ({ $reason }); tam bez kandidátov vegetácie
warning-point-clouds-unavailable = laserové mračná bodov nie sú k dispozícii ({ $reason }); kandidáti vegetácie použijú dlaždice z vyrovnávacej pamäte, ak nejaké sú
progress-no-point-clouds = pre vegetáciu tu zatiaľ nie sú laserové mračná bodov: spolková krajina nezverejňuje žiadne, ktoré freemapper vie čítať, alebo nie v súradnicovom systéme mapy
warning-point-clouds-not-fetched = vegetácia je zvolená, ale nie sú načítané žiadne mračná bodov; načítaš ich cez `freemapper fetch`
warning-point-cloud-unreadable = mračno bodov { $tile } sa nedá prečítať ({ $error }); zmaž { $path } a spusti `freemapper fetch`
warning-osm-nrw-nearest = do 100 m od oblasti nie je výrez okresu zo Severného Porýnia-Vestfálska; číta sa najbližší, { $district }
warning-osm-disk-full-cached = disk je plný, preto freemapper používa výrez OSM z vyrovnávacej pamäte z { $date }. Uvoľni miesto a spusti `freemapper fetch` znova, aby si dostal novší
warning-osm-geofabrik-unavailable = Geofabrik nie je k dispozícii ({ $error }); používa sa výrez OSM z vyrovnávacej pamäte z { $date }
progress-osm-extract-cached = výrez OSM { $region } z { $date } vo vyrovnávacej pamäti
progress-osm-extract-download = sťahuje sa { $url } (približne { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = sťahuje sa dlaždica bDOM { $tile } ({ $megabytes } MB, { $index } z { $total })
progress-sh-dgm-not-in-atom = dlaždica DGM1 { $tile } nie je v adresári INSPIRE ATOM; sťahuje sa jej XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = dlaždica DGM1 { $tile } z adresára INSPIRE ATOM sa nedá prečítať ({ $error }); sťahuje sa jej XYZ ({ $megabytes } MB)
progress-sh-tile = dlaždica { $tile }: { $text }
progress-sh-download-again = dlaždica { $tile }: stiahnutý súbor sa nedá prečítať ({ $message }); sťahuje sa ešte raz
warning-sh-portal-no-date = Downloadportal neuvádza dátum pre dlaždicu { $tile } modelu { $model }
warning-sh-dgm-xyz-used = dlaždica DGM1 { $tile } nie je v adresári INSPIRE ATOM; použil sa jej XYZ z Downloadportal
warning-sh-dgm-xyz-used-unreadable = dlaždica DGM1 { $tile } z adresára INSPIRE ATOM sa nedá prečítať ({ $error }); použil sa jej XYZ z Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = dlaždica { $tile } modelu { $model }: { $count } buniek oblasti mapy nemá výšku
warning-centre-tile-missing = dlaždica { $tile } modelu { $model } v strede oblasti mapy chýba, preto nie je dátum
warning-centre-tile-unlisted = zoznam dlaždíc neuvádza dlaždicu { $tile } modelu { $model } v strede oblasti mapy, preto nie je dátum
warning-list-unreadable-kept = zoznam dlaždíc modelu { $model } sa nepodarilo prečítať ({ $error }); použil sa zoznam zachovaný z predchádzajúceho behu
warning-portal-list-unreadable-kept = zoznam dlaždíc modelu { $model } z Downloadportal sa nepodarilo prečítať ({ $error }); použil sa zoznam zachovaný z predchádzajúceho behu
warning-surface-much-older = { $model } pochádza z { $surface }, ortofoto z { $flight }. V nDSM môžu chýbať novšie budovy a stromy.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objektov zoznamu dlaždíc nemá použiteľný kľúč dlaždice; preskočené
warning-rp-list-unreadable-kept = zoznam dlaždíc LVermGeo RP sa nepodarilo prečítať ({ $error }); použil sa zoznam zachovaný z predchádzajúceho behu
warning-rp-metalink-not-kept = Metalink LVermGeo RP sa neuložil: { $error }
warning-rp-metalink-unreadable-kept = Metalink LVermGeo RP modelu { $model } sa nepodarilo prečítať ({ $detail }); použil sa Metalink zachovaný z predchádzajúceho behu
warning-rp-metalink-unreadable = Metalink LVermGeo RP modelu { $model } sa nepodarilo prečítať ({ $detail }), preto dlaždice, pre ktoré jeho zoznam dlaždíc neuvádza rok nasnímania, zostanú bez výšok
warning-rp-tile-without-name = dlaždica { $tile } modelu { $model }: tento štvorec 1 km nemá na mape výšky. Zoznam dlaždíc LVermGeo RP (Laserbefliegung) { $note } pre ňu, takže názov súboru nie je známy{ $either ->
        [metalink] , a ani jeho Metalink pre ňu neuvádza žiadny súbor
       *[none] {""}
    }
warning-rp-centre-no-date = dlaždica { $tile } modelu { $model } nemá dátum: zoznam dlaždíc LVermGeo RP (Laserbefliegung) { $note } pre ňu
word-rp-list-neither = neuvádza ani rok nasnímania, ani dni nasnímania
word-rp-list-no-days = neuvádza dni nasnímania
word-rp-list-no-year = neuvádza rok nasnímania
word-rp-list-days-unreadable = uvádza dni nasnímania { $days }, ktoré sa nedajú prečítať
word-rp-list-disagree = uvádza dni nasnímania { $days } a rok nasnímania { $year }, ktoré sa nezhodujú na jednom roku

## Hesse

warning-hvbg-undated = zoznam metadát nemá čitateľný Bildflugdatum pre dlaždicu { $tiles }; ich záznamy projektu nemajú dátum nasnímania
warning-hvbg-undated-more = zoznam metadát nemá čitateľný Bildflugdatum pre dlaždicu { $tiles } a { $more } ďalších; ich záznamy projektu nemajú dátum nasnímania
warning-hvbg-list-unreadable-older = nový zoznam metadát HVBG { $name } sa nepodarilo prečítať ({ $detail }); používa sa starší zoznam { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: bez dátumu, STAC neuvádza v strede oblasti mapy žiadnu dlaždicu
warning-lgln-no-date = { $model }: bez dátumu ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; uložený model si ponecháva zaznamenaný dátum
warning-mv-cells-empty = { $model }: { $count } buniek v dlaždiciach Meklenburska-Predpomoranska nemá výšku
warning-mv-centre-no-date = { $model }: bez dátumu pre strednú dlaždicu { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } riadkov zoznamu dlaždíc { $model } nemá použiteľný názov dlaždice; preskočené
warning-nrw-list-rows-undated = { $count } riadkov zoznamu dlaždíc { $model } nemá čitateľný dátum
warning-nrw-list-older = zoznam dlaždíc modelu { $model } na serveri je z { $server }, starší než zachovaný z { $kept }; použil sa zachovaný
warning-nrw-list-no-date = zoznam dlaždíc neuvádza dátum pre dlaždicu { $tile } modelu { $model }

## After `init`

note-scale-outside-norm = note: 1:5000 nie je v norme pre šprint; freemapper-print na to upozorní a symboly zostanú na 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } nie je id OSM ako way/992423119
error-place-invalid-answer = neplatná odpoveď vyhľadávania miesta: { $detail }
error-place-not-found = { $id } sa nenašlo. Nominatim vedie iba označené objekty OSM, takže neoznačená vonkajšia cesta multipolygónu tam nie je. Použi namiesto toho reláciu multipolygónu (--osm relation/<id>) alebo ulož obrys ako GeoJSON a použi --geojson
error-place-is-a-point = { $id } je bod, nie plocha; vyber jeho areál (cestu alebo reláciu)
error-place-no-such-part = { $id } má { $count ->
        [one] 1 časť
        [few] { $count } časti
       *[other] { $count } častí
    }; časť { $part } neexistuje
error-place-several-parts = { $id } je oblasť z { $count } oddelených častí ({ $sizes }). Oblasť mapy je jeden mnohouholník: vyber časť cez --part <n> (1 je najväčšia) alebo ulož obrys, ktorý chceš, ako GeoJSON a použi --geojson
error-layer-not-a-world-file = { $path }: nie je world file
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = pre { $layer } nie je náhľad; vrstvy s náhľadmi: { $names }
error-layer-no-ndsm-published = pre dlaždice tejto oblasti mapy nie je zverejnený nDSM, preto neexistuje podklad ndsm
error-layer-not-made = { $layer } ešte nevznikol; spusti { $step ->
        [fetch] freemapper fetch alebo run (MCP: start_pipeline); iba Berlín
        [export] freemapper export alebo run (MCP: start_pipeline)
       *[derive] freemapper derive alebo run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; mapa OCAD z neho nedostane podklad
warning-map-preview-failed = mapa je zapísaná, ale jej náhľad chýba: { $error }
warning-training-no-tree-cover = tréningová mapa ponecháva otvorené plochy parkov celé: chýba pokrytie stromami (data/tree-cover.tif); spusti `freemapper derive` znova
