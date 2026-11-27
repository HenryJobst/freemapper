# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = a(z) { $value } nem olyan nyelv, amelyhez a freemapper szövegei megvannak; elérhető: { $available }

## init

cli-part-needs-osm = a --part egy OSM-terület egy részét választja ki, és a --osm kapcsolót igényli; a --place esetén előbb keresse meg az azonosítót, majd használja az --osm <id> --part <n> parancsot
cli-reading-file = { $path } beolvasása
cli-no-area-for-place = nincs terület ehhez: "{ $query }". Ha a hely több különálló részből álló terület, válasszon egyet az --osm <id> --part <n> kapcsolókkal, vagy használja a --geojson kapcsolót
cli-using-place = Használt hely: { $name } ({ $id })
cli-created-project = Projekt létrehozva: { $path }

## Cache

cli-no-cache-dir = ezen a rendszeren nincs gyorsítótármappa; adja meg a --cache-dir kapcsolóval

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] 1 csempe letöltve
       *[other] { $count } csempe letöltve
    }
cli-wrote = Megírva: { $path }
cli-kept-map = Megtartva: { $path } (tartalmazhatja a munkáját)
cli-map-lacks-templates = A térkép még nem hivatkozik a képsablonokra. Adja hozzá őket az OOM-ban (Sablonok > Sablon megnyitása…), vagy cserélje le a térképet a --force kapcsolóval, amely a régit .omap.bak néven megtartja.
cli-map-lacks-buildings = A térképen még nincsenek ALKIS-épületek. Hozzáadásukhoz cserélje le a --force kapcsolóval, amely a régit .omap.bak néven megtartja.
cli-map-lacks-canopies = A térképről hiányoznak az ALKIS-féle fedett részek és autóbeállók (522), amelyek most már térképobjektumok. Hozzáadásukhoz futtassa a --force kapcsolóval, amely a régi térképet .omap.bak néven megtartja.
cli-map-light = Az alaptérkép: { $joined } vonal egyesítve, { $left } rejtett vagy túl kicsi objektum a jelölt rétegekben maradt (semmi sem mozdult el)
cli-map-light-kept = A jelölt rétegek most { $left } olyan objektumot vagy szakaszt is tartalmaznak, amelyet ez a verzió levesz az alaptérképről (freemapper:generalized címkével). A megtartott térképen még megvannak, ezért kétszer látszanak. A tisztításhoz cserélje le a térképet a --force kapcsolóval (a régi .omap.bak néven megmarad).
cli-map-old-candidates = A térkép még a templates/candidates.omap fájlt nevezi meg; ez a futás minden jelölttel feltöltötte. A jelöltek mostantól hét rétegben is megvannak, rétegenként egy fájlban (templates/candidates-*.omap). A térkép akkor listázza őket, ha lecseréli a --force kapcsolóval (a régi .omap.bak néven megmarad), vagy maga adja hozzá őket az OOM-ban (Sablonok > Sablon megnyitása…).

## PDF

cli-pdf-without-credits = PDF forrásmegjelölés nélkül: az ODbL és a dl-de/by-2-0 megköveteli a források megnevezését, ha továbbadja a térképet; lásd ATTRIBUTION.txt
cli-wrote-pdf = Megírva: { $path } (a térkép jelöltek nélkül)
cli-wrote-pdf-candidates = Megírva: { $path } (az ellenőrző nyomat: a térkép minden jelölttel, helyhű; ellenőrizze őket a terepen)
cli-wrote-training = Megírva: { $map } és { $pdf } (az edzéstérkép: { $joined } egyesítve, { $shifted } eltolva, { $left } kihagyva vagy elvágva)
cli-training-free-ends = Szabadon hagyott vonalvégek: { $ways } ott, ahol egy út másik úttal találkozott, { $others } más vonalaknál
cli-training-apart-left = az edzéstérképen még { $length } mm-nyi (papíron) szakaszon túl kicsi az objektumok közti távolság; ellenőrizze az OOM-ban
cli-kept-training = Az edzéstérkép megtartva: { $path }, hogy egy futás ne írjon felül olyat, amelyet kinyomtattak vagy továbbadtak; törölje, vagy a megújításhoz adja meg a --force kapcsolót

## OCAD

cli-ocad-kept = Megtartva: { $path } (tartalmazhatja a munkáját; a --force lecseréli, és a régit .ocd.bak néven megtartja)
cli-ocad-old-candidates = Az OCAD-térkép még a(z) { $path } fájlt nevezi meg háttértérképként; ez a futás minden jelölttel feltöltötte. A jelöltek mostantól hét rétegben is megvannak, rétegenként egy fájlban (-candidates-*.ocd). Az OCAD-térkép akkor listázza őket, ha lecseréli a --force kapcsolóval (a régi .ocd.bak néven megmarad), vagy maga adja hozzá őket az OCAD-ban háttértérképként.
cli-ocad-wrote = { $count ->
        [one] Megírva: { $path } (1 objektum)
       *[other] Megírva: { $path } ({ $count } objektum)
    }
cli-ocad-left-out = Kihagyva az OCAD-térképből: { $skipped } objektum (nincs hozzá OCAD-jel, nincs területe, vagy az OCAD ±2 m-es rajzterületén kívül van) és { $dropped } elfajult vonalrész vagy lyuk.
cli-ocad-load-symbols = Az OCAD-ban válassza a Symbol > Load Colors and Symbols From… menüpontot, és adja meg a hivatalos jelkulcsot{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] {" "}az IOF iskolai jelkulcs 2019
       *[other] {" "}ISSprOM 2019
    }; ez a freemapper helyettesítő jeleit szám szerint kicseréli.

## select

cli-select-running = Területválasztás ehhez: { $path }: nyomja meg az oldalon a „Kész” gombot, vagy a Ctrl+C-t a leállításhoz.
cli-select-no-browser = A böngészőt nem sikerült megnyitni ({ $error }); nyissa meg a fenti címet.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; nincs területe, nem lehet térképterület): { $place }
cli-find-hint = A projekt létrehozása: freemapper init <dir> --osm { $id }
cli-find-no-area = Egyik helynek sincs olyan területe, amely térképterület lehetne. Rajzolja meg a területet így: freemapper select <dir>
cli-find-nothing = nem található hely ehhez: "{ $query }"; próbáljon másik nevet, vagy rajzolja meg a területet így: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = figyelmeztetés: a(z) { $folder } mappában lévő adaptert kihagytuk: { $finding }
cli-adapter-note = figyelmeztetés: a(z) { $folder } mappában lévő adapter betöltődik, de: { $finding }
cli-adapter-from-project = figyelmeztetés: a projektmappából származó { $adapter } adaptert használjuk; ezeket a hosztokat kérdezheti: { $hosts }
cli-adapter-overlaps = figyelmeztetés: a(z) { $adapter } adapter a(z) { $others } területét is lefedi; az objektumai az ottani térképekre kerülnek
cli-adapter-dir-relative = figyelmeztetés: a FREEMAPPER_ADAPTER_DIR nem abszolút útvonal ({ $path }); figyelmen kívül hagyjuk
cli-adapters-no-project = figyelmeztetés: a(z) { $path } nem mappa; projektadaptereket nem olvastunk be
cli-adapters-export-unknown = a(z) { $adapter } adaptert a program egyetlen jegyzékfájlja sem írja le, ezért nincs mit exportálni; a `freemapper adapters list` megmutatja az azonosítókat
cli-adapters-export-taken = A(z) { $path } létezik, és nem üres; exportáljon olyan mappába, amely még nem létezik
cli-adapters-export-done = A(z) { $adapter } adapter { $count } fájlja a(z) { $path } mappába került. Csak az van bennük, amit a jegyzékfájl leír; a program kódjában lévő címek nincsenek benne.
cli-adapters-export-next = Módosítsa a címeket a manifest.toml fájlban, ellenőrizze a mappát a `freemapper adapters check` paranccsal, majd tegye át ide: { $path }. A következő indítástól kezdve figyelmeztetéssel felváltja a beépített adaptert. A mappa törlése visszahozza a beépítettet.
cli-adapters-overlaps = közös terület: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revízió { $revision }  ellenőrizve { $checked }  hosztok: { $hosts }
cli-adapters-source-built-in = beépített
cli-adapters-source-user = a felhasználó mappájából
cli-adapters-source-project = a projektből
cli-adapters-never-checked = soha
cli-adapters-user-dir = Az adapterek innen olvashatók: { $path }
cli-adapters-no-user-dir = Ezen a rendszeren nincs mappa az adaptereknek; a FREEMAPPER_ADAPTER_DIR megad egyet

## adapters allow-host (ADR 0110)

cli-allow-host-added = a freemapper mostantól ezt a hosztot is megkérdezheti ehhez: { $name }: { $host }. A bejegyzés itt van: { $file }; a `freemapper adapters disallow-host { $adapter } { $host }` parancs visszavonja. Egy futó területoldal vagy MI-asszisztens a bejegyzést újraindításkor olvassa be
cli-allow-host-already = { $host } már engedélyezett ehhez: { $name }
cli-allow-host-declared = a(z) { $name } adaptere maga megnevezi ezt a hosztot: { $host }; nincs mit engedélyezni
cli-allow-host-refused = { $host } nem engedélyezhető: { $reason }
cli-allow-host-unknown-adapter = nincs beépített { $adapter } adapter; a beépített adapterek: { $adapters }
cli-allow-host-folder-adapter = a(z) { $adapter } a felhasználó adaptermappájának adaptere; adja inkább a hosztot a manifestje `[net] hosts` részéhez
cli-allow-host-no-config = ezen a rendszeren nincs konfigurációs mappa az engedélyezett hosztok tárolásához
cli-allow-host-write = a(z) { $file } nem írható: { $detail }
cli-allow-host-unreadable = a(z) { $file } nem olvasható ({ $detail }); javítsa vagy távolítsa el a fájlt; semmi sem változott
cli-disallow-host-removed = a freemapper mostantól nem kérdezi ezt a hosztot ehhez: { $name }: { $host }
cli-disallow-host-absent = { $host } nem volt engedélyezve ehhez: { $name }; nincs mit visszavonni
cli-allowed-hosts-unreadable = figyelmeztetés: a(z) { $file } nem olvasható ({ $detail }); nincsenek további engedélyezett hosztok
cli-allowed-hosts-ignored = figyelmeztetés: { $file }: a(z) { $adapter } adapterhez tartozó { $host } bejegyzést figyelmen kívül hagyjuk: { $reason }
cli-allowed-hosts-no-adapter = a freemapperben nincs ilyen nevű beépített adapter
cli-allowed-host-redundant = figyelmeztetés: { $host } mostantól azon hosztok egyike, amelyeket a(z) { $name } adaptere megnevez; a(z) { $file } fájlban lévő bejegyzését eltávolítottuk
cli-adapters-allowed = az Ön által engedélyezett: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revízió { $revision }, hosztok: { $hosts }
cli-check-error = hiba: { $finding }
cli-check-warning = figyelmeztetés: { $finding }
cli-check-passed = { $folder } megfelelt az ellenőrzésen
cli-check-passed-warnings = { $folder } megfelelt az ellenőrzésen, { $count ->
        [one] egy figyelmeztetéssel
       *[other] { $count } figyelmeztetéssel
    }
cli-check-failed = { $folder } nem felelt meg az ellenőrzésen
cli-check-no-folder = nincs ilyen mappa: { $folder }
cli-check-live-ok = { $capability } rendben: { $detail }
cli-check-live-failed = { $capability } SIKERTELEN: { $detail }
cli-check-live-not-asked = { $capability } nem lett lekérdezve: { $detail }
cli-check-live-box-outside = a téglalap nem esik a(z) { $adapter } adapter határain belülre; az élő ellenőrzés a szolgáltatásokat olyan helyre kérdezi, amelyet az adapter lefed
cli-check-live-cannot = az élő ellenőrzés nem futtatható: { $detail }
cli-check-detail-not-declared = az adapter egyet sem deklarál
cli-check-detail-stopped = egy korábbi rész sikertelen volt
cli-check-detail-see-warnings = lásd a figyelmeztetéseket
cli-check-detail-outside-dataset = a téglalap az adathalmazon kívül esik
cli-check-detail-objects = { $count ->
        [one] egy objektum
       *[other] { $count } objektum
    }
cli-check-detail-layers = { $layers } réteg, { $count } objektum
cli-check-detail-trees = { $count ->
        [one] egy fa
       *[other] { $count } fa
    }
cli-check-detail-extract = { $label }, adatok kelte: { $date }
cli-check-raster-orthophoto = ortofotó
cli-check-raster-terrain = terepmodell
cli-check-raster-surface = felszínmodell
cli-check-raster-tiles = { $product }: { $count ->
        [one] egy csempe
       *[other] { $count } csempe
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } és { $last }

## Folder lock

cli-lock-held = egy másik freemapper-futás használja ezt a mappát (ekkortól: { $time }, indította: { $who }). Várja meg, vagy állítsa le előbb.
cli-lock-who-cli = a terminál
cli-lock-who-mcp = egy MI-ügynök (MCP)
cli-lock-who-page = a területválasztó oldal
cli-lock-who-unknown = egy másik program
cli-lock-failed = nem sikerült zárolni a projektmappát ehhez a futáshoz: { $error }
cli-lock-stuck = valami más van az útban, nem zárolófájl, és nem lehetett eltávolítani

## Run routes of the selection page

