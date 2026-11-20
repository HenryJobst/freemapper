# Komunikaty postępu i ostrzeżenia silnika (ADR 0092, etap 3). Wzorcem jest wersja angielska.
# Identyfikatory: progress- dla komunikatów postępu, warning- dla ostrzeżeń, word- dla nazw
# wewnątrz innych komunikatów. Ostrzeżenie dostaje prefiks z note-warning.

note-warning = ostrzeżenie: { $text }

warning-other-zone = obszar mapy leży w strefie UTM { $zone } (EPSG:{ $epsg }); freemapper pobiera wszystko ponownie w tej strefie
warning-other-state = obszar mapy leży teraz w danych innego kraju związkowego; freemapper pobiera wszystko ponownie
warning-beyond-coverage = część obszaru mapy leży poza obsługiwanymi krajami związkowymi. Leży za granicą, na morzu albo w kraju związkowym, którego freemapper jeszcze nie obsługuje, na przykład w Turyngii. Brak tam danych, więc ortofotomapa może być biała, a ALKIS nie poda budynków ani użytków gruntowych
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: skala 1:{ $scale } nie należy do skal dla map { $kind } ({ $allowed }); freemapper-print odrzuci mapę
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: pominięto wpis { $key } warstwy { $german ->
       *[any] { $what }
    }: { $reason ->
        [name] to nie jest zwykła nazwa pliku
       *[unknown] freemapper nie zna takiej warstwy
    }

## Disk space

warning-disk-space-unreadable = nie można odczytać wolnego miejsca na dysku dla { $path } ({ $error }), więc freemapper go nie sprawdza

## Tiles

warning-tile-changed = arkusz { $tile } produktu { $product } zmienił się u źródła ({ $old } → { $new })
warning-bdom-tile-unavailable = model pokrycia terenu (bDOM) dla arkusza { $tile } niedostępny ({ $error }); brak tam szablonu nDSM
warning-bdom-unavailable = model pokrycia terenu (bDOM) niedostępny ({ $error }); brak szablonu nDSM dla: { $tiles }
progress-ndsm-from-bdom = nDSM arkusza { $tile } z modelu pokrycia terenu (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] numeryczny model terenu
        [surface] numeryczny model pokrycia terenu
       *[orthophoto] ortofotomapa
    } ({ $name })
progress-model-download = pobieranie: { $model }
progress-model-read = odczyt: { $model }
warning-model-unavailable = { $model } niedostępny ({ $error }); brak szablonu nDSM
warning-model-unavailable-kept = { $model } niedostępny ({ $error }); zachowano model pokrycia terenu pobrany { $fetched }
warning-model-tile-unavailable = { $model }, arkusz { $tile }: niedostępny ({ $error }); brak tam szablonu nDSM
warning-archive-passed-over = { $model }: nie można odczytać archiwum ({ $error }); pominięto je, a arkusze obszaru mapy pochodzą z pozostałych archiwów
warning-archive-directory-kept = { $model }: nie można odczytać archiwum ({ $error }); użyto jego spisu zachowanego z wcześniejszego uruchomienia, więc arkusz, który LVGL od tego czasu wymienił, nie zostanie zauważony
warning-model-tiles-missing = { $model }: brak { $count ->
        [one] arkusza
        [few] arkuszy
        [many] arkuszy
       *[other] arkusza
    } { $names } w obrębie { $state ->
        [the-map-area] obszaru mapy
        [hamburg] Hamburga
        [hesse] Hesji
        [north-rhine-westphalia] Nadrenii Północnej-Westfalii
        [schleswig-holstein] Szlezwiku-Holsztynu
        [mecklenburg-vorpommern] Meklemburgii-Pomorza Przedniego
        [lower-saxony] Dolnej Saksonii
        [bavaria] Bawarii
       *[other] { $state }
    }; szablony pozostają tam puste
warning-no-surface-stored = brak zapisanego modelu pokrycia terenu; uruchom `freemapper fetch` online, aby dostać szablon nDSM
warning-surface-older-than-orthophoto = model pokrycia terenu (bDOM { $surface }) jest starszy niż ortofotomapa ({ $photo }); w nDSM mogą brakować nowych budynków
progress-surface-kept = freemapper zachowuje model pokrycia terenu pobrany { $fetched } dla szablonu nDSM
progress-surface-tiles-kept = freemapper zachowuje z pamięci podręcznej model pokrycia terenu dla { $count ->
        [one] arkusza
        [few] arkuszy
        [many] arkuszy
       *[other] arkusza
    } { $names } dla szablonu nDSM
