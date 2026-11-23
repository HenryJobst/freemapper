# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } nėra kalba, kuria freemapper turi tekstų; yra: { $available }

## init

cli-part-needs-osm = --part parenka OSM ploto dalį ir reikalauja --osm; su --place pirmiausia susiraskite id ir naudokite --osm <id> --part <n>
cli-reading-file = skaitoma { $path }
cli-no-area-for-place = vietai "{ $query }" ploto nerasta. Jei vieta yra plotas iš kelių atskirų dalių, pasirinkite vieną su --osm <id> --part <n> arba naudokite --geojson
cli-using-place = Naudojama { $name } ({ $id })
cli-created-project = Sukurtas projektas { $path }

## Cache

cli-no-cache-dir = šioje sistemoje nėra talpyklos katalogo; nurodykite --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Atsisiųsta { $count } plytelė
        [few] Atsisiųstos { $count } plytelės
       *[other] Atsisiųsta { $count } plytelių
    }
cli-wrote = Įrašyta { $path }
cli-kept-map = Palikta { $path } (joje gali būti jūsų darbo)
cli-map-lacks-templates = Žemėlapis dar nenurodo vaizdų šablonų. Pridėkite juos OOM (Templates > Open template…) arba pakeiskite žemėlapį su --force, kuris senąjį palieka kaip .omap.bak.
cli-map-lacks-buildings = Žemėlapyje dar nėra ALKIS pastatų. Norėdami juos pridėti, pakeiskite žemėlapį su --force, kuris senąjį palieka kaip .omap.bak.
cli-map-lacks-canopies = Žemėlapyje trūksta ALKIS stogelių ir automobilių pastogių (522), kurie dabar yra žemėlapio objektai. Norėdami juos pridėti, paleiskite su --force, kuris senąjį žemėlapį palieka kaip .omap.bak.
cli-map-light = Pagrindinis žemėlapis: sujungta { $joined } linijų, { $left } paslėptų arba per mažų palikta kandidatų sluoksniams (niekas nepajudinta)
cli-map-light-kept = Kandidatų sluoksniuose dabar yra ir { $left } objektų ar atkarpų, kuriuos ši versija nuima nuo pagrindinio žemėlapio (pažymėta freemapper:generalized). Palikti žemėlapyje jie vis dar yra, todėl rodomi dukart. Pakeiskite žemėlapį su --force, kad jį sutvarkytumėte (senasis lieka kaip .omap.bak).
cli-map-old-candidates = Žemėlapis vis dar nurodo templates/candidates.omap; šis paleidimas jį užpildė visais kandidatais. Kandidatai dabar taip pat pateikiami septyniais sluoksniais, po vieną failą (templates/candidates-*.omap). Žemėlapis juos išvardija, kai jį pakeičiate su --force (senasis lieka kaip .omap.bak), arba pridėkite juos patys OOM (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF be šaltinių nuorodų: ODbL ir dl-de/by-2-0 reikalauja nurodyti šaltinius, kai žemėlapį perduodate toliau; žr. ATTRIBUTION.txt
cli-wrote-pdf = Įrašyta { $path } (žemėlapis be kandidatų)
cli-wrote-pdf-candidates = Įrašyta { $path } (tikrinimo spaudinys: žemėlapis su visais kandidatais, tiksliai pagal padėtį; patikrinkite juos vietovėje)
cli-wrote-training = Įrašyta { $map } ir { $pdf } (treniruočių žemėlapis: sujungta { $joined }, pastumta { $shifted }, praleista arba išpjauta { $left })
cli-training-free-ends = Laisvai paliktų linijų galų: { $ways } ten, kur kelias susitiko su kitu keliu, { $others } ties kitomis linijomis
cli-training-apart-left = treniruočių žemėlapyje vis dar yra { $length } mm (popieriuje), kur objektų tarpai per maži; patikrinkite jį OOM
cli-kept-training = Paliktas treniruočių žemėlapis { $path }, kad paleidimas neperrašytų atspausdinto ar kitiems perduoto; ištrinkite jį arba nurodykite --force, kad jį atnaujintumėte

## OCAD

cli-ocad-kept = Palikta { $path } (joje gali būti jūsų darbo; --force ją pakeičia, o senąją palieka kaip .ocd.bak)
cli-ocad-old-candidates = OCAD žemėlapis vis dar nurodo { $path } kaip fono žemėlapį; šis paleidimas jį užpildė visais kandidatais. Kandidatai dabar taip pat pateikiami septyniais sluoksniais, po vieną failą (-candidates-*.ocd). OCAD žemėlapis juos išvardija, kai jį pakeičiate su --force (senasis lieka kaip .ocd.bak), arba pridėkite juos patys OCAD kaip fono žemėlapius.
cli-ocad-wrote = { $count ->
        [one] Įrašyta { $path } ({ $count } objektas)
        [few] Įrašyta { $path } ({ $count } objektai)
       *[other] Įrašyta { $path } ({ $count } objektų)
    }
cli-ocad-left-out = OCAD žemėlapyje nėra: { $skipped } objektų (OCAD simbolio nėra, ploto nėra arba už OCAD piešimo ploto ±2 m) ir { $dropped } išsigimusių linijų dalių ar skylių.
cli-ocad-load-symbols = OCAD pasirinkite Symbol > Load Colors and Symbols From… ir nurodykite oficialų simbolių rinkinį{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF mokyklų rinkinį 2019
       *[other] {" "}ISSprOM 2019
    }; jis pakeičia freemapper vietos rezervavimo simbolius pagal numerį.

## select

cli-select-running = Ploto pasirinkimas { $path }: paspauskite „Baigta“ puslapyje arba Ctrl+C, kad sustabdytumėte.
cli-select-no-browser = Naršyklės atidaryti nepavyko ({ $error }); atidarykite aukščiau nurodytą adresą.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; ploto nėra, negali būti žemėlapio plotas): { $place }
cli-find-hint = Projektą sukurkite komanda: freemapper init <dir> --osm { $id }
cli-find-no-area = Nė viena iš šių vietų neturi ploto, kuris galėtų būti žemėlapio plotas. Nubraižykite plotą komanda: freemapper select <dir>
cli-find-nothing = vietos "{ $query }" nerasta; pabandykite kitą pavadinimą arba nubraižykite plotą komanda: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = įspėjimas: adapteris aplanke { $folder } praleistas: { $finding }
cli-adapter-note = įspėjimas: adapteris aplanke { $folder } įkeliamas, bet: { $finding }
cli-adapter-from-project = įspėjimas: naudojamas projekto aplanko adapteris { $adapter }; jis gali kreiptis į šiuos hostus: { $hosts }
cli-adapter-overlaps = įspėjimas: adapteris { $adapter } taip pat apima { $others } teritoriją; jo objektai patenka į ten esančius žemėlapius
cli-adapter-dir-relative = įspėjimas: FREEMAPPER_ADAPTER_DIR nėra absoliutus kelias ({ $path }); jis ignoruojamas
cli-adapters-no-project = įspėjimas: { $path } nėra aplankas; projekto adapterių neskaityta
cli-adapters-overlaps = dalijasi teritorija su: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  redakcija { $revision }  patikrinta { $checked }  hostai: { $hosts }
cli-adapters-source-built-in = įtaisytas
cli-adapters-source-user = iš jūsų aplanko
cli-adapters-source-project = iš projekto
cli-adapters-never-checked = niekada
cli-adapters-user-dir = Jūsų adapteriai skaitomi iš { $path }
cli-adapters-no-user-dir = Šioje sistemoje nėra aplanko jūsų adapteriams; FREEMAPPER_ADAPTER_DIR nurodo vieną

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper dabar gali kreiptis ir į { $host } dėl { $name }. Įrašas yra faile { $file }; `freemapper adapters disallow-host { $adapter } { $host }` jį atšaukia. Veikiantis ploto puslapis arba DI asistentas įrašą perskaito, kai paleidžiamas iš naujo
cli-allow-host-already = { $host } jau leidžiamas dėl { $name }
cli-allow-host-declared = { $name } adapteris pats nurodo { $host }; leisti nieko nereikia
cli-allow-host-refused = { $host } leisti negalima: { $reason }
cli-allow-host-unknown-adapter = įtaisyto adapterio { $adapter } nėra; įtaisyti adapteriai: { $adapters }
cli-allow-host-folder-adapter = { $adapter } yra jūsų adapterių aplanko adapteris; vietoj to pridėkite hostą į jo manifesto `[net] hosts`
cli-allow-host-no-config = šioje sistemoje nėra konfigūracijos aplanko leidžiamiems hostams laikyti
cli-allow-host-write = { $file } nepavyko įrašyti: { $detail }
cli-allow-host-unreadable = { $file } neperskaitomas ({ $detail }); pirmiausia pataisykite arba pašalinkite failą; niekas nepakeista
cli-disallow-host-removed = freemapper daugiau nesikreipia į { $host } dėl { $name }
cli-disallow-host-absent = { $host } dėl { $name } nebuvo leidžiamas; atšaukti nėra ko
cli-allowed-hosts-unreadable = įspėjimas: { $file } nepavyko perskaityti ({ $detail }); papildomų hostų neleidžiama
cli-allowed-hosts-ignored = įspėjimas: { $file }: { $adapter } įrašas { $host } ignoruojamas: { $reason }
cli-allowed-hosts-no-adapter = freemapper tokio pavadinimo įtaisyto adapterio neturi
cli-allowed-host-redundant = įspėjimas: { $host } dabar yra vienas iš hostų, kuriuos nurodo { $name } adapteris; jūsų įrašas faile { $file } pašalintas
cli-adapters-allowed = jūsų leidžiami: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapteris { $adapter } ({ $name }), redakcija { $revision }, hostai: { $hosts }
cli-check-error = klaida: { $finding }
cli-check-warning = įspėjimas: { $finding }
cli-check-passed = { $folder } patikrą išlaiko
cli-check-passed-warnings = { $folder } patikrą išlaiko, su { $count ->
        [one] { $count } įspėjimu
        [few] { $count } įspėjimais
       *[other] { $count } įspėjimų
    }
cli-check-failed = { $folder } patikros neišlaiko
cli-check-no-folder = aplanko { $folder } nėra
cli-check-live-ok = { $capability } gerai: { $detail }
cli-check-live-failed = { $capability } NEPAVYKO: { $detail }
cli-check-live-not-asked = { $capability } neklausta: { $detail }
cli-check-live-box-outside = langelis nepatenka į adapterio { $adapter } ribas; gyva patikra klausia paslaugų apie vietą, kurią adapteris apima
cli-check-live-cannot = gyvos patikros paleisti negalima: { $detail }
cli-check-detail-not-declared = adapteris nieko nedeklaruoja
cli-check-detail-stopped = ankstesnė dalis nepavyko
cli-check-detail-see-warnings = žr. įspėjimus
cli-check-detail-outside-dataset = langelis yra už duomenų rinkinio ribų
cli-check-detail-objects = { $count ->
        [one] { $count } objektas
        [few] { $count } objektai
       *[other] { $count } objektų
    }
cli-check-detail-layers = { $layers } sluoksnių, { $count } objektų
cli-check-detail-trees = { $count ->
        [one] { $count } medis
        [few] { $count } medžiai
       *[other] { $count } medžių
    }
cli-check-detail-extract = { $label }, duomenys iš { $date }
cli-check-raster-orthophoto = ortofoto
cli-check-raster-terrain = reljefo modelis
cli-check-raster-surface = paviršiaus modelis
cli-check-raster-tiles = { $product }: { $count ->
        [one] { $count } plytelė
        [few] { $count } plytelės
       *[other] { $count } plytelių
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } ir { $last }

## Folder lock

cli-lock-held = kitas freemapper paleidimas naudoja šį aplanką (nuo { $time }, paleista: { $who }). Palaukite, kol jis baigs, arba pirmiausia jį sustabdykite.
cli-lock-who-cli = terminalas
cli-lock-who-mcp = DI agentas (MCP)
cli-lock-who-page = pasirinkimo puslapis
cli-lock-who-unknown = kita programa
cli-lock-failed = nepavyko užrakinti projekto aplanko šiam paleidimui: { $error }
cli-lock-stuck = kelią užstoja kažkas kita nei užrakto failas, ir jo pašalinti nepavyko

## Run routes of the selection page

