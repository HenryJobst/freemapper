# Teksty programu freemapper, polski (ADR 0092). Wzorcem jest angielski: każdy inny język
# ma te same identyfikatory komunikatów i te same zmienne. Teksty z innych programów (błędy
# systemu operacyjnego, komunikaty parsera) trafiają w zmiennych i zostają bez zmian.

## Przełącznik --lang

cli-lang-unknown = { $value } nie jest językiem, dla którego freemapper ma teksty; dostępne: { $available }

## init

cli-part-needs-osm = --part wybiera część obszaru OSM i wymaga --osm; przy --place najpierw znajdź identyfikator i użyj --osm <id> --part <n>
cli-reading-file = odczyt { $path }
cli-no-area-for-place = nie znaleziono obszaru dla "{ $query }". Jeśli miejsce składa się z kilku oddzielnych części, wybierz jedną przez --osm <id> --part <n> albo użyj --geojson
cli-using-place = Używam: { $name } ({ $id })
cli-created-project = Utworzono projekt { $path }

## Pamięć podręczna

cli-no-cache-dir = brak folderu pamięci podręcznej w tym systemie; podaj --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Pobrano 1 arkusz
        [few] Pobrano { $count } arkusze
        [many] Pobrano { $count } arkuszy
       *[other] Pobrano { $count } arkusza
    }
cli-wrote = Zapisano { $path }
cli-kept-map = Zachowano { $path } (może zawierać twoją pracę)
cli-map-lacks-templates = Mapa nie odwołuje się jeszcze do szablonów rastrowych. Dodaj je w OOM (Templates > Open template…) albo zastąp mapę przez --force, co zachowuje starą jako .omap.bak.
cli-map-lacks-buildings = Mapa nie ma jeszcze budynków z ALKIS. Aby je dodać, zastąp mapę przez --force, co zachowuje starą jako .omap.bak.
cli-map-lacks-canopies = Mapie brakuje wiat i carportów z ALKIS (522), które są teraz obiektami mapy. Aby je dodać, uruchom z --force, co zachowuje starą mapę jako .omap.bak.
cli-map-light = Mapa bazowa: połączono { $joined } linii, { $left } ukrytych lub zbyt małych obiektów zostawiono w warstwach kandydatów (nic nie przesunięto)
cli-map-light-kept = Warstwy kandydatów zawierają teraz także { $left } obiektów lub odcinków, które ta wersja zdejmuje z mapy bazowej (z tagiem freemapper:generalized). Zachowana mapa wciąż je ma, więc widać je dwa razy. Zastąp mapę przez --force, aby ją oczyścić (stara zostaje jako .omap.bak).
cli-map-old-candidates = Mapa nadal wskazuje templates/candidates.omap; to uruchomienie wypełniło go wszystkimi kandydatami. Kandydaci są teraz także w siedmiu warstwach, po jednym pliku (templates/candidates-*.omap). Mapa wyświetli je po zastąpieniu przez --force (stara zostaje jako .omap.bak) albo możesz dodać je sam w OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF bez informacji o źródłach: ODbL i dl-de/by-2-0 wymagają podania źródeł przy przekazywaniu mapy dalej; zob. ATTRIBUTION.txt
cli-wrote-pdf = Zapisano { $path } (mapa bez kandydatów)
cli-wrote-pdf-candidates = Zapisano { $path } (wydruk kontrolny: mapa ze wszystkimi kandydatami, zgodnie z położeniem; sprawdź ich w terenie)
cli-wrote-training = Zapisano { $map } i { $pdf } (mapa treningowa: połączono { $joined }, przesunięto { $shifted }, pominięto lub przycięto { $left })
cli-training-free-ends = Wolne końce linii: { $ways } tam, gdzie droga spotykała inną drogę, { $others } przy innych liniach
cli-training-apart-left = mapa treningowa ma jeszcze { $length } mm (na papierze), gdzie obiekty mają zbyt mały odstęp; sprawdź to w OOM
cli-kept-training = Zachowano mapę treningową { $path }, aby uruchomienie nie nadpisało mapy, którą wydrukowano lub przekazano dalej; usuń ją albo podaj --force, aby ją odnowić

## OCAD

cli-ocad-kept = Zachowano { $path } (może zawierać twoją pracę; --force zastępuje go i zachowuje stary jako .ocd.bak)
cli-ocad-old-candidates = Mapa OCAD nadal wskazuje { $path } jako mapę w tle; to uruchomienie wypełniło ją wszystkimi kandydatami. Kandydaci są teraz także w siedmiu warstwach, po jednym pliku (-candidates-*.ocd). Mapa OCAD wyświetli je po zastąpieniu przez --force (stara zostaje jako .ocd.bak) albo możesz dodać je sam w OCAD jako mapy w tle.
cli-ocad-wrote = { $count ->
        [one] Zapisano { $path } (1 obiekt)
        [few] Zapisano { $path } ({ $count } obiekty)
        [many] Zapisano { $path } ({ $count } obiektów)
       *[other] Zapisano { $path } ({ $count } obiektu)
    }
cli-ocad-left-out = Pominięto w mapie OCAD: { $skipped } obiektów (brak znaku w OCAD, brak powierzchni lub poza obszarem rysowania OCAD ±2 m) oraz { $dropped } zdegenerowanych części linii lub otworów.
cli-ocad-load-symbols = W OCAD wybierz Symbol > Load Colors and Symbols From… i wskaż oficjalny zestaw znaków{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , zestaw szkolny IOF 2019
       *[other] {" "}ISSprOM 2019
    }; zastąpi on znaki zastępcze freemappera według numeru.

## select

cli-select-running = Wybór obszaru dla { $path }: naciśnij "Gotowe" na stronie albo Ctrl+C, aby przerwać.
cli-select-no-browser = Nie udało się otworzyć przeglądarki ({ $error }); otwórz powyższy adres.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; brak obszaru, nie może być obszarem mapy): { $place }
cli-find-hint = Utwórz projekt poleceniem: freemapper init <dir> --osm { $id }
cli-find-no-area = Żadne z tych miejsc nie ma obszaru, który mógłby być obszarem mapy. Narysuj obszar poleceniem: freemapper select <dir>
cli-find-nothing = nie znaleziono miejsca dla "{ $query }"; spróbuj innej nazwy albo narysuj obszar poleceniem: freemapper select <dir>

## adaptery (ADR 0082). Ustalenie z manifestu trafia do `{ $finding }` wraz z plikiem, wierszem, kluczem
## i kodem, po angielsku.

cli-adapter-skipped = ostrzeżenie: pominięto adapter w { $folder }: { $finding }
cli-adapter-note = ostrzeżenie: adapter w { $folder } się wczytuje, ale: { $finding }
cli-adapter-from-project = ostrzeżenie: używany jest adapter { $adapter } z folderu projektu; może odpytywać te hosty: { $hosts }
cli-adapter-overlaps = ostrzeżenie: adapter { $adapter } obejmuje też obszar adapterów { $others }; jego obiekty trafiają tam na mapy
cli-adapter-dir-relative = ostrzeżenie: FREEMAPPER_ADAPTER_DIR nie jest ścieżką bezwzględną ({ $path }); jest ignorowane
cli-adapters-no-project = ostrzeżenie: { $path } nie jest folderem; nie wczytano adapterów projektu
cli-adapters-overlaps = obejmuje wspólny obszar z: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  rewizja { $revision }  sprawdzono { $checked }  hosty: { $hosts }
cli-adapters-source-built-in = wbudowany
cli-adapters-source-user = z twojego folderu
cli-adapters-source-project = z projektu
cli-adapters-never-checked = nigdy
cli-adapters-user-dir = Twoje adaptery są wczytywane z { $path }
cli-adapters-no-user-dir = Ten system nie ma folderu na twoje adaptery; można go wskazać przez FREEMAPPER_ADAPTER_DIR

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper może teraz odpytywać także { $host } dla { $name }. Wpis jest w { $file }; `freemapper adapters disallow-host { $adapter } { $host }` go cofa. Uruchomiona strona obszaru lub asystent AI wczytuje wpis przy następnym starcie
cli-allow-host-already = { $host } jest już dozwolony dla { $name }
cli-allow-host-declared = adapter { $name } sam wymienia { $host }; nie ma czego zezwalać
cli-allow-host-refused = nie można zezwolić na { $host }: { $reason }
cli-allow-host-unknown-adapter = nie ma wbudowanego adaptera { $adapter }; wbudowane adaptery to: { $adapters }
cli-allow-host-folder-adapter = { $adapter } jest adapterem z twojego folderu adapterów; dodaj host do `[net] hosts` jego manifestu
cli-allow-host-no-config = ten system nie ma folderu konfiguracji do przechowywania dozwolonych hostów
cli-allow-host-write = nie udało się zapisać { $file }: { $detail }
cli-allow-host-unreadable = nie można odczytać { $file } ({ $detail }); popraw lub usuń plik; nic nie zmieniono
cli-disallow-host-removed = freemapper nie odpytuje już { $host } dla { $name }
cli-disallow-host-absent = { $host } nie był dozwolony dla { $name }; nie ma czego cofać
cli-allowed-hosts-unreadable = ostrzeżenie: nie udało się odczytać { $file } ({ $detail }); żadne dodatkowe hosty nie są dozwolone
cli-allowed-hosts-ignored = ostrzeżenie: { $file }: wpis { $host } dla { $adapter } jest ignorowany: { $reason }
cli-allowed-hosts-no-adapter = freemapper nie ma wbudowanego adaptera o tej nazwie
cli-allowed-host-redundant = ostrzeżenie: { $host } jest teraz jednym z hostów wymienianych przez adapter { $name }; twój wpis w { $file } usunięto
cli-adapters-allowed = dozwolone przez ciebie: { $hosts }

## adapters check. Ustalenie trafia do `{ $finding }` jak w wierszach powyżej. `{ $capability }` to
## stałe słowo (`rasters`, `alkis-buildings`, `nas`, `osm`), takie samo w każdym języku.

cli-check-adapter = adapter { $adapter } ({ $name }), rewizja { $revision }, hosty: { $hosts }
cli-check-error = błąd: { $finding }
cli-check-warning = ostrzeżenie: { $finding }
cli-check-passed = { $folder } przechodzi kontrolę
cli-check-passed-warnings = { $folder } przechodzi kontrolę, z { $count ->
        [one] jednym ostrzeżeniem
        [few] { $count } ostrzeżeniami
        [many] { $count } ostrzeżeniami
       *[other] { $count } ostrzeżenia
    }
cli-check-failed = { $folder } nie przechodzi kontroli
cli-check-no-folder = nie ma folderu { $folder }
cli-check-live-ok = { $capability } w porządku: { $detail }
cli-check-live-failed = { $capability } BŁĄD: { $detail }
cli-check-live-not-asked = { $capability } nie sprawdzano: { $detail }
cli-check-live-box-outside = prostokąt nie leży w granicach adaptera { $adapter }; kontrola na żywo odpytuje usługi dla miejsca, które adapter obejmuje
cli-check-live-cannot = kontrola na żywo nie może się wykonać: { $detail }
cli-check-detail-not-declared = adapter niczego nie deklaruje
cli-check-detail-stopped = wcześniejsza część się nie powiodła
cli-check-detail-see-warnings = zobacz ostrzeżenia
cli-check-detail-outside-dataset = prostokąt leży poza zbiorem danych
cli-check-detail-objects = { $count ->
        [one] jeden obiekt
        [few] { $count } obiekty
        [many] { $count } obiektów
       *[other] { $count } obiektu
    }
cli-check-detail-layers = warstw: { $layers }, obiektów: { $count }
cli-check-detail-trees = { $count ->
        [one] jedno drzewo
        [few] { $count } drzewa
        [many] { $count } drzew
       *[other] { $count } drzewa
    }
cli-check-detail-extract = { $label }, dane z { $date }
cli-check-raster-orthophoto = ortofotomapa
cli-check-raster-terrain = numeryczny model terenu
cli-check-raster-surface = numeryczny model pokrycia terenu
cli-check-raster-tiles = { $product }: { $count ->
        [one] jeden arkusz
        [few] { $count } arkusze
        [many] { $count } arkuszy
       *[other] { $count } arkusza
    }

## Listy w zdaniach

# Lista w zdaniu: ostatnia pozycja po pozostałych, które łączą przecinki.
cli-list-and = { $first } i { $last }

## Blokada folderu

cli-lock-held = inne uruchomienie freemappera używa tego folderu (od { $time }, uruchomione przez: { $who }). Poczekaj na nie albo najpierw je zatrzymaj.
cli-lock-who-cli = terminal
cli-lock-who-mcp = agent AI (MCP)
cli-lock-who-page = strona wyboru obszaru
cli-lock-who-unknown = inny program
cli-lock-failed = nie udało się zablokować folderu projektu na to uruchomienie: { $error }
cli-lock-stuck = na przeszkodzie stoi coś innego niż plik blokady i nie dało się tego usunąć

## Trasy uruchomień strony wyboru