cli-run-bad-request = A futtatási beállítások érvénytelenek: { $error }
cli-run-no-project = Ebben a mappában még nincs mentett projekt. Előbb mentse el a területet.
cli-run-already-running = Ehhez a projekthez már fut egy futás ({ $job }). Várja meg a végét, vagy előbb szakítsa meg.
cli-run-disk-stops = Nincs elég szabad lemezterület a letöltéshez: { $messages }
cli-run-busy-elsewhere = A program egy másik projektje éppen folyamatot futtat. A projektek közös letöltési gyorsítótáron osztoznak, ezért egyszerre csak egy futás megy. Próbálja újra, ha végzett.
cli-run-nothing-to-cancel = Nem fut semmi, ezért nincs mit megszakítani.
cli-run-unknown-job = Ehhez a projekthez nincs { $job } futás.
cli-status-too-fast = Az állapotlekérések túl gyakoriak. Legfeljebb másodpercenként ötször kérdezzen.
cli-open-folder-failed = A mappát nem sikerült megnyitni ({ $error }).
cli-open-folder-no-opener = Ezen a számítógépen nincs program a mappák megnyitásához ({ $program }). Nyissa meg a mappát kézzel.
cli-open-folder-missing = A projektmappa még nem létezik. Előbb mentse el a területet.
cli-open-folder-refused = A mappa neve nem biztonságos a fájlkezelőnek való átadáshoz.
cli-save-running = Ehhez a projekthez fut egy futás. Mentse újra, ha végzett.
cli-save-unknown-kind = Ismeretlen térképfajta: "{ $kind }". Használja: schoolyard, sprint vagy forest.
cli-save-scale-too-large = A { $scale } méretarány túl nagy.
cli-save-bad-content = A tartalombeállítások érvénytelenek. Semmi sem lett mentve.
cli-map-image-none = Előbb hozza létre a térképet.
cli-map-image-gone = Ez az előnézet már nem érhető el.
cli-map-image-running = Ehhez a projekthez fut egy futás. Az előnézet a végeztével számítható ki.
cli-map-image-stale = A mentett projekt már nem egyezik a létrehozott térképpel, például mert a terület, a térképfajta vagy a méretarány megváltozott. Előbb indítson egy futást, amely lecseréli a térképet (az oldalon „A meglévő térkép cseréje”, vagy freemapper run --force).
cli-map-image-superseded = Időközben újabb előnézetet kértek, ezért ezt nem számoltuk ki.
cli-map-image-bad-content = A tartalombeállítások érvénytelenek.
cli-map-image-bad-request = Az előnézetkérés érvénytelen. Küldjön egy JSON-objektumot, ha van, a tartalombeállításokkal.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – területválasztás
page-heading = Válaszd ki a térképterületet
page-update-available = A freemapper { $version } elérhető. Frissítés ezzel:
page-update-notes = Kiadási jegyzetek
page-folder = Projektmappa
page-pick = Hely keresése
page-query-placeholder = pl. Clara-Zetkin-Schule Strausberg
page-search-button = Keresés
page-searching = Keresés …
page-no-results = Nincs találat.
page-point-only = csak egy pont, nem terület
page-draw = Rajzold meg magad
page-rectangle = Téglalap
page-polygon = Sokszög
page-finish-polygon = Sokszög lezárása
page-clear = Törlés
page-hint-rectangle = Kattints a térképen két szemközti sarokra.
page-hint-polygon = Kattints a sarkokra egymás után. A lezáráshoz kattints az első sarokra, kattints duplán, vagy kattints a „Sokszög lezárása” gombra. Az Esc megszakítja, a Backspace törli az utolsó sarkot.
page-hint-idle = Válassz egy helyet a keresésből, vagy rajzold meg magad a területet.
page-summary = Terület
page-source-osm = Az OpenStreetMapből: { $name }
page-source-drawn = Kézzel rajzolva
page-size = Méret ráhagyással: { $width } × { $height } m
page-tiles-one = { $n } csempe: { $names }
page-tiles-other = { $n } csempe: { $names }
page-download = Letöltés: kb. { $mb } MB
page-download-none = Letöltés: minden gyorsítótárban van már
page-download-details = Mi töltődik le
page-cached = gyorsítótárban
page-disk = Lemezterület: { $volumes }
page-disk-volume = { $free } szabad, kb. { $needed } kell
page-cache-volume = gyorsítótár
page-project-volume = projektmappa
page-no-room = nem fér el a lemezen
page-no-room-skip = Nem fér el minden a lemezen (a freemapper { $reserve } szabadon hagy). A letöltés kihagyja a megjelölt részeket. Ha szükséged van rájuk, előbb szabadíts fel helyet:
page-no-room-stop = Nincs elég lemezterület (a freemapper { $reserve } szabadon hagy). A megjelölt részek nélkül nincs térkép, ezért a letöltés megáll, mielőtt bármit letöltene. Előbb szabadíts fel helyet:
page-disk-unknown = a freemapper nem tudja kiolvasni a szabad lemezterületet ({ $error }). Ellenőrzés nélkül tölt le.
page-too-large = A terület túl nagy: oldalanként legfeljebb { $max } m (ráhagyással). Válassz kisebbet.
page-outside = A terület a támogatott tartományokon ({ $states }) kívül esik; más régiókhoz a freemappernek még nincsenek adatforrásai.
page-outside-unknown = A terület a freemapper által támogatott régiókon kívül esik; ott még nincsenek adatforrásai.
page-adapters-failed = A légifotók nem listázhatók ({ $detail }). A térkép csak az OpenStreetMapet mutatja; minden más működik.
page-settings = Térkép
page-kind = Térképfajta
page-schoolyard = Iskolaudvari térkép (1:1000)
page-sprint = Sprinttérkép (1:4000)
page-forest = Erdei térkép (1:15000)
page-scale = Méretarány 1:
page-map-title = Cím
page-contour-interval = Szintvonalköz
page-contour-auto = automatikus
page-steep-slope = Meredekségi küszöb
page-steep-slope-auto = automatikus (27°)
page-steep-slope-hint = Ettől a lejtőszögtől számít a terep meredeknek a rézsűk és sziklafalak keresésekor. A 27° (automatikus) dombvidékre, parkokra és városokra való. Sziklavárosokban, szurdokokban és kőbányákban szinte minden lejtő meredek, ezért a rézsűk és a sziklafalak egyetlen foltba olvadnak, és egyik sem található meg; az 55° és 65° közötti érték szétválasztja őket. Túl magas érték csak a legmeredekebb falakat hagyja meg.
page-face-slope = Falküszöb
page-face-slope-auto = automatikus (70°)
page-face-slope-hint = Ettől a lejtőszögtől a terep fekete sziklafalfelületként jelenik meg. A 70° (automatikus) a sziklavárosokhoz illik, mint Adršpach (a terület 5,2 %-a). A 65° több és nagyobb fekete felületet ad, a meredek lejtőket is beleértve (8,4 %). A 75° csak a valódi falakat hagyja meg, rojtos szélekkel (2,5 %). Túl magas érték mellett alig marad felület.
page-settings-fixed = A projekt már létezik; a térképfajta, a méretarány és a cím a freemapper.toml fájlban van. A szintvonalköz még módosítható.
page-save = Mentés
page-saving = Mentés …
page-saved = Elmentve a freemapper.toml fájlba.
page-zone-changed = Elmentve. A terület most egy másik UTM-zónában van; a freemapper a következő futáskor újra letölti az adatait.
page-provider-changed = Elmentve. Az új terület magassági adatait és légifotóit másik adatszolgáltató nyújtja; a freemapper a következő futáskor újra letölti az adatait.
page-save-stale = A freemapper.toml fájlt egy másik ablakban vagy egy másik program módosította, mióta ez az oldal beolvasta. Semmi sem lett mentve; az oldal most már a mentett állapotot ismeri. Mentsd újra, ha le akarod cserélni az itt látható területtel.
page-has-map = A meglévő térkép (.omap) változatlan marad; a freemapper új sablonokat és jelölteket készít az új területhez. Az alábbi „A meglévő térkép cseréje” beállítás (vagy a „freemapper run --force”) a térképet is újra elkészíti, és a régit .omap.bak néven megtartja.
page-done = Kész
page-source = Forráskód (AGPL-3.0-or-later)
page-closed = Kész. Bezárhatod ezt az ablakot.
page-next = Következő: freemapper run "{ $dir }"
page-made-files = Létrehozva: { $files }. Mappa: { $dir }
page-back-to-agent = Mondd meg az MI-asszisztensednek, hogy a terület el van mentve.
page-nothing-saved = Semmi sem lett mentve.
page-failed = Hiba: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Légifotó { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Mi kerül a térképre
page-content-help = A profil dönti el, mi kerül közvetlenül a térképre, és mi marad ellenőrzendő jelölt. A csoportra hozott döntés erősebb a profilnál. A területtel együtt mentődik.
page-content-profile-legend = Profil
page-content-profile-safe = Biztonságos
page-content-profile-safe-hint = Minden objektum ott marad, ahová a feldolgozás teszi: az épületek és a kataszter területei a térképre kerülnek; a falak, kerítések, fák és sok OpenStreetMap-objektum (tavak, ismert burkolat nélküli utak) jelöltek maradnak.
page-content-profile-balanced = Kiegyensúlyozott
page-content-profile-balanced-hint = A kataszter, a nyilvántartások és az utcafelmérés objektumai a térképre kerülnek, a szintvonalakkal és a vasúti sínek menti tiltott területekkel együtt. A falak, kerítések, sövények, rézsűk, lépcsők, vizek, fák, emlékművek, átjárók, utak és vasutak szintén az OpenStreetMapből jönnek. Az utcabútorok, a többi tiltott terület és a terepmodellből származó formák jelöltek maradnak.
page-content-profile-full = Teljes
page-content-profile-full-hint = Minden a térképre kerül, mint az ellenőrző nyomaton; csak a rejtett jelöltek maradnak ki.
page-content-groups-legend = Csoportonként
page-content-groups-help = A „Mint a profilban” a profilt követi. Hogy ez egy csoportnál mit jelent, az attól függ, honnan származik az egyes objektum.
page-content-place-profile = Mint a profilban
page-content-place-map = Térkép
page-content-place-candidates = Jelölt
page-content-place-off = Ki
page-content-group-walls = Falak
page-content-group-fences = Kerítések
page-content-group-hedges = Sövények
page-content-group-slopes = Rézsűk és sziklafalak
page-content-group-stairs = Lépcsők
page-content-group-trees = Fák és facsoportok
page-content-group-water-features = Vizek, szökőkutak és források
page-content-group-monuments = Tornyok, emlékművek és légvezetékek
page-content-group-furniture = Utcabútorok és lámpák
page-content-group-passages = Átjárók
page-content-group-contours = Szintvonalak
page-content-group-landforms = Dombok, mélyedések, rézsűk és sziklafalak a terepmodellből
page-content-group-vegetation = Növényzet lézerpontfelhőkből (Brandenburg, Berlin, Észak-Rajna-Vesztfália, Rajna-vidék-Pfalz, Saarland és Bajorország; kikapcsolva, amíg nem választod; kb. 40–600 MB km²-enként)
page-content-group-out-of-bounds = Tiltott területek
page-content-group-paths = Utak és vasutak
page-content-group-streets = Utcák és terek
page-run-heading = Térkép létrehozása
page-preview-heading = Térképelőnézet
page-preview-stored = Az utoljára létrehozott állapot.
page-preview-live = Az ezen az oldalon lévő beállításokkal, még nincs mentve.
page-preview-live-saved = A mentett beállításokkal, futásban még nem lett létrehozva.
page-preview-computing = Számítás …
page-preview-none = Még nincs létrehozott térkép.
page-preview-later = Ehhez a térképhez még nincs előnézet. A következő futással jön, amely megírja a térképet.
page-preview-stale = A terület, a térképfajta vagy a méretarány megváltozott a térkép létrehozása óta. A kép még az utoljára létrehozott térképet mutatja; az új előnézethez futás kell „A meglévő térkép cseréje” beállítással.
page-preview-area-unsaved = Az ezen az oldalon lévő terület még nincs mentve. A kép az utoljára létrehozott térképet mutatja; az új területhez futás kell „A meglévő térkép cseréje” beállítással.
page-preview-run = Az utoljára létrehozott állapot. Az oldal a futás végén frissíti a képet.
page-preview-open = Megnyitás teljes méretben
page-preview-alt = A térképobjektumok előnézete
page-opt-ocad = OCAD-kezdőprojekt írása
page-help-ocad = Mindenkinek, aki OCAD-ban folytatja.
page-opt-pdf = Nyomtatható PDF
page-help-pdf = Csak a térképobjektumok, jelöltek nélkül.
page-opt-pdf-candidates = Ellenőrző nyomat minden jelölttel
page-opt-training = Edzéstérkép
page-help-training = A térkép és minden jelölt, úgy megtisztítva, hogy úgy nyomtatható, ahogy van; OCAD-dal OCAD-fájlként is.
page-views-heading = Az edzéstérkép nézetei
page-view-no-paths = Utak nélkül (Ultimate-térkép)
page-view-brown-only = Csak barna (szintvonalas tájfutás)
page-view-no-north-lines = Északi vonalak nélkül
page-view-custom = Másik nézet
page-help-views = Minden kipipált nézet egy további fájl az edzéstérkép mellett, `<name>-training-<view>.omap` és PDF-je, OCAD-dal OCAD-fájl is. Másik nézet például `no:505,506` (jelek kihagyása) vagy `only:101,102` (csak ezek megtartása); a `+` egyesíti a nézeteket. A cím, a méretarány és a források mindig megmaradnak.
page-help-views-forest = Az utak nélküli és a csak barna nézet csak erdei térképekhez van.
page-opt-scale-bar = Méretarány-vonalzó a PDF-eken
page-help-scale-bar = A német térképeken általában nincs.
page-opt-sources = Forrássorok a PDF-eken
page-sources-auto = automatikus
page-sources-full = teljes
page-sources-credit-only = csak freemapper
page-warn-credit-only = A PDF-ek ekkor csak a freemappert nevezik meg. Az adatlicencek (ODbL, dl-de/by-2-0) megkövetelik a forrásmegjelölést, ha továbbadod a térképet; ekkor a forrásokat te magad nevezed meg. Az ATTRIBUTION.txt mindet felsorolja.
page-need-pdf = A méretarány-vonalzó és a forrássorok csak akkor érvényesek, ha PDF készül.
page-opt-offline = Csak gyorsítótár, hálózat nélkül
page-opt-force = A meglévő térképek cseréje
page-help-force = A térkép, és ha az „Edzéstérkép” be van pipálva, az edzéstérkép is. A régi fájlok .omap.bak néven megmaradnak.
page-opt-permits = Engedélyezési tudnivalók
page-help-permits = Közigazgatási egységek, védett területek és hogy kit kérdezz, a permits.md fájlban. Útmutatás, nem jogi tanács.
page-run-start = Térkép létrehozása
page-run-again = Létrehozás újra
page-run-cancel = Mégse
page-run-cancelling = Megszakítás …
page-warnings-heading = Figyelmeztetések
page-where-files = A fájlok a projektmappában vannak:
page-open-folder = Mappa megnyitása
page-log-label = A futás naplója
page-force-title = Lecseréled a meglévő térképeket?
page-force-text = A meglévő térkép lecserélődik, és ha az „Edzéstérkép” be van pipálva, az edzéstérkép is. A szerkesztéseid nem vesznek el: a régi fájlok .omap.bak néven a projektmappában maradnak. Folytatod?
page-force-cancel = Mégse
page-force-confirm = Csere
page-step-fetch = Adatok letöltése
page-step-derive = Sablonok számítása
page-step-map = A térkép írása
page-step-ocad = Az OCAD-projekt írása
page-step-pdf = A PDF írása
page-step-pdf-candidates = Az ellenőrző nyomat írása
page-step-training = Az edzéstérkép írása
page-step-permits = Az engedélyezési tudnivalók írása
page-step-line = { $index }. lépés / { $total }: { $label }
page-starting = A futás indul …
page-file-kind-map = térkép
page-file-kind-template = sablon
page-file-kind-ocad = OCAD-térkép
page-file-kind-ocad-candidates = OCAD-jelöltek
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = ellenőrző nyomat
page-file-kind-training-map = edzéstérkép
page-file-kind-training-pdf = edzéstérkép-PDF
page-file-kind-training-ocad = OCAD-edzéstérkép
page-file-kind-training-view-map = edzéstérkép-nézet
page-file-kind-training-view-pdf = edzéstérkép-nézet PDF
page-file-kind-training-view-ocad = OCAD-edzéstérkép-nézet
page-file-kind-preview = térképelőnézet
page-file-kind-permits = engedélyezési tudnivalók
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Kész. A fájlok megírva.
page-result-failed = A futás sikertelen: { $error }
page-result-failed-kept = Ami megíródott, a projektmappában marad; egy új futás onnan folytatja.
page-result-cancelled = Megszakítva. Ami már letöltődött, a gyorsítótárban marad; egy új futás onnan folytatja.
page-result-title-done = A térkép elkészült
page-result-title-failed = A futás sikertelen
page-result-title-cancelled = A futás megszakítva
page-kept-map = A meglévő térkép változatlan maradt. A „A meglévő térkép cseréje” újra elkészíti, és a régit .omap.bak néven megtartja.
page-no-files = Nem készült fájl.
page-opened-folder = A mappa meg van nyitva.
page-open-failed = { $message } Mappa: { $dir }
page-why-no-area = Előbb válassz területet.
page-why-checking = Az előnézet számítása folyamatban …
page-why-too-large = A terület túl nagy.
page-why-outside = A terület a támogatott tartományokon kívül esik.
page-why-disk = Nincs elég szabad lemezterület (lásd fent).
page-why-running = Fut egy futás.
page-err-scale-bar-no-pdf = A méretarány-vonalzóhoz PDF kell.
page-err-sources-no-pdf = A forrássorokhoz PDF kell.
page-save-failed = A terület nincs elmentve; futás nem indult.
page-poll-lost = Nincs válasz a szervertől. A freemapper próbálkozik tovább.
page-run-unknown = A szerver már nem ismeri ezt a futást.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Bezárás

## update (spec 3.4 to 3.6)

update-source-build = Ez a freemapper forrásból készült. Frissítse a git pull és a cargo build --release --locked parancsokkal.
update-current = A freemapper { $version } naprakész.
update-available = A freemapper { $new } elérhető (önnél { $old } van), kiadva: { $released }.
update-details =
    Kiadási jegyzetek: { $notes }
    Letöltés: { $size } MB
update-decimal-separator = ,
update-ask = Telepíti most? [i/N]
update-not-installed = Semmi sem lett telepítve.
update-needs-yes = a bemenet nem terminál, ezért a freemapper nem tud rákérdezni a telepítés előtt; a kérdezés nélküli telepítéshez futtassa: freemapper update --yes
update-installing = a freemapper { $version } letöltése és ellenőrzése
update-done = a freemapper frissítve: { $old } → { $new }. Kiadási jegyzetek: { $notes }
update-mcp-restart = Egy futó MCP-szerver a régi verziót tartja meg, amíg az MI-asszisztens újra nem indítja.
update-rolled-back = Visszaléptünk a freemapper { $version } verzióra. Az elhagyott verzió itt marad: { $path }.
update-not-writable = nem lehet írni a(z) { $path } programmappába, és a freemapper nem kér rendszergazdai jogot. Csomagolja ki oda kézzel az új kiadást, vagy költöztesse a freemappert egy olyan mappába, amelybe írhat.
update-too-old = a freemapper { $new } nem telepíthető a(z) { $old } önfrissítésével; telepítse kézzel a(z) { $needed } vagy újabb verziót, lásd: { $notes }
update-busy = egy másik frissítés vagy visszaállítás fut a programmappában (zárolófájl: { $path }); várja meg a végét, és próbálja újra
update-bad-signature = a kiadást nem sikerült ellenőrizni ({ $error }); semmi sem lett telepítve
update-check-bad-signature = a legújabb freemapper-kiadást nem sikerült ellenőrizni ({ $error }); semmi sem lett telepítve
update-no-manifest = a legújabb kiadás nem tudja magát telepíteni (nincs frissítési manifestje); töltse le innen: { $page }
update-no-manifest-version = a(z) { $version } kiadás nem létezik, vagy nem tudja magát telepíteni (nincs frissítési manifestje); lásd: { $page }
update-no-asset = a kiadásban nincs program ehhez a rendszerhez ({ $platform }); semmi sem lett telepítve
update-no-previous = nincs előző verzió, amelyre vissza lehetne lépni ({ $path } hiányzik)
update-failed = a frissítés sikertelen: { $error }
update-mode-set = A frissítési mód beállítva: { $mode }, itt: { $path }.
update-mode-env = A FREEMAPPER_UPDATE={ $value } felülbírálja ezt a beállítást, amíg be van állítva.
update-mode-failed = nem írható: { $path }: { $error }
update-notice = a freemapper { $new } elérhető (önnél { $old } van). Frissítés: freemapper update
update-applies-next-start = Az új verzió a freemapper következő indításától él.
update-install-failed = a freemapper { $new } nem lett telepítve: { $error }
update-no-config-dir = ezen a rendszeren nincs konfigurációs mappa; állítsa be a FREEMAPPER_CONFIG_DIR változót
mcp-adapter-note = Megjegyzés a személynek: az indításakor a freemapper-szerver ezt találta az adaptermappákról. Adja tovább; a személy kijavítja vagy eltávolítja a mappát, majd újraindítja a szervert.
    { $lines }
update-mcp-note = Megjegyzés a személynek: a freemapper { $new } elérhető (ez a { $old }). Kérdezze meg, mielőtt az install_update-et hívja.
update-mcp-needs-confirm = semmi sem lett telepítve; előbb kérdezze meg a személyt, és csak a beleegyezése után hívja az install_update-et confirm: true értékkel
update-mcp-note-manual = Megjegyzés a személynek: a freemapper { $new } elérhető (ez a { $old }), de az install_update nem tudja telepíteni; telepítse kézzel a(z) { $needed } vagy újabb verziót, lásd: { $notes }
update-mcp-note-bad-signature = Megjegyzés a személynek: a legújabb freemapper-kiadást nem sikerült ellenőrizni ({ $error }); semmi sem lett telepítve. Mondja el neki, és ne telepítse más úton.
update-mcp-restart-now = Indítsa újra most a freemapper MCP-szervert; még a régi verziót futtatja.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = hiba
error-cancelled = megszakítva
cli-cancelled-kept = megszakítva; megtartva: { $files }
error-io = { $path }: { $message }
error-io-disk-full = megtelt a lemez, ezért a freemapper nem tudta megírni: { $path }. Szabadítson fel helyet, és próbálja újra

## Errors: project

error-project-already-exists = a projekt már létezik: { $path }
error-project-map-exists = a térkép már létezik (tartalmazhatja a munkáját); a lecseréléséhez használja a --force kapcsolót: { $path }
error-project-invalid-scale = érvénytelen méretarány: 1:{ $scale }
error-project-invalid-contour-interval = érvénytelen szintvonalköz ({ $interval } m) a freemapper.toml fájlban; 0-nál nagyobbnak kell lennie
error-project-invalid-steep-slope = érvénytelen meredekségi küszöb ({ $slope }°) a freemapper.toml fájlban; 15° és 80° között kell lennie
error-project-invalid-face-slope = érvénytelen falküszöb ({ $slope }°) a freemapper.toml fájlban; 55° és 85° között kell lennie
error-project-scale-not-allowed = az 1:{ $scale } méretarány nem engedélyezett { $kind } térképekhez; használja ezt: { $allowed }
error-project-scale-not-allowed-enlarge = az 1:{ $scale } méretarány nem engedélyezett { $kind } térképekhez; használja ezt: { $allowed }. Rajzoljon 1:4000-ben, a freemapper-print nyomtatáskor 1:3000-re nagyít
word-kind-schoolyard = iskolaudvari
word-kind-sprint = sprint
word-kind-forest = erdei
error-project-toml = érvénytelen projektfájl: { $message }
error-project-unknown-adapter = a projektfájl a(z) "{ $adapter }" adaptert nevezi meg, amellyel ez a freemapper nem rendelkezik; egy újabb freemapper hozhatja (`freemapper update`), vagy az adaptermappájából származik (a `freemapper adapters list` mutatja), vagy a parancssorban a --adapters-from-project kapcsolóval a projekt `adapters` mappájából (csak megbízható projekthez)
error-project-not-fetched = még nincs letöltött csempe; előbb futtassa a `freemapper fetch` parancsot
error-project-ocd = OCAD-export: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: a kép és a világfájl nem egyezik a térképterülettel; futtassa újra a `freemapper derive` parancsot
error-project-tiles-outdated = a térképterülethez olyan csempék kellenek ({ $tiles }), amelyek nincsenek letöltve; futtassa a `freemapper fetch` parancsot
error-project-buildings-not-fetched = ehhez a térképterülethez nincs eltárolt ALKIS-épület; előbb futtassa a `freemapper fetch` parancsot online
error-project-orthophoto-not-fetched = ehhez a térképterülethez nincs eltárolt ortofotó; előbb futtassa a `freemapper fetch` parancsot online
error-project-terrain-not-fetched = ehhez a térképterülethez nincs eltárolt terepmodell; előbb futtassa a `freemapper fetch` parancsot online
error-project-candidates-outdated = a tárolt OSM-jelöltek nem fedik le a térképterületet; futtassa a `freemapper fetch` parancsot
error-project-stored = { $path }: { $message }; futtassa újra a `freemapper fetch` parancsot
error-project-area-too-large = a térképterület és a ráhagyás { $width } × { $height } m, oldalanként több, mint { $limit } m; csökkentse a térképterületet vagy a buffer_m értékét a freemapper.toml fájlban
error-project-area-crosses-country = a térképterület országhatár mindkét oldalán fekszik; a határon átnyúló térképek még nem támogatottak, ezért egy országon belül rajzolja meg a területet
error-project-permits-not-for-region = az engedélyezési tudnivalók ehhez a régióhoz még nem érhetők el: német szolgáltatásokat kérdeznek, és német hatóságokat neveznek meg
error-project-outside-coverage = a térképterület a támogatott tartományokon ({ $states }) kívül esik; a freemappernek egyelőre csak ezekhez vannak adatforrásai

## Errors: disk space

error-space-required = nincs elég lemezterület ehhez: { $part }. { $needed } kell, de a(z) { $path } helyen a { $reserve } tartalékon felül csak { $left } szabad. Szabadítson fel helyet, és futtassa újra a `freemapper fetch` parancsot
error-space-skipped = a freemapper kihagyja ezt: { $part }, mert nincs elég lemezterület. { $needed } kell, de a(z) { $path } helyen a { $reserve } tartalékon felül csak { $left } szabad. Szabadítson fel helyet, és futtassa újra a `freemapper fetch` parancsot
word-part-map-data = az ALKIS- és OSM-térképadatok
word-part-terrain = a terepmodell
word-part-orthophoto = az ortofotó
word-part-surface = az nDSM-sablonhoz tartozó felszínmodell
word-part-osm-update = a gyorsítótárazott OSM-kivonat frissítése
word-part-k-five = a berlini K5-térképsablon
word-part-berlin-layers = a berlini ALKIS-szerkezetek, a fakataszter, az utcafelmérés és a ponttérképrétegek
word-part-state-layers = a teljes ALKIS-szerkezetek, a tájmodell objektumai és a fakataszterek
word-part-point-cloud = a növényzetjelöltekhez tartozó lézerpontfelhők

## Errors: downloads

error-fetch-http = a(z) { $url } letöltése sikertelen: { $message }
error-fetch-http-temporary = a(z) { $url } letöltése sikertelen: { $message }. A szolgáltatás valószínűleg túlterhelt vagy karbantartás alatt áll; a freemapper legfeljebb { $attempts } alkalommal próbálkozott. Indítsa újra a futtatást néhány perc múlva; ami már letöltődött, megmarad.
error-fetch-corrupt-archive = a(z) { $url } letöltése nem teljes csempearchívum: { $message }
error-fetch-corrupt-extract = a(z) { $url } letöltése nem teljes OSM-kivonat: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] a(z) { $tiles } csempét nem fedik le a közzétett csempék; csökkentse a térképterületet vagy a buffer_m értékét a freemapper.toml fájlban
       *[other] a(z) { $tiles } csempéket nem fedik le a közzétett csempék; csökkentse a térképterületet vagy a buffer_m értékét a freemapper.toml fájlban
    }