cli-run-bad-request = Paleidimo parinktys netinkamos: { $error }
cli-run-no-project = Šiame aplanke dar nėra išsaugoto projekto. Pirmiausia išsaugokite plotą.
cli-run-already-running = Šiam projektui jau vyksta paleidimas ({ $job }). Palaukite, kol jis baigsis, arba pirmiausia jį atšaukite.
cli-run-disk-stops = Atsisiuntimui nepakanka laisvos disko vietos: { $messages }
cli-run-busy-elsewhere = Kitas šios programos projektas dabar vykdo konvejerį. Projektai dalijasi atsisiuntimų talpykla, todėl vienu metu vyksta tik vienas paleidimas. Pabandykite vėliau, kai jis baigsis.
cli-run-nothing-to-cancel = Joks paleidimas nevyksta, todėl atšaukti nėra ko.
cli-run-unknown-job = Šiam projektui paleidimo { $job } nėra.
cli-status-too-fast = Būsenos užklausos siunčiamos per dažnai. Klauskite ne dažniau kaip penkis kartus per sekundę.
cli-open-folder-failed = Aplanko atidaryti nepavyko ({ $error }).
cli-open-folder-no-opener = Šiame kompiuteryje nėra programos aplankams atidaryti ({ $program }). Atidarykite aplanką rankiniu būdu.
cli-open-folder-missing = Projekto aplanko dar nėra. Pirmiausia išsaugokite plotą.
cli-open-folder-refused = Aplanko vardas nesaugus perduoti failų tvarkyklei.
cli-save-running = Šiam projektui vyksta paleidimas. Išsaugokite dar kartą, kai jis baigsis.
cli-save-unknown-kind = Nežinoma žemėlapio rūšis "{ $kind }". Naudokite schoolyard, sprint arba forest.
cli-save-scale-too-large = Mastelis { $scale } per didelis.
cli-save-bad-content = Turinio nustatymai netinkami. Nieko neišsaugota.
cli-map-image-none = Pirmiausia sukurkite žemėlapį.
cli-map-image-gone = Ši peržiūra nebepasiekiama.
cli-map-image-running = Šiam projektui vyksta paleidimas. Peržiūrą galima apskaičiuoti, kai jis baigsis.
cli-map-image-stale = Išsaugotas projektas nebeatitinka sugeneruoto žemėlapio, pavyzdžiui, nes pasikeitė plotas, žemėlapio rūšis ar mastelis. Pirmiausia paleiskite paleidimą, kuris pakeičia žemėlapį (puslapyje „Pakeisti esamą žemėlapį“ arba freemapper run --force).
cli-map-image-superseded = Tuo metu paprašyta naujesnės peržiūros, todėl ši neapskaičiuota.
cli-map-image-bad-content = Turinio nustatymai netinkami.
cli-map-image-bad-request = Peržiūros užklausa netinkama. Siųskite JSON objektą, jei yra, su turinio nustatymais.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – ploto pasirinkimas
page-heading = Pasirinkite žemėlapio plotą
page-update-available = Yra freemapper { $version }. Atnaujinkite su
page-update-notes = Laidos pastabos
page-folder = Projekto aplankas
page-pick = Ieškoti vietos
page-query-placeholder = pvz. Clara-Zetkin-Schule Strausberg
page-search-button = Ieškoti
page-searching = Ieškoma …
page-no-results = Nieko nerasta.
page-point-only = tik taškas, ne plotas
page-draw = Nubraižykite patys
page-rectangle = Stačiakampis
page-polygon = Daugiakampis
page-finish-polygon = Uždaryti daugiakampį
page-clear = Išvalyti
page-hint-rectangle = Spustelėkite du priešingus žemėlapio kampus.
page-hint-polygon = Spustelėkite kampus paeiliui. Kad uždarytumėte, spustelėkite pirmąjį kampą, spustelėkite dukart arba pasirinkite „Uždaryti daugiakampį“. Esc atšaukia, Backspace pašalina paskutinį kampą.
page-hint-idle = Pasirinkite vietą iš paieškos arba nubraižykite plotą patys.
page-summary = Plotas
page-source-osm = Iš OpenStreetMap: { $name }
page-source-drawn = Nubraižyta ranka
page-size = Dydis su buferiu: { $width } × { $height } m
page-tiles-one = Plytelės ({ $n }): { $names }
page-tiles-other = Plytelės ({ $n }): { $names }
page-download = Atsisiuntimas: apie { $mb } MB
page-download-none = Atsisiuntimas: viskas jau talpykloje
page-download-details = Kas atsisiunčiama
page-cached = talpykloje
page-disk = Disko vieta: { $volumes }
page-disk-volume = laisva { $free }, reikia apie { $needed }
page-cache-volume = talpykla
page-project-volume = projekto aplankas
page-no-room = netelpa diske
page-no-room-skip = Ne viskas telpa diske (freemapper palieka { $reserve } laisvos). Atsisiuntimas praleidžia pažymėtas dalis. Jei jų reikia, pirmiausia atlaisvinkite vietos:
page-no-room-stop = Nepakanka disko vietos (freemapper palieka { $reserve } laisvos). Be pažymėtų dalių žemėlapio nėra, todėl atsisiuntimas sustoja dar nieko neatsisiuntęs. Pirmiausia atlaisvinkite vietos:
page-disk-unknown = freemapper negali nuskaityti laisvos disko vietos ({ $error }). Atsisiunčia be patikros.
page-too-large = Plotas per didelis: ne daugiau kaip { $max } m vienoje pusėje (su buferiu). Pasirinkite mažesnį.
page-outside = Plotas yra už palaikomų federalinių žemių ribų ({ $states }); kitiems regionams freemapper duomenų šaltinių dar neturi.
page-outside-unknown = Plotas yra už regionų, kuriuos freemapper palaiko, ribų; ten jis duomenų šaltinių dar neturi.
page-adapters-failed = Aerofotonuotraukų sąrašo gauti nepavyksta ({ $detail }). Žemėlapyje rodomas tik OpenStreetMap; viskas kita veikia.
page-settings = Žemėlapis
page-kind = Žemėlapio rūšis
page-schoolyard = Mokyklos kiemo žemėlapis (1:1000)
page-sprint = Sprinto žemėlapis (1:4000)
page-forest = Miško žemėlapis (1:15000)
page-scale = Mastelis 1:
page-map-title = Pavadinimas
page-contour-interval = Horizontalių intervalas
page-contour-auto = automatinis
page-settings-fixed = Projektas jau yra; žemėlapio rūšis, mastelis ir pavadinimas yra faile freemapper.toml. Horizontalių intervalą dar galima pakeisti.
page-save = Išsaugoti
page-saving = Saugoma …
page-saved = Išsaugota faile freemapper.toml.
page-zone-changed = Išsaugota. Plotas dabar yra kitoje UTM zonoje; kitą kartą paleidus freemapper jo duomenis atsisiunčia iš naujo.
page-provider-changed = Išsaugota. Naujo ploto aukščių ir aerofotonuotraukų duomenis teikia kitas tiekėjas; kitą kartą paleidus freemapper jo duomenis atsisiunčia iš naujo.
page-save-stale = freemapper.toml buvo pakeistas kitame lange arba kitos programos, kai šis puslapis jį perskaitė. Nieko neišsaugota; puslapis dabar žino išsaugotą būseną. Išsaugokite dar kartą, kad ją pakeistumėte čia rodomu plotu.
page-has-map = Esamas žemėlapis (.omap) lieka nepakeistas; freemapper sukuria naujus šablonus ir kandidatus naujam plotui. Parinktis „Pakeisti esamą žemėlapį“ žemiau (arba „freemapper run --force“) sukuria ir žemėlapį iš naujo, o senąjį palieka kaip .omap.bak.
page-done = Baigta
page-source = Pirminis kodas (AGPL-3.0-or-later)
page-closed = Baigta. Galite uždaryti šį langą.
page-next = Toliau: freemapper run "{ $dir }"
page-made-files = Sukurta: { $files }. Aplankas: { $dir }
page-back-to-agent = Pasakykite savo DI asistentui, kad plotas išsaugotas.
page-nothing-saved = Nieko neišsaugota.
page-failed = Klaida: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Aerofotonuotrauka { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Kas patenka į žemėlapį
page-content-help = Profilis nusprendžia, kas patenka tiesiai į žemėlapį, o kas lieka kandidatu patikrinti. Grupės pasirinkimas viršija profilį. Jis išsaugomas kartu su plotu.
page-content-profile-legend = Profilis
page-content-profile-safe = Saugus
page-content-profile-safe-hint = Kiekvienas objektas lieka ten, kur jį padeda sudarymas: pastatai ir kadastro plotai patenka į žemėlapį; sienos, tvoros, medžiai ir daug kas iš OpenStreetMap (tvenkiniai, keliai be žinomos dangos) lieka kandidatais.
page-content-profile-balanced = Subalansuotas
page-content-profile-balanced-hint = Kadastro, registrų ir gatvių apžvalgos objektai patenka į žemėlapį, kartu su horizontalėmis ir draudžiamais plotais prie geležinkelio bėgių. Sienos, tvoros, gyvatvorės, šlaitai, laiptai, vanduo, medžiai, paminklai, praėjimai, keliai ir geležinkeliai taip pat imami iš OpenStreetMap. Gatvių įranga, kiti draudžiami plotai ir reljefo modelio formos lieka kandidatais.
page-content-profile-full = Pilnas
page-content-profile-full-hint = Viskas patenka į žemėlapį, kaip tikrinimo spaudinyje; lauke lieka tik paslėpti kandidatai.
page-content-groups-legend = Pagal grupę
page-content-groups-help = „Kaip profilyje“ seka profilį. Ką tai reiškia grupei, priklauso nuo to, iš kur kiekvienas objektas kilęs.
page-content-place-profile = Kaip profilyje
page-content-place-map = Žemėlapis
page-content-place-candidates = Kandidatas
page-content-place-off = Išjungta
page-content-group-walls = Sienos
page-content-group-fences = Tvoros
page-content-group-hedges = Gyvatvorės
page-content-group-slopes = Šlaitai ir skardžiai
page-content-group-stairs = Laiptai
page-content-group-trees = Medžiai ir medžių grupės
page-content-group-water-features = Vanduo, fontanai ir šaltiniai
page-content-group-monuments = Bokštai, paminklai ir elektros linijos
page-content-group-furniture = Gatvių įranga ir žibintai
page-content-group-passages = Praėjimai
page-content-group-contours = Horizontalės
page-content-group-landforms = Kalvelės, įdubos, šlaitai ir skardžiai iš reljefo modelio
page-content-group-vegetation = Augmenija iš lazerinių taškų debesų (Brandenburgas, Berlynas, Šiaurės Reinas-Vestfalija, Reinlandas-Pfalcas, Zaarlandas ir Bavarija; išjungta, kol pasirinksite; atsisiunčia apie 40–600 MB vienam km²)
page-content-group-out-of-bounds = Draudžiami plotai
page-content-group-paths = Keliai ir geležinkeliai
page-content-group-streets = Gatvės ir aikštės
page-run-heading = Sukurti žemėlapį
page-preview-heading = Žemėlapio peržiūra
page-preview-stored = Kaip sugeneruota paskutinį kartą.
page-preview-live = Su šiame puslapyje esančiais nustatymais, dar neišsaugota.
page-preview-live-saved = Su išsaugotais nustatymais, dar nesugeneruota paleidimu.
page-preview-computing = Skaičiuojama …
page-preview-none = Žemėlapis dar nesugeneruotas.
page-preview-later = Šis žemėlapis dar neturi peržiūros. Ji atsiras per kitą paleidimą, kuris įrašo žemėlapį.
page-preview-stale = Nuo žemėlapio sugeneravimo pasikeitė plotas, žemėlapio rūšis ar mastelis. Vaizdas vis dar rodo paskutinį kartą sugeneruotą žemėlapį; naujai peržiūrai reikia paleidimo su „Pakeisti esamą žemėlapį“.
page-preview-area-unsaved = Šiame puslapyje esantis plotas dar neišsaugotas. Vaizdas rodo paskutinį kartą sugeneruotą žemėlapį; naujam plotui reikia paleidimo su „Pakeisti esamą žemėlapį“.
page-preview-run = Kaip sugeneruota paskutinį kartą. Puslapis atnaujina vaizdą, kai paleidimas baigiasi.
page-preview-open = Atidaryti pilno dydžio
page-preview-alt = Žemėlapio objektų peržiūra
page-opt-ocad = Įrašyti OCAD pradinį projektą
page-help-ocad = Visiems, kurie tęsia darbą OCAD.
page-opt-pdf = Spausdinamas PDF
page-help-pdf = Tik žemėlapio objektai, be kandidatų.
page-opt-pdf-candidates = Tikrinimo spaudinys su visais kandidatais
page-opt-training = Treniruočių žemėlapis
page-help-training = Žemėlapis ir visi kandidatai, sutvarkyti taip, kad juos būtų galima spausdinti tokius, kokie yra; su OCAD taip pat kaip OCAD failas.
page-views-heading = Treniruočių žemėlapio vaizdai
page-view-no-paths = Be takų (Ultimate žemėlapis)
page-view-brown-only = Tik ruda (horizontalių orientavimasis)
page-view-no-north-lines = Be šiaurės linijų
page-view-custom = Kitas vaizdas
page-help-views = Kiekvienas pažymėtas vaizdas yra dar vienas failas šalia treniruočių žemėlapio, `<name>-training-<view>.omap` ir jo PDF, su OCAD taip pat OCAD failas. Kitas vaizdas yra `no:505,506` (praleisti simbolius) arba `only:101,102` (palikti tik šiuos); `+` sujungia vaizdus. Pavadinimas, mastelis ir šaltiniai lieka visada.
page-help-views-forest = Be takų ir tik ruda skirta tik miško žemėlapiams.
page-opt-scale-bar = Mastelio liniuotė PDF failuose
page-help-scale-bar = Vokietijos žemėlapiai paprastai jos neturi.
page-opt-sources = Šaltinių eilutės PDF failuose
page-sources-auto = automatinės
page-sources-full = pilnos
page-sources-credit-only = tik freemapper
page-warn-credit-only = Tada PDF nurodo tik freemapper. Duomenų licencijos (ODbL, dl-de/by-2-0) reikalauja nurodyti šaltinį, kai perduodate žemėlapį toliau; tada šaltinius nurodote patys. ATTRIBUTION.txt išvardija juos visus.
page-need-pdf = Mastelio liniuotė ir šaltinių eilutės taikomos tik kuriant PDF.
page-opt-offline = Tik talpykla, be tinklo
page-opt-force = Pakeisti esamus žemėlapius
page-help-force = Žemėlapį ir, jei pažymėtas „Treniruočių žemėlapis“, treniruočių žemėlapį. Seni failai lieka kaip .omap.bak.
page-opt-permits = Leidimų pastabos
page-help-permits = Administraciniai vienetai, saugomos teritorijos ir ko klausti, faile permits.md. Užuomina, o ne teisinė konsultacija.
page-run-start = Sukurti žemėlapį
page-run-again = Sukurti dar kartą
page-run-cancel = Atšaukti
page-run-cancelling = Atšaukiama …
page-warnings-heading = Įspėjimai
page-where-files = Failai yra projekto aplanke:
page-open-folder = Atidaryti aplanką
page-log-label = Paleidimo žurnalas
page-force-title = Pakeisti esamus žemėlapius?
page-force-text = Esamas žemėlapis bus pakeistas, o jei pažymėtas „Treniruočių žemėlapis“, ir treniruočių žemėlapis. Jūsų pakeitimai nežus: seni failai lieka projekto aplanke kaip .omap.bak. Tęsti?
page-force-cancel = Atšaukti
page-force-confirm = Pakeisti
page-step-fetch = Atsisiunčiami duomenys
page-step-derive = Skaičiuojami šablonai
page-step-map = Rašomas žemėlapis
page-step-ocad = Rašomas OCAD projektas
page-step-pdf = Rašomas PDF
page-step-pdf-candidates = Rašomas tikrinimo spaudinys
page-step-training = Rašomas treniruočių žemėlapis
page-step-permits = Rašomos leidimų pastabos
page-step-line = Žingsnis { $index } iš { $total }: { $label }
page-starting = Paleidimas pradedamas …
page-file-kind-map = žemėlapis
page-file-kind-template = šablonas
page-file-kind-ocad = OCAD žemėlapis
page-file-kind-ocad-candidates = OCAD kandidatai
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = tikrinimo spaudinys
page-file-kind-training-map = treniruočių žemėlapis
page-file-kind-training-pdf = treniruočių žemėlapio PDF
page-file-kind-training-ocad = OCAD treniruočių žemėlapis
page-file-kind-training-view-map = treniruočių žemėlapio vaizdas
page-file-kind-training-view-pdf = treniruočių žemėlapio vaizdo PDF
page-file-kind-training-view-ocad = OCAD treniruočių žemėlapio vaizdas
page-file-kind-preview = žemėlapio peržiūra
page-file-kind-permits = leidimų pastabos
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Baigta. Failai įrašyti.
page-result-failed = Paleidimas nepavyko: { $error }
page-result-failed-kept = Tai, kas įrašyta, lieka projekto aplanke; naujas paleidimas tęsia nuo ten.
page-result-cancelled = Atšaukta. Tai, kas jau atsisiųsta, lieka talpykloje; naujas paleidimas tęsia nuo ten.
page-result-title-done = Žemėlapis sukurtas
page-result-title-failed = Paleidimas nepavyko
page-result-title-cancelled = Paleidimas atšauktas
page-kept-map = Esamas žemėlapis paliktas nepakeistas. „Pakeisti esamą žemėlapį“ sukuria jį iš naujo, o senąjį palieka kaip .omap.bak.
page-no-files = Failų nesukurta.
page-opened-folder = Aplankas atidarytas.
page-open-failed = { $message } Aplankas: { $dir }
page-why-no-area = Pirmiausia pasirinkite plotą.
page-why-checking = Skaičiuojama peržiūra …
page-why-too-large = Plotas per didelis.
page-why-outside = Plotas yra už palaikomų federalinių žemių ribų.
page-why-disk = Nepakanka laisvos disko vietos (žr. aukščiau).
page-why-running = Vyksta paleidimas.
page-err-scale-bar-no-pdf = Mastelio liniuotei reikia PDF.
page-err-sources-no-pdf = Šaltinių eilutėms reikia PDF.
page-save-failed = Plotas neišsaugotas; paleidimas nepradėtas.
page-poll-lost = Serveris neatsako. freemapper bando toliau.
page-run-unknown = Serveris šio paleidimo nebežino.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Uždaryti

## update (spec 3.4 to 3.6)

update-source-build = Šis freemapper sukompiliuotas iš pirminio kodo. Atnaujinkite jį su git pull ir cargo build --release --locked.
update-current = freemapper { $version } yra naujausias.
update-available = Yra freemapper { $new } (jūsų { $old }), išleistas { $released }.
update-details =
    Laidos pastabos: { $notes }
    Atsisiuntimas: { $size } MB
update-decimal-separator = ,
update-ask = Įdiegti dabar? [t/N]
update-not-installed = Nieko neįdiegta.
update-needs-yes = įvestis nėra terminalas, todėl freemapper prieš diegdamas paklausti negali; paleiskite freemapper update --yes, kad įdiegtumėte nesiklausę
update-installing = atsisiunčiamas ir tikrinamas freemapper { $version }
update-done = freemapper atnaujintas iš { $old } į { $new }. Laidos pastabos: { $notes }
update-mcp-restart = Veikiantis MCP serveris išlaiko senąją versiją, kol DI asistentas jį paleis iš naujo.
update-rolled-back = Grįžta prie freemapper { $version }. Versija, kurią palikote, išsaugota kaip { $path }.
update-not-writable = negalima rašyti į programos aplanką { $path }, o freemapper neprašo administratoriaus teisių. Išpakuokite naują laidą ten rankiniu būdu arba perkelkite freemapper į aplanką, į kurį galite rašyti.
update-too-old = freemapper { $new } negalima įdiegti { $old } savaiminiu atnaujinimu; įdiekite { $needed } ar naujesnį rankiniu būdu, žr. { $notes }
update-busy = programos aplanke vyksta kitas atnaujinimas arba grąžinimas (užrakto failas { $path }); palaukite, kol jis baigsis, ir bandykite dar kartą
update-bad-signature = laidos nepavyko patikrinti ({ $error }); nieko neįdiegta
update-check-bad-signature = naujausios freemapper laidos nepavyko patikrinti ({ $error }); nieko neįdiegta
update-no-manifest = naujausia laida negali savaime įsidiegti (neturi atnaujinimo manifesto); atsisiųskite ją iš { $page }
update-no-manifest-version = laidos { $version } nėra arba ji negali savaime įsidiegti (neturi atnaujinimo manifesto); žr. { $page }
update-no-asset = laida neturi programos šiai sistemai ({ $platform }); nieko neįdiegta
update-no-previous = nėra ankstesnės versijos, prie kurios galima grįžti ({ $path } trūksta)
update-failed = atnaujinimas nepavyko: { $error }
update-mode-set = Atnaujinimo režimas nustatytas į { $mode } faile { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } pakeičia šį nustatymą, kol yra nustatytas.
update-mode-failed = nepavyko įrašyti { $path }: { $error }
update-notice = Yra freemapper { $new } (jūsų { $old }). Atnaujinkite komanda: freemapper update
update-applies-next-start = Nauja versija įsigalioja nuo kito freemapper paleidimo.
update-install-failed = freemapper { $new } neįdiegtas: { $error }
update-no-config-dir = šioje sistemoje nėra konfigūracijos aplanko; nustatykite FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Pastaba naudotojui: paleidimo metu freemapper serveris rado tai apie adapterių aplankus. Perduokite tai toliau; naudotojas pataiso aplanką arba jį pašalina, tada paleidžia serverį iš naujo.
    { $lines }
update-mcp-note = Pastaba naudotojui: yra freemapper { $new } (ši versija { $old }). Paklauskite jo prieš kviesdami install_update.
update-mcp-needs-confirm = nieko neįdiegta; pirmiausia paklauskite naudotojo ir kvieskite install_update su confirm: true, kai jis sutiks
update-mcp-note-manual = Pastaba naudotojui: yra freemapper { $new } (ši versija { $old }), bet install_update jo įdiegti negali; įdiekite { $needed } ar naujesnį rankiniu būdu, žr. { $notes }
update-mcp-note-bad-signature = Pastaba naudotojui: naujausios freemapper laidos nepavyko patikrinti ({ $error }); nieko neįdiegta. Pasakykite jam ir neįdiekite jos kitu būdu.
update-mcp-restart-now = Dabar paleiskite freemapper MCP serverį iš naujo; jis vis dar veikia su sena versija.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = klaida
error-cancelled = atšaukta
cli-cancelled-kept = atšaukta; palikta: { $files }
error-io = { $path }: { $message }
error-io-disk-full = diskas pilnas, todėl freemapper negalėjo įrašyti { $path }. Atlaisvinkite vietos ir bandykite dar kartą

## Errors: project

error-project-already-exists = projektas jau yra: { $path }
error-project-map-exists = žemėlapis jau yra (jame gali būti jūsų darbo); naudokite --force, kad jį pakeistumėte: { $path }
error-project-invalid-scale = netinkamas mastelis 1:{ $scale }
error-project-invalid-contour-interval = netinkamas horizontalių intervalas { $interval } m faile freemapper.toml; jis turi būti didesnis už 0
error-project-scale-not-allowed = mastelis 1:{ $scale } neleidžiamas { $kind } žemėlapiams; naudokite { $allowed }
error-project-scale-not-allowed-enlarge = mastelis 1:{ $scale } neleidžiamas { $kind } žemėlapiams; naudokite { $allowed }. Braižykite 1:4000, freemapper-print spausdindamas padidina iki 1:3000
word-kind-schoolyard = mokyklos kiemo
word-kind-sprint = sprinto
word-kind-forest = miško
error-project-toml = netinkamas projekto failas: { $message }
error-project-unknown-adapter = projekto faile nurodytas adapteris "{ $adapter }", kurio šis freemapper neturi; jį gali atnešti naujesnis freemapper (`freemapper update`), arba jis yra jūsų adapterių aplanke (`freemapper adapters list` jį parodo), arba, su komandinės eilutės parinktimi --adapters-from-project, projekto aplanke `adapters` (tik jūsų pasitikimam projektui)
error-project-not-fetched = plytelių dar neatsisiųsta; pirmiausia paleiskite `freemapper fetch`
error-project-ocd = OCAD eksportas: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: vaizdas ir world failas neatitinka žemėlapio ploto; dar kartą paleiskite `freemapper derive`
error-project-tiles-outdated = žemėlapio plotui reikia plytelių { $tiles }, kurios neatsisiųstos; paleiskite `freemapper fetch`
error-project-buildings-not-fetched = šiam žemėlapio plotui ALKIS pastatų neišsaugota; pirmiausia paleiskite `freemapper fetch` su tinklo ryšiu
error-project-orthophoto-not-fetched = šiam žemėlapio plotui ortofoto neišsaugota; pirmiausia paleiskite `freemapper fetch` su tinklo ryšiu
error-project-terrain-not-fetched = šiam žemėlapio plotui reljefo modelio neišsaugota; pirmiausia paleiskite `freemapper fetch` su tinklo ryšiu
error-project-candidates-outdated = išsaugoti OSM kandidatai neapima žemėlapio ploto; paleiskite `freemapper fetch`
error-project-stored = { $path }: { $message }; dar kartą paleiskite `freemapper fetch`
error-project-area-too-large = žemėlapio plotas su buferiu yra { $width } × { $height } m, daugiau nei { $limit } m vienoje pusėje; sumažinkite žemėlapio plotą arba buffer_m faile freemapper.toml
error-project-area-crosses-country = žemėlapio plotas yra abipus šalies sienos; žemėlapiai per sieną kol kas nepalaikomi, todėl braižykite plotą vienos šalies viduje
error-project-permits-not-for-region = leidimų pastabų šiam regionui dar nėra: jos klausia Vokietijos paslaugų ir nurodo Vokietijos institucijas
error-project-outside-coverage = žemėlapio plotas yra už palaikomų federalinių žemių ribų ({ $states }); kol kas freemapper duomenų šaltinių turi tik joms

## Errors: disk space

error-space-required = nepakanka disko vietos dalyje: { $part }. Reikia { $needed }, bet { $path } turi tik { $left } laisvos, neįskaitant { $reserve } rezervo. Atlaisvinkite vietos ir vėl paleiskite `freemapper fetch`
error-space-skipped = freemapper praleidžia dalį dėl disko vietos trūkumo: { $part }. Reikia { $needed }, bet { $path } turi tik { $left } laisvos, neįskaitant { $reserve } rezervo. Atlaisvinkite vietos ir vėl paleiskite `freemapper fetch`
word-part-map-data = ALKIS ir OSM žemėlapio duomenys
word-part-terrain = reljefo modelis
word-part-orthophoto = ortofoto
word-part-surface = nDSM šablono paviršiaus modelis
word-part-osm-update = talpykloje esančios OSM ištraukos atnaujinimas
word-part-k-five = Berlyno K5 žemėlapio šablonas
word-part-berlin-layers = Berlyno ALKIS statiniai, medžių kadastras, gatvių apžvalga ir taškų sluoksniai
word-part-state-layers = pilni ALKIS statiniai, kraštovaizdžio modelio objektai ir medžių kadastrai
word-part-point-cloud = augmenijos kandidatų lazerinių taškų debesys

## Errors: downloads

error-fetch-http = { $url } atsisiuntimas nepavyko: { $message }
error-fetch-http-temporary = { $url } atsisiuntimas nepavyko: { $message }. Paslauga greičiausiai perkrauta arba vykdoma priežiūra; freemapper bandė iki { $attempts } kartų. Paleiskite vykdymą iš naujo po kelių minučių; jau atsisiųsta išsaugoma.
error-fetch-corrupt-archive = { $url } atsisiuntimas nėra pilnas plytelių archyvas: { $message }
error-fetch-corrupt-extract = { $url } atsisiuntimas nėra pilna OSM ištrauka: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] plytelė { $tiles } nepatenka į paskelbtas plyteles; sumažinkite žemėlapio plotą arba buffer_m faile freemapper.toml
       *[other] plytelės { $tiles } nepatenka į paskelbtas plyteles; sumažinkite žemėlapio plotą arba buffer_m faile freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] be tinklo ir ne talpykloje: { $product } plytelė { $tiles }; paleiskite `freemapper fetch` su tinklo ryšiu
       *[other] be tinklo ir ne talpykloje: { $product } plytelės { $tiles }; paleiskite `freemapper fetch` su tinklo ryšiu
    }