cli-run-bad-request = Opcje uruchomienia są nieprawidłowe: { $error }
cli-run-no-project = W tym folderze nie ma jeszcze zapisanego projektu. Najpierw zapisz obszar.
cli-run-already-running = Dla tego projektu trwa już uruchomienie ({ $job }). Poczekaj na jego koniec albo najpierw je anuluj.
cli-run-disk-stops = Za mało wolnego miejsca na dysku na pobieranie: { $messages }
cli-run-busy-elsewhere = Inny projekt tego programu uruchamia teraz potok. Projekty dzielą pamięć podręczną pobranych danych, więc naraz działa tylko jedno uruchomienie. Spróbuj ponownie po jego zakończeniu.
cli-run-nothing-to-cancel = Żadne uruchomienie nie trwa, więc nie ma czego anulować.
cli-run-unknown-job = Dla tego projektu nie ma uruchomienia { $job }.
cli-status-too-fast = Zapytania o stan są zbyt częste. Pytaj najwyżej pięć razy na sekundę.
cli-open-folder-failed = Nie udało się otworzyć folderu ({ $error }).
cli-open-folder-no-opener = Ten komputer nie ma programu do otwierania folderów ({ $program }). Otwórz folder ręcznie.
cli-open-folder-missing = Folder projektu jeszcze nie istnieje. Najpierw zapisz obszar.
cli-open-folder-refused = Nazwa folderu nie jest bezpieczna do przekazania menedżerowi plików.
cli-save-running = Dla tego projektu trwa uruchomienie. Zapisz ponownie po jego zakończeniu.
cli-save-unknown-kind = Nieznany rodzaj mapy "{ $kind }". Użyj schoolyard, sprint lub forest.
cli-save-scale-too-large = Skala { $scale } jest zbyt duża.
cli-save-bad-content = Ustawienia zawartości są nieprawidłowe. Nic nie zapisano.
cli-map-image-none = Najpierw utwórz mapę.
cli-map-image-gone = Ten podgląd nie jest już dostępny.
cli-map-image-running = Dla tego projektu trwa uruchomienie. Podgląd można obliczyć po jego zakończeniu.
cli-map-image-stale = Zapisany projekt nie zgadza się już z wygenerowaną mapą, na przykład z powodu zmiany obszaru, rodzaju mapy lub skali. Najpierw rozpocznij uruchomienie, które zastąpi mapę ("Zastąp istniejącą mapę" na stronie albo freemapper run --force).
cli-map-image-superseded = W międzyczasie zażądano nowszego podglądu, więc ten nie został obliczony.
cli-map-image-bad-content = Ustawienia zawartości są nieprawidłowe.
cli-map-image-bad-request = Żądanie podglądu jest nieprawidłowe. Wyślij obiekt JSON, w razie potrzeby z ustawieniami zawartości.

## Strona obszaru (`web/`, ADR 0096). Strona pobiera każdy komunikat `page-*` i `suite-*` przez
## `api/texts`; `{ $name }` zostaje w tekście, a strona go wypełnia. Bez selektorów: liczba ma
## dwa komunikaty, `<id>-one` i `<id>-other`. AGENTS.md, podręcznik i scripts/browser-smoke.md
## cytują te słowa.

page-title = freemapper – wybór obszaru
page-heading = Wybierz obszar mapy
page-update-available = Dostępny jest freemapper { $version }. Zaktualizuj poleceniem
page-update-notes = Informacje o wydaniu
page-folder = Folder projektu
page-pick = Wyszukaj miejsce
page-query-placeholder = np. Clara-Zetkin-Schule Strausberg
page-search-button = Szukaj
page-searching = Szukam …
page-no-results = Nic nie znaleziono.
page-point-only = tylko punkt, nie obszar
page-draw = Narysuj sam
page-rectangle = Prostokąt
page-polygon = Wielokąt
page-finish-polygon = Zamknij wielokąt
page-clear = Wyczyść
page-hint-rectangle = Kliknij na mapie dwa przeciwległe rogi.
page-hint-polygon = Klikaj kolejne rogi. Aby zamknąć wielokąt, kliknij pierwszy róg, kliknij dwukrotnie albo kliknij "Zamknij wielokąt". Esc anuluje, Backspace usuwa ostatni róg.
page-hint-idle = Wybierz miejsce z wyników wyszukiwania albo narysuj obszar sam.
page-summary = Obszar
page-source-osm = Z OpenStreetMap: { $name }
page-source-drawn = Narysowany ręcznie
page-size = Rozmiar z buforem: { $width } × { $height } m
page-tiles-one = Arkusze ({ $n }): { $names }
page-tiles-other = Arkusze ({ $n }): { $names }
page-download = Pobieranie: około { $mb } MB
page-download-none = Pobieranie: wszystko jest już w pamięci podręcznej
page-download-details = Co zostanie pobrane
page-cached = w pamięci podręcznej
page-disk = Miejsce na dysku: { $volumes }
page-disk-volume = wolne { $free }, potrzeba około { $needed }
page-cache-volume = pamięć podręczna
page-project-volume = folder projektu
page-no-room = nie mieści się na dysku
page-no-room-skip = Nie wszystko mieści się na dysku (freemapper zostawia { $reserve } wolnego miejsca). Pobieranie pomija zaznaczone części. Jeśli ich potrzebujesz, najpierw zwolnij miejsce:
page-no-room-stop = Za mało miejsca na dysku (freemapper zostawia { $reserve } wolnego miejsca). Bez zaznaczonych części nie powstanie mapa, więc pobieranie kończy się, zanim cokolwiek pobierze. Najpierw zwolnij miejsce:
page-disk-unknown = freemapper nie może odczytać wolnego miejsca na dysku ({ $error }). Pobiera bez tej kontroli.
page-too-large = Obszar jest zbyt duży: najwyżej { $max } m na bok (z buforem). Wybierz mniejszy.
page-outside = Obszar leży poza obsługiwanymi krajami związkowymi ({ $states }); freemapper nie ma jeszcze źródeł danych dla innych regionów.
page-outside-unknown = Obszar leży poza regionami obsługiwanymi przez freemappera; nie ma tam jeszcze źródeł danych.
page-adapters-failed = Nie można wyświetlić listy zdjęć lotniczych ({ $detail }). Mapa pokazuje tylko OpenStreetMap; reszta działa.
page-settings = Mapa
page-kind = Rodzaj mapy
page-schoolyard = Mapa dziedzińca szkolnego (1:1000)
page-sprint = Mapa sprintowa (1:4000)
page-forest = Mapa leśna (1:15000)
page-scale = Skala 1:
page-map-title = Tytuł
page-contour-interval = Cięcie warstwic
page-contour-auto = automatyczne
page-settings-fixed = Projekt już istnieje; rodzaj mapy, skala i tytuł są w freemapper.toml. Cięcie warstwic nadal możesz zmienić.
page-save = Zapisz
page-saving = Zapisuję …
page-saved = Zapisano w freemapper.toml.
page-zone-changed = Zapisano. Obszar leży teraz w innej strefie UTM; freemapper pobierze jego dane ponownie przy następnym uruchomieniu.
page-provider-changed = Zapisano. Wysokości i zdjęcia lotnicze nowego obszaru pochodzą od innego dostawcy danych; freemapper pobierze jego dane ponownie przy następnym uruchomieniu.
page-save-stale = freemapper.toml zmieniono w innym oknie lub innym programem od czasu, gdy ta strona go odczytała. Nic nie zapisano; strona zna teraz zapisany stan. Zapisz ponownie, aby zastąpić go obszarem pokazanym tutaj.
page-has-map = Istniejąca mapa (.omap) zostaje bez zmian; freemapper tworzy nowe szablony i kandydatów dla nowego obszaru. Opcja "Zastąp istniejącą mapę" poniżej (albo "freemapper run --force") tworzy od nowa także mapę i zachowuje starą jako .omap.bak.
page-done = Gotowe
page-source = Kod źródłowy (AGPL-3.0-or-later)
page-closed = Gotowe. Możesz zamknąć to okno.
page-next = Dalej: freemapper run "{ $dir }"
page-made-files = Utworzono: { $files }. Folder: { $dir }
page-back-to-agent = Powiedz swojemu asystentowi AI, że obszar jest zapisany.
page-nothing-saved = Nic nie zapisano.
page-failed = Błąd: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Zdjęcie lotnicze { $state } ({ $agency })

## Blok "Utwórz mapę" strony obszaru (ADR 0090)

page-content-heading = Co trafia na mapę
page-content-help = Profil decyduje, co trafia od razu na mapę, a co zostaje kandydatem do sprawdzenia. Wybór dla grupy ma pierwszeństwo przed profilem. Jest zapisywany razem z obszarem.
page-content-profile-legend = Profil
page-content-profile-safe = Bezpieczny
page-content-profile-safe-hint = Każdy obiekt zostaje tam, gdzie umieszcza go budowanie mapy: budynki i powierzchnie z katastru trafiają na mapę; mury, ogrodzenia, drzewa i wiele z OpenStreetMap (stawy, ścieżki o nieznanej nawierzchni) zostają kandydatami.
page-content-profile-balanced = Zrównoważony
page-content-profile-balanced-hint = Obiekty z katastru, rejestrów i inwentaryzacji ulic trafiają na mapę, razem z warstwicami i obszarami zakazu wstępu wzdłuż torów kolejowych. Mury, ogrodzenia, żywopłoty, skarpy, schody, wody, drzewa, pomniki, przejścia, ścieżki i koleje pochodzą też z OpenStreetMap. Wyposażenie ulic, pozostałe obszary zakazu wstępu i formy z modelu terenu zostają kandydatami.
page-content-profile-full = Pełny
page-content-profile-full-hint = Wszystko trafia na mapę, jak na wydruku kontrolnym; poza mapą zostają tylko ukryci kandydaci.
page-content-groups-legend = Według grup
page-content-groups-help = "Jak w profilu" podąża za profilem. Co to znaczy dla grupy, zależy od pochodzenia każdego obiektu.
page-content-place-profile = Jak w profilu
page-content-place-map = Mapa
page-content-place-candidates = Kandydat
page-content-place-off = Wyłączone
page-content-group-walls = Mury
page-content-group-fences = Ogrodzenia
page-content-group-hedges = Żywopłoty
page-content-group-slopes = Skarpy i urwiska
page-content-group-stairs = Schody
page-content-group-trees = Drzewa i grupy drzew
page-content-group-water-features = Woda, fontanny i źródła
page-content-group-monuments = Wieże, pomniki i linie energetyczne
page-content-group-furniture = Wyposażenie ulic i latarnie
page-content-group-passages = Przejścia
page-content-group-contours = Warstwice
page-content-group-landforms = Pagórki, zagłębienia, skarpy i urwiska z modelu terenu
page-content-group-vegetation = Roślinność z laserowych chmur punktów (Brandenburgia, Berlin, Nadrenia Północna-Westfalia, Nadrenia-Palatynat, Saara i Bawaria; wyłączona, dopóki jej nie wybierzesz; pobiera około 40 do 600 MB na km²)
page-content-group-out-of-bounds = Obszary zakazu wstępu
page-content-group-paths = Ścieżki i koleje
page-content-group-streets = Ulice i place
page-run-heading = Utwórz mapę
page-preview-heading = Podgląd mapy
page-preview-stored = Tak, jak ostatnio wygenerowano.
page-preview-live = Z ustawieniami z tej strony, jeszcze niezapisanymi.
page-preview-live-saved = Z zapisanymi ustawieniami, jeszcze niewygenerowanymi w uruchomieniu.
page-preview-computing = Obliczam …
page-preview-none = Nie wygenerowano jeszcze mapy.
page-preview-later = Ta mapa nie ma jeszcze podglądu. Pojawi się przy następnym uruchomieniu, które zapisze mapę.
page-preview-stale = Obszar, rodzaj mapy lub skala zmieniły się od wygenerowania mapy. Obraz nadal pokazuje mapę tak, jak ostatnio wygenerowano; nowy podgląd wymaga uruchomienia z opcją "Zastąp istniejącą mapę".
page-preview-area-unsaved = Obszar na tej stronie nie jest jeszcze zapisany. Obraz pokazuje mapę tak, jak ostatnio wygenerowano; nowy obszar wymaga uruchomienia z opcją "Zastąp istniejącą mapę".
page-preview-run = Tak, jak ostatnio wygenerowano. Strona odświeży obraz po zakończeniu uruchomienia.
page-preview-open = Otwórz w pełnym rozmiarze
page-preview-alt = Podgląd obiektów mapy
page-opt-ocad = Zapisz projekt startowy dla OCAD
page-help-ocad = Dla wszystkich, którzy pracują dalej w OCAD.
page-opt-pdf = PDF do druku
page-help-pdf = Tylko obiekty mapy, bez kandydatów.
page-opt-pdf-candidates = Wydruk kontrolny ze wszystkimi kandydatami
page-opt-training = Mapa treningowa
page-help-training = Mapa i wszyscy kandydaci, uporządkowani tak, by drukować je bez zmian; z OCAD także jako plik OCAD.
page-views-heading = Widoki mapy treningowej
page-view-no-paths = Bez ścieżek (mapa Ultimate)
page-view-brown-only = Tylko brąz (bieg warstwicowy)
page-view-no-north-lines = Bez linii północy
page-view-custom = Inny widok
page-help-views = Każdy zaznaczony widok to jeden dodatkowy plik obok mapy treningowej, `<name>-training-<view>.omap` i jego PDF, z OCAD także plik OCAD. Inny widok to `no:505,506` (pomiń znaki) albo `only:101,102` (zachowaj tylko te); `+` łączy widoki. Tytuł, skala i źródła zawsze zostają.
page-help-views-forest = Bez ścieżek i tylko brąz są dostępne wyłącznie dla map leśnych.
page-opt-scale-bar = Podziałka liniowa w PDF
page-help-scale-bar = Mapy niemieckie zwykle jej nie mają.
page-opt-sources = Wiersze ze źródłami w PDF
page-sources-auto = automatyczne
page-sources-full = pełne
page-sources-credit-only = tylko freemapper
page-warn-credit-only = PDF wymieniają wtedy tylko freemappera. Licencje danych (ODbL, dl-de/by-2-0) wymagają podania źródeł przy przekazywaniu mapy dalej; wtedy źródła podajesz sam. ATTRIBUTION.txt wymienia je wszystkie.
page-need-pdf = Podziałka liniowa i wiersze ze źródłami dotyczą tylko wtedy, gdy powstaje PDF.
page-opt-offline = Tylko pamięć podręczna, bez sieci
page-opt-force = Zastąp istniejące mapy
page-help-force = Mapę, a przy zaznaczonej "Mapie treningowej" także mapę treningową. Stare pliki zostają jako .omap.bak.
page-opt-permits = Wskazówki dotyczące zgód
page-help-permits = Jednostki administracyjne, obszary chronione i kogo zapytać, w permits.md. Wskazówka, nie porada prawna.
page-run-start = Utwórz mapę
page-run-again = Utwórz ponownie
page-run-cancel = Anuluj
page-run-cancelling = Anuluję …
page-warnings-heading = Ostrzeżenia
page-where-files = Pliki są w folderze projektu:
page-open-folder = Otwórz folder
page-log-label = Dziennik uruchomienia
page-force-title = Zastąpić istniejące mapy?
page-force-text = Istniejąca mapa zostanie zastąpiona, a przy zaznaczonej "Mapie treningowej" także mapa treningowa. Twoje zmiany nie przepadną: stare pliki zostają w folderze projektu jako .omap.bak. Kontynuować?
page-force-cancel = Anuluj
page-force-confirm = Zastąp
page-step-fetch = Pobieranie danych
page-step-derive = Obliczanie szablonów
page-step-map = Zapisywanie mapy
page-step-ocad = Zapisywanie projektu OCAD
page-step-pdf = Zapisywanie PDF
page-step-pdf-candidates = Zapisywanie wydruku kontrolnego
page-step-training = Zapisywanie mapy treningowej
page-step-permits = Zapisywanie wskazówek dotyczących zgód
page-step-line = Krok { $index } z { $total }: { $label }
page-starting = Uruchomienie się rozpoczyna …
page-file-kind-map = mapa
page-file-kind-template = szablon
page-file-kind-ocad = mapa OCAD
page-file-kind-ocad-candidates = kandydaci OCAD
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = wydruk kontrolny
page-file-kind-training-map = mapa treningowa
page-file-kind-training-pdf = PDF mapy treningowej
page-file-kind-training-ocad = mapa treningowa OCAD
page-file-kind-training-view-map = widok mapy treningowej
page-file-kind-training-view-pdf = PDF widoku mapy treningowej
page-file-kind-training-view-ocad = widok mapy treningowej OCAD
page-file-kind-preview = podgląd mapy
page-file-kind-permits = wskazówki dotyczące zgód
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Gotowe. Pliki są zapisane.
page-result-failed = Uruchomienie się nie powiodło: { $error }
page-result-failed-kept = To, co zapisano, zostaje w folderze projektu; nowe uruchomienie kontynuuje od tego miejsca.
page-result-cancelled = Anulowano. To, co już pobrano, zostaje w pamięci podręcznej; nowe uruchomienie kontynuuje od tego miejsca.
page-result-title-done = Mapa utworzona
page-result-title-failed = Uruchomienie się nie powiodło
page-result-title-cancelled = Uruchomienie anulowane
page-kept-map = Istniejącą mapę zostawiono bez zmian. "Zastąp istniejącą mapę" tworzy ją od nowa i zachowuje starą jako .omap.bak.
page-no-files = Nie utworzono żadnych plików.
page-opened-folder = Folder jest otwarty.
page-open-failed = { $message } Folder: { $dir }
page-why-no-area = Najpierw wybierz obszar.
page-why-checking = Podgląd jest obliczany …
page-why-too-large = Obszar jest zbyt duży.
page-why-outside = Obszar leży poza obsługiwanymi krajami związkowymi.
page-why-disk = Za mało wolnego miejsca na dysku (zob. wyżej).
page-why-running = Trwa uruchomienie.
page-err-scale-bar-no-pdf = Podziałka liniowa wymaga PDF.
page-err-sources-no-pdf = Wiersze ze źródłami wymagają PDF.
page-save-failed = Obszar nie jest zapisany; nie rozpoczęto uruchomienia.
page-poll-lost = Brak odpowiedzi serwera. freemapper próbuje dalej.
page-run-unknown = Serwer nie zna już tego uruchomienia.

