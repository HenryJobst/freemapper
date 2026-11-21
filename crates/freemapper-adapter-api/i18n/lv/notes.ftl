# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = brīdinājums: { $text }

warning-other-zone = kartes apgabals atrodas UTM zonā { $zone } (EPSG:{ $epsg }); freemapper visu lejupielādē no jauna šajā zonā
warning-other-state = kartes apgabals tagad atrodas citas federālās zemes datos; freemapper visu lejupielādē no jauna
warning-beyond-coverage = daļa kartes apgabala atrodas ārpus atbalstītajām federālajām zemēm. Tā atrodas ārzemēs, jūrā vai federālajā zemē, ko freemapper vēl neatbalsta, piemēram, Tīringenē. Tur zemēm nav datu, tāpēc ortofoto var būt balts un ALKIS nedod ēkas vai zemes izmantošanu
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: mērogs 1:{ $scale } nav neviens no { $kind } kartēm paredzētajiem mērogiem ({ $allowed }); freemapper-print karti noraidīs
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorēts { $german ->
       *[any] { $what }
    } slāņa ieraksts { $key }: { $reason ->
        [name] nav vienkāršs faila nosaukums
       *[unknown] nav slānis, ko freemapper pazīst
    }

## Disk space

warning-disk-space-unreadable = nevar nolasīt brīvo vietu { $path } diskā ({ $error }), tāpēc freemapper to nepārbauda

## Tiles

warning-tile-changed = { $product } flīze { $tile } avotā mainījusies ({ $old } → { $new })
warning-bdom-tile-unavailable = flīzes { $tile } virsmas modelis (bDOM) nav pieejams ({ $error }); tur nav nDSM veidnes
warning-bdom-unavailable = virsmas modelis (bDOM) nav pieejams ({ $error }); nav nDSM veidnes flīzēm { $tiles }
progress-ndsm-from-bdom = flīzes { $tile } nDSM no virsmas modeļa (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] reljefa modelis
        [surface] virsmas modelis
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = lejupielādē: { $model }
progress-model-read = nolasa: { $model }
warning-model-unavailable = { $model }: dati nav pieejami ({ $error }); nav nDSM veidnes
warning-model-unavailable-kept = { $model }: dati nav pieejami ({ $error }); atstāts { $fetched } lejupielādētais virsmas modelis
warning-model-tile-unavailable = { $model }: flīzes { $tile } dati nav pieejami ({ $error }); tur nav nDSM veidnes
warning-archive-passed-over = { $model }: arhīvu nevarēja nolasīt ({ $error }); tas tika izlaists, un kartes apgabala flīzes nāk no pārējiem arhīviem
warning-archive-directory-kept = { $model }: arhīvu nevarēja nolasīt ({ $error }); tiek izmantots tā satura saraksts no iepriekšējās palaišanas, tāpēc flīze, ko LVGL kopš tā laika nomainīja, netiek pamanīta
warning-model-tiles-missing = { $model }: trūkst { $count ->
        [one] flīzes
       *[other] flīžu
    } { $names } { $state ->
        [the-map-area] kartes apgabalā
        [hamburg] Hamburgā
        [hesse] Hesē
        [north-rhine-westphalia] Ziemeļreinā-Vestfālenē
        [schleswig-holstein] Šlēsvigā-Holšteinā
        [mecklenburg-vorpommern] Meklenburgā-Priekšpomerānijā
        [lower-saxony] Lejassaksijā
        [bavaria] Bavārijā
       *[other] reģionā { $state }
    }; veidnes tur paliek tukšas
warning-no-surface-stored = virsmas modelis nav saglabāts; nDSM veidnei palaidiet `freemapper fetch` tiešsaistē
warning-surface-older-than-orthophoto = virsmas modelis (bDOM { $surface }) ir vecāks par ortofoto ({ $photo }); nDSM var trūkt jaunu ēku
progress-surface-kept = freemapper nDSM veidnei patur { $fetched } lejupielādēto virsmas modeli
progress-surface-tiles-kept = freemapper nDSM veidnei patur kešoto virsmas modeli { $count ->
        [one] flīzei
       *[other] flīzēm
    } { $names }