error-fetch-missing-offline = { $count ->
        [one] offline és nincs gyorsítótárban: { $product } csempe { $tiles }; futtassa a `freemapper fetch` parancsot online
       *[other] offline és nincs gyorsítótárban: { $product } csempék { $tiles }; futtassa a `freemapper fetch` parancsot online
    }
word-product-orthophoto = ortofotó (DOP20)
word-product-dtm = terepmodell (DGM1)
word-product-image-dsm = felszínmodell (bDOM)
word-product-saxony-dtm = terepmodell (GeoSN DGM1)
word-product-saxony-dsm = felszínmodell (GeoSN DOM1)
error-fetch-osm-extract-missing = nincs gyorsítótárazott OSM-kivonat; futtassa a `freemapper fetch` parancsot online
error-fetch-not-found = { $name } nem található a szerveren
error-fetch-refused = { $url }: a szerver elutasította a kérést (HTTP { $status })
error-fetch-refused-detail = { $url }: a szerver elutasította a kérést (HTTP { $status }): { $detail }
error-fetch-too-large = a(z) { $url } letöltése nagyobb, mint { $limit } MB; leállítva
error-fetch-not-stored = ehhez a térképterülethez nincs eltárolt { $label }: a(z) { $path } hiányzik, nem olvasható, vagy nem fedi le; előbb futtassa a `freemapper fetch` parancsot online
error-fetch-range-ignored = { $url }: a szerver figyelmen kívül hagyta a(z) { $range } bájttartományt (HTTP { $status })
error-fetch-not-allowed = { $url }: a freemapper nem küld kéréseket erre a címre: { $reason }
error-fetch-host-not-named-allow = { $target }: a freemapper nem küld kéréseket erre a címre: a(z) { $name } adaptere nem nevezi meg a(z) { $host } hosztot{ $from }. Ha a szolgáltatás oda költözött, engedélyezze a hosztot a `freemapper adapters allow-host { $adapter } { $host }` paranccsal, és jelentse itt: https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: a freemapper nem küld kéréseket erre a címre: a(z) { $adapter } adapter nem nevezi meg a(z) { $host } hosztot{ $from }. Ha a szolgáltatás oda költözött, adja hozzá a hosztot az adapter manifestjének `[net] hosts` részéhez
error-fetch-host-redirected-from = {" "}({ $url } oda irányít át)
error-reason-not-a-host-name = nem hosztnév
error-reason-not-ascii = az ASCII-n kívüli nevet xn-- alakban kell írni
error-reason-not-a-dns-name = nem DNS-név
error-reason-no-domain = ennek a számítógépnek a neve, vagy tartomány nélküli név
error-reason-local-name = a helyi hálózat neve
error-reason-ip-address = IP-cím, nem DNS-név
error-reason-unexpected-character = váratlan karakter
error-reason-not-https = nem https
error-reason-user-info = felhasználói adatok
error-reason-port = a 443-tól eltérő port
error-reason-host-not-declared = a hoszt nem tartozik a deklarált hosztok közé
error-reason-local-address = ennek a számítógépnek vagy a helyi hálózatnak a címe, nem internetes szerveré
error-reason-too-many-redirects = több mint { $count } átirányítás
error-reason-redirect-nowhere = átirányítás (HTTP { $status }) sehová
error-fetch-currency-csv = pénznem-CSV, { $line }. sor: { $message }
error-alkis = érvénytelen ALKIS-adatok: { $message }
error-osm = OSM-adatok: { $message }

## Errors: templates and rasters