progress-orthophoto-unchanged = ortofotomapa bez zmian
progress-orthophoto-unchanged-dated = ortofotomapa bez zmian ({ $label } { $date })
progress-orthophoto-download = pobieranie ortofotomapy z { $provider } { $service }
progress-orthophoto-in-leaf = ortofotomapę wykonano { $date }, gdy drzewa miały liście; ich korony zasłaniają to, co pod nimi
progress-orthophoto-bare = ortofotomapę wykonano { $date }, gdy drzewa były bez liści
warning-lgln-orthophoto-kept = ortofotomapa (LGLN WMS): { $reason }; zachowano zapisaną ortofotomapę i datę jej wykonania { $date }
warning-lgln-orthophoto-undated = ortofotomapa (LGLN WMS): brak daty wykonania, { $reason }
word-stac-no-image = STAC nie podaje obrazu w środku obszaru mapy
word-stac-no-answer = wyszukiwanie w STAC nie dało użytecznej odpowiedzi ({ $error })
warning-downloadcenter-no-file = Downloadcenter nie ma pliku dla: { $places }
warning-downloadcenter-no-file-empty = Downloadcenter nie ma pliku dla: { $places }; { $count ->
        [one] arkusz { $names } pozostał pusty
        [few] arkusze { $names } pozostały puste
        [many] arkuszy { $names } pozostało pustych
       *[other] arkusza { $names } pozostało pustych
    }
progress-k-five-unchanged = szablon K5 bez zmian
progress-k-five-download = pobieranie mapy K5 (1:5000) z WMS Geoportalu Berlina
warning-k-five-unavailable = szablon mapy K5 niedostępny ({ $detail }); pominięto

## The state border

progress-beyond-border = obszar mapy sięga poza granicę kraju związkowego; za nią szablony są puste tam, gdzie kończą się ortofotomapa i modele wysokościowe { $provider }. ALKIS i OSM pochodzą z obu krajów związkowych
progress-beyond-border-no-data = obszar mapy sięga poza granicę kraju związkowego; brak danych { $provider } dla: { $tiles }, więc szablony pozostają tam puste

## OSM

warning-osm-extract-missing-kept = brak wyciągu OSM w pamięci podręcznej; zachowuję zapisanych kandydatów OSM z { $date }
progress-osm-extract-read = odczyt wyciągu OSM { $region } z { $date }
progress-osm-candidates = kandydaci OSM: { $count }

## Permit notes

progress-permits-fetch = pobieranie danych { $source } dla wskazówek dotyczących zgód
progress-permits-osm = odczyt wyciągu OSM { $region } z { $date } dla wskazówek dotyczących zgód
progress-permits-stored = odczyt zapisanych danych wskazówek dotyczących zgód
warning-permits-source-unreachable = nie można odczytać { $source }, więc wskazówki dotyczące zgód go pomijają: { $error }
warning-permits-osm-no-extract = dla tego obszaru mapy brak wyciągu OSM w pamięci podręcznej, więc wskazówki dotyczące zgód pomijają OSM; `freemapper fetch` go pobiera

## ALKIS

warning-alkis-coverage-unknown = zasięg budynków ALKIS nieznany; uruchom `freemapper fetch` online, aby go zapisać
progress-alkis-buildings-download = pobieranie budynków ALKIS
progress-alkis-buildings = budynki ALKIS: { $count }
warning-alkis-no-buildings = w obszarze mapy nie znaleziono budynków ALKIS
progress-alkis-skipped = pominięto obiekty ALKIS: { $count }, { $list }
warning-alkis-buildings-changed = budynki ALKIS zmieniły się u źródła ({ $changes })
progress-alkis-no-land-use-stored = brak zapisanych użytków gruntowych ALKIS; uruchom `freemapper fetch` online, aby dostać kandydatów na tereny zakazane
progress-alkis-land-use-download = pobieranie użytków gruntowych ALKIS
progress-alkis-land-use-parts = części użytków gruntowych ALKIS: { $count }
warning-alkis-land-use-kept = użytki gruntowe ALKIS niedostępne ({ $error }); zachowuję zapisane użytki
warning-alkis-land-use-removed = użytki gruntowe ALKIS niedostępne ({ $error }); zapisane użytki nie pokrywają już obszaru mapy, więc freemapper je usunął
warning-alkis-land-use-none = użytki gruntowe ALKIS niedostępne ({ $error }); brak kandydatów na tereny zakazane

