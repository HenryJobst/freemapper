# freemapper: podręcznik użytkownika (wersja skrócona po polsku)

[English](en.md) · [Deutsch](de.md) · [Słownik](../glossary.md) · [README](../../README.en.md)

Ten podręcznik jest przeznaczony dla kartografów i trenerów, którzy chcą przygotować mapę do biegu na orientację
(BnO) bez pracy z danymi geodezyjnymi. Jest skróconą wersją podręcznika angielskiego ([en.md](en.md)) i obejmuje to,
czego potrzebuje użytkownik w Polsce. Numeracja rozdziałów odpowiada numeracji podręcznika angielskiego, więc
odsyłacze z innych dokumentów można łatwo dopasować. Rozdziały, które w tej wersji pominięto, są w podręczniku
angielskim.

**Polska jest w wersji podglądowej.** Mapa w Polsce powstaje na razie wyłącznie z danych OpenStreetMap (OSM). Nie ma
jeszcze warstwic, ortofotomapy, budynków ani użytków z ewidencji gruntów i budynków (EGiB). Szczegóły są w
[rozdziale 13](#13-regiony-i-obszary-przygraniczne).

## Spis treści

1. [Czym jest freemapper](#1-czym-jest-freemapper)
2. [Czego potrzeba](#2-czego-potrzeba)
3. [Instalacja](#3-instalacja)
4. [Droga A: z asystentem AI](#4-droga-a-z-asystentem-ai)
5. [Droga B: bez asystenta](#5-droga-b-bez-asystenta)
6. [Wybór obszaru mapy w przeglądarce](#6-wybór-obszaru-mapy-w-przeglądarce)
7. [Wynik](#7-wynik)
8. [Co pokazują szablony](#8-co-pokazują-szablony)
9. [Co sprawdzić w terenie](#9-co-sprawdzić-w-terenie)
10. [Mapa jako PDF](#10-mapa-jako-pdf)
11. [OCAD](#11-ocad)
12. [Wskazówki dotyczące zgód](#12-wskazówki-dotyczące-zgód)
13. [Regiony i obszary przygraniczne](#13-regiony-i-obszary-przygraniczne)
14. [Dalsza praca](#14-dalsza-praca)
15. [Komunikaty i co robić](#15-komunikaty-i-co-robić)
16. [Informacje o źródłach danych](#16-informacje-o-źródłach-danych)
17. [Pomoc i zgłaszanie błędów](#17-pomoc-i-zgłaszanie-błędów)

## 1. Czym jest freemapper

freemapper pobiera otwarte dane geodezyjne obszaru i przygotowuje **punkt wyjścia** do mapy dziedzińca szkolnego
(1:1000), mapy sprintowej (1:4000) albo mapy leśnej (od 1:15 000 do 1:7500). Do danych należą ortofotomapa,
numeryczny model terenu (NMT) oraz budynki i użytki z ewidencji. Uzupełniają je dane z OpenStreetMap. Który z tych
elementów jest dostępny w danym regionie, zależy od regionu (rozdział 13).

- **Jako gotowe obiekty mapy** rysowane jest tylko to, co jest wiarygodne: budynki, wody, a na mapach sprintowych
  także nawierzchnie utwardzone (ulice, place) i drogi nieutwardzone o znanej nawierzchni.
- **Jako szablony** podawane jest wszystko, co trzeba ocenić samodzielnie: ortofotomapa, cieniowanie rzeźby,
  nachylenie, rzeźba lokalna, wysokość nad terenem (nDSM), warstwice, kopce i skarpy z modelu terenu, ogrodzenia,
  żywopłoty, ławki, urządzenia placów zabaw, obszary zakazane, stawy, strumienie, fontanny i mosty.
- Mapę rysujesz i sprawdzasz w terenie sam, w programie **OpenOrienteering Mapper (OOM)** albo **OCAD**.

Na życzenie freemapper zapisuje też **gotową do druku mapę w PDF**, **wydruk kontrolny** ze wszystkimi kandydatami
oraz **mapę treningową**, którą drukuje się i używa bez zmian (rozdział 10).

Rodzaje map:

| Rodzaj (`--kind`) | Norma | Skala |
|---|---|---|
| `schoolyard` (mapa dziedzińca szkolnego) | zestaw znaków IOF dla szkół | 1:1000 |
| `sprint` (mapa sprintowa) | ISSprOM 2019 | 1:4000 (domyślnie) lub 1:5000 |
| `forest` (mapa leśna) | ISOM 2017-2 | 1:15 000 (domyślnie), 1:10 000 lub 1:7500 |

Mapa leśna jest przeznaczona dla małych lasów, w których potrzebna jest szybko mapa treningowa. Ścieżki pochodzą w
niej z OSM.

## 2. Czego potrzeba

- Komputer z systemem Windows, macOS lub Linux. Programy GIS ani inne narzędzia nie są potrzebne.
- [OpenOrienteering Mapper](https://www.openorienteering.org/apps/mapper/) (bezpłatny) albo OCAD 12 lub nowszy.
- Połączenie z internetem przy pierwszym pobieraniu.
- Wolne miejsce na dysku:
  - Przy pierwszej mapie w danym województwie freemapper pobiera wyciąg OSM tego województwa, około 60 do 300 MB.
    Wyciąg jest używany przez wszystkie projekty.
  - Poza tym każda mapa potrzebuje zwykle 10 do 100 MB.
  - Przed każdym pobieraniem freemapper sprawdza wolne miejsce w pamięci podręcznej i w folderze projektu i
    zostawia 1 GB wolnego miejsca na każdym dysku. Jeśli nie wszystko się mieści, pomija opcjonalne części i
    ostrzega o każdej z nich. Jeśli nie mieszczą się dane niezbędne, zatrzymuje się przed pobieraniem.
- Obszar w **Polsce** (podgląd, patrz rozdział 13) albo w jednym z niemieckich krajów związkowych, które mają
  oficjalne źródła danych (rozdział 13). Obszar musi leżeć w całości w jednym kraju.
- Opcjonalnie asystent AI, na przykład Claude, żeby nie wpisywać poleceń.

## 3. Instalacja

freemapper to pojedynczy program bez instalatora.

1. Na [stronie wydań](https://github.com/HenryJobst/freemapper/releases/latest) pobierz archiwum dla swojego systemu:
   - **Windows:** `freemapper-…-windows-x86_64.zip`
   - **Mac z procesorem Apple** (M1 i nowsze): `freemapper-…-macos-arm64.tar.gz`
   - **Mac z procesorem Intel:** `freemapper-…-macos-x86_64.tar.gz`
   - **Linux:** `freemapper-…-linux-x86_64.tar.gz` (dla ARM: `freemapper-…-linux-arm64.tar.gz`)

   Pliki `.gz` i `freemapper-update.json` na tej stronie służą do `freemapper update`, nie są potrzebne przy
   pierwszej instalacji. Obok każdego archiwum leży plik `.sha256` do sprawdzenia sumy kontrolnej.
2. Rozpakuj archiwum i umieść folder w stałym miejscu, na przykład `Documents/freemapper`.
3. Przy pierwszym uruchomieniu system ostrzeże, ponieważ program nie jest podpisany:
   - **Windows:** "Windows protected your PC" → *More info* → *Run anyway*.
   - **macOS:** "cannot be opened …" → *System Settings › Privacy & Security* → *Open Anyway*. Można też usunąć
     znacznik kwarantanny poleceniem `xattr -d com.apple.quarantine freemapper`.
   - **Linux:** w razie potrzeby `chmod +x freemapper`.
4. Sprawdź instalację: `./freemapper --version` wypisuje `freemapper <wersja>`.

Zapisz **pełną ścieżkę** do programu, na przykład `C:\Users\Anna\Documents\freemapper\freemapper.exe` albo
`/Users/anna/Documents/freemapper/freemapper`. Asystent będzie jej potrzebował.

Oprócz programu folder zawiera pliki README, licencję, podręczniki i `AGENTS.md`, czyli instrukcje dla asystentów AI
(rozdział 4).

### Aktualizacja

`freemapper update` zastępuje program najnowszą wersją. Ścieżka się nie zmienia, więc asystent, który zna program po
ścieżce, działa dalej. Wersje 0.11.0 i starsze nie potrafią się aktualizować: pobierz raz najnowsze archiwum jak w
kroku 1.

```sh
./freemapper update
```

Polecenie pokazuje nową wersję, datę wydania, odsyłacz do notatek wydania i rozmiar pobierania, a potem pyta
`Install it now? [y/N]`. Instaluje tylko odpowiedź `y` lub `yes`.

| Wywołanie | Działanie |
|---|---|
| `freemapper update --check` | Tylko sprawdza. Kod wyjścia 10 oznacza, że jest nowsza wersja, 0 że ta jest aktualna. |
| `freemapper update --yes` | Instaluje bez pytania. Potrzebne w skryptach i wszędzie, gdzie nie ma terminala. |
| `freemapper update --version 0.12.0` | Instaluje wskazane wydanie, także starsze. |
| `freemapper update --rollback` | Wraca do wersji, którą ostatnia aktualizacja zastąpiła. |
| `freemapper update --mode notify` | Zapisuje, co robi codzienne sprawdzanie: `notify`, `install` albo `off`. |

Każde wydanie ma listę plików programu z sumami SHA-256, podpisaną kluczem minisign projektu. freemapper instaluje
tylko wtedy, gdy podpis i sumy się zgadzają. Zastąpiony program zostaje obok jako `freemapper.previous` (Windows:
`freemapper.previous.exe`). freemapper musi mieć prawo zapisu w swoim folderze; nie prosi o uprawnienia
administratora. Program zbudowany ze źródeł nie aktualizuje się sam.

**Codzienne sprawdzanie.** Po `find`, `init`, `select`, `fetch`, `run` i `permits` freemapper sprawdza co najwyżej raz
dziennie, czy jest nowsze wydanie. Jeśli jest, po wyniku polecenia pojawia się jedna linia na stderr:

```text
freemapper 0.12.0 is available (you have 0.11.0). Update with: freemapper update
```

Sprawdzania nie ma z `--offline`, przy ustawionej zmiennej środowiskowej `CI`, w trybie `off` i w programie
zbudowanym ze źródeł. Tryb zapisany poleceniem `freemapper update --mode install` leży w pliku `settings.toml`:

- Windows: `%APPDATA%\freemapper\settings.toml`
- macOS: `~/Library/Application Support/freemapper/settings.toml`
- Linux: `~/.config/freemapper/settings.toml` (albo `$XDG_CONFIG_HOME/freemapper/settings.toml`)

Zmienna `FREEMAPPER_UPDATE` (`off`, `notify`, `install`) ma pierwszeństwo przed plikiem. Sprawdzanie pyta tylko
github.com o dwa małe pliki najnowszego wydania. Każde żądanie niesie wyłącznie stały nagłówek `User-Agent`
`freemapper/<wersja> (+https://github.com/HenryJobst/freemapper)`. freemapper nie wysyła identyfikatora, danych
projektu ani niczego o użytkowniku.

## 4. Droga A: z asystentem AI

Asystent obsługuje freemapper za Ciebie. W tym celu freemapper działa jako serwer MCP (`freemapper mcp`). Zarejestruj
go raz, podając pełną ścieżkę:

```sh
claude mcp add freemapper -- /pelna/sciezka/do/freemapper mcp
```

Dla innych asystentów (Claude Desktop, Codex) wpis konfiguracji jest w pliku `AGENTS.md` w folderze programu. Potem
opisujesz asystentowi, jaką mapę chcesz i dla jakiego miejsca. Asystent wyszukuje miejsce, podaje wielkość pobierania,
czeka na Twoją zgodę i uruchamia budowanie mapy. Jeśli asystent potrafi uruchamiać polecenia na Twoim komputerze,
wystarczy mu wiersz poleceń (rozdział 5), który jest szybszy niż MCP. Zalecenia dla asystenta są w `AGENTS.md`.

Serwer MCP sprawdza raz dziennie, czy jest nowsze wydanie, ale nigdy nie instaluje samodzielnie: asystent musi
najpierw zapytać Ciebie. Po aktualizacji uruchom serwer MCP ponownie (w Claude Desktop zamknij i otwórz program).

## 5. Droga B: bez asystenta

Nie da się całkiem bez wiersza poleceń, ale wystarczą dwa polecenia. Otwórz terminal (w Windows PowerShell) w folderze
z freemapper.

1. Wybierz obszar w przeglądarce. To tworzy też folder projektu:

   ```sh
   ./freemapper select szkola
   ```

   W Windows: `.\freemapper.exe select szkola`.
2. Pobierz dane, oblicz je i zapisz mapę:

   ```sh
   ./freemapper run szkola
   ```

   `--ocad` zapisuje też plik OCAD, `--pdf` mapę jako PDF, `--pdf-candidates` wydruk kontrolny z kandydatami,
   `--training` mapę treningową.

Jeśli znasz obiekt OSM terenu, możesz pominąć przeglądarkę:

```sh
./freemapper init szkola --osm way/23838018 --title "Szkoła Podstawowa nr 1"
./freemapper run szkola
```

`find` wyszukuje identyfikator po nazwie. Wypisuje do pięciu miejsc z OpenStreetMap, najpierw te z powierzchnią, i
podaje polecenie `init` dla pierwszego z nich:

```sh
./freemapper find "Szkoła Podstawowa nr 1 Słubice"
```

Wiersz z napisem "no area, cannot be a map area" to punkt w OSM (na przykład przystanek nazwany od szkoły) i nie może
być obszarem mapy. Jeśli szukanego miejsca nie ma na liście, spróbuj dłuższej nazwy z miejscowością albo narysuj
obszar poleceniem `select`. Gdy nazwa jest jednoznaczna, `init --place "<nazwa>"` pomija listę i bierze pierwsze
trafienie z powierzchnią.

Jeśli teren w OSM składa się z kilku oddzielnych powierzchni (relacja multipolygon z kilkoma obwiedniami zewnętrznymi),
freemapper wypisuje części z ich rozmiarami. `--part 1` bierze największą, `--part 2` następną. Zewnętrzne linie takiej
relacji często nie mają znaczników, więc `--osm way/…` nic nie znajduje: użyj relacji (`--osm relation/…`) albo zapisz
obrys jako GeoJSON i użyj `--geojson`.

Pozostałe polecenia:

| Polecenie | Działanie |
|---|---|
| `find <zapytanie>` | Wyszukuje do 5 miejsc w OSM. `--json` wypisuje tablicę JSON. |
| `init <folder>` z `--geojson <plik>`, `--osm way/<id>` (z `--part <n>`) albo `--place <nazwa>` | Tworzy projekt. `--kind schoolyard\|sprint\|forest`, `--scale`, `--title`. GeoJSON zawiera jeden wielokąt w EPSG:4326. |
| `select <folder>` | Otwiera stronę wyboru obszaru w przeglądarce. `--no-browser` tylko wypisuje adres. `--training` zaznacza pole "Training map". |
| `run <folder>` | `fetch`, `derive` i `export` w jednym kroku. |
| `fetch <folder>` | Pobiera dane do wspólnej pamięci podręcznej. `--offline` używa tylko pamięci podręcznej. |
| `derive <folder>` | Oblicza szablony i warstwice. |
| `export <folder>` | Zapisuje mapę (także `--training`). `--force` zastępuje istniejącą mapę (stara zostaje jako `.bak`). |
| `permits <folder>` | Wskazówki dotyczące zgód (rozdział 12). |
| `update` | Instaluje najnowsze wydanie (rozdział 3). |

Przełączniki `run` i `export`: `--ocad`, `--pdf`, `--pdf-candidates`, `--training`, `--pdf-scale-bar`,
`--pdf-sources auto|full|credit-only`. `init`, `export` i `run` przyjmują też `--content safe|balanced|full`
(co trafia od razu na mapę; domyślnie `balanced`) oraz `--content-set <grupa>=map|candidates|off`. `init`, `derive` i
`run` przyjmują `--contour-interval <metry>|auto`. Wszystkie polecenia, które pobierają dane, przyjmują
`--cache-dir <folder>`. `freemapper --help` i `freemapper <polecenie> --help` wypisują wszystkie przełączniki.

**Skala.** `--scale` przyjmuje tylko wartości, które drukuje freemapper-print: 1000 dla map dziedzińca szkolnego,
4000 (domyślnie) lub 5000 dla map sprintowych, 15000 (domyślnie), 10000 lub 7500 dla map leśnych. `init` odrzuca skalę
3000 dla sprintu: rysuj w 1:4000, a freemapper-print powiększa do 1:3000 przy druku.

**Tytuł.** `init` zawsze zapisuje `title` w `freemapper.toml`. Bez `--title` jest to nazwa folderu pisana wielkimi
literami wyrazów. Tytuł można zmienić w pliku.

**Jeden projekt, jedno uruchomienie.** Na folder projektu działa naraz tylko jedno uruchomienie. Istniejąca mapa nie
jest nadpisywana bez `--force`, ponieważ może zawierać Twoją pracę.

## 6. Wybór obszaru mapy w przeglądarce

`freemapper select <folder>` otwiera lokalną stronę, na której rysujesz obszar mapy albo wybierasz obiekt OSM i
zapisujesz go. Zapis tworzy projekt. Na stronie można też ustawić rodzaj mapy, skalę, tytuł i produkty wyjściowe
(PDF, OCAD, mapa treningowa, wskazówki dotyczące zgód). Podkład to mapa OpenStreetMap; dla Polski strona pokazuje
ortofotomapę GUGiK. Strona działa tylko na Twoim komputerze i nic nie instaluje. Obszar nie może mieć więcej niż
4000 m z boku (wraz z buforem).

## 7. Wynik

Folder projektu zawiera między innymi:

| Plik | Co to jest |
|---|---|
| `<nazwa>.omap` | mapa dla OpenOrienteering Mapper, georeferencjonowana, z pewnymi obiektami i informacjami pozaramkowymi |
| `templates/` | szablony: ortofotomapa, cieniowanie rzeźby, nachylenie, rzeźba lokalna, nDSM i siedem warstw kandydatów |
| `ocad/` | z `--ocad`: mapa OCAD i jej szablony |
| `<nazwa>.pdf` | z `--pdf`: mapa gotowa do druku, bez kandydatów |
| `<nazwa>-candidates.pdf` | z `--pdf-candidates`: wydruk kontrolny, mapa ze wszystkimi kandydatami |
| `<nazwa>-training.omap`, `<nazwa>-training.pdf` | z `--training`: mapa treningowa (rozdział 10), z `--ocad` także `ocad/<nazwa>-training.ocd` |
| `<nazwa>-preview.webp` | obraz obiektów mapy (bez kandydatów), pokazywany na stronie wyboru obszaru |
| `ATTRIBUTION.txt` | informacje o źródłach danych |
| `freemapper.toml` | ustawienia projektu: obszar, rodzaj, skala, użyte dane |
| `data/` | pobrane obiekty |
| `permits.md`, `permits.json` | z `--permits` lub `freemapper permits`: wskazówki dotyczące zgód (rozdział 12) |

Otwórz plik `.omap` w OOM. Szablony są już podłączone; włączasz i wyłączasz je w menu *Templates*. **Kandydaci**
leżą na wierzchu w siedmiu mapach szablonowych, po jednej na rodzaj obiektów, żeby można było oglądać jeden rodzaj
naraz:

| Warstwa | Plik | Zawiera |
|---|---|---|
| terrain | `templates/candidates-terrain.omap` | skarpy, urwiska, kopce i zagłębienia, warstwice |
| water | `templates/candidates-water.omap` | potoki, rowy, źródła, fontanny, stawy |
| vegetation | `templates/candidates-vegetation.omap` | tereny otwarte, powierzchnie roślinności, drzewa, żywopłoty |
| paths | `templates/candidates-paths.omap` | ścieżki, ulice, nawierzchnie utwardzone, schody, przejścia, mosty |
| barriers | `templates/candidates-barriers.omap` | mury, ogrodzenia |
| features | `templates/candidates-features.omap` | wieże, pomniki, ławki i inne wyposażenie, budynki, wiaty |
| out-of-bounds | `templates/candidates-out-of-bounds.omap` | obszary, na które nie wolno wchodzić |

Warstwy startują ukryte, więc mapa otwiera się tylko z pewnymi obiektami. W liście szablonów (menu *Templates*)
włączasz każdą warstwę polem wyboru i ustawiasz jej przezroczystość (na początku 70 %). Każda warstwa ma też obrys
obszaru mapy. Sprawdź kandydatów i przejmij to, co jest poprawne: obrysuj ich albo otwórz plik warstwy i skopiuj
obiekty. Wszystkie te mapy mają to samo georeferencjonowanie.

**Czego mapa bazowa nie zawiera.** Na mapie bazowej nic nie jest przesuwane: każdy obiekt leży tam, gdzie ma go
źródło. Linia, która leży na znaku o wyższej randze i byłaby przez niego zakryta, przechodzi do warstwy kandydatów
swojego rodzaju z znacznikiem `freemapper:generalized` i podaniem powodu. To samo dotyczy obiektów poniżej minimalnego
rozmiaru z normy. Linie jednego źródła, które schodzą się końcami, są łączone w jedną. Mapa i jej warstwy kandydatów
razem zawierają każdy obiekt.

### Mapy leśne

Mapa leśna (`--kind forest`) jest zgodna z ISOM 2017-2:

```sh
freemapper init las --osm relation/4513841 --kind forest --scale 10000
freemapper run las --training
```

Ścieżki pochodzą z OSM i trafiają od razu na mapę (na przykład drogi główne jako 502, inne drogi jako 503, drogi
leśne jako 504, wąskie ścieżki jako 506, słabo widoczne jako 507, schody jako 532). W skali 1:10 000 freemapper
rysuje znaki w 150 %, w 1:7500 w 200 %, jak przewiduje norma dla map powiększonych. W Polsce mapa leśna ma ścieżki z OSM,
a warstwice z modelu terenu GUGiK (rozdział 13).

## 8. Co pokazują szablony

- **Ortofotomapa**: zdjęcie lotnicze obszaru.
- **Cieniowanie rzeźby** (hillshade): model terenu oświetlony z boku, dobrze pokazuje małe skarpy i stopnie.
- **Nachylenie** (slope): stromość terenu.
- **Rzeźba lokalna** (local relief): odchylenie wysokości od otoczenia, uwypukla kopce i zagłębienia.
- **nDSM**: wysokość obiektów nad terenem (budynki, drzewa), z różnicy modelu pokrycia terenu (NMPT) i modelu
  terenu (NMT).

Szablony i wszystko, co z nich wynika, istnieją tylko tam, gdzie region dostarcza dane rastrowe. Dla Polski patrz
rozdział 13. **Co trafia od razu na mapę** zależy od ustawienia `--content`: `safe` (tylko najpewniejsze obiekty),
`balanced` (domyślnie, także obiekty z ewidencji i rejestrów) albo `full`. Wybór jest zapisywany w `freemapper.toml`.

## 9. Co sprawdzić w terenie

freemapper oszczędza rysowanie podkładu, ale nie spaceru po terenie. Sprawdź przede wszystkim:

- **Budynki:** czy obrys i przejścia są poprawne, czy brakuje nowych budynków, czy rozebrane jeszcze stoją.
- **Nawierzchnie utwardzone** (sprint): czy granice i place zgadzają się z ortofotomapą.
- **Wiaty i przejścia:** czy wiata istnieje, czy przejście jest otwarte, czy ma bramę.
- **Ogrodzenia, mury, żywopłoty:** pochodzą z OSM i z nDSM. Czy można je przekroczyć, czy są bramy?
- **Obszary zakazane:** tereny prywatne, ogrody działkowe, tereny kolejowe, a na mapach dziedzińca szkolnego wszystko
  poza terenem szkoły.
- **Brzegi i mosty:** linia brzegu przebiega przez mosty i trzeba ją tam przeciąć ręcznie. Woda z OSM jest tylko
  kandydatem: czy staw i fontanna jeszcze istnieją, czy rów niesie wodę?
- **Zredukowane punkty:** gdy dwa znaki punktowe leżałyby bliżej, niż norma dopuszcza, zostaje większy, a pominięte
  są wymienione w znaczniku `freemapper:generalized`. Sprawdź odstępy w terenie.
- **Grupy drzew jako powierzchnie:** trzy lub więcej drzew blisko siebie może tworzyć powierzchnię (na przykład las
  405 albo teren otwarty z pojedynczymi drzewami 402). Porównaj obrys z ortofotomapą.
- **Wyposażenie i place zabaw:** pochodzi z OSM i jest często niepełne.
- **Warstwice:** małe skarpy i stopnie widać lepiej w cieniowaniu rzeźby, nachyleniu lub rzeźbie lokalnej; kandydaci
  na kopce i skarpy na nie wskazują.

Na mapach w Polsce, które powstają z samego OSM, kandydaci pochodzą z tabel OSM. Obiekty OSM bywają niekompletne lub
nieaktualne, więc sprawdzaj w terenie więcej niż na mapach z danych urzędowych.

## 10. Mapa jako PDF

Trzy przełączniki zapisują mapę gotową do druku w A4 lub A3, dwa kolejne ustawiają podziałkę liniową i wiersze ze
źródłami:

| Przełącznik | Plik | Zawartość |
|---|---|---|
| `--pdf` | `<nazwa>.pdf` | mapa: dokładnie obiekty pliku OOM, bez kandydatów |
| `--pdf-candidates` | `<nazwa>-candidates.pdf` | wydruk kontrolny: mapa i wszyscy kandydaci, zgodnie z położeniem |
| `--training` | `<nazwa>-training.omap`, `<nazwa>-training.pdf` | mapa treningowa: mapa i kandydaci, uporządkowani do druku i biegu |
| `--pdf-scale-bar` | | rysuje podziałkę liniową na PDF |
| `--pdf-sources auto\|full\|credit-only` | | wybiera wiersze ze źródłami na PDF (domyślnie `auto`) |

Przełączniki można łączyć; pliki leżą wtedy obok siebie w folderze projektu. Wybór nie jest zapisywany w projekcie,
obowiązuje dla danego wywołania.

Kolory to wartości CMYK według IOF Printing and Colour Definitions Rev. 4, takie same jak w plikach OOM i OCAD.
Jeśli drukarka drukuje brąz lub zieleń wyraźnie inaczej, porównaj z IOF Print Test Sheet i skoryguj kolory.

**Wiersze ze źródłami.** Źródła z licencjami są pod blokiem tytułowym. Z `auto` freemapper drukuje do pięciu wierszy
w całości, dłuższą listę skraca. Wiersze dla OpenStreetMap, dla innych licencji i dla freemapper zostają w całości.
`--pdf-sources full` nigdy nie skraca, `--pdf-sources credit-only` drukuje tylko wiersz freemapper (wtedy źródła
podajesz sam). Wiersz freemapper z adresem projektu jest zawsze obecny.

**Polskie litery w PDF.** Czcionka PDF (Helvetica, WinAnsi) nie ma wszystkich polskich liter. Litery, których brak,
drukują się jako litera, od której się je pisze (`ł` jako `l`). Dotyczy to też plików OCAD, w nazwach plików i
tekstach. Wiersze ze źródłami są więc w PDF pisane bez niektórych znaków diakrytycznych.

### Wydruk kontrolny

Na wydruku kontrolnym każdy kandydat staje się obiektem mapy, także niepewny. Pominięte są tylko punkty wewnątrz
kandydata na obszar zakazany, nieosiągalne przejścia i urwiska usunięte przy generalizacji. Każdy obiekt leży tam,
gdzie ma go źródło, także tam, gdzie znaki się zakrywają. Weź wydruk w teren, żeby sprawdzić mapę i kandydatów.

### Mapa treningowa

Mapa treningowa to osobna mapa obok mapy bazowej (`<nazwa>.omap`). Drukuje się ją i biegnie na niej bez zmian.
freemapper bierze obiekty mapy i wszystkich widocznych kandydatów i porządkuje je tak, żeby żaden znak nie leżał na
innym, a odstępy z normy były zachowane:

- łączy linie, wycina lub pomija obiekt niższej rangi tam, gdzie dwa leżą zbyt blisko (mury i nieprzekraczalne
  ogrodzenia są tylko przycinane);
- pomija obiekty poniżej minimalnego rozmiaru z normy, wyposażenie ulic na ulicach i placach oraz szczegóły wewnątrz
  oliwkowozielonych obszarów;
- rysuje park na żółto tylko tam, gdzie nDSM nie pokazuje drzew (tylko tam, gdzie istnieje nDSM);
- przesuwa drzewa i inne znaki punktowe o najwyżej 1,5 mm na papierze;
- linie jednego rodzaju biegnące obok siebie, takie jak dwie jezdnie, łączy w jedną na ich środku.

Na mapie treningowej nie mapuje się: mapa bazowa i kandydaci zostają materiałem roboczym zgodnym z położeniem.
freemapper buduje mapę treningową z zapisanych danych, więc poprawki zrobione w OOM do niej nie trafiają. Aby
odnowić tylko mapę treningową, usuń `<nazwa>-training.omap` (z `--ocad` także `ocad/<nazwa>-training.ocd`) i uruchom
`run --training` bez `--force`, ponieważ `--force` zastępuje także mapę bazową. Niepewne obiekty na mapie
treningowej wymagają jeszcze rzutu oka w terenie przed zawodami. `run` zachowuje istniejącą mapę treningową i mówi
o tym; `export --training` kończy się wtedy błędem "map already exists".

## 11. OCAD

Z `--ocad` freemapper zapisuje folder `ocad/`:

- `<nazwa>.ocd`: mapa w formacie OCAD 12, z georeferencjonowaniem, budynkami i szablonami jako mapami tła;
- `<nazwa>-candidates-<warstwa>.ocd`: kandydaci, po jednej mapie na warstwę (`terrain`, `water`, `vegetation`,
  `paths`, `barriers`, `features`, `out-of-bounds`), jako ukryte mapy tła;
- obrazy szablonów z plikami georeferencji.

Znaki w pliku OCAD są zastępcze: mają właściwy numer i kolor, ale prostą grafikę. Wczytaj oficjalny zestaw znaków w
OCAD (*Symbol › Load Colors and Symbols from …*): ISSprOM 2019 dla map sprintowych, zestaw IOF dla szkół 2019 dla
map dziedzińca szkolnego, ISOM 2017-2 dla map leśnych. OCAD zastąpi znaki zastępcze według numeru. Wynik w OCAD nie
był jeszcze sprawdzony we wszystkich wersjach programu; uwagi są mile widziane (rozdział 17).

## 12. Wskazówki dotyczące zgód

Kto planuje zawody lub trening na mapie, najpierw prosi o zgodę. Na życzenie freemapper zapisuje w tym celu plik
`permits.md` (i `permits.json`): dla obszaru mapy wymienia jednostki administracyjne, obszary chronione i znanych
zarządców oraz podpowiada, kogo zwykle trzeba zapytać. Plik to wskazówka, nie porada prawna i nie zgoda; decydują
właściwe organy.

```sh
./freemapper permits szkola
./freemapper run szkola --permits
```

**Polska.** Dla mapy w Polsce wskazówki dotyczące zgód nie są jeszcze dostępne. Obecnie pytałyby niemieckie usługi,
dlatego `permits` kończy się dla takiej mapy błędem "not available for this region yet". Polskie źródła (jednostki
administracyjne, obszary chronione, zarządcy) są dopiero planowane. Zgody trzeba na razie uzgodnić samodzielnie z
właścicielem lub zarządcą terenu, gminą, a w lesie z nadleśnictwem lub właścicielem lasu.

Dla niemieckich krajów związkowych działają źródła urzędowe; opis jest w podręcznikach niemieckim i angielskim.

## 13. Regiony i obszary przygraniczne

### Polska (podgląd)

Polska jest w wersji podglądowej. Działa jeden adapter regionu `polska` dla całego kraju; różnią się tylko wyciągi
OSM poszczególnych województw. Co jest dostępne:

- **Dane OSM.** `freemapper init` przyjmuje obszar w dowolnym miejscu w Polsce, a `run` buduje mapę wyłącznie z
  OpenStreetMap. Pobierany jest wyciąg województwa (60 do 300 MB), który jest wspólny dla wszystkich projektów.
- **Czego nie ma.** Brak warstwic, ortofotomapy i innych szablonów, brak budynków i użytków z ewidencji gruntów i
  budynków (EGiB), brak wskazówek dotyczących zgód. Ostrzeżenie przy uruchomieniu mówi o tym wprost. Na mapie nie ma
  więc na razie żadnych budynków, bo reguły OSM ich nie przejmują.
- **W przygotowaniu.** Numeryczny model terenu (NMT), numeryczny model pokrycia terenu (NMPT) i ortofotomapa z
  Głównego Urzędu Geodezji i Kartografii (GUGiK) są budowane. Czytniki plików (siatki ASCII, ortofotomapy) i
  przeliczenie z układu PL-1992 (EPSG:2180) do siatki projektu już istnieją, ale pobieranie nie działa jeszcze dla
  użytkownika, ponieważ klient usługi indeksu arkuszy GUGiK jeszcze nie powstał. Po jego dodaniu pojawią się
  warstwice i ortofotomapa. Potem mają dojść budynki i użytki z EGiB. Dane GUGiK będą pobierane arkusz po arkuszu,
  najwyżej 2 GB na obszar mapy.
- **Układ współrzędnych.** Mapa używa siatki UTM strefy swojego środka (strefa 34 na wschód od 18° E). EPSG:2180 jest
  tylko układem źródłowym rastrów polskich, nie siatką projektu.
- **Granica.** Obszar musi leżeć w całości w Polsce. Granicą jest granica państwa z OpenStreetMap; obszar, który ją
  przekracza (margines 50 m), jest odrzucany. Mapa ze środkiem w Niemczech, która sięga do Polski, działa, ale
  ostrzega, że w części za granicą nie ma modelu terenu.
- **Strona wyboru obszaru** pokazuje ortofotomapę z usługi WMS GUGiK. Usługa służy do oglądania; freemapper nigdy
  jej nie pobiera hurtowo.
- **Język programu.** Komunikaty programu są po polsku, angielsku i niemiecku (`--lang pl`, `--lang en`,
  `--lang de`). Bez `--lang` freemapper bierze język z `FREEMAPPER_LANG`, potem z języka systemu.
- **Polskie litery** w PDF i w plikach OCAD: patrz rozdział 10.

Ponieważ na mapie w Polsce jest tylko to, co jest w OSM, traktuj ją jako szkic do uzupełnienia w terenie i z
ortofotomapy (na przykład z Geoportalu).

### Niemieckie kraje związkowe

Dla niemieckich krajów związkowych freemapper ma oficjalne źródła danych (ortofotomapa, model terenu, kataster
ALKIS i inne): Brandenburgia, Berlin, Saksonia, Saksonia-Anhalt, Hamburg, Hesja, Szlezwik-Holsztyn,
Meklemburgia-Pomorze Przednie, Nadrenia Północna-Westfalia, Dolna Saksonia, Nadrenia-Palatynat, Saara,
Badenia-Wirtembergia, Brema i Bawaria. Turyngia nie ma jeszcze źródeł. Opis każdego kraju związkowego jest w
podręcznikach niemieckim ([de.md](de.md)) i angielskim ([en.md](en.md)), rozdział 13.

### Obszary przygraniczne

Obszar, którego środek leży w Niemczech i który sięga do Polski, powstaje z danych niemieckich i OSM; w części
polskiej szablony zostają puste tam, gdzie kończą się niemieckie modele (zwykle kilkaset metrów do kilku kilometrów
za granicą). Taką część trzeba narysować ręcznie. Odwrotnie, obszar w Polsce przekraczający granicę jest odrzucany.

### Własne adaptery i zmiana adresu usługi

`freemapper adapters list` wypisuje adaptery regionów (m.in. `polska`). `freemapper adapters allow-host <adapter>
<host>` zezwala adapterowi na jeszcze jeden serwer, gdy komunikat mówi, że jego adapter nie zna danego hosta. Wydaj
to polecenie tylko wtedy, gdy ufasz serwerowi, i podaj host dokładnie tak, jak w komunikacie. Potem powtórz
uruchomienie i zgłoś zmianę (rozdział 17). Szczegóły i własne adaptery opisuje podręcznik angielski.

## 14. Dalsza praca

Mapę bazową (`<nazwa>.omap`) poprawiasz w OOM. Kolejne uruchomienie `run` nie nadpisuje istniejącej mapy bez
`--force`; stara mapa zostaje wtedy jako `<nazwa>.omap.bak`. Do druku w innych skalach służy freemapper-print, do
planowania tras freemapper-courseplan. Oba czytają `.omap` i `freemapper.toml` z freemapper.

## 15. Komunikaty i co robić

Język programu wynika z `--lang`, potem ze zmiennej środowiskowej `FREEMAPPER_LANG`, potem z języka systemu (`LANG`),
a w ostateczności jest to angielski. Obecnie dostępne są angielski i niemiecki. Błąd zaczyna się od `error:`,
a program kończy się wtedy kodem wyjścia 1. Ostrzeżenie zaczyna się od `warning:`. Błędne wywołanie daje
kod 2. Najważniejsze komunikaty w angielskim brzmieniu:

| Komunikat (początek) | Znaczenie | Co robić |
|---|---|---|
| `the map area lies outside the supported states` | Obszar nie leży w obsługiwanym regionie. | Wybierz inny obszar. |
| `another freemapper run is using this folder (since …` | Inne uruchomienie (terminal, asystent, strona wyboru obszaru) pracuje w tym folderze. Wskazuje na to plik `.freemapper-run.lock`. | Poczekaj na koniec albo zatrzymaj je. Pozostałą po przerwanym uruchomieniu blokadę freemapper zastępuje po minucie. |
| `map area plus buffer is … more than 4000 m per side` | Obszar jest za duży. | Narysuj mniejszy albo podziel na kilka map. |
| `no area found for …` | Miejsca nie znaleziono albo jego teren to kilka części w OSM. | Zapisz nazwę inaczej, narysuj obszar w przeglądarce albo wybierz część: `--osm <id> --part <n>`. |
| `… is an area of … separate parts (…)` | Obiekt OSM składa się z kilku oddzielnych powierzchni. | Wybierz jedną przez `--part <n>` (1 jest największa) albo zapisz obrys jako GeoJSON i użyj `--geojson`. |
| `… not found. Nominatim lists only tagged OSM objects …` | Nominatim nie zna obiektu OSM, często jest to linia zewnętrzna relacji bez własnych znaczników. | Użyj relacji (`--osm relation/…`) albo `--geojson`. |
| `--part picks a part of an OSM area and needs --osm` | `--part` działa tylko z `--osm`. | Podaj `--osm <id> --part <n>`. |
| `warning: the map is written, but its preview is missing: …` | Mapa jest zapisana, ale nie dało się zapisać `<nazwa>-preview.webp` (na przykład brak miejsca na dysku). | Zrób miejsce i uruchom ponownie z `--force`. |
| `warning: part of the map area lies outside the supported states` | Część obszaru leży poza obsługiwanymi regionami (za granicą, na morzu). | Narysuj tę część ręcznie albo przesuń obszar. |
| `download of … failed` | Serwer był nieosiągalny lub przysłał błędne dane. | Spróbuj później. Wcześniej pobrane dane zostają. |
| `… refused the request (HTTP 400)` | Usługa odrzuca żądanie. | Spróbuj ponownie. Jeśli się powtarza, zgłoś błąd. |
| `download of … is larger than … MB; stopped` | Serwer przysyła znacznie więcej, niż plik może mieć; freemapper przerywa i usuwa częściowe pobranie. | Spróbuj później, inaczej zgłoś błąd. |
| `…: freemapper does not send requests to this address: the adapter of … does not name the host …` | freemapper pyta tylko serwery, które zna adapter regionu. Usługa przeniosła się lub przekierowała gdzie indziej. | Patrz rozdział 13, "Własne adaptery i zmiana adresu usługi". |
| `…: freemapper does not send requests to this address: …` z innym powodem | freemapper nie łączy się z adresami tego komputera ani sieci lokalnej, podąża za najwyżej trzema przekierowaniami i nie za przekierowaniem na `http`. | Nic nie zezwalać. Dla wbudowanego regionu zgłoś błąd. |
| `map already exists` | W folderze jest już mapa, która może zawierać Twoją pracę. | Zapytaj siebie, czy ją zastąpić, i dopiero wtedy użyj `--force`. |
| `not available for this region yet` | Dla tego regionu dana funkcja jeszcze nie działa (dla Polski: wskazówki dotyczące zgód). | Patrz rozdział 12 i 13. |
| `warning-osm-only-no-terrain` (w treści: mapa tylko z OSM, bez modelu terenu) | Region ma tylko dane OSM: nie będzie warstwic, ortofotomapy ani szablonów. | Patrz rozdział 13, Polska. |
| `the release could not be verified` | Podpis wydania nie zgadza się; nic nie zainstalowano. | Nie obchodź tego. Spróbuj później. |
| `cannot write to the program folder` | Aktualizacja nie może zastąpić programu w tym folderze. | Rozpakuj nowe wydanie ręcznie albo przenieś program do folderu, do którego masz prawo zapisu. |

Typowe problemy:

- **macOS: "cannot be opened" lub "killed".** Usuń znacznik kwarantanny: `xattr -d com.apple.quarantine freemapper`.
- **Pobieranie nie działa.** Usługi bywają chwilowo niedostępne. Spróbuj później; `--offline` działa z danymi
  pobranymi wcześniej.
- **Za mało miejsca.** Sprawdź wolne miejsce przed pierwszym uruchomieniem w nowym województwie; pamięć podręczna
  (macOS `~/Library/Caches/freemapper`, Linux `~/.cache/freemapper`, Windows `%LOCALAPPDATA%\freemapper`) może urosnąć
  do kilku GB.
- **Podkład na stronie wyboru obszaru jest szary lub "access blocked".** Zaktualizuj freemapper i sprawdź połączenie
  z internetem.

## 16. Informacje o źródłach danych

Dane są objęte otwartymi licencjami, które wymagają podania źródła. freemapper zapisuje te informacje w informacjach
pozaramkowych, w uwagach do mapy i w `ATTRIBUTION.txt`. Dla map w Polsce są to na razie:

- `© OpenStreetMap contributors, ODbL`
- `Created with freemapper – https://github.com/HenryJobst/freemapper`

Gdy dojdą dane GUGiK, dojdzie też wiersz ze źródłem i licencją tych danych. Dla niemieckich regionów wiersze
źródłowe są wymienione w podręczniku angielskim. Zachowaj te wiersze na wydrukowanych i opublikowanych mapach. Zgodnie
z licencją ODbL trzeba podać źródło, gdy przekazuje się mapę dalej; z `--pdf-sources credit-only` robisz to sam.
Wiersz freemapper z adresem projektu jest zawsze obecny.

## 17. Pomoc i zgłaszanie błędów

Pytania, błędy i życzenia zgłaszaj jako [issue na GitHubie](https://github.com/HenryJobst/freemapper/issues).
Pomaga dołączenie:

- komunikatu,
- miejsca (nazwa lub współrzędne),
- pliku `freemapper.toml` projektu.

Nie wpisuj w zgłoszeniu swojego adresu e-mail ani innych danych osobowych, jeśli nie jest to potrzebne.