## Wspólne moduły stron pakietu (core ADR 0030)

suite-banner-dismiss = Zamknij

## update (spec 3.4 do 3.6)

update-source-build = Ten freemapper został zbudowany ze źródeł. Zaktualizuj go poleceniami git pull i cargo build --release --locked.
update-current = freemapper { $version } jest aktualny.
update-available = Dostępny jest freemapper { $new } (masz { $old }), wydany { $released }.
update-details =
    Informacje o wydaniu: { $notes }
    Pobranie: { $size } MB
update-decimal-separator = ,
update-ask = Zainstalować teraz? [t/N]
update-not-installed = Niczego nie zainstalowano.
update-needs-yes = wejście nie jest terminalem, więc freemapper nie może zapytać przed instalacją; uruchom freemapper update --yes, aby zainstalować bez pytania
update-installing = pobieranie i sprawdzanie freemapper { $version }
update-done = freemapper zaktualizowano z { $old } do { $new }. Informacje o wydaniu: { $notes }
update-mcp-restart = Działający serwer MCP zachowuje starą wersję, dopóki asystent AI go nie uruchomi ponownie.
update-rolled-back = Wrócono do freemapper { $version }. Wersja, którą opuszczono, zostaje jako { $path }.
update-not-writable = nie można zapisywać w folderze programu { $path }, a freemapper nie prosi o uprawnienia administratora. Rozpakuj tam nowe wydanie ręcznie albo przenieś freemappera do folderu, w którym możesz zapisywać.
update-too-old = freemapper { $new } nie może zostać zainstalowany przez samoaktualizację wersji { $old }; zainstaluj { $needed } lub nowszą ręcznie, zob. { $notes }
update-busy = w folderze programu działa inna aktualizacja lub cofnięcie wersji (plik blokady { $path }); poczekaj na jej zakończenie i spróbuj ponownie
update-bad-signature = nie udało się zweryfikować wydania ({ $error }); niczego nie zainstalowano
update-check-bad-signature = nie udało się zweryfikować najnowszego wydania freemappera ({ $error }); niczego nie zainstalowano
update-no-manifest = najnowsze wydanie nie może zainstalować się samo (nie ma manifestu aktualizacji); pobierz je z { $page }
update-no-manifest-version = wydanie { $version } nie istnieje albo nie może zainstalować się samo (nie ma manifestu aktualizacji); zob. { $page }
update-no-asset = wydanie nie ma programu dla tego systemu ({ $platform }); niczego nie zainstalowano
update-no-previous = nie ma poprzedniej wersji, do której można wrócić (brakuje { $path })
update-failed = aktualizacja się nie powiodła: { $error }
update-mode-set = Tryb aktualizacji ustawiono na { $mode } w { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } zastępuje to ustawienie, dopóki jest ustawione.
update-mode-failed = nie udało się zapisać { $path }: { $error }
update-notice = Dostępny jest freemapper { $new } (masz { $old }). Zaktualizuj poleceniem: freemapper update
update-applies-next-start = Nowa wersja zacznie obowiązywać od następnego uruchomienia freemappera.
update-install-failed = freemapper { $new } nie został zainstalowany: { $error }
update-no-config-dir = brak folderu konfiguracji w tym systemie; ustaw FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Uwaga dla użytkownika: przy starcie serwer freemappera znalazł to w folderach adapterów. Przekaż to dalej; użytkownik poprawia folder albo go usuwa, a potem uruchamia serwer ponownie.
    { $lines }
update-mcp-note = Uwaga dla użytkownika: dostępny jest freemapper { $new } (ta wersja to { $old }). Zapytaj go, zanim wywołasz install_update.
update-mcp-needs-confirm = niczego nie zainstalowano; najpierw zapytaj użytkownika i wywołaj install_update z confirm: true, gdy się zgodzi
update-mcp-note-manual = Uwaga dla użytkownika: dostępny jest freemapper { $new } (ta wersja to { $old }), ale install_update nie może go zainstalować; zainstaluj { $needed } lub nowszą ręcznie, zob. { $notes }
update-mcp-note-bad-signature = Uwaga dla użytkownika: nie udało się zweryfikować najnowszego wydania freemappera ({ $error }); niczego nie zainstalowano. Powiedz mu o tym i nie instaluj go w inny sposób.
update-mcp-restart-now = Uruchom teraz ponownie serwer MCP freemappera; nadal działa stara wersja.


## Błędy. W wersji angielskiej każdy tekst jest równy komunikatowi typu błędu biblioteki, który
## pozostaje angielski w logach i testach. Części formułowane przez inne programy (system
## operacyjny, parsery, serwery) trafiają do zmiennych i pozostają bez zmian.

cli-error-prefix = błąd
error-cancelled = anulowano
cli-cancelled-kept = anulowano; zachowano: { $files }
error-io = { $path }: { $message }
error-io-disk-full = dysk jest pełny, więc freemapper nie mógł zapisać { $path }. Zwolnij trochę miejsca i spróbuj ponownie

## Błędy: projekt

error-project-already-exists = projekt już istnieje: { $path }
error-project-map-exists = mapa już istnieje (może zawierać twoją pracę); użyj --force, aby ją zastąpić: { $path }
error-project-invalid-scale = nieprawidłowa skala 1:{ $scale }
error-project-invalid-contour-interval = nieprawidłowe cięcie warstwic { $interval } m w freemapper.toml; musi być większe niż 0
error-project-scale-not-allowed = skala 1:{ $scale } nie jest dozwolona dla map typu { $kind }; użyj { $allowed }
error-project-scale-not-allowed-enlarge = skala 1:{ $scale } nie jest dozwolona dla map typu { $kind }; użyj { $allowed }. Rysuj w skali 1:4000, freemapper-print powiększa do 1:3000 przy drukowaniu
word-kind-schoolyard = dziedziniec szkolny
word-kind-sprint = sprint
word-kind-forest = las
error-project-toml = nieprawidłowy plik projektu: { $message }
error-project-unknown-adapter = plik projektu wskazuje adapter "{ $adapter }", którego ten freemapper nie ma; może go dostarczyć nowszy freemapper (`freemapper update`), może pochodzić z folderu adapterów użytkownika (pokazuje go `freemapper adapters list`) albo, z opcją --adapters-from-project w wierszu poleceń, z folderu `adapters` projektu (tylko dla projektu, któremu ufasz)
error-project-not-fetched = nie pobrano jeszcze arkuszy; najpierw uruchom `freemapper fetch`
error-project-ocd = eksport OCAD: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: obraz i plik georeferencji nie pasują do obszaru mapy; uruchom ponownie `freemapper derive`
error-project-tiles-outdated = obszar mapy wymaga arkuszy { $tiles }, które nie zostały pobrane; uruchom `freemapper fetch`
error-project-buildings-not-fetched = brak zapisanych budynków ALKIS dla tego obszaru mapy; najpierw uruchom `freemapper fetch` online
error-project-orthophoto-not-fetched = brak zapisanej ortofotomapy dla tego obszaru mapy; najpierw uruchom `freemapper fetch` online
error-project-terrain-not-fetched = brak zapisanego numerycznego modelu terenu dla tego obszaru mapy; najpierw uruchom `freemapper fetch` online
error-project-candidates-outdated = zapisani kandydaci z OSM nie obejmują obszaru mapy; uruchom `freemapper fetch`
error-project-stored = { $path }: { $message }; uruchom `freemapper fetch` ponownie
error-project-area-too-large = obszar mapy wraz z buforem ma { $width } × { $height } m, więcej niż { $limit } m na bok; zmniejsz obszar mapy lub buffer_m w freemapper.toml
error-project-area-crosses-country = obszar mapy leży po obu stronach granicy państwa; mapy przez granicę nie są jeszcze obsługiwane, więc narysuj obszar w obrębie jednego państwa
error-project-permits-not-for-region = wskazówki dotyczące zgód nie są jeszcze dostępne dla tego regionu: korzystają z niemieckich usług i wskazują niemieckie urzędy
error-project-outside-coverage = obszar mapy leży poza obsługiwanymi obszarami ({ $states }); freemapper ma na razie źródła danych tylko dla nich

## Błędy: miejsce na dysku

error-space-required = za mało miejsca na dysku dla: { $part }. Potrzeba { $needed }, ale w { $path } jest wolne tylko { $left } ponad rezerwę { $reserve }. Zwolnij trochę miejsca i uruchom `freemapper fetch` ponownie
error-space-skipped = freemapper pomija: { $part }, bo brakuje miejsca na dysku. Potrzeba { $needed }, ale w { $path } jest wolne tylko { $left } ponad rezerwę { $reserve }. Zwolnij trochę miejsca i uruchom `freemapper fetch` ponownie
word-part-map-data = dane mapy ALKIS i OSM
word-part-terrain = numeryczny model terenu
word-part-orthophoto = ortofotomapa
word-part-surface = numeryczny model pokrycia terenu dla szablonu nDSM
word-part-osm-update = aktualizacja przechowywanego wyciągu OSM
word-part-k-five = szablon mapy K5 Berlina
word-part-berlin-layers = struktury ALKIS Berlina, kataster drzew, pomiar ulic i warstwy punktowe
word-part-state-layers = pełne struktury ALKIS, obiekty modelu krajobrazu i katastry drzew
word-part-point-cloud = laserowe chmury punktów dla kandydatów roślinności