progress-orthophoto-unchanged = ortofoto nav mainīts
progress-orthophoto-unchanged-dated = ortofoto nav mainīts ({ $label } { $date })
progress-orthophoto-download = lejupielādē ortofoto: { $provider } { $service }
progress-orthophoto-in-leaf = ortofoto uzņemts { $date }, kad kokiem bija lapas; to vainagi aizsedz to, kas atrodas zem tiem
progress-orthophoto-bare = ortofoto uzņemts { $date }, kad koki bija kaili
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; atstāts saglabātais ortofoto un tā uzņemšanas datums { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): nav uzņemšanas datuma, { $reason }
word-stac-no-image = STAC neuzskaita nevienu attēlu kartes apgabala centrā
word-stac-no-answer = STAC meklēšana nedeva lietojamu atbildi ({ $error })
warning-downloadcenter-no-file = Downloadcenter nav faila šim: { $places }
warning-downloadcenter-no-file-empty = Downloadcenter nav faila šim: { $places }; { $count ->
        [one] flīze { $names } palika tukša
       *[other] flīzes { $names } palika tukšas
    }
progress-k-five-unchanged = K5 veidne nav mainīta
progress-k-five-download = lejupielādē K5 karti (1:5000) no Geoportal Berlin WMS
warning-k-five-unavailable = K5 kartes veidne nav pieejama ({ $detail }); izlaista

## The state border

progress-beyond-border = kartes apgabals sniedzas pāri federālās zemes robežai; aiz tās veidnes ir tukšas visur, kur { $provider } ortofoto un augstuma modeļi beidzas. ALKIS un OSM nāk no abām federālajām zemēm
progress-beyond-border-no-data = kartes apgabals sniedzas pāri federālās zemes robežai; nav { $provider } datu flīzēm { $tiles }, tāpēc veidnes tur paliek tukšas

## OSM

warning-osm-extract-missing-kept = kešatmiņā nav OSM izvilkuma; tiek atstāti saglabātie OSM kandidāti no { $date }
progress-osm-extract-read = nolasa OSM izvilkumu { $region } no { $date }
progress-osm-candidates = OSM kandidātu objekti: { $count }

## Permit notes

progress-permits-fetch = lejupielādē { $source } datus atļauju piezīmēm
progress-permits-osm = nolasa OSM izvilkumu { $region } no { $date } atļauju piezīmēm
progress-permits-stored = nolasa saglabātos atļauju piezīmju datus
warning-permits-source-unreachable = { $source } nevarēja nolasīt, tāpēc atļauju piezīmes to izlaiž: { $error }
warning-permits-osm-no-extract = šim kartes apgabalam kešatmiņā nav OSM izvilkuma, tāpēc atļauju piezīmes OSM izlaiž; `freemapper fetch` to lejupielādē

## ALKIS

warning-alkis-coverage-unknown = ALKIS ēku pārklājums nav zināms; palaidiet `freemapper fetch` tiešsaistē, lai to ierakstītu
progress-alkis-buildings-download = lejupielādē ALKIS ēkas
progress-alkis-buildings = ALKIS ēkas: { $count }
warning-alkis-no-buildings = kartes apgabalā netika atrastas ALKIS ēkas
progress-alkis-skipped = izlaisti ALKIS objekti: { $count }: { $list }
warning-alkis-buildings-changed = ALKIS ēkas avotā mainījušās ({ $changes })
progress-alkis-no-land-use-stored = nav saglabātas ALKIS zemes izmantošanas; aizliegto zonu kandidātiem palaidiet `freemapper fetch` tiešsaistē
progress-alkis-land-use-download = lejupielādē ALKIS zemes izmantošanu
progress-alkis-land-use-parts = ALKIS zemes izmantošanas daļas: { $count }
warning-alkis-land-use-kept = ALKIS zemes izmantošana nav pieejama ({ $error }); tiek atstāta saglabātā zemes izmantošana
warning-alkis-land-use-removed = ALKIS zemes izmantošana nav pieejama ({ $error }); saglabātā zemes izmantošana kartes apgabalu vairs neaptver, tāpēc freemapper to noņēma
warning-alkis-land-use-none = ALKIS zemes izmantošana nav pieejama ({ $error }); nav aizliegto zonu kandidātu