error-derive-zip = csempearchívum: { $message }
error-derive-image = kép: { $message }
error-derive-no-tiles = nincs csempe, amelyből számítani lehetne
error-derive-area-too-large = a térképterület túl nagy egyetlen sablonhoz: { $width } × { $height } képpont (oldalanként legfeljebb { $limit }); csökkentse a térképterületet vagy a buffer_m értékét a freemapper.toml fájlban
error-raster-world-file = érvénytelen világfájl: { $message }
error-raster-misaligned = a rasztereket nem ugyanarra a képpontrácsra igazították: { $message }
error-raster-pixel-size = a rasztereknek különböző a képpontméretük: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = érvénytelen GeoJSON: { $message }
error-area-no-polygon = a GeoJSON nem tartalmaz sokszöget
error-area-multi-polygon = { $parts } részből álló MultiPolygon; ossza a térképterületet egy-egy sokszögre
error-area-out-of-range = a koordináta kívül esik a tartományon: szélesség { $lat }, hosszúság { $lon }
error-area-not-an-area = nem használható térképterület: { $message }
error-georef-date = a(z) { $date } dátum kívül esik a World Magnetic Model érvényességi tartományán
error-georef-map-range = a K { $easting } É { $northing } pont a térképi koordinátatartományon kívül esik
error-omap-xml = érvénytelen OOM-XML: { $message }
error-omap-missing-element = a(z) <{ $name }> elem nem található
error-ocd-duplicate-symbol = a(z) { $symbol } számú jel kétszer szerepel
error-ocd-unknown-symbol = egy objektum a(z) { $symbol } jelre hivatkozik, amely nincs a fájlban
error-ocd-wrong-type = a(z) { $symbol } jelű objektum típusa nem megfelelő a jeléhez

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Használat:
cli-heading-commands = Parancsok:
cli-heading-options = Kapcsolók:
cli-heading-arguments = Argumentumok:
cli-help-possible-values = lehetséges értékek
cli-help-default = alapértelmezett
cli-arg-help = Súgó megjelenítése
cli-arg-version = Verzió megjelenítése
cli-help-help = Egy parancs súgójának megjelenítése
cli-arg-help-command = A parancs, amelynek a súgóját mutatja, például run
cli-parse-hint = További információért próbálja: '--help'.
cli-double-click-start = A freemapper megnyitja a böngészőben azt az oldalt, ahol kiválasztja a térkép területét. A projektmappa: { $path }. A program befejezéséhez zárja be ezt az ablakot.
cli-parse-unknown-argument = váratlan argumentum: { $arg }
cli-parse-missing-required = hiányzó kötelező argumentum: { $args }
cli-parse-invalid-value = érvénytelen érték: { $value } ehhez: { $arg }
cli-parse-invalid-value-choices = érvénytelen érték: { $value } ehhez: { $arg }; lehetséges értékek: { $values }
cli-parse-invalid-subcommand = ismeretlen parancs: { $value }
cli-parse-missing-subcommand = parancs megadása kötelező
cli-parse-too-many-values = váratlan érték: { $value } ehhez: { $arg }
cli-parse-argument-conflict = a(z) { $arg } nem használható a(z) { $other } mellett

help-freemapper = OOM-kezdőprojekteket készít geoadatokból
help-freemapper-lang = A programszövegek nyelve, `de` vagy `en` (alapértelmezés: FREEMAPPER_LANG, majd a rendszernyelv, majd az angol)
help-init = Projektmappa létrehozása egy térképterülethez
help-init-dir = A létrehozandó projektmappa
help-init-geojson = GeoJSON-fájl a térképterület sokszögével (EPSG:4326)
help-init-osm = A térképterület OSM-objektuma, pl. way/992423119
help-init-place = A hely neve, pl. "Clara-Zetkin-Schule Strausberg"; a freemapper a legjobb, területtel rendelkező találatot használja
help-init-part = Egy több különálló részből álló OSM-terület melyik részét használja (az 1. a legnagyobb)
help-init-kind = Térképfajta
help-arg-scale = Új projekt méretaránynevezője: iskolaudvar 1000 (alapértelmezett); sprint 4000 (alapértelmezett) vagy 5000; erdő 15000 (alapértelmezett), 10000 vagy 7500. Más értékeket elutasít, mert a freemapper-print is elutasítja őket
help-init-title = A térkép címe a peremi információkhoz (alapértelmezés: a mappa neve)
help-export = A kezdőprojekt (OOM-térkép, sablonok, forrásmegjelölés) megírása egy projektmappába
help-export-dir = Projektmappa
help-arg-force = Egy meglévő térképfájl lecserélése, a régit `<name>.omap.bak` néven megtartja
help-export-ocad = OCAD-kezdőpontot is ír az `ocad/` mappába (OCAD 12 formátum); a meglévő OOM-térkép ilyenkor marad, kivéve ha megadja a --force kapcsolót
help-export-pdf = A `<name>.pdf` fájlt is megírja, a nyomtatható térképet (csak a térképobjektumok, jelöltek nélkül); a meglévő OOM-térkép ilyenkor marad, kivéve ha megadja a --force kapcsolót
help-export-pdf-candidates = A `<name>-candidates.pdf` fájlt is megírja, az ellenőrző nyomatot: a térkép minden jelölttel, helyhűen; a meglévő OOM-térkép ilyenkor marad, kivéve ha megadja a --force kapcsolót
help-export-training = Az edzéstérképet `<name>-training.omap` és PDF-jét `<name>-training.pdf` is megírja: a térkép és minden jelölt, úgy megtisztítva, hogy úgy nyomtatható, ahogy van; a --ocad kapcsolóval az `ocad/<name>-training.ocd` fájlt is. A meglévő OOM-térkép ilyenkor marad, kivéve ha megadja a --force kapcsolót, és a --force a meglévő edzéstérképet is lecseréli
help-export-view = A --training kapcsolóval: az edzéstérképet azon részek nélkül is megírja, amelyeket egy nézet kihagy, `<name>-training-<view>.omap` néven és PDF-jével (a --ocad kapcsolóval OCAD-fájllal is). Nézetek: `no-paths` (nincsenek utak, ösvények, lépcsők), `brown-only` (csak barna jelek és a keret), `no-north-lines`, vagy egy `no:<codes>` / `only:<codes>` lista; a `+` kombinálja őket. Több fájlhoz ismételje a kapcsolót
help-export-pdf-scale-bar = Méretarány-vonalzó rajzolása a PDF-ekre (alapból ki, a német térképeken általában nincs; a méretarány száma a címblokkban marad)
help-export-pdf-sources = Forrássorok a PDF-eken: az `auto` a teljes listát nyomtatja, és csak akkor rövidíti, ha több mint 5 sor lenne, a `full` sosem rövidít, a `credit-only` csak a freemapper-forrásmegjelölést nyomtatja (az ODbL és a dl-de/by-2-0 ilyenkor megköveteli a források megnevezését, ha továbbadja a térképet). Az ATTRIBUTION.txt mindig felsorol minden forrást
help-fetch = A térképterület adatainak letöltése (ortofotó, terepmodell, nDSM, ALKIS-épületek, OSM)
help-fetch-dir = Projektmappa
help-fetch-offline = Csak a gyorsítótárat használja, hálózati hozzáférés nélkül
help-arg-cache-dir = A letöltött csempék gyorsítótármappája (alapértelmezés: a rendszer gyorsítótármappája + `freemapper`)
help-derive = A sablonok (ortofotó, domborzatárnyékolás, lejtő, helyi domborzat, nDSM), szintvonalak és domborzati formák számítása a letöltött adatokból
help-derive-dir = Projektmappa
help-run = Letöltés, számítás és export egyben
help-run-dir = Projektmappa
help-run-offline = Csak a gyorsítótárat használja, hálózati hozzáférés nélkül
help-run-ocad = OCAD-kezdőpontot is ír az `ocad/` mappába (OCAD 12 formátum)
help-run-pdf = A `<name>.pdf` fájlt is megírja, a nyomtatható térképet (csak a térképobjektumok, jelöltek nélkül)
help-run-pdf-candidates = A `<name>-candidates.pdf` fájlt is megírja, az ellenőrző nyomatot: a térkép minden jelölttel, helyhűen
help-run-training = Az edzéstérképet `<name>-training.omap` és PDF-jét `<name>-training.pdf` is megírja: a térkép és minden jelölt, úgy megtisztítva, hogy úgy nyomtatható, ahogy van; a --ocad kapcsolóval az `ocad/<name>-training.ocd` fájlt is. A meglévő edzéstérkép marad, kivéve ha megadja a --force kapcsolót
help-run-view = A --training kapcsolóval: az edzéstérképet azon részek nélkül is megírja, amelyeket egy nézet kihagy, `<name>-training-<view>.omap` néven és PDF-jével (a --ocad kapcsolóval OCAD-fájllal is). Nézetek: `no-paths` (nincsenek utak, ösvények, lépcsők), `brown-only` (csak barna jelek és a keret), `no-north-lines`, vagy egy `no:<codes>` / `only:<codes>` lista; a `+` kombinálja őket. A meglévő nézetfájl marad, kivéve ha megadja a --force kapcsolót
help-run-pdf-scale-bar = Méretarány-vonalzó rajzolása a PDF-ekre (alapból ki; a méretarány száma a címblokkban marad)
help-run-pdf-sources = Forrássorok a PDF-eken: az `auto` a teljes listát nyomtatja, és csak akkor rövidíti, ha több mint 5 sor lenne, a `full` sosem rövidít, a `credit-only` csak a freemapper-forrásmegjelölést nyomtatja (az ODbL és a dl-de/by-2-0 ilyenkor megköveteli a források megnevezését, ha továbbadja a térképet). Az ATTRIBUTION.txt mindig felsorol minden forrást
help-select = A térképterület kiválasztása a böngészőben: megrajzolja, vagy kiválaszt egy helyet. A mentés megírja a freemapper.toml fájlt, és szükség esetén létrehozza a projektmappát
help-select-dir = Projektmappa (mentéskor jön létre, ha nem létezik)
help-select-kind = Egy új projekt térképfajtája
help-select-title = Egy új projekt térképcíme
help-select-no-browser = Csak az oldal címét írja ki; nem nyitja meg a böngészőt
help-select-port = Port a 127.0.0.1 címen (alapértelmezés: egy szabad)
help-select-offline = Az oldal a „csak gyorsítótár, hálózat nélkül” bejelölésével indul
help-select-ocad = Az oldal az OCAD-kezdőprojekt bejelölésével indul
help-select-pdf = Az oldal a nyomtatható PDF bejelölésével indul
help-select-pdf-candidates = Az oldal az ellenőrző nyomat (minden jelölt) bejelölésével indul
help-select-training = Az oldal az edzéstérkép bejelölésével indul
help-select-pdf-scale-bar = Az oldal a PDF-ek méretarány-vonalzójának bejelölésével indul
help-select-pdf-sources = Az oldal a PDF-ek forrássorainak ezzel a választásával indul, mint a `run` esetén
help-find = Egy névhez tartozó OpenStreetMap-helyek listázása, területek elöl, az init --osm azonosítójával
help-find-query = A hely neve, pl. "Clara-Zetkin-Schule Strausberg"
help-find-json = A találatok kiírása JSON-ként, ugyanazokkal a mezőkkel, mint az MCP find_place eszköze
help-mcp = Az eszközöket MCP-n (stdin/stdout) keresztül egy MI-ügynöknek szolgáltatja
help-update = Újabb freemapper telepítése a GitHub-kiadásokból, rákérdezés után. A forrásból készült build ehelyett azt mondja meg, hogyan frissíthető
help-update-check = Csak ellenőriz: a kilépési kód 10, ha létezik újabb verzió, és 0, ha ez naprakész
help-update-yes = Telepítés kérdezés nélkül; akkor kell, ha a bemenet nem terminál
help-update-version = Ennek a verziónak (x.y.z) a telepítése a legújabb helyett, régebbié is
help-update-rollback = Visszalépés arra a verzióra, amelyet az utolsó frissítés lecserélt
help-update-mode = Azt tárolja, mit tegyen a napi ellenőrzés, ha újabb verziót talál (alapértelmezés: notify). Minden buildben működik, mert csak a beállítást írja
help-adapters = A régióadapterek megjelenítése: azok a tartományok és országok, amelyekhez a freemappernek vannak adatforrásai
help-adapters-list = Az adapterek listázása, hogy honnan származnak (beépített, a felhasználó mappája, egy projekt), a revíziójukkal, az utolsó ellenőrzésük napjával és a kérdezett hosztokkal
help-adapters-list-json = A lista kiírása JSON-ként
help-adapters-list-project = A projektmappa `adapters` mappájában lévő adaptereket is beolvassa, ahogy az --adapters-from-project tenné
help-adapters-check = Egy adapter mappájának ellenőrzése használata vagy továbbadása előtt: betöltődne-e, mit kell tudnia a szerzőjének, és a --live kapcsolóval azt is, hogy válaszolnak-e a szolgáltatásai. Kilépési kód 0: megfelel; 1: vannak megállapításai; 2: az ellenőrzés nem futhatott
help-adapters-check-dir = Az adapter mappája; a neve az adapter azonosítója
help-adapters-check-live = A --bbox által megadott téglalapra az adapter szolgáltatásait is lekérdezi, és megmondja, mit szolgáltat mindegyik. A kérések a manifestben deklarált hosztokon maradnak
help-adapters-check-bbox = Az élő ellenőrzés téglalapja fokban, minLon,minLat,maxLon,maxLat alakban. Az adapter határain belül van; 150 m × 150 m elég
help-adapters-check-json = Az eredmény kiírása JSON-ként
help-adapters-probe = Minden, a beépített adapterek által megnevezett szolgáltatást egyszer lekérdez, és megmondja, válaszol-e: ok, slow, moved vagy error. A kilépési kód akkor is 0, ha egy szolgáltatás hibázik; az eredmény maga a válasz
help-adapters-probe-json = Az eredmény kiírása JSON-ként
help-adapters-probe-status = Az előző éjszaka állapotát ebből a fájlból olvassa (hiányozhat), és az új állapotot JSON-ként írja ki: egy szolgáltatás két egymást követő rossz éjszaka után számít kiesettnek. A változások a standard hibakimenetre mennek
help-adapters-allow-host = Egy beépített adapternek még egy hoszt engedélyezése: olyan tartományhoz, amelynek szolgáltatása olyan szerverre költözött, amelyet az adaptere még nem nevez meg. Az elutasított címről szóló üzenet megmondja, melyik hoszt. A hosztot úgy ellenőrzi, mint egy manifest hosztját, és a freemapper továbbra is csak internetes szerverekhez kapcsolódik
help-adapters-allow-host-adapter = A beépített adapter azonosítója, ahogy az `adapters list` mutatja
help-adapters-allow-host-host = Az engedélyezendő hoszt: DNS-név, nem cím, nem port
help-adapters-disallow-host = Egy hoszt visszavonása, amelyet az allow-host engedélyezett
help-adapters-disallow-host-adapter = A beépített adapter azonosítója
help-adapters-disallow-host-host = A visszavonandó hoszt
help-adapters-export = A tartomány beépített adapterét módosítható adaptermappaként írja ki: a jegyzékfájlját és a jegyzékfájl által megnevezett fájlokat a `<dir>/<id>` mappába. A mappa még nem létezhet, vagy üresnek kell lennie. Módosítsa a címeket a `manifest.toml` fájlban, ellenőrizze az `adapters check` paranccsal, és tegye az adaptermappájába: ekkor figyelmeztetéssel felváltja a beépített adaptert. A mappa eltávolítása visszahozza a beépítettet
help-adapters-export-adapter = A beépített adapter azonosítója, ahogy az `adapters list` mutatja
help-adapters-export-dir = A mappa, amelyben a `<id>` mappa létrejön
help-sources = Az adatforrások címeinek olvasása és javítása új kiadás nélkül. A javítás a saját adaptermappádba kerül, és felülírja a beépített címet
help-sources-list = A jegyzékek (manifestek) címeinek listázása útvonalukkal és az egyes jegyzékek eredetével. Adapterazonosítóval csak annak az adapternek a címei
help-sources-get = Egy cím kiírása, semmi más. 1-es kilépési kód, ha az adapter vagy az útvonal ismeretlen
help-sources-set = Egy cím javítása a saját adaptermappádban, miután az egész jegyzék átment az ellenőrzésen. Olyan gazdagépen lévő cím, amelyet a jegyzék nem deklarál, elutasításra kerül, hacsak nem adod meg az --add-host kapcsolót
help-sources-diff = Megmutatja, mit változtat az adaptermappád a beépített jegyzékhez képest
help-sources-check = Az adott azonosítójú adapter mappájának ellenőrzése, mint az `adapters check`. Kilépési kód 0: megfelelt; 1: vannak észrevételek; 2: az ellenőrzés nem futhatott
help-sources-schema = A jegyzék címet tartalmazó kulcsainak kiírása, --json esetén JSON-sémaként
help-arg-adapter = Az adapter azonosítója, ahogy az `adapters list` mutatja
help-arg-path = A cím útvonala, ahogy a `sources list` mutatja
help-sources-set-url = Az új cím: https, lekérdezés nélkül
help-sources-set-add-host = Az új cím gazdagépét is felvenni a jegyzék által deklarált gazdagépek közé
cli-sources-no-user-dir = ennek a rendszernek nincs mappája az adaptereidnek; állítsd a FREEMAPPER_ADAPTER_DIR változót abszolút útvonalra
cli-sources-bad-id = a(z) { $adapter } nem adapterazonosító
cli-sources-unknown-adapter = a freemapper nem ismer { $adapter } nevű adaptert; a `freemapper adapters list` mutatja az azonosítókat
cli-sources-no-manifest = a program egyetlen jegyzéke sem írja le a(z) { $adapter } adaptert; a címei a kódban vannak
cli-sources-unknown-path-of = a(z) { $adapter } adapternek nincs címe a(z) { $path } alatt; a `freemapper sources list { $adapter }` mutatja az útvonalakat
cli-sources-bad-value = az új címnek https-címnek kell lennie megnevezett gazdagépen, idézőjel, fordított perjel, vezérlőkarakter, felhasználó és port nélkül
cli-sources-host-not-declared = az új cím a(z) { $host } gazdagéphez vezet, amelyet a jegyzék nem deklarál; az --add-host deklarálja
cli-sources-invalid = a jegyzék ezzel a változtatással nem felelne meg az ellenőrzésnek, ezért semmi sem íródott ki: { $findings }
cli-sources-set-created = A(z) { $adapter } adapter { $folder } mappája a beépített jegyzékéből készült; a következő indítástól felülírja a beépített adaptert.
cli-sources-set-host-added = A(z) { $host } gazdagép mostantól deklarált a [net] hosts alatt; a freemapper ezt a szervert kérdezi ehhez az adapterhez.
cli-sources-set-next = Ellenőrizd a `freemapper adapters check { $folder }` paranccsal. A forrásmegjelölés és a licenc változatlan marad: tartsd helyesen, ha a szolgáltatás gazdát cserélt. Indítsd újra az MCP-szervert vagy a terület oldalát.
cli-sources-diff-no-folder = nincs mappád a(z) { $adapter } adapterhez ({ $folder }); nincs mit összehasonlítani
cli-sources-diff-none = a mappád ugyanazokat a címeket tartalmazza, mint a beépített jegyzék
help-sources-list-json = A lista kiírása JSON-ként
help-sources-check-live = A --bbox által megadott téglalapra az adapter szolgáltatásait is lekérdezi, és megmondja, mit szolgáltat mindegyik. A kérések a manifestben deklarált hosztokon maradnak
help-sources-check-bbox = Az élő ellenőrzés téglalapja fokban, minLon,minLat,maxLon,maxLat alakban. Az adapter határain belül van; 150 m × 150 m elég
help-sources-check-json = Az eredmény kiírása JSON-ként
help-sources-schema-json = Az eredmény kiírása JSON-ként
help-arg-adapters-from-project = A projektmappa `adapters` mappájában lévő adaptereket is betölti. A freemapper mindegyiket megnevezi a kérdezett hosztokkal. Használja megbízható projekthez: egy adapter a freemappert az általa megnevezett szerverek kérdezésére készteti