## The Berlin layers

progress-berlin-none-stored = brak zapisanych struktur ALKIS Berlina; uruchom `freemapper fetch` online, aby dostać schody, mury, drzewa i korony drzew
progress-berlin-removing = usuwam zapisane warstwy Berlina: obszar mapy nie leży już w pobliżu Berlina
progress-berlin-download = pobieranie struktur ALKIS Berlina, katastru drzew, pomiaru ulic i warstw punktowych
progress-berlin-features = obiekty Berlina { $file }: { $count }
warning-berlin-layer-kept = Berlin { $file } niedostępne ({ $detail }); zachowuję zapisaną warstwę
warning-berlin-layer-skipped = Berlin { $file } niedostępne ({ $detail }); pominięto warstwę
progress-feinkartierung-none-stored = brak zapisanej Feinkartierung Hamburga; uruchom `freemapper fetch` online, aby dostać balustrady, mury, schody i wyposażenie ulic
progress-feinkartierung-removing = usuwam zapisaną Feinkartierung Hamburga: obszar mapy nie leży już w Hamburgu
progress-feinkartierung-download = pobieranie Feinkartierung Hamburga (przestrzeń uliczna i zieleń publiczna)
progress-feinkartierung-features = obiekty Feinkartierung { $file }: { $count }
warning-feinkartierung-layer-kept = Feinkartierung { $file } niedostępne ({ $detail }); zachowuję zapisaną warstwę
warning-feinkartierung-layer-skipped = Feinkartierung { $file } niedostępne ({ $detail }); pominięto warstwę
error-extra-layer-unknown = to nie jest warstwa { $folder }
warning-berlin-buildings-missing = brak zapisanych budynków ALKIS dla tego obszaru w Berlinie; uruchom `freemapper fetch`, aby pobrać je z Geoportalu Berlina

## The full ALKIS layers (NAS)

progress-nas-none-stored = brak zapisanych pełnych struktur ALKIS; uruchom `freemapper fetch` online, aby dostać mury, ogrodzenia, żywopłoty i mosty
progress-nas-removing = usuwam zapisane pełne warstwy ALKIS: obszar mapy nie leży już w kraju związkowym z usługą NAS
warning-nas-folder-outside = pominięto pełne warstwy ALKIS ({ $folder } wychodzi poza folder projektu)
progress-nas-download = pobieranie pełnych struktur ALKIS (NAS)
progress-nas-features = obiekty ALKIS NAS { $file }: { $count }
progress-nas-skipped = pominięto obiekty ALKIS NAS { $file }: { $count }, { $list }
warning-nas-layer-kept = ALKIS NAS { $file } niedostępne ({ $detail }); zachowuję zapisaną warstwę
warning-nas-layer-skipped = ALKIS NAS { $file } niedostępne ({ $detail }); pominięto warstwę

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = brak zapisanych obiektów ATKIS Basis-DLM; uruchom `freemapper fetch` online, aby dostać źródła, jaskinie, wieże, pomniki i fontanny
progress-dlm-removing = usuwam zapisane warstwy ATKIS Basis-DLM: obszar mapy nie leży już w kraju związkowym, który je udostępnia
warning-dlm-folder-outside = pominięto warstwy ATKIS Basis-DLM ({ $folder } wychodzi poza folder projektu)
progress-dlm-download = pobieranie obiektów ATKIS Basis-DLM
progress-dlm-features = obiekty ATKIS Basis-DLM { $file }: { $count }
progress-dlm-skipped = pominięto obiekty ATKIS Basis-DLM { $file }: { $count }, { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } niedostępne ({ $detail }); zachowuję zapisaną warstwę
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } niedostępne ({ $detail }); pominięto warstwę

## Tree cadastres