## The Berlin layers

progress-berlin-none-stored = nav saglabātu Berlīnes ALKIS struktūru; kāpnēm, mūriem, kokiem un nojumēm palaidiet `freemapper fetch` tiešsaistē
progress-berlin-removing = noņem saglabātos Berlīnes slāņus: kartes apgabals vairs nav pie Berlīnes
progress-berlin-download = lejupielādē Berlīnes ALKIS struktūras, koku kadastru, ielu uzmērījumu un punktu slāņus
progress-berlin-features = Berlīnes { $file } objekti: { $count }
warning-berlin-layer-kept = Berlīnes { $file } nav pieejams ({ $detail }); tiek atstāts saglabātais slānis
warning-berlin-layer-skipped = Berlīnes { $file } nav pieejams ({ $detail }); slānis izlaists
progress-feinkartierung-none-stored = nav saglabātas Hamburgas Feinkartierung; margām, mūriem, kāpnēm un ielu aprīkojumam palaidiet `freemapper fetch` tiešsaistē
progress-feinkartierung-removing = noņem saglabāto Hamburgas Feinkartierung: kartes apgabals vairs nav Hamburgā
progress-feinkartierung-download = lejupielādē Hamburgas Feinkartierung (ielu telpa un publiskā zaļā zona)
progress-feinkartierung-features = Feinkartierung { $file } objekti: { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file } nav pieejams ({ $detail }); tiek atstāts saglabātais slānis
warning-feinkartierung-layer-skipped = Feinkartierung { $file } nav pieejams ({ $detail }); slānis izlaists
error-extra-layer-unknown = nav { $folder } slānis
warning-berlin-buildings-missing = šim apgabalam Berlīnē nav saglabātu ALKIS ēku; palaidiet `freemapper fetch`, lai tās iegūtu no Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = nav saglabātu pilno ALKIS struktūru; mūriem, žogiem, dzīvžogiem un tiltiem palaidiet `freemapper fetch` tiešsaistē
progress-nas-removing = noņem saglabātos pilnos ALKIS slāņus: kartes apgabals vairs nav federālajā zemē ar NAS pakalpojumu
warning-nas-folder-outside = pilnie ALKIS slāņi izlaisti ({ $folder } ved ārpus projekta mapes)
progress-nas-download = lejupielādē pilnās ALKIS struktūras (NAS)
progress-nas-features = ALKIS NAS { $file } objekti: { $count }
progress-nas-skipped = izlaisti ALKIS NAS { $file } objekti: { $count }: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } nav pieejams ({ $detail }); tiek atstāts saglabātais slānis
warning-nas-layer-skipped = ALKIS NAS { $file } nav pieejams ({ $detail }); slānis izlaists

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = nav saglabātu ATKIS Basis-DLM objektu; avotiem, alām, torņiem, pieminekļiem un strūklakām palaidiet `freemapper fetch` tiešsaistē
progress-dlm-removing = noņem saglabātos ATKIS Basis-DLM slāņus: kartes apgabals vairs nav federālajā zemē, kas to nodrošina
warning-dlm-folder-outside = ATKIS Basis-DLM slāņi izlaisti ({ $folder } ved ārpus projekta mapes)
progress-dlm-download = lejupielādē ATKIS Basis-DLM objektus
progress-dlm-features = ATKIS Basis-DLM { $file } objekti: { $count }
progress-dlm-skipped = izlaisti ATKIS Basis-DLM { $file } objekti: { $count }: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } nav pieejams ({ $detail }); tiek atstāts saglabātais slānis
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } nav pieejams ({ $detail }); slānis izlaists

## Tree cadastres

progress-trees-none-stored = nav saglabāta pilsētas koku kadastra; atsevišķiem kokiem palaidiet `freemapper fetch` tiešsaistē
progress-trees-removing = noņem saglabātos koku kadastrus: kartes apgabals vairs nav pilsētā, kurā tāds ir
warning-trees-folder-outside = pilsētu koku kadastri izlaisti ({ $folder } ved ārpus projekta mapes)
progress-trees-download = lejupielādē pilsētu koku kadastrus
progress-trees-count = { $id } koki: { $count }
progress-trees-skipped = izlaisti { $id } koki: { $count }: { $list }
warning-trees-kept = { $id } koku kadastrs nav pieejams ({ $detail }); tiek atstāts saglabātais
warning-trees-skipped = { $id } koku kadastrs nav pieejams ({ $detail }); koki izlaisti