## The summary of a run, for the MCP server and the area page

cli-internal-error = belső hiba: { $message }
cli-pdf-sources-invalid = a pdf_sources értéke auto, full vagy credit-only lehet, nem { $value }
cli-view-unknown = a nézet no-paths, brown-only, no-north-lines, no:<codes> vagy only:<codes> lehet, nem { $value }
cli-view-empty = a(z) { $value } lista nem nevez meg jelkódot
cli-view-bad-code = { $value } nem jelkód, mint például 505 vagy 501.6
cli-summary-training-view = Megírva: { $map } és { $pdf } (az edzéstérkép egy nézete)
cli-summary-kept-training-view = Az edzéstérkép { $view } nézetének fájljai megtartva; a megújításhoz adja meg a --force kapcsolót
cli-summary-training-view-failed = Az edzéstérkép nézete sikertelen: { $error }
cli-estimate-disk-unreadable = nem olvasható a szabad lemezterület itt: { $path } ({ $message })
cli-summary-wrote-map = A térkép megírva: { $path }
cli-summary-kept-map = A meglévő térkép megtartva: { $path } (tartalmazhatja a térképező munkáját)
cli-summary-map-lacks-templates = A térkép még nem hivatkozik a képsablonokra. Indítsa a folyamatot force beállítással a lecseréléséhez; a régit .omap.bak néven megtartja.
cli-summary-map-lacks-buildings = A térképen még nincsenek ALKIS-épületek. Hozzáadásukhoz indítsa a folyamatot force beállítással; lecseréli a térképet, és a régit .omap.bak néven megtartja.
cli-summary-map-lacks-canopies = A térképről hiányoznak az ALKIS-féle fedett részek és autóbeállók (522), amelyek most már térképobjektumok. Hozzáadásukhoz indítsa a folyamatot force beállítással; lecseréli a térképet, és a régit .omap.bak néven megtartja.
cli-summary-map-light = Az alaptérkép: { $joined } vonal egyesítve, { $left } rejtett vagy túl kicsi objektum a jelölt rétegekben maradt (semmi sem mozdult el)
cli-summary-map-light-kept = A jelölt rétegek most { $left } olyan objektumot vagy szakaszt is tartalmaznak, amelyet ez a verzió levesz az alaptérképről (freemapper:generalized címkével). A megtartott térképen még megvannak, ezért kétszer látszanak. A térkép tisztításához indítsa a folyamatot force beállítással; a régit .omap.bak néven megtartja.
cli-summary-map-old-candidates = A térkép még a templates/candidates.omap fájlt nevezi meg; ez a futás minden jelölttel feltöltötte. A jelöltek mostantól hét rétegben is megvannak, rétegenként egy fájlban (templates/candidates-*.omap). A térkép egy force beállítású futás után listázza őket (a régi térkép .omap.bak néven marad), vagy a térképező maga adja hozzá őket az OOM-ban (Sablonok > Sablon megnyitása…).
cli-summary-wrote-ocad = Az OCAD-térkép megírva: { $path } ({ $count } objektum)
cli-summary-kept-ocad = A meglévő OCAD-térkép megtartva: { $path } (tartalmazhatja a térképező munkáját; a force lecseréli)
cli-summary-ocad-old-candidates = Az OCAD-térkép még a(z) { $path } fájlt nevezi meg háttértérképként; ez a futás minden jelölttel feltöltötte. A jelöltek mostantól hét rétegben is megvannak, rétegenként egy fájlban (-candidates-*.ocd). Az OCAD-térkép egy force beállítású futás után listázza őket (a régi térkép .ocd.bak néven marad), vagy a térképező maga adja hozzá őket az OCAD-ban háttértérképként.
cli-summary-ocad-left-out = Kihagyva az OCAD-térképből: { $skipped } objektum és { $dropped } elfajult rész
cli-summary-ocad-symbols = Az OCAD-ban válassza a Symbol > Load Colors and Symbols From… menüpontot, és adja meg a hivatalos jelkulcsot{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] {" "}az IOF iskolai jelkulcs 2019
       *[other] {" "}ISSprOM 2019
    }; ez a helyettesítő jeleket szám szerint kicseréli.