word-product-orthophoto = ortofoto (DOP20)
word-product-dtm = reljefo modelis (DGM1)
word-product-image-dsm = paviršiaus modelis (bDOM)
word-product-saxony-dtm = reljefo modelis (GeoSN DGM1)
word-product-saxony-dsm = paviršiaus modelis (GeoSN DOM1)
error-fetch-osm-extract-missing = talpykloje nėra OSM ištraukos; paleiskite `freemapper fetch` su tinklo ryšiu
error-fetch-not-found = { $name } serveryje nerasta
error-fetch-refused = { $url }: serveris atsisakė užklausos (HTTP { $status })
error-fetch-refused-detail = { $url }: serveris atsisakė užklausos (HTTP { $status }): { $detail }
error-fetch-too-large = { $url } atsisiuntimas didesnis nei { $limit } MB; sustabdyta
error-fetch-not-stored = šiam žemėlapio plotui neišsaugota: { $label }: { $path } trūksta, jo neįmanoma perskaityti arba jis neapima ploto; pirmiausia paleiskite `freemapper fetch` su tinklo ryšiu
error-fetch-range-ignored = { $url }: serveris ignoravo baitų intervalą { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper nesiunčia užklausų šiuo adresu: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper nesiunčia užklausų šiuo adresu: { $name } adapteris nenurodo hosto { $host }{ $from }. Jei paslauga persikėlė ten, leiskite hostą komanda `freemapper adapters allow-host { $adapter } { $host }` ir praneškite apie tai adresu https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper nesiunčia užklausų šiuo adresu: adapteris { $adapter } nenurodo hosto { $host }{ $from }. Jei paslauga persikėlė ten, pridėkite hostą į adapterio manifesto `[net] hosts`
error-fetch-host-redirected-from = {" "}({ $url } nukreipia ten)
error-reason-not-a-host-name = tai ne hosto vardas
error-reason-not-ascii = vardas ne ASCII turi būti užrašytas xn-- forma
error-reason-not-a-dns-name = tai ne DNS vardas
error-reason-no-domain = šio kompiuterio vardas arba be domeno
error-reason-local-name = vietinio tinklo vardas
error-reason-ip-address = IP adresas, o ne DNS vardas
error-reason-unexpected-character = netikėtas simbolis
error-reason-not-https = ne https
error-reason-user-info = naudotojo informacija
error-reason-port = kitas prievadas nei 443
error-reason-host-not-declared = hostas nėra vienas iš deklaruotų hostų
error-reason-local-address = šio kompiuterio arba vietinio tinklo adresas, o ne interneto serverio
error-reason-too-many-redirects = daugiau nei { $count ->
        [one] { $count } nukreipimas
        [few] { $count } nukreipimai
       *[other] { $count } nukreipimų
    }
error-reason-redirect-nowhere = nukreipimas (HTTP { $status }) į niekur
error-fetch-currency-csv = aktualumo CSV eilutė { $line }: { $message }
error-alkis = netinkami ALKIS duomenys: { $message }
error-osm = OSM duomenys: { $message }

## Errors: templates and rasters

error-derive-zip = plytelių archyvas: { $message }
error-derive-image = vaizdas: { $message }
error-derive-no-tiles = nėra plytelių, iš kurių būtų galima skaičiuoti
error-derive-area-too-large = žemėlapio plotas per didelis vienam šablonui: { $width } × { $height } pikselių (daugiausia { $limit } vienoje pusėje); sumažinkite žemėlapio plotą arba buffer_m faile freemapper.toml
error-raster-world-file = netinkamas world failas: { $message }
error-raster-misaligned = rasteriai nesulygiuoti pagal tą patį pikselių tinklelį: { $message }
error-raster-pixel-size = rasteriai turi skirtingus pikselių dydžius: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = netinkamas GeoJSON: { $message }
error-area-no-polygon = GeoJSON neturi poligono
error-area-multi-polygon = MultiPolygon su { $parts } dalimis; padalykite žemėlapio plotą į po vieną poligoną
error-area-out-of-range = koordinatė už ribų: lat { $lat }, lon { $lon }
error-area-not-an-area = netinkamas žemėlapio plotas: { $message }
error-georef-date = data { $date } yra už World Magnetic Model galiojimo ribų
error-georef-map-range = taškas E { $easting } N { $northing } yra už žemėlapio koordinačių ribų
error-omap-xml = netinkamas OOM XML: { $message }
error-omap-missing-element = elemento <{ $name }> nerasta
error-ocd-duplicate-symbol = simbolio numeris { $symbol } naudojamas du kartus
error-ocd-unknown-symbol = objektas nurodo simbolį { $symbol }, kurio faile nėra
error-ocd-wrong-type = simbolio { $symbol } objektas turi netinkamą savo simboliui tipą

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Naudojimas:
cli-heading-commands = Komandos:
cli-heading-options = Parinktys:
cli-heading-arguments = Argumentai:
cli-help-possible-values = galimos reikšmės
cli-help-default = numatyta
cli-arg-help = Rodyti žinyną
cli-arg-version = Rodyti versiją
cli-help-help = Rodyti komandos žinyną
cli-arg-help-command = Komanda, kurios žinyną rodyti, pavyzdžiui, run
cli-parse-hint = Daugiau informacijos gausite su '--help'.
cli-double-click-start = freemapper naršyklėje atidaro puslapį, kuriame pasirenkate žemėlapio plotą. Projekto aplankas yra { $path }. Uždarykite šį langą, kad baigtumėte programą.
cli-parse-unknown-argument = netikėtas argumentas { $arg }
cli-parse-missing-required = trūksta privalomo argumento: { $args }
cli-parse-invalid-value = netinkama reikšmė { $value } argumentui { $arg }
cli-parse-invalid-value-choices = netinkama reikšmė { $value } argumentui { $arg }; galimos reikšmės: { $values }
cli-parse-invalid-subcommand = nežinoma komanda { $value }
cli-parse-missing-subcommand = reikalinga komanda
cli-parse-too-many-values = netikėta reikšmė { $value } argumentui { $arg }
cli-parse-argument-conflict = { $arg } negalima naudoti kartu su { $other }

help-freemapper = Kuria OOM pradinius projektus iš geoduomenų
help-freemapper-lang = Programos tekstų kalba, `de` arba `en` (numatyta: FREEMAPPER_LANG, tada sistemos kalba, tada anglų)
help-init = Sukurti projekto aplanką žemėlapio plotui
help-init-dir = Kuriamas projekto aplankas
help-init-geojson = GeoJSON failas su žemėlapio ploto poligonu (EPSG:4326)
help-init-osm = Žemėlapio ploto OSM objektas, pvz. way/992423119
help-init-place = Vietos pavadinimas, pvz. "Clara-Zetkin-Schule Strausberg"; freemapper naudoja geriausią atitiktį su plotu
help-init-part = Kurią kelių atskirų dalių OSM ploto dalį naudoti (1 yra didžiausia)
help-init-kind = Žemėlapio rūšis
help-arg-scale = Naujo projekto mastelio vardiklis: mokyklos kiemas 1000 (numatyta); sprintas 4000 (numatyta) arba 5000; miškas 15000 (numatyta), 10000 arba 7500. Kitos reikšmės atmetamos, nes freemapper-print jas atmeta
help-init-title = Žemėlapio pavadinimas pakraščio informacijai (numatyta: aplanko vardas)
help-export = Įrašyti pradinį projektą (OOM žemėlapį, šablonus, šaltinių nuorodas) į projekto aplanką
help-export-dir = Projekto aplankas
help-arg-force = Pakeisti esamą žemėlapio failą ir palikti senąjį kaip `<name>.omap.bak`
help-export-ocad = Taip pat įrašyti OCAD pradinį projektą į `ocad/` (OCAD 12 formatas); esamas OOM žemėlapis tada lieka, nebent nurodysite --force
help-export-pdf = Taip pat įrašyti `<name>.pdf`, spausdinamą žemėlapį (tik žemėlapio objektai, be kandidatų); esamas OOM žemėlapis tada lieka, nebent nurodysite --force
help-export-pdf-candidates = Taip pat įrašyti `<name>-candidates.pdf`, tikrinimo spaudinį: žemėlapį su visais kandidatais, tiksliai pagal padėtį; esamas OOM žemėlapis tada lieka, nebent nurodysite --force
help-export-training = Taip pat įrašyti treniruočių žemėlapį `<name>-training.omap` ir jo PDF `<name>-training.pdf`: žemėlapį ir visus kandidatus, sutvarkytus taip, kad juos būtų galima spausdinti tokius, kokie yra; su --ocad taip pat `ocad/<name>-training.ocd`. Esamas OOM žemėlapis tada lieka, nebent nurodysite --force, o --force taip pat pakeičia esamą treniruočių žemėlapį
help-export-view = Su --training: taip pat įrašyti treniruočių žemėlapį be to, ką vaizdas praleidžia, kaip `<name>-training-<view>.omap` ir jo PDF (su --ocad taip pat OCAD failą). Vaizdai: `no-paths` (be kelių, takų, laiptų), `brown-only` (tik rudi simboliai ir rėmelis), `no-north-lines`, arba sąrašas `no:<codes>` / `only:<codes>`; sujunkite su `+`. Kelių failų atveju kartokite parinktį
help-export-pdf-scale-bar = Braižyti mastelio liniuotę PDF failuose (numatyta išjungta, Vokietijos žemėlapiai paprastai jos neturi; mastelio skaičius lieka antraštės bloke)
help-export-pdf-sources = Šaltinių eilutės PDF failuose: `auto` spausdina pilną sąrašą ir sutrumpina jį tik tada, kai jis užimtų daugiau nei 5 eilutes, `full` niekada netrumpina, `credit-only` spausdina tik freemapper autorystės eilutę (ODbL ir dl-de/by-2-0 tada reikalauja nurodyti šaltinius, kai žemėlapį perduodate toliau). ATTRIBUTION.txt visada išvardija kiekvieną šaltinį
help-fetch = Atsisiųsti žemėlapio ploto duomenis (ortofoto, reljefo modelį, nDSM, ALKIS pastatus, OSM)
help-fetch-dir = Projekto aplankas
help-fetch-offline = Naudoti tik talpyklą, be prieigos prie tinklo
help-arg-cache-dir = Atsisiųstų plytelių talpyklos aplankas (numatyta: sistemos talpyklos aplankas + `freemapper`)
help-derive = Apskaičiuoti šablonus (ortofoto, šešėliuotą reljefą, nuolydį, vietinį reljefą, nDSM), horizontales ir reljefo formas iš atsisiųstų duomenų
help-derive-dir = Projekto aplankas
help-run = Atsisiųsti, apskaičiuoti ir eksportuoti vienu žingsniu
help-run-dir = Projekto aplankas
help-run-offline = Naudoti tik talpyklą, be prieigos prie tinklo
help-run-ocad = Taip pat įrašyti OCAD pradinį projektą į `ocad/` (OCAD 12 formatas)
help-run-pdf = Taip pat įrašyti `<name>.pdf`, spausdinamą žemėlapį (tik žemėlapio objektai, be kandidatų)
help-run-pdf-candidates = Taip pat įrašyti `<name>-candidates.pdf`, tikrinimo spaudinį: žemėlapį su visais kandidatais, tiksliai pagal padėtį
help-run-training = Taip pat įrašyti treniruočių žemėlapį `<name>-training.omap` ir jo PDF `<name>-training.pdf`: žemėlapį ir visus kandidatus, sutvarkytus taip, kad juos būtų galima spausdinti tokius, kokie yra; su --ocad taip pat `ocad/<name>-training.ocd`. Esamas treniruočių žemėlapis lieka, nebent nurodysite --force
help-run-view = Su --training: taip pat įrašyti treniruočių žemėlapį be to, ką vaizdas praleidžia, kaip `<name>-training-<view>.omap` ir jo PDF (su --ocad taip pat OCAD failą). Vaizdai: `no-paths` (be kelių, takų, laiptų), `brown-only` (tik rudi simboliai ir rėmelis), `no-north-lines`, arba sąrašas `no:<codes>` / `only:<codes>`; sujunkite su `+`. Esamas vaizdo failas lieka, nebent nurodysite --force
help-run-pdf-scale-bar = Braižyti mastelio liniuotę PDF failuose (numatyta išjungta; mastelio skaičius lieka antraštės bloke)
help-run-pdf-sources = Šaltinių eilutės PDF failuose: `auto` spausdina pilną sąrašą ir sutrumpina jį tik tada, kai jis užimtų daugiau nei 5 eilutes, `full` niekada netrumpina, `credit-only` spausdina tik freemapper autorystės eilutę (ODbL ir dl-de/by-2-0 tada reikalauja nurodyti šaltinius, kai žemėlapį perduodate toliau). ATTRIBUTION.txt visada išvardija kiekvieną šaltinį
help-select = Pasirinkti žemėlapio plotą naršyklėje nubraižant jį arba parenkant vietą. Išsaugojus įrašomas freemapper.toml ir, jei reikia, sukuriamas projekto aplankas
help-select-dir = Projekto aplankas (išsaugant sukuriamas, jei jo nėra)
help-select-kind = Naujo projekto žemėlapio rūšis
help-select-title = Naujo projekto žemėlapio pavadinimas
help-select-no-browser = Tik išspausdinti puslapio adresą; neatidaryti naršyklės
help-select-port = Prievadas 127.0.0.1 (numatyta: laisvas)
help-select-offline = Puslapis prasideda su pažymėta „tik talpykla, be tinklo“
help-select-ocad = Puslapis prasideda su pažymėtu OCAD pradiniu projektu
help-select-pdf = Puslapis prasideda su pažymėtu spausdinamu PDF
help-select-pdf-candidates = Puslapis prasideda su pažymėtu tikrinimo spaudiniu (visi kandidatai)
help-select-training = Puslapis prasideda su pažymėtu treniruočių žemėlapiu
help-select-pdf-scale-bar = Puslapis prasideda su pažymėta mastelio liniuote PDF failuose
help-select-pdf-sources = Puslapis prasideda su šiuo šaltinių eilučių PDF failuose pasirinkimu, kaip `run`
help-find = Išvardyti OpenStreetMap vietas pagal pavadinimą, pirmiausia plotus, su id komandai init --osm
help-find-query = Vietos pavadinimas, pvz. "Clara-Zetkin-Schule Strausberg"
help-find-json = Išspausdinti atitiktis kaip JSON, tie patys laukai kaip MCP įrankio find_place
help-mcp = Teikti įrankius DI agentui per MCP (stdin/stdout)
help-update = Įdiegti naujesnį freemapper iš GitHub laidų, prieš tai paklausus. Sukompiliuotas iš pirminio kodo freemapper vietoj to pasako, kaip jį atnaujinti
help-update-check = Tik patikrinti: išėjimo kodas 10, kai yra naujesnė versija, 0, kai ši yra naujausia
help-update-yes = Įdiegti nesiklausus; reikalinga, kai įvestis nėra terminalas
help-update-version = Įdiegti šią versiją (x.y.z) vietoj naujausios, taip pat ir senesnę
help-update-rollback = Grįžti prie versijos, kurią pakeitė paskutinis atnaujinimas
help-update-mode = Išsaugoti, ką daro kasdienė patikra, radusi naujesnę versiją (numatyta: notify). Veikia kiekvienoje versijoje, nes tik įrašo nustatymą
help-adapters = Rodyti regionų adapterius: federalines žemes ir šalis, kurioms freemapper turi duomenų šaltinių
help-adapters-list = Išvardyti adapterius su tuo, iš kur kiekvienas kilęs (įtaisytas, jūsų aplankas, projektas), jo redakcija, paskutinio patikrinimo diena ir hostais, į kuriuos jis kreipiasi
help-adapters-list-json = Išspausdinti sąrašą kaip JSON
help-adapters-list-project = Taip pat skaityti adapterius šio projekto aplanko aplanke `adapters`, kaip darytų --adapters-from-project
help-adapters-check = Patikrinti adapterio aplanką prieš jį naudojant ar perduodant: ar jis įsikeltų, ką jo autorius turėtų žinoti, ir su --live, ar jo paslaugos atsako. Išėjimo kodas 0: patikrą išlaiko; 1: yra pastabų; 2: patikros paleisti nepavyko
help-adapters-check-dir = Adapterio aplankas; jo vardas yra adapterio id
help-adapters-check-live = Taip pat paklausti adapterio paslaugų apie langelį, kurį nurodo --bbox, ir pasakyti, ką kiekviena pateikia. Užklausos lieka manifeste deklaruotuose hostuose
help-adapters-check-bbox = Gyvos patikros langelis laipsniais, kaip minLon,minLat,maxLon,maxLat. Jis yra adapterio ribų viduje; pakanka 150 m × 150 m
help-adapters-check-json = Išspausdinti rezultatą kaip JSON
help-adapters-probe = Vieną kartą užklausti kiekvieną integruotų adapterių nurodytą paslaugą ir pasakyti, ar ji atsako: ok, slow, moved arba error. Išėjimo kodas yra 0 ir tada, kai paslauga nepavyksta; rezultatas yra atsakymas
help-adapters-probe-json = Išspausdinti rezultatą kaip JSON
help-adapters-probe-status = Perskaityti praėjusios nakties būseną iš šio failo (jo gali nebūti) ir išspausdinti naują būseną kaip JSON: paslauga laikoma neveikiančia po dviejų blogų naktų iš eilės. Pokyčiai eina į standartinę klaidų išvestį
help-adapters-allow-host = Leisti įtaisytam adapteriui dar vieną hostą: federalinei žemei, kurios paslauga persikėlė į serverį, kurio jos adapteris dar nenurodo. Pranešimas apie atmestą adresą pasako, kuris hostas. Hostas tikrinamas kaip manifesto hostas, o freemapper vis tiek jungiasi tik prie interneto serverių
help-adapters-allow-host-adapter = Įtaisyto adapterio id, kaip rodo `adapters list`
help-adapters-allow-host-host = Leidžiamas hostas: DNS vardas, ne adresas, be prievado
help-adapters-disallow-host = Atšaukti hostą, kurį leido allow-host
help-adapters-disallow-host-adapter = Įtaisyto adapterio id
help-adapters-disallow-host-host = Atšaukiamas hostas
help-arg-adapters-from-project = Taip pat įkelti adapterius projekto aplanko aplanke `adapters`. freemapper įvardija kiekvieną iš jų su hostais, į kuriuos jis kreipiasi. Naudokite pasitikimam projektui: adapteris priverčia freemapper kreiptis į jo nurodytus serverius

## The summary of a run, for the MCP server and the area page

cli-internal-error = vidinė klaida: { $message }
cli-pdf-sources-invalid = pdf_sources turi būti auto, full arba credit-only, o ne { $value }
cli-view-unknown = vaizdas yra no-paths, brown-only, no-north-lines, no:<codes> arba only:<codes>, o ne { $value }
cli-view-empty = sąraše { $value } nėra nė vieno simbolio kodo
cli-view-bad-code = { $value } nėra simbolio kodas, kaip 505 ar 501.6
cli-summary-training-view = Įrašyta { $map } ir { $pdf } (treniruočių žemėlapio vaizdas)
cli-summary-kept-training-view = Palikti treniruočių žemėlapio vaizdo { $view } failai; nurodykite --force, kad juos atnaujintumėte
cli-summary-training-view-failed = Treniruočių žemėlapio vaizdas nepavyko: { $error }
cli-estimate-disk-unreadable = nepavyksta nuskaityti laisvos disko vietos { $path } ({ $message })
cli-summary-wrote-map = Įrašytas žemėlapis { $path }
cli-summary-kept-map = Paliktas esamas žemėlapis { $path } (jame gali būti kartografo darbo)
cli-summary-map-lacks-templates = Žemėlapis dar nenurodo vaizdų šablonų. Paleiskite konvejerį su force, kad jį pakeistumėte ir senąjį palikti kaip .omap.bak.
cli-summary-map-lacks-buildings = Žemėlapyje dar nėra ALKIS pastatų. Paleiskite konvejerį su force, kad juos pridėtumėte; jis pakeičia žemėlapį ir senąjį palieka kaip .omap.bak.
cli-summary-map-lacks-canopies = Žemėlapyje trūksta ALKIS stogelių ir automobilių pastogių (522), kurie dabar yra žemėlapio objektai. Paleiskite konvejerį su force, kad juos pridėtumėte; jis pakeičia žemėlapį ir senąjį palieka kaip .omap.bak.
cli-summary-map-light = Pagrindinis žemėlapis: sujungta { $joined } linijų, { $left } paslėptų arba per mažų palikta kandidatų sluoksniams (niekas nepajudinta)
cli-summary-map-light-kept = Kandidatų sluoksniuose dabar yra ir { $left } objektų ar atkarpų, kuriuos ši versija nuima nuo pagrindinio žemėlapio (pažymėta freemapper:generalized). Palikti žemėlapyje jie vis dar yra, todėl rodomi dukart. Paleiskite konvejerį su force, kad žemėlapį sutvarkytumėte; jis senąjį palieka kaip .omap.bak.
cli-summary-map-old-candidates = Žemėlapis vis dar nurodo templates/candidates.omap; šis paleidimas jį užpildė visais kandidatais. Kandidatai dabar taip pat pateikiami septyniais sluoksniais, po vieną failą (templates/candidates-*.omap). Žemėlapis juos išvardija po paleidimo su force (senasis žemėlapis lieka kaip .omap.bak), arba kartografas prideda juos OOM (Templates > Open template…).
cli-summary-wrote-ocad = Įrašytas OCAD žemėlapis { $path } ({ $count } objektų)
cli-summary-kept-ocad = Paliktas esamas OCAD žemėlapis { $path } (jame gali būti kartografo darbo; force jį pakeičia)
cli-summary-ocad-old-candidates = OCAD žemėlapis vis dar nurodo { $path } kaip fono žemėlapį; šis paleidimas jį užpildė visais kandidatais. Kandidatai dabar taip pat pateikiami septyniais sluoksniais, po vieną failą (-candidates-*.ocd). OCAD žemėlapis juos išvardija po paleidimo su force (senasis žemėlapis lieka kaip .ocd.bak), arba kartografas prideda juos OCAD kaip fono žemėlapius.
cli-summary-ocad-left-out = OCAD žemėlapyje nėra: { $skipped } objektų ir { $dropped } išsigimusių dalių
cli-summary-ocad-symbols = OCAD pasirinkite Symbol > Load Colors and Symbols From… ir nurodykite oficialų simbolių rinkinį{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , IOF mokyklų rinkinį 2019
       *[other] {" "}ISSprOM 2019
    }; jis pakeičia vietos rezervavimo simbolius pagal numerį.
cli-summary-ocad-failed = OCAD eksportas nepavyko: { $error }
cli-summary-pdf-without-credits = PDF be šaltinių nuorodų: ODbL ir dl-de/by-2-0 reikalauja nurodyti šaltinius, kai žemėlapį perduodate toliau; žr. ATTRIBUTION.txt
cli-summary-wrote-pdf = Įrašytas žemėlapis { $path } (be kandidatų)
cli-summary-pdf-failed = PDF eksportas nepavyko: { $error }
cli-summary-wrote-pdf-candidates = Įrašytas tikrinimo spaudinys { $path } (žemėlapis su visais kandidatais, tiksliai pagal padėtį)
cli-summary-pdf-candidates-failed = Tikrinimo spaudinio eksportas nepavyko: { $error }
cli-summary-training-failed = Treniruočių žemėlapio eksportas nepavyko: { $error }
cli-summary-kept-training = Paliktas esamas treniruočių žemėlapis { $path }, kad paleidimas neperrašytų atspausdinto ar kitiems perduoto; ištrinkite jį arba nurodykite force, kad jį atnaujintumėte

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = šioje sistemoje nėra namų aplanko; naudokite absoliutų kelią
mcp-error-relative-path = { $path } yra santykinis kelias; naudokite absoliutų kelią arba pradedantį ~
mcp-error-home-folder = { $path } yra pats namų aplankas; naudokite jo viduje esantį aplanką, pvz. ~/maps/school
mcp-error-no-project = aplanke { $path } freemapper projekto nėra; pirmiausia naudokite create_project
mcp-error-unknown-kind = nežinoma rūšis { $kind }; naudokite "schoolyard", "sprint" arba "forest"
mcp-error-reading = skaitoma { $path }: { $error }
mcp-error-one-area-source = nurodykite lygiai vieną: osm_id (iš find_place) arba geojson
mcp-error-page-open = šio projekto ploto pasirinkimo puslapis vis dar atidarytas; paprašykite naudotojo išsaugoti plotą ir puslapyje paspausti „Baigta“, tada paleiskite konvejerį
mcp-error-no-job = užduoties { $job } nėra; start_pipeline ir permits_for_area grąžina šios sesijos užduočių id
mcp-error-job-running = { $job } vis dar vykdoma projektui { $path }; vienu metu vyksta vienas konvejeris, nes projektai dalijasi atsisiuntimų talpykla. Šį paleiskite, kai project_status praneš finished arba failed
mcp-error-page-start = puslapio paleisti nepavyksta: { $error }
mcp-created-project = Sukurtas projektas { $path } ({ $area }, { $kind ->
        [schoolyard] mokyklos kiemas
        [forest] miškas
       *[sprint] sprintas
    }, 1:{ $scale }). Toliau: estimate_download, tada start_pipeline.
mcp-created-project-given-area = Sukurtas projektas { $path } nurodytam plotui ({ $kind ->
        [schoolyard] mokyklos kiemas
        [forest] miškas
       *[sprint] sprintas
    }, 1:{ $scale }). Toliau: estimate_download, tada start_pipeline.
mcp-page-already-open = Šio aplanko puslapis jau atidarytas; jis išlaiko nustatymus, su kuriais buvo atidarytas.
mcp-page-next = Paprašykite naudotojo puslapyje pasirinkti plotą, paspausti „Išsaugoti“ ir tada „Baigta“.
mcp-preview-of = { $layer } peržiūra
mcp-permits-next = Leidimų pastabos kuriamos. Klauskite project_status su šiuo job_id; kai bus finished, permits_md turės permits.md tekstą. Perduokite jį kaip užuominą, o ne teisinę konsultaciją.
mcp-tool-check-update = Ar yra naujesnė freemapper laida: jos versija, išleidimo data, laidos pastabos ir atsisiuntimo dydis, arba kad ši yra naujausia. Nieko neįdiegia.
mcp-tool-create-project = Sukurti projekto aplanką žemėlapio plotui. Nurodykite osm_id (iš find_place) arba GeoJSON failą. kind yra schoolyard (1:1000, numatyta), sprint (1:4000) arba forest (1:15000).
mcp-param-create-project-dir = Naujas projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-param-create-project-geojson = Vietoj osm_id: GeoJSON failas su žemėlapio ploto poligonu (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (numatyta, 1:1000), "sprint" (1:4000) arba "forest" (1:15000).
mcp-param-create-project-osm-id = Žemėlapio ploto OSM objektas iš find_place, pvz. "way/992423119".
mcp-param-create-project-part = Su osm_id: kurią kelių atskirų dalių ploto dalį naudoti (1 yra didžiausia).
mcp-param-create-project-scale = Mastelio vardiklis: mokyklos kiemas 1000 (numatyta); sprintas 4000 (numatyta) arba 5000; miškas 15000 (numatyta), 10000 arba 7500. Kitos reikšmės atmetamos; 1:3000 braižykite 4000, freemapper-print spausdindamas padidina.
mcp-param-create-project-title = Žemėlapio pavadinimas pakraščio informacijai (numatyta: aplanko vardas).
mcp-tool-estimate-download = Ką start_pipeline atsisiųs projektui: plyteles, kiekvieną atsisiuntimą su dydžiu MB, ar jis talpykloje ir ar telpa diske, dar atsisiųstinus MB, o po disk talpyklos ir projekto tomų laisvą vietą su pranešimais apie tai, kas netelpa (pasakykite naudotojui).
mcp-param-estimate-download-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-tool-find-place = Ieškoti mokyklos, parko ar kitos vietos pagal pavadinimą (OpenStreetMap Nominatim). Grąžina iki 5 atitikčių, pirmiausia plotus; žemėlapio plotu gali būti tik atitiktys su has_area. Tuščias sąrašas reiškia, kad nieko nerasta.
mcp-param-find-place-query = Mokyklos ar vietos pavadinimas, pvz. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Įdiegti naujausią freemapper laidą ant šios programos. Kvieskite tik naudotojui sutikus, su confirm true. Nauja versija įsigalioja, kai asistentas paleidžia freemapper serverį iš naujo.
mcp-param-install-update-confirm = true, kai naudotojas sutiko įdiegti naujesnę versiją; be to nieko neįdiegiama.
mcp-tool-list-layers = Projekto sluoksniai: žemėlapis, septyni kandidatų sluoksniai ir rasterių šablonai, kiekvienas su savo failu, ar jis jau sukurtas ir kuris žingsnis jį sukuria.
mcp-param-list-layers-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-tool-open-area-selector = Atidaryti naršyklėje puslapį, kuriame naudotojas nubraižo žemėlapio plotą (stačiakampį ar daugiakampį) arba parenka vietą, mato plyteles ir atsisiuntimo dydį ir jį išsaugo; išsaugojus sukuriamas projekto aplankas arba pakeičiamas jo plotas. Grąžina URL, kurį reikia duoti naudotojui. Puslapis veikia, kol naudotojas paspaudžia „Baigta“; kvietus dar kartą tam pačiam aplankui, grąžinamas tas pats URL.
mcp-param-open-area-selector-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~); sukuriamas, kai naudotojas išsaugo.
mcp-param-open-area-selector-kind = Naujo projekto žemėlapio rūšis: "schoolyard" (numatyta, 1:1000), "sprint" (1:4000) arba "forest" (1:15000).
mcp-param-open-area-selector-ocad = Paleidimo skydelis prasideda su pažymėtu OCAD pradiniu projektu. Numatyta false.
mcp-param-open-area-selector-offline = Paleidimo skydelis prasideda su pažymėta „tik talpykla, be tinklo“. Numatyta false.
mcp-param-open-area-selector-open-browser = Taip pat atidaryti puslapį šio kompiuterio numatytojoje naršyklėje. Numatyta true.
mcp-param-open-area-selector-pdf = Paleidimo skydelis prasideda su pažymėtu spausdinamu PDF. Numatyta false.
mcp-param-open-area-selector-pdf-candidates = Paleidimo skydelis prasideda su pažymėtu tikrinimo spaudiniu (visi kandidatai). Numatyta false.
mcp-param-open-area-selector-pdf-scale-bar = Paleidimo skydelis prasideda su pažymėta mastelio liniuote PDF failuose. Numatyta false.
mcp-param-open-area-selector-pdf-sources = Šaltinių eilutės PDF failuose, su kuriomis prasideda skydelis: "auto" (numatyta), "full" arba "credit-only", kaip start_pipeline.
mcp-param-open-area-selector-permits = Paleidimo skydelis prasideda su pažymėtomis leidimų pastabomis. Numatyta false.
mcp-param-open-area-selector-scale = Naujo projekto mastelio vardiklis: mokyklos kiemas 1000 (numatyta); sprintas 4000 (numatyta) arba 5000; miškas 15000 (numatyta), 10000 arba 7500. Kitos reikšmės atmetamos.
mcp-param-open-area-selector-title = Naujo projekto žemėlapio pavadinimas.
mcp-param-open-area-selector-training = Paleidimo skydelis prasideda su pažymėtu treniruočių žemėlapiu. Numatyta false.
mcp-tool-preview-layer = Sluoksnio PNG peržiūra (daugiausia 800 px): map (žemėlapio objektų vaizdas, kurį įrašė paskutinis paleidimas, be kandidatų) arba rasterio šablonas: orthophoto, hillshade, slope, relief, ndsm, arba k5 Berlyne.
mcp-param-preview-layer-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" arba "k5" (tik Berlynas).
mcp-tool-project-status = start_pipeline arba permits_for_area užduoties būsena: running, finished arba failed, paskutinės eigos eilutės, įspėjimai, įrašyti failai ir rezultatas arba klaida. Baigta permits_for_area užduotis taip pat pateikia permits.md tekstą kaip permits_md.
mcp-param-project-status-job-id = Užduoties id iš start_pipeline arba permits_for_area, pvz. "job-1".
mcp-tool-permits-for-area = Sudaryti projekto leidimų pastabas: žemėlapio ploto administracinius vienetus, jame esančias saugomas teritorijas ir kas ten valdo žemę, su institucijomis, kurių reikia klausti; užuomina, o ne teisinė konsultacija. Įrašo permits.md ir permits.json į aplanką. Veikia fone ir grąžina job_id; klauskite project_status, kurio atsakyme baigus užduotį permits.md tekstas pateikiamas kaip permits_md. Vienu metu vyksta viena užduotis.
mcp-param-permits-for-area-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-param-permits-for-area-offline = Skaityti tik duomenis, kuriuos ankstesnis paleidimas išsaugojo projekte, be prieigos prie tinklo. Numatyta false.
mcp-tool-start-pipeline = Atsisiųsti duomenis, apskaičiuoti šablonus ir įrašyti pradinį projektą (fetch, derive, export). Veikia fone ir grąžina job_id; klauskite project_status. Vienu metu vyksta vienas konvejeris. Esamą žemėlapį palieka, nebent force yra true.
mcp-param-start-pipeline-dir = Projekto aplankas (absoliutus kelias arba pradedantis ~).
mcp-param-start-pipeline-force = Pakeisti esamą žemėlapio failą ir palikti senąjį kaip .omap.bak. Numatyta false.
mcp-param-start-pipeline-ocad = Taip pat įrašyti OCAD pradinį projektą į aplanką ocad/ (OCAD 12 formatas, tiems, kurie redaguoja OCAD). Numatyta false.
mcp-param-start-pipeline-offline = Naudoti tik atsisiuntimų talpyklą, be prieigos prie tinklo. Numatyta false.
mcp-param-start-pipeline-pdf = Taip pat įrašyti <name>.pdf, žemėlapį, paruoštą spausdinti (tik žemėlapio objektai, be kandidatų). Numatyta false.
mcp-param-start-pipeline-pdf-candidates = Taip pat įrašyti <name>-candidates.pdf, tikrinimo spaudinį: žemėlapį su visais kandidatais, tiksliai pagal padėtį (patikrinkite juos vietovėje). Numatyta false.
mcp-param-start-pipeline-pdf-scale-bar = Braižyti mastelio liniuotę PDF failuose. Numatyta false (Vokietijos žemėlapiai paprastai jos neturi; mastelio skaičius lieka antraštės bloke).
mcp-param-start-pipeline-pdf-sources = Šaltinių eilutės PDF failuose: "auto" (numatyta; pilnas sąrašas, sutrumpinamas tik tada, kai užimtų daugiau nei 5 eilutes), "full" (niekada netrumpinama) arba "credit-only" (tik freemapper autorystės eilutė; ODbL ir dl-de/by-2-0 tada reikalauja nurodyti šaltinius, kai žemėlapis perduodamas toliau). ATTRIBUTION.txt visada išvardija kiekvieną šaltinį.
mcp-param-start-pipeline-permits = Taip pat įrašyti leidimų pastabas (permits.md ir permits.json) po žemėlapio: administracinius vienetus, saugomas teritorijas ir kas valdo žemę. Nesėkmė čia yra įspėjimas, o ne nepavykusi užduotis. Numatyta false.
mcp-param-start-pipeline-training = Taip pat įrašyti treniruočių žemėlapį <name>-training.omap ir jo PDF <name>-training.pdf: žemėlapį ir visus kandidatus, sutvarkytus taip, kad juos būtų galima spausdinti tokius, kokie yra; su ocad taip pat ocad/<name>-training.ocd. Esamas treniruočių žemėlapis paliekamas, nebent force yra true. Numatyta false.
mcp-param-start-pipeline-training-views = Su training: taip pat įrašyti treniruočių žemėlapį be to, ką vaizdas praleidžia, po vieną failų rinkinį kiekvienam įrašui: "no-paths", "brown-only", "no-north-lines" arba sąrašas, pvz. "no:505,506" ar "only:101,102", sujungtas su +. Failai yra <name>-training-<view>.omap ir jo PDF.
mcp-instructions = freemapper kuria OpenOrienteering Mapper pradinius projektus orientavimosi sporto žemėlapiams (mokyklos kiemas 1:1000, sprintas 1:4000, miškas 1:15000) iš oficialių geoduomenų: { $states }, ir iš OSM. Eiga: find_place (ieškoti mokyklos ar ploto pagal pavadinimą; pasirinkti rezultatą su has_area) → create_project (naujas aplankas, absoliutus kelias, su osm_id) → estimate_download (pasakyti naudotojui, kiek bus atsisiųsta) → start_pipeline (grąžina užduoties id; atsisiuntimas gali užtrukti minutes) → project_status (klausti kas kelias sekundes, kol būsena bus finished arba failed) → list_layers ir preview_layer (parodyti naudotojui šablonus). Tada naudotojas atidaro žemėlapio failą OpenOrienteering Mapper. Kai joks plotas netinka arba naudotojas nori nubraižyti plotą, open_area_selector naršyklėje atidaro puslapį, kuriame naudotojas nubraižo arba parenka plotą ir jį išsaugo (tai sukuria projektą); tęskite su estimate_download, kai naudotojas pasako, kad išsaugota. Keliai turi būti absoliutūs arba prasidėti ~. Vienu metu vyksta vienas konvejeris. permits_for_area sudaro projekto leidimų pastabas (administraciniai vienetai, saugomos teritorijos, kas valdo žemę; užuomina, o ne teisinė konsultacija) kaip užduotį, panašią į start_pipeline. check_update pasako, ar yra naujesnis freemapper; install_update jį įdiegia, tik su confirm: true, naudotojui sutikus, o asistentas tada paleidžia freemapper serverį iš naujo.

cli-contour-interval-saved = Horizontalių intervalas išsaugotas faile freemapper.toml: { $interval } m; derive horizontales braižo pagal jį
cli-contour-interval-auto = Horizontalių intervalas faile freemapper.toml: automatinis (pagal žemėlapio rūšį; miško žemėlapis lygumoje gauna 2,5 m, stataus reljefo vietoje 10 m)
help-arg-contour-interval = Horizontalių intervalas metrais, pvz. `2.5`, arba `auto` pagal žemėlapio rūšį (miško žemėlapis: 5 m, 2,5 m kur lyguma, 10 m kur status reljefas). Išsaugomas faile freemapper.toml; derive horizontales braižo pagal jį
cli-content-saved = Turinio nustatymas išsaugotas faile freemapper.toml: { $summary }
cli-content-invalid = Turinio nustatymo perskaityti nepavyksta: { $detail }
help-arg-content = Kas patenka tiesiai į žemėlapį: `safe` palieka kiekvieną objektą ten, kur jį padeda sudarymas, `balanced` (numatyta) deda kadastro ir registrų objektus į žemėlapį, `full` deda į žemėlapį viską. Pasirinkimas išsaugomas faile freemapper.toml
help-arg-content-set = Nukreipti objektų grupę į `map`, `candidates` arba `off`, kaip `GROUP=PLACE`, pvz. `walls=map`; galima kartoti. Grupės: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Išsaugoma faile freemapper.toml
mcp-param-create-project-content = Kas patenka tiesiai į žemėlapį: "safe", "balanced" (numatyta) arba "full". Išsaugoma projekte. balanced deda kadastro ir registrų objektus į žemėlapį, full deda į jį viską.
mcp-param-create-project-content-set = Grupės, nukreiptos kitur, nei sako profilis, kaip "group=place", kur place yra map, candidates arba off, pvz. "walls=map". Grupės: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Horizontalių intervalas metrais, pvz. 2.5; 0 grąžina automatinį pasirinkimą (pagal žemėlapio rūšį; miško žemėlapis lygumoje gauna 2,5 m, stataus reljefo vietoje 10 m). Išsaugoma projekte; kitas derive horizontales braižo pagal jį.
mcp-param-start-pipeline-content = Pakeisti, kas patenka tiesiai į žemėlapį, prieš paleidimą: "safe", "balanced" arba "full". Išsaugoma projekte. Numatyta: projekto nustatymas, kitaip balanced.
mcp-param-start-pipeline-content-set = Grupės, nukreiptos kitur, nei sako profilis, kaip "group=place", kur place yra map, candidates arba off, pvz. "fences=candidates". Išsaugoma projekte.
mcp-param-start-pipeline-contour-interval = Horizontalių intervalas metrais, pvz. 2.5; 0 grąžina automatinį pasirinkimą (pagal žemėlapio rūšį; miško žemėlapis lygumoje gauna 2,5 m, stataus reljefo vietoje 10 m). Išsaugoma projekte; kitas derive horizontales braižo pagal jį.
mcp-error-content = Turinio nustatymo perskaityti nepavyksta: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Įrašyti projekto leidimų pastabas (permits.md ir permits.json): administracinius vienetus, saugomas teritorijas ir valdytojus, kuriuos paliečia žemėlapio plotas, ir ko tai paprastai reiškia klausti. Klausia BfN ir BKG ir, kur žemėlapio ploto federalinė žemė turi savų atvirų paslaugų, jų per savo adapterio hostus (sąrašas vadove, 12 skyrius); Šiaurės Reine-Vestfalijoje pirmą kartą atsisiunčia apie 32 MB, Brandenburge apie 4 MB, vėliau tik naujesnius failus. Skaito jau talpykloje esančią OSM ištrauką; --offline naudoja tai, kas buvo atsisiųsta anksčiau
help-permits-dir = Projekto aplankas
help-permits-offline = Naudoti tik anksčiau atsisiųstus duomenis, be prieigos prie tinklo
help-run-permits = Taip pat įrašyti leidimų pastabas (permits.md ir permits.json) po žemėlapio; klausia BfN, BKG ir federalinės žemės atvirų paslaugų (vadovas, 12 skyrius). Nesėkmė čia yra įspėjimas, o žemėlapis lieka
cli-wrote-permits = Įrašytos leidimų pastabos { $path }
cli-summary-wrote-permits = Įrašytos leidimų pastabos { $path }
cli-permits-failed = leidimų pastabų įrašyti nepavyko: { $error }
cli-permits-source-left-out = leidimų pastabose nėra { $source }: { $reason }
cli-permits-not-read = jis neperskaitytas
error-permits-stored = { $path }: { $message }; dar kartą atsisiųskite leidimų pastabas su tinklo ryšiu
error-permits-nothing-read = nė vieno leidimų pastabų šaltinio perskaityti nepavyko ({ $missing }); pirmiausia atsisiųskite juos su tinklo ryšiu
error-permits-missing-not-read = { $source }: neperskaityta

permits-title = Leidimų pastabos
permits-disclaimer = Užuomina, o ne teisinė konsultacija. Sprendžia kompetentingos institucijos. Duomenų datą ir tikslumą rasite lentelėje po antrašte „Šaltiniai ir duomenų data“; plotų gali trūkti arba jie gali būti pasenę (BfN: sujungiama kartą per metus, „Nicht für Planungszwecke geeignet.“, t. y. netinka planavimui).
permits-disclaimer-regional = Užuomina, o ne teisinė konsultacija. Sprendžia kompetentingos institucijos. Duomenų datą ir tikslumą rasite lentelėje po antrašte „Šaltiniai ir duomenų data“; plotų gali trūkti arba jie gali būti pasenę.
permits-bfn-merge = BfN kartą per metus sujungia federalinių žemių duomenis ir pažymi juos kaip netinkamus planavimui. Sprendžia federalinės žemės institucijos.
permits-created = Sudaryta { $date } { $kind ->
        [schoolyard] mokyklos kiemo žemėlapio
        [forest] miško žemėlapio
       *[other] sprinto žemėlapio
    } plotui ({ $area } ha).
permits-section-admin = Administraciniai vienetai
permits-section-protected = Saugomos teritorijos (BfN)
permits-section-osm-protected = Daugiau saugomų teritorijų iš OpenStreetMap
permits-section-operators = Valdytojai ir savininkai pagal OpenStreetMap
permits-section-hints = Užuominos
permits-section-sources = Šaltiniai ir duomenų data
permits-none-admin = VG250 žemėlapio plotui nenurodo jokio administracinio vieneto.
permits-none-protected = Pagal BfN žemėlapio plote saugomų teritorijų nėra.
permits-none-osm-protected = OpenStreetMap žemėlapio plote nenurodo jokios saugomos teritorijos, kurios BfN dar neišvardija.
permits-none-operators = OpenStreetMap žemėlapio plotui nenurodo nei valdytojo, nei savininko.
permits-none-hints = Užuominų nėra.
permits-section-not-read = { $source } duomenys neperskaityti, todėl šis skyrius tuščias, nors įrašų gali būti.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] BKG administraciniai plotai 1:250 000 (VG250)
        [bfn] BfN Vokietijos saugomos teritorijos
       *[osm] OpenStreetMap, iš ištraukos, kurią freemapper laiko savo talpykloje
    }