## Błędy: pobieranie

error-fetch-http = pobieranie { $url } nie powiodło się: { $message }
error-fetch-http-temporary = pobieranie { $url } nie powiodło się: { $message }. Usługa jest prawdopodobnie przeciążona lub w trakcie konserwacji; freemapper próbował do { $attempts } razy. Uruchom przebieg ponownie za kilka minut; to, co już pobrano, zostaje zachowane.
error-fetch-corrupt-archive = pobrany plik { $url } nie jest kompletnym archiwum arkuszy: { $message }
error-fetch-corrupt-extract = pobrany plik { $url } nie jest kompletnym wyciągiem OSM: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] arkusz { $tiles } nie jest objęty opublikowanymi arkuszami; zmniejsz obszar mapy lub buffer_m w freemapper.toml
        [few] arkusze { $tiles } nie są objęte opublikowanymi arkuszami; zmniejsz obszar mapy lub buffer_m w freemapper.toml
        [many] arkusze { $tiles } nie są objęte opublikowanymi arkuszami; zmniejsz obszar mapy lub buffer_m w freemapper.toml
       *[other] arkusze { $tiles } nie są objęte opublikowanymi arkuszami; zmniejsz obszar mapy lub buffer_m w freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] tryb offline i brak w pamięci podręcznej: { $product }, arkusz { $tiles }; uruchom `freemapper fetch` online
        [few] tryb offline i brak w pamięci podręcznej: { $product }, arkusze { $tiles }; uruchom `freemapper fetch` online
        [many] tryb offline i brak w pamięci podręcznej: { $product }, arkusze { $tiles }; uruchom `freemapper fetch` online
       *[other] tryb offline i brak w pamięci podręcznej: { $product }, arkusze { $tiles }; uruchom `freemapper fetch` online
    }