cli-summary-ocad-failed = Az OCAD-export sikertelen: { $error }
cli-summary-pdf-without-credits = PDF forrásmegjelölés nélkül: az ODbL és a dl-de/by-2-0 megköveteli a források megnevezését, ha továbbadja a térképet; lásd ATTRIBUTION.txt
cli-summary-wrote-pdf = A térkép megírva: { $path } (jelöltek nélkül)
cli-summary-pdf-failed = A PDF-export sikertelen: { $error }
cli-summary-wrote-pdf-candidates = Az ellenőrző nyomat megírva: { $path } (a térkép minden jelölttel, helyhűen)
cli-summary-pdf-candidates-failed = Az ellenőrző nyomat exportja sikertelen: { $error }
cli-summary-training-failed = Az edzéstérkép exportja sikertelen: { $error }
cli-summary-kept-training = A meglévő edzéstérkép megtartva: { $path }, hogy egy futás ne írjon felül olyat, amelyet kinyomtattak vagy továbbadtak; törölje, vagy a megújításhoz állítsa be a force értéket

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = ezen a rendszeren nincs saját mappa; használjon abszolút útvonalat
mcp-error-relative-path = a(z) { $path } relatív útvonal; használjon abszolút útvonalat vagy ~-gal kezdődőt
mcp-error-home-folder = a(z) { $path } maga a saját mappa; használjon egy benne lévő mappát, pl. ~/maps/school
mcp-error-no-project = nincs freemapper-projekt itt: { $path }; előbb használja a create_project eszközt
mcp-error-unknown-kind = ismeretlen kind: { $kind }; használja: "schoolyard", "sprint" vagy "forest"
mcp-error-reading = { $path } beolvasása: { $error }
mcp-error-one-area-source = pontosan az egyiket adja meg: osm_id (a find_place eszközből) vagy geojson
mcp-error-page-open = ennek a projektnek a területválasztó oldala még nyitva van; kérje meg a felhasználót, hogy mentse el a területet, és nyomja meg az oldalon a „Kész” gombot, majd indítsa el a folyamatot
mcp-error-no-job = nincs { $job } feladat; a start_pipeline és a permits_for_area ennek a munkamenetnek a feladatazonosítóit adja vissza
mcp-error-job-running = a(z) { $job } még fut ehhez: { $path }; egyszerre egy folyamat fut, mert a projektek közös letöltési gyorsítótáron osztoznak. Ezt akkor indítsa, ha a project_status finished vagy failed állapotot jelent
mcp-error-page-start = az oldal nem indítható: { $error }
mcp-created-project = Projekt létrehozva: { $path } ehhez: { $area } ({ $kind }, 1:{ $scale }). Következő: estimate_download, majd start_pipeline.
mcp-created-project-given-area = Projekt létrehozva: { $path } a megadott területhez ({ $kind }, 1:{ $scale }). Következő: estimate_download, majd start_pipeline.
mcp-page-already-open = Ennek a mappának az oldala már nyitva van; azokat a beállításokat tartja meg, amelyekkel megnyitották.
mcp-page-next = Kérje meg a felhasználót, hogy válassza ki a területet az oldalon, nyomja meg a „Mentés”, majd a „Kész” gombot.
mcp-preview-of = Előnézet: { $layer }
mcp-permits-next = Az engedélyezési tudnivalók készülnek. Kérdezze le a project_status eszközt ezzel a job_id-vel; ha kész, a permits_md a permits.md szövegét tartalmazza. Útmutatásként adja tovább, nem jogi tanácsként.
mcp-tool-check-update = Létezik-e újabb freemapper-kiadás: verziója, kiadási dátuma, kiadási jegyzetei és letöltési mérete, vagy hogy ez naprakész. Nem telepít semmit.
mcp-tool-create-project = Projektmappa létrehozása egy térképterülethez. Adjon meg osm_id-t (a find_place eszközből) vagy egy GeoJSON-fájlt. A kind értéke schoolyard (1:1000, alapértelmezett), sprint (1:4000) vagy forest (1:15000).
mcp-param-create-project-dir = Új projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-param-create-project-geojson = Az osm_id helyett: GeoJSON-fájl a térképterület sokszögével (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (alapértelmezett, 1:1000), "sprint" (1:4000) vagy "forest" (1:15000).
mcp-param-create-project-osm-id = A térképterület OSM-objektuma a find_place eszközből, pl. "way/992423119".
mcp-param-create-project-part = Az osm_id-val: egy több különálló részből álló terület melyik részét használja (az 1. a legnagyobb).
mcp-param-create-project-scale = Méretaránynevező: iskolaudvar 1000 (alapértelmezett); sprint 4000 (alapértelmezett) vagy 5000; erdő 15000 (alapértelmezett), 10000 vagy 7500. Más értékeket elutasít; 1:3000-hez 4000-ben rajzoljon, a freemapper-print nyomtatáskor nagyít.
mcp-param-create-project-title = A térkép címe a peremi információkhoz (alapértelmezés: a mappa neve).
mcp-tool-estimate-download = Mit tölt le a start_pipeline egy projekthez: a csempéket, minden letöltést a méretével MB-ban, hogy gyorsítótárban van-e, és hogy elfér-e a lemezen, a még letöltendő MB-ot, valamint a disk alatt a gyorsítótár és a projekt kötetének szabad helyét üzenetekkel arról, ami nem fér el (mondja el a felhasználónak).
mcp-param-estimate-download-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-tool-find-place = Iskola, park vagy más hely keresése név alapján (OpenStreetMap Nominatim). Legfeljebb 5 találatot ad vissza, területek elöl; csak a has_area találatok lehetnek térképterület. Az üres lista azt jelenti, hogy nem találtak semmit.
mcp-param-find-place-query = Az iskola vagy a hely neve, pl. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = A legújabb freemapper-kiadás telepítése e program fölé. Csak azután hívja, hogy a személy beleegyezett, confirm true értékkel. Az új verzió akkor él, amikor az asszisztens újraindítja a freemapper-szervert.
mcp-param-install-update-confirm = true, ha a személy beleegyezett az újabb verzió telepítésébe; enélkül semmi sem települ.
mcp-tool-list-layers = Egy projekt rétegei: a térkép, a hét jelölt réteg és a raszteres sablonok, mindegyik a fájljával, hogy elkészült-e már, és melyik lépés készíti.
mcp-param-list-layers-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-tool-open-area-selector = Megnyit egy oldalt a böngészőben, ahol a felhasználó megrajzolja a térképterületet (téglalap vagy sokszög) vagy kiválaszt egy helyet, látja a csempéket és a letöltés méretét, és elmenti; a mentés létrehozza a projektmappát vagy módosítja a területét. A felhasználónak átadandó URL-t adja vissza. Az oldal addig fut, amíg a felhasználó meg nem nyomja a „Kész” gombot; ugyanahhoz a mappához újra hívva ugyanazt az URL-t adja vissza.
mcp-param-open-area-selector-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő); a felhasználó mentésekor jön létre.
mcp-param-open-area-selector-kind = Egy új projekt térképfajtája: "schoolyard" (alapértelmezett, 1:1000), "sprint" (1:4000) vagy "forest" (1:15000).
mcp-param-open-area-selector-ocad = A futtatási panel az OCAD-kezdőprojekt bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-offline = A futtatási panel a „csak gyorsítótár, hálózat nélkül” bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-open-browser = Az oldalt ennek a számítógépnek az alapértelmezett böngészőjében is megnyitja. Alapértelmezés: true.
mcp-param-open-area-selector-pdf = A futtatási panel a nyomtatható PDF bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-pdf-candidates = A futtatási panel az ellenőrző nyomat (minden jelölt) bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-pdf-scale-bar = A futtatási panel a PDF-ek méretarány-vonalzójának bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-pdf-sources = A PDF-ek forrássorai, amelyekkel a panel indul: "auto" (alapértelmezett), "full" vagy "credit-only", mint a start_pipeline esetén.
mcp-param-open-area-selector-permits = A futtatási panel az engedélyezési tudnivalók bejelölésével indul. Alapértelmezés: false.
mcp-param-open-area-selector-scale = Egy új projekt méretaránynevezője: iskolaudvar 1000 (alapértelmezett); sprint 4000 (alapértelmezett) vagy 5000; erdő 15000 (alapértelmezett), 10000 vagy 7500. Más értékeket elutasít.
mcp-param-open-area-selector-title = Egy új projekt térképcíme.
mcp-param-open-area-selector-training = A futtatási panel az edzéstérkép bejelölésével indul. Alapértelmezés: false.
mcp-tool-preview-layer = Egy réteg PNG-előnézete (legfeljebb 800 px): a map (a térképobjektumok képe, amelyet az utolsó futás megírt, jelöltek nélkül), vagy egy raszteres sablon: orthophoto, hillshade, slope, relief, ndsm, vagy k5 Berlinben.
mcp-param-preview-layer-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" vagy "k5" (csak Berlin).
mcp-tool-project-status = A start_pipeline vagy a permits_for_area egy feladatának állapota: running, finished vagy failed, az utolsó folyamatsorok, figyelmeztetések, a megírt fájlok, valamint az eredmény vagy a hiba. Egy befejezett permits_for_area feladat a permits.md szövegét is hozza permits_md néven.
mcp-param-project-status-job-id = A start_pipeline vagy a permits_for_area feladatazonosítója, pl. "job-1".
mcp-tool-permits-for-area = Egy projekt engedélyezési tudnivalóinak elkészítése: a térképterület közigazgatási egységei, a benne lévő védett területek és hogy ki kezeli ott a földet, a megkérdezendő hatóságokkal; útmutatás, nem jogi tanács. A permits.md és a permits.json fájlt a mappába írja. A háttérben fut, és job_id-t ad vissza; kérdezze le a project_status eszközt, amelynek válasza a feladat végeztével a permits.md szövegét hozza permits_md néven. Egyszerre egy feladat fut.
mcp-param-permits-for-area-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-param-permits-for-area-offline = Csak azokat az adatokat olvassa, amelyeket egy korábbi futás a projektben tárolt, hálózati hozzáférés nélkül. Alapértelmezés: false.
mcp-tool-start-pipeline = Az adatok letöltése, a sablonok számítása és a kezdőprojekt megírása (fetch, derive, export). A háttérben fut, és job_id-t ad vissza; kérdezze le a project_status eszközt. Egyszerre egy folyamat fut. Egy meglévő térképet megtart, hacsak a force nem true.
mcp-param-start-pipeline-dir = Projektmappa (abszolút útvonal vagy ~-gal kezdődő).
mcp-param-start-pipeline-force = Egy meglévő térképfájl lecserélése, a régit .omap.bak néven megtartja. Alapértelmezés: false.
mcp-param-start-pipeline-ocad = OCAD-kezdőpontot is ír az ocad/ mappába (OCAD 12 formátum, az OCAD-ban szerkesztőknek). Alapértelmezés: false.
mcp-param-start-pipeline-offline = Csak a letöltési gyorsítótárat használja, hálózati hozzáférés nélkül. Alapértelmezés: false.
mcp-param-start-pipeline-pdf = A <name>.pdf fájlt is megírja, a nyomtatásra kész térképet (csak a térképobjektumok, jelöltek nélkül). Alapértelmezés: false.
mcp-param-start-pipeline-pdf-candidates = A <name>-candidates.pdf fájlt is megírja, az ellenőrző nyomatot: a térkép minden jelölttel, helyhűen (ellenőrizze őket a terepen). Alapértelmezés: false.
mcp-param-start-pipeline-pdf-scale-bar = Méretarány-vonalzó rajzolása a PDF-ekre. Alapértelmezés: false (a német térképeken általában nincs; a méretarány száma a címblokkban marad).
mcp-param-start-pipeline-pdf-sources = Forrássorok a PDF-eken: "auto" (alapértelmezett; a teljes lista, csak akkor rövidítve, ha több mint 5 sor lenne), "full" (sosem rövidít) vagy "credit-only" (csak a freemapper-forrásmegjelölés; az ODbL és a dl-de/by-2-0 ilyenkor megköveteli a források megnevezését, ha a térképet továbbadják). Az ATTRIBUTION.txt mindig felsorol minden forrást.
mcp-param-start-pipeline-permits = A térkép után az engedélyezési tudnivalókat (permits.md és permits.json) is megírja: közigazgatási egységek, védett területek és hogy ki kezeli a földet. Az ottani hiba figyelmeztetés, nem sikertelen feladat. Alapértelmezés: false.
mcp-param-start-pipeline-training = Az edzéstérképet <name>-training.omap és PDF-jét <name>-training.pdf is megírja: a térkép és minden jelölt, úgy megtisztítva, hogy úgy nyomtatható, ahogy van; az ocad-dal az ocad/<name>-training.ocd fájlt is. A meglévő edzéstérképet megtartja, hacsak a force nem true. Alapértelmezés: false.
mcp-param-start-pipeline-training-views = A training mellett: az edzéstérképet azon részek nélkül is megírja, amelyeket egy nézet kihagy, bejegyzésenként egy fájlkészlet: "no-paths", "brown-only", "no-north-lines", vagy egy lista, például "no:505,506" vagy "only:101,102", +-szal kombinálva. A fájlok a <name>-training-<view>.omap és PDF-je.
mcp-instructions = A freemapper OpenOrienteering Mapper kezdőprojekteket készít tájfutótérképekhez (iskolaudvar 1:1000, sprint 1:4000, erdő 1:15000) a(z) { $states } hivatalos geoadataiból és az OSM-ből. Munkafolyamat: find_place (az iskola vagy terület keresése név alapján; válasszon egy has_area találatot) → create_project (új mappa, abszolút útvonal, az osm_id-val) → estimate_download (mondja meg a felhasználónak, mennyi töltődik le) → start_pipeline (feladatazonosítót ad vissza; a letöltés percekig tarthat) → project_status (néhány másodpercenként kérdezze le, amíg az állapot finished vagy failed nem lesz) → list_layers és preview_layer (mutassa meg a felhasználónak a sablonokat). A felhasználó ezután megnyitja a térképfájlt az OpenOrienteering Mapperben. Ha egyetlen hely sem megfelelő, vagy a felhasználó meg akarja rajzolni a területet, az open_area_selector megnyit egy oldalt a böngészőben, ahol a felhasználó megrajzolja vagy kiválasztja a területet, és elmenti (ez létrehozza a projektet); ha a felhasználó azt mondja, hogy el van mentve, folytassa az estimate_download eszközzel. Az útvonalaknak abszolútnak kell lenniük, vagy ~-gal kell kezdődniük. Egyszerre egy folyamat fut. A permits_for_area egy projekt engedélyezési tudnivalóit (közigazgatási egységek, védett területek, ki kezeli a földet; útmutatás, nem jogi tanács) készíti el a start_pipeline-hoz hasonló feladatként. A check_update megmondja, létezik-e újabb freemapper; az install_update telepíti, csak confirm: true értékkel, miután a személy beleegyezett, és az asszisztens ezután újraindítja a freemapper-szervert.

cli-contour-interval-saved = A szintvonalköz elmentve a freemapper.toml fájlban: { $interval } m; a derive ennél vezeti le a szintvonalakat
cli-contour-interval-auto = A szintvonalköz a freemapper.toml fájlban: automatikus (a térképfajtáé; az erdei térkép síkvidéken 2,5 m-t, meredek vidéken 10 m-t kap)
cli-steep-slope-saved = A meredekségi küszöb elmentve a freemapper.toml fájlba: { $slope }°; a derive ezzel keresi a rézsűket és a sziklafalakat
cli-steep-slope-auto = Meredekségi küszöb a freemapper.toml fájlban: alapértelmezett (kb. 27°)
cli-face-slope-saved = A falküszöb elmentve a freemapper.toml fájlba: { $slope }°; a derive ezzel keresi a sziklafalfelületeket
cli-face-slope-auto = Falküszöb a freemapper.toml fájlban: alapértelmezett (70°)
help-arg-contour-interval = A szintvonalköz méterben, pl. `2.5`, vagy `auto` a térképfajta értékéhez (erdei térkép: 5 m, ahol sík, 2,5 m, ahol meredek, 10 m). A freemapper.toml fájlban mentődik; a derive ennél vezeti le a szintvonalakat
help-arg-steep-slope = A lejtőszög fokban, amelytől a terep meredeknek számít a rézsűk és sziklafalak keresésekor, pl. `55`, vagy `auto` az alapértelmezett 27°-hoz. Emelje (55 és 65 közé) sziklavárosokban és szurdokokban, ahol szinte minden lejtő meredek, és a rézsűk meg a sziklafalak egyetlen foltba olvadnak. A freemapper.toml fájlba kerül; a derive ezzel keresi a rézsűket és a sziklafalakat
help-arg-face-slope = Lejtőszög fokban, amelytől a terep sziklafal (fekete felület, 206-os jel), pl. `75`, vagy `auto` a 70°-os alapértékhez. A 65 több és nagyobb felületet ad, a meredek lejtőkön is; a 75 csak a valódi falakat hagyja meg. A freemapper.toml fájlba kerül; a derive ezzel keresi a sziklafalakat
cli-content-saved = A tartalombeállítás elmentve a freemapper.toml fájlban: { $summary }
cli-content-invalid = A tartalombeállítás nem olvasható: { $detail }
help-arg-content = Mi kerül közvetlenül a térképre: a `safe` minden objektumot ott hagy, ahová a feldolgozás teszi, a `balanced` (alapértelmezett) a kataszter és a nyilvántartások objektumait a térképre teszi, a `full` mindent a térképre tesz. A választás a freemapper.toml fájlban mentődik
help-arg-content-set = Objektumcsoport küldése a `map`, `candidates` vagy `off` helyre, `GROUP=PLACE` alakban, pl. `walls=map`; ismételhető. Csoportok: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. A freemapper.toml fájlban mentődik
mcp-param-create-project-content = Mi kerül közvetlenül a térképre: "safe", "balanced" (alapértelmezett) vagy "full". A projektben mentődik. A balanced a kataszter és a nyilvántartások objektumait a térképre teszi, a full mindent.
mcp-param-create-project-content-set = A profil szerintitől eltérő helyre küldött csoportok, "group=place" alakban, ahol a place map, candidates vagy off, pl. "walls=map". Csoportok: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = A szintvonalköz méterben, pl. 2.5; a 0 visszatér az automatikus választáshoz (a térképfajtáé; az erdei térkép síkvidéken 2,5 m-t, meredek vidéken 10 m-t kap). A projektben mentődik; a következő derive ennél vezeti le a szintvonalakat.
mcp-param-create-project-steep-slope = A lejtőszög fokban, amelytől a terep meredeknek számít a rézsűk és sziklafalak keresésekor, pl. 55; a 0 visszaállítja az alapértelmezett 27°-ot. Emelje (55 és 65 közé) sziklavárosokban és szurdokokban, ahol szinte minden lejtő meredek, és a rézsűk meg a sziklafalak egyetlen foltba olvadnak. A projektben tárolódik; a következő derive ezzel keresi a rézsűket és a sziklafalakat.
mcp-param-create-project-face-slope = Lejtőszög fokban, amelytől a terep sziklafal (fekete felület, 206-os jel), pl. 75; a 0 visszaáll a 70°-os alapértékre. A 65 több és nagyobb felületet ad, a meredek lejtőkön is; a 75 csak a valódi falakat hagyja meg. A projektbe kerül; a következő derive ezzel keresi a sziklafalakat.
mcp-param-start-pipeline-content = A futás előtt módosítja, mi kerül közvetlenül a térképre: "safe", "balanced" vagy "full". A projektben mentődik. Alapértelmezés: a projekt beállítása, egyébként balanced.
mcp-param-start-pipeline-content-set = A profil szerintitől eltérő helyre küldött csoportok, "group=place" alakban, ahol a place map, candidates vagy off, pl. "fences=candidates". A projektben mentődik.
mcp-param-start-pipeline-contour-interval = A szintvonalköz méterben, pl. 2.5; a 0 visszatér az automatikus választáshoz (a térképfajtáé; az erdei térkép síkvidéken 2,5 m-t, meredek vidéken 10 m-t kap). A projektben mentődik; a következő derive ennél vezeti le a szintvonalakat.
mcp-param-start-pipeline-steep-slope = A lejtőszög fokban, amelytől a terep meredeknek számít a rézsűk és sziklafalak keresésekor, pl. 55; a 0 visszaállítja az alapértelmezett 27°-ot. Emelje (55 és 65 közé) sziklavárosokban és szurdokokban, ahol szinte minden lejtő meredek, és a rézsűk meg a sziklafalak egyetlen foltba olvadnak. A projektben tárolódik; a következő derive ezzel keresi a rézsűket és a sziklafalakat.
mcp-param-start-pipeline-face-slope = Lejtőszög fokban, amelytől a terep sziklafal (fekete felület, 206-os jel), pl. 75; a 0 visszaáll a 70°-os alapértékre. A 65 több és nagyobb felületet ad, a meredek lejtőkön is; a 75 csak a valódi falakat hagyja meg. A projektbe kerül; a következő derive ezzel keresi a sziklafalakat.
mcp-error-content = A tartalombeállítás nem olvasható: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = A projekt engedélyezési tudnivalóinak megírása (permits.md és permits.json): a térképterületet érintő közigazgatási egységek, védett területek és üzemeltetők, és hogy ez általában kinek a megkérdezését jelenti. Lekérdezi a BfN-t és a BKG-t, és ahol a térképterület tartományának saját nyílt szolgáltatásai vannak, azokat az adapterének hosztjain keresztül (lista a kézikönyv 12. fejezetében); Észak-Rajna-Vesztfáliában ehhez először kb. 32 MB-ot tölt le, Brandenburgban kb. 4 MB-ot, később csak az újabb fájlokat. A már gyorsítótárazott OSM-kivonatot olvassa; az --offline a korábban letöltöttet használja
help-permits-dir = Projektmappa
help-permits-offline = Csak a korábban letöltött adatokat használja, hálózati hozzáférés nélkül
help-run-permits = A térkép után az engedélyezési tudnivalókat (permits.md és permits.json) is megírja; lekérdezi a BfN-t, a BKG-t és a tartomány nyílt szolgáltatásait (kézikönyv, 12. fejezet). Az ottani hiba figyelmeztetés, a térkép megmarad
cli-wrote-permits = Az engedélyezési tudnivalók megírva: { $path }
cli-summary-wrote-permits = Az engedélyezési tudnivalók megírva: { $path }
cli-permits-failed = az engedélyezési tudnivalókat nem sikerült megírni: { $error }
cli-permits-source-left-out = az engedélyezési tudnivalók kihagyják ezt: { $source }: { $reason }
cli-permits-not-read = nem lett beolvasva
error-permits-stored = { $path }: { $message }; töltse le újra online az engedélyezési tudnivalókat
error-permits-nothing-read = az engedélyezési tudnivalók egyik forrása sem olvasható ({ $missing }); előbb töltse le őket online
error-permits-missing-not-read = { $source }: nem lett beolvasva