permits-level = { $level ->
        [land] Federalinė žemė
        [kreis] Apskritis (Kreis)
        [verwaltungsgemeinschaft] Amt arba Verwaltungsgemeinschaft
       *[gemeinde] Savivaldybė (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Paliesta pakraštyje (mažiau nei 0,5 % žemėlapio ploto):
permits-at-edge-generalised = VG250 apibendrintas iki mastelio 1:250 000. Vienetas, kuris tik paliečia pakraštį, gali būti to pasekmė.
permits-kind = { $kind ->
        [nationalpark] Nacionalinis parkas
        [nationales-naturmonument] Nacionalinis gamtos paminklas
        [naturschutzgebiet] Gamtos rezervatas (Naturschutzgebiet)
        [ffh] FFH teritorija (Natura 2000)
        [vogelschutzgebiet] Paukščių apsaugos teritorija (Natura 2000)
        [biosphaerenreservat] Biosferos rezervatas
        [biosphaerenreservat-zone] Biosferos rezervato zona
        [landschaftsschutzgebiet] Kraštovaizdžio apsaugos teritorija (Landschaftsschutzgebiet)
       *[naturpark] Gamtos parkas
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Paskelbta { $date }
permits-protected-title = Teisinis pagrindas: { $title }
permits-protected-link = Teisės aktas: { $url }
permits-protected-site-link = Teritorijos puslapis (Natura 2000): { $url }
permits-protected-year = BfN duomenų metai: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = apsaugos klasė { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = be pavadinimo
permits-osm-operator = Valdytojas pagal OSM: { $operator }
permits-osm-enclosing-gap = Čia gali trūkti OpenStreetMap plotų, kurie apima visą žemėlapio plotą be kampo jo viduje, taip pat saugomų teritorijų, OSM pažymėtų kaip santykis su `type=boundary`. BfN ir VG250 dalys to neliečia.
permits-osm-enclosing-gap-short = Čia gali trūkti „OpenStreetMap“ plotų, kurie supa visą žemėlapio sritį, bet nė vienas jų kampas nėra joje.
permits-operator-line = { $name } ({ $roles ->
        [operator] valdytojas
        [owner] savininkas
       *[both] valdytojas ir savininkas
    }): { $objects }
permits-column-source = Šaltinis
permits-column-state = Būsena
permits-column-credit = Autorystė ir licencija
permits-state-read = perskaityta, atsisiųsta { $date }
permits-state-read-osm = perskaityta, OSM ištrauka iš { $date }
permits-state-unreachable = nepasiekiama ({ $reason })
permits-state-not-read = neperskaityta
permits-no-credit = nenaudota
permits-hint-city-state = Žemėlapio plotas yra Berlyne arba Hamburge. Ten paprastai atsakinga Bezirksamt (rajono valdyba) arba federalinės žemės institucija. Šios pastabos Bezirk dar nenustato.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Saugomos teritorijos priklauso žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde). Čia tai pats miestas { $kreis }.
        [no] Saugomos teritorijos priklauso žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde), paprastai { $kreis }.
       *[unknown] Saugomos teritorijos priklauso kompetentingai žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde), paprastai Landkreis arba kreisfreie Stadt.
    }