## Deriving the templates

progress-writing = raksta { $path }
progress-contours-steep = horizontāles ik pēc { $interval } m: apvidus ir tik stāvs, ka horizontāles ik pēc { $default } m saplūstu; horizontāļu intervāls { $default } m projektā tās saglabā (`--contour-interval { $default }`)
warning-osm-only-no-terrain = šim reģionam vēl nav reljefa datu: karte tiek veidota tikai no OpenStreetMap, bez horizontālēm, veidnēm un ēkām
progress-no-ndsm-recorded = nDSM nav reģistrēts; nDSM veidnei palaidiet `freemapper fetch`
warning-ndsm-tile-invalid = nederīga nDSM flīze { $tile } failā freemapper.toml; izlaista
warning-ndsm-not-cached = nDSM { $tile } nav kešatmiņā; palaidiet `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } nav nolasāms ({ $error }); dzēsiet { $path } un palaidiet `freemapper fetch`
warning-surface-unusable = virsmas modelis nav lietojams ({ $error }); palaidiet `freemapper fetch`

## Downloads and the cache

progress-downloading = lejupielādē { $url }
progress-downloaded = lejupielādēts { $megabytes } MB
progress-downloaded-of = lejupielādēts { $megabytes } MB no { $total } MB
progress-tile-index-download = lejupielādē { $product } flīžu indeksu
progress-tile-cached = kešatmiņā: { $tile }
progress-surface-blocks = nolasa virsmas modeļa flīzes { $tile } blokus: { $count } no { $total } ({ $megabytes } MB)
progress-surface-whole = virsmas modeļa flīze { $tile } tiek lejupielādēta pilnībā: { $reason }
warning-tile-cached-older = tiek izmantota kešotā { $product } flīze { $tile } no { $published } (jaunāka { $newer } nav lejupielādēta)
warning-ndsm-no-cached = flīzei { $tile } nav kešota nDSM
warning-ndsm-tile-unavailable-cached = nDSM flīze { $tile } nav pieejama ({ $reason }); tiek izmantots kešotais fails
warning-ndsm-tile-unavailable-skipped = nDSM flīze { $tile } nav pieejama ({ $reason }); izlaista
warning-file-check-failed = nevarēja pārbaudīt, vai { $file } vēl ir aktuāls ({ $reason }); tiek izmantots kešotais fails un šī servera pārējie kešotie faili bez vaicāšanas
warning-ldbv-no-buildings-file = serverī nav ēku faila flīzei { $tile }; kartē tur var trūkt ēku
progress-ndsm-not-published = flīzei { $tile } nav publicēts nDSM punktu mākonis
progress-ndsm-cached = kešatmiņā nDSM { $tile }
progress-point-cloud-not-published = flīzei { $tile } nav publicēts lāzera punktu mākonis
progress-point-cloud-cached = kešatmiņā punktu mākonis { $tile }
warning-point-cloud-no-cached = flīzei { $tile } nav kešota punktu mākoņa; tur nav veģetācijas kandidātu
warning-point-cloud-unavailable-cached = punktu mākonis { $tile } nav pieejams ({ $reason }); tiek izmantots kešotais fails
warning-point-cloud-unavailable-skipped = punktu mākonis { $tile } nav pieejams ({ $reason }); tur nav veģetācijas kandidātu
warning-point-clouds-unavailable = lāzera punktu mākoņi nav pieejami ({ $reason }); veģetācijas kandidāti izmanto kešotās flīzes, ja tādas ir
progress-no-point-clouds = šeit vēl nav lāzera punktu mākoņu veģetācijai: federālā zeme nepublicē tādus, ko freemapper var nolasīt, vai ne kartes koordinātu sistēmā
warning-point-clouds-not-fetched = veģetācija ir izvēlēta, bet punktu mākoņi nav ielādēti; palaidiet `freemapper fetch`, lai tos ielādētu
warning-point-cloud-unreadable = punktu mākonis { $tile } nav nolasāms ({ $error }); dzēsiet { $path } un palaidiet `freemapper fetch`
warning-osm-nrw-nearest = 100 m attālumā no apgabala nav Ziemeļreinas-Vestfālenes apriņķa izvilkuma; tiek nolasīts tuvākais, { $district }
warning-osm-disk-full-cached = disks ir pilns, tāpēc freemapper izmanto kešoto OSM izvilkumu no { $date }. Atbrīvojiet vietu un vēlreiz palaidiet `freemapper fetch`, lai iegūtu jaunāku
warning-osm-geofabrik-unavailable = Geofabrik nav pieejams ({ $error }); tiek izmantots kešotais OSM izvilkums no { $date }
progress-osm-extract-cached = kešatmiņā OSM izvilkums { $region } no { $date }
progress-osm-extract-download = lejupielādē { $url } (apmēram { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = lejupielādē bDOM flīzi { $tile } ({ $megabytes } MB, { $index } no { $total })
progress-sh-dgm-not-in-atom = DGM1 flīzes { $tile } nav INSPIRE ATOM katalogā; lejupielādē tās XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1 flīzi { $tile } no INSPIRE ATOM kataloga nevar nolasīt ({ $error }); lejupielādē tās XYZ ({ $megabytes } MB)
progress-sh-tile = flīze { $tile }: { $text }
progress-sh-download-again = flīze { $tile }: lejupielādi nevar nolasīt ({ $message }); lejupielādē vēlreiz
warning-sh-portal-no-date = Downloadportal nesniedz datumu { $model } flīzei { $tile }
warning-sh-dgm-xyz-used = DGM1 flīzes { $tile } nav INSPIRE ATOM katalogā; izmantots tās XYZ no Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1 flīzi { $tile } no INSPIRE ATOM kataloga nevar nolasīt ({ $error }); izmantots tās XYZ no Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model } flīze { $tile }: kartes apgabala šūnām bez augstuma: { $count }
warning-centre-tile-missing = kartes apgabala centrā trūkst { $model } flīzes { $tile }, tāpēc nav datuma
warning-centre-tile-unlisted = flīžu sarakstā nav { $model } flīzes { $tile } kartes apgabala centrā, tāpēc nav datuma
warning-list-unreadable-kept = { $model } flīžu sarakstu nevarēja nolasīt ({ $error }); izmantots saraksts no iepriekšējās palaišanas
warning-portal-list-unreadable-kept = Downloadportal { $model } flīžu sarakstu nevarēja nolasīt ({ $error }); izmantots saraksts no iepriekšējās palaišanas
warning-surface-much-older = { $model } datēts ar { $surface }, ortofoto ar { $flight }. nDSM var trūkt jaunāku ēku un koku.

## Rhineland-Palatinate

warning-rp-list-features-unusable = flīžu saraksta objekti bez lietojamas flīzes atslēgas: { $count }; izlaisti
warning-rp-list-unreadable-kept = LVermGeo RP flīžu sarakstu nevarēja nolasīt ({ $error }); izmantots saraksts no iepriekšējās palaišanas
warning-rp-metalink-not-kept = LVermGeo RP Metalink netika saglabāts: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP { $model } Metalink nevarēja nolasīt ({ $detail }); izmantots Metalink no iepriekšējās palaišanas
warning-rp-metalink-unreadable = LVermGeo RP { $model } Metalink nevarēja nolasīt ({ $detail }), tāpēc flīzes, kurām tā flīžu saraksts nesniedz lidojuma gadu, paliek bez augstumiem
warning-rp-tile-without-name = { $model } flīze { $tile }: šim 1 km kvadrātam kartē nav augstumu. LVermGeo RP flīžu saraksts (Laserbefliegung) šim kvadrātam { $note }, tāpēc faila nosaukums nav zināms{ $either ->
        [metalink] , un arī tā Metalink šim kvadrātam nenosauc nevienu failu
       *[none] {""}
    }
warning-rp-centre-no-date = { $model } flīzei { $tile } nav datuma: LVermGeo RP flīžu saraksts (Laserbefliegung) šai flīzei { $note }
word-rp-list-neither = nesniedz ne lidojuma gadu, ne lidojuma dienas
word-rp-list-no-days = nesniedz lidojuma dienas
word-rp-list-no-year = nesniedz lidojuma gadu
word-rp-list-days-unreadable = sniedz lidojuma dienas { $days }, ko nevar nolasīt
word-rp-list-disagree = sniedz lidojuma dienas { $days } un lidojuma gadu { $year }, kas nesaskan vienā gadā

## Hesse

warning-hvbg-undated = metadatu sarakstā nav nolasāma Bildflugdatum flīzēm { $tiles }; to projekta ierakstos nav lidojuma datuma
warning-hvbg-undated-more = metadatu sarakstā nav nolasāma Bildflugdatum flīzēm { $tiles } un vēl { $more }; to projekta ierakstos nav lidojuma datuma
warning-hvbg-list-unreadable-older = jauno HVBG metadatu sarakstu { $name } nevarēja nolasīt ({ $detail }); tiek izmantots vecākais saraksts { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: nav datuma, STAC neuzskaita nevienu flīzi kartes apgabala centrā
warning-lgln-no-date = { $model }: nav datuma ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; saglabātais modelis patur savu reģistrēto datumu
warning-mv-cells-empty = { $model }: Meklenburgas-Priekšpomerānijas flīžu šūnām bez augstuma: { $count }
warning-mv-centre-no-date = { $model }: nav datuma centra flīzei { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $model } flīžu saraksta rindām bez lietojama flīzes nosaukuma: { $count }; izlaistas
warning-nrw-list-rows-undated = { $model } flīžu saraksta rindām bez nolasāma datuma: { $count }
warning-nrw-list-older = servera { $model } flīžu saraksts ir no { $server }, vecāks par saglabāto no { $kept }; izmantots saglabātais
warning-nrw-list-no-date = flīžu saraksts nesniedz datumu { $model } flīzei { $tile }

## After `init`

note-scale-outside-norm = piezīme: 1:5000 nav sprinta normā; freemapper-print par to brīdina, un simboli paliek 100% apmērā

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } nav OSM id kā way/992423119
error-place-invalid-answer = nederīga atbilde no vietas meklēšanas: { $detail }
error-place-not-found = { $id } nav atrasts. Nominatim uzskaita tikai marķētus OSM objektus, tāpēc multipoligona nemarķēta ārējā līnija tur nav. Lietojiet multipoligona relāciju (--osm relation/<id>) vai saglabājiet kontūru kā GeoJSON un lietojiet --geojson
error-place-is-a-point = { $id } ir punkts, nevis apgabals; izvēlieties tā teritoriju (way vai relation)
error-place-no-such-part = { $id } ir { $count ->
        [zero] { $count } daļu
        [one] { $count } daļa
       *[other] { $count } daļas
    }; daļas { $part } nav
error-place-several-parts = { $id } ir apgabals ar vairākām atsevišķām daļām ({ $count }: { $sizes }). Kartes apgabals ir viens daudzstūris: izvēlieties daļu ar --part <n> (1 ir lielākā) vai saglabājiet vēlamo kontūru kā GeoJSON un lietojiet --geojson
error-layer-not-a-world-file = { $path }: nav world fails
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = { $layer } nav priekšskatījuma; slāņi ar priekšskatījumiem: { $names }
error-layer-no-ndsm-published = šī kartes apgabala flīzēm nav publicēts nDSM, tāpēc nav ndsm veidnes
error-layer-not-made = { $layer } vēl nav izveidots; palaidiet { $step ->
        [fetch] freemapper fetch vai run (MCP: start_pipeline); tikai Berlīne
        [export] freemapper export vai run (MCP: start_pipeline)
       *[derive] freemapper derive vai run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD karte no tā nesaņem fonu
warning-map-preview-failed = karte ir ierakstīta, bet tās priekšskatījuma nav: { $error }
warning-training-no-tree-cover = treniņu karte patur parku atklātās zemes veselas: trūkst koku seguma (data/tree-cover.tif); vēlreiz palaidiet `freemapper derive`