word-product-orthophoto = ortofotomapa (DOP20)
word-product-dtm = numeryczny model terenu (DGM1)
word-product-image-dsm = numeryczny model pokrycia terenu (bDOM)
word-product-saxony-dtm = numeryczny model terenu (GeoSN DGM1)
word-product-saxony-dsm = numeryczny model pokrycia terenu (GeoSN DOM1)
error-fetch-osm-extract-missing = brak wyciągu OSM w pamięci podręcznej; uruchom `freemapper fetch` online
error-fetch-not-found = nie znaleziono { $name } na serwerze
error-fetch-refused = { $url }: serwer odrzucił żądanie (HTTP { $status })
error-fetch-refused-detail = { $url }: serwer odrzucił żądanie (HTTP { $status }): { $detail }
error-fetch-too-large = pobierany plik { $url } jest większy niż { $limit } MB; przerwano
error-fetch-not-stored = brak zapisanego zasobu ({ $label }) dla tego obszaru mapy: brakuje { $path }, nie można go odczytać albo nie obejmuje obszaru; najpierw uruchom `freemapper fetch` online
error-fetch-range-ignored = { $url }: serwer zignorował zakres bajtów { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper nie wysyła żądań na ten adres: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper nie wysyła żądań na ten adres: adapter { $name } nie wymienia hosta { $host }{ $from }. Jeśli usługa została tam przeniesiona, zezwól na host poleceniem `freemapper adapters allow-host { $adapter } { $host }` i zgłoś to na https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper nie wysyła żądań na ten adres: adapter { $adapter } nie wymienia hosta { $host }{ $from }. Jeśli usługa została tam przeniesiona, dodaj host do `[net] hosts` w manifeście adaptera
error-fetch-host-redirected-from = {" "}({ $url } przekierowuje tam)
error-reason-not-a-host-name = to nie jest nazwa hosta
error-reason-not-ascii = nazwa spoza ASCII musi być zapisana w formie xn--
error-reason-not-a-dns-name = to nie jest nazwa DNS
error-reason-no-domain = nazwa tego komputera lub bez domeny
error-reason-local-name = nazwa sieci lokalnej
error-reason-ip-address = adres IP, nie nazwa DNS
error-reason-unexpected-character = nieoczekiwany znak
error-reason-not-https = nie https
error-reason-user-info = dane użytkownika
error-reason-port = port inny niż 443
error-reason-host-not-declared = host nie należy do zadeklarowanych hostów
error-reason-local-address = adres tego komputera lub sieci lokalnej, a nie serwera w internecie
error-reason-too-many-redirects = więcej niż { $count } przekierowań
error-reason-redirect-nowhere = przekierowanie (HTTP { $status }) donikąd
error-fetch-currency-csv = CSV aktualności, wiersz { $line }: { $message }
error-alkis = nieprawidłowe dane ALKIS: { $message }
error-osm = dane OSM: { $message }

## Błędy: szablony i rastry

error-derive-zip = archiwum arkuszy: { $message }
error-derive-image = obraz: { $message }
error-derive-no-tiles = brak arkuszy, z których można wyliczyć dane
error-derive-area-too-large = obszar mapy zbyt duży dla jednego szablonu: { $width } × { $height } pikseli (najwyżej { $limit } na bok); zmniejsz obszar mapy lub buffer_m w freemapper.toml
error-raster-world-file = nieprawidłowy plik georeferencji: { $message }
error-raster-misaligned = rastry nie są wyrównane do tej samej siatki pikseli: { $message }
error-raster-pixel-size = rastry mają różne rozmiary pikseli: { $message }

## Błędy: obszar mapy, georeferencja, plik mapy, plik OCAD

error-area-geojson = nieprawidłowy GeoJSON: { $message }
error-area-no-polygon = GeoJSON nie zawiera wielokąta
error-area-multi-polygon = MultiPolygon z { $parts } częściami; podziel obszar mapy na pojedyncze wielokąty
error-area-out-of-range = współrzędna poza zakresem: szer. { $lat }, dł. { $lon }
error-area-not-an-area = to nie jest użyteczny obszar mapy: { $message }
error-georef-date = data { $date } leży poza zakresem ważności World Magnetic Model
error-georef-map-range = punkt E { $easting } N { $northing } leży poza zakresem współrzędnych mapy
error-omap-xml = nieprawidłowy XML OOM: { $message }
error-omap-missing-element = nie znaleziono elementu <{ $name }>
error-ocd-duplicate-symbol = numer znaku { $symbol } użyto dwa razy
error-ocd-unknown-symbol = obiekt odwołuje się do znaku { $symbol }, którego nie ma w pliku
error-ocd-wrong-type = obiekt znaku { $symbol } ma niewłaściwy typ dla swojego znaku

## Pomoc (`--help`). Angielskie teksty są równe komentarzom dokumentacyjnym w `src/main.rs`; test je porównuje.

cli-heading-usage = Użycie:
cli-heading-commands = Polecenia:
cli-heading-options = Opcje:
cli-heading-arguments = Argumenty:
cli-help-possible-values = możliwe wartości
cli-help-default = domyślnie
cli-arg-help = Wyświetl pomoc
cli-arg-version = Wyświetl wersję
cli-help-help = Pokaż pomoc polecenia
cli-arg-help-command = Polecenie, którego pomoc ma zostać pokazana, na przykład run
cli-parse-hint = Więcej informacji: '--help'.
cli-double-click-start = freemapper otwiera w przeglądarce stronę, na której wybierasz obszar mapy. Folder projektu to { $path }. Zamknij to okno, aby zakończyć program.
cli-parse-unknown-argument = nieoczekiwany argument { $arg }
cli-parse-missing-required = brak wymaganego argumentu: { $args }
cli-parse-invalid-value = nieprawidłowa wartość { $value } dla { $arg }
cli-parse-invalid-value-choices = nieprawidłowa wartość { $value } dla { $arg }; możliwe wartości: { $values }
cli-parse-invalid-subcommand = nieznane polecenie { $value }
cli-parse-missing-subcommand = wymagane jest polecenie
cli-parse-too-many-values = nieoczekiwana wartość { $value } dla { $arg }
cli-parse-argument-conflict = { $arg } nie można użyć razem z { $other }

help-freemapper = Tworzy projekty startowe OOM z danych geodezyjnych
help-freemapper-lang = Język tekstów programu, `de` lub `en` (domyślnie: FREEMAPPER_LANG, potem język systemu, potem angielski)
help-init = Utwórz folder projektu dla obszaru mapy
help-init-dir = Folder projektu do utworzenia
help-init-geojson = Plik GeoJSON z wielokątem obszaru mapy (EPSG:4326)
help-init-osm = Obiekt OSM obszaru mapy, np. way/992423119
help-init-place = Nazwa miejsca, np. "Clara-Zetkin-Schule Strausberg"; freemapper użyje najlepszego dopasowania z obszarem
help-init-part = Której części obszaru OSM złożonego z kilku oddzielnych części użyć (1 to największa)
help-init-kind = Rodzaj mapy
help-arg-scale = Mianownik skali nowego projektu: dziedziniec szkolny 1000 (domyślnie); sprint 4000 (domyślnie) lub 5000; las 15000 (domyślnie), 10000 lub 7500. Inne wartości są odrzucane, bo freemapper-print ich nie przyjmuje
help-init-title = Tytuł mapy do informacji na marginesie (domyślnie: nazwa folderu)
help-export = Zapisz projekt startowy (mapę OOM, szablony, informacje o źródłach) w folderze projektu
help-export-dir = Folder projektu
help-arg-force = Zastąp istniejący plik mapy i zachowaj starą wersję jako `<name>.omap.bak`
help-export-ocad = Zapisz też punkt startowy OCAD w `ocad/` (format OCAD 12); istniejąca mapa OOM zostaje wtedy bez zmian, chyba że podasz --force
help-export-pdf = Zapisz też `<name>.pdf`, mapę do wydruku (tylko obiekty mapy, bez kandydatów); istniejąca mapa OOM zostaje wtedy bez zmian, chyba że podasz --force
help-export-pdf-candidates = Zapisz też `<name>-candidates.pdf`, wydruk kontrolny: mapa ze wszystkimi kandydatami, zgodnie z położeniem; istniejąca mapa OOM zostaje wtedy bez zmian, chyba że podasz --force
help-export-training = Zapisz też mapę treningową `<name>-training.omap` i jej PDF `<name>-training.pdf`: mapa i wszyscy kandydaci, uporządkowane do wydruku w tej postaci; z --ocad także `ocad/<name>-training.ocd`. Istniejąca mapa OOM zostaje wtedy bez zmian, chyba że podasz --force, a --force zastępuje też istniejącą mapę treningową
help-export-view = Z --training: zapisz też mapę treningową bez tego, co pomija widok, jako `<name>-training-<view>.omap` i jej PDF (z --ocad także plik OCAD). Widoki: `no-paths` (bez dróg, ścieżek, schodów), `brown-only` (tylko brązowe znaki i ramka), `no-north-lines`, albo lista `no:<codes>` / `only:<codes>`; łącz za pomocą `+`. Powtórz opcję, aby uzyskać kilka plików
help-export-pdf-scale-bar = Narysuj podziałkę liniową w plikach PDF (domyślnie wyłączone, niemieckie mapy zwykle jej nie mają; liczba skali pozostaje w tabliczce tytułowej)
help-export-pdf-sources = Wiersze źródeł w plikach PDF: `auto` drukuje pełną listę i skraca ją tylko wtedy, gdy zajęłaby więcej niż 5 wierszy, `full` nigdy nie skraca, `credit-only` drukuje tylko wiersz z informacją o freemapper (ODbL i dl-de/by-2-0 wymagają wtedy, by przy przekazywaniu mapy podać źródła). ATTRIBUTION.txt zawsze zawiera wszystkie źródła
help-fetch = Pobierz dane dla obszaru mapy (ortofotomapę, numeryczny model terenu, nDSM, budynki ALKIS, OSM)
help-fetch-dir = Folder projektu
help-fetch-offline = Użyj tylko pamięci podręcznej, bez dostępu do sieci
help-arg-cache-dir = Folder pamięci podręcznej dla pobranych arkuszy (domyślnie: systemowy folder pamięci podręcznej + `freemapper`)
help-derive = Wylicz szablony (ortofotomapę, cieniowanie, spadki, rzeźbę lokalną, nDSM), warstwice i formy terenu z pobranych danych
help-derive-dir = Folder projektu
help-run = Pobierz, wylicz i zapisz za jednym razem
help-run-dir = Folder projektu
help-run-offline = Użyj tylko pamięci podręcznej, bez dostępu do sieci
help-run-ocad = Zapisz też punkt startowy OCAD w `ocad/` (format OCAD 12)
help-run-pdf = Zapisz też `<name>.pdf`, mapę do wydruku (tylko obiekty mapy, bez kandydatów)
help-run-pdf-candidates = Zapisz też `<name>-candidates.pdf`, wydruk kontrolny: mapa ze wszystkimi kandydatami, zgodnie z położeniem
help-run-training = Zapisz też mapę treningową `<name>-training.omap` i jej PDF `<name>-training.pdf`: mapa i wszyscy kandydaci, uporządkowane do wydruku w tej postaci; z --ocad także `ocad/<name>-training.ocd`. Istniejąca mapa treningowa zostaje, chyba że podasz --force
help-run-view = Z --training: zapisz też mapę treningową bez tego, co pomija widok, jako `<name>-training-<view>.omap` i jej PDF (z --ocad także plik OCAD). Widoki: `no-paths` (bez dróg, ścieżek, schodów), `brown-only` (tylko brązowe znaki i ramka), `no-north-lines`, albo lista `no:<codes>` / `only:<codes>`; łącz za pomocą `+`. Istniejący plik widoku zostaje, chyba że podasz --force
help-run-pdf-scale-bar = Narysuj podziałkę liniową w plikach PDF (domyślnie wyłączone; liczba skali pozostaje w tabliczce tytułowej)
help-run-pdf-sources = Wiersze źródeł w plikach PDF: `auto` drukuje pełną listę i skraca ją tylko wtedy, gdy zajęłaby więcej niż 5 wierszy, `full` nigdy nie skraca, `credit-only` drukuje tylko wiersz z informacją o freemapper (ODbL i dl-de/by-2-0 wymagają wtedy, by przy przekazywaniu mapy podać źródła). ATTRIBUTION.txt zawsze zawiera wszystkie źródła
help-select = Wybierz obszar mapy w przeglądarce, rysując go lub wskazując miejsce. Zapis tworzy freemapper.toml i w razie potrzeby folder projektu
help-select-dir = Folder projektu (tworzony przy zapisie, jeśli nie istnieje)
help-select-kind = Rodzaj mapy nowego projektu
help-select-title = Tytuł mapy nowego projektu
help-select-no-browser = Tylko wypisz adres strony; nie otwieraj przeglądarki
help-select-port = Port na 127.0.0.1 (domyślnie: wolny)
help-select-offline = Strona startuje z zaznaczonym "tylko pamięć podręczna, bez sieci"
help-select-ocad = Strona startuje z zaznaczonym projektem startowym OCAD
help-select-pdf = Strona startuje z zaznaczonym PDF do wydruku
help-select-pdf-candidates = Strona startuje z zaznaczonym wydrukiem kontrolnym (wszyscy kandydaci)
help-select-training = Strona startuje z zaznaczoną mapą treningową
help-select-pdf-scale-bar = Strona startuje z zaznaczoną podziałką liniową w plikach PDF
help-select-pdf-sources = Strona startuje z tym wyborem wierszy źródeł w plikach PDF, jak dla `run`
help-find = Wypisz miejsca OpenStreetMap dla nazwy, najpierw obszary, z identyfikatorem dla init --osm
help-find-query = Nazwa miejsca, np. "Clara-Zetkin-Schule Strausberg"
help-find-json = Wypisz dopasowania jako JSON, z tymi samymi polami co narzędzie MCP find_place
help-mcp = Udostępnij narzędzia agentowi AI przez MCP (stdin/stdout)
help-update = Zainstaluj nowszy freemapper z wydań GitHub, po zapytaniu. Wersja zbudowana ze źródeł podaje zamiast tego, jak ją zaktualizować
help-update-check = Tylko sprawdź: kod wyjścia 10, gdy istnieje nowsza wersja, 0, gdy ta jest aktualna
help-update-yes = Zainstaluj bez pytania; potrzebne, gdy wejście nie jest terminalem
help-update-version = Zainstaluj tę wersję (x.y.z) zamiast najnowszej, także starszą
help-update-rollback = Wróć do wersji zastąpionej przez ostatnią aktualizację
help-update-mode = Zapisz, co robi codzienne sprawdzanie po znalezieniu nowszej wersji (domyślnie: notify). Działa w każdej wersji, bo zapisuje tylko ustawienie
help-adapters = Pokaż adaptery regionów: kraje związkowe i państwa, dla których freemapper ma źródła danych
help-adapters-list = Wypisz adaptery wraz z pochodzeniem (wbudowany, twój folder, projekt), rewizją, dniem ostatniego sprawdzenia i hostami, do których się zwraca
help-adapters-list-json = Wypisz listę jako JSON
help-adapters-list-project = Wczytaj też adaptery z folderu `adapters` tego folderu projektu, tak jak zrobiłoby to --adapters-from-project
help-adapters-check = Sprawdź folder adaptera, zanim zostanie użyty lub przekazany dalej: czy da się go wczytać, co powinien wiedzieć jego autor, a z --live także czy jego usługi odpowiadają. Kod wyjścia 0: bez uwag; 1: są uwagi; 2: nie można było wykonać sprawdzenia
help-adapters-check-dir = Folder adaptera; jego nazwa jest identyfikatorem adaptera
help-adapters-check-live = Zapytaj też usługi adaptera o obszar podany w --bbox i powiedz, co każda z nich dostarcza. Żądania pozostają na hostach zadeklarowanych w manifeście
help-adapters-check-bbox = Obszar sprawdzenia na żywo w stopniach, jako minLon,minLat,maxLon,maxLat. Leży wewnątrz granicy adaptera; wystarczy 150 m na 150 m
help-adapters-check-json = Wypisz wynik jako JSON
help-adapters-allow-host = Zezwól wbudowanemu adapterowi na jeszcze jeden host: dla kraju związkowego, którego usługa przeniosła się na serwer jeszcze niewymieniony w adapterze. Komunikat o odrzuconym adresie podaje, o który host chodzi. Host jest sprawdzany jak host manifestu, a freemapper nadal łączy się tylko z serwerami w internecie
help-adapters-allow-host-adapter = Identyfikator wbudowanego adaptera, jak pokazuje go `adapters list`
help-adapters-allow-host-host = Host, na który zezwalasz: nazwa DNS, bez adresu, bez portu
help-adapters-disallow-host = Cofnij host, na który zezwoliło allow-host
help-adapters-disallow-host-adapter = Identyfikator wbudowanego adaptera
help-adapters-disallow-host-host = Host do cofnięcia
help-arg-adapters-from-project = Wczytaj też adaptery z folderu `adapters` w folderze projektu. freemapper wymienia każdy z nich wraz z hostami, do których się zwraca. Używaj dla projektu, któremu ufasz: adapter sprawia, że freemapper odpytuje serwery, które wymienia

## Podsumowanie uruchomienia, dla serwera MCP i strony obszaru

cli-internal-error = błąd wewnętrzny: { $message }
cli-pdf-sources-invalid = pdf_sources musi mieć wartość auto, full lub credit-only, a nie { $value }
cli-view-unknown = widok to no-paths, brown-only, no-north-lines, no:<codes> lub only:<codes>, a nie { $value }
cli-view-empty = lista { $value } nie zawiera żadnego kodu znaku
cli-view-bad-code = { $value } nie jest kodem znaku, takim jak 505 lub 501.6
cli-summary-training-view = Zapisano { $map } i { $pdf } (widok mapy treningowej)
cli-summary-kept-training-view = Zachowano pliki widoku { $view } mapy treningowej; podaj --force, aby je odnowić
cli-summary-training-view-failed = Widok mapy treningowej nie powiódł się: { $error }
cli-estimate-disk-unreadable = nie można odczytać wolnego miejsca na dysku dla { $path } ({ $message })
cli-summary-wrote-map = Zapisano mapę { $path }
cli-summary-kept-map = Zachowano istniejącą mapę { $path } (może zawierać pracę kartografa)
cli-summary-map-lacks-templates = Mapa nie odwołuje się jeszcze do szablonów rastrowych. Uruchom potok z force, aby ją zastąpić; stara wersja zostanie jako .omap.bak.
cli-summary-map-lacks-buildings = Mapa nie ma jeszcze budynków ALKIS. Uruchom potok z force, aby je dodać; zastąpi to mapę, a stara wersja zostanie jako .omap.bak.
cli-summary-map-lacks-canopies = W mapie brakuje wiat i carportów z ALKIS (522), które są teraz obiektami mapy. Uruchom potok z force, aby je dodać; zastąpi to mapę, a stara wersja zostanie jako .omap.bak.
cli-summary-map-light = Mapa bazowa: połączono { $joined } linii, { $left } ukrytych lub zbyt małych obiektów trafiło do warstw kandydatów (nic nie przesunięto)
cli-summary-map-light-kept = Warstwy kandydatów zawierają teraz także { $left } obiektów lub odcinków, które ta wersja zdejmuje z mapy bazowej (z oznaczeniem freemapper:generalized). Zachowana mapa nadal je ma, więc są widoczne dwa razy. Uruchom potok z force, aby oczyścić mapę; stara wersja zostanie jako .omap.bak.
cli-summary-map-old-candidates = Mapa nadal wskazuje templates/candidates.omap; to uruchomienie wypełniło go wszystkimi kandydatami. Kandydaci są teraz dostępni także w siedmiu warstwach, po jednym pliku (templates/candidates-*.omap). Mapa wymienia je po uruchomieniu z force (stara mapa zostaje jako .omap.bak) albo kartograf dodaje je w OOM (Templates > Open template…).
cli-summary-wrote-ocad = Zapisano mapę OCAD { $path } ({ $count } obiektów)
cli-summary-kept-ocad = Zachowano istniejącą mapę OCAD { $path } (może zawierać pracę kartografa; force ją zastępuje)
cli-summary-ocad-old-candidates = Mapa OCAD nadal wskazuje { $path } jako mapę w tle; to uruchomienie wypełniło ją wszystkimi kandydatami. Kandydaci są teraz dostępni także w siedmiu warstwach, po jednym pliku (-candidates-*.ocd). Mapa OCAD wymienia je po uruchomieniu z force (stara mapa zostaje jako .ocd.bak) albo kartograf dodaje je w OCAD jako mapy w tle.
cli-summary-ocad-left-out = Pominięto w mapie OCAD: { $skipped } obiektów i { $dropped } zdegenerowanych części
cli-summary-ocad-symbols = W OCAD wybierz Symbol > Load Colors and Symbols From… i wskaż oficjalny zestaw znaków{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , zestaw szkolny IOF 2019
       *[other] {" "}ISSprOM 2019
    }; zastępuje on znaki zastępcze według numerów.
cli-summary-ocad-failed = Eksport OCAD nie powiódł się: { $error }
cli-summary-pdf-without-credits = PDF bez informacji o źródłach: ODbL i dl-de/by-2-0 wymagają podania źródeł przy przekazywaniu mapy; zobacz ATTRIBUTION.txt
cli-summary-wrote-pdf = Zapisano mapę { $path } (bez kandydatów)
cli-summary-pdf-failed = Eksport PDF nie powiódł się: { $error }
cli-summary-wrote-pdf-candidates = Zapisano wydruk kontrolny { $path } (mapa ze wszystkimi kandydatami, zgodnie z położeniem)
cli-summary-pdf-candidates-failed = Eksport wydruku kontrolnego nie powiódł się: { $error }
cli-summary-training-failed = Eksport mapy treningowej nie powiódł się: { $error }
cli-summary-kept-training = Zachowano istniejącą mapę treningową { $path }, aby uruchomienie nie nadpisało mapy, która została wydrukowana lub przekazana dalej; usuń ją lub ustaw force, aby ją odnowić

## Serwer MCP. Opisy narzędzi i parametrów są równe tekstom w `src/mcp.rs`; test je porównuje.

mcp-error-no-home = w tym systemie nie ma folderu domowego; użyj ścieżki bezwzględnej
mcp-error-relative-path = { $path } jest ścieżką względną; użyj ścieżki bezwzględnej lub zaczynającej się od ~
mcp-error-home-folder = { $path } to sam folder domowy; użyj folderu wewnątrz niego, np. ~/maps/school
mcp-error-no-project = w { $path } nie ma projektu freemapper; najpierw użyj create_project
mcp-error-unknown-kind = nieznany rodzaj { $kind }; użyj "schoolyard", "sprint" lub "forest"
mcp-error-reading = odczyt { $path }: { $error }
mcp-error-one-area-source = podaj dokładnie jedno z: osm_id (z find_place) lub geojson
mcp-error-page-open = strona wyboru obszaru dla tego projektu jest nadal otwarta; poproś użytkownika, aby zapisał obszar i nacisnął Gotowe na stronie, a potem uruchom potok
mcp-error-no-job = brak zadania { $job }; start_pipeline i permits_for_area zwracają identyfikatory zadań tej sesji
mcp-error-job-running = { $job } nadal działa dla { $path }; naraz działa jeden potok, bo projekty dzielą pamięć podręczną pobierania. Uruchom ten, gdy project_status zgłosi finished lub failed
mcp-error-page-start = nie można uruchomić strony: { $error }
mcp-created-project = Utworzono projekt { $path } dla { $area } ({ $kind }, 1:{ $scale }). Dalej: estimate_download, potem start_pipeline.
mcp-created-project-given-area = Utworzono projekt { $path } dla podanego obszaru ({ $kind }, 1:{ $scale }). Dalej: estimate_download, potem start_pipeline.
mcp-page-already-open = Strona dla tego folderu jest już otwarta; zachowuje ustawienia, z którymi została otwarta.
mcp-page-next = Poproś użytkownika, aby wybrał obszar na stronie, nacisnął Zapisz, a potem Gotowe.
mcp-preview-of = Podgląd warstwy { $layer }
mcp-permits-next = Wskazówki dotyczące zgód są w trakcie tworzenia. Odpytuj project_status z tym job_id; gdy zadanie się zakończy, permits_md zawiera tekst permits.md. Przekaż go jako wskazówkę, nie poradę prawną.
mcp-tool-check-update = Czy istnieje nowsze wydanie freemapper: jego wersja, data wydania, uwagi do wydania i rozmiar pobierania, albo informacja, że ta wersja jest aktualna. Niczego nie instaluje.
mcp-tool-create-project = Utwórz folder projektu dla obszaru mapy. Podaj osm_id (z find_place) lub plik GeoJSON. kind to schoolyard (1:1000, domyślnie), sprint (1:4000) lub forest (1:15000).
mcp-param-create-project-dir = Nowy folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-param-create-project-geojson = Zamiast osm_id: plik GeoJSON z wielokątem obszaru mapy (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (domyślnie, 1:1000), "sprint" (1:4000) lub "forest" (1:15000).
mcp-param-create-project-osm-id = Obiekt OSM obszaru mapy z find_place, np. "way/992423119".
mcp-param-create-project-part = Z osm_id: której części obszaru złożonego z kilku oddzielnych części użyć (1 to największa).
mcp-param-create-project-scale = Mianownik skali: schoolyard 1000 (domyślnie); sprint 4000 (domyślnie) lub 5000; forest 15000 (domyślnie), 10000 lub 7500. Inne wartości są odrzucane; dla 1:3000 rysuj w 4000, freemapper-print powiększa przy drukowaniu.
mcp-param-create-project-title = Tytuł mapy do informacji na marginesie (domyślnie: nazwa folderu).
mcp-tool-estimate-download = Co start_pipeline pobierze dla projektu: arkusze, każde pobranie z rozmiarem w MB, czy jest w pamięci podręcznej i czy mieści się na dysku, ile MB zostało do pobrania, a w disk wolne miejsce w pamięci podręcznej i na woluminach projektu wraz z komunikatami o tym, co się nie mieści (przekaż je użytkownikowi).
mcp-param-estimate-download-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-tool-find-place = Wyszukaj szkołę, park lub inne miejsce po nazwie (OpenStreetMap Nominatim). Zwraca do 5 dopasowań, najpierw obszary; obszarem mapy mogą być tylko dopasowania z has_area. Pusta lista oznacza, że niczego nie znaleziono.
mcp-param-find-place-query = Nazwa szkoły lub miejsca, np. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Zainstaluj najnowsze wydanie freemapper w miejsce tego programu. Wywołuj tylko po zgodzie osoby, z confirm true. Nowa wersja zacznie działać po ponownym uruchomieniu serwera freemapper przez asystenta.
mcp-param-install-update-confirm = true, gdy osoba zgodziła się na instalację nowszej wersji; bez tego nic nie jest instalowane.
mcp-tool-list-layers = Warstwy projektu: mapa, siedem warstw kandydatów i szablony rastrowe, każda z plikiem, informacją, czy już powstała, i krokiem, który ją tworzy.
mcp-param-list-layers-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-tool-open-area-selector = Otwórz w przeglądarce stronę, na której użytkownik rysuje obszar mapy (prostokąt lub wielokąt) albo wskazuje miejsce, widzi arkusze i rozmiar pobierania, i zapisuje go; zapis tworzy folder projektu lub zmienia jego obszar. Zwraca adres URL do przekazania użytkownikowi. Strona działa, dopóki użytkownik nie naciśnie Gotowe; ponowne wywołanie dla tego samego folderu zwraca ten sam adres URL.
mcp-param-open-area-selector-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~); tworzony, gdy użytkownik zapisze.
mcp-param-open-area-selector-kind = Rodzaj mapy nowego projektu: "schoolyard" (domyślnie, 1:1000), "sprint" (1:4000) lub "forest" (1:15000).
mcp-param-open-area-selector-ocad = Panel uruchomienia startuje z zaznaczonym projektem startowym OCAD. Domyślnie false.
mcp-param-open-area-selector-offline = Panel uruchomienia startuje z zaznaczonym "tylko pamięć podręczna, bez sieci". Domyślnie false.
mcp-param-open-area-selector-open-browser = Otwórz też stronę w domyślnej przeglądarce tego komputera. Domyślnie true.
mcp-param-open-area-selector-pdf = Panel uruchomienia startuje z zaznaczonym PDF do wydruku. Domyślnie false.
mcp-param-open-area-selector-pdf-candidates = Panel uruchomienia startuje z zaznaczonym wydrukiem kontrolnym (wszyscy kandydaci). Domyślnie false.
mcp-param-open-area-selector-pdf-scale-bar = Panel uruchomienia startuje z zaznaczoną podziałką liniową w plikach PDF. Domyślnie false.
mcp-param-open-area-selector-pdf-sources = Wiersze źródeł w plikach PDF, z którymi startuje panel: "auto" (domyślnie), "full" lub "credit-only", jak dla start_pipeline.
mcp-param-open-area-selector-permits = Panel uruchomienia startuje z zaznaczonymi wskazówkami dotyczącymi zgód. Domyślnie false.
mcp-param-open-area-selector-scale = Mianownik skali nowego projektu: schoolyard 1000 (domyślnie); sprint 4000 (domyślnie) lub 5000; forest 15000 (domyślnie), 10000 lub 7500. Inne wartości są odrzucane.
mcp-param-open-area-selector-title = Tytuł mapy nowego projektu.
mcp-param-open-area-selector-training = Panel uruchomienia startuje z zaznaczoną mapą treningową. Domyślnie false.
mcp-tool-preview-layer = Podgląd PNG (najwyżej 800 px) warstwy: map (obraz obiektów mapy zapisany przez ostatnie uruchomienie, bez kandydatów) albo szablon rastrowy: orthophoto, hillshade, slope, relief, ndsm, albo k5 w Berlinie.
mcp-param-preview-layer-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" lub "k5" (tylko Berlin).
mcp-tool-project-status = Stan zadania start_pipeline lub permits_for_area: running, finished lub failed, ostatnie wiersze postępu, ostrzeżenia, zapisane pliki oraz wynik lub błąd. Zakończone zadanie permits_for_area zawiera też tekst permits.md jako permits_md.
mcp-param-project-status-job-id = Identyfikator zadania z start_pipeline lub permits_for_area, np. "job-1".
mcp-tool-permits-for-area = Przygotuj wskazówki dotyczące zgód dla projektu: jednostki administracyjne obszaru mapy, leżące w nim obszary chronione i kto zarządza tamtejszym terenem, wraz z urzędami, do których należy się zwrócić; wskazówka, nie porada prawna. Zapisuje permits.md i permits.json w folderze. Działa w tle i zwraca job_id; odpytuj project_status, którego odpowiedź po zakończeniu zadania zawiera tekst permits.md jako permits_md. Naraz działa jedno zadanie.
mcp-param-permits-for-area-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-param-permits-for-area-offline = Czytaj tylko dane zapisane w projekcie przez wcześniejsze uruchomienie, bez dostępu do sieci. Domyślnie false.
mcp-tool-start-pipeline = Pobierz dane, wylicz szablony i zapisz projekt startowy (fetch, derive, export). Działa w tle i zwraca job_id; odpytuj project_status. Naraz działa jeden potok. Zachowuje istniejącą mapę, chyba że force ma wartość true.
mcp-param-start-pipeline-dir = Folder projektu (ścieżka bezwzględna lub zaczynająca się od ~).
mcp-param-start-pipeline-force = Zastąp istniejący plik mapy i zachowaj starą wersję jako .omap.bak. Domyślnie false.
mcp-param-start-pipeline-ocad = Zapisz też punkt startowy OCAD w folderze ocad/ (format OCAD 12, dla osób pracujących w OCAD). Domyślnie false.
mcp-param-start-pipeline-offline = Użyj tylko pamięci podręcznej pobierania, bez dostępu do sieci. Domyślnie false.
mcp-param-start-pipeline-pdf = Zapisz też <name>.pdf, mapę gotową do wydruku (tylko obiekty mapy, bez kandydatów). Domyślnie false.
mcp-param-start-pipeline-pdf-candidates = Zapisz też <name>-candidates.pdf, wydruk kontrolny: mapa ze wszystkimi kandydatami, zgodnie z położeniem (sprawdź ich w terenie). Domyślnie false.
mcp-param-start-pipeline-pdf-scale-bar = Narysuj podziałkę liniową w plikach PDF. Domyślnie false (niemieckie mapy zwykle jej nie mają; liczba skali pozostaje w tabliczce tytułowej).
mcp-param-start-pipeline-pdf-sources = Wiersze źródeł w plikach PDF: "auto" (domyślnie; pełna lista, skracana tylko wtedy, gdy zajmuje więcej niż 5 wierszy), "full" (nigdy nie skracana) lub "credit-only" (tylko wiersz z informacją o freemapper; ODbL i dl-de/by-2-0 wymagają wtedy podania źródeł przy przekazywaniu mapy). ATTRIBUTION.txt zawsze zawiera wszystkie źródła.
mcp-param-start-pipeline-permits = Zapisz też wskazówki dotyczące zgód (permits.md i permits.json) po mapie: jednostki administracyjne, obszary chronione i kto zarządza terenem. Błąd w tym kroku to ostrzeżenie, a nie nieudane zadanie. Domyślnie false.
mcp-param-start-pipeline-training = Zapisz też mapę treningową <name>-training.omap i jej PDF <name>-training.pdf: mapa i wszyscy kandydaci, uporządkowane do wydruku w tej postaci; z ocad także ocad/<name>-training.ocd. Istniejąca mapa treningowa zostaje, chyba że force ma wartość true. Domyślnie false.
mcp-param-start-pipeline-training-views = Z training: zapisz też mapę treningową bez tego, co pomija widok, po jednym zestawie plików na wpis: "no-paths", "brown-only", "no-north-lines", albo lista, taka jak "no:505,506" lub "only:101,102", łączona za pomocą +. Pliki to <name>-training-<view>.omap i jego PDF.
mcp-instructions = freemapper tworzy projekty startowe OpenOrienteering Mapper dla map do biegu na orientację (dziedziniec szkolny 1:1000, sprint 1:4000, las 1:15000) z oficjalnych danych geodezyjnych: { $states }, oraz z OSM. Przebieg: find_place (wyszukaj szkołę lub obszar po nazwie; wybierz wynik z has_area) → create_project (nowy folder, ścieżka bezwzględna, z osm_id) → estimate_download (powiedz użytkownikowi, ile zostanie pobrane) → start_pipeline (zwraca identyfikator zadania; pobieranie może trwać kilka minut) → project_status (odpytuj co kilka sekund, aż stan będzie finished lub failed) → list_layers i preview_layer (pokaż użytkownikowi szablony). Następnie użytkownik otwiera plik mapy w OpenOrienteering Mapper. Gdy żadne miejsce nie pasuje albo użytkownik chce narysować obszar, open_area_selector otwiera w przeglądarce stronę, na której użytkownik rysuje lub wybiera obszar i go zapisuje (to tworzy projekt); kontynuuj od estimate_download, gdy użytkownik powie, że zapisał. Ścieżki muszą być bezwzględne lub zaczynać się od ~. Naraz działa jeden potok. permits_for_area przygotowuje wskazówki dotyczące zgód dla projektu (jednostki administracyjne, obszary chronione, kto zarządza terenem; wskazówka, nie porada prawna) jako zadanie, tak jak start_pipeline. check_update mówi, czy istnieje nowszy freemapper; install_update go instaluje, tylko z confirm: true po zgodzie osoby, a asystent następnie ponownie uruchamia serwer freemapper.

cli-contour-interval-saved = Cięcie warstwic zapisane w freemapper.toml: { $interval } m; derive wyznacza warstwice przy tym cięciu
cli-contour-interval-auto = Cięcie warstwic w freemapper.toml: automatyczne (zależne od rodzaju mapy; mapa leśna ma 2,5 m na terenie płaskim i 10 m na stromym)
help-arg-contour-interval = Cięcie warstwic w metrach, np. `2.5`, albo `auto` dla cięcia właściwego rodzajowi mapy (mapa leśna: 5 m, 2,5 m na terenie płaskim, 10 m na stromym). Zapisane w freemapper.toml; derive wyznacza warstwice przy tym cięciu
cli-content-saved = Ustawienie zawartości zapisane w freemapper.toml: { $summary }
cli-content-invalid = Nie można odczytać ustawienia zawartości: { $detail }
help-arg-content = Co trafia bezpośrednio na mapę: `safe` zostawia każdy obiekt tam, gdzie umieszcza go budowa mapy, `balanced` (domyślnie) umieszcza na mapie obiekty z katastru i rejestrów, `full` umieszcza na mapie wszystko. Wybór jest zapisywany w freemapper.toml
help-arg-content-set = Skieruj grupę obiektów do `map`, `candidates` lub `off`, jako `GROUP=PLACE`, np. `walls=map`; można powtarzać. Grupy: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Zapisane w freemapper.toml
mcp-param-create-project-content = Co trafia bezpośrednio na mapę: "safe", "balanced" (domyślnie) lub "full". Zapisane w projekcie. balanced umieszcza na mapie obiekty z katastru i rejestrów, full umieszcza na niej wszystko.
mcp-param-create-project-content-set = Grupy skierowane gdzie indziej, niż mówi profil, jako "group=place", gdzie place to map, candidates lub off, np. "walls=map". Grupy: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Cięcie warstwic w metrach, np. 2.5; 0 wraca do wyboru automatycznego (zależnego od rodzaju mapy; mapa leśna ma 2,5 m na terenie płaskim i 10 m na stromym). Zapisane w projekcie; następny derive wyznacza warstwice przy tym cięciu.
mcp-param-start-pipeline-content = Zmień przed uruchomieniem, co trafia bezpośrednio na mapę: "safe", "balanced" lub "full". Zapisane w projekcie. Domyślnie: ustawienie projektu, a w razie braku balanced.
mcp-param-start-pipeline-content-set = Grupy skierowane gdzie indziej, niż mówi profil, jako "group=place", gdzie place to map, candidates lub off, np. "fences=candidates". Zapisane w projekcie.
mcp-param-start-pipeline-contour-interval = Cięcie warstwic w metrach, np. 2.5; 0 wraca do wyboru automatycznego (zależnego od rodzaju mapy; mapa leśna ma 2,5 m na terenie płaskim i 10 m na stromym). Zapisane w projekcie; następny derive wyznacza warstwice przy tym cięciu.
mcp-error-content = Nie można odczytać ustawienia zawartości: { $detail }

## Wskazówki dotyczące zgód (`freemapper permits`, `run --permits`, ADR 0117). Identyfikatory `permits-hint-*`
## to klucze, które silnik zapisuje w permits.json; `src/permits.rs` sprawdza, czy każdy
## identyfikator `permits-*` jest użyty. Cytat "Nicht für Planungszwecke geeignet." to
## własne sformułowanie BfN i w każdym języku pozostaje po niemiecku.

help-permits = Zapisz wskazówki dotyczące zgód dla projektu (permits.md i permits.json): jednostki administracyjne, obszary chronione i zarządców, których dotyka obszar mapy, oraz to, kogo zwykle trzeba zapytać. Odpytuje BfN i BKG, a tam, gdzie kraj związkowy obszaru mapy ma własne otwarte usługi, także je, przez hosty swojego adaptera (lista w podręczniku, rozdział 12); w Nadrenii Północnej-Westfalii pobiera przy pierwszym razie około 32 MB, w Brandenburgii około 4 MB, później tylko nowsze pliki. Czyta już przechowywany wyciąg OSM; --offline używa tego, co pobrano wcześniej
help-permits-dir = Folder projektu
help-permits-offline = Użyj tylko danych pobranych wcześniej, bez dostępu do sieci
help-run-permits = Zapisz też wskazówki dotyczące zgód (permits.md i permits.json) po mapie; odpytuje BfN, BKG i otwarte usługi kraju związkowego (podręcznik, rozdział 12). Błąd w tym kroku to ostrzeżenie, a mapa zostaje
cli-wrote-permits = Zapisano wskazówki dotyczące zgód { $path }
cli-summary-wrote-permits = Zapisano wskazówki dotyczące zgód { $path }
cli-permits-failed = nie udało się zapisać wskazówek dotyczących zgód: { $error }
cli-permits-source-left-out = wskazówki dotyczące zgód pomijają { $source }: { $reason }
cli-permits-not-read = nie odczytano
error-permits-stored = { $path }: { $message }; pobierz wskazówki dotyczące zgód ponownie online
error-permits-nothing-read = nie udało się odczytać żadnego źródła wskazówek dotyczących zgód ({ $missing }); najpierw pobierz je online
error-permits-missing-not-read = { $source }: nie odczytano

permits-title = Wskazówki dotyczące zgód
permits-disclaimer = Wskazówka, nie porada prawna. Decydują właściwe organy. Datę i dokładność danych podaje tabela w części "Źródła i data danych"; obszarów może brakować lub mogą być nieaktualne (BfN: dane łączone raz w roku, "Nicht für Planungszwecke geeignet.", czyli nieodpowiednie do celów planistycznych).
permits-disclaimer-regional = Wskazówka, nie porada prawna. Decydują właściwe organy. Datę i dokładność danych podaje tabela w części "Źródła i data danych"; obszarów może brakować lub mogą być nieaktualne.
permits-bfn-merge = BfN łączy dane krajów związkowych raz w roku i oznacza je jako nieodpowiednie do celów planistycznych. Decydują organy kraju związkowego.
permits-created = Sporządzono dnia { $date } dla obszaru mapy: { $kind ->
        [schoolyard] mapa dziedzińca szkolnego
        [forest] mapa leśna
       *[other] mapa sprintowa
    } ({ $area } ha).
permits-section-admin = Jednostki administracyjne
permits-section-protected = Obszary chronione (BfN)
permits-section-osm-protected = Dalsze obszary chronione z OpenStreetMap
permits-section-operators = Zarządcy i właściciele według OpenStreetMap
permits-section-hints = Wskazówki
permits-section-sources = Źródła i data danych
permits-none-admin = VG250 nie wymienia żadnej jednostki administracyjnej dla obszaru mapy.
permits-none-protected = Według BfN w obszarze mapy nie leży żaden obszar chroniony.
permits-none-osm-protected = OpenStreetMap nie wymienia w obszarze mapy żadnego obszaru chronionego, którego BfN jeszcze nie podaje.
permits-none-operators = OpenStreetMap nie wymienia dla obszaru mapy żadnego zarządcy ani właściciela.
permits-none-hints = Brak wskazówek.
permits-section-not-read = Nie odczytano danych { $source }, więc ta część jest pusta, choć mogą istnieć wpisy.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Granice administracyjne 1:250 000 (VG250) z BKG
        [bfn] Obszary chronione w Niemczech z BfN
       *[osm] OpenStreetMap, z wyciągu przechowywanego przez freemapper w pamięci podręcznej
    }