permits-title = Engedélyezési tudnivalók
permits-disclaimer = Útmutatás, nem jogi tanács. Az illetékes hatóságok döntenek. Az adatok keltéhez és pontosságához lásd a „Források és az adatok kelte” alatti táblázatot; területek hiányozhatnak vagy elavultak lehetnek (BfN: évente egyszer egyesítik, „Nicht für Planungszwecke geeignet.”, vagyis tervezési célra nem alkalmas).
permits-disclaimer-regional = Útmutatás, nem jogi tanács. Az illetékes hatóságok döntenek. Az adatok keltéhez és pontosságához lásd a „Források és az adatok kelte” alatti táblázatot; területek hiányozhatnak vagy elavultak lehetnek.
permits-bfn-merge = A BfN évente egyszer egyesíti a tartományok adatait, és tervezési célra alkalmatlannak jelöli őket. A tartomány hatóságai döntenek.
permits-created = Készült: { $date }, egy { $kind ->
        [schoolyard] iskolaudvari térkép
        [forest] erdei térkép
       *[other] sprinttérkép
    } térképterületéhez ({ $area } ha).
permits-section-admin = Közigazgatási egységek
permits-section-protected = Védett területek (BfN)
permits-section-osm-protected = További védett területek az OpenStreetMapből
permits-section-operators = Üzemeltetők és tulajdonosok az OpenStreetMap szerint
permits-section-hints = Tudnivalók
permits-section-sources = Források és az adatok kelte
permits-none-admin = A VG250 nem nevez meg közigazgatási egységet a térképterülethez.
permits-none-protected = A BfN szerint nincs védett terület a térképterületen.
permits-none-osm-protected = Az OpenStreetMap nem nevez meg olyan védett területet a térképterületen, amelyet a BfN ne sorolna fel már.
permits-none-operators = Az OpenStreetMap nem nevez meg üzemeltetőt és tulajdonost a térképterülethez.
permits-none-hints = Nincsenek tudnivalók.
permits-section-not-read = A(z) { $source } adatai nem lettek beolvasva, ezért ez a szakasz üres, bár lehetnek bejegyzések.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] A BKG 1:250 000-es közigazgatási területei (VG250)
        [bfn] A BfN németországi védett területei
       *[osm] OpenStreetMap, abból a kivonatból, amelyet a freemapper a gyorsítótárában tart
    }
permits-level = { $level ->
        [land] Tartomány
        [kreis] Járás (Kreis)
        [verwaltungsgemeinschaft] Amt vagy Verwaltungsgemeinschaft
       *[gemeinde] Község (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = A szélén érinti (a térképterület kevesebb mint 0,5%-a):
permits-at-edge-generalised = A VG250 1:250 000-es méretarányra van általánosítva. Egy egység, amely csak a szélét érinti, ennek következménye lehet.
permits-kind = { $kind ->
        [nationalpark] Nemzeti park
        [nationales-naturmonument] Nemzeti természeti emlék
        [naturschutzgebiet] Természetvédelmi terület (Naturschutzgebiet)
        [ffh] FFH-terület (Natura 2000)
        [vogelschutzgebiet] Madárvédelmi terület (Natura 2000)
        [biosphaerenreservat] Bioszféra-rezervátum
        [biosphaerenreservat-zone] Bioszféra-rezervátum övezete
        [landschaftsschutzgebiet] Tájvédelmi terület (Landschaftsschutzgebiet)
       *[naturpark] Natúrpark
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Kijelölés dátuma: { $date }
permits-protected-title = Jogalap: { $title }
permits-protected-link = Jogszabály szövege: { $url }
permits-protected-site-link = A terület oldala (Natura 2000): { $url }
permits-protected-year = Az adatok éve a BfN-nél: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = név nélkül
permits-osm-operator = Üzemeltető az OSM szerint: { $operator }
permits-osm-enclosing-gap = Itt hiányozhatnak az olyan OpenStreetMap-területek, amelyek az egész térképterületet körbeveszik anélkül, hogy egy sarkuk benne lenne, valamint az OSM-ben `type=boundary` relációként térképezett védett területek. A BfN és a VG250 részei ezt nem érintik.
permits-osm-enclosing-gap-short = Az OpenStreetMap azon területei, amelyek az egész térképterületet körülveszik, de egyetlen sarkuk sincs benne, itt hiányozhatnak.
permits-operator-line = { $name } ({ $roles ->
        [operator] üzemeltető
        [owner] tulajdonos
       *[both] üzemeltető és tulajdonos
    }): { $objects }
permits-column-source = Forrás
permits-column-state = Állapot
permits-column-credit = Forrásmegjelölés és licenc
permits-state-read = beolvasva, letöltve: { $date }
permits-state-read-osm = beolvasva, OSM-kivonat kelte: { $date }
permits-state-unreachable = nem elérhető ({ $reason })
permits-state-not-read = nincs beolvasva
permits-no-credit = nincs használva
permits-hint-city-state = A térképterület Berlinben vagy Hamburgban van. Ott rendszerint a Bezirksamt (kerületi hivatal) vagy a tartomány valamelyik hatósága illetékes. Ezek a tudnivalók a kerületet még nem határozzák meg.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] A védett területek az alsó fokú természetvédelmi hatóságot (untere Naturschutzbehörde) érintik. Itt ez maga a(z) { $kreis } város.
        [no] A védett területek az alsó fokú természetvédelmi hatóságot (untere Naturschutzbehörde) érintik, rendszerint a(z) { $kreis } járást.
       *[unknown] A védett területek az illetékes alsó fokú természetvédelmi hatóságot (untere Naturschutzbehörde) érintik, rendszerint a Landkreis vagy a kreisfreie Stadt hivatalát.
    }
permits-hint-national-park = A nemzeti parknak saját szabályai vannak. Kérdezze meg korán a nemzeti park igazgatóságát.
permits-hint-nsg-paths = Természetvédelmi területen rendszerint az utakon kell maradni. Utakon kívüli futáshoz kérdezze a természetvédelmi hatóságot.
permits-hint-natura2000 = Egy FFH-terület vagy madárvédelmi terület (Natura 2000) érinti a térképterületet. Rendezvényhez a természetvédelmi hatóság hatásvizsgálatot kérhet.
permits-hint-protected-areas-pl = A térképterületen természetvédelmi védett területek vagy védett objektumok találhatók. Nemzeti park, tájpark vagy természetvédelmi terület esetén rendszerint a park vagy a rezervátum igazgatósága az első kapcsolat; a többi területnél és objektumnál a vajdaság regionális környezetvédelmi igazgatósága (RDOŚ) általában meg tudja mondani, ki az illetékes. Kérdezz időben, ha ott rendezvényt tervezel.
permits-hint-order-office = { $amt ->
        [none] Utcák, terek és más közterületek ügyében rendszerint a(z) { $gemeinde } község Ordnungsamtja (közrendi hivatal) illetékes.
       *[other] Utcák, terek és más közterületek ügyében rendszerint az Ordnungsamt (közrendi hivatal) illetékes: { $amt }, a(z) { $gemeinde } községhez.
    }
permits-hint-park-operator = Egy park vagy kert üzemeltetője a térképterületen az OpenStreetMap szerint: { $operator }. Használat előtt kérdezze meg őket.
permits-hint-school-operator = Az iskola vagy intézmény üzemeltetője az OpenStreetMap szerint: { $operator }. A területe ügyében kérdezze az iskola vezetőjét vagy ezt a szervet.
permits-section-districts = Bezirke (kerületek, { $source })
permits-section-green-spaces = Zöldterületek ({ $source })
permits-section-owners = A tartomány és a Bund földje ({ $source })
permits-section-monuments = Műemlékek ({ $source })
permits-section-protected-objects = Természeti emlékek és más védett objektumok ({ $source })
permits-none-districts = A(z) { $source } szerint a térképterület nincs egyetlen Bezirkben sem.
permits-none-green-spaces = A(z) { $source } szerint nincs közzöldterület a térképterületen.
permits-none-owners = A(z) { $source } szerint nincs a tartomány vagy a Bund tulajdonában lévő telek a térképterületen.
permits-none-monuments = A(z) { $source } szerint nincs műemlék a térképterületen.
permits-none-protected-objects = A(z) { $source } szerint a tárolt védett objektumok egyike sincs a térképterületen.
permits-none-sampled = A lekérdezett pontokon a(z) { $source } itt nem nevez meg semmit.
permits-gaps = A(z) { $source } adatai nem teljesek: { $gaps }.
permits-sampled = A(z) { $source } csak egyes pontokra válaszol. A freemapper a térkép felett egy pontrácsban kérdezett: a részesedések becslések, és ami a pontok között van, hiányozhat.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = { $district } Bezirk
permits-green-space-dedication = rendeltetés: { $dedication }
permits-playgrounds-none = Nincs közjátszótér a térképterületen.
permits-playgrounds-one = Egy közjátszótér van a térképterületen. Külön nincs felsorolva.
permits-playgrounds-many = { $count } közjátszótér van a térképterületen. Egyenként nincsenek felsorolva.
permits-owner-other = Más köztulajdonosok
permits-owners-rest = A terület többi részére az adatok nem neveznek meg tulajdonost.
permits-monuments-total = Teljesen vagy részben a térképterületen lévő műemlékek: { $count }.{ $links ->
        [yes] {" "}A hivatkozások a műemléklista bejegyzéseire vezetnek.
       *[no] {""}
    }
permits-monuments-links-left = További { $count } bejegyzés itt csak meg van számolva, hivatkozás nélkül.
permits-object-kind = { $kind ->
        [natural-monument] Természeti emlék (Naturdenkmal)
        [protected-landscape-part] Védett tájelem (geschützter Landschaftsbestandteil)
        [biotope] Törvény által védett biotóp (geschütztes Biotop)
        [national-park] Nemzeti park (park narodowy)
        [park-buffer-zone] A park védőövezete (otulina)
        [landscape-park] Tájpark (park krajobrazowy)
        [nature-reserve] Természetvédelmi terület (rezerwat przyrody)
        [protected-landscape-area] Védett tájterület (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000 terület élőhelyekre (SOO)
        [natura2000-birds] Natura 2000 terület madarakra (OSO)
        [ecological-site] Ökológiai terület (użytek ekologiczny)
        [landscape-complex] Természeti és táji együttes (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentációs lelőhely (stanowisko dokumentacyjne)
        [natural-monument-pl] Természeti emlék (pomnik przyrody)
       *[special-rules] Különleges szabályokkal bíró terület
    }
permits-object-count = { $count } objektum
permits-object-citations = Jogalapok:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] A(z) { $district } Bezirk utcái, terei és zöldterületei ügyében rendszerint a Bezirksamt { $district } illetékes, ott a Fachamt Management des öffentlichen Raumes (a közterület-kezelési szakhivatal).
       *[berlin] A(z) { $district } Bezirk utcái, terei és zöldterületei ügyében rendszerint a Bezirksamt { $district } Ordnungsamtja (közrendi hivatal) és Straßen- und Grünflächenamtja (út- és zöldterületi hivatal) illetékes.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] A térképterületen lévő műemlékek ügyében a Denkmalschutzamt Hamburg (műemlékvédelmi hivatal) illetékes.
       *[berlin] A(z) { $district } Bezirk műemlékei ügyében a Bezirksamt { $district } alsó fokú műemlékvédelmi hatósága (untere Denkmalschutzbehörde) illetékes.
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Hogy a térképterületen vannak-e védett műemlékek, az itteni adatokból nem lehet biztosan megmondani.
       *[yes] A térképterületen védett műemlékek vannak.
    } Hogy egy futás érinti-e őket, például egy ellenőrzőponttal egy műemléknél vagy egy védett területen átvezető szakasszal, azt az alsó fokú műemlékvédelmi hatóság (untere Denkmalschutzbehörde) dönti el{ $kreisfrei ->
        [yes] . Itt ez maga a(z) { $kreis } város.
        [no] , rendszerint a(z) { $kreis } járás{ $gemeinde ->
            [none] .
           *[other] ; egyes tartományokban a(z) { $gemeinde } község.
        }
       *[unknown] {" "}(rendszerint a Landkreis vagy a kreisfreie Stadt, egyes tartományokban a község).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] A térképen lévő természeti emlékek, védett tájelemek és védett biotópok az alsó fokú természetvédelmi hatóság (untere Naturschutzbehörde) ügyei. Itt ez maga a(z) { $kreis } város.
        [no] A térképen lévő természeti emlékek, védett tájelemek és védett biotópok az alsó fokú természetvédelmi hatóság (untere Naturschutzbehörde) ügyei, rendszerint a(z) { $kreis } járásé.
       *[unknown] A térképen lévő természeti emlékek, védett tájelemek és védett biotópok az alsó fokú természetvédelmi hatóság (untere Naturschutzbehörde) ügyei, rendszerint a Landkreis vagy a kreisfreie Stadt hivataláé.
    }
permits-hint-biotope = Törvény által védett biotópok vannak a térképterületen{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Tilos mindent, ami elpusztíthatja vagy jelentősen károsíthatja őket (§ 30 BNatSchG). Az ellenőrzőpontokat és szakaszokat helyezze rajtuk kívülre, vagy kérdezze az alsó fokú természetvédelmi hatóságot (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] A(z) { $district } Bezirk természeti emlékei és védett tájelemei ügyében rendszerint a Bezirksamt { $district } illetékes.
       *[berlin] A(z) { $district } Bezirk természeti emlékei és védett tájelemei ügyében a Bezirksamt { $district } alsó fokú természetvédelmi hatósága (untere Naturschutzbehörde) illetékes.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] A tartomány nyílt adatai szerint a térképterület { $share } része más köztulajdonosoké.
       *[no] A tartomány nyílt adatai szerint a térképterület { $share } része a(z) "{ $owner }" tulajdonosé.
    }
permits-hint-owners-unknown = { $rest ->
        [yes] A terület többi részének tulajdonosai nyílt adatokból nem deríthetők ki. Jogos érdek esetén (például rendezőként) a földhivataltól (Grundbuchamt) kérhet kivonatot; köztulajdonú terület esetén az illetékes Ordnungsamt vagy a zöldterületi hivatal segít.
       *[no] Ezeknek a területeknek a tulajdonosai nyílt adatokból nem deríthetők ki. Jogos érdek esetén (például rendezőként) a földhivataltól (Grundbuchamt) kérhet kivonatot; köztulajdonú terület esetén az illetékes Ordnungsamt vagy a zöldterületi hivatal segít.
    }