progress-trees-none-stored = brak zapisanego miejskiego katastru drzew; uruchom `freemapper fetch` online, aby dostać pojedyncze drzewa
progress-trees-removing = usuwam zapisane katastry drzew: obszar mapy nie leży już w mieście, które je ma
warning-trees-folder-outside = pominięto miejskie katastry drzew ({ $folder } wychodzi poza folder projektu)
progress-trees-download = pobieranie miejskich katastrów drzew
progress-trees-count = drzewa { $id }: { $count }
progress-trees-skipped = pominięto drzewa { $id }: { $count }, { $list }
warning-trees-kept = kataster drzew { $id } niedostępny ({ $detail }); zachowuję zapisany
warning-trees-skipped = kataster drzew { $id } niedostępny ({ $detail }); pominięto drzewa

## Deriving the templates

progress-writing = zapisywanie { $path }
progress-contours-steep = warstwice co { $interval } m: teren jest tak stromy, że warstwice co { $default } m zlałyby się ze sobą; cięcie warstwic { $default } m w projekcie je zachowuje (`--contour-interval { $default }`)
warning-osm-only-no-terrain = brak jeszcze danych terenu dla tego regionu: mapa powstaje tylko z OpenStreetMap, bez warstwic, szablonów i budynków
progress-no-ndsm-recorded = brak zapisanego nDSM; uruchom `freemapper fetch`, aby dostać szablon nDSM
warning-ndsm-tile-invalid = nieprawidłowy arkusz nDSM { $tile } w freemapper.toml; pominięto
warning-ndsm-not-cached = nDSM { $tile } nie ma w pamięci podręcznej; uruchom `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } nie daje się odczytać ({ $error }); usuń { $path } i uruchom `freemapper fetch`
warning-surface-unusable = model pokrycia terenu nie nadaje się do użycia ({ $error }); uruchom `freemapper fetch`

## Downloads and the cache

progress-downloading = pobieranie { $url }
progress-downloaded = pobrano { $megabytes } MB
progress-downloaded-of = pobrano { $megabytes } z { $total } MB
progress-tile-index-download = pobieranie indeksu arkuszy { $product }
progress-tile-cached = w pamięci podręcznej: { $tile }
progress-surface-blocks = odczyt { $count } z { $total } bloków arkusza { $tile } modelu pokrycia terenu ({ $megabytes } MB)
progress-surface-whole = arkusz { $tile } modelu pokrycia terenu pobrano w całości: { $reason }
warning-tile-cached-older = użyto arkusza { $tile } produktu { $product } z pamięci podręcznej z { $published } (nowszego z { $newer } nie pobrano)
warning-ndsm-no-cached = brak nDSM w pamięci podręcznej dla arkusza { $tile }
warning-ndsm-tile-unavailable-cached = arkusz nDSM { $tile } niedostępny ({ $reason }); użyto pliku z pamięci podręcznej
warning-ndsm-tile-unavailable-skipped = arkusz nDSM { $tile } niedostępny ({ $reason }); pominięto
warning-file-check-failed = nie można sprawdzić, czy { $file } jest nadal aktualny ({ $reason }); użyto pliku z pamięci podręcznej oraz bez pytania pozostałych plików z pamięci podręcznej tego serwera
warning-ldbv-no-buildings-file = brak pliku budynków dla arkusza { $tile } na serwerze; na mapie mogą tam brakować budynków
progress-ndsm-not-published = brak opublikowanej chmury punktów nDSM dla arkusza { $tile }
progress-ndsm-cached = nDSM { $tile } w pamięci podręcznej
progress-point-cloud-not-published = brak opublikowanej chmury punktów lasera dla arkusza { $tile }
progress-point-cloud-cached = chmura punktów { $tile } w pamięci podręcznej
warning-point-cloud-no-cached = brak chmury punktów w pamięci podręcznej dla arkusza { $tile }; brak tam kandydatów roślinności
warning-point-cloud-unavailable-cached = chmura punktów { $tile } niedostępna ({ $reason }); użyto pliku z pamięci podręcznej
warning-point-cloud-unavailable-skipped = chmura punktów { $tile } niedostępna ({ $reason }); brak tam kandydatów roślinności
warning-point-clouds-unavailable = chmury punktów lasera niedostępne ({ $reason }); kandydaci roślinności korzystają z arkuszy z pamięci podręcznej, jeśli jakieś są
progress-no-point-clouds = brak jeszcze chmur punktów lasera dla roślinności w tym miejscu: kraj związkowy nie publikuje żadnych, które freemapper potrafi odczytać, albo nie w układzie współrzędnych mapy
warning-point-clouds-not-fetched = wybrano roślinność, ale nie wczytano chmur punktów; uruchom `freemapper fetch`, aby je wczytać
warning-point-cloud-unreadable = chmura punktów { $tile } nie daje się odczytać ({ $error }); usuń { $path } i uruchom `freemapper fetch`
warning-osm-nrw-nearest = brak wyciągu powiatu Nadrenii Północnej-Westfalii w promieniu 100 m od obszaru; odczytuję najbliższy, { $district }
warning-osm-disk-full-cached = dysk jest pełny, więc freemapper używa wyciągu OSM z pamięci podręcznej z { $date }. Zwolnij miejsce i uruchom `freemapper fetch` ponownie, aby dostać nowszy
warning-osm-geofabrik-unavailable = Geofabrik niedostępny ({ $error }); używam wyciągu OSM z pamięci podręcznej z { $date }
progress-osm-extract-cached = wyciąg OSM { $region } z { $date } w pamięci podręcznej
progress-osm-extract-download = pobieranie { $url } (około { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = pobieranie arkusza bDOM { $tile } ({ $megabytes } MB, { $index } z { $total })
progress-sh-dgm-not-in-atom = arkusza DGM1 { $tile } nie ma w katalogu INSPIRE ATOM; pobieram jego XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = arkusz DGM1 { $tile } z katalogu INSPIRE ATOM nie daje się odczytać ({ $error }); pobieram jego XYZ ({ $megabytes } MB)
progress-sh-tile = arkusz { $tile }: { $text }
progress-sh-download-again = arkusz { $tile }: pobrany plik nie daje się odczytać ({ $message }); pobieram go jeszcze raz
warning-sh-portal-no-date = Downloadportal nie podaje daty dla arkusza { $tile } ({ $model })
warning-sh-dgm-xyz-used = arkusza DGM1 { $tile } nie ma w katalogu INSPIRE ATOM; użyto jego XYZ z Downloadportal
warning-sh-dgm-xyz-used-unreadable = arkusz DGM1 { $tile } z katalogu INSPIRE ATOM nie daje się odczytać ({ $error }); użyto jego XYZ z Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, arkusz { $tile }: komórki obszaru mapy bez wysokości: { $count }
warning-centre-tile-missing = brak arkusza { $tile } ({ $model }) w środku obszaru mapy, więc nie ma daty
warning-centre-tile-unlisted = lista arkuszy nie wymienia arkusza { $tile } ({ $model }) w środku obszaru mapy, więc nie ma daty
warning-list-unreadable-kept = nie można odczytać listy arkuszy ({ $model }) ({ $error }); użyto listy zachowanej z wcześniejszego uruchomienia
warning-portal-list-unreadable-kept = nie można odczytać listy arkuszy Downloadportal ({ $model }) ({ $error }); użyto listy zachowanej z wcześniejszego uruchomienia
warning-surface-much-older = { $model } pochodzi z { $surface }, ortofotomapa z { $flight }. W nDSM mogą brakować nowszych budynków i drzew.

## Rhineland-Palatinate

warning-rp-list-features-unusable = obiekty listy arkuszy bez użytecznego klucza arkusza: { $count }; pominięto
warning-rp-list-unreadable-kept = nie można odczytać listy arkuszy LVermGeo RP ({ $error }); użyto listy zachowanej z wcześniejszego uruchomienia
warning-rp-metalink-not-kept = nie zachowano Metalinku LVermGeo RP: { $error }
warning-rp-metalink-unreadable-kept = nie można odczytać Metalinku LVermGeo RP dla { $model } ({ $detail }); użyto Metalinku zachowanego z wcześniejszego uruchomienia
warning-rp-metalink-unreadable = nie można odczytać Metalinku LVermGeo RP dla { $model } ({ $detail }), więc arkusze, dla których jego lista nie podaje roku nalotu, pozostają bez wysokości
warning-rp-tile-without-name = { $model }, arkusz { $tile }: ten kwadrat 1 km nie ma na mapie wysokości. Lista arkuszy LVermGeo RP (Laserbefliegung) { $note }, więc nazwa pliku nie jest znana{ $either ->
        [metalink] , a jego Metalink też nie podaje dla niego pliku
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }, arkusz { $tile }, nie ma daty: lista arkuszy LVermGeo RP (Laserbefliegung) { $note }
word-rp-list-neither = nie podaje ani roku nalotu, ani dni nalotu
word-rp-list-no-days = nie podaje dni nalotu
word-rp-list-no-year = nie podaje roku nalotu
word-rp-list-days-unreadable = podaje dni nalotu { $days }, których nie można odczytać
word-rp-list-disagree = podaje dni nalotu { $days } i rok nalotu { $year }, które nie zgadzają się co do jednego roku

## Hesse

warning-hvbg-undated = lista metadanych nie ma czytelnego Bildflugdatum dla arkusza { $tiles }; ich rekordy projektu nie mają daty nalotu
warning-hvbg-undated-more = lista metadanych nie ma czytelnego Bildflugdatum dla arkusza { $tiles } i jeszcze { $more }; ich rekordy projektu nie mają daty nalotu
warning-hvbg-list-unreadable-older = nie można odczytać nowej listy metadanych HVBG { $name } ({ $detail }); używam starszej listy { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: brak daty, STAC nie podaje arkusza w środku obszaru mapy
warning-lgln-no-date = { $model }: brak daty ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; zapisany model zachowuje swoją zapisaną datę
warning-mv-cells-empty = { $model }: komórki bez wysokości w arkuszach Meklemburgii-Pomorza Przedniego: { $count }
warning-mv-centre-no-date = { $model }: brak daty dla arkusza środkowego { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = wiersze listy arkuszy ({ $model }) bez użytecznej nazwy arkusza: { $count }; pominięto
warning-nrw-list-rows-undated = wiersze listy arkuszy ({ $model }) bez czytelnej daty: { $count }
warning-nrw-list-older = lista arkuszy serwera dla { $model } pochodzi z { $server }, jest starsza niż zachowana z { $kept }; użyto zachowanej
warning-nrw-list-no-date = lista arkuszy nie podaje daty dla arkusza { $tile } ({ $model })

## After `init`

note-scale-outside-norm = uwaga: 1:5000 nie należy do normy sprintowej; freemapper-print ostrzega o tym, a znaki pozostają na 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } nie jest identyfikatorem OSM w rodzaju way/992423119
error-place-invalid-answer = nieprawidłowa odpowiedź wyszukiwarki miejsc: { $detail }
error-place-not-found = nie znaleziono { $id }. Nominatim wymienia tylko obiekty OSM z tagami, więc zewnętrznej linii multipoligonu bez tagów tam nie ma. Użyj zamiast niej relacji multipoligonu (--osm relation/<id>) albo zapisz obrys jako GeoJSON i użyj --geojson
error-place-is-a-point = { $id } jest punktem, a nie obszarem; wybierz jego teren (way lub relation)
error-place-no-such-part = { $id } ma { $count ->
        [one] 1 część
        [few] { $count } części
        [many] { $count } części
       *[other] { $count } części
    }; nie ma części { $part }
error-place-several-parts = { $id } to obszar z { $count } oddzielnych części ({ $sizes }). Obszar mapy to jeden wielokąt: wybierz część przez --part <n> (1 jest największa) albo zapisz potrzebny obrys jako GeoJSON i użyj --geojson
error-layer-not-a-world-file = { $path }: to nie jest plik georeferencji (world file)
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = brak podglądu dla { $layer }; warstwy z podglądem: { $names }
error-layer-no-ndsm-published = dla arkuszy tego obszaru mapy nie opublikowano nDSM, więc nie ma szablonu ndsm
error-layer-not-made = { $layer } nie jest jeszcze utworzona; uruchom { $step ->
        [fetch] freemapper fetch lub run (MCP: start_pipeline); tylko Berlin
        [export] freemapper export lub run (MCP: start_pipeline)
       *[derive] freemapper derive lub run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; mapa OCAD nie dostaje z tego tła
warning-map-preview-failed = mapa jest zapisana, ale brakuje jej podglądu: { $error }
warning-training-no-tree-cover = mapa treningowa zachowuje otwarte tereny parków w całości: brakuje pokrycia drzewami (data/tree-cover.tif); uruchom `freemapper derive` ponownie