permits-hint-national-park = Nacionalinis parkas turi savo taisyklių. Anksti kreipkitės į nacionalinio parko administraciją.
permits-hint-nsg-paths = Gamtos rezervate paprastai reikia laikytis takų. Bėgti ne takais leidimo klauskite gamtos apsaugos institucijos.
permits-hint-natura2000 = FFH teritorija arba paukščių apsaugos teritorija (Natura 2000) paliečia žemėlapio plotą. Renginiui gamtos apsaugos institucija gali pareikalauti poveikio vertinimo.
permits-hint-protected-areas-pl = Žemėlapio srityje yra saugomų teritorijų arba gamtos apsaugos objektų. Nacionalinio parko, kraštovaizdžio parko ar gamtos rezervato atveju paprastai kreipiamasi į parko ar rezervato administraciją; dėl kitų teritorijų ir objektų, kas atsakingas, paprastai pasakys vaivadijos regioninė aplinkos apsaugos direkcija (RDOŚ). Teiraukitės iš anksto, jei ten planuojate renginį.
permits-hint-order-office = { $amt ->
        [none] Gatvėms, aikštėms ir kitai viešai žemei paprastai atsakingas Gemeinde { $gemeinde } Ordnungsamt (viešosios tvarkos tarnyba).
       *[other] Gatvėms, aikštėms ir kitai viešai žemei paprastai atsakingas Ordnungsamt (viešosios tvarkos tarnyba): { $amt }, Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Žemėlapio plote esančio parko ar sodo valdytojas pagal OpenStreetMap: { $operator }. Prieš naudodami, paklauskite jų.
permits-hint-school-operator = Mokyklos ar įstaigos valdytojas pagal OpenStreetMap: { $operator }. Dėl jos teritorijos klauskite mokyklos vadovo arba šios įstaigos.
permits-section-districts = Bezirke (rajonai, { $source })
permits-section-green-spaces = Žaliosios erdvės ({ $source })
permits-section-owners = Federalinės žemės ir Bund žemė ({ $source })
permits-section-monuments = Paminklai ({ $source })
permits-section-protected-objects = Gamtos paminklai ir kiti saugomi objektai ({ $source })
permits-none-districts = Pagal { $source } žemėlapio plotas nepatenka į jokį Bezirk.
permits-none-green-spaces = Pagal { $source } žemėlapio plote viešųjų žaliųjų erdvių nėra.
permits-none-owners = Pagal { $source } žemėlapio plote nėra nė vieno federalinės žemės ar Bund sklypo.
permits-none-monuments = Pagal { $source } žemėlapio plote paminklų nėra.
permits-none-protected-objects = Pagal { $source } žemėlapio plote nėra nė vieno jo laikomo saugomo objekto.
permits-none-sampled = Paklaustuose taškuose { $source } čia nieko nenurodo.
permits-gaps = { $source } duomenys nepilni: { $gaps }.
permits-sampled = { $source } atsako tik apie pavienius taškus. freemapper klausė taškų tinklelyje žemėlapio plote: dalys yra įverčiai, o to, kas tarp taškų, gali trūkti.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = paskirtis: { $dedication }
permits-playgrounds-none = Žemėlapio plote viešųjų žaidimų aikštelių nėra.
permits-playgrounds-one = Žemėlapio plote yra viena vieša žaidimų aikštelė. Atskirai ji neišvardijama.
permits-playgrounds-many = Žemėlapio plote viešųjų žaidimų aikštelių: { $count }. Jos neišvardijamos po vieną.
permits-owner-other = Kiti viešieji savininkai
permits-owners-rest = Likusiai ploto daliai duomenys savininko nenurodo.
permits-monuments-total = Paminklų, visiškai ar iš dalies žemėlapio plote: { $count }.{ $links ->
        [yes] {" "}Nuorodos veda į jų įrašus paminklų sąraše.
       *[no] {""}
    }
permits-monuments-links-left = Dar { $count } įrašų čia tik suskaičiuota, be nuorodos.
permits-object-kind = { $kind ->
        [natural-monument] Gamtos paminklas (Naturdenkmal)
        [protected-landscape-part] Saugomas kraštovaizdžio elementas (geschützter Landschaftsbestandteil)
        [biotope] Teisiškai saugomas biotopas (geschütztes Biotop)
        [national-park] Nacionalinis parkas (park narodowy)
        [park-buffer-zone] Parko apsaugos zona (otulina)
        [landscape-park] Kraštovaizdžio parkas (park krajobrazowy)
        [nature-reserve] Gamtos rezervatas (rezerwat przyrody)
        [protected-landscape-area] Saugomo kraštovaizdžio teritorija (obszar chronionego krajobrazu)
        [natura2000-habitats] „Natura 2000“ teritorija buveinėms (SOO)
        [natura2000-birds] „Natura 2000“ teritorija paukščiams (OSO)
        [ecological-site] Ekologinė teritorija (użytek ekologiczny)
        [landscape-complex] Gamtos ir kraštovaizdžio kompleksas (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentavimo vieta (stanowisko dokumentacyjne)
        [natural-monument-pl] Gamtos paminklas (pomnik przyrody)
       *[special-rules] Plotas su ypatingomis taisyklėmis
    }
permits-object-count = objektų: { $count }
permits-object-citations = Teisiniai pagrindai:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Gatvėms, aikštėms ir žaliosioms erdvėms Bezirk { $district } paprastai atsakinga Bezirksamt { $district }, ten Fachamt Management des öffentlichen Raumes (viešosios erdvės valdymo skyrius).
       *[berlin] Gatvėms, aikštėms ir žaliosioms erdvėms Bezirk { $district } paprastai atsakingi Bezirksamt { $district } Ordnungsamt (viešosios tvarkos tarnyba) ir Straßen- und Grünflächenamt (kelių ir parkų skyrius).
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Žemėlapio plote esantiems paminklams atsakinga Denkmalschutzamt Hamburg (paminklosaugos tarnyba).
       *[berlin] Bezirk { $district } paminklams atsakinga Bezirksamt { $district } žemesnioji paminklosaugos institucija (untere Denkmalschutzbehörde).
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Ar žemėlapio plote yra saugomų paminklų, čia esantys duomenys tikrai pasakyti negali.
       *[yes] Žemėlapio plote yra saugomų paminklų.
    } Ar bėgimas jų paliečia, pavyzdžiui, kontrolinis punktas prie paminklo ar ruožas per saugomą vietą, sprendžia žemesnioji paminklosaugos institucija (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Čia tai pats miestas { $kreis }.
        [no] , paprastai { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; kai kuriose federalinėse žemėse tai Gemeinde { $gemeinde }.
        }
       *[unknown] {" "}(paprastai Landkreis arba kreisfreie Stadt, kai kuriose federalinėse žemėse Gemeinde).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Žemėlapyje esantys gamtos paminklai, saugomi kraštovaizdžio elementai ir saugomi biotopai priklauso žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde). Čia tai pats miestas { $kreis }.
        [no] Žemėlapyje esantys gamtos paminklai, saugomi kraštovaizdžio elementai ir saugomi biotopai priklauso žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde), paprastai { $kreis }.
       *[unknown] Žemėlapyje esantys gamtos paminklai, saugomi kraštovaizdžio elementai ir saugomi biotopai priklauso žemesniajai gamtos apsaugos institucijai (untere Naturschutzbehörde), paprastai Landkreis arba kreisfreie Stadt.
    }