permits-level = { $level ->
        [land] Kraj związkowy
        [kreis] Powiat (Kreis)
        [verwaltungsgemeinschaft] Amt lub Verwaltungsgemeinschaft
       *[gemeinde] Gmina (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Dotknięte na krawędzi (mniej niż 0,5% obszaru mapy):
permits-at-edge-generalised = VG250 jest uogólniony do skali 1:250 000. Jednostka, która tylko dotyka krawędzi, może być skutkiem tego uogólnienia.
permits-kind = { $kind ->
        [nationalpark] Park narodowy
        [nationales-naturmonument] Narodowy pomnik przyrody
        [naturschutzgebiet] Rezerwat przyrody (Naturschutzgebiet)
        [ffh] Obszar FFH (Natura 2000)
        [vogelschutzgebiet] Obszar ochrony ptaków (Natura 2000)
        [biosphaerenreservat] Rezerwat biosfery
        [biosphaerenreservat-zone] Strefa rezerwatu biosfery
        [landschaftsschutzgebiet] Obszar chronionego krajobrazu (Landschaftsschutzgebiet)
       *[naturpark] Park przyrody
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Ustanowiono dnia { $date }
permits-protected-title = Podstawa prawna: { $title }
permits-protected-link = Tekst aktu prawnego: { $url }
permits-protected-site-link = Strona obszaru (Natura 2000): { $url }
permits-protected-year = Rok danych w BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = klasa ochrony { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = bez nazwy
permits-osm-operator = Zarządca według OSM: { $operator }
permits-osm-enclosing-gap = Obszarów OpenStreetMap, które otaczają cały obszar mapy bez żadnego wierzchołka wewnątrz niego, może tu brakować, podobnie jak obszarów chronionych zapisanych w OSM jako relacja z `type=boundary`. Części z BfN i VG250 nie dotyczy to.
permits-osm-enclosing-gap-short = Obszarów OpenStreetMap, które otaczają cały obszar mapy bez żadnego wierzchołka wewnątrz niego, może tu brakować.
permits-operator-line = { $name } ({ $roles ->
        [operator] zarządca
        [owner] właściciel
       *[both] zarządca i właściciel
    }): { $objects }
permits-column-source = Źródło
permits-column-state = Stan
permits-column-credit = Informacja o źródle i licencja
permits-state-read = odczytano, pobrano dnia { $date }
permits-state-read-osm = odczytano, wyciąg OSM z dnia { $date }
permits-state-unreachable = niedostępne ({ $reason })
permits-state-not-read = nie odczytano
permits-no-credit = nieużyte
permits-hint-city-state = Obszar mapy leży w Berlinie lub Hamburgu. Tam zwykle właściwy jest Bezirksamt (urząd dzielnicy) albo organ kraju związkowego. Te wskazówki nie ustalają jeszcze dzielnicy (Bezirk).
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Obszary chronione podlegają dolnemu organowi ochrony przyrody (untere Naturschutzbehörde). Tutaj jest to samo miasto { $kreis }.
        [no] Obszary chronione podlegają dolnemu organowi ochrony przyrody (untere Naturschutzbehörde), zwykle: { $kreis }.
       *[unknown] Obszary chronione podlegają właściwemu dolnemu organowi ochrony przyrody (untere Naturschutzbehörde), zwykle powiatowi (Landkreis) lub miastu na prawach powiatu (kreisfreie Stadt).
    }
permits-hint-national-park = Park narodowy ma własne zasady. Zapytaj zarząd parku narodowego odpowiednio wcześnie.
permits-hint-nsg-paths = W rezerwacie przyrody zwykle trzeba trzymać się ścieżek. Aby biec poza ścieżkami, zapytaj organ ochrony przyrody.
permits-hint-natura2000 = Obszar FFH lub obszar ochrony ptaków (Natura 2000) dotyka obszaru mapy. Przy imprezie organ ochrony przyrody może zażądać oceny jej oddziaływania.
permits-hint-protected-areas-pl = W obszarze mapy leżą obszary lub obiekty ochrony przyrody. W przypadku parku narodowego, parku krajobrazowego lub rezerwatu zwykle pierwszym kontaktem jest administracja parku lub rezerwatu; w przypadku pozostałych obszarów i obiektów właściwy organ zwykle wskaże Regionalna Dyrekcja Ochrony Środowiska (RDOŚ) właściwa dla województwa. Zapytaj odpowiednio wcześnie, gdy planujesz tam imprezę.
permits-hint-order-office = { $amt ->
        [none] Za ulice, place i inne tereny publiczne zwykle odpowiada Ordnungsamt (urząd porządku publicznego) gminy { $gemeinde }.
       *[other] Za ulice, place i inne tereny publiczne zwykle odpowiada Ordnungsamt (urząd porządku publicznego): { $amt }, dla gminy { $gemeinde }.
    }
permits-hint-park-operator = Zarządca parku lub ogrodu w obszarze mapy według OpenStreetMap: { $operator }. Zapytaj go, zanim z niego skorzystasz.
permits-hint-school-operator = Zarządca szkoły lub instytucji według OpenStreetMap: { $operator }. W sprawie jej terenu zwróć się do dyrekcji szkoły lub tego podmiotu.
permits-section-districts = Bezirke (dzielnice, { $source })
permits-section-green-spaces = Tereny zieleni ({ $source })
permits-section-owners = Grunty kraju związkowego i Federacji (Bund) ({ $source })
permits-section-monuments = Zabytki ({ $source })
permits-section-protected-objects = Pomniki przyrody i inne obiekty chronione ({ $source })
permits-none-districts = Według { $source } obszar mapy nie leży w żadnej dzielnicy (Bezirk).
permits-none-green-spaces = Według { $source } w obszarze mapy nie leży żaden publiczny teren zieleni.
permits-none-owners = Według { $source } w obszarze mapy nie leży żadna działka kraju związkowego ani Federacji (Bund).
permits-none-monuments = Według { $source } w obszarze mapy nie leży żaden zabytek.
permits-none-protected-objects = Według { $source } w obszarze mapy nie leży żaden z obiektów chronionych, które zawiera.
permits-none-sampled = W zapytanych punktach { $source } niczego tu nie wymienia.
permits-gaps = Dane { $source } nie są kompletne: { $gaps }.
permits-sampled = { $source } odpowiada tylko dla pojedynczych punktów. freemapper zapytał w siatce punktów nad mapą: udziały są szacunkami, a to, co leży między punktami, może być pominięte.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = przeznaczenie: { $dedication }
permits-playgrounds-none = W obszarze mapy nie leży żaden publiczny plac zabaw.
permits-playgrounds-one = W obszarze mapy leży jeden publiczny plac zabaw. Nie jest wymieniony osobno.
permits-playgrounds-many = { $count ->
        [one] W obszarze mapy leży { $count } publiczny plac zabaw. Nie jest wymieniony osobno.
        [few] W obszarze mapy leżą { $count } publiczne place zabaw. Nie są wymienione pojedynczo.
        [many] W obszarze mapy leży { $count } publicznych placów zabaw. Nie są wymienione pojedynczo.
       *[other] W obszarze mapy leży { $count } publicznego placu zabaw. Nie są wymienione pojedynczo.
    }
permits-owner-other = Inni właściciele publiczni
permits-owners-rest = Dla reszty obszaru dane nie wskazują właściciela.
permits-monuments-total = Zabytki leżące w całości lub części w obszarze mapy: { $count }.{ $links ->
        [yes] {" "}Odnośniki prowadzą do ich wpisów w wykazie zabytków.
       *[no] {""}
    }
permits-monuments-links-left = Dalsze wpisy ({ $count }) są tu tylko policzone, bez odnośnika.
permits-object-kind = { $kind ->
        [natural-monument] Pomnik przyrody (Naturdenkmal)
        [protected-landscape-part] Chroniony element krajobrazu (geschützter Landschaftsbestandteil)
        [biotope] Prawnie chroniony biotop (geschütztes Biotop)
        [national-park] Park narodowy
        [park-buffer-zone] Otulina parku
        [landscape-park] Park krajobrazowy
        [nature-reserve] Rezerwat przyrody
        [protected-landscape-area] Obszar chronionego krajobrazu
        [natura2000-habitats] Obszar Natura 2000 (siedliska, SOO)
        [natura2000-birds] Obszar Natura 2000 (ptaki, OSO)
        [ecological-site] Użytek ekologiczny
        [landscape-complex] Zespół przyrodniczo-krajobrazowy
        [documentation-site] Stanowisko dokumentacyjne
        [natural-monument-pl] Pomnik przyrody
       *[special-rules] Obszar ze szczególnymi zasadami
    }
permits-object-count = { $count ->
        [one] { $count } obiekt
        [few] { $count } obiekty
        [many] { $count } obiektów
       *[other] { $count } obiektu
    }
permits-object-citations = Podstawy prawne:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Za ulice, place i tereny zieleni w dzielnicy (Bezirk) { $district } zwykle odpowiada Bezirksamt { $district }, a w nim Fachamt Management des öffentlichen Raumes (urząd ds. przestrzeni publicznej).
       *[berlin] Za ulice, place i tereny zieleni w dzielnicy (Bezirk) { $district } zwykle odpowiadają Ordnungsamt (urząd porządku publicznego) oraz Straßen- und Grünflächenamt (urząd dróg i zieleni) w Bezirksamt { $district }.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Za zabytki w obszarze mapy odpowiada Denkmalschutzamt Hamburg (urząd ochrony zabytków).
       *[berlin] Za zabytki w dzielnicy (Bezirk) { $district } odpowiada dolny organ ochrony zabytków (untere Denkmalschutzbehörde) w Bezirksamt { $district }.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Czy w obszarze mapy leżą wpisane zabytki, tutejsze dane nie pozwalają stwierdzić z pewnością.
       *[yes] W obszarze mapy są wpisane zabytki.
    } O tym, czy bieg je dotyka, na przykład punkt kontrolny przy zabytku albo odcinek przez chroniony obiekt, decyduje dolny organ ochrony zabytków (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Tutaj jest to samo miasto { $kreis }.
        [no] , zwykle: { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; w niektórych krajach związkowych jest to gmina (Gemeinde) { $gemeinde }.
        }
       *[unknown] {" "}(zwykle powiat (Landkreis) lub miasto na prawach powiatu (kreisfreie Stadt), w niektórych krajach związkowych gmina (Gemeinde)).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Pomniki przyrody, chronione elementy krajobrazu i chronione biotopy na mapie podlegają dolnemu organowi ochrony przyrody (untere Naturschutzbehörde). Tutaj jest to samo miasto { $kreis }.
        [no] Pomniki przyrody, chronione elementy krajobrazu i chronione biotopy na mapie podlegają dolnemu organowi ochrony przyrody (untere Naturschutzbehörde), zwykle: { $kreis }.
       *[unknown] Pomniki przyrody, chronione elementy krajobrazu i chronione biotopy na mapie podlegają dolnemu organowi ochrony przyrody (untere Naturschutzbehörde), zwykle powiatowi (Landkreis) lub miastu na prawach powiatu (kreisfreie Stadt).
    }
permits-hint-biotope = W obszarze mapy leżą prawnie chronione biotopy{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Zabronione jest wszystko, co może je zniszczyć lub znacznie uszkodzić (§ 30 BNatSchG). Umieszczaj punkty kontrolne i odcinki poza nimi albo zapytaj dolny organ ochrony przyrody (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Za pomniki przyrody i chronione elementy krajobrazu w dzielnicy (Bezirk) { $district } zwykle odpowiada Bezirksamt { $district }.
       *[berlin] Za pomniki przyrody i chronione elementy krajobrazu w dzielnicy (Bezirk) { $district } odpowiada dolny organ ochrony przyrody (untere Naturschutzbehörde) w Bezirksamt { $district }.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Według otwartych danych kraju związkowego { $share } obszaru mapy należy do innych właścicieli publicznych.
       *[no] Według otwartych danych kraju związkowego { $share } obszaru mapy należy do właściciela "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Właścicieli reszty obszaru nie da się ustalić z otwartych danych. Mając uzasadniony interes (na przykład jako organizator), możesz poprosić urząd ksiąg wieczystych (Grundbuchamt) o wypis; w sprawie gruntów publicznych pomoże właściwy Ordnungsamt lub urząd zieleni.
       *[no] Właścicieli tych obszarów nie da się ustalić z otwartych danych. Mając uzasadniony interes (na przykład jako organizator), możesz poprosić urząd ksiąg wieczystych (Grundbuchamt) o wypis; w sprawie gruntów publicznych pomoże właściwy Ordnungsamt lub urząd zieleni.
    }
permits-section-forest-units = Administracja leśna ({ $source })
permits-section-state-forest = Lasy państwowe kraju związkowego ({ $source })
permits-none-forest-units = Według { $source } obszar mapy nie leży w obszarze żadnego urzędu leśnego.
permits-section-admin-units = Jednostki administracyjne ({ $source })
permits-none-admin-units = Według { $source } obszar mapy nie leży w żadnej jednostce administracyjnej rejestru.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Według { $source } w obszarze mapy nie leży żaden las państwowy kraju związkowego.
permits-forest-units-reach = Urząd leśny odpowiada za las na swoim obszarze. Jego obszar może obejmować także miasta i pola.
permits-state-forest-office = właściwy: { $office }
permits-hint-forest-office = { $units ->
        [none] Urząd leśny właściwy dla lasu w obszarze mapy: { $office } ({ $administration }). Zapytaj tam odpowiednio wcześnie, gdy planujesz imprezę w lesie.
       *[other] Urząd leśny właściwy dla lasu w obszarze mapy: { $office } ({ $administration }), { $units }. Zapytaj tam odpowiednio wcześnie, gdy planujesz imprezę w lesie.
    }
permits-hint-forest-owner = { $office ->
        [known] Impreza w lesie wymaga zgody właścicieli lasu. Urząd leśny zwykle może powiedzieć, kto jest właścicielem. W kilku krajach związkowych zorganizowaną imprezę w lesie trzeba ponadto zgłosić organowi leśnemu lub uzyskać jego zezwolenie.
       *[unknown] Impreza w lesie wymaga zgody właścicieli lasu. W kilku krajach związkowych zorganizowaną imprezę w lesie trzeba ponadto zgłosić organowi leśnemu lub uzyskać jego zezwolenie. Te wskazówki nie wskazują właściwego tu urzędu leśnego; może to powiedzieć administracja leśna kraju związkowego lub dolny organ leśny.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Impreza w lesie zwykle wymaga zgody właściciela lasu. Wymienione wyżej nadleśnictwo zwykle może powiedzieć, kto opiekuje się lasem. Lasu, który nie należy do Lasów Państwowych, na przykład lasu prywatnego, nie ma w danych tych wskazówek.
       *[unknown] Impreza w lesie zwykle wymaga zgody właściciela lasu. Te wskazówki nie wskazują dla obszaru mapy żadnego nadleśnictwa Lasów Państwowych; dane obejmują tylko Lasy Państwowe, więc las może należeć do kogoś innego, na przykład do właściciela prywatnego. Właściciela trzeba ustalić osobno.
    }
permits-hint-gmina-nature-pl = { $gmina ->
        [none] W obszarze mapy leży pomnik przyrody, użytek ekologiczny, zespół przyrodniczo-krajobrazowy lub stanowisko dokumentacyjne. Ustanawia je rada gminy w drodze uchwały, która określa, kto sprawuje nadzór nad obiektem i jakie zakazy obowiązują (art. 44 ustawy o ochronie przyrody). Sama ustawa nie przewiduje tu żadnego zezwolenia na imprezę. Zapytaj w urzędzie gminy, na której terenie leży obiekt, o uchwałę i osobę sprawującą nadzór.
       *[other] W obszarze mapy leży pomnik przyrody, użytek ekologiczny, zespół przyrodniczo-krajobrazowy lub stanowisko dokumentacyjne. Ustanawia je rada gminy w drodze uchwały, która określa, kto sprawuje nadzór nad obiektem i jakie zakazy obowiązują (art. 44 ustawy o ochronie przyrody). Sama ustawa nie przewiduje tu żadnego zezwolenia na imprezę. Zapytaj w urzędzie gminy ({ $gmina }), na której terenie leży obiekt, o uchwałę i osobę sprawującą nadzór.
    }
permits-hint-gmina-roads-pl = Impreza, która utrudnia ruch lub wymaga korzystania z drogi w sposób szczególny, wymaga zezwolenia na podstawie art. 65 Prawa o ruchu drogowym; wniosek składa się co najmniej 30 dni wcześniej. Zezwolenie wydaje organ zarządzający ruchem na drodze: starosta dla dróg powiatowych i gminnych, prezydent miasta w mieście na prawach powiatu, marszałek województwa lub GDDKiA dla pozostałych dróg (art. 10 i 65b). Zarządcą drogi gminnej jest wójt, burmistrz lub prezydent miasta ({ $gmina }); zarządca decyduje o zajęciu pasa drogowego (art. 19 i 40 ustawy o drogach publicznych). Te wskazówki nie pokazują kategorii dróg.
permits-hint-gmina-land-pl = Jeśli grunt w obszarze mapy należy do gminy, na przykład park miejski, plac, teren szkoły lub las, zarządza nim wójt, burmistrz lub prezydent miasta, który reprezentuje gminę (art. 30 i 31 ustawy o samorządzie gminnym). Zapytaj o zgodę w urzędzie gminy ({ $gmina }), zanim z niego skorzystasz. Te wskazówki nie pokazują, do kogo należy grunt.
permits-section-water-protection = Strefy ochrony wód ({ $source })
permits-section-forest-functions = Funkcje lasu i lasy ochronne ({ $source })
permits-none-water-protection = Według { $source } w obszarze mapy nie leży żadna strefa ochrony wód.
permits-none-forest-functions = Według { $source } w obszarze mapy nie leży ani las wypoczynkowy, ani ochronny, ani naturalny, ani zamknięty obszar leśny.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Las wypoczynkowy (Erholungswald)
        [protected] Las ochronny (Bannwald lub Schutzwald)
        [natural] Las naturalny lub rezerwat ścisły (Naturwald, Totalreservat)
       *[closed] Zamknięty obszar leśny
    }
permits-hint-water-protection = { $inner ->
        [yes] Strefa { $zone } strefy ochrony wód "{ $name }" leży w obszarze mapy. W strefach I i II imprezy są zwykle wykluczone. Zapytaj dolny organ ds. wód (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] Strefa { $zone } strefy ochrony wód "{ $name }" leży w obszarze mapy. Imprezy są tam zwykle możliwe; dolny organ ds. wód (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) może powiedzieć, na co pozwala rozporządzenie.
       *[unknown] Strefa ochrony wód "{ $name }" leży w obszarze mapy. Dane nie mówią, która to strefa. W strefach I i II imprezy są zwykle wykluczone; zapytaj dolny organ ds. wód (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Las wypoczynkowy (Erholungswald) leży w obszarze mapy{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. W sprawie imprez tam zapytaj organ leśny.
        [protected] Las ochronny (Bannwald lub Schutzwald) leży w obszarze mapy{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Zapytaj organ leśny, zanim zaplanujesz tam imprezę.
        [natural] Las naturalny lub rezerwat ścisły leży w obszarze mapy{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Opuszczanie ścieżek jest tam często zabronione; nie umieszczaj w nim punktów kontrolnych bez zapytania organu leśnego.
       *[closed] Zamknięty obszar leśny, do którego nie wolno wchodzić, leży w obszarze mapy{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Wyłącz go z trasy.
    }
permits-section-roads = Drogi sklasyfikowane według OpenStreetMap
permits-none-roads = Według OpenStreetMap przez obszar mapy nie biegnie żadna Autobahn ani Bundes-, Landes-, Staats- czy Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (autostrada)
        [bundesstrasse] Bundesstraße (droga federalna)
        [landesstrasse] Landesstraße lub Staatsstraße (droga krajowa związkowa)
       *[kreisstrasse] Kreisstraße (droga powiatowa)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } biegnie przez obszar mapy. Trasa może ją przekraczać tylko mostami lub przejściami podziemnymi.
        [bundesstrasse] Bundesstraße { $refs } biegnie przez obszar mapy. Zamknięcie lub przekraczanie z asystą porządkową zarządza organ ruchu drogowego (Straßenverkehrsbehörde); drogę zwykle utrzymuje Federacja (Bund) przez administrację drogową kraju związkowego, w większych miastach często miasto.
        [landesstrasse] Landesstraße lub Staatsstraße { $refs } biegnie przez obszar mapy. Zamknięcie lub przekraczanie z asystą porządkową zarządza organ ruchu drogowego (Straßenverkehrsbehörde); drogę zwykle utrzymuje kraj związkowy, w większych miastach często miasto.
       *[kreisstrasse] Kreisstraße { $refs } biegnie przez obszar mapy. Zamknięcie lub przekraczanie z asystą porządkową zarządza organ ruchu drogowego (Straßenverkehrsbehörde); drogę zwykle utrzymuje powiat (Kreis), w większych miastach często miasto.
    }
permits-hint-state-forest = { $office ->
        [none] Według otwartych danych kraju związkowego { $share } obszaru mapy to las państwowy właściciela "{ $owner }".
       *[other] Według otwartych danych kraju związkowego { $share } obszaru mapy to las państwowy właściciela "{ $owner }". Właściwy: { $office }.
    }