permits-section-forest-units = Erdészeti igazgatás ({ $source })
permits-section-state-forest = Tartományi erdő ({ $source })
permits-none-forest-units = A(z) { $source } szerint a térképterület egyetlen erdészeti hivatal területén sincs.
permits-section-admin-units = Közigazgatási egységek ({ $source })
permits-none-admin-units = A(z) { $source } szerint a térképterület a nyilvántartás egyetlen közigazgatási egységében sem fekszik.
permits-section-register-monuments = A nemzeti nyilvántartás műemlékei ({ $source })
permits-none-register-monuments = A(z) { $source } szerint a térképterület körüli téglalapban nincs nyilvántartott műemlék.
permits-register-monuments-total = Nyilvántartott műemlékek a térképterület körüli téglalapban: { $count }. A nyilvántartás nem ad meg helyzetet, ezért ezek a megjegyzések nem mondhatják meg, hol vannak; ezt a térképen kell ellenőrizni.
permits-register-monuments-left = További { $count } itt csak meg van számolva, név nélkül.
permits-section-register-archaeology = A nemzeti nyilvántartás régészeti emlékei ({ $source })
permits-none-register-archaeology = A(z) { $source } szerint a térképterület körüli téglalapban nincs nyilvántartott régészeti emlék.
permits-register-archaeology-total = Nyilvántartott régészeti emlékek a térképterület körüli téglalapban: { $count }. A nyilvántartás nem ad meg helyzetet, ezért ezek a megjegyzések nem mondhatják meg, hol vannak; ezt a térképen kell ellenőrizni.
permits-register-archaeology-left = További { $count } itt csak meg van számolva, név nélkül.
permits-section-register-history = A történelem emlékei ({ $source })
permits-none-register-history = { $source } szerint a térképterület körüli téglalapban nincs a történelem emléke.
permits-register-history-in-area = A történelem emlékei a térképterületen: { $count }.
permits-register-history-nearby = További történelmi emlékek a térképterület körüli téglalapban, de nem a térképterületen: { $count }.
permits-register-history-left = További { $count } itt csak meg van számolva, név nélkül.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = A(z) { $source } szerint nincs tartományi erdő a térképterületen.
permits-forest-units-reach = Az erdészeti hivatal a saját területén lévő erdőért felel. A területe városokat és mezőket is magában foglalhat.
permits-state-forest-office = illetékes: { $office }
permits-hint-forest-office = { $units ->
        [none] A térképterületen lévő erdőért felelős erdészeti hivatal: { $office } ({ $administration }). Ha erdőben tervez rendezvényt, kérdezzen ott korán.
       *[other] A térképterületen lévő erdőért felelős erdészeti hivatal: { $office } ({ $administration }), { $units }. Ha erdőben tervez rendezvényt, kérdezzen ott korán.
    }
permits-hint-forest-owner = { $office ->
        [known] Az erdőben tartott rendezvényhez az erdőtulajdonosok beleegyezése kell. Az erdészeti hivatal rendszerint meg tudja mondani, ki az erdő tulajdonosa. Több tartományban a szervezett erdei rendezvényt az erdészeti hatóságnak is be kell jelenteni, vagy engedélyeztetni kell vele.
       *[unknown] Az erdőben tartott rendezvényhez az erdőtulajdonosok beleegyezése kell. Több tartományban a szervezett erdei rendezvényt az erdészeti hatóságnak is be kell jelenteni, vagy engedélyeztetni kell vele. Ezek a tudnivalók nem nevezik meg az itt illetékes erdészeti hivatalt; a tartomány erdészeti igazgatása vagy az alsó fokú erdészeti hatóság meg tudja mondani.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Az erdőben tartott rendezvényhez rendszerint az erdő tulajdonosának hozzájárulása kell. A fent megnevezett erdészeti körzet (Nadleśnictwo) általában meg tudja mondani, ki gondozza az erdőt. Az Állami Erdőkhöz nem tartozó erdő, például a magánerdő, nincs benne e megjegyzések adataiban. Az erdőt, amely nem az államé, például a magán- vagy önkormányzati erdőt, az erdőgazdálkodás tekintetében a starosta (starosta) felügyeli (erdőtörvény, 5. cikk 1. bekezdés 2. pont); ott a hivatalnál (starostwo powiatowe) kérdezzen.
       *[unknown] Az erdőben tartott rendezvényhez rendszerint az erdő tulajdonosának hozzájárulása kell. E megjegyzések a térképterülethez nem neveznek meg az Állami Erdők erdészeti körzetet; az adatok csak az Állami Erdőket fedik le, így az erdő másé is lehet, például magántulajdonosé. A tulajdonost külön kell kideríteni. Az erdőt, amely nem az államé, például a magán- vagy önkormányzati erdőt, az erdőgazdálkodás tekintetében a starosta (starosta) felügyeli (erdőtörvény, 5. cikk 1. bekezdés 2. pont); ott a hivatalnál (starostwo powiatowe) kérdezzen.
    }
permits-hint-monument-register-pl = A térképterület körüli téglalapban a nemzeti nyilvántartás műemlékei találhatók. Helyzetük nincs az adatokban, ezért ellenőrizze a térképen, hol vannak. A műemlékvédelmi törvény (ustawa o ochronie zabytków) 36. cikk 1. bekezdése szerint a nyilvántartott műemléken végzett konzerválási, restaurálási és építési munkához a vajdasági műemlékvédelmi felügyelő (Wojewódzki Konserwator Zabytków) engedélye kell. 2026. június 3. óta a műemlék környezetében végzett építési munkához, rendeltetésének megváltoztatásához, a rajta elhelyezett berendezésekhez vagy táblákhoz és minden más olyan cselekvéshez, amely megváltoztathatja az anyagát vagy megjelenését, csak a felügyelőnek tett bejelentés kell (36aa. cikk); a felügyelő 60 napon belül kifogást emelhet, és ekkor engedélyt kérhet. A törvény a futást vagy az ellenőrzőpontot mint olyat nem említi; kérdezze meg a felügyelő hivatalát, ha egy rendezvény műemléket vagy annak környezetét érinti.
permits-hint-archaeology-register-pl = A nemzeti nyilvántartás régészeti emlékei a térképterület körüli téglalapban találhatók. Helyzetük nincs az adatokban, ezért a térképen ellenőrizze, hol vannak. A műemlékvédelmi törvény (ustawa o ochronie zabytków) 36. cikk 1. bekezdése szerint a régészeti kutatáshoz (pkt 5), az elrejtett vagy elhagyott ingó emlékek elektronikus vagy műszaki eszközökkel, például fémdetektorral való kereséséhez (pkt 12), valamint a nyilvántartott műemléken végzett konzerválási, restaurálási vagy építési munkához (pkt 1) a vajdasági műemlékvédelmi felügyelő (Wojewódzki Konserwator Zabytków) engedélye kell. A keresési engedély 2027. január 1-jén megszűnik: a 36b. cikk ekkortól megengedi, hogy nagykorú személy az ingatlan tulajdonosának és birtokosának hozzájárulásával keressen, miután a kulturális minisztérium alkalmazásán keresztül bejelentette a keresést a keresési nyilvántartásba, a 36c. cikk pedig az ilyen keresést többek között a nyilvántartott műemlékeken és a tőlük számított 10 m-en belül tiltja. A törvény nem nevesít sétát, futást vagy ellenőrzőpontot mint olyat; kérdezze meg a felügyelő hivatalát, ha egy rendezvény vagy terepi munka nyilvántartott lelőhelyet érint.
permits-hint-history-register-pl = A történelem egy emléke (Pomnik historii) a térképterületen vagy annak közelében található; a történelmi emlékekről szóló rész megmondja, melyik. A Lengyel Köztársaság elnöke rendelettel Pomnik historii-vé nyilváníthatja a nyilvántartásba vett ingatlan emléket vagy a különleges értékű kultúrparkot (a műemlékvédelmi törvény, ustawa o ochronie zabytków, 15. cikk 1. bekezdése). Tehát már a nyilvántartás vagy a kultúrpark védelme alatt áll. A nyilvántartásba vett emléken végzett munkához a vajdasági műemlékvédelmi felügyelő (Wojewódzki Konserwator Zabytków) engedélye kell (36. cikk 1. bekezdés); kultúrparkban a tilalmakat és korlátozásokat a gmina tanácsának határozata állapítja meg (16. cikk 2. bekezdés, 17. cikk 1. bekezdés). A törvény nem nevez meg sétát, futást vagy ellenőrzőpontot mint olyat; kérdezze meg a felügyelő hivatalát, ha egy rendezvény vagy terepmunka érinti az emléket.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] A térképterületen természeti emlék (pomnik przyrody), ökológiai terület (użytek ekologiczny), természeti és táji együttes (zespół przyrodniczo-krajobrazowy) vagy dokumentációs lelőhely (stanowisko dokumentacyjne) található. Ezeket a gmina tanácsa határozattal hozza létre; a határozat megnevezi, ki felügyeli az objektumot, és mely tilalmak érvényesek (a természetvédelmi törvény 44. cikke). A törvény maga itt nem említ engedélyt rendezvényhez. Kérje el a határozatot és a felügyelő nevét annak a gminának a hivatalától, amelynek területén az objektum fekszik.
       *[other] A térképterületen természeti emlék (pomnik przyrody), ökológiai terület (użytek ekologiczny), természeti és táji együttes (zespół przyrodniczo-krajobrazowy) vagy dokumentációs lelőhely (stanowisko dokumentacyjne) található. Ezeket a gmina tanácsa határozattal hozza létre; a határozat megnevezi, ki felügyeli az objektumot, és mely tilalmak érvényesek (a természetvédelmi törvény 44. cikke). A törvény maga itt nem említ engedélyt rendezvényhez. Kérje el a határozatot és a felügyelő nevét { $count ->
        [one] annak a gminának ({ $gmina }) a hivatalától, amelynek területén az objektum fekszik
       *[other] ezen gminák ({ $gmina }) hivatalaitól, mindegyiktől a saját területén fekvő objektumok esetében
    }.
    }
permits-hint-gmina-roads-pl = Az a rendezvény, amely akadályozza a forgalmat vagy különleges módon használ egy utcát, a közúti közlekedési törvény 65. cikke szerinti engedélyt igényel, amelyet legalább 30 nappal előre kell kérni. Az engedélyt az úton a forgalmat irányító szerv adja ki: a powiat- és gminautak esetén a starosta, powiat-jogú városban a polgármester, egyéb utak esetén a vajdaság marsallja vagy a GDDKiA (10. és 65b. cikk). A gminaút kezelője { $count ->
        [one] a gmina ({ $gmina }) wójtja
       *[other] mindegyik gmina ({ $gmina }) wójtja
    }, burmistrza vagy prezydent miastája; a kezelő dönt az úttest igénybevételéről (a közutakról szóló törvény 19. és 40. cikke). Ezek a megjegyzések nem mutatják az utak kategóriáját.
permits-hint-gmina-land-pl = Ha a térképterület földje a gminához tartozik, például városi park, tér, iskolaterület vagy erdő, azt a wójt, burmistrz vagy prezydent miasta kezeli, aki a gminát képviseli (a gmina önkormányzatáról szóló törvény 30. és 31. cikke). Használat előtt kérje { $count ->
        [one] a gmina ({ $gmina }) hivatalának hozzájárulását
       *[other] a gminák ({ $gmina }) hivatalainak hozzájárulását
    }. Ezek a megjegyzések nem mutatják, kié a föld.
permits-section-water-protection = Vízvédelmi területek ({ $source })
permits-section-forest-functions = Erdőfunkciók és védett erdő ({ $source })
permits-none-water-protection = A(z) { $source } szerint nincs vízvédelmi terület a térképterületen.
permits-none-forest-functions = A(z) { $source } szerint nincs a térképterületen sem pihenőerdő, sem védett erdő, sem természetes erdő, sem lezárt erdőterület.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Pihenőerdő (Erholungswald)
        [protected] Védett erdő (Bannwald vagy Schutzwald)
        [natural] Természetes erdő vagy szigorú rezervátum (Naturwald, Totalreservat)
       *[closed] Lezárt erdőterület
    }
permits-hint-water-protection = { $inner ->
        [yes] A(z) "{ $name }" vízvédelmi terület { $zone } része a térképterületen van. Az I. és II. övezetben a rendezvények rendszerint kizártak. Kérdezze az alsó fokú vízügyi hatóságot (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] A(z) "{ $name }" vízvédelmi terület { $zone } része a térképterületen van. Ott a rendezvények rendszerint lehetségesek; az alsó fokú vízügyi hatóság (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) meg tudja mondani, mit enged a rendelet.
       *[unknown] A(z) "{ $name }" vízvédelmi terület a térképterületen van. Az adatok nem mondják meg, melyik övezet. Az I. és II. övezetben a rendezvények rendszerint kizártak; kérdezze az alsó fokú vízügyi hatóságot (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Pihenőerdő (Erholungswald) van a térképterületen{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Az ottani rendezvényekhez kérdezze az erdészeti hatóságot.
        [protected] Védett erdő (Bannwald vagy Schutzwald) van a térképterületen{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Rendezvény tervezése előtt kérdezze az erdészeti hatóságot.
        [natural] Természetes erdő vagy szigorú rezervátum van a térképterületen{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Az utakról való lelépés ott gyakran tilos; ne helyezzen el benne ellenőrzőpontot az erdészeti hatóság megkérdezése nélkül.
       *[closed] Belépni tilos erdőterület van a térképterületen{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Hagyja ki a pályából.
    }
permits-section-roads = Osztályba sorolt utak az OpenStreetMap szerint
permits-none-roads = Az OpenStreetMap szerint nem halad át a térképterületen sem Autobahn, sem Bundes-, Landes-, Staats- vagy Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (autópálya)
        [bundesstrasse] Bundesstraße (szövetségi főút)
        [landesstrasse] Landesstraße vagy Staatsstraße (tartományi út)
       *[kreisstrasse] Kreisstraße (járási út)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] A(z) { $refs } Autobahn áthalad a térképterületen. A pálya csak hidakon vagy aluljárókon keresztezheti.
        [bundesstrasse] A(z) { $refs } Bundesstraße áthalad a térképterületen. A lezárást vagy a pályabírókkal biztosított átkelést a közlekedési hatóság (Straßenverkehrsbehörde) rendeli el; az utat rendszerint a Bund tartja fenn a tartomány útkezelésén keresztül, nagyobb városokban gyakran a város.
        [landesstrasse] A(z) { $refs } Landes- vagy Staatsstraße áthalad a térképterületen. A lezárást vagy a pályabírókkal biztosított átkelést a közlekedési hatóság (Straßenverkehrsbehörde) rendeli el; az utat rendszerint a tartomány tartja fenn, nagyobb városokban gyakran a város.
       *[kreisstrasse] A(z) { $refs } Kreisstraße áthalad a térképterületen. A lezárást vagy a pályabírókkal biztosított átkelést a közlekedési hatóság (Straßenverkehrsbehörde) rendeli el; az utat rendszerint a Kreis tartja fenn, nagyobb városokban gyakran a város.
    }
permits-hint-state-forest = { $office ->
        [none] A tartomány nyílt adatai szerint a térképterület { $share } része a(z) "{ $owner }" tulajdonos tartományi erdeje.
       *[other] A tartomány nyílt adatai szerint a térképterület { $share } része a(z) "{ $owner }" tulajdonos tartományi erdeje. Illetékes: { $office }.
    }