permits-hint-biotope = Žemėlapio plote yra teisiškai saugomų biotopų{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Viskas, kas gali juos sunaikinti ar jiems žymiai pakenkti, draudžiama (§ 30 BNatSchG). Kontrolinius punktus ir ruožus rinkitės už jų ribų arba klauskite žemesniosios gamtos apsaugos institucijos (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Gamtos paminklams ir saugomiems kraštovaizdžio elementams Bezirk { $district } paprastai atsakinga Bezirksamt { $district }.
       *[berlin] Gamtos paminklams ir saugomiems kraštovaizdžio elementams Bezirk { $district } atsakinga Bezirksamt { $district } žemesnioji gamtos apsaugos institucija (untere Naturschutzbehörde).
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Pagal federalinės žemės atvirus duomenis { $share } žemėlapio ploto priklauso kitiems viešiesiems savininkams.
       *[no] Pagal federalinės žemės atvirus duomenis { $share } žemėlapio ploto priklauso savininkui "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Likusios ploto dalies savininkų iš atvirų duomenų nustatyti negalima. Turėdami teisėtą interesą (pavyzdžiui, kaip organizatorius) galite paprašyti žemės knygos tarnybos (Grundbuchamt) išrašo; dėl viešosios žemės padeda kompetentingas Ordnungsamt arba parkų skyrius.
       *[no] Šių plotų savininkų iš atvirų duomenų nustatyti negalima. Turėdami teisėtą interesą (pavyzdžiui, kaip organizatorius) galite paprašyti žemės knygos tarnybos (Grundbuchamt) išrašo; dėl viešosios žemės padeda kompetentingas Ordnungsamt arba parkų skyrius.
    }
permits-section-forest-units = Miškų administracija ({ $source })
permits-section-state-forest = Federalinės žemės miškas ({ $source })
permits-none-forest-units = Pagal { $source } žemėlapio plotas nepatenka į jokios miškų tarnybos veiklos plotą.
permits-section-admin-units = Administraciniai vienetai ({ $source })
permits-none-admin-units = Pagal { $source } žemėlapio teritorija nepatenka į jokį registro administracinį vienetą.
permits-section-register-monuments = Nacionalinio registro paminklai ({ $source })
permits-none-register-monuments = Pagal { $source } stačiakampyje aplink žemėlapio plotą nėra registruotų paminklų.
permits-register-monuments-total = Registruoti paminklai stačiakampyje aplink žemėlapio plotą: { $count }. Registras nenurodo vietos, todėl šios pastabos negali pasakyti, kur jie yra; tai reikia patikrinti žemėlapyje.
permits-register-monuments-left = Dar { $count } čia tik suskaičiuoti, be pavadinimų.
permits-section-register-archaeology = Nacionalinio registro archeologijos paminklai ({ $source })
permits-none-register-archaeology = Pagal { $source } stačiakampyje aplink žemėlapio plotą nėra registruotų archeologijos paminklų.
permits-register-archaeology-total = Registruoti archeologijos paminklai stačiakampyje aplink žemėlapio plotą: { $count }. Registras nenurodo vietos, todėl šios pastabos negali pasakyti, kur jie yra; tai reikia patikrinti žemėlapyje.
permits-register-archaeology-left = Dar { $count } čia tik suskaičiuoti, be pavadinimų.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Pagal { $source } žemėlapio plote federalinės žemės miško nėra.
permits-forest-units-reach = Miškų tarnyba atsako už savo ploto mišką. Jos plotas gali apimti ir miestus bei laukus.
permits-state-forest-office = atsakinga: { $office }
permits-hint-forest-office = { $units ->
        [none] Už žemėlapio plote esantį mišką atsakinga miškų tarnyba: { $office } ({ $administration }). Planuodami renginį miške, kreipkitės ten anksti.
       *[other] Už žemėlapio plote esantį mišką atsakinga miškų tarnyba: { $office } ({ $administration }), { $units }. Planuodami renginį miške, kreipkitės ten anksti.
    }
permits-hint-forest-owner = { $office ->
        [known] Renginiui miške reikia miško savininkų sutikimo. Miškų tarnyba paprastai gali pasakyti, kam miškas priklauso. Keliose federalinėse žemėse organizuotą renginį miške taip pat reikia pranešti miškų institucijai arba gauti jos leidimą.
       *[unknown] Renginiui miške reikia miško savininkų sutikimo. Keliose federalinėse žemėse organizuotą renginį miške taip pat reikia pranešti miškų institucijai arba gauti jos leidimą. Šios pastabos neįvardija čia atsakingos miškų tarnybos; ją gali pasakyti federalinės žemės miškų administracija arba žemesnioji miškų institucija.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Renginiui miške paprastai reikia miško savininko sutikimo. Aukščiau nurodyta miškų urėdija (Nadleśnictwo) paprastai gali pasakyti, kas miške tvarkosi. Miško, kuris nepriklauso valstybiniams miškams, pavyzdžiui, privataus miško, šių pastabų duomenyse nėra. Miško, kuris nepriklauso valstybei, pavyzdžiui, privataus ar savivaldybės miško, miškų ūkio priežiūrą atlieka starosta (starosta) (Miškų įstatymas, 5 str. 1 d. 2 p.); ten klauskite tarnyboje (starostwo powiatowe).
       *[unknown] Renginiui miške paprastai reikia miško savininko sutikimo. Šios pastabos žemėlapio sričiai nenurodo jokios valstybinių miškų urėdijos; duomenys apima tik valstybinius miškus, todėl miškas gali priklausyti kam nors kitam, pavyzdžiui, privačiam savininkui. Savininką reikia išsiaiškinti atskirai. Miško, kuris nepriklauso valstybei, pavyzdžiui, privataus ar savivaldybės miško, miškų ūkio priežiūrą atlieka starosta (starosta) (Miškų įstatymas, 5 str. 1 d. 2 p.); ten klauskite tarnyboje (starostwo powiatowe).
    }
permits-hint-monument-register-pl = Stačiakampyje aplink žemėlapio plotą yra nacionalinio registro paminklų. Jų vietų duomenyse nėra, todėl žemėlapyje patikrinkite, kur jie yra. Pagal Paminklų apsaugos įstatymo (ustawa o ochronie zabytków) 36 str. 1 dalį konservavimo, restauravimo ir statybos darbams registruotame paminkle reikia vaivadijos paminklų konservatoriaus (Wojewódzki Konserwator Zabytków) leidimo. Nuo 2026 m. birželio 3 d. statybos darbams paminklo aplinkoje, jo paskirties keitimui, ant jo pritvirtintiems įrenginiams ar lentelėms ir kitiems veiksmams, galintiems pakeisti jo esmę ar išvaizdą, reikia tik pranešimo konservatoriui (36aa str.); konservatorius per 60 dienų gali pateikti prieštaravimą ir tada reikalauti leidimo. Įstatymas nemini bėgimo ar punkto kaip tokių; paklauskite konservatoriaus tarnybos, jei renginys liečia paminklą ar jo aplinką.
permits-hint-archaeology-register-pl = Nacionalinio registro archeologijos paminklų yra stačiakampyje aplink žemėlapio plotą. Jų vietų duomenyse nėra, todėl žemėlapyje patikrinkite, kur jie yra. Pagal Paminklų apsaugos įstatymo (ustawa o ochronie zabytków) 36 str. 1 dalį archeologiniams tyrimams (pkt 5), paslėptų ar apleistų kilnojamųjų paminklų paieškai elektroniniais ar techniniais prietaisais, pavyzdžiui, metalo ieškikliais (pkt 12), ir konservavimo, restauravimo ar statybos darbams registruotame paminkle (pkt 1) reikia vaivadijos paminklų konservatoriaus (Wojewódzki Konserwator Zabytków) leidimo. Leidimas paieškoms nustoja galioti 2027 m. sausio 1 d.: 36b str. tada leis pilnamečiam asmeniui ieškoti gavus nekilnojamojo turto savininko ir valdytojo sutikimą ir pranešus paieškų registrui per kultūros ministerijos programėlę, o 36c str. tokias paieškas, be kita ko, draudžia registruotuose paminkluose ir arčiau kaip 10 m nuo jų. Įstatymas nemini pasivaikščiojimo, bėgimo ar kontrolinio punkto kaip tokių; teiraukitės konservatoriaus biure, jei renginys ar lauko darbai liečia registruotą vietą.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Žemėlapio teritorijoje yra gamtos paminklas (pomnik przyrody), ekologinė teritorija (użytek ekologiczny), gamtos ir kraštovaizdžio kompleksas (zespół przyrodniczo-krajobrazowy) arba dokumentavimo vieta (stanowisko dokumentacyjne). Juos steigia gminos taryba sprendimu, kuriame nurodoma, kas prižiūri objektą ir kokie draudimai galioja (Gamtos apsaugos įstatymo 44 straipsnis). Pats įstatymas čia renginiui jokio leidimo nenumato. Paprašykite sprendimo ir prižiūrėtojo gminos, kurios teritorijoje yra objektas, įstaigoje.
       *[other] Žemėlapio teritorijoje yra gamtos paminklas (pomnik przyrody), ekologinė teritorija (użytek ekologiczny), gamtos ir kraštovaizdžio kompleksas (zespół przyrodniczo-krajobrazowy) arba dokumentavimo vieta (stanowisko dokumentacyjne). Juos steigia gminos taryba sprendimu, kuriame nurodoma, kas prižiūri objektą ir kokie draudimai galioja (Gamtos apsaugos įstatymo 44 straipsnis). Pats įstatymas čia renginiui jokio leidimo nenumato. Paprašykite sprendimo ir prižiūrėtojo gminos ({ $gmina }), kurios teritorijoje yra objektas, įstaigoje.
    }
permits-hint-gmina-roads-pl = Renginiui, kuris trukdo eismui arba ypatingai naudoja gatvę, reikia leidimo pagal Kelių eismo įstatymo 65 straipsnį; prašymas teikiamas bent 30 dienų iš anksto. Leidimą išduoda kelyje eismą valdanti institucija: powiato ir gminos keliuose starosta, powiato teises turinčiame mieste miesto prezidentas, kitiems keliams vaivadijos maršalas arba GDDKiA (10 ir 65b straipsniai). Gminos kelio valdytojas yra gminos ({ $gmina }) wójt, burmistrz arba prezydent miasta; valdytojas sprendžia dėl kelio juostos užėmimo (Viešųjų kelių įstatymo 19 ir 40 straipsniai). Šios pastabos nerodo kelių kategorijos.
permits-hint-gmina-land-pl = Jei žemėlapio teritorijos žemė priklauso gminai, pavyzdžiui, miesto parkas, aikštė, mokyklos teritorija ar miškas, ją valdo wójt, burmistrz arba prezydent miasta, kuris atstovauja gminai (Gminos savivaldos įstatymo 30 ir 31 straipsniai). Prieš naudodami paprašykite sutikimo gminos ({ $gmina }) įstaigoje. Šios pastabos nerodo, kam žemė priklauso.
permits-section-water-protection = Vandens apsaugos zonos ({ $source })
permits-section-forest-functions = Miško funkcijos ir apsauginis miškas ({ $source })
permits-none-water-protection = Pagal { $source } žemėlapio plote vandens apsaugos zonų nėra.
permits-none-forest-functions = Pagal { $source } žemėlapio plote nėra nei poilsio miško, nei apsauginio miško, nei gamtinio miško, nei uždaro miško ploto.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Poilsio miškas (Erholungswald)
        [protected] Apsauginis miškas (Bannwald arba Schutzwald)
        [natural] Gamtinis miškas arba griežtas rezervatas (Naturwald, Totalreservat)
       *[closed] Uždaras miško plotas
    }
permits-hint-water-protection = { $inner ->
        [yes] Vandens apsaugos zonos "{ $name }" { $zone } yra žemėlapio plote. I ir II zonose renginiai paprastai neįmanomi. Klauskite žemesniosios vandens institucijos (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] Vandens apsaugos zonos "{ $name }" { $zone } yra žemėlapio plote. Renginiai ten paprastai galimi; žemesnioji vandens institucija (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) gali pasakyti, ką leidžia nuostatai.
       *[unknown] Vandens apsaugos zona "{ $name }" yra žemėlapio plote. Duomenys nenurodo, kokia tai zona. I ir II zonose renginiai paprastai neįmanomi; klauskite žemesniosios vandens institucijos (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Žemėlapio plote yra poilsio miško (Erholungswald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Renginiams ten klauskite miškų institucijos.
        [protected] Žemėlapio plote yra apsauginio miško (Bannwald arba Schutzwald){ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Prieš planuodami renginį ten, paklauskite miškų institucijos.
        [natural] Žemėlapio plote yra gamtinio miško arba griežto rezervato{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Išeiti iš takų ten dažnai draudžiama; nedėkite ten kontrolinių punktų, neatsiklausę miškų institucijos.
       *[closed] Žemėlapio plote yra miško plotas, į kurį įeiti negalima{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Neįtraukite jo į trasą.
    }
permits-section-roads = Klasifikuoti keliai pagal OpenStreetMap
permits-none-roads = Pagal OpenStreetMap per žemėlapio plotą neina nei Autobahn, nei Bundes-, Landes-, Staats- ar Kreisstraße.
permits-road-line = { $class ->
        [autobahn] Autobahn (automagistralė)
        [bundesstrasse] Bundesstraße (federalinis kelias)
        [landesstrasse] Landesstraße arba Staatsstraße (federalinės žemės kelias)
       *[kreisstrasse] Kreisstraße (apskrities kelias)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Per žemėlapio plotą eina Autobahn { $refs }. Trasa gali ją kirsti tik tiltais arba pravažiavimais po ja.
        [bundesstrasse] Per žemėlapio plotą eina Bundesstraße { $refs }. Uždarymą arba kirtimą su pagalbininkais nurodo kelių eismo institucija (Straßenverkehrsbehörde); kelią paprastai prižiūri Bund per federalinės žemės kelių administraciją, didesniuose miestuose dažnai miestas.
        [landesstrasse] Per žemėlapio plotą eina Landesstraße arba Staatsstraße { $refs }. Uždarymą arba kirtimą su pagalbininkais nurodo kelių eismo institucija (Straßenverkehrsbehörde); kelią paprastai prižiūri federalinė žemė, didesniuose miestuose dažnai miestas.
       *[kreisstrasse] Per žemėlapio plotą eina Kreisstraße { $refs }. Uždarymą arba kirtimą su pagalbininkais nurodo kelių eismo institucija (Straßenverkehrsbehörde); kelią paprastai prižiūri Kreis, didesniuose miestuose dažnai miestas.
    }
permits-hint-state-forest = { $office ->
        [none] Pagal federalinės žemės atvirus duomenis { $share } žemėlapio ploto yra federalinės žemės miškas, savininkas "{ $owner }".
       *[other] Pagal federalinės žemės atvirus duomenis { $share } žemėlapio ploto yra federalinės žemės miškas, savininkas "{ $owner }". Atsakinga: { $office }.
    }
